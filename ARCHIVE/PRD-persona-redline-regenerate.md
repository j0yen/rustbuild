# PRD: persona-redline-regenerate — recover gracefully, not robotically

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/wintermute-brain
Vision: visions/persona.md

## TL;DR

When the model leaks a forbidden term, today's redline guard replaces the
*entire* reply with a single canned phrase ("Let me put that a different way —
everything's fine."). That keeps the invariant — no dirty reply ships — but it
is jarring and content-free: Jocelyn asked a real question and got a deflection.
`redline.rs` itself documents the better path as deferred "future work": a
`Regenerate` action that re-issues the request with a hardened addendum naming
the leaked term, and only falls back to the safe phrase if regeneration *also*
leaks. This PRD ships that variant — the guarantee stays absolute, but the
experience becomes a real answer phrased without the wall-words.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The gap is documented in code.** `wintermute-brain/src/redline.rs:39`
  (`enum RedlineAction`) has exactly two variants — `Off` and `SafePhrase` —
  with a doc comment: *"A future iteration may add a `Regenerate` variant that
  re-issues the LLM request with a hardened system addendum naming the leaked
  terms. For now, `SafePhrase` guarantees the invariant… while keeping the
  wiring simple."* This PRD is that iteration.
- **The canned phrase is a poor companion experience.** For a technophobe elder
  the failure mode of "everything's fine" on every leak is a conversation that
  stalls. The hearth/persona vision is about a warm friend, not a guard that
  changes the subject. Regenerate preserves the answer's substance.
- **The enforcement seam already exists.** `redline::enforce()` is called at
  `src/daemon.rs:2277` with `redline_forbidden` + `redline_action` resolved at
  `src/daemon.rs:2060`. The daemon owns the model client that produced the
  reply, so a single bounded re-issue is a localized change at that call site
  plus a new variant in `redline.rs`.

## What this builds

A `rust-extend` of `wintermute-brain`:

- **New variant** `RedlineAction::Regenerate { max_attempts: u8, fallback:
  Option<String> }` in `src/redline.rs`. `max_attempts` bounds re-issues
  (default 1); `fallback` is the safe phrase used if every attempt still leaks
  (defaults to `DEFAULT_SAFE_PHRASE`). Deserializes from a `[persona].redline`
  TOML table; existing `Off`/`SafePhrase` configs keep working unchanged.
- **A pure helper** `redline::hardened_addendum(forbidden_hits: &[Hit]) ->
  String` that builds the system-prompt addendum naming the specific leaked
  terms (e.g. "Do not use these words in your reply: AI, computer. Rephrase as a
  warm friend would."). Unit-testable without a model.
- **Daemon wiring** at `src/daemon.rs:2277`: when the action is `Regenerate`
  and `scan()` finds hits, loop up to `max_attempts`: append
  `hardened_addendum(hits)` to the system prompt, re-issue the *same* user turn
  through the existing model client, re-scan. On a clean result, publish it; on
  exhaustion, substitute `fallback` (the current `SafePhrase` behavior) so the
  invariant "a dirty reply is never published" still holds. Emit the existing
  `wm.persona.redline` observability event, extended with `attempts` and
  `outcome: regenerated|fellback`.
- **Tests:** `hardened_addendum` names every hit term and nothing else;
  `Regenerate` config round-trips through TOML; an `enforce`-level test with an
  injected fake "model" closure (regenerate succeeds on attempt 2 → returns the
  clean reply; regenerate never succeeds → returns the fallback). No live model
  needed for tests.

Versioning: minor bump (additive variant + default-preserving). MSRV 1.85, no
let-chains (per `[[self_recall_baseline_gate_red]]` / brain crate conventions).

## Acceptance criteria

1. `RedlineAction::Regenerate { max_attempts, fallback }` exists and
   deserializes from a `[persona].redline` TOML table; `Off` and `SafePhrase`
   configs still deserialize unchanged (a round-trip test for each).
2. `redline::hardened_addendum(&hits)` returns a string naming every distinct
   forbidden term in `hits` and is empty/neutral when `hits` is empty.
3. With an injected model closure that leaks on attempt 1 but is clean on
   attempt 2, `enforce` under `Regenerate { max_attempts: 1, .. }` falls back
   (1 attempt exhausted), and under `{ max_attempts: 2, .. }` returns the clean
   regenerated reply.
4. With an injected model closure that always leaks, `enforce` under
   `Regenerate` returns the `fallback` phrase and never the dirty reply
   (invariant preserved).
5. The daemon path at `src/daemon.rs:2277` handles the `Regenerate` action,
   publishes a `wm.persona.redline` event carrying `attempts` and `outcome`,
   and compiles with no new clippy `-D warnings` regressions beyond the
   documented baseline.
6. `cargo test -p wintermute-brain` is green (existing tests unaffected).
7. `wmd persona profile show jocelyn` / `apply` continue to work; if a profile
   sets `redline = Regenerate`, the apply path writes it correctly.

## Out of scope

- Activating Regenerate in the live config (that's `persona-deploy-jocelyn`'s
  `--redline` flag, extended to accept `regenerate`).
- Measuring its leak rate (that's `persona-redline-eval`, which should re-run
  once this ships).
- Streaming/partial-reply enforcement — this operates on the complete reply, as
  the current guard does.

## Build note for /build

This is the third `wintermute-brain` `rust-extend` in the persona fleet
(after the shipped redline/profile). Per the standing caution, **serialize or
worktree-isolate** brain extends within a tick — shared build target.
