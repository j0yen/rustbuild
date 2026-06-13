# PRD: scion-verdict — make `adopt scan` answer freshness by lineage

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/scion.md

## TL;DR

`adopt scan` decides `installed-stale` by comparing two clocks
(`adopt/src/scan.rs:291`: `installed_ts < src_commit_ts`). Because the
build pipeline installs the binary *before* committing the source it was
built from, that comparison is `true` by construction for every
correctly-built artifact — a permanent false positive. `vest-incremental`
already records the *commit a binary was built from* in an
`InstallMarker` (`adopt/src/marker.rs`), but the verdict never reads it.
This PRD wires the marker into `derive_verdict`: a binary is
`installed-current` iff its marker fingerprint equals the repo's current
committed-HEAD fingerprint; the clock comparison is demoted to a
fallback used only when no marker exists.

## Why this exists (Phase 1 evidence, 2026-06-13)

Measured live this session:

- `adopt scan --format json` reports `installed-stale` with `age_vs_head:
  "0h stale"` for `bon-mot-anagram` (src commit `1781050135`, installed
  `1781050122` — installed **13s before** the commit), `bon-mot-epigram`
  (**5s**), `changeover` (**33s**). None is behind; each is the exact
  commit, off by the build-then-commit gap.
- `adopt/src/scan.rs:291` `derive_verdict` returns `InstalledStale` on
  `its < sts` with no content check. `scan.rs:127` `mtime_ts` reads
  provfs `user.prov.ts` (the binary's build-write time) first.
- `adopt/src/marker.rs:57` `compute_fingerprint` returns `git rev-parse
  HEAD`; `marker.rs:189` `write_marker` persists an `InstallMarker` with
  that fingerprint; `read_marker`, `marker_path`, `compute_fingerprint`
  are all `pub`. A `grep` of `scan.rs`/`verify.rs` for
  `marker|fingerprint|rev-parse|hash` matches nothing — the verdict path
  ignores the lineage data entirely.
- The false-positive set has been re-parked on the docket as
  `adopt-scan-stale-binaries` across self-review runs 2026-06-11/12/13
  (reflective memories `01KTZYJZQY…`, `01KTZS50DF…`).

## What this builds

Extend the `adopt` crate (no new crate). MSRV 1.85, no let-chains;
`sigpipe::reset()` is already first in `main`.

- **Verdict consults the marker.** In `scan.rs`, before the timestamp
  branch, call `marker::read_marker(bin)`. If a marker exists, compute
  the repo's current committed-HEAD fingerprint via
  `marker::compute_fingerprint(repo)` and compare:
  - marker fingerprint **==** current committed-HEAD fingerprint →
    `InstalledCurrent` (basis: lineage).
  - marker fingerprint **!=** current committed-HEAD fingerprint →
    `InstalledStale` (basis: lineage) — a *genuine* behind.
  - If the current fingerprint is a `dirty:*` form (uncommitted tree),
    compare against the committed HEAD commit hash instead, per
    vision open-question 1 (dirty working trees are out of scope; a
    binary built from the last commit is current even while the tree is
    dirty). Practically: derive the comparison fingerprint from `git
    rev-parse HEAD` directly so a dirty tree does not force a false
    stale.
- **Clock fallback only when no marker.** If `read_marker` returns
  `None`, keep the existing `its < sts` logic unchanged — it is the only
  signal available for an unmarked install (legacy installs are minted a
  marker by scion-reconcile).
- **Surface the basis.** Add a `freshness_basis` field to the JSON
  artifact record with values `"lineage"` or `"clock-fallback"`, so a
  reader (and scion-truth) can tell a proven verdict from a heuristic
  one. The table output gains no new column but may annotate
  clock-fallback rows (e.g. `0h stale?`).
- **No behavior change to `apply`.** This PRD touches only the scan/
  verdict path. `derive_verdict` is currently `const fn`; it becomes a
  regular `fn` (marker IO is not const) — keep the signature otherwise
  stable and the timestamp logic byte-identical in the fallback arm.

### Shape

- `scan.rs`: `derive_verdict` gains `bin: &str, repo: &Path` params (or a
  small helper `verdict_with_lineage(...)` wrapping the existing const
  fn for the fallback arm).
- `types.rs`: add `pub freshness_basis: FreshnessBasis` (enum
  `Lineage | ClockFallback`, `Serialize`) to the artifact record.
- Reuse `marker::{read_marker, compute_fingerprint}` — do not duplicate
  the fingerprint logic; a marker written by `apply` and a fingerprint
  read by `scan` MUST come from the same function or they will never
  agree.

## Acceptance criteria

1. `derive_verdict` (or its replacement) returns `InstalledCurrent` when
   a marker exists whose fingerprint equals the repo's current
   committed-HEAD fingerprint, regardless of the `installed_ts` /
   `src_commit_ts` ordering. Unit test with a fixture marker proves a
   binary that is "0h stale" by clocks is `installed-current` by lineage.
2. `derive_verdict` returns `InstalledStale` (basis lineage) when a
   marker exists and its fingerprint differs from current committed HEAD.
   Unit test proves a genuinely-behind marker still reports stale.
3. When no marker exists, the verdict is byte-for-byte the existing
   `its < sts` behavior. Unit test pins the fallback path unchanged.
4. A repo with a dirty working tree but an installed binary built from
   the current committed HEAD reports `installed-current`, not stale.
   Unit test with a dirty-fingerprint repo proves no false stale.
5. `adopt scan --format json` emits `freshness_basis` on every artifact
   record; value is `"lineage"` when a marker was consulted, else
   `"clock-fallback"`. Existing keys (`repo`, `bin`, `verdict`,
   `is_daemon`, `fix_cmd`) remain present and unchanged.
6. The fingerprint used by `scan` is produced by the same
   `marker::compute_fingerprint` used by `apply`'s marker writer
   (verified by a test that writes a marker via the apply path and reads
   `installed-current` via scan, with no reinstall between).
7. `cargo test` green; `cargo build` clean. No live-fleet or network
   dependency; all tests hermetic (temp git repos / temp marker dir via
   `XDG_STATE_HOME` override).
