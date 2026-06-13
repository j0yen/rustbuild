# Vision: docket — findings get a memory, not just a mention

> The self-review notices the same thing every run. It writes it down,
> in prose, and parks it. Next run it notices it again. A docket gives
> each finding an identity, a lifespan, and a rule for when "noticed
> again" becomes "do something."

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-05-29
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection (recall reflective
seeds + journal recurrence + self-review SKILL.md).

---

## TL;DR

Every self-review run rediscovers findings it discovered before, writes
them into a hand-maintained "Carried forward from prior reflections"
prose section, and parks them under "Pending your call." The recurrence
that *should* trigger action — the SKILL.md rule that a signal seen
across **3+ separate runs** justifies a durable playbook — is detected
by eyeballing `recall query 'self-review'` output across runs. There is
no structured store that counts how many runs a finding has survived,
escalates it when it crosses the threshold, or closes it when it stops
appearing. `docket` is that store: a small ledger keyed by stable
anomaly slug, where producers (the self-review, and later any tool)
*report* findings, and the ledger tracks first-seen, last-seen,
consecutive-run streak, occurrence count, escalation, and auto-close.

## Why this is real (Phase 1 evidence, 2026-05-29)

Measured live this session:

- **Recurrence is the norm, not the exception.** `grep -l "Carried
  forward" ~/brain/journal/*.md` matches **6 consecutive days**
  (2026-05-24 → 2026-05-29). The "agorabus daemon stale binary" finding
  alone appears **7× in the 2026-05-28 journal** and 3× in 2026-05-29.
- **The threshold is codified but eyeballed.** `self-review/SKILL.md`
  line 359: *"A new playbook is justified when a signal recurs in
  `recall query 'self-review'` results across **3+ separate runs**."*
  The mechanism for "recurs across 3+ runs" is a human/agent reading
  prose. Run-18 and run-19 reflective memories (recall
  `01KSRV7R4FERPP40HQGV5RGZNT`, `01KSS21WFN5H6V42JF723Z8K2J`) both say
  the stale-binary item is *"approaching the 3-runs threshold where a
  more durable handling would be justified"* — i.e. the agent is
  manually counting.
- **The store is unstructured.** `self-review/SKILL.md` lines 452-465:
  each run persists **one** reflective recall memory whose free-text
  *"Pending"* line is the entire carry-forward state. Future runs hit it
  with `recall query` (semantic/FTS over prose) — there is no per-finding
  entity, no lifecycle, no count. `~/.claude/skills/self-review/state/`
  does not exist; the skill has no structured state at all.
- **Findings stay open for ~20+ runs with no escalation.** "agentns
  agent_session all-zeros" has been Pending for ~21 consecutive runs
  (run-13 reflective `01KSK8SDM4...` through today). "ctrace missing
  SessionEnd summaries" has been open 5 runs. Each is rediscovered,
  re-typed, re-parked.

This is the third axis of staleness the laptop has been missing.
`vigil` watches *running binaries* drift from source. `freshness`
watches *memory bodies* go stale. `drift` watches *skill text*. None of
them watch the **self-review's own findings** accumulate, recur, and
demand escalation. docket is that watcher.

## End-state

When this vision is done:

1. The self-review reads its carry-forward state from `docket list
   --open` (structured), not by grepping journals/recall prose.
2. Each Pending finding is a docket entry with a stable key, a
   first-seen date, a consecutive-run streak, an occurrence count, and a
   typed evidence trail (recall ULIDs, journal lines, pids, commits).
3. When a finding crosses the 3-run threshold the ledger marks it
   `escalated` automatically and records *why* — turning SKILL.md
   line 359 from a manual rule into a mechanical one.
4. When a finding stops appearing for K runs the ledger closes it
   `resolved(stale)` automatically — so the carry-forward list shrinks
   without anyone deciding to drop an item.
5. A `docket digest` surface exposes the standing open/escalated set to
   the SessionStart banner and to consumers (kin's health digest,
   homestead's readiness-beacon), reusing the `wm.health.*` envelope
   rather than inventing a parallel one.

## Components (PRD-sized)

1. **docket-core** — new `rust-cli` (`j0yen/docket`, `~/.local/bin/docket`).
   SQLite ledger at `~/.local/share/docket/docket.db`. Entities keyed by
   stable slug. `docket report --run <id> --key <slug> --title <t>
   [--severity] [--evidence <ref>]` (dedupes within a run, bumps streak
   across runs), `docket list [--open|--escalated|--resolved]
   [--format json]`, `docket show <key>`, `docket resolve <key>
   [--reason]`. Run-aware occurrence counting (distinct runs, not raw
   reports). The foundation everything else reads.

2. **docket-escalate** — `rust-extend` into docket. The lifecycle rules:
   on report, if `consecutive_runs` ≥ threshold (default 3) →
   `status=escalated` + reason citing SKILL.md §line-359; `docket sweep
   --run <id>` auto-resolves open entries not seen in the last K runs as
   `resolved(stale)`. Configurable thresholds. Automates the manual
   carry-forward bookkeeping.

3. **docket-evidence** — `rust-extend` into docket. Typed evidence refs
   (`recall:<ulid>`, `journal:<date>#<line>`, `pid:<n>`,
   `provfs:<ts>`, `commit:<sha>`) accumulated across a finding's
   occurrences; `docket show` renders the trail. Lets an entry point at
   every run that observed it.

4. **docket-self-review-bind** — `mixed` (edit `self-review/SKILL.md` +
   a small wrapper script). The load-bearing integration: Phase 0 reads
   `docket list --open`; the "Carried forward" / "Pending" sections
   `docket report` each finding; Phase E runs `docket sweep`; the 3-run
   playbook-justification check becomes `docket list --escalated`.

5. **docket-digest** — `rust-extend` into docket. `docket digest
   [--format json|text]` produces the standing open/escalated set for
   the SessionStart banner and for reuse by kin / readiness-beacon.
   REUSES the `wm.health.*` envelope (owned by companion-degrade,
   consumed by kin) — does not invent a parallel schema.

## Order

```
docket-core ──┬── docket-escalate ──┐
              └── docket-evidence ───┴── docket-self-review-bind
                                      └── docket-digest
```

core first (defines the store + report/list contract). escalate and
evidence both extend the store and are independent of each other.
self-review-bind needs core + escalate (it reports findings and reads
escalated). digest needs core (lists) and is nicer with escalate but
doesn't strictly require it.

## Fleet 2 — the adoption forcing function (drafted 2026-06-12)

Fleet 1 made docket a *passive ledger*: it counts recurrence, escalates,
auto-closes. It tracks a finding; it does not act on one. That was the
right v1 boundary. But six days of evidence have proven there is a
*specific class* of finding the ledger will track forever without
anyone closing it — because closing it is an **adoption action** that
nothing on the laptop performs.

Three prior `/dream` passes (gossip 2026-06-06 03:05, 2026-06-06 07:45,
2026-06-08 05:25) independently reached the same conclusion, verbatim
from the 03:05 note:

> the only un-visioned recurring escalations are install/arming ACTIONS,
> not design gaps (memlog staged-awaiting-install, warden arming,
> binstale never installed). These want a user action or a
> forcing-function PRD under docket … Reconsider if they keep aging.

**They kept aging.** Phase 1 of this pass (2026-06-12) measured live:

- `rollout` — the tool whose *entire job* is bringing stale fleet
  binaries current — was built `2026-06-03 09:45` (`target/release/rollout`
  present), committed, has **1 unpushed commit**, and is **not on PATH**
  (`rollout not found`; absent from both `~/.local/bin` and
  `~/.cargo/bin`). Unadopted for **9 days**. The fixer is itself the
  thing that never got fixed.
- The 4 voice daemons (`wm-audio|dialog|tts|stt`) are `behind-head`
  right now per `binstale scan` — and would be brought current by
  `rollout apply`, except `rollout` isn't installed to run.
- `warden` — not on PATH (only `bpolicy` is); inert across every
  self-review since it shipped.
- Self-reviews on 06-08/09/10/11/12 *all* re-flag
  "fleet-binary-staleness … (rollout plan needed)" by hand.

The structural gap: **`/build` ships a repo (build → commit → push) but
adoption into the live system (install → land on PATH → become
invokable, or for daemons restart the unit) is a separate, frequently-
skipped, unverified step.** And critically, the existing detector
cannot see this class: `binstale check <PID>` operates on a *running
process* (exit 2 if the process isn't found) — it structurally cannot
flag a CLI that was *never installed and therefore never runs*. vigil
covers daemons running stale bytes; nothing covers an artifact that
never entered the live system at all.

This is docket open-question #4 ("Who else reports?") coming due: a new
**adoption detector** that reports unadopted artifacts to docket with
the exact one-command fix, plus the *optional* mutating half that
actually performs the safe installs — turning "tracked forever" into
"closed."

### Components (Fleet 2)

1. **adopt-scan** (`rust-cli`, new repo `~/wintermute/adopt/` →
   `~/.local/bin/adopt`) — read-only detector. Enumerates the laptop's
   shipped artifacts (wintermute repos that declare a `[[bin]]` /
   `default-run` in `Cargo.toml`, plus the autobuilder/build manifest of
   shipped slugs) and, per artifact, classifies adoption:
   `installed-current | not-installed | installed-stale | not-a-bin`.
   Signals: binary present on `$PATH`; invokable (`--version`/`--help`
   exits 0); install timestamp / provfs `user.prov.ts` vs the repo's
   newest `src/` commit. For the *running-daemon* case it defers to
   `binstale` (no re-implementation); its own contribution is the
   *never-installed* and *installed-but-stale-on-disk* cases binstale
   can't reach. Emits the exact fix per artifact
   (`cargo install --path <repo> --root ~/.local`). JSON + table. **No
   mutation.** Foundational — everything in Fleet 2 reads it.

2. **adopt-docket-report** (`shell` or `rust-extend` → adopt) — the
   forcing function the three prior dreams asked for. Maps each
   `not-installed`/`installed-stale` verdict to a stable docket key
   (`adopt:<slug>`) and calls `docket report --run <id> --key … --title
   … --severity --evidence path:<repo> --evidence commit:<sha>`, so an
   unadopted artifact accrues a streak and auto-escalates at the 3-run
   threshold exactly like any other finding — and auto-closes (`docket
   sweep`) the run after it's finally installed. Depends on the live
   `docket report` contract (shipped, v0.5.0) + adopt-scan.

3. **adopt-self-review-bind** (`shell` → edits
   `~/.claude/skills/self-review/SKILL.md` Phase B.5) — runs `adopt
   scan` structurally each review, pipes verdicts through
   adopt-docket-report, and surfaces the one-command fix in Pending —
   retiring the hand-written "binstale never installed / rollout plan
   needed / fleet-binary-staleness" prose that has appeared every run
   for a week. Depends on adopt-scan (+ adopt-docket-report).

4. **adopt-apply** (`rust-extend` → adopt) — the mutating half that
   actually *closes* the loop rather than only tracking it. `adopt
   apply` installs the safe `not-installed` **non-daemon** CLIs
   (`cargo install --path … --root ~/.local`), one at a time,
   idempotent, with `--dry-run` the default posture. Daemon-backed
   artifacts are **out of scope by default** and delegated to
   `rollout`/vigil's `vigil-install-restart` (only touched under an
   explicit `--with-daemons` that shells to `rollout install`) — so
   adopt never bounces a live voice daemon. No overlap with vigil:
   vigil = a daemon running stale bytes needs a guarded restart; adopt =
   a plain CLI never entered PATH and has no unit to restart. Depends on
   adopt-scan.

### Order (Fleet 2)

```
adopt-scan  (read-only detector; ship first)
   ├──► adopt-docket-report     (reports verdicts to docket; the forcing function)
   │       └──► adopt-self-review-bind   (wires scan+report into self-review B.5)
   └──► adopt-apply             (optional mutating half; non-daemon installs, --dry-run default)
```

adopt-scan first (defines the artifact taxonomy + verdict contract).
adopt-docket-report and adopt-apply both consume adopt-scan and are
independent of each other (report tracks; apply closes).
adopt-self-review-bind needs scan + report. adopt-apply can ship
anytime after scan but is most useful once report proves the verdict
set is stable.

## Open questions

- **Run identity.** What is a "run"? Proposed: caller-supplied string
  (self-review passes e.g. `2026-05-29.1`, or the reflective ULID it's
  about to write). docket stays agnostic. Confirm with jsy.
- **Store format.** SQLite (queryable, transactional, matches recall's
  precedent) vs. JSONL (greppable, diffable, matches gossip/journal
  ethos). Leaning SQLite for the streak/sweep queries; open to JSONL if
  jsy wants the ledger in git.
- **Overlap with recall.** docket entries link to recall ULIDs but are a
  distinct lifecycle store, not a recall extension (recall is
  similarity-retrieval; docket is per-key state machine). Confirm jsy
  agrees this stays a separate tool.
- **Who else reports?** v1 producer is the self-review. Future producers
  (vigil's binstale, homestead's readiness-beacon, /build blockers)
  could all report to one docket — left as a vision boundary note, not a
  v1 PRD, until the contract proves out with the self-review alone.
  **Coming due 2026-06-12:** Fleet 2's `adopt-docket-report` is the
  first non-self-review producer — the contract has proven out (docket
  v0.5.0 live), so the boundary note becomes a real PRD.
- **Should adopt auto-install? (Fleet 2)** `adopt-apply` defaults to
  `--dry-run`. Auto-installing a *non-daemon* CLI is reversible and
  low-blast-radius, and the standing user instruction is "always
  auto-publish, always commit, no caps" — leaning toward letting
  `adopt-apply` run safe non-daemon installs autonomously from
  self-review within guardrails (one at a time, idempotent, never
  daemons). Daemon restarts stay deferred to `rollout`/vigil. Confirm
  the autonomy posture with jsy before adopt-self-review-bind calls
  `apply` (vs only `report`).
