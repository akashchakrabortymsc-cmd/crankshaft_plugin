# crankshaft-plugin-core

Shared types and contracts between Crankshaft (the host) and any plugin
process. Neither side does anything useful without agreeing on these first.

---

## What's here

- **`JobId`** — unique identifier for a submitted job.
- **`Job`** — the unit of work sent to a plugin: program, args, environment,
  working directory, resource requirements (`Resources`), and an optional
  timeout.
- **`JobStatus`** — `Pending`, `Running`, `Completed`, `Cancelled`, or
  `Failed(String)`.
- **`PluginError`** / **`PluginResult<T>`** — the error type every fallible
  operation in this system returns: `ConnectionFailed`, `JobNotFound`,
  `InvalidResponse`, `Timeout`, `Unknown`.
- **`PluginHandler`** — the async trait (`#[async_trait]`, since native
  `async fn` in traits isn't `dyn`-compatible) that defines `submit`,
  `status`, `cancel`, and `health_check`.

---

## Test Results

| Module | Test | Result |
|---|---|---|
| `job` | `test_job_id_display` | ✅ pass |
| `job` | `test_job_id_as_str` | ✅ pass |
| `job` | `test_job_id_equality` | ✅ pass |
| `job` | `test_job_new_defaults` | ✅ pass |
| `job` | `test_job_builder_chain` | ✅ pass |
| `job` | `test_job_with_resources` | ✅ pass |
| `job` | `test_resources_defaults` | ✅ pass |
| `job` | `test_job_serialization` | ✅ pass |
| `error` | `test_connection_failed_message` | ✅ pass |
| `error` | `test_job_not_found_message` | ✅ pass |
| `error` | `test_invalid_response_message` | ✅ pass |
| `error` | `test_timeout_message` | ✅ pass |
| `error` | `test_unknown_message` | ✅ pass |
| `error` | `test_plugin_result_ok` | ✅ pass |
| `error` | `test_plugin_result_err` | ✅ pass |
| `status` | `test_pending_display` | ✅ pass |
| `status` | `test_running_display` | ✅ pass |
| `status` | `test_completed_display` | ✅ pass |
| `status` | `test_failed_display` | ✅ pass |
| `status` | `test_cancelled_display` | ✅ pass |
| `status` | `test_is_terminal_completed` | ✅ pass |
| `status` | `test_is_terminal_failed` | ✅ pass |
| `status` | `test_is_terminal_cancelled` | ✅ pass |
| `status` | `test_is_not_terminal_pending` | ✅ pass |
| `status` | `test_is_not_terminal_running` | ✅ pass |
| `status` | `test_equality` | ✅ pass |
| `status` | `test_serialization` | ✅ pass |
| `status` | `test_clone` | ✅ pass |
| `traits` | `test_submit_returns_job_id` | ✅ pass |
| `traits` | `test_status_returns_completed` | ✅ pass |
| `traits` | `test_cancel_returns_ok` | ✅ pass |
| `traits` | `test_health_check_default_ok` | ✅ pass |
| `traits` | `test_submit_connection_failed` | ✅ pass |
| `traits` | `test_status_job_not_found` | ✅ pass |
| `traits` | `test_cancel_unknown_error` | ✅ pass |

---

## Status

**✅ Part 1 complete.**

All core types and the handler trait are implemented, tested, and passing.

### What is done

`job.rs` defines `JobId` (newtype over `String` with `Display`, `Hash`,
`Serialize`, `Deserialize`), `Resources` (CPU, RAM, disk, GPU, preemptible —
units aligned with Crankshaft's own `Resources` struct), and `Job` (the full
unit of work with builder methods for every field including `image`, `stdin`,
`stdout`, `stderr` — aligned with Crankshaft's `Execution` struct).

`status.rs` defines `JobStatus` with all five variants. The `is_terminal()`
helper returns `true` for `Completed`, `Failed(_)`, and `Cancelled` — used
by the host's polling loop to know when to stop.

`error.rs` defines `PluginError` with five typed variants. `PartialEq` is
derived so tests can assert on specific error kinds. `PluginResult<T>` is
a type alias over `Result<T, PluginError>` used throughout the system.

`traits.rs` defines `PluginHandler` — the single async trait every plugin
must implement. `async_trait` is required because native `async fn` in traits
is not yet `dyn`-compatible in stable Rust. `health_check` has a default
implementation returning `Ok(())` so existing plugins do not break when the
host adds health check support.

### What is not yet decided

Whether the host-side contract (used for `Box<dyn ...>` dispatch alongside
Crankshaft's existing Docker and Slurm backends) should be a separate trait
from `PluginHandler`, mirroring the host/SDK split in the rest of the
roadmap. Current leaning: keep one trait, let the SDK wrap it.

### Next

`plugin-host` — TCP listener, JSON-RPC message handling, process spawn
via `tokio::process::Command` with `kill_on_drop`, and the status polling
loop.