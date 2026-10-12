use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::{
    fs,
    path::{Path, PathBuf},
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
pub fn run(paths: &[String]) -> Result<(), String> {
    if paths.len() != 4 {
        return Err("resource-proof needs base, v2, bad-render and schema-extra libraries".into());
    }
    let directory = PathBuf::from(format!(".runtime/file-proof-{}", std::process::id()));
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    let fixture = Fixture(directory);
    let path = fixture.0.join("document.txt");
    fs::write(&path, b"original document").map_err(|e| e.to_string())?;
    let mut writable = Actor::new(69, Path::new(".runtime"))?;
    writable.reload(Path::new(&paths[0]))?;
    writable.set_text(key("notes"), path.to_str().unwrap())?;
    action(&mut writable, "file_open", 3)?;
    let descriptor = writable.file_descriptor("document.file").unwrap();
    writable.reload(Path::new(&paths[1]))?;
    assert_eq!(writable.file_descriptor("document.file"), Some(descriptor));
    for name in ["document.file", "document.restart"] {
        let fd = writable.file_descriptor(name).unwrap();
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_GETFL) } & libc::O_ACCMODE,
            libc::O_RDWR
        );
    }
    assert_eq!(fs::read(&path).unwrap(), b"original document");
    drop(writable);
    let mut actor = Actor::new(63, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.set_text(key("notes"), path.to_str().unwrap())?;
    assert!(action(&mut actor, "file_reject", 0).is_err());
    assert!(actor.file_descriptor("document.file").is_none());
    action(&mut actor, "file_open", 0)?;
    let descriptor = actor.file_descriptor("document.file").unwrap();
    let restart = actor.file_descriptor("document.restart").unwrap();
    assert!(action(&mut actor, "file_open", 0).is_err());
    assert_eq!(actor.file_descriptor("document.file"), Some(descriptor));
    assert!(action(&mut actor, "file_close", 1).is_err());
    assert_eq!(actor.file_descriptor("document.file"), Some(descriptor));
    action(&mut actor, "file_read", 0)?;
    assert_eq!(actor.text("notes"), "original document");
    let new_path = fixture.0.join("new.txt");
    fs::write(&new_path, b"replacement document").map_err(|e| e.to_string())?;
    fs::rename(&new_path, &path).map_err(|e| e.to_string())?;
    let storage = actor.state_address();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.file_descriptor("document.file"), Some(descriptor));
    assert!(actor.file_descriptor("document.cancel").is_none());
    assert_ne!(actor.file_descriptor("document.restart"), Some(restart));
    action(&mut actor, "file_read", 0)?;
    assert_eq!(actor.text("notes"), "original document"); // old inode, not reopened path
    action(&mut actor, "file_read", 2)?;
    assert_eq!(actor.text("notes"), "replacement document");
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    let implementation = actor.implementation_version();
    let restart = actor.file_descriptor("document.restart");
    assert!(actor.reload(Path::new(&paths[0])).is_err()); // restart open fails before commit
    assert_eq!(actor.implementation_version(), implementation);
    assert_eq!(actor.file_descriptor("document.restart"), restart);
    action(&mut actor, "file_read", 2)?;
    assert_eq!(actor.text("notes"), "replacement document");
    fs::write(&path, b"third document").map_err(|e| e.to_string())?;
    actor.reload(Path::new(&paths[2]))?;
    assert!(action(&mut actor, "file_close", 2).is_err()); // late rendering failure
    assert_eq!(actor.file_descriptor("document.file"), Some(descriptor));
    assert_eq!(actor.text("notes"), "replacement document");
    action(&mut actor, "file_read", 0)?;
    assert_eq!(actor.text("notes"), "original document");
    action(&mut actor, "file_close", 0)?;
    assert!(actor.file_descriptor("document.file").is_none());
    assert!(action(&mut actor, "file_read", 0).is_err());
    let descriptor = actor.file_descriptor("document.restart");
    actor.set_text(key("notes"), path.to_str().unwrap())?;
    action(&mut actor, "file_open", 1)?;
    let preserved = actor.file_descriptor("document.file");
    actor.reload(Path::new(&paths[3]))?;
    assert_eq!(actor.file_descriptor("document.file"), preserved);
    assert_eq!(actor.integer("bonus"), 7);
    assert_ne!(actor.file_descriptor("document.restart"), descriptor);
    action(&mut actor, "file_read", 2)?;
    assert_eq!(actor.text("notes"), "third document");
    actor.reload(Path::new(&paths[0]))?; // SDK default migration removes bonus automatically
    assert_eq!(actor.slot_address("bonus"), 0);
    assert_eq!(actor.file_descriptor("document.file"), preserved);
    let mut bounded = Actor::new(64, Path::new(".runtime"))?;
    bounded.reload(Path::new(&paths[0]))?;
    bounded.set_text(key("notes"), path.to_str().unwrap())?;
    let descriptors = fs::read_dir("/proc/self/fd").unwrap().count();
    assert!(action(&mut bounded, "file_fill", 65).is_err());
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), descriptors);
    action(&mut bounded, "file_fill", 64)?;
    assert_eq!(
        fs::read_dir("/proc/self/fd").unwrap().count(),
        descriptors + 64
    );
    drop(bounded);
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), descriptors);
    println!(
        "PASS typed host File resources preserve descriptor and inode across replacement, cancel/reopen policies, open/close/late-render rollback, failed reopen retains old implementation and resources, schema migration, continued reads, 64-file limit and no descriptor leaks on rollback/shutdown"
    );
    Ok(())
}
