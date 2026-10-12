//! Popup geometry shared by Select painting and pointer routing.
//! Coordinates are logical pixels. Rows use a bounded visible window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupLayout {
    pub rect: [f32; 4],
    pub row_height: f32,
    pub start: usize,
    pub end: usize,
}
impl PopupLayout {
    /// Prefer below the control, flip above when it offers more room, and keep
    /// the active row visible. Tiny/invalid viewports produce no popup.
    pub fn new(anchor: [f32; 4], viewport: [f32; 2], count: usize, active: usize) -> Option<Self> {
        if count == 0
            || count > blockui_abi::select::MAX_OPTIONS
            || active >= count
            || anchor
                .iter()
                .chain(viewport.iter())
                .any(|value| !value.is_finite())
            || viewport[0] < 16.0
            || viewport[1] < 40.0
            || anchor[2] <= anchor[0]
            || anchor[3] <= anchor[1]
        {
            return None;
        }
        let margin = 4.0;
        let row_height = 32.0;
        let top_edge = margin;
        let bottom_edge = viewport[1] - margin;
        let below = (bottom_edge - anchor[3] - margin).max(0.0);
        let above = (anchor[1] - margin - top_edge).max(0.0);
        let desired = row_height * count.min(8) as f32;
        let flip = below < desired && above > below;
        let room = if flip { above } else { below };
        let rows = ((room / row_height).floor() as usize).min(count).min(8);
        if rows == 0 {
            return None;
        }
        let height = rows as f32 * row_height;
        let width = (anchor[2] - anchor[0])
            .max(120.0)
            .min(viewport[0] - margin * 2.0);
        let left = anchor[0].clamp(margin, viewport[0] - margin - width);
        let top = if flip {
            anchor[1] - margin - height
        } else {
            anchor[3] + margin
        };
        let top = top.clamp(top_edge, bottom_edge - height);
        let start = active.saturating_sub(rows / 2).min(count - rows);
        Some(Self {
            rect: [left, top, left + width, top + height],
            row_height,
            start,
            end: start + rows,
        })
    }
    /// Preserve an existing visible window until its active row leaves it.
    pub fn retain_window(mut self, previous_start: usize, count: usize, active: usize) -> Self {
        let rows = self.end - self.start;
        if count < rows || active >= count {
            return self;
        }
        let mut start = previous_start.min(count - rows);
        if active < start {
            start = active;
        }
        if active >= start + rows {
            start = active + 1 - rows;
        }
        self.start = start;
        self.end = start + rows;
        self
    }
    /// Return an option index, never an enabled/committed value. Callers must
    /// resolve it against the current model and validate before dispatch.
    pub fn hit(&self, point: [f32; 2]) -> Option<usize> {
        if point.iter().any(|value| !value.is_finite())
            || point[0] < self.rect[0]
            || point[0] >= self.rect[2]
            || point[1] < self.rect[1]
            || point[1] >= self.rect[3]
        {
            return None;
        }
        let index = self.start + ((point[1] - self.rect[1]) / self.row_height).floor() as usize;
        (index < self.end).then_some(index)
    }
    pub fn row_rect(&self, index: usize) -> Option<[f32; 4]> {
        if !(self.start..self.end).contains(&index) {
            return None;
        }
        let top = self.rect[1] + (index - self.start) as f32 * self.row_height;
        Some([self.rect[0], top, self.rect[2], top + self.row_height])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flips_clamps_and_keeps_last_of_256_options_reachable() {
        for viewport in [[320.0, 480.0], [640.0, 720.0]] {
            let layout = PopupLayout::new(
                [
                    viewport[0] - 80.0,
                    viewport[1] - 50.0,
                    viewport[0],
                    viewport[1] - 14.0,
                ],
                viewport,
                256,
                255,
            )
            .unwrap();
            assert!(layout.rect[3] < viewport[1] - 50.0);
            assert!(layout.rect[0] >= 4.0 && layout.rect[2] <= viewport[0] - 4.0);
            assert_eq!(layout.end, 256);
            assert!(layout.end - layout.start <= 8);
            let row = layout.row_rect(255).unwrap();
            assert_eq!(
                layout.hit([(row[0] + row[2]) / 2.0, (row[1] + row[3]) / 2.0]),
                Some(255)
            );
            assert_eq!(layout.hit([layout.rect[2], row[1]]), None);
            assert_eq!(layout.hit([row[0], layout.rect[3]]), None);
        }
    }
    #[test]
    fn hover_keeps_window_stationary_and_navigation_reveals() {
        let layout = PopupLayout::new([16.0, 20.0, 200.0, 56.0], [320.0, 480.0], 256, 6)
            .unwrap()
            .retain_window(0, 256, 6);
        assert_eq!((layout.start, layout.end), (0, 8));
        let next = layout.retain_window(0, 256, 9);
        assert_eq!((next.start, next.end), (2, 10));
        assert_eq!(next.retain_window(2, 256, 255).end, 256);
        assert_eq!(next.retain_window(2, 256, 0).start, 0);
    }
    #[test]
    fn below_and_tiny_or_invalid_geometry() {
        let layout = PopupLayout::new([16.0, 20.0, 200.0, 56.0], [320.0, 480.0], 2, 0).unwrap();
        assert_eq!(layout.rect, [16.0, 60.0, 200.0, 124.0]);
        assert_eq!(layout.hit([30.0, 92.0]), Some(1));
        assert_eq!(layout.hit([f32::NAN, 60.0]), None);
        assert!(PopupLayout::new([0.0, 0.0, 10.0, 36.0], [10.0, 30.0], 2, 0).is_none());
        assert!(PopupLayout::new([f32::NAN, 0.0, 200.0, 36.0], [320.0, 480.0], 2, 0).is_none());
        assert!(PopupLayout::new([0.0, 0.0, 200.0, 36.0], [320.0, 480.0], 257, 0).is_none());
    }
}
