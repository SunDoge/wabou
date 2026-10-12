use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;

fn check(actor: &Actor, prefix: &str, count: i64) {
    assert_eq!(actor.artifact_count(), 1);
    #[cfg(target_os = "linux")]
    assert_eq!(actor.mapped_implementation_count().unwrap(), 1);
    assert_eq!(actor.integer("mapped.count"), count);
    assert_eq!(actor.float("mapped.ratio").to_bits(), (-0.0_f64).to_bits());
    assert_eq!(actor.text("mapped.title"), "你好 🌍");
    assert_eq!(actor.list("mapped.items"), &[3, 5]);
    for (name, value) in [
        ("count.text", count.to_string()),
        ("ratio.text", "-0".into()),
        ("enabled.text", "true".into()),
        ("title.text", "你好 🌍".into()),
        ("items.text", "[3, 5]".into()),
    ] {
        assert_eq!(
            actor.view.bindings[&key(name)].text,
            format!("{prefix}:{value}")
        );
    }
}

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 3 {
        return Err("mapped_values_proof base replacement rejected".into());
    }
    let mut actor = Actor::new(93, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let message = Message {
        action: key("Values.change"),
        value: 0,
    };
    actor.dispatch(message)?;
    check(&actor, "old", 1);
    let storage = actor.state_address();
    let names = [
        "mapped.count",
        "mapped.ratio",
        "mapped.enabled",
        "mapped.title",
        "mapped.items",
    ];
    let slots = names.map(|name| actor.slot_address(name));
    let schema = actor.schema_version;
    actor.enqueue(message)?;
    actor.reload(Path::new(&paths[1]))?;
    check(&actor, "new", 1);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(names.map(|name| actor.slot_address(name)), slots);
    assert_eq!(actor.schema_version, schema);
    actor.tick()?;
    check(&actor, "new", 11);
    let version = actor.implementation_version();
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert_eq!(actor.implementation_version(), version);
    check(&actor, "new", 11);
    actor.dispatch(message)?;
    check(&actor, "new", 21);
    assert_eq!(names.map(|name| actor.slot_address(name)), slots);
    // Repeated replacement exercises retired formatter tokens and copied DLL cleanup.
    for _ in 0..8 {
        actor.reload(Path::new(&paths[0]))?;
        check(&actor, "old", 21);
        actor.reload(Path::new(&paths[1]))?;
        check(&actor, "new", 21);
    }
    let mut typed = Actor::new(95, Path::new(".runtime"))?;
    typed.reload(Path::new(&paths[0]))?;
    let ratio_slot = typed.slot_address("mapped.ratio");
    let typed_storage = typed.state_address();
    let bits = [
        (-0.0_f64).to_bits(),
        0xfff8_0000_0000_1234,
        f64::INFINITY.to_bits(),
        1,
    ];
    for (index, bits) in bits.into_iter().enumerate() {
        typed.enqueue(Message {
            action: key("Values.sample"),
            value: bits as i64,
        })?;
        let version = index % 2;
        typed.reload(Path::new(&paths[version]))?;
        typed.tick()?;
        assert_eq!(typed.float("mapped.ratio").to_bits(), bits);
        assert_eq!(typed.state_address(), typed_storage);
        assert_eq!(typed.slot_address("mapped.ratio"), ratio_slot);
        assert_eq!(typed.integer("mapped.count"), [1, 11, 12, 22][index]);
        assert_eq!(typed.artifact_count(), 1);
        #[cfg(target_os = "linux")]
        assert_eq!(typed.mapped_implementation_count()?, 1);
    }
    let before = typed.float("mapped.ratio").to_bits();
    assert!(
        typed
            .dispatch(Message {
                action: key("Values.sample"),
                value: 42.0_f64.to_bits() as i64
            })
            .is_err()
    );
    assert_eq!(typed.float("mapped.ratio").to_bits(), before);
    assert_eq!(typed.integer("mapped.count"), 22);
    for value in [1, 0, 1] {
        typed.enqueue(Message {
            action: key("Values.toggle"),
            value,
        })?;
        typed.reload(Path::new(&paths[(value as usize) % 2]))?;
        typed.tick()?;
        assert!(
            typed.view.bindings[&key("enabled.text")]
                .text
                .ends_with(if value == 1 { ":true" } else { ":false" })
        );
    }
    assert!(
        typed
            .dispatch(Message {
                action: key("Values.toggle"),
                value: 2
            })
            .is_err()
    );
    assert!(
        typed.view.bindings[&key("enabled.text")]
            .text
            .ends_with(":true")
    );
    println!(
        "PASS actual DLL typed receiver actions: exact float bits, queued replacement, changed code, rollback and boolean validation"
    );
    println!(
        "PASS actual DLL: all five projections, retained slots/schema, queued action, rejected kind, repeated formatter replacement"
    );
    Ok(())
}
