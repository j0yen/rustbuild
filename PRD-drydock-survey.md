# PRD: drydock-survey — one normalized inventory of the fleet's drift

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/drydock.md
**Repo:** j0yen/drydock-survey (NEVER AtScaleInc)

## TL;DR

Three good tools each answer part of "what has drifted" — `binstale` for running
daemons, `adopt` for non-daemon CLIs, and an ad-hoc `pacman -Q` for the kernel
package — and nothing stitches them together. Every self-review re-stitches them
by hand into prose. drydock-survey is the read-only aggregation layer the rest of
the drydock fleet trusts: it shells out to `binstale fleet --format json` and
`adopt verify --format json`, runs a kernel-package staleness probe, and emits
one normalized JSON inventory of every drifted item. It restarts nothing,
installs nothing, deletes nothing.

## Why this exists

Verified 2026-06-16:

- **Every self-review for a week+ re-reports the same staleness wall.** Journals
  2026-06-12 through 2026-06-16 each carry a "fleet-binary-staleness" line and an
  "adopt … installed-stale (many)" line; 2026-06-16's "Pending your call"
  section lists wm-stt 9d stale, kernel pkgrel staged-not-installed, "adopt scan
  shows dozens of installed-stale," and 7 not-installed new bins — as unranked
  prose the human must re-triage daily.
- **The detectors already emit JSON.** Confirmed live: `binstale fleet --format
  json` returns rich per-PID records (`pid, comm, exe_path, verdict,
  source_repo, source_head_commit, …`); `adopt verify --format json` classifies
  not-current artifacts into named buckets. The data exists; no one aggregates it.
- **The kernel case proves a naive compare is wrong.** Booted
  `linux-wintermute 7.0.11.arch1-1` is a *higher base version* than the staged
  patched `7.0.10.arch1-12` pkgs in `~/wintermute/wintermute-kernel/pkg/` that
  carry the agentns-prctl fix. The SessionStart hook still prints
  `[agentns] ACTIVATION BLOCKED at kernel-prctl — install linux-wintermute
  pkgrel >= 12 and reboot`. A `vercmp` would call the booted kernel "fresh" and
  hide the missing patch — so staleness here is a *capability* probe, not a
  version compare.

There is no tool that turns three separate staleness feeds into one ranked
inventory. drydock-survey makes the muster first-class so routing and remediation
(drydock-classify/digest/apply) act on one ground truth.

## What this builds

A single Rust CLI `drydock-survey` (clap), no network, no mutation.

**Modules**
- `sources::daemons` — run `binstale fleet --format json`, parse, normalize each
  record to `{kind: "daemon", item: comm, pid, verdict, current_ref:
  exe_inode/proc_start summary, head_ref: source_head_commit, source_repo}`.
  A `fresh` verdict is dropped (not drift).
- `sources::clis` — run `adopt verify --format json`, normalize each not-current
  artifact to `{kind: "cli", item: bin name, verdict: failure bucket, head_ref:
  HEAD marker, source_repo}`.
- `sources::kernel` — probe kernel-package staleness by *capability*, not
  version: read the booted package (`pacman -Q linux-wintermute`), check whether
  the agentns-prctl surface is live (`/proc/self/agent_session` non-zero OR the
  inverse of the boot hook's ACTIVATION-BLOCKED state) and whether a
  higher-pkgrel patched pkg is staged in `~/wintermute/wintermute-kernel/pkg/`.
  Emit at most one `{kind: "kernel-pkg", item: "linux-wintermute", verdict:
  "patch-staged-not-booted" | "fresh", current_ref: booted pkgver-pkgrel,
  head_ref: newest staged pkgrel}`.
- `normalize` — unify into one `DriftItem` shape; compute `age_days` from the
  item's reference timestamp (daemon proc_start / source_head_ts; CLI marker ts;
  kernel staged-pkg mtime) against `--now`/`DRYDOCK_NOW`.
- `emit` — JSON array sorted by `age_days` desc, plus a summary header
  (`total_items`, counts per kind, `surveyed_at`).

**Resilience:** if a source tool is missing or errors, record a
`{kind, item:"<source>", verdict:"source-unavailable"}` sentinel and continue —
a broken `adopt` must not blind the daemon survey. Exit 0 when at least one
source succeeded; exit 2 only if all sources fail.

**Deps:** `clap`, `serde`/`serde_json`, `anyhow`. No `Date::now()` in library
code — accept `--now <rfc3339>` or `DRYDOCK_NOW` for deterministic age; CLI
reads the clock only at the boundary.

**UX**
```
drydock-survey                 # human table, oldest drift first
drydock-survey --json          # machine inventory (classify consumes this)
drydock-survey --kind daemon   # filter to one source
drydock-survey --now 2026-06-16T12:00:00Z   # deterministic age
```

## Acceptance criteria

1. `drydock-survey --json` emits a JSON array of `DriftItem`s, each with
   `kind` (daemon|cli|kernel-pkg), `item`, `verdict`, `current_ref`, `head_ref`,
   `age_days`, `source_repo` (nullable for kernel).
2. Daemon items are sourced by parsing `binstale fleet --format json`; a `fresh`
   binstale verdict produces NO drydock item (only drift is listed). Verified
   against a captured binstale JSON fixture.
3. CLI items are sourced by parsing `adopt verify --format json`; each
   not-current bucket maps to one item with its bucket as `verdict`.
4. The kernel probe emits `patch-staged-not-booted` when a higher-pkgrel patched
   pkg is staged but the booted kernel lacks the agentns-prctl capability, and
   emits nothing (or `fresh`) otherwise. Verified with a fixture where booted
   base-version > staged base-version but staged pkgrel carries the patch — a
   raw `vercmp` would wrongly say fresh; drydock must say drift.
5. Items are sorted by `age_days` descending; the summary header reports
   per-kind counts equal to the listed items.
6. Age math is deterministic under `--now`/`DRYDOCK_NOW`; a fixture with known
   reference timestamps yields known `age_days`.
7. A missing/erroring source yields a `source-unavailable` sentinel and does not
   abort the run; exit 0 if any source succeeded, exit 2 only if all failed.
8. The tool performs no restart, install, write, or delete (verified: run leaves
   the live fleet and filesystem unchanged).
9. `--help` documents every flag; `cargo test` green; `cargo clippy` clean on the
   crate's own code.
