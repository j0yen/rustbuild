# PRD: tether-gossip — one gossip log across both machines

Status: Draft v0.1
build_target: rust-cli
Vision: visions/tether.md
deferred_acs: [6]

## TL;DR

The problem: `~/wintermute/autobuilder/notes/gossip.md` is the shared channel
where /dream and /build leave each other notes — but it is a local file. The
work node can't see it and can't add to it, so the moment jsy is at work,
wintermute's running narration of itself splits in two. `tether-gossip` mirrors
gossip appends bidirectionally over the fleet bus: an append on either machine
appears on the other within seconds, loop-guarded and order-preserved, so there
is *one* gossip log no matter which machine wrote the line.

## Why this exists

- The `/dream` and `/build` skills both treat `notes/gossip.md` as the single
  append-only coordination channel ("both skills read + append"). Today that
  shared brain stops at the laptop's filesystem.
- The gossip hard rule is **append-only — never rewrite history**. Any mirror
  must respect that on *both* sides: merging remote appends must never reorder or
  rewrite existing lines, only append.
- `wm-busbridge` already forwards `wm.fleet.>`, so a `wm.fleet.gossip.append`
  subject crosses to the work node for free once `tether-link` is up — the only
  missing piece is a tail-and-apply daemon at each end.
- The user asked to "share … gossip" by name. Of the four mirrors, gossip is the
  one with an existing, well-defined append-only file contract, so it's the
  cleanest to get right first among the consumers.

## What this builds

A `wm-tether-gossip` binary (single crate, `~/wintermute/tether-gossip`):

- **Tail half.** Watches `notes/gossip.md` for new appends (size/inode-aware,
  survives the file being rotated/recreated), and for each new block publishes a
  `wm.fleet.gossip.append` event: `{node, seq, ts, body}` where `seq` is a
  monotonic per-node counter persisted to `~/.cache/wm-tether-gossip/state`.
- **Apply half.** Subscribes to `wm.fleet.gossip.append`, and for each event from
  a *different* node, appends the body to the local `gossip.md` under a provenance
  header (`## <ts>  <node>  (via tether)`) — append-only, never edits existing
  content. Deduplicates by `(node, seq)` so a redelivered event is applied once.
- **Loop guard.** A block this node appended-then-published must not be
  re-applied when it (or its echo) returns from the bus — guarded by node-origin
  check + the `(node, seq)` dedup set (mirrors nats-bridge's exactly-once-per-
  direction contract).
- **Ordering.** Per-node `seq` is monotonic; applied blocks preserve source
  order per node. Cross-node interleaving is timestamp-ordered best-effort (a
  distributed total order is explicitly NOT attempted — flagged as acceptable
  for an append-only human-readable log).
- `wm-tether-gossip status` reports last-published seq, last-applied seq per
  known node, and pending/dedup counts. SKIPs honestly when no link is configured.

Deps: `clap`, `serde`/`serde_json`, a file-watch (notify or poll), async NATS
client (embedded test server for tests), `sigpipe`. MSRV 1.85, edition 2021.
`sigpipe::reset()` first line of `main()`.

## Acceptance criteria

1. Appending a new block to a fixture `gossip.md` causes `wm-tether-gossip` to
   publish exactly one `wm.fleet.gossip.append` event whose `body` equals the
   appended block and whose `seq` is the previous seq + 1 (asserted against a
   captured publish sink / embedded NATS).
2. An incoming `wm.fleet.gossip.append` from node `"worknode"` is appended to the
   local fixture file under a `(via tether)` provenance header; the file's prior
   content is byte-for-byte unchanged above the new block (append-only invariant).
3. Dedup: the same `(node, seq)` event delivered twice is applied exactly once
   (the local file gains one block, not two).
4. Loop guard: a block this node published is NOT re-applied to its own local
   file when the same event arrives back over the bus (no self-echo duplication).
5. `seq` persists across restarts: after a restart, the next published event's
   `seq` continues from the persisted value, not from zero.
6. (deferred — embedded NATS) A round-trip selftest publishes a tagged block
   locally, confirms it arrives on `wm.fleet.gossip.append` exactly once, and (in
   a two-instance harness) is applied to the second instance's file exactly once.
7. `cargo test` green; `sigpipe::reset()` first in `main()` (grep-asserted);
   `wm-tether-gossip status | head` does not panic.
