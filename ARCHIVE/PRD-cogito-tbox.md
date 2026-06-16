# PRD: cogito-tbox — the operational TBox of the box itself

Status: Draft v0.1
build_target: rust-cli
Vision: visions/cogito.md

## TL;DR

The box has an OWL 2 DL forge (`ousia-forge build`: declarative TOML →
RDF/XML) but the only spec it has ever forged is the *ethical* World
Ontology. There is no formal vocabulary for the box's **own operational
world** — no class for `Daemon`, `Socket`, `Bus`, `Tool`, `Repo`,
`Session`, `KernelPrimitive`, and no relations like `dependsOn`,
`registersOn`, `writesTo`, `backedBy`, `providesCapability`. This PRD
ships `cogito tbox build`, which forges `cogito.owl` — a small,
BFO-grounded, OWL 2 DL operational TBox — reproducibly from a
human-auditable TOML spec, including the disjointness/closure axioms that
turn operational category errors into reasoner-detectable inconsistencies.

## Why this exists

- Live inspection (2026-06-16) of `~/.local/bin/`: `ousia-forge --help`
  shows `build` (spec dir → "OWL 2 DL RDF/XML"), `ousia-reason --help`
  shows `check` ("Validate the OWL 2 DL profile … report consistency").
  The machinery to build and validate an operational TBox already exists;
  only the operational *spec* is missing.
- `atlas` proves the value of modelling the box as a graph but is a
  bespoke, non-OWL join over the static PRD corpus only (`atlas --help`:
  "Reads PRDs, visions, manifests, and REPOS.md") — no class hierarchy,
  no reasoner, no runtime daemons/sockets.
- The recalld healthcheck bug fixed this session (a bus healthcheck on a
  daemon that writes to a socket and never registers on the bus) is a
  *category error* — exactly the kind a TBox with a disjointness axiom
  catches. See vision §"Why now."

## What this builds

A new rust-cli repo `~/wintermute/cogito` (binary `cogito`, subcommand
`tbox`):

- **`cogito tbox build [--out cogito.owl]`** — forge the operational
  TBox to OWL 2 DL RDF/XML. Implementation reuses the ousia-forge
  approach: ship a declarative TOML spec under `spec/` in the repo and
  either (a) shell to `ousia-forge build spec/ --out <path>` if present
  on `$PATH`, or (b) emit equivalent RDF/XML directly via the `oxigraph`
  / `oxrdf` crates already used by ousia. Prefer (a); fall back to (b) so
  cogito does not hard-depend on ousia-forge being installed.
- **`cogito tbox check`** — validate the spec without emitting (mirrors
  `ousia-forge check`).
- **`cogito tbox stats`** — class/property/axiom counts.

The TBox (`spec/cogito.toml`), BFO-grounded:

- Classes: `Daemon ⊑ BFO:process`, `Unit`, `Socket ⊑ BFO:continuant`,
  `Bus`, `Tool`, `Repo`, `Session ⊑ BFO:process`, `KernelPrimitive`,
  `Healthcheck`, `BusHealthcheck ⊑ Healthcheck`.
- Object properties: `dependsOn` (transitive), `registersOn`,
  `writesTo`, `backedBy`, `providesCapability`, `healthcheckedBy`,
  `ownedByVision`.
- Signature axioms: `dependsOn` is `owl:TransitiveProperty`;
  `BusRegistrant ≡ Daemon ⊓ ∃registersOn.Bus`; the consistency hook —
  `BusHealthcheck ⊑ ∃healthcheckedBy⁻.BusRegistrant` (a daemon checked by
  a bus healthcheck must be a bus registrant), so asserting a bus
  healthcheck over a socket-only daemon yields an inconsistency.

IRI base `https://wintermute.local/cogito#`. Same `[[bin]]` + strict
clippy profile as the wm-* crates (deny unwrap/expect/panic, MSRV 1.85
unless oxigraph forces higher — match ousia-reason's MSRV).

## Acceptance criteria

1. `cargo test --release` passes in `~/wintermute/cogito`.
2. `cogito tbox build --out /tmp/cogito.owl` exits 0 and writes a
   non-empty RDF/XML file.
3. Re-running the build yields a **byte-identical** file (reproducible).
4. `cogito tbox stats /tmp/cogito.owl` reports ≥9 classes and ≥6 object
   properties.
5. The emitted ontology passes an OWL 2 DL profile check — verified by
   `ousia-reason check /tmp/cogito.owl` exiting 0 when `ousia-reason` is
   on `$PATH` (test is skipped with a logged note if it is not).
6. `dependsOn` is declared `owl:TransitiveProperty` and the
   `BusHealthcheck`/`BusRegistrant` axioms are present in the output
   (assert via a SPARQL ASK or a substring check on the RDF/XML).
7. `cogito tbox check` exits non-zero on a deliberately malformed spec
   fixture and zero on the shipped spec.
