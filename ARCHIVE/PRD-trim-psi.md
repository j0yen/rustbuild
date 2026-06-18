# PRD: trim-psi — warn before the box thrashes

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/trim
Vision: visions/trim.md

## TL;DR

Relief is reactive — it acts on pages already cold in swap. trim-psi is
the early-warning: it tails `/proc/pressure/memory` (and per-cgroup
`memory.pressure`) plus the swap-in rate, and when sustained pressure
crosses a threshold it emits an `agorabus` event (`trim.pressure`) so
the fleet can see the stall *coming*. It NEVER auto-acts — relief stays
behind policy. It is the memory analogue of `pulse`'s deafness
early-warning. Extends the `trim` crate.

## Why this exists

Live evidence (this laptop, 2026-06-18):

- `/proc/pressure/memory` reads `some avg10=0.00 … total=364808468`
  and `full … total=337243465`. Right now `avg*` is calm, but the
  cumulative `total` proves the box has already spent **~365
  ms-seconds stalled on memory** — past thrash that nobody saw,
  because nothing watches PSI.
- Self-review only samples `free -h` once per run (every ~24h) and
  writes "swap heavy, monitor" — it cannot catch a 30-second pressure
  spike that stalls a live voice TURN, only the slow average after.
- `agorabus` is the established fleet event bus (12 peers live this
  session); `pulse` already proves the "emit a warning event before
  the bad thing fully manifests" pattern for voice deafness — trim-psi
  reuses it for memory.

## What this builds

Extends the `trim` library + CLI.

**Library:**
- `psi::watch(cfg, sink) -> !`: poll `/proc/pressure/memory` +
  configured cgroup `memory.pressure` files on an interval; compute
  swap-in delta from `/proc/vmstat` `pswpin` between polls.
- `psi::Trigger { kind: SomeAvg60 | FullAvg60 | SwapInRate, value,
  threshold }`: fires when a metric stays over threshold for
  `cfg.sustain_secs` (debounced — a single spike does not fire).
- `event::PressureEvent { ts, trigger, top_holders: Vec<(unit,
  swap_kb)> }` — includes the current top swap holders from
  trim-survey so the consumer knows *who* without a second query.
- Emit via agorabus (`trim.pressure` subject); degrade to stderr +
  a state file if the bus is unreachable (never crash on bus-down,
  per `self_agorabus_restart_kills_voice`).

**CLI:**
- `trim psi` → one-shot: print current PSI + swap-in rate + whether
  any threshold is currently tripped.
- `trim psi --watch` → long-running watcher loop (the form
  trim-cron / a future unit invokes); emits events on trigger.
- `--threshold-some-avg60 <pct>` / `--sustain-secs <n>` overrides.

## Acceptance criteria

1. `trim psi` (one-shot) prints `some`/`full avg10/avg60/total` from
   `/proc/pressure/memory` and the swap-in rate derived from two
   `/proc/vmstat` `pswpin` samples.
2. A `Trigger` fires only after a metric stays above threshold for
   `sustain_secs`; a single over-threshold sample followed by an
   under-threshold one does NOT fire (debounce verified by a
   fixture-driven sample sequence).
3. A fired trigger produces a `PressureEvent` whose `top_holders` is
   populated from the survey (non-empty when swap is in use), so the
   event names the likely cause.
4. The watcher emits on the agorabus `trim.pressure` subject; with the
   bus unreachable it falls back to stderr + a state file and keeps
   running (no panic, no exit) — asserted with a stub sink.
5. trim-psi performs NO relief: it issues no `systemctl` calls and no
   signals; it only reads `/proc` and emits events. Verified by a
   command-recorder shim asserting zero mutating calls.
6. `trim psi --watch` is interruptible (SIGTERM/SIGINT exits cleanly,
   flushing any pending state file) so it is safe under a systemd
   unit. (AC may be marked deferred/mocked if a live long-run loop is
   not exercisable in CI; the debounce + emit logic must still be
   unit-tested.)
7. `cargo test` green: fixture tests for PSI parsing, swap-in delta,
   debounce, and event shape. Build via /cloudbuild.
