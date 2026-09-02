---
name: cloudbuild
description: Build and test Rust on a cheap on-demand Hetzner Cloud x86 box instead of this laptop, then tear it down so billing stops. Boots from a pre-provisioned snapshot (~30s, rustc 1.85+1.88 + sccache), rsyncs the crate, runs the build/test remotely with a shared sccache cache, pulls artifacts back, and destroys the server. Use when the user says /cloudbuild, asks to "build in the cloud", "burst this build", "run the build on the cloud box", or wants the cloud-burst builder brought up/down. This is the preferred build path for the user's work — it replaces local /autobuilder cargo builds for heavy/cold compiles. Exception: on RedBaron (hostname RedBaron) cargo runs locally by default — use this skill there only on explicit request or for fleet fan-out.
user_invocable: true
---

# /cloudbuild — burst Rust builds to a Hetzner Cloud x86 box

This skill offloads heavy Rust compiles and `cargo test` from this CPU-only laptop
to a cheap, **on-demand** Hetzner Cloud server that **matches the laptop's arch**
(x86_64, rustc 1.85.0) — so the offload produces runnable-on-laptop binaries and
shares an sccache cache. The box boots from a pre-provisioned **snapshot** in ~30s,
does the work, and is **destroyed afterward** so billing stops (~€0.12/hr while up).

It is the cloud counterpart to `/autobuilder`: same goal (build/validate Rust), but
the expensive compilation runs in the cloud instead of pinning local cores (which on
this box are needed for the voice stack + local LLM). For the user's work on
carbon/ryzen7, prefer `/cloudbuild` over local building. **On RedBaron, build
locally** — it is the designated Rust build machine (2026-09-01): 16 threads,
30 GB, sccache + mold; a clean `recall` release build there beats a ccx53 burst.

**Current builder:** `ccx53` (32 vCPU, 128 GB RAM) at `nbg1`. This fills the account's
32-core dedicated limit exactly. `ccx63` (48 vCPU) would exceed it.

**Two snapshots — pick by deploy target** (as of 2026-08-31):
- `SNAPSHOT_ID=427125061` — **default** (rebuilt 2026-09-01; 426737971 was snapshotted unclean and had an unreadable libLLVM), Ubuntu 26.04/resolute (glibc 2.43,
  libfuse3.so.4, rustc 1.85+1.88, sccache via apt), cpx32-derived (160GB disk
  footprint, deploys on cpx32 and larger). Matches **carbon** — use for
  anything installed to this laptop.
- `SNAPSHOT_ID_NOBLE=394184500` — Ubuntu 24.04/noble (glibc 2.39), the old
  ccx-derived 240GB snapshot. Matches the **constellation hub** — use for
  hub-targeted builds (homeward daemons) by exporting
  `SNAPSHOT_ID="$SNAPSHOT_ID_NOBLE"` before invoking cloudbuild.sh.
- Why both: a binary linking versioned system sonames (fuse, system rocksdb,
  non-vendored openssl) only runs where the soname matches; resolute-built
  provfs needs .so.4, noble-built needs .so.3. Building on the wrong snapshot
  fails at local verify, not at compile.

## Warm hub = RedBaron (2026-09-01)

Carbon and ryzen7 route **every** `cloudbuild build/test` to RedBaron over
Tailscale SSH instead of Hetzner: `~/.config/wm-burst/hub.json` on each client
is `{"ip":"100.73.175.108","user":"jsy","build_root":"/home/jsy/build",
"sccache_dir":"/home/jsy/.cache/sccache","prefer":"always"}`. `prefer=always`
sends cold and incremental builds alike to the hub whenever port 22 answers;
`prefer=incremental` (or absent) restores the old heuristic. The hub build runs
as `jsy` (no root), syncs into `~/build/<crate>`, uses RedBaron's sccache +
mold, and rsyncs `target/` back — the binary runs on the client because all
three machines are Ubuntu 26.04 x86_64. `.env` (Hetzner token) is optional on a
hub-only client; burst commands then fail with a clear message. If RedBaron is
down, routing falls back to burst automatically (needs `.env`). Measured from
carbon: `cradle --release` in 15 s build / 17 s end-to-end.

## The workhorse

Everything is driven by `cloudbuild.sh` (next to this file). It reads config + the
API token from `~/.config/wm-burst/.env` (`HCLOUD_TOKEN`, `SNAPSHOT_ID`,
`BUILDER_TYPE`, `BUILDER_LOC`, `SSH_KEY_NAME`, `SSH_KEY`).

```sh
SK=~/.claude/skills/cloudbuild/cloudbuild.sh
bash "$SK" build <crate> [-- <cargo args>]   # one-shot: up → build → pull → DOWN
bash "$SK" test  <crate> [-- <cargo args>]   # one-shot: up → cargo test → DOWN
bash "$SK" fleet [--type cpx41] [--max N] <crate>... [-- <cargo args>]
                                             # PARALLEL: one box per crate, concurrent, each torn down
bash "$SK" session-start [ttl_hours]         # warm a server for a build batch; build/test reuse it
bash "$SK" session-end                       # tear down the session server (STOPS BILLING)
bash "$SK" up                                # bring the box up, leave it running
bash "$SK" keep-build <crate> [-- ...]       # build but keep the box up (reuse)
bash "$SK" status                            # is a box up? session warm? rate? recent lifecycle
bash "$SK" doctor                            # verify arch/toolchain on the box
bash "$SK" ssh [cmd]                         # ssh into the running box
bash "$SK" down                              # destroy the box (STOPS BILLING)
```

### Session mode — use for build batches

When building multiple crates in sequence (e.g. a `/build` tick), the per-build
create→destroy cycle wastes money: Hetzner bills per **started hour**, so a 2-min
build costs the same as a 60-min build. A 300-session day at €0.48/hr = €144.

Use session mode instead:

```sh
bash "$SK" session-start          # up once; lock written to ~/.config/wm-burst/session.lock
bash "$SK" build crate-a          # reuses warm server — no create/destroy
bash "$SK" build crate-b          # same server
bash "$SK" build crate-c          # same server
bash "$SK" session-end            # one destroy — billing stops
# 300 builds → 1 billed hour instead of 300
```

The default session TTL is 4 hours. The watchdog (`cloudbuild-watchdog.sh`) kills
any builder older than 2h as a safety net if `session-end` is never called.
`<crate>` may be an absolute path or a bare name resolved under `~/wintermute/`.

## How to run this skill

1. **Confirm intent + cost.** A `build`/`test` brings up a server that bills while
   alive. One-shot commands auto-destroy on exit (EXIT trap), so a single build costs
   one billed hour regardless of actual duration. For multiple sequential builds,
   **always use `session-start` / `session-end`** to amortize that hour across all
   builds in the batch. If the user asked to *keep* the box (`up`/`keep-build`),
   remind them it bills until `down`.
2. **Run the command.** For a normal "build X in the cloud" request:
   `bash "$SK" build <crate> -- --release`. Stream the result back to the user —
   report where it ran, the exit code, sccache hit rate, and pulled artifacts.
3. **Verify teardown.** After a one-shot, run `bash "$SK" status` and confirm
   "builder: none running". **Never leave a server billing silently** — if a one-shot
   was interrupted, run `bash "$SK" down` explicitly and confirm.
4. **On a fresh snapshot need.** If `SNAPSHOT_ID` is empty/invalid (e.g. the snapshot
   was deleted), the box can't boot pre-provisioned. Recreate it: provision a plain
   Ubuntu box (see `~/wintermute/constellation-burst-builder/REMOTE-SETUP.md`),
   snapshot it, and write the new id into `~/.config/wm-burst/.env`.

## Parallel fleet — how many at a time

`fleet` runs one ephemeral box **per crate**, concurrently, each torn down on
completion (a global EXIT trap reaps any box labelled `fleet=1` as a backstop).
The ceiling on concurrency is **your Hetzner account's limits**, not the script:

**Dedicated (ccx) fleet — now viable (limit raised to 32 cores, 2026-06-11):**
- `ccx33` (8 vCPU, €0.12/hr): up to **4 parallel** boxes (4×8 = 32 cores). Use
  `--type ccx33 --max 4` for small-crate parallelism with dedicated cores.
- `ccx43` (16 vCPU, €0.24/hr): up to **2 parallel** boxes (2×16 = 32 cores). Good
  for 2 heavy crates in parallel.
- `ccx53` (32 vCPU, €0.47/hr): only **1 box** (single-server path, not fleet).
- Previously the dedicated limit was 8–15 cores so only `ccx33` worked as a single
  box. With 32 cores, multi-box dedicated fleets are now usable.

**Shared vCPU (cpx) fleet — still the default for >4 crates:**
- Server type must be the `cpx*2` generation (`cpx42`/`cpx52`/`cpx62`). The `x1`
  types (`cpx41`…) are phased out → "unsupported location"; `cpx22`/`cpx32` have
  <240 GB disks so the ccx33-derived snapshot won't deploy. Default `cpx42`
  (8 vCPU, €0.048/hr).
- Separate shared-CPU limit, higher ceiling, cheaper. Use for 5+ crates.

**Self-limiting:** `--max N` is a soft cost/concurrency cap. A worker that hits
`resource_limit_exceeded` waits and retries, so the fleet auto-fits the account's
actual ceiling — no need to know the exact number.

**Fleet cost guide:**
- 4× ccx33 ≈ €0.48/hr (dedicated, fast per-box)
- 2× ccx43 ≈ €0.47/hr (dedicated, medium)
- 5× cpx42 ≈ €0.24/hr (shared, cheaper, more boxes)

If you need a bigger snapshot footprint on smaller/cheaper boxes, re-snapshot from a
cpx box (smaller source disk) so `cpx22`/`cpx32` become usable.

## Guardrails

- **Billing is real.** The cardinal rule: a server must never be left running by
  accident. `build`/`test` trap-destroy on exit; for `up`/`keep-build` you own the
  teardown. Always end a session by confirming `status` shows nothing running.
- **Token hygiene.** The API token lives only in `~/.config/wm-burst/.env` (chmod
  600). Never echo it, never commit it, never paste it into output.
- **Arch must match.** The snapshot is x86_64 to match the laptop. Do not switch the
  builder to aarch64 — an ARM box can't produce runnable-on-laptop binaries or share
  the x86 sccache cache (this was deliberately decided; see memory
  `project_burst_builder_hcloud`).
- **Relation to `wm-burst`.** `wm-burst` (the Rust CLI) is the longer-term home for
  this; its real hcloud pod provider is still queued (PRD-constellation-burst-builder-hcloud).
  Until that ships, this skill's `cloudbuild.sh` is the working automation.

## Recreate the builder manually (reference)

If the script's `up` ever fails, the equivalent manual call:
```sh
source ~/.config/wm-burst/.env
curl -s -X POST -H "Authorization: Bearer $HCLOUD_TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"wintermute-builder","server_type":"ccx53","image":'"$SNAPSHOT_ID"',"location":"nbg1","ssh_keys":["wintermute-build"]}' \
  https://api.hetzner.cloud/v1/servers
```
Destroy: `curl -s -X DELETE -H "Authorization: Bearer $HCLOUD_TOKEN" $API/servers/<id>`.
