//! The framed TCP transport underneath the JSON-RPC client.
//!
//! Owns request/response id correlation, sending, and receiving. Knows
//! nothing about what a "job" is, or what `submit`/`status`/`cancel` mean —
//! that vocabulary lives one layer up, in [`super::client`].

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use futures::SinkExt;
use futures::StreamExt;
use plugin_core::PluginError;
use plugin_protocol::RequestId;
use plugin_protocol::RpcErrorObject;
use plugin_protocol::RpcRequest;
use plugin_protocol::RpcResponse;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio_util::codec::LinesCodec;

use crate::error::HostResult;

/// The wire-level half of talking to a plugin: newline-delimited JSON-RPC
/// over TCP, with request/response id correlation.
pub struct RpcTransport {
    framed: Framed<TcpStream, LinesCodec>,
    next_id: AtomicU64,
}

impl RpcTransport {
    /// Wraps an already-connected [`TcpStream`].
    ///
    /// Establishing the connection — including the startup retry loop — is
    /// [`crate::poll`]'s job, not this type's.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            framed: Framed::new(stream, LinesCodec::new()),
            next_id: AtomicU64::new(1),
        }
    }

    fn next_request_id(&self) -> RequestId {
        RequestId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Sends `method` with `params` and returns the decoded result.
    ///
    /// A well-formed error response comes back as `Ok(Err(RpcErrorObject))`
    /// rather than a [`PluginError`] — turning a raw error code into the
    /// right [`PluginError`] variant needs domain knowledge (e.g. which
    /// [`plugin_core::JobId`] a `"job_not_found"` refers to) that this
    /// transport-level type doesn't have. Only a genuine transport failure
    /// (can't connect, malformed envelope, id mismatch) is an `Err` here.
    pub async fn call<R: DeserializeOwned>(
        &mut self,
        method: &str,
        params: Value,
    ) -> HostResult<Result<R, RpcErrorObject>> {
        let id = self.next_request_id();
        let request = RpcRequest::new(id, method, params);

        let encoded = serde_json::to_string(&request)
            .map_err(|e| PluginError::InvalidResponse(e.to_string()))?;

        self.framed
            .send(encoded)
            .await
            .map_err(|e| PluginError::ConnectionFailed(e.to_string()))?;

        let line = self
            .framed
            .next()
            .await
            .ok_or_else(|| PluginError::ConnectionFailed("connection closed by plugin".into()))?
            .map_err(|e| PluginError::ConnectionFailed(e.to_string()))?;

        let response: RpcResponse = serde_json::from_str(&line)
            .map_err(|e| PluginError::InvalidResponse(e.to_string()))?;

        if response.id != id {
            return Err(PluginError::InvalidResponse(format!(
                "response id {:?} did not match request id {:?}",
                response.id, id
            ))
            .into());
        }

        match response.outcome {
            plugin_protocol::RpcOutcome::Ok { result } => {
                let decoded = serde_json::from_value(result)
                    .map_err(|e| PluginError::InvalidResponse(e.to_string()))?;
                Ok(Ok(decoded))
            }
            plugin_protocol::RpcOutcome::Error { error } => Ok(Err(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use plugin_protocol::RpcResponse;
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    use super::*;
    use crate::error::HostError;

    /// Spins up a one-shot mock plugin: accepts a single connection, reads
    /// one request, runs `handler` on it, and writes back the response.
    async fn spawn_mock_plugin<F>(handler: F) -> (RpcTransport, JoinHandle<()>)
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
        (RpcTransport::new(stream), server)
    }

    #[tokio::test]
    async fn test_call_success() {
        let (mut transport, server) =
            spawn_mock_plugin(|req| RpcResponse::ok(req.id, Value::from("pong"))).await;

        let result: Result<String, RpcErrorObject> =
            transport.call("ping", Value::Null).await.expect("call should succeed");
        assert_eq!(result, Ok("pong".to_string()));
        server.await.expect("mock plugin task panicked");
    }

    #[tokio::test]
    async fn test_call_returns_raw_error_object() {
        let (mut transport, server) = spawn_mock_plugin(|req| {
            RpcResponse::err(req.id, "job_not_found", "job job-404 not found")
        })
        .await;

        let result: Result<Value, RpcErrorObject> =
            transport.call("status", Value::Null).await.expect("call should succeed");
        let error = result.expect_err("plugin reported an error");
        assert_eq!(error.code, "job_not_found");
        server.await.expect("mock plugin task panicked");
    }

    #[tokio::test]
    async fn test_response_id_mismatch_is_invalid_response() {
        let (mut transport, server) =
            spawn_mock_plugin(|_req| RpcResponse::ok(RequestId(999), Value::Null)).await;

        let err = transport
            .call::<Value>("ping", Value::Null)
            .await
            .expect_err("mismatched response id should fail");
        assert!(matches!(err, HostError::Plugin(PluginError::InvalidResponse(_))));
        server.await.expect("mock plugin task panicked");
    }

    #[tokio::test]
    async fn test_malformed_response_is_invalid_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local_addr");

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut framed = Framed::new(stream, LinesCodec::new());
            // Consume the request, then send back a line that doesn't even
            // parse as an `RpcResponse` (missing the `"status"` tag that
            // distinguishes `RpcOutcome::Ok`/`RpcOutcome::Error`) — this is
            // now the only way to trigger a malformed response, since
            // `RpcResponse` itself can't represent "neither result nor
            // error" anymore.
            framed.next().await.expect("request").expect("codec error");
            framed
                .send(r#"{"id":1}"#.to_string())
                .await
                .expect("send malformed response");
        });

        let stream = TcpStream::connect(addr).await.expect("connect");
        let mut transport = RpcTransport::new(stream);

        let err = transport
            .call::<Value>("ping", Value::Null)
            .await
            .expect_err("malformed response should fail");
        assert!(matches!(err, HostError::Plugin(PluginError::InvalidResponse(_))));
        server.await.expect("mock plugin task panicked");
    }

    #[tokio::test]
    async fn test_connection_closed_before_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local_addr");

        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.expect("accept");
            // Drop the connection immediately without responding.
        });

        let stream = TcpStream::connect(addr).await.expect("connect");
        let mut transport = RpcTransport::new(stream);

        let err = transport
            .call::<Value>("ping", Value::Null)
            .await
            .expect_err("closed connection should fail");
        assert!(matches!(err, HostError::Plugin(PluginError::ConnectionFailed(_))));
        server.await.expect("mock plugin task panicked");
    }
}