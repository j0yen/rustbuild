# PRD: christen-detect — classify a session's namespace state, honestly

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/christen`
**Vision:** visions/christen.md

## TL;DR

For twenty-plus self-review runs, the all-zero `/proc/self/agent_session`
reading was mis-narrated as "registration failed" when it actually means
"this session was born in the init namespace — unwrapped, expected until the
launchers are routed." **christen-detect** adds `christen probe`: it reads
the `/proc` agent-namespace surface for the current process (or a target
PID), classifies it as `init` / `live` / `absent` / `malformed` with the
*correct* prose, and edge-triggers a `docket report`/`resolve` against the
open `agentns-session-zeros` finding so the docket stops carrying a
mis-diagnosis and starts carrying the real, actionable state.

## Why this exists

- **The misdiagnosis is documented and costly.** The draft proposal
  `~/wintermute/autobuilder/proposals/self-review-agentns-block.draft.md`
  records that the old SKILL.md prose "caused 20 consecutive mis-diagnoses
  by presenting the `init` (all-zeros) reading as 'registration failed'."
  The anti-regression rule there is explicit: the string "registration
  failed" MUST NOT appear for any state.
- **The docket item is stuck and unactionable.** `agentns-session-zeros`
  (warn, 8 reports across 6 runs since 2026-05-30, `consecutive_runs: 0`,
  evidence `recall:01KT6VCBX1PSWT9MAQP607TAWY`) keeps re-firing with no
  playbook because nothing classifies the state correctly or knows the
  difference between "kernel broken" (a fault) and "launchers not yet
  routed" (expected, fixed by christen-route).
- **The surface is live and readable, measured 2026-06-05.**
  `/proc/self/ns/agent -> agent:[4026531996]` (init NS inode),
  `/proc/self/agent_session = 0…0`, `/proc/self/agent_counters` all-zero,
  `CONFIG_AGENT_NS=y`. `agentns-doctor status` already classifies
  absent/init/live/malformed — christen-detect reuses that semantics but
  adds the **docket edge-trigger** the doctor lacks.
- Depends on christen-plan having shipped (the `christen` repo + `WrapState`
  exist). Read-only except for the single `docket` side effect.

## What this builds

Extend the `christen` crate (bump minor) with a `probe` module and a
`christen probe` subcommand.

**The classifier.** A pure `classify(reading: NsReading) -> NsState`:

- `NsReading { ns_inode: Option<u64>, session_hex: Option<String>,
  counters: Option<Counters>, kernel_is_wintermute: bool,
  wrapper_installed: bool }` — the raw `/proc` observation + context,
  injected so `classify` is pure.
- `NsState` enum, each with correct human prose (no "registration failed"):
  - `Absent` — surface missing. Fault **only** on a `-wintermute` kernel;
    expected on stock.
  - `Init { reason: InitReason }` — id is all-zeros / ns inode is the init
    inode. `InitReason::UnwrappedExpected` when the wrapper is installed but
    launches aren't routed (the current reality — points at christen-route);
    not a fault.
  - `Live { session_hex: String, intent: Option<String> }` — nonzero id;
    healthy.
  - `Malformed { detail: String }` — surface present but unparseable.
- `verdict(&NsState) -> Verdict { ok: bool, prose: String, docket: DocketOp }`
  where `DocketOp` is `Report { severity, title, evidence }` |
  `Resolve { id }` | `None`. The anti-regression invariant — the literal
  string "registration failed" never appears in any `prose` — is asserted by
  a test over every `NsState` variant.

**The `/proc` reader.** A `ProcReader` trait (`read(pid) -> NsReading`) with a
real impl (reads `/proc/$pid/{ns/agent,agent_session,agent_counters}`,
detects the init inode, reads `uname` + checks the installed wrapper) and a
`FakeReader` for tests. The init-inode constant and detection are documented.

**The docket edge-trigger.** `christen probe --emit` runs the classifier and
applies the `DocketOp` by shelling `docket` (`report`/`resolve`):

- `Live` → `docket resolve agentns-session-zeros` (the substrate came alive).
- `Init { UnwrappedExpected }` on a `-wintermute` kernel with wrapper
  installed → `docket report` re-titled to the *actionable* state
  ("agentns init NS — launches not routed through agentns-claude; run
  `christen route`") rather than the stale "stuck at zero" framing.
- `Absent` on `-wintermute` → `docket report` (real fault: driver/boot).
- Anything on a stock kernel → no docket op (expected).

`--emit` is the only side-effecting path; default `christen probe` prints
and exits without touching the docket. Per-`docket`-call failure is non-fatal
(absent `docket` binary must not crash the probe).

**UX:** `christen probe [--pid N] [--emit] [--format json]`. JSON carries
`{ state, ok, prose, session_hex?, intent?, docket_op }`. Exit code:
0 for `Live`/`Init`(expected), non-zero for `Absent`/`Malformed` on
`-wintermute`. `sigpipe::reset()` already in `main` from christen-plan.

## Acceptance criteria

1. `cargo test` passes offline; `classify` is pure (operates only on the
   injected `NsReading`) — a test asserts it reads no files and shells
   nothing.
2. `classify` returns the right `NsState` for each `FakeReader` fixture:
   all-zero id + init inode + wrapper-installed + `-wintermute` →
   `Init{UnwrappedExpected}`; a nonzero hex id → `Live`; missing surface on
   `-wintermute` → `Absent`; missing surface on stock → `Absent` but
   `verdict.ok == true`; unparseable id → `Malformed`.
3. **Anti-regression:** a test iterates every `NsState` variant and asserts
   the rendered `prose` never contains the substring "registration failed".
4. `verdict` maps states to the documented `DocketOp`: `Live` → `Resolve`,
   `Init{UnwrappedExpected}`/`-wintermute` → `Report` (actionable title),
   `Absent`/`-wintermute` → `Report`, any stock-kernel state → `None`.
5. `christen probe --format json` against a `FakeReader` emits the documented
   schema including `docket_op`; a `Live` fixture and an `Init` fixture each
   covered.
6. `christen probe --emit` shells `docket` with the mapped op (verified via a
   fake `docket` on `PATH` in an integration test that records argv); a
   missing `docket` binary is handled non-fatally (probe still prints, exits
   on classification, not on the docket failure).
7. Run against this machine's real `/proc/self` (a `tests/` case gated to a
   `-wintermute` kernel, else skipped) yields `Init{UnwrappedExpected}` with
   `ok == true` — **deferred_acs:[7]** (only meaningful on the laptop; the
   cloud build box has no `-wintermute` kernel).
8. README documents the four states, the init-inode detection, the
   anti-regression rule, and the `--emit` docket contract.
