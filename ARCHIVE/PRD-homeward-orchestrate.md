# PRD: homeward-orchestrate — stand the fleet up as one running system

Status: Shipped
build_target: shell
Vision: visions/homeward.md

## TL;DR

Homeward is six crates and a Python sidecar that compile, ship binaries, and
have never been run together. `homeward-ingestd run` is a real AIMD-cadence
loop, `homeward-reportd serve` serves the owner API, and the embed sidecar is a
FastAPI service — but three independent processes meant to talk to each other
have nothing that brings them up, wires their shared env (DB path, sidecar URL,
source keys), supervises them, and proves the system is live. This PRD is the
deployment layer: systemd-user units + a `homeward` up/down/status wrapper + an
env contract + a health gate, so "the loop works here" becomes "the system runs
unattended."

## Why this exists

Phase-1 live inspection (2026-06-13, `~/wintermute/homeward` v0.9.3):

- Three runnable daemons exist as **separate** binaries with **no orchestration**:
  - `homeward-ingest/src/main.rs:4` — `homeward-ingestd run [--db <path>]`;
    `main.rs:125-129` confirms a real continuous loop
    (`tokio::time::interval`, 60s tick) over the AIMD orchestrator
    (`homeward-ingest/src/orchestrator.rs:1`).
  - `homeward-report/src/bin/reportd.rs:7` — `homeward-reportd serve [--port N]`.
  - `homeward/embed/homeward_embed/service.py` — FastAPI sidecar
    (`POST /enroll`, `/query`, `GET /health`), launched via `uv run
    homeward-embed-svc`.
- `find` for any `*compose*`, `*.service`, `deploy*`, or `Dockerfile` in the
  repo returned **only `.venv` site-packages** — there is no deployment artifact
  of any kind. The three processes share a DB and a sidecar URL by convention,
  with nothing that sets or validates that convention.
- The vision end-state (#2 "near-real-time aggregation keeps that store fresh,"
  #5 "a match alert fires within minutes") is unreachable without a supervised,
  composed, always-on deployment. This mirrors the lesson behind `homestead`
  (a companion device must keep itself alive) — applied to homeward's own fleet.

This is shell/config only: no Rust, no Python. It wires binaries that already
exist.

## What this builds

A `deploy/` subtree in `~/wintermute/homeward/` plus a `homeward` CLI wrapper:

- **systemd-user units** (templated, installed under `~/.config/systemd/user/`):
  - `homeward-embed.service` — runs the FastAPI sidecar (`Type=exec`,
    `Restart=always`, health via `GET /health`).
  - `homeward-ingest.service` — runs `homeward-ingestd run`, `After`/`Wants` the
    embed service, `Restart=always`.
  - `homeward-report.service` — runs `homeward-reportd serve`, `Restart=always`.
  - `homeward.target` — binds the three; `WantedBy` controls the fleet as a unit.
- **An env contract** at `~/.config/homeward/homeward.env` (a documented sample
  committed as `deploy/homeward.env.sample`): `HOMEWARD_DB`, `HOMEWARD_EMBED_URL`,
  `HOMEWARD_REPORT_PORT`, `RESCUEGROUPS_API_KEY`, Socrata feed list. Every unit
  reads `EnvironmentFile=`; no secret is hard-coded.
- **A `homeward` wrapper script** (`deploy/homeward`, installed to `~/.local/bin/`):
  `homeward up|down|status|logs|health`. `up` installs/links units + starts the
  target; `status` shows each unit's active state + the sidecar `/health`;
  `health` exits non-zero if any leg is down (a usable smoke for CI/self-review).
- **Idempotent install** (`deploy/install.sh`): re-runnable, links units, runs
  `systemctl --user daemon-reload`, never duplicates state.

### Non-goals

- No new ingest/report logic — only standing up what ships.
- No multi-machine/constellation deployment (that's constellation's domain).
- No public internet exposure — binds localhost by default; remote exposure is a
  later, explicitly-gated decision.

## Acceptance criteria

1. `deploy/install.sh` is idempotent: running it twice leaves exactly one copy of
   each unit and exits 0 both times.
2. After `homeward up`, `systemctl --user is-active homeward.target` reports
   `active` and all three child units are `active` (verified on this laptop, or
   the AC documents the exact failure if a binary is missing).
3. `homeward health` exits 0 when all three legs are up and non-zero when any one
   is stopped (test by stopping `homeward-embed.service` and re-running).
4. No secret or absolute machine-specific path is committed; every such value is
   read from `EnvironmentFile=~/.config/homeward/homeward.env`, and
   `deploy/homeward.env.sample` documents each key.
5. `homeward-ingest.service` declares `After=homeward-embed.service` and
   `Wants=homeward-embed.service`; `homeward down` stops the whole target cleanly
   (no orphaned process in the target's cgroup).
6. `homeward status` prints, in one view, each unit's active-state plus the embed
   sidecar's `/health` JSON.
