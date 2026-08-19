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

use lru::LruCache;
use parley::{
    Alignment, AlignmentOptions, FontContext, Layout, LayoutContext, PositionedLayoutItem,
    StyleProperty,
};
use unicode_segmentation::UnicodeSegmentation;
use vello::Glyph as VelloGlyph;
use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::Fill;

use crate::style::TextAlign;
use vello::peniko::Color;

/// Convert a Vello color into Parley's RGBA brush representation.
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
    glyph_cache: LruCache<(usize, u64), GlyphSceneEntry>,
}

type GlyphSceneEntry = (std::sync::Weak<Layout<[u8; 4]>>, Arc<Scene>);

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
            glyph_cache: LruCache::new(NonZeroUsize::new(2048).unwrap()),
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
        self.glyph_cache.clear();
    }

    /// Encode a positioned Parley layout once and reuse the retained Vello
    /// fragment while only its node transform changes. Animated text commonly
    /// keeps identical shaping for thousands of frames; rebuilding every glyph
    /// run into the scene each frame is pure encoding overhead.
    pub fn glyph_scene(&mut self, layout: &Arc<Layout<[u8; 4]>>) -> Arc<Scene> {
        self.glyph_scene_scaled(layout, 1.0)
    }

    /// Encode glyphs at the target device scale.
    ///
    /// Outline glyphs can be scaled after encoding without losing detail, but
    /// bitmap emoji cannot: Vello selects an embedded bitmap strike from the
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
                    let embolden = if synthesis.embolden() {
                        // Match the conventional FreeType synthetic-bold
                        // strength of roughly one twenty-fourth of an em.
                        let amount = f64::from(run.font_size()) * device_scale / 24.0;
                        vello::FontEmbolden::new(vello::kurbo::Diagonal2::new(amount, amount))
                    } else {
                        vello::FontEmbolden::default()
                    };
                    let glyphs: Vec<VelloGlyph> = gr
                        .positioned_glyphs()
                        .map(|g| VelloGlyph {
                            id: g.id,
                            x: g.x * device_scale as f32,
                            y: g.y * device_scale as f32,
                        })
                        .collect();
                    if !glyphs.is_empty() {
                        scene
                            .draw_glyphs(&font_data)
                            .font_size(run.font_size() * device_scale as f32)
                            .normalized_coords(run.normalized_coords())
                            .glyph_transform(glyph_transform)
                            .font_embolden(embolden)
                            // Vello defaults glyph hinting to false. Request it
                            // here so uniform device-scale transforms are folded
                            // into the physical font size and stems align to the
                            // device pixel grid. Vello safely disables hinting
                            // for rotated, skewed, or non-uniform transforms.
                            .hint(true)
                            .brush(Color::from_rgba8(
                                gr.style().brush[0],
                                gr.style().brush[1],
                                gr.style().brush[2],
                                gr.style().brush[3],
                            ))
                            .transform(Affine::IDENTITY)
                            .draw(Fill::NonZero, glyphs.into_iter());
                    }
                }
            }
        }
        let scene = Arc::new(scene);
        self.glyph_cache
            .put(key, (Arc::downgrade(layout), scene.clone()));
        scene
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
    let key = text_layout_key(
        text.clone(),
        font_size,
        font_weight,
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

/// Layout text with CSS-style single-line ellipsis when it exceeds `max_width`.
/// Truncation follows grapheme boundaries, so emoji sequences and combining
/// marks are never split in the middle.
#[allow(clippy::too_many_arguments)]
pub fn layout_text_styled_overflow(
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
    ellipsis: bool,
) -> Arc<Layout<[u8; 4]>> {
    let layout =
        |tcx: &mut TextContext, text: Arc<str>, runs: Arc<[TextRun]>, max_width: Option<f32>| {
            layout_text_styled(
                tcx,
                text,
                font_size,
                font_weight,
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
    fn ellipsis_layout_respects_width_and_grapheme_boundaries() {
        let mut context = TextContext::new();
        let text: Arc<str> = Arc::from("Family 👨‍👩‍👧‍👦 credentials and recovery material");
        let full = layout_text_styled_overflow(
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
            true,
        );
        let truncated = layout_text_styled_overflow(
            &mut context,
            text,
            16.0,
            400.0,
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
