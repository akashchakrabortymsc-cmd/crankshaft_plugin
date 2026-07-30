//! Host-side configuration for spawning and talking to a plugin.

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for spawning a plugin process and talking to it over
/// JSON-RPC/TCP.
///
/// Defaults follow the plan from the original design discussion: 5 connect
/// attempts, 500ms apart, while the plugin's TCP listener comes up after
/// spawn.
#[derive(Debug, Clone)]
pub struct HostConfig {
    /// Path to the plugin binary to spawn.
    plugin_path: PathBuf,

    /// Arguments passed to the plugin binary on spawn.
    plugin_args: Vec<String>,

    /// How many times to retry connecting to the plugin's TCP listener
    /// after spawning it, before giving up.
    connect_retries: u32,

    /// How long to wait between connection retry attempts.
    connect_retry_delay: Duration,

    /// How often to poll the plugin for job status once a job has been
    /// submitted.
    poll_interval: Duration,

    /// How long to wait for a single JSON-RPC round trip before treating it
    /// as a timeout.
    rpc_timeout: Duration,
}

impl HostConfig {
    /// Creates a new [`HostConfig`] with sensible defaults, given the path
    /// to the plugin binary.
    pub fn new(plugin_path: impl Into<PathBuf>) -> Self {
        Self {
            plugin_path: plugin_path.into(),
            plugin_args: Vec::new(),
            connect_retries: 5,
            connect_retry_delay: Duration::from_millis(500),
            poll_interval: Duration::from_secs(1),
            rpc_timeout: Duration::from_secs(30),
        }
    }

    /// Sets the arguments passed to the plugin binary on spawn.
    pub fn with_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.plugin_args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Sets how many times to retry connecting to the plugin's TCP
    /// listener after spawning it.
    pub fn with_connect_retries(mut self, retries: u32) -> Self {
        self.connect_retries = retries;
        self
    }

    /// Sets the delay between connection retry attempts.
    pub fn with_connect_retry_delay(mut self, delay: Duration) -> Self {
        self.connect_retry_delay = delay;
        self
    }

    /// Sets how often to poll the plugin for job status.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Sets the timeout for a single JSON-RPC round trip.
    pub fn with_rpc_timeout(mut self, timeout: Duration) -> Self {
        self.rpc_timeout = timeout;
        self
    }

    /// The path to the plugin binary.
    pub fn plugin_path(&self) -> &Path {
        &self.plugin_path
    }

    /// The arguments passed to the plugin binary on spawn.
    pub fn plugin_args(&self) -> &[String] {
        &self.plugin_args
    }

    /// How many times to retry connecting to the plugin's TCP listener.
    pub fn connect_retries(&self) -> u32 {
        self.connect_retries
    }

    /// The delay between connection retry attempts.
    pub fn connect_retry_delay(&self) -> Duration {
        self.connect_retry_delay
    }

    /// How often to poll the plugin for job status.
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    /// The timeout for a single JSON-RPC round trip.
    pub fn rpc_timeout(&self) -> Duration {
        self.rpc_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let config = HostConfig::new("/usr/local/bin/my-plugin");
        assert_eq!(config.plugin_path(), Path::new("/usr/local/bin/my-plugin"));
        assert!(config.plugin_args().is_empty());
        assert_eq!(config.connect_retries(), 5);
        assert_eq!(config.connect_retry_delay(), Duration::from_millis(500));
        assert_eq!(config.poll_interval(), Duration::from_secs(1));
        assert_eq!(config.rpc_timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_builder_chain() {
        let config = HostConfig::new("plugin-bin")
            .with_args(["--verbose"])
            .with_connect_retries(10)
            .with_connect_retry_delay(Duration::from_millis(100))
            .with_poll_interval(Duration::from_millis(250))
            .with_rpc_timeout(Duration::from_secs(5));

        assert_eq!(config.plugin_args(), &["--verbose".to_string()]);
        assert_eq!(config.connect_retries(), 10);
        assert_eq!(config.connect_retry_delay(), Duration::from_millis(100));
        assert_eq!(config.poll_interval(), Duration::from_millis(250));
        assert_eq!(config.rpc_timeout(), Duration::from_secs(5));
    }
}