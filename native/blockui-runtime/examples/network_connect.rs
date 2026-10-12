//! Saturated local listen backlog proves connect waits through actual DLL reload.
#[cfg(target_os = "linux")]
fn main() -> Result<(), String> {
    use blockui_abi::key;
    use blockui_runtime::{Actor, AsyncMessage, AsyncReloadPolicy, Message, network::NetworkIo};
    use std::{
        io::Write,
        net::{TcpListener, TcpStream},
        os::fd::AsRawFd,
        path::Path,
        time::{Duration, Instant},
    };
    fn backlog() -> (TcpListener, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        assert_eq!(unsafe { libc::listen(listener.as_raw_fd(), 0) }, 0);
        let filler = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        listener.set_nonblocking(true).unwrap();
        (listener, filler)
    }
    fn accept(listener: &TcpListener) -> TcpStream {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match listener.accept() {
                Ok((stream, _)) => return stream,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("bounded accept failed: {error}"),
            }
        }
    }
    fn wait_until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !condition() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(condition());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("network_connect <base.so> <replacement.so> <rejected.so>".into());
    }
    let network = NetworkIo::new().map_err(|e| e.to_string())?;
    let mut actor = Actor::new(641, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[0]))?;
    let slot = actor.slot_address("notes");
    let state = actor.state_address();
    let (listener, _filler) = backlog();
    let connect = network
        .connect(listener.local_addr().unwrap(), Duration::from_secs(3))
        .map_err(|e| e.to_string())?;
    let adapter = network.clone();
    actor.spawn_async_message(
        1,
        async move {
            let stream = connect.await.unwrap();
            // This fixture's protocol is one newline-terminated UTF-8 frame.
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut bytes = Vec::new();
            loop {
                let chunk = adapter
                    .read(
                        &stream,
                        64,
                        deadline.saturating_duration_since(Instant::now()),
                    )
                    .unwrap()
                    .await
                    .unwrap();
                assert!(!chunk.is_empty());
                bytes.extend(chunk);
                assert!(bytes.len() <= 64);
                if bytes.last() == Some(&b'\n') {
                    break;
                }
            }
            AsyncMessage::text(key("text_message"), String::from_utf8(bytes).unwrap()).unwrap()
        },
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Preserve,
    )?;
    wait_until(|| network.waiting_count() == 1);
    let worker = actor.tasks[&1].worker_id();
    let generation = actor.tasks[&1].generation();
    actor.spawn_async(
        2,
        async {
            Message {
                action: key("task_done"),
                value: 42,
            }
        },
        Message {
            action: key("task_done"),
            value: -1,
        },
    )?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while actor.integer("task_result") != 42 && Instant::now() < deadline {
        actor.tick()?;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(actor.integer("task_result"), 42);
    assert!(actor.reload(Path::new(&args[2])).is_err());
    assert_eq!(network.waiting_count(), 1);
    assert!(actor.tasks.contains_key(&1));
    actor.reload(Path::new(&args[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("notes"), slot);
    assert_eq!(actor.tasks[&1].worker_id(), worker);
    assert_eq!(actor.tasks[&1].generation(), generation);
    let _accepted_filler = accept(&listener); // Release the occupied accept queue.
    let mut peer = accept(&listener);
    peer.write_all("connected 新代码😀\n".as_bytes()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while actor.text("notes") != "v2:connected 新代码😀\n" && Instant::now() < deadline {
        actor.tick()?;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(actor.text("notes"), "v2:connected 新代码😀\n");
    assert_eq!(network.waiting_count(), 0);
    assert!(!actor.tasks.contains_key(&1));
    let (listener, _filler) = backlog();
    let connect = network
        .connect(listener.local_addr().unwrap(), Duration::from_secs(3))
        .map_err(|e| e.to_string())?;
    actor.spawn_async_message(
        3,
        async move {
            let _stream = connect.await.unwrap();
            AsyncMessage::text(key("text_message"), "unexpected connection").unwrap()
        },
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Cancel,
    )?;
    wait_until(|| network.waiting_count() == 1);
    assert!(actor.reload(Path::new(&args[2])).is_err());
    assert_eq!(network.waiting_count(), 1);
    assert!(actor.tasks.contains_key(&3));
    let before = actor.text("notes").to_owned();
    actor.reload(Path::new(&args[0]))?;
    assert!(!actor.tasks.contains_key(&3));
    assert_eq!(network.waiting_count(), 0);
    actor.tick()?;
    assert_eq!(actor.text("notes"), before);
    assert_eq!(actor.io_worker_count(), 0);
    println!(
        "PASS asynchronous TCP connect: real backlog wait permits unrelated work, state/slot/task generation/worker survive DLL replacement, connected response runs new code, rejected candidate retains Cancel connect and successful reload unregisters it, zero file workers"
    );
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), String> {
    Err("network_connect backlog proof currently requires Linux".into())
}
