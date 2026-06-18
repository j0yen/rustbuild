# PRD: corpus-attest — who is legitimately "me"

Status: Draft v0.1
build_target: rust-cli
Vision: visions/corpus.md
deferred_acs: [7]

## TL;DR

The problem: once the bus reaches another machine (tether), *anything* that can
talk to the bus can announce itself as part of the self. "Me" must be a closed,
verifiable set, not whoever shows up. `corpus-attest` is the membership
primitive: a node proves it is a legitimate limb of the self by presenting a
fleet credential bound to its session identity, and other nodes verify that
proof before counting it as "me." It answers the first question a multinode
entity must answer — *who am I, and who isn't?*

## Why this exists

- `tether` (vision, drafted today) opens `wm.fleet.*` to a second machine. The
  agorabus trust model is *single-user, single-host*; the moment the bus spans
  machines, "co-located = trusted" no longer holds and membership needs an
  explicit gate.
- The kernel can identify a session: agentns mints a 128-bit
  `/proc/$PID/agent_session` id and provfs stamps `user.prov.session`. But that
  id is an **attribute, not a credential** — it says *which* session, not *that
  the session is authorized*. And activation is currently **BLOCKED**
  (SessionStart reminder 2026-06-17: *"[agentns] ACTIVATION BLOCKED at
  kernel-prctl — install linux-wintermute pkgrel >= 12 and reboot"*). So attest
  must authenticate in userspace and treat the agentns id as an optional bound
  attribute, present only when the kernel surface is live.
- Every downstream corpus component (roster membership, arbiter leases,
  convergence participation) is only safe if granted to attested nodes — attest
  is the root of the dependency graph for that reason.

## What this builds

A `corpus-attest` binary (single crate, `~/wintermute/corpus-attest`):

- **Identity.** On first run a node generates a per-node keypair stored under
  `~/.config/corpus/` (0600). Its node identity is `{hostname, tailscale_node?,
  pubkey, agentns_session?}` — `agentns_session` populated from
  `/proc/self/agent_session` when readable and non-zero, omitted (with a logged
  `degraded: agentns-inactive`) otherwise.
- **Credential.** Fleet membership is established by **per-node Ed25519 keypairs
  + Tailscale ACL identity** (jsy decision 2026-06-18; no shared fleet secret,
  no fleet CA). Each node generates a keypair on first enroll and records its
  Tailscale node name as the authoritative network identity. The fleet root of
  trust is: "this Tailscale node name is enrolled, and it owns this pubkey."
  Enrollment writes `~/.config/corpus/fleet.toml` (the set of enrolled
  `{tailscale_node, pubkey}` pairs — the fleet roster); each node ships a copy
  (synced over gossip or manual `corpus-attest sync-roster`). `corpus-attest
  enroll` produces a signed attestation `{node_identity, issued_ts, expiry,
  sig}` where `sig` is the node's own Ed25519 signature over the identity +
  timestamps, and `verify` checks sig against the enrolled pubkey for that
  Tailscale node — never a standalone CA.
- **Verify.** `corpus-attest verify <attestation>` checks the signature against
  the fleet root and the expiry window, returning `valid | invalid | expired`
  with the reason. This is the function roster/arbiter call before trusting a
  node.
- **Present / challenge.** `corpus-attest present` emits this node's current
  attestation (for publishing on `wm.fleet.attest.*`); `corpus-attest
  whoami [--json]` prints the local node identity + attestation status +
  degraded flags.
- **SKIP-honest.** With no fleet root configured, `enroll`/`verify` exit with a
  clear `not-enrolled` status (non-zero) rather than fabricating membership;
  `whoami` still prints the local identity marked `unattested`.

Deps: `clap`, `serde`/`serde_json`, `toml`, a vetted signature crate
(ed25519-dalek or ring), `sigpipe`. No `unsafe`. MSRV 1.85, edition 2021.
`sigpipe::reset()` first in `main()`. Honesty discipline (per `persona`/`tether`):
fixture-tested here (generated test keypairs + a test fleet root); the live
cross-node + live-agentns paths are deferred.

## Acceptance criteria

1. `corpus-attest whoami --json` emits a node identity containing `hostname` and
   `pubkey`; `agentns_session` is present iff `/proc/self/agent_session` is
   readable and non-zero, and a `degraded:["agentns-inactive"]` marker appears
   when it is not (the current real state on this box).
2. `enroll` produces an Ed25519-signed attestation; `verify` returns `valid` for
   an enrolled `(tailscale_node, pubkey)` pair and `invalid` if any field is
   tampered (flip one byte of the node identity or sig).
3. An attestation past its `expiry` verifies as `expired` (injected clock), not
   `valid`.
4. No private key material is committed in the repo (grep-asserted); the
   private key is written 0600 under `~/.config/corpus/` at runtime. The fleet
   roster (`fleet.toml`) contains only pubkeys + Tailscale node names — no
   secrets — and is safe to commit/sync.
5. With no fleet root configured, `enroll` and `verify` exit non-zero with a
   `not-enrolled` reason and do NOT emit a usable attestation; `whoami` still
   succeeds and marks the node `unattested`.
6. Two distinct generated keypairs produce distinct, non-colliding node
   identities; an attestation issued for node A does NOT verify as node B.
7. (deferred — embedded NATS) `present` publishes this node's attestation on
   `wm.fleet.attest.announce`; a peer consuming it and calling `verify` accepts a
   valid attestation and rejects a forged one — round-tripped against an embedded
   NATS server.
8. `cargo test` green; no `unsafe`; `sigpipe::reset()` first in `main()`
   (grep-asserted).
