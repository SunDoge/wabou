#![cfg(feature = "testing")]
use blockui::*;
#[derive(Record, Default)]
#[record(name = "declared.document")]
struct Document {
    title: String,
    count: i64,
}
#[actor(name = "declared")]
struct Model {
    #[state(default = Document { title: "Draft 🦀".into(), count: 7 })]
    document: State<Document>,
}
#[actor_impl]
impl Model {
    #[action]
    fn edit(&self, cx: &Cx<'_>, _: i64) -> Result<()> {
        self.document.bind(cx)?.update(|document| {
            document.count += 1;
            Ok(())
        })
    }
    #[action(name = "declared.reentrant")]
    fn reentrant(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        let original = self.document.get(cx)?;
        let edit = |document: &mut Document| {
            self.document.set(
                cx,
                &Document {
                    title: "Inner write".into(),
                    count: 100,
                },
            )?;
            if mode == 2 {
                self.document.set(cx, &original)?;
            }
            document.count += 1;
            Ok(())
        };
        if mode == 1 {
            self.document.bind(cx)?.update(edit)
        } else {
            self.document.update(cx, edit)
        }
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let document = self.document.bind(cx)?.get()?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(format!("{}: {}", document.title, document.count)).key("value"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "declared")]
struct Replacement {
    #[state(default = Document { title: "Different default".into(), count: 999 })]
    document: State<Document>,
}
#[actor_impl]
impl Replacement {
    #[action(name = "declared.edit")]
    fn edit(&self, cx: &Cx<'_>, _: i64) -> Result<()> {
        self.document.bind(cx)?.update(|document| {
            document.count += 10;
            Ok(())
        })
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let document = self.document.get(cx)?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(format!("{}: {}", document.title, document.count)).key("value"));
            }
            __blockui_tree.finish()
        })
    }
}
#[derive(Record, Default)]
#[record(name = "declared.document", previous = Document)]
struct RevisedDocument {
    title: String,
    count: i64,
    #[record(default)]
    archived: bool,
}
#[derive(Record, Default)]
#[record(name = "declared.document")]
struct UnsupportedDocument {
    title: String,
    count: i64,
    archived: bool,
}
#[actor(name = "declared")]
struct Automatic {
    document: State<RevisedDocument>,
}
#[actor_impl]
impl Automatic {
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let document = self.document.get(cx)?;
        assert!(!document.archived);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(format!("{}: {}", document.title, document.count)).key("value"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "declared")]
struct Missing {
    document: State<UnsupportedDocument>,
}
#[actor_impl]
impl Missing {
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Unrendered document"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "declared")]
struct Noop {
    document: State<RevisedDocument>,
}
#[actor_impl]
impl Noop {
    fn migrate(&self, _cx: &Cx<'_>, _previous: u64) -> Result<()> {
        Ok(())
    }
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Unrendered document"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "declared")]
struct Upgraded {
    document: State<RevisedDocument>,
}
#[actor_impl]
impl Upgraded {
    fn migrate(&self, cx: &Cx<'_>, previous: u64) -> Result<()> {
        if previous != 0 {
            self.document.set(
                cx,
                &RevisedDocument::upgrade(self.document.previous::<Document>(cx)?)?,
            )?;
        }
        Ok(())
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let document = self.document.get(cx)?;
        assert!(!document.archived);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(format!("{}: {}", document.title, document.count)).key("value"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "declared")]
struct Scalar {
    document: State<i64>,
}
#[actor_impl]
impl Scalar {
    fn view(&self, _cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Scalar"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "declared")]
struct Additional {
    document: State<Document>,
    extra: State<Document>,
}
#[actor_impl]
impl Additional {
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let original = self.document.get(cx)?;
        let extra = self.extra.get(cx)?;
        assert_eq!(extra.title, "");
        assert_eq!(extra.count, 0);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(format!("{}: {}", original.title, original.count)).key("value"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "container")]
struct Container {
    #[state(record, scope = "pane")]
    model: Model,
}
#[actor_impl]
impl Container {
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let document = self.model.document.get(cx)?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(format!("{}: {}", document.title, document.count)).key("value"));
            }
            __blockui_tree.finish()
        })
    }
}
#[derive(Default)]
struct InvalidDefault;
impl Record for InvalidDefault {
    const FORMAT: &'static str = "invalid.default";
    fn encode(&self) -> Result<String> {
        Err(Error::Application("default record rejected".into()))
    }
    fn decode(_: &str) -> Result<Self> {
        Ok(Self)
    }
}
#[actor(name = "declared")]
struct BadDefault {
    document: State<Document>,
    new_field: State<InvalidDefault>,
}
#[actor_impl]
impl BadDefault {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Never admitted"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "container")]
struct ContainerAutomatic {
    #[state(record, scope = "pane")]
    model: Automatic,
}
#[actor_impl]
impl ContainerAutomatic {
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let document = self.model.document.get(cx)?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(format!("{}: {}", document.title, document.count)).key("value"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "pair")]
struct Pair {
    first: State<Document>,
    second: State<Document>,
}
#[actor_impl]
impl Pair {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Pair"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor(name = "pair")]
struct PartialPair {
    first: State<RevisedDocument>,
    second: State<UnsupportedDocument>,
}
#[actor_impl]
impl PartialPair {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Pair"));
            }
            __blockui_tree.finish()
        })
    }
}
#[derive(Record, Default)]
#[record(name = "declared.foreign")]
struct ForeignDocument {
    title: String,
    count: i64,
}
#[actor(name = "declared")]
struct Foreign {
    document: State<ForeignDocument>,
}
#[actor_impl]
impl Foreign {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Foreign"));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn declared_records_initialize_once_and_validate_hidden_migration()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let state = actor.state_address();
    let slot = actor.slot_address("declared.document");
    let buffer = actor.text("declared.document").as_ptr();
    let schema = actor.schema_version;
    actor.replace_for_test::<Replacement>()?;
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text("declared.document").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 7");
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("declared.edit"),
        value: 0,
    })?;
    let before = actor.text("declared.document").to_owned();
    assert!(actor.replace_for_test::<BadDefault>().is_err());
    assert!(actor.replace_for_test::<Missing>().is_err());
    assert!(actor.replace_for_test::<Noop>().is_err());
    assert_eq!(actor.text("declared.document"), before);
    actor.replace_for_test::<Upgraded>()?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("declared.document"), slot);
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 17");
    let mut scalar = blockui_runtime::Actor::for_test::<Scalar>()?;
    assert!(scalar.replace_for_test::<Model>().is_err());
    Ok(())
}

#[test]
fn default_new_fields_and_scoped_records() -> std::result::Result<(), String> {
    let cold = blockui_runtime::Actor::for_test::<Upgraded>()?;
    assert_eq!(cold.view.bindings[&abi::key("value")].text, ": 0");
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let buffer = actor.text("declared.document").as_ptr();
    actor.replace_for_test::<Additional>()?;
    assert_eq!(actor.text("declared.document").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 7");
    let scoped = blockui_runtime::Actor::for_test::<Container>()?;
    assert_eq!(scoped.view.bindings[&abi::key("value")].text, "Draft 🦀: 7");
    Ok(())
}

mod selected_fields {
    use blockui::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static TITLES: AtomicUsize = AtomicUsize::new(0);
    static FORMATS: AtomicUsize = AtomicUsize::new(0);
    #[derive(Record, Default)]
    #[record(name = "selected.document")]
    struct Document {
        title: String,
        count: i64,
    }
    #[actor(name = "selected")]
    struct Model {
        document: State<Document>,
    }
    #[actor_impl]
    impl Model {
        #[action]
        fn count(&self, cx: &Cx<'_>) -> Result<()> {
            self.document.update(cx, |value| {
                value.count += 1;
                Ok(())
            })
        }
        #[action]
        fn title(&self, cx: &Cx<'_>) -> Result<()> {
            self.document.update(cx, |value| {
                value.title.push('!');
                Ok(())
            })
        }
        #[computed]
        fn selected_title(&self, cx: &Cx<'_>) -> Result<String> {
            TITLES.fetch_add(1, Ordering::Relaxed);
            Ok(self.document.get(cx)?.title)
        }
        #[computed]
        fn formatted(&self, cx: &Cx<'_>) -> Result<String> {
            FORMATS.fetch_add(1, Ordering::Relaxed);
            Ok(format!(
                "Title: {}",
                Self::selected_title_computed().get(cx)?
            ))
        }
        fn view(&self, _: &Cx<'_>) -> Result<View> {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(Self::formatted_computed()).key("selected"));
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn selected_record_field_prunes_unchanged_downstream_computation()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let titles = TITLES.load(Ordering::Relaxed);
        let formats = FORMATS.load(Ordering::Relaxed);
        actor.tell(Model::<DescriptorMode>::count_action(), 0)?;
        actor.tick()?;
        assert_eq!(TITLES.load(Ordering::Relaxed), titles + 1);
        assert_eq!(FORMATS.load(Ordering::Relaxed), formats);
        actor.tell(Model::<DescriptorMode>::title_action(), 0)?;
        actor.tick()?;
        assert_eq!(FORMATS.load(Ordering::Relaxed), formats + 1);
        assert_eq!(actor.view.bindings[&abi::key("selected")].text, "Title: !");
        Ok(())
    }
}

mod invalid_writes {
    use blockui::*;
    #[derive(Default)]
    struct Checked {
        invalid: bool,
    }
    impl Record for Checked {
        const FORMAT: &'static str = "checked.record";
        fn encode(&self) -> Result<String> {
            Ok(if self.invalid { "unreadable" } else { "valid" }.into())
        }
        fn decode(payload: &str) -> Result<Self> {
            if payload != "valid" {
                return Err(Error::Application("record payload rejected".into()));
            }
            Ok(Self::default())
        }
    }
    #[actor(name = "checked")]
    pub(super) struct Model {
        document: State<Checked>,
        #[state(codec = Checked)]
        entries: State<Vec<String>>,
        count: State<i64>,
    }
    #[actor_impl]
    impl Model {
        #[action(name = "checked.invalid")]
        fn invalid(&self, cx: &Cx<'_>, map: i64) -> Result<()> {
            self.count.set(cx, 42)?;
            if map == 0 {
                self.document.set(cx, &Checked { invalid: true })
            } else {
                RecordMap::<Checked>::new(self.entries).insert(
                    cx,
                    "bad",
                    &Checked { invalid: true },
                )
            }
        }
        #[action]
        fn valid(&self, cx: &Cx<'_>) -> Result<()> {
            self.count.update(cx, |value| Ok(value + 1))
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            // Intentionally never read the record or collection.
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(self.count.get(cx)?.to_string()).key("count"));
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn malformed_unrendered_record_writes_reject_before_publication()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let before = actor.text("checked.document").to_owned();
        let buffer = actor.text("checked.document").as_ptr();
        for map in [0, 1] {
            let error = actor
                .dispatch(blockui_runtime::Message {
                    action: Model::<DescriptorMode>::invalid_action().id,
                    value: map,
                })
                .unwrap_err();
            assert!(error.contains("record payload rejected"), "{error}");
            assert_eq!(actor.text("checked.document"), before);
            assert_eq!(actor.text("checked.document").as_ptr(), buffer);
            assert_eq!(actor.integer("checked.count"), 0);
        }
        actor.tell(Model::<DescriptorMode>::valid_action(), 0)?;
        actor.tick()?;
        assert_eq!(actor.integer("checked.count"), 1);
        Ok(())
    }
}

#[test]
fn record_update_detects_reentrant_changes_without_overwriting_them()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let before = actor.text("declared.document").to_owned();
    let buffer = actor.text("declared.document").as_ptr();
    for mode in [0, 1] {
        let error = actor
            .dispatch(blockui_runtime::Message {
                action: Model::<DescriptorMode>::reentrant_action().id,
                value: mode,
            })
            .unwrap_err();
        assert!(error.contains("Record changed during update"), "{error}");
        assert_eq!(actor.text("declared.document"), before);
        assert_eq!(actor.text("declared.document").as_ptr(), buffer);
    }
    actor.tell(Model::<DescriptorMode>::reentrant_action(), 2)?;
    actor.tick()?;
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 8");
    actor.tell(Model::<DescriptorMode>::edit_action(), 0)?;
    actor.tick()?;
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 9");
    Ok(())
}

#[test]
fn declared_previous_record_upgrades_without_actor_migration_boilerplate()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let state = actor.state_address();
    let slot = actor.slot_address("declared.document");
    let schema = actor.schema_version;
    actor.replace_for_test::<Automatic>()?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("declared.document"), slot);
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 7");
    Ok(())
}

#[test]
fn automatic_record_upgrade_scopes_and_partial_failure_are_transactional()
-> std::result::Result<(), String> {
    let mut scoped = blockui_runtime::Actor::for_test::<Container>()?;
    let state = scoped.state_address();
    scoped.replace_for_test::<ContainerAutomatic>()?;
    assert_eq!(scoped.state_address(), state);
    assert_eq!(scoped.view.bindings[&abi::key("value")].text, "Draft 🦀: 7");
    let mut pair = blockui_runtime::Actor::for_test::<Pair>()?;
    let first = pair.text("pair.first").to_owned();
    let first_buffer = pair.text("pair.first").as_ptr();
    let second_buffer = pair.text("pair.second").as_ptr();
    let schema = pair.schema_version;
    let error = pair.replace_for_test::<PartialPair>().unwrap_err();
    assert!(error.contains("explicit migration is required"), "{error}");
    assert_eq!(pair.schema_version, schema);
    assert_eq!(pair.text("pair.first"), first);
    assert_eq!(pair.text("pair.first").as_ptr(), first_buffer);
    assert_eq!(pair.text("pair.second").as_ptr(), second_buffer);
    Ok(())
}

#[test]
fn automatic_record_upgrade_rejects_an_undeclared_previous_format()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Foreign>()?;
    let before = actor.text("declared.document").to_owned();
    let buffer = actor.text("declared.document").as_ptr();
    let schema = actor.schema_version;
    let error = actor.replace_for_test::<Automatic>().unwrap_err();
    assert!(
        error.contains("Record format requires migration"),
        "{error}"
    );
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text("declared.document"), before);
    assert_eq!(actor.text("declared.document").as_ptr(), buffer);
    Ok(())
}
