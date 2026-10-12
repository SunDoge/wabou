use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::{
    fs,
    path::{Path, PathBuf},
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
fn ready(actor: &mut Actor) -> Result<i64, String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while actor.integer("task_result") == 0 {
        actor.tick()?;
        if Instant::now() >= deadline {
            return Err("creation completion timed out".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(actor.integer("task_result"))
}
pub fn run(paths: &[String]) -> Result<(), String> {
    if paths.len() != 3 {
        return Err("create-proof base replacement failed-binding".into());
    }
    let root = PathBuf::from(format!(".runtime/create-proof-{}", std::process::id()));
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    let fixture = Fixture(root);
    let path = fixture.0.join("新文档.txt");
    let mut actor = Actor::new(944, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.set_text(key("notes"), path.to_str().unwrap())?;
    for mode in [1, 2, 3] {
        assert!(action(&mut actor, "create_file", mode).is_err());
        assert!(!path.exists());
        assert!(actor.tasks.is_empty());
        assert_eq!(actor.io_result_count(), 0);
    }
    {
        let mut failed = Actor::new(945, Path::new(".runtime"))?;
        failed.reload(Path::new(&paths[2]))?;
        failed.set_text(key("notes"), path.to_str().unwrap())?;
        assert!(action(&mut failed, "create_file", 5).is_err());
        assert_eq!(failed.integer("count"), 0);
        assert!(failed.tasks.is_empty());
        assert!(!path.exists());
        assert!(action(&mut failed, "create_read_only", 0).is_err());
        assert_eq!(failed.integer("count"), 0);
        assert!(failed.tasks.is_empty());
        assert!(!path.exists());
    }
    let fifo = fixture.0.join("fifo");
    let fifo_name = std::ffi::CString::new(fifo.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo_name.as_ptr(), 0o600) }, 0);
    actor.set_text(key("notes"), fifo.join("new.txt").to_str().unwrap())?;
    assert!(action(&mut actor, "create_file", 0).is_err());
    actor.set_text(key("notes"), path.to_str().unwrap())?;
    fs::remove_file(&fifo).map_err(|e| e.to_string())?;
    let storage = actor.state_address();
    let slot = actor.slot_address("notes");
    action(&mut actor, "create_file", 0)?;
    actor.reload(Path::new(&paths[1]))?;
    let changed = fixture.0.join("later.txt");
    actor.set_text(key("notes"), changed.to_str().unwrap())?;
    assert_eq!(ready(&mut actor)?, 1001);
    assert_eq!(
        fs::read(&path).map_err(|e| e.to_string())?,
        "新文档😀".as_bytes()
    );
    assert!(!changed.exists());
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("notes"), slot);
    assert_eq!(actor.io_result_count(), 0);
    assert_eq!(actor.artifact_count(), 1);
    actor.set_text(key("notes"), path.to_str().unwrap())?;
    action(&mut actor, "task_done", 0)?;
    action(&mut actor, "create_file", 0)?;
    assert_eq!(ready(&mut actor)?, -1);
    assert_eq!(
        fs::read(&path).map_err(|e| e.to_string())?,
        "新文档😀".as_bytes()
    );
    let link = fixture.0.join("link");
    std::os::unix::fs::symlink(path.file_name().unwrap(), &link).map_err(|e| e.to_string())?;
    actor.set_text(key("notes"), link.to_str().unwrap())?;
    action(&mut actor, "task_done", 0)?;
    action(&mut actor, "create_file", 0)?;
    assert_eq!(ready(&mut actor)?, -1);
    assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(
        fs::read(&path).map_err(|e| e.to_string())?,
        "新文档😀".as_bytes()
    );
    actor.set_text(key("notes"), changed.to_str().unwrap())?;
    action(&mut actor, "task_done", 0)?;
    action(&mut actor, "create_file", 4)?;
    assert_eq!(ready(&mut actor)?, 1001);
    assert!(fs::read(&changed).map_err(|e| e.to_string())?.is_empty());
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 3);
    assert!(actor.tasks.is_empty());
    println!(
        "PASS atomic new-file creation, owned UTF-8 path/bytes across real replacement, current-code completion, state allocation retention, handler/binding rollback, size/replay rejection, empty files, existing file/symlink preservation and staging cleanup"
    );
    Ok(())
}
