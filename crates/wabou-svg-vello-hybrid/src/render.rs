// Copyright 2024 the Vello Authors and 2026 the Wabou contributors
// SPDX-License-Identifier: Apache-2.0

use vello_common::kurbo::{Affine, BezPath, PathEl, Rect};
use vello_common::peniko::{BlendMode, Compose, Fill, Mix};
use vello_hybrid::Scene;

use crate::{Error, RenderReport, UnsupportedFeature, convert};

pub(crate) fn render_group(
    scene: &mut Scene,
    group: &usvg::Group,
    base_transform: Affine,
    report: &mut RenderReport,
) -> Result<(), Error> {
    if group.mask().is_some() {
        report.unsupported(UnsupportedFeature::Mask, group.id());
    }
    if !group.filters().is_empty() {
        report.unsupported(UnsupportedFeature::Filter, group.id());
    }

    let group_transform = base_transform * convert::affine(&group.abs_transform());
    let clip = group
        .clip_path()
        .and_then(|clip| simple_clip_path(clip, group_transform));
    if group.clip_path().is_some() && clip.is_none() {
        report.unsupported(UnsupportedFeature::ComplexClipPath, group.id());
    }

    let opacity = group.opacity().get();
    let blend = blend_mode(group.blend_mode());
    let needs_layer = clip.is_some() || opacity != 1.0 || blend.mix != Mix::Normal;
    if needs_layer {
        scene.set_transform(Affine::IDENTITY);
        scene.push_layer(
            clip.as_ref(),
            (blend.mix != Mix::Normal).then_some(blend),
            (opacity != 1.0).then_some(opacity),
            None,
            None,
        );
    }

    for node in group.children() {
        match node {
            usvg::Node::Group(group) => render_group(scene, group, base_transform, report)?,
            usvg::Node::Path(path) => render_path(scene, path, base_transform, report),
            usvg::Node::Image(image) => render_image(scene, image, base_transform, report)?,
            usvg::Node::Text(text) => {
                render_group(scene, text.flattened(), base_transform, report)?;
            }
        }
    }

    if needs_layer {
        scene.pop_layer();
    }
    Ok(())
}

fn render_path(
    scene: &mut Scene,
    path: &usvg::Path,
    base_transform: Affine,
    report: &mut RenderReport,
) {
    if !path.is_visible() {
        return;
    }
    let geometry = convert::path(path);
    let transform = base_transform * convert::affine(&path.abs_transform());

    match path.paint_order() {
        usvg::PaintOrder::FillAndStroke => {
            render_fill(scene, path, &geometry, transform, report);
            render_stroke(scene, path, &geometry, transform, report);
        }
        usvg::PaintOrder::StrokeAndFill => {
            render_stroke(scene, path, &geometry, transform, report);
            render_fill(scene, path, &geometry, transform, report);
        }
    }
}

fn render_fill(
    scene: &mut Scene,
    path: &usvg::Path,
    geometry: &BezPath,
    transform: Affine,
    report: &mut RenderReport,
) {
    let Some(fill) = path.fill() else { return };
    let Some((paint, paint_transform)) = convert::paint(fill.paint(), fill.opacity()) else {
        report.unsupported(UnsupportedFeature::PatternPaint, path.id());
        return;
    };
    scene.set_transform(transform);
    scene.set_paint(paint);
    scene.set_paint_transform(paint_transform);
    scene.set_fill_rule(match fill.rule() {
        usvg::FillRule::NonZero => Fill::NonZero,
        usvg::FillRule::EvenOdd => Fill::EvenOdd,
    });
    scene.fill_path(geometry);
    report.draw_count += 1;
}

fn render_stroke(
    scene: &mut Scene,
    path: &usvg::Path,
    geometry: &BezPath,
    transform: Affine,
    report: &mut RenderReport,
) {
    let Some(stroke) = path.stroke() else {
        return;
    };
    let Some((paint, paint_transform)) = convert::paint(stroke.paint(), stroke.opacity()) else {
        report.unsupported(UnsupportedFeature::PatternPaint, path.id());
        return;
    };
    scene.set_transform(transform);
    scene.set_paint(paint);
    scene.set_paint_transform(paint_transform);
    scene.set_stroke(convert::stroke(stroke));
    scene.stroke_path(geometry);
    report.draw_count += 1;
}

fn render_image(
    scene: &mut Scene,
    image: &usvg::Image,
    base_transform: Affine,
    report: &mut RenderReport,
) -> Result<(), Error> {
    if !image.is_visible() {
        return Ok(());
    }
    match image.kind() {
        usvg::ImageKind::SVG(tree) => {
            let transform = base_transform * convert::affine(&image.abs_transform());
            render_group(scene, tree.root(), transform, report)?;
        }
        _ => {
            let paint = convert::raster_image(image)?;
            scene.set_transform(base_transform * convert::affine(&image.abs_transform()));
            scene.set_paint(paint);
            scene.reset_paint_transform();
            scene.set_fill_rule(Fill::NonZero);
            let size = image.size();
            scene.fill_rect(&Rect::new(
                0.0,
                0.0,
                f64::from(size.width()),
                f64::from(size.height()),
            ));
            report.draw_count += 1;
        }
    }
    Ok(())
}

fn simple_clip_path(clip: &usvg::ClipPath, reference_transform: Affine) -> Option<BezPath> {
    if clip.clip_path().is_some() {
        return None;
    }
    let [usvg::Node::Path(path)] = clip.root().children() else {
        return None;
    };
    let transform = reference_transform
        * convert::affine(&clip.transform())
        * convert::affine(&path.abs_transform());
    Some(transform_path(&convert::path(path), transform))
}

fn transform_path(path: &BezPath, transform: Affine) -> BezPath {
    path.elements()
        .iter()
        .map(|element| match *element {
            PathEl::MoveTo(point) => PathEl::MoveTo(transform * point),
            PathEl::LineTo(point) => PathEl::LineTo(transform * point),
            PathEl::QuadTo(first, second) => PathEl::QuadTo(transform * first, transform * second),
            PathEl::CurveTo(first, second, third) => {
                PathEl::CurveTo(transform * first, transform * second, transform * third)
            }
            PathEl::ClosePath => PathEl::ClosePath,
        })
        .collect()
}

fn blend_mode(mode: usvg::BlendMode) -> BlendMode {
    let mix = match mode {
        usvg::BlendMode::Normal => Mix::Normal,
        usvg::BlendMode::Multiply => Mix::Multiply,
        usvg::BlendMode::Screen => Mix::Screen,
        usvg::BlendMode::Overlay => Mix::Overlay,
        usvg::BlendMode::Darken => Mix::Darken,
        usvg::BlendMode::Lighten => Mix::Lighten,
        usvg::BlendMode::ColorDodge => Mix::ColorDodge,
        usvg::BlendMode::ColorBurn => Mix::ColorBurn,
        usvg::BlendMode::HardLight => Mix::HardLight,
        usvg::BlendMode::SoftLight => Mix::SoftLight,
        usvg::BlendMode::Difference => Mix::Difference,
        usvg::BlendMode::Exclusion => Mix::Exclusion,
        usvg::BlendMode::Hue => Mix::Hue,
        usvg::BlendMode::Saturation => Mix::Saturation,
        usvg::BlendMode::Color => Mix::Color,
        usvg::BlendMode::Luminosity => Mix::Luminosity,
    };
    BlendMode::new(mix, Compose::SrcOver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vello_common::kurbo::Shape;

    #[test]
    fn clip_geometry_includes_the_referencing_group_transform() {
        let tree = usvg::Tree::from_str(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40">
                <defs><clipPath id="clip"><path d="M1 2h3v4H1z"/></clipPath></defs>
                <g transform="translate(10 20)" clip-path="url(#clip)">
                    <path d="M0 0h40v40H0z"/>
                </g>
            </svg>"#,
            &usvg::Options::default(),
        )
        .expect("parse fixture");
        let usvg::Node::Group(group) = &tree.root().children()[0] else {
            panic!("fixture group was flattened unexpectedly");
        };
        let clip = simple_clip_path(
            group.clip_path().expect("clip path"),
            convert::affine(&group.abs_transform()),
        )
        .expect("simple clip");
        let bounds = clip.bounding_box();
        assert_eq!(bounds.x0, 11.0);
        assert_eq!(bounds.y0, 22.0);
        assert_eq!(bounds.x1, 14.0);
        assert_eq!(bounds.y1, 26.0);
    }
}
