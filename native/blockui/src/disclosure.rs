//! Controlled disclosure recipes. Content is unmounted when closed; state remains
//! actor-owned and relationships keep stable hidden panel placeholders.
use crate::*;
use abi::semantics as s;

pub struct AccordionItem {
    pub value: i64,
    pub label: String,
    pub enabled: bool,
    pub content: View,
}
impl AccordionItem {
    pub fn new(value: i64, label: impl Into<String>, content: impl IntoView) -> Self {
        Self {
            value,
            label: label.into(),
            enabled: true,
            content: content.into_view(),
        }
    }
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}
#[derive(Clone, Copy)]
enum Selection {
    Single(State<i64>),
    Multiple(State<Vec<i64>>),
}
/// Vertical accordion. All enabled headers participate in Tab order; Up/Down,
/// Home/End move focus without opening. Enter/Space activate the focused header.
/// Single mode reserves `Accordion::NONE` for no open item. Multiple mode uses a
/// unique list of item values. Removed/duplicate selected values are rejected.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with children and mount the returned view"]
pub struct Accordion {
    #[setters(generate, into)]
    pub key: Key,
    selection: Selection,
    collapsible: bool,
    #[setters(generate)]
    enabled: bool,
    #[setters(generate)]
    heading_level: u32,
}
impl Accordion {
    pub const NONE: i64 = i64::MIN;
    pub fn single(value: impl Into<State<i64>>) -> Self {
        let value = value.into();
        Self {
            key: Key::named("component.accordion").scoped(value.key()),
            selection: Selection::Single(value),
            collapsible: false,
            enabled: true,
            heading_level: 3,
        }
    }
    pub fn multiple(value: impl Into<State<Vec<i64>>>) -> Self {
        let value = value.into();
        Self {
            key: Key::named("component.accordion").scoped(value.key()),
            selection: Selection::Multiple(value),
            collapsible: true,
            enabled: true,
            heading_level: 3,
        }
    }
    /// Applies to single mode. Multiple mode always permits closing an item.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
    pub fn children(self, cx: &Cx<'_>, items: Vec<AccordionItem>) -> Result<View> {
        let mut values = std::collections::BTreeSet::new();
        for item in &items {
            if item.value == Self::NONE || !values.insert(item.value) {
                return Err(Error::InvalidView(
                    "accordion values must be unique and not NONE",
                ));
            }
        }
        let open = match self.selection {
            Selection::Single(state) => {
                let value = state.get(cx)?;
                if value == Self::NONE {
                    vec![]
                } else {
                    vec![value]
                }
            }
            Selection::Multiple(state) => state.get(cx)?,
        };
        let selected: std::collections::BTreeSet<_> = open.iter().copied().collect();
        if selected.len() != open.len() || selected.iter().any(|value| !values.contains(value)) {
            return Err(Error::InvalidView(
                "accordion selection must contain unique present values",
            ));
        }
        let mut children = Vec::with_capacity(items.len());
        for item in items {
            let item_key = self.key.scoped(item.value as u64);
            let trigger_key = item_key.scoped(abi::key("trigger"));
            let panel_key = item_key.scoped(abi::key("panel"));
            let active = selected.contains(&item.value);
            let value = item.value;
            let selection = self.selection;
            let collapsible = self.collapsible;
            let mut trigger = Button {
                enabled: self.enabled && item.enabled,
                variant: ButtonVariant::Secondary,
                ..Button::label(item.label.as_str()).key(trigger_key)
            }
            .on_click(move |cx| match selection {
                Selection::Single(state) => state.update(cx, |current| {
                    Ok(if current == value && collapsible {
                        Self::NONE
                    } else {
                        value
                    })
                }),
                Selection::Multiple(state) => state.update(cx, |current| {
                    if current.contains(&value) {
                        current.retain(|entry| *entry != value);
                    } else {
                        current.push(value);
                    }
                    Ok(())
                }),
            });
            trigger.node.semantics = s::Semantics {
                role: s::ACCORDION_TRIGGER,
                expanded: if active { 2 } else { 1 },
                locked: u32::from(
                    active && !self.collapsible && matches!(self.selection, Selection::Single(_)),
                ),
                controls: panel_key.0,
                ..default()
            };
            let mut heading = Column {
                gap: 0.0,
                padding: 0.0,
                ..default()
            }
            .key(item_key.scoped(abi::key("heading")))
            .children(vec![trigger]);
            heading.node.semantics = s::Semantics {
                role: s::HEADING,
                level: self.heading_level,
                ..default()
            };
            heading.text = item.label.as_str().into();
            let panel = panel(panel_key, trigger_key, active, &item.label, item.content);
            let mut view = Column {
                gap: 0.0,
                padding: 0.0,
                ..default()
            }
            .key(item_key)
            .children(vec![heading, panel]);
            view.node.semantics.role = s::ACCORDION_ITEM;
            children.push(view);
        }
        let mut view = Column {
            gap: 0.0,
            padding: 0.0,
            ..default()
        }
        .key(self.key)
        .children(children);
        view.node.semantics.role = s::ACCORDION;
        Ok(view)
    }
}
/// Independent boolean disclosure. Disabled state blocks user activation while
/// leaving the controlled state and any currently open content intact.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with children and mount the returned view"]
pub struct Collapsible {
    #[setters(generate, into)]
    pub key: Key,
    pub label: String,
    pub open: State<bool>,
    #[setters(generate)]
    pub enabled: bool,
}
impl Collapsible {
    pub fn new(label: impl Into<String>, open: impl Into<State<bool>>) -> Self {
        let open = open.into();
        Self {
            key: Key::named("component.collapsible").scoped(open.key()),
            label: label.into(),
            open,
            enabled: true,
        }
    }
    pub fn children(self, cx: &Cx<'_>, content: impl IntoView) -> Result<View> {
        let active = self.open.get(cx)?;
        let trigger_key = self.key.scoped(abi::key("trigger"));
        let panel_key = self.key.scoped(abi::key("panel"));
        let state = self.open;
        let mut trigger = Button {
            enabled: self.enabled,
            variant: ButtonVariant::Secondary,
            ..Button::label(self.label.as_str()).key(trigger_key)
        }
        .on_click(move |cx| state.update(cx, |current| Ok(!current)));
        trigger.node.semantics = s::Semantics {
            expanded: if active { 2 } else { 1 },
            controls: panel_key.0,
            ..default()
        };
        Ok(Column {
            gap: 0.0,
            padding: 0.0,
            ..default()
        }
        .key(self.key)
        .children(vec![
            trigger,
            panel(
                panel_key,
                trigger_key,
                active,
                &self.label,
                content.into_view(),
            ),
        ]))
    }
}
fn panel(key: Key, trigger: Key, active: bool, label: &str, content: View) -> View {
    let mut panel = Column {
        gap: 12.0,
        padding: 0.0,
        ..default()
    }
    .key(key)
    .children(if active { vec![content] } else { vec![] });
    panel.node.semantics = s::Semantics {
        role: s::DISCLOSURE_PANEL,
        hidden: u32::from(!active),
        labelled_by: trigger.0,
        ..default()
    };
    panel.text = label.into();
    panel
}
