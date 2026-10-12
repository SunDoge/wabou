# Async results in tag UI

The current API uses host-owned work, a durable `AsyncValue<T>` and an explicit
UI boundary. It does not automatically suspend a Rust computed function or
accept an arbitrary plugin Future. Normal Rust async work can execute in the
stable host through services; never retain a plugin future, waker or callback in
a host task. The timer below is a deterministic stand-in for such work.

`begin()` marks a new request pending and returns its generation. Pass that
value to the host task, then use `resolve` or `reject` with the same generation.
An older completion cannot overwrite a newer request. Task cancellation and
result generation are separate protections. Keeping the last successful value
with `latest()` is an explicit UI choice; `into_read()` selects the strict
idle/loading/error/ready boundary instead.

This complete headless example demonstrates loading, failure, retry,
cancellation, stale display and completion after implementation replacement:

```rust
use std::time::Duration;
use wabou_native::testing::{Role, Screen};
use wabou_native::*;
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
```

Reads inside the `live!` boundary belong to its structural observer. The
`latest` text has a separate property observer. Loading a new result does not
rerun the root view; replacement rebuilds it once. `forwarded` shows a derived
record result that can still react to synchronous state after loading.

The test advances a virtual clock; it never sleeps or initializes a renderer.
Compiled-in replacement checks actor state and timer delivery but does not
prove real dylib unloading. `scripts/native/verify_watch.py --async-value --wabou-native` covers the
separate real replacement path. For application-owned network/file work, see
[headless service integration](sdk.md#business-services-in-headless-tests).
For the planned async graph design and remaining limitations, see
[reactive async design](reactive-async-design.md).

Run the exact example as a repository test:

```sh
cargo test -p wabou-native --features testing --test async_guide
python3 scripts/native/verify_watch.py --async-value --wabou-native
```
