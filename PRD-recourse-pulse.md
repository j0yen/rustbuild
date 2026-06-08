# PRD: recourse-pulse — is the shipped conscience behaving the way tribunal said?

Status: Draft v0.1
build_priority: high
build_target: rust-cli
build_into: /home/jsy/wintermute/recourse
Vision: visions/recourse.md

## TL;DR

`tribunal` proves the engine behaves correctly on a held-out corpus *before*
shipping. It cannot prove the engine keeps behaving that way *in the field*, where
the action distribution is whatever the world throws at it. This PRD builds the
**pulse**: an honest, aggregate-only, opt-in read of the receipt stream — verdict
distribution, contest rate, which axioms actually fire in the wild, and drift
across ontology versions. It answers one question with numbers instead of hope:
*is the shipped `/conscience` behaving the way `tribunal` predicted?*

## Why this exists

- The arc ships verdicts to machines this laptop never sees with no measurement
  back (vision recourse §"Why"). The pulse is the measurement — built strictly on
  `recourse-receipt`'s local sink, never on raw actions.
- `recall-outcome-feedback` (MEMORY.md) is the laptop's prior art for a
  "did-what-we-shipped-actually-behave?" loop; the pulse is that loop for verdicts.
- **No silent caps** is a fleet rule (PRD-tribunal-corpus: "surfaces under-covered
  tenets and any verdict class with zero cases"). The pulse inherits it: it must
  surface under-fired tenets and verdict-class skew, not hide them behind an
  average.

## What this builds

Adds the `pulse` subcommand to the `recourse` crate. Pure read over the local
receipt + contest sinks; emits **aggregates only**.

- **`recourse pulse [--since <dur>] [--format json|table]`** reports:
  - **Verdict distribution** — counts and % of `allow|flag|deny` over the window.
  - **Contest rate** — contests / receipts, and uphold rate of reviewed contests.
  - **Per-axiom fire counts** — how often each `fired_rule`/`tenet` actually fired
    in the field, sorted; **explicitly flags tenets with zero field fires** (the
    "we proved it but it never triggers in the wild" signal).
  - **Version drift** — verdict distribution sliced by `ontology_version`, so a
    distribution shift after a version bump is visible.
- **`recourse pulse --export <file>`** writes the aggregate report to a file the
  user can choose to share. **This is the only path by which any field data leaves
  the machine, and it carries aggregates only** — no `receipt_id`, no
  `action_digest`, no per-action rows. A test asserts the export contains none of
  those fields.
- Privacy invariant: `pulse` **never** reads the opt-in raw `actions/` store and
  never emits a per-receipt row in any mode.

**Deps:** shares the `recourse` lib; `serde_json`, `time`, `clap`. SIGPIPE reset.
rustc 1.85, no let-chains.

## Acceptance criteria

1. `recourse pulse --since 30d` over a fixture sink reports verdict counts and
   percentages that sum to 100% (± rounding) and match the fixture by hand-count.
2. Contest rate = contests/receipts over the window; uphold rate = upheld/(upheld+
   rejected) reviewed contests; both verified against a fixture.
3. Per-axiom fire counts are sorted desc and **every tenet that never fired in the
   window is listed with a zero/under-fired flag** — not omitted. A fixture with an
   unused tenet asserts it appears flagged.
4. Version drift: with receipts under two `ontology_version`s, the report shows a
   per-version verdict breakdown; a test with a deliberate post-bump shift asserts
   the shift is visible.
5. **Aggregate-only invariant (the point):** no `pulse` mode emits `receipt_id`,
   `action_digest`, or any per-action row. `--export` output contains none of these
   fields (asserted by a test that greps the export).
6. `pulse` never opens the raw `actions/` directory (asserted: run with a sentinel
   file in `actions/` and confirm it is never read — e.g. via a read-tracking temp
   dir or by asserting output independence).
7. SIGPIPE-safe; `cargo test` green; `pulse --help` documents `--since`, `--format`,
   `--export`.
