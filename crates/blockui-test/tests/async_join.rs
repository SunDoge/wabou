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
        #[ui(Column::default())]
        {
            #[ui]
            live!(self.combined().unwrap().map(|data| Text::new(data.text)))
                .boundary(fallback)
                .mount("joined");
            #[ui]
            Button::new("Start", self.start_action());
            #[ui]
            Button::new("Fail right", self.fail_right_action());
        }
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
#[test]
fn join_uses_deterministic_state_precedence() {
    let cases: [AsyncResult<i64>; 5] = [
        Ok(1),
        Err(AsyncReadError::Idle),
        Err(AsyncReadError::Pending),
        Err(AsyncReadError::Failed("left failure".into())),
        Err(AsyncReadError::Failed("right failure".into())),
    ];
    let expected = [
        ["ready", "idle", "pending", "left failure", "right failure"],
        ["idle", "idle", "pending", "left failure", "right failure"],
        [
            "pending",
            "pending",
            "pending",
            "left failure",
            "right failure",
        ],
        ["left failure"; 5],
        ["right failure"; 5],
    ];
    for (row, left) in cases.iter().enumerate() {
        for (column, right) in cases.iter().enumerate() {
            let joined = left.clone().join(right.clone());
            let actual = match &joined {
                Ok(pair) => {
                    assert_eq!(*pair, (1, 1));
                    "ready"
                }
                Err(AsyncReadError::Idle) => "idle",
                Err(AsyncReadError::Pending) => "pending",
                Err(AsyncReadError::Failed(message)) => message.as_str(),
            };
            assert_eq!(actual, expected[row][column]);
        }
    }
}
