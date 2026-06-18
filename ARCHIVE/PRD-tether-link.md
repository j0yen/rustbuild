# PRD: tether-link — a persistent, self-healing link to the work node

Status: Draft v0.1
build_target: rust-cli
Vision: visions/tether.md
deferred_acs: [5, 6]
mock_unjustified_for: [6]
mock_justifications:
  6: "AC6 requires a real second machine (the work node) reachable over a real
      Tailscale mesh + NATS hub across a real sleep/wake/network-flap cycle. No
      in-crate fake can recreate a physical laptop suspending and resuming its
      network; the link-survival property is only truthfully verifiable in the
      world, on the work box, after install."

## TL;DR

The problem: agorabus is a Unix-domain socket — it cannot reach another machine
(its README: "co-located sessions"). The `agorabus-nats-bridge` (`wm-busbridge`,
already installed) can mirror `wm.fleet.*` onto a NATS leaf, but *nothing keeps
that leaf up* across a work laptop's day — sleep/wake, VPN flap, network change,
reboot all silently drop it, and the bridge then mirrors into the void.
`tether-link` is the supervisor that makes the connection *persistent*: it brings
up the link, watches it, and reconnects with backoff, so the work node stays
wired into the self without anyone babysitting it.

## Why this exists

- agorabus `README.md`: *"advisory presence+pub/sub substrate over a Unix-domain
  socket so co-located sessions can announce themselves"* — single-host by
  construction; the bus has no network reach.
- `wm-busbridge` is INSTALLED (`~/.local/bin/wm-busbridge`) and forwards
  `wm.fleet.>` to a NATS leaf at `127.0.0.1:4222`, but its ACs assume the leaf is
  *already reachable*; there is no component that keeps the leaf connected across
  a real machine's power/network lifecycle.
- constellation Phase-1 research already chose the transport (Tailscale mesh +
  NATS leaf↔hub) and noted NATS has no Unix listener — so the link is a
  supervised sidecar, exactly this PRD's scope.
- The work node is a *laptop* — it sleeps. A persistent link to a sleeping peer
  must reconnect cleanly on wake, not wedge. That reconnect discipline is the
  load-bearing behavior here.

## What this builds

A `wm-tether` binary (single crate, `~/wintermute/tether-link`) wrapping a small
state machine:

- `wm-tether up` — ensure Tailscale is up (shells `tailscale status --json`,
  does not manage Tailscale auth), then start/adopt the NATS leaf connection and
  launch `wm-busbridge` if not already running; idempotent.
- `wm-tether status [--json]` — report link state (`down|connecting|up`), the
  resolved hub/peer endpoint, the NATS leaf RTT (a `wm.fleet.ping` round-trip),
  and the last reconnect time + count.
- `wm-tether down` — tear the link down cleanly (stop the supervised leaf;
  leave Tailscale alone).
- A supervisor loop (`wm-tether run`, the service entrypoint) that watches the
  leaf, detects drop (missed heartbeat / closed conn), and reconnects with
  exponential backoff (capped, jittered); emits a structured `link-event` line
  per transition (`up|down|reconnect`, ts, backoff_ms, attempt).
- A `tether-link.service` (systemd --user) unit shipped in the repo, `Restart=
  always`, `WantedBy=wintermute.target`, so the supervisor itself self-heals.
- Config from `~/.config/wm-tether/config.toml` (hub endpoint, peer fallback,
  backoff caps, ping interval); a `wm-tether config-example` subcommand prints a
  ready-to-edit example. No secrets in the repo (grep-asserted); NATS creds come
  from the existing encrypted store the bridge already uses.

Deps: `clap`, `serde`/`serde_json`, `toml`, an async NATS client for the ping
round-trip (test against an embedded/test NATS server, the nats-bridge
precedent), `sigpipe`. MSRV 1.85, edition 2021. `sigpipe::reset()` first line of
`main()`.

Honesty discipline (inherited from `persona`): authored + fixture-tested on this
laptop; the real cross-machine survival property (AC6) is verified on the work
box after install. `wm-tether status` SKIPs honestly (`state: down, reason:
no-link-configured`) when no config/link is present — never false-green.

## Acceptance criteria

1. `wm-tether config-example` prints a valid TOML config (parses back via the
   crate's own loader) containing hub endpoint, peer fallback, backoff caps, and
   ping interval; no secret values present.
2. `wm-tether status --json` against a configured-but-down link emits
   `{"state":"down", ...}` with exit code reflecting down-state, and does NOT
   panic or hang (bounded by a connect timeout).
3. The supervisor's backoff is exponential, capped, and jittered: given a stub
   clock and a connect function that fails N times then succeeds, the recorded
   `backoff_ms` sequence is non-decreasing, never exceeds the configured cap, and
   the loop converges to `up` exactly once (no thundering reconnect after
   success). Unit-tested against an injected fake transport + clock.
4. Each link state transition emits exactly one structured `link-event` line
   (valid JSON: `kind`, `ts`, `attempt`, `backoff_ms`) — asserted by capturing
   the event sink across a scripted down→reconnect→up sequence.
5. (deferred — embedded NATS) `wm-tether status` reports a real RTT via a
   `wm.fleet.ping` round-trip against an embedded/test NATS server; the ping
   subject is `wm.fleet.*` so it crosses the existing bridge.
6. (deferred — real hardware, no mock per justification) On the work box, the
   link survives a sleep/wake cycle and a network change: `wm-tether status`
   returns to `up` within the configured reconnect window without manual
   intervention, verified live after install.
7. `sigpipe::reset()` is the first statement in `main()` (grep-asserted); piping
   `wm-tether status` into `head` does not panic.
8. The shipped `tether-link.service` unit parses (`systemd-analyze verify` or a
   structural check), declares `Restart=always`, and is `WantedBy=
   wintermute.target`; install is documented but not auto-enabled by the test.
