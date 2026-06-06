# Vision: coda — every session gets its closing summary

## TL;DR

Every Claude session is traced to a ctrace NDJSON log, and a one-page
`.summary.md` is supposed to render when it ends. But the SessionEnd hook
only fires on a *graceful* exit — and the sessions that matter most (the
headless `/build`, `/dream`, `/self-review` timer ticks) are SIGKILLed by
cgroup teardown before the hook runs, leaving their logs permanently
un-summarized. The repair engine already exists (`scribe backfill`), but
nothing runs it reliably; it only happens when a self-review pass *remembers*
to. The result is a slowly-growing pile of orphaned logs — **623 of 1874
logs (33%) have no summary as of 2026-06-05**. **coda** is the missing
trigger/detection/self-healing layer: a small inward toolkit that measures
the summary debt, classifies it, and closes it automatically — so a
session's coda is never lost just because it died ungracefully.

## End-state

- A single source of truth for **summary debt**: which session logs are
  closed-but-un-summarized, how old, and why (active / fresh / orphaned).
- Debt is closed **automatically and idempotently**, on a cadence and on the
  next session start — *not* only when a human-driven self-review runs it.
- `coda audit` answers "how far behind are we" at a glance; self-review reads
  it instead of re-deriving the gap by hand each run.
- The debt count is **legible to the docket** (edge-triggered), so a
  regression (the renderer breaks, the timer dies) surfaces as a tracked
  signal instead of silent rot.
- self-review's per-run "ctrace backfill: rendered N, skipped M" toil
  disappears — coda has already closed the gap before the review looks.

## Why this direction (evidence)

- **Open docket item `ctrace-sessionend-flake`** ("ctrace session-end event
  missing", runs_seen 3, first_seen 2026-05-30) — the symptom, still open.
- **The gap is large and live.** `ls ~/.cache/ctrace/sessions/` = 1874
  `*.ndjson` vs 1251 `*.summary.md` → **623 orphaned** (observed 2026-06-05).
  Every one of today's timer ticks (`claude-20260605T210001`, `…T213000`,
  `…T220007`, `…T223000`, `…T230000`) is missing its summary.
- **The cause is documented by the engine itself.** `ctrace-scribe/README.md`:
  *"Heavy headless sessions are SIGKILLed by cgroup teardown before the hook
  runs, leaving logs permanently un-summarized."*
- **The repair is inconsistent.** Journals show backfill rendering **50 on
  06-03, 2 on 06-02, 0 on 06-01** — entirely dependent on whether a
  self-review pass invoked it. The 06-02 journal explicitly flagged it:
  *"if it keeps missing, the SessionEnd hook needs a look."*
- **The engine already exists** (`scribe render` / `scribe backfill` /
  `scribe rollup` in `~/wintermute/ctrace-scribe`). coda does **not**
  re-implement rendering — it builds the trigger and self-healing the engine
  was always missing.

## Components (one bullet per future PRD)

- **coda-sweep** — new repo `~/wintermute/coda`; the corpus. Shared types
  (`SessionLog` / `SummaryState` / `DebtClass` / `SweepPlan` / `SweepAction`),
  a `LogStore` trait abstracting the sessions dir, a `FakeStore` for tests,
  and a **pure** `sweep(logs, active_log, now)` that classifies every log and
  emits a print-only plan. `coda plan` shows it. (Mirrors anchor-roots.)
- **coda-audit** — rust-extend; the live read. Implements the real `LogStore`
  over `~/.cache/ctrace/sessions/`, reads the active log from `ctrace status`,
  scans, and reports the debt (human table + `--format json`). Read-only:
  never renders. (Mirrors anchor-probe.)
- **coda-close** — rust-extend; the actuator. Takes the `SweepPlan` and closes
  debt by shelling `scribe render` per orphaned log. Default **print-only**;
  `--apply` is the one live-side-effect path. Idempotent. (Mirrors
  anchor-reconcile.)
- **coda-boot** — mixed; the trigger. A systemd-user timer running `coda close
  --apply` on a cadence, plus a SessionStart hook that closes the *previous*
  session's debt (the reliable repair point, since the prior headless session
  was SIGKILLed). Installer **prints** the unit + hook entry; never auto-edits
  `~/.claude` or auto-enables. (Mirrors anchor-boot.)

## Order (strict dependency)

```
coda-sweep   (FIRST — creates repo + types + trait + pure sweep)
   │
   ├── coda-audit   (live read; depends on sweep types ONLY)   ┐ build in
   └── coda-close   (--apply actuator; depends on sweep ONLY)  ┘ parallel
                          │
                     coda-boot  (systemd timer + SessionStart hook; needs close --apply)
```

Do **not** start any rust-extend until coda-sweep has SHIPPED and the repo
exists, or extend-validate fails (the rule that bit relay/concord/quicken/
keel/anchor). coda-audit and coda-close depend on coda-sweep's types only and
build in parallel.

## Open questions (next /dream pass / user)

- **coda-witness** (rust-extend) — emit a `wm.coda.debt` reading to the docket
  (orphan count + oldest-unclosed age), edge-triggered like keel-beacon, so a
  renderer regression surfaces as a tracked signal. Held: leaning yes, but the
  hook (coda-boot) may be the right place to open the docket key rather than a
  separate probe. Decide after boot ships.
- **Retention** — 623 logs is already a lot; should coda also *prune*
  summarized ndjson older than N days (the raw trace is large; the summary is
  the durable artifact)? Held — pruning is a separate, destructive concern;
  keep coda repair-only until the user opts into a retention policy.
- **Should coda-close subsume self-review's backfill step entirely**, or run
  alongside it? Leaning: coda becomes the canonical path and self-review's
  Phase reads `coda audit` instead of invoking `scribe backfill` directly.
  Coordinate via gossip once coda-boot is live so /build doesn't ship a
  self-review edit that races coda.
- **Grace window** — how long after a log's last write before it's "orphaned"
  vs "fresh, hook may still fire"? Start at 120 s (the headless ticks die
  fast); make it a manifest knob in coda-sweep.
