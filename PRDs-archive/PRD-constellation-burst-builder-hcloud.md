# PRD: constellation-burst-builder-hcloud — a real Hetzner Cloud x86 pod, bursted per build

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/constellation-burst-builder
deferred_acs: [3, 5, 6]
Vision: visions/constellation.md
Extends: PRD-constellation-burst-builder (shipped as `wm-burst` v0.1.0)
Supersedes: the aarch64 direction (PRD-constellation-burst-builder-arch, withdrawn
  2026-06-05 — laptop is x86_64, so an ARM box can't produce runnable binaries or
  share the sccache cache). If that PRD's in-flight build landed a `remote_arch`
  config field, this PRD pins its default to `x86_64-unknown-linux-gnu` and does not
  build out the ARM provision/doctor paths.
Ordering: build only AFTER the current `autobuilder/constellation-burst-builder`
  branch has settled (an in-flight tick was editing the same crate/branch) to avoid a
  worktree collision.

## TL;DR

`wm-burst` v0.1 ships a pod abstraction whose **real provider is a stub** —
`make_provider` warns "real provider not yet implemented in v0.1" and falls back to
mock. This PRD implements the **one real provider that matters for build-only on an
x86 laptop: Hetzner Cloud** (`hcloud` API). It boots an **ephemeral x86 AMD server
(CCX23 / CPX41) per build session** from a pre-provisioned snapshot, runs the build
against a persistent shared sccache cache, and destroys the server on completion —
**~€0.03/hr, pennies per build, torn down after**. Because the pod is x86 it matches
the laptop: the cache is shared and any produced binary runs locally. This is the
build-only path jsy chose (2026-06-05) over a flat €80/mo dedicated box and over the
free-but-ARM Oracle box.

## Why this exists

jsy, 2026-06-05: priced the Hetzner dedicated auction ("too high, even a ryzen 7 is
80 euros"), then — on confirming the laptop is **x86_64** and that an aarch64 box
can neither produce runnable-on-laptop binaries nor share an x86 sccache cache —
chose **Hetzner Cloud x86, on-demand**. The shipped `wm-burst` already has the
generic pod lifecycle (create/run/destroy + cost log + budget cap, all proven
against a `MockPodProvider`) and an Ubuntu/Debian `apt` provision path. What's
missing is a **real** provider behind that abstraction. Hetzner Cloud is the right
first real provider:

- Hourly billing with a monthly cap; a CCX23 (4 dedicated vCPU, 16 GB) is ~€0.03/hr,
  so a bursty build workload costs ~€1–2/mo vs a flat €80.
- x86 AMD — matches the laptop, so the offload produces usable artifacts and the
  shared cache actually hits (the whole point an ARM box failed on).
- Simple, well-documented API (create server, attach SSH key + cloud-init, delete);
  fits the existing `PodProvider` trait (`create_pod` / `run_job` / `destroy_pod`).

## What this builds (extends `wm-burst`)

- **`HcloudPodProvider`** implementing the existing `PodProvider` trait:
  - `create_pod` — calls the hcloud API to create a server of the configured type
    (`server_type`, default `ccx23`) in the configured `location`, from a configured
    **snapshot/image** (the pre-provisioned builder image: toolchain + sccache
    baked in via `wm-burst provision` → snapshot), injecting the SSH key. Returns
    the server id + IP.
  - `run_job` — SSHes the build/command onto the booted server with
    `RUSTC_WRAPPER=sccache` pointed at the persistent shared cache; streams output;
    returns the real exit code.
  - `destroy_pod` — deletes the server (idempotent; safe to call on a partial/failed
    create so a half-booted server is never orphaned).
- **Config** — a `[pod]` section gains `provider = "hcloud"`, `server_type`,
  `location`, `image` (snapshot id/name), and the API token sourced from
  `$HCLOUD_TOKEN` (never written to the config file or logs). `remote_arch` (if
  present from the withdrawn arch PRD) defaults to `x86_64-unknown-linux-gnu`.
- **Persistent shared sccache backend** — because the build pod is fully ephemeral
  (no standing box), the cache must live off-pod for cross-run hits. Default to
  **Hetzner Object Storage** (S3-compatible) for the sccache bucket; `wm-burst init`
  scaffolds the endpoint/bucket and documents it. (This supersedes the earlier
  MinIO-on-a-standing-box default, which assumed an always-on host that no longer
  exists in the build-only plan.)
- **`wm-burst build --burst`** — the on-demand flow: create an hcloud pod, run the
  build there against the shared cache, report where it ran + cache hit/miss + cost,
  and tear the pod down. Reuses the shipped cost log + monthly-budget guardrail.
- **Snapshot helper** — `wm-burst provision --snapshot` provisions a fresh server,
  bakes the toolchain + sccache, and creates an hcloud snapshot so subsequent
  `--burst` runs boot from it in ~30s instead of re-provisioning each time.

Non-goals: GPU pods / training (waketrain-offload owns those — that's x86+CUDA on
RunPod/Vast, a different provider); the generic pod lifecycle, cost log, and budget
cap (already shipped); aarch64/ARM support (withdrawn — x86 only).

## Acceptance criteria

1. `HcloudPodProvider` implements `PodProvider` (`create_pod`/`run_job`/`destroy_pod`)
   and is selected when `[pod] provider = "hcloud"`; the API token is read from
   `$HCLOUD_TOKEN` and never appears in config or logs (asserted by a test scanning
   the cost log + rendered config for the token).
2. `destroy_pod` is idempotent and tears down even a partially-created server — proven
   by a test where `create_pod` half-fails and the server is still deleted (no orphan).
3. `wm-burst build --burst` creates an hcloud pod, runs a `~/wintermute` crate build
   against the persistent shared cache, reports where it ran + cache hit/miss + cost,
   and destroys the pod afterward — demonstrated against real hcloud **or**
   reproducibly documented, with the full path unit-tested via the mock provider.
4. Config carries `[pod] provider/server_type/location/image` with sane defaults
   (`hcloud` / `ccx23` / a documented EU location / a snapshot name); `wm-burst init`
   round-trips them and `remote_arch` defaults to `x86_64-unknown-linux-gnu`.
5. The default sccache backend scaffolded by `init` is Hetzner Object Storage
   (S3-compatible), documented; a second build of the same crate shows a higher
   cache-hit ratio (warm persistent cache) — demonstrated or reproducibly documented.
6. `wm-burst provision --snapshot` produces a reusable builder snapshot and a
   subsequent `--burst` boots from it (documented end-to-end; the API calls are
   unit-tested against the mock).
7. The monthly budget cap still blocks new pods when exceeded (no regression to the
   shipped guardrail); every pod lifecycle + cost estimate is logged.
8. No regression: `cargo test` green, `sigpipe::reset()` first line of `main()`,
   MSRV 1.85, no let-chains; the real-provider "not implemented" warning is gone for
   `hcloud`.

### Deferred ACs (live-Hetzner-spend, mock + documentation paired)

ACs **3, 5, 6** each take the "demonstrated against real hcloud **or**
reproducibly documented, with the full path unit-tested via the mock
provider" arm. Live demonstration requires a real `$HCLOUD_TOKEN` and
incurs actual Hetzner spend, so they are **deferred from a live run** and
satisfied by:

- **AC3** (`build --burst` full path): in-process mock provider exercises
  create → run_job → cost-log → destroy (`tests/acceptance_ac6.rs`,
  `provider::tests::mock_provider_run_job_succeeds`); real burst documented
  in `REMOTE-SETUP.md`.
- **AC5** (Hetzner Object Storage sccache, warm-cache hit ratio):
  `tests/mocks/ac4.rs::warm_cache_has_higher_hit_ratio`; bucket + endpoint
  setup documented in `REMOTE-SETUP.md` §4.
- **AC6** (`provision --snapshot` reusable builder): provisioning playbook
  generation unit-tested in `tests/mocks/ac2.rs`; manual provision path
  documented in `REMOTE-SETUP.md` §3.

The remaining ACs (1, 2, 4, 7, 8) are paired to real, offline unit/
integration tests that run green under `cargo test --release`.

## Open questions (for /build or jsy)

- Server type default: `ccx23` (4 dedicated vCPU) vs `cpx41` (8 shared vCPU, cheaper
  per build but variable). Default `ccx23` for predictable build times.
- Whether to keep one warm pool server alive during an active dev session (faster
  successive builds) vs strict per-build create/destroy (cheapest). Default strict;
  revisit if boot latency annoys.
