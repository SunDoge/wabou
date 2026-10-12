use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct App {
    count: State<i64>,
}
#[app_impl]
impl App {
    #[action(name = "app.bound.increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 1))
    }
    #[computed(name = "app.bound.double")]
    fn double(&self) -> i64 {
        self.count.get() * 2
    }
    #[scope(name = "app.bound.panel")]
    #[blockui::view]
    fn panel(&self, ui: &Ui) -> View {
        #[node]
        Text::new(self.double_computed());
    }
    #[blockui::view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            self.panel_scope();
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[component]
struct Counter {
    count: State<i64>,
}
#[component_impl]
impl Counter {
    #[action(name = "component.bound.increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 1))
    }
    #[computed(name = "component.bound.double")]
    fn double(&self) -> i64 {
        self.count.get() * 2
    }
    #[scope(name = "component.bound.panel")]
    #[blockui::view]
    fn panel(&self, ui: &Ui) -> View {
        #[node]
        Text::new(self.double_computed());
    }
    #[blockui::view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            self.panel_scope();
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
struct Host;
impl Actor for Host {
    const STATE: &'static [StateSpec] = Counter::SCHEMA;
    const ACTIONS: &'static [Action] = <Counter as Component>::ACTIONS;
    const COMPUTEDS: &'static [ComputedSpec] = <Counter as Component>::COMPUTEDS;
    const SCOPES: &'static [Scope] = <Counter as Component>::SCOPES;
    fn view(cx: &Cx<'_>) -> Result<View> {
        render_bound(&Counter::new(), cx)
    }
}
fn exercise<A: Actor>() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<A>()?;
    let value = screen.get_by_text("0")?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.get_by_text("2")?, value);
    screen.replace::<A>()?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.get_by_text("4")?, value);
    Ok(())
}
#[test]
fn bound_inference_ignores_words_inside_stable_names() -> std::result::Result<(), String> {
    exercise::<App>()?;
    exercise::<Host>()
}

mod forwarded_context {
    use super::*;
    macro_rules! declare {
        ($context:ty) => {
            #[app]
            struct Forwarded {
                count: State<i64>,
            }
            #[app_impl]
            impl Forwarded {
                #[action]
                fn increment(&self, cx: $context) -> Result<()> {
                    self.count.update(cx, |n| Ok(n + 1))
                }
                #[computed]
                fn double(&self, cx: $context) -> i64 {
                    self.count.get(cx).unwrap() * 2
                }
                #[scope]
                #[blockui::view]
                fn panel(&self, cx: $context, ui: &Ui) -> Result<View> {
                    let _: i64 = self.double(cx);
                    #[node]
                    {
                        Column::default();
                        #[node]
                        Text::new(Self::double_computed());
                        #[node]
                        Button::new("Increment", Self::increment_action());
                    }
                }
                fn view(&self) -> View {
                    self.panel_scope().into_view()
                }
            }
        };
    }
    declare!(&Cx<'_>);
    #[test]
    fn forwarded_context_type_preserves_explicit_context_methods() -> std::result::Result<(), String>
    {
        exercise::<Forwarded>()
    }
}
