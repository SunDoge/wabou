use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Prefixed {
    count: State<i64>,
    ratio: State<f64>,
    enabled: State<bool>,
    title: State<String>,
}
#[app_impl]
impl Prefixed {
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
        let mut prefix = String::from("Count: ");
        let count = self.count.text_with_prefix(prefix.clone());
        prefix.clear();
        #[node]
        {
            Column::default();
            #[node]
            Text::new(count).key("prefix.count");
            #[node]
            Text::new(State::from(&self.count).text_with_prefix("Descriptor: "))
                .key("prefix.descriptor");
            #[node]
            Text::new(self.ratio.text_with_prefix("Ratio: ")).key("prefix.ratio");
            #[node]
            Text::new(self.enabled.text_with_prefix("Enabled: ")).key("prefix.enabled");
            #[node]
            Text::new(self.title.text_with_prefix("Title: ")).key("prefix.title");
            #[node]
            Text::new(self.even_computed().text_with_prefix("Even: ")).key("prefix.memo");
            #[node]
            Button::new("Advance", self.advance_action());
        }
    }
}
#[test]
fn prefixed_scalars_own_parameters_track_independently_and_skip_equal_memos()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Prefixed>()?;
    for text in [
        "Count: 0",
        "Descriptor: 0",
        "Ratio: 0",
        "Enabled: false",
        "Title: ",
        "Even: true",
    ] {
        screen.get_by_text(text)?;
    }
    let count = screen.get_by_key("prefix.count")?;
    let memo = screen.get_by_key("prefix.memo")?;
    let count_before = screen.evaluations(count)?;
    let memo_before = screen.evaluations(memo)?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    for text in [
        "Count: 2",
        "Descriptor: 2",
        "Ratio: 1.25",
        "Enabled: true",
        "Title: ready",
        "Even: true",
    ] {
        screen.get_by_text(text)?;
    }
    assert_eq!(screen.root_evaluations(), 1);
    assert_eq!(screen.evaluations(count)?.text, count_before.text + 1);
    assert_eq!(screen.evaluations(memo)?, memo_before);
    screen.replace::<Prefixed>()?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.text(count)?, "Count: 4");
    assert_eq!(screen.text(memo)?, "Even: true");
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}

#[component]
struct PrefixedCounter {
    count: State<i64>,
}
#[component_impl]
impl PrefixedCounter {
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
            Text::new(self.count.text_with_prefix("Count: "));
            #[node]
            Text::new(self.double_computed().text_with_prefix("Double: "));
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[app]
struct PrefixedPair {
    left: PrefixedCounter,
    right: PrefixedCounter,
}
#[app_impl]
impl PrefixedPair {
    fn view(&self) -> View {
        Row::default().children(vec![self.left.render(), self.right.render()])
    }
}
#[test]
fn prefixed_state_and_memo_default_identities_follow_component_instances()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<PrefixedPair>()?;
    let counts = screen.query_all_by_text("Count: 0");
    let doubles = screen.query_all_by_text("Double: 0");
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
    assert_eq!(screen.text(counts[0])?, "Count: 1");
    assert_eq!(screen.text(doubles[0])?, "Double: 2");
    assert_eq!(screen.evaluations(counts[1])?, untouched_count);
    assert_eq!(screen.evaluations(doubles[1])?, untouched_memo);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<PrefixedPair>()?;
    assert_eq!(screen.get_by_text("Count: 1")?, counts[0]);
    assert_eq!(screen.get_by_text("Double: 2")?, doubles[0]);
    let left_count = screen.evaluations(counts[0])?;
    let left_memo = screen.evaluations(doubles[0])?;
    screen.click(buttons[1])?;
    assert_eq!(screen.text(counts[1])?, "Count: 1");
    assert_eq!(screen.text(doubles[1])?, "Double: 2");
    assert_eq!(screen.evaluations(counts[0])?, left_count);
    assert_eq!(screen.evaluations(doubles[0])?, left_memo);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
