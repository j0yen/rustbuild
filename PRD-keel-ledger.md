# PRD: keel-ledger — account for the cloud before the wall

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/keel`
**Vision:** visions/keel.md

## TL;DR

"Anthropic credit exhausted" keeps being discovered the expensive way — by a
402 mid-turn, or by a human noticing at self-review. There is no local record of
what the brain has spent. **keel-ledger** adds an append-only spend ledger to
the `keel` workspace: every cloud call records its tokens and estimated cost, a
402/401 stamps the tier `Exhausted`/`Keyless` with a timestamp, and `keel spend
--since <dur>` shows the burn with a threshold warning *before* the wall.

## Why this exists

- docket `wm-anthropic-key-empty` (open since 2026-05-30, 6 runs, 8 reports) and
  every journal since note "credit also previously exhausted" /
  "`WM_ANTHROPIC_KEY` empty" (`project_brain_local_first_ladder`: "Cloud key
  WM_ANTHROPIC_KEY live" was the *goal*; reality has been empty/exhausted for
  weeks). Exhaustion is treated as a binary surprise, never a trend.
- `thrift`'s whole premise is that "`wintermute-brain` (wmd) is the fleet's only
  Anthropic API consumer." That means **one** place produces all cloud spend —
  so one local ledger can capture all of it. keel-ledger is the accounting half
  that thrift's cost-routing half can later read (vision Open question:
  `keel spend → thrift`).
- Builds on keel-pulse's `LedgerEntry` type and `TierStatus::{Exhausted,
  Keyless}` (already declared there) — this PRD makes them live.

## What this builds

rust-extend of `~/wintermute/keel` (the repo keel-pulse created; do not start
until keel-pulse has SHIPPED — extend-validate will fail otherwise).

- **Append-only ledger store** at a config path (default
  `~/.local/state/keel/spend.ndjson`), one `LedgerEntry` per line. Append-only,
  never rewritten (mirrors the gossip/recall discipline). A `LedgerStore` trait
  abstracts the path so tests write to a fixture dir.
- **`keel record`** (and a library `record()` the brain can call) — appends an
  entry `{tier, tokens_in, tokens_out, est_cost_usd, ts}`. Cost is computed from
  a small static per-tier price table (haiku/sonnet/opus input+output $/Mtok),
  documented and dated in the source; unknown tier → cost `null`, never a panic.
- **Status stamping** — `keel mark <tier> exhausted|keyless` (and a library
  call) writes a `TierHealth` status override with a timestamp into a small
  `status.json`, so a 402 the brain saw becomes durable state cordon/beacon can
  read. (The brain calls this on a 402/401; the wiring PRD is separate.)
- **`keel spend --since <dur>`** — aggregates the ledger: total est cost, per-
  tier breakdown, call count, token totals. `--format json` too. A
  `--warn-at <usd>` threshold (default from config) makes the command exit
  non-zero and print a warning when the windowed spend crosses it.
- **No clock calls in code** — `ts` and "now" are injected (`now: i64` param /
  a `Clock` trait), per the no-`Date::now` discipline that keel-pulse and the
  quicken PRDs follow, so the build stays deterministic and cloud-safe.

## Acceptance criteria

1. `cargo build` + `cargo test` succeed offline; the ledger store path is
   injected in tests (fixture dir), no writes outside it (assert).
2. `record()` appends exactly one NDJSON line per call; the file is never
   rewritten or truncated (assert byte-prefix stability across two records).
3. Cost is computed from the documented price table for a known tier; an unknown
   tier yields `est_cost_usd: null` and does not panic.
4. `keel spend --since 7d --format json` aggregates only entries within the
   window (clock injected); per-tier and total sums are correct over a fixture
   ledger.
5. `keel mark sonnet exhausted` writes a durable `TierStatus::Exhausted` with the
   injected timestamp; a subsequent `keel pulse`/status read reflects the
   override (cross-checks the keel-pulse type surface).
6. `keel spend --warn-at 5.00` over a fixture ledger summing to >$5 exits
   non-zero and prints the breach; under threshold exits zero.
7. `keel spend | head` does not panic (SIGPIPE reset).
8. Integration test entry file (e.g. `tests/ledger.rs`) appears in `cargo test`
   output — guards against the orphaned-mock-subdir false-green
   (`self_orphaned_mock_tests`).
