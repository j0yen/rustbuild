# PRD: persona-deploy-doctor — the companion should not silently drift

Status: Draft v0.1
build_target: shell
Vision: visions/persona.md

## TL;DR

Once `persona-deploy-jocelyn` activates the elder persona, nothing keeps it
activated. `brain.toml` is hand-editable, and other tools (`wmd route prefer`,
future installers) write adjacent tables; a stray edit, a config reset, or a
fresh-box re-provision can quietly drop `forbidden_terms`, flip `redline` back
to `Off`, or reset `self_name` to `wintermute` — and no one would notice until
Jocelyn hears the word "computer." `persona-deploy-doctor` is a periodic health
check that asserts the deployed persona still matches what Joe deployed, and
surfaces drift loudly the way self-review surfaces fleet staleness.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **Deployment without monitoring decays.** The whole reason this fleet exists
  is that the *mechanism* shipped months of work ago and silently never reached
  production (live `[persona]` had no safety fields as of 2026-06-13). The same
  silence is the risk after deployment: a guarantee that isn't re-checked is a
  guarantee that erodes. This mirrors the recurring `fleet-binary-staleness`
  and `adopt-scan-stale-binaries` findings every self-review hand-walks.
- **The drift primitive already exists.** `wmd persona profile diff jocelyn`
  (`wintermute-brain/src/main.rs`, `diff_profile_vs_config` in
  `src/profile.rs`) reports exactly how the live `[persona]` differs from the
  jocelyn profile. The doctor wraps this into a pass/fail check with the
  intentional deltas (the chosen `self_name`/`wake_word`/`redline`) allowlisted,
  so only *unexpected* drift fails.
- **Redline-off is the dangerous drift.** Because `RedlineAction::Off` is the
  default (`src/redline.rs:39`), any config reset silently disables the guard
  while leaving everything looking superficially fine. The doctor must check
  `redline` is active specifically, not just that the table parses.
- **Aligns with the freshness discipline.** [[freshness]] is the existing
  vision for "the store's claims about the world go stale silently"; this is the
  persona-config analogue.

## What this builds

A new directory `~/wintermute/persona-deploy-doctor/` (shell), published as
`j0yen/persona-deploy-doctor`:

- **`doctor.sh`** — reads an expected-state file (written by
  `persona-deploy-jocelyn` at deploy time, or passed via flags:
  `--expect-self-name`, `--expect-wake-word`, `--expect-redline active`) and
  checks the live `brain.toml`:
  1. `wmd persona profile diff jocelyn` shows only the allowlisted intentional
     deltas — any other delta is drift.
  2. `forbidden_terms` is non-empty.
  3. `[persona].redline` is an active action (not `Off`).
  4. `self_name` equals the expected warm name (and is not `wintermute`).
  Exit 0 = healthy; exit 1 = drift, with a human-readable report of every
  failing check; exit 2 = could not read config / `wmd` missing.
  `--json` emits `{healthy, drift:[...], checked_at_unset}` for machine use.
- **`expected.example.toml`** — documents the expected-state file shape.
- **A systemd-user timer + service** (`persona-deploy-doctor.timer/.service`)
  that runs `doctor.sh --json` on a schedule (default daily) and, on drift,
  publishes a `wm.persona.drift` bus event via the existing agorabus CLI so the
  failure is visible to the fleet (and to a future herald/alert path). The
  installer does **not** auto-enable the timer — it prints the enable command
  (no surprise services, consistent with the homeward-orchestrate posture).
- **`README.md`** — what healthy looks like, how to read drift output, how to
  enable the timer.
- **`tests/`** — against temp fixture configs: a fully-deployed fixture passes;
  a fixture with `redline = Off` fails check 3; a fixture with `self_name =
  "wintermute"` fails check 4; an empty-`forbidden_terms` fixture fails check 2;
  a missing config exits 2. No live config touched.

Dependencies: `wmd` (≥ v0.22.0); `agorabus` CLI for the optional bus event
(degrade gracefully — skip the event with a warning if absent, still report
drift on stdout/exit code).

## Acceptance criteria

1. `doctor.sh --help` documents the expect flags, `--json`, and exit codes
   (0 healthy / 1 drift / 2 error).
2. Against a fully-deployed fixture (jocelyn profile applied, `self_name=Clara`,
   `redline` active), `doctor.sh --expect-self-name Clara --config <fixture>`
   exits 0.
3. A fixture with `[persona].redline = Off` (or absent) exits 1 and the report
   names the redline check.
4. A fixture with `self_name = "wintermute"` exits 1 and names the self_name
   check.
5. A fixture with empty `forbidden_terms` exits 1 and names that check.
6. A missing/unreadable config exits 2 (not 1), distinguishing error from
   drift.
7. `--json` emits valid JSON with a boolean `healthy` and a `drift` array
   listing each failed check by name.
8. The timer/service units install but are **not** auto-enabled; the installer
   prints the `systemctl --user enable --now` command.
9. `tests/run.sh` is green with no network and no writes outside the temp dir;
   the agorabus event path degrades to a warning when the CLI is absent.

## Out of scope

- Auto-*remediating* drift (re-running deploy) — doctor reports; a human or a
  future PRD decides whether to re-apply. Auto-restart of daemons is explicitly
  avoided (drops subscribers — see standing self-review caution).
- Watching anything beyond the `[persona]` section.
- The alert/notification transport itself (it publishes a bus event; delivery is
  a separate concern, cf. homeward-alert-delivery).
