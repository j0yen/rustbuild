# PRD: rollout-selfreview-apply — pre-fill the one window-guarded command that actually cures staleness

Status: Draft v0.1
build_target: shell
Vision: visions/vigil.md

## TL;DR

Self-review's `fleet-binary-staleness` playbook detects stale voice
daemons every run and pre-fills `rollout plan --only <daemon>` under
"Pending your call." That command **errors today** (no `fleet.toml`) and,
even fixed, only *shows* a plan — the human must then run a second
`apply` step. Once rollout-fleet-gen, rollout-apply-systemd, and
rollout-window-guard-turnaware land, the safe cure is a single command.
This PRD updates the playbook to pre-fill the window-guarded
`rollout apply --only <daemon> --window` for the human to run — **still
human-initiated, never autonomous**. It changes *which command is
suggested*, not *who runs it*.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The recurring finding never gets cured.** Self-review journals
  2026-06-11/12/13 all carry `fleet-binary-staleness` for
  wm-audio/dialog/tts/stt under "Pending your call," each pre-filling
  `rollout plan --only <daemon>`. Run after run, the same four daemons
  stay behind-head.
- **The pre-filled command currently errors.** `rollout plan --only
  wm-audio` →
  `fleet.toml error: cannot read ~/.config/rollout/fleet.toml`. After
  rollout-fleet-gen authors the config, plan works — but plan still
  mutates nothing.
- **The playbook is explicitly two-step-and-stop.** SKILL.md:837-842:
  *"Pre-filled command (plan only, never auto-applied): rollout plan
  --only <daemon> … Do not emit `rollout apply` anywhere — plan-mode
  only."* The act half is missing by design — a design that predates the
  window guard existing.
- **The safety the guardrail was protecting now exists in the tool.**
  The guardrail (SKILL.md:830, *"daemon restarts drop every subscriber …
  must remain explicit human-approved"*) is satisfied differently now:
  rollout-window-guard-turnaware refuses to bounce a voice daemon
  mid-turn, and rollout-apply-systemd uses `systemctl --user restart`
  (Restart=always-aware) instead of a racing SIGTERM. The disruption the
  guardrail feared is now bounded by the tool — so the *human-run*
  command can safely be `apply` instead of `plan`.

## What this builds

A surgical edit to `~/.claude/skills/self-review/SKILL.md`, playbook
`fleet-binary-staleness` (around SKILL.md:820-844):

1. **Change the pre-filled command** in the Pending template from
   `rollout plan --only <daemon>` to
   `rollout apply --only <daemon> --window` (the `--window`/turn-aware
   guard makes a voice bounce safe; `--only` keeps it one daemon).
2. **Preserve the autonomy guardrail, precisely.** Keep "Auto-fix
   conditions: NONE" — self-review still **never** runs `rollout apply`
   itself. The only change is the text of the command handed to the
   human. Update the immutability note to read: *self-review never runs
   rollout apply autonomously; the pre-filled command a human may run is
   the window-guarded apply.*
3. **Guard for the un-built precondition.** Extend the existing "binstale
   not installed" guard: if `~/.config/rollout/fleet.toml` is absent,
   pre-fill `rollout fleet-gen` (review + accept) **first**, then the
   apply — so the playbook degrades gracefully before fleet-gen has been
   accepted on this box.
4. **No behavioural change to detection** — the binstale scan, JSON
   parse, and Snapshot/Pending emission are untouched.

This is a documentation/playbook edit only; no Rust, no new tool.

## Acceptance criteria

1. The `fleet-binary-staleness` playbook's Pending template pre-fills
   `rollout apply --only <daemon> --window` (not `rollout plan`).
2. "Auto-fix conditions: NONE" remains; the text still forbids
   self-review from running `rollout apply` *autonomously*. The change
   is confined to the *suggested human command*, and the SKILL says so
   explicitly.
3. A precondition guard exists: when `~/.config/rollout/fleet.toml` is
   absent, the playbook pre-fills `rollout fleet-gen` (review-and-accept)
   ahead of the apply command, and does not pretend apply will work.
4. The detection half (binstale scan invocation, JSON parsing, exit-code
   handling, Snapshot line on all-fresh) is byte-for-byte unchanged.
5. A grep of the edited SKLL.md shows no remaining `rollout plan --only`
   in the `fleet-binary-staleness` Pending template, and the line
   `Do not emit rollout apply anywhere — plan-mode only` is removed or
   rewritten to the human-run-apply posture.
6. SKILL.md still passes whatever lint/line-cap the self-review skill
   enforces; the edit adds no new playbook, only modifies the existing
   one.

## Build notes for /build

- **Depends on rollout-fleet-gen + rollout-apply-systemd +
  rollout-window-guard-turnaware shipping and being verified.** A
  pre-filled `rollout apply --window` is only safe once all three exist;
  do not ship this PRD before them.
- **Needs Joe's explicit approval.** SKILL.md:830 calls the
  escalate-don't-apply guardrail "immutable." This PRD keeps autonomous
  application forbidden but changes the human-suggested command — a
  deliberate posture change to a block marked immutable. Treat as
  user-gated even though `build_auto` is otherwise the default; the
  vision's Open Questions records this.
- **Serialize on SKILL.md** with any other in-flight vigil self-review
  PRD (e.g. `agorabus-reload-self-review`, `vigil-selfreview-concurrent-guard`)
  — they edit the same file.
