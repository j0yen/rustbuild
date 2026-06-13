# PRD: adopt-self-review-bind — retire the hand-written "never installed" prose

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/.claude/skills/self-review/SKILL.md
Vision: visions/docket.md (Fleet 2 — the adoption forcing function)

## TL;DR

Every self-review since 2026-06-08 hand-writes the same Pending lines:
"fleet-binary-staleness wm-audio/dialog/tts/stt (rollout plan needed)"
and "binstale never installed." That recurrence is exactly what `adopt
scan` + `adopt report` were built to mechanize. This PRD wires them into
the self-review's Phase B.5 playbook section so the adoption gap becomes
a deterministic probe with the one-command fix pre-filled in Pending —
and the prose note retires.

## Why this exists (Phase 1 evidence, 2026-06-12)

- **The prose is load-bearing and manual.** Reflective memories on
  2026-06-08/09/10/11/12 each carry the staleness/never-installed items
  by hand under "Pending." This is precisely the carry-forward-by-prose
  pattern docket Fleet 1 mechanized for *recurring findings* — adoption
  findings are the un-mechanized remainder.
- **The detector + reporter now exist** (PRD-adopt-scan,
  PRD-adopt-docket-report). The only missing link is the self-review
  *invoking* them, the same way binstale-self-review wired `binstale
  scan` into Phase B.5 (vigil Fleet 1) and docket-self-review-bind wired
  `docket report`/`docket list` into Phase 0/E.
- **Precedent for the wiring is established.** `self-review/SKILL.md`
  already has a Phase B.5 deterministic-probe section and a docket
  report/list/sweep integration. This PRD adds one more probe in the
  same idiom; it does not invent a mechanism.

## What this builds

An edit to `~/.claude/skills/self-review/SKILL.md` (and, if the playbook
factors probes into helper scripts, a small wrapper under the skill's
`scripts/` dir) that:

1. **Phase B.5 probe.** Adds an `adopt` probe: run `adopt scan
   --format json`; if any artifact is `not-installed`/`installed-stale`,
   the probe is a hit.
2. **Report to docket.** On a hit, run `adopt report --run <review-run-id>`
   so each unadopted artifact lands as an `adopt:<bin>` docket entry
   (streak/escalation handled by docket).
3. **Pending rendering.** Surface the open `adopt:*` docket entries in
   the review's Pending section *with the pre-filled fix line* from the
   scan (`cargo install --path <repo> --root ~/.local`, or the
   `rollout install` path for daemons) — replacing the hand-written
   "rollout plan needed / binstale never installed" prose.
4. **Guardrail.** The probe is **report-only** by default — it does not
   install anything (adopt-apply, gated on jsy's autonomy confirmation,
   owns mutation). The bind explicitly notes that distinction so a
   future reader doesn't assume the probe self-heals.
5. **Degradation.** If `adopt` is not on `$PATH` (the bootstrap
   chicken-and-egg: adopt itself not yet adopted), the probe emits a
   single Pending line naming `adopt` as the missing tool with its own
   install command, and the review continues. The probe must never
   abort the review.

## Acceptance criteria

1. `self-review/SKILL.md` Phase B.5 documents an `adopt` probe that runs
   `adopt scan --format json` and reports hits via `adopt report --run
   <id>`.
2. The playbook text specifies that open `adopt:*` docket entries are
   rendered in Pending **with the pre-filled fix command**, replacing
   the hand-written staleness/never-installed prose.
3. The probe is documented as **report-only** (no install) by default,
   with mutation explicitly delegated to `adopt apply` under jsy's
   autonomy gate.
4. The playbook specifies graceful degradation when `adopt` is absent
   from `$PATH`: one Pending line, review continues, no abort.
5. If a wrapper script is added under the skill's `scripts/` dir, it is
   SIGPIPE-safe and exits 0 on the no-hit path.
6. The edit does not remove or weaken any existing Phase B.5 probe or
   the existing docket report/list/sweep integration (additive only).
7. A dry verification (documented in the PRD's iter log or a comment)
   shows the probe, run against the current laptop, would surface at
   least `adopt:rollout` (the live 9-day-unadopted artifact) as a
   Pending item.

## Iter log

### 2026-06-13 — wiring committed, dry-verification run

- Playbook `adopt_scan_probe` added to Phase B.5 of
  `~/.claude/skills/self-review/SKILL.md` via commit `53954c6`
  ("adopt-self-review-bind: wire adopt scan into Phase B.5").
  Insertions: 60 lines. All structural ACs (1–4, 6) satisfied inline.
  No wrapper script added (AC5 vacuously satisfied — probe is inline).

- **AC7 dry verification** (2026-06-13): `adopt scan --format json`
  run against current laptop state:
  - `rollout`: `not-installed` → fix_cmd:
    `cargo install --path /home/jsy/wintermute/rollout --root ~/.local`
  - Also surfaced: `ac-judge` (installed-stale), `apipe` (not-installed),
    `agentns-claude`/`agentns-doctor`/`wm-busbridge` (installed-stale),
    and others. AC7 criterion met: `adopt:rollout` surfaces as a Pending
    item.

- `verified-completed.sh` run with `--paired 1,2,3,4,5,6,7`:
  all 7 ACs classified PAIRED, `missing: []`. Gate passes.
