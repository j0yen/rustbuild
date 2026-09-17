# PRD — build-burst-gate-canary-invariant: the box proves it passes the gate, in three variants, before and while branches route to it

- Status: queued
- build_target: shell
- build_into: /home/jsy/wintermute/build-skill
- build_priority: high
- publish: none
- test_prefix: canary
- Vision: visions/buildloop-operations.md
- Depends-on: PRD-build-gate-route-parity-ledger.md
- PM: Joe
- Drafted: 2026-09-15
- Grounding: failure-derived — five-whys in visions/buildloop-operations.md (2026-09-16 canary section, level 4): `prove` proves cargo, never the gate; the only box-level gate proof was a hand-typed `gate-launch.sh` on 09-15 16:07Z and 09-16 02:57Z
- iter_log: 2026-09-16T03:15:00Z amended by /dream (open-questions recommendations, Joe: "apply") — R1 HEAD fallback to last green tag with head_source; R7 once-per-box-day + skip-when-recent-routed-pass, confirm-twice before disable, auto re-enable; open questions resolved; AC11–14 appended, AC1–10 unchanged
- iter_log: 2026-09-16T05:49:46Z amended by /dream (Joe: "amend", five-whys 2026-09-16 05:45Z) — R13–R16 interface-contract invariants that block (route attestation, unbounded same-target cap under burst, producer cargo attestation) and recorded-output fixtures with a hand-written-fixture lint; AC15–AC20 appended; Engineering target + success metrics + vision addendum extended. Existing ACs 1–14 untouched.
- Engineering target: extend `~/wintermute/build-skill` — `scripts/burst-lane.sh` (`cmd_canary`, hooks in `cmd_up`/`cmd_bake`/`cmd_enable`/`cmd_status`), `scripts/gate-launch.sh`, `scripts/gate-delta.sh`, new `scripts/gate-receipt-diff.sh`, `state/burst-lane/boxes/<id>/canary.json` Amendment 05:55Z adds `scripts/select-guard.sh` (cap-unbounded block), `scripts/extend-gate.sh` (route attestation block, producer cargo attestation), `scripts/cargo-budget.sh` (ledger `parent_step` for producers), `tests/fixtures/burst-status.json` + parity test, `scripts/prd-lint.sh` or a sibling selftest lint (hand-written interface fixture), and one cross-repo edit in `~/wintermute/rustbuild/skill/scripts/extended-receipts.sh` (export `CARGO_BUDGET_PARENT_STEP=<producer>` per producer; commit through `gated-targets.sh`).
- Operator-authorization: Joe 2026-09-15T06:17Z "authorize box ACs. stop deferring without operator authorization. you have it. dont stop -- go!" scope: every real-box AC in this PRD — boot Hetzner burst boxes (ccx43/ccx53, up to BURST_MAX_BOXES) under the existing money guards (ttl, idle-guard, disk floors), run the AC, tear down; do not defer for scope or risk.

## TL;DR

`burst-lane.sh canary` runs the gate on the box against last-known-green main in three variants: `--scope main`, `--scope branch` (a throwaway branch with one no-op commit on top of main), and the delta verdict against the recorded baseline. It diffs the 25 receipts producer by producer against a local baseline run of the same HEAD, writes `canary.json {head, verdicts[3], diverged[], ts}`, and journals one line per variant. `up`, `bake` and `enable` call it before declaring success; a daily timer calls it once per box-day; `status` shows `canary=<pass|diverged>@<age>`. A divergence disables dispatch and names the producer. "Fix casper so it passes all gates" becomes a thing the lane checks, not a thing the operator asks.

## Problem statement

**Who:** the operator (Joe) and the loop's dispatcher.
**What:** they cannot know whether a box will pass the gate until real branches have blocked on it. On 2026-09-15 the box served 209 routed runs and 117 blocks before a hand-launched canary (`gate-launch.sh … --scope main --slug mcphost-gate-debt-a1fcdba`) passed 25/25 at 16:32Z; on 2026-09-16 the operator asked for "canary and variants" by hand again after `casper up`.
**Why they cannot today:** `prove` proves one cargo build (`proof.json`: routed, fresh, image_id); `enable` gates on `proof_routed` only; no gate variant is ever run by the lane, and no receipt diff against a local run exists.
**Consequence:** a day of box time judged on no evidence, the box deleted (17:50Z 09-15) and re-bought (02:51Z 09-16), and every future image or lane change re-opens the question.

Failure under this seed: yes — see the vision section dated 2026-09-16 (canary), why-chain levels 3–4.

## Goals

- Three gate variants on the box, launched through `gate-launch.sh`, against a fixed HEAD.
- A receipt-level diff against a local baseline of the same HEAD, producer by producer, route-aware (PRD-build-gate-route-parity-ledger's `route` field).
- Wiring: `up`, `bake`, `enable` require a passing canary; daily cadence; `status` line; divergence disables dispatch.

## Non-goals

- Fixing any producer. A diverged producer is named; its fix is its own PRD.
- Replacing `prove`. `prove` stays the cargo-level proof and runs first; the canary assumes `proof_routed=true`.
- Running the canary on every routed branch. One HEAD, three variants, on lifecycle events and daily.

## User stories

1. As `enable`, I refuse when the box has no passing canary for the current image, journaling `enable refused (cause=canary-missing|canary-diverged producer=<name>)`.
2. As the operator, I run `burst-lane.sh canary` after a lane change and get three verdict lines and a diff summary within one gate wall (about 20 minutes cold, 6 warm).
3. As the operator, I read `burst-lane.sh status` and see `canary=pass@0.4h head=4f1112d` next to `proof_routed`.
4. As the daily timer, I run the canary once per box-day at low priority and, on divergence, disable dispatch and alarm through `alert-deliver.sh`.
5. As the dispatcher, when the canary is diverged I do not route new runs, and the tick journal says why.
6. As the operator, when no local baseline for HEAD exists I see the canary run one locally at `nice 10` first, never report divergence against nothing.

## Requirements

**P0**
- R1. `cmd_canary [--head <sha>] [--variants main,branch,delta] [--no-baseline]`: resolves HEAD to the newest mcphost main commit whose CI is green (`gh run list` or the last `ci-checks` pass in the journal); when main is red it falls back to the newest tag whose CI was green; it never runs against a red HEAD and refuses with `cause=no-green-head` only when neither exists. The journal and `canary.json` carry `head_source=green-main|last-green-tag|operator` (Joe 2026-09-16: last green tag, never red, refuse only if neither).
- R2. Baseline: `state/burst-lane/canary-baseline/<head>/receipts/*.json` from a local `--scope main` gate at that HEAD; if absent, run it via `gate-launch.sh` with `nice -n 10` and `CARGO_BUDGET_TEST_THREADS=2`, then continue; `--no-baseline` skips the diff and journals `baseline=skipped`.
- R3. Variant main: `gate-launch.sh <mcphost> --head <sha> --scope main --slug canary-main-<ts> --wait` with `BURST_LANE=1`; variant branch: create `canary/<ts>` from HEAD with one empty commit, run `--scope branch`, delete the branch after; variant delta: `gate-delta.sh verdict` against the recorded baseline for that HEAD.
- R4. `scripts/gate-receipt-diff.sh <baseline-dir> <run-dir>`: per producer, compares verdict and route; ignores wall, timestamps, paths under `target/`; prints `producer verdict_local verdict_box route_box same|DIVERGED`; exit 1 when any producer with `verdict_local=pass` has `verdict_box=fail`.
- R5. `canary.json` written atomically under `boxes/<server_id>/` with `{head, image_id, ts, variants:{main,branch,delta}, diverged:[{producer, local, box, route}], baseline_dir}`; journal `canary  <variant>  <pass|block|diverged>  (head=… receipts=25 pass=… block=… diverged=<n> route=burst:<id>)`.
- R6. Wiring: `cmd_up` (after `gate_ready=true`), `cmd_bake` (before snapshot), `cmd_enable` (before writing the drop-in) call `cmd_canary`; a non-pass makes `up` journal `canary-diverged` (box kept, dispatch not enabled), `bake` refuse with `cause=canary`, `enable` refuse with `cause=canary-<missing|diverged>`; `BURST_CANARY_SKIP=1` bypasses with a journaled `by=<user>` for emergencies.
- R7. Daily: `claude-burst-canary.timer` (user unit) fires once per box-day while a session is active, and is skipped with a journaled `canary skipped (cause=recent-routed-pass head=…)` when a routed `--scope main` gate at a green HEAD passed within the last 24 h (Joe 2026-09-16: once per box-day plus lifecycle events, never after-N-runs). On divergence it alarms through `alert-deliver.sh` at once, journals `canary diverged (confirm=pending …)`, and immediately re-runs the diverged variant; only a second consecutive divergence runs `cmd_disable` (`confirm=2/2`). `disable` stops new routing and lets in-flight runs finish. A later passing canary (daily or operator-run) re-enables dispatch without operator action, journaling `canary re-enable (after=diverged@<ts>)` (Joe 2026-09-16: disable after confirmation, auto re-enable).
- R8. `cmd_status` line 2 gains `canary=<pass|diverged|missing>@<age_h>h head=<sha7>`.
- R9. Selftest `tests/canary_ac*.sh` with fixture receipts for pass, one-producer divergence, missing baseline, and no-green-head.
- R13. Route attestation blocks: when `route_intended=burst` and the gate's own `target/autobuilder/route.log` has zero ` burst ` lines at gate end, extend-gate's verdict is `block` with `cause=route-mismatch intended=burst burst=0 local=<n> first_local_cause=<cause>` (today: `probe_emit gate-cargo-route dirty`, gate passes). Evidence: 2026-09-16 03:22Z canary, 0 routed calls, cause `burst-lane-disabled` (build-skill 222088f).
- R14. Cap never unbounded under burst: in `select-guard.sh`, when the burst status probe reports `gate_ready=true` and the same-target cap would still resolve to 999999 (`BUILD_DISTINCT_TARGETS=0`, or neither `.width` nor `.run_slots.cap` present), the second candidate for that target is blocked with journaled `same-target-blocked cause=cap-unbounded-under-burst`, and the tick summary line carries `cap_source=blocked`. `.run_slots.cap` is the canonical key (build-skill fca2c8d); `.width` stays accepted. Evidence: journal 2026-09-16 05:30–05:41Z, 39 `same-target-admit … cap=999999 source=local` lines against a 4-slot box.
- R15. Producer cargo attestation: `extended-receipts.sh` exports `CARGO_BUDGET_PARENT_STEP=<producer>` per producer; `cargo-budget.sh` writes it to the ledger row; extend-gate, for a burst-intended gate, requires one ledger row per producer named in `state/cargo-producers.txt` (initial list: flake-audit, mutation-kill, cold-build-time, bench-delta, msrv-verify, hermetic-build, determinism, semver-check) within the gate window, else `block cause=producer-cargo-unattested producer=<name>`. A producer whose cargo resolves outside the shims never reaches the ledger, which is exactly the 2026-09-16 04:56Z case (rustbuild 93abb03: `PATH=$HOME/.cargo/bin:$PATH` ahead of the shims, all 17 producers local, gate-wedge `budget-exceeded wall=1800s route=local`).
- R16. Fixtures from recorded producer output: `tests/fixtures/burst-status.json` is a recorded real `burst-lane.sh status --json` (secrets and ids redacted to placeholders, key set intact) with a parity test that fails when the live command's key set differs; every selftest that fakes `status --json` (`select-guard-same-target-cap-selftest.sh` AC9 first) loads the fixture and overrides fields, never an inline literal; a lint fails any selftest under `scripts/` or `tests/` that contains an inline JSON literal carrying a `gate_ready` key (`handwritten-interface-fixture`). Evidence: AC9's hand-written `width` fixture encoded the consumer's assumption for a week (build-skill df62b99).
- R17. Knob ownership (2026-09-16 ten whys, level 8): `BUILD_BURST_ENABLED=1` may be written only by `burst-lane.sh enable`, and only after a passing canary (R6). `enable` writes the drop-in and the env file together with a `state/burst-lane/enable.json` record `{ts, canary_verdict, canary_ts, head}`. `burst-lane.sh disable` is the only sanctioned way to clear it. Every tick (`select-tick.sh`) compares the live knob against `enable.json`: a knob set to 1 with no record, or with a record whose canary is older than 24 h or not `pass`, is journaled `ALARM burst-knob-unsanctioned (source=<file>)`, delivered through `alert-deliver.sh gate-red build-loop` (PRD-build-gate-red-alarm-invariant's rule), and the tick runs with routing off for that tick. On 2026-09-15 23:38 EDT the knob was set by hand in two files with no canary verdict and every mcphost gate went red at 02:20 EDT.

**P1**
- R10. `cmd_canary --report` prints the last canary.json as a table.
- R11. Cost line: each canary journals `canary cost (eur=… minutes=…)` through the existing cost path so the box-day cost line includes it.

**P2**
- R12. `--variants` accepts a named proof lane from `agent/proof-lanes.toml` to run only that lane's required commands as a fourth variant.

**Non-functional**
- Canary wall ≤ 1 gate wall + 2 min per extra variant; variants run sequentially (09-15 evidence: 5 concurrent gates drove load to 21 on 16 cores and hung one at 1700 s).
- Never more than one canary in flight per box (`canary.inflight` via the lane's lock helper).

## Success metrics

| metric | baseline | target | method | timeframe |
|---|---|---|---|---|
| hand-launched canaries per box lifecycle | 2 (09-15 16:07Z, 09-16 02:57Z) | 0 | journal `canary` lines with `by=lane` | at dispatch |
| routed runs before the box's gate fitness is known | 209 on 09-15 | 0: canary precedes `enable` | R6, AC5 | at dispatch |
| time from `up` to "box passes all three variants" | unknown, manual | ≤ 25 min cold, ≤ 10 min warm | AC1 timing | at dispatch |
| divergence detection | manual triage (one day, 09-15) | same box-day, producer named | AC3, AC6 | at dispatch |
| silent cross-script interface mismatches (flag name, PATH order, JSON key) reaching production | 3 in one night (2026-09-16: 222088f, 93abb03, fca2c8d) | 0: each blocks a gate or an admission | R13–R15, AC15–AC17, AC20 | at dispatch |

## Technical considerations

- Reuse `gate-launch.sh` unchanged; `--wait` already exists. The canary's own unit name must not collide with tick-launched gates (`gate-canary-<variant>-<ts>-<sha7>`).
- The branch variant's throwaway branch lives only in a temp worktree from `worktree-extend.sh add`; it is never pushed. `--scope branch` scope artifacts (PRD-build-branch-gate-scope-artifacts) count as non-blocking in the diff.
- Route field comes from PRD-build-gate-route-parity-ledger; the diff refuses (`cause=no-route-field`) on receipts without it, so the dependency is enforced by data.
- `gate-delta.sh` already has a committed baseline path; the delta variant reuses it and records `baseline=present|absent`.

## Migration / compatibility

- First run on an existing box: `canary=missing` in status until the operator or the timer runs it; `enable` on a box that is already enabled is not re-gated until the next `enable`/`bake`.
- `BURST_CANARY_SKIP=1` exists for the first dispatch so the lane can ship this PRD through itself.

## Open questions

| question | owner | due |
|---|---|---|
| Daily cadence — RESOLVED 2026-09-16 (Joe): once per box-day plus lifecycle events; skip when a routed green-HEAD main gate passed within 24 h. | Joe | done |
| Divergence — RESOLVED 2026-09-16 (Joe): alarm on first, `disable` on two consecutive, auto re-enable on a later pass. | Joe | done |
| HEAD when main is red — RESOLVED 2026-09-16 (Joe): last green tag, never a red HEAD, refuse only when neither exists. | Joe | done |

## Acceptance criteria

1. P0 — Given a live proven box and green mcphost main, When `burst-lane.sh canary` runs, Then three journal lines `canary main|branch|delta` appear with `pass`, `canary.json` exists under the box's state dir with `diverged=[]`, and wall time is under 25 minutes cold. (Real-box; deferrable only with a justification naming why no box was reachable.)
2. P0 — Given no local baseline for HEAD, When `canary` runs, Then a local `--scope main` gate at `nice -n 10` is run first, `baseline_dir` is populated, and the diff runs against it; Given `--no-baseline`, Then the journal says `baseline=skipped` and no divergence is reported.
3. P0 — Given fixture receipts where `extended-receipts` passes locally and fails on the box, When `gate-receipt-diff.sh` runs, Then it prints `extended-receipts pass fail burst:<id> DIVERGED`, exits 1, and `canary.json.diverged` names the producer and route.
4. P0 — Given fixture receipts identical except wall, timestamps and `target/` paths, When the diff runs, Then every producer prints `same` and exit is 0.
5. P0 — Given a fresh box with `proof_routed=true` and `canary=missing`, When `burst-lane.sh enable` runs, Then it refuses with `cause=canary-missing` and no drop-in is written; When `canary` passes and `enable` re-runs, Then the drop-in is written. (Real-box; deferrable only with a justification naming why no box was reachable.)
6. P0 — Given a canary run whose diff reports a divergence (fixture-injected via `BURST_LANE_GATE_TOOLS_REMOTE_BIN_DIR` breaking one producer's tool on the box), When the daily unit runs, Then dispatch is disabled, the journal carries `canary diverged (producer=… route=burst:<id>)`, and `alert-deliver.sh` was invoked with the producer name. (Real-box; deferrable only with a justification naming why no box was reachable.)
7. P0 — Given a canary in flight, When a second `canary` starts, Then it refuses with `cause=inflight` within 1 s.
8. P0 — Given `burst-lane.sh status` on a box with a passing canary, When it runs, Then line 2 contains `canary=pass@<age>h head=<sha7>`. (Real-box; deferrable only with a justification naming why no box was reachable.)
9. P0 — Given mcphost main red (fixture `gh run list` returning failure), When `canary` runs without `--head`, Then it refuses with `cause=no-green-head` and boots nothing.
10. P0 — Given `tests/canary_ac*.sh`, When `scripts/canary-selftest.sh` runs on RedBaron, Then 0 FAIL.
11. P0 — Given mcphost main red and a tag whose CI was green, When `canary` runs without `--head`, Then it runs against that tag, journals `head_source=last-green-tag`, and `canary.json.head_source` says the same; AC9's fixture has no green tag, which is why it refuses.
12. P0 — Given a routed `--scope main` gate at a green HEAD passed 3 h ago, When the daily timer fires, Then no canary runs and the journal carries `canary skipped (cause=recent-routed-pass …)`; Given the last such pass is 25 h old, Then the canary runs.
13. P0 — Given a first divergence, When the daily unit handles it, Then `alert-deliver.sh` is invoked, the journal shows `confirm=pending`, dispatch is still enabled, and the variant is re-run at once; Given the re-run also diverges, Then `cmd_disable` runs with `confirm=2/2`; Given the re-run passes, Then dispatch stays enabled and the journal shows `canary flake (…)`.
14. P0 — Given dispatch disabled by a confirmed divergence, When a later canary passes, Then dispatch is re-enabled with `canary re-enable (after=diverged@<ts>)` and no operator command was needed. (Real-box; deferrable only with a justification naming why no box was reachable.)
15. P0 — Given a burst-intended gate whose per-gate `route.log` has zero ` burst ` lines at gate end, When extend-gate computes the route attestation, Then the gate verdict is `block` with `cause=route-mismatch intended=burst burst=0 local=<n> first_local_cause=<cause>` and no dirty-only probe; fixture: `BURST_LANE` unset with `BUILD_BURST_ENABLED=1` reproduces cause `burst-lane-disabled`.
16. P0 — Given `select-guard.sh` with a fixture burst status `gate_ready=true` and no `width`/`run_slots.cap`, When five same-target candidates are offered with `BUILD_DISTINCT_TARGETS=0`, Then exactly one is admitted, the rest are journaled `same-target-blocked cause=cap-unbounded-under-burst`, and the summary carries `cap_source=blocked`; Given the same fixture with `run_slots.cap=4`, Then four are admitted with `cap=4 source=burst`.
17. P0 — Given a burst-intended gate and a fixture `extended-receipts.sh` that prepends `$HOME/.cargo/bin` to PATH, When the gate finishes, Then it blocks with `cause=producer-cargo-unattested producer=flake-audit` and the journal names every producer in `state/cargo-producers.txt` with no ledger row; Given the shim-first PATH, Then every listed producer has a ledger row with `parent_step=<producer>` and the gate is not blocked by R15.
18. P0 — Given `tests/fixtures/burst-status.json`, When the fixture parity test runs on RedBaron against the live `burst-lane.sh status --json`, Then it passes when the key sets match and fails naming the missing or extra keys otherwise; `select-guard-same-target-cap-selftest.sh` AC9 loads this fixture and still reports `cap=4 source=burst` and `cap=2 source=burst`. (Real-box for the live half; the fixture half runs anywhere.)
19. P0 — Given a selftest file under `scripts/` or `tests/` containing an inline JSON literal with a `gate_ready` key, When the selftest lint runs, Then it fails with `handwritten-interface-fixture <file>:<line>`; Given the same file loading `tests/fixtures/burst-status.json` and overriding fields, Then the lint passes.
20. P0 — Given the three 2026-09-16 regressions replayed as fixtures (`BURST_LANE` unexported; `$HOME/.cargo/bin` ahead of the shims; status JSON with `width` only and with neither key), When `scripts/canary-selftest.sh` runs, Then AC15, AC17 and AC16 each catch their case and the suite reports 0 FAIL.
21. P0 — Given `BUILD_BURST_ENABLED=1` in the drop-in and no `state/burst-lane/enable.json`, When `select-tick.sh` runs, Then the journal has `ALARM burst-knob-unsanctioned (source=burst.conf)`, the notify stub receives it under rule `gate-red`, and the tick's admitted gates run with `BUILD_BURST_ENABLED=0`.
22. P0 — Given `burst-lane.sh enable` after a canary `pass` (fixture), When it completes, Then `enable.json` records `canary_verdict=pass` with the canary ts and head, both knob files read 1, and the next tick journals no knob alarm.
23. P0 — Given `enable.json` whose `canary_ts` is 25 h old, When the tick runs, Then the knob alarm fires with `cause=canary-stale` and routing is off for that tick.
- iter_log: 2026-09-16T14:55:00Z amended by /dream (Joe 2026-09-16 "FIX IT ALL", ten whys level 8: hand-set knob bypassed the canary) — R17 + AC21–23 appended (knob only via enable after canary pass; unsanctioned knob alarmed via gate-red and routing off for the tick); ACs 1–20 unchanged
