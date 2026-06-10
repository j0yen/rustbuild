# PRD: homeward-federation-export — syndicate a lost report out, honestly

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md
Depends: none (extends shipped homeward-report; independent of the petfbi/dedup
  pull-IN pair — outbound vs inbound). Reads `LostReport` from homeward-schema.

## TL;DR

The homeward vision's federation dream is "one report, every channel" — file once, reach
PawBoost, Pet FBI, Nextdoor, the neighborhood. The federation research (2026-06-10)
killed the easy version of that: **no lost-pet network accepts a third-party report via
open API.** PawBoost is shelter-inbound-only, Nextdoor's write path is partnership-gated,
Facebook Groups' API was deprecated by Meta in April 2024, Petco Love Lost is closed.
Building "syndication connectors" for them would be fiction. But the owner-facing value
is still real and *is* buildable: take a homeward `LostReport` and produce a **portable,
standards-shaped export** — a structured JSON-LD record (no open lost-pet schema exists,
so homeward defines one extending schema.org) plus a human-postable flyer — that the
owner (or a future partnership) can carry to any channel. Plus a **pluggable `Syndicator`
trait** with the one realistic machine target (a Pet FBI partner-POST adapter) built as a
**gated, dry-run-by-default** path that refuses to claim delivery it can't make. Honest
outbound: what works, works now; what needs a partnership says so out loud.

## Why this exists

Phase-1 evidence (federation research 2026-06-10, citations below):

- **Every post-OUT channel is gated or dead.** Verdicts: PawBoost — no third-party API,
  shelter-inbound only (https://www.pawboost.com/blog/pawboost-for-shelters-faqs/).
  Nextdoor — "Share Content" is partnership-approval-gated, no lost-pet write endpoint
  (https://developer.nextdoor.com/). Facebook **Groups API deprecated April 2024**, all
  third-party group posting removed
  (https://www.sprinklr.com/help/articles/getting-started/meta-deprecates-facebook-groups-api/66229eb25f9dd9599d632712).
  Petco Love Lost — closed/partner-only. Drafting connectors for these would violate the
  vision's own hard rule against dreaming past the research.
- **Pet FBI is the only plausible partner write, and it's an outreach not an endpoint.**
  Pet FBI/HeLP is an open-ethos nonprofit with a read feed (see
  homeward-federation-petfbi) but **no published write API** — a partner POST is the
  realistic ask, contingent on an email and a credential. So the adapter must exist in
  *shape* but never pretend to deliver until a real endpoint+key is configured.
- **No open lost-pet interchange standard exists.** schema.org has `AnimalShelter` and a
  `Pet` enum but **no lost/found-report type** (https://schema.org/AnimalShelter) — a
  genuine schema gap. The portable export is therefore homeward defining its own JSON-LD
  shape, extending schema.org vocabulary where it overlaps.
- **The owner-facing artifact is unconditionally useful.** Manual posting to the gated
  networks is exactly what owners do today by hand; a generated, consistent flyer +
  structured record makes that faster and is the one outbound deliverable that depends on
  nobody's partnership. `homeward-report` already owns the owner side
  (`homeward-report/src/{api,alerts,exif}.rs`), including EXIF-strip — the right home.

## What this builds

An extension to `homeward-report` (no new crate):

- **A `LostReportExport` JSON-LD serializer** — maps a `LostReport` to a portable
  record using schema.org vocabulary for the overlapping fields (animal, location,
  contact) plus homeward-namespaced fields for lost/found semantics schema.org lacks.
  EXIF-stripped photo reference reusing `homeward-report/src/exif.rs`; brokered contact
  (never the owner's raw details) consistent with the report side's privacy model.
- **A human-postable flyer renderer** — a deterministic text/markdown (and/or simple
  HTML) "LOST: <name>, <breed>, last seen <coarse location/date>, contact via <broker>"
  artifact the owner can paste into any of the gated channels by hand. Always works,
  zero external dependency.
- **A `Syndicator` trait** — `fn syndicate(&self, export: &LostReportExport) ->
  SyndicationOutcome` — with two implementations:
  - **`LocalArtifactSyndicator`** (default, always-on): writes the JSON-LD + flyer to a
    configured output dir. This is the real, working outbound path.
  - **`PetFbiPartnerSyndicator`** (behind Cargo feature `petfbi-partner`, **dry-run by
    default**): formats the partner POST but returns `SyndicationOutcome::DryRun` unless a
    real endpoint URL + credential are configured; on a configured run it POSTs and
    reports honest delivery/failure. It must NEVER report success without an actual 2xx.
- **No fictional transports.** PawBoost/Nextdoor/Facebook/Petco are represented (if at
  all) only as `SyndicationOutcome::ManualOnly { channel, reason }` entries that tell the
  owner "post this flyer here yourself — no automated path exists," citing the gated
  status. Zero code pretends to deliver to them.

## Acceptance criteria

1. `cargo build -p homeward-report` clean; workspace `cargo test` green with new tests.
2. A `LostReport` fixture serializes to a `LostReportExport` JSON-LD document that
   validates as well-formed JSON-LD and carries schema.org-typed fields plus the
   homeward-namespaced lost/found fields — asserted field by field.
3. The exported photo reference is EXIF-stripped (reuses `exif.rs`) and the contact is
   the brokered form, never raw owner PII — asserted by a test that checks no raw
   contact/EXIF leaks into the export.
4. The flyer renderer produces a deterministic artifact for a given report (same input ⇒
   byte-identical output) containing name, species/breed, coarse last-seen location+date,
   and brokered contact.
5. `LocalArtifactSyndicator` writes both artifacts to the configured dir and returns
   `SyndicationOutcome::Written { paths }`; asserted by a tempdir test.
6. `PetFbiPartnerSyndicator` (feature on) returns `SyndicationOutcome::DryRun` when no
   endpoint/credential is configured and **never** returns a success variant in that
   state — asserted by a test. (A real POST path may be exercised against a mock server;
   no live Pet FBI call in tests.)
7. Gated channels appear only as `ManualOnly { channel, reason }` with the documented
   gated-status reason; a test asserts no `Syndicator` impl claims automated delivery to
   PawBoost/Nextdoor/Facebook/Petco.
8. `clippy -D warnings` and `cargo deny check bans licenses sources` pass; the
   `petfbi-partner` feature is off by default and the crate builds both with and without
   it.

## Notes for /build

- Home this in `homeward-report` — it already owns the owner side, EXIF strip, and
  contact brokering. Don't spin a new crate.
- The JSON-LD shape is homeward's to define (no standard exists). Lean on schema.org
  types where they fit and namespace the rest under a homeward IRI; document the mapping
  in the module.
- The cardinal rule for the partner adapter: **no success without a real 2xx.** A lost-pet
  tool that lies about reaching a network is worse than one that says "post this
  yourself." Dry-run is the honest default.
- When/if a Pet FBI partner write endpoint is secured (an outreach, not a build task),
  configuring the endpoint+credential flips `PetFbiPartnerSyndicator` from DryRun to live
  with no code change. That outreach belongs in the vision's open questions, not a PRD.
