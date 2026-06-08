# PRD: vellum-read — typed reader for one PRD's frontmatter

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/vellum.md

## TL;DR

`/build` and `/dream` need to read PRD frontmatter (status, build target,
deferred ACs, blockers, iter_log) reliably. Today that read is a ~150-line
hand-rolled bash+sed parser in `scan-prds.sh` with documented gaps. This PRD
builds the `vellum` crate and its first subcommand, `vellum read <PRD>`, which
parses one PRD file's frontmatter into a typed model and emits typed JSON —
behavioral parity with the bash parser, plus the known gaps closed.

## Why this exists

The build pipeline's PRD parsing is hand-rolled shell with limitations its own
comments admit. From `~/.claude/skills/build/scripts/scan-prds.sh` (read
2026-06-08):

- line 40: "Only inline-list form supported here (e.g. `deferred_acs: [1, 3,
  5]`)" — the YAML block form and `AC2-foo` form silently parse to `[]`
  (corroborated by memory `self_deferred_acs_inline_only`: the archive gate
  never passes for block-form PRDs).
- lines 68–80: a `sed`-based bold-markdown-key normalization plus a
  fenced-code-block skip plus first-match-wins, all hand-maintained.

This fragility is a proven dispatch-bug source (`self_build_jq_escape_reads_absent`,
`self_build_manifest_join_slug`). A typed parser with a test suite is the
root-cause fix. `vellum read` is the lib core every later vellum subcommand
builds on.

## What this builds

- New crate at `~/wintermute/vellum` (rust-cli, MSRV 1.85, no let-chains per
  `self_recall_baseline_gate_red`). `sigpipe::reset()` as the first line of
  `main()` per `self_sigpipe_panic_toolkit`.
- A `Frontmatter` typed model: `slug`, `path`, `status_line: Option<String>`,
  `build_target: Option<String>`, `build_priority: Option<String>`,
  `build_into: Option<String>`, `build_version_bump: Option<String>`,
  `deferred_acs: Vec<u32>`, `blockers: Vec<String>`, `iter_log: Vec<String>`,
  `vision: Option<String>`, `size_bytes: u64`, `mtime_iso: String`.
- A parser that: detects `---`-delimited YAML frontmatter and also the bare /
  bold-markdown key forms PRDs use; skips fenced code blocks; honors
  first-match-wins; and parses `deferred_acs` from inline `[1,3,5]`, YAML block
  list, AND `AC2-foo`-style tokens (extracting the leading integer).
- `vellum read <PRD>` emits the model as one JSON object on stdout. `--pretty`
  for human reading. Exit non-zero with a JSON error object on parse failure.
- Unit tests over fixtures covering: YAML frontmatter, bold-markdown keys,
  fenced-code poisoning, all three deferred_acs forms, and a malformed file.

## Acceptance criteria

1. `cargo build` and `cargo test` are green; `clippy` adds no NEW warnings over
   a fresh crate baseline. MSRV 1.85, no let-chains.
2. `main()`'s first statement is `sigpipe::reset()` (or equivalent); `vellum
   read <PRD> | head` does not panic.
3. `vellum read <PRD>` on a real PRD with YAML frontmatter emits a JSON object
   with all model keys; values match the file.
4. `vellum read` correctly parses `deferred_acs` from ALL three forms (inline
   `[1,3,5]`, YAML block list, `AC2-foo`) — a fixture per form asserts the same
   `[2]`/`[1,3,5]` output the form encodes. (Closes the gap in
   `self_deferred_acs_inline_only`.)
5. `vellum read` skips keys that appear only inside fenced code blocks, and
   honors first-match-wins for duplicated keys — parity with `scan-prds.sh`.
6. A malformed/empty PRD yields exit non-zero and a JSON `{"error": …, "path":
   …}` object, NOT a panic and NOT a partial object.
