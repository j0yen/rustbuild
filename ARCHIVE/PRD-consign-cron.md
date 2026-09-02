# PRD: consign-cron — drain push-debt on a timer

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/consign
deferred_acs: [6]
Vision: visions/consign.md

## TL;DR

consign-drain pushes eligible repos when invoked, but a manual tool that the
human has to remember to run is exactly how the fleet accumulated 14+ repos of
push-debt in the first place. consign-cron wires `consign drain --no-dry-run`
to a systemd-user timer on the same 6h cadence as `adopt-cron`, drains within
guardrails, appends a one-line summary to the docket/journal, and stays silent
on a clean pass.

## Why this exists

- Live evidence: `adopt-cron.{service,timer}` already exists at
  `~/.config/systemd/user/` and drains artifact-adoption debt every 6h. Git
  push-debt has no equivalent timer, so it only shrinks when a human runs
  `git push` by hand — which the self-review record (8 unpushed, every day,
  2026-06-14..18) shows does not happen reliably.
- The autonomous-but-guardrailed cadence is the established pattern on this box
  (`adopt-cron`, `claude-dream.timer`, `claude-self-review.timer`,
  `cloudbuild-watchdog.timer`). consign-cron joins it rather than inventing a
  new mechanism.
- Memory `feedback_always_commit_push` (always auto-publish/commit/push, no
  caps) is the standing user instruction this directly serves: the timer is the
  mechanism that makes "always push" actually true for the whole fleet.

## What this builds

- `consign-drain.service` + `consign-drain.timer` under
  `~/.config/systemd/user/`, modelled on `adopt-cron.*`:
  - `OnCalendar`/`OnUnitActiveSec` ≈ every 6h (match adopt-cron's cadence).
  - `Type=oneshot`, runs `consign drain --no-dry-run --format json`.
  - Honors the consign-policy gate (private-hold/manual-only never pushed).
- A thin wrapper script (`consign-cron.sh`, installed alongside the unit) that:
  - Runs `consign drain --no-dry-run`, captures the receipt.
  - Appends a one-line summary to the journal/docket only when it pushed
    something or hit an error (`consign-cron: pushed N repos, M errors, K
    needs-human`); emits nothing on a zero-debt pass.
  - Never force-pushes (inherits drain's guarantee); exits non-zero only on an
    actual push error, so a failed unit is a real signal.
- An installer step (`install.sh` or documented `systemctl --user enable
  --now consign-drain.timer`) and a matching disable instruction.

## Acceptance criteria

1. `consign-drain.service` and `consign-drain.timer` exist under
   `~/.config/systemd/user/`, parse cleanly (`systemd-analyze verify` or
   `systemctl --user cat`), and the timer schedule is ~6h matching adopt-cron.
2. The service runs `consign drain --no-dry-run` (verifiable in `ExecStart`),
   not the dry-run default.
3. The wrapper appends a one-line summary to the journal/docket **only** when
   repos were pushed or errors occurred; a clean pass writes nothing.
4. `systemctl --user enable --now consign-drain.timer` activates it and
   `systemctl --user list-timers` shows it scheduled; a documented disable
   command deactivates it.
5. The unit/wrapper inherits drain's no-force guarantee — nothing in the cron
   path adds `--force`.
6. A dry verification run (timer fired once manually via
   `systemctl --user start consign-drain.service`) completes with exit 0 on a
   clean fleet and produces a receipt artifact. (May be deferred/hardware-gated
   if the build host lacks the live fleet.)
7. Install/enable/disable steps are documented in the consign README; the unit
   files are committed into `~/wintermute/consign`.
