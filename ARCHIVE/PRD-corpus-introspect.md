# PRD: corpus-introspect — the self sees itself whole

Status: Draft v0.1
build_target: rust-cli
Vision: visions/corpus.md
deferred_acs: [6]

## TL;DR

The problem: even with attestation, a federated roster, convergence, and an
arbiter, the self has no single place to *look at all of itself at once*. The
answer to "what is the whole of me right now?" is scattered across four tools and
two machines. `corpus-introspect` is the capstone: one command that synthesizes
attested membership, per-node activity, self-state convergence, and held leases —
plus link health — into a single human-readable picture of the entire entity. It
is the multinode self's mirror.

## Why this exists

- The whole point of the `corpus` vision is *one entity, not several*. attest,
  roster, converge, and arbiter each produce a *facet*; without a unifier, the
  self still can't answer the seed's question ("yourself as a multinode entity")
  in one breath — it can only answer four sub-questions on whichever node you
  happen to be typing on.
- The self-review loop already wants a single "state of me" summary and currently
  assembles it per-host by hand (the recall reflective self-review notes are
  exactly this, done manually, for one machine). corpus-introspect is the
  multinode generalization of that summary, machine-readable for the playbook.
- Every input already exists as a corpus component with a `--json` surface, so
  this PRD is *synthesis*, not new sensing — which is why it's the capstone and
  the smallest-risk of the five.

## What this builds

A `corpus-introspect` binary (single crate, `~/wintermute/corpus-introspect`):

- **Synthesis.** `corpus introspect [--json]` calls (via subprocess of the
  installed corpus/tether CLIs, or their libs where exposed): `corpus-attest`
  (this node's membership + the attested-node set), `muster --fleet`
  (per-node live sessions), `corpus-converge version`/`fresh?` (sync state +
  per-node lag), `corpus-arbiter status` (held leases), and `wm-tether status`
  (link health + RTT per node). It assembles a `WholeSelf` record:
  `{nodes:[{node, attested, link, sessions, version_lag}], leases:[…],
  converged:bool, generated_ts}`.
- **Human view.** The default text output reads as a short self-portrait: how
  many nodes constitute the self right now, which are attested + linked, what
  each is doing (session counts + notable verdicts from roster), whether memory
  is converged or some node is behind, and what is currently locked. One screen.
- **Graceful degradation.** Any missing input (a CLI not installed, the link
  down, no authority) is reported as an explicit `unknown`/`degraded` facet, NOT
  silently omitted — the self-portrait never claims completeness it doesn't have
  (the `persona`/`tether` honesty discipline, applied to introspection itself).
- **selfreview emit.** `corpus introspect --format selfreview` emits the block
  the self-review playbook can paste, so the whole-self view feeds the existing
  review loop.

Deps: `clap`, `serde`/`serde_json`, `sigpipe`. MSRV 1.85, edition 2021.
`sigpipe::reset()` first in `main()`. Tests stub each upstream CLI with a fixture
shim on PATH returning known JSON, so synthesis + degradation are verified without
a live fleet.

## Acceptance criteria

1. Given fixture shims for attest/roster/converge/arbiter/tether on PATH returning
   known JSON, `corpus introspect --json` emits a `WholeSelf` record whose
   `nodes`, `leases`, and `converged` fields correctly reflect the fixtures
   (golden-compared).
2. The text view renders, for the multi-node fixture, a self-portrait naming each
   node, its attested+link status, its session count, and the converged/lagging
   verdict — on a bounded number of lines (snapshot-tested).
3. Graceful degradation: when one upstream shim is absent (e.g. no `wm-tether` on
   PATH), that facet is reported as `degraded`/`unknown` with a reason and the
   command still exits success; the missing facet is never silently dropped.
4. When NO corpus components are installed/configured, `corpus introspect`
   reports a single-node self (just this box, `unattested`, no fleet) honestly,
   rather than erroring — the lone-laptop case reads correctly.
5. `--format selfreview` emits a parseable block containing the node count,
   converged verdict, and held-lease count, suitable for the self-review playbook
   (structural assert).
6. (deferred — embedded NATS / live fleet) Run on a real two-node fleet,
   `corpus introspect` lists both nodes with live link RTT and real session
   counts — verified after the upstream corpus components are installed on both.
7. `cargo test` green; `sigpipe::reset()` first in `main()` (grep-asserted);
   `corpus introspect | head` does not panic (no SIGPIPE).
