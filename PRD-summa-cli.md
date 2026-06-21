# PRD-summa-cli

**Status:** Draft v0.1
**Vision:** visions/summa.md
**build_target:** rust-cli

**Depends on:** PRD-summa-schema (the vault conventions this binary encodes)

## TL;DR

summa needs a deterministic mechanics layer the LLM can lean on: extract text from
a dropped source, regenerate the index, append to the log, and compute the wikilink
graph (orphans + dangling targets). These are pure, testable operations with no LLM
judgment — exactly what belongs in a CLI rather than a skill. This PRD builds the
`summa` binary (new repo `j0yen/summa`) providing `ingest`, `index`, `log`, and
`links`. The `/summa` skill (later PRD) calls these for mechanics and adds the
synthesis on top.

## Why this exists

**Evidence (2026-06-21):**
- The vault has **81 PDFs with no synthesized companion page** — ingest must start
  by extracting text deterministically before any LLM can summarize it. That
  extraction (PDF→text, URL→reader-markdown, md→passthrough) is mechanical and
  should be a tested CLI command, not ad-hoc shell in a skill.
- `index.md`/`log.md` (seeded by summa-schema) must be **regenerated/appended
  programmatically** — Karpathy's navigation files are only useful if they stay in
  sync with the tree, and hand-maintenance is the decay he warns about.
- **149 files use `[[wikilinks]]`** but the graph is uncomputed; finding orphan
  pages (no inbound links) and dangling targets (`[[X]]` with no `X.md`) is the
  raw material the lint PRD consumes — it belongs in a deterministic `links`
  command emitting JSON.
- This ecosystem's pattern is **CLI mechanics + skill judgment** (cf. `recall` the
  CLI vs. recall the skill behavior; `vellum` the typed PRD reader vs. /build).
  summa-cli is the recall-shaped half.

## What this builds

New repo `j0yen/summa`, binary `summa` (Rust, rustc 1.85, MSRV-safe, no let-chains).
Subcommands:

1. **`summa ingest <path-or-url>`** — classify the source; extract text:
   - PDF → text (via a pure-Rust pdf text crate, e.g. `pdf-extract`, or shell to
     `pdftotext` if present — declare the chosen path in the README).
   - URL → reader markdown (fetch + HTML→markdown; offline-fail gracefully).
   - `.md` → passthrough.
   Stage the raw source under `~/Notes/Clippings/` (or `--dest`), and emit a
   **synthesis stub** to stdout (JSON: `{source, staged_path, extracted_text_path,
   suggested_title}`) for the skill to consume. Does NOT write summary/entity pages
   (that's the skill's LLM job).
2. **`summa index`** — regenerate `~/Notes/index.md` from the vault tree: group by
   subject subdir, list entity pages and answer pages in their sections, between
   `<!-- summa:index-start -->` / `<!-- summa:index-end -->` anchors so any
   hand-written preamble is preserved.
3. **`summa log <kind> <subject> <note>`** — append one timestamped line to
   `~/Notes/log.md` in the schema's documented format. Append-only; never rewrites.
4. **`summa links [--json]`** — parse all `[[wikilinks]]` across the vault, build
   the graph, and report: orphan pages (no inbound links), dangling targets
   (linked but no matching file), and malformed links (the `\|` clipper escaping).
   JSON output for machine consumption by summa-lint.

Vault root resolves from `$SUMMA_VAULT` (default `~/Notes`). `sigpipe::reset()` in
`main()` (local-CLI SIGPIPE convention). Unit tests over a temp fixture vault.

## Acceptance criteria

1. `summa ingest <fixture.pdf>` extracts non-empty text, stages the source under
   the vault's `Clippings/`, and prints valid JSON with `source`, `staged_path`,
   and `extracted_text_path` keys. (PDF text extraction verified on a fixture.)
2. `summa ingest <fixture.md>` passes the markdown through and emits the stub JSON;
   `summa ingest https://example.com` either returns reader markdown or fails
   cleanly with a non-zero exit and a clear offline/fetch error (no panic).
3. `summa index` writes `index.md` with content between the `summa:index-start/end`
   anchors and leaves any text outside the anchors untouched (idempotent: a second
   run produces no diff).
4. `summa log ingest AtScale "test entry"` appends exactly one correctly-formatted
   timestamped line to `log.md` and changes nothing else.
5. `summa links --json` over a fixture vault correctly enumerates orphans, dangling
   targets, and malformed `\|` links (asserted against a known fixture graph).
6. `cargo test --release` passes; `summa --help` lists all four subcommands.
