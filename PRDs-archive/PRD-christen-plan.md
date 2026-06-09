# PRD: christen-plan — the launch-site model and the route plan

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/christen`
**Vision:** visions/christen.md

## TL;DR

The wintermute agent-namespace substrate is built and booted but inert:
every session is born in the *initial* namespace with session id `0…0`
because no launch path routes through `agentns-claude`. Before anything can
fix that, there must be a typed, testable model of **where sessions are
born** (`LaunchSite`), **whether each is wrapped** (`WrapState`), and **what
edit would route it through the wrapper** (`RouteAction`). **christen-plan**
creates the `christen` workspace and that foundation: the shared types, a
`LaunchSiteSource` trait that abstracts discovery, a `FakeSource` for tests,
and a **pure** `plan(sites, kernel, wrapper_installed)` that emits a
print-only `RoutePlan`. `christen plan` shows it at a glance. The rest of
the vision extends this crate.

## Why this exists

- **The substrate is inert, measured 2026-06-05.** `/proc/self/ns/agent ->
  agent:[4026531996]` (the init NS); all three live Claude PIDs read
  `agent_session = 00000000000000000000000000000000`; all `agent_counters`
  zero. `CONFIG_AGENT_NS=y` — the kernel works; the gap is pure userspace
  wiring.
- **The launchers exist but aren't on the path.** `~/.local/bin/agentns-claude`
  (Rust, `--intent`/`--budget`/`--no-unshare`) and `~/wintermute/agentns/userspace/agent-wrap`
  (C, needs `setcap cap_sys_admin+ep`) are installed; `agorabus-session-start.sh:51`
  already reads `/proc/self/agent_session` expecting a real id "once
  `unshare(CLONE_NEWAGENT)` has been called (via agentns-claude)" — and
  falls back because it never is.
- **The launch sites are concrete and editable.** `~/.config/systemd/user/`
  holds `claude-build.service` (`ExecStart=/home/jsy/.local/bin/claude-build-headless.sh`),
  `claude-dream.service`, `claude-self-review.service` — deterministic
  ExecStart lines christen can rewrite. The interactive shell site is
  user-typed and can only be advised.
- **Open docket `agentns-session-zeros`** (8 reports since 2026-05-30, no
  playbook) is exactly this. The model this PRD builds is the prerequisite
  for resolving it.
- This is the FIRST PRD of the fleet: it creates the repo + shared types +
  the `LaunchSiteSource` trait. relay/concord/quicken/keel/anchor/coda all
  learned the rule — **no rust-extend starts until the corpus repo has
  SHIPPED** or extend-validate fails. christen-plan is christen's corpus.

## What this builds

New cargo workspace at `~/wintermute/christen` with a `christen` binary crate.

**The launch-site model.** A `LaunchSite` is a place sessions are born;
`plan` is the pure function that decides what each needs.

- `SiteKind` enum: `SystemdUnit { unit: String, exec_start: String }`,
  `ShellRc { path: PathBuf }`, `Hook`, `Other { note: String }`.
- `WrapState` enum: `Unwrapped`, `Wrapped { via: String }`, `Uncertain`.
  Derived from whether the site's command already invokes `agentns-claude` /
  `agent-wrap` (string match on the exec line) — NOT from a live `/proc`
  read (that is christen-detect's job; `plan` stays pure).
- `LaunchSite { id: String, kind: SiteKind, wrap: WrapState, intent: String }`
  — `intent` is the derived `intent_tag` (e.g. `claude-build.service` →
  `/build`, `claude-dream.service` → `/dream`, `claude-self-review.service`
  → `/self-review`, interactive → `interactive`). The derivation table is a
  pure `intent_for(site_id) -> String`.
- `RouteAction` enum:
  - `Wire { site: String, from: String, to: String }` — the exec line
    rewritten to route through `agentns-claude --intent <intent> --budget
    <default> -- <original>`; declarative, no edit performed here.
  - `Advise { site: String, snippet: String }` — for shell/user sites that
    christen can only print (e.g. an alias for `~/.zshrc`).
  - `AlreadyWrapped { site: String }` — site already routes; no-op.
  - `Skip { site: String, reason: String }` — e.g. stock kernel, or wrapper
    not installed.
- `RoutePlan { actions: Vec<RouteAction>, to_wire: usize, advised: usize,
  already: usize, skipped: usize }` — declarative diff, **no side effects**.
  christen-route executes the `Wire`/`Advise` actions; christen-cap reads
  the same plan for the binary paths it must `setcap`.

**The `LaunchSiteSource` trait** — abstracts discovery so `plan` is pure and
testable and so christen-route/christen-cap share one contract:

```rust
pub trait LaunchSiteSource {
    fn sites(&self) -> Result<Vec<RawSite>>;   // (id, kind, raw_exec_line) per discovered site
}
```

`RawSite { id, kind, exec_line }` is the source's raw observation; `plan`
turns `&[RawSite]` + a `KernelInfo { agent_ns: bool, release: String }` +
`wrapper_installed: bool` into typed `LaunchSite`s and a `RoutePlan`.
christen-plan ships a `FakeSource` (in-memory fixtures) for tests; the real
`SystemdSource` (parses `~/.config/systemd/user/*.{service}` ExecStart) lands
in christen-route.

**Config** — `~/.config/christen/christen.toml`, optional:

```toml
default_budget = "wall=7200s,fork=2000"   # injected into every Wire action
systemd_dir = "/home/jsy/.config/systemd/user"
[intent_overrides]                         # site-id -> intent_tag, overrides the derivation table
"claude-build.service" = "/build"
```

`ChristenConfig::load(path)` parses it; defaults ship in
`config/christen.example.toml`. Loading is pure — no filesystem scan, no
unit parse, no `/proc` read.

**UX:** `christen plan` → table of `site | kind | wrap | intent | action`;
`--format json` for machines. Print-only — no `--apply` in this PRD (that is
christen-route). Exit non-zero when ≥1 site is `Unwrapped` on a `-wintermute`
kernel with the wrapper installed (so a hook can gate on it).

**Deps:** minimal — `serde`/`serde_json`, `toml`, `clap`. MSRV 1.85, no
let-chains (`self_recall_baseline_gate_red`). `sigpipe::reset()` first line
of `main()` (`self_sigpipe_panic_toolkit` — `christen plan | head`). Match
the workspace shape of sibling toolkit repos (vigil/quicken/keel/anchor/coda):
`clippy.toml`, `deny.toml`, `rust-toolchain.toml`, `CHANGELOG.md`, `README.md`.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed offline; a test asserts `plan`
   makes **zero** source calls (it operates only on the passed `&[RawSite]`
   + injected `KernelInfo` + `wrapper_installed`).
2. `LaunchSite`, `SiteKind`, `WrapState`, `RouteAction`, `RoutePlan` are
   public and `serde`-(de)serializable; a round-trip test covers each.
3. `ChristenConfig::load` parses `config/christen.example.toml`; a fixture
   yields the expected `default_budget` + `systemd_dir` + one
   `intent_overrides` entry, and an absent file yields documented defaults.
4. `plan` classifies correctly against `FakeSource` fixtures: a systemd site
   whose exec line lacks `agentns-claude` on a `-wintermute` kernel with the
   wrapper installed → `Unwrapped` + `Wire` (the `to` line contains
   `agentns-claude --intent <derived> --budget <default> --`); a site whose
   exec line already contains `agentns-claude` → `Wrapped` + `AlreadyWrapped`;
   a shell-rc site → `Advise`; any site on a kernel with `agent_ns:false` or
   `wrapper_installed:false` → `Skip` with the documented reason.
5. `intent_for` derives `/build`/`/dream`/`/self-review`/`interactive` for
   the four canonical site ids, and `intent_overrides` from config wins over
   the derivation; both covered by tests.
6. `christen plan --format json` emits one entry per site plus the
   `RoutePlan` tallies (`to_wire`/`advised`/`already`/`skipped`); schema
   matches the documented `RoutePlan`.
7. `christen plan` exits non-zero when ≥1 site is `Unwrapped` (wrapper
   installed, `-wintermute` kernel), zero otherwise; two integration cases
   driving `FakeSource`. `christen plan | head -1` does not panic (SIGPIPE
   reset verified by a test that closes the read end early).
8. README documents the config format, the type surface, the `intent_for`
   derivation table, and the `LaunchSiteSource` trait so christen-detect /
   christen-route / christen-cap / christen-ledger have a contract to extend.
