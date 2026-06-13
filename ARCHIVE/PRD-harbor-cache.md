# PRD: harbor-cache — one warm compile cache the whole fleet shares

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/constellation-burst-builder
Vision: visions/harbor.md

## TL;DR

Every burst pod runs its own sccache and throws it away on destroy — so each cold
burst recompiles the world. This PRD stands up a **shared sccache backend on the
permanent hub** (a small MinIO/S3-compatible store, the simplest path that gets
~90% of the warm-cache win) and emits client config so burst pods *and* the laptop
point their `SCCACHE_*` at the hub. The first cold compile of the day hits a
populated cache instead of an empty one.

## Why this exists

- `config.rs` already has an `SccacheConfig { endpoint, bucket, access_key,
  secret_key }` — the *intent* of a shared cache is in the schema, but nothing
  stands up the backend; in practice each pod's sccache is local and dies on
  `destroy_pod`.
- The vision's open question lands on **shared-cache-first** (MinIO/S3 on the hub,
  `SCCACHE_ENDPOINT` pointed at it) over full `sccache-dist` distributed
  compilation — far simpler, almost all the benefit. This PRD takes that path.
- ccx53 cold bursts (the last two logged runs) recompile from scratch every time;
  a warm shared cache is the single biggest wall-clock win for the burst pattern.

## What this builds

**`scripts/harbor-cache-up.sh`** — run against the hub (ssh target from
`hub.json`, written by harbor-hub):
- Install + start MinIO (single-node, single-disk) as a systemd unit on the hub,
  bound to the mesh/private interface (not public) where reachable; create an
  `sccache` bucket; generate an access/secret keypair.
- Idempotent: re-running detects a live MinIO + bucket and only reconciles.
- Print the `SCCACHE_ENDPOINT`, `SCCACHE_BUCKET`, and keypair as shell-exportable
  lines.

**`scripts/harbor-cache-client-env.sh`** — emits the `SCCACHE_*` env block clients
source: `SCCACHE_ENDPOINT`, `SCCACHE_BUCKET`, `SCCACHE_S3_USE_SSL`,
`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `RUSTC_WRAPPER=sccache`. Reads the
hub endpoint from `hub.json`; writes nothing secret to a tracked file (keys live in
`~/.config/wm-burst/cache.env`, gitignored).

**`scripts/harbor-cache-check.sh`** — offline structural gate: asserts the up
script is idempotent (mock/dry-run mode), the client env block contains all
required vars, and the bucket name matches `config.rs`'s `SccacheConfig.bucket`
default.

## Acceptance criteria

1. `harbor-cache-up.sh --dry-run` (no live hub) prints the planned MinIO unit +
   bucket + keypair-gen steps and exits 0 without touching anything.
2. `harbor-cache-up.sh` is idempotent: a second `--dry-run` against a
   "cache-already-up" fixture reports "reconcile only, no create" (assert via a
   marker the script checks).
3. `harbor-cache-client-env.sh` emits all of: `SCCACHE_ENDPOINT`, `SCCACHE_BUCKET`,
   `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `RUSTC_WRAPPER=sccache` — verified
   by `harbor-cache-check.sh` grepping the output.
4. The endpoint + bucket are read from `hub.json` (harbor-hub's persisted state),
   not hardcoded; with no `hub.json`, the client-env script exits non-zero with
   "no hub — run `wm-burst hub up` first".
5. Secrets (keypair) are written only to `~/.config/wm-burst/cache.env`; a grep of
   any tracked script/file in the repo finds no access/secret key value.
6. `harbor-cache-check.sh` passes as the PRD's offline acceptance gate and is wired
   so `cargo test` or a `scripts/` runner can invoke it.
7. README/REMOTE-SETUP gains a "shared cache" section documenting the up + client
   flow and the ARM-vs-x86 hub caveat from the vision's open questions.
