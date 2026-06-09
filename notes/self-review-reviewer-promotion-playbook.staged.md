<!--
STAGED EDIT (blocked by auto-mode classifier on 2026-05-29, PRD autobuilder-reviewer-promotion).
This is Artifact 3 of PRD-autobuilder-reviewer-promotion: the `reviewer_promotion_check`
Phase B.5 playbook to be inserted into ~/.claude/skills/self-review/SKILL.md immediately
BEFORE the "### Adding new playbooks" heading.

The /build tick's edit-agent was blocked editing ~/.claude/skills/self-review/SKILL.md
(self-modification of a sibling skill). A human with permission should paste the block below
verbatim above "### Adding new playbooks". No other edit is needed; the autobuilder-side
Phase A doctrine already ships in ~/.claude/skills/autobuilder/SKILL.md.
-->

### Playbook: `reviewer_promotion_check`

**Cadence**: weekly — only run when today is Sunday (`date +%u` returns `7`). On any other weekday this playbook is inert; skip it without logging. (It reads a slow-moving calibration log; daily evaluation would just re-emit the same verdict.)

**Trigger** (all must hold, evaluated only on Sunday):
- `~/.claude/skills/autobuilder/state/reviewer-calibration.jsonl` exists and is non-empty.
- The autobuilder SKILL.md still declares the reviewer gate in its **current** phase below the latest already-promoted phase (i.e. there is a higher phase to promote to). Detect the current phase by grepping `~/.claude/skills/autobuilder/SKILL.md` for the marker line `**Phase A (current` / `**Phase B (current` — whichever `(current` marker is present is the active phase. If `Phase C` is already current, this playbook is inert (no higher phase).

**Investigation (read-only)**:
1. Count shipped lines: `jq -s '[.[] | select(.shipped==true)] | length' state/reviewer-calibration.jsonl`. Call it `n`.
2. Over the **last 30 shipped** lines, count those with `verdict=="concern"` whose `post_ship_revert==true`, and those with `verdict=="concern"` total. Compute `concern_to_revert_rate = concern_reverts / max(concern_total, 1)`. If `concern_total == 0` the rate is `0` by definition. Record `n`, `concern_total`, `concern_reverts`, and the rate to the apply-log diagnosis.
3. Determine the current active phase from the autobuilder SKILL.md marker (see Trigger). Record it.

**Auto-fix conditions** (ALL must hold to attempt a promotion edit):
- `n >= 30`. Below 30 there is not enough calibration data — log `step:diagnose` with `n=<X>, threshold=30, no_promotion` and stop (this satisfies the n<30 stub-fixture path: clean run, no edit).
- The target promotion is a strict forward step (A→B or A/B→C), never a downgrade.
- The autobuilder SKILL.md is writable AND `git -C ~/.claude/skills/autobuilder rev-parse --git-dir` succeeds (the edit is recorded as an `evolve:` commit per the autobuilder Stage 5 self-evolve mechanism).
- The most recent `investigate.reviewer_promotion_check` entry with `step:fix_attempted` in `apply-log.jsonl` is from a **different ISO week** than today, OR there is no prior entry. **Loop-breaker**: at most one promotion per calendar week, even across self-review runs.

**Fix** (pick exactly one target by rate, then perform a single in-place doctrine edit):
- If `n >= 30` AND `concern_to_revert_rate < 0.50` AND current phase is A → promote to **Phase B (soft-block)**: in `~/.claude/skills/autobuilder/SKILL.md`, move the `(current` marker from the Phase A bullet to the Phase B bullet (and update the Stage 4 receipt-table `reviewer-agent` cell's parenthetical to read `Phase B (current)`), so `concern` becomes a soft-block bypassable only by PRD frontmatter `reviewer_override: true` + `reviewer_override_reason:`. Commit in the autobuilder skill repo with the j0yen identity: `git -C ~/.claude/skills/autobuilder -c user.email=jyen.tech@gmail.com -c user.name="j0yen" commit -am "evolve: reviewer-agent concern → soft-block (n=N, rate=R)"`.
- If `n >= 30` AND `concern_to_revert_rate >= 0.50` AND current phase is A or B → promote to **Phase C (hard block)**: move the `(current` marker to the Phase C bullet (and update the receipt-table cell to `Phase C (current)`); `concern` becomes a hard block with no frontmatter override. Commit message: `evolve: reviewer-agent concern → hard-block (n=N, rate=R)`.

Use `~/.local/bin/txn-edit snap ~/.claude/skills/autobuilder/SKILL.md` before the edit and `txn-edit commit <id>` only after the marker move verifies (exactly one `(current` marker remains, on the new phase). On any inconsistency, `txn-edit rollback <id>` and log `step:fix_failed`. After the git commit, log `step:fix_verified` with the new phase and the commit sha. The `evolve:` git commit + apply-log entry are the durable record.

**Escalation**: if `n >= 30` but the autobuilder SKILL.md is not a git repo, or the `(current` marker is absent/ambiguous (zero or >1 matches), do NOT edit — write to Pending: `reviewer_promotion_check: n=N rate=R wants <phase>; blocked on <not-a-repo|ambiguous-marker>` so a human applies the phase edit deliberately. A miscalibrated auto-edit to the build system's own gate is worse than a one-week delay.
