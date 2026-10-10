use super::*;
use blockui::{
    Action, Button, ButtonVariant, Column, Cx, IntoView, StateSpec, Styled, Text, TextTone, View,
    ViewStyle,
};

struct Appearance;
impl blockui::Actor for Appearance {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[Action::new("appearance.click", |_, _| Ok(()))];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        let mut children: Vec<View> = [
            ("Primary", ButtonVariant::Primary, true),
            ("Secondary", ButtonVariant::Secondary, true),
            ("Delete", ButtonVariant::Danger, true),
            ("Disabled delete", ButtonVariant::Danger, false),
            ("Disabled primary", ButtonVariant::Primary, false),
        ]
        .into_iter()
        .map(|(text, variant, enabled)| {
            Button {
                text: text.into(),
                variant,
                enabled,
                on_click: Self::ACTIONS[0],
                ..Default::default()
            }
            .key(text)
            .into_view()
        })
        .collect();
        children.push(
            Text::new("Enter a valid date.")
                .key("validation.error")
                .style(ViewStyle {
                    text_tone: TextTone::Error,
                    ..Default::default()
                }),
        );
        Ok(Column {
            padding: 16.0,
            gap: 12.0,
            ..Default::default()
        }
        .children(children))
    }
}

#[test]
#[ignore = "explicit Vello Hybrid button appearance pixels"]
fn button_appearance_pixels() -> Result<(), String> {
    let actor = Actor::for_test::<Appearance>()?;
    let mut source = NativeSource::without_watch(actor)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    let danger = key("Delete");
    source.build_frame(&mut tcx, 320, 400);
    source.set_focus(Some(danger));
    for pressed in [false, true] {
        if pressed {
            let rect = source.widget_layout_rect(danger).unwrap();
            source.handle_event(UiEvent::Pointer(shell::PointerEvent {
                phase: PointerPhase::Down,
                position: shell::Point {
                    x: f64::from(rect[0] + 8.0),
                    y: f64::from(rect[1] + 8.0),
                },
                button: Some(shell::PointerButton::Primary),
                buttons: 1,
                modifiers: Default::default(),
                properties: Default::default(),
            }));
        }
        for width in [320, 640] {
            let nodes = source.build_frame(&mut tcx, width, 400);
            for scale in [1, 2] {
                let mut scene = shell::Scene::new();
                shell::scene::build_scene_scaled(
                    &mut scene,
                    &nodes,
                    &mut tcx,
                    width,
                    400,
                    source.base_color(),
                    f64::from(scale),
                );
                let image = shell::renderer::render_to_image(
                    &scene,
                    width * scale,
                    400 * scale,
                    source.base_color(),
                )
                .map_err(|error| error.to_string())?;
                for (name, expected) in [
                    ("Primary", [31u8, 88, 155]),
                    ("Secondary", [240, 243, 247]),
                    (
                        "Delete",
                        if pressed {
                            [255, 225, 225]
                        } else {
                            [255, 245, 245]
                        },
                    ),
                    ("Disabled delete", [240, 243, 247]),
                    ("Disabled primary", [240, 243, 247]),
                ] {
                    let rect = source.widget_layout_rect(key(name)).unwrap();
                    let pixel = image.get_pixel(
                        ((rect[2] - 8.0) * scale as f32) as u32,
                        ((rect[1] + 8.0) * scale as f32) as u32,
                    );
                    for channel in 0..3 {
                        assert!(
                            pixel[channel].abs_diff(expected[channel]) <= 2,
                            "{name} pressed={pressed} scale={scale}: {pixel:?}"
                        );
                    }
                }
                let error = source.widget_layout_rect(key("validation.error")).unwrap();
                let mut red_ink = 0;
                for y in (error[1] * scale as f32) as u32..(error[3] * scale as f32) as u32 {
                    for x in (error[0] * scale as f32) as u32..(error[2] * scale as f32) as u32 {
                        let pixel = image.get_pixel(x, y);
                        if i16::from(pixel[0]) - i16::from(pixel[1]) > 40
                            && i16::from(pixel[0]) - i16::from(pixel[2]) > 40
                        {
                            red_ink += 1;
                        }
                    }
                }
                assert!(
                    red_ink > 10 * scale * scale,
                    "missing error text ink at {width} scale {scale}"
                );
                let rect = source.widget_layout_rect(danger).unwrap();
                for (x, expected) in [
                    (rect[0] + 0.5, [239u8, 174, 174]),
                    (rect[0] - 3.0, [31u8, 88, 155]),
                ] {
                    let pixel = image.get_pixel(
                        (x * scale as f32) as u32,
                        (((rect[1] + rect[3]) * 0.5) * scale as f32) as u32,
                    );
                    for channel in 0..3 {
                        assert!(
                            pixel[channel].abs_diff(expected[channel]) <= 3,
                            "danger border/focus pressed={pressed} scale={scale}: {pixel:?}"
                        );
                    }
                }
                if !pressed && width == 320 && scale == 1 {
                    let reports = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reports");
                    std::fs::create_dir_all(&reports).map_err(|error| error.to_string())?;
                    image
                        .save(reports.join("button-appearance.png"))
                        .map_err(|error| error.to_string())?;
                }
            }
        }
    }
    Ok(())
}
