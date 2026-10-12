//! Controlled, compact pagination using ordinary host-owned buttons.
use crate::*;

#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with view and mount the returned view"]
pub struct Pagination {
    #[setters(generate, into)]
    pub key: Key,
    /// Include First/Last buttons, useful for larger collections.
    #[setters(generate)]
    pub edge_controls: bool,
    /// Show a bounded row of direct-page buttons above the compact controls.
    #[setters(generate)]
    pub numbered: bool,
    #[setters(generate, into)]
    pub previous_label: String,
    #[setters(generate, into)]
    pub next_label: String,
    page: State<i64>,
    pages: i64,
}
impl Pagination {
    /// `page` is zero-based. Zero pages displays an empty, disabled paginator.
    /// An out-of-range state is clamped for display without writing during view
    /// construction; the next enabled action commits a valid page.
    pub fn new(page: impl Into<State<i64>>, pages: i64) -> Self {
        let page = page.into();
        Self {
            key: Key::named("component.pagination").scoped(page.key()),
            edge_controls: false,
            numbered: false,
            previous_label: "Previous".into(),
            next_label: "Next".into(),
            page,
            pages,
        }
    }
    pub fn view(self, cx: &Cx<'_>) -> Result<View> {
        if self.pages < 0 {
            return Err(Error::InvalidView(
                "pagination page count must be nonnegative",
            ));
        }
        let pages = self.pages;
        let page = self.page;
        let last = pages.saturating_sub(1).max(0);
        let current = page.get(cx)?.clamp(0, last);
        let button = |name: &str, part: &'static str, enabled, movement: i64| {
            Button {
                enabled,
                variant: ButtonVariant::Secondary,
                ..Button::label(name).key(self.key.scoped(abi::key(part)))
            }
            .on_click(move |cx| {
                let current = page.get(cx)?.clamp(0, last);
                let next = match movement {
                    i64::MIN => 0,
                    i64::MAX => last,
                    delta => current.saturating_add(delta).clamp(0, last),
                };
                page.set(cx, next)
            })
        };
        let mut children = Vec::with_capacity(if self.edge_controls { 5 } else { 3 });
        if self.edge_controls {
            children.push(button("First", "first", pages > 0 && current > 0, i64::MIN));
        }
        children.push(button(
            &self.previous_label,
            "previous",
            pages > 0 && current > 0,
            -1,
        ));
        let mut status = Column {
            padding: 0.0,
            gap: 0.0,
            ..default()
        }
        .key(self.key.scoped(abi::key("status")))
        .children(vec![
            Text::new(format!(
                "Page {} of {pages}",
                if pages == 0 { 0 } else { current + 1 }
            ))
            .key(self.key.scoped(abi::key("count")))
            .into_view(),
        ]);
        status.node.semantics.role = abi::semantics::STATUS;
        children.push(status);
        children.push(button(
            &self.next_label,
            "next",
            pages > 0 && current < last,
            1,
        ));
        if self.edge_controls {
            children.push(button(
                "Last",
                "last",
                pages > 0 && current < last,
                i64::MAX,
            ));
        }
        let mut view = Row {
            padding: 0.0,
            gap: 8.0,
            ..default()
        }
        .key(self.key)
        .children(children);
        if !self.numbered || pages == 0 {
            view.node.semantics.role = abi::semantics::NAVIGATION;
            return Ok(view);
        }
        let numbers_key = self.key.scoped(abi::key("numbers"));
        let mut numbers = Vec::with_capacity(7);
        for entry in page_window(current, pages) {
            match entry {
                PageEntry::Page(target) => {
                    let mut button = Button {
                        variant: if target == current {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Secondary
                        },
                        ..Button::label((target + 1).to_string())
                            .key(numbers_key.scoped(target as u64))
                    }
                    .on_click(move |cx| page.set(cx, target))
                    .style(ViewStyle {
                        width: if target < 99 { 32.0 } else { 0.0 },
                        ..default()
                    });
                    button.node.semantics.current = u32::from(target == current);
                    numbers.push(button);
                }
                PageEntry::Gap(before) => numbers.push(
                    Text::new("…")
                        .key(numbers_key.scoped(abi::key("gap")).scoped(before as u64))
                        .style(ViewStyle {
                            width: 24.0,
                            ..default()
                        }),
                ),
            }
        }
        view.node.key = self.key.scoped(abi::key("controls")).0;
        let mut navigation = Column {
            padding: 0.0,
            gap: 8.0,
            ..default()
        }
        .key(self.key)
        .children(vec![
            Row {
                padding: 0.0,
                gap: 4.0,
                ..default()
            }
            .key(numbers_key)
            .children(numbers),
            view,
        ]);
        navigation.node.semantics.role = abi::semantics::NAVIGATION;
        Ok(navigation)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PageEntry {
    Page(i64),
    Gap(i64),
}

// At most five page buttons and two gaps, independent of the collection size.
fn page_window(current: i64, pages: i64) -> Vec<PageEntry> {
    let last = pages - 1;
    let current = current.clamp(0, last);
    let mut targets = vec![
        0,
        current.saturating_sub(1).max(0),
        current,
        current.saturating_add(1).min(last),
        last,
    ];
    targets.sort_unstable();
    targets.dedup();
    let mut entries = Vec::with_capacity(7);
    let mut previous = None;
    for target in targets {
        if previous.is_some_and(|previous| target - previous > 1) {
            entries.push(PageEntry::Gap(target));
        }
        entries.push(PageEntry::Page(target));
        previous = Some(target);
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn page_window_is_bounded_ordered_and_keeps_boundaries_and_current() {
        for pages in [1, 2, 3, 10, 1000, i64::MAX] {
            for current in [0, 1.min(pages - 1), pages / 2, pages - 1] {
                let entries = page_window(current, pages);
                assert!(entries.len() <= 7);
                let targets: Vec<_> = entries
                    .iter()
                    .filter_map(|entry| match entry {
                        PageEntry::Page(page) => Some(*page),
                        _ => None,
                    })
                    .collect();
                assert_eq!(targets.first(), Some(&0));
                assert_eq!(targets.last(), Some(&(pages - 1)));
                assert!(targets.contains(&current));
                assert!(targets.windows(2).all(|pair| pair[0] < pair[1]));
                assert!(targets.iter().all(|page| (0..pages).contains(page)));
            }
        }
    }
}
