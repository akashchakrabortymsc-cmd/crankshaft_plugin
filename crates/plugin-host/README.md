# crankshaft-plugin-host

The engine-side half of the Crankshaft plugin system. `plugin-host` spawns a
plugin as a standalone OS process and talks to it over JSON-RPC/TCP. From
Crankshaft's point of view, it should eventually look like an ordinary
`Backend` — everything in this crate exists to make that adapter possible.

Shared vocabulary (`Job`, `JobStatus`, `PluginHandler`, `PluginError`) lives
in the sibling `plugin-core` crate. The JSON-RPC envelope (`RpcRequest`,
`RpcResponse`) lives in `plugin-protocol`. Neither of those crates knows
anything about processes, TCP, or Crankshaft's real `Backend` trait — that's
this crate's job.

---

## What's here

- **`HostConfig`** (`config.rs`) — path to the plugin binary, spawn
  arguments, connection retry count/delay, poll interval, RPC timeout.
  Builder-style, with defaults matching the original design plan (5 connect
  attempts, 500ms apart).
- **`HostError`** / **`HostResult<T>`** (`error.rs`) — the host's error
  type. Deliberately thin: `Spawn` (subprocess spawn failure), `Process`
  (I/O error managing an already-spawned child), `ProcessExited`, and
  `Plugin(PluginError)` — everything that's really a `plugin-core::PluginError`
  (connection failures, timeouts, malformed responses, unknown jobs) is
  passed through rather than re-invented here.
- **`PluginProcess`** (`process.rs`) — wraps `tokio::process::Child` with
  `kill_on_drop(true)` set unconditionally, so a dropped handle never
  leaves a zombie plugin process behind. Owns only the process lifecycle
  (`spawn`/`try_wait`/`wait`/`kill`) — no networking, no RPC.
- **`rpc`** — the JSON-RPC/TCP client, split into two layers:
  - **`rpc::transport::RpcTransport`** — wire mechanics only: newline-delimited
    JSON framing (`tokio_util::codec::LinesCodec`), request/response id
    correlation, envelope decoding. Knows nothing about jobs; a well-formed
    error response comes back as a raw `RpcErrorObject`, not a `PluginError`.
  - **`rpc::client::RpcClient`** — the plugin-protocol vocabulary
    (`submit`/`status`/`cancel`/`health_check`), built on `RpcTransport`.
    Maps error codes to `PluginError` variants, using the caller's own
    `JobId` to build a proper `PluginError::JobNotFound` (the wire error
    object doesn't carry one).
- **`poll`** (`poll.rs`) — two independent loops:
  - `connect_with_retry` — retries connecting to the plugin's TCP listener
    after spawn, since it needs a moment to bind.
  - `poll_until_terminal` — repeatedly checks a job's status until it's
    done. Deliberately has no cancellation logic of its own; a caller that
    needs to race this against a `CancellationToken` wraps the call in
    `tokio::select!` instead.
- **`backend`** — not yet implemented. Will be the actual adapter:
  `impl Backend for PluginBackend`, converting `Task` → `Job`, submitting,
  polling, and building `NonEmpty<ExitStatus>` on completion. See
  [What's next](#whats-next).

---

## Design decisions worth knowing about

- **Framing: newline-delimited JSON, not length-prefixed.** Simpler, and
  JSON strings escape embedded newlines, so it's safe. Can be swapped for
  `tokio_util::codec::LengthDelimitedCodec` later without touching anything
  above the transport layer.
- **`RpcResponse` uses a tagged `RpcOutcome` enum, not two `Option` fields.**
  The original design (`result: Option<Value>`, `error: Option<Value>`) had
  a real bug: a successful response whose result was JSON `null` (e.g.
  `health_check`) round-tripped back indistinguishable from "no result set
  at all," because serde's `Option<T>` deserializer treats a literal `null`
  as `None` regardless of `T`. `plugin-protocol::RpcOutcome` tags the
  variant explicitly on the wire (`"status": "ok" | "error"`), which also
  makes "both set" / "neither set" unrepresentable instead of something
  every caller has to check for.
- **Host connects out to the plugin, not the other way around.** The
  original planning notes are inconsistent on this point — one bullet says
  the host listens and the plugin connects in, another describes a
  host-side connection *retry* loop, which only makes sense if the plugin
  is the one listening. This crate is built on the second reading, since
  `HostConfig`'s retry/delay fields and `poll::connect_with_retry` only make
  sense that way. **Still open:** how the host learns which port the
  plugin bound to (fixed/pre-agreed port vs. the plugin reporting it back
  somehow) hasn't been decided — needs an answer before `process.rs` and
  `poll.rs` can be wired together end-to-end.

---

## Test Results

| Module | Test | Result |
|---|---|---|
| `config` | `test_new_defaults` | ✅ pass |
| `config` | `test_builder_chain` | ✅ pass |
| `error` | `test_spawn_message` | ✅ pass |
| `error` | `test_process_exited_with_code` | ✅ pass |
| `error` | `test_process_exited_no_code` | ✅ pass |
| `error` | `test_process_io_error_passthrough` | ✅ pass |
| `error` | `test_plugin_error_passthrough` | ✅ pass |
| `process` | `test_spawn_and_wait_success` | ✅ pass |
| `process` | `test_try_wait_before_exit` | ✅ pass |
| `process` | `test_kill` | ✅ pass |
| `process` | `test_spawn_missing_binary_returns_error` | ✅ pass |
| `rpc::transport` | `test_call_success` | ✅ pass |
| `rpc::transport` | `test_call_returns_raw_error_object` | ✅ pass |
| `rpc::transport` | `test_response_id_mismatch_is_invalid_response` | ✅ pass |
| `rpc::transport` | `test_malformed_response_is_invalid_response` | ✅ pass |
| `rpc::transport` | `test_connection_closed_before_response` | ✅ pass |
| `rpc::client` | `test_submit_success` | ✅ pass |
| `rpc::client` | `test_status_job_not_found` | ✅ pass |
| `rpc::client` | `test_health_check_success` | ✅ pass |
| `rpc::client` | `test_cancel_unknown_error_code_falls_back_to_unknown` | ✅ pass |
| `poll` | `test_connect_with_retry_succeeds_immediately` | ✅ pass |
| `poll` | `test_connect_with_retry_succeeds_after_delay` | ✅ pass |
| `poll` | `test_connect_with_retry_gives_up` | ✅ pass |
| `poll` | `test_poll_until_terminal_returns_first_terminal_status` | ✅ pass |

24/24 passing, zero warnings.

---

## Status

**🚧 In progress.** `config`, `error`, `process`, `rpc` (transport + client),
and `poll` are implemented and tested. `backend` — the actual `Backend`
trait adapter, and the crux of this crate — has not been started.

### What's done

Everything below `Backend` is in place: spawning the plugin subprocess with
`kill_on_drop`, connecting to it with retries, speaking JSON-RPC over a
newline-delimited TCP transport, mapping wire errors back to `PluginError`,
and polling a job's status to completion.

