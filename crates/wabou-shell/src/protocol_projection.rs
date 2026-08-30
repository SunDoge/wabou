//! Retained GPUI projection of completed Solid protocol frames.

// This is an internal runtime seam re-exported across workspace crates while
// the old applier is extracted. It is not a stable application-facing API.
#![allow(missing_docs)]

use crate::{
    DirtyKind, NodeKey, ProjectedNodeKind, ProjectionError, ProjectionTree, StyleDiagnostic,
    StyleProjection,
};
use wabou_style::{IrColor, IrLength, IrValue};

use wabou_protocol::{AtomPool, Frame, GRAPHIC_SOURCE_RESOURCE_RASTER, GRAPHIC_SOURCE_SVG, Op};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuiTextControl {
    pub key: NodeKey,
    pub multiline: bool,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub readonly: bool,
}

#[derive(Clone, Debug)]
pub struct GpuiNativeWidget {
    pub key: NodeKey,
    pub tag: crate::gpui::SharedString,
    pub attributes:
        std::collections::BTreeMap<crate::gpui::SharedString, crate::gpui::SharedString>,
}

#[derive(Debug)]
pub struct GpuiProjection {
    tree: ProjectionTree,
}

impl Default for GpuiProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuiProjection {
    pub fn new() -> Self {
        let mut tree = ProjectionTree::default();
        tree.insert(
            NodeKey::ROOT,
            None,
            0,
            gpui_style(),
            None,
            ProjectedNodeKind::Root,
        )
        .expect("the canonical projection root is unique");
        let _ = tree.commit();
        Self { tree }
    }

    /// Apply the structural part of one Solid flush without publishing it.
    ///
    /// Resolved styles are projected later while the legacy applier processes
    /// the same frame. `finish_frame` is therefore the only commit point: GPUI
    /// never observes a newly attached node with the previous/default style.
    pub fn apply_ops(
        &mut self,
        frame: &Frame<'_>,
        atoms: &AtomPool,
        mut resolve_raster: impl FnMut(&str) -> Option<std::sync::Arc<crate::gpui::Image>>,
    ) -> Result<(), ProjectionError> {
        for op in &frame.ops {
            match op {
                Op::CreateElement { id, tag } => {
                    let tag = atoms.resolve(*tag).unwrap_or("unknown");
                    self.tree.insert_detached(
                        *id,
                        gpui_style(),
                        None,
                        ProjectedNodeKind::Element(tag.into()),
                    )?;
                }
                Op::CreateText { id, text } => {
                    self.tree.insert_detached(
                        *id,
                        gpui_style(),
                        Some((*text).into()),
                        ProjectedNodeKind::Text,
                    )?;
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
                Op::SetAttribute { id, name, value } => {
                    if let Some(name) = atoms.resolve(*name) {
                        self.tree
                            .update_attribute(*id, name.into(), (*value).into())?;
                    }
                }
                Op::RemoveAttribute { id, name } => {
                    if let Some(name) = atoms.resolve(*name) {
                        self.tree.remove_attribute(*id, name)?;
                    }
                }
                Op::SetGraphicSource { id, kind, source } => match *kind {
                    GRAPHIC_SOURCE_SVG => self.tree.update_image(
                        *id,
                        Some(std::sync::Arc::new(crate::gpui::Image::from_bytes(
                            crate::gpui::ImageFormat::Svg,
                            source.as_bytes().to_vec(),
                        ))),
                    )?,
                    GRAPHIC_SOURCE_RESOURCE_RASTER => {
                        self.tree.update_image(*id, resolve_raster(source))?;
                    }
                    _ => {}
                },
                Op::ClearGraphicSource { id, kind }
                    if matches!(*kind, GRAPHIC_SOURCE_SVG | GRAPHIC_SOURCE_RESOURCE_RASTER) =>
                {
                    self.tree.update_image(*id, None)?
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Publish structure, text, and resolved-style changes as one GPUI update.
    #[must_use]
    pub fn finish_frame(&mut self) -> bool {
        !self.tree.commit().is_empty()
    }

    #[doc(hidden)]
    pub fn revision(&self) -> u64 {
        self.tree.revision()
    }

    pub fn contains(&self, key: NodeKey) -> bool {
        self.tree.node(key).is_some()
    }

    pub fn text_controls(&self) -> Vec<GpuiTextControl> {
        self.tree
            .roots()
            .iter()
            .flat_map(|root| self.text_controls_below(*root))
            .collect()
    }

    pub fn native_widgets(&self, mut accepts: impl FnMut(&str) -> bool) -> Vec<GpuiNativeWidget> {
        let mut widgets = Vec::new();
        let mut pending = self.tree.roots().to_vec();
        while let Some(key) = pending.pop() {
            let Some(node) = self.tree.node(key) else {
                continue;
            };
            pending.extend(node.children.iter().rev().copied());
            let ProjectedNodeKind::Element(tag) = &node.kind else {
                continue;
            };
            if accepts(tag.as_ref()) {
                widgets.push(GpuiNativeWidget {
                    key,
                    tag: tag.clone(),
                    attributes: node.attributes.clone(),
                });
            }
        }
        widgets
    }

    fn text_controls_below(&self, root: NodeKey) -> Vec<GpuiTextControl> {
        let mut controls = Vec::new();
        let mut pending = vec![root];
        while let Some(key) = pending.pop() {
            let Some(node) = self.tree.node(key) else {
                continue;
            };
            pending.extend(node.children.iter().rev().copied());
            let ProjectedNodeKind::Element(tag) = &node.kind else {
                continue;
            };
            if !matches!(tag.as_ref(), "input" | "textarea") {
                continue;
            }
            controls.push(GpuiTextControl {
                key,
                multiline: tag.as_ref() == "textarea",
                value: node
                    .attributes
                    .get("value")
                    .map_or_else(String::new, ToString::to_string),
                placeholder: node
                    .attributes
                    .get("placeholder")
                    .map_or_else(String::new, ToString::to_string),
                disabled: node.attributes.contains_key("disabled"),
                readonly: node.attributes.contains_key("readonly")
                    || node.attributes.contains_key("readOnly"),
            });
        }
        controls
    }

    #[doc(hidden)]
    pub fn tree_element(&self, root: NodeKey) -> Result<crate::ProjectedElement, ProjectionError> {
        self.tree.element(root)
    }

    pub fn interactive_tree_element(
        &self,
        root: NodeKey,
        input: crate::ProjectedInputSink,
        focus: crate::gpui::FocusHandle,
        text_input: crate::ProjectedTextInputState,
        native: Option<crate::ProjectedNativeElementFactory>,
    ) -> Result<crate::ProjectedElement, ProjectionError> {
        self.tree
            .interactive_element(root, input, focus, text_input, native)
    }

    pub fn update_style(
        &mut self,
        key: NodeKey,
        style: crate::gpui::Style,
    ) -> Result<(), ProjectionError> {
        self.tree
            .update_style(key, style, DirtyKind::LAYOUT | DirtyKind::PAINT)
    }

    pub fn apply_style_declaration(
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

    #[doc(hidden)]
    pub fn style(&self, key: NodeKey) -> Option<&crate::gpui::Style> {
        self.tree.node(key).map(|node| &node.style)
    }
}

pub fn project_ir(
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

fn gpui_style() -> crate::gpui::Style {
    crate::gpui::Style::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder as _;

    fn key(lo: u32) -> NodeKey {
        NodeKey::new(lo, 1)
    }

    #[test]
    fn completed_solid_frame_projects_structure_and_text_with_protocol_identity() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
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
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert!(projection.finish_frame());

        let tree = projection.tree();
        assert_eq!(tree.node(NodeKey::ROOT).unwrap().children, [key(2)]);
        assert_eq!(tree.node(key(2)).unwrap().children, [key(3)]);
        assert_eq!(
            tree.node(key(2)).unwrap().kind,
            ProjectedNodeKind::Element("view".into())
        );
        assert_eq!(
            tree.node(key(3)).unwrap().text.as_deref(),
            Some("updated once per flush")
        );
    }

    #[test]
    fn structure_and_style_publish_in_one_commit() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let initial_revision = projection.revision();
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
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

    #[test]
    fn graphic_sources_project_to_gpui_images_and_clear_explicitly() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let image_tag = atoms.intern("img");
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&[1, 2, 3, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        let raster = std::sync::Arc::new(crate::gpui::Image::from_bytes(
            crate::gpui::ImageFormat::Png,
            png,
        ));
        let source = "image:1";

        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: image_tag,
                        },
                        Op::SetGraphicSource {
                            id: key(2),
                            kind: GRAPHIC_SOURCE_RESOURCE_RASTER,
                            source,
                        },
                    ],
                },
                &atoms,
                |_| Some(raster.clone()),
            )
            .unwrap();

        assert_eq!(
            projection
                .tree()
                .node(key(2))
                .unwrap()
                .image
                .as_ref()
                .unwrap()
                .format(),
            crate::gpui::ImageFormat::Png
        );

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::ClearGraphicSource {
                        id: key(2),
                        kind: GRAPHIC_SOURCE_RESOURCE_RASTER,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.tree().node(key(2)).unwrap().image.is_none());
    }

    #[test]
    fn text_control_descriptors_follow_attached_generational_nodes_and_authored_state() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let input = atoms.intern("input");
        let textarea = atoms.intern("textarea");
        let value = atoms.intern("value");
        let placeholder = atoms.intern("placeholder");
        let disabled = atoms.intern("disabled");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: input,
                        },
                        Op::SetAttribute {
                            id: key(2),
                            name: value,
                            value: "typed",
                        },
                        Op::SetAttribute {
                            id: key(2),
                            name: placeholder,
                            value: "Search",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: textarea,
                        },
                        Op::SetAttribute {
                            id: key(3),
                            name: disabled,
                            value: "",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(3),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert_eq!(
            projection.text_controls(),
            vec![
                GpuiTextControl {
                    key: key(2),
                    multiline: false,
                    value: "typed".into(),
                    placeholder: "Search".into(),
                    disabled: false,
                    readonly: false,
                },
                GpuiTextControl {
                    key: key(3),
                    multiline: true,
                    value: String::new(),
                    placeholder: String::new(),
                    disabled: true,
                    readonly: false,
                },
            ]
        );

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::RemoveChild {
                        parent: NodeKey::ROOT,
                        child: key(2),
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert_eq!(
            projection
                .text_controls()
                .into_iter()
                .map(|control| control.key)
                .collect::<Vec<_>>(),
            [key(3)]
        );
    }

    #[test]
    fn native_widget_descriptors_preserve_authored_state_and_generational_identity() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let fractal = atoms.intern("fractal");
        let view = atoms.intern("view");
        let center_x = atoms.intern("center-x");
        let recreated = NodeKey::new(2, 7);

        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: recreated,
                            tag: fractal,
                        },
                        Op::SetAttribute {
                            id: recreated,
                            name: center_x,
                            value: "-0.745",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: recreated,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(3),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let widgets = projection.native_widgets(|tag| tag == "fractal");
        assert_eq!(widgets.len(), 1);
        assert_eq!(widgets[0].key, recreated);
        assert_eq!(widgets[0].tag.as_ref(), "fractal");
        assert_eq!(
            widgets[0].attributes.get("center-x").map(AsRef::as_ref),
            Some("-0.745")
        );

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::RemoveChild {
                        parent: NodeKey::ROOT,
                        child: recreated,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.native_widgets(|tag| tag == "fractal").is_empty());
    }
}
