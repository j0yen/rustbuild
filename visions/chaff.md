# Vision: chaff — keep build artifacts out of git

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-18
**Status:** active
**Seed:** bare `/dream` (auto-mode, no topic), field FRESH (streak=0,
last-productive 2026-06-18T09:15Z). Phase-1 live walk of every
`~/wintermute/*` git working tree found a sharp, recurring, **unowned**
hygiene gap underneath the self-review line "56 dirty repos".

## TL;DR

The fleet's git repos are tracking files they should never track. A live
walk on 2026-06-18 (`git ls-files | grep -E '^target/|node_modules/|…'`)
found **12 of 226** `~/wintermute/*` repos with build artifacts committed
into version control — roughly **2,745** tracked junk files in total:

- `careen-ledger` — **1942** tracked `target/` files, and **no
  `.gitignore` at all**.
- `rosetta-prov` — **464** tracked `target/` files, no `.gitignore`.
- `hold-guard` — **203** tracked `target/` files (plus 6 src).
- `agorabus-nats-bridge` (37), `tether-recall` (34), `wintermute-reach`
  (29), `mqo-result-cache` (9), `coda` (7), `tether-gossip` (8),
  `tether-link` (6), `corpus-converge` (4), `headway` (2).

The disease has two strains:

1. **No `.gitignore`** (careen-ledger, rosetta-prov, corpus-converge,
   headway, tether-gossip) — nothing was ever stopping `git add` from
   swallowing `target/`.
2. **`.gitignore` added *after* the commit** — the classic git footgun.
   `coda/.gitignore` literally contains `/target/`, yet
   `target/.rustc_info.json`, `target/autobuilder/receipts/…` remain
   tracked, because `.gitignore` only ignores *untracked* paths. The
   ignore line is cosmetic; the bytes are still in the index and history.

Why this matters now, and why it's distinct from every existing fleet:

- **`consign`** (drafted 2026-06-18) gets *committed* work off the laptop
  (push unpushed commits). It does **not** decide *what should be
  committed* — it would happily push 1942 build-artifact blobs.
- **`ballast`/`careen`/`drydock`/`thrift`/`hold`** reclaim disk *space*
  by deleting `target/` from the filesystem. When they `cargo clean` a
  repo whose `target/` is tracked, the result is 1942 phantom ` D`
  (deleted) entries — which is exactly why careen-ledger shows 1942
  "dirty" files today. The disk fleet's cleanup *manufactures* the noise.
- **`adopt`/`binstale`/`rollout`** care about *installed binary*
  freshness, not repo contents.

So the recurring "56 dirty repos" self-review line is, in large part,
**not real work** — it's tracked build junk churning against the disk
fleet. Nobody owns making it go away at the root. `chaff` does: it is the
discipline of separating the wheat (source) from the chaff (regenerable
build artifacts) in git itself.

## End-state

When chaff is fully built:

- A scan names every `~/wintermute/*` repo tracking regenerable artifacts,
  distinguishing strain 1 (no `.gitignore`) from strain 2 (`.gitignore`
  present but predates the commit), with exact file counts and byte-in-
  index estimates. JSON for tooling, ranked table for humans.
- A default-deny policy decides which untrack actions are safe (only
  recognized regenerable patterns; never a source file; never a diverged
  or detached repo).
- A repair pass `git rm -r --cached`s the approved junk, ensures the
  matching ignore line exists, and commits the deletion — dry-run by
  default, one repo at a time, structured verdict per repo (mirrors
  `adopt apply` / `consign drain`).
- Repos with no `.gitignore` get a complete, language-appropriate one
  synthesized from repo type (Cargo.toml → Rust, package.json → node,
  pyproject → python), so the next `git add` can't reintroduce the junk.
- A guard prevents recurrence: a pre-commit check (installable per-repo
  and/or wired into /build's publish step) refuses to stage junk paths.
- A 6-hourly timer keeps the fleet clean and replaces the noisy
  "N dirty repos" self-review line with an honest "N repos tracking build
  junk (M files)" — degrading to a one-liner + exit 0 when chaff is
  absent.

## Components (one bullet per future PRD)

- **chaff-survey** — NEW repo `~/wintermute/chaff` (rust-cli + lib). Walk
  every `~/wintermute/*` git repo; classify tracked files against a
  regenerable-artifact pattern set; per repo emit `{strain, has_gitignore,
  gitignore_covers, tracked_junk, sample, bytes_in_index_est}`. Foundation.
- **chaff-policy** — rust-extend into chaff. Default-deny gate: which
  untrack actions are safe. Pattern allowlist + HARD exclusions (never a
  source file, never a diverged/detached/mid-rebase repo, never a path
  outside a recognized regenerable dir). Gates repair.
- **chaff-gitignore** — rust-extend into chaff. Synthesize a complete,
  language-appropriate `.gitignore` for repos that have none. Additive and
  safe (no untracking), so it runs independent of policy.
- **chaff-repair** — rust-extend into chaff. For policy-approved repos:
  `git rm -r --cached <junk>`, ensure the ignore line, stage + commit the
  deletion with the Joe Yen identity. Dry-run by default; one repo at a
  time; structured verdict.
- **chaff-guard** — rust-extend into chaff (+ shell installer). Prevention:
  a pre-commit hook (installable per-repo) that rejects staging
  regenerable junk, closing the recurrence loop.
- **chaff-cron** — shell. 6-hourly systemd-user timer (mirrors
  adopt-cron / ballast-pilot / consign-cron) + additive guarded
  self-review skill-doc block replacing the "N dirty repos" line.

## Order

```
chaff-survey ─► chaff-policy ─► chaff-repair ─► chaff-guard ─► chaff-cron
            └─► chaff-gitignore (parallel; additive, no policy gate)
```

- chaff-survey is foundational; build first.
- chaff-gitignore depends only on survey and is independent of the
  destructive arm — parallelizable with policy/repair.
- chaff-repair GATES on chaff-policy (never untrack without the gate).
- chaff-guard depends on the pattern set settled by policy.
- chaff-cron is last (wires survey+repair into a timer + self-review).

## Open questions

- Should chaff-repair rewrite *history* to purge already-committed blobs
  (`git filter-repo`) to actually reclaim `.git` bloat, or only stop
  tracking going forward? Leaning **forward-only** for v1 — history
  rewrite is irreversible and breaks any existing clone/remote; park it as
  a separate opt-in PRD if `.git` bloat proves material.
- Where does chaff-guard live — per-repo `.git/hooks/pre-commit`, a
  `core.hooksPath` global hook, or a check inside /build's publish step?
  Leaning per-repo installer + an optional /build integration, so it works
  for hand-driven commits too.
- Overlap with consign-publish/consign-verify: chaff should run *before*
  consign drains, so consign never pushes junk. Sequencing hint for /build
  in gossip; no code dependency.
- Should `target/autobuilder/receipts/` be exempt (they're arguably
  audit records, not pure chaff)? Leaning **no** — they're regenerable per
  the autobuilder-receipt-order note; treat as chaff.
