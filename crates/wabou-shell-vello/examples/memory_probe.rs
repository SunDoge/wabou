//! Staged, headless two-rectangle probe. Drive with scripts/measure-memory.ts.
use std::io::{self, Write};
use std::time::{Duration, Instant};

use serde_json::json;
use vello_common::{kurbo::Rect, peniko::Color};
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Scene, TextureBindings};
use wgpu_context::{BufferRenderer, BufferRendererConfig, DeviceHandle, WGPUContext};

fn checkpoint(stage: &str, details: serde_json::Value) {
    println!("{}", json!({"stage": stage, "details": details}));
    io::stdout().flush().unwrap();
    let mut ack = String::new();
    assert!(
        io::stdin().read_line(&mut ack).unwrap() > 0,
        "sampler disconnected"
    );
    assert_eq!(ack.trim(), "continue", "expected sampler acknowledgement");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "usage: memory_probe PHYSICAL_WIDTH PHYSICAL_HEIGHT STEADY_SECONDS"
    );
    let width: u16 = args[1].parse().expect("width must fit u16");
    let height: u16 = args[2].parse().expect("height must fit u16");
    let seconds: u64 = args[3].parse().expect("invalid steady duration");
    assert!(width >= 8 && height >= 8 && seconds > 0);
    checkpoint(
        "process_started",
        json!({"physicalWidth": width, "physicalHeight": height, "debugAssertions": cfg!(debug_assertions)}),
    );
    let context = WGPUContext::with_features_and_limits(Some(wgpu::Features::CLEAR_TEXTURE), None);
    let device = pollster::block_on(DeviceHandle::new_from_compatible_surface(
        context.instance.clone(),
        None,
        context.extra_features(),
        context.override_limits(),
    ))
    .expect("no compatible GPU device");
    let info = device.adapter.get_info();
    checkpoint(
        "device_created",
        json!({"adapter": info.name, "backend": format!("{:?}", info.backend), "deviceType": format!("{:?}", info.device_type)}),
    );
    let buffer = BufferRenderer::new(
        BufferRendererConfig {
            width: width.into(),
            height: height.into(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        },
        device,
        0,
    );
    checkpoint(
        "target_created",
        json!({"includesReadbackStagingBuffer": true}),
    );
    let (mut renderer, mut resources) = Renderer::new(
        buffer.device(),
        &RenderTargetConfig {
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: width.into(),
            height: height.into(),
        },
    );
    checkpoint("renderer_created", json!({"settings": "default"}));
    let mut scene = Scene::new(width, height);
    scene.set_paint(Color::from_rgb8(255, 0, 0));
    scene.fill_rect(&Rect::new(0.0, 0.0, f64::from(width), f64::from(height)));
    scene.set_paint(Color::from_rgb8(0, 255, 0));
    scene.fill_rect(&Rect::new(
        f64::from(width) / 4.0,
        f64::from(height) / 4.0,
        f64::from(width) * 0.75,
        f64::from(height) * 0.75,
    ));
    let mut render = || {
        let mut encoder = buffer.device().create_command_encoder(&Default::default());
        renderer
            .render(
                &scene,
                &mut resources,
                buffer.device(),
                buffer.queue(),
                &mut encoder,
                &RenderSize {
                    width: width.into(),
                    height: height.into(),
                },
                &buffer.target_texture_view(),
                &TextureBindings::new(),
            )
            .expect("render failed");
        buffer.queue().submit([encoder.finish()]);
        buffer
            .device()
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU completion failed");
    };
    render();
    checkpoint("first_frame_completed", json!({"gpuCompleted": true}));
    let start = Instant::now();
    let mut frames = 0;
    while start.elapsed() < Duration::from_secs(seconds) {
        render();
        frames += 1;
        std::thread::sleep(Duration::from_millis(16));
    }
    checkpoint(
        "steady_state",
        json!({"frames": frames, "seconds": start.elapsed().as_secs_f64()}),
    );
    // Readback happens after all baseline measurements so CPU pixels do not inflate them.
    let mut rgba = vec![0; usize::from(width) * usize::from(height) * 4];
    buffer.copy_texture_to_buffer(&mut rgba);
    assert_eq!(&rgba[..4], &[255, 0, 0, 255], "background pixel");
    let center = (usize::from(height / 2) * usize::from(width) + usize::from(width / 2)) * 4;
    assert_eq!(
        &rgba[center..center + 4],
        &[0, 255, 0, 255],
        "foreground pixel"
    );
    checkpoint("pixels_verified", json!({"checkedPixels": 2}));
}
