# PRD: rosetta-attest — prove the ethical-decision chain composes into linked data

Status: Draft v0.1
build_target: rust-cli
Vision: visions/rosetta.md

## TL;DR

`rosetta-prov`, `rosetta-shacl`, `rosetta-credential`, and `rosetta-serve` each
translate one facet of an ethical decision into the semantic web. But a fleet of
translators proves nothing until something drives a **real action through the
whole chain** and asserts it composes and round-trips. `rosetta-attest` is the
capstone: take an action, ground it, guard it, emit PROV-O, validate with SHACL,
sign as a Verifiable Credential, load into the SPARQL endpoint, query it back,
and assert every link is non-empty and consistent — emitting a committed
receipt. The `continuity-attest` pattern applied to the rosetta chain.

## Why this exists

Phase-1 live inspection (2026-06-15):

- `continuity-attest`'s README is the exact precedent: "*every component to make
  [the vision] true is now built… what's missing is the proof that they
  compose*." It drives a real wrapped session through four kernel signals and
  asserts a non-empty, same-id join, with a committed receipt. The rosetta fleet
  has the identical risk — five tools that *should* compose but are never
  exercised end-to-end.
- Without this, "ethical decisions are interoperable linked data" is *believed*,
  not *attested*. A round-trip test (emit → serve → SPARQL-back → bytes match) is
  the only honest proof that the translation is lossless and the surfaces agree.
- `lattice-ground`'s README documents the natural-language entry point ("a
  feverish patient" → `wo:SentientBeing` → `wo:Dignity`) — the front of the
  chain rosetta-attest exercises.

## What this builds

New repo `~/wintermute/rosetta-attest/` (rust-cli, edition 2021, MSRV 1.85).

**Deps:** `serde`/`serde_json`, `oxrdf`/`oxrdfio`, `clap` v4; invokes the other
four rosetta binaries (and `ousia-guard` / `lattice-ground`) as subprocesses —
this is an integration harness, not a re-implementation.

**The attested chain (one `run`):**

1. **Ground** — feed a natural-language action mention to `lattice-ground`,
   resolve to ontology classes (e.g. → `wo:SentientBeing`/`wo:Dignity`).
2. **Guard** — run `ousia-guard` over the grounded action → `Evaluation`.
3. **PROV-O** — `rosetta-prov emit` the verdict → Turtle graph G1.
4. **SHACL** — `rosetta-shacl validate` the action-graph → `sh:ValidationReport`;
   assert the SHACL verdict and the guard verdict **agree** on allow/deny.
5. **Credential** — `rosetta-credential issue` over the verdict + G1 → signed VC;
   `verify` it → must pass.
6. **Serve** — `rosetta-serve` load G1, SPARQL the verdict back → graph G2.
7. **Round-trip** — assert G1 and G2 are isomorphic (same triples), the VC
   verifies, and the SHACL/guard verdicts match.

Emit a **receipt** (`receipt.json` + human summary) recording each step's
status, the verdict, the triple counts, the VC key id, and a pass/fail per link;
commit it (the continuity-attest convention).

**CLI:**

```
rosetta-attest run   --mention "a doctor denies a sentient patient care"
                     [--receipt-dir receipts/]    # drive + assert the full chain
rosetta-attest run   --action action.json         # skip grounding, start from a built action
rosetta-attest doctor                              # check all rosetta binaries are on PATH + runnable
```

`run` exits 0 only if **every** link passes and the round-trip is isomorphic;
any broken link exits non-zero and the receipt records which.

## Acceptance criteria

1. `rosetta-attest doctor` reports presence + `--version` of rosetta-prov,
   rosetta-shacl, rosetta-credential, rosetta-serve, ousia-guard, and
   lattice-ground; exits non-zero (with the missing list) if any is absent.
2. `rosetta-attest run --mention <fixture>` executes all seven steps and writes
   a `receipt.json` with a per-step status and an overall verdict.
3. The receipt records that the SHACL `sh:conforms`/severity verdict and the
   `ousia-guard` `allow|flag|deny` verdict **agree**; disagreement fails the run
   (exit non-zero) and is recorded as the failing link.
4. The issued Verifiable Credential `verify`s within the run; a verification
   failure fails the run and is recorded.
5. The PROV-O graph emitted in step 3 and the graph queried back from
   `rosetta-serve` in step 6 are **isomorphic** (equal triple sets); a mismatch
   fails the run with the diff recorded.
6. On a fully successful run, exit code is 0 and the receipt's overall verdict is
   `attested`; the receipt is written under `--receipt-dir`.
7. On any broken link (kill one dependency / corrupt one graph), exit is
   non-zero and the receipt names exactly which link failed — no false `attested`.
8. The receipt is committed with the Joe Yen identity (the continuity-attest
   convention), and the README documents the seven-step chain it attests.
