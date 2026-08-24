use std::{fs, num::NonZeroUsize, path::PathBuf};

use bytemuck::{Pod, Zeroable};
use vello::{
    AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene,
    kurbo::{Affine, BezPath, Rect, Stroke},
    peniko::{Color, Fill},
    wgpu::{self, util::DeviceExt},
};
use wgpu_context::{BufferRendererConfig, WGPUContext};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct OrbUniforms {
    resolution_time: [f32; 4],
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = WGPUContext::new();
    let output = pollster::block_on(context.create_buffer_renderer(BufferRendererConfig {
        width: WIDTH,
        height: HEIGHT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    }))?;
    let device = output.device();
    let queue = output.queue();

    let background = texture_view(device, "orb-background");
    let ui = texture_view(device, "vello-ui");
    render_orb(device, queue, &background);
    render_vello_ui(device, queue, &ui)?;
    compose(
        device,
        queue,
        &background,
        &ui,
        &output.target_texture_view(),
    );

    let mut pixels = vec![0; WIDTH as usize * HEIGHT as usize * 4];
    output.copy_texture_to_buffer(&mut pixels);
    let image = image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels)
        .ok_or("failed to construct the output image")?;
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("out/composed.png");
    fs::create_dir_all(output_path.parent().expect("output has a parent"))?;
    image.save(&output_path)?;
    println!("wrote {}", output_path.display());
    Ok(())
}

fn texture_view(device: &wgpu::Device, label: &str) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn render_orb(device: &wgpu::Device, queue: &wgpu::Queue, target: &wgpu::TextureView) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("orb-shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("orb.wgsl").into()),
    });
    let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("orb-uniforms"),
        contents: bytemuck::bytes_of(&OrbUniforms {
            resolution_time: [WIDTH as f32, HEIGHT as f32, 2.35, 0.0],
        }),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("orb-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("orb-bind-group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniforms.as_entire_binding(),
        }],
    });
    let pipeline = fullscreen_pipeline(device, &shader, &[&layout], "orb-pipeline");
    draw_fullscreen(device, queue, target, &pipeline, Some(&bind_group));
}

fn render_vello_ui(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &wgpu::TextureView,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut scene = Scene::new();
    let card = Rect::new(700.0, 118.0, 1160.0, 682.0).to_rounded_rect(34.0);
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(10, 18, 38, 218),
        None,
        &card,
    );
    scene.stroke(
        &Stroke::new(1.5),
        Affine::IDENTITY,
        Color::from_rgba8(184, 220, 255, 105),
        None,
        &card,
    );
    for (index, width) in [310.0, 250.0, 340.0, 285.0].into_iter().enumerate() {
        let y = 270.0 + index as f64 * 58.0;
        let line = Rect::new(760.0, y, 760.0 + width, y + 14.0).to_rounded_rect(7.0);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgba8(205, 225, 255, 185),
            None,
            &line,
        );
    }
    let mut accent = BezPath::new();
    accent.move_to((760.0, 220.0));
    accent.curve_to((820.0, 150.0), (930.0, 270.0), (1085.0, 192.0));
    scene.stroke(
        &Stroke::new(8.0),
        Affine::IDENTITY,
        Color::from_rgba8(119, 196, 255, 255),
        None,
        &accent,
    );

    let mut renderer = Renderer::new(
        device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: AaSupport::area_only(),
            num_init_threads: NonZeroUsize::new(1),
            pipeline_cache: None,
        },
    )?;
    renderer.render_to_texture(
        device,
        queue,
        &scene,
        target,
        &RenderParams {
            base_color: Color::from_rgba8(0, 0, 0, 0),
            width: WIDTH,
            height: HEIGHT,
            antialiasing_method: AaConfig::Area,
        },
    )?;
    Ok(())
}

fn compose(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    background: &wgpu::TextureView,
    ui: &wgpu::TextureView,
    target: &wgpu::TextureView,
) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("compose-shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("compose.wgsl").into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("compose-layout"),
        entries: &[
            texture_entry(0),
            texture_entry(1),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("compose-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("compose-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(background),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(ui),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let pipeline = fullscreen_pipeline(device, &shader, &[&layout], "compose-pipeline");
    draw_fullscreen(device, queue, target, &pipeline, Some(&bind_group));
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn fullscreen_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layouts: &[&wgpu::BindGroupLayout],
    label: &str,
) -> wgpu::RenderPipeline {
    let layouts = layouts.iter().copied().map(Some).collect::<Vec<_>>();
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn draw_fullscreen(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bind_group: Option<&wgpu::BindGroup>,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("fullscreen-pass"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fullscreen-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        if let Some(bind_group) = bind_group {
            pass.set_bind_group(0, bind_group, &[]);
        }
        pass.draw(0..3, 0..1);
    }
    queue.submit([encoder.finish()]);
}
