// Larger UI macro regression scenarios: no per-node markers.
mod typed_live {
    use blockui::*;
    use blockui_test::{Role, Screen};

    #[app]
    struct Unified {
        left_selected: State<bool>,
        left: State<i64>,
        right: State<i64>,
        caption: State<bool>,
    }
    #[app_impl]
    impl Unified {
        #[action]
        fn toggle(&self) -> Result<()> {
            self.left_selected.update(|value| Ok(!value))
        }
        #[action]
        fn increment_left(&self) -> Result<()> {
            self.left.update(|value| Ok(value + 1))
        }
        #[action]
        fn increment_right(&self) -> Result<()> {
            self.right.update(|value| Ok(value + 1))
        }
        #[action]
        fn toggle_caption(&self) -> Result<()> {
            self.caption.update(|value| Ok(!value))
        }
        // UI blocks preserve completion while an observer expression is being edited.
        fn view(&self) -> View {
            ui!(Column::default().children({
                Text::new(live!(if self.caption.get() {
                    "Changed child"
                } else {
                    "Stable child"
                }))
                .style(live!({
                    if self.left_selected.get() {
                        ViewStyle {
                            width: 100.0 + self.left.get() as f32,
                            ..default()
                        }
                    } else {
                        ViewStyle {
                            width: 100.0 + self.right.get() as f32,
                            ..default()
                        }
                    }
                }))
                .key("branch.text");
                Text::new("Static final")
                    .style(live!(ViewStyle {
                        width: self.left.get() as f32,
                        ..default()
                    }))
                    .style(ViewStyle {
                        width: 220.0,
                        ..default()
                    })
                    .key("static.final");
                Button::new("Caption", self.toggle_caption_action());
                Button::new("Toggle", self.toggle_action());
                Button::new("Left", self.increment_left_action());
                Button::new("Right", self.increment_right_action());
            }))
        }
    }

    #[test]
    fn typed_live_separates_text_and_style_observers() -> std::result::Result<(), String> {
        let mut screen = Screen::render::<Unified>()?;
        let text = screen.get_by_key("branch.text")?;
        let right_dependencies = screen.dependencies(text)?.style;
        let initial = screen.evaluations(text)?;
        let static_final = screen.get_by_key("static.final")?;
        let static_evaluations = screen.evaluations(static_final)?;
        assert!(screen.dependencies(static_final)?.style.is_empty());
        assert_eq!(screen.style(static_final)?.width, 220.0);
        screen.expect_text(text, "Stable child")?;
        screen.click(screen.get_by_role(Role::Button, "Left")?)?;
        assert_eq!(screen.evaluations(text)?, initial);
        screen.click(screen.get_by_role(Role::Button, "Right")?)?;
        assert_eq!(screen.style(text)?.width, 101.0);
        let before_toggle = screen.evaluations(text)?;
        screen.click(screen.get_by_role(Role::Button, "Toggle")?)?;
        assert_eq!(screen.style(text)?.width, 101.0);
        assert_eq!(screen.evaluations(text)?.style, before_toggle.style + 1);
        assert_ne!(screen.dependencies(text)?.style, right_dependencies);
        let left_only = screen.evaluations(text)?;
        screen.click(screen.get_by_role(Role::Button, "Right")?)?;
        assert_eq!(screen.evaluations(text)?, left_only);
        screen.click(screen.get_by_role(Role::Button, "Left")?)?;
        assert_eq!(screen.style(text)?.width, 102.0);
        assert_eq!(screen.evaluations(text)?.text, initial.text);
        let before_caption = screen.evaluations(text)?;
        screen.click(screen.get_by_role(Role::Button, "Caption")?)?;
        screen.expect_text(text, "Changed child")?;
        assert_eq!(screen.evaluations(text)?.style, before_caption.style);
        assert_eq!(screen.evaluations(text)?.text, before_caption.text + 1);
        assert_eq!(screen.style(static_final)?.width, 220.0);
        assert_eq!(screen.evaluations(static_final)?, static_evaluations);
        assert_eq!(screen.root_evaluations(), 1);
        screen.replace::<Unified>()?;
        assert_eq!(screen.style(text)?.width, 102.0);
        screen.expect_text(text, "Changed child")?;
        let replaced = screen.evaluations(text)?;
        screen.click(screen.get_by_role(Role::Button, "Right")?)?;
        assert_eq!(screen.evaluations(text)?, replaced);
        assert_eq!(screen.root_evaluations(), 2);
        Ok(())
    }
}
mod live_subtree {
    use blockui::*;
    use blockui_test::{Role, Screen};

    #[app]
    struct Dynamic {
        open: State<bool>,
        count: State<i64>,
        draft: State<String>,
    }
    #[app_impl]
    impl Dynamic {
        #[action]
        fn toggle(&self) -> Result<()> {
            self.open.update(|value| Ok(!value))
        }
        #[action]
        fn increment(&self) -> Result<()> {
            self.count.update(|value| Ok(value + 1))
        }

        fn details(&self) -> View {
            ui!(Column::default().children({
                Text::new(live!(self.count.get().to_string())).key("count");
                TextInput::new(&self.draft).key("draft");
                Button::new("Increment", self.increment_action());
            }))
        }
        fn view(&self) -> View {
            ui!(Column::default().children({
                Text::new("Outside").key("outside");
                Button::new("Toggle", self.toggle_action());
                Button::new("Outside increment", self.increment_action());
                live!(if self.open.get() {
                    self.details()
                } else {
                    View::empty()
                })
                .mount("details");
            }))
        }
    }

    #[test]
    fn live_subtree_mounts_locally_and_retains_actor_state() -> std::result::Result<(), String> {
        let mut screen = Screen::render::<Dynamic>()?;
        let outside = screen.get_by_key("outside")?;
        let outside_evaluations = screen.evaluations(outside)?;
        let toggle = screen.get_by_role(Role::Button, "Toggle")?;
        let scope_key = Key::named("details").0;
        assert!(screen.root_dependencies().is_empty());
        assert!(screen.get_by_role(Role::Textbox, "").is_err());
        let closed = screen.actor().projected_view().scope_dependencies[&scope_key].evaluations;
        screen.click(screen.get_by_role(Role::Button, "Outside increment")?)?;
        assert_eq!(
            screen.actor().projected_view().scope_dependencies[&scope_key].evaluations,
            closed
        );
        screen.click(toggle)?;
        let scope = screen.get_by_key("details")?;
        let opened = screen.actor().projected_view().scope_dependencies[&scope_key].evaluations;
        let content = screen.within(scope)?;
        let text = content.get_by_text("1")?;
        let input = content.get_by_role(Role::Textbox, "")?;
        let increment = content.get_by_role(Role::Button, "Increment")?;
        screen.input(input, "Retained draft")?;
        screen.click(increment)?;
        screen.expect_text(text, "2")?;
        assert_eq!(
            screen.actor().projected_view().scope_dependencies[&scope_key].evaluations,
            opened
        );
        screen.click(toggle)?;
        assert!(screen.click(increment).is_err());
        assert!(screen.input(input, "stale").is_err());
        let closed = screen.actor().projected_view().scope_dependencies[&scope_key].evaluations;
        screen.click(screen.get_by_role(Role::Button, "Outside increment")?)?;
        assert_eq!(
            screen.actor().projected_view().scope_dependencies[&scope_key].evaluations,
            closed
        );
        screen.click(toggle)?;
        let content = screen.within(scope)?;
        content.get_by_text("3")?;
        assert_eq!(
            screen.text(content.get_by_role(Role::Textbox, "Retained draft")?)?,
            "Retained draft"
        );
        assert_eq!(screen.evaluations(outside)?, outside_evaluations);
        assert_eq!(screen.root_evaluations(), 1);
        screen.replace::<Dynamic>()?;
        screen.within(scope)?.get_by_text("3")?;
        screen.click(
            screen
                .within(scope)?
                .get_by_role(Role::Button, "Increment")?,
        )?;
        screen.within(scope)?.get_by_text("4")?;
        assert_eq!(screen.root_evaluations(), 2);
        Ok(())
    }
}
mod live_each {
    use blockui::*;
    use blockui_test::{Role, Screen};
    #[app]
    struct List {
        order: State<Vec<String>>,
        a: State<String>,
        b: State<String>,
    }
    #[app_impl]
    impl List {
        #[computed]
        fn keys(&self) -> Vec<String> {
            self.order.get()
        }
        #[action]
        fn reset(&self) -> Result<()> {
            self.order.set(&["a".into(), "b".into()])
        }
        #[action]
        fn reverse(&self) -> Result<()> {
            self.order.update(|values| {
                values.reverse();
                Ok(())
            })
        }
        #[action]
        fn remove_a(&self) -> Result<()> {
            self.order.set(&["b".into()])
        }
        #[action]
        fn duplicate(&self) -> Result<()> {
            self.order.set(&["b".into(), "b".into()])
        }
        #[action]
        fn empty_key(&self) -> Result<()> {
            self.order.set(&[String::new()])
        }

        fn row(&self, key: &str) -> View {
            let value = if key == "a" {
                self.a.handle()
            } else {
                self.b.handle()
            };
            ui!(Column::default().children({
                Field::new(key, TextInput::new(value).key("input"));
                Text::new(value).key("value");
            }))
        }
        fn view(&self) -> View {
            ui!(Column::default().children({
                Button::new("Reset", self.reset_action());
                Button::new("Reverse", self.reverse_action());
                Button::new("Remove", self.remove_a_action());
                Button::new("Duplicate", self.duplicate_action());
                Button::new("Empty key", self.empty_key_action());
                self.keys_computed()
                    .each(self.live_each(|this, key| this.row(&key)))
                    .mount("rows");
            }))
        }
    }
    #[test]
    fn rows_update_independently_and_keep_identity_through_reorder()
    -> std::result::Result<(), String> {
        let mut screen = Screen::render::<List>()?;
        screen.click(screen.get_by_role(Role::Button, "Reset")?)?;
        let rows = screen.get_by_key("rows")?;
        let a = screen.within(rows)?.get_by_role(Role::Textbox, "a")?;
        let b = screen.within(rows)?.get_by_role(Role::Textbox, "b")?;
        let input_order = |screen: &Screen| {
            let view = screen.actor().projected_view();
            view.order
                .iter()
                .copied()
                .filter(|key| view.bindings[key].node.kind == abi::ENTRY)
                .collect::<Vec<_>>()
        };
        let original_order = input_order(&screen);
        assert_eq!(original_order.len(), 2);
        let before = screen.scope_evaluations(rows)?;
        let b_evaluations = screen.evaluations(b)?;
        screen.input(a, "Edited A")?;
        assert_eq!(screen.scope_evaluations(rows)?, before);
        assert_eq!(screen.evaluations(b)?, b_evaluations);
        screen.click(screen.get_by_role(Role::Button, "Reverse")?)?;
        assert_eq!(
            input_order(&screen),
            original_order.iter().rev().copied().collect::<Vec<_>>()
        );
        assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "a")?, a);
        assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "b")?, b);
        assert_eq!(screen.text(a)?, "Edited A");
        screen.input(b, "Edited B")?;
        screen.click(screen.get_by_role(Role::Button, "Remove")?)?;
        assert!(screen.input(a, "stale").is_err());
        assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "b")?, b);
        let stable = screen.scope_evaluations(rows)?;
        assert!(
            screen
                .click(screen.get_by_role(Role::Button, "Duplicate")?)
                .unwrap_err()
                .contains("nonempty and unique")
        );
        assert!(
            screen
                .click(screen.get_by_role(Role::Button, "Empty key")?)
                .unwrap_err()
                .contains("nonempty and unique")
        );
        assert_eq!(screen.scope_evaluations(rows)?, stable);
        assert_eq!(screen.text(b)?, "Edited B");
        screen.click(screen.get_by_role(Role::Button, "Reset")?)?;
        assert_eq!(
            screen.text(screen.within(rows)?.get_by_role(Role::Textbox, "a")?)?,
            "Edited A"
        );
        assert_eq!(screen.root_evaluations(), 1);
        screen.replace::<List>()?;
        assert_eq!(screen.within(rows)?.get_by_role(Role::Textbox, "b")?, b);
        screen.input(b, "After reload")?;
        assert_eq!(screen.text(b)?, "After reload");
        Ok(())
    }
}
mod style_patch {
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
            ui!(Column::default().children({
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
                Button::new("Toggle", self.toggle_action());
                Button::new("Increment", self.increment_action());
                Button::new("Poison", self.poison_action());
            }))
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
}
mod async_value {
    use blockui::*;
    use blockui_test::{Role, Screen};
    use std::time::Duration;
    #[derive(Record, Default, Clone, Debug, PartialEq)]
    struct Data {
        text: String,
    }
    fn async_fallback(error: AsyncReadError) -> Text {
        Text::new(match error {
            AsyncReadError::Idle => "Not started".to_owned(),
            AsyncReadError::Pending => "Boundary loading".to_owned(),
            AsyncReadError::Failed(message) => format!("Failed: {message}"),
        })
    }
    #[app]
    struct AsyncApp {
        value: State<AsyncValue<Data>>,
        job: Task,
        decoration: State<i64>,
    }
    #[app_impl]
    impl AsyncApp {
        #[computed(record)]
        fn derived(&self) -> Result<AsyncResult<Data>> {
            Ok(self.value.get()?.into_read())
        }
        #[computed(record)]
        fn forwarded(&self) -> Result<AsyncResult<Data>> {
            Ok(self.derived()?.map(|mut data| {
                let decoration = self.decoration.get();
                if decoration != 0 {
                    data.text = format!("{} / {decoration}", data.text);
                }
                data
            }))
        }
        #[action]
        fn decorate(&self) -> Result<()> {
            self.decoration.update(|value| Ok(value + 1))
        }
        #[action]
        fn fail(&self) -> Result<()> {
            self.job.cancel()?;
            self.value.update(|value| {
                value.reject(value.generation(), "offline".into());
                Ok(())
            })
        }
        #[action]
        fn cancel(&self) -> Result<()> {
            self.job.cancel()?;
            self.value.update(|value| {
                value.cancel(value.generation())?;
                Ok(())
            })
        }
        #[computed]
        fn pending(&self) -> Result<bool> {
            Ok(self.value.get()?.is_pending())
        }
        #[action]
        fn start(&self) -> Result<()> {
            let mut value = self.value.get()?;
            let token = value.begin()?;
            self.value.set(&value)?;
            self.job.cancel()?;
            self.job
                .delay(Duration::from_millis(10), self.complete_action(), token)
        }
        #[action]
        fn complete(&self, token: i64) -> Result<()> {
            self.value.update(|value| {
                value.resolve(
                    token,
                    Data {
                        text: format!("Result {token}"),
                    },
                );
                Ok(())
            })
        }
        fn view(&self) -> View {
            ui!(Column::default().children({
                Button::new("Start", self.start_action());
                Button::new("Fail", self.fail_action());
                Button::new("Cancel", self.cancel_action());
                Button::new("Decorate", self.decorate_action());
                live!(
                    self.forwarded()
                        .unwrap()
                        .map(|data| Text::new(format!("Boundary {}", data.text)))
                )
                .boundary(async_fallback)
                .mount("async.boundary");
                self.pending_computed()
                    .choose(live!(Text::new("Loading")), live!(Text::new("Idle")))
                    .mount("status");
                Text::new(live!({
                    let value = self.value.get().unwrap();
                    value
                        .latest()
                        .map(|data| data.text.clone())
                        .unwrap_or_default()
                }))
                .key("latest");
            }))
        }
    }
    #[test]
    fn strict_async_boundary_switches_locally_and_recovers_after_failure()
    -> std::result::Result<(), String> {
        let mut screen = Screen::with_manual_time::<AsyncApp>()?;
        let start = screen.get_by_role(Role::Button, "Start")?;
        screen.get_by_text("Not started")?;
        screen.click(start)?;
        screen.get_by_text("Boundary loading")?;
        screen.advance_time(Duration::from_millis(10))?;
        screen.get_by_text("Boundary Result 1")?;
        screen.click(start)?;
        screen.get_by_text("Boundary loading")?;
        screen.expect_text(screen.get_by_key("latest")?, "Result 1")?;
        screen.click(screen.get_by_role(Role::Button, "Fail")?)?;
        screen.get_by_text("Failed: offline")?;
        screen.advance_time(Duration::from_millis(10))?;
        screen.get_by_text("Failed: offline")?;
        screen.click(start)?;
        screen.replace::<AsyncApp>()?;
        screen.get_by_text("Boundary loading")?;
        screen.advance_time(Duration::from_millis(10))?;
        screen.get_by_text("Boundary Result 3")?;
        screen.click(start)?;
        screen.click(screen.get_by_role(Role::Button, "Cancel")?)?;
        screen.get_by_text("Not started")?;
        screen.advance_time(Duration::from_millis(10))?;
        screen.get_by_text("Not started")?;
        assert_eq!(screen.root_evaluations(), 2);
        Ok(())
    }
    #[test]
    fn host_timer_completion_publishes_owned_state_and_retains_stale_value()
    -> std::result::Result<(), String> {
        let mut screen = Screen::with_manual_time::<AsyncApp>()?;
        let start = screen.get_by_role(Role::Button, "Start")?;
        screen.click(start)?;
        screen.get_by_text("Loading")?;
        screen.advance_time(Duration::from_millis(10))?;
        screen.expect_text(screen.get_by_key("latest")?, "Result 1")?;
        screen.click(start)?;
        screen.expect_text(screen.get_by_key("latest")?, "Result 1")?;
        screen.get_by_text("Loading")?;
        screen.replace::<AsyncApp>()?;
        screen.advance_time(Duration::from_millis(10))?;
        screen.expect_text(screen.get_by_key("latest")?, "Result 2")?;
        assert_eq!(screen.root_evaluations(), 2);
        Ok(())
    }

    #[test]
    fn pending_results_skip_transforms_and_prune_ready_only_dependencies()
    -> std::result::Result<(), String> {
        let mut screen = Screen::with_manual_time::<AsyncApp>()?;
        let start = screen.get_by_role(Role::Button, "Start")?;
        let decorate = screen.get_by_role(Role::Button, "Decorate")?;
        let memo = AsyncApp::<DescriptorMode>::forwarded_computed();
        let initial = screen.record_computed_evaluations(memo)?;
        screen.click(decorate)?;
        assert_eq!(screen.record_computed_evaluations(memo)?, initial);
        screen.click(start)?;
        let pending = screen.record_computed_evaluations(memo)?;
        assert_eq!(pending, initial + 1);
        // A new request token changes durable state, but Pending remains equal.
        screen.click(start)?;
        assert_eq!(screen.record_computed_evaluations(memo)?, pending);
        screen.click(decorate)?;
        assert_eq!(screen.record_computed_evaluations(memo)?, pending);
        screen.advance_time(Duration::from_millis(10))?;
        screen.get_by_text("Boundary Result 2 / 2")?;
        let ready = screen.record_computed_evaluations(memo)?;
        screen.click(decorate)?;
        screen.get_by_text("Boundary Result 2 / 3")?;
        assert_eq!(screen.record_computed_evaluations(memo)?, ready + 1);
        screen.click(start)?;
        let refresh = screen.record_computed_evaluations(memo)?;
        screen.click(decorate)?;
        assert_eq!(screen.record_computed_evaluations(memo)?, refresh);
        screen.click(screen.get_by_role(Role::Button, "Fail")?)?;
        let failed = screen.record_computed_evaluations(memo)?;
        screen.click(decorate)?;
        assert_eq!(screen.record_computed_evaluations(memo)?, failed);
        screen.get_by_text("Failed: offline")?;
        assert_eq!(screen.root_evaluations(), 1);
        screen.replace::<AsyncApp>()?;
        screen.get_by_text("Failed: offline")?;
        Ok(())
    }
}
mod async_join {
    use blockui::*;
    use blockui_test::{Role, Screen};
    use std::time::Duration;

    #[derive(Record, Default)]
    struct Data {
        text: String,
    }
    #[app]
    struct Joined {
        left: State<AsyncValue<Data>>,
        right: State<AsyncValue<Data>>,
        left_job: Task,
        right_job: Task,
    }
    fn fallback(error: AsyncReadError) -> Text {
        Text::new(match error {
            AsyncReadError::Idle => "Idle".into(),
            AsyncReadError::Pending => "Pending".into(),
            AsyncReadError::Failed(message) => message,
        })
    }
    #[app_impl]
    impl Joined {
        #[computed(record)]
        fn combined(&self) -> Result<AsyncResult<Data>> {
            Ok(self
                .left
                .get()?
                .into_read()
                .join(self.right.get()?.into_read())
                .map(|(left, right)| Data {
                    text: format!("{} + {}", left.text, right.text),
                }))
        }
        #[action]
        fn start(&self) -> Result<()> {
            let mut left = self.left.get()?;
            let left_token = left.begin()?;
            self.left.set(&left)?;
            let mut right = self.right.get()?;
            let right_token = right.begin()?;
            self.right.set(&right)?;
            self.left_job.cancel()?;
            self.right_job.cancel()?;
            self.left_job.delay(
                Duration::from_millis(30),
                self.finish_left_action(),
                left_token,
            )?;
            self.right_job.delay(
                Duration::from_millis(10),
                self.finish_right_action(),
                right_token,
            )
        }
        #[action]
        fn finish_left(&self, token: i64) -> Result<()> {
            self.left.update(|value| {
                value.resolve(
                    token,
                    Data {
                        text: format!("left {token}"),
                    },
                );
                Ok(())
            })
        }
        #[action]
        fn finish_right(&self, token: i64) -> Result<()> {
            self.right.update(|value| {
                value.resolve(
                    token,
                    Data {
                        text: format!("right {token}"),
                    },
                );
                Ok(())
            })
        }
        #[action]
        fn fail_right(&self) -> Result<()> {
            self.right_job.cancel()?;
            self.right.update(|value| {
                value.reject(value.generation(), "Right failed".into());
                Ok(())
            })
        }
        fn view(&self) -> View {
            ui!(Column::default().children({
                live!(self.combined().unwrap().map(|data| Text::new(data.text)))
                    .boundary(fallback)
                    .mount("joined");
                Button::new("Start", self.start_action());
                Button::new("Fail right", self.fail_right_action());
            }))
        }
    }
    #[test]
    fn independent_inputs_are_observed_and_partial_completion_keeps_pending()
    -> std::result::Result<(), String> {
        let mut screen = Screen::with_manual_time::<Joined>()?;
        screen.click(screen.get_by_role(Role::Button, "Start")?)?;
        let memo = Joined::<DescriptorMode>::combined_computed();
        let pending = screen.record_computed_evaluations(memo)?;
        let scope = screen.get_by_key("joined")?;
        let scope_pending = screen.scope_evaluations(scope)?;
        screen.advance_time(Duration::from_millis(10))?;
        screen.get_by_text("Pending")?;
        // Right is observed even though left remains pending.
        assert_eq!(screen.record_computed_evaluations(memo)?, pending + 1);
        // Equal Pending output prevents rebuilding the boundary.
        assert_eq!(screen.scope_evaluations(scope)?, scope_pending);
        screen.replace::<Joined>()?;
        screen.advance_time(Duration::from_millis(20))?;
        screen.get_by_text("left 1 + right 1")?;
        assert_eq!(screen.root_evaluations(), 2);
        Ok(())
    }
    #[test]
    fn failure_in_other_input_is_visible_while_first_input_is_pending()
    -> std::result::Result<(), String> {
        let mut screen = Screen::with_manual_time::<Joined>()?;
        screen.click(screen.get_by_role(Role::Button, "Start")?)?;
        screen.click(screen.get_by_role(Role::Button, "Fail right")?)?;
        screen.get_by_text("Right failed")?;
        screen.advance_time(Duration::from_millis(30))?;
        screen.get_by_text("Right failed")?;
        screen.click(screen.get_by_role(Role::Button, "Start")?)?;
        screen.get_by_text("Pending")?;
        screen.advance_time(Duration::from_millis(30))?;
        screen.get_by_text("left 2 + right 2")?;
        assert_eq!(screen.root_evaluations(), 1);
        Ok(())
    }
}
mod component_isolation {
    use blockui::*;
    use blockui_test::{Role, Screen};
    #[component]
    struct TrailingToggle {
        count: State<bool>,
    }
    #[component_impl]
    impl TrailingToggle {
        #[computed]
        fn double(&self) -> bool {
            !self.count.get()
        }
        #[action]
        fn increment(&self) -> Result<()> {
            self.count.update(|value| Ok(!value))
        }
        fn view(&self) -> View {
            ui!(Column {
                gap: 8.px(),
                ..default()
            }
            .children({
                Text::new("An explicit parent");
                Row {
                    gap: 4.px(),
                    ..default()
                }
                .children({
                    Text::new(live!(if self.count.get() { "On" } else { "Off" }));
                    Text::new(live!(if self.double() { "inactive" } else { "active" }));
                });
                Button::new("Increment", self.increment_action());
            }))
        }
    }
    #[app]
    struct TrailingPair {
        left: TrailingToggle,
        right: TrailingToggle,
    }
    #[app_impl]
    impl TrailingPair {
        fn view(&self) -> View {
            ui!(Row::default().children({
                self.left.render();
                self.right.render();
            }))
        }
    }
    #[test]
    fn ui_blocks_keep_reactivity_and_component_isolation() -> std::result::Result<(), String> {
        let mut screen = Screen::render::<TrailingPair>()?;
        let counts = screen.query_all_by_text("Off");
        let doubles = screen.query_all_by_text("inactive");
        let buttons = screen.query_all_by_role(Role::Button, "Increment");
        assert_eq!(counts.len(), 2);
        assert_eq!(doubles.len(), 2);
        assert_eq!(buttons.len(), 2);
        assert_ne!(counts[0], counts[1]);
        assert_ne!(doubles[0], doubles[1]);
        assert_ne!(
            screen.dependencies(counts[0])?.text,
            screen.dependencies(counts[1])?.text
        );
        assert_ne!(
            screen.dependencies(doubles[0])?.text,
            screen.dependencies(doubles[1])?.text
        );
        let untouched_count = screen.evaluations(counts[1])?;
        let untouched_memo = screen.evaluations(doubles[1])?;
        screen.click(buttons[0])?;
        assert_eq!(screen.text(counts[0])?, "On");
        assert_eq!(screen.text(doubles[0])?, "active");
        assert_eq!(screen.evaluations(counts[1])?, untouched_count);
        assert_eq!(screen.evaluations(doubles[1])?, untouched_memo);
        assert_eq!(screen.root_evaluations(), 1);
        screen.replace::<TrailingPair>()?;
        assert_eq!(screen.get_by_text("On")?, counts[0]);
        assert_eq!(screen.get_by_text("active")?, doubles[0]);
        let left_count = screen.evaluations(counts[0])?;
        let left_memo = screen.evaluations(doubles[0])?;
        screen.click(buttons[1])?;
        assert_eq!(screen.text(counts[1])?, "On");
        assert_eq!(screen.text(doubles[1])?, "active");
        assert_eq!(screen.evaluations(counts[0])?, left_count);
        assert_eq!(screen.evaluations(doubles[0])?, left_memo);
        assert_eq!(screen.root_evaluations(), 2);
        Ok(())
    }
}

mod expression_composition {
    use blockui::*;
    use blockui_test::Screen;

    struct Panel(&'static str);
    impl IntoView for Panel {
        fn into_view(self) -> View {
            Column::default().children(vec![Text::new(self.0).into_view()])
        }
    }
    fn footer() -> View {
        ui!(Row::default().children({
            Text::new("Function footer");
        }))
    }
    fn label() -> Result<&'static str> {
        Ok("Fallible label")
    }
    #[app]
    struct Expressions {}
    #[app_impl]
    impl Expressions {
        fn view(&self) -> Result<View> {
            ui!(Panel("Custom root")
                .style(ViewStyle {
                    max_width: 320.0,
                    radius: 8.0,
                    ..default()
                })
                .key("custom.root")
                .children({
                    let make_text = |label| Text::new(label);
                    let local_panel = Row::default().children({
                        Text::new("Locally built panel");
                    });
                    local_panel;
                    make_text("Closure child");
                    Text::new(label()?);
                    Text::new({
                        let prefix = "Ordinary";
                        format!("{prefix} expression block")
                    });
                    Column::default().children(vec![Text::new("Explicit vector").into_view()]);
                    Row::default().children({
                        let label = "Nested child";
                        Text::new(label);
                        Text::new("Repeated label");
                        Text::new("Repeated label");
                    });
                    footer();
                }))
        }
    }
    #[test]
    fn arbitrary_components_fallible_values_locals_and_expression_blocks_compose()
    -> std::result::Result<(), String> {
        let screen = Screen::render::<Expressions>()?;
        let root = screen.get_by_key("custom.root")?;
        assert_eq!(screen.style(root)?.max_width, 320.0);
        assert_eq!(screen.style(root)?.radius, 8.0);
        for label in [
            "Custom root",
            "Closure child",
            "Locally built panel",
            "Fallible label",
            "Ordinary expression block",
            "Explicit vector",
            "Nested child",
            "Function footer",
        ] {
            screen.within(root)?.get_by_text(label)?;
        }
        let repeated = screen.query_all_by_text("Repeated label");
        assert_eq!(repeated.len(), 2);
        assert_ne!(repeated[0], repeated[1]);
        Ok(())
    }
}
