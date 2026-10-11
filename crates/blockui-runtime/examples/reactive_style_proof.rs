//! Real DLL proof, driven by scripts/verify_reactive_style.py.
use blockui::Key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("expected base, replacement, rejected and snapshot directory".into());
    }
    let editor = Key::named("proof.editor").0;
    let label = Key::named("proof.label").0;
    let other = Key::named("proof.other").0;
    let caption = Key::named("proof.caption").0;
    let panel = Key::named("proof.scope").0;
    let local = Key::named("proof.captured.scope")
        .scoped(Key::named("label").0)
        .0;
    let syntax =
        ["proof.syntax.plain.value", "proof.syntax.bound.value"].map(|key| Key::named(key).0);
    let prefixed = ["proof.prefixed.state", "proof.prefixed.memo"].map(|name| Key::named(name).0);
    let formatted = Key::named("proof.computed.formatted").0;
    let mut actor = Actor::new(811, Path::new(&paths[3]))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.edit_text(editor, "first")?;
    assert_eq!(actor.text("proof.status"), "base:first");
    assert_eq!(actor.view.bindings[&syntax[0]].text, "plain=18");
    assert_eq!(actor.view.bindings[&syntax[1]].text, "bound syntax=18");
    assert_eq!(
        actor.view.bindings[&formatted].text,
        "base mapped: bound syntax=18"
    );
    assert_eq!(actor.view.bindings[&prefixed[0]].text, "base prefix: 18");
    assert_eq!(
        actor.view.bindings[&prefixed[1]].text,
        "base memo: bound syntax=18"
    );
    let state = actor.state_address();
    let slot = actor.slot_address("proof.left");
    let initial = actor.view.bindings[&label].clone();
    assert_eq!(actor.view.bindings[&local].text, "captured subtree=18");
    actor.view.renderer.get_mut(&local).unwrap().selection = (2, 5);
    assert_eq!(actor.view.bindings[&caption].text, "bound=18");
    assert_eq!(actor.view.bindings[&panel].node.presentation.width, 180.0);
    assert_eq!(actor.view.bindings[&panel].text, "scroll root=18");
    actor.view.renderer.get_mut(&label).unwrap().focus = true;
    actor.view.renderer.get_mut(&label).unwrap().selection = (1, 3);
    let dispatch = |actor: &mut Actor, action: &str, value| {
        actor.dispatch(Message {
            action: blockui::abi::key(action),
            value,
        })
    };
    dispatch(&mut actor, "proof.row.change", 0)?;
    let row_list = Key::named("proof.row.list").0;
    let row_a = Key::named("proof.row.a").0;
    let row_b = Key::named("proof.row.b").0;
    let before_rows = [row_list, row_a, row_b].map(|key| actor.view.bindings[&key].evaluations);
    dispatch(&mut actor, "proof.row.change", 1)?;
    assert_eq!(actor.view.bindings[&row_a].text, "Changed");
    assert_eq!(actor.view.bindings[&row_list].evaluations, before_rows[0]);
    assert_eq!(actor.view.bindings[&row_a].evaluations, before_rows[1] + 1);
    assert_eq!(actor.view.bindings[&row_b].evaluations, before_rows[2]);
    dispatch(&mut actor, "proof.left", 20)?;
    assert_eq!(
        actor.view.bindings[&label].node.presentation.font_size,
        20.0
    );
    assert_eq!(actor.view.bindings[&label].evaluations, initial.evaluations);
    assert_eq!(actor.view.bindings[&other].evaluations, 1);
    assert_eq!(actor.view.bindings[&caption].text, "bound=20");
    assert_eq!(actor.view.bindings[&panel].node.presentation.width, 200.0);
    assert_eq!(actor.view.bindings[&panel].text, "scroll root=20");
    assert_eq!(actor.view.bindings[&local].text, "captured subtree=20");
    assert_eq!(actor.view.bindings[&syntax[0]].text, "plain=20");
    assert_eq!(actor.view.bindings[&syntax[1]].text, "bound syntax=20");
    assert_eq!(
        actor.view.bindings[&formatted].text,
        "base mapped: bound syntax=20"
    );
    for index in 0..20 {
        actor.reload(Path::new(&paths[1]))?;
        assert_eq!(
            actor.text("proof.draft"),
            if index == 0 { "first" } else { "next" }
        );
        actor.edit_text(editor, "next")?;
        assert_eq!(actor.text("proof.status"), "replacement:next");
        dispatch(&mut actor, "proof.right", 24 + index % 4)?;
        assert_eq!(
            actor.view.bindings[&label].node.presentation.font_size,
            (24 + index % 4) as f32
        );
        assert_eq!(
            actor.view.bindings[&caption].text,
            format!("bound={}", 24 + index % 4)
        );
        assert_eq!(
            actor.view.bindings[&panel].node.presentation.width,
            (24 + index % 4) as f32 * 10.0
        );
        assert_eq!(
            actor.view.bindings[&local].text,
            format!("captured subtree={}", 24 + index % 4)
        );
        assert_eq!(
            actor.view.bindings[&panel].text,
            format!("scroll root={}", 24 + index % 4)
        );
        assert_eq!(actor.view.renderer[&local].selection, (2, 5));
        assert_eq!(
            actor.view.bindings[&syntax[0]].text,
            format!("plain={}", 24 + index % 4)
        );
        assert_eq!(
            actor.view.bindings[&syntax[1]].text,
            format!("bound syntax={}", 24 + index % 4)
        );
        assert_eq!(
            actor.view.bindings[&formatted].text,
            format!("replacement mapped: bound syntax={}", 24 + index % 4)
        );
        assert_eq!(
            actor.view.bindings[&caption].node.presentation.font_size,
            (24 + index % 4) as f32
        );
        assert_eq!(
            actor.view.bindings[&formatted].node.presentation.font_size,
            (24 + index % 4) as f32
        );
        assert_eq!(
            actor.view.bindings[&prefixed[0]].text,
            format!("new prefix: {}", 24 + index % 4)
        );
        assert_eq!(
            actor.view.bindings[&prefixed[1]].text,
            format!("new memo: bound syntax={}", 24 + index % 4)
        );
        assert_eq!(actor.artifact_count(), 1);
    }
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("proof.left"), slot);
    assert!(actor.view.renderer[&label].focus);
    assert_eq!(actor.view.renderer[&label].selection, (1, 3));
    let syntax_before = syntax.map(|key| actor.view.bindings[&key].clone());
    let style_evaluations = actor.view.bindings[&label].style_evaluations;
    dispatch(&mut actor, "proof.left", 21)?;
    assert_eq!(
        actor.view.bindings[&label].style_evaluations,
        style_evaluations
    );
    for (key, before) in syntax.iter().zip(&syntax_before) {
        assert_eq!(actor.view.bindings[key].text, before.text);
        assert_eq!(actor.view.bindings[key].evaluations, before.evaluations);
    }
    let value = actor.integer("proof.right");
    let old_prefixed = prefixed.map(|key| actor.view.bindings[&key].clone());
    let old_formatted = actor.view.bindings[&formatted].clone();
    let old_caption = actor.view.bindings[&caption].clone();
    let old_panel = actor.view.bindings[&panel].clone();
    let old = actor.view.bindings[&label].clone();
    let old_local = actor.view.bindings[&local].clone();
    let root_evaluations = actor.view.root_evaluations();
    let style_error = dispatch(&mut actor, "proof.right", 93).unwrap_err();
    assert!(
        style_error.contains(&format!("style binding {label:#018x} evaluation failed")),
        "{style_error}"
    );
    assert!(
        style_error.contains("reject captured style"),
        "{style_error}"
    );
    assert_eq!(actor.integer("proof.right"), value);
    assert_eq!(actor.view.root_evaluations(), root_evaluations);
    assert_eq!(actor.view.bindings[&label].node, old.node);
    assert_eq!(
        actor.view.bindings[&label].style_evaluations,
        old.style_evaluations
    );
    let error = dispatch(&mut actor, "proof.right", 94).unwrap_err();
    assert!(
        error.contains(&format!("text binding {formatted:#018x} evaluation failed")),
        "{error}"
    );
    assert!(
        error.contains("reject fallible computed formatter"),
        "{error}"
    );
    assert_eq!(actor.integer("proof.right"), value);
    assert_eq!(actor.view.bindings[&formatted].text, old_formatted.text);
    assert_eq!(
        actor.view.bindings[&formatted].evaluations,
        old_formatted.evaluations
    );
    assert!(dispatch(&mut actor, "proof.right", 95).is_err());
    assert_eq!(actor.integer("proof.right"), value);
    for (key, before) in prefixed.iter().zip(&old_prefixed) {
        assert_eq!(actor.view.bindings[key].text, before.text);
        assert_eq!(actor.view.bindings[key].evaluations, before.evaluations);
    }
    assert_eq!(actor.view.bindings[&formatted].text, old_formatted.text);
    assert_eq!(actor.view.bindings[&formatted].node, old_formatted.node);
    assert_eq!(
        actor.view.bindings[&formatted].evaluations,
        old_formatted.evaluations
    );
    let error = dispatch(&mut actor, "proof.right", 96).unwrap_err();
    assert!(
        error.contains(&format!("text binding {panel:#018x} evaluation failed")),
        "{error}"
    );
    assert!(error.contains("reject root caption"), "{error}");
    assert_eq!(actor.integer("proof.right"), value);
    assert_eq!(actor.view.bindings[&panel].text, old_panel.text);
    assert_eq!(actor.view.bindings[&caption].text, old_caption.text);
    let error = dispatch(&mut actor, "proof.right", 98).unwrap_err();
    let captured_scope = Key::named("proof.captured.scope").0;
    assert!(
        error.contains(&format!("scope {captured_scope:#018x} projection failed")),
        "{error}"
    );
    assert!(error.contains("reject captured subtree"), "{error}");
    assert_eq!(actor.integer("proof.right"), value);
    assert_eq!(actor.view.bindings[&local].text, old_local.text);
    let invalid_style = dispatch(&mut actor, "proof.right", 97).unwrap_err();
    for fragment in [
        "invalid widget presentation",
        "font_size",
        "0..=96",
        "got 97",
    ] {
        assert!(invalid_style.contains(fragment), "{invalid_style}");
    }
    assert!(
        [label, caption, formatted]
            .iter()
            .any(|key| invalid_style.contains(&format!("{key:#018x}"))),
        "{invalid_style}"
    );
    assert_eq!(actor.integer("proof.right"), value);
    assert_eq!(actor.view.bindings[&caption].text, old_caption.text);
    assert_eq!(actor.view.bindings[&panel].node, old_panel.node);
    assert_eq!(actor.view.bindings[&label].node, old.node);
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.view.bindings[&label].node, old.node);
    assert_eq!(actor.view.bindings[&caption].text, old_caption.text);
    assert_eq!(actor.view.bindings[&panel].node, old_panel.node);
    for (key, before) in syntax.iter().zip(&syntax_before) {
        assert_eq!(actor.view.bindings[key].text, before.text);
    }
    assert_eq!(actor.view.bindings[&formatted].text, old_formatted.text);
    assert_eq!(actor.view.bindings[&formatted].node, old_formatted.node);
    actor.edit_text(editor, "after rollback")?;
    assert_eq!(actor.text("proof.status"), "replacement:after rollback");
    assert_eq!(actor.view.bindings[&panel].text, old_panel.text);
    dispatch(&mut actor, "proof.right", 32)?;
    assert_eq!(actor.view.bindings[&prefixed[0]].text, "new prefix: 32");
    assert_eq!(
        actor.view.bindings[&prefixed[1]].text,
        "new memo: bound syntax=32"
    );
    assert_eq!(actor.view.bindings[&caption].text, "bound=32");
    assert_eq!(
        actor.view.bindings[&caption].node.presentation.font_size,
        32.0
    );
    assert_eq!(
        actor.view.bindings[&formatted].node.presentation.font_size,
        32.0
    );
    assert_eq!(
        actor.view.bindings[&formatted].text,
        "replacement mapped: bound syntax=32"
    );
    assert_eq!(actor.view.bindings[&local].text, "captured subtree=32");
    assert_eq!(actor.view.bindings[&panel].node.presentation.width, 320.0);
    assert_eq!(
        actor.view.bindings[&label].node.presentation.font_size,
        32.0
    );
    assert_eq!(actor.view.bindings[&row_a].text, "Changed");
    dispatch(&mut actor, "proof.row.change", 2)?;
    assert_eq!(
        actor.view.bindings[&Key::named("proof.row.c").0].text,
        "Carol"
    );
    dispatch(&mut actor, "proof.row.change", 3)?;
    assert!(!actor.view.bindings.contains_key(&row_b));
    dispatch(&mut actor, "proof.visible", 0)?;
    assert!(!actor.view.bindings.contains_key(&local));
    dispatch(&mut actor, "proof.visible", 1)?;
    assert_eq!(actor.view.bindings[&local].text, "captured subtree=32");
    assert_eq!(actor.artifact_count(), 1);
    let maps = std::fs::read_to_string("/proc/self/maps").map_err(|e| e.to_string())?;
    let mapped: std::collections::BTreeSet<_> = maps
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .filter(|path| path.contains("actor-811-") && path.contains("impl-"))
        .collect();
    assert_eq!(mapped.len(), 1);
    println!(
        "PASS real style DLL: captured text/style/conditional structural scopes and scope roots, plain/bound scope UI macros, independent observers/memoized record membership, keyed Field callbacks, 20 reloads/one mapping, native state retention, root text/invalid style and migration rollback"
    );
    Ok(())
}
