# PRD: memlog-udev-mode-repair — let the writer actually write

**Status:** Draft v0.1
**build_target:** mixed
**build_into:** /home/jsy/wintermute/wintermute-kernel/pkg
**Vision:** visions/continuity.md (Activation Fleet 2.0 — memlog capture)

## TL;DR

`/dev/memlog` ships mode **`0640 root:memlog`** — the `memlog` group can read
but **not write**. The PreCompact hook that snapshots session context into the
ring runs as `jsy` (a member of the `memlog` group) and therefore fails on
*every single compaction* with `[Errno 13] Permission denied`. The ring has
been empty for the entire life of the kernel tier. This PRD corrects the
one-digit skew (`0640` → `0660`) at its packaged source, ships a no-reboot
runtime override so the fix applies to the already-live device, and verifies
the fix with a real write→stats round-trip.

## Why this exists

Measured live on `7.0.11-arch1-1-wintermute`, 2026-06-16:

- `memlog stats` → `total_writes 0`, `records_in_ring 0`, on a 4-day uptime
  (device ctime 2026-06-12 22:52). The ring has been empty on every uptime
  since the device first appeared.
- The PreCompact writer is wired and fires: `~/.claude/settings.json`
  `PreCompact` → `~/.claude/scripts/memlog-precompact.sh`. Its log
  `~/.cache/memlog/precompact.log` records **36 firings**; the last 17+ are all
  `FAIL(1): memlog: [Errno 13] Permission denied: '/dev/memlog' — add yourself
  to the 'memlog' group or run as root.`
- `id` confirms `jsy` is in the `memlog` group, so membership is not the
  problem. `ls -l /dev/memlog` → `crw-r----- 1 root memlog` = **0640**: the
  group bit is `r--`, no write.
- The skew is three-way and the wrong source wins:
  - kernel driver `~/wintermute/memlog/driver/memlog.c:401` → `.mode = 0660`
    (`root:memlog rw`) — correct.
  - `~/wintermute/memlog/README.md:47` → documented rule `MODE="0660"` —
    correct.
  - **installed** `/usr/lib/udev/rules.d/70-linux-wintermute-memlog.rules`,
    sourced from `~/wintermute/wintermute-kernel/pkg/linux-wintermute-memlog.rules`
    → `MODE="0640"` — **wrong**, and it is what udev applies. Owned by
    `linux-wintermute 7.0.11.arch1-1` (`pacman -Qo` confirms).

The driver's own `devnode` callback would create the node `0660`, but the
packaged udev rule re-stamps `0640` after the fact and strips group-write. One
digit in one packaged file has kept the whole continuity memlog pipeline
(writer → `/dev/memlog` → witness → postmortem) inert since 2026-05-24, and no
probe ever flagged it because self-review read "ring empty" as *expected* run
after run (see `~/brain/journal/2026-06-15.md`: "Ring empty (total_writes=0) …
is expected").

## What this builds

A `mixed` (config + shell) change in the kernel package plus a runtime bridge,
all idempotent:

1. **Source fix (durable).** In `~/wintermute/wintermute-kernel/pkg/linux-wintermute-memlog.rules`,
   change `MODE="0640"` → `MODE="0660"` so the packaged rule matches the driver
   and README. Apply the same correction to the build-worktree copy
   `~/wintermute/wintermute-kernel/.build-worktrees/memlog-group-autojoin/linux-wintermute-memlog.rules`
   if present. Do **not** bump the running kernel or require a reinstall to
   benefit — see the runtime bridge.

2. **Runtime bridge (no reboot).** Ship a higher-priority drop-in
   `/etc/udev/rules.d/72-memlog-mode-fix.rules` containing
   `KERNEL=="memlog", GROUP="memlog", MODE="0660"` (rules in `/etc` override
   `/usr/lib`; the `72-` prefix sorts after the packaged `70-`). An install
   script (run via the build/install step, sudo per the pre-approved policy)
   writes the drop-in, then `udevadm control --reload-rules` and
   `udevadm trigger --name-match=memlog` so the live `/dev/memlog` node is
   re-stamped `0660` immediately. Idempotent: re-running is a no-op if the node
   is already `0660` and the drop-in already present.

3. **Verifier.** A `memlog-mode-verify` shell entrypoint that asserts, in order:
   (a) `/dev/memlog` exists and its mode is `0660` with group `memlog`;
   (b) the current user is in the `memlog` group;
   (c) a **real round-trip**: capture `total_writes` from `memlog stats`, pipe a
   small fixed test blob to `memlog write`, re-read `total_writes`, assert it
   incremented by exactly 1. Exit non-zero with a precise message naming the
   failing assertion. This is the proof that the fix is end-to-end, not just a
   permission bit cosmetically flipped.

4. **Uninstall.** Remove the `/etc/udev/rules.d/72-memlog-mode-fix.rules`
   drop-in and reload udev. The packaged-source correction stays (it is the
   right value); uninstall only retracts the runtime bridge.

### Shape / deps

- Pure shell + a udev rule file; no Rust crate. Install/uninstall/verify are
  three small scripts (or one script with subcommands `install|uninstall|verify`).
- Uses `sudo` for the `/etc/udev/rules.d/` write and `udevadm` reload (sudo is
  pre-approved on this box). Fails closed with a clear message if `udevadm` is
  absent.
- No wall-clock in any logic; the verifier is deterministic.

## Acceptance criteria

1. `~/wintermute/wintermute-kernel/pkg/linux-wintermute-memlog.rules` contains
   `MODE="0660"` and no longer contains `MODE="0640"`.
2. The build-worktree copy (if it exists at the path above) is likewise
   `MODE="0660"`; if it does not exist, the AC is satisfied vacuously and the
   install log says so.
3. After `install`, `/etc/udev/rules.d/72-memlog-mode-fix.rules` exists and
   contains exactly `KERNEL=="memlog", GROUP="memlog", MODE="0660"`.
4. After `install`, `stat -c '%a %G' /dev/memlog` reports `660 memlog` (the
   live device node was re-stamped without a reboot).
5. `memlog-mode-verify` exits 0 on a correctly-installed system and its
   round-trip assertion observes `total_writes` increment by exactly 1.
6. `memlog-mode-verify` exits non-zero with a message naming the specific
   failed assertion when run against a deliberately reverted `0640` node
   (test harness may restore `0640` via `sudo chmod 0640 /dev/memlog`, assert
   failure, then re-run `install` to restore).
7. `install` is idempotent: a second consecutive `install` makes no further
   change and exits 0 (verified by comparing `stat`/drop-in before and after).
8. `uninstall` removes the drop-in, reloads udev, and exits 0; the packaged
   `pkg/linux-wintermute-memlog.rules` is left at `0660` (uninstall does not
   re-introduce the bug).
9. The repo README documents that the packaged-source fix needs the next
   `linux-wintermute` pkgrel rebuild to become the *only* needed layer, and
   that the `/etc` drop-in is the no-reboot bridge until then.

## Out of scope

- The agentns all-zeros session-id bug (owned by `assay` / continuity Fleet
  1.9). The writer's `comm:claude:PID` fallback already produces a usable key;
  this PRD does not touch session identity.
- The witness daemon / postmortem join (continuity Fleet 1). This PRD only
  unblocks the *write* into the ring; consumers are separate.
- Any change to the kernel driver's `.mode` (already correct at `0660`).
