use blockui::*;
use blockui_test::{Role, Screen};
use std::cell::Cell;
type Even = bool;
type MemoResult<T> = std::result::Result<T, blockui::Error>;
thread_local! { static ROOT_BUILDS: Cell<usize> = const { Cell::new(0) }; }

#[actor(name = "scope.syntax.direct")]
struct Direct {
    count: State<i64>,
}
thread_local! { static DIRECT_PANEL_BUILDS: Cell<usize> = const { Cell::new(0) }; }
#[actor_impl]
impl Direct {
    #[action]
    fn increment(&self, cx: &Cx<'_>) -> Result<()> {
        self.count.update(cx, |value| Ok(value + 1))
    }
    #[scope]
    fn panel(&self, cx: &Cx<'_>) -> Result<View> {
        DIRECT_PANEL_BUILDS.with(|count| count.set(count.get() + 1));
        Ok(Column::default().children(vec![
            Text::new(format!("Direct: {}", self.count.get(cx)?)).into_view(),
            Button::new("Increment", Self::increment_action()).into_view(),
        ]))
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        ROOT_BUILDS.with(|count| count.set(count.get() + 1));
        let before = DIRECT_PANEL_BUILDS.with(Cell::get);
        let descriptor = Self::panel_scope();
        assert_eq!(descriptor.id(), <Self as Actor>::SCOPES[0].id());
        assert_eq!(DIRECT_PANEL_BUILDS.with(Cell::get), before);
        self.panel(cx)
    }
}

#[test]
fn direct_scope_method_call_tracks_in_parent_and_descriptor_is_lazy()
-> std::result::Result<(), String> {
    ROOT_BUILDS.with(|count| count.set(0));
    DIRECT_PANEL_BUILDS.with(|count| count.set(0));
    let mut screen = Screen::render::<Direct>()?;
    let text = screen.get_by_text("Direct: 0")?;
    assert!(screen.scope_of(text).is_err());
    let expected = std::collections::BTreeSet::from([blockui_test::Dependency::State(
        Direct::new().count.key(),
    )]);
    assert_eq!(screen.root_dependencies(), expected);
    assert!(screen.dependencies(text)?.text.is_empty());
    assert_eq!(ROOT_BUILDS.with(Cell::get), 1);
    assert_eq!(DIRECT_PANEL_BUILDS.with(Cell::get), 1);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.get_by_text("Direct: 1")?;
    assert_eq!(ROOT_BUILDS.with(Cell::get), 2);
    assert_eq!(DIRECT_PANEL_BUILDS.with(Cell::get), 2);
    assert_eq!(screen.root_dependencies(), expected);
    screen.replace::<Direct>()?;
    screen.expect_text(text, "Direct: 1")?;
    assert!(screen.scope_of(text).is_err());
    assert_eq!(screen.root_dependencies(), expected);
    assert!(screen.dependencies(text)?.text.is_empty());
    assert_eq!(ROOT_BUILDS.with(Cell::get), 3);
    assert_eq!(DIRECT_PANEL_BUILDS.with(Cell::get), 3);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.expect_text(text, "Direct: 2")?;
    assert_eq!(screen.root_dependencies(), expected);
    assert_eq!(ROOT_BUILDS.with(Cell::get), 4);
    assert_eq!(DIRECT_PANEL_BUILDS.with(Cell::get), 4);
    Ok(())
}

#[actor(name = "scope.syntax.plain")]
struct Plain {
    count: State<i64>,
}
#[actor_impl]
impl Plain {
    #[computed]
    fn even(&self, cx: &Cx<'_>) -> Even {
        self.count.get(cx).unwrap() % 2 == 0
    }
    #[action]
    fn increment(&self, cx: &Cx<'_>) -> Result<()> {
        self.count.update(cx, |n| Ok(n + 1))
    }
    #[scope]
    #[blockui::view]
    fn panel(&self, cx: &Cx<'_>, ui: &Ui) -> Result<View> {
        assert_eq!(self.even(cx), self.count.get(cx)? % 2 == 0);
        #[node]
        {
            Column::default();
            #[node]
            Text::new(format!("Count: {}", self.count.get(cx)?));
            #[node]
            Button::new("Increment", Self::increment_action());
        }
    }
    #[blockui::view]
    fn view(&self, _: &Cx<'_>, ui: &Ui) -> View {
        ROOT_BUILDS.with(|n| n.set(n.get() + 1));
        #[node]
        {
            Column::default();
            #[node]
            Self::panel_scope();
        }
    }
}

#[actor(name = "scope.syntax.bound")]
struct Bound {
    count: State<i64>,
    use_right: State<bool>,
    right: State<i64>,
}
#[actor_impl(render)]
impl Bound {
    #[action(bound)]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 1))
    }
    #[action(bound)]
    fn toggle(&self) -> Result<()> {
        self.use_right.update(|value| Ok(!value))
    }
    #[action(bound)]
    fn increment_right(&self) -> Result<()> {
        self.right.update(|value| Ok(value + 1))
    }
    #[computed(bound)]
    fn double(&self) -> MemoResult<i64> {
        Ok(self.count.get() * 2)
    }
    #[scope(bound)]
    #[blockui::view]
    fn panel(&self, ui: &Ui) -> View {
        let displayed = if self.use_right.get() {
            self.right.get()
        } else {
            self.count.get()
        };
        #[node]
        {
            Column::default();
            #[node]
            Text::new(format!("Count: {displayed}"));
            #[node]
            Text::new(self.double_computed()).key("double");
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
impl Render for BoundBound<'_, '_> {
    #[blockui::view]
    fn render(&self, ui: &Ui) -> View {
        ROOT_BUILDS.with(|n| n.set(n.get() + 1));
        #[node]
        {
            Column::default();
            #[node]
            self.panel_scope();
            #[node]
            Button::new("Toggle", self.toggle_action());
            #[node]
            Button::new("Increment right", self.increment_right_action());
        }
    }
}

fn exercise<A: Actor>() -> std::result::Result<(), String> {
    ROOT_BUILDS.with(|n| n.set(0));
    let mut screen = Screen::render::<A>()?;
    let button = screen.get_by_role(Role::Button, "Increment")?;
    let panel = screen.scope_of(button)?;
    let before = screen.scope_evaluations(panel)?;
    screen.click(button)?;
    screen.get_by_text("Count: 1")?;
    assert_eq!(screen.scope_evaluations(panel)?, before + 1);
    assert_eq!(ROOT_BUILDS.with(Cell::get), 1);
    screen.replace::<A>()?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.get_by_text("Count: 2")?;
    assert_eq!(ROOT_BUILDS.with(Cell::get), 2);
    Ok(())
}
#[test]
fn scoped_ui_syntax_tracks_dependencies_and_survives_replacement() -> std::result::Result<(), String>
{
    exercise::<Plain>()?;
    exercise::<Bound>()
}

#[test]
fn external_descriptor_accessors_resolve_without_explicit_mode_parameters()
-> std::result::Result<(), String> {
    const PANEL: Scope = <Bound>::panel_scope();
    const INCREMENT: Action = <Bound>::increment_action();
    const DOUBLE: Computed<i64> = <Bound>::double_computed();
    assert_eq!(PANEL.id(), <Bound as Actor>::SCOPES[0].id());
    assert_eq!(INCREMENT.id, <Bound as Actor>::ACTIONS[0].id);
    let mut screen = Screen::render::<Bound>()?;
    screen.get_by_text("0")?;
    let before = screen.computed_evaluations(DOUBLE)?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.get_by_text("2")?;
    assert_eq!(screen.computed_evaluations(DOUBLE)?, before + 1);
    Ok(())
}

#[test]
fn bound_scope_switches_actual_branch_dependencies() -> std::result::Result<(), String> {
    ROOT_BUILDS.with(|n| n.set(0));
    let mut screen = Screen::render::<Bound>()?;
    let panel = screen.scope_of(screen.get_by_text("Count: 0")?)?;
    let initial_dependencies = screen.scope_dependencies(panel)?;
    let state = Bound::new();
    let expected_left = std::collections::BTreeSet::from([
        blockui_test::Dependency::State(state.use_right.key()),
        blockui_test::Dependency::State(state.count.key()),
    ]);
    let expected_right = std::collections::BTreeSet::from([
        blockui_test::Dependency::State(state.use_right.key()),
        blockui_test::Dependency::State(state.right.key()),
    ]);
    assert_eq!(initial_dependencies, expected_left);
    assert!(screen.root_dependencies().is_empty());
    let double = screen.within(panel)?.get_by_text("0")?;
    let double_dependencies = screen.dependencies(double)?;
    assert_eq!(
        double_dependencies.text,
        std::collections::BTreeSet::from([blockui_test::Dependency::Computed(
            <Bound>::double_computed().key()
        ),])
    );
    assert!(double_dependencies.style.is_empty());
    assert_eq!(
        screen.computed_dependencies(<Bound>::double_computed())?,
        std::collections::BTreeSet::from([blockui_test::Dependency::State(state.count.key())])
    );
    let before = screen.scope_evaluations(panel)?;
    screen.click(screen.get_by_role(Role::Button, "Increment right")?)?;
    assert_eq!(screen.scope_evaluations(panel)?, before);
    screen.click(screen.get_by_role(Role::Button, "Toggle")?)?;
    screen.get_by_text("Count: 1")?;
    assert_eq!(screen.scope_dependencies(panel)?, expected_right);
    assert_eq!(screen.dependencies(double)?, double_dependencies);
    let switched = screen.scope_evaluations(panel)?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.scope_evaluations(panel)?, switched);
    screen.get_by_text("2")?; // The independent computed text still updates.
    screen.click(screen.get_by_role(Role::Button, "Increment right")?)?;
    screen.get_by_text("Count: 2")?;
    assert_eq!(screen.scope_evaluations(panel)?, switched + 1);
    screen.click(screen.get_by_role(Role::Button, "Toggle")?)?;
    screen.get_by_text("Count: 1")?;
    assert_eq!(screen.scope_dependencies(panel)?, initial_dependencies);
    assert_eq!(ROOT_BUILDS.with(Cell::get), 1);
    screen.replace::<Bound>()?;
    assert_eq!(screen.scope_dependencies(panel)?, expected_left);
    assert_eq!(screen.dependencies(double)?, double_dependencies);
    assert!(screen.root_dependencies().is_empty());
    let replaced = screen.scope_evaluations(panel)?;
    screen.click(screen.get_by_role(Role::Button, "Increment right")?)?;
    assert_eq!(screen.scope_evaluations(panel)?, replaced);
    screen.expect_text(double, "2")?;
    screen.click(screen.get_by_role(Role::Button, "Toggle")?)?;
    screen.get_by_text("Count: 3")?;
    assert_eq!(screen.scope_dependencies(panel)?, expected_right);
    assert_eq!(screen.dependencies(double)?, double_dependencies);
    assert_eq!(ROOT_BUILDS.with(Cell::get), 2);
    Ok(())
}

#[actor(name = "scope.leaf")]
struct Leaf {
    count: State<i64>,
}
#[actor_impl(render)]
impl Leaf {
    #[computed(bound)]
    fn doubled(&self) -> i64 {
        self.count.get() * 2
    }
    #[action(bound)]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    #[scope(bound)]
    #[blockui::view]
    fn panel(&self, ui: &Ui) -> View {
        assert_eq!(self.doubled(), self.count.get() * 2);
        #[node]
        Text::new(format!("Leaf count: {}", self.count.get())).key("leaf.value");
    }
}
impl Render for LeafBound<'_, '_> {
    #[blockui::view]
    fn render(&self, ui: &Ui) -> View {
        ROOT_BUILDS.with(|n| n.set(n.get() + 1));
        #[node]
        {
            Column::default();
            #[node]
            self.panel_scope();
            #[node]
            Text::new(self.doubled_computed()).key("leaf.doubled");
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[test]
fn macro_leaf_scope_updates_locally_and_preserves_identity() -> std::result::Result<(), String> {
    ROOT_BUILDS.with(|n| n.set(0));
    let mut screen = Screen::render::<Leaf>()?;
    let leaf = screen.get_by_text("Leaf count: 0")?;
    let scope = screen.scope_of(leaf)?;
    let before = screen.scope_evaluations(scope)?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.get_by_text("Leaf count: 1")?, leaf);
    screen.get_by_text("2")?;
    assert_eq!(screen.scope_evaluations(scope)?, before + 1);
    assert_eq!(ROOT_BUILDS.with(Cell::get), 1);
    screen.replace::<Leaf>()?;
    assert_eq!(screen.get_by_text("Leaf count: 1")?, leaf);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.get_by_text("Leaf count: 2")?;
    Ok(())
}

#[actor(name = "computed.panic")]
struct PanicComputed {
    count: State<i64>,
}
#[actor_impl(render)]
impl PanicComputed {
    #[computed(bound)]
    fn value(&self) -> i64 {
        let value = self.count.get();
        assert_ne!(value, 2, "intentional computed failure");
        value * 10
    }
    #[action(bound)]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))?;
        let _: i64 = self.value();
        Ok(())
    }
    #[action(bound)]
    fn lazy_increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    #[action(bound)]
    fn reset(&self) -> Result<()> {
        self.count.set(0)
    }
}
impl Render for PanicComputedBound<'_, '_> {
    #[blockui::view]
    fn render(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.value_computed()).key("panic.value");
            #[node]
            Button::new("Increment", self.increment_action());
            #[node]
            Button::new("Reset", self.reset_action());
            #[node]
            Button::new("Lazy increment", self.lazy_increment_action());
        }
    }
}
#[test]
fn infallible_computed_failure_rolls_back_the_turn_and_recovers_after_replacement()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<PanicComputed>()?;
    let value = screen.get_by_text("0")?;
    let increment = screen.get_by_role(Role::Button, "Increment")?;
    screen.click(increment)?;
    assert_eq!(screen.get_by_text("10")?, value);
    let version = screen.actor().implementation_version();
    assert!(screen.click(increment).is_err());
    assert_eq!(screen.actor().implementation_version(), version);
    assert_eq!(screen.get_by_text("10")?, value);
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Lazy increment")?)
            .is_err()
    );
    assert_eq!(screen.get_by_text("10")?, value);
    screen.replace::<PanicComputed>()?;
    assert_eq!(screen.get_by_text("10")?, value);
    screen.click(screen.get_by_role(Role::Button, "Reset")?)?;
    assert_eq!(screen.get_by_text("0")?, value);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.get_by_text("10")?, value);
    Ok(())
}
