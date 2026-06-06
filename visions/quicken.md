# Vision: quicken — a built primitive that never came alive isn't built

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-05
**Updated:** 2026-06-06 (`/dream extend quicken` — Fleet 2 drafted: the
boot/bus-reactive half. `quicken-watch` + `quicken-notify` move from
open-questions to drafted PRDs, now that this pass re-confirmed the dark
set live *and* surfaced the real-time gap: self-review is daily, but
primitives die mid-day.)
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. The user declined a
steer, so this pass dreamed from the laptop's strongest *unaddressed*
signal: kernel primitives that are compiled, packaged, even installed —
and yet runtime-**inert**, reported identically in self-review after
self-review, escalated "once" then left to rot across reboots.

## TL;DR

`vigil` catches a daemon running **stale bytes** (built, installed,
running the *wrong* version). `freshness` catches stale memory bodies;
`drift` catches stale skill text. None of them catch the failure mode
that bites this laptop every single self-review: a primitive that is
built and installed but **never came alive at all** — not running the
wrong bytes, running *no* bytes. The kernel ships the surface; nothing
ever activated it; nothing notices it stayed dark.

Caught live this pass (verbatim probes, 2026-06-05):

- **memlog**: `/dev/memlog` exists (`crw-rw---- root:root`), but the
  user is **not in the `memlog` group** → every write is EACCES.
  Installed `linux-wintermute` is `pkgrel-5`; the sysusers+udev fix is
  built and sitting at `pkgrel-11` — **uninstalled**. The gap has
  *widened* across reviews (5→10→11). The fix has existed for days.
- **agentns**: `/proc/self/agent_session` = all-zeros despite live
  Claude PIDs → `CLONE_NEWAGENT` registration never took.
- **warden/bpolicy**: `{"loaded": false}` → never armed this boot; no
  writes audited or enforced.
- **provfs**: live — but **degraded**. Its xattr reads
  `user.prov.session="comm:zsh:pid:…:uid:1000"` (the *fallback* form),
  **not** the 128-bit agentns session id it's designed to stamp.
  provfs can't record rich provenance *because agentns is inert.*
  One dark primitive silently degrades a live one.

The self-review reports each of these, escalates "once," then goes
silent — so they persist boot after boot. There is no probe that says
"primitive X is supposed to be live and is not, here is the one command
that fixes it, and I will say so *louder* every boot until it is."

`quicken` builds that missing layer: a read-only **liveness
attestation** for every wintermute kernel/userspace primitive, the
**exact remediation** for each dark one, a **persisted receipt** with a
monotonic streak counter that escalates instead of forgetting, and a
**cross-dependency** model so "provfs is degraded *because* agentns is
dark" is stated, not left for a human to infer.

Same evidence-rich, proposal-first, never-silently-mutate ethos as
`vigil`/`freshness`/`drift` — the read half is pure; the action half
defaults to print-only and only ever auto-applies the safe userspace
subset (group/udev), never a kernel install or `sudo` without opt-in.

## End-state

When quicken is fully built:

- `quicken probe` reports every registered primitive with a verdict —
  `live | live-degraded | staged-not-installed | installed-not-activated
  | inert | unknown` — plus the concrete evidence (dev-node perms +
  group, `/proc/self/agent_session` bytes, bpolicy status JSON, provfs
  xattr form, installed-vs-available pkgrel). `--json` for machines.
- `quicken remedy` prints the *exact* command to revive each dark
  primitive (e.g. `sudo pacman -U …pkgrel-11…` + reboot, or the
  no-reboot `systemd-sysusers && udevadm trigger && newgrp memlog`
  path). `--apply` opts into the **safe userspace subset only**;
  kernel/sudo steps are always print-only. `--dry-run` is the default
  posture, mirroring `rollout`.
- `quicken attest` writes a timestamped liveness receipt and a delta vs
  the last one ("memlog gap widened 5→11", "agentns inert for 7
  consecutive boots"). The streak counter makes a rotting primitive get
  *louder*, not quieter — the antidote to escalate-once-then-silent.
- `quicken` is wired into self-review as a deterministic Phase-B.5
  playbook, so the inert-primitive findings are surfaced structurally
  with the remediation pre-filled, instead of hand-written every tick.
- Cross-dependencies are explicit: a degraded `live` primitive names
  the dark primitive that degrades it and what fixing it would upgrade.

## Components (one bullet per future PRD)

- **quicken-probe** (new repo `~/wintermute/quicken`, rust-cli): the
  foundation — workspace + `quicken` binary, the `Primitive`/`Verdict`/
  `Evidence` types, a `Probe` trait, and probes for memlog, agentns,
  bpolicy/warden, provfs. Pure read (proc/dev/xattr/pacman-query),
  fixture-driven tests, zero network → cloud-build-safe. Ships first.
- **quicken-remedy** (rust-extend → quicken): per-verdict remediation
  command emission; `--apply` for the safe userspace subset only,
  print-only for kernel/sudo; `--dry-run` default. Deterministic
  (asserts command strings against fixtures).
- **quicken-attest** (rust-extend → quicken): persisted liveness
  receipt + delta vs prior + a monotonic per-primitive inert-streak
  counter that escalates over consecutive dark boots.
- **quicken-crossdep** (rust-extend → quicken): a small primitive→
  primitive enablement DAG; annotates each verdict with `blocked-by`
  and `would-upgrade` (the provfs↔agentns degradation chain caught live
  this pass is the canonical case).

## Order

`quicken-probe` **first** (creates the repo + binary + core types +
`Probe` trait; do not start any rust-extend until it has shipped and
the repo exists, or extend-validate fails — same rule that bit relay
and concord). Then `remedy`, `attest`, `crossdep` are all rust-extend
into `~/wintermute/quicken` and may build in any order after probe
lands. `crossdep` reads cleanest after `probe`'s verdicts exist;
`attest` is independent of both `remedy` and `crossdep`.

## Fleet 2 — the boot/bus-reactive half (drafted 2026-06-06)

Fleet 1 (`probe`/`remedy`/`attest`/`crossdep`) makes the dark set
*knowable on demand* — but only when something runs `quicken probe`,
which in practice is the **daily** self-review tick. That leaves a
real-time hole. A primitive can be live at 09:00 and dark by 14:00 and
nothing notices until tomorrow's review. This is not hypothetical: the
`agorabus-restart-kills-voice` incident (memory
`self_agorabus_restart_kills_voice`) is exactly this shape — the
`wm-{audio,stt,tts}` daemons exited on a bus-close and **stayed dead**
because nothing watched the transition; it was caught manually, not by a
probe. Fleet 2 closes the hole by making liveness an *event*, not a poll.

- **quicken-watch** (rust-extend → quicken): a oneshot (`quicken watch
  --once`) wired to a systemd-user unit (boot + a low-frequency timer)
  that runs the probe set and **publishes each verdict to agorabus** on
  the existing `wm.health.*` envelope (`wm.health.primitive.<name>` —
  verdict, evidence digest, inert-streak, `blocked_by`). REUSES the
  health envelope already produced by `wintermute-brain/degrade.rs` and
  consumed by `docket/digest.rs`; does **not** mint a parallel
  `wm.quicken.*` topic (the `wm.quicken.*` idea in the prior open
  question is superseded — composing with `wm.health.*` is why
  `docket-digest` can pick it up for free). Pure publish; no enforcement.

- **quicken-notify** (rust-extend → quicken): the report-side consumer.
  `agorabus subscribe wm.health.primitive.` (auto-reconnect, already in
  the CLI) and fire **one** signal on a *transition* — live→dark, or an
  inert-streak crossing a threshold — debounced so a steadily-dark
  primitive doesn't spam every boot. Output is a surfaced line
  (SessionStart banner fragment + optional peon-ping), never a heal.
  This is deliberately the **report** end of the report-vs-heal split:
  it tells you the moment voice went dark; it does not restart it
  (`homestead` owns any future unattended self-heal — see boundary).

### Open questions (still held — not yet motivated enough to draft)

- **Self-heal vs. report**: should `quicken` ever auto-install the
  staged kernel pkg (it's user-protected, needs reboot), or stay
  strictly report+userspace-only forever? Leaning report-only; revisit
  if `homestead` wants an unattended path for jsy's mother's device.
  Fleet 2 keeps the line firm: `watch` publishes, `notify` surfaces,
  neither acts.
- **agentns root cause**: the all-zeros registration is kernel-side
  (no userspace playbook). quicken can *detect* and *report* it (and
  Fleet 2 can now alert the moment provfs degrades because of it), but
  *reviving* it may need a `wintermute-kernel` patch — a separate vision
  thread (`agentns` repo), not a quicken PRD. Honest frontier: the next
  step there is an **instrumentation** PRD that proves *where* the zero
  comes from (launcher not wrapping vs. kernel hook returning zero vs.
  `/proc` read bug) before anyone proposes a fix —
  see `feedback_verify_before_concluding`.
- **Boundary vs. `wintermute_watchdog`**: `wintermute-platform` already
  ships a `wintermute_watchdog` binary that watches *daemon* liveness on
  `wm.health.*`. `quicken-watch` watches a different axis — *kernel/
  userspace primitive* liveness (dark/inert/degraded), not "is daemon X
  heartbeating." They publish to the same envelope but cover disjoint
  subjects (`wm.health.primitive.<name>` vs the daemon health subjects).
  Confirm with jsy that one envelope / two producers is right, not a
  merge.
