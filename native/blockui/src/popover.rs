//! Controlled nonmodal overlay. The stable renderer owns placement and focus.
use crate::*;

/// Physical side and alignment; collision handling may flip the side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum PopoverPlacement {
    #[default]
    BottomStart = 0,
    BottomCenter = 1,
    BottomEnd = 2,
    TopStart = 3,
    TopCenter = 4,
    TopEnd = 5,
    RightStart = 6,
    RightCenter = 7,
    RightEnd = 8,
    LeftStart = 9,
    LeftCenter = 10,
    LeftEnd = 11,
}
impl TryFrom<i64> for PopoverPlacement {
    type Error = Error;
    fn try_from(value: i64) -> Result<Self> {
        Ok(match value {
            0 => Self::BottomStart,
            1 => Self::BottomCenter,
            2 => Self::BottomEnd,
            3 => Self::TopStart,
            4 => Self::TopCenter,
            5 => Self::TopEnd,
            6 => Self::RightStart,
            7 => Self::RightCenter,
            8 => Self::RightEnd,
            9 => Self::LeftStart,
            10 => Self::LeftCenter,
            11 => Self::LeftEnd,
            _ => return Err(Error::InvalidView("invalid popover placement")),
        })
    }
}

#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with children and mount the returned view"]
pub struct Popover {
    #[setters(generate, into)]
    pub key: Key,
    pub label: String,
    pub enabled: bool,
    pub width: f32,
    #[setters(generate)]
    placement: PopoverPlacement,
    open: State<bool>,
}
impl Popover {
    pub fn new(open: impl Into<State<bool>>, label: impl Into<String>) -> Self {
        let open = open.into();
        Self {
            key: Key::named("component.popover").scoped(open.key()),
            label: label.into(),
            enabled: true,
            width: 320.0,
            placement: PopoverPlacement::default(),
            open,
        }
    }
    /// Closed content is unmounted. Keep drafts in actor state to retain edits.
    pub fn children(self, cx: &Cx<'_>, children: Vec<View>) -> Result<View> {
        if self.label.trim().is_empty() {
            return Err(Error::InvalidView("popover needs a trigger label"));
        }
        if !self.width.is_finite() || !(64.0..=1280.0).contains(&self.width) {
            return Err(Error::InvalidView(
                "popover width must be between 64 and 1280",
            ));
        }
        let open = self.open;
        let expanded = self.enabled && open.get(cx)?;
        let trigger_key = self.key.scoped(abi::key("trigger"));
        let panel_key = self.key.scoped(abi::key("panel"));
        let mut trigger = Button {
            enabled: self.enabled,
            variant: ButtonVariant::Secondary,
            ..Button::label(self.label).key(trigger_key)
        }
        .on_click(move |cx| open.set(cx, !open.get(cx)?));
        trigger.node.semantics = abi::semantics::Semantics {
            role: abi::semantics::POPOVER_TRIGGER,
            expanded: if expanded { 2 } else { 1 },
            controls: panel_key.0,
            ..default()
        };
        let mut panel = ScrollView {
            height: 0.0,
            ..default()
        }
        .key(panel_key)
        .children(if expanded { children } else { vec![] });
        panel.node.gap = 12.0;
        panel.node.padding = 16.0;
        panel.node.presentation.width = self.width;
        panel.node.overlay_position = self.placement as u32;
        panel.node.semantics = abi::semantics::Semantics {
            role: abi::semantics::POPOVER_PANEL,
            hidden: u32::from(!expanded),
            labelled_by: trigger_key.0,
            ..default()
        };
        panel.event = Some(std::rc::Rc::new(move |cx, _| open.set(cx, false)));
        Ok(Column::default()
            .key(self.key)
            .children(vec![trigger, panel]))
    }
}
