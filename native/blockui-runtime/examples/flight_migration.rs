use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 2 {
        return Err("flight_migration legacy current".into());
    }
    for value in [0, 1] {
        let mut actor = Actor::new(138 + value as u64, Path::new(".runtime"))?;
        actor.reload(Path::new(&paths[0]))?;
        if value == 1 {
            actor.dispatch_widget(key("flight.trip.input"))?;
        }
        actor.edit_text(key("flight.departure.input"), "2026-")?;
        actor.edit_text(key("flight.return.input"), "invalid 🦀")?;
        let state = actor.state_address();
        let slot = actor.slot_address("flight.departure");
        let buffer = actor.text("flight.departure").as_ptr();
        actor.reload(Path::new(&paths[1]))?;
        assert_eq!(actor.state_address(), state);
        assert_eq!(actor.slot_address("flight.departure"), slot);
        assert_eq!(actor.text("flight.departure").as_ptr(), buffer);
        assert_eq!(actor.text("flight.departure"), "2026-");
        assert_eq!(actor.text("flight.return"), "invalid 🦀");
        assert_eq!(actor.integer("flight.trip_kind"), value);
        assert_eq!(actor.boolean("flight.return_trip"), value == 1);
        assert_eq!(
            actor.view.bindings[&key("flight.trip.input")].node.kind,
            blockui_abi::SELECT
        );
        assert!(actor.edit_select(key("flight.trip.input"), 2).is_err());
        actor.edit_select(key("flight.trip.input"), 1 - value)?;
        assert_eq!(actor.integer("flight.trip_kind"), 1 - value);
        assert_eq!(actor.boolean("flight.return_trip"), value == 0);
        actor.edit_text(key("flight.departure.input"), "2026-09-01")?;
        actor.edit_select(key("flight.trip.input"), 1)?;
        actor.edit_text(key("flight.return.input"), "2026-09-08")?;
        actor.dispatch(Message {
            action: key("flight.book"),
            value: 0,
        })?;
        assert_eq!(
            actor.text("flight.confirmation"),
            "Return flight booked: 2026-09-01 to 2026-09-08."
        );
    }
    println!(
        "PASS legacy Flight checkbox migration retains both trip modes and date drafts/state/slot/buffer; current Select actions and booking remain usable"
    );
    Ok(())
}
