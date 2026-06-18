# Vision: consign — no fleet work lives only on this laptop

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-18
**Status:** active
**Seed:** bare `/dream` (auto-mode, no topic), field FRESH (streak=0,
last-productive 2026-06-18T07:54Z). Phase-1 live inspection of every
`~/wintermute/*` git repo found a durability gap that has carried across a
week of self-reviews under the line "8 unpushed repos" — and that the
self-review count is itself a **systematic undercount**.

## TL;DR

The fleet's work is committed but stranded. A live walk of
`~/wintermute/*/.git` on 2026-06-18 found:

- **7** repos *ahead of upstream* (commits made, never pushed) — roughly the
  "8 unpushed" self-review reports each day.
- **5** repos with unpushed commits **and no upstream tracking branch** —
  `constellation-burst-builder`, `doxa`, `homeward` (on a `/build` worktree
  branch `autobuilder/homeward-found-geocode`), `mqo-chart-caption` (4 commits),
  `wm-skills`. These are **invisible** to the self-review counter, which only
  measures `@{u}..HEAD` ahead-ness and silently skips any branch with no
  upstream set. The real number is bigger than the reported one.
- **2** repos with **no git remote at all** — `colophon` and `headway`, both
  freshly built this morning. /build's publish step never completed for them,
  so they exist *only* on this disk.
- **1** repo *diverged* — `mqo-narrative-compose` (ahead 2 / behind 2),
  which needs human triage, not an autonomous push.

A broader walk (`git log --branches --not --remotes`) counts **29** repos
holding at least one commit that is on no remote. Self-review says 8.

This matters now because two visions on the board — `constellation` (grow
wintermute onto a multi-machine fleet) and `homeward` (the first
outward-facing product) — both assume the work *is somewhere other than this
one laptop's SSD*. Today it largely isn't. If this disk dies, 29 repos'
worth of un-mirrored commits die with it. The `disk-critical` emergency of
2026-06-17 (99% full, 8.8 G free) was a near-miss reminder that this machine
is a single point of failure.

There is a mature *detection-and-reconcile* idiom on this laptop to copy:
`adopt` scans for shipped-but-not-installed artifacts and drains them on a
6h `adopt-cron.timer`; `binstale`/`headway` does the same for stale daemons.
**Nothing does it for un-mirrored git commits.** consign is that tool: an
honest push-debt survey, a guardrailed eligibility policy, a publisher for
never-published repos, an autonomous drainer on a timer, and a convergence
check that refuses to false-close.

## End-state

Running `consign survey` enumerates every fleet repo's push-debt accurately —
including the no-upstream and no-remote cases the current self-review check
misses — and classifies each as `clean | ahead | no-upstream | no-remote |
diverged`. A guardrailed `consign drain`, run every 6h by a systemd-user
timer, pushes every *policy-approved* repo to its remote (minting a GitHub
repo under `j0yen` first for the never-published ones), sets missing upstreams,
never force-pushes, and surfaces `diverged`/ineligible repos for human triage.
A post-drain `consign verify` proves push-debt fell to the policy floor and
emits a *contradicted* verdict (non-zero) for any repo that claimed pushed but
is still ahead. The "N unpushed" self-review line stops being an undercount and
trends to zero.

## Why this is honest, not invented

Every number above came from a live walk on 2026-06-18 (see each PRD's "Why"
for the exact `git` invocation). `gh auth status` confirms an active
`j0yen` login in the keyring, so `consign publish` can actually mint remotes.
`adopt-cron.{service,timer}` already exists at
`~/.config/systemd/user/` as the timer precedent to mirror.

## Components (one bullet per future PRD)

- **consign-survey** — NEW repo `~/wintermute/consign/` (rust-cli + lib). The
  accurate push-debt enumerator: walk a configured set of roots (default
  `~/wintermute`), classify each repo `clean | ahead(n) | no-upstream(n) |
  no-remote | diverged(a/b)`, emit structured JSON + a human table. Explicitly
  counts the no-upstream and no-remote cases the self-review `@{u}` check drops.
- **consign-policy** — rust-extend into `~/wintermute/consign`. The
  push-eligibility gate that must pass before any autonomous push: classify
  each repo `auto-ok | private-hold | manual-only`. `private-hold` covers
  autobuilder-private (see memory `project_autobuilder_repo_private`) and any
  repo carrying a `.consign-hold` marker or detectable secret; `manual-only`
  covers `diverged` and detached/feature-branch states. Default-deny anything
  ambiguous. No push primitive may run without a policy verdict.
- **consign-publish** — rust-extend. For `no-remote` repos: `gh repo create
  j0yen/<name> --source . --private=<policy>`, set `origin`, push current
  branch, set upstream. Honest abort + structured error on gh-unauth or name
  collision — never a half-created remote. Cites `colophon`, `headway`.
- **consign-drain** — rust-extend. The reconcile loop: for every `auto-ok`
  `ahead`/`no-upstream` repo, push (setting upstream when absent), serialized,
  one receipt per repo. Never `--force`. Skip + surface `manual-only` and
  `diverged`. This is the autonomous drainer `adopt apply` is for artifacts.
- **consign-cron** — shell/config. A systemd-user `consign-drain.{service,timer}`
  mirroring `adopt-cron`'s 6h cadence; runs `consign drain` within guardrails,
  appends a one-line summary to the docket/journal, emits nothing on a clean
  (zero-debt) pass. Pre/post `consign survey` snapshot in the receipt.
- **consign-verify** — rust-extend. Post-drain convergence check: re-run the
  survey, prove the `auto-ok` push-debt dropped to zero; if a repo that
  `drain` reported pushed is still `ahead`, emit a *contradicted* verdict
  (exit 1) — never a silent green. Reuses headway-verify's
  contradicted-never-false-close precedent.

## Order

```
consign-survey ─► consign-policy ─┬─► consign-publish ─┐
                                  └─► consign-drain ───┼─► consign-cron
                                                       └─► consign-verify
```

`consign-survey` is foundational (NEW repo). `consign-policy` gates every
write path, so it lands before `publish`/`drain`. `publish` and `drain` are
parallel (publish handles `no-remote`, drain handles `ahead`/`no-upstream`).
`consign-cron` wires the drainer to a timer; `consign-verify` proves
convergence. Both depend on `drain`.

## Open questions (for the next /dream pass or /build)

- **Branch policy.** `homeward`'s unpushed commits are on a `/build` worktree
  branch (`autobuilder/homeward-found-geocode`), not `main`. Should consign
  push feature/worktree branches at all, or only the default branch? Leaning:
  `consign-policy` marks non-default-branch HEADs `manual-only` by default, so
  drain never silently publishes a half-baked worktree branch as if it were
  release work. Confirm with jsy.
- **Roots beyond ~/wintermute.** `~/.claude` (settings/skills) and `~/projects`
  also hold git repos. Start with `~/wintermute` only; widen via config later.
- **Private-by-default for new mints.** Should `consign-publish` default new
  repos to private or public? Memory says shipped fleet repos are
  public-by-design but autobuilder-private is private. Leaning: inherit from
  `consign-policy` (public for shipped fleet crates, private if any hold/secret
  signal), never an unconditional `--public`.
- **Diverged repos.** consign should *never* auto-resolve a diverged repo
  (`mqo-narrative-compose`). Surface only; a human rebases/merges.
