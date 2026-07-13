use crate::job::JobId;

/// All the ways a plugin interaction can fail.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("job not found: {0}")]
    JobNotFound(JobId),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("timeout")]
    Timeout,

    #[error("unknown error: {0}")]
    Unknown(String),
}

/// Shorthand Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;
