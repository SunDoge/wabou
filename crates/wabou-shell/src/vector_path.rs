use gpui::{
    FillOptions, FillRule, PathBuilder, PathStyle, Pixels, StrokeOptions, Window, point, px,
};
use lyon_tessellation::{LineCap, LineJoin};

const MAGIC: u32 = 0x3150_4257;
const VERSION: u16 = 1;
const HEADER_SIZE: usize = 36;

#[derive(Clone, Debug)]
enum Command {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectedVectorPath {
    commands: Vec<Command>,
    fill: u32,
    stroke: u32,
    stroke_width: f32,
    fill_rule: FillRule,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f32,
}

impl ProjectedVectorPath {
    pub(crate) fn decode(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < HEADER_SIZE {
            return Err("vector path header is truncated");
        }
        if u32_at(data, 0)? != MAGIC {
            return Err("vector path magic is invalid");
        }
        if u16_at(data, 4)? != VERSION {
            return Err("vector path version is unsupported");
        }
        let command_count = u32_at(data, 8)? as usize;
        if u32_at(data, 12)? as usize != data.len() {
            return Err("vector path byte length does not match its header");
        }
        let fill = u32_at(data, 16)?;
        let stroke = u32_at(data, 20)?;
        let stroke_width = finite_positive(f32_at(data, 24)?, "vector path stroke width")?;
        let fill_rule = match data[28] {
            0 => FillRule::NonZero,
            1 => FillRule::EvenOdd,
            _ => return Err("vector path fill rule is invalid"),
        };
        let line_cap = match data[29] {
            0 => LineCap::Butt,
            1 => LineCap::Round,
            2 => LineCap::Square,
            _ => return Err("vector path line cap is invalid"),
        };
        let line_join = match data[30] {
            0 => LineJoin::Miter,
            1 => LineJoin::Round,
            2 => LineJoin::Bevel,
            _ => return Err("vector path line join is invalid"),
        };
        let miter_limit = finite_positive(f32_at(data, 32)?, "vector path miter limit")?;
        let mut offset = HEADER_SIZE;
        let mut commands = Vec::with_capacity(command_count);
        for _ in 0..command_count {
            let kind = *data.get(offset).ok_or("vector path command is truncated")?;
            offset = offset.checked_add(4).ok_or("vector path offset overflow")?;
            commands.push(match kind {
                1 => Command::MoveTo(read_f32(data, &mut offset)?, read_f32(data, &mut offset)?),
                2 => Command::LineTo(read_f32(data, &mut offset)?, read_f32(data, &mut offset)?),
                3 => Command::QuadTo(
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                ),
                4 => Command::CubicTo(
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                    read_f32(data, &mut offset)?,
                ),
                5 => Command::Close,
                _ => return Err("vector path command is invalid"),
            });
        }
        if offset != data.len() {
            return Err("vector path contains trailing bytes");
        }
        Ok(Self {
            commands,
            fill,
            stroke,
            stroke_width,
            fill_rule,
            line_cap,
            line_join,
            miter_limit,
        })
    }

    pub(crate) fn paint(&self, origin: gpui::Point<Pixels>, window: &mut Window) {
        if self.fill & 0xff != 0 {
            let options = FillOptions::default().with_fill_rule(self.fill_rule);
            if let Ok(path) = self.build(
                PathBuilder::fill().with_style(PathStyle::Fill(options)),
                origin,
            ) {
                window.paint_path(path, gpui::rgba(self.fill));
            }
        }
        if self.stroke & 0xff != 0 {
            let options = StrokeOptions::default()
                .with_line_width(self.stroke_width)
                .with_line_cap(self.line_cap)
                .with_line_join(self.line_join)
                .with_miter_limit(self.miter_limit);
            if let Ok(path) = self.build(
                PathBuilder::stroke(px(self.stroke_width)).with_style(PathStyle::Stroke(options)),
                origin,
            ) {
                window.paint_path(path, gpui::rgba(self.stroke));
            }
        }
    }

    fn build(
        &self,
        mut builder: PathBuilder,
        origin: gpui::Point<Pixels>,
    ) -> anyhow::Result<gpui::Path<Pixels>> {
        for command in &self.commands {
            match *command {
                Command::MoveTo(x, y) => builder.move_to(point(origin.x + px(x), origin.y + px(y))),
                Command::LineTo(x, y) => builder.line_to(point(origin.x + px(x), origin.y + px(y))),
                Command::QuadTo(cx, cy, x, y) => builder.curve_to(
                    point(origin.x + px(x), origin.y + px(y)),
                    point(origin.x + px(cx), origin.y + px(cy)),
                ),
                Command::CubicTo(c1x, c1y, c2x, c2y, x, y) => builder.cubic_bezier_to(
                    point(origin.x + px(x), origin.y + px(y)),
                    point(origin.x + px(c1x), origin.y + px(c1y)),
                    point(origin.x + px(c2x), origin.y + px(c2y)),
                ),
                Command::Close => builder.close(),
            }
        }
        builder.build()
    }
}

fn u16_at(data: &[u8], offset: usize) -> Result<u16, &'static str> {
    Ok(u16::from_le_bytes(
        data.get(offset..offset + 2)
            .ok_or("vector path field is truncated")?
            .try_into()
            .expect("slice length was checked"),
    ))
}

fn u32_at(data: &[u8], offset: usize) -> Result<u32, &'static str> {
    Ok(u32::from_le_bytes(
        data.get(offset..offset + 4)
            .ok_or("vector path field is truncated")?
            .try_into()
            .expect("slice length was checked"),
    ))
}

fn f32_at(data: &[u8], offset: usize) -> Result<f32, &'static str> {
    let value = f32::from_bits(u32_at(data, offset)?);
    value
        .is_finite()
        .then_some(value)
        .ok_or("vector path coordinate is not finite")
}

fn read_f32(data: &[u8], offset: &mut usize) -> Result<f32, &'static str> {
    let value = f32_at(data, *offset)?;
    *offset = offset.checked_add(4).ok_or("vector path offset overflow")?;
    Ok(value)
}

fn finite_positive(value: f32, error: &'static str) -> Result<f32, &'static str> {
    (value.is_finite() && value > 0.0)
        .then_some(value)
        .ok_or(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_path() -> Vec<u8> {
        let mut bytes = vec![0_u8; 60];
        bytes[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        bytes[4..6].copy_from_slice(&VERSION.to_le_bytes());
        bytes[8..12].copy_from_slice(&2_u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&60_u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&0xff00_00ff_u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&1_f32.to_le_bytes());
        bytes[32..36].copy_from_slice(&4_f32.to_le_bytes());
        bytes[36] = 1;
        bytes[40..44].copy_from_slice(&1_f32.to_le_bytes());
        bytes[44..48].copy_from_slice(&2_f32.to_le_bytes());
        bytes[48] = 2;
        bytes[52..56].copy_from_slice(&9_f32.to_le_bytes());
        bytes[56..60].copy_from_slice(&10_f32.to_le_bytes());
        bytes
    }

    #[test]
    fn stable_commands_build_gpui_geometry_at_the_element_origin() {
        let path = ProjectedVectorPath::decode(&line_path()).unwrap();
        let geometry = path
            .build(PathBuilder::stroke(px(1.0)), point(px(20.0), px(30.0)))
            .unwrap();

        assert!(geometry.bounds.origin.x >= px(20.0));
        assert!(geometry.bounds.origin.y >= px(30.0));
        assert!(!geometry.vertices.is_empty());
    }

    #[test]
    fn malformed_or_non_finite_commands_never_reach_gpui() {
        let mut truncated = line_path();
        truncated.pop();
        assert_eq!(
            ProjectedVectorPath::decode(&truncated).unwrap_err(),
            "vector path byte length does not match its header"
        );

        let mut non_finite = line_path();
        non_finite[40..44].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(
            ProjectedVectorPath::decode(&non_finite).unwrap_err(),
            "vector path coordinate is not finite"
        );
    }
}
