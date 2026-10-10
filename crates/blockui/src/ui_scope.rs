//! Ordinary Rust construction: a guard closes each parent scope on block exit.
use crate::{IntoView, View};
use std::cell::RefCell;

/// Build a view with ordinary Rust blocks rather than a tree macro or nested
/// callbacks. State and events use the same runtime as `#[node]`.
///
/// ```
/// use blockui::{Column, Text, Ui};
/// let ui = Ui::new();
/// {
///     let _column = ui.scope(Column::default());
///     ui.add(Text::new("Hello"));
/// }
/// let view = ui.finish();
/// ```
/// Call [`Ui::finish`] or [`Ui::try_finish`] to obtain the completed view.
/// Discarding a builder expression produces an `unused_must_use` warning;
/// this does not detect a bound local that is later dropped without finishing.
#[derive(Default)]
#[must_use = "finish the UI builder with finish() or try_finish() to obtain the view"]
pub struct Ui {
    tree: RefCell<Tree>,
}
#[derive(Default)]
struct Tree {
    frames: Vec<View>,
    roots: Vec<View>,
    invalid: bool,
}
impl Tree {
    fn append(&mut self, view: View) {
        if let Some(parent) = self.frames.last_mut() {
            parent.children.push(view);
        } else {
            self.roots.push(view);
        }
    }
}
impl Ui {
    pub fn new() -> Self {
        Self::default()
    }
    /// Append a control or component to the current parent.
    pub fn add(&self, widget: impl IntoView) {
        // Convert before borrowing: component projection may itself build views.
        self.append_view(widget.into_view());
    }
    fn append_view(&self, view: View) {
        self.tree.borrow_mut().append(view);
    }
    /// Open a parent. Bind the guard to a local until the block exits; an
    /// unbound temporary closes immediately. Existing children remain first.
    pub fn scope(&self, parent: impl IntoView) -> UiScope<'_> {
        self.open_view(parent.into_view())
    }
    fn open_view(&self, view: View) -> UiScope<'_> {
        let mut tree = self.tree.borrow_mut();
        tree.frames.push(view);
        UiScope {
            ui: self,
            depth: tree.frames.len(),
        }
    }
    /// Finish one root after all scopes have closed. Incorrect nesting or root
    /// counts are programming errors. This does not wrap an ordinary view in Result.
    #[track_caller]
    pub fn finish(self) -> View {
        self.try_finish().expect("invalid UI construction")
    }
    /// Finish construction without panicking on invalid nesting or root counts.
    pub fn try_finish(self) -> crate::Result<View> {
        let mut tree = self.tree.into_inner();
        if tree.invalid {
            return Err(crate::Error::InvalidView(
                "UI scopes closed out of order; close each child scope before its parent",
            ));
        }
        if !tree.frames.is_empty() {
            return Err(crate::Error::InvalidView(
                "UI scope is still open; drop its guard before finishing the view",
            ));
        }
        if tree.roots.is_empty() {
            return Err(crate::Error::InvalidView(
                "UI has no root; add a root with #[node], Ui::add or Ui::scope",
            ));
        }
        if tree.roots.len() > 1 {
            return Err(crate::Error::InvalidView(
                "UI has multiple roots; wrap sibling nodes in one parent container",
            ));
        }
        Ok(tree.roots.pop().unwrap())
    }
}

/// Scope closes on normal return, `?`, loop exit or unwinding. It retains only
/// the builder, never a rendering Cx or durable state value.
#[must_use = "bind the scope guard to a local so the parent stays open"]
pub struct UiScope<'ui> {
    ui: &'ui Ui,
    depth: usize,
}
impl Drop for UiScope<'_> {
    fn drop(&mut self) {
        let mut tree = self.ui.tree.borrow_mut();
        if tree.invalid || tree.frames.len() != self.depth {
            // Do not panic inside Drop (in particular during an existing unwind).
            tree.invalid = true;
            return;
        }
        let mut parent = tree.frames.pop().unwrap();
        parent.finish_children();
        tree.append(parent);
    }
}

/// Return-type adaptation for #[view], including ordinary Rust type aliases.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "unsupported UI builder return type `{Self}`",
    note = "#[view] builders return View or Result<View, E>, where E: From<blockui::Error>; type aliases must resolve to one of these types"
)]
pub trait FinishView: Sized {
    #[track_caller]
    fn finish_view(ui: Ui) -> Self;
}
impl FinishView for View {
    fn finish_view(ui: Ui) -> Self {
        ui.finish()
    }
}
impl<E: From<crate::Error>> FinishView for std::result::Result<View, E> {
    fn finish_view(ui: Ui) -> Self {
        ui.try_finish().map_err(E::from)
    }
}
#[doc(hidden)]
#[track_caller]
pub fn finish_view<T: FinishView>(ui: Ui) -> T {
    T::finish_view(ui)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Column, Row, Text};
    #[test]
    fn nested_scopes_keep_existing_children_and_control_flow() {
        let ui = Ui::new();
        {
            let _root =
                ui.scope(Column::default().children(vec![Text::new("heading").into_view()]));
            for text in ["one", "skip", "two"] {
                let _row = ui.scope(Row::default());
                if text == "skip" {
                    continue;
                }
                ui.add(Text::new(text));
            }
        }
        let view = ui.finish();
        assert_eq!(view.children.len(), 4);
        assert_eq!(view.children[1].children.len(), 1);
        assert!(view.children[2].children.is_empty());
        assert_eq!(view.children[3].children.len(), 1);
    }
    #[test]
    fn invalid_order_is_reported_at_finish_without_panicking_in_drop() {
        let ui = Ui::new();
        let outer = ui.scope(Column::default());
        let inner = ui.scope(Row::default());
        drop(outer);
        drop(inner);
        let error = ui.try_finish().err().expect("invalid nesting is rejected");
        assert!(
            error
                .to_string()
                .contains("close each child scope before its parent")
        );
    }
    #[test]
    fn forgotten_guard_reports_an_open_scope() {
        let ui = Ui::new();
        std::mem::forget(ui.scope(Column::default()));
        let error = ui.try_finish().err().expect("open scope is rejected");
        assert!(
            error
                .to_string()
                .contains("drop its guard before finishing")
        );
    }
    #[cfg(feature = "macros")]
    #[test]
    fn fallible_view_reports_missing_and_multiple_roots() {
        #[crate::view]
        fn empty(ui: &Ui) -> crate::Result<View> {
            let _ = ui;
        }
        #[crate::view]
        fn siblings(ui: &Ui) -> crate::Result<View> {
            #[node]
            Text::new("first");
            #[node]
            Text::new("second");
        }
        let missing = empty().err().expect("missing root is rejected");
        assert!(missing.to_string().contains("UI has no root"));
        let multiple = siblings().err().expect("multiple roots are rejected");
        assert!(
            multiple
                .to_string()
                .contains("wrap sibling nodes in one parent container")
        );
    }
}
