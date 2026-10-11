//! Actual document libraries exercise retained native editor instances.
use super::*;
use crate::{ActorId, Runtime, native_model::NativeFamily};
pub fn proof(
    parent: &Path,
    replacement: &Path,
    editor: &Path,
    editor_v2: &Path,
) -> Result<(), String> {
    let root = ActorId(81);
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("editor", editor)?;
    runtime.spawn(root, parent)?;
    let children: Vec<_> = runtime
        .declared_children()
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(children.len(), 2);
    let mut source = NativeSource::without_watch(NativeFamily::new(runtime, root)?)?;
    let mut text = TextContext::new();
    source.build_frame(&mut text, 800, 600);
    let mut retained = Vec::new();
    for (index, child) in children.iter().copied().enumerate() {
        let local = *source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .view
            .bindings
            .iter()
            .find(|(_, binding)| binding.node.slot == key("editor.draft"))
            .unwrap()
            .0;
        let widget = source.actor.widget(child, local).unwrap();
        let draft = format!("Document {index}: unsaved 中文 🦀");
        source.actor.set_widget_text(widget, &draft)?;
        source.sync()?;
        source.build_frame(&mut text, 800, 600);
        source
            .editors
            .get_mut(&widget)
            .unwrap()
            .config_changed(r#"{"selection":{"anchor":2,"head":5}}"#)
            .map_err(|e| e.to_string())?;
        source.build_frame(&mut text, 800, 600);
        retained.push((
            child,
            widget,
            draft,
            (&**source.editors.get(&widget).unwrap() as *const TextInput) as usize,
            source.ids[&widget],
            source.editors[&widget].text_selection(),
            source
                .actor
                .runtime()
                .actor(child)
                .unwrap()
                .slot_address("editor.draft"),
        ));
    }
    let scroll = source
        .actor
        .root_widget(key("workspace.documents"))
        .unwrap();
    source
        .actor
        .view_mut()
        .renderer
        .get_mut(&scroll)
        .unwrap()
        .scroll = 40.0;
    source.build_frame(&mut text, 800, 600);
    assert_eq!(source.actor.view().renderer[&scroll].scroll, 40.0);
    source.set_focus(Some(retained[0].1));
    if let Some(paths) = std::env::var_os("BLOCKUI_DOCUMENTS_WATCH_CONFIGS") {
        use std::io::Write;
        let paths: Vec<_> = std::env::split_paths(&paths).collect();
        if paths.len() != 2 {
            return Err("expected parent and editor watcher configurations".into());
        }
        let repo = std::env::current_dir().map_err(|e| e.to_string())?;
        source.watch_actor(root, ReloadConfig::from_file(&paths[0], &repo)?)?;
        source.watch_implementation("editor", ReloadConfig::from_file(&paths[1], &repo)?)?;
        let parent_version = source
            .actor
            .runtime()
            .actor(root)
            .unwrap()
            .implementation_version();
        let versions: Vec<_> = children
            .iter()
            .map(|id| {
                source
                    .actor
                    .runtime()
                    .actor(*id)
                    .unwrap()
                    .implementation_version()
            })
            .collect();
        println!("DOCUMENTS_WATCH_READY");
        std::io::stdout().flush().unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut parent_failed = false;
        let mut child_failed = false;
        let mut parent_changed = false;
        let mut children_changed = false;
        while Instant::now() < deadline {
            source.poll_async();
            source.build_frame(&mut text, 800, 600);
            for (child, widget, draft, address, node, selection, slot) in &retained {
                assert_eq!(
                    (&**source.editors.get(widget).unwrap() as *const TextInput) as usize,
                    *address
                );
                assert_eq!(source.ids[widget], *node);
                assert_eq!(source.editors[widget].text_selection(), *selection);
                assert_eq!(
                    source
                        .actor
                        .runtime()
                        .actor(*child)
                        .unwrap()
                        .text("editor.draft"),
                    draft
                );
                assert_eq!(
                    source
                        .actor
                        .runtime()
                        .actor(*child)
                        .unwrap()
                        .slot_address("editor.draft"),
                    *slot
                );
            }
            assert_eq!(source.focus, Some(retained[0].1));
            assert_eq!(source.actor.view().renderer[&scroll].scroll, 40.0);
            if !parent_failed
                && source
                    .actor_diagnostic(root)
                    .is_some_and(|s| s.contains("documents_parent_failure"))
            {
                assert_eq!(
                    source
                        .actor
                        .runtime()
                        .actor(root)
                        .unwrap()
                        .implementation_version(),
                    parent_version
                );
                parent_failed = true;
                println!("DOCUMENTS_PARENT_FAILED");
            }
            if !child_failed
                && children.iter().any(|id| {
                    source
                        .actor_diagnostic(*id)
                        .is_some_and(|s| s.contains("documents_editor_failure"))
                })
            {
                for (id, version) in children.iter().zip(&versions) {
                    assert_eq!(
                        source
                            .actor
                            .runtime()
                            .actor(*id)
                            .unwrap()
                            .implementation_version(),
                        *version
                    );
                }
                child_failed = true;
                println!("DOCUMENTS_EDITOR_FAILED");
            }
            if !parent_changed
                && source
                    .actor
                    .runtime()
                    .actor(root)
                    .unwrap()
                    .implementation_version()
                    != parent_version
            {
                assert!(parent_failed);
                parent_changed = true;
                println!("DOCUMENTS_PARENT_RELOADED");
            }
            if !children_changed
                && children.iter().zip(&versions).all(|(id, version)| {
                    source
                        .actor
                        .runtime()
                        .actor(*id)
                        .unwrap()
                        .implementation_version()
                        != *version
                })
            {
                assert!(child_failed);
                children_changed = true;
                println!("DOCUMENTS_EDITORS_RELOADED");
            }
            std::io::stdout().flush().unwrap();
            if parent_changed && children_changed {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            parent_failed && child_failed && parent_changed && children_changed,
            "watch proof timed out"
        );
        println!(
            "PASS actual parent/editor source failure/recovery preserves both native documents through shared watcher replacement"
        );
    }
    source.actor.tell(
        root,
        Message {
            action: key("workspace.reverse"),
            value: 0,
        },
    )?;
    source.actor.tick()?;
    source.sync()?;
    source.replace(replacement)?;
    source.actor.reload_targets(&children, editor_v2)?;
    source.sync()?;
    source.build_frame(&mut text, 800, 600);
    assert_eq!(source.focus, Some(retained[0].1));
    assert_eq!(source.actor.view().renderer[&scroll].scroll, 40.0);
    for (child, widget, draft, address, node, selection, slot) in &retained {
        assert_eq!(
            (&**source.editors.get(widget).unwrap() as *const TextInput) as usize,
            *address
        );
        assert_eq!(source.ids[widget], *node);
        assert_eq!(source.editors[widget].text_selection(), *selection);
        assert_eq!(
            source
                .actor
                .runtime()
                .actor(*child)
                .unwrap()
                .text("editor.draft"),
            draft
        );
        assert_eq!(
            source
                .actor
                .runtime()
                .actor(*child)
                .unwrap()
                .slot_address("editor.draft"),
            *slot
        );
    }
    source.actor.tell(
        root,
        Message {
            action: key("workspace.new"),
            value: 0,
        },
    )?;
    source.actor.tick()?;
    source.sync()?;
    source.build_frame(&mut text, 800, 600);
    assert_eq!(source.editors.len(), 6); // path + text for each of three documents
    assert_eq!(source.focus, Some(retained[0].1));
    assert_eq!(source.actor.view().renderer[&scroll].scroll, 40.0);
    for (_, widget, _, address, node, selection, _) in &retained {
        assert_eq!(
            (&**source.editors.get(widget).unwrap() as *const TextInput) as usize,
            *address
        );
        assert_eq!(source.ids[widget], *node);
        assert_eq!(source.editors[widget].text_selection(), *selection);
    }
    println!(
        "PASS native document editors retain allocation, nodes, Unicode drafts, selection, focus, list scroll and state slots across reorder, parent/grouped-child replacement and insertion"
    );
    let closed_key = 2_u64
        .to_le_bytes()
        .into_iter()
        .fold(key("workspace.document"), |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    let closed_actor = source.actor.runtime().child(root, closed_key).unwrap();
    let closed = retained
        .iter()
        .find(|entry| entry.0 == closed_actor)
        .unwrap();
    let survivor = retained
        .iter()
        .find(|entry| entry.0 != closed_actor)
        .unwrap();
    source.set_focus(Some(closed.1));
    source.actor.tell(
        root,
        Message {
            action: key("workspace.close"),
            value: 2,
        },
    )?;
    source.actor.tick()?;
    source.sync()?;
    source.build_frame(&mut text, 800, 600);
    assert_eq!(source.editors.len(), 4);
    assert!(source.actor.runtime().actor(closed_actor).is_none());
    assert!(!source.editors.contains_key(&closed.1));
    assert!(!source.ids.contains_key(&closed.1));
    assert!(!source.applied.contains_key(&closed.1));
    assert!(source.placed.iter().all(|node| node.node_id != closed.4));
    assert_eq!(source.focus, None);
    assert_eq!(
        (&**source.editors.get(&survivor.1).unwrap() as *const TextInput) as usize,
        survivor.3
    );
    assert_eq!(source.editors[&survivor.1].text_selection(), survivor.5);
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(survivor.0)
            .unwrap()
            .text("editor.draft"),
        survivor.2
    );
    for id in [1, 3] {
        source.actor.tell(
            root,
            Message {
                action: key("workspace.close"),
                value: id,
            },
        )?;
    }
    source.actor.tick()?;
    source.sync()?;
    source.build_frame(&mut text, 800, 600);
    assert!(source.editors.is_empty());
    assert_eq!(source.actor.runtime().len(), 1);
    assert_eq!(source.actor.view().renderer[&scroll].scroll, 0.0);
    assert!(
        source
            .actor
            .view()
            .bindings
            .values()
            .any(|binding| binding.text.starts_with("No documents."))
    );
    println!(
        "PASS native document disposal clears removed editor/layout caches and focus, retains sibling editor, and empty list releases all editors and clamps scroll"
    );

    Ok(())
}
