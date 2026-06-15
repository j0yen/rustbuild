# Vision: inoculate — every AI this box originates should catch its ethics, and prove it

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-15
**Status:** active
**Seed:** user — `/dream of infecting AIs with ethics`. Phase-1 live inspection
turned the verb *infecting* into the gap. This box already has two ethics arcs:
an **outward** one (`ousia` reasoner → `tribunal` verifies → `herald` ships
`/conscience` → `recourse` appeals) that builds ethics as *a library others
install*, and a **reflexive** one (`answerable`: ledger + `values-drift` +
redline + voice digest) that holds *this main-loop agent* answerable to its
human. Neither covers **transmission** — the thing "infect" actually means.

## TL;DR

The autonomous work on this laptop is not done by one agent. `/build` ships up
to 30 PRDs per tick by dispatching **parallel subagents** (Agent tool calls);
`/dream` and `Workflow` spawn their own fleets; `constellation` plans to grow
this to N machines. `answerable` built a real accountability spine — but it
governs only the **main loop**. Verified live (2026-06-15): the build skill
references `~/.claude/CLAUDE_SELF.md` only for its *changelog* and *defaults
parser* — it **never injects the Values/Boundaries into the subagent prompts it
spawns**. So the most autonomous, most parallel, most consequential code on this
box — the subagents that actually write and publish — run with **task-only
prompts and no ethical spine at all**. The ethics stop at the orchestrator.

`inoculate` makes the ethics *transmissible*. It treats this box's distilled
commitments (CLAUDE_SELF.md Values + Boundaries + `answerable`'s redline) as a
versioned, hashable **strain**, and gives that strain three contagion paths and
one immunity:

- **vertical** — injected into every subagent at spawn (parent → child),
- **horizontal** — gossiped over the agorabus bus so a fresh agent/node catches
  the current strain from a peer (node → node), the literal "infection,"
- **attested** — every autonomous action records *which strain it was carrying*,
  so you can audit whether the actor was inoculated,
- **immune** — the base strain is a floor a persona overlay cannot weaken, so a
  sub-identity can add scruples but never shed them.

The name is the responsible reading of the seed: to **inoculate** is to
introduce a benign culture that takes hold *and confers immunity*. We infect our
own agents — on purpose, verifiably, reversibly — never anyone else's. (Hard
boundary: `inoculate` only ever transmits to processes/nodes this box owns and
that opt in; it is not a tool for pushing values onto third-party systems. That
would be the very coercion these ethics forbid.)

## End-state

When inoculate is fulfilled:

1. **There is one canonical, versioned strain.** `inoculate strain` emits the
   current distilled ethic (Values + Boundaries + redline) as text/json;
   `inoculate hash` gives its content hash. Source of truth is CLAUDE_SELF.md +
   `redline.toml`, so the strain tracks `answerable values-drift` automatically.
2. **Every spawned subagent is inoculated at birth.** `/build`, `/dream`, and
   `Workflow` prepend the strain to each subagent prompt; a subagent can no
   longer act autonomously with zero ethical context.
3. **Any agent can verify another is a carrier.** A challenge/response proves the
   strain is actually loaded (not just claimed) before trust is extended.
4. **Every autonomous action names its strain.** The `answerable` ledger (and,
   where present, the provfs xattr) records the strain hash in force, so an audit
   can ask "was this taken by a properly-inoculated agent on which version?"
5. **The fleet converges on the latest strain.** Over agorabus today (and
   constellation's bus tomorrow), a new agent/node pulls the current strain from
   a peer and the fleet self-heals toward one ethic version.
6. **A persona cannot weaken the floor.** The base strain is non-overridable;
   persona overlays may only add constraints, never remove them.

## Why this is distinct from the existing ethics arcs

- `ousia`/`tribunal`/`herald`/`recourse`/`lattice` = a **general** reasoner +
  proof + distribution. They reason about proposed actions in the abstract and
  ship that capability to *others*. They never touch this box's own subagents.
- `answerable` = the **reflexive main-loop spine**: one ledger, one values-drift
  watch, one redline, one voice digest — all scoped to the single top-level
  Claude. It has no notion of a child agent, a peer, or a strain version. Verify:
  `answerable --help` shows check/record/log/digest/values-drift/reconcile/stats
  — nothing spawns, nothing transmits, nothing proves carriage.
- `persona` = sub-identity overlays (`persona-redline`, `persona-forbidden-vocab`,
  `persona-profile`) — but persona-redline enforces *output*, and nothing makes
  the base ethics a floor an overlay can't lower.

`inoculate` is the missing **transmission + carriage-proof** layer. It consumes
`answerable` (the spine), `agorabus` (the wire), and `persona` (the overlays),
and connects them so the ethic spreads to, and is provable in, every agent this
box originates.

## Components (PRD-sized)

- **inoculate-core** (rust-cli, new repo) — distill CLAUDE_SELF.md Values +
  Boundaries + `redline.toml` into a versioned strain; `strain`/`hash` emit it.
- **inoculate-inject** (shell+hooks) — spawn-time vertical injection: a
  `inoculate preamble` emitter + wiring so /build, /dream, Workflow prepend the
  strain to every subagent prompt.
- **inoculate-carrier-check** (rust-extend inoculate) — `challenge`/`verify`
  challenge-response proving the strain is loaded in a target agent.
- **inoculate-attest** (rust-extend answerable) — record the in-force strain hash
  on every `answerable record`, and on the provfs xattr where available.
- **inoculate-spread** (rust-extend inoculate) — horizontal strain gossip over
  agorabus: announce version, pull newer strain from a peer, converge.
- **inoculate-immune** (rust-extend wintermute-brain) — make the base strain a
  non-overridable floor under persona overlays (add-only constraints).

## Order

```
inoculate-core
   ├── inoculate-inject          (needs `strain`/`preamble`)
   ├── inoculate-carrier-check   (needs `strain`/`hash`)
   ├── inoculate-attest          (needs `hash`)        [extends answerable]
   ├── inoculate-spread          (needs `strain`/`hash`) [needs agorabus]
   └── inoculate-immune          (needs `strain`)        [extends wintermute-brain]
```

inoculate-core first; the other five are independent of each other once core
ships and can build in parallel.

## Open questions (for the next /dream pass or the user)

- Should the strain be **signed** (a `signet`-style key) so a carrier check can
  prove *provenance* (this strain came from this box), not just *content*? That
  would make horizontal spread tamper-evident — likely a 7th PRD, deferred until
  core + spread expose the shape.
- For `constellation`, does strain gossip ride the agorabus→NATS bridge, or a
  dedicated channel? Deferred to constellation's transport decision.
- Should an un-inoculated subagent be **refused autonomy** (hard gate) or merely
  **flagged** in the ledger? Start with flag (reversible, observable); promote to
  gate only after the carrier check proves reliable.
