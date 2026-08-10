use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum PaintTransform {
    Translate(IrLength, IrLength),
    Scale(f32, f32),
    Rotate(f32),
    Skew(f32, f32),
    Matrix([f32; 6]),
}

impl PaintTransform {
    pub(super) fn from_ir(value: &IrValue) -> Option<Self> {
        let kind = field(value, "kind")?.keyword()?;
        let value = field(value, "value")?;
        let list = match value {
            IrValue::List { values } => Some(values.as_slice()),
            _ => None,
        };
        let length = |v: &IrValue| match v {
            IrValue::Length { value } => Some(value.clone()),
            IrValue::Number { value } if *value == 0.0 => Some(IrLength::Px { value: 0.0 }),
            _ => None,
        };
        match kind {
            "translate" => Some(Self::Translate(length(&list?[0])?, length(&list?[1])?)),
            "translateX" => Some(Self::Translate(length(value)?, IrLength::Px { value: 0.0 })),
            "translateY" => Some(Self::Translate(IrLength::Px { value: 0.0 }, length(value)?)),
            "scale" => Some(Self::Scale(list?[0].number()?, list?[1].number()?)),
            "scaleX" => Some(Self::Scale(value.number()?, 1.0)),
            "scaleY" => Some(Self::Scale(1.0, value.number()?)),
            "rotate" | "rotateZ" => Some(Self::Rotate(value.number()?)),
            "skew" => Some(Self::Skew(list?[0].number()?, list?[1].number()?)),
            "skewX" => Some(Self::Skew(value.number()?, 0.0)),
            "skewY" => Some(Self::Skew(0.0, value.number()?)),
            "matrix" => {
                let v = list?;
                Some(Self::Matrix([
                    v[0].number()?,
                    v[1].number()?,
                    v[2].number()?,
                    v[3].number()?,
                    v[4].number()?,
                    v[5].number()?,
                ]))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub spread: f32,
    pub std_dev: f32,
    pub color: Color,
    pub radius: Option<f32>,
}

impl Shadow {
    pub(super) fn from_ir(value: &IrValue) -> Option<Self> {
        let px = |name| match field(value, name)? {
            IrValue::Length {
                value: IrLength::Px { value },
            } => Some(*value),
            IrValue::Number { value } => Some(*value),
            _ => None,
        };
        Some(Self {
            offset_x: px("x")?,
            offset_y: px("y")?,
            spread: px("spread")?,
            std_dev: px("stdDev")?,
            color: ir_color(field(value, "color")?)?,
            radius: field(value, "radius").and_then(|_| px("radius")),
        })
    }
}

/// Parent-propagated values for CSS-inherited paint properties.
///
/// Used when resolving [`DeclaredPaint`] → [`Paint`]. Seeded at the document
/// root with CSS initial values.
#[derive(Clone, Debug)]
pub struct InheritedPaint {
    pub text_color: Color,
    pub font_size: f32,
    pub font_weight: f32,
    pub line_height: Option<(f32, bool)>,
    pub wrap_text: bool,
    pub text_selectable: bool,
    pub text_select_all: bool,
    pub text_align: TextAlign,
    pub font_family: Option<Arc<str>>,
}

impl Default for InheritedPaint {
    fn default() -> Self {
        Self {
            text_color: Color::BLACK,
            font_size: 16.0,
            font_weight: 400.0,
            line_height: None,
            wrap_text: true, // CSS `white-space: normal`
            text_selectable: true,
            text_select_all: false,
            text_align: TextAlign::Start,
            font_family: None,
        }
    }
}

/// Paint properties as authored after cascade (class rules + inline).
///
/// Inherited fields are `Option`: `None` means the node did **not** declare
/// the property and must inherit from the parent. This is deliberately
/// separate from [`Paint`], which holds only fully resolved values used by
/// layout/render. Do not encode "declared vs default" as dual fields on
/// `Paint` (e.g. `wrap_text` + `wrap_text_declared`).
#[derive(Clone, Debug)]
pub struct DeclaredPaint {
    pub background: Option<Color>,
    pub opacity: f32,
    pub transform: Vec<PaintTransform>,
    pub shadows: Vec<Shadow>,
    /// Uniform corner radius in px.
    pub border_radius: f32,
    /// Uniform border (width px, color).
    pub border: Option<(f32, Color)>,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
    pub font_weight: Option<f32>,
    /// `(value, relative)` where relative means a font-size multiplier.
    pub line_height: Option<(f32, bool)>,
    /// Own `white-space` → wrap mapping. `None` = inherit.
    pub wrap_text: Option<bool>,
    /// Whether overflowing inline text is shortened with an ellipsis.
    pub text_ellipsis: bool,
    /// Own `user-select` mapping. `None` = inherit.
    pub text_selectable: Option<bool>,
    pub text_select_all: Option<bool>,
    pub text_align: Option<TextAlign>,
    pub pointer_events: bool,
    pub z_index: i32,
    pub font_family: Option<Arc<str>>,
}

impl Default for DeclaredPaint {
    fn default() -> Self {
        Self {
            background: None,
            opacity: 1.0,
            transform: Vec::new(),
            shadows: Vec::new(),
            border_radius: 0.0,
            border: None,
            text_color: None,
            font_size: None,
            font_weight: None,
            line_height: None,
            wrap_text: None,
            text_ellipsis: false,
            text_selectable: None,
            text_select_all: None,
            text_align: None,
            pointer_events: true,
            z_index: 0,
            font_family: None,
        }
    }
}

impl DeclaredPaint {
    /// Resolve declared + parent-inherited values into the node's own
    /// contribution to the inheritance chain (still no host content fields).
    pub fn resolve_inherited(&self, parent: &InheritedPaint) -> InheritedPaint {
        InheritedPaint {
            text_color: self.text_color.unwrap_or(parent.text_color),
            font_size: self.font_size.unwrap_or(parent.font_size),
            font_weight: self.font_weight.unwrap_or(parent.font_weight),
            line_height: self.line_height.or(parent.line_height),
            wrap_text: self.wrap_text.unwrap_or(parent.wrap_text),
            text_selectable: self.text_selectable.unwrap_or(parent.text_selectable),
            text_select_all: self.text_select_all.unwrap_or(parent.text_select_all),
            text_align: self.text_align.unwrap_or(parent.text_align),
            font_family: self
                .font_family
                .clone()
                .or_else(|| parent.font_family.clone()),
        }
    }

    /// Build a fully resolved [`Paint`] for layout/render.
    ///
    /// `host` carries non-CSS content (text runs, svg, widget scenes, …) that
    /// the cascade does not own and must survive re-resolution.
    pub fn resolve(&self, parent: &InheritedPaint, host: HostPaint) -> Paint {
        let inherited = self.resolve_inherited(parent);
        Paint {
            background: self.background,
            opacity: self.opacity,
            transform: self.transform.clone(),
            runtime_transform: host.runtime_transform,
            overlay_plane: host.overlay_plane,
            scrollbar: host.scrollbar,
            shadows: self.shadows.clone(),
            border_radius: self.border_radius,
            border: self.border,
            text: host.text,
            text_runs: host.text_runs,
            selection_rects: host.selection_rects,
            text_color: inherited.text_color,
            font_size: inherited.font_size,
            font_weight: inherited.font_weight,
            line_height: inherited.line_height,
            wrap_text: inherited.wrap_text,
            text_ellipsis: self.text_ellipsis,
            text_selectable: inherited.text_selectable,
            text_select_all: inherited.text_select_all,
            text_align: inherited.text_align,
            pointer_events: self.pointer_events,
            z_index: self.z_index,
            font_family: inherited.font_family,
            svg: host.svg,
            widget: host.widget,
            intrinsic_size: host.intrinsic_size,
        }
    }
}

/// Host-owned stacking plane. CSS `z-index` only orders siblings inside one
/// plane; it cannot move content above modal or system UI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum OverlayPlane {
    #[default]
    Content,
    Floating,
    Modal,
    System,
    Debug,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    #[default]
    Auto,
    Always,
    Hidden,
}

/// Host-owned appearance for native overlay scrollbars.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarStyle {
    pub visibility: ScrollbarVisibility,
    pub hide_delay: std::time::Duration,
    pub fade_duration: std::time::Duration,
    pub thickness: f32,
    pub margin: f32,
    pub min_thumb_length: f32,
    /// Negative means half the thumb thickness.
    pub radius: f32,
    pub track_color: Color,
    pub thumb_color: Color,
    pub hover_color: Color,
    pub active_color: Color,
}

impl Default for ScrollbarStyle {
    fn default() -> Self {
        Self {
            visibility: ScrollbarVisibility::Auto,
            hide_delay: std::time::Duration::from_millis(500),
            fade_duration: std::time::Duration::from_millis(200),
            thickness: 10.0,
            margin: 2.0,
            min_thumb_length: 32.0,
            radius: -1.0,
            track_color: Color::TRANSPARENT,
            thumb_color: Color::from_rgba8(100, 116, 139, 190),
            hover_color: Color::from_rgba8(100, 116, 139, 225),
            active_color: Color::from_rgba8(71, 85, 105, 255),
        }
    }
}

/// Host-owned paint content that is not part of the CSS cascade.
#[derive(Clone, Default)]
pub struct HostPaint {
    pub text: Option<Arc<str>>,
    pub text_runs: Arc<[crate::text::TextRun]>,
    pub selection_rects: Arc<[[f32; 4]]>,
    pub svg: Option<Arc<crate::svg::SvgImage>>,
    pub widget: Option<Arc<vello::Scene>>,
    pub intrinsic_size: Option<[f32; 2]>,
    pub runtime_transform: Option<[f32; 6]>,
    pub overlay_plane: OverlayPlane,
    pub scrollbar: ScrollbarStyle,
}

/// Fully resolved paint used by layout and rendering.
///
/// Inherited properties are concrete values — no `Option` meaning "inherit"
/// and no parallel `*_declared` flags. Authored declarations live on
/// [`DeclaredPaint`].
#[derive(Clone)]
pub struct Paint {
    pub background: Option<Color>,
    pub opacity: f32,
    pub transform: Vec<PaintTransform>,
    /// Host-driven state, composed after the static CSS transform.
    pub runtime_transform: Option<[f32; 6]>,
    /// Explicit host stacking plane, ordered before sibling `z-index`.
    pub overlay_plane: OverlayPlane,
    pub scrollbar: ScrollbarStyle,
    pub shadows: Vec<Shadow>,
    /// Uniform corner radius in px.
    pub border_radius: f32,
    /// Uniform border (width px, color).
    pub border: Option<(f32, Color)>,
    pub text: Option<Arc<str>>,
    pub text_runs: Arc<[crate::text::TextRun]>,
    /// Selection highlight rectangles in text-layout-local coordinates.
    pub selection_rects: Arc<[[f32; 4]]>,
    pub text_color: Color,
    pub font_size: f32,
    pub font_weight: f32,
    /// `(value, relative)` where relative means a font-size multiplier.
    /// `None` means CSS `normal` (engine default metrics).
    pub line_height: Option<(f32, bool)>,
    /// Whether normal line wrapping is allowed (resolved `white-space`).
    pub wrap_text: bool,
    /// Whether a constrained single line is shortened with an ellipsis.
    pub text_ellipsis: bool,
    pub text_selectable: bool,
    pub text_select_all: bool,
    pub text_align: TextAlign,
    /// Whether this node is a hit target. `false` (CSS `pointer-events: none`)
    /// makes the node transparent to hit testing — descendants may still hit.
    pub pointer_events: bool,
    /// Sibling-relative paint/hit order (Slint/Qt-Quick z model). Higher z
    /// paints on top and wins hit_test. No DOM stacking context.
    pub z_index: i32,
    /// Resolved font family (`None` = parley default sans-serif).
    pub font_family: Option<Arc<str>>,
    /// Parsed inline SVG attached to an `<svg>` root. Descendant SVG elements
    /// are collapsed into this retained scene fragment by the host runtime.
    pub svg: Option<Arc<crate::svg::SvgImage>>,
    /// A Rust-side `Widget`'s painted scene fragment (e.g. TextInput, Canvas).
    /// Composited by `build_scene` at the node's border-box origin, on top of
    /// the standard bg+border. The host runtime calls `Widget::paint` every
    /// frame and stores the result here.
    pub widget: Option<Arc<vello::Scene>>,
    /// Intrinsic content size supplied by a host widget. CSS known dimensions
    /// override either axis during Taffy measurement.
    pub intrinsic_size: Option<[f32; 2]>,
}

/// Cascaded (pre-inherit) style: layout + declared paint.
#[derive(Clone, Default)]
pub struct CascadedStyle {
    pub layout: taffy::Style,
    pub paint: DeclaredPaint,
}

/// Fully resolved style for a node: taffy layout style + computed paint.
#[derive(Clone, Default)]
pub struct ComputedStyle {
    pub layout: taffy::Style,
    pub paint: Paint,
}

impl Default for Paint {
    fn default() -> Self {
        // CSS initial values at the root of the inheritance chain. These are
        // computed defaults, not "maybe declared" sentinels.
        DeclaredPaint::default().resolve(&InheritedPaint::default(), HostPaint::default())
    }
}

/// Build a uniform `Rect<LengthPercentage>` from a single px value.
pub(super) fn rect_lp_uniform(w: f32) -> taffy::Rect<taffy::LengthPercentage> {
    let l = taffy::LengthPercentage::length(w);
    taffy::Rect {
        top: l,
        right: l,
        bottom: l,
        left: l,
    }
}

/// Parse a CSS color: `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb(r,g,b)`,
/// `rgba(r,g,b,a)`, and a small set of named colors.
pub fn parse_color(value: &str) -> Option<Color> {
    let v = value.trim();
    if let Some(h) = v.strip_prefix('#') {
        return parse_hex(h);
    }
    if let Some(rest) = v
        .strip_prefix("rgba(")
        .or_else(|| v.strip_prefix("rgb("))
        .and_then(|s| s.strip_suffix(')'))
    {
        // Accept comma- OR space-separated, optional `/ alpha`:
        //   rgb(r,g,b) | rgb(r,g,b,a) | rgb(r g b) | rgb(r g b / a)
        let mut alpha = 1.0f32;
        let (rest, alpha_part) = match rest.split_once('/') {
            Some((rgb, a)) => (rgb, Some(a.trim())),
            None => (rest, None),
        };
        let parts: Vec<&str> = rest
            .split([',', ' '])
            .filter(|p| !p.trim().is_empty())
            .collect();
        let n: Vec<f32> = parts.iter().filter_map(|t| t.trim().parse().ok()).collect();
        let (r, g, b) = (*n.first()?, *n.get(1)?, *n.get(2)?);
        if let Some(a) = alpha_part {
            alpha = a.parse().unwrap_or(1.0);
        } else if let Some(a4) = n.get(3) {
            alpha = *a4;
        }
        return Some(Color::from_rgba8(
            r as u8,
            g as u8,
            b as u8,
            (alpha * 255.0) as u8,
        ));
    }
    named_color(v)
}

fn parse_hex(h: &str) -> Option<Color> {
    match h.len() {
        3 => {
            let r = u8::from_str_radix(&h[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&h[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&h[2..3].repeat(2), 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            let a = u8::from_str_radix(&h[6..8], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, a))
        }
        _ => None,
    }
}

fn named_color(name: &str) -> Option<Color> {
    let c = match name.to_ascii_lowercase().as_str() {
        "black" => Color::BLACK,
        "white" => Color::WHITE,
        "transparent" => Color::TRANSPARENT,
        "red" => Color::from_rgb8(0xff, 0x00, 0x00),
        "green" => Color::from_rgb8(0x00, 0x80, 0x00),
        "blue" => Color::from_rgb8(0x00, 0x00, 0xff),
        "yellow" => Color::from_rgb8(0xff, 0xff, 0x00),
        "cyan" | "aqua" => Color::from_rgb8(0x00, 0xff, 0xff),
        "magenta" | "fuchsia" => Color::from_rgb8(0xff, 0x00, 0xff),
        "gray" | "grey" => Color::from_rgb8(0x80, 0x80, 0x80),
        "orange" => Color::from_rgb8(0xff, 0xa5, 0x00),
        "purple" => Color::from_rgb8(0x80, 0x00, 0x80),
        "pink" => Color::from_rgb8(0xff, 0xc0, 0xcb),
        "lime" => Color::from_rgb8(0x00, 0xff, 0x00),
        "teal" => Color::from_rgb8(0x00, 0x80, 0x80),
        "navy" => Color::from_rgb8(0x00, 0x00, 0x80),
        "maroon" => Color::from_rgb8(0x80, 0x00, 0x00),
        "olive" => Color::from_rgb8(0x80, 0x80, 0x00),
        "silver" => Color::from_rgb8(0xc0, 0xc0, 0xc0),
        _ => return None,
    };
    Some(c)
}
