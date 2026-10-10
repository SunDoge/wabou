#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record)]
#[record(name = "schema.document")]
struct Old {
    title: String,
}
#[derive(Record)]
#[record(name = "schema.document", previous = Old)]
struct New {
    title: String,
    #[record(default)]
    priority: i64,
    #[record(default)]
    note: Option<String>,
}
#[derive(Record)]
#[record(name = "schema.document")]
struct Unsupported {
    title: String,
    priority: i64,
    note: Option<String>,
}
#[actor]
struct Original {
    #[state(key = "documents", codec = Old)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl Original {
    #[action(name = "put")]
    fn put(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<Old>::new(self.documents).insert(
            cx,
            "42",
            &Old {
                title: "Draft 🦀".into(),
            },
        )?;
        RecordMap::<Old>::new(self.documents).insert(
            cx,
            "99",
            &Old {
                title: "Second".into(),
            },
        )?;
        RecordMap::<Old>::new(self.archive).insert(
            cx,
            "7",
            &Old {
                title: "Archived".into(),
            },
        )
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Original"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Missing {
    #[state(key = "documents", codec = Unsupported)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl Missing {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        // Does not read records: schema admission must detect codec changes.
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Missing migration"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Upgraded {
    #[state(key = "documents", codec = New)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl Upgraded {
    fn migrate(&self, cx: &Cx<'_>, previous: u64) -> Result<()> {
        if previous != 0 {
            RecordMap::<New>::new(self.documents).upgrade_previous::<Old>(cx)?;
        }
        Ok(())
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Upgraded"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct AddedCollection {
    #[state(key = "documents", codec = Old)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
    #[state(key = "new.documents", codec = New)]
    added: State<Vec<String>>,
}
#[actor_impl]
impl AddedCollection {
    #[action(name = "add")]
    fn add(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<New>::new(self.added).insert(
            cx,
            "73",
            &New {
                title: "New collection".into(),
                priority: 9,
                note: Some("Optional 🦀".into()),
            },
        )
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Added collection"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct NestedOriginal {
    #[state(record, scope = "nested")]
    child: Original,
}
#[actor_impl]
impl NestedOriginal {
    #[action(name = "nested.put")]
    fn put(&self, cx: &Cx<'_>) -> Result<()> {
        self.child.put(cx)
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Nested"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct NestedMissing {
    #[state(record, scope = "nested")]
    child: Missing,
}
#[actor_impl]
impl NestedMissing {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Nested changed"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Automatic {
    #[state(key = "documents", codec = New)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl Automatic {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Automatic"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Noop {
    #[state(key = "documents", codec = New)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl Noop {
    fn migrate(&self, _: &Cx<'_>, _: u64) -> Result<()> {
        Ok(())
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Noop"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Partial {
    #[state(key = "documents", codec = New)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Unsupported)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl Partial {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Partial"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct NestedAutomatic {
    #[state(record, scope = "nested")]
    child: Automatic,
}
#[actor_impl]
impl NestedAutomatic {
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let documents = RecordMap::<New>::new(self.child.documents).entries(cx)?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Text::new(
                        documents
                            .first()
                            .map_or("Empty", |(_, record)| record.title.as_str()),
                    )
                    .key("nested.document"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
struct Failing {
    title: String,
}
impl Record for Failing {
    const FORMAT: &'static str = "schema.reject";
    const PREVIOUS_FORMAT: Option<&'static str> = Some(Old::FORMAT);
    fn encode(&self) -> Result<String> {
        Old {
            title: self.title.clone(),
        }
        .encode()
    }
    fn decode(payload: &str) -> Result<Self> {
        Ok(Self {
            title: Old::decode(payload)?.title,
        })
    }
    fn upgrade_encoded(encoded: &str) -> Result<Self> {
        let previous = record_codec::decode_record::<Old>(encoded)?;
        if previous.title == "Second" {
            return Err(Error::Application("second record rejected".into()));
        }
        Ok(Self {
            title: previous.title,
        })
    }
}
#[actor]
struct BadCollection {
    #[state(key = "documents", codec = Failing)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl BadCollection {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Bad collection"));
            }
            __blockui_tree.finish()
        })
    }
}
#[derive(Record)]
#[record(name = "schema.foreign")]
struct Foreign {
    title: String,
}
#[derive(Record)]
#[record(name = "schema.document", previous = Foreign)]
struct Wrong {
    title: String,
    #[record(default)]
    priority: i64,
    #[record(default)]
    note: Option<String>,
}
#[actor]
struct WrongPredecessor {
    #[state(key = "documents", codec = Wrong)]
    documents: State<Vec<String>>,
    #[state(key = "archive", codec = Old)]
    archive: State<Vec<String>>,
}
#[actor_impl]
impl WrongPredecessor {
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Wrong predecessor"));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn nested_scoped_codec_changes_also_require_migration() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<NestedOriginal>()?;
    let schema = actor.schema_version;
    actor.replace_for_test::<NestedOriginal>()?;
    assert!(actor.replace_for_test::<NestedMissing>().is_err());
    assert_eq!(actor.schema_version, schema);
    Ok(())
}
#[test]
fn adding_a_new_codec_collection_keeps_existing_records_without_custom_migration()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Original>()?;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    let buffer = actor.text_list("documents").as_ptr();
    let archive = actor.text_list("archive").as_ptr();
    let identity = actor.state_address();
    actor.replace_for_test::<AddedCollection>()?;
    assert_eq!(actor.state_address(), identity);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert_eq!(actor.text_list("archive").as_ptr(), archive);
    assert!(actor.text_list("new.documents").is_empty());
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("add"),
        value: 0,
    })?;
    let added = actor.text_list("new.documents").as_ptr();
    actor.replace_for_test::<AddedCollection>()?;
    assert_eq!(actor.text_list("new.documents").as_ptr(), added);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert!(actor.replace_for_test::<Missing>().is_err());
    assert_eq!(actor.text_list("new.documents").as_ptr(), added);
    Ok(())
}
#[test]
fn codec_metadata_is_scoped_per_collection_and_requires_migration()
-> std::result::Result<(), String> {
    assert_eq!(Original::SCHEMA.len(), 4);
    let mut actor = blockui_runtime::Actor::for_test::<Original>()?;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    let identity = actor.state_address();
    let documents = actor.slot_address("documents");
    let archive = actor.text_list("archive").as_ptr();
    let buffer = actor.text_list("documents").as_ptr();
    let schema = actor.schema_version;
    actor.replace_for_test::<Original>()?;
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert_eq!(actor.schema_version, schema);
    assert!(actor.replace_for_test::<Missing>().is_err());
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    actor.replace_for_test::<Upgraded>()?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), identity);
    assert_eq!(actor.slot_address("documents"), documents);
    assert_eq!(actor.text_list("archive").as_ptr(), archive);
    assert!(actor.text_list("documents")[1].starts_with(New::FORMAT));
    let (_, payload) = actor.text_list("documents")[1].split_once('\n').unwrap();
    let record = New::decode(payload).unwrap();
    assert_eq!(record.title, "Draft 🦀");
    assert_eq!(record.priority, 0);
    assert_eq!(record.note, None);
    Ok(())
}

#[test]
fn declared_collection_predecessor_migrates_and_validates_unrendered_records()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Original>()?;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    let documents = actor.text_list("documents").as_ptr();
    let archive = actor.text_list("archive").as_ptr();
    let slot = actor.slot_address("documents");
    let state = actor.state_address();
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    let original_documents = actor.text_list("documents").to_vec();
    let original_archive = actor.text_list("archive").to_vec();
    let current = |actor: &blockui_runtime::Actor| {
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.state_address(), state);
        assert_eq!(actor.slot_address("documents"), slot);
        assert_eq!(actor.text_list("documents").as_ptr(), documents);
        assert_eq!(actor.text_list("archive").as_ptr(), archive);
        assert_eq!(actor.text_list("documents"), original_documents);
        assert_eq!(actor.text_list("archive"), original_archive);
    };
    assert!(actor.replace_for_test::<Noop>().is_err());
    current(&actor);
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    current(&actor);
    assert!(actor.replace_for_test::<Partial>().is_err());
    current(&actor);
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    current(&actor);
    let error = actor.replace_for_test::<BadCollection>().unwrap_err();
    assert!(error.contains("second record rejected"), "{error}");
    current(&actor);
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("put"),
        value: 0,
    })?;
    current(&actor);
    actor.replace_for_test::<Automatic>()?;
    assert_eq!(actor.text_list("documents")[0], "42");
    let record = record_codec::decode_record::<New>(&actor.text_list("documents")[1]).unwrap();
    assert_eq!(record.title, "Draft 🦀");
    assert_eq!(record.priority, 0);
    assert_eq!(record.note, None);
    assert_eq!(actor.slot_address("documents"), slot);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.text_list("archive").as_ptr(), archive);
    let current = actor.text_list("documents").as_ptr();
    actor.replace_for_test::<Upgraded>()?;
    assert_eq!(actor.text_list("documents").as_ptr(), current);
    Ok(())
}
#[test]
fn empty_and_scoped_collections_still_require_the_declared_predecessor()
-> std::result::Result<(), String> {
    let mut empty = blockui_runtime::Actor::for_test::<Original>()?;
    assert!(empty.replace_for_test::<Missing>().is_err());
    let error = empty.replace_for_test::<WrongPredecessor>().unwrap_err();
    assert!(
        error.contains("predecessor format requires migration"),
        "{error}"
    );
    empty.replace_for_test::<Automatic>()?;
    assert!(empty.text_list("documents").is_empty());
    let mut scoped = blockui_runtime::Actor::for_test::<NestedOriginal>()?;
    scoped.dispatch(blockui_runtime::Message {
        action: abi::key("nested.put"),
        value: 0,
    })?;
    scoped.replace_for_test::<NestedAutomatic>()?;
    assert_eq!(
        scoped.view.bindings[&abi::key("nested.document")].text,
        "Draft 🦀"
    );
    Ok(())
}
