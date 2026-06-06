# Vision: relay — force-multipliers for the people who help

## TL;DR

The highest-leverage way to help the most people is to help the people who
*already help* — caseworkers, shelter staff, clinic volunteers, mutual-aid
organizers, teachers, crisis-line responders. They are chronically
under-resourced and spend a punishing fraction of their time not on people but
on **overhead**: figuring out which service fits a person's situation, checking
eligibility, writing the referral, doing the paperwork. `relay` is a private,
offline-capable toolkit that collapses that overhead — it turns a person's messy
story into the right resources, a structured case, and the dreaded paperwork, in
minutes instead of hours. Every helper who uses it serves more people. That is
the multiplier: ship it once, and the reach is everyone *they* reach.

A hard constraint runs through the whole vision: **the data is sacred.** A
person disclosing they're unhoused, undocumented, fleeing abuse, or in crisis
cannot have their story sent to a cloud API. `relay` runs on the **local LLM
ladder** (qwen/llama, already installed) so nothing leaves the helper's machine.
Privacy isn't a feature here; it's the reason this can exist at all.

## End-state

A volunteer at a food bank, a caseworker at a shelter, or a teacher worried
about a student can: type (or paste) what they heard in plain language; get back
the matching local resources ranked by fit, eligibility, and proximity; get a
clean structured case record and a checklist of next actions; and get a
first-draft referral or benefits-appeal letter they can edit and send — all
offline, all in one tool, on a cheap laptop, for free. Small orgs that today run
on paper and spreadsheets get capability that today only big funded agencies can
afford.

## Why this is buildable here (evidence)

- **Local LLM ladder** is live (`ollama list` 2026-06-05: qwen3:8b, qwen2.5:3b,
  llama3.2:3b). Parsing a situation → structured needs, and drafting letters,
  run offline on these. Privacy-by-construction — the whole reason `relay` is
  ethical to ship.
- **`homeward` precedent**: it already ingests open civic data
  (Socrata/RescueGroups) and matches records. `relay-directory` reuses that exact
  ingest-and-normalize pattern, pointed at the **Open Referral / HSDS** standard
  (the real interop format for health & human-services resource directories) and
  211/OpenStreetMap data.
- **deep-research fan-out harness** (`/deep-research`) does fetch → verify →
  synthesize; `relay-fresh` reuses it to keep resource listings (hours, phone,
  eligibility) from going stale — the #1 failure of every resource directory.
- **`wintermute-brain`** provides the local→cloud routing abstraction the LLM
  crates call instead of hand-rolling an ollama client.
- This is an outward-facing sibling of `homeward` (reuniting lost pets). Same
  shape (open data + local matching + a humane mission), aimed at people.

## Components (PRD-sized, dependency order)

1. **relay-directory** (new repo `~/wintermute/relay`, rust-cli + lib) — ingest
   resource/service listings (HSDS / 211 / OSM amenities) → normalize to a local
   schema → query by need + location + eligibility. Foundation crate + the
   `relay` binary that hosts the workspace.
2. **relay-match** (rust-extend → relay) — given a person's situation in free
   text, extract structured needs (local LLM) and rank directory resources by
   fit / eligibility / proximity. The deterministic matcher is the testable core.
3. **relay-intake** (rust-extend → relay) — turn a messy human story into a
   structured, privacy-respecting case record + a next-actions checklist, fully
   offline (no personal data leaves the machine).
4. **relay-letters** (rust-extend → relay) — draft the paperwork helpers hate:
   referral letters, benefits-appeal letters, intake summaries, from a case
   record + template. Human-edits-before-send; never auto-sends.

## Order

`directory → match → intake → letters` is a clean dependency chain and a
**complete, useful product on its own**: gather resources → match a person →
structure their case → produce the paperwork. /build can ship `directory` first
(usable standalone as a searchable local resource directory), then layer the
rest.

## Design note for buildability (carried from concord)

`/build` runs cargo on the **cloud box (no ollama)**, so every LLM-touching crate
injects its model client behind a trait and tests the deterministic logic against
a **mock LLM + golden fixtures** (schema normalization, dedup, the ranking
matcher, template filling, the next-actions extractor on a labelled set). Live
local-LLM behavior is a manually-verified / deferred AC — never a cloud-gated
test. This keeps the fleet buildable AND keeps the privacy guarantee honest (the
real model is always local).

## Open questions (next /dream pass — not yet drafted)

- **relay-fresh** — re-verify listings via the deep-research harness, flag dead
  entries. Well-scoped; draft next pass.
- **relay-serve** + **relay-web** — a bus/HTTP surface and a browser UI so
  non-CLI helpers (most of them) can use it. Needs the homeward-style hosting
  decision; outward-facing = non-technical users.
- **relay-volunteer** — match volunteers to needs/shifts. Separate enough it may
  be its own vision.
- Which directory standard to anchor on first (HSDS is the right interop bet);
  and the hardest real problem — keeping local resource data accurate.
- Safety: never give legal/medical *advice*, only navigation + drafts a human
  reviews. Where exactly is that line drawn in `relay-letters`?
