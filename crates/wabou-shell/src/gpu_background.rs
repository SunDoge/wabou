//! Optional application-owned GPU background composed below the Vello scene.

use std::{sync::Arc, time::Duration};

use vello::wgpu;

/// Factory used to create fresh GPU background state for every native surface.
pub type GpuBackgroundFactory = Arc<dyn Fn() -> Box<dyn GpuBackground> + Send + Sync>;

/// Per-frame GPU resources borrowed from Wabou's window renderer.
pub struct GpuBackgroundFrame<'a> {
    /// Device shared with the Vello renderer.
    pub device: &'a wgpu::Device,
    /// Queue shared with the Vello renderer.
    pub queue: &'a wgpu::Queue,
    /// Opaque full-window render target owned by Wabou for this frame.
    pub target: &'a wgpu::TextureView,
    /// Physical target width.
    pub width: u32,
    /// Physical target height.
    pub height: u32,
    /// Time since this native surface installed the background.
    pub elapsed: Duration,
    /// Texture format of `target`.
    pub format: wgpu::TextureFormat,
}

/// Application-defined full-window GPU paint composed below ordinary Vello UI.
pub trait GpuBackground {
    /// Encode one opaque background frame. Pipeline and resource creation should
    /// be retained by the implementation rather than repeated every frame.
    fn render(&mut self, frame: GpuBackgroundFrame<'_>) -> Result<(), String>;

    /// Keep the native window redrawing at vsync while this returns true.
    fn is_animated(&self) -> bool {
        true
    }
}

pub(crate) struct GpuBackgroundRenderer {
    effect: Box<dyn GpuBackground>,
    started: std::time::Instant,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    background: wgpu::TextureView,
    ui: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl GpuBackgroundRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        effect: Box<dyn GpuBackground>,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let (background, ui) = texture_views(device, width, height, format);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("wabou GPU background compositor sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wabou GPU background compositor layout"),
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
        let bind_group =
            compositor_bind_group(device, &bind_group_layout, &background, &ui, &sampler);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wabou GPU background compositor shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gpu_composite.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("wabou GPU background compositor pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wabou GPU background compositor pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
        });
        Self {
            effect,
            started: std::time::Instant::now(),
            width,
            height,
            format,
            background,
            ui,
            sampler,
            bind_group_layout,
            bind_group,
            pipeline,
        }
    }

    pub(crate) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        (self.background, self.ui) = texture_views(device, width, height, self.format);
        self.bind_group = compositor_bind_group(
            device,
            &self.bind_group_layout,
            &self.background,
            &self.ui,
            &self.sampler,
        );
    }

    pub(crate) fn is_animated(&self) -> bool {
        self.effect.is_animated()
    }

    pub(crate) fn ui_target(&self) -> &wgpu::TextureView {
        &self.ui
    }

    pub(crate) fn render_effect(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), String> {
        self.effect.render(GpuBackgroundFrame {
            device,
            queue,
            target: &self.background,
            width: self.width,
            height: self.height,
            elapsed: self.started.elapsed(),
            format: self.format,
        })
    }

    pub(crate) fn compose(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("wabou GPU background composite encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("wabou GPU background composite pass"),
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}

fn texture_views(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> (wgpu::TextureView, wgpu::TextureView) {
    let make = |label| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    };
    (
        make("wabou GPU background"),
        make("wabou transparent Vello UI"),
    )
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

fn compositor_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    background: &wgpu::TextureView,
    ui: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("wabou GPU background compositor bind group"),
        layout,
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
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
