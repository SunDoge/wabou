#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record, Default)]
struct Child {
    text: String,
}
#[derive(Record, Default)]
struct Data {
    label: String,
    optional: Option<i64>,
    values: Vec<String>,
    #[record(nested)]
    child: Child,
    #[record(nested)]
    optional_child: Option<Child>,
    #[record(nested)]
    children: Vec<Child>,
    ignored: bool,
}
#[actor(name = "fields")]
struct Model {
    data: State<Data>,
}
#[actor_impl]
impl Model {
    #[computed(name = "fields.selected")]
    fn selected(&self, cx: &Cx<'_>) -> Result<String> {
        let data = self.data.bind(cx)?;
        let child = data.field(Data::optional_child_field())?;
        let children = data.field(Data::children_field())?;
        let suffix = format!(
            "{:?}|{:?}",
            child.map(|child| child.text),
            children
                .into_iter()
                .map(|child| child.text)
                .collect::<Vec<_>>()
        );
        Ok(format!(
            "{}|{:?}|{:?}|{}|{suffix}",
            data.field(Data::label_field())?,
            data.field(Data::optional_field())?,
            data.field(Data::values_field())?,
            data.field(Data::child_field())?.text
        ))
    }
    #[action]
    fn update(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        self.data.update(cx, |data| {
            if mode == 0 {
                data.ignored = !data.ignored;
            } else {
                data.label = "🦀".into();
                data.optional = Some(7);
                data.values = vec!["".into(), "α\nβ".into()];
                data.child.text = "child".into();
                data.optional_child = Some(Child {
                    text: "optional".into(),
                });
                data.children = vec![Child {
                    text: "list".into(),
                }];
            }
            Ok(())
        })?;
        if mode == 2 {
            return Err(Error::Application("rollback fields".into()));
        }
        Ok(())
    }
    #[action]
    fn corrupt(&self, cx: &Cx<'_>) -> Result<()> {
        // Raw escape hatch deliberately bypasses the generated record codec.
        State::<String>::new("fields.data").set(cx, "bad\n1\nx")
    }
    fn view(&self, _: &Cx<'_>) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(Self::selected_computed()).key("selected"));
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn selected_fields_skip_unrelated_writes_and_rollback_cache_revisions()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let selected = abi::key("selected");
    let initial = actor.view.bindings[&selected].text.clone();
    let message = |value| blockui_runtime::Message {
        action: Model::<DescriptorMode>::update_action().id,
        value,
    };
    let evaluations = actor.computed_evaluations("fields.selected");
    assert!(evaluations > 0);
    actor.dispatch(message(0))?;
    assert_eq!(actor.computed_evaluations("fields.selected"), evaluations);
    let buffer = actor.text("fields.data").as_ptr();
    assert!(
        actor
            .dispatch(message(2))
            .unwrap_err()
            .contains("rollback fields")
    );
    assert_eq!(actor.text("fields.data").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&selected].text, initial);
    assert_eq!(actor.computed_evaluations("fields.selected"), evaluations);
    actor.dispatch(message(1))?;
    assert_eq!(
        actor.computed_evaluations("fields.selected"),
        evaluations + 1
    );
    assert!(actor.view.bindings[&selected].text.contains("🦀|Some(7)"));
    assert!(
        actor.view.bindings[&selected]
            .text
            .ends_with("|child|Some(\"optional\")|[\"list\"]")
    );
    let buffer = actor.text("fields.data").as_ptr();
    let published = actor.view.bindings[&selected].text.clone();
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: Model::<DescriptorMode>::corrupt_action().id,
                value: 0,
            })
            .is_err()
    );
    assert_eq!(actor.text("fields.data").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&selected].text, published);
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text("fields.data").as_ptr(), buffer);
    actor.dispatch(message(0))?;
    assert_eq!(actor.view.bindings[&selected].text, published);
    Ok(())
}

#[derive(Record, Default)]
#[record(name = "direct.document")]
struct Before {
    body: String,
}
#[derive(Record, Default)]
#[record(name = "direct.document", previous = Before)]
struct After {
    #[record(default = "header".into())]
    header: String,
    body: String,
}
#[actor(name = "direct")]
struct OldEditor {
    document: State<Before>,
}
#[actor_impl]
impl OldEditor {
    #[action(name = "direct.edit")]
    fn edit(&self, cx: &Cx<'_>, text: String) -> Result<()> {
        self.document.update(cx, |document| {
            document.body = text;
            Ok(())
        })
    }
    fn view(&self, _: &Cx<'_>) -> View {
        let mut root = Column::default().into_view();
        root.push(TextInput::field(
            self.document.text(Before::body_field()),
            Self::edit_action(),
        ));
        root
    }
}
#[actor(name = "direct")]
struct NewEditor {
    document: State<After>,
}
#[actor_impl]
impl NewEditor {
    #[action(name = "direct.edit")]
    fn edit(&self, cx: &Cx<'_>, text: String) -> Result<()> {
        self.document.update(cx, |document| {
            document.body = text;
            Ok(())
        })
    }
    fn view(&self, _: &Cx<'_>) -> View {
        let mut root = Column::default().into_view();
        root.push(TextInput::field(
            self.document.text(After::body_field()),
            Self::edit_action(),
        ));
        root.push(Text::new(self.document.text(After::header_field())).key("header"));
        root
    }
}
#[test]
fn automatic_field_widget_identity_survives_changed_schema_ordinals()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<OldEditor>()?;
    let input = actor
        .view
        .bindings
        .values()
        .find(|binding| binding.node.kind == abi::ENTRY)
        .unwrap()
        .node
        .key;
    actor.edit_text(input, "before 🦀")?;
    let state = actor.state_address();
    let slot = actor.slot_address("direct.document");
    actor.replace_for_test::<NewEditor>()?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("direct.document"), slot);
    assert_eq!(
        actor
            .view
            .bindings
            .values()
            .find(|binding| binding.node.kind == abi::ENTRY)
            .unwrap()
            .node
            .key,
        input
    );
    assert_eq!(actor.view.bindings[&input].text, "before 🦀");
    assert_eq!(actor.view.bindings[&abi::key("header")].text, "header");
    actor.edit_text(input, "after 🦀")?;
    assert_eq!(actor.view.bindings[&input].text, "after 🦀");
    let buffer = actor.text("direct.document").as_ptr();
    actor.replace_for_test::<NewEditor>()?;
    assert_eq!(actor.text("direct.document").as_ptr(), buffer);
    Ok(())
}
