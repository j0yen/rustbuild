# PRD: inoculate-attest — every autonomous action names its strain

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/answerable
Vision: visions/inoculate.md

## TL;DR

`answerable`'s ledger records *what* the agent did autonomously, but not *which
ethic was in force* when it did. After a strain changes (a Boundary added, a
redline tightened), there's no way to audit "was this push taken by an agent
carrying the new strain, or the old one?" `inoculate-attest` stamps the in-force
`strain_hash` onto every ledger entry — and, where provfs is active, onto the
written file's provenance xattr — so accountability and inoculation are joined.

## Why this exists

Verified live: `answerable record` appends to `$XDG_STATE_HOME/answerable/ledger.jsonl`
(`answerable --help`), and `answerable values-drift` already watches CLAUDE_SELF.md
— so the ledger knows *about* values but never tags an action with the values
*version* it ran under. Separately, the provfs LSM stamps `user.prov.session` on
closed-after-write files (vision Phase-1.5 kernel survey). Joining the strain
hash to both gives a complete answer to "inoculated actor, which version?"

## What this builds

Extends `answerable` (the existing repo/binary), depending on `inoculate-core`'s
`inoculate hash`:

- `answerable record` gains an optional `--strain <hash>` attribute; when omitted,
  it auto-resolves the current strain by calling `inoculate hash` (graceful: if
  `inoculate` is absent, record `strain=unknown` rather than failing the record).
  The hash is stored as a first-class field on the JSONL entry.
- `answerable log` / `answerable digest` surface the strain: log shows a short
  `strain=<6hex>` tag; digest can say "all of today's actions ran under strain
  <version>" or flag "3 actions ran under a since-superseded strain."
- New `answerable strain-audit [--since <dur>]` — group ledger entries by strain
  hash, show counts, and flag any entries whose strain differs from the *current*
  one (actions taken under a now-stale ethic).
- Optional provfs bridge: `answerable record --provfs-stamp <file>` reads the
  existing `user.prov.*` xattrs and writes a companion `user.inoculate.strain`
  xattr (best-effort; no-op with a logged note if xattrs unsupported on the fs).

## Acceptance criteria

1. `cargo test --release` passes; `answerable` reinstalls to `~/.local/bin/`.
2. `answerable record --action publish --attr visibility=private --strain deadbeef…`
   produces a ledger entry whose JSON contains the strain hash.
3. With `--strain` omitted and `inoculate` on PATH, the entry's strain equals
   `inoculate hash`; with `inoculate` absent, the entry records `strain=unknown`
   and the record still succeeds (exit 0).
4. `answerable strain-audit` groups entries by strain and flags any entry whose
   strain ≠ the current `inoculate hash`, asserted against a fixture ledger.
5. `answerable log` shows a `strain=` tag; existing log output for pre-existing
   (un-stamped) entries still renders (back-compat: missing field → `strain=—`).
6. provfs bridge writes `user.inoculate.strain` on a tmpfs/ext4 test file when
   xattrs are supported, and no-ops cleanly when not.
7. No regression in existing `answerable` tests; CHANGELOG + version bump.

## Out of scope

Producing the strain (inoculate-core) and proving an agent carries it
(inoculate-carrier-check). This PRD only *records* the strain on actions.
