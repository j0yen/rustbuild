# PRD: relay-intake

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/relay
Vision: visions/relay.md

## TL;DR

After a helper talks to someone, they have to write it up — a case record, the
key facts, what to do next. It's slow, it's inconsistent, and it's where details
get lost. `relay-intake` turns a messy, spoken-style story into a structured,
privacy-respecting **case record** plus a **next-actions checklist** — offline,
so nothing about a vulnerable person leaves the machine. The structuring is the
testable contract; the prose understanding is the local LLM.

## Why this exists

- **Evidence:** `relay-match` produces `Needs` for resource lookup, but a helper
  also needs a durable, shareable-with-consent record of the *person's situation*
  and the plan — the artifact the next shift or the supervising caseworker reads.
  That write-up is named overhead in visions/relay.md.
- **Evidence — privacy is the whole point.** Intake notes are the most sensitive
  data a helper handles (disclosures of abuse, immigration status, health). The
  local LLM ladder (qwen, installed) is what makes structuring them ethical;
  cloud APIs are categorically off-limits (visions/relay.md design note).

## What this builds

- New crate `relay-intake` in the workspace, wired as `relay intake`.
- **`CaseRecord` schema**: consented summary, presenting needs (links to
  `relay-match` `Needs`), risk/urgency flags, demographics-only-if-volunteered,
  barriers, a timeline, and a `redactions` list — with an explicit `consent`
  field gating what may be exported.
- **`Structurer` trait** (`LocalLlmStructurer` + `MockStructurer`): free-text
  story → `CaseRecord` + `Vec<NextAction>` (each action: what, who-owns-it,
  optional linked resource from the directory). The LLM proposes; the schema
  constrains.
- **PII minimization (deterministic, not the LLM):** a redaction pass that
  detects obvious direct identifiers (emails, phone numbers, SSNs) via regex and
  flags them in `redactions`, so an exported summary can be auto-minimized. This
  is a tested, rule-based layer independent of the model.
- **CLI**: `relay intake --story <file|-> [--json] [--minimized]`; `--minimized`
  applies the redaction pass on output.

## Acceptance criteria

1. `relay intake --help` listed under `relay`; crate builds in the workspace.
2. With a `MockStructurer`, a fixture story yields the expected `CaseRecord` +
   `NextAction` list (golden test).
3. Redaction pass (rule-based) detects emails / phone numbers / SSN-shaped
   strings on fixtures and lists them in `redactions`; `--minimized` output
   contains none of them (tested, no LLM).
4. `consent` defaults to the most restrictive value; export/`--minimized`
   refuses to emit fields above the consent level (tested).
5. Next actions can link to a directory/`relay-match` resource id; an action
   referencing an unknown resource is rejected, not silently dropped.
6. Privacy: under `MockStructurer`, processing a story makes no outbound network
   connection and writes nothing outside the explicit `--out` path (asserted).
7. Graceful degrade: if the local model is unreachable, intake still produces a
   minimal record from the rule-based layer (timeline + redactions) and a clear
   "LLM unavailable, partial record" notice.
8. **Deferred / manually-verified AC:** structuring quality vs. real qwen on 10
   sample stories — hand-verified, not a cloud test (`deferred_acs`).

ACs 1–7 deterministic / cloud-build-safe; AC8 is the only live-model check,
deferred.
