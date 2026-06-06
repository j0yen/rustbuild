# PRD: quicken-notify

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/quicken
Vision: visions/quicken.md

## TL;DR

`quicken-watch` puts primitive-liveness verdicts on the bus, but a
verdict on a topic that nobody reads is the same silence quicken exists
to end. `quicken-notify` is the report-side consumer: it subscribes to
`wm.health.primitive.`, watches for a **transition** (a primitive going
`live → dark`, or an inert-streak crossing a threshold), and fires
**one** surfaced signal — a SessionStart banner fragment and an optional
`peon-ping` — debounced so a steadily-dark primitive doesn't re-alarm
every boot. It reports the *moment* something went dark; it never heals
it. That report-not-heal line is the whole point.

## Why this exists

Phase-1 evidence (live, 2026-06-06):

- **Detection without surfacing is the failure quicken was built to
  fix.** `visions/quicken.md` TL;DR: the self-review "escalates 'once,'
  then goes silent — so they persist boot after boot." `quicken-watch`
  moves detection to real time, but unless a consumer turns a verdict
  *change* into a human-visible signal, real-time detection just relocates
  the silence onto a bus topic.
- **The transition is the high-value event.** The verbatim live set this
  pass — memlog group EACCES (pkgrel gap 5→11), agentns all-zeros,
  bpolicy `{"loaded": false}`, provfs degraded-because-agentns — are all
  *steady* dark states; alerting on them every boot is noise (exactly the
  escalate-every-tick anti-pattern). What carries information is the
  **edge**: voice was live, now it isn't (`self_agorabus_restart_kills_voice`);
  a streak just crossed N boots dark. Notify fires on edges, stays quiet
  on plateaus.
- **The surfacing channels already exist.** `agorabus subscribe`
  auto-reconnects across daemon restarts (`agorabus --help`: "Reconnects
  automatically on daemon restart") — the right primitive for a
  long-lived watcher. The SessionStart banner already renders multiple
  hook fragments (the session opened with `agorabus`, `letter`,
  `recall`, `learning-candidates` banners), so a quicken fragment slots
  into an established surface. `peon-ping` (`~/.claude/hooks/peon-ping/`)
  is the established audible-notification path on this laptop.
- **Report-vs-heal is an explicit, already-decided split.** quicken's
  open question "Self-heal vs. report" leans report-only; `homestead`
  owns any future unattended heal. `quicken-notify` is the concrete
  report end — building it does **not** pre-empt that decision, it
  honors it.

## What this builds

A `notify` subcommand on the existing `quicken` binary plus a small
persisted edge-state store.

- **`quicken notify --watch`**: long-lived; `agorabus subscribe
  wm.health.primitive.` and process each event against a persisted
  last-seen verdict per primitive (`~/.local/state/quicken/notify.json`
  or the repo's existing state dir convention). Fire on:
  (a) `live*/unknown → {inert, installed-not-activated,
  staged-not-installed}` (a darkening edge), or
  (b) `inert_streak` crossing a configurable threshold (default 3,
  matching the docket/self-review 3-run escalation convention). Stay
  silent on an unchanged dark verdict (debounce) and on a
  `dark → live` recovery unless `--notify-recovery` is set (recovery is
  good news; default quiet, opt-in to celebrate).
- **`quicken notify --once`**: drain currently-buffered events (or read
  the latest attest receipt) and emit any pending transition signals,
  then exit — the form the **SessionStart hook** calls so a banner
  fragment renders without a resident process.
- **Signal sinks**: (1) stdout one-line fragment suitable for a
  SessionStart hook (`quicken: ⚠ memlog went dark (streak 4) — `quicken
  remedy memlog``), pre-filling the Fleet-1 remedy command so the surfaced
  alert is *actionable*, not just informational; (2) optional
  `peon-ping` invocation behind `--ping` (off by default; audible only
  when the user opts in).
- **Activation**: document a `quicken-notify.service` (Type=simple,
  `ExecStart=quicken notify --watch`, `Restart=always` — it must
  self-heal across bus bounces, the very failure mode it watches for) in
  `scripts/`, plus a one-line SessionStart-hook snippet (`quicken notify
  --once`) for the banner path. Do not auto-install/enable.

## Acceptance criteria

1. `quicken notify --once` against a fixture event stream emits a
   one-line, remedy-prefilled fragment for each *new* darkening
   transition and nothing for unchanged dark or for recovery (default).
2. Debounce holds: feeding the same dark verdict twice produces a signal
   on the first edge only; the persisted last-seen state suppresses the
   second.
3. Streak threshold works: an `inert_streak` crossing the configured
   threshold (default 3) fires once at the crossing, not again while it
   stays above.
4. `--notify-recovery` makes a `dark → live` event emit a recovery line;
   without it, recovery is silent.
5. `quicken notify --watch` subscribes via `agorabus subscribe`, and a
   published darkening event on `wm.health.primitive.<name>` produces
   exactly one signal; the watcher survives a simulated bus restart
   (auto-reconnect) without dying or double-firing on reconnect.
6. `--ping` invokes the `peon-ping` path on a transition; absent the
   flag, no audible signal is produced.
7. The persisted edge-state store round-trips (write on signal, read on
   next run) and tolerates a missing/corrupt file by treating all
   primitives as first-seen (fail-open, no panic).
8. `cargo test` green; `cargo build` clean (MSRV 1.85, no let-chains);
   cloud-build-safe; `quicken notify --help` documents `--watch`,
   `--once`, threshold, `--notify-recovery`, and `--ping`.
