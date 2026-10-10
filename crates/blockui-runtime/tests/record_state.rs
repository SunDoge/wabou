#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record)]
#[record(name = "typed.document")]
struct Document {
    title: String,
    count: i64,
}
const DOCUMENT: State<Document> = State::<Document>::new("document");
struct Model;
impl Actor for Model {
    const STATE: &'static [StateSpec] = &DOCUMENT.schema();
    const ACTIONS: &'static [Action] = &[Action::new("edit", |cx, value| {
        DOCUMENT.bind(cx)?.update(|document| {
            document.count += 1;
            document.title.push('!');
            if value < 0 {
                Err(Error::Application("rejected edit".into()))
            } else {
                Ok(())
            }
        })
    })];
    fn migrate(cx: &Cx<'_>, previous: u64) -> Result<()> {
        if previous == 0 {
            DOCUMENT.set(
                cx,
                &Document {
                    title: "Draft 🦀".into(),
                    count: 7,
                },
            )?;
        }
        Ok(())
    }
    fn view(cx: &Cx<'_>) -> Result<View> {
        let document = DOCUMENT.bind(cx)?.get()?;
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
struct Replacement;
impl Actor for Replacement {
    const STATE: &'static [StateSpec] = Model::STATE;
    const ACTIONS: &'static [Action] = &[Action::new("edit", |cx, _| {
        DOCUMENT.update(cx, |document| {
            document.count += 10;
            Ok(())
        })
    })];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Model::view(cx)
    }
}
struct Uninitialized;
impl Actor for Uninitialized {
    const STATE: &'static [StateSpec] = Model::STATE;
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Model::view(cx)
    }
}
#[derive(Record)]
#[record(name = "typed.document", previous = Document)]
struct RevisedDocument {
    title: String,
    count: i64,
    #[record(default)]
    archived: bool,
}
const REVISED: State<RevisedDocument> = State::<RevisedDocument>::new("document");
struct MissingMigration;
impl Actor for MissingMigration {
    const ACTIONS: &'static [Action] = &[];
    const STATE: &'static [StateSpec] = &REVISED.schema();
    fn view(cx: &Cx<'_>) -> Result<View> {
        REVISED.get(cx)?;
        Model::view(cx)
    }
}
struct Migrated;
impl Actor for Migrated {
    const ACTIONS: &'static [Action] = &[];
    const STATE: &'static [StateSpec] = &REVISED.schema();
    fn migrate(cx: &Cx<'_>, _: u64) -> Result<()> {
        let previous = REVISED.bind(cx)?.previous::<Document>()?;
        REVISED.set(cx, &RevisedDocument::upgrade(previous)?)
    }
    fn view(cx: &Cx<'_>) -> Result<View> {
        let document = REVISED.get(cx)?;
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
#[test]
fn typed_record_state_keeps_host_buffer_and_rolls_back_failed_edits_and_migrations()
-> std::result::Result<(), String> {
    assert!(blockui_runtime::Actor::for_test::<Uninitialized>().is_err());
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let state = actor.state_address();
    let slot = actor.slot_address("document");
    let buffer = actor.text("document").as_ptr();
    actor.replace_for_test::<Replacement>()?;
    assert_eq!(actor.text("document").as_ptr(), buffer);
    assert!(actor.replace_for_test::<MissingMigration>().is_err());
    assert_eq!(actor.text("document").as_ptr(), buffer);
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("edit"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 17");
    actor.replace_for_test::<Model>()?;
    let before = actor.text("document").to_owned();
    let before_buffer = actor.text("document").as_ptr();
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: abi::key("edit"),
                value: -1
            })
            .is_err()
    );
    assert_eq!(actor.text("document"), before);
    assert_eq!(actor.text("document").as_ptr(), before_buffer);
    actor.replace_for_test::<Migrated>()?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("document"), slot);
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "Draft 🦀: 17");
    Ok(())
}
