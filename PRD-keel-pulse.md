# PRD: keel-pulse — can the brain even reach this tier?

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/keel`
**Vision:** visions/keel.md

## TL;DR

The brain's tier ladder discovers a dead cloud tier *by trying to use it* — it
dispatches the rung, eats a round-trip + auth failure, and degrades. There is no
cheap "is this tier reachable and authed at all?" probe, and no shared type for
the answer. **keel-pulse** creates the `keel` workspace and the foundational
surface the rest of the vision extends: the `TierHealth` / `TierStatus` /
`LedgerEntry` types, a `TierProbe` trait, and a probe that answers reachability +
auth *without generating a token* (so it never bills and never blocks on a CPU-
pinned local model). `keel pulse` prints a one-glance health table.

## Why this exists

- `wintermute-brain/src/ladder.rs:91-94` documents the degrade trigger: *"Every
  reachable tier failed (top tier unreachable, or an Anthropic tier with no
  key)."* That determination is made **per turn, by attempting the rung** —
  there is no standalone health check the ladder (or a human) can consult first.
- docket `wm-anthropic-key-empty` is **open since 2026-05-30: 6 runs seen, 8
  reports** (evidence `recall:01KT6VCBX1PSWT9MAQP607TAWY`). Every self-review
  re-derives "cloud tier ladder cannot reach cloud models" by hand because
  nothing computes it on demand.
- `project_brain_local_first_ladder`: the ladder is local-first with cloud
  haiku/sonnet/opus above; `WM_BRAIN_SKIP_TIERS`/`WM_BRAIN_MAX_TIER` already
  configure which rungs exist. A probe must read that same config so it reports
  the *actually-configured* ladder, not a hardcoded one.
- This is the FIRST PRD of the fleet: it creates the repo + shared types + the
  `TierProbe` trait. The relay/concord/quicken passes all learned the hard rule
  — **no rust-extend starts until the corpus/probe repo has SHIPPED** or
  extend-validate fails. keel-pulse is keel's corpus.

## What this builds

New cargo workspace at `~/wintermute/keel` with a `keel` binary crate.

**Core types (the surface ledger/cordon/beacon extend):**

- `TierStatus` enum: `Reachable`, `Unreachable { reason }`, `Keyless`,
  `Exhausted`, `Unconfigured`, `Skipped` (matches a `WM_BRAIN_SKIP_TIERS` entry).
- `TierHealth { tier: String, status: TierStatus, checked_at: i64,
  consecutive_failures: u32 }`.
- `LedgerEntry { tier, tokens_in, tokens_out, est_cost_usd, ts }` — defined here
  (the shared schema) but written/read by keel-ledger; pulse only declares it.
- `Ladder` — the resolved list of configured tiers (local-3b … opus) after
  applying skip/max config, so every keel subcommand agrees on what the rungs
  are.

**The `TierProbe` trait** — `fn probe(&self, tier: &TierConfig) -> TierHealth`.
A local tier probes its OpenAI-compatible endpoint with a *non-generating*
check (TCP connect + `/v1/models` or `/health` GET, never `/v1/chat/completions`)
so it never blocks on a CPU-pinned model. A cloud tier checks key presence then
a minimal authed reachability call that returns 401/402/200 *without* a
completion request (token-count or models endpoint). All network behind the
trait; tests inject a `FakeProbe`.

**`ProbeEnv` trait** for config + env reads (`WM_ANTHROPIC_KEY`,
`WM_BRAIN_SKIP_TIERS`, `WM_BRAIN_MAX_TIER`, endpoint URLs), so tests run with
fixtures and zero real env/network.

**UX:** `keel pulse` → table of `tier | status | checked | fails`;
`--format json` for machines; exit non-zero if the top configured tier is not
`Reachable` (so a hook/self-review can gate on it).

**Deps:** keep minimal — `serde`/`serde_json`, a thin HTTP client already used
elsewhere in the fleet (match `wm-local-llm`'s choice), `clap`. MSRV 1.85, no
let-chains (`self_recall_baseline_gate_red` discipline). `sigpipe::reset()` as
the first line of `main()` (`self_sigpipe_panic_toolkit` — `keel pulse | head`
must not coredump).

## Acceptance criteria

1. `cargo build` and `cargo test` succeed offline (no live network); a test
   asserts the probe path makes **zero** real outbound connections (FakeProbe
   only).
2. `TierStatus`, `TierHealth`, `LedgerEntry`, `Ladder`, `TierConfig` are public
   and `serde`-(de)serializable; a round-trip test covers each.
3. `keel pulse --format json` emits one `TierHealth` per *configured* tier; with
   `WM_BRAIN_SKIP_TIERS=local-8b` set (via `ProbeEnv` fixture) that tier appears
   with `status: "Skipped"` and is not probed.
4. A cloud `TierConfig` with no key (empty `WM_ANTHROPIC_KEY`) yields
   `status: "Keyless"` **without** the probe opening a socket (assert the
   FakeProbe records no connect attempt for that tier).
5. A local `TierConfig` whose endpoint refuses connection yields
   `Unreachable { reason }`; a reachable one yields `Reachable`. The probe uses a
   non-generating endpoint (assert the FakeProbe sees a `/v1/models`-class path,
   never `/v1/chat/completions`).
6. `keel pulse` exits non-zero when the top configured tier is not `Reachable`,
   zero otherwise; covered by two integration cases.
7. `keel pulse | head -1` does not panic (SIGPIPE reset verified by a test that
   closes the read end early).
8. README documents the type surface and the `TierProbe`/`ProbeEnv` traits so
   keel-ledger/cordon/beacon have a contract to extend.
