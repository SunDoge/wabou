#![cfg(feature = "testing")]
use blockui::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static ROOTS: AtomicUsize = AtomicUsize::new(0);
static FIRST: AtomicUsize = AtomicUsize::new(0);
static SECOND: AtomicUsize = AtomicUsize::new(0);
#[derive(Record)]
#[record(name = "reactivity.row")]
struct RowData {
    text: String,
}
#[actor(name = "record.reactivity")]
struct Model {
    #[state(key = "rows", codec = RowData)]
    rows: State<Vec<String>>,
    #[state(key = "captions")]
    captions: State<Vec<String>>,
}
#[actor_impl]
impl Model {
    #[action]
    fn batch_initialize(&self, cx: &Cx<'_>) -> Result<()> {
        TextMap::new(self.captions).update(cx, |entries| {
            entries.insert("first".into(), "First".into());
            entries.insert("second".into(), "Second".into());
            Ok(())
        })
    }
    #[action]
    fn batch_edit(&self, cx: &Cx<'_>) -> Result<()> {
        TextMap::new(self.captions).update(cx, |entries| {
            entries.insert("first".into(), "Changed first".into());
            Ok(())
        })
    }
    #[action]
    fn batch_conflict(&self, cx: &Cx<'_>) -> Result<()> {
        let map = TextMap::new(self.captions);
        map.update(cx, |entries| {
            map.insert(cx, "second", "must roll back")?;
            entries.clear();
            Ok(())
        })
    }
    #[action]
    fn record_batch_update(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<RowData>::new(self.rows).update(cx, |rows| {
            rows.insert(
                "first".into(),
                RowData {
                    text: "Batch first".into(),
                },
            );
            Ok(())
        })
    }
    #[action]
    fn record_batch_fail(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<RowData>::new(self.rows).update(cx, |rows| {
            rows.clear();
            Err(Error::Application("batch callback failed".into()))
        })
    }
    #[action]
    fn record_batch_conflict(&self, cx: &Cx<'_>) -> Result<()> {
        let map = RecordMap::<RowData>::new(self.rows);
        map.update(cx, |rows| {
            rows.clear();
            map.insert(
                cx,
                "second",
                &RowData {
                    text: "must roll back".into(),
                },
            )
        })
    }
    #[action]
    fn record_batch_invalid(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<RowData>::new(self.rows).update(cx, |rows| {
            rows.insert(
                "first".into(),
                RowData {
                    text: "x".repeat(abi::MAX_TEXT_LIST_BYTES + 1),
                },
            );
            Ok(())
        })
    }
    fn caption_first(cx: &Cx<'_>) -> Result<String> {
        Ok(TextMap::new(Self::new().captions)
            .get(cx, "first")?
            .unwrap_or_default())
    }
    fn caption_second(cx: &Cx<'_>) -> Result<String> {
        Ok(TextMap::new(Self::new().captions)
            .get(cx, "second")?
            .unwrap_or_default())
    }
    #[action]
    fn initialize(&self, cx: &Cx<'_>) -> Result<()> {
        let rows = RecordMap::<RowData>::new(self.rows);
        rows.insert(
            cx,
            "first",
            &RowData {
                text: "First".into(),
            },
        )?;
        rows.insert(
            cx,
            "second",
            &RowData {
                text: "Second".into(),
            },
        )
    }
    #[action]
    fn edit_first(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<RowData>::new(self.rows).edit(cx, "first", |row| {
            row.text = "Changed first".into();
            Ok(())
        })?;
        Ok(())
    }
    #[action]
    fn prepend(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<RowData>::new(self.rows).insert(
            cx,
            "aaa",
            &RowData {
                text: "Earlier key".into(),
            },
        )
    }
    #[action]
    fn remove_first(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<RowData>::new(self.rows).remove(cx, "first")
    }
    #[action]
    fn fail_edit(&self, cx: &Cx<'_>) -> Result<()> {
        self.edit_first(cx)?;
        Err(Error::Application("rollback record edit".into()))
    }
    fn first(cx: &Cx<'_>) -> Result<String> {
        FIRST.fetch_add(1, Ordering::Relaxed);
        Ok(RecordMap::<RowData>::new(Self::new().rows)
            .get(cx, "first")?
            .map_or_else(|| "Missing first".into(), |row| row.text))
    }
    fn second(cx: &Cx<'_>) -> Result<String> {
        SECOND.fetch_add(1, Ordering::Relaxed);
        Ok(RecordMap::<RowData>::new(Self::new().rows)
            .get(cx, "second")?
            .map_or_else(|| "Missing second".into(), |row| row.text))
    }
    #[computed(name = "second.memo")]
    fn second_memo(&self, cx: &Cx<'_>) -> Result<String> {
        Self::second(cx)
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        ROOTS.fetch_add(1, Ordering::Relaxed);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(Binding::named("first.binding", Self::first)).key("first"));
                __blockui_tree
                    .add(Text::new(Binding::named("second.binding", Self::second)).key("second"));
                __blockui_tree.add(Text::new(Self::second_memo_computed()).key("second.memo.text"));
                __blockui_tree.add(
                    Text::new(Binding::named("caption.first", Self::caption_first))
                        .key("caption.first"),
                );
                __blockui_tree.add(
                    Text::new(Binding::named("caption.second", Self::caption_second))
                        .key("caption.second"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn editing_one_record_does_not_evaluate_an_unrelated_row() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    actor.tell(Model::<DescriptorMode>::initialize_action(), 0)?;
    actor.tick()?;
    let state = actor.state_address();
    let slot = actor.slot_address("rows");
    let roots = ROOTS.load(Ordering::Relaxed);
    let first = FIRST.load(Ordering::Relaxed);
    let second = SECOND.load(Ordering::Relaxed);
    actor.tell(Model::<DescriptorMode>::edit_first_action(), 0)?;
    actor.tick()?;
    assert_eq!(
        actor.view.bindings[&abi::key("first")].text,
        "Changed first"
    );
    assert_eq!(actor.view.bindings[&abi::key("second")].text, "Second");
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("rows"), slot);
    assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
    assert_eq!(FIRST.load(Ordering::Relaxed), first + 1);
    assert_eq!(
        SECOND.load(Ordering::Relaxed),
        second,
        "unrelated record callback must remain cached"
    );
    actor.tell(Model::<DescriptorMode>::prepend_action(), 0)?;
    actor.tick()?;
    assert_eq!(FIRST.load(Ordering::Relaxed), first + 1);
    assert_eq!(SECOND.load(Ordering::Relaxed), second);
    actor.tell(Model::<DescriptorMode>::remove_first_action(), 0)?;
    actor.tick()?;
    assert_eq!(
        actor.view.bindings[&abi::key("first")].text,
        "Missing first"
    );
    assert_eq!(FIRST.load(Ordering::Relaxed), first + 2);
    assert_eq!(SECOND.load(Ordering::Relaxed), second);
    actor.tell(Model::<DescriptorMode>::initialize_action(), 0)?;
    actor.tick()?;
    assert_eq!(actor.view.bindings[&abi::key("first")].text, "First");
    assert_eq!(FIRST.load(Ordering::Relaxed), first + 3);
    assert_eq!(SECOND.load(Ordering::Relaxed), second);
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: Model::<DescriptorMode>::fail_edit_action().id,
                value: 0
            })
            .is_err()
    );
    assert_eq!(actor.view.bindings[&abi::key("first")].text, "First");
    actor.tell(Model::<DescriptorMode>::edit_first_action(), 0)?;
    actor.tick()?;
    assert_eq!(FIRST.load(Ordering::Relaxed), first + 4);
    assert_eq!(SECOND.load(Ordering::Relaxed), second);
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("Model.record_batch_update"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&abi::key("first")].text, "Batch first");
    assert_eq!(SECOND.load(Ordering::Relaxed), second);
    let committed = actor.text_list("rows").to_vec();
    let first_evaluations = actor.view.bindings[&abi::key("first")].evaluations;
    for (action, expected) in [
        ("Model.record_batch_fail", "batch callback failed"),
        (
            "Model.record_batch_conflict",
            "Text map changed during update",
        ),
        ("Model.record_batch_invalid", "Record payload exceeds 1 MiB"),
    ] {
        let error = actor
            .dispatch(blockui_runtime::Message {
                action: abi::key(action),
                value: 0,
            })
            .unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert_eq!(actor.text_list("rows"), committed);
        assert_eq!(actor.view.bindings[&abi::key("first")].text, "Batch first");
        assert_eq!(
            actor.view.bindings[&abi::key("first")].evaluations,
            first_evaluations
        );
        assert_eq!(SECOND.load(Ordering::Relaxed), second);
    }
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("Model.record_batch_update"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&abi::key("first")].text, "Batch first");
    Ok(())
}

mod key_churn {
    use blockui::*;
    #[actor(name = "churn")]
    struct Model {
        selection: State<i64>,
        entries: State<Vec<String>>,
    }
    #[actor_impl]
    impl Model {
        #[action]
        fn conflicting_update(&self, cx: &Cx<'_>) -> Result<()> {
            let map = TextMap::new(self.entries);
            map.update(cx, |entries| {
                map.insert(cx, "reentrant", "must roll back")?;
                entries.insert("staged".into(), "must not publish".into());
                Ok(())
            })
        }
        #[action]
        fn rejected_update(&self, cx: &Cx<'_>) -> Result<()> {
            TextMap::new(self.entries).update(cx, |entries| {
                entries.clear();
                Err(Error::Application("reject map edit".into()))
            })
        }
        #[action]
        fn next(&self, cx: &Cx<'_>) -> Result<()> {
            let next = self.selection.get(cx)? + 1;
            self.selection.set(cx, next)?;
            TextMap::new(self.entries).update(cx, |entries| {
                entries.clear();
                entries.insert(format!("key.{next}"), format!("Row {next}"));
                Ok(())
            })
        }
        fn selected(cx: &Cx<'_>) -> Result<String> {
            let model = Self::new();
            Ok(TextMap::new(model.entries)
                .get(cx, &format!("key.{}", model.selection.get(cx)?))?
                .unwrap_or_else(|| "Empty".into()))
        }
        fn view(&self, _: &Cx<'_>) -> Result<View> {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        Text::new(Binding::named("selected.binding", Self::selected))
                            .key("selected"),
                    );
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn published_rows_can_churn_past_cache_capacity_without_retaining_old_keys()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let state = actor.state_address();
        for next in 1..=abi::MAX_LIST_ITEMS + 64 {
            actor.tell(Model::<DescriptorMode>::next_action(), 0)?;
            actor.tick()?;
            assert_eq!(
                actor.view.bindings[&abi::key("selected")].text,
                format!("Row {next}")
            );
            assert_eq!(actor.state_address(), state);
        }
        let committed = actor.view.bindings[&abi::key("selected")].text.clone();
        let entries = actor.text_list("churn.entries").to_vec();
        for (action, reason) in [
            (
                Model::<DescriptorMode>::conflicting_update_action(),
                "Text map changed during update",
            ),
            (
                Model::<DescriptorMode>::rejected_update_action(),
                "reject map edit",
            ),
        ] {
            actor.tell(action, 0)?;
            let error = actor.tick().unwrap_err();
            assert!(error.contains(reason), "{error}");
            assert_eq!(actor.view.bindings[&abi::key("selected")].text, committed);
            assert_eq!(actor.state_address(), state);
            assert_eq!(actor.text_list("churn.entries"), entries);
        }
        actor.tell(Model::<DescriptorMode>::next_action(), 0)?;
        actor.tick()?;
        assert_eq!(
            actor.view.bindings[&abi::key("selected")].text,
            format!("Row {}", abi::MAX_LIST_ITEMS + 65)
        );
        Ok(())
    }
}

mod page_capacity {
    use blockui::*;
    #[actor(name = "capacity")]
    struct Model {
        page: State<i64>,
        entries: State<Vec<String>>,
    }
    #[actor_impl]
    impl Model {
        #[action]
        fn next(&self, cx: &Cx<'_>) -> Result<()> {
            self.page.update(cx, |page| Ok(page + 1))
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            let page = self.page.get(cx)?;
            let map = TextMap::new(self.entries);
            let count = if page == 3 {
                abi::MAX_LIST_ITEMS + 1
            } else {
                3000
            };
            for row in 0..count {
                assert!(map.get(cx, &format!("page.{page}.row.{row}"))?.is_none());
            }
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(format!("Page {page}")).key("page"));
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn page_transition_has_staging_headroom_but_rejects_oversized_publication()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()
            .map_err(|error| format!("initial: {error}"))?;
        for page in 1..=2 {
            actor.tell(Model::<DescriptorMode>::next_action(), 0)?;
            actor
                .tick()
                .map_err(|error| format!("page {page}: {error}"))?;
            assert_eq!(
                actor.view.bindings[&abi::key("page")].text,
                format!("Page {page}")
            );
        }
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: Model::<DescriptorMode>::next_action().id,
                    value: 0
                })
                .is_err()
        );
        assert_eq!(actor.integer("capacity.page"), 2);
        assert_eq!(actor.view.bindings[&abi::key("page")].text, "Page 2");
        Ok(())
    }
}

mod batch_codec_failure {
    use blockui::*;
    struct Rejecting;
    impl Record for Rejecting {
        const FORMAT: &'static str = "rejecting.row";
        fn encode(&self) -> Result<String> {
            Ok("undecodable".into())
        }
        fn decode(_: &str) -> Result<Self> {
            Err(Error::Application("codec rejected encoded row".into()))
        }
    }
    #[actor]
    struct CodecFailures {
        #[state(codec = Rejecting)]
        rows: State<Vec<String>>,
        count: State<i64>,
    }
    #[actor_impl]
    impl CodecFailures {
        #[action]
        fn reject(&self, cx: &Cx<'_>) -> Result<()> {
            RecordMap::<Rejecting>::new(self.rows).update(cx, |rows| {
                self.count.set(cx, 1)?;
                rows.insert("bad".into(), Rejecting);
                Ok(())
            })
        }
        #[action]
        fn recover(&self, cx: &Cx<'_>) -> Result<()> {
            RecordMap::<Rejecting>::new(self.rows).update(cx, |rows| {
                rows.clear();
                Ok(())
            })?;
            self.count.set(cx, 2)
        }
        fn view(&self, _: &Cx<'_>) -> View {
            Column::default().children(vec![Text::new(self.count).key("count").into_view()])
        }
    }
    #[test]
    fn encoded_row_validation_failure_rolls_back_callback_writes() -> std::result::Result<(), String>
    {
        let mut actor = blockui_runtime::Actor::for_test::<CodecFailures>()?;
        let evaluations = actor.view.bindings[&abi::key("count")].evaluations;
        let error = actor
            .dispatch(blockui_runtime::Message {
                action: <CodecFailures>::reject_action().id,
                value: 0,
            })
            .unwrap_err();
        assert!(error.contains("codec rejected encoded row"), "{error}");
        assert!(actor.text_list("CodecFailures.rows").is_empty());
        assert_eq!(actor.integer("CodecFailures.count"), 0);
        assert_eq!(actor.view.bindings[&abi::key("count")].text, "0");
        assert_eq!(
            actor.view.bindings[&abi::key("count")].evaluations,
            evaluations
        );
        actor.dispatch(blockui_runtime::Message {
            action: <CodecFailures>::recover_action().id,
            value: 0,
        })?;
        assert!(actor.text_list("CodecFailures.rows").is_empty());
        assert_eq!(actor.integer("CodecFailures.count"), 2);
        assert_eq!(actor.view.bindings[&abi::key("count")].text, "2");
        Ok(())
    }
}
