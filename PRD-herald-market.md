# PRD: herald-market — a j0yen marketplace for installable skills

Status: Draft v0.1
build_target: rust-cli
Vision: visions/herald.md

## TL;DR

A packaged plugin (from `herald-pack`) is only distributable if a user can
*discover and add* it. `herald-market` generates and maintains a `j0yen`
`marketplace.json` — the catalog manifest a user adds with
`claude plugin marketplace add j0yen/wintermute-skills` — aggregating packaged
plugins as pinned `git-subdir` sources, exactly like the official 222-plugin
marketplace.

## Why this exists

- **The catalog is the distribution channel.** Phase-1 research (2026-06-08):
  the only marketplace on the box is `claude-plugins-official`; there is no
  `j0yen` marketplace. herald-pack produces installable trees, but nothing tells
  a remote user they exist or where to fetch them. herald-market is that index.
- **The format is proven.** The official
  `.claude-plugin/marketplace.json` shows the exact aggregate shape: top-level
  `name` + `owner`, and a `plugins[]` array where each entry carries a `source`
  of `{source: git-subdir, url, path, ref, sha}`. herald-market emits the same
  structure for `j0yen` plugins — pinning `ref` + `sha` so installs are
  reproducible.
- **Hand-maintaining JSON drifts.** A growing catalog edited by hand will rot
  (stale shas, mismatched versions) — the same class of drift `skill-manifest`
  exists to catch in SKILL.md. A generator keyed off herald-pack manifests keeps
  the catalog honest.

## What this builds

A `cargo` Rust CLI `herald-market` at `~/wintermute/herald-market/`, plus the
seed `j0yen/wintermute-skills` marketplace repo layout it emits into.

**Deps (pinned at build time):** `serde`/`serde_json`, `clap`, a git
invocation layer (shell out to `git` for ref/sha resolution).

**UX.**
```
herald-market init   --owner j0yen --name wintermute-skills        # scaffold marketplace repo
herald-market add    --plugin dist/conscience/ --repo <url>        # add/update a plugin entry
herald-market sync                                                 # refresh ref+sha for every entry
herald-market lint                                                 # validate the marketplace.json
```

`add` ingests the `plugin.json` from a herald-pack `dist/<skill>/` tree and the
plugin's published git URL, resolves the current `ref`/`sha`, and writes/updates
the matching `plugins[]` entry in `.claude-plugin/marketplace.json`. `sync`
re-resolves shas for all entries (catches a plugin repo that moved forward).

## Acceptance criteria

1. `herald-market init` scaffolds a marketplace repo with a valid
   `.claude-plugin/marketplace.json` carrying `name`, `owner`, and an empty
   `plugins[]`.
2. `herald-market add --plugin dist/<skill>/ --repo <url>` appends (or updates,
   by name) a `plugins[]` entry whose `source` is
   `{source: git-subdir, url, path, ref, sha}` with `ref`/`sha` resolved from
   the repo's current HEAD — matching the official marketplace's entry shape.
3. Adding a plugin whose `name` already exists updates that entry in place
   rather than duplicating it (idempotent catalog).
4. `herald-market lint` validates the manifest: every entry has the required
   fields, no duplicate names, every `source.path` is non-empty — non-zero exit
   on any violation, naming the offending entry.
5. `herald-market sync` re-resolves `ref`/`sha` for all entries and reports
   which entries advanced (old sha → new sha).
6. The emitted `marketplace.json` is accepted by the Claude Code marketplace-add
   format (verified structurally against the official manifest's schema in a
   test — no live `claude plugin` call required).
7. Generation is deterministic given fixed input shas: re-running `add`/`sync`
   with no upstream change produces byte-identical output.
8. `cargo test` covers: init scaffolds a valid manifest; add inserts a
   conformant entry from a fixture dist tree; add is idempotent on duplicate
   name; lint rejects a manifest with a duplicate name and one with a missing
   field.
