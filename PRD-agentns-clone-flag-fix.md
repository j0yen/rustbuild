# PRD: agentns-clone-flag-fix

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/agentns
Vision: visions/assay.md

## TL;DR

`assay-agentns` proves the agentns namespace can never be created on the
booted kernel: `CLONE_NEWAGENT` is `#define`d as `0x00000100`, which **is
`CLONE_VM`**, so `unshare(CLONE_NEWAGENT)` returns EINVAL. The legacy 32-bit
clone-flag space is exhausted — there is no free bit to grab. This PRD stops
hanging the eighth namespace off a legacy clone bit and re-routes namespace
creation through a path with room: a dedicated
`prctl(PR_SET_AGENT_NS)` "enter a fresh agent namespace" operation (the prctl
dispatch already exists, patch 0005), leaving the legacy clone/unshare flag
table untouched. The result: `unshare`-style creation works again, a child
gets a non-zero `/proc/self/agent_session`, and `assay-agentns` reports `Live`.

## Why this exists

This PRD is the fix `assay-agentns` motivates — and it must **not** be built
until that attestation exists and reproduces the diagnosis, per
`feedback_verify_before_concluding`. Phase-1 evidence (2026-06-06):

- `~/wintermute/agentns/tests/test_unshare` →
  `unshare(CLONE_NEWAGENT) failed: Invalid argument` on the booted kernel.
- `agentns/include/uapi/linux/agent_namespaces.h:19` →
  `#define CLONE_NEWAGENT 0x00000100` (== `CLONE_VM`); patch 0001's commit
  message claims `0x40000000` (== `CLONE_NEWNET`). Both legacy bits are taken;
  `CLONE_NEWTIME` (0x80) consumed the last free low bit years ago.
- `patches/0002-nsproxy-add-agent_ns.patch:82` gates the unshare path on
  `(unshare_flags & (… | CLONE_NEWAGENT))` — but with `CLONE_NEWAGENT ==
  CLONE_VM` the `check_unshare_flags()` mask in `kernel/fork.c` interprets the
  bit as `CLONE_VM` and rejects it before the agentns code is ever reached.

So the bug is **architectural**: the design assumed a free legacy clone bit
that does not exist. Picking a different legacy bit cannot work; every bit is
occupied. The namespace must be created through a non-clone-flag path.

**Why prctl, not clone3 (the open question, answered with a default).**
`clone3()`'s `__u64 flags` has room above bit 31, but legacy `unshare(2)` only
accepts the 32-bit set — a `clone3`-only flag can't be reached by `unshare`,
and the existing userspace (`agent-wrap`, `unshare-helper`) is `unshare`-based.
A dedicated `prctl(PR_SET_AGENT_NS)` that creates-and-enters a fresh agent
namespace for the calling task: (a) reuses the prctl dispatch already shipped
in patch 0005; (b) needs no clone-flag bit at all; (c) is trivially callable
from the existing C/Rust userspace and from a thin `agentns-unshare` shim.
**This default should be confirmed with jsy before build** (see vision open
questions) — it is the one real design decision here.

## What this builds

A correction to the agentns kernel patch series + the inline-edit applier,
following the kernel-pkg convention (`apply-agentns.py`'s idempotent,
anchor-based inline edits — `feedback`/memory: more durable than raw `.patch`
files across version bumps).

- **New prctl op** `PR_SET_AGENT_NS` added to
  `include/uapi/linux/agent_namespaces.h` (next value in the `PR_AGENT_BASE`
  block). Semantics: create a fresh `agent_ns` (fresh 128-bit session id, zeroed
  counters, cleared intent tag), install it on the calling task's `nsproxy`,
  return 0; `-EPERM`/`-ENOMEM` on failure.
- **Kernel handler** in `kernel/agent_namespaces.c` (+ the prctl dispatch in
  the patch-0005 hunk) implementing create-and-enter, reusing the existing
  `agent_ns` alloc/free and counter-init code paths.
- **Decouple the legacy flag.** Remove/retire the `CLONE_NEWAGENT 0x100`
  `#define` from the namespace-creation path (keep it only if some non-creation
  code references it; otherwise delete it) and drop the
  `CLONE_NEWAGENT` term from the patch-0002 `check_unshare_flags` gate, so the
  legacy clone/unshare tables are left exactly as mainline. `/proc` surfaces
  (patch 0006) and counters (patch 0008) are unchanged.
- **`apply-agentns.py`** extended with a new anchor block for the prctl op +
  handler (idempotent, re-runnable), and the now-invalid clone-bit edit removed
  from its anchor set. A `--check` mode that fails if the stale `0x100`
  `#define` is still present on the creation path.
- **Userspace shim** `agentns-unshare` (a few lines in
  `~/wintermute/agentns/userspace`) that calls `prctl(PR_SET_AGENT_NS)` and
  then `exec`s its argv — the working replacement for the EINVAL-bound
  `agent-wrap`/`test_unshare` path. `onramp`'s launch wrap consumes *this*, not
  raw `unshare(CLONE_NEWAGENT)`.

## Acceptance criteria

1. `apply-agentns.py` runs idempotently against a clean linux source tree
   (apply twice → no diff the second time) and its `--check` mode fails if the
   `CLONE_NEWAGENT 0x100` creation `#define` is still present.
2. The corrected patch series compiles into a `linux-wintermute` kernel package
   via the existing `scripts/build.sh` / pkg PKGBUILD (build-only AC; cloud or
   local per `feedback_cloudbuild_over_build`).
3. `agentns-unshare cat /proc/self/agent_session`, run under the **rebuilt
   kernel**, prints a **non-zero** 128-bit session id (contrast: the booted
   kernel prints 32 zeros). *(User-gated: requires installing + rebooting the
   corrected kernel — the same reboot window self-review flags for the blocked
   `linux`/`linux-firmware` queue.)*
4. `assay agentns` (from PRD-assay-agentns), run under the rebuilt kernel,
   reports `Verdict::Live` with all five layers passed. **This is the
   definition of done** — the attestation, not a hand-written fixture, is the
   proof (`feedback_agent_written_fixtures_tautology`).
5. The legacy clone/unshare flag tables in the patched tree are byte-identical
   to mainline for the `CLONE_*` mask (no new legacy bit claimed) — verified by
   a diff in the PR/receipt.
6. `tests/test_unshare.c` is updated (or a sibling `test_prctl_ns.c` added) to
   exercise the prctl path and assert a non-zero session post-create; the old
   EINVAL expectation is replaced with the new contract.
7. `agentns/README.md` is updated to document `PR_SET_AGENT_NS` as the creation
   mechanism and to record *why* the legacy clone-flag approach was abandoned
   (flag-space exhaustion), citing the `assay-agentns` finding.

## Notes / dependencies

- **Depends on PRD-assay-agentns** — do not build until the attestation exists
  and reproduces the EINVAL/collision verdict on the booted kernel.
- **Gates onramp's `claude-agentns-wrap`** — that wrap is futile until this
  ships; it must be re-pointed at `agentns-unshare` (prctl path). Flagged in
  gossip for /build so the wrap isn't re-shipped as-is.
- Final proof (AC3/AC4) is **user-gated on a reboot**; the build/apply ACs
  (1, 2, 5, 6, 7) are not, so /build can advance everything except the live
  proof autonomously.
