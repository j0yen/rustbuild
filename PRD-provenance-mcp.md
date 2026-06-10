# PRD: provenance-mcp — kernel-stamped file provenance & context snapshots over MCP

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/provenance-mcp`
**Vision:** visions/conduit.md

## TL;DR

The wintermute kernel already answers two forensic questions no userspace
plumbing has to invent: **"who wrote this file"** (provfs stamps
`user.prov.session`/`user.prov.ts` xattrs on every closed-after-write file)
and **"what was Claude thinking just before it compacted"** (memlog is a
per-uid circular log of pre-compaction context snapshots). This PRD exposes
both as read-only MCP tools, so an agent can attribute a file or pull
recent context snapshots through a connector instead of shelling `getfattr`
and `memlog show`.

## Why this exists

- **The kernel surface is shipped and stable.** Per the conduit vision's
  Phase-1.5 survey and live check: provfs stamps `user.prov.session` /
  `user.prov.ts` (readable via `getfattr -d <file>`), skip-prefixed for
  `/proc`,`/tmp`,`.git`,`target`,`node_modules`; memlog's reader is
  `cli/memlog show --since <DUR> --limit N --format json`. These are the
  exact primitives Phase 1.5 says to *consume from userspace* rather than
  reinvent.
- **Provenance is the forensic half the other conduit servers don't
  cover.** recall/docket/muster expose agent-level state; provenance
  exposes filesystem- and context-level history. Together they let an
  external agent reconstruct "what happened on this box."
- **It rides the conduit core.** Thin `Tool` layer over `mcp-core`; shells
  `getfattr` and the memlog CLI. No new protocol code.

## What this builds

A binary crate `provenance-mcp` (binary `provenance-mcp`) at
`~/wintermute/provenance-mcp`. `sigpipe::reset()` first line of `main()`.
Depends on `mcp-core` (path).

- `backend.rs`:
  - `file_provenance(path)` — runs `getfattr -d --absolute-names <path>`,
    parses the `user.prov.session` / `user.prov.ts` xattrs into
    `{path, session, ts, raw}`; a path with no prov xattrs (skip-prefixed
    or never written) returns `{path, session:null, ts:null}` — not an
    error.
  - `memlog_recent(since, limit)` — runs `memlog show --since <DUR>
    --limit <N> --format json` (memlog CLI path default resolved on PATH;
    `--memlog-bin`/`MEMLOG_BIN` override), returns the parsed snapshot
    array.
  - Both via `std::process::Command`; verbs are fixed literals
    (`getfattr` read-only; `memlog show` only — never `write`/`clear`).
- `tools.rs` — two `Tool` impls: `file_provenance {path: string}`,
  `memlog_recent {since?: string, limit?: integer}`. JSON-Schema
  `input_schema` each.
- `main.rs` — clap: `provenance-mcp serve [--memlog-bin <path>]` registers
  the two tools and calls `mcp_core::serve_stdio`.

Deps: `mcp-core` (path), `serde_json`, `clap`, `sigpipe`, `anyhow`.

## Acceptance criteria

1. `cargo build --release` produces `provenance-mcp`; `cargo test` green;
   MSRV 1.85, no let-chains; `sigpipe::reset()` first in `main()`.
2. `provenance-mcp serve` completes `initialize` and `tools/list` returns
   exactly the two read tools with valid JSON-Schema `inputSchema`.
3. `file_provenance {path}` parses a fixtured `getfattr -d` output (a
   captured string with `user.prov.session=…` / `user.prov.ts=…`) into the
   `{path, session, ts}` shape — tested against the fixture, no real
   getfattr call needed; a fixture with no prov xattrs yields
   `session:null, ts:null` and exit-success (not an error).
4. **Read-only by construction**: a test asserts the only external verbs
   are `getfattr` (read) and `memlog show`; a guard rejects `setfattr`,
   `memlog write`, `memlog clear` appearing as command literals.
5. `memlog_recent {since:"1h", limit:5}` invokes `memlog show --since 1h
   --limit 5 --format json` and returns the parsed array (verified against
   a stubbed `memlog` binary on PATH; offline).
6. Missing required `path` on `file_provenance` → JSON-RPC error, no panic,
   no shell call; absent a usable `getfattr`/`memlog`, `serve` still starts
   and the affected tool returns a clean `ToolError`.
7. README documents both tools, the read-only guarantee, the provfs/memlog
   kernel dependency (and graceful behavior when the kernel features aren't
   present), and an MCP client config snippet; no settings.json edits
   performed by this PRD.
