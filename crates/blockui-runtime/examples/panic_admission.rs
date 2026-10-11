//! Isolated-process proof for a candidate that panics during view admission.
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 2);
    let mut actor = Actor::new(134, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[0]))?;
    let state = actor.state_address();
    let slot = actor.slot_address("labels.items");
    let version = actor.implementation_version();
    let error = actor
        .reload(Path::new(&args[1]))
        .expect_err("candidate must be rejected");
    println!("Candidate rejected: {error}");
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("labels.items"), slot);
    assert_eq!(actor.implementation_version(), version);
    actor.dispatch(Message {
        action: blockui_abi::key("seed"),
        value: 0,
    })?;
    assert_eq!(actor.text_list("labels.items"), ["", "a\0b", "标题😀"]);
    println!("PASS panicking candidate rejected; current actor and actions retained");
    Ok(())
}
