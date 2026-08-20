//! Errors returned while configuring or running the QuickJS host.

use std::path::PathBuf;

use snafu::Snafu;

/// Failure at an application bundle, JavaScript, shell, or tooling boundary.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// A required bundle, source map, or configuration file could not be read.
    #[snafu(display("failed to read {kind} {}: {source}", path.display()))]
    ReadFile {
        /// Human-readable file purpose.
        kind: &'static str,
        /// Requested filesystem path.
        path: PathBuf,
        /// Underlying filesystem failure.
        source: std::io::Error,
    },

    /// QuickJS failed while performing a named lifecycle operation.
    #[snafu(display("failed to {operation}: {source}"))]
    JavaScript {
        /// Operation being attempted.
        operation: &'static str,
        /// QuickJS exception or runtime error.
        source: rquickjs::Error,
    },

    /// A required CLI or host argument was absent.
    #[snafu(display("missing required argument: {argument}"))]
    MissingArgument {
        /// Missing argument name.
        argument: &'static str,
    },

    /// Local DevTools transport failed to start.
    #[snafu(display("failed to start DevTools: {source}"))]
    Devtools {
        /// Socket or filesystem failure.
        source: std::io::Error,
    },

    /// Native window/render host failed.
    #[snafu(display("window host failed: {source}"))]
    Shell {
        /// Underlying shell error.
        source: wabou_shell::Error,
    },

    /// Effect recording or replay failed validation or I/O.
    #[snafu(display("effect trace failed: {message}"))]
    EffectTrace {
        /// Human-readable trace diagnostic.
        message: String,
    },

    /// Platform application directories could not be resolved.
    #[snafu(display("cannot determine application directories for {application}"))]
    AppDirectories {
        /// Application identity whose paths were requested.
        application: String,
    },

    /// An application-owned background service failed to start.
    #[snafu(display("host service `{name}` failed to start: {message}"))]
    HostService {
        /// Stable diagnostic name supplied by the service.
        name: &'static str,
        /// Service-specific startup failure.
        message: String,
    },

    /// One or more application-owned services failed during orderly shutdown.
    #[snafu(display("host service shutdown failed: {message}"))]
    HostServiceShutdown {
        /// All shutdown failures, after every service received a stop request.
        message: String,
    },

    /// Built-in behavior test scenario reported failure.
    #[snafu(display("test scenario failed: {message}"))]
    TestScenario {
        /// Serialized scenario diagnostic.
        message: String,
    },

    /// Vite development integration failed.
    #[cfg(feature = "vite")]
    #[snafu(display("Vite integration failed: {source}"))]
    Vite {
        /// Underlying Vite client error.
        source: crate::vite::ViteError,
    },
}

/// Result type returned by QuickJS host operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;
