// Copyright 2023 the Vello Authors and 2026 the Wabou contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use vello_common::kurbo::{Affine, BezPath, Cap, Join, Point, Stroke};
use vello_common::paint::{Image, ImageSource, PaintType};
use vello_common::peniko::color::DynamicColor;
use vello_common::peniko::{
    Blob, Color, ColorStop, Extend, Gradient, ImageAlphaType, ImageBrush, ImageData, ImageFormat,
    ImageQuality, ImageSampler,
};

use super::{Error, RasterImageSnafu};
use snafu::ResultExt;

pub(crate) fn affine(transform: &usvg::Transform) -> Affine {
    Affine::new(
        [
            transform.sx,
            transform.ky,
            transform.kx,
            transform.sy,
            transform.tx,
            transform.ty,
        ]
        .map(f64::from),
    )
}

pub(crate) fn path(path: &usvg::Path) -> BezPath {
    let mut output = BezPath::new();
    let mut just_closed = false;
    let mut subpath_start = Point::ZERO;

    for segment in path.data().segments() {
        let reopen = |output: &mut BezPath, just_closed: &mut bool| {
            if std::mem::take(just_closed) {
                output.move_to(subpath_start);
            }
        };
        match segment {
            usvg::tiny_skia_path::PathSegment::MoveTo(point) => {
                just_closed = false;
                subpath_start = Point::new(point.x.into(), point.y.into());
                output.move_to(subpath_start);
            }
            usvg::tiny_skia_path::PathSegment::LineTo(point) => {
                reopen(&mut output, &mut just_closed);
                output.line_to((f64::from(point.x), f64::from(point.y)));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(control, point) => {
                reopen(&mut output, &mut just_closed);
                output.quad_to(
                    (f64::from(control.x), f64::from(control.y)),
                    (f64::from(point.x), f64::from(point.y)),
                );
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(first, second, point) => {
                reopen(&mut output, &mut just_closed);
                output.curve_to(
                    (f64::from(first.x), f64::from(first.y)),
                    (f64::from(second.x), f64::from(second.y)),
                    (f64::from(point.x), f64::from(point.y)),
                );
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                just_closed = true;
                output.close_path();
            }
        }
    }
    output
}

pub(crate) fn stroke(stroke: &usvg::Stroke) -> Stroke {
    let mut output = Stroke::new(f64::from(stroke.width().get()))
        .with_caps(match stroke.linecap() {
            usvg::LineCap::Butt => Cap::Butt,
            usvg::LineCap::Round => Cap::Round,
            usvg::LineCap::Square => Cap::Square,
        })
        .with_join(match stroke.linejoin() {
            usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => Join::Miter,
            usvg::LineJoin::Round => Join::Round,
            usvg::LineJoin::Bevel => Join::Bevel,
        })
        .with_miter_limit(f64::from(stroke.miterlimit().get()));
    if let Some(dashes) = stroke.dasharray() {
        output = output.with_dashes(
            f64::from(stroke.dashoffset()),
            dashes.iter().map(|value| f64::from(*value)),
        );
    }
    output
}

pub(crate) fn paint(paint: &usvg::Paint, opacity: usvg::Opacity) -> Option<(PaintType, Affine)> {
    match paint {
        usvg::Paint::Color(color) => Some((
            Color::from_rgba8(color.red, color.green, color.blue, opacity.to_u8()).into(),
            Affine::IDENTITY,
        )),
        usvg::Paint::LinearGradient(gradient) => {
            let stops = color_stops(gradient.stops(), opacity);
            let paint = Gradient::new_linear(
                Point::new(f64::from(gradient.x1()), f64::from(gradient.y1())),
                Point::new(f64::from(gradient.x2()), f64::from(gradient.y2())),
            )
            .with_extend(extend(gradient.spread_method()))
            .with_stops(stops.as_slice());
            Some((paint.into(), affine(&gradient.transform())))
        }
        usvg::Paint::RadialGradient(gradient) => {
            let stops = color_stops(gradient.stops(), opacity);
            let paint = Gradient::new_two_point_radial(
                Point::new(f64::from(gradient.cx()), f64::from(gradient.cy())),
                0.0,
                Point::new(f64::from(gradient.fx()), f64::from(gradient.fy())),
                gradient.r().get(),
            )
            .with_extend(extend(gradient.spread_method()))
            .with_stops(stops.as_slice());
            Some((paint.into(), affine(&gradient.transform())))
        }
        usvg::Paint::Pattern(_) => None,
    }
}

fn color_stops(stops: &[usvg::Stop], opacity: usvg::Opacity) -> Vec<ColorStop> {
    stops
        .iter()
        .map(|stop| ColorStop {
            offset: stop.offset().get(),
            color: DynamicColor::from_alpha_color(Color::from_rgba8(
                stop.color().red,
                stop.color().green,
                stop.color().blue,
                (stop.opacity() * opacity).to_u8(),
            )),
        })
        .collect()
}

fn extend(method: usvg::SpreadMethod) -> Extend {
    match method {
        usvg::SpreadMethod::Pad => Extend::Pad,
        usvg::SpreadMethod::Reflect => Extend::Reflect,
        usvg::SpreadMethod::Repeat => Extend::Repeat,
    }
}

pub(crate) fn raster_image(image: &usvg::Image) -> Result<Image, Error> {
    let (bytes, format) = match image.kind() {
        usvg::ImageKind::JPEG(bytes) => (bytes.as_slice(), image::ImageFormat::Jpeg),
        usvg::ImageKind::PNG(bytes) => (bytes.as_slice(), image::ImageFormat::Png),
        usvg::ImageKind::GIF(bytes) => (bytes.as_slice(), image::ImageFormat::Gif),
        usvg::ImageKind::WEBP(bytes) => (bytes.as_slice(), image::ImageFormat::WebP),
        usvg::ImageKind::SVG(_) => unreachable!("nested SVG is rendered as vector nodes"),
    };
    let decoded = image::load_from_memory_with_format(bytes, format)
        .context(RasterImageSnafu {
            node_id: node_id(image.id()),
        })?
        .into_rgba8();
    let (width, height) = decoded.dimensions();
    if u16::try_from(width).is_err() || u16::try_from(height).is_err() {
        return Err(Error::RasterImageTooLarge {
            node_id: node_id(image.id()),
            width,
            height,
        });
    };

    let image_data = ImageData {
        data: Blob::new(Arc::new(decoded.into_raw())),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    };
    Ok(ImageBrush {
        image: ImageSource::from_peniko_image_data(&image_data),
        sampler: ImageSampler::new().with_quality(match image.rendering_mode() {
            usvg::ImageRendering::OptimizeSpeed
            | usvg::ImageRendering::CrispEdges
            | usvg::ImageRendering::Pixelated => ImageQuality::Low,
            usvg::ImageRendering::Smooth => ImageQuality::Medium,
            usvg::ImageRendering::OptimizeQuality | usvg::ImageRendering::HighQuality => {
                ImageQuality::High
            }
        }),
    })
}

fn node_id(id: &str) -> Option<String> {
    (!id.is_empty()).then(|| id.to_owned())
}
