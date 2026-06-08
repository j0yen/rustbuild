# Vision: tide — managing the reboot crossing

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. The user gave no steer, so
this pass dreamed from the laptop's strongest *unaddressed* recurring signal:
the pacman update queue that self-review reports as "BLOCKED — needs a reboot
window" run after run after run, and never acts on.

## TL;DR

Every self-review for the last three days (2026-06-06, -07, -08) has reported
the same line under "Pending your call":

> **pacman 101 updates BLOCKED** — `linux 7.0.9→7.0.11`, `linux-firmware*`
> (×12), `glibc`/`systemd`/`mesa` protected. 81 safe updates cannot be
> selectively applied — requires a reboot window.

The queue is **growing**: 29 packages (2026-06-06) → 101 (2026-06-08), +72 in
two days. self-review flags it and declines to act because applying it needs a
reboot, and a reboot is a human-gated decision. `mend.md:51` makes that explicit
— `pacman-kernel-update-blocked` is tagged *"user-gated reboot, not buildable."*

That tag is right about the *reboot* and wrong about *everything around it*. The
laptop has **no tooling for the update→reboot→verify lifecycle at all**.
Verified live this pass (2026-06-08): `checkupdates`, `needrestart`, and
`reflector` are all **absent** (`command -v` → nothing). So:

1. **Nothing honestly enumerates the queue.** self-review hand-counts it. The
   booted kernel is `7.0.10-arch1-5-wintermute` (a custom build), the `linux`
   package is `7.0.9.arch1-1`, and the queue wants `7.0.11` — a three-way
   version skew that no tool renders, so "what reboot would actually change" is
   guesswork written by hand each tick.
2. **Nothing tells you what a reboot would even fix.** After a routine upgrade,
   which *running* processes and fleet daemons are mapping deleted/upgraded
   shared libs? Which need only a `systemctl restart` vs a true kernel reboot?
   `needrestart` answers this on other distros; here it doesn't exist.
3. **Nothing helps choose the window.** "Safe to reboot now?" depends on live
   state self-review already half-knows: active Claude sessions, in-flight
   /build worktrees, pevent jobs, the agorabus fleet. Nothing assembles that
   into a go/no-go.
4. **Nothing verifies the crossing.** After a reboot the fleet has to come back
   — agorabus (14 peers, `doctor=current`), the voice daemons (wm-stt/tts/
   dialog/brain), recalld liveness, watchman roots (which `anchor` notes are
   *silently dropped* on every reboot). self-review re-discovers brokenness ad
   hoc; nothing attests "the crossing succeeded, here's before/after."

`tide` is the harbormaster for the reboot boundary. It does **not** reboot the
machine (that stays human-gated, per `mend.md:51`). It makes the crossing
**legible, advised, and verified**: survey the rising queue honestly, name what
a reboot would fix, propose a safe window, and attest the fleet returned whole.
Same proposal-first, never-silently-mutate, evidence-rich ethos as
`recourse-contest` / `mend-bridge` / `muster-reap` — the action half is always
print-only for anything privileged; the reboot itself is never automated.

## End-state

When tide is fully built:

- `tide survey` reports the real update/reboot state with a single verdict —
  `current | updates-pending | reboot-pending | reboot-required` — backed by
  evidence: the pacman queue partitioned into *reboot-requiring*
  (linux/linux-firmware/glibc/systemd/mesa/linux-api-headers) vs
  *safe-to-apply-live*, plus the booted-vs-installed-vs-available kernel skew.
  `--json` for machines. This is the honest replacement for self-review's
  hand-written "101 BLOCKED" line.
- `tide restart` is a `needrestart`-equivalent: it scans `/proc/*/maps` for
  `(deleted)` library mappings, cross-references the wintermute fleet (agorabus
  daemon, wm-* voice daemons, recalld), and reports which running units a
  reboot-free `systemctl restart` would refresh vs what genuinely needs a kernel
  reboot. Read-only; proposal-first.
- `tide window` reads the live fleet (active Claude sessions, in-flight /build
  worktrees, pevent jobs, agorabus peers) and emits a go/no-go for rebooting now
  plus the exact privileged plan it is *proposing* (`sudo pacman -Su &&
  systemctl reboot`) — print-only, never executed. Mirrors `muster-reap`'s hard
  refusal to act on live work.
- `tide landfall` is the post-reboot verifier: a SessionStart/boot probe that
  confirms the kernel advanced as `survey` predicted, the fleet came back
  (agorabus `doctor=current` + expected peers, voice daemons live, recalld
  responsive), and watchman roots re-asserted — then writes a timestamped
  **crossing receipt** with before/after. If the fleet didn't return healthy, it
  says so *loudly*. This closes the loop self-review keeps leaving open.
- tide is wired into self-review as a deterministic Phase-B.5 playbook, so the
  "needs reboot window" finding is surfaced structurally with the queue
  partition and go/no-go pre-filled, instead of hand-written every tick.

## Components (one bullet per future PRD)

- **tide-survey** (new repo `~/wintermute/tide`, rust-cli): the foundation —
  workspace + `tide` binary, the `UpdateState`/`Verdict`/`Evidence` types, a
  read-only pacman-state reader (sync to a private dbpath the `checkupdates`
  way, never touching the system db), the reboot-requiring vs safe-to-live
  partition, and the booted-vs-installed-vs-available kernel skew. Pure read,
  fixture-driven parser tests, zero network at test → cloud-build-safe. Ships
  first; creates the repo + binary other PRDs extend.
- **tide-restart** (rust-extend → tide): the `needrestart`-equivalent. Scan
  `/proc/<pid>/maps` for `(deleted)` mappings of upgraded libs, attribute each
  PID to a unit/fleet daemon, partition into restart-fixable vs reboot-only.
  Pure read; report + `--json`.
- **tide-window** (rust-extend → tide): the go/no-go window advisor. Reads live
  fleet state (active sessions, /build worktrees, pevent jobs, agorabus peers),
  emits a reboot-now verdict + the proposed privileged plan, **print-only**.
  HARD refusal to ever execute a reboot or `pacman -Su`; `--plan` emits the
  command for a human to run.
- **tide-landfall** (rust-extend → tide): the post-reboot verifier + crossing
  receipt. Consumes `survey`'s pre-reboot expected-state, confirms the kernel
  advanced + the fleet returned + watchman re-asserted, writes a before/after
  receipt, escalates loudly on a failed crossing.

## Order

`tide-survey` **first** — it creates the repo, the `tide` binary, the core
types, and the pacman-state reader. Do not start any rust-extend PRD until
survey has shipped and `~/wintermute/tide` exists, or extend-validate fails
(the same rule that bit relay and concord). Then:

```
tide-survey → { tide-restart, tide-window } → tide-landfall
```

`restart` and `window` are independent of each other (both extend survey).
`landfall` depends on survey's expected-state type for its before/after diff;
it can also read window's verdict but doesn't require it.

## Open questions (for the next pass / the user)

- **checkupdates-without-checkupdates.** The canonical safe-read pattern syncs
  the pacman db to a private `--dbpath` and diffs `-Qu`. tide-survey should ship
  that as a built-in (the laptop has no `checkupdates`), but it needs a writable
  cache dir and a `pacman -Sy` against it. Confirm that's acceptable as the
  read path, or fall back to parsing `pacman -Sup` output only.
- **landfall trigger.** SessionStart hook vs a boot-time systemd-user oneshot vs
  both? A boot oneshot catches the crossing even if no Claude session starts;
  SessionStart catches it for the interactive case. Leaning both, with the
  receipt deduped by boot-id.
- **window ↔ muster.** tide-window's "active Claude sessions" input is exactly
  `muster census`'s output. Soft dependency: window should consume muster's
  roster if present and fall back to `pgrep` if not. Coordinate so window
  doesn't re-implement the census.
- **anchor handoff.** landfall's "watchman roots re-asserted" check overlaps
  `anchor`'s reconciler. landfall should *verify* anchor did its job, not
  re-watch — keep the reconcile in anchor, the attestation in tide.
