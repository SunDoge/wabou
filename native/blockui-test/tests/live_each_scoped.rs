use blockui::*;
use blockui_test::{Role, Screen};
#[app]
struct ScopedList {
    order: State<Vec<String>>,
    a: State<String>,
    b: State<String>,
}
#[app_impl]
impl ScopedList {
    #[computed]
    fn keys(&self) -> Vec<String> {
        self.order.get()
    }
    #[action]
    fn reset(&self) -> Result<()> {
        self.order.set(&["a".into(), "b".into()])
    }
    #[action]
    fn reverse(&self) -> Result<()> {
        self.order.update(|values| {
            values.reverse();
            Ok(())
        })
    }
    #[action]
    fn remove_a(&self) -> Result<()> {
        self.order.set(&["b".into()])
    }
    #[action]
    fn duplicate(&self) -> Result<()> {
        self.order.set(&["b".into(), "b".into()])
    }
    #[action]
    fn empty_key(&self) -> Result<()> {
        self.order.set(&[String::new()])
    }
    #[view(block)]
    fn row(&self, key: &str) -> View {
        let value = if key == "a" {
            self.a.handle()
        } else {
            self.b.handle()
        };
        #[ui(Column::default())]
        {
            #[ui]
            Field::new(key, TextInput::new(value).key("input"));
            #[ui]
            Text::new(if key == "a" {
                self.a.get()
            } else {
                self.b.get()
            })
            .key("value");
        }
    }
    fn view(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Button::new("Reset", self.reset_action());
            #[ui]
            Button::new("Reverse", self.reverse_action());
            #[ui]
            Button::new("Remove", self.remove_a_action());
            #[ui]
            Button::new("Duplicate", self.duplicate_action());
            #[ui]
            Button::new("Empty key", self.empty_key_action());
            #[ui]
            self.keys_computed()
                .each_scoped(self.live_each(|this, key| this.row(&key)))
                .mount("rows");
        }
    }
}
#[test]
fn row_structure_updates_without_rebuilding_the_list_or_sibling() -> std::result::Result<(), String>
{
    let mut screen = Screen::render::<ScopedList>()?;
    screen.click(screen.get_by_role(Role::Button, "Reset")?)?;
    let rows = screen.get_by_key("rows")?;
    let a = screen.within(rows)?.get_by_role(Role::Textbox, "a")?;
    let b = screen.within(rows)?.get_by_role(Role::Textbox, "b")?;
    let input_order = |screen: &Screen| {
        let view = screen.actor().projected_view();
        view.order
            .iter()
            .copied()
            .filter(|key| view.bindings[key].node.kind == abi::ENTRY)
            .collect::<Vec<_>>()
    };
    let original_order = input_order(&screen);
    assert_eq!(original_order.len(), 2);
    let a_scope = screen.scope_of(a)?;
    let b_scope = screen.scope_of(b)?;
    let a_before = screen.scope_evaluations(a_scope)?;
    let b_before = screen.scope_evaluations(b_scope)?;
    let before = screen.scope_evaluations(rows)?;
    let b_evaluations = screen.evaluations(b)?;
    screen.input(a, "Edited A")?;
    assert_eq!(screen.scope_evaluations(rows)?, before);
    assert_eq!(screen.scope_evaluations(a_scope)?, a_before + 1);
    assert_eq!(screen.scope_evaluations(b_scope)?, b_before);
    assert_eq!(screen.evaluations(b)?, b_evaluations);
    let a_before_reorder = screen.scope_evaluations(a_scope)?;
    let b_before_reorder = screen.scope_evaluations(b_scope)?;
    screen.click(screen.get_by_role(Role::Button, "Reverse")?)?;
    assert_eq!(screen.scope_evaluations(a_scope)?, a_before_reorder);
    assert_eq!(screen.scope_evaluations(b_scope)?, b_before_reorder);
    assert_eq!(
        input_order(&screen),
        original_order.iter().rev().copied().collect::<Vec<_>>()
    );
    assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "a")?, a);
    assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "b")?, b);
    assert_eq!(screen.text(a)?, "Edited A");
    screen.input(b, "Edited B")?;
    screen.click(screen.get_by_role(Role::Button, "Remove")?)?;
    assert!(screen.input(a, "stale").is_err());
    assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "b")?, b);
    assert_eq!(screen.scope_evaluations(b_scope)?, b_before_reorder + 1);
    let stable = screen.scope_evaluations(rows)?;
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Duplicate")?)
            .unwrap_err()
            .contains("nonempty and unique")
    );
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Empty key")?)
            .unwrap_err()
            .contains("nonempty and unique")
    );
    assert_eq!(screen.scope_evaluations(rows)?, stable);
    assert_eq!(screen.scope_evaluations(b_scope)?, b_before_reorder + 1);
    assert_eq!(screen.text(b)?, "Edited B");
    screen.click(screen.get_by_role(Role::Button, "Reset")?)?;
    assert_eq!(
        screen.text(screen.within(rows)?.get_by_role(Role::Textbox, "a")?)?,
        "Edited A"
    );
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<ScopedList>()?;
    assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "b")?, b);
    screen.input(b, "After reload")?;
    assert_eq!(screen.text(b)?, "After reload");
    Ok(())
}

const PREFIX: State<String> = State::<String>::new("rows.prefix");
const CONSTANT_KEYS: Computed<Vec<String>> =
    Computed::<Vec<String>>::new("rows.constant.keys", |_| Ok(vec!["a".into(), "b".into()]));
struct Parameters;
impl Actor for Parameters {
    const STATE: &'static [StateSpec] = &[PREFIX.spec()];
    const COMPUTEDS: &'static [ComputedSpec] = &[CONSTANT_KEYS.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let prefix = PREFIX.get(cx)?;
        Ok(Column::default().children(Vec::from([
            Button::label("Change")
                .key("change")
                .on_click(|cx| PREFIX.set(cx, "New")),
            CONSTANT_KEYS
                .each_scoped(LiveEach::captured(move |_, key| {
                    Ok(Text::new(format!("{prefix} {key}")))
                }))
                .mount("parameter.rows"),
        ])))
    }
}
#[test]
fn changed_parent_parameters_invalidate_cached_row_builders() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Parameters>()?;
    let text = screen.get_by_text(" a")?;
    let row = screen.scope_of(text)?;
    let before = screen.scope_evaluations(row)?;
    screen.click(screen.get_by_role(Role::Button, "Change")?)?;
    screen.expect_text(text, "New a")?;
    screen.get_by_text("New b")?;
    assert_eq!(screen.scope_evaluations(row)?, before + 1);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
