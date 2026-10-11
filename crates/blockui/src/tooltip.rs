//! Noninteractive hint. Stable hosts own transient visibility and timing.
use crate::*;

#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with trigger and mount the returned view"]
pub struct Tooltip {
    #[setters(generate, into)]
    key: Key,
    text: Binding,
    #[setters(generate)]
    placement: PopoverPlacement,
    #[setters(generate)]
    delay_ms: u32,
    #[setters(generate)]
    width: f32,
}
impl Tooltip {
    pub fn new(text: impl Into<Binding>) -> Self {
        Self {
            key: Key::named("component.tooltip"),
            text: text.into(),
            placement: PopoverPlacement::TopCenter,
            delay_ms: 700,
            width: 240.0,
        }
    }
    /// Use distinct keys for repeated hints. The supplied control retains its
    /// action, label and semantics; hints cannot contain interactive children.
    pub fn trigger(self, trigger: impl IntoView) -> Result<View> {
        if self.delay_ms > 10000 || !self.width.is_finite() || !(64.0..=640.0).contains(&self.width)
        {
            return Err(Error::InvalidView("invalid tooltip width or delay"));
        }
        let mut trigger = trigger.into_view();
        if !matches!(
            trigger.node.kind,
            abi::BUTTON | abi::ENTRY | abi::TEXTAREA | abi::CHECKBOX | abi::SLIDER | abi::SELECT
        ) || trigger.node.semantics.described_by != 0
        {
            return Err(Error::InvalidView(
                "tooltip needs one undescribed focusable control",
            ));
        }
        if trigger.node.key == 0 {
            trigger = trigger.key(self.key.scoped(abi::key("trigger")));
        }
        let panel_key = self.key.scoped(abi::key("hint"));
        trigger.node.semantics.described_by = panel_key.0;
        let mut hint = Text::new(self.text).key(panel_key).into_view();
        hint.node.height = 0.0;
        hint.node.padding = 8.0;
        hint.node.presentation.width = self.width;
        hint.node.presentation.font_size = 12.0;
        hint.node.semantics.role = abi::semantics::TOOLTIP;
        hint.node.tooltip_anchor = trigger.node.key;
        hint.node.tooltip_delay_ms = self.delay_ms;
        hint.node.overlay_position = self.placement as u32;
        Ok(Column {
            padding: 0.0,
            gap: 0.0,
            ..default()
        }
        .key(self.key)
        .children(vec![trigger, hint]))
    }
}
