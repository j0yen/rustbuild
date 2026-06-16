# Vision: careen — scrape build-cruft off living target dirs

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-16
**Status:** active
**Fleet 1 drafted:** 4 PRDs (survey → sweep → {guard ∥ ledger})

---

## TL;DR

`ballast` reaps whole dead target dirs; `hold` shares one target dir
across repos to stop duplication accruing. Neither touches the *inside*
of a target dir that belongs to a **living, in-use** repo. But that's
where the heaviest weight sits: `~/wintermute/recall/target` is 13G, of
which `debug/deps` alone is **9.2G** — a midden of every old-version
`.rlib` cargo ever compiled and never garbage-collects. `recall` and
`wintermute-brain` (13G each) are the two largest target dirs on the
box, both LIVE: ballast can't reap them (their installed binary is
current), and hold can't drain them mid-flight. They are stuck.

careen is the third disk axis: **intra-target reclamation for repos
still in service**. To careen a ship is to heel it over and scrape the
barnacles off the hull *without scrapping the vessel* — exactly this:
prune stale `incremental/`, orphaned `deps/` rlibs, wrong-toolchain
fingerprints, and dead `build/` script outputs from a target dir a
clean rebuild would regenerate, while never corrupting an in-flight
build.

## Why now (evidence, 2026-06-16)

- `df / ` = **97%, 17G free**. Self-review has flagged accelerating
  growth three runs running (86% → 92% → 97%).
- `du -sch ~/wintermute/*/target` = **215G total**.
- `recall/target/debug/deps` = **9.2G** in one repo; 295 `.rlib` files,
  most superseded by newer dep versions but never collected.
- `find ~/wintermute/*/target/debug/incremental -maxdepth 1 -type d
  -mtime +7` = **2550 stale incremental dirs** fleet-wide.
- Self-review cross-session digest: top write prefixes are
  `target/debug/incremental (748)`, `target/debug/.fingerprint (570)`,
  `target/debug/deps (285)` — the churn is *inside* target dirs.
- `cargo-sweep` is **not installed**; nothing on the box does
  timestamp- or lockfile-aware intra-target pruning today.

## End-state

When this is done:

- A read-only survey can size, per repo, the reclaimable-by-careening
  bytes (stale incremental, orphaned deps, wrong-toolchain, dead build
  outputs) — distinct from ballast's reclaimable-by-deletion.
- A sweep tool prunes those classes from a single repo's target dir,
  dry-run by default, lock-aware (never corrupts an in-flight build),
  and provably safe (only removes artifacts a clean rebuild regenerates).
- A guard arm careens the largest *live* targets automatically when disk
  crosses a watermark — catching the giants (recall, brain) that
  ballast-guard structurally cannot reap.
- A ledger records each sweep's reclaimed bytes AND the rebuild cost it
  forced on the next build, so the fleet learns whether careening a
  given repo nets out positive or just thrashes the compiler.

## Components (PRD-sized)

1. **careen-survey** (keystone, read-only) — per-target-dir
   classification of reclaimable-by-careening bytes across four classes:
   stale `incremental/` (mtime-aged), orphaned `deps/` rlibs (versions
   not referenced by current `Cargo.lock` / live `.fingerprint`),
   wrong-toolchain artifacts (1.85 vs 1.88 buckets), dead `build/`
   script-output dirs. Emits JSON. Ships standalone first to size the
   prize. Explicitly orthogonal to `ballast-survey` (whole-dir) — careen
   is the *intra*-dir complement.

2. **careen-sweep** — the reclaimer for a single repo. Lock-aware
   (respects cargo's target-dir advisory flock; refuses to run against a
   building target), dry-run default, `--apply` gated. Only prunes
   classes a clean `cargo build` regenerates; never the live binary,
   never the lockfile, never current-toolchain fingerprints. Emits what
   it removed and the bytes reclaimed.

3. **careen-guard** — SLO-triggered automatic careen of the largest LIVE
   target dirs when disk breaches a watermark. Composes ballast-guard's
   `Event` schema (`event.rs`: level/used_pct_before/after/
   bytes_reclaimed/reclaimable_bytes/candidates/ts) — emits the same
   shape, does NOT fork it. Targets exactly the repos ballast-guard skips
   (binary-current, in-use). Reuses careen-sweep as its reclaim engine.

4. **careen-ledger** — append-only accounting of each sweep: bytes
   reclaimed, and the rebuild cost the *next* build paid (wall-time /
   recompiled-crate count) for what was pruned. Closes the feedback loop
   so the guard can learn which repos are worth careening vs which just
   force a thrash. Distinguishes careen from a naive `cargo clean`.

## Order

```
careen-survey  →  careen-sweep  →  { careen-guard ∥ careen-ledger }
```

- survey is independent and read-only — build first to size the prize.
- sweep is the load-bearing reclaim engine.
- guard and ledger both consume sweep, independent of each other.

## Relationship to ballast / drydock / hold

| vision  | axis                                   | reclaim mode            |
|---------|----------------------------------------|-------------------------|
| ballast | whole dead target dirs (fossil-first)  | by deletion             |
| hold    | one shared target across repos         | by sharing (preventive) |
| drydock | fleet drift inventory                  | survey only             |
| careen  | inside living target dirs              | by intra-dir scraping   |

careen is **not** a duplicate of ballast: ballast deletes a whole target
dir whose binary is stale/uninstalled; careen scrapes cruft from a
target dir whose binary is *current and in use* — a dir ballast must
leave alone. The two are complementary and the guard arms should
coordinate (open question #1).

## Open questions

1. Should `careen-guard` and `ballast-guard` share a single watermark
   evaluator (one reads disk, dispatches to whichever reclaimer has safe
   candidates) rather than two timers racing on the same mount? Likely
   yes — factor the watermark/Event logic into a shared crate.
2. Orphaned-rlib detection: is parsing `Cargo.lock` + `.fingerprint`
   sufficient to prove an rlib is dead, or does careen need to actually
   ask cargo (e.g. `cargo build --build-plan` / metadata) to be safe?
   Survey should validate the cheap path before sweep trusts it.
3. Does careen interact badly with `hold`? Once repos share one target
   dir (hold-anchor), careen operates on the shared hold instead of
   per-repo — the orphaned-rlib set is then fleet-union, not per-repo.
   Sequence: careen is useful NOW (pre-hold); after hold lands, careen's
   target is the single hold dir. Both remain valid.
4. sccache (hold-sccache) changes the deps/ economics — cached objects
   live in sccache, not deps/. Does careen-survey need an sccache-aware
   mode? Defer until hold-sccache ships.
