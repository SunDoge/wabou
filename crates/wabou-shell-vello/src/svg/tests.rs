use super::*;

fn options() -> usvg::Options<'static> {
    usvg::Options::default()
}

#[test]
fn renders_ui_svg_primitives_without_diagnostics() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
          <defs>
            <linearGradient id="paint" x1="0" y1="0" x2="64" y2="64">
              <stop offset="0" stop-color="#7c3aed"/>
              <stop offset="1" stop-color="#22d3ee"/>
            </linearGradient>
            <clipPath id="clip"><path d="M4 4h56v56H4z"/></clipPath>
          </defs>
          <g opacity="0.8" clip-path="url(#clip)" transform="translate(2 3)">
            <path id="icon" d="M8 32 C8 12 56 12 56 32 S8 52 8 32 Z"
                  fill="url(#paint)" stroke="#111827" stroke-width="2"/>
          </g>
        </svg>
    "##;
    let (_scene, report) = render(svg, &options(), 64, 64).expect("render SVG");
    assert_eq!(report.draw_count, 2);
    assert!(report.diagnostics.is_empty(), "{report:?}");
}

#[test]
fn reports_unsupported_features_without_panicking() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32">
          <defs>
            <pattern id="pattern" width="4" height="4" patternUnits="userSpaceOnUse">
              <path d="M0 0h2v2H0z" fill="red"/>
            </pattern>
            <filter id="blur"><feGaussianBlur stdDeviation="2"/></filter>
          </defs>
          <g id="filtered" filter="url(#blur)">
            <path id="pattern-path" d="M0 0h32v32H0z" fill="url(#pattern)"/>
          </g>
        </svg>
    "##;
    let (_scene, report) = render(svg, &options(), 32, 32).expect("render SVG");
    assert_eq!(report.draw_count, 0);
    assert!(report.diagnostics.iter().any(|item| {
        item.feature == UnsupportedFeature::Filter && item.node_id.as_deref() == Some("filtered")
    }));
    assert!(report.diagnostics.iter().any(|item| {
        item.feature == UnsupportedFeature::PatternPaint
            && item.node_id.as_deref() == Some("pattern-path")
    }));
}

#[test]
fn renders_embedded_png_into_the_scene() {
    let svg = r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="1" height="1">
          <image id="pixel" width="1" height="1"
            href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="/>
        </svg>
    "#;
    let (_scene, report) = render(svg, &options(), 1, 1).expect("render PNG image");
    assert_eq!(report.draw_count, 1);
    assert!(report.diagnostics.is_empty());
}

#[test]
fn rejects_invalid_svg() {
    let error = render("<svg>", &options(), 16, 16).expect_err("invalid SVG must fail");
    assert!(matches!(error, Error::Svg { .. }));
}
