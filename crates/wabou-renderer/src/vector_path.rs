//! Shared decoder for the stable WBP1 vector path wire format.
use std::sync::Arc;
use vello_common::peniko::Color;

/// Validate the complete payload and retain its geometry and paint.
pub fn decode_vector_path(data: &[u8]) -> Option<Arc<crate::style::VectorPath>> {
    use vello_common::kurbo::{BezPath, Cap, Join, Stroke};

    const MAGIC: u32 = 0x3150_4257;
    if data.len() < 36
        || u32::from_le_bytes(data[0..4].try_into().ok()?) != MAGIC
        || u16::from_le_bytes(data[4..6].try_into().ok()?) != 1
        || u16::from_le_bytes(data[6..8].try_into().ok()?) != 0
        || u32::from_le_bytes(data[12..16].try_into().ok()?) as usize != data.len()
    {
        return None;
    }
    let count = u32::from_le_bytes(data[8..12].try_into().ok()?) as usize;
    if count > (data.len() - 36) / 4 {
        return None;
    }
    let fill = u32::from_le_bytes(data[16..20].try_into().ok()?);
    let stroke = u32::from_le_bytes(data[20..24].try_into().ok()?);
    let stroke_width = f32::from_le_bytes(data[24..28].try_into().ok()?);
    let fill_rule = data[28];
    let line_cap = data[29];
    let line_join = data[30];
    let miter_limit = f32::from_le_bytes(data[32..36].try_into().ok()?);
    if data[31] != 0
        || !stroke_width.is_finite()
        || stroke_width <= 0.0
        || !miter_limit.is_finite()
        || miter_limit <= 0.0
        || fill_rule > 1
        || line_cap > 2
        || line_join > 2
    {
        return None;
    }
    let mut offset = 36usize;
    let mut path = BezPath::new();
    let mut has_subpath = false;
    let read = |offset: &mut usize| -> Option<f64> {
        let end = offset.checked_add(4)?;
        let value = f32::from_le_bytes(data.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        value.is_finite().then_some(f64::from(value))
    };
    for _ in 0..count {
        let command = *data.get(offset)?;
        if data.get(offset + 1..offset + 4)? != [0, 0, 0] {
            return None;
        }
        offset += 4;
        match command {
            1 => {
                path.move_to((read(&mut offset)?, read(&mut offset)?));
                has_subpath = true;
            }
            2 if has_subpath => path.line_to((read(&mut offset)?, read(&mut offset)?)),
            3 if has_subpath => path.quad_to(
                (read(&mut offset)?, read(&mut offset)?),
                (read(&mut offset)?, read(&mut offset)?),
            ),
            4 if has_subpath => path.curve_to(
                (read(&mut offset)?, read(&mut offset)?),
                (read(&mut offset)?, read(&mut offset)?),
                (read(&mut offset)?, read(&mut offset)?),
            ),
            5 if has_subpath => {
                path.close_path();
                has_subpath = false;
            }
            _ => return None,
        }
    }
    if offset != data.len() || path.is_empty() {
        return None;
    }
    let color = |rgba: u32| {
        ((rgba & 0xff) != 0).then(|| {
            Color::from_rgba8(
                (rgba >> 24) as u8,
                (rgba >> 16) as u8,
                (rgba >> 8) as u8,
                rgba as u8,
            )
        })
    };
    let cap = match line_cap {
        1 => Cap::Round,
        2 => Cap::Square,
        _ => Cap::Butt,
    };
    let join = match line_join {
        1 => Join::Round,
        2 => Join::Bevel,
        _ => Join::Miter,
    };
    Some(Arc::new(crate::style::VectorPath {
        path: Arc::new(path),
        fill: color(fill),
        stroke: color(stroke),
        even_odd: fill_rule == 1,
        stroke_style: Stroke::new(f64::from(stroke_width))
            .with_caps(cap)
            .with_join(join)
            .with_miter_limit(f64::from(miter_limit)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x3150_4257u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&60u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0x4f46_e5ffu32.to_le_bytes());
        bytes.extend_from_slice(&2f32.to_le_bytes());
        bytes.extend_from_slice(&[0, 1, 1, 0]);
        bytes.extend_from_slice(&4f32.to_le_bytes());
        for (command, x, y) in [(1u8, 0f32, 0f32), (2, 20f32, 10f32)] {
            bytes.extend_from_slice(&[command, 0, 0, 0]);
            bytes.extend_from_slice(&x.to_le_bytes());
            bytes.extend_from_slice(&y.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn decodes_geometry_and_paint() {
        let vector = decode_vector_path(&line()).unwrap();
        assert_eq!(vector.path.elements().len(), 2);
        assert!(vector.fill.is_none());
        assert!(vector.stroke.is_some());
        assert_eq!(vector.stroke_style.width, 2.0);
    }

    #[test]
    fn rejects_truncation_nonfinite_coordinates_and_invalid_subpaths() {
        let valid = line();
        for size in 0..valid.len() {
            assert!(decode_vector_path(&valid[..size]).is_none(), "{size}");
        }
        let mut bytes = valid.clone();
        bytes[40..44].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_vector_path(&bytes).is_none());
        let mut bytes = valid.clone();
        bytes[36] = 2; // line without an initial move
        assert!(decode_vector_path(&bytes).is_none());
        let mut bytes = valid;
        bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_vector_path(&bytes).is_none());
    }
}
