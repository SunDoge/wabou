//! Real application editor continuity while a TCP read crosses DLL replacement.
use super::*;
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

fn target(source: &NativeSource, label: &str, role: shell::SemanticRole) -> u64 {
    source
        .semantic_snapshot()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.role == role && node.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
        .id
}
fn wait(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "TCP application native wait expired"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub fn proof(base: &Path, replacement: &Path, flat: Option<&Path>) -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).unwrap();
    let mut actor = Actor::new(654, Path::new(".runtime"))?;
    actor.reload(flat.unwrap_or(base))?;
    actor.set_text(
        key("tcp.client.endpoint"),
        &listener.local_addr().unwrap().to_string(),
    )?;
    actor.set_text(key("tcp.client.draft"), "A😀bc unsaved")?;
    let state = actor.state_address();
    let slot = actor.slot_address("tcp.client.draft");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 720, 620);
    let input = target(&source, "Outgoing text", shell::SemanticRole::TextInput);
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 720, 620);
    let selection = source.editors[&input].text_selection();
    assert_eq!(selection.as_ref().map(|s| (s.anchor, s.head)), Some((1, 3)));
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let old_parent = source.actor.view.bindings[&input].node.parent;
    let connect = target(&source, "Connect", shell::SemanticRole::Button);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: connect }));
    wait(|| {
        source.poll_async();
        source.build_frame(&mut tcx, 720, 620);
        source.actor.integer("tcp.client.phase") == 2
    });
    let mut peer = None;
    wait(|| match listener.accept() {
        Ok((stream, _)) => {
            peer = Some(stream);
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => false,
        Err(e) => panic!("accept: {e}"),
    });
    let mut peer = peer.unwrap();
    let socket = key("tcp.client.socket");
    let connection = source.actor.socket(socket).unwrap() as *const TcpStream;
    let receive = target(&source, "Receive chunk", shell::SemanticRole::Button);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: receive }));
    assert_eq!(source.actor.integer("tcp.client.phase"), 4);
    wait(|| source.actor.network_waiting_count() == 1);
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 620);
    if flat.is_some() {
        assert_ne!(source.actor.view.bindings[&input].node.parent, old_parent);
        let root = source
            .actor
            .view
            .bindings
            .values()
            .find(|b| b.node.parent == 0)
            .unwrap();
        assert_eq!(root.node.kind, SCROLL);
    }
    let check = |source: &NativeSource| {
        assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
        assert_eq!(source.actor.state_address(), state);
        assert_eq!(source.actor.slot_address("tcp.client.draft"), slot);
        assert_eq!(source.actor.text("tcp.client.draft"), "A😀bc unsaved");
        assert_eq!(
            source.actor.socket(socket).unwrap() as *const TcpStream,
            connection
        );
        assert_eq!(source.focus, Some(input));
        assert_eq!(source.ids[&input], node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            widget
        );
        assert_eq!(source.editors[&input].text_selection(), selection);
        assert_eq!(
            source.editors[&input].current_value(),
            Some("A😀bc unsaved")
        );
        assert_eq!(source.semantic_snapshot().unwrap().focus, Some(input));
    };
    check(&source);
    assert_eq!(source.actor.integer("tcp.client.phase"), 4);
    assert_eq!(source.actor.network_waiting_count(), 1);
    let evaluations = source.actor.view.bindings[&input].evaluations;
    peer.write_all(b"echo response").unwrap();
    wait(|| {
        source.poll_async();
        source.build_frame(&mut tcx, 720, 620);
        source.actor.integer("tcp.client.phase") == 2
    });
    check(&source);
    assert_eq!(
        source.actor.text("tcp.client.status"),
        "Updated · received 13 bytes"
    );
    assert_eq!(source.actor.view.bindings[&input].evaluations, evaluations);
    let snapshot = source.semantic_snapshot().unwrap();
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|n| n.label.as_deref() == Some("Updated · received 13 bytes"))
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|n| n.label.as_deref() == Some("Last received chunk")
                && n.value.as_deref() == Some("echo response"))
    );
    for width in [320, 720] {
        source.build_frame(&mut tcx, width, 620);
        check(&source);
        let output = target(
            &source,
            "Last received chunk",
            shell::SemanticRole::TextInput,
        );
        let mut bottom = 0.0;
        for id in [
            key("tcp.title"),
            target(
                &source,
                "Server address · IP:port",
                shell::SemanticRole::Label,
            ),
            target(&source, "Outgoing text", shell::SemanticRole::Label),
            input,
            target(&source, "Last received chunk", shell::SemanticRole::Label),
            output,
        ] {
            let rect = source.widget_layout_rect(id).ok_or("layout missing")?;
            assert!(rect.iter().all(|v| v.is_finite()));
            assert!(
                rect[0] >= 0.0
                    && rect[1] >= bottom
                    && rect[2] <= width as f32 + 0.5
                    && rect[3] <= 620.0,
                "{id}: {rect:?}"
            );
            bottom = rect[3];
        }
    }
    // A deterministic long-feedback fixture exercises the application surface,
    // independently of the length/wording of this platform's network errors.
    source.actor.set_text(
        key("tcp.client.status"),
        &"Network feedback with retained response. ".repeat(60),
    )?;
    source.build_frame(&mut tcx, 320, 360);
    let shell = key("tcp.shell");
    let port = source
        .placed
        .iter()
        .find(|p| p.node_id == source.ids[&shell])
        .unwrap();
    assert!(
        port.scroll.range[1] > 0.0,
        "long feedback must be reachable through root scrolling"
    );
    let output = target(
        &source,
        "Last received chunk",
        shell::SemanticRole::TextInput,
    );
    assert!(
        source.handle_semantic_action(shell::SemanticAction::ScrollIntoView { target: output })
    );
    source.build_frame(&mut tcx, 320, 360);
    let rect = source.widget_layout_rect(output).unwrap();
    assert!(
        rect[1] >= 0.0 && rect[3] <= 360.0,
        "revealed response: {rect:?}"
    );
    assert_eq!(source.actor.text("tcp.client.response"), "echo response");
    assert_eq!(
        source.actor.socket(socket).unwrap() as *const TcpStream,
        connection
    );
    assert!(source.actor.view.renderer[&shell].scroll > 0.0);
    source.handle_event(UiEvent::TextInput("🦀".into()));
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.actor.text("tcp.client.draft"), "A🦀bc unsaved");
    let disconnect = target(&source, "Disconnect", shell::SemanticRole::Button);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: disconnect }));
    assert_eq!(source.actor.socket_count(), 0);
    assert_eq!(source.actor.integer("tcp.client.phase"), 0);
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(
        source.actor.view.renderer[&shell].scroll, 0.0,
        "short status after disconnect clamps old feedback scroll"
    );
    println!(
        "PASS TCP client native UI: flat-to-Field/root-scroll GUI replacement preserves pending read, socket allocation, state slot, editor allocation, Taffy node, Unicode draft, focus and UTF16 selection; new-code completion appears in semantic frame without invalidating input, narrow layout and subsequent editing work"
    );
    Ok(())
}
