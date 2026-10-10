#![cfg(feature = "macros")]
use blockui::{Column, Row, Text, Ui, View};

#[blockui::view]
fn automatic_view(tree: &Ui) -> View {
    #[node]
    {
        Column::default();
        for label in ["one", "skip", "two"] {
            #[node]
            {
                Row::default();
                if label == "skip" {
                    continue;
                }
                #[node]
                Text::new(label);
            }
        }
        #[node]
        Text::new("tail");
    }
}

#[test]
fn automatic_builder_finishes_after_nested_guards_close() {
    // finish() rejects open scopes and multiple roots. In particular, continue
    // must drop each loop's guard before the next sibling is constructed.
    let _view = automatic_view();
}

#[blockui::view]
fn fallible_view(reject: bool, tree: &Ui) -> blockui::Result<View> {
    #[node]
    {
        Column::default();
        #[node]
        Text::new("before failure");
        if reject {
            Err(blockui::Error::Overflow)?;
        }
        #[node]
        Text::new("after failure");
    }
}

#[test]
fn fallible_construction_propagates_error_and_closes_partial_scopes() {
    assert!(matches!(fallible_view(true), Err(blockui::Error::Overflow)));
    assert!(fallible_view(false).is_ok());
}

// The outer attribute must leave this function's tags for the inner attribute.
#[blockui::view]
fn nested_view(outer: &Ui) -> View {
    #[blockui::view]
    fn child(inner: &Ui) -> View {
        #[node]
        Text::new("inner");
    }
    #[node]
    child();
}

#[test]
fn nested_views_use_their_own_builder() {
    let _view = nested_view();
}

type ViewOutcome = actor_result::Result<View>;
mod actor_result {
    pub type Result<T> = std::result::Result<T, blockui::Error>;
}
#[blockui::view]
fn aliased_result(empty: bool, tree: &Ui) -> ViewOutcome {
    if !empty {
        #[node]
        Text::new("root");
    }
}

#[test]
fn aliases_and_construction_errors_follow_the_return_type() {
    assert!(aliased_result(false).is_ok());
    assert!(matches!(
        aliased_result(true),
        Err(blockui::Error::InvalidView(_))
    ));
}
