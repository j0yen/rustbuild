# PRD: homeward-embed-provision — the model exists, the index serves, proven once

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

The embed sidecar calls `AutoModel.from_pretrained("facebook/dinov2-base")` —
which downloads the weights silently on first request, from inside whatever
process happens to call `/enroll` first, with no offline guarantee, no warmup,
and no proof it has ever embedded a real photo end-to-end. This is the exact
shape of the bug that bit voice (`wm-stt` shipped with `model: distil-small.en`
in config but the bytes weren't there — see [[project_voice_input_null_detectors]]).
This PRD provisions the model deterministically and proves the
enroll→index→query path on a real image with a measured latency.

## Why this exists

Phase-1 live inspection (2026-06-13, `homeward/embed/homeward_embed/`):

- `embedder.py:74-75` loads the model lazily with `from_pretrained` and a
  network round-trip; there is **no prefetch step, no pinned cache dir, no
  offline mode**. First `/enroll` after a cold boot blocks on a multi-hundred-MB
  download, or fails entirely if the machine is offline — invisible until a real
  query arrives.
- The crate-level smoke tests (`embed/tests/test_embedder.py`, `test_index.py`)
  exercise the API surface but **do not prove the real DINOv2 weights load and
  produce a discriminative vector** on an actual photo — they are unit tests, not
  an end-to-end provisioning proof.
- The vision's hardware claim ("feasible on modest hardware … query latency
  dominated by one forward pass <1s") has **never been measured on this box**.
  `facebook/dinov2-small` (384-d, `embedder.py:28`) is the CPU-friendly variant
  and the right default for this CPU-only laptop.

## What this builds

- **A prefetch/warmup command** in `homeward_embed`: `homeward-embed warmup`
  (and a `service.py` startup hook) that loads the configured DINOv2 variant
  into a **pinned, committed-to-config cache dir** (`HF_HOME`/`HOMEWARD_MODEL_DIR`)
  before serving, and supports `HF_HUB_OFFLINE=1` so a provisioned box never hits
  the network at query time.
- **A real end-to-end smoke** (`homeward-embed smoke`): takes a bundled, CC-
  licensed sample pet image, runs it through the live sidecar
  (`/enroll` then `/query`), and asserts (a) the query returns the enrolled image
  as rank-1, (b) a *different* animal scores lower, and (c) wall-clock query
  latency is recorded and printed. This proves the model is real and the index
  discriminates — not a mock.
- **A small committed fixture set** (`embed/fixtures/`): a handful of permissively-
  licensed (Wikimedia CC / public-domain) dog and cat photos, enough for the smoke
  to assert rank-ordering. Provenance + license recorded in `embed/fixtures/SOURCES.md`.
- **Variant default for CPU**: smoke and warmup default to `dinov2-small`;
  `HOMEWARD_EMBED_VARIANT` overrides. The vision's `<1s` claim is checked against
  the measured number, and the README records the real figure observed on this box.

### Non-goals

- No accuracy claim on real shelter data — that's `homeward-eval-harness`.
- No GPU path (constellation territory).
- No change to the embedding model itself; this provisions and proves what ships.

## Acceptance criteria

1. `homeward-embed warmup` loads the configured DINOv2 variant into the pinned
   cache dir and exits 0; a second run with `HF_HUB_OFFLINE=1` set also exits 0
   (proving no network dependency after provisioning).
2. `homeward-embed smoke` starts (or targets a running) sidecar, enrolls a bundled
   fixture image, queries with the same image, and asserts it returns as rank-1.
3. The smoke asserts a *different* fixture animal (different species or individual)
   scores strictly below the self-match, proving the vector is discriminative and
   not constant.
4. The smoke records and prints measured query wall-clock latency; the README
   records the real figure observed on this laptop (no asserted-but-unmeasured
   `<1s`).
5. Every bundled fixture image has its source URL and license recorded in
   `embed/fixtures/SOURCES.md`; no image without a documented permissive license
   is committed.
6. `uv run pytest` for the embed subtree passes including the new smoke (guarded
   to skip with a clear message if the model cannot be fetched in a sandbox
   without network, so CI stays honest rather than silently green).
