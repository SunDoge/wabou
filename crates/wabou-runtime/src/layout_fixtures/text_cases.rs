use super::*;

/// HN comments badge: nowrap text must not wrap inside a narrow flex parent.
#[test]
fn hn_comments_badge_does_not_wrap() {
    let mut h = Harness::new();
    let div = h.intern("div");
    let span = h.intern("span");
    let row = h.intern("detail-actions");
    let badge = h.intern("comments-badge");

    let row_id = h.alloc_id();
    let badge_id = h.alloc_id();
    let text_id = h.alloc_id();
    h.apply(vec![
        Op::CreateElement {
            id: nk(row_id),
            tag: div,
        },
        Op::SetClassName {
            id: nk(row_id),
            classes: vec![row],
        },
        Op::CreateElement {
            id: nk(badge_id),
            tag: span,
        },
        Op::SetClassName {
            id: nk(badge_id),
            classes: vec![badge],
        },
        Op::CreateText {
            id: nk(text_id),
            text: "128 comments",
        },
        Op::AppendChild {
            parent: nk(badge_id),
            child: nk(text_id),
        },
        Op::AppendChild {
            parent: nk(row_id),
            child: nk(badge_id),
        },
        Op::AppendChild {
            parent: NodeKey::ROOT,
            child: nk(row_id),
        },
    ]);
    h.queue_stylesheet(vec![
        rule(
            "detail-actions",
            vec![
                declaration("display", keyword("flex")),
                // Narrow parent — wrap would squeeze the badge under 48px.
                declaration("width", px(48.0)),
            ],
        ),
        rule(
            "comments-badge",
            vec![
                declaration("display", keyword("flex")),
                declaration("height", px(36.0)),
                declaration("padding-left", px(16.0)),
                declaration("padding-right", px(16.0)),
                declaration("white-space", keyword("nowrap")),
                declaration("flex", flex_none()),
            ],
        ),
    ]);

    let placed = h.layout(200, 100);
    assert!(
        !h.snapshot(text_id).wrap_text,
        "text node must inherit nowrap from badge"
    );
    let badge_w = width(h.rect(&placed, badge_id));
    let text_w = width(h.rect(&placed, text_id));
    assert!(
        badge_w > 48.0,
        "badge must overflow narrow flex parent when nowrap (got {badge_w})"
    );
    assert!(
        text_w > 48.0,
        "text measure must stay one line (got {text_w})"
    );
}

/// Sidebar width follows the viewport while respecting readable bounds.
#[test]
fn adaptive_sidebar_width_is_clamped() {
    let mut h = Harness::new();
    let aside = h.intern("aside");
    let sidebar = h.intern("adaptive-sidebar");
    let sidebar_id = h.alloc_id();
    h.apply(vec![
        Op::CreateElement {
            id: nk(sidebar_id),
            tag: aside,
        },
        Op::SetClassName {
            id: nk(sidebar_id),
            classes: vec![sidebar],
        },
        Op::AppendChild {
            parent: NodeKey::ROOT,
            child: nk(sidebar_id),
        },
    ]);
    h.queue_stylesheet(vec![rule(
        "adaptive-sidebar",
        vec![
            declaration("width", percent(0.26)),
            declaration("min-width", px(256.0)),
            declaration("max-width", px(320.0)),
            declaration("flex", flex_none()),
        ],
    )]);

    let compact = h.layout(800, 600);
    almost(width(h.rect(&compact, sidebar_id)), 256.0);

    let wide = h.layout(1400, 600);
    almost(width(h.rect(&wide, sidebar_id)), 320.0);
}

/// A live numeric label must keep one line when its sign or digit count changes.
#[test]
fn changing_numeric_text_keeps_footer_label_on_one_line() {
    let mut h = Harness::new();
    let div = h.intern("div");
    let span = h.intern("span");
    let footer = h.intern("demo-footer");
    let label = h.intern("numeric-label");

    let footer_id = h.alloc_id();
    let label_id = h.alloc_id();
    let text_id = h.alloc_id();
    h.apply(vec![
        Op::CreateElement {
            id: nk(footer_id),
            tag: div,
        },
        Op::SetClassName {
            id: nk(footer_id),
            classes: vec![footer],
        },
        Op::CreateElement {
            id: nk(label_id),
            tag: span,
        },
        Op::SetClassName {
            id: nk(label_id),
            classes: vec![label],
        },
        Op::CreateText {
            id: nk(text_id),
            text: "c = 0.788500 + 0.000000i",
        },
        Op::AppendChild {
            parent: nk(label_id),
            child: nk(text_id),
        },
        Op::AppendChild {
            parent: nk(footer_id),
            child: nk(label_id),
        },
        Op::AppendChild {
            parent: NodeKey::ROOT,
            child: nk(footer_id),
        },
    ]);
    h.queue_stylesheet(vec![
        rule(
            "demo-footer",
            vec![
                declaration("display", keyword("flex")),
                declaration("width", px(120.0)),
            ],
        ),
        rule(
            "numeric-label",
            vec![
                declaration("display", keyword("flex")),
                declaration("white-space", keyword("nowrap")),
                declaration("flex", flex_none()),
            ],
        ),
    ]);

    let before = h.layout(200, 100);
    let before_height = height(h.rect(&before, text_id));
    h.apply(vec![Op::SetText {
        id: nk(text_id),
        text: "c = -0.788500 + -0.000000i",
    }]);
    let after = h.layout(200, 100);

    assert!(!h.snapshot(text_id).wrap_text);
    almost(height(h.rect(&after, text_id)), before_height);
    assert!(
        width(h.rect(&after, label_id)) > 120.0,
        "the label should overflow its narrow fixture instead of wrapping"
    );
}

/// A native `<Text>` absorbs Solid's adjacent dynamic/static text nodes into
/// one measured flex item and invalidates that run when either child changes.
#[test]
fn text_host_aggregates_reactive_footer_label() {
    let mut h = Harness::new();
    let view = h.intern("view");
    let text = h.intern("text");
    let footer_class = h.intern("text-host-footer");

    let footer_id = h.alloc_id();
    let label_id = h.alloc_id();
    let count_id = h.alloc_id();
    let suffix_id = h.alloc_id();
    h.apply(vec![
        Op::CreateElement {
            id: nk(footer_id),
            tag: view,
        },
        Op::SetClassName {
            id: nk(footer_id),
            classes: vec![footer_class],
        },
        Op::CreateElement {
            id: nk(label_id),
            tag: text,
        },
        Op::CreateText {
            id: nk(count_id),
            text: "0",
        },
        Op::CreateText {
            id: nk(suffix_id),
            text: " stories",
        },
        Op::AppendChild {
            parent: nk(label_id),
            child: nk(count_id),
        },
        Op::AppendChild {
            parent: nk(label_id),
            child: nk(suffix_id),
        },
        Op::AppendChild {
            parent: nk(footer_id),
            child: nk(label_id),
        },
        Op::AppendChild {
            parent: NodeKey::ROOT,
            child: nk(footer_id),
        },
    ]);
    h.queue_stylesheet(vec![rule(
        "text-host-footer",
        vec![declaration("display", keyword("flex"))],
    )]);

    let before = h.layout(240, 100);
    let before_width = width(h.rect(&before, label_id));
    let before_height = height(h.rect(&before, label_id));
    let label = h.snapshot(label_id);
    assert!(!label.wrap_text);
    assert_eq!(label.layout.flex_shrink, 0.0);
    let count_node = h.solid_node(count_id);
    let suffix_node = h.solid_node(suffix_id);
    assert!(before.iter().all(|item| item.node_id != count_node));
    assert!(before.iter().all(|item| item.node_id != suffix_node));

    h.apply(vec![Op::SetText {
        id: nk(count_id),
        text: "1000",
    }]);
    let after = h.layout(240, 100);

    assert!(width(h.rect(&after, label_id)) > before_width);
    almost(height(h.rect(&after, label_id)), before_height);
}

/// Metadata row `gap-2` is exactly 8px between flex items.
#[test]
fn metadata_gap_2_is_exactly_8px() {
    let mut h = Harness::new();
    let div = h.intern("div");
    let span = h.intern("span");
    let flex = h.intern("flex");
    let gap2 = h.intern("gap-2");

    let row = h.alloc_id();
    let mut ops = vec![
        Op::CreateElement {
            id: nk(row),
            tag: div,
        },
        Op::SetClassName {
            id: nk(row),
            classes: vec![flex, gap2],
        },
    ];
    let mut item_ids = Vec::new();
    for value in ["420 points", "·", "alice"] {
        let el = h.alloc_id();
        let text = h.alloc_id();
        item_ids.push(el);
        ops.extend([
            Op::CreateElement {
                id: nk(el),
                tag: span,
            },
            Op::CreateText {
                id: nk(text),
                text: value,
            },
            Op::AppendChild {
                parent: nk(el),
                child: nk(text),
            },
            Op::AppendChild {
                parent: nk(row),
                child: nk(el),
            },
        ]);
    }
    ops.extend([Op::AppendChild {
        parent: NodeKey::ROOT,
        child: nk(row),
    }]);
    h.apply(ops);
    h.queue_stylesheet(vec![
        rule("flex", vec![declaration("display", keyword("flex"))]),
        rule(
            "gap-2",
            vec![declaration(
                "gap",
                record([("row", px(8.0)), ("column", px(8.0))]),
            )],
        ),
    ]);

    let placed = h.layout(800, 600);
    assert_eq!(
        h.applier
            .document
            .node_store
            .tree
            .child_count(h.solid_node(row)),
        3
    );
    assert!(
        !h.applier
            .document
            .node_store
            .inline_roots
            .contains(&h.solid_node(row))
    );

    let a = h.rect(&placed, item_ids[0]);
    let b = h.rect(&placed, item_ids[1]);
    let c = h.rect(&placed, item_ids[2]);
    almost(b[0] - a[2], 8.0);
    almost(c[0] - b[2], 8.0);
}
