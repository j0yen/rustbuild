# PRD: ballast-pilot — wire the disk guard to a cadence so 96% never recurs

**Status:** Draft v0.1
**build_target:** shell
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-pilot (NEVER AtScaleInc)
**Depends on:** PRD-ballast-contract-repair (guard must complete a pass first)

## TL;DR

The ballast vision's fourth end-state — "the disk defends a high/low-water SLO
autonomously" — is unfulfilled because **nothing ever runs `ballast-guard`**.
There is no `claude-ballast.timer`, and there is no `~/.config/ballast/guard.toml`
for the guard to read (both verified absent 2026-06-16). The guard is a watcher
that no one wound up. ballast-pilot ships the missing harness: a default
`guard.toml` encoding the vision's water marks, a systemd-user service that runs
one `ballast-guard run` pass with a JSONL event-sink, and a timer that fires it
on a cadence — defaulting to **dry-run/report-only** so the first deployment
observes before it ever deletes, exactly like every other gated loop on this box.

## Why this exists

The disk climbed unattended **86% → 92% → 96%** over 2026-06-14/15/16 (three
consecutive self-review journals), each one printing a manual "suggest `du -sh
~/wintermute/*/target`" note for a human to act on by hand. Meanwhile:

- `systemctl --user list-timers | grep ballast` → **empty** (verified).
- `ls ~/.config/systemd/user/ | grep ballast` → **empty** (verified).
- `cat ~/.config/ballast/guard.toml` → **No such file or directory** (verified);
  `ballast-guard run` falls back to defaults with nowhere to tune water marks.

The vision's own open question already specified the policy ("advisory at 85%,
reap at 90%, target 80% — but these belong in a config file, not hard-coded").
That config and its scheduler are the only things standing between a shipped
guard and an SLO that actually self-corrects. The drydock fleet deliberately
shipped its `apply` step dry-run-default and **not** cron-wired pending jsy's
opt-in (gossip 2026-06-16); ballast-pilot mirrors that discipline — it schedules
the *observation* loop autonomously and leaves the *reaping* opt-in explicit.

## What this builds

A small shell/config package installed under `~/.config/` and
`~/.config/systemd/user/`, plus an idempotent `install.sh`.

**1. Default `guard.toml`** at `~/.config/ballast/guard.toml`: high-water 90,
low-water 80, advisory 85, scan roots = `~/wintermute`, safety floor = `fossil`,
and a top-level `mode = "report"` (dry-run) so a fresh install never deletes
until jsy flips it to `enforce`. Ship it only if absent — never clobber an
existing tuned config.

**2. `ballast-guard.service`** (Type=oneshot, user): runs `ballast-guard run
--mount / --event-sink ~/.local/state/ballast/guard-events.jsonl`, honoring the
config. Records each pass's exit code and SLO band to the event log.

**3. `ballast-guard.timer`** (user): `OnCalendar` cadence (default hourly, e.g.
`*:00`), `Persistent=true` so a missed window after sleep still fires.
`WantedBy=timers.target`.

**4. `install.sh`**: idempotent — writes the units + default config if absent,
`systemctl --user daemon-reload`, enables the timer, prints next-fire time.
Safe to re-run. A matching `uninstall.sh` disables and removes the units.

**5. Report-vs-enforce gate.** In `report` mode the service runs the guard with
the dry-run posture (measure + emit events, no `--apply` to reap). Flipping
`mode = "enforce"` in `guard.toml` is the single, documented, jsy-owned switch
that turns on autonomous reaping. The installer never sets `enforce`.

## Acceptance criteria

1. `./install.sh` is idempotent: a second run makes no further changes and exits
   0; it never overwrites a pre-existing `guard.toml`.
2. After install, `systemctl --user list-timers` shows `ballast-guard.timer`
   with a future next-fire time.
3. `systemctl --user start ballast-guard.service` runs one guard pass and
   appends at least one JSON event line to
   `~/.local/state/ballast/guard-events.jsonl` containing the usage percent and
   SLO band/exit code.
4. The shipped default `guard.toml` sets `mode = "report"`; in this mode the
   service performs no deletion (verified: a guard pass over a deliberately
   over-high-water test mount emits a breach event but the candidate paths still
   exist afterward).
5. `./uninstall.sh` disables and removes the timer and service and reloads the
   user daemon; `guard.toml` and the event log are left intact.
6. The package hard-codes no secrets, no absolute home path other than via
   `$HOME`/`%h`, and never calls `ballast-reap --apply` directly — reaping is
   reached only through the guard under `mode = "enforce"`.

## Out of scope

- Flipping the box into `enforce` mode (jsy's explicit, one-line opt-in).
- Any change to guard/reap binaries themselves.
- Notification transport for breach events — the event sink is a file; a
  notifier can compose it later (vision keeps transport uncoupled).
