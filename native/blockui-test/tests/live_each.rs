use blockui::*;
use blockui_test::{Role, Screen};
#[app]
struct List {
    order: State<Vec<String>>,
    a: State<String>,
    b: State<String>,
}
#[app_impl]
impl List {
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
            Text::new(value).key("value");
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
                .each(self.live_each(|this, key| this.row(&key)))
                .mount("rows");
        }
    }
}
#[test]
fn rows_update_independently_and_keep_identity_through_reorder() -> std::result::Result<(), String>
{
    let mut screen = Screen::render::<List>()?;
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
    let before = screen.scope_evaluations(rows)?;
    let b_evaluations = screen.evaluations(b)?;
    screen.input(a, "Edited A")?;
    assert_eq!(screen.scope_evaluations(rows)?, before);
    assert_eq!(screen.evaluations(b)?, b_evaluations);
    screen.click(screen.get_by_role(Role::Button, "Reverse")?)?;
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
    assert_eq!(screen.text(b)?, "Edited B");
    screen.click(screen.get_by_role(Role::Button, "Reset")?)?;
    assert_eq!(
        screen.text(screen.within(rows)?.get_by_role(Role::Textbox, "a")?)?,
        "Edited A"
    );
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<List>()?;
    assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "b")?, b);
    screen.input(b, "After reload")?;
    assert_eq!(screen.text(b)?, "After reload");
    Ok(())
}
