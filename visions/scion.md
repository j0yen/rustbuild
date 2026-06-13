# Vision: scion — freshness is a question of lineage, not of which clock ran last

> A scion is a cutting grafted from a parent: it *is* the parent's
> tissue, carried into new wood. An installed binary is a scion of the
> commit it was built from. Ask "is this binary fresh?" and the honest
> answer is genealogical — *which commit is it descended from?* — not
> chronological — *was its file written before or after some other
> timestamp?* `adopt` currently answers the chronological question, and
> the build pipeline makes that answer wrong by construction.

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-13
**Status:** active
**Seed:** bare `/dream` (interactive, manual). `fallow check` = fresh
  (state=fresh, streak=0, last_productive 2026-06-13T18:33Z). Phase-1
  live inspection of `adopt scan --format json` against `adopt/src/scan.rs`
  and `adopt/src/marker.rs`. The strongest *unaddressed* signal this pass
  was not a missing tool and not a daemon-restart gap (changeover covers
  that) — it was a **freshness verdict that is false by construction for
  every correctly-built artifact**, sitting one unwired function call
  away from being correct.

---

## TL;DR

`adopt scan` decides an installed binary is `installed-stale` with one
comparison (`adopt/src/scan.rs:291`, `derive_verdict`):

```rust
if its < sts { return Verdict::InstalledStale; }  // its = install ts, sts = newest src/ commit ts
```

`its` is read from the binary's provfs `user.prov.ts` xattr (set at the
moment the binary file is *written* during build; `scan.rs:127`). `sts`
is the timestamp of the newest `src/` commit. The build pipeline does
**`cargo install` first, then `git commit`** — so for any artifact built
the normal way, the binary is written *seconds before* the commit it was
built from is recorded. `its < sts` is therefore **structurally
guaranteed** for a freshly, correctly built binary. The verdict reads
`installed-stale` not because the binary is behind, but because two
clocks were sampled in the order the pipeline always uses.

Measured live this session (`adopt scan --format json`):

| repo | src commit ts | installed ts | Δ | verdict |
|---|---|---|---|---|
| bon-mot-anagram | 1781050135 | 1781050122 | **13s** | `0h stale` |
| bon-mot-epigram | 1781056298 | 1781056293 | **5s** | `0h stale` |
| changeover | 1781376731 | 1781376698 | **33s** | `0h stale` |

These are not stale. They are the exact commit, off by the
build-then-commit gap. The docket has carried
`adopt-scan-stale-binaries` (84/84, now 16/16) "not-current" for run
after run (self-review reflective memories `01KTZYJZQY…` 2026-06-13,
`01KTZS50DF…` 2026-06-12) — a permanent false alarm that **buries any
genuinely-behind binary** in noise the review re-parks every day.

The fix already half-exists. `vest-incremental` shipped a content
fingerprint — `SourceFingerprint` = `git rev-parse HEAD` of the repo —
and an `InstallMarker` written to `$XDG_STATE_HOME/adopt/markers/<bin>.json`
recording *which commit the installed binary was built from*
(`adopt/src/marker.rs:57,189`). But that marker is consumed **only** to
skip redundant reinstalls in `adopt apply`. **`adopt scan`'s verdict
never reads it.** The lineage is recorded and then ignored at the exact
moment a lineage answer is needed.

`scion` is the discipline of answering freshness genealogically: a
binary is *current* iff its recorded source-commit equals the repo's
current committed HEAD — proven by the fingerprint, not inferred from
clock ordering. The timestamp comparison survives only as a fallback for
artifacts with no marker yet.

This is the same epistemic move `plumb` makes for self-review probes ("a
reading taken without ground truth is a guess in a verdict's clothes")
and the same hash-binding `changeover-autoapply` wants for its proof
ledger ("proof bound to the daemon's binary hash"). scion builds the
shared primitive both reach for.

## Why this exists (Phase 1 evidence, 2026-06-13)

Measured live this session, against the source and the live store:

- **The verdict is clock-ordered.** `adopt/src/scan.rs:291` —
  `derive_verdict` returns `InstalledStale` on `its < sts` with no
  content check. `scan.rs:127` — `mtime_ts` reads provfs `user.prov.ts`
  (build-write time) before falling back to file mtime.
- **The build order guarantees the false positive.** The three live
  rows above all show installed ts a few seconds *before* the commit
  they descend from — the build-then-commit ordering, not staleness.
- **The lineage data exists but is unconsumed by the verdict.**
  `adopt/src/marker.rs:57` `compute_fingerprint` returns `git rev-parse
  HEAD`; `marker.rs:189` `write_marker` persists it; `read_marker`,
  `marker_path`, `compute_fingerprint` are all `pub`. `grep` of
  `scan.rs`/`verify.rs` for `marker|hash|fingerprint|rev-parse` finds
  nothing — the verdict path does not touch any of it.
- **No markers exist yet.** `$XDG_STATE_HOME/adopt/markers/` does **not
  exist** (0 files). Markers are written only on an actual reinstall,
  which the 16 currently-installed binaries have not had since
  vest-incremental shipped. So even wiring the verdict to the marker
  helps nothing until the existing installs are *reconciled* — given a
  marker without a needless rebuild.
- **The noise is metered.** Same false "stale" set re-parked across
  self-review runs 2026-06-11/12/13; the count moved 84→16 only because
  adopt-cron churned reinstalls, not because anything got proven fresh.

## End-state

- `adopt scan` calls a binary `installed-current` iff its `InstallMarker`
  fingerprint equals the repo's current committed-HEAD fingerprint.
- The clock comparison (`its < sts`) is used **only** as a fallback when
  no marker exists, and is labeled as a heuristic in the JSON output.
- Every already-installed binary has a marker — minted once by an
  explicit reconcile pass, then kept authoritative by real installs —
  so the false-positive floor is gone and `adopt scan` returns **zero**
  stale for a fully-current laptop.
- The docket receives a *lineage-proven* "genuinely behind" count
  (marker fingerprint ≠ HEAD), so self-review acts on real drift instead
  of re-parking a fabricated 16.
- `changeover-autoapply` (when built) and any future "is what's running
  descended from HEAD" question can read the same marker rather than
  inventing a parallel proof.

## Components (one bullet per future PRD)

- **scion-verdict** — `adopt scan`'s verdict consults the `InstallMarker`
  `SourceFingerprint`: `installed-current` iff marker fingerprint ==
  current committed-HEAD fingerprint; demote `its < sts` to a fallback
  used only when no marker exists, and surface which basis was used in
  the JSON (`freshness_basis: "lineage" | "clock-fallback"`).
- **scion-reconcile** — `adopt reconcile`: for every installed binary
  with no marker, mint one. Seed it from current committed HEAD when the
  install is provably not behind (binary present + repo HEAD unchanged
  since the marker would have been written); the seed is clock-informed
  *once*, after which the marker is authoritative. Clears the legacy
  false-positive set without a rebuild.
- **scion-truth** — `adopt report`/scan emits a docket finding keyed on
  the *lineage-proven* behind count (marker ≠ HEAD), distinct from
  "no-marker / unknown", so self-review's `adopt-scan-stale-binaries`
  reflects real drift and auto-resolves at zero.

## Order

```
scion-verdict ──► scion-reconcile ──► scion-truth
```

`scion-verdict` first: it is the one-function fix that makes the marker
authoritative for the verdict (and is correct the moment any marker
exists). `scion-reconcile` next: it populates markers for the existing
installs so the verdict has data to act on — without it, scion-verdict
silently falls back to the broken clock for every legacy binary.
`scion-truth` last: it reframes the docket signal on top of a verdict
that is now lineage-based.

## Open questions

1. **Dirty trees.** `compute_fingerprint` returns `dirty:<max_mtime>`
   for an uncommitted working tree, which can never equal a clean-commit
   marker. The vision's stance: freshness compares against *committed*
   HEAD (`git rev-parse HEAD`), and uncommitted changes are out of scope
   — they are not shipped, so a binary built from the last commit is
   "current" even while the tree is dirty. scion-verdict should compare
   against the commit hash, not the dirty fingerprint. Confirm this is
   the intended semantics before scion-truth keys the docket on it.
2. **Embedded provenance (rigorous form).** The marker is external state
   (a sidecar JSON); it can drift from the binary if the file is moved
   or hand-copied. The eventual rigorous form embeds the source commit
   *into* the binary at build time (a `build.rs` stamp / version
   string), so the binary is self-describing. That retrofits `build.rs`
   across dozens of crates and belongs to a later pass — left here, not
   drafted, until the sidecar marker proves insufficient.
3. **Shared proof with changeover.** `changeover-autoapply` (drafted
   2026-06-13, not yet built) wants a proof ledger "bound to the
   daemon's binary hash." Should it consume scion's marker directly, or
   does a *running* daemon need a binary-hash proof distinct from a
   *commit* fingerprint (a daemon can run stale bytes from a deleted
   binary)? Resolve when changeover-autoapply is built; scion-verdict
   stays commit-fingerprint-based regardless.
