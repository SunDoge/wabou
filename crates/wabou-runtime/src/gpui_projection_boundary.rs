//! Cached GPUI view boundary for a committed Solid projection.

use std::{collections::BTreeMap, rc::Rc};

use wabou_shell::gpui::{AnyElement, AnyEntity, Context, Render, Window};
use wabou_shell::{
    GpuiNativeWidget, GpuiProjectionRenderSnapshot, NativeWidgetContext, NativeWidgetFactory,
    NativeWidgetInputHandler, NodeKey, ProjectedInputSink, ProjectedNativeElementFactory,
    ProjectedSubtreeElementFactory, ProjectedTextInputState, ProjectedTextSelection,
};

/// Rebuilds one native child element from retained GPUI state.
pub(crate) type NativeElementBuilder = Rc<dyn Fn() -> AnyElement>;

/// One explicit invalidation boundary between Solid's retained tree and GPUI.
///
/// The window root replaces the immutable projection snapshot only after a
/// completed Solid mutation frame. Unrelated GPUI entity notifications can
/// then reuse this View through `Entity::cached` without recursively
/// materializing every projected node.
pub(crate) struct GpuiProjectionBoundary {
    revision: u64,
    snapshot: GpuiProjectionRenderSnapshot,
    input: ProjectedInputSink,
    focus: wabou_shell::gpui::FocusHandle,
    text_input: ProjectedTextInputState,
    native_builders: BTreeMap<NodeKey, NativeElementBuilder>,
    subtree_builders: BTreeMap<NodeKey, NativeElementBuilder>,
    text_selections: Rc<BTreeMap<NodeKey, ProjectedTextSelection>>,
    widgets: Vec<GpuiNativeWidget>,
    native_widget_factories: std::collections::HashMap<String, NativeWidgetFactory>,
    native_widget_entities: BTreeMap<NodeKey, AnyEntity>,
    native_widget_inputs: BTreeMap<NodeKey, NativeWidgetInputHandler>,
    native_widget_attributes: BTreeMap<
        NodeKey,
        BTreeMap<wabou_shell::gpui::SharedString, wabou_shell::gpui::SharedString>,
    >,
    #[cfg(any(test, feature = "profiling"))]
    materialization_count: u64,
}

pub(crate) struct GpuiProjectionBoundaryState {
    pub(crate) revision: u64,
    pub(crate) snapshot: GpuiProjectionRenderSnapshot,
    pub(crate) input: ProjectedInputSink,
    pub(crate) focus: wabou_shell::gpui::FocusHandle,
    pub(crate) text_input: ProjectedTextInputState,
    pub(crate) native_builders: BTreeMap<NodeKey, NativeElementBuilder>,
    pub(crate) subtree_builders: BTreeMap<NodeKey, NativeElementBuilder>,
    pub(crate) text_selections: Rc<BTreeMap<NodeKey, ProjectedTextSelection>>,
    pub(crate) widgets: Vec<GpuiNativeWidget>,
    pub(crate) native_widget_factories: std::collections::HashMap<String, NativeWidgetFactory>,
}

impl GpuiProjectionBoundary {
    pub(crate) fn new(state: GpuiProjectionBoundaryState) -> Self {
        Self {
            revision: state.revision,
            snapshot: state.snapshot,
            input: state.input,
            focus: state.focus,
            text_input: state.text_input,
            native_builders: state.native_builders,
            subtree_builders: state.subtree_builders,
            text_selections: state.text_selections,
            widgets: state.widgets,
            native_widget_factories: state.native_widget_factories,
            native_widget_entities: BTreeMap::new(),
            native_widget_inputs: BTreeMap::new(),
            native_widget_attributes: BTreeMap::new(),
            #[cfg(any(test, feature = "profiling"))]
            materialization_count: 0,
        }
    }

    pub(crate) fn synchronize(
        &mut self,
        state: GpuiProjectionBoundaryState,
        cx: &mut Context<Self>,
    ) {
        if self.revision == state.revision {
            return;
        }
        self.revision = state.revision;
        self.snapshot = state.snapshot;
        self.input = state.input;
        self.focus = state.focus;
        self.text_input = state.text_input;
        self.native_builders = state.native_builders;
        self.subtree_builders = state.subtree_builders;
        self.text_selections = state.text_selections;
        self.widgets = state.widgets;
        self.native_widget_factories = state.native_widget_factories;
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn materialization_count(&self) -> u64 {
        self.materialization_count
    }

    pub(crate) fn dispatch_native_input(
        &self,
        key: NodeKey,
        event: wabou_shell::UiEvent,
        window: &mut Window,
        cx: &mut wabou_shell::gpui::App,
    ) -> bool {
        self.native_widget_inputs
            .get(&key)
            .is_some_and(|input| input(event, window, cx))
    }
}

impl Render for GpuiProjectionBoundary {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl wabou_shell::gpui::IntoElement {
        #[cfg(any(test, feature = "profiling"))]
        {
            self.materialization_count += 1;
        }
        #[cfg(feature = "profiling")]
        tracing::trace!(
            target: "wabou::perf",
            revision = self.revision,
            materializations = self.materialization_count,
            "gpui.projection.boundary_materialize"
        );

        let mut native_controls = self
            .native_builders
            .iter()
            .map(|(key, build)| (*key, build()))
            .collect::<BTreeMap<_, _>>();
        let retained_subtrees = Rc::new(std::cell::RefCell::new(
            self.subtree_builders
                .iter()
                .map(|(key, build)| (*key, build()))
                .collect::<BTreeMap<_, _>>(),
        ));
        self.native_widget_entities
            .retain(|key, _| self.widgets.iter().any(|widget| widget.key == *key));
        self.native_widget_inputs
            .retain(|key, _| self.widgets.iter().any(|widget| widget.key == *key));
        self.native_widget_attributes
            .retain(|key, _| self.widgets.iter().any(|widget| widget.key == *key));
        for widget in &self.widgets {
            let Some(factory) = self.native_widget_factories.get(widget.tag.as_ref()) else {
                continue;
            };
            let mount = factory(
                NativeWidgetContext::new_with_previous_attributes(
                    widget.key,
                    &widget.attributes,
                    self.native_widget_attributes.get(&widget.key),
                    widget.config.as_deref(),
                    self.native_widget_entities.get(&widget.key),
                    self.input.clone(),
                ),
                window,
                cx,
            );
            let (element, entity, input) = mount.into_parts();
            if let Some(entity) = entity {
                self.native_widget_entities.insert(widget.key, entity);
            } else {
                self.native_widget_entities.remove(&widget.key);
            }
            self.native_widget_attributes
                .insert(widget.key, widget.attributes.clone());
            if let Some(input) = input {
                self.native_widget_inputs.insert(widget.key, input);
            } else {
                self.native_widget_inputs.remove(&widget.key);
            }
            native_controls.insert(widget.key, element);
        }
        let native_controls = Rc::new(std::cell::RefCell::new(native_controls));
        let native: ProjectedNativeElementFactory =
            Rc::new(move |key| native_controls.borrow_mut().remove(&key));
        let subtree: ProjectedSubtreeElementFactory =
            Rc::new(move |key| retained_subtrees.borrow_mut().remove(&key));

        self.snapshot
            .interactive_element(
                NodeKey::ROOT,
                self.input.clone(),
                self.focus.clone(),
                self.text_input.clone(),
                Some(native),
                Some(subtree),
                self.text_selections.clone(),
            )
            .expect("the canonical Wabou root remains retained")
    }
}
