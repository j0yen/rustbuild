# PRD: vest-verify — turn "84/84 stale" into actionable failure buckets reported to the docket

Status: Shipped (adopt v0.5.0, 2026-06-13)
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/vest.md

## TL;DR

Self-review reports `adopt-scan-stale-binaries: 84/84 tracked artifacts
not-current` run after run, and re-parks it with no insight into *why*
each one is stale. `adopt`'s install path emits exactly one failure
string — `"install exited 0 but --version / --help failed"` — which
flattens every distinct failure mode (installed to the wrong prefix,
not on PATH, build failed, smoke failed, source is simply newer than the
installed binary) into one opaque verdict. `vest-verify` adds a real
failure taxonomy: each not-current artifact is classified into a named
bucket, and the counts are reported to the docket so the self-review
acts on a *category* instead of re-typing "84/84" every run.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The backlog never moves.** self-review reflective memories
  `01KTZYJZQY…` (2026-06-13) and `01KTZS50DF…` (2026-06-12) both carry
  the identical line `84/84 tracked artifacts not-current (29 never
  installed, 55 installed-stale)`. The count is stable because nothing
  diagnoses the *cause* per artifact.
- **The only signal is binary.** `adopt/src/apply.rs:248-265`: on a
  `cargo install` that exits 0, adopt calls `is_invokable(&artifact.bin)`
  and, on miss, records the single string `"{bin} install exited 0 but
  --version / --help failed"`. A wrong-prefix install (PRD-vest-root-guard
  case) and a genuinely broken `--version` produce the *same* verdict.
- **The docket producer already exists.** `adopt report` (shipped,
  `PRD-adopt-docket-report.md`) wires scan findings to the docket
  ledger — but reports the flat "not-current" state, not a cause. The
  docket (`visions/docket.md`) is designed to escalate a finding when it
  recurs across 3+ runs; an undifferentiated "84/84" can never be acted
  on, only re-counted.

## What this builds

Extend `adopt` (`~/wintermute/adopt`). No new crate.

### Failure taxonomy

A `enum StaleReason` classifying each not-current artifact:

- `NeverInstalled` — no binary in `~/.local/bin`, `~/.cargo/bin`, or PATH.
- `WrongPrefix` — a binary exists under a bogus prefix (the
  `/home/jsy/~/…` tree from vest-root-guard); reuses that detection.
- `OffPath` — installed in a convention dir but not resolvable via the
  current `PATH` (distinguishes the vest-path env gap).
- `SourceNewer` — installed and invokable, but the repo HEAD / source
  mtime is newer than the installed binary (a real "needs reinstall").
- `BuildFail` — `cargo install` exited non-zero on the last attempt.
- `SmokeFail` — installed, on PATH, but `<bin> --version`/`--help`
  exited non-zero (a genuinely broken binary).

### `adopt verify` subcommand

`adopt verify [--format json|table]`:

- Runs the scan, classifies every not-current artifact into a
  `StaleReason`, and prints either a human table (`BIN | REASON |
  DETAIL`) or JSON (`[{bin, reason, detail}]`).
- Emits a summary line: `verify: N total · {NeverInstalled: a,
  WrongPrefix: b, OffPath: c, SourceNewer: d, BuildFail: e, SmokeFail:
  f}`.
- Exit 0 when all current; exit 1 when any not-current (so cron /
  self-review can branch on it).

### Docket reporting

Extend the existing `adopt report` path so each reported finding carries
its `StaleReason` as a stable per-reason docket slug
(`adopt-stale-<reason>`), not one blanket `adopt-scan-stale-binaries`.
This lets the docket escalate, say, `adopt-stale-buildfail` separately
from `adopt-stale-sourcenewer`.

## Acceptance criteria

1. `adopt verify --format json` emits one object per not-current
   artifact with `bin`, `reason` (one of the six `StaleReason`
   variants), and a `detail` string.
2. A fixture with a binary under a bogus `~/…` prefix classifies as
   `WrongPrefix` (not `NeverInstalled` and not `SmokeFail`).
3. A fixture binary present in `~/.local/bin` but absent from a
   restricted `PATH` classifies as `OffPath`, not `NeverInstalled`.
4. A fixture where the source file mtime is newer than the installed
   binary classifies as `SourceNewer`.
5. `adopt verify` prints a summary count line and exits 1 when any
   artifact is not-current, 0 when all are current.
6. `adopt report` writes per-reason docket slugs
   (`adopt-stale-<reason>`); a test asserts at least two distinct slugs
   are produced from a mixed fixture.
7. `cargo test` green; `cargo build --release` clean; `adopt verify
   --help` listed in `adopt --help`.

## Out of scope

- Acting on the buckets (reinstalling, cleaning) — verify only reports.
  `WrongPrefix` cleanup is PRD-vest-root-guard; `SourceNewer` reinstall
  is the normal `adopt apply` path made idempotent in
  PRD-vest-incremental.
- Changing the env so `OffPath` disappears (PRD-vest-path).
