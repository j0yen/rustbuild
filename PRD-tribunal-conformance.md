# PRD: tribunal-conformance — prove the ontology is what the paper claims

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/tribunal
Vision: visions/tribunal.md

## TL;DR

`ousia` ships an ontology whose value rests on three *structural* claims the
paper makes about it — OWL 2 DL profile compliance (§8.1), single inheritance
across every class, and TBox-only content (§8.4) — plus a §8.3 table mapping
ten foundational axioms onto OWL encodings. Today those claims live in prose and
in the axiom-author's own test suite. This PRD builds `tribunal conformance`: a
standalone, offline Rust CLI that parses the emitted `.owl` and turns each
structural claim into a deterministic pass/fail with a machine-readable report.
It is the `ousia-conformance` suite the ousia vision explicitly deferred, rehomed
into `tribunal` so the judge lives outside the thing it judges.

## Why this exists

- `visions/ousia.md` open questions: *"A dedicated `ousia-conformance` PRD may be
  worth splitting out of reason's ACs once reason exists. Deferred to a later
  /dream pass."* This is that pass.
- `PRD-ousia-reason.md` records the testable claims but folds conformance into a
  reasoner's own ACs: line 14 ("a consistency/conformance verdict"), line 23
  ("forge produces only the TBox … §8.4"), line 27 ("§8.1 claims OWL 2 DL
  compliance"), line 43 ("The ten axioms (§8.3 table)"). A reasoner grading its
  own output's conformance is the tautology this vision exists to break
  (`feedback_agent_written_fixtures_tautology`: agent-written rules + fixtures
  prove nothing — `wm-router` 100% → 73.5% on a held-out check).
- `PRD-ousia-forge.md` emits the TBox `.owl` via `horned-owl`; `tribunal
  conformance` parses that same artifact independently.

## What this builds

A new Cargo workspace at `~/wintermute/tribunal` (the home for all four tribunal
PRDs) with a binary `tribunal` and a first subcommand `conformance`.

- **Deps:** `horned-owl` (parse OWL, walk axioms — same parser ousia uses, so the
  read is faithful), `clap`, `serde`, `serde_json`. No network, no JVM.
- **UX:**
  ```
  tribunal conformance --owl world-ontology.owl              # human table, exit 0/1
  tribunal conformance --owl world-ontology.owl --format json # structured report
  ```
- **Checks (each → a named pass/fail line in the report):**
  1. **OWL 2 DL profile (§8.1):** no use of constructs outside OWL 2 DL (no
     punning violations, no illegal cardinality on transitive/complex roles,
     declared entities). Report names the first offending axiom on failure.
  2. **Single inheritance:** every named class has ≤ 1 named asserted
     superclass (the paper's BFO-style single-inheritance claim). Report counts
     classes checked and lists any with multiple named parents.
  3. **TBox-only (§8.4):** zero ABox individuals / assertional axioms in the
     ontology. Report names any individual found.
  4. **Ten-axiom encoding table (§8.3):** each of the ten foundational axioms is
     present and is a well-formed all-some `SubClassOf`/equivalence restriction
     of the expected pattern. The expected ten are declared in a checked-in
     `axioms.toml` manifest (axiom id → expected shape); report shows
     present/absent/malformed per axiom.
- **Fixture for build-time test:** a vendored minimal `fixtures/world-min.owl`
  (a few classes exercising each check, plus a deliberately-broken
  `fixtures/world-bad.owl`) so the suite is fully testable before `ousia-forge`
  exists. When forge ships, the same binary runs against the real `.owl` with no
  code change.
- **Provenance:** SIGPIPE reset on first line of `main()` per
  `self_sigpipe_panic_toolkit` (this is a `~/.local/bin`-class CLI that will be
  piped into `head`).

## Acceptance criteria

1. `tribunal conformance --owl fixtures/world-min.owl` exits 0 and the report
   marks all four checks `pass`.
2. `tribunal conformance --owl fixtures/world-bad.owl` exits non-zero and the
   report names the specific failing check(s) and the offending axiom/class/
   individual.
3. `--format json` emits a stable schema: `{ checks: [{ name, status, detail }],
   ok: bool }`, parseable by `serde_json` round-trip in a test.
4. The single-inheritance check correctly flags a class with two named asserted
   superclasses in `world-bad.owl` and passes a class with one in
   `world-min.owl`.
5. The §8.3 check reads `axioms.toml` (ten entries), reports present/absent/
   malformed per axiom, and fails if any of the ten is absent or malformed.
6. `cargo test` is green; the binary builds clean on rustc 1.85 (no let-chains,
   MSRV-respecting); SIGPIPE does not panic when output is piped to `head`.
