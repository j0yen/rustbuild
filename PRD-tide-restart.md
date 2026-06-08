# PRD: tide-restart

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/tide
Vision: visions/tide.md

## TL;DR

`tide survey` says a reboot is required, but not *why a reboot vs a restart*.
After a routine library upgrade, many running processes can be refreshed with a
plain `systemctl restart` — only kernel/glibc-class changes truly need a reboot.
On Arch the tool that answers this is `needrestart`, and it is **absent** on
this laptop (verified `command -v needrestart` → nothing, 2026-06-08). `tide
restart` is the missing `needrestart`-equivalent: it scans running processes for
`(deleted)` shared-library mappings, attributes each to a systemd unit / fleet
daemon, and partitions them into *restart-fixable* vs *reboot-only*.

## Why this exists

- `needrestart`, `checkupdates`, and `reflector` are all absent (verified live
  this pass). The laptop has **zero** introspection for "what is running stale
  bytes after an upgrade" — the exact question that decides whether the 101-deep
  queue (journal 2026-06-08) needs a full reboot or can be largely drained with
  targeted restarts.
- The wintermute fleet is long-lived and upgrade-sensitive: `agorabus` daemon
  (14 peers, `doctor=current`), the voice daemons `wm-stt`/`wm-tts`/`wm-dialog`/
  `wm-brain` (pids 636–639 this session), and `recalld` (liveness is
  safety-critical per [[project_brain_local_first_ladder]]). After a shared-lib
  upgrade these keep mapping the **old** inode until restarted — `vigil` already
  exists to catch *stale-version daemons*, but nothing tells you a routine
  upgrade just made the live fleet stale, or which units a restart would fix.
- `survey` (the prior PRD) classifies the *queue*; `restart` classifies the
  *running world* — the complement that turns "reboot-required" into "these 6
  units need restart, only the kernel needs the reboot."

## What this builds

A rust-extend into `~/wintermute/tide` (do not start until `tide-survey` has
shipped and the repo exists, or extend-validate fails — same rule that bit relay
and concord). Preserve survey's modules; add:

### Modules

- `maps.rs` — for each readable `/proc/<pid>`, scan `maps` for mappings whose
  pathname ends in `(deleted)` or points at a path whose on-disk inode no longer
  matches (an upgraded `.so`). Collect `StalePid { pid, comm, deleted_libs:
  Vec<String> }`. Skip kernel threads and unreadable pids gracefully.
- `attribute.rs` — map each stale pid to a systemd unit via
  `/proc/<pid>/cgroup` (the `…/system.slice/<unit>` or `…/user@.../<unit>` leaf)
  and tag known fleet daemons (`agorabus`, `wm-stt`/`tts`/`dialog`/`brain`,
  `recalld`) by `comm`/argv. Produce `RestartTarget { unit, pids, fleet_role:
  Option<FleetRole>, fixable_by: Restart | RebootOnly }`. A target is
  `RebootOnly` when any deleted lib is kernel/glibc-class (reuse survey's
  `RebootClass` set against the lib's owning package via `pacman -Qo`).
- `render.rs` (extend) — a `restart` view + `--json`.
- `main.rs` (extend) — `tide restart [--json]`.

### UX

```
$ tide restart
3 unit(s) running upgraded libraries:
  restart-fixable:
    agorabus.service        wm-brain (639)   libssl.so.3 (deleted)
    wm-stt.service          wm-stt (637)     libonnxruntime.so (deleted)
  reboot-only:
    (none — no kernel/glibc-class deletions in running set)
$ tide restart --json | jq '.targets[] | select(.fixable_by=="Restart").unit'
```

### Dependencies

Reuses survey's deps; no new crates. `/proc` read via `std::fs`; `pacman -Qo`
via `std::process::Command` (read-only).

## Acceptance criteria

1. `tide restart` is a working subcommand after a clean rust-extend build;
   `cargo install --path .` refreshes the binary; survey's subcommand and tests
   still pass unchanged (extend, not rewrite).
2. The `/proc/<pid>/maps` parser is unit-tested against a **fixture** maps file
   containing both normal mappings and `(deleted)` library lines, and extracts
   exactly the deleted library paths.
3. Cgroup→unit attribution is unit-tested against fixture `/proc/<pid>/cgroup`
   contents for a `system.slice/agorabus.service` case and a `user@1000.service`
   case, yielding the correct unit leaf.
4. A stale pid whose deleted lib is owned by a kernel/glibc-class package
   classifies `RebootOnly`; one owned by an ordinary package classifies
   `Restart` — both asserted with the `pacman -Qo` lookup injected/faked in
   tests (no live pacman in `cargo test`).
5. Known fleet daemons (`agorabus`, `wm-stt`/`tts`/`dialog`/`brain`, `recalld`)
   are tagged with a `FleetRole`; unknown units carry `fleet_role: null`.
6. Unreadable or vanished pids (race during scan) are skipped without aborting
   the whole scan; a permission-denied `/proc/<pid>/maps` does not panic.
7. `tide restart --json` emits a stable schema
   (`targets[]{unit,pids,fleet_role,fixable_by,deleted_libs}`) validated by a
   serde round-trip test, and `tide restart --json | head -1` does not SIGPIPE-
   panic.
8. README gains a `tide restart` section explaining the `(deleted)`-mapping
   scan, the restart-vs-reboot rule, and that it never restarts anything itself
   (report-only).
