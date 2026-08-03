//! The shared error type for `plugin-host`.

use std::io;

use plugin_core::PluginError;
use thiserror::Error;

/// Everything that can go wrong on the host side of running a plugin.
///
/// Anything that's really a [`PluginError`] (connection failures, timeouts,
/// malformed responses, unknown-job lookups) is passed through as-is via
/// [`HostError::Plugin`] rather than re-invented here. This type only adds
/// the handful of failure modes that are specific to *hosting* a plugin
/// process, as opposed to talking to one that's already up and running:
/// spawning the child process, and the process dying unexpectedly.
#[derive(Debug, Error)]
pub enum HostError {
    /// Failed to spawn the plugin subprocess.
    #[error("failed to spawn plugin binary `{path}`: {source}")]
    Spawn {
        /// Path to the plugin binary that failed to spawn.
        path: String,
        /// The underlying I/O error from [`std::process::Command`].
        #[source]
        source: io::Error,
    },

    /// The plugin process exited (or was killed) before the host got a
    /// response it was waiting for.
    #[error("plugin process exited unexpectedly (exit code: {0:?})")]
    ProcessExited(Option<i32>),

    /// An I/O error occurred while managing an already-spawned plugin
    /// process (waiting on it, checking whether it's still alive, etc.) —
    /// distinct from [`HostError::Spawn`], which is specifically about the
    /// initial spawn call failing.
    #[error("I/O error managing plugin process: {0}")]
    Process(#[from] io::Error),

    /// Something went wrong at the plugin protocol level. See
    /// [`PluginError`] for the specific cases (connection failure, timeout,
    /// malformed response, unknown job, etc.).
    #[error(transparent)]
    Plugin(#[from] PluginError),
}

/// Shorthand Result type for `plugin-host` operations.
pub type HostResult<T> = std::result::Result<T, HostError>;

#[cfg(test)]
mod tests {
    use plugin_core::PluginError;

    use super::*;

    #[test]
    fn test_spawn_message() {
        let source = io::Error::new(io::ErrorKind::NotFound, "no such file or directory");
        let err = HostError::Spawn {
            path: "/usr/local/bin/my-plugin".into(),
            source,
        };
        assert_eq!(
            err.to_string(),
            "failed to spawn plugin binary `/usr/local/bin/my-plugin`: no such file or \
             directory"
        );
    }

    #[test]
    fn test_process_exited_with_code() {
        let err = HostError::ProcessExited(Some(1));
        assert_eq!(
            err.to_string(),
            "plugin process exited unexpectedly (exit code: Some(1))"
        );
    }

    #[test]
    fn test_process_exited_no_code() {
        let err = HostError::ProcessExited(None);
        assert_eq!(
            err.to_string(),
            "plugin process exited unexpectedly (exit code: None)"
        );
    }

    #[test]
    fn test_process_io_error_passthrough() {
        let source = io::Error::new(io::ErrorKind::Other, "no such process");
        let err: HostError = source.into();
        assert_eq!(
            err.to_string(),
            "I/O error managing plugin process: no such process"
        );
    }

    #[test]
    fn test_plugin_error_passthrough() {
        let err: HostError = PluginError::Timeout.into();
        assert_eq!(err.to_string(), "timeout");
    }
}
