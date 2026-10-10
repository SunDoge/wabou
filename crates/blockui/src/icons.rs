//! Monochrome SVG icons, independent of a particular icon library.
use super::*;

/// Statically embedded SVG. Only referenced resources need to be linked.
#[derive(Clone, Copy, Debug)]
pub struct IconSource(&'static str);
impl IconSource {
    pub const fn svg(svg: &'static str) -> Self {
        Self(svg)
    }
    pub(super) fn data(self, size: f32, only: bool) -> abi::icon::IconData {
        abi::icon::IconData {
            svg: self.0.into(),
            size,
            stroke_width: 0.0,
            only,
        }
    }
}
/// Decorative by default. Use `label` to expose an informative icon to accessibility.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Icon {
    pub key: Key,
    source: IconSource,
    size: f32,
    stroke_width: f32,
    label: String,
}
impl From<IconSource> for Icon {
    fn from(source: IconSource) -> Self {
        Self::new(source)
    }
}
impl Icon {
    pub(super) fn data(self, only: bool) -> abi::icon::IconData {
        let mut data = self.source.data(self.size, only);
        data.stroke_width = self.stroke_width;
        data
    }
    pub fn new(source: IconSource) -> Self {
        Self {
            key: Key::default(),
            source,
            size: 16.0,
            stroke_width: 0.0,
            label: String::new(),
        }
    }
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    /// Override stroke width in SVG viewBox units, before scaling.
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}
impl IntoView for Icon {
    fn into_view(self) -> View {
        let mut data = self.source.data(self.size, true);
        data.stroke_width = self.stroke_width;
        View {
            reactive_style: None,
            icon: Some(data),
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node: node(
                if self.key.0 == 0 {
                    Key::named("identity.auto.icon").scoped(abi::key(self.source.0))
                } else {
                    self.key
                },
                abi::ICON,
                self.size,
            ),
            text: self.label.into(),
            children: Vec::new(),
        }
    }
}
/// A button whose visible content is an icon. Its label is the accessible name.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct IconButton {
    pub key: Key,
    pub variant: ButtonVariant,
    pub enabled: bool,
    icon: Icon,
    action: Action,
    label: String,
}
impl IconButton {
    pub fn new(source: IconSource, action: Action) -> Self {
        Self {
            key: Key::default(),
            variant: ButtonVariant::Ghost,
            enabled: true,
            icon: Icon::new(source),
            action,
            label: String::new(),
        }
    }
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.icon.size = size;
        self
    }
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.icon.stroke_width = width;
        self
    }
    pub fn on_click(
        self,
        callback: impl for<'cx, 'host> Fn(&'cx Cx<'host>) -> Result<()> + 'static,
    ) -> View {
        let mut view = self.into_view();
        view.event = Some(std::rc::Rc::new(move |cx, _| callback(cx)));
        view
    }
}
impl IntoView for IconButton {
    fn into_view(self) -> View {
        let mut view = Button {
            key: self.key,
            variant: self.variant,
            enabled: self.enabled,
            ..Button::new(self.label, self.action)
        }
        .into_view();
        let mut data = self.icon.source.data(self.icon.size, true);
        data.stroke_width = self.icon.stroke_width;
        view.icon = Some(data);
        view
    }
}
