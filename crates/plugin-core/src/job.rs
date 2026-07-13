use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// A unique identifier for a submitted job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

impl JobId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Resource requirements for a job.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    /// Number of CPU cores requested.
    pub cpu: Option<f64>,
    /// RAM in gibibytes (GiB) — matches Crankshaft's Resources.
    pub ram_gb: Option<f64>,
    /// Disk in gibibytes (GiB) — matches Crankshaft's Resources.
    pub disk_gb: Option<f64>,
    /// Number of GPUs requested.
    pub gpu: Option<u64>,
    /// Whether preemptible resources may be used.
    pub preemptible: Option<bool>,
}

/// The unit of work sent to a plugin for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job.
    pub id: JobId,
    /// Human-readable name.
    pub name: String,
    /// Container image to run inside (optional).
    pub image: Option<String>,
    /// Program / executable to run.
    pub program: String,
    /// Arguments for the program.
    pub args: Vec<String>,
    /// Environment variables.
    pub environment: HashMap<String, String>,
    /// Path to a file to pipe into stdin.
    pub stdin: Option<String>,
    /// Path to a file to write stdout into.
    pub stdout: Option<String>,
    /// Path to a file to write stderr into.
    pub stderr: Option<String>,
    /// Working directory inside the container.
    pub work_dir: Option<String>,
    /// Resource requirements.
    pub resources: Option<Resources>,
    /// Optional execution timeout.
    pub timeout: Option<Duration>,
}

impl Job {
    /// Creates a minimal Job with only required fields.
    pub fn new(id: JobId, program: impl Into<String>) -> Self {
        Job {
            id,
            name: String::new(),
            image: None,
            program: program.into(),
            args: Vec::new(),
            environment: HashMap::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            work_dir: None,
            resources: None,
            timeout: None,
        }
    }

    /// Sets the human-readable name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the container image.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Sets the program arguments.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Adds a single environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Sets the stdin file path.
    pub fn with_stdin(mut self, path: impl Into<String>) -> Self {
        self.stdin = Some(path.into());
        self
    }

    /// Sets the stdout file path.
    pub fn with_stdout(mut self, path: impl Into<String>) -> Self {
        self.stdout = Some(path.into());
        self
    }

    /// Sets the stderr file path.
    pub fn with_stderr(mut self, path: impl Into<String>) -> Self {
        self.stderr = Some(path.into());
        self
    }

    /// Sets the working directory.
    pub fn with_work_dir(mut self, dir: impl Into<String>) -> Self {
        self.work_dir = Some(dir.into());
        self
    }

    /// Sets resource requirements.
    pub fn with_resources(mut self, resources: Resources) -> Self {
        self.resources = Some(resources);
        self
    }

    /// Sets the execution timeout.
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_job_id_display() {
        let id = JobId::new("job-123");
        assert_eq!(format!("{}", id), "job-123");
    }

    #[test]
    fn test_job_id_as_str() {
        let id = JobId::new("job-abc");
        assert_eq!(id.as_str(), "job-abc");
    }

    #[test]
    fn test_job_id_equality() {
        let a = JobId::new("job-1");
        let b = JobId::new("job-1");
        let c = JobId::new("job-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_job_new_defaults() {
        let id = JobId::new("job-001");
        let job = Job::new(id.clone(), "echo");
        assert_eq!(job.id, id);
        assert_eq!(job.program, "echo");
        assert!(job.name.is_empty());
        assert!(job.image.is_none());
        assert!(job.args.is_empty());
        assert!(job.environment.is_empty());
        assert!(job.stdin.is_none());
        assert!(job.stdout.is_none());
        assert!(job.stderr.is_none());
        assert!(job.work_dir.is_none());
        assert!(job.resources.is_none());
        assert!(job.timeout.is_none());
    }

    #[test]
    fn test_job_builder_chain() {
        let id = JobId::new("job-002");
        let job = Job::new(id, "python3")
            .with_name("my-job")
            .with_image("python:3.11")
            .with_args(vec!["script.py".into()])
            .with_env("ENV", "prod")
            .with_stdin("/tmp/in.txt")
            .with_stdout("/tmp/out.txt")
            .with_stderr("/tmp/err.txt")
            .with_work_dir("/workspace")
            .with_timeout(Duration::from_secs(60));

        assert_eq!(job.name, "my-job");
        assert_eq!(job.image.as_deref(), Some("python:3.11"));
        assert_eq!(job.args, vec!["script.py"]);
        assert_eq!(job.environment.get("ENV").map(|s| s.as_str()), Some("prod"));
        assert_eq!(job.stdin.as_deref(), Some("/tmp/in.txt"));
        assert_eq!(job.stdout.as_deref(), Some("/tmp/out.txt"));
        assert_eq!(job.stderr.as_deref(), Some("/tmp/err.txt"));
        assert_eq!(job.work_dir.as_deref(), Some("/workspace"));
        assert_eq!(job.timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_resources_defaults() {
        let r = Resources::default();
        assert!(r.cpu.is_none());
        assert!(r.ram_gb.is_none());
        assert!(r.disk_gb.is_none());
        assert!(r.gpu.is_none());
        assert!(r.preemptible.is_none());
    }

    #[test]
    fn test_job_with_resources() {
        let id = JobId::new("job-003");
        let res = Resources {
            cpu: Some(4.0),
            ram_gb: Some(16.0),
            disk_gb: Some(100.0),
            gpu: Some(1),
            preemptible: Some(false),
        };
        let job = Job::new(id, "train.sh").with_resources(res);
        let r = job.resources.unwrap();
        assert_eq!(r.cpu, Some(4.0));
        assert_eq!(r.ram_gb, Some(16.0));
        assert_eq!(r.disk_gb, Some(100.0));
        assert_eq!(r.gpu, Some(1));
        assert_eq!(r.preemptible, Some(false));
    }

    #[test]
    fn test_job_serialization() {
        let id = JobId::new("job-ser-01");
        let job = Job::new(id, "echo").with_name("test");
        let json = serde_json::to_string(&job).expect("serialize failed");
        let back: Job = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.name, "test");
        assert_eq!(back.program, "echo");
    }
}
