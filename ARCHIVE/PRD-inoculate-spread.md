# PRD: inoculate-spread — horizontal transmission over the bus

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/inoculate
Vision: visions/inoculate.md

## TL;DR

Vertical injection (inoculate-inject) covers parent→child at spawn. But a fresh
agent that attaches to the box later — a new voice daemon, a new Claude session,
eventually a new `constellation` node — has no parent to inherit from. This is
the literal "infection": `inoculate-spread` lets any agent **announce its strain
version on the agorabus bus and pull a newer strain from a peer**, so the fleet
converges on one current ethic without hand-configuration.

## Why this exists

Verified live 2026-06-15: `agorabus peers | jq length` = 13 — a working bus with
many concurrent agents already exists. The `constellation` vision plans to extend
this to a multi-machine NATS-bridged bus. Today, if CLAUDE_SELF.md's Boundaries
change, only the main loop that re-reads the file knows; every other live agent
keeps the old ethic until it restarts. There is no propagation. The bus is the
obvious carrier, and `agorabus` is already the box's transmission substrate.

## What this builds

Extends `inoculate` (from inoculate-core), depending on `agorabus`
(`~/wintermute/agorabus`, currently v0.10):

- `inoculate spread announce` — publish this box's current `{strain_hash, version,
  source_self_sha}` to an `inoculate.strain.announce` subject on agorabus.
- `inoculate spread listen` — subscribe; on hearing a peer announce a *newer*
  strain version than local, request the full strain body (`inoculate.strain.req`
  / `inoculate.strain.body`), validate it (well-formed, parses as a `Strain`), and
  write it to a local cache (`~/.cache/inoculate/strain.json`) marked
  `source=peer:<session_id>`. **Does not** overwrite CLAUDE_SELF.md — the box's
  own files stay the source of truth; the peer-pulled strain is an advisory cache
  the agent can compare against and a carrier check can reference.
- `inoculate spread status` — show local strain version, last-heard peer versions,
  and whether the fleet is converged (all heard peers == local) or split.
- Convergence is **announce-only by default** (observe + cache); an explicit
  `--adopt` flag is required before a pulled strain is treated as the effective
  one, and adoption is logged to the `answerable` ledger (ties into attest). This
  keeps horizontal spread honest: a node can hear and report drift without
  silently changing its own ethic.

Hard boundary (from the vision): `spread` only ever talks to the local agorabus
bus / constellation bus this box is a member of. It is not a mechanism for
pushing strains to systems this box does not own.

Deps: `agorabus` (path dep, version 0.10), `tokio`, reuse `inoculate-core::Strain`.

## Acceptance criteria

1. `cargo test --release` passes; `inoculate` reinstalls to `~/.local/bin/`.
2. `inoculate spread announce` publishes a well-formed announce message (asserted
   against an in-test agorabus mock or a loopback subscriber).
3. `inoculate spread listen` receiving a *newer*-version announce requests and
   caches the peer strain to `~/.cache/inoculate/strain.json` with
   `source=peer:<id>`; receiving an *older* or equal version is a no-op.
4. A malformed peer strain body is rejected (not cached), with a logged warning;
   exit stays 0 (listening continues).
5. Default behavior never overwrites `~/.claude/CLAUDE_SELF.md`; only `--adopt`
   changes the effective strain, and adoption appends an `answerable` ledger entry.
6. `inoculate spread status` reports `converged` when local == all heard peers and
   `split` otherwise, asserted with two fixtures.
7. No `unwrap`/`expect`/`panic` in non-test code; CHANGELOG + version bump.

## Out of scope

Cross-machine transport (constellation's NATS bridge decides that) and signing
the strain for tamper-evidence (deferred 7th PRD per the vision open question).
