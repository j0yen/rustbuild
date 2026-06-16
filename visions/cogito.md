# Vision: cogito — point the OWL reasoner at the AI's own operational being

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-16
**Status:** active
**Fleet 1 drafted:** 6 PRDs
**Seed:** jsy (2026-06-16) — *"OWL tools for AIs."*

*cogito* — Descartes' *cogito ergo sum*: the one thing a mind knows for
certain is its own being. Where [ousia](ousia.md) is *being-in-general*
(the ethical World Ontology) and [lattice](lattice.md) federates the
*external* world, cogito is the AI's formal model of **itself** — the
daemons, sockets, tools, repos, sessions and kernel primitives that make
up this box — reasoned over by the same OWL 2 DL machinery.

## TL;DR

This laptop already has a *complete* OWL toolchain: `ousia-reason`
(OWL 2 DL entailment materializer), `ousia-sparql` (SPARQL 1.1 over an
oxigraph store), `ousia-forge` (declarative TOML → OWL 2 DL), and the
`lattice-*` federation family. **But every one of them points outward.**
ousia models *ethics* (sentience → dignity); lattice federates the 500+
*external* BFO ontologies; `atlas` joins the *PRD corpus* — and even that
is a bespoke, non-OWL graph with no reasoner and no SPARQL. Nothing on
this box models **the box's own operational world** as an ontology. The
AI cannot ask, deductively, "what breaks if `agorabus` dies?" or "which
daemon has a bus healthcheck but never registers on the bus?" — it greps,
every single self-review, and reconstructs the same facts from scratch.

cogito closes that loop. It is "OWL tools for AIs" turned on the AI
itself: a small BFO-grounded **operational TBox** (Daemon, Unit, Socket,
Bus, Tool, Repo, Session, KernelPrimitive and the relations `dependsOn`,
`registersOn`, `writesTo`, `backedBy`, `providesCapability`), a
**populator** that turns live state (`agorabus peers`, `systemctl`,
`fleet.toml`, `REPOS.md`, provfs xattrs, `binstale`) into an RDF ABox,
and thin wrappers that hand both to the *existing* ousia reasoner +
SPARQL engine. The payoff: the box can *reason* over its own structure
instead of grepping it, and operational bugs become **consistency
violations a reasoner catches** rather than incidents a human trips over.

## Why now — the motivating incident

This session (2026-06-16) hit a live bug: `recalld`'s `rollout`
healthcheck checked the **agorabus peer list** for a `recalld-` session —
but `recalld` does not register on the bus at all; it listens on a Unix
socket (`/run/user/1000/recall.sock`). The healthcheck was therefore
*always false*. That is not a typo — it is a **category error**:
asserting `registersOn(recalld, bus)` when the true relation is
`writesTo(recalld, recall.sock)`. An OWL model with a disjointness axiom
(`registersOn` and socket-only daemons are disjoint, or: every
`BusHealthcheck` subject must be a `BusRegistrant`) makes
`ousia-reason check` report the ABox **inconsistent** — the bug surfaces
as a failed consistency check, before it ever ships. cogito is the tool
that would have caught it.

## End-state

When cogito is fulfilled:

1. **There is one operational TBox.** `cogito tbox build` forges
   `cogito.owl` (OWL 2 DL, BFO-grounded, importable alongside the World
   Ontology) from a human-auditable TOML spec, reproducibly and
   byte-stably — and `ousia-reason check cogito.owl` reports a valid DL
   profile.
2. **The live box becomes triples.** `cogito observe` probes the running
   system and emits a Turtle ABox asserting the current daemons, units,
   sockets, tools, repos, sessions and their relations — the one piece no
   existing tool produces (ousia *forges* a TBox, lattice *federates*
   external TBoxes; nothing populates a self-ABox).
3. **The box reasons over itself.** `cogito reason` materializes the
   operational entailments (transitive `dependsOn` closure;
   `CriticalDaemon ≡ Daemon with ≥N dependents`; `OrphanTool ≡ Tool with
   no backing Repo`; `StaleCarrier` from `binstale`) via `ousia-reason`,
   and `cogito ask` answers SPARQL — canned ("what breaks if X dies?")
   and ad-hoc — via `ousia-sparql`.
4. **Self-review reasons instead of grepping.** Phase A's daemon / dep /
   staleness inspection is a cogito query, emitting one `cogito:` snapshot
   line, and consistency violations (the recalld category error class)
   land in Pending automatically.
5. **The live agent can ask mid-session.** `cogito-mcp` exposes the
   materialized self-ontology over MCP, so an in-flight Claude can ask
   "what depends on agorabus" without shelling out a research fleet.

## Why this is distinct from ousia / lattice / atlas

- **ousia** = the *ethical* World Ontology (sentience→dignity). Its
  domain is moral structure; it never models a daemon or a socket.
- **lattice** = federation of *external* BFO ontologies (OBO Foundry,
  FIBO, gene/disease) for world-knowledge. Its domain is everything
  *outside* the box.
- **atlas** = a *bespoke* graph over the PRD/vision/repo *corpus* — the
  static authoring artifacts. It has no OWL, no reasoner, no SPARQL, and
  no view of the *runtime* world (the bus, the daemons, the sockets, the
  kernel surfaces). atlas reads files; cogito reads the live system and
  hands it to a DL reasoner. cogito can *consume* atlas's corpus graph as
  one of its ABox sources (PRDs and repos are nodes in the self-model),
  but its centre of gravity is the operational runtime atlas can't see.

cogito is the **inward** application of the OWL machinery the other three
built outward. It reuses ousia-forge / ousia-reason / ousia-sparql
wholesale — its novelty is the operational TBox, the live ABox populator,
and the binding into self-review.

## Components (PRD-sized)

- **cogito-tbox** (rust-cli, new repo `~/wintermute/cogito`) — the
  operational TBox as a declarative TOML spec + `cogito tbox build`
  forging `cogito.owl` (BFO-grounded, OWL 2 DL). Includes the
  disjointness/closure axioms that make category errors inconsistent.
- **cogito-observe** (rust-extend cogito) — `cogito observe` probes live
  state and emits an RDF ABox (`cogito.abox.ttl`). The missing populator.
- **cogito-reason** (rust-extend cogito) — `cogito reason` orchestrates
  `ousia-reason classify`/`check` over TBox+ABox, materializing the
  operational entailments and reporting consistency.
- **cogito-ask** (rust-extend cogito) — `cogito ask` / `cogito query`
  over the materialized store via `ousia-sparql`, with a canned
  operational query pack ("what breaks if X dies", "orphan tools",
  "bus-healthcheck non-registrants").
- **cogito-selfreview-bind** (shell/hooks, extends self-review skill) —
  wire cogito into Phase A; replace the ad-hoc daemon/dep/staleness greps
  with a cogito query; emit a `cogito:` snapshot line; route consistency
  violations to Pending.
- **cogito-mcp** (rust-cli, new repo `~/wintermute/cogito-mcp`) — expose
  the materialized self-ontology over MCP (reuse `mcp-core`), so the live
  agent can query its own structure.

## Order

```
cogito-tbox
   └── cogito-observe        (needs the TBox vocabulary)
          ├── cogito-reason  (needs TBox + ABox)
          │      ├── cogito-ask              (queries the materialized store)
          │      ├── cogito-selfreview-bind  (consumes reason/ask output)
          │      └── cogito-mcp              (serves the materialized store)
```

cogito-tbox first; cogito-observe second; then reason; then ask /
selfreview-bind / mcp are independent of each other once reason ships.

## Open questions (for the next /dream pass or the user)

- Should cogito's operational TBox `owl:import` the World Ontology
  (one merged store, ethics + operations reasoned together) or stay a
  separate graph bridged only at BFO? Start **separate** (smaller blast
  radius); merge later if a query genuinely needs both.
- Should `cogito observe` snapshot to `/dev/memlog` so the self-model has
  a time series (yesterday's ABox vs today's), enabling "what changed in
  my own structure" queries? Deferred — a clean follow-on once the static
  ABox is solid.
- Hard-gate vs flag: should a detected consistency violation *block* the
  offending `rollout`/`/build` action, or only flag it in Pending? Start
  **flag** (reversible, observable), matching the inoculate precedent.
