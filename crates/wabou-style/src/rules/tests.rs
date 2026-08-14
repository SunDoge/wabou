use super::*;
use crate::manifest;

#[test]
fn parses_dynamic_spacing() {
    let parsed = parse_utility("px-[13px]").unwrap();
    assert_eq!(parsed.declarations.len(), 2);
    assert_eq!(
        parsed.declarations[0].value,
        Value::Length {
            value: Length::Px { value: 13.0 }
        }
    );
}

#[test]
fn parses_fractional_dimensions_without_extending_spacing_or_length_rules() {
    for (candidate, property, value) in [
        ("w-2/3", "width", 2.0 / 3.0),
        ("h-4/5", "height", 4.0 / 5.0),
        ("max-w-3/4", "max-width", 3.0 / 4.0),
        ("-left-1/2", "left", -0.5),
    ] {
        let declaration = &parse_utility(candidate).unwrap().declarations[0];
        assert_eq!(declaration.property, property);
        assert_eq!(
            declaration.value,
            Value::Length {
                value: Length::Percent { value }
            }
        );
    }
    for candidate in ["w-1/0", "w-1.5/3", "p-1/2", "rounded-1/2"] {
        assert!(
            parse_utility(candidate).is_err(),
            "{candidate} must be rejected"
        );
    }
}

#[test]
fn named_container_widths_are_scoped_to_max_width() {
    for (candidate, value) in [
        ("max-w-xs", 320.0),
        ("max-w-sm", 384.0),
        ("max-w-md", 448.0),
        ("max-w-lg", 512.0),
        ("max-w-xl", 576.0),
        ("max-w-2xl", 672.0),
        ("max-w-3xl", 768.0),
        ("max-w-4xl", 896.0),
        ("max-w-5xl", 1024.0),
        ("max-w-6xl", 1152.0),
        ("max-w-7xl", 1280.0),
    ] {
        assert_eq!(
            parse_utility(candidate).unwrap().declarations[0].value,
            Value::Length {
                value: Length::Px { value }
            }
        );
    }
    for candidate in ["w-md", "h-md", "p-md"] {
        assert!(
            parse_utility(candidate).is_err(),
            "{candidate} must be rejected"
        );
    }
}

#[test]
fn spacing_scale_includes_eighty_eight_pixels() {
    assert_eq!(
        parse_utility("min-h-22").unwrap().declarations[0].value,
        Value::Length {
            value: Length::Px { value: 88.0 }
        }
    );
}

#[test]
fn truncate_emits_clipping_nowrap_and_ellipsis() {
    let parsed = parse_utility("truncate").unwrap();
    assert_eq!(
        parsed
            .declarations
            .iter()
            .map(|declaration| declaration.property.as_str())
            .collect::<Vec<_>>(),
        ["overflow", "white-space", "text-overflow"]
    );
}

#[test]
fn text_ellipsis_does_not_require_overflow_clipping() {
    let parsed = parse_utility("text-ellipsis").unwrap();
    assert_eq!(parsed.declarations.len(), 1);
    assert_eq!(parsed.declarations[0].property, "text-overflow");
    assert_eq!(
        parsed.declarations[0].value,
        Value::Keyword {
            value: "ellipsis".into()
        }
    );
}

#[test]
fn static_transform_utilities_emit_typed_nested_ir() {
    let parsed = parse_utility("translate-x-4").unwrap();
    assert!(matches!(
        &parsed.declarations[0].value,
        Value::List { values }
            if matches!(&values[0], Value::Record { fields }
                if fields.get("kind") == Some(&Value::Keyword { value: "translateX".into() }))
    ));
}

#[test]
fn vello_shadow_scale_uses_ordered_ambient_and_key_layers() {
    for candidate in ["shadow", "shadow-md", "shadow-lg", "shadow-xl"] {
        let parsed = parse_utility(candidate).unwrap();
        assert!(matches!(
            &parsed.declarations[0].value,
            Value::List { values } if values.len() == 2
        ));
    }

    let parsed = parse_utility("shadow-xl").unwrap();
    let Value::List { values } = &parsed.declarations[0].value else {
        panic!("shadow-xl must emit an ordered layer list");
    };
    let Value::Record { fields: ambient } = &values[0] else {
        panic!("ambient layer must be a record");
    };
    let Value::Record { fields: key } = &values[1] else {
        panic!("key layer must be a record");
    };
    assert_eq!(
        ambient.get("stdDev"),
        Some(&Value::Length {
            value: Length::Px { value: 16.0 }
        })
    );
    assert_eq!(
        key.get("spread"),
        Some(&Value::Length {
            value: Length::Px { value: -6.0 }
        })
    );
}

#[test]
fn rejects_css_expressions() {
    assert!(matches!(
        parse_utility("w-[calc(100%-2rem)]"),
        Err(ParseError::InvalidValue { .. })
    ));
    assert!(matches!(
        parse_utility("p-[var(--space)]"),
        Err(ParseError::InvalidValue { .. })
    ));
}

#[test]
fn winnow_parses_typed_arbitrary_lengths() {
    assert_eq!(
        parse_length("[-1.5rem]", false, default_theme()),
        Some(Length::Px { value: -24.0 })
    );
    assert_eq!(
        parse_length("[12.5%]", false, default_theme()),
        Some(Length::Percent { value: 0.125 })
    );
    assert_eq!(parse_length("[12px]junk", false, default_theme()), None);
}

#[test]
fn rejects_variants_and_invalid_arbitrary_values() {
    for candidate in [
        "hover:bg-slate-900",
        "focus:w-4",
        "active:scale-150",
        "disabled:opacity-50",
        "sm:flex",
        "dark:bg-black",
    ] {
        assert!(matches!(
            parse_utility(candidate),
            Err(ParseError::InvalidCandidate { .. })
        ));
    }
    assert!(matches!(
        parse_utility("hover:w-[12px"),
        Err(ParseError::InvalidCandidate { .. })
    ));
    assert!(matches!(
        parse_utility("transition"),
        Err(ParseError::UnknownUtility(_))
    ));
    assert!(matches!(
        parse_utility("animate-spin"),
        Err(ParseError::UnknownUtility(_))
    ));
}

#[test]
fn manifest_is_generated_from_runtime_definitions() {
    let manifest = manifest();
    assert_eq!(manifest.spacing["3"], 12.0);
    assert_eq!(manifest.colors["slate-900"], 0x0f172aff);
    assert!(manifest.static_utilities.contains_key("flex-1"));
}

#[test]
fn custom_theme_extends_typed_spacing_and_colors() {
    let mut theme = Theme::default();
    theme.spacing.insert("18.5".into(), 74.0);
    theme.colors.insert("brand".into(), 0x336699ff);
    assert_eq!(
        parse_utility_with_theme("mt-18.5", &theme)
            .unwrap()
            .declarations[0]
            .value,
        Value::Length {
            value: Length::Px { value: 74.0 }
        }
    );
    assert_eq!(
        parse_utility_with_theme("bg-brand", &theme)
            .unwrap()
            .declarations[0]
            .value,
        Value::Color {
            value: Color::Literal { rgba: 0x336699ff }
        }
    );
}

#[test]
fn supports_typed_stateless_arbitrary_values_and_restricted_negatives() {
    assert!(parse_utility("-mt-4").is_ok());
    assert!(parse_utility("-inset-[5%]").is_ok());
    assert!(parse_utility("bg-[#336699cc]").is_ok());
    assert!(parse_utility("aspect-[16/9]").is_ok());
    assert!(parse_utility("z-[-2]").is_ok());
    assert!(matches!(
        parse_utility("-p-4"),
        Err(ParseError::InvalidValue { .. })
    ));
    assert!(matches!(
        parse_utility("p-auto"),
        Err(ParseError::InvalidValue { .. })
    ));
    assert!(matches!(
        parse_utility("-w-4"),
        Err(ParseError::InvalidValue { .. })
    ));
    assert!(matches!(
        parse_utility("aspect-[1/0]"),
        Err(ParseError::InvalidValue { .. })
    ));
}
