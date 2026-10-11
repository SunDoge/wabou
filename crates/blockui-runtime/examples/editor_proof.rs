//! Real DLL workflow proof for the independent SDK editor application.
use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn save(actor: &mut Actor) -> Result<(), String> {
    actor.dispatch(Message {
        action: key("editor.save"),
        value: 0,
    })
}
fn finish(actor: &mut Actor) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while actor.boolean("editor.busy") && Instant::now() < deadline {
        actor.tick()?;
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!actor.boolean("editor.busy"));
    Ok(())
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let file = Path::new(&args[3]);
    std::fs::write(file, b"original longer contents").map_err(|e| e.to_string())?;
    let mut actor = Actor::new(72, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[1]))?;
    actor.set_text(key("editor.draft"), "first draft")?;
    save(&mut actor)?;
    assert_eq!(actor.text("editor.status"), "Enter a file path.");
    assert_eq!(actor.text("editor.draft"), "first draft");
    actor.set_text(key("editor.path"), file.to_str().unwrap())?;
    let slot = actor.slot_address("editor.draft");
    save(&mut actor)?;
    assert!(actor.boolean("editor.busy"));
    assert_eq!(actor.text("editor.status"), "Saving…");
    assert_eq!(actor.text("editor.snapshot"), "first draft");
    let status_evaluations = actor.computed_evaluations("editor.display_status");
    actor.set_text(key("editor.draft"), "latest edit")?;
    // While saving, the status computation reads no draft: unrelated edits prune it.
    assert_eq!(
        actor.computed_evaluations("editor.display_status"),
        status_evaluations
    );
    save(&mut actor)?; // Busy guard preserves admitted snapshot and one task.
    assert_eq!(actor.tasks.len(), 1);
    assert_eq!(actor.text("editor.snapshot"), "first draft");
    actor.reload(Path::new(&args[2]))?;
    assert_eq!(actor.slot_address("editor.draft"), slot);
    assert_eq!(actor.text("editor.draft"), "latest edit");
    finish(&mut actor)?;
    assert_eq!(std::fs::read(file).unwrap(), b"first draft");
    assert_eq!(actor.text("editor.draft"), "latest edit");
    assert!(actor.text("editor.status").contains("still unsaved"));
    assert!(actor.tasks.is_empty());
    assert_eq!(actor.io_result_count(), 0);
    save(&mut actor)?;
    finish(&mut actor)?;
    assert_eq!(std::fs::read(file).unwrap(), b"latest edit");
    assert_eq!(actor.text("editor.status"), "Saved."); // Only replacement code uses this wording.
    actor.set_text(key("editor.draft"), "another edit")?;
    assert!(
        actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "Unsaved changes.")
    );
    let status_evaluations = actor.computed_evaluations("editor.display_status");
    actor.set_text(key("editor.draft"), "another edit")?;
    assert_eq!(
        actor.computed_evaluations("editor.display_status"),
        status_evaluations
    );
    actor.set_text(key("editor.draft"), "latest edit")?;
    assert_eq!(actor.view.bindings[&key("editor.status")].text, "Saved.");
    actor.set_text(key("editor.path"), "/missing/identity-editor/document")?;
    save(&mut actor)?;
    assert!(
        actor
            .text("editor.status")
            .starts_with("Could not start saving")
    );
    assert_eq!(actor.text("editor.draft"), "latest edit");
    assert!(!actor.boolean("editor.busy"));
    actor.set_text(key("editor.path"), file.to_str().unwrap())?;
    save(&mut actor)?;
    finish(&mut actor)?;
    assert_eq!(actor.text("editor.status"), "Saved.");
    actor.dispatch(Message {
        action: key("editor.load"),
        value: 0,
    })?;
    assert!(actor.boolean("editor.busy"));
    actor.set_text(key("editor.draft"), "edit during load")?;
    actor.reload(Path::new(&args[1]))?;
    finish(&mut actor)?;
    assert_eq!(actor.text("editor.draft"), "edit during load");
    assert!(actor.text("editor.status").contains("Kept your edits"));
    actor.dispatch(Message {
        action: key("editor.load"),
        value: 0,
    })?;
    actor.reload(Path::new(&args[2]))?;
    finish(&mut actor)?;
    assert_eq!(actor.text("editor.draft"), "latest edit");
    assert_eq!(actor.text("editor.status"), "Loaded.");
    assert_eq!(actor.io_result_count(), 0);
    std::fs::write(file, [0xff]).map_err(|e| e.to_string())?;
    actor.dispatch(Message {
        action: key("editor.load"),
        value: 0,
    })?;
    finish(&mut actor)?;
    assert_eq!(actor.text("editor.draft"), "latest edit");
    assert!(actor.text("editor.status").contains("not UTF-8"));
    std::fs::write(file, vec![b'a'; 1024 * 1024]).map_err(|e| e.to_string())?;
    actor.dispatch(Message {
        action: key("editor.load"),
        value: 0,
    })?;
    finish(&mut actor)?;
    assert_eq!(actor.text("editor.draft"), "latest edit");
    assert!(actor.text("editor.status").contains("too large"));
    save(&mut actor)?;
    finish(&mut actor)?;
    assert_eq!(std::fs::read(file).unwrap(), b"latest edit");
    println!(
        "PASS asynchronous editor load survives reload, preserves concurrent edits, rejects invalid UTF8 and oversized contents, cleans results and recovers by saving"
    );
    let other = file.parent().unwrap().join("other.txt");
    std::fs::write(&other, b"other file").map_err(|e| e.to_string())?;
    actor.set_text(key("editor.draft"), "snapshot for original path")?;
    save(&mut actor)?;
    actor.set_text(key("editor.path"), other.to_str().unwrap())?;
    actor.reload(Path::new(&args[1]))?;
    finish(&mut actor)?;
    assert_eq!(std::fs::read(file).unwrap(), b"snapshot for original path");
    assert_eq!(std::fs::read(&other).unwrap(), b"other file");
    assert!(actor.text("editor.status").contains("earlier file"));
    assert!(
        actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text.contains("has not been saved there"))
    );
    actor.set_text(key("editor.path"), file.to_str().unwrap())?;
    actor.dispatch(Message {
        action: key("editor.load"),
        value: 0,
    })?;
    actor.set_text(key("editor.path"), other.to_str().unwrap())?;
    actor.reload(Path::new(&args[2]))?;
    finish(&mut actor)?;
    assert_eq!(actor.text("editor.draft"), "snapshot for original path");
    assert!(
        actor
            .text("editor.status")
            .contains("path changed while loading")
    );
    actor.dispatch(Message {
        action: key("editor.load"),
        value: 0,
    })?;
    finish(&mut actor)?;
    assert_eq!(actor.text("editor.draft"), "other file");
    assert_eq!(actor.text("editor.status"), "Loaded.");
    assert_eq!(actor.io_result_count(), 0);
    println!(
        "PASS editor path snapshots survive reload, save only admitted file, reject stale load results for changed paths and recover by loading the new path"
    );
    let created = file
        .parent()
        .unwrap()
        .join(format!("new-document-{}.txt", std::process::id()));
    assert!(!created.exists());
    let mut new_document = Actor::new(76, Path::new(".runtime"))?;
    new_document.reload(Path::new(&args[1]))?;
    new_document.set_text(key("editor.path"), created.to_str().unwrap())?;
    new_document.set_text(key("editor.draft"), "新文档😀 first draft")?;
    let new_slot = new_document.slot_address("editor.draft");
    let new_storage = new_document.state_address();
    save(&mut new_document)?;
    assert!(
        new_document.boolean("editor.busy"),
        "Save must admit creation of a missing file"
    );
    new_document.set_text(key("editor.draft"), "newer unsaved edits")?;
    new_document.reload(Path::new(&args[2]))?;
    finish(&mut new_document)?;
    assert_eq!(
        std::fs::read(&created).unwrap(),
        "新文档😀 first draft".as_bytes()
    );
    assert_eq!(new_document.text("editor.draft"), "newer unsaved edits");
    assert!(new_document.text("editor.status").contains("still unsaved"));
    assert_eq!(new_document.slot_address("editor.draft"), new_slot);
    assert_eq!(new_document.state_address(), new_storage);
    save(&mut new_document)?;
    finish(&mut new_document)?;
    assert_eq!(std::fs::read(&created).unwrap(), b"newer unsaved edits");
    assert_eq!(new_document.text("editor.status"), "Saved.");
    new_document.set_text(key("editor.path"), file.parent().unwrap().to_str().unwrap())?;
    save(&mut new_document)?;
    finish(&mut new_document)?;
    assert!(
        new_document
            .text("editor.status")
            .starts_with("Save failed")
    );
    assert_eq!(new_document.text("editor.draft"), "newer unsaved edits");
    assert_eq!(new_document.io_result_count(), 0);
    assert!(new_document.tasks.is_empty());
    new_document.set_text(key("editor.path"), created.to_str().unwrap())?;
    save(&mut new_document)?;
    finish(&mut new_document)?;
    assert_eq!(new_document.text("editor.status"), "Saved.");
    println!(
        "PASS editor creates a new document across reload without losing newer edits, then replaces it on subsequent saves, preserves text on failure and releases tasks/results"
    );
    #[cfg(feature = "native")]
    {
        let mut long = Actor::new(74, Path::new(".runtime"))?;
        long.reload(Path::new(&args[1]))?;
        blockui_runtime::native::textarea_proof(long, Path::new(&args[2]))?;
        let native_new_path = file
            .parent()
            .unwrap()
            .join(format!("native-new-document-{}.txt", std::process::id()));
        assert!(!native_new_path.exists());
        let mut native_new = Actor::new(75, Path::new(".runtime"))?;
        native_new.reload(Path::new(&args[1]))?;
        blockui_runtime::native::editor_proof(native_new, Path::new(&args[2]), &native_new_path)?;
        println!(
            "PASS native editor Save creates a new file while retaining editor identity, Unicode selection, focus and draft through reload"
        );
        let mut native = Actor::new(73, Path::new(".runtime"))?;
        native.reload(Path::new(&args[1]))?;
        if args.get(4).map(String::as_str) == Some("--window") {
            let window_new_path = file
                .parent()
                .unwrap()
                .join(format!("window-new-document-{}.txt", std::process::id()));
            assert!(!window_new_path.exists());
            let source = blockui_runtime::native::NativeSource::without_watch(native)?
                .editor_window(Path::new(&args[2]), &window_new_path)?;
            wabou_shell_vello::run_window_with_options(Box::new(source), Default::default())
                .map_err(|e| e.to_string())?;
        } else {
            blockui_runtime::native::editor_proof(native, Path::new(&args[2]), file)?;
        }
    }
    println!(
        "PASS independent editor preserves newer edits and busy snapshot through real reload, saves exact contents with new-code status, guards repeated save, releases task/results and recovers from invalid paths"
    );
    Ok(())
}
