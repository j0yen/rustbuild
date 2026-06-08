# PRD: lattice-ground — deeply understand by grounding language in the lattice

Status: Draft v0.1
build_target: rust-cli
Vision: visions/lattice.md

## TL;DR

The capstone of the seed — "deeply understand the world around them."
`lattice-ground` maps a natural-language mention ("a feverish patient," "a
central bank," "a sentient robot") to its best-fitting ontological class across
the *entire* federated lattice, and returns that class with its axiom + bridge
neighborhood. The AI stops guessing what a word means and instead grounds it in
a formally-defined, BFO-anchored, ethically-connected concept.

## Why this exists

- **Grounding is what turns retrieval into understanding.** `lattice-context`
  fetches the region around a *known* entity; ground answers the prior question —
  *which* entity does this language refer to? Without it, an AI can query the
  lattice only when it already knows the IRI. ground is the natural-language door.
- **The federated lattice makes grounding deep, not shallow.** Resolving "a
  feverish patient" to `disease:FebrilePatient`, then — via bridges — to
  `wo:SentientBeing` (hence `wo:Dignity`) and `wo:WelfareCondition`, means the
  grounding carries both the *factual* domain meaning and the *ethical* World-
  Ontology consequences in one shot. This is the §9.2 healthcare-ethics linkage
  realized as a single grounding call, and the precise sense in which the ousia
  ethics skill becomes "infinitely wise": every grounded mention arrives pre-
  connected to the dignity/welfare/justice structure.
- **It is the bridge back to the conscience skill.** `herald-conscience` /
  `ousia-guard` evaluate actions described over ontology classes; ground is what
  lets a user describe an action in plain language and have it resolved to the
  right classes before guard reasons over it.

## What this builds

A `cargo` Rust CLI + lib `lattice-ground` at `~/wintermute/lattice-ground/`.

**Deps (pinned at build time):** `lattice-context` (lib), `lattice-bridge` (lib —
to follow equivalences from the grounded class), `ousia-reason` (lib —
materialized types), an embedding/lexical matcher (reuse `recall`'s BGE model +
`strsim`), `serde_json`, `clap`.

**Grounding flow.**
1. Candidate generation — lexical + embedding match of the mention against class
   labels / synonyms / definitions across all lattice ontologies.
2. Disambiguation — rank candidates by lexical+embedding score, BFO-category
   plausibility, and neighborhood coherence with any context terms supplied.
3. Expansion — for the top class, follow bridge equivalences and is_a to surface
   the cross-ontology consequences (e.g. domain class → `wo:SentientBeing` →
   `wo:Dignity`).
4. Return — the grounded class, alternatives with scores, and the connected
   axiom/bridge neighborhood (via lattice-context).

**UX.**
```
lattice-ground resolve --store ... "a feverish patient"          # → ranked classes + ethical links
lattice-ground resolve --store ... "a central bank" --context "monetary policy"
lattice-ground link    --store ... --class disease:FebrilePatient  # just the bridge consequences
```

## Acceptance criteria

1. `resolve "<mention>"` returns a ranked list of candidate ontological classes
   across the federated lattice, each with a score and its source ontology.
2. The top result carries its BFO category and its bridge/is_a neighborhood,
   including cross-ontology consequences (a fixture where a domain `Patient`
   class bridges to `wo:SentientBeing` must surface `wo:Dignity` in the grounded
   result).
3. `--context <terms>` disambiguates polysemous mentions: a mention with two
   plausible classes resolves to the one whose neighborhood best matches the
   supplied context terms (verified on a fixture with a deliberate ambiguity).
4. `link --class <IRI>` returns just the bridge + subsumption consequences of an
   already-known class (the grounding-free path, for callers that have the IRI).
5. Output is structured JSON consumable by `ousia-guard` (so a plain-language
   action can be grounded to classes, then ethically evaluated) — the
   contract back to the conscience skill is documented and shape-tested.
6. A mention with no acceptable match returns an explicit "ungrounded" result
   (empty/low-confidence) rather than forcing a spurious class — honesty over
   false precision.
7. Grounding is deterministic given a fixed store + model, and read-only.
8. `cargo test` covers: ranked resolution on a fixture; the bridge-consequence
   surfacing (patient→dignity); context-based disambiguation; the ungrounded
   case; and the ousia-guard output-shape contract.
