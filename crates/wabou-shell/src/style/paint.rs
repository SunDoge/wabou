use super::*;

#[derive(Clone)]
/// Host-owned vector geometry and its renderer-independent paint contract.
pub struct VectorPath {
    /// Decoded local-coordinate geometry.
    pub path: Arc<vello::kurbo::BezPath>,
    /// Optional fill color.
    pub fill: Option<Color>,
    /// Optional stroke color.
    pub stroke: Option<Color>,
    /// Even-odd rather than non-zero filling.
    pub even_odd: bool,
    /// Vello stroke configuration, including width, caps and joins.
    pub stroke_style: vello::kurbo::Stroke,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Platform cursor requested by the hovered retained node.
pub enum CursorStyle {
    /// Platform default arrow.
    #[default]
    Default,
    /// Link or button pointer.
    Pointer,
    /// Editable text caret.
    Text,
    /// Precise crosshair.
    Crosshair,
    /// Movable content.
    Move,
    /// Busy cursor.
    Wait,
    /// Operation is unavailable.
    NotAllowed,
    /// Horizontal resize.
    EwResize,
    /// Vertical resize.
    NsResize,
}

#[derive(Clone, Debug, PartialEq)]
/// One affine transform operation in authored order.
pub enum PaintTransform {
    /// Translate by horizontal and vertical lengths.
    Translate(IrLength, IrLength),
    /// Scale horizontally and vertically around the transform origin.
    Scale(f32, f32),
    /// Rotate by radians.
    Rotate(f32),
    /// Skew horizontally and vertically by radians.
    Skew(f32, f32),
    /// Explicit `[a, b, c, d, e, f]` affine coefficients.
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
/// Resolved outer box shadow rendered behind a node.
pub struct Shadow {
    /// Horizontal offset in logical pixels.
    pub offset_x: f32,
    /// Vertical offset in logical pixels.
    pub offset_y: f32,
    /// Expansion before blur in logical pixels.
    pub spread: f32,
    /// Gaussian blur standard deviation.
    pub std_dev: f32,
    /// Shadow color including opacity.
    pub color: Color,
    /// Optional corner radius override.
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
#[derive(Clone, Debug, PartialEq)]
pub struct InheritedPaint {
    /// Resolved platform cursor.
    pub cursor: CursorStyle,
    /// Resolved foreground/text color.
    pub text_color: Color,
    /// Resolved font size in logical pixels.
    pub font_size: f32,
    /// Resolved numeric CSS font weight.
    pub font_weight: f32,
    /// Resolved additional spacing between glyphs in logical pixels.
    pub letter_spacing: f32,
    /// Resolved line height and whether it is font-relative.
    pub line_height: Option<(f32, bool)>,
    /// Whether normal inline wrapping is allowed.
    pub wrap_text: bool,
    /// Whether pointer text selection is allowed.
    pub text_selectable: bool,
    /// Whether one selection gesture selects the complete text node.
    pub text_select_all: bool,
    /// Resolved horizontal alignment.
    pub text_align: TextAlign,
    /// Resolved preferred font family.
    pub font_family: Option<Arc<str>>,
}

impl Default for InheritedPaint {
    fn default() -> Self {
        Self {
            cursor: CursorStyle::Default,
            text_color: Color::BLACK,
            font_size: 16.0,
            font_weight: 400.0,
            letter_spacing: 0.0,
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
    /// Authored background fill.
    pub background: Option<Color>,
    /// Node opacity before ancestor composition.
    pub opacity: f32,
    /// Static transform list in authored order.
    pub transform: Vec<PaintTransform>,
    /// Transform pivot relative to the border box.
    pub transform_origin: [IrLength; 2],
    /// Outer shadows rendered behind the border box.
    pub shadows: Vec<Shadow>,
    /// Uniform corner radius in px.
    pub border_radius: f32,
    /// Uniform border (width px, color).
    pub border: Option<(f32, Color)>,
    /// Outline width in logical pixels. Unlike borders, this does not affect layout.
    pub outline_width: f32,
    /// Gap between the border box and outline.
    pub outline_offset: f32,
    /// Outline color; `None` disables outline painting.
    pub outline_color: Option<Color>,
    /// Authored platform cursor, or inheritance when absent.
    pub cursor: Option<CursorStyle>,
    /// Authored foreground color, or inheritance when absent.
    pub text_color: Option<Color>,
    /// Authored font size, or inheritance when absent.
    pub font_size: Option<f32>,
    /// Authored numeric font weight, or inheritance when absent.
    pub font_weight: Option<f32>,
    /// Authored additional spacing between glyphs, or inheritance when absent.
    pub letter_spacing: Option<f32>,
    /// `(value, relative)` where relative means a font-size multiplier.
    pub line_height: Option<(f32, bool)>,
    /// Own `white-space` → wrap mapping. `None` = inherit.
    pub wrap_text: Option<bool>,
    /// Whether overflowing inline text is shortened with an ellipsis.
    pub text_ellipsis: bool,
    /// Own `user-select` mapping. `None` = inherit.
    pub text_selectable: Option<bool>,
    /// Own `user-select: all` mapping, or inheritance when absent.
    pub text_select_all: Option<bool>,
    /// Own text alignment, or inheritance when absent.
    pub text_align: Option<TextAlign>,
    /// Whether the node itself participates in pointer hit testing.
    pub pointer_events: bool,
    /// Sibling-relative paint/hit order inside its overlay plane.
    pub z_index: i32,
    /// Own preferred font family, or inheritance when absent.
    pub font_family: Option<Arc<str>>,
}

impl Default for DeclaredPaint {
    fn default() -> Self {
        Self {
            background: None,
            opacity: 1.0,
            transform: Vec::new(),
            transform_origin: [
                IrLength::Percent { value: 0.5 },
                IrLength::Percent { value: 0.5 },
            ],
            shadows: Vec::new(),
            border_radius: 0.0,
            border: None,
            outline_width: 0.0,
            outline_offset: 0.0,
            outline_color: None,
            cursor: None,
            text_color: None,
            font_size: None,
            font_weight: None,
            letter_spacing: None,
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
            cursor: self.cursor.unwrap_or(parent.cursor),
            text_color: self.text_color.unwrap_or(parent.text_color),
            font_size: self.font_size.unwrap_or(parent.font_size),
            font_weight: self.font_weight.unwrap_or(parent.font_weight),
            letter_spacing: self.letter_spacing.unwrap_or(parent.letter_spacing),
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
            transform_origin: self.transform_origin.clone(),
            runtime_transform: host.runtime_transform,
            overlay_plane: host.overlay_plane,
            scrollbar: host.scrollbar,
            shadows: self.shadows.clone(),
            border_radius: self.border_radius,
            border: self.border,
            outline_width: self.outline_width,
            outline_offset: self.outline_offset,
            outline_color: self.outline_color,
            cursor: inherited.cursor,
            text: host.text,
            text_runs: host.text_runs,
            selection_rects: host.selection_rects,
            text_color: inherited.text_color,
            font_size: inherited.font_size,
            font_weight: inherited.font_weight,
            letter_spacing: inherited.letter_spacing,
            line_height: inherited.line_height,
            wrap_text: inherited.wrap_text,
            text_ellipsis: self.text_ellipsis,
            text_max_lines: host.text_max_lines,
            text_selectable: inherited.text_selectable,
            text_select_all: inherited.text_select_all,
            text_align: inherited.text_align,
            pointer_events: self.pointer_events,
            z_index: self.z_index,
            font_family: inherited.font_family,
            svg: host.svg,
            vector_path: host.vector_path,
            image: host.image,
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
    /// Ordinary application content.
    Content,
    /// Popovers, menus, and other non-modal floating content.
    Floating,
    /// Active modal surfaces and their backdrops.
    Modal,
    /// Host/system UI that must remain above application modals.
    System,
    /// Development overlays rendered above every product surface.
    Debug,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Visibility policy for host-owned overlay scrollbars.
pub enum ScrollbarVisibility {
    #[default]
    /// Show during scrolling/interaction, then fade out.
    Auto,
    /// Keep visible whenever the corresponding axis is scrollable.
    Always,
    /// Never paint or hit-test a native scrollbar.
    Hidden,
}

/// Host-owned appearance for native overlay scrollbars.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarStyle {
    /// Visibility and auto-hide policy.
    pub visibility: ScrollbarVisibility,
    /// Idle delay before an automatic scrollbar begins fading.
    pub hide_delay: std::time::Duration,
    /// Fade duration after [`Self::hide_delay`].
    pub fade_duration: std::time::Duration,
    /// Track/thumb thickness in logical pixels.
    pub thickness: f32,
    /// Inset from the scroll port edge.
    pub margin: f32,
    /// Minimum draggable thumb length.
    pub min_thumb_length: f32,
    /// Negative means half the thumb thickness.
    pub radius: f32,
    /// Track fill.
    pub track_color: Color,
    /// Resting thumb fill.
    pub thumb_color: Color,
    /// Hovered thumb fill.
    pub hover_color: Color,
    /// Actively dragged thumb fill.
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
    /// Plain text content owned by the host node.
    pub text: Option<Arc<str>>,
    /// Styled byte ranges within [`Self::text`].
    pub text_runs: Arc<[crate::text::TextRun]>,
    /// Selection highlights in text-layout-local coordinates.
    pub selection_rects: Arc<[[f32; 4]]>,
    /// Maximum rendered text lines; zero means unlimited.
    pub text_max_lines: u32,
    /// Parsed SVG retained by an SVG root node.
    pub svg: Option<Arc<crate::svg::SvgImage>>,
    /// Typed vector path retained by a `<vector-path>` node.
    pub vector_path: Option<Arc<VectorPath>>,
    /// Decoded raster image retained by a replaced image node.
    pub image: Option<Arc<crate::image::RasterImage>>,
    /// Scene fragment painted by a Rust widget.
    pub widget: Option<Arc<anyrender::Scene>>,
    /// Intrinsic content size used for automatic layout axes.
    pub intrinsic_size: Option<[f32; 2]>,
    /// Host-driven affine coefficients composed after static transforms.
    pub runtime_transform: Option<[f32; 6]>,
    /// Host-owned stacking plane.
    pub overlay_plane: OverlayPlane,
    /// Host-owned overlay-scrollbar appearance.
    pub scrollbar: ScrollbarStyle,
}

/// Fully resolved paint used by layout and rendering.
///
/// Inherited properties are concrete values — no `Option` meaning "inherit"
/// and no parallel `*_declared` flags. Authored declarations live on
/// [`DeclaredPaint`].
#[derive(Clone)]
pub struct Paint {
    /// Resolved background fill.
    pub background: Option<Color>,
    /// Node opacity before ancestor composition.
    pub opacity: f32,
    /// Resolved static transform list.
    pub transform: Vec<PaintTransform>,
    /// Resolved transform pivot relative to the border box.
    pub transform_origin: [IrLength; 2],
    /// Host-driven state, composed after the static CSS transform.
    pub runtime_transform: Option<[f32; 6]>,
    /// Explicit host stacking plane, ordered before sibling `z-index`.
    pub overlay_plane: OverlayPlane,
    /// Host-owned overlay-scrollbar appearance.
    pub scrollbar: ScrollbarStyle,
    /// Resolved outer shadows.
    pub shadows: Vec<Shadow>,
    /// Uniform corner radius in px.
    pub border_radius: f32,
    /// Uniform border (width px, color).
    pub border: Option<(f32, Color)>,
    /// Non-layout outline width in logical pixels.
    pub outline_width: f32,
    /// Gap between the border box and outline.
    pub outline_offset: f32,
    /// Outline color.
    pub outline_color: Option<Color>,
    /// Platform cursor inherited by descendants.
    pub cursor: CursorStyle,
    /// Host-owned plain text content.
    pub text: Option<Arc<str>>,
    /// Styled byte ranges within [`Self::text`].
    pub text_runs: Arc<[crate::text::TextRun]>,
    /// Selection highlight rectangles in text-layout-local coordinates.
    pub selection_rects: Arc<[[f32; 4]]>,
    /// Resolved foreground/text color.
    pub text_color: Color,
    /// Resolved font size in logical pixels.
    pub font_size: f32,
    /// Resolved numeric CSS font weight.
    pub font_weight: f32,
    /// Resolved additional spacing between glyphs in logical pixels.
    pub letter_spacing: f32,
    /// `(value, relative)` where relative means a font-size multiplier.
    /// `None` means CSS `normal` (engine default metrics).
    pub line_height: Option<(f32, bool)>,
    /// Whether normal line wrapping is allowed (resolved `white-space`).
    pub wrap_text: bool,
    /// Whether a constrained single line is shortened with an ellipsis.
    pub text_ellipsis: bool,
    /// Maximum rendered text lines; zero means unlimited.
    pub text_max_lines: u32,
    /// Whether pointer text selection is allowed.
    pub text_selectable: bool,
    /// Whether one selection gesture selects the complete text node.
    pub text_select_all: bool,
    /// Resolved horizontal text alignment.
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
    /// Typed vector path attached to a `<vector-path>` node.
    pub vector_path: Option<Arc<VectorPath>>,
    /// A decoded raster image attached to an `<img>` replaced element.
    pub image: Option<Arc<crate::image::RasterImage>>,
    /// A Rust-side `Widget`'s painted scene fragment (e.g. TextInput, Canvas).
    /// Composited by `build_scene` at the node's border-box origin, on top of
    /// the standard bg+border. The host runtime calls `Widget::paint` every
    /// frame and stores the result here.
    pub widget: Option<Arc<anyrender::Scene>>,
    /// Intrinsic content size supplied by a host widget. CSS known dimensions
    /// override either axis during Taffy measurement.
    pub intrinsic_size: Option<[f32; 2]>,
}

/// Cascaded (pre-inherit) style: layout + declared paint.
#[derive(Clone, Default)]
pub struct CascadedStyle {
    /// Cascaded Taffy layout properties.
    pub layout: taffy::Style,
    /// Cascaded paint declarations before inheritance.
    pub paint: DeclaredPaint,
}

/// Fully resolved style for a node: taffy layout style + computed paint.
#[derive(Clone, Default)]
pub struct ComputedStyle {
    /// Final Taffy layout properties.
    pub layout: taffy::Style,
    /// Final inherited and host-enriched paint state.
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
