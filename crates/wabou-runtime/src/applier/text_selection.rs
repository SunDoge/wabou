use super::*;

#[derive(Clone)]
pub(super) struct SelectableText {
    pub(super) text: Arc<str>,
    pub(super) layout: Arc<Layout<[u8; 4]>>,
    pub(super) origin: [f32; 2],
    pub(super) visual_y: std::ops::Range<f32>,
    pub(super) select_all: bool,
    pub(super) order: usize,
}

#[derive(Clone)]
pub(super) struct ActiveTextSelection {
    pub(super) anchor_target: u32,
    pub(super) focus_target: u32,
    pub(super) base_selection: Selection,
    pub(super) focus_selection: Selection,
    pub(super) granularity: TextSelectionGranularity,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TextSelectionSnapshot {
    text: Option<String>,
    kind: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum TextSelectionGranularity {
    #[default]
    Cluster,
    Word,
    Line,
}

#[derive(Default)]
pub(super) struct TextSelectionState {
    pub(super) selectable: HashMap<u32, SelectableText>,
    pub(super) order: Vec<u32>,
    pub(super) active: Option<ActiveTextSelection>,
    last_snapshot: TextSelectionSnapshot,
    event_target: Option<u32>,
    pub(super) last_click: Option<(Instant, u32, f64, f64, u8)>,
    pub(super) next_scroll: Option<Instant>,
}

impl Applier {
    pub(super) fn prepare_text_selection(
        &mut self,
        placed: &mut [PlacedNode],
        tcx: &mut TextContext,
    ) {
        self.text_selection.selectable.clear();
        self.text_selection.order.clear();
        for node in placed.iter_mut() {
            node.paint.selection_rects = Arc::from([]);
            let Some(text) = node.paint.text.clone() else {
                continue;
            };
            if !node.paint.text_selectable
                || node.paint.text_ellipsis
                || self.widget_manager.widgets.contains_key(&node.node_id)
            {
                continue;
            }
            let Some(&target) = self.node_store.node_to_solid.get(&node.node_id) else {
                continue;
            };
            let layout = wabou_shell::text::layout_text_styled_overflow(
                tcx,
                text.clone(),
                node.paint.font_size,
                node.paint.font_weight,
                node.paint.line_height,
                node.paint.text_align,
                wabou_shell::text::brush_for_color(node.paint.text_color),
                node.paint.text_runs.clone(),
                node.paint.font_family.as_ref(),
                (node.paint.wrap_text || node.paint.text_ellipsis)
                    .then_some((node.rect[2] - node.rect[0]).max(0.0)),
                node.paint.text_ellipsis,
            );
            let selectable = SelectableText {
                text,
                visual_y: node.content_origin[1]..node.content_origin[1] + layout.height().max(0.0),
                layout,
                origin: node.content_origin,
                select_all: node.paint.text_select_all,
                order: self.text_selection.order.len(),
            };
            self.text_selection.order.push(target);
            self.text_selection.selectable.insert(target, selectable);
        }

        let valid = self.text_selection.active.as_ref().is_none_or(|active| {
            self.text_selection
                .selectable
                .contains_key(&active.anchor_target)
                && self
                    .text_selection
                    .selectable
                    .contains_key(&active.focus_target)
        });
        if !valid {
            self.text_selection.active = None;
            self.text_selection.next_scroll = None;
            return;
        }
        if let Some(active) = &mut self.text_selection.active {
            let anchor = &self.text_selection.selectable[&active.anchor_target].layout;
            active.base_selection = active.base_selection.refresh(anchor);
            let focus = &self.text_selection.selectable[&active.focus_target].layout;
            active.focus_selection = active.focus_selection.refresh(focus);
        }
        for node in placed.iter_mut() {
            let Some(&target) = self.node_store.node_to_solid.get(&node.node_id) else {
                continue;
            };
            let Some(range) = self.text_selection_range(target) else {
                continue;
            };
            let text = &self.text_selection.selectable[&target];
            let selection = Selection::new(
                Cursor::from_byte_index(&text.layout, range.start, Affinity::Downstream),
                Cursor::from_byte_index(&text.layout, range.end, Affinity::Upstream),
            );
            node.paint.selection_rects = selection
                .geometry(&text.layout)
                .into_iter()
                .map(|(rect, _)| {
                    [
                        rect.x0 as f32,
                        rect.y0 as f32,
                        rect.x1 as f32,
                        rect.y1 as f32,
                    ]
                })
                .collect::<Vec<_>>()
                .into();
        }
    }

    pub(super) fn selection_from_point(
        text: &SelectableText,
        granularity: TextSelectionGranularity,
        x: f32,
        y: f32,
    ) -> Selection {
        if text.select_all {
            return Selection::new(
                Cursor::from_byte_index(&text.layout, 0, Affinity::Downstream),
                Cursor::from_byte_index(&text.layout, text.text.len(), Affinity::Upstream),
            );
        }
        match granularity {
            TextSelectionGranularity::Cluster => Selection::from_point(&text.layout, x, y),
            TextSelectionGranularity::Word => Selection::word_from_point(&text.layout, x, y),
            TextSelectionGranularity::Line => Selection::line_from_point(&text.layout, x, y),
        }
    }

    pub(super) fn begin_text_selection(
        &mut self,
        target: u32,
        x: f64,
        y: f64,
        modifiers: Modifiers,
    ) -> bool {
        if modifiers.shift() && self.text_selection.active.is_some() {
            self.text_selection.last_click = None;
            return self.extend_text_selection(Some(target), x, y);
        }
        let Some(text) = self.text_selection.selectable.get(&target) else {
            self.text_selection.last_click = None;
            return self.text_selection.active.take().is_some();
        };
        let local_x = x as f32 - text.origin[0];
        let local_y = y as f32 - text.origin[1];
        let now = Instant::now();
        let clicks = self.text_selection.last_click.map_or(
            1,
            |(time, last_target, last_x, last_y, count)| {
                if last_target == target
                    && now.duration_since(time) <= Duration::from_millis(400)
                    && (x - last_x).abs() <= 4.0
                    && (y - last_y).abs() <= 4.0
                {
                    count % 3 + 1
                } else {
                    1
                }
            },
        );
        self.text_selection.last_click = Some((now, target, x, y, clicks));
        let granularity = match clicks {
            2 => TextSelectionGranularity::Word,
            3 => TextSelectionGranularity::Line,
            _ => TextSelectionGranularity::Cluster,
        };
        let selection = Self::selection_from_point(text, granularity, local_x, local_y);
        self.text_selection.active = Some(ActiveTextSelection {
            anchor_target: target,
            focus_target: target,
            base_selection: selection,
            focus_selection: selection,
            granularity,
        });
        true
    }

    pub(super) fn extend_text_selection(
        &mut self,
        hit_target: Option<u32>,
        x: f64,
        y: f64,
    ) -> bool {
        if self.text_selection.active.is_none() {
            return false;
        }
        let target = hit_target
            .filter(|target| self.text_selection.selectable.contains_key(target))
            .or_else(|| {
                self.text_selection
                    .order
                    .iter()
                    .copied()
                    .min_by(|left, right| {
                        let distance = |target: u32| {
                            let text = &self.text_selection.selectable[&target];
                            let dx = if x < f64::from(text.origin[0]) {
                                f64::from(text.origin[0]) - x
                            } else if x > f64::from(text.origin[0] + text.layout.width()) {
                                x - f64::from(text.origin[0] + text.layout.width())
                            } else {
                                0.0
                            };
                            let dy = if y < f64::from(text.origin[1]) {
                                f64::from(text.origin[1]) - y
                            } else if y > f64::from(text.origin[1] + text.layout.height()) {
                                y - f64::from(text.origin[1] + text.layout.height())
                            } else {
                                0.0
                            };
                            dx * dx + dy * dy
                        };
                        distance(*left).total_cmp(&distance(*right))
                    })
            });
        let Some(target) = target else {
            return false;
        };
        let text = &self.text_selection.selectable[&target];
        let local_x = x as f32 - text.origin[0];
        let local_y = y as f32 - text.origin[1];
        let active = self.text_selection.active.as_mut().unwrap();
        active.focus_target = target;
        active.focus_selection = if target == active.anchor_target {
            active
                .base_selection
                .extend_to_point(&text.layout, local_x, local_y)
        } else {
            Self::selection_from_point(text, active.granularity, local_x, local_y)
        };
        true
    }

    pub(super) fn text_selection_range(&self, target: u32) -> Option<std::ops::Range<usize>> {
        let active = self.text_selection.active.as_ref()?;
        let anchor_index = self
            .text_selection
            .selectable
            .get(&active.anchor_target)?
            .order;
        let focus_index = self
            .text_selection
            .selectable
            .get(&active.focus_target)?
            .order;
        let target_index = self.text_selection.selectable.get(&target)?.order;
        if anchor_index == focus_index {
            return (target_index == anchor_index).then(|| active.focus_selection.text_range());
        }
        let anchor_range = active.base_selection.text_range();
        let focus_range = active.focus_selection.text_range();
        let text_len = self.text_selection.selectable.get(&target)?.text.len();
        if anchor_index < focus_index {
            match target_index {
                index if index < anchor_index || index > focus_index => None,
                index if index == anchor_index => Some(anchor_range.start..text_len),
                index if index == focus_index => Some(0..focus_range.end),
                _ => Some(0..text_len),
            }
        } else {
            match target_index {
                index if index < focus_index || index > anchor_index => None,
                index if index == focus_index => Some(focus_range.start..text_len),
                index if index == anchor_index => Some(0..anchor_range.end),
                _ => Some(0..text_len),
            }
        }
        .filter(|range| !range.is_empty())
    }

    pub(super) fn selected_text(&self) -> Option<String> {
        self.text_selection.active.as_ref()?;
        let mut selected = String::new();
        let mut previous_visual_y: Option<std::ops::Range<f32>> = None;
        for target in &self.text_selection.order {
            let Some(range) = self.text_selection_range(*target) else {
                continue;
            };
            let text = &self.text_selection.selectable[target];
            if previous_visual_y.as_ref().is_some_and(|previous| {
                previous.end <= text.visual_y.start || text.visual_y.end <= previous.start
            }) {
                selected.push('\n');
            }
            selected.push_str(&text.text[range]);
            previous_visual_y = Some(text.visual_y.clone());
        }
        (!selected.is_empty()).then_some(selected)
    }

    pub(super) fn sync_text_selection_change(&mut self) -> bool {
        let text = self.selected_text();
        let kind = text.as_ref().and_then(|_| {
            self.text_selection
                .active
                .as_ref()
                .map(|selection| match selection.granularity {
                    TextSelectionGranularity::Cluster => "simple",
                    TextSelectionGranularity::Word => "word",
                    TextSelectionGranularity::Line => "line",
                })
        });
        let snapshot = TextSelectionSnapshot { text, kind };
        if snapshot == self.text_selection.last_snapshot {
            return false;
        }
        if let Some(target) = self
            .text_selection
            .active
            .as_ref()
            .map(|selection| selection.anchor_target)
        {
            self.text_selection.event_target = Some(target);
        }
        self.text_selection.last_snapshot = snapshot.clone();
        let Some(target) = self.text_selection.event_target else {
            return false;
        };
        self.dispatch_json(
            target,
            event::TEXTSELECTIONCHANGE,
            &serde_json::json!({ "text": snapshot.text, "kind": snapshot.kind }).to_string(),
        )
    }

    pub(super) fn select_all_text(&mut self) -> bool {
        let Some((&anchor_target, &focus_target)) = self
            .text_selection
            .order
            .first()
            .zip(self.text_selection.order.last())
        else {
            return false;
        };
        let anchor = &self.text_selection.selectable[&anchor_target];
        let focus = &self.text_selection.selectable[&focus_target];
        let whole = |text: &SelectableText| {
            Selection::new(
                Cursor::from_byte_index(&text.layout, 0, Affinity::Downstream),
                Cursor::from_byte_index(&text.layout, text.text.len(), Affinity::Upstream),
            )
        };
        self.text_selection.active = Some(ActiveTextSelection {
            anchor_target,
            focus_target,
            base_selection: whole(anchor),
            focus_selection: whole(focus),
            granularity: TextSelectionGranularity::Cluster,
        });
        true
    }
}
