use blockui::*;
use blockui_test::{EvaluationCounts, Role, Screen};
const NAME: State<String> = State::<String>::with_default("counts.name", "Ada");
const OTHER: State<String> = State::<String>::with_default("counts.other", "Grace");
const ALTERNATE: State<bool> = State::<bool>::new("counts.alternate", false);
const LARGE: State<bool> = State::<bool>::new("counts.large", false);
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] =
        &[NAME.spec(), OTHER.spec(), ALTERNATE.spec(), LARGE.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::reactive(|cx| {
                let source = if ALTERNATE.get(cx)? { OTHER } else { NAME };
                source.get(cx)
            })
            .key("counts.label")
            .style_reactive(|cx| {
                Ok(ViewStyle {
                    font_size: if LARGE.get(cx)? { 24.0 } else { 14.0 },
                    ..default()
                })
            }),
            Text::new("Unrelated").key("counts.static").into_view(),
            Field::new("Name", TextInput::new(NAME)).into_view(),
            Field::new("Other", TextInput::new(OTHER)).into_view(),
            Button::label("Alternate")
                .key("counts.alternate")
                .on_click(|cx| ALTERNATE.set(cx, true)),
            Button::label("Large")
                .key("counts.large")
                .on_click(|cx| LARGE.set(cx, true)),
            Button::label("Reject").key("counts.reject").on_click(|cx| {
                NAME.set(cx, "rejected")?;
                Err(Error::Application("reject update".into()))
            }),
        ]))
    }
}
#[test]
fn snapshots_verify_independent_observers_and_conditional_dependencies()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<App>()?;
    let label = screen.get_by_key("counts.label")?;
    let unrelated = screen.get_by_text("Unrelated")?;
    let initial = screen.evaluations(label)?;
    let untouched = screen.evaluations(unrelated)?;
    let name = screen.get_by_role(Role::Textbox, "Name")?;
    let other = screen.get_by_role(Role::Textbox, "Other")?;
    screen.input(other, "Katherine")?;
    assert_eq!(screen.evaluations(label)?, initial);
    screen.input(name, "Augusta")?;
    assert_eq!(screen.text(label)?, "Augusta");
    assert_eq!(
        screen.evaluations(label)?,
        EvaluationCounts {
            text: initial.text + 1,
            ..initial
        }
    );
    let before_style = screen.evaluations(label)?;
    screen.click(screen.get_by_role(Role::Button, "Large")?)?;
    assert_eq!(
        screen.evaluations(label)?,
        EvaluationCounts {
            style: before_style.style + 1,
            ..before_style
        }
    );
    screen.click(screen.get_by_role(Role::Button, "Alternate")?)?;
    assert_eq!(screen.text(label)?, "Katherine");
    let switched = screen.evaluations(label)?;
    screen.input(name, "ignored")?;
    assert_eq!(screen.evaluations(label)?, switched);
    screen.input(other, "Johnson")?;
    assert_eq!(screen.text(label)?, "Johnson");
    assert_eq!(
        screen.evaluations(label)?,
        EvaluationCounts {
            text: switched.text + 1,
            ..switched
        }
    );
    assert_eq!(screen.evaluations(unrelated)?, untouched);
    Ok(())
}
#[test]
fn rejected_update_does_not_publish_evaluation_counts() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<App>()?;
    let label = screen.get_by_key("counts.label")?;
    let before = screen.evaluations(label)?;
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Reject")?)
            .is_err()
    );
    assert_eq!(screen.text(label)?, "Ada");
    assert_eq!(screen.evaluations(label)?, before);
    Ok(())
}
