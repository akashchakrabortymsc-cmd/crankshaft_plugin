# crankshaft-plugin-core

Shared types and contracts between Crankshaft (the host) and any plugin
process. Neither side does anything useful without agreeing on these first.

---

## What's here

- **`JobId`** — unique identifier for a submitted job. Newtype over a private
  `String`; use `JobId::new()` to construct and `.as_str()` to read, rather
  than reaching into the field directly.
- **`JobExecution`** — one executable step: container `image`, `program`,
  `args`, `work_dir`, `stdin`/`stdout`/`stderr` redirect paths, and `env`.
  Mirrors Crankshaft's own `Execution` type field-for-field.
- **`Job`** — the unit of work sent to a plugin. Holds `id`, optional `name`
  and `description`, one-or-more `executions: NonEmpty<JobExecution>`,
  optional `resources`, `volumes` shared across executions, and an optional
  `timeout` (Crankshaft's `Task` has no timeout field of its own — see note
  below).
- **`Resources`** — `cpu`, `cpu_limit`, `ram`/`ram_limit` (GiB, `f64`),
  `disk` (GiB), `preemptible`, `zones`, `gpu`. Field names and units are
  aligned exactly with Crankshaft's `Resources` struct — in particular, RAM
  and disk are GiB, not MB, to avoid a silent unit-conversion bug.
- **`JobStatus`** — `Pending`, `Running`, `Completed(Vec<i32>)` (one exit
  code per execution, same order as `Job.executions`), `Cancelled`, or
  `Failed(String)`.
- **`PluginError`** / **`PluginResult<T>`** — the error type every fallible
  operation in this system returns: `ConnectionFailed`, `JobNotFound`,
  `InvalidResponse`, `Timeout`, `Unknown`.
- **`PluginHandler`** — the async trait (`#[async_trait]`, since native
  `async fn` in traits isn't `dyn`-compatible yet) that defines `submit`,
  `status`, `cancel`, and `health_check`.

---

## Why `Job` supports multiple executions

Crankshaft's `Task` type holds `NonEmpty<Execution>`, not a single execution
— per Clay McLeod: TES (the Task Execution Schema Crankshaft implements)
supports multiple co-located executions running on the same task/machine.
`Job` mirrors this shape (`executions: NonEmpty<JobExecution>`) rather than
assuming one command per job, so a `Job` built from a real Crankshaft `Task`
never loses information.

`Job::from_task(id, &Task)` builds a `Job` from a real
`crankshaft_engine::Task`, given an externally-assigned `JobId` (`Task`
itself carries no ID — that's assigned by the runner/caller).

**Known gap:** `Task` has no `timeout` field anywhere upstream. `Job.timeout`
exists on our side for the plugin/host protocol but has no source to convert
*from* yet — worth confirming with Clay where (or whether) timeout is
enforced in Crankshaft today before relying on it.

---

## Dependencies of note

- **`crankshaft-engine`** (git dependency) — `Job::from_task` builds directly
  against Crankshaft's real `Task`/`Execution`/`Resources` types, not a
  hand-mirrored approximation.
- **`nonempty`** (with the `serialize` feature enabled) — backs
  `Job.executions`. The `serialize` feature is required explicitly; it is
  not on by default.

---

## Test Results

| Module | Test | Result |
|---|---|---|
| `job` | `test_job_id_display` | ✅ pass |
| `job` | `test_job_id_as_str` | ✅ pass |
| `job` | `test_job_id_equality` | ✅ pass |
| `job` | `test_job_new_defaults` | ✅ pass |
| `job` | `test_job_builder_chain` | ✅ pass |
| `job` | `test_job_multi_execution` | ✅ pass |
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
| `status` | `test_completed_multi_execution_display` | ✅ pass |
| `status` | `test_failed_display` | ✅ pass |
| `status` | `test_cancelled_display` | ✅ pass |
| `status` | `test_is_terminal_completed` | ✅ pass |
| `status` | `test_is_terminal_failed` | ✅ pass |
| `status` | `test_is_terminal_cancelled` | ✅ pass |
| `status` | `test_is_not_terminal_pending` | ✅ pass |
| `status` | `test_is_not_terminal_running` | ✅ pass |
| `status` | `test_equality` | ✅ pass |
| `status` | `test_serialization` | ✅ pass |
| `status` | `test_completed_serialization` | ✅ pass |
| `status` | `test_clone` | ✅ pass |
| `traits` | `test_submit_returns_job_id` | ✅ pass |
| `traits` | `test_status_returns_completed` | ✅ pass |
| `traits` | `test_cancel_returns_ok` | ✅ pass |
| `traits` | `test_health_check_default_ok` | ✅ pass |
| `traits` | `test_submit_connection_failed` | ✅ pass |
| `traits` | `test_status_job_not_found` | ✅ pass |
| `traits` | `test_cancel_unknown_error` | ✅ pass |

**38 / 38 passing.**

---

## Status

**✅ Part 1 complete — including post-review alignment with `crankshaft-engine`.**

All core types and the handler trait are implemented, tested, and passing.
This crate went through one substantive revision after initial review: Clay
recommended importing `crankshaft-engine`'s real types (or bridging to them)
rather than maintaining a parallel, hand-guessed set of types — see "Why
`Job` supports multiple executions" above for what changed as a result.

### What is done

`job.rs` defines `JobId` (newtype over a private `String`, with `Display`,
`Hash`, `Serialize`, `Deserialize`, and `as_str()`), `JobExecution` (mirrors
Crankshaft's `Execution`: image, program, args, work_dir, stdin/stdout/stderr,
env), `Resources` (field-for-field aligned with Crankshaft's own `Resources`,
including matching GiB units), and `Job` (id, name, description, one-or-more
`executions`, resources, volumes, timeout — plus `Job::from_task` for
building a `Job` from a real `crankshaft_engine::Task`).

`status.rs` defines `JobStatus` with all five variants, where `Completed`
now carries `Vec<i32>` (one exit code per execution) rather than being a
bare unit variant, so a multi-execution job's per-step outcomes aren't lost.
The `is_terminal()` helper returns `true` for `Completed(_)`, `Failed(_)`,
and `Cancelled` — used by the host's polling loop to know when to stop.

`error.rs` defines `PluginError` with five typed variants. `PartialEq` is
derived so tests can assert on specific error kinds. `PluginResult<T>` is
a type alias over `Result<T, PluginError>` used throughout the system.

`traits.rs` defines `PluginHandler` — the single async trait every plugin
must implement. `async_trait` is required because native `async fn` in traits
is not yet `dyn`-compatible in stable Rust. `health_check` has a default
implementation returning `Ok(())` so existing plugins do not break when the
host adds health check support.

### What's next

`plugin-host` — the engine-side crate that spawns a plugin subprocess,
speaks JSON-RPC over TCP to it, and implements Crankshaft's real `Backend`
trait (`run(task, cancellation_token) -> Future<Result<NonEmpty<ExitStatus>>>`)
as an adapter over this crate's submit/poll/cancel model.
