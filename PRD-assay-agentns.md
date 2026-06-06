# PRD: assay-agentns

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/assay
Vision: visions/assay.md

## TL;DR

The booted `7.0.10-arch1-5-wintermute` kernel reports `agent_session` as 32
zeros for every process. Two existing visions read that surface and reach
opposite-but-both-wrong conclusions: `quicken` calls it `Inert`, `onramp`
proposes wrapping the Claude launch in `unshare(CLONE_NEWAGENT)`. Neither ever
**creates** an agent namespace, so neither can tell whether the cause is
"nothing wraps the launch" or "the kernel physically rejects the flag." A
30-second live run settles it: `unshare(CLONE_NEWAGENT)` returns **EINVAL** —
the namespace cannot be created at all. `assay-agentns` is the missing tool: a
read-only Rust CLI that *exercises the actual mechanism* in a throwaway child
and emits a structured verdict localizing the broken layer, so a futile fix
never ships and a real root cause stops recurring in self-review.

## Why this exists

Phase-1 evidence, measured live 2026-06-06 ~08:00 UTC on this laptop:

- `cat /proc/self/agent_session` → `00000000000000000000000000000000`;
  `/proc/self/agent_counters` → every field 0. The `/proc` surface (agentns
  patch 0006) **is** present in the booted kernel.
- `~/wintermute/agentns/userspace/agent-wrap cat /proc/self/agent_session` →
  still 32 zeros, **exit 0**. The shipped wrapper "succeeds" while changing
  nothing.
- `~/wintermute/agentns/tests/test_unshare` →
  `unshare(CLONE_NEWAGENT) failed: Invalid argument`.
- `agentns/include/uapi/linux/agent_namespaces.h:19` →
  `#define CLONE_NEWAGENT 0x00000100`, which **is `CLONE_VM`**. Patch 0001's
  commit message claims `0x40000000` (== `CLONE_NEWNET`). The legacy clone-flag
  space is exhausted (`CLONE_NEWTIME` 0x80 took the last bit).
- Build manifest: `claude-agentns-wrap [shipped]`, `agentns-claude [shipped]`.
  A fix shipped 2026-05; the all-zeros symptom outlived it. Self-review
  2026-06-03 and 2026-06-06 both carry "agentns session id all-zeros" as an
  open, escalated item with **no playbook**.

The lesson (`feedback_verify_before_concluding`,
`feedback_agent_written_fixtures_tautology`): never assert a root cause from
indirect signals; instrument the *actual* failing path and prove the value at
the failing step first. `quicken-probe`'s `AgentnsProbe` reads the live
process — which is *always* in the init agent_ns and *always* reads zero — so
it can never produce this proof. This tool produces it.

## What this builds

A new cargo workspace at `~/wintermute/assay` (Rust 2021,
`rust-toolchain.toml` pinned to 1.85 per lib-crate convention), with an
`assay-core` lib crate and a thin `assay` binary (`[[bin]]`) hosting
subcommands (`assay agentns` first). Local-tool conventions: `sigpipe::reset()`
as the first line of `main()` (per `self_sigpipe_panic_toolkit`); `nix` (or
raw `libc`) for `unshare`/`prctl`/`fork`.

**Core types (`assay-core`):**

- `Layer` enum naming the mechanism step that was exercised:
  `FlagAccepted`, `NsCreated`, `SessionNonZero`, `CountersAdvance`,
  `IntentTagRoundtrip`.
- `Verdict` enum localizing the *first* failed layer:
  - `FlagRejected { flag: u32, collides_with: Option<String>, errno: i32 }`
  - `NsCreatedButSessionZero` (unshare succeeded yet session stayed zero)
  - `CountersDead` (ns created, session set, but counters never moved)
  - `IntentTagLost`
  - `Live { session: String }` (every layer passed)
  - `Unknown { detail: String }`
- `Evidence`: ordered key/value records (e.g. `before_session`,
  `after_session`, `unshare_errno`, `compiled_flag`, `collides_with`,
  `counters_before`, `counters_after`) so every verdict carries the bytes it
  was derived from.
- `AttestReport { primitive: "agentns", verdict, layers_passed: Vec<Layer>,
  evidence, kernel_release, checked_at }`.

**The agentns attestation (`assay agentns`):**

1. Read `/proc/self/agent_session` (the `before` value) and the compiled
   `CLONE_NEWAGENT` value from a small build-time constant mirrored from the
   uapi header (documented as derived from
   `agentns/include/uapi/linux/agent_namespaces.h`).
2. Check the flag value against a table of known-occupied clone bits
   (`CLONE_VM 0x100`, `CLONE_FS 0x200`, … `CLONE_NEWNET 0x40000000`,
   `CLONE_NEWTIME 0x80`); if it collides, record `collides_with`.
3. `fork()` a child. In the child: call `unshare(CLONE_NEWAGENT)`, capture
   `errno`, then (if it succeeded) read `/proc/self/agent_session` (the
   `after` value), perform a fixed number of counted syscalls (e.g. N
   `openat` of `/dev/null`, M bytes written to a pipe), read
   `/proc/self/agent_counters`, set + get an intent tag via
   `prctl(PR_SET_AGENT_INTENT_TAG)` / `PR_GET_AGENT_INTENT_TAG`. The child
   reports its findings to the parent over a pipe and `_exit`s — **nothing in
   the live system is mutated** (the namespace dies with the child).
4. Parent assembles the `AttestReport`, choosing the verdict from the first
   failed layer.

**CLI:**

- `assay agentns` → human one-screen report: verdict line, the layer ladder
  (✓/✗ per `Layer`), and the decisive evidence (errno, compiled flag,
  collision).
- `assay agentns --json` → the `AttestReport` as JSON (stable shape for the
  `assay-quicken-bridge` consumer).
- Exit code: `0` if `Live`, non-zero otherwise (usable as a gate). The
  specific non-zero code encodes the verdict class so a shell consumer can
  branch without parsing JSON.

**Testability:** the syscall surface is abstracted behind a trait
(`AgentSyscalls`) so unit tests inject a fake that returns a scripted
`errno`/session/counters, letting CI assert that a given fake produces a given
`Verdict` without a special kernel. The live `agentns` subcommand uses the
real `nix`-backed impl.

## Acceptance criteria

1. `cargo build` and `cargo test` are green on rustc 1.85 in
   `~/wintermute/assay`; `main()`'s first line is `sigpipe::reset()`.
2. `assay agentns --json` on the **current booted kernel** emits a report with
   `verdict.FlagRejected`, `evidence.unshare_errno == 22` (EINVAL),
   `evidence.compiled_flag == "0x100"`, and
   `evidence.collides_with == "CLONE_VM"`. (This is the proof the vision
   claims; it must reproduce the live finding.)
3. `assay agentns` (human form) prints a layer ladder showing `FlagAccepted ✗`
   as the first failed layer and a one-line remediation pointer
   ("kernel rejects the flag — see PRD-agentns-clone-flag-fix; wrapping the
   launch will not help").
4. Exit code is non-zero on the current kernel and the code encodes the
   `FlagRejected` class.
5. A unit test injects a fake `AgentSyscalls` whose `unshare` succeeds and
   whose post-unshare session is non-zero with advancing counters, and asserts
   the verdict is `Live { session }` with all five `Layer`s passed — proving
   the tool will correctly report success once the kernel is fixed.
6. A unit test injects a fake whose `unshare` succeeds but whose session stays
   zero, and asserts `NsCreatedButSessionZero` (so the tool distinguishes the
   flag bug from a *different* future kernel bug).
7. The child process leaves no residue: an assertion (or documented manual
   check) that `/proc/self/agent_session` in the **parent** is unchanged after
   the run (the attestation is non-mutating).
8. `README.md` documents the layer ladder, the verdict table, and the exact
   live command + expected output from AC2.
