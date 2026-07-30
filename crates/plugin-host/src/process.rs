//! Spawns the plugin binary and owns its process handle.

use std::process::ExitStatus;

use tokio::process::Child;
use tokio::process::Command;

use crate::config::HostConfig;
use crate::error::HostError;
use crate::error::HostResult;

/// A running plugin subprocess.
///
/// Wraps a [`tokio::process::Child`] with `kill_on_drop(true)` set
/// unconditionally, so a dropped [`PluginProcess`] — the host exiting
/// unexpectedly, a panic unwinding, whatever — never leaves a zombie
/// plugin process behind.
///
/// This type only owns the process handle. It has nothing to do with
/// *talking* to the plugin (see [`crate::rpc`]) or deciding when to give up
/// waiting for it to come up (see [`crate::poll`]).
#[derive(Debug)]
pub struct PluginProcess {
    child: Child,
}

impl PluginProcess {
    /// Spawns the plugin binary described by `config`.
    pub fn spawn(config: &HostConfig) -> HostResult<Self> {
        let child = Command::new(config.plugin_path())
            .args(config.plugin_args())
            .kill_on_drop(true)
            .spawn()
            .map_err(|source| HostError::Spawn {
                path: config.plugin_path().display().to_string(),
                source,
            })?;

        Ok(Self { child })
    }

    /// The OS process id of the spawned plugin.
    ///
    /// Returns `None` if the process has already been polled to
    /// completion via [`Self::try_wait`] or [`Self::wait`].
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    /// Checks whether the plugin process has exited, without blocking.
    ///
    /// Returns `Ok(None)` if the process is still running. Meant to be
    /// polled alongside RPC status checks so the host can tell "the plugin
    /// hasn't responded yet" apart from "the plugin process is gone".
    pub fn try_wait(&mut self) -> HostResult<Option<ExitStatus>> {
        Ok(self.child.try_wait()?)
    }

    /// Waits for the plugin process to exit, blocking until it does.
    pub async fn wait(&mut self) -> HostResult<ExitStatus> {
        Ok(self.child.wait().await?)
    }

    /// Kills the plugin process immediately.
    ///
    /// This is a last resort — normal shutdown should go through the RPC
    /// protocol so the plugin gets a chance to clean up. This exists for
    /// cases where the plugin isn't responding at all.
    pub async fn kill(&mut self) -> HostResult<()> {
        Ok(self.child.kill().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn test_spawn_and_wait_success() {
        let config = HostConfig::new("/bin/true");
        let mut process = PluginProcess::spawn(&config).expect("should spawn");
        let status = process.wait().await.expect("should wait");
        assert!(status.success());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_try_wait_before_exit() {
        let config = HostConfig::new("/bin/sleep").with_args(["1"]);
        let mut process = PluginProcess::spawn(&config).expect("should spawn");
        let status = process
            .try_wait()
            .expect("try_wait should not error while the process is alive");
        assert!(
            status.is_none(),
            "process should still be running immediately after spawn"
        );
        process.kill().await.expect("should kill");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_kill() {
        let config = HostConfig::new("/bin/sleep").with_args(["30"]);
        let mut process = PluginProcess::spawn(&config).expect("should spawn");
        process.kill().await.expect("should kill");
        let status = process.wait().await.expect("should wait after kill");
        assert!(!status.success());
    }

    #[test]
    fn test_spawn_missing_binary_returns_error() {
        let config = HostConfig::new("/definitely/does/not/exist/binary");
        let err = PluginProcess::spawn(&config).expect_err("spawn should fail for missing binary");
        assert!(matches!(err, HostError::Spawn { .. }));
    }
}