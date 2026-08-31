use gpui::{
    AbsoluteLength, AlignContent, AlignItems, BoxShadow, CursorStyle, DefiniteLength, Display,
    FlexDirection, FlexWrap, FontStyle, FontWeight, Hsla, Length, Overflow, Position, Style,
    TextAlign, TextOverflow, Visibility, WhiteSpace,
};
use wabou_style::{Color, Declaration, Length as IrLength, Value};

/// A deterministic reason why a Style IR declaration was not projected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StyleDiagnostic {
    /// This migration stage does not implement the otherwise valid property.
    UnsupportedProperty(String),
    /// The value does not match the property's required closed vocabulary.
    InvalidValue { property: String },
}

/// Accumulates ordered Style IR declarations into one GPUI style.
///
/// This is the production layout bridge. GPUI remains responsible for creating
/// its internal Taffy nodes; Wabou never owns or persists GPUI `LayoutId`s.
#[derive(Clone, Debug, Default)]
pub struct StyleProjection {
    style: Style,
}

impl StyleProjection {
    /// Continue projecting declarations over an already resolved GPUI style.
    #[must_use]
    pub fn from_style(style: Style) -> Self {
        Self { style }
    }

    #[must_use]
    pub fn style(&self) -> &Style {
        &self.style
    }

    #[must_use]
    pub fn into_style(self) -> Style {
        self.style
    }

    pub fn apply(&mut self, declaration: &Declaration) -> Result<(), StyleDiagnostic> {
        let property = declaration.property.as_str();
        let value = &declaration.value;
        match property {
            "display" => self.style.display = display(value).ok_or_else(|| invalid(property))?,
            "position" => {
                self.style.position = position(value).ok_or_else(|| invalid(property))?;
            }
            "width" => self.style.size.width = length(value).ok_or_else(|| invalid(property))?,
            "height" => self.style.size.height = length(value).ok_or_else(|| invalid(property))?,
            "min-width" => {
                self.style.min_size.width = length(value).ok_or_else(|| invalid(property))?;
            }
            "min-height" => {
                self.style.min_size.height = length(value).ok_or_else(|| invalid(property))?;
            }
            "max-width" => {
                self.style.max_size.width = length(value).ok_or_else(|| invalid(property))?;
            }
            "max-height" => {
                self.style.max_size.height = length(value).ok_or_else(|| invalid(property))?;
            }
            "aspect-ratio" => {
                self.style.aspect_ratio = Some(
                    number(value)
                        .filter(|ratio| *ratio > 0.0)
                        .ok_or_else(|| invalid(property))?,
                );
            }
            "top" => self.style.inset.top = length(value).ok_or_else(|| invalid(property))?,
            "right" => self.style.inset.right = length(value).ok_or_else(|| invalid(property))?,
            "bottom" => self.style.inset.bottom = length(value).ok_or_else(|| invalid(property))?,
            "left" => self.style.inset.left = length(value).ok_or_else(|| invalid(property))?,
            "margin" => {
                if let Some((top, right, bottom, left)) = length_edges(value) {
                    self.style.margin.top = top;
                    self.style.margin.right = right;
                    self.style.margin.bottom = bottom;
                    self.style.margin.left = left;
                } else {
                    let margin = length(value).ok_or_else(|| invalid(property))?;
                    self.style.margin.top = margin;
                    self.style.margin.right = margin;
                    self.style.margin.bottom = margin;
                    self.style.margin.left = margin;
                }
            }
            "margin-top" => {
                self.style.margin.top = length(value).ok_or_else(|| invalid(property))?;
            }
            "margin-right" | "margin-inline-end" => {
                self.style.margin.right = length(value).ok_or_else(|| invalid(property))?;
            }
            "margin-bottom" => {
                self.style.margin.bottom = length(value).ok_or_else(|| invalid(property))?;
            }
            "margin-left" | "margin-inline-start" => {
                self.style.margin.left = length(value).ok_or_else(|| invalid(property))?;
            }
            "padding" => {
                if let Some((top, right, bottom, left)) = definite_edges(value) {
                    self.style.padding.top = top;
                    self.style.padding.right = right;
                    self.style.padding.bottom = bottom;
                    self.style.padding.left = left;
                } else {
                    let padding = definite_length(value).ok_or_else(|| invalid(property))?;
                    self.style.padding.top = padding;
                    self.style.padding.right = padding;
                    self.style.padding.bottom = padding;
                    self.style.padding.left = padding;
                }
            }
            "padding-top" => {
                self.style.padding.top = definite_length(value).ok_or_else(|| invalid(property))?;
            }
            "padding-right" => {
                self.style.padding.right =
                    definite_length(value).ok_or_else(|| invalid(property))?;
            }
            "padding-bottom" => {
                self.style.padding.bottom =
                    definite_length(value).ok_or_else(|| invalid(property))?;
            }
            "padding-left" => {
                self.style.padding.left =
                    definite_length(value).ok_or_else(|| invalid(property))?;
            }
            "gap" => {
                if let Some((column, row)) = axis_lengths(value) {
                    self.style.gap.width = column;
                    self.style.gap.height = row;
                } else {
                    let gap = definite_length(value).ok_or_else(|| invalid(property))?;
                    self.style.gap.width = gap;
                    self.style.gap.height = gap;
                }
            }
            "column-gap" | "gap-x" => {
                self.style.gap.width = definite_length(value).ok_or_else(|| invalid(property))?;
            }
            "row-gap" | "gap-y" => {
                self.style.gap.height = definite_length(value).ok_or_else(|| invalid(property))?;
            }
            "overflow" => {
                if let Some((x, y)) = axis_overflow(value) {
                    self.style.overflow.x = x;
                    self.style.overflow.y = y;
                } else {
                    let overflow = overflow(value).ok_or_else(|| invalid(property))?;
                    self.style.overflow.x = overflow;
                    self.style.overflow.y = overflow;
                }
            }
            "overflow-x" => {
                self.style.overflow.x = overflow(value).ok_or_else(|| invalid(property))?;
            }
            "overflow-y" => {
                self.style.overflow.y = overflow(value).ok_or_else(|| invalid(property))?;
            }
            "flex-direction" => {
                self.style.flex_direction =
                    flex_direction(value).ok_or_else(|| invalid(property))?;
            }
            "flex-wrap" => {
                self.style.flex_wrap = flex_wrap(value).ok_or_else(|| invalid(property))?;
            }
            "flex-grow" => {
                self.style.flex_grow = number(value).ok_or_else(|| invalid(property))?;
            }
            "flex-shrink" => {
                self.style.flex_shrink = number(value).ok_or_else(|| invalid(property))?;
            }
            "flex-basis" => {
                self.style.flex_basis = length(value).ok_or_else(|| invalid(property))?;
            }
            "align-items" => {
                self.style.align_items = Some(align_items(value).ok_or_else(|| invalid(property))?);
            }
            "align-self" => {
                self.style.align_self = Some(align_items(value).ok_or_else(|| invalid(property))?);
            }
            "align-content" => {
                self.style.align_content =
                    Some(align_content(value).ok_or_else(|| invalid(property))?);
            }
            "justify-content" => {
                self.style.justify_content =
                    Some(align_content(value).ok_or_else(|| invalid(property))?);
            }
            "background" | "background-color" => {
                self.style.background = Some(color(value).ok_or_else(|| invalid(property))?.into());
            }
            "color" => {
                self.style.text.color = Some(color(value).ok_or_else(|| invalid(property))?);
            }
            "border-color" => {
                self.style.border_color = Some(color(value).ok_or_else(|| invalid(property))?);
            }
            "border-width" => {
                if let Some((top, right, bottom, left)) = absolute_edges(value) {
                    self.style.border_widths.top = top;
                    self.style.border_widths.right = right;
                    self.style.border_widths.bottom = bottom;
                    self.style.border_widths.left = left;
                } else {
                    let width = absolute_length(value).ok_or_else(|| invalid(property))?;
                    self.style.border_widths.top = width;
                    self.style.border_widths.right = width;
                    self.style.border_widths.bottom = width;
                    self.style.border_widths.left = width;
                }
            }
            "border-top-width" => {
                self.style.border_widths.top =
                    absolute_length(value).ok_or_else(|| invalid(property))?;
            }
            "border-right-width" => {
                self.style.border_widths.right =
                    absolute_length(value).ok_or_else(|| invalid(property))?;
            }
            "border-bottom-width" => {
                self.style.border_widths.bottom =
                    absolute_length(value).ok_or_else(|| invalid(property))?;
            }
            "border-left-width" => {
                self.style.border_widths.left =
                    absolute_length(value).ok_or_else(|| invalid(property))?;
            }
            "border-radius" => {
                let radius = absolute_length(value).ok_or_else(|| invalid(property))?;
                self.style.corner_radii.top_left = radius;
                self.style.corner_radii.top_right = radius;
                self.style.corner_radii.bottom_right = radius;
                self.style.corner_radii.bottom_left = radius;
            }
            "font-size" => {
                self.style.text.font_size =
                    Some(absolute_length(value).ok_or_else(|| invalid(property))?);
            }
            "font-family" => {
                self.style.text.font_family =
                    Some(font_family(value).ok_or_else(|| invalid(property))?.into());
            }
            "font-weight" => {
                let weight = match keyword(value) {
                    Some("normal") => Some(400.0),
                    Some("bold") => Some(700.0),
                    _ => number(value),
                }
                .filter(|weight| (1.0..=1000.0).contains(weight))
                .ok_or_else(|| invalid(property))?;
                self.style.text.font_weight = Some(FontWeight(weight));
            }
            "font-style" => {
                self.style.text.font_style = Some(match keyword(value) {
                    Some("normal") => FontStyle::Normal,
                    Some("italic") => FontStyle::Italic,
                    Some("oblique") => FontStyle::Oblique,
                    _ => return Err(invalid(property)),
                });
            }
            "letter-spacing" => {
                self.style.text.letter_spacing = Some(match value {
                    Value::Keyword { value } if value == "normal" => gpui::px(0.0),
                    _ => pixels(value).ok_or_else(|| invalid(property))?,
                });
            }
            "line-height" => {
                self.style.text.line_height = Some(match value {
                    Value::Number { value } if value.is_finite() && *value >= 0.0 => {
                        DefiniteLength::Fraction(*value)
                    }
                    _ => definite_length(value).ok_or_else(|| invalid(property))?,
                });
            }
            "white-space" => {
                self.style.text.white_space = Some(match keyword(value) {
                    Some("normal") => WhiteSpace::Normal,
                    Some("nowrap" | "pre") => WhiteSpace::Nowrap,
                    _ => return Err(invalid(property)),
                });
            }
            "text-overflow" => {
                self.style.text.text_overflow = match keyword(value) {
                    Some("clip") => None,
                    Some("ellipsis") => Some(TextOverflow::Truncate("…".into())),
                    _ => return Err(invalid(property)),
                };
            }
            "text-align" => {
                self.style.text.text_align = Some(match keyword(value) {
                    Some("left" | "start") => TextAlign::Left,
                    Some("center") => TextAlign::Center,
                    Some("right" | "end") => TextAlign::Right,
                    _ => return Err(invalid(property)),
                });
            }
            "opacity" => {
                self.style.opacity = Some(
                    number(value)
                        .filter(|opacity| (0.0..=1.0).contains(opacity))
                        .ok_or_else(|| invalid(property))?,
                );
            }
            "visibility" => {
                self.style.visibility = match keyword(value) {
                    Some("visible") => Visibility::Visible,
                    Some("hidden") => Visibility::Hidden,
                    _ => return Err(invalid(property)),
                };
            }
            "cursor" => {
                self.style.mouse_cursor = Some(cursor(value).ok_or_else(|| invalid(property))?);
            }
            "box-shadow" => {
                self.style.box_shadow = box_shadows(value).ok_or_else(|| invalid(property))?;
            }
            _ => return Err(StyleDiagnostic::UnsupportedProperty(property.to_owned())),
        }
        Ok(())
    }
}

fn invalid(property: &str) -> StyleDiagnostic {
    StyleDiagnostic::InvalidValue {
        property: property.to_owned(),
    }
}

fn keyword(value: &Value) -> Option<&str> {
    let Value::Keyword { value } = value else {
        return None;
    };
    Some(value)
}

fn number(value: &Value) -> Option<f32> {
    let Value::Number { value } = value else {
        return None;
    };
    value.is_finite().then_some(*value)
}

fn font_family(value: &Value) -> Option<&str> {
    match keyword(value)? {
        "sans-serif" => Some(".SystemUIFont"),
        "monospace" => Some(".SystemUIFontMonospaced"),
        family if !family.is_empty() => Some(family),
        _ => None,
    }
}

fn pixels(value: &Value) -> Option<gpui::Pixels> {
    match ir_length(value)? {
        IrLength::Px { value } if value.is_finite() => Some(gpui::px(*value)),
        _ => None,
    }
}

fn cursor(value: &Value) -> Option<CursorStyle> {
    match keyword(value)? {
        "auto" | "default" => Some(CursorStyle::Arrow),
        "pointer" => Some(CursorStyle::PointingHand),
        "text" => Some(CursorStyle::IBeam),
        "crosshair" => Some(CursorStyle::Crosshair),
        "move" => Some(CursorStyle::OpenHand),
        "not-allowed" => Some(CursorStyle::OperationNotAllowed),
        "ew-resize" | "col-resize" => Some(CursorStyle::ResizeLeftRight),
        "ns-resize" | "row-resize" => Some(CursorStyle::ResizeUpDown),
        _ => None,
    }
}

fn box_shadows(value: &Value) -> Option<Vec<BoxShadow>> {
    let Value::List { values } = value else {
        return None;
    };
    values.iter().map(box_shadow).collect()
}

fn box_shadow(value: &Value) -> Option<BoxShadow> {
    let offset_x = pixels(field(value, "x")?)?;
    let offset_y = pixels(field(value, "y")?)?;
    let blur_radius = pixels(field(value, "stdDev")?)?;
    let spread_radius = pixels(field(value, "spread")?)?;
    let color = color(field(value, "color")?)?;
    Some(
        BoxShadow::new(offset_x, offset_y, color)
            .blur_radius(blur_radius)
            .spread_radius(spread_radius),
    )
}

fn ir_length(value: &Value) -> Option<&IrLength> {
    let Value::Length { value } = value else {
        return None;
    };
    Some(value)
}

fn field<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    let Value::Record { fields } = value else {
        return None;
    };
    fields.get(name)
}

fn length_edges(value: &Value) -> Option<(Length, Length, Length, Length)> {
    Some((
        length(field(value, "top")?)?,
        length(field(value, "right")?)?,
        length(field(value, "bottom")?)?,
        length(field(value, "left")?)?,
    ))
}

fn definite_edges(
    value: &Value,
) -> Option<(
    DefiniteLength,
    DefiniteLength,
    DefiniteLength,
    DefiniteLength,
)> {
    Some((
        definite_length(field(value, "top")?)?,
        definite_length(field(value, "right")?)?,
        definite_length(field(value, "bottom")?)?,
        definite_length(field(value, "left")?)?,
    ))
}

fn absolute_edges(
    value: &Value,
) -> Option<(
    AbsoluteLength,
    AbsoluteLength,
    AbsoluteLength,
    AbsoluteLength,
)> {
    Some((
        absolute_length(field(value, "top")?)?,
        absolute_length(field(value, "right")?)?,
        absolute_length(field(value, "bottom")?)?,
        absolute_length(field(value, "left")?)?,
    ))
}

fn axis_lengths(value: &Value) -> Option<(DefiniteLength, DefiniteLength)> {
    Some((
        definite_length(field(value, "column")?)?,
        definite_length(field(value, "row")?)?,
    ))
}

fn axis_overflow(value: &Value) -> Option<(Overflow, Overflow)> {
    Some((overflow(field(value, "x")?)?, overflow(field(value, "y")?)?))
}

fn definite_length(value: &Value) -> Option<DefiniteLength> {
    match ir_length(value)? {
        IrLength::Px { value } if value.is_finite() => Some(gpui::px(*value).into()),
        IrLength::Percent { value } if value.is_finite() => Some(DefiniteLength::Fraction(*value)),
        IrLength::Px { .. } | IrLength::Percent { .. } | IrLength::Auto => None,
    }
}

fn absolute_length(value: &Value) -> Option<AbsoluteLength> {
    if let Some(value) = number(value).filter(|value| value.is_finite()) {
        return Some(gpui::px(value).into());
    }
    match ir_length(value)? {
        IrLength::Px { value } if value.is_finite() => Some(gpui::px(*value).into()),
        IrLength::Px { .. } | IrLength::Percent { .. } | IrLength::Auto => None,
    }
}

fn color(value: &Value) -> Option<Hsla> {
    let Value::Color {
        value: Color::Literal { rgba },
    } = value
    else {
        return None;
    };
    Some(gpui::rgb_to_hsla(gpui::rgba(*rgba)))
}

fn length(value: &Value) -> Option<Length> {
    if keyword(value) == Some("auto") {
        return Some(Length::Auto);
    }
    match ir_length(value)? {
        IrLength::Auto => Some(Length::Auto),
        _ => definite_length(value).map(Length::Definite),
    }
}

fn display(value: &Value) -> Option<Display> {
    match keyword(value)? {
        "block" | "flow-root" => Some(Display::Block),
        "flex" => Some(Display::Flex),
        "grid" => Some(Display::Grid),
        "none" => Some(Display::None),
        _ => None,
    }
}

fn position(value: &Value) -> Option<Position> {
    match keyword(value)? {
        "relative" | "static" => Some(Position::Relative),
        "absolute" | "fixed" => Some(Position::Absolute),
        _ => None,
    }
}

fn overflow(value: &Value) -> Option<Overflow> {
    match keyword(value)? {
        "visible" => Some(Overflow::Visible),
        "clip" => Some(Overflow::Clip),
        "hidden" => Some(Overflow::Hidden),
        "scroll" | "auto" => Some(Overflow::Scroll),
        _ => None,
    }
}

fn flex_direction(value: &Value) -> Option<FlexDirection> {
    match keyword(value)? {
        "row" => Some(FlexDirection::Row),
        "column" => Some(FlexDirection::Column),
        "row-reverse" => Some(FlexDirection::RowReverse),
        "column-reverse" => Some(FlexDirection::ColumnReverse),
        _ => None,
    }
}

fn flex_wrap(value: &Value) -> Option<FlexWrap> {
    match keyword(value)? {
        "nowrap" => Some(FlexWrap::NoWrap),
        "wrap" => Some(FlexWrap::Wrap),
        "wrap-reverse" => Some(FlexWrap::WrapReverse),
        _ => None,
    }
}

fn align_items(value: &Value) -> Option<AlignItems> {
    match keyword(value)? {
        "start" => Some(AlignItems::Start),
        "end" => Some(AlignItems::End),
        "flex-start" => Some(AlignItems::FlexStart),
        "flex-end" => Some(AlignItems::FlexEnd),
        "center" => Some(AlignItems::Center),
        "baseline" => Some(AlignItems::Baseline),
        "stretch" => Some(AlignItems::Stretch),
        _ => None,
    }
}

fn align_content(value: &Value) -> Option<AlignContent> {
    match keyword(value)? {
        "start" => Some(AlignContent::Start),
        "end" => Some(AlignContent::End),
        "flex-start" => Some(AlignContent::FlexStart),
        "flex-end" => Some(AlignContent::FlexEnd),
        "center" => Some(AlignContent::Center),
        "stretch" => Some(AlignContent::Stretch),
        "space-between" => Some(AlignContent::SpaceBetween),
        "space-evenly" => Some(AlignContent::SpaceEvenly),
        "space-around" => Some(AlignContent::SpaceAround),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(property: &str, value: Value) -> Declaration {
        Declaration {
            property: property.to_owned(),
            value,
        }
    }

    fn length_value(value: IrLength) -> Value {
        Value::Length { value }
    }

    fn keyword_value(value: &str) -> Value {
        Value::Keyword {
            value: value.to_owned(),
        }
    }

    fn color_value(rgba: u32) -> Value {
        Value::Color {
            value: Color::Literal { rgba },
        }
    }

    fn record(entries: &[(&str, Value)]) -> Value {
        Value::Record {
            fields: entries
                .iter()
                .cloned()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        }
    }

    #[test]
    fn projects_core_layout_without_creating_a_second_taffy_tree() {
        let mut projection = StyleProjection::default();
        for declaration in [
            declaration("display", keyword_value("flex")),
            declaration("flex-direction", keyword_value("column")),
            declaration("width", length_value(IrLength::Percent { value: 0.75 })),
            declaration("min-height", length_value(IrLength::Px { value: 120.0 })),
            declaration("gap", length_value(IrLength::Px { value: 8.0 })),
            declaration("overflow-y", keyword_value("auto")),
            declaration("align-items", keyword_value("center")),
        ] {
            projection.apply(&declaration).unwrap();
        }

        let style = projection.style();
        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(
            style.size.width,
            Length::Definite(DefiniteLength::Fraction(0.75))
        );
        assert_eq!(
            style.min_size.height,
            Length::Definite(gpui::px(120.0).into())
        );
        assert_eq!(style.gap.width, gpui::px(8.0).into());
        assert_eq!(style.gap.height, gpui::px(8.0).into());
        assert_eq!(style.overflow.y, Overflow::Scroll);
        assert_eq!(style.align_items, Some(AlignItems::Center));
    }

    #[test]
    fn default_projection_preserves_wabou_block_layout_semantics() {
        assert_eq!(StyleProjection::default().style().display, Display::Block);
    }

    #[test]
    fn projects_composite_spacing_and_overflow_axes() {
        let mut projection = StyleProjection::default();

        projection
            .apply(&declaration(
                "gap",
                record(&[
                    ("column", length_value(IrLength::Px { value: 6.0 })),
                    ("row", length_value(IrLength::Px { value: 10.0 })),
                ]),
            ))
            .unwrap();
        projection
            .apply(&declaration(
                "overflow",
                record(&[("x", keyword_value("hidden")), ("y", keyword_value("auto"))]),
            ))
            .unwrap();

        assert_eq!(projection.style().gap.width, gpui::px(6.0).into());
        assert_eq!(projection.style().gap.height, gpui::px(10.0).into());
        assert_eq!(projection.style().overflow.x, Overflow::Hidden);
        assert_eq!(projection.style().overflow.y, Overflow::Scroll);
    }

    #[test]
    fn projects_common_visual_and_text_styles_into_gpui() {
        let mut projection = StyleProjection::default();
        for declaration in [
            declaration("background-color", color_value(0x112233ff)),
            declaration("color", color_value(0xf0e0d0ff)),
            declaration("border-color", color_value(0x445566cc)),
            declaration("border-width", length_value(IrLength::Px { value: 2.0 })),
            declaration("border-radius", length_value(IrLength::Px { value: 12.0 })),
            declaration("font-size", length_value(IrLength::Px { value: 18.0 })),
            declaration("font-weight", Value::Number { value: 600.0 }),
            declaration("line-height", Value::Number { value: 1.5 }),
        ] {
            projection.apply(&declaration).unwrap();
        }

        let style = projection.style();
        assert_eq!(
            style.background.as_ref().and_then(gpui::Fill::color),
            Some(color(&color_value(0x112233ff)).unwrap().into())
        );
        assert_eq!(
            style.text.color,
            Some(color(&color_value(0xf0e0d0ff)).unwrap())
        );
        assert_eq!(
            style.border_color,
            Some(color(&color_value(0x445566cc)).unwrap())
        );
        assert_eq!(style.border_widths.top, gpui::px(2.0).into());
        assert_eq!(style.border_widths.right, gpui::px(2.0).into());
        assert_eq!(style.corner_radii.top_left, gpui::px(12.0).into());
        assert_eq!(style.corner_radii.bottom_right, gpui::px(12.0).into());
        assert_eq!(style.text.font_size, Some(gpui::px(18.0).into()));
        assert_eq!(style.text.font_weight, Some(FontWeight(600.0)));
        assert_eq!(style.text.line_height, Some(DefiniteLength::Fraction(1.5)));
    }

    #[test]
    fn accepts_authored_numeric_borders_and_keyword_auto_flex_basis() {
        let mut projection = StyleProjection::default();
        projection
            .apply(&declaration("border-width", Value::Number { value: 1.0 }))
            .unwrap();
        projection
            .apply(&declaration("flex-basis", keyword_value("auto")))
            .unwrap();

        assert_eq!(projection.style().border_widths.top, gpui::px(1.0).into());
        assert_eq!(projection.style().border_widths.right, gpui::px(1.0).into());
        assert_eq!(projection.style().flex_basis, Length::Auto);
    }

    #[test]
    fn projects_high_frequency_text_and_interaction_styles() {
        let mut projection = StyleProjection::default();
        for declaration in [
            declaration("aspect-ratio", Value::Number { value: 16.0 / 9.0 }),
            declaration("opacity", Value::Number { value: 0.6 }),
            declaration("font-family", keyword_value("monospace")),
            declaration("font-style", keyword_value("italic")),
            declaration("letter-spacing", length_value(IrLength::Px { value: 0.5 })),
            declaration("white-space", keyword_value("nowrap")),
            declaration("text-overflow", keyword_value("ellipsis")),
            declaration("text-align", keyword_value("center")),
            declaration("cursor", keyword_value("pointer")),
            declaration("visibility", keyword_value("hidden")),
        ] {
            projection.apply(&declaration).unwrap();
        }

        let style = projection.style();
        assert_eq!(style.aspect_ratio, Some(16.0 / 9.0));
        assert_eq!(style.opacity, Some(0.6));
        assert_eq!(
            style.text.font_family.as_deref(),
            Some(".SystemUIFontMonospaced")
        );
        assert_eq!(style.text.font_style, Some(FontStyle::Italic));
        assert_eq!(style.text.letter_spacing, Some(gpui::px(0.5)));
        assert_eq!(style.text.white_space, Some(WhiteSpace::Nowrap));
        assert_eq!(
            style.text.text_overflow,
            Some(TextOverflow::Truncate("…".into()))
        );
        assert_eq!(style.text.text_align, Some(TextAlign::Center));
        assert_eq!(style.mouse_cursor, Some(CursorStyle::PointingHand));
        assert_eq!(style.visibility, Visibility::Hidden);
    }

    #[test]
    fn projects_all_authored_box_shadows_in_order() {
        let mut projection = StyleProjection::default();
        projection
            .apply(&declaration(
                "box-shadow",
                Value::List {
                    values: vec![
                        record(&[
                            ("x", length_value(IrLength::Px { value: 1.0 })),
                            ("y", length_value(IrLength::Px { value: 2.0 })),
                            ("stdDev", length_value(IrLength::Px { value: 8.0 })),
                            ("spread", length_value(IrLength::Px { value: 0.0 })),
                            ("color", color_value(0x00000080)),
                        ]),
                        record(&[
                            ("x", length_value(IrLength::Px { value: 0.0 })),
                            ("y", length_value(IrLength::Px { value: 1.0 })),
                            ("stdDev", length_value(IrLength::Px { value: 2.0 })),
                            ("spread", length_value(IrLength::Px { value: 1.0 })),
                            ("color", color_value(0x11223344)),
                        ]),
                    ],
                },
            ))
            .unwrap();

        let shadows = &projection.style().box_shadow;
        assert_eq!(shadows.len(), 2);
        assert_eq!(shadows[0].offset, gpui::point(gpui::px(1.0), gpui::px(2.0)));
        assert_eq!(shadows[0].blur_radius, gpui::px(8.0));
        assert_eq!(shadows[1].spread_radius, gpui::px(1.0));
    }

    #[test]
    fn unsupported_and_invalid_values_are_never_silently_ignored() {
        let mut projection = StyleProjection::default();
        assert_eq!(
            projection.apply(&declaration("backdrop-filter", keyword_value("blur"))),
            Err(StyleDiagnostic::UnsupportedProperty(
                "backdrop-filter".to_owned()
            ))
        );
        assert_eq!(
            projection.apply(&declaration("display", keyword_value("table"))),
            Err(StyleDiagnostic::InvalidValue {
                property: "display".to_owned()
            })
        );
    }

    #[test]
    fn auto_is_rejected_where_gpui_requires_a_definite_length() {
        let mut projection = StyleProjection::default();
        assert_eq!(
            projection.apply(&declaration("padding", length_value(IrLength::Auto))),
            Err(StyleDiagnostic::InvalidValue {
                property: "padding".to_owned()
            })
        );
    }

    #[test]
    fn bounded_visual_values_are_rejected_instead_of_clamped_silently() {
        let mut projection = StyleProjection::default();
        for (property, value) in [
            ("opacity", Value::Number { value: 1.1 }),
            ("aspect-ratio", Value::Number { value: 0.0 }),
        ] {
            assert_eq!(
                projection.apply(&declaration(property, value)),
                Err(StyleDiagnostic::InvalidValue {
                    property: property.to_owned(),
                })
            );
        }
    }
}
