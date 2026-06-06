# PRD: constellation-headscale — own the mesh control plane on the cloud node

Status: Draft v0.1
build_target: shell
Vision: visions/constellation.md
Depends: PRD-constellation-mesh.md (the enrollment client already exposes the
  Headscale flag; this PRD stands up the server it points at),
  PRD-constellation-secrets.md (the Headscale server key + issued pre-auth keys are
  managed secrets), PRD-constellation-cloud.md (the host this runs on).

## TL;DR

mesh ships the *client* half of the sovereignty path — `tailscale up` can be
flagged to point at a self-hosted control server — but nothing **stands up that
server**. `constellation-headscale` provisions and operates a **Headscale** control
plane on the always-on cloud node: persistent state, an ACL/policy that mirrors
mesh's tailnet policy, pre-auth key issuance for enrollment, and a health surface —
so the fleet's coordination plane is owned end-to-end instead of depending on
Tailscale's SaaS coordinator. It is the server that makes mesh's documented flag a
real, selectable reality.

## Why this exists

The vision's second open question, and the explicit seam mesh left:

> **Mesh control plane sovereignty.** Tailscale (easiest) uses a third-party control
> plane; **Headscale** self-hosted on the cloud node keeps the same UX while owning
> the control. Which matters more — ops simplicity or sovereignty?
> — `visions/constellation.md`, Open questions

- **mesh stops at "documented + scripted-optional."** `PRD-constellation-mesh.md`
  AC9: *"The Headscale (self-hosted control) path is at least documented and
  selectable by a flag, so the sovereignty option is real, not hypothetical."* §What
  this builds calls it *"a flag to point enrollment at a self-hosted Headscale on the
  cloud node."* The **server side** — installing Headscale, persisting its DB,
  issuing the keys the flag consumes, keeping it up — is named nowhere and built
  nowhere. A flag that points at an absent server is not yet the sovereignty path.
- **the cloud node is already the designated home.** The vision's fleet table makes
  the cloud node the *"always-on hub (NATS + mesh exit)"*; `constellation-cloud`
  provisions it. Headscale belongs on exactly that host — co-located with the NATS
  hub it coordinates access to.
- **the secret layer this needs now exists.** Headscale's server private key and the
  pre-auth keys it issues are precisely the "service secrets" `constellation-secrets`
  manages — so the server can be stood up without any plaintext key in the repo,
  closing the loop mesh AC1/AC10 require.
- **sovereignty is a stated user value.** The fleet seed and the user's broader
  posture (private repos by default, local-first brain ladder, owning the kernel)
  point at owning the control plane rather than renting it; this PRD makes that the
  real default-available path, not a deferred note.

## What this builds

A `constellation/headscale/` Ansible role + a thin `constellation headscale` helper,
targeting the cloud node:

- **Server install + persistence** — Headscale as a `systemd` service on the cloud
  node, listening **mesh-only** (bound to the tailnet/loopback + fronted per mesh
  ACL, never a raw public port beyond what DERP/registration strictly needs),
  with its SQLite/Postgres state on a persisted path that survives a cloud-node
  rebuild (documented backup of the state + server key).
- **Server key from the store** — the Headscale private/noise key is pulled from
  `constellation-secrets` at provision, never generated-into-the-repo; rotating it is
  a documented `secrets` operation.
- **Policy parity with mesh** — the Headscale ACL/policy is generated from (or
  asserted equal to) the **same committed tailnet ACL** mesh defines, so switching
  control planes does not change who-can-reach-what (bus/brain ports stay fleet-tag
  only). One source of ACL truth, two backends.
- **Pre-auth key issuance** — `constellation headscale preauth --role <tag>` mints a
  (reusable/ephemeral, tagged, expiring) pre-auth key and writes it **into the
  secret store** for mesh enrollment to consume — closing mesh AC1 against the
  self-hosted plane.
- **Node lifecycle** — register / list / expire / delete a node by name; a stale or
  decommissioned node can be removed so the roster doesn't drift.
- **Health + selftest** — `constellation headscale status` reports the server up,
  the DB reachable, and the registered-node roster; `… selftest` registers a throwaway
  ephemeral node against the local server and confirms it appears, then expires it.
- **The switch** — flipping mesh's enrollment flag to this server is a documented,
  reversible operation (Tailscale SaaS ↔ self-hosted Headscale) with the trade-offs
  (ops burden vs sovereignty) written down so the choice stays the user's.

Non-goals: client enrollment (mesh owns `tailscale up`); the ACL *content* (mesh
defines it — this consumes it); secret storage mechanics (secrets owns the store);
NATS/bus (bus). This PRD delivers the **operating control server + its keys +
policy parity + node lifecycle** only.

## Acceptance criteria

1. An Ansible role installs Headscale as a `systemd` service on the cloud-node host
   with state on a persisted path; after a simulated service restart the registered
   node roster survives (state is durable, asserted).
2. The Headscale server key is loaded from `constellation-secrets` (not generated
   into the repo); the repo contains no Headscale private key (grep-asserted, reusing
   the secrets audit gate).
3. The Headscale ACL/policy is derived from or asserted byte-equivalent to mesh's
   committed tailnet ACL, so bus/brain ports remain reachable only from fleet tags
   under the self-hosted plane (asserted against the shared policy file).
4. `constellation headscale preauth --role <tag>` mints a tagged, expiring pre-auth
   key and stores it (encrypted) where mesh enrollment reads it; a node enrolled with
   that key joins with the correct tag (proven against a local Headscale, or a
   documented reproducible test).
5. `constellation headscale status` reports server-up + DB-reachable + the node
   roster, exiting non-zero if the server is down or the DB is unreachable.
6. `constellation headscale selftest` registers an ephemeral throwaway node against
   the server, confirms it appears in the roster, and expires it — leaving the roster
   unchanged (idempotent, no residue).
7. Node lifecycle commands (register/list/expire/delete by name) work and a deleted
   node no longer appears in the roster (asserted).
8. Switching mesh enrollment between Tailscale SaaS and the self-hosted Headscale is
   documented as a reversible operation with the sovereignty-vs-ops trade-off written
   down; the flag flip is demonstrated against the local server (or a documented
   reproducible test).
9. The server listens mesh-only / on the minimum surface registration+DERP require;
   a probe of any unnecessary public port is refused (asserted against the bind +
   firewall/ACL config).
10. `sigpipe::reset()` (or the shell equivalent) guards `constellation headscale …`
    output against `SIGPIPE` panics (`self_sigpipe_panic_toolkit`); no Headscale key
    or pre-auth key plaintext is ever committed.
