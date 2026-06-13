# PRD: homeward-deliver-query — the owner's photo actually reaches the matcher

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

The vision's central promise is: *an owner submits one photo of their lost pet
and gets a ranked shortlist of visually-similar animals in nearby shelters.*
Today the owner can submit the photo — and it goes nowhere near the matcher. The
`homeward-reportd` match path is a hardcoded stub. This PRD replaces the stub
with the real flow: the stored lost-report photo is embedded via the sidecar
`/query`, the returned visual scores are fused with geo/date by `homeward-match`,
and a real ranked shortlist is returned. This removes the last stub on the
owner-facing critical path.

## Why this exists

Phase-1 live inspection (2026-06-13, `~/wintermute/homeward`):

- `homeward-reportd submit` reads the owner's `--photo` bytes
  (`reportd.rs:93-97,128-140`) and stores the report — the photo is captured.
- But `reportd`'s match demonstration builds **stubs**:
  `make_stub_report(&report_id)` and `make_stub_candidate(0.9)` with
  `photos: vec![]` (`reportd.rs:260-261,309-369`, doc-commented "stub for CLI
  pipeline demonstration"). The owner's real photo is never embedded or matched.
- `homeward-match` fuses a **caller-supplied** `visual_scores` map keyed by
  `canonical_id` (`report.rs:28`, `lib.rs:75`) — it does not embed anything
  itself. Nothing in the report path computes those scores, so even if wired,
  fusion would run geo+date only.
- `homeward-report/Cargo.toml` depends on neither `homeward-match` nor the embed
  client (confirmed: empty grep). The wire from owner photo → `/query` → visual
  scores → fusion → shortlist does not exist end-to-end.

This is the owner half of [[project_voice_input_null_detectors]] — a real input
captured at the front door, dropped before the engine. Honest matching here is
also bound by [[feedback_agent_written_fixtures_tautology]]: scores must come
from the real embedder over real photos, never fabricated.

Depends on **homeward-embed-client**; consumes the gallery built by
**homeward-deliver-enroll**. This is a `rust-extend` of `homeward-report`.

## What this builds

- **`homeward-report` gains real dependencies** on `homeward-embed-client` and
  `homeward-match` (added to its `Cargo.toml`).
- **A real `reportd match --report <id>` path**: load the stored `LostReport`,
  read its photo, call `/query` via `homeward-embed-client` to get visual
  similarity scores keyed by `canonical_id` (the gallery enrolled by
  `homeward-deliver-enroll`), build the `ReportFilter`/match input from the
  report's species + coarse geo + intake-date window, run `homeward-match`'s
  fusion, and print the **real** ranked shortlist with per-candidate signals
  (the "visual similarity X%" line `homeward-match` already renders,
  `fusion.rs:157`). `make_stub_report` / `make_stub_candidate` are deleted.
- **Candidate-not-confirmation framing preserved.** Output stays "possible
  matches for human review," never "we found your pet" — the vision's UX
  guardrail and the existing report-crate contract.
- **Honest degradation.** If the sidecar is unreachable, the command returns a
  clear "visual matching unavailable — falling back to geo+date only" notice and
  still produces the structured shortlist, rather than a stubbed 0.9 or a crash.
  If the gallery is empty (nothing enrolled yet), it says so explicitly.
- **No PII leak.** The shortlist carries no owner contact and no raw coordinates
  (existing `api.rs:56-64` sanitisation posture); the brokered token path is
  untouched.

## Acceptance criteria

1. `cargo build` and `cargo test` pass for the workspace with `homeward-report`
   depending on `homeward-embed-client` and `homeward-match`.
2. `make_stub_report` and `make_stub_candidate` are removed from
   `reportd.rs`; no remaining `reportd` code path emits a fabricated candidate
   score (grep-asserted in review).
3. Given a stored report with a photo and a stub/mock `/query` endpoint returning
   known `canonical_id → score` pairs, `reportd match --report <id>` produces a
   shortlist whose ordering reflects the fused visual+geo+date scores (asserted
   by a test against the mock).
4. With the sidecar unreachable, the command still returns a geo+date shortlist
   and prints an explicit "visual matching unavailable" notice — no panic, no
   fabricated visual score (asserted by a test).
5. The shortlist output contains no owner contact string and no precise
   coordinates; framing is candidate-not-confirmation (asserted by a test on the
   rendered output).
6. `cargo clippy` clean at the workspace lint level; no new `-D` regressions.
