# PRD: scion-reconcile — mint lineage markers for already-installed binaries

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/scion.md
Depends: PRD-scion-verdict.md

## TL;DR

scion-verdict makes `adopt scan` answer freshness by reading each
binary's `InstallMarker`. But **no markers exist** —
`$XDG_STATE_HOME/adopt/markers/` is absent (0 files), because markers are
written only on an actual reinstall, which the currently-installed
binaries have not had since `vest-incremental` shipped. Without markers,
scion-verdict silently falls back to the broken clock comparison for
every legacy install and the false-positive floor stays. This PRD adds
`adopt reconcile`: a one-shot pass that mints a marker for every
installed-but-unmarked binary that is provably not behind, so the
verdict has lineage data to act on — **without a needless rebuild**.

## Why this exists (Phase 1 evidence, 2026-06-13)

Measured live this session:

- `$XDG_STATE_HOME/adopt/markers/` (i.e. `~/.local/state/adopt/markers/`)
  **does not exist**; `ls` returns "No such file or directory", count 0.
- `adopt scan --format json` lists 16 `installed-stale` binaries
  (`adopt`, `bon-mot-*`, `changeover`, `claude-self`, …), all of which
  scion-verdict would still read as stale via clock-fallback because none
  has a marker.
- `adopt/src/marker.rs` already provides `compute_fingerprint`,
  `write_marker`, `read_marker`, `marker_path` (all `pub`) and the
  `InstallMarker` shape — the writer exists; nothing currently calls it
  outside the `apply` reinstall path.

## What this builds

A new `reconcile` subcommand on the existing `adopt` CLI. MSRV 1.85, no
let-chains; reuse `marker::*` — do not reimplement fingerprinting.

- **`adopt reconcile`** — for each repo artifact that is *installed* and
  has *no marker*:
  1. Compute the current committed-HEAD fingerprint
     (`marker::compute_fingerprint`, commit-hash form; skip / warn on a
     `dirty:*` result rather than stamping a dirty fingerprint).
  2. Decide whether the install is *provably not behind* using the
     existing signals available at reconcile time — the install is
     seeded as current when the binary is present **and** the repo HEAD
     has not advanced past what a marker written at install time would
     have recorded. Concretely (conservative seed): treat the installed
     binary as built from current HEAD only when `installed_ts >=` the
     *previous* commit's timestamp window — i.e. it is the same
     build-then-commit pair scion-verdict's clock-fallback would
     otherwise mislabel. When the install is genuinely behind (HEAD
     advanced by a real later commit), do **not** seed a current marker;
     leave it for a real reinstall.
  3. Write the marker via `marker::write_marker`.
- **Idempotent.** Re-running `reconcile` is a no-op for binaries that
  already have a marker (never overwrite an authoritative marker with a
  clock-seeded one).
- **`--dry-run`** — print what would be minted (bin, repo, fingerprint,
  decision) without writing. Default run writes.
- **`--force-reinstall`** escape hatch — for a binary whose lineage
  cannot be seeded safely, defer to the existing `apply` path rather than
  guessing; reconcile does **not** rebuild anything itself.
- **Honest seed labeling.** The minted marker records that it was
  clock-seeded by reconcile (e.g. an `origin: "reconcile-seed"` field on
  `InstallMarker`, distinct from `"install"`), so a future audit can tell
  a proven-at-install marker from a one-time seed. After seeding, the
  marker is authoritative; subsequent real installs overwrite it with an
  `"install"`-origin marker.

## Acceptance criteria

1. `adopt reconcile` creates `$XDG_STATE_HOME/adopt/markers/<bin>.json`
   for each installed-but-unmarked artifact it deems current, with a
   fingerprint equal to `marker::compute_fingerprint` of the repo's
   committed HEAD. Test against a temp `XDG_STATE_HOME` + temp git repos.
2. After `adopt reconcile`, `adopt scan` (with scion-verdict) reports
   `installed-current` (basis: lineage) for a binary that was
   `installed-stale` (basis: clock-fallback) beforehand — proving the
   floor is cleared without reinstall. End-to-end test.
3. A genuinely-behind install (repo HEAD advanced by a real later commit
   after the binary's install time) is **not** seeded a current marker;
   `reconcile` leaves it markerless (or marks it behind) so scan still
   reports it stale. Test proves no false-current seeding.
4. `adopt reconcile` is idempotent: a second run writes no new markers
   and overwrites no existing marker. Test asserts byte-identical marker
   set across two runs.
5. `adopt reconcile --dry-run` writes nothing and prints the planned
   actions. Test asserts the markers dir is unchanged after a dry run.
6. Minted markers carry an `origin` distinguishing `reconcile-seed` from
   `install`; a real `apply` reinstall overwrites a seed with an
   `install`-origin marker. Test covers the overwrite direction.
7. `reconcile` never invokes `cargo`/`rollout` or the network; it only
   reads git + writes marker JSON. `cargo test` green, `cargo build`
   clean, all tests hermetic.
