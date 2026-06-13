# PRD: persona-work-doctor — the work identity should not silently drift

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/persona-work
Vision: visions/persona.md

## TL;DR

Once the work persona is installed and its guard is wired, nothing keeps it
that way. `~/.claude/CLAUDE_SELF.md` is a hand-editable symlink, the
`PreToolUse` hook can be dropped from `settings.json`, and the work
`redline.toml` can be deleted or weakened — silently, leaving the box looking
fine while it quietly accepts a `j0yen` push again. This PRD is a periodic
health check that asserts the deployed work persona still matches what Joe
deployed and surfaces drift loudly. It mirrors `persona-deploy-doctor`, which
does the same for the elder persona, and composes two primitives that already
exist on this box.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **Deployment without monitoring decays — proven this week.** The entire
  persona deployment frontier exists because shipped mechanism silently never
  reached production (the live `[persona]` block had no safety fields as of
  2026-06-13 despite the code shipping). The same silence threatens the work
  persona after install. `persona-deploy-doctor` (build_target shell, vision
  persona.md) was drafted for exactly this on the elder side; the work side has
  no equivalent.
- **This is the recurring self-review pattern.** Every self-review hand-walks
  `fleet-binary-staleness` and `adopt-scan-stale-binaries` — "a thing drifted
  from its intended state and only a manual check noticed." The work persona is
  another such thing; a doctor turns a silent erosion into a flagged finding.
- **Both checks already exist as shipped tools.** `answerable values-drift`
  (v0.4.0) compares `CLAUDE_SELF.md` Values + Boundaries against a git baseline
  and reports REMOVED / WEAKENED / SECTION-REMOVED with exit 1 on drift —
  precisely the "did someone soften 'Never commit to j0yen' to 'usually'" check.
  `persona-work/validate.sh` (commit 558f961) already exits 0/1/2 for
  work/personal/absent identity. The doctor composes the two; it invents no new
  detection logic.

## What this builds

Extends `~/wintermute/persona-work/` (published as `j0yen/persona-work`):

- **`doctor.sh`** — a single drift check, exit 0 healthy / 1 drift / 2 cannot
  determine, with a human-readable report of every failing check:
  1. **Identity installed.** `validate.sh` reports the `work` identity (exit 0).
     Otherwise → drift (the work box is running the wrong self file).
  2. **Boundaries intact.** `answerable values-drift --json` against the
     persona-work baseline (the committed `CLAUDE_WORK.md`) reports no REMOVED
     or WEAKENED bullet in Values/Boundaries/Autonomy-scope — a softened
     "Never"→"usually" or a dropped "no force push" fails.
  3. **Guard present.** The work `redline.toml`
     (`$XDG_CONFIG_HOME/answerable/redline-work.toml`) exists and parses
     (`answerable check --policy … --action noop` not exit 3).
  4. **Hook wired.** The `pretooluse-work-guard.sh` hook is registered in
     `~/.claude/settings.json` `PreToolUse`. A missing entry = the gate is
     installed but unwired = silent drift, the work-side analogue of
     `RedlineAction::Off`.
- **`doctor-install.sh`** — optional: registers `doctor.sh` as a systemd-user
  timer on the work box (daily), writing a one-line result to the journal the
  way self-review does. Idempotent; reversible. Timer install is gated on the
  work identity being present.

### Acceptance criteria

1. `doctor.sh` exits 0 when all four checks pass against a fixture that has the
   work identity installed, a valid `redline-work.toml`, and the hook wired.
2. `doctor.sh` exits 1 and names the failing check for each of: wrong identity
   installed; a Values/Boundaries bullet REMOVED or WEAKENED vs baseline;
   `redline-work.toml` absent/malformed; hook missing from `settings.json`.
   Each failure mode covered by a fixture test.
3. The Boundaries check uses `answerable values-drift` (not a bespoke grep) and
   correctly flags a WEAKENED bullet (e.g. "Never commit to j0yen" →
   "Avoid committing to j0yen when possible") as drift.
4. `doctor.sh` exits 2 (cannot determine) — never a false 0 — when `answerable`
   or `validate.sh` is missing, or `settings.json` is unreadable.
5. `doctor-install.sh` is idempotent (one timer after two runs), reversible
   (uninstall path), and refuses to install the timer unless the work identity
   is the installed one.
6. README documents the four checks, the exit-code contract, and how a failing
   doctor maps back to the remediation (re-run `install.sh` /
   `install-guard.sh`).

## Out of scope

- Auto-remediation — the doctor reports and exits non-zero; re-installing is a
  human-invoked `install.sh` / `install-guard.sh` per the report.
- Producing the block-rate number — that is persona-work-eval.
- Any drift check for the elder persona — that is `persona-deploy-doctor`.
