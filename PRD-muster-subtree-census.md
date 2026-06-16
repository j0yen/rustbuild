# PRD: muster-subtree-census

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

`muster census` claims to reconcile each Claude session against its agorabus
worker subtree, but the `worker_pids` field returns `[]` even for sessions that
have dozens of live workers. This PRD makes the subtree real: populate
`worker_pids`, and add a `subtree` block reporting subscriber PIDs, worker PIDs,
deleted-exe (stale-binary) count, and distinct worker generations — the data
every downstream verdict and reap needs.

## Why this exists

Verified live on this box (2026-06-16):

- `muster census --format json` for pid 1054 returns `"worker_pids": []`.
- `pgrep -fc "agorabus-worker.sh claude-1054-jsy"` = **21** live workers for
  that same session.
- 3 of session 1054's `agorabus subscribe` children are deleted-exe:
  `readlink /proc/{1906,2201,579203}/exe` each ends in ` (deleted)`.

So the field the vision's End-state §1 promised ("its agorabus peer session-id +
worker subtree PIDs") is declared in `census.rs` but never populated. muster
cannot see the subtree it is supposed to reconcile, which is why the recurring
self-review finding "fleet-binary-staleness: 3 deleted-exe agorabus subs" has
to be hand-rolled with `binstale`/`readlink` instead of read from the tool that
owns the session→subtree map.

## What this builds

Extends `~/wintermute/muster/src/census.rs` (and the JSON shape consumed by
`verdict.rs`):

1. **Populate `worker_pids`.** Match a session's workers by argv
   `agorabus-worker.sh <sid>` (the live argv is
   `bash …/agorabus-worker.sh claude-<pid>-jsy /home/jsy` — match the sid token,
   tolerating the trailing cwd arg). Include both the `<sid>` and
   `<sid>-worker` connection forms.
2. **Add a `subtree` object** to each roster entry:
   - `subscriber_pids`: live `agorabus subscribe … --session-id <sid>[-worker]`.
   - `worker_pids`: as above (also keep the top-level field for back-compat).
   - `deleted_exe`: count of subtree members whose `/proc/<pid>/exe` symlink
     resolves to a path ending ` (deleted)`.
   - `worker_generations`: count of distinct live `agorabus-worker.sh <sid>`
     PIDs (the leak magnitude — 21 for pid 1054 today).
3. Read `/proc/<pid>/exe` and `/proc/<pid>/cmdline` directly; degrade
   gracefully (skip, don't error) on EACCES/ESRCH races.
4. Text output gains a compact subtree summary column (e.g.
   `sub=N work=M dead=K`); JSON gains the `subtree` block.

Deps: existing crate deps only (procfs walk already present for census). No new
external crates expected; if the crate already reads `/proc` via a helper,
reuse it.

## Acceptance criteria

1. `muster census --format json` populates `worker_pids` with a non-empty array
   for any session that has ≥1 live `agorabus-worker.sh <sid>` process
   (regression fixture reproduces the trailing-cwd-arg argv that currently
   yields `[]`).
2. Each JSON roster entry contains a `subtree` object with integer fields
   `subscriber_pids` (array), `worker_pids` (array), `deleted_exe`, and
   `worker_generations`.
3. `deleted_exe` correctly counts subtree members whose `/proc/<pid>/exe` ends
   in ` (deleted)`, proven by a unit test over a faked procfs layout (or an
   injected enumerator) with a mix of present and deleted exes.
4. A `/proc` read that races (pid exits mid-scan; EACCES) is skipped without
   failing the census; covered by a test.
5. Text output shows a subtree summary token per session and remains parseable
   (no column-count regression in existing census output tests).
6. `cargo test` green; `cargo build` clean. Existing census/verdict/reap tests
   still pass (additive change).
