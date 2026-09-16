# PRD — hermetic-build runs its cargo locally, whatever the lane routes

- Status: queued
- build_target: rust-extend
- build_into: /home/jsy/wintermute/rustbuild
- build_priority: high
- publish: j0yen/private
- test_prefix: hermetic_route
- deferred_acs: [7]
- deferred_ac_reasons: {"7": "AC7 requires routing re-enabled through burst-lane.sh enable, which per standing operator policy only a human/later session runs (never hand-set by a build dispatch) — R1-R4 (ACs 1-5) are proven by fake-cargo tests and R5 (AC6) is proven against build-skill's real burst-lane-bin/cargo shim on RedBaron with BUILD_BURST_ENABLED=1 scoped to that one test subprocess; AC7's own gate-goes-green observation is deferred to whoever runs the sanctioned re-enable."}
- Vision: visions/buildloop-operations.md
- Grounding: failure-derived — ten whys in visions/buildloop-operations.md ("2026-09-16 — a red gate on every mcphost branch…"), levels 1 and 9
- PM: Joe Yen
- Drafted: 2026-09-16
- Engineering target: extend `~/wintermute/rustbuild` — `autobuilder/crates/extended-gates/src/producers/hermetic_build.rs` (the `Command::new("cargo")` at line 391, receipt fields at 109–118, ignore rules at 381–384), tests beside `hermetic_scope_ac*.rs`

## TL;DR

The hermetic-build producer spawns `cargo build --offline` and blocks on any new non-loopback socket the build tree opens. Since the burst lane became mandatory at 23:38 EDT on 2026-09-15, that `cargo` resolves to the lane's shim, which opens ssh and hcloud connections to the box, so the producer blocked every mcphost branch (63 blocker lines on 2026-09-16, zero before). Allow-listing those sockets would be wrong: the build itself would then run on the box, unobserved, and the check would prove nothing. The fix is for this one producer to pin its cargo child to the local route, record that in its receipt, and block with a distinct cause if the shim routed anyway.

## Problem statement

**Who:** every Rust branch in the build loop, and the operator who reads `blockers=hermetic-build` as "the build phoned home".

**What:** `hermetic_build.rs:391` runs `Command::new("cargo")` with the gate's environment. Under `BUILD_BURST_ENABLED=1` and `BURST_LANE=1`, `cargo` on PATH is `cargo-budget-bin/cargo → burst-lane-bin/cargo`, which ships the build to the burst box over ssh. The producer's socket snapshot then sees ssh (port 22 to the box) and hcloud (443) from its own process tree and blocks. Journal, 2026-09-16 06:20:17Z: `gate-block attempt=1 blockers=vti-plan,reviewer-agent,hermetic-build`; burst-lane log attribution `ssh/hcloud-sockets-to-2.28.58.3`.

**Why they cannot today:** the producer has no notion of routing. Its ignore rules are the two static ones, `loopback` and `unix-domain` (`hermetic_build.rs:384`), and the shim gives the parent no signal that the build left the machine.

**Consequence, measured:** 6 of 6 in-flight mcphost branches red for 12 ticks; zero mcphost ships since 2026-09-15 16:57Z; two burst boxes billed with nothing to show. Routing was switched off by hand at 14:31Z on 2026-09-16 so branches can go green locally, which removes the lane from the loop until this ships.

## Goals

1. hermetic-build's verdict means what it says: the local build tree opened no network socket.
2. The producer works identically with routing on or off, and its receipt says which route its cargo took.
3. A shim that routes despite the pin is a named block, not a silent pass or a misattributed egress.

## Non-goals

- Allow-listing destination hosts or ports in hermetic-build.
- Changing the shim's precedence rules (documented by `cargo-route-precedence-selftest.sh` in build-skill).
- Routing any other producer differently.

## User stories

1. **Build loop** — When routing is mandatory, hermetic-build still builds locally and passes on a clean tree.
2. **Operator** — When I read a hermetic-build block, it is about the code's egress, or it says `route-not-local` and points at the shim.
3. **Builder** — When I change the shim's env contract, a test with a fake cargo that records its env fails loudly.

## Requirements

**P0 — R1. Local pin.** The producer sets `BUILD_BURST_ENABLED=0` and `BURST_LANE=0` on the cargo child's environment and unsets `BURST_LANE_SH`, which the shim honors as "run locally" per build-skill's route precedence. The receipt gains `cargo_route: "local-pinned"`.

**P0 — R2. Route attestation.** After the child exits, the producer reads the gate's `target/autobuilder/route.log` (when present) for lines newer than the child's start with ` burst ` in the route column. If any exist, the verdict is `block` with `cause: "route-not-local"` and `attributed_sockets` left as observed, so the operator sees the shim ignored the pin rather than a phantom egress.

**P0 — R3. Receipt clarity.** Receipt `ignore_rules` stays `["loopback","unix-domain"]`; new fields `route_pinned: true` and `route_observed: "local"|"burst"|"unknown"`. The producer's journal line (`hermetic-build: cargo exit=…, N attributed socket(s) …`) appends `route=<observed>`.

**P0 — R4. Fake-cargo tests.** A test installs a fake `cargo` on PATH that (a) writes its environment to a file and exits 0, asserting `BUILD_BURST_ENABLED=0` and `BURST_LANE=0`; (b) opens a TCP connection to the host's own non-loopback address, asserting the producer blocks with the socket attributed; (c) writes a ` burst ` line into a temp `route.log`, asserting `route-not-local`.

**P1 — R5. Real-shim test on RedBaron.** With build-skill's real `burst-lane-bin/cargo` first on PATH and `BUILD_BURST_ENABLED=1` in the parent env, run the producer on a tiny crate: receipt shows `route_observed: local`, verdict pass, and `route.log` gains a `local` line, not `burst`.

**Non-functional.** No new dependencies. The route.log read is bounded to the last 2,000 lines.

## Success metrics

| metric | baseline (2026-09-16) | target | method | timeframe |
|---|---|---|---|---|
| hermetic-build blocks with routing on, clean tree | 63 lines, 6/6 branches | 0 | journal grep after routing is re-enabled | first 24 h after re-enable |
| hermetic-build receipts naming their route | 0% | 100% | receipt field present | at ship |
| mcphost branches unblocked from this family | 0 of 6 | 6 of 6 | manifest blockers | first tick after land |

## Technical considerations

- Env names come from the shim: `BUILD_BURST_ENABLED`, `BURST_LANE`, `BURST_LANE_SH`, `BURST_LANE_ENV_FILE` (grep of `burst-lane-bin/cargo`, 2026-09-16). Confirm precedence against `cargo-route-precedence-selftest.sh` before relying on which one wins; set all that mean "local".
- `route.log` lives at `<worktree>/target/autobuilder/route.log`; extend-gate's route attestation (canary-invariant R13) already reads it, so the format is stable: one line per cargo invocation with a route column `local|burst`.
- The producer already tree-scopes attribution (PRD-rustbuild-hermetic-scope, `hermetic_scope_ac2`), so the shim's ssh is correctly attributed today; the fix is to make the tree not need ssh.

## Migration / compatibility

- Receipt fields are additive. Existing receipt consumers ignore unknown keys.
- Rollback: revert; blocks resume when routing is re-enabled.

## Open questions

| question | owner | due |
|---|---|---|
| Should other producers that spawn cargo (cold_build_time, bench_delta) pin local too, for comparable numbers? | Joe | after ship |

## Acceptance criteria

1. P0 — Given a fake `cargo` that dumps its env, When hermetic-build runs under a parent env with `BUILD_BURST_ENABLED=1 BURST_LANE=1`, Then the dump shows `BUILD_BURST_ENABLED=0` and `BURST_LANE=0` and no `BURST_LANE_SH`, and the receipt has `cargo_route: "local-pinned"`.
2. P0 — Given a fake `cargo` that opens a TCP connection to the host's non-loopback address, When the producer runs, Then the verdict is `block`, the socket is attributed, and `route_observed` is `local`.
3. P0 — Given a fake `cargo` that appends `<ts> burst cargo build` to `target/autobuilder/route.log`, When the producer runs, Then the verdict is `block` with `cause: "route-not-local"` and the journal line ends `route=burst`.
4. P0 — Given a quiet fake `cargo` and no route.log, When the producer runs, Then the verdict is `pass`, `route_observed` is `unknown`, and `ignore_rules` is unchanged.
5. P0 — Given the existing `hermetic_scope_ac1..ac4` tests, When `cargo test -p extended-gates` runs, Then they still pass unchanged.
6. P1 — Given RedBaron with build-skill's real shim first on PATH and `BUILD_BURST_ENABLED=1`, When the producer runs on a one-file crate, Then the receipt shows `route_observed: "local"`, verdict `pass`, and the changelog quotes the receipt.
7. P1 — Given the shipped producer and routing re-enabled through `burst-lane.sh enable`, When the next mcphost branch gate runs, Then its blockers list does not contain `hermetic-build`; the changelog quotes the gate line.
