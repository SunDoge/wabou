//! Style IR types, the single [`apply_ir`] application backend, and the
//! runtime string→`IrValue` parser [`parse_ir_value`] used for inline styles.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use taffy::prelude::*;
use taffy::style::{GridTemplateArea, GridTemplateRepetition};
use vello::peniko::Color;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
    Justify,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "unit", rename_all = "kebab-case")]
pub enum IrLength {
    Px { value: f32 },
    Percent { value: f32 },
    Auto,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum IrColor {
    Literal { rgba: u32 },
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum IrValue {
    Keyword { value: String },
    Boolean { value: bool },
    Number { value: f32 },
    Length { value: IrLength },
    Color { value: IrColor },
    List { values: Vec<IrValue> },
    Record { fields: HashMap<String, IrValue> },
}

impl IrValue {
    fn keyword(&self) -> Option<&str> {
        match self {
            Self::Keyword { value } => Some(value),
            _ => None,
        }
    }

    fn number(&self) -> Option<f32> {
        match self {
            Self::Number { value } => Some(*value),
            _ => None,
        }
    }

    fn length(&self) -> Option<&IrLength> {
        match self {
            Self::Length { value } => Some(value),
            _ => None,
        }
    }
}

fn field<'a>(value: &'a IrValue, name: &str) -> Option<&'a IrValue> {
    let IrValue::Record { fields } = value else {
        return None;
    };
    fields.get(name)
}

fn track_breadth(
    value: &IrValue,
    maximum: bool,
) -> Option<(MinTrackSizingFunction, MaxTrackSizingFunction)> {
    let kind = field(value, "kind")?.keyword()?;
    let pair = match kind {
        "length" => {
            let lp = ir_lp(field(value, "value")?)?;
            (
                MinTrackSizingFunction::from(lp),
                MaxTrackSizingFunction::from(lp),
            )
        }
        "flex" if maximum => (
            MinTrackSizingFunction::AUTO,
            MaxTrackSizingFunction::fr(field(value, "value")?.number()?),
        ),
        "auto" => (MinTrackSizingFunction::AUTO, MaxTrackSizingFunction::AUTO),
        "min-content" => (
            MinTrackSizingFunction::MIN_CONTENT,
            MaxTrackSizingFunction::MIN_CONTENT,
        ),
        "max-content" => (
            MinTrackSizingFunction::MAX_CONTENT,
            MaxTrackSizingFunction::MAX_CONTENT,
        ),
        _ => return None,
    };
    Some(pair)
}

fn track_size(value: &IrValue) -> Option<TrackSizingFunction> {
    match field(value, "kind")?.keyword()? {
        "breadth" => {
            let (min, max) = track_breadth(field(value, "value")?, true)?;
            Some(TrackSizingFunction { min, max })
        }
        "minmax" => {
            let (min, _) = track_breadth(field(value, "min")?, false)?;
            let (_, max) = track_breadth(field(value, "max")?, true)?;
            Some(TrackSizingFunction { min, max })
        }
        "fit-content" => Some(TrackSizingFunction::fit_content(ir_lp(field(
            value, "value",
        )?)?)),
        _ => None,
    }
}

fn grid_template(value: &IrValue) -> Option<Vec<GridTemplateComponent<String>>> {
    let IrValue::List { values } = value else {
        return None;
    };
    values
        .iter()
        .map(|item| match field(item, "kind")?.keyword()? {
            "single" => Some(GridTemplateComponent::Single(track_size(field(
                item, "value",
            )?)?)),
            "repeat" => {
                let count = match field(item, "count")? {
                    IrValue::Number { value } => RepetitionCount::Count(*value as u16),
                    v if v.keyword() == Some("auto-fill") => RepetitionCount::AutoFill,
                    v if v.keyword() == Some("auto-fit") => RepetitionCount::AutoFit,
                    _ => return None,
                };
                let IrValue::List { values } = field(item, "values")? else {
                    return None;
                };
                let tracks = values.iter().map(track_size).collect::<Option<Vec<_>>>()?;
                Some(GridTemplateComponent::Repeat(GridTemplateRepetition {
                    count,
                    tracks,
                    line_names: vec![],
                }))
            }
            _ => None,
        })
        .collect()
}

fn grid_template_areas(value: &IrValue) -> Option<Vec<GridTemplateArea<String>>> {
    let columns = field(value, "columns")?.number()? as usize;
    let IrValue::List { values } = field(value, "cells")? else {
        return None;
    };
    if columns == 0 {
        return Some(Vec::new());
    }
    let mut bounds: HashMap<&str, (usize, usize, usize, usize)> = HashMap::new();
    for (index, cell) in values.iter().enumerate() {
        let name = cell.keyword()?;
        if name == "." {
            continue;
        }
        let row = index / columns;
        let column = index % columns;
        bounds
            .entry(name)
            .and_modify(|b| {
                b.0 = b.0.min(row);
                b.1 = b.1.max(row);
                b.2 = b.2.min(column);
                b.3 = b.3.max(column);
            })
            .or_insert((row, row, column, column));
    }
    Some(
        bounds
            .into_iter()
            .map(|(name, (r0, r1, c0, c1))| GridTemplateArea {
                name: name.to_owned(),
                row_start: r0 as u16 + 1,
                row_end: r1 as u16 + 2,
                column_start: c0 as u16 + 1,
                column_end: c1 as u16 + 2,
            })
            .collect(),
    )
}

fn ir_lp(value: &IrValue) -> Option<taffy::LengthPercentage> {
    // A unitless number is only valid as a length when it's `0` (CSS), but
    // accepting any number as px is harmless and lets inline `top: 0` etc.
    // work without forcing a unit.
    if let IrValue::Number { value } = value {
        return Some(taffy::LengthPercentage::length(*value));
    }
    match value.length()? {
        IrLength::Px { value } => Some(taffy::LengthPercentage::length(*value)),
        IrLength::Percent { value } => Some(taffy::LengthPercentage::percent(*value)),
        IrLength::Auto => None,
    }
}

fn ir_lpa(value: &IrValue) -> Option<taffy::LengthPercentageAuto> {
    if let IrValue::Number { value } = value {
        return Some(taffy::LengthPercentageAuto::length(*value));
    }
    match value.length()? {
        IrLength::Px { value } => Some(taffy::LengthPercentageAuto::length(*value)),
        IrLength::Percent { value } => Some(taffy::LengthPercentageAuto::percent(*value)),
        IrLength::Auto => Some(taffy::LengthPercentageAuto::AUTO),
    }
}

fn ir_dim(value: &IrValue) -> Option<taffy::Dimension> {
    if let IrValue::Number { value } = value {
        return Some(taffy::Dimension::length(*value));
    }
    match value.length()? {
        IrLength::Px { value } => Some(taffy::Dimension::length(*value)),
        IrLength::Percent { value } => Some(taffy::Dimension::percent(*value)),
        IrLength::Auto => Some(taffy::Dimension::AUTO),
    }
}

fn ir_color(value: &IrValue) -> Option<Color> {
    let IrValue::Color { value } = value else {
        return None;
    };
    let IrColor::Literal { rgba } = value;
    Some(Color::from_rgba8(
        (rgba >> 24) as u8,
        (rgba >> 16) as u8,
        (rgba >> 8) as u8,
        *rgba as u8,
    ))
}

/// Parse a CSS length string into a Style IR length: `Npx`/`Nrem` (×16) →
/// `Px`, `N%` → `Percent`. Anything else returns `None`.
fn parse_ir_length(value: &str) -> Option<IrLength> {
    let v = value.trim();
    if let Some(r) = v.strip_suffix("rem") {
        return r
            .trim()
            .parse::<f32>()
            .ok()
            .map(|n| IrLength::Px { value: n * 16.0 });
    }
    if let Some(p) = v.strip_suffix("px") {
        return p
            .trim()
            .parse::<f32>()
            .ok()
            .map(|n| IrLength::Px { value: n });
    }
    if let Some(p) = v.strip_suffix('%') {
        return p
            .trim()
            .parse::<f32>()
            .ok()
            // Taffy stores percentages as a 0..1 ratio, while inline CSS
            // strings use the authored 0..100 representation.
            .map(|n| IrLength::Percent { value: n / 100.0 });
    }
    None
}

/// Parse a runtime CSS string value (from an inline `style={{...}}`) into a
/// typed [`IrValue`] so inline styles share the single [`apply_ir`] backend
/// with class rules — no property awareness, no shorthand expansion. Tries
/// color → length → number, then falls back to a keyword.
pub fn parse_ir_value(value: &str) -> IrValue {
    if let Some(c) = parse_color(value) {
        let rgba8 = c.to_rgba8();
        let rgba = ((rgba8.r as u32) << 24)
            | ((rgba8.g as u32) << 16)
            | ((rgba8.b as u32) << 8)
            | (rgba8.a as u32);
        return IrValue::Color {
            value: IrColor::Literal { rgba },
        };
    }
    if let Some(length) = parse_ir_length(value) {
        return IrValue::Length { value: length };
    }
    if let Ok(number) = value.trim().parse::<f32>() {
        return IrValue::Number { value: number };
    }
    IrValue::Keyword {
        value: value.trim().to_string(),
    }
}

fn font_family_stack(value: &IrValue) -> Option<Arc<str>> {
    let authored: Vec<&str> = match value {
        IrValue::Keyword { value } => value.split(',').map(str::trim).collect(),
        IrValue::List { values } => values.iter().filter_map(IrValue::keyword).collect(),
        _ => return None,
    };
    let mut normalized = Vec::with_capacity(authored.len());
    for raw in authored {
        let raw = raw
            .strip_prefix(['\'', '"'])
            .and_then(|value| value.strip_suffix(['\'', '"']))
            .unwrap_or(raw)
            .trim();
        if raw.is_empty() {
            continue;
        }
        let family = match raw.to_ascii_lowercase().as_str() {
            "ui-monospace" => "monospace".to_string(),
            "ui-sans-serif" | "system-ui" | "-apple-system" | "blinkmacsystemfont" => {
                "sans-serif".to_string()
            }
            "ui-serif" => "serif".to_string(),
            "ui-rounded" => "sans-serif".to_string(),
            "serif" | "sans-serif" | "monospace" | "cursive" | "fantasy" => {
                raw.to_ascii_lowercase()
            }
            _ => format!("\"{}\"", raw.replace('\\', "\\\\").replace('"', "\\\"")),
        };
        if normalized.last() != Some(&family) {
            normalized.push(family);
        }
    }
    (!normalized.is_empty()).then(|| Arc::from(normalized.join(", ")))
}

/// Apply an already parsed Style IR declaration into cascaded layout +
/// [`DeclaredPaint`]. This path performs no CSS parsing or unit normalization
/// at runtime, and never writes computed/inherited values — those are produced
/// later by resolving [`DeclaredPaint`] against the parent.
///
/// # Support contract
///
/// Property coverage is the CSS support matrix
/// (`packages/style-compiler/css-support-matrix.json`):
/// - **supported** — applied here
/// - anything else — returns `false` (compiler must not emit these)
///
/// A host test walks that matrix so compiler allowlists and this match cannot
/// drift silently.
pub fn apply_ir(
    style: &mut taffy::Style,
    paint: &mut DeclaredPaint,
    property: &str,
    value: &IrValue,
) -> bool {
    match property {
        "display" => {
            let keyword = value.keyword().or_else(|| match value {
                IrValue::Record { fields } => fields.get("inside")?.keyword(),
                _ => None,
            });
            style.display = match keyword {
                Some("flex") => taffy::Display::Flex,
                Some("grid") => taffy::Display::Grid,
                Some("none") => taffy::Display::None,
                _ => taffy::Display::Block,
            };
        }
        "flex-direction" => {
            style.flex_direction = match value.keyword() {
                Some("column") => taffy::FlexDirection::Column,
                Some("column-reverse") => taffy::FlexDirection::ColumnReverse,
                Some("row-reverse") => taffy::FlexDirection::RowReverse,
                _ => taffy::FlexDirection::Row,
            }
        }
        "justify-content" => style.justify_content = value.keyword().and_then(|v| v.parse().ok()),
        "align-items" => style.align_items = value.keyword().and_then(|v| v.parse().ok()),
        "align-content" => style.align_content = value.keyword().and_then(|v| v.parse().ok()),
        "align-self" => style.align_self = value.keyword().and_then(|v| v.parse().ok()),
        "flex-grow" => style.flex_grow = value.number().unwrap_or(style.flex_grow),
        "flex-shrink" => style.flex_shrink = value.number().unwrap_or(style.flex_shrink),
        "flex-basis" => style.flex_basis = ir_dim(value).unwrap_or(style.flex_basis),
        "flex" => {
            if let IrValue::Record { fields } = value {
                if let Some(v) = fields.get("grow").and_then(IrValue::number) {
                    style.flex_grow = v;
                }
                if let Some(v) = fields.get("shrink").and_then(IrValue::number) {
                    style.flex_shrink = v;
                }
                if let Some(v) = fields.get("basis").and_then(ir_dim) {
                    style.flex_basis = v;
                }
            }
        }
        "grid-template-columns" => {
            if let Some(v) = grid_template(value) {
                style.grid_template_columns = v;
            }
        }
        "grid-template-rows" => {
            if let Some(v) = grid_template(value) {
                style.grid_template_rows = v;
            }
        }
        "grid-template-areas" => {
            if let Some(v) = grid_template_areas(value) {
                style.grid_template_areas = v;
            }
        }
        "grid-template" => {
            if let IrValue::Record { fields } = value {
                if let Some(v) = fields.get("columns").and_then(grid_template) {
                    style.grid_template_columns = v;
                }
                if let Some(v) = fields.get("rows").and_then(grid_template) {
                    style.grid_template_rows = v;
                }
                if let Some(v) = fields.get("areas").and_then(grid_template_areas) {
                    style.grid_template_areas = v;
                }
            }
        }
        "width" => style.size.width = ir_dim(value).unwrap_or(style.size.width),
        "height" => style.size.height = ir_dim(value).unwrap_or(style.size.height),
        "min-width" => style.min_size.width = ir_dim(value).unwrap_or(style.min_size.width),
        "min-height" => style.min_size.height = ir_dim(value).unwrap_or(style.min_size.height),
        "max-width" => style.max_size.width = ir_dim(value).unwrap_or(style.max_size.width),
        "max-height" => style.max_size.height = ir_dim(value).unwrap_or(style.max_size.height),
        "flex-wrap" => {
            style.flex_wrap = value
                .keyword()
                .and_then(|v| taffy::FlexWrap::from_str(v).ok())
                .unwrap_or(style.flex_wrap);
        }
        "aspect-ratio" => {
            style.aspect_ratio = value.number().or(style.aspect_ratio);
        }
        "position" => {
            // taffy models only `relative` (default, in-flow) and `absolute`
            // (out-of-flow, inset offsets from the parent's content box).
            // `static`/`fixed`/`sticky` have no taffy equivalent → relative.
            style.position = match value.keyword() {
                Some("absolute") => taffy::Position::Absolute,
                _ => taffy::Position::Relative,
            };
        }
        "top" => {
            if let Some(v) = ir_lpa(value) {
                style.inset.top = v;
            }
        }
        "right" => {
            if let Some(v) = ir_lpa(value) {
                style.inset.right = v;
            }
        }
        "bottom" => {
            if let Some(v) = ir_lpa(value) {
                style.inset.bottom = v;
            }
        }
        "left" => {
            if let Some(v) = ir_lpa(value) {
                style.inset.left = v;
            }
        }
        "gap" => {
            if let IrValue::Record { fields } = value {
                if let Some(v) = fields.get("column").and_then(ir_lp) {
                    style.gap.width = v;
                }
                if let Some(v) = fields.get("row").and_then(ir_lp) {
                    style.gap.height = v;
                }
            } else if let Some(v) = ir_lp(value) {
                style.gap.width = v;
                style.gap.height = v;
            }
        }
        "row-gap" | "gap-y" => {
            if let Some(v) = ir_lp(value) {
                style.gap.height = v;
            }
        }
        "column-gap" | "gap-x" => {
            if let Some(v) = ir_lp(value) {
                style.gap.width = v;
            }
        }
        "padding-top" => {
            if let Some(v) = ir_lp(value) {
                style.padding.top = v;
            }
        }
        "padding-right" => {
            if let Some(v) = ir_lp(value) {
                style.padding.right = v;
            }
        }
        "padding-bottom" => {
            if let Some(v) = ir_lp(value) {
                style.padding.bottom = v;
            }
        }
        "padding-left" => {
            if let Some(v) = ir_lp(value) {
                style.padding.left = v;
            }
        }
        "padding" => {
            if let IrValue::Record { fields } = value {
                if let Some(v) = fields.get("top").and_then(ir_lp) {
                    style.padding.top = v;
                }
                if let Some(v) = fields.get("right").and_then(ir_lp) {
                    style.padding.right = v;
                }
                if let Some(v) = fields.get("bottom").and_then(ir_lp) {
                    style.padding.bottom = v;
                }
                if let Some(v) = fields.get("left").and_then(ir_lp) {
                    style.padding.left = v;
                }
            } else if let Some(v) = ir_lp(value) {
                style.padding = taffy::Rect {
                    top: v,
                    right: v,
                    bottom: v,
                    left: v,
                };
            }
        }
        "margin-top" => {
            if let Some(v) = ir_lpa(value) {
                style.margin.top = v;
            }
        }
        "margin-right" | "margin-inline-end" => {
            if let Some(v) = ir_lpa(value) {
                style.margin.right = v;
            }
        }
        "margin-bottom" => {
            if let Some(v) = ir_lpa(value) {
                style.margin.bottom = v;
            }
        }
        "margin-left" | "margin-inline-start" => {
            if let Some(v) = ir_lpa(value) {
                style.margin.left = v;
            }
        }
        "margin" => {
            if let IrValue::Record { fields } = value {
                if let Some(v) = fields.get("top").and_then(ir_lpa) {
                    style.margin.top = v;
                }
                if let Some(v) = fields.get("right").and_then(ir_lpa) {
                    style.margin.right = v;
                }
                if let Some(v) = fields.get("bottom").and_then(ir_lpa) {
                    style.margin.bottom = v;
                }
                if let Some(v) = fields.get("left").and_then(ir_lpa) {
                    style.margin.left = v;
                }
            }
        }
        "border-radius" => {
            if let Some(IrLength::Px { value }) = value.length() {
                paint.border_radius = *value;
            }
        }
        "background-color" | "background" => {
            paint.background = ir_color(value).or(paint.background)
        }
        "color" => paint.text_color = ir_color(value).or(paint.text_color),
        "font-size" => {
            if let Some(IrLength::Px { value }) = value.length() {
                paint.font_size = Some(*value);
            }
        }
        "font-family" => {
            if let Some(stack) = font_family_stack(value) {
                paint.font_family = Some(stack);
            }
        }
        "font-weight" => {
            let fw = match value.keyword() {
                Some("normal") => Some(400.0),
                Some("bold") => Some(700.0),
                _ => value.number(),
            };
            paint.font_weight = fw.or(paint.font_weight);
        }
        "line-height" => match value {
            IrValue::Number { value } => paint.line_height = Some((*value, true)),
            IrValue::Length {
                value: IrLength::Px { value },
            } => {
                paint.line_height = Some((*value, false));
            }
            _ => {}
        },
        "overflow" => {
            if let IrValue::Record { fields } = value {
                if let Some(v) = fields.get("x").and_then(ir_overflow) {
                    style.overflow.x = v;
                }
                if let Some(v) = fields.get("y").and_then(ir_overflow) {
                    style.overflow.y = v;
                }
            } else if let Some(v) = ir_overflow(value) {
                style.overflow.x = v;
                style.overflow.y = v;
            }
        }
        "overflow-x" => {
            if let Some(v) = ir_overflow(value) {
                style.overflow.x = v;
            }
        }
        "overflow-y" => {
            if let Some(v) = ir_overflow(value) {
                style.overflow.y = v;
            }
        }
        "border-width" => {
            if let IrValue::Record { fields } = value {
                let mut max_width = 0.0f32;
                for (side, target) in [
                    ("top", &mut style.border.top),
                    ("right", &mut style.border.right),
                    ("bottom", &mut style.border.bottom),
                    ("left", &mut style.border.left),
                ] {
                    if let Some(IrLength::Px { value }) = fields.get(side).and_then(IrValue::length)
                    {
                        *target = taffy::LengthPercentage::length(*value);
                        max_width = max_width.max(*value);
                    }
                }
                paint.border = Some((
                    max_width,
                    paint.border.map_or(Color::BLACK, |(_, color)| color),
                ));
            } else if let Some(IrLength::Px { value }) = value.length() {
                style.border = rect_lp_uniform(*value);
                paint.border = Some((
                    *value,
                    paint.border.map_or(Color::BLACK, |(_, color)| color),
                ));
            }
        }
        "border-color" => {
            if let Some(color) = ir_color(value) {
                paint.border = Some((paint.border.map_or(1.0, |(width, _)| width), color));
            }
        }
        "border-top-width" | "border-right-width" | "border-bottom-width" | "border-left-width" => {
            if let Some(IrLength::Px { value }) = value.length() {
                let width = taffy::LengthPercentage::length(*value);
                match property {
                    "border-top-width" => style.border.top = width,
                    "border-right-width" => style.border.right = width,
                    "border-bottom-width" => style.border.bottom = width,
                    "border-left-width" => style.border.left = width,
                    _ => {}
                }
                paint.border = Some((
                    *value,
                    paint.border.map_or(Color::BLACK, |(_, color)| color),
                ));
            }
        }
        "box-sizing" => {
            style.box_sizing = if value.keyword() == Some("content-box") {
                taffy::BoxSizing::ContentBox
            } else {
                taffy::BoxSizing::BorderBox
            };
        }
        "white-space" => {
            // Record the declaration only. Effective wrap is resolved against
            // the parent during inherit — never baked into a Paint default.
            let wrap = !matches!(value.keyword(), Some("nowrap" | "pre"));
            paint.wrap_text = Some(wrap);
        }
        "text-align" => {
            paint.text_align = Some(match value.keyword() {
                Some("center") => TextAlign::Center,
                Some("right" | "end") => TextAlign::End,
                Some("justify") => TextAlign::Justify,
                _ => TextAlign::Start,
            });
        }
        "opacity" => paint.opacity = value.number().unwrap_or(paint.opacity).clamp(0.0, 1.0),
        "z-index" => {
            // Slint/Qt-Quick z model: sibling-relative paint order, no DOM
            // stacking context. `auto` and non-numbers sort as 0.
            paint.z_index = match value {
                IrValue::Number { value } => *value as i32,
                _ => 0,
            };
        }
        "pointer-events" => {
            // `none` makes the node transparent to hit testing; descendants may
            // re-enable with `auto` (children still hit). Other CSS values
            // (`visiblePainted`, `all`, …) all behave as the default `auto`.
            paint.pointer_events = value.keyword() != Some("none");
        }
        "transform" => {
            let IrValue::List { values } = value else {
                return false;
            };
            paint.transform = values.iter().filter_map(PaintTransform::from_ir).collect();
        }
        "transform-translate-x" | "transform-translate-y" => {
            let IrValue::List { values } = value else {
                return false;
            };
            let Some(PaintTransform::Translate(x, y)) =
                values.iter().find_map(PaintTransform::from_ir)
            else {
                return true;
            };
            if let Some(PaintTransform::Translate(current_x, current_y)) = paint
                .transform
                .iter_mut()
                .find(|item| matches!(item, PaintTransform::Translate(_, _)))
            {
                if property == "transform-translate-x" {
                    *current_x = x;
                } else {
                    *current_y = y;
                }
            } else {
                paint.transform.push(PaintTransform::Translate(x, y));
            }
        }
        "transform-scale" => {
            let IrValue::List { values } = value else {
                return false;
            };
            let Some(component @ PaintTransform::Scale(_, _)) =
                values.iter().find_map(PaintTransform::from_ir)
            else {
                return true;
            };
            if let Some(current @ PaintTransform::Scale(_, _)) = paint
                .transform
                .iter_mut()
                .find(|item| matches!(item, PaintTransform::Scale(_, _)))
            {
                *current = component;
            } else {
                paint.transform.push(component);
            }
        }
        "transform-rotate" => {
            let IrValue::List { values } = value else {
                return false;
            };
            let Some(component @ PaintTransform::Rotate(_)) =
                values.iter().find_map(PaintTransform::from_ir)
            else {
                return true;
            };
            if let Some(current @ PaintTransform::Rotate(_)) = paint
                .transform
                .iter_mut()
                .find(|item| matches!(item, PaintTransform::Rotate(_)))
            {
                *current = component;
            } else {
                paint.transform.push(component);
            }
        }
        "transform-component" => {
            let IrValue::List { values } = value else {
                return false;
            };
            paint
                .transform
                .extend(values.iter().filter_map(PaintTransform::from_ir));
        }
        "box-shadow" => {
            if let IrValue::List { values } = value {
                paint.shadows = values.iter().filter_map(Shadow::from_ir).collect();
            }
        }
        "user-select" => match value.keyword() {
            Some("none") => {
                paint.text_selectable = Some(false);
                paint.text_select_all = Some(false);
            }
            Some("all") => {
                paint.text_selectable = Some(true);
                paint.text_select_all = Some(true);
            }
            Some("auto") | Some("text") => {
                paint.text_selectable = Some(true);
                paint.text_select_all = Some(false);
            }
            _ => {}
        },
        _ => return false,
    }
    true
}

fn ir_overflow(value: &IrValue) -> Option<taffy::Overflow> {
    match value.keyword()? {
        "visible" => Some(taffy::Overflow::Visible),
        "hidden" | "clip" => Some(taffy::Overflow::Hidden),
        "scroll" | "auto" => Some(taffy::Overflow::Scroll),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaintTransform {
    Translate(IrLength, IrLength),
    Scale(f32, f32),
    Rotate(f32),
    Skew(f32, f32),
    Matrix([f32; 6]),
}

impl PaintTransform {
    fn from_ir(value: &IrValue) -> Option<Self> {
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
    fn from_ir(value: &IrValue) -> Option<Self> {
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
fn rect_lp_uniform(w: f32) -> taffy::Rect<taffy::LengthPercentage> {
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
        let (r, g, b) = (*n.get(0)?, *n.get(1)?, *n.get(2)?);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn keyword(value: &str) -> IrValue {
        IrValue::Keyword {
            value: value.into(),
        }
    }
    fn number(value: f32) -> IrValue {
        IrValue::Number { value }
    }
    fn px(value: f32) -> IrValue {
        IrValue::Length {
            value: IrLength::Px { value },
        }
    }
    fn record(fields: impl IntoIterator<Item = (&'static str, IrValue)>) -> IrValue {
        IrValue::Record {
            fields: fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        }
    }

    #[test]
    fn preserves_and_normalizes_font_family_fallbacks() {
        let value = IrValue::List {
            values: [
                "ui-monospace",
                "SFMono-Regular",
                "Menlo",
                "Liberation Mono",
                "monospace",
            ]
            .into_iter()
            .map(keyword)
            .collect(),
        };
        let mut layout = taffy::Style::default();
        let mut paint = DeclaredPaint::default();
        assert!(apply_ir(&mut layout, &mut paint, "font-family", &value));
        assert_eq!(
            paint.font_family.as_deref(),
            Some("monospace, \"SFMono-Regular\", \"Menlo\", \"Liberation Mono\", monospace")
        );

        let sans = IrValue::List {
            values: ["ui-sans-serif", "system-ui", "Noto Sans", "sans-serif"]
                .into_iter()
                .map(keyword)
                .collect(),
        };
        assert!(apply_ir(&mut layout, &mut paint, "font-family", &sans));
        assert_eq!(
            paint.font_family.as_deref(),
            Some("sans-serif, \"Noto Sans\", sans-serif")
        );
    }

    #[test]
    fn maps_non_wrapping_non_shrinking_badges() {
        let mut layout = taffy::Style::default();
        let mut paint = DeclaredPaint::default();

        assert!(apply_ir(
            &mut layout,
            &mut paint,
            "white-space",
            &keyword("nowrap")
        ));
        assert!(apply_ir(
            &mut layout,
            &mut paint,
            "flex-shrink",
            &number(0.0)
        ));

        // Cascade records the declaration only; inherit resolves effective wrap.
        assert_eq!(paint.wrap_text, Some(false));
        assert_eq!(layout.flex_shrink, 0.0);
        let computed = paint.resolve(&InheritedPaint::default(), HostPaint::default());
        assert!(!computed.wrap_text);
    }

    #[test]
    fn white_space_inherit_is_not_confused_with_initial() {
        // Parent declares nowrap; child declares nothing → child must not wrap.
        let parent = DeclaredPaint {
            wrap_text: Some(false),
            ..DeclaredPaint::default()
        }
        .resolve_inherited(&InheritedPaint::default());
        let child = DeclaredPaint::default().resolve(&parent, HostPaint::default());
        assert!(!child.wrap_text);

        // Child explicitly declares normal → wraps even under nowrap parent.
        let child_normal = DeclaredPaint {
            wrap_text: Some(true),
            ..DeclaredPaint::default()
        }
        .resolve(&parent, HostPaint::default());
        assert!(child_normal.wrap_text);
    }

    #[test]
    fn maps_repeated_minmax_grid_tracks() {
        let breadth = |kind, value| record([("kind", keyword(kind)), ("value", value)]);
        let minmax = record([
            ("kind", keyword("minmax")),
            ("min", breadth("length", px(0.0))),
            ("max", breadth("flex", number(1.0))),
        ]);
        let value = IrValue::List {
            values: vec![record([
                ("kind", keyword("repeat")),
                ("count", number(3.0)),
                (
                    "values",
                    IrValue::List {
                        values: vec![minmax],
                    },
                ),
            ])],
        };
        let mut layout = taffy::Style::default();
        let mut paint = DeclaredPaint::default();
        assert!(apply_ir(
            &mut layout,
            &mut paint,
            "grid-template-columns",
            &value
        ));
        let GridTemplateComponent::Repeat(repeat) = &layout.grid_template_columns[0] else {
            panic!()
        };
        assert_eq!(repeat.count, RepetitionCount::Count(3));
        assert_eq!(repeat.tracks.len(), 1);

        let areas = record([
            ("columns", number(2.0)),
            (
                "cells",
                IrValue::List {
                    values: vec![
                        keyword("head"),
                        keyword("head"),
                        keyword("nav"),
                        keyword("main"),
                    ],
                },
            ),
        ]);
        assert!(apply_ir(
            &mut layout,
            &mut paint,
            "grid-template-areas",
            &areas
        ));
        let head = layout
            .grid_template_areas
            .iter()
            .find(|area| area.name == "head")
            .unwrap();
        assert_eq!(
            (
                head.row_start,
                head.row_end,
                head.column_start,
                head.column_end
            ),
            (1, 2, 1, 3)
        );
    }

    #[test]
    fn maps_vello_paint_properties() {
        let mut layout = taffy::Style::default();
        let mut paint = DeclaredPaint::default();
        assert!(apply_ir(&mut layout, &mut paint, "opacity", &number(0.4)));
        let transform = IrValue::List {
            values: vec![record([
                ("kind", keyword("scale")),
                (
                    "value",
                    IrValue::List {
                        values: vec![number(2.0), number(3.0)],
                    },
                ),
            ])],
        };
        assert!(apply_ir(&mut layout, &mut paint, "transform", &transform));
        let shadow = IrValue::List {
            values: vec![record([
                ("x", px(1.0)),
                ("y", px(2.0)),
                ("stdDev", px(8.0)),
                ("spread", px(0.0)),
                (
                    "color",
                    IrValue::Color {
                        value: IrColor::Literal { rgba: 0x00000080 },
                    },
                ),
            ])],
        };
        assert!(apply_ir(&mut layout, &mut paint, "box-shadow", &shadow));
        assert_eq!(paint.opacity, 0.4);
        assert_eq!(paint.transform, vec![PaintTransform::Scale(2.0, 3.0)]);
        assert_eq!(paint.shadows.len(), 1);
        assert_eq!(paint.shadows[0].std_dev, 8.0);
    }

    #[test]
    fn inline_percentages_are_normalized_for_taffy() {
        assert_eq!(
            parse_ir_value("100%"),
            IrValue::Length {
                value: IrLength::Percent { value: 1.0 }
            }
        );
        assert_eq!(
            parse_ir_value("25%"),
            IrValue::Length {
                value: IrLength::Percent { value: 0.25 }
            }
        );
    }
}
