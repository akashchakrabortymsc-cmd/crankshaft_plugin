//! Shared types and contracts for the Crankshaft plugin system.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ─── JobId ─────
/// A unique identifier for a submitted job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

impl JobId {
    /// Creates a new JobId.
    pub fn new(id: impl Into<String>) -> Self {
        JobId(id.into())
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Resources ─────
/// Resource requirements for a job.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    /// Number of CPUs requested.
    pub cpus: Option<f64>,
    /// Memory in megabytes.
    pub memory_mb: Option<u64>,
    /// Number of GPUs requested (future).
    pub gpus: Option<u32>,
}

// ─── JobStatus ─────
/// The current state of a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Submitted but not yet running.
    Pending,
    /// Actively executing.
    Running,
    /// Finished successfully.
    Completed,
    /// Finished with an error (with message).
    Failed(String),
    /// Stopped before completion.
    Cancelled,
}

// ─── Job ───
/// The unit of work sent to a plugin for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job.
    pub id: JobId,
    /// Human readable name of the job.
    pub name: String,
    /// Program / executable to run.
    pub program: String,
    /// Arguments for the program.
    pub args: Vec<String>,
    /// Environment variables.
    pub environment: HashMap<String, String>,
    /// Working directory.
    pub work_dir: Option<String>,
    /// Resource requirements.
    pub resources: Option<Resources>,
    /// Optional execution timeout.
    pub timeout: Option<Duration>,
}

impl Job {
    /// Creates a new simple Job.
    pub fn new(id: JobId, program: String) -> Self {
        Job {
            id,
            name: String::new(),
            program,
            args: Vec::new(),
            environment: HashMap::new(),
            work_dir: None,
            resources: None,
            timeout: None,
        }
    }
}

// ─── PluginError ──────
/// All the ways a plugin interaction can fail.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Could not reach the plugin process.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    /// No job with this ID exists.
    #[error("job not found: {0}")]
    JobNotFound(String),
    /// Plugin sent unexpected data.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// Plugin did not respond in time.
    #[error("timeout")]
    Timeout,
    /// Catch-all for unexpected errors.
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// Shorthand Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

// ─── PluginHandler trait ───────
/// The main contract every plugin must implement.
///
/// This is the core trait that all external execution backends must satisfy.
#[async_trait]
pub trait PluginHandler: Send + Sync + 'static {
    /// Submit a job for execution. Returns a JobId.
    async fn submit(&self, job: Job) -> PluginResult<JobId>;

    /// Get the current status of a job.
    async fn status(&self, id: JobId) -> PluginResult<JobStatus>;

    /// Cancel a running job.
    async fn cancel(&self, id: JobId) -> PluginResult<()>;

    /// Health check - called periodically by the host.
    async fn health_check(&self) -> PluginResult<()> {
        Ok(())
    }
}

// ─── Tests ────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_display() {
        let id = JobId::new("job-123");
        assert_eq!(format!("{}", id), "job-123");
    }

    #[test]
    fn test_job_creation() {
        let id = JobId::new("job-001");
        let job = Job::new(id.clone(), "echo".to_string());
        assert_eq!(job.program, "echo");
        assert_eq!(job.id, id);
    }

    #[test]
    fn test_job_default_values() {
        let id = JobId::new("job-002");
        let job = Job::new(id, "ls".to_string());
        assert!(job.args.is_empty());
        assert!(job.environment.is_empty());
        assert!(job.work_dir.is_none());
    }
}