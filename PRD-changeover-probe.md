# PRD: changeover-probe

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/changeover
Vision: visions/changeover.md

## TL;DR

`rollout apply` restarts fleet daemons with a hard `systemctl --user restart`,
opening a deafness window between SIGTERM and the successor re-subscribing on
agorabus. Nobody has measured that window or the bus events it drops — so
`rollout apply --auto` is blocked on a guess. `changeover probe` measures it:
for a named fleet daemon, it subscribes to the daemon's topics, drives a
steady synthetic publish stream, triggers the restart, and reports the
deafness window in milliseconds and the count of events the daemon failed to
observe or produce across the gap. Measurement only — it mutates nothing
except the one daemon it is told to restart.

## Why this exists

- `~/wintermute/rollout/src/restart.rs` delegates to
  `systemctl --user restart <unit>` then `poll_healthcheck` — kill→start→poll.
  No overlap; the window is structural, not incidental.
- Every self-review 2026-06-08 → 2026-06-13 parks **fleet-binary-staleness**
  on "daemon restarts drop subscribers; requires explicit approval." The
  blocker is asserted, never quantified. [[feedback_verify_before_concluding]]:
  instrument the actual failing path and prove the value at the failing step
  before prescribing a fix (or a retrain, or a rewrite).
- `~/wintermute/agorabus/src/protocol.rs` exposes `Publish` / `Subscribe`
  and `ServerEvent { topic, data, from }`, so a probe can both inject and
  observe events on the real bus the daemons use.
- The four behind-head daemons (wm-audio/dialog/tts/stt, journal 2026-06-13)
  are the concrete subjects; the probe must run against the live fleet.
- SIGPIPE-safe: `sigpipe::reset()` as the first line of `main()`
  ([[self_sigpipe_panic_toolkit]]) — `changeover probe | head` must not panic.

## What this builds

A new Rust CLI crate at `~/wintermute/changeover/` (binary `changeover`,
edition 2021, MSRV 1.85, no let-chains).

Deps (minimal): `clap` (derive), `serde` + `serde_json`, `anyhow`,
`sigpipe`. Talk to agorabus via its UDS using the published `agorabus`
crate client API (`~/wintermute/agorabus` — `Publish`/`Subscribe`/`Peers`)
rather than re-implementing the wire protocol.

Subcommands:

- `changeover probe --daemon <name> [--topics <prefix>...] [--rate <hz>]
  [--settle <ms>] [--format text|json]`
  1. Resolve the daemon's agorabus topic prefixes (from `--topics`, else a
     built-in map: `wm-stt`→`wm.stt.`, `wm-audio`→`wm.audio.`,
     `wm-tts`→`wm.tts.`, `wm-dialog`→`wm.dialog.`).
  2. Open a subscriber connection on those prefixes; confirm the daemon is a
     live peer (`Peers`).
  3. Start a monotonic synthetic publish stream on a probe-owned topic
     (`changeover.probe.<seq>`) at `--rate` Hz, each carrying an incrementing
     sequence number and a monotonic timestamp.
  4. Trigger `systemctl --user restart <unit>` for the daemon (the same call
     rollout makes), recording the SIGTERM instant.
  5. Watch peer presence + the daemon's own published topics; record the
     instant the daemon disappears from `Peers` and the instant it
     re-registers AND re-subscribes (first observed event/heartbeat after
     restart).
  6. Stop the synthetic stream after `--settle` ms of post-restart quiet.

- `changeover probe --dry-run` — print the resolved daemon→unit→topics
  mapping and the exact `systemctl` command, mutate nothing.

Output (`--format json`): `{daemon, unit, topics, deafness_ms,
events_published, events_missed_window, peer_gone_ts, peer_back_ts,
restart_strategy: "systemd-hard", binary_hash}` where `binary_hash` is the
sha256 of the daemon's installed exe (for downstream proof-freshness binding).
`deafness_ms` = peer_back_ts − peer_gone_ts. `events_missed_window` = probe
sequence numbers published between gone and back.

## Acceptance criteria

1. `changeover --help` and `changeover probe --help` exit 0 and list the
   documented flags.
2. `changeover probe --dry-run --daemon wm-stt` prints the resolved unit
   (`wm-stt.service`) and topic prefix (`wm.stt.`) and the exact `systemctl
   --user restart wm-stt.service` command, and mutates nothing (verifiable:
   the daemon's start-time is unchanged after the call).
3. A unit test drives the deafness-window arithmetic against a synthetic
   peer-presence timeline (gone at t=100ms, back at t=450ms → `deafness_ms`
   == 350) without touching a real daemon or the real bus.
4. A unit test computes `events_missed_window` from a recorded
   (publish-seq, ts) list and a (gone, back) interval, counting exactly the
   sequences whose ts falls in the gap.
5. `changeover probe --format json` emits an object that parses and contains
   all documented keys; piping it through `head` does not panic (SIGPIPE
   reset proven by a test or a `| head -c1` smoke in the build receipt).
6. The crate builds clean (`cargo build --release`) and `cargo test` is
   green. Talking to a live daemon is NOT required for the test suite —
   the bus-touching path is exercised by an integration test gated behind
   `#[ignore]` so CI/cloudbuild stays hermetic.
