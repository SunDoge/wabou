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
//! 2. **Block boundaries are not crossed.** A child that is block-level (HTML
//!    block tag, or computed `display: block|flex|grid|none`) is not an IFC
//!    participant; if any child fails, the parent does not collapse.
//! 3. **Independent principal boxes are kept.** Background, non-zero
//!    padding/margin/border, or explicit width/height mean the node paints or
//!    sizes on its own and must not disappear into a parent leaf.
//! 4. **Event targets are kept.** Nodes with listeners keep a layout box so
//!    hit testing remains addressable (see below).
//! 5. **Replaced subtrees stay out.** SVG roots and host widgets own their
//!    content; descendants are logical-only and never become Taffy boxes under
//!    the replaced parent.
//! 6. **Styled runs stay correct.** Collapse only merges geometry. Color,
//!    font, weight, line-height and white-space are re-applied as Parley
//!    `TextRun`s over the logical tree after inherit (not in this module).
//!
//! ## All-or-nothing (current)
//!
//! A parent collapses **only if every** logical child is a collapsible IFC
//! participant. Mixed block + inline keeps all children as separate boxes.
//! Anonymous-box run splitting is future work.
//!
//! ## Inline margin / padding
//!
//! **Not supported as inline-level decoration.** Non-zero padding or margin on
//! an otherwise-phrasing element makes [`NodeFacts::independent_box`] true, so
//! the node refuses collapse and keeps a Taffy box. True CSS inline
//! margin/padding (affecting line boxes without a principal block box) is out
//! of scope.
//!
//! ## Hit testing after merge
//!
//! Collapsed logical nodes are not in the Taffy tree, so they are **not**
//! hit-test targets. Hits land on the IFC root (the parent that absorbed
//! them). Nodes that need their own hit region must either:
//! - register listeners (blocks collapse), or
//! - establish an independent box (padding/background/…), or
//! - sit under a flex/grid parent as a direct item.
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
use taffy::prelude::TaffyAuto;
use taffy::style::Display;

/// Per-node facts the IFC needs from the host (logical tree + computed style).
#[derive(Clone, Debug)]
pub struct NodeFacts {
    /// HTML tag name when this is an element; `None` for `#text`.
    pub tag: Option<String>,
    /// Text content when this is a text node (or element with only stored text).
    pub text: Option<Arc<str>>,
    /// Computed `display` used for flex/grid/block boundary checks.
    pub display: Display,
    /// Whether cascade/inline style explicitly declared `display`.
    /// Taffy's default happens to be flex and must not create a CSS flex
    /// formatting context for an otherwise unstyled element.
    pub display_explicit: bool,
    /// SVG root, host widget, or other replaced content.
    pub replaced: bool,
    /// Solid event listeners registered on this node.
    pub has_listeners: bool,
    /// Own background, padding, margin, border, or explicit size — principal box.
    pub independent_box: bool,
}

impl NodeFacts {
    /// HTML phrasing / presentational inline tags we allow as IFC participants
    /// when they do not otherwise establish a principal box.
    pub fn is_phrasing_tag(tag: &str) -> bool {
        matches!(
            tag,
            "span"
                | "strong"
                | "b"
                | "em"
                | "i"
                | "small"
                | "code"
                | "a"
                | "label"
                | "abbr"
                | "cite"
                | "dfn"
                | "kbd"
                | "mark"
                | "q"
                | "s"
                | "samp"
                | "sub"
                | "sup"
                | "time"
                | "u"
                | "var"
        )
    }

    /// HTML elements whose default display is block-level (not IFC participants).
    pub fn is_block_tag(tag: &str) -> bool {
        matches!(
            tag,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "div"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "li"
                | "main"
                | "nav"
                | "ol"
                | "p"
                | "section"
                | "ul"
                | "button"
                | "input"
                | "textarea"
                | "select"
                | "table"
                | "thead"
                | "tbody"
                | "tr"
                | "td"
                | "th"
                | "pre"
                | "hr"
                | "figure"
                | "figcaption"
                | "view"
        )
    }

    /// Parent establishes flex or grid formatting context → direct children
    /// must keep layout boxes.
    pub fn establishes_item_layout(&self) -> bool {
        self.display_explicit && matches!(self.display, Display::Flex | Display::Grid)
    }

    /// Computed display is block-level for IFC purposes.
    ///
    /// Taffy's default `Display` is `Flex` when the flexbox feature is on, so
    /// we **cannot** treat bare `Flex` on a phrasing tag as "user asked for
    /// flex". Block tags and explicit grid/none always block participation.
    /// Flex is treated as a layout context only on the **parent** side via
    /// [`Self::establishes_item_layout`] after cascade has set `display:flex`.
    fn is_block_level_display(&self) -> bool {
        match self.display {
            Display::Block | Display::Grid | Display::None => self.display_explicit,
            // Flex on a block tag (or after CSS) is block-level participation.
            Display::Flex => self.display_explicit,
        }
    }

    /// Whether this node may sit inside a collapsed IFC (as content, not root).
    pub fn is_collapsible_participant(&self) -> bool {
        if self.replaced || self.has_listeners || self.independent_box {
            return false;
        }
        if self.text.is_some() && self.tag.is_none() {
            // Pure #text leaf — always inline content.
            return true;
        }
        let Some(tag) = self.tag.as_deref() else {
            return false;
        };
        if Self::is_block_tag(tag) || self.is_block_level_display() {
            return false;
        }
        Self::is_phrasing_tag(tag)
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
    /// Parent → children that remain Taffy layout boxes.
    ///
    /// Empty means: replaced parent, or full collapse into the parent leaf.
    /// Missing keys are not updated by the applier (parent had no logical entry).
    pub layout_children: HashMap<NodeId, Vec<NodeId>>,
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

        for (&parent, kids) in logical_children {
            let parent_facts = facts(parent);

            // Replaced: logical descendants stay for serialization; no Taffy kids.
            if parent_facts.replaced {
                ctx.layout_children.insert(parent, Vec::new());
                continue;
            }

            // Flex/grid: every direct child keeps a principal box.
            if parent_facts.establishes_item_layout() {
                ctx.layout_children.insert(parent, kids.clone());
                continue;
            }

            let can_collapse = !kids.is_empty()
                && kids
                    .iter()
                    .all(|&child| is_collapsible_subtree(child, logical_children, facts));

            if can_collapse {
                let mut text = String::new();
                for &child in kids {
                    collect_plain_text(child, logical_children, facts, &mut text);
                }
                ctx.roots.insert(parent);
                ctx.collapsed_text.insert(parent, Arc::from(text));
                ctx.layout_children.insert(parent, Vec::new());
            } else {
                ctx.layout_children.insert(parent, kids.clone());
            }
        }

        ctx
    }
}

fn is_collapsible_subtree(
    node: NodeId,
    logical_children: &HashMap<NodeId, Vec<NodeId>>,
    facts: &impl Fn(NodeId) -> NodeFacts,
) -> bool {
    let f = facts(node);
    if !f.is_collapsible_participant() {
        return false;
    }
    // Text leaf (no element tag): collapsible only when childless.
    if f.tag.is_none() {
        return logical_children.get(&node).is_none_or(|c| c.is_empty());
    }
    let kids = logical_children
        .get(&node)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    // Phrasing element must have at least one child (mirrors prior behaviour:
    // empty spans are not collapsed as intermediate roots of content).
    !kids.is_empty()
        && kids
            .iter()
            .all(|&c| is_collapsible_subtree(c, logical_children, facts))
}

fn collect_plain_text(
    node: NodeId,
    logical_children: &HashMap<NodeId, Vec<NodeId>>,
    facts: &impl Fn(NodeId) -> NodeFacts,
    out: &mut String,
) {
    let f = facts(node);
    if let Some(text) = &f.text {
        out.push_str(text);
        return;
    }
    if let Some(kids) = logical_children.get(&node) {
        for &child in kids {
            collect_plain_text(child, logical_children, facts, out);
        }
    }
}

/// Whether padding/margin rect has any non-zero length edge (px or %).
pub fn rect_has_nonzero_lp(rect: &taffy::Rect<taffy::LengthPercentage>) -> bool {
    [rect.top, rect.right, rect.bottom, rect.left]
        .into_iter()
        .any(|edge| match edge {
            // LengthPercentage is opaque; compare against zero length/percent.
            e if e == taffy::LengthPercentage::length(0.0) => false,
            e if e == taffy::LengthPercentage::percent(0.0) => false,
            _ => true,
        })
}

pub fn rect_has_nonzero_lpa(rect: &taffy::Rect<taffy::LengthPercentageAuto>) -> bool {
    [rect.top, rect.right, rect.bottom, rect.left]
        .into_iter()
        .any(|edge| {
            if edge == taffy::LengthPercentageAuto::length(0.0)
                || edge == taffy::LengthPercentageAuto::percent(0.0)
                || edge == taffy::LengthPercentageAuto::AUTO
            {
                return false;
            }
            true
        })
}

pub fn size_is_explicit(size: &taffy::Size<taffy::Dimension>) -> bool {
    !size.width.is_auto() || !size.height.is_auto()
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
            tag: None,
            text: Some(Arc::from(s)),
            display: Display::Block,
            display_explicit: false,
            replaced: false,
            has_listeners: false,
            independent_box: false,
        }
    }

    fn el(tag: &str, display: Display) -> NodeFacts {
        NodeFacts {
            tag: Some(tag.into()),
            text: None,
            display,
            display_explicit: false,
            replaced: false,
            has_listeners: false,
            independent_box: false,
        }
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
                let mut f = el("div", Display::Flex);
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
        assert_eq!(ctx.layout_children[&parent], vec![a, b]);
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
                let mut f = el("view", Display::Flex);
                f.display_explicit = true;
                f
            } else if id == text_host {
                el("text", Display::Flex)
            } else if id == dynamic {
                text_facts("0")
            } else {
                text_facts(" stories")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert_eq!(ctx.layout_children[&parent], vec![text_host]);
        assert!(ctx.roots.contains(&text_host));
        assert_eq!(ctx.collapsed_text[&text_host].as_ref(), "0 stories");
        assert!(ctx.layout_children[&text_host].is_empty());
    }

    #[test]
    fn pure_inline_under_block_collapses() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let t1 = nid(&mut tree, &mut n);
        let strong = nid(&mut tree, &mut n);
        let t2 = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![t1, strong]);
        children.insert(t1, vec![]);
        children.insert(strong, vec![t2]);
        children.insert(t2, vec![]);

        let facts = |id: NodeId| {
            if id == parent {
                el("div", Display::Block)
            } else if id == t1 {
                text_facts("Hello ")
            } else if id == strong {
                el("strong", Display::Flex) // taffy default; phrasing still ok
            } else {
                text_facts("world")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(ctx.roots.contains(&parent));
        assert!(ctx.roots.contains(&strong));
        assert_eq!(ctx.collapsed_text[&parent].as_ref(), "Hello world");
        assert!(ctx.layout_children[&parent].is_empty());
    }

    #[test]
    fn independent_box_blocks_collapse() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let badge = nid(&mut tree, &mut n);
        let t = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![badge]);
        children.insert(badge, vec![t]);
        children.insert(t, vec![]);

        let facts = |id: NodeId| {
            if id == parent {
                el("div", Display::Block)
            } else if id == badge {
                let mut f = el("span", Display::Flex);
                f.display_explicit = true;
                f.independent_box = true;
                f
            } else {
                text_facts("1 comments")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(!ctx.roots.contains(&parent));
        assert_eq!(ctx.layout_children[&parent], vec![badge]);
    }

    #[test]
    fn listeners_block_collapse() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let link = nid(&mut tree, &mut n);
        let t = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![link]);
        children.insert(link, vec![t]);
        children.insert(t, vec![]);

        let facts = |id: NodeId| {
            if id == parent {
                el("div", Display::Block)
            } else if id == link {
                let mut f = el("a", Display::Flex);
                f.has_listeners = true;
                f
            } else {
                text_facts("click")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(!ctx.roots.contains(&parent));
    }

    #[test]
    fn block_child_blocks_collapse() {
        let mut tree = TaffyTree::<()>::new();
        let mut n = 0;
        let parent = nid(&mut tree, &mut n);
        let child = nid(&mut tree, &mut n);
        let t = nid(&mut tree, &mut n);
        let mut children = HashMap::new();
        children.insert(parent, vec![child]);
        children.insert(child, vec![t]);
        children.insert(t, vec![]);

        let facts = |id: NodeId| {
            if id == parent || id == child {
                el("div", Display::Block)
            } else {
                text_facts("x")
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(!ctx.roots.contains(&parent));
        assert_eq!(ctx.layout_children[&parent], vec![child]);
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
                let mut f = el("svg", Display::Block);
                f.replaced = true;
                f
            } else {
                el("path", Display::Block)
            }
        };

        let ctx = InlineFormattingContext::build(&children, &facts);
        assert!(ctx.layout_children[&parent].is_empty());
        assert!(!ctx.roots.contains(&parent));
    }
}
