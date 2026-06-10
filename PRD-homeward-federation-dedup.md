# PRD: homeward-federation-dedup — don't double-list the same animal across networks

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md
Depends: homeward-federation-petfbi (this dedups records that connector introduces).
  Extends shipped homeward-ingest (dedup.rs + departure.rs already present).

## TL;DR

Once homeward-federation-petfbi lands, the same physical animal can enter the store
twice: a municipal shelter logs a stray intake (Socrata) *and* the finder files a
"found" report on Pet FBI for the same dog, hours apart. The shipped ingest dedup
(`homeward-ingest/src/dedup.rs`) was built to dedup *within and across shelter sources*
keyed on stable source-id + perceptual hash. A community found-report has no shelter
source-id and a different photo of the same animal, so it slips past as a new record —
and a searching owner sees the same dog listed twice with conflicting contact info.
This PRD extends the dedup pass to reconcile **federated community reports against
shelter intakes**: same species, perceptually-similar photo, compatible found-location
and intake-date window ⇒ one canonical record carrying both provenances, not two.

## Why this exists

Phase-1 evidence (direct grep of shipped homeward-ingest, plus the federation research
2026-06-10):

- **Dedup exists but is source-id-anchored.** `homeward-ingest/src/dedup.rs` ships the
  cross-source dedup the vision's End-state #2 promised ("cross-source dedup (stable
  source-id + perceptual hash)"). Its stable-id arm assumes every record has a
  shelter-issued id. A Pet FBI / HeLP report has its own id namespace and is *about the
  same animal from a different reporter* — the id arm can't bridge it.
- **The overlap is real and load-bearing.** The federation research established that
  Pet FBI/HeLP carry **owner-filed lost and finder-filed found reports**; the shelter
  feeds carry **stray intakes**. A stray that someone also reported "found" is the exact
  case where the two populations intersect — and it's the highest-value case, because a
  found-report + a stray-intake for one animal is strong signal it's someone's lost pet.
  Double-listing it is precisely the "haunting a searching owner" failure the vision's
  End-state #2 and departure detection were built to avoid.
- **The perceptual-hash arm is the right tool, under-used.** dedup.rs already computes
  perceptual hashes for the photo arm; the gap is a *cross-population* match policy
  (visual similarity + structured agreement) rather than a new primitive. departure.rs
  (`departure.rs:43 run_departure_detection`) already shows the pattern of a post-sync
  reconciliation pass this can mirror.
- **The matcher must not be fed duplicates.** `homeward-match` ranks visual neighbors
  for an owner's query; two store records for one animal waste a shortlist slot and
  split the signal. Dedup at ingest is the right layer (before embed/match), matching
  the shipped order schema─►connectors─►ingest─►embed─►match.

## What this builds

An extension to `homeward-ingest/src/dedup.rs` (no new crate):

- **A cross-population reconciliation policy** `federated_merge` run after a federated
  source syncs: for each new federated `PetRecord`/found-report, search existing shelter
  `PetRecord`s within (a) same species, (b) a found-location ↔ intake-location proximity
  bound, (c) an intake-date / found-date window, and (d) perceptual-hash distance below a
  configured threshold. A candidate clearing all four is merged.
- **Merge, not delete.** A merged record keeps a single canonical identity but carries
  **both** `Provenance` entries (shelter + federated), so the owner sees "in Austin
  Animal Center *and* reported found on Pet FBI" — strictly more reclaim signal, with
  honest multi-source attribution. The schema's provenance is already a per-record value;
  this PRD stores a set rather than a single (additive `PetRecord` change if needed, or a
  side provenance list — chosen to minimize schema churn).
- **Conservative thresholds + an audit counter.** Defaults tuned to favor *false-split
  over false-merge* (merging two different animals is worse than listing one twice — it
  could hide a real match). A `DedupConfig` field for each bound; a logged count of
  federated-vs-shelter merges per sync so the rate is observable.
- **No cross-merge of two lost-reports.** Two owners reporting two different lost dogs
  must never merge; this pass only reconciles a *found/stray* federated record against a
  *shelter intake*. Lost-report dedup stays out of scope (owner reports are authoritative
  and owner-controlled).

## Acceptance criteria

1. `cargo build -p homeward-ingest` is clean and workspace `cargo test` is green with
   new dedup tests.
2. A test fixture pairing one Socrata stray intake and one Pet FBI found-report of the
   *same* animal (same species, near location, overlapping date window, perceptually
   close photos) yields **one** canonical record after `federated_merge`, carrying both
   provenances.
3. A negative fixture — same species but distant location, or perceptual-hash distance
   above threshold — yields **two** records (no false merge). At least one test per
   guard dimension (species / geo / date / phash).
4. The merged record's provenance set contains both the shelter `SourceId` and the
   federated `SourceId`; neither is dropped.
5. Two distinct lost-reports never merge with each other (explicit test); the pass only
   reconciles found/stray-federated against shelter intakes.
6. `DedupConfig` exposes the geo, date-window, and perceptual-distance thresholds with
   documented defaults biased toward false-split; changing a threshold changes the merge
   outcome in a test.
7. Per-sync merge counts are logged/returned (observable), asserted by a test that runs a
   mixed batch and checks the reported merge count.
8. `clippy -D warnings` and `cargo deny check bans licenses sources` pass for the crate.

## Notes for /build

- Mirror `departure.rs`'s post-sync-pass structure (`run_departure_detection`) for the
  reconciliation entrypoint — same "after a full sync of source X, reconcile" shape.
- Reuse the existing perceptual-hash computation in dedup.rs; do not add a second
  hashing dependency.
- Bias every threshold toward NOT merging. A wrongly-merged pair can bury a true reclaim
  match — the one outcome the whole vision exists to prevent. When unsure, list twice.
- If `PetRecord` needs a provenance *set* rather than a single value, make it additive
  (e.g., a `Vec<Provenance>` with the existing single retained as the primary) so shipped
  connectors/tests don't break.
