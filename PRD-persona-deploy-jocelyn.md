# PRD: persona-deploy-jocelyn — turn the parts into a companion

Status: Draft v0.1
build_target: shell
Vision: visions/persona.md

## TL;DR

Every piece of the elder persona is shipped and wired, yet none of it is
running. The live `~/.config/wintermute/brain.toml` `[persona]` still reads
`self_name = "wintermute"` with **no `forbidden_terms`, no
`[persona.introduction]`, and no `redline`** — so the forbidden-vocab list
reaches the model only as prompt advice and the wired `redline::enforce()`
guard is dormant (`RedlineAction::Off`). `persona-deploy-jocelyn` is the
idempotent, reversible installer that *assembles* the deployment: it applies
the `jocelyn` profile, sets the warm `self_name` Joe chose, activates redline
enforcement, restarts wm-brain, and proves the result reconciles. A drawer of
parts becomes a companion.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The deployment has never been run.** Live `[persona]` (read this session):
  `self_name = "wintermute"`, `register = "warm-elder"`, `wake_word = "hey
  wintermute"`, and **no** `forbidden_terms` / `introduction` / `redline`
  keys. The mechanism to write them — `wmd persona profile apply jocelyn
  --write` (`wintermute-brain/src/main.rs:476`, backs up to `brain.toml.bak`
  via `apply_profile_to_config`, `src/profile.rs`) — exists but was never
  invoked.
- **redline is wired but dormant.** `redline::enforce()` is genuinely called
  in the daemon reply→TTS path (`src/daemon.rs:2277`, reading
  `cfg.persona.redline` at `src/daemon.rs:2060`). Its default is
  `RedlineAction::Off` (`src/redline.rs:39`, `impl Default`). Because the live
  config sets no `redline` key, the guarantee `persona-redline` shipped is
  inert in production. Deployment, not code, is the missing link.
- **The preset's self_name is a placeholder that is actually wrong.** The
  `jocelyn` profile sets `self_name = "jocelyn"` (`src/profile.rs:151`) — but
  Jocelyn is the *principal* (Joe's mother), not what the assistant should call
  *itself*. The vision's Open Question #1 leaves the real name to Joe
  ("Clara/Rose/Nora/Wren"). This installer must parameterize the name, never
  hardcode the principal's name onto the assistant.
- **It must be reversible and idempotent.** `brain.toml` is hand-editable and
  other tools (`wmd route prefer`) also write adjacent sections; a blind
  overwrite is unsafe. `apply_profile_to_config` already writes `brain.toml.bak`
  — the installer leans on that and is safe to re-run.

## What this builds

A new standalone repo `~/wintermute/persona-deploy-jocelyn/` (shell), published
as `j0yen/persona-deploy-jocelyn`, containing:

- **`deploy.sh`** — the installer. Flags:
  - `--self-name <NAME>` (required) — the warm name the assistant answers to.
    Refuses to proceed if `NAME` equals `jocelyn` (the principal's name) unless
    `--allow-principal-name` is also passed, with a printed warning explaining
    why naming the assistant after the user is discouraged.
  - `--wake-word <PHRASE>` (default: `hey <self-name lowercased>`).
  - `--redline <safe-phrase|off>` (default: `safe-phrase`) — sets the
    `[persona].redline` action; `safe-phrase` activates enforcement.
  - `--dry-run` (default) vs `--execute` — dry-run prints the resulting diff via
    `wmd persona profile diff jocelyn` plus the planned name/redline overrides
    and exits 0 without writing.
  - `--no-restart` — skip the wm-brain restart (config-only).
- **Flow on `--execute`:**
  1. Snapshot current `brain.toml` to a timestamped backup beside the tool's
     own state dir (in addition to the `.bak` that `apply` writes).
  2. `wmd persona profile apply jocelyn --write`.
  3. Patch `self_name`, `wake_word`, and `[persona].redline` to the chosen
     values using `wmd` where a subcommand exists, else a minimal `toml`-safe
     edit (prefer `wmd`; the script must not corrupt unrelated tables).
  4. Restart the user wm-brain unit (`systemctl --user restart wm-brain` if the
     unit exists; otherwise print the manual restart command and continue).
  5. Verify: `wmd persona show` reflects the new name + non-empty
     `forbidden_terms` + active redline; `wmd persona profile diff jocelyn`
     reports only the intentional name/wake-word/redline deltas (everything else
     reconciled).
- **`rollback.sh`** — restores the most recent timestamped backup and restarts
  wm-brain. Prints what it restored.
- **`README.md`** — what it does, the name decision (points at vision Open
  Q#1), and the rollback path.
- **`tests/`** — shell tests using a *fixture* `brain.toml` in a temp dir
  (never the live config): assert dry-run writes nothing; assert `--execute`
  on the fixture yields a config whose `diff jocelyn` reconciles; assert the
  principal-name guard fires; assert rollback restores byte-for-byte.

Dependencies: `wmd` (wintermute-brain ≥ v0.22.0, on `$PATH` or
`~/wintermute/wintermute-brain/target`). No Rust built here — this orchestrates
the shipped binary.

## Acceptance criteria

1. `deploy.sh --help` prints usage including `--self-name`, `--redline`,
   `--dry-run/--execute`, and `--allow-principal-name`.
2. `deploy.sh --self-name Clara` (no `--execute`) is a dry-run: it writes
   nothing to any config and exits 0, printing the would-be diff.
3. Against a temp fixture `brain.toml` lacking persona safety fields,
   `deploy.sh --self-name Clara --execute --config <fixture> --no-restart`
   produces a config where `wmd persona profile diff jocelyn --config <fixture>`
   shows only `self_name`/`wake_word`/`redline` deltas — `forbidden_terms` and
   `introduction` fully reconciled (non-empty).
4. After that run the fixture's `[persona].redline` is an active `SafePhrase`
   action (not `Off`), and `[persona].self_name = "Clara"`.
5. `deploy.sh --self-name jocelyn --execute` (without `--allow-principal-name`)
   refuses with a non-zero exit and an explanatory message; adding
   `--allow-principal-name` lets it proceed.
6. `rollback.sh` restores the pre-deploy backup of the fixture byte-for-byte.
7. The deploy run is idempotent: running it twice against the same fixture
   yields an identical final file (second run's diff is unchanged).
8. `tests/` runs green via a single `./tests/run.sh` entrypoint with no network
   and no writes outside the temp dir.

## Out of scope

- Choosing the actual name (Joe's call — vision Open Q#1).
- The `Regenerate` redline action (see `PRD-persona-redline-regenerate.md`).
- Any change to `wintermute-brain` source — this consumes the shipped CLI.
- Auto-starting wm-brain on a box where it isn't already a user unit.
