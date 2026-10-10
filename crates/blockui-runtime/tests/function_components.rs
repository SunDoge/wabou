#![cfg(feature = "testing")]
use blockui::*;

#[component]
fn Counter(#[state(default = 3)] count: State<i64>) -> View {
    let increment = Button::label("Increment").on_click(move |cx| count.update(cx, |n| Ok(n + 1)));
    {
        let __blockui_tree = ::blockui::Ui::new();
        {
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(count));
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(increment)
                        .__local_identity(stringify!(increment)),
                );
            }
        }
        __blockui_tree.finish()
    }
}
#[app]
fn Pair(left: Counter, right: Counter) -> View {
    {
        let __blockui_tree = ::blockui::Ui::new();
        {
            {
                let _scope_0 = __blockui_tree.scope(Row::default());
                __blockui_tree.add(left.view());
                __blockui_tree.add(right.view());
            }
        }
        __blockui_tree.finish()
    }
}
#[test]
fn function_local_state_and_captured_events_are_independent_and_survive_replacement()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Pair>()?;
    let pair = Pair::new();
    assert_ne!(pair.left.count.key(), pair.right.count.key());
    let buttons: Vec<_> = actor
        .view
        .order
        .iter()
        .filter(|key| actor.view.bindings[key].node.kind == abi::BUTTON)
        .copied()
        .collect();
    assert_eq!(buttons.len(), 2);
    actor.dispatch_widget(buttons[0])?;
    assert_eq!(actor.integer_by_key(pair.left.count.key()), 4);
    assert_eq!(actor.integer_by_key(pair.right.count.key()), 3);
    let slot = actor.slot_address_by_key(pair.left.count.key());
    actor.replace_for_test::<Pair>()?;
    actor.dispatch_widget(buttons[1])?;
    assert_eq!(actor.integer_by_key(pair.left.count.key()), 4);
    assert_eq!(actor.integer_by_key(pair.right.count.key()), 4);
    assert_eq!(actor.slot_address_by_key(pair.left.count.key()), slot);
    Ok(())
}

#[component]
fn Heading() -> View {
    Text::new("Heading").into_view()
}
#[app]
fn WithContext(cx: &Cx<'_>, count: State<i64>, heading: Heading) -> Result<View> {
    let value = count.get(cx)?;
    Ok({
        let __blockui_tree = ::blockui::Ui::new();
        {
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(heading.view());
                __blockui_tree.add(Text::new(format!("Value: {value}")));
            }
        }
        __blockui_tree.finish()
    })
}
#[test]
fn stateless_components_and_fallible_context_functions_use_the_same_runtime()
-> std::result::Result<(), String> {
    let actor = blockui_runtime::Actor::for_test::<WithContext>()?;
    assert!(
        actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "Heading")
    );
    assert!(
        actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "Value: 0")
    );
    Ok(())
}

#[app]
fn WithHandles(count: State<i64>, clock: Task, document: Resource<File>) -> View {
    let _ = (clock.key(), document.key());
    {
        let __blockui_tree = ::blockui::Ui::new();
        {
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(count));
            }
        }
        __blockui_tree.finish()
    }
}
#[test]
fn function_components_pass_only_task_and_resource_identity_handles()
-> std::result::Result<(), String> {
    let _actor = blockui_runtime::Actor::for_test::<WithHandles>()?;
    Ok(())
}

#[component]
fn caption() -> View {
    Text::new("lowercase function").into_view()
}
#[app]
fn lowercase_root(label: caption) -> View {
    {
        let __blockui_tree = ::blockui::Ui::new();
        {
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(label.view());
            }
        }
        __blockui_tree.finish()
    }
}
#[test]
fn normal_rust_function_names_are_supported() -> std::result::Result<(), String> {
    let _actor = blockui_runtime::Actor::for_test::<lowercase_root>()?;
    Ok(())
}

#[app]
fn Reordered(right: Counter, left: Counter) -> View {
    {
        let __blockui_tree = ::blockui::Ui::new();
        {
            {
                let _scope_0 = __blockui_tree.scope(Row::default());
                __blockui_tree.add(right.view());
                __blockui_tree.add(left.view());
            }
        }
        __blockui_tree.finish()
    }
}
#[test]
fn reordering_parameters_and_children_preserves_state_and_widget_identities()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Pair>()?;
    let pair = Pair::new();
    let old_schema = actor.schema_version;
    let old_state = actor.state_address();
    let slot = actor.slot_address_by_key(pair.left.count.key());
    let buttons: std::collections::BTreeSet<_> = actor
        .view
        .bindings
        .values()
        .filter(|binding| binding.node.kind == abi::BUTTON)
        .map(|binding| binding.node.key)
        .collect();
    actor.replace_for_test::<Reordered>()?;
    assert_eq!(actor.schema_version, old_schema);
    assert_eq!(actor.state_address(), old_state);
    assert_eq!(actor.slot_address_by_key(pair.left.count.key()), slot);
    let current: std::collections::BTreeSet<_> = actor
        .view
        .bindings
        .values()
        .filter(|binding| binding.node.kind == abi::BUTTON)
        .map(|binding| binding.node.key)
        .collect();
    assert_eq!(current, buttons);
    Ok(())
}
