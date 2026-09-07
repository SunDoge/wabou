//! A [`vello_hybrid`] backend for the [`anyrender`] 2D drawing abstraction
#![cfg_attr(docsrs, feature(doc_cfg))]

mod filters;
#[cfg(not(target_arch = "wasm32"))]
mod image_renderer;
mod scene;
#[cfg(all(target_arch = "wasm32", feature = "webgl"))]
mod webgl_scene;
mod window_renderer;

pub use scene::ImageManager;
pub use scene::VelloHybridScenePainter;
#[cfg(not(target_arch = "wasm32"))]
pub use image_renderer::VelloHybridImageRenderer;
#[cfg(all(target_arch = "wasm32", feature = "webgl"))]
pub use webgl_scene::*;
pub use window_renderer::*;

pub use wgpu;
