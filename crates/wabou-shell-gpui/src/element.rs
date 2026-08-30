use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window, div, prelude::*,
};

use crate::{NodeKey, ProjectionError, ProjectionTree};

/// A lightweight GPUI element generated from one Wabou retained node.
///
/// GPUI drops element objects after each frame. Stable state survives through
/// [`Element::id`], which is derived losslessly from Wabou's generational key.
pub struct ProjectedElement {
    key: NodeKey,
    style: gpui::Style,
    children: Vec<AnyElement>,
}

impl ProjectedElement {
    pub(crate) fn from_tree(tree: &ProjectionTree, key: NodeKey) -> Result<Self, ProjectionError> {
        let node = tree.node(key).ok_or(ProjectionError::MissingNode(key))?;
        let mut children =
            Vec::with_capacity(node.children.len() + usize::from(node.text.is_some()));
        if let Some(text) = &node.text {
            children.push(div().child(text.clone()).into_any_element());
        }
        for child in &node.children {
            children.push(Self::from_tree(tree, *child)?.into_any_element());
        }
        Ok(Self {
            key,
            style: node.style.clone(),
            children,
        })
    }
}

impl IntoElement for ProjectedElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ProjectedElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.key.gpui_element_id())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let child_layouts = self
            .children
            .iter_mut()
            .map(|child| child.request_layout(window, cx))
            .collect::<Vec<_>>();
        (
            window.request_layout(self.style.clone(), child_layouts, cx),
            (),
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        for child in &mut self.children {
            child.prepaint(window, cx);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        for child in &mut self.children {
            child.paint(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Style;

    #[test]
    fn generated_element_uses_the_retained_generational_identity() {
        let key = NodeKey::new(17, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(key, None, 0, Style::default(), Some("hello".into()))
            .unwrap();

        let element = ProjectedElement::from_tree(&tree, key).unwrap();
        assert_eq!(element.id(), Some(key.gpui_element_id()));
    }
}
