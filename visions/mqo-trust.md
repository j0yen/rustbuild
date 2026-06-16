# Vision: mqo-trust — the safety & runtime layer for agentic BI on mqo-mcp

> Drafted 2026-06-15. Seed: user prompt — "more for mqo-mcp server."
> Sibling fleets: mqo-tools (analytical tools) + ousia-mqo (semantic grounding),
> both dreamed earlier today. This fleet covers the THIRD slice: trust, governance,
> and runtime efficiency — what an enterprise needs before pointing an AI agent at
> production data.

## TL;DR

`mqo-mcp` makes the semantic layer the contract and grounds every field before
execution — but it answers with a number and no calibrated confidence, picks
silently when a bind is ambiguous, has no notion of sensitive/PII columns, returns
raw backend faults the agent can't act on, and re-executes identical queries. Five
capabilities — confirmed absent from the 50-crate workspace (grepped 2026-06-15:
no crate matches confidence/clarif/pii/error/cache) — close the trust-and-runtime
gap: a confidence signal on every bind, a clarifying question when a bind is
genuinely ambiguous, a sensitivity scan that flags/redacts before rows reach the
model, an error explainer that turns XMLA/DAX faults into actionable causes, and a
semantic result cache keyed on the bound query.

## End-state

An agent querying production data through mqo-mcp: knows when a binding is shaky
(and says so instead of bluffing); asks "did you mean gross or net margin?" rather
than guessing; never leaks an un-flagged SSN column into the chat; gets "the date
hierarchy isn't queryable at day grain — use month" instead of a raw HRESULT; and
pays the query cost once for two phrasings of the same question.

## Components

- **mqo-binding-confidence** (`rust-cli`, new repo) — `score` subcommand: a
  BoundMqo (or bind candidates) → a calibrated confidence per field + ranked
  alternatives, so the agent/user know which binds to double-check.
- **mqo-clarify** (`rust-cli`, new repo) — `ask` subcommand: when ≥2 candidates
  for a field score within a margin, emit a natural-language disambiguation
  question + the option set, instead of silently picking one.
- **mqo-sensitivity-scan** (`rust-cli`, new repo) — `scan` subcommand: a model or
  result + a sensitivity tag/pattern map → flagged sensitive fields, with an
  optional `--redact` that masks values before they reach the model.
- **mqo-error-explain** (`rust-cli`, new repo) — `explain` subcommand: a backend
  fault string (XMLA/DAX/MDX/SQL/PGWire) → a structured `{cause, category,
  suggested_fix}` mapped from a known-fault catalog.
- **mqo-result-cache** (`rust-cli`, new repo) — `key`/`get`/`put` subcommands: a
  content-addressed cache keyed on the canonicalized BoundMqo, so semantically
  identical queries (different phrasings) share one cached result.

## Order

All five are independent (each consumes documented mqo-spec shapes or plain
strings; none depends on another). binding-confidence + clarify are a natural pair
(clarify consumes confidence's margins) but clarify can also run standalone over a
supplied candidate set. Build priority: binding-confidence → clarify → sensitivity
→ error-explain → result-cache.

## Open questions

- Confidence calibration source: lexical/structural features only (deterministic,
  offline) or a learned model? Default deterministic feature score in v1; a learned
  calibrator is a future PRD once mcp-trace-store has labeled binds.
- Sensitivity tags: from a supplied map, from `describe_model` column tags if
  present, or pattern-based (regex on names/values)? Support all three; default to
  the supplied map + name patterns.
- Result-cache backing: in-memory + on-disk JSON (v1), or wire into the existing
  `mqo-duckdb-handle-store`? Default standalone on-disk; integration is a follow-up.
- Error catalog coverage: seed with the most common XMLA/DAX faults; the catalog is
  data (a TOML/JSON file) so it grows without code changes.
