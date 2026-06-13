# PRD: homeward-deliver-enroll — the gallery the matcher queries is actually built

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

The matcher can only return a real shortlist if the shelter-intake photos have
been embedded and indexed — that is the *gallery* side of kNN. homeward has the
client to do it (`/enroll`) but the ingest daemon never calls it: every intake
photo is stored as a hotlink URL and nothing is ever embedded. This PRD wires the
ingest daemon to call `/enroll` for each new or changed intake photo as records
land, so the vector index the owner query searches against is populated honestly.

## Why this exists

Phase-1 live inspection (2026-06-13, `~/wintermute/homeward`):

- `EmbedClient` exposes `POST /enroll` ("embed + index one intake photo",
  `embed_client.rs:5`) but it is **never called** anywhere in the workspace
  (no `.enroll(` outside the definition).
- `homeward-ingest/src/dedup.rs:299` states a record is "passed to
  `homeward-match` which runs the full embedding pipeline" — but no such pass
  exists; `dedup.rs:272` admits the perceptual hash is "a stand-in until
  homeward-embed lands." homeward-embed *has* landed (Python sidecar +
  `embed_client`); the wire was never built.
- `homeward-ingest/src/events.rs:3` already says "Consumers (homeward-embed,
  homeward-match) subscribe to deltas" — the delta event stream the ingest
  daemon emits (`main.rs`) is the natural trigger point for enrollment, but
  nothing currently consumes it to enroll.
- Without enrollment, `homeward-match`'s `visual_scores` (`report.rs:28`) are
  always empty and the fusion matcher silently degrades to geo+date only. The
  visual half of the whole vision is inert until this wire exists.

Depends on **homeward-embed-client** (the shared client extraction). This is a
`rust-extend` of `homeward-ingest`.

## What this builds

- **An enrollment step in the ingest daemon** (`homeward-ingest/src/main.rs` +
  a new `enroll.rs` module): when a record is newly ingested or its photo set
  changes, enqueue each `PhotoRef` for `/enroll` via `homeward-embed-client`,
  keyed by the record's `canonical_id` so the index entry maps back to the
  canonical record the matcher ranks.
- **Honest degradation when the sidecar is absent.** If
  `EmbedClientConfig::from_env` points nowhere or `/health` fails, enrollment is
  **skipped with a logged warning**, never an ingest-failing error — ingest must
  keep aggregating even with no embedder (same posture as alert-delivery's
  dry-run default). A counter/log records how many photos went un-enrolled.
- **Idempotent + change-aware enrollment.** A photo already enrolled (same
  `canonical_id` + photo URL) is not re-embedded on every poll; re-enroll only on
  a changed photo set. Departure/expiry (existing `departure.rs`) should be able
  to signal index removal, or at minimum the design notes how stale gallery
  entries are reconciled (full removal wire may be a follow-on if non-trivial).
- **Backpressure-aware.** Enrollment runs off the delta stream, not inline in the
  poll hot path, so a slow embedder cannot stall aggregation.

## Acceptance criteria

1. `cargo build` and `cargo test` pass for the workspace with `homeward-ingest`
   depending on `homeward-embed-client`.
2. On ingesting a record with N photos, the daemon issues N `/enroll` calls
   carrying the record's `canonical_id` — proven by a test against a mock/stub
   embed endpoint that records the requests it received.
3. With no embed sidecar configured/reachable, ingest completes a full poll
   cycle successfully, enrolls nothing, and logs a clear "embedder unavailable —
   K photos un-enrolled" warning (asserted by a test; no error, no panic).
4. Re-running ingest over an unchanged record set issues **zero** new `/enroll`
   calls (idempotence), while a changed photo set re-enrolls only the changed
   photos (asserted by a test).
5. Enrollment runs off the delta/event path, not inline in the connector poll —
   a slow (artificially delayed) embed endpoint does not increase measured poll
   latency beyond a documented bound (asserted by a timing test or a structural
   assertion that the poll path does not await enroll).
6. `cargo clippy` clean at the workspace lint level; no new `-D` regressions.
