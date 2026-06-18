# Vision: colophon — every file the kernel stamps should say who made it, in words you can query

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-18
**Status:** active
**Seed:** bare `/dream` (auto-mode, no topic) + Phase-1 live inspection on the
now-booted `7.0.11-arch1-1-wintermute` kernel.

## TL;DR

The wintermute kernel is finally *booted*, not just built — `uname -r` reads
`7.0.11-arch1-1-wintermute`, and the **provfs LSM is live**: every file closed
after a write now carries a `user.prov.session` xattr. As of the v0.3 enriched
fallback that string is no longer opaque — it is structured:
`comm-chain:<c0>>c1>c2;env:<KEY>=<val>;cwd:<path>;pid:<n>;uid:<n>`. Reading
this very dream pass's output proves it:

```
$ getfattr -d ~/wintermute/autobuilder/notes/gossip.md
user.prov.session="comm-chain:cat>zsh>claude;cwd:/home/jsy/wintermute/autobuilder;pid:2662703;uid:1000"
user.prov.ts="1781768028"
```

But **nothing in userspace parses it.** `provenance-mcp` — the one consumer —
returns the raw blob and its tests assume the old opaque `"sess-abc123"` form
(`provenance-mcp/src/backend.rs:141`). The kernel is emitting queryable facts
(who, where, when, under which skill) and userspace is throwing the structure
away.

`colophon` is the userspace half that catches it: a canonical parser for the
enriched format, plus two consumers that turn provenance into answers —
*attribute* (which session/skill wrote this pile of files?) and *stale* (which
state/config files were written by a session that's long dead?).

A colophon is the inscription a printer leaves at the end of a book: who set
it, where, when. That is exactly what provfs stamps. This vision makes it
legible.

## End-state

When this is done:

- A single Rust library (`colophon`) is the **one** place the enriched
  `user.prov.session` format is parsed — into a `Provenance` struct with
  `comm_chain: Vec<String>`, `env: BTreeMap`, `cwd`, `pid`, `uid`, and (when
  `CONFIG_AGENT_NS=y` finally lands) the 128-bit `agent_session` id. Both the
  comm-chain fallback form and the opaque agentns-id form parse cleanly.
- The library exposes a **skill-attribution** heuristic: walk the comm-chain
  for `claude-dream-*`, `claude-build*`, `claude-self*`, `Bun Pool *>claude*`
  and resolve the *originating skill* of a write. (Phase-1 confirmed the chain
  for this pass: `Bun Pool N>claude-dream-he…` — the writing skill is
  recoverable today.)
- `colophon attribute <dir>` walks a tree, reads each file's xattr, and emits a
  ranked report grouping files (and bytes) by writing actor / skill — turning
  cruft sweeps from "delete by mtime" into "this 11G of `.build-worktrees` was
  written by /build across N sessions."
- `colophon stale <dir>` flags state/config files whose `prov.ts` predates the
  consuming binary, or whose writing pid/session is no longer alive — a
  provenance-grounded orphaned-state detector for `~/.claude` and
  `~/wintermute`.
- self-review consumes a `colophon digest` block instead of guessing which
  session leaked what.

## Components (one bullet ≈ one PRD)

- **colophon-parse** (NEW, rust-cli+lib at `~/wintermute/colophon`) — the
  canonical parser. `Provenance` struct + `parse(xattr: &str) -> Provenance`
  handling both the enriched comm-chain form and the opaque agentns-id form,
  plus the skill-attribution heuristic. Ships a `colophon parse <file>`
  inspection subcommand. Foundation crate the rest extend.
- **colophon-attribute** (rust-extend colophon) — `colophon attribute <dir>`:
  walk, read xattr, parse, group by actor/skill, ranked report (files + bytes).
  Respects provfs's own skip-prefixes (`target`, `.git`, `node_modules`, `/tmp`,
  `/proc`) so unstamped-by-design paths are reported as *skipped*, never as
  *unattributed*.
- **colophon-stale** (rust-extend colophon) — `colophon stale <dir>`: flag files
  whose `prov.ts` < the consuming binary's mtime, or whose writing pid is dead /
  session is gone. Orphaned-state / stale-config detector.
- **colophon-digest** (rust-extend colophon, mixed: CLI + self-review block) —
  `colophon digest`: run attribute over the known cruft dirs + stale over the
  config dirs, emit a markdown digest block for self-review. Depends on both
  attribute and stale.

## Order

```
colophon-parse ─┬─► colophon-attribute ─┐
                └─► colophon-stale ──────┴─► colophon-digest
```

attribute and stale are independent of each other (parallelizable); digest
converges both.

## Open questions

- Should `provenance-mcp` be re-pointed to depend on `colophon` as its parser
  (so the MCP `file_provenance` tool returns structured fields)? Strongly
  implied, but it lives in the *conduit* vision and is also the *threshold*
  vision's "next-session reads provenance" territory — left as a deliberate
  boundary, not folded in here. A follow-on `colophon-mcp` PRD can do it once
  the lib is proven.
- When agentns finally activates (blocked on the EINVAL `CLONE_NEWAGENT` flag
  collision; see the *assay* vision), `user.prov.session` switches to the
  128-bit id form. colophon-parse must already handle it — its ACs require both
  forms parse today so the switch is a no-op. No new PRD needed for that
  transition.
- History ring (`user.prov.history`, provfs Phase 2) is deferred in the kernel;
  no colophon component consumes it until it ships.
