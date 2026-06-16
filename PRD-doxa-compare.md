# PRD: doxa-compare — where the philosophies agree, and where they collide

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/doxa
Vision: visions/doxa.md
Depends-on: doxa-reason (per-framework verdicts)

## TL;DR

The point of holding many philosophies is to *compare* them. This PRD ships
`doxa compare <fw...> --scenario <abox>`, which runs N frameworks on one scenario and
emits an agreement/conflict matrix: where the traditions converge on a verdict (an
overlapping consensus — the most defensible moral ground) and where they diverge (the
genuine dilemma, where the choice of framework decides the outcome).

## Why this exists

doxa-reason answers one framework at a time. The seed — "extending ousia to all
philosophies" — is fundamentally about *plurality*, and plurality's value is in the
comparison: an action all three normative families call wrong is far more robustly wrong
than one only consequentialism flags. The trolley scenario shipped in doxa-reason
already proves the frameworks diverge; this PRD makes that divergence a first-class,
inspectable artifact rather than something the user reconstructs by running `doxa reason`
three times by hand.

## What this builds

Extend `~/wintermute/doxa`:

- **`doxa compare <fw1> <fw2> [...] --scenario <abox.ttl> [--format text|json]`**:
  - Run `doxa reason` (the in-process path from doxa-reason) for each named framework on
    the scenario; collect each verdict (`RightAction`/`WrongAction`/`PermissibleAction`/
    `undetermined`).
  - **Consensus**: if all decided frameworks agree, report it as the overlapping consensus.
  - **Conflict**: if decided frameworks disagree, report the split — which frameworks say
    what, with each one's one-line justification.
  - **Abstention**: frameworks returning `undetermined` are listed separately (they neither
    consent nor dissent).
- **`--scenario` may be omitted** for a *structural* comparison: with no scenario, compare
  the frameworks' `RightAction` *definitions* themselves (what feature each treats as
  morally decisive — consequence vs maxim vs character) — a "how do these philosophies
  differ in principle" view independent of any case.
- **Text output**: a matrix (frameworks × verdict) + a one-line summary
  (`consensus: WrongAction (3/3)` or `conflict: 2 wrong / 1 permissible`).
- **JSON output**: `{scenario, verdicts:{fw:verdict}, consensus:<verdict|null>,
  conflict:bool, abstentions:[...]}` for the guard PRD and MCP to consume.

Reuses doxa-reason; adds only the fan-out + matrix logic. MSRV 1.85.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `doxa compare consequentialism deontology virtue-ethics --scenario scenarios/trolley.ttl`
   runs and prints a matrix with one verdict per framework (skip live reasoning if
   `ousia-reason` absent; unit-test the matrix/consensus logic on recorded verdicts).
3. The trolley comparison reports a **conflict** (the frameworks do not all agree) — a
   test asserts `conflict: true`.
4. `doxa compare <a> <a> --scenario S` (same framework twice) reports consensus
   (degenerate agreement) — a sanity check.
5. A scenario crafted to make all three agree reports `consensus` with the shared verdict
   and `conflict: false`.
6. `doxa compare consequentialism deontology` with **no** `--scenario` prints the
   structural difference (the decisive feature each framework uses) and exits 0.
7. `--format json` emits the documented shape; a test round-trips it.
8. `undetermined` frameworks are reported as abstentions, not counted as agreement.
