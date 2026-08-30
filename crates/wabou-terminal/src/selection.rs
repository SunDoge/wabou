use super::*;

impl TerminalWidget {
    pub(super) fn sync_selection_change(&mut self) {
        // Selection changes stay renderer-local while a pointer gesture is in
        // progress. PTY output, resize, and rio-vt side events can all call
        // this method mid-drag; only the gesture terminator publishes the
        // committed snapshot to Solid.
        if self.selecting {
            return;
        }
        let terminal = self.terminal.lock();
        let text = terminal.selection_to_string();
        let kind = text.as_ref().and_then(|_| {
            terminal
                .selection
                .as_ref()
                .map(|selection| match selection.ty {
                    SelectionType::Simple => "simple",
                    SelectionType::Block => "block",
                    SelectionType::Semantic => "word",
                    SelectionType::Lines => "line",
                })
        });
        drop(terminal);
        let selection = TerminalSelectionSnapshot { text, kind };
        if selection == self.last_reported_selection {
            return;
        }
        self.last_reported_selection = selection.clone();
        self.pending_node_events.push_back(WidgetNodeEvent::json(
            event::TERMINALSELECTIONCHANGE,
            serde_json::json!({ "text": selection.text, "kind": selection.kind }).to_string(),
        ));
    }

    pub(super) fn local_point(&self, x: f64, y: f64) -> (f32, f32) {
        (x as f32, y as f32)
    }

    pub(super) fn pointer_cell(&self, x: f64, y: f64, display_offset: usize) -> (Pos, Side) {
        let (local_x, local_y) = self.local_point(x, y);
        let local_y = local_y.max(0.0);
        let raw_column = local_x.max(0.0) / self.cell_width;
        let column = (raw_column.floor() as usize).min(self.size.columns.saturating_sub(1));
        let viewport_row = (local_y / self.line_height).floor() as usize;
        let viewport_row = viewport_row.min(self.size.rows.saturating_sub(1));
        let line = Line(viewport_row as i32 - display_offset as i32);
        let viewport_width = self.size.columns as f32 * self.cell_width;
        let side = if local_x <= 0.0 {
            Side::Left
        } else if local_x >= viewport_width {
            Side::Right
        } else if raw_column.fract() < 0.5 {
            Side::Left
        } else {
            Side::Right
        };
        (Pos::new(line, Column(column)), side)
    }

    pub(super) fn begin_selection(&mut self, x: f64, y: f64, block: bool) {
        let (local_x, local_y) = self.local_point(x, y);
        let now = Instant::now();
        let clicks = self.last_click.map_or(1, |(time, last_x, last_y, count)| {
            if now.duration_since(time) <= Duration::from_millis(400)
                && (local_x - last_x).abs() <= 4.0
                && (local_y - last_y).abs() <= 4.0
            {
                count % 3 + 1
            } else {
                1
            }
        });
        self.last_click = Some((now, local_x, local_y, clicks));
        let mut terminal = self.terminal.lock();
        let (point, side) = self.pointer_cell(x, y, terminal.display_offset());
        terminal.selection = Some(Selection::new(
            if block {
                SelectionType::Block
            } else {
                match clicks {
                    2 => SelectionType::Semantic,
                    3 => SelectionType::Lines,
                    _ => SelectionType::Simple,
                }
            },
            point,
            side,
        ));
        drop(terminal);
        self.selecting = true;
        self.selection_pointer_origin = Some((x, y));
        self.selection_dragged = false;
    }

    pub(super) fn begin_or_extend_selection(&mut self, x: f64, y: f64, modifiers: Modifiers) {
        if modifiers.shift() && self.terminal.lock().selection.is_some() {
            self.selecting = true;
            self.selection_pointer_origin = Some((x, y));
            self.selection_dragged = false;
            self.last_click = None;
            self.update_selection(x, y);
        } else {
            self.begin_selection(x, y, modifiers.alt());
        }
    }

    pub(super) fn update_selection(&mut self, x: f64, y: f64) {
        if let Some((origin_x, origin_y)) = self.selection_pointer_origin {
            self.selection_dragged |= (x - origin_x).hypot(y - origin_y) > SELECTION_DRAG_THRESHOLD;
        }
        let mut terminal = self.terminal.lock();
        let (_, local_y) = self.local_point(x, y);
        let viewport_height = self.size.rows as f32 * self.line_height;
        let outside = if local_y < 0.0 {
            -local_y
        } else if local_y >= viewport_height {
            local_y - viewport_height
        } else {
            0.0
        };
        if outside > 0.0 {
            let lines = (outside / self.line_height).ceil().clamp(1.0, 8.0) as i32;
            terminal.scroll_display(Scroll::Delta(if local_y < 0.0 { lines } else { -lines }));
        }
        let (point, side) = self.pointer_cell(x, y, terminal.display_offset());
        if let Some(selection) = terminal.selection.as_mut() {
            selection.update(point, side);
        }
        let above = local_y < 0.0;
        let below = local_y >= viewport_height;
        let can_continue_scrolling = if above {
            terminal.display_offset() < terminal.history_size()
        } else if below {
            terminal.display_offset() > 0
        } else {
            false
        };
        drop(terminal);
        self.selection_drag_point = Some((x, y));
        self.next_selection_scroll =
            can_continue_scrolling.then(|| Instant::now() + Duration::from_millis(50));
    }

    pub(super) fn finish_selection_gesture(&mut self) {
        if self.selection_dragged {
            self.last_click = None;
        }
        self.selecting = false;
        self.selection_pointer_origin = None;
        self.selection_dragged = false;
        self.selection_drag_point = None;
        self.next_selection_scroll = None;
        self.sync_selection_change();
    }

    pub(super) fn select_all(&mut self) {
        let mut terminal = self.terminal.lock();
        let start = Pos::new(Line(-(terminal.history_size() as i32)), Column(0));
        let end = Pos::new(
            terminal.bottommost_line(),
            Column(terminal.columns().saturating_sub(1)),
        );
        let mut selection = Selection::new(SelectionType::Lines, start, Side::Left);
        selection.update(end, Side::Right);
        terminal.selection = Some(selection);
        drop(terminal);

        self.selecting = false;
        self.selection_pointer_origin = None;
        self.selection_dragged = false;
        self.selection_drag_point = None;
        self.next_selection_scroll = None;
        self.last_click = None;
        self.sync_selection_change();
    }

    pub(super) fn tick_selection_autoscroll(&mut self) {
        if !self.selecting
            || !self
                .next_selection_scroll
                .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return;
        }
        if let Some((x, y)) = self.selection_drag_point {
            self.update_selection(x, y);
        }
    }

    pub(super) fn scroll_active_selection(
        &mut self,
        wheel: &wabou_shell_api::WheelEvent,
        lines: i32,
    ) {
        if lines == 0 {
            return;
        }
        let mut terminal = self.terminal.lock();
        terminal.scroll_display(Scroll::Delta(lines));
        let (point, side) = self.pointer_cell(
            wheel.position.x,
            wheel.position.y,
            terminal.display_offset(),
        );
        if let Some(selection) = terminal.selection.as_mut() {
            selection.update(point, side);
        }
        drop(terminal);
        self.selection_drag_point = Some((wheel.position.x, wheel.position.y));
    }

    pub(super) fn mouse_grid_position(&self, x: f64, y: f64) -> (usize, usize) {
        let (local_x, local_y) = self.local_point(x, y);
        let column = ((local_x.max(0.0) / self.cell_width).floor() as usize)
            .min(self.size.columns.saturating_sub(1));
        let row = ((local_y.max(0.0) / self.line_height).floor() as usize)
            .min(self.size.rows.saturating_sub(1));
        (column + 1, row + 1)
    }

    pub(super) fn hyperlink_at(&self, x: f64, y: f64) -> Option<String> {
        let terminal = self.terminal.lock();
        let (point, _) = self.pointer_cell(x, y, terminal.display_offset());
        let hyperlink = terminal.cell_hyperlink(point.row, point.col).or_else(|| {
            let square = terminal.grid[point];
            (square.wide() == Wide::Spacer && point.col.0 > 0)
                .then(|| terminal.cell_hyperlink(point.row, Column(point.col.0 - 1)))
                .flatten()
        });
        hyperlink.map(|link| link.uri().to_owned())
    }
}
