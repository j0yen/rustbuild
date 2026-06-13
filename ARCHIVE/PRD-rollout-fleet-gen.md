# PRD: rollout-fleet-gen — author the missing fleet.toml from live state, instead of hand-editing it

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/rollout
Vision: visions/vigil.md

## TL;DR

`rollout` is fully built but **inert**: every `rollout plan`/`apply`
refuses with `fleet.toml error: cannot read
~/.config/rollout/fleet.toml: No such file or directory`. The config it
needs — a per-daemon recipe map — was never authored, so the one tool
that would safely cure fleet-staleness cannot run at all. `rollout
fleet-gen` derives a *candidate* `fleet.toml` from what is actually live
on the box (the binstale scan crossed with the systemd-user unit map),
writes it to `fleet.toml.proposed` with a diff against any existing
config, and never touches the live file. It turns the vision's deferred
"discuss the canonical launch path per daemon" open question into a tool
the user reviews and accepts, rather than a hand-edit that never happens.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The config does not exist.** `find ~/.config -name 'fleet.toml*'`
  returns nothing. `rollout plan --only wm-audio` errors immediately:
  `rollout: fleet.toml error: cannot read
  /home/jsy/.config/rollout/fleet.toml (os error 2)`. The orchestrator
  is a no-op until this file exists.
- **Staleness stays open because of it.** Self-review journals for
  2026-06-11/12/13 all carry `fleet-binary-staleness`
  (wm-audio/dialog/tts/stt `behind-head`) under "Pending your call" and
  pre-fill only `rollout plan --only <daemon>` — which itself errors.
  The finding recurs because the cure is unreachable.
- **The inputs to derive the recipe are all present.** `binstale scan
  --format json` already emits per-daemon `comm`, `exe_path`,
  `source_repo`, `source_head_commit` (verified live: wm-audio →
  `source_repo:/home/jsy/wintermute/wintermute-audio`). The systemd
  units name the launch path: `wm-audio.service` →
  `ExecStart=%h/.cargo/bin/wm-audio start`; `wm-dialog|stt` →
  `%h/.local/bin/...`; `wm-tts` →
  `%h/.local/bin/wm-tts --cache-config /etc/wintermute/tts-cache.yaml
  start`. Everything `DaemonRecipe` needs can be read, not guessed.
- **The vision flagged this as a decision, never a tool.** vigil Open
  Q#1: *"rollout reads launch recipes from a config file the user
  authors; deriving them automatically … is deferred. Discuss the
  canonical launch path per daemon before rollout apply runs."* The
  discussion never happened; the file never got written. A generator
  resolves it by proposing, not deciding.

## What this builds

A new `Command::FleetGen` in `~/wintermute/rollout/` (`src/fleetgen.rs`,
wired in `cli.rs`/`main.rs`).

`rollout fleet-gen [--out <path>] [--match <regex>]`:

1. Run `binstale scan --format json`; for each daemon entry take `comm`,
   `exe_path`, `source_repo`.
2. Scan `~/.config/systemd/user/*.service` for the `ExecStart=` line
   whose first token (after `%h`→`$HOME` expansion) canonicalises to the
   daemon's `exe_path`; capture the owning **unit name**. (Reuse the
   `find_unit_for_dest` logic that already exists in `install.rs` rather
   than reimplementing the scan.)
3. Emit a `[[daemon]]` recipe per matched daemon:
   - `name` = binstale `comm`
   - `repo` = binstale `source_repo`
   - `build_cmd` = default (`cargo build --release`)
   - `install_cmd` = derived from the exe dest convention
     (`~/.cargo/bin/<name>` → `cargo install --path . --root ~/.cargo`;
     `~/.local/bin/<name>` → `cargo install --path . --root ~/.local`)
   - `unit` = the systemd unit found in step 2 (new optional field;
     consumed by PRD rollout-apply-systemd)
   - `launch_cmd` = `systemctl --user restart <unit>` (a safe default so
     a fleet.toml is usable even before apply-systemd ships; the
     dedicated `unit` field is what apply-systemd will prefer)
   - leave `healthcheck` unset (the default agorabus-peers check applies)
4. Write the rendered TOML to `--out` (default
   `~/.config/rollout/fleet.toml.proposed`), creating
   `~/.config/rollout/` if absent. **Never** write `fleet.toml` itself.
5. If a live `fleet.toml` already exists, print a unified diff
   (proposed vs live) to stdout and note which daemons are new/changed.
6. Print a one-line footer telling the user the exact command to accept:
   `review ~/.config/rollout/fleet.toml.proposed, then mv it to fleet.toml`.

Adding the optional `unit: Option<String>` field to `DaemonRecipe`
(`#[serde(default)]`) is in scope here so the generated TOML round-trips;
the field is *consumed* by the sibling PRD rollout-apply-systemd.

Deps: `serde`/`toml` (already in the crate), `regex` (already used in
`health.rs`). No new heavy deps.

## Acceptance criteria

1. `rollout fleet-gen --help` exists and documents `--out` and `--match`.
2. On this box, `rollout fleet-gen` exits 0 and writes a non-empty
   `~/.config/rollout/fleet.toml.proposed` containing a `[[daemon]]`
   block for each daemon that binstale reports AND whose `exe_path`
   matches a systemd-user unit's `ExecStart` (at minimum the live
   `wm-audio`, `wm-dialog`, `wm-stt`, `wm-tts`).
3. Each generated `[[daemon]]` block has `name`, `repo`, `install_cmd`,
   `launch_cmd`, and a `unit` field; the `unit` value equals the real
   owning unit (e.g. `wm-audio` → `wm-audio.service`).
4. `rollout fleet-gen` never creates or overwrites
   `~/.config/rollout/fleet.toml`; only the `.proposed` (or `--out`)
   path is written. A test asserts the live path is untouched when it
   pre-exists with sentinel content.
5. The generated `fleet.toml.proposed`, when copied to `fleet.toml`,
   parses cleanly: `rollout plan` exits 0 and no longer errors with
   "cannot read fleet.toml" / "unknown daemons". (Verify by copying to a
   temp `--config` path if one exists, else document the manual check.)
6. A daemon binstale reports but with no matching systemd unit is
   **omitted** from the output with a `# skipped: no systemd unit for
   <name>` comment line, not guessed.
7. `cargo build --release` and `cargo test` pass; `cargo clippy
   -D warnings` clean for the new module (crate baseline permitting).
