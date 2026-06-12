# PRD: continuity-e2e-attest

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/continuity-attest
Vision: visions/continuity.md

## TL;DR

The continuity vision's end-state is a single claim: *next-Claude can read what
last-Claude did, joined across four kernel-backed signals keyed on one stable
session id.* Every component to make that true is now built — the kernel prctl
fix (on disk), `agentns-claude` (prctl-wired by `PRD-agentns-claude-prctl-wire`),
`provq`, `recall-session-stamp`, `memlog-witness`, and `session-postmortem`.
What's missing is the **proof that they compose**: nothing drives a real wrapped
session through all four signals and asserts the join is non-empty and keyed on
the *same* 128-bit id. `session-postmortem`'s own AC9 is exactly this test,
deferred as a post-boot checkpoint. This PRD builds `continuity-attest` — the
capstone that runs that end-to-end and turns the vision's end-state from
"believed" into "attested," with a committed receipt.

## Why this exists

- `session-postmortem` README + PRD: AC9 ("Real session end-to-end") is
  explicitly deferred — "gates on the upstream continuity fleet (agentns, provq,
  memlog-witness, recall-session-stamp) and is a post-boot live checkpoint."
  Until something *runs* it, the vision's end-state is asserted, not verified —
  precisely the failure mode `feedback_verify_before_concluding` and
  `visions/assay.md` warn against ("a shipped fix the symptom outlived is an
  unverified fix").
- Live this pass (2026-06-12): `session-postmortem latest` →
  `no sessions found under ~/.claude/memlog`; `session-postmortem current` →
  `AGENTNS_SESSION_ID … unset`. The join has never executed against real data
  because no session has ever had a real id. The moment the launcher + reboot
  land, *someone* must prove the four signals actually line up — ids drift,
  fallback forms sneak in, a consumer keys on the wrong field. An attestation
  catches that; a hopeful changelog line does not.
- The continuity vision lists end-state points 1–5 (per-session id, file→session
  attribution, per-session memlog snapshots, session-stamped recall, one-command
  postmortem). This tool is the executable assertion of all five at once.

## What this builds

`continuity-attest` — a single read-mostly CLI that drives a controlled probe
session and verifies the join. It does **not** re-implement any signal reader;
it orchestrates the real tools and checks their agreement.

`continuity-attest run [--keep] [--json]`:

1. Launch a throwaway wrapped child via the real `agentns-claude --intent
   continuity-attest -- <probe-script>`. The probe script, inside the
   namespace, captures its own `/proc/self/agent_session` as the **ground-truth
   id** `S`, then:
   - **provfs signal**: writes a unique probe file under `$HOME`
     (`~/.cache/continuity-attest/probe-<rand>`), closes it.
   - **recall signal**: writes a throwaway recall memory tagged for the run.
   - **memlog signal**: emits a synthetic pre-compaction snapshot to the memlog
     device/path (or triggers the witness) under id `S`.
   - **ctrace signal**: ensures at least one execve is recorded under the
     session (the probe runs `true` a couple of times).
2. After the child exits, assert, for ground-truth id `S`:
   - `provq <probe-file>` → `session == S` (real id, **not** the
     `comm:<comm>:pid` fallback form).
   - `recall list --session S` → contains the probe memory.
   - the memlog snapshot for `S` is present and readable.
   - `session-postmortem S --format json` → all four signal sections present and
     **non-empty**, and every section that carries an id carries `S` (no drift,
     no second synthesized id).
3. Emit a verdict: `CONTINUITY: ATTESTED (id=S)` with a per-signal ✓/✗ table, or
   `CONTINUITY: BROKEN at <signal> — <detail>`. Write a receipt to
   `~/brain/continuity/attest-<S>.json` (and a human `.md` sibling) so the proof
   is durable and self-review can cite it. `--keep` retains the probe artifacts
   for debugging; default cleans them.

Design constraints: pure orchestration over the existing binaries (resolve them
from `$PATH` with env overrides for testing, same pattern as
`session-postmortem`); no privileged ops of its own beyond what the wrapped
launcher already has; every assertion has a precise failure string naming which
signal disagreed and how. Keep the crate's lints strict (no unwrap/panic), MSRV
1.85.

## Acceptance criteria

1. **AC1 — orchestration, not reimplementation.** `continuity-attest` shells out
   to `agentns-claude`, `provq`, `recall`, the memlog reader, and
   `session-postmortem`; it contains no xattr-parsing, ndjson-parsing, or
   memlog-format code of its own. (Grep the source: no `getfattr`/xattr/ndjson
   parsing.)
2. **AC2 — id agreement check.** The verdict is `ATTESTED` only when all four
   signals resolve to the *same* ground-truth id `S`; an injected mismatch
   (fixture where provq returns a different id) yields `BROKEN at provfs` with
   the two ids printed. Unit-tested against fixtures via env-overridden binaries.
3. **AC3 — fallback-form rejection.** If provq returns the
   `comm:<comm>:pid:<n>` fallback form rather than a 32-hex id, the verdict is
   `BROKEN at provfs (fallback stamp — agentns not live)`, not `ATTESTED`.
   Fixture-tested.
4. **AC4 — non-empty join.** `ATTESTED` requires every `session-postmortem`
   signal section to be non-empty; an empty memlog section yields `BROKEN at
   memlog`. Fixture-tested.
5. **AC5 — receipt.** A run writes `~/brain/continuity/attest-<S>.{json,md}`;
   the JSON validates against the committed schema and includes the per-signal
   results and the ground-truth id. `--json` prints the same object to stdout.
6. **AC6 — cleanup.** Without `--keep`, probe artifacts (probe file, throwaway
   recall memory) are removed after the run; with `--keep` they persist and
   their paths are printed. Verified by listing before/after.
7. **AC7 — read-mostly + safe.** The only persistent writes are the receipt and
   (transiently) the probe artifacts; the tool issues no `setcap`, `pacman`, or
   `reboot`, and never mutates the *parent* (attesting) process's namespace.
8. **AC8 [boot] — real ATTESTED.** On a box booted into pkgrel >= 12 with the
   launcher installed+wired, `continuity-attest run` reports `CONTINUITY:
   ATTESTED` with a non-zero `id=S` agreed across all four signals, and the same
   run satisfies `session-postmortem` AC9. **User-gated on reboot + the launcher
   PRDs.** This is the criterion that flips the continuity vision end-state
   (points 1–5) from drafted to verified.
9. **AC9 — README + CHANGELOG.** Documents the probe protocol, the receipt
   format, the signal-agreement model, and that AC8 is the vision's capstone
   checkpoint.

## Boot gating + ordering

Last in the fleet. Buildable now against fixtures (AC1–AC7, AC9); the real
`ATTESTED` (AC8) gates on: reboot into pkgrel >= 12 +
`PRD-agentns-claude-prctl-wire` + `PRD-agentns-launch-flip`. Once AC8 passes,
update `visions/continuity.md` end-state to "verified <date>" and resolve the
`agentns-session-zeros` docket with a link to the receipt.

## Open questions

- **Memlog synthetic snapshot.** Triggering a *real* pre-compaction snapshot
  from a throwaway child may not be possible without a real compaction; the
  probe may need to write directly to the memlog device under id `S` (requires
  memlog-group membership). Confirm whether `memlog-precompact-witness` exposes
  a test/inject path; if not, this PRD adds a tiny `--inject` affordance to the
  witness, or asserts the memlog signal via the witness's own self-test rather
  than a fresh snapshot. Decide in iter-1.
- **Where receipts live.** `~/brain/continuity/` keeps them with the journal
  substrate and lets self-review cite them. Alternative: alongside
  `session-postmortem`'s `~/brain/postmortems/`. Leaning a dedicated
  `continuity/` dir. Confirm with jsy.
- **Cadence.** One-shot (run manually / post-boot once) vs. a periodic
  self-review check that the chain is *still* live after kernel bumps. Leaning
  one-shot for now, with the receipt durable enough that self-review can re-run
  it on demand. A periodic wire is a future `/dream extend continuity` bullet.
