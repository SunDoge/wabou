//! # Layer 3 — layout fixture tests (final geometry)
//!
//! These tests are the primary regression surface for "does the UI look right?".
//! They construct small but realistic component trees (classes → Style IR →
//! cascade → inherit → Taffy/Parley measure → absolute rects) and assert
//! **final layout semantics**, not merely that an IR property was emitted.
//!
//! ## Three-layer pyramid (see also `computed_style` + `@wabou/style-compiler`)
//!
//! 1. **Compiler** (`packages/style-compiler`) — class CSS → typed Style IR.
//! 2. **Computed style** (`computed_style` module) — cascade, replace, inherit,
//!    inline priority via [`crate::applier::ComputedNodeSnapshot`].
//! 3. **Layout fixtures** (this module) — final rects, gaps, wrap, overflow.
//!
//! Prefer adding real regressions here when a visual bug ships (HN badge wrap,
//! gap, chrome heights, column compression, theme, resize, scroll).

#![cfg(test)]

use std::collections::HashMap;

use taffy::TraversePartialTree;
use vello::peniko::Color;
use wabou_shell::layout::PlacedNode;
use wabou_shell::{FrameSource, TextContext};

use super::{Applier, ComputedNodeSnapshot};
use crate::Atom;
use crate::jsrt::JsRuntime;
use crate::protocol::{Frame, Op};
use crate::style_ir::fixture::{
    auto, color, declaration, keyword, number, percent, px, record, rule, sheet,
};
use crate::style_ir::{StyleRule, StylesheetUpdate};

// ── harness ──────────────────────────────────────────────────────────────

struct Harness {
    applier: Applier,
    text: TextContext,
    atoms: HashMap<&'static str, Atom>,
    next_id: u32,
}

impl Harness {
    fn new() -> Self {
        let js = JsRuntime::new().expect("runtime");
        js.with(|ctx| {
            ctx.eval::<(), _>(
                "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
            )
        })
        .unwrap();
        let applier = Applier::from_runtime(js, Color::BLACK);
        Self {
            applier,
            text: TextContext::new(),
            atoms: HashMap::new(),
            next_id: 2, // 1 is the host root
        }
    }

    fn intern(&mut self, name: &'static str) -> Atom {
        if let Some(&a) = self.atoms.get(name) {
            return a;
        }
        let a = self.applier.atoms.borrow_mut().intern(name);
        self.atoms.insert(name, a);
        a
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn queue_stylesheet(&self, rules: Vec<StyleRule>) {
        *self.applier.pending_css.as_ref().unwrap().borrow_mut() =
            Some(StylesheetUpdate::Ir(sheet(rules)));
    }

    fn apply(&mut self, ops: Vec<Op>) {
        let seq = 1;
        self.applier.apply_frame(&Frame { seq, ops });
    }

    fn layout(&mut self, w: u32, h: u32) -> Vec<PlacedNode> {
        self.applier.build_frame(&mut self.text, w, h)
    }

    fn rect(&self, placed: &[PlacedNode], solid_id: u32) -> [f32; 4] {
        let node = self.applier.node_store.solid_to_node[&solid_id];
        placed
            .iter()
            .find(|item| item.node_id == node)
            .unwrap_or_else(|| panic!("no placed rect for solid_id {solid_id}"))
            .rect
    }

    fn snapshot(&self, solid_id: u32) -> ComputedNodeSnapshot {
        self.applier
            .computed_node_snapshot(solid_id)
            .unwrap_or_else(|| panic!("no snapshot for solid_id {solid_id}"))
    }

    fn solid_node(&self, solid_id: u32) -> taffy::NodeId {
        self.applier.node_store.solid_to_node[&solid_id]
    }
}

fn width(r: [f32; 4]) -> f32 {
    r[2] - r[0]
}
fn height(r: [f32; 4]) -> f32 {
    r[3] - r[1]
}
fn almost(a: f32, b: f32) {
    assert!(
        (a - b).abs() < 0.51,
        "expected {a} ≈ {b} (tol 0.5px), delta={}",
        (a - b).abs()
    );
}

fn flex_none() -> wabou_shell::style::IrValue {
    record([
        ("grow", number(0.0)),
        ("shrink", number(0.0)),
        ("basis", auto()),
    ])
}

fn flex_one() -> wabou_shell::style::IrValue {
    record([
        ("grow", number(1.0)),
        ("shrink", number(1.0)),
        ("basis", percent(0.0)),
    ])
}

// ── fixtures ─────────────────────────────────────────────────────────────

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
            id: row_id,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: row_id,
            classes: vec![row],
        },
        Op::CreateElement {
            id: badge_id,
            tag: span,
            attrs: vec![],
        },
        Op::SetClassName {
            id: badge_id,
            classes: vec![badge],
        },
        Op::CreateText {
            id: text_id,
            text: "128 comments",
        },
        Op::AppendChild {
            parent: badge_id,
            child: text_id,
        },
        Op::AppendChild {
            parent: row_id,
            child: badge_id,
        },
        Op::AppendChild {
            parent: 1,
            child: row_id,
        },
        Op::FrameEnd,
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
            id: sidebar_id,
            tag: aside,
            attrs: vec![],
        },
        Op::SetClassName {
            id: sidebar_id,
            classes: vec![sidebar],
        },
        Op::AppendChild {
            parent: 1,
            child: sidebar_id,
        },
        Op::FrameEnd,
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
            id: footer_id,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: footer_id,
            classes: vec![footer],
        },
        Op::CreateElement {
            id: label_id,
            tag: span,
            attrs: vec![],
        },
        Op::SetClassName {
            id: label_id,
            classes: vec![label],
        },
        Op::CreateText {
            id: text_id,
            text: "c = 0.788500 + 0.000000i",
        },
        Op::AppendChild {
            parent: label_id,
            child: text_id,
        },
        Op::AppendChild {
            parent: footer_id,
            child: label_id,
        },
        Op::AppendChild {
            parent: 1,
            child: footer_id,
        },
        Op::FrameEnd,
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
    h.apply(vec![
        Op::SetText {
            id: text_id,
            text: "c = -0.788500 + -0.000000i",
        },
        Op::FrameEnd,
    ]);
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
            id: footer_id,
            tag: view,
            attrs: vec![],
        },
        Op::SetClassName {
            id: footer_id,
            classes: vec![footer_class],
        },
        Op::CreateElement {
            id: label_id,
            tag: text,
            attrs: vec![],
        },
        Op::CreateText {
            id: count_id,
            text: "0",
        },
        Op::CreateText {
            id: suffix_id,
            text: " stories",
        },
        Op::AppendChild {
            parent: label_id,
            child: count_id,
        },
        Op::AppendChild {
            parent: label_id,
            child: suffix_id,
        },
        Op::AppendChild {
            parent: footer_id,
            child: label_id,
        },
        Op::AppendChild {
            parent: 1,
            child: footer_id,
        },
        Op::FrameEnd,
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

    h.apply(vec![
        Op::SetText {
            id: count_id,
            text: "1000",
        },
        Op::FrameEnd,
    ]);
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
            id: row,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: row,
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
                id: el,
                tag: span,
                attrs: vec![],
            },
            Op::CreateText {
                id: text,
                text: value,
            },
            Op::AppendChild {
                parent: el,
                child: text,
            },
            Op::AppendChild {
                parent: row,
                child: el,
            },
        ]);
    }
    ops.extend([
        Op::AppendChild {
            parent: 1,
            child: row,
        },
        Op::FrameEnd,
    ]);
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
    assert_eq!(h.applier.node_store.tree.child_count(h.solid_node(row)), 3);
    assert!(
        !h.applier
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
            attrs: vec![],
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
            attrs: vec![],
        },
        Op::SetClassName {
            id: header,
            classes: ["h-16", "flex-none", "px-6"].map(|n| classes[n]).to_vec(),
        },
        Op::CreateElement {
            id: content,
            tag: div,
            attrs: vec![],
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
        Op::FrameEnd,
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
        Op::CreateElement {
            id: row,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: row,
            classes: vec![row_c],
        },
        // rank
        Op::CreateElement {
            id: rank,
            tag: span,
            attrs: vec![],
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
        Op::CreateElement {
            id: body,
            tag: div,
            attrs: vec![],
        },
        Op::SetClassName {
            id: body,
            classes: vec![body_c],
        },
        Op::CreateElement {
            id: title_wrap,
            tag: div,
            attrs: vec![],
        },
        Op::CreateElement {
            id: title,
            tag: strong,
            attrs: vec![],
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
            attrs: vec![],
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
            attrs: vec![],
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
        Op::FrameEnd,
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
