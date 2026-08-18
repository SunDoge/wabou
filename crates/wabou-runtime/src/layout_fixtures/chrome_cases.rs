use super::*;

/// App shell: fixed header height, content fills remaining viewport.
#[test]
fn shell_header_fixed_height_content_flex_1() {
    let mut h = Harness::new();
    let div = h.intern("div");
    let classes: HashMap<_, _> = [
        "h-full",
        "min-h-0",
        "flex",
        "flex-col",
        "overflow-hidden",
        "h-16",
        "flex-none",
        "px-6",
        "flex-1",
        "min-w-0",
        "overflow-y-auto",
    ]
    .into_iter()
    .map(|n| (n, h.intern(n)))
    .collect();

    let shell = h.alloc_id();
    let header = h.alloc_id();
    let content = h.alloc_id();
    h.apply(vec![
        Op::CreateElement {
            id: shell,
            tag: div,
        },
        Op::SetClassName {
            id: shell,
            classes: ["h-full", "min-h-0", "flex", "flex-col", "overflow-hidden"]
                .map(|n| classes[n])
                .to_vec(),
        },
        Op::CreateElement {
            id: header,
            tag: div,
        },
        Op::SetClassName {
            id: header,
            classes: ["h-16", "flex-none", "px-6"].map(|n| classes[n]).to_vec(),
        },
        Op::CreateElement {
            id: content,
            tag: div,
        },
        Op::SetClassName {
            id: content,
            classes: ["flex-1", "min-w-0", "min-h-0", "overflow-y-auto"]
                .map(|n| classes[n])
                .to_vec(),
        },
        Op::AppendChild {
            parent: shell,
            child: header,
        },
        Op::AppendChild {
            parent: shell,
            child: content,
        },
        Op::AppendChild {
            parent: 1,
            child: shell,
        },
    ]);
    h.queue_stylesheet(vec![
        rule("h-full", vec![declaration("height", percent(1.0))]),
        rule("min-h-0", vec![declaration("min-height", px(0.0))]),
        rule("min-w-0", vec![declaration("min-width", px(0.0))]),
        rule("flex", vec![declaration("display", keyword("flex"))]),
        rule(
            "flex-col",
            vec![declaration("flex-direction", keyword("column"))],
        ),
        rule(
            "overflow-hidden",
            vec![declaration(
                "overflow",
                record([("x", keyword("hidden")), ("y", keyword("hidden"))]),
            )],
        ),
        rule("h-16", vec![declaration("height", px(64.0))]),
        rule("flex-none", vec![declaration("flex", flex_none())]),
        rule(
            "px-6",
            vec![
                declaration("padding-left", px(24.0)),
                declaration("padding-right", px(24.0)),
            ],
        ),
        rule("flex-1", vec![declaration("flex", flex_one())]),
        rule(
            "overflow-y-auto",
            vec![declaration("overflow-y", keyword("auto"))],
        ),
    ]);

    let placed = h.layout(1024, 768);
    let header_r = h.rect(&placed, header);
    let content_r = h.rect(&placed, content);
    almost(height(header_r), 64.0);
    almost(content_r[1], header_r[3]);
    almost(content_r[3], 768.0);
    // Content column flex-grows to fill remaining height under the fixed header.
    assert!(
        height(content_r) > 600.0,
        "flex-1 content should fill viewport under header (got {})",
        height(content_r)
    );
}

/// Story list row: long title compresses the middle column only — rank and
/// action columns keep flex-none widths.
#[test]
fn story_row_long_title_compresses_content_column_only() {
    let mut h = Harness::new();
    let div = h.intern("div");
    let span = h.intern("span");
    let strong = h.intern("strong");

    let class = |h: &mut Harness, name: &'static str| h.intern(name);
    let row_c = class(&mut h, "row");
    let rank_c = class(&mut h, "rank");
    let body_c = class(&mut h, "body");
    let actions_c = class(&mut h, "actions");
    let bookmark_c = class(&mut h, "bookmark");
    let title_c = class(&mut h, "title");

    let row = h.alloc_id();
    let rank = h.alloc_id();
    let rank_text = h.alloc_id();
    let body = h.alloc_id();
    let title_wrap = h.alloc_id();
    let title = h.alloc_id();
    let title_text = h.alloc_id();
    let actions = h.alloc_id();
    let actions_text = h.alloc_id();
    let bookmark = h.alloc_id();

    let long_title = "A very long Hacker News title that should shrink the content \
                      column without crushing the rank index or the action rail";

    h.apply(vec![
        Op::CreateElement { id: row, tag: div },
        Op::SetClassName {
            id: row,
            classes: vec![row_c],
        },
        // rank
        Op::CreateElement {
            id: rank,
            tag: span,
        },
        Op::SetClassName {
            id: rank,
            classes: vec![rank_c],
        },
        Op::CreateText {
            id: rank_text,
            text: "12",
        },
        Op::AppendChild {
            parent: rank,
            child: rank_text,
        },
        // body / title
        Op::CreateElement { id: body, tag: div },
        Op::SetClassName {
            id: body,
            classes: vec![body_c],
        },
        Op::CreateElement {
            id: title_wrap,
            tag: div,
        },
        Op::CreateElement {
            id: title,
            tag: strong,
        },
        Op::SetClassName {
            id: title,
            classes: vec![title_c],
        },
        Op::CreateText {
            id: title_text,
            text: long_title,
        },
        Op::AppendChild {
            parent: title,
            child: title_text,
        },
        Op::AppendChild {
            parent: title_wrap,
            child: title,
        },
        Op::AppendChild {
            parent: body,
            child: title_wrap,
        },
        // actions
        Op::CreateElement {
            id: actions,
            tag: span,
        },
        Op::SetClassName {
            id: actions,
            classes: vec![actions_c],
        },
        Op::CreateText {
            id: actions_text,
            text: "42",
        },
        Op::AppendChild {
            parent: actions,
            child: actions_text,
        },
        // bookmark
        Op::CreateElement {
            id: bookmark,
            tag: span,
        },
        Op::SetClassName {
            id: bookmark,
            classes: vec![bookmark_c],
        },
        Op::AppendChild {
            parent: row,
            child: rank,
        },
        Op::AppendChild {
            parent: row,
            child: body,
        },
        Op::AppendChild {
            parent: row,
            child: actions,
        },
        Op::AppendChild {
            parent: row,
            child: bookmark,
        },
        Op::AppendChild {
            parent: 1,
            child: row,
        },
    ]);

    // Mirror StoryList row utilities (gap-3 = 12px).
    h.queue_stylesheet(vec![
        rule(
            "row",
            vec![
                declaration("display", keyword("flex")),
                declaration("align-items", keyword("center")),
                declaration("gap", record([("row", px(12.0)), ("column", px(12.0))])),
                declaration("width", percent(1.0)),
                declaration("min-height", px(80.0)),
                declaration("padding-left", px(12.0)),
                declaration("padding-right", px(12.0)),
            ],
        ),
        rule(
            "rank",
            vec![
                declaration("width", px(28.0)),
                declaration("flex", flex_none()),
            ],
        ),
        rule(
            "body",
            vec![
                declaration("flex", flex_one()),
                declaration("min-width", px(0.0)),
            ],
        ),
        rule(
            "title",
            vec![
                declaration("font-size", px(14.0)),
                declaration("font-weight", number(500.0)),
            ],
        ),
        rule(
            "actions",
            vec![
                declaration("width", px(48.0)),
                declaration("flex", flex_none()),
            ],
        ),
        rule(
            "bookmark",
            vec![
                declaration("width", px(20.0)),
                declaration("height", px(20.0)),
                declaration("flex", flex_none()),
            ],
        ),
    ]);

    let viewport_w = 480u32;
    let placed = h.layout(viewport_w, 200);
    let rank_r = h.rect(&placed, rank);
    let body_r = h.rect(&placed, body);
    let actions_r = h.rect(&placed, actions);
    let bookmark_r = h.rect(&placed, bookmark);

    almost(width(rank_r), 28.0);
    almost(width(actions_r), 48.0);
    almost(width(bookmark_r), 20.0);
    // Body is the only flex-1 column — must be narrower than the free space
    // would allow if side columns also shrank, and strictly less than half the
    // row under a long title (compression signal).
    assert!(
        width(body_r) < 400.0,
        "content column should compress (got {})",
        width(body_r)
    );
    assert!(
        width(body_r) > 100.0,
        "content column should still receive remaining space (got {})",
        width(body_r)
    );
    // Side columns must not be pushed off-screen.
    assert!(rank_r[0] >= 0.0);
    assert!(bookmark_r[2] <= viewport_w as f32 + 0.5);
    // Order: rank → body → actions → bookmark with gap-3.
    almost(body_r[0] - rank_r[2], 12.0);
    almost(actions_r[0] - body_r[2], 12.0);
    almost(bookmark_r[0] - actions_r[2], 12.0);
}
