# PRD: quicken-crossdep

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/quicken
Vision: visions/quicken.md

## TL;DR

A "live" primitive can be silently crippled by a dark one. provfs is live but
stamps only a fallback session id *because agentns is inert* — a fact no probe
states; a human has to infer it from two unrelated outputs. `quicken-crossdep`
extends the `quicken` workspace with a small primitive→primitive enablement
graph, so each verdict names what it's `blocked-by` and what reviving a dark
primitive `would-upgrade`. It turns four independent verdicts into a causal map.

## Why this exists

- **Evidence — the degradation chain is real and caught live 2026-06-05.**
  `getfattr -d ~/brain/state/last-run.txt` →
  `user.prov.session="comm:zsh:pid:64758:uid:1000"`. provfs is working, but the
  session id is the `comm:` fallback, not the 128-bit agentns id it's designed
  to record (`~/wintermute/provfs` README + Phase-1.5 kernel notes: "When
  `CONFIG_AGENT_NS=y`, `user.prov.session` carries the agentns 128-bit session
  id; otherwise `comm:<comm>:pid:<n>:uid:<n>`"). Since `/proc/self/agent_session`
  is all-zeros, provfs *cannot* upgrade — its degradation is *caused by* agentns
  being dark.
- **Evidence — without the link, the fix priority is invisible.** Reviving
  agentns silently upgrades provfs provenance across the whole laptop; nothing
  today connects those two findings, so agentns reads as "one more dark thing"
  rather than "the keystone that also fixes provfs."
- **Why a graph, not ad-hoc notes:** the relationships are few but real and will
  grow (memlog group ← sysusers activation; warden ← policy profile). Modeling
  them once lets every future probe inherit `blocked-by`/`would-upgrade`
  annotations for free.

## What this builds

- A `quicken-crossdep` lib module + integration into `quicken probe`'s output
  (a `--deps` flag, plus inclusion in `--json`).
- **Dependency model**: a static `EnablementEdge { from: PrimitiveId, to:
  PrimitiveId, effect: Effect }` set, where `Effect` is
  `EnablesLiveness` (target is inert/blocked until source is live) or
  `UpgradesQuality { from_state, to_state }` (target is live-degraded until
  source is live). Seeded with the canonical edge: `agentns --UpgradesQuality{
  comm-fallback → 128bit-session }--> provfs`.
- **Annotation pass**: given the current `Vec<PrimitiveReport>` + the edge set,
  derive per-primitive `blocked_by: Vec<PrimitiveId>` (sources that are not live)
  and, for live sources, `would_upgrade: Vec<PrimitiveId>` (targets a revival
  would improve). A `LiveDegraded` report whose degrading source is identified
  gets its `reason` enriched with the causing primitive.
- **CLI**: `quicken probe --deps` prints the verdict table with a `blocked-by` /
  `would-upgrade` column; `quicken probe --json` includes the annotations.
  Optionally `quicken deps` prints the static edge set (for inspection).

## Acceptance criteria

1. `quicken probe --deps --help` (or `quicken deps --help`) documents the
   dependency view.
2. The seeded edge set contains the `agentns → provfs`
   `UpgradesQuality{comm-fallback → 128bit-session}` edge, asserted directly.
3. Given a fixture report set where agentns is `Inert` and provfs is
   `LiveDegraded`, the annotation pass marks provfs `blocked_by` /
   degraded-because including `agentns`, and annotates agentns with
   `would_upgrade` including `provfs` (asserted on the fixture).
4. Given a fixture where agentns is `Live`, provfs's `would_upgrade`/`blocked_by`
   no longer references it and provfs is not reported as agentns-degraded
   (asserted — the edge only fires while the source is dark).
5. `quicken probe --deps --json` includes the `blocked_by` and `would_upgrade`
   fields and round-trips into the annotated report type.
6. The annotation pass is a pure function of (reports, edges) with no I/O, and a
   cycle in the edge set is handled without infinite loop or panic (defensive
   test with a deliberately cyclic fixture edge set).
7. Tests perform **zero network and zero filesystem writes outside the test
   tmpdir** (cloud-build-safe). The integration test entry file (e.g.
   `tests/crossdep.rs`) must appear in cargo output (`Running tests/crossdep.rs`)
   per the orphaned-mock-tests guard.
