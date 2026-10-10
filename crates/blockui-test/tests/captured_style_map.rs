use blockui::*;
use blockui_test::{Role, Screen};

#[derive(Clone)]
struct Theme {
    font: f32,
    name: String,
}
impl Theme {
    fn style(&self, value: i64) -> ViewStyle {
        ViewStyle {
            font_size: self.font + value as f32,
            width: 100.0 + self.name.len() as f32,
            ..default()
        }
    }
}
#[app]
struct CapturedStyles {
    count: State<i64>,
    large: State<bool>,
}
#[app_impl]
impl CapturedStyles {
    #[action]
    fn advance(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 2))
    }
    #[action]
    fn reject(&self) -> Result<()> {
        self.count.set(1)
    }
    #[action]
    fn retheme(&self) -> Result<()> {
        self.large.set(true)
    }
    #[computed(name = "captured.style.even")]
    fn even(&self) -> bool {
        self.count.get() % 2 == 0
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        let large = self.large.get();
        let theme = Theme {
            font: if large { 24.0 } else { 14.0 },
            name: String::from(if large { "large theme" } else { "small" }),
        };
        let state_theme = theme.clone();
        let memo_theme = theme.clone();
        #[node]
        {
            Column::default();
            #[node]
            Text::new("Count")
                .key("count")
                .style_reactive(self.count.map_style_with(move |value| theme.style(value)));
            #[node]
            Text::new("Checked").key("checked").style_reactive(
                State::from(&self.count).try_map_style_with(move |value| {
                    if value == 1 {
                        return Err(Error::InvalidView("captured theme rejected"));
                    }
                    Ok(state_theme.style(value))
                }),
            );
            #[node]
            Text::new("Even").key("memo").style_reactive(
                self.even_computed()
                    .map_style_with(move |even| memo_theme.style(if even { 10 } else { 0 })),
            );
            #[node]
            Button::new("Advance", self.advance_action());
            #[node]
            Button::new("Reject", self.reject_action());
            #[node]
            Button::new("Theme", self.retheme_action());
        }
    }
}
#[test]
fn owned_theme_parameters_refresh_without_invalidating_equal_memos()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<CapturedStyles>()?;
    let count = screen.get_by_key("count")?;
    let checked = screen.get_by_key("checked")?;
    let memo = screen.get_by_key("memo")?;
    let counts = screen.evaluations(count)?;
    let memo_counts = screen.evaluations(memo)?;
    let error = screen
        .click(screen.get_by_role(Role::Button, "Reject")?)
        .unwrap_err();
    assert!(error.contains("captured theme rejected"), "{error}");
    assert_eq!(screen.style(count)?.font_size, 14.0);
    assert_eq!(screen.style(checked)?.font_size, 14.0);
    assert_eq!(screen.evaluations(count)?, counts);
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.style(count)?.font_size, 16.0);
    assert_eq!(screen.evaluations(count)?.text, counts.text);
    assert_eq!(screen.evaluations(memo)?, memo_counts);
    assert_eq!(screen.root_evaluations(), 1);
    let memo_evaluations = screen.actor().computed_evaluations("captured.style.even");
    screen.click(screen.get_by_role(Role::Button, "Theme")?)?;
    assert_eq!(screen.style(count)?.font_size, 26.0);
    assert_eq!(screen.style(count)?.width, 111.0);
    assert_eq!(screen.style(checked)?.font_size, 26.0);
    assert_eq!(screen.style(memo)?.font_size, 34.0);
    assert_eq!(
        screen.actor().computed_evaluations("captured.style.even"),
        memo_evaluations
    );
    assert_eq!(screen.root_evaluations(), 2);
    screen.replace::<CapturedStyles>()?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.style(count)?.font_size, 28.0);
    assert_eq!(screen.style(memo)?.font_size, 34.0);
    assert_eq!(screen.root_evaluations(), 3);
    Ok(())
}
