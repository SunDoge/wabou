use super::*;
fn assert_value(source: &NativeSource, value: f64) {
    let snapshot = source.semantic_snapshot().unwrap();
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.id == key("progress"))
        .unwrap();
    assert_eq!(node.role, shell::SemanticRole::ProgressBar);
    assert_eq!(node.label.as_deref(), Some("Download"));
    assert_eq!(node.numeric_value, Some(value));
    assert_eq!(node.min_numeric_value, Some(0.0));
    assert_eq!(node.max_numeric_value, Some(1.0));
    assert_eq!(
        source.actor.view.bindings[&key("progress")].text,
        value.to_string()
    );
}
pub fn proof(
    base: &Path,
    replacement: &Path,
    missing: &Path,
    evolved: &Path,
) -> Result<(), String> {
    let mut actor = Actor::new(216, Path::new(".runtime"))?;
    actor.reload(base)?;
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    let send = |actor: &mut Actor, value: f64| {
        actor.dispatch(Message {
            action: key("progress.fields.change"),
            value: value.to_bits() as i64,
        })
    };
    send(&mut source.actor, 0.5)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, 0.5);
    let node = source.ids[&key("progress")];
    let buffer = source.actor.text("progress.fields.settings").as_ptr();
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(
        source.actor.text("progress.fields.settings").as_ptr(),
        buffer
    );
    assert_value(&source, 0.5);
    assert!(source.replace(missing).is_err());
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, 0.5);
    source.replace(evolved)?;
    for (width, height) in [(320, 480), (640, 480)] {
        source.build_frame(&mut tcx, width, height);
        let mut bottom = 0.0;
        for name in ["title", "progress", "fraction", "edits"] {
            let rect = source
                .widget_layout_rect(key(name))
                .ok_or("missing progress layout")?;
            assert!(rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01);
            assert!(rect[1] >= bottom && rect[3] <= height as f32 + 0.01);
            assert!(rect[2] > rect[0] && rect[3] > rect[1]);
            bottom = rect[3];
        }
    }
    assert_value(&source, 0.5);
    assert_eq!(source.ids[&key("progress")], node);
    let buffer = source.actor.text("progress.fields.settings").as_ptr();
    assert!(send(&mut source.actor, f64::NAN).is_err());
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, 0.5);
    assert_eq!(
        source.actor.text("progress.fields.settings").as_ptr(),
        buffer
    );
    send(&mut source.actor, 0.5)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_value(&source, 0.25);
    assert_eq!(source.ids[&key("progress")], node);
    println!(
        "PASS native record progress: numeric semantics and 320/640 layout agree, real code/schema replacement keeps node identity, rejected fraction preserves frame and current code continues"
    );
    Ok(())
}
