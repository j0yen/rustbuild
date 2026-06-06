# PRD: warrant-docket — a refuted close becomes a tracked signal

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/warrant
Vision: visions/warrant.md

## TL;DR

warrant-audit can prove a close is `Refuted`, but if that verdict only
prints to a terminal nobody watches, the false close still rots silently —
the exact failure that let `session-end-resilient`'s 2026-06-02 close hide
a 33% summary-debt gap for three days. **warrant-docket** closes the loop:
`Refuted` verdicts (and aged `Unwarranted` ones) are reported to `docket`
under a stable slug (`warrant:<source>`), re-opening the retired finding as
a tracked, edge-triggered signal — so the *next* `/dream` or `/build` sees
*"this close is a lie, here's the evidence"* instead of rediscovering the
symptom from scratch.

## Why this exists

Phase 1 evidence (2026-06-06, this `/dream` pass):

- The whole cost of the false close was *invisibility*: the gap reopened to
  623/1874 logs and nothing flagged that a 2026-06-02 close had become
  false — the `coda` fleet had to re-derive it. A verdict that re-enters the
  `docket` ledger is precisely the "findings get a memory, not just a
  mention" remedy `visions/docket.md` already argues for; warrant is simply a
  *new producer* into that ledger, like self-review.
- `keel-beacon` establishes the edge-triggered-into-docket shape this reuses
  (emit only on transition, fail-open if the sink is absent). The gossip
  `wm.health.*` two-producer / shared-envelope question is still open, so v1
  reports into docket directly (the durable ledger), leaving the bus
  `wm.warrant.*` witness to Fleet 2.
- warrant-audit (this repo) ships the `WarrantVerdict` JSON this PRD
  consumes; strict-chain rule applies — build this only after audit ships.

## What this builds

`rust-extend` into `~/wintermute/warrant/`:

- **`DocketSink` trait** — `fn reopen(&self, slug: &str, summary: &str,
  evidence: &str) -> Result<()>` with a `FakeDocketSink` (records calls) for
  tests and a real impl that shells/links the `docket` producer API. **Fail-open**:
  if `docket` is absent or errors, `warrant report` prints the would-be
  entries and exits 0 (never block on a missing sink —
  `self_build_jq_escape_reads_absent`).
- **Mapping** — each `WarrantVerdict` with `status == Refuted` maps to a
  docket reopen under slug `warrant:<source-basename>` with a one-line
  summary ("close claims mechanism M; assertion shows ¬M") and the audit
  evidence string. `Unwarranted` verdicts older than a threshold (config,
  default off in v1 to avoid noise) may also report; `Proven`/`Unproven` do
  **not** report.
- **Edge-trigger** — a small persisted state file
  (`~/.local/state/warrant/reported.json`, path injectable) remembers which
  `(slug, status)` pairs were already reported, so re-running `warrant
  report` does not re-open an already-tracked finding every tick (monotonic,
  like `keel-beacon` / docket streak semantics). A verdict that flips back to
  `Proven` clears the slug (auto-close).
- **CLI:** `warrant report [--apply] [--registry …] [--archive …]`.
  Default **print-only** (lists what *would* be reported); `--apply` performs
  the docket reopens (the one live-side-effect path, gated exactly like
  `coda-close --apply`). `sigpipe::reset()` inherited.

## Acceptance criteria

1. `cargo build --release` + `cargo test` green in `~/wintermute/warrant/`
   with `warrant report` wired; warrant-corpus + warrant-audit ACs remain
   green (no regression).
2. A `FakeDocketSink` test: given an audit result containing one `Refuted`
   verdict, `warrant report --apply` calls `reopen("warrant:<source>", …)`
   exactly once with the evidence string; a `Proven` verdict produces **no**
   reopen call. Asserted.
3. Edge-trigger: running `warrant report --apply` twice over the same
   `Refuted` verdict reopens **once** (second run is a no-op against the
   persisted state file); asserted with an injected state path.
4. Auto-close: a verdict that was `Refuted` then becomes `Proven` clears its
   slug from the state file (and, with a real sink, would close the docket
   entry); asserted against `FakeDocketSink`.
5. Fail-open: with **no** docket sink available, `warrant report --apply`
   prints the intended entries and exits 0 (does not error); asserted.
6. Default `warrant report` (no `--apply`) is **print-only** — a test
   asserts zero `reopen` calls on the `FakeDocketSink` without `--apply`.
7. The integration-test entry file (`tests/report.rs`) appears in `cargo
   test` output as `Running tests/report.rs` (`self_orphaned_mock_tests`).
8. **[live]** On the laptop, `warrant report` (print-only) lists a
   `warrant:ctrace-session-end-resilient` reopen derived from the real
   `Refuted` verdict, cross-referencing `coda` as the superseding fix.
   (deferred_acs candidate — needs the live archive + a real verdict; gate
   it, keep 1–7 cloud-safe.)
