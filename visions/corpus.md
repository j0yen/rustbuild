# Vision: corpus — the many nodes are one body

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-17
**Status:** active
**Seed:** user — *"focus on the vision of yourself as a multinode entity."*

---

## TL;DR

`tether` connects the nerves — a persistent bus link and four mirrors between the
laptop and the work node. `constellation` lays the fleet infrastructure.
`muster` rosters the sessions *on one machine*. But none of them answers the
question the seed actually asks: **when wintermute runs on many nodes, what makes
it one self rather than several?** `corpus` — the body — is that self-model
layer. It is the difference between *the nerves are wired* (tether) and *there is
one mind using them* (corpus): a fleet-wide roster of "what is all of me doing
right now," an attestation that proves a node is a legitimate limb of the self
(not an impostor), a convergence protocol so a node that was offline rejoins
*this* self rather than a stale fork, an arbiter so two nodes never clobber the
shared state they both are, and a single introspective view that lets the self
see itself whole. `corpus` is wintermute's answer to *"where, and who, am I —
across all of me?"*

## End-state

When this is done:

- `corpus introspect` (run on any node) prints the whole self: every attested
  node, what each is doing right now, whether shared memory is converged or
  diverging, link health per node, and any held write-leases. One command, one
  picture of the entire entity — the self able to see itself whole.
- A node cannot join the self by merely announcing a name. It must **attest** —
  prove membership against a fleet credential bound to its session identity — so
  "me" is a closed, verifiable set, not whoever shows up on the bus.
- `muster --fleet` lists every live `claude` session across every node, not just
  the local box; the self-review's "which sessions are running" question is
  answered for the *whole* entity, not one machine's guess.
- A node that slept, lost network, or rebooted **rejoins coherently**: it
  re-syncs to the current self-state version rather than acting on a stale view,
  and the rest of the self learns it is back.
- Two nodes that both want to write the same shared resource (the memory store,
  `settings.json`, a repo) **arbitrate**: one acquires a lease, the other waits
  or yields, and a dead node's lease auto-releases — so the self never clobbers
  itself across the wire (agorabus's local `claim_guard`, lifted to fleet scale).

## Why now (Phase 1 evidence, 2026-06-17)

- **The plumbing is being dreamed; the self-model is not.** `tether` (drafted
  today) gives presence/gossip/recall/tools mirrors over a persistent link, but
  every one of those is a *channel*. None of them says which nodes legitimately
  constitute the self, how the self stays coherent under partition, or how to
  see the whole at once. That gap is exactly what "yourself as a multinode
  entity" names.
- **`muster` already rosters one machine.** `~/wintermute/muster` (j0yen/muster)
  enumerates live `claude` processes, attributes origin, and classifies
  live/duplicate/orphan/stale — but its own vision scopes it to *"which `claude`
  processes are running right now"* on **this** box. The multinode self needs
  that roster federated; `corpus-roster` extends muster rather than forking it.
- **The kernel already mints a session identity — when it's live.** agentns
  gives a 128-bit `/proc/$PID/agent_session` id, and provfs stamps
  `user.prov.session` on written files. That is the natural primitive for "prove
  this node's work is mine." BUT activation is currently **blocked** (SessionStart
  reminder, 2026-06-17: *"[agentns] ACTIVATION BLOCKED at kernel-prctl — install
  linux-wintermute pkgrel >= 12 and reboot"*; the long-standing all-zeros
  EINVAL). So `corpus-attest` must use the agentns id *when present* and fall
  back to a userspace identity (hostname + Tailscale node + per-node keypair)
  when it isn't — degrading honestly, never assuming the kernel surface is hot.
- **agorabus already solved single-writer locally.** `agorabus/src/claim_guard.rs`
  arbitrates which co-located session "owns" a resource. The multinode self has
  the *same* problem one network hop away; `corpus-arbiter` is that pattern at
  fleet scale, not a new invention.
- **Self-review currently guesses per-machine.** The recall reflective notes
  repeatedly flag "duplicate Claude sessions … may both be" and answer the
  population question with `pgrep` on one host. A multinode self makes that a
  single federated truth.

## Components (PRD-sized)

- **corpus-attest** (rust-cli) — membership & identity. A node proves it is a
  legitimate limb of the self via a fleet credential bound to its session
  identity (agentns 128-bit id when live; userspace hostname+Tailscale+keypair
  fallback otherwise). Answers *who is me?* Foundational — roster membership and
  arbiter leases are only granted to attested nodes.

- **corpus-roster** (rust-extend `muster`) — federated presence. Extend muster so
  `muster --fleet` aggregates each node's local roster over `wm.fleet.muster.*`,
  filtered to attested nodes. Answers *what is all of me doing right now?* Reuses
  muster's census + verdict (no reimplementation). Depends on tether-presence +
  corpus-attest.

- **corpus-converge** (rust-cli) — coherence under partition. A versioned
  self-state (per-node monotonic sequence / lamport-style clock) published over
  the bus; a node that rejoins after sleep/network-loss/reboot re-syncs to the
  current version before acting, and announces its return. Answers *after I was
  away, what is true now?*

- **corpus-arbiter** (rust-cli) — single-writer across nodes. A lease-based
  advisory lock over `wm.fleet.lock.*`: a node acquires a lease on a named
  shared resource (memory store, settings, a repo path) before writing; a dead
  node's lease auto-releases on TTL. Deny-by-default; lifts agorabus
  `claim_guard` to fleet scale. Answers *two of me want to write — who wins?*

- **corpus-introspect** (rust-cli) — the whole-self view. `corpus introspect`
  synthesizes attested nodes (attest) + per-node activity (roster) + self-state
  convergence (converge) + held leases (arbiter) + link health (tether) into one
  human-readable picture. Answers *describe all of me right now.* The capstone —
  consumes the other four; degrades gracefully when any source is absent.

## Order

```
corpus-attest ──► corpus-roster ─────────────────┐
              ├──► corpus-converge ───────────────┼──► corpus-introspect
              └──► corpus-arbiter ────────────────┘
```

`corpus-attest` is the root — membership gates everything ("me" must be defined
before the self can be rostered, converged, or locked). `corpus-roster`,
`corpus-converge`, and `corpus-arbiter` each consume attest and are independent of
each other. `corpus-introspect` is the capstone that unifies all four into the
self's view of itself.

## Open questions

1. **Where does the authoritative self-state live?** Hub-authoritative (Hetzner
   via `harbor`, durable, survives laptop sleep) vs fully peer-to-peer (no single
   point, but harder convergence). Leaning hub-authoritative with peer cache —
   but that's a corpus-converge design decision, flag it.
2. ~~**Attestation root of trust.**~~ **RESOLVED (2026-06-18, jsy):**
   per-node Ed25519 keypairs + Tailscale ACL identity. No shared fleet secret,
   no fleet CA. Each node self-signs its attestation with its own keypair;
   `verify` checks the sig against the enrolled pubkey for that Tailscale node.
   The fleet roster (`fleet.toml`) is a set of `{tailscale_node, pubkey}` pairs
   — pubkeys only, safe to sync. The agentns id remains an *attribute* bound to
   the attestation when live, not the credential itself.
3. **Arbiter scope.** Which resources are lease-gated? The memory store and
   `settings.json` are obvious; is a whole repo too coarse? Probably a
   named-resource registry, deny-by-default, grown as collisions are observed.
4. **Relationship to muster-reap.** A federated roster that can see orphans on
   *other* nodes is one `--confirm` away from killing processes across the wire —
   explicitly OUT of scope for corpus-roster (read-only). Cross-node reap, if
   ever, is a separate reviewer-gated vision, not this one.
5. **corpus-narrate** (next pass) — a unified self-review/journal across nodes
   ("today, across all of me, I did X on the laptop and Y on the work node"),
   composing with the existing self-review machinery. Left as a bullet until the
   five core PRDs have shape.
