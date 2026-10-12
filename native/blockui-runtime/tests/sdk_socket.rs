#![cfg(feature = "testing")]
use blockui::*;
use std::time::Duration;
trait SocketIdentity {
    fn raw_identity(&self) -> u64;
}
impl SocketIdentity for std::net::TcpStream {
    #[allow(clippy::unnecessary_cast)] // RawSocket is usize-sized.
    fn raw_identity(&self) -> u64 {
        #[cfg(unix)]
        {
            std::os::fd::AsRawFd::as_raw_fd(self) as u64
        }
        #[cfg(windows)]
        {
            std::os::windows::io::AsRawSocket::as_raw_socket(self) as u64
        }
    }
}
#[actor(name = "sdk.net")]
struct Client {
    endpoint: State<String>,
    draft: State<String>,
    result: State<String>,
    completed: State<i64>,
    fail_view: State<bool>,
    reject_callback: State<bool>,
    socket: Resource<Socket>,
    file: Resource<File>,
    connect_task: Task,
    read_task: Task,
    write_task: Task,
}
fn decorate(text: &str) -> String {
    format!("v1:{text}")
}
#[actor_impl]
impl Client {
    #[action(name = "file.batch")]
    fn file_batch(&self, cx: &Cx<'_>) -> Result<()> {
        self.file.close(cx)?;
        self.file.open(cx, &self.endpoint.get(cx)?)?;
        for index in 0..64 {
            Task::new(&format!("file.batch.{index}")).read_file(
                cx,
                self.file,
                0,
                128,
                Self::noop_action(),
            )?;
        }
        Ok(())
    }

    #[action(name = "file.read")]
    fn file_read(&self, cx: &Cx<'_>) -> Result<()> {
        let this = self.bind(cx)?;
        this.file.close()?;
        this.file.open(&this.endpoint.get())?;
        this.read_task
            .read_file(&this.file, 0, 128, Self::complete_action())
    }

    #[action(name = "net.endpoint_port")]
    fn endpoint_port(&self, cx: &Cx<'_>, port: i64) -> Result<()> {
        if !(1..=65535).contains(&port) {
            return Err(Error::Application("invalid test server port".into()));
        }
        self.endpoint.set(cx, &format!("127.0.0.1:{port}"))
    }

    #[action(name = "net.connect")]
    fn connect(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        let this = self.bind(cx)?;
        let task = this.connect_task.on_reload(match mode {
            3 => ReloadPolicy::Cancel,
            4 => ReloadPolicy::Restart,
            _ => ReloadPolicy::Preserve,
        });
        let socket = this.socket.on_reload(match mode {
            5 => ReloadPolicy::Cancel,
            6 => ReloadPolicy::Restart,
            _ => ReloadPolicy::Preserve,
        });
        task.connect_socket(
            &socket,
            &this.endpoint.get(),
            Duration::from_secs(3),
            Self::complete_action(),
        )?;
        if mode == 1 {
            return Err(Error::Application("rollback connect".into()));
        }
        if mode == 2 {
            this.fail_view.set(true)?;
        }
        Ok(())
    }
    #[action(name = "net.read")]
    fn read(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        let this = self.bind(cx)?;
        let task = this.read_task.on_reload(match mode {
            3 => ReloadPolicy::Cancel,
            4 => ReloadPolicy::Restart,
            _ => ReloadPolicy::Preserve,
        });
        let timeout = if mode == 5 {
            Duration::from_millis(30)
        } else {
            Duration::from_secs(3)
        };
        task.read_socket(&this.socket, 128, timeout, Self::complete_action())?;
        if mode == 1 {
            return Err(Error::Application("rollback read".into()));
        }
        if mode == 2 {
            this.fail_view.set(true)?;
        }
        if mode == 6 {
            this.socket.close()?;
        }
        Ok(())
    }
    #[action(name = "net.write")]
    fn write(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        let this = self.bind(cx)?;
        this.write_task.write_socket(
            &this.socket,
            this.result.get().as_bytes(),
            Duration::from_secs(3),
            Self::complete_action(),
        )?;
        if mode == 1 {
            return Err(Error::Application("rollback write".into()));
        }
        if mode == 2 {
            this.fail_view.set(true)?;
        }
        Ok(())
    }
    #[action(name = "net.reject")]
    fn reject(&self, cx: &Cx<'_>, enabled: bool) -> Result<()> {
        self.reject_callback.set(cx, enabled)
    }
    #[action(name = "net.noop")]
    fn noop(&self, _: &Cx<'_>) -> Result<()> {
        Ok(())
    }
    #[action(name = "net.close")]
    fn close(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        self.socket.close(cx)?;
        if mode != 0 {
            return Err(Error::Application("rollback close".into()));
        }
        Ok(())
    }
    #[action(name = "net.clear")]
    fn clear(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        let this = self.bind(cx)?;
        this.connect_task.cancel()?;
        this.read_task.cancel()?;
        this.write_task.cancel()?;
        if mode != 0 {
            return Err(Error::Application("rollback clear".into()));
        }
        Ok(())
    }
    #[action(name = "net.consume_read")]
    fn consume_read(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        let this = self.bind(cx)?;
        let bytes = this
            .read_task
            .result()?
            .ok_or_else(|| Error::Application("missing read".into()))?;
        this.read_task.cancel()?;
        this.result.set(
            &String::from_utf8(bytes).map_err(|_| Error::Application("invalid UTF8".into()))?,
        )?;
        if mode == 1 {
            return Err(Error::Application("reject consumed result".into()));
        }
        if mode == 2 {
            this.fail_view.set(true)?;
        }
        Ok(())
    }
    #[action(name = "net.complete")]
    fn complete(&self, cx: &Cx<'_>, id: i64) -> Result<()> {
        let this = self.bind(cx)?;
        let id = id as u64;
        let task = if id == this.connect_task.key() {
            self.connect_task
        } else if id == this.read_task.key() {
            self.read_task
        } else if id == this.write_task.key() {
            self.write_task
        } else {
            return Err(Error::Application("unexpected network task".into()));
        };
        let value = match task.result(cx) {
            Ok(Some(bytes)) if id == this.connect_task.key() => {
                assert!(bytes.is_empty());
                "connected".into()
            }
            Ok(Some(bytes)) if id == this.write_task.key() => {
                assert!(bytes.is_empty());
                "sent".into()
            }
            Ok(Some(bytes)) => String::from_utf8(bytes)
                .map_err(|_| Error::Application("invalid UTF8 response".into()))?,
            Ok(None) => return Err(Error::Application("completion has no result".into())),
            Err(error) => format!("error:{error}"),
        };
        this.result.set(&decorate(&value))?;
        this.completed.update(|value| Ok(value + 1))?;
        if this.reject_callback.get() {
            return Err(Error::Application("rollback network completion".into()));
        }
        Ok(())
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let this = self.bind(cx)?;
        if this.fail_view.get() {
            return Err(Error::InvalidView("reject prepared network turn"));
        }
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    TextInput::new(self.draft)
                        .key("sdk.net.draft.editor")
                        .label("Draft"),
                );
                __blockui_tree.add(Text::new(this.result.get()).key("sdk.net.output"));
                __blockui_tree.add(
                    Text::new(format!("Completed {}", this.completed.get())).key("sdk.net.count"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
export_actor!(Client);

#[test]
fn descriptor_view_has_no_native_handles_or_reactor() {
    let actor = blockui_runtime::Actor::for_test::<Client>().unwrap();
    assert_eq!(actor.socket_count(), 0);
    assert_eq!(actor.network_worker_count(), 0);
    assert_eq!(actor.io_worker_count(), 0);
    assert_eq!(actor.view.order.len(), 4);
}

fn action(
    actor: &mut blockui_runtime::Actor,
    name: &str,
    value: i64,
) -> std::result::Result<(), String> {
    actor.dispatch(blockui_runtime::Message {
        action: abi::key(name),
        value,
    })
}
fn wait(actor: &mut blockui_runtime::Actor, completed: i64) {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while actor.integer("sdk.net.completed") < completed && std::time::Instant::now() < deadline {
        actor.tick().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(actor.integer("sdk.net.completed"), completed);
}
fn server() -> std::net::TcpListener {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}
fn accept(listener: &std::net::TcpListener) -> std::net::TcpStream {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        match listener.accept() {
            Ok((peer, _)) => {
                peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                return peer;
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(1))
            }
            Err(error) => panic!("bounded accept failed: {error}"),
        }
    }
}
fn connected() -> (blockui_runtime::Actor, std::net::TcpStream) {
    let mut actor = blockui_runtime::Actor::for_test::<Client>().unwrap();
    let listener = server();
    actor
        .set_text(
            abi::key("sdk.net.endpoint"),
            &listener.local_addr().unwrap().to_string(),
        )
        .unwrap();
    action(&mut actor, "net.connect", 0).unwrap();
    wait(&mut actor, 1);
    (actor, accept(&listener))
}
#[test]
fn failed_actions_and_prepared_views_never_initiate_connect_or_consume_data() {
    use std::io::{Read, Write};
    let mut actor = blockui_runtime::Actor::for_test::<Client>().unwrap();
    let listener = server();
    actor
        .set_text(
            abi::key("sdk.net.endpoint"),
            &listener.local_addr().unwrap().to_string(),
        )
        .unwrap();
    for mode in [1, 2, 4, 6] {
        assert!(action(&mut actor, "net.connect", mode).is_err());
        assert!(actor.tasks.is_empty());
        assert_eq!(actor.socket_count(), 0);
        assert_eq!(actor.io_result_count(), 0);
        assert_eq!(actor.network_waiting_count(), 0);
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    // Reservation and generation rollback permit retry with the same identities.
    action(&mut actor, "net.connect", 0).unwrap();
    wait(&mut actor, 1);
    let mut peer = accept(&listener);
    assert_eq!(actor.text("sdk.net.result"), "v1:connected");
    let resource = abi::key("sdk.net.socket");
    let fd = actor.socket(resource).unwrap().raw_identity();
    peer.write_all(b"stay").unwrap();
    assert!(action(&mut actor, "net.read", 1).is_err());
    assert!(action(&mut actor, "net.read", 2).is_err());
    assert_eq!(actor.socket(resource).unwrap().raw_identity(), fd);
    action(&mut actor, "net.read", 0).unwrap();
    wait(&mut actor, 2);
    assert_eq!(actor.text("sdk.net.result"), "v1:stay");
    assert_eq!(actor.network_buffer_bytes(), 4);
    actor
        .set_text(abi::key("sdk.net.result"), "outgoing")
        .unwrap();
    peer.set_nonblocking(true).unwrap();
    for mode in [1, 2] {
        assert!(action(&mut actor, "net.write", mode).is_err());
        assert_eq!(
            peer.read(&mut [0; 16]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    peer.set_nonblocking(false).unwrap();
    action(&mut actor, "net.write", 0).unwrap();
    wait(&mut actor, 3);
    let mut bytes = [0; 8];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(&bytes, b"outgoing");
    assert_eq!(actor.text("sdk.net.result"), "v1:sent");
    assert!(action(&mut actor, "net.close", 1).is_err());
    assert_eq!(actor.socket(resource).unwrap().raw_identity(), fd);
    action(&mut actor, "net.clear", 0).unwrap();
    assert_eq!(actor.io_result_count(), 0);
    assert_eq!(actor.network_buffer_bytes(), 0);
    action(&mut actor, "net.close", 0).unwrap();
    assert_eq!(actor.socket_count(), 0);
    assert_eq!(peer.read(&mut [0; 1]).unwrap(), 0);
    assert_eq!(actor.io_worker_count(), 0);
}
#[test]
fn rejecting_callback_keeps_host_connection_and_result_for_current_code_retry() {
    let listener = server();
    let mut actor = blockui_runtime::Actor::for_test::<Client>().unwrap();
    actor
        .set_text(
            abi::key("sdk.net.endpoint"),
            &listener.local_addr().unwrap().to_string(),
        )
        .unwrap();
    action(&mut actor, "net.reject", 1).unwrap();
    action(&mut actor, "net.connect", 0).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    let mut rejected = false;
    while std::time::Instant::now() < deadline {
        if actor.tick().is_err() {
            rejected = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(rejected);
    assert_eq!(actor.text("sdk.net.result"), "");
    assert_eq!(actor.integer("sdk.net.completed"), 0);
    assert_eq!(actor.socket_count(), 1);
    assert_eq!(actor.io_result_count(), 1);
    let _peer = accept(&listener);
    let fd = actor
        .socket(abi::key("sdk.net.socket"))
        .unwrap()
        .raw_identity();
    action(&mut actor, "net.reject", 0).unwrap();
    action(
        &mut actor,
        "net.complete",
        abi::key("sdk.net.connect_task") as i64,
    )
    .unwrap();
    assert_eq!(actor.text("sdk.net.result"), "v1:connected");
    assert_eq!(
        actor
            .socket(abi::key("sdk.net.socket"))
            .unwrap()
            .raw_identity(),
        fd
    );
    assert_eq!(actor.integer("sdk.net.completed"), 1);
}
#[test]
fn tasks_capture_connection_ownership_and_errors_release_reservations() {
    use std::io::Write;
    let (mut actor, mut peer) = connected();
    action(&mut actor, "net.read", 6).unwrap(); // Close registry while admitted read owns the connection.
    assert_eq!(actor.socket_count(), 0);
    peer.write_all(b"captured").unwrap();
    wait(&mut actor, 2);
    assert_eq!(actor.text("sdk.net.result"), "v1:captured");
    action(&mut actor, "net.clear", 0).unwrap();
    let listener = server();
    actor
        .set_text(
            abi::key("sdk.net.endpoint"),
            &listener.local_addr().unwrap().to_string(),
        )
        .unwrap();
    action(&mut actor, "net.connect", 0).unwrap();
    wait(&mut actor, 3);
    let _peer = accept(&listener);
    action(&mut actor, "net.read", 5).unwrap();
    wait(&mut actor, 4);
    assert!(actor.text("sdk.net.result").starts_with("v1:error:"));
    assert!(actor.text("sdk.net.result").contains("timed out"));
    assert_eq!(actor.network_waiting_count(), 0);
    action(&mut actor, "net.close", 0).unwrap();
    action(&mut actor, "net.clear", 0).unwrap();
    let unused = server();
    let address = unused.local_addr().unwrap();
    drop(unused);
    actor
        .set_text(abi::key("sdk.net.endpoint"), &address.to_string())
        .unwrap();
    action(&mut actor, "net.connect", 0).unwrap();
    wait(&mut actor, 5);
    assert!(actor.text("sdk.net.result").starts_with("v1:error:"));
    assert_eq!(actor.socket_count(), 0);
    action(&mut actor, "net.clear", 0).unwrap();
    let retry = server();
    actor
        .set_text(
            abi::key("sdk.net.endpoint"),
            &retry.local_addr().unwrap().to_string(),
        )
        .unwrap();
    action(&mut actor, "net.connect", 0).unwrap();
    wait(&mut actor, 6);
    assert_eq!(actor.socket_count(), 1);
    let _peer = accept(&retry);
    assert_eq!(actor.io_worker_count(), 0);
}

#[test]
fn queued_results_do_not_alias_reused_tasks_and_failed_clear_preserves_callback() {
    use std::io::Write;
    let (mut actor, mut peer) = connected();
    action(&mut actor, "net.clear", 0).unwrap();
    action(&mut actor, "net.read", 0).unwrap();
    for _ in 0..256 {
        actor
            .enqueue(blockui_runtime::Message {
                action: abi::key("net.noop"),
                value: 0,
            })
            .unwrap();
    }
    peer.write_all(b"old").unwrap();
    let task = abi::key("sdk.net.read_task");
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while !actor.tasks[&task].is_finished() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(actor.tasks[&task].is_finished());
    actor.tick().unwrap(); // Full mailbox keeps completion in the channel; frees 32 slots.
    actor.tick().unwrap(); // Admit completion behind the remaining noops.
    assert!(!actor.tasks.contains_key(&task));
    assert_eq!(actor.integer("sdk.net.completed"), 1);
    action(&mut actor, "net.read", 0).unwrap(); // Supersede cached old bytes before old callback.
    peer.write_all(b"new").unwrap();
    wait(&mut actor, 2);
    assert_eq!(actor.text("sdk.net.result"), "v1:new");
    assert_eq!(actor.integer("sdk.net.completed"), 2);
    action(&mut actor, "net.read", 0).unwrap();
    for _ in 0..256 {
        actor
            .enqueue(blockui_runtime::Message {
                action: abi::key("net.noop"),
                value: 0,
            })
            .unwrap();
    }
    peer.write_all(b"retained").unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while !actor.tasks[&task].is_finished() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(actor.tasks[&task].is_finished());
    actor.tick().unwrap();
    actor.tick().unwrap();
    assert!(action(&mut actor, "net.clear", 1).is_err());
    wait(&mut actor, 3);
    assert_eq!(actor.text("sdk.net.result"), "v1:retained");
    action(&mut actor, "net.read", 0).unwrap();
    for _ in 0..256 {
        actor
            .enqueue(blockui_runtime::Message {
                action: abi::key("net.noop"),
                value: 0,
            })
            .unwrap();
    }
    peer.write_all(b"discarded").unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while !actor.tasks[&task].is_finished() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(actor.tasks[&task].is_finished());
    actor.tick().unwrap();
    actor.tick().unwrap();
    action(&mut actor, "net.clear", 0).unwrap();
    for _ in 0..16 {
        actor.tick().unwrap();
    }
    assert_eq!(actor.integer("sdk.net.completed"), 3);
    assert_eq!(actor.text("sdk.net.result"), "v1:retained");
    assert_eq!(actor.io_result_count(), 0);
}

#[test]
fn file_completion_respects_mailbox_order_and_reuse_invalidates_old_result() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let root = std::env::temp_dir().join(format!("identity-file-mailbox-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("input.txt");
    std::fs::write(&path, b"old").unwrap();
    let mut actor = blockui_runtime::Actor::for_test::<Client>().unwrap();
    actor
        .set_text(abi::key("sdk.net.endpoint"), path.to_str().unwrap())
        .unwrap();
    let wakes = Arc::new(AtomicUsize::new(0));
    let observer = wakes.clone();
    actor.set_wake(Arc::new(move || {
        observer.fetch_add(1, Ordering::SeqCst);
    }));
    let before = wakes.load(Ordering::SeqCst);
    action(&mut actor, "file.read", 0).unwrap();
    for _ in 0..256 {
        actor
            .enqueue(blockui_runtime::Message {
                action: abi::key("net.noop"),
                value: 0,
            })
            .unwrap();
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while wakes.load(Ordering::SeqCst) == before && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        wakes.load(Ordering::SeqCst) > before,
        "file completion was not published"
    );
    actor.tick().unwrap();
    assert_eq!(
        actor.integer("sdk.net.completed"),
        0,
        "file callback bypassed full mailbox"
    );
    actor.tick().unwrap(); // Admit its result behind existing messages.
    assert_eq!(actor.integer("sdk.net.completed"), 0);
    std::fs::write(&path, b"new").unwrap();
    action(&mut actor, "file.read", 0).unwrap();
    wait(&mut actor, 1);
    assert_eq!(actor.text("sdk.net.result"), "v1:new");
    action(&mut actor, "net.clear", 0).unwrap();
    drop(actor);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coalesced_wake_drains_mailbox_even_after_handler_failure() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    for fail in [false, true] {
        let mut actor = blockui_runtime::Actor::for_test::<Client>().unwrap();
        let pending = Arc::new(AtomicBool::new(false));
        let notification = pending.clone();
        actor.set_wake(Arc::new(move || {
            notification.store(true, Ordering::SeqCst);
        }));
        pending.store(false, Ordering::SeqCst);
        if fail {
            actor
                .enqueue(blockui_runtime::Message {
                    action: abi::key("unknown.handler"),
                    value: 0,
                })
                .unwrap();
        }
        for _ in 0..70 {
            actor
                .enqueue(blockui_runtime::Message {
                    action: abi::key("net.noop"),
                    value: 0,
                })
                .unwrap();
        }
        pending.store(true, Ordering::SeqCst); // One coalesced event for the whole burst.
        let mut turns = 0;
        let mut failures = 0;
        while pending.swap(false, Ordering::SeqCst) {
            turns += 1;
            assert!(turns <= 5, "idle wake loop did not stop");
            if actor.tick().is_err() {
                failures += 1;
            }
        }
        assert!(actor.mailbox.is_empty(), "mailbox stalled after one wake");
        assert_eq!(failures, usize::from(fail));
        assert!(
            turns >= 3,
            "bounded turns must yield between message batches"
        );
    }
}
#[test]
fn coalesced_wake_drains_file_completion_batches() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    let root = std::env::temp_dir().join(format!("identity-file-batch-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("input.txt");
    std::fs::write(&path, b"all").unwrap();
    let mut actor = blockui_runtime::Actor::for_test::<Client>().unwrap();
    actor
        .set_text(abi::key("sdk.net.endpoint"), path.to_str().unwrap())
        .unwrap();
    let pending = Arc::new(AtomicBool::new(false));
    let wakes = Arc::new(AtomicUsize::new(0));
    let notification = pending.clone();
    let observer = wakes.clone();
    actor.set_wake(Arc::new(move || {
        observer.fetch_add(1, Ordering::SeqCst);
        notification.store(true, Ordering::SeqCst);
    }));
    let before = wakes.load(Ordering::SeqCst);
    action(&mut actor, "file.batch", 0).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while wakes.load(Ordering::SeqCst) < before + 64 && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        wakes.load(Ordering::SeqCst) >= before + 64,
        "workers did not publish every result"
    );
    let mut turns = 0;
    while pending.swap(false, Ordering::SeqCst) {
        turns += 1;
        assert!(turns <= 4, "completion wake loop did not stop");
        actor.tick().unwrap();
    }
    assert!(
        actor.tasks.is_empty(),
        "file results stalled after admission budget"
    );
    assert_eq!(actor.io_result_bytes(), 64 * 3);
    drop(actor);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn consumed_read_result_and_quota_roll_back_with_handler_or_view_failure() {
    use std::io::Write;
    let (mut actor, mut peer) = connected();
    action(&mut actor, "net.clear", 0).unwrap();
    action(&mut actor, "net.read", 0).unwrap();
    peer.write_all(b"retained").unwrap();
    wait(&mut actor, 2);
    let slot = actor.slot_address("sdk.net.result");
    for mode in [1, 2] {
        assert!(action(&mut actor, "net.consume_read", mode).is_err());
        assert_eq!(actor.text("sdk.net.result"), "v1:retained");
        assert_eq!(actor.io_result_count(), 1);
        assert_eq!(actor.io_result_bytes(), 8);
        assert_eq!(actor.network_buffer_bytes(), 8);
        assert_eq!(actor.slot_address("sdk.net.result"), slot);
    }
    actor.replace_for_test::<Client>().unwrap();
    action(&mut actor, "net.consume_read", 0).unwrap();
    assert_eq!(actor.text("sdk.net.result"), "retained");
    assert_eq!(actor.slot_address("sdk.net.result"), slot);
    assert_eq!(actor.io_result_count(), 0);
    assert_eq!(actor.io_result_bytes(), 0);
    assert_eq!(actor.network_buffer_bytes(), 0);
    assert_eq!(actor.socket_count(), 1);
}
