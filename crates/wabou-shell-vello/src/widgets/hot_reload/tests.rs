use super::*;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
fn build(directory: &Path, step: u64, schema: u32, broken: bool) -> Result<PathBuf, String> {
    let source = include_str!("../../../../wabou-capability-sdk/examples/widget_counter.rs")
        .replace(
            "const STEP: u64 = 1;",
            &format!("const STEP: u64 = {step};"),
        )
        .replace(
            "SCHEMA_VERSION: u32 = 1",
            &format!("SCHEMA_VERSION: u32 = {schema}"),
        );
    build_source(
        directory,
        if broken {
            "compile_error!(\"intentional broken edit\");"
        } else {
            &source
        },
    )
}
fn build_source(directory: &Path, source: &str) -> Result<PathBuf, String> {
    let sdk = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../wabou-capability-sdk")
        .canonicalize()
        .unwrap();
    std::fs::create_dir_all(directory.join("src")).unwrap();
    std::fs::write(directory.join("src/lib.rs"), source).unwrap();
    let name = if source.contains("struct InputFixture") {
        "input-reload-fixture"
    } else if source.contains("export_plugin!") {
        "plugin-reload-fixture"
    } else {
        "widget-reload-fixture"
    };
    std::fs::write(directory.join("Cargo.toml"), format!("[package]\nname = {name:?}\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[lib]\ncrate-type = [\"cdylib\"]\n[dependencies]\nwabou-capability-sdk = {{ path = {sdk:?} }}\nserde_json = \"1\"\n[profile.dev]\ndebug = 0\n")).unwrap();
    // Share SDK artifacts across fixture builds; loader copies only live generations.
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/capability-fixtures");
    let output = std::process::Command::new("cargo")
        .args(["build", "--offline", "--manifest-path"])
        .arg(directory.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&target)
        .output()
        .unwrap();
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(target.join("debug").join(format!(
        "{}{}{}",
        std::env::consts::DLL_PREFIX,
        name.replace('-', "_"),
        std::env::consts::DLL_SUFFIX
    )))
}
#[test]
fn actual_dylib_replacement_preserves_widget_state_focus_and_drawing() {
    let directory = tempfile::tempdir().unwrap();
    let library = build(directory.path(), 1, 1, false).unwrap();
    let module = CapabilityModule::new(1, Arc::new(|_, _| Err("no widget host operations".into())));
    unsafe {
        module.reload(&library).unwrap();
    }
    let wakes = Arc::new(AtomicUsize::new(0));
    let count = wakes.clone();
    let wake: WakeCallback = Arc::new(move || {
        count.fetch_add(1, Ordering::Relaxed);
    });
    let mut widget = ReloadableWidget::new(module.clone(), serde_json::json!(40));
    let mut sibling = ReloadableWidget::new(module.clone(), serde_json::json!(7));
    widget.set_wake_callback(wake);
    widget.focus_changed(true);
    widget.config_changed(r#"{"label":"retained"}"#).unwrap();
    let result = widget.handle_event(&UiEvent::Pointer(wabou_shell::PointerEvent {
        phase: wabou_shell::PointerPhase::Down,
        position: wabou_shell::Point { x: 10.0, y: 10.0 },
        button: Some(wabou_shell::PointerButton::Primary),
        buttons: 1,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
    assert!(result.is_handled());
    assert!(result.changes().contains(WidgetChanges::LAYOUT));
    assert_eq!(widget.state(), &serde_json::json!(41));
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    assert!(
        matches!(&widget.drawing()[1], Draw::Text { text, .. } if text.contains("41 (step 1)"))
    );
    let state_address = widget.state() as *const Value;
    let generation = module.generation().unwrap();
    let library = build(directory.path(), 10, 1, false).unwrap();
    unsafe {
        module.reload(&library).unwrap();
    }
    assert_ne!(module.generation().unwrap(), generation);
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    assert!(
        matches!(&widget.drawing()[1], Draw::Text { text, .. } if text.contains("41 (step 1)"))
    );
    assert!(widget.poll_async_changes().contains(WidgetChanges::LAYOUT));
    assert!(widget.poll_async_changes().is_empty());
    assert_eq!(widget.state() as *const Value, state_address);
    assert!(widget.focused());
    assert_eq!(widget.config["label"], "retained");
    assert_eq!(widget.state(), &serde_json::json!(41));
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    assert!(
        matches!(&widget.drawing()[1], Draw::Text { text, .. } if text.contains("41 (step 10)"))
    );
    assert!(sibling.poll_async());
    sibling
        .dispatch(Event::Pointer {
            down: true,
            position: [0.0, 0.0],
        })
        .unwrap();
    assert_eq!(sibling.state(), &serde_json::json!(17));
    assert_eq!(widget.state(), &serde_json::json!(41));
    widget
        .dispatch(Event::Pointer {
            down: true,
            position: [10.0, 10.0],
        })
        .unwrap();
    assert_eq!(widget.state(), &serde_json::json!(51));
    // A failed compilation does not touch the loaded implementation.
    assert!(build(directory.path(), 20, 1, true).is_err());
    widget
        .dispatch(Event::Pointer {
            down: true,
            position: [10.0, 10.0],
        })
        .unwrap();
    assert_eq!(widget.state(), &serde_json::json!(61));
    let generation = module.generation().unwrap();
    let library = build(directory.path(), 20, 2, false).unwrap();
    assert!(unsafe { module.reload(&library) }.is_err());
    assert!(unsafe { module.reload(&directory.path().join("missing")) }.is_err());
    assert_eq!(module.generation().unwrap(), generation);
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    let drawing = widget.drawing().to_vec();
    // A callback failure retains the last valid drawing/state and reports the error.
    widget.state = Value::Null;
    assert!(widget.refresh_drawing([240.0, 80.0]).is_err());
    assert_eq!(widget.drawing(), drawing);
    assert!(
        widget
            .dispatch(Event::Pointer {
                down: true,
                position: [0.0, 0.0]
            })
            .is_err()
    );
    assert_eq!(widget.state(), &Value::Null);
    assert!(widget.last_error().is_some());
    // Exercise the actual native Widget::paint path, including text shaping.
    widget.state = serde_json::json!(61);
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    let mut text = wabou_shell::TextContext::new();
    let mut cx = PaintContext::new_clipped(240.0, 80.0, 4.0, 2.0, &mut text);
    widget.paint(&mut cx);
    let scene = cx.finish();
    assert!(
        scene
            .commands
            .iter()
            .any(|c| matches!(c, wabou_shell::paint::PaintCommand::Fill { .. }))
    );
    assert!(
        scene
            .commands
            .iter()
            .any(|c| matches!(c, wabou_shell::paint::PaintCommand::GlyphRun { .. }))
    );
    assert!(widget.last_error().is_none());
}
#[test]
fn invalid_draw_commands_are_rejected_before_scene_encoding() {
    assert!(
        validate_drawing(&[Draw::Rect {
            rect: [0.0, 0.0, -1.0, 1.0],
            color: [0; 4]
        }])
        .is_err()
    );
    assert!(
        validate_drawing(&[Draw::Text {
            origin: [f64::NAN, 0.0],
            text: "x".into(),
            font_size: 12.0,
            color: [0; 4]
        }])
        .is_err()
    );
}

#[test]
fn combined_plugin_swaps_all_exports_and_preserves_independent_state() {
    use wabou_capability_sdk::plugin::ExportKind;
    use wabou_runtime::capability_state::CapabilityState;
    let directory = tempfile::tempdir().unwrap();
    let source = include_str!("../../../../wabou-capability-sdk/examples/plugin.rs");
    let library = build_source(directory.path(), source).unwrap();
    let root = CapabilityModule::new(
        wabou_capability_sdk::plugin::PLUGIN_CONTRACT_VERSION,
        Arc::new(|_, _| Err("no root state".into())),
    );
    let counter = root
        .select_export(
            ExportKind::Capability,
            "counter",
            1,
            CapabilityState::default().operations(),
        )
        .unwrap();
    let secondary = root
        .select_export(
            ExportKind::Capability,
            "secondary",
            1,
            CapabilityState::default().operations(),
        )
        .unwrap();
    let widget_module = root
        .select_export(
            ExportKind::Widget,
            "reload-counter",
            1,
            Arc::new(|_, _| Err("no widget operations".into())),
        )
        .unwrap();
    let sibling_module = root
        .select_export(
            ExportKind::Widget,
            "secondary-counter",
            1,
            Arc::new(|_, _| Err("no widget operations".into())),
        )
        .unwrap();
    unsafe {
        root.reload(&library).unwrap();
    }
    assert_eq!(
        counter.generation().unwrap(),
        widget_module.generation().unwrap()
    );
    assert_eq!(
        counter.start("increment", b"null").unwrap().poll().unwrap(),
        Some(b"1".to_vec())
    );
    assert_eq!(
        secondary
            .start("increment", b"null")
            .unwrap()
            .poll()
            .unwrap(),
        Some(b"1".to_vec())
    );
    let mut pending = counter.start("later", b"null").unwrap();
    assert_eq!(pending.poll().unwrap(), None);
    let mut widget = ReloadableWidget::new(widget_module.clone(), serde_json::json!(40));
    let mut sibling = ReloadableWidget::new(sibling_module, serde_json::json!(7));
    widget.focus_changed(true);
    widget
        .dispatch(Event::Pointer {
            down: true,
            position: [0.0, 0.0],
        })
        .unwrap();
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    let address = widget.state() as *const Value;
    let next = source.replace("const STEP: u64 = 1;", "const STEP: u64 = 10;");
    let library = build_source(directory.path(), &next).unwrap();
    unsafe {
        root.reload(&library).unwrap();
    }
    assert_eq!(
        counter.start("increment", b"null").unwrap().poll().unwrap(),
        Some(b"11".to_vec())
    );
    assert_eq!(
        secondary.start("read", b"null").unwrap().poll().unwrap(),
        Some(b"1".to_vec())
    );
    assert_eq!(pending.poll().unwrap(), Some(b"1".to_vec())); // old code remains pinned
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    assert!(matches!(&widget.drawing()[1], Draw::Text { text, .. } if text.contains("step 1)")));
    assert!(widget.poll_async());
    assert!(sibling.poll_async());
    assert!(widget.focused());
    assert_eq!(widget.state() as *const Value, address);
    widget
        .dispatch(Event::Pointer {
            down: true,
            position: [0.0, 0.0],
        })
        .unwrap();
    assert_eq!(widget.state(), &serde_json::json!(51));
    assert_eq!(sibling.state(), &serde_json::json!(7));
    widget.refresh_drawing([240.0, 80.0]).unwrap();
    assert!(
        matches!(&widget.drawing()[1], Draw::Text { text, .. } if text.contains("51 (step 10)"))
    );
    let current = root.generation().unwrap();
    // One incompatible widget rejects the entire candidate, including capabilities.
    let incompatible = next.replace("SCHEMA_VERSION: u32 = 1", "SCHEMA_VERSION: u32 = 2");
    let library = build_source(directory.path(), &incompatible).unwrap();
    assert!(
        unsafe { root.reload(&library) }
            .unwrap_err()
            .contains("incompatible plugin export")
    );
    assert_eq!(root.generation().unwrap(), current);
    assert_eq!(
        counter.start("increment", b"null").unwrap().poll().unwrap(),
        Some(b"21".to_vec())
    );
    let missing = next.replace(
        "\"secondary-counter\" => CounterWidget",
        "\"renamed-counter\" => CounterWidget",
    );
    let library = build_source(directory.path(), &missing).unwrap();
    assert!(unsafe { root.reload(&library) }.is_err());
    assert_eq!(widget_module.generation().unwrap(), current);
    assert!(
        counter
            .select_export(
                ExportKind::Widget,
                "reload-counter",
                1,
                Arc::new(|_, _| Ok(Vec::new()))
            )
            .is_err()
    );
}

#[test]
fn native_input_reaches_plugin_without_losing_text_or_coordinates() {
    let directory = tempfile::tempdir().unwrap();
    let library = build_source(directory.path(), r#"
use wabou_capability_sdk::{export_module, widget::{Inputs, Event, Update, Draw, WidgetImplementation, WidgetModule}};
struct InputFixture;
impl WidgetImplementation for InputFixture {
 const SCHEMA_VERSION: u32 = 1;
 fn measure(_: &Inputs) -> Result<Option<[f32;2]>, String> { Ok(None) }
 fn paint(_: &Inputs) -> Result<Vec<Draw>, String> { Ok(vec![]) }
 fn accepts_text_input(_: &Inputs) -> Result<bool, String> { Ok(true) }
 fn event(_: &Inputs, event: Event) -> Result<Update, String> {
  Ok(Update { state: serde_json::to_value(event).unwrap(), handled: true })
 }
}
export_module!(WidgetModule<InputFixture>);
"#).unwrap();
    let module = CapabilityModule::new(1, Arc::new(|_, _| Err("unused".into())));
    unsafe {
        module.reload(&library).unwrap();
    }
    let mut widget = ReloadableWidget::new(module, Value::Null);
    assert!(widget.accepts_text_input());
    let cases = [
        (
            UiEvent::Wheel(wabou_shell::WheelEvent {
                position: wabou_shell::Point { x: 5.0, y: 6.0 },
                delta_x: -2.5,
                delta_y: 40.0,
                delta_mode: wabou_shell::WheelDeltaMode::Pixel,
                phase: wabou_shell::GesturePhase::Changed,
                modifiers: wabou_shell::Modifiers::CONTROL,
            }),
            serde_json::json!({"kind":"wheel", "position":[5.0,6.0], "delta":[-2.5,40.0], "modifiers":2}),
        ),
        (
            UiEvent::Key(wabou_shell::KeyEvent {
                phase: wabou_shell::KeyPhase::Down,
                key: "Enter".into(),
                key_without_modifiers: "Enter".into(),
                code: "Enter".into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers: wabou_shell::Modifiers::SHIFT,
                repeat: true,
                synthetic: false,
            }),
            serde_json::json!({"kind":"key", "down":true, "key":"Enter", "code":"Enter", "repeat":true, "modifiers":1}),
        ),
        (
            UiEvent::Ime(wabou_shell::ImeEvent::Commit("好".into())),
            serde_json::json!({"kind":"ime","event":{"kind":"commit","text":"好"}}),
        ),
        (
            UiEvent::Ime(wabou_shell::ImeEvent::DeleteSurrounding {
                before_bytes: 3,
                after_bytes: 0,
            }),
            serde_json::json!({"kind":"ime","event":{"kind":"deleteSurrounding","before_bytes":3,"after_bytes":0}}),
        ),
        (
            UiEvent::TextInput("你好🦀".into()),
            serde_json::json!({"kind":"textInput", "text":"你好🦀"}),
        ),
        (
            UiEvent::Paste("clipboard".into()),
            serde_json::json!({"kind":"paste", "text":"clipboard"}),
        ),
        (
            UiEvent::Ime(wabou_shell::ImeEvent::Preedit {
                text: "输入".into(),
                cursor: Some((0, 6)),
            }),
            serde_json::json!({"kind":"ime", "event":{"kind":"preedit", "text":"输入", "cursor":[0,6]}}),
        ),
        (
            UiEvent::Pointer(wabou_shell::PointerEvent {
                phase: wabou_shell::PointerPhase::Move,
                position: wabou_shell::Point { x: 12.5, y: 3.0 },
                button: None,
                buttons: 1,
                modifiers: wabou_shell::Modifiers::SHIFT,
                properties: Default::default(),
            }),
            serde_json::json!({"kind":"pointerMotion", "phase":"move", "position":[12.5,3.0], "buttons":1,"modifiers":1}),
        ),
    ];
    for (event, expected) in cases {
        assert!(widget.handle_event(&event).is_handled());
        assert_eq!(widget.state(), &expected);
    }
}

#[test]
fn linked_plugin_registry_drives_selected_widget_without_a_library() {
    use wabou_capability_sdk::{
        plugin::{ExportKind, Plugin, PluginModule, Registry},
        widget::WidgetImplementation,
    };
    struct LinkedWidget;
    impl WidgetImplementation for LinkedWidget {
        const SCHEMA_VERSION: u32 = 1;
        fn measure(_: &Inputs) -> Result<Option<[f32; 2]>, String> {
            Ok(Some([80.0, 30.0]))
        }
        fn paint(_: &Inputs) -> Result<Vec<Draw>, String> {
            Ok(Vec::new())
        }
        fn event(
            inputs: &Inputs,
            _: Event,
        ) -> Result<wabou_capability_sdk::widget::Update, String> {
            Ok(wabou_capability_sdk::widget::Update {
                state: serde_json::json!(inputs.state.as_u64().unwrap() + 1),
                handled: true,
            })
        }
    }
    struct LinkedPlugin;
    impl Plugin for LinkedPlugin {
        fn register(registry: &mut Registry) -> Result<(), String> {
            registry.widget::<LinkedWidget>("linked-counter")
        }
    }
    let root = CapabilityModule::new(
        wabou_capability_sdk::plugin::PLUGIN_CONTRACT_VERSION,
        Arc::new(|_, _| Err("unused".into())),
    );
    let selected = root
        .select_export(
            ExportKind::Widget,
            "linked-counter",
            1,
            Arc::new(|_, _| Err("unused".into())),
        )
        .unwrap();
    unsafe {
        root.install_linked(wabou_capability_sdk::module_api::<PluginModule<LinkedPlugin>>())
            .unwrap();
    }
    let mut widget = ReloadableWidget::new(selected, serde_json::json!(5));
    assert!(
        widget
            .dispatch(Event::Pointer {
                down: true,
                position: [1.0, 2.0]
            })
            .unwrap()
    );
    assert_eq!(widget.state(), &serde_json::json!(6));
    widget.refresh_drawing([80.0, 30.0]).unwrap();
}
