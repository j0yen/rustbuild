# Vision: tend — the fleet's working trees should never be the human's problem

**Author:** /dream (Claude Sonnet 4.6), for jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` (interactive, no steer). The inward/outward arcs are
saturated (~48 visions); the strongest *uncovered* recurring signal is the
git-hygiene list every self-review hand-walks and dumps under "Pending your
call" — dirty working trees and unpushed commits across `~/wintermute/*`, run
after run, with no tool behind it.

## TL;DR

`~/wintermute/` is a fleet of ~30 git repos that `/build` and `/dream` churn
constantly. Every self-review walks them by hand, counts dirty paths, lists
unpushed commits, and hands the user a list it explicitly will not act on
("do not auto-stash"; "review before pushing"). The list is dominated by the
**same artifact noise every run** — `.build-worktrees/`, `.cache/`,
`__pycache__/`, `skill/state/`, `.run-*/` — that should have been `.gitignore`d
once and silenced forever, plus a tail of genuinely unpushed committed work
that quietly never lands on the remote. **tend** turns that recurring hand-walk
into a structured, attributed, proposal-first tool: survey the fleet, classify
every dirty path (artifact vs real source WIP, using provfs provenance xattrs
to attribute who wrote it), propose `.gitignore` patches that kill the recurring
noise, propose pushes for clean-and-ahead repos, and emit a self-review digest
so the "Pending your call" git section shrinks to only what a human truly must
decide. tend never stashes, never commits source, never force-pushes — it
reads, classifies, and proposes.

## Why this is real (Phase 1 evidence)

Captured live during the dream that drafted this vision (2026-06-08):

- **Every self-review reports it.** 2026-06-06, -07, -08 reflections each list,
  under "Pending your call," a dirty-tree roll-call (`autobuilder`,
  `build-skill`, `keel`, `concord`, `coda`, …) plus an unpushed-commit list
  (`build-skill 1`, `rollout 1`, `wintermute-desktop 2`). It is the most
  durable recurring "your call" item after the pacman queue (now owned by
  [[tide]]).
- **Live survey, this pass:** 13 repos under `~/wintermute/` are dirty or
  ahead: `autobuilder(8)`, `build-skill(16, +1 unpushed)`, `concord(2)`,
  `coda(1)`, `dream-skill(1)`, `wintermute-almanac(1)`, `wintermute-reach(1)`,
  `wintermute-desktop(1, +2 unpushed)`, `rollout(+1 unpushed)`,
  `cradle-…-bak(3)`, plus others.
- **The dirt is mostly silenceable noise.** autobuilder's untracked set this
  pass: `.build-worktrees/`, `.cache/`, `.run-ambient/`, `skill/state/`,
  `skill/scripts/__pycache__/`, `skill/tests/__pycache__/` — runtime/build
  artifacts, not source. The same prefixes recur every review; a one-time
  per-repo `.gitignore` patch removes them permanently.
- **provfs is live and attributes writers.** `getfattr -d` on the untracked
  `notes/…playbook.staged.md` returns `user.prov.session="comm:Bun Pool
  2:pid:39673:uid:1000"` + `user.prov.ts` — the kernel already stamps who-wrote
  and when on every closed-after-write file ([[project_agent_tooling]],
  Phase 1.5). tend can use this to attribute an untracked file to a timer
  session (build/dream/self-review → likely artifact) vs an interactive human
  session (→ likely real WIP), instead of guessing from the path alone.
- **No existing owner.** `loom` is the only vision touching git, and it is
  strictly /build-internal worktree integration (parallel same-target rebase),
  not fleet-wide working-tree hygiene. Nothing surveys, classifies, or proposes
  cleanup across the repos.

## End-state

When this is done:

- `tend survey` prints (and `--json` emits) the whole fleet's git state in one
  read-only pass: per repo, dirty count, untracked/modified paths, and
  ahead/behind vs upstream.
- `tend survey --classify` labels every dirty path `Artifact | Source |
  Generated`, using path globs as the primary signal and provfs
  `user.prov.session` provenance as a secondary one, so the human-relevant WIP
  is separated from recurring noise.
- `tend gitignore --plan` prints a per-repo `.gitignore` patch that would
  silence the artifact-class noise (dedup against existing entries), proposal
  only — it prints the diff, it does not write unless explicitly asked.
- `tend push --plan` lists repos that are clean-tree **and** ahead of upstream
  (committed work that never shipped) and prints the exact `git push` command
  for each, marking ff-safe vs diverged. It never pushes.
- self-review splices `tend report --format selfreview` into its "Pending your
  call" section, so the git hand-walk is replaced by a tool whose output is
  already classified down to only the items a human must decide.

## Components (one bullet per future PRD)

- **tend-survey** (`rust-cli`, NEW at `~/wintermute/tend`) — the foundation:
  walk the fleet, parse `git status --porcelain` + ahead/behind, core types
  (`RepoState`, `DirtPath`, `FleetReport`), human + `--json`. Read-only.
  **Ships first**; the rest rust-extend into this repo.
- **tend-classify** (`rust-extend`) — classify each `DirtPath` as
  `Artifact | Source | Generated` via a curated noise-glob set + provfs
  `user.prov.session`/`user.prov.ts` provenance as a secondary signal.
  Adds `survey --classify`.
- **tend-gitignore** (`rust-extend`) — propose per-repo `.gitignore` patches
  for the recurring artifact noise, deduped against the existing file.
  Proposal-first (`--plan` prints the diff).
- **tend-push** (`rust-extend`) — propose push plans for clean-and-ahead repos,
  ff-safe vs diverged. Proposal-first; never pushes.
- **tend-report** (`rust-extend`) — a self-review-shaped digest
  (`report --format selfreview`) that collapses the classified survey into the
  short list a human must actually decide, for self-review to splice in.

## Order

```
survey ──► classify ──► { gitignore, push } ──► report
```

- `tend-survey` must ship first (creates the repo + binary + core types).
- `tend-classify` extends survey (the classifier needs `DirtPath`).
- `tend-gitignore` ⟂ `tend-push` — both consume the classified survey, neither
  depends on the other.
- `tend-report` consumes all of the above and is the self-review bridge.

## Open questions (discuss with user)

- **Scope of the walk:** just `~/wintermute/*`, or also the skill repos under
  `~/.claude/skills/*` and `~/.local/bin/` source trees? Leaning configurable
  roots (default `~/wintermute`), like anchor's watched-roots.
- **Should `tend gitignore --write` ever exist?** Writing a `.gitignore` is a
  tracked-file edit (reversible, low blast radius) — defensible as an opt-in
  `--write` behind confirmation, but the proposal-first default stays. Decide
  before tend-gitignore ships.
- **Push autonomy:** push touches shared remote state and stays human-gated by
  default ([executing-actions-with-care]). Is a `--confirm`-gated ff-only push
  ever wanted for the obviously-safe clean-ahead case (rollout, desktop), or
  always proposal-only? Leaning always-proposal, matching tide-window /
  muster-reap / recourse-contest.
- **provfs reliability:** the session id is the `comm:pid:uid` fallback today
  (agentns all-zeros, [[self_agentns_einval_flag_collision]]); provenance is a
  weak signal until [[PRD-agentns-clone-flag-fix]] lands. classify must treat
  the glob set as primary and provfs as additive, becoming stronger later.
