# PRD-summa-lint

**Status:** Draft v0.1
**Vision:** visions/summa.md
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/summa
**build_version_bump:** minor

**Depends on:** PRD-summa-cli (extends the `summa` binary; consumes `summa links`)
**Design spec (authoritative):** visions/summa-design.md §4 — exact lint predicates
and `--fix` scope table; §0 — ownership rule that bounds what `--fix` may touch.

## TL;DR

Karpathy's pattern relies on "periodic lint passes to detect stale claims, orphan
pages, and missing cross-references" — that's what keeps the compounding artifact
from rotting. The vault already shows the rot: 18 files carry broken `\|` escaping
from the web clipper, wikilinks point at numeric clipper IDs, and 81 PDFs have no
companion page. This PRD extends the `summa` binary with `lint [--fix]`: a report
of orphans, dangling/malformed links, missing-index pages, and stale-vs-source
summaries, with mechanical auto-repair behind `--fix`.

## Why this exists

**Evidence (2026-06-21):**
- `grep -rl '\|' ~/Notes/` → **18 files** with backslash-pipe escaping like
  `[[PRD: PR #80\|4712398850]]`. This is mechanical, repeatable rot the LLM
  shouldn't burn judgment on — a deterministic `--fix` should normalize `\|`→`|`
  and strip clipper numeric-ID aliases.
- **81 PDFs with no companion summary page** = source-without-synthesis; lint should
  surface these as "un-ingested sources" so the user/skill knows what to feed
  `/summa ingest`.
- `summa links` (from summa-cli) already computes orphans + dangling targets; lint
  is the consumer that turns that graph into an actionable, prioritized report and
  optionally repairs the mechanical subset.
- Karpathy explicitly lists "stale claims" — a source-summary page whose
  `ingested:` timestamp predates its source file's mtime is a deterministic
  staleness signal lint can flag without an LLM.

## What this builds

Extends `~/wintermute/summa` (the binary from summa-cli) with one subcommand:

**`summa lint [--fix] [--json]`** — runs these checks:
1. **Orphans** — pages with no inbound `[[links]]` (from `summa links`), excluding
   `index.md`/`log.md`/`CLAUDE.md` and raw sources.
2. **Dangling links** — `[[X]]` with no matching `X.md`; report the linking files.
3. **Malformed links** — the `\|` escaping and clipper numeric-ID aliases.
   **`--fix` repairs these**: `\|`→`|`, and `[[Title\|12345]]`→`[[Title]]`.
4. **Missing-index** — pages absent from `index.md`'s anchored section.
   **`--fix` runs `summa index`** to regenerate.
5. **Stale-vs-source** — `source-summary` pages whose `ingested:` frontmatter is
   older than the mtime of the `source:` they cite. Report only (no auto-fix — a
   true re-summary is the skill's LLM job).
6. **Un-ingested sources** — raw PDFs/clippings with no `source-summary` page
   pointing at them.

Output: a grouped human report (default) or `--json`. `--fix` performs ONLY the
mechanical repairs (malformed links, index regen), prints what it changed, and
leaves stale/un-ingested/orphan items as report-only (they need human or LLM
judgment). Idempotent: `--fix` twice produces no second diff.

## Acceptance criteria

1. `summa lint` over a fixture vault reports each category (orphans, dangling,
   malformed, missing-index, stale-vs-source, un-ingested) with counts and the
   offending file paths.
2. `summa lint --fix` repairs `\|`→`|` and strips clipper numeric-ID aliases in a
   fixture, verified by a follow-up `summa links` showing zero malformed links; a
   second `--fix` run produces no further changes (idempotent).
3. `summa lint --fix` regenerates `index.md` so previously missing-index pages are
   listed; pages already present are unchanged.
4. Stale-vs-source detection flags a fixture page whose `ingested:` predates its
   `source:` mtime, and does NOT flag a fresh one.
5. `--fix` never modifies a `source-summary` body, a raw source, or a stale page's
   content — only mechanical link/index repairs (asserted on a fixture).
6. `cargo test --release` passes; `CHANGELOG.md` gains a `## v<new>` section.
