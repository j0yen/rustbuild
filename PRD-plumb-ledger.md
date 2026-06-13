# PRD: plumb-ledger — a calibration history so a probe that has lied stays distrusted

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/plumb
Vision: visions/plumb.md

## TL;DR

`plumb check` (plumb-core) answers "does this probe agree with ground
truth *right now*?" — a single snapshot. But a probe that read false on
2026-06-12 and happens to read true on 2026-06-13 should not be trusted
just because today it agreed. `plumb-ledger` adds an append-only
calibration history: every `plumb check` records its outcome, and
`plumb trust <probe-id>` reports the probe's agreement track record and
an `uncalibrated` verdict that persists until the probe re-proves itself.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The same probe oscillates across runs.** The ctrace-wiring probe
  read **false-absent** on 2026-06-12 (journal: *"Phase A probe was wrong
  — said wiring absent when it exists"*) and **true-absent** on
  2026-06-13 (journal: *"scribe backfill NOT wired"*). A single
  `plumb check` on 06-13 would say `agree` and hand the probe a clean
  bill — erasing the fact that it lied the day before. Trust must be
  *historical*, not instantaneous.
- **Self-review carries findings for many runs.** Journal/recall show
  `ctrace-sessionend-flake` runs_seen=9, `agentns-session-zeros`
  runs_seen=13+. A probe feeding a finding that recurs for 9–13 runs is
  exactly the place a one-bad-reading-in-the-history should stick.
- **Append-only ledgers are the laptop's established pattern.** The
  `answerable` PRD shipped an append-only autonomous-action ledger
  (O_APPEND, SIGPIPE-safe, library API) on 2026-06-12; this reuses that
  shape rather than inventing a new persistence model.

## What this builds

Extends the `~/wintermute/plumb/` crate (no new repo).

### Persistence

Append-only NDJSON at `~/.local/share/plumb/calibration.ndjson`, one
record per `plumb check` invocation:

```json
{"probe":"memlog-active","ts":"2026-06-13T05:48:00Z","verdict":"inactive","oracle":"active","result":"disagree"}
```

- Opened `O_APPEND`; concurrent self-review + manual runs never truncate.
- Timestamp is **injected** (a `--at <rfc3339>` flag, defaulting to the
  process's wall clock at the boundary) so the core comparison logic
  stays clock-free and unit-testable — matches the workflow/journaling
  convention of stamping at the edge.
- `plumb check` (from plumb-core) gains an implicit ledger append; a
  `--no-record` flag suppresses it for dry runs.

### Command surface (added)

```
plumb trust <probe-id>              # agreement rate + last-disagreement + verdict
plumb trust --all                   # every probe's trust line
plumb trust <probe-id> --format json
```

`plumb trust` output:

```json
{"probe":"memlog-active","checks":7,"agreements":3,"rate":0.43,
 "last_disagreement":"2026-06-13T05:48:00Z","verdict":"uncalibrated"}
```

### Trust verdict

- `uncalibrated` when the agreement rate over the window is below the
  threshold (default **strict 1.0** — any disagreement in the window
  marks the probe uncalibrated until it agrees again).
- `trusted` when every check in the window agreed.
- `unknown` when the probe has no ledger history yet.
- Window and threshold are configurable (`--window N`, `--threshold F`);
  strict-1.0, full-history is the default.

### Constraints

- SIGPIPE-safe (`sigpipe::reset()` already established in plumb-core).
- Ledger reads tolerate a partial final line (crash mid-append) without
  panicking — skip-and-warn, don't abort.
- `--format json` for `trust` is the contract `plumb-selfreview-bind`
  consumes to decide quarantine.

## Acceptance criteria

1. After running `plumb check memlog-active` once, the NDJSON ledger file
   exists and contains exactly one well-formed JSON record with `probe`,
   `ts`, `result` keys.
2. Two sequential `plumb check` calls append two records (file grows,
   prior record intact) — confirms O_APPEND, no truncation.
3. `plumb check --no-record` runs the comparison but appends nothing.
4. `plumb trust memlog-active --format json` reports `checks`,
   `agreements`, `rate`, `last_disagreement`, `verdict`.
5. A probe with one or more `disagree` records in the window reports
   `verdict: "uncalibrated"` under the default strict threshold; a probe
   whose every windowed check is `agree` reports `"trusted"`; a probe with
   no history reports `"unknown"`.
6. `--window N` limits the trust computation to the last N records;
   `--threshold F` changes the rate cutoff — both reflected in the verdict.
7. A ledger file whose last line is truncated mid-write is read without
   panic (the partial record is skipped, a warning is emitted to stderr).
8. Piping `plumb trust --all` into `head` does not panic.
