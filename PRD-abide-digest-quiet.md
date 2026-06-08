# PRD: abide-digest-quiet — acked findings go quiet, never invisible

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/docket
Vision: visions/abide.md

## TL;DR

`abide-ack-state` adds an acknowledged flag to docket findings, but on its own
an acked finding still appears in `docket digest`, `docket list`, and the
SessionStart banner exactly as before — the flag exists, but nothing acts on
it. This PRD makes the ack *do something*: the default digest/list views
exclude acked findings from the `open`/`escalated` counts and from
`HealthStatus`, while a new `acked` tally and an `--include-acked` flag keep
them one keystroke away. The result is the noise reduction the `abide` vision
exists for — the morning banner stops re-surfacing items the human triaged days
ago — without ever fully hiding a carried finding.

## Why this exists

Phase 1 evidence (2026-06-08 — see `visions/abide.md`):

- The whole point of `docket` is to stop self-review re-discovering the same
  finding (`visions/docket.md`). But `docket list --open` (live) still shows
  `memlog-activation` and `warden-enforcer-inert` as `[open] warn` despite both
  being "acked" in prose for ≥3 runs — so the banner re-surfaces them every
  morning, which is the exact noise docket was built to kill.
- The digest envelope (`src/digest.rs:64` `DigestEnvelope`,
  `DigestDetail` at `src/digest.rs:64`/`compute()` at `:106`) currently counts
  every open/escalated finding. Its `summary` reads `"N open, M escalated"` and
  `HealthStatus` (`src/digest.rs:40`) goes `Degraded` on any escalated warn —
  so an acked-but-carried warn drags the whole component to `Degraded`,
  defeating the carry decision.
- `abide-ack-state` (strict-chain dependency) ships the `is_acked()` predicate
  and the ack columns this PRD filters on; build this only after it lands.

## What this builds

`rust-extend` into `~/wintermute/docket` (`digest.rs`, `cli.rs`, `db.rs`
query/list path):

- **Default exclusion.** `docket digest` and `docket list` filter out findings
  where `is_acked()` before computing the open/escalated counts and the text
  listing. The DB `list` query gains an `include_acked: bool` parameter
  (default false).
- **`acked` count, never hidden.** `DigestDetail` (`digest.rs`) gains
  `pub acked: u64` — the number of currently-acked findings passing the
  severity filter. `compute()` takes the acked findings (or their count) so the
  tally is accurate. The `summary` string folds it in:
  `"3 open, 1 escalated, 2 acked"` (omit the `acked` clause when zero, to keep
  existing summaries byte-identical when nothing is acked).
- **`HealthStatus` ignores acked.** An acked finding does not contribute to
  `Degraded`/`Down` — only non-acked open/escalated findings do. (An acked
  finding whose fingerprint changed has already auto-resurfaced via
  `abide-ack-state`, so a genuinely-changed condition still counts.)
- **`--include-acked` flag** on both `docket digest` and `docket list` — shows
  acked findings in the listing/counts as before, for when the human wants the
  full picture. With the flag, acked findings render with an `(acked)` marker
  in text output.

No new crates. MSRV 1.85, no let-chains.

## Acceptance criteria

1. **Acked findings are excluded from default `docket list`.** Given an acked
   finding and an open one, `docket list --open` shows only the open one;
   `docket list --open --include-acked` shows both (the acked one marked
   `(acked)`).
2. **`docket digest` excludes acked from open/escalated counts.** With one open
   warn and one acked warn, `detail.open == 1` (not 2) and the acked one is not
   in `escalated_keys`.
3. **`DigestDetail` reports a correct `acked` count.** In the same scenario,
   `detail.acked == 1`. With zero acked findings, `acked == 0` and the
   `summary` string is byte-identical to the pre-PRD format (no trailing
   `", 0 acked"`).
4. **`summary` includes the acked clause when non-zero**, e.g.
   `"1 open, 0 escalated, 1 acked"`.
5. **`HealthStatus` ignores acked findings.** A store whose only escalated warn
   is acked computes `HealthStatus::Ok` (not `Degraded`); un-acking it (or a
   fingerprint-change resurface) returns it to `Degraded`.
6. **`--include-acked` restores prior behaviour** — counts and listing match
   what the pre-abide digest/list produced for the same findings.
7. **Existing `digest.rs` tests stay green**; the pre-existing `compute()`
   tests that pass only non-acked findings produce unchanged envelopes (proving
   backward compatibility for the all-open case). `cargo test` and
   `cargo build` clean in `~/wintermute/docket`.
