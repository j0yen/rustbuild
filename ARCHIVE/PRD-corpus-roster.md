# PRD: corpus-roster — what is all of me doing right now

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/muster
Vision: visions/corpus.md
version_bump: minor
deferred_acs: [6]

## TL;DR

The problem: `muster` answers "which `claude` sessions are running" — but only on
*this* machine. A multinode self needs that question answered for the *whole
entity*: every live session on every attested node, in one roster. `corpus-roster`
extends muster with a `--fleet` view that aggregates each node's local roster over
the bus, filtered to attested nodes, so the self (and the self-review) can see all
of itself at once instead of guessing per-host.

## Why this exists

- `muster`'s own vision scopes it to *"which `claude` processes are running right
  now"* on **this** box: census enumerates local procs, verdict classifies them
  (live/duplicate/orphan/stale). That foundation is exactly what a fleet roster
  reuses — extend, don't fork.
- The recall reflective notes repeatedly answer the session-population question
  per-machine ("5 live claude sessions concurrent…", "duplicate Claude sessions …
  may both be"). With a second node (tether), per-host counting under-reports the
  self by construction.
- `corpus-attest` defines who legitimately *is* the self; a fleet roster must
  show only attested nodes, or it would list any bus participant as "me" —
  membership and roster compose directly.
- `wm-busbridge` forwards `wm.fleet.>`, so `wm.fleet.muster.*` crosses for free
  once tether-link is up; the only new work is request/aggregate/merge.

## What this builds

A minor-version extension of the `muster` crate (extend, do not replace):

- **Responder.** Each node answers `wm.fleet.muster.request` by running its own
  local census+verdict (the existing muster code path) and replying on
  `wm.fleet.muster.roster.<req_id>` with `{node, attestation, entries:[…]}` where
  each entry carries muster's existing fields (pid, origin, verdict, evidence).
- **Aggregator.** `muster --fleet` publishes a request, collects replies within a
  bounded window, verifies each responder's attestation via `corpus-attest verify`
  (drops unattested responders with a logged reason), and merges the entries into
  one roster keyed by `(node, pid)`. Local entries are included exactly as
  `muster` (no flag) would show them.
- **Output.** Text view groups entries by node with a per-node header (node name,
  attestation status, session count); `--format json` emits the federated roster.
  A `--fleet --format selfreview` mode extends the existing selfreview emit to the
  fleet shape so the self-review playbook can consume it.
- **Read-only.** corpus-roster never terminates anything, local or remote;
  cross-node reap is explicitly out of scope (vision OQ#4). The existing
  `muster-reap` remains local-only and reviewer-gated.

No new binary; this is `muster` gaining a `--fleet` flag, a responder, and an
aggregator. Tests use an embedded/test NATS server for the fleet half and
muster's existing in-process census harness for the local half.

## Acceptance criteria

1. `muster` (no `--fleet`) produces byte-identical output to the pre-change
   version for a fixed local process set (golden-compared) — the federation path
   is strictly additive and never alters local behavior.
2. `muster --fleet` with the fleet transport absent (no NATS) still returns the
   local roster and exits success — it never blocks or fails on the missing
   fleet path (degrades to local).
3. A `wm.fleet.muster.request` triggers a responder reply containing this node's
   attestation and its local entries with muster's existing per-entry fields
   (pid, origin, verdict, evidence) — asserted against embedded NATS.
4. `muster --fleet` merges a remote responder's entries into the roster, grouped
   under the responder's `node`, and dedups so the same `(node, pid)` appears
   once even if the reply is delivered twice.
5. A responder whose attestation fails `corpus-attest verify` is dropped from the
   federated roster with a logged reason; only attested nodes' entries appear.
6. (deferred — embedded NATS) End-to-end: aggregator request → two responders
   (local + one remote stub) → merged roster lists both nodes' sessions within
   the bounded collection window; a slow/absent responder does not hang the
   aggregator (window-bounded).
7. `cargo test --workspace` green; existing muster tests pass unchanged (no
   regression in census/verdict/reap).
