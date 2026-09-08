//! Parsed, reusable SVG scene fragments.

#![warn(missing_docs)]

use std::fmt;
use std::sync::Arc;

use snafu::{ResultExt, Snafu};
use vello_hybrid::Scene;

use crate::SvgDocument;
pub use usvg;

mod convert;
mod render;

/// A feature which could not be represented faithfully by Vello Hybrid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnsupportedFeature {
    /// SVG pattern paints are not translated yet.
    PatternPaint,
    /// The clip path is nested or contains more than one path.
    ComplexClipPath,
    /// Vello Hybrid mask layers are not implemented and would panic.
    Mask,
    /// SVG filter graphs do not map safely to the current Hybrid filter API.
    Filter,
}

/// A non-fatal loss of SVG fidelity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Unsupported feature encountered by the adapter.
    pub feature: UnsupportedFeature,
    /// SVG element id when the source supplied one.
    pub node_id: Option<String>,
}

/// Summary returned after appending an SVG tree.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderReport {
    /// Number of fill, stroke, and image draw commands emitted.
    pub draw_count: usize,
    /// Non-fatal fidelity losses encountered during conversion.
    pub diagnostics: Vec<Diagnostic>,
}

impl RenderReport {
    fn unsupported(&mut self, feature: UnsupportedFeature, node_id: &str) {
        self.diagnostics.push(Diagnostic {
            feature,
            node_id: (!node_id.is_empty()).then(|| node_id.to_owned()),
        });
    }
}

/// Failure to parse or decode an SVG resource.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum Error {
    /// The SVG document is invalid.
    #[snafu(display("invalid SVG: {source}"))]
    Svg {
        /// Parser error.
        source: usvg::Error,
    },
    /// An embedded raster image could not be decoded.
    #[snafu(display("cannot decode embedded raster image in node {node_id:?}: {source}"))]
    RasterImage {
        /// SVG image element id, when present.
        node_id: Option<String>,
        /// Decoder error.
        source: image::ImageError,
    },
    /// Vello Hybrid stores image dimensions as `u16`.
    #[snafu(display("embedded raster image in node {node_id:?} is too large: {width}x{height}"))]
    RasterImageTooLarge {
        /// SVG image element id, when present.
        node_id: Option<String>,
        /// Decoded width.
        width: u32,
        /// Decoded height.
        height: u32,
    },
}

/// Parse `svg` with caller-provided [`usvg::Options`].
pub fn parse(svg: &str, options: &usvg::Options) -> Result<usvg::Tree, Error> {
    usvg::Tree::from_str(svg, options).context(SvgSnafu)
}

/// Parse and append an SVG document to an existing Hybrid scene.
pub fn append(
    scene: &mut Scene,
    svg: &str,
    options: &usvg::Options,
) -> Result<RenderReport, Error> {
    let tree = parse(svg, options)?;
    append_tree(scene, &tree)
}

/// Append a normalized SVG tree to an existing Hybrid scene.
pub fn append_tree(scene: &mut Scene, tree: &usvg::Tree) -> Result<RenderReport, Error> {
    append_tree_with_transform(scene, tree, vello_common::kurbo::Affine::IDENTITY)
}

/// Append a normalized SVG tree under a caller-provided root transform.
pub fn append_tree_with_transform(
    scene: &mut Scene,
    tree: &usvg::Tree,
    transform: vello_common::kurbo::Affine,
) -> Result<RenderReport, Error> {
    let mut report = RenderReport::default();
    render::render_group(scene, tree.root(), transform, &mut report)?;
    Ok(report)
}

/// Parse an SVG and create a Hybrid scene sized by the caller.
pub fn render(
    svg: &str,
    options: &usvg::Options,
    width: u16,
    height: u16,
) -> Result<(Scene, RenderReport), Error> {
    let mut scene = Scene::new(width, height);
    let report = append(&mut scene, svg, options)?;
    Ok((scene, report))
}

/// A static SVG that has already been normalized by `usvg` and encoded into a
/// backend-neutral scene. Keeping this in the retained paint state avoids XML parsing on
/// every frame.
#[derive(Clone)]
pub struct SvgImage {
    document: SvgDocument,
    size: [f32; 2],
}

impl SvgImage {
    /// Parse SVG XML and retain its normalized tree for direct Hybrid projection.
    pub fn parse(source: &str) -> Result<Self, usvg::Error> {
        let tree = usvg::Tree::from_str(source, &usvg::Options::default())?;
        let size = tree.size();
        Ok(Self {
            document: SvgDocument(Arc::new(tree)),
            size: [size.width(), size.height()],
        })
    }

    /// Borrow the retained content-local backend-neutral scene.
    pub fn document(&self) -> &SvgDocument {
        &self.document
    }

    /// Intrinsic SVG `[width, height]` in logical pixels.
    pub fn size(&self) -> [f32; 2] {
        self.size
    }
}

impl fmt::Debug for SvgImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SvgImage")
            .field("size", &self.size)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lucide_style_svg() {
        let image = SvgImage::parse(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#2563eb" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2 2 0 0 1 2 2v15"/></svg>"##,
        )
        .unwrap();
        assert_eq!(image.size(), [24.0, 24.0]);
    }
}

#[cfg(test)]
#[path = "svg/tests.rs"]
mod adapter_tests;
