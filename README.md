# crankshaft-plugin

A production-oriented plugin architecture for [Crankshaft](https://github.com/stjude-rust-labs/crankshaft) that enables external execution backends to be developed, distributed, and maintained independently from the core engine.

> **Status:** 🚧 Early Development — Part 1 (`plugin-core`) in progress.

---

## The Problem

Crankshaft currently supports a fixed set of execution backends: Docker, Slurm, LSF, and TES.

Every organisation that needs a custom backend — a private Kubernetes cluster, a proprietary HPC scheduler, a cloud-burst environment — must either fork Crankshaft or wait for an upstream release.

This project removes that constraint.

---

## The Solution

A subprocess-based plugin system where:

- Crankshaft spawns an external plugin binary at runtime
- Host and plugin communicate via JSON-RPC over TCP
- Plugin authors implement a single Rust trait
- A plugin crash **never** crashes the engine

```
                        USER
                          │
                          ▼
                     config.toml
                          │
                          ▼
                     CRANKSHAFT
                  Workflow Engine
                          │
                          ▼
                   Backend Factory
                          │
         ┌────────────────┼────────────────┐
         │                │                │
         ▼                ▼                ▼
   DockerBackend    SlurmBackend    PluginBackend
                                          │
                                          ▼
                                    Plugin Host
                                          │
                               JSON-RPC over TCP
                                          │
               ┌──────────────────────────┴──────────────────────┐
               │                                                  │
               ▼                                                  ▼
       Kubernetes Plugin                               Custom HPC Plugin
               │                                                  │
               ▼                                                  ▼
       Real Compute Resources                          Real Compute Resources
```

---

## Workspace Structure

```
crankshaft-plugin/
│
├── crates/
│   ├── plugin-core        # Shared types, traits, and errors
│   ├── plugin-host        # Engine-side: spawns plugins, manages RPC
│   ├── plugin-sdk         # Plugin-author toolkit
│   └── plugin-example     # Reference implementation
│
├── Cargo.toml             # Workspace root
└── README.md
```

---

## Crates

### `plugin-core`

Shared contracts used by both the engine and plugin authors.

Provides:
- `JobId` — unique job identifier (newtype over `String`)
- `Job` — unit of work sent to a plugin
- `JobStatus` — `Pending | Running | Completed | Failed(String) | Cancelled`
- `Resources` — CPU, memory, GPU requirements
- `PluginHandler` — the async trait every plugin must implement
- `PluginError` — typed error enum
- `PluginResult<T>` — type alias for `Result<T, PluginError>`

```rust
#[async_trait]
pub trait PluginHandler: Send + Sync + 'static {
    async fn submit(&self, job: Job) -> PluginResult<JobId>;
    async fn status(&self, id: JobId) -> PluginResult<JobStatus>;
    async fn cancel(&self, id: JobId) -> PluginResult<()>;
    async fn health_check(&self) -> PluginResult<()> { Ok(()) }
}
```

---

### `plugin-host`

Runs inside the Crankshaft engine process.

Responsibilities:
- Spawn the plugin binary as a child process (`tokio::process::Command`)
- Manage plugin lifecycle (`kill_on_drop`, restart on crash)
- Send and receive JSON-RPC messages over TCP
- Poll job status and forward events to the engine
- Health check loop every 60 seconds
- State persistence and recovery

---

### `plugin-sdk`

Toolkit for plugin authors. Hides all TCP and JSON-RPC boilerplate.

A complete plugin in ~20 lines:

```rust
use crankshaft_plugin_sdk::prelude::*;

struct MyBackend;

#[async_trait]
impl PluginHandler for MyBackend {
    async fn submit(&self, job: Job) -> PluginResult<JobId> {
        // submit to your backend here
        Ok(JobId::new("job-001"))
    }

    async fn status(&self, id: JobId) -> PluginResult<JobStatus> {
        Ok(JobStatus::Completed)
    }

    async fn cancel(&self, id: JobId) -> PluginResult<()> {
        Ok(())
    }
}

plugin_main!(MyBackend);
```

---

### `plugin-example`

Reference implementation using local shell execution.

- Runs jobs via `std::process::Command`
- Captures stdout and stderr
- Tracks job state in memory
- Full end-to-end test with the plugin host

---

## Configuration

```toml
[backend]
type = "plugin"

[backend.plugin]
path   = "/path/to/my-backend-plugin"
port   = 7878
timeout_secs = 30
```

---

## Runtime Flow

**Job Submission**

```
User
 └─▶ Crankshaft
       └─▶ PluginBackend::submit()
             └─▶ Plugin Host
                   └─▶ JSON-RPC: submit { job }
                         └─▶ Plugin Process
                               └─▶ PluginHandler::submit()
                                     └─▶ Returns JobId
```

**Status Polling**

```
Plugin Host (loop every N seconds)
 └─▶ JSON-RPC: status { job_id }
       └─▶ Plugin Process
             └─▶ Returns JobStatus
                   └─▶ Completed / Failed → exit loop
```

---

## Reliability

| Feature | Description |
|---|---|
| Process isolation | Plugin crash never affects the engine |
| Auto-restart | Up to 3 restart attempts with exponential backoff |
| Health checks | Ping every 60s; stuck plugin triggers restart |
| State persistence | Job state written to `~/.crankshaft/plugin-state.json` |
| Startup retry | 5 TCP connection attempts with 500ms delay |
| Structured logging | `tracing` crate throughout — `info`, `warn`, `error`, `debug` |

---

## Development Roadmap

| Phase | Crate | Status |
|---|---|---|
| 1 | `plugin-core` | 🔄 In Progress |
| 2 | `plugin-host` | ⬜ Not Started |
| 3 | `plugin-sdk` |  ⬜ Not Started |
| 4 | `plugin-example` |  ⬜ Not Started |
| 5 | Crankshaft config integration | ⬜ Not Started |
| 6 | Failure handling + observability |  ⬜ Not Started |

---

## Design Principles

**Extensibility** — New backends ship as independent binaries. No Crankshaft fork required.

**Isolation** — Subprocess boundary means a plugin crash cannot corrupt the engine.

**Simplicity** — Plugin authors implement one trait and call `plugin_main!`. Everything else is handled.

**Reliability** — Crash recovery, state persistence, and health checks are built into the host, not left to plugin authors.

**Stability** — JSON-RPC message schema is versioned. Host and plugin can be updated independently.

---


Built with direct reference to the [Crankshaft](https://github.com/stjude-rust-labs/crankshaft) and [Sprocket](https://github.com/stjude-rust-labs/sprocket) codebases maintained by St. Jude Rust Labs.

---

## License

MIT OR Apache-2.0
