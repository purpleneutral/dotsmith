use thiserror::Error;

#[derive(Debug, Error)]
pub enum DotsmithError {
    #[error("tool '{0}' is already managed by dotsmith")]
    ToolAlreadyTracked(String),

    #[error("tool '{0}' is not managed by dotsmith")]
    ToolNotTracked(String),

    #[error("tool '{0}' is not installed (command '{1}' failed)")]
    ToolNotInstalled(String, String),

    #[error("no config files found for '{0}'")]
    NoConfigFound(String),

    #[error("dotsmith is not initialized — run `dotsmith init` first")]
    NotInitialized,

    #[error("dotsmith is already initialized at {0}")]
    AlreadyInitialized(String),

    #[error("path '{path}' resolves outside your home directory to '{resolved}'")]
    #[allow(dead_code)]
    PathTraversal { path: String, resolved: String },
}
