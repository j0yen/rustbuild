# PRD: corpus-converge — rejoin this self, not a stale fork

Status: Draft v0.1
build_target: rust-cli
Vision: visions/corpus.md
deferred_acs: [6]

## TL;DR

The problem: the work node is a laptop — it sleeps, loses network, reboots. While
it is away, the self moves on (memories written, gossip appended, leases taken).
When it rejoins, it must catch up to the *current* self-state before acting, or it
becomes a stale fork making decisions on a stale view. `corpus-converge` is the
coherence primitive: a versioned self-state published over the bus, and a
rejoin protocol that re-syncs a returning node to the current version and
announces its return to the rest of the self.

## Why this exists

- `tether-link`'s own design notes that the work node sleeps and the link must
  reconnect cleanly on wake. Reconnecting the *transport* is necessary but not
  sufficient — the node also has a stale *view* of the self after a gap, and
  nothing yet tells it "here is what changed while you were out."
- The corpus components produce per-node monotonic sequences already
  (tether-gossip persists a per-node `seq`; corpus-roster stamps replies). A
  self-state version is the natural generalization: a vector of per-node
  sequences (lamport-style) that defines "how current am I?"
- Partition is the normal case here, not an edge case: a work laptop is offline
  most evenings and weekends. The self must treat rejoin-after-partition as
  routine and converge without manual intervention.
- This is deliberately *not* a CRDT or a consensus engine (vision OQ#1/#3): the
  shared mutable state (memory, gossip) is append-only or arbiter-gated
  elsewhere, so converge only needs a *version vector + a catch-up fetch*, not
  conflict resolution. Keeping scope this narrow is what makes it a single PRD.

## What this builds

A `corpus-converge` binary (single crate, `~/wintermute/corpus-converge`):

- **Self-state version.** A version vector `{node: seq}` representing the latest
  known sequence per attested node, persisted to `~/.cache/corpus/state` and
  published on `wm.fleet.state.version` on change. `corpus-converge version
  [--json]` prints the local view.
- **Heartbeat.** While linked, a node periodically publishes its current version
  vector; receiving a peer's vector merges it (element-wise max) into the local
  view, so every linked node trends toward the same vector.
- **Rejoin protocol.** On startup / link-up after a gap, `corpus-converge sync`
  requests the current authoritative version (hub-authoritative per OQ#1, with a
  peer-max fallback), computes the delta against the local persisted version, and
  reports what the node is behind on per channel (gossip seqs, roster epoch,
  lease epoch) — emitting a `rejoin` event `{node, was_version, now_version,
  gap}`. It does not itself replay channel contents (each channel's own daemon —
  tether-gossip etc. — does the catch-up); converge tells them *how far behind*.
- **Staleness guard.** `corpus-converge fresh?` exits 0 if the local version is
  within a configured lag of the authoritative version, non-zero (with the gap)
  if stale — a gate other tools can call before acting on possibly-stale state.
- SKIP-honest: with no link/authority reachable, `version` prints the local view
  marked `unsynced`; `sync`/`fresh?` exit non-zero with `no-authority`.

Deps: `clap`, `serde`/`serde_json`, async NATS client (embedded test server),
`sigpipe`. MSRV 1.85, edition 2021. `sigpipe::reset()` first in `main()`. Tests
use injected version vectors + an embedded NATS server; no dependency on a live
second node.

## Acceptance criteria

1. Version-vector merge is element-wise max and commutative: merging vectors in
   either order yields the same result; merging a vector with itself is a no-op
   (property-tested over generated vectors).
2. `corpus-converge version --json` round-trips through persistence: after a
   restart the printed vector equals the last persisted vector, not an empty one.
3. `sync` against a stubbed authoritative version emits a `rejoin` event whose
   `gap` correctly lists, per channel, how far behind the local version is
   (e.g. `gossip: worknode behind by 4`); a node already current emits `gap: {}`.
4. `fresh?` exits 0 when the local version is within the configured lag of the
   authority and non-zero (printing the gap) when it exceeds it (injected
   vectors + clock).
5. With no authority reachable, `version` is marked `unsynced` and `sync`/`fresh?`
   exit non-zero with `no-authority` — never fabricates currency.
6. (deferred — embedded NATS) Heartbeat round-trip: two instances publishing
   version vectors on `wm.fleet.state.version` converge to the same merged vector
   within a bounded number of heartbeats, asserted against an embedded NATS
   server.
7. `cargo test` green; `sigpipe::reset()` first in `main()` (grep-asserted);
   `corpus-converge version | head` does not panic.
