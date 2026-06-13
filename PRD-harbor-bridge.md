# PRD: harbor-bridge — the hub announces itself on the bus

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/constellation-burst-builder
Vision: visions/harbor.md

## TL;DR

agorabus is Unix-socket-only ("co-located sessions") — the cloud box is invisible
to the fleet, so the laptop can't ask the bus "is a builder up?" without an SSH
probe. This PRD stands up a **NATS server on the permanent hub** and a
**`wm.fleet.hub.{up,down}` + heartbeat publisher**, so the hub announces itself.
It's the bus-reachable **landing pad** the constellation-bus keystone (the
agorabus↔NATS bridge) connects to later — harbor builds the hub-side half now.

## Why this exists

- agorabus runs only over `~/.cache/agorabus/sock`; this session shows 13 peers,
  all local PIDs. The burst/hub box never appears on the bus — fleet-wide
  coordination has no transport off the laptop.
- The constellation vision names the **agorabus↔NATS bridge the keystone new
  component** and notes NATS has no Unix-socket listener, so the integration is a
  sidecar. harbor doesn't build the full bridge (that's constellation-bus) — it
  stands up the NATS endpoint + a hub heartbeat so the landing pad exists and the
  hub is observable the moment the bridge lands.
- The day-2 constellation layer (`constellation-status`, `constellation-hub-
  failover`) both assume a `wm.fleet.hub.*` event stream that **nothing currently
  emits**. harbor-bridge is the first emitter.

## What this builds

**`scripts/harbor-bridge-up.sh`** — against the hub:
- Install + start `nats-server` (JetStream enabled) as a systemd unit, bound to
  the private/mesh interface, with a generated auth token written to the secret
  store (not a tracked file).
- Install a small `harbor-heartbeat` systemd timer/service on the hub that
  publishes `wm.fleet.hub.up` on start and a periodic heartbeat
  (`wm.fleet.hub.heartbeat` with `{id, type, ip, ts}` from `hub.json`), and
  `wm.fleet.hub.down` on stop (systemd `ExecStop`).
- Idempotent: re-run reconciles units without duplicating.

**`scripts/harbor-bridge-probe.sh`** — run on the laptop: connect to the hub's NATS
(endpoint from `hub.json` + token from secret store), subscribe briefly, and report
whether a recent `wm.fleet.hub.heartbeat` was seen → "hub reachable / stale / down".
This is the "is a builder up?" answer without SSH.

**`scripts/harbor-bridge-check.sh`** — offline gate: asserts the subject names
match the `wm.fleet.hub.*` contract the constellation day-2 PRDs expect, the up
script is idempotent, and the auth token is never written to a tracked file.

## Acceptance criteria

1. `harbor-bridge-up.sh --dry-run` prints the planned `nats-server` unit +
   `harbor-heartbeat` unit + token-gen and exits 0 without touching the hub.
2. Subjects are exactly `wm.fleet.hub.up`, `wm.fleet.hub.down`,
   `wm.fleet.hub.heartbeat` — asserted by `harbor-bridge-check.sh` (these must
   match what `constellation-status`/`constellation-hub-failover` consume).
3. The heartbeat payload is JSON with at least `{id, type, ip, ts}`, sourced from
   `hub.json`; verified by parsing a sample emitted in `--dry-run`.
4. Idempotency: a second `--dry-run` against a "bridge-up" fixture reports
   "reconcile units, no duplicate" (assert via marker).
5. The NATS auth token is written only to the secret store
   (`~/.config/wm-burst/bridge.env`, gitignored); a grep of tracked files finds no
   token value.
6. `harbor-bridge-probe.sh --dry-run` (no live hub) explains it would subscribe to
   `wm.fleet.hub.heartbeat` and classify up/stale/down by heartbeat age; with no
   `hub.json` it exits non-zero with "no hub".
7. `harbor-bridge-check.sh` passes as the offline acceptance gate and is invocable
   from the `scripts/` runner.
8. README/REMOTE-SETUP documents that this is the **landing pad** for
   constellation-bus, not the full agorabus↔NATS bridge — scope boundary explicit.
