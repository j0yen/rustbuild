# PRD: tribunal-bench — run the conscience against the held-out corpus and score it

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/tribunal
Vision: visions/tribunal.md

## TL;DR

With a structural conformance suite (`tribunal-conformance`) and an independent
corpus (`tribunal-corpus`) in hand, this PRD builds the harness that actually
measures the reasoner: `tribunal bench` runs `ousia-guard` over every corpus
case, compares the *actual* verdict and fired rule against the *expected* ones,
and emits an accuracy report with a confusion table. It singles out the one
failure an ethics engine must never make — a **false-allow** of a deny-class
case (the conscience waving through something it should have denied) — and fails
hard on any.

## Why this exists

- The corpus and conformance suite are inert without something that exercises
  `ousia-guard` against them. `PRD-ousia-guard.md` defines the contract this
  harness drives: `ousia-guard check --owl world-ontology.owl --action
  action.json --format json --explain` → `allow|flag|deny` + the axiom chain,
  with named rules `dignity-floor`, `rights-violation`,
  `authority-without-accountability`, `flourishing`.
- A scalar "accuracy" hides the asymmetry that matters: for a conscience,
  *false-allow* (deny → allow) is categorically worse than *false-deny* (allow →
  deny). The bench must report the confusion matrix, not a single number — the
  same lesson as `feedback_agent_written_fixtures_tautology` (a headline number
  concealed a real-world drop). The gate (next PRD) keys off false-allow == 0.
- `ousia` is freshly drafted and **unbuilt**; per the vision's cross-vision note,
  bench must build and self-test against a recorded-fixture stub of guard now,
  with the wire contract asserted so it transfers cleanly to the real binary —
  the same stub-with-asserted-contract discipline gossip noted for
  `herald-conscience`.

## What this builds

Extends `~/wintermute/tribunal` with a `bench` subcommand.

- **UX:**
  ```
  tribunal bench --guard ousia-guard --owl world.owl --corpus corpus/
  tribunal bench --guard ./fixtures/guard-stub --corpus corpus/ --format json
  ```
  `--guard` is the path to the `ousia-guard`-contract binary (real or stub).
- **Behaviour:**
  - For each case: invoke `<guard> check --owl <owl> --action <case>/action.json
    --format json --explain`, parse the verdict + fired rule + axiom chain.
  - Compare to `expected.toml`: verdict match, and rule match (the engine got the
    right answer *for the right reason* — a right verdict via the wrong rule is
    reported as a distinct "right-verdict-wrong-rule" bucket, not a clean pass).
  - **Report:** overall accuracy; per-tenet accuracy with N (no silent caps —
    under-covered tenets show their small N); a 3×3 verdict confusion table; a
    distinguished **false-allow count** (expected `deny`, actual `allow`) listing
    each offending case id; and a "wrong-rule" count.
  - Exit code: 0 only if every case ran (no guard crashes / unparseable output);
    the pass/fail *threshold* decision belongs to `tribunal gate`, but bench
    exits non-zero if any guard invocation errors or any output fails the wire
    contract.
- **Wire-contract assertion:** a test parses the stub's JSON against the same
  serde structs used for the real binary; a contract test documents the exact
  expected `--format json` shape so a drift in `ousia-guard` is caught, not
  silently mis-scored.
- **Fixture stub:** `fixtures/guard-stub` — a tiny script/binary that emits
  contract-shaped verdicts for the corpus (deterministic, lets the full bench +
  scoring be tested with no ousia build). The stub is for CI only; real runs pass
  the real `ousia-guard`.
- **Deps:** `serde`, `serde_json`, `clap`, `std::process`. SIGPIPE reset in
  `main()`.

## Acceptance criteria

1. `tribunal bench --guard fixtures/guard-stub --corpus corpus/` runs every case
   and prints overall + per-tenet accuracy and a 3×3 confusion table.
2. With a stub crafted to mis-handle a known deny-class case as `allow`, bench's
   report shows `false_allow >= 1` and lists that case id.
3. `--format json` emits a stable schema `{ overall, per_tenet: [...],
   confusion: {...}, false_allow: [case_ids], wrong_rule: n, ran: n, errored:
   n }`, serde round-trip tested.
4. A "right verdict, wrong rule" case (verdict matches, fired rule differs from
   `expected.rule`) is counted in `wrong_rule`, not as a clean pass.
5. A guard invocation that emits malformed/contract-violating JSON makes bench
   exit non-zero and name the offending case (contract assertion bites).
6. `cargo test` green on rustc 1.85 with the stub; no panic when piped to `head`.
   A contract test pins the expected `ousia-guard --format json` shape.
