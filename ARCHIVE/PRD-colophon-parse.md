# PRD: colophon-parse — one parser for the provenance the kernel now stamps

Status: Draft v0.1
build_target: rust-cli
Vision: visions/colophon.md

## TL;DR

The booted `7.0.11-arch1-1-wintermute` kernel stamps a structured
`user.prov.session` xattr on every file, but no userspace tool parses the
structure. This PRD builds `colophon`, a new Rust CLI+library whose
`Provenance` model and `parse()` function are the single canonical decoder of
the enriched provfs format. It is the foundation crate the rest of the
`colophon` fleet extends.

## Why this exists

Phase-1 inspection, 2026-06-18:

- The wintermute kernel is **booted, not just built**: `uname -r` →
  `7.0.11-arch1-1-wintermute`; `pacman -Q linux-wintermute` →
  `7.0.11.arch1-1`. provfs is live.
- The xattr on this dream pass's own output is structured, not opaque:
  `getfattr -d ~/wintermute/autobuilder/notes/gossip.md` →
  `user.prov.session="comm-chain:cat>zsh>claude;cwd:/home/jsy/wintermute/autobuilder;pid:2662703;uid:1000"`
  and `user.prov.ts="1781768028"`.
- The format is documented at `~/wintermute/provfs/lsm/provfs_lsm.c:24` and
  `README.md:78`:
  `comm-chain:<c0>>c1>c2;env:<KEY>=<val>;cwd:<path>;pid:<p>;uid:<u>`. Any field
  may be absent; pid+uid are the floor.
- The **only** existing consumer, `provenance-mcp`, throws the structure away:
  `src/backend.rs:37` strips the `user.prov.session=` prefix and returns the
  remainder as one opaque string; its tests assume the old form
  (`backend.rs:141`: `user.prov.session="sess-abc123"`). The richer
  `comm-chain:…` form returned today is never decomposed.
- Real samples across `~/wintermute/autobuilder/*.md` show the writing skill is
  recoverable from the chain — e.g. `comm-chain:Bun Pool 3>claude-dream-he…`
  identifies a /dream write. Nothing extracts that today.
- When agentns activates (currently blocked on the EINVAL `CLONE_NEWAGENT`
  collision — see `visions/assay.md`), `user.prov.session` becomes a 128-bit id
  instead. A parser that handles only one form will break on the switch.

## What this builds

New repo `~/wintermute/colophon/` (Rust workspace, MSRV 1.85, edition 2021;
follows the autobuilder scaffold incl. `sigpipe::reset()` as the first line of
`main` per the standing SIGPIPE-panic lesson).

- **Model:** a `Provenance` struct — `comm_chain: Vec<String>`,
  `env: BTreeMap<String,String>`, `cwd: Option<String>`, `pid: Option<u32>`,
  `uid: Option<u32>`, `agent_session: Option<String>` (the 128-bit form),
  `ts: Option<u64>`, and `raw: String`. A `Form` enum distinguishing
  `CommChain` vs `AgentId` vs `Unstamped`.
- **`parse(session: &str, ts: Option<&str>) -> Provenance`** — pure, total,
  never panics. Splits the enriched form on `;`, decodes `comm-chain:` on `>`,
  `env:` on `=`, and the scalar `cwd`/`pid`/`uid` fields; recognizes a bare
  32-hex-char value (non-zero) as the agentns-id form; treats an all-zero id or
  empty value as `Unstamped`. Tolerates missing/extra/reordered fields.
- **`read_file(path) -> io::Result<Provenance>`** — shell out to
  `getfattr -d --absolute-names <path>`, parse both `user.prov.session` and
  `user.prov.ts`. Absent xattrs → `Form::Unstamped`, not an error.
- **Skill attribution:** `fn originating_skill(&self) -> Option<Skill>` — walk
  `comm_chain` matching `claude-dream*` → Dream, `claude-build*` → Build,
  `claude-self*` → SelfReview, else `Claude` if any `claude`/`Bun Pool*>claude`
  link, else `None`. Pure, table-driven, unit-tested against real captured
  chains.
- **`colophon parse <file>`** subcommand: read + parse + render. `--format
  text|json`. `--from-string <xattr>` to parse a literal value (testing / piping
  `getfattr` output) without touching the filesystem.
- README + install script (`~/.local/bin/colophon`).

Out of scope (later PRDs): tree-walking attribution (colophon-attribute),
stale detection (colophon-stale), the self-review digest (colophon-digest),
re-pointing provenance-mcp (open question in the vision).

## Acceptance criteria

1. `cargo build` and `cargo test` are green; `cargo clippy` adds no new warnings
   over the autobuilder baseline; binary installs to `~/.local/bin/colophon`.
2. `parse()` decodes the live sample
   `comm-chain:cat>zsh>claude;cwd:/home/jsy/wintermute/autobuilder;pid:2662703;uid:1000`
   into `comm_chain == ["cat","zsh","claude"]`, `cwd == Some("/home/jsy/wintermute/autobuilder")`,
   `pid == Some(2662703)`, `uid == Some(1000)`, `Form::CommChain`.
3. `parse()` decodes an `env:`-bearing sample
   (`comm-chain:bash>claude;env:CLAUDE_TOOL=/build;cwd:/x;pid:1;uid:1000`) with
   `env["CLAUDE_TOOL"] == "/build"`.
4. A 32-hex-char non-zero value parses as `Form::AgentId` with
   `agent_session` set; an all-zero id and an empty string both parse as
   `Form::Unstamped`. No input — including malformed, truncated, or reordered —
   causes a panic (proven by a fuzz-style table of bad inputs).
5. `originating_skill()` returns `Dream` for a chain containing
   `claude-dream-he…`, `Build` for `claude-build…`, `SelfReview` for
   `claude-self…`, and `None` for a pure `bash>zsh>xterm` chain (fixtures from
   real captured xattrs).
6. `colophon parse --from-string '<value>' --format json` emits the structured
   fields; `colophon parse <real-file>` round-trips a file actually stamped by
   provfs on this machine (integration test gated on provfs presence, skipped
   with a logged note if `getfattr` reports no `user.prov.session`).
7. `colophon parse` piped to `head` does not panic (SIGPIPE reset verified).
