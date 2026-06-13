# PRD: harbor-thrift — the standing cost of a permanent box is legible

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/constellation-burst-builder
Vision: visions/harbor.md

## TL;DR

A permanent hub changes the cost model: Hetzner bills a box whether it's on or off
— only **deletion** stops billing — so the hub is a steady monthly charge, not a
per-burst one. `cost.log` and the existing `monthly_budget_usd` don't distinguish
the two. This PRD makes the money legible: `wm-burst cost` separates the hub's
standing charge from per-burst pod cost, projects the month, and warns past a cap.

## Why this exists

- `config.rs` already carries `monthly_budget_usd: f64` but nothing computes spend
  against it; `cost.log` is a flat UP/DOWN ledger (528 entries) with no notion of a
  box that *stays up*.
- harbor-hub adds `HUB-UP`/`HUB-DOWN` lines (distinct prefix) — this PRD is the
  consumer that turns those into a standing-cost line item.
- The vision's open question: when projection exceeds the cap, **warn, don't
  auto-teardown** — the hub is stateful (holds the warm cache + git mirror), so an
  automatic destroy is dangerous. This PRD implements warn-only.

## What this builds

**New cost module work (`src/cost.rs`):**
- Parse `cost.log` into two streams: **burst** (existing `UP`/`DOWN` pairs → summed
  duration × per-burst rate) and **hub** (`HUB-UP` → still-running → `now - HUB-UP`
  × standing rate, or `HUB-UP`/`HUB-DOWN` pair).
- A `CostReport { burst_usd, hub_standing_usd, month_to_date_usd,
  month_projection_usd, budget_usd, over_cap: bool }`.
- Projection: hub standing rate × hours-remaining-in-month + month-to-date.
- Per-type rates table (cx22/cpx11/ccx53…) — small static map, default to a
  conservative rate if a type is unknown.

**New command (`src/commands/`, wired):**
- `wm-burst cost` — prints the `CostReport`: this month's burst spend, the hub's
  standing charge, month-to-date, projected month-end, the budget, and a clear
  **WARN** line if projection > budget. Exit non-zero on over-cap so a wrapper/cron
  can surface it.
- `wm-burst cost --json` — the `CostReport` as JSON for the bus/heartbeat.

**No teardown action.** Over-cap warns only (vision decision). A future PRD may add
opt-in auto-pause, but not here.

## Acceptance criteria

1. `cargo build` + `cargo test` pass under rustc 1.85; clippy clean (no new lints).
2. `cost.rs` parses a fixture `cost.log` containing both burst `UP`/`DOWN` pairs
   and a `HUB-UP` line into the correct two streams (unit test asserts
   `burst_usd > 0` and `hub_standing_usd > 0`).
3. A still-running hub (`HUB-UP` with no matching `HUB-DOWN`) is charged from its
   `HUB-UP` timestamp to a passed-in "now" (now is injected, not read from the
   clock, so the test is deterministic).
4. `month_projection_usd` = month-to-date + standing-rate × remaining-month-hours;
   a unit test with a fixed now + a known rate asserts the exact figure.
5. `over_cap` is true iff `month_projection_usd > budget_usd`; `wm-burst cost`
   exits non-zero exactly when `over_cap` and prints a `WARN` line naming the
   projected figure and the cap.
6. `wm-burst cost --json` emits a `CostReport` whose fields parse back to the same
   values as the human output (round-trip test).
7. An unknown server type uses the conservative default rate and the report notes
   "rate estimated for type=<x>" rather than silently undercounting.
8. Existing burst-only `cost.log` (no `HUB-*` lines) still produces a valid report
   with `hub_standing_usd == 0` — no regression for the pre-harbor ledger.
