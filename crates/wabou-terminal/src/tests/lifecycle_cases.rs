use super::*;

#[test]
fn resizing_updates_rio_grid() {
    let mut widget = TerminalWidget::headless(80, 24);
    widget.resize(DEFAULT_CELL_WIDTH * 40.0, DEFAULT_LINE_HEIGHT * 10.0, 1.0);
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
fn missing_working_directory_reports_the_path_before_spawning_env() {
    let missing =
        std::env::temp_dir().join(format!("wabou-terminal-missing-cwd-{}", std::process::id()));
    let mut widget = TerminalWidget::lazy_default_shell();
    widget.attribute_changed("cwd", &missing.display().to_string());
    widget.ensure_launched();

    let error = widget.spawn_error.as_deref().expect("spawn error");
    assert!(error.contains("terminal working directory"), "{error}");
    assert!(error.contains(&missing.display().to_string()), "{error}");
    assert!(!error.contains("/usr/bin/env"), "{error}");
}

#[test]
fn terminal_reports_physical_text_area_and_cell_dimensions() {
    let mut widget = TerminalWidget::headless(40, 10);
    widget.resize(340.0, 180.0, 2.0);

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
            synthetic: false,
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
        synthetic: false,
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
