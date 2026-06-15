# PRD: answerable-digest-reconcile-bind — tell the principal about your own honesty gap

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/answerable
build_version_bump: minor
Vision: visions/answerable.md
Depends-on: PRD-answerable-reconcile.md

## TL;DR

Fold `reconcile`'s verdict into the consent digest so the non-reading
principal *hears the agent's own honesty gap*, not just a clean summary
of what the agent chose to record. Today the digest reads only the
ledger — the very record that can omit. After this PRD, the spoken
summary becomes: "Today I recorded 14 actions; an independent check found
3 more that I did and did not tell you about." Honesty about one's own
dishonesty is the entire point of an accountability spine.

## Why this exists (Phase 1 evidence, 2026-06-14)

- The Fleet-1 `answerable-consent-digest` shipped: `answerable digest`
  composes a plain-language summary and can `--speak` it to
  `wm.tts.speak` (see `~/wintermute/answerable/src/digest.rs`:
  `compose_summary`, `compose_details`, `speak`/`--wait-ack`).
- But `digest` reads **only the ledger**. Phase-1 inspection showed the
  ledger is empty of real actions (49 self-test lines; 22 unrecorded
  `homeward` pushes on 2026-06-14). A digest over an incomplete ledger
  confidently tells the principal "nothing much happened" while the agent
  shipped two releases. That is the failure mode the whole vision exists
  to prevent — now reproduced *inside* the consent layer.
- PRD-answerable-reconcile adds the independent check
  (`answerable reconcile --json` → omissions/phantoms/matched). This PRD
  consumes that verdict so the spoken word the user hears reflects ground
  truth, not self-report.

## What this builds

Extend `digest.rs` (and its `DigestConfig`) in the `answerable` crate:

- A new flag `answerable digest --reconcile` (default off for
  back-compat). When set, the digest internally runs the same
  reconciliation `answerable reconcile` performs over the digest window
  and incorporates the result.
- `compose_summary` gains a one-sentence honesty clause when reconcile is
  enabled and omissions/phantoms are non-zero:
  - omissions > 0: "An independent check found N action(s) I did not
    record — say 'details' to hear them."
  - omissions == 0 && phantoms > 0: "My record matches what I did, with N
    note(s) for actions I logged but later reversed."
  - all clean: "An independent check confirms my record is complete."
- `compose_details` lists the specific omitted actions (kind + target +
  time) in plain language, after the recorded actions.
- The `--speak` path is unchanged in mechanism; it now speaks the
  reconcile-aware summary when `--reconcile` is set.
- Reconcile failure is non-fatal: if the ground-truth probes error, the
  digest falls back to ledger-only with a spoken/printed caveat ("I could
  not independently verify my record this time") and still completes.

## Acceptance criteria

1. `answerable digest --reconcile --json` over a fixture window where the
   ledger omits one real action includes an `omissions` count of 1 and
   the summary text contains the honesty clause naming one unrecorded
   action (test with fixture ledger + fixture ground truth).
2. With a fully-reconciled fixture (ledger complete), `--reconcile`
   produces the "independent check confirms my record is complete"
   clause and zero omissions (test).
3. `compose_details --reconcile` lists each omitted action with its kind,
   target, and time in plain language (test asserts the omitted target
   string appears in the details output).
4. Without `--reconcile`, digest behavior is byte-identical to the
   pre-PRD output for the same ledger (back-compat test — the existing
   digest tests still pass unchanged).
5. When the reconcile probes fail (e.g. forced IO error), `--reconcile`
   degrades to ledger-only with the "could not independently verify"
   caveat and exits 0 (test).
6. `cargo test --release` green; no new clippy `-D warnings`; version
   bumped to the next minor; `CHANGELOG.md` prepended.
