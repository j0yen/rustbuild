# PRD: trim-policy — default-deny gate for memory relief

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/trim
Vision: visions/trim.md

## TL;DR

trim-attribute hints which holders *might* be relief candidates;
trim-policy is the gate that turns a hint into an authorized decision —
and it is default-deny. A holder is relief-eligible ONLY if it is a
systemd-managed wintermute daemon, self-healing (`Restart=always`),
currently idle, and over the swap/resident floor. User-apps, the live
brain/dialog path during a voice TURN, build processes, and anything
outside `user.slice` are hard-excluded and cannot be overridden by
config. Extends the `trim` crate.

## Why this exists

Relief is the first trim capability that *touches* the system, so the
gate must exist before the act (mirrors how `consign-policy` gates
`consign-drain`/`publish`, and `ballast`'s guard precedes its reap).

Live evidence for each guardrail:

- **Self-healing requirement.** The agorabus memory
  (`self_agorabus_restart_kills_voice`) records that fleet daemons
  silently died on bus-close until `Restart=always` drop-ins were
  added 2026-06-05; restart is only safe for units that self-heal.
  `systemctl --user show -p Restart` distinguishes them.
- **Never mid-turn.** The voice TURN is live end-to-end
  (`project_voice_input_null_detectors`, 2026-06-04); restarting
  `wmd.service`/`wm-dialog.service` during a turn would drop the user
  mid-sentence. Policy must read turn-state (agorabus `wm-dialog`
  claim-holder presence) and refuse.
- **Never user-apps / builds.** Live holders include `firefox`,
  `slack`, `rustc` ×3 — none are ever safe to restart for memory.
- **Idle floor.** `homeward-ingest.service` runs an AIMD cadence loop
  with quiet windows; relieving it mid-batch wastes work. Idle must be
  checked, not assumed.

## What this builds

Extends the `trim` library + CLI.

**Library:**
- `policy::Decision { Eligible { lever_hint }, Denied { reason } }`.
- `policy::evaluate(holder, unit_meta, turn_state, cfg) -> Decision`:
  default `Denied`. Promote to `Eligible` ONLY when ALL hold:
  1. class == `wintermute-daemon` AND has a resolvable systemd unit;
  2. `Restart=always` (or `on-failure` with a drop-in) per
     `systemctl --user show -p Restart`;
  3. swap_kb ≥ `cfg.swap_floor_kb` (default 64 MiB) OR rss_kb ≥
     `cfg.rss_floor_kb`;
  4. holder is idle: recent CPU below `cfg.idle_cpu_pct` over a window;
  5. NOT in the live conversational path while a voice TURN is active.
- **Hard exclusions (non-overridable):** any `user-app`, `build`,
  `local-llm`, `system` class; any unit not under `user.slice`; `wmd`
  / `wm-dialog` while turn-state is active. Config can *tighten* (raise
  floors, add excludes) but never *loosen* these.
- `policy::Config` loaded from
  `~/.config/trim/policy.toml` (optional; sane defaults if absent).

**CLI:**
- `trim policy` → table of every holder with its `Decision` and reason.
- `trim policy --eligible` → only the eligible set (what relief would
  consider).
- `--format json`.

## Acceptance criteria

1. `policy::evaluate` returns `Denied` by default; a holder is
   `Eligible` only when class==`wintermute-daemon`, a unit resolves,
   `Restart=always`, over the swap/rss floor, and idle — verified by a
   table-driven test exercising each failing condition individually.
2. A `user-app`, `build`, `local-llm`, or `system` holder is ALWAYS
   `Denied`, even with a config that attempts to allowlist it (the
   exclusion is hard-coded, not config-driven). Test the override
   attempt explicitly.
3. With a fixture turn-state marking a voice TURN active, `wmd` and
   `wm-dialog` evaluate to `Denied { reason: "live conversational
   turn" }`; with turn-state idle and Restart=always, they may be
   eligible.
4. A unit with `Restart=no` is `Denied { reason: "not self-healing" }`
   regardless of swap held.
5. `trim policy --eligible` on this box surfaces only
   wintermute-daemon units (e.g. `homeward-embed.service` when idle &
   over floor) and never `firefox`/`slack`/`rustc`.
6. Missing `~/.config/trim/policy.toml` is not an error — defaults
   apply; a present config can raise floors / add excludes but a unit
   test proves it cannot remove a hard exclusion.
7. No relief is performed in this PRD — `trim policy` is read-only
   (it may call `systemctl --user show` to read properties, never
   `start`/`stop`/`restart`/`set-property`). `cargo test` green; build
   via /cloudbuild.
