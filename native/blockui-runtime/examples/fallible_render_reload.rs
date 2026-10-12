use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;

fn send(actor: &mut Actor, action: &str) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(action),
        value: 0,
    })
}
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 2 {
        return Err("fallible_render_reload base rejected".into());
    }
    let mut actor = Actor::new(148, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    send(&mut actor, "put")?;
    let identity = actor.state_address();
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    let records = actor.text_list("documents").to_vec();
    let buffer = actor.text_list("documents").as_ptr();
    let draft = actor.text("draft").as_ptr();
    assert!(send(&mut actor, "corrupt").is_err());
    assert_eq!(actor.text_list("documents"), records);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert_eq!(actor.text("draft").as_ptr(), draft);
    assert_eq!(actor.text("draft"), "Draft 🦀");
    assert_eq!(actor.view.bindings[&key("document.42")].text, "Draft 🦀");
    assert!(actor.reload(Path::new(&paths[1])).is_err());
    assert_eq!(actor.state_address(), identity);
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert_eq!(actor.text("draft").as_ptr(), draft);
    assert_eq!(actor.view.bindings[&key("document.42")].text, "Draft 🦀");
    send(&mut actor, "put")?;
    println!(
        "PASS fallible bound render through real DLLs: ordinary question-mark decoding errors roll back turn writes and preserve buffers/view; rejected render candidate keeps old code callable"
    );
    Ok(())
}
