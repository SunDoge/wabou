use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{DebugSnapshot, MAX_VALIDATION_ISSUES, NodeKey, Rect};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// One deterministic retained-snapshot validation finding.
pub struct DebugValidationIssue {
    /// `error` for broken invariants or `warning` for actionable diagnostics.
    pub level: String,
    /// Stable machine-readable category.
    pub code: String,
    /// Human-readable evidence.
    pub message: String,
    /// Related retained node when the finding is node-specific.
    pub node_id: Option<NodeKey>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Self-consistency report for the latest retained snapshot.
pub struct DebugValidationReport {
    /// Snapshot revision that was validated.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub revision: u64,
    /// Whether no invariant errors were found.
    pub valid: bool,
    /// Number of error-level findings.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub error_count: usize,
    /// Number of warning-level findings.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub warning_count: usize,
    /// Whether additional findings were omitted from [`Self::issues`].
    pub truncated: bool,
    /// Deterministically ordered findings.
    pub issues: Vec<DebugValidationIssue>,
}

pub(crate) fn validate(snapshot: &DebugSnapshot) -> DebugValidationReport {
    let mut validator = Validator::default();
    validator.validate_snapshot(snapshot);
    validator.finish(snapshot.status.revision)
}

#[derive(Default)]
struct Validator {
    issues: Vec<DebugValidationIssue>,
}

impl Validator {
    fn validate_snapshot(&mut self, snapshot: &DebugSnapshot) {
        if snapshot.status.node_count != snapshot.nodes.len() {
            self.issue(
                "error",
                "node-count-mismatch",
                format!(
                    "status reports {} nodes but snapshot contains {}",
                    snapshot.status.node_count,
                    snapshot.nodes.len()
                ),
                None,
            );
        }
        if !snapshot.status.device_scale.is_finite() || snapshot.status.device_scale <= 0.0 {
            self.issue(
                "error",
                "invalid-device-scale",
                format!("device scale is {}", snapshot.status.device_scale),
                None,
            );
        }

        let mut live = HashSet::new();
        let mut parents = HashMap::new();
        for node in &snapshot.nodes {
            if !live.insert(node.id) {
                self.issue(
                    "error",
                    "duplicate-node-id",
                    format!("node {} appears more than once", node.id),
                    Some(node.id),
                );
            }
            parents.insert(node.id, node.parent_id);
        }

        for node in &snapshot.nodes {
            if let Some(parent) = node.parent_id
                && !live.contains(&parent)
            {
                self.issue(
                    "error",
                    "dangling-parent",
                    format!("parent {parent} is not present in this snapshot"),
                    Some(node.id),
                );
            }
            self.validate_rect("border-box", &node.rect, node.id);
            self.validate_rect("content-box", &node.content_rect, node.id);
            if rect_is_valid(&node.rect)
                && rect_is_valid(&node.content_rect)
                && !rect_contains_rect(&node.rect, &node.content_rect, 0.25)
            {
                self.issue(
                    "error",
                    "content-outside-border",
                    "content box extends outside its border box".to_owned(),
                    Some(node.id),
                );
            }
            for clip in node
                .clip
                .widget_local
                .iter()
                .chain(node.clip.chain.iter())
                .chain(node.clip.effective.iter())
            {
                self.validate_rect("clip", &clip.rect, node.id);
                if !clip.radius.is_finite() || clip.radius < 0.0 {
                    self.issue(
                        "error",
                        "invalid-clip-radius",
                        format!("{} clip radius is {}", clip.kind, clip.radius),
                        Some(node.id),
                    );
                }
                if !clip.transform.iter().all(|value| value.is_finite()) {
                    self.issue(
                        "error",
                        "invalid-clip-transform",
                        format!("{} clip transform contains a non-finite value", clip.kind),
                        Some(node.id),
                    );
                }
            }
            for (name, transform) in [
                ("static", node.clip.static_transform.as_slice()),
                ("border", node.clip.border_transform.as_slice()),
                ("scene", node.clip.scene_transform.as_slice()),
            ] {
                if !transform.iter().all(|value| value.is_finite()) {
                    self.issue(
                        "error",
                        "invalid-node-transform",
                        format!("{name} transform contains a non-finite value"),
                        Some(node.id),
                    );
                }
            }
            if node
                .clip
                .runtime_transform
                .is_some_and(|transform| !transform.iter().all(|value| value.is_finite()))
            {
                self.issue(
                    "error",
                    "invalid-node-transform",
                    "runtime transform contains a non-finite value".to_owned(),
                    Some(node.id),
                );
            }
            if !node.style_diagnostics.is_empty() {
                self.issue(
                    "warning",
                    "style-diagnostic",
                    node.style_diagnostics.join("; "),
                    Some(node.id),
                );
            }
            if let Some(semantic) = &node.semantic {
                for controlled in &semantic.controls {
                    if !live.contains(controlled) {
                        self.issue(
                            "error",
                            "dangling-semantic-reference",
                            format!("aria-controls target {controlled} is not live"),
                            Some(node.id),
                        );
                    }
                }
                if let Some(active) = semantic.active_descendant
                    && !live.contains(&active)
                {
                    self.issue(
                        "error",
                        "dangling-semantic-reference",
                        format!("active descendant {active} is not live"),
                        Some(node.id),
                    );
                }
            }
        }

        for node in &snapshot.nodes {
            let mut seen = HashSet::new();
            let mut current = Some(node.id);
            while let Some(id) = current {
                if !seen.insert(id) {
                    self.issue(
                        "error",
                        "parent-cycle",
                        format!("parent chain cycles through {id}"),
                        Some(node.id),
                    );
                    break;
                }
                current = parents.get(&id).copied().flatten();
            }
        }
        for (name, id) in [
            ("focused", snapshot.status.focused_node),
            ("hovered", snapshot.status.hovered_node),
        ] {
            if let Some(id) = id
                && !live.contains(&id)
            {
                self.issue(
                    "error",
                    "dangling-interaction-target",
                    format!("{name} node {id} is not live"),
                    Some(id),
                );
            }
        }
    }

    fn validate_rect(&mut self, kind: &str, rect: &Rect, node_id: NodeKey) {
        if !rect_is_valid(rect) {
            self.issue(
                "error",
                "invalid-geometry",
                format!(
                    "{kind} has invalid geometry x={} y={} width={} height={}",
                    rect.x, rect.y, rect.width, rect.height
                ),
                Some(node_id),
            );
        }
    }

    fn issue(&mut self, level: &str, code: &str, message: String, node_id: Option<NodeKey>) {
        self.issues.push(DebugValidationIssue {
            level: level.to_owned(),
            code: code.to_owned(),
            message,
            node_id,
        });
    }

    fn finish(mut self, revision: u64) -> DebugValidationReport {
        let error_count = self
            .issues
            .iter()
            .filter(|issue| issue.level == "error")
            .count();
        let warning_count = self
            .issues
            .iter()
            .filter(|issue| issue.level == "warning")
            .count();
        let truncated = self.issues.len() > MAX_VALIDATION_ISSUES;
        self.issues.truncate(MAX_VALIDATION_ISSUES);
        DebugValidationReport {
            revision,
            valid: error_count == 0,
            error_count,
            warning_count,
            truncated,
            issues: self.issues,
        }
    }
}

fn rect_is_valid(rect: &Rect) -> bool {
    [rect.x, rect.y, rect.width, rect.height]
        .into_iter()
        .all(f32::is_finite)
        && rect.width >= 0.0
        && rect.height >= 0.0
}

fn rect_contains_rect(outer: &Rect, inner: &Rect, tolerance: f32) -> bool {
    inner.x >= outer.x - tolerance
        && inner.y >= outer.y - tolerance
        && inner.x + inner.width <= outer.x + outer.width + tolerance
        && inner.y + inner.height <= outer.y + outer.height + tolerance
}
