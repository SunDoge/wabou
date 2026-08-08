use super::*;

/// Theme tokens change paint only — geometry stays bit-identical.
#[test]
fn theme_switch_preserves_all_rects() {
    let mut h = Harness::new();
    let div = h.intern("div");
    let light = h.intern("theme-light");
    let dark = h.intern("theme-dark");
    let box_c = h.intern("box");

    let shell = h.alloc_id();
    let panel = h.alloc_id();
    h.apply(vec![
        Op::CreateElement {
            id: shell,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: shell,
            classes: vec![light],
        },
        Op::CreateElement {
            id: panel,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: panel,
            classes: vec![box_c],
        },
        Op::AppendChild {
            parent: shell,
            child: panel,
        },
        Op::AppendChild {
            parent: 1,
            child: shell,
        },
        Op::FrameEnd,
    ]);
    h.queue_stylesheet(vec![
        rule(
            "theme-light",
            vec![
                declaration("display", keyword("flex")),
                declaration("width", percent(1.0)),
                declaration("height", percent(1.0)),
                declaration("background-color", color(0xf8fafcff)),
            ],
        ),
        rule(
            "theme-dark",
            vec![
                declaration("display", keyword("flex")),
                declaration("width", percent(1.0)),
                declaration("height", percent(1.0)),
                declaration("background-color", color(0x111827ff)),
            ],
        ),
        rule(
            "box",
            vec![
                declaration("width", px(120.0)),
                declaration("height", px(48.0)),
                declaration("margin-left", px(16.0)),
                declaration("margin-top", px(24.0)),
            ],
        ),
    ]);

    let before = h.layout(800, 600);
    let shell_before = h.rect(&before, shell);
    let panel_before = h.rect(&before, panel);
    let bg_before = h.snapshot(shell).background;

    h.apply(vec![
        Op::SetClassName {
            id: shell,
            classes: vec![dark],
        },
        Op::FrameEnd,
    ]);
    let after = h.layout(800, 600);
    let shell_after = h.rect(&after, shell);
    let panel_after = h.rect(&after, panel);
    let bg_after = h.snapshot(shell).background;

    assert_ne!(bg_before, bg_after, "theme must change background paint");
    assert_eq!(shell_before, shell_after);
    assert_eq!(panel_before, panel_after);
    assert_eq!(
        h.solid_node(shell),
        h.applier.node_store.solid_to_node[&shell],
        "theme must not remount"
    );
}

/// Viewport resize relayouts geometry but keeps the same NodeId (no remount).
#[test]
fn viewport_resize_relayouts_without_remount() {
    let mut h = Harness::new();
    let div = h.intern("div");
    let fill = h.intern("fill");
    let id = h.alloc_id();
    h.apply(vec![
        Op::CreateElement {
            id,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id,
            classes: vec![fill],
        },
        Op::AppendChild {
            parent: 1,
            child: id,
        },
        Op::FrameEnd,
    ]);
    h.queue_stylesheet(vec![rule(
        "fill",
        vec![
            declaration("width", percent(1.0)),
            declaration("height", percent(1.0)),
        ],
    )]);

    let node = h.solid_node(id);
    let first = h.layout(800, 600);
    almost(width(h.rect(&first, id)), 800.0);
    almost(height(h.rect(&first, id)), 600.0);

    let second = h.layout(1600, 900);
    almost(width(h.rect(&second, id)), 1600.0);
    almost(height(h.rect(&second, id)), 900.0);
    assert_eq!(h.solid_node(id), node);
}

/// Scrollable content must not change fixed header/footer chrome sizes.
#[test]
fn overflow_scroll_preserves_fixed_chrome_sizes() {
    let mut h = Harness::new();
    let div = h.intern("div");

    let shell_c = h.intern("shell");
    let header_c = h.intern("header");
    let scroller_c = h.intern("scroller");
    let content_c = h.intern("scroll-content");
    let footer_c = h.intern("footer");
    let badge_c = h.intern("footer-badge");
    let tall_c = h.intern("tall");

    let shell = h.alloc_id();
    let header = h.alloc_id();
    let scroller = h.alloc_id();
    let content = h.alloc_id();
    let tall = h.alloc_id();
    let footer = h.alloc_id();
    let badge = h.alloc_id();

    h.apply(vec![
        Op::CreateElement {
            id: shell,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: shell,
            classes: vec![shell_c],
        },
        Op::CreateElement {
            id: header,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: header,
            classes: vec![header_c],
        },
        Op::CreateElement {
            id: scroller,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: scroller,
            classes: vec![scroller_c],
        },
        Op::CreateElement {
            id: content,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: content,
            classes: vec![content_c],
        },
        Op::CreateElement {
            id: tall,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: tall,
            classes: vec![tall_c],
        },
        Op::CreateElement {
            id: footer,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: footer,
            classes: vec![footer_c],
        },
        Op::CreateElement {
            id: badge,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: badge,
            classes: vec![badge_c],
        },
        Op::AppendChild {
            parent: content,
            child: tall,
        },
        Op::AppendChild {
            parent: scroller,
            child: content,
        },
        Op::AppendChild {
            parent: footer,
            child: badge,
        },
        Op::AppendChild {
            parent: shell,
            child: header,
        },
        Op::AppendChild {
            parent: shell,
            child: scroller,
        },
        Op::AppendChild {
            parent: shell,
            child: footer,
        },
        Op::AppendChild {
            parent: 1,
            child: shell,
        },
        Op::FrameEnd,
    ]);
    h.queue_stylesheet(vec![
        rule(
            "shell",
            vec![
                declaration("display", keyword("flex")),
                declaration("flex-direction", keyword("column")),
                declaration("width", percent(1.0)),
                declaration("height", percent(1.0)),
                declaration(
                    "overflow",
                    record([("x", keyword("hidden")), ("y", keyword("hidden"))]),
                ),
            ],
        ),
        rule(
            "header",
            vec![
                declaration("height", px(64.0)),
                declaration("flex", flex_none()),
                declaration("width", percent(1.0)),
            ],
        ),
        rule(
            "scroller",
            vec![
                declaration("flex", flex_one()),
                declaration("min-height", px(0.0)),
                declaration("overflow-y", keyword("auto")),
                declaration("width", percent(1.0)),
            ],
        ),
        rule(
            "scroll-content",
            vec![
                declaration("display", keyword("flex")),
                declaration("flex-direction", keyword("column")),
                declaration("flex", flex_none()),
                declaration("min-height", percent(1.0)),
            ],
        ),
        rule(
            "tall",
            vec![
                declaration("height", px(2000.0)),
                declaration("width", percent(1.0)),
                declaration("flex-shrink", number(0.0)),
            ],
        ),
        rule(
            "footer",
            vec![
                declaration("display", keyword("flex")),
                declaration("padding-top", px(8.0)),
                declaration("padding-bottom", px(8.0)),
                declaration("flex", flex_none()),
                declaration("width", percent(1.0)),
            ],
        ),
        rule(
            "footer-badge",
            vec![
                declaration("height", px(20.0)),
                declaration("flex", flex_none()),
            ],
        ),
    ]);

    let viewport_h = 400u32;
    let placed = h.layout(640, viewport_h);
    let header_r = h.rect(&placed, header);
    let scroller_r = h.rect(&placed, scroller);
    let footer_r = h.rect(&placed, footer);

    almost(height(header_r), 64.0);
    almost(height(footer_r), 36.0);
    almost(header_r[1], 0.0);
    almost(footer_r[3], viewport_h as f32);
    almost(scroller_r[1], header_r[3]);
    almost(scroller_r[3], footer_r[1]);
    let scroller_node = h.solid_node(scroller);
    assert!(
        h.applier
            .node_store
            .tree
            .layout(scroller_node)
            .unwrap()
            .content_size
            .height
            > height(scroller_r),
        "intrinsic scroll content must produce a scroll range"
    );
    // Content area is the remainder — not expanded by the 2000px tall child.
    almost(height(scroller_r), viewport_h as f32 - 64.0 - 36.0);

    // Scroll must not reflow chrome (scroll offsets are paint-time).
    let container = h.solid_node(scroller);
    h.applier.scroll_offsets.insert(container, [0.0, 120.0]);
    h.applier
        .invalidation
        .remove(super::InvalidationFlags::LAYOUT);
    let after_scroll = h.layout(640, viewport_h);
    assert_eq!(h.rect(&after_scroll, header), header_r);
    assert_eq!(h.rect(&after_scroll, footer), footer_r);
    almost(height(h.rect(&after_scroll, scroller)), height(scroller_r));
}
