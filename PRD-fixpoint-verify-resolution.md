# PRD: fixpoint-verify-resolution — give the stale count a true denominator

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/fixpoint.md

## TL;DR

`adopt verify` classifies every not-current artifact under a single reason
`SourceNewer`, lumping twelve same-day clock-noise installs together with
four genuinely-behind daemons (`wm-audio` 8d, `wm-dialog` 7d, `wm-tts` 6d,
`wm-reach` 2d). The real, actionable count is buried. This PRD splits
`SourceNewer` into `SourceNewer-sameday` (≤1d) and `SourceNewer-behind`
(≥2d, threshold configurable) so self-review and docket see the true number.

## Why this exists

Phase-1 live inspection, 2026-06-13:

```
$ adopt verify
BIN                       REASON             DETAIL
bon-mot-anagram           SourceNewer        source is 0d newer than installed binary
...
wm-audio                  SourceNewer        source is 8d newer than installed binary
wm-dialog                 SourceNewer        source is 7d newer than installed binary
wm-tts                    SourceNewer        source is 6d newer than installed binary
wm-reach                  SourceNewer        source is 2d newer than installed binary
verify: 16 total · {SourceNewer: 16}
```

The `DETAIL` column already computes the day-delta — it knows
`wm-audio` is 8d behind and `bon-mot-anagram` is 0d "behind" (committed
seconds after install; pure build-order/clock noise). But the *reason
bucket* throws that signal away: `{SourceNewer: 16}`. Self-review reads
"16/16 not-current" and treats it as undifferentiated backlog, when the
true actionable set is the four multi-day-behind voice daemons. The data
to split exists; only the classification is too coarse.

## What this builds

In `~/wintermute/adopt` (`src/verify.rs` and the reason enum in
`src/types.rs`):

- Split the `SourceNewer` reason into two variants based on the already-
  computed day delta against a threshold (default 2 days, overridable via
  `--behind-days <N>`):
  - `SourceNewer-sameday` — delta `< behind_days` (clock/build-order noise)
  - `SourceNewer-behind` — delta `>= behind_days` (genuine drift)
- Surface both in the table `REASON` column and in `--format json`
  (the per-artifact object gains the resolved reason string; the trailing
  summary line reports each bucket count, e.g.
  `verify: 16 total · {SourceNewer-behind: 4, SourceNewer-sameday: 12}`).
- Keep the existing exit-code contract (non-zero when any artifact is not
  current) unchanged — resolution is about *labeling*, not gating.

Scope guard: do not change `scan`, `apply`, `reconcile`, or marker logic.
This PRD only re-labels `verify`'s output. MSRV 1.85, no let-chains,
`sigpipe::reset()` already in `main`.

## Acceptance criteria

1. `adopt verify --format json` emits, for each not-current artifact, a
   reason of either `SourceNewer-sameday` or `SourceNewer-behind` (no bare
   `SourceNewer` remains for source-newer artifacts).
2. With the default threshold, `wm-audio` (8d), `wm-dialog` (7d),
   `wm-tts` (6d), `wm-reach` (2d) classify as `SourceNewer-behind`; the
   `0d`-newer bon-mot-* artifacts classify as `SourceNewer-sameday`.
   (Validate on a held-out fixture, not only live state, so the test is
   not tautological — construct fixtures with explicit 0d / 8d deltas.)
3. `--behind-days <N>` overrides the threshold; `--behind-days 0` puts all
   source-newer artifacts in `SourceNewer-behind`.
4. The trailing summary line reports per-bucket counts.
5. `cargo test` is green and `cargo build --release` succeeds; the
   existing non-zero exit-on-not-current behavior is preserved.
