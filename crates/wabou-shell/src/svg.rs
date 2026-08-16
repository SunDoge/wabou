//! Parsed, reusable SVG scene fragments.

#![warn(missing_docs)]

use std::fmt;
use std::sync::Arc;

use vello::Scene;
use vello_svg::usvg;

/// A static SVG that has already been normalized by `usvg` and encoded into a
/// Vello scene. Keeping this in the retained paint state avoids XML parsing on
/// every frame.
#[derive(Clone)]
pub struct SvgImage {
    scene: Arc<Scene>,
    size: [f32; 2],
}

impl SvgImage {
    /// Parse SVG XML and encode its normalized usvg tree into a Vello scene.
    pub fn parse(source: &str) -> Result<Self, usvg::Error> {
        let tree = usvg::Tree::from_str(source, &usvg::Options::default())?;
        let size = tree.size();
        Ok(Self {
            scene: Arc::new(vello_svg::render_tree(&tree)),
            size: [size.width(), size.height()],
        })
    }

    /// Borrow the retained content-local Vello scene.
    pub fn scene(&self) -> &Scene {
        &self.scene
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
