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

    #[error("plugin '{0}' is already installed for {1}")]
    PluginAlreadyInstalled(String, String),

    #[error("plugin '{0}' is not installed for {1}")]
    PluginNotInstalled(String, String),

    #[error("no init file detected in plugin '{0}' — expected {1}")]
    PluginNoInitFile(String, String),

    #[error("git is not installed — required for plugin management")]
    GitNotInstalled,

    #[error("git clone failed for '{0}': {1}")]
    GitCloneFailed(String, String),

    #[error("git pull failed for '{0}': {1}")]
    GitPullFailed(String, String),

    #[error("invalid plugin repository specifier: '{0}'")]
    InvalidPluginRepo(String),

    #[error("plugin management is not supported for '{0}'")]
    PluginsNotSupported(String),

    #[error("profile '{0}' already exists — use a different name or delete it first")]
    ProfileAlreadyExists(String),

    #[error("profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("invalid profile name '{0}' — use only letters, digits, hyphens, and underscores")]
    InvalidProfileName(String),

    #[error("ssh is not installed — required for remote deploy")]
    SshNotInstalled,

    #[error("scp to '{0}' failed for file '{1}': {2}")]
    ScpFailed(String, String, String),
}
