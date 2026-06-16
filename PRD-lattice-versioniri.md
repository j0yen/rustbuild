# PRD: lattice-versioniri — fix the OWL/XML versionIRI parse bug blocking all federation

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/lattice-bridge
Vision: visions/lattice.md

## TL;DR

`lattice-bridge align` fails on every real OBO Foundry ontology with:

```
error: OWL parse error: Validity Error: Unexpected tag: found versionIRI at <N> at Unknown
```

Every OBO Foundry OWL/XML file includes `<owl:versionIRI rdf:resource="…"/>` inside
the `<owl:Ontology>` element — this is valid OWL 2 DL (the spec requires parsers to
tolerate it). The lattice-bridge OWL/XML parser treats it as an unexpected tag and
aborts. Tested live (2026-06-16): BFO, IAO, RO, COB, SWO all fail with this error.
The entire bridge capability is dead against any real external ontology until this is
fixed.

## Why this exists

Live test (2026-06-16): all five priority ontologies fetched successfully via
`lattice-registry add` (bfo_grounded=true for all), but every attempt at
`lattice-bridge align --a <any-fetched.owl> --b <any-fetched.owl>` fails immediately.
Investigation shows the error fires on `versionIRI` inside the `owl:Ontology` header
block. `owl:versionIRI` is defined in OWL 2 §3.1 as a standard annotation on the
ontology entity itself, not a class or property — parsers must accept and store or skip it.

## What this builds

Extend `~/wintermute/lattice-bridge` (the existing `lattice-bridge` binary):

- Find the OWL/XML deserializer in `lattice-bridge` (likely in `src/parser.rs` or
  equivalent). The parser currently rejects unknown tags inside `owl:Ontology` as
  errors. Change the behavior: tags that appear inside an `owl:Ontology` element but
  are not owl:imports, owl:versionIRI, or known annotation properties should be
  *skipped with a warn-level trace*, not rejected with an error. Specifically
  `owl:versionIRI` must be tolerated (and its IRI captured in the parsed `Ontology`
  struct so `lattice-registry` can store the version).
- Add a test fixture: a minimal OWL/XML file containing `<owl:versionIRI …/>` inside
  `<owl:Ontology>` — the parser must parse it without error.
- Regression test: load the actual cached BFO and IAO OWL files (if present at a
  well-known path under `$HOME/.cache/lattice/registry/owls/`) and assert no parse error.
  Skip if the files are not present (not a build-time download dependency).

No new binary; no new crate. This is a one-function fix in `lattice-bridge`'s OWL/XML
parser with two test cases.

MSRV: match the existing `lattice-bridge` workspace (1.85 unless already higher).

## Acceptance criteria

1. `cargo test --release` passes in `~/wintermute/lattice-bridge`.
2. A minimal OWL/XML fixture containing `<owl:versionIRI rdf:resource="…"/>` inside
   `<owl:Ontology>` parses without error.
3. `lattice-bridge align --a <bfo-cached.owl> --b <iao-cached.owl>` exits 0 (or
   exits non-zero only for a *semantic* reason — no mappings found — not a parse error).
   Run against the cached files at `~/.cache/lattice/registry/owls/{bfo,iao}.owl` if
   they exist; if absent, skip this AC and log `(cached files absent, install test skipped)`.
4. The parsed ontology struct retains the `versionIRI` value from the OWL/XML header
   (accessible via a `Ontology::version_iri() -> Option<IRI>` method or equivalent).
5. No previously-passing parser tests regress.
