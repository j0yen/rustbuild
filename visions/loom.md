# Vision: loom — parallel same-target /build branches that weave without tangling

## TL;DR

`/build` fans out several rust-extend branches per tick, each in its own git
worktree off a shared `build_into` repo's `main` HEAD. The expensive work
(build/clippy/test) runs in parallel; the cheap work (merge + version bump)
runs serially under `worktree-extend.sh integrate`. The model assumes
non-overlapping diffs. When two same-target branches edit the **same lines** —
the `main.rs` subcommand match, the `lib.rs` pub surface, `Cargo.lock`,
`Cargo.toml` — the second branch's merge conflicts, `integrate` aborts and
defers (exit 4 / exit 3), and a naïve retry re-runs the same free-form edit and
**re-conflicts forever**. No forward progress without a human. **loom** makes
parallel same-target integration clean by construction (append-only edit
conventions + lockfile regen) and self-healing when it isn't (auto-rebase the
loser onto the just-merged HEAD and retry; route persistently-conflicting
targets serially).

## Why this is real (Phase 1 evidence)

- **gossip 2026-06-06T06:39:07Z** ("build reflect candidate — Integrate-collision
  pattern"): this tick, 4/8 parallel same-target branches built green in their
  worktrees but **deferred at integrate** because siblings touched the same
  files. `concord-bridge` + `concord-cruxes` → both exit 3 (Cargo.lock dirty
  from the other); **neither** integrated. `quicken-attest` → exit 4 (conflict
  in main.rs/Cargo.toml vs `quicken-remedy`, which won). `anchor-probe` → exit 4
  (conflict in lib.rs/main.rs vs `anchor-reconcile`, which won).
- **journal 2026-06-05**: `quicken-attest` "merge conflict with quicken-remedy
  on main.rs — branch preserved for integration next tick"; `anchor-probe`
  "integrate blocked by merge conflict with anchor-reconcile (both touched
  lib.rs+main.rs same tick) — branch kept for next tick rebase".
- **Code: `~/wintermute/build-skill/scripts/worktree-extend.sh:81-84`** —
  `cmd_integrate` does `git merge --no-ff`; on failure it `merge --abort`s and
  `die 4`s. There is **no rebase, no retry, no lockfile handling**. Line 76-78
  `die 3` on any dirty tree (which is how a half-applied Cargo.lock surfaces).
- **Existing PRD `build-shared-cli-dispatch-merge-safe` (status
  needs_classification)** addresses **only** the `main.rs` subcommand-dispatch
  facet (a registry + cli-register.sh + ensure-main + conflict telemetry). It
  is the CLI-dispatch leaf of this vision; loom does **not** redraft it — loom
  covers the *other* collision surfaces and the self-healing retry loop, and
  consumes its `last_error=integrate-conflict:<files>` telemetry.

## End-state

When `/build` fans out N same-target branches in one tick:

1. Distinct new subcommands never conflict (CLI registry — existing PRD).
2. Distinct new `pub` modules/types in `lib.rs` never conflict (append-only
   re-export convention — loom-libapi-append).
3. `Cargo.lock` is never a merge conflict; it is regenerated deterministically
   at integrate (loom-lockfile-regen).
4. If two branches *do* collide on real source lines, the loser **auto-rebases**
   onto the just-merged `main`, re-checks, and retries the merge once before
   deferring — turning "stalls forever" into "lands one tick later"
   (loom-rebase-retry).
5. A target whose branches keep conflicting after rebase is detected and routed
   **serially** (one same-target branch per tick) instead of re-fanned in
   parallel (loom-serial-fallback).

The net: parallel same-target throughput stops silently collapsing to "first
branch wins, rest stall," which is what's happening today.

## Components (PRD-sized)

- **loom-rebase-retry** (self-mod, `worktree-extend.sh integrate`) — on merge
  conflict, rebase `autobuilder/<slug>` onto the new `main` HEAD; if rebase is
  clean and `cargo check --offline` passes, retry the merge (now a fast-forward)
  instead of deferring. Only defer (exit 4) if rebase itself conflicts or the
  re-check fails. The load-bearing self-healing piece.
- **loom-lockfile-regen** (self-mod, `worktree-extend.sh` + a `.gitattributes`
  convention) — `Cargo.lock` gets a `merge=ours`-style driver and is
  regenerated (`cargo generate-lockfile`/`--offline` canonicalization) after the
  source merge, so two branches that each touched the lockfile never block on it.
- **loom-libapi-append** (self-mod + shell helper) — `scripts/lib-register.sh`
  appends `pub mod`/`pub use` lines at an anchored end-of-list region in
  `lib.rs` (mirrors the existing cli-register.sh idea for the library surface);
  shared-lib branch prompts call it instead of free-form editing `lib.rs`.
- **loom-serial-fallback** (self-mod, /build dispatcher) — read the
  `integrate-conflict:<files>` sidecar telemetry; when the same target's
  branches conflict on the same pathset ≥2 consecutive ticks, stop fanning that
  target in parallel and route it one-branch-per-tick until it drains.

## Order

```
build-shared-cli-dispatch-merge-safe  (EXISTING — CLI-dispatch leaf, ships independently)
loom-rebase-retry  ─┐
loom-lockfile-regen ─┤  all three edit worktree-extend.sh / build-skill:
loom-libapi-append ─┘  SERIALIZE them (same self-mod target — eat our own
                       dog food; do NOT fan these three in parallel).
loom-serial-fallback   LAST — consumes the conflict telemetry that
                       build-shared-cli-dispatch-merge-safe writes.
```

## Open questions (held for next pass / user)

- `loom doctor` — a read-only report of targets with ≥2 stalled same-target
  `in_progress` branches and their conflict pathsets. Useful but not load-bearing;
  draft once serial-fallback is live and writing the telemetry it would read.
- Should rebase-retry cap at 1 retry (proposed) or escalate to a small N with
  backoff? Start at 1; widen only if evidence shows 2-deep stacks are common.
- `Cargo.lock` driver choice: `merge=ours`+regen (proposed, deterministic) vs a
  true union driver (risks invalid TOML). Lock the decision in loom-lockfile-regen.
