//! Transitional projection from the existing Solid protocol into GPUI.
//!
//! This mirrors completed protocol frames while the old Winit renderer remains
//! the presentation path. It makes protocol ordering and retained identity
//! executable against the new backend before window ownership switches over.

use wabou_shell_gpui::{NodeKey, ProjectionError, ProjectionTree};

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

    pub(crate) fn apply_frame(&mut self, frame: &Frame<'_>) -> Result<(), ProjectionError> {
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
        let _ = self.tree.commit();
        Ok(())
    }

    #[cfg(test)]
    fn tree(&self) -> &ProjectionTree {
        &self.tree
    }
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
            .apply_frame(&Frame {
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

        let tree = projection.tree();
        assert_eq!(tree.node(NodeKey::ROOT).unwrap().children, [key(2)]);
        assert_eq!(tree.node(key(2)).unwrap().children, [key(3)]);
        assert_eq!(
            tree.node(key(3)).unwrap().text.as_deref(),
            Some("updated once per flush")
        );
    }
}
