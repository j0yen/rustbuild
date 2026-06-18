# PRD: chaff-survey — honest tracked-build-artifact enumerator

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/chaff
Vision: visions/chaff.md

## TL;DR

Self-review reports "56 dirty repos" in every journal, but a large slice
of that count is not real work — it is regenerable build artifacts that
were committed into git and now churn against the disk fleet's
`cargo clean`. chaff-survey is the honest enumerator: it walks every
`~/wintermute/*` git repo, lists tracked files matching a regenerable
artifact pattern set (`target/`, `node_modules/`, `.venv/`, `dist/`,
`*.o`, `*.rlib`, `*.rmeta`, `__pycache__/`, `.pytest_cache/`), and
classifies each repo into a *strain* — distinguishing "no `.gitignore`
at all" from "`.gitignore` present but the junk predates it." It is the
foundation of the chaff vision (a NEW repo at `~/wintermute/chaff`).

## Why this exists

Live evidence (commands reproducible today on this laptop, 2026-06-18):

- A walk of `~/wintermute/*/.git` running
  `git ls-files | grep -E '^(target/|node_modules/|\.venv/|.*\.(o|rlib|rmeta)$)'`
  found **12 of 226** repos tracking build junk — **~2,745** files total.
- `careen-ledger`: **1942** tracked `target/` files and **no
  `.gitignore`** (`ls .gitignore` → No such file). Its 1942 "dirty"
  entries in `git status` are all ` D target/...` — the disk fleet
  `cargo clean`ed a tracked `target/`, manufacturing the noise.
- `rosetta-prov`: **464** tracked `target/` files, no `.gitignore`.
- `hold-guard`: **203** tracked `target/` files, `.gitignore` present.
- `coda`: `.gitignore` contains `/target/`, yet
  `target/.rustc_info.json`, `target/autobuilder/receipts/iter-0-…json`,
  `target/autobuilder/run.log` are still tracked — proving the
  footgun: `.gitignore` only ignores *untracked* paths, so a line added
  after the commit is cosmetic. This is **strain 2**.
- Others: `agorabus-nats-bridge` (37), `tether-recall` (34),
  `wintermute-reach` (29), `mqo-result-cache` (9), `tether-gossip` (8),
  `tether-link` (6), `corpus-converge` (4), `headway` (2).

No existing fleet owns this. `consign` (visions/consign.md) pushes
*committed* work and would happily drain 1942 junk blobs.
`ballast`/`careen`/`drydock`/`thrift`/`hold` delete `target/` from the
*filesystem* and so *create* the phantom-deletion noise.
`adopt`/`binstale`/`rollout` track *installed binary* freshness. The
"dirty repos" self-review line has recurred every day for a week
(journals 2026-06-12..18) with no owner — see vision §TL;DR.

## What this builds

A new repo `~/wintermute/chaff` with a `chaff` binary and a library crate
the later chaff-* PRDs extend.

- `chaff survey [--root <dir>] [--format json|text]` — default root
  `~/wintermute`. For each git repo found (a dir containing `.git`):
  - run `git ls-files` and match against the regenerable pattern set
    (the set lives in a `patterns` module so chaff-policy can reuse it);
  - read `.gitignore` if present and detect whether it *would* cover the
    matched paths (line-prefix match on `target/`, `/target/`, etc.);
  - classify `strain`:
    - `none` — no tracked junk (omitted from default output, shown with
      `--all`);
    - `no-gitignore` — tracked junk and no `.gitignore` file;
    - `gitignore-stale` — tracked junk AND `.gitignore` covers the
      pattern (the footgun: files predate the ignore line);
    - `gitignore-gap` — tracked junk and a `.gitignore` exists but does
      NOT cover this pattern.
  - emit per repo: `{repo, path, strain, has_gitignore, gitignore_covers,
    tracked_junk, bytes_in_index_est, sample[<=5]}`.
- `bytes_in_index_est` from `git ls-files -s` blob sizes
  (`git cat-file --batch-check` over the matched paths), summed.
- Text output: a ranked table (most tracked_junk first) + a one-line
  summary `N repos, M files, ~B MiB in index`.
- Library API: `survey(root) -> Vec<RepoChaff>` so chaff-policy /
  chaff-repair / chaff-cron consume structured results, not parsed text.

Deps: `serde`/`serde_json`, `clap` (derive), no network. Shell out to
`git` via `std::process::Command` (the repos use a mix of git versions;
don't vendor a git lib). MSRV 1.85, no let-chains (per recall baseline
note). `sigpipe::reset()` first line of `main()` (the `chaff survey |
head` SIGPIPE-panic footgun — see self_sigpipe_panic_toolkit).

## Acceptance criteria

1. `chaff survey --format json` over a fixture tree containing a repo with
   a tracked `target/foo.o` and no `.gitignore` emits one record with
   `strain == "no-gitignore"`, `tracked_junk == 1`, and `foo.o` in
   `sample`.
2. A fixture repo whose `.gitignore` contains `/target/` but still tracks
   `target/x` is classified `strain == "gitignore-stale"` with
   `gitignore_covers == true`.
3. A fixture repo with a `.gitignore` that does NOT mention `target/` but
   tracks `target/x` is classified `strain == "gitignore-gap"`.
4. A clean repo (no tracked junk) is omitted from default output and
   present with `strain == "none"` under `--all`.
5. `bytes_in_index_est` is > 0 for a repo with tracked junk and equals the
   summed blob sizes of the matched paths (±0, exact).
6. `--format text` prints a table ordered by descending `tracked_junk` and
   a final summary line matching `^\d+ repos, \d+ files`.
7. `chaff survey | head -1` does not panic (SIGPIPE handled); exit status
   reflects scan success (0 = scan ran, 2 = usage/IO error).
8. The pattern set is exposed as a public library item (e.g.
   `chaff::patterns::REGENERABLE`) so downstream chaff-* crates reuse it
   rather than re-encoding the list.
