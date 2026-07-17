use std::collections::HashMap;
use std::time::Duration;

use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};

/// A unique identifier for a submitted job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(String);

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

/// Resource requirements for a job. Field-for-field aligned with
/// crankshaft_engine::task::Resources (units matter: ram/disk are GiB, not MB).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    pub cpu: Option<f64>,
    pub cpu_limit: Option<f64>,
    /// RAM in GiB.
    pub ram: Option<f64>,
    pub ram_limit: Option<f64>,
    /// Disk in GiB.
    pub disk: Option<f64>,
    pub preemptible: Option<bool>,
    pub zones: Vec<String>,
    pub gpu: Option<u64>,
}

/// One executable step within a job. Mirrors crankshaft_engine::task::Execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    pub image: String,
    pub program: String,
    pub args: Vec<String>,
    pub work_dir: Option<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub env: HashMap<String, String>,
}

impl JobExecution {
    pub fn new(image: impl Into<String>, program: impl Into<String>) -> Self {
        JobExecution {
            image: image.into(),
            program: program.into(),
            args: Vec::new(),
            work_dir: None,
            stdin: None,
            stdout: None,
            stderr: None,
            env: HashMap::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn with_work_dir(mut self, dir: impl Into<String>) -> Self {
        self.work_dir = Some(dir.into());
        self
    }

    pub fn with_stdin(mut self, path: impl Into<String>) -> Self {
        self.stdin = Some(path.into());
        self
    }

    pub fn with_stdout(mut self, path: impl Into<String>) -> Self {
        self.stdout = Some(path.into());
        self
    }

    pub fn with_stderr(mut self, path: impl Into<String>) -> Self {
        self.stderr = Some(path.into());
        self
    }
}

/// The unit of work sent to a plugin for execution.
///
/// Holds one-or-more executions to match Crankshaft's `Task`/`NonEmpty<Execution>`
/// model—TES supports multiple co-located executions
/// per task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub executions: NonEmpty<JobExecution>,
    pub resources: Option<Resources>,
    pub volumes: Vec<String>,
    /// NOTE: Crankshaft's `Task` has no timeout field.
    /// whether/where timeout enforcement happens upstream before relying on this.
    pub timeout: Option<Duration>,
}

impl Job {
    /// Creates a minimal single-execution Job.
    pub fn new(id: JobId, execution: JobExecution) -> Self {
        Job {
            id,
            name: None,
            description: None,
            executions: NonEmpty::new(execution),
            resources: None,
            volumes: Vec::new(),
            timeout: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_resources(mut self, resources: Resources) -> Self {
        self.resources = Some(resources);
        self
    }

    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn push_execution(mut self, execution: JobExecution) -> Self {
        self.executions.push(execution);
        self
    }

    /// Builds a Job from a Crankshaft Task, given an externally-assigned JobId
    /// (Task itself carries no ID — the caller/runner assigns one).
    pub fn from_task(id: JobId, task: &crankshaft_engine::Task) -> Self {
        let executions: Vec<JobExecution> = task
            .executions()
            .map(|e| JobExecution {
                image: e.image().to_string(),
                program: e.program().to_string(),
                args: e.args().to_vec(),
                work_dir: e.work_dir().map(String::from),
                stdin: e.stdin().map(String::from),
                stdout: e.stdout().map(String::from),
                stderr: e.stderr().map(String::from),
                env: e.env().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            })
            .collect();

        let executions = NonEmpty::from_vec(executions)
            .expect("Task guarantees at least one execution");

        Job {
            id,
            name: task.name().map(String::from),
            description: task.description().map(String::from),
            executions,
            resources: task.resources().map(|r| Resources {
                cpu: r.cpu(),
                cpu_limit: r.cpu_limit(),
                ram: r.ram(),
                ram_limit: r.ram_limit(),
                disk: r.disk(),
                preemptible: r.preemptible(),
                zones: r.zones().to_vec(),
                gpu: r.gpu(),
            }),
            volumes: task.shared_volumes().map(String::from).collect(),
            timeout: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let exec = JobExecution::new("alpine:latest", "echo");
        let job = Job::new(id.clone(), exec);
        assert_eq!(job.id, id);
        assert_eq!(job.executions.head.program, "echo");
        assert!(job.name.is_none());
        assert!(job.resources.is_none());
        assert!(job.timeout.is_none());
    }

    #[test]
    fn test_job_builder_chain() {
        let id = JobId::new("job-002");
        let exec = JobExecution::new("python:3.11", "python3")
            .with_args(vec!["script.py".into()])
            .with_env("ENV", "prod")
            .with_stdin("/tmp/in.txt")
            .with_stdout("/tmp/out.txt")
            .with_stderr("/tmp/err.txt")
            .with_work_dir("/workspace");

        let job = Job::new(id, exec)
            .with_name("my-job")
            .with_timeout(Duration::from_secs(60));

        assert_eq!(job.name.as_deref(), Some("my-job"));
        assert_eq!(job.executions.head.image, "python:3.11");
        assert_eq!(job.executions.head.args, vec!["script.py"]);
        assert_eq!(
            job.executions.head.env.get("ENV").map(|s| s.as_str()),
            Some("prod")
        );
        assert_eq!(job.timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_job_multi_execution() {
        let id = JobId::new("job-multi");
        let first = JobExecution::new("alpine:latest", "setup.sh");
        let second = JobExecution::new("alpine:latest", "main.sh");
        let job = Job::new(id, first).push_execution(second);
        assert_eq!(job.executions.len(), 2);
        assert_eq!(job.executions.last().program, "main.sh");
    }

    #[test]
    fn test_resources_defaults() {
        let r = Resources::default();
        assert!(r.cpu.is_none());
        assert!(r.ram.is_none());
        assert!(r.disk.is_none());
        assert!(r.gpu.is_none());
        assert!(r.preemptible.is_none());
        assert!(r.zones.is_empty());
    }

    #[test]
    fn test_job_with_resources() {
        let id = JobId::new("job-003");
        let res = Resources {
            cpu: Some(4.0),
            cpu_limit: Some(4.0),
            ram: Some(16.0),
            ram_limit: Some(16.0),
            disk: Some(100.0),
            preemptible: Some(false),
            zones: vec!["us-east-1".into()],
            gpu: Some(1),
        };
        let exec = JobExecution::new("cuda:12", "train.sh");
        let job = Job::new(id, exec).with_resources(res);
        let r = job.resources.unwrap();
        assert_eq!(r.ram, Some(16.0));
        assert_eq!(r.zones, vec!["us-east-1".to_string()]);
    }

    #[test]
    fn test_job_serialization() {
        let id = JobId::new("job-ser-01");
        let exec = JobExecution::new("alpine:latest", "echo");
        let job = Job::new(id, exec).with_name("test");
        let json = serde_json::to_string(&job).expect("serialize failed");
        let back: Job = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.name.as_deref(), Some("test"));
        assert_eq!(back.executions.head.program, "echo");
    }
}