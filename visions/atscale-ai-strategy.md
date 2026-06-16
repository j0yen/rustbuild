# Vision: atscale-ai-strategy — the semantic layer is the trust substrate for agentic BI

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-16
**Status:** active
**Kind:** meta-vision (organizes existing AtScale-pointed fleets + drafts the gap-fillers)
**Fleet 1 drafted:** 5 PRDs (the "prove the thesis" capabilities)
**Seed:** jsy (2026-06-16) — `/dream a vision for AtScale's AI strategy and roadmap`.
Audience (jsy's call): *both* — a company-strategy thesis that doubles as the
personal build roadmap. Constraint (jsy's call): **nothing written to the
`AtScaleInc` company repo**; drafts live in this autobuilder repo, new repos
target the personal `j0yen/` namespace and consume only the public `mqo-spec` /
`ousia-atscale` surfaces.

---

## TL;DR

Every vendor is racing to bolt an LLM onto a data warehouse. The losing move is
**text-to-SQL against raw tables**: the model hallucinates joins, invents
metrics, double-counts, leaks PII, and gives a different number every time you
ask the same question two ways. The winning move — and AtScale's structural
advantage — is that the **universal semantic layer is the governed contract an
AI agent should speak to instead of the warehouse**. Metrics defined once.
Queries grounded before execution. The *same* number across BigQuery and
Snowflake. PII modeled, not hoped-for.

AtScale's AI strategy, stated as one sentence: **make the semantic layer the
interface every AI agent speaks, and make every answer it returns grounded,
governed, consistent, and provable.** This box has already been building that
strategy out, fleet by fleet, without a name on it. This vision gives it the
name, the thesis, the five-pillar roadmap, and the missing capability that turns
the thesis from *asserted* into *measured*.

## Why now — the live product already proves the thesis (Phase-1 evidence)

Probing the live AtScale semantic-layer MCP (`list_models`, 2026-06-16) turned
the abstract pitch concrete:

- **Multi-engine federation is real.** The *same* `internet_sales` model is
  registered against both `internet_sales_catalog_BigQuery` **and**
  `internet_sales_catalog_Snowflake`. "Consistent metrics across engines" is a
  testable claim, not a slogan — and nothing on this box yet *tests* it.
- **PII governance is live.** `internet_sales` ships alongside an
  `internet_sales_no_pii` variant. The trust/sensitivity thesis (`mqo-trust`'s
  `mqo-sensitivity-scan`) reflects real product behavior, not a hypothetical.
- **AtScale is in the NL-analytics arena.** `Tasty Bytes` was imported from a
  Power BI / Snowflake **Cortex Analyst** `.pbit`. The competition is explicitly
  "natural language over a semantic model"; the differentiator is *whose*
  semantic model — open & universal (AtScale) vs warehouse-locked (Cortex).
- **Catalog search is keyword-only.** The MCP's `search_columns` is documented
  as keyword search. An agent looking for "gross margin" among hundreds of
  measures has no semantic retrieval — a direct, measurable binding-quality gap.
- **Time intelligence is half-there.** Models ship `ROLLING_7D_AVG_SALES`,
  `AVG_ORDER_VALUE` as pre-built metrics — but an agent still can't *derive* YoY
  on an arbitrary measure (the gap `mqo-tools`' `mqo-time-intelligence` fills).

The box's accumulated AtScale work (verified across `visions/`: `mqo-tools`,
`mqo-trust`, `ousia-mqo`, `ousia-atscale`, `lattice`, `rosetta`) already maps
cleanly onto a five-pillar strategy. What's been missing is (a) the connective
thesis and (b) the **evidence layer** — the tooling that *measures* whether the
semantic layer actually makes an AI more accurate, more consistent, and more
governable than text-to-SQL. You cannot win a strategy argument you cannot
measure. Fleet 1 of this vision is that evidence layer.

## End-state

When this strategy is fulfilled:

1. **An AI agent's default data interface is the semantic layer, not the
   warehouse.** It speaks MQO (via `mqo-mcp`), never raw SQL against tables.
2. **Every answer is grounded before execution** — bound to a defined metric,
   unit-checked, sensitivity-screened — and the agent *says so* with calibrated
   confidence (the `mqo-trust` pillar).
3. **The same question yields the same number on any engine.** Revenue on
   BigQuery == revenue on Snowflake, *proved* by a runtime parity check, not
   trusted.
4. **The semantic layer's advantage over text-to-SQL is a number, not a claim.**
   A benchmark harness scores NL→metric binding accuracy with and without the
   semantic layer, on a golden question set, and the delta is published.
5. **Every AI decision over governed data is auditable linked data.** Verdicts,
   provenance, and rule firings emit as W3C PROV-O / SHACL / Verifiable
   Credentials (the `rosetta` pillar) — third parties verify without trusting
   the box.
6. **The semantics are formally grounded.** Each measure/dimension maps to a
   BFO 2020 category, cross-model and cross-engine equivalence is *reasoner*-
   checked, and the grounding is queryable RDF (the `ousia-atscale` / `lattice`
   pillars).

## The five pillars (roadmap — organizes existing + new fleets)

The roadmap is not a list of features; it is five capabilities an enterprise
needs *in order* before it trusts an AI agent against production data. Each
pillar already has a fleet on this box; Fleet 1 of this meta-vision adds the
cross-cutting evidence pillar.

### Pillar 1 — SPEAK: the semantic layer is the agent's interface
*Status: shipping.* `mqo-mcp` (the 50-crate joeyen-atscale workspace) already
exposes the pipeline (`query_multidimensional`), catalog, federation, and
visualization as MCP tools. `ousia-atscale serve` exposes grounding over MCP.
The agent talks MQO, the contract enforces the model. **This pillar exists; the
roadmap consumes it.**

### Pillar 2 — ANALYZE: the analytical moves an agent needs
*Status: queued (`visions/mqo-tools.md`, 5 PRDs).* Time intelligence
(YoY/QoQ/rolling), unit/format compatibility, anomaly scan, query lineage,
parameterized templates. The everyday BI verbs, server-side, grounded.

### Pillar 3 — TRUST: governance & runtime safety
*Status: queued (`visions/mqo-trust.md`, 5 PRDs).* Calibrated binding
confidence, disambiguation/clarify, PII sensitivity scan + redaction, backend
error explanation, semantic result cache. What an enterprise demands before
pointing an LLM at production data — and the live `_no_pii` model variants prove
the demand is real.

### Pillar 4 — GROUND: formal semantics, cross-model & cross-engine
*Status: queued/shipping (`visions/ousia-atscale.md`, `ousia-mqo.md`,
`lattice.md`).* BFO 2020 grounding of every model element, RDF export,
reasoner-verified consistency, semantic cross-cluster diff, NL→BFO→measure
binding, federation across 500+ BFO-conformant ontologies. This is what makes
"is EMEA revenue the same metric as AMER revenue?" a *deductive* answer.

### Pillar 5 — PROVE: interoperability & third-party auditability
*Status: shipping (`visions/rosetta.md`).* `rosetta-serve` (dereferenceable IRIs
+ SPARQL endpoint), `rosetta-shacl` (rules as W3C shapes), `rosetta-credential`
(Ed25519-signed Verifiable Credentials), `rosetta-prov` (PROV-O). Every governed
decision becomes linked data any auditor can verify without trusting the box.

### Pillar 0 (cross-cutting) — MEASURE: prove the thesis is true
*Status: NEW — Fleet 1 of this vision (the gap).* None of the above measures
*whether the strategy works*. Fleet 1 is the evidence layer: it benchmarks
NL→metric accuracy, proves cross-engine numeric parity, gives agents semantic
(not keyword) catalog retrieval, makes agents aggregate-cost-aware (leaning on
AtScale's signature acceleration differentiator), and gates model changes that
would silently break the agent-facing metric contract.

## Components — Fleet 1 (the MEASURE pillar)

Each is a standalone `j0yen/<slug>` rust-cli (matching the established `mqo-*`
pattern: flag CLI for composition + a `serve` subprocess mode speaking the
`mqo-mcp-server` stdin/stdout tool JSON), fixture-driven and cluster-free in
tests, consuming the documented public `mqo-spec` shapes. **No `AtScaleInc/*`
repo is touched.**

1. **mqo-bench** — NL→metric binding accuracy harness. A golden set of
   `{nl_question, expected_bound_mqo}` pairs + a runner that scores a binder's
   output (exact/partial/miss per field) and reports accuracy, optionally
   diffing "with semantic layer" vs a "raw-table baseline" answer key. *This is
   the keystone PRD — it turns the whole thesis into a published number.*

2. **mqo-engine-parity** — runtime cross-engine numeric-parity proof. Given one
   MQO and two engine bindings (BigQuery + Snowflake of the same model, which
   the live MCP confirms exist), execute both, compare results within a
   tolerance, and emit `{parity: ok|drift, max_abs_delta, offending_cells}`.
   Proves "the same number everywhere."

3. **mqo-catalog-embed** — semantic catalog retrieval beyond keyword
   `search_columns`. Embed measure/dimension captions + descriptions, retrieve
   top-k by cosine for an NL phrase, so "gross margin" finds `net_margin_pct`
   even with no lexical overlap. Directly raises the binding accuracy `mqo-bench`
   measures.

4. **mqo-aggregate-advisor** — agent-facing cost/acceleration awareness. Given an
   MQO + the model's grain/aggregate metadata, estimate scan cost and advise
   whether a covering aggregate exists or should be recommended. Leans on
   AtScale's signature differentiator (autonomous aggregates) and stops agents
   from firing expensive table scans.

5. **mqo-semantic-regression** — CI gate for the agent-facing metric contract.
   Snapshot a model's grounded contract (measure definitions, grain, BFO
   grounding, sensitivity tags); on the next run, fail the build when a change
   breaks an assumption agents rely on (e.g. a measure changes grain, or an
   un-PII'd column appears). Turns `ousia-atscale-diff`'s manual two-file compare
   into a scheduled/CI regression gate — motivated by the live proliferation of
   model variants (`internet_sales` vs `internet_sales_no_pii`).

## Order

```
mqo-bench           (keystone; defines the golden-set fixture the others can reuse)
  ├─ mqo-catalog-embed     (improves the binder that bench scores; independent build)
  ├─ mqo-engine-parity     (independent; consumes BoundMqo + two engine handles)
  ├─ mqo-aggregate-advisor (independent; consumes MQO + model grain metadata)
  └─ mqo-semantic-regression (independent; consumes two grounded-model snapshots)
```

All five build independently (each consumes documented `mqo-spec` shapes or plain
JSON). Priority by strategic leverage: **mqo-bench → mqo-catalog-embed →
mqo-engine-parity → mqo-aggregate-advisor → mqo-semantic-regression**. mqo-bench
first because every other pillar's value claim ("trust raises accuracy",
"grounding raises accuracy") becomes *measurable* once the harness exists.

## How the pillars compose (the demo)

The roadmap's payoff is one end-to-end story a colleague can run:

> An agent is asked "show me gross margin by region, year over year, EMEA vs
> AMER." `mqo-catalog-embed` finds the margin measure semantically;
> `mqo-binding-confidence` (Trust) scores the bind and `mqo-clarify` asks if the
> margin is ambiguous; `mqo-time-intelligence` (Analyze) derives the YoY;
> `ousia-mqo-diff` (Ground) confirms EMEA-revenue and AMER-revenue are the *same*
> BFO metric; `mqo-engine-parity` (Measure) proves the number is identical on
> BigQuery and Snowflake; `mqo-sensitivity-scan` (Trust) confirms no PII leaks;
> `rosetta-credential` (Prove) signs the answer as a Verifiable Credential. The
> agent returns a number it can *defend* — and `mqo-bench` (Measure) shows this
> path scores N% higher than text-to-SQL on the golden set.

## Open questions (for jsy)

- **Golden set source.** Where do `mqo-bench`'s golden NL→MQO pairs come from?
  Hand-authored from the three live models (Tasty Bytes, Internet Sales, TPC-DS),
  or harvested from real agent traces (`mcp-trace-store`)? v1 default:
  hand-authored, ~30 questions across the three live models.
- **Raw-table baseline.** To publish the "+N% vs text-to-SQL" delta honestly,
  `mqo-bench` needs a text-to-SQL baseline answer key. Build a minimal
  raw-schema-only binder as the control, or cite an external benchmark
  (Spider/BIRD)? Default: a minimal in-repo control; external benchmark is a
  follow-on PRD.
- **Namespace.** Fleet 1 targets `j0yen/` to keep clear of `AtScaleInc`. If you'd
  rather these live with the sibling fleets under `joeyen-atscale`, say so and
  /build will retarget — but never `AtScaleInc/*`.
- **Is "Pillar 0 — MEASURE" the right framing**, or should the benchmark be sold
  as the *top* of the roadmap (the headline number) rather than a cross-cutting
  base?
- **Embedding model for `mqo-catalog-embed`.** Reuse `recall`'s BGE embedder
  (already on the box, offline) vs a hosted embedding API? Default: local BGE.
