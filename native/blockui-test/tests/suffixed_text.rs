use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Suffixed {
    count: State<i64>,
    ratio: State<f64>,
    enabled: State<bool>,
    title: State<String>,
}
#[app_impl]
impl Suffixed {
    #[action]
    fn advance(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 2))?;
        self.ratio.set(1.25)?;
        self.enabled.set(true)?;
        self.title.set("ready")
    }
    #[computed]
    fn even(&self) -> bool {
        self.count.get() % 2 == 0
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        let mut suffix = String::from(" items");
        let count = self.count.text_with_suffix(suffix.clone());
        suffix.clear();
        #[node]
        {
            Column::default();
            #[node]
            Text::new(count).key("suffix.count");
            #[node]
            Text::new(self.count.text_with_prefix("Count: ")).key("mixed.prefix");
            #[node]
            Text::new(State::from(&self.count).text_with_suffix(" units")).key("suffix.descriptor");
            #[node]
            Text::new(self.ratio.text_with_suffix(" ratio")).key("suffix.ratio");
            #[node]
            Text::new(self.enabled.text_with_suffix(" enabled")).key("suffix.enabled");
            #[node]
            Text::new(self.title.text_with_suffix(" ✓")).key("suffix.title");
            #[node]
            Text::new(self.even_computed().text_with_suffix(" even")).key("suffix.memo");
            #[node]
            Button::new("Advance", self.advance_action());
        }
    }
}
#[test]
fn suffixed_scalars_own_parameters_track_independently_and_skip_equal_memos()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Suffixed>()?;
    for text in [
        "0 items",
        "0 units",
        "0 ratio",
        "false enabled",
        " ✓",
        "true even",
    ] {
        screen.get_by_text(text)?;
    }
    let count = screen.get_by_key("suffix.count")?;
    let memo = screen.get_by_key("suffix.memo")?;
    let count_before = screen.evaluations(count)?;
    let memo_before = screen.evaluations(memo)?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    for text in [
        "2 items",
        "2 units",
        "1.25 ratio",
        "true enabled",
        "ready ✓",
        "true even",
    ] {
        screen.get_by_text(text)?;
    }
    screen.expect_text(screen.get_by_key("mixed.prefix")?, "Count: 2")?;
    assert_eq!(screen.root_evaluations(), 1);
    assert_eq!(screen.evaluations(count)?.text, count_before.text + 1);
    assert_eq!(screen.evaluations(memo)?, memo_before);
    screen.replace::<Suffixed>()?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.text(count)?, "4 items");
    assert_eq!(screen.text(memo)?, "true even");
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}

#[component]
struct SuffixedCounter {
    count: State<i64>,
}
#[component_impl]
impl SuffixedCounter {
    #[computed]
    fn double(&self) -> i64 {
        self.count.get() * 2
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.count.text_with_suffix(" items"));
            #[node]
            Text::new(self.double_computed().text_with_suffix(" doubled"));
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[app]
struct SuffixedPair {
    left: SuffixedCounter,
    right: SuffixedCounter,
}
#[app_impl]
impl SuffixedPair {
    fn view(&self) -> View {
        Row::default().children(vec![self.left.render(), self.right.render()])
    }
}
#[test]
fn suffixed_state_and_memo_default_identities_follow_component_instances()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<SuffixedPair>()?;
    let counts = screen.query_all_by_text("0 items");
    let doubles = screen.query_all_by_text("0 doubled");
    let buttons = screen.query_all_by_role(Role::Button, "Increment");
    assert_eq!(counts.len(), 2);
    assert_eq!(doubles.len(), 2);
    assert_eq!(buttons.len(), 2);
    assert_ne!(counts[0], counts[1]);
    assert_ne!(doubles[0], doubles[1]);
    assert_ne!(
        screen.dependencies(counts[0])?.text,
        screen.dependencies(counts[1])?.text
    );
    assert_ne!(
        screen.dependencies(doubles[0])?.text,
        screen.dependencies(doubles[1])?.text
    );
    let untouched_count = screen.evaluations(counts[1])?;
    let untouched_memo = screen.evaluations(doubles[1])?;
    screen.click(buttons[0])?;
    assert_eq!(screen.text(counts[0])?, "1 items");
    assert_eq!(screen.text(doubles[0])?, "2 doubled");
    assert_eq!(screen.evaluations(counts[1])?, untouched_count);
    assert_eq!(screen.evaluations(doubles[1])?, untouched_memo);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<SuffixedPair>()?;
    assert_eq!(screen.get_by_text("1 items")?, counts[0]);
    assert_eq!(screen.get_by_text("2 doubled")?, doubles[0]);
    let left_count = screen.evaluations(counts[0])?;
    let left_memo = screen.evaluations(doubles[0])?;
    screen.click(buttons[1])?;
    assert_eq!(screen.text(counts[1])?, "1 items");
    assert_eq!(screen.text(doubles[1])?, "2 doubled");
    assert_eq!(screen.evaluations(counts[0])?, left_count);
    assert_eq!(screen.evaluations(doubles[0])?, left_memo);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
