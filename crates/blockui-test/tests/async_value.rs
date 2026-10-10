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
        #[ui(Column::default())]
        {
            #[ui]
            Button::new("Start", self.start_action());
            #[ui]
            Button::new("Fail", self.fail_action());
            #[ui]
            Button::new("Cancel", self.cancel_action());
            #[ui]
            Button::new("Decorate", self.decorate_action());
            #[ui]
            live!(
                self.forwarded()
                    .unwrap()
                    .map(|data| Text::new(format!("Boundary {}", data.text)))
            )
            .boundary(async_fallback)
            .mount("async.boundary");
            #[ui]
            self.pending_computed()
                .choose(live!(Text::new("Loading")), live!(Text::new("Idle")))
                .mount("status");
            #[ui]
            Text::new(live!({
                let value = self.value.get().unwrap();
                value
                    .latest()
                    .map(|data| data.text.clone())
                    .unwrap_or_default()
            }))
            .key("latest");
        }
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
fn stale_and_duplicate_completions_cannot_overwrite_current_data() -> Result<()> {
    let mut value = AsyncValue::<Data>::default();
    assert_eq!(value.read(), Err(AsyncReadError::Idle));
    let first = value.begin()?;
    assert_eq!(value.read(), Err(AsyncReadError::Pending));
    let second = value.begin()?;
    assert!(!value.resolve(
        first,
        Data {
            text: "stale".into()
        }
    ));
    assert!(value.resolve(
        second,
        Data {
            text: "current".into()
        }
    ));
    assert!(!value.reject(second, "late failure".into()));
    assert_eq!(value.read().unwrap().text, "current");
    let third = value.begin()?;
    assert_eq!(value.latest().unwrap().text, "current");
    assert_eq!(value.read(), Err(AsyncReadError::Pending));
    assert!(value.reject(third, "failed".into()));
    assert_eq!(value.error(), Some("failed"));
    assert_eq!(value.read(), Err(AsyncReadError::Failed("failed".into())));
    let encoded = value.encode()?;
    assert_eq!(AsyncValue::<Data>::decode(&encoded)?, value);
    let fourth = value.begin()?;
    assert!(value.cancel(fourth)?);
    assert!(!value.resolve(fourth, Data::default()));
    assert_eq!(value.phase(), AsyncPhase::Idle);
    assert_eq!(value.read(), Err(AsyncReadError::Idle));
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

#[test]
fn async_result_codec_preserves_domain_states_and_child_schema() -> Result<()> {
    #[derive(Record, Debug, PartialEq)]
    struct OtherData {
        text: String,
    }
    assert_ne!(
        <AsyncResult<Data> as Record>::FORMAT,
        <AsyncResult<OtherData> as Record>::FORMAT
    );
    for value in [
        Err(AsyncReadError::Idle),
        Err(AsyncReadError::Pending),
        Err(AsyncReadError::Failed(String::new())),
        Err(AsyncReadError::Failed("offline\n☁".into())),
        Ok(Data {
            text: "value\n☀".into(),
        }),
    ] {
        let encoded = record_codec::encode_record(&value)?;
        assert_eq!(
            record_codec::decode_record::<AsyncResult<Data>>(&encoded)?,
            value
        );
        assert!(record_codec::decode_record::<AsyncResult<OtherData>>(&encoded).is_err());
        let mut extra = value.encode()?;
        record_codec::append(&mut extra, &0_i64)?;
        assert!(AsyncResult::<Data>::decode(&extra).is_err());
    }
    let mut invalid = String::new();
    record_codec::append(&mut invalid, &9_i64)?;
    assert!(AsyncResult::<Data>::decode(&invalid).is_err());
    Ok(())
}

#[test]
fn owned_reads_and_standard_question_mark_preserve_unavailability() -> Result<()> {
    fn project(input: AsyncResult<Data>) -> AsyncResult<Data> {
        let mut data = input?;
        data.text.push('!');
        Ok(data)
    }
    let mut value = AsyncValue::<Data>::default();
    assert_eq!(
        project(value.clone().into_read()),
        Err(AsyncReadError::Idle)
    );
    let generation = value.begin()?;
    assert_eq!(
        project(value.clone().into_read()),
        Err(AsyncReadError::Pending)
    );
    value.resolve(
        generation,
        Data {
            text: "ready".into(),
        },
    );
    assert_eq!(project(value.clone().into_read()).unwrap().text, "ready!");
    let generation = value.begin()?;
    assert_eq!(
        project(value.clone().into_read()),
        Err(AsyncReadError::Pending)
    );
    value.reject(generation, "failed".into());
    assert_eq!(
        project(value.into_read()),
        Err(AsyncReadError::Failed("failed".into()))
    );
    Ok(())
}
