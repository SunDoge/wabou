//! Errors returned while creating or running the native shell.

#![warn(missing_docs)]

use std::path::PathBuf;

use snafu::Snafu;

/// Failure at a native window, renderer, event-loop, or extension boundary.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// The selected renderer was not compiled into this Wabou build.
    #[snafu(display("renderer backend `{backend}` is unavailable; enable {feature}"))]
    RendererBackendUnavailable {
        /// Requested backend name.
        backend: &'static str,
        /// Cargo feature which enables it.
        feature: &'static str,
    },
    /// Native window creation failed.
    #[snafu(display("failed to create window: {source}"))]
    CreateWindow {
        /// Winit request failure.
        source: winit::error::RequestError,
    },

    /// Renderer output length did not match its declared dimensions.
    #[snafu(display("renderer returned an invalid {width}x{height} RGBA buffer"))]
    InvalidImageBuffer {
        /// Declared image width.
        width: u32,
        /// Declared image height.
        height: u32,
    },

    /// Encoding or writing an offscreen PNG failed.
    #[snafu(display("failed to save PNG {}: {source}", path.display()))]
    SavePng {
        /// Requested output path.
        path: PathBuf,
        /// Image encoding or I/O failure.
        source: image::ImageError,
    },

    /// Applying private permissions to an offscreen PNG failed.
    #[snafu(display("failed to secure PNG {}: {source}", path.display()))]
    SecurePng {
        /// PNG whose permissions could not be changed.
        path: PathBuf,
        /// Filesystem failure.
        source: std::io::Error,
    },

    /// Winit event-loop construction failed.
    #[snafu(display("failed to create event loop: {source}"))]
    CreateEventLoop {
        /// Winit event-loop failure.
        source: winit::error::EventLoopError,
    },

    /// Winit event-loop execution failed.
    #[snafu(display("event loop failed: {source}"))]
    RunEventLoop {
        /// Winit event-loop failure.
        source: winit::error::EventLoopError,
    },

    /// Application-provided shell extension failed during initialization.
    #[snafu(display("shell extension failed to initialize: {message}"))]
    Extension {
        /// Extension-provided diagnostic.
        message: String,
    },
}

/// Result type returned by shell operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;
