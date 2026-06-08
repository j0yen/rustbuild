# PRD: mend-warden-doctor — say *why* the enforcer is inert, in one command

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/bpolicy
Vision: visions/mend.md

## TL;DR

The bpolicy / warden eBPF-LSM write-enforcer reads `loaded:false` — present on
disk but never armed this boot — and has done so for multiple self-review runs
(docket `warden-enforcer-inert`, first_run 2026-06-03). Every run the
self-review notes it, escalates/acks it, and moves on, because there is no way
to tell *why* it's inert: missing from the kernel `lsm=` list? no BTF? attach
EPERM? wrong group? This PRD adds a read-only `bpolicy doctor` that probes each
precondition and prints the activation gap. It never arms anything — it makes
the inert state diagnosable so a human (or a future gated PRD) can act on a
specific cause instead of a shrug.

## Why this exists

- **Counted, owner-less, acked-but-never-diagnosed.** Live docket:
  `[open] warden-enforcer-inert (warn) — "bpolicy enforcer present but never
  armed this boot" first_run: 2026-06-03`. journal 2026-06-07 / 2026-06-06:
  "warden: not loaded (inert — escalated, see docket key
  warden-enforcer-inert) … Carry only." It is carried, never resolved.
- **The carry is forced by missing information.** The self-review can see
  `loaded:false` but not the cause, so the only safe action is "escalate and
  ack." A diagnosis turns a perennial carry into an actionable finding.
- **bpolicy is a real Rust package.** `~/wintermute/bpolicy/` has `Cargo.toml`,
  `src/`, `bpf/`, `reference/` — extendable in place (rust-extend). The eBPF-LSM
  write-enforcer is the tool from [[feedback_local_tools]] ("bpolicy: eBPF-LSM
  write enforcer").
- **Fleet stance is diagnose-don't-mutate.** recourse and tribunal both settle
  on read-only/proposal-only for anything touching enforcement. A *doctor*
  (read-only) fits that stance; *arming* is deliberately out of scope here.

## What this builds

Extends `~/wintermute/bpolicy` with a `bpolicy doctor` subcommand that probes,
in order, every precondition for the LSM enforcer to be armed, and reports each
as `ok | gap | unknown` with the observed value:

1. **Kernel LSM list** — is `bpf` (and any required LSM) present in
   `/sys/kernel/security/lsm` and/or the `lsm=` kernel cmdline
   (`/proc/cmdline`)? A common inert cause: BPF-LSM not in the active list.
2. **BTF availability** — `/sys/kernel/btf/vmlinux` present and readable (CO-RE
   needs it).
3. **Attach capability** — can the loader attach (probe `bpf()` /
   `BPF_PROG_LOAD` capability, `CAP_BPF`/`CAP_SYS_ADMIN`, attach errno if a
   dry-run attach is attempted read-only)? Report the errno symbolically
   (`EPERM`, `EINVAL`, …).
4. **Group / permission** — is the invoking user in the group the enforcer
   expects (mirror the memlog-group pattern), and are the pinned-map paths
   under `/sys/fs/bpf` accessible?
5. **Boot-arm wiring** — is there a systemd unit / hook that *should* arm it at
   boot, and did it run this boot (last `loaded:false` since boot)?

Output: `bpolicy doctor` (human, one line per check) and
`bpolicy doctor --format json` (machine, for the self-review to parse and
`docket report` against). A `--format docket` mode emits a `docket report
--key warden-enforcer-inert --evidence cause:<first-gap>` line so the finding
finally carries a *cause*, not just a state.

Hard constraints: **read-only**. No arming, no `bpf()` program load that
persists, no map pinning, no privilege change. A dry-run attach (if used) must
be immediately detached and must not alter system state. SIGPIPE reset per
[[self_sigpipe_panic_toolkit]]. rustc 1.85, no let-chains.

## Acceptance criteria

1. `cargo build --release` + `cargo test` green in `~/wintermute/bpolicy`;
   existing enforcer code paths unchanged.
2. `bpolicy doctor` prints one line per precondition (LSM-list, BTF, attach,
   group, boot-arm) with `ok|gap|unknown` and the observed value.
3. On this box's current state (enforcer inert), `doctor` identifies at least
   one concrete `gap` and names it — output is not merely "inert".
4. `bpolicy doctor --format json` emits a parseable object keyed by check name;
   `--format docket` emits exactly one `docket report --key
   warden-enforcer-inert` line carrying a `cause:` evidence token equal to the
   first gap.
5. Read-only proof: running `doctor` does not change `/sys/kernel/security/lsm`,
   does not leave a loaded/pinned BPF program, and does not alter the
   `loaded:false` state (verified before/after).
6. When every precondition is `ok` but the enforcer is still inert, `doctor`
   reports `boot-arm: gap` (the wiring exists but didn't run) rather than
   declaring success.
7. SIGPIPE: `bpolicy doctor | head -1` exits 0, no panic.
