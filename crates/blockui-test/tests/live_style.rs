use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Branch {
    left_selected: State<bool>,
    left: State<i64>,
    right: State<i64>,
}
#[app_impl]
impl Branch {
    #[action]
    fn toggle(&self) -> Result<()> {
        self.left_selected.update(|value| Ok(!value))
    }
    #[action]
    fn increment_left(&self) -> Result<()> {
        self.left.update(|value| Ok(value + 1))
    }
    #[action]
    fn increment_right(&self) -> Result<()> {
        self.right.update(|value| Ok(value + 1))
    }
    // UI blocks preserve completion while an observer expression is being edited.
    fn view(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Text::new("Stable child")
                .style_reactive(self.live_style(|this| {
                    if this.left_selected.get() {
                        ViewStyle {
                            width: 100.0 + this.left.get() as f32,
                            ..default()
                        }
                    } else {
                        ViewStyle {
                            width: 100.0 + this.right.get() as f32,
                            ..default()
                        }
                    }
                }))
                .key("branch.text");
            #[ui]
            Button::new("Toggle", self.toggle_action());
            #[ui]
            Button::new("Left", self.increment_left_action());
            #[ui]
            Button::new("Right", self.increment_right_action());
        }
    }
}

#[test]
fn style_function_tracks_only_the_active_branch() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Branch>()?;
    let text = screen.get_by_key("branch.text")?;
    let right_dependencies = screen.dependencies(text)?.style;
    let initial = screen.evaluations(text)?;
    screen.expect_text(text, "Stable child")?;
    screen.click(screen.get_by_role(Role::Button, "Left")?)?;
    assert_eq!(screen.evaluations(text)?, initial);
    screen.click(screen.get_by_role(Role::Button, "Right")?)?;
    assert_eq!(screen.style(text)?.width, 101.0);
    let before_toggle = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Toggle")?)?;
    assert_eq!(screen.style(text)?.width, 101.0);
    assert_eq!(screen.evaluations(text)?.style, before_toggle.style + 1);
    assert_ne!(screen.dependencies(text)?.style, right_dependencies);
    let left_only = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Right")?)?;
    assert_eq!(screen.evaluations(text)?, left_only);
    screen.click(screen.get_by_role(Role::Button, "Left")?)?;
    assert_eq!(screen.style(text)?.width, 102.0);
    assert_eq!(screen.evaluations(text)?.text, initial.text);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Branch>()?;
    assert_eq!(screen.style(text)?.width, 102.0);
    let replaced = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Right")?)?;
    assert_eq!(screen.evaluations(text)?, replaced);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
