# PRD: newyorker-cover — a generative cover/identity for each issue, seeded from its own vocabulary

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/new-yorker`
**Vision:** visions/new-yorker.md

## TL;DR

`newyorker-issue` binds the columns into a numbered, dated issue, but every issue
looks identical — a wall of Markdown with the same masthead. A periodical's cover
is part of its identity (*The New Yorker*'s cover art is as famous as its prose).
This PRD gives each issue a **generative cover**: a deterministic SVG/ASCII motif
seeded from the issue's *own* vocabulary (the most-frequent words of that issue's
columns drive a geometric/typographic pattern), stamped with the masthead folio.
No two issues look alike; the same issue always renders the same cover.

## Why this exists

`visions/new-yorker.md` lists the generative cover as the fourth component and
notes it "could reuse `~/wintermute/ambient` or a simple SVG/ASCII generative
motif from the issue's vocabulary" — this PRD takes the **vocabulary-seeded
deterministic** route (the umbrella's grammar-over-API default). `~/wintermute/
ambient/` is the existing precedent that wintermute turns structured signal into
a generated artifact (telemetry → sound); here it is issue-vocabulary →
visual motif. `~/wintermute/daily-receipt/`'s glyph renderer is the direct
prior art for **deterministic-from-a-u64-seed** generative art on this laptop
(its AC6: same seed → identical bitmap, different seeds → different bitmaps) —
the cover follows that same seed-determinism contract. The folio stamped on the
cover comes from `newyorker-masthead`.

## What this builds

A binary crate `newyorker-cover` (binary `cover`) in the `~/wintermute/new-yorker`
workspace. Depends path-locally on `newyorker-masthead` (for the folio block).

Modules:
- `vocab` — derive the seed from issue contents. Read the issue's `issue.json`
  (or raw text via `--text`), tokenize, drop a small built-in stop-word list,
  rank by frequency, take the top-K words. Hash the ordered top-K + folio into a
  stable `u64` seed (deterministic hasher, e.g. a fixed-seed FNV/`DefaultHasher`
  with explicit seeding — no `RandomState`).
- `motif` — a deterministic generative model: from the `u64` seed drive a
  parametric geometric/typographic motif (a grid of glyphs / concentric strokes
  / a halftone field — the daily-receipt glyph approach generalized). The top
  vocabulary words are laid into the motif as typographic elements. All
  randomness comes from a seeded PRNG (`rand` + `StdRng::seed_from_u64`), never
  the OS RNG.
- `render_svg` — emit the motif as a self-contained SVG document (fixed canvas,
  e.g. 800×1000), with the masthead title + folio typeset at the top.
- `render_ascii` — emit the motif as an ASCII/Unicode block-art cover to stdout
  (for terminal preview and for embedding atop `issue.md`).

Deps: `newyorker-masthead` (path), `rand` (StdRng only), `serde_json`, `clap`,
`thiserror`. No network, no clock, no OS entropy.

CLI subcommands:
- `cover make --issue <issues/NNNN dir | issue.json> [--top-k <N>] [--format
  svg|ascii] [--out <path>]` — derive the seed from the issue's vocabulary and
  render the cover; `svg` defaults to a file, `ascii` to stdout.
- `cover make --text <file> --folio "<folio string>"` — render a cover from raw
  text + an explicit folio (decouples cover from the issue tree, for testing).
- `cover seed --issue <...>` — print the derived `u64` seed and the top-K words
  (inspection / determinism debugging).

## Acceptance criteria

1. `cover make --issue <fixture issue.json> --format svg --out c.svg` writes a
   well-formed SVG (root `<svg>` element, non-empty body) that contains the
   issue's folio string and the masthead title, and exits 0.
2. `cover make --format ascii` prints a non-empty multi-line ASCII motif to
   stdout containing the folio line.
3. **Seed determinism** (daily-receipt AC6 contract): the same issue input
   yields the same `u64` seed across runs (`cover seed` is stable), and the same
   seed yields **byte-identical** SVG and ASCII output across two runs (seeded
   PRNG, no `RandomState`/OS entropy).
4. **Seed sensitivity**: two issues with different top-K vocabularies produce
   different seeds and visibly different covers (different SVG byte content);
   verified by a fixture pair whose vocabularies differ.
5. The seed is driven by **vocabulary**: changing one high-frequency word in the
   input changes the derived seed (`cover seed` differs), proving the motif is a
   function of the issue's contents, not a constant.
6. The stop-word filter works: common words (`the`, `and`, `of`, `a`) are not
   among the top-K reported by `cover seed` for ordinary English input.
7. A missing issue file or empty/whitespace-only text exits non-zero with a
   `thiserror` message (no panic, no zero-seed silent cover).
8. `cargo test` passes and `cargo build --release` produces the `cover` binary.
