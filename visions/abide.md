# Vision: abide — a finding you've chosen to carry shouldn't shout every morning

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. This was the third dream
pass of 2026-06-08; the strongest *unaddressed* recurring signals (pacman →
`tide`, dirty-trees → `tend`, ctrace-flake → `scribe`/`mend`, agentns → `assay`)
were already owned. The remaining live signal was inside `docket`'s own data:
two findings the human has explicitly **acknowledged and chosen to carry**
still surface as `[open] warn` every single run.

---

## TL;DR

`docket` (shipped: `docket-core`/`digest`/`escalate`/`evidence`/
`self-review-bind`) gave standing findings a memory — first-seen, last-seen,
consecutive-run streak, escalation, auto-close. It dedupes the *same* finding
across runs so self-review stops re-discovering it from scratch. But it has
exactly three lifecycle states (`docket --help`, verified live 2026-06-08):

```
Status: Open | Escalated | Resolved
```

There is **no state for "I have seen this, I have decided to carry it, stop
telling me until it changes."** Yet self-review uses precisely that semantic in
prose, run after run. From today's journal (`~/brain/journal/2026-06-08.md`):

> **memlog staged-awaiting-install**: pkgrel-11 staged, installed pkgrel-5
> (**acked — will not re-escalate until pkgrel changes**)
>
> **warden inert**: bpolicy present but never armed (**acked**)

`docket list --open` (live this pass) shows both still `[open] warn`,
`runs_seen:3`, surfacing in every digest and every SessionStart banner. The
human's triage decision — *"acknowledged, carry, requite me only on change"* —
exists only as re-typed prose in each day's journal. The ledger cannot
represent it, so the noise `docket` was built to kill leaks right back in: the
banner shows items the human already triaged and deferred days ago.

`abide` adds the missing lifecycle layer: an **acknowledged** flag, orthogonal
to the recurrence `Status`, that keeps a finding *fully tracked* (streak and
`runs_seen` keep counting) but *quiet* in the default digest/banner — and that
**auto-clears the instant its evidence fingerprint changes**, so an acked
finding cannot silently hide a real state change. Same proposal-first,
never-lose-data, evidence-rich ethos as the rest of the docket fleet: ack is
additive and reversible (`docket unack`), and resurfacing on change is
edge-triggered, not time-only.

## Why this is real (Phase 1 evidence, 2026-06-08)

Measured live this pass on `7.0.10-arch1-5-wintermute`:

- **`docket --help` lists `report/list/show/resolve/digest/sweep` — no `ack`.**
  `src/model.rs:50` `enum Status { Open, Escalated, Resolved }` — three
  variants, none for acknowledged-but-carried.
- **`docket list --open` shows two acked-in-prose items still warn-open:**
  `memlog-activation` (runs_seen:3, consecutive:2, "acked — will not
  re-escalate until pkgrel changes") and `warden-enforcer-inert` (runs_seen:3,
  consecutive:2, "acked").
- **The "acked" prose recurs across runs.** Self-review's
  `01KTK2KF17SQYDZ4MZ3PQCK5T1` (2026-06-08) lists both as `(4) ... acked`
  and `(5) ... acked` — the same hand-typed acknowledgements the 2026-06-07
  and -06 reviews carried. The decision is durable in the human's head and
  *ephemeral in the tooling*.
- **`docket` already has the producer-bind seam.** `docket-self-review-bind`
  is shipped — self-review already reports findings into the ledger. Emitting
  an `ack` from the same bind is a small extension, not new plumbing.
- **The fingerprint data already exists.** memlog's carry-condition is "until
  pkgrel changes" — self-review already reads installed-vs-staged pkgrel each
  run. warden's is "until bpolicy arms" — already read as `loaded:bool`. The
  change-signal `abide` keys auto-resurfacing on is *already measured*; it just
  has nowhere to live.

## End-state

When `abide` is fully built:

- `docket ack <key> --reason "<why carry>" [--until-change <fingerprint>]
  [--runs N]` marks a finding acknowledged. It stays in the ledger, keeps
  accruing `runs_seen`/`consecutive_runs`, but drops out of the **default**
  digest, `list`, and SessionStart banner.
- `docket unack <key>` reverses it; no data is ever lost.
- A `report` of an acked finding whose **evidence fingerprint differs** from
  the one recorded at ack-time **auto-clears the ack** — the finding resurfaces
  loudly, because the thing the human agreed to carry has changed. Same
  fingerprint → ack persists.
- `docket digest` and `docket list` exclude acked findings from the
  open/escalated counts but report a separate `acked: N` tally and an
  `--include-acked` flag — nothing is hidden, just quieted. `HealthStatus`
  ignores acked findings (an acked warn no longer reads as `Degraded`).
- Self-review's docket-bind emits `docket ack` whenever it writes an "acked —
  carry until X" note, so the journal prose and the ledger state never diverge
  again. The two live items (`memlog-activation`, `warden-enforcer-inert`) are
  the first real consumers.

## Components (PRD-sized)

1. **abide-ack-state** (`rust-extend` → `~/wintermute/docket`) — the ack
   lifecycle. Add nullable columns `acknowledged_at`, `ack_reason`,
   `ack_fingerprint`, `ack_until_run` (an additive, idempotent SQLite
   migration — **not** a new `Status` variant, so recurrence/escalation logic
   is untouched and ack is orthogonal to open/escalated). New subcommands
   `docket ack <key> [--reason] [--until-change <fp>] [--runs N]` and
   `docket unack <key>`. `report`'s upsert path auto-clears the ack when the
   incoming evidence fingerprint differs from `ack_fingerprint` (and when
   `ack_until_run` is crossed). Fully self-contained; gates the other two.

2. **abide-digest-quiet** (`rust-extend` → `~/wintermute/docket`) — make the
   ack actually quiet the surfaces. `docket digest` and `docket list` exclude
   acked findings from `open`/`escalated` counts by default, add an
   `acked: u64` field to `DigestDetail`, fold it into the `summary` string
   (`"3 open, 1 escalated, 2 acked"`), and exclude acked findings from
   `HealthStatus`. Add `--include-acked` to both. Depends on #1's columns.

3. **abide-selfreview-emit** (`shell` → self-review skill / docket-bind layer)
   — close the prose↔ledger gap. When self-review marks a finding "acked,
   carry until X" it emits `docket ack <key> --reason "<prose>"
   --until-change "<fingerprint>"` via the existing `docket-self-review-bind`
   seam. Wires the two live items: `memlog-activation` (fingerprint = installed
   pkgrel) and `warden-enforcer-inert` (fingerprint = bpolicy `loaded` bool).
   Fail-open if `docket` is absent. Depends on #1 (ack must exist) and #2 (so
   the ack actually quiets the banner).

## Order

- **abide-ack-state first** — it is the foundation (the columns + the ack/unack
  commands + the auto-clear-on-fingerprint-change rule) and ships entirely
  independently inside `~/wintermute/docket`. No consumers.
- **abide-digest-quiet** second — depends only on #1's schema; it is the piece
  that delivers the actual noise reduction (the payoff the vision exists for).
- **abide-selfreview-emit** last — depends on both #1 and #2 and lives in a
  *different* tree (the self-review skill, not the docket crate), so it cannot
  collide with #1/#2's edits. It turns the two live acked-in-prose findings
  into the first durable ledger acks.

## Open questions (for jsy)

- **Fingerprint format.** Lean: a short opaque string the producer chooses
  (e.g. `pkgrel:5`, `bpolicy:false`) compared verbatim — `abide` does not
  parse it, it only checks equality. Keeps docket domain-agnostic. Alternative:
  a structured `{key,value}` — heavier, no clear payoff. Drafted as opaque
  string.
- **Should `sweep` interact with acks?** Today `sweep` auto-resolves findings
  not seen in recent runs. An acked finding *is* still being reported each run
  (that is why it stays open), so `sweep` won't touch it — correct as-is.
  Drafted: leave `sweep` untouched; ack and stale-resolve are disjoint.
- **`--runs N` vs `--until-change` precedence.** If both are set, ack clears on
  *whichever fires first* (fingerprint change OR run-count crossed). Drafted
  that way; flag if you'd prefer change-only.
- Should an acked **crit** be allowed to go quiet, or only warn/info? Drafted:
  allow any severity (the human explicitly chose to carry it), but `digest`
  still surfaces the `acked` count so a quieted crit is never fully invisible.
