# PRD: agorabus-reload-build — the `reload --build` flag that rollout already calls

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/agorabus
Vision: visions/headway.md

## TL;DR

`rollout install` shells to `agorabus reload --build` to bring a `behind-head`
agorabus daemon current — but `agorabus reload` has no `--build` flag, so the
call is a dead end and the agorabus daemon stays days behind its source. This
PRD adds the missing `--build` flag: on it, `reload` detects staleness, routes
the recompile through cloudbuild (subprocess, never in-process cargo), installs
the fresh binary, then runs the existing reload bounce.

## Why this exists

Phase-1 inspection, 2026-06-18:

- `rollout install --help` states it restarts agorabus "via `agorabus reload
  --build` (for agorabus) or `systemctl --user restart` (for all other
  daemons)." So rollout already depends on the flag.
- `agorabus reload --help` lists only `--format`, `--socket`, `--require-fresh`,
  `--start-if-absent`, `--dry-run`/`--apply`, and the drain/reconnect timeouts.
  **There is no `--build`.** The flag rollout calls does not exist.
- The journal records the consequence directly: "agorabus reload --build flag
  absent — the playbook assumes a `--build` subcommand that doesn't exist… The
  manual fix is: build via cloudbuild → install → `reload --apply`. Worth filing
  a PRD to add `reload --build` support." (journal 2026-06-18, "Notable").
- `agorabus doctor` already compares the running image to the installed binary
  (exits 0=current/1=stale/2=unknown), and `reload` already does a clean
  SIGTERM-relaunch-reconnect bounce (`src/reload.rs`). The only missing link is
  the *rebuild* step ahead of the bounce.

## What this builds

Extend `~/wintermute/agorabus` (the existing `reload` subcommand in
`src/reload.rs` / `ReloadConfig`).

- Add `--build` to `agorabus reload`. When set, before the bounce:
  1. Determine whether the installed agorabus binary is behind its source HEAD
     (reuse the doctor/binstale notion of `behind-head`; the agorabus repo path
     is known — its own `CARGO_MANIFEST_DIR` lineage / `~/wintermute/agorabus`).
  2. If behind (or `--no-require-fresh`), shell out to
     `cloudbuild.sh build agorabus` (path from `AGORABUS_CLOUDBUILD` env or the
     default `~/.claude/skills/cloudbuild/cloudbuild.sh`), then install the
     pulled artifact to the daemon's exec path via atomic temp-then-rename.
  3. Then perform the existing reload bounce against the freshly-installed binary.
- `--build` composes with `--dry-run` (default): dry-run prints the rebuild +
  install + bounce plan and the exact cloudbuild command, mutating nothing.
- The build step is a **subprocess to cloudbuild**, never an in-process or local
  `cargo build`. If cloudbuild is unreachable, `reload --build` aborts with a
  structured error and a non-zero exit *before* touching the running daemon —
  the live daemon is never bounced toward a binary that failed to build.

## Acceptance criteria

1. `agorabus reload --build --dry-run --format json` emits a plan containing the
   rebuild step (the exact `cloudbuild.sh build agorabus` command), the install
   destination, and the bounce target; mutates nothing; exits 0.
2. `agorabus reload --build` (apply) invokes cloudbuild as a subprocess for the
   build step — verified with a stub cloudbuild via `AGORABUS_CLOUDBUILD` — and
   only proceeds to install + bounce when the stub reports success.
3. When the installed binary is already current and `--require-fresh` holds
   (default), `reload --build` skips both the rebuild and the bounce and reports
   a no-op, matching the existing `--require-fresh` semantics.
4. When cloudbuild is unreachable/missing, `reload --build` aborts with a
   structured error and non-zero exit and does NOT SIGTERM the running daemon
   (asserted: the pre-existing daemon pid is unchanged after the failed call).
5. There is no local `cargo build`/`cargo install` path in the `--build` flow;
   the only compiler invocation is the cloudbuild subprocess.
6. `rollout install` against an agorabus daemon now reaches a real flag — an
   integration-style test (or a documented manual check) confirms
   `agorabus reload --build --dry-run` exits 0 where it previously errored on an
   unknown flag.
7. All pre-existing agorabus tests stay green; `cargo test` green and no new
   clippy warnings over the repo baseline; `agorabus reload --help` documents
   `--build`.
