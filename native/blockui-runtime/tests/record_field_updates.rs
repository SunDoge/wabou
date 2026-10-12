#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record, Default)]
#[record(name = "field.updates.Child")]
struct Child {
    text: String,
}
#[derive(Record, Default)]
#[record(name = "field.updates.Document")]
struct Document {
    body: String,
    count: i64,
    optional: Option<i64>,
    rows: Vec<String>,
    #[record(nested)]
    child: Child,
}
#[actor(name = "field.updates")]
struct Model {
    document: State<Document>,
}
#[actor_impl]
impl Model {
    #[action(name = "field.updates.apply")]
    fn apply(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        match mode {
            0 => {
                Document::body_field().set(self.document, cx, "Changed 🦀".into())?;
                let document = self.document.bind(cx)?;
                document.update_field(Document::count_field(), |count| {
                    *count = count.checked_add(1).ok_or(Error::Overflow)?;
                    Ok(())
                })?;
                document.set_field(Document::optional_field(), Some(7))?;
                document.update_field(Document::rows_field(), |rows| {
                    rows.push("α\nβ".into());
                    Ok(())
                })?;
                Document::child_field().update(self.document, cx, |child| {
                    child.text = "child".into();
                    Ok(())
                })
            }
            1 => {
                Document::body_field().set(self.document, cx, "partial".into())?;
                self.document
                    .bind(cx)?
                    .update_field(Document::count_field(), |count| {
                        *count += 1;
                        Err(Error::Application("reject second field".into()))
                    })
            }
            2 => Document::body_field().update(self.document, cx, |body| {
                body.push_str("local");
                Document::count_field().set(self.document, cx, 9)?;
                Ok(())
            }),
            3 => Document::body_field().set(self.document, cx, "x".repeat(1024 * 1024)),
            4 => {
                let body = Document::body_field().get(self.document, cx)?;
                Document::body_field().set(self.document, cx, body)
            }
            5 => Document::count_field().set(self.document, cx, i64::MAX),
            _ => Err(Error::Application("unknown update".into())),
        }
    }
    #[action(name = "field.updates.edit")]
    fn edit(&self, cx: &Cx<'_>, body: String) -> Result<()> {
        Document::body_field().set(self.document, cx, body)?;
        Document::count_field().update(self.document, cx, |count| {
            *count = count.checked_add(1).ok_or(Error::Overflow)?;
            Ok(())
        })
    }
    #[computed(name = "field.updates.count")]
    fn count(&self, cx: &Cx<'_>) -> Result<i64> {
        Document::count_field().get(self.document, cx)
    }
    fn view(&self, _: &Cx<'_>) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    TextInput::field(
                        self.document.text(Document::body_field()),
                        Self::edit_action(),
                    )
                    .key("input"),
                );
                __blockui_tree.add(Text::new(Self::count_computed()).key("count"));
            }
            __blockui_tree.finish()
        }
    }
}

#[test]
fn field_writes_preserve_siblings_and_rollback_all_writes_and_allocations()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let message = |value| blockui_runtime::Message {
        action: abi::key("field.updates.apply"),
        value,
    };
    actor.dispatch(message(0))?;
    let encoded = actor.text("field.updates.document").to_owned();
    let document: Document =
        record_codec::decode_record(&encoded).map_err(|error| error.to_string())?;
    assert_eq!(document.body, "Changed 🦀");
    assert_eq!(document.count, 1);
    assert_eq!(document.optional, Some(7));
    assert_eq!(document.rows, ["α\nβ"]);
    assert_eq!(document.child.text, "child");
    let buffer = actor.text("field.updates.document").as_ptr();
    let evaluations = actor.view.bindings[&abi::key("input")].evaluations;
    actor.dispatch(message(4))?;
    assert_eq!(actor.text("field.updates.document").as_ptr(), buffer);
    assert_eq!(
        actor.view.bindings[&abi::key("input")].evaluations,
        evaluations
    );
    for (mode, expected) in [
        (1, "reject second field"),
        (2, "Record changed during update"),
        (3, "exceeds 1 MiB"),
    ] {
        assert!(
            actor
                .dispatch(message(mode))
                .unwrap_err()
                .contains(expected)
        );
        assert_eq!(actor.text("field.updates.document"), encoded);
        assert_eq!(actor.text("field.updates.document").as_ptr(), buffer);
        assert_eq!(actor.view.bindings[&abi::key("input")].text, "Changed 🦀");
        assert_eq!(actor.view.bindings[&abi::key("count")].text, "1");
    }
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text("field.updates.document").as_ptr(), buffer);
    actor.edit_text(abi::key("input"), "after reload")?;
    assert_eq!(actor.view.bindings[&abi::key("count")].text, "2");
    actor.dispatch(message(5))?;
    let buffer = actor.text("field.updates.document").as_ptr();
    assert!(actor.edit_text(abi::key("input"), "overflow").is_err());
    assert_eq!(actor.text("field.updates.document").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&abi::key("input")].text, "after reload");
    assert_eq!(
        actor.view.bindings[&abi::key("count")].text,
        i64::MAX.to_string()
    );
    Ok(())
}
