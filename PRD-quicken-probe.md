# PRD: quicken-probe

Status: Draft v0.1
build_target: rust-cli
Vision: visions/quicken.md

## TL;DR

A wintermute kernel primitive can be compiled, packaged, and even installed
and still be runtime-**inert** — `/dev/memlog` exists but the user can't write
it, `agentns` registers an all-zeros session, `bpolicy` reports
`{"loaded": false}`. Today the only way to learn this is three Bash invocations
and a human reading the journal. `quicken-probe` is the foundation of the
`quicken` workspace: a read-only CLI that classifies every primitive's
**liveness** with a verdict and its evidence, in one command. This PRD creates
the `~/wintermute/quicken` cargo workspace and the `quicken` binary.

## Why this exists

- **Evidence — every primitive is dark, confirmed live 2026-06-05.** Direct
  probes this pass: `/dev/memlog` is `crw-rw---- root:root` and the user is not
  in the `memlog` group (EACCES); `/proc/self/agent_session` =
  `00000000000000000000000000000000`; `~/.local/bin/bpolicy status` →
  `{"loaded": false}`. All three reported again in the 2026-06-03 self-review
  journal and in `recall` reflective carryforward — escalated once, never
  resolved.
- **Evidence — one dark primitive degrades a live one.** `getfattr -d
  ~/brain/state/last-run.txt` returns
  `user.prov.session="comm:zsh:pid:64758:uid:1000"` — the *fallback* form, not
  the 128-bit agentns session id provfs is designed to stamp. provfs is "live"
  but degraded *because* agentns is inert. A verdict needs to express that.
- **Evidence — the gap rots and widens.** Installed `linux-wintermute` is
  `pkgrel-5`; the activation fix is built and uninstalled at `pkgrel-11`
  (`ls ~/wintermute/wintermute-kernel/pkg/*.pkg.tar.zst`). The self-review noted
  the gap widening 5→10; it is now 5→11. Nothing tracks this drift structurally.
- **Why a probe, not more journaling:** the `vigil` vision proved the pattern —
  turn a hand-written, re-discovered-every-tick anomaly into a structured
  read-only detector (`binstale`). `quicken-probe` is the sibling detector for
  the *never-activated* axis (vs. vigil's *stale-but-running* axis).

## What this builds

- New cargo workspace at `~/wintermute/quicken` (Rust 2021, `rust-toolchain.toml`
  pinned to 1.85 per lib-crate convention), with a `quicken-probe` lib crate and
  a thin `quicken` binary (`[[bin]]`) hosting subcommands (`quicken probe` first).
  Scaffold follows the local-tool conventions: `sigpipe::reset()` as the first
  line of `main()` (per `self_sigpipe_panic_toolkit`).
- **Core types** (`quicken-probe` lib):
  - `Verdict` enum: `Live`, `LiveDegraded { reason }`, `StagedNotInstalled`,
    `InstalledNotActivated`, `Inert`, `Unknown`.
  - `Evidence`: a structured record (key/value pairs + a free-text detail) so
    every verdict carries the bytes it was derived from.
  - `PrimitiveReport { name, verdict, evidence, checked_at }`.
  - A `Probe` trait: `fn name(&self) -> &str; fn probe(&self, env: &ProbeEnv) ->
    PrimitiveReport;` where `ProbeEnv` abstracts the filesystem roots
    (`/dev`, `/proc/self`, xattr source path, pacman db, pkg dir) so tests can
    point probes at fixtures instead of the real system.
- **Probes** (one impl each, all read-only):
  - `MemlogProbe`: dev-node present? perms/owner? current uid in `memlog` group?
    installed pkgrel vs highest available pkgrel in the pkg dir →
    `StagedNotInstalled` if a higher pkgrel exists, `InstalledNotActivated` if
    installed but group/perms block writes, `Live` if writable.
  - `AgentnsProbe`: read `/proc/self/agent_session`; all-zeros → `Inert`,
    non-zero 128-bit → `Live` (optionally read `/proc/self/agent_counters`).
  - `WardenProbe`: parse `bpolicy status` JSON; `loaded:false` → `Inert`,
    `loaded:true` → `Live`. (Shell out to the configured bpolicy path, or read a
    status file if present; injected via `ProbeEnv` for tests.)
  - `ProvfsProbe`: `getfattr`/`lgetxattr` the configured source path for
    `user.prov.session`; absent → `Inert`; present and `comm:`-form →
    `LiveDegraded { reason: "agentns-fallback session id" }`; present and
    128-bit hex → `Live`.
- **CLI**: `quicken probe` → human table (primitive · verdict · one-line
  evidence); `quicken probe --json` → array of `PrimitiveReport`; non-zero exit
  if any primitive is worse than `Live`/`LiveDegraded` (so it's usable as a gate).

## Acceptance criteria

1. `cargo build --release` produces a `quicken` binary; `quicken probe --help`
   lists the probe subcommand and the `--json` flag.
2. Each of the four probes returns the correct `Verdict` against a **fixture
   `ProbeEnv`** with known inputs (golden tests): e.g. a fixture `/dev/memlog`
   with the real-world perms + no group membership → `InstalledNotActivated`;
   an all-zeros `agent_session` fixture → `Inert`; a `comm:`-form provfs xattr
   fixture → `LiveDegraded`; a 128-bit-hex fixture → `Live`.
3. `MemlogProbe` returns `StagedNotInstalled` when the fixture pkg dir contains a
   higher pkgrel than the fixture "installed" version, and the evidence records
   both pkgrels (deterministic on the known 5-vs-11 fixture).
4. `quicken probe --json` emits valid JSON deserializable back into
   `Vec<PrimitiveReport>` (round-trip test).
5. `quicken probe` exits non-zero when any primitive verdict is worse than
   `LiveDegraded`, and zero when all are `Live`/`LiveDegraded` (tested via two
   fixture environments).
6. No probe panics on a missing surface (absent dev node, unreadable proc file,
   no xattr) — each maps to `Unknown` or `Inert` with evidence, never a crash.
7. Tests perform **zero network access and zero writes outside the test tmpdir**;
   a test asserts the probe path is pure-read (cloud-build-safe).
