use super::*;

#[test]
fn pointer_drag_selects_terminal_grid_text() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    widget.handle_event(&pointer(PointerPhase::Down, 0.1, 0.1, 1));
    widget.handle_event(&pointer(
        PointerPhase::Move,
        f64::from(DEFAULT_CELL_WIDTH) * 4.75,
        0.1,
        1,
    ));
    widget.handle_event(&pointer(
        PointerPhase::Up,
        f64::from(DEFAULT_CELL_WIDTH) * 4.75,
        0.1,
        0,
    ));

    assert_eq!(widget.selected_text().as_deref(), Some("hello"));
    assert!(!widget.selecting);

    let copy = widget.handle_event(&UiEvent::Key(KeyEvent {
        phase: KeyPhase::Down,
        key: "C".into(),
        key_without_modifiers: "c".into(),
        code: "KeyC".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL | Modifiers::SHIFT
        },
        repeat: false,
        synthetic: false,
    }));
    assert_eq!(
        copy.clipboard_request(),
        Some(&ClipboardRequest::Write("hello".into()))
    );

    let paste = widget.handle_event(&UiEvent::Key(KeyEvent {
        phase: KeyPhase::Down,
        key: "V".into(),
        key_without_modifiers: "v".into(),
        code: "KeyV".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL | Modifiers::SHIFT
        },
        repeat: false,
        synthetic: false,
    }));
    assert_eq!(paste.clipboard_request(), Some(&ClipboardRequest::Read));
}

#[test]
fn selection_changes_are_deduplicated_and_exposed_to_solid() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let cell = f64::from(DEFAULT_CELL_WIDTH);
    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    assert!(widget.take_node_event().is_none());
    for index in 0..1_000 {
        let column = 1.25 + f64::from(index % 4);
        widget.handle_event(&pointer(PointerPhase::Move, cell * column, 1.0, 1));
    }
    widget.handle_event(&pointer(PointerPhase::Move, cell * 4.75, 1.0, 1));
    assert!(
        widget.take_node_event().is_none(),
        "dragging updates paint locally without serializing selection text"
    );
    widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
    let selected = widget.take_node_event().expect("committed selection event");
    assert_eq!(selected.event_code, event::TERMINALSELECTIONCHANGE);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&selected.json).unwrap(),
        serde_json::json!({ "text": "hello", "kind": "simple" })
    );
    assert!(widget.take_node_event().is_none());

    widget.handle_event(&UiEvent::TextInput("x".into()));
    let cleared = widget.take_node_event().expect("selection clear event");
    assert_eq!(cleared.event_code, event::TERMINALSELECTIONCHANGE);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&cleared.json).unwrap(),
        serde_json::json!({ "text": null, "kind": null })
    );
    assert_eq!(widget.take_input(), b"x");
}

#[test]
fn live_pty_output_cannot_publish_an_uncommitted_selection() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    widget.handle_rio_events();
    let cell = f64::from(DEFAULT_CELL_WIDTH);
    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 4.75, 1.0, 1));

    widget.feed(b"!");
    widget.handle_rio_events();
    assert!(
        widget.take_node_event().is_none(),
        "PTY output must not bypass the pointer gesture's commit boundary"
    );

    widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
    let selected = widget.take_node_event().expect("committed selection event");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&selected.json).unwrap(),
        serde_json::json!({ "text": "hello", "kind": "simple" })
    );
}

#[test]
fn selection_event_kind_changes_even_when_text_is_identical() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let mut simple = Selection::new(
        SelectionType::Simple,
        Pos::new(Line(0), Column(0)),
        Side::Left,
    );
    simple.update(Pos::new(Line(0), Column(4)), Side::Right);
    widget.terminal.lock().selection = Some(simple);
    widget.sync_selection_change();
    let simple = widget.take_node_event().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&simple.json).unwrap(),
        serde_json::json!({ "text": "hello", "kind": "simple" })
    );

    widget.terminal.lock().selection = Some(Selection::new(
        SelectionType::Semantic,
        Pos::new(Line(0), Column(2)),
        Side::Left,
    ));
    widget.sync_selection_change();
    let word = widget.take_node_event().expect("selection kind changed");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&word.json).unwrap(),
        serde_json::json!({ "text": "hello", "kind": "word" })
    );
}

#[test]
fn shift_click_extends_existing_terminal_selection_in_both_directions() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let cell = f64::from(DEFAULT_CELL_WIDTH);

    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
    assert_eq!(widget.selected_text().as_deref(), Some("hello"));

    widget.handle_event(&pointer_with_modifiers(
        PointerPhase::Down,
        cell * 10.75,
        1.0,
        1,
        Modifiers::SHIFT,
    ));
    widget.handle_event(&pointer_with_modifiers(
        PointerPhase::Up,
        cell * 10.75,
        1.0,
        0,
        Modifiers::SHIFT,
    ));
    assert_eq!(widget.selected_text().as_deref(), Some("hello world"));

    widget.handle_event(&pointer(PointerPhase::Down, cell * 10.75, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 6.25, 1.0, 0));
    assert_eq!(widget.selected_text().as_deref(), Some("world"));

    widget.handle_event(&pointer_with_modifiers(
        PointerPhase::Down,
        cell * 0.25,
        1.0,
        1,
        Modifiers::SHIFT,
    ));
    widget.handle_event(&pointer_with_modifiers(
        PointerPhase::Up,
        cell * 0.25,
        1.0,
        0,
        Modifiers::SHIFT,
    ));
    assert_eq!(widget.selected_text().as_deref(), Some("hello world"));
}

#[test]
fn triple_click_selects_a_soft_wrapped_logical_line() {
    let mut widget = TerminalWidget::headless(5, 4);
    widget.feed(b"abcdefghij\r\nnext");
    let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
    let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;

    for _ in 0..3 {
        widget.handle_event(&pointer(PointerPhase::Down, x, y, 1));
        widget.handle_event(&pointer(PointerPhase::Up, x, y, 0));
    }

    assert_eq!(widget.selected_text().as_deref(), Some("abcdefghij\n"));
}

#[test]
fn reverse_drag_selection_preserves_visual_text_order() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"zero one two");
    let cell = f64::from(DEFAULT_CELL_WIDTH);

    widget.handle_event(&pointer(PointerPhase::Down, cell * 11.75, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 5.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 5.25, 1.0, 0));

    assert_eq!(widget.selected_text().as_deref(), Some("one two"));
}

#[test]
fn selection_dragged_past_horizontal_edges_includes_boundary_cells() {
    let mut widget = TerminalWidget::headless(20, 2);
    widget.feed(b"0123456789abcdefghij");
    let cell = f64::from(DEFAULT_CELL_WIDTH);

    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 25.0, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 25.0, 1.0, 0));
    assert_eq!(
        widget.selected_text().as_deref(),
        Some("0123456789abcdefghij")
    );

    widget.handle_event(&pointer(PointerPhase::Down, cell * 19.75, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, -cell * 5.0, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, -cell * 5.0, 1.0, 0));
    assert_eq!(
        widget.selected_text().as_deref(),
        Some("0123456789abcdefghij")
    );
}

#[test]
fn alt_drag_keeps_rectangular_selection_after_modifier_release() {
    let mut widget = TerminalWidget::headless(10, 3);
    widget.feed(b"abcdef\r\nuvwxyz");
    let cell = f64::from(DEFAULT_CELL_WIDTH);
    let line = f64::from(DEFAULT_LINE_HEIGHT);

    widget.handle_event(&pointer_with_modifiers(
        PointerPhase::Down,
        cell * 1.25,
        line * 0.25,
        1,
        Modifiers::ALT,
    ));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 3.75, line * 1.25, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 3.75, line * 1.25, 0));

    assert_eq!(widget.selected_text().as_deref(), Some("bcd\nvwx"));
}

#[test]
fn rectangular_selection_never_copies_half_of_a_wide_glyph() {
    let mut widget = TerminalWidget::headless(10, 3);
    widget.feed("a你bc\r\n12345".as_bytes());
    let cell = f64::from(DEFAULT_CELL_WIDTH);
    let line = f64::from(DEFAULT_LINE_HEIGHT);

    widget.handle_event(&pointer_with_modifiers(
        PointerPhase::Down,
        cell * 2.25,
        line * 0.25,
        1,
        Modifiers::ALT,
    ));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 3.75, line * 1.25, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 3.75, line * 1.25, 0));

    assert_eq!(widget.selected_text().as_deref(), Some("你b\n34"));
}

#[test]
fn drag_gesture_does_not_seed_the_double_click_streak() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let cell = f64::from(DEFAULT_CELL_WIDTH);

    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 4.75, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
    assert_eq!(widget.selected_text().as_deref(), Some("hello"));

    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 0.25, 1.0, 0));
    assert_ne!(widget.selected_text().as_deref(), Some("hello"));
}

#[test]
fn input_and_focus_loss_break_the_terminal_click_streak() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
    let click = |widget: &mut TerminalWidget| {
        widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, x, 1.0, 0));
    };

    click(&mut widget);
    widget.handle_event(&UiEvent::TextInput("x".into()));
    click(&mut widget);
    assert_eq!(
        widget.terminal.lock().selection.as_ref().unwrap().ty,
        SelectionType::Simple,
        "keyboard input must terminate a pending multi-click sequence"
    );

    widget.focus_changed(false);
    widget.focus_changed(true);
    click(&mut widget);
    assert_eq!(
        widget.terminal.lock().selection.as_ref().unwrap().ty,
        SelectionType::Simple,
        "window focus sessions must not share a multi-click sequence"
    );

    let mut secondary = match pointer(PointerPhase::Down, x, 1.0, 4) {
        UiEvent::Pointer(pointer) => pointer,
        _ => unreachable!(),
    };
    secondary.button = Some(PointerButton::Secondary);
    widget.handle_event(&UiEvent::Pointer(secondary));
    click(&mut widget);
    assert_eq!(
        widget.terminal.lock().selection.as_ref().unwrap().ty,
        SelectionType::Simple,
        "another pointer button must break the primary click sequence"
    );
}

#[test]
fn local_keyboard_shortcuts_break_the_terminal_click_streak() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
    let click = |widget: &mut TerminalWidget| {
        widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, x, 1.0, 0));
    };
    let key = |name: &str, modifiers| {
        UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers,
            repeat: false,
            synthetic: false,
        })
    };

    click(&mut widget);
    let copy_modifiers = if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL | Modifiers::SHIFT
    };
    widget.handle_event(&key("c", copy_modifiers));
    click(&mut widget);
    assert_eq!(
        widget.terminal.lock().selection.as_ref().unwrap().ty,
        SelectionType::Simple,
        "a local clipboard shortcut must separate click gestures"
    );

    click(&mut widget);
    widget.handle_event(&key("PageUp", Modifiers::SHIFT));
    click(&mut widget);
    assert_eq!(
        widget.terminal.lock().selection.as_ref().unwrap().ty,
        SelectionType::Simple,
        "a local scrollback shortcut must separate click gestures"
    );
}

#[test]
fn cancelled_terminal_click_does_not_seed_word_selection() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;

    widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
    let mut cancelled = match pointer(PointerPhase::Cancel, x, 1.0, 0) {
        UiEvent::Pointer(pointer) => pointer,
        _ => unreachable!(),
    };
    cancelled.button = None;
    widget.handle_event(&UiEvent::Pointer(cancelled));
    widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));

    assert_eq!(
        widget.terminal.lock().selection.as_ref().unwrap().ty,
        SelectionType::Simple
    );
}

#[test]
fn terminal_clipboard_shortcuts_do_not_consume_control_process_input() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"selected text");
    widget.begin_selection(1.0, 1.0, false);
    widget.update_selection(f64::from(DEFAULT_CELL_WIDTH) * 7.75, 1.0);
    let key = |name: &str, modifiers| {
        UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: format!("Key{}", name.to_ascii_uppercase()),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers,
            repeat: false,
            synthetic: false,
        })
    };

    if cfg!(target_os = "macos") {
        let copy = widget.handle_event(&key("c", Modifiers::META));
        assert_eq!(
            copy.clipboard_request(),
            Some(&ClipboardRequest::Write("selected".into()))
        );
        let paste = widget.handle_event(&key("v", Modifiers::META));
        assert_eq!(paste.clipboard_request(), Some(&ClipboardRequest::Read));
    } else {
        let interrupt = widget.handle_event(&key("c", Modifiers::CONTROL));
        assert!(interrupt.is_handled());
        assert_eq!(interrupt.clipboard_request(), None);
        assert_eq!(widget.take_input(), [0x03]);
        assert!(widget.selected_text().is_none());

        widget.begin_selection(1.0, 1.0, false);
        widget.update_selection(f64::from(DEFAULT_CELL_WIDTH) * 7.75, 1.0);
        let copy = widget.handle_event(&key("c", Modifiers::CONTROL | Modifiers::SHIFT));
        assert_eq!(
            copy.clipboard_request(),
            Some(&ClipboardRequest::Write("selected".into()))
        );
        assert!(widget.take_input().is_empty());

        let quoted_insert = widget.handle_event(&key("v", Modifiers::CONTROL));
        assert!(quoted_insert.is_handled());
        assert_eq!(quoted_insert.clipboard_request(), None);
        assert_eq!(widget.take_input(), [0x16]);
        let paste = widget.handle_event(&key("v", Modifiers::CONTROL | Modifiers::SHIFT));
        assert_eq!(paste.clipboard_request(), Some(&ClipboardRequest::Read));
    }
}

#[test]
fn terminal_select_all_covers_scrollback_and_publishes_a_line_selection() {
    let mut widget = TerminalWidget::headless(10, 3);
    widget.feed(b"one\r\ntwo\r\nthree\r\nfour\r\nfive");
    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL | Modifiers::SHIFT
    };
    let key = UiEvent::Key(KeyEvent {
        phase: KeyPhase::Down,
        key: "a".into(),
        key_without_modifiers: "a".into(),
        code: "KeyA".into(),
        text: None,
        text_with_all_modifiers: None,
        location: KeyLocation::Standard,
        modifiers,
        repeat: false,
        synthetic: false,
    });

    let result = widget.handle_event(&key);

    assert!(result.is_handled());
    assert!(widget.terminal.lock().history_size() > 0);
    assert_eq!(
        widget.selected_text().as_deref(),
        Some("one\ntwo\nthree\nfour\nfive\n")
    );
    let event = widget.take_node_event().expect("select-all event");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&event.json).unwrap(),
        serde_json::json!({
            "text": "one\ntwo\nthree\nfour\nfive\n",
            "kind": "line"
        })
    );

    if !cfg!(target_os = "macos") {
        let interrupt = UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "a".into(),
            key_without_modifiers: "a".into(),
            code: "KeyA".into(),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers: Modifiers::CONTROL,
            repeat: false,
            synthetic: false,
        });
        widget.handle_event(&interrupt);
        assert_eq!(widget.take_input(), [0x01]);
        assert!(widget.selected_text().is_none());
    }
}

#[test]
fn copy_shortcut_without_selection_never_reaches_the_pty() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[>3u");
    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL | Modifiers::SHIFT
    };
    let mut key = KeyEvent {
        phase: KeyPhase::Down,
        key: "c".into(),
        key_without_modifiers: "c".into(),
        code: "KeyC".into(),
        text: None,
        text_with_all_modifiers: None,
        location: KeyLocation::Standard,
        modifiers,
        repeat: false,
        synthetic: false,
    };
    let response = widget.handle_event(&UiEvent::Key(key.clone()));

    assert!(response.is_handled());
    assert_eq!(response.clipboard_request(), None);
    key.phase = KeyPhase::Up;
    assert!(widget.handle_event(&UiEvent::Key(key)).is_handled());
    assert!(widget.take_input().is_empty());
}

#[test]
fn selecting_the_trailing_cell_copies_the_complete_wide_glyph() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed("你a".as_bytes());
    let y = 1.0;
    let left = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
    let right = f64::from(DEFAULT_CELL_WIDTH) * 1.75;

    widget.handle_event(&pointer(PointerPhase::Down, left, y, 1));
    widget.handle_event(&pointer(PointerPhase::Up, right, y, 0));

    assert_eq!(widget.selected_text().as_deref(), Some("你"));
}

#[test]
fn dragging_selection_outside_viewport_scrolls_history() {
    let mut widget = TerminalWidget::headless(20, 2);
    widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
    assert_eq!(widget.terminal.lock().display_offset(), 0);

    widget.handle_event(&pointer(PointerPhase::Down, 1.0, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, 1.0, -5.0, 1));

    assert_eq!(widget.terminal.lock().display_offset(), 1);

    widget.next_selection_scroll = Some(Instant::now() - Duration::from_millis(1));
    let mut tcx = TextContext::new();
    widget.paint(
        DEFAULT_CELL_WIDTH * 20.0,
        DEFAULT_LINE_HEIGHT * 2.0,
        &mut tcx,
    );
    assert_eq!(widget.terminal.lock().display_offset(), 2);
}

#[test]
fn selection_autoscroll_accelerates_far_outside_the_viewport() {
    let mut widget = TerminalWidget::headless(20, 2);
    for index in 0..20 {
        widget.feed(format!("line {index}\r\n").as_bytes());
    }
    widget.handle_event(&pointer(PointerPhase::Down, 1.0, 1.0, 1));

    widget.handle_event(&pointer(
        PointerPhase::Move,
        1.0,
        -f64::from(DEFAULT_LINE_HEIGHT) * 4.5,
        1,
    ));

    assert_eq!(widget.terminal.lock().display_offset(), 5);
}

#[test]
fn selection_autoscroll_stops_at_scrollback_boundaries() {
    let mut widget = TerminalWidget::headless(20, 2);
    widget.feed(b"one\r\ntwo");
    widget.handle_event(&pointer(PointerPhase::Down, 1.0, 1.0, 1));

    widget.handle_event(&pointer(PointerPhase::Move, 1.0, -5.0, 1));
    assert_eq!(widget.terminal.lock().display_offset(), 0);
    assert!(widget.next_selection_scroll.is_none());

    widget.handle_event(&pointer(
        PointerPhase::Move,
        1.0,
        f64::from(DEFAULT_LINE_HEIGHT) * 3.0,
        1,
    ));
    assert_eq!(widget.terminal.lock().display_offset(), 0);
    assert!(widget.next_selection_scroll.is_none());
}

#[test]
fn smooth_wheel_deltas_accumulate_into_terminal_lines() {
    let mut widget = TerminalWidget::headless(20, 2);
    widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
    let wheel = |delta_y| {
        UiEvent::Wheel(wabou_shell::WheelEvent {
            position: wabou_shell::Point { x: 1.0, y: 1.0 },
            delta_x: 0.0,
            delta_y,
            delta_mode: wabou_shell::WheelDeltaMode::Pixel,
            phase: wabou_shell::GesturePhase::Changed,
            modifiers: Modifiers::default(),
        })
    };

    for _ in 0..3 {
        assert!(widget.handle_event(&wheel(10.0)).is_handled());
        assert_eq!(widget.terminal.lock().display_offset(), 0);
    }
    widget.handle_event(&wheel(10.0));
    assert_eq!(widget.terminal.lock().display_offset(), 1);

    // A new gesture in the opposite direction is responsive instead of
    // first consuming a stale same-direction remainder.
    widget.handle_event(&wheel(30.0));
    widget.handle_event(&wheel(-40.0));
    assert_eq!(widget.terminal.lock().display_offset(), 0);
}

#[test]
fn wheel_remainders_do_not_leak_between_terminal_input_owners() {
    let mut widget = TerminalWidget::headless(20, 2);
    widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
    let wheel = |delta_y| {
        UiEvent::Wheel(wabou_shell::WheelEvent {
            position: wabou_shell::Point { x: 1.0, y: 1.0 },
            delta_x: 0.0,
            delta_y,
            delta_mode: wabou_shell::WheelDeltaMode::Pixel,
            phase: wabou_shell::GesturePhase::Changed,
            modifiers: Modifiers::default(),
        })
    };

    widget.handle_event(&wheel(30.0));
    assert_eq!(widget.terminal.lock().display_offset(), 0);

    widget.feed(b"\x1b[?1000h\x1b[?1006h");
    widget.handle_event(&wheel(10.0));
    widget.handle_event(&wheel(20.0));
    assert!(widget.take_input().is_empty());

    widget.feed(b"\x1b[?1000l\x1b[?1006l");
    widget.handle_event(&wheel(10.0));
    assert_eq!(widget.terminal.lock().display_offset(), 0);
    widget.handle_event(&wheel(30.0));
    assert_eq!(widget.terminal.lock().display_offset(), 1);
    assert!(widget.take_input().is_empty());
}

#[test]
fn selection_tracks_content_as_new_output_scrolls_the_grid() {
    let mut widget = TerminalWidget::headless(20, 3);
    widget.feed(b"one\r\ntwo\r\nthree");
    let cell = f64::from(DEFAULT_CELL_WIDTH);
    let row = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, row, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 2.75, row, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 2.75, row, 0));
    assert_eq!(widget.selected_text().as_deref(), Some("two"));

    widget.feed(b"\r\nfour");

    assert_eq!(widget.selected_text().as_deref(), Some("two"));
    assert!(widget.visible_line(0).starts_with("two"));
}

#[test]
fn selection_survives_vertical_resize_without_reflow() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"one\r\ntwo\r\nthree");
    let cell = f64::from(DEFAULT_CELL_WIDTH);
    let row = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, row, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 2.75, row, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 2.75, row, 0));

    widget.resize(DEFAULT_CELL_WIDTH * 20.0, DEFAULT_LINE_HEIGHT * 2.0, 1.0);

    assert_eq!(widget.selected_text().as_deref(), Some("two"));
}

#[test]
fn scale_aware_paint_tracks_retina_density() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed("🚀".as_bytes());
    let mut tcx = TextContext::new();

    widget.paint_scaled(200.0, 80.0, 2.0, &mut tcx);

    assert_eq!(widget.size.pixel_width, 400);
    assert_eq!(widget.size.pixel_height, 160);
}

#[test]
fn cell_backgrounds_share_device_pixel_aligned_edges() {
    let scale = 1.5;
    let first = cell_fill_rect(0, 0, 8.3, 19.7, scale);
    let second = cell_fill_rect(1, 0, 8.3, 19.7, scale);
    let next_row = cell_fill_rect(0, 1, 8.3, 19.7, scale);

    assert_eq!(first.x1, second.x0);
    assert_eq!(first.y1, next_row.y0);
    for edge in [first.x0, first.y0, first.x1, first.y1] {
        assert_eq!(edge * scale, (edge * scale).round());
    }
}

#[test]
fn double_click_selects_semantic_word() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    let x = f64::from(DEFAULT_CELL_WIDTH) * 7.25;

    for _ in 0..2 {
        widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, x, 1.0, 0));
    }

    assert_eq!(widget.selected_text().as_deref(), Some("world"));
}

#[test]
fn sgr_mouse_mode_reports_to_pty_and_shift_bypasses_it() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[?1000h\x1b[?1006h\r\nabcdef");
    let x = f64::from(DEFAULT_CELL_WIDTH) * 2.25;
    let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;

    widget.handle_event(&pointer(PointerPhase::Down, x, y, 1));
    widget.handle_event(&pointer(PointerPhase::Up, x, y, 0));
    assert_eq!(widget.take_input(), b"\x1b[<0;3;2M\x1b[<0;3;2m");

    let mut shifted = match pointer(PointerPhase::Down, x, y, 1) {
        UiEvent::Pointer(pointer) => pointer,
        _ => unreachable!(),
    };
    shifted.modifiers.insert(Modifiers::SHIFT);
    widget.handle_event(&UiEvent::Pointer(shifted));
    assert!(widget.take_input().is_empty());
    assert!(widget.selecting);

    // Once local selection owns the gesture, releasing Shift must not
    // hand the remaining move/up back to the terminal application.
    widget.handle_event(&pointer(
        PointerPhase::Move,
        f64::from(DEFAULT_CELL_WIDTH) * 5.75,
        y,
        1,
    ));
    widget.handle_event(&pointer(
        PointerPhase::Up,
        f64::from(DEFAULT_CELL_WIDTH) * 5.75,
        y,
        0,
    ));
    assert!(widget.take_input().is_empty());
    assert!(!widget.selecting);
    assert_eq!(widget.selected_text().as_deref(), Some("cdef"));
}

#[test]
fn pointer_cancel_releases_remote_mouse_buttons() {
    let x = f64::from(DEFAULT_CELL_WIDTH) * 2.25;
    let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
    let mut cancelled = match pointer(PointerPhase::Cancel, x, y, 0) {
        UiEvent::Pointer(pointer) => pointer,
        _ => unreachable!(),
    };
    cancelled.button = None;

    let mut sgr = TerminalWidget::headless(20, 4);
    sgr.feed(b"\x1b[?1000h\x1b[?1006h");
    sgr.handle_event(&pointer(PointerPhase::Down, x, y, 1));
    sgr.handle_event(&UiEvent::Pointer(cancelled));
    assert_eq!(sgr.take_input(), b"\x1b[<0;3;2M\x1b[<0;3;2m");

    let mut legacy = TerminalWidget::headless(20, 4);
    legacy.feed(b"\x1b[?1000h");
    legacy.handle_event(&pointer(PointerPhase::Down, x, y, 1));
    legacy.handle_event(&UiEvent::Pointer(cancelled));
    assert_eq!(legacy.take_input(), b"\x1b[M #\"\x1b[M##\"");
}

#[test]
fn remote_mouse_gesture_keeps_ownership_and_button_identity() {
    let x = f64::from(DEFAULT_CELL_WIDTH) * 2.25;
    let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
    let mut down = match pointer(PointerPhase::Down, x, y, 4) {
        UiEvent::Pointer(pointer) => pointer,
        _ => unreachable!(),
    };
    down.button = Some(PointerButton::Secondary);
    let mut cancelled =
        match pointer_with_modifiers(PointerPhase::Cancel, x, y, 0, Modifiers::SHIFT) {
            UiEvent::Pointer(pointer) => pointer,
            _ => unreachable!(),
        };
    cancelled.button = None;

    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[?1000h\x1b[?1006h");
    widget.handle_event(&UiEvent::Pointer(down));
    assert_eq!(widget.remote_mouse_button, Some(PointerButton::Secondary));
    widget.handle_event(&UiEvent::Pointer(cancelled));

    assert_eq!(widget.take_input(), b"\x1b[<2;3;2M\x1b[<6;3;2m");
    assert_eq!(widget.remote_mouse_button, None);
    assert!(widget.terminal.lock().selection.is_none());
}

#[test]
fn focus_loss_finishes_an_in_progress_selection_gesture() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"abcdef");
    let cell = f64::from(DEFAULT_CELL_WIDTH);
    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 3.75, 1.0, 1));
    assert!(widget.selecting);
    assert!(widget.take_node_event().is_none());

    widget.focus_changed(false);

    assert!(!widget.selecting);
    assert!(widget.last_click.is_none());
    assert_eq!(widget.selected_text().as_deref(), Some("abcd"));
    let node_event = widget.take_node_event().expect("committed selection");
    assert_eq!(node_event.event_code, event::TERMINALSELECTIONCHANGE);
}

#[test]
fn active_selection_owns_wheel_input_in_mouse_reporting_mode() {
    let mut widget = TerminalWidget::headless(20, 2);
    widget.feed(b"one\r\ntwo\r\nthree\r\nfour\x1b[?1000h\x1b[?1006h");
    let position = wabou_shell::Point {
        x: f64::from(DEFAULT_CELL_WIDTH) * 3.75,
        y: f64::from(DEFAULT_LINE_HEIGHT) * 1.25,
    };
    widget.handle_event(&pointer_with_modifiers(
        PointerPhase::Down,
        position.x,
        position.y,
        1,
        Modifiers::SHIFT,
    ));
    assert!(widget.selecting);

    widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
        position,
        delta_x: 0.0,
        delta_y: 40.0,
        delta_mode: wabou_shell::WheelDeltaMode::Pixel,
        phase: wabou_shell::GesturePhase::Changed,
        modifiers: Modifiers::empty(),
    }));

    assert!(widget.terminal.lock().display_offset() > 0);
    assert!(widget.take_input().is_empty());
    assert!(widget.selecting);
    widget.handle_event(&pointer(PointerPhase::Up, position.x, position.y, 0));
    assert!(!widget.selecting);
    assert!(widget.selected_text().is_some());
}

#[test]
fn mouse_reporting_sends_wheel_buttons_at_pointer_position() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[?1000h\x1b[?1006h");
    let position = wabou_shell::Point {
        x: f64::from(DEFAULT_CELL_WIDTH) * 2.25,
        y: f64::from(DEFAULT_LINE_HEIGHT) * 1.25,
    };

    for delta_y in [-40.0, 40.0] {
        widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
            position,
            delta_x: 0.0,
            delta_y,
            delta_mode: wabou_shell::WheelDeltaMode::Pixel,
            phase: wabou_shell::GesturePhase::Changed,
            modifiers: Modifiers::default(),
        }));
    }

    assert_eq!(widget.take_input(), b"\x1b[<64;3;2M\x1b[<65;3;2M");
    widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
        position,
        delta_x: 0.0,
        delta_y: -40.0,
        delta_mode: wabou_shell::WheelDeltaMode::Pixel,
        phase: wabou_shell::GesturePhase::Changed,
        modifiers: Modifiers::ALT | Modifiers::CONTROL,
    }));
    assert_eq!(widget.take_input(), b"\x1b[<88;3;2M");
    assert_eq!(widget.terminal.lock().display_offset(), 0);
}

#[test]
fn utf8_mouse_encodes_large_coordinates_without_legacy_clamping() {
    let mut widget = TerminalWidget::headless(300, 4);
    widget.feed(b"\x1b[?1000h\x1b[?1005h");
    let x = f64::from(DEFAULT_CELL_WIDTH) * 99.25;

    widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));

    assert_eq!(
        widget.take_input(),
        [b"\x1b[M ".as_slice(), &[0xc2, 0x84, 33]].concat()
    );

    let mut legacy = TerminalWidget::headless(300, 4);
    legacy.feed(b"\x1b[?1000h");
    let outside_legacy_range = f64::from(DEFAULT_CELL_WIDTH) * 223.25;
    let result = legacy.handle_event(&pointer(PointerPhase::Down, outside_legacy_range, 1.0, 1));
    assert!(result.is_handled());
    assert!(legacy.take_input().is_empty());
    assert!(!legacy.selecting);
}

#[test]
fn focus_and_alternate_scroll_modes_report_to_pty() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[?1004h");
    widget.focus_changed(true);
    widget.focus_changed(false);
    assert_eq!(widget.take_input(), b"\x1b[I\x1b[O");

    widget.feed(b"\x1b[?1049h\x1b[?1007h");
    widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
        position: wabou_shell::Point { x: 0.0, y: 0.0 },
        delta_x: 0.0,
        delta_y: -48.0,
        delta_mode: wabou_shell::WheelDeltaMode::Pixel,
        phase: wabou_shell::GesturePhase::Changed,
        modifiers: Modifiers::default(),
    }));
    assert_eq!(widget.take_input(), b"\x1b[A");
}
