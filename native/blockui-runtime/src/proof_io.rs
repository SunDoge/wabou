use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn action(actor: &mut Actor, name: &str, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(name),
        value,
    })
}
fn finish(actor: &mut Actor, expected: i64) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while actor.integer("count") < expected && Instant::now() < deadline {
        actor.tick()?;
        thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(actor.integer("count"), expected);
    Ok(())
}
pub fn run(paths: &[String]) -> Result<(), String> {
    if paths.len() != 3 {
        return Err("io-proof needs base, v2, bad-render libraries".into());
    }
    let directory = PathBuf::from(format!(".runtime/io-proof-{}", std::process::id()));
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    let fixture = Fixture(directory);
    let path = fixture.0.join("document.txt");
    fs::write(&path, b"async document").map_err(|e| e.to_string())?;
    let mut actor = Actor::new(65, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.set_text(key("notes"), path.to_str().unwrap())?;
    action(&mut actor, "file_open", 1)?;
    let slot = actor.slot_address("notes");
    assert!(action(&mut actor, "io_start", 1).is_err());
    assert!(actor.tasks.is_empty());
    actor.reload(Path::new(&paths[2]))?;
    assert!(action(&mut actor, "io_start", 2).is_err());
    assert!(actor.tasks.is_empty());
    action(&mut actor, "io_start", 0)?;
    assert!(action(&mut actor, "io_start", 0).is_err());
    action(&mut actor, "file_close", 0)?; // worker owns its resource snapshot
    actor.reload(Path::new(&paths[1]))?;
    finish(&mut actor, 1)?;
    assert_eq!(actor.text("notes"), "v2:async document");
    assert_eq!(actor.slot_address("notes"), slot);
    assert!(actor.tasks.is_empty());
    // Result survives reload and remains usable independently of the closed file.
    actor.reload(Path::new(&paths[0]))?;
    action(&mut actor, "io_finish", key("document.read") as i64)?;
    assert_eq!(actor.text("notes"), "v1:async document");
    actor.set_text(key("notes"), path.to_str().unwrap())?;
    action(&mut actor, "file_open", 1)?;
    action(&mut actor, "io_start", 0)?;
    action(&mut actor, "io_cancel", 0)?;
    assert!(actor.tasks.is_empty());
    // Reuse before draining old completions: generation prevents stale delivery.
    action(&mut actor, "io_start", 0)?;
    finish(&mut actor, 3)?;
    for _ in 0..10 {
        actor.tick()?;
        thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(actor.integer("count"), 3);
    action(&mut actor, "io_policies", 0)?;
    actor.reload(Path::new(&paths[1]))?;
    assert!(!actor.tasks.contains_key(&key("document.read_cancel")));
    finish(&mut actor, 4)?;
    for _ in 0..10 {
        actor.tick()?;
        thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(actor.integer("count"), 4); // only restarted job's current generation delivered
    assert_eq!(actor.text("notes"), "v2:async document");
    action(&mut actor, "io_start", -1)?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut rejected = false;
    while Instant::now() < deadline {
        if actor.tick().is_err() {
            rejected = true;
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    assert!(rejected); // OS read failure is data, not a worker/plugin panic
    assert_eq!(actor.integer("count"), 4);
    action(&mut actor, "io_cancel", 0)?;
    action(&mut actor, "io_start", 0)?;
    finish(&mut actor, 5)?;
    assert_eq!(actor.text("notes"), "v2:async document");
    for _ in 0..100 {
        action(&mut actor, "io_start", 0)?;
        action(&mut actor, "io_cancel", 0)?;
    }
    actor.tick()?;
    assert_eq!(actor.integer("count"), 5);
    assert_eq!(actor.io_worker_count(), 2);
    let mut bounded = Actor::new(66, Path::new(".runtime"))?;
    bounded.reload(Path::new(&paths[0]))?;
    bounded.set_text(key("notes"), path.to_str().unwrap())?;
    action(&mut bounded, "file_open", 1)?;
    assert!(action(&mut bounded, "io_fill", 65).is_err());
    assert!(bounded.tasks.is_empty());
    assert_eq!(bounded.io_result_count(), 0);
    action(&mut bounded, "io_fill", 64)?;
    assert_eq!(bounded.tasks.len(), 64);
    assert!(bounded.io_queue_depth() <= 64);
    finish(&mut bounded, 64)?;
    assert_eq!(bounded.io_worker_count(), 2);
    assert_eq!(bounded.io_result_count(), 64);
    assert_eq!(bounded.io_result_bytes(), 64 * b"async document".len());
    assert!(action(&mut bounded, "io_start", 0).is_err()); // retained result budget
    action(&mut bounded, "io_clear", 0)?;
    assert_eq!(bounded.io_result_count(), 0);
    assert_eq!(bounded.io_result_bytes(), 0);
    action(&mut bounded, "io_start", 0)?;
    finish(&mut bounded, 65)?;
    // Task restart repeats its admitted handle snapshot even if its resource
    // separately reopens the pathname. Newly requested work sees the new handle.
    let mut snapshot = Actor::new(67, Path::new(".runtime"))?;
    snapshot.reload(Path::new(&paths[0]))?;
    snapshot.set_text(key("notes"), path.to_str().unwrap())?;
    action(&mut snapshot, "file_open", 0)?;
    let descriptor = snapshot.file_descriptor("document.restart").unwrap();
    action(&mut snapshot, "io_resource_snapshot", 0)?;
    let replacement = fixture.0.join("replacement.txt");
    fs::write(&replacement, b"new resource inode").map_err(|e| e.to_string())?;
    fs::rename(&replacement, &path).map_err(|e| e.to_string())?;
    snapshot.reload(Path::new(&paths[1]))?;
    assert_ne!(
        snapshot.file_descriptor("document.restart"),
        Some(descriptor)
    );
    finish(&mut snapshot, 1)?;
    assert_eq!(snapshot.text("notes"), "v2:async document");
    action(&mut snapshot, "io_resource_snapshot", 0)?;
    finish(&mut snapshot, 2)?;
    assert_eq!(snapshot.text("notes"), "v2:new resource inode");
    drop(snapshot);
    // Writes are host-owned effects admitted only after a successful turn.
    let write_path = fixture.0.join("write.txt");
    fs::write(&write_path, b"original document").map_err(|e| e.to_string())?;
    let mut writer = Actor::new(70, Path::new(".runtime"))?;
    writer.reload(Path::new(&paths[0]))?;
    writer.set_text(key("notes"), write_path.to_str().unwrap())?;
    action(&mut writer, "file_open", 1)?;
    assert!(action(&mut writer, "io_write", 0).is_err()); // read-only descriptor
    assert!(action(&mut writer, "io_write", 7).is_err()); // save also requires explicit write access
    action(&mut writer, "file_close", 0)?;
    action(&mut writer, "file_open", 2)?;
    for mode in [1, 3, 4, 5, 8, 10] {
        // rollback and restart/replay rejection
        assert!(action(&mut writer, "io_write", mode).is_err());
        assert!(writer.tasks.is_empty());
        assert_eq!(writer.io_result_count(), 0);
        assert_eq!(fs::read(&write_path).unwrap(), b"original document");
    }
    writer.reload(Path::new(&paths[2]))?;
    assert!(action(&mut writer, "io_write", 2).is_err()); // downstream view rejection
    assert!(action(&mut writer, "io_write", 9).is_err()); // save cannot escape failed view
    assert!(writer.tasks.is_empty());
    assert_eq!(fs::read(&write_path).unwrap(), b"original document");
    writer.reload(Path::new(&paths[0]))?;
    action(&mut writer, "io_write", 6)?; // representable u64 range, rejected by OS positional I/O
    writer.reload(Path::new(&paths[1]))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut rejected = false;
    while Instant::now() < deadline {
        if writer.tick().is_err() {
            rejected = true;
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    assert!(rejected); // I/O error reaches the new implementation's result handler
    assert_eq!(writer.integer("count"), 0);
    assert_eq!(fs::read(&write_path).unwrap(), b"original document");
    action(&mut writer, "io_cancel", 0)?;
    assert!(writer.tasks.is_empty());
    assert_eq!(writer.io_result_count(), 0);
    let slot = writer.slot_address("notes");
    action(&mut writer, "io_write", 0)?;
    action(&mut writer, "file_close", 0)?; // admitted snapshot retains the writable inode
    writer.reload(Path::new(&paths[1]))?; // no tick can dispatch an old-code completion
    finish(&mut writer, 1)?;
    assert_eq!(writer.text("notes"), "v2:");
    assert_eq!(writer.slot_address("notes"), slot);
    assert_eq!(fs::read(&write_path).unwrap(), b"savednal document");
    assert!(writer.tasks.is_empty());
    assert_eq!(writer.io_result_bytes(), 0);
    writer.reload(Path::new(&paths[0]))?;
    action(&mut writer, "io_finish", key("document.read") as i64)?;
    assert_eq!(writer.text("notes"), "v1:"); // immutable empty success result survives reload
    writer.set_text(key("notes"), write_path.to_str().unwrap())?;
    action(&mut writer, "file_open", 2)?;
    let old = fs::File::open(&write_path).unwrap();
    action(&mut writer, "io_write", 7)?;
    writer.reload(Path::new(&paths[1]))?;
    finish(&mut writer, 3)?;
    assert_eq!(writer.text("notes"), "v2:");
    assert_eq!(fs::read(&write_path).unwrap(), b"saved");
    use std::os::unix::fs::{FileExt, MetadataExt};
    assert_ne!(
        old.metadata().unwrap().ino(),
        fs::metadata(&write_path).unwrap().ino()
    );
    let mut retained = [0; 17];
    old.read_exact_at(&mut retained, 0).unwrap();
    assert_eq!(&retained, b"savednal document");
    assert_eq!(
        fs::read_dir(&fixture.0)
            .unwrap()
            .filter(|entry| entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".identity-save-"))
            .count(),
        0
    );
    let batch_path = fixture.0.join("batch.txt");
    fs::write(&batch_path, b"before batch").map_err(|e| e.to_string())?;
    let mut batch = Actor::new(71, Path::new(".runtime"))?;
    batch.reload(Path::new(&paths[0]))?;
    batch.set_text(key("notes"), batch_path.to_str().unwrap())?;
    action(&mut batch, "file_open", 2)?;
    let batch_slot = batch.slot_address("notes");
    assert!(action(&mut batch, "io_save_batch", 65).is_err());
    assert!(batch.tasks.is_empty());
    assert_eq!(batch.io_result_count(), 0);
    assert_eq!(fs::read(&batch_path).unwrap(), b"before batch");
    action(&mut batch, "io_save_batch", 32)?;
    assert_eq!(batch.tasks.len(), 32);
    batch.reload(Path::new(&paths[1]))?;
    assert_eq!(batch.slot_address("notes"), batch_slot);
    finish(&mut batch, 64)?; // v2 adds two per completion; v1 would leave 32
    assert_eq!(fs::read(&batch_path).unwrap(), b"save:31");
    assert_eq!(batch.io_result_count(), 32);
    assert_eq!(batch.io_result_bytes(), 0);
    assert!(batch.tasks.is_empty());
    drop(batch);
    println!(
        "PASS ordered save batch survives real replacement with stable state, latest admitted contents, all completions and atomic rollback of capacity overflow"
    );
    println!(
        "PASS atomic save replaces complete contents through new code, keeps old inode readable, rolls back failed turns, rejects replay and read-only saves, removes staging files and recovers from changed target paths"
    );
    let backup = fixture.0.join("saved-backup");
    fs::rename(&write_path, &backup).map_err(|e| e.to_string())?;
    fs::create_dir(&write_path).map_err(|e| e.to_string())?;
    action(&mut writer, "io_write", 7)?;
    writer.reload(Path::new(&paths[0]))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut rejected = false;
    while Instant::now() < deadline {
        if writer.tick().is_err() {
            rejected = true;
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    assert!(rejected);
    assert_eq!(writer.integer("count"), 3);
    assert!(write_path.is_dir());
    assert_eq!(fs::read(&backup).unwrap(), b"saved");
    action(&mut writer, "io_cancel", 0)?;
    fs::remove_dir(&write_path).map_err(|e| e.to_string())?;
    fs::rename(&backup, &write_path).map_err(|e| e.to_string())?;
    action(&mut writer, "io_write", 7)?;
    writer.reload(Path::new(&paths[1]))?;
    finish(&mut writer, 4)?;
    assert_eq!(writer.text("notes"), "v2:");
    assert_eq!(fs::read(&write_path).unwrap(), b"saved");
    drop(writer);
    println!(
        "PASS host async writes copy plugin bytes, reject read-only resources, restart replay, overflowing offsets and oversized buffers, recover from host write errors, rollback handler/view failures before any write, retain closed resource snapshots, complete through new code with unchanged state slot, preserve trailing bytes and results across replacement"
    );
    println!(
        "PASS bounded host async file reads, current-code completion, resource snapshot/result preservation across separately reopened resources, new requests use the reopened inode, transactional admission, cancel/reuse generation safety, restart/cancel policies, I/O error rollback and recovery, repeated cancellation, 64-task/result limits and result disposal"
    );
    Ok(())
}
