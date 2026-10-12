//! Data-only canvas scene copied synchronously by the host. These owned types
//! are local API values, never passed by value through an extern function.
use crate::LocalPoint;
pub const MAX_CIRCLES: usize = 256;
pub const MAX_SCENE_BYTES: usize = 64 * 1024;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawCircle {
    pub center: LocalPoint,
    pub radius: f32,
    /// RGBA bytes packed big-endian; None omits the corresponding paint.
    pub fill: Option<u32>,
    pub stroke: Option<u32>,
    pub stroke_width: f32,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasScene {
    pub label: String,
    pub circles: Vec<DrawCircle>,
}
impl CanvasScene {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.label.len() > 1024
            || self.label.contains(['\n', '\r'])
            || self.circles.len() > MAX_CIRCLES
        {
            return Err("canvas label or circle capacity exceeded");
        }
        for circle in &self.circles {
            if !circle.center.x.is_finite()
                || !circle.center.y.is_finite()
                || circle.center.x.abs() > 10000.0
                || circle.center.y.abs() > 10000.0
                || !circle.radius.is_finite()
                || !(0.0..=10000.0).contains(&circle.radius)
                || !circle.stroke_width.is_finite()
                || !(0.0..=100.0).contains(&circle.stroke_width)
                || (circle.fill.is_none() && circle.stroke.is_none())
            {
                return Err("invalid canvas circle");
            }
        }
        Ok(())
    }
    pub fn to_wire(&self) -> String {
        use std::fmt::Write;
        let mut wire = format!("canvas-v1\n{}\n", self.label);
        for circle in &self.circles {
            let color =
                |color: Option<u32>| color.map_or_else(|| "none".into(), |color| color.to_string());
            writeln!(
                &mut wire,
                "{} {} {} {} {} {}",
                circle.center.x,
                circle.center.y,
                circle.radius,
                color(circle.fill),
                color(circle.stroke),
                circle.stroke_width
            )
            .expect("String formatting");
        }
        wire
    }
    pub fn from_wire(wire: &str) -> Result<Self, &'static str> {
        if wire.len() > MAX_SCENE_BYTES {
            return Err("canvas scene too large");
        }
        let mut lines = wire.split('\n');
        if lines.next() != Some("canvas-v1") {
            return Err("invalid canvas scene version");
        }
        let label = lines.next().ok_or("missing canvas label")?.to_owned();
        let mut circles = Vec::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            if circles.len() == MAX_CIRCLES {
                return Err("canvas circle capacity exceeded");
            }
            let mut fields = line.split_whitespace();
            let mut next = || fields.next().ok_or("missing circle field");
            let x = next()?.parse().map_err(|_| "invalid circle x")?;
            let y = next()?.parse().map_err(|_| "invalid circle y")?;
            let radius = next()?.parse().map_err(|_| "invalid circle radius")?;
            let color = |text: &str| {
                if text == "none" {
                    Ok(None)
                } else {
                    text.parse::<u32>()
                        .map(Some)
                        .map_err(|_| "invalid circle color")
                }
            };
            let fill = color(next()?)?;
            let stroke = color(next()?)?;
            let stroke_width = next()?.parse().map_err(|_| "invalid stroke width")?;
            if fields.next().is_some() {
                return Err("extra circle field");
            }
            circles.push(DrawCircle {
                center: LocalPoint { x, y },
                radius,
                fill,
                stroke,
                stroke_width,
            });
        }
        let scene = Self { label, circles };
        scene.validate()?;
        Ok(scene)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn circle() -> DrawCircle {
        DrawCircle {
            center: LocalPoint { x: 12.5, y: 20.25 },
            radius: 26.0,
            fill: Some(0x12345678),
            stroke: Some(0xffffffff),
            stroke_width: 2.0,
        }
    }
    #[test]
    fn scene_round_trip_preserves_unicode_and_paint_data() {
        let scene = CanvasScene {
            label: "圆画布".into(),
            circles: vec![circle()],
        };
        assert_eq!(CanvasScene::from_wire(&scene.to_wire()).unwrap(), scene);
        assert_eq!(
            CanvasScene::from_wire(&CanvasScene::default().to_wire()).unwrap(),
            CanvasScene::default()
        );
    }
    #[test]
    fn scene_rejects_invalid_geometry_and_bounded_wire() {
        for malformed in [
            "bad\nlabel\n",
            "canvas-v1\nlabel\n1 2 3 none none 0\n",
            "canvas-v1\nlabel\n1 2 NaN 0 none 1\n",
            "canvas-v1\nlabel\n1 2 3 0 none -1\n",
            "canvas-v1\nlabel\n1 2 3 0 none 1 extra\n",
            "canvas-v1\nlabel\n1 2\n",
        ] {
            assert!(CanvasScene::from_wire(malformed).is_err());
        }
        let mut scene = CanvasScene {
            label: "Canvas".into(),
            circles: vec![circle(); MAX_CIRCLES],
        };
        assert!(CanvasScene::from_wire(&scene.to_wire()).is_ok());
        scene.circles.push(circle());
        assert!(CanvasScene::from_wire(&scene.to_wire()).is_err());
        assert!(CanvasScene::from_wire(&"x".repeat(MAX_SCENE_BYTES + 1)).is_err());
    }
}
