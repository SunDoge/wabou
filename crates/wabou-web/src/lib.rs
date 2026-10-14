//! Browser host: existing Solid binary protocol → shared Taffy/Parley/Vello.
#![cfg(target_arch = "wasm32")]
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use taffy::{NodeId, Style, TaffyTree};
use wabou_protocol::{AtomPool, NodeKey, Op, StyleValue, decode_frame};
use wabou_render_core::{
    Scene, TextContext,
    layout::{PlacedNode, compute_and_walk_with_scroll},
    motion::MotionSystem,
    scene::build_scene_scaled,
    style::{DeclaredPaint, HostPaint, InheritedPaint, Paint, apply_ir, parse_ir_value},
    web::WebRenderer,
};
use wabou_style::{IrColor, IrLength, IrValue};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[derive(Default)]
struct Node {
    children: Vec<NodeKey>,
    classes: Vec<String>,
    style: HashMap<String, IrValue>,
    text: Option<Arc<str>>,
    events: Vec<u8>,
    transform: Option<[f32; 6]>,
    text_behavior: u8,
    text_max_lines: u32,
    svg: Option<Arc<wabou_render_core::svg::SvgImage>>,
    svg_color_source: Option<Arc<str>>,
    svg_color: Option<vello_common::peniko::Color>,
    vector_path: Option<Arc<wabou_render_core::style::VectorPath>>,
    shadows: Vec<wabou_render_core::style::Shadow>,
    attributes: HashMap<String, String>,
    scroll: [f32; 2],
}

/// One browser preview, with its own retained tree, fonts and GPU resources.
#[wasm_bindgen]
pub struct BrowserRuntime {
    renderer: WebRenderer,
    atoms: AtomPool,
    stylesheet: wabou_style::stylesheet::StyleSheet,
    nodes: HashMap<NodeKey, Node>,
    text: TextContext,
    placed: Vec<PlacedNode>,
    ids: HashMap<NodeId, NodeKey>,
    motions: MotionSystem<NodeKey>,
    value_motions: wabou_motion::ValueMotionStore,
    last_motion_time_ms: Option<f64>,
}
fn err(message: impl ToString) -> JsValue {
    JsValue::from_str(&message.to_string())
}
fn value(v: StyleValue) -> IrValue {
    match v {
        StyleValue::Px(value) => IrValue::Length {
            value: IrLength::Px { value },
        },
        StyleValue::Percent(value) => IrValue::Length {
            value: IrLength::Percent { value },
        },
        StyleValue::Number(value) => IrValue::Number { value },
        StyleValue::Boolean(value) => IrValue::Boolean { value },
        StyleValue::Color(rgba) => IrValue::Color {
            value: IrColor::Literal { rgba },
        },
        StyleValue::Auto => IrValue::Length {
            value: IrLength::Auto,
        },
    }
}
#[wasm_bindgen]
impl BrowserRuntime {
    /// Create a WebGL2 canvas host. JS remains in the browser.
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let mut nodes = HashMap::new();
        nodes.insert(NodeKey::ROOT, Node::default());
        Ok(Self {
            renderer: WebRenderer::new(canvas).map_err(err)?,
            atoms: AtomPool::default(),
            stylesheet: wabou_style::stylesheet::StyleSheet::default(),
            nodes,
            text: TextContext::new(),
            placed: Vec::new(),
            ids: HashMap::new(),
            motions: MotionSystem::default(),
            value_motions: wabou_motion::ValueMotionStore::default(),
            last_motion_time_ms: None,
        })
    }
    /// Runtime-scoped protocol atom interning, called by the shared Writer.
    pub fn intern(&mut self, text: &str) -> u32 {
        self.atoms.intern(text).get()
    }
    /// Create a runtime-owned scalar animation curve.
    pub fn motion_create(&mut self, descriptor: &str) -> Result<u32, JsValue> {
        self.value_motions
            .create_json(descriptor)
            .map(wabou_motion::ValueMotionId::into_raw)
            .map_err(err)
    }
    /// Sample one scalar curve at elapsed seconds.
    pub fn motion_sample(&self, id: u32, elapsed: f64, output: &mut [f64]) -> bool {
        let Some(sample) = self
            .value_motions
            .sample(wabou_motion::ValueMotionId::from_raw(id), elapsed)
        else {
            return false;
        };
        if output.len() != 3 {
            return false;
        }
        output.copy_from_slice(&[sample.value, sample.velocity, f64::from(sample.done)]);
        true
    }
    /// Release a scalar animation curve.
    pub fn motion_drop(&mut self, id: u32) -> bool {
        self.value_motions
            .remove(wabou_motion::ValueMotionId::from_raw(id))
    }
    /// Install compiler-produced Style IR; shared class cascade handles utilities.
    pub fn set_stylesheet(&mut self, json: &str) -> Result<(), JsValue> {
        let sheet: wabou_style::stylesheet::StyleSheet = serde_json::from_str(json).map_err(err)?;
        sheet.validate().map_err(err)?;
        self.stylesheet = sheet;
        Ok(())
    }
    /// Register a TTF/OTF font. Browser CSS fonts are not visible to Parley.
    pub fn load_font(&mut self, bytes: Vec<u8>) {
        self.text.load_font(bytes);
    }
    /// Apply the shared binary protocol. Unsupported operations fail visibly.
    pub fn apply(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let frame = decode_frame(bytes).map_err(err)?;
        for op in frame.ops {
            match op {
                Op::CreateElement { id, tag } => {
                    let tag = self
                        .atoms
                        .resolve(tag)
                        .ok_or_else(|| err("unknown tag atom"))?;
                    if !matches!(tag, "view" | "text" | "button" | "svg" | "vector-path") {
                        return Err(err(format!("browser widget not implemented: {tag}")));
                    }
                    if self.nodes.insert(id, Node::default()).is_some() {
                        return Err(err("duplicate node"));
                    }
                }
                Op::CreateText { id, text } => {
                    self.nodes.insert(
                        id,
                        Node {
                            text: Some(Arc::from(text)),
                            ..Default::default()
                        },
                    );
                }
                Op::AppendChild { parent, child } => {
                    for node in self.nodes.values_mut() {
                        node.children.retain(|key| *key != child);
                    }
                    self.node(parent)?.children.push(child);
                }
                Op::InsertBefore {
                    parent,
                    child,
                    ref_id,
                } => {
                    for node in self.nodes.values_mut() {
                        node.children.retain(|key| *key != child);
                    }
                    let node = self.node(parent)?;
                    let index = node
                        .children
                        .iter()
                        .position(|key| *key == ref_id)
                        .ok_or_else(|| err("missing reference child"))?;
                    node.children.insert(index, child);
                }
                Op::RemoveChild { parent, child } => {
                    self.node(parent)?.children.retain(|key| *key != child);
                }
                Op::DropNode { id } => {
                    self.nodes.remove(&id);
                    self.motions.remove(id);
                }
                Op::SetText { id, text } => self.node(id)?.text = Some(Arc::from(text)),
                Op::SetStyle { id, prop, value } => {
                    let prop = self
                        .atoms
                        .resolve(prop)
                        .ok_or_else(|| err("unknown style atom"))?
                        .to_owned();
                    self.node(id)?.style.insert(prop, parse_ir_value(value));
                }
                Op::SetStyleValue { id, prop, value: v } => {
                    let prop = self
                        .atoms
                        .resolve(prop)
                        .ok_or_else(|| err("unknown style atom"))?
                        .to_owned();
                    self.node(id)?.style.insert(prop, value(v));
                }
                Op::RemoveStyle { id, prop } => {
                    let prop = self
                        .atoms
                        .resolve(prop)
                        .ok_or_else(|| err("unknown style atom"))?
                        .to_owned();
                    self.node(id)?.style.remove(&prop);
                }
                Op::AddEventListener { id, event_type } => self.node(id)?.events.push(event_type),
                Op::RemoveEventListener { id, event_type } => {
                    self.node(id)?.events.retain(|v| *v != event_type)
                }
                Op::SetGraphicSource {
                    id,
                    kind: wabou_protocol::GRAPHIC_SOURCE_SVG,
                    source,
                } => {
                    let image = wabou_render_core::svg::SvgImage::parse(source).map_err(err)?;
                    let node = self.node(id)?;
                    node.svg = Some(Arc::new(image));
                    node.svg_color_source =
                        source.contains("currentColor").then(|| Arc::from(source));
                    node.svg_color = None;
                }
                Op::ClearGraphicSource {
                    id,
                    kind: wabou_protocol::GRAPHIC_SOURCE_SVG,
                } => {
                    let node = self.node(id)?;
                    node.svg = None;
                    node.svg_color_source = None;
                    node.svg_color = None;
                }
                Op::SetGraphicData {
                    id,
                    kind: wabou_protocol::GRAPHIC_DATA_VECTOR_PATH,
                    data,
                } => {
                    let path = wabou_render_core::vector_path::decode_vector_path(data)
                        .ok_or_else(|| err("invalid vector path"))?;
                    self.node(id)?.vector_path = Some(path);
                }
                Op::ClearGraphicData {
                    id,
                    kind: wabou_protocol::GRAPHIC_DATA_VECTOR_PATH,
                } => {
                    self.node(id)?.vector_path = None;
                }
                Op::SetTransform2D { id, matrix } => self.node(id)?.transform = Some(matrix),
                Op::SetAttribute { id, name, value } => {
                    let name = self
                        .atoms
                        .resolve(name)
                        .ok_or_else(|| err("unknown attribute"))?
                        .to_owned();
                    if name == "__wabou_native_motion" {
                        self.motions.set_json(id, value).map_err(err)?;
                        self.last_motion_time_ms = None;
                    }
                    self.node(id)?.attributes.insert(name, value.to_owned());
                }
                Op::RemoveAttribute { id, name } => {
                    let name = self
                        .atoms
                        .resolve(name)
                        .ok_or_else(|| err("unknown attribute"))?
                        .to_owned();
                    if name == "__wabou_native_motion" {
                        self.motions.remove(id);
                    }
                    self.node(id)?.attributes.remove(&name);
                }
                Op::SetShadows { id, shadows } => {
                    self.node(id)?.shadows = shadows
                        .into_iter()
                        .map(|s| wabou_render_core::style::Shadow {
                            offset_x: s.offset_x,
                            offset_y: s.offset_y,
                            spread: s.spread,
                            std_dev: s.std_dev,
                            radius: s.radius,
                            color: vello_common::peniko::Color::from_rgba8(
                                (s.color >> 24) as u8,
                                (s.color >> 16) as u8,
                                (s.color >> 8) as u8,
                                s.color as u8,
                            ),
                        })
                        .collect();
                }
                Op::ScrollTo { id, x, y } => self.node(id)?.scroll = [x, y],
                Op::ScrollBy { id, x, y } => {
                    let node = self.node(id)?;
                    node.scroll[0] += x;
                    node.scroll[1] += y;
                }
                Op::SetClassName { id, classes } => {
                    let classes = classes
                        .into_iter()
                        .map(|atom| {
                            self.atoms
                                .resolve(atom)
                                .map(str::to_owned)
                                .ok_or_else(|| err("unknown class atom"))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    self.node(id)?.classes = classes;
                }
                Op::SetTextBehavior { id, flags }
                    if flags & wabou_protocol::TEXT_BEHAVIOR_AGGREGATE_STYLED == 0 =>
                {
                    self.node(id)?.text_behavior = flags;
                }
                Op::SetTextMaxLines { id, max_lines } => self.node(id)?.text_max_lines = max_lines,
                // Focus metadata is accepted for shared components; browser keyboard focus
                // is not implemented yet. Blocking/containment still fail explicitly.
                Op::SetInteractionPolicy { flags, .. }
                    if flags & !wabou_protocol::INTERACTION_POLICY_FOCUSABLE == 0 => {}
                other => {
                    return Err(err(format!(
                        "browser protocol operation not implemented: {other:?}"
                    )));
                }
            }
        }
        Ok(())
    }
    /// Advance renderer-owned motion and return completion events for JS.
    pub fn tick_motion(&mut self, time_ms: f64) -> Result<String, JsValue> {
        if !time_ms.is_finite() || time_ms < 0.0 {
            return Err(err("invalid animation frame timestamp"));
        }
        let elapsed = self
            .last_motion_time_ms
            .replace(time_ms)
            .map_or(Duration::ZERO, |previous| {
                Duration::from_secs_f64(((time_ms - previous).max(0.0)) / 1_000.0)
            });
        let completed = self
            .motions
            .advance(elapsed)
            .into_iter()
            .map(|completion| {
                serde_json::json!({
                    "lo": completion.node.lo,
                    "hi": completion.node.hi,
                    "generation": completion.generation,
                })
            })
            .collect::<Vec<_>>();
        Ok(serde_json::json!({
            "active": self.motions.is_active(),
            "completed": completed,
        })
        .to_string())
    }
    /// Layout in logical pixels, render at the requested device scale.
    pub fn render(&mut self, width: f32, height: f32, scale: f64) -> Result<(), JsValue> {
        if !width.is_finite()
            || !height.is_finite()
            || !scale.is_finite()
            || width <= 0.0
            || height <= 0.0
            || scale <= 0.0
        {
            return Err(err("invalid viewport"));
        }
        let mut tree = TaffyTree::new();
        self.ids.clear();
        let root = self.build(
            &mut tree,
            NodeKey::ROOT,
            &InheritedPaint::default(),
            &mut Vec::new(),
        )?;
        let mut style = tree.style(root).map_err(err)?.clone();
        style.size = taffy::Size {
            width: taffy::Dimension::length(width),
            height: taffy::Dimension::length(height),
        };
        tree.set_style(root, style).map_err(err)?;
        let offsets = self
            .ids
            .iter()
            .filter_map(|(id, key)| {
                let node = self.nodes.get(key)?;
                (node.scroll != [0.0, 0.0]).then_some((*id, node.scroll))
            })
            .collect();
        self.placed =
            compute_and_walk_with_scroll(&mut tree, root, width, height, &mut self.text, &offsets);
        let mut clamped = false;
        for placed in &self.placed {
            if let Some(key) = self.ids.get(&placed.node_id) {
                let node = self.nodes.get_mut(key).unwrap();
                for axis in 0..2 {
                    let offset = node.scroll[axis].clamp(0.0, placed.scroll.range[axis].max(0.0));
                    clamped |= offset != node.scroll[axis];
                    node.scroll[axis] = offset;
                }
            }
        }
        if clamped {
            let offsets = self
                .ids
                .iter()
                .map(|(id, key)| (*id, self.nodes[key].scroll))
                .collect();
            self.placed = compute_and_walk_with_scroll(
                &mut tree,
                root,
                width,
                height,
                &mut self.text,
                &offsets,
            );
        }
        let mut scene = Scene::new();
        build_scene_scaled(
            &mut scene,
            &self.placed,
            &mut self.text,
            width.ceil() as u32,
            height.ceil() as u32,
            vello_common::peniko::Color::TRANSPARENT,
            scale,
        );
        self.renderer
            .render(
                &scene,
                (f64::from(width) * scale).round() as u32,
                (f64::from(height) * scale).round() as u32,
            )
            .map_err(err)
    }
    /// Scroll the deepest vertical viewport under a browser wheel event.
    pub fn scroll(&mut self, x: f32, y: f32, delta: f32) -> bool {
        if !x.is_finite() || !y.is_finite() || !delta.is_finite() {
            return false;
        }
        for placed in self.placed.iter().rev() {
            let [x0, y0, x1, y1] = placed.rect;
            if x < x0 || x >= x1 || y < y0 || y >= y1 || !placed.scroll.scrollable[1] {
                continue;
            }
            if placed
                .clip
                .is_some_and(|[x0, y0, x1, y1]| x < x0 || x >= x1 || y < y0 || y >= y1)
            {
                continue;
            }
            let Some(key) = self.ids.get(&placed.node_id) else {
                continue;
            };
            let node = self.nodes.get_mut(key).unwrap();
            let next = (node.scroll[1] + delta).clamp(0.0, placed.scroll.range[1].max(0.0));
            if next != node.scroll[1] {
                node.scroll[1] = next;
                return true;
            }
        }
        false
    }
    /// Find the deepest visible node with an event listener.
    pub fn hit_test(&self, x: f32, y: f32, event: u8) -> String {
        self.placed
            .iter()
            .rev()
            .find_map(|node| {
                let [x0, y0, x1, y1] = node.rect;
                if x < x0 || x >= x1 || y < y0 || y >= y1 {
                    return None;
                }
                if node
                    .clip
                    .is_some_and(|[x0, y0, x1, y1]| x < x0 || x >= x1 || y < y0 || y >= y1)
                {
                    return None;
                }
                let key = self.ids.get(&node.node_id)?;
                if !self.nodes.get(key)?.events.contains(&event) {
                    return None;
                }
                Some(format!("{},{}", key.lo, key.hi))
            })
            .unwrap_or_default()
    }
    /// Inspect exact Rust layout and text in browser tests.
    pub fn snapshot(&self) -> String {
        serde_json::to_string(&self.placed.iter().map(|node| serde_json::json!({ "id": self.ids.get(&node.node_id), "rect": node.rect, "text": node.paint.text.as_deref(), "attributes": self.ids.get(&node.node_id).and_then(|key| self.nodes.get(key)).map(|node| &node.attributes), "scroll": node.scroll.offset, "scrollRange": node.scroll.range, "graphic": if node.paint.vector_path.is_some() { Some("path") } else if node.paint.svg.is_some() { Some("svg") } else { None } })).collect::<Vec<_>>()).unwrap()
    }
}
impl BrowserRuntime {
    fn node(&mut self, id: NodeKey) -> Result<&mut Node, JsValue> {
        self.nodes.get_mut(&id).ok_or_else(|| err("unknown node"))
    }
    fn build(
        &mut self,
        tree: &mut TaffyTree<Paint>,
        key: NodeKey,
        inherited: &InheritedPaint,
        stack: &mut Vec<NodeKey>,
    ) -> Result<NodeId, JsValue> {
        if stack.contains(&key) || stack.len() >= 256 {
            return Err(err("cycle or excessive tree depth"));
        }
        stack.push(key);
        let node = self.nodes.get(&key).ok_or_else(|| err("dangling child"))?;
        let mut style = Style {
            display: taffy::Display::Block,
            ..Style::default()
        };
        let mut declared = DeclaredPaint::default();
        let classes: Vec<_> = node.classes.iter().map(String::as_str).collect();
        let cascade = wabou_style::stylesheet::resolve_classes(&self.stylesheet, &classes);
        if !cascade.diagnostics.is_empty() {
            return Err(err(cascade.diagnostics.join("\n")));
        }
        for declaration in cascade.declarations {
            if !apply_ir(
                &mut style,
                &mut declared,
                &declaration.property,
                &declaration.value,
            ) {
                return Err(err(format!(
                    "unsupported class style: {}",
                    declaration.property
                )));
            }
        }
        for (property, value) in &node.style {
            if !apply_ir(&mut style, &mut declared, property, value) {
                return Err(err(format!("unsupported style: {property}")));
            }
        }
        declared.shadows = node.shadows.clone();
        if node.text_behavior & wabou_protocol::TEXT_BEHAVIOR_SINGLE_LINE != 0 {
            declared.wrap_text = Some(false);
            style.flex_shrink = 0.0;
        }
        let aggregate = node.text_behavior & wabou_protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT != 0;
        let mut text = node.text.clone();
        let children = if aggregate {
            let mut combined = node.text.as_deref().unwrap_or_default().to_owned();
            let mut children = Vec::new();
            for child in &node.children {
                let child_node = self.nodes.get(child).ok_or_else(|| err("dangling child"))?;
                if let Some(value) = &child_node.text {
                    combined.push_str(value);
                } else {
                    children.push(*child);
                }
            }
            text = Some(Arc::from(combined));
            children
        } else {
            node.children.clone()
        };
        let next_inherited = declared.resolve_inherited(inherited);
        let svg = if let Some(source) = &node.svg_color_source {
            if node.svg_color == Some(next_inherited.text_color) {
                node.svg.clone()
            } else {
                let color = format!("{:x}", next_inherited.text_color.to_rgba8());
                Some(Arc::new(
                    wabou_render_core::svg::SvgImage::parse(
                        &source.replace("currentColor", &color),
                    )
                    .map_err(err)?,
                ))
            }
        } else {
            node.svg.clone()
        };
        let motion = self.motions.sample(key);
        let paint = declared.resolve(
            inherited,
            HostPaint {
                text,
                text_max_lines: node.text_max_lines,
                runtime_transform: node.transform,
                motion_transform: motion.transform,
                motion_opacity: motion.opacity,
                svg: svg.clone(),
                vector_path: node.vector_path.clone(),
                intrinsic_size: node.svg.as_ref().map(|svg| svg.size()),
                ..Default::default()
            },
        );
        let node = self.node(key)?;
        node.svg = svg;
        node.svg_color = Some(next_inherited.text_color);
        let children = children
            .into_iter()
            .map(|child| self.build(tree, child, &next_inherited, stack))
            .collect::<Result<Vec<_>, _>>()?;
        let id = tree.new_with_children(style, &children).map_err(err)?;
        tree.set_node_context(id, Some(paint)).map_err(err)?;
        self.ids.insert(id, key);
        stack.pop();
        Ok(id)
    }
}
