# PRD: threshold-hook — wire the briefing in, and never block the boot

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/threshold
Vision: visions/threshold.md

## TL;DR

`threshold brief` (+ verify + ledger) only helps if a new session actually reads
it. This PRD wires `threshold brief` as a consolidating SessionStart hook,
records each arrival in the ledger, and guarantees the session still starts —
falling back to today's raw firehose — if the binary is missing or errors. The
briefing is additive; it is never load-bearing for boot.

## Why this exists

Phase-1 inspection, 2026-06-18:

- Ten SessionStart hooks fire independently today (`jq '.hooks.SessionStart'
  ~/.claude/settings.json`), producing the 20,888-byte firehose this fleet
  exists to tame. None of them synthesizes the others; there is no consolidation
  point.
- SessionStart hooks are on the critical path of every session. A hook that
  errors or hangs degrades or blocks startup — so the wiring must be defensive:
  exit 0 always, time-bounded, and degrade to the existing behavior if
  `threshold` is absent. (The standing lesson from `wmd-init` and the kernel
  banners this pass: a hard dependency on an un-shipped primitive — e.g. a live
  agentns id — must never break the common path.)
- `update-config`/settings.json is the harness-sanctioned way to register hooks;
  this PRD must go through it, not hand-edit around it.
- The vision's open question (replace vs wrap the ten hooks) resolves here:
  **wrap, don't replace** — a big-bang cutover of ten working hooks is the risky
  move; consolidation can come incrementally.

## What this builds

Extends `~/wintermute/threshold` plus a settings.json hook registration.

- **`threshold brief --hook` mode:** a SessionStart-shaped entrypoint that
  renders the synthesized briefing (with verify badges + open questions from the
  earlier PRDs) to stdout, **always exits 0**, is time-bounded (internal
  deadline; partial briefing on timeout beats a hung hook), and records the
  arrival in the ledger (session id, ts).
- **Degrade path:** a tiny shell shim
  (`~/.claude/scripts/threshold-session-start.sh`) that runs `threshold brief
  --hook` if the binary exists and is executable, and otherwise exits 0 silently
  so the ten existing hooks remain the sole output (no regression, no error).
- **Registration:** add the shim to `.hooks.SessionStart` via the
  settings.json mechanism. Ordering: register it *first* so its synthesized
  briefing reads above the raw dumps (which still run, per wrap-don't-replace).
- **Arrival record:** each hook fire appends an `arrival` record to the threshold
  ledger so successors can see when predecessors started (and `threshold open`
  context is anchored in real session boundaries).

Out of scope: removing or rewriting any of the ten existing hooks (a later
`threshold-spool` PRD can teach them to write to a spool the brief consumes);
bus publication of arrivals.

## Acceptance criteria

1. `cargo build` / `cargo test` green; clippy adds no new warnings over baseline.
2. `threshold brief --hook` exits 0 even when every signal source and the ledger
   are unavailable (test-proven: point it at empty/nonexistent roots — still
   exit 0, still emits at least a minimal header).
3. `threshold brief --hook` honors an internal time bound: a test that injects a
   slow `FakeSource` shows the command returns within the documented deadline
   with a partial briefing rather than hanging.
4. The shell shim exits 0 and emits nothing when the `threshold` binary is absent
   or non-executable (test by running it with an empty PATH / renamed binary).
5. After the hook fires, an `arrival` record exists in the threshold ledger for
   the current session id (test against a temp ledger).
6. The settings.json registration is applied through the supported mechanism, the
   shim is listed in `.hooks.SessionStart`, and a fresh session shows the
   synthesized briefing **above** the existing raw hook output (manually verified;
   note in the PRD close that hook-ordering is observation-verified, not unit-
   testable).
7. Removing/renaming the `threshold` binary leaves session startup fully
   functional with the original ten-hook firehose (degrade proven, no boot
   regression).
