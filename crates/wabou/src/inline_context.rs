//! Inline formatting context (IFC): when logical subtrees may collapse into one
//! Parley leaf for text performance.
//!
//! # Boundary with flex/grid layout
//!
//! Collapsing is a **formatting-context** decision, not merely "the node looks
//! inline". This module owns that decision. [`crate::applier::Applier`] applies
//! the result to Taffy children and host text state — it does not re-derive the
//! rules inline.
//!
//! ## Rules (must hold for a parent to absorb all children)
//!
//! 1. **Flex/grid items are never absorbed into the container.** If the parent
//!    establishes a flex or grid formatting context, each direct child keeps
//!    its own principal box (gap, alignment, fixed sizing, per-item hit test).
//! 2. **Only explicit text containers collapse.** JavaScript publishes the
//!    typed text-behavior contract. Rust never infers text behavior from
//!    HTML-like tag names.
//! 3. **Plain text only absorbs direct leaves by default.** An explicitly
//!    styled text container may also absorb nested text-only descendants; the
//!    host never guesses this behavior from tag names.
//! 4. **Replaced subtrees stay out.** SVG roots and host widgets own their
//!    content; descendants are logical-only and never become Taffy boxes under
//!    the replaced parent.
//! 5. **Styled runs stay correct.** Collapse only merges geometry. Color,
//!    font, weight, line-height and white-space are re-applied as Parley
//!    `TextRun`s over the logical tree after inherit (not in this module).
//!
//! ## All-or-nothing (current)
//!
//! A text container collapses **only if every** logical child satisfies its
//! declared mode: direct text for `Text`, recursively text-only descendants for
//! `RichText`. Mixed or replaced content keeps all children as separate boxes.
//!
//! ## Hit testing after merge
//!
//! Collapsed text leaves are not in the Taffy tree, so hits land on the
//! explicit text container.
//!
//! ## Accessibility
//!
//! Logical nodes remain in the Solid/DOM tree (`Declared` + child lists) for
//! updates and inheritance. Geometry/a11y mapping of collapsed phrasing into
//! the parent's box is **not** modeled yet: platform accessibility should treat
//! the IFC root as the text container until a separate a11y tree is built.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use taffy::NodeId;
use taffy::style::Display;

/// Per-node facts the IFC needs from the host (logical tree + computed style).
#[derive(Clone, Debug)]
pub struct NodeFacts {
    /// JavaScript-authored permission to absorb direct text children.
    pub text_container: bool,
    /// Explicit permission to recursively absorb non-replaced text-only
    /// descendants and preserve their styles as Parley runs.
    pub styled_text_container: bool,
    /// Text content when this is a `#text` leaf.
    pub text: Option<Arc<str>>,
    /// Computed `display` used for flex/grid/block boundary checks.
    pub display: Display,
    /// Whether cascade/inline style explicitly declared `display`.
    /// Taffy's default happens to be flex and must not create a CSS flex
    /// formatting context for an otherwise unstyled element.
    pub display_explicit: bool,
    /// SVG root, host widget, or other replaced content.
    pub replaced: bool,
}

impl NodeFacts {
    /// Parent establishes flex or grid formatting context → direct children
    /// must keep layout boxes.
    pub fn establishes_item_layout(&self) -> bool {
        self.display_explicit && matches!(self.display, Display::Flex | Display::Grid)
    }
}

/// Result of walking the logical tree once: what becomes layout boxes vs
/// Parley leaves.
#[derive(Clone, Debug, Default)]
pub struct InlineFormattingContext {
    /// Parents that absorbed all children into one Parley leaf.
    pub roots: HashSet<NodeId>,
    /// Flattened plain text for each IFC root (styled runs applied later).
    pub collapsed_text: HashMap<NodeId, Arc<str>>,
    /// Parents whose logical children do not become Taffy layout boxes because
    /// the parent is replaced content or absorbed its direct text leaves.
    /// Every other parent projects its existing logical child slice directly.
    pub suppressed_children: HashSet<NodeId>,
}

impl InlineFormattingContext {
    /// Build IFC decisions from the logical child map and per-node facts.
    ///
    /// `facts(node)` must be cheap and consistent with the cascade already
    /// applied for this frame. Text is collected only for parents that fully
    /// collapse.
    pub fn build(
        logical_children: &HashMap<NodeId, Vec<NodeId>>,
        facts: &impl Fn(NodeId) -> NodeFacts,
    ) -> Self {
        let mut ctx = Self::default();

        fn collect_text(
            node: NodeId,
            logical_children: &HashMap<NodeId, Vec<NodeId>>,
            facts: &impl Fn(NodeId) -> NodeFacts,
            recursive: bool,
            text: &mut String,
        ) -> bool {
            let node_facts = facts(node);
            if node_facts.replaced || node_facts.establishes_item_layout() {
                return false;
            }
            if let Some(value) = node_facts.text {
                if logical_children
                    .get(&node)
                    .is_some_and(|children| !children.is_empty())
                {
                    return false;
                }
                text.push_str(&value);
                return true;
            }
            if !recursive {
                return false;
            }
            let Some(children) = logical_children.get(&node) else {
                return false;
            };
            !children.is_empty()
                && children
                    .iter()
                    .all(|child| collect_text(*child, logical_children, facts, true, text))
        }

        for (&parent, kids) in logical_children {
            let parent_facts = facts(parent);

            // Replaced: logical descendants stay for serialization; no Taffy kids.
            if parent_facts.replaced {
                ctx.suppressed_children.insert(parent);
                continue;
            }

            // Flex/grid: every direct child keeps a principal box.
            if parent_facts.establishes_item_layout() {
                continue;
            }

            let mut text = String::new();
            let recursive = parent_facts.styled_text_container;
            let can_collapse = parent_facts.text_container
                && !kids.is_empty()
                && kids.iter().all(|child| {
                    collect_text(*child, logical_children, facts, recursive, &mut text)
                });

            if can_collapse {
                ctx.roots.insert(parent);
                ctx.collapsed_text.insert(parent, Arc::from(text));
                ctx.suppressed_children.insert(parent);
            }
        }

        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::TaffyTree;

    fn nid(tree: &mut TaffyTree<()>, n: &mut u64) -> NodeId {
        *n += 1;
        let _ = *n;
        tree.new_leaf(taffy::Style::default()).unwrap()
    }

    fn text_facts(s: &str) -> NodeFacts {
        NodeFacts {
            text_container: false,
            styled_text_container: false,
            text: Some(Arc::from(s)),
            display: Display::Block,
            display_explicit: false,
            replaced: false,
        }
    }

    fn element(display: Display) -> NodeFacts {
        NodeFacts {
            text_container: false,
            styled_text_container: false,
            text: None,
            display,
            display_explicit: false,
            replaced: false,
        }
    }

    #[test]
    fn ordinary_container_does_not_infer_text_flow() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let text = nid(&mut tree, &mut n);
        let children = HashMap::from([(parent, vec![text]), (text, vec![])]);
        let facts = |id| {
            if id == parent {
                element(Display::Block)
            } else {
                text_facts("hello")
            }
        };
        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(!ctx.roots.contains(&parent));
        assert!(!ctx.suppressed_children.contains(&parent));
    }

    #[test]
    fn flex_parent_never_collapses_direct_children() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let a = nid(&mut tree, &mut n);
        let b = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![a, b]);
        children.insert(a, vec![]);
        children.insert(b, vec![]);

        let facts = |id: NodeId| {
            if id == parent {
                let mut f = element(Display::Flex);
                f.display_explicit = true;
                f
            } else if id == a {
                text_facts("x")
            } else {
                text_facts("y")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(!ctx.roots.contains(&parent));
        assert!(!ctx.suppressed_children.contains(&parent));
    }

    #[test]
    fn explicit_text_host_is_one_item_inside_flex() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let text_host = nid(&mut tree, &mut n);
        let dynamic = nid(&mut tree, &mut n);
        let suffix = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![text_host]);
        children.insert(text_host, vec![dynamic, suffix]);
        children.insert(dynamic, vec![]);
        children.insert(suffix, vec![]);

        let facts = |id: NodeId| {
            if id == parent {
                let mut f = element(Display::Flex);
                f.display_explicit = true;
                f
            } else if id == text_host {
                let mut facts = element(Display::Flex);
                facts.text_container = true;
                facts
            } else if id == dynamic {
                text_facts("0")
            } else {
                text_facts(" stories")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(!ctx.suppressed_children.contains(&parent));
        assert!(ctx.roots.contains(&text_host));
        assert_eq!(ctx.collapsed_text[&text_host].as_ref(), "0 stories");
        assert!(ctx.suppressed_children.contains(&text_host));
    }

    #[test]
    fn explicit_text_container_rejects_nested_elements() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let nested = nid(&mut tree, &mut n);
        let text = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![nested]);
        children.insert(nested, vec![text]);
        children.insert(text, vec![]);

        let facts = |id: NodeId| {
            if id == parent {
                let mut facts = element(Display::Block);
                facts.text_container = true;
                facts
            } else if id == nested {
                element(Display::Block)
            } else {
                text_facts("world")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(!ctx.roots.contains(&parent));
        assert!(!ctx.suppressed_children.contains(&parent));
    }

    #[test]
    fn explicit_styled_text_container_collapses_text_only_descendants() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let plain = nid(&mut tree, &mut n);
        let span = nid(&mut tree, &mut n);
        let styled = nid(&mut tree, &mut n);
        let children = HashMap::from([
            (parent, vec![plain, span]),
            (plain, vec![]),
            (span, vec![styled]),
            (styled, vec![]),
        ]);
        let facts = |id: NodeId| {
            if id == parent {
                let mut facts = element(Display::Block);
                facts.text_container = true;
                facts.styled_text_container = true;
                facts
            } else if id == plain {
                text_facts("Before ")
            } else if id == span {
                element(Display::Block)
            } else {
                text_facts("code")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(ctx.roots.contains(&parent));
        assert_eq!(ctx.collapsed_text[&parent].as_ref(), "Before code");
        assert!(ctx.suppressed_children.contains(&parent));
    }

    #[test]
    fn replaced_parent_has_no_layout_children() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let path = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![path]);
        children.insert(path, vec![]);

        let facts = |id: NodeId| {
            if id == parent {
                let mut f = element(Display::Block);
                f.replaced = true;
                f
            } else {
                element(Display::Block)
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(ctx.suppressed_children.contains(&parent));
        assert!(!ctx.roots.contains(&parent));
    }
}
