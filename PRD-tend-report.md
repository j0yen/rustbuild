# PRD: tend-report

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/tend
Vision: visions/tend.md

## TL;DR

self-review currently hand-walks the fleet's git state and dumps a raw dirty/
unpushed roll-call under "Pending your call" every run. `tend report --format
selfreview` collapses the classified survey + gitignore + push proposals into
the short, already-triaged digest a human must actually decide on — so
self-review can splice tend's output in instead of re-deriving the list by hand.

## Why this exists

Captured live during the dream that drafted this PRD (2026-06-08):

- The 2026-06-08 self-review's "Pending your call" section spends three bullets
  on git state: a dirty-tree roll-call, an unpushed-commit list, and a "do not
  auto-stash" caveat — all hand-derived, all repeated nearly verbatim from the
  2026-06-07 and -06-06 reflections.
- Once [[PRD-tend-classify]], [[PRD-tend-gitignore]], and [[PRD-tend-push]]
  exist, that hand-walk is redundant: the artifact noise is silenceable
  (gitignore plan), the unpushed work is a push plan, and only `Source`-class
  WIP and `Diverged` repos genuinely need a human. A digest that surfaces *only*
  those is the right input for self-review.
- This is the self-review-bridge pattern already used elsewhere — `muster
  verdict --format selfreview` and the `muster-selfreview-bridge` PRD emit a
  self-review-shaped block for splicing. tend-report mirrors it.

## What this builds

`tend-report` rust-extends `~/wintermute/tend` (created by [[PRD-tend-survey]],
consuming [[PRD-tend-classify]], [[PRD-tend-gitignore]], [[PRD-tend-push]]). Do
not start until those exist and build. Conventions inherited: rustc 1.85, no
let-chains, cloud-build-safe, `sigpipe::reset()` in `main()`.

### Modules

- `report.rs`:
  - `Digest { source_wip: Vec<(repo, path)>, gitignore_repos: usize,
    gitignore_patterns: usize, ff_pushes: Vec<PushPlan>, diverged:
    Vec<PushPlan>, fully_clean: usize }` — built by running the classified
    survey once and folding in the gitignore + push proposals.
  - The digest deliberately **drops** pure-artifact dirt from the human view
    (it's covered by the gitignore plan, summarized as a count) and surfaces
    only `Source` WIP, ff-safe pushes, and diverged repos.
- `render.rs` extension — a `--format selfreview` renderer that emits a markdown
  block matching self-review's "Pending your call" idiom:

  ```
  ### Fleet git (tend)
  - Source WIP awaiting your review (2): autobuilder/notes/…staged.md, …
  - Unpushed, ff-safe (2): rollout, wintermute-desktop — `tend push` for cmds
  - Diverged, needs rebase (0)
  - Artifact noise: 9 patterns across 3 repos — `tend gitignore --write` to silence
  - Fully clean: 19 repos
  ```

  Plus existing `--json` (the `Digest`) and a default human form.
- extend `main.rs`: `tend report [--root <p>]... [--format human|json|
  selfreview]`.

### UX

```
$ tend report --format selfreview     # paste-ready for the self-review journal
$ tend report --json | jq '.source_wip | length'
2
```

### Dependencies

No new crates (composes survey/classify/gitignore/push). No network.

## Acceptance criteria

1. `Digest` construction is unit-tested against a fixture classified
   `FleetReport`: `source_wip` contains only `Source`-class paths; pure-artifact
   repos contribute to `gitignore_*` counts but **not** to `source_wip`; a clean
   repo ahead>0 lands in `ff_pushes`; a clean repo with no dirt and ahead 0
   counts in `fully_clean`.
2. `--format selfreview` emits a markdown block whose headings match the idiom
   shown (Source WIP / Unpushed ff-safe / Diverged / Artifact noise / Fully
   clean) with correct counts, validated against a fixture digest by string
   assertion.
3. The selfreview block is **paste-safe**: it contains no ANSI color codes and
   no trailing whitespace (asserted), so it can be written verbatim into a
   journal file.
4. `tend report --json` emits the stable `Digest` schema validated by a serde
   round-trip test.
5. The report runs the survey/classify pipeline **once** (no redundant fleet
   walk per sub-proposal); a test confirms the digest is built from a single
   `FleetReport` input rather than re-walking.
6. Empty fleet (no dirty/ahead repos) renders a clean digest
   (`fully_clean == N`, all other lists empty) and a one-line "fleet clean"
   selfreview block, not an error.
7. The read-only-git guard from [[PRD-tend-survey]] AC7 still passes (report
   composes only read-only readers and the proposal builders).
8. `README.md` documents the three output formats, the self-review splice
   workflow, and that the digest intentionally hides artifact noise behind a
   count.
