# PRD: harbor-route — /cloudbuild defaults warm, bursts only when cold

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/.claude/skills/cloudbuild/cloudbuild.sh
Vision: visions/harbor.md

## TL;DR

`/cloudbuild` is destroy-only: `up → build → pull → DOWN`, a fresh ccx53 for every
job. With a permanent warm hub (harbor-hub) and a shared cache (harbor-cache), most
builds shouldn't pay 30s of boot + a cold compile. This PRD makes `/cloudbuild`
hub-aware: **incremental builds run warm on the hub**; **cold/full compiles still
burst a ccx53 pod** but point sccache at the hub's shared cache. Destroy-only
becomes an explicit `--ephemeral` opt-out, not the default.

## Why this exists

- `cloudbuild.sh` (read 2026-06-12) drives everything through `up`/`build`/`down`
  with an EXIT trap that destroys the box — correct for "stop billing," wasteful
  now that a warm hub exists.
- harbor-hub + harbor-cache make a *standing, warm* builder available; without this
  PRD, `/cloudbuild` ignores it and keeps cold-bursting ccx53 (the last two logged
  runs were exactly that — ~2 min each, mostly boot + cold compile).
- The routing decision (warm hub vs cold burst) is the payoff of the whole harbor
  vision — the place the permanent box actually saves wall-clock and money.

## What this builds

**`cloudbuild.sh` gains hub-aware routing:**
- A `route <crate>` decision helper: if `hub.json` exists and the hub is reachable
  (via harbor-bridge-probe, or an ssh ping fallback) AND the build is "small"
  (heuristic: incremental — a prior target dir exists / changed-files below a
  threshold), route to the **warm hub** (ssh build on the standing box, no
  create/destroy). Else **burst** a ccx53 pod as today, but export the
  harbor-cache `SCCACHE_*` block so even the cold burst hits the shared cache.
- `build`/`test` consult `route` by default. A new `--ephemeral` flag forces the
  old create→build→destroy path (escape hatch / when the hub is down).
- A new `--hub` flag forces the warm-hub path (skip the heuristic).
- Teardown logic unchanged for burst pods; the **hub is never destroyed** by
  `/cloudbuild` (only `wm-burst hub down --yes` does that).

**SKILL.md updated** to document the warm-default model, the `--ephemeral`/`--hub`
flags, and that the hub bills monthly (so `status` should show hub standing cost,
deferring the number to `wm-burst cost`).

## Acceptance criteria

1. `cloudbuild.sh route <crate> --dry-run` prints its decision ("warm hub @ <ip>"
   vs "burst ccx53") and the reason (hub reachable? incremental?) without doing any
   work.
2. With a `hub.json` present + a reachable-hub fixture + an incremental crate, the
   default `build` routes to the **warm hub** and makes **no** `create_pod`/destroy
   calls (assert via dry-run plan).
3. With no `hub.json` (or `--ephemeral`), `build` falls back to the existing
   create→build→destroy path unchanged — the current behavior is preserved exactly
   when no hub exists.
4. When bursting a pod, the harbor-cache `SCCACHE_*` env block is exported into the
   pod's build environment (assert the planned remote command sources
   `cache.env`/`harbor-cache-client-env.sh`).
5. `--hub` forces the warm path and errors clearly if the hub is unreachable (no
   silent fallback to a 30s burst when the user explicitly asked for the hub).
6. `/cloudbuild` never destroys the hub: grep the routing/teardown code paths —
   `destroy_pod`/`down` is only ever invoked against a burst pod id, never the
   persisted `hub_id`.
7. SKILL.md documents the warm-default model, both new flags, and the monthly
   billing caveat with a pointer to `wm-burst cost`.
8. An offline check script (`route --dry-run` over fixtures: hub-up-incremental,
   hub-up-coldfull, no-hub, hub-unreachable) exercises all four routing branches
   and is the PRD's acceptance gate.
