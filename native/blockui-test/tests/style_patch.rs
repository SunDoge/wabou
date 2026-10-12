use blockui::*;
use blockui_test::{Role, Screen};
#[app]
struct Styles {
    enabled: State<bool>,
    amount: State<i64>,
}
#[app_impl]
impl Styles {
    #[action]
    fn toggle(&self) -> Result<()> {
        self.enabled.update(|value| Ok(!value))
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.amount.update(|value| Ok(value + 1))
    }
    #[action]
    fn poison(&self) -> Result<()> {
        self.amount.set(-200)
    }
    fn view(&self) -> View {
        let base = ViewStyle {
            width: 200.0,
            max_width: 400.0,
            font_size: 18.0,
            radius: 0.0,
            surface: Surface::Default,
            text_tone: TextTone::Accent,
            theme: ThemeMode::Dark,
            density: Density::Compact,
            fill_height: false,
        };
        #[ui(Column::default())]
        {
            #[ui]
            Text::new("Composed").key("composed").style((
                base,
                live!(if self.enabled.get() {
                    StylePatch {
                        width: Some(100.0 + self.amount.get() as f32),
                        ..default()
                    }
                } else {
                    StylePatch::default()
                }),
                StylePatch {
                    radius: Some(0.0),
                    max_width: Some(500.0),
                    fill_height: Some(false),
                    theme: Some(ThemeMode::Inherit),
                    ..default()
                },
            ));
            #[ui]
            Column::default()
                .key("static")
                .style(ViewStyle {
                    radius: 8.0,
                    surface: Surface::Panel,
                    fill_height: true,
                    font_size: 0.0,
                    text_tone: TextTone::Default,
                    ..base
                })
                .style(StylePatch {
                    width: Some(0.0),
                    fill_height: Some(false),
                    ..default()
                });
            #[ui]
            Button::new("Toggle", self.toggle_action());
            #[ui]
            Button::new("Increment", self.increment_action());
            #[ui]
            Button::new("Poison", self.poison_action());
        }
    }
}
#[test]
fn ordered_patches_preserve_unspecified_fields_and_track_selected_reads()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Styles>()?;
    let composed = screen.get_by_key("composed")?;
    let static_style = screen.style(screen.get_by_key("static")?)?;
    assert_eq!(static_style.width, 0.0);
    assert_eq!(static_style.radius, 8.0);
    assert!(!static_style.fill_height);
    assert_eq!(static_style.surface, Surface::Panel);
    let initial = screen.style(composed)?;
    assert_eq!(initial.width, 200.0);
    assert_eq!(initial.max_width, 500.0);
    assert_eq!(initial.radius, 0.0);
    assert!(!initial.fill_height);
    assert_eq!(initial.theme, ThemeMode::Inherit);
    assert_eq!(initial.density, Density::Compact);
    assert_eq!(initial.surface, Surface::Default);
    assert_eq!(initial.text_tone, TextTone::Accent);
    assert_eq!(initial.font_size, 18.0);
    let before = screen.evaluations(composed)?;
    let inactive_dependencies = screen.dependencies(composed)?.style;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.evaluations(composed)?, before);
    let toggle = screen.get_by_role(Role::Button, "Toggle")?;
    screen.click(toggle)?;
    assert_eq!(screen.style(composed)?.width, 101.0);
    let active = screen.evaluations(composed)?;
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Poison")?)
            .is_err()
    );
    assert_eq!(screen.style(composed)?.width, 101.0);
    assert_eq!(screen.evaluations(composed)?, active);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.style(composed)?.width, 102.0);
    screen.click(toggle)?;
    assert_eq!(screen.style(composed)?, initial);
    assert_eq!(screen.dependencies(composed)?.style, inactive_dependencies);
    let closed = screen.evaluations(composed)?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.evaluations(composed)?, closed);
    assert_eq!(closed.text, before.text);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Styles>()?;
    screen.click(toggle)?;
    assert_eq!(screen.style(composed)?.width, 103.0);
    assert_eq!(screen.style(composed)?.text_tone, TextTone::Accent);
    Ok(())
}
