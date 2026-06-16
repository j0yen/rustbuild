# PRD: roundtable-cadence — the standing invitation, for the whole table

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/roundtable
Vision: visions/roundtable.md
Depends-on: roundtable-session, roundtable-games, roundtable-bind

## TL;DR

`the-lunch` ships a noon timer — but it only fires `the-lunch lunch`, which sets
the table and stops. Now that `roundtable session --with-games` runs the whole
gathering and `roundtable bind` binds the periodical, the cadence must fire
*those*, not the convening half alone. This PRD ships the standing invitation: a
`roundtable.timer` at noon firing `roundtable session --with-games`, a weekly
`roundtable-bind.timer` firing `roundtable bind`, and an `install.sh` that wires
them and disables the now-subsumed `the-lunch.timer` so the table convenes once,
not twice.

## Why this exists

Verified 2026-06-15 by reading `~/wintermute/the-lunch/install.sh` and
`the-lunch.timer`: the shipped timer fires daily at noon (`Persistent=true`) and
runs `the-lunch lunch`. Because `roundtable session` calls `the-lunch lunch`
internally as its first stage, leaving `the-lunch.timer` enabled alongside a
`roundtable.timer` would convene the table twice at noon. The cadence layer must
supersede, not stack. The 30-min `claude-dream.timer` and `the-lunch.timer`
already establish the systemd-user timer pattern this reuses; no new mechanism is
invented.

## What this builds

Extend `~/wintermute/roundtable`:

- **`units/roundtable.timer` + `units/roundtable.service`** — `OnCalendar`
  daily at noon (local), `Persistent=true`, `ExecStart=%h/.local/bin/roundtable
  session --with-games`. Type=oneshot.
- **`units/roundtable-bind.timer` + `units/roundtable-bind.service`** —
  `OnCalendar` weekly (default Monday), `Persistent=true`,
  `ExecStart=%h/.local/bin/roundtable bind`. Type=oneshot.
- **`install.sh`** (idempotent, mirroring `the-lunch/install.sh`):
  - Build + install the `roundtable` binary to `~/.local/bin/`.
  - Copy the four unit files to `~/.config/systemd/user/`, `daemon-reload`,
    `enable --now` both timers.
  - **Disable the subsumed timer**: if `the-lunch.timer` is enabled, run
    `systemctl --user disable --now the-lunch.timer` and print a one-line note
    explaining that `roundtable session` now convenes the table (so the lunch
    still happens, just via the fuller chain). Idempotent: a no-op if already
    disabled.
  - Print the disable/enable summary and the "run now" hints
    (`roundtable session --with-games`, `roundtable bind`).
- No `~/.claude/settings.json` edits in this PRD (the digest hook is
  roundtable-digest's job); install.sh only touches `~/.local/bin` and
  `~/.config/systemd/user`.

MSRV 1.85 for any binary touch (none expected; the session/games/bind commands
already exist). This PRD is unit files + install plumbing.

## Acceptance criteria

1. `roundtable session --with-games` and `roundtable bind` exist and run (built
   by the prior PRDs); `cargo test --release` + `cargo clippy -- -D warnings`
   stay green in the repo.
2. `install.sh` is idempotent: running it twice produces no duplicate units and
   the second run reports already-installed (a test or a `--dry-run`/`--check`
   mode asserts this without requiring an active systemd in CI).
3. The shipped `roundtable.timer` parses (`systemd-analyze verify
   units/roundtable.timer units/roundtable.service` exits 0) and fires
   `roundtable session --with-games`.
4. The shipped `roundtable-bind.timer` parses and fires `roundtable bind`.
5. `install.sh` disables `the-lunch.timer` when present and enabled, and says so;
   when `the-lunch.timer` is absent/already-disabled it is a clean no-op (test
   the detection logic without mutating the live system — e.g. a `--dry-run`
   that prints the intended `systemctl` calls).
6. `install.sh --dry-run` (or equivalent) prints every `systemctl`/`cp`/`install`
   action it would take and mutates nothing — a test asserts no writes under it.
