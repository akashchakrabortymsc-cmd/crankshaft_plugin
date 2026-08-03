//! The engine-side half of the Crankshaft plugin system.
//!
//! `plugin-host` spawns a plugin as a standalone OS process and talks to it
//! over JSON-RPC/TCP (envelope types live in the sibling `plugin-protocol`
//! crate; the shared `Job`/`JobStatus`/`PluginHandler` vocabulary lives in
//! `plugin-core`). The crate's job is to look, from Crankshaft's point of
//! view, like an ordinary `Backend` — everything else here exists to make
//! that adapter possible:
//!
//! - [`process`] — spawns the plugin binary (`tokio::process::Command`,
//!   `kill_on_drop(true)` so a dropped host never leaves a zombie behind)
//!   and owns the child process handle.
//! - [`rpc`] — the client side of the JSON-RPC/TCP protocol: connects to the
//!   plugin's listener and issues `submit`/`status`/`cancel`/`health_check`
//!   calls using the wire types from `plugin-protocol`.
//! - [`poll`] — the startup retry loop (the plugin needs a moment to bind
//!   its TCP listener after spawn) and the in-flight status-polling loop
//!   used once a job has been submitted.
//! - [`backend`] — the actual adapter: implements Crankshaft's real
//!   `Backend::run(task, token) -> Result<BoxFuture<Result<NonEmpty<ExitStatus>,
//!   TaskRunError>>>`. Internally converts `Task` to a `plugin_core::Job`,
//!   submits it, polls status in a loop racing against
//!   `token.cancelled()`, and on completion builds the `NonEmpty<ExitStatus>`
//!   Crankshaft expects from the plugin's per-execution exit codes. This is
//!   the crux of the crate.
//! - [`config`] — host-side configuration: path to the plugin binary,
//!   startup retry count/delay, poll interval, and similar knobs.
//! - [`error`] — the shared error type for everything above.
//!
//! Every module here is still a stub — no logic has been written yet. This
//! file exists to fix the shape of the crate before filling any of it in.

pub mod backend;
pub mod config;
pub mod error;
pub mod poll;
pub mod process;
pub mod rpc;

pub use config::HostConfig;
pub use error::HostError;
pub use error::HostResult;
pub use process::PluginProcess;
pub use rpc::RpcClient;
