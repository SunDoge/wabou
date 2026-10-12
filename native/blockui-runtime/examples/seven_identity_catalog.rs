use blockui_runtime::{Actor, Message};
#[path = "../src/seven_identity_fixture.rs"]
mod identities;
use blockui_abi as abi;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let mut actor = Actor::new(812, std::path::Path::new(".runtime"))?;
    actor.reload(std::path::Path::new(&args[1]))?;
    if let Some(width) = args.get(3) {
        let action = actor
            .view
            .bindings
            .values()
            .find(|binding| binding.node.viewport_action != 0)
            .ok_or("missing viewport action")?
            .node
            .viewport_action;
        actor.dispatch(Message {
            action,
            value: blockui_abi::LocalPoint {
                x: width.parse().unwrap(),
                y: 720.0,
            }
            .to_wire()
            .ok_or("invalid viewport")?,
        })?;
    }
    let select: u64 = args[2].parse().unwrap();
    for page in 0..7 {
        actor.dispatch(Message {
            action: select,
            value: page,
        })?;
        if page == 5 && args.get(4).is_some_and(|mode| mode == "editing") {
            actor.dispatch(Message {
                action: identities::seven_action("circle.add"),
                value: abi::LocalPoint { x: 100.0, y: 100.0 }.to_wire().unwrap(),
            })?;
            actor.dispatch(Message {
                action: identities::seven_action("circle.edit"),
                value: 0,
            })?;
        }
        for (index, key) in actor.view.order.iter().enumerate() {
            let b = &actor.view.bindings[key];
            println!(
                "{page}\t{index}\t{key}\t{}\t{}\t{}\t{}",
                b.node.kind,
                b.node.slot,
                b.node.action,
                b.text.replace('\n', "\\n")
            );
        }
    }
    Ok(())
}
