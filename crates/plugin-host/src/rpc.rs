//! The client side of the JSON-RPC/TCP protocol: connects to a running
//! plugin and issues `submit`/`status`/`cancel`/`health_check` calls.
//!
//! Split into two layers:
//!
//! - [`transport`] — the wire mechanics: framing, request/response id
//!   correlation, decoding the JSON-RPC envelope. Knows nothing about
//!   jobs.
//! - [`client`] — the plugin-protocol vocabulary
//!   (`submit`/`status`/`cancel`/`health_check`) and error mapping, built
//!   on top of [`transport::RpcTransport`].

pub mod client;
pub mod transport;

pub use client::RpcClient;
pub use transport::RpcTransport;