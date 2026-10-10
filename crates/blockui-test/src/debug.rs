use super::Dependency;
use super::{Locator, Role, Screen, abi, role};
use std::collections::BTreeSet;
use std::fmt::Write;

fn dependencies(output: &mut String, observer: &str, values: &BTreeSet<Dependency>) {
    if values.is_empty() {
        return;
    }
    write!(output, " {observer}-deps=[").unwrap();
    for (index, dependency) in values.iter().take(4).enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        let (kind, key) = match dependency {
            Dependency::State(key) => ("state", key),
            Dependency::Computed(key) => ("computed", key),
            Dependency::MapEntry(key) => ("map-entry", key),
        };
        write!(output, "{kind}:{key:#018x}").unwrap();
    }
    if values.len() > 4 {
        write!(output, ", … +{}", values.len() - 4).unwrap();
    }
    output.push(']');
}

pub(super) fn tree(screen: &Screen, root: Option<Locator>) -> String {
    let view = screen.actor.projected_view();
    let mut output = String::new();
    let mut count = 0;
    for key in &view.order {
        if !screen.visible(*key) || !screen.in_subtree(root, *key) {
            continue;
        }
        if count == 128 {
            output.push_str("… (remaining nodes omitted)\n");
            break;
        }
        count += 1;
        let binding = &view.bindings[key];
        let mut depth = 0;
        let mut ancestor = *key;
        while root != Some(Locator(ancestor)) {
            let Some(parent) = view
                .bindings
                .get(&ancestor)
                .and_then(|node| view.bindings.get(&node.node.parent))
            else {
                break;
            };
            depth += 1;
            ancestor = parent.node.key;
        }
        let kind = role(binding)
            .map(|role| format!("{role:?}"))
            .unwrap_or_else(|| match binding.node.kind {
                abi::COLUMN => "Column".into(),
                abi::ROW => "Row".into(),
                abi::SCROLL => "Scroll".into(),
                abi::LABEL => "Text".into(),
                other => format!("Node({other})"),
            });
        let label = screen.accessible_name(binding);
        write!(
            output,
            "{}{kind} {:?} [key {key:#018x}, text={}, style={}]",
            "  ".repeat(depth.min(16)),
            Screen::query_preview(label),
            binding.evaluations,
            binding.style_evaluations
        )
        .unwrap();
        if label != binding.text {
            write!(output, " text={:?}", Screen::query_preview(&binding.text)).unwrap();
        }
        let node = binding.node;
        dependencies(&mut output, "text", &binding.text_dependencies);
        dependencies(&mut output, "style", &binding.style_dependencies);
        if node.parent == 0 {
            dependencies(&mut output, "root", &view.root_dependencies);
            write!(output, " root={}", view.root_evaluations()).unwrap();
        }
        if let Some(scope) = view.scope_dependencies.get(key) {
            dependencies(&mut output, "scope", &scope.dependencies);
            write!(output, " scope={}", scope.evaluations).unwrap();
        }
        if node.disabled != 0 || node.semantics.locked != 0 {
            output.push_str(" disabled=true");
        }
        if let Some(checked) = screen.actor.checkbox_value(*key) {
            write!(output, " checked={checked}").unwrap();
        }
        if matches!(role(binding), Some(Role::Tab | Role::Radio)) {
            write!(output, " selected={}", node.semantics.selected != 0).unwrap();
        }
        match node.semantics.expanded {
            1 => output.push_str(" expanded=false"),
            2 => output.push_str(" expanded=true"),
            _ => {}
        }
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependency_preview_bounds_large_sets_and_distinguishes_observers() {
        let mut output = String::new();
        let values = (1..=8).map(Dependency::State).collect();
        dependencies(&mut output, "scope", &values);
        assert!(output.contains("state:0x0000000000000004"));
        assert!(!output.contains("state:0x0000000000000005"));
        assert!(output.ends_with(", … +4]"));
        output.clear();
        dependencies(
            &mut output,
            "text",
            &BTreeSet::from([Dependency::Computed(7), Dependency::MapEntry(9)]),
        );
        assert!(output.contains("text-deps=[computed:0x0000000000000007"));
        assert!(output.contains("map-entry:0x0000000000000009"));
    }
}
