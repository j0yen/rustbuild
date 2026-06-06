# Vision: assay — a shipped fix the symptom outlived is an unverified fix

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-06
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. The strongest *unaddressed*
signal this pass was not a missing feature — it was a **wrong conclusion baked
into two existing visions**, caught only by actually running the primitive
instead of reading its surface.

## TL;DR

`quicken` reads a primitive's surface and reports a **verdict** (agentns →
`Inert` because `/proc/self/agent_session` is all-zeros). `onramp` reads the
same zeros and proposes the **remedy** (wrap the Claude launch in
`unshare(CLONE_NEWAGENT)`). Both are wrong about agentns, and neither can know
it, because **neither one ever creates the namespace** — they only read the
live process, which is always in the init agent_ns and therefore always reads
zero. The verdict ("Inert") and the remedy ("wrap the launch") are both
inferred from a surface that cannot distinguish *"nothing wrapped the launch"*
from *"the kernel physically cannot create this namespace."*

Run the actual mechanism and the truth falls out. Measured live this pass on
the booted `7.0.10-arch1-5-wintermute` kernel:

```
$ ~/wintermute/agentns/tests/test_unshare
unshare(CLONE_NEWAGENT) failed: Invalid argument
parent (init NS): session=00000000000000000000000000000000
```

`unshare(CLONE_NEWAGENT)` returns **EINVAL**. The agentns namespace can never
be created from userspace on this kernel — so:

- `onramp`'s shipped `claude-agentns-wrap` PRD is **futile**: even a perfect
  launch wrapper calls `unshare(CLONE_NEWAGENT)`, which the kernel rejects.
  The all-zeros it was meant to fix will persist no matter how the launch is
  wired.
- `quicken`'s `AgentnsProbe` verdict `Inert` is *correct but uninformative*:
  it cannot tell jsy whether to fix userspace (wiring) or the kernel (the
  flag), so the finding has recurred in self-review after self-review,
  escalated "once," and never closed.
- `provfs` is `LiveDegraded` (fallback `comm:`-form session id) **because of
  this** — it has nothing but zeros to stamp.

**Why EINVAL — proven, not guessed.** The compiled flag is
`#define CLONE_NEWAGENT 0x00000100` (`agentns/include/uapi/linux/agent_namespaces.h`),
which **is `CLONE_VM`** — a direct collision with an existing clone bit. The
patch-0001 commit message even says it meant `0x40000000`, which is itself
`CLONE_NEWNET`. The legacy 32-bit `clone()`/`unshare()` flag space is
**exhausted** — `CLONE_NEWTIME` (0x80) took the last free low bit — so there
is no free legacy bit for an eighth namespace type. `unshare(2)`'s fixed
`check_unshare_flags()` mask therefore reads `0x100` as `CLONE_VM`, which
`unshare` does not honor in this context, and returns EINVAL. This is an
**architectural** problem, not a one-line wiring fix.

`assay` builds the missing layer between "read the surface" (`quicken`) and
"ship the remedy" (`onramp`): a **functional attestation** that *exercises the
primitive's real mechanism* — creates the namespace, forces the counted
syscalls, round-trips the intent tag — and emits a verdict that **localizes
the broken layer** (kernel-rejects-flag vs created-but-counters-dead vs
works-but-nothing-wires-it). The same evidence-rich, read-mostly,
proposal-first ethos as `quicken`/`vigil`: the attestation is pure
observation (it forks a throwaway child; it never mutates the live system),
and it refuses to let a consumer ship a remedy for a cause it never proved.

## End-state

When this is done:

- `assay agentns` on any booted kernel reports, in one command, **whether the
  agentns mechanism physically works** — not whether the live process happens
  to be in a non-init namespace. On today's kernel it reports
  `FlagRejected{ flag: 0x100, collides_with: "CLONE_VM", errno: EINVAL }`,
  turning a recurring "Inert (cause unknown)" into a closeable, kernel-side
  root cause.
- The agentns kernel patch series no longer hangs a new namespace off an
  exhausted legacy clone bit. `unshare`/`clone3` for an agent namespace either
  succeeds and yields a non-zero `/proc/self/agent_session`, or the attestation
  says exactly why not.
- `quicken`'s `AgentnsProbe` consumes the attestation, so its verdict carries
  the *cause* ("kernel rejects the flag — wrapping the launch will not help"),
  not just the *symptom* — and `onramp`'s wrap PRD is explicitly gated on the
  attestation passing, so a futile fix can't ship.
- The pattern generalizes: any future "we shipped the fix but the symptom
  persists" primitive gets an `assay <name>` that proves the mechanism before
  a consumer wires it or a human re-escalates it.

## Why this is real (Phase 1 evidence, 2026-06-06 ~08:00 UTC)

Measured live this session on `7.0.10-arch1-5-wintermute` (uptime 2d17h):

- `cat /proc/self/agent_session` → 32 zeros; `/proc/self/agent_counters` → all
  zero. The `/proc` surface (patch 0006) **is** in the booted kernel and reads
  cleanly — so this is not "the kernel lacks agentns entirely."
- `~/wintermute/agentns/userspace/agent-wrap cat /proc/self/agent_session` →
  still 32 zeros, **exit 0** (the wrap "succeeds" while doing nothing).
- `~/wintermute/agentns/tests/test_unshare` →
  `unshare(CLONE_NEWAGENT) failed: Invalid argument`. **The smoking gun.**
- `agentns/include/uapi/linux/agent_namespaces.h:19` →
  `#define CLONE_NEWAGENT 0x00000100` == `CLONE_VM`. Collision.
- `patches/0001-*.patch:5` → commit message claims bit `0x40000000`
  (== `CLONE_NEWNET`, also taken).
- Build manifest: `claude-agentns-wrap [shipped]`, `agentns-claude [shipped]`
  — a fix shipped; the symptom outlived it. Nobody verified.
- Self-review 2026-06-03 and 2026-06-06 both list "agentns session id
  all-zeros" as an open, escalated, no-playbook carry-forward.

## Components (PRD-sized)

1. **assay-agentns** — the functional attestation. A small Rust CLI
   (`~/wintermute/assay`) that forks a child, attempts
   `unshare(CLONE_NEWAGENT)`, captures the exact errno, reads the compiled
   flag value, checks it against the known-occupied clone bits, reads
   `/proc/self/agent_session` before/after, forces N counted syscalls and
   checks the counters move, and round-trips the intent tag via `prctl`.
   Emits a structured `AttestReport` with a localized verdict. Airtight: the
   diagnosis is already proven by the live run above.

2. **agentns-clone-flag-fix** — the kernel fix the attestation motivates.
   Stop hanging the namespace off an exhausted legacy clone bit; expose it via
   `clone3()`'s 64-bit flag field (or a dedicated prctl/syscall path) and
   correct patches 0001/0002/0003 + the `unshare` mask. Drops a corrected
   patch + extends `apply-agentns.py` with a new anchor (per the kernel-pkg
   inline-edit convention). Depends on #1's verdict; carries a real open
   question on the exact bit/mechanism.

3. **assay-quicken-bridge** — wire #1 into `quicken` so `AgentnsProbe`
   upgrades from `Inert` to `Inert + cause` by shelling out to
   `assay agentns --json`, and so `onramp`'s wrap PRD is gated on the
   attestation passing. Composes with `quicken`; does not duplicate it.

## Order

- **assay-agentns first** — it is the only fully-thought-through piece and it
  gates the other two. It can ship entirely independently (new workspace, no
  consumers).
- **agentns-clone-flag-fix** and **assay-quicken-bridge** both depend on #1
  but are independent of *each other* and can build in parallel: the fix is in
  `~/wintermute/agentns` (kernel patches), the bridge is in
  `~/wintermute/quicken` (rust-extend) — disjoint `build_into`, no
  integrate-collision.
- The kernel fix additionally needs a **reboot of the corrected kernel** to
  *prove* — so its final AC is "assay-agentns reports `Live` on the rebuilt
  kernel," which is user-gated on a reboot window (the same window self-review
  keeps flagging for the blocked `linux`/`linux-firmware` pacman queue).

## Open questions (for jsy)

- **Flag mechanism for the fix.** `clone3()`'s `__u64 flags` has room
  (e.g. a bit above 0xFFFFFFFF), but legacy `unshare(2)` only takes the 32-bit
  set — so a `clone3`-only namespace can't be created by `unshare`. Options:
  (a) `clone3`-only + a thin `agentns-unshare` helper that uses `clone3`;
  (b) a dedicated `prctl(PR_SET_AGENT_NS)` "enter a fresh agent ns" path that
  sidesteps clone flags entirely; (c) reclaim a legacy bit (risky). Lean (b) —
  the prctl dispatch (patch 0005) already exists. **This is the real design
  decision and should be settled before agentns-clone-flag-fix is built.**
- Should `assay` absorb `quicken`'s passive probes over time, or stay strictly
  the *active-exercise* half (assay creates/forces; quicken reads)? Drafted as:
  stay disjoint — assay exercises mechanisms, quicken reads live state.
- Does `assay` want a `--all` dispatcher across primitives now, or only
  `agentns` until a second primitive earns an active attestation? Drafted:
  `agentns` only — memlog/warden/provfs root causes are already known
  (group/install, never-armed, downstream-of-agentns), so they don't need an
  *active* attestation yet. Don't dream past the research.
