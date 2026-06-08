# PRD: mend-binstale-wire — the built staleness detector the fleet never wired in

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/binstale
Vision: visions/mend.md

## TL;DR

`~/wintermute/binstale/` is a complete, mature tool — five source modules
(`proc`, `source`, `verdict`, `output`, `error`), a verdict taxonomy README,
git history through 2026-06-02 — that classifies a running binary as
`fresh | deleted-exe | inode-drift | prov-stale | behind-head`. It is also
**not installed**: `which binstale` fails, so every self-review prints "fleet
staleness check skipped" and the fleet drifts undetected. This PRD installs
binstale and gives it a self-review-shaped aggregate (`binstale fleet`) that
emits a `docket report` when any daemon is running stale. A built tool finally
does its job.

## Why this exists

- **The detector exists but no one runs it.** `ls ~/wintermute/binstale/`
  shows `src/{proc,source,verdict,output,error}.rs`, `Cargo.toml`, `README.md`,
  `target/`, `.git/`. `ls ~/.local/bin/binstale` → "No such file or
  directory". The build is done; the install never happened.
- **The self-review actively skips it, every run.** journal 2026-06-07:
  "binstale not yet installed — fleet staleness check skipped". The
  2026-06-06 reflection: "binstale still absent". This is a recurring,
  evidence-rich skip across multiple runs.
- **The exact failure binstale was built to catch already recurred by hand.**
  binstale's own README cites the agorabus stale-binary incident
  ("Rediscovered by hand across three consecutive self-review runs before
  binstale existed") — the resolution noted in memory
  ([[self_agorabus_restart_kills_voice]] adjacent; agorabus-stale-binary
  RESOLVED 2026-06-06 only after manual sweep). The tool that would have
  caught it automatically was sitting uninstalled the whole time.
- **It is not even on the docket.** Unlike `ctrace-sessionend-flake` and
  `warden-enforcer-inert`, the binstale skip is mentioned in prose but never
  `docket report`ed — so it can't escalate. Wiring it to docket fixes that too.

## What this builds

Extends `~/wintermute/binstale` (rust-extend; preserve existing modules and
verdict taxonomy):

1. **`binstale fleet` subcommand** — aggregate mode over a curated fleet list:
   - Default targets: the wintermute daemon fleet (agorabus, wm-stt, wm-tts,
     wm-dialog, wm-brain, recalld) by `comm`/`intent`, plus installed
     `~/.local/bin/*` tools that map to a `~/wintermute/<slug>` source repo.
   - `--all` opt-in to scan every running `/proc/PID/exe` (cold-cost flagged).
   - Reuses the existing per-process verdict engine; output is a JSON array of
     `{pid, comm, exe, source_repo, verdict, evidence}`.
2. **`--format json|human|docket`** on `fleet`:
   - `json` — machine aggregate for the self-review to parse.
   - `human` — table, priority-sorted (`deleted-exe` > `inode-drift` >
     `prov-stale` > `behind-head` > `fresh`).
   - `docket` — emits, for each non-`fresh` daemon, a ready-to-run
     `docket report --key binstale-<comm> --severity <warn|alert> --title …
     --evidence pid:<pid>` line (alert for `deleted-exe`/`inode-drift`, warn
     otherwise). The self-review pipes these to `docket`.
3. **An install deliverable** — a `make install` / `cargo install --path .`
   target (or `scripts/install.sh`) that places the release binary at
   `~/.local/bin/binstale`, and a one-line self-review playbook snippet
   (committed as `docs/self-review-wire.md`) that calls
   `binstale fleet --format docket | sh` in the fleet-health phase.

Constraints: SIGPIPE reset on first line of `main()` per
[[self_sigpipe_panic_toolkit]] (binstale is a `~/.local/bin` tool that will be
piped to `head`/`sh`). rustc 1.85, no let-chains. Detection only — never
restart, kill, or reinstall anything (preserve the existing README contract).

## Acceptance criteria

1. `cargo build --release` in `~/wintermute/binstale` succeeds; `cargo test`
   green. Existing per-process verdict tests still pass unchanged.
2. `binstale fleet --format json` over a synthetic fixture of running-process
   descriptors (mocked `/proc` inputs) returns a JSON array with one entry per
   target and a correct `verdict` per the existing taxonomy.
3. `binstale fleet --format docket` emits, for a fixture containing one
   `deleted-exe` and one `behind-head` daemon, exactly two `docket report`
   lines with `--severity alert` and `--severity warn` respectively, each
   carrying a stable `--key` and a `pid:` evidence token.
4. A `fresh`-only fleet produces zero `docket` lines and exit code 0.
5. SIGPIPE: `binstale fleet --format docket | head -1` exits 0 with no panic
   (sigpipe reset verified).
6. The install deliverable places an executable at `~/.local/bin/binstale`
   and `which binstale` resolves it; `docs/self-review-wire.md` documents the
   exact pipe the self-review fleet phase should run.
7. `--all` is gated behind an explicit flag; without it, `fleet` scans only the
   curated list (verified: a process outside the list is absent from default
   output, present under `--all`).
