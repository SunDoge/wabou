use super::*;

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
            .commands
            .len()
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
        false,
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
        synthetic: false,
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
            synthetic: false,
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
            synthetic: false,
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
            synthetic: false,
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
        synthetic: false,
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
        synthetic: false,
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
            synthetic: false,
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
        synthetic: false,
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
            synthetic: false,
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
