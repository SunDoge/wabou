use wabou_native::*;
#[app(name = "counter")]
pub struct Counter {
    count: State<i64>,
}
#[app_impl]
impl Counter {
    #[action]
    fn increment(&self) -> Result<()> {
        self.count
            .update(|value| value.checked_add(1).ok_or(Error::Overflow))
    }

    fn view(&self) -> View {
        #[ui]
        {
            Card {
                key: "counter.root".into(),
                ..default()
            };
            #[ui]
            Heading::new("Hello, Wabou").key("counter.heading");
            #[ui]
            Text::new(live!(format!("Count: {}", self.count.get()))).key("counter.value");
            #[ui]
            Button::new("Increment", self.increment_action()).key("counter.increment");
        }
    }
}
export_actor!(Counter);

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_native::testing::{Role, Screen};
    #[test]
    fn increment_updates_the_counter_without_rendering() -> std::result::Result<(), String> {
        let mut screen = Screen::render::<Counter>()?;
        assert_eq!(screen.root_evaluations(), 1, "{}", screen.debug_tree());
        let heading = screen.get_by_role(Role::Heading, "Hello, Wabou")?;
        let before = screen.evaluations(heading)?;
        let value = screen.get_by_text("Count: 0")?;
        let value_before = screen.evaluations(value)?;
        screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
        screen.expect_text(value, "Count: 1")?;
        assert_eq!(screen.root_evaluations(), 1, "{}", screen.debug_tree());
        assert_eq!(
            screen.evaluations(heading)?,
            before,
            "{}",
            screen.debug_tree()
        );
        assert_eq!(
            screen.evaluations(value)?.text,
            value_before.text + 1,
            "{}",
            screen.debug_tree()
        );
        // Exercise implementation replacement without opening a native window.
        // This uses a compiled-in implementation; real dylib reload is tested separately.
        screen.replace::<Counter>()?;
        screen.expect_text(value, "Count: 1")?;
        assert_eq!(screen.root_evaluations(), 2, "{}", screen.debug_tree());
        let value_before = screen.evaluations(value)?;
        let heading_before = screen.evaluations(heading)?;
        screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
        screen.expect_text(value, "Count: 2")?;
        assert_eq!(screen.root_evaluations(), 2, "{}", screen.debug_tree());
        assert_eq!(
            screen.evaluations(heading)?,
            heading_before,
            "{}",
            screen.debug_tree()
        );
        assert_eq!(
            screen.evaluations(value)?.text,
            value_before.text + 1,
            "{}",
            screen.debug_tree()
        );
        Ok(())
    }
}
