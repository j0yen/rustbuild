# PRD: mcp-register — wire the conduit servers into a client's MCP config

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/mcp-register`
**Vision:** visions/conduit.md

## TL;DR

ousia-mcp's AC7 says it ships a client-config snippet but performs **"no
settings.json edits by the PRD itself."** Every conduit server repeats that
stance — correctly, since editing a client's MCP config is a shared-state,
human-visible action. The result: a fleet of working MCP servers that
nothing actually registers, so each must be wired in by hand. This PRD
closes the loop with a proposal-by-default registrar: it emits the exact
config block for each conduit server and, only on `--write`, applies it to
the target MCP config idempotently with a timestamped atomic backup.

## Why this exists

- **The gap is designed-in across the fleet.** ousia-mcp AC7 (read this
  pass) and every conduit server PRD explicitly decline to touch
  settings.json — so registration is genuinely un-owned. mcp-register is
  the single place that owns it, mirroring the fleet's settled
  propose-don't-do pattern (tend-push prints the push command; tide-window
  prints the reboot command; muster-reap is `--confirm`-gated).
- **Editing MCP config is exactly the class that needs care.** The build
  skill's own rule for settings.json is "jq + atomic rename, snapshot first
  to `settings.json.bak.<ts>`." mcp-register encodes that discipline once
  instead of leaving each human to hand-edit JSON and risk a malformed
  config that silently drops every server.
- **The server names are now knowable.** recall-mcp / docket-mcp /
  muster-mcp / provenance-mcp / ousia-mcp each declare a stable
  `<name> serve` invocation; mcp-register registers them by that contract.

## What this builds

A binary crate `mcp-register` (binary `mcp-register`) at
`~/wintermute/mcp-register`. `sigpipe::reset()` first line of `main()`.

- `servers.rs` — a built-in registry of the conduit servers:
  `Server { name, command, args: Vec<String>, description }` (e.g.
  `recall-mcp` → `recall-mcp serve`). The binary path is resolved on PATH
  at register time; an unresolved binary is reported, not silently
  written.
- `config.rs` — read/merge/write a target MCP config. Default target
  `~/.claude.json` (`mcpServers` object); `--project` targets `.mcp.json`
  in cwd; `--target <path>` overrides. Merge is **idempotent**: registering
  an already-present server with identical command/args is a no-op;
  re-registering with changed args updates only that entry. Write is
  temp-file + atomic rename, preceded by a `<target>.bak.<ts>` snapshot.
- `plan.rs` — compute the diff (servers to add / update / already-present)
  without writing.
- `main.rs` — clap:
  - `mcp-register list` — print the built-in conduit server registry and
    whether each binary resolves on PATH.
  - `mcp-register plan [--target <p>|--project]` — print the would-be
    config changes (default; writes nothing).
  - `mcp-register apply --write [--only <name>]... [--target <p>|--project]`
    — perform the idempotent merge with backup. `--write` is **required**
    to mutate; a bare `apply` is a dry-run alias for `plan`.

Deps: `serde_json`, `clap`, `sigpipe`, `anyhow`. (No `mcp-core` dep —
mcp-register configures servers, it isn't one.)

## Acceptance criteria

1. `cargo build --release` produces `mcp-register`; `cargo test` green;
   MSRV 1.85, no let-chains; `sigpipe::reset()` first in `main()`.
2. `mcp-register list` prints the conduit registry (recall/docket/muster/
   provenance/ousia-mcp) with a resolved/unresolved PATH marker per binary.
3. `mcp-register plan --target <tmp.json>` against a fixture config prints
   the add/update/already-present classification and writes nothing
   (asserted: the target file is byte-identical after).
4. **Idempotence**: `apply --write` twice against a temp target produces a
   config where each server appears exactly once; the second run reports
   "no changes" and the file is byte-stable after the first apply.
5. **Atomic + backed up**: `apply --write` creates a `<target>.bak.<ts>`
   snapshot before writing and writes via temp+rename; an injected write
   failure leaves the original target intact (test).
6. A bare `mcp-register apply` (no `--write`) mutates nothing and prints the
   plan — safe by default; `--only recall-mcp` restricts the write to that
   one server.
7. Registering into an existing config preserves unrelated `mcpServers`
   entries and all other top-level keys byte-for-byte except the touched
   `mcpServers` members (test against a fixture with pre-existing
   unrelated servers).
8. README documents the propose-by-default stance, the `--write`/backup
   behavior, `--target`/`--project`, and that it is the one place in the
   conduit fleet permitted to edit a client's MCP config.
