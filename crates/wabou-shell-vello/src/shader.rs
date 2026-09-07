//! Validated custom WGSL effects and their renderer-owned GPU resources.

use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use lru::LruCache;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use rustc_hash::FxHashMap;
use vello_common::TextureId;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, Queue, RenderPipeline, Texture,
    TextureFormat, TextureView,
};

const MAX_SOURCE_BYTES: usize = 64 * 1024;
const PARAMETER_COUNT: usize = 16;
const PIPELINE_CACHE_SIZE: usize = 16;
const EFFECT_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

const SHADER_PREFIX: &str = r#"
struct WabouShaderUniforms {
    // x/y: physical output size, z: elapsed seconds, w: device scale.
    resolution_time_scale: vec4<f32>,
    values: array<vec4<f32>, 4>,
};

@group(0) @binding(0)
var<uniform> wabou: WabouShaderUniforms;

struct WabouVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> WabouVertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let position = positions[vertex_index];
    var output: WabouVertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = position * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return output;
}
"#;

const SHADER_SUFFIX: &str = r#"
@fragment
fn fs_main(input: WabouVertexOutput) -> @location(0) vec4<f32> {
    return wabou_effect(input.uv);
}
"#;

static NEXT_EFFECT_ID: AtomicU64 = AtomicU64::new(1);

/// Stable identity for one shader surface.
///
/// The renderer uses this identity to retain its output texture across frames
/// and drops the texture after the widget no longer contributes paint.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ShaderEffectId(u64);

impl ShaderEffectId {
    /// Allocate a process-unique shader surface identity.
    pub fn new() -> Self {
        Self(NEXT_EFFECT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ShaderEffectId {
    fn default() -> Self {
        Self::new()
    }
}

/// A validated WGSL function compiled under Wabou's stable shader prelude.
///
/// Source must define `fn wabou_effect(uv: vec2<f32>) -> vec4<f32>`. It can
/// read `wabou.resolution_time_scale` and the sixteen scalar values packed in
/// `wabou.values`. Extra bind groups are deliberately rejected so renderer
/// resource ownership remains explicit.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ShaderEffectSource(Arc<str>);

impl ShaderEffectSource {
    /// Validate and retain a custom WGSL effect function.
    pub fn new(source: &str) -> Result<Self, String> {
        if source.trim().is_empty() {
            return Err("shader source must not be empty".into());
        }
        if source.len() > MAX_SOURCE_BYTES {
            return Err(format!(
                "shader source exceeds the {MAX_SOURCE_BYTES}-byte limit"
            ));
        }
        let complete = format!("{SHADER_PREFIX}\n{source}\n{SHADER_SUFFIX}");
        let module = naga::front::wgsl::parse_str(&complete)
            .map_err(|error| error.emit_to_string(&complete))?;
        Validator::new(ValidationFlags::all(), Capabilities::default())
            .validate(&module)
            .map_err(|error| format!("invalid shader interface: {error}"))?;
        for (_, variable) in module.global_variables.iter() {
            if let Some(binding) = &variable.binding
                && (binding.group != 0 || binding.binding != 0)
            {
                return Err(format!(
                    "custom shader binding @group({}) @binding({}) is unsupported; use wabou.values",
                    binding.group, binding.binding
                ));
            }
        }
        Ok(Self(complete.into()))
    }

    pub(crate) fn wgsl(&self) -> &Arc<str> {
        &self.0
    }
}

/// One resolved custom-shader draw retained in Wabou's paint stream.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderEffect {
    pub(crate) id: ShaderEffectId,
    pub(crate) source: ShaderEffectSource,
    pub(crate) time: f32,
    pub(crate) values: [f32; PARAMETER_COUNT],
    pub(crate) physical_size: [u16; 2],
    pub(crate) logical_size: [f32; 2],
    pub(crate) device_scale: f32,
}

impl ShaderEffect {
    /// Output texture dimensions in physical pixels.
    pub fn physical_size(&self) -> [u16; 2] {
        self.physical_size
    }

    /// Destination dimensions in logical pixels.
    pub fn logical_size(&self) -> [f32; 2] {
        self.logical_size
    }
}

struct EffectTarget {
    _texture: Texture,
    view: TextureView,
    uniforms: Buffer,
    bind_group: BindGroup,
    size: [u16; 2],
    used_in_frame: u64,
}

impl EffectTarget {
    fn new(device: &Device, layout: &BindGroupLayout, size: [u16; 2]) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Wabou custom shader target"),
            size: wgpu::Extent3d {
                width: u32::from(size[0]),
                height: u32::from(size[1]),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: EFFECT_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Wabou custom shader uniforms"),
            size: (20 * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Wabou custom shader bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });
        Self {
            _texture: texture,
            view,
            uniforms,
            bind_group,
            size,
            used_in_frame: 0,
        }
    }
}

pub(crate) struct ShaderRenderer {
    layout: BindGroupLayout,
    pipelines: LruCache<Arc<str>, Arc<RenderPipeline>>,
    targets: FxHashMap<ShaderEffectId, EffectTarget>,
    frame: u64,
}

impl ShaderRenderer {
    pub(crate) fn new(device: &Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Wabou custom shader layout"),
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
        Self {
            layout,
            pipelines: LruCache::new(NonZeroUsize::new(PIPELINE_CACHE_SIZE).unwrap()),
            targets: FxHashMap::default(),
            frame: 0,
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1).max(1);
    }

    pub(crate) fn finish_frame(&mut self) {
        let frame = self.frame;
        self.targets
            .retain(|_, target| target.used_in_frame == frame);
    }

    pub(crate) fn render(
        &mut self,
        effect: &ShaderEffect,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        bindings: &mut vello_hybrid::TextureBindings,
    ) -> TextureId {
        let pipeline = self.pipeline(device, &effect.source);
        let target = self
            .targets
            .entry(effect.id)
            .and_modify(|target| {
                if target.size != effect.physical_size {
                    *target = EffectTarget::new(device, &self.layout, effect.physical_size);
                }
            })
            .or_insert_with(|| EffectTarget::new(device, &self.layout, effect.physical_size));
        target.used_in_frame = self.frame;

        let mut uniforms = [0.0_f32; 20];
        uniforms[0] = f32::from(effect.physical_size[0]);
        uniforms[1] = f32::from(effect.physical_size[1]);
        uniforms[2] = effect.time;
        uniforms[3] = effect.device_scale;
        uniforms[4..].copy_from_slice(&effect.values);
        queue.write_buffer(&target.uniforms, 0, bytemuck::cast_slice(&uniforms));

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Wabou custom shader pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
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
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &target.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        let texture_id = TextureId(effect.id.0);
        bindings.insert(texture_id, target.view.clone());
        texture_id
    }

    fn pipeline(&mut self, device: &Device, source: &ShaderEffectSource) -> Arc<RenderPipeline> {
        if let Some(pipeline) = self.pipelines.get(source.wgsl()) {
            return pipeline.clone();
        }
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Wabou custom shader"),
            source: wgpu::ShaderSource::Wgsl(source.wgsl().as_ref().into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wabou custom shader pipeline layout"),
            bind_group_layouts: &[Some(&self.layout)],
            immediate_size: 0,
        });
        let pipeline = Arc::new(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Wabou custom shader pipeline"),
                layout: Some(&layout),
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
                        format: EFFECT_TEXTURE_FORMAT,
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
            }),
        );
        self.pipelines.put(source.wgsl().clone(), pipeline.clone());
        pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_the_public_effect_function_contract() {
        ShaderEffectSource::new(
            "fn wabou_effect(uv: vec2<f32>) -> vec4<f32> { return vec4<f32>(uv, 0.0, 1.0); }",
        )
        .unwrap();
        assert!(ShaderEffectSource::new("fn nope() {}").is_err());
        assert!(ShaderEffectSource::new("").is_err());
    }

    #[test]
    fn rejects_resources_outside_the_uniform_contract() {
        let error = ShaderEffectSource::new(
            "@group(1) @binding(0) var image: texture_2d<f32>;\nfn wabou_effect(uv: vec2<f32>) -> vec4<f32> { return textureLoad(image, vec2<i32>(uv), 0); }",
        )
        .unwrap_err();
        assert!(error.contains("@group(1) @binding(0)"), "{error}");
    }
}
