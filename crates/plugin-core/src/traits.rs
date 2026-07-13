use crate::error::PluginResult;
use crate::job::{Job, JobId};
use crate::status::JobStatus;
use async_trait::async_trait;

/// The main contract every plugin must implement.
#[async_trait]
pub trait PluginHandler: Send + Sync + 'static {
    async fn submit(&self, job: Job) -> PluginResult<JobId>;
    async fn status(&self, id: JobId) -> PluginResult<JobStatus>;
    async fn cancel(&self, id: JobId) -> PluginResult<()>;

    /// Health check — called periodically by the host.
    async fn health_check(&self) -> PluginResult<()> {
        Ok(())
    }
}
