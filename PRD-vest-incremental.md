# PRD: vest-incremental — skip the reinstall when the source hasn't changed

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/vest.md

## TL;DR

`adopt-cron.service` runs `adopt apply --execute --with-daemons` every
6h, and `adopt apply` runs `cargo install --force` on every stale
artifact unconditionally — even when the artifact's source is
byte-identical to the last successful install. The unit's own
accounting reports `Consumed 3min 33.285s CPU time … 1.1G memory peak`
for a single run, and because the backlog never converges (84/84
not-current, per PRD-vest-verify), every 6-hour tick re-pays that cost
to reinstall binaries that did not change. `vest-incremental` records a
per-artifact source marker after each successful install and skips the
`cargo install` when the marker is unchanged — turning the steady-state
run into a near-instant no-op.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The cost is metered.** `journalctl --user -u adopt-cron.service`:
  `adopt-cron.service: Consumed 3min 33.285s CPU time over 42.369s wall
  clock time, 1.1G memory peak` — for a run that adopted one binary and
  aborted. A full successful sweep of ~30+ crates is far heavier.
- **`--force` is unconditional.** `adopt/src/scan.rs:308` builds
  `cargo install --force --path {repo} --root …` for every stale
  artifact; `apply.rs` execs it with no "is this already current?"
  short-circuit beyond the binary's existence.
- **The cadence is fixed.** `systemctl --user list-timers adopt*` →
  `adopt-cron.timer` fires every 6h. With no skip, that is 4 full
  rebuild attempts/day of unchanged source.
- **Backlog non-convergence amplifies it.** Because today's installs
  read back FAILED (the tilde/PATH issues), nothing is ever recorded as
  "current", so even a correct apply would rebuild everything each run.
  An explicit success marker breaks that loop.

## What this builds

Extend `adopt` (`~/wintermute/adopt`). No new crate. Depends on
PRD-vest-root-guard landing first (installs must actually reach the real
prefix before a "current" marker is meaningful).

### Per-artifact install marker

After a *verified* successful install (cargo exit 0 **and**
`is_invokable` true), write a marker recording what was installed:

- Location: `~/.local/state/adopt/markers/<bin>.json` (respect
  `$XDG_STATE_HOME` if set; fall back to `~/.local/state`).
- Contents: `{ bin, repo_path, source_fingerprint, installed_at }`.
- `source_fingerprint`: the repo's current git HEAD commit if the repo
  is a git worktree and clean; otherwise the max mtime across the
  repo's tracked source files (`src/**`, `Cargo.toml`, `Cargo.lock`).
  A dirty worktree always fingerprints as "changed" (never skipped).

### Skip logic in `apply`

Before building the `cargo install` command for an artifact:

- Compute the current `source_fingerprint`.
- If a marker exists, its fingerprint matches, **and** the binary is
  still `is_invokable`, record `ApplyOutcome::AlreadyCurrent` and skip
  the install entirely (no `cargo` spawn).
- Otherwise install as today, and on verified success write/update the
  marker.
- A `--force-all` flag bypasses the skip (escape hatch for "rebuild
  everything regardless").

## Acceptance criteria

1. After a successful install in a test, a marker file exists at the
   state path with `bin`, `repo_path`, `source_fingerprint`,
   `installed_at`.
2. A second `adopt apply --execute` with an unchanged source fingerprint
   records `ApplyOutcome::AlreadyCurrent` for that artifact and spawns
   **no** `cargo install` (verified by asserting cargo is not invoked).
3. Touching a tracked source file (changing the fingerprint) causes the
   next apply to reinstall, not skip.
4. A dirty git worktree fingerprints as changed and is never skipped.
5. `--force-all` reinstalls even when the marker matches.
6. A skipped (`AlreadyCurrent`) artifact still reads as current in the
   apply summary; the summary distinguishes installed / already-current
   / skipped-daemon / failed counts.
7. `cargo test` green; `cargo build --release` clean; marker path
   honors `$XDG_STATE_HOME`.

## Out of scope

- Changing the cron cadence (the timer stays 6h; the win is per-run cost).
- Cross-machine marker sharing (fleet constellation concern, not here).
- Reinstalling daemons (delegated to `rollout`, unchanged).
