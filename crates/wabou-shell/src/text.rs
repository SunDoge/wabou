//! Text layout via parley.
//!
//! parley handles font selection (system fonts via fontique), shaping, and
//! positioning. We build a `Layout` from a node's text string + font size,
//! use it for both taffy measurement (width/height) and vello rasterisation
//! (positioned glyph runs). Layouts are cached by width constraint so
//! measurement and painting use identical line breaks.

#![warn(missing_docs)]

use std::num::NonZeroUsize;
use std::sync::Arc;

use anyrender::{Glyph, PaintScene, Scene};
use lru::LruCache;
use parley::{
    Alignment, AlignmentOptions, FontContext, Layout, LayoutContext, PositionedLayoutItem,
    StyleProperty,
};
use unicode_segmentation::UnicodeSegmentation;
use vello::kurbo::{Affine, Vec2};
use vello::peniko::Fill;

use crate::style::TextAlign;
use vello::peniko::Color;

/// Geometry shared by ordinary text nodes and text-backed native widgets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SingleLineTextMetrics {
    /// Local line box `[x, y, width, height]` inside the owning content box.
    pub line_box: [f32; 4],
    /// Local baseline offset from the content-box top.
    pub baseline: f32,
}

/// Center a Parley single-line layout inside a content box.
///
/// This is the canonical vertical-positioning rule for both normal text and
/// native editors. Components must not add tag-specific optical offsets.
pub fn single_line_text_metrics<B: parley::Brush>(
    layout: &Layout<B>,
    container_height: f32,
) -> Option<SingleLineTextMetrics> {
    let line = layout.lines().next()?;
    let origin_y = ((container_height - layout.height()).max(0.0)) * 0.5;
    Some(SingleLineTextMetrics {
        line_box: [0.0, origin_y, layout.width(), layout.height()],
        baseline: origin_y + line.metrics().baseline,
    })
}

/// Convert a Peniko color into Parley's RGBA brush representation.
pub fn brush_for_color(color: Color) -> [u8; 4] {
    color.to_rgba8().to_u8_array()
}

#[derive(Clone, Debug, PartialEq)]
/// Styled UTF-8 byte range layered over a text node's base style.
pub struct TextRun {
    /// UTF-8 byte range in the owning text string.
    pub range: std::ops::Range<usize>,
    /// Font size in logical pixels.
    pub font_size: f32,
    /// Numeric CSS font weight.
    pub font_weight: f32,
    /// Line height and whether it is relative to font size.
    pub line_height: Option<(f32, bool)>,
    /// RGBA text brush.
    pub color: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Synthetic styles requested by Parley after resolving the available fonts.
pub struct TextSynthesis {
    /// At least one glyph run requires geometric emboldening.
    pub embolden: bool,
    /// At least one glyph run requires a synthetic skew.
    pub skew: bool,
}

/// Inspect the resolved glyph runs without reshaping or rasterizing them.
pub fn text_synthesis(layout: &Layout<[u8; 4]>) -> TextSynthesis {
    let mut result = TextSynthesis::default();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            result.embolden |= run.run().synthesis().embolden();
            result.skew |= run.run().synthesis().skew().is_some();
        }
    }
    result
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct TextRunKey {
    start: usize,
    end: usize,
    font_size: u32,
    font_weight: u32,
    line_height: Option<(u32, bool)>,
    color: [u8; 4],
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct TextLayoutKey {
    text: Arc<str>,
    font_size: u32,
    font_weight: u32,
    letter_spacing: u32,
    line_height: Option<(u32, bool)>,
    max_width: Option<u32>,
    alignment: TextAlign,
    color: [u8; 4],
    runs: Arc<[TextRunKey]>,
    font_family: Option<Arc<str>>,
}

/// The lifetime of a returned [`Layout`] is tied to a borrow of this context.
pub struct TextContext {
    /// Font discovery and fallback database shared by layouts.
    pub font_cx: FontContext,
    /// Scratch state used by Parley while constructing layouts.
    pub layout_cx: LayoutContext,
    cache: LruCache<TextLayoutKey, Arc<Layout<[u8; 4]>>>,
    ellipsis_cache: LruCache<TextLayoutKey, Arc<Layout<[u8; 4]>>>,
    clamp_cache: LruCache<(TextLayoutKey, u32), Arc<Layout<[u8; 4]>>>,
    glyph_cache: LruCache<(usize, u64), GlyphSceneEntry>,
    raster_cache: LruCache<(usize, u64, u8, u8), GlyphSceneEntry>,
    raster_scale_cx: swash::scale::ScaleContext,
    use_swash_raster: bool,
}

type GlyphSceneEntry = (std::sync::Weak<Layout<[u8; 4]>>, Arc<Scene>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OutlineFallback {
    DirectNativeWeight,
    RetainedSyntheticWeight,
}

const fn outline_fallback_for(is_apple_platform: bool) -> OutlineFallback {
    if is_apple_platform {
        OutlineFallback::DirectNativeWeight
    } else {
        OutlineFallback::RetainedSyntheticWeight
    }
}

pub(crate) const OUTLINE_FALLBACK: OutlineFallback =
    outline_fallback_for(cfg!(any(target_os = "macos", target_os = "ios")));

fn use_swash_raster_for(backend: Option<&str>) -> bool {
    match backend {
        Some(value) if value.eq_ignore_ascii_case("swash") => true,
        Some(value) if value.eq_ignore_ascii_case("vello") => false,
        _ => true,
    }
}

fn synthetic_embolden(requested: bool, allowed: bool, font_size: f32, device_scale: f64) -> Vec2 {
    if requested && allowed {
        // Match the conventional FreeType synthetic-bold strength of roughly
        // one twenty-fourth of an em on raster-oriented platforms.
        let amount = f64::from(font_size) * device_scale / 24.0;
        Vec2::new(amount, amount)
    } else {
        Vec2::ZERO
    }
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TextContext {
    /// Discover system fonts (parley default = "system" feature → fontique).
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
            cache: LruCache::new(NonZeroUsize::new(2048).unwrap()),
            ellipsis_cache: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            clamp_cache: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            glyph_cache: LruCache::new(NonZeroUsize::new(2048).unwrap()),
            raster_cache: LruCache::new(NonZeroUsize::new(256).unwrap()),
            raster_scale_cx: swash::scale::ScaleContext::new(),
            use_swash_raster: use_swash_raster_for(
                std::env::var("WABOU_TEXT_BACKEND").ok().as_deref(),
            ),
        }
    }

    /// Whether ordinary axis-aligned text should use Swash rasterization.
    pub(crate) fn uses_swash_raster(&self) -> bool {
        self.use_swash_raster
    }

    /// Stable diagnostic name for the active ordinary-text raster backend.
    pub fn raster_backend_name(&self) -> &'static str {
        if self.use_swash_raster {
            "swash"
        } else {
            "vello-outline"
        }
    }

    /// Stable diagnostic name for the platform outline fallback policy.
    pub fn outline_fallback_name(&self) -> &'static str {
        match OUTLINE_FALLBACK {
            OutlineFallback::DirectNativeWeight => "direct-native-weight",
            OutlineFallback::RetainedSyntheticWeight => "retained-synthetic-weight",
        }
    }

    /// Register a raw font file (TTF/OTF) from its bytes so it becomes
    /// available to parley's font matching. WOFF/WOFF2 are not decoded here —
    /// decode upstream if needed. The text layout cache is cleared because
    /// newly-registered fonts change how existing text resolves.
    pub fn load_font(&mut self, bytes: Vec<u8>) {
        let blob = parley::fontique::Blob::from(bytes);
        self.font_cx.collection.register_fonts(blob, None);
        self.cache.clear();
        self.ellipsis_cache.clear();
        self.clamp_cache.clear();
        self.glyph_cache.clear();
        self.raster_cache.clear();
    }

    /// Encode a positioned Parley layout once and reuse the retained AnyRender
    /// fragment while only its node transform changes. Animated text commonly
    /// keeps identical shaping for thousands of frames; rebuilding every glyph
    /// run into the scene each frame is pure encoding overhead.
    pub fn glyph_scene(&mut self, layout: &Arc<Layout<[u8; 4]>>) -> Arc<Scene> {
        self.glyph_scene_scaled(layout, 1.0)
    }

    /// Encode glyphs at the target device scale.
    ///
    /// Outline glyphs can be scaled after encoding without losing detail, but
    /// bitmap emoji cannot: the renderer selects an embedded bitmap strike from the
    /// font while `draw_glyphs` is encoded. Encoding physical font sizes and
    /// positions here lets Apple Color Emoji select a Retina-sized `sbix`
    /// strike instead of enlarging a logical-pixel bitmap later.
    pub fn glyph_scene_scaled(
        &mut self,
        layout: &Arc<Layout<[u8; 4]>>,
        device_scale: f64,
    ) -> Arc<Scene> {
        let device_scale = device_scale.max(f64::EPSILON);
        let id = Arc::as_ptr(layout) as usize;
        let key = (id, device_scale.to_bits());
        if let Some((cached_layout, scene)) = self.glyph_cache.get(&key)
            && cached_layout
                .upgrade()
                .is_some_and(|cached| Arc::ptr_eq(&cached, layout))
        {
            return scene.clone();
        }

        #[cfg(feature = "profiling")]
        let span = tracing::trace_span!(
            target: "wabou::perf",
            "text.glyph_encode.cache_miss",
            lines = layout.len() as u64,
            device_scale,
        );
        #[cfg(feature = "profiling")]
        let _guard = span.enter();

        let mut scene = Scene::new();
        Self::draw_layout_into(
            &mut scene,
            layout,
            device_scale,
            Affine::IDENTITY,
            true,
            true,
        );
        let scene = Arc::new(scene);
        self.glyph_cache
            .put(key, (Arc::downgrade(layout), scene.clone()));
        scene
    }

    /// Draw a synthesis-free glyph layout directly into the destination scene.
    ///
    /// Direct encoding avoids the retained glyph-fragment issue observed with
    /// Vello/Metal. Apple platforms render unhinted outlines at the font's
    /// native weight and do not geometrically synthesize missing weights;
    /// Linux and Windows retain the Swash raster path selected by the painter.
    pub(crate) fn draw_native_weight_layout_into(
        &self,
        scene: &mut Scene,
        layout: &Layout<[u8; 4]>,
        transform: Affine,
        device_scale: f64,
    ) {
        Self::draw_layout_into(scene, layout, device_scale, transform, false, false);
    }

    fn draw_layout_into(
        scene: &mut Scene,
        layout: &Layout<[u8; 4]>,
        device_scale: f64,
        transform: Affine,
        hint: bool,
        allow_synthetic_embolden: bool,
    ) {
        let device_scale = device_scale.max(f64::EPSILON);
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(gr) = item {
                    let run = gr.run();
                    let font_data = run.font().clone();
                    let glyph_transform = run
                        .synthesis()
                        .skew()
                        .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));
                    let synthesis = run.synthesis();
                    let embolden = synthetic_embolden(
                        synthesis.embolden(),
                        allow_synthetic_embolden,
                        run.font_size(),
                        device_scale,
                    );
                    let glyphs: Vec<Glyph> = gr
                        .positioned_glyphs()
                        .map(|g| Glyph {
                            id: g.id,
                            x: g.x * device_scale as f32,
                            y: g.y * device_scale as f32,
                        })
                        .collect();
                    if !glyphs.is_empty() {
                        scene.draw_glyphs(
                            &font_data,
                            run.font_size() * device_scale as f32,
                            hint,
                            run.normalized_coords(),
                            embolden,
                            Fill::NonZero,
                            Color::from_rgba8(
                                gr.style().brush[0],
                                gr.style().brush[1],
                                gr.style().brush[2],
                                gr.style().brush[3],
                            ),
                            1.0,
                            transform,
                            glyph_transform,
                            glyphs.into_iter(),
                        );
                    }
                }
            }
        }
    }

    /// Rasterize a text layout into a device-pixel-aligned retained fragment.
    ///
    /// This is the preferred path for ordinary axis-aligned UI text. Parley
    /// still owns font matching, shaping, wrapping, and glyph positioning;
    /// Swash only produces the final hinted masks. Callers must place the
    /// fragment at an integer physical-pixel origin. Unsupported or unusually
    /// large layouts return `None` so painting can use vector outlines.
    pub fn raster_scene_scaled(
        &mut self,
        layout: &Arc<Layout<[u8; 4]>>,
        device_scale: f64,
        subpixel_variant: [u8; 2],
    ) -> Option<Arc<Scene>> {
        let device_scale = device_scale.max(f64::EPSILON);
        let id = Arc::as_ptr(layout) as usize;
        let key = (
            id,
            device_scale.to_bits(),
            subpixel_variant[0],
            subpixel_variant[1],
        );
        if let Some((cached_layout, scene)) = self.raster_cache.get(&key)
            && cached_layout
                .upgrade()
                .is_some_and(|cached| Arc::ptr_eq(&cached, layout))
        {
            return Some(scene.clone());
        }
        let scene = crate::text_raster::rasterize_layout(
            layout,
            device_scale,
            subpixel_variant,
            &mut self.raster_scale_cx,
        )?;
        let scene = Arc::new(scene);
        self.raster_cache
            .put(key, (Arc::downgrade(layout), scene.clone()));
        Some(scene)
    }
}

/// Build a single-line layout for `text` at `font_size`. The returned layout
/// borrows `tcx` for its lifetime.
pub fn layout_text(tcx: &mut TextContext, text: &str, font_size: f32) -> Arc<Layout<[u8; 4]>> {
    layout_text_styled(
        tcx,
        Arc::from(text),
        font_size,
        400.0,
        None,
        TextAlign::Start,
        [0, 0, 0, 255],
        Arc::from([]),
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn text_layout_key(
    text: Arc<str>,
    font_size: f32,
    font_weight: f32,
    letter_spacing: f32,
    line_height: Option<(f32, bool)>,
    alignment: TextAlign,
    color: [u8; 4],
    runs: &[TextRun],
    font_family: Option<&Arc<str>>,
    max_width: Option<f32>,
) -> TextLayoutKey {
    TextLayoutKey {
        text,
        font_size: font_size.to_bits(),
        font_weight: font_weight.to_bits(),
        letter_spacing: letter_spacing.to_bits(),
        line_height: line_height.map(|(value, relative)| (value.to_bits(), relative)),
        max_width: max_width.map(f32::to_bits),
        alignment,
        color,
        runs: runs
            .iter()
            .map(|run| TextRunKey {
                start: run.range.start,
                end: run.range.end,
                font_size: run.font_size.to_bits(),
                font_weight: run.font_weight.to_bits(),
                line_height: run
                    .line_height
                    .map(|(value, relative)| (value.to_bits(), relative)),
                color: run.color,
            })
            .collect::<Vec<_>>()
            .into(),
        font_family: font_family.cloned(),
    }
}

#[allow(clippy::too_many_arguments)]
/// Shape and cache styled text with an optional logical width constraint.
pub fn layout_text_styled_with_spacing(
    tcx: &mut TextContext,
    text: Arc<str>,
    font_size: f32,
    font_weight: f32,
    letter_spacing: f32,
    line_height: Option<(f32, bool)>,
    alignment: TextAlign,
    color: [u8; 4],
    runs: Arc<[TextRun]>,
    font_family: Option<&Arc<str>>,
    max_width: Option<f32>,
) -> Arc<Layout<[u8; 4]>> {
    let key = text_layout_key(
        text.clone(),
        font_size,
        font_weight,
        letter_spacing,
        line_height,
        alignment,
        color,
        &runs,
        font_family,
        max_width,
    );
    if let Some(layout) = tcx.cache.get(&key) {
        return layout.clone();
    }
    #[cfg(feature = "profiling")]
    let span = tracing::trace_span!(
        target: "wabou::perf",
        "text.shape.cache_miss",
        bytes = text.len() as u64,
        styled_runs = runs.len() as u64,
        constrained = max_width.is_some(),
    );
    #[cfg(feature = "profiling")]
    let _guard = span.enter();
    let mut builder = tcx
        .layout_cx
        .ranged_builder(&mut tcx.font_cx, &text, 1.0, false);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(StyleProperty::Brush(color));
    builder.push_default(StyleProperty::FontWeight(parley::FontWeight::new(
        font_weight,
    )));
    builder.push_default(StyleProperty::LetterSpacing(letter_spacing));
    if let Some(family) = font_family {
        builder.push_default(StyleProperty::FontFamily(parley::FontFamily::from(
            family.as_ref(),
        )));
    }
    if let Some((value, relative)) = line_height {
        builder.push_default(StyleProperty::LineHeight(if relative {
            parley::LineHeight::FontSizeRelative(value)
        } else {
            parley::LineHeight::Absolute(value)
        }));
    }
    for run in runs.iter() {
        builder.push(StyleProperty::FontSize(run.font_size), run.range.clone());
        builder.push(
            StyleProperty::FontWeight(parley::FontWeight::new(run.font_weight)),
            run.range.clone(),
        );
        builder.push(StyleProperty::Brush(run.color), run.range.clone());
        if let Some((value, relative)) = run.line_height {
            builder.push(
                StyleProperty::LineHeight(if relative {
                    parley::LineHeight::FontSizeRelative(value)
                } else {
                    parley::LineHeight::Absolute(value)
                }),
                run.range.clone(),
            );
        }
    }
    let mut layout = builder.build(&text);
    layout.break_all_lines(max_width);
    layout.align(
        match alignment {
            TextAlign::Start => Alignment::Start,
            TextAlign::Center => Alignment::Center,
            TextAlign::End => Alignment::End,
            TextAlign::Justify => Alignment::Justify,
        },
        AlignmentOptions::default(),
    );
    let layout = Arc::new(layout);
    tcx.cache.put(key, layout.clone());
    layout
}

#[allow(clippy::too_many_arguments)]
/// Shape and cache styled text using the default letter spacing.
pub fn layout_text_styled(
    tcx: &mut TextContext,
    text: Arc<str>,
    font_size: f32,
    font_weight: f32,
    line_height: Option<(f32, bool)>,
    alignment: TextAlign,
    color: [u8; 4],
    runs: Arc<[TextRun]>,
    font_family: Option<&Arc<str>>,
    max_width: Option<f32>,
) -> Arc<Layout<[u8; 4]>> {
    layout_text_styled_with_spacing(
        tcx,
        text,
        font_size,
        font_weight,
        0.0,
        line_height,
        alignment,
        color,
        runs,
        font_family,
        max_width,
    )
}

/// Shape text and limit the visible result to `max_lines` using an ellipsis.
///
/// `max_lines == 0` means unlimited. The truncation point follows grapheme
/// boundaries and is derived from Parley's actual line breaking rather than
/// an estimated character count.
#[allow(clippy::too_many_arguments)]
pub fn layout_text_styled_clamped(
    tcx: &mut TextContext,
    text: Arc<str>,
    font_size: f32,
    font_weight: f32,
    letter_spacing: f32,
    line_height: Option<(f32, bool)>,
    alignment: TextAlign,
    color: [u8; 4],
    runs: Arc<[TextRun]>,
    font_family: Option<&Arc<str>>,
    max_width: Option<f32>,
    max_lines: u32,
) -> Arc<Layout<[u8; 4]>> {
    let base = layout_text_styled_with_spacing(
        tcx,
        text.clone(),
        font_size,
        font_weight,
        letter_spacing,
        line_height,
        alignment,
        color,
        runs.clone(),
        font_family,
        max_width,
    );
    let Ok(line_limit) = usize::try_from(max_lines) else {
        return base;
    };
    if line_limit == 0 || base.len() <= line_limit {
        return base;
    }

    let key = text_layout_key(
        text.clone(),
        font_size,
        font_weight,
        letter_spacing,
        line_height,
        alignment,
        color,
        &runs,
        font_family,
        max_width,
    );
    if let Some(layout) = tcx.clamp_cache.get(&(key.clone(), max_lines)) {
        return layout.clone();
    }
    let final_line_end = base
        .lines()
        .nth(line_limit - 1)
        .map(|line| line.text_range().end.min(text.len()))
        .unwrap_or(0);
    let boundaries = std::iter::once(0)
        .chain(
            text[..final_line_end]
                .grapheme_indices(true)
                .map(|(index, grapheme)| index + grapheme.len()),
        )
        .collect::<Vec<_>>();
    let mut low = 0;
    let mut high = boundaries.len();
    let mut best = layout_text_styled_with_spacing(
        tcx,
        Arc::from("…"),
        font_size,
        font_weight,
        letter_spacing,
        line_height,
        alignment,
        color,
        Arc::from([]),
        font_family,
        max_width,
    );
    while low < high {
        let middle = usize::midpoint(low, high);
        let prefix_end = boundaries[middle];
        let candidate: Arc<str> = Arc::from(format!("{}…", &text[..prefix_end]));
        let candidate_runs: Arc<[TextRun]> = runs
            .iter()
            .filter(|run| run.range.start < prefix_end)
            .map(|run| TextRun {
                range: run.range.start..run.range.end.min(prefix_end),
                ..run.clone()
            })
            .collect::<Vec<_>>()
            .into();
        let candidate_layout = layout_text_styled_with_spacing(
            tcx,
            candidate,
            font_size,
            font_weight,
            letter_spacing,
            line_height,
            alignment,
            color,
            candidate_runs,
            font_family,
            max_width,
        );
        if candidate_layout.len() <= line_limit {
            best = candidate_layout;
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    tcx.clamp_cache.put((key, max_lines), best.clone());
    best
}

/// Layout text with CSS-style single-line ellipsis when it exceeds `max_width`.
/// Truncation follows grapheme boundaries, so emoji sequences and combining
/// marks are never split in the middle.
#[allow(clippy::too_many_arguments)]
pub fn layout_text_styled_overflow(
    tcx: &mut TextContext,
    text: Arc<str>,
    font_size: f32,
    font_weight: f32,
    letter_spacing: f32,
    line_height: Option<(f32, bool)>,
    alignment: TextAlign,
    color: [u8; 4],
    runs: Arc<[TextRun]>,
    font_family: Option<&Arc<str>>,
    max_width: Option<f32>,
    ellipsis: bool,
) -> Arc<Layout<[u8; 4]>> {
    let layout =
        |tcx: &mut TextContext, text: Arc<str>, runs: Arc<[TextRun]>, max_width: Option<f32>| {
            layout_text_styled_with_spacing(
                tcx,
                text,
                font_size,
                font_weight,
                letter_spacing,
                line_height,
                alignment,
                color,
                runs,
                font_family,
                max_width,
            )
        };
    let Some(max_width) = max_width.filter(|width| width.is_finite()) else {
        return layout(tcx, text, runs, max_width);
    };
    if !ellipsis {
        return layout(tcx, text, runs, Some(max_width));
    }

    let overflow_key = text_layout_key(
        text.clone(),
        font_size,
        font_weight,
        letter_spacing,
        line_height,
        alignment,
        color,
        &runs,
        font_family,
        Some(max_width),
    );
    if let Some(layout) = tcx.ellipsis_cache.get(&overflow_key) {
        return layout.clone();
    }

    let unconstrained = layout(tcx, text.clone(), runs.clone(), None);
    if unconstrained.width() <= max_width.max(0.0) {
        tcx.ellipsis_cache.put(overflow_key, unconstrained.clone());
        return unconstrained;
    }

    let boundaries = text
        .grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect::<Vec<_>>();
    let mut low = 0;
    let mut high = boundaries.len();
    let mut best = layout(tcx, Arc::from(""), Arc::from([]), None);
    while low < high {
        let middle = usize::midpoint(low, high);
        let prefix_end = boundaries[middle];
        let candidate: Arc<str> = Arc::from(format!("{}…", &text[..prefix_end]));
        let candidate_runs: Arc<[TextRun]> = runs
            .iter()
            .filter(|run| run.range.start < prefix_end)
            .map(|run| TextRun {
                range: run.range.start..run.range.end.min(prefix_end),
                ..run.clone()
            })
            .collect::<Vec<_>>()
            .into();
        let candidate_layout = layout(tcx, candidate, candidate_runs, None);
        if candidate_layout.width() <= max_width.max(0.0) {
            best = candidate_layout;
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    tcx.ellipsis_cache.put(overflow_key, best.clone());
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_outline_policy_suppresses_synthetic_bold() {
        let suppressed = synthetic_embolden(true, false, 16.0, 2.0);
        let raster = synthetic_embolden(true, true, 16.0, 2.0);

        assert_eq!(suppressed, Vec2::ZERO);
        assert_eq!(raster, Vec2::new(4.0 / 3.0, 4.0 / 3.0));
    }

    #[test]
    fn platform_outline_policy_is_the_only_platform_specific_text_fallback() {
        assert_eq!(
            outline_fallback_for(true),
            OutlineFallback::DirectNativeWeight
        );
        assert_eq!(
            outline_fallback_for(false),
            OutlineFallback::RetainedSyntheticWeight
        );
    }

    #[test]
    fn text_backend_defaults_to_swash_and_keeps_vello_override() {
        assert!(use_swash_raster_for(None));
        assert!(use_swash_raster_for(Some("swash")));
        assert!(use_swash_raster_for(Some("SWASH")));
        assert!(!use_swash_raster_for(Some("vello")));
        assert!(!use_swash_raster_for(Some("VELLO")));

        let context = TextContext::new();
        assert_eq!(context.raster_backend_name(), "swash");
        assert!(matches!(
            context.outline_fallback_name(),
            "direct-native-weight" | "retained-synthetic-weight"
        ));
    }

    #[test]
    fn identical_text_layouts_are_shared() {
        let mut context = TextContext::new();
        let text: Arc<str> = Arc::from("repeated UI text");
        let first = layout_text_styled(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            None,
        );
        let second = layout_text_styled(
            &mut context,
            text,
            16.0,
            400.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            None,
        );

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(context.cache.len(), 1);
    }

    #[test]
    fn resolved_text_synthesis_is_inspectable_without_rasterizing() {
        let mut context = TextContext::new();
        let layout = layout_text(&mut context, "ordinary text", 16.0);

        assert_eq!(text_synthesis(&layout), TextSynthesis::default());
    }

    #[test]
    fn ellipsis_layout_respects_width_and_grapheme_boundaries() {
        let mut context = TextContext::new();
        let text: Arc<str> = Arc::from("Family 👨‍👩‍👧‍👦 credentials and recovery material");
        let full = layout_text_styled_overflow(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            0.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            None,
            true,
        );
        let truncated = layout_text_styled_overflow(
            &mut context,
            text,
            16.0,
            400.0,
            0.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(120.0),
            true,
        );
        let cached = layout_text_styled_overflow(
            &mut context,
            Arc::from("Family 👨‍👩‍👧‍👦 credentials and recovery material"),
            16.0,
            400.0,
            0.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(120.0),
            true,
        );

        assert!(full.width() > 120.0);
        assert!(truncated.width() <= 120.0);
        assert_eq!(truncated.lines().count(), 1);
        assert!(Arc::ptr_eq(&truncated, &cached));
    }

    #[test]
    fn clamped_layout_uses_actual_line_breaks_and_is_cached() {
        let mut context = TextContext::new();
        let text: Arc<str> = Arc::from(
            "A long native UI description with emoji 👨‍👩‍👧‍👦 that needs several wrapped lines",
        );
        let full = layout_text_styled(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(120.0),
        );
        let clamped = layout_text_styled_clamped(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            0.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(120.0),
            2,
        );
        let cached = layout_text_styled_clamped(
            &mut context,
            text,
            16.0,
            400.0,
            0.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(120.0),
            2,
        );

        assert!(full.len() > 2);
        assert_eq!(clamped.len(), 2);
        assert!(clamped.height() < full.height());
        assert!(Arc::ptr_eq(&clamped, &cached));
    }

    #[test]
    fn letter_spacing_changes_measurement_and_cache_identity() {
        let mut context = TextContext::new();
        let text: Arc<str> = Arc::from("Tracking");
        let normal = layout_text_styled_with_spacing(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            0.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            None,
        );
        let wide = layout_text_styled_with_spacing(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            2.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            None,
        );
        let cached = layout_text_styled_with_spacing(
            &mut context,
            text,
            16.0,
            400.0,
            2.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            None,
        );

        assert!(wide.width() > normal.width());
        assert!(!Arc::ptr_eq(&normal, &wide));
        assert!(Arc::ptr_eq(&wide, &cached));
    }

    #[test]
    fn identical_layout_reuses_encoded_glyph_scene() {
        let mut context = TextContext::new();
        let layout = layout_text(&mut context, "🚀", 28.0);
        let first = context.glyph_scene(&layout);
        let second = context.glyph_scene(&layout);

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(context.glyph_cache.len(), 1);
    }

    #[test]
    fn glyph_scene_cache_distinguishes_device_scales() {
        let mut context = TextContext::new();
        let layout = layout_text(&mut context, "🚀", 28.0);
        let one_x = context.glyph_scene_scaled(&layout, 1.0);
        let two_x = context.glyph_scene_scaled(&layout, 2.0);

        assert!(!Arc::ptr_eq(&one_x, &two_x));
        assert_eq!(context.glyph_cache.len(), 2);
    }

    #[test]
    fn single_line_metrics_center_the_line_box_without_baseline_guessing() {
        let mut context = TextContext::new();
        let layout = layout_text(&mut context, "example.com", 14.0);
        let metrics = single_line_text_metrics(&layout, layout.height() + 8.0)
            .expect("a single-line layout should expose metrics");
        let baseline = layout
            .lines()
            .next()
            .expect("the layout should contain one line")
            .metrics()
            .baseline;

        assert_eq!(metrics.line_box[1], 4.0);
        assert_eq!(metrics.baseline, 4.0 + baseline);
    }

    #[test]
    fn raster_scene_is_cached_by_scale_and_fractional_origin() {
        let mut context = TextContext::new();
        let layout = layout_text(&mut context, "Hinted UI text", 14.0);
        let first = context
            .raster_scene_scaled(&layout, 1.0, [0, 0])
            .expect("the system font should rasterize through Swash");
        let cached = context
            .raster_scene_scaled(&layout, 1.0, [0, 0])
            .expect("the cached raster scene should remain available");
        let fractional = context
            .raster_scene_scaled(&layout, 1.0, [1, 0])
            .expect("a quarter-pixel origin should rasterize independently");
        let hidpi = context
            .raster_scene_scaled(&layout, 2.0, [0, 0])
            .expect("the same layout should rasterize at HiDPI scale");

        assert!(Arc::ptr_eq(&first, &cached));
        assert!(!Arc::ptr_eq(&first, &fractional));
        assert!(!Arc::ptr_eq(&first, &hidpi));
        assert_eq!(context.raster_cache.len(), 3);
    }

    #[test]
    fn constrained_text_wraps_and_uses_a_distinct_cache_entry() {
        let mut context = TextContext::new();
        let text: Arc<str> = Arc::from("a title that should wrap across multiple lines");
        let wide = layout_text_styled(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            None,
        );
        let narrow = layout_text_styled(
            &mut context,
            text,
            16.0,
            400.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(80.0),
        );
        assert!(narrow.height() > wide.height());
        assert_eq!(context.cache.len(), 2);
    }

    #[test]
    fn alignment_is_part_of_the_layout_cache_key() {
        let mut context = TextContext::new();
        let text: Arc<str> = Arc::from("centered");
        layout_text_styled(
            &mut context,
            text.clone(),
            16.0,
            400.0,
            None,
            TextAlign::Start,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(200.0),
        );
        layout_text_styled(
            &mut context,
            text,
            16.0,
            400.0,
            None,
            TextAlign::Center,
            [0, 0, 0, 255],
            Arc::from([]),
            None,
            Some(200.0),
        );
        assert_eq!(context.cache.len(), 2);
    }
}
