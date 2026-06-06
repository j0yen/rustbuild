# PRD: anchor-boot — the SessionStart re-watch hook the journal asked for

**Status:** Draft v0.1
**build_target:** mixed
**build_into:** `~/wintermute/anchor`
**Vision:** visions/anchor.md
**deferred_acs:** [5, 6]

## TL;DR

anchor-reconcile gives us an idempotent `anchor reconcile --apply`; this PRD
*wires it into the laptop's lifecycle* so the watch roots are restored without
anyone thinking about it. **anchor-boot** ships a systemd-user boot oneshot
(`anchor-reconcile.service`, ordered After watchman.service) and a SessionStart
hook, both running `anchor reconcile --apply`. This is the exact fix the
2026-06-03 journal proposed — *"Worth a SessionStart re-watch hook if it
recurs"* — finally built, so a reboot or a fresh session never again costs a
hand-re-watch or a silently-zeroed weekly delta.

## Why this exists

- **The journal already specified the fix.** 06-03 Notable: *"watchman drops
  watched roots across a reboot … Worth a SessionStart re-watch hook if it
  recurs."* `recall:01KT6VCBX1PSWT9MAQP607TAWY`. It recurs (06-01/02/03 all
  re-watch by hand); this is that hook.
- **There is no such hook today.** Of the 8 SessionStart hooks wired in
  `~/.claude/settings.json` (observed 2026-06-05: check-review-due,
  ctrace-session-start, recall-session-start, scratch-tools-start,
  claude-self-start, agorabus-session-start, learning-candidates-start, peon),
  **none** touch watchman or wchg. The gap is structural, not a misconfiguration.
- **Boot is the primary loss point; session start is the backstop.**
  watchman.service is socket-activated and forgets its roots across a reboot
  (confirmed live). A boot oneshot ordered After it covers the reboot case; the
  SessionStart hook covers the mid-session/between-runs loss
  (`recall:01KSGR03…`, `01KSNM4CP4…`). Together they close both observed loss
  windows.
- **anchor-reconcile shipped the idempotent `--apply`** — safe to fire from a
  oneshot and a hook that may overlap. This PRD adds only the unit + hook +
  install glue; no new Rust logic.

## What this builds

Extends `~/wintermute/anchor` with a `boot/` directory and an installer; does
**not** auto-edit live system files (install is a user-run, idempotent step,
per the boot-handshake / timer-enable user-gate precedents).

- **`boot/anchor-reconcile.service`** — a `Type=oneshot` systemd-user unit:
  `ExecStart=%h/.local/bin/anchor reconcile --apply`, `After=watchman.service`,
  `Wants=watchman.service`, `WantedBy=default.target` (so it runs once at login
  ramp-up, after watchman's socket is up). A companion
  `anchor-reconcile.timer` is **optional and not enabled by default** (the
  SessionStart hook already gives per-session coverage; a periodic timer is left
  as a documented opt-in to avoid an always-on background tick the user didn't
  ask for).
- **`boot/anchor-session-start.sh`** — a SessionStart hook script that runs
  `anchor reconcile --apply` (quietly; prints a one-line summary only if it
  actually re-asserted a root, so a healthy session stays silent — matches the
  low-noise posture of the existing hooks). Exits 0 always (a watch failure must
  never block a session from starting).
- **`boot/install.sh`** — idempotent installer: symlinks/copies the `.service`
  into `~/.config/systemd/user/`, `daemon-reload`, `enable` the service, and
  prints the exact `settings.json` `SessionStart` hook entry to add (it
  **prints** the settings edit rather than rewriting `settings.json`
  unprompted — settings edits are user-gated, `feedback_classifier_per_command`).
- **README "Install" section** documenting the two-line manual step and how to
  back it out (`systemctl --user disable --now anchor-reconcile.service`, remove
  the hook line).
- The live ACs (the unit actually running at boot and re-establishing a watch;
  the hook firing on a real session) are **deferred** — they need this laptop
  with systemd-user + watchman + a real reboot/session, which the cloud build
  box cannot provide. The unit-file validity, the installer's idempotence, and
  the hook script's exit-0-on-failure behavior are all verified offline.

## Acceptance criteria

1. `boot/anchor-reconcile.service` is a valid systemd unit:
   `systemd-analyze verify boot/anchor-reconcile.service` passes (run in CI/
   offline; it does not require the service to run), and the unit declares
   `After=` and `Wants=watchman.service` and `ExecStart` invoking `anchor
   reconcile --apply`.
2. `boot/install.sh` is idempotent: running it twice leaves exactly one unit
   symlink and one enabled state; a test (shell `bats`-style or a Rust
   integration test shelling the script against a temp `XDG_CONFIG_HOME`)
   asserts the second run makes no further change and exits 0.
3. `boot/install.sh` **prints** the `settings.json` SessionStart hook entry to
   add and does **not** modify any file under `~/.claude/` (asserted: the script
   touches only `$XDG_CONFIG_HOME/systemd/user/`).
4. `boot/anchor-session-start.sh` exits `0` even when `anchor reconcile --apply`
   returns non-zero (a watch failure must not block session start); covered by a
   test that stubs `anchor` with a failing shim and asserts exit 0.
5. *(deferred — needs a real reboot on the laptop)* After enabling the service
   and rebooting, `watchman watch-list` contains every declared root without any
   manual `watchman watch` call. User-verified on the laptop.
6. *(deferred — needs a live Claude session)* Opening a new Claude session with
   the hook installed re-asserts a manually-removed declared root within that
   session (verified by `anchor probe` returning all-`Ok` afterward). User-
   verified on the laptop.
7. README documents install, the boot-vs-session coverage split, the optional
   timer, and back-out.
8. The installer/hook test entry file appears in the test runner's output
   (`self_orphaned_mock_tests` guard — no silently-skipped test subdir).
