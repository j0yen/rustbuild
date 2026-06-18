# PRD: chaff-guard — pre-commit prevention so the junk never comes back

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/chaff
Vision: visions/chaff.md

## TL;DR

chaff-repair cleans the index once; without prevention the disease recurs,
because the root cause is that `git add` (often `git add -A` from automated
flows) keeps swallowing regenerable artifacts. chaff-guard closes the loop:
a fast pre-commit check that refuses to let regenerable junk be committed,
installable per-repo as a `.git/hooks/pre-commit`, and runnable standalone
so /build's publish step can call it before committing.

## Why this exists

Live evidence (2026-06-18):

- The footgun is structural: `coda` has `/target/` in `.gitignore` yet
  tracks `target/` files, meaning something committed them *before* the
  ignore — i.e. an ignore line is not sufficient prevention when a flow
  uses `git add -f`, adds a specific path, or the file was tracked first.
- The fleet is built by automation (/build, /autobuilder) that commits on
  every tick; a passive `.gitignore` does not stop a tracked file from
  being re-committed once modified. A `pre-commit` hook that inspects the
  *staged* set is the durable guard.
- The recurrence is observable: across journals 2026-06-12..18 the "dirty
  repos" count never trends to zero — chaff-repair would zero it once, but
  only a guard keeps it there.
- Precedent on this laptop: the bpolicy eBPF-LSM write enforcer and the
  inoculate strain model both encode "prevent the bad action at the
  boundary." chaff-guard is the git-boundary analogue, scoped to staged
  paths only (no kernel involvement).

## What this builds

A `guard` module + `chaff guard` subcommand in `~/wintermute/chaff`.

- `chaff guard check [--staged] [--repo <dir>]` — inspect the staged set
  (`git diff --cached --name-only`) of the current/named repo; if any
  staged path matches `chaff::patterns::REGENERABLE` (reusing the shared
  set, with the same `src/`-style safe-path exclusions as chaff-policy),
  print the offending paths and exit non-zero. Otherwise exit 0. Fast
  enough to run on every commit (no network, no full survey).
- `chaff guard install [--root <dir>] [--all]` — install a
  `.git/hooks/pre-commit` in the named repo (or every `~/wintermute/*`
  repo under `--all`) that invokes `chaff guard check --staged`. The hook
  is idempotent: if a pre-commit hook already exists, append a guarded,
  clearly-delimited block (anchor-comment markers, like the
  `apply-agentns.py` inline-edit pattern) rather than clobbering it; if
  the chaff block already exists, do nothing.
- `chaff guard uninstall` — remove only the chaff-delimited block,
  restoring any prior hook content.
- An `--allow <glob>` escape hatch (and `CHAFF_GUARD_BYPASS=1` env) so a
  deliberate `git add -f` of a genuinely-needed artifact can still go
  through, logged to stderr.
- Exit codes: 0 = clean, 1 = junk staged (block the commit), 2 = usage.

Deps: reuse chaff-survey's pattern set + policy's safe-path exclusions. No
network. MSRV 1.85, no let-chains. `sigpipe::reset()` in `main`. The
installed hook must degrade gracefully: if the `chaff` binary is absent at
commit time, the hook prints a one-line warning and exits 0 (never block a
commit because the tool was uninstalled).

## Acceptance criteria

1. `chaff guard check` in a repo with `target/x.o` staged exits non-zero
   and prints the offending path.
2. `chaff guard check` in a repo with only `src/main.rs` staged exits 0.
3. `chaff guard install` creates `.git/hooks/pre-commit` (executable) that
   calls `chaff guard check --staged`; a subsequent `git commit` that
   stages a `target/` file is rejected.
4. `install` on a repo with a pre-existing pre-commit hook appends a
   delimited chaff block and preserves the original hook's behavior;
   running `install` again does not duplicate the block.
5. `chaff guard uninstall` removes only the chaff block, leaving any prior
   hook content intact.
6. `CHAFF_GUARD_BYPASS=1 chaff guard check` with junk staged exits 0 and
   logs the bypass to stderr.
7. The installed hook exits 0 with a warning (does NOT block) when the
   `chaff` binary is not found on PATH.
8. `chaff guard install --all` installs into every `~/wintermute/*` git
   repo and reports a count of repos touched vs already-installed.
