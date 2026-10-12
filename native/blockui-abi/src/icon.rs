//! Owned monochrome SVG resource decoded from borrowed binding bytes.
//! This Rust type is host-local and is never passed through the C ABI.
#[derive(Clone, Debug, PartialEq)]
pub struct IconData {
    pub svg: String,
    pub size: f32,
    /// Zero preserves the SVG's authored stroke widths.
    pub stroke_width: f32,
    pub only: bool,
}
impl IconData {
    pub fn encode(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.size,
            self.stroke_width,
            u8::from(self.only),
            self.svg
        )
    }
    pub fn decode(wire: &str) -> Result<Self, &'static str> {
        if wire.len() > 64 * 1024 {
            return Err("icon exceeds byte budget");
        }
        let mut parts = wire.splitn(4, '\n');
        let size: f32 = parts
            .next()
            .ok_or("missing icon size")?
            .parse()
            .map_err(|_| "invalid icon size")?;
        let stroke_width: f32 = parts
            .next()
            .ok_or("missing icon stroke")?
            .parse()
            .map_err(|_| "invalid icon stroke")?;
        let only = match parts.next() {
            Some("0") => false,
            Some("1") => true,
            _ => return Err("invalid icon mode"),
        };
        let svg = parts.next().ok_or("missing icon SVG")?;
        if !size.is_finite()
            || !(1.0..=256.0).contains(&size)
            || !stroke_width.is_finite()
            || !(0.0..=24.0).contains(&stroke_width)
            || svg.is_empty()
        {
            return Err("invalid icon resource");
        }
        Ok(Self {
            svg: svg.into(),
            size,
            stroke_width,
            only,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icon_wire_roundtrips_svg_newlines_without_executable_tokens() {
        let icon = IconData {
            svg: "<svg>\n<path/>\n</svg>".into(),
            size: 18.0,
            stroke_width: 1.5,
            only: true,
        };
        assert_eq!(IconData::decode(&icon.encode()), Ok(icon));
    }
    #[test]
    fn icon_wire_rejects_nonfinite_dimensions_unknown_modes_and_oversize_assets() {
        for wire in [
            "NaN\n2\n0\n<svg/>",
            "0\n2\n0\n<svg/>",
            "16\ninf\n0\n<svg/>",
            "16\n2\n2\n<svg/>",
            "16\n2\n0\n",
        ] {
            assert!(IconData::decode(wire).is_err());
        }
        assert!(IconData::decode(&format!("16\n2\n0\n{}", "x".repeat(65536))).is_err());
    }
}
