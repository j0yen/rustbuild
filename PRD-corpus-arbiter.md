# PRD: corpus-arbiter — two of me want to write; who wins

Status: Draft v0.1
build_target: rust-cli
Vision: visions/corpus.md
deferred_acs: [7]

## TL;DR

The problem: agorabus already arbitrates which *co-located* session owns a
resource (`claim_guard.rs`) — but across nodes there is no such guard. When the
laptop and the work node both try to write the same shared thing (the recall
memory store, `settings.json`, a repo), they clobber each other one network hop
apart. `corpus-arbiter` lifts the single-writer discipline to fleet scale: a
node acquires a lease on a named resource before writing, others wait or yield,
and a dead node's lease auto-releases. The self stops clobbering itself across
the wire.

## Why this exists

- `agorabus/src/claim_guard.rs` exists precisely because concurrent sessions on
  one host clobber shared files (its README names `settings.json` and the recall
  DB as the casualties). tether makes those exact files reachable from a second
  node — so the same failure mode reappears across machines, where claim_guard
  cannot see.
- `corpus-attest` defines membership; leases must only be granted to attested
  nodes, or an impostor could starve the self of its own resources. Arbiter
  consumes attest.
- A laptop that sleeps mid-lease must not deadlock the self forever — leases need
  TTL + auto-release, mirroring how agorabus reaps stale peers and how
  corpus-converge treats partition as routine.
- This is an *advisory* lock (single-user trust model — it's all jsy): correctness
  rests on every writer politely acquiring first, exactly as flock-based
  `cargo` target locking and claim_guard already work here. Not a Paxos; a lease
  registry.

## What this builds

A `corpus-arbiter` binary (single crate, `~/wintermute/corpus-arbiter`):

- **Lease acquire/release.** `corpus-arbiter acquire <resource> [--ttl N]`
  requests a lease on a named resource over `wm.fleet.lock.acquire`; the holder
  (hub-authoritative registry, per vision OQ#1) grants it if free, else replies
  `held-by <node> until <ts>`. `release <resource>` frees a held lease.
  `corpus-arbiter with <resource> -- <cmd...>` acquires, runs the command, and
  releases on exit (even on failure) — the ergonomic path for "lock around this
  write."
- **Registry.** The authoritative lease registry tracks `{resource: {holder_node,
  acquired_ts, expiry}}`. A lease past `expiry` with no renewal is auto-released
  (dead-node safety). Renewal (`renew`) extends a held lease's expiry.
- **Deny-by-default + attestation.** Only attested nodes (corpus-attest verify)
  may acquire; an unattested requester is refused. A resource not in the
  lease-able registry is refused (named-resource allowlist, OQ#3) so the arbiter
  can't be used to lock arbitrary paths.
- **Status.** `corpus-arbiter status [--json]` lists held leases with holder +
  remaining TTL. SKIP-honest: with no authority reachable, `acquire` exits
  non-zero with `no-arbiter` rather than proceeding lock-free (fail-safe: if you
  can't lock, you don't write).

Deps: `clap`, `serde`/`serde_json`, `toml`, async NATS client (embedded test
server), `sigpipe`. MSRV 1.85, edition 2021. `sigpipe::reset()` first in
`main()`. Tests drive the registry logic with injected clock + embedded NATS; no
live second node required.

## Acceptance criteria

1. The registry grants a lease on a free resource and refuses a second acquire of
   the same resource while held, replying `held-by <node> until <ts>` (unit-tested
   against the registry with an injected clock).
2. A lease past its `expiry` with no renewal is auto-released: a subsequent
   acquire by another node succeeds, and `status` no longer lists the expired
   holder (injected clock).
3. `renew` by the current holder extends `expiry`; `renew` by a non-holder is
   refused and does not alter the lease.
4. Deny-by-default: an acquire from a node whose attestation fails `corpus-attest
   verify` is refused; an acquire for a resource absent from the lease-able
   registry is refused (table-driven).
5. `corpus-arbiter with <resource> -- <cmd>` releases the lease when `<cmd>`
   succeeds AND when it fails/panics (no leaked lease on the error path) —
   asserted with a command that exits non-zero.
6. With no authority reachable, `acquire` exits non-zero with `no-arbiter` and
   the wrapped `with` command is NOT run (fail-safe: no lock → no write).
7. (deferred — embedded NATS) End-to-end: two instances contend for one resource
   over `wm.fleet.lock.*`; exactly one holds at a time, the loser observes
   `held-by`, and after the holder releases the loser can acquire — round-tripped
   against an embedded NATS server.
8. `cargo test` green; `sigpipe::reset()` first in `main()` (grep-asserted).
