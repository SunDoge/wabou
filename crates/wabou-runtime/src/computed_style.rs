//! # Layer 2 — computed-style tests
//!
//! Assert cascade / inheritance / priority at the
//! [`ComputedNodeSnapshot`](crate::applier::ComputedNodeSnapshot) boundary
//! without requiring pixel geometry. Prefer layer 3 (`layout_fixtures`) when
//! the bug is "rects/gaps/wrap look wrong".
//!
//! ## Pyramid
//!
//! 1. Compiler — `packages/vite/src/style-compiler` (class CSS → typed IR)
//! 2. **Computed style** (this module + selected `applier` tests)
//! 3. Layout fixtures — final rects (`layout_fixtures`)

#![cfg(test)]

use std::collections::HashMap;

use vello::peniko::Color;
use wabou_shell::FrameSource;

use super::{Applier, InvalidationFlags};
use crate::jsrt::JsRuntime;
use crate::protocol::{Frame, Op};
use crate::style_ir::StylesheetUpdate;
use crate::style_ir::fixture::{color, color_token, declaration, edges, keyword, px, rule, sheet};
use crate::style_ir::{Appearance, ColorTheme, ColorThemes, StyleSheet};

fn idle_runtime() -> JsRuntime {
    let js = JsRuntime::new().expect("runtime");
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    js
}

fn queue_stylesheet(applier: &Applier, rules: Vec<crate::style_ir::StyleRule>) {
    *applier.pending_css.as_ref().unwrap().borrow_mut() = Some(StylesheetUpdate::Ir(sheet(rules)));
}

#[test]
fn repeated_inline_updates_reuse_property_metadata() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, width) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("width"))
    };
    applier.apply_op(&Op::CreateElement { id: 2, tag: div });
    for value in ["10px", "20px"] {
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: width,
            value,
        });
    }

    assert_eq!(applier.style.inline_properties.len(), 1);
    let property = &applier.style.inline_properties[&width];
    assert_eq!(&*property.name, "width");
    assert!(!property.inherited);
}

#[test]
fn class_cascade_resolves_into_computed_snapshot() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, card) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("card"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![card],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    queue_stylesheet(
        &applier,
        vec![rule(
            "card",
            vec![
                declaration("width", px(160.0)),
                declaration("padding", edges(px(16.0))),
                declaration("background-color", color(0xef4444ff)),
                declaration("font-size", px(14.0)),
            ],
        )],
    );

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);
    let snapshot = applier.computed_node_snapshot(2).unwrap();

    assert_eq!(snapshot.classes, ["card"]);
    assert_eq!(snapshot.layout.size.width, taffy::Dimension::length(160.0));
    assert_eq!(
        snapshot.layout.padding.left,
        taffy::LengthPercentage::length(16.0)
    );
    assert_eq!(
        snapshot.background,
        Some(Color::from_rgb8(0xef, 0x44, 0x44))
    );
    assert_eq!(snapshot.font_size, 14.0);
}

#[test]
fn explicit_color_theme_switch_re_resolves_semantic_tokens() {
    use std::collections::HashMap;

    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, surface) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("bg-surface"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![surface],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    let themes = ColorThemes {
        default: "dark".into(),
        themes: HashMap::from([
            (
                "dark".into(),
                ColorTheme {
                    _appearance: Appearance::Dark,
                    colors: HashMap::from([("surface".into(), 0x0f172aff)]),
                },
            ),
            (
                "light".into(),
                ColorTheme {
                    _appearance: Appearance::Light,
                    colors: HashMap::from([("surface".into(), 0xffffffff)]),
                },
            ),
        ]),
    };
    *applier.pending_css.as_ref().unwrap().borrow_mut() = Some(StylesheetUpdate::Ir(
        StyleSheet::builder()
            .color_themes(themes)
            .rules(vec![rule(
                "bg-surface",
                vec![declaration("background-color", color_token("surface"))],
            )])
            .build(),
    ));

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);
    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().background,
        Some(Color::from_rgb8(0x0f, 0x17, 0x2a))
    );

    *applier.pending_color_theme.as_ref().unwrap().borrow_mut() = Some("light".into());
    applier.build_frame(&mut text, 800, 600);
    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().background,
        Some(Color::WHITE)
    );

    *applier.pending_color_palette.as_ref().unwrap().borrow_mut() = Some(vec![0x808080ff]);
    applier.build_frame(&mut text, 800, 600);
    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().background,
        Some(Color::from_rgb8(0x80, 0x80, 0x80))
    );
    assert!(
        !applier.invalidation.contains(InvalidationFlags::LAYOUT),
        "palette-only animation frames must retain the native layout cache"
    );
}

#[test]
fn native_utility_fallback_resolves_without_a_stylesheet() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, flex, padding, width, background, transform) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("flex"),
            atoms.intern("px-[13px]"),
            atoms.intern("w-full"),
            atoms.intern("bg-slate-900"),
            atoms.intern("translate-x-4"),
        )
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![flex, padding, width, background, transform],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);
    let snapshot = applier.computed_node_snapshot(2).unwrap();
    assert_eq!(snapshot.layout.display, taffy::Display::Flex);
    assert_eq!(
        snapshot.layout.padding.left,
        taffy::LengthPercentage::length(13.0)
    );
    assert_eq!(snapshot.layout.size.width, taffy::Dimension::percent(1.0));
    assert_eq!(
        snapshot.background,
        Some(Color::from_rgb8(0x0f, 0x17, 0x2a))
    );
    assert_eq!(
        snapshot.transforms,
        vec![wabou_shell::style::PaintTransform::Translate(
            wabou_shell::style::IrLength::Px { value: 16.0 },
            wabou_shell::style::IrLength::Px { value: 0.0 },
        )]
    );
}

#[test]
fn identical_ordered_class_lists_reuse_resolved_declarations() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, classes) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            vec![atoms.intern("flex"), atoms.intern("p-4")],
        )
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: classes.clone(),
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::CreateElement { id: 3, tag: div },
            Op::SetClassName { id: 3, classes },
            Op::AppendChild {
                parent: 1,
                child: 3,
            },
            Op::FrameEnd,
        ],
    });

    assert_eq!(applier.style.class_resolution_cache.len(), 1);
    assert!(applier.style.class_resolution_cache_hits >= 1);
    let left = applier.computed_node_snapshot(2).unwrap();
    let right = applier.computed_node_snapshot(3).unwrap();
    assert_eq!(left.layout.display, right.layout.display);
    assert_eq!(left.layout.padding, right.layout.padding);
}

#[test]
fn runtime_utility_fallback_uses_the_stylesheet_theme() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, brand) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("bg-brand"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![brand],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    let mut theme = wabou_style::Theme::default();
    theme.colors.insert("brand".to_string(), 0x336699ff);
    *applier.pending_css.as_ref().unwrap().borrow_mut() = Some(StylesheetUpdate::Ir(
        crate::style_ir::StyleSheet::builder().theme(theme).build(),
    ));

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);

    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().background,
        Some(Color::from_rgb8(0x33, 0x66, 0x99))
    );
}

#[test]
fn utility_order_is_last_wins_and_transform_components_compose() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, width_4, width_8, translate_x_4, translate_y_6, translate_x_2, scale, rotate) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("w-4"),
            atoms.intern("w-8"),
            atoms.intern("translate-x-4"),
            atoms.intern("translate-y-6"),
            atoms.intern("translate-x-2"),
            atoms.intern("scale-150"),
            atoms.intern("rotate-45"),
        )
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![
                    width_4,
                    width_8,
                    translate_x_4,
                    translate_y_6,
                    translate_x_2,
                    scale,
                    rotate,
                ],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);
    let snapshot = applier.computed_node_snapshot(2).unwrap();
    assert_eq!(snapshot.layout.size.width, taffy::Dimension::length(32.0));
    assert_eq!(
        snapshot.transforms,
        vec![
            wabou_shell::style::PaintTransform::Translate(
                wabou_shell::style::IrLength::Px { value: 8.0 },
                wabou_shell::style::IrLength::Px { value: 24.0 },
            ),
            wabou_shell::style::PaintTransform::Scale(1.5, 1.5),
            wabou_shell::style::PaintTransform::Rotate(std::f32::consts::FRAC_PI_4),
        ]
    );
}

#[test]
fn typed_inline_style_reaches_layout_without_string_parsing() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, width, opacity) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("width"),
            atoms.intern("opacity"),
        )
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::SetStyleValue {
                id: 2,
                prop: width,
                value: crate::protocol::StyleValue::Px(123.5),
            },
            Op::SetStyleValue {
                id: 2,
                prop: opacity,
                value: crate::protocol::StyleValue::Number(0.4),
            },
            Op::FrameEnd,
        ],
    });

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);
    let snapshot = applier.computed_node_snapshot(2).unwrap();
    assert_eq!(snapshot.layout.size.width, taffy::Dimension::length(123.5));
    assert_eq!(snapshot.opacity, 0.4);
}

#[test]
fn unknown_runtime_utility_is_recorded_for_diagnostics() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, unknown) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("stateful-magic"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![unknown],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);

    assert!(matches!(
        applier.style.utility_cache.get(&unknown),
        Some(Err(_))
    ));
    assert!(applier.style.warned_utility_classes.contains(&unknown));
    let node = applier.node_store.solid_to_node[&2];
    assert_eq!(applier.style.diagnostics[&node].len(), 1);
    assert!(applier.style.diagnostics[&node][0].contains("stateful-magic"));
}

#[test]
fn ignored_runtime_class_never_becomes_a_utility_diagnostic() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, lucide) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("lucide-sun"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![lucide],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    *applier.pending_css.as_ref().unwrap().borrow_mut() = Some(StylesheetUpdate::Ir(
        StyleSheet::builder()
            .ignored_class_patterns(vec!["lucide-*".into()])
            .build(),
    ));

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);

    assert!(!applier.style.utility_cache.contains_key(&lucide));
    assert!(!applier.style.warned_utility_classes.contains(&lucide));
    let node = applier.node_store.solid_to_node[&2];
    assert!(applier.style.diagnostics[&node].is_empty());
}

#[test]
fn runtime_utility_fallback_resolves_semantic_theme_colors_as_tokens() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, success) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("bg-success-surface"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![success],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    let themes = ColorThemes {
        default: "dark".into(),
        themes: HashMap::from([
            (
                "dark".into(),
                ColorTheme {
                    _appearance: Appearance::Dark,
                    colors: HashMap::from([("success-surface".into(), 0x064e3bff)]),
                },
            ),
            (
                "light".into(),
                ColorTheme {
                    _appearance: Appearance::Light,
                    colors: HashMap::from([("success-surface".into(), 0xecfdf5ff)]),
                },
            ),
        ]),
    };
    *applier.pending_css.as_ref().unwrap().borrow_mut() = Some(StylesheetUpdate::Ir(
        StyleSheet::builder().color_themes(themes).build(),
    ));

    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);
    assert!(!applier.style.warned_utility_classes.contains(&success));
    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().background,
        Some(Color::from_rgb8(0x06, 0x4e, 0x3b))
    );

    *applier.pending_color_theme.as_ref().unwrap().borrow_mut() = Some("light".into());
    applier.build_frame(&mut text, 800, 600);
    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().background,
        Some(Color::from_rgb8(0xec, 0xfd, 0xf5))
    );
}

#[test]
fn unsupported_inline_css_never_enters_cascade_state() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, transition, transform) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("transition"),
            atoms.intern("transform"),
        )
    };
    applier.apply_op(&Op::CreateElement { id: 2, tag: div });
    applier.apply_op(&Op::SetStyle {
        id: 2,
        prop: transition,
        value: "all 1s",
    });
    applier.apply_op(&Op::SetStyle {
        id: 2,
        prop: transform,
        value: "translate(10px, 0px)",
    });

    let node = applier.node_store.solid_to_node[&2];
    let inline = &applier.node_store.declared[&node].inline;
    assert!(!inline.contains_key(&transition));
    assert!(!inline.contains_key(&transform));
}

#[test]
fn replacing_class_resets_previous_declarations() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, compact, spacious) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("compact"),
            atoms.intern("spacious"),
        )
    };
    queue_stylesheet(
        &applier,
        vec![
            rule(
                "compact",
                vec![
                    declaration("padding", edges(px(8.0))),
                    declaration("background-color", color(0x111827ff)),
                ],
            ),
            rule(
                "spacious",
                vec![
                    declaration("padding", edges(px(24.0))),
                    declaration("background-color", color(0xf8fafcff)),
                ],
            ),
        ],
    );
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![compact],
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 200, 100);
    assert_eq!(
        applier
            .computed_node_snapshot(2)
            .unwrap()
            .layout
            .padding
            .left,
        taffy::LengthPercentage::length(8.0)
    );

    applier.apply_frame(&Frame {
        seq: 2,
        ops: vec![
            Op::SetClassName {
                id: 2,
                classes: vec![spacious],
            },
            Op::FrameEnd,
        ],
    });
    let snap = applier.computed_node_snapshot(2).unwrap();
    assert_eq!(snap.classes, ["spacious"]);
    assert_eq!(
        snap.layout.padding.left,
        taffy::LengthPercentage::length(24.0)
    );
    assert_eq!(snap.background, Some(Color::from_rgb8(0xf8, 0xfa, 0xfc)));
}

#[test]
fn inline_style_wins_over_class_for_same_property() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, card, width) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("card"),
            atoms.intern("width"),
        )
    };
    queue_stylesheet(
        &applier,
        vec![rule(
            "card",
            vec![
                declaration("width", px(100.0)),
                declaration("height", px(40.0)),
            ],
        )],
    );
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![card],
            },
            Op::SetStyle {
                id: 2,
                prop: width,
                value: "200px",
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 800, 600);
    let snap = applier.computed_node_snapshot(2).unwrap();
    assert_eq!(snap.layout.size.width, taffy::Dimension::length(200.0));
    assert_eq!(snap.layout.size.height, taffy::Dimension::length(40.0));
}

#[test]
fn white_space_nowrap_inherits_to_text_computed_style() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, span, badge) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("span"),
            atoms.intern("comments-badge"),
        )
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::CreateElement { id: 3, tag: span },
            Op::SetClassName {
                id: 3,
                classes: vec![badge],
            },
            Op::CreateText {
                id: 4,
                text: "1 comments",
            },
            Op::AppendChild {
                parent: 3,
                child: 4,
            },
            Op::AppendChild {
                parent: 2,
                child: 3,
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    queue_stylesheet(
        &applier,
        vec![rule(
            "comments-badge",
            vec![declaration("white-space", keyword("nowrap"))],
        )],
    );
    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 200, 100);
    assert!(!applier.computed_node_snapshot(4).unwrap().wrap_text);
    assert!(!applier.computed_node_snapshot(3).unwrap().wrap_text);
}

#[test]
fn font_color_inherits_from_parent_class() {
    let mut applier = Applier::from_runtime(idle_runtime(), Color::BLACK);
    let (div, parent_c) = {
        let mut atoms = applier.atoms.borrow_mut();
        (atoms.intern("div"), atoms.intern("ink"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement { id: 2, tag: div },
            Op::SetClassName {
                id: 2,
                classes: vec![parent_c],
            },
            Op::CreateText {
                id: 3,
                text: "hello",
            },
            Op::AppendChild {
                parent: 2,
                child: 3,
            },
            Op::AppendChild {
                parent: 1,
                child: 2,
            },
            Op::FrameEnd,
        ],
    });
    queue_stylesheet(
        &applier,
        vec![rule(
            "ink",
            vec![
                declaration("color", color(0xff0000ff)),
                declaration("font-size", px(20.0)),
            ],
        )],
    );
    let mut text = wabou_shell::TextContext::new();
    applier.build_frame(&mut text, 400, 200);
    let child = applier.computed_node_snapshot(3).unwrap();
    assert_eq!(child.text_color, Color::from_rgb8(0xff, 0x00, 0x00));
    assert_eq!(child.font_size, 20.0);
}
