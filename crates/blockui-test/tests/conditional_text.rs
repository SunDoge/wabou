use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Conditional {
    enabled: State<bool>,
    count: State<i64>,
}

#[app_impl]
impl Conditional {
    #[action]
    fn advance(&self) -> Result<()> {
        self.enabled.update(|value| Ok(!value))?;
        self.count.update(|value| Ok(value + 2))
    }
    #[computed]
    fn even(&self) -> bool {
        self.count.get() % 2 == 0
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        let mut yes = String::from("On");
        let mut no = String::from("Off");
        let text = self.enabled.text_when(yes.clone(), no.clone());
        yes.clear();
        no.clear();
        #[node]
        {
            Column::default();
            #[node]
            Text::new(text).key("conditional.owned");
            #[node]
            Text::new(State::from(&self.enabled).text_when("Yes", "No"))
                .key("conditional.descriptor");
            #[node]
            Text::new(self.enabled.text_with_prefix("Raw: ")).key("conditional.raw");
            #[node]
            Text::new(self.even_computed().text_when("even", "odd")).key("conditional.memo");
            #[node]
            Button::new("Advance", self.advance_action());
        }
    }
}

#[test]
fn conditional_text_owns_labels_updates_independently_and_preserves_equal_memos()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Conditional>()?;
    let owned = screen.get_by_key("conditional.owned")?;
    let descriptor = screen.get_by_key("conditional.descriptor")?;
    let raw = screen.get_by_key("conditional.raw")?;
    let memo = screen.get_by_key("conditional.memo")?;
    screen.expect_text(owned, "Off")?;
    screen.expect_text(descriptor, "No")?;
    screen.expect_text(raw, "Raw: false")?;
    screen.expect_text(memo, "even")?;
    assert!(screen.root_dependencies().is_empty());
    let before = screen.evaluations(owned)?;
    let memo_before = screen.evaluations(memo)?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    screen.expect_text(owned, "On")?;
    screen.expect_text(descriptor, "Yes")?;
    screen.expect_text(raw, "Raw: true")?;
    assert_eq!(screen.root_evaluations(), 1);
    assert_eq!(screen.evaluations(owned)?.text, before.text + 1);
    assert_eq!(screen.evaluations(memo)?, memo_before);
    screen.replace::<Conditional>()?;
    screen.expect_text(owned, "On")?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    screen.expect_text(owned, "Off")?;
    screen.expect_text(descriptor, "No")?;
    screen.expect_text(raw, "Raw: false")?;
    screen.expect_text(memo, "even")?;
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}

#[component]
struct ConditionalToggle {
    count: State<bool>,
}
#[component_impl]
impl ConditionalToggle {
    #[computed]
    fn double(&self) -> bool {
        !self.count.get()
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(!value))
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.count.text_when("On", "Off"));
            #[node]
            Text::new(self.double_computed().text_when("inactive", "active"));
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[app]
struct ConditionalPair {
    left: ConditionalToggle,
    right: ConditionalToggle,
}
#[app_impl]
impl ConditionalPair {
    fn view(&self) -> View {
        Row::default().children(vec![self.left.render(), self.right.render()])
    }
}
#[test]
fn conditional_state_and_memo_follow_component_instances() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<ConditionalPair>()?;
    let counts = screen.query_all_by_text("Off");
    let doubles = screen.query_all_by_text("inactive");
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
    assert_eq!(screen.text(counts[0])?, "On");
    assert_eq!(screen.text(doubles[0])?, "active");
    assert_eq!(screen.evaluations(counts[1])?, untouched_count);
    assert_eq!(screen.evaluations(doubles[1])?, untouched_memo);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<ConditionalPair>()?;
    assert_eq!(screen.get_by_text("On")?, counts[0]);
    assert_eq!(screen.get_by_text("active")?, doubles[0]);
    let left_count = screen.evaluations(counts[0])?;
    let left_memo = screen.evaluations(doubles[0])?;
    screen.click(buttons[1])?;
    assert_eq!(screen.text(counts[1])?, "On");
    assert_eq!(screen.text(doubles[1])?, "active");
    assert_eq!(screen.evaluations(counts[0])?, left_count);
    assert_eq!(screen.evaluations(doubles[0])?, left_memo);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
