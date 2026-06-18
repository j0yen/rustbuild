# PRD: tether-tools — the laptop's tools, callable from the work node

Status: Draft v0.1
build_target: rust-cli
Vision: visions/tether.md
deferred_acs: [7]

## TL;DR

The problem: the wintermute toolkit (`recall`, `ctrace`, `procstat`, `wchg`,
`pevent`, …) lives in the laptop's `~/.local/bin`. The work node doesn't have
them, and re-installing the whole kernel-coupled toolchain there is exactly what
constellation deferred. `tether-tools` takes the lighter path the user asked for
— "share your tools": the laptop advertises an allowlisted subset of its tools
over the fleet bus and executes allowlisted invocations on behalf of the work
node, returning stdout + exit code. The work node *calls the tools on the machine
that has them*, deny-by-default and arg-sanitized.

## Why this exists

- The local toolkit (memory `feedback_local_tools`) is eight-plus built Rust
  CLIs, several of which are coupled to this laptop's kernel features (ctrace =
  eBPF, bpolicy = eBPF-LSM, agentns) and cannot simply be copied to the work box.
  Remote-invocation sidesteps the porting problem entirely.
- `wm-busbridge` forwards `wm.fleet.>`, so `wm.fleet.tools.manifest` and
  `wm.fleet.tools.invoke` cross the link for free once `tether-link` is up.
- The user said "share your tools" by name. The honest scope is *remote
  capability*, not *remote install*: the work node gets the tools' *results*,
  computed where the tools (and their kernel/data dependencies) actually live.
- This is the highest-risk component (it executes commands on request), so it is
  built last in the fleet, deny-by-default, with the allowlist + arg-sanitization
  as the load-bearing, grep-asserted safety surface (vision OQ#4).

## What this builds

A `wm-tether-tools` binary (single crate, `~/wintermute/tether-tools`):

- **Responder (laptop side).** Reads an allowlist from
  `~/.config/wm-tether-tools/allow.toml` — each entry names a tool, its absolute
  path, and a permitted-flags spec. On `wm.fleet.tools.manifest` requests it
  replies with the advertised tools + their permitted-flag specs (never the raw
  filesystem). On `wm.fleet.tools.invoke` `{req_id, node, tool, args[]}` it:
  validates the tool is allowlisted; validates every arg against the permitted
  spec (no `;`, `|`, backtick, `$(`, redirection, or path-escape; flags must be
  in the allowed set); runs the binary with `Command` (argv array, **no shell**),
  timeout-bounded; replies on `wm.fleet.tools.result.<req_id>` with `{stdout
  (size-capped), exit_code, truncated, duration_ms}`.
- **Requester (work-node side).** `wm-tether-tools list` prints the advertised
  manifest; `wm-tether-tools run <tool> -- <args...>` invokes a tool remotely and
  prints its stdout, propagating the remote exit code locally. Bounded timeout;
  clean non-zero exit if no responder.
- **Deny-by-default.** A tool not in `allow.toml` is never advertised and never
  executed (rejected with an error reply). A flag outside the permitted spec is
  rejected before any process spawn.
- `wm-tether-tools status` reports responder reachability; `config-example`
  prints a starter `allow.toml` (recall/ctrace-query/procstat — read-only tools
  first). SKIPs honestly when no link is configured.

Deps: `clap`, `serde`/`serde_json`, `toml`, async NATS client (embedded test
server for tests), `sigpipe`. MSRV 1.85, edition 2021. `sigpipe::reset()` first
in `main()`. No `unsafe`. The subprocess path must never construct a shell
command line (grep-asserted: no `sh -c`).

## Acceptance criteria

1. `wm-tether-tools config-example` prints a valid `allow.toml` (parses via the
   crate's loader) listing read-only tools with permitted-flag specs and absolute
   paths; no secrets present.
2. A `wm.fleet.tools.manifest` request replies with exactly the allowlisted tools
   and their permitted-flag specs — a tool absent from `allow.toml` does not
   appear (deny-by-default, embedded NATS / captured sink).
3. An `invoke` for an allowlisted tool with permitted args runs it via an argv
   array (no shell) and replies with the captured stdout, exit code, and a
   `duration_ms`; stdout is size-capped with `truncated` set when clamped.
4. An `invoke` for a tool NOT in the allowlist is rejected with an error reply and
   no process is spawned (assert via a spawn-counter / fake runner).
5. Arg sanitization: an `invoke` whose args contain a shell metacharacter
   (`;`, `|`, `` ` ``, `$(`, `>`), a disallowed flag, or a path-escape is rejected
   before any spawn (table-driven test over a metacharacter corpus).
6. Requester timeout: with no responder, `wm-tether-tools run` exits non-zero
   within the bounded timeout and does not hang.
7. (deferred — embedded NATS) End-to-end: requester `run <tool>` →
   `wm.fleet.tools.invoke` → responder executes a fixture tool (e.g. a stub that
   echoes argv) → result returns and the requester prints stdout + propagates the
   exit code, exactly one reply per req_id.
8. `cargo test` green; no `unsafe`; no `sh -c` in the source (grep-asserted);
   `sigpipe::reset()` first in `main()`.
