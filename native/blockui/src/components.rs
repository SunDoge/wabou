//! Shared native component recipes. All output is ordinary View data.
use crate::*;

/// A panel with shared spacing and rounded native chrome.
/// Stable keys and child identities follow the same rules as Column.
/// ```
/// use blockui::*;
/// let card = Card::default().children(vec![
///     Heading::new("Account").into_view(),
///     Description::new("Changes are saved locally.").into_view(),
/// ]);
/// ```
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Card {
    pub key: Key,
    pub padding: f32,
    pub gap: f32,
    pub width: f32,
    pub max_width: f32,
}
impl Default for Card {
    fn default() -> Self {
        Self {
            key: Key::default(),
            padding: 24.0,
            gap: 12.0,
            width: 0.0,
            max_width: 0.0,
        }
    }
}
impl Card {
    pub fn children(self, children: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children = children;
        view
    }
}
impl IntoView for Card {
    fn into_view(self) -> View {
        Column {
            key: self.key,
            padding: self.padding,
            gap: self.gap,
        }
        .style(ViewStyle {
            width: self.width,
            max_width: self.max_width,
            surface: Surface::Panel,
            radius: 12.0,
            ..default()
        })
    }
}

/// A page or section heading. Reactive text keeps its ordinary binding identity.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Heading(Text, u32);
impl Heading {
    pub fn new(text: impl Into<Binding>) -> Self {
        Self(Text::new(text), 2)
    }
    /// Heading level is validated as 1..=6 at view admission.
    pub fn level(mut self, level: u32) -> Self {
        self.1 = level;
        self
    }
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.0 = self.0.key(key);
        self
    }
}
impl IntoView for Heading {
    fn into_view(self) -> View {
        let mut view = self.0.style(ViewStyle {
            font_size: 24.0,
            ..default()
        });
        view.node.semantics = abi::semantics::Semantics {
            role: abi::semantics::HEADING,
            level: self.1,
            ..default()
        };
        view
    }
}

/// Readable supporting copy with the shared muted text role.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Description(Text);
impl Description {
    pub fn new(text: impl Into<Binding>) -> Self {
        Self(Text::new(text))
    }
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.0 = self.0.key(key);
        self
    }
}
impl IntoView for Description {
    fn into_view(self) -> View {
        self.0.style(ViewStyle {
            font_size: 13.0,
            text_tone: TextTone::Muted,
            ..default()
        })
    }
}

/// Inline validation feedback. Empty text collapses without losing binding identity.
/// Pair the message with an actionable explanation rather than color alone.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct ValidationMessage(Text);
impl ValidationMessage {
    pub fn new(text: impl Into<Binding>) -> Self {
        Self(Text::new(text))
    }
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.0 = self.0.key(key);
        self
    }
}
impl IntoView for ValidationMessage {
    fn into_view(self) -> View {
        self.0.style(ViewStyle {
            text_tone: TextTone::Error,
            ..default()
        })
    }
}

/// Related actions with consistent spacing. Children share available row width.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct ActionGroup {
    pub key: Key,
    pub gap: f32,
}
impl Default for ActionGroup {
    fn default() -> Self {
        Self {
            key: Key::default(),
            gap: 8.0,
        }
    }
}
impl ActionGroup {
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    pub fn children(self, children: Vec<View>) -> View {
        Row {
            key: self.key,
            padding: 0.0,
            gap: self.gap,
        }
        .children(children)
    }
}
impl IntoView for ActionGroup {
    fn into_view(self) -> View {
        Row {
            key: self.key,
            padding: 0.0,
            gap: self.gap,
        }
        .into_view()
    }
}

/// Persistent feedback with a title and an actionable explanation.
/// Uses the shared text tone on both lines; the title remains explicit so that
/// information is never conveyed through color alone.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Alert {
    #[setters(generate, into)]
    pub key: Key,
    pub title: String,
    pub description: String,
    #[setters(generate)]
    pub tone: TextTone,
}
impl Alert {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            key: Key::named("component.alert").scoped(abi::key(&title)),
            title,
            description: description.into(),
            tone: TextTone::Default,
        }
    }
}
impl IntoView for Alert {
    fn into_view(self) -> View {
        let mut view = Card::default().key(self.key).children(vec![
            Text::new(self.title)
                .key(self.key.scoped(abi::key("title")))
                .style(ViewStyle {
                    font_size: 16.0,
                    text_tone: self.tone,
                    ..default()
                }),
            Text::new(self.description)
                .key(self.key.scoped(abi::key("description")))
                .style(ViewStyle {
                    font_size: 13.0,
                    text_tone: self.tone,
                    ..default()
                }),
        ]);
        view.node.semantics.role = abi::semantics::ALERT;
        view.node.semantics.labelled_by = self.key.scoped(abi::key("title")).0;
        view
    }
}
/// An empty collection with a clear next step. Add an ordinary Button through
/// `children` so its action remains owned by the current implementation.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct EmptyState {
    #[setters(generate, into)]
    pub key: Key,
    pub title: String,
    pub description: String,
}
impl EmptyState {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            key: Key::named("component.empty").scoped(abi::key(&title)),
            title,
            description: description.into(),
        }
    }
    pub fn children(self, actions: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children.extend(actions);
        view
    }
}
impl IntoView for EmptyState {
    fn into_view(self) -> View {
        Card::default().key(self.key).children(vec![
            Heading::new(self.title)
                .key(self.key.scoped(abi::key("title")))
                .into_view(),
            Description::new(self.description)
                .key(self.key.scoped(abi::key("description")))
                .into_view(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_recipes_preserve_binding_and_explicit_identity() {
        let state = State::<String>::new("component.message");
        let baseline = Text::new(state).key("message").into_view();
        for view in [
            Heading::new(state).key("message").into_view(),
            Description::new(state).key("message").into_view(),
            ValidationMessage::new(state).key("message").into_view(),
        ] {
            assert_eq!(view.node.key, baseline.node.key);
            let Binding::MappedText { slot, .. } = view.text else {
                panic!("lost state binding")
            };
            assert_eq!(slot, state.spec.key);
            assert!(view.explicit_identity);
        }
    }
    #[test]
    fn containers_keep_children_keys_and_shared_geometry() {
        let group = ActionGroup::default().gap(12.0).children(vec![
            Text::new("Save").key("save").into_view(),
            Text::new("Cancel").key("cancel").into_view(),
        ]);
        assert_eq!(group.node.kind, abi::ROW);
        assert_eq!(group.node.gap, 12.0);
        assert_eq!(group.node.padding, 0.0);
        let card = Card::default().children(vec![group]);
        assert_eq!(card.node.kind, abi::COLUMN);
        assert_eq!(card.node.presentation.surface, 2);
        assert_eq!(card.node.presentation.radius, 12.0);
        assert_eq!(card.children[0].children[0].node.key, Key::named("save").0);
    }
}
