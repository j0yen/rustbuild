# PRD: vellum-wire — cut /build over to vellum, with a parity gate

**Status:** Draft v0.1
**build_target:** shell
**Vision:** visions/vellum.md

## TL;DR

The `vellum` crate (read/scan/amend/manifest) replaces the hand-rolled bash+sed
parser and the loop's sed storm — but only once /build actually calls it. This
PRD repoints `scan-prds.sh` to `vellum scan` (with a bash-parser fallback if the
binary is absent), documents the `vellum amend` workflow in /build's SKILL.md,
and adds an assay step that proves parity on the live corpus before cutover.

## Why this exists

vellum-read/scan/amend/manifest land the capability; nothing consumes it until
it's wired. The cutover is the risky step — `scan-prds.sh` runs every /build
tick and a wrong array bricks the dispatch. So the cutover needs a parity gate,
not a blind swap. Memory `feedback_cloudbuild_over_build` and the build skill's
own scripts directory (`~/.claude/skills/build/scripts/`) are the integration
surface; the hook script is a symlink into dotfiles, so edits go into
`~/wintermute/dotfiles` (the relay/scribe same-tree note).

## What this builds

- Edit `~/.claude/skills/build/scripts/scan-prds.sh` so that, if the `vellum`
  binary is on `$PATH`, it execs `vellum scan "$PRD_DIR"` and emits that;
  otherwise it falls through to the existing bash parser unchanged. The bash
  parser is NOT deleted — it stays as the degraded fallback so a missing/broken
  `vellum` never bricks a /build tick.
- Add a short "frontmatter edits" section to `/build`'s SKILL.md instructing the
  model to use `vellum amend <PRD> --set-status/--add-blocker/--append-iter-log`
  instead of free-form `sed -i`.
- A parity assay script: run the legacy bash parser (`VELLUM_DISABLE=1
  scan-prds.sh`) and `vellum scan` over the live `~/wintermute/autobuilder`
  corpus, normalize and diff the two arrays, and report any slug whose
  `build_target`/`status_line`/`build_into`/`deferred_acs` differ. Cutover is
  gated on this diff being empty for well-formed PRDs.

## Acceptance criteria

1. With `vellum` on `$PATH`, `scan-prds.sh` emits `vellum scan` output; with
   `VELLUM_DISABLE=1` (or the binary absent), it emits the legacy bash-parser
   output unchanged. Both paths exit 0 on the live corpus.
2. The legacy fallback is byte-identical to today's `scan-prds.sh` output when
   vellum is disabled (no behavioral regression for the fallback path).
3. The parity assay reports an EMPTY diff for all well-formed PRDs in the live
   corpus between legacy and vellum output (the cutover gate).
4. `/build`'s SKILL.md contains a frontmatter-edit section naming `vellum amend`
   and its flags; a grep for `vellum amend` in SKILL.md succeeds.
5. The existing `claude-build` cgroup guard / any existing short-circuits at the
   top of `scan-prds.sh` are preserved and still fire BEFORE the vellum exec
   (no new work runs inside a claude-build cgroup —
   `self_build_jam_leaked_tracer`).
6. The change is reversible: setting `VELLUM_DISABLE=1` (or removing the binary)
   restores the exact prior behavior with no other edits.
