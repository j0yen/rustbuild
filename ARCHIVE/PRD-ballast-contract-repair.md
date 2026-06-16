# PRD: ballast-contract-repair — fix the guard↔survey skew so the SLO loop runs

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/ballast-guard
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-guard (NEVER AtScaleInc)

## TL;DR

The ballast fleet shipped, but the autonomous loop the vision promised is
**dead on arrival**: `ballast-guard run` invokes `ballast-survey --json
--candidates`, and `ballast-survey` v0.3.0 no longer accepts `--candidates`,
so every SLO pass aborts before it measures anything. The guard already knows
how to parse survey's standard `--json` schema and rank candidates itself
(`guard.rs:162-193`); the only defect is that it passes one removed flag. This
PRD removes the dead `--candidates` argument, makes the guard derive its reap
candidates from survey's stable JSON, and adds a smoke test that proves the
guard's own survey invocation parses — so a future survey-schema bump fails a
test instead of silently bricking the disk guard.

## Why this exists

Verified live 2026-06-16:

```
$ ballast-guard run --mount /home
ballast-guard: ballast-survey exited exit status: 2: error: unexpected
argument '--candidates' found
```

The call site is `ballast-guard/src/guard.rs:141`:

```rust
cmd.arg("--json").arg("--candidates");
```

`ballast-survey --help` (v0.3.0, confirmed installed) exposes `--json`,
`--root`, `--min-size`, `--now`, `--no-cloudaware`, `--bin-name` — **no
`--candidates`**. The two crates drifted: guard was authored against a survey
interface that survey v0.3.0 dropped. Result: the disk climbed **86% → 92% →
96% across three consecutive days** (self-review journals 2026-06-14/15/16;
`df` at draft time = 423G/468G, 22G free) while a complete, shipped reaping
toolkit sat inert because its orchestrator could not take its first step.

This is the keystone of the ballast *extension*: nothing else in the loop —
the timer (PRD-ballast-pilot), the trend tracker, the digest — matters until
`ballast-guard run` can complete a single pass.

## What this builds

A surgical `rust-extend` of `ballast-guard` (no new crate).

**1. Drop the dead flag.** In `guard.rs::gather_candidates` (line ~140) and
`guard.rs::estimate_reclaimable` (line ~103), invoke `ballast-survey --json`
only. Remove `.arg("--candidates")`.

**2. Derive candidates from the stable schema.** The guard already deserializes
survey JSON into its `Candidate` list and sorts by `(class asc, size desc)`
(`guard.rs:166-193`). Confirm the field names it reads match survey v0.3.0's
emitted JSON (`reap_safety`/`class`/`size_bytes`/`path` — read survey's actual
output with `ballast-survey --json | jq '.[0]'` during build and align the
serde struct). If survey v0.3.0 renamed a field, update the guard's deserialize
target; do **not** re-add a flag to survey.

**3. Pin the contract with a test.** Add an integration test that runs the real
`ballast-survey --json` (skipped with a clear message if the binary is absent
from `$PATH`, so CI on the cloud box still passes) and asserts the guard's
deserialize succeeds and yields ≥0 candidates without error. This converts the
next schema drift from a silent brick into a red test.

**4. Preserve the SLO exit-code contract.** The documented contract
(`guard.rs:3-7`: 0 ok / 2 warn / 3 breach-acted / 4 breach-unresolved) and the
dry-run/`--apply` posture of the underlying reap are unchanged. This PRD fixes
plumbing only; it must not widen what the guard is allowed to delete.

## Acceptance criteria

1. `ballast-guard run --mount /home` exits without the `unexpected argument
   '--candidates'` error and completes a full SLO evaluation pass.
2. `grep -rn '"--candidates"\|--candidates' src/` returns no matches in
   `ballast-guard`.
3. With disk below the high-water mark, the command exits `0` (ok) or `2`
   (warn) per the existing contract; the exit-code semantics in `guard.rs:3-7`
   are unchanged.
4. A new integration test invokes the real `ballast-survey --json`, parses it
   through the guard's candidate deserializer, and passes; it is skipped (not
   failed) with an explanatory message when `ballast-survey` is not on `$PATH`.
5. `cargo test` is green and `cargo clippy` emits no new warnings versus the
   crate's pre-change baseline.
6. The guard still never deletes a build in flight and still honors the reap
   safety floor — no change to the deletion-selection surface, proven by the
   candidate-selection unit test(s) remaining green.

## Out of scope

- Re-adding `--candidates` to `ballast-survey` (the fix lives in the consumer,
  not by reviving a removed flag).
- Any change to reap's deletion gate, ledger, or safety tiers.
- Wiring the guard to a timer — that is PRD-ballast-pilot.
