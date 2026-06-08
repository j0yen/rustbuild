# PRD: lattice-bridge — automatically align two BFO-grounded ontologies

Status: Draft v0.1
build_target: rust-cli
Vision: visions/lattice.md

## TL;DR

This is the core capability of the seed — "tools to automatically bridge
ontologies." `lattice-bridge` takes two BFO-grounded ontologies and computes
cross-ontology mappings (equivalence / subsumption between their classes),
anchored by the BFO upper categories they both descend from and refined by
lexical + structural matching. It emits OWL **bridge axioms** with a confidence
score and the evidence for each — gated for review, never silently merged.

## Why this exists

- **The paper promises exactly this and stops at "can."** §9.2: a medical AI
  "could link disease processes to the World Ontology's WelfareCondition…"; §8.2:
  the World Ontology "can serve as a bridge ontology connecting specialized
  domain ontologies." The linking is asserted as possible but no tool computes
  it. bridge is that tool.
- **Shared BFO grounding makes it tractable.** Generic ontology matching (the
  OAEI task) is hard, but two ontologies that both anchor to `BFO:quality`,
  `BFO:process`, `BFO:role` share an upper scaffold that massively prunes the
  candidate-mapping space — a `Disease`-domain class under `BFO:disposition`
  only needs alignment against World-Ontology classes under the same BFO
  category. This is the genuine leverage the lattice vision rests on.
- **Honesty requires confidence + review.** The vision open-question is explicit:
  lexical/structural matching yields false mappings, so bridge must emit
  confidence + evidence and route low-confidence mappings to a review gate
  (mirroring `recall observe` / `skill-doctor` proposals), not auto-merge.

## What this builds

A `cargo` Rust CLI `lattice-bridge` at `~/wintermute/lattice-bridge/`, with a
reusable lib (consumed by lattice-join + lattice-ground).

**Deps (pinned at build time):** `horned-owl`, `ousia-reason` (lib — to resolve
each class's BFO upper category), a string-similarity crate (e.g. `strsim`),
`serde`/`serde_json`, `clap`.

**Matching pipeline (each mapping carries its evidence):**
1. **BFO anchor** — for each class in A and B, resolve its BFO upper category
   via ousia-reason. Only classes under the *same* BFO category are mapping
   candidates (the pruning step).
2. **Lexical** — label / synonym / definition similarity (normalized edit +
   token overlap) within each BFO bucket.
3. **Structural** — agreement of parents/children already mapped (a class whose
   parent maps to B's parent is a stronger candidate).
4. **Score + classify** — combine into a confidence in [0,1]; classify each
   mapping as `equivalent` (high) or `subClassOf` (directional/medium).

**UX.**
```
lattice-bridge align --a world-ontology.owl --b disease.owl --out bridge.owl
lattice-bridge align --a <id> --b <id> --from-registry          # resolve via lattice-registry
lattice-bridge review bridge.proposals.jsonl                    # inspect mappings + evidence
```

Output: `bridge.owl` (accepted high-confidence mappings as OWL
`EquivalentClasses`/`SubClassOf` bridge axioms) **and**
`bridge.proposals.jsonl` (every mapping with confidence + evidence, low ones
held for review).

## Acceptance criteria

1. `align --a <owl> --b <owl>` emits, for two BFO-grounded ontologies, a set of
   candidate mappings each tagged `equivalent | subClassOf`, a confidence in
   [0,1], and the evidence (BFO category, lexical score, structural support).
2. The BFO-anchor prune is enforced: no mapping is ever proposed between two
   classes under different BFO upper categories (verified on a fixture where a
   quality and a process share a label — they must NOT map).
3. High-confidence mappings (≥ threshold, configurable) are written to
   `bridge.owl` as valid OWL bridge axioms re-loadable by horned-owl;
   below-threshold mappings go to `bridge.proposals.jsonl`, NOT into the OWL.
4. `bridge.owl` imports/references both source ontologies' namespaces correctly
   so the axioms are resolvable when joined.
5. `--from-registry` resolves both ontologies via `lattice-registry path`,
   so bridging composes with find.
6. `review` prints each proposed mapping with its evidence in a readable form;
   the proposals file is append-only and stable (re-running align is
   deterministic given the same inputs + threshold).
7. A known-good fixture pair (a small domain ontology + a World-Ontology subset
   sharing a few obvious equivalences, e.g. domain `Person`↔WO `Person`)
   produces the expected equivalence mappings at high confidence.
8. `cargo test` covers: the BFO-anchor prune (cross-category label collision
   does not map), threshold routing (OWL vs proposals), equivalence vs
   subClassOf classification, and deterministic re-run.
