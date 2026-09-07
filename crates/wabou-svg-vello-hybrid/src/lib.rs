//! A small, explicit SVG adapter for [`vello_hybrid`].
//!
//! [`usvg`] owns SVG parsing and normalization. This crate only projects the
//! normalized tree into Vello Hybrid commands. Unsupported SVG features are
//! reported as structured diagnostics instead of silently disappearing or
//! reaching Vello Hybrid APIs which currently panic.

mod convert;
mod render;

use snafu::{ResultExt, Snafu};
use vello_hybrid::Scene;

pub use usvg;
pub use vello_common;
pub use vello_hybrid;

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
    let mut report = RenderReport::default();
    render::render_group(
        scene,
        tree.root(),
        vello_common::kurbo::Affine::IDENTITY,
        &mut report,
    )?;
    Ok(report)
}

/// Parse an SVG and create a Hybrid scene sized by the caller.
///
/// Hybrid scene dimensions describe the render target, not the SVG view box,
/// so they remain explicit rather than being guessed from the document.
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

#[cfg(test)]
mod tests;
