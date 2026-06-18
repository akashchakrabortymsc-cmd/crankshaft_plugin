# Crankshaft Plugin System (Rust)

> A production-oriented plugin architecture for Crankshaft that enables external execution backends to be developed, distributed, and maintained independently from the core engine.

---

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
- Hashicorp Plugin Framework
- Language Server Protocol (LSP)

---

# Architecture Overview

```text
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

- Job types
- Status types
- Error handling
- Shared traits
- RPC message definitions

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

Runs inside the engine.

Responsible for:

- Spawning plugins
- Managing plugin lifecycle
- Sending RPC requests
- Receiving responses
- Retry logic
- Health checks
- State recovery

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
        todo!()
    }
}
```

---

## crankshaft-plugin-example

Reference implementation.

Provides:

- Working plugin
- Example job execution
- Integration tests
- Documentation

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

```toml
[backend]
type = "plugin"

[backend.plugin]
path = "/plugins/kubernetes-plugin"
port = 7878
timeout_secs = 30
```

---

# Reliability Features

## Crash Detection

The host continuously monitors plugin processes.

If a plugin crashes:

- Jobs are marked failed
- Error is logged
- Recovery process starts

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
```

After maximum retries the plugin is permanently marked unhealthy.

---

## State Persistence

Job state is stored on disk.

```text
~/.crankshaft/plugin-state.json
```

Benefits:

- Engine restart recovery
- Resume monitoring
- Failure recovery

---

## Health Checks

Host periodically sends:

```json
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

- Execution Backends
- Storage Providers
- Authentication Providers
- Metrics Exporters
- Workflow Extensions
- Scheduling Policies

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

- Rust async programming
- Tokio
- Distributed systems
- RPC design
- Plugin architectures
- Process management
- Reliability engineering
- Workflow execution engines

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

MIT OR Apache-2.0
