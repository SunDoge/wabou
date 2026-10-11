#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record, Default)]
struct Document {
    body: String,
    edits: i64,
}
#[actor(name = "controlled")]
struct Editor {
    document: State<Document>,
}
#[actor_impl]
impl Editor {
    #[computed(name = "controlled.body")]
    fn body(&self, cx: &Cx<'_>) -> Result<String> {
        Document::body_field().get(self.document, cx)
    }
    #[action]
    fn edit(&self, cx: &Cx<'_>, text: String) -> Result<()> {
        self.document.update(cx, |document| {
            document.body = text.trim().to_uppercase();
            document.edits += 1;
            Ok(())
        })?;
        if text.contains('!') {
            return Err(Error::Application("rejected after writing".into()));
        }
        Ok(())
    }
    #[action]
    fn bump(&self, cx: &Cx<'_>) -> Result<()> {
        self.document.update(cx, |document| {
            document.edits += 1;
            Ok(())
        })
    }
    #[action]
    fn reset(&self, cx: &Cx<'_>) -> Result<()> {
        self.document.set(cx, &Document::default())
    }
    fn view(&self, _: &Cx<'_>) -> View {
        let editor = TextArea::computed(Self::body_computed(), Self::edit_action())
            .key("input")
            .label("Document");
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(editor).__local_identity(stringify!(editor)),
                );
            }
            __blockui_tree.finish()
        }
    }
}

#[test]
fn computed_editor_normalizes_rolls_back_and_updates_from_model() -> std::result::Result<(), String>
{
    let mut actor = blockui_runtime::Actor::for_test::<Editor>()?;
    let input = abi::key("input");
    actor.edit_text(input, "  héllo  ")?;
    assert_eq!(actor.view.bindings[&input].text, "HÉLLO");
    let evaluations = actor.computed_evaluations("controlled.body");
    assert!(evaluations > 0);
    actor.dispatch(blockui_runtime::Message {
        action: Editor::actions().bump.id,
        value: 0,
    })?;
    assert_eq!(actor.computed_evaluations("controlled.body"), evaluations);
    assert_eq!(actor.view.bindings[&input].text, "HÉLLO");
    let bytes = actor.text("controlled.document").to_owned();
    let buffer = actor.text("controlled.document").as_ptr();
    let state = actor.state_address();
    assert!(
        actor
            .edit_text(input, "broken!")
            .unwrap_err()
            .contains("rejected after writing")
    );
    assert_eq!(actor.text("controlled.document"), bytes);
    assert_eq!(actor.text("controlled.document").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&input].text, "HÉLLO");
    actor.replace_for_test::<Editor>()?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.text("controlled.document").as_ptr(), buffer);
    actor.edit_text(input, "reloaded")?;
    assert_eq!(actor.view.bindings[&input].text, "RELOADED");
    actor.dispatch(blockui_runtime::Message {
        action: Editor::actions().reset.id,
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&input].text, "");
    Ok(())
}

struct Input<const MODE: u8>;
impl<const MODE: u8> Actor for Input<MODE> {
    const STATE: &'static [StateSpec] = <Editor as Actor>::STATE;
    const ACTIONS: &'static [Action] = <Editor as Actor>::ACTIONS;
    const COMPUTEDS: &'static [ComputedSpec] = <Editor as Actor>::COMPUTEDS;
    fn migrate(cx: &Cx<'_>, previous: u64) -> Result<()> {
        <Editor as Actor>::migrate(cx, previous)
    }
    fn view(_: &Cx<'_>) -> Result<View> {
        let mut input = TextInput::computed(
            Editor::<DescriptorMode>::body_computed(),
            Editor::actions().edit,
        )
        .key("input");
        input.read_only = MODE == 1;
        input.enabled = MODE != 2;
        if MODE == 3 {
            input.on_change = None;
        }
        let input = input.label("Document title");
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(::blockui::IntoView::into_view(input).__local_identity(stringify!(input)));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn single_line_projection_survives_replacement_and_enforces_edit_admission()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Input<0>>()?;
    let input = abi::key("input");
    actor.edit_text(input, " title ")?;
    assert_eq!(actor.view.bindings[&input].text, "TITLE");
    let state = actor.state_address();
    let buffer = actor.text("controlled.document").as_ptr();
    assert!(actor.replace_for_test::<Input<3>>().is_err());
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.text("controlled.document").as_ptr(), buffer);
    actor.edit_text(input, "still works")?;
    assert_eq!(actor.view.bindings[&input].text, "STILL WORKS");
    actor.replace_for_test::<Input<1>>()?;
    assert!(actor.edit_text(input, "readonly").is_err());
    actor.replace_for_test::<Input<2>>()?;
    assert!(actor.edit_text(input, "disabled").is_err());
    assert_eq!(actor.view.bindings[&input].text, "STILL WORKS");
    actor.replace_for_test::<Input<0>>()?;
    actor.edit_text(input, "working again")?;
    assert_eq!(actor.view.bindings[&input].text, "WORKING AGAIN");
    Ok(())
}

#[derive(Record)]
struct WrongDocument {
    body: String,
}
struct Direct<const MODE: u8>;
impl<const MODE: u8> Actor for Direct<MODE> {
    const STATE: &'static [StateSpec] = <Editor as Actor>::STATE;
    const ACTIONS: &'static [Action] = <Editor as Actor>::ACTIONS;
    fn migrate(cx: &Cx<'_>, previous: u64) -> Result<()> {
        <Editor as Actor>::migrate(cx, previous)
    }
    fn view(_: &Cx<'_>) -> Result<View> {
        let value = if MODE == 4 {
            State::<WrongDocument>::new("controlled.document").text(WrongDocument::body_field())
        } else {
            Editor::<DescriptorMode>::new()
                .document
                .text(Document::body_field())
        };
        let mut input = TextInput::field(value, Editor::actions().edit).key("input");
        input.read_only = MODE == 1;
        input.enabled = MODE != 2;
        if MODE == 3 {
            input.on_change = None;
        }
        let input = input.label("Document title\nAccessible name");
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(::blockui::IntoView::into_view(input).__local_identity(stringify!(input)));
                __blockui_tree.add(
                    TextArea::field(value, Editor::actions().edit)
                        .key("area")
                        .label("Document body"),
                );
                __blockui_tree.add(Text::new(value).key("caption"));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn direct_field_bindings_share_model_prune_metadata_and_reject_bad_candidates()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Direct<0>>()?;
    let keys = [abi::key("input"), abi::key("area"), abi::key("caption")];
    assert_eq!(
        actor.view.bindings[&keys[0]].semantic_label.as_deref(),
        Some("Document title\nAccessible name")
    );
    actor.edit_text(keys[0], " héllo ")?;
    for key in keys {
        assert_eq!(actor.view.bindings[&key].text, "HÉLLO");
    }
    let counts = keys.map(|key| actor.view.bindings[&key].evaluations);
    assert!(counts.iter().all(|count| *count > 0));
    actor.dispatch(blockui_runtime::Message {
        action: Editor::actions().bump.id,
        value: 0,
    })?;
    assert_eq!(
        keys.map(|key| actor.view.bindings[&key].evaluations),
        counts
    );
    let state = actor.state_address();
    let buffer = actor.text("controlled.document").as_ptr();
    assert!(actor.edit_text(keys[1], "bad!").is_err());
    assert_eq!(actor.text("controlled.document").as_ptr(), buffer);
    assert!(actor.replace_for_test::<Direct<3>>().is_err());
    assert!(
        actor
            .replace_for_test::<Direct<4>>()
            .unwrap_err()
            .contains("invalid record text field binding")
    );
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.text("controlled.document").as_ptr(), buffer);
    for key in keys {
        assert_eq!(actor.view.bindings[&key].text, "HÉLLO");
    }
    actor.replace_for_test::<Direct<1>>()?;
    assert!(actor.edit_text(keys[0], "readonly").is_err());
    actor.replace_for_test::<Direct<2>>()?;
    assert!(actor.edit_text(keys[0], "disabled").is_err());
    actor.replace_for_test::<Direct<0>>()?;
    actor.edit_text(keys[1], "reloaded")?;
    for key in keys {
        assert_eq!(actor.view.bindings[&key].text, "RELOADED");
    }
    actor.dispatch(blockui_runtime::Message {
        action: Editor::actions().reset.id,
        value: 0,
    })?;
    for key in keys {
        assert_eq!(actor.view.bindings[&key].text, "");
    }
    Ok(())
}
