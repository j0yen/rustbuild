# PRD: inoculate-inject — vertical transmission at spawn

Status: Draft v0.1
build_target: shell
Vision: visions/inoculate.md

## TL;DR

The subagents `/build`, `/dream`, and `Workflow` spawn do the most consequential
autonomous work on this box — they write and publish code — yet they start from
**task-only prompts with no ethical context**. `inoculate-inject` is the vertical
infection vector: a tiny strain-preamble emitter plus the wiring that prepends it
to every subagent prompt, so the ethic is present *at birth*, not just in the
orchestrator.

## Why this exists

Verified live 2026-06-15: `grep -in 'CLAUDE_SELF\|values\|ethic\|redline\|boundaries'
~/.claude/skills/build/SKILL.md` returns only changelog-prepend and
defaults-parser references — the build skill **never** injects Values/Boundaries
into the Agent prompts it dispatches. The build skill's own text: "dispatched as
parallel Agent (subagent) tool calls in a single tool-use message." Those
prompts carry the PRD task and nothing about how this box expects an agent to
behave. `answerable`'s spine governs the main loop only; the children are
ungoverned.

## What this builds

Depends on `inoculate-core` (`inoculate strain`).

- `~/.local/bin/inoculate-preamble` (shell) — prints a compact, prompt-ready
  preamble derived from `inoculate strain --format text`: a short header ("You
  are a subagent on wintermute. You inherit these commitments; they bound your
  autonomy:") followed by the Boundaries + the redline summary + the in-force
  `strain_hash` (so the child can echo it for a carrier check). Caps length
  (e.g. ≤1500 chars) so it doesn't dominate small prompts; falls back to a
  built-in minimal floor if `inoculate` is not on PATH (must never block a spawn).
- Wiring docs + helper `inoculate-inject wrap <prompt-file>` that emits
  `<preamble>\n\n---\n\n<original prompt>` — the canonical composition the
  skills call.
- SKILL.md edits (self-mod, distributed via the build skill's `self-push.sh`):
  add a one-line "inoculate the subagent" step to `/build` Phase 4 dispatch and
  `/dream` research-agent dispatch, and a note in the Workflow-usage guidance,
  instructing the orchestrator to prepend `inoculate-preamble` output to each
  spawned prompt.

This PRD is shell/hooks + a documented convention; it does not change the Agent
tool itself (can't), it changes what the orchestrator *puts in the prompt*.

## Acceptance criteria

1. `inoculate-preamble` prints a non-empty preamble ending with the current
   `strain_hash`, and exits 0; with `inoculate` absent from PATH it prints the
   built-in minimal floor and still exits 0 (never blocks a spawn).
2. Output is ≤1500 chars by default; `--max <n>` overrides.
3. `inoculate-inject wrap <file>` emits `<preamble>` + separator + the file's
   contents, in that order, verified by a fixture test.
4. `set -uo pipefail` clean; shellcheck has no errors.
5. The build SKILL.md and dream SKILL.md each contain exactly one new step
   referencing `inoculate-preamble`, and `scripts/self-push.sh` (or direct ff
   push) lands the build-skill edit.
6. Smoke: piping `inoculate-inject wrap` output into `wc -c` succeeds with no
   SIGPIPE error.

## Out of scope

Proving the child actually *loaded* the preamble (that's inoculate-carrier-check)
and recording which strain an action carried (inoculate-attest).
