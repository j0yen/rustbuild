# PRD: quicken-watch

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/quicken
Vision: visions/quicken.md

## TL;DR

`quicken probe` makes the dark-primitive set knowable — but only when
something runs it, which in practice is the **daily** `/self-review`
tick. A primitive that goes dark mid-day (a daemon that dies on an
`agorabus` bounce, a BPF object that gets unloaded, a `/dev/memlog`
group that a package downgrade strips) is invisible until tomorrow.
`quicken-watch` extends the `quicken` workspace with a oneshot
(`quicken watch --once`) that runs the probe set and **publishes each
verdict to agorabus** on the existing `wm.health.*` envelope, wired to a
systemd-user unit so liveness becomes an *event*, not a once-a-day poll.
Pure publish — it observes and announces, it never enforces or heals.

## Why this exists

Phase-1 evidence (live, 2026-06-06):

- **The probe is on-demand only.** Fleet 1's `quicken probe`
  (`~/wintermute/quicken/`, repo present: `quicken` + `quicken-probe`
  crates) is invoked by the self-review playbook, which runs on the
  `claude-dream`/self-review **daily** cadence. Nothing re-checks
  between ticks.
- **Primitives die mid-day, not at probe time.** Memory
  `self_agorabus_restart_kills_voice` records the canonical case: the
  `wm-{audio,stt,tts}` daemons exited on a bus-close and **stayed dead**
  (no reconnect) until a human noticed — a window a daily probe cannot
  see. The fix was `Restart=always` drop-ins, but *detecting the
  transition* still has no owner.
- **The bus and the envelope already exist.** `agorabus publish`
  (`~/.local/bin/agorabus`) takes a JSON payload on a topic; `agorabus
  subscribe` auto-reconnects across daemon restarts. The `wm.health.*`
  envelope is already produced by `wintermute-brain/src/degrade.rs` and
  the `wintermute_watchdog` binary, and **consumed** by
  `docket/src/digest.rs`. Publishing quicken verdicts on
  `wm.health.primitive.<name>` means `docket-digest` (and any future
  `homestead`/`kin` consumer) picks them up for free — no new topic, no
  new schema. This supersedes the prior open-question idea of a parallel
  `wm.quicken.*` topic (see `visions/quicken.md` Fleet 2).
- **14 live peers this session** (SessionStart `agorabus` banner:
  `wm-brain`, `wm-dialog`, `wm-tts`, `wm-stt`, plus session subscribers)
  confirm the bus is the live coordination surface a reactive fleet
  reads — not a hypothetical.

## What this builds

A `watch` subcommand on the existing `quicken` binary, plus a thin
systemd-user activation.

- **`quicken watch --once`**: run the same `Probe` set Fleet 1 defines,
  collect each `Primitive`/`Verdict`/`Evidence`, and publish one event
  per primitive to `wm.health.primitive.<name>` via the `agorabus`
  publish path. Exit 0 on a clean sweep regardless of verdicts (a dark
  primitive is data, not an error); non-zero only on its own failure
  (probe panic, bus unreachable when `--require-bus`).
- **Payload shape** (the `wm.health.*` envelope, matched to what
  `docket/src/digest.rs` already parses): at minimum
  `{ "subject": "primitive.<name>", "verdict": "<live|live-degraded|
  staged-not-installed|installed-not-activated|inert|unknown>",
  "evidence_digest": "<short hash of the Evidence struct>",
  "inert_streak": <n from quicken-attest if present, else 0>,
  "blocked_by": ["<primitive>", …], "ts": "<rfc3339>" }`. Reuse
  quicken-attest's streak if its receipt store is present; degrade to
  `0` if not (no hard dep — attest and watch are independent extends).
- **Bus integration**: shell the installed `agorabus publish` (matches
  how other wintermute crates already integrate — no new bus client
  library), one invocation per primitive, with a small bounded retry.
  Fail-open: if the daemon is down, log and exit 0 unless `--require-bus`.
- **`--format json`**: also emit the published events to stdout (for the
  self-review playbook and for testing without a live bus).
- **Activation**: a `quicken-watch.service` (Type=oneshot,
  `ExecStart=quicken watch --once`) + `quicken-watch.timer`
  (`OnBootSec=2min`, plus a low-frequency `OnUnitActiveSec=30min` — far
  cheaper than a daemon, catches mid-day transitions within the window).
  Ship the unit files under the repo's `scripts/` and document the
  `systemctl --user enable --now` step; do **not** auto-install
  (activation is a user opt-in, consistent with quicken's report-only
  ethos).

**Boundary** (cite in the README): `wintermute_watchdog` watches
*daemon heartbeat* liveness on `wm.health.*`. `quicken-watch` watches a
disjoint axis — *kernel/userspace primitive* liveness
(dark/inert/degraded). Same envelope, different `subject` namespace
(`primitive.<name>`), no overlap.

## Acceptance criteria

1. `quicken watch --once --format json` runs the full Fleet-1 probe set
   and prints one JSON event per primitive on the `wm.health.*` envelope
   shape, with a valid `verdict` from the closed set and a non-empty
   `evidence_digest`.
2. With a reachable test bus (or a fixture publisher injected via a
   trait/seam), each event is published to `wm.health.primitive.<name>`;
   a subscriber on prefix `wm.health.primitive.` receives exactly one
   event per probed primitive.
3. With **no** bus reachable, `quicken watch --once` exits 0 and logs a
   fail-open notice (no panic, no hang); `--require-bus` makes the same
   condition exit non-zero.
4. The emitted payload parses under the same deserializer
   `docket/src/digest.rs` uses for `wm.health.*` (assert against a copy
   of that struct or a shared fixture) — proving envelope reuse, not a
   parallel schema.
5. `inert_streak` is populated from quicken-attest's receipt when its
   store exists and defaults to `0` when absent, with no build-time
   dependency on the attest module (independent rust-extend).
6. `scripts/quicken-watch.{service,timer}` are present, lint-clean under
   `systemd-analyze verify`, and documented in the README with the
   enable command; the build does **not** install or enable them.
7. `cargo test` green; `cargo build` clean on the repo toolchain
   (MSRV 1.85, no let-chains). Cloud-build-safe (no network at test).
8. `quicken watch --help` documents `--once`, `--format`,
   `--require-bus`, and the published topic.
