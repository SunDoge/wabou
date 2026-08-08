use super::*;

use wabou_shell::{ClipboardRequest, KeyEvent, KeyLocation, Modifiers, Point, PointerEvent};

fn pointer(phase: PointerPhase, x: f64, y: f64, buttons: u32) -> UiEvent {
    pointer_with_modifiers(phase, x, y, buttons, Modifiers::default())
}

fn pointer_with_modifiers(
    phase: PointerPhase,
    x: f64,
    y: f64,
    buttons: u32,
    modifiers: Modifiers,
) -> UiEvent {
    UiEvent::Pointer(PointerEvent {
        phase,
        position: Point { x, y },
        button: Some(PointerButton::Primary),
        buttons,
        modifiers,
    })
}

#[test]
fn rio_parser_preserves_text_and_sgr_cells() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[31mred\x1b[0m plain\r\n\x1b[3;4;9mstyled\x1b[0m");
    assert!(widget.visible_line(0).starts_with("red plain"));

    let terminal = widget.terminal.lock();
    let rows = terminal.visible_rows();
    let style = terminal.grid.style_set.get(rows[0][Column(0)].style_id());
    assert_eq!(style.fg, AnsiColor::Named(NamedColor::Red));
    let decorated = terminal.grid.style_set.get(rows[1][Column(0)].style_id());
    assert!(decorated.flags.contains(StyleFlags::ITALIC));
    assert!(decorated.flags.contains(StyleFlags::UNDERLINE));
    assert!(decorated.flags.contains(StyleFlags::STRIKEOUT));
}

#[test]
fn renderer_preserves_combining_grapheme_extras() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed("e\u{301}".as_bytes());

    let terminal = widget.terminal.lock();
    let rows = terminal.visible_rows();
    let square = rows[0][Column(0)];
    assert!(square.has_extras());
    assert_eq!(
        cell_text(
            square,
            square
                .extras_id()
                .and_then(|id| terminal.grid.extras_table.get(id))
        ),
        "e\u{301}"
    );
}

#[test]
fn primary_click_on_osc8_link_opens_only_after_a_committed_click() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b]8;;https://example.com/docs\x1b\\link\x1b]8;;\x1b\\");

    let down = pointer_with_modifiers(PointerPhase::Down, 1.0, 1.0, 1, Modifiers::CONTROL);
    assert_eq!(widget.handle_event(&down), WidgetEventResult::HANDLED);
    assert_eq!(widget.take_host_action(), None);
    let up = pointer_with_modifiers(PointerPhase::Up, 1.0, 1.0, 0, Modifiers::CONTROL);
    assert_eq!(widget.handle_event(&up), WidgetEventResult::HANDLED);
    assert_eq!(
        widget.take_host_action(),
        Some(HostAction::OpenUrl("https://example.com/docs".into()))
    );
}

#[test]
fn dragging_or_cancelling_an_osc8_link_does_not_open_it() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b]8;;https://example.com/docs\x1b\\link\x1b]8;;\x1b\\");

    for terminal_phase in [PointerPhase::Up, PointerPhase::Cancel] {
        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Down,
            1.0,
            1.0,
            1,
            Modifiers::CONTROL,
        ));
        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Move,
            20.0,
            1.0,
            1,
            Modifiers::CONTROL,
        ));
        widget.handle_event(&pointer_with_modifiers(
            terminal_phase,
            20.0,
            1.0,
            0,
            Modifiers::CONTROL,
        ));
        assert_eq!(widget.take_host_action(), None);
        assert!(widget.pending_hyperlink.is_none());
    }
}

#[test]
fn osc8_links_resolve_from_wide_spacers_and_scrollback_rows() {
    let mut wide = TerminalWidget::headless(20, 4);
    wide.feed("\x1b]8;;https://example.com/wide\x1b\\你\x1b]8;;\x1b\\".as_bytes());
    assert_eq!(
        wide.hyperlink_at(f64::from(DEFAULT_CELL_WIDTH) * 1.5, 1.0),
        Some("https://example.com/wide".into())
    );

    let mut history = TerminalWidget::headless(20, 2);
    history.feed(b"\x1b]8;;https://example.com/history\x1b\\old\x1b]8;;\x1b\\\r\nnew\r\nlatest");
    history.terminal.lock().scroll_display(Scroll::Top);
    assert!(history.terminal.lock().display_offset() > 0);
    assert_eq!(
        history.hyperlink_at(1.0, 1.0),
        Some("https://example.com/history".into())
    );
}

#[test]
fn selection_highlight_covers_both_cells_of_wide_glyphs() {
    let row = Line(0);
    let lead_point = Pos::new(row, Column(3));
    let spacer_point = Pos::new(row, Column(4));
    let mut lead = Square::default();
    lead.set_wide(Wide::Wide);
    let mut spacer = Square::default();
    spacer.set_wide(Wide::Spacer);

    let trailing_half = SelectionRange::new(spacer_point, spacer_point, false);
    assert!(selection_contains_square(trailing_half, lead_point, lead));
    assert!(selection_contains_square(
        trailing_half,
        spacer_point,
        spacer
    ));

    let leading_half = SelectionRange::new(lead_point, lead_point, false);
    assert!(selection_contains_square(leading_half, lead_point, lead));
    assert!(selection_contains_square(
        leading_half,
        spacer_point,
        spacer
    ));
}

#[test]
fn wide_spacer_cells_keep_background_and_decorations() {
    let scene_paths = |bytes: &[u8]| {
        let mut widget = TerminalWidget::headless(10, 2);
        widget.feed(bytes);
        let mut tcx = TextContext::new();
        widget
            .paint(
                DEFAULT_CELL_WIDTH * 10.0,
                DEFAULT_LINE_HEIGHT * 2.0,
                &mut tcx,
            )
            .encoding()
            .n_paths
    };

    let narrow = scene_paths(b"\x1b[41;4mA");
    let wide = scene_paths("\x1b[41;4m你".as_bytes());
    assert!(
        wide >= narrow + 2,
        "the wide spacer must contribute its background and underline"
    );
}

#[test]
fn font_metrics_drive_cell_width_and_grid_resize() {
    let mut widget = TerminalWidget::headless(80, 24);
    widget.attribute_changed("font-size", "20px");
    assert!(widget.metrics_dirty);
    let mut tcx = TextContext::new();
    let family: Arc<str> = Arc::from("monospace");
    let expected = layout_text_styled(
        &mut tcx,
        Arc::from("0"),
        20.0,
        400.0,
        None,
        Default::default(),
        [255, 255, 255, 255],
        Arc::from([]),
        Some(&family),
        None,
    )
    .width();

    widget.paint(400.0, 200.0, &mut tcx);

    assert!((widget.cell_width - expected).abs() < 0.001);
    assert_eq!(widget.size.columns, (400.0 / expected).floor() as usize);
    assert!(!widget.metrics_dirty);

    widget.attribute_changed("line-height", "32px");
    widget.attribute_changed("font-size", "22px");
    widget.paint(400.0, 200.0, &mut tcx);
    assert_eq!(widget.line_height, 32.0);
    widget.attribute_removed("line-height");
    widget.paint(400.0, 200.0, &mut tcx);
    assert_ne!(widget.line_height, 32.0);

    widget.attribute_changed("line-height", "12px");
    widget.attribute_changed("font-size", "30px");
    widget.paint(400.0, 200.0, &mut tcx);
    assert_eq!(widget.line_height, 30.0);
    widget.attribute_changed("font-size", "10px");
    widget.paint(400.0, 200.0, &mut tcx);
    assert_eq!(
        widget.line_height, 12.0,
        "the authored line height must survive temporary font-size clamping"
    );
}

#[test]
fn terminal_selection_colors_are_themeable_and_removable() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.attribute_changed("selection-background", "#11223380");
    widget.attribute_changed("selection-foreground", "rgb(240 241 242)");
    assert_eq!(
        widget.selection_background,
        Color::from_rgba8(0x11, 0x22, 0x33, 0x80)
    );
    assert_eq!(
        widget.selection_foreground,
        Some(Color::from_rgb8(240, 241, 242))
    );

    widget.attribute_changed("selection-background", "not-a-color");
    widget.attribute_changed("selection-foreground", "not-a-color");
    assert_eq!(
        widget.selection_background,
        Color::from_rgba8(0x11, 0x22, 0x33, 0x80)
    );
    assert_eq!(
        widget.selection_foreground,
        Some(Color::from_rgb8(240, 241, 242))
    );

    widget.attribute_removed("selection-background");
    widget.attribute_removed("selection-foreground");
    assert_eq!(widget.selection_background, DEFAULT_SELECTION_BACKGROUND);
    assert_eq!(widget.selection_foreground, None);
}

#[test]
fn sgr_underline_color_does_not_recolor_strikeout() {
    let foreground = Color::from_rgb8(10, 20, 30);
    let underline = ColorRgb {
        r: 200,
        g: 40,
        b: 50,
    };
    let style = Style {
        underline_color: Some(AnsiColor::Spec(underline)),
        flags: StyleFlags::STRIKEOUT | StyleFlags::UNDERLINE,
        ..Style::default()
    };
    let colors = TermColors::default();
    let (strike_color, underline_color) = decoration_colors(
        style,
        foreground,
        &colors,
        named_color(NamedColor::Foreground, true),
        named_color(NamedColor::Background, false),
        None,
    );

    assert_eq!(strike_color, foreground);
    assert_eq!(underline_color, Color::from_rgb8(200, 40, 50));

    let selected = Color::from_rgb8(240, 241, 242);
    assert_eq!(
        decoration_colors(
            style,
            foreground,
            &colors,
            named_color(NamedColor::Foreground, true),
            named_color(NamedColor::Background, false),
            Some(selected),
        ),
        (selected, selected)
    );
}

#[test]
fn headless_input_uses_terminal_escape_sequences() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.handle_event(&UiEvent::Key(KeyEvent {
        phase: KeyPhase::Down,
        key: "ArrowUp".into(),
        key_without_modifiers: "ArrowUp".into(),
        code: "ArrowUp".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Modifiers::default(),
        repeat: false,
    }));
    widget.handle_event(&UiEvent::TextInput("λ".into()));
    assert_eq!(widget.take_input(), b"\x1b[A\xce\xbb");
}

#[test]
fn key_encoding_honors_terminal_modes_and_modifiers() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[?1h");
    let key = |name: &str, modifiers: Modifiers| {
        UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers,
            repeat: false,
        })
    };

    widget.handle_event(&key("ArrowUp", Modifiers::default()));
    widget.handle_event(&key("ArrowRight", Modifiers::SHIFT));
    let alt = widget.handle_event(&key("x", Modifiers::ALT));

    assert!(alt.consumes_key_text());
    assert_eq!(widget.take_input(), b"\x1bOA\x1b[1;2C\x1bx");
}

#[test]
fn legacy_keyboard_encodes_extended_function_keys() {
    let mut widget = TerminalWidget::headless(20, 4);
    let key = |name: &str, modifiers: Modifiers| {
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
        })
    };

    for (name, sequence) in [
        ("F13", "\x1b[1;2P"),
        ("F14", "\x1b[1;2Q"),
        ("F15", "\x1b[1;2R"),
        ("F16", "\x1b[1;2S"),
        ("F17", "\x1b[15;2~"),
        ("F18", "\x1b[17;2~"),
        ("F19", "\x1b[18;2~"),
        ("F20", "\x1b[19;2~"),
        ("F21", "\x1b[20;2~"),
        ("F22", "\x1b[21;2~"),
        ("F23", "\x1b[23;2~"),
        ("F24", "\x1b[24;2~"),
        ("F25", "\x1b[1;5P"),
        ("F26", "\x1b[1;5Q"),
        ("F27", "\x1b[1;5R"),
        ("F28", "\x1b[1;5S"),
        ("F29", "\x1b[15;5~"),
        ("F30", "\x1b[17;5~"),
        ("F31", "\x1b[18;5~"),
        ("F32", "\x1b[19;5~"),
        ("F33", "\x1b[20;5~"),
        ("F34", "\x1b[21;5~"),
        ("F35", "\x1b[23;5~"),
    ] {
        let result = widget.handle_event(&key(name, Modifiers::empty()));
        assert!(result.consumes_key_text());
        assert_eq!(widget.take_input(), sequence.as_bytes(), "{name}");
    }

    widget.handle_event(&key("F15", Modifiers::SHIFT | Modifiers::CONTROL));
    assert_eq!(widget.take_input(), b"\x1b[1;6R");
    widget.handle_event(&key("F27", Modifiers::ALT));
    assert_eq!(widget.take_input(), b"\x1b[1;7R");

    widget.feed(b"\x1b[>8u");
    widget.handle_event(&key("F35", Modifiers::empty()));
    assert_eq!(
        widget.take_input(),
        b"\x1b[57398u",
        "Kitty mode must keep its dedicated F35 keycode"
    );
}

#[test]
fn application_keypad_mode_encodes_physical_numpad_keys() {
    let mut widget = TerminalWidget::headless(20, 4);
    let key = |logical: &str, code: &str| {
        UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: logical.into(),
            key_without_modifiers: logical.into(),
            code: format!("Code({code})"),
            text: Some(logical.into()),
            text_with_all_modifiers: Some(logical.into()),
            location: KeyLocation::Numpad,
            modifiers: Modifiers::empty(),
            repeat: false,
        })
    };

    let numeric = widget.handle_event(&key("1", "Numpad1"));
    assert!(!numeric.is_handled());
    assert!(widget.take_input().is_empty());

    widget.feed(b"\x1b=");
    for (logical, code, sequence) in [
        ("0", "Numpad0", "\x1bOp"),
        ("1", "Numpad1", "\x1bOq"),
        ("5", "Numpad5", "\x1bOu"),
        ("9", "Numpad9", "\x1bOy"),
        (".", "NumpadDecimal", "\x1bOn"),
        (",", "NumpadComma", "\x1bOl"),
        ("+", "NumpadAdd", "\x1bOk"),
        ("-", "NumpadSubtract", "\x1bOm"),
        ("*", "NumpadMultiply", "\x1bOj"),
        ("/", "NumpadDivide", "\x1bOo"),
        ("Enter", "NumpadEnter", "\x1bOM"),
        ("=", "NumpadEqual", "\x1bOX"),
    ] {
        let result = widget.handle_event(&key(logical, code));
        assert!(result.consumes_key_text(), "{code}");
        assert_eq!(widget.take_input(), sequence.as_bytes(), "{code}");
    }

    widget.feed(b"\x1b>");
    assert!(!widget.handle_event(&key("1", "Numpad1")).is_handled());
    assert!(widget.take_input().is_empty());
}

#[test]
fn negotiated_kitty_keyboard_reports_press_and_release_without_double_text() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[>31u");
    let mut key = KeyEvent {
        phase: KeyPhase::Down,
        key: "x".into(),
        key_without_modifiers: "x".into(),
        code: "Code(KeyX)".into(),
        text: Some("x".into()),
        text_with_all_modifiers: Some("x".into()),
        location: KeyLocation::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
    };

    let pressed = widget.handle_event(&UiEvent::Key(key.clone()));
    assert!(pressed.is_handled());
    assert!(pressed.consumes_key_text());
    key.phase = KeyPhase::Up;
    let released = widget.handle_event(&UiEvent::Key(key));
    assert!(released.is_handled());
    assert_eq!(widget.take_input(), b"\x1b[120;1;120u\x1b[120;1:3u");
}

#[test]
fn kitty_disambiguation_does_not_repeat_key_on_release() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[>1u");
    let mut key = KeyEvent {
        phase: KeyPhase::Down,
        key: "Backspace".into(),
        key_without_modifiers: "Backspace".into(),
        code: "Backspace".into(),
        text: None,
        text_with_all_modifiers: None,
        location: KeyLocation::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
    };

    assert!(widget.handle_event(&UiEvent::Key(key.clone())).is_handled());
    key.phase = KeyPhase::Up;
    assert!(!widget.handle_event(&UiEvent::Key(key)).is_handled());
    assert_eq!(widget.take_input(), b"\x1b[127u");
}

#[test]
fn legacy_control_keys_cover_ascii_control_range() {
    let mut widget = TerminalWidget::headless(20, 4);
    let key = |name: &str, modifiers: Modifiers| {
        UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers,
            repeat: false,
        })
    };

    for name in ["@", "a", "[", "\\", "]", "^", "_", "?", "3", "8"] {
        widget.handle_event(&key(name, Modifiers::CONTROL));
    }
    widget.handle_event(&key("a", Modifiers::CONTROL | Modifiers::ALT));

    assert_eq!(
        widget.take_input(),
        [
            0x00, 0x01, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x7f, 0x1b, 0x7f, 0x1b, 0x01
        ]
    );

    let alt_graph = widget.handle_event(&UiEvent::Key(KeyEvent {
        phase: KeyPhase::Down,
        key: "@".into(),
        key_without_modifiers: "q".into(),
        code: "KeyQ".into(),
        text: Some("@".into()),
        text_with_all_modifiers: Some("@".into()),
        location: Default::default(),
        modifiers: Modifiers::CONTROL | Modifiers::ALT,
        repeat: false,
    }));
    assert!(!alt_graph.is_handled());
    assert!(widget.take_input().is_empty());
    widget.handle_event(&UiEvent::TextInput("@".into()));
    assert_eq!(widget.take_input(), b"@");
}

#[test]
fn scrollback_shortcuts_are_local_and_typing_returns_to_bottom() {
    let mut widget = TerminalWidget::headless(20, 2);
    widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
    let key = |name: &str, modifiers: Modifiers| {
        UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers,
            repeat: false,
        })
    };

    widget.handle_event(&key("PageUp", Modifiers::SHIFT));
    assert!(widget.terminal.lock().display_offset() > 0);
    assert!(widget.take_input().is_empty());
    widget.handle_event(&key("Home", Modifiers::SHIFT));
    let top = widget.terminal.lock().display_offset();
    assert!(top >= 2);

    widget.begin_selection(1.0, 1.0, false);
    widget.update_selection(f64::from(DEFAULT_CELL_WIDTH) * 2.75, 1.0);
    assert!(widget.selected_text().is_some());
    let ignored = widget.handle_event(&key("Unidentified", Modifiers::default()));
    assert!(!ignored.is_handled());
    assert!(widget.selected_text().is_some());
    assert_eq!(widget.terminal.lock().display_offset(), top);

    widget.handle_event(&key("ArrowUp", Modifiers::default()));
    assert_eq!(widget.take_input(), b"\x1b[A");
    assert_eq!(widget.terminal.lock().display_offset(), 0);
    assert!(widget.selected_text().is_none());

    widget.feed(b"\x1b[?1049h");
    widget.handle_event(&key("PageUp", Modifiers::SHIFT));
    assert_eq!(widget.take_input(), b"\x1b[5;2~");
}

#[test]
fn paste_honors_bracketed_paste_mode() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b[?2004h");

    widget.handle_event(&UiEvent::Paste(
        "hello\n世界\tworld\x1b[201~unsafe\x03\0\u{7f}still pasted".into(),
    ));

    assert_eq!(
        widget.take_input(),
        "\x1b[200~hello\n世界\tworld[201~unsafestill pasted\x1b[201~".as_bytes()
    );
}

#[test]
fn plain_paste_converts_line_breaks_to_enter_keys() {
    let mut widget = TerminalWidget::headless(20, 4);

    widget.handle_event(&UiEvent::Paste(
        "one\ntwo\r\n三\tthree\rfour\x1b[31m\x03\0\u{7f}".into(),
    ));

    assert_eq!(
        widget.take_input(),
        "one\rtwo\r三\tthree\rfour[31m".as_bytes()
    );
}

#[test]
fn resizing_updates_rio_grid() {
    let mut widget = TerminalWidget::headless(80, 24);
    widget.resize(DEFAULT_CELL_WIDTH * 40.0, DEFAULT_LINE_HEIGHT * 10.0);
    let terminal = widget.terminal.lock();
    assert_eq!((terminal.columns(), terminal.screen_lines()), (40, 10));
}

#[test]
fn lazy_terminal_collects_initial_process_attributes_before_launch() {
    let mut widget = TerminalWidget::lazy_default_shell();
    assert!(!widget.launch_started);
    assert!(widget.pty_send.is_none());

    widget.attribute_changed("command", "ssh");
    widget.attribute_changed("args", r#"["-t","example.com"]"#);
    widget.attribute_changed("cwd", "/tmp/project");

    assert_eq!(
        widget.launch,
        Some(LaunchConfig {
            command: "ssh".into(),
            args: vec!["-t".into(), "example.com".into()],
            cwd: Some("/tmp/project".into()),
            login_shell: false,
        })
    );
    assert!(!widget.launch_started);
    assert!(widget.spawn_error.is_none());
}

#[test]
fn pty_spawn_injects_terminal_capabilities_without_mutating_parent_environment() {
    let launch = LaunchConfig::default_shell();
    let (command, args) = pty_spawn_parts(&launch);

    #[cfg(not(windows))]
    {
        assert_eq!(command, "/usr/bin/env");
        assert!(args.iter().any(|arg| arg == "TERM=xterm-256color"));
        assert!(args.iter().any(|arg| arg == "COLORTERM=truecolor"));
        assert!(args.iter().any(|arg| arg == "TERM_PROGRAM=wabou"));
        assert!(
            args.iter()
                .any(|arg| arg == concat!("TERM_PROGRAM_VERSION=", env!("CARGO_PKG_VERSION")))
        );
        assert!(args.iter().any(|arg| arg == &launch.command));
        #[cfg(target_os = "macos")]
        assert_eq!(args.last().map(String::as_str), Some("-l"));
    }
    #[cfg(windows)]
    {
        assert_eq!(command, quote_windows_command_arg(&launch.command));
        assert_eq!(
            args,
            launch
                .args
                .iter()
                .map(|arg| quote_windows_command_arg(arg))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn windows_command_line_quoting_preserves_argument_boundaries() {
    assert_eq!(quote_windows_command_arg("plain"), "plain");
    assert_eq!(quote_windows_command_arg(""), r#""""#);
    assert_eq!(
        quote_windows_command_arg(r"C:\Program Files\tool.exe"),
        r#""C:\Program Files\tool.exe""#
    );
    assert_eq!(
        quote_windows_command_arg(r#"say "hello""#),
        r#""say \"hello\"""#
    );
    assert_eq!(
        quote_windows_command_arg("trailing slash \\"),
        "\"trailing slash \\\\\""
    );
}

#[cfg(unix)]
#[test]
fn child_reaper_waits_for_and_collects_exited_processes() {
    let child = std::process::Command::new("/bin/sh")
        .args(["-c", "exit 0"])
        .spawn()
        .unwrap();
    let pid = child.id() as libc::pid_t;
    // Dropping std::process::Child does not wait; ownership is handed to
    // the same reaper used after Rio drops a terminal PTY.
    drop(child);

    spawn_child_reaper(pid).unwrap().join().unwrap().unwrap();
    let mut status = 0;
    assert_eq!(
        unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
        -1
    );
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ECHILD)
    );
}

#[cfg(unix)]
#[test]
fn child_reaper_escalates_for_terminal_jobs_that_ignore_hangup() {
    use std::io::BufRead;
    use std::os::unix::process::CommandExt;

    let mut command = std::process::Command::new("/bin/sh");
    command
        .args(["-c", "trap '' HUP; echo ready; exec sleep 30"])
        .stdout(std::process::Stdio::piped());
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    let mut child = command.spawn().unwrap();
    let pid = child.id() as libc::pid_t;
    let mut ready = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut ready)
        .unwrap();
    assert_eq!(ready, "ready\n");
    drop(child);

    let started = Instant::now();
    spawn_child_reaper(pid).unwrap().join().unwrap().unwrap();
    assert!(started.elapsed() >= Duration::from_millis(400));
    assert!(started.elapsed() < Duration::from_secs(3));
    let mut status = 0;
    assert_eq!(
        unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
        -1
    );
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ECHILD)
    );
}

#[cfg(unix)]
#[test]
fn unmount_shuts_down_and_reaps_the_terminal_process() {
    let mut widget =
        TerminalWidget::spawn("/bin/sh", vec!["-c".into(), "exec sleep 30".into()], None);
    let pid = widget.child_pid.expect("spawned terminal child");

    widget.unmount();

    assert!(widget.pty_send.is_none());
    assert!(widget.child_pid.is_none());
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let result = unsafe { libc::kill(pid, 0) };
        if result == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "terminal child {pid} was not reaped"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn removing_unlaunched_process_attributes_restores_launch_defaults() {
    let mut widget = TerminalWidget::lazy_default_shell();
    widget.attribute_changed("command", "ssh");
    widget.attribute_changed("args", r#"["example.com"]"#);
    widget.attribute_changed("cwd", "/tmp/project");

    widget.attribute_removed("command");
    widget.attribute_removed("args");
    widget.attribute_removed("cwd");

    assert_eq!(widget.launch, Some(LaunchConfig::default_shell()));
    assert!(!widget.launch_started);
    assert!(widget.spawn_error.is_none());
}

#[test]
fn invalid_lazy_terminal_args_block_launch_until_corrected() {
    let mut widget = TerminalWidget::lazy_default_shell();
    widget.attribute_changed("args", "not-json");
    widget.ensure_launched();
    assert!(!widget.launch_started);
    assert!(
        widget
            .spawn_error
            .as_deref()
            .is_some_and(|error| { error.starts_with("invalid terminal args JSON:") })
    );

    widget.attribute_changed("args", r#"["-l"]"#);
    assert!(widget.launch_error.is_none());
    assert!(widget.spawn_error.is_none());
    assert_eq!(widget.launch.as_ref().unwrap().args, ["-l"]);
}

#[test]
fn failed_terminal_spawn_can_be_reconfigured_without_retrying_each_frame() {
    let mut widget = TerminalWidget::lazy_default_shell();
    widget.attribute_changed("command", "/wabou/definitely/missing-terminal-command");
    widget.ensure_launched();

    assert!(widget.launch_started);
    assert!(widget.spawn_error.is_some());
    widget.ensure_launched();
    assert!(widget.launch_started, "a failed launch remains attempted");

    widget.attribute_changed("command", "/bin/sh");
    assert!(!widget.launch_started);
    assert!(widget.spawn_error.is_none());
    assert_eq!(widget.launch.as_ref().unwrap().command, "/bin/sh");
}

#[test]
fn terminal_reports_physical_text_area_and_cell_dimensions() {
    let mut widget = TerminalWidget::headless(40, 10);
    widget.device_scale = 2.0;
    widget.resize(340.0, 180.0);

    // 340 logical pixels still fit 40 columns, but the pixel width must
    // update independently from the grid and be reported in device pixels.
    assert_eq!((widget.size.columns, widget.size.rows), (40, 10));
    widget.feed(b"\x1b[14t\x1b[16t");
    assert!(widget.poll_async());

    assert_eq!(widget.take_input(), b"\x1b[4;360;680t\x1b[6;36;17t");
}

#[test]
fn terminal_size_saturates_pty_fields_without_losing_rio_dimensions() {
    let size = TerminalSize::from_viewport(
        40_000.0,
        20_000.0,
        DEFAULT_CELL_WIDTH,
        DEFAULT_LINE_HEIGHT,
        4.0,
    );

    assert_eq!((size.pixel_width, size.pixel_height), (160_000, 80_000));
    assert_eq!(
        (size.winsize().width, size.winsize().height),
        (u16::MAX, u16::MAX)
    );
}

#[test]
fn parser_damage_wakes_the_host_once_and_is_drained() {
    let mut widget = TerminalWidget::headless(20, 4);
    let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let wake_count = wakes.clone();
    widget.set_wake_callback(Arc::new(move || {
        wake_count.fetch_add(1, Ordering::Relaxed);
    }));

    widget.feed(b"output");
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
    assert!(widget.poll_async());
    assert!(!widget.poll_async());
}

#[test]
fn paint_consumes_damage_and_reuses_visible_row_storage() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"first");
    assert!(widget.terminal.lock().peek_damage_event().is_some());
    let mut tcx = TextContext::new();

    widget.paint(
        DEFAULT_CELL_WIDTH * 20.0,
        DEFAULT_LINE_HEIGHT * 4.0,
        &mut tcx,
    );

    assert!(widget.terminal.lock().peek_damage_event().is_none());
    let outer_capacity = widget.visible_rows.capacity();
    let row_buffers = widget
        .visible_rows
        .iter()
        .map(|row| row.inner.as_ptr())
        .collect::<Vec<_>>();

    widget.feed(b" second");
    widget.paint(
        DEFAULT_CELL_WIDTH * 20.0,
        DEFAULT_LINE_HEIGHT * 4.0,
        &mut tcx,
    );

    assert_eq!(widget.visible_rows.capacity(), outer_capacity);
    assert_eq!(
        widget
            .visible_rows
            .iter()
            .map(|row| row.inner.as_ptr())
            .collect::<Vec<_>>(),
        row_buffers
    );
    assert!(widget.visible_line(0).starts_with("first second"));
    assert!(widget.terminal.lock().peek_damage_event().is_none());
}

#[test]
fn kitty_graphic_updates_flow_from_vt_parser_into_renderer_cache() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b_Ga=T,f=32,s=1,v=1,i=42,p=9;/wAA/w==\x1b\\");

    assert!(widget.poll_async());
    assert!(widget.graphics.contains(rio_graphics::kitty_image_key(42)));
    let terminal = widget.terminal.lock();
    assert!(terminal.graphics.kitty_placements.contains_key(&(42, 9)));
}

#[test]
fn osc_titles_are_node_scoped_while_clipboard_store_remains_a_host_action() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b]0;project shell\x07\x1b]52;c;aGVsbG8=\x07");

    assert!(widget.poll_async());
    assert_eq!(
        widget.take_host_action(),
        Some(HostAction::SetClipboard("hello".into()))
    );
    assert_eq!(widget.take_host_action(), None);
    let title = widget.take_node_event().expect("title node event");
    assert_eq!(title.event_code, event::TERMINALTITLECHANGE);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&title.json).unwrap(),
        serde_json::json!({ "title": "project shell", "subtitle": null })
    );

    let mut synced = TerminalWidget::headless(20, 4);
    synced.attribute_changed("sync-window-title", "true");
    synced.feed(b"\x1b]0;synced shell\x07");
    synced.poll_async();
    assert_eq!(
        synced.take_host_action(),
        Some(HostAction::SetWindowTitle(Some("synced shell".into())))
    );
    synced.attribute_removed("sync-window-title");
    assert!(!synced.sync_window_title);
    assert_eq!(
        synced.take_host_action(),
        Some(HostAction::SetWindowTitle(None))
    );
    synced.attribute_changed("sync-window-title", "true");
    synced.unmount();
    assert_eq!(
        synced.take_host_action(),
        Some(HostAction::SetWindowTitle(None))
    );
}

#[test]
fn terminal_subtitle_and_title_reset_remain_scoped_to_the_widget_node() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.listener.send_event(
        RioEvent::TitleWithSubtitle("editor".into(), "README.md".into()),
        WindowId::from(0),
    );
    widget
        .listener
        .send_event(RioEvent::ResetTitle, WindowId::from(0));

    widget.poll_async();
    assert_eq!(widget.take_host_action(), None);
    let titled = widget.take_node_event().expect("title event");
    assert_eq!(titled.event_code, event::TERMINALTITLECHANGE);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&titled.json).unwrap(),
        serde_json::json!({ "title": "editor", "subtitle": "README.md" })
    );
    let reset = widget.take_node_event().expect("reset event");
    assert_eq!(reset.event_code, event::TERMINALTITLECHANGE);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&reset.json).unwrap(),
        serde_json::json!({ "title": null, "subtitle": null })
    );
}

#[test]
fn osc_current_directory_is_scoped_and_deduplicated() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"\x1b]7;file://localhost/tmp/project%20dir\x07");
    widget.poll_async();

    let cwd = widget.take_node_event().expect("cwd event");
    assert_eq!(cwd.event_code, event::TERMINALCWDCHANGE);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&cwd.json).unwrap(),
        serde_json::json!({ "path": "/tmp/project dir" })
    );

    widget.feed(b"\x1b]7;file://localhost/tmp/project%20dir\x07");
    widget.poll_async();
    assert_eq!(widget.take_node_event(), None);

    widget.feed(b"\x1b]7;file://localhost/tmp/next\x07");
    widget.poll_async();
    let next = widget.take_node_event().expect("changed cwd event");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&next.json).unwrap(),
        serde_json::json!({ "path": "/tmp/next" })
    );
}

#[test]
fn terminal_lifecycle_reports_are_exposed_as_node_events() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.listener.send_event(
        RioEvent::ProgressReport(rio_vt::event::ProgressReport {
            state: ProgressState::Set,
            progress: Some(42),
        }),
        WindowId::from(0),
    );
    widget.listener.send_event(
        RioEvent::DesktopNotification {
            title: "Build \"done\"".into(),
            body: "All tests passed".into(),
        },
        WindowId::from(0),
    );
    widget
        .listener
        .send_event(RioEvent::CloseTerminal(0), WindowId::from(0));
    widget
        .listener
        .send_event(RioEvent::Exit, WindowId::from(0));
    widget
        .listener
        .send_event(RioEvent::Quit, WindowId::from(0));

    widget.poll_async();
    let progress = widget.take_node_event().expect("progress event");
    assert_eq!(progress.event_code, event::TERMINALPROGRESS);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&progress.json).unwrap(),
        serde_json::json!({ "state": "set", "progress": 42 })
    );
    let notification = widget.take_node_event().expect("notification event");
    assert_eq!(notification.event_code, event::TERMINALNOTIFICATION);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&notification.json).unwrap(),
        serde_json::json!({
            "title": "Build \"done\"",
            "body": "All tests passed"
        })
    );
    assert_eq!(
        widget.take_node_event(),
        Some(WidgetNodeEvent::json(
            event::TERMINALEXIT,
            r#"{"reason":"exit"}"#
        ))
    );
    assert_eq!(widget.take_node_event(), None);
    assert!(widget.exit_reported);
    assert!(widget.next_cursor_blink.is_none());
}

#[test]
fn exited_terminal_is_read_only_but_remains_selectable_and_copyable() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"abcdef\x1b[?1000h\x1b[?1006h\x1b[?1004h");
    widget.report_exit_once();
    let cell = f64::from(DEFAULT_CELL_WIDTH);

    // Stale application mouse mode must not prevent local selection once
    // there is no process left to receive mouse reports.
    widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Move, cell * 5.75, 1.0, 1));
    widget.handle_event(&pointer(PointerPhase::Up, cell * 5.75, 1.0, 0));
    assert_eq!(widget.selected_text().as_deref(), Some("abcdef"));
    assert!(widget.take_input().is_empty());

    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL | Modifiers::SHIFT
    };
    let key = |name: &str| {
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
        })
    };
    assert_eq!(
        widget.handle_event(&key("c")).clipboard_request(),
        Some(&ClipboardRequest::Write("abcdef".into()))
    );
    assert_eq!(widget.handle_event(&key("v")).clipboard_request(), None);

    widget.handle_event(&UiEvent::TextInput("x".into()));
    widget.handle_event(&UiEvent::Paste("paste".into()));
    widget.handle_event(&UiEvent::Key(KeyEvent {
        phase: KeyPhase::Down,
        key: "Enter".into(),
        key_without_modifiers: "Enter".into(),
        code: "Enter".into(),
        text: None,
        text_with_all_modifiers: None,
        location: KeyLocation::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
    }));
    widget.focus_changed(true);

    assert_eq!(widget.selected_text().as_deref(), Some("abcdef"));
    assert!(widget.take_input().is_empty());
    assert!(widget.next_cursor_blink.is_none());
}

#[test]
fn osc_clipboard_read_requires_opt_in_and_formats_the_reply() {
    let mut denied = TerminalWidget::headless(20, 4);
    denied.feed(b"\x1b]52;c;?\x07");
    assert!(denied.poll_async());
    assert_eq!(denied.take_host_action(), None);
    assert_eq!(denied.take_input(), b"\x1b]52;c;\x07");

    let mut allowed = TerminalWidget::headless(20, 4);
    allowed.attribute_changed("allow-clipboard-read", "true");
    allowed.feed(b"\x1b]52;c;?\x07");
    assert!(allowed.poll_async());
    let request_id = match allowed.take_host_action() {
        Some(HostAction::ReadClipboard { request_id }) => request_id,
        action => panic!("expected clipboard read, got {action:?}"),
    };
    assert!(allowed.take_input().is_empty());
    allowed.complete_host_action(HostActionResult::Clipboard {
        request_id,
        text: Some("hello".into()),
    });
    assert_eq!(allowed.take_input(), b"\x1b]52;c;aGVsbG8=\x07");

    allowed.attribute_removed("allow-clipboard-read");
    assert!(!allowed.allow_clipboard_read);
}

#[test]
fn color_queries_use_the_rendered_palette() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(
        b"\x1b]4;1;#010203\x07\x1b]4;1;?\x07\x1b]4;2;?\x07\
          \x1b]10;?\x07\x1b]11;?\x07\x1b]12;?\x07",
    );

    assert!(widget.poll_async());
    assert_eq!(
        widget.take_input(),
        b"\x1b]4;1;rgb:0101/0202/0303\x07\
          \x1b]4;2;rgb:2222/c5c5/5e5e\x07\
          \x1b]10;rgb:e2e2/e8e8/f0f0\x07\
          \x1b]11;rgb:0f0f/1717/2a2a\x07\
          \x1b]12;rgb:e2e2/e8e8/f0f0\x07"
    );
    let colors = widget.terminal.lock().colors;
    assert_eq!(
        terminal_indexed_color(1, &colors),
        Color::from_rgb8(1, 2, 3)
    );
}

#[test]
fn host_theme_defaults_are_opt_in_and_osc_overrides_them() {
    let mut widget = TerminalWidget::headless(20, 4);
    let paint = Paint {
        text_color: Color::from_rgb8(0x11, 0x22, 0x33),
        background: Some(Color::from_rgb8(0x44, 0x55, 0x66)),
        ..Paint::default()
    };

    widget.style_changed(&WidgetStyle::from(&paint));
    assert_eq!(
        widget.theme_foreground,
        named_color(NamedColor::Foreground, true)
    );
    widget.attribute_changed("inherit-theme", "true");
    widget.style_changed(&WidgetStyle::from(&paint));
    assert_eq!(widget.theme_foreground, Color::from_rgb8(0x11, 0x22, 0x33));
    assert_eq!(widget.theme_background, Color::from_rgb8(0x44, 0x55, 0x66));

    widget.attribute_changed("inherit-theme", "false");
    assert_eq!(
        widget.theme_foreground,
        named_color(NamedColor::Foreground, true)
    );
    assert_eq!(
        widget.theme_background,
        named_color(NamedColor::Background, false)
    );
    widget.attribute_changed("inherit-theme", "true");
    widget.style_changed(&WidgetStyle::from(&paint));

    widget.feed(b"\x1b]10;#010203\x07\x1b]10;?\x07\x1b]11;?\x07");
    assert!(widget.poll_async());
    assert_eq!(
        widget.take_input(),
        b"\x1b]10;rgb:0101/0202/0303\x07\x1b]11;rgb:4444/5555/6666\x07"
    );
    let colors = widget.terminal.lock().colors;
    assert_eq!(
        terminal_ansi_color(
            AnsiColor::Named(NamedColor::Foreground),
            true,
            &colors,
            widget.theme_foreground,
            widget.theme_background,
        ),
        Color::from_rgb8(1, 2, 3)
    );
    widget.attribute_removed("inherit-theme");
    assert!(!widget.inherit_theme);
    assert_eq!(
        widget.theme_foreground,
        named_color(NamedColor::Foreground, true)
    );
}

#[test]
fn cursor_blink_mode_and_bell_reach_the_frontend() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.focus_changed(true);
    assert!(
        widget.next_cursor_blink.is_none(),
        "default cursor is steady"
    );

    widget.feed(b"\x1b[1 q");
    assert!(widget.poll_async());
    assert!(widget.terminal.lock().blinking_cursor);
    assert!(widget.next_cursor_blink.is_some());

    widget.feed(b"\x1b[2 q\x07");
    assert!(widget.poll_async());
    assert!(!widget.terminal.lock().blinking_cursor);
    assert!(widget.next_cursor_blink.is_none());
    assert_eq!(
        widget.take_host_action(),
        Some(HostAction::RequestAttention)
    );
    assert_eq!(
        widget.take_node_event(),
        Some(WidgetNodeEvent::json(event::TERMINALBELL, "{}"))
    );
}

#[test]
fn cursor_visual_stays_hollow_when_terminal_is_unfocused() {
    let cell = Rect::new(10.5, 20.5, 17.5, 37.5);
    assert_eq!(
        cursor_visual(false, false, CursorShape::Beam, 10.0, 20.0, 8.0, 18.0),
        Some(CursorVisual::Hollow(cell))
    );
    assert_eq!(
        cursor_visual(true, false, CursorShape::Block, 10.0, 20.0, 8.0, 18.0),
        None
    );
    assert_eq!(
        cursor_visual(true, true, CursorShape::Beam, 10.0, 20.0, 8.0, 18.0),
        Some(CursorVisual::Filled(Rect::new(10.0, 20.0, 12.0, 38.0)))
    );
    assert_eq!(
        cursor_visual(false, true, CursorShape::Hidden, 10.0, 20.0, 8.0, 18.0),
        None
    );
}

#[test]
fn pointer_drag_selects_terminal_grid_text() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed(b"hello world");
    widget.set_position(10.0, 20.0);

    widget.handle_event(&pointer(PointerPhase::Down, 10.1, 20.1, 1));
    widget.handle_event(&pointer(
        PointerPhase::Move,
        10.0 + f64::from(DEFAULT_CELL_WIDTH) * 4.75,
        20.1,
        1,
    ));
    widget.handle_event(&pointer(
        PointerPhase::Up,
        10.0 + f64::from(DEFAULT_CELL_WIDTH) * 4.75,
        20.1,
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

    widget.resize(DEFAULT_CELL_WIDTH * 20.0, DEFAULT_LINE_HEIGHT * 2.0);

    assert_eq!(widget.selected_text().as_deref(), Some("two"));
}

#[test]
fn scale_aware_paint_tracks_retina_density() {
    let mut widget = TerminalWidget::headless(20, 4);
    widget.feed("🚀".as_bytes());
    let mut tcx = TextContext::new();

    widget.paint_scaled(200.0, 80.0, 2.0, &mut tcx);

    assert_eq!(widget.device_scale, 2.0);
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
            modifiers: Modifiers::default(),
        }));
    }

    assert_eq!(widget.take_input(), b"\x1b[<64;3;2M\x1b[<65;3;2M");
    widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
        position,
        delta_x: 0.0,
        delta_y: -40.0,
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
        modifiers: Modifiers::default(),
    }));
    assert_eq!(widget.take_input(), b"\x1b[A");
}
