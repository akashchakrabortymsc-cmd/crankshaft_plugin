# Crankshaft Plugin System (Rust)

<<<<<<< HEAD
> A production-oriented plugin architecture for Crankshaft that enables external execution backends to be developed, distributed, and maintained independently from the core engine.

---
=======
A production-oriented plugin architecture for Crankshaft that enables external execution backends to be developed, distributed, and maintained independently from the core engine.
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282

## Vision

Crankshaft currently supports built-in execution backends such as Docker, Slurm, and other HPC systems.

This project introduces a generic plugin framework that allows new execution backends to be added without modifying Crankshaft itself.

Instead of compiling every backend into the engine, Crankshaft will communicate with external plugins through a stable RPC interface.

The long-term goal is:

- Extensible backend ecosystem
- Independent plugin development
- Stable versioned interfaces
- Fault isolation
- Production-grade reliability

Inspired by:

- Nextflow Plugin System
- PF4J
<<<<<<< HEAD
- Hashicorp Plugin Framework
- Language Server Protocol (LSP)

---

# Architecture Overview

```text
=======
- HashiCorp Plugin Framework
- Language Server Protocol (LSP)

## Architecture Overview

```
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
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
      ┌───────────────┼────────────────┐
      │               │                │
      ▼               ▼                ▼
 DockerBackend  SlurmBackend  PluginBackend
                                     │
                                     ▼
                               Plugin Host
                                     │
                            JSON-RPC over TCP
                                     │
            ┌────────────────────────┴─────────────────────┐
            │                                              │
            ▼                                              ▼
      Kubernetes Plugin                           Custom HPC Plugin
            │                                              │
            ▼                                              ▼
      Real Compute Resources                      Real Compute Resources
```

<<<<<<< HEAD
---

# Core Design Principles

## 1. Extensibility

New backends should be installable without modifying Crankshaft source code.

## 2. Isolation

A plugin crash should never crash the engine.

## 3. Simplicity

Plugin authors should only implement a small Rust trait.

## 4. Reliability

State persistence, crash recovery, retries, and health checks are built into the system.

## 5. Versioning

Host and plugins communicate through stable interfaces.

---

# Project Structure

```text
crankshaft-plugin-system/
│
├── crankshaft-plugin-core
│
├── crankshaft-plugin-host
│
├── crankshaft-plugin-sdk
│
├── crankshaft-plugin-example
│
└── docs
```

---

# Crates

## crankshaft-plugin-core

Shared contracts between the host and plugins.

### Responsibilities

=======
## Core Design Principles

1. **Extensibility** — New backends should be installable without modifying Crankshaft source code.
2. **Isolation** — A plugin crash should never crash the engine.
3. **Simplicity** — Plugin authors should only implement a small Rust trait.
4. **Reliability** — State persistence, crash recovery, retries, and health checks are built into the system.
5. **Versioning** — Host and plugins communicate through stable interfaces.

## Project Structure

```
crankshaft-plugin-system/
│
├── crankshaft-plugin-core
├── crankshaft-plugin-host
├── crankshaft-plugin-sdk
├── crankshaft-plugin-example
└── docs
```

## Crates

### crankshaft-plugin-core

Shared contracts between the host and plugins.

**Responsibilities**
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
- Job types
- Status types
- Error handling
- Shared traits
- RPC message definitions

<<<<<<< HEAD
### Example

```rust
pub trait PluginBackend {
    async fn submit(&self, job: Job)
        -> PluginResult<JobId>;

    async fn status(&self, id: JobId)
        -> PluginResult<JobStatus>;

    async fn cancel(&self, id: JobId)
        -> PluginResult<()>;
}
```

---

## crankshaft-plugin-host
=======
**Example**

Native `async fn` in traits isn't object-safe yet, and `PluginBackend` needs to be stored as a `dyn` trait object inside the Backend Factory (alongside `DockerBackend` and `SlurmBackend`), so the trait is defined with `async-trait`:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait PluginBackend: Send + Sync {
    async fn submit(&self, job: Job) -> PluginResult<JobId>;
    async fn status(&self, id: JobId) -> PluginResult<JobStatus>;
    async fn cancel(&self, id: JobId) -> PluginResult<()>;
}
```

### crankshaft-plugin-host
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282

Runs inside the engine.

Responsible for:
<<<<<<< HEAD

=======
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
- Spawning plugins
- Managing plugin lifecycle
- Sending RPC requests
- Receiving responses
- Retry logic
- Health checks
- State recovery

<<<<<<< HEAD
---

## crankshaft-plugin-sdk

Plugin author toolkit.

Allows backend authors to build a plugin with minimal code.

Example:

```rust
struct MyHandler;

impl PluginHandler for MyHandler {
    async fn execute(
        &self,
        job: Job,
    ) -> PluginResult<JobId> {
=======
### crankshaft-plugin-sdk

Plugin author toolkit. Allows backend authors to build a plugin with minimal code.

```rust
use async_trait::async_trait;

struct MyHandler;

#[async_trait]
impl PluginHandler for MyHandler {
    async fn execute(&self, job: Job) -> PluginResult<JobId> {
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
        todo!()
    }
}
```

<<<<<<< HEAD
---

## crankshaft-plugin-example

Reference implementation.

Provides:

=======
### crankshaft-plugin-example

Reference implementation. Provides:
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
- Working plugin
- Example job execution
- Integration tests
- Documentation

<<<<<<< HEAD
---

# Runtime Flow

## Job Submission

```text
User
 │
 ▼
Crankshaft
 │
 ▼
PluginBackend.submit()
 │
 ▼
Plugin Host
 │
 ▼
JSON-RPC Request
 │
 ▼
Plugin Server
 │
 ▼
PluginHandler.execute()
 │
 ▼
Compute Backend
 │
 ▼
JobId Returned
```

---

## Status Polling

```text
Crankshaft
     │
     ▼
status(job_id)
     │
     ▼
Plugin Host
     │
     ▼
RPC Request
     │
     ▼
Plugin
     │
     ▼
Job Status
     │
     ▼
Completed
```

---

# Configuration

Example:
=======
## Runtime Flow

**Job Submission**

```
User → Crankshaft → PluginBackend.submit() → Plugin Host
     → JSON-RPC Request → Plugin Server → PluginHandler.execute()
     → Compute Backend → JobId Returned
```

**Status Polling**

```
Crankshaft → status(job_id) → Plugin Host → RPC Request
          → Plugin → Job Status → Completed
```

## Configuration
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282

```toml
[backend]
type = "plugin"

[backend.plugin]
path = "/plugins/kubernetes-plugin"
port = 7878
timeout_secs = 30
```

<<<<<<< HEAD
---

# Reliability Features

## Crash Detection

The host continuously monitors plugin processes.

If a plugin crashes:

=======
## Reliability Features

**Crash Detection**

The host continuously monitors plugin processes. If a plugin crashes:
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
- Jobs are marked failed
- Error is logged
- Recovery process starts

<<<<<<< HEAD
---

## Auto Restart

```text
Plugin Crash
      │
      ▼
Restart Attempt #1
      │
      ▼
Restart Attempt #2
      │
      ▼
Restart Attempt #3
=======
**Auto Restart**

```
Plugin Crash → Restart Attempt #1 → Restart Attempt #2 → Restart Attempt #3
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
```

After maximum retries the plugin is permanently marked unhealthy.

<<<<<<< HEAD
---

## State Persistence

Job state is stored on disk.

```text
~/.crankshaft/plugin-state.json
```

Benefits:

=======
**State Persistence**

Job state is stored on disk at `~/.crankshaft/plugin-state.json`.

Benefits:
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
- Engine restart recovery
- Resume monitoring
- Failure recovery

<<<<<<< HEAD
---

## Health Checks
=======
**Health Checks**
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282

Host periodically sends:

```json
<<<<<<< HEAD
{
  "method": "health_check"
}
```

If no response is received:

- Plugin considered unhealthy
- Automatic restart triggered

---

# Future Extension Points

The architecture is intentionally designed to support future plugin categories.

Potential extensions:
=======
{ "method": "health_check" }
```

If no response is received, the plugin is considered unhealthy and an automatic restart is triggered.

## Future Extension Points

The architecture is intentionally designed to support future plugin categories:
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282

- Execution Backends
- Storage Providers
- Authentication Providers
- Metrics Exporters
- Workflow Extensions
- Scheduling Policies

<<<<<<< HEAD
---

# Development Roadmap

## Phase 1

plugin-core

- Job types
- Status types
- Traits
- Errors

## Phase 2

plugin-host

- TCP communication
- JSON-RPC
- Process management

## Phase 3

plugin-sdk

- PluginServer
- PluginHandler
- Configuration helpers

## Phase 4

Reference plugin

- Local execution backend
- End-to-end validation

## Phase 5

Crankshaft integration

- Config support
- Backend registration

## Phase 6

Production reliability

- Logging
- Persistence
- Recovery
- Chaos testing

---

# Learning Objectives

This project explores:

=======
## Development Roadmap

| Phase | Crate | Focus |
|---|---|---|
| 1 | plugin-core | Job types, status types, traits, errors |
| 2 | plugin-host | TCP communication, JSON-RPC, process management |
| 3 | plugin-sdk | PluginServer, PluginHandler, config helpers |
| 4 | plugin-example | Local execution backend, end-to-end validation |
| 5 | Crankshaft integration | Config support, backend registration |
| 6 | Production reliability | Logging, persistence, recovery, chaos testing |

## Learning Objectives

This project explores:
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282
- Rust async programming
- Tokio
- Distributed systems
- RPC design
- Plugin architectures
- Process management
- Reliability engineering
- Workflow execution engines

<<<<<<< HEAD
---

# Status

🚧 Early Development

Currently building the foundational architecture and validating concepts through a toy implementation before integration with Crankshaft.

---

# Acknowledgements

Inspired by:

- Nextflow Plugin System
- Hashicorp Plugin Framework
- PF4J
- Crankshaft
- Sprocket

Special thanks to the Crankshaft maintainers for guidance and feedback during the learning process.

---

# License
=======
## Status

🚧 **Early Development**

Building the foundational architecture and validating concepts through an independent toy implementation, with the goal of eventually proposing it for integration into Crankshaft.

## Acknowledgements

Inspired by the Nextflow Plugin System, the HashiCorp Plugin Framework, and PF4J. Built with direct reference to the existing Crankshaft and Sprocket codebases maintained by St. Jude Rust Labs.

## License
>>>>>>> bd78bbc387b79fe981cfb97c269a1e2cb4f22282

MIT OR Apache-2.0
