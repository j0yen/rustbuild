# PRD-carbon-reminders-cloud

**Status:** Draft v0.1
**Vision:** visions/carbon.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-carbon-hub-access (a working hub session), PRD-carbon-node-identity

## TL;DR

The scheduled "reminders" — node-agnostic systemd timers — only fire while this
laptop is awake. Close the lid and `roundtable`, `claude-review-due`,
`adopt-cron`, `consign-drain`, and friends silently stop. This PRD relocates the
node-agnostic timers to the always-on hub so they fire on schedule regardless of
which laptop is asleep, while leaving genuinely node-local timers
(`cloudbuild-watchdog`, `ctrace-reap`) where they belong.

## Why this exists

**Evidence (2026-06-20):**
- `systemctl --user list-timers` on the laptop shows node-agnostic scheduled
  jobs: `roundtable`, `roundtable-bind`, `claude-self-review`,
  `claude-review-due`, `adopt-cron`, `consign-drain`, `claude-chaff`,
  `trim-relief`, `ballast-guard`. All are bound to this laptop's uptime.
- These are "reminders" in the user's framing: periodic tasks that should fire on
  wall-clock time, not on "is the laptop open." A nightly self-review that only
  runs when the laptop happens to be on at 3am is unreliable by construction.
- Some timers ARE node-local and must NOT move: `cloudbuild-watchdog` (kills
  stale build servers — only meaningful where builds launch), `ctrace-reap`
  (reaps local tracer state). Classification matters.

## What this builds

Runs on: **this laptop**, deploying timer units to the **hub**.

1. **Classify** every `claude-*` / app timer as `fleet` (fire once, anywhere) or
   `node-local` (fire per node). Encode the classification in
   `~/.config/wintermute/placement.toml` (consumed by `wm-node should-run`).
2. **Relocate fleet timers** to the hub: copy the `.timer` + `.service` units,
   adjust any laptop-specific paths, enable on the hub. The job binaries they
   call must be present on the hub (ARM builds via carbon-hub-access).
3. **Disable the relocated timers on the laptop** (so a fleet job does not
   double-fire). Each relocated unit gets an `ExecCondition=wm-node role hub` as
   a belt-and-suspenders guard.
4. **Leave node-local timers** running on every node unchanged.
5. Document the placement table in `constellation/docs/` so the next node added
   to the fleet inherits the same split.

## Acceptance criteria

1. `placement.toml` classifies every current `claude-*`/app timer as `fleet` or
   `node-local`, with no timer unclassified.
2. At least the nightly `claude-self-review` (or its fleet-appropriate
   equivalent) runs on the hub: `ssh hub 'systemctl --user is-enabled
   claude-self-review.timer'` → `enabled`.
3. Relocated fleet timers are **disabled on the laptop**
   (`systemctl --user is-enabled` → `disabled`/`masked`) — no double-fire.
4. Node-local timers (`cloudbuild-watchdog`, `ctrace-reap`) remain `enabled` on
   the laptop and are NOT present on the hub.
5. A relocated timer actually fires on the hub on schedule (verified via
   `journalctl --user -u <unit>` on the hub showing a post-relocation run), or
   its `ExecCondition` correctly gates it.
