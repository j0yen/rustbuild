# PRD: coda-boot — the trigger that closes debt without a human

**Status:** Draft v0.1
**build_target:** mixed
**build_into:** `~/wintermute/coda`
**Vision:** visions/coda.md
**deferred_acs:** [5, 6]

## TL;DR

coda-close can close the debt, but only when something runs it — and today
that something is a human running self-review. **coda-boot** is the trigger:
(1) a systemd-user timer that runs `coda close --apply` on a cadence, and
(2) a SessionStart hook that closes the *previous* session's debt — the
reliable repair point, because the prior headless tick was SIGKILLed before
its own SessionEnd hook could render it. The installer **prints** the unit and
the hook entry for the user to place; it never auto-enables a timer or edits
`~/.claude`. With coda-boot live, the 33% summary gap closes itself and
self-review's per-run backfill toil disappears.

## Why this exists

- **SessionEnd is the wrong place to fix this.** Headless `/build`, `/dream`,
  `/self-review` ticks are SIGKILLed by cgroup teardown *before* their
  SessionEnd hook runs (`ctrace-scribe/README.md`; confirmed live — today's
  `…T230000.ndjson` died mid-`execve`, no summary). The fix cannot live in the
  dying session; it must run *after*, from a survivor.
- **The next session start is the reliable survivor.** Every SIGKILLed log is
  closed-and-stable by the time the *next* session begins — so a SessionStart
  hook that runs `coda close --apply --limit N` reliably renders the previous
  tick's orphan. The 06-03 journal proposed exactly this shape for the sibling
  watchman problem (*"a SessionStart re-watch hook if it recurs"*); coda-boot
  is that pattern applied to summary debt.
- **A timer covers the idle gap.** Between sessions (overnight, or a long
  quiet stretch) a low-frequency `coda close --apply` timer keeps the backlog
  from accumulating even when no new session starts.
- **The toil it removes is recurring.** self-review runs `scribe backfill` by
  hand, inconsistently (50/2/0 across 06-03/02/01). coda-boot makes that
  automatic; self-review then only *reads* `coda audit`.
- coda-close has shipped: coda-boot wires its `--apply` path into a unit + a
  hook (the rust-extend/mixed rule — the actuator must exist first).

## What this builds

Extends the `coda` repo with an `install/` dir and a thin `coda boot` helper;
the binary itself already exists from coda-sweep/audit/close.

**The systemd-user timer + service** (`install/coda-close.{service,timer}`):

- `coda-close.service` — `Type=oneshot`, `ExecStart=%h/.local/bin/coda close
  --apply --limit 200`. Runs as the user; no root. Writes only `*.summary.md`
  via `scribe`.
- `coda-close.timer` — `OnCalendar` a few times a day (e.g. `*-*-* 07,13,19:45`)
  plus `Persistent=true` so a missed fire (laptop asleep) runs on resume.
  `WantedBy=timers.target`.
- The unit is shipped as a file to *install*, not auto-enabled. `coda boot
  install` copies it to `~/.config/systemd/user/` only when run with an
  explicit `--enable` flag *and* prints the `systemctl --user enable --now
  coda-close.timer` line otherwise (timer-enable is an ongoing-effect the user
  gates — `feedback_classifier_per_command`).

**The SessionStart hook** (`install/coda-session-start.sh`):

- a fast, non-blocking shell snippet: `coda close --apply --limit 50 >/dev/null
  2>&1 &` (background, capped, so it never delays session start) — renders the
  previous tick's orphan opportunistically.
- **always exits 0**, even if `coda` errors or is absent — a SessionStart hook
  must never block a session from starting (the contract every existing hook in
  `settings.json` follows).

**The installer** (`coda boot install`):

- prints the exact `settings.json` `SessionStart` hook entry to add (a JSON
  snippet), and the unit-install + enable commands — it does **not** edit
  `~/.claude/settings.json` (settings edits are user-gated —
  `feedback_classifier_per_command`).
- `--enable` (explicit) copies the unit files into
  `~/.config/systemd/user/` and runs `systemctl --user daemon-reload`; even
  then it prints (not runs) the final `enable --now` unless `--enable` is
  paired with a second explicit `--start`.
- `coda boot status` reports whether the unit is installed/enabled and whether
  the hook line is present in `settings.json` (read-only check).

**Offline validation.** `systemd-analyze verify install/coda-close.service`
and `…timer` are the offline gate (the cloud box has systemd-analyze but no
user session bus). The hook script is `shellcheck`-clean and has a test that
asserts it exits 0 even when `coda` is not on PATH.

**Deps:** no new Rust deps; `coda boot` is argument-parsing + file copy +
printing. MSRV 1.85, no let-chains. `sigpipe::reset()` in `main`.

## Acceptance criteria

1. `cargo build` / `cargo test` succeed offline; a `tests/boot.rs` entry file
   appears in cargo test output (`self_orphaned_mock_tests` guard).
2. `coda boot install` (no flag) **prints** the `settings.json` SessionStart
   hook entry and the unit-install/enable commands and **does not** write to
   `~/.config/systemd/user/` or `~/.claude/` — an integration test runs it
   against temp `$HOME`/`$XDG_CONFIG_HOME` and asserts no files were created.
3. `coda boot install --enable` copies `coda-close.service` + `.timer` into
   `$XDG_CONFIG_HOME/systemd/user/` (temp dir in the test) and still **prints**
   (does not run) the `enable --now` line unless `--start` is also passed.
4. `systemd-analyze verify` passes for both shipped unit files (run in CI/
   offline); the `.service` `ExecStart` invokes `coda close --apply --limit`,
   the `.timer` carries `Persistent=true` and a `WantedBy=timers.target`.
5. **[deferred — laptop-only]** the SessionStart hook, once installed in
   `~/.claude/settings.json`, runs `coda close` in the background on a real
   session start and the previous tick's orphan is rendered within the session.
   (deferred_acs:[5] — needs a real Claude session + the live hook.)
6. **[deferred — laptop-only]** `coda-close.timer` enabled on the laptop fires
   and drives the real orphan count toward 0 across a day. (deferred_acs:[6] —
   needs the real timer + session backlog; the cloud box has neither.)
7. `install/coda-session-start.sh` is `shellcheck`-clean and a test asserts it
   exits 0 when `coda` is absent from `PATH` (never blocks session start).
8. `coda boot status` reports unit installed/enabled state and hook-present
   state as a read-only check; `coda boot status --json | head -1` does not
   panic (SIGPIPE).
9. README documents the install flow, why the SessionStart hook (not
   SessionEnd) is the repair point, and that the timer-enable + settings edit
   are user-gated steps the installer only prints.
