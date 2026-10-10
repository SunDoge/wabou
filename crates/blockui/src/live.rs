//! Typed, lazy computations mounted by individual widget properties.
use crate::{Binding, Computed, Cx, IntoView, Key, Result, Scope, View, ViewStyle};
use std::collections::BTreeSet;
use std::rc::Rc;

type Compute<T> = dyn for<'a> Fn(&Cx<'a>) -> Result<T>;
type ItemCompute<T> = dyn for<'a> Fn(&Cx<'a>, String) -> Result<T>;

fn validate_keys(keys: &[String]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for key in keys {
        if key.is_empty() || !seen.insert(key) {
            return Err(crate::Error::InvalidView(
                "list keys must be nonempty and unique",
            ));
        }
    }
    Ok(())
}

/// A lazy row builder owned by the current implementation.
#[must_use = "pass this builder to a computed key list's each method"]
pub struct LiveEach<T> {
    compute: Rc<ItemCompute<T>>,
    identity: Rc<()>,
}
impl<T: 'static> LiveEach<T> {
    /// Capture owned values and use a fresh context for each business key.
    pub fn captured(compute: impl for<'a> Fn(&Cx<'a>, String) -> Result<T> + 'static) -> Self {
        Self {
            compute: Rc::new(compute),
            identity: Rc::new(()),
        }
    }
}

impl Computed<Vec<String>> {
    /// Build keyed rows in the computed list's order. Mount the result to observe
    /// membership separately from row properties. Equal key lists suppress list
    /// rebuilds; membership changes rebuild row descriptions with stable identities.
    pub fn each<T: IntoView + 'static>(self, rows: LiveEach<T>) -> Live<View> {
        Live::captured(move |cx| {
            let keys = self.get(cx)?;
            validate_keys(&keys)?;
            let mut children = Vec::with_capacity(keys.len());
            for key in keys {
                let identity = Key::named("identity.each.item").scoped(crate::abi::key(&key));
                let row = (rows.compute)(cx, key)?.into_view();
                children.push(crate::__component_view(
                    row,
                    crate::InstancePath::EMPTY.scoped(identity),
                    Key::named("identity.each.root"),
                ));
            }
            Ok(crate::Column::default().children(children))
        })
    }

    /// Mount each keyed row in its own structural observer. Direct reads in a
    /// row builder invalidate only that row, not the parent list or siblings.
    /// Reordering reuses admitted row callbacks from the same builder. Changing
    /// the builder identity invalidates them; removal prunes them on commit.
    pub fn each_scoped<T: IntoView + 'static>(self, rows: LiveEach<T>) -> Live<View> {
        Live::captured(move |cx| {
            let keys = self.get(cx)?;
            validate_keys(&keys)?;
            let mut children = Vec::with_capacity(keys.len());
            for key in keys {
                let identity = Key::named("identity.each.item").scoped(crate::abi::key(&key));
                let compute = rows.compute.clone();
                children.push(Scope::captured_row_view(
                    identity,
                    Rc::new(move |cx| Ok(compute(cx, key.clone())?.into_view())),
                    rows.identity.clone(),
                ));
            }
            Ok(crate::Column::default().children(children))
        })
    }
}

/// A lazy property computation. Reads subscribe in the observer that mounts it,
/// never in the view that constructs it. This value belongs to the current
/// implementation; it must not escape into host tasks or cross the plugin ABI.
#[must_use = "mount this computation in a widget property to observe its reads"]
pub struct Live<T> {
    pub(crate) compute: Rc<Compute<T>>,
}

impl<T> Clone for Live<T> {
    fn clone(&self) -> Self {
        Self {
            compute: self.compute.clone(),
        }
    }
}

impl<T: 'static> Live<T> {
    /// Capture owned descriptors or values, and read with each fresh context.
    pub fn captured(compute: impl for<'a> Fn(&Cx<'a>) -> Result<T> + 'static) -> Self {
        Self {
            compute: Rc::new(compute),
        }
    }

    /// Transform lazily in the mounting observer. This does not introduce a
    /// memo node: source reads and transform reads belong to that observer.
    pub fn map<U: 'static>(self, transform: impl Fn(T) -> U + 'static) -> Live<U> {
        Live::captured(move |cx| Ok(transform((self.compute)(cx)?)))
    }

    /// A fallible lazy transform. Errors propagate to the actor turn so its
    /// candidate state and projection are rolled back together.
    pub fn try_map<U: 'static>(self, transform: impl Fn(T) -> Result<U> + 'static) -> Live<U> {
        Live::captured(move |cx| transform((self.compute)(cx)?))
    }
}

impl<T: Into<String> + 'static> From<Live<T>> for Binding {
    fn from(value: Live<T>) -> Self {
        Self::captured(move |cx| Ok((value.compute)(cx)?.into()))
    }
}

impl<T: IntoView + 'static> Live<T> {
    /// Mount this computation as an independently rebuilt structural scope.
    /// The stable key namespaces child identities. Returning `View::empty()`
    /// unmounts content and its callbacks while keeping actor-owned state.
    /// Reads in nested live properties remain independent of structural reads.
    pub fn mount(self, key: impl Into<Key>) -> View {
        Scope::reactive(key, move |cx| Ok((self.compute)(cx)?.into_view()))
    }
}

impl<T: IntoView + 'static, E: 'static> Live<std::result::Result<T, E>> {
    /// Select content or a fallback from a typed read result. Mount this value
    /// to isolate structural updates. The fallback executes only on Err;
    /// nested property reads stay in their own observers. Runtime errors from
    /// the outer lazy computation still propagate and roll back the turn.
    pub fn boundary<U: IntoView + 'static>(
        self,
        fallback: impl Fn(E) -> U + 'static,
    ) -> Live<View> {
        Live::captured(move |cx| {
            Ok(match (self.compute)(cx)? {
                Ok(content) => content.into_view(),
                Err(error) => fallback(error).into_view(),
            })
        })
    }
}

impl Computed<bool> {
    /// Select one lazy branch through a memoized boolean condition.
    /// Only the selected branch executes and contributes structural reads.
    /// Mount the result to create the boundary; branch roots may have different
    /// types as long as both implement IntoView. Switching branches unmounts
    /// the old content, rather than retaining it in an offscreen cache.
    pub fn choose<T: IntoView + 'static, U: IntoView + 'static>(
        self,
        content: Live<T>,
        fallback: Live<U>,
    ) -> Live<View> {
        Live::captured(move |cx| {
            if self.get(cx)? {
                Ok((content.compute)(cx)?.into_view())
            } else {
                Ok((fallback.compute)(cx)?.into_view())
            }
        })
    }

    /// Select content through a memoized condition, without reading it yet.
    /// Mount the result to create a structural boundary. Changes to the memo's
    /// inputs that leave its boolean output equal do not rebuild that boundary.
    /// Content's own structural reads can still rebuild it; nested live
    /// properties remain independent. A false condition unmounts the content.
    pub fn when<T: IntoView + 'static>(self, content: Live<T>) -> Live<View> {
        Live::captured(move |cx| {
            if self.get(cx)? {
                Ok((content.compute)(cx)?.into_view())
            } else {
                Ok(View::empty())
            }
        })
    }
}

/// Static values, partial overrides, or independently observed dynamic styles.
/// Full styles replace values; patches override explicit fields, and tuple layers
/// compose left to right in one independently tracked style observer.
pub enum StyleBinding {
    Static(ViewStyle),
    Live(Live<ViewStyle>),
    Patch(crate::StylePatch),
    LivePatch(Live<crate::StylePatch>),
    Layers(Vec<StyleBinding>),
}

impl From<ViewStyle> for StyleBinding {
    fn from(value: ViewStyle) -> Self {
        Self::Static(value)
    }
}

impl From<Live<ViewStyle>> for StyleBinding {
    fn from(value: Live<ViewStyle>) -> Self {
        Self::Live(value)
    }
}
