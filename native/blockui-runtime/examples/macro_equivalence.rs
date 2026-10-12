//! The declaration macros are optional: both forms share the same durable schema.
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(paths.len(), 4);
    for (case, pair) in paths.as_chunks::<2>().0.iter().enumerate() {
        let mut actor = Actor::new(135 + case as u64, Path::new(".runtime"))?;
        actor.reload(Path::new(&pair[0]))?;
        let state = actor.state_address();
        let slot = actor.slot_address("bench.count");
        let schema = actor.schema_version;
        let mut expected = 0;
        for round in 0..12 {
            actor.reload(Path::new(&pair[round % 2]))?;
            assert_eq!(actor.state_address(), state);
            assert_eq!(actor.slot_address("bench.count"), slot);
            assert_eq!(actor.schema_version, schema);
            assert_eq!(actor.integer("bench.count"), expected);
            actor.dispatch(Message {
                action: blockui_abi::key("bench.step_0"),
                value: round as i64,
            })?;
            expected = round as i64 + 8;
            assert_eq!(actor.integer("bench.count"), expected);
            assert_eq!(
                actor
                    .view
                    .bindings
                    .values()
                    .find(|binding| !binding.text.is_empty())
                    .unwrap()
                    .text,
                expected.to_string()
            );
            if case == 1 {
                actor.dispatch(Message {
                    action: blockui_abi::key("bench.step_63"),
                    value: round as i64,
                })?;
                expected = round as i64 + 63;
                assert_eq!(actor.integer("bench.count"), expected);
                assert_eq!(
                    actor
                        .view
                        .bindings
                        .values()
                        .find(|binding| !binding.text.is_empty())
                        .unwrap()
                        .text,
                    expected.to_string()
                );
            }
            assert_eq!(actor.artifact_count(), 1);
            assert_eq!(actor.mapped_implementation_count()?, 1);
        }
    }
    println!(
        "PASS plain/macro actual DLL equivalence: stable schema/state slots, current actions/views and bounded mappings across 24 replacements"
    );
    Ok(())
}
