//! Row projection. Geometry is local Rust data, never durable state.
use std::ops::Range;

/// Contiguous rows to materialize, or empty space for omitted rows.
#[derive(Clone, Debug, PartialEq)]
pub enum RowSpan {
    Rows(Range<usize>),
    Space { start: usize, height: f32 },
}

/// A viewport with symmetric overscan and one optionally retained row.
/// Applications still provide stable row keys and exact row heights. Spacer
/// heights and node counts remain subject to normal runtime admission limits.
#[derive(Clone, Debug)]
pub struct VirtualRows {
    total: usize,
    height: f32,
    offsets: Option<Vec<f64>>,
    visible: Range<usize>,
    pinned: Option<usize>,
}
impl VirtualRows {
    /// Reject invalid geometry; clamp offsets beyond the end of the content.
    pub fn new(
        total: usize,
        height: f32,
        offset: f32,
        viewport: f32,
        overscan: usize,
    ) -> Option<Self> {
        if !height.is_finite()
            || height <= 0.0
            || !offset.is_finite()
            || offset < 0.0
            || !viewport.is_finite()
            || viewport < 0.0
            || !(total as f32 * height).is_finite()
        {
            return None;
        }
        let extent = total as f64 * f64::from(height);
        let offset = f64::from(offset).min((extent - f64::from(viewport)).max(0.0));
        let first = (offset / f64::from(height)).floor() as usize;
        let last = ((offset + f64::from(viewport)) / f64::from(height)).ceil() as usize;
        Some(Self {
            total,
            height,
            offsets: None,
            visible: first.saturating_sub(overscan).min(total)
                ..last.saturating_add(overscan).min(total),
            pinned: None,
        })
    }
    /// Project rows with known, positive individual heights. Builds a prefix
    /// index in O(rows) time/storage, then locates the viewport by binary search.
    /// Heights must match rendered geometry (including any authored row spacing).
    /// This does not measure widgets or observe native viewport changes.
    ///
    /// ```
    /// use blockui::{RowSpan, VirtualRows};
    /// let rows = VirtualRows::with_heights(&[20.0, 60.0, 30.0], 20.0, 30.0, 0)
    ///     .unwrap().pin(0).unwrap();
    /// assert_eq!(rows.visible(), 1..2);
    /// assert_eq!(rows.spans(), vec![
    ///     RowSpan::Rows(0..1), RowSpan::Rows(1..2),
    ///     RowSpan::Space { start: 2, height: 30.0 },
    /// ]);
    /// ```
    pub fn with_heights(
        heights: &[f32],
        offset: f32,
        viewport: f32,
        overscan: usize,
    ) -> Option<Self> {
        if !offset.is_finite() || offset < 0.0 || !viewport.is_finite() || viewport < 0.0 {
            return None;
        }
        let mut offsets = Vec::with_capacity(heights.len().checked_add(1)?);
        offsets.push(0.0_f64);
        for &height in heights {
            if !height.is_finite() || height <= 0.0 {
                return None;
            }
            let previous = offsets.last().copied()?;
            let end = previous + f64::from(height);
            if end <= previous || end > f64::from(f32::MAX) {
                return None;
            }
            offsets.push(end);
        }
        let total = heights.len();
        let extent = *offsets.last()?;
        let offset = f64::from(offset).min((extent - f64::from(viewport)).max(0.0));
        let first = offsets
            .partition_point(|&start| start <= offset)
            .saturating_sub(1)
            .min(total);
        let last = offsets
            .partition_point(|&end| end < offset + f64::from(viewport))
            .min(total);
        Some(Self {
            total,
            height: 0.0,
            offsets: Some(offsets),
            visible: first.saturating_sub(overscan)..last.saturating_add(overscan).min(total),
            pinned: None,
        })
    }
    fn span_height(&self, start: usize, end: usize) -> f32 {
        if let Some(offsets) = &self.offsets {
            (offsets[end] - offsets[start]) as f32
        } else {
            (end - start) as f32 * self.height
        }
    }
    /// Keep an edited/focused row materialized outside the viewport.
    pub fn pin(mut self, index: usize) -> Option<Self> {
        if index >= self.total {
            return None;
        }
        self.pinned = Some(index);
        Some(self)
    }
    pub fn visible(&self) -> Range<usize> {
        self.visible.clone()
    }
    /// Ordered disjoint spans cover the entire dataset without duplicating a pin.
    pub fn spans(&self) -> Vec<RowSpan> {
        let mut ranges = Vec::new();
        if !self.visible.is_empty() {
            ranges.push(self.visible.clone());
        }
        if let Some(index) = self.pinned
            && !self.visible.contains(&index)
        {
            ranges.push(index..index + 1);
        }
        ranges.sort_unstable_by_key(|range| range.start);
        let mut spans = Vec::new();
        let mut end = 0;
        for range in ranges {
            if range.start > end {
                spans.push(RowSpan::Space {
                    start: end,
                    height: self.span_height(end, range.start),
                });
            }
            end = range.end;
            spans.push(RowSpan::Rows(range));
        }
        if end < self.total {
            spans.push(RowSpan::Space {
                start: end,
                height: self.span_height(end, self.total),
            });
        }
        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variable_rows_cover_viewports_and_preserve_geometry_with_distant_pins() {
        let heights = [10.0, 70.0, 5.0, 120.0, 15.0, 40.0];
        let extent: f32 = heights.iter().sum();
        for offset in 0..300 {
            for viewport in [0.0, 1.0, 50.0, 1000.0] {
                for pin in 0..heights.len() {
                    let rows = VirtualRows::with_heights(&heights, offset as f32, viewport, 0)
                        .unwrap()
                        .pin(pin)
                        .unwrap();
                    let actual_offset = (offset as f32).min((extent - viewport).max(0.0));
                    let mut start = 0.0;
                    for (index, &height) in heights.iter().enumerate() {
                        if start < actual_offset + viewport && start + height > actual_offset {
                            assert!(rows.visible().contains(&index));
                        }
                        start += height;
                    }
                    let mut position = 0.0;
                    let mut seen = vec![false; heights.len()];
                    for span in rows.spans() {
                        match span {
                            RowSpan::Rows(range) => {
                                assert_eq!(position, heights[..range.start].iter().sum::<f32>());
                                for index in range {
                                    assert!(!seen[index]);
                                    seen[index] = true;
                                    position += heights[index];
                                }
                            }
                            RowSpan::Space { start, height } => {
                                assert_eq!(position, heights[..start].iter().sum::<f32>());
                                position += height;
                            }
                        }
                    }
                    assert!(seen[pin]);
                    assert_eq!(position, extent);
                }
            }
        }
    }
    #[test]
    fn variable_uniform_rows_match_fixed_rows_and_reject_invalid_geometry() {
        for offset in [0.0, 11.0, 44.0, 10000.0] {
            for viewport in [0.0, 1.0, 200.0, 10000.0] {
                for overscan in [0, 2, usize::MAX] {
                    let fixed = VirtualRows::new(10, 44.0, offset, viewport, overscan)
                        .unwrap()
                        .pin(9)
                        .unwrap();
                    let variable =
                        VirtualRows::with_heights(&[44.0; 10], offset, viewport, overscan)
                            .unwrap()
                            .pin(9)
                            .unwrap();
                    assert_eq!(fixed.visible(), variable.visible());
                    assert_eq!(fixed.spans(), variable.spans());
                }
            }
        }
        for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(VirtualRows::with_heights(&[10.0, invalid], 0.0, 20.0, 0).is_none());
        }
        for (offset, viewport) in [
            (-1.0, 20.0),
            (f32::INFINITY, 20.0),
            (0.0, f32::NAN),
            (0.0, -1.0),
        ] {
            assert!(VirtualRows::with_heights(&[10.0], offset, viewport, 0).is_none());
        }
        assert!(VirtualRows::with_heights(&[f32::MAX, f32::MAX], 0.0, 20.0, 0).is_none());
        assert!(VirtualRows::with_heights(&[f32::MAX, 1.0], 0.0, 20.0, 0).is_none());
        assert!(
            VirtualRows::with_heights(&[], 100.0, 20.0, 2)
                .unwrap()
                .spans()
                .is_empty()
        );
    }
    #[test]
    fn every_viewport_and_pin_preserves_extent_and_bounds_projection() {
        for first in 0..100 {
            for pin in [0, first, 99] {
                let rows = VirtualRows::new(100, 44.0, first as f32 * 44.0, 200.0, 2)
                    .unwrap()
                    .pin(pin)
                    .unwrap();
                let mut extent = 0.0;
                let mut count = 0;
                let mut pins = 0;
                let mut next = 0;
                for span in rows.spans() {
                    match span {
                        RowSpan::Rows(range) => {
                            assert_eq!(range.start, next);
                            next = range.end;
                            pins += usize::from(range.contains(&pin));
                            count += range.len();
                            extent += range.len() as f32 * 44.0;
                        }
                        RowSpan::Space { start, height } => {
                            assert_eq!(start, next);
                            next += (height / 44.0) as usize;
                            extent += height;
                        }
                    }
                }
                assert_eq!(next, 100);
                assert_eq!(extent, 4400.0);
                assert_eq!(pins, 1);
                assert!(count <= 11);
            }
        }
    }
    #[test]
    fn geometry_rejects_invalid_values_and_handles_empty_and_oversized_viewports() {
        for height in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(VirtualRows::new(10, height, 0.0, 20.0, 2).is_none());
        }
        assert!(VirtualRows::new(10, 44.0, -1.0, 20.0, 2).is_none());
        assert!(VirtualRows::new(10, 44.0, 0.0, f32::NAN, 2).is_none());
        assert!(
            VirtualRows::new(0, 44.0, 0.0, 20.0, 2)
                .unwrap()
                .spans()
                .is_empty()
        );
        assert_eq!(
            VirtualRows::new(3, 44.0, 10000.0, 500.0, usize::MAX)
                .unwrap()
                .visible(),
            0..3
        );
        assert!(
            VirtualRows::new(3, 44.0, 0.0, 20.0, 2)
                .unwrap()
                .pin(3)
                .is_none()
        );
        assert_eq!(
            VirtualRows::new(100, 44.0, 10000.0, 200.0, 2)
                .unwrap()
                .visible(),
            93..100
        );
    }
}
