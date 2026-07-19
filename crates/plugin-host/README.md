# crankshaft-plugin-host

The engine-side crate. `plugin-host` spawns an external plugin process,
speaks JSON-RPC over TCP to it, and implements Crankshaft's real `Backend`
trait as an adapter over `crankshaft-plugin-core`'s submit/status/cancel
model. Nothing calls into this crate except Crankshaft itself; nothing in
this crate depends on any particular plugin.

---

## What this crate does

Crankshaft calls `Backend::run(task, cancellation_token)` on whatever backend
is configured. For a plugin-backed task, that call needs to:

1. Convert the `Task` into a `Job` (`crankshaft-plugin-core::Job::from_task`).
2. Get that `Job` to a running plugin process and get a `JobId` back.
3. Poll the plugin for status until it's terminal, honoring the
   `CancellationToken` Crankshaft gave us.
4. Convert the final `JobStatus` back into what `Backend::run` promises to
   return: `NonEmpty<ExitStatus>` on success, `TaskRunError` otherwise.

`plugin-host` is the code that does all four steps, plus the subprocess
management (spawning the plugin, knowing when it's ready, noticing if it
dies) that steps 2–3 depend on.

---

## Module layout

- **`backend.rs`** — `PluginBackend`, the actual `impl Backend for
  PluginBackend`. This is the thing Crankshaft holds a reference to.
- **`process.rs`** — spawns and supervises the plugin subprocess: startup,
  readiness detection, crash detection, shutdown.
- **`rpc/`** — the JSON-RPC-over-TCP layer used to talk to a running plugin:
  - `message.rs` — request/response wire types for `submit` / `status` /
    `cancel` / `health_check`.
  - `transport.rs` — frames those messages over a raw `TcpStream`.
  - `client.rs` — host-side caller: opens/holds the connection, sends a
    request, correlates it with its response, surfaces `PluginError`s. The
    mirror image of plugin-core's `PluginHandler` trait — that's what a
    *plugin* implements in-process; `Client` is what the *host* calls
    against a plugin over the wire.
- **`poll.rs`** — the status-polling loop: calls `Client::status` on an
  interval until `JobStatus::is_terminal()`, or the task's
  `CancellationToken` fires.
- **`config.rs`** — `PluginConfig`: where to find the plugin binary, how to
  launch it, startup/RPC timeouts. Intended to be the eventual target of
  Crankshaft's `config.toml` deserialization (roadmap phase 5), even though
  nothing reads it from a file yet.
- **`error.rs`** — `PluginHostError`, this crate's error type. Wraps
  `crankshaft-plugin-core::PluginError` (errors surfaced *by* the plugin) and
  is meant to map onto `TaskRunError` (what `Backend::run` must return).

---

## Open design questions

These are genuinely undecided, not implementation detail to fill in later —
they shape the module boundaries above:

1. **Transport crate.** Hand-rolled framing over `tokio::net::TcpStream` +
   `serde_json`, vs pulling in `jsonrpsee`. Leaning hand-rolled — the surface
   area is four methods — but not committed.
2. **Framing strategy**, if hand-rolled: newline-delimited JSON vs
   length-prefixed frames.
3. **Readiness handshake.** How does the host know the plugin's TCP listener
   is up before it tries to connect — fixed delay, a ready-signal on stdout,
   or connect-with-backoff up to `startup_timeout`?
4. **Port assignment.** Host-assigned (passed to the plugin via arg/env) vs
   plugin-chosen-and-reported.
5. **Crash detection mid-flight.** Does something watch the child process
   concurrently with RPC calls, so an in-flight `submit`/`status` fails fast
   with a clear error instead of hanging on a dead socket?
6. **`PluginBackend` lifecycle.** One `PluginBackend` = one already-spawned,
   already-connected plugin process (construct once, `run()` many times,
   mirroring how `crankshaft-docker`'s `Backend for Docker` holds one shared
   client) — or spawn/connect per task? Needs a full read of
   `crankshaft-docker`'s actual `Backend` impl to confirm the pattern.
7. **`PluginError` serialization.** It currently derives `PartialEq` but not
   `Serialize`/`Deserialize` — needs one or the other (or a wire-safe mirror
   type) before `Response::Err` can round-trip over JSON-RPC.

These are also the questions worth getting Clay's take on once there's
something concrete to show him, same as the timeout/`.expect()` questions
from plugin-core.

---

## Dependencies of note

- **`crankshaft-plugin-core`** (path dependency) — `Job`, `JobId`,
  `JobStatus`, `PluginError`, `PluginHandler`. This crate is the consumer of
  everything plugin-core defines.
- **`crankshaft-engine`** (git dependency) — the real `Backend` trait and
  `Task`/`TaskRunError` types this crate implements against, not a
  hand-mirrored approximation.
- **`tokio`** — process spawning, TCP, async runtime.
- **JSON-RPC transport dependency** — not yet added; see open question 1.

---

## Status

**🚧 Scaffolded, not implemented.** Module structure and type signatures are
in place (`backend.rs`, `config.rs`, `error.rs`, `process.rs`, `poll.rs`,
`rpc/{mod,message,transport,client}.rs`); function bodies are `todo!()`
pending the design decisions above. Depends on `plugin-core` being finalized
(pending Clay's review — see that crate's README).

### What's next

Resolve the open design questions above, starting with a full read of
`crankshaft-docker`'s `Backend for Docker` implementation (currently we've
only seen the trait signature) — that answers questions 3, 4, and 6 by
example rather than by guessing.