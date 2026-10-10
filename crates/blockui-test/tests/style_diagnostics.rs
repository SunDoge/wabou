use blockui::*;
use blockui_test::{Role, Screen};

struct InvalidContainer;
impl Actor for InvalidContainer {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default()
            .children(vec![])
            .key("diagnostic.column")
            .style(ViewStyle {
                font_size: 18.0,
                ..default()
            }))
    }
}
#[test]
fn unsupported_style_names_the_property_node_and_supported_widgets() {
    let error = Screen::render::<InvalidContainer>()
        .err()
        .expect("invalid container must reject");
    for fragment in [
        "invalid widget presentation",
        "Column",
        "font_size",
        "requires a label or Button",
        &format!("{:#018x}", abi::key("diagnostic.column")),
    ] {
        assert!(error.contains(fragment), "{error}");
    }
}
const COUNT: State<i64> = State::<i64>::new("diagnostic.count", 0);
const REJECT: Action = Action::new("diagnostic.reject", |cx, _| COUNT.set(cx, 1));
struct InvalidWidth;
impl Actor for InvalidWidth {
    const STATE: &'static [StateSpec] = &[COUNT.spec()];
    const ACTIONS: &'static [Action] = &[REJECT];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::new("Value")
                .key("diagnostic.label")
                .style_reactive(COUNT.map_style(|value| ViewStyle {
                    width: if value == 0 { 120.0 } else { f32::NAN },
                    ..default()
                })),
            Button::new("Invalid width", REJECT).into_view(),
        ]))
    }
}
#[test]
fn reactive_style_error_names_the_bad_value_and_preserves_committed_state()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<InvalidWidth>()?;
    let label = screen.get_by_text("Value")?;
    let before = screen.evaluations(label)?;
    let error = screen
        .click(screen.get_by_role(Role::Button, "Invalid width")?)
        .unwrap_err();
    for fragment in [
        "width",
        "finite",
        "NaN",
        "label",
        &format!("{:#018x}", abi::key("diagnostic.label")),
    ] {
        assert!(error.contains(fragment), "{error}");
    }
    assert_eq!(screen.style(label)?.width, 120.0);
    assert_eq!(screen.evaluations(label)?, before);
    assert_eq!(screen.actor().integer("diagnostic.count"), 0);
    assert_eq!(screen.root_evaluations(), 1);
    Ok(())
}
