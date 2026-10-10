//! Coordinate and keyboard-navigation contracts shared by native grids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridExtent {
    rows: usize,
    columns: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridMove {
    Left,
    Right,
    Up,
    Down,
    RowStart,
    RowEnd,
    First,
    Last,
    PageUp,
    PageDown,
}
impl TryFrom<i64> for GridMove {
    type Error = ();
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            2 => Ok(Self::Up),
            3 => Ok(Self::Down),
            4 => Ok(Self::RowStart),
            5 => Ok(Self::RowEnd),
            6 => Ok(Self::First),
            7 => Ok(Self::Last),
            8 => Ok(Self::PageUp),
            9 => Ok(Self::PageDown),
            _ => Err(()),
        }
    }
}
impl GridExtent {
    pub fn rows(self) -> usize {
        self.rows
    }
    pub fn columns(self) -> usize {
        self.columns
    }
    pub fn new(rows: usize, columns: usize) -> Option<Self> {
        (rows > 0 && columns > 0 && rows.checked_mul(columns).is_some())
            .then_some(Self { rows, columns })
    }
    pub fn index(self, row: usize, column: usize) -> Option<usize> {
        (row < self.rows && column < self.columns).then(|| row * self.columns + column)
    }
    pub fn coordinates(self, index: usize) -> Option<(usize, usize)> {
        (index < self.rows * self.columns).then(|| (index / self.columns, index % self.columns))
    }
    /// Arrow moves clamp to the same row/column at edges. Page moves preserve
    /// the column and move at least one row; Home/End use the current row.
    pub fn navigate(self, index: usize, movement: GridMove, page_rows: usize) -> Option<usize> {
        let (mut row, mut column) = self.coordinates(index)?;
        match movement {
            GridMove::Left => column = column.saturating_sub(1),
            GridMove::Right => column = column.saturating_add(1).min(self.columns - 1),
            GridMove::Up => row = row.saturating_sub(1),
            GridMove::Down => row = row.saturating_add(1).min(self.rows - 1),
            GridMove::RowStart => column = 0,
            GridMove::RowEnd => column = self.columns - 1,
            GridMove::First => {
                row = 0;
                column = 0;
            }
            GridMove::Last => {
                row = self.rows - 1;
                column = self.columns - 1;
            }
            GridMove::PageUp => row = row.saturating_sub(page_rows.max(1)),
            GridMove::PageDown => row = row.saturating_add(page_rows.max(1)).min(self.rows - 1),
        }
        self.index(row, column)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_preserves_axes_and_clamps_without_overflow() {
        let grid = GridExtent::new(100, 26).unwrap();
        assert_eq!(grid.navigate(0, GridMove::Left, 5), Some(0));
        assert_eq!(grid.navigate(25, GridMove::Right, 5), Some(25));
        assert_eq!(grid.navigate(27, GridMove::Up, 5), Some(1));
        assert_eq!(grid.navigate(27, GridMove::RowEnd, 5), Some(51));
        assert_eq!(grid.navigate(27, GridMove::PageDown, 5), Some(157));
        assert_eq!(grid.navigate(27, GridMove::PageUp, usize::MAX), Some(1));
        assert_eq!(
            grid.navigate(27, GridMove::PageDown, usize::MAX),
            Some(2575)
        );
        assert_eq!(grid.navigate(27, GridMove::Last, 5), Some(2599));
        assert_eq!(grid.navigate(2600, GridMove::First, 5), None);
        assert_eq!(grid.coordinates(2599), Some((99, 25)));
        assert_eq!(GridExtent::new(0, 26), None);
        assert_eq!(GridExtent::new(2, usize::MAX), None);
        for index in 0..2600 {
            let (row, column) = grid.coordinates(index).unwrap();
            assert_eq!(grid.index(row, column), Some(index));
        }
    }
}
