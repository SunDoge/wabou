use blockui_abi::key;
mod proof_bindings;
#[cfg(unix)]
mod proof_create;
mod proof_float;
mod proof_graph;
mod proof_interval;
#[cfg(unix)]
mod proof_io;
mod proof_migration;
#[cfg(unix)]
mod proof_resources;
use blockui_runtime::{Actor, Message};
use std::{
    io::{self, BufRead, Write},
    path::Path,
    time::Duration,
};
fn status(actor: &Actor) {
    println!(
        "id={} count={} state={} slot={} impl={:016x} schema={:016x} tasks={} resources={} mailbox={} artifacts={}",
        actor.id,
        actor.integer("count"),
        actor.state_address(),
        actor.slot_address("count"),
        actor.implementation_version(),
        actor.schema_version,
        actor.tasks.len(),
        actor.socket_count(),
        actor.mailbox.len(),
        actor.artifact_count()
    );
    io::stdout().flush().unwrap();
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    #[cfg(unix)]
    if args.get(1).map(String::as_str) == Some("create-proof") {
        return proof_create::run(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("binding-bench") {
        let path = Path::new(args.get(2).ok_or("binding-bench <plugin.so> <rounds>")?);
        let rounds = args
            .get(3)
            .ok_or("missing benchmark rounds")?
            .parse()
            .map_err(|_| "invalid benchmark rounds")?;
        return proof_bindings::benchmark(path, rounds);
    }
    if args.get(1).map(String::as_str) == Some("float-proof") {
        return proof_float::run(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("binding-proof") {
        return proof_bindings::run(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("interval-proof") {
        return proof_interval::run(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("seven-guis-proof") {
        #[cfg(not(feature = "native-window"))]
        return Err("this proof requires native".into());
        #[cfg(feature = "native-window")]
        {
            let mut paths = args[2..].to_vec();
            let window = paths.last().map(String::as_str) == Some("--window");
            if window {
                paths.pop();
            }
            return blockui_runtime::native::seven_guis_proof(&paths, window);
        }
    }
    if args.get(1).map(String::as_str) == Some("request-feedback-proof") {
        #[cfg(not(feature = "native-window"))]
        return Err("this proof requires native".into());
        #[cfg(feature = "native-window")]
        {
            let mut paths = args[2..].to_vec();
            let window = paths.last().map(String::as_str) == Some("--window");
            if window {
                paths.pop();
            }
            return blockui_runtime::native::request_feedback_proof(&paths, window);
        }
    }
    if args.get(1).map(String::as_str) == Some("button-arguments-proof") {
        #[cfg(not(feature = "native-window"))]
        return Err("this proof requires the native feature".into());
        #[cfg(feature = "native-window")]
        return blockui_runtime::native::button_arguments_proof(&args[2..]);
    }
    if matches!(
        args.get(1).map(String::as_str),
        Some("checkbox-proof" | "read-only-proof")
    ) {
        #[cfg(not(feature = "native-window"))]
        return Err("this proof requires the native feature".into());
        #[cfg(feature = "native-window")]
        {
            let mut paths = args[2..].to_vec();
            let capture = paths.last().map(String::as_str) == Some("--window");
            if capture {
                paths.pop();
            }
            return if args[1] == "checkbox-proof" {
                blockui_runtime::native::checkbox_proof(&paths, capture)
            } else {
                blockui_runtime::native::read_only_proof(&paths, capture)
            };
        }
    }
    if args.get(1).map(String::as_str) == Some("verify") {
        return verify(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("view-proof") {
        return view_proof(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("graph-proof") {
        return proof_graph::run(&args[2..]);
    }
    #[cfg(unix)]
    if args.get(1).map(String::as_str) == Some("resource-proof") {
        return proof_resources::run(&args[2..]);
    }
    #[cfg(unix)]
    if args.get(1).map(String::as_str) == Some("io-proof") {
        return proof_io::run(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("migration-proof") {
        return proof_migration::run(&args[2..]);
    }
    #[cfg(feature = "native-window")]
    if args.get(1).map(String::as_str) == Some("family-proof") {
        return blockui_runtime::native::family_proof(
            Path::new(args.get(2).ok_or("family-proof needs base library")?),
            Path::new(
                args.get(3)
                    .ok_or("family-proof needs replacement library")?,
            ),
            args.get(4).map(Path::new),
        );
    }
    #[cfg(feature = "native-window")]
    if matches!(
        args.get(1).map(String::as_str),
        Some("family-gui" | "family-capture")
    ) {
        use blockui_runtime::{ActorId, Runtime, native::NativeSource, native_model::NativeFamily};
        let mut runtime = Runtime::new(".runtime");
        let parent = ActorId(100);
        runtime.spawn(
            parent,
            Path::new(args.get(2).ok_or("missing parent implementation")?),
        )?;
        runtime.spawn_child(
            parent,
            ActorId(101),
            Path::new(args.get(3).ok_or("missing child implementation")?),
        )?;
        let source = NativeSource::without_watch(NativeFamily::new(runtime, parent)?)?;
        let source = if args[1] == "family-capture" {
            source.capture()
        } else {
            source
        };
        return wabou_shell_vello::run_window_with_options(
            Box::new(source),
            wabou_shell_vello::WindowOptions::new()
                .title("BlockUI — Actor Family")
                .initial_inner_size(720, 1000),
        )
        .map_err(|error| error.to_string());
    }
    #[cfg(feature = "native-window")]
    if matches!(
        args.get(1).map(String::as_str),
        Some("family-watch-proof" | "shared-watch-proof")
    ) {
        let repo = std::env::current_dir().map_err(|e| e.to_string())?;
        let config = |index| {
            blockui_runtime::native::ReloadConfig::from_file(
                Path::new(args.get(index).ok_or("missing watch config")?),
                &repo,
            )
        };
        return if args[1] == "shared-watch-proof" {
            blockui_runtime::native::shared_watch_proof(config(2)?, config(3)?)
        } else {
            blockui_runtime::native::family_watch_proof(config(2)?, config(3)?)
        };
    }
    #[cfg(feature = "native-window")]
    if matches!(
        args.get(1).map(String::as_str),
        Some("family-dev" | "family-dev-capture")
    ) {
        use blockui_runtime::{
            ActorId, Runtime,
            native::{NativeSource, ReloadConfig},
            native_model::NativeFamily,
        };
        let repo = std::env::current_dir().map_err(|e| e.to_string())?;
        let parent_config = ReloadConfig::from_file(
            Path::new(args.get(2).ok_or("missing parent config")?),
            &repo,
        )?;
        let child_config =
            ReloadConfig::from_file(Path::new(args.get(3).ok_or("missing child config")?), &repo)?;
        let mut runtime = Runtime::new(repo.join(".runtime"));
        let parent = ActorId(100);
        let child = ActorId(101);
        runtime.spawn(
            parent,
            Path::new(args.get(4).ok_or("missing initial parent snapshot")?),
        )?;
        runtime.spawn_child(
            parent,
            child,
            Path::new(args.get(5).ok_or("missing initial child snapshot")?),
        )?;
        let mut source = NativeSource::without_watch(NativeFamily::new(runtime, parent)?)?;
        source.watch_actor(parent, parent_config)?;
        source.watch_actor(child, child_config)?;
        let source = if args[1] == "family-dev-capture" {
            source.capture()
        } else {
            source
        };
        return wabou_shell_vello::run_window_with_options(
            Box::new(source),
            wabou_shell_vello::WindowOptions::new()
                .title("BlockUI — Live Actor Family")
                .initial_inner_size(720, 1000),
        )
        .map_err(|e| e.to_string());
    }
    if args.get(1).map(String::as_str) == Some("admission-proof") {
        let path = |index| {
            args.get(index)
                .map(Path::new)
                .ok_or("admission-proof needs base, replacement and migration libraries")
        };
        return blockui_runtime::admission_proof(
            path(2)?,
            path(3)?,
            path(4)?,
            args.get(5).map(Path::new),
        );
    }
    #[cfg(feature = "native-window")]
    if args.get(1).map(String::as_str) == Some("declared-proof") {
        let path = |index| {
            args.get(index).map(Path::new).ok_or("declared-proof needs parent base/replacement/rejected, child and recursive libraries")
        };
        blockui_runtime::native::declared_children_proof(
            path(2)?,
            path(3)?,
            path(4)?,
            path(5)?,
            path(6)?,
            path(7)?,
            path(8)?,
        )?;
        blockui_runtime::native::declared_row_proof(path(2)?, path(13)?, path(5)?)?;
        blockui_runtime::native::accepted_children_proof(path(2)?, path(5)?, path(9)?, path(10)?)?;
        blockui_runtime::native::group_children_proof(path(2)?, path(11)?, path(5)?)?;
        return blockui_runtime::native::child_routing_proof(
            path(2)?,
            path(3)?,
            path(5)?,
            path(12)?,
        );
    }
    #[cfg(feature = "native-window")]
    if matches!(
        args.get(1).map(String::as_str),
        Some("declared-gui" | "declared-capture" | "declared-dev")
    ) {
        use blockui_runtime::{ActorId, Runtime, native::NativeSource, native_model::NativeFamily};
        let mut runtime = Runtime::new(".runtime");
        runtime.register_implementation(
            "counter",
            Path::new(args.get(3).ok_or("missing counter implementation")?),
        )?;
        runtime.spawn(
            ActorId(42),
            Path::new(args.get(2).ok_or("missing parent implementation")?),
        )?;
        let mut source = NativeSource::without_watch(NativeFamily::new(runtime, ActorId(42))?)?;
        if args[1] == "declared-dev" {
            use blockui_runtime::native::ReloadConfig;
            let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap();
            let parent_config = ReloadConfig::from_file(
                Path::new(args.get(4).ok_or("missing parent watch config")?),
                repo,
            )?;
            let child_config = ReloadConfig::from_file(
                Path::new(args.get(5).ok_or("missing child watch config")?),
                repo,
            )?;
            source.watch_actor(ActorId(42), parent_config)?;
            source.watch_implementation("counter", child_config)?;
        }
        let source = if args[1] == "declared-capture" {
            source.capture()
        } else {
            source
        };
        return wabou_shell_vello::run_window_with_options(
            Box::new(source),
            wabou_shell_vello::WindowOptions::new()
                .title("BlockUI — Declared Actors")
                .initial_inner_size(720, 1200),
        )
        .map_err(|e| e.to_string());
    }
    #[cfg(feature = "native-window")]
    if args.get(1).map(String::as_str) == Some("implementation-watch-proof") {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let config = blockui_runtime::native::ReloadConfig::from_file(
            Path::new(args.get(3).ok_or("missing implementation config")?),
            repo,
        )?;
        return blockui_runtime::native::implementation_watch_proof(
            Path::new(args.get(2).ok_or("missing parent DLL")?),
            config,
        );
    }
    #[cfg(feature = "native-window")]
    if args.get(1).map(String::as_str) == Some("supersession-proof") {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let config = blockui_runtime::native::ReloadConfig::from_file(
            Path::new(args.get(3).ok_or("missing implementation config")?),
            repo,
        )?;
        return blockui_runtime::native::supersession_proof(
            Path::new(args.get(2).ok_or("missing parent DLL")?),
            config,
        );
    }
    let path = args.get(2).ok_or("usage: blockui-runtime run|gui <plugin.so> [repo] | verify <v1> <v2> <auto> <explicit> <reject> <abi>")?;
    let mut actor = Actor::new(42, Path::new(".runtime"))?;
    actor.reload(Path::new(path))?;
    if args[1] == "check" {
        let candidate = args.get(3).ok_or("check <current.so> <candidate.so>")?;
        print!("{}", actor.preview_reload(Path::new(candidate))?);
        actor.check_reload(Path::new(candidate))?;
        println!("candidate preparation succeeded; current implementation retained");
        return Ok(());
    }
    if args[1] == "preview" {
        let candidate = args.get(3).ok_or("preview <current.so> <candidate.so>")?;
        print!("{}", actor.preview_reload(Path::new(candidate))?);
        return Ok(());
    }

    if args[1] == "typed-proof" {
        if args.len() != 9 {
            return Err(
                "typed-proof needs base, replacement, action, slot, initial, first, second".into(),
            );
        }
        let slot = &args[5];
        let address = actor.slot_address(slot);
        let storage = actor.state_address();
        assert_eq!(actor.text(slot), args[6]);
        actor.dispatch(Message {
            action: key(&args[4]),
            value: 0,
        })?;
        assert_eq!(actor.text(slot), args[7]);
        actor.reload(Path::new(&args[3]))?;
        assert_eq!(actor.text(slot), args[7]);
        assert_eq!(actor.slot_address(slot), address);
        assert_eq!(actor.state_address(), storage);
        actor.dispatch(Message {
            action: key(&args[4]),
            value: 0,
        })?;
        assert_eq!(actor.text(slot), args[8]);
        println!(
            "PASS independent typed application, string default and action, same state identity through code replacement"
        );
        return Ok(());
    }
    if args[1] == "boolean-task-proof" {
        let address = actor.slot_address("document.active");
        actor.dispatch(Message {
            action: key("boolean_timer"),
            value: 0,
        })?;
        // Wait for the host worker, leaving its completion unaccepted across replacement.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !actor.tasks[&key("boolean_timer")].is_finished() {
            if std::time::Instant::now() > deadline {
                return Err("boolean timer timeout".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        actor.reload(Path::new(args.get(3).ok_or("missing replacement")?))?;
        actor.tick()?;
        assert!(!actor.boolean("document.active"));
        assert_eq!(actor.integer("document.count"), 105);
        assert_eq!(actor.slot_address("document.active"), address);
        assert!(actor.tasks.is_empty());
        actor.tick()?;
        assert_eq!(actor.integer("document.count"), 105);
        let mut runtime = blockui_runtime::Runtime::new(".runtime");
        let sender = blockui_runtime::ActorId(43);
        let receiver = blockui_runtime::ActorId(44);
        runtime.spawn(sender, Path::new(path))?;
        runtime.spawn(receiver, Path::new(path))?;
        runtime.tell(
            sender,
            Message {
                action: key("boolean_send"),
                value: 1,
            },
        )?;
        let faults = runtime.tick();
        assert_eq!(faults.len(), 1);
        assert_eq!(faults[0].actor, sender);
        assert!(runtime.tick().is_empty());
        assert!(runtime.actor(receiver).unwrap().boolean("document.active"));
        assert_eq!(
            runtime.actor(receiver).unwrap().integer("document.count"),
            5
        );
        runtime.tell(
            sender,
            Message {
                action: key("boolean_send"),
                value: 0,
            },
        )?;
        assert!(runtime.tick().is_empty());
        // The committed message is now in the receiver's durable mailbox.
        runtime.reload(
            receiver,
            Path::new(args.get(3).ok_or("missing replacement")?),
        )?;
        assert!(runtime.tick().is_empty());
        let receiver = runtime.actor(receiver).unwrap();
        assert!(!receiver.boolean("document.active"));
        assert_eq!(receiver.integer("document.count"), 105);
        println!(
            "PASS typed boolean timer retains buffered completion across replacement, invokes new code exactly once and preserves state identity; typed cross-actor send rolls back on failure and queued delivery uses new code"
        );
        return Ok(());
    }
    if args[1] == "declaration-proof" {
        let old_version = actor.implementation_version();
        let old_schema = actor.schema_version;
        let storage = actor.state_address();
        let base = actor.preview_reload(Path::new(path))?;
        assert!(base.compatible);
        assert!(!base.requires_migration());
        let extra = actor.preview_reload(Path::new(args.get(4).ok_or("missing extra schema")?))?;
        assert!(!extra.compatible);
        assert!(!extra.requires_migration());
        assert!(extra.changes.iter().any(|change| matches!(change, blockui_runtime::StateChange::Added { key: id, default: blockui_runtime::Value::Integer(7) } if *id == key("document.bonus"))));
        let integer =
            actor.preview_reload(Path::new(args.get(6).ok_or("missing integer schema")?))?;
        assert!(integer.requires_migration());
        assert!(integer.has_migration);
        assert!(integer.changes.iter().any(|change| matches!(change, blockui_runtime::StateChange::TypeChanged { key: id, from: blockui_abi::BOOLEAN, to: blockui_abi::I64, .. } if *id == key("document.active"))));
        assert!(
            actor
                .preview_reload(Path::new("/missing/preview.so"))
                .is_err()
        );
        let trap = Path::new(args.get(7).ok_or("missing preview migration trap")?);
        assert!(actor.preview_reload(trap)?.has_migration);
        assert!(actor.reload(trap).is_err());
        for _ in 0..40 {
            assert!(actor.preview_reload(Path::new(path))?.compatible);
        }
        let snapshots =
            std::path::PathBuf::from(".runtime").join(format!("actor-42-{}", std::process::id()));
        assert_eq!(
            std::fs::read_dir(snapshots)
                .map_err(|e| e.to_string())?
                .count(),
            1
        );
        let mut extra_actor = Actor::new(45, Path::new(".runtime"))?;
        extra_actor.reload(Path::new(args.get(4).ok_or("missing extra schema")?))?;
        let removed = extra_actor.preview_reload(Path::new(path))?;
        assert!(removed.changes.iter().any(|change| matches!(change, blockui_runtime::StateChange::Removed { key: id, kind: blockui_abi::I64 } if *id == key("document.bonus"))));
        assert_eq!(extra_actor.integer("document.bonus"), 7);
        drop(extra_actor);
        assert_eq!(actor.implementation_version(), old_version);
        assert_eq!(actor.schema_version, old_schema);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.integer("document.count"), 5);
        let active = actor.slot_address("document.active");
        assert!(actor.boolean("document.active"));
        assert!(
            actor
                .view
                .bindings
                .contains_key(&key("document.active-label"))
        );
        assert!(
            actor
                .dispatch(Message {
                    action: key("set_active"),
                    value: 2
                })
                .is_err()
        );
        assert!(actor.boolean("document.active"));
        actor.dispatch(Message {
            action: key("set_active"),
            value: 0,
        })?;
        assert!(!actor.boolean("document.active"));
        actor.dispatch(Message {
            action: key("set_active"),
            value: 1,
        })?;
        assert!(actor.boolean("document.active"));
        let count = actor.slot_address("document.count");
        let name = actor.slot_address("document.name");
        assert_eq!(actor.integer("document.count"), 5);
        assert_eq!(actor.text("document.name"), "visitor");
        actor.dispatch(Message {
            action: key("append"),
            value: 0,
        })?;
        assert_eq!(actor.integer("document.count"), 6);
        assert!(!actor.boolean("document.active"));
        assert!(
            !actor
                .view
                .bindings
                .contains_key(&key("document.active-label"))
        );
        assert!(
            actor
                .dispatch(Message {
                    action: key("invalid_boolean"),
                    value: 0
                })
                .is_err()
        );
        assert_eq!(actor.integer("document.count"), 6);
        assert!(!actor.boolean("document.active"));
        actor.enqueue(Message {
            action: key("set_active"),
            value: 1,
        })?;
        actor.reload(Path::new(args.get(3).ok_or("missing replacement")?))?;
        actor.tick()?;
        assert!(actor.boolean("document.active"));
        actor.dispatch(Message {
            action: key("set_active"),
            value: 0,
        })?;
        actor.dispatch(Message {
            action: key("append"),
            value: 0,
        })?;
        assert_eq!(actor.integer("document.count"), 8);
        assert_eq!(actor.text("document.name"), "visitor!?");
        actor.reload(Path::new(args.get(4).ok_or("missing extra schema")?))?;
        assert_eq!(actor.integer("document.bonus"), 7);
        assert_eq!(actor.integer("document.count"), 8);
        assert_eq!(actor.slot_address("document.count"), count);
        assert_eq!(actor.slot_address("document.name"), name);
        actor.reload(Path::new(args.get(5).ok_or("missing reordered schema")?))?;
        assert_eq!(actor.integer("document.count"), 8);
        assert_eq!(actor.text("document.name"), "visitor!?");
        assert_eq!(actor.slot_address("document.count"), count);
        assert_eq!(actor.slot_address("document.name"), name);
        assert_eq!(actor.slot_address("document.bonus"), 0);
        assert!(actor.boolean("document.active"));
        assert_eq!(actor.slot_address("document.active"), active);
        assert!(
            actor
                .view
                .bindings
                .contains_key(&key("document.active-label"))
        );
        let memo = actor.computed("document.is_active").unwrap() as *const blockui_runtime::Value;
        let evaluations = actor.computed_evaluations("document.is_active");
        let root_evaluations = actor.view.bindings[&key("document.root")].evaluations;
        actor.dispatch(Message {
            action: key("bump_active"),
            value: 0,
        })?;
        assert_eq!(
            actor.computed_evaluations("document.is_active"),
            evaluations + 1
        );
        assert_eq!(
            actor.computed("document.is_active").unwrap() as *const blockui_runtime::Value,
            memo
        );
        assert_eq!(
            actor.view.bindings[&key("document.root")].evaluations,
            root_evaluations
        );
        let schema = actor.schema_version;
        actor.reload(Path::new(args.get(6).ok_or("missing integer schema")?))?;
        assert_ne!(actor.schema_version, schema);
        assert_eq!(actor.integer("document.active"), 1);
        assert_eq!(actor.slot_address("document.count"), count);
        actor.reload(Path::new(&args[2]))?;
        assert_eq!(actor.schema_version, schema);
        assert!(actor.boolean("document.active"));
        assert_eq!(actor.slot_address("document.count"), count);
        println!(
            "PASS generated actor schema/default expressions/cfg fields, changed behavior and preserved named slots across code/schema replacement"
        );
        return Ok(());
    }
    #[cfg(feature = "native-window")]
    if args[1] == "native-proof" {
        return blockui_runtime::native::proof(
            actor,
            Path::new(args.get(3).ok_or("missing replacement")?),
        );
    }
    #[cfg(feature = "native-window")]
    if args[1] == "watch-probe" {
        return blockui_runtime::native::watch_probe(
            actor,
            blockui_runtime::native::ReloadConfig::from_environment(
                &std::env::current_dir().map_err(|error| error.to_string())?,
            ),
        );
    }
    #[cfg(feature = "native-window")]
    if args[1] == "native-migration-proof" {
        return blockui_runtime::native::migration_proof(
            actor,
            Path::new(args.get(3).ok_or("missing migration replacement")?),
        );
    }
    #[cfg(feature = "native-window")]
    if args[1] == "native-view-proof" {
        return blockui_runtime::native::view_proof(
            actor,
            Path::new(args.get(3).ok_or("missing dynamic implementation")?),
        );
    }
    #[cfg(feature = "native-window")]
    if args[1] == "native-scalar-graph-proof" {
        return blockui_runtime::native::scalar_graph_proof(
            actor,
            Path::new(args.get(3).ok_or("missing graph replacement")?),
        );
    }
    #[cfg(feature = "native-window")]
    if args[1] == "native-graph-proof" {
        return blockui_runtime::native::graph_proof(
            actor,
            Path::new(args.get(3).ok_or("missing graph replacement")?),
        );
    }
    #[cfg(feature = "native-window")]
    if args[1] == "gui"
        || args[1] == "smoke"
        || args[1] == "watch-smoke"
        || args[1] == "capture"
        || args[1] == "focus-capture"
        || args[1] == "clipboard-smoke"
        || args[1] == "graph-smoke"
        || args[1] == "io-smoke"
        || args[1] == "save-smoke"
    {
        let repo = std::env::current_dir().map_err(|e| e.to_string())?;
        let source = if args[1] == "clipboard-smoke" {
            blockui_runtime::native::NativeSource::without_watch(actor)?.clipboard_smoke(
                args.get(3)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| repo.join("reports/native-clipboard.json")),
            )?
        } else if args[1] == "save-smoke" {
            blockui_runtime::native::NativeSource::without_watch(actor)?.save_smoke(
                Path::new(args.get(3).ok_or("missing save replacement")?),
                Path::new(args.get(4).ok_or("missing save file")?),
            )?
        } else if args[1] == "io-smoke" {
            blockui_runtime::native::NativeSource::without_watch(actor)?.io_smoke(
                Path::new(args.get(3).ok_or("missing I/O replacement")?),
                Path::new(args.get(4).ok_or("missing I/O file")?),
            )?
        } else if args[1] == "graph-smoke" {
            blockui_runtime::native::NativeSource::without_watch(actor)?
                .graph_smoke(Path::new(args.get(3).ok_or("missing graph replacement")?))?
        } else if args[1] == "focus-capture" {
            use wabou_shell_vello::{FrameSource, KeyEvent, KeyPhase, UiEvent};
            let mut source = blockui_runtime::native::NativeSource::without_watch(actor)?;
            source.handle_event(UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: "Tab".into(),
                key_without_modifiers: "Tab".into(),
                code: "Tab".into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers: Default::default(),
                repeat: false,
                synthetic: false,
            }));
            source.capture_on_focus()
        } else if args[1] == "capture" {
            blockui_runtime::native::NativeSource::without_watch(actor)?.capture()
        } else if args[1] == "smoke" {
            blockui_runtime::native::NativeSource::without_watch(actor)?
                .smoke(Path::new(args.get(3).ok_or("missing replacement")?))
        } else {
            blockui_runtime::native::NativeSource::new(
                actor,
                blockui_runtime::native::ReloadConfig::from_environment(&repo),
            )?
        };
        let source = if args[1] == "watch-smoke" {
            source.watch_smoke()?
        } else {
            source
        };
        return wabou_shell_vello::run_window_with_options(
            Box::new(source),
            wabou_shell_vello::WindowOptions::new()
                .title("BlockUI")
                .initial_inner_size(720, 620)
                .min_inner_size(420, 460),
        )
        .map_err(|e| e.to_string());
    }
    status(&actor);
    for line in io::stdin().lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        let (command, arg) = line.split_once(' ').unwrap_or((&line, ""));
        let result = match command {
            "increment" => actor.dispatch(Message {
                action: key("increment"),
                value: 0,
            }),
            "enqueue" => actor.enqueue(Message {
                action: key("increment"),
                value: 0,
            }),
            "tick" => actor.tick(),
            "reload" => actor.reload(Path::new(arg)),
            "preview" => actor
                .preview_reload(Path::new(arg))
                .map(|preview| print!("{preview}")),
            "notes" => actor.set_text(key("notes"), arg),
            "delay" => actor.delay(
                9,
                Duration::from_millis(150),
                Message {
                    action: key("task_done"),
                    value: 123,
                },
            ),
            "quit" => break,
            "status" => Ok(()),
            _ => Err("unknown command".into()),
        };
        if let Err(error) = result {
            println!("error={error}");
        }
        status(&actor);
    }
    Ok(())
}
#[allow(clippy::unnecessary_cast)] // RawSocket is usize-sized on supported Windows targets.
fn socket_handle(stream: &std::net::TcpStream) -> u64 {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        stream.as_raw_fd() as u64
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawSocket;
        stream.as_raw_socket() as u64
    }
}
fn verify(paths: &[String]) -> Result<(), String> {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
    };
    if paths.len() != 6 {
        return Err("six compiled variants required".into());
    }
    let mut actor = Actor::new(42, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert!(
        actor.view.dependencies.is_empty(),
        "static Counter tree must not subscribe to count for empty rows"
    );
    assert_eq!(
        actor.view.bindings[&key("counter.task")].dependencies.len(),
        1,
        "task label must depend only on task_result"
    );
    actor.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    actor.set_text(key("notes"), "unsaved document · 中文")?;
    let state = actor.state_address();
    let count_slot = actor.slot_address("count");
    let text_slot = actor.slot_address("notes");
    let schema = actor.schema_version;
    let version = actor.implementation_version();
    let old_count_binding = &actor.view.bindings[&key("counter.count")];
    let binding_identity = old_count_binding.node.binding;
    assert_ne!(binding_identity, 0);
    assert!(!old_count_binding.text.contains("new code"));
    let editor = actor.view.renderer.get_mut(&key("counter.notes")).unwrap();
    editor.text = "unsaved document · 中文".into();
    editor.selection = (3, 8);
    editor.focus = true;
    actor
        .view
        .renderer
        .get_mut(&key("counter.scroll"))
        .unwrap()
        .scroll = 137.0;
    let editor = actor.view.renderer[&key("counter.notes")].clone();
    actor.enqueue(Message {
        action: key("increment"),
        value: 0,
    })?;
    let awakened = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = awakened.clone();
    actor.set_wake(std::sync::Arc::new(move || {
        observed.store(true, std::sync::atomic::Ordering::Relaxed)
    }));
    actor.delay(
        9,
        Duration::from_millis(100),
        Message {
            action: key("task_done"),
            value: 123,
        },
    )?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let connection =
        TcpStream::connect(listener.local_addr().unwrap()).map_err(|e| e.to_string())?;
    let (mut peer, _) = listener.accept().map_err(|e| e.to_string())?;
    connection
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let fd = socket_handle(&connection);
    actor.register_socket(8, connection, blockui_runtime::SocketReloadPolicy::Preserve)?;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("count"), count_slot);
    assert_eq!(actor.slot_address("notes"), text_slot);
    assert_eq!(actor.schema_version, schema);
    assert_ne!(actor.implementation_version(), version);
    // Identical state and semantic binding identity must still run the new code.
    assert_eq!(actor.integer("count"), 1);
    let refreshed = &actor.view.bindings[&key("counter.count")];
    assert_eq!(refreshed.node.binding, binding_identity);
    assert!(refreshed.text.contains("new code"));
    assert_eq!(actor.text("notes"), "unsaved document · 中文");
    assert_eq!(actor.mailbox.len(), 1);
    assert!(actor.tasks.contains_key(&9));
    assert_eq!(actor.view.renderer[&key("counter.notes")], editor);
    assert_eq!(actor.view.renderer[&key("counter.scroll")].scroll, 137.0);
    actor.tick()?;
    assert_eq!(actor.integer("count"), 11);
    let unchanged = actor.view.bindings[&key("counter.task")].evaluations;
    actor.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    assert_eq!(actor.integer("count"), 21);
    assert_eq!(
        actor.view.bindings[&key("counter.task")].evaluations,
        unchanged
    );
    assert_eq!(
        actor.view.bindings[&key("counter.count")]
            .dependencies
            .len(),
        2
    );
    std::thread::sleep(Duration::from_millis(130));
    actor.tick()?;
    assert_eq!(actor.integer("task_result"), 123);
    assert!(awakened.load(std::sync::atomic::Ordering::Relaxed));
    assert_eq!(socket_handle(actor.socket(8).unwrap()), fd);
    actor.socket(8).unwrap().write_all(b"live").unwrap();
    let mut data = [0; 4];
    peer.read_exact(&mut data).unwrap();
    assert_eq!(&data, b"live");
    let before = actor.implementation_version();
    assert!(actor.reload(Path::new(&paths[4])).is_err());
    assert_eq!(actor.integer("count"), 21);
    assert_eq!(actor.implementation_version(), before);
    assert_eq!(actor.schema_version, schema);
    assert!(actor.reload(Path::new(&paths[5])).is_err());
    assert_eq!(actor.implementation_version(), before);
    let cancelled_listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let cancelled =
        TcpStream::connect(cancelled_listener.local_addr().unwrap()).map_err(|e| e.to_string())?;
    let (mut cancelled_peer, _) = cancelled_listener.accept().map_err(|e| e.to_string())?;
    cancelled_peer
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let cancelled_fd = socket_handle(&cancelled);
    actor.register_socket(10, cancelled, blockui_runtime::SocketReloadPolicy::Cancel)?;
    assert!(actor.reload(Path::new(&paths[4])).is_err());
    assert_eq!(socket_handle(actor.socket(10).unwrap()), cancelled_fd);
    actor.reload(Path::new(&paths[3]))?;
    assert!(actor.socket(10).is_none());
    assert_eq!(cancelled_peer.read(&mut [0; 1]).unwrap(), 0);
    assert_eq!(socket_handle(actor.socket(8).unwrap()), fd);
    assert_eq!(actor.socket_count(), 1);
    println!(
        "PASS bounded host socket resources preserve descriptors through real DLL replacement; rejected candidates retain Cancel sockets and successful schema migration releases them"
    );
    assert_eq!(actor.integer("bonus"), 42);
    assert_eq!(actor.integer("count"), 21);
    assert_ne!(actor.schema_version, schema);
    // A separate actor proves automatic/default addition without explicit migration.
    let mut defaults = Actor::new(43, Path::new(".runtime"))?;
    defaults.reload(Path::new(&paths[0]))?;
    defaults.reload(Path::new(&paths[2]))?;
    assert_eq!(defaults.integer("bonus"), 7);
    let stable = actor.slot_address("count");
    for _ in 0..40 {
        actor.reload(Path::new(&paths[3]))?;
        assert_eq!(actor.slot_address("count"), stable);
        assert_eq!(actor.artifact_count(), 1);
    }
    let mut cancelled = Actor::new(44, Path::new(".runtime"))?;
    cancelled.reload(Path::new(&paths[0]))?;
    cancelled.delay(
        1,
        Duration::from_secs(600),
        Message {
            action: key("task_done"),
            value: 0,
        },
    )?;
    let shutdown = std::time::Instant::now();
    drop(cancelled);
    assert!(shutdown.elapsed() < Duration::from_millis(100));
    println!(
        "PASS state identity, slot identity, mailbox, new behavior, schema separation, default migration, explicit migration, rejected migration rollback, ABI rejection, keyed renderer state, dependency rebuild, selective invalidation, host task, live socket, 40 reloads bounded artifacts"
    );
    Ok(())
}
fn view_proof(paths: &[String]) -> Result<(), String> {
    if paths.len() != 5 {
        return Err(
            "view-proof needs base, dynamic, duplicate-key, invalid-layout, short-table plugins"
                .into(),
        );
    }
    let mut actor = Actor::new(50, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.set_text(key("notes"), "Unsaved through topology changes")?;
    let slot = actor.slot_address("notes");
    let state = actor.state_address();
    actor.reload(Path::new(&paths[1]))?;
    let row = |id: u64| {
        id.to_le_bytes()
            .into_iter()
            .fold(key("counter.rows"), |hash, byte| {
                (hash ^ byte as u64).wrapping_mul(0x100000001b3)
            })
    };
    assert!(actor.view.bindings.contains_key(&row(1)));
    assert!(
        actor
            .view
            .dependencies
            .contains(&blockui_runtime::Dependency::State(key("count")))
    );
    let unrelated = actor.view.bindings[&key("counter.task")].evaluations;
    let first_row = actor.view.bindings[&row(1)].evaluations;
    let anonymous = actor.view.bindings[&key("counter.anonymous")].evaluations;
    assert_eq!(
        actor.view.bindings[&key("counter.anonymous")].node.binding,
        0
    );
    assert_eq!(
        actor.view.bindings[&key("counter.task")].dependencies.len(),
        1
    );
    assert_eq!(
        actor.view.bindings[&key("counter.shape")].text,
        "Even structure"
    );
    actor.view.renderer.get_mut(&row(1)).unwrap().focus = true;
    actor.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&key("counter.task")].evaluations,
        unrelated
    );
    assert_eq!(actor.view.bindings[&row(1)].evaluations, first_row);
    assert_eq!(
        actor.view.bindings[&key("counter.anonymous")].evaluations,
        anonymous + 1
    );
    assert_eq!(
        actor.view.bindings[&key("counter.shape")].text,
        "Odd structure"
    );
    assert_eq!(
        actor.view.bindings[&key("counter.switch")].text,
        "Background result: 0"
    );
    assert!(actor.view.bindings.contains_key(&row(2)));
    assert!(
        actor
            .view
            .bindings
            .contains_key(&key("counter.conditional"))
    );
    actor.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&key("counter.task")].evaluations,
        unrelated
    );
    assert_eq!(
        actor.view.bindings[&key("counter.shape")].text,
        "Even structure"
    );
    assert!(
        actor.view.bindings[&key("counter.switch")]
            .text
            .starts_with("Count: 2")
    );
    assert!(actor.view.bindings.contains_key(&row(3)));
    assert!(
        !actor
            .view
            .bindings
            .contains_key(&key("counter.conditional"))
    );
    actor.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    assert!(!actor.view.bindings.contains_key(&row(2)));
    assert!(!actor.view.renderer.contains_key(&row(2)));
    assert!(!actor.view.bindings.contains_key(&row(3)));
    // Reintroduced nodes must evaluate afresh, while retained siblings keep their cache.
    assert_eq!(
        actor.view.bindings[&key("counter.conditional")].evaluations,
        1
    );
    assert_eq!(actor.view.bindings[&row(1)].evaluations, first_row);
    assert_eq!(
        actor.view.bindings[&key("counter.anonymous")].evaluations,
        anonymous + 3
    );
    assert!(actor.view.renderer[&row(1)].focus);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("notes"), slot);
    assert_eq!(actor.text("notes"), "Unsaved through topology changes");
    let version = actor.implementation_version();
    let view = actor.view.order.clone();
    for path in &paths[2..] {
        assert!(actor.reload(Path::new(path)).is_err());
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.view.order, view);
        assert_eq!(actor.integer("count"), 3);
        assert_eq!(actor.slot_address("notes"), slot);
        assert_eq!(actor.artifact_count(), 1);
    }
    println!(
        "PASS typed SDK, nested view, tracked if/for structure, scoped keys, retained keyed renderer state, removed node disposal, anonymous binding recalculation, reinserted binding cold evaluation, duplicate key, invalid layout and short ABI table rollback"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
