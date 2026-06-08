# Vision: vellum — a typed reader for the corpus /build hand-parses with sed

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` (interactive, no steer). Fourth dream pass of
  2026-06-08. The inward self-tooling space is broadly saturated (recourse
  vision 2026-06-08 logged it; three earlier ticks logged "saturation scan —
  no PRDs"). The earlier passes today mined the *recurring findings* layer
  (tide/tend/scribe/mend/assay/abide). This pass found the one inward signal
  none of them touched: the **build pipeline parses and edits its own corpus
  with hand-rolled shell text-munging**, and that munging is both expensive
  and a proven bug source.

## TL;DR

`/build` and `/dream` read PRD frontmatter and the build manifest, and mutate
PRD frontmatter (status, blockers, iter_log) every tick. Today that reading is
a ~150-line hand-rolled bash+sed YAML/markdown parser (`scan-prds.sh`), and
that mutating is the model running free-form `sed -i` against PRD files in the
build loop. The 2026-06-08 self-review measured **sed×97,812 and jq×2,696 in a
single 79-minute build session**. The fragility is not theoretical — it has
already produced wrong dispatches:

- a bad jq escape in one manifest entry makes every per-slug join error out, so
  every slug reads ABSENT and /build mis-selects a blocked PRD
  (`self_build_jq_escape_reads_absent`)
- joining the manifest on `path` instead of `slug` re-selects already-tracked
  PRDs (`self_build_manifest_join_slug`)
- `scan-prds.sh` parses `deferred_acs` ONLY in inline `[3,4,6]` form; the YAML
  block form and the `AC2-foo` form silently parse to `[]`, so the archive gate
  never passes (`self_deferred_acs_inline_only`)

`vellum` is a single typed Rust CLI that owns this surface: it reads PRD
frontmatter into a typed model, emits the exact JSON shapes `/build` consumes,
amends frontmatter through typed subcommands instead of free-form sed, and
performs manifest joins keyed correctly on slug. The hand-rolled bash parser
and the loop's sed storm both retire behind one tool with a test suite.

## End-state

When this is done:

- `scan-prds.sh`'s `emit_one` parser is replaced by `vellum scan <dir>`, which
  emits a byte-compatible superset of the current array — same keys, but with
  `deferred_acs` parsed from inline AND block AND `AC2-foo` forms, and with no
  jq-escape failure mode (a malformed entry fails *that entry*, never the whole
  scan).
- `/build`'s SKILL.md instructs the model to call `vellum amend <PRD>
  --set-status … --add-blocker … --append-iter-log …` for frontmatter edits
  instead of free-form `sed -i`. The sed×97k/session figure drops by an order
  of magnitude.
- Manifest reads/joins go through `vellum manifest`, keyed on slug, so the
  ABSENT-mis-select and re-select-tracked-PRD bug classes are structurally
  impossible.
- The whole surface has a cargo test suite over real corpus fixtures, so
  frontmatter-shape regressions are caught at build time, not at dispatch time.

## Components (one bullet per PRD)

- **vellum-read** — new crate `~/wintermute/vellum`; `vellum read <PRD>` parses
  one PRD's full frontmatter (status, build_target, build_into,
  build_version_bump, build_priority, deferred_acs in all three forms,
  blockers[], iter_log, vision pointer) into a typed model and emits typed
  JSON. The lib core + first subcommand. Handles YAML frontmatter, bare keys,
  bold-markdown keys (`**build_target:**`), fenced-code-block skipping, and
  first-match-wins — i.e. behavioral parity with `scan-prds.sh` plus the gaps
  closed.
- **vellum-scan** — rust-extend `vellum`; `vellum scan <dir>` walks the PRD
  directory (excluding `PRDs-archive/`) and emits the array shape `scan-prds.sh`
  emits today, as a drop-in. A malformed PRD degrades to a per-entry error
  object, never aborts the scan.
- **vellum-amend** — rust-extend `vellum`; typed in-place frontmatter edits:
  `--set-status`, `--add-blocker`/`--clear-blockers`, `--append-iter-log`,
  `--set build_version_bump=…`. Idempotent, preserves byte-for-byte everything
  it doesn't touch, and refuses to corrupt a file (writes via temp+rename).
  This is the subcommand that retires the loop's sed storm.
- **vellum-manifest** — rust-extend `vellum`; typed manifest ops keyed on slug:
  `vellum manifest get <slug>`, `vellum manifest join <scan.json>`,
  `vellum manifest set <slug> <key> <value>`. Makes the join-on-slug and
  no-ABSENT-on-bad-entry invariants structural.
- **vellum-wire** — shell/hooks; repoint `scan-prds.sh` to call `vellum scan`
  (keeping the bash parser as a fallback if the binary is absent), document the
  `vellum amend` workflow in `/build`'s SKILL.md, and an assay step that diffs
  `scan-prds.sh` legacy output against `vellum scan` on the live 101-PRD corpus
  to prove parity before the cutover.

## Order

```
vellum-read → vellum-scan → vellum-amend → vellum-manifest → vellum-wire
```

- vellum-read builds the crate (lib + `read`). It must land first.
- vellum-scan, vellum-amend, vellum-manifest all **rust-extend the same crate**
  and edit overlapping files (`main.rs` subcommand match, `lib.rs` pub surface,
  the parse module). Build them **strictly serially**, each fully landed before
  the next starts, or the edit anchors collide (the relay/tide/scribe/abide
  same-crate rule). Do NOT dispatch them in parallel.
- vellum-wire lives in a different tree (the build skill + dotfiles), so it
  can't collide with the crate PRDs; but it's last — it needs `vellum scan`
  (scan), the documented `amend` workflow (amend), and `manifest` (manifest)
  all live to prove parity.

## Open questions

- Should `vellum` be a brand-new crate or extend an existing corpus tool?
  Lean: new crate. `almanac`/`christen`/`docket` are adjacent but each owns a
  different domain; a PRD-frontmatter parser is its own concern and conflating
  it would bloat an unrelated crate.
- Cutover safety for `scan-prds.sh`: hard replace vs. fallback-on-absent-binary?
  Lean: fallback. The bash parser stays as a degraded path so a missing/broken
  `vellum` binary never bricks the whole `/build` tick. vellum-wire's AC
  asserts the fallback works.
- Should `vellum amend` enforce the docket/abide invariant that status edits
  don't silently clobber recurrence state? Out of scope here — vellum amends
  PRD files, not the docket DB. Noted for cross-vision awareness only.
- Does `vellum read` also emit acceptance-criteria structure (for the archive
  gate / verified-completed pairing)? Lean: not in this fleet — keep vellum a
  frontmatter tool; AC-body parsing is a separate concern. Left as a bullet for
  a future `/dream extend vellum`.
