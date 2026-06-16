# Vision: rosetta — make this box's ethical decisions speak the semantic web

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-15
**Status:** active
**Seed:** user — `/dream semantic web tools for ethical AI`. Phase-1 live
inspection turned the abstract topic concrete: this laptop already has a deep
*reasoning* arc for ethical AI (`ousia` BFO/OWL/SPARQL reasoner → `lattice`
federation → `ousia-guard` deductive gate → `tribunal` verification →
`herald` distribution → `inoculate` immune system → `answerable` ledger). What
it does **not** have is the *interoperability* arc: nothing on this box emits
its ethical decisions, justifications, or provenance as **W3C-standard linked
data** — PROV-O, SHACL, JSON-LD, Verifiable Credentials, dereferenceable IRIs.
The reasoning is locked inside bespoke Rust structs and JSONL.

## TL;DR

*A Rosetta stone translates one inscription into the languages everyone reads.*
The ousia/lattice arc has produced a working ethical reasoner whose outputs —
`ousia-guard`'s `{verdict, rules_fired, justifications}` (gate.rs:51), provfs's
kernel-stamped `user.prov.*` xattrs, `answerable`'s JSONL ledger, `inoculate`'s
signed strains — are each in a private dialect. The semantic web is the lingua
franca that would make those decisions **portable, queryable, third-party
auditable, and joinable into the lattice graph**. `rosetta` is the fleet of
tools that translates this box's ethical artifacts into the four standard
vocabularies the W3C built for exactly this:

- **PROV-O** (W3C provenance) — *why* an action was allowed/denied, as a graph.
- **SHACL** (W3C shapes) — the ethical rule battery as portable, engine-agnostic
  constraints emitting a standard `sh:ValidationReport`.
- **Verifiable Credentials** (W3C, JSON-LD) — a *signed* ethical clearance any
  third party can verify without trusting this box.
- **Linked Data / dereferenceable IRIs + SPARQL endpoint** — make
  `wo:SentientBeing`, `wo:Dignity` and the emitted decision graphs *addressable*
  by any semantic-web client.

This is the honest semantic-web contribution to ethical AI: not new ethics, but
making the ethics this box already reasons about **interoperable and provable
to the outside world**.

## Why now (Phase-1 evidence)

- `ousia-guard/src/gate.rs:51` produces `Evaluation { verdict, rules_fired,
  justifications: Vec<RuleFiring> }` — a rich axiom-chain justification that
  dies as a Rust struct / one-shot JSON. Never serialized as PROV-O.
- `ousia-guard`'s README rule battery (dignity-floor, rights-violation,
  unaccountable-authority, welfare-undermine) is **deductive Rust only** — no
  portable SHACL form a non-Rust verifier could run.
- `provenance-mcp` already exposes provfs `user.prov.session` / `user.prov.ts`
  xattrs read-only, and `answerable` is a JSONL audit trail with a session id —
  but neither is RDF; they cannot be joined to a guard verdict in one graph.
- `inoculate/src/signet.rs` already signs strains (provenance-signed strains,
  v0.5.0) — the signing primitive for Verifiable Credentials exists; nothing
  wraps a *decision* in it.
- `lattice-join` already runs an `oxigraph 0.4` store; the World Ontology IRIs
  are **not dereferenceable** and there is no SPARQL *endpoint* — only a CLI.
- `grep` over all PRDs + visions: **zero** existing coverage of PROV-O / SHACL /
  JSON-LD / Verifiable-Credential / dereferenceable-IRI (the one homeward hit is
  schema.org product markup, unrelated). The interoperability layer is unclaimed.

## End-state

When this vision is fulfilled:

1. Every `ousia-guard` verdict can be emitted as a **PROV-O graph** (Turtle +
   JSON-LD): the guard check is a `prov:Activity`, the verdict an `prov:Entity`
   `prov:wasGeneratedBy` it, `prov:used` the ontology version, and
   `prov:wasAssociatedWith` the agentns/provfs session id — joining the *why*
   (axiom chain), the *what* (action), and the *who/when* (kernel provenance)
   into one queryable graph.
2. The ousia rule battery exists as **W3C SHACL shapes**; any SHACL engine —
   not just ousia's Rust — can validate an action-graph and emit a standard
   `sh:ValidationReport`. A second, declarative, portable opinion alongside the
   deductive guard.
3. A guard verdict + its PROV-O graph can be **signed as a W3C Verifiable
   Credential** (JSON-LD), so an ethical clearance travels off-box as a
   cryptographically verifiable claim — leaning on `inoculate-signet`'s keys.
4. The joined lattice + emitted decision graphs are served over a **read-only
   SPARQL endpoint with dereferenceable IRIs** (content-negotiated
   Turtle/JSON-LD/HTML) — the World Ontology becomes addressable linked data.
5. An end-to-end **attestation** drives a real action through
   ground → guard → PROV-O → SHACL → VC → serve → SPARQL-back and proves the
   chain composes and round-trips, with a committed receipt (the
   `continuity-attest` pattern).

## Components (PRD-sized)

1. **rosetta-prov** (`PRD-rosetta-prov`) — new repo `~/wintermute/rosetta-prov/`
   (rust-cli). Translate an `ousia-guard` `Evaluation` (+ action context +
   provfs/answerable provenance) into a W3C **PROV-O** graph, emit Turtle and
   JSON-LD via `oxrdf`/`oxrdfio`. *Foundational — every other component consumes
   or serves these graphs.*
2. **rosetta-shacl** (`PRD-rosetta-shacl`) — new repo (rust-cli). Author the
   ousia rule battery as **SHACL shapes** + a validator that runs them over an
   action-graph and emits a standard `sh:ValidationReport` (Turtle/JSON-LD).
   Independent of rosetta-prov; depends conceptually on the ontology vocabulary.
3. **rosetta-credential** (`PRD-rosetta-credential`) — new repo (rust-cli). Wrap
   a guard verdict + its PROV-O graph as a W3C **Verifiable Credential**
   (JSON-LD), signed with the `inoculate-signet` key; `verify` re-checks the
   signature and the embedded claim. Depends on rosetta-prov.
4. **rosetta-serve** (`PRD-rosetta-serve`) — new repo (rust-cli, axum). A local
   **linked-data + SPARQL endpoint** over the joined lattice store + emitted
   decision graphs: dereferenceable IRIs with content negotiation
   (Turtle/JSON-LD/HTML) and a read-only SPARQL 1.1 query endpoint. Depends on
   rosetta-prov (to serve the graphs) + lattice-join (the store).
5. **rosetta-attest** (`PRD-rosetta-attest`) — new repo (rust-cli). The capstone:
   drive a real action through the whole chain and assert it composes and
   round-trips, with a committed receipt. The `continuity-attest` pattern
   applied to the rosetta chain. Depends on prov + shacl + credential + serve.

## Order

```
rosetta-prov  ──┬─→ rosetta-credential ──┐
                │                          ├─→ rosetta-attest
rosetta-shacl ──┤                          │
                └─→ rosetta-serve  ────────┘
```

rosetta-prov is foundational. rosetta-shacl is independent and can build in
parallel. rosetta-credential and rosetta-serve both depend on rosetta-prov.
rosetta-attest is the capstone and waits for all four.

## Open questions (for the user / next /dream pass)

- **VC suite**: W3C VC Data Integrity with an Ed25519 cryptosuite (`eddsa-rdfc-
  2022`) is the right target, but full RDF Dataset Canonicalization (RDFC-1.0)
  is heavy. v1 may sign a stable JSON-LD serialization (documented as a
  simplification) and leave true RDFC-1.0 canonicalization as a follow-on PRD.
- **Outward serving**: rosetta-serve is local-only (127.0.0.1). Publishing
  dereferenceable IRIs on a *public* domain is a constellation/herald concern
  (hosting + stable IRI policy) — deferred, not in this fleet.
- **SHACL engine**: no mature pure-Rust SHACL engine exists; rosetta-shacl
  likely implements the subset of SHACL-core needed for the four rules rather
  than a general engine. Scope it to the rule battery, not all of SHACL.
- **Federated SPARQL** (`SERVICE` to remote OBO/CCO endpoints) is a natural
  lattice extension but needs live remote endpoints — left as a future
  `/dream extend lattice` bullet, not a rosetta PRD.

## Relationship to neighboring visions

- **ousia / lattice** produce the reasoning rosetta translates; rosetta is
  strictly downstream and never mutates the ontology.
- **answerable / inoculate** provide the ledger and signing keys rosetta reuses;
  rosetta-credential is the linked-data face of an answerable+signet attestation.
- **tribunal / warrant** verify *internal* claims; rosetta makes claims
  verifiable *externally*, by any W3C-compliant tool.
- **herald** could one day distribute rosetta as an installable skill — out of
  scope for this fleet.
