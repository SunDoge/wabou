//! Controlled choice components. Callbacks stay in the current implementation.
use crate::*;
use abi::semantics as s;

/// A boolean switch with a visible label. Shares Checkbox's typed state and event
/// contract; the stable renderer supplies its switch visual and accessible role.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Switch<V = State<bool>> {
    control: Checkbox<V>,
}
impl Switch {
    /// Create a switch with directly inferred closure argument types.
    /// ```
    /// use blockui::*;
    /// let enabled = State::<bool>::new("enabled", false);
    /// let switch = Switch::with_handler("Enabled", enabled, move |cx, value| enabled.set(cx, value));
    /// ```
    pub fn with_handler(
        label: impl Into<String>,
        value: impl Into<State<bool>>,
        on_change: impl for<'cx, 'host> Fn(&'cx Cx<'host>, bool) -> Result<()> + 'static,
    ) -> EventControl<Self> {
        Self::new(label, value, on_change)
    }
    pub fn new<H: BooleanControlHandler<Self>>(
        label: impl Into<String>,
        value: impl Into<State<bool>>,
        on_change: H,
    ) -> H::Output {
        let action = on_change.action();
        on_change.attach(Self {
            control: Checkbox::new(label, value, action),
        })
    }
}
impl<V> Switch<V> {
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.control.key = key.into();
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.control.enabled = enabled;
        self
    }
}
impl<V> IntoView for Switch<V>
where
    Checkbox<V>: IntoView,
{
    fn into_view(self) -> View {
        let mut view = self.control.into_view();
        view.node.semantics.role = s::SWITCH;
        view
    }
}

/// A stable integer choice. Values must be unique within their group.
pub struct ChoiceOption {
    pub value: i64,
    pub label: String,
    pub enabled: bool,
}
impl ChoiceOption {
    pub fn new(value: i64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            enabled: true,
        }
    }
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}
/// Radio selection with one Tab stop and automatic selection on arrow navigation.
/// An unmatched state means no checked option; Tab enters at the first enabled one.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with view and mount the returned view"]
pub struct RadioGroup {
    #[setters(generate, into)]
    pub key: Key,
    pub label: String,
    pub value: State<i64>,
    pub options: Vec<ChoiceOption>,
}
impl RadioGroup {
    pub fn new(
        label: impl Into<String>,
        value: impl Into<State<i64>>,
        options: Vec<ChoiceOption>,
    ) -> Self {
        let value = value.into();
        Self {
            key: Key::named("component.radio").scoped(value.key()),
            label: label.into(),
            value,
            options,
        }
    }
    pub fn view(self, cx: &Cx<'_>) -> Result<View> {
        unique(self.options.iter().map(|o| o.value))?;
        let selected = self.value.get(cx)?;
        let stop = tab_stop(
            self.options
                .iter()
                .map(|option| (option.value, option.enabled)),
            selected,
        );
        let caption = self.key.scoped(abi::key("caption"));
        let mut children = vec![
            Description::new(self.label.as_str())
                .key(caption)
                .into_view(),
        ];
        for option in self.options {
            children.push(choice(
                self.key.scoped(option.value as u64),
                self.value,
                option.value,
                &option.label,
                option.enabled,
                s::Semantics {
                    role: s::RADIO,
                    selected: u32::from(selected == option.value),
                    tab_stop: u32::from(stop == Some(option.value)),
                    ..default()
                },
            ));
        }
        let mut view = Column {
            gap: 8.0,
            padding: 0.0,
            ..default()
        }
        .key(self.key)
        .children(children);
        view.node.semantics = s::Semantics {
            role: s::RADIO_GROUP,
            labelled_by: caption.0,
            ..default()
        };
        view.text = self.label.into();
        Ok(view)
    }
}
/// One tab and the view displayed when it is selected. Unselected content is
/// unmounted; actor state survives switching. Each tab keeps a hidden, empty
/// panel placeholder so every controls/labelled-by relationship resolves.
pub struct Tab {
    pub value: i64,
    pub label: String,
    pub enabled: bool,
    pub content: View,
}
impl Tab {
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
/// Horizontal tabs with automatic activation, wrapping Left/Right and Home/End.
/// State controls selection; changes use a current-implementation callback.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with children and mount the returned view"]
pub struct Tabs {
    #[setters(generate, into)]
    pub key: Key,
    pub label: String,
    pub value: State<i64>,
}
impl Tabs {
    pub fn new(label: impl Into<String>, value: impl Into<State<i64>>) -> Self {
        let value = value.into();
        Self {
            key: Key::named("component.tabs").scoped(value.key()),
            label: label.into(),
            value,
        }
    }
    pub fn children(self, cx: &Cx<'_>, tabs: Vec<Tab>) -> Result<View> {
        unique(tabs.iter().map(|t| t.value))?;
        let selected = self.value.get(cx)?;
        if !tabs.is_empty() && !tabs.iter().any(|tab| tab.value == selected) {
            return Err(Error::InvalidView("selected tab is absent"));
        }
        let stop = tab_stop(tabs.iter().map(|tab| (tab.value, tab.enabled)), selected);
        let mut triggers = Vec::new();
        let mut panels = Vec::new();
        for tab in tabs {
            let key = self.key.scoped(tab.value as u64);
            let panel_key = key.scoped(abi::key("panel"));
            let trigger = choice(
                key,
                self.value,
                tab.value,
                &tab.label,
                tab.enabled,
                s::Semantics {
                    role: s::TAB,
                    selected: u32::from(selected == tab.value),
                    tab_stop: u32::from(stop == Some(tab.value)),
                    controls: panel_key.0,
                    ..default()
                },
            );
            triggers.push(trigger);
            let active = selected == tab.value;
            let mut panel = Column {
                gap: 12.0,
                padding: 0.0,
                ..default()
            }
            .key(panel_key)
            .children(if active { vec![tab.content] } else { vec![] });
            panel.node.semantics = s::Semantics {
                role: s::TAB_PANEL,
                labelled_by: key.0,
                hidden: u32::from(!active),
                tab_stop: u32::from(active),
                ..default()
            };
            panel.text = tab.label.into();
            panels.push(panel);
        }
        let mut list = Row {
            gap: 4.0,
            padding: 4.0,
            ..default()
        }
        .key(self.key.scoped(abi::key("list")))
        .children(triggers);
        list.node.semantics.role = s::TAB_LIST;
        list.text = self.label.into();
        let mut children = vec![list];
        children.extend(panels);
        Ok(Column {
            gap: 16.0,
            padding: 0.0,
            ..default()
        }
        .key(self.key)
        .children(children))
    }
}
fn unique(values: impl Iterator<Item = i64>) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(Error::InvalidView("duplicate choice value"));
        }
    }
    Ok(())
}
fn tab_stop(options: impl IntoIterator<Item = (i64, bool)>, selected: i64) -> Option<i64> {
    let mut first = None;
    for (value, enabled) in options {
        if enabled {
            first.get_or_insert(value);
            if value == selected {
                return Some(value);
            }
        }
    }
    first
}
fn choice(
    key: Key,
    state: State<i64>,
    value: i64,
    label: &str,
    enabled: bool,
    semantics: s::Semantics,
) -> View {
    let mut view = Button {
        enabled,
        variant: if semantics.selected != 0 && semantics.role != s::RADIO {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        },
        ..Button::label(label).key(key)
    }
    .on_click(move |cx| state.set(cx, value));
    view.node.semantics = semantics;
    view
}
