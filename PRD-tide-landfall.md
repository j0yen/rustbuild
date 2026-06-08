# PRD: tide-landfall

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/tide
Vision: visions/tide.md

## TL;DR

After a reboot crossing, something has to confirm it *worked* — the kernel
advanced as `tide survey` predicted, the fleet came back, watchman roots
re-asserted. Today nothing does: self-review re-discovers brokenness ad hoc each
run, and `anchor` notes that watchman silently drops every watched root on
reboot. `tide landfall` is the post-reboot verifier: it consumes survey's
pre-reboot expected-state, checks the kernel and the fleet on the other side,
and writes a timestamped **crossing receipt** with before/after. On a failed
crossing it escalates loudly instead of leaving the gap open.

## Why this exists

- self-review keeps re-discovering post-reboot fleet state by hand. The
  2026-06-07 review (fresh-reboot run, uptime 3 min) had to *re-watch*
  `~/.claude` + `~/brain` because "watchman dropped roots on reboot" — exactly
  the silent-drop `anchor.md` describes. Nothing attested the crossing; the
  breakage was found, not verified-against.
- The fleet must return whole after a reboot: agorabus (`doctor=current`, 14
  peers, voice fleet, no orphans — the 06-07 review confirmed this *manually*),
  the voice daemons (wm-stt/tts/dialog/brain), recalld (safety-critical liveness,
  [[project_brain_local_first_ladder]]). [[self_agorabus_restart_kills_voice]]
  is a logged case where a bus bounce silently killed voice until drop-ins fixed
  it — a reboot is a bus bounce, so a verifier is warranted.
- `survey` already computes the pre-reboot expected next-kernel (`available_pkg`
  = `7.0.11` in the live skew). landfall is the natural consumer: it closes the
  loop by asserting `booted == expected` after the crossing and recording the
  delta.
- This is the piece that makes the whole tide lifecycle *accountable*: survey →
  window propose the crossing; landfall proves it happened correctly or says
  loudly that it didn't.

## What this builds

A rust-extend into `~/wintermute/tide` (after `tide-survey` ships; preserve
survey/restart/window modules). Add:

### Modules

- `expected.rs` — read/write a small pre-reboot **expectation file** under
  `$XDG_STATE_HOME/tide/expected.json` capturing survey's `KernelSkew.available`
  + the queue's reboot-requiring set + a `recorded_boot_id`
  (`/proc/sys/kernel/random/boot_id`). `tide survey --record` (a flag added to
  survey's existing subcommand, the only survey touch) writes it; landfall reads
  it. If absent, landfall runs in best-effort mode (verifies fleet, skips the
  kernel-advanced assertion).
- `fleet.rs` — read-only fleet health: `agorabus doctor` status + peer count vs
  an expected baseline, voice-daemon presence (`wm-stt`/`tts`/`dialog`/`brain`
  bus peers or processes), recalld liveness (its health probe / socket), and a
  watchman-roots check (are `~/.claude` + `~/brain` currently watched — *verify*
  anchor's reconcile, don't re-watch).
- `landfall.rs` — compute `Crossing { boot_id, before: ExpectedState, after:
  FleetState, kernel_ok: bool, fleet_ok: bool, regressions: Vec<String> }`. The
  boot-id changing vs `recorded_boot_id` confirms a real reboot occurred.
- `receipt.rs` — append a timestamped crossing receipt under
  `$XDG_STATE_HOME/tide/crossings/<boot_id>.json` (deduped by boot-id) + a
  human one-line summary. Loud (non-zero exit / `WARN` lines) when `kernel_ok`
  or `fleet_ok` is false.
- `main.rs` (extend) — `tide landfall [--json]`; survey gains `--record`.

### Trigger (see vision open question)

Ship the binary subcommand only in this PRD. Wiring it to run automatically is a
**config follow-on** (a boot-time `systemd --user` oneshot and/or a SessionStart
hook, deduped by boot-id) — note it in gossip for a later `mixed` PRD rather
than baking a hook into this rust-extend.

### UX

```
$ tide landfall
crossing 9f3c… : kernel ok (7.0.10→7.0.11) · fleet ok (agorabus current/14, voice 4/4, recalld up)
$ # failed crossing:
$ tide landfall
WARN crossing 9f3c… : kernel ok · fleet DEGRADED
  - agorabus doctor=stale (expected current)
  - voice daemons 2/4 (wm-tts, wm-brain missing)
  - watchman roots NOT re-asserted (~/.claude, ~/brain unwatched)
```

### Dependencies

Reuses survey deps. Read-only `std::process::Command` to `agorabus`, `pgrep`/
bus, `watchman` (or `wchg`); `/proc` + `$XDG_STATE_HOME` via `std::fs`.

## Acceptance criteria

1. `tide landfall` is a working subcommand after a clean rust-extend build;
   survey/restart/window subcommands and tests still pass; survey gains a
   `--record` flag that writes the expectation file.
2. The expectation round-trip is unit-tested: `--record` writes
   `expected.json` with the available kernel, reboot-requiring set, and a
   boot-id; landfall reads it back and compares.
3. `kernel_ok` is true iff the booted `uname -r` reflects the recorded
   `available` kernel; a fixture where booted still equals the *old* kernel
   yields `kernel_ok: false` with a regression line. Tested with injected
   uname/expectation values.
4. A boot-id equal to `recorded_boot_id` (no reboot actually happened) is
   reported distinctly ("no crossing detected — boot-id unchanged"), not as a
   success. Tested.
5. Fleet health is assembled from injected `agorabus doctor`/peer output, voice-
   daemon presence, recalld liveness, and watchman-roots state; a fixture with a
   missing voice daemon or unwatched root yields `fleet_ok: false` and the
   specific regression strings. The watchman check *reports* an unwatched root;
   it does not re-watch (no mutation — asserted).
6. A failed crossing (`kernel_ok` or `fleet_ok` false) exits non-zero and prints
   `WARN`/regression lines; a clean crossing exits zero. Both tested.
7. Receipts are written under `$XDG_STATE_HOME/tide/crossings/<boot_id>.json`,
   deduped by boot-id (a second run for the same boot-id updates, doesn't
   duplicate), and `tide landfall --json | head -1` does not SIGPIPE-panic.
8. README documents landfall, the expectation file, the boot-id reboot-detection,
   the "verify anchor, don't re-watch" boundary, and that auto-trigger wiring is
   a deferred config follow-on.
