use crate::{Cx, Key, Navigation, Outlet, PathRouter, Result, Route, RoutePath, Router, View};

/// A router borrowed for the current actor turn. History still belongs to the host.
/// No context or guard callback is retained by navigation.
///
/// ```compile_fail
/// use blockui::{BoundPathRouter, Cx, PathRouter, RoutePath};
/// fn escape<R: RoutePath>(router: PathRouter<R>, cx: &Cx<'_>)
///     -> BoundPathRouter<'static, 'static, R> {
///     router.bind(cx)
/// }
/// ```
/// ```compile_fail
/// use blockui::{Cx, PathRouter, RoutePath};
/// fn send<R: RoutePath + Send>(router: PathRouter<R>, cx: &Cx<'_>) {
///     let router = router.bind(cx);
///     std::thread::scope(|scope| { scope.spawn(move || router.current()); });
/// }
/// ```
pub struct BoundPathRouter<'cx, 'host, R> {
    router: PathRouter<R>,
    cx: &'cx Cx<'host>,
}

/// A finite typed router borrowed for the current actor turn.
///
/// ```compile_fail
/// use blockui::{BoundRouter, Cx, Route, Router};
/// fn escape<R: Route>(router: Router<R>, cx: &Cx<'_>)
///     -> BoundRouter<'static, 'static, R> {
///     router.bind(cx)
/// }
/// ```
pub struct BoundRouter<'cx, 'host, R> {
    router: Router<R>,
    cx: &'cx Cx<'host>,
}

impl<R: Route> Router<R> {
    /// Borrow the current turn for context-free navigation methods.
    pub fn bind<'cx, 'host>(self, cx: &'cx Cx<'host>) -> BoundRouter<'cx, 'host, R> {
        BoundRouter { router: self, cx }
    }
}
impl<R: RoutePath> PathRouter<R> {
    /// Borrow the current turn for context-free navigation methods.
    pub fn bind<'cx, 'host>(self, cx: &'cx Cx<'host>) -> BoundPathRouter<'cx, 'host, R> {
        BoundPathRouter { router: self, cx }
    }
}

// Both adapters preserve the descriptor methods' validation and transaction rules.
macro_rules! navigation_methods {
    () => {
        pub fn current(&self) -> Result<R> {
            self.router.current(self.cx)
        }
        pub fn push(&self, route: R) -> Result<()> {
            self.router.push(self.cx, route)
        }
        pub fn replace(&self, route: R) -> Result<()> {
            self.router.replace(self.cx, route)
        }
        pub fn can_back(&self) -> Result<bool> {
            self.router.can_back(self.cx)
        }
        pub fn can_forward(&self) -> Result<bool> {
            self.router.can_forward(self.cx)
        }
        pub fn back(&self) -> Result<bool> {
            self.router.back(self.cx)
        }
        pub fn forward(&self) -> Result<bool> {
            self.router.forward(self.cx)
        }
        /// The synchronous guard is called at most once and never retained.
        /// See [`Router::navigate`] for denial and rollback semantics.
        pub fn navigate(
            &self,
            request: Navigation<R>,
            guard: impl FnOnce(&R, &R) -> Result<bool>,
        ) -> Result<bool> {
            self.router.navigate(self.cx, request, guard)
        }
    };
}
impl<R: Route> BoundRouter<'_, '_, R> {
    navigation_methods!();
    /// Project the current route inside a page/focus boundary. Page identity is
    /// the supplied namespace scoped by the stable route ID. The render closure
    /// runs once in this turn and is never retained by the host.
    ///
    /// ```
    /// use blockui::{BoundRouter, Route, Result, View, Text, IntoView};
    /// fn page<R: Route>(router: &BoundRouter<'_, '_, R>) -> Result<View> {
    ///     router.outlet("main.pages", |_| Ok(Text::new("Page").into_view()))
    /// }
    /// ```
    pub fn outlet(
        &self,
        namespace: impl Into<Key>,
        render: impl FnOnce(R) -> Result<View>,
    ) -> Result<View> {
        let route = self.current()?;
        let key = namespace.into().scoped(route.id() as u64);
        Ok(Outlet::new(key).children(vec![render(route)?]))
    }
}
impl<R: RoutePath> BoundPathRouter<'_, '_, R> {
    navigation_methods!();
    /// Project the current route inside a page/focus boundary. The canonical
    /// path (including parameters) scopes the supplied namespace, so separate
    /// documents get separate renderer identities without source-location keys.
    /// Changing path encoding changes page identity; history still requires its
    /// normal schema migration. Durable drafts belong to actor state. Renderer
    /// restoration requires bounded host retention. The closure runs once in
    /// this turn and is never retained across implementation replacement.
    pub fn outlet(
        &self,
        namespace: impl Into<Key>,
        render: impl FnOnce(R) -> Result<View>,
    ) -> Result<View> {
        let route = self.current()?;
        let key = namespace.into().scoped_named(&route.to_path());
        Ok(Outlet::new(key).children(vec![render(route)?]))
    }
    pub fn push_path(&self, path: &str) -> Result<()> {
        self.router.push_path(self.cx, path)
    }
}
