# PRD: warrant-audit — the live read and the assertion runner

Status: Draft v0.1
build_target: rust-extend
build_priority: high
build_into: /home/jsy/wintermute/warrant
Vision: visions/warrant.md

## TL;DR

warrant-corpus classifies close notes into typed `CloseClaim`s behind a
`CloseSource` trait, but only over `FakeSource` fixtures and without ever
running a check. **warrant-audit** adds the live half: the real
`CloseSource` over `~/wintermute/autobuilder/PRDs-archive/` (plus docket if
present, fail-open), a declarative `warrants.toml` registry mapping each
`MechanismAsserted`/`Superseded` close to a **side-effect-free assertion**,
and a runner that executes each and emits a `WarrantVerdict`
(`Proven`/`Refuted`/`Unproven`/`Unwarranted`). The first warrant shipped is
the one that proves `PRD-ctrace-session-end-resilient.md`'s 2026-06-02 close
is **`Refuted`** — by grepping the live SessionEnd hook and reaper verdict.

## Why this exists

Phase 1 evidence (2026-06-06, this `/dream` pass):

- The false close is real and live-verifiable *right now*:
  `~/.claude/scripts/ctrace-session-end.sh` still shells the slow
  `summarize-ctrace-session.sh` (never `scribe`) and never fires on SIGKILL;
  `ctrace-orphan-reap --apply` renders only on `verdict: orphaned-tracer`;
  `journalctl --user -u ctrace-reap.service` shows `healthy → nothing to do`.
  An assertion that greps these is a *deterministic, re-runnable* proof the
  close is false — exactly what no tool runs today.
- warrant-corpus (this repo's root PRD) ships `classify` + the `CloseSource`
  trait + `AssertionSpec` + `WarrantVerdict`, but explicitly defers the real
  source and the runner to this PRD. The strict-chain rule
  (`do not start a rust-extend until the root repo ships`) that bit
  relay/concord/quicken/keel/anchor applies: this PRD must build *after*
  warrant-corpus exists.
- The `coda-audit`/`anchor-probe` PRDs establish the live-read shape this
  reuses: real store over the live dir, `--format json`, read-only, per-item
  failure counted not fatal.

## What this builds

`rust-extend` into `~/wintermute/warrant/`:

- **`FsDocketSource`** implementing `CloseSource`: reads `Status: Closed`
  PRDs from `~/wintermute/autobuilder/PRDs-archive/*.md` (path injectable for
  tests; defaults to the live archive). If a `docket` binary/store is present,
  also pulls closed docket entries; **fail-open** if absent
  (`self_build_jq_escape_reads_absent` — an absent/garbled docket must not
  abort the audit, just contribute zero rows).
- **`warrants.toml` loader** — a registry keyed by close source
  (`[[warrant]] source = "PRD-ctrace-session-end-resilient.md"` + an
  `assertion` table matching one `AssertionSpec` variant). Ships seeded with
  the `session-end-resilient` warrant: a `GrepAbsent` /`CommandExit`
  composite asserting the *claimed* mechanism holds — and therefore
  resolving `Refuted` on today's system, because
  `ctrace-session-end.sh` greps positive for `summarize-ctrace-session.sh`
  (not `scribe`) and `ctrace-orphan-reap --json` reports a non-`orphaned-tracer`
  verdict (so `render_log` never runs).
- **Assertion runner** — executes each `AssertionSpec` under a contract: an
  assertion must be **side-effect-free** (read/grep/exit-code only); the
  runner documents and, where cheap, sandboxes this (no `--apply`-style
  flags permitted in a `CommandExit` cmd; a denylist of mutating verbs is
  rejected at load with a clear error). Each run yields
  `Proven` (assertion confirms the mechanism delivers the outcome) /
  `Refuted` (assertion shows it does not) / `Unproven` (assertion errored or
  was inconclusive). A close in the corpus with **no** registry entry →
  `Unwarranted`. Per-assertion failure is counted, never fatal (one bad
  warrant must not block the other 40).
- **CLI:** `warrant audit [--archive <dir>] [--registry <toml>] [--format
  json] [--status refuted|unwarranted|all]` prints a human table (source │
  kind │ status │ evidence) + a tally, or stable JSON. Default lists all;
  `--status refuted` is the actionable view. `sigpipe::reset()` already in
  `main()` from warrant-corpus.

## Acceptance criteria

1. `cargo build --release` + `cargo test` green in `~/wintermute/warrant/`
   with `warrant audit` wired; warrant-corpus's ACs remain green (no
   regression to the pure core).
2. `FsDocketSource` reads real `Status: Closed` PRDs from an **injected**
   fixture archive dir in tests (never the live FS during `cargo test` —
   cloud-build-safe); a test with a fixture archive containing the
   `session-end-resilient` close + a `warrants.toml` seed produces a
   `WarrantVerdict { status: Refuted, … }` for it.
3. The assertion runner rejects an `AssertionSpec::CommandExit` whose `cmd`
   contains a mutating verb (`--apply`, `rm`, `>`, `write`, …) at registry
   load, with a clear error — proving the side-effect-free contract is
   enforced, not just documented. Asserted in a test.
4. A close present in the corpus but absent from `warrants.toml` resolves to
   `Unwarranted` (not `Unproven`); a test asserts the distinction.
5. `warrant audit --format json` emits a stable serde shape (array of
   `WarrantVerdict` + tally); snapshot-tested. `--status refuted` filters to
   refuted-only.
6. Per-assertion failure is non-fatal: a fixture with one deliberately
   broken warrant + one good warrant returns a verdict for **both** (the
   broken one `Unproven`), exit code 0; asserted in a test.
7. The integration-test entry file (`tests/audit.rs`) appears in `cargo test`
   output as `Running tests/audit.rs` (`self_orphaned_mock_tests`).
8. **[live]** Run on the laptop, `warrant audit --status refuted` reports the
   real `session-end-resilient` close as `Refuted` with evidence citing the
   live `ctrace-session-end.sh` / reaper verdict. (deferred_acs candidate: a
   cloud box has no `~/.claude/scripts/` nor `ctrace` — gate this AC, keep
   1–7 cloud-safe.)
