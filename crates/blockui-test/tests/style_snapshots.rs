use blockui::*;
use blockui_test::Screen;

const STYLE: ViewStyle = ViewStyle {
    width: 640.0,
    max_width: 800.0,
    font_size: 0.0,
    text_tone: TextTone::Default,
    surface: Surface::Panel,
    radius: 8.0,
    theme: ThemeMode::Dark,
    density: Density::Compact,
    fill_height: true,
};
struct StyledApp;
impl Actor for StyledApp {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default()
            .children(vec![Text::new("Child").style(ViewStyle {
                font_size: 18.0,
                text_tone: TextTone::Error,
                ..default()
            })])
            .key("styled.root")
            .style(STYLE))
    }
}
struct Empty;
impl Actor for Empty {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(View::empty())
    }
}
#[test]
fn style_snapshots_are_owned_typed_and_keep_inheritance_unresolved()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<StyledApp>()?;
    let root = screen.get_by_key("styled.root")?;
    let child = screen.get_by_text("Child")?;
    let saved = screen.style(root)?;
    assert_eq!(saved, STYLE);
    assert_eq!(
        screen.style(child)?,
        ViewStyle {
            font_size: 18.0,
            text_tone: TextTone::Error,
            ..default()
        }
    );
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Empty>()?;
    for locator in [root, child] {
        let error = screen.style(locator).unwrap_err();
        assert!(
            error.contains("element is no longer mounted (key 0x"),
            "{error}"
        );
    }
    assert_eq!(
        screen.root_evaluations(),
        2,
        "diagnostics do not evaluate UI"
    );
    drop(screen);
    assert_eq!(saved, STYLE);
    Ok(())
}
