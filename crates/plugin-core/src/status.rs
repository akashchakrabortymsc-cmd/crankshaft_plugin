use serde::{Deserialize, Serialize};

/// The current state of a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    /// One exit code per execution, in the same order as Job.executions.
    Completed(Vec<i32>),
    Failed(String),
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Completed(codes) => write!(f, "Completed: {:?}", codes),
            JobStatus::Failed(msg) => write!(f, "Failed: {}", msg),
            JobStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl JobStatus {
    /// Returns true if the job has finished (success or failure).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed(_) | JobStatus::Failed(_) | JobStatus::Cancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_display() {
        assert_eq!(JobStatus::Pending.to_string(), "Pending");
    }

    #[test]
    fn test_running_display() {
        assert_eq!(JobStatus::Running.to_string(), "Running");
    }

    #[test]
    fn test_completed_display() {
        let s = JobStatus::Completed(vec![0]);
        assert_eq!(s.to_string(), "Completed: [0]");
    }

    #[test]
    fn test_completed_multi_execution_display() {
        let s = JobStatus::Completed(vec![0, 0, 1]);
        assert_eq!(s.to_string(), "Completed: [0, 0, 1]");
    }

    #[test]
    fn test_failed_display() {
        let s = JobStatus::Failed("out of memory".into());
        assert_eq!(s.to_string(), "Failed: out of memory");
    }

    #[test]
    fn test_cancelled_display() {
        assert_eq!(JobStatus::Cancelled.to_string(), "Cancelled");
    }

    #[test]
    fn test_is_terminal_completed() {
        assert!(JobStatus::Completed(vec![0]).is_terminal());
    }

    #[test]
    fn test_is_terminal_failed() {
        assert!(JobStatus::Failed("err".into()).is_terminal());
    }

    #[test]
    fn test_is_terminal_cancelled() {
        assert!(JobStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_is_not_terminal_pending() {
        assert!(!JobStatus::Pending.is_terminal());
    }

    #[test]
    fn test_is_not_terminal_running() {
        assert!(!JobStatus::Running.is_terminal());
    }

    #[test]
    fn test_equality() {
        assert_eq!(JobStatus::Pending, JobStatus::Pending);
        assert_eq!(
            JobStatus::Completed(vec![0, 0]),
            JobStatus::Completed(vec![0, 0])
        );
        assert_ne!(
            JobStatus::Completed(vec![0]),
            JobStatus::Completed(vec![1])
        );
        assert_eq!(JobStatus::Failed("x".into()), JobStatus::Failed("x".into()));
        assert_ne!(JobStatus::Failed("x".into()), JobStatus::Failed("y".into()));
    }

    #[test]
    fn test_serialization() {
        let status = JobStatus::Failed("disk full".into());
        let json = serde_json::to_string(&status).expect("serialize failed");
        let back: JobStatus = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(status, back);
    }

    #[test]
    fn test_completed_serialization() {
        let status = JobStatus::Completed(vec![0, 1, 0]);
        let json = serde_json::to_string(&status).expect("serialize failed");
        let back: JobStatus = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(status, back);
    }

    #[test]
    fn test_clone() {
        let a = JobStatus::Running;
        let b = a.clone();
        assert_eq!(a, b);
    }
}