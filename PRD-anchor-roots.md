# PRD: anchor-roots — the declared set of watch roots, and the plan to restore it

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/anchor`
**Vision:** visions/anchor.md

## TL;DR

watchman holds its watched-root list only in memory, so a reboot or a socket
bounce silently drops every root — and the `wchg` delta layer then returns
empty *without saying why*. There is no canonical record of which roots
*should* be watched, and no tool that diffs declared-vs-live. **anchor-roots**
creates the `anchor` workspace and the foundation the rest of the vision
extends: a declared-roots manifest loader (`RootsConfig`), the shared types
(`WatchRoot` / `WatchState` / `RootStatus` / `ReconcilePlan` / `ReconcileAction`),
the `WatchBackend` trait, and a **pure** `reconcile` that compares the declared
set against the live watchman roots and emits a print-only action plan.
`anchor plan` shows it at a glance.

## Why this exists

- **The loss is recurring and self-evidenced.** `recall:01KT6VCBX1PSWT9MAQP607TAWY`
  (score 10.3, reaffirmed across reboots): *"watchman drops watched roots
  across reboot — wchg since/reset silently empty/error until re-watch."*
  Reinforced by `01KSTJ29…`, `01KSTRY6…`, `01KSMZNQ…`, `01KSSFPE…`, `01KS95K6…`.
- **The mechanism is confirmed live.** `watchman.service` is socket-activated
  `/usr/bin/watchman --foreground --inetd` (observed 2026-06-05, pid 2596,
  started 2026-06-03 06:35) — no persisted-root restore in this unit. `wchg
  list` shows a *mix* of clock ages (`c:1779915649…` mid-May beside
  `c:1780493700…`), the signature of ad-hoc per-root re-watching with no
  canonical declared set.
- **There is no record of intent.** Today self-review re-watches `~/.claude` +
  `~/brain` by hand each reboot (journals 06-01/02/03). The set of roots that
  *should* exist lives nowhere; it must become a versioned manifest before
  anything can reconcile against it.
- This is the FIRST PRD of the fleet: it creates the repo + shared types +
  the `WatchBackend` trait. The relay/concord/quicken/keel passes all learned
  the hard rule — **no rust-extend starts until the corpus repo has SHIPPED**
  or extend-validate fails. anchor-roots is anchor's corpus.

## What this builds

New cargo workspace at `~/wintermute/anchor` with an `anchor` binary crate.

**The declared-roots manifest** — `~/.config/anchor/roots.toml`:

```toml
# Each root self-review / wchg consumers depend on.
[[root]]
path = "/home/jsy/.claude"
max_age_secs = 86400      # a clock older than this is "stale" (probe uses it)

[[root]]
path = "/home/jsy/brain"

[[root]]
path = "/home/jsy/wintermute"
```

`RootsConfig::load(path)` parses it; a default set ships in the repo
(`config/roots.example.toml`) covering the roots observed live today
(`~/.claude`, `~/brain`, `~/wintermute`, `~/.local/bin`,
`~/.config/systemd/user`, `~/.cache/ctrace/sessions`). Loading is pure — no
filesystem watch, no watchman call.

**Core types (the surface probe/reconcile/boot extend):**

- `RootStatus` enum: `Watched`, `Missing` (declared but not live), `Stale {
  age_secs }` (watched but clock older than `max_age_secs`), `Undeclared`
  (live but not in the manifest — informational, never auto-removed).
- `WatchRoot { path: PathBuf, max_age_secs: Option<u64> }` — one manifest entry.
- `WatchState { path: PathBuf, clock: Option<String>, present: bool }` — one
  live watchman root as seen through the backend.
- `ReconcileAction` enum: `Watch { path }`, `ReseedCursor { path }`,
  `NoOp { path }`, `NoteUndeclared { path }`.
- `ReconcilePlan { actions: Vec<ReconcileAction>, summary: … }` — the diff
  result; **declarative, no side effects** at this layer.

**The `WatchBackend` trait** — abstracts watchman so reconcile is pure and
testable, and so a later backend swap (or watchman replacement) doesn't touch
the diff logic:

```rust
pub trait WatchBackend {
    fn live_roots(&self) -> Result<Vec<WatchState>>;   // watch-list
    fn watch(&self, path: &Path) -> Result<()>;        // re-assert (apply-only; unused here)
    fn reseed_cursor(&self, path: &Path) -> Result<()>;// wchg reset (apply-only; unused here)
}
```

anchor-roots ships a real `WatchmanBackend` (shells `watchman watch-list` /
`watchman watch`) **and** a `FakeBackend` for tests. The pure `reconcile(declared:
&[WatchRoot], live: &[WatchState], now: i64) -> ReconcilePlan` takes both lists +
an *injected* clock and returns the plan — it never calls the backend, never
reads the real clock.

**UX:** `anchor plan` → table of `root | status | action`; `--format json` for
machines. Print-only — no `--apply` in this PRD (that's anchor-reconcile). Exit
non-zero if any declared root is `Missing` (so a hook can gate on it even before
`--apply` exists).

**Deps:** minimal — `serde`/`serde_json`, `toml`, `clap`. MSRV 1.85, no
let-chains (`self_recall_baseline_gate_red` discipline). `sigpipe::reset()` as
the first line of `main()` (`self_sigpipe_panic_toolkit` — `anchor plan | head`
must not coredump). Match the workspace shape of sibling toolkit repos
(vigil/quicken/keel): `clippy.toml`, `deny.toml`, `rust-toolchain.toml`,
`CHANGELOG.md`, `README.md`.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed offline; a test asserts `reconcile`
   makes **zero** backend calls (it operates only on the two passed slices +
   the injected `now`).
2. `RootStatus`, `WatchRoot`, `WatchState`, `ReconcileAction`, `ReconcilePlan`
   are public and `serde`-(de)serializable; a round-trip test covers each.
3. `RootsConfig::load` parses `config/roots.example.toml`; a test with a fixture
   manifest yields the expected `Vec<WatchRoot>`, including a per-root
   `max_age_secs` override.
4. `reconcile` classifies correctly against a `FakeBackend` snapshot: a declared
   root absent from live → `Missing` + `Watch` action; a watched root with a
   clock older than `max_age_secs` (injected `now`) → `Stale` + `ReseedCursor`;
   a live root not in the manifest → `Undeclared` + `NoteUndeclared` (never a
   removal); a fresh watched root → `Watched` + `NoOp`.
5. `anchor plan --format json` emits one entry per declared root plus
   undeclared-live notes; the schema matches the documented `ReconcilePlan`.
6. `anchor plan` exits non-zero when at least one declared root is `Missing`,
   zero when all are `Watched`/`Stale` only; covered by two integration cases
   driving a `FakeBackend`.
7. `anchor plan | head -1` does not panic (SIGPIPE reset verified by a test that
   closes the read end early).
8. README documents the manifest format, the type surface, and the
   `WatchBackend` trait so anchor-probe / anchor-reconcile / anchor-boot have a
   contract to extend.
