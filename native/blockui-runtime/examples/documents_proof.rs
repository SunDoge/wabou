use blockui_abi::key;
use blockui_runtime::{ActorId, Message, Runtime};
use std::path::Path;
fn document_key(id: u64) -> u64 {
    id.to_le_bytes()
        .into_iter()
        .fold(key("workspace.document"), |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        })
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("documents_proof parent parent-v2 editor editor-v2".into());
    }
    #[cfg(feature = "native")]
    blockui_runtime::native::documents_proof(
        Path::new(&args[1]),
        Path::new(&args[2]),
        Path::new(&args[3]),
        Path::new(&args[4]),
    )?;
    let root = ActorId(80);
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("editor", Path::new(&args[3]))?;
    runtime.spawn(root, Path::new(&args[1]))?;
    let first = runtime.child(root, document_key(1)).unwrap();
    let second = runtime.child(root, document_key(2)).unwrap();
    for (id, text) in [(first, "first unsaved 中文"), (second, "second unsaved 🦀")] {
        let local = *runtime
            .actor(id)
            .unwrap()
            .view
            .bindings
            .iter()
            .find(|(_, binding)| binding.node.slot == key("editor.draft"))
            .unwrap()
            .0;
        let view = runtime.compose(root)?;
        view.set_text(&mut runtime, view.widget(id, local).unwrap(), text)?;
    }
    let first_storage = runtime.actor(first).unwrap().state_address();
    let first_slot = runtime.actor(first).unwrap().slot_address("editor.draft");
    runtime.tell(
        root,
        Message {
            action: key("workspace.reverse"),
            value: 0,
        },
    )?;
    assert!(runtime.tick().is_empty());
    assert_eq!(
        runtime.actor(root).unwrap().list("workspace.documents"),
        &[2, 1]
    );
    runtime.reload(root, Path::new(&args[2]))?;
    runtime.reload(first, Path::new(&args[4]))?;
    assert_eq!(runtime.child(root, document_key(1)), Some(first));
    assert_eq!(runtime.child(root, document_key(2)), Some(second));
    assert_eq!(runtime.actor(first).unwrap().state_address(), first_storage);
    assert_eq!(
        runtime.actor(first).unwrap().slot_address("editor.draft"),
        first_slot
    );
    assert_eq!(
        runtime.actor(first).unwrap().text("editor.draft"),
        "first unsaved 中文"
    );
    assert_eq!(
        runtime.actor(second).unwrap().text("editor.draft"),
        "second unsaved 🦀"
    );
    runtime.tell(
        root,
        Message {
            action: key("workspace.new"),
            value: 0,
        },
    )?;
    assert!(runtime.tick().is_empty());
    let third = runtime.child(root, document_key(3)).unwrap();
    assert_ne!(third, first);
    assert_eq!(
        runtime.actor(root).unwrap().list("workspace.documents"),
        &[2, 1, 3]
    );
    runtime.tell(
        root,
        Message {
            action: key("workspace.close"),
            value: 2,
        },
    )?;
    assert!(runtime.tick().is_empty());
    assert!(runtime.actor(second).is_none());
    assert_eq!(runtime.child(root, document_key(1)), Some(first));
    assert_eq!(
        runtime.actor(first).unwrap().text("editor.draft"),
        "first unsaved 中文"
    );
    for id in [1, 3] {
        runtime.tell(
            root,
            Message {
                action: key("workspace.close"),
                value: id,
            },
        )?;
    }
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.len(), 1);
    assert!(
        runtime
            .actor(root)
            .unwrap()
            .list("workspace.documents")
            .is_empty()
    );
    let view = &runtime.actor(root).unwrap().view;
    assert!(
        view.bindings[&key("workspace.empty")]
            .text
            .contains("New document")
    );
    assert_ne!(view.bindings[&key("workspace.reverse")].node.disabled, 0);
    runtime.reload(root, Path::new(&args[1]))?;
    assert_eq!(runtime.len(), 1);
    runtime.tell(
        root,
        Message {
            action: key("workspace.new"),
            value: 0,
        },
    )?;
    assert!(runtime.tick().is_empty());
    assert_eq!(
        runtime.actor(root).unwrap().list("workspace.documents"),
        &[4]
    );
    let fourth = runtime.child(root, document_key(4)).unwrap();
    assert_ne!(fourth, first);
    assert_ne!(fourth, second);
    assert_ne!(fourth, third);
    assert_eq!(runtime.actor(fourth).unwrap().text("editor.draft"), "");
    assert!(
        !runtime
            .actor(root)
            .unwrap()
            .view
            .bindings
            .contains_key(&key("workspace.empty"))
    );
    println!(
        "PASS closing all documents exposes an empty state and recreates a fresh document identity after replacement"
    );
    assert!(runtime.remove(root));
    assert!(runtime.is_empty());
    println!(
        "PASS dynamic documents preserve actor/state/slot identity and Unicode drafts across reorder, parent/child replacement, insertion and sibling disposal"
    );
    Ok(())
}
