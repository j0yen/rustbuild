# Vision: trim — the box thrashes its own memory and nobody is steering

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-18
**Status:** active
**Seed:** bare `/dream` (auto-mode), field FRESH (streak=0,
last-productive 2026-06-18T09:04Z). Phase-1 live inspection found a
recurring self-review finding with **no owner**: "swap 5.4G/8G — heavy,
monitor; not actionable today." It recurs in every journal
(2026-06-12..18) and is explicitly marked un-actionable each time.

## TL;DR

The disk side of this laptop has a rich fleet watching it — `ballast`,
`careen`, `drydock`, `thrift` all reclaim or cap disk. The **memory**
side has nobody. This is a CPU-only box with 15 GiB RAM running, at
once: the wintermute daemon fleet (`agorabus`, `recalld`, `wmd`,
`wm-dialog`, `homeward-embed`, `homeward-ingest`, …), a local LLM
(ollama qwen3:8b pins ~3 cores), Firefox/Slack, and multiple concurrent
Claude sessions. Swap sits at 5.4 GiB of 8 GiB and the box has logged
**364 ms-seconds of cumulative memory stall** (`/proc/pressure/memory`
`total=364808468`). Yet self-review can only write "monitor" because it
has no tool that *attributes* the pressure to a process, *decides*
whether relief is safe, and *applies* the gentlest lever.

trim is that tool. It is to RAM/swap what `ballast` is to disk: survey
→ attribute → policy-gate → relieve → early-warn → automate. The
relief levers are deliberately gentle and default-deny: only idle
wintermute daemons that are systemd-managed and self-healing are ever
touched, never user-facing apps, never a build mid-compile, never the
brain/dialog path during a live voice TURN.

## End-state

When trim is done:

- `trim survey` replaces self-review's "swap 5.4G/8G — not actionable"
  line with an honest, attributed table: which processes/units/cgroups
  hold the resident and swapped pages, and the PSI trend.
- Idle daemons that have been pushed into swap (today:
  `homeward-embed.service` holds **476 MB** swapped, `recalld.service`
  **237 MB**) are recognized as relief-eligible and gently reclaimed —
  a `systemctl --user try-restart` for self-healing units, or a
  `MemoryHigh=` transient cap to bound future growth — only when policy
  allows and only when they are genuinely idle.
- A PSI watcher emits a fleet event before the box thrashes, so the
  problem is seen *before* it stalls a voice turn, not after.
- A 6-hourly timer (mirroring `adopt-cron`) runs the survey + applies
  in-policy relief with a receipt, dry-run unless told otherwise.
- Every relief action is reversible, logged, and never touches a
  user-facing process or a live conversational turn.

## Components (one bullet per future PRD)

1. **trim-survey** — NEW repo `~/wintermute/trim` (rust-cli+lib).
   The honest enumerator. Snapshots `/proc/meminfo`, every
   `/proc/<pid>/status` (VmRSS, VmSwap), cgroup-v2
   `memory.current` / `memory.swap.current` / `memory.pressure`, and
   `/proc/pressure/memory` (PSI). Attributes resident+swapped bytes to
   process, systemd unit, and cgroup. Emits JSON for tooling + a ranked
   table for humans. FOUNDATIONAL.

2. **trim-attribute** — rust-extend `build_into=~/wintermute/trim`.
   Classifies each holder into a class: `wintermute-daemon`,
   `claude-session`, `local-llm`, `build` (rustc/cargo), `user-app`
   (firefox/slack), `system`. Uses systemd unit identity (and provfs /
   agentns session id where present) for attribution. Tags each holder
   with a relief-eligibility *candidacy* (not yet a decision — that's
   policy's job).

3. **trim-policy** — rust-extend. The default-deny gate (mirrors
   `consign-policy`). A holder is relief-eligible ONLY if it is a
   systemd-managed wintermute daemon, marked self-healing
   (`Restart=always` drop-in present), currently idle (below an
   activity threshold), and holds swapped/resident bytes above a floor.
   Hard exclusions, never overridable by config: user-apps, the live
   brain/dialog path during an active voice TURN, any build process,
   anything not under `user.slice`.

4. **trim-relief** — rust-extend. The act. For an eligible daemon,
   pick the gentlest effective lever in order: (a) IPC/signal "drop
   caches" if the daemon supports it (recalld's cold-load cache), else
   (b) `systemctl --user try-restart <unit>` for self-healing units,
   else (c) `systemctl --user set-property <unit> MemoryHigh=<cap>` to
   bound future growth. **Dry-run by default** (`--no-dry-run` to act),
   mirroring `adopt apply` / `consign drain`. Writes a receipt per
   action; every lever is reversible.

5. **trim-psi** — rust-extend. The early-warning watcher. Tails
   `/proc/pressure/memory` (+ per-cgroup `memory.pressure`) and swap
   rate; when `some avg60` or swap-in rate crosses a sustained
   threshold, emits an `agorabus` event (`trim.pressure`). It NEVER
   auto-acts — it only warns, so relief stays behind policy. The memory
   analogue of `pulse`'s deafness early-warning.

6. **trim-cron** — shell (systemd-user units). A 6-hourly timer
   (mirror `adopt-cron` / `consign-cron`) running `trim survey` +
   `trim relief --no-dry-run` within policy, plus a guarded, additive
   self-review skill-doc block that replaces the "swap … not actionable"
   line with trim's attributed one-liner. WRAP, don't replace,
   self-review reporting; degrade to an honest one-liner + exit 0 when
   trim is absent.

## Order

```
trim-survey ─► trim-attribute ─► trim-policy ─┬─► trim-relief ─► trim-cron
                                              └─► trim-psi
```

- `trim-survey` is foundational (the NEW repo). Build first.
- `trim-attribute` and `trim-policy` extend it in sequence (policy
  needs attribute's class tags).
- `trim-relief` and `trim-psi` both depend on policy and are
  **independent of each other** (parallelizable). relief acts; psi
  only warns.
- `trim-cron` depends on `trim-relief` (it invokes `relief --no-dry-run`).

## Open questions

- Should `trim-relief` ever escalate to `oom_score_adj` nudging (make a
  daemon a *preferred* OOM victim) rather than restarting it? Leaning
  no for v1 — restart/cap is reversible and legible; oom_score_adj is a
  blunt instrument.
- recalld "drop caches" IPC: does recalld expose a signal/socket to
  shed its in-memory BGE/index cache, or is `try-restart` the only
  lever today? If no IPC exists, that's a follow-on PRD against recall
  (`recall daemon --shed`), left for the next pass rather than dreamed
  here without evidence.
- Activity threshold for "idle": CPU% over a window, or
  unit-specific (e.g. homeward-ingest's AIMD cadence has known quiet
  windows)? Start with a simple recent-CPU floor; refine per-unit later.
- Should trim coordinate with `thrift`'s model-tier swap so that
  relieving the local LLM (ollama) is a *thrift* decision, not a trim
  one? Leaning: trim treats ollama as `local-llm` class and defers its
  relief to thrift — trim only owns the daemon fleet. Boundary to
  confirm with jsy.
