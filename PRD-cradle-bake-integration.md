# PRD-cradle-bake-integration

**Status:** Shipped (cradle v0.2.0, main bca7b9f)
**Vision:** (cradle — self-trained models baked into Rust)
**build_target:** rust-extend
**build_into:** ~/wintermute/cradle (j0yen/cradle, v0.1.0 → v0.2.0)

## TL;DR

`cradle` v0.1 harvests labeled examples from Claude transcripts but stops
there: `train.py` is a stub that writes `test_accuracy: null`, and `cradle
bake` emits a placeholder crate with no weights. Today's harvest of the
`redirect` model yields 194 balanced examples of 8 features. v0.2 closes the
loop for that one model: a real trainer, a bake step that writes the trained
weights into Rust as `const` arrays over `morsel` primitives, and a
`classify` command that runs the baked model — with a parity test proving the
Rust output matches the Python output on the held-out set.

## Why this exists

The point of cradle is a classifier a daemon can call without Python, a model
file, or a network round-trip. Harvest alone produces a dataset nobody
consumes. The missing three stages are small: the feature vector is 8 floats,
the dataset fits in memory, and `morsel` already provides `linear`, `sigmoid`,
and `argmax` as allocation-free functions with weights as `const` arrays. What
`morsel` lacks is any bake tool — it is inference-only by design — so the
code generation belongs in cradle.

The first consumer is `redirect`: does a user turn redirect the assistant
("wait", "no", "actually", followed by a change of behavior)? A baked
`redirect` model is the seed for a learning-candidate gate that today relies
on keyword matching.

## What this builds

Extends the `cradle` workspace (MSRV 1.85, no let-chains, `sigpipe::reset()`
first line of `main()`):

- **`models/redirect/train.py` + `models/redirect/pyproject.toml`** — a real
  trainer in numpy only (no torch): a 2-layer MLP 8→8→1 with tanh hidden and
  sigmoid output, full-batch gradient descent, fixed seed. Reads
  `$CRADLE_DATA_DIR/{train,val,test}.jsonl`, early-stops on val loss, writes
  `$CRADLE_OUTPUT_DIR/checkpoint.json`:
  ```json
  {"schema":"cradle.checkpoint.v1","model":"redirect","input_dim":8,
   "layers":[{"w":[[...8x8]],"b":[...8],"act":"tanh"},
             {"w":[[...8x1]],"b":[...1],"act":"sigmoid"}],
   "threshold":0.5}
  ```
  and `metrics.json` (`cradle.metrics.v1`) with real `test_accuracy`,
  `test_auc`, `n_train`, `n_val`, `n_test`, `epochs`. Also writes
  `predictions.jsonl` for the test split: `{"source_session","source_turn",
  "p"}` — the reference for the parity test.
- **`cradle train <model>`** — unchanged shellout (`uv run python train.py`),
  plus: after the run, reads `metrics.json` and compares to
  `threshold`/`auc_threshold` in `spec.toml`; prints PASS/FAIL per metric and
  exits 3 on FAIL unless `--allow-below-threshold`.
- **`cradle bake <model>`** — reads `checkpoint.json` (JSON, replacing the
  never-implemented safetensors path) and writes a library crate at
  `output/morsel-<model>/` with:
  - `Cargo.toml` depending on `morsel = { git = "https://github.com/j0yen/morsel" }`
  - `src/lib.rs` containing `pub const INPUT_DIM: usize`, one `const` array
    per weight and bias, `pub fn score(x: &[f32; INPUT_DIM]) -> f32` (the
    sigmoid probability) and `pub fn predict(x: &[f32; INPUT_DIM]) -> bool`
    (`score >= THRESHOLD`), implemented with `morsel::linear` and
    `morsel::activation::{tanh,sigmoid}` — no allocation, no `Vec`.
  - a generated unit test asserting `score` on the first 5 rows of
    `predictions.jsonl` matches `p` within 1e-5.
  Output is deterministic: same checkpoint → byte-identical `lib.rs` (floats
  formatted with `{:?}`, no timestamps).
- **`cradle classify <model> --features f1,…,f8 | --turn-pair <json>`** —
  compiles nothing at runtime: the workspace gains a `crates/cradle-baked`
  member whose `build.rs` includes `output/morsel-<model>/src/lib.rs` when
  present (feature-gated per model, `--features redirect`), so the binary
  built after `bake` runs the model natively. `--turn-pair` runs
  `cradle-features::turn_pair_v1` first. Prints `{"model","p","label"}`.
- **`cradle eval <model>`** — parity check: runs the baked Rust model over
  every row of `test.jsonl`, compares to `predictions.jsonl`, prints max
  absolute error and agreement rate, exits 0 only if max error < 1e-4 and
  agreement is 100%.
- **`cradle build <model>`** — now runs harvest → train → bake → eval;
  the v0.1 "bake skipped" branch is removed.

Non-goals: the `playbook-match` and `session-productivity` label extractors
(their `spec.toml` shells stay); wiring a consumer daemon; any model larger
than what fits in a few hundred floats; GPU or torch.

## Acceptance criteria

1. Given the harvested `models/redirect/data/`, when `cradle train redirect`
   runs, then `checkpoint.json`, `metrics.json` and `predictions.jsonl` exist,
   `metrics.json` has numeric `test_accuracy` and `test_auc`, and the run is
   reproducible: two runs produce identical `checkpoint.json`.
2. Given `spec.toml` thresholds (0.85 / 0.85), when `metrics.json` is below
   either, then `cradle train` exits 3 and prints which metric failed; with
   `--allow-below-threshold` it exits 0 — verified with a fixture
   `metrics.json` on each side of the threshold.
3. Given a `checkpoint.json` fixture (8→8→1), when `cradle bake redirect`
   runs, then `output/morsel-redirect/src/lib.rs` compiles as part of the
   workspace test build, exposes `score`/`predict`/`INPUT_DIM`, contains no
   `Vec`/`Box`/`alloc`, and a second bake yields byte-identical output.
4. Given the baked crate, when its generated unit test runs, then `score` on
   the fixture's first 5 prediction rows matches `p` within 1e-5.
5. Given the baked crate, when `cradle classify redirect --features <8
   floats>` runs, then it prints one JSON line with `p` in [0,1] and boolean
   `label`; `--turn-pair '{"user_turn":"wait, no — go back","prev_assistant":"…"}'`
   featurizes and classifies without error.
6. Given `test.jsonl` and `predictions.jsonl`, when `cradle eval redirect`
   runs, then max absolute error < 1e-4 and agreement 100%, exit 0; a
   deliberately perturbed fixture checkpoint makes it exit non-zero.
7. `cradle build redirect` runs all four stages end-to-end from the real
   transcripts on this machine and leaves a `classify`-able binary; the "bake
   skipped in v0.1" message is gone from `--help` and code.
8. `cargo test --release` green across the workspace; version 0.2.0;
   `sigpipe::reset()` present; README's stage table updated to "shipped" for
   train and bake, with the checkpoint schema documented.
