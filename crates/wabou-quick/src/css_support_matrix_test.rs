//! Contract test: every property in the shared CSS support matrix must be
//! accepted by `wabou_shell::style::apply_ir`. The matrix is the TypeScript
//! compiler's allowlist — if Rust returns `false`, the compiler would emit IR
//! the host ignores (layout "looks fine" but wrong).

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Deserialize;
    use wabou_shell::style::{DeclaredPaint, IrValue, apply_ir};

    #[derive(Debug, Deserialize)]
    struct Matrix {
        supported: HashMap<String, serde_json::Value>,
        #[allow(dead_code)]
        unsupported: HashMap<String, String>,
    }

    const MATRIX_JSON: &str =
        include_str!("../../../packages/style-compiler/css-support-matrix.json");

    fn keyword(value: &str) -> IrValue {
        IrValue::Keyword {
            value: value.to_string(),
        }
    }

    fn sample_value(property: &str) -> IrValue {
        if property == "transform" {
            return IrValue::List { values: Vec::new() };
        }
        keyword("auto")
    }

    #[test]
    fn every_matrix_host_property_is_handled_by_apply_ir() {
        let matrix: Matrix = serde_json::from_str(MATRIX_JSON).expect("matrix json");
        let mut missing = Vec::new();
        for property in matrix.supported.keys() {
            let mut layout = taffy::Style::default();
            let mut paint = DeclaredPaint::default();
            // A dummy keyword is enough: known arms return true even when the
            // value shape is ignored; unknown names return false.
            if !apply_ir(&mut layout, &mut paint, property, &sample_value(property)) {
                missing.push(property.clone());
            }
        }
        assert!(
            missing.is_empty(),
            "apply_ir does not handle matrix properties (compiler would emit \
             dead IR): {missing:?}\n\
             Update apply_ir or css-support-matrix.json so they match."
        );
    }

    #[test]
    fn unsupported_matrix_names_are_not_silently_accepted() {
        let matrix: Matrix = serde_json::from_str(MATRIX_JSON).expect("matrix json");
        for property in matrix.unsupported.keys() {
            let mut layout = taffy::Style::default();
            let mut paint = DeclaredPaint::default();
            assert!(
                !apply_ir(&mut layout, &mut paint, property, &keyword("auto")),
                "unsupported property {property} should not be accepted by apply_ir"
            );
        }
        // Prefix-only names are not full property keys; sample a few.
        for property in [
            "transition-duration",
            "animation-name",
            "filter",
            "backdrop-filter",
        ] {
            let mut layout = taffy::Style::default();
            let mut paint = DeclaredPaint::default();
            assert!(
                !apply_ir(&mut layout, &mut paint, property, &keyword("auto")),
                "non-matrix property {property} must not be a silent apply_ir success"
            );
        }
    }
}
