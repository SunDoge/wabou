use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("failed to read {kind} {}: {source}", path.display()))]
    ReadFile {
        kind: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to {operation}: {source}"))]
    JavaScript {
        operation: &'static str,
        source: rquickjs::Error,
    },

    #[snafu(display("missing required argument: {argument}"))]
    MissingArgument { argument: &'static str },

    #[snafu(display("failed to start DevTools: {source}"))]
    Devtools { source: std::io::Error },

    #[snafu(display("window host failed: {source}"))]
    Shell { source: wabou_shell::Error },

    #[cfg(feature = "vite")]
    #[snafu(display("Vite integration failed: {source}"))]
    Vite { source: crate::vite::ViteError },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
