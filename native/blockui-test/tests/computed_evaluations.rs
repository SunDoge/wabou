use blockui::*;
use blockui_test::Screen;
const COUNT: State<i64> = State::<i64>::new("memo.count", 0);
const PARITY: Computed<i64> = Computed::<i64>::new("memo.parity", |cx| Ok(COUNT.get(cx)? % 2));
const UNUSED: Computed<bool> = Computed::<bool>::new("memo.unused", |cx| Ok(COUNT.get(cx)? > 0));
const MISSING: Computed<i64> = Computed::<i64>::new("memo.missing", |_| Ok(0));
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[COUNT.spec()];
    const COMPUTEDS: &'static [ComputedSpec] = &[PARITY.spec(), UNUSED.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::reactive(|cx| Ok(PARITY.get(cx)?.to_string()))
                .key("memo.label")
                .into_view(),
            Button::label("Same parity")
                .key("memo.same")
                .on_click(|cx| COUNT.update(cx, |n| Ok(n + 2))),
            Button::label("Other parity")
                .key("memo.other")
                .on_click(|cx| COUNT.update(cx, |n| Ok(n + 1))),
            Button::label("Reject").key("memo.reject").on_click(|cx| {
                COUNT.set(cx, 99)?;
                Err(Error::Application("reject".into()))
            }),
        ]))
    }
}
#[test]
fn typed_memo_snapshots_show_equality_pruning_without_evaluating_unused_memos()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<App>()?;
    let label = screen.get_by_key("memo.label")?;
    let memo = screen.computed_evaluations(PARITY)?;
    assert_eq!(
        screen.computed_dependencies(PARITY)?,
        std::collections::BTreeSet::from([blockui_test::Dependency::State(COUNT.key())])
    );
    assert!(screen.computed_dependencies(MISSING).is_err());
    let text = screen.evaluations(label)?;
    assert_eq!(
        screen.dependencies(label)?.text,
        std::collections::BTreeSet::from([blockui_test::Dependency::Computed(PARITY.key())])
    );
    let tree = screen.debug_tree();
    assert!(
        tree.contains(&format!("text-deps=[computed:{:#018x}]", PARITY.key())),
        "{tree}"
    );
    assert_eq!(screen.computed_evaluations(PARITY)?, memo);
    assert_eq!(screen.evaluations(label)?, text);
    // Admission validates all registered computeds once; inspection adds no work.
    assert_eq!(screen.computed_evaluations(UNUSED)?, 1);
    assert!(
        screen
            .computed_evaluations(MISSING)
            .unwrap_err()
            .contains("unregistered computed")
    );
    screen.click(screen.get_by_key("memo.same")?)?;
    assert_eq!(screen.computed_evaluations(PARITY)?, memo + 1);
    assert_eq!(screen.evaluations(label)?, text);
    screen.click(screen.get_by_key("memo.other")?)?;
    assert_eq!(screen.computed_evaluations(PARITY)?, memo + 2);
    assert_eq!(screen.evaluations(label)?.text, text.text + 1);
    assert_eq!(screen.text(label)?, "1");
    assert!(screen.click(screen.get_by_key("memo.reject")?).is_err());
    assert_eq!(screen.computed_evaluations(PARITY)?, memo + 2);
    assert_eq!(screen.computed_evaluations(UNUSED)?, 1);
    screen.replace::<App>()?;
    assert_eq!(screen.text(screen.get_by_key("memo.label")?)?, "1");
    assert!(screen.computed_evaluations(PARITY)? > 0);
    assert_eq!(screen.computed_evaluations(UNUSED)?, 1);
    Ok(())
}
