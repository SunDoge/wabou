//! Style IR types, the single [`apply_ir`] application backend, and the
//! runtime string→`IrValue` parser [`parse_ir_value`] used for inline styles.

#![warn(missing_docs)]

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use taffy::prelude::*;
use taffy::style::{GridTemplateArea, GridTemplateAreas, GridTemplateRepetition};
use vello::peniko::Color;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
/// Resolved horizontal alignment for shaped text.
pub enum TextAlign {
    #[default]
    /// Start edge determined by writing direction.
    Start,
    /// Center each line in its available width.
    Center,
    /// End edge determined by writing direction.
    End,
    /// Expand inter-word spacing to fill each eligible line.
    Justify,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "unit", rename_all = "kebab-case")]
/// Length representation accepted by the runtime Style IR.
pub enum IrLength {
    /// Logical-pixel length.
    Px {
        /// Logical-pixel magnitude.
        value: f32,
    },
    /// Parent-relative ratio in Taffy's `0.0..=1.0` representation.
    Percent {
        /// Parent-relative ratio.
        value: f32,
    },
    /// Automatic value resolved by layout.
    Auto,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
/// Color representation accepted by the runtime Style IR.
pub enum IrColor {
    /// Packed RGBA channels in network byte order (`0xRRGGBBAA`).
    Literal {
        /// Packed channel value.
        rgba: u32,
    },
    /// Theme token resolved before paint application.
    Token {
        /// Theme color name.
        name: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
/// Typed value transported from the style compiler to the Rust backend.
pub enum IrValue {
    /// Property-specific closed-vocabulary keyword.
    Keyword {
        /// Canonical keyword spelling.
        value: String,
    },
    /// Boolean value.
    Boolean {
        /// Boolean payload.
        value: bool,
    },
    /// Finite unitless number.
    Number {
        /// Numeric payload.
        value: f32,
    },
    /// Typed layout length.
    Length {
        /// Length payload.
        value: IrLength,
    },
    /// Typed color.
    Color {
        /// Color payload.
        value: IrColor,
    },
    /// Ordered composite value.
    List {
        /// Child values.
        values: Vec<IrValue>,
    },
    /// Named composite value.
    Record {
        /// Child values keyed by schema field name.
        fields: HashMap<String, IrValue>,
    },
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

fn grid_template_areas(value: &IrValue) -> Option<GridTemplateAreas<String>> {
    let columns = field(value, "columns")?.number()? as usize;
    let IrValue::List { values } = field(value, "cells")? else {
        return None;
    };
    if columns == 0 {
        return Some(GridTemplateAreas {
            areas: Vec::new(),
            row_count: 0,
            column_count: 0,
        });
    }
    if values.len() % columns != 0 {
        return None;
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
    Some(GridTemplateAreas {
        areas: bounds
            .into_iter()
            .map(|(name, (r0, r1, c0, c1))| GridTemplateArea {
                name: name.to_owned(),
                row_start: r0 as u16 + 1,
                row_end: r1 as u16 + 2,
                column_start: c0 as u16 + 1,
                column_end: c1 as u16 + 2,
            })
            .collect(),
        row_count: (values.len() / columns).try_into().ok()?,
        column_count: columns.try_into().ok()?,
    })
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
    let IrColor::Literal { rgba } = value else {
        return None;
    };
    Some(Color::from_rgba8(
        (rgba >> 24) as u8,
        (rgba >> 16) as u8,
        (rgba >> 8) as u8,
        *rgba as u8,
    ))
}

/// Parse a CSS length string into a Style IR length: `Npx` → `Px`,
/// `Nrem`/`Nem` → `Px` at the 16px root font size, `N%` → `Percent`.
/// Anything else returns `None`.
///
/// `rem` is matched before `em` because `1rem` also ends with `em`. Both
/// units use the root size; parent-relative `em` is not resolved here
/// (`apply_ir` does not receive inherited `font-size`).
fn parse_ir_length(value: &str) -> Option<IrLength> {
    let v = value.trim();
    if let Some(r) = v.strip_suffix("rem") {
        return r
            .trim()
            .parse::<f32>()
            .ok()
            .map(|n| IrLength::Px { value: n * 16.0 });
    }
    if let Some(em) = v.strip_suffix("em") {
        return em
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

fn apply_transform_ir(paint: &mut DeclaredPaint, property: &str, value: &IrValue) -> Option<bool> {
    let values = match property {
        "transform"
        | "transform-translate-x"
        | "transform-translate-y"
        | "transform-scale"
        | "transform-rotate"
        | "transform-component" => match value {
            IrValue::List { values } => values,
            _ => return Some(false),
        },
        _ => return None,
    };
    match property {
        "transform" => {
            paint.transform = values.iter().filter_map(PaintTransform::from_ir).collect();
        }
        "transform-translate-x" | "transform-translate-y" => {
            let Some(PaintTransform::Translate(x, y)) =
                values.iter().find_map(PaintTransform::from_ir)
            else {
                return Some(true);
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
            let Some(component @ PaintTransform::Scale(_, _)) =
                values.iter().find_map(PaintTransform::from_ir)
            else {
                return Some(true);
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
            let Some(component @ PaintTransform::Rotate(_)) =
                values.iter().find_map(PaintTransform::from_ir)
            else {
                return Some(true);
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
        "transform-component" => paint
            .transform
            .extend(values.iter().filter_map(PaintTransform::from_ir)),
        _ => unreachable!("transform properties were filtered above"),
    }
    Some(true)
}

fn apply_paint_ir(paint: &mut DeclaredPaint, property: &str, value: &IrValue) -> Option<bool> {
    if let Some(applied) = apply_transform_ir(paint, property, value) {
        return Some(applied);
    }
    match property {
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
            let weight = match value.keyword() {
                Some("normal") => Some(400.0),
                Some("bold") => Some(700.0),
                _ => value.number(),
            };
            paint.font_weight = weight.or(paint.font_weight);
        }
        "line-height" => match value {
            IrValue::Number { value } => paint.line_height = Some((*value, true)),
            IrValue::Length {
                value: IrLength::Px { value },
            } => paint.line_height = Some((*value, false)),
            _ => {}
        },
        "white-space" => {
            paint.wrap_text = Some(!matches!(value.keyword(), Some("nowrap" | "pre")));
        }
        "text-overflow" => paint.text_ellipsis = value.keyword() == Some("ellipsis"),
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
            // Sibling-relative paint order; `auto` and non-numbers sort as 0.
            paint.z_index = match value {
                IrValue::Number { value } => *value as i32,
                _ => 0,
            };
        }
        "pointer-events" => paint.pointer_events = value.keyword() != Some("none"),
        "cursor" => {
            paint.cursor = Some(match value.keyword()? {
                "pointer" => CursorStyle::Pointer,
                "text" => CursorStyle::Text,
                "crosshair" => CursorStyle::Crosshair,
                "move" => CursorStyle::Move,
                "wait" => CursorStyle::Wait,
                "not-allowed" => CursorStyle::NotAllowed,
                "ew-resize" | "col-resize" => CursorStyle::EwResize,
                "ns-resize" | "row-resize" => CursorStyle::NsResize,
                "auto" | "default" => CursorStyle::Default,
                _ => return None,
            });
        }
        "outline-width" => {
            if let Some(IrLength::Px { value }) = value.length() {
                paint.outline_width = (*value).max(0.0);
            }
        }
        "outline-offset" => {
            if let Some(IrLength::Px { value }) = value.length() {
                paint.outline_offset = *value;
            }
        }
        "outline-color" => paint.outline_color = ir_color(value).or(paint.outline_color),
        "outline-style" => match value.keyword() {
            Some("none") => paint.outline_width = 0.0,
            Some("solid") => {}
            _ => {}
        },
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
        _ => return None,
    }
    Some(true)
}

fn apply_sizing_ir(style: &mut taffy::Style, property: &str, value: &IrValue) -> Option<bool> {
    match property {
        "width" => style.size.width = ir_dim(value).unwrap_or(style.size.width),
        "height" => style.size.height = ir_dim(value).unwrap_or(style.size.height),
        "min-width" => style.min_size.width = ir_lpa(value).unwrap_or(style.min_size.width),
        "min-height" => style.min_size.height = ir_lpa(value).unwrap_or(style.min_size.height),
        "max-width" => style.max_size.width = ir_lpa(value).unwrap_or(style.max_size.width),
        "max-height" => style.max_size.height = ir_lpa(value).unwrap_or(style.max_size.height),
        "aspect-ratio" => style.aspect_ratio = value.number().or(style.aspect_ratio),
        "position" => {
            // Taffy has no distinct static/fixed/sticky positioning model.
            style.position = if value.keyword() == Some("absolute") {
                taffy::Position::Absolute
            } else {
                taffy::Position::Relative
            };
        }
        "top" => {
            if let Some(inset) = ir_lpa(value) {
                style.inset.top = inset;
            }
        }
        "right" => {
            if let Some(inset) = ir_lpa(value) {
                style.inset.right = inset;
            }
        }
        "bottom" => {
            if let Some(inset) = ir_lpa(value) {
                style.inset.bottom = inset;
            }
        }
        "left" => {
            if let Some(inset) = ir_lpa(value) {
                style.inset.left = inset;
            }
        }
        "box-sizing" => {
            style.box_sizing = if value.keyword() == Some("content-box") {
                taffy::BoxSizing::ContentBox
            } else {
                taffy::BoxSizing::BorderBox
            };
        }
        _ => return None,
    }
    Some(true)
}

fn apply_gap_padding_ir(style: &mut taffy::Style, property: &str, value: &IrValue) -> Option<bool> {
    match property {
        "gap" => {
            if let IrValue::Record { fields } = value {
                if let Some(column) = fields.get("column").and_then(ir_lp) {
                    style.gap.width = column;
                }
                if let Some(row) = fields.get("row").and_then(ir_lp) {
                    style.gap.height = row;
                }
            } else if let Some(gap) = ir_lp(value) {
                style.gap.width = gap;
                style.gap.height = gap;
            }
        }
        "row-gap" | "gap-y" => {
            if let Some(gap) = ir_lp(value) {
                style.gap.height = gap;
            }
        }
        "column-gap" | "gap-x" => {
            if let Some(gap) = ir_lp(value) {
                style.gap.width = gap;
            }
        }
        "padding-top" => {
            if let Some(padding) = ir_lp(value) {
                style.padding.top = padding;
            }
        }
        "padding-right" => {
            if let Some(padding) = ir_lp(value) {
                style.padding.right = padding;
            }
        }
        "padding-bottom" => {
            if let Some(padding) = ir_lp(value) {
                style.padding.bottom = padding;
            }
        }
        "padding-left" => {
            if let Some(padding) = ir_lp(value) {
                style.padding.left = padding;
            }
        }
        "padding" => {
            if let IrValue::Record { fields } = value {
                if let Some(padding) = fields.get("top").and_then(ir_lp) {
                    style.padding.top = padding;
                }
                if let Some(padding) = fields.get("right").and_then(ir_lp) {
                    style.padding.right = padding;
                }
                if let Some(padding) = fields.get("bottom").and_then(ir_lp) {
                    style.padding.bottom = padding;
                }
                if let Some(padding) = fields.get("left").and_then(ir_lp) {
                    style.padding.left = padding;
                }
            } else if let Some(padding) = ir_lp(value) {
                style.padding = taffy::Rect {
                    top: padding,
                    right: padding,
                    bottom: padding,
                    left: padding,
                };
            }
        }
        _ => return None,
    }
    Some(true)
}

fn apply_margin_ir(style: &mut taffy::Style, property: &str, value: &IrValue) -> Option<bool> {
    match property {
        "margin-top" => {
            if let Some(margin) = ir_lpa(value) {
                style.margin.top = margin;
            }
        }
        "margin-right" | "margin-inline-end" => {
            if let Some(margin) = ir_lpa(value) {
                style.margin.right = margin;
            }
        }
        "margin-bottom" => {
            if let Some(margin) = ir_lpa(value) {
                style.margin.bottom = margin;
            }
        }
        "margin-left" | "margin-inline-start" => {
            if let Some(margin) = ir_lpa(value) {
                style.margin.left = margin;
            }
        }
        "margin" => {
            if let IrValue::Record { fields } = value {
                if let Some(margin) = fields.get("top").and_then(ir_lpa) {
                    style.margin.top = margin;
                }
                if let Some(margin) = fields.get("right").and_then(ir_lpa) {
                    style.margin.right = margin;
                }
                if let Some(margin) = fields.get("bottom").and_then(ir_lpa) {
                    style.margin.bottom = margin;
                }
                if let Some(margin) = fields.get("left").and_then(ir_lpa) {
                    style.margin.left = margin;
                }
            } else if let Some(margin) = ir_lpa(value) {
                style.margin = taffy::Rect {
                    top: margin,
                    right: margin,
                    bottom: margin,
                    left: margin,
                };
            }
        }
        _ => return None,
    }
    Some(true)
}

fn apply_overflow_border_ir(
    style: &mut taffy::Style,
    paint: &mut DeclaredPaint,
    property: &str,
    value: &IrValue,
) -> Option<bool> {
    match property {
        "overflow" => {
            if let IrValue::Record { fields } = value {
                if let Some(overflow) = fields.get("x").and_then(ir_overflow) {
                    style.overflow.x = overflow;
                }
                if let Some(overflow) = fields.get("y").and_then(ir_overflow) {
                    style.overflow.y = overflow;
                }
            } else if let Some(overflow) = ir_overflow(value) {
                style.overflow.x = overflow;
                style.overflow.y = overflow;
            }
        }
        "overflow-x" => {
            if let Some(overflow) = ir_overflow(value) {
                style.overflow.x = overflow;
            }
        }
        "overflow-y" => {
            if let Some(overflow) = ir_overflow(value) {
                style.overflow.y = overflow;
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
                    _ => unreachable!(),
                }
                paint.border = Some((
                    *value,
                    paint.border.map_or(Color::BLACK, |(_, color)| color),
                ));
            }
        }
        _ => return None,
    }
    Some(true)
}

/// Apply an already parsed Style IR declaration into cascaded layout +
/// [`DeclaredPaint`]. This path performs no CSS parsing or unit normalization
/// at runtime, and never writes computed/inherited values — those are produced
/// later by resolving [`DeclaredPaint`] against the parent.
///
/// # Support contract
///
/// Property coverage is the CSS support matrix
/// (`packages/vite/src/style-compiler/css-support-matrix.json`):
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
    if let Some(applied) = apply_paint_ir(paint, property, value) {
        return applied;
    }
    if let Some(applied) = apply_sizing_ir(style, property, value) {
        return applied;
    }
    if let Some(applied) = apply_gap_padding_ir(style, property, value) {
        return applied;
    }
    if let Some(applied) = apply_margin_ir(style, property, value) {
        return applied;
    }
    if let Some(applied) = apply_overflow_border_ir(style, paint, property, value) {
        return applied;
    }
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
                Some("flow-root") => taffy::Display::FlowRoot,
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
                style.grid_template_areas = Some(v);
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
                    style.grid_template_areas = Some(v);
                }
            }
        }
        "flex-wrap" => {
            style.flex_wrap = value
                .keyword()
                .and_then(|v| taffy::FlexWrap::from_str(v).ok())
                .unwrap_or(style.flex_wrap);
        }
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

mod paint;
pub use paint::*;

#[cfg(test)]
mod tests;
