# crankshaft-plugin-core

Shared types and contracts between Crankshaft (the host) and any plugin
process. Neither side does anything useful without agreeing on these first.

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

## Status

Core types and the handler trait are implemented and tested
(`cargo test -p plugin-core`). Not yet decided: whether the host-side
contract (used for `Box<dyn ...>` dispatch alongside Crankshaft's existing
Docker/Slurm backends) should be a separate trait from `PluginHandler`,
mirroring the host/SDK split in the rest of the roadmap.