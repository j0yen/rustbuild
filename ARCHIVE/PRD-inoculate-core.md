# PRD: inoculate-core — the strain

Status: Draft v0.1
build_target: rust-cli
Vision: visions/inoculate.md

## TL;DR

There is no single, machine-readable artifact that *is* "this box's ethics."
The commitments live as prose across `~/.claude/CLAUDE_SELF.md` (Values +
Boundaries sections) and as rules in `answerable`'s `redline.toml`. To transmit
ethics to other agents, prove an agent is carrying them, or attest which version
was in force, we first need to **distill those sources into one versioned,
hashable "strain."** `inoculate-core` is that distiller and its CLI.

## Why this exists

Verified live 2026-06-15:
- `~/.claude/CLAUDE_SELF.md` has structured `## Values` and `## Boundaries`
  sections (read this session) — the canonical commitments, but as prose.
- `answerable check --action … --attr …` enforces `~/.config/answerable/redline.toml`
  (`answerable check --help`: exit 0=allow/1=flag/2=redline) — the machine rules,
  but a *separate* file from CLAUDE_SELF.md.
- `answerable values-drift` already parses the Values/Boundaries sections, so the
  parsing shape is known and reusable.

Nothing composes these into one object you can version, hash, and hand to another
agent. Every other PRD in this vision needs that object.

## What this builds

A new repo `inoculate` (`~/wintermute/inoculate`), one binary `inoculate`:

- `inoculate strain [--format text|json] [--self <path>] [--redline <path>]` —
  read CLAUDE_SELF.md Values + Boundaries and redline.toml, emit the distilled
  strain. Defaults: `~/.claude/CLAUDE_SELF.md`, `~/.config/answerable/redline.toml`.
  JSON shape: `{version, source_self_sha, source_redline_sha, values:[...],
  boundaries:[...], redlines:[...], strain_hash}`.
- `inoculate hash [--self …] [--redline …]` — print only the `strain_hash`
  (blake3 of the canonicalized strain content). Stable across reorderings of the
  source files (canonicalize: trim, sort within section) so cosmetic edits don't
  churn the hash; a semantic change (added/removed/reworded line) does change it.
- `inoculate version` — a human-meaningful version string derived from
  CLAUDE_SELF.md's changelog date + a short hash suffix (e.g. `2026-06-15+af3e09`).

Library crate `inoculate-core` exposes `Strain::distill(self_path, redline_path)
-> Result<Strain>`, `Strain::hash() -> String`, `Strain::to_json()` so the
sibling PRDs (inject, carrier-check, spread) link it rather than shelling out.

Deps: `blake3`, `serde`/`serde_json`, `clap` (derive), `anyhow`, `sigpipe`
(reset first line of main per the SIGPIPE-panic lesson).

## Acceptance criteria

1. `cargo test --release` passes; `inoculate` installs to `~/.local/bin/`.
2. `inoculate strain --format json` against a fixture CLAUDE_SELF.md emits valid
   JSON containing non-empty `values`, `boundaries`, and a 64-hex `strain_hash`.
3. `inoculate hash` output equals the `strain_hash` field from `strain --format json`
   for the same inputs.
4. Canonicalization: reordering two Boundary lines in the fixture does NOT change
   `inoculate hash`; rewording one Boundary line DOES change it. (Two fixtures,
   asserted in tests.)
5. Missing/unreadable source file → clear error on stderr, exit non-zero (not a
   panic); `unsafe_code = "deny"` and no `unwrap`/`expect` in non-test code.
6. `inoculate version` emits a non-empty `<date>+<6hex>` string.
7. `inoculate strain | head` does not panic (SIGPIPE reset verified).

## Out of scope

Injection, transmission, attestation, persona floor — all sibling PRDs. This PRD
only produces and hashes the strain.
