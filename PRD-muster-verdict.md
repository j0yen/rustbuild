# PRD: muster-verdict

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

`muster census` says *who is present and where they came from*. It does not say
*which ones shouldn't be*. `muster verdict` is the judgment layer: it classifies
each roster entry as `live | duplicate | orphan | stale` with an explicit reason
and the evidence it rested on — turning self-review's chronic "may both be real"
into a defensible call.

## Why this exists

The self-review journals for 2026-06-06 and 2026-06-07 both park *"duplicate
Claude sessions … may both be real"* under "Pending your call" — the playbook
sees two PIDs and cannot decide. The information to decide exists once census
attributes origin; what is missing is the rule that turns a roster into a
verdict. Concretely, at dream time (2026-06-08) the roster was
{interactive 33958, headless:/self-review 402723, headless:/dream 402724} —
three sessions, **zero** of which are duplicates or orphans: two distinct
headless roles plus one interactive. A verdict layer says that out loud instead
of leaving a human to eyeball it.

The danger of *not* having an explicit verdict is asymmetric: self-review's own
SKILL warns it must never kill a duplicate because it cannot tell a real
concurrent session from a leak (SKILL.md:70, *"do not kill"*). A defensible
verdict is the precondition for ever safely reaping (muster-reap) — without it,
any kill is a coin flip.

## What this builds

A `verdict` subcommand extending the existing `muster` crate
(`~/wintermute/muster`), consuming census's roster — preserve all census
modules; add a classifier module. SIGPIPE reset already in `main()`; rustc 1.85,
no let-chains.

Per-entry classification with a `reason` string and the `evidence` fields used:

- **`live`** — origin is `interactive-tty` with a healthy tty ancestor, OR a
  `headless:<script>`/`timer:<unit>` run whose launcher ancestor is still alive
  and whose uptime is within the expected window for its role. The normal case.
- **`duplicate`** — two or more `interactive-tty` sessions resolving to the
  **same** project (`~/.claude/projects/<slug>`, derived from cwd). This is the
  exact case the journals flag. Headless roles are never duplicates of each
  other (distinct roles) or of an interactive session.
- **`orphan`** — the session's launcher ancestor is dead (reparented to pid 1)
  **and** the session has been idle past a grace window (no recent ctrace
  activity / RSS+IO flat). Parentless-but-fresh is *not* an orphan (a wrapper
  that exec-replaced itself looks parentless momentarily).
- **`stale`** — a `headless:<script>` `-p` run whose uptime exceeds the known
  timeout budget for its role (dream/self-review/build ticks have bounded
  budgets) — it should have exited and didn't.

Verdict derivation must be **explainable**: every non-`live` verdict carries the
specific evidence (`launcher_pid_dead`, `same_project_slug=<slug>`,
`uptime_s>budget=<n>`, `idle_s=<n>`) so a reader (and muster-reap) can audit the
call rather than trust a label.

- **Output.** Default: census rows annotated with a verdict column + reason.
  `--format json`: census objects each extended with `verdict` and
  `verdict_reason` and `verdict_evidence`. `--only <verdict>` filters
  (e.g. `muster verdict --only orphan`).
- **Grace windows / budgets are config, not magic.** Thresholds (orphan idle
  seconds, per-role stale budgets) live in one documented constants block (or a
  `~/.config/muster/budgets.toml` if one exists), citing where each role's
  budget comes from — not scattered literals.

Out of scope: taking any action on a verdict (muster-reap) and the self-review
playbook edit (muster-selfreview-bridge).

## Acceptance criteria

1. `muster verdict` annotates every census row with exactly one of
   `live | duplicate | orphan | stale` plus a non-empty `reason`.
2. Given two simulated `interactive-tty` entries sharing one project slug, both
   are classified `duplicate` with `same_project_slug` in the evidence; given
   two interactive entries in *different* slugs, both are `live`.
3. Two distinct headless roles (e.g. `/dream` and `/self-review`) running
   concurrently are **both** `live` — never flagged as duplicates of each other
   (the exact false positive the journals worried about).
4. An entry whose launcher ancestor is pid 1 and whose idle time exceeds the
   grace window is `orphan` with `launcher_pid_dead` + `idle_s` in the evidence;
   a parentless entry younger than the grace window is **not** `orphan`.
5. A `headless` entry whose uptime exceeds its role budget is `stale` with
   `uptime_s>budget` in the evidence.
6. `muster verdict --format json` emits `verdict`, `verdict_reason`, and
   `verdict_evidence` on every element; `--only orphan` returns only orphan
   entries.
7. Thresholds are defined in one documented place with a comment citing each
   role's budget source; no bare magic numbers inline at the call sites.
8. `cargo test` green (classifier unit tests cover each verdict + each negative
   boundary); rustc 1.85, no let-chains; existing census tests still pass.
