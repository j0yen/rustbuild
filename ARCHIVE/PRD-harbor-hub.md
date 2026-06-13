# PRD: harbor-hub — a permanent home port distinct from burst pods

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/constellation-burst-builder
Vision: visions/harbor.md

## TL;DR

The burst builder is **destroy-only**: `provider.rs` exposes exactly `create_pod`
and `destroy_pod`, and every one of the 528 logged lifecycles ends in a delete to
stop billing. There is no way to stand up a box and *keep* it. This PRD adds a
permanent-hub lifecycle — `wm-burst hub up|down|status` — that provisions or
**reuses** a single long-lived box, persists its identity separately from burst
pods, never auto-destroys it, and tears it down only on an explicit, confirmed
`hub down`. It's the keystone every other harbor component stands on.

## Why this exists

- `constellation-burst-builder/src/provider.rs` (read 2026-06-12) has only
  `create_pod(&cfg) -> id` and `destroy_pod(id)`. No `get_server`, no power-state,
  no reuse — so "is my permanent box already up?" is unanswerable in-tool.
- `~/.config/wm-burst/.env`: `BUILDER_IP=''`, `BUILDER_ID=''` (`# server deleted
  2026-06-05`). State doesn't survive a burst; there's nowhere to record a
  permanent box's identity.
- `config.rs` has `remote_host: String` and an optional `HcloudPodConfig`, but
  nothing models a hub separate from an ephemeral pod.
- User asked directly (2026-06-12): *"how do i upgrade to a permanent box?"* and
  chose Option A (permanent cheap hub + keep ephemeral bursts).

## What this builds

**New config (`src/config.rs`):** a `HubConfig` (optional, parallel to `pod`):
```
pub struct HubConfig {
    pub server_type: String,   // default "cpx11" (x86, so it can host an x86 build-server)
    pub location: String,      // default "nbg1"
    pub image: String,         // snapshot id, reuses SNAPSHOT_ID
    pub ssh_key_name: String,
    pub hub_id: Option<String>,   // persisted once provisioned: "<id>@<ip>"
}
```
Persisted to/from a `[hub]` section written into `~/.config/wm-burst/hub.json`
(NOT `.env` — keep the secret file untouched; hub identity is non-secret).

**New provider methods (`src/provider.rs`, `PodProvider` trait):**
- `get_server(&self, id: &str) -> Result<Option<ServerState>>` — query Hetzner for
  a server by id; `None` if it doesn't exist. `ServerState { id, ip, status }`.
- Reuse `create_pod` for provisioning (same snapshot path); the hub just doesn't
  get destroyed.
- Mock provider implements `get_server` deterministically for tests.

**New command (`src/commands/hub.rs`, wired in `mod.rs` + `main.rs`):**
- `hub up` — if `hub_id` is persisted AND `get_server` reports it running → no-op,
  print "hub already up @ <ip>". Else provision via `create_pod`, persist the new
  `hub_id`, print the ip. **Idempotent.**
- `hub down` — **refuse unless `--yes` is passed** (the hub will become stateful;
  see harbor-mirror/harbor-cache). On `--yes`: `destroy_pod(hub_id)`, clear
  persisted `hub_id`, log a `HUB-DOWN` line to `cost.log`.
- `hub status` — print persisted id, live status from `get_server`, ip, and
  uptime-since (from the `HUB-UP` cost.log line). "hub: none" if unprovisioned.
- Log `HUB-UP id=… type=… ip=…` / `HUB-DOWN id=…` lines to `cost.log` with a
  distinct prefix so harbor-thrift can separate standing cost from burst cost.

**No change to existing `build`/`test`/`pod`/`up`/`down` paths** — burst pods stay
destroy-only. Hub is a parallel lifecycle.

## Acceptance criteria

1. `cargo build` and `cargo test` pass under rustc 1.85 (workspace MSRV); clippy
   clean under the repo's existing `-D warnings` config (no new lints).
2. `HubConfig` round-trips: writing then reading `hub.json` yields an equal struct
   (unit test with a temp dir).
3. `wm-burst hub up` is idempotent: with a persisted+running `hub_id` (mock
   provider reporting `running`), it makes **zero** `create_pod` calls and prints
   "already up" (assert via mock call counter).
4. `wm-burst hub up` with no persisted hub calls `create_pod` exactly once and
   persists the returned id (assert via mock + re-read `hub.json`).
5. `wm-burst hub down` **without** `--yes` refuses (non-zero exit, no `destroy_pod`
   call) and prints why; **with** `--yes` calls `destroy_pod` once and clears
   `hub_id`.
6. `wm-burst hub status` with no persisted hub prints "hub: none" and exits 0;
   with a persisted hub it prints id + live status from `get_server`.
7. `cost.log` gains a `HUB-UP`/`HUB-DOWN` line (distinct prefix) on the respective
   transitions; existing burst `UP`/`DOWN` lines are unchanged.
8. The mock provider's `get_server` is deterministic and the existing
   `mock_provider_*` tests still pass (no regression to create/destroy).
