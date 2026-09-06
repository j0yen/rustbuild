# PRD: rustbuild-hermetic-scope — the hermetic receipt blames the build, not the machine

- Status: building
- Lane: RedBaron 2026-09-06T07:51:28Z
- build_target: rust-extend
- build_into: /home/jsy/wintermute/rustbuild
- build_version_bump: minor
- build_priority: high
- test_prefix: [hermetic_scope]
- publish: j0yen/private
- Vision: visions/buildloop-operations.md
- PM: Joe
- Drafted: 2026-09-05
- Engineering target: j0yen/rustbuild `autobuilder/crates/extended-gates` (the `hermetic-build` producer)
- Blocked: gate-red at 824341f (v0.4.0, HEAD moved since the last gate run because the sibling PRD-autobuilder-rollback-tag-aware landed on the same `build_into` in between) — re-ran `extend-gate.sh` fresh at this HEAD per the archive gate's check #6 (a moved HEAD invalidates a prior gate run). pass=11 block=14. hermetic-build itself (this PRD's subject) is CONFIRMED CORRECT: `new_sockets: []` in its own receipt — the block is `cargo_exit_code: 101`, not an attribution failure — and all 8 of this PRD's own AC tests (`hermetic_scope_ac1..ac8`, run directly against the crate at `autobuilder/` with real cargo builds) pass. The exit-101 is a newly-surfaced pre-existing infrastructure defect, not this PRD's bug: `extend-gate.sh` invokes `extended-receipts.sh <build_into>` with the REPO ROOT as `--project`, but this crate's Cargo.toml has always lived one level down at `autobuilder/Cargo.toml` (confirmed: no root Cargo.toml at any commit in history) — so every cargo-invoking producer run against the raw repo root fails with "could not find Cargo.toml" (cargo_exit 101), not just hermetic-build. Same root cause explains cold-build-time and flake-audit blocking here too. Fixing that path resolution lives in the shared `~/.claude/skills/build/scripts/extend-gate.sh` / `~/.claude/skills/rustbuild/scripts/extended-receipts.sh` tooling, not in this PRD's `build_into` — out of scope for this PRD, flagged as a follow-on gap. Remaining blockers are the same class of pre-existing, unrelated-to-this-PRD gaps already noted: intake/risk-gate/reviewer-agent missing (repo never onboarded to its own gate), rollback-plan block (435/816 commits since base 48165ad6 not revert-clean; rustbuild is a tool crate correctly left on the default revert-commits model, not the tag-aware redeploy model), ci-checks block (no workflow runs yet for this HEAD), supply-audit/license-audit/sbom/determinism missing (no receipt — likely the same repo-root-vs-autobuilder path defect), secrets-scan block (same planted-fixture false positive as before, by design), ac-traceability block (crate root carries stale unrelated legacy PRD-fluid-*/PRD-carbon-node-identity/PRD-cradle-bake-integration files from 2026-09-01/02 — traceability picks one of those instead of finding no PRD at all, same underlying gap as before). Onboarding this repo onto its own gate, and fixing the shared gate script's project-path resolution, are both out of scope for this PRD.

## TL;DR

The hermetic-build producer snapshots machine-wide sockets before and after its cargo build and blocks on any new one. On 2026-09-05 it blocked the mcphost gate twice on sockets the build never opened — first three HTTPS connections from the co-running review session, then a single tcp6 :443 keepalive from an unrelated daemon — costing gate reruns each time on a workstation that always has cohabiting network activity. Scope the capture to the build's own process tree and the receipt stops lying in both directions.

## Problem statement

RedBaron is Joe's workstation: agorabus keeps a NATS link to the hub, tailscale chats, Claude sessions call the Anthropic API, snaps refresh. The hermetic receipt's machine-wide before/after socket diff cannot distinguish any of that from the crate build it judges, so on this fleet it produces false blocks routinely (two documented on 2026-09-05: `new_sockets` carrying Anthropic-API and one :443 keepalive with `cargo_exit_code: 0`) — and, symmetrically, it would miss a genuinely non-hermetic build step whose connection closed before the after-snapshot. Each false block stalls the deploy gate until a human reruns the producer in a lottery for a quiet window; tonight that lottery is literally the recovery plan.

## Goals

- The receipt's `new_sockets` contains sockets attributable to the build's own process tree — cargo and its descendants — and nothing else.
- A build that opens no network is `pass` regardless of what else the machine is doing; a build step that phones out is `block` even if brief.

## Non-goals

- No network namespace enforcement (an `unshare -n` build would break sccache's local socket and rustup shims in ways this PRD does not take on; detection stays observational).
- No policy change: what counts as hermetic (no new outbound sockets) is unchanged — only attribution changes.
- No changes to other extended producers.

## User stories

1. As the mcphost deploy gate, the hermetic verdict reflects the crate's build, so a green gate never waited on a quiet-machine lottery and a red one names a socket cargo actually opened.
2. As a build agent whose build.rs sneaks a download, the receipt blocks with the offending process and destination, even though the fetch finished in two seconds.
3. As Joe reading a hermetic block, the receipt shows pid, process name, and destination for each attributed socket, so judging it takes seconds.

## Requirements

P0 — Attribution by process tree: the producer runs its cargo build as a tracked child and samples `/proc/<pid>` descendants' socket inodes (fd → socket inode joined against /proc/net/tcp{,6}/udp{,6}) during the build window, at a sampling interval ≤1 s; `new_sockets` lists only sockets owned by the build's tree, each entry carrying pid, comm, and remote address.
P0 — Machine noise immunity: a cohabiting process opening connections during the window must not appear in the receipt (verified by test: spawn a socket-opening bystander during a hermetic fixture build → pass).
P0 — True-positive retention: a child of the build opening an outbound connection appears even when it closes before the build ends (the sampling catches it live or via short-lived-socket accounting; verified by a fixture whose build script connects briefly → block).
P1 — Receipt schema bumps to v2 (`autobuilder.hermetic_build_receipt.v2`) adding per-socket attribution fields; the gate accepts both versions during transition.
P1 — Loopback and unix-domain sockets are ignored by default (sccache's local protocol is not network egress); the receipt records the ignore rule applied.
P2 — `--strict` flag including loopback, for crates that must not even talk locally.

Edge cases: sampling misses an ultra-short socket between ticks (documented residual risk; interval and the accounting mechanism bound it — state the bound in the receipt); build tree exceeding sampling capacity (hundreds of children — sampler walks the tree iteratively, never recurses unbounded); /proc read races as children exit (ESRCH is skipped silently); cargo_exit_code nonzero (receipt is `crash`-equivalent as today, attribution list still emitted).

## Success metrics

| metric | baseline | target | method | timeframe |
|---|---|---|---|---|
| false hermetic blocks on cohabited builds | 2 on 2026-09-05 alone | 0 | gate journal + receipts month-over | first month |
| gate reruns spent on hermetic retries | ≥2 tonight | 0 | operator journal | first month |
| attributed evidence per blocked socket | none (bare address) | pid+comm+remote on every entry | receipt inspection | at ship |

## Technical considerations

Extends the extended-gates crate (Rust). The sampler is a thread polling /proc while cargo runs — no ptrace, no eBPF, no root; the ≤1 s interval and inode-join approach match what procstat already proved workable on this fleet. Short-lived-socket accounting can additionally read /proc/net/tcp's per-inode uid field to filter obvious foreign-uid sockets cheaply. Receipt v2 keeps every v1 field so downstream jq stays valid.

## Migration / compatibility

The gate's verdict consumption is unchanged (`verdict` field). v1 receipts remain readable; extended-receipts.sh needs no changes (same binary name and CLI). First ship regenerates mcphost's receipt at next gate run.

## Open questions

| question | owner | due |
|---|---|---|
| Is a netns-based enforcing mode (with sccache accommodations) worth a follow-on once observational attribution is trusted | Joe | after a month of v2 receipts |

## Acceptance criteria

1. P0 — Given a hermetic fixture crate and a bystander process opening outbound sockets during the build, When hermetic-build runs, Then the verdict is pass and new_sockets is empty.
2. P0 — Given a fixture whose build script opens a brief outbound TCP connection, When hermetic-build runs, Then the verdict is block and the entry carries the build-tree pid, comm, and remote address.
3. P0 — Given the same fixture builds with no network activity anywhere, When hermetic-build runs, Then pass with empty new_sockets.
4. P0 — Given children exiting mid-sample, When the sampler walks the tree, Then ESRCH races are tolerated and the run completes.
5. P1 — Given sccache active over its local socket, When a hermetic fixture builds, Then loopback/unix traffic does not appear and the receipt names the ignore rule.
6. P1 — Given a v1 receipt on disk from before the ship, When autobuilder gate reads it, Then it is accepted during the transition window.
7. P0 — Given cargo exits nonzero, When the producer finishes, Then the receipt records the exit code and the attribution list gathered up to that point.
8. P2 — Given --strict, When a fixture talks to loopback, Then the verdict is block with the loopback entry attributed.
