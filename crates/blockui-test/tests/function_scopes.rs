use blockui::*;
use blockui_test::{Role, Screen};
#[component]
fn Counter(count: State<i64>) -> View {
    Scope::reactive("panel", move |_| {
        Ok(Column::default().children(vec![
            Text::reactive(move |cx| Ok(format!("Count: {}", count.get(cx)?)))
                .key("count")
                .into_view(),
            Button::label("Increment")
                .key("increment")
                .on_click(move |cx| count.update(cx, |n| Ok(n + 1)))
                .into_view(),
        ]))
    })
}
#[app]
fn Pair(left: Counter, right: Counter) -> View {
    Row::default().children(vec![left.view(), right.view()])
}
#[test]
fn repeated_function_components_need_no_manual_instance_or_scope_keys()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Pair>()?;
    let buttons = screen.query_all_by_role(Role::Button, "Increment");
    assert_eq!(buttons.len(), 2);
    let left = screen.scope_of(buttons[0])?;
    let right = screen.scope_of(buttons[1])?;
    assert_ne!(left, right);
    assert_eq!(
        screen
            .within(left)?
            .get_by_role(Role::Button, "Increment")?,
        buttons[0]
    );
    screen.click(buttons[0])?;
    screen.within(left)?.get_by_text("Count: 1")?;
    screen.within(right)?.get_by_text("Count: 0")?;
    screen.replace::<Pair>()?;
    screen.click(buttons[1])?;
    screen.within(left)?.get_by_text("Count: 1")?;
    screen.within(right)?.get_by_text("Count: 1")?;
    Ok(())
}

#[component]
fn LeafButton(count: State<i64>) -> View {
    Scope::reactive("button", move |cx| {
        Ok(Button::label(format!("Clicks: {}", count.get(cx)?))
            .key("increment")
            .on_click(move |cx| count.update(cx, |n| Ok(n + 1)))
            .into_view())
    })
}
#[app]
fn LeafPair(left: LeafButton, right: LeafButton) -> View {
    Row::default().children(vec![left.view(), right.view()])
}
#[test]
fn captured_leaf_callbacks_remain_isolated_after_refresh_and_replacement()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<LeafPair>()?;
    let buttons = screen.query_all_by_role(Role::Button, "Clicks: 0");
    assert_eq!(buttons.len(), 2);
    let left = screen.scope_of(buttons[0])?;
    let right = screen.scope_of(buttons[1])?;
    assert_ne!(left, right);
    let right_before = screen.scope_evaluations(right)?;
    screen.click(buttons[0])?;
    assert_eq!(
        screen
            .within(left)?
            .get_by_role(Role::Button, "Clicks: 1")?,
        buttons[0]
    );
    assert_eq!(screen.scope_evaluations(right)?, right_before);
    screen.replace::<LeafPair>()?;
    screen.click(buttons[1])?;
    screen.click(buttons[0])?;
    assert_eq!(
        screen
            .within(left)?
            .get_by_role(Role::Button, "Clicks: 2")?,
        buttons[0]
    );
    assert_eq!(
        screen
            .within(right)?
            .get_by_role(Role::Button, "Clicks: 1")?,
        buttons[1]
    );
    Ok(())
}
