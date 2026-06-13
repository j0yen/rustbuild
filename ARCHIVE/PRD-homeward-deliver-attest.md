# PRD: homeward-deliver-attest — prove the owner round-trip on real fixtures

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

Once the gallery is enrolled (`homeward-deliver-enroll`) and the owner query is
wired (`homeward-deliver-query`), homeward can — for the first time — answer the
question the whole vision exists to answer: *given one photo of a lost pet, does
the right shelter animal come back at the top of the shortlist?* This PRD builds
the end-to-end attestation: a reproducible smoke that stands the embed sidecar
up, enrolls the bundled gallery, submits the bundled query photo through
`homeward-reportd`, and asserts the correct individual ranks first — with a
measured latency number. The "made to deliver" capstone.

## Why this exists

Phase-1 live inspection (2026-06-13, `~/wintermute/homeward`):

- The fixtures for an honest smoke already exist:
  `homeward/embed/fixtures/eval-smoke/gallery/` (`dog1_photo1.png`,
  `dog2_photo1.png`, `cat1_photo1.png`) and `.../query/cat1_photo2.png` — a
  second photo of `cat1`, the held-out query whose correct answer is `cat1`.
- Operate-fleet gossip (2026-06-13) named the frontier exactly: homeward is "a
  library fleet that has never been operated, provisioned, proven, or made to
  deliver." The deliver fleet wires the parts; nothing yet *proves the wired
  whole works* against a known-answer input.
- The honesty bar is non-negotiable here ([[feedback_agent_written_fixtures_
  tautology]]): the attestation must use the **real** DINOv2 embedder over real
  photos, gallery and query must be photos the embedder did not co-generate, and
  the asserted top-1 must be the genuinely-correct individual — not a fixture
  written to make the test pass. This mirrors the continuity-e2e-attest and
  homeward-eval-harness disjoint-split discipline already in the ecosystem.

Depends on **homeward-deliver-enroll** and **homeward-deliver-query** (both
halves must be wired before the round-trip can be proven). `mixed` because it
drives the Python sidecar and the Rust `reportd`/ingest binaries together.

## What this builds

- **An attestation script/subcommand** (`homeward attest deliver`, or a
  `deploy/`-adjacent script invoked by a test) that, hermetically:
  1. starts the embed sidecar (`homeward-embed-svc`) on an ephemeral port with a
     provisioned (offline-capable, per embed-provision) DINOv2;
  2. enrolls the three `eval-smoke/gallery` photos through the real ingest enroll
     path (or the embed `/enroll`) under known `canonical_id`s;
  3. constructs a lost report for a cat, attaches `eval-smoke/query/cat1_photo2.png`,
     and runs `homeward-reportd match --report <id>` against the live sidecar;
  4. asserts the top-ranked candidate's `canonical_id` is `cat1` (the correct
     held-out individual), and that the two dogs rank below it;
  5. records the measured wall-clock query latency.
- **An honest report artifact** (`DELIVER.md` or appended to `EVAL.md`): the
  asserted top-1-correct result, the measured latency, the sidecar/model version,
  and an explicit statement that gallery and query individuals are disjoint
  photos (held-out), with the SOURCES of the fixtures cited.
- **Graceful skip, never a false green.** If the model cannot be provisioned in
  the build environment (no network + no cached weights), the attestation
  **skips with a loud, logged reason** and a non-success-but-non-failing marker —
  it must never silently pass without having actually embedded and matched. A
  skipped attestation is reported as skipped, not as proof.

## Acceptance criteria

1. The attestation runs the **real** embed sidecar (not a stub/mock) end to end:
   enroll gallery → submit query → ranked shortlist, against the bundled
   `eval-smoke` fixtures.
2. The top-ranked candidate for the `cat1_photo2.png` query is `cat1`, and both
   dog gallery entries rank strictly below it — asserted, with the assertion
   failing loudly if the order is wrong.
3. A measured query latency (one forward pass + fusion) is recorded and written
   to the report artifact, with the model/sidecar version alongside it.
4. The report artifact states explicitly that gallery and query are disjoint
   held-out photos and cites the fixture SOURCES — the tautology guardrail is
   documented, not just assumed.
5. When the model cannot be provisioned (offline + no cached weights), the
   attestation **skips with a clear logged reason** and reports SKIPPED — it does
   not pass, does not crash, and does not emit a fabricated success.
6. The attestation is reproducible from a documented single entrypoint
   (`homeward attest deliver` or the named script), and the workspace
   `cargo build`/`cargo test` and `cargo clippy` remain clean.
