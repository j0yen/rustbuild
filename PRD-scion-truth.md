# PRD: scion-truth — report a lineage-proven "behind" count to the docket

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/scion.md
Depends: PRD-scion-verdict.md, PRD-scion-reconcile.md

## TL;DR

Once scion-verdict answers freshness by lineage and scion-reconcile
gives every install a marker, the count `adopt` reports to the docket
should reflect *real* drift: binaries whose marker fingerprint differs
from current committed HEAD. Today `adopt report` (shipped via
`adopt-docket-report`) keys the docket finding
`adopt-scan-stale-binaries` on the timestamp verdict, so self-review has
re-parked a fabricated count (84→16) run after run. This PRD reframes the
reported finding on the lineage verdict and splits "genuinely behind"
from "unknown / no-marker", so the finding auto-resolves at zero when the
laptop is current.

## Why this exists (Phase 1 evidence, 2026-06-13)

Measured live this session:

- Self-review reflective memories `01KTZYJZQY…` (2026-06-13) and
  `01KTZS50DF…` (2026-06-12) both carry
  `adopt-scan-stale-binaries: …not-current` as a pending item the review
  cannot act on — the count is dominated by build-then-commit false
  positives (see PRD-scion-verdict evidence), so "act on it" has no
  meaning and it is re-parked.
- `adopt report` exists (`adopt-docket-report`, shipped 2026-06-13) and
  wires scan findings to the docket, but on the pre-scion verdict, so it
  faithfully reports the false count.
- The docket protocol is the same one self-review's Phase B.5 consumes;
  a finding that can reach zero auto-resolves (cf.
  `ctrace_sessionend_resolve` closing at `N_MISSING=0`).

## What this builds

Extend `adopt`'s `report`/scan-summary path (no new crate). MSRV 1.85.

- **Count on lineage, not clocks.** The docket finding's headline count
  is the number of artifacts with `verdict == installed-stale` **and**
  `freshness_basis == lineage` (marker fingerprint ≠ committed HEAD) —
  i.e. genuinely behind.
- **Separate the unknowns.** Artifacts still on `clock-fallback` (no
  marker even after reconcile, e.g. a repo with no commits or a
  hand-copied binary) are reported under a distinct, lower-severity
  finding (`adopt-unmarked-installs`) — they need a marker, not a
  reinstall, and must not inflate the "behind" count.
- **Auto-resolve at zero.** When the lineage-behind count is 0, `adopt
  report` resolves `adopt-scan-stale-binaries` (emits the resolve marker
  the docket expects) instead of re-parking it, so self-review stops
  carrying it. `not-installed` artifacts keep their existing separate
  treatment.
- **Per-bucket detail.** `adopt report --format json` lists, per finding,
  the artifacts and their `freshness_basis`, so a human or self-review
  can see *why* something is counted.

## Acceptance criteria

1. With all installs marker-current (post-reconcile on a current laptop),
   `adopt report` reports a lineage-behind count of **0** and resolves
   the `adopt-scan-stale-binaries` docket finding (resolve marker
   emitted), rather than re-parking it. End-to-end test against temp
   docket + temp markers.
2. A single genuinely-behind artifact (marker fingerprint ≠ committed
   HEAD) produces a lineage-behind count of exactly 1 and the finding
   stays open. Test proves real drift is still surfaced.
3. Artifacts on `clock-fallback` (no marker) are reported under
   `adopt-unmarked-installs`, never counted in the `adopt-scan-stale-binaries`
   behind count. Test proves the two buckets are disjoint.
4. `adopt report --format json` includes per-artifact `freshness_basis`
   under each finding. Existing report consumers (the docket-report
   wiring) keep working — no removed/renamed top-level keys they depend
   on without a shim.
5. The resolve path uses the same docket resolve mechanism self-review
   already honors (parity with how a finding reaches zero elsewhere);
   verified by a test that asserts the resolve marker/exit contract.
6. `cargo test` green, `cargo build` clean; no network/live-fleet
   dependency; docket and marker IO go through temp dirs in tests.
