#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record, Default)]
#[record(name = "list.row")]
struct Row {
    id: i64,
}
#[derive(Record, Default)]
#[record(name = "list.row", previous = Row)]
struct EvolvedRow {
    id: i64,
    #[record(default = "untitled".to_owned())]
    title: String,
}
#[derive(Record, Default)]
#[record(name = "list.row")]
struct UnsupportedRow {
    id: i64,
    enabled: bool,
}
macro_rules! collection {
    ($actor:ident, $row:ty) => {
        #[actor(name = "list.actor")]
        struct $actor {
            rows: State<Vec<$row>>,
        }
        #[actor_impl]
        impl $actor {
            #[action(name = "list.append")]
            fn append(&self, cx: &Cx<'_>, id: i64) -> Result<()> {
                self.rows.update(cx, |rows| {
                    let mut row = <$row>::default();
                    row.id = id;
                    rows.push(row);
                    if id < 0 {
                        return Err(Error::Application("rejected row".into()));
                    }
                    Ok(())
                })
            }
            #[action(name = "list.reverse")]
            fn reverse(&self, cx: &Cx<'_>) -> Result<()> {
                self.rows.update(cx, |rows| {
                    rows.reverse();
                    Ok(())
                })
            }
            fn view(&self, cx: &Cx<'_>) -> Result<View> {
                let rows = self.rows.get(cx)?;
                let ui = Ui::new();
                {
                    let _root = ui.scope(Column::default());
                    ui.add(Text::new(rows.len().to_string()).key("list.length"));
                    for row in rows {
                        ui.add(
                            Button::new(format!("Row {}", row.id), Self::append_action())
                                .key(Key::named("list.row").scoped(row.id as u64))
                                .on_click_with(Self::append_action(), row.id + 100),
                        );
                    }
                }
                ui.try_finish()
            }
        }
    };
}
collection!(Rows, Row);
collection!(CompatibleRows, Row);
collection!(EvolvedRows, EvolvedRow);
collection!(UnsupportedRows, UnsupportedRow);

#[test]
fn record_list_retains_storage_rolls_back_and_migrates_each_element() {
    let mut actor = blockui_runtime::Actor::for_test::<Rows>().unwrap();
    actor.tell(Rows::actions().append, 7).unwrap();
    actor.tick().unwrap();
    let slot = actor.slot_address("list.actor.rows");
    let bytes = actor.text("list.actor.rows").to_owned();
    let buffer = actor.text("list.actor.rows").as_ptr();
    actor.tell(Rows::actions().append, 8).unwrap();
    actor.replace_for_test::<CompatibleRows>().unwrap();
    assert_eq!(actor.text("list.actor.rows"), bytes);
    assert_eq!(actor.text("list.actor.rows").as_ptr(), buffer);
    actor.tick().unwrap();
    let rows: Vec<Row> = record_codec::decode_record(actor.text("list.actor.rows")).unwrap();
    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), [7, 8]);
    let bytes = actor.text("list.actor.rows").to_owned();
    let buffer = actor.text("list.actor.rows").as_ptr();
    actor.tell(Rows::actions().append, -1).unwrap();
    assert!(actor.tick().unwrap_err().contains("rejected row"));
    assert_eq!(actor.text("list.actor.rows"), bytes);
    assert_eq!(actor.text("list.actor.rows").as_ptr(), buffer);
    assert!(actor.replace_for_test::<UnsupportedRows>().is_err());
    assert_eq!(actor.text("list.actor.rows").as_ptr(), buffer);
    actor.replace_for_test::<EvolvedRows>().unwrap();
    let rows: Vec<EvolvedRow> = record_codec::decode_record(actor.text("list.actor.rows")).unwrap();
    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), [7, 8]);
    assert!(rows.iter().all(|row| row.title == "untitled"));
    assert_eq!(actor.slot_address("list.actor.rows"), slot);
}

#[test]
fn empty_record_list_schema_and_capacity_are_checked() {
    let mut actor = blockui_runtime::Actor::for_test::<Rows>().unwrap();
    assert!(actor.replace_for_test::<UnsupportedRows>().is_err());
    actor.replace_for_test::<EvolvedRows>().unwrap();
    let rows: Vec<EvolvedRow> = record_codec::decode_record(actor.text("list.actor.rows")).unwrap();
    assert!(rows.is_empty());
    let too_many: Vec<Row> = (0..=record_codec::MAX_RECORD_ITEMS)
        .map(|id| Row { id: id as i64 })
        .collect();
    assert!(record_codec::encode_record(&too_many).is_err());
    let one = record_codec::encode_record(&vec![Row { id: 1 }]).unwrap();
    assert!(record_codec::decode_record::<Vec<EvolvedRow>>(&one).is_err());
    assert!(Vec::<EvolvedRow>::upgrade_encoded("wrong\n").is_err());
}

#[derive(Record, Default)]
#[record(name = "list.manual-row")]
struct ManualRow {
    id: i64,
    doubled: i64,
}
impl RecordUpgrade<Row> for ManualRow {
    fn upgrade(previous: Row) -> Result<Self> {
        if previous.id == 8 {
            return Err(Error::Application("cannot upgrade row 8".into()));
        }
        Ok(Self {
            id: previous.id,
            doubled: previous.id.checked_mul(2).ok_or(Error::Overflow)?,
        })
    }
}
#[actor(name = "list.actor")]
struct ManualRows {
    rows: State<Vec<ManualRow>>,
}
#[actor_impl]
impl ManualRows {
    fn migrate(&self, cx: &Cx<'_>, _: u64) -> Result<()> {
        let previous = self.rows.previous::<Vec<Row>>(cx)?;
        self.rows.set(cx, &Vec::<ManualRow>::upgrade(previous)?)
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(self.rows.get(cx)?.len().to_string()));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn explicit_collection_upgrade_uses_element_conversion_and_rolls_back_failure() {
    for ids in [vec![7], vec![7, 8]] {
        let mut actor = blockui_runtime::Actor::for_test::<Rows>().unwrap();
        for id in &ids {
            actor.tell(Rows::actions().append, *id).unwrap();
            actor.tick().unwrap();
        }
        let slot = actor.slot_address("list.actor.rows");
        let bytes = actor.text("list.actor.rows").to_owned();
        let buffer = actor.text("list.actor.rows").as_ptr();
        if ids.len() == 1 {
            actor.replace_for_test::<ManualRows>().unwrap();
            let rows: Vec<ManualRow> =
                record_codec::decode_record(actor.text("list.actor.rows")).unwrap();
            assert_eq!(rows[0].id, 7);
            assert_eq!(rows[0].doubled, 14);
        } else {
            assert!(
                actor
                    .replace_for_test::<ManualRows>()
                    .unwrap_err()
                    .contains("cannot upgrade row 8")
            );
            assert_eq!(actor.text("list.actor.rows"), bytes);
            assert_eq!(actor.text("list.actor.rows").as_ptr(), buffer);
            actor.tell(Rows::actions().append, 9).unwrap();
            actor.tick().unwrap();
        }
        assert_eq!(actor.slot_address("list.actor.rows"), slot);
    }
}

#[derive(Record, Default)]
#[record(name = "list.reactive-row")]
struct ReactiveRow {
    id: i64,
    title: String,
    edits: i64,
}
#[actor(name = "list.reactive")]
struct ReactiveRows {
    #[state(default = vec![ReactiveRow { id: 1, title: "first".into(), edits: 0 }, ReactiveRow { id: 2, title: "second".into(), edits: 0 }])]
    rows: State<Vec<ReactiveRow>>,
}
#[actor_impl]
impl ReactiveRows {
    #[computed(name = "list.first-title")]
    fn first_title(&self, cx: &Cx<'_>) -> Result<String> {
        ReactiveRow::title_field().get_from_list(self.rows, cx, ReactiveRow::id_field(), 1)
    }
    #[action]
    fn edit(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        self.rows.update(cx, |rows| {
            match mode {
                0 => rows.iter_mut().find(|row| row.id == 1).unwrap().edits += 1,
                1 => rows
                    .iter_mut()
                    .find(|row| row.id == 2)
                    .unwrap()
                    .title
                    .push('!'),
                2 => rows.reverse(),
                3 => rows
                    .iter_mut()
                    .find(|row| row.id == 1)
                    .unwrap()
                    .title
                    .push('!'),
                4 => rows.push(ReactiveRow {
                    id: 1,
                    title: "duplicate".into(),
                    edits: 0,
                }),
                _ => rows.retain(|row| row.id != 1),
            }
            Ok(())
        })
    }
    #[action]
    fn title_changed(&self, cx: &Cx<'_>, title: String) -> Result<()> {
        self.rows.update(cx, |rows| {
            let row = rows
                .iter_mut()
                .find(|row| row.id == 1)
                .ok_or(Error::UnknownAction)?;
            row.title = title.trim().to_uppercase();
            row.edits += 1;
            if title.contains('!') {
                return Err(Error::Application("invalid list title".into()));
            }
            Ok(())
        })
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        let source =
            ReactiveRow::title_field().text_from_list(self.rows, ReactiveRow::id_field(), 1);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(Self::first_title_computed()).key("list.computed"));
                __blockui_tree.add(Text::new(source).key("list.caption"));
                __blockui_tree.add(
                    TextInput::field(source, Self::title_changed_action())
                        .key("list.input")
                        .label("Title"),
                );
                __blockui_tree.add(
                    TextArea::field(source, Self::title_changed_action())
                        .key("list.area")
                        .label("Body"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn keyed_record_list_field_prunes_unrelated_changes_and_reorder() {
    let mut actor = blockui_runtime::Actor::for_test::<ReactiveRows>().unwrap();
    let evaluations = actor.computed_evaluations("list.first-title");
    assert!(evaluations > 0);
    let keys = [
        abi::key("list.caption"),
        abi::key("list.input"),
        abi::key("list.area"),
    ];
    let bindings = keys.map(|key| actor.view.bindings[&key].evaluations);
    for mode in [0, 1, 2] {
        actor.tell(ReactiveRows::actions().edit, mode).unwrap();
        actor.tick().unwrap();
        assert_eq!(actor.computed_evaluations("list.first-title"), evaluations);
        assert_eq!(
            keys.map(|key| actor.view.bindings[&key].evaluations),
            bindings
        );
    }
    actor.replace_for_test::<ReactiveRows>().unwrap();
    let evaluations = actor.computed_evaluations("list.first-title");
    actor.tell(ReactiveRows::actions().edit, 3).unwrap();
    actor.tick().unwrap();
    assert_eq!(
        actor.computed_evaluations("list.first-title"),
        evaluations + 1
    );
    let bytes = actor.text("list.reactive.rows").to_owned();
    let buffer = actor.text("list.reactive.rows").as_ptr();
    for mode in [4, 5] {
        actor.tell(ReactiveRows::actions().edit, mode).unwrap();
        assert!(actor.tick().is_err());
        assert_eq!(actor.text("list.reactive.rows"), bytes);
        assert_eq!(actor.text("list.reactive.rows").as_ptr(), buffer);
    }
}

#[test]
fn direct_list_text_editors_share_controlled_model_and_rollback() {
    let mut actor = blockui_runtime::Actor::for_test::<ReactiveRows>().unwrap();
    let keys = [
        abi::key("list.caption"),
        abi::key("list.input"),
        abi::key("list.area"),
    ];
    actor.edit_text(keys[1], " héllo ").unwrap();
    assert!(
        keys.iter()
            .all(|key| actor.view.bindings[key].text == "HÉLLO")
    );
    assert_eq!(
        actor.view.bindings[&keys[1]].semantic_label.as_deref(),
        Some("Title")
    );
    let bytes = actor.text("list.reactive.rows").to_owned();
    let buffer = actor.text("list.reactive.rows").as_ptr();
    assert!(
        actor
            .edit_text(keys[2], "bad!")
            .unwrap_err()
            .contains("invalid list title")
    );
    assert_eq!(actor.text("list.reactive.rows"), bytes);
    assert_eq!(actor.text("list.reactive.rows").as_ptr(), buffer);
    actor.tell(ReactiveRows::actions().edit, 2).unwrap();
    actor.tick().unwrap();
    actor.replace_for_test::<ReactiveRows>().unwrap();
    actor.edit_text(keys[2], "reloaded").unwrap();
    assert!(
        keys.iter()
            .all(|key| actor.view.bindings[key].text == "RELOADED")
    );
}
