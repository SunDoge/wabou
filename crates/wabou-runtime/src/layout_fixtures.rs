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

use super::{Applier, ComputedNodeSnapshot, InvalidationFlags};
use crate::atom::Atom;
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

#[path = "layout_fixtures/chrome_cases.rs"]
mod chrome_cases;
#[path = "layout_fixtures/dynamic_cases.rs"]
mod dynamic_cases;
#[path = "layout_fixtures/text_cases.rs"]
mod text_cases;
