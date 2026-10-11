//! Host-owned, bounded vector geometry; theme changes never parse XML.
use super::*;
use blockui_abi::icon::IconData;
use shell::svg::usvg;
use vello_common::{
    kurbo::{Affine, BezPath, Cap, Join, Stroke},
    peniko::Fill,
};

struct Shape {
    path: BezPath,
    transform: Affine,
    stroke: Option<Stroke>,
    fill: Option<Fill>,
}
pub(super) struct CachedIcon {
    data: Arc<IconData>,
    size: [f64; 2],
    shapes: Vec<Shape>,
}
pub(crate) fn validate(data: &IconData) -> Result<(), String> {
    CachedIcon::parse(Arc::new(data.clone())).map(|_| ())
}
impl CachedIcon {
    fn parse(data: Arc<IconData>) -> Result<Self, String> {
        let tree = usvg::Tree::from_str(&data.svg, &usvg::Options::default())
            .map_err(|e| format!("invalid icon SVG: {e}"))?;
        let mut shapes = Vec::new();
        fn visit(group: &usvg::Group, shapes: &mut Vec<Shape>, width: f32) -> Result<(), String> {
            if group.clip_path().is_some()
                || group.mask().is_some()
                || !group.filters().is_empty()
                || group.opacity().get() != 1.0
            {
                return Err(
                    "monochrome icons do not support clipping, masks, filters or group opacity"
                        .into(),
                );
            }
            for node in group.children() {
                match node {
                    usvg::Node::Group(group) => visit(group, shapes, width)?,
                    usvg::Node::Path(path) if path.is_visible() => {
                        if shapes.len() >= 512 {
                            return Err("icon exceeds path budget".into());
                        }
                        if path.data().len() > 8192 {
                            return Err("icon exceeds segment budget".into());
                        }
                        let mut geometry = BezPath::new();
                        let mut start = (0.0, 0.0);
                        let mut closed = false;
                        for segment in path.data().segments() {
                            use usvg::tiny_skia_path::PathSegment::*;
                            let point =
                                |p: usvg::tiny_skia_path::Point| (f64::from(p.x), f64::from(p.y));
                            if closed && !matches!(segment, MoveTo(_) | Close) {
                                geometry.move_to(start);
                                closed = false;
                            }
                            match segment {
                                MoveTo(p) => {
                                    start = point(p);
                                    closed = false;
                                    geometry.move_to(start);
                                }
                                LineTo(p) => geometry.line_to(point(p)),
                                QuadTo(c, p) => geometry.quad_to(point(c), point(p)),
                                CubicTo(a, b, p) => geometry.curve_to(point(a), point(b), point(p)),
                                Close => {
                                    geometry.close_path();
                                    closed = true;
                                }
                            }
                        }
                        let t = path.abs_transform();
                        let transform =
                            Affine::new([t.sx, t.ky, t.kx, t.sy, t.tx, t.ty].map(f64::from));
                        let stroke = path.stroke().map(|s| {
                            let mut stroke = Stroke::new(f64::from(if width == 0.0 {
                                s.width().get()
                            } else {
                                width
                            }))
                            .with_caps(match s.linecap() {
                                usvg::LineCap::Butt => Cap::Butt,
                                usvg::LineCap::Round => Cap::Round,
                                usvg::LineCap::Square => Cap::Square,
                            })
                            .with_join(match s.linejoin() {
                                usvg::LineJoin::Round => Join::Round,
                                usvg::LineJoin::Bevel => Join::Bevel,
                                _ => Join::Miter,
                            })
                            .with_miter_limit(f64::from(s.miterlimit().get()));
                            if let Some(dashes) = s.dasharray() {
                                stroke = stroke.with_dashes(
                                    f64::from(s.dashoffset()),
                                    dashes.iter().copied().map(f64::from),
                                );
                            }
                            stroke
                        });
                        let fill = path.fill().map(|f| match f.rule() {
                            usvg::FillRule::EvenOdd => Fill::EvenOdd,
                            usvg::FillRule::NonZero => Fill::NonZero,
                        });
                        shapes.push(Shape {
                            path: geometry,
                            transform,
                            stroke,
                            fill,
                        });
                    }
                    usvg::Node::Path(_) => {}
                    _ => return Err("icons require vector paths, not text or raster images".into()),
                }
            }
            Ok(())
        }
        visit(tree.root(), &mut shapes, data.stroke_width)?;
        if shapes.is_empty() {
            return Err("icon SVG has no visible paths".into());
        }
        Ok(Self {
            data,
            size: [
                f64::from(tree.size().width()),
                f64::from(tree.size().height()),
            ],
            shapes,
        })
    }
}
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn icon_is_applied(&self, key: u64, binding: &crate::Binding) -> bool {
        match (self.icons.get(&key), &binding.icon) {
            (None, None) => true,
            (Some(old), Some(data)) => Arc::ptr_eq(&old.data, data) || *old.data == **data,
            _ => false,
        }
    }
    pub(super) fn sync_icons(&mut self) -> Result<(), String> {
        #[cfg(test)]
        {
            self.icon_syncs += 1;
        }
        for key in self.icons.keys() {
            if self
                .actor
                .view()
                .bindings
                .get(key)
                .is_none_or(|b| b.icon.is_none())
            {
                self.paint_dirty.insert(*key);
            }
        }
        self.icons.retain(|key, _| {
            self.actor
                .view()
                .bindings
                .get(key)
                .is_some_and(|b| b.icon.is_some())
        });
        for (key, binding) in &self.actor.view().bindings {
            let Some(data) = &binding.icon else { continue };
            if !self.icon_is_applied(*key, binding) {
                self.icons.insert(*key, CachedIcon::parse(data.clone())?);
                self.paint_dirty.insert(*key);
            }
        }
        Ok(())
    }
    pub(super) fn style_icon(
        &self,
        binding: &crate::Binding,
        style: &mut Style,
        paint: &mut Paint,
    ) {
        let Some(icon) = &binding.icon else { return };
        let n = binding.node;
        if n.kind == ICON {
            style.align_self = Some(AlignSelf::CENTER);
            style.size = Size {
                width: Dimension::length(icon.size),
                height: Dimension::length(icon.size),
            };
            style.flex_basis = Dimension::auto();
            style.flex_grow = 0.0;
            style.flex_shrink = 0.0;
            paint.text = None;
        } else if icon.only {
            paint.text = None;
            let height = match style.size.height.expand() {
                taffy::style::ExpandedDimension::Length(v) => v,
                _ => n.height,
            };
            let extent = height.max(icon.size + 12.0);
            style.size.width = Dimension::length(extent);
            style.size.height = Dimension::length(extent);
            style.padding = Rect::length(5.0);
            style.flex_basis = Dimension::auto();
            style.flex_grow = 0.0;
            style.flex_shrink = 0.0;
        } else {
            let left = match style.padding.left.expand() {
                taffy::style::ExpandedLengthPercentage::Length(v) => v,
                _ => 8.0,
            };
            style.padding.left = LengthPercentage::length(left + icon.size + 6.0);
        }
    }
    pub(super) fn paint_icons(&self, nodes: &mut [PlacedNode], tcx: &mut TextContext) {
        use shell::PaintScene;
        let by_id: HashMap<_, _> = self
            .icons
            .iter()
            .filter_map(|(key, icon)| self.ids.get(key).map(|id| (*id, icon)))
            .collect();
        for node in nodes {
            let Some(icon) = by_id.get(&node.node_id) else {
                continue;
            };
            let size = f64::from(icon.data.size);
            let x = if icon.data.only {
                (f64::from(node.content_size[0]) - size) * 0.5
            } else {
                -size - 6.0
            };
            let y = (f64::from(node.content_size[1]) - size) * 0.5;
            let scale = (size / icon.size[0]).min(size / icon.size[1]);
            let root = Affine::translate((
                x + (size - icon.size[0] * scale) * 0.5,
                y + (size - icon.size[1] * scale) * 0.5,
            )) * Affine::scale(scale);
            let mut paint = shell::widget::PaintContext::new(
                node.content_size[0],
                node.content_size[1],
                self.scale,
                tcx,
            );
            for shape in &icon.shapes {
                let transform = root * shape.transform;
                if let Some(fill) = shape.fill {
                    paint.scene_mut().fill(
                        fill,
                        transform,
                        node.paint.text_color,
                        None,
                        &shape.path,
                    );
                }
                if let Some(stroke) = &shape.stroke {
                    paint.scene_mut().stroke(
                        stroke,
                        transform,
                        node.paint.text_color,
                        None,
                        &shape.path,
                    );
                }
            }
            node.paint.widget = Some(Arc::new(paint.finish()));
        }
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use blockui::{
        Actor as UiActor, Binding, Button, Column, Cx, Density, Icon, IconButton, IconSource,
        IntoView, State, StateSpec, TextInput, ThemeMode, View, ViewStyle,
    };
    const SEARCH: IconSource =
        IconSource::svg(include_str!("../../blockui-icons-lucide/icons/search.svg"));
    const CHECK: IconSource =
        IconSource::svg(include_str!("../../blockui-icons-lucide/icons/check.svg"));
    const MODE: State<i64> = State::<i64>::new("icons.mode", 1);
    const COUNT: State<i64> = State::<i64>::new("icons.count", 0);
    const DRAFT: State<String> = State::<String>::with_default("icons.draft", "draft 🦀");
    const SET: blockui::Action = blockui::Action::new("icons.set", |cx, value| MODE.set(cx, value));
    const CLICK: blockui::Action =
        blockui::Action::new("icons.click", |cx, _| COUNT.update(cx, |n| Ok(n + 1)));
    fn view(source: IconSource) -> View {
        Column::default()
            .key("icons.root")
            .children(vec![
                Icon::new(source)
                    .key("icons.decorative")
                    .size(18.0)
                    .into_view(),
                Icon::new(source)
                    .key("icons.named")
                    .label("Search")
                    .into_view(),
                Button::new(
                    Binding::reactive(|cx| Ok(format!("Count {}", COUNT.get(cx)?))),
                    CLICK,
                )
                .key("icons.leading")
                .leading_icon(source),
                IconButton::new(source, CLICK)
                    .key("icons.only")
                    .label("Search records")
                    .into_view(),
                TextInput::new(DRAFT).key("icons.editor").into_view(),
            ])
            .style_reactive(|cx| {
                Ok(ViewStyle {
                    theme: if MODE.get(cx)? == 1 {
                        ThemeMode::Light
                    } else {
                        ThemeMode::Dark
                    },
                    density: Density::Compact,
                    ..Default::default()
                })
            })
    }
    struct First;
    impl UiActor for First {
        const STATE: &'static [StateSpec] = &[MODE.spec(), COUNT.spec(), DRAFT.spec()];
        const ACTIONS: &'static [blockui::Action] = &[SET, CLICK];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(view(SEARCH))
        }
    }
    struct Second;
    impl UiActor for Second {
        const STATE: &'static [StateSpec] = First::STATE;
        const ACTIONS: &'static [blockui::Action] = First::ACTIONS;
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(view(CHECK))
        }
    }
    #[test]
    fn queued_icon_change_requests_redraw_with_identical_node_and_text() -> Result<(), String> {
        struct Dynamic;
        impl UiActor for Dynamic {
            const STATE: &'static [StateSpec] = First::STATE;
            const ACTIONS: &'static [blockui::Action] = First::ACTIONS;
            fn view(cx: &Cx<'_>) -> blockui::Result<View> {
                Ok(Column::default().children(vec![
                    Icon::new(if MODE.get(cx)? == 1 { SEARCH } else { CHECK })
                        .key("dynamic.icon")
                        .into_view(),
                ]))
            }
        }
        let mut source = NativeSource::without_watch(Actor::for_test::<Dynamic>()?)?;
        let mut tcx = TextContext::new();
        source.build_frame(&mut tcx, 320, 200);
        let key = key("dynamic.icon");
        let before = source.actor.view.bindings[&key].clone();
        let id = source.ids[&key];
        assert!(!source.poll_async());
        source.actor.tell(SET, 2)?;
        assert!(source.poll_async());
        let after = &source.actor.view.bindings[&key];
        assert_eq!(before.node, after.node);
        assert_eq!(before.text, after.text);
        assert_ne!(before.icon, after.icon);
        source.build_frame(&mut tcx, 320, 200);
        assert_eq!(source.ids[&key], id);
        assert!(source.icon_is_applied(key, &source.actor.view.bindings[&key]));
        assert!(!source.poll_async());
        Ok(())
    }
    struct Invalid;
    impl UiActor for Invalid {
        const STATE: &'static [StateSpec] = First::STATE;
        const ACTIONS: &'static [blockui::Action] = First::ACTIONS;
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(view(IconSource::svg("invalid SVG")))
        }
    }
    #[test]
    fn native_icons_retain_geometry_on_theme_change_and_replace_assets_atomically()
    -> Result<(), String> {
        let mut source = NativeSource::without_watch(Actor::for_test::<First>()?)?;
        let mut tcx = TextContext::new();
        source.set_semantics_enabled(true);
        let light = source.build_frame(&mut tcx, 480, 360);
        let icon_syncs = source.icon_syncs;
        source.sync()?;
        assert_eq!(
            source.icon_syncs, icon_syncs,
            "idle sync must retain the icon cache without a full scan"
        );
        let decorative = key("icons.decorative");
        let only = key("icons.only");
        let leading = key("icons.leading");
        let editor = key("icons.editor");
        let editor_id = source.ids[&editor];
        let editor_instance = &*source.editors[&editor] as *const _;
        source.set_focus(Some(editor));
        let geometry = source.icons[&decorative].shapes.as_ptr();
        let snapshot = source.semantic_snapshot().unwrap();
        assert!(!snapshot.nodes.iter().any(|n| n.id == decorative));
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|n| n.children.contains(&decorative))
        );
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .find(|n| n.id == only)
                .unwrap()
                .label
                .as_deref(),
            Some("Search records")
        );
        assert_eq!(
            snapshot.nodes.iter().find(|n| n.id == only).unwrap().role,
            shell::SemanticRole::Button
        );
        let placed = light
            .iter()
            .find(|n| n.node_id == source.ids[&only])
            .unwrap();
        assert!(placed.paint.widget.is_some());
        assert!(placed.paint.text.is_none());
        assert_eq!(
            placed.rect[2] - placed.rect[0],
            placed.rect[3] - placed.rect[1]
        );
        let label_evaluations = source.actor.view.bindings[&leading].evaluations;
        source.actor.dispatch(Message {
            action: SET.id,
            value: 2,
        })?;
        source.sync()?;
        assert_eq!(source.icons[&decorative].shapes.as_ptr(), geometry);
        assert_eq!(
            source.actor.view.bindings[&leading].evaluations,
            label_evaluations
        );
        let dark = source.build_frame(&mut tcx, 480, 360);
        let color = |nodes: &[PlacedNode]| {
            nodes
                .iter()
                .find(|n| n.node_id == source.ids[&decorative])
                .unwrap()
                .paint
                .text_color
        };
        assert_ne!(color(&light), color(&dark));
        source.actor.dispatch_widget(only)?;
        source.sync()?;
        assert_eq!(source.actor.integer("icons.count"), 1);
        assert_eq!(source.actor.view.bindings[&leading].text, "Count 1");
        assert_eq!(source.icons[&decorative].shapes.as_ptr(), geometry);
        let address = source.actor.state_address();
        assert!(
            source
                .actor
                .replace_for_test::<Invalid>()
                .unwrap_err()
                .contains("invalid icon SVG")
        );
        assert_eq!(source.actor.state_address(), address);
        assert_eq!(source.actor.integer("icons.count"), 1);
        assert!(
            source.actor.view.bindings[&only]
                .icon
                .as_ref()
                .unwrap()
                .svg
                .contains("circle")
        );
        source.actor.replace_for_test::<Second>()?;
        source.build_frame(&mut tcx, 480, 360);
        assert_eq!(source.actor.state_address(), address);
        assert_eq!(source.ids[&editor], editor_id);
        assert_eq!(&*source.editors[&editor] as *const _, editor_instance);
        assert_eq!(source.focus, Some(editor));
        assert_eq!(source.actor.text("icons.draft"), "draft 🦀");
        assert_eq!(source.icons.len(), 4);
        assert!(!source.icons[&only].data.svg.contains("circle"));
        Ok(())
    }
    #[test]
    fn icon_data_only_change_invalidates_idle_sync_guard() -> Result<(), String> {
        let mut source = NativeSource::without_watch(Actor::for_test::<First>()?)?;
        let id = key("icons.decorative");
        let binding = &source.actor.view.bindings[&id];
        let node = binding.node;
        let text = binding.text.clone();
        let mut data = (**binding.icon.as_ref().unwrap()).clone();
        data.svg = include_str!("../../blockui-icons-lucide/icons/check.svg").into();
        let replacement = Arc::new(data);
        source.actor.view.bindings.get_mut(&id).unwrap().icon = Some(replacement.clone());
        let before = source.icon_syncs;
        source.sync()?;
        assert_eq!(source.actor.view.bindings[&id].node, node);
        assert_eq!(source.actor.view.bindings[&id].text, text);
        assert_eq!(source.icon_syncs, before + 1);
        assert!(Arc::ptr_eq(&source.icons[&id].data, &replacement));
        source.sync()?;
        assert_eq!(source.icon_syncs, before + 1);
        Ok(())
    }
    #[test]
    fn all_vendored_lucide_icons_parse_as_native_vectors() -> Result<(), String> {
        for entry in std::fs::read_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../blockui-icons-lucide/icons"
        ))
        .map_err(|e| e.to_string())?
        {
            let path = entry.map_err(|e| e.to_string())?.path();
            let svg = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            validate(&IconData {
                svg,
                size: 16.0,
                stroke_width: 0.0,
                only: false,
            })
            .map_err(|e| format!("{}: {e}", path.display()))?;
        }
        Ok(())
    }
    #[test]
    fn icon_button_requires_an_accessible_name() {
        struct Unnamed;
        impl UiActor for Unnamed {
            const STATE: &'static [StateSpec] = &[];
            const ACTIONS: &'static [blockui::Action] = &[CLICK];
            fn view(_: &Cx<'_>) -> blockui::Result<View> {
                Ok(Column::default()
                    .key("unnamed.root")
                    .children(vec![IconButton::new(SEARCH, CLICK).into_view()]))
            }
        }
        assert!(
            Actor::for_test::<Unnamed>()
                .err()
                .unwrap()
                .contains("accessible label")
        );
    }
}
