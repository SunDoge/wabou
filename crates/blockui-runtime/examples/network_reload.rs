//! Connected TCP futures stay in the host while their receiver code reloads.
use blockui_abi::key;
use blockui_runtime::{
    Actor, AsyncMessage, AsyncReloadPolicy, Message, SocketReloadPolicy, network::NetworkIo,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::AsRawFd,
    path::Path,
    time::{Duration, Instant},
};
fn pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let peer = listener.accept().unwrap().0;
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    (stream, peer)
}
fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !condition() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(condition());
}
fn finish(actor: &mut Actor, expected: &str) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while actor.text("notes") != expected && Instant::now() < deadline {
        actor.tick()?;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(actor.text("notes"), expected);
    Ok(())
}
// The application protocol owns framing and a total deadline, not the read API.
async fn line(network: NetworkIo, stream: TcpStream) -> AsyncMessage {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut bytes = Vec::new();
    loop {
        let chunk = network
            .read(
                &stream,
                1024,
                deadline.saturating_duration_since(Instant::now()),
            )
            .unwrap()
            .await
            .unwrap();
        assert!(!chunk.is_empty(), "unexpected EOF before frame");
        bytes.extend(chunk);
        assert!(bytes.len() <= blockui_abi::MAX_MESSAGE_TEXT_BYTES);
        if bytes.last() == Some(&b'\n') {
            break;
        }
    }
    AsyncMessage::text(key("text_message"), String::from_utf8(bytes).unwrap()).unwrap()
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("network_reload <base.so> <replacement.so> <rejected.so>".into());
    }
    let network = NetworkIo::new().map_err(|e| e.to_string())?;
    let mut actor = Actor::new(640, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[0]))?;
    let state = actor.state_address();
    let slot = actor.slot_address("notes");
    let (stream, mut peer) = pair();
    let fd = stream.as_raw_fd();
    actor.register_socket(1, stream, SocketReloadPolicy::Preserve)?;
    let read_stream = actor.socket(1).unwrap().try_clone().unwrap();
    actor.spawn_async_message(
        1,
        line(network.clone(), read_stream),
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Preserve,
    )?;
    wait_until(|| network.waiting_count() == 1);
    let worker = actor.tasks[&1].worker_id();
    // A parked network read cannot prevent the same worker completing other work.
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
    assert!(actor.tasks.contains_key(&1));
    assert!(actor.reload(Path::new(&args[2])).is_err());
    assert_eq!(actor.socket(1).unwrap().as_raw_fd(), fd);
    actor.reload(Path::new(&args[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("notes"), slot);
    assert_eq!(actor.tasks[&1].worker_id(), worker);
    assert_eq!(actor.socket(1).unwrap().as_raw_fd(), fd);
    // Split inside UTF-8 to prove framing precedes decode.
    let response = "网络响应🦀\n".as_bytes();
    peer.write_all(&response[..2]).unwrap();
    peer.write_all(&response[2..]).unwrap();
    finish(&mut actor, "v2:网络响应🦀\n")?;
    wait_until(|| network.waiting_count() == 0);
    let send = network
        .write(
            actor.socket(1).unwrap(),
            "request 😀\n".as_bytes(),
            Duration::from_secs(2),
        )
        .map_err(|e| e.to_string())?;
    actor.spawn_async_message(
        3,
        async move {
            let sent = send.await.unwrap();
            AsyncMessage::text(key("text_message"), format!("sent:{sent}")).unwrap()
        },
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Preserve,
    )?;
    let mut bytes = vec![0; "request 😀\n".len()];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(bytes, "request 😀\n".as_bytes());
    finish(&mut actor, &format!("v2:sent:{}", bytes.len()))?;
    let read = network
        .read(actor.socket(1).unwrap(), 64, Duration::from_secs(2))
        .map_err(|e| e.to_string())?;
    actor.spawn_async_message(
        4,
        async move {
            let bytes = read.await.unwrap();
            AsyncMessage::text(key("text_message"), String::from_utf8(bytes).unwrap()).unwrap()
        },
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Cancel,
    )?;
    wait_until(|| network.waiting_count() == 1);
    assert!(actor.reload(Path::new(&args[2])).is_err());
    assert!(actor.tasks.contains_key(&4));
    assert_eq!(network.waiting_count(), 1);
    let before = actor.text("notes").to_owned();
    actor.reload(Path::new(&args[0]))?;
    assert!(!actor.tasks.contains_key(&4));
    wait_until(|| network.waiting_count() == 0);
    actor.tick()?;
    assert_eq!(actor.text("notes"), before);
    assert_eq!(actor.socket(1).unwrap().as_raw_fd(), fd);
    assert_eq!(actor.io_worker_count(), 0);
    println!(
        "PASS connected TCP futures: pending read permits unrelated work, actor/state/slot/socket/future worker survive real DLL replacement, split UTF8 frame reaches new code, exact write bytes arrive, rejected upgrade retains Cancel read, successful reload unregisters it, zero file workers"
    );
    Ok(())
}
