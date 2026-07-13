use crate::job::JobId;

/// All the ways a plugin interaction can fail.
#[derive(Debug, thiserror::Error, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::JobId;

    #[test]
    fn test_connection_failed_message() {
        let err = PluginError::ConnectionFailed("refused".into());
        assert_eq!(err.to_string(), "connection failed: refused");
    }

    #[test]
    fn test_job_not_found_message() {
        let id = JobId::new("job-404");
        let err = PluginError::JobNotFound(id);
        assert_eq!(err.to_string(), "job not found: job-404");
    }

    #[test]
    fn test_invalid_response_message() {
        let err = PluginError::InvalidResponse("bad json".into());
        assert_eq!(err.to_string(), "invalid response: bad json");
    }

    #[test]
    fn test_timeout_message() {
        let err = PluginError::Timeout;
        assert_eq!(err.to_string(), "timeout");
    }

    #[test]
    fn test_unknown_message() {
        let err = PluginError::Unknown("something broke".into());
        assert_eq!(err.to_string(), "unknown error: something broke");
    }

    #[test]
    fn test_plugin_result_ok() {
        let result: PluginResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_plugin_result_err() {
        let result: PluginResult<i32> = Err(PluginError::Timeout);
        assert!(result.is_err());
    }
}
