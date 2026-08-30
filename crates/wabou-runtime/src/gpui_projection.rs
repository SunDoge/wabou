//! Transitional projection from the existing Solid protocol into GPUI.
//!
//! This mirrors completed protocol frames while the old Winit renderer remains
//! the presentation path. It makes protocol ordering and retained identity
//! executable against the new backend before window ownership switches over.

use wabou_shell::style::{IrColor, IrLength, IrValue};
use wabou_shell_gpui::{
    DirtyKind, NodeKey, ProjectionError, ProjectionTree, StyleDiagnostic, StyleProjection,
};

use crate::protocol::{Frame, Op};

#[derive(Debug)]
pub(crate) struct GpuiProjection {
    tree: ProjectionTree,
}

impl GpuiProjection {
    pub(crate) fn new() -> Self {
        let mut tree = ProjectionTree::default();
        tree.insert(NodeKey::ROOT, None, 0, gpui_style(), None)
            .expect("the canonical projection root is unique");
        let _ = tree.commit();
        Self { tree }
    }

    /// Apply the structural part of one Solid flush without publishing it.
    ///
    /// Resolved styles are projected later while the legacy applier processes
    /// the same frame. `finish_frame` is therefore the only commit point: GPUI
    /// never observes a newly attached node with the previous/default style.
    pub(crate) fn apply_ops(&mut self, frame: &Frame<'_>) -> Result<(), ProjectionError> {
        for op in &frame.ops {
            match op {
                Op::CreateElement { id, .. } => {
                    self.tree.insert_detached(*id, gpui_style(), None)?;
                }
                Op::CreateText { id, text } => {
                    self.tree
                        .insert_detached(*id, gpui_style(), Some((*text).into()))?;
                }
                Op::AppendChild { parent, child } => {
                    let index = self
                        .tree
                        .node(*parent)
                        .ok_or(ProjectionError::MissingParent(*parent))?
                        .children
                        .len();
                    self.tree.attach_child(*child, *parent, index)?;
                }
                Op::InsertBefore {
                    parent,
                    child,
                    ref_id,
                } => {
                    let index = self
                        .tree
                        .node(*parent)
                        .ok_or(ProjectionError::MissingParent(*parent))?
                        .children
                        .iter()
                        .position(|candidate| candidate == ref_id)
                        .ok_or(ProjectionError::MissingNode(*ref_id))?;
                    self.tree.attach_child(*child, *parent, index)?;
                }
                Op::RemoveChild { child, .. } => self.tree.detach(*child)?,
                Op::SetText { id, text } => {
                    self.tree.update_text(*id, Some((*text).into()))?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Publish structure, text, and resolved-style changes as one GPUI update.
    #[must_use]
    pub(crate) fn finish_frame(&mut self) -> bool {
        !self.tree.commit().is_empty()
    }

    #[cfg(test)]
    pub(crate) fn revision(&self) -> u64 {
        self.tree.revision()
    }

    pub(crate) fn contains(&self, key: NodeKey) -> bool {
        self.tree.node(key).is_some()
    }

    #[cfg(test)]
    pub(crate) fn tree_element(
        &self,
        root: NodeKey,
    ) -> Result<wabou_shell_gpui::ProjectedElement, ProjectionError> {
        self.tree.element(root)
    }

    pub(crate) fn interactive_tree_element(
        &self,
        root: NodeKey,
        input: wabou_shell_gpui::ProjectedInputSink,
        focus: wabou_shell_gpui::gpui::FocusHandle,
    ) -> Result<wabou_shell_gpui::ProjectedElement, ProjectionError> {
        self.tree.interactive_element(root, input, focus)
    }

    pub(crate) fn update_style(
        &mut self,
        key: NodeKey,
        style: wabou_shell_gpui::gpui::Style,
    ) -> Result<(), ProjectionError> {
        self.tree
            .update_style(key, style, DirtyKind::LAYOUT | DirtyKind::PAINT)
    }

    pub(crate) fn apply_style_declaration(
        &mut self,
        key: NodeKey,
        property: &str,
        value: &IrValue,
    ) -> Result<Option<StyleDiagnostic>, ProjectionError> {
        let current = self
            .tree
            .node(key)
            .ok_or(ProjectionError::MissingNode(key))?
            .style
            .clone();
        let mut projection = StyleProjection::from_style(current);
        let diagnostic = project_ir(&mut projection, property, value);
        self.update_style(key, projection.into_style())?;
        Ok(diagnostic)
    }

    #[cfg(test)]
    fn tree(&self) -> &ProjectionTree {
        &self.tree
    }

    #[cfg(test)]
    pub(crate) fn style(&self, key: NodeKey) -> Option<&wabou_shell_gpui::gpui::Style> {
        self.tree.node(key).map(|node| &node.style)
    }
}

pub(crate) fn project_ir(
    projection: &mut StyleProjection,
    property: &str,
    value: &IrValue,
) -> Option<StyleDiagnostic> {
    let value = tooling_value(value)?;
    projection
        .apply(&wabou_style::Declaration {
            property: property.to_owned(),
            value,
        })
        .err()
}

fn tooling_value(value: &IrValue) -> Option<wabou_style::Value> {
    Some(match value {
        IrValue::Keyword { value } => wabou_style::Value::Keyword {
            value: value.clone(),
        },
        IrValue::Boolean { value } => wabou_style::Value::Boolean { value: *value },
        IrValue::Number { value } => wabou_style::Value::Number { value: *value },
        IrValue::Length { value } => wabou_style::Value::Length {
            value: match value {
                IrLength::Px { value } => wabou_style::Length::Px { value: *value },
                IrLength::Percent { value } => wabou_style::Length::Percent { value: *value },
                IrLength::Auto => wabou_style::Length::Auto,
            },
        },
        IrValue::Color {
            value: IrColor::Literal { rgba },
        } => wabou_style::Value::Color {
            value: wabou_style::Color::Literal { rgba: *rgba },
        },
        IrValue::Color {
            value: IrColor::Token { .. },
        } => return None,
        IrValue::List { values } => wabou_style::Value::List {
            values: values.iter().map(tooling_value).collect::<Option<_>>()?,
        },
        IrValue::Record { fields } => wabou_style::Value::Record {
            fields: fields
                .iter()
                .map(|(key, value)| Some((key.clone(), tooling_value(value)?)))
                .collect::<Option<_>>()?,
        },
    })
}

fn gpui_style() -> wabou_shell_gpui::gpui::Style {
    wabou_shell_gpui::gpui::Style::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom::Atom;

    fn key(lo: u32) -> NodeKey {
        NodeKey::new(lo, 1)
    }

    #[test]
    fn completed_solid_frame_projects_structure_and_text_with_protocol_identity() {
        let mut projection = GpuiProjection::new();
        projection
            .apply_ops(&Frame {
                seq: 1,
                ops: vec![
                    Op::CreateElement {
                        id: key(2),
                        tag: Atom::from_raw(1),
                    },
                    Op::CreateText {
                        id: key(3),
                        text: "hello GPUI",
                    },
                    Op::AppendChild {
                        parent: NodeKey::ROOT,
                        child: key(2),
                    },
                    Op::AppendChild {
                        parent: key(2),
                        child: key(3),
                    },
                    Op::SetText {
                        id: key(3),
                        text: "updated once per flush",
                    },
                ],
            })
            .unwrap();

        assert!(projection.finish_frame());

        let tree = projection.tree();
        assert_eq!(tree.node(NodeKey::ROOT).unwrap().children, [key(2)]);
        assert_eq!(tree.node(key(2)).unwrap().children, [key(3)]);
        assert_eq!(
            tree.node(key(3)).unwrap().text.as_deref(),
            Some("updated once per flush")
        );
    }

    #[test]
    fn structure_and_style_publish_in_one_commit() {
        let mut projection = GpuiProjection::new();
        let initial_revision = projection.revision();
        projection
            .apply_ops(&Frame {
                seq: 1,
                ops: vec![
                    Op::CreateElement {
                        id: key(2),
                        tag: Atom::from_raw(1),
                    },
                    Op::AppendChild {
                        parent: NodeKey::ROOT,
                        child: key(2),
                    },
                ],
            })
            .unwrap();
        projection
            .apply_style_declaration(
                key(2),
                "background-color",
                &IrValue::Color {
                    value: IrColor::Literal { rgba: 0x4080_bfff },
                },
            )
            .unwrap();

        assert_eq!(projection.revision(), initial_revision);
        assert!(projection.finish_frame());
        assert_eq!(projection.revision(), initial_revision + 1);
        assert!(!projection.finish_frame());
        assert_eq!(projection.revision(), initial_revision + 1);
    }
}
