# PRD: tend-classify

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/tend
Vision: visions/tend.md

## TL;DR

`tend survey` reports every dirty path equally, but most of the dirt is the
**same artifact noise every run** — `.build-worktrees/`, `.cache/`,
`__pycache__/`, `skill/state/`, `.run-*/` — while only a few paths are real
source WIP a human should look at. `tend survey --classify` labels each
`DirtPath` as `Artifact | Source | Generated`, using a curated noise-glob set as
the primary signal and the provfs `user.prov.session` / `user.prov.ts`
provenance xattrs as a secondary one, so the survey separates "silenceable
churn" from "work that matters."

## Why this exists

Captured live during the dream that drafted this PRD (2026-06-08):

- autobuilder's untracked set this pass was `.build-worktrees/`, `.cache/`,
  `.run-ambient/`, `skill/state/`, `skill/scripts/__pycache__/`,
  `skill/tests/__pycache__/` — six paths, **all** runtime/build artifacts, plus
  two genuine WIP files (`notes/…playbook.staged.md`,
  `proposals/…backfill.draft.sh`). A flat dirty count of 8 hides that only 2
  matter. The same artifact prefixes recur in every self-review.
- **provfs is live and attributes writers.** `getfattr -d` on the untracked
  `notes/…playbook.staged.md` this pass returned
  `user.prov.session="comm:Bun Pool 2:pid:39673:uid:1000"` plus
  `user.prov.ts="1780084443"`. The kernel stamps who-wrote-and-when on every
  closed-after-write file ([[project_agent_tooling]], Phase 1.5 of the dream
  skill). That lets classify attribute an untracked file to a timer session
  (build/dream/self-review → likely artifact) vs an interactive human session
  (→ likely real WIP) instead of guessing from the path alone.
- The session id is the `comm:pid:uid` fallback today — agentns is all-zeros
  ([[self_agentns_einval_flag_collision]]) — so provenance is a **weak,
  additive** signal until [[PRD-agentns-clone-flag-fix]] lands. classify must
  keep the glob set primary and treat provfs as a refinement.

## What this builds

`tend-classify` rust-extends `~/wintermute/tend` (created by
[[PRD-tend-survey]]). Do not start until `~/wintermute/tend` exists and builds.
Conventions inherited: rustc 1.85, no let-chains, cloud-build-safe,
`sigpipe::reset()` already in `main()`.

### Modules

- `classify.rs` — the classifier:
  - `DirtClass` = `Artifact | Source | Generated | Unknown`.
  - A curated, documented **noise-glob set**: `target/`, `node_modules/`,
    `__pycache__/`, `.cache/`, `.build-worktrees/`, `.run-*/`, `.venv/`,
    `*.pyc`, `dist/`, `*.staged.md` and `*.draft.*` → these last two are
    `Source`-class WIP, not artifact (drafts are real work). The set lives in
    one table with a comment justifying each entry from the live survey.
  - `classify_path(repo, dirt_path) -> (DirtClass, ProvHint)` — glob match
    first; `Generated` for known-generated files (`Cargo.lock` when it's the
    only change, lockfiles); `Source` for anything under tracked source dirs
    not matching noise; `Unknown` otherwise.
- `prov.rs` — read provfs xattrs via `getfattr`-equivalent (`std::fs` + the
  `xattr` crate, or shell `getfattr -d --only-values`): return
  `ProvHint { session: Option<String>, ts: Option<u64>, writer_kind:
  WriterKind }` where `WriterKind` = `Timer | Interactive | Unknown`, derived by
  matching the `comm`/session against known timer comms (`claude-build`,
  `claude-dream`, `claude-self`, `Bun Pool`) — best-effort, never fatal if the
  xattr is absent (non-provfs filesystem, `/tmp`, etc.).
  - When `writer_kind == Timer` and glob says `Unknown`, nudge toward
    `Artifact`; when `Interactive`, never downgrade a `Source` to `Artifact`.
    The glob result is authoritative for known patterns; prov only breaks ties.
- extend `model.rs`: add `class: Option<DirtClass>` and `prov:
  Option<ProvHint>` to `DirtPath` (None when `--classify` not requested, so the
  bare survey schema is unchanged).
- extend `render.rs`: under `--classify`, group each repo's paths by class and
  print an artifact/source/generated count; `--json` includes the new fields.
- extend `main.rs`: `tend survey --classify`.

### UX

```
$ tend survey --classify
autobuilder   main   dirty 8  (artifact 6 · source 2)
  source:   notes/…playbook.staged.md   proposals/…backfill.draft.sh
  artifact: .build-worktrees/ .cache/ .run-ambient/ skill/state/ …
$ tend survey --classify --json | jq '.repos[].dirty[] | select(.class=="Source") | .path'
```

### Dependencies

Adds `xattr` (or shells `getfattr`). No network. Existing `clap`/`serde`/
`sigpipe` reused.

## Acceptance criteria

1. `classify_path` is unit-tested: each noise-glob entry classifies as
   `Artifact`; `*.staged.md`/`*.draft.sh` classify as `Source`; a `src/foo.rs`
   classifies as `Source`; a lone `Cargo.lock` classifies as `Generated`. Table
   coverage is asserted (every glob in the set has a test case).
2. `tend survey --classify` on a fixture repo with mixed dirt prints correct
   per-class counts and lists `Source` paths separately from `Artifact`.
3. provenance reading is **non-fatal**: a path with no provfs xattr (or on a
   filesystem without the LSM) yields `ProvHint { session: None, writer_kind:
   Unknown }` and never errors; tested by classifying a path under `/tmp`.
4. `WriterKind` derivation is unit-tested against fixture xattr strings:
   `comm:claude-build:…` → `Timer`; `comm:Bun Pool 2:…` → `Timer`; an
   interactive-shell comm → `Interactive`; malformed → `Unknown`.
5. The glob result is authoritative for known patterns: a `target/` path with
   an `Interactive` writer still classifies `Artifact`; a `src/x.rs` with a
   `Timer` writer still classifies `Source` (prov only breaks `Unknown` ties).
   Both asserted.
6. The bare `tend survey` (no `--classify`) output and `--json` schema are
   **unchanged** — `class`/`prov` are `None`/omitted; a serde test confirms the
   v0.1 survey schema still round-trips.
7. No mutating git or filesystem write is introduced (classify is read-only);
   the read-only command-set guard from [[PRD-tend-survey]] AC7 still passes.
8. `README.md` gains a "classification" section documenting the noise-glob set
   (with the justification for each entry), the provfs provenance refinement,
   and that provenance is a weak additive signal until agentns session ids land.
