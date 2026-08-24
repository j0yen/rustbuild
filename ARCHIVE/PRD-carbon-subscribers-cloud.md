# PRD-carbon-subscribers-cloud

**Status:** Shipped v1.0 — 2026-06-20. Homeward daemons (ingest+report) active on hub, API answering (/health → {"status":"ok"}). Laptop daemons left running in parallel during burn-in (AC4/5 deferred to follow-on PRD once hub proves stable).
**Vision:** visions/carbon.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-carbon-hub-access, PRD-carbon-node-identity

## TL;DR

The persistent NATS subscriber daemons that must run 24/7 — `homeward-ingest`
(AIMD cadence loop pulling shelter data), `homeward-report` (owner API), and
their embed sidecar — currently live on this laptop and go dark when it sleeps.
This PRD relocates the fleet-wide subscribers to the always-on hub and classifies
which subscribers are per-node (voice, tether, busbridge) and must stay local.

## Why this exists

**Evidence (2026-06-20):**
- Active laptop-bound, always-on daemons this session: `homeward-ingest`
  (`homeward-ingestd run` — AIMD cadence loop), `homeward-report`
  (`homeward-reportd serve --port 8081`), `homeward-embed` (DINOv2 FastAPI
  sidecar). A cadence loop that pauses every time the lid closes corrupts its
  own AIMD timing assumptions.
- Per-node subscribers also run here and must NOT move: `wm-busbridge` (bridges
  *this* node's agorabus to NATS), `wm-tether` (this node's fleet link). One
  busbridge per node is the topology — relocating it makes no sense.
- `recalld` is a subscriber-shaped daemon but holds canonical memory; its
  placement is an open question in the vision, explicitly **out of scope** here.
- The fleet bus already carries events cross-node (constellation verified e2e
  pub/sub wintermute↔ryzen7), so a subscriber on the hub sees the same `wm.*`
  events it would see locally.

## What this builds

Runs on: **this laptop**, deploying daemons to the **hub**.

1. **Classify** each subscriber daemon in `placement.toml`: `homeward-ingest`,
   `homeward-report`, `homeward-embed` → `hub`; `wm-busbridge`, `wm-tether` →
   `node-local` (every node). `recalld` → leave unset (out of scope).
2. **Build the relocated daemons for ARM** (via carbon-hub-access path) and
   install them to the hub's `~/.local/bin/`. The homeward embed sidecar is
   Python/uv — relocate by syncing the project + `uv run`, no ARM compile needed.
3. **Migrate state:** the homeward SQLite DBs
   (`~/.local/share/homeward/*.db`) move with the daemons; document a one-time
   rsync + a note that the laptop copies become stale.
4. **Enable on the hub, disable on the laptop**, each unit guarded by
   `ExecCondition=wm-node should-run <daemon>`.
5. **Verify the API still answers** from its new home (the report API at :8081 is
   now hub-hosted; update any client base-URL that pointed at the laptop).

## Acceptance criteria

1. `placement.toml` maps `homeward-ingest`/`homeward-report`/`homeward-embed` to
   `hub` and `wm-busbridge`/`wm-tether` to `node-local`.
2. `ssh hub 'systemctl --user is-active homeward-ingest homeward-report'` → both
   `active`; the ingest cadence loop logs a successful pull on the hub.
3. The homeward report API answers on the hub: `curl
   http://<hub-tailscale-ip>:8081/healthz` (or the real health route) returns 2xx.
4. The relocated daemons are **disabled on the laptop** and their `ExecCondition`
   prevents accidental local start.
5. `wm-busbridge` and `wm-tether` remain `active` on the laptop (per-node
   subscribers untouched); the hub's busbridge still passes `wm-busbridge
   selftest`.
