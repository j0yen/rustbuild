# PRD: tide-window

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/tide
Vision: visions/tide.md

## TL;DR

Knowing a reboot is required (`tide survey`) and what it would fix (`tide
restart`) still leaves the hardest question unanswered: *is it safe to reboot
right now?* That depends on live state self-review already half-knows — active
Claude sessions, in-flight /build worktrees, supervised pevent jobs, the
agorabus fleet. Nothing assembles it into a decision. `tide window` is the
go/no-go advisor: it reads the live world, emits a `reboot-now` verdict with the
blocking reasons, and prints the **proposed** privileged plan for a human to
run. It never reboots anything and never runs `pacman -Su`.

## Why this exists

- self-review reports "needs a reboot window" every run (2026-06-06/-07/-08) but
  has no way to decide *which* window is safe, so it defers indefinitely and the
  queue grows (29 → 101 in two days, journal 2026-06-08). The decision inputs
  exist but are scattered.
- Live at dream time (journal 2026-06-08 snapshot): **3 concurrent Claude
  processes** — pid 33958 interactive, 402723 `/self-review`, 402724 `/dream` —
  plus heavy /build worktree activity (a 79-min session, 51k writes). Rebooting
  into that would kill in-flight work. The roster needed to *see* this is
  exactly what `muster census` renders (soft dependency, see below).
- The fleet that a reboot disrupts is concrete and observable: agorabus (14
  peers, `doctor=current`), the voice daemons, recalld. A safe window is "low
  fleet activity + no in-flight build + only the interactive session, ideally
  during the night-only /dream timer's quiet hours."
- This is the proposal-only sibling of `recourse-contest` / `mend-bridge` /
  `muster-reap`: it advises and prints a plan, the human pulls the trigger. The
  reboot stays user-gated per `mend.md:51`; tide just makes the gate *informed*.

## What this builds

A rust-extend into `~/wintermute/tide` (after `tide-survey` ships; preserve
survey + restart modules). Add:

### Modules

- `liveness.rs` — gather the go/no-go inputs, all read-only:
  - active Claude sessions: prefer `muster census --format json` if `muster` is
    on `PATH`; else fall back to `pgrep -af claude` and a minimal argv parse.
  - in-flight /build worktrees: presence of `.build-worktrees/*` with a live
    cargo/git process, or running `claude -p /build`.
  - supervised jobs: `pevent list` (running entries).
  - agorabus fleet: `agorabus peers` count + `agorabus doctor` status.
- `window.rs` — combine into `WindowVerdict { decision: Go | Hold, blockers:
  Vec<Blocker> }`. A `Hold` blocker is any of: >1 non-interactive Claude session
  active, an in-flight /build worktree, a running pevent job not on an allowlist,
  agorabus `doctor` not `current`. `Go` only when none fire. The interactive
  session running tide itself is never counted as a blocker.
- `plan.rs` — emit the proposed privileged command string **as text only**:
  the safe-apply step (`sudo pacman -Su`) and the reboot
  (`sudo systemctl reboot`), annotated with what survey/restart say each will
  resolve. Behind `--plan`; default output is the verdict + blockers only.
- `main.rs` (extend) — `tide window [--json] [--plan]`.

### Hard rules (proposal-only)

- tide-window **never** executes `pacman`, `systemctl reboot`, or any state
  change. It has no `--apply`/`--confirm`/`--arm` path. The only thing `--plan`
  does is *print* the command for a human to copy. This mirrors
  `muster-reap`'s discipline ([[gossip]] muster note) and `recourse-contest`.
- No timer. tide-window is invoked by hand or read by self-review's playbook; it
  is never wired to auto-run a reboot.

### UX

```
$ tide window
decision: hold
blockers:
  - 2 non-interactive claude sessions active (402723 /self-review, 402724 /dream)
  - in-flight /build worktree: aurora-foo
$ tide window --plan
decision: go
plan (run by hand — tide never executes this):
  sudo pacman -Su        # applies 97 safe + 4 reboot-requiring updates
  sudo systemctl reboot  # resolves: linux 7.0.9→7.0.11, glibc, systemd, mesa
```

### Dependencies

Reuses survey/restart deps. External tools invoked read-only via
`std::process::Command`: `muster` (optional), `pgrep`, `pevent`, `agorabus`.

## Acceptance criteria

1. `tide window` is a working subcommand after a clean rust-extend build;
   survey + restart subcommands and their tests still pass unchanged.
2. The verdict logic is unit-tested: injected live-state fixtures producing each
   blocker (extra non-interactive session, in-flight worktree, running pevent
   job, agorabus doctor ≠ current) yield `Hold` with the right blocker; an
   all-clear fixture yields `Go`. The interactive (self) session is excluded
   from the count in a dedicated test.
3. The muster integration is tested both ways: when `muster census --format
   json` output is present it is parsed for the session list; when muster is
   absent the `pgrep` fallback path is used. Both via injected command output
   (no live muster/pgrep in `cargo test`).
4. `tide window --plan` prints the `sudo pacman -Su` + `sudo systemctl reboot`
   strings annotated with survey's reboot-requiring set, and the test asserts
   the exact strings — proving the plan is *emitted*, not run.
5. There is **no** code path in the crate that executes `pacman`, `systemctl
   reboot`, or any reboot/power syscall. A test (or a documented grep gate in
   the README/CI note) asserts the binary contains no such invocation. The only
   process spawns are the read-only introspection tools.
6. `tide window --json` emits a stable schema (`decision`,
   `blockers[]{kind,detail}`, optional `plan{apply,reboot,resolves[]}`)
   validated by a serde round-trip; `tide window --json | head -1` does not
   SIGPIPE-panic.
7. README documents the proposal-only contract (never executes), the blocker
   set, the muster soft-dependency + pgrep fallback, and that the reboot stays
   human-gated per the tide vision.
