#![cfg(feature = "testing")]
use blockui::*;

#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum Original {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/documents/{id}", guard = id > 0)]
    Document { id: i64 },
}
#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum Revised {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/docs/{id}", guard = id > 0)]
    Document { id: i64 },
}

#[actor(name = "automatic.routes")]
struct Old {
    #[state(route = Original)]
    history: State<Vec<String>>,
    counter: State<i64>,
}
#[actor_impl]
impl Old {
    #[action]
    fn open(&self, cx: &Cx<'_>, id: i64) -> Result<()> {
        PathRouter::new(self.history)
            .bind(cx)
            .push(Original::Document { id })
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        // Deliberately never reads history: compatibility must not depend on rendering it.
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(self.counter.get(cx)?.to_string()).key("count"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "automatic.routes")]
struct Changed {
    #[state(route = Revised)]
    history: State<Vec<String>>,
    counter: State<i64>,
}
#[actor_impl]
impl Changed {
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(self.counter.get(cx)?.to_string()).key("count"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "automatic.routes")]
struct ForgottenMigration {
    #[state(route = Revised)]
    history: State<Vec<String>>,
    counter: State<i64>,
}
#[actor_impl]
impl ForgottenMigration {
    fn migrate(&self, _cx: &Cx<'_>, _previous: u64) -> Result<()> {
        Ok(())
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(self.counter.get(cx)?.to_string()).key("count"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "automatic.routes")]
struct Upgraded {
    #[state(route = Revised)]
    history: State<Vec<String>>,
    counter: State<i64>,
}
#[actor_impl]
impl Upgraded {
    fn migrate(&self, cx: &Cx<'_>, previous: u64) -> Result<()> {
        if previous != 0 {
            let mut history = self.history.previous(cx)?;
            for path in history.iter_mut().skip(1) {
                if let Some(id) = path.strip_prefix("/documents/") {
                    *path = format!("/docs/{id}");
                }
            }
            self.history.set(cx, &history)?;
        }
        Ok(())
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(self.counter.get(cx)?.to_string()).key("count"));
            }
            __blockui_tree.finish()
        })
    }
}

#[actor(name = "automatic.parent")]
struct Parent {
    #[state(record, scope = "pane")]
    child: Old,
}
#[actor_impl]
impl Parent {
    #[action]
    fn open(&self, cx: &Cx<'_>, id: i64) -> Result<()> {
        self.child.open(cx, id)
    }
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Parent"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "automatic.parent")]
struct BadParent {
    #[state(record, scope = "pane")]
    child: ForgottenMigration,
}
#[actor_impl]
impl BadParent {
    fn migrate(&self, _cx: &Cx<'_>, _previous: u64) -> Result<()> {
        Ok(())
    }
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Parent"));
            }
            __blockui_tree.finish()
        })
    }
}

#[test]
fn post_migration_validation_descends_into_scoped_unrendered_history()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Parent>()?;
    actor.dispatch(blockui_runtime::Message {
        action: Parent::<DescriptorMode>::open_action().id,
        value: 42,
    })?;
    let state = actor.state_address();
    let schema = actor.schema_version;
    actor.replace_for_test::<Parent>()?;
    let error = actor.replace_for_test::<BadParent>().unwrap_err();
    assert!(error.contains("Invalid route history"), "{error}");
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.schema_version, schema);
    actor.dispatch(blockui_runtime::Message {
        action: Parent::<DescriptorMode>::open_action().id,
        value: 73,
    })?;
    Ok(())
}

#[test]
fn unrendered_route_history_requires_migration_and_preserves_identity_on_rejection()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Old>()?;
    actor.dispatch(blockui_runtime::Message {
        action: Old::<DescriptorMode>::open_action().id,
        value: 42,
    })?;
    let schema = actor.schema_version;
    let state = actor.state_address();
    let slot = actor.slot_address("automatic.routes.history");
    let buffer = actor.text_list("automatic.routes.history").as_ptr();
    actor.replace_for_test::<Old>()?;
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text_list("automatic.routes.history").as_ptr(), buffer);
    let error = actor.replace_for_test::<Changed>().unwrap_err();
    assert!(error.contains("Route format changed"), "{error}");
    assert!(actor.replace_for_test::<ForgottenMigration>().is_err());
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("automatic.routes.history"), slot);
    assert_eq!(actor.text_list("automatic.routes.history").as_ptr(), buffer);
    assert_eq!(
        actor.text_list("automatic.routes.history"),
        &["1", "/", "/documents/42"]
    );
    actor.replace_for_test::<Upgraded>()?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("automatic.routes.history"), slot);
    assert_eq!(
        actor.text_list("automatic.routes.history"),
        &["1", "/", "/docs/42"]
    );
    Ok(())
}

#[actor(name = "pending.routes")]
struct PendingOld {
    #[state(route_request = Original)]
    request: State<String>,
}
#[actor_impl]
impl PendingOld {
    #[action]
    fn defer(&self, cx: &Cx<'_>, id: i64) -> Result<()> {
        self.request.set(cx, &Original::Document { id }.to_path())
    }
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Hidden request"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "pending.routes")]
struct PendingDefault {
    #[state(route_request = Revised)]
    request: State<String>,
}
#[actor_impl]
impl PendingDefault {
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Hidden request"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "pending.routes")]
struct PendingBad {
    #[state(route_request = Revised)]
    request: State<String>,
}
#[actor_impl]
impl PendingBad {
    fn migrate(&self, _cx: &Cx<'_>, _previous: u64) -> Result<()> {
        Ok(())
    }
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Hidden request"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "pending.routes")]
struct PendingGood {
    #[state(route_request = Revised)]
    request: State<String>,
}
#[actor_impl]
impl PendingGood {
    fn migrate(&self, cx: &Cx<'_>, previous: u64) -> Result<()> {
        if previous != 0 {
            let request = self.request.previous(cx)?;
            if let Some(id) = request.strip_prefix("/documents/") {
                self.request.set(cx, &format!("/docs/{id}"))?;
            }
        }
        Ok(())
    }
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Hidden request"));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn hidden_pending_request_requires_valid_migration() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<PendingOld>()?;
    actor.dispatch(blockui_runtime::Message {
        action: PendingOld::<DescriptorMode>::defer_action().id,
        value: 42,
    })?;
    let state = actor.state_address();
    let slot = actor.slot_address("pending.routes.request");
    let schema = actor.schema_version;
    let buffer = actor.text("pending.routes.request").as_ptr();
    let error = actor.replace_for_test::<PendingDefault>().unwrap_err();
    assert!(error.contains("Route format changed"), "{error}");
    assert!(actor.replace_for_test::<PendingBad>().is_err());
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text("pending.routes.request").as_ptr(), buffer);
    assert_eq!(actor.text("pending.routes.request"), "/documents/42");
    actor.replace_for_test::<PendingGood>()?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("pending.routes.request"), slot);
    assert_eq!(actor.text("pending.routes.request"), "/docs/42");
    Ok(())
}
