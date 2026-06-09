# PRD: the-lunch-standing-invitation — wire the noon cadence (timer + hook)

**Status:** Draft v0.1
**build_target:** mixed
build_priority: high
**build_into:** (new repo) `~/wintermute/the-lunch`
**Vision:** visions/the-lunch.md

## TL;DR

The convene → seat → menu → minutes chain exists but only runs when someone
types the commands — which means it never runs, the same write-only trap the
soloists are in. The Round Table's defining feature was that lunch was a
*standing engagement*: no one was invited, everyone just came at noon. This PRD
ships that standing invitation — a systemd-user timer at 12:00 that drives the
full chain, plus a SessionStart-style hook that surfaces today's table when a
Claude session opens — so the lunch happens without being asked.

## Why this exists

`visions/the-lunch.md` makes cadence the whole point: "one table per day" is a
*ritual*, and a ritual needs a clock, not a person. This laptop already runs
the exact pattern this PRD reuses: `~/.config/systemd/user/daily-receipt.timer`
(`OnCalendar=*-*-* 21:30:00`, `Persistent=true`, `Unit=daily-receipt.service`)
prints a fixed daily record, and `claude-dream.timer` fires a Claude command on
a schedule. `the-lunch-standing-invitation` extends that proven systemd-user
timer pattern to noon and wires it to the convene chain built by the other four
PRDs. The SessionStart hook mirrors the existing `/build` and `/self-review`
"the X is due" banners so a person opening a session *sees* that the table
convened, closing the write-only loop the umbrella `roundtable.md` calls out.

This PRD ships last in the vision's order — there is nothing to wire until the
convene/seat/menu/minutes chain exists.

## What this builds

Ships into `~/wintermute/the-lunch` (the binary built by the prior PRDs is the
thing the units invoke). `build_target: mixed` — shell installer + config units;
no new crate, but it adds a `the-lunch lunch` driver subcommand to the existing
Rust binary so the timer has a single entrypoint.

- Rust: a `lunch` subcommand on the `the-lunch` binary that runs the full chain
  for today in order — `convene` → `seat` → `menu` → `minutes open` — each step
  idempotent (re-running at/after noon refreshes, never duplicates, per the
  vision's idempotency decision). Exit non-zero with a clear message if a step
  fails; partial progress is persisted (the table survives even if menu fails).
- systemd-user units under `units/`:
  - `the-lunch.service` — `Type=oneshot`,
    `ExecStart=%h/.local/bin/the-lunch lunch`.
  - `the-lunch.timer` — `OnCalendar=*-*-* 12:00:00`, `Persistent=true`,
    `Unit=the-lunch.service`, `WantedBy=timers.target` (noon, after the Round
    Table's real cadence; `Persistent` catches a lunch missed while asleep).
- Hook: a `hooks/the-lunch-sessionstart.sh` that, on SessionStart, checks
  whether today's table convened and prints a one-line banner
  ("the table convened at noon — N dishes, M seated; `the-lunch minutes show`")
  or, if it's past noon and no table exists, a gentle "no lunch today yet" note.
  Reads only the persisted `$XDG_STATE_HOME/the-lunch/<today>/` files; no
  Claude/API call.
- `install.sh` (the wiring entrypoint): installs the units into
  `~/.config/systemd/user/`, runs `systemctl --user daemon-reload`, enables and
  starts `the-lunch.timer`, and prints how to register the SessionStart hook in
  `~/.claude/settings.json` (it prints the JSON snippet; it does **not** edit
  settings.json unprompted). Idempotent: re-running re-syncs units without
  duplicating timer entries.
- README section documenting the cadence, how to disable
  (`systemctl --user disable --now the-lunch.timer`), and the on-demand override
  (`the-lunch convene --now` / `the-lunch lunch`).

Deps: clap subcommand reuses prior crates; shell uses only `systemctl`,
`jq`-free pure-shell JSON probing (or a `the-lunch table --json` read) for the
banner.

## Acceptance criteria

1. `the-lunch lunch` runs convene → seat → menu → minutes-open in order for
   today and exits 0 on a healthy run; an integration test (temp
   `XDG_STATE_HOME`) asserts all four `<date>/*.json[l]` files exist after.
2. `the-lunch lunch` is idempotent: a second run on the same day refreshes the
   table and leaves exactly one table/seating/menu and an intact (not wiped)
   minutes transcript — the vision's one-table-per-day rule holds.
3. If a middle step fails, earlier persisted artifacts survive (test: force
   menu to fail, assert `table.json` and `seating.json` still present) and the
   command exits non-zero with a message naming the failed step.
4. `the-lunch.timer` parses under `systemd-analyze verify units/the-lunch.timer`
   (or `systemctl --user verify`) with no errors and has
   `OnCalendar=*-*-* 12:00:00` and `Persistent=true`.
5. `the-lunch.service` is `Type=oneshot` and its `ExecStart` invokes
   `%h/.local/bin/the-lunch lunch`.
6. `install.sh` is idempotent: running it twice leaves a single enabled
   `the-lunch.timer` and re-synced units (test/dry-run asserts no duplicate unit
   files and `daemon-reload` invoked); it does **not** modify
   `~/.claude/settings.json` — it only prints the hook snippet.
7. `hooks/the-lunch-sessionstart.sh` prints a one-line banner when today's table
   exists (dish + seat counts) and a distinct "no lunch yet" line otherwise;
   it makes no network/API call and reads only the day's state dir
   (verified by running it against a fixture state dir, offline).
8. README documents enabling, disabling, and the `--now` / `the-lunch lunch`
   on-demand override; `install.sh` prints the exact
   `~/.claude/settings.json` SessionStart hook snippet to paste.
9. The `lunch` subcommand and `install.sh` shellcheck-clean (shell parts) and
   the Rust parts `cargo build`/`cargo test` green on toolchain 1.85.
