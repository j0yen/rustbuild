# PRD: rosetta-shacl — the ethical rule battery as portable W3C SHACL shapes

Status: Draft v0.1
build_target: rust-cli
Vision: visions/rosetta.md

## TL;DR

`ousia-guard`'s ethical rule battery — `dignity-floor`, `rights-violation`,
`unaccountable-authority`, `welfare-undermine` — is **deductive Rust, and only
Rust**. A third party who wants to check "does this action graph violate the
dignity floor?" must run this box's binary and trust it. `rosetta-shacl`
expresses those four rules as **W3C SHACL shapes** and ships a validator that
runs them over an RDF action-graph, emitting a standard `sh:ValidationReport`.
The same ethical constraint, now portable: any SHACL engine can run the shapes,
and the report is itself RDF that joins into the lattice.

## Why this exists

Phase-1 live inspection (2026-06-15):

- `ousia-guard`'s README rule battery table defines four rules with verdicts and
  paper citations (§5.2 dignity-floor → deny, §5.7/§6.6 rights-violation → deny,
  §5.3/§7.2 unaccountable-authority → flag, §5.6/§5.7 welfare-undermine → flag).
  The logic lives in `ousia-guard/src/rules.rs` — a closed Rust implementation.
- The guard is *deductive over OWL*; SHACL is *declarative constraint
  validation*. They are complementary: SHACL gives a portable, engine-agnostic
  second opinion and a standard machine-readable report, where the guard gives a
  reasoned axiom chain. Neither exists in SHACL form today.
- The emitted `sh:ValidationReport` is RDF — it can be joined into the
  `oxigraph` lattice store and served by `rosetta-serve`, unifying "what the
  rules say" with "what the reasoner derived."
- `grep` over all PRDs + visions: zero SHACL coverage. Unclaimed.

## What this builds

New repo `~/wintermute/rosetta-shacl/` (rust-cli, edition 2021, MSRV 1.85).

**Deps:** `oxrdf` / `oxrdfio` (match locked ecosystem versions), `clap` v4.
No general SHACL engine crate is mature in Rust — this implements the **subset
of SHACL-core** the four rules need (`sh:NodeShape`, `sh:targetClass`,
`sh:property`, `sh:path`, `sh:minCount`/`sh:hasValue`/`sh:not`, severity), not a
general engine. This is an explicit, documented scope boundary.

**Artifacts:**

1. `shapes/ethics-shapes.ttl` — the four rules as `sh:NodeShape`s over the World
   Ontology vocabulary. E.g. dignity-floor: a shape targeting actions that
   `wo:harms` a `wo:SentientBeing` with `sh:severity sh:Violation`;
   unaccountable-authority: a shape requiring any `wo:AuthorityRole` bearer to
   have an asserted `wo:Accountability` (`sh:minCount 1`) with
   `sh:severity sh:Warning`. Each shape carries an `rdfs:comment` citing the
   same paper section as the guard rule, so the SHACL and the deductive rule are
   traceably the same constraint.

2. The validator CLI:

```
rosetta-shacl validate --data action.ttl [--shapes shapes/ethics-shapes.ttl]
                       [--format turtle|jsonld]            # → sh:ValidationReport
rosetta-shacl shapes                                       # print the bundled shapes
rosetta-shacl rules                                        # list rule→shape→severity→citation
```

`validate` loads the data graph and the shapes, runs the SHACL-core subset, and
emits a conforming `sh:ValidationReport` (`sh:conforms` boolean, plus a
`sh:result` per violation with `sh:focusNode`, `sh:resultPath`,
`sh:resultSeverity`, `sh:sourceShape`, and a `sh:resultMessage`). Exit code is
0 when `sh:conforms true`, 1 when violations of `sh:Violation` severity exist
(warnings alone keep exit 0), 2 on input error.

## Acceptance criteria

1. `rosetta-shacl shapes` emits Turtle that round-trips through `oxrdfio` parse,
   containing exactly four `sh:NodeShape`s, one per guard rule.
2. Each shape carries an `rdfs:comment` citing the same paper section as the
   corresponding `ousia-guard` rule (dignity-floor §5.2, rights-violation §5.7,
   unaccountable-authority §5.3, welfare-undermine §5.6).
3. `validate` on a fixture action-graph that harms a `wo:SentientBeing` returns
   a `sh:ValidationReport` with `sh:conforms false` and a `sh:result` whose
   `sh:sourceShape` is the dignity-floor shape and `sh:resultSeverity`
   `sh:Violation`; exit code 1.
4. `validate` on a clean action-graph returns `sh:conforms true` with no
   `sh:result` nodes; exit code 0.
5. An action granting `wo:AuthorityRole` with no `wo:Accountability` produces a
   result at `sh:Warning` severity, and exit code is 0 (warnings do not fail).
6. `--format jsonld` emits a report that parses as valid JSON and re-parses
   through `oxrdfio` to an isomorphic graph.
7. `rosetta-shacl rules` lists all four rules with shape IRI, severity, and
   citation; exit 0.
8. The README documents the SHACL-core subset implemented and explicitly states
   what SHACL features are out of scope (no `sh:sparql`, no `sh:and`/`sh:or`
   beyond the rules' needs).
