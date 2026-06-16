# PRD: lattice-seed — ingest + bridge + join the priority BFO ontology set

Status: Draft v0.1
build_target: shell
Vision: visions/lattice.md
Depends-on: lattice-versioniri (bridge parser fix must ship first)

## TL;DR

The `lattice-*` pipeline (registry → bridge → join → traverse → context → ground)
is fully installed but the federated store is empty — `lattice-registry list` after
sync shows 265 entries, all with `class_count=0` and `cached_path=(empty)`.
`lattice-bridge align` is blocked by the versionIRI bug (PRD-lattice-versioniri).
Once that ships, this PRD drives the priority ontology set through the full pipeline
to produce the first real, queryable federated store at `~/.cache/lattice/store/`.

## Why this exists

Verified live (2026-06-16): `lattice-registry sync` populated 265 OBO Foundry entries;
`lattice-registry add` successfully fetched BFO (35 classes), RO (58, CC0), IAO (266),
COB (69, CC0), SWO (1970) with bfo_grounded=true. The next step — bridge — blocked on
the versionIRI parse error (every OBO ontology header contains `owl:versionIRI`).
Once that is fixed, these five are the priority set for a general AI ops agent:

| Ontology | Classes | License | Relevance |
|----------|---------|---------|-----------|
| BFO 2020 (ISO/IEC 21838-2) | 35 | CC BY 4.0 | Upper anchor — everything else grounds here |
| RO (Relation Ontology) | 58 | CC0 1.0 | Shared relations, used by every OBO ontology |
| IAO (Information Artifact Ontology) | 266 | CC BY 4.0 | Information entities — files, code, specs, plans |
| COB (Core Ontology for Biology) | 69 | CC0 1.0 | Lightweight interoperability bridge |
| SWO (Software Ontology) | 1970 | CC BY 4.0 | Software tools, versions, tasks, licenses — directly relevant |

## What this builds

A shell script `scripts/lattice-seed.sh` (installable to `~/.local/bin/lattice-seed`)
that drives the pipeline:

**Phase 1 — ingest** (idempotent; re-run is safe):
```sh
lattice-registry add http://purl.obolibrary.org/obo/bfo.owl    # BFO 2020 via OBO PURL
lattice-registry add http://purl.obolibrary.org/obo/ro.owl     # RO
lattice-registry add http://purl.obolibrary.org/obo/iao.owl    # IAO
lattice-registry add http://purl.obolibrary.org/obo/cob.owl    # COB
lattice-registry add https://github.com/allysonlister/swo/raw/master/swo.owl  # SWO
```

**Phase 2 — bridge** (pairs with clear semantic relationships):
```sh
# BFO is the anchor — bridge each domain ontology against BFO
lattice-bridge align --a iao_path --b bfo_path --out ~/.cache/lattice/bridges/iao-bfo.owl \
  --proposals ~/.cache/lattice/bridges/iao-bfo.proposals.jsonl --threshold 0.75
lattice-bridge align --a swo_path --b bfo_path --out ~/.cache/lattice/bridges/swo-bfo.owl \
  --proposals ~/.cache/lattice/bridges/swo-bfo.proposals.jsonl --threshold 0.75
# SWO↔IAO: software ontology ↔ information artifacts (strong overlap: software IS an info artifact)
lattice-bridge align --a swo_path --b iao_path --out ~/.cache/lattice/bridges/swo-iao.owl \
  --proposals ~/.cache/lattice/bridges/swo-iao.proposals.jsonl --threshold 0.75
# RO is a relation ontology — bridge it against BFO properties
lattice-bridge align --a ro_path --b bfo_path --out ~/.cache/lattice/bridges/ro-bfo.owl \
  --proposals ~/.cache/lattice/bridges/ro-bfo.proposals.jsonl --threshold 0.75
```
Paths are resolved via `lattice-registry path <id>`. If bridge exits with "no mappings
above threshold", log a note to `~/.cache/lattice/bridges/seed.log` and continue (not
an error — sparse bridges are expected for distant domains).

**Phase 3 — join**:
```sh
lattice-join build \
  --base ~/.cache/lattice/registry/owls/bfo.owl \
  --add ~/.cache/lattice/registry/owls/ro.owl \
  --add ~/.cache/lattice/registry/owls/iao.owl \
  --add ~/.cache/lattice/registry/owls/cob.owl \
  --add ~/.cache/lattice/registry/owls/swo.owl \
  --bridge ~/.cache/lattice/bridges/iao-bfo.owl \
  --bridge ~/.cache/lattice/bridges/swo-bfo.owl \
  --bridge ~/.cache/lattice/bridges/swo-iao.owl \
  --bridge ~/.cache/lattice/bridges/ro-bfo.owl \
  --store ~/.cache/lattice/store/ \
  --allow-unreviewed
```
`--allow-unreviewed` is intentional on the first seed run: the proposals workflow
(review + accept/reject) is a future gate; the seed pass demonstrates connectivity.

**Phase 4 — smoke-test** (script exits non-zero if either fails):
```sh
lattice-join stats ~/.cache/lattice/store/   # confirm non-zero triple count
lattice-traverse path --from "http://www.ebi.ac.uk/swo/SWO_0000001" \
                       --to "http://purl.obolibrary.org/obo/BFO_0000001"  # software → entity
lattice-context get "software tool"
```

The script emits a one-line summary to stdout:
`lattice-seed: <N_triples> triples in store; bridge pairs=<N>; traverse smoke=OK`

## Acceptance criteria

1. `lattice-seed.sh` runs to completion without error (all four phases).
2. `lattice-registry list | grep -E 'bfo|ro|iao|cob|swo'` shows class_count > 0 and
   bfo_grounded=true for all five.
3. `lattice-join stats ~/.cache/lattice/store/` reports ≥ 2000 triples.
4. `lattice-traverse path --from <SWO:software> --to <BFO:continuant>` exits 0 and
   prints a path of ≥1 hop (confirming bridge axioms are in the store).
5. `lattice-context get "information artifact"` exits 0 and returns ≥1 result.
6. Bridge proposals files are written to `~/.cache/lattice/bridges/*.proposals.jsonl`
   (even if empty), so the review workflow is wired for a future pass.
7. Script is idempotent: re-running it does not fail if the store already exists.
