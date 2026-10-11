use blockui::*;
use blockui_test::{Dependency, Screen};
use std::collections::BTreeSet;

const LEFT: State<bool> = State::<bool>::new("deps.left", true);
const A: State<i64> = State::<i64>::new("deps.a", 1);
const B: State<i64> = State::<i64>::new("deps.b", 2);
const TONE: State<bool> = State::<bool>::new("deps.tone", false);

fn selected(cx: &Cx<'_>) -> Result<String> {
    Ok(if LEFT.get(cx)? {
        A.get(cx)?
    } else {
        B.get(cx)?
    }
    .to_string())
}

struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[LEFT.spec(), A.spec(), B.spec(), TONE.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::reactive(selected)
                .key("deps.label")
                .style_reactive(|cx| {
                    Ok(ViewStyle {
                        text_tone: if TONE.get(cx)? {
                            TextTone::Error
                        } else {
                            TextTone::Default
                        },
                        ..default()
                    })
                }),
            Button::label("Switch")
                .key("deps.switch")
                .on_click(|cx| LEFT.update(cx, |v| Ok(!v))),
            Button::label("Reject").key("deps.reject").on_click(|cx| {
                LEFT.set(cx, true)?;
                Err(Error::Application("reject".into()))
            }),
        ]))
    }
}

#[test]
fn snapshots_follow_helper_reads_and_branch_changes_without_running_observers()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<App>()?;
    let label = screen.get_by_key("deps.label")?;
    let counts = screen.evaluations(label)?;
    let initial = screen.dependencies(label)?;
    assert_eq!(
        initial.text,
        BTreeSet::from([Dependency::State(LEFT.key()), Dependency::State(A.key())])
    );
    assert_eq!(
        initial.style,
        BTreeSet::from([Dependency::State(TONE.key())])
    );
    assert_eq!(screen.evaluations(label)?, counts);
    screen.click(screen.get_by_key("deps.switch")?)?;
    let switched = screen.dependencies(label)?;
    assert_eq!(
        switched.text,
        BTreeSet::from([Dependency::State(LEFT.key()), Dependency::State(B.key())])
    );
    assert_eq!(switched.style, initial.style);
    assert!(screen.click(screen.get_by_key("deps.reject")?).is_err());
    assert_eq!(screen.dependencies(label)?, switched);
    screen.replace::<App>()?;
    assert_eq!(screen.dependencies(label)?, switched);
    Ok(())
}
