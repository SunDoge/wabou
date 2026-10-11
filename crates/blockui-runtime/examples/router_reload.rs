use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn send(actor: &mut Actor, name: &str, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(name),
        value,
    })
}
fn location(actor: &Actor) -> &str {
    &actor.view.bindings[&key("routes.location")].text
}
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 6 {
        return Err(
            "router_reload <base> <replacement> <reject> <missing> <evolved> <removed>".into(),
        );
    }
    let mut encoded = Actor::new(199, Path::new(".runtime"))?;
    encoded.reload(Path::new(&paths[0]))?;
    let path = "/search/folder%2Fname%20%F0%9F%A6%80%20%252F%3F%23";
    encoded.dispatch_text_request(key("routes.navigate"), path.into())?;
    assert_eq!(location(&encoded), path);
    assert_eq!(
        encoded.view.bindings[&key("routes.search.term")].text,
        "Search: folder/name 🦀 %2F?#"
    );
    let state = encoded.state_address();
    let history = encoded.text_list("routes.paths").to_vec();
    let history_buffer = encoded.text_list("routes.paths").as_ptr();
    encoded.reload(Path::new(&paths[1]))?;
    assert_eq!(encoded.state_address(), state);
    assert_eq!(encoded.text_list("routes.paths"), history);
    assert_eq!(encoded.text_list("routes.paths").as_ptr(), history_buffer);
    for invalid in [
        "/search/folder%2fname",
        "/search/%41",
        "/search/%FF",
        "/search/%00",
        "/search/name?query",
        "/search/文档",
    ] {
        assert!(
            encoded
                .dispatch_text_request(key("routes.navigate"), invalid.into())
                .is_err()
        );
        assert_eq!(encoded.text_list("routes.paths"), history);
        assert_eq!(location(&encoded), path);
    }
    send(&mut encoded, "routes.back", 0)?;
    assert_eq!(location(&encoded), "/");
    send(&mut encoded, "routes.forward", 0)?;
    assert_eq!(location(&encoded), path);
    assert_eq!(
        encoded.view.bindings[&key("routes.search.term")].text,
        "Search: folder/name 🦀 %2F?#"
    );
    println!(
        "PASS encoded route parameters decode once and preserve canonical history allocation through real DLL replacement; invalid encodings roll back and history travel restores decoded text"
    );
    let encoded_schema = encoded.schema_version;
    encoded.reload(Path::new(&paths[4]))?;
    assert_ne!(encoded.schema_version, encoded_schema);
    assert_eq!(encoded.state_address(), state);
    assert_eq!(encoded.text_list("routes.paths"), history);
    assert_eq!(location(&encoded), path);
    assert_eq!(
        encoded.view.bindings[&key("routes.search.term")].text,
        "Search: folder/name 🦀 %2F?#"
    );
    println!(
        "PASS generated route schema changes without a manual version slot; migration preserves unaffected encoded routes"
    );
    let mut replacing = Actor::new(200, Path::new(".runtime"))?;
    replacing.reload(Path::new(&paths[0]))?;
    send(&mut replacing, "routes.open", 42)?;
    replacing.edit_text(key("routes.document.42.notes"), "Keep this draft")?;
    let replace_state = replacing.state_address();
    let before_replace = replacing.text_list("routes.paths").to_vec();
    send(&mut replacing, "routes.request_replace", 99)?;
    assert_eq!(
        replacing.text("routes.pending_navigation"),
        "replace:/documents/99"
    );
    replacing.reload(Path::new(&paths[1]))?;
    send(&mut replacing, "routes.cancel_navigation", 0)?;
    assert_eq!(replacing.text_list("routes.paths"), before_replace);
    assert_eq!(replacing.text("routes.notes"), "Keep this draft");
    send(&mut replacing, "routes.request_replace", 73)?;
    for candidate in &paths[2..4] {
        assert!(replacing.reload(Path::new(candidate)).is_err());
        assert_eq!(
            replacing.text("routes.pending_navigation"),
            "replace:/documents/73"
        );
        assert_eq!(replacing.text_list("routes.paths"), before_replace);
    }
    replacing.reload(Path::new(&paths[4]))?;
    assert_eq!(replacing.state_address(), replace_state);
    assert_eq!(
        replacing.text("routes.pending_navigation"),
        "replace:/docs/73"
    );
    send(&mut replacing, "routes.confirm_navigation", 0)?;
    assert_eq!(replacing.text_list("routes.paths"), &["1", "/", "/docs/73"]);
    assert!(replacing.text("routes.pending_navigation").is_empty());
    assert_eq!(
        replacing.text_list("routes.document_notes"),
        &["42", "Keep this draft"]
    );
    send(&mut replacing, "routes.back", 0)?;
    assert_eq!(location(&replacing), "/");
    println!(
        "PASS pending replace retains intent through real DLL replacement and path migration; cancellation/rejected upgrades preserve draft and history, confirmation replaces rather than pushes"
    );
    for (action, first, second) in [("routes.guarded", 42, 73), ("pages.guarded", 1, 2)] {
        let mut guarded = Actor::new(93, Path::new(".runtime"))?;
        guarded.reload(Path::new(&paths[0]))?;
        send(&mut guarded, action, -1)?;
        send(&mut guarded, action, -2)?;
        assert_eq!(guarded.integer("routes.guard_calls"), 0);
        send(&mut guarded, action, first)?;
        send(&mut guarded, action, second)?;
        send(&mut guarded, action, -1)?;
        send(&mut guarded, "routes.dirty", 1)?;
        let history = guarded.text_list("routes.paths").to_vec();
        let page = guarded.integer("routes.page");
        let back = guarded.list("routes.page_back").to_vec();
        let forward = guarded.list("routes.page_forward").to_vec();
        guarded.reload(Path::new(&paths[1]))?;
        for request in [second, -1, -2, -3] {
            let calls = guarded.integer("routes.guard_calls");
            send(&mut guarded, action, request)?;
            assert_eq!(guarded.integer("routes.guard_calls"), calls + 1);
            assert_eq!(guarded.text_list("routes.paths"), history);
            assert_eq!(guarded.integer("routes.page"), page);
            assert_eq!(guarded.list("routes.page_back"), back);
            assert_eq!(guarded.list("routes.page_forward"), forward);
        }
        if action == "routes.guarded" {
            let calls = guarded.integer("routes.guard_calls");
            assert!(send(&mut guarded, action, -9).is_err());
            assert_eq!(guarded.integer("routes.guard_calls"), calls);
            assert_eq!(guarded.text_list("routes.paths"), history);
        }
        send(&mut guarded, "routes.dirty", 0)?;
        for dirty in [0, 1] {
            send(&mut guarded, "routes.dirty", dirty)?;
            for mutation in [-8, -7] {
                let calls = guarded.integer("routes.guard_calls");
                let error = send(&mut guarded, action, mutation).unwrap_err();
                assert!(
                    error.contains("Navigation guard changed router state"),
                    "{error}"
                );
                assert_eq!(guarded.integer("routes.guard_calls"), calls);
                assert_eq!(guarded.text_list("routes.paths"), history);
                assert_eq!(guarded.integer("routes.page"), page);
                assert_eq!(guarded.list("routes.page_back"), back);
                assert_eq!(guarded.list("routes.page_forward"), forward);
            }
        }
        send(&mut guarded, "routes.dirty", 0)?;
        send(&mut guarded, action, -2)?;
        if action == "routes.guarded" {
            assert_eq!(location(&guarded), "/documents/73");
        } else {
            assert_eq!(guarded.integer("routes.page"), 2);
        }
    }
    println!(
        "PASS navigation guards deny push/replace/back/forward across DLL replacement; unavailable travel skips guard and guard errors roll back actor writes"
    );
    let mut queries = Actor::new(93, Path::new(".runtime"))?;
    queries.reload(Path::new(&paths[0]))?;
    for implementation in [&paths[0], &paths[1]] {
        queries.reload(Path::new(implementation))?;
        for operation in 0..20 {
            assert!(send(&mut queries, "routes.invalid_cursor", operation).is_err());
            assert!(queries.text_list("routes.paths").is_empty());
        }
        for operation in 0..12 {
            assert!(send(&mut queries, "pages.invalid_query", operation).is_err());
            assert_eq!(queries.integer("routes.page"), 0);
            assert!(queries.list("routes.page_back").is_empty());
            assert!(queries.list("routes.page_forward").is_empty());
        }
    }
    println!(
        "PASS router availability and travel reject invalid current routes or either history and roll back across DLL replacement"
    );
    println!(
        "PASS path router queries reject noncanonical and overflowing history cursors across DLL replacement with transaction rollback"
    );
    for removed_in_forward in [false, true] {
        let mut pages = Actor::new(92, Path::new(".runtime"))?;
        pages.reload(Path::new(&paths[0]))?;
        let compatible_schema = pages.schema_version;
        pages.reload(Path::new(&paths[5]))?;
        assert_eq!(pages.schema_version, compatible_schema);
        pages.reload(Path::new(&paths[0]))?;
        send(&mut pages, "pages.open", 2)?;
        if removed_in_forward {
            send(&mut pages, "pages.back", 0)?;
        } else {
            send(&mut pages, "pages.open", 1)?;
            send(&mut pages, "pages.open", 0)?;
        }
        assert_eq!(pages.integer("routes.page"), 0);
        let version = pages.implementation_version();
        let schema = pages.schema_version;
        let state = pages.state_address();
        let back = pages.list("routes.page_back").to_vec();
        let forward = pages.list("routes.page_forward").to_vec();
        let back_pointer = pages.list("routes.page_back").as_ptr();
        let forward_pointer = pages.list("routes.page_forward").as_ptr();
        assert!(pages.reload(Path::new(&paths[5])).is_err());
        assert_eq!(pages.implementation_version(), version);
        assert_eq!(pages.schema_version, schema);
        assert_eq!(pages.state_address(), state);
        assert_eq!(pages.list("routes.page_back"), back);
        assert_eq!(pages.list("routes.page_forward"), forward);
        assert_eq!(pages.list("routes.page_back").as_ptr(), back_pointer);
        assert_eq!(pages.list("routes.page_forward").as_ptr(), forward_pointer);
        assert_eq!(pages.view.bindings[&key("pages.location")].text, "0");
        if removed_in_forward {
            send(&mut pages, "pages.forward", 0)?;
        } else {
            send(&mut pages, "pages.back", 0)?;
            send(&mut pages, "pages.back", 0)?;
        }
        assert_eq!(pages.integer("routes.page"), 2);
    }
    println!(
        "PASS removed integer routes in deep back/forward history reject a schema-compatible DLL; old implementation, state and history allocations remain usable"
    );
    let mut confirmation = Actor::new(96, Path::new(".runtime"))?;
    {
        let mut external = Actor::new(97, Path::new(".runtime"))?;
        external.reload(Path::new(&paths[0]))?;
        external.dispatch_text_request(key("routes.request_path"), "/documents/42".into())?;
        external.edit_text(
            key("routes.document.42.notes"),
            "Draft from external route 🦀",
        )?;
        let history = external.text_list("routes.paths").to_vec();
        external.dispatch_text_request(key("routes.request_path"), "/documents/42".into())?;
        assert!(external.text("routes.pending_navigation").is_empty());
        assert!(external.boolean("routes.dirty"));
        assert_eq!(external.text_list("routes.paths"), history);
        external.dispatch_text_request(key("routes.request_path"), "/documents/73".into())?;
        assert_eq!(external.text_list("routes.paths"), history);
        let cursor: usize = history[0].parse().unwrap();
        assert_eq!(history[cursor + 1], "/documents/42");
        assert_eq!(external.text("routes.pending_navigation"), "/documents/73");
        for invalid in ["/documents/0073", "/documents/-1", "/unknown"] {
            assert!(
                external
                    .dispatch_text_request(key("routes.request_path"), invalid.into())
                    .is_err()
            );
            assert_eq!(external.text("routes.pending_navigation"), "/documents/73");
            assert_eq!(external.text_list("routes.paths"), history);
        }
        external.reload(Path::new(&paths[1]))?;
        assert_eq!(external.text("routes.pending_navigation"), "/documents/73");
        send(&mut external, "routes.request_open", 42)?;
        assert!(external.text("routes.pending_navigation").is_empty());
        assert!(external.boolean("routes.dirty"));
        assert_eq!(external.text_list("routes.paths"), history);
        assert_eq!(location(&external), "/documents/42");
        assert_eq!(
            external.text("routes.notes"),
            "Draft from external route 🦀"
        );
        external.dispatch_text_request(key("routes.request_path"), "/documents/73".into())?;
        send(&mut external, "routes.confirm_navigation", 0)?;
        assert_eq!(location(&external), "/documents/73");
        send(&mut external, "routes.back", 0)?;
        assert_eq!(location(&external), "/documents/42");
        assert_eq!(
            external.text("routes.notes"),
            "Draft from external route 🦀"
        );
    }
    println!(
        "PASS canonical external route requests preserve dirty drafts and history until confirmation across real DLL replacement; invalid requests preserve pending navigation"
    );
    confirmation.reload(Path::new(&paths[0]))?;
    send(&mut confirmation, "routes.dirty", 1)?;
    send(&mut confirmation, "routes.request_open", 42)?;
    assert_eq!(
        confirmation.text("routes.pending_navigation"),
        "/documents/42"
    );
    let pending_slot = confirmation.slot_address("routes.pending_navigation");
    let old_version = confirmation.implementation_version();
    for candidate in &paths[2..4] {
        assert!(confirmation.reload(Path::new(candidate)).is_err());
        assert_eq!(confirmation.implementation_version(), old_version);
        assert_eq!(
            confirmation.text("routes.pending_navigation"),
            "/documents/42"
        );
        assert_eq!(
            confirmation.slot_address("routes.pending_navigation"),
            pending_slot
        );
    }
    confirmation.reload(Path::new(&paths[4]))?;
    assert_eq!(confirmation.text("routes.pending_navigation"), "/docs/42");
    assert_eq!(
        confirmation.slot_address("routes.pending_navigation"),
        pending_slot
    );
    send(&mut confirmation, "routes.confirm_navigation", 0)?;
    assert_eq!(location(&confirmation), "/docs/42");
    assert!(confirmation.text("routes.pending_navigation").is_empty());
    println!(
        "PASS pending route confirmation requires codec migration; rejected upgrades preserve pending state and migrated confirmation runs current code"
    );
    let mut actor = Actor::new(91, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(location(&actor), "/");
    send(&mut actor, "routes.open", 42)?;
    send(&mut actor, "routes.open", 73)?;
    send(&mut actor, "routes.back", 0)?;
    assert_eq!(location(&actor), "/documents/42");
    let editor_key = key("routes.document.42.notes");
    assert!(actor.view.bindings.contains_key(&editor_key));
    assert!(
        !actor
            .view
            .bindings
            .contains_key(&key("routes.home.heading"))
    );
    actor.edit_text(editor_key, "Unsaved workspace notes 🦀")?;
    let notes_slot = actor.slot_address("routes.notes");
    let state = actor.state_address();
    let slot = actor.slot_address("routes.paths");
    let allocation = actor.text_list("routes.paths").as_ptr();
    let history = actor.text_list("routes.paths").to_vec();
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("routes.paths"), slot);
    assert_eq!(actor.text_list("routes.paths").as_ptr(), allocation);
    assert_eq!(actor.text_list("routes.paths"), history);
    assert_eq!(actor.schema_version, schema);
    assert_ne!(actor.implementation_version(), version);
    assert_eq!(location(&actor), "/documents/42");
    assert_eq!(actor.slot_address("routes.notes"), notes_slot);
    assert_eq!(actor.text("routes.notes"), "Unsaved workspace notes 🦀");
    assert_eq!(
        actor.view.bindings[&editor_key].text,
        "Unsaved workspace notes 🦀"
    );
    send(&mut actor, "routes.forward", 0)?;
    assert_eq!(location(&actor), "/documents/73");
    assert!(!actor.view.bindings.contains_key(&editor_key));
    assert_eq!(
        actor.view.bindings[&key("routes.document.73.notes")].text,
        ""
    );
    actor.edit_text(key("routes.document.73.notes"), "Document 73 draft")?;
    assert_eq!(
        actor.text_list("routes.document_notes"),
        [
            "42",
            "Unsaved workspace notes 🦀",
            "73",
            "Document 73 draft"
        ]
    );
    send(&mut actor, "routes.back", 0)?;
    assert_eq!(actor.text("routes.notes"), "Unsaved workspace notes 🦀");
    send(&mut actor, "routes.forward", 0)?;
    assert_eq!(actor.text("routes.notes"), "Document 73 draft");
    assert!(actor.boolean("routes.dirty"));
    assert_eq!(actor.list("routes.dirty_documents"), [42, 73]);
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.list("routes.dirty_documents"), [42, 73]);
    send(&mut actor, "routes.request_open", 0)?;
    assert_eq!(actor.text("routes.pending_navigation"), "/");
    send(&mut actor, "routes.confirm_navigation", 0)?;
    assert_eq!(actor.list("routes.dirty_documents"), [42]);
    send(&mut actor, "routes.back", 0)?;
    assert!(!actor.boolean("routes.dirty"));
    send(&mut actor, "routes.back", 0)?;
    assert!(actor.boolean("routes.dirty"));
    send(&mut actor, "routes.forward", 0)?;
    let before = actor.text_list("routes.paths").to_vec();
    assert!(send(&mut actor, "routes.rejected", 99).is_err());
    assert_eq!(actor.text_list("routes.paths"), before);
    assert_eq!(actor.integer("routes.implementation"), 1);
    assert!(send(&mut actor, "routes.open", -1).is_err());
    send(&mut actor, "routes.back", 0)?;
    send(&mut actor, "routes.open", 99)?;
    assert_eq!(actor.integer("routes.implementation"), 2);
    send(&mut actor, "routes.forward", 0)?;
    assert_eq!(location(&actor), "/documents/99");
    send(&mut actor, "routes.replace", 100)?;
    send(&mut actor, "routes.back", 0)?;
    assert_eq!(location(&actor), "/documents/42");
    send(&mut actor, "routes.forward", 0)?;
    assert_eq!(location(&actor), "/documents/100");
    for id in 101..250 {
        send(&mut actor, "routes.open", id)?;
    }
    assert_eq!(actor.text_list("routes.paths").len(), 130);
    assert_eq!(location(&actor), "/documents/249");
    for _ in 0..150 {
        send(&mut actor, "routes.back", 0)?;
    }
    assert_eq!(location(&actor), "/documents/121");
    actor.dispatch_text_request(key("routes.navigate"), "/documents/42".into())?;
    assert_eq!(location(&actor), "/documents/42");
    let ingress_history = actor.text_list("routes.paths").to_vec();
    actor.dispatch_text_request(key("routes.navigate"), "/documents/42".into())?;
    assert_eq!(actor.text_list("routes.paths"), ingress_history);
    for invalid in [
        "/missing",
        "/documents/0",
        "/documents/042",
        "/documents/+42",
        "/documents/42?x=1",
        "/documents/42#part",
    ] {
        assert!(
            actor
                .dispatch_text_request(key("routes.navigate"), invalid.into())
                .is_err()
        );
        assert_eq!(actor.text_list("routes.paths"), ingress_history);
        assert_eq!(location(&actor), "/documents/42");
    }
    assert!(
        actor
            .dispatch_text_request(key("routes.navigate"), "x".repeat(4097))
            .is_err()
    );
    assert_eq!(actor.text_list("routes.paths"), ingress_history);
    assert!(send(&mut actor, "routes.legacy", 0).is_err());
    assert_eq!(actor.text_list("routes.paths"), ingress_history);
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(location(&actor), "/documents/42");
    send(&mut actor, "routes.back", 0)?;
    assert_eq!(location(&actor), "/documents/121");
    let old_history = actor.text_list("routes.paths").to_vec();
    let old_version = actor.implementation_version();
    let old_schema = actor.schema_version;
    let old_state = actor.state_address();
    let old_slot = actor.slot_address("routes.paths");
    for candidate in &paths[2..4] {
        assert!(actor.reload(Path::new(candidate)).is_err());
        assert_eq!(actor.implementation_version(), old_version);
        assert_eq!(actor.schema_version, old_schema);
        assert_eq!(actor.state_address(), old_state);
        assert_eq!(actor.slot_address("routes.paths"), old_slot);
        assert_eq!(actor.text_list("routes.paths"), old_history);
        assert_eq!(actor.integer("routes.implementation"), 2);
        send(&mut actor, "routes.forward", 0)?;
        assert_eq!(location(&actor), "/documents/42");
        send(&mut actor, "routes.back", 0)?;
    }
    actor.reload(Path::new(&paths[4]))?;
    assert_ne!(actor.schema_version, old_schema);
    assert_eq!(actor.state_address(), old_state);
    assert_eq!(actor.slot_address("routes.paths"), old_slot);
    assert_eq!(location(&actor), "/docs/121");
    assert_eq!(
        actor.text_list("routes.paths"),
        &["0", "/docs/121", "/docs/42"]
    );
    send(&mut actor, "routes.forward", 0)?;
    assert_eq!(location(&actor), "/docs/42");
    assert!(
        actor
            .dispatch_text_request(key("routes.navigate"), "/documents/42".into())
            .is_err()
    );
    actor.dispatch_text_request(key("routes.navigate"), "/docs/73".into())?;
    assert_eq!(location(&actor), "/docs/73");
    println!(
        "PASS parameterized routes retain state/slot/history allocation through real DLL replacement; current code, history branching/replace/bounds and rejected navigation rollback; rejected/missing migration preserves old code and explicit migration renames route history"
    );
    Ok(())
}
