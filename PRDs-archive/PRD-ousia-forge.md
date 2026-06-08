# PRD: ousia-forge — build the World Ontology from a declarative spec

Status: Draft v0.1
build_target: rust-cli
build_priority: high
Vision: visions/ousia.md

## TL;DR

The World Ontology paper (`~/Notes/AtScale/World-Ontology-paper.md`) describes a
509-class OWL 2 DL ontology in exhaustive detail — but **the `.owl` file does
not exist anywhere on disk** (`find /home/jsy -iname '*world-ontology*'` returns
only the paper). `ousia-forge` is a Rust CLI that builds `world-ontology.owl`
(OWL 2 DL / RDF/XML, importing BFO 2020) from a declarative, human-auditable
TOML/YAML spec — so the ontology is reproducible, diffable, and re-generatable
rather than hand-maintained XML.

## Why this exists

- **The artifact is missing.** Phase-1 research (2026-06-08) confirmed: the
  paper exists, the ontology does not. Every downstream ousia tool
  (reason/sparql/guard/mcp/atscale) needs an `.owl` to operate on. forge is the
  gate.
- **Hand-edited OWL/XML is unmaintainable.** The paper reports 3,524 lines of
  RDF/XML across 509 classes, 37 properties, 21 restrictions, 6 defined classes
  (§4.1). Authoring that by hand is error-prone and undiffable. A declarative
  spec compiled by `horned-owl` makes class/axiom changes reviewable as small
  source edits.
- **Reproducibility is the paper's own claim.** §8.1 asserts XML
  well-formedness and Protege loadability as validation gates; a forge that
  emits a byte-stable artifact makes those gates a CI check, not a manual step.

## What this builds

A `cargo` Rust CLI crate `ousia-forge` at `~/wintermute/ousia-forge/`.

**Deps (versions pinned at build time):**
- `horned-owl` — OWL 2 ontology construction + RDF/XML serialization.
- `serde` + `toml` (and/or `serde_yaml`) — parse the declarative spec.
- `clap` — CLI.

**Spec format.** A directory of TOML files, one per domain (the paper's eight
domains, §4.2): `physical.toml`, `organisms.toml`, `artifacts.toml`,
`qualities.toml`, `dispositions.toml`, `roles.toml`, `information.toml`,
`processes.toml`. Each declares classes (IRI suffix, label, single BFO parent,
Aristotelian definition, annotations) and axioms (all-some SubClassOf
restrictions, equivalence axioms for the 6 defined classes). A top-level
`ontology.toml` declares namespaces (`https://w3id.org/world-ontology/`, BFO
PURL `http://purl.obolibrary.org/obo/`), imports, and the 7 custom annotation
properties (§4.4).

**UX.**
```
ousia-forge build   --spec spec/ --out world-ontology.owl   # emit RDF/XML
ousia-forge check   --spec spec/                            # validate spec only
ousia-forge stats   --out world-ontology.owl                # class/prop/axiom counts
```

**Seed content.** The spec ships with at minimum the philosophically load-bearing
classes and all 10 axioms from the paper §5/§6 (Sentience, SentientBeing,
Dignity, Person, Agent, AuthorityRole, Accountability, CognitiveCapacity,
AffectiveCapacity, Error, Learning, Flourishing, WelfareCondition,
InjusticeProcess, DueProcessProceeding, Right, Justice, Organization,
GovernanceStructure, JustSociety, UnjustSystem). Full 509-class breadth is an
incremental fill (the spec is additive); the forge must build a valid ontology
from a partial spec.

## Acceptance criteria

1. `ousia-forge build --spec spec/ --out world-ontology.owl` produces a file
   that parses as well-formed XML and re-loads in `horned-owl` without error.
2. The output declares `owl:imports` for BFO 2020 via the OBO PURL namespace,
   and World Ontology classes use the `https://w3id.org/world-ontology/`
   namespace with PascalCase class / camelCase property names (paper §4.1).
3. The 6 defined classes (SentientBeing, Person, Agent, Organization,
   JustSociety, UnjustSystem) are emitted with OWL **equivalence** axioms
   exactly matching the paper §6 (e.g. `SentientBeing ≡ BFO:object ∧ bearer_of
   some Sentience`).
4. The 10 foundational axioms (§5.1–5.7, §8.3 table) are emitted as the stated
   SubClassOf all-some restrictions — verifiable by `stats` reporting ≥10
   `SubClassOf` restriction axioms and by a golden-triple test for the
   sentience→dignity and authority→accountability axioms.
5. `ousia-forge check` rejects a spec with a class that has zero or ≥2 asserted
   parents (enforces the paper's single-inheritance design principle, §7.2),
   with a non-zero exit and a message naming the offending class.
6. `build` is deterministic: running it twice on the same spec yields
   byte-identical output (stable IRI ordering, no timestamps in the artifact).
7. `ousia-forge stats --out world-ontology.owl` prints named-class,
   object-property, defined-class, and restriction-axiom counts.
8. `cargo test` covers: a minimal 3-class spec round-trips; the
   sentience→dignity axiom chain is present in output; a 2-parent spec fails
   `check`.
