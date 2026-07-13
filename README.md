# crankshaft-plugin

A production-oriented plugin architecture for [Crankshaft](https://github.com/stjude-rust-labs/crankshaft) that enables external execution backends to be developed, distributed, and maintained independently from the core engine.

> **Status:** 🚧 Early Development — Part 1 (`plugin-core`) in progress.

---

## The Problem

Crankshaft currently supports a fixed set of execution backends: Docker, Slurm.

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
