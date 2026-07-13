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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PluginError;
    use crate::job::{Job, JobId};
    use crate::status::JobStatus;
    use async_trait::async_trait;

    struct MockPlugin;

    #[async_trait]
    impl PluginHandler for MockPlugin {
        async fn submit(&self, job: Job) -> PluginResult<JobId> {
            Ok(job.id)
        }
        async fn status(&self, _id: JobId) -> PluginResult<JobStatus> {
            Ok(JobStatus::Completed)
        }
        async fn cancel(&self, _id: JobId) -> PluginResult<()> {
            Ok(())
        }
    }

    struct FailingPlugin;

    #[async_trait]
    impl PluginHandler for FailingPlugin {
        async fn submit(&self, _job: Job) -> PluginResult<JobId> {
            Err(PluginError::ConnectionFailed("unreachable".into()))
        }
        async fn status(&self, id: JobId) -> PluginResult<JobStatus> {
            Err(PluginError::JobNotFound(id))
        }
        async fn cancel(&self, _id: JobId) -> PluginResult<()> {
            Err(PluginError::Unknown("cannot cancel".into()))
        }
    }

    #[tokio::test]
    async fn test_submit_returns_job_id() {
        let plugin = MockPlugin;
        let id = JobId::new("job-001");
        let job = Job::new(id.clone(), "echo");
        let result = plugin.submit(job).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);
    }

    #[tokio::test]
    async fn test_status_returns_completed() {
        let plugin = MockPlugin;
        let id = JobId::new("job-001");
        let result = plugin.status(id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JobStatus::Completed);
    }

    #[tokio::test]
    async fn test_cancel_returns_ok() {
        let plugin = MockPlugin;
        let id = JobId::new("job-001");
        let result = plugin.cancel(id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check_default_ok() {
        let plugin = MockPlugin;
        let result = plugin.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_submit_connection_failed() {
        let plugin = FailingPlugin;
        let id = JobId::new("job-002");
        let job = Job::new(id, "echo");
        let result = plugin.submit(job).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginError::ConnectionFailed(_)
        ));
    }

    #[tokio::test]
    async fn test_status_job_not_found() {
        let plugin = FailingPlugin;
        let id = JobId::new("job-999");
        let result = plugin.status(id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::JobNotFound(_)));
    }

    #[tokio::test]
    async fn test_cancel_unknown_error() {
        let plugin = FailingPlugin;
        let id = JobId::new("job-999");
        let result = plugin.cancel(id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::Unknown(_)));
    }
}
