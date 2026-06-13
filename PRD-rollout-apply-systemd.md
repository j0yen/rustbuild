# PRD: rollout-apply-systemd — restart systemd-managed daemons the systemd way, not by racing it with SIGTERM

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/rollout
Vision: visions/vigil.md

## TL;DR

`rollout apply` brings a stale daemon current by SIGTERM-ing the old PID
and running the recipe's `launch_cmd`. That model is correct for a
hand-launched daemon (`agorabus serve &`) but **wrong for the live
fleet**, which is systemd-managed: `wm-audio|dialog|stt|tts.service` all
have `Restart=always` drop-ins. A manual SIGTERM *races systemd's own
auto-restart*, and a manual `launch_cmd` spawns a process systemd does
not track — leaving a duplicate the unit will fight. This PRD teaches the
`apply` path to honour a recipe's `unit` field and restart through
`systemctl --user restart <unit>` — the exact path `rollout install`
already owns — so `apply` finally matches how the fleet is actually run.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The restart model mismatches the fleet.** `restart.rs:restart_daemon`
  is documented as: build → install → `SIGTERM old_pid` → wait/grace →
  SIGKILL fallback → `launch_cmd` → healthcheck. Every live voice daemon
  is a systemd-user unit (`systemctl --user list-units 'wm-*'` shows
  `wm-audio|dialog|stt|tts.service` all `active running`).
- **Restart=always makes the SIGTERM a race.** Per
  [[self_agorabus_restart_kills_voice]], the wm- daemons carry
  `Restart=always` drop-ins (added 2026-06-05 to self-heal after an
  agorabus bounce). When `rollout apply` SIGTERMs the old pid, systemd
  *also* restarts the unit — then `launch_cmd` starts a third copy.
  Three actors, one daemon.
- **The correct path already exists in the same crate.** `install.rs`
  (Fleet 4's `rollout install`) does exactly the right thing: reverse
  unit-map from dest → `systemctl --user restart <unit>` (special-casing
  `agorabus reload`), then verifies the unit is active and re-registered.
  The `apply` path simply never learned it. This PRD is mostly *reuse*,
  not new logic.
- **fleet-gen emits the `unit` field.** The sibling PRD
  rollout-fleet-gen writes a `unit = "wm-audio.service"` into each
  recipe; this PRD is what consumes it. Without this PRD, that field is
  dead.

## What this builds

In `~/wintermute/rollout/` (`restart.rs`, reusing `install.rs` helpers):

1. **Honour `DaemonRecipe.unit`.** When a recipe carries a non-empty
   `unit`, `restart_daemon` takes a **systemd branch**: still run
   `build_cmd` + `install_cmd` (to put fresh bytes on disk), then instead
   of SIGTERM+launch, call the shared `restart_unit(unit)` from
   `install.rs` (`systemctl --user restart <unit>`, with the
   `agorabus reload` special-case preserved). Then run the same
   healthcheck poll. No manual SIGTERM, no `launch_cmd`, no SIGKILL
   fallback on the systemd branch.
2. **Keep the legacy branch.** When `unit` is absent/empty, behaviour is
   unchanged (SIGTERM + `launch_cmd`) — non-systemd daemons still work.
3. **Refactor for reuse.** Extract `install.rs`'s `restart_unit` /
   `find_unit_for_dest` into a shared `systemd.rs` (or `pub(crate)`) so
   both `install` and `apply` call one implementation; do not copy-paste.
4. **Report the path taken.** `RestartResult` gains a `restart_path`
   field (`"systemd-unit:<name>" | "agorabus-reload" | "sigterm-launch"`)
   so a rollout run is auditable about *how* each daemon was bounced.

The healthcheck (agorabus peers re-registration poll) is unchanged and
runs on both branches — a systemd restart still has to prove the daemon
came back on the bus before the run proceeds to the next daemon.

## Acceptance criteria

1. A `DaemonRecipe` with `unit = "wm-tts.service"` causes
   `restart_daemon` to invoke `systemctl --user restart wm-tts.service`
   and **not** send SIGTERM to the old pid or run `launch_cmd`. Verify
   with a unit test that injects a command-runner spy and asserts the
   systemctl call + absence of a kill/launch call.
2. A `DaemonRecipe` with no `unit` field restarts via the existing
   SIGTERM + `launch_cmd` path (a regression test pins the legacy
   behaviour).
3. The `agorabus` recipe (unit `agorabus.service`) routes through the
   `agorabus reload` special-case, identical to `rollout install`'s
   behaviour — one code path, asserted by a shared-helper test.
4. `restart_unit`/`find_unit_for_dest` exist in exactly one module and
   are called by both `install` and `apply` (no duplicated unit-scan
   code remains in `install.rs`).
5. `RestartResult` serialises a `restart_path` field reflecting the
   branch taken; `rollout apply` output (JSON or table) shows it.
6. On the systemd branch, the healthcheck/re-registration poll still
   runs and a failure to re-register still stops the run non-zero (the
   serialized one-at-a-time guarantee is preserved).
7. `cargo build --release` + `cargo test` pass; clippy `-D warnings`
   clean for changed modules (crate baseline permitting).

## Build note for /build

This PRD and **rollout-window-guard-turnaware** both modify the rollout
crate. **Serialize** their /build cycles (or worktree-isolate) to avoid
`Cargo`/`restart.rs`/`health.rs` rebase churn — same caution the vigil
vision raised for its other same-crate fleets.
