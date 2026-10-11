use blockui::*;
use blockui_test::{Role, Screen};

fn footer() -> View {
    rsx! { <Row><Text text="Function footer" /></Row> }
}

#[derive(Default)]
struct CustomPanel {
    title: String,
}
impl IntoView for CustomPanel {
    fn into_view(self) -> View {
        Column::default().children(vec![Text::new(self.title).into_view()])
    }
}

#[app]
struct RsxDemo {
    count: State<i64>,
    open: State<bool>,
}
#[app_impl]
impl RsxDemo {
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| Ok(n + 1))
    }
    #[action]
    fn toggle(&self) -> Result<()> {
        self.open.update(|open| Ok(!open))
    }
    fn details(&self) -> View {
        rsx! {
            <CustomPanel title="Custom component" style={ViewStyle { max_width: 320.0, ..default() }}>
                <Text text={live!(self.count.get().to_string())} key="count" />
                <Button text="Increment" on_click={self.increment_action()} />
            </CustomPanel>
        }
    }
    fn view(&self) -> View {
        rsx! {
            <Column gap={8.0_f32} padding={12.0_f32}>
                <Text text="Outside" key="outside" />
                <Button text="Toggle" on_click={self.toggle_action()} />
                <Text text={live!(self.count.get().to_string())}
                    style={live!(ViewStyle { width: 100.0 + self.count.get() as f32, ..default() })}
                    key="summary" />
                {live!(if self.open.get() { self.details() } else { View::empty() }).mount("details")}
                {footer()}
            </Column>
        }
    }
}

#[test]
fn rsx_composes_custom_components_and_updates_locally() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<RsxDemo>()?;
    screen.get_by_text("Function footer")?;
    let outside = screen.get_by_key("outside")?;
    let initial = screen.evaluations(outside)?;
    let summary = screen.get_by_key("summary")?;
    let toggle = screen.get_by_role(Role::Button, "Toggle")?;
    screen.click(toggle).expect("toggle remains mounted");
    screen.get_by_text("Custom component")?;
    let details = screen.get_by_key("details")?;
    let count = screen.within(details)?.get_by_text("0")?;
    let increment = screen.get_by_role(Role::Button, "Increment")?;
    screen.click(increment).expect("increment remains mounted");
    screen
        .expect_text(count, "1")
        .expect("count remains mounted after increment");
    screen
        .expect_text(summary, "1")
        .expect("summary remains mounted after increment");
    assert_eq!(screen.style(summary)?.width, 101.0);
    assert_eq!(screen.evaluations(outside)?, initial);
    assert_eq!(screen.root_evaluations(), 1);
    screen.click(toggle).expect("toggle remains mounted");
    assert!(screen.click(increment).is_err());
    screen.click(toggle).expect("toggle remains mounted");
    screen.within(details)?.get_by_text("1")?;
    screen.replace::<RsxDemo>()?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen
        .expect_text(summary, "2")
        .expect("summary remains mounted after replacement");
    Ok(())
}
