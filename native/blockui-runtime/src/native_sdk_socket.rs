//! Native editor continuity while SDK socket tasks complete through replaced code.
use super::*;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;
use std::{io::Write, net::TcpListener};

fn dispatch(source: &mut NativeSource, name: &str, value: i64) -> Result<(), String> {
    source.actor.dispatch(Message {
        action: key(name),
        value,
    })
}
fn wait(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !ready() {
        assert!(Instant::now() < deadline, "native socket wait expired");
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn completion(source: &mut NativeSource, tcx: &mut TextContext, count: i64, text: &str) {
    wait(|| {
        source.poll_async();
        source.build_frame(tcx, 480, 320);
        source.actor.integer("sdk.net.completed") == count
    });
    assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
    assert_eq!(
        source.actor.view.bindings[&key("sdk.net.output")].text,
        text
    );
    let snapshot = source.semantic_snapshot().unwrap();
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|node| node.id == key("sdk.net.output") && node.label.as_deref() == Some(text))
    );
}
pub fn proof(
    base: &Path,
    replacement: &Path,
    rejected: &Path,
    evolved: &Path,
) -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    listener.set_nonblocking(true).unwrap();
    let mut actor = Actor::new(653, Path::new(".runtime"))?;
    actor.reload(base)?;
    actor.set_text(
        key("sdk.net.endpoint"),
        &listener.local_addr().unwrap().to_string(),
    )?;
    actor.set_text(key("sdk.net.draft"), "A😀bc unsaved")?;
    let state = actor.state_address();
    let slot = actor.slot_address("sdk.net.draft");
    let input = key("sdk.net.draft.editor");
    let output = key("sdk.net.output");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.set_focus(Some(input));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 320);
    source
        .editors
        .get_mut(&input)
        .ok_or("editor missing")?
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 480, 320);
    let selection = source.editors[&input].text_selection();
    assert_eq!(selection.as_ref().map(|s| (s.anchor, s.head)), Some((1, 3)));
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let check = |source: &NativeSource| {
        assert_eq!(source.actor.state_address(), state);
        assert_eq!(source.actor.slot_address("sdk.net.draft"), slot);
        assert_eq!(source.actor.text("sdk.net.draft"), "A😀bc unsaved");
        assert_eq!(source.focus, Some(input));
        assert_eq!(source.ids[&input], node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            widget
        );
        assert_eq!(
            source.editors[&input].current_value(),
            Some("A😀bc unsaved")
        );
        assert_eq!(source.editors[&input].text_selection(), selection);
        let snapshot = source.semantic_snapshot().unwrap();
        assert_eq!(snapshot.focus, Some(input));
        assert!(snapshot.nodes.iter().any(|n| n.id == input
            && n.label.as_deref() == Some("Draft")
            && n.value.as_deref() == Some("A😀bc unsaved")));
    };
    dispatch(&mut source, "net.connect", 0)?;
    let mut peer = None;
    wait(|| match listener.accept() {
        Ok((stream, _)) => {
            peer = Some(stream);
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => false,
        Err(error) => panic!("accept failed: {error}"),
    });
    let mut peer = peer.unwrap();
    source.replace(replacement)?;
    completion(&mut source, &mut tcx, 1, "v2:connected");
    check(&source);
    let socket = key("sdk.net.socket");
    #[cfg(unix)]
    let fd = source.actor.socket(socket).unwrap().as_raw_fd();
    #[cfg(windows)]
    let fd = source.actor.socket(socket).unwrap().as_raw_socket();
    dispatch(&mut source, "net.read", 0)?;
    wait(|| source.actor.network_waiting_count() == 1);
    assert!(source.replace(rejected).is_err());
    source.build_frame(&mut tcx, 480, 320);
    check(&source);
    source.replace(base)?;
    source.build_frame(&mut tcx, 480, 320);
    let evaluations = source.actor.view.bindings[&input].evaluations;
    peer.write_all("response 😀".as_bytes()).unwrap();
    completion(&mut source, &mut tcx, 2, "v1:response 😀");
    assert_eq!(source.actor.view.bindings[&input].evaluations, evaluations);
    check(&source);
    dispatch(&mut source, "net.read", 0)?;
    wait(|| source.actor.network_waiting_count() == 1);
    source.replace(evolved)?;
    source.build_frame(&mut tcx, 480, 320);
    #[cfg(unix)]
    assert_eq!(source.actor.socket(socket).unwrap().as_raw_fd(), fd);
    #[cfg(windows)]
    assert_eq!(source.actor.socket(socket).unwrap().as_raw_socket(), fd);
    assert_eq!(source.actor.integer("sdk.net.bonus"), 0);
    peer.write_all(b"after migration").unwrap();
    completion(&mut source, &mut tcx, 3, "v2:after migration");
    check(&source);
    // The retained selection still handles native editing after replacement.
    source.handle_event(UiEvent::TextInput("🦀".into()));
    source.build_frame(&mut tcx, 480, 320);
    assert_eq!(source.actor.text("sdk.net.draft"), "A🦀bc unsaved");
    assert_eq!(
        source.editors[&input].current_value(),
        Some("A🦀bc unsaved")
    );
    for width in [320, 640] {
        source.build_frame(&mut tcx, width, 320);
        let mut bottom = 0.0;
        for id in [input, output, key("sdk.net.count")] {
            let rect = source.widget_layout_rect(id).ok_or("layout missing")?;
            assert!(rect.iter().all(|value| value.is_finite()));
            assert!(
                rect[0] >= 0.0
                    && rect[1] >= bottom
                    && rect[2] <= width as f32 + 0.5
                    && rect[3] <= 320.0
            );
            bottom = rect[3];
        }
    }
    dispatch(&mut source, "net.clear", 0)?;
    dispatch(&mut source, "net.close", 0)?;
    wait(|| source.actor.network_buffer_bytes() == 0);
    assert_eq!(source.actor.socket_count(), 0);
    println!(
        "PASS native SDK socket: new-code responses appear in completed semantic frames through real DLL replacement/schema migration without recreating editor or Taffy node, losing Unicode draft/focus/UTF16 selection, or invalidating unrelated input; rejected upgrade retains state, post-reload native editing works, constrained layouts remain ordered"
    );
    Ok(())
}
