#![cfg(feature = "testing")]
use blockui::*;

#[component]
struct LocalCounter {
    count: State<i64>,
}
#[component_impl]
impl LocalCounter {
    fn migrate(&self, _: &Cx<'_>, previous: u64) -> Result<()> {
        if previous != 0 {
            return Err(Error::Application(
                "unchanged schema must not migrate".into(),
            ));
        }
        Ok(())
    }
    #[action(bound)]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 1))
    }
    #[computed(bound)]
    fn double(&self) -> Result<i64> {
        Ok(self.count.get() * 2)
    }
    #[scope(bound)]
    #[blockui::view]
    fn detail(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.double_computed());
        }
    }
    #[blockui::view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            self.detail_scope();
        }
    }
}
// Legacy declaration mode keeps the historical flat identity contract.
#[actor]
struct Panel {
    counter: LocalCounter,
    #[cfg(any())]
    absent: MissingComponent,
}
#[component_impl]
impl Panel {
    fn view(&self) -> View {
        self.counter.render()
    }
}
#[actor]
struct App {
    panel: Panel,
}
#[actor_impl(render)]
impl App {
    #[action]
    fn idle(&self, _: &Cx<'_>) -> Result<()> {
        Ok(())
    }
}
impl Render for AppBound<'_, '_> {
    fn render(&self) -> View {
        self.panel.render()
    }
}

#[test]
fn nested_components_register_behavior_without_child_actors() -> std::result::Result<(), String> {
    assert_eq!(<App as Actor>::STATE.len(), 1);
    assert_eq!(<App as Actor>::ACTIONS.len(), 2);
    assert_eq!(<App as Actor>::SCOPES.len(), 1);
    assert_eq!(<App as Actor>::COMPUTEDS.len(), 1);
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    assert_eq!(actor.integer("LocalCounter.count"), 0);
    let state = actor.state_address();
    let slot = actor.slot_address("LocalCounter.count");
    let evaluations = actor.computed_evaluations("LocalCounter.double");
    actor.tell(LocalCounter::actions().increment, 0)?;
    actor.tick()?;
    assert_eq!(actor.integer("LocalCounter.count"), 1);
    assert!(actor.computed_evaluations("LocalCounter.double") > evaluations);
    actor.tell(LocalCounter::actions().increment, 0)?;
    actor.replace_for_test::<App>()?;
    actor.tick()?;
    assert_eq!(actor.integer("LocalCounter.count"), 2);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("LocalCounter.count"), slot);
    Ok(())
}

#[component]
struct RejectCounter {
    #[state(key = "LocalCounter.count")]
    count: State<i64>,
    added: State<bool>,
}
#[component_impl]
impl RejectCounter {
    fn migrate(&self, cx: &Cx<'_>, _: u64) -> Result<()> {
        self.count.set(cx, 999)?;
        Err(Error::Application("rejected component migration".into()))
    }
    fn view(&self) -> View {
        Text::new(&self.count).into_view()
    }
}
#[actor]
struct RejectedApp {
    child: RejectCounter,
}
#[actor_impl(render)]
impl RejectedApp {}
impl Render for RejectedAppBound<'_, '_> {
    fn render(&self) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(self.child.render());
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn rejected_component_migration_rolls_back_and_keeps_old_handler() -> std::result::Result<(), String>
{
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    actor.tell(LocalCounter::actions().increment, 0)?;
    actor.tick()?;
    let state = actor.state_address();
    let slot = actor.slot_address("LocalCounter.count");
    let error = actor.replace_for_test::<RejectedApp>().unwrap_err();
    assert!(error.contains("rejected component migration"), "{error}");
    assert_eq!(actor.integer("LocalCounter.count"), 1);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("LocalCounter.count"), slot);
    assert_eq!(actor.slot_address("RejectCounter.added"), 0);
    actor.tell(LocalCounter::actions().increment, 0)?;
    actor.tick()?;
    assert_eq!(actor.integer("LocalCounter.count"), 2);
    Ok(())
}

#[actor]
struct DuplicateApp {
    first: LocalCounter,
    second: LocalCounter,
}
#[actor_impl(render)]
impl DuplicateApp {}
impl Render for DuplicateAppBound<'_, '_> {
    fn render(&self) -> View {
        self.first.render()
    }
}
#[test]
fn duplicate_component_identities_are_rejected() {
    assert!(blockui_runtime::Actor::for_test::<DuplicateApp>().is_err());
}

#[component]
struct EvolvedCounter {
    #[state(key = "LocalCounter.count")]
    count: State<i64>,
    #[state(default = true)]
    enabled: State<bool>,
}
#[component_impl]
impl EvolvedCounter {
    #[action(bound, name = "LocalCounter.increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 10))
    }
    fn view(&self) -> View {
        Text::new(&self.count).into_view()
    }
}
#[actor]
struct EvolvedApp {
    renamed_panel: EvolvedCounter,
}
#[actor_impl(render)]
impl EvolvedApp {}
impl Render for EvolvedAppBound<'_, '_> {
    fn render(&self) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(self.renamed_panel.render());
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn component_schema_addition_preserves_slots_and_queued_action_identity()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    actor.tell(LocalCounter::actions().increment, 0)?;
    actor.tick()?;
    let state = actor.state_address();
    let slot = actor.slot_address("LocalCounter.count");
    actor.tell(LocalCounter::actions().increment, 0)?;
    actor.replace_for_test::<EvolvedApp>()?;
    assert_eq!(actor.integer("LocalCounter.count"), 1);
    assert!(actor.boolean("EvolvedCounter.enabled"));
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("LocalCounter.count"), slot);
    actor.tick()?;
    assert_eq!(actor.integer("LocalCounter.count"), 11);
    Ok(())
}

#[actor]
struct FlatApp {
    #[state(key = "LocalCounter.count")]
    count: State<i64>,
}
#[actor_impl(render)]
impl FlatApp {
    #[action(bound, name = "LocalCounter.increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 1))
    }
}
impl Render for FlatAppBound<'_, '_> {
    fn render(&self) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(&self.count));
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn moving_flat_state_into_components_does_not_change_schema_or_run_migration()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<FlatApp>()?;
    actor.tell(FlatApp::actions().increment, 0)?;
    actor.tick()?;
    let schema = actor.schema_version;
    let state = actor.state_address();
    let slot = actor.slot_address("LocalCounter.count");
    actor.replace_for_test::<App>()?;
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("LocalCounter.count"), slot);
    assert_eq!(actor.integer("LocalCounter.count"), 1);
    actor.tell(LocalCounter::actions().increment, 0)?;
    actor.tick()?;
    assert_eq!(actor.integer("LocalCounter.count"), 2);
    Ok(())
}
