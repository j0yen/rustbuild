# PRD: herald-pack — package a skill + its binary into an installable plugin

Status: Draft v0.1
build_target: rust-cli
Vision: visions/herald.md

## TL;DR

On this laptop, a "skill" is a `SKILL.md` directory symlinked into
`~/.claude/skills/` from `~/wintermute/` — usable by the author, **uninstallable
by anyone else**. `herald-pack` is a Rust CLI that turns a skill directory plus
its Rust binary dependency into a self-contained, installable Claude Code plugin:
a `.claude-plugin/plugin.json` manifest, the bundled `SKILL.md` (+ scripts), and
an `install.sh` that builds/fetches the binary and links the skill into a user's
`~/.claude/`.

## Why this exists

- **The distribution gap is real and verified.** Phase-1 research (2026-06-08):
  no repo under `~/wintermute/` ships a `.claude-plugin/plugin.json`; the only
  marketplace on the box is the official one. The existing skill tooling
  (`skill-doctor`, `skill-manifest`, `wm-skill-edit`) validates/edits skills
  that are *already installed* — none produces an installable artifact. Without
  a packager, the seed ("distribute ethical reasoning as a skill") is impossible.
- **The target format is known and stable.** The official marketplace
  (`~/.claude/plugins/marketplaces/claude-plugins-official/.claude-plugin/marketplace.json`,
  222 plugins) shows the exact plugin/source shape: a plugin entry with `name`,
  `description`, `author`, `category`, and a `source` of
  `{source: git-subdir, url, path, ref, sha}`. herald-pack emits artifacts that
  slot into that shape — no protocol invention required.
- **It must be generic.** The flagship is the ethics skill, but the same
  packager should later ship `recall`, `ctrace`, etc. as skills. Building it
  capability-agnostic now avoids a one-off.

## What this builds

A `cargo` Rust CLI `herald-pack` at `~/wintermute/herald-pack/`.

**Deps (pinned at build time):** `serde` + `serde_json` (plugin.json),
`clap`, `toml` (read a small `herald.toml` packaging spec).

**Packaging spec.** A `herald.toml` in the skill's repo declares: skill name,
description, category, author, the binary crate to bundle (path or crates.io),
the binaries the SKILL.md invokes, and the install target. herald-pack reads it
and the `SKILL.md`, then emits a `dist/` plugin tree.

**Emitted plugin tree:**
```
dist/<skill>/
├── .claude-plugin/plugin.json     # name, description, version, author, skills[]
├── skills/<skill>/SKILL.md        # the skill text (+ scripts/ copied)
└── install.sh                     # builds/fetches binary, links skill into ~/.claude
```

**UX.**
```
herald-pack build   --spec herald.toml --out dist/      # emit installable plugin tree
herald-pack check   --spec herald.toml                  # validate spec + referenced binaries exist
herald-pack manifest --out dist/<skill>/                # print the plugin.json it would publish
```

## Acceptance criteria

1. `herald-pack build --spec herald.toml` emits a `dist/<skill>/` tree
   containing a valid `.claude-plugin/plugin.json`, the `SKILL.md` (+ any
   `scripts/`), and an executable `install.sh`.
2. The emitted `plugin.json` validates against the field shape the official
   marketplace uses (`name`, `description`, `version`, `author`, and a `skills`
   list pointing at the bundled SKILL.md) — verified by a schema/shape test.
3. `install.sh`, run in a clean `$HOME` (test uses a temp HOME), places the
   skill so `~/.claude/skills/<skill>/SKILL.md` resolves and the declared binary
   is on `PATH` (built via cargo or copied from a provided artifact dir).
4. `herald-pack check` fails with a non-zero exit and a clear message if the
   spec references a binary crate that does not exist or a SKILL.md that invokes
   a binary not declared in the spec (reuses the spirit of skill-doctor's
   text↔tooling cross-check).
5. `build` is deterministic: same spec + same skill dir → byte-identical
   `plugin.json` (no embedded timestamps; version comes from the spec).
6. The packager is capability-agnostic: a test packages a trivial fixture skill
   ("echo-skill" wrapping a one-line binary) end-to-end, proving no
   ethics/ousia-specific assumptions are baked in.
7. `herald-pack manifest` prints the marketplace-ready plugin entry (the object
   herald-market will aggregate), including a `source` stub to be filled with
   url/ref/sha at publish time.
8. `cargo test` covers: build emits the tree; plugin.json shape is valid; check
   rejects a missing-binary spec; install.sh links into a temp HOME; the
   fixture-skill round-trip.
