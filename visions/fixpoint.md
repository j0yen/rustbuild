# Vision: fixpoint — the adoption pipeline must actually reach zero, and prove it

> A control loop that never measures its own error is not a control
> loop — it is a hamster wheel with good intentions. The laptop built
> six tools to detect and fix binary staleness (`vigil`, `scion`,
> `vest`, `adopt`, `changeover`, `rollout`). Run after run, self-review
> still reports a flat "N/N not-current." Nobody has wired the cure to
> run autonomously, nobody has separated real debt from accounting
> noise, and nobody watches whether the number trends to zero.
> `fixpoint` is the discipline of *convergence*: make the pipeline run
> its own cure on every cron tick, report a true count, and escalate
> when it stops shrinking.

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-13
**Status:** active
**Seed:** bare `/dream` (interactive). `fallow check` = fresh
  (state=fresh, streak=0, last_productive 2026-06-13T20:05Z). Phase-1
  live inspection of `adopt scan/verify/reconcile --dry-run` against the
  running `adopt-cron.service`. Recall ideation seeds (reflective/self)
  were dominated by self-review notes parking `fleet-binary-staleness`
  and `adopt-scan-stale-binaries` — the chronic, never-converging finding.

---

## TL;DR

`scion` (shipped 2026-06-13, three PRDs) gave `adopt` a *lineage* answer
to "is this binary fresh?": a binary is current iff its install marker's
source fingerprint equals committed HEAD, with the old timestamp
comparison demoted to a `clock-fallback` used only when no marker exists.
The machinery landed. The world did not move:

```
$ adopt scan --format json | jq 'group_by(.freshness_basis)[]|"\(.[0].freshness_basis): \(length)"'
clock-fallback: 16        # ALL 16 not-current still on the fallback basis — zero markers minted
```

scion built the cure (`adopt reconcile` mints markers without a rebuild).
Phase-1 inspection found three concrete, measured reasons the cure never
reaches the patient, and a fourth reason nobody would notice if it did:

1. **The cron never runs `reconcile`.** `adopt-cron.service` ExecStart is
   `adopt apply --execute --with-daemons` then `adopt report`. `apply`
   *reinstalls* stale binaries (updating `installed_ts`) but cannot mint a
   lineage marker — only `reconcile` does. So every 6h the cron reinstalls
   the same binaries and they flap straight back to `installed-stale` on
   the next source commit. A hamster wheel: motion without convergence.

2. **`reconcile` skips dirty working trees.** `adopt reconcile --dry-run`
   shows it would seed markers for ~dozens of clean repos but
   `skip … (dirty working tree)` for every bon-mot-* repo and others.
   ~30 repos are dirty (every self-review). Those can *never* get a marker
   under current behavior → a permanent clock-fallback residue that keeps
   the count off zero no matter how many cron ticks run.

3. **`verify`'s one bucket hides the real problem.** `adopt verify`
   classifies all 16 as a single reason `SourceNewer`, but the `DETAIL`
   column tells the true story: twelve are "0d newer" (source committed
   the same day, often seconds apart — build-order noise / clock-fallback),
   while four are genuinely behind: `wm-audio` 8d, `wm-dialog` 7d,
   `wm-tts` 6d, `wm-reach` 2d. The four real, actionable stale daemons are
   buried in a count of sixteen. Self-review reads "16/16 not-current" and
   reasonably treats it as undifferentiated backlog.

4. **Nothing measures convergence.** The headline went 84/84 (2026-06-11)
   → 16/16 (2026-06-13) — a 5× change — entirely unremarked, because no
   instrument stores the count over time or asserts it is non-increasing.
   Whether that drop was real progress or a tracking-set change, no one can
   say. A control loop with no error signal cannot know if it is working.

`fixpoint` closes the loop: wire the cure into the cron, make the cure
handle the dirty-tree residue honestly, give `verify` a true denominator,
and add a convergence ledger that escalates when the real-behind count
rises or stalls. Distinct from `scion` (which built the marker *verdict*
and the `reconcile` *actuator*) — `fixpoint` makes them *run themselves to
a measured zero*.

## End-state

When this is done:

- `adopt-cron.service` runs `reconcile --execute` before `apply` every
  cycle. Clean, not-behind installs get markers minted autonomously; the
  clock-fallback false-stale set drains to zero without a single rebuild.
- `adopt reconcile` no longer leaves dirty-tree repos in permanent limbo:
  a dirty working tree whose *committed HEAD* matches the installed binary
  gets a marker seeded from that committed fingerprint (uncommitted source
  is irrelevant to what the binary was built from), OR — if that is judged
  unsafe — dirty-skipped repos are classified into their own bucket so they
  stop inflating the stale count under a false label.
- `adopt verify` splits `SourceNewer` into `SourceNewer-sameday`
  (≤1d, build-order / clock noise) and `SourceNewer-behind` (≥2d, genuine
  drift), so self-review and docket see the true actionable count — today
  that would read "4 genuinely behind (wm-audio, wm-dialog, wm-tts,
  wm-reach), 12 noise" instead of "16 not-current."
- A convergence ledger records `{total, behind, dirty-blocked, fallback}`
  per cron run, asserts `behind` is non-increasing, and emits a docket
  finding (`fixpoint-not-converging`) if it rises or fails to reach zero
  after a configured number of runs — so the pipeline's own health is
  observable, not inferred from prose in a journal.

## Components (one bullet per future PRD)

- **fixpoint-cron-reconcile** (config) — edit `adopt-cron.service` to run
  `adopt reconcile --execute` immediately before `adopt apply --execute`.
  Drains the clock-fallback false-stale set on the next cron tick with no
  rebuild. Independent; ships first; immediate measured effect.
- **fixpoint-verify-resolution** (rust-extend `~/wintermute/adopt`) — split
  `verify`'s `SourceNewer` reason into `SourceNewer-sameday` (≤1d) and
  `SourceNewer-behind` (≥2d, threshold configurable), surfaced in both
  table and JSON. Gives the pipeline a true denominator. Independent.
- **fixpoint-dirty-reconcile** (rust-extend `~/wintermute/adopt`) —
  teach `reconcile` to seed a marker from committed HEAD when a working
  tree is dirty *but* the installed binary matches that committed
  fingerprint; otherwise emit a distinct `dirty-blocked` classification
  instead of leaving the repo silently in the stale count. Independent of
  the above but shares `reconcile`/marker internals with scion.
- **fixpoint-converge-ledger** (rust-extend `~/wintermute/adopt`) —
  persist a per-run convergence record under `~/.local/state/adopt/` (or
  the existing adopt state dir), expose `adopt converge` to print the
  trend, assert `behind` non-increasing, and emit/resolve a
  `fixpoint-not-converging` docket finding. Depends on
  fixpoint-verify-resolution for the `behind` count.

## Order

```
fixpoint-cron-reconcile        (config; ship now, independent)
fixpoint-verify-resolution ──▶ fixpoint-converge-ledger
fixpoint-dirty-reconcile       (independent; reconcile internals)
```

`fixpoint-cron-reconcile` is pure config and can land and prove itself
immediately (the next cron tick should drop the clock-fallback count).
`fixpoint-converge-ledger` needs the `behind` bucket that
`fixpoint-verify-resolution` introduces. `fixpoint-dirty-reconcile` shares
marker internals with scion and can land any time.

## Open questions (for the user / next /dream pass)

- **Dirty-tree marker safety.** Is it ever correct to mint a lineage
  marker for a repo with uncommitted changes? The conservative answer is
  "marker = committed HEAD fingerprint; uncommitted source is not what the
  binary was built from, so a clean-built binary at HEAD is genuinely
  current." But if someone built from a dirty tree and never committed,
  the marker would lie. fixpoint-dirty-reconcile should require that the
  installed binary's build provenance matches committed HEAD before
  seeding, and fall back to `dirty-blocked` otherwise. Confirm the
  appetite for the marker-seeding path vs. classification-only.
- **sameday threshold.** Is ≤1d the right "noise" cutoff for verify
  resolution, or should it be lineage-strict (only a present-and-matching
  marker counts as current, everything else is "behind-or-unknown")? The
  latter is purer but would, until reconcile fully runs, label everything
  behind. Start with the 1d clock heuristic; tighten to lineage-strict
  once fixpoint-cron-reconcile has drained the fallback set.
- **Where does the convergence ledger live** — a new file under the adopt
  state dir, or folded into the docket ledger as a time-series finding?
  Start standalone (adopt owns its own convergence record); docket gets
  only the escalation finding.
- **Should `apply` and `reconcile` be reordered or interleaved** so a
  freshly-reinstalled binary is immediately marked in the same tick?
  Likely yes (apply → reconcile, so the just-installed binary gets a
  marker), but verify against the dirty-tree skip behavior first.
