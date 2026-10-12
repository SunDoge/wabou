//! Readable tabular content with stable row identities and ordinary controls.
use crate::*;

pub struct TableRow {
    key: Key,
    cells: Vec<View>,
}
impl TableRow {
    /// Identity belongs to the record, not its current position in a sorted page.
    pub fn new(key: impl Into<Key>, cells: Vec<View>) -> Self {
        Self {
            key: key.into(),
            cells,
        }
    }
}
/// Active ordering exposed on one column header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TableSortDirection {
    Ascending = 1,
    Descending = 2,
    Other = 3,
}
pub struct Table {
    key: Key,
    caption: Binding,
    headers: Vec<View>,
    widths: Option<Vec<f32>>,
    sorted_column: Option<(usize, TableSortDirection)>,
}
impl Table {
    pub fn new(caption: impl Into<Binding>, headers: Vec<View>) -> Self {
        Self {
            key: Key::named("component.table"),
            caption: caption.into(),
            headers,
            widths: None,
            sorted_column: None,
        }
    }
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
    /// Preferred logical-pixel widths shared by headers and records. Zero
    /// shares remaining space with other automatic columns. Fixed columns can
    /// shrink in a constrained viewport; content still wraps normally.
    pub fn column_widths(mut self, widths: Vec<f32>) -> Self {
        self.widths = Some(widths);
        self
    }
    /// Expose the active ordering on exactly one header. Sorting data and
    /// composing a header button remain the caller's responsibility.
    pub fn sorted_column(mut self, column: usize, direction: TableSortDirection) -> Self {
        self.sorted_column = Some((column, direction));
        self
    }
    /// Every row must have exactly one cell for each header. Cells may contain
    /// ordinary controls; the table itself introduces no keyboard tab stop.
    pub fn rows(self, rows: Vec<TableRow>) -> Result<View> {
        if self.headers.is_empty() || rows.iter().any(|r| r.cells.len() != self.headers.len()) {
            return Err(Error::InvalidView("table rows must match nonempty headers"));
        }
        if self.widths.as_ref().is_some_and(|widths| {
            widths.len() != self.headers.len()
                || widths
                    .iter()
                    .any(|width| !width.is_finite() || !(0.0..=10000.0).contains(width))
        }) {
            return Err(Error::InvalidView("invalid table column widths"));
        }
        if self
            .sorted_column
            .is_some_and(|(column, _)| column >= self.headers.len())
        {
            return Err(Error::InvalidView("sorted table column is out of range"));
        }
        let caption_key = self.key.scoped(abi::key("caption"));
        let header_key = self.key.scoped(abi::key("header"));
        let mut identities = std::collections::HashSet::new();
        for row in &rows {
            if row.key.0 == 0 || !identities.insert(row.key.0) {
                return Err(Error::InvalidView("table rows need distinct record keys"));
            }
        }
        let make_row = |key: Key, cells: Vec<View>, header: bool| {
            let children = cells
                .into_iter()
                .enumerate()
                .map(|(i, content)| {
                    let mut cell = Column {
                        padding: 8.0,
                        gap: 0.0,
                        ..default()
                    }
                    .key(key.scoped(i as u64))
                    .children(vec![content]);
                    if let Some(widths) = &self.widths {
                        cell.node.presentation.width = widths[i];
                    }
                    cell.node.semantics.role = if header {
                        abi::semantics::COLUMN_HEADER
                    } else {
                        abi::semantics::CELL
                    };
                    if header
                        && let Some((column, direction)) = self.sorted_column
                        && column == i
                    {
                        cell.node.semantics.sort_direction = direction as u32;
                    }
                    cell
                })
                .collect();
            let mut row = Row {
                gap: 0.0,
                padding: 0.0,
                ..default()
            }
            .key(key)
            .children(children);
            row.node.semantics.role = abi::semantics::TABLE_ROW;
            if header {
                row = row.style(ViewStyle {
                    surface: Surface::Canvas,
                    ..default()
                });
            }
            row
        };
        let mut children = vec![
            Description::new(self.caption).key(caption_key).into_view(),
            make_row(header_key, self.headers, true),
        ];
        for row in rows {
            children.push(make_row(
                self.key.scoped(abi::key("record")).scoped(row.key.0),
                row.cells,
                false,
            ));
        }
        let mut view = Column {
            gap: 0.0,
            padding: 0.0,
            ..default()
        }
        .key(self.key)
        .children(children);
        view.node.semantics.role = abi::semantics::TABLE;
        view.node.semantics.labelled_by = caption_key.0;
        Ok(view)
    }
}
