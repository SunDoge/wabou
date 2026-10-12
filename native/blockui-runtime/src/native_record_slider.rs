use super::*;
fn assert_value(source: &NativeSource, slider: u64, value: i64) {
    assert_eq!(source.actor.widget_integer(slider), Some(value));
    assert_eq!(i64::from(source.sliders[&slider].value()), value);
    let snapshot = source.semantic_snapshot().unwrap();
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.id == slider)
        .unwrap();
    assert_eq!(node.role, shell::SemanticRole::Slider);
    assert_eq!(node.label.as_deref(), Some("Volume"));
    assert_eq!(node.numeric_value, Some(value as f64));
    assert_eq!(node.min_numeric_value, Some(0.0));
    assert_eq!(node.max_numeric_value, Some(100.0));
    assert_eq!(snapshot.focus, Some(slider));
}
pub fn proof(
    base: &Path,
    replacement: &Path,
    invalid: &Path,
    evolved: &Path,
) -> Result<(), String> {
    let mut actor = Actor::new(214, Path::new(".runtime"))?;
    actor.reload(base)?;
    let mut source = NativeSource::without_watch(actor)?;
    let slider = key("volume");
    source.set_semantics_enabled(true);
    source.set_focus(Some(slider));
    let mut tcx = TextContext::new();
    for (width, height) in [(640, 480), (320, 480)] {
        source.build_frame(&mut tcx, width, height);
        let mut bottom = 0.0;
        for name in ["volume", "value", "other"] {
            let rect = source
                .widget_layout_rect(key(name))
                .ok_or("missing slider layout")?;
            assert!(rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01);
            assert!(rect[1] >= bottom && rect[3] <= height as f32 + 0.01);
            assert!(rect[2] > rect[0] && rect[3] > rect[1]);
            bottom = rect[3];
        }
    }
    let instance = (&**source.sliders.get(&slider).unwrap()
        as *const crate::native_slider::NativeSlider) as usize;
    let node = source.ids[&slider];
    let key_event = || {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: "ArrowRight".into(),
            key_without_modifiers: "ArrowRight".into(),
            code: "ArrowRight".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: shell::Modifiers::empty(),
            repeat: false,
            synthetic: false,
        })
    };
    assert_value(&source, slider, 0);
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, slider, 10);
    let buffer = source.actor.text("slider.fields.settings").as_ptr();
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.text("slider.fields.settings").as_ptr(), buffer);
    assert_value(&source, slider, 10);
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, slider, 30);
    assert!(
        source
            .replace(invalid)
            .unwrap_err()
            .contains("within its range")
    );
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, slider, 30);
    source.replace(evolved)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, slider, 30);
    assert_eq!(source.actor.view.bindings[&key("title")].text, "Sound");
    let mut bottom = 0.0;
    for name in ["title", "volume", "value", "other"] {
        let rect = source
            .widget_layout_rect(key(name))
            .ok_or("missing migrated slider layout")?;
        assert!(rect[0] >= 0.0 && rect[2] <= 320.01);
        assert!(rect[1] >= bottom && rect[3] <= 480.01);
        assert!(rect[2] > rect[0] && rect[3] > rect[1]);
        bottom = rect[3];
    }
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 640, 480);
    assert_value(&source, slider, 50);
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, slider, 70);
    let buffer = source.actor.text("slider.fields.settings").as_ptr();
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, slider, 70);
    assert_eq!(source.actor.text("slider.fields.settings").as_ptr(), buffer);
    assert_eq!(source.ids[&slider], node);
    assert_eq!(
        (&**source.sliders.get(&slider).unwrap() as *const crate::native_slider::NativeSlider)
            as usize,
        instance
    );
    println!(
        "PASS native record slider: ArrowRight updates durable field through current code; real DLL/range rejection/schema migration retain control instance/node/focus; rejected native change restores thumb/field/buffer; numeric semantics and narrow layout agree"
    );
    Ok(())
}
