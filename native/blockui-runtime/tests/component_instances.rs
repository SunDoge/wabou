#![cfg(feature = "testing")]
use blockui::*;

#[component]
struct Counter {
    count: State<i64>,
    draft: State<String>,
}
#[component_impl]
impl Counter {
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 1))
    }
    #[computed]
    fn double(&self) -> Result<i64> {
        Ok(self.count.get() * 2)
    }
    #[scope]
    fn detail(&self) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(self.double_computed()));
            }
            __blockui_tree.finish()
        }
    }
    fn view(&self) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Counter"));
                __blockui_tree.add(Text::new(&self.count));
                __blockui_tree.add(TextInput::new(&self.draft));
                __blockui_tree.add(Button::new("Increment", self.events().increment));
                __blockui_tree.add(self.detail_scope());
            }
            __blockui_tree.finish()
        }
    }
}
#[component]
struct Pair {
    left: Counter,
    right: Counter,
}
#[component_impl]
impl Pair {
    fn view(&self) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Row::default());
                __blockui_tree.add(self.left.render());
                __blockui_tree.add(self.right.render());
            }
            __blockui_tree.finish()
        }
    }
}
#[app]
struct App {
    pair: Pair,
}
#[app_impl]
impl App {
    fn view(&self) -> View {
        self.pair.render()
    }
}

#[test]
fn repeated_nested_components_have_independent_state_actions_and_computations()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    let app = App::new();
    let left = app.pair.left.count.key();
    let right = app.pair.right.count.key();
    assert_ne!(left, right);
    let left_increment = app.pair.left.events().increment;
    let right_increment = app.pair.right.events().increment;
    actor.tell(left_increment, 0)?;
    actor.tick()?;
    assert_eq!(actor.integer_by_key(left), 1);
    assert_eq!(actor.integer_by_key(right), 0);
    let state = actor.state_address();
    let slot = actor.slot_address_by_key(left);
    actor.tell(right_increment, 0)?;
    actor.replace_for_test::<App>()?;
    actor.tick()?;
    assert_eq!(actor.integer_by_key(left), 1);
    assert_eq!(actor.integer_by_key(right), 1);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address_by_key(left), slot);
    assert_eq!(actor.computed_count(), 2);
    assert_eq!(actor.scope_ids().len(), 2);
    Ok(())
}
