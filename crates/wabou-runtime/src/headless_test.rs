#[derive(Clone, Copy)]
pub(super) struct HeadlessViewport {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) scale_factor: f64,
    pub(super) window_index: usize,
}

impl HeadlessViewport {
    pub(super) fn with_logical_size(self, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..self
        }
    }

    pub(super) fn from_environment() -> crate::Result<Self> {
        fn parse<T>(name: &'static str, default: T) -> crate::Result<T>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            let Some(value) = std::env::var_os(name) else {
                return Ok(default);
            };
            value
                .to_string_lossy()
                .parse()
                .map_err(|error| crate::Error::TestScenario {
                    message: format!("invalid {name}: {error}"),
                })
        }

        let width = parse("WABOU_TEST_VIEWPORT_WIDTH", 1100_u32)?;
        let height = parse("WABOU_TEST_VIEWPORT_HEIGHT", 720_u32)?;
        let scale_factor = parse("WABOU_TEST_SCALE_FACTOR", 1.0_f64)?;
        let window_id = parse("WABOU_TEST_CAPTURE_WINDOW_ID", 1_u32)?;
        if width == 0 || height == 0 {
            return Err(crate::Error::TestScenario {
                message: "headless viewport dimensions must be greater than zero".into(),
            });
        }
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return Err(crate::Error::TestScenario {
                message: "headless scale factor must be finite and greater than zero".into(),
            });
        }
        let window_index = window_id
            .checked_sub(1)
            .ok_or_else(|| crate::Error::TestScenario {
                message: "headless capture window id must be greater than zero".into(),
            })? as usize;
        Ok(Self {
            width,
            height,
            scale_factor,
            window_index,
        })
    }

    pub(super) fn physical_width(self) -> u32 {
        self.physical_width_for(self.width)
    }

    pub(super) fn physical_width_for(self, width: u32) -> u32 {
        (f64::from(width) * self.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32
    }

    pub(super) fn physical_height(self) -> u32 {
        self.physical_height_for(self.height)
    }

    pub(super) fn physical_height_for(self, height: u32) -> u32 {
        (f64::from(height) * self.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::HeadlessViewport;

    #[test]
    fn converts_logical_dimensions_with_one_shared_scale_policy() {
        let viewport = HeadlessViewport {
            width: 801,
            height: 601,
            scale_factor: 1.5,
            window_index: 0,
        };

        assert_eq!(viewport.physical_width(), 1202);
        assert_eq!(viewport.physical_height(), 902);
        assert_eq!(viewport.physical_width_for(640), 960);
        assert_eq!(viewport.with_logical_size(640, 480).window_index, 0);
    }
}
