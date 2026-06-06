# PRD: assay-quicken-bridge

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/quicken
Vision: visions/assay.md

## TL;DR

`quicken`'s `AgentnsProbe` reads the live process's `/proc/self/agent_session`,
sees 32 zeros, and reports `Verdict::Inert`. That verdict is correct but
*uninformative*: it cannot tell jsy whether to fix userspace (nothing wraps the
launch) or the kernel (the flag is rejected). `assay-agentns` already knows the
answer — `unshare(CLONE_NEWAGENT)` returns EINVAL because the flag collides with
`CLONE_VM`. This PRD wires the two together: `AgentnsProbe` shells out to
`assay agentns --json` and folds its localized verdict into the quicken report,
upgrading "Inert (cause unknown)" to "Inert — kernel rejects the flag; wrapping
the launch will not help; see PRD-agentns-clone-flag-fix." The recurring
self-review carry-forward finally carries its own cause.

## Why this exists

Phase-1 evidence, 2026-06-06:

- `PRD-quicken-probe.md` defines `AgentnsProbe` as: read
  `/proc/self/agent_session`; all-zeros → `Inert`; non-zero → `Live`. By
  construction it reads the *live* process, which is always in the init
  agent_ns and therefore always zero — so it can **never** distinguish a wiring
  gap from a kernel bug.
- The live run (`tests/test_unshare` → EINVAL; compiled flag `0x100` ==
  `CLONE_VM`) proves the cause is kernel-side. `quicken` has no way to surface
  that; `onramp` would "fix" it with a launch wrap that cannot work.
- Self-review 2026-06-03 / 2026-06-06 carry "agentns all-zeros" as open with
  **no playbook** — precisely because the verdict lacks a cause to act on.

Composing the active attestation (`assay`, which *creates* a namespace) with
the passive liveness probe (`quicken`, which *reads* the live process) is the
whole point of the `assay` vision: read-the-surface and exercise-the-mechanism
are different questions, and the second answers what the first cannot.

## What this builds

A `rust-extend` of the existing `~/wintermute/quicken` workspace (no new repo,
no publish). Append-only where it can be, per the loom/integrate-collision
lessons (`self_build_manifest_join_slug`, lib-api append pattern).

- **Optional `assay` integration in `AgentnsProbe`.** When `assay` is on
  `PATH` (or at a configured path via `ProbeEnv`, so tests inject it), and the
  live session reads all-zeros, `AgentnsProbe` runs `assay agentns --json`,
  parses the `AttestReport`, and:
  - If `verdict == FlagRejected{…}` → quicken verdict becomes
    `Inert` **with** `Evidence` enriched by `cause: "kernel rejects
    CLONE_NEWAGENT (flag 0x100 == CLONE_VM), errno EINVAL"` and
    `remediation: "PRD-agentns-clone-flag-fix; launch-wrap will not help"`.
  - If `verdict == Live{session}` (kernel fixed, but the *live* process simply
    isn't wrapped) → quicken verdict becomes a new
    `InstalledNotActivated`-flavored variant `MechanismLiveNotWired` with
    `remediation: "wrap the launch (onramp claude-agentns-wrap)"` — the
    *correct* remedy, now that the kernel works.
  - If `assay` is absent or errors → fall back to today's behavior exactly
    (`Inert`, no enrichment). **Fail-open**, per
    `self_build_jq_escape_reads_absent`: absent attestation ≠ a worse verdict.
- **One new `Verdict` variant** `MechanismLiveNotWired { remediation }` (or an
  enrichment field on the existing report — implementer's choice, kept
  append-only to the public enum to avoid breaking `quicken probe --json`
  consumers like `docket-digest`).
- **Evidence passthrough.** The decisive bytes from `assay` (errno, compiled
  flag, collision) appear verbatim in `quicken probe --json` under the agentns
  entry, so a downstream reader (`docket`, self-review) needs only quicken's
  output, not a second tool invocation.
- **No change to the other probes.** memlog/warden/provfs probes are untouched
  (their root causes are already known and don't warrant an active
  attestation).

## Acceptance criteria

1. `cargo build` and `cargo test` green on rustc 1.85 in `~/wintermute/quicken`
   after the extend; existing quicken tests still pass (no regression).
2. With a fake `assay` (a fixture script on the injected path) that emits a
   `FlagRejected` `AttestReport`, `quicken probe --json` for agentns shows
   verdict `Inert` **plus** `cause`/`remediation` evidence quoting the EINVAL +
   `0x100 == CLONE_VM` collision.
3. With a fake `assay` that emits `Live{session}`, `quicken probe` reports
   `MechanismLiveNotWired` with the *launch-wrap* remediation (proving the tool
   will redirect the fix to userspace once the kernel is fixed).
4. With `assay` **absent** from the configured path, `quicken probe` for
   agentns is byte-identical to today's `Inert` output — fail-open, no panic,
   no non-zero from the bridge itself.
5. `quicken probe --json` remains parseable by existing consumers: the public
   `PrimitiveReport` JSON shape is extended append-only (new optional fields /
   new enum variant), and a test asserts an old-shaped consumer still
   deserializes it.
6. `assay agentns` is invoked **at most once per `quicken probe` run** and only
   when agentns reads all-zeros (no needless forks when the kernel is already
   `Live`).
7. quicken's `README.md` documents the `assay` dependency as optional and the
   new agentns verdicts, with the live before/after the kernel fix.

## Notes / dependencies

- **Depends on PRD-assay-agentns** (consumes its `--json` `AttestReport`
  contract). Independent of `agentns-clone-flag-fix` — can build in parallel
  with it (disjoint `build_into`: `quicken` vs `agentns`).
- Reuses quicken's existing `ProbeEnv` injection seam (already designed for
  fixtures) — no new test scaffolding needed.
