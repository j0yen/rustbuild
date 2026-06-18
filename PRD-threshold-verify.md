# PRD: threshold-verify — trust the predecessor's letter only where it's true

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/threshold
Vision: visions/threshold.md

## TL;DR

The predecessor session leaves a hand-written "letter to next-me" (a
`reflective/self` recall note). These letters are real, heavily surfaced, and
acted upon — but they can claim things that never happened. This PRD adds
`threshold verify`: it parses the latest letter into discrete claims and
cross-checks each against live ground truth, tagging every claim `confirmed |
stale | contradicted | unverifiable` so the arriving session trusts only the
true parts.

## Why this exists

Phase-1 inspection, 2026-06-18:

- The letter genre is real and load-bearing. The canonical "letter to next-me"
  recall note (`01KSS3VF07GFM3CFWXWPZP4MG0`, `reflective/self`) reads literally
  *"What next-me should expect… Don't redo any of this from scratch"* and shows
  `surfaced_count=234`, `used_count=65` — successors read and act on these.
- But letters lie by omission and drift. The standing
  `feedback_letter_vs_stack` memory: *"past-Claude's letter can describe
  'pending in stack' items that were never actually `stack push`-ed; verify with
  live state before acting."* Two whole visions — `warrant` and `assay` — exist
  because a **closed PRD's closing claim was mechanically false**, caught only by
  exercising the live pipeline instead of trusting the note.
- The ground-truth cross-check primitive already exists and is proven:
  `answerable reconcile` *"cross-check[s] the ledger against ground-truth sources
  (git pushes, gh repo-creates, dotfiles)"* (`answerable --help`, this pass).
  `threshold verify` should reuse that precedent rather than reinvent it.
- `threshold-brief` (this fleet's foundation) already surfaces letter content in
  its briefing — but today it would surface it *unverified*. This PRD is what
  makes the briefing honest.

## What this builds

Extends `~/wintermute/threshold` (the crate from PRD-threshold-brief).

- **Claim extraction:** parse the latest `reflective/self` recall note into a
  `Vec<Claim>`. Heuristic v1 — split on bullets/lines, classify each line's
  *kind* (e.g. `pushed-repo`, `shipped-prd`, `daemon-up`, `in-flight-agent`,
  `pending-todo`, `narrative`) by keyword. `narrative` claims are recorded as
  `unverifiable` by design. An `--llm` flag (local qwen via the brain ladder) is
  explicitly deferred to a later pass.
- **Verifiers**, one per checkable claim kind, each returning a `Verdict`:
  - `pushed-repo` / `shipped-prd` → git porcelain + `git log` + (optionally)
    `answerable reconcile` output for that repo.
  - `daemon-up` / `peer-present` → agorabus peer list.
  - `in-flight-agent` / `pending-todo` → build manifest state.
- **`ClaimVerdict { claim, status, evidence }`** where `status ∈ {confirmed,
  stale, contradicted, unverifiable}` and `evidence` cites the source checked.
- **`threshold verify`** subcommand: `--format text|json`, `--note <id>` to
  target a specific note (default: latest), `--source-root` testing seam.
- **Brief integration:** when a verdict set is available, `threshold brief`
  renders letter-derived signals with their trust badge (a `confirmed` claim
  reads differently from a `contradicted` one). Wire through the existing
  `Briefing` model; do not break the brief's standalone (no-verify) path.

## Acceptance criteria

1. `cargo build` / `cargo test` green in the extended crate; clippy adds no new
   warnings over baseline; `threshold verify --help` lists the documented flags.
2. Claim extraction over a fixture letter yields the documented `Claim` kinds,
   and any line classified `narrative` is reported `unverifiable` (test-proven).
3. Each verifier is unit-tested against fixtures for all four verdict values:
   a claim true in ground truth → `confirmed`; a claim about a repo with no such
   push → `contradicted`; a claim now superseded → `stale`; an unparseable/
   narrative claim → `unverifiable`.
4. `threshold verify --format json` emits a stable, documented schema:
   `[{claim, kind, status, evidence}]`, validated by a test.
5. A `contradicted` verdict is *never* silently dropped — it appears in both text
   and JSON output (regression guard against the `warrant`/`assay` false-close
   class).
6. `threshold verify` run live on this box against the latest reflective note
   completes without panic, exits 0, and produces at least one non-`unverifiable`
   verdict (proves the verifiers reach real ground truth).
7. `threshold brief` still works with verification absent (graceful: letter
   signals show without a badge), proven by an existing-path test.
