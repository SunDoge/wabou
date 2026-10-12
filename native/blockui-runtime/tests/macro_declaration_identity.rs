#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record)]
#[record(name = "macro.document")]
struct PlainDocument {
    title: String,
    count: i64,
    optional: Option<i64>,
}
macro_rules! document {
    ($text:ty, $integer:ty) => {
        #[derive(Record)]
        #[record(name = "macro.document")]
        struct GeneratedDocument {
            title: $text,
            count: $integer,
            optional: Option<$integer>,
        }
    };
}
document!(String, i64);

#[actor(name = "macro.identity")]
struct Plain {
    count: State<i64>,
}
#[actor_impl]
impl Plain {
    #[action(name = "identity.increment")]
    fn increment(&self, cx: &Cx<'_>, delta: i64) -> Result<()> {
        self.count.update(cx, |count| Ok(count + delta))
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(self.count));
            }
            __blockui_tree.finish()
        })
    }
}
macro_rules! application {
    ($state:ty, $context:ty, $payload:ty, $reply:ty) => {
        #[actor(name = "macro.identity")]
        struct Generated {
            count: $state,
        }
        #[actor_impl]
        impl Generated {
            fn migrate(&self, _: $context, _: u64) -> Result<()> {
                Err(Error::Application(
                    "unchanged schema must not migrate".into(),
                ))
            }
            #[action(name = "identity.increment")]
            fn add_to_count(&self, cx: $context, delta: $payload) -> $reply {
                self.count.update(cx, |count| Ok(count + delta * 2))
            }
            fn view(&self, _: $context) -> Result<View> {
                Ok(Column::default().children(vec![Text::new(self.count).into_view()]))
            }
        }
    };
}
application!(State<i64>, &Cx<'_>, i64, Result<()>);

#[test]
fn generated_record_has_identical_canonical_schema_and_payload() {
    let plain = PlainDocument {
        title: "Draft 🦀".into(),
        count: 7,
        optional: Some(9),
    };
    let generated = GeneratedDocument {
        title: plain.title.clone(),
        count: plain.count,
        optional: plain.optional,
    };
    let encoded = record_codec::encode_record(&plain).unwrap();
    assert_eq!(record_codec::encode_record(&generated).unwrap(), encoded);
    let decoded: GeneratedDocument = record_codec::decode_record(&encoded).unwrap();
    assert_eq!(decoded.title, plain.title);
    assert_eq!(decoded.count, plain.count);
    assert_eq!(decoded.optional, plain.optional);
}

#[test]
fn generated_actor_replacement_retains_slots_and_executes_queued_new_code() {
    let mut actor = blockui_runtime::Actor::for_test::<Plain>().unwrap();
    actor.tell(Plain::actions().increment, 5).unwrap();
    actor.tick().unwrap();
    let slot = actor.slot_address("macro.identity.count");
    let state = actor.state_address();
    assert_eq!(
        Plain::actions().increment.id,
        Generated::actions().add_to_count.id
    );
    actor.tell(Plain::actions().increment, 3).unwrap();
    actor.replace_for_test::<Generated>().unwrap();
    assert_eq!(actor.integer("macro.identity.count"), 5);
    assert_eq!(actor.slot_address("macro.identity.count"), slot);
    assert_eq!(actor.state_address(), state);
    actor.tick().unwrap();
    assert_eq!(actor.integer("macro.identity.count"), 11);
    assert_eq!(actor.slot_address("macro.identity.count"), slot);
    actor.tell(Generated::actions().add_to_count, 2).unwrap();
    actor.tick().unwrap();
    assert_eq!(actor.integer("macro.identity.count"), 15);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("macro.identity.count"), slot);
}
