# PRD: warrant-corpus — the close-claim corpus and pure classifier

Status: Draft v0.1
build_target: rust-cli
build_priority: high
build_into: /home/jsy/wintermute/warrant
Vision: visions/warrant.md

## TL;DR

A PRD closed with *"outcome achieved live by a different mechanism"* retires
a finding, but nothing ever parses, counts, or re-checks those closing
claims — so a false one (verified live this pass:
`PRD-ctrace-session-end-resilient.md`'s claim that `ctrace-reap.timer`
backfills summaries — it does **not**, the reaper renders only on a *live
orphaned tracer*, which `session.bt` self-termination now prevents) sits
undetected until a whole new fleet (`coda`) rediscovers the symptom from
scratch. **warrant-corpus** is the root of the `warrant` toolkit: a new
`~/wintermute/warrant` rust-cli that defines the close-claim domain model
and a **pure** classifier turning raw close notes into typed `CloseClaim`s,
behind a `CloseSource` trait so the live read (warrant-audit) and the tests
share one engine.

## Why this exists

Phase 1 evidence (2026-06-06 ~08:30 UTC, this `/dream` pass):

- `~/wintermute/autobuilder/PRDs-archive/PRD-ctrace-session-end-resilient.md`
  carries `Status: Closed (2026-06-02) — outcome achieved live by a different
  mechanism` and asserts the reaper's `render_log` step "summarizes any
  orphaned `*.ndjson`." Verified false this pass: `ctrace-orphan-reap --apply`
  renders only on `verdict: orphaned-tracer`; the live
  `journalctl --user -u ctrace-reap.service` shows `verdict: healthy` →
  `apply: state is healthy; nothing to do`, and the service comment states
  `session.bt now self-terminates on root exit, so this should normally find
  nothing`. The render path is dead for the common SIGKILL case.
- The cost of nobody parsing that close: the gap reopened to 623/1874 logs
  (33%) by 2026-06-05, forcing the entire `coda` vision + 4 PRDs to be
  drafted from scratch (`visions/coda.md`).
- It is the **second** false-mechanism close caught by live verification in
  one day (the first: `assay`/`onramp`'s futile `claude-agentns-wrap`). The
  recurrence is the motivation; `feedback_verify_before_concluding` is the
  standing memory it keeps violating.
- The `coda-sweep` / `anchor-roots` PRDs in this same repo family establish
  the exact shape this PRD reuses: a new repo whose root PRD ships the
  workspace, shared types, a store trait, a `Fake` store, and one pure
  classifier function asserted to do zero IO.

## What this builds

New Cargo workspace at `~/wintermute/warrant/` with a `warrant` binary
(`warrant-cli`) and a `warrant-core` lib crate.

**Domain types** (`warrant-core`):

- `ClaimKind` — `AcsMet` (self-evident close) | `MechanismAsserted` (the
  "achieved live by a different mechanism" / "delivered by M" shape) |
  `Superseded` (closed "by `PRD-x` / vision-y") | `LiveDeferred`
  (`deferred_acs` present) | `Unclassified`.
- `CloseClaim` — `{ source: CloseRef, close_date: Option<String>, kind:
  ClaimKind, mechanism_text: Option<String>, outcome_text: Option<String>,
  raw_excerpt: String }`. `CloseRef` is `Prd(PathBuf)` or `Docket(String)`.
- `Warrant` — `{ source: CloseRef, assertion: AssertionSpec }` (the spec
  type is *defined* here, *run* in warrant-audit). `AssertionSpec` enum:
  `CommandExit { cmd, expect_code }` | `FileExists { path }` |
  `GrepAbsent { path, pattern }` | `CounterNonzero { path, key }`.
- `WarrantStatus` — `Proven` | `Refuted` | `Unproven` | `Unwarranted` |
  `NotApplicable`.
- `WarrantVerdict` — `{ source, kind, status, evidence: String }`.
- `AuditPlan` — the pure result over a corpus: `Vec<CloseClaim>` plus a
  per-kind tally; serde-serializable for `--format json` parity downstream.

**`CloseSource` trait** — `fn close_notes(&self) -> Result<Vec<RawClose>>`
where `RawClose { reference: CloseRef, text: String }`. The real
filesystem/docket impl lives in warrant-audit; this PRD ships only
`FakeSource` (in-memory fixtures, including verbatim excerpts of the
`session-end-resilient` and `agentns-wrap` close notes).

**Pure classifier** — `pub fn classify(notes: &[RawClose]) -> AuditPlan`.
No IO, no clock, no env: it only pattern-matches close-note prose into typed
claims. Recognizes, at minimum: `Status: Closed … "by a different
mechanism"` → `MechanismAsserted`; `superseded` / `Closed … by PRD-` /
`by vision-` → `Superseded`; a `deferred_acs:` line → `LiveDeferred`; ACs
all met / no mechanism phrase → `AcsMet`.

**CLI:** `warrant list [--format json]` wires `FakeSource` (v0.1; the real
source arrives in warrant-audit) through `classify` and prints the corpus +
tally. `sigpipe::reset()` is the first line of `main()`
(`self_sigpipe_panic_toolkit` — `warrant list | head`).

**Conventions:** MSRV 1.85, no let-chains (recall baseline-gate discipline).
`rust-toolchain.toml` pins 1.85. Inward toolkit → publishes as a `j0yen`
repo (sibling of `coda`/`quicken`/`assay`). Cloud-build-safe: no live FS in
this crate's tests — everything behind `FakeSource`.

## Acceptance criteria

1. `cargo build --release` and `cargo test` are green in `~/wintermute/warrant/`;
   the workspace exposes a `warrant` binary and a `warrant-core` lib crate.
2. `classify` is **pure**: a unit test passes a `FakeSource`-derived
   `&[RawClose]` and asserts the function performs **zero** `CloseSource`
   calls (the source is consulted by the caller, not by `classify`) and reads
   no file/clock/env. (Mirrors `coda-sweep`'s zero-store-call test.)
3. Given a fixture containing the verbatim `session-end-resilient` close
   note, `classify` tags it `MechanismAsserted` and captures the reaper
   mechanism text; given the `agentns-wrap` futile-close fixture, it tags
   that `Superseded`/`MechanismAsserted` appropriately. Asserted in a test.
4. `warrant list --format json` emits a stable, serde-serialized `AuditPlan`
   (claims + per-kind tally); a snapshot test pins the shape.
5. `warrant list` (human form) prints a readable table and the per-kind
   tally; `warrant list | head` does **not** panic (sigpipe reset verified).
6. The integration-test entry file (`tests/corpus.rs`) appears in
   `cargo test` output as `Running tests/corpus.rs`
   (`self_orphaned_mock_tests` — no orphaned mock subdir).
7. `clippy.toml` + `deny.toml` present; `cargo clippy` clean at the repo's
   own gate (recall baseline discipline — not the upstream `-D warnings`
   debt). README documents the `warrant` toolkit and this root crate.
