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
    let template = layout.grid_template_areas.as_ref().unwrap();
    assert_eq!((template.row_count, template.column_count), (2, 2));
    let head = template
        .areas
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
