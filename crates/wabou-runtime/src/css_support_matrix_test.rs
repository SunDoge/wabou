//! Contract test: every property in the shared CSS support matrix must be
//! accepted by the GPUI style projection. The matrix is the TypeScript
//! compiler's allowlist — if Rust returns `false`, the compiler would emit IR
//! the host ignores (layout "looks fine" but wrong).

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use serde::Deserialize;
    use wabou_shell::{StyleDiagnostic, StyleProjection, project_ir};
    use wabou_style::IrValue;

    use crate::style_ir::StyleSheet;

    #[derive(Debug, Deserialize)]
    struct Matrix {
        supported: HashMap<String, serde_json::Value>,
        #[allow(dead_code)]
        unsupported: HashMap<String, String>,
    }

    const MATRIX_JSON: &str =
        include_str!("../../../packages/vite/src/style-compiler/css-support-matrix.json");
    const CONFORMANCE_JSON: &str = include_str!("gen/style-conformance.json");

    // These properties intentionally map to retained GPUI node metadata rather
    // than `gpui::Style`. Everything else in the formal matrix must project
    // directly; there is no legacy migration allowlist anymore.
    const GPUI_NODE_PROPERTIES: &[&str] = &["pointer-events", "z-index"];

    fn keyword(value: &str) -> IrValue {
        IrValue::Keyword {
            value: value.to_string(),
        }
    }

    fn sample_value(property: &str) -> IrValue {
        if property == "transform" || property.starts_with("transform-") {
            return IrValue::List { values: Vec::new() };
        }
        keyword("auto")
    }

    #[test]
    fn every_matrix_host_property_is_known_by_gpui_projection() {
        let matrix: Matrix = serde_json::from_str(MATRIX_JSON).expect("matrix json");
        let node_properties = GPUI_NODE_PROPERTIES
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let mut missing = Vec::new();
        let mut stale_gaps = Vec::new();
        for property in matrix.supported.keys() {
            let mut projection = StyleProjection::default();
            let unsupported = matches!(
                project_ir(&mut projection, property, &sample_value(property)),
                Some(StyleDiagnostic::UnsupportedProperty(_))
            );
            if unsupported && !node_properties.contains(property.as_str()) {
                missing.push(property.clone());
            } else if !unsupported && node_properties.contains(property.as_str()) {
                stale_gaps.push(property.clone());
            }
        }
        assert!(
            missing.is_empty() && stale_gaps.is_empty(),
            "GPUI does not handle matrix properties (compiler would emit \
             dead IR): {missing:?}; stale migration gaps: {stale_gaps:?}\n\
             Implement the GPUI projection or remove the property from the formal matrix."
        );
    }

    #[test]
    fn compiler_emitted_values_are_accepted_by_the_native_style_backend() {
        let stylesheet: StyleSheet =
            serde_json::from_str(CONFORMANCE_JSON).expect("generated compiler Style IR");
        stylesheet.validate().expect("Style IR version");
        assert!(
            stylesheet.rules.len() > 100,
            "fixture must exercise the complete generated utility surface"
        );
        let mut invalid_values = Vec::new();
        for rule in stylesheet.rules {
            for declaration in rule.declarations {
                let mut projection = StyleProjection::default();
                let diagnostic =
                    project_ir(&mut projection, &declaration.property, &declaration.value);
                if GPUI_NODE_PROPERTIES.contains(&declaration.property.as_str()) {
                    assert!(
                        matches!(diagnostic, Some(StyleDiagnostic::UnsupportedProperty(_))),
                        "node-metadata classification is stale: class={} property={} is now a GPUI Style field",
                        rule.class_name,
                        declaration.property,
                    );
                } else {
                    if diagnostic.is_some() {
                        invalid_values.push(format!(
                            "{}:{}:{diagnostic:?}",
                            rule.class_name, declaration.property
                        ));
                    }
                }
            }
        }
        assert!(
            invalid_values.is_empty(),
            "compiler emitted values outside the GPUI contract: {invalid_values:#?}"
        );
    }

    #[test]
    fn unsupported_matrix_names_are_not_silently_accepted() {
        let matrix: Matrix = serde_json::from_str(MATRIX_JSON).expect("matrix json");
        for property in matrix.unsupported.keys() {
            let mut projection = StyleProjection::default();
            assert!(
                matches!(
                    project_ir(&mut projection, property, &keyword("auto")),
                    Some(StyleDiagnostic::UnsupportedProperty(_))
                ),
                "unsupported property {property} should not be accepted by GPUI"
            );
        }
        // Prefix-only names are not full property keys; sample a few.
        for property in [
            "transition-duration",
            "animation-name",
            "filter",
            "backdrop-filter",
        ] {
            let mut projection = StyleProjection::default();
            assert!(
                matches!(
                    project_ir(&mut projection, property, &keyword("auto")),
                    Some(StyleDiagnostic::UnsupportedProperty(_))
                ),
                "non-matrix property {property} must not be a silent GPUI success"
            );
        }
    }
}
