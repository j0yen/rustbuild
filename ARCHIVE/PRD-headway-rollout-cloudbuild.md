# PRD: headway-rollout-cloudbuild — rollout builds through cloudbuild, not local cargo

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/rollout
Vision: visions/headway.md

## TL;DR

`rollout` is the fleet's serialized safe-restart tool, but its build step
compiles **locally** (`cargo build --release`), violating the standing rule that
all cargo routes through cloudbuild. This PRD replaces rollout's local-build
default with the cloudbuild route (consuming the `headway-build` primitive),
while preserving rollout's proven orchestration — serialized restart,
re-register confirmation, and the voice `--restart-window` guard.

## Why this exists

Phase-1 inspection, 2026-06-18:

- `rollout/src/fleet.rs:71` returns `"cargo build --release"` as the default
  `build_cmd`; `src/fleetgen.rs:116/422/430` writes that same local command into
  generated fleet configs. Every `behind-head` daemon rollout rebuilds is
  compiled on this laptop.
- This contradicts `feedback_cloudbuild_over_build`: ALL cargo must route
  through `cloudbuild.sh` (Hetzner); local `/autobuilder` cargo is disabled.
  Silent local fallback is explicitly not allowed.
- The cloudbuild path was always the intent but was left as a manual
  `#[ignore]`d note: `rollout/src/warmswap.rs:630` — "CI (cloudbuild) stays
  hermetic. Run manually with: …".
- `headway-build` (this vision's foundation PRD) provides the missing primitive:
  a `build(crate)` that shells to `cloudbuild.sh` and returns a structured
  verdict. rollout should consume it instead of carrying its own local command.

## What this builds

Extend `~/wintermute/rollout`.

- Change the default build path so `rollout apply` / `rollout install` recompile
  a `behind-head` daemon via the cloudbuild route — either by invoking the
  `headway` binary (`headway build <crate-dir>`) or by shelling directly to
  `cloudbuild.sh` with the same guard. The default `build_cmd` is no longer a
  local `cargo build`.
- Keep all existing orchestration unchanged: strictly serialized one-at-a-time
  restart, agorabus re-register confirmation between daemons, and the
  `--restart-window` voice-set guard (`rollout install --restart-window`).
- Provide a `--local-build` escape hatch ONLY as an explicit opt-in that logs a
  loud warning to stderr when used; the default and all unattended/cron paths
  must never build locally.
- Where rollout installs agorabus specifically, prefer `agorabus reload --build`
  (the flag added by the sibling PRD) so the agorabus path is consistent.

## Acceptance criteria

1. The default build path for a `behind-head` daemon routes through cloudbuild
   (via `headway build` or a direct `cloudbuild.sh` subprocess); no rollout code
   path runs `cargo build --release` locally by default. The
   `fleet.rs:71`/`fleetgen.rs` default is updated accordingly.
2. `rollout apply --dry-run` (and `rollout install --dry-run`) print a plan whose
   build step names the cloudbuild command, not a local cargo command.
3. The voice `--restart-window` guard is preserved unchanged — an existing test
   asserting refusal-to-restart-within-window still passes.
4. The serialized-restart + re-register-confirm orchestration is unchanged — the
   pre-existing rollout ordering/confirmation tests still pass.
5. `--local-build` exists, is off by default, and when passed emits a loud
   stderr warning identifying it as a hard-rule escape hatch. No unattended path
   selects it implicitly.
6. With a stub cloudbuild (injected via the same env var `headway-build` honors),
   a `rollout apply` over a one-daemon fixture drives the build through the stub
   and proceeds to install/restart only on stub success.
7. All pre-existing rollout tests stay green; `cargo test` green and no new
   clippy warnings over the repo baseline.
