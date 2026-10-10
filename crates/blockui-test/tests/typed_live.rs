use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Unified {
    left_selected: State<bool>,
    left: State<i64>,
    right: State<i64>,
    caption: State<bool>,
}
#[app_impl]
impl Unified {
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
    #[action]
    fn toggle_caption(&self) -> Result<()> {
        self.caption.update(|value| Ok(!value))
    }
    // UI blocks preserve completion while an observer expression is being edited.
    fn view(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Text::new(live!(if self.caption.get() {
                "Changed child"
            } else {
                "Stable child"
            }))
            .style(live!({
                if self.left_selected.get() {
                    ViewStyle {
                        width: 100.0 + self.left.get() as f32,
                        ..default()
                    }
                } else {
                    ViewStyle {
                        width: 100.0 + self.right.get() as f32,
                        ..default()
                    }
                }
            }))
            .key("branch.text");
            #[ui]
            Text::new("Static final")
                .style(live!(ViewStyle {
                    width: self.left.get() as f32,
                    ..default()
                }))
                .style(ViewStyle {
                    width: 220.0,
                    ..default()
                })
                .key("static.final");
            #[ui]
            Button::new("Caption", self.toggle_caption_action());
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
fn typed_live_separates_text_and_style_observers() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Unified>()?;
    let text = screen.get_by_key("branch.text")?;
    let right_dependencies = screen.dependencies(text)?.style;
    let initial = screen.evaluations(text)?;
    let static_final = screen.get_by_key("static.final")?;
    let static_evaluations = screen.evaluations(static_final)?;
    assert!(screen.dependencies(static_final)?.style.is_empty());
    assert_eq!(screen.style(static_final)?.width, 220.0);
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
    let before_caption = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Caption")?)?;
    screen.expect_text(text, "Changed child")?;
    assert_eq!(screen.evaluations(text)?.style, before_caption.style);
    assert_eq!(screen.evaluations(text)?.text, before_caption.text + 1);
    assert_eq!(screen.style(static_final)?.width, 220.0);
    assert_eq!(screen.evaluations(static_final)?, static_evaluations);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Unified>()?;
    assert_eq!(screen.style(text)?.width, 102.0);
    screen.expect_text(text, "Changed child")?;
    let replaced = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Right")?)?;
    assert_eq!(screen.evaluations(text)?, replaced);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
