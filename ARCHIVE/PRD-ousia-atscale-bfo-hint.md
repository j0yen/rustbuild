# PRD: ousia-atscale-bfo-hint — honor the per-column BFO override the README promises

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-atscale
Vision: visions/ousia-atscale.md

## TL;DR

The `ousia-atscale` README (written 2026-06-16, when the repo went public) documents:

> You can override [the Quality-vs-Role heuristic] by setting `"bfo_hint": "quality"`
> or `"bfo_hint": "role"` in the column JSON.

The mapper (`src/mapper.rs`) does **not** read `bfo_hint`. The documented feature
does not exist. This PRD implements it — a small, honest fix that also gives
colleagues control over the one genuinely ambiguous mapping (dimension → Quality vs
Role is a name-substring heuristic that will misfire on their real models).

## Why this exists

Verified 2026-06-16 by reading `src/mapper.rs`: the dimension/attribute mapping uses
a name-substring heuristic ("status"/"type"/"category" → Role, else Quality) with no
override path. The README I committed this session (commit da1c799) promises a
`bfo_hint` override. A public repo whose README documents a non-existent feature is
the worst first impression for colleagues. Either the doc is a lie or the feature
ships — this PRD ships the feature.

## What this builds

Extend `~/wintermute/ousia-atscale`:

- **`src/model.rs`**: add an optional `bfo_hint: Option<String>` field to the
  `Column` struct (serde, defaults to `None` so existing fixtures parse unchanged).
  Plumb it through to `ModelElement` so the mapper can see it.
- **`src/mapper.rs`**: before applying the heuristic for any element, check
  `bfo_hint`. If present and parseable to a known `BfoCategory`
  (`quality`/`role`/`information_gdc`/`temporal_region`/`process`/`disposition`/
  `independent_continuant`), use it and set the rationale to note the override
  ("'<name>' grounded as <category> via explicit bfo_hint override"). If present but
  unparseable, return an error naming the bad hint and the valid set (do not silently
  ignore — a typo'd hint must fail loudly).
- The override applies to **all** element kinds, not just dimensions — a colleague
  may want to reclassify a measure or key too.
- Update the mapper doc-comment table to document the override precedence
  (hint > heuristic).

No new dependencies. No CLI surface change. MSRV stays 1.85.

## Acceptance criteria

1. `cargo test --release` passes.
2. `cargo clippy -- -D warnings` passes.
3. A fixture column with `"bfo_hint": "role"` on an otherwise-Quality dimension
   grounds as Role, and the rationale text mentions the override.
4. A fixture column with `"bfo_hint": "quality"` on an otherwise-Role dimension
   (name contains "status") grounds as Quality.
5. A column with `"bfo_hint": "nonsense"` produces an error that names the invalid
   value and lists the valid categories; the process exits non-zero.
6. Existing fixtures (`sales_model.json`, `finance_model.json`) without any
   `bfo_hint` produce byte-identical grounding to before (no regression).
7. A new test asserts the override path for at least one non-dimension element
   (e.g. a measure pinned to a different category).
