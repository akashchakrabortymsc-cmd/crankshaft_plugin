## [Unreleased]
### Changed
- `Job` now supports multiple executions (`executions: NonEmpty<JobExecution>`),
  matching Crankshaft's `Task`/`NonEmpty<Execution>` model — per Clay's guidance
  that TES supports multiple co-located executions per task.
- `Resources` fields renamed/re-typed to match `crankshaft_engine::task::Resources`
  units exactly (`ram`/`ram_limit`/`disk` now `f64` in GiB, not `u64` MB).
- Added `JobExecution` (image, program, args, work_dir, stdin/stdout/stderr, env)
  as the wire-format mirror of Crankshaft's `Execution`.
- `JobId`'s inner field is now private; use `JobId::new()` / `.as_str()`.

### Added
- `crankshaft-engine` as a direct dependency of `plugin-core`.
- `Job::from_task(id, &Task)` — converts a Crankshaft `Task` into a wire-format `Job`.