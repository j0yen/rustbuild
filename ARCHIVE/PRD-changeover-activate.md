# PRD: changeover-activate

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/rollout
Vision: visions/changeover.md
Depends: PRD-changeover-proof-seed.md, PRD-rollout-selfreview-apply.md (blocked, user-gated)

## TL;DR

The problem: with claims held (`changeover-daemon-claims`) and a green
proof ledger seeded (`changeover-proof-seed`), every piece needed to roll
the stale voice fleet to HEAD without dropping a conversation exists — but
nothing ever runs `rollout apply --auto`. Self-review's fleet-staleness
playbook forbids it as an immutable guardrail, and there is no autonomous
loop that probes, records, applies, and *verifies the voice loop
survived*. This PRD builds that loop: a systemd-user timer that runs
probe → record-proof → `apply --auto` (warm-swap only, turn-aware quiet
window), gated behind `ROLLOUT_AUTO_ENABLED=1`, followed by live
post-swap verification that all four daemons re-hold their claims and a
synthetic voice turn round-trips — recording a receipt. It ships in a
**dormant** posture (`ROLLOUT_AUTO_ENABLED=0`, dry-run only) until the
blocked `PRD-rollout-selfreview-apply.md` reaches jsy's explicit
approval, so it never restarts a daemon autonomously before the policy
unlock lands.

## Why this exists

Phase-1 research, 2026-06-13:

- `self-review/SKILL.md` lines 303, 852, 864: fleet-staleness is
  **escalation-only**; self-review "never runs `rollout apply`
  autonomously" and emits `rollout plan` only. The rationale cited is the
  hard-restart deafness window — which warm-swap (v0.6.0) + daemon-claims
  now eliminate. The reconciliation of that guardrail is already drafted
  as `PRD-rollout-selfreview-apply.md`, which the CLAUDE_SELF changelog
  records as **BLOCKED** ("classifier blocked autonomous SKILL.md
  guardrail edit; needs explicit user approval"). This PRD depends on that
  unlock; it does not redraft it.
- `rollout apply --auto` (v0.7.0) requires a `ROLLOUT_AUTO_ENABLED=1`
  interlock and a green ledger — both must be deliberately turned on,
  which is the safe seam to ship dormant.
- rollout v0.5.0 already provides a turn-aware voice guard (defers
  voice-daemon restarts mid-turn). Activation must reuse it so a swap
  never fires while the user is mid-conversation.
- [[feedback_verify_before_concluding]]: do not declare the loop safe
  because it *ran* — instrument the actual voice path and prove a turn
  round-trips after the swap. Post-swap verification is a first-class AC,
  not an afterthought.
- boot-telemetry (`wm-audio boot-time`) and the live turn loop
  ([[project_voice_input_null_detectors]]) give a concrete "voice
  survived" signal to assert against.

## What this builds

In `~/wintermute/rollout` + `~/.config/systemd/user/`:

- A `rollout cycle` (or `apply --auto --verify`) flow that, in order:
  1. `rollout prove --all` (from proof-seed) to refresh the ledger,
  2. `rollout apply --auto` restricted to the **warm-swap** strategy and
     the turn-aware quiet window — never the hard-restart fallback (a
     daemon whose warm-swap can't be confirmed is skipped, not
     hard-restarted),
  3. **post-swap verification**: confirm each rolled daemon re-holds its
     `agorabus://daemon/<unit>` claim and a synthetic voice turn (or the
     daemon's healthcheck + a bus round-trip) completes, then record a
     receipt (timestamp, daemons rolled, window ms, events lost, verify
     pass/fail) under `~/.local/state/rollout/` or the existing receipt
     path.
- A `changeover-activate.timer` + oneshot `.service` under
  `~/.config/systemd/user/`, **disabled by default**, that runs
  `rollout cycle`. The service reads `ROLLOUT_AUTO_ENABLED`; with it unset
  or `0`, the cycle runs in `--dry-run` (probe + plan + would-verify,
  zero restarts) and logs what it *would* do.
- A documented one-command enable path for jsy
  (`ROLLOUT_AUTO_ENABLED=1` + `systemctl --user enable --now
  changeover-activate.timer`) that is **not** taken by the build — the
  PRD ships the capability dormant.

Out of scope: the self-review SKILL.md guardrail edit (separate blocked
PRD), the claim primitive and daemon wiring (their own PRDs), mic-handoff.

## Acceptance criteria

1. `rollout cycle` runs prove → `apply --auto` (warm-swap only) → post-swap
   verify, in that order; a daemon whose warm-swap cannot be confirmed is
   skipped (logged), never hard-restarted by this path.
2. Post-swap verification asserts each rolled daemon re-holds its claim
   and a voice-turn / bus round-trip completes; a verify failure is
   recorded and surfaced (non-zero exit on the cycle), not swallowed.
3. Every cycle writes a receipt (timestamp, daemons rolled, per-daemon
   window ms + events lost, verify result) to a documented path.
4. With `ROLLOUT_AUTO_ENABLED` unset or `0`, `rollout cycle` performs
   **no restarts** — it runs probe + plan + would-verify in dry-run and
   logs the intended actions (test asserts zero `systemctl restart` /
   warm-swap invocations in this mode).
5. With `ROLLOUT_AUTO_ENABLED=1` and a green ledger, the cycle warm-swaps
   only the daemons with a fresh Allow proof and respects the turn-aware
   quiet window (deferring when a turn is active) — verifiable against a
   stub bus/fixture.
6. `changeover-activate.timer` + `.service` are provided, **disabled by
   default**, `systemctl --user --dry-run`-installable, and documented with
   the explicit jsy-only enable command; the build does not enable them.
7. `cargo test` green; no new clippy warnings beyond the documented red
   baseline; CHANGELOG documents `rollout cycle`, the dormant timer, and
   the dependency on `PRD-rollout-selfreview-apply.md` for the policy
   unlock.
