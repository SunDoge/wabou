use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct MappedStyles {
    count: State<i64>,
}
#[app_impl]
impl MappedStyles {
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    #[action]
    fn add_two(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 2))
    }
    #[action]
    fn invalid_style(&self) -> Result<()> {
        self.count.set(100)
    }
    #[action]
    fn panic_style(&self) -> Result<()> {
        self.count.set(101)
    }
    #[action]
    fn error_style(&self) -> Result<()> {
        self.count.set(4)
    }
    #[action]
    fn error_computed_style(&self) -> Result<()> {
        self.count.set(6)
    }
    #[action]
    fn error_descriptor_style(&self) -> Result<()> {
        self.count.set(8)
    }
    #[computed(name = "mapped.size")]
    fn size(&self) -> i64 {
        self.count.get()
    }
    #[computed]
    fn even(&self) -> bool {
        self.count.get() % 2 == 0
    }
    #[blockui::view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new("State")
                .key("mapped.state")
                .style_reactive(self.count.map_style(|value| {
                    assert_ne!(value, 101, "reject mapped style");
                    ViewStyle {
                        font_size: if value == 100 {
                            f32::NAN
                        } else {
                            14.0 + value as f32
                        },
                        ..default()
                    }
                }));
            #[node]
            Text::new("Memo").key("mapped.memo").style_reactive(
                self.even_computed().try_map_style(|even| {
                    Ok(ViewStyle {
                        font_size: if even { 20.0 } else { 16.0 },
                        ..default()
                    })
                }),
            );
            #[node]
            Text::new("Fallible bound")
                .key("mapped.fallible")
                .style_reactive(self.count.try_map_style(|value| {
                    if value == 4 {
                        return Err(Error::InvalidView("mapped style rejected"));
                    }
                    Ok(ViewStyle {
                        width: 120.0 + value as f32,
                        ..default()
                    })
                }));
            #[node]
            Text::new("Descriptor")
                .key("mapped.descriptor")
                .style_reactive(State::from(&self.count).try_map_style(|value| {
                    if value == 8 {
                        return Err(Error::InvalidView("descriptor style rejected"));
                    }
                    Ok(ViewStyle {
                        width: 120.0 + value as f32,
                        ..default()
                    })
                }));
            #[node]
            Text::new("Fallible computed")
                .key("mapped.computed")
                .style_reactive(self.size_computed().try_map_style(|value| {
                    if value == 6 {
                        return Err(Error::InvalidView("computed style rejected"));
                    }
                    Ok(ViewStyle {
                        width: 120.0 + value as f32,
                        ..default()
                    })
                }));
            #[node]
            Button::new("Increment", self.increment_action());
            #[node]
            Button::new("Add two", self.add_two_action());
            #[node]
            Button::new("Invalid style", self.invalid_style_action());
            #[node]
            Button::new("Panic style", self.panic_style_action());
            #[node]
            Button::new("Error style", self.error_style_action());
            #[node]
            Button::new("Error computed", self.error_computed_style_action());
            #[node]
            Button::new("Error descriptor", self.error_descriptor_style_action());
        }
    }
}
#[test]
fn mapped_styles_keep_bound_contexts_ephemeral_and_update_independently()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<MappedStyles>()?;
    let state = screen.get_by_key("mapped.state")?;
    let memo = screen.get_by_key("mapped.memo")?;
    let state_before = screen.evaluations(state)?;
    let memo_before = screen.evaluations(memo)?;
    assert_eq!(screen.style(state)?.font_size, 14.0);
    assert_eq!(screen.style(memo)?.font_size, 20.0);
    assert_eq!(screen.evaluations(state)?, state_before);
    assert_eq!(screen.evaluations(memo)?, memo_before);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.root_evaluations(), 1);
    assert_eq!(screen.evaluations(state)?.text, state_before.text);
    assert_eq!(screen.evaluations(state)?.style, state_before.style + 1);
    assert_eq!(screen.evaluations(memo)?.text, memo_before.text);
    assert_eq!(screen.evaluations(memo)?.style, memo_before.style + 1);
    assert_eq!(screen.style(state)?.font_size, 15.0);
    assert_eq!(screen.style(memo)?.font_size, 16.0);
    screen.replace::<MappedStyles>()?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.root_evaluations(), 2);
    assert_eq!(screen.style(state)?.font_size, 16.0);
    assert_eq!(screen.style(memo)?.font_size, 20.0);
    Ok(())
}

#[test]
fn unchanged_memo_skips_style_mapping_and_failed_mapping_rolls_back()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<MappedStyles>()?;
    let state = screen.get_by_key("mapped.state")?;
    let memo = screen.get_by_key("mapped.memo")?;
    let before_memo = screen.evaluations(memo)?;
    screen.click(screen.get_by_role(Role::Button, "Add two")?)?;
    assert_eq!(screen.evaluations(memo)?, before_memo);
    let committed = screen.evaluations(state)?;
    let mapped = ["mapped.fallible", "mapped.descriptor", "mapped.computed"]
        .map(|key| screen.get_by_key(key).unwrap());
    assert_eq!(
        screen.actor().computed("mapped.size"),
        Some(&blockui_runtime::Value::Integer(2))
    );
    let memo_count = screen.actor().computed_evaluations("mapped.size");
    let snapshots = mapped.map(|locator| {
        (
            screen.style(locator).unwrap(),
            screen.evaluations(locator).unwrap(),
        )
    });
    for action in [
        "Invalid style",
        "Panic style",
        "Error style",
        "Error computed",
        "Error descriptor",
    ] {
        let error = screen
            .click(screen.get_by_role(Role::Button, action)?)
            .unwrap_err();
        let failed_key = match action {
            "Error style" => Some("mapped.fallible"),
            "Error computed" => Some("mapped.computed"),
            "Error descriptor" => Some("mapped.descriptor"),
            _ => None,
        };
        if let Some(key) = failed_key {
            assert!(
                error.contains(&format!(
                    "style binding {:#018x} evaluation failed",
                    Key::named(key).0
                )),
                "{error}"
            );
        }
        if action == "Error style" {
            assert!(error.contains("mapped style rejected"), "{error}");
        }
        if action == "Error computed" {
            assert!(error.contains("computed style rejected"), "{error}");
        }
        if action == "Error descriptor" {
            assert!(error.contains("descriptor style rejected"), "{error}");
        }
        for (locator, (style, evaluations)) in mapped.into_iter().zip(&snapshots) {
            assert_eq!(screen.style(locator)?, *style);
            assert_eq!(screen.evaluations(locator)?, *evaluations);
        }
        assert_eq!(
            screen.actor().computed("mapped.size"),
            Some(&blockui_runtime::Value::Integer(2))
        );
        assert_eq!(
            screen.actor().computed_evaluations("mapped.size"),
            memo_count
        );
        assert_eq!(screen.evaluations(state)?, committed);
        assert_eq!(screen.evaluations(memo)?, before_memo);
        assert_eq!(screen.root_evaluations(), 1);
        assert_eq!(screen.style(state)?.font_size, 16.0);
    }
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.style(state)?.font_size, 17.0);
    for locator in mapped {
        assert_eq!(screen.style(locator)?.width, 123.0);
    }
    assert_eq!(screen.root_evaluations(), 1);
    Ok(())
}
