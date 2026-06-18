# PRD: tether-presence — agorabus peers go fleet-wide

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/agorabus
Vision: visions/tether.md
version_bump: minor
deferred_acs: [6]

## TL;DR

The problem: `agorabus peers` only ever lists sessions on *this* laptop, because
announce + peer state live entirely behind the local Unix socket. Once
`tether-link` is up, the work node's `claude` sessions are reachable over
`wm.fleet.presence.*` — but agorabus has no notion of a peer on another machine.
This PRD is the user's literal "extend agorabus": add a `node` identity to the
peer model and a `peers --fleet` view that merges remote peers learned from the
bus with local UDS peers, so presence is fleet-wide. Purely additive — every
existing local client behaves identically.

## Why this exists

- `agorabus/src/` (daemon.rs, client.rs, protocol.rs) implements announce +
  peer-list over the UDS only; there is no `node` field on a peer, so two
  machines' peers can never be told apart even if their events crossed a bridge.
- The bus already namespaces `wm.fleet.>` and `wm-busbridge` forwards exactly
  that prefix — so a `wm.fleet.presence.announce` subject crosses to the work
  node *for free* the moment `tether-link` is up. The only missing piece is
  agorabus understanding "this peer is on node X."
- agorabus AC2 establishes the invariant this extension must preserve: existing
  announce/peer behavior for local clients must not change. The fleet view is an
  opt-in flag, not a protocol break.
- The user framed the work node as "a part of you" — presence is the first,
  smallest proof the limb is attached: you run `agorabus peers --fleet` and *see*
  the work node's sessions.

## What this builds

A minor-version extension of the `agorabus` crate (extend, do not replace):

- **`node` on the peer record.** Add an optional `node` field to the announce op
  and the stored peer (defaulting to the local hostname / Tailscale name when
  absent). Backward-compatible: an announce without `node` is treated as local
  (existing clients unaffected — AC2 preserved).
- **Fleet presence mirror.** When a `node` peer announces locally, the daemon
  publishes a compact `wm.fleet.presence.announce` event (session_id, pid, cwd,
  node, ts); on a matching `wm.fleet.presence.gone` (or TTL expiry) it drops the
  remote peer. Loop-guarded: a presence event that originated from the fleet is
  not re-published to the fleet (mirrors the nats-bridge loop-guard contract).
- **`agorabus peers --fleet`.** Without the flag, `peers` lists local UDS peers
  exactly as today (byte-identical output). With `--fleet`, it merges in remote
  peers learned from `wm.fleet.presence.*`, each tagged with its `node` and a
  freshness age; stale remote peers (past TTL) are omitted.
- **Remote-peer TTL/reaping.** Remote peers expire after a configurable TTL with
  no refresh (a work laptop that sleeps shouldn't leave ghost peers) — consistent
  with agorabus's existing reconnect/reaping discipline.

No new binary; this is the existing `agorabus` CLI/daemon gaining a field, a
flag, and a fleet mirror. Tests use an embedded/test NATS server for the fleet
half and the existing in-process UDS harness for the local half.

## Acceptance criteria

1. The announce op accepts an optional `node` field; an announce that omits it is
   accepted and the peer is recorded as local — `agorabus peers` output is
   byte-identical to the pre-change output for an all-local peer set (AC2
   invariant preserved, golden-compared).
2. An announce carrying `node: "worknode"` records a peer whose `node` is
   `"worknode"`; `agorabus peers --json` includes the `node` field for that peer
   and omits/defaults it for local peers.
3. `agorabus peers` (no flag) lists ONLY local UDS peers and never blocks on the
   fleet/NATS path — verified with the fleet transport absent (no NATS): the
   command succeeds and returns local peers.
4. `agorabus peers --fleet` merges a remote peer published on
   `wm.fleet.presence.announce` (embedded NATS) into the listing, tagged with its
   `node` and a freshness age; the same peer published twice does not appear
   twice (dedup by session_id+node).
5. Loop guard: a presence event injected from the fleet into the local daemon is
   NOT re-published to `wm.fleet.presence.*` (exactly-once per direction), asserted
   against the embedded NATS server.
6. (deferred — embedded NATS) A remote peer with no refresh past its TTL is
   reaped from `--fleet` output; a refreshed remote peer persists. Verified with
   an injected clock + embedded NATS.
7. `cargo test --workspace` green; existing agorabus tests still pass unchanged
   (no regression in the local protocol).
