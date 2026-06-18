# PRD: consign-policy — the push-eligibility gate

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/consign
Vision: visions/consign.md

## TL;DR

Before consign autonomously pushes any repo, something must decide *whether it
is allowed to*. Not every unpushed repo should be auto-published: autobuilder-
private must stay private, a diverged repo must not be force-resolved, and a
half-baked `/build` worktree branch must not be published as if it were release
work. consign-policy is that gate — a default-deny classifier that turns each
`RepoDebt` from consign-survey into an `auto-ok | private-hold | manual-only`
verdict. No consign write path (publish, drain) may run without it.

## Why this exists

- Memory `project_autobuilder_repo_private` (2026-06-05): the autobuilder
  PRDs/visions repo is **private** (`j0yen/autobuilder-private`), overriding
  "drafts public-by-design." An autonomous pusher that blindly pushes every
  repo would be fine for shipped crates but must never expose a private repo —
  so eligibility cannot be "push everything."
- Live evidence: `homeward`'s 2 unpushed commits are on branch
  `autobuilder/homeward-found-geocode`, a `/build` worktree branch — NOT
  `main`. Pushing it as-is would publish in-flight worktree state. consign-survey
  already records `branch_is_default`; policy must act on it.
- `mqo-narrative-compose` is diverged (ahead 2 / behind 2). Auto-pushing a
  diverged repo is wrong (needs rebase/merge first) — policy must route it to
  `manual-only`, never `auto-ok`.
- The standing guardrail idiom on this box is default-deny: `plumb` quarantines
  uncalibrated probes; `headway-verify` refuses to false-close. Policy follows
  suit — ambiguous ⇒ `manual-only`, never `auto-ok`.

## What this builds

Extend `~/wintermute/consign` with a `policy` module + subcommand.

- `consign policy [--root <dir>]... [--format json|table]` — runs survey
  internally, then attaches a verdict to each repo.
- Verdict `PolicyClass`:
  - `auto-ok` — `ahead` or `no-upstream` or `no-remote`, on the default branch,
    no hold marker, no detected secret, and (for no-remote) a derivable
    `j0yen/<name>` target. The only class publish/drain may act on.
  - `private-hold` — repo path/remote matches a private allowlist (default:
    anything named `*-private`, plus `autobuilder*`), OR a `.consign-hold` file
    exists at repo root, OR a secret heuristic fires (a tracked `.env`,
    `*.pem`, `id_*`, or `*credential*` file). Surface, never auto-push.
  - `manual-only` — `diverged`, OR `branch_is_default == false` (worktree /
    feature branch), OR detached HEAD. Needs a human.
- A small embedded default policy + optional `~/.config/consign/policy.toml`
  override (allow/deny globs, extra hold markers). Absent config ⇒ built-in
  defaults; never a hard error for missing config.
- Library-exported `classify(repo: &RepoDebt, cfg: &PolicyCfg) -> PolicyClass`
  so publish/drain call it directly rather than re-parsing CLI output.

## Acceptance criteria

1. `consign policy --format json` returns survey output augmented with a
   `policy` field (`auto-ok | private-hold | manual-only`) per repo.
2. A repo named/remoted `*-private` or `autobuilder*`, or containing a
   `.consign-hold` file, classifies `private-hold` even if it is `ahead`.
   (Test via fixture repos.)
3. A `diverged` repo and a repo whose HEAD is a non-default branch both
   classify `manual-only`, never `auto-ok`. (Fixtures modelling
   `mqo-narrative-compose` and `homeward`'s worktree branch.)
4. A plain `ahead`/`no-upstream`/`no-remote` repo on its default branch with no
   hold/secret signal classifies `auto-ok`.
5. A repo with a tracked secret-shaped file (`.env`, `*.pem`, `id_rsa`,
   `*credential*`) classifies `private-hold` regardless of other signals.
6. Ambiguous/unknown states default to `manual-only` (default-deny); the
   classifier never returns `auto-ok` for a state it did not explicitly affirm.
7. `classify()` is unit-tested for each branch; `cargo test` green; build via
   /cloudbuild. `consign policy --help` documents the three classes and the
   config override path.
