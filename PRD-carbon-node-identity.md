# PRD-carbon-node-identity

**Status:** Draft v0.1
**Vision:** visions/carbon.md
**build_target:** rust-cli
**build_into:** (new repo — j0yen/wm-node)

## TL;DR

There is no explicit node identity anywhere in wintermute. `grep WM_NODE
~/.config/wintermute ~/.config/systemd/user /etc/wintermute` returns **zero
hits** — a daemon "belongs" to the laptop only because that is where it happens
to be installed. Before any work can move to the cloud and the laptop can become
"a node like any other," every node needs a declared identity (`WM_NODE`) and a
**role** (`voice`, `hub`, `builder`), plus a tiny resolver that answers the only
question relocation cares about: *should this daemon run on this node?*

## Why this exists

**Evidence (2026-06-20):**
- Zero `WM_NODE` references in any config tree. Node identity is implicit
  (hostname). Placement decisions ("homeward-ingest runs on the laptop") are
  accidents of install location, not declarations.
- constellation already hand-set `WM_NODE=ryzen7` in one PRD's bootstrap env
  (see PRD-constellation-wm-daemons-ryzen) but nothing *reads* it — it is a
  string in a file with no consumer.
- The carbon vision's whole premise ("move non-voice work to the cloud; laptop
  is a peer") requires a declarative placement layer. Without it, "move X to the
  hub" is a manual rsync with no way to assert X is in the right place.

## What this builds

A small Rust CLI + library, `wm-node`, installed to `~/.local/bin/wm-node`:

- **Identity file** `~/.config/wintermute/node.toml`:
  ```toml
  name = "carbon"          # WM_NODE
  roles = ["voice"]        # any of: voice, hub, builder
  fleet = "wintermute"
  ```
- **`wm-node id`** → prints the node name (for scripts / env).
- **`wm-node role <role>`** → exit 0 if this node has the role, 1 otherwise.
  Lets a systemd unit gate itself: `ExecCondition=wm-node role hub`.
- **`wm-node should-run <daemon>`** → consults a placement table
  (`~/.config/wintermute/placement.toml`, e.g. `homeward-ingest = "hub"`) and
  exits 0/1 so a unit only starts where it is assigned.
- **`wm-node env`** → emits `WM_NODE=…` / `WM_ROLES=…` for `EnvironmentFile=`.
- Library crate exposes `Node::load()`, `Node::has_role()`, `placement_of()`.

Deps: `serde`, `toml`, `clap`. MSRV 1.85, no let-chains. `sigpipe::reset()` first
line of `main()` (per the SIGPIPE-panic toolkit lesson).

## Acceptance criteria

1. `wm-node id` prints the `name` from `node.toml` (and falls back to `hostname`
   if the file is absent), exit 0.
2. `wm-node role hub` exits 0 on a node whose `node.toml` lists `hub`, exits 1
   otherwise — verified with two fixture files.
3. `wm-node should-run homeward-ingest` consults `placement.toml` and exits 0
   only when the mapped node equals this node's name.
4. `wm-node env` output is valid for systemd `EnvironmentFile` (KEY=VALUE lines,
   no quotes-needed values), asserted by a parse test.
5. `cargo test --release` green; binary installs to `~/.local/bin/wm-node`;
   `sigpipe::reset()` present (piping `wm-node id | head` does not panic).
