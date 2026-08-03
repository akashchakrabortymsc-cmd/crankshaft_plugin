//! Wire types for the plugin/host JSON-RPC protocol.
//!
//! Both `plugin-host` (the client side, talking to a spawned plugin) and
//! `plugin-sdk` (the plugin side, receiving calls) depend on this crate so
//! they agree on exactly one envelope shape. Nothing in here is
//! transport-specific — how the bytes get framed over the TCP connection
//! (newline-delimited, length-prefixed, whatever) is a `plugin-host`/
//! `plugin-sdk` concern, not this crate's.

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// The known RPC methods in the plugin protocol.
///
/// Plain string constants rather than an enum: the method name still
/// travels as a JSON string on the wire either way, and callers on both
/// sides need to match it against a `&str` (e.g. in a `match` on an
/// incoming [`RpcRequest::method`]), so a constant is exactly as safe as an
/// enum here without adding a serialization layer to unwrap.
pub mod method {
    /// Submit a new job to the plugin.
    pub const SUBMIT: &str = "submit";
    /// Query the status of a previously submitted job.
    pub const STATUS: &str = "status";
    /// Cancel a running job.
    pub const CANCEL: &str = "cancel";
    /// Check that the plugin is alive and responsive.
    pub const HEALTH_CHECK: &str = "health_check";
}

/// Correlates an [`RpcRequest`] with its [`RpcResponse`].
///
/// Requests and responses share a single TCP connection, so without an id
/// there'd be no way to tell which response answers which request once
/// more than one call is in flight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

/// A JSON-RPC request sent between host and plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Correlates this request with its response.
    pub id: RequestId,
    /// The RPC method being called (see [`mod@method`]).
    pub method: String,
    /// The method's parameters, encoded as JSON.
    pub params: Value,
}

impl RpcRequest {
    /// Creates a new [`RpcRequest`].
    pub fn new(id: RequestId, method: impl Into<String>, params: Value) -> Self {
        Self {
            id,
            method: method.into(),
            params,
        }
    }
}

/// A structured error inside an [`RpcResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcErrorObject {
    /// A short, machine-readable error code.
    pub code: String,
    /// A human-readable error message.
    pub message: String,
}

/// The outcome of an RPC call: success with a result, or failure with an
/// error.
///
/// Tagged explicitly on the wire with a `"status"` field, rather than
/// inferred from which of two `Option` fields happens to be set. A bare
/// `Option<Value>` can't survive a JSON round trip and still tell "the call
/// succeeded and returned `null`" apart from "no result was set" — serde's
/// `Option<T>` deserializer treats a literal JSON `null` as `None`
/// regardless of what `T` is. Tagging the variant explicitly sidesteps that
/// entirely, and as a bonus makes "both set" / "neither set" unrepresentable
/// instead of something callers have to check for.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RpcOutcome {
    /// The call succeeded.
    Ok {
        /// The method's return value. `Value::Null` for methods that
        /// return nothing on success (e.g. `health_check`, `cancel`).
        result: Value,
    },
    /// The call failed.
    Error {
        /// The structured error.
        error: RpcErrorObject,
    },
}

/// A JSON-RPC response sent between host and plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Matches the [`RequestId`] of the request this responds to.
    pub id: RequestId,
    /// Whether the call succeeded or failed, and its payload.
    #[serde(flatten)]
    pub outcome: RpcOutcome,
}

impl RpcResponse {
    /// Builds a successful response.
    pub fn ok(id: RequestId, result: Value) -> Self {
        Self {
            id,
            outcome: RpcOutcome::Ok { result },
        }
    }

    /// Builds a failed response.
    pub fn err(id: RequestId, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id,
            outcome: RpcOutcome::Error {
                error: RpcErrorObject {
                    code: code.into(),
                    message: message.into(),
                },
            },
        }
    }

    /// Whether this response represents success.
    pub fn is_ok(&self) -> bool {
        matches!(self.outcome, RpcOutcome::Ok { .. })
    }

    /// Whether this response represents failure.
    pub fn is_err(&self) -> bool {
        matches!(self.outcome, RpcOutcome::Error { .. })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_request_round_trip() {
        let request = RpcRequest::new(RequestId(1), method::SUBMIT, json!({"job": "payload"}));
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: RpcRequest = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.id, RequestId(1));
        assert_eq!(decoded.method, method::SUBMIT);
        assert_eq!(decoded.params, json!({"job": "payload"}));
    }

    #[test]
    fn test_response_ok_round_trip() {
        let response = RpcResponse::ok(RequestId(2), json!({"job_id": "abc-123"}));
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: RpcResponse = serde_json::from_str(&encoded).unwrap();

        assert!(decoded.is_ok());
        assert!(!decoded.is_err());
        assert!(matches!(
            decoded.outcome,
            RpcOutcome::Ok { result } if result == json!({"job_id": "abc-123"})
        ));
    }

    /// Regression test: this is exactly the case that broke under the old
    /// `Option<Value>` representation — a successful call whose result is
    /// JSON `null` (e.g. `health_check`) used to round-trip back as
    /// `is_ok() == false`.
    #[test]
    fn test_response_ok_with_null_result_round_trip() {
        let response = RpcResponse::ok(RequestId(3), Value::Null);
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: RpcResponse = serde_json::from_str(&encoded).unwrap();

        assert!(decoded.is_ok());
        assert!(!decoded.is_err());
        assert!(matches!(decoded.outcome, RpcOutcome::Ok { result } if result == Value::Null));
    }

    #[test]
    fn test_response_err_round_trip() {
        let response = RpcResponse::err(RequestId(4), "job_not_found", "job job-404 not found");
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: RpcResponse = serde_json::from_str(&encoded).unwrap();

        assert!(decoded.is_err());
        assert!(!decoded.is_ok());
        match decoded.outcome {
            RpcOutcome::Error { error } => {
                assert_eq!(error.code, "job_not_found");
                assert_eq!(error.message, "job job-404 not found");
            }
            RpcOutcome::Ok { .. } => panic!("expected an error outcome"),
        }
    }
}