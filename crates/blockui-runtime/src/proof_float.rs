use blockui_abi::key;
use blockui_runtime::{Actor, ActorId, Message, Runtime, Value};
use std::path::Path;
fn action(actor: &mut Actor, name: &str, value: f64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(name),
        value: value.to_bits() as i64,
    })
}
pub fn run(paths: &[String]) -> Result<(), String> {
    if paths.len() != 8 {
        return Err(
            "float-proof base replacement integer failed-migration failed-view write-in-computed float-migration changed-default"
                .into(),
        );
    }
    let mut actor = Actor::new(941, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.float("amount"), 0.25);
    assert_eq!(actor.float("zero").to_bits(), 0.0_f64.to_bits());
    let storage = actor.state_address();
    let slot = actor.slot_address("amount");
    actor.set_text(key("notes"), "a😀bc unsaved")?;
    let bucket = actor.view.bindings[&key("bucket")].evaluations;
    let bucket_compute = actor.computed_evaluations("bucket");
    action(&mut actor, "bits", 0.5)?;
    assert_eq!(actor.view.bindings[&key("bucket")].evaluations, bucket);
    assert_eq!(actor.computed_evaluations("bucket"), bucket_compute + 1);
    assert_eq!(actor.view.bindings[&key("bucket")].text, "Bucket: 0");
    for value in [
        1.25,
        -0.0,
        0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(1),
    ] {
        action(&mut actor, "bits", value)?;
        assert_eq!(actor.float("amount").to_bits(), value.to_bits());
        let evaluations = actor.computed_evaluations("double");
        action(&mut actor, "bits", value)?;
        assert_eq!(
            actor.computed_evaluations("double"),
            evaluations,
            "equal float bits, including NaN, must not invalidate reactive caches"
        );
        let text = actor.view.bindings[&key("caption")].text.clone();
        assert!(action(&mut actor, "fail", 42.0).is_err());
        assert_eq!(actor.float("amount").to_bits(), value.to_bits());
        assert_eq!(actor.view.bindings[&key("caption")].text, text);
    }
    for name in ["wrong_read", "wrong_write", "wrong_float", "previous"] {
        assert_eq!(
            actor.dispatch_request(Message {
                action: key(name),
                value: 0
            })?,
            Some(1),
            "registered callback must observe host type/scope rejection"
        );
    }
    // Compatible replacement must preserve raw storage, even where numeric equality
    // is unsuitable (NaN payloads and signed zero). Each failed candidate must also
    // roll back its staged graph without changing the retained slot.
    for value in [
        -0.0,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0xfff8_0000_0000_1234),
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(1),
    ] {
        action(&mut actor, "bits", value)?;
        let schema = actor.schema_version;
        let implementation = actor.implementation_version();
        assert!(actor.reload(Path::new(&paths[4])).is_err());
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(actor.float("amount").to_bits(), value.to_bits());
        for path in [&paths[1], &paths[0]] {
            actor.reload(Path::new(path))?;
            assert_eq!(actor.schema_version, schema);
            assert_eq!(actor.float("amount").to_bits(), value.to_bits());
            assert_eq!(actor.state_address(), storage);
            assert_eq!(actor.slot_address("amount"), slot);
            assert_eq!(actor.text("notes"), "a😀bc unsaved");
            assert_eq!(actor.artifact_count(), 1);
            assert_eq!(actor.mapped_implementation_count()?, 1);
        }
    }
    action(&mut actor, "bits", 0.5)?;
    actor.enqueue(Message {
        action: key("bits"),
        value: 0.875_f64.to_bits() as i64,
    })?;
    let schema = actor.schema_version;
    assert!(format!("{}", actor.preview_reload(Path::new(&paths[1]))?).contains("kind=f64"));
    for path in [&paths[4], &paths[5]] {
        assert!(actor.reload(Path::new(path)).is_err());
        assert_eq!(actor.float("amount"), 0.5);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.slot_address("amount"), slot);
        assert_eq!(actor.artifact_count(), 1);
    }
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("amount"), slot);
    assert_eq!(actor.float("amount"), 0.5);
    assert_eq!(actor.computed("double"), Some(&Value::Float(1.5)));
    actor.tick()?;
    assert_eq!(actor.float("amount"), 0.875);
    assert_eq!(actor.computed("double"), Some(&Value::Float(2.625)));
    assert_eq!(
        actor.view.bindings[&key("caption")].text,
        "Amount: 0.875 / 2.625"
    );
    assert_eq!(actor.text("notes"), "a😀bc unsaved");
    assert_eq!(actor.artifact_count(), 1);
    assert_eq!(actor.mapped_implementation_count()?, 1);
    let previous_code = actor.implementation_version();
    let preview = actor.preview_reload(Path::new(&paths[7]))?;
    assert!(preview.compatible);
    assert_eq!(preview.schema_version, schema);
    actor.reload(Path::new(&paths[7]))?;
    assert_ne!(actor.implementation_version(), previous_code);
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.float("amount"), 0.875);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("amount"), slot);
    assert_eq!(actor.text("notes"), "a😀bc unsaved");
    {
        let mut fresh = Actor::new(943, Path::new(".runtime"))?;
        fresh.reload(Path::new(&paths[7]))?;
        assert_eq!(fresh.schema_version, schema);
        assert_eq!(fresh.float("amount").to_bits(), (-0.0_f64).to_bits());
    }
    actor.reload(Path::new(&paths[6]))?;
    assert_eq!(actor.float("amount"), 1.0);
    assert_eq!(actor.float("extra"), 7.5);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("amount"), slot);
    assert_eq!(actor.text("notes"), "a😀bc unsaved");
    let mut migration = Actor::new(942, Path::new(".runtime"))?;
    migration.reload(Path::new(&paths[2]))?;
    migration.set_text(key("notes"), "kept migration draft")?;
    let notes = migration.slot_address("notes");
    assert_eq!(migration.integer("amount"), 4);
    assert!(
        format!("{}", migration.preview_reload(Path::new(&paths[0]))?).contains("from=i64 to=f64")
    );
    assert!(migration.reload(Path::new(&paths[3])).is_err());
    assert_eq!(migration.integer("amount"), 4);
    migration.reload(Path::new(&paths[0]))?;
    assert_eq!(migration.float("amount"), 1.0);
    assert_eq!(migration.computed("double"), Some(&Value::Float(2.0)));
    assert_eq!(migration.slot_address("notes"), notes);
    assert_eq!(migration.text("notes"), "kept migration draft");
    assert_eq!(migration.artifact_count(), 1);
    {
        let mut runtime = Runtime::new(".runtime");
        runtime.spawn(ActorId(1), Path::new(&paths[0]))?;
        runtime.spawn(ActorId(2), Path::new(&paths[0]))?;
        let storage = runtime.actor(ActorId(2)).unwrap().state_address();
        let slot = runtime.actor(ActorId(2)).unwrap().slot_address("amount");
        for bits in [
            0_u64,
            1_u64 << 63,
            1,
            1.25_f64.to_bits(),
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            0x7ff8_0000_0000_0042,
            0xfff8_0000_0000_0043,
        ] {
            runtime.tell(
                ActorId(1),
                Message {
                    action: key("forward_float"),
                    value: bits as i64,
                },
            )?;
            assert!(runtime.tick().is_empty()); // Source commits its typed outbox.
            runtime.reload(ActorId(1), Path::new(&paths[1]))?;
            runtime.reload(ActorId(2), Path::new(&paths[1]))?;
            assert!(runtime.tick().is_empty()); // Target receives owned bits through current code.
            let target = runtime.actor(ActorId(2)).unwrap();
            assert_eq!(target.float("amount").to_bits(), bits);
            assert_eq!(target.float("zero"), 2.0);
            assert_eq!(target.state_address(), storage);
            assert_eq!(target.slot_address("amount"), slot);
        }
        let before = runtime.actor(ActorId(2)).unwrap().float("amount").to_bits();
        runtime.tell(
            ActorId(1),
            Message {
                action: key("reject_float"),
                value: 99.0_f64.to_bits() as i64,
            },
        )?;
        assert_eq!(runtime.tick().len(), 1);
        assert!(runtime.tick().is_empty());
        assert_eq!(
            runtime.actor(ActorId(2)).unwrap().float("amount").to_bits(),
            before
        );
        println!(
            "PASS SDK FloatAction routing preserves IEEE bits and state allocation across source/target replacement, invokes new code and rolls back rejected source turns"
        );
    }
    {
        use std::time::{Duration, Instant};
        let patterns = [
            0_u64,
            1_u64 << 63,
            1,
            1.25_f64.to_bits(),
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            0x7ff8_0000_0000_0042,
            0xfff8_0000_0000_0043,
        ];
        for (index, bits) in patterns.into_iter().enumerate() {
            let mut timer = Actor::new(970 + index as u64, Path::new(".runtime"))?;
            timer.reload(Path::new(&paths[0]))?;
            let storage = timer.state_address();
            let slot = timer.slot_address("amount");
            timer.dispatch(Message {
                action: key("start_float_delay"),
                value: bits as i64,
            })?;
            let task = key("float.delay");
            let generation = timer.tasks[&task].generation();
            let worker = timer.tasks[&task].worker_id();
            timer.reload(Path::new(&paths[1]))?;
            assert_eq!(timer.tasks[&task].generation(), generation);
            assert_eq!(timer.tasks[&task].worker_id(), worker);
            let deadline = Instant::now() + Duration::from_secs(2);
            while timer.float("zero") != 2.0 {
                timer.tick()?;
                assert!(Instant::now() < deadline, "typed float delay timed out");
                std::thread::sleep(Duration::from_millis(2));
            }
            assert_eq!(timer.float("amount").to_bits(), bits);
            assert_eq!(timer.state_address(), storage);
            assert_eq!(timer.slot_address("amount"), slot);
            assert!(timer.tasks.is_empty());
        }
        let mut periodic = Actor::new(980, Path::new(".runtime"))?;
        periodic.reload(Path::new(&paths[0]))?;
        let bits = (-0.0_f64).to_bits();
        periodic.dispatch(Message {
            action: key("start_float_interval"),
            value: bits as i64,
        })?;
        let task = key("float.interval");
        let generation = periodic.tasks[&task].generation();
        let worker = periodic.tasks[&task].worker_id();
        assert!(periodic.reload(Path::new(&paths[4])).is_err());
        let deadline = Instant::now() + Duration::from_secs(2);
        while periodic.float("zero") != 1.0 {
            periodic.tick()?;
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(periodic.float("amount").to_bits(), bits);
        periodic.reload(Path::new(&paths[1]))?;
        assert_eq!(periodic.tasks[&task].generation(), generation);
        assert_eq!(periodic.tasks[&task].worker_id(), worker);
        let deadline = Instant::now() + Duration::from_secs(2);
        while periodic.float("zero") != 2.0 {
            periodic.tick()?;
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(periodic.float("amount").to_bits(), bits);
        periodic.dispatch(Message {
            action: key("stop_float_interval"),
            value: 0,
        })?;
        assert!(periodic.tasks.is_empty());
        action(&mut periodic, "bits", 123.0)?;
        std::thread::sleep(Duration::from_millis(40)); // Cross two authored interval periods.
        periodic.tick()?;
        assert_eq!(periodic.float("amount"), 123.0);
        println!(
            "PASS typed FloatAction one-shot and interval payloads preserve exact IEEE bits, task generation/worker and current callback across accepted/rejected DLL replacement; cancellation prevents later periodic delivery"
        );
    }
    {
        let mut runtime = Runtime::new(".runtime");
        runtime.spawn(ActorId(1), Path::new(&paths[0]))?;
        runtime.spawn(ActorId(2), Path::new(&paths[0]))?;
        let storage = runtime.actor(ActorId(1)).unwrap().state_address();
        for bits in [
            0_u64,
            1_u64 << 63,
            1,
            1.25_f64.to_bits(),
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            0x7ff8_0000_0000_0042,
            0xfff8_0000_0000_0043,
        ] {
            runtime.tell(
                ActorId(1),
                Message {
                    action: key("ask_float"),
                    value: bits as i64,
                },
            )?;
            assert!(runtime.tick().is_empty());
            runtime.reload(ActorId(1), Path::new(&paths[1]))?;
            runtime.reload(ActorId(2), Path::new(&paths[1]))?;
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(
                runtime.actor(ActorId(2)).unwrap().float("amount").to_bits(),
                bits
            );
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().float("amount").to_bits(),
                bits
            );
            assert_eq!(runtime.actor(ActorId(1)).unwrap().float("zero"), 2.0);
            assert_eq!(runtime.actor(ActorId(1)).unwrap().state_address(), storage);
        }
        let before = runtime.actor(ActorId(2)).unwrap().float("amount").to_bits();
        runtime.tell(
            ActorId(2),
            Message {
                action: key("query_float"),
                value: 99.0_f64.to_bits() as i64,
            },
        )?;
        assert_eq!(runtime.tick().len(), 1); // A request handler requires reply scope.
        assert_eq!(
            runtime.actor(ActorId(2)).unwrap().float("amount").to_bits(),
            before
        );
        println!(
            "PASS typed float ask input and reply preserve IEEE bits across both actor replacements with current-code completion"
        );
    }
    {
        let mut runtime = Runtime::new(".runtime");
        for id in [ActorId(1), ActorId(2)] {
            runtime.spawn(id, Path::new(&paths[0]))?;
        }
        let allocations: Vec<_> = [ActorId(1), ActorId(2)]
            .iter()
            .map(|id| {
                let actor = runtime.actor(*id).unwrap();
                (actor.state_address(), actor.slot_address("amount"))
            })
            .collect();
        let cases = [("ask_bool_float", 0_i64), ("ask_bool_float", 1)]
            .into_iter()
            .chain(
                [
                    0_u64,
                    1_u64 << 63,
                    1,
                    1.25_f64.to_bits(),
                    f64::INFINITY.to_bits(),
                    f64::NEG_INFINITY.to_bits(),
                    0x7ff8_0000_0000_0042,
                    0xfff8_0000_0000_0043,
                ]
                .into_iter()
                .map(|bits| ("ask_float_bool", bits as i64)),
            );
        for (action, value) in cases {
            for id in [ActorId(1), ActorId(2)] {
                runtime.reload(id, Path::new(&paths[0]))?;
            }
            runtime.tell(
                ActorId(1),
                Message {
                    action: key(action),
                    value,
                },
            )?;
            assert!(runtime.tick().is_empty());
            for id in [ActorId(1), ActorId(2)] {
                runtime.reload(id, Path::new(&paths[1]))?;
            }
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            let source = runtime.actor(ActorId(1)).unwrap();
            let target = runtime.actor(ActorId(2)).unwrap();
            if action == "ask_bool_float" {
                let expected = if value == 0 {
                    1_u64 << 63
                } else {
                    0x7ff8_0000_0000_0042
                };
                assert_eq!(source.float("amount").to_bits(), expected);
                assert_eq!(target.float("amount").to_bits(), expected);
                assert_eq!(source.float("zero"), 2.0);
            } else {
                let negative = f64::from_bits(value as u64).is_sign_negative();
                assert_eq!(source.float("amount"), if negative { 1.0 } else { 0.0 });
                assert_eq!(target.float("amount").to_bits(), value as u64);
                assert_eq!(source.float("zero"), if negative { 21.0 } else { 20.0 });
            }
            assert_eq!(target.float("zero"), 200.0);
            for (index, id) in [ActorId(1), ActorId(2)].iter().enumerate() {
                let actor = runtime.actor(*id).unwrap();
                assert_eq!(
                    (actor.state_address(), actor.slot_address("amount")),
                    allocations[index]
                );
            }
        }
        let before = runtime.actor(ActorId(2)).unwrap().float("amount").to_bits();
        for invalid in [2_i64, -1, i64::MAX, 0] {
            runtime.tell(
                ActorId(2),
                Message {
                    action: key("query_bool_float"),
                    value: invalid,
                },
            )?;
            assert_eq!(runtime.tick().len(), 1);
            let target = runtime.actor(ActorId(2)).unwrap();
            assert_eq!(target.float("amount").to_bits(), before);
            assert_eq!(target.float("zero"), 200.0);
        }
        println!(
            "PASS mixed bool/float request input and reply types across both DLL replacements, exact IEEE bits, current handlers, allocation retention and malformed/tell rollback"
        );
    }
    println!(
        "PASS typed float state/computed and actor defaults, exact IEEE bits including NaN/infinity/signed zero, equal-bit memoization, turn/read-only/candidate rollback, compatible state allocation/mailbox retention and explicit integer-to-float migration through real DLLs"
    );
    Ok(())
}
