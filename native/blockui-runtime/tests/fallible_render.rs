#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record)]
#[record(name = "fallible.document")]
struct Document {
    title: String,
}
#[actor]
struct Model {
    #[state(key = "documents", codec = Document)]
    documents: State<Vec<String>>,
    #[state(key = "draft", default = "Draft 🦀")]
    draft: State<String>,
}
#[actor_impl(render)]
impl Model {
    #[action(name = "put")]
    fn put(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<Document>::new(self.documents).insert(
            cx,
            "42",
            &Document {
                title: self.draft.get(cx)?,
            },
        )
    }
    #[action(name = "corrupt")]
    fn corrupt(&self, cx: &Cx<'_>) -> Result<()> {
        self.draft.set(cx, "changed before rejection")?;
        self.documents.set(cx, &["42".into(), "invalid".into()])
    }
}
impl Render<Result<View>> for ModelBound<'_, '_> {
    fn render(&self) -> Result<View> {
        let documents = self.documents.records::<Document>()?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(TextInput::new(&self.draft).key("draft.editor"));
                for (id, document) in documents {
                    __blockui_tree.add(Text::new(document.title).key(format!("document.{id}")));
                }
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Rejected {
    #[state(key = "documents", codec = Document)]
    documents: State<Vec<String>>,
    #[state(key = "draft", default = "Draft 🦀")]
    draft: State<String>,
}
#[actor_impl(render)]
impl Rejected {}
impl Render<Result<View>> for RejectedBound<'_, '_> {
    fn render(&self) -> Result<View> {
        Err(Error::Application("candidate render rejected".into()))
    }
}
#[test]
fn fallible_bound_render_rolls_back_turn_writes_and_rejects_candidates()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    let records = actor.text_list("documents").to_vec();
    let buffer = actor.text_list("documents").as_ptr();
    let draft = actor.text("draft").as_ptr();
    let version = actor.implementation_version();
    assert_eq!(
        actor.view.bindings[&abi::key("document.42")].text,
        "Draft 🦀"
    );
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: abi::key("corrupt"),
                value: 0
            })
            .is_err()
    );
    assert_eq!(actor.text_list("documents"), records);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert_eq!(actor.text("draft").as_ptr(), draft);
    assert_eq!(actor.text("draft"), "Draft 🦀");
    assert_eq!(
        actor.view.bindings[&abi::key("document.42")].text,
        "Draft 🦀"
    );
    assert!(actor.replace_for_test::<Rejected>().is_err());
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    Ok(())
}
