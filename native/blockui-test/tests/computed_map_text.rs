use blockui::*;
use blockui_test::{Role, Screen};

const COUNT: State<i64> = State::<i64>::new("formatted.count", 0);
const DOUBLE: Computed<i64> = Computed::<i64>::new("formatted.double", |cx| {
    let count = COUNT.get(cx)?;
    if count == 3 {
        return Err(Error::Application("reject formatted memo".into()));
    }
    Ok(count * 2)
});
const EVEN: Computed<bool> =
    Computed::<bool>::new("formatted.even", |cx| Ok(COUNT.get(cx)? % 2 == 0));
const RATIO: Computed<f64> =
    Computed::<f64>::new("formatted.ratio", |cx| Ok(COUNT.get(cx)? as f64 / 2.0));
const TITLE: Computed<String> =
    Computed::<String>::new("formatted.title", |cx| Ok(COUNT.get(cx)?.to_string()));
const NUMBERS: Computed<Vec<i64>> =
    Computed::<Vec<i64>>::new("formatted.numbers", |cx| Ok(vec![COUNT.get(cx)?]));
const WORDS: Computed<Vec<String>> =
    Computed::<Vec<String>>::new("formatted.words", |cx| Ok(vec![COUNT.get(cx)?.to_string()]));
const STEP: Action = Action::new("formatted.step", |cx, value| {
    COUNT.update(cx, |count| Ok(count + value))
});
struct Formatted;
impl Actor for Formatted {
    const STATE: &'static [StateSpec] = &[COUNT.spec()];
    const ACTIONS: &'static [Action] = &[STEP];
    const COMPUTEDS: &'static [ComputedSpec] = &[
        DOUBLE.spec(),
        EVEN.spec(),
        RATIO.spec(),
        TITLE.spec(),
        NUMBERS.spec(),
        WORDS.spec(),
    ];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::new(DOUBLE.map_text(|v| format!("Double: {v}"))).into_view(),
            Text::new(EVEN.map_text(|v| format!("Even: {v}"))).into_view(),
            Text::new(RATIO.map_text(|v| format!("Ratio: {v}"))).into_view(),
            Text::new(TITLE.map_text(|v| format!("Title: {v}"))).into_view(),
            Text::new(NUMBERS.map_text(|v| format!("Numbers: {v:?}"))).into_view(),
            Text::new(WORDS.map_text(|v| format!("Words: {v:?}"))).into_view(),
            Button::label("Add one")
                .on_click_with(STEP, 1_i64)
                .into_view(),
            Button::label("Add two")
                .on_click_with(STEP, 2_i64)
                .into_view(),
        ]))
    }
}
#[test]
fn formatted_memos_update_independently_and_skip_unchanged_outputs()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Formatted>()?;
    let even = screen.get_by_text("Even: true")?;
    let even_before = screen.evaluations(even)?;
    let double = screen.get_by_text("Double: 0")?;
    let before = screen.evaluations(double)?;
    screen.click(screen.get_by_role(Role::Button, "Add two")?)?;
    assert_eq!(screen.get_by_text("Double: 4")?, double);
    for text in ["Ratio: 1", "Title: 2", "Numbers: [2]", "Words: [\"2\"]"] {
        screen.get_by_text(text)?;
    }
    assert_eq!(screen.evaluations(even)?, even_before);
    assert_eq!(screen.evaluations(double)?.text, before.text + 1);
    assert_eq!(screen.root_evaluations(), 1);
    let committed = screen.evaluations(double)?;
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Add one")?)
            .is_err()
    );
    assert_eq!(screen.get_by_text("Double: 4")?, double);
    assert_eq!(screen.evaluations(double)?, committed);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Formatted>()?;
    assert_eq!(screen.root_evaluations(), 2);
    assert_eq!(screen.get_by_text("Double: 4")?, double);
    screen.click(screen.get_by_role(Role::Button, "Add two")?)?;
    assert_eq!(screen.get_by_text("Double: 8")?, double);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}

struct PanickingFormatter;
impl Actor for PanickingFormatter {
    const STATE: &'static [StateSpec] = Formatted::STATE;
    const ACTIONS: &'static [Action] = Formatted::ACTIONS;
    const COMPUTEDS: &'static [ComputedSpec] = Formatted::COMPUTEDS;
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::new(DOUBLE.map_text(|value| {
                assert_ne!(value, 2, "reject formatter");
                format!("Formatted: {value}")
            }))
            .into_view(),
            Button::label("Add one")
                .on_click_with(STEP, 1_i64)
                .into_view(),
            Button::label("Add two")
                .on_click_with(STEP, 2_i64)
                .into_view(),
        ]))
    }
}
struct RejectedFormatter;
impl Actor for RejectedFormatter {
    const STATE: &'static [StateSpec] = Formatted::STATE;
    const ACTIONS: &'static [Action] = Formatted::ACTIONS;
    const COMPUTEDS: &'static [ComputedSpec] = Formatted::COMPUTEDS;
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Text::new(DOUBLE.map_text(|_| panic!("reject candidate formatter"))).into_view())
    }
}
#[test]
fn formatter_panic_rolls_back_and_keeps_the_previous_implementation_usable()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<PanickingFormatter>()?;
    let label = screen.get_by_text("Formatted: 0")?;
    let before = screen.evaluations(label)?;
    let slot = screen.actor().slot_address("formatted.count");
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Add one")?)
            .is_err()
    );
    assert_eq!(screen.actor().integer("formatted.count"), 0);
    assert_eq!(screen.get_by_text("Formatted: 0")?, label);
    assert_eq!(screen.evaluations(label)?, before);
    assert_eq!(screen.root_evaluations(), 1);
    screen.click(screen.get_by_role(Role::Button, "Add two")?)?;
    assert_eq!(screen.get_by_text("Formatted: 4")?, label);
    assert!(screen.replace::<RejectedFormatter>().is_err());
    assert_eq!(screen.root_evaluations(), 1);
    assert_eq!(screen.get_by_text("Formatted: 4")?, label);
    screen.click(screen.get_by_role(Role::Button, "Add two")?)?;
    assert_eq!(screen.get_by_text("Formatted: 8")?, label);
    assert_eq!(screen.actor().integer("formatted.count"), 4);
    assert_eq!(screen.actor().slot_address("formatted.count"), slot);
    assert_eq!(screen.root_evaluations(), 1);
    Ok(())
}

#[component]
struct FormattedCounter {
    count: State<i64>,
}
#[component_impl]
impl FormattedCounter {
    #[computed]
    fn double(&self) -> i64 {
        self.count.get() * 2
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|count| Ok(count + 1))
    }
    #[blockui::view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(
                self.double_computed()
                    .map_text(|value| format!("Double: {value}")),
            );
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[app]
struct FormattedPair {
    left: FormattedCounter,
    right: FormattedCounter,
}
#[app_impl]
impl FormattedPair {
    fn view(&self) -> View {
        Row::default().children(vec![self.left.render(), self.right.render()])
    }
}
#[test]
fn mapped_computed_default_identities_and_observers_are_isolated_per_component()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<FormattedPair>()?;
    let labels = screen.query_all_by_text("Double: 0");
    assert_eq!(labels.len(), 2);
    assert_ne!(labels[0], labels[1]);
    assert_ne!(
        screen.dependencies(labels[0])?.text,
        screen.dependencies(labels[1])?.text
    );
    let buttons = screen.query_all_by_role(Role::Button, "Increment");
    assert_eq!(buttons.len(), 2);
    let before: Vec<_> = labels
        .iter()
        .map(|label| screen.evaluations(*label))
        .collect::<std::result::Result<_, _>>()?;
    screen.click(buttons[0])?;
    let left = screen.get_by_text("Double: 2")?;
    let right = screen.get_by_text("Double: 0")?;
    let untouched = labels.iter().position(|label| *label == right).unwrap();
    assert_eq!(screen.evaluations(right)?, before[untouched]);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<FormattedPair>()?;
    assert_eq!(screen.get_by_text("Double: 2")?, left);
    assert_eq!(screen.get_by_text("Double: 0")?, right);
    let left_before = screen.evaluations(left)?;
    let right_before = screen.evaluations(right)?;
    screen.click(buttons[1])?;
    assert_eq!(screen.text(left)?, "Double: 2");
    assert_eq!(screen.text(right)?, "Double: 2");
    assert_eq!(screen.evaluations(left)?, left_before);
    assert_eq!(screen.evaluations(right)?.text, right_before.text + 1);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
