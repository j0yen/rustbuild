# PRD: constellation-waketrain-offload — the wake-word retrain runs in the cloud, validated before it lands

Status: Draft v0.1
build_target: shell
Vision: visions/constellation.md
Depends: PRD-constellation-burst-builder.md (uses `wm-burst exec` / `wm-burst pod`
  as the transport + cost/teardown machinery — this is the first-class recipe on top)
Refines: PRD-wintermute-wake-word (the train-wintermute.sh pipeline) — adds a
  remote-execution + safe-install wrapper, changes none of its stages

## TL;DR

`burst-train.sh` is the wake-word-specific recipe that runs the heavy
`train-wintermute.sh` job on a cloud box instead of this laptop, then **validates
the returned ONNX against its I/O contract and a local smoke pass before it is ever
installed** into wm-audio. It exists because the full retrain OOM-kills here (11.2 GB
peak on a no-swap box) and takes hours on CPU — exactly the job that should burst.
It pushes the inputs (positives, voice/libritts models, the pinned py3.11 env spec),
runs `train-wintermute.sh` on an **on-demand GPU pod** (the default — training is
rare and GPU collapses the multi-hour CPU run to minutes), pulls back the model +
receipts, and refuses to swap the live model in unless the new one is shaped
`[1,186,40]→[1,1]` and passes the `verify` stage locally. A failed or mis-shaped run
leaves the working model untouched. Because training happens only a handful of times,
there is **no standing training infra** — the pod exists only for the minutes a run
takes, then is torn down. (Distinct from the always-on Oracle build box, which does
*builds*, not training.)

## Why this exists

Two recorded facts make this the highest-value single offload:

- The real-voice retrain **OOM-killed** on 2026-06-03 (`wake-retrain-realvoice.service`,
  11.2 GB peak, no swap headroom — memory `self_recall_baseline_gate_red`), and the
  full run is "hours on CPU" per the script's own header. It is the canonical
  burstable job.
- The wake model has a **strict, easy-to-break I/O contract**:
  `[1,186,40]` mel → `[1,1]`, and only the non-streaming variant converts to ONNX
  (memory `project_wintermute_wake_training`). A retrain that silently produces the
  wrong shape, or the streaming variant, would break wm-audio's `ort` inference
  path. So "run it elsewhere" is necessary but not sufficient — the result must be
  **proven** before it replaces a working model. The lesson on record
  (`feedback_verify_before_concluding`): a one-line wake bug already cost three
  retrains + 120 user recordings by being assumed-good instead of verified at the
  failing step. This PRD makes verification a gate, not an afterthought.

`burst-builder` gives the generic `wm-burst exec` + pod/cost/teardown plumbing; this
PRD is the wake-train-shaped recipe with the dataset-sync and the install-gate that
generic exec can't know about.

## What this builds

A recipe `~/wintermute/wake-train/burst-train.sh` (+ a thin
`wake-retrain-burst.service` drop-in that calls it instead of running locally):

- **Input sync** — pushes only what the remote needs: the positives/recordings, the
  piper voice model (`en_US-lessac-medium.onnx`) + libritts `.pt`, and a **pinned
  environment spec** (py3.11, TF 2.21, microwakeword, torch — exact versions) so the
  remote venv is byte-for-byte the training env, not a drifted one. Large static
  assets are content-addressed/cached so re-runs don't re-upload them.
- **GPU pod run (default)** — runs `train-wintermute.sh` (optionally `--smoke` first,
  on a tiny/cheap shape, for a fast end-to-end check) on an **on-demand GPU pod**
  (TF/torch CUDA) so the multi-hour CPU run collapses to minutes; the pod also has
  ample RAM so the 11.2 GB peak can't OOM. A `--cpu` fallback shape (≥ 32 GB RAM, no
  GPU) is available for environments without a GPU provider, but GPU is the default
  because training is infrequent and the per-run GPU cost is trivial. Streams stage
  logs locally; propagates the real exit code.
- **Artifact pull + receipts** — retrieves the exported ONNX and the per-stage
  receipts/metrics (train/val accuracy, the `verify` stage output) back to
  `wintermute-train/out/`.
- **Install gate (the point of the PRD)** — before touching the live model it
  asserts: (a) ONNX inputs/outputs are exactly `[1,186,40] → [1,1]`; (b) it is the
  non-streaming variant; (c) the local `verify` stage passes on the returned model;
  (d) the new model's held-out accuracy is reported. Only on all-pass does it
  atomically swap the model into wm-audio's model path (with a backup of the prior
  one for one-command rollback). Any failure leaves the working model in place and
  exits non-zero with the reason.
- **Cost + teardown via wm-burst** — the pod lifecycle, cost estimate, and
  idle-timeout teardown are delegated to `wm-burst pod`, so a training pod is never
  left running and the spend is logged against the monthly cap.

Non-goals: changing the training pipeline/stages (PRD-wintermute-wake-word owns
`train-wintermute.sh`); the generic burst transport (burst-builder owns it); model
architecture or the recording UX.

## Acceptance criteria

1. `burst-train.sh --smoke` runs the full deps→…→verify pipeline on the remote and
   returns a smoke ONNX locally — a cheap end-to-end proof that the offload path
   works before paying for a full run (demonstrated or reproducibly documented).
2. Input sync uploads the positives + voice/libritts models + a **pinned** py3.11/
   TF2.21/microwakeword/torch env spec, and a re-run does **not** re-upload unchanged
   large assets (content-addressed/cached) — proven by a second run showing the
   cache skip.
3. The full run executes on an **on-demand GPU (CUDA) pod by default** (with a
   `--cpu` fallback shape of ≥ 32 GB RAM for GPU-less environments); stage logs stream
   locally and the remote exit code is faithfully propagated (a forced remote failure
   exits the wrapper non-zero with the failing stage named).
4. **Install gate:** a returned model is installed into wm-audio **only if** its ONNX
   I/O is exactly `[1,186,40] → [1,1]`, it is the non-streaming variant, and the
   local `verify` stage passes — proven by a test that feeds a deliberately
   wrong-shaped/streaming ONNX and asserts the live model is **not** replaced and the
   exit is non-zero.
5. A successful install is **atomic** and backs up the prior model; `burst-train.sh
   --rollback` restores it in one command.
6. Pod lifecycle (create/run/teardown) + cost estimate is delegated to `wm-burst pod`
   and logged against the monthly cap; no training pod is left running after
   completion/idle-timeout (provable with a mocked provider — no real spend to pass).
7. The `wake-retrain-burst.service` drop-in invokes `burst-train.sh` (not a local
   train) so the systemd retrain path no longer risks OOM-killing this laptop; the
   old local unit is documented as superseded.
8. Script hygiene: `set -euo pipefail`, `--help`, clear errors; idempotent re-runs;
   no secrets written to logs.

## Resolved decisions

- **GPU pod is the default** (jsy, 2026-06-05): training is rare and per-run GPU cost
  is trivial, so default the full run to an on-demand CUDA pod (RunPod/Vast),
  torn down after; `--cpu` is a fallback, `--smoke` runs on a tiny/cheap shape. No
  standing training infra — the always-on Oracle box does *builds*, not training.

## Open questions (for /build or jsy)

- Where the pinned env spec lives so local and remote can't drift — a lockfile
  (`requirements.txt` + exact wheels, **arm64-aware if a build host is ever reused**)
  checked into `wake-train/`, regenerated when the local venv changes. (Note: the GPU
  pod is x86+CUDA, so the training env is x86 wheels — independent of the aarch64
  build box.)
- Held-out accuracy threshold for the install gate: block install if val accuracy
  regresses vs the currently-installed model? (Guards against a bad retrain landing.)
