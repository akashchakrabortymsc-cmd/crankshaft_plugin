//! The `submit`/`status`/`cancel`/`health_check` vocabulary of the plugin
//! protocol, built on top of [`super::transport::RpcTransport`].

use plugin_core::Job;
use plugin_core::JobId;
use plugin_core::JobStatus;
use plugin_core::PluginError;
use plugin_protocol::RpcErrorObject;
use plugin_protocol::method;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::net::TcpStream;

use super::transport::RpcTransport;
use crate::error::HostResult;

/// A connection to a running plugin.
///
/// This is the type the rest of `plugin-host` (eventually [`crate::poll`]
/// and [`crate::backend`]) should reach for. It knows the plugin protocol's
/// four methods and how to turn an error response into the right
/// [`PluginError`] variant; everything below the JSON-RPC envelope is
/// [`RpcTransport`]'s problem.
pub struct RpcClient {
    transport: RpcTransport,
}

impl RpcClient {
    /// Wraps an already-connected [`TcpStream`] as an [`RpcClient`].
    pub fn new(stream: TcpStream) -> Self {
        Self {
            transport: RpcTransport::new(stream),
        }
    }

    /// Submits a job to the plugin, returning the [`JobId`] it assigns.
    pub async fn submit(&mut self, job: &Job) -> HostResult<JobId> {
        let params =
            serde_json::to_value(job).map_err(|e| PluginError::InvalidResponse(e.to_string()))?;
        self.call(method::SUBMIT, params, None).await
    }

    /// Queries the status of a previously submitted job.
    pub async fn status(&mut self, id: &JobId) -> HostResult<JobStatus> {
        let params =
            serde_json::to_value(id).map_err(|e| PluginError::InvalidResponse(e.to_string()))?;
        self.call(method::STATUS, params, Some(id)).await
    }

    /// Cancels a running job.
    pub async fn cancel(&mut self, id: &JobId) -> HostResult<()> {
        let params =
            serde_json::to_value(id).map_err(|e| PluginError::InvalidResponse(e.to_string()))?;
        self.call(method::CANCEL, params, Some(id)).await
    }

    /// Checks that the plugin is alive and responsive.
    pub async fn health_check(&mut self) -> HostResult<()> {
        self.call(method::HEALTH_CHECK, Value::Null, None).await
    }

    /// Sends a call through the transport and decodes any error response
    /// into a [`PluginError`].
    ///
    /// `job_id` is only used to enrich a `"job_not_found"` error code into
    /// [`PluginError::JobNotFound`] — the wire error object doesn't carry a
    /// [`JobId`], but the caller already knows which job it asked about.
    async fn call<R: DeserializeOwned>(
        &mut self,
        method: &str,
        params: Value,
        job_id: Option<&JobId>,
    ) -> HostResult<R> {
        match self.transport.call(method, params).await? {
            Ok(result) => Ok(result),
            Err(error) => Err(decode_rpc_error(error, job_id).into()),
        }
    }
}

/// Maps a wire-level [`RpcErrorObject`] back into a [`PluginError`].
///
/// Best-effort: a code the plugin sends that isn't recognized here falls
/// back to [`PluginError::Unknown`] rather than being treated as a
/// transport-level failure.
fn decode_rpc_error(error: RpcErrorObject, job_id: Option<&JobId>) -> PluginError {
    match (error.code.as_str(), job_id) {
        ("connection_failed", _) => PluginError::ConnectionFailed(error.message),
        ("timeout", _) => PluginError::Timeout,
        ("invalid_response", _) => PluginError::InvalidResponse(error.message),
        ("job_not_found", Some(id)) => PluginError::JobNotFound(id.clone()),
        _ => PluginError::Unknown(error.message),
    }
}

#[cfg(test)]
mod tests {
    use futures::SinkExt;
    use futures::StreamExt;
    use plugin_core::JobExecution;
    use plugin_protocol::RpcRequest;
    use plugin_protocol::RpcResponse;
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;
    use tokio_util::codec::Framed;
    use tokio_util::codec::LinesCodec;

    use super::*;
    use crate::error::HostError;

    /// Spins up a one-shot mock plugin: accepts a single connection, reads
    /// one request, runs `handler` on it, and writes back the response.
    async fn spawn_mock_plugin<F>(handler: F) -> (RpcClient, JoinHandle<()>)
    where
        F: FnOnce(RpcRequest) -> RpcResponse + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local_addr");

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut framed = Framed::new(stream, LinesCodec::new());
            let line = framed
                .next()
                .await
                .expect("connection closed before a request arrived")
                .expect("codec error");
            let request: RpcRequest = serde_json::from_str(&line).expect("bad request json");
            let response = handler(request);
            let encoded = serde_json::to_string(&response).expect("encode response");
            framed.send(encoded).await.expect("send response");
        });

        let stream = TcpStream::connect(addr).await.expect("connect");
        (RpcClient::new(stream), server)
    }

    #[tokio::test]
    async fn test_submit_success() {
        let job = Job::new(JobId::new("unused"), JobExecution::new("ubuntu:latest", "/bin/true"));

        let (mut client, server) = spawn_mock_plugin(|req| {
            assert_eq!(req.method, method::SUBMIT);
            RpcResponse::ok(req.id, serde_json::to_value(JobId::new("job-1")).unwrap())
        })
        .await;

        let id = client.submit(&job).await.expect("submit should succeed");
        assert_eq!(id, JobId::new("job-1"));
        server.await.expect("mock plugin task panicked");
    }

    #[tokio::test]
    async fn test_status_job_not_found() {
        let (mut client, server) = spawn_mock_plugin(|req| {
            RpcResponse::err(req.id, "job_not_found", "job job-404 not found")
        })
        .await;

        let err = client
            .status(&JobId::new("job-404"))
            .await
            .expect_err("status should fail");
        assert!(matches!(
            err,
            HostError::Plugin(PluginError::JobNotFound(id)) if id == JobId::new("job-404")
        ));
        server.await.expect("mock plugin task panicked");
    }

    #[tokio::test]
    async fn test_health_check_success() {
        let (mut client, server) =
            spawn_mock_plugin(|req| RpcResponse::ok(req.id, Value::Null)).await;

        client.health_check().await.expect("health_check should succeed");
        server.await.expect("mock plugin task panicked");
    }

    #[tokio::test]
    async fn test_cancel_unknown_error_code_falls_back_to_unknown() {
        let (mut client, server) = spawn_mock_plugin(|req| {
            RpcResponse::err(req.id, "something_weird", "the plugin exploded")
        })
        .await;

        let err = client
            .cancel(&JobId::new("job-1"))
            .await
            .expect_err("cancel should fail");
        assert!(matches!(err, HostError::Plugin(PluginError::Unknown(_))));
        server.await.expect("mock plugin task panicked");
    }
}