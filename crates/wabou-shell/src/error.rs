use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("failed to create window: {source}"))]
    CreateWindow { source: winit::error::RequestError },

    #[snafu(display("failed to create window GPU surface: {source}"))]
    CreateSurfaceRenderer {
        source: wgpu_context::WgpuContextError,
    },

    #[snafu(display("failed to create offscreen GPU renderer: {source}"))]
    CreateBufferRenderer {
        source: wgpu_context::WgpuContextError,
    },

    #[snafu(display("failed to create Vello renderer: {source}"))]
    CreateVelloRenderer { source: vello::Error },

    #[snafu(display("failed to render scene: {source}"))]
    RenderScene { source: vello::Error },

    #[snafu(display("renderer returned an invalid {width}x{height} RGBA buffer"))]
    InvalidImageBuffer { width: u32, height: u32 },

    #[snafu(display("failed to save PNG {}: {source}", path.display()))]
    SavePng {
        path: PathBuf,
        source: image::ImageError,
    },

    #[snafu(display("failed to secure PNG {}: {source}", path.display()))]
    SecurePng {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to create event loop: {source}"))]
    CreateEventLoop {
        source: winit::error::EventLoopError,
    },

    #[snafu(display("event loop failed: {source}"))]
    RunEventLoop {
        source: winit::error::EventLoopError,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
