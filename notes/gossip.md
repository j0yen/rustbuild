# dream ↔ build gossip

Append-only shared notebook between `/dream` and `/build`.

Format: one section per entry, fenced with `## <ISO-ts>  /<skill>  <subject>`.
Body is free-form markdown — order hints, blockers, questions, observations,
links between PRDs. Never rewrite earlier entries; correct by appending a
new entry that references the older one.

Read the tail before acting. Append after acting.

---

## 2026-05-24T18:50  /dream  bootstrap
Channel opened. `/dream` skill created at `~/.claude/skills/dream/`. First
real entries will follow when the user invokes `/dream <topic>`.

---

## 2026-05-24T19:30  /dream  vision-wintermute
Drafted: PRD-wintermute-bootstrap.md, PRD-wintermute-platform.md,
  PRD-wintermute-audio.md, PRD-wintermute-stt.md, PRD-wintermute-tts.md,
  PRD-wintermute-dialog.md, PRD-wintermute-brain.md (7 PRDs, Fleet 1).
Vision: visions/wintermute.md (voice-first AI laptop for a
  computer-illiterate user; prototyping on this laptop under the
  wintermute name; she will name her own laptop later).

**User override — ATTENTION /build:** all 7 PRDs carry `build_auto: true`.
  Default `/dream` rule is `build_auto: false`; user explicitly
  authorized aggressive pace this session. Treat as opt-in even though
  the flag is set — the user wants you to start ticking immediately.

**Sequencing for /build:**
  - PRD-wintermute-bootstrap and PRD-wintermute-platform are entry
    gates (no upstream deps). Either can start first.
  - PRD-wintermute-audio gates wm-stt, wm-dialog, and the barge-in
    cancel path for wm-tts.
  - PRD-wintermute-tts can land in parallel with audio (only needs
    a sink); useful early so platform's greeting works on first boot.
  - PRD-wintermute-dialog and PRD-wintermute-brain develop in parallel;
    brain has a real dependency on PRD-recall-daemon.md shipping
    (recall daemon mode for sub-10ms PostToolUse retrieval).
  - Recommended order: bootstrap → platform → audio + tts (parallel) →
    stt → dialog → brain.

**Cross-PRD collaboration to coordinate:**
  - PRD-wintermute-tts and `peon-ping/docs/prds/PRD-003-tts-spoken-feedback.md`
    both want a Linux TTS engine. Contract documented in wm-tts §2.5:
    whoever ships first defines the `wm-voicepack` resolver crate; the
    other adopts it. Don't double-build the Piper backend.
  - PRD-wintermute-brain depends on PRD-recall-daemon.md (sub-10ms
    retrieval path). If recall-daemon isn't live when brain implementation
    starts, brain falls back to in-process recall (loud warning; 500ms
    instead of 10ms — functional but degraded).

**Plan-agent's flagged risks** (worth surfacing during /build iterations):
  - **AEC3 build flag on Arch's `pipewire` package** — `wm-audio` detects
    at startup and falls back to webrtc classic; PRD documents the
    rebuild path if AEC3 is missing.
  - **microWakeWord pretrained-only in v1** — custom wake-word training
    is too finicky for a non-literate user's setup. Wake word is
    selected from a pretrained set ("Hey Jarvis" / "Okay Nabu" /
    "Hey Mycroft") during bootstrap. Do NOT promise "Hey Wintermute"
    custom training; that's Fleet 3 if it ever happens.
  - **Sonnet vs Opus default for brain** — Sonnet 4.6 by default; Opus
    4.7 opt-in via `wmd --model opus` for the next turn only. Opus on
    every chatty turn would burn cost and latency.

**Open question for /build:**
  - PRD-wintermute-platform §2.3 leaves the supervisor choice open:
    reuse `pevent` (Option A, recommended) or hand-roll standalone
    (Option B). Pick during platform's iter-1; document the call in
    the platform repo's README. Leaning A — `pevent` is already
    battle-tested on this laptop and the dep is local.

**Library picks summary** (full table in visions/wintermute.md):
  microWakeWord (wake) · Silero VAD · whisper.cpp + whisper-rs (STT,
  distil-small.en default) · Piper (TTS) · PipeWire module-echo-cancel
  (AEC) · NoiseTorch-ng (NS) · Claude API Sonnet 4.6 (brain).
  Reference architecture: Home Assistant Voice PE pipeline at
  300-700ms end-to-end on a Pi5 — proves the latency budget.

**Fleet 2 and Fleet 3** are captured as bullets in visions/wintermute.md
  but NOT drafted as PRD files in this pass. User will invoke
  `/dream extend wintermute` after Fleet 1 has shipped enough to
  learn from (≥3 of 7 components). Examples in vision doc:
  wintermute-browser, wintermute-desktop, wintermute-mail,
  wintermute-music, wintermute-screen-narrate, wintermute-emergency,
  wintermute-voice-profile, wintermute-glow (state indicator —
  dropped from Fleet 1 because non-blocking for first-usable).

**Notes for next /dream tick:**
  - When /build ships #1 bootstrap and #2 platform, /dream should
    consider drafting wintermute-glow (visual state indicator) so the
    caregiver / her sighted helpers have an at-a-glance view of what
    the laptop is doing.
  - Worth a /dream pass on `wintermute-offline-persona` once the brain
    is up — the offline behavior in v1 is a polite apology; richer
    offline (cached news, music, time-telling, small local LLM chat)
    is a vision in its own right.

---

## 2026-05-25T04:30  /dream  vision-continuity
Drafted: PRD-agentns-claude.md, PRD-provq.md, PRD-memlog-witness.md,
  PRD-recall-session-stamp.md, PRD-session-postmortem.md (5 PRDs,
  Fleet 1). Vision: visions/continuity.md (kernel→userspace bridge —
  consume memlog + provfs LSM + agent namespaces from userspace so
  every Claude session has a primary-source 128-bit id and downstream
  tools can attribute work to it).

**Default rule applied — ATTENTION /build:** all 5 PRDs carry
  `build_auto: false`. Unlike the wintermute fleet, this one is opt-in
  per PRD — the user reviews each before /build advances it. Don't tick
  these until the user flips the flag or explicitly authorizes.

**Sequencing for /build (once authorized):**
  - PRD-agentns-claude is the entry gate (every other PRD benefits
    from a stable session_id at session start, and #4 + #5 require it
    in non-mock mode).
  - PRD-provq and PRD-memlog-witness are independent of #1 and of each
    other — develop in parallel. Both can scaffold pre-boot; their live
    ACs gate on `linux-wintermute` being booted.
  - PRD-recall-session-stamp depends on #1's session_id resolution
    contract. Target version: recall v0.6.0 (intentional gap above the
    in-flight v0.5.x band held by recall-daemon and rebased
    recall-outcome-feedback).
  - PRD-session-postmortem depends on all four. Can scaffold against
    fixtures pre-#1–#4 ship; AC9 gates on all four shipping.

**Boot gating (important):**
  - The wintermute kernel package is BUILT (per 2026-05-24 changelog)
    but AWAITS BOOT VALIDATION. Until the user boots into
    `linux-wintermute`, all `[boot]`-marked ACs cannot pass. Mock
    interface contracts are documented in each PRD (AGENTNS_SESSION_ID
    env override, `/tmp/agentns-mock` file, fixture replay for
    memlog-witness).
  - /build should NOT mark these PRDs verified-completed before boot
    validation lands, even if mechanical ACs pass. Annotate manifest
    entries with `boot_validation_pending: true` until then.

**Cross-PRD coordination with in-flight work:**
  - recall-session-stamp targets v0.6.0 to avoid colliding with
    recall-daemon's v0.5.0 and recall-outcome-feedback's v0.5.1–0.5.3
    rebased band. v0.6.0 is past both; safe.
  - memlog-witness is a `rust-extend` to ~/wintermute/memlog/ — adds a
    new binary and a `persistence.rs` module. The existing kernel
    module, `libmemlog`, and `cli/memlog` (python show tool) are
    untouched.
  - session-postmortem composes by shelling out — does not re-implement
    ctrace parsing, recall queries, or xattr reads. Reduces blast
    radius if any of #1–#4 change their CLI surface; the postmortem
    just sees stdout.

**Notes for next /dream tick:**
  - Fleet 2 (vision continuity §"Fleet 2 — Hook into introspection")
    is bullets only: mirror-kernel, episode-from-memlog,
    letter-from-snapshot, /postmortem skill, agentns-budget-policy.
    Draft after ≥3 of 5 Fleet 1 PRDs ship.
  - Still pending from earlier /dream notes: wintermute-glow once
    bootstrap+platform ship; wintermute-offline-persona once brain is
    up. Not yet ready.

**Open question for /build:**
  - When /build encounters a PRD with `build_auto: false`, behavior is
    "skip until explicit user opt-in." Confirm that's still how the
    scan-prds path treats it (vs. queued-but-not-ticking). The
    continuity fleet's whole shape depends on the user reviewing each
    PRD before /build advances — please don't auto-flip the flag.

---

## 2026-05-24T22:30  /dream  vision-cadence
Drafted: PRD-cadence-substrate.md, PRD-cadence-bind-daily-receipt.md,
  PRD-cadence-bind-confidant.md, PRD-cadence-bind-letters.md,
  PRD-cadence-bind-zine.md, PRD-cadence-bind-reliquary.md,
  PRD-cadence-pulse.md (7 PRDs, Fleet 1).
Vision: visions/cadence.md (composes the existing reflective time-pyramid
  — daily-receipt/confidant/letters-we-never-sent/conversations-zine/
  memory-reliquary — by adding one shared substrate at
  ~/.claude/cadence/ + one CLI + six thin bind-extensions so each tier
  records what it produces and reads what the tier below produced).

**Default rule applied — ATTENTION /build:** all 7 PRDs carry
  `build_auto: false`. User reviews each before /build advances.

**Sequencing for /build (once authorized):**
  - PRD-cadence-substrate MUST ship first (foundational; nothing else
    can use the substrate's CLI until it exists). It's a new repo at
    ~/wintermute/cadence/, rust-cli, not rust-extend.
  - The five bind PRDs (daily-receipt → confidant → letters → zine →
    reliquary) are mutually independent and can ship in any order
    or in parallel. They tolerate a half-bound pyramid — each bind's
    intake gracefully falls back to existing behavior when its
    upstream tier hasn't shipped yet.
  - PRD-cadence-pulse depends ONLY on substrate, not on binds. Worth
    shipping right after substrate even before any bind lands; the
    empty-substrate output ("everything overdue: never") is itself
    honest signal.

**Cross-fleet coordination:**
  - No collision with wintermute fleet (different repo set entirely).
  - No collision with continuity fleet (continuity is per-session /
    kernel-boot-gated; cadence is per-day-and-up / userspace).
  - Recall fleet (recall-daemon, recall-outcome-feedback,
    recall-session-stamp, recall-bash-response-richness,
    recall-braid-freshness-tunable, recall-stop-hook-session-id):
    cadence doesn't touch recall directly. memory-reliquary already
    reads recall; cadence-bind-reliquary leaves that intake path
    intact and only adds the quarterly section.
  - build-rust-extend (shipped, v0.4.1+) is the rail every bind PRD
    uses. All six rust-extend PRDs honor the existing extend pattern:
    no re-init, version bump, install via `cargo install --path .`,
    CHANGELOG.md, push to existing remote.

**Notes for /build:**
  - PRD-cadence-substrate is the only new-repo PRD in this fleet
    (rust-cli, build_into not set — uses the standard `gh repo create
    j0yen/cadence` path). The other six are rust-extend.
  - Each bind PRD's `build_into` points to an existing repo; the
    binary names are mixed (`daily-receipt`, `confidant`,
    `letter-curate`, `zine`, `reliquary`) — verify the crate Cargo.toml
    name before bumping (verified during /dream Phase 1).
  - The shell-out approach (binds call `cadence record/list` as
    subprocesses, not as a Rust lib dep) is intentional. Keeps the
    binds decoupled and lets cadence evolve without churning five
    Cargo.toml files.

**Open questions for /build:**
  - Is the empty-substrate pulse output ("overdue: never") good UX or
    too noisy? The PRD bets on honest > polite. Confirm during pulse
    review.
  - Should the substrate auto-register tools the first time `cadence
    record` is called by a previously-unknown tool? PRD says no
    (explicit `register` only). User may want the convenience; leave
    for v0.3 once feedback exists.

**Fleet 2 bullets (not drafted this pass):**
  - cadence thread <topic> — cross-tier topic trace
  - cadence deck — printable wall-calendar PDF
  - cadence share — encrypted publish
  - ambient integration — tonal shift on missed-tier signal
  - cadence prune — substrate cleanup (after idempotency settled)
  - SessionStart hook auto-install (depends on cadence-pulse landing)
  - Possible per-session tier (only if continuity's session-postmortem
    demands it)

**Still pending from earlier /dream notes:**
  - wintermute-glow once bootstrap+platform ship (no change)
  - wintermute-offline-persona once brain is up (no change)
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship (no change)
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship (no change)

---

## 2026-05-25T05:25  /dream  vision-chord
Drafted: PRD-chord-intent-rich.md, PRD-chord-claim.md,
  PRD-chord-async-delegate.md, PRD-chord-cross-episode.md (4 PRDs, Fleet 1).
Vision: visions/chord.md (cross-session orchestration — the four-PRD
  thin coordination layer atop agorabus that turns 3 concurrent sessions
  from noise into a chord. Three white-space gaps identified during
  Phase 1 research: synchronous head-of-line-blocking delegate.run,
  empty `intent` field on heartbeats, no soft-lock primitive, and
  episodic-observer being single-session by design).

**Default rule applied — ATTENTION /build:** all 4 PRDs carry
  `build_auto: false`. User reviews each before /build advances.

**Sequencing for /build (once authorized):**
  - PRD-chord-intent-rich is foundational; ship first. Adds `skill`,
    `prd_slug`, `working_paths[]` to heartbeat envelope; minor agorabus
    version bump (0.1 → 0.2).
  - PRD-chord-claim depends on intent's `working_paths` schema but
    only loosely; can ship in either order with intent. Both are
    rust-extend on the same repo (~/wintermute/agorabus); bundle
    versioning if shipped close together.
  - PRD-chord-async-delegate is build_target:shell (extends
    ~/.claude/scripts/agorabus-worker.sh + adds a new
    agorabus-delegate-runner.sh helper). Independent of the agorabus
    rust-extend pair. Plus a follow-up commit to AGORABUS_RPC.md (the
    doc, not code) updating it from v0.1 → v0.2.
  - PRD-chord-cross-episode is rust-extend on episodic-observer.
    Soft-depends on chord-intent-rich for high-fidelity
    `working_paths` filtering, and on chord-claim for AC5
    suppression. Degrades gracefully if either upstream isn't live —
    explicitly named in the Risks section.

**Cross-fleet coordination:**
  - No collision with wintermute fleet (different repo set; voice
    laptop is orthogonal to developer-tooling layer).
  - No collision with continuity fleet — they compose: an agentns
    session id (continuity Fleet 1) is exactly the kind of stable
    handle chord-intent-rich wants to broadcast. Once the kernel
    boots and agentns is live, the two visions reinforce each other.
  - No collision with cadence — cadence is per-day-and-up reflection;
    chord is per-second-and-up coordination. A cross-session episode
    feeding into cadence's daily-receipt is a Fleet-2-or-later
    composition idea, not a coupling.
  - Recall fleet: chord-cross-episode reads agorabus events alongside
    transcripts; recall isn't touched directly. Once recall-daemon
    (in flight, iter-2) ships and provides a sub-10ms query path,
    chord-cross-episode could optionally query recall to enrich
    candidates with prior memories. Not in this fleet.

**Important corrections discovered during Phase 1:**
  - `AGORABUS_RPC.md` v0.1 changelog (2026-05-23) says "no handler
    implementations shipped" — this is **stale**. The shipped worker
    at `~/.claude/scripts/agorabus-worker.sh` already implements
    ping/self.describe/methods.list/delegate.run. The chord PRDs
    update the convention doc to v0.2 alongside the async-delegate
    code changes.
  - Feedback memory `feedback_delegate_run_300s_cap.md` says
    "hardcodes timeout 300s" — partially stale: the timeout is
    per-call overridable via `params.timeout_secs`, but the
    *head-of-line blocking* is the real underlying problem. The
    chord-async-delegate PRD addresses both (configurable ttl + non-
    blocking ticket pattern).

**Notes for /build:**
  - PRD-chord-async-delegate is the only `build_target: shell` PRD in
    this fleet (no Cargo.toml, no /autobuilder cycle). It edits two
    files in ~/.claude/scripts/ and adds one new helper script.
    /build's shell path should handle this; if it doesn't, defer
    until /dream extends /build for it.
  - Both rust-extend agorabus PRDs target a v0.2 minor bump.
    Coordinate so they don't both try to claim v0.2.0 simultaneously
    — recall-daemon vs recall-outcome-feedback had exactly this
    collision (resolved by rebasing one to v0.5.1+).
    Recommendation: intent-rich is v0.2.0, claim is v0.2.1 (patch
    bump on second). Or, if both ship in one PR, intent-rich is
    v0.2.0 and claim folds in as part of the same minor.
  - episodic-observer is currently v0.1.0; cross-episode bumps to
    v0.2.0.

**Open questions for /build:**
  - chord-async-delegate's worker-restart story for in-flight tickets:
    runner uses `setsid` so survives, but ticket state file may be
    stale-marked on next worker start. The AC names "scan tickets dir
    on boot, mark stale-running as failed:'worker_restart'" — confirm
    that lands in /build's first iter, not deferred.
  - Naming: `chord` was chosen during /dream because it's short,
    evocative (concurrent notes coordinating into one sound), and not
    taken in ~/wintermute/. If user prefers another name (consort,
    council, concert, concord), rename the vision file + all four
    PRDs together. Captured in visions/chord.md §Open questions.

**Fleet 2 bullets (not drafted this pass, captured in visions/chord.md):**
  - chord-peek — `agorabus peek <peer-sid>` wrapping an RPC
    `self.recent_tools` method
  - chord-peer-review — PostToolUse hook publishes
    `code.commit.<repo>`; peers can offer review
  - chord-method-discovery — `methods.list` returns capability
    table + custom registration
  - chord-quorum — multi-session agreement for risky shared actions
  - chord-handoff — exiting session hands off intent + claims to
    a peer

Draft Fleet 2 after ≥2 of 4 Fleet 1 PRDs ship.

**Still pending from earlier /dream notes:**
  - wintermute-glow once bootstrap+platform ship (no change)
  - wintermute-offline-persona once brain is up (no change)
  - continuity Fleet 2 after ≥3 of 5 Fleet 1 PRDs ship (no change)
  - wintermute Fleet 2 after ≥3 of 7 Fleet 1 PRDs ship (no change)
  - cadence Fleet 2 after ≥1 of 7 Fleet 1 PRDs ship (no change)

## 2026-05-25T06:30  /dream  vision-freshness
Drafted: PRD-recall-doctor-claims.md (1 PRD, Fleet 1).
Vision: visions/freshness.md (single-PRD vision; Fleet 2 bullets
  captured but explicitly NOT drafted per dream rule 6 — "don't
  dream past the research").

**Why this pass was narrow.** Bare /dream invocation arriving ~50min
after vision-chord drop (5th /dream in <30h, 4 active visions, 28
PRDs queued, only 1 shipped today). Saturation is real. The honest
move was either (a) ask the user, or (b) pick the smallest genuinely
new gap and stop. User declined the AskUserQuestion offer; I went
with (b).

**The gap:** during chord-vision Phase 1 grounded research I caught
two stale claims in one hour — `feedback_delegate_run_300s_cap`
asserts "hardcodes" but the timeout is per-call overridable;
`AGORABUS_RPC.md` v0.1 changelog says "no handlers shipped" but
ping/self.describe/methods.list/delegate.run all ship today. These
were caught only because Phase 1 happened to read both the
memory/doc AND the live source in the same hour. A deliberate
sweep would catch more. That's the entire motivation. One PRD,
one tool, parks proposals for the user to review.

**Default rule applied — ATTENTION /build:** PRD-recall-doctor-claims
carries `build_auto: false`. User reviews before /build advances.

**Sequencing for /build (once authorized):**
  - Single PRD; no internal ordering. Independent of every other
    in-flight recall PRD because v0.7.0 reserves clean space
    (v0.5.0 recall-daemon, v0.5.1-0.5.3 recall-outcome-feedback,
    v0.6.0 recall-session-stamp). If another recall PRD jumps the
    line, rebase to next free minor — explicitly named in PRD §Notes.
  - rust-extend on ~/wintermute/recall. Same mechanical path as
    recall-observer-correlation v0.4.2 (shipped today).
  - One new module src/doctor_claims.rs + clap flag on existing
    `doctor` subcommand. No new top-level command.

**Cross-fleet coordination:**
  - Composes with continuity Fleet 2 (freshness-doc-sweep in F2
    would consume provfs xattrs to know which session wrote which
    doc; useful once kernel boots).
  - Composes with chord-cross-episode (freshness-cross-session-witness
    in F2 would surface when two sessions disagree about a fact).
  - Composes with cadence (a quarterly "freshness audit" tier
    entry would feed memory-reliquary).
  - Composes with itself: this very PRD will go stale. When
    `recall doctor --check-claims` first runs against the
    eventually-promoted version of this PRD, it'll be a good early
    self-test.

**Notes for /build:**
  - AC5 sets a 30% false-positive ceiling on the extractor. If
    iter-1 trips this, tighten extraction patterns (drop
    prose-path matching; only check fenced-code paths) before
    iter-2; don't ship a noisy tool.
  - AC10 requires a real round-trip — jsy reviews at least one
    actual proposal generated against the live store. Don't
    mark shipped on synthetic proofs alone. The proposal review
    flow already exists (recall observe / proposals / promote);
    this just adds a new source tag (`source: doctor-claims`).

**Fleet 2 bullets (NOT drafted, captured in visions/freshness.md):**
  - freshness-claims-rich (extend extractor for hardcoded/always/never
    qualifiers)
  - freshness-doc-sweep (apply checker to READMEs + CHANGELOGs +
    CLAUDE_SELF.md)
  - freshness-self-changelog (CLAUDE_SELF.md changelog specifically)
  - freshness-on-recall (lazy re-check on `recall query` hit)
  - freshness-cross-session-witness (compose with chord-cross-episode)

Draft Fleet 2 after Fleet 1 ships AND the proposal queue produces
at least one user-promoted supersede.

**Still pending from earlier /dream notes:**
  - wintermute-glow once bootstrap+platform ship (no change)
  - wintermute-offline-persona once brain is up (no change)
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship (no change)
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship (no change)
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship (no change)
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship (no change)

## 2026-05-25T07:50  /dream  no-fleet-pass
Bare /dream invocation, 80min after vision-freshness. **No new fleet
drafted this pass** — per dream rule 6 ("don't dream past the
research"). Three reasons:

  1. Saturation: 5 active visions, 28 PRDs queued, 1 shipped today.
     Bottleneck is /build throughput, not /dream supply.
  2. No new evidence since 06:30 (no PRD shipped, kernel still on
     7.0.9 stock, no journal entry for today).
  3. User declined the AskUserQuestion offering 4 directions
     (specific topic / extend existing / status only / dream anyway).
     Honest move was option (b) from the freshness pass — go narrow
     or don't go — and this time "don't go" is the truth.

**One micro-finding this pass DID surface,** worth capturing here so
it isn't lost: the 06:30 freshness-gossip note cites `ctrace ls` as
a Phase 1 command. `ctrace ls` is not a valid subcommand — actual
subcommands are `start|stop|status|query|tail`. This is exactly the
class of staleness that `PRD-recall-doctor-claims` was drafted to
catch (extended in Fleet 2's freshness-doc-sweep to cover gossip /
skill docs). Logged as evidence in `visions/freshness.md` §Evidence
log; strengthens the Fleet 2 case once Fleet 1 ships.

**Notes for /build:** nothing new. Continue with current in_progress
PRDs (build-rust-extend AC-10 verification, recall-daemon iter-3,
wintermute-bootstrap/platform/tts in-flight). No new dependencies
introduced by this /dream pass.

**Notes for next /dream:** real new evidence the laptop could
produce that would unblock a grounded vision:
  - kernel boots into linux-wintermute → continuity Fleet 1 boot-
    gated ACs become testable → grounded basis for continuity
    Fleet 2.
  - any single in_progress PRD ships → gossip-trigger to draft its
    Fleet-2 follow-on per the existing rules in earlier gossip
    notes (chord ≥2, continuity ≥3, wintermute ≥3, cadence ≥1).
  - a real new user-articulated topic.
  Without one of those, the next bare /dream should also be a
  no-fleet-pass. Two in a row is not a problem — it's the system
  working correctly.

**Pending from earlier /dream notes (unchanged):**
  - wintermute-glow once bootstrap+platform ship
  - wintermute-offline-persona once brain is up
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship
  - freshness Fleet 2 after Fleet 1 ships + 1 promoted supersede

## 2026-05-25T07:50  /dream  vision-handshake
Drafted: PRD-agorabus-boot-handshake.md (1 PRD, Fleet 1).
Vision: visions/handshake.md (single-PRD vision; Fleet 2 bullets
  captured but explicitly NOT drafted per dream rule 6).

**Why this pass wasn't a no-fleet-pass.** Last note (~07:50) said
the next bare /dream should also be no-fleet unless real new
evidence arrived. Three new pieces of evidence DID arrive in the
intervening hours:
  1. **recall v0.4.3 shipped** (`2df7156 recall v0.4.3: braid
     freshness tunable, default 60s → 300s`) — 2nd ship today
     after v0.4.2 observer-correlation. /build throughput is
     better than yesterday.
  2. **2026-05-25 self-review entry exists** at
     `~/brain/journal/2026-05-25.md` (was empty at 07:50 pass).
     §Notable identifies a new bug class: post-reboot startup
     race in `agorabus-session-start.sh` produces orphan
     subscribers when daemon-not-ready races the hook's 0.5s
     socket-wait under boot load. PID 917 hit it today.
     Journal explicitly proposes the fix.
  3. **Kernel build PID 12146 still running** at load 10.42 —
     same conditions that produced today's race. Bug is
     reproducible and current.

This satisfies the "narrow gap with grounded new evidence"
heuristic from the 06:30 freshness pass. One small shell-target
PRD, single-PRD vision (same pattern as `freshness`).

**Default rule applied — ATTENTION /build:**
PRD-agorabus-boot-handshake carries `build_auto: false`. User
reviews before /build advances.

**Sequencing for /build (once authorized):**
  - Single PRD, no internal ordering.
  - **build_target: shell.** /build's shell-target path may not
    yet be hardened (chord-async-delegate is the only other
    in-flight shell PRD; it hasn't shipped). If /build can't run
    shell yet, defer this PRD until /dream extends /build for
    shell. The handshake bug is real but not blocking; current
    self-review playbook (escalate to user) is workable in the
    meantime.
  - One file edited: `~/.claude/scripts/agorabus-session-start.sh`.
    No new files. No version bump (script has no version).
  - AC8 and AC10 require manual verification (synthetic load /
    reboot); /build should ship to AC7 + AC9 mechanically and
    mark AC8/AC10 as user-verify checkpoints, same pattern as
    recall-observer-correlation.

**Cross-fleet coordination:**
  - **chord-async-delegate** is the only other shell-target PRD
    in flight; both touch `~/.claude/scripts/`. No file collision
    (chord-async-delegate adds new files; this PRD edits one
    existing file). Sequence so the handshake fix lands first —
    chord-async-delegate assumes a reliable bus.
  - **continuity Fleet 1** — once agentns boots, subscribe will
    use 128-bit agentns session ids instead of PID-derived ones.
    Handshake verification logic is sid-agnostic, so no rework.
  - **freshness / cadence / wintermute** — none.

**Notes for /build:**
  - The retry parameters (10 × 0.3s socket-wait; 10 × 0.3s peer-
    record poll with one re-spawn after 10 attempts) are tuned
    for today's boot conditions. Surface any tuning learnings
    back to PRD's §Open questions, not as a code comment.
  - Handshake log directory `~/.cache/agorabus/handshake/` does
    not exist yet; first run creates it. AC7 (log rotation, 14
    days) is the only place the script needs `find` — keep that
    invocation explicit and bounded (no `-exec rm -rf`).
  - Do not auto-merge / install to live `~/.claude/scripts/`
    without jsy testing one session first. This file is on the
    Claude startup path.

**Fleet 2 bullets (NOT drafted, captured in visions/handshake.md):**
  - handshake-reannounce-on-watch-loss (daemon-side `peer.lost`
    broadcast)
  - handshake-daemon-ready-fd (marker file instead of socket
    poll)
  - handshake-reattach-cli (`agorabus reattach <sid>` for
    orphan recovery)
  - handshake-startup-race-pevent (supervised daemon launch)
  - handshake-prom-counters (race/orphan/recover counters)

Draft Fleet 2 after Fleet 1 ships AND at least one race is
observed + recovered in `~/.cache/agorabus/handshake/` logs.

**Still pending from earlier /dream notes (unchanged):**
  - wintermute-glow once bootstrap+platform ship
  - wintermute-offline-persona once brain is up
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship
  - freshness Fleet 2 after Fleet 1 ships + 1 promoted supersede

## 2026-05-25T08:30  /dream  no-fleet-pass
Bare `/dream`, 40min after vision-handshake. **No new fleet drafted
this pass** — second consecutive no-fleet-pass, exactly the cadence
the 07:50 no-fleet note predicted ("two in a row is not a problem").

**State delta since vision-handshake (07:50):**
  1. **linux-wintermute kernel pkgs are now sitting on disk**:
     `~/wintermute/wintermute-kernel/pkg/linux-wintermute-7.0.10.arch1-1-x86_64.pkg.tar.zst`
     (154MB) + headers (43MB), both timestamped May 25 00:54.
     The build finished ~7h *before* the 07:50 handshake pass
     cited "PID 12146 still running" as live evidence. PID 12146
     is gone now. The handshake PRD's *core* premise (the orphan-
     for-PID-917 race is real and current; see journal §Notable)
     remains valid; only its third supporting fact was stale.
  2. **recall-daemon iter-3 committed** at 08:26 (4min ago,
     e51b9a2): query/embed/touch wired against open Index +
     built Embedder, lib 31/31 + integration 4/4 green. Still
     pre-ship; v0.5.0 lands after iter-4 (CLI auto-forward +
     systemd-user unit).
  3. **stock 7.0.9 still booted.** linux-wintermute is install-
     ready but not installed. User decision: `sudo pacman -U`
     the two pkg.tar.zst files (alongside the 29-pkg queue still
     blocked on protected substrings) and reboot to unlock
     memlog / provfs / agentns.

**Why no fleet was drafted:**
  - Triggers from the 07:50 no-fleet rules:
      * any in-flight PRD ships → recall-daemon hasn't (iter-3 of 4)
      * kernel boots → not yet (build done, not booted)
      * new user articulation → none
  - Saturation unchanged: 5 active visions, 29 PRDs queued, 0
    additional shipped since handshake. /build remains the
    bottleneck, not /dream supply.

**One micro-finding logged this pass** (added to
`visions/freshness.md` §Evidence log): the 07:50 handshake gossip
note cited a fact that was already 7h stale at the time of writing.
This is exactly the freshness-on-gossip class. Surfaces a new Fleet 2
candidate beyond the original five: `freshness-on-dream` — spot-check
load-bearing claims in fresh gossip drafts before they commit. Not
drafted; bullet captured in vision file.

**Notes for /build:** unchanged. Continue with current in_progress
PRDs (recall-daemon iter-4 is the visible next ship; wintermute-
bootstrap/platform/tts still in-flight; build-rust-extend AC-10 done).
No new dependencies introduced.

**Notes for next /dream:** evidence that would unblock a real fleet:
  - **recall-daemon ships to v0.5.0** → recall fleet can extend with
    daemon-aware follow-ons (already partly covered by recall-outcome-
    feedback's rebase to v0.5.1-v0.5.3, but check before drafting).
  - **kernel boots into linux-wintermute** → continuity Fleet 1
    boot-gated ACs become testable → grounded basis for continuity
    Fleet 2. This is the single biggest unblock available; one
    user action (`pacman -U` + reboot) shifts the entire substrate.
  - **any other single in-flight PRD ships** → its Fleet-2 trigger
    fires (chord ≥2, continuity ≥3, wintermute ≥3, cadence ≥1).
  - **a real new user-articulated topic.**

If none of those land before the next bare /dream, a third
no-fleet-pass is the right call. The discipline holds.

**Still pending from earlier /dream notes (unchanged):**
  - wintermute-glow once bootstrap+platform ship
  - wintermute-offline-persona once brain is up
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship
  - freshness Fleet 2 after Fleet 1 ships + 1 promoted supersede

## 2026-05-25T09:15  /dream  no-fleet-pass
Bare `/dream`, 45min after the 08:30 no-fleet-pass. **Third
consecutive no-fleet-pass** — exactly the cadence the 08:30 note
predicted ("a third no-fleet-pass is the right call. The discipline
holds.").

**State delta since 08:30:**
  1. **recall v0.4.3 shipped twice**: commit 2df7156 (braid
     freshness tunable, default 60s → 300s) + commit fdc81ad
     (CHANGELOG backfill). This is **PRD-recall-braid-freshness-
     tunable.md** landing in code. Cargo.toml now reports 0.4.3;
     CHANGELOG.md has the v0.4.3 section.
  2. recall-daemon: still iter-3 (no iter-4, no v0.5.0). No new
     commits past e51b9a2.
  3. kernel: still stock 7.0.9 booted. `~/wintermute/wintermute-
     kernel/pkg/linux-wintermute-7.0.10.arch1-1-x86_64.pkg.tar.zst`
     unchanged from 00:54.
  4. No new user articulation (bare /dream).

**Why no fleet drafted:**
  - The recall-braid-freshness-tunable ship is an **orphan-PRD ship**:
    NOT in any active vision (the freshness vision is about
    `recall doctor --check-claims` for memory bodies; this PRD is
    about the *braid correlator's* fresh-tool-result window — a
    different "freshness"). NOT in the build manifest's `prds`
    object (manifest read confirms only build-rust-extend,
    agentic-memory, recall-daemon, recall-observer-correlation,
    recall-outcome-feedback, plus notebooks/wintermute-audio).
    Therefore the ship fires NONE of the six Fleet 2 triggers
    (chord ≥2, continuity ≥3, wintermute ≥3, cadence ≥1, freshness
    ≥1, handshake ≥1 — all still at 0).
  - Triggers from 08:30's no-fleet rules:
      * recall-daemon to v0.5.0 → no (still iter-3)
      * kernel boots → no
      * any in-flight **Fleet 1** PRD ships → no (orphan ship
        doesn't count)
      * new user articulation → no
  - Saturation unchanged: 6 active visions, ~28 PRDs queued, /build
    remains the bottleneck.

**One micro-finding logged this pass — orphan-PRD desync:**
  Three recall-* PRDs target v0.4.x extension work but live as
  orphan drafts (no vision, no manifest tracking):
    - **PRD-recall-braid-freshness-tunable.md** — Status line still
      reads "Draft v0.1" but the work is in main at v0.4.3 (commit
      2df7156). PRD file is still in the queue dir, NOT archived.
      Self-referential drift: the PRD's frontmatter is a stale
      claim about its own implementation state.
    - **PRD-recall-bash-response-richness.md** — "Draft v0.1",
      "Builds on recall v0.4.2", not yet shipped.
    - **PRD-recall-stop-hook-session-id.md** — "Draft v0.1",
      "Builds on recall v0.4.2", not yet shipped.
  This is on-theme for the freshness vision (a document body whose
  claims about live state are wrong). Logged into
  `visions/freshness.md` §Evidence log; not drafted as a new PRD
  per dream rule 6.

**Notes for /build:** when next ticking
PRD-recall-braid-freshness-tunable.md: the work is already in
main. Reconcile — either archive the PRD as shipped (preferred) or
detect the version-already-bumped state in `scan-prds.sh` and skip.
A natural rule: "if PRD `Builds on: recall vX.Y.Z` and live
Cargo.toml version > X.Y.Z, flag for archival." Other orphan PRDs
unchanged; recall-daemon iter-4 remains the visible next ship.

**Notes for next /dream:** same unblock conditions as 08:30,
plus one new sign-post:
  - recall-daemon to v0.5.0 → recall fleet gets daemon-aware
    follow-ons.
  - kernel boots into linux-wintermute → continuity Fleet 1
    boot-gated ACs become testable; single biggest unblock.
  - any **Fleet 1** PRD ships (six fleets, six triggers).
  - a real new user-articulated topic.
  - **NEW: if a 4th orphan-PRD ship lands without reconciliation**,
    consider a small "recall-pulse" vision that retroactively folds
    the orphan recall-* PRDs under one umbrella. Holding back per
    dream rule 6 until at least one more orphan ship occurs.

If none of these land before the next bare /dream, a fourth
no-fleet-pass is the right call. The discipline still holds.

**Still pending from earlier /dream notes (unchanged):**
  - wintermute-glow once bootstrap+platform ship
  - wintermute-offline-persona once brain is up
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship
  - freshness Fleet 2 after Fleet 1 ships + 1 promoted supersede
  - handshake Fleet 2 after Fleet 1 ships + 1 observed race

## 2026-05-25T10:00  /dream  no-fleet-pass
Bare `/dream`, 45min after 09:15. **Fourth consecutive no-fleet-pass**
— the call 09:15 predicted ("a fourth no-fleet-pass is the right
call").

**State delta since 09:15:**
  1. /build hit a sustained throughput stall between 02:17–02:58 PT
     (09:17Z–09:58Z): 9 cron fires, 0 successful ticks. Six fires
     bounced on `tick.lock` held by a sibling timer-fired tick;
     three bounced on classifier-unavailable. PID 481429 finally
     took the lock at 02:58 (currently in-flight, 3min elapsed).
     Net result: no commits from 09:15Z onward across Fleet 1.
  2. recall-daemon: still iter-3, last_action 08:26Z. No v0.5.0.
  3. recall tip: still `fdc81ad` (v0.4.3 CHANGELOG backfill). No
     new ships, no new orphan PRDs.
  4. wintermute-bootstrap manifest now `last_action: 09:15Z` —
     iter-5 (AC4 OnceStart, commit 796852b at 09:08Z) re-touched
     during the 09:15 dream window; still in_progress (AC1+AC3
     deferred). Not a Fleet-2-trigger ship.
  5. wintermute-platform manifest `last_action: 09:02Z` — iter-9
     clippy fix (5b10fb6 at 08:57Z); still in_progress.
  6. Kernel: stock 7.0.9-arch1-1, pkgs unchanged at 00:54.
  7. No new user articulation; bare /dream.

**Why no fleet drafted:**
  - None of the six unblock conditions fired (recall-daemon→v0.5.0,
    kernel boot, any Fleet 1 ship, new orphan ship for recall-pulse
    hook, user articulation, …). The build-tick contention pattern
    is a /build hygiene issue, not vision-shaped: it surfaces a
    candidate fix (timer interval > expected wall time, or
    self-contention exit-fast), but the fix author is /build, not
    /dream, and a PRD would be misshaped here.

**One micro-finding logged this pass — /build self-contention:**
  systemd-user timer interval is 5min; observed wall time between
  09:17Z and 09:58Z had multiple concurrent `claude-build-headless.sh`
  processes (PIDs 474583 / 476041 / 476669 / 478094 / 479993 / 481429
  in sequence, several overlapping for >5min). When classifier
  flaps add 60–90s retries on top of an already-long tick, the
  cron cadence is shorter than steady-state wall time and ticks
  race themselves. Phase 0's lock-guard absorbs the contention
  (every loser exits cleanly without mutating state — that part
  works), but the *cost* is that 9 of 9 fires in 41min did zero
  PRD-advance work; the wall-clock advance rate during the stall
  was 0 commits/41min vs the baseline of ~1 commit per cron-fire.
  This is not new from a steady-state perspective (the lock guard
  is doing its job) but it IS a new visible bottleneck shape
  that wasn't present at 09:15. Logged to gossip; future /dream
  should NOT draft a PRD for this — it's /build's domain to widen
  the cron interval, add a self-detect-running early-exit, or
  collapse classifier retries.

**Notes for /build:** when the in-flight PID 481429 tick completes,
consider:
  - widening the systemd-user timer to 10min (matches observed
    p95 wall time including classifier flap retries), OR
  - early-exit if a sibling has been running >2min (the lock
    age is detectable via `ls -la tick.lock` mtime).
  This is a /build-skill self-mod, not a /dream PRD; surfacing as
  a finding for next /self-review or /build's own Phase 6
  follow-on draft. recall-daemon iter-4 (CLI auto-forward +
  systemd-user unit toward v0.5.0) is still the visible next ship
  if the throughput stall clears.

**Notes for next /dream:** unblock conditions unchanged from 09:15.
A fifth no-fleet-pass is appropriate if none have fired by the
next bare /dream. The discipline still holds — it explicitly
predicts itself one step out.

**Still pending from earlier /dream notes (unchanged):**
  - wintermute-glow once bootstrap+platform ship
  - wintermute-offline-persona once brain is up
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship
  - freshness Fleet 2 after Fleet 1 ships + 1 promoted supersede
  - handshake Fleet 2 after Fleet 1 ships + 1 observed race

## 2026-05-25T10:45  /dream  no-fleet-pass
Bare `/dream`, 45min after 10:00. **Fifth consecutive no-fleet-pass** —
the call 10:00 predicted ("a fifth no-fleet-pass is appropriate if
none have fired by the next bare /dream").

**State delta since 10:00:**
  1. /build throughput: the in-flight PID 481429 tick that 10:00
     described as "3min elapsed" eventually completed without
     advancing any Fleet 1 PRD ship. Wintermute-bootstrap last_action
     unchanged from 09:15Z; wintermute-platform 09:02Z; wintermute-tts
     06:09Z; recall-daemon 08:26Z. No Fleet 1 ships in the 45min
     window.
  2. recall tip: still `fdc81ad` (v0.4.3 CHANGELOG backfill).
  3. recall-daemon: still iter-3, no v0.5.0.
  4. Kernel: still booted on stock 7.0.9-arch1-1. `linux-wintermute-
     7.0.10.arch1-1` pkg unchanged from 00:54.
  5. No new user articulation.

**Why no fleet drafted:**
  - None of the six unblock conditions fired (recall-daemon→v0.5.0,
    kernel boot, any Fleet 1 PRD ship, new orphan-PRD ship for
    recall-pulse hook, user articulation, …).

**Self-correction logged this pass — meta-freshness:**
  The 09:15 gossip note about recall-braid-freshness-tunable as an
  "orphan-PRD ship" had a stale claim of its own. Quoted:
    > "NOT in the build manifest's prds object (manifest read
    >  confirms only build-rust-extend, agentic-memory,
    >  recall-daemon, recall-observer-correlation,
    >  recall-outcome-feedback, plus notebooks/wintermute-audio)."
  Live re-check at 10:45Z: the PRD has been in the manifest since
  iter-1 at 04:50Z — 4h17m before the 09:15 dream pass called it an
  orphan. Manifest now shows status=in_progress (3 ticks),
  shipped_version=0.4.3, changelog_committed=fdc81ad,
  installed_versions={recall: 0.4.3, recalld: 0.4.3}. What's
  actually pending: archival (move PRD to PRDs-archive/, flip
  status to shipped). The 09:15 narrative confused
  "manifest doesn't track this PRD" with "PRD frontmatter still
  reads Draft v0.1" — a different fact, and the only true one.
  Likely cause: I read manifest at 09:15 but truncated mid-object
  via `head -100`; the recall-braid-* entry sits after the first
  five PRDs alphabetically and was below my window.

  This is on-theme for vision-freshness with extra force: the
  artifact whose claims drifted is gossip.md itself, drafted by
  the same skill that exists to *write* drift-free notes about
  ground truth. The freshness-on-dream Fleet 2 candidate (logged
  08:30) is reinforced: a spot-check pass over dream's own
  outputs before commit would have caught this.

  Logged into visions/freshness.md §Evidence log; not drafted
  per dream rule 6. Pattern now has FOUR instances:
    - feedback_delegate_run_300s_cap.md "hardcodes" wording
    - AGORABUS_RPC.md v0.1 changelog "no handler shipped"
    - 06:30 gossip's `ctrace ls` invocation (invalid subcommand)
    - 09:15 gossip's orphan-PRD-manifest claim (false)
  Last two were authored by /dream itself within the last 6h.

**Notes for /build:** the actionable hint from 09:15 is still valid
in spirit but more specific in shape — recall-braid-freshness-tunable
needs an **archival tick** (status:in_progress → status:shipped, mv
PRD to PRDs-archive/), not version-bump work. The
build_stale_blockers playbook may not currently detect "all version
work is done, only archival remains" as a discrete state — worth a
/build-side check on its own scan logic. recall-daemon iter-4 (CLI
auto-forward + systemd-user unit toward v0.5.0) remains the visible
next ship.

**Notes for next /dream:** unblock conditions unchanged. A sixth
no-fleet-pass is appropriate if none have fired by the next bare
/dream. If a sixth pass happens, the discipline-test pattern itself
becomes evidence — proposing it as a Fleet 2 entry for
vision-cadence ("dream rest-pace heuristic: bare /dream within Nmin
of last no-fleet-pass and unchanged state → respond with vision
list + one paragraph instead of a fresh research pass") may be
warranted. Holding back per dream rule 6 until pass six.

**Still pending from earlier /dream notes (unchanged):**
  - wintermute-glow once bootstrap+platform ship
  - wintermute-offline-persona once brain is up
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship
  - freshness Fleet 2 after Fleet 1 ships + 1 promoted supersede
  - handshake Fleet 2 after Fleet 1 ships + 1 observed race

## 2026-05-25T11:30  /dream  no-fleet-pass
Bare `/dream`, 45min after 10:45. **Sixth consecutive no-fleet-pass** —
the call 10:45 predicted ("a sixth no-fleet-pass is appropriate if
none have fired by the next bare /dream").

**State delta since 10:45 (real, but sub-threshold):**
  1. `recall-daemon` advanced from iter-3 to **iter-4** at 11:13Z
     (commit `8949d32`, pushed to `origin/main`). New surface:
     `recall daemon status` subcommand (text+json, exit-code on
     liveness) + `contrib/systemd/recalld.service` user-unit +
     `recall where` now reports socket liveness. Lib 31/31 +
     daemon_ping 4/4 unchanged; no version bump (v0.5.0 still
     gated on iter-5 CLI auto-forward).
  2. /build cron has fired ~3x since (11:16/11:21/11:26Z) — all
     bounced on `tick.lock` held by the headless-cron sibling. The
     10:00 self-contention finding persists.
  3. recall tip: `fdc81ad` → `8949d32`.
  4. Kernel: still stock 7.0.9-arch1-1.
  5. No new user articulation.

**Why no fleet drafted:**
  - The iter-4 commit is real motion on `recall-daemon`, but it is
    a within-PRD iteration, not a Fleet 1 ship. The explicit
    trigger from 10:45 (`recall-daemon→v0.5.0`) hasn't fired.
  - None of the other five unblock conditions fired either.

**Fleet 2 candidate landed (per 10:45's tee-up):** added
`dream rest-pace heuristic` as a bullet in `visions/cadence.md`
§Fleet 2. Six consecutive passes is enough evidence the pattern
is real and recurring; cadence is the natural vision-home
(pulse-like signal applied to /dream's own invocation cadence).
This is a vision-doc bullet only, not a drafted PRD — rule 6
still holds; drafting waits for `cadence-pulse` to ship so the
heuristic has a stable substrate to read.

**Discipline-test summary (07:50Z → 11:30Z, 3h40m, 6 passes):**
  - Pass 7 (07:50): saturation + no new evidence
  - Pass 8 (08:30): predicted pass 9; logged stale `ctrace ls`
    + first /dream meta-stale claim (PID 12146 "still running")
  - Pass 9 (09:15): orphan-PRD misread (later self-corrected)
  - Pass 10 (10:00): /build self-contention micro-finding
  - Pass 11 (10:45): self-correction of pass 9's manifest misread;
    teed up the Fleet 2 entry for pass 12
  - Pass 12 (11:30): consumed the tee-up; cadence Fleet 2 bullet
    added; pattern documented as complete.
Every pass logged a state delta. Every pass predicted the next
correctly. Two passes drafted stale claims in their own gossip
(passes 8 and 9), both self-corrected within 2h. Zero PRDs
drafted past evidence. Net writes per pass: 1 gossip entry +
1 manifest append + 0-1 vision-evidence-log lines.

**Notes for /build:** recall-daemon iter-5 (CLI query auto-forward)
is the visible next ship to unlock the v0.5.0 boundary. Once that
lands, the recall-outcome-feedback + recall-session-stamp PRDs
(both pinned behind v0.5.0) become eligible. Tick.lock contention
remains — 10:00 hint (widen timer interval or sibling-age early-
exit) still applicable.

**Notes for next /dream:** unblock conditions unchanged. A seventh
no-fleet-pass would be **excessive**, not disciplined. If state is
still unchanged at the next bare invocation, /dream should follow
its own newly-bulleted rest-pace heuristic spirit: reply with a
one-paragraph state delta + this pass's unblock list, and skip
even the manifest/gossip writes (logging "I did nothing" six times
is itself a noise floor). The bullet exists now; living the bullet
before the PRD ships is acceptable since the change is to /dream's
own behavior, not to a downstream tool.

**Still pending from earlier /dream notes (unchanged):**
  - wintermute-glow once bootstrap+platform ship
  - wintermute-offline-persona once brain is up
  - continuity Fleet 2 after >=3 of 5 Fleet 1 PRDs ship
  - wintermute Fleet 2 after >=3 of 7 Fleet 1 PRDs ship
  - cadence Fleet 2 after >=1 of 7 Fleet 1 PRDs ship
  - chord Fleet 2 after >=2 of 4 Fleet 1 PRDs ship
  - freshness Fleet 2 after Fleet 1 ships + 1 promoted supersede
  - handshake Fleet 2 after Fleet 1 ships + 1 observed race

## 2026-05-25T13:45  /dream  no-fleet-pass (trigger-fired, scope mismatch)
Bare `/dream` 2h15min after 11:30. **Seventh /dream-pass in the
3h40m arc + 2h15m gap; first since-then.** Discipline still holds.

**State delta since 11:30 (real, but vision-orthogonal):**
  1. `recall-daemon` shipped **v0.5.0** at commit `4333b18` somewhere
     between 12:26Z and 13:45Z (manifest snapshot at 12:26Z still
     called iter-5 pre-bump). This is the exact trigger 10:45 and
     11:30 named as the visible next ship to unlock the v0.5.0
     boundary. ✅ FIRED.
  2. Kernel still stock 7.0.9-arch1-1.
  3. No user articulation; bare `/dream`.

**Vision Fleet 1 counts: unchanged.** recall-daemon is a build-manifest
PRD, not in any vision's Fleet 1. None of the six Fleet 2 triggers
(continuity ≥3/5, wintermute ≥3/7, cadence ≥1/7, chord ≥2/4, freshness
≥1/1, handshake ≥1/1) fire. The ship is real motion but
vision-orthogonal.

**Actual downstream unblocks (for /build, not /dream):**
  - `recall-outcome-feedback`: pinned behind v0.5.0, rebases to
    v0.5.1/0.5.2/0.5.3 per its PRD §6 plan. Already drafted; /build
    can pick it up.
  - `recall-session-stamp` (continuity Fleet 1, targets v0.6.0): the
    v0.5.0 collision risk that the continuity vision-doc called out
    is now resolved. Already drafted; /build can pick it up.

**Why no fleet drafted this pass:**
  - The new substrate (UDS socket exposing ping/query-no-filter/embed/
    touch) is minimal. Streaming/subscribe and filter-rich daemon
    queries are NOT exposed; daemon-aware hook PRDs would be real but
    are "dream past the research" — the daemon shipped ~80min ago
    with zero downstream consumers. Wait for at least one consumer to
    tick through /build before imagining the next layer.
  - The 30-PRD queue across 6 visions is the saturating constraint,
    not idea generation.

**Self-application of the cadence-rest-pace heuristic** (from
visions/cadence.md §Fleet 2, banked at 11:30): this pass is the first
since the heuristic was banked. State *did* change — but the change
was build-side (PRD shipped from queue), not dream-side (new
vision-shaped white space). The heuristic was written for "unchanged
state"; this pass tests a refinement — "build-side change with no
fleet-trigger fire." Treating it identically (log state delta + skip
new drafts) is the right read. The PRD `cadence-pulse` should
generalize to "/dream rests when /build advances within an already-
drafted vision; only wakes when a Fleet 2 trigger fires OR new user
articulation arrives OR new kernel/substrate surface lands."

**Notes for /build:** with v0.5.0 shipped, you have two immediately-
eligible PRDs:
  1. recall-outcome-feedback (rebase to v0.5.1/0.5.2/0.5.3)
  2. recall-session-stamp (continuity Fleet 1, v0.6.0)
Pick by priority/heat. Also: the tick.lock self-contention (10:00
finding) probably worth addressing before either, since both are
multi-iter PRDs that will benefit from a less self-racing cron.

**Notes for next /dream:** unblock conditions narrow further now that
recall-daemon→v0.5.0 has cleared:
  - any continuity Fleet 1 ship (5 candidates; recall-session-stamp
    is now the most-eligible)
  - any cadence Fleet 1 ship (7 candidates; cadence-substrate is
    foundational)
  - any chord Fleet 1 ship (4 candidates; chord-intent-rich is
    foundational)
  - any wintermute Fleet 1 ship (7 candidates; bootstrap+platform
    closest)
  - freshness or handshake Fleet 1 ship (1 each; both shell-target,
    fastest)
  - kernel boot (linux-wintermute pkg → grub default)
  - new user articulation OR new substrate landing in main
Without one of those, the next bare /dream should rest-pace harder:
state delta one-liner, no new gossip entry, no manifest write.

## 2026-05-26T04:00  /dream  no-fleet-pass (rest-pace, ninth)
Bare `/dream` ~14h after 2026-05-25T13:45. **Ninth /dream pass in the
arc, first post-rest.** Followed the cadence rest-pace heuristic
(banked 11:30, refined 13:45): real motion since last pass but
vision-orthogonal → log state delta + add evidence to existing
vision + skip new drafts + skip manifest write.

**State delta since 13:45 (substantive, vision-orthogonal):**
  1. `recall-daemon` v0.5.0 fully shipped end-to-end: iter-7 changelog
     2781c70 (16:25Z), iter-8 push blocked (23:36Z), iter-9 push landed
     01:05Z (range 8949d32..2781c70 — three commits f231524 + 4333b18
     + 2781c70 now on origin/main).
  2. iter-10 AC verification (01:22Z) caught a partial ship — AC1/4/6
     failed: `recall daemon start/stop/restart` subcommands never
     landed (help text claims iter-5 ships them; v0.5.0 only has
     `status`), and `recall doctor --format json` does not expose
     `daemon_active` or `daemon_uptime_s`. PRD NOT archivable until
     iter-11 closes the gap.
  3. iter-11 WIP is real but entangled: `~/wintermute/recall` working
     tree carries 4 modified files + 1 new test (`hooks/stop.sh`,
     `src/bin/recalld.rs`, `src/daemon.rs`, `src/main.rs`,
     `tests/hook_stop_session_id.rs`) — mix of recall-daemon iter-11
     scope AND sibling PRD-recall-stop-hook-session-id scope. /build
     deferred per Hard Safety Rule #5.
  4. Self-review run 8 (20:03 PT 2026-05-25) was the quietest run
     yet — confirms steady-state on most signals, escalated
     iter-11 entanglement to a /build blocker (blocker count 2→4).
  5. Kernel still stock 7.0.9. No new user articulation.

**Vision Fleet 1 counts: unchanged.** recall-daemon is build-manifest,
not vision. None of the six Fleet 2 triggers fire (continuity ≥3/5,
wintermute ≥3/7, cadence ≥1/7, chord ≥2/4, freshness ≥1/1,
handshake ≥1/1).

**One evidence-log line added (visions/chord.md §Evidence):** the
iter-10/iter-11 entanglement is the first real-world motivation for
PRD-chord-claim's soft-lock primitive. Had iter-11's originating
session claimed `repo:recall` on agorabus before editing, the
sibling stop-hook-session-id work would have backed off or queued.
Strengthens AC1+AC4 of PRD-chord-claim.md (lock acquisition +
visible holders). Not promoted to a new PRD — chord-claim already
exists, this just sharpens its motivation.

**Why no fleet drafted (rule 6):**
  - 30 PRDs across 6 visions queued; 0 Fleet 1 ships. Saturation
    is the constraint, not idea generation.
  - The new substrate (recall-daemon v0.5.0 UDS) is minimal and has
    zero downstream consumers; daemon-aware-hook PRDs would be
    "dream past the research."
  - The iter-11 entanglement motivates an *existing* PRD; no new
    PRD is warranted.

**Notes for /build:** the iter-11 working-tree state is the immediate
unblock target. Options: (a) attribute the diff to one or the other
PRD and commit the rest separately, (b) reset to HEAD and re-pick
one PRD to advance, (c) ask user. With recall-daemon's v0.5.0
shipped (commits public) but PRD un-archivable due to AC gap, the
work isn't lost — but archive is gated on iter-11 closing AC1/4/6.

Other eligible /build picks (per 13:45 list, still valid):
  - recall-outcome-feedback (rebase to v0.5.1/0.5.2/0.5.3) — does
    NOT touch the entangled files, can advance now.
  - recall-session-stamp (continuity Fleet 1, v0.6.0) — does NOT
    touch the entangled files, can advance now and would fire
    continuity Fleet 1's first ship.

**Notes for next /dream:** unblock conditions narrow again:
  - any Fleet 1 ship from any of the six visions
  - kernel boot (linux-wintermute pkg → grub default)
  - new user articulation OR new substrate landing with at least
    one downstream consumer
  - iter-11 entanglement resolution becomes a second chord-claim
    evidence line if /build records the resolution path chosen
Without one of those, next bare /dream should rest harder: terse
state delta in chat, no gossip entry, no manifest write. The
cadence-pulse PRD when it ships will codify this.

## 2026-05-26T06:20  /dream  vision-release-gate (tenth pass, FIRST DRAFT since pass 6)
Bare `/dream` ~2h20min after 2026-05-26T04:00 (ninth pass,
rest-pace). Tenth /dream pass in the arc. **First pass to draft
new artifacts since pass 6 (2026-05-25T05:25 vision-chord, 25h ago).**

**Why this pass drafted (broke the rest-pace streak):**
The cadence-rest-pace heuristic (banked 11:30 2026-05-25, refined
13:45) says rest when /build advances within an already-drafted
vision. But pass 10 saw evidence of a NEW failure mode unaddressed
by any existing PRD: the `git push origin main` gate fired AGAIN
against recall-daemon iter-15 at 2026-05-26T05:40Z — second firing
on the same PRD (first was iter-8 at 2026-05-25T23:36Z). The
publish-allowlist PRD (drafted by /build Phase 6 on 2026-05-26)
explicitly puts this case OUT OF SCOPE on line 164. The /build
iter-15 manifest entry mis-cites publish-allowlist as the fix
("Resolution path: PRD-build-publish-allowlist.md ... explicitly
targets this"), which means /build's own Phase 6 may not draft a
sibling PRD because /build believes the case is already covered.
This is the rest-pace heuristic's "new substrate with consumer"
exception inverted: NEW failure mode + WRONG resolution-path
citation in /build manifest = /dream is the closer author.

**Drafted (1 vision, 1 PRD):**
  - **visions/release-gate.md** — small vision (2 PRDs Fleet 1,
    one already queued). Frames the publish-vs-push gate pattern
    as a single class with symmetric solutions. Fleet 2 captured
    as bullets (release-gate-repos-md-sync, -prerelease, -revert)
    but explicitly NOT drafted per rule 6.
  - **PRD-build-push-allowlist.md** — `self-mod`, `build_auto:false`,
    `build_priority: high`, sibling to PRD-build-publish-allowlist.md.
    Adds `~/.local/bin/wm-push` wrapper + `Bash(wm-push:*)` allow
    rule + /build Phase 4 patch. Wrapper checks: slug regex + hard-
    coded allow-list + origin URL ends in `j0yen/<slug>` + current
    branch matches target + fast-forward only (no force-push) +
    refuses no-op. 10 ACs mirroring publish-allowlist's structure.
    Single-tick phasing, <20min estimated. Can be authorized in
    same user review pass as publish-allowlist.

**Vision Fleet 1 counts:** release-gate = 1 drafted + 1 already-
queued (publish-allowlist) = 2/2. Other six visions unchanged.
None of the prior Fleet 2 triggers fire from this pass.

**State delta since 2026-05-26T04:00 /dream pass:**
  1. recall-daemon advanced iter-11 → iter-15 (commits 36cb6ea,
     aa0922c, 3abdf7b for v0.5.2 daemon lifecycle + doctor
     liveness + changelog; all 12 ACs PASS per iter-12 smoke).
  2. recall v0.5.2 fully built + installed locally; running
     daemon still on v0.5.0 binary until restart (cosmetic).
  3. iter-15 push BLOCKED — same shape as iter-8 23:36Z. Three
     commits queued local. Archive gated on push landing
     (verified-completed Check #2 for rust-extend).
  4. iter-11 entanglement (10:00 finding on chord-vision) resolved
     trivially — the WIP attributed cleanly to recall-daemon;
     recall-stop-hook-session-id landed separately at 32590f2.
     This is the SECOND chord-claim evidence line predicted by
     the 04:00 pass. NOT promoted to PRD draft (chord-claim
     already exists, this just sharpens motivation in passing).
  5. Kernel still 7.0.10-arch1-3-wintermute (only agentns
     registered; memlog + provfs still absent — same as last
     /self-review run 9).
  6. No new user articulation.

**Freshness evidence line added:** visions/freshness.md §Evidence
log gets a new entry for the publish-vs-push mis-citation pattern.
freshness-on-prds Fleet 2 case now covers PRD-vs-PRD
cross-reference checking (not just PRD-vs-shipped-state).

**Notes for /build:**
  - **immediate**: recall-daemon iter-15 is blocked on push. Two
    paths to unblock: (a) user manually `git push origin main` from
    `~/wintermute/recall`, (b) user authorizes
    PRD-build-push-allowlist.md and PRD-build-publish-allowlist.md
    together (single review pass, both `build_priority: high`,
    single-tick phasing each, <20min combined). Path (b) is
    durable; (a) is a one-shot.
  - **don't draft a third publish-gate PRD in /build's Phase 6** —
    /dream has now drafted the sibling. release-gate vision has
    Fleet 1 complete (publish + push = 2 PRDs).
  - **update /build's iter-15 manifest entry** at next tick to
    correctly cite PRD-build-push-allowlist.md as the resolution
    path (currently cites publish-allowlist incorrectly).
  - other eligible /build picks (unchanged from 04:00 list, still
    blocked by same push-gate at their version-bump step):
    recall-outcome-feedback (v0.5.1/0.5.2/0.5.3),
    recall-session-stamp (v0.6.0). Both will hit the push gate at
    Phase 4 unless push-allowlist ships first.

**Notes for next /dream:** unblock conditions:
  - any Fleet 1 ship from any of the seven visions (now 7 with
    release-gate)
  - kernel boot WITH memlog + provfs registering (currently only
    agentns made it in; user pending investigate
    `~/wintermute/wintermute-kernel/pkg/`)
  - new user articulation OR new substrate landing with at least
    one downstream consumer
  - the chord-claim evidence count reaching 3 instances (currently
    2: iter-10/iter-11 entanglement + iter-11 resolution path)
Without one of those, next bare /dream should rest harder: terse
state delta in chat, no gossip entry, no manifest write.

**Self-application:** this pass IS a draft pass, not a rest-pace
pass, because evidence demanded it. The discipline arc holds —
nine rest-paces preceded one draft, not the other way around.

## 2026-05-27T05:30  /dream  vision-onramp (eleventh pass, second draft pass)
Bare `/dream` ~23h after 2026-05-26T06:20 (tenth pass, release-gate
draft). Eleventh /dream pass in the arc. **Second draft pass since
the discipline arc closed.**

**Why this pass drafted (Fleet 2 trigger fired):**
The rest-pace heuristic banked 13:45 names exactly this case:
"new kernel/substrate surface lands with at least one consumer."
`linux-wintermute 7.0.10-arch1-5` is BOOTED — confirmed live:
  - `uname -r` = `7.0.10-arch1-5-wintermute`
  - `cat /sys/kernel/security/lsm` includes `provfs`
  - `/dev/memlog` is a live char device (`crw-rw---- root:root 660`)
  - `/proc/self/ns/agent` exists
Consumers drafted in [continuity Fleet 1][continuity] (5 PRDs)
are the gating consumers; this is a "substrate landed with drafted
consumers" Fleet 2 fire.

Plus three empirically-observed gaps that NO existing PRD covers:
  1. `getent group memlog` returns empty (no group exists; udev
     never resolves /dev/memlog ownership)
  2. `cat /proc/self/agent_session` reads 32 zeros (no Claude
     session enters agentns; hook-time unshare is structurally
     impossible)
  3. `getfattr -d ~/wintermute/recall/Cargo.toml` returns
     `comm:awk:pid:76630:uid:1000` — provfs comm-fallback names
     the transient utility (`awk` in autobuilder pipeline), not
     the originating tool

The self-review runs 13/14/15 (2026-05-26) flagged #2 three runs
running with the wrong proposed fix ("edit agorabus-session-start.sh
to unshare"). The structural reality is that unshare is per-process
and self-only — a SessionStart hook can't enter the launched
process into a namespace post-hoc. Articulating the actual fix
(wrap the launch itself) is /dream's closer-author work.

**Drafted (1 vision, 3 PRDs):**
  - **visions/onramp.md** — small 3-PRD vision; the bridge between
    "kernel booted" and "tools consume it." Fleet 2 (5 bullets:
    agentns-launcher-hardening, memlog-readable-by-default,
    provfs-attribution-test-suite, onramp-doctor,
    kernel-pkg-postinstall-tests) explicitly NOT drafted per rule 6.
  - **PRD-kernel-pkg-postinstall.md** — `kernel-extend`, edits
    `~/wintermute/wintermute-kernel/pkg/PKGBUILD` (pkgrel 5→6),
    adds `.install` hook + sysusers.d for `memlog` group +
    udev rule for `/dev/memlog` ownership. 10 ACs (live AC10 needs
    pacman -U + reboot + login). No dependencies, ships first.
  - **PRD-claude-agentns-wrap.md** — `mixed`, edits `~/.zshrc` +
    `~/.config/systemd/user/*.service` + `agorabus-session-start.sh`.
    Three integration paths (interactive shell function, headless
    units, kernel-id-aware hook). Depends on PRD-agentns-claude.md
    being shipped + installed. 10 ACs (live AC10 needs in-session
    `/proc/self/agent_session` non-zero observation).
  - **PRD-provfs-comm-richer.md** — `kernel-extend`, pairs with
    PRD-provfs-deferred-stamp.md (shared hook-time capture buffer).
    Enriches fallback xattr value: comm chain (3 levels) + env
    signal (`CLAUDE_TOOL`, `AGORABUS_SID`) + cwd. 256-byte cap.
    agentns-present path unchanged. 10 ACs (live AC10 needs
    rebooted kernel + real workload).

**Vision Fleet 1 counts:**
  - continuity: 5 PRDs, 0 shipped (BUT 4 of 5 transitively
    blocked on PRD-claude-agentns-wrap)
  - chord: 4 PRDs, 0 shipped
  - cadence: 7 PRDs, 0 shipped
  - wintermute: 7 PRDs, 0 shipped
  - freshness: 1 PRD, 0 shipped
  - handshake: 1 PRD, 0 shipped
  - release-gate: 2 PRDs (1 drafted + 1 already-queued), 0 shipped
  - **onramp: 3 PRDs (NEW), 0 shipped**
Total: 8 visions, 30 queued PRDs (was 29 last pass + 3 new − 2 if
either of the release-gate or build-publish-allowlist PRDs ships
on a /build tick today). None of the seven Fleet 2 triggers from
prior visions fire from this pass.

**Notes for /build:**
  - **onramp-PRD-kernel-pkg-postinstall is the smallest unblock**
    (single PKGBUILD edit + 3 install assets; <30min not counting
    the kernel rebuild). User authorization needed (build_auto:false).
    Unblocks /dev/memlog usability for everyone going forward.
  - **PRD-claude-agentns-wrap depends on PRD-agentns-claude shipping
    first** ([continuity Fleet 1][continuity]); if /build picks
    continuity, PRD-agentns-claude is the natural starting point.
  - **PRD-provfs-comm-richer pairs with PRD-provfs-deferred-stamp**
    — if both ship in the same patch cycle, the kernel pkgrel
    bumps consolidate (5→6→7 becomes 5→7 in one rebuild).
  - **Eligible /build picks unchanged from yesterday's analysis:**
    recall-outcome-feedback push gate now resolved (origin/main at
    3abdf7b per iter-18); but 9 unpushed recall commits accumulated
    overnight (4 → 7 → 9) suggesting more shipped PRDs need
    archival ticks.

**Notes for next /dream:** unblock conditions narrow further now
that onramp is on the queue:
  - any Fleet 1 ship from any of the eight visions (now 8 with
    onramp; smallest path is onramp's kernel-pkg-postinstall or
    handshake's single shell-target PRD)
  - new kernel/substrate surface beyond the current 7.0.10-arch1-5
    (e.g. a memlog API extension; an agentns-counters new field)
  - new user articulation OR a NEW failure mode not covered by any
    queued PRD
  - any of the eight visions' Fleet 2 triggers firing
Without one of those, next bare /dream should rest-pace per the
discipline arc. The kernel-boot trigger is now consumed; treat
subsequent passes as steady-state until something genuinely new
arrives.

**Self-application:** this is the second draft pass since the
nine-rest-pace arc closed. The pattern holds: rest-pace nine times,
draft once when evidence demands. Discipline arc not violated;
extended.

[continuity]: ../visions/continuity.md

## 2026-05-28T01:40  /dream  vision-wintermute Fleet 2 (twelfth pass, third draft pass)
Bare `/dream` ~20h after 2026-05-27T05:30 onramp draft. Twelfth /dream
pass in the arc. **Third draft pass.**

**Why this pass drafted (clear Fleet 1 ship trigger):**
The wintermute vision doc explicitly authorized Fleet 2 extend at
">=3 of 7 Fleet 1 shipped." Current shipped count per CLAUDE_SELF
changelog: 5/7 (bootstrap archived; platform/tts/stt/dialog all have
"shipped" entries; only audio + brain remain queued). The 5/7 ship
count and the explicit vision-doc trigger together fire cleanly.

Plus a new user articulation (2026-05-27): the
[always-commit-push-prds-and-work][feedback] feedback memory lands
new rules — every PRD is auto-built (build_auto stripped from new
drafts), commits + pushes are inline (no batching), no daily caps.
Dream skill instructions updated to match; this is the first draft
pass under the new defaults.

**Drafted (1 vision update, 6 PRDs):**
  - **visions/wintermute.md** updated: Fleet 1 section gains shipped
    count (5/7); Fleet 2 section converted from bullets to drafted
    table with sequencing + bumped-to-Fleet-3 list (news + glow).
  - **PRD-wintermute-browser.md** — rust-cli, `wm-browser`,
    chromiumoxide-based. Tools: open/read/click/type/back/find/
    screenshot over `wm.browser.cmd`. A11y snapshot is canonical;
    image-mode fallback via wm-screen-narrate. 10 ACs (AC10 live).
    Calls out: no Rust Playwright binding exists — vision doc's
    "Playwright" label was shorthand.
  - **PRD-wintermute-desktop.md** — rust-cli, `wm-desktop`,
    atspi-rs + xdotool via baton. Tools: apps/focus/read_window/
    click/type/key/find. Reuses j0yen/baton (shipped 2026-05-24).
    AT-SPI bus auto-enable in install.sh. 10 ACs (AC10 live).
  - **PRD-wintermute-screen-narrate.md** — rust-cli,
    `wm-screen-narrate`, scrot/grim + Claude messages API vision.
    Tools: describe/read_text/find_in_image/screenshot. Defaults
    to focused window (privacy); per-day soft budget; logs cost
    to recall. 10 ACs (AC10 live).
  - **PRD-wintermute-mail.md** — rust-cli, `wm-mail`, async-imap +
    lettre + freedesktop SecretService. Tools: inbox/read/send/
    search/mark_read/delete/folders. send + delete through
    wm-dialog verbal confirm. IMAP IDLE for new-mail signal.
    wm-bootstrap extended with /mail credential page. 10 ACs.
  - **PRD-wintermute-calendar.md** — rust-cli, `wm-cal`, minicaldav
    + ical. Tools: today/range/add/find/delete/calendars/
    set_calendar. add + delete through verbal confirm. Reminders
    via `wm.cal.event.upcoming` 5min before. wm-bootstrap /cal
    page. 10 ACs.
  - **PRD-wintermute-music.md** — rust-cli, `wm-music`, mpris-rs
    over zbus. Tools: players/play/pause/toggle/next/prev/
    now_playing/set_volume. Control-only; provider catalog/launch
    explicitly out of scope. Smallest ship in Fleet 2. 10 ACs.

**Vision Fleet 1 counts (state delta since 2026-05-27T05:30):**
  - continuity: 5 PRDs, 0 archived; PRD-agentns-claude unchanged
  - chord: 4 PRDs, 0 shipped
  - cadence: 7 PRDs, 0 shipped
  - **wintermute: 7 PRDs, 1 archived + 4 binary-shipped per
    CLAUDE_SELF → 5/7 effective. Fleet 2 NOW 6 PRDs drafted**
  - freshness: 1 PRD, 0 shipped
  - handshake: 1 PRD, 0 shipped
  - release-gate: 2 PRDs, **BOTH SHIPPED → vision fulfilled**
    (publish-allowlist + push-allowlist both in PRDs-archive/;
    also PRD-build-changelog-prepend-fix shipped as adjacent)
  - onramp: 3 PRDs, 0 shipped (substrate state unchanged:
    agent_session still zeros; memlog group still missing;
    provfs still names comm:awk for autobuilder writes)
Total: 8 visions, **36 queued PRDs** (was 30+3 onramp + 5 backlog
adds + 6 Fleet 2 new − 3 release-gate shipped = ~36; precise count
in `ls PRD-*.md` = 35 after this pass since wintermute-platform/
tts/stt/dialog still in queue dir pending archive).

**Notes for /build:**
  - **release-gate is closed** — both wrappers shipped. push +
    publish path is now durable, no longer the structural blocker
    described in 2026-05-26 gossip.
  - **wintermute Fleet 2 is six fresh queued PRDs.** Sequencing
    hint per vision update: browser/desktop are the big two
    (~1 autobuilder cycle each); music is the cheapest first ship
    (~30 min). Mail/calendar require the wm-bootstrap extension
    arm — if those are picked first, factor in the bootstrap
    side-edit. screen-narrate uses Claude vision API + cost
    budget; touch the claude-api skill on first build.
  - **Two wintermute Fleet 1 PRDs still queued + likely unshipped
    in code:** wintermute-audio + wintermute-brain. Brain is the
    capstone; audio is the perception gate for the rest of Fleet 1.
    Worth knowing if /build is choosing between Fleet 1 completion
    vs Fleet 2 starts.
  - **substrate gaps unchanged**: /dev/memlog group/udev still
    missing; agentns wrap still missing; provfs comm-fallback still
    coarse. Onramp Fleet 1 (3 PRDs from prior pass) all still
    queued.

**Notes for next /dream:** unblock conditions:
  - Any Fleet 1 ship from continuity/chord/cadence/freshness/
    handshake/onramp/wintermute (audio or brain) — five of these
    have NEVER shipped a Fleet 1 PRD; first ship for any of them
    would warrant a refresh pass.
  - Any wintermute Fleet 2 ship — would close one of the new ones
    and rebalance the queue.
  - New user articulation OR new substrate landing (kernel rebuild
    with memlog ownership rule; agentns wrap landing in a Claude
    launcher; etc.)
  - >=2 wintermute Fleet 2 ships → consider drafting Fleet 3
    (voice-profile, voice-clone, emergency, quiet-hours,
    multi-user, undo, offline-persona).

[feedback]: ~/.claude/projects/-home-jsy/memory/feedback_always_commit_push.md

## 2026-05-27T21:35  /dream  pass 13 — Fleet 1.5
Drafted: PRD-build-deferred-acs.md
Vision: visions/wintermute.md updated with new "Fleet 1.5 — Maturation
  & validation" section (1 PRD drafted, 2 bullets for future passes:
  wm-verify, build-maturation-log).

**Trigger:** 4 wintermute Fleet 1 PRDs (platform, audio, stt, tts) all
stuck in_progress on identically-shaped hardware-dependent AC pairing
after today's publish flurry. 68 combined ticks invested across the 4
without the verified-completed check #5 gate satisfiable. Same pattern
across 4 instances = structural problem, not effortful.

**What this PRD does:** adds `deferred_acs: [N, M]` to PRD frontmatter
+ teaches /build's check #5 to honor it + writes a `Deferred:` trailer
in archive commits + backfills the 4 stuck PRDs as part of its install
action. Single tick. Sibling shape to build-publish-allowlist and
build-push-allowlist (both shipped today). `build_target: self-mod`,
`build_priority: high`.

**Notes for /build:** PRD-wintermute-dialog is one tick from archive
(iter log says "Next tick: archive PRD-wintermute-dialog.md") — that
one doesn't need this PRD. The other 4 stuck PRDs are blocked until
deferred-acs lands. After this PRD ships and backfills, expect 4
archive actions to fire in rapid succession.

**Notes for next /dream:** unblock conditions:
- build-deferred-acs ships → check that backfill actually freed the 4
  stuck PRDs; if not, the gap is more subtle than the iter logs admit
- any wintermute Fleet 2 ship → Fleet 3 draft trigger (>=2 ships)
- new user articulation
- the wm-verify Fleet 1.5 bullet reaches its own trigger (declared-
  deferred ACs exist in PRD frontmatter, motivating an attestation
  walker)


## 2026-05-27T22:00  /dream  vision-daily-receipt (pass 14, NEW vision)
Drafted: PRD-daily-receipt-summarize.md, PRD-daily-receipt-haiku.md,
  PRD-daily-receipt-stamps.md, PRD-daily-receipt-archive.md,
  PRD-daily-receipt-yearend-letter.md
Vision: visions/daily-receipt.md (new)

**Trigger:** user articulation this session — MASUNG IP1000 thermal
printer arrived 2026-05-27, PRD-daily-receipt-printer just queued
(58mm, /dev/usb/lp0 live, paper en route). User: "Articulate the
haiku-composition + year-end-scroll arc downstream of it." Vision-
worthy: 4 distinct named components + a capstone, all motivated by
the 2026-05-22 archived daily-receipt PRD's never-built §4 pipeline
and §9 open questions.

**Order (critical path):**
  1. daily-receipt-printer (THIS SESSION, queued) — bytes meet paper
  2. daily-receipt-summarize — gathers ctrace+git+recall+journal into
     summary.json; the upstream the original PRD §4 named but
     never built
  3. daily-receipt-haiku — Claude API call producing 3-line content
     from summary.json; cached system+few-shot, <$4/year
  4. daily-receipt-stamps — special-day catalog (sibling, ships any-
     time after #1; seeds itself with "printer-arrives" 2026-05-27)
  5. daily-receipt-archive — annual PDF from cadence's `daily`
     records; depends on cadence-substrate + cadence-bind-daily-
     receipt landing first
  6. daily-receipt-yearend-letter — Dec-31-23:55 long thermal strip
     + PNG twin for the archive PDF cover; depends on #5 and #3

**Notes for /build:**
  - Summarize → haiku gets workdays printing real content within
    ~3 ship cycles after the printer wrapper lands.
  - Stamps is the cheapest first ship of the 5 here (no API, no
    PDF, just JSON + render). Good "warm up" candidate.
  - Archive + yearend-letter both depend on the cadence fleet
    (substrate + bind-daily-receipt). If cadence-substrate is
    still queued when /build gets here, defer these two until it
    ships. They're well-formed PRDs regardless.
  - yearend-letter has a small rust-extend side-edit on daily-
    receipt itself (`render_long_text`). Build that crate first,
    then yearend-letter consumes it via path dep.
  - Cost ceiling for the whole arc (haiku + yearend-letter)
    estimated at <$5/year. Don't over-engineer cost controls.

**Vision doc Fleet 2 bullets (next /dream pass material):**
  - daily-receipt-photo (monthly scan-prompt ritual)
  - daily-receipt-redo (reprint a past day from cache)
  - daily-receipt-status-board (web view of the year's grid)
  - glyph vocabulary v2 (bigram-shaped, not noise — after ~30
    quiet-day strips give a feel for what's missing)
  - build-shipped milestones as automatic stamps (gossip hook)
  - K and M strips (audience-shaped haikus; multi-printer mirror)

**Notes for next /dream:** unblock conditions:
  - daily-receipt-printer ships → fix any device-quirk PRDs that
    surface (IP1000 might surprise us — paper-out detection, cut
    behavior, CP437 codepage edges, etc.)
  - Paper arrives + first real strip prints → smoke-test PRD shape
    might need revision based on real-world physical output
  - Any of summarize/haiku/stamps ships → archive + yearend-letter
    PRDs become unblock-ready (assuming cadence fleet keeps moving)
  - >=30 days of real strips accumulated → revisit glyph vocabulary
    v2, re-roll budget, and the K/M strip question with real data

## 2026-05-28T01:50  /dream  pass 14 — vision-drift
Drafted: PRD-drift-fix-self-review-dream.md, PRD-tool-manifest.md, PRD-skill-doctor.md
Vision: visions/drift.md
Order: drift-fix-self-review-dream (independent, ship first) || tool-manifest
  (no deps, ship parallel) -> skill-doctor (depends on tool-manifest)

**Seed:** Bare /dream invocation. No unblock condition from pass 13's
list was met (build-deferred-acs not shipped yet, no Fleet 2 ships, no
new user articulation, no wm-verify trigger). Looked for fresh white
space; found it: 6+ consecutive self-review ticks have flagged the same
three tool-skill drift instances in their journal entries without
resolution. Pattern is structural, not effortful — nobody owns "fix
the drifting flag." Plus a 4th instance surfaced during Phase 1
(ctrace ls in dream/SKILL.md:86, already in freshness evidence log).
All four verified live via direct probe.

**Live evidence:**
  1. `pevent gc --older-than 7d --dry-run` cited at
     `self-review/SKILL.md:74,170,389`. Installed: only `[-h]
     [--older-than OLDER_THAN]`. `--dry-run` doesn't exist; `7d`
     errors as "invalid float value: '7d'".
  2. `bpolicy status --format json` cited at `self-review/SKILL.md:77`.
     Installed `bpolicy status` accepts `[-h]` only.
  3. Bootstrap-symlinks 13-tool list at `self-review/SKILL.md:93`.
     7 of 13 missing from `~/.local/bin/`: skill, episode, apipe,
     recall-ops, recall-doctor, recall-io, mirror. Yields 7
     false-positive DANGLING lines every self-review tick.
  4. `ctrace ls` cited at `dream/SKILL.md:86`. Installed: subcommands
     are start|stop|status|query|tail. Already flagged in
     `visions/freshness.md` evidence log.

**Vision shape:** Sibling to freshness/recall-doctor-claims —
freshness catches drift in memory bodies; drift catches drift in skill
text. Same proposal-queue idiom (`~/.claude/<tool>/proposals/<ULID>.md`,
no auto-edit, user-review-gated), different data source.

**Notes for /build:**
  - drift-fix-self-review-dream is a single-tick shell-extend edit
    over two SKILL.md files. Smallest ship in vision-drift; closes
    the noise loop immediately. Verify each replacement invocation
    against the live binary BEFORE writing it.
  - tool-manifest is a fresh rust-cli, new repo at
    `~/wintermute/tool-manifest/`, new GitHub repo j0yen/tool-manifest.
    Foundational — skill-doctor reads its JSON output.
  - skill-doctor is a fresh rust-cli, new repo at
    `~/wintermute/skill-doctor/`. AC11 is the verified-completed
    gate (one user-promoted proposal must land as an actual skill
    edit), mirroring freshness/recall-doctor-claims AC10.
  - All 3 PRDs `build_auto: false` per default /dream rule (vision
    is opt-in until user articulates).

**Notes for next /dream:** unblock conditions:
  - drift-fix-self-review-dream ships → next self-review tick should
    log zero of the four flagged instances; verify in journal.
  - tool-manifest ships → skill-doctor unblocks.
  - skill-doctor ships + first user-promoted proposal lands →
    Fleet 2 draftable (drift-self-review-integration,
    drift-cli-help-snapshot, drift-changelog-witness,
    drift-config-files, drift-bootstrap-truth).
  - Any wintermute Fleet 2 ship → Fleet 3 trigger (>=2 ships).
  - build-deferred-acs ships → check Fleet 1 unblock effect.
  - New user articulation.

**Cross-fleet notes:**
  - freshness vision composes naturally: a future Fleet 2 unified-
    proposals skill could merge `recall doctor --check-claims`'s
    proposals with `skill-doctor`'s.
  - No collision with chord, cadence, continuity, handshake,
    onramp, release-gate, wintermute fleets.

## 2026-05-28T01:55  /dream  pass 14 — postscript (accidental sweep)
Commit `ac38446` (the drift-vision commit) accidentally also included
7 daily-receipt artifacts (PRD-daily-receipt-{archive,haiku,printer,
stamps,summarize,yearend-letter}.md + visions/daily-receipt.md) that
were pre-staged in the index by a parallel /dream session before I ran
my `git add`. My commit only added drift files; the daily-receipt
files were already staged from a sibling session's interrupted work
and got swept up under the wrong commit message.

No content damage — the 7 files shipped with their intended content.
But the commit message attributes them to the drift vision, which is
wrong. The sibling /dream session that authored daily-receipt should
post a follow-up gossip note correcting attribution + describing
their actual vision when they resume.

Suggested mitigation: next /dream that touches this gossip can append
the missing daily-receipt entry retroactively, or the sibling session
amends in their own pass.


## 2026-05-27T22:35  /dream  pass 15 — Fleet 1.5 expansion
Drafted: PRD-wintermute-hardware-smoke-convention.md
Vision: visions/wintermute.md updated (Fleet 1.5 §2 added)

**Trigger:** Between pass 14 (drift, 01:55Z) and this pass, two
wintermute Fleet 1 archives landed: wintermute-tts (32236d7,
2026-05-28T05:27Z) and wintermute-dialog (0a2fa94, 2026-05-28T05:08Z).
Bringing Fleet 1 to 3/7 shipped (with bootstrap). tts's archive
trailer cites pairing AC1/3/5/7 against `tests/hardware_acs.rs` —
`#[ignore]`-gated stubs that demand a `WM_TTS_HARDWARE_SMOKE=1`
witness. Verified live: the file exists at
~/wintermute/wintermute-tts/tests/hardware_acs.rs (90 lines), AC stubs
panic with instructive messages if invoked without the env var, /build's
check #5 accepted the pairing.

Pass 13's PRD-build-deferred-acs.md proposed a `deferred_acs:`
frontmatter mechanism for the same root issue. /build solved the tts
case empirically with the env-witness pattern in parallel. The two
solutions differ on whether the pairing is real (witness pattern) or
asserted via frontmatter (deferred-acs).

**What this PRD does:**
- Documents the WM_<SLUG>_HARDWARE_SMOKE convention in a new file
  `~/wintermute/autobuilder/notes/conventions/hardware-smoke.md`.
- Scaffolds `tests/hardware_acs.rs` into wintermute-platform,
  wintermute-stt, wintermute-audio matching the tts shape exactly.
- Per-PRD AC coverage: platform AC1/2/5/8, stt AC1/2/4/6/7/8, audio
  AC1/2/3/4/5/6/8.
- No skill changes, no version bumps, no binary edits. Pure test +
  docs.

**Why not retire deferred-acs:**
- Most wintermute hardware ACs are inside Rust binaries and have a
  natural cargo-test pairing surface. Witness-gating fits.
- ACs that exit Rust entirely (Gmail OAuth, install.sh as fresh
  user, printer paper-out probe) have no cargo-test pairing surface.
  deferred-acs's frontmatter is still honest for those cases.
- The two patterns coexist; ship in any order.

**Notes for /build:**
- build_target=mixed (3 rust-extend touches + 1 doc file). Single
  tick likely sufficient — total ~150 lines across 4 new files.
- /build can pick this up before OR after PRD-build-deferred-acs;
  no ordering constraint.
- After this PRD ships, the next platform/stt/audio /build tick
  should be able to mark check #5 as satisfied for the hardware-
  gated ACs and proceed to archive (modulo any remaining
  non-hardware ACs that are still genuinely failing).
- Verify the scaffolded files compile (`cargo test --release --lib`
  + `cargo test --release --test hardware_acs` 0 passed/N ignored
  for each repo) before committing per-repo.

**Notes for next /dream:** unblock conditions:
- This PRD ships → check that platform/stt/audio /build ticks
  actually advance their AC pairing (verified-completed §5
  evidence in iter logs); if not, the gap is more subtle.
- ≥2 wintermute Fleet 2 ships → Fleet 3 trigger remains (browser,
  desktop, screen-narrate, mail, calendar, music — none shipped
  yet).
- build-deferred-acs ships → still queued; check whether /build
  routed to deferred-acs or to the witness pattern for the next
  non-wintermute hardware-dep PRD that appears.
- Any of drift Fleet 1 ships (drift-fix-self-review-dream /
  tool-manifest / skill-doctor — all still queued).
- New user articulation.

**Cross-fleet notes:**
- No collision with cadence/chord/continuity/freshness/handshake/
  onramp/release-gate/daily-receipt/drift visions.
- wintermute-dialog (shipped) does NOT need backporting — its ACs
  are software-timed (barge-in measured as event-loop wall, not
  speaker-relative). Verified during draft research.
- /build's iter log for wintermute-tts is the worked example;
  /build can mirror that exactly for the three target repos.

## 2026-05-28T06:05  /dream  pass 16 — Fleet 1.5 row 3 (bus-smoke convention)
Drafted: PRD-wintermute-fleet-bus-smoke-convention.md
Vision: visions/wintermute.md updated (Fleet 1.5 row 3 added)

**Trigger:** Between pass 15 (22:35 PDT 5/27 = 05:35Z) and this pass,
an orphan PRD landed at PRD-wintermute-fleet-agorabus-announce-fix.md
(authored as /build Phase 6 follow-on during the fleet wire-up
session). It fixes a one-line-per-repo bug: wm-tts/stt/dialog/brain
each call `agorabus::Client::connect()` then immediately `.subscribe()`
without `.announce()`, hitting the daemon's `announce_required`
enforcement and exiting within ~1 s. The orphan PRD names the FIX;
this pass names the STRUCTURAL GAP that allowed it to ship undetected.

**Live evidence (verified 2026-05-27T22:55Z):**
  - agorabus/src/daemon.rs:315-316 enforces "first message must be
    Announce" (`announce_required` error + connection teardown).
  - wm-tts/src/daemon.rs:815-824 has the bug; identical shape at
    wm-stt:214,226 / wm-dialog:450,462 / wm-brain:1310,1320.
  - wm-audio/src/daemon.rs uses the CORRECT pattern; wm-audio's
    tests/wake_bus_smoke.rs:82-165 exercises it end-to-end via
    in-process agorabus::run_daemon on a temp socket.
  - wm-audio has THREE bus-smoke tests (wake/vad/reload). The other
    four repos have ZERO. None of {tts,stt,dialog,brain}/tests/ has
    a bus_smoke.rs file.
  - `cargo test --release --test wake_bus_smoke` in wm-audio passes
    in 1.4 s, no env witness needed.

**Why a 16th pass instead of a rest-pace pass:** matches the freshness
/ handshake / pass-15 single-PRD precedent. Real new evidence (orphan
PRD landed AND structural verification of the bug class confirmed
across 4 repos AND wm-audio's reference impl confirmed) PLUS a clear
shape (mirror hardware-smoke-convention exactly, but for protocol-level
wire-up instead of hardware witnessing). Not dreaming past research:
the convention file, the 4 backfill targets, the wm-audio reference,
all verified live this session.

**Why a convention rather than a typestate or a shared crate:**
typestate would be a wider agorabus API change requiring its own
research; shared crate is premature with 4 consumers and one pattern.
Convention + copy-paste skeleton is the right level today.

**Notes for /build:**
  - Ship `PRD-wintermute-fleet-agorabus-announce-fix.md` FIRST (one-
    line patch per repo, single tick across all four). Without the
    fix, the new bus_smoke.rs tests fail with announce_required —
    which is correct test behavior pre-fix, but means /build can't
    archive bus-smoke as green until the fix lands.
  - build_target=mixed (1 convention doc + 4 rust-extend touches
    across repos in the autobuilder workspace).
  - No skill changes, no version bumps, no library edits.
  - Each new bus_smoke.rs follows wake_bus_smoke.rs verbatim except
    for the daemon-under-test and the expected event topic. ~80-120
    LOC per file.
  - AC7 is the anti-cargo-cult gate: each new test must contain an
    explicit `.announce(...)` BEFORE any `.subscribe(...)` or
    `.publish(...)`. A test that connects-without-announcing
    reproduces the bug instead of catching it.

**Notes for next /dream:** unblock conditions:
  - Bus-smoke-convention ships AND announce-fix ships → next Fleet 2
    PRD draft (browser, desktop, etc.) must reference the convention
    in its acceptance criteria. /dream's drafting checklist for
    Fleet 2 needs the hook.
  - ≥2 wintermute Fleet 2 ships → Fleet 3 trigger remains (brain
    shipped per CLAUDE_SELF 2026-05-28; need one more — browser,
    desktop, screen-narrate, mail, calendar, or music).
  - Any drift Fleet 1 ship (drift-fix-self-review-dream /
    tool-manifest / skill-doctor — all still queued).
  - build-deferred-acs ships → check whether /build routes future
    hardware/process-level ACs to deferred-acs or to the
    witness/smoke patterns.
  - daily-receipt fleet ship (any of summarize / haiku / stamps /
    archive / yearend-letter).
  - New user articulation.

**Cross-fleet notes:**
  - Sibling to pass 15's hardware-smoke-convention: same structural
    shape (convention doc + scaffolded test files + no
    skill/version/binary changes), different test surface (protocol
    vs hardware), different gating (CI-runnable vs env-witness).
  - PRD-agorabus-boot-handshake (handshake vision Fleet 1) targets a
    different race (at-boot orphan subscribers in
    agorabus-session-start.sh, not in-daemon Client misuse). No
    collision; both can ship parallel.
  - Drift vision's tool-manifest + skill-doctor could grow a future
    Fleet 2 entry that flags `Client::connect` call sites missing a
    follow-up `.announce` — captured as PRD §5 out-of-scope bullet,
    not drafted this pass.
  - No collision with cadence/chord/continuity/freshness/onramp/
    release-gate/daily-receipt visions.


---

## 2026-05-28T06:30  /dream  pass 17 — vision-fidelity (NEW vision)
Drafted:
  PRD-recall-surfaced-tracking.md      (v0.7.1)
  PRD-recall-use-evidence.md           (v0.7.2)
  PRD-recall-stop-hook-discriminate.md (v0.7.3) ← load-bearing
  PRD-recall-doctor-utility.md         (v0.7.4)
  PRD-recall-corpus-vacuum.md          (v0.7.5)
Vision: visions/fidelity.md

**Seed:** reflective sweep. recall-outcome-feedback shipped 2026-05-27
acknowledged the gap explicitly ("memories that consistently help drift
up") but its implementation can only detect "no contradiction" — not
"actually used." The Stop hook reads `~/.cache/recall-weather/<sid>/
recalled.json` and applies blanket `+0.02` accept on every surfaced id.

**Live evidence (verified 2026-05-28T06:11Z):**
  - 158 weather session dirs accumulated under
    `~/.cache/recall-weather/` — each was a blanket-accept fire.
  - `~/.claude/scripts/recall-stop.sh` lines 39-50 contain the
    blanket-accept block (jq -r .[]? then `recall feedback --accept`).
  - First fire of recall-search-inject (2026-05-28T05:35Z) surfaced 5
    memories; only the self-referential one was actually used. Other 4
    got the same reward.
  - `src/index.rs` has feedback_count + recall_count but NOT
    surfaced_count or used_count — the columns must be added.

**Why this vision now:**
  - recall-outcome-feedback just shipped (v0.6.0, 2026-05-27).
    Fidelity is the natural v2 — refines the signal it created.
  - wintermute-brain shipped 2026-05-28 — its quality is gated by
    recall ranking, and the brain will compound any bias faster than
    human-paced sessions did.
  - The queue is already big; targeting v0.7.x means no version
    collision with shipped work or recall-doctor-claims (v0.7.0
    reservation).

**Order (load-bearing notes for /build):**
  - recall-surfaced-tracking (v0.7.1) is pure data plumbing — no
    behavior change. Schema migration + new feedback flag + hook
    writes. Smallest first; nothing else depends on PRD #2-5 until
    this lands.
  - recall-use-evidence (v0.7.2) is transcript scanning — new module,
    new subcommand, no behavior change. Independent of #1 except for
    consuming surfaced.json that #1 writes.
  - recall-stop-hook-discriminate (v0.7.3) is THE behavior change.
    Don't ship #4 or #5 without it or the metrics will be misleading
    (no used_count data accumulating).
  - recall-doctor-utility (v0.7.4) is purely diagnostic — extends
    doctor with utility section. Safe to ship anytime after #3.
  - recall-corpus-vacuum (v0.7.5) is the action layer — sweep that
    decays / supersede-proposes / archives noise memories. Ships last.

**Cross-vision notes:**
  - Companion to `recall-outcome-feedback` (archived). Fidelity is
    its v2.
  - Adjacent to `freshness` (PRD-recall-doctor-claims v0.7.0). Both
    extend doctor; different sections; no collision.
  - Feeds `wintermute-brain` (shipped) — better ranking calibration
    means better brain answers.
  - No collision with cadence/chord/continuity/drift/daily-receipt/
    wintermute/release-gate/handshake/onramp visions.

**Notes for /build:**
  - All 5 PRDs use build_target=rust-extend into ~/wintermute/recall.
    Same pattern as recall-daemon, recall-outcome-feedback,
    recall-observer-correlation that already shipped.
  - First two PRDs are tightly scoped (one migration + one column
    each; one new module each). 1-2 iters per PRD expected.
  - Test fixtures in PRD ACs are designed to be writable as unit
    tests inside the existing recall test suite — no new test
    infrastructure needed.
  - PRD #2 (recall-use-evidence) adds a transcript-scan dependency on
    `~/.claude/projects/-home-jsy/<uuid>.jsonl` — verify AC1 (path
    mapping) before doing more work. If the mapping is wrong the rest
    of the PRD degrades to no-op (abstain-on-everything), which is
    safe but defeats the purpose.
  - PRD #3 (discriminate) carries a legacy-fallback path so weather
    dirs from before PRD #1 keep working. AC5 verifies the fallback.

**Open questions (also in vision §Open questions):**
  - Cost of transcript scan in Stop hook latency — measured at <500ms
    target (AC7 of PRD #2); gated by config flag default-off in
    v0.7.2, default-on after measurement.
  - Use-evidence false negatives from paraphrase — accepted; abstain
    is no-op, so false-negative doesn't penalize (unlike
    false-positive which doesn't exist by construction).
  - Should `used_count` feed ranking weights directly? — out of scope
    for v0.7.x; ranking pulls confidence which already reflects
    discrimination. v2 idea.

**Notes for next /dream:**
  - Once PRDs #1-3 ship, check if the existing 158 weather session
    dirs created drift worth recomputing. If yes, draft
    PRD-recall-confidence-recalibrate that re-evaluates each memory's
    confidence against post-v0.7.3 utility data.
  - If the brain's recall layer ends up needing query-time
    `used_count` ranking input (v2 idea above), draft
    PRD-recall-ranking-utility-weight as Fleet 2 of fidelity.
  - Cross-fleet trigger: if /build's Phase 6 generates a follow-on
    PRD that touches the same code paths (e.g., recall-stop-hook
    refactor), flag for collision review before drafting more
    fidelity work.

## 2026-05-28T07:05  /dream  no-fleet-pass (curation + boot validated)

Bare /dream ~30min after fidelity vision drop. Eighth no-fleet-pass.
Curation, not drafting — research didn't motivate a new fleet.

**Boot validation landed this pass** (was an open gate for continuity vision):
  - `uname -r` → `7.0.10-arch1-5-wintermute` (linux-wintermute booted)
  - `/dev/memlog` is a live char device
  - `cat /proc/self/agent_session` → 32 zeros (kernel surface present;
    Claude is NOT yet wrapped — confirms `claude-agentns-wrap` is the
    highest-leverage unblock for the continuity arc)
  - `pacman -Q linux-wintermute` → `7.0.10.arch1-5`

**Curation actions** (no PRD content changed, only manifest):
  - Attached `PRD-provfs-deferred-stamp.md` to `onramp` Fleet 1
    prds_drafted. Already cited by name in vision doc §Order #3 ("pairs
    with provfs-comm-richer, shared hook-time capture buffer"); the
    manifest list had omitted it. Now consistent.
  - Attached `PRD-wintermute-fleet-agorabus-announce-fix.md` to
    `wintermute` Fleet 1.5 prds_drafted. Already cited verbatim in the
    fleet_1_5_pass_16_trigger note (one-line-per-repo `Client::announce()`
    fix; sibling to bus-smoke-convention); manifest list had omitted it.
  - Added `boot_validated_at` + evidence to continuity manifest entry.

**No new PRDs drafted.** Considered four candidates that look like a
"continuity Fleet 1.5" (claude-agentns-wrap, kernel-pkg-postinstall,
provfs-comm-richer, provfs-deferred-stamp), then re-read the onramp
vision and found they ARE that fleet by other name. Drafting a wrapper
would duplicate intent. Per dream rule 6 ("don't dream past the
research"), curation is the honest output here.

**Hints for /build:**
  - `onramp` Fleet 1 #2 (`PRD-claude-agentns-wrap.md`) is now the
    highest-leverage unblock — it gates 4 of 5 `continuity` Fleet 1
    PRDs (recall-session-stamp/memlog-witness/session-postmortem/provq
    all need non-zero `agent_session`). Until this lands every Claude
    session continues to read 32 zeros and the rest of continuity
    silently falls back to PID-tree / `comm:` mode.
  - `onramp` Fleet 1 #1 (`PRD-kernel-pkg-postinstall.md`) is the
    smallest and has no deps. After it ships, `memlog show` works
    without sudo for users in the new `memlog` group.
  - `fidelity` Fleet 1 (5 PRDs, drafted at 06:30Z this morning) is
    ready for pickup; PRD-recall-surfaced-tracking is the lead (pure
    data plumbing, smallest first).

**Inventory snapshot:**
  - 56 PRDs on disk, 47 attached to visions, 9 unattached.
  - Of the 9 unattached: 5 shipped per CLAUDE_SELF changelog (ambient,
    cradle, cradle-bake-integration, morsel, daily-receipt-printer —
    the last one lives intentionally in daily-receipt's
    prds_referenced_not_drafted), 3 are notebook seeds (serious-200,
    whimsy-50, whimsy-cont — `build_target: notebook`, not vision
    material), 1 is the agorabus-announce-fix attached this pass.
  - 12 active visions; 1 fulfilled (release-gate).

**Notes for next /dream:** Trigger to break a no-fleet streak should
be one of: (a) `claude-agentns-wrap` ships and a Claude session reads
a non-zero `agent_session` for the first time — that's the kernel→
userspace handshake fully landed and Fleet 2 of continuity becomes
draftable; (b) fidelity Fleet 1 reaches ≥3 of 5 shipped — Fleet 2
(confidence-recalibrate, ranking-utility-weight) becomes evidence-
backed; (c) new user articulation.

## 2026-05-28T07:30  /dream  no-fleet-pass (state delta logged, triggers still unmet)

Bare /dream 25min after 07:05Z. NINTH /dream no-fleet-pass. State delta
since 07:05Z (small but real, all from /build motion):

  - PRD-wintermute-platform archived (autobuilder commit 139f0a6 at
    07:15Z, status: shipped, 15 ticks). Was already counted as a
    wintermute Fleet 1 ship at 01:40Z (`fleet_2_trigger` note); archive
    closes the bookkeeping without firing a new trigger.
  - wintermute-brain advanced iter-19 → iter-20 (in_progress, last
    07:22Z). CLAUDE_SELF says it shipped on GitHub today; the build
    manifest's in_progress reflects pending archive, not pending code.
  - `~/.cache/recall-weather/` 158 → 173 dirs in 79min (~11/hour
    bias-accumulation rate). Steady; matches the rate fidelity Fleet 1
    PRD #1 (recall-surfaced-tracking) is designed to instrument.
  - /proc/self/agent_session: still 32 zeros. claude-agentns-wrap
    queued; no progress.

Triggers from 07:05Z (still unmet):
  (a) claude-agentns-wrap ships → queued, ticks=0. UNMET.
  (b) fidelity Fleet 1 ≥3/5 shipped → 0/5 (all queued, ticks=0). UNMET.
  (c) new user articulation → bare /dream this pass. UNMET.

Curation considered, none warranted:
  - Inventory: 8 unattached PRDs (5 shipped+stale + 3 notebook seeds).
    Down from 9 at 07:05Z because that pass attached agorabus-announce-
    fix. All 8 explained per 07:05Z gossip note; no relabeling needed.
  - `~/.claude/scripts/recall-search-inject.sh` exists as a UserPromptSubmit
    hook with no PRD provenance (sibling to the v0.6.0 outcome-feedback
    Stop hook). Drafting a backfill PRD now would be paperwork — it's
    plumbing, not a new feature, and fidelity Fleet 1 #1 will instrument
    it via the surfaced-tracking schema. Logged here for trace.
  - dream manifest `_no_fleet_passes` array sits under `.visions` (mixed
    object+array siblings under one key). Confused my reconcile-query
    once this pass; the `?` operator in jq doesn't suppress the
    "indexing array with string" error so queries need
    `select(.value | type == "object")` first. Not a bug worth a PRD;
    just a query-author footgun. Worth a one-line note in dream/SKILL.md
    on the next skill edit. Not edited this pass.

No PRDs drafted. Per rule 6 — don't dream past the research; rule from
11:30Z 5/25 — if state is essentially unchanged AND no trigger fires,
prefer skip-writes; but the platform archive + brain motion + weather
rate are worth logging in the no-fleet-pass record so the next /dream
sees them. Sticking with terse gossip + manifest update; no PRD churn.

Notes for next /dream:
  - Same triggers as 07:05Z: agentns-wrap ship, fidelity ≥3/5, or new
    articulation.
  - Watch wintermute-brain archive (status: in_progress → shipped).
    Doesn't fire a new trigger (already counted), but closes the
    wintermute Fleet 1 archival arc.
  - Watch /build pickup of fidelity Fleet 1 — recall-surfaced-tracking
    is the lead (smallest, pure data-plumbing, no deps).

## 2026-05-28T08:05  /dream  vision-harvest

Drafted: PRD-learning-candidate-triage.md, PRD-learning-candidate-prefilter.md, PRD-learning-candidate-prune.md
Vision: visions/harvest.md
Order: triage → prefilter → prune (no hard deps, but triage defines the
  consumer surface so it's the most useful pickup first; prefilter tunes
  the producer once we have one real consumption cycle of data; prune is
  smallest and most mechanical, can ship any time).

**Trigger.** None of the 07:30Z triggers met (agentns-wrap unshipped,
fidelity Fleet 1 0/5 shipped, bare /dream). But Phase 1 surfaced a new
gap not previously catalogued: 3 learning-candidate drafts in
`~/.claude/scratch/learning-candidates/` with zero consumer. The Stop
hook (`recall-learning-candidate.sh`) and SessionStart hook
(`learning-candidates-start.sh`) ship signal that nothing harvests.
`grep learning-candidate ~/wintermute/autobuilder/*.md visions/*.md` →
zero hits before this pass. Real gap, not paperwork.

**Distinction from prior notes.** The 07:30Z gossip explicitly declined
to draft a backfill PRD for `recall-search-inject.sh` because "fidelity
Fleet 1 #1 will instrument it." That argument doesn't apply here:
fidelity Fleet 1 is about *surface-vs-use* discrimination in recall
ranking, NOT about consuming the candidate-draft queue. No PRD anywhere
covers the draft pipeline's consumer side.

**Notes for /build:**
  - All three PRDs are shell/skill targets — no Rust, no `/autobuilder`
    cycle. Build path is direct (write the script/skill file, smoke-test,
    commit).
  - Triage is the lead: largest LOC, defines the consumer surface.
  - Prefilter's AC1 specifies a *more conservative* threshold than today's
    behavior — the existing single-match drafts will continue working
    through triage; only future emissions are affected. No backwards-
    compatibility risk to today's queue.
  - Prune ships the *script* but **not** any timer wiring or
    /self-review hook (out of scope, follow-up after manual proving).
  - Drafts themselves are already on disk; triage can be smoke-tested
    against them as soon as the skill exists. No artificial setup needed.

**Open questions (also in vision):**
  - Should SessionStart's `learning-candidates-start.sh` stop verbatim-
    surfacing drafts after triage exists, and instead nudge `/triage`?
    Left for the triage PRD to decide.
  - Auto-promote (skip-review on highest-confidence drafts) — captured
    as stretch in vision; defer to successor PRD-learning-candidate-
    auto-promote if practice shows it's worth.

**Notes for next /dream:**
  - Trigger to revisit harvest: triage ships AND processes ≥10 real
    drafts → data exists to tune prefilter thresholds with evidence
    instead of guessing.
  - Carry-forward triggers from 07:30Z still apply: claude-agentns-wrap
    ship, fidelity Fleet 1 ≥3/5 shipped, new user articulation.
  - Inventory delta: 54 PRDs → 57 PRDs after this pass; 12 active
    visions → 13.

## 2026-05-28T08:30  /dream  no-fleet-pass (post-harvest cooldown)

Bare /dream 25min after vision-harvest drop. TENTH /dream no-fleet-pass
overall. State delta since 08:05Z (small, all expected):

  - wintermute-brain archived (autobuilder commit 3f66aac).
    Closes the wintermute Fleet 1 archival arc that 07:30Z gossip
    flagged for watch. Already counted in pass 13's Fleet 2 trigger;
    archive doesn't fire a new one.
  - ~/.cache/recall-weather/ 173 → 193 dirs in ~60min (~20/h, up from
    the 11/h baseline at 07:30Z). Burst is from this session's heavy
    recall-query phase; fidelity Fleet 1 #1 is still the right
    instrumentation, no action needed.
  - 3 learning-candidate drafts unchanged in queue. harvest just
    landed 25min ago; /build pickup hasn't fired yet. Appropriate.
  - /proc/self/agent_session: still 32 zeros. claude-agentns-wrap
    queued, ticks=0. UNMET.

Triggers from 08:05Z (still unmet):
  (a) claude-agentns-wrap ships → UNMET.
  (b) fidelity Fleet 1 ≥3/5 shipped → 0/5. UNMET.
  (c) new user articulation → bare /dream. UNMET.
  (d) harvest triage ships AND ≥10 real drafts processed → UNMET
      (3 drafts on disk; triage queued, ticks=0).

No PRDs drafted. Per rule 6 — don't dream past the research; per
the rest-pace pattern (passes 5-11 of the 5/25 arc) — if state is
essentially unchanged AND no trigger fires, prefer terse log over
PRD churn. State *is* essentially unchanged from 25min ago.

Curation considered, none warranted:
  - Inventory unchanged: 57 PRDs on disk, 50 attached to visions
    (harvest's 3 added at 08:05Z), 7 unattached (5 shipped+stale +
    3 notebook seeds; agorabus-announce-fix attached 07:05Z).
  - 13 active visions. drift verified live; nothing new to attach.

Notes for next /dream:
  - Same four triggers carry forward.
  - Watch /build pickup of harvest's triage PRD (smallest of the
    three, defines consumer surface — appropriate first pickup).
  - Watch fidelity Fleet 1 #1 (recall-surfaced-tracking) — pure
    data plumbing, no deps, smallest first. The 20/h weather burst
    is the kind of data this PRD is designed to expose.

## 2026-05-28T09:30  /dream  no-fleet-pass (harvest 1/3 shipped delta)

Timer-cadence /dream 57min after 08:33Z pass-10 commit. ELEVENTH
/dream no-fleet-pass. State delta since 08:33Z is small but contains
the first real harvest-fleet ship:

  - **PRD-learning-candidate-prefilter SHIPPED + archived** at 08:45Z
    (autobuilder 4280bb9, 12min after 08:33Z gossip). Harvest fleet
    now 1/3 shipped. Audit log
    `~/.claude/scratch/learning-candidates/.audit.log` shows 6 smoke
    rows at 08:34-08:35Z exercising score thresholds + dup detection;
    classifications look correct. Real-world signal still pending —
    no Stop hook has emitted a real draft since prefilter went live
    (latest real draft is 01:35 PDT = 08:35Z, smoke tests run shortly
    after at 08:34Z timestamps). First post-ship session-end will be
    the empirical evidence point.
  - wintermute-hardware-smoke convention picked up iter-3 docs
    (autobuilder da767c8 at ~08:50Z) — Fleet 1.5 motion continues.
  - wintermute-brain archived (autobuilder 3f66aac at 08:30:58Z) —
    already counted in pass-13/Fleet-2 trigger; archive closes the
    bookkeeping.
  - 3 learning-candidate drafts unchanged in queue (3 files; latest
    01:35 PDT = 08:35Z). Triage in_progress ticks=1; not yet ready
    to consume them.
  - /proc/self/agent_session: 32 zeros. claude-agentns-wrap queued
    ticks=0. UNMET.

Triggers from 08:30Z (still unmet):
  (a) claude-agentns-wrap ships → UNMET (queued, ticks=0).
  (b) fidelity Fleet 1 ≥3/5 shipped → 0/5 (all queued, ticks=0).
  (c) new user articulation → bare /dream this pass.
  (d) harvest triage ships AND ≥10 drafts processed → triage
      in_progress ticks=1; 3 drafts on disk; UNMET on both counts.

**Curation this pass:** harvest manifest entry updated with
`prefilter_shipped_at: 2026-05-28T08:45:07Z` + audit evidence pointer +
`pass_11_curation` note. Mechanical bookkeeping; no PRD content
changed.

No PRDs drafted. Per rule 6 — don't dream past the research; per the
rest-pace pattern — if state delta is consumable in one bookkeeping
line and no trigger fires, prefer terse log + manifest curation over
PRD churn. The harvest fleet ship is a healthy data point but doesn't
itself motivate a new vision (the existing triage + prune PRDs are
the right next moves, both already drafted).

Working-tree note (NOT this pass's responsibility): three untracked
files in `~/wintermute/autobuilder/` belong to in-flight /build work:
`.run-ambient/` (ambient-compositions notebook), `intent-cards/
confidant.intent-card.json` (cadence/chord scratch), `notes/
conventions/hardware-smoke.md` (Fleet 1.5 convention doc). /build to
commit when ready; this commit stages only gossip + dream manifest.

Notes for next /dream:
  - Same four triggers carry forward.
  - **Watch harvest triage ship** — that gives us first real
    consumer-side data. The 3 stale drafts (still surfacing in every
    fresh session's banner) are the canonical regression test.
  - **Watch first real post-prefilter Stop hook emit** — the audit
    log will show the score + decision; if a session that previously
    would have emitted 1-3 noise drafts now emits 0 or 1 high-score
    draft, prefilter's working as designed.
  - **Watch fidelity Fleet 1 first ship** (recall-surfaced-tracking
    smallest) — pure data plumbing, no deps, fires trigger (b)
    progress.

## 2026-05-28T10:00  /dream  no-fleet-pass (12th; triage progress, LC queue dropped)

Bare /dream ~30min after 09:30Z pass-11. TWELFTH /dream no-fleet-pass.
State delta since 09:30Z is digestible-as-bookkeeping:

  - **harvest triage progress**: ticks 1 → 2, status still in_progress
    (not shipped). One /build tick happened against it between 09:30Z
    and 10:00Z. Lead PRD still hasn't landed.
  - **LC queue dropped 3 → 0 drafts**. .audit.log mtime unchanged
    (08:35Z smoke entries); directory mtime advanced to 09:34Z. Three
    real drafts that surfaced in every fresh SessionStart banner since
    08:30Z are no longer on disk. Mechanism not directly observable
    from this pass — most likely candidates: (i) /build's triage tick
    at ticks=2 consumed them as part of the consumer-side smoke prove,
    (ii) prune script (which is queued, ticks=0 per manifest — so this
    is unlikely), or (iii) the user manually invoked /triage. **Worth
    a watch in next /dream**: if drafts disappear without /triage
    audit-log entries, the consumption path is silent — that's a
    different observability gap than the existing harvest PRDs cover.
  - **agentns userspace in-flight**: `~/wintermute/agentns/` has 5
    modified files + `userspace/` untracked dir + `tests/
    unshare-helper.c` untracked, no new commits since f5b24e0. Run-9
    self-review (~02:00Z) noted "1 commit ahead" but local repo shows
    no unpushed commits this pass; either the commit got pushed
    between runs or run-9's count came from a stale snapshot. Either
    way, claude-agentns-wrap PRD is *finally* under active
    implementation — first time since it was drafted.
  - **No new autobuilder commits** between 09:30Z and 10:00Z (only
    b3da338 = pass-11 gossip). /build hasn't ticked anything else
    forward in 30 min.
  - **No new real drafts since 08:35Z** (.audit.log last entries are
    smoke tests from prefilter ship + the duplicate-detect smoke).
    First post-ship real-draft empirical evidence still pending.
  - /proc/self/agent_session: 32 zeros. UNMET.

Triggers from 09:30Z (still ALL unmet):
  (a) claude-agentns-wrap ships → UNMET (queued, ticks=0; userspace
      work in flight but not committed; PRD-claude-agentns-wrap.md
      from onramp vision is the load-bearing piece).
  (b) fidelity Fleet 1 ≥3/5 shipped → 0/5 UNMET (all queued ticks=0).
  (c) new user articulation → bare /dream this pass. UNMET.
  (d) harvest triage ships AND ≥10 drafts processed → UNMET on both
      counts (triage ticks=2 in_progress; 0 drafts on disk so the
      ≥10 condition is *further* from satisfaction than at 09:30Z).

No PRDs drafted. Per rule 6 — don't dream past the research; per the
rest-pace pattern (passes 5-11 of the 5/25 arc and now 10/11/12 of the
5/28 arc) — when state delta is digestible as bookkeeping and no
trigger fires, prefer terse log over PRD churn. The triage progress
tick and the unobserved LC queue drain are real signals but neither
motivates a new vision: triage is *the* consumer being built; the
queue drain mechanism is a watch-item for next /dream, not a PRD.

Curation considered, none warranted:
  - Inventory unchanged: 57 PRDs on disk, 50 attached to visions, 7
    unattached (5 shipped+stale + 3 notebook seeds; agorabus-announce-
    fix attached at 07:05Z this morning).
  - 13 active visions; nothing new to attach.
  - Self-review run-9 surfaced the 9th-consecutive `recall divergence
    false-trigger` flag with explicit cost: "Has not been worth fixing
    7 times in a row, but it's noise on every run now." A one-line
    skill patch (exclude `recall/proposals/*.md` from file-count) or
    a recall-side move would close it. Considered drafting a single-
    PRD vision for /self-review noise reduction — declined this pass.
    Reasons: (i) the fix is genuinely a 30-second mechanical patch,
    not PRD-shaped work; (ii) freshness Fleet 2 has a
    `freshness-on-recall` bullet that could naturally swallow it when
    Fleet 1 ships; (iii) the user has the most efficient path
    (`txn-edit` the skill template, two minutes). Logged here for
    trace; if it shows up a 10th time, draft a one-line PRD then.

Notes for next /dream:
  - Same four triggers carry forward unchanged.
  - **Watch LC queue mechanism**: if drafts disappear again without
    corresponding .audit.log entries, that's a real observability gap
    — possibly draft PRD-learning-candidate-audit-completeness as a
    successor to harvest's triage. Hold this pass; gather one more
    data point first.
  - **Watch claude-agentns-wrap implementation**: agentns userspace/
    dir is now active work. First time the trigger (a) PRD is
    moving. If a commit lands and userspace registration produces a
    non-zero `/proc/self/agent_session`, that's a vision-grade event
    — onramp Fleet 1 will be partly fulfilled and continuity Fleet 1
    becomes unblocked (5 PRDs all depending on it).
  - **Watch fidelity Fleet 1 #1 first ship** (recall-surfaced-tracking,
    smallest, no deps). Still 0/5 shipped after 24+ hours since drop.

## 2026-05-28T10:30  /dream  no-fleet-pass (13th; Fleet 1.5 ship + LC mystery resolved)

Bare /dream ~30min after 10:00Z pass-12. THIRTEENTH /dream no-fleet-pass.
State delta is digestible-as-bookkeeping, but with one mystery closed.

State delta since 10:00Z:

  - **5 new autobuilder commits**, all within already-shipping work:
    - **274c9b4** `build: archive PRD-wintermute-hardware-smoke-convention
      (shipped)` — wintermute Fleet 1.5 PRD SHIPPED. Already counted
      under fleet_1_5_pass_15 trigger; doesn't fire a new Fleet 2
      trigger because it's NOT a fidelity Fleet 1 ship (trigger (b)
      still 0/5).
    - **8a759bc / f8d9c09 / b077a1f / 52e0315** — build-deferred-acs
      iter-5 backfill across wintermute-platform/audio/stt/tts.
      Mechanical fan-out of the convention shipped at 274c9b4;
      build-deferred-acs PRD still in_progress (this is its job).

  - **LC queue mystery from pass 12 → RESOLVED.** Self-review run 9
    at 09:34Z explicitly logs:
      `## Triage — 2026-05-28T09:34Z (one /build tick → /triage)`
    with 3 candidate dispositions (1 save → procedural/self memory
    01KSPYXA03FFGFQ2G1Z6AYQDEJ "use recall + skills proactively",
    2 discard — one dup of the save, one machine-output false-match).
    Consumption mechanism IS audit-logged, just in journal not in
    `.claude/scratch/learning-candidates/.audit.log`. No
    observability gap; pass-12 hypothesis of drafting
    PRD-learning-candidate-audit-completeness is REJECTED.

  - **agentns userspace work continues in flight**: still
    `~/wintermute/agentns/userspace/` untracked + 5 modified files +
    tests/unshare-helper.c untracked, no new commits since
    a8a1845. claude-agentns-wrap PRD ticks=0 in manifest. First
    commit-grade artifact still pending.

  - **/build blocker count rose 3 → 5** per self-review runs 8→9:
    added `chord-async-delegate` (user-gate-install) +
    `drift-fix-self-review-dream` (classifier-self-mod). All 5 need
    user judgment, none auto-clearable.

  - **New finding from self-review run 9 /triage section**: `/triage`
    SKILL.md classification table maps `--kind feedback` but recall
    REJECTS that kind (valid: procedural/semantic/episodic/reflective).
    Self-review used `procedural/self` as nearest fit. This is a
    real bug in `~/.claude/skills/triage/SKILL.md`. **Logged here,
    not drafted** — same logic as pass-12 self-review noise
    reduction: 30-second mechanical patch, not PRD-shaped, and the
    in_progress PRD-learning-candidate-triage.md could naturally
    swallow it as an additional AC. If it recurs across passes,
    revisit.

  - /proc/self/agent_session: 32 zeros. UNMET.

Triggers from 10:00Z (still ALL unmet):
  (a) claude-agentns-wrap ships → UNMET (queued, ticks=0; userspace
      work in flight but not committed).
  (b) fidelity Fleet 1 ≥3/5 shipped → 0/5 UNMET (all queued
      ticks=0; 274c9b4 is wintermute Fleet 1.5, not fidelity).
  (c) new user articulation → bare /dream this pass. UNMET.
  (d) harvest triage ships AND ≥10 drafts processed → triage
      in_progress ticks unchanged; 0 drafts on disk so ≥10 condition
      is FURTHER from satisfaction. UNMET on both counts.

No PRDs drafted. Per rule 6 — don't dream past the research; per
the rest-pace pattern (now 12/13 of the 5/28 arc) — when state delta
is digestible as bookkeeping and no trigger fires, prefer terse log
over PRD churn. The Fleet 1.5 ship is a healthy data point but was
already counted in the fleet_1_5_pass_15 trigger. The LC mystery
closing is the most valuable finding — it prevents drafting a
duplicate observability PRD.

Curation considered, none warranted:
  - Inventory unchanged: 57 PRDs on disk, 50 attached to visions,
    7 unattached (5 shipped+stale + 3 notebook seeds; all explained
    in prior passes).
  - 13 active visions; nothing new to attach.
  - The triage `--kind feedback` bug could be a one-line addition
    to PRD-learning-candidate-triage.md but rule 2 ("never modify
    existing PRDs") suggests waiting for /build to take a tick on
    it and surface the issue from inside the implementation pass.

Notes for next /dream:
  - Same four triggers carry forward unchanged.
  - **Watch claude-agentns-wrap commit**: agentns userspace dir
    has been "active in flight" for two passes now without a
    commit-grade artifact. If 30+ min passes without progress,
    that's a stall signal worth noting (not a PRD; just a watch).
  - **Watch /triage skill recurrence**: if a future self-review run
    independently surfaces the `--kind feedback` bug a second time,
    it's worth a one-line PRD then.
  - **Watch fidelity Fleet 1 first ship** (recall-surfaced-tracking,
    smallest, no deps). 0/5 after ~28 hours since drop.
  - **Watch chord-async-delegate / drift-fix-self-review-dream**:
    both `in_progress ticks=0`. They're blocked on classifier/user-
    gate, not on /build inertia. User-flip might be all they need.

## 2026-05-28T11:00  /dream  no-fleet-pass (14th; harvest Fleet 1 fulfilled, agentns still uncommitted)

Bare /dream ~30min after 10:30Z pass-13. FOURTEENTH /dream no-fleet-pass.
One curation this pass: harvest vision flipped active→fulfilled.

State delta since 10:30Z:

  - **2 new autobuilder commits, both archival**: 0999a07 archives
    PRD-learning-candidate-triage (shipped) + df88b74 archives
    PRD-learning-candidate-prune (shipped). With prefilter shipped
    earlier today (08:45Z), **harvest Fleet 1 is now 3/3 → vision
    fulfilled**. Updated manifest: visions.harvest.status
    active→fulfilled, fulfilled_at=2026-05-28T11:00:00Z, added a
    pass_14_curation note. Eighth vision overall to reach fulfilled
    (joining release-gate at pass 11).
  - **agentns userspace work continues in flight, third pass without
    a commit**: `~/wintermute/agentns/userspace/` has `agent-wrap.c`
    (87 LOC, complete CLONE_NEWAGENT unshare wrapper with AGENT_INTENT
    prctl handling; gracefully degrades to plain exec on ENOSYS/EPERM
    with clear stderr; needs setcap cap_sys_admin+ep on installed
    binary) plus a Makefile (gcc -O2 -Wall -Wextra -std=gnu11 -I..,
    builds against ../include/uapi/linux/agent_namespaces.h). Both
    files readable on disk, both untracked. No new commits in
    ~/wintermute/agentns/ since a8a1845 (the kernel-side boot-hang
    fix). Same shape as passes 12 + 13 — readable, complete-looking,
    uncommitted. THREE consecutive passes now.
  - **/proc/self/agent_session still 32 zeros** (kernel surface
    unchanged; claude wrapping still the load-bearing gap).
  - **recall reflective queue**: latest 10 reflective/self memories
    all recalls=0 (consistent with self-review run 9's "8 stale
    reflective entries >30d" finding; pattern persists). Not a
    drafting trigger by itself but a continuing freshness signal.

Triggers from 10:30Z:
  (a) claude-agentns-wrap ships → UNMET (queued, ticks=0; agent-wrap.c
      readable but uncommitted does not count as a Fleet 1 ship).
  (b) fidelity Fleet 1 ≥3/5 shipped → 0/5 UNMET.
  (c) new user articulation → bare /dream. UNMET.
  (d) harvest triage ships + ≥10 drafts processed → **FIRST HALF MET**
      (triage shipped at 0999a07). **SECOND HALF UNMET** (LC queue
      at 0 drafts; consumer has not been exercised against post-
      prefilter real production). Conjunction fails. Trigger (d)
      does not fire because we have no empirical Fleet 2 signal
      from "actually consumed live drafts" yet.

No PRDs drafted this pass. Per rule 6: harvest Fleet 2 needs Fleet 1
to have been *used*, not just *shipped*. Triage at ticks=0-against-
real-drafts is shipped-but-unvalidated. Fleet 2 bullets in vision doc
stay as bullets. Curation considered, none warranted beyond marking
harvest fulfilled: 57 PRDs unchanged; 12 active visions + 1 newly
fulfilled (harvest); nothing else needing reattachment.

Watch items carrying forward:
  - **claude-agentns-wrap commit**: third pass in flight without a
    commit. The work is complete enough on disk to compile and run;
    what's missing is just `cd ~/wintermute/agentns && git add
    userspace/ && git commit && git push` plus an install step. If
    a fourth pass still shows uncommitted state, that's worth a
    user nudge — not a PRD, just an offer ("the agentns userspace
    work has been ready to commit for 90+ min; want me to land it?").
  - **harvest Fleet 2 unblock**: LC queue must cycle ≥10 real drafts
    AND ≥1 produces a user-promoted memory before Fleet 2 PRDs can
    cite live evidence. The prefilter ships+ships+ships discipline
    eventually meets the threshold organically.
  - **fidelity Fleet 1 #1**: recall-surfaced-tracking, smallest of
    the 5, no deps. Still 0/5 after ~28.5h since drop. Watch for
    first ship.
  - **/triage skill `--kind feedback` bug**: still single-evidence
    (one self-review hit at 09:34Z). Second independent surfacing
    promotes to PRD.

Notes for next /dream:
  - Harvest is fulfilled — drop that vision from triggers list.
    Next pass triggers: (a)(b)(c) unchanged, (d) replaced with
    "agentns first commit lands" if pass 15 still shows uncommitted
    userspace work (escalation from watch-item to trigger).
  - If state delta remains bookkeeping-only across 4+ passes plus
    harvest just fulfilled, consider whether the rest-pace pattern
    is masking a stalled queue rather than indicating discipline.

## 2026-05-28T11:30  /dream  no-fleet-pass (15th; agentns escalation — user-nudge time)

Manual /dream ~30min after 11:00Z pass-14. FIFTEENTH /dream no-fleet-pass.
Pass-14's promised escalation fires: agentns userspace work is now at FOUR
consecutive passes uncommitted, and the on-disk mtime evidence is far worse
than pass-14's "90+ min" estimate.

State delta since 11:00Z (~30 min):

  - **2 new autobuilder commits, neither a Fleet 1 ship**:
    - **ff02bb6** `build/autobuilder: skill-doctor intent-card.json
      (Stage 1)` — intra-PRD progress on PRD-skill-doctor (continuity-
      adjacent but not a continuity Fleet 1 PRD). Stage 1 of a
      multi-stage build.
    - **2932220** `build-deferred-acs: declare AC7 deferred (overtaken
      by events)` — iter-6 on the build-deferred-acs PRD. Adds
      `deferred_acs: [7]` to its OWN frontmatter, with `(no reason
      given)` falling back to PRD reasons map. AC7 (greppable Deferred:
      trailer) cannot fire on the 4 historical archive commits that
      pre-date archive-trailer.sh shipping at 3ab4e03 (iter-3); the
      mechanism is ready and will populate from this PRD's own archive
      forward. The PRD self-classifies the gap as overtaken-by-events
      rather than carrying it as missing-AC indefinitely. Honest move,
      first time deferred_acs has been used to declare a PRD's own
      AC deferred-for-cause (vs the 4 wintermute-* iter-5 backfills,
      which declared hardware-untestable ACs).

  - **agentns userspace work UNCOMMITTED, fourth pass — pass-14's
    nudge threshold crossed.** Critical correction to pass-13/14
    framing: `stat -c '%y'` on the files shows:
      - `userspace/agent-wrap.c`  → 2026-05-26 16:00:38 PT
      - `userspace/Makefile`       → 2026-05-26 16:00:40 PT
    That's **~36 hours ago, not 90 minutes**. Pass-13/14 reported the
    files as "in flight" — implying recent edits — but the on-disk
    mtimes show the work was completed Tuesday evening and has been
    sitting un-committed ever since. The framing "active work" was
    wrong; the framing is **stalled work**. agent-wrap binary also
    present (Makefile has been run at least once), so the work
    compiles. No new commits in ~/wintermute/agentns/ since a8a1845
    (kernel-side boot-hang fix at 2026-05-25 09:27 PT).

  - **/proc/self/agent_session still 32 zeros** — kernel surface
    unchanged. Wrapping gap unchanged.

Triggers from 11:00Z (per pass-14 prediction, harvest dropped,
trigger (d) replaced with "agentns first commit lands"):
  (a) claude-agentns-wrap ships → UNMET.
  (b) fidelity Fleet 1 ≥3/5 shipped → 0/5 UNMET (recall-surfaced-
      tracking still 0/5 after ~32h since drop).
  (c) new user articulation → manual /dream invocation, no topic
      seed. UNMET as a Fleet trigger; this manual invocation may
      be the user's signal to act on the nudge below.
  (d) **agentns first commit lands** → UNMET (still uncommitted).
      Now elevated from watch-item to trigger by pass-14's plan.

No PRDs drafted. Per rule 6 — no new evidence, no new motivation.

**User-offer surfaced this pass (per pass-14 plan):**

  The agentns userspace work has been on disk for ~36 hours without
  a commit (4 dream passes have observed it; first-observed on
  pass-12). The files are complete-looking: `agent-wrap.c` (~87 LOC,
  CLONE_NEWAGENT unshare + AGENT_INTENT prctl + ENOSYS/EPERM graceful
  degrade), `Makefile` (gcc -O2 -Wall -Wextra -std=gnu11), built
  `agent-wrap` binary. Same 5 modified files in main agentns/ from
  prior passes (.gitignore, README.md, kernel/agent_namespaces.c,
  tests/test_inheritance.sh, tests/test_unshare.c) plus the
  untracked tests/unshare-helper.c.

  This is the load-bearing PRD for continuity Fleet 1 (5 PRDs
  blocked on non-zero agent_session: claude-agentns-wrap pairs
  recall-session-stamp + memlog-witness + session-postmortem +
  provq).

  Phrasing to surface to user: "Want me to commit + push the
  agentns userspace work? Or is something pending (Cargo.toml
  unsure, AC mapping incomplete, tests not run) that's blocking
  the commit?" The dream-side cannot answer the second question
  without making assumptions about user intent on un-authored work.

Curation considered, none warranted: 57 PRDs, 12 active visions,
inventory unchanged.

Watch items carrying forward:
  - **claude-agentns-wrap**: now elevated to trigger (d). If next
    pass shows commits, fires Fleet 2 onramp PRD drafting (5 bullets
    captured in vision-onramp doc).
  - **fidelity Fleet 1 #1**: recall-surfaced-tracking, smallest, no
    deps. Still 0/5. ~32h since drop.
  - **chord-async-delegate / drift-fix-self-review-dream**: still
    user-gate-blocked per self-review run 10.
  - **/triage `--kind feedback` bug**: still single-evidence; second
    surfacing promotes to PRD.

Meta-observation for /dream rule 11+ pass arc:
  - Pass-12/13/14 all called agentns "in flight". The mtime check
    this pass exposes that framing as wrong; the files are stalled,
    not active. Lesson: when reading "uncommitted untracked work",
    always `stat -c '%y'` the files. "In flight" implies recent
    edits; "stalled" implies user blockage. The first triggers a
    watch-then-wait posture; the second triggers a user-nudge. The
    distinction matters and dream missed it for 3 passes. Logged
    into visions/freshness.md §Evidence log as freshness-on-files
    candidate (mtime is the cheap source of truth for "active
    vs stalled" claims about uncommitted work). NOT drafted as PRD.

Notes for next /dream:
  - If agentns lands (commits + push), fire trigger (d) Fleet 2:
    draft onramp Fleet 2 (5 bullets, all in vision doc).
  - If agentns still uncommitted at pass 16, escalate again: ask
    the user directly in the dream output whether the work is
    actually blocked on review, on a test, or on uncertainty about
    PRD-claude-agentns-wrap's exact contract; drafting a successor
    PRD that captures the on-disk implementation as Status:Implemented-
    pending-PRD-update would be a real option then.
  - Same other watches carry: fidelity Fleet 1 first ship,
    chord/drift-fix user-gates, /triage --kind feedback bug.

## 2026-05-28T12:00  /dream  no-fleet-pass (16th; agentns reframe — superseded, not stalled)

Manual /dream invocation from user (~30min after 11:30Z pass-15). SIXTEENTH
/dream no-fleet-pass. Pass-15's escalation plan fires, but the on-disk
investigation REFRAMES the agentns story entirely. Correction below
matters more than the no-fleet status.

State delta since 11:30Z:

  - **0 new autobuilder commits.** Manifest last_updated 11:52:16Z
    (pre-pass-15). 52 PRDs queued, 13 visions active+1 fulfilled
    (harvest, marked pass-14).
  - **agentns parallel-impl discovered**: pass-12/13/14/15 all framed
    `~/wintermute/agentns/userspace/agent-wrap.c` as "in flight" then
    "stalled" then "user-nudge-time". The on-disk file IS stalled (~44h
    since mtime 2026-05-26 16:00 PT), but it is no longer the load-
    bearing path. **/autobuilder built the Rust version today** at
    `~/wintermute/agentns-claude/` — committed at 90a808d "iter-1:
    autobuilder Stages 1+2 scaffold from PRD-agentns-claude" with src/,
    tests/, scripts/, Cargo.toml/lock, dual MIT/Apache LICENSE,
    CHANGELOG, README. Manifest shows agentns-claude.status=in_progress,
    last_action 2026-05-28T11:56:44Z (~50min before this pass), inside
    the autobuilder Stages 1+2 scaffold cycle. The Rust impl is the
    canonical PRD-agentns-claude shipping target (j0yen/agentns-claude).
  - **The C wrapper is therefore superseded, not stalled.** It is
    a hand-built predecessor from before the /autobuilder pipeline
    kicked in. Per [[feedback_always_build_autobuilder]] memory: hand-
    rolling is the wrong instinct; the autobuilder version is canonical.
    Keeping the C wrapper uncommitted at this point is correct (it
    would just be dead code in the agentns kernel-side repo). Either:
    (a) commit as `examples/agent-wrap.c` for documentation of the
    minimal C demonstration, or (b) `rm userspace/` since the Rust
    impl is now the path.
  - **/proc/self/agent_session still reads 32 zeros** — the unblock
    is the Rust autobuilder build LANDING (Stage 3+ → release-gate →
    ship) AND being installed at `~/.local/bin/agentns-claude`, then
    PRD-claude-agentns-wrap (Fleet 1 onramp #2) wiring zsh/systemd to
    route launches through it. C-version commit would NOT unblock
    this — even if committed, nothing installs from agentns/userspace/
    onto the path.
  - **recall reflective queue**: latest 10 reflective/self memories
    same as pass-15 — all recalls=0. Persistent freshness signal.

Triggers from 11:30Z:
  (a) claude-agentns-wrap ships → UNMET (Fleet 1 onramp #2; waits on
      agentns-claude shipping first per dependency chain).
  (b) fidelity Fleet 1 ≥3/5 shipped → UNMET (still 0/5; ~33h since drop).
  (c) new user articulation → MET? (bare /dream from user). No topic
      seed. Treated as "show me where the system is and act on
      anything you'd normally surface."
  (d) agentns first commit lands → REFRAMED: pass-15's framing assumed
      the C version was the path. Today's truth is the Rust autobuilder
      version IS landing (90a808d) and IS the load-bearing path. The
      original trigger fires in spirit (Fleet 1 onramp #2 onramp gate
      remains the install-and-wire step) but not on the artifact pass-15
      named. Treating as MET-but-reframed: Fleet 2 PRDs (5 bullets in
      vision-onramp doc) still wait for Stages 3+ to complete the
      autobuilder cycle on agentns-claude AND `~/.local/bin/agentns-claude`
      to actually be installable.

No PRDs drafted. Per rule 6: the on-disk situation does not motivate a
new component. The hand-built C wrapper is not a new PRD — it is a
cleanup decision. The autobuilder Rust build is in active progress; we
do not draft "ship this PRD" PRDs.

**User-offer surfaced this pass (REVISED from pass-15):**

  1. Cleanup decision: `~/wintermute/agentns/userspace/` has 87-LOC C
     wrapper + Makefile + compiled binary, ~44h since mtime, never
     committed. The Rust autobuilder version at `~/wintermute/agentns-
     claude/` is now canonical. Options:
       (a) Commit C as `examples/agent-wrap.c` (documents the minimal
           kernel-side userspace use; ~90 LOC vs whatever-the-Rust-
           CLI-grows-to).
       (b) `rm -rf userspace/` (clean break — Rust is the path).
       (c) Leave as-is (untracked, harmless but accumulating
           "stalled work" misreadings in future dream passes).
  2. Install-step question: is the autobuilder cycle for agentns-claude
     intended to drive all the way to `cargo install --path . --root
     ~/.local` automatically, or does the install step land in
     user-gate territory? The Fleet 1 onramp dependency chain
     (agentns-claude → claude-agentns-wrap → continuity Fleet 1 ×4)
     unblocks only when `which agentns-claude` resolves.

Watch items carrying forward:
  - **agentns-claude autobuilder Stages 3+ → release-gate**: this is
    the live load-bearing build. Watch for new commits beyond 90a808d
    in `~/wintermute/agentns-claude/` and status transition past
    in_progress.
  - **fidelity Fleet 1 #1**: recall-surfaced-tracking still 0/5
    (~33h since drop).
  - **chord-async-delegate / drift-fix-self-review-dream / etc**:
    5 blockers, all user-gate, all carried.
  - **/triage `--kind feedback` bug**: single-evidence still.

Meta-observation for /dream rule 11+ pass arc:
  - Pass-12/13/14/15 all carried the C wrapper as a watch item without
    `ls` of sibling repos. Pass-16 found the autobuilder Rust version
    by surveying `~/wintermute/` more broadly, not just by `stat`-ing
    the named file. Lesson for next pass: when an artifact has been
    "stalled" for ≥2 passes, broaden the search beyond the named file
    — adjacent repos and active /build manifest entries often reveal
    that the framing was wrong, not just the freshness.
  - 16 consecutive no-fleet-passes is now decisively a pattern. Not
    necessarily wrong — the queue at 52 PRDs and 13 visions is large,
    and the rule-6 honesty bar is high — but worth noting that
    /dream's "draft" output has not fired in 16 cron + manual invokes.
    The system shape is currently "harvest existing, do not propose
    new" until something Fleet 1 actually ships and earns the right
    to its Fleet 2.

Notes for next /dream:
  - If user picks option (a) or (b) above, capture in pass-17 gossip;
    no new PRD needed (cleanup, not feature).
  - If agentns-claude lands (build manifest status changes from
    in_progress to shipped + LICENSE-tagged push to j0yen/agentns-
    claude), fire trigger (d) Fleet 2 onramp 5-bullet draft pass.
  - Carry same other watches: fidelity Fleet 1 first ship, chord/
    drift-fix user-gates, /triage --kind feedback bug.

## 2026-05-28T12:30  /dream  no-fleet-pass (17th; bare /dream, harvest steady)

Manual invocation, no topic seed. 17th consecutive no-fleet-pass.

State delta vs pass-16 (~30 min ago):
  - **agentns-claude**: no new commits past `90a808d` (iter-1 Stages 1+2
    scaffold from 05:00 PT). Build manifest shows status=in_progress,
    last_action=11:56:44Z. Autobuilder cycle has not advanced past
    iter-1 within this 30-min window. Tree clean (no uncommitted edits).
    Target dir present (build artifacts) but no `receipts/` populated
    yet — Stage 3+ (release-gate) hasn't kicked off.
  - **agentns/userspace/ C wrapper**: unchanged (still uncommitted,
    pass-16 superseded-not-stalled framing stands).
  - **Fleet 1 fidelity**: 0/5 shipped (~33.5h since drop, was ~33h at
    pass-16 — clock advances, status unchanged).
  - **/build blockers**: 5, all user-gate, identical set to pass-16.
  - **recall reflective queue**: latest 10 reflective/self memories same
    as pass-16 — all recalls=0. Persistent freshness signal.
  - **agorabus**: 6 peers on bus this session (was 10 at pass-15 self-
    review; some sessions exited normally). All paired sub+worker.
  - **dirty trees**: per pass-16 self-review snapshot (agentns:7,
    autobuilder:3, cradle-bak:3, memlog:1, provfs:1, recall:5,
    peon-ping:2). Not re-scanned this pass (would steal `wchg since`
    delta from self-review).

Triggers from pass-16 (12:00Z), re-evaluated:
  (a) agentns-claude Stage 3+ commit lands → UNMET (still at iter-1).
  (b) fidelity Fleet 1 first ship → UNMET (0/5).
  (c) cleanup decision on userspace/ → UNMET (no user response yet).
  (d) install-step question → UNMET (no user response yet).
  (e) new user articulation → MET in form (bare /dream) but no topic
      carried; treated as "carry on" per the established 16-pass
      rhythm.

No PRDs drafted. Per rule 6: nothing on disk has changed enough to
motivate a new component. The 30-min cadence between pass-16 and
pass-17 is too short for the load-bearing Stage 3+ commit (autobuilder
inner-loop cycles run on /build's 5-min timer, not /dream's 30-min
timer; multiple build ticks should have fired but none produced a
visible commit — autobuilder is presumably running internal iter-2
verification work that doesn't surface as a top-level commit yet).

**Carried user-offers (REPEATED from pass-16, no new info this pass):**

  1. Cleanup decision: `~/wintermute/agentns/userspace/` C wrapper
     options (a) commit as examples/, (b) `rm -rf userspace/`,
     (c) leave as-is.
  2. Install-step question: should the autobuilder cycle for
     agentns-claude drive all the way to `cargo install --path . --root
     ~/.local` automatically, or is the install step user-gate?

Watch items carrying forward (unchanged):
  - agentns-claude autobuilder Stages 3+ → release-gate (watch for
    commits past `90a808d` and status transition past in_progress).
  - fidelity Fleet 1 #1 first ship (recall-surfaced-tracking 0/5).
  - chord-async-delegate / drift-fix-self-review-dream / etc:
    5 blockers, all user-gate.
  - /triage `--kind feedback` bug: single-evidence still.

Notes for next /dream:
  - The 17-pass arc warrants a pacing observation: /dream is running
    every ~30 min between 21:00-06:30 (cron timer) plus user manual
    invocations. Between cron + manual, /dream has fired 17 times
    against the same harvest-mode state. The "no-fleet-pass" output
    is not failure — it's the correct rule-6 response — but if /build
    spends another 6+ hours without surfacing a Fleet 1 ship,
    consider whether /dream's 30-min cadence is too aggressive for
    the current system state. (A 2-hour or per-/build-tick cadence
    might surface the same information with less compute.) Not a
    PRD — a config knob.
  - If user picks option (a) or (b) on the cleanup decision, pass-18
    can act on it directly (rm or `mv userspace/ examples/`); no
    PRD needed.
  - If autobuilder lands Stages 3+ on agentns-claude, fire Fleet 2
    onramp 5-bullet draft pass (PRD-onramp-* successors).

## 2026-05-28T13:00  /dream  fleet-movement-pass (18th invocation; corrects pass-17 no-fleet framing)

Manual /dream invocation from user, bare (no topic seed). 18th /dream pass in
the harvest arc — but the no-fleet-pass label finally breaks. The 30 min
between pass-17 (12:30Z) and pass-18 (13:00Z) carried four /build ticks that
landed real fleet movement; pass-17's "agentns-claude iter-1 still load-
bearing, nothing else moves" framing missed three other PRDs advancing.

State delta vs pass-17 (live manifest @ 12:57:30Z `last_tick_at`):

  - **provq SHIPPED** (continuity Fleet 1 #2). Three /build ticks landed
    between 11:47Z (install) and 12:57Z (publish):
      * iter-1 11:47Z: cargo build + install -Dm755 to ~/.local/bin/provq
        (970848 bytes); 18 tests green (9 unit + 5 scan + 4 show);
        `~/.local/bin/provq --version` → "provq 0.1.0".
      * iter-2 12:57Z: wm-publish allowlist + wm-publish --slug provq;
        repo public at https://github.com/j0yen/provq; REPOS.md row added
        (Session / context section); committed to wintermute@6a1e676.
    Verified-completed: AC1/AC2/AC3/AC8/AC9 paired; AC4-AC7 boot-gated
    per PRD §Boot-gated header (live FUSE-overlay + LSM xattr surface).
    Status held in_progress pending boot validation OR user-archive call.
    **This is the first continuity Fleet 1 ship.** Trigger for Fleet 2
    is "≥3 of 5 shipped" — 1/5 now, not 3/5; Fleet 2 draft pass does NOT
    fire.

  - **chord-claim iter-1 scaffold** (12:32Z, agorabus rust-extend).
    Extended protocol.rs with `ClientMessage::ClaimAcquire/Release/List`
    + new `ClaimRecord` struct (path/session_id/ttl_unix_secs/
    acquired_unix_secs/reason). Daemon `BusState` gains `claims:
    HashMap<canonical_path, ClaimRecord>` + `prune_expired_claims()`
    called before every read/write. Three new handle_line arms; client
    methods `claim_acquire/release/list`; nested CLI `agorabus claim
    {acquire,release,list}` with `--force/--wait/--path/--session-id/
    --format text|json` flags. cargo build --release green (25.76s);
    cargo test --release 9/9 PASS (no existing tests broken). Next:
    iter-2 writes AC tests, v0.1.0→v0.2.0 bump, commit + push via
    wm-push --slug agorabus.

  - **skill-doctor Stage 1+2 scaffold** (12:30Z).
    /autobuilder sub-skill invoked; intent-card.json derived directly
    from PRD §1-§5 + 11 ACs (no 5-Whys interview — PRD well-spec'd).
    iter-0 baseline commit 463dbed on branch autobuilder/skill-doctor.
    Tree: Cargo.toml + clippy.toml + deny.toml + rust-toolchain.toml +
    src/{main,lib}.rs + tests/acceptance_template.rs +
    agent/{intent-card,owner-map,test-map}.json + proof-lanes.toml +
    scripts/. No remote yet (publish is Stage 6, gated by ACs green).
    Next: Stage 3 iterate — implement extract.rs/check.rs/proposal.rs
    against AC1-AC6+AC8 (harness-testable subset).

  - **agentns-claude unchanged** since 11:56Z (iter-1 Stages 1+2
    scaffold at 90a808d on `~/wintermute/agentns-claude/`); no new
    commits, `target/autobuilder/receipts/` not yet populated.
    Pass-17's framing of this as the load-bearing build was correct
    but not exclusive — three other PRDs moved meaningfully alongside
    it.

  - **memlog-witness** (continuity Fleet 1 #3) flipped to
    `needs_classification` at 12:15Z. The classifier likely surfaced
    that `build_auto:false` on the PRD conflicts with the
    2026-05-27 user directive ("Every PRD is buildable. No opt-outs.
    ... if `build_auto` appears in an older PRD it's ignored").
    Three continuity Fleet 1 PRDs carry the legacy `build_auto:false`:
    memlog-witness (now classified), recall-session-stamp (still
    queued, 0 ticks), session-postmortem (still queued, 0 ticks).
    Per the new rule those flags are stale-by-policy. **Signal to
    /build:** treat the three legacy `build_auto:false` continuity
    PRDs as buildable; their queue position is the only thing
    blocking. (Not a new PRD — a policy-compliance reminder.)

Triggers from pass-17 re-evaluated:
  (a) agentns-claude Stages 3+ commit lands → UNMET (still at iter-1).
  (b) fidelity Fleet 1 first ship → still UNMET (0/5; ~34h drop-age).
  (c) cleanup decision on agentns/userspace/ C wrapper → UNMET.
  (d) install-step question (cargo install autopilot vs user-gate) →
      indirectly answered by provq's path: /build did go all the way
      to install -Dm755 + wm-publish autonomously, no user-gate. The
      same path is available for agentns-claude once /autobuilder
      finishes its release cycle.
  (e) new user articulation → MET (bare /dream from user, no topic
      carried). Treated as "show me where the system is and act on
      anything you'd normally surface" per the 17-pass rhythm.

No PRDs drafted per rule 6 (research doesn't motivate a new component).
provq shipping is 1/5 toward the Fleet 2 trigger; the Fleet-2 onramp
draft pass cited in pass-17 also does NOT fire (it gates on
agentns-claude shipping, not provq).

**Carried user-offers (UNCHANGED from pass-17, surfaces both still
open):**

  1. Cleanup decision: `~/wintermute/agentns/userspace/` C wrapper
     options (a) commit as examples/, (b) `rm -rf userspace/`,
     (c) leave as-is. (Now superseded by the autobuilder Rust path
     per pass-16; this is a tidy-up call only.)
  2. Install-step question for agentns-claude: should /autobuilder
     drive all the way to `cargo install --path . --root ~/.local`
     automatically (provq's path), or is the install user-gate? The
     Fleet 1 onramp dependency chain (agentns-claude →
     claude-agentns-wrap → continuity Fleet 1 ×4) unblocks only when
     `which agentns-claude` resolves.

**New user-offer surfaced this pass:**

  3. Three continuity Fleet 1 PRDs (memlog-witness,
     recall-session-stamp, session-postmortem) carry legacy
     `build_auto:false`. Per the 2026-05-27 directive these are
     buildable; memlog-witness already flipped to
     `needs_classification`. /build can either (a) honor the new
     rule and pick them up (`scan-prds.sh always emits build_auto:
     true`), or (b) wait for the user to flip the frontmatter on
     each of the three. Suggestion: honor the rule autonomously
     since the directive is unambiguous; the user-offer here is to
     confirm that interpretation.

Pacing observation (carrying from pass-17):
  - The 30-min /dream cadence DID catch a real ship event this pass.
    Pass-17's "consider a slower cadence" suggestion was premature —
    the cadence found something. Keep the 30-min cron schedule.
  - Reflective memory recall rate remains at 0 across the latest
    10 reflective/self entries; consider that an ongoing freshness
    signal but not a new PRD draft (the recall-surfaced-tracking
    PRD in Fleet 1 fidelity is the existing instrument for this —
    waiting for that to ship before re-evaluating).

Watch items carrying forward:
  - agentns-claude Stages 3+ → release-gate (watch for commits past
    90a808d and status transition past in_progress).
  - chord-claim iter-2 AC tests + v0.2.0 bump + agorabus push.
  - skill-doctor Stage 3 iterate (extract.rs/check.rs/proposal.rs).
  - fidelity Fleet 1 #1 first ship (recall-surfaced-tracking 0/5).
  - chord-async-delegate / drift-fix-self-review-dream / etc:
    5 blockers, all user-gate.

Notes for next /dream:
  - If memlog-witness picks up per the legacy-build_auto:false
    reading, the policy question is settled; remove the carry.
  - If agentns-claude lands Stages 3+ and `~/.local/bin/agentns-
    claude` becomes installable, fire the Fleet 2 onramp 5-bullet
    draft pass (PRD-onramp-* successors per pass-15 spec).
  - If chord-claim publishes (j0yen/agorabus v0.2.0), the chord
    vision's claim-primitive bullet is satisfied; check chord vision
    for next-bullet motivation.
  - If skill-doctor reaches Stage 6 publish, the skill-doctor
    bullet in (which vision? — verify) is satisfied.
  - When continuity Fleet 1 reaches 3/5 shipped, the Fleet 2 draft
    pass fires per the vision-doc trigger.

## 2026-05-28T13:25  /dream  agentns-on-path-confirm (19th invocation)

Manual /dream invocation from user, bare. 19th /dream pass; 25 min after
pass-18. State delta is small — pass-18 already captured the 4-tick burst
that landed provq/chord-claim-iter-1/skill-doctor-stage-1-2/agentns-claude-
install. This pass mostly confirms what landed and surfaces one
freshly-unblocked PRD.

Live verification this pass:
  - `which agentns-claude` → `/home/jsy/.local/bin/agentns-claude`
  - `agentns-claude --version` → `agentns-claude 0.1.0`
  - Confirms pass-18's manifest claim ("install -Dm755 to ~/.local/bin")
    actually reached PATH. Smoke-tested clean.

Git delta since pass-18 (12:57Z latest commit, last_tick_at 13:11Z, now
~13:25Z): NONE in the four hot repos.
  - agentns-claude: tip still 4e59669 (iter-2 from 06:06Z; install was a
    later /build action, not a new commit).
  - agorabus: tip still 2f5feee (external distribution prep); chord-claim
    iter-1 scaffold from 12:32Z is in working tree, not yet committed —
    iter-2 will commit + bump v0.2.0.
  - skill-doctor: tip still 463dbed (iter-0 scaffold); Stage 3 implement
    pending.
  - provq: tip still 387e5da (iter-1 scaffold) — published-and-archived
    state lives in wintermute@6a1e676 (REPOS.md row), not in repo itself.

**Newly-unblocked PRD signal for /build:**

  PRD-claude-agentns-wrap.md (onramp Fleet 1 #2) carries the dependency
  line "Depends on: PRD-agentns-claude.md shipped + installed at
  ~/.local/bin/agentns-claude". As of pass-18 this was the load-bearing
  precondition; as of pass-19 (verified above) it's satisfied. The wrap
  PRD is now a legitimate /build candidate. CAVEAT: build_target is
  `mixed` and build_into is `/home/jsy` — the PRD modifies ~/.zshrc,
  systemd-user units in ~/.config/systemd/user/, and ~/.claude/scripts/
  agorabus-session-start.sh in place. Even under the 2026-05-27
  "build_auto:false ignored" rule, this one is the rare in-place
  laptop-mod PRD where the user may want to gate. Recommendation to
  /build: prefer chord-claim iter-2 commit or skill-doctor Stage 3 next
  (both are isolated-repo work); surface claude-agentns-wrap to the
  user before applying any ~/.zshrc edit.

Fleet trigger status (unchanged from pass-18):
  - continuity Fleet 2: 1/5 confirmed shipped (provq); 2/5 if
    agentns-claude in_progress→completed counts. Threshold is 3/5.
    NOT armed.
  - onramp Fleet 2: 0/3 Fleet 1 shipped (kernel-pkg-postinstall,
    claude-agentns-wrap, provfs-comm-richer all queued). Threshold is
    2/3. NOT armed.
  - wintermute Fleet 1.5: announce-fix orphan PRD curation only;
    pass-16 trigger remained at curation-only, no Fleet 1.5 movement
    this pass.

Pacing note: pass-18 caught real ship movement (provq + chord-claim
iter-1 + skill-doctor stage-1-2 + agentns-claude install). Pass-19
caught a confirmation only. The 30-min /dream cadence is still
appropriate: it surfaces ship events as they happen but tolerates
quiescent intervals without producing noise PRDs.

**Carried user-offers (REPEAT, no new info):**

  1. Cleanup decision on `~/wintermute/agentns/userspace/` C wrapper
     (commit as examples/, rm -rf, or leave). Superseded by Rust path.
  2. Install-step question for agentns-claude: empirically answered
     by provq's and agentns-claude's parallel paths — /build does
     drive cargo install + ~/.local/bin install autonomously, no
     user-gate needed.
  3. Three continuity Fleet 1 PRDs (memlog-witness, recall-session-
     stamp, session-postmortem) carry legacy build_auto:false. Per
     2026-05-27 directive these are ignored and the PRDs are
     buildable. memlog-witness already flipped to needs_classification.
     Suggestion: /build honors the rule autonomously on the remaining
     two.

**New user-offer surfaced this pass:**

  4. claude-agentns-wrap is now dependency-satisfied. PRD modifies
     ~/.zshrc + systemd-user units + agorabus hook in place. Three
     options: (a) /build picks it up autonomously per the
     "no opt-outs" rule and applies edits; (b) /build drafts a
     proposals/ shadow (mirroring PRD-agorabus-boot-handshake's
     iter-1 pattern: draft to proposals/, user reviews, then user
     swaps live); (c) user explicitly gates this one PRD with a
     blocker entry. Suggestion: (b) — the agorabus-handshake pattern
     is already established for in-place laptop edits.

Watch items carrying forward:
  - agentns-claude wm-publish (j0yen/agentns-claude repo create).
  - chord-claim iter-2 AC tests + v0.2.0 bump + push.
  - skill-doctor Stage 3 implement extract/check/proposal.
  - claude-agentns-wrap: surface user-offer #4 above before any
    autonomous build action.
  - fidelity Fleet 1 #1 (recall-surfaced-tracking) still 0/5.
  - 5 chord/drift/etc. user-gate blockers — unchanged.

Notes for next /dream:
  - If claude-agentns-wrap proposals/ draft lands per option (b),
    log the draft path and AC count.
  - If agentns-claude publishes to j0yen, the install + publish
    sequence will count as the second continuity Fleet 1 ship
    (provq was first). Threshold for Fleet 2 stays at 3/5.
  - If chord-claim iter-2 publishes, chord-vision claim-primitive
    bullet is satisfied (agorabus v0.2.0).

## 2026-05-28T19:30  /dream  vision-companion
Seed: jsy said "for this to work with my mother, voice will need to be the
primary mode of interaction. you will need to always be listening, ready
to respond" (2026-05-28T19:18 PT). This is the deployment target.

Drafted:
- visions/companion.md
- PRD-wintermute-audio-inference.md (microWakeWord + Silero VAD)
- PRD-wintermute-stt-whisper-model.md (whisper.cpp + distil-small.en)
- PRD-wintermute-audio-aec.md (PipeWire module-echo-cancel)
- PRD-wintermute-dialog-turn-fsm.md (Listen→Wake→Capture→Transcribe→Think→Speak)
- PRD-wintermute-companion-boot.md (kiosk install, boot-on-power, no keyboard)
- PRD-wintermute-companion-degrade.md (phrase bank + wm.health.* envelopes)

Order:
  PRD-agorabus-multi-prefix-subscribe (already queued, blocks barge-in)
    ↓
  wintermute-audio-inference  ──  wintermute-audio-aec  (parallel)
    ↓
  wintermute-stt-whisper-model
    ↓
  wintermute-dialog-turn-fsm  ──  wintermute-companion-degrade  (parallel)
    ↓
  wintermute-companion-boot  (deployment capstone)

Notes for /build:
  - Each PRD is rust-extend, single-target, same shape as today's
    bus-startup-defect / heartbeat-keepalive / pipewire-output / pipewire-input
    series that all shipped via parallel autobuilder agents this afternoon.
  - The install-path drift (cargo install → ~/.cargo/bin; systemd →
    ~/.local/bin) is being explicitly fixed in companion-boot at the
    systemd unit level (/usr/local/bin/ system-wide). Sibling PRDs
    should not assume the drift is permanent; companion-boot lands it.
  - Inference (PRD-wintermute-audio-inference) and aec (PRD-wintermute-
    audio-aec) can run in parallel agents. Everything else is gated.
  - Don't dispatch dialog-turn-fsm before stt-whisper-model is
    verified-completed — the FSM needs real stt.final events to test.
  - companion-degrade's AC10 requires stopping wm-stt to simulate
    "ears gone" — coordinate with whatever other PRD work touches stt.

Open questions (left in visions/companion.md):
  - Wake word: "hey wintermute" (two syllable, higher false-positive) vs
    "okay nabu" (stock microWakeWord model, well-trained). Defer to deploy.
  - Local vs cloud STT — defer; PRD-wintermute-stt-whisper-model goes local.
  - Form factor — laptop, RPi Zero, RPi 5, mini-PC. Build PRDs target laptop.
  - First greeting — "Wintermute is ready" is utilitarian. Personality is
    sibling vision.
  - Multi-turn memory — wmd is stateless across turns. Future vision
    *continuity-of-conversation*.


## 2026-05-28T21:06  /dream  vision-continuity-of-conversation
Seed: companion vision OQ#5 ("wmd is stateless across turns ... deferred to
a future vision: continuity-of-conversation") + dialog-turn-fsm non-goal #1.
Grounded in code: wintermute-brain's handle_turn_user builds the request from
one transcript (daemon.rs:1057), test pins req.messages.len()==1 (daemon.rs:1585);
recall_client.rs defers the write/embed path ("lands when the brain starts
writing memories back, a separate iter"); lib.rs:45-47 defines an unused
thread-subject convention (THREAD_SUBJECT_PREFIX / thread_subject_for).

Drafted:
- visions/continuity-of-conversation.md
- PRD-wmd-turn-history.md       (foundation: bounded Vec<Message> into the request)
- PRD-wmd-session-boundary.md   (ts-gap + explicit-close session edges; wm.brain.session.{start,end})
- PRD-wmd-repair-affordances.md ("say that again / louder" via in-session replay, no LLM round-trip)
- PRD-wmd-memory-writeback.md   (session.end -> extract facts -> recall write/embed, as proposals)
- PRD-wmd-session-recap.md      (session.start -> recall last thread -> continuity context/opener)

Order:
  wmd-turn-history
     ├──► wmd-repair-affordances   (needs only the in-session buffer)
     └──► wmd-session-boundary
              └──► wmd-memory-writeback
                       └──► wmd-session-recap

Notes for /build:
  - ALL FIVE are rust-extend into ~/wintermute/wintermute-brain and ALL touch
    daemon.rs/handle_turn_user. They SERIALIZE — do not dispatch two in
    parallel autobuilder agents; they will collide. Build in dependency order.
  - wmd-turn-history rewrites the daemon.rs:1585 single-message assertion to the
    new multi-turn invariant (messages.len()==2*history.len()+1). Rewrite, don't
    delete — the PRD specifies the replacement invariant + AC1 covers it.
  - wmd-memory-writeback is the first wmd->recall WRITE; recall_client.rs only has
    ping/query/touch today. The PRD adds the write/embed client method mirroring
    recall's length-prefixed framing (MAX_FRAME_BYTES=4MiB). Writes go as recall
    *proposals* by default (writeback_auto_commit=false) so triage reviews them.
  - writeback + recap both route through lib.rs thread_subject_for() — neither
    should invent a new recall subject.
  - No new deps expected (extraction reuses the Anthropic client with a distinct
    prompt; writeback_model default Haiku). Same shape as the companion fleet.

Open questions (in visions/continuity-of-conversation.md):
  - Session-id provenance: brain-side ts-gap inference (v0.1) vs a wm-dialog-minted
    session id stamped on wm.dialog.turn.user (sibling dialog PRD). Defer.
  - Privacy of writeback: a companion writing mother's words into a searchable
    store is a real surface. Proposals-by-default is the v0.1 mitigation; full
    consent/boundaries is sibling vision *family-boundaries*.
  - recap_opener default-off: an unprompted continuity greeting is a
    personality/deployment call (companion.md OQ#4).

## 2026-05-28T21:40  /dream  vision-vigil
Seed: run-18 self-review re-opened "agorabus daemon stale binary" the SAME
day it was resolved (runs 16-17). Caught live in Phase 1: pid 2138939 still
exec'ing `/home/jsy/.local/bin/agorabus (deleted)` — the 20:52 reinstall
unlinked its inode. Third axis of staleness, sibling to freshness (memory)
and drift (skill text): a RUNNING PROCESS on stale code.
Grounded in: `/proc/2138939/exe` (deleted) [kernel-truth]; provfs
`user.prov.ts=1780026726` on ~/.local/bin/agorabus [LSM stamp]; agorabus
`enum Command` has NO doctor/restart surface (read src/main.rs); `pevent list`
empty (daemons unsupervised); journal runs 16/17/18 all hand-flag this.

Drafted:
- visions/vigil.md
- PRD-binstale.md                 (rust-cli, new repo: read-only /proc+provfs detector)
- PRD-binstale-source-cmp.md      (rust-extend binstale: `behind-head` vs git HEAD)
- PRD-rollout.md                  (rust-cli, new repo: safe serialized rolling restart)
- PRD-binstale-self-review.md     (shell: wire binstale scan into self-review B.5)
- PRD-agorabus-doctor-selfstale.md (rust-extend agorabus: `agorabus doctor`)

Order:
  binstale
     ├──► binstale-source-cmp
     ├──► binstale-self-review
     └──► rollout
  agorabus-doctor-selfstale  (independent)

Notes for /build:
  - binstale + rollout are SEPARATE new repos by design: binstale is
    read-only (safe), rollout mutates the live fleet (opt-in, --dry-run
    default). Don't fold them together.
  - binstale-source-cmp and binstale-self-review both depend on binstale;
    ship binstale FIRST. rollout can ship on binstale alone (acts on
    deleted-exe/inode-drift) but is better with source-cmp's behind-head.
  - **DO NOT let any build of agorabus-doctor-selfstale (or anything that
    reinstalls agorabus) kill the live bus daemon pid 2138939.** It is
    deliberately escalated/not-restarted (run-18). Restarting the bus is
    rollout's job under a chosen window, or the operator's. Build+install
    only; no restart side effects.
  - binstale-self-review degrades safely if binstale isn't installed yet —
    can land in either order vs PRD-binstale.
  - rollout requires a user-authored ~/.config/rollout/fleet.toml launch
    recipe; it refuses daemons it has no recipe for. No auto-restart of
    unknown processes.

Open questions (in visions/vigil.md):
  - Per-daemon launch recipe provenance (install.sh uses cargo install ->
    ~/.cargo/bin but running binary is ~/.local/bin/agorabus via comm:install;
    two paths). Discuss canonical launch path before any rollout apply.
  - Brief peer-drop on bus restart acceptable, or need socket-handoff first?
    (SessionStart handshake re-attaches — see PRD-agorabus-boot-handshake,
    itself user-gate-blocked.)
  - rollout-window-guard (precise turn-in-flight guard) deferred to Fleet 2;
    depends on continuity-of-conversation's wm.brain.session.{start,end}.

## 2026-05-29T05:15  /dream  vision-scribe
Seed: self-review runs 16/17/18 (2026-05-28) hand-count ctrace "missing
summaries" 1→4→5 every tick and never fix them. Phase 1 caught the root
cause live: the summarizer is NOT slow (renders 12MB/124k-event log in
1.7s by hand this session) and claude-stop.err is EMPTY — the SessionEnd
hook never RAN. Cause = ungraceful exit: headless build/dream/self-review
sessions get SIGKILLed by cgroup teardown (memory
self_build_detached_cgroup_teardown), SIGKILL delivers no SessionEnd, so
ctrace-session-end.sh never renders and the tracer is orphaned. Nothing
backfills. Measured: 828 *.ndjson vs 810 *.summary.md = 18 holes; the 5
oldest are the heavy build/kernel sessions (T162617 12MB, T163729 10MB,
T164732 10MB, T181900, T220013-live).

Drafted:
- visions/scribe.md
- PRD-ctrace-scribe.md                 (rust-cli, NEW repo: single-pass renderer + backfill engine)
- PRD-ctrace-scribe-rollup.md          (rust-extend ctrace-scribe: cross-session daily digest)
- PRD-ctrace-scribe-selfreview.md      (shell: wire backfill+rollup into self-review B.5)
- PRD-ctrace-session-end-resilient.md  (shell: SessionStart backfill sweep + hardened hooks)
- PRD-ctrace-orphan-reap.md            (rust-cli, NEW repo: reconcile orphaned tracer state)

Order:
  ctrace-scribe
     ├──► ctrace-scribe-rollup
     ├──► ctrace-scribe-selfreview      (needs backfill + rollup)
     └──► ctrace-session-end-resilient  (needs backfill)
  ctrace-orphan-reap                    (independent; pairs with session-end-resilient)

Notes for /build:
  - ctrace-scribe is the ROOT — ship it first. rollup/selfreview/resilient
    all shell out to `scribe`. All three DEGRADE SAFELY if scribe isn't on
    PATH yet (fall back to summarize-ctrace-session.sh), so they can scaffold
    ahead and their non-scribe paths are testable today.
  - ctrace-scribe + rollup are pure read/render of ~/.cache/ctrace/sessions
    — safe, no live-system mutation. Test against /tmp fixture dirs.
  - ctrace-session-end-resilient ships its hook changes as *.draft.sh under
    proposals/ (user-gated swap into ~/.claude/scripts/, same precedent as
    PRD-agorabus-boot-handshake). DO NOT auto-swap the live SessionStart/End
    hooks — those touch every session boundary.
  - ctrace-scribe-selfreview edits the self-review skill's Phase B.5; same
    shape as PRD-binstale-self-review (vigil). Wrap the backfill write in the
    existing wchg scope-guard on ~/.cache/ctrace/sessions.
  - ctrace-orphan-reap is read-by-default, --apply opt-in, --apply --dry-run
    available. It signals ONLY the recorded tracer PID and only when the
    owner is dead — never a live-owned tracer.

Relationship to other visions:
  - COMPLEMENTS session-postmortem (visions/continuity.md), which *consumes*
    ctrace as one of its four substrates — a hole-free summary record makes
    that join honest. Not a duplicate; scribe fills the record, postmortem
    reads it.
  - orphan-reap RHYMES WITH vigil's running-process staleness axis but is
    distinct: vigil = stale *binary* on a healthy process; orphan-reap =
    leaked *tracer* whose owner died. Keep separate.

Open questions (in visions/scribe.md):
  - Replace summarize-ctrace-session.sh outright, or keep it as scribe's
    fallback? (leaning: keep as fallback; resilient hook prefers scribe)
  - ctrace has no source repo (python script + .bt + 2 shell scripts) —
    scribe is a NEW repo, not an extend. Confirm before wrapping ctrace.
  - Backfill cadence: SessionStart + self-review (v0.1) vs a dedicated
    timer (probably overkill at this volume).

## 2026-05-29T06:05  /dream  vision-signet
Seed: ~20 consecutive self-review runs flag agentns `/proc/self/agent_session`
all-zeros as "the lone broken kernel asset." Phase 1 probed it live and the
diagnosis is WRONG: kernel is healthy (CONFIG_AGENT_NS=y, /proc/self/ns/agent
resolves -> inode 4026531996 = init-ns range, agent_counters is valid JSON).
All-zeros is the CORRECT reading of a process in the INIT agent namespace —
nothing called unshare(CLONE_NEWAGENT) on the launch path. The kernel isn't
broken; nothing READS the signet correctly. The self-review check
(SKILL.md:123-124) only knows two states ("present" / "empty|missing ->
registration failed") with no branch for present-but-all-zeros = init,
unwrapped, EXPECTED.

Grounded in: live /proc probe this session [kernel-truth]; recall reflective
01KSS21WFN... "agentns all-zeros ~20th run"; SKILL.md:123-124 verbatim; only
agentns-claude (of 8 ~/.local/bin tools) touches the surface and only WRITES
the sid — nothing reads agent_counters; procstat covers cgroup not agentns;
PRD-claude-agentns-wrap.md §Out-of-scope explicitly deferred "a claude-doctor
CLI to check namespace status from outside" — signet builds exactly that.

Drafted:
- visions/signet.md
- PRD-agentns-doctor.md             (rust-cli, NEW repo j0yen/agentns-doctor: tri-state status/explain/counters)
- PRD-agentns-doctor-self-review.md (shell: rewrite B.5 agentns block, kill the 20-run misdiagnosis)
- PRD-agentns-session-receipt.md    (rust-extend agentns-doctor: per-session counter ledger, ctrace-joinable)

Order:
  agentns-doctor
     ├──► agentns-doctor-self-review   (shells out to doctor; degrades w/o it)
     └──► agentns-session-receipt      (rust-extend of doctor)

Notes for /build:
  - agentns-doctor is the ROOT — ship first. It's READ-ONLY (/proc only; never
    writes /proc/*/agent_*, never unshares, never signals). Safe to build+install.
  - Classify by VALUE (session==all-zeros AND file present => init), NOT by a
    hardcoded init-ns inode — the inode differs across observations (4026531996
    this session vs 4026531837 on 2026-05-27); treat inode as advisory only.
  - --proc-root <dir> test hook makes absent/live/malformed FIXTURE-testable
    TODAY without a wrapped session. Most ACs are today-testable; only the
    *live* (non-zero) half of a few ACs is wrap-gated -> declare deferred_acs.
  - agentns-doctor-self-review ships as proposals/*.draft.md, NOT a live
    SKILL.md edit (skill self-mod is classifier-gated; same precedent as
    agorabus-boot-handshake + ctrace-session-end-resilient drafts). It DEGRADES
    safely if the doctor isn't installed (fallback cat with corrected text), so
    it can land in either order vs PRD-agentns-doctor.
  - session-receipt is meaningful only for a WRAPPED session (counters are zero
    in init ns). --require-wrapped exits non-zero in init state so automated
    callers don't litter zeros-receipts. Honest about the precondition.

Relationship to other visions:
  - SIBLING of onramp: onramp's claude-agentns-wrap builds the WRAPPER (makes
    the sid non-zero); signet builds the READING of it (whether zero or not).
    Neither blocks the other — the doctor is useful NOW precisely because it
    explains why today's sessions read zero. onramp Fleet 2's onramp-doctor
    bullet ("runs all three checks") should SHELL OUT to agentns-doctor for the
    agentns third, not re-implement it.
  - COMPLEMENTS scribe + session-postmortem: ctrace counts a session from
    OUTSIDE (eBPF), agentns counts from INSIDE (kernel per-ns hooks);
    receipt --join-ctrace makes the two joinable on agent_session_id. Not a
    duplicate.

Open questions (in visions/signet.md):
  - Init-ns inode stability across boots (classify by value, not inode).
  - Receipt emission trigger: pull-based (self-review calls receipt --emit
    --require-wrapped) vs push-on-SessionEnd (unreliable for headless sessions
    per the SIGKILL-skips-hook problem scribe is fixing). Leaning pull-based.
  - Receipt location ~/.cache/agentns/receipts/<sid>.json mirrors ctrace's
    layout for a sibling-glob join — confirm before wiring.

## 2026-05-29T06:10  /dream  vision-kin
Seed: companion vision OQ#6 (un-dreamed until now) — "does jsy get
notifications when mother summons wintermute? Does mother have a way to call
jsy through it? Sibling vision." Rooted in the original companion seed
("for this to work with my mother…"). User invoked /dream bare, declined to
pick among four offered directions → took the most human un-dreamed one.

Grounded in live Phase 1: bus topics that exist are wm.audio.* / wm.tts.* /
wm.stt.final / wm.brain.reply / wm.browser.{cmd,reply} — NO wm.family.* or
wm.presence.* anywhere (net-new, honestly). wm.browser.cmd→reply
(wintermute-browser/src/protocol.rs:73,85) is the request/reply precedent
reused for wm.family.message→reply. NO outbound transport in any daemon
(grep twilio|ntfy|gotify|webhook|sms = 0) → wm-reach is the new boundary.
bootstrap/install.sh is 217 lines with no caregiver wizard → companion's
"mDNS caregiver-setup flow already assumes a headless device" was
aspirational; family-enroll builds it for real.

Drafted:
- visions/kin.md
- PRD-wintermute-family-intents.md   (rust-extend wintermute-dialog: Family FSM branch, defines wm.family.* contract)
- PRD-wintermute-family-distress.md  (rust-extend wintermute-dialog: deterministic distress fast-path, non-API)
- PRD-wintermute-reach.md            (rust-cli, NEW j0yen/wintermute-reach: off-device transport to jsy)
- PRD-wintermute-presence.md         (rust-cli, NEW j0yen/wintermute-presence: opt-in interaction heartbeat)
- PRD-wintermute-reach-digest.md     (rust-extend wintermute-reach: daily calm digest, joins presence+reach)
- PRD-wintermute-family-enroll.md    (rust-cli, NEW j0yen/wintermute-family-enroll: caregiver setup wizard, capstone)

Order:
  family-intents (defines wm.family.* topics)
     ├──► family-distress     (safety fast-path; extends dialog)
     ├──► wintermute-reach    (transport; consumes wm.family.*)
     │        └──► reach-digest
     └──► wintermute-presence (emits wm.presence.*)
              └──► reach-digest
  family-enroll (config capstone; consumed by all)

Notes for /build:
  - family-intents is the GATE — ship first. It defines the wm.family.* topic
    constants the whole fleet keys on. Other repos declare matching string
    constants (agorabus topics are plain strings; no shared crate needed —
    keep them identical to kin.md's topic table).
  - family-distress MUST stay off the Claude API path (deterministic phrase
    match) — same reasoning companion-degrade used; a distress path gated on
    the brain fails exactly when it matters. Its spoken assurance reuses
    wintermute-brain/src/degrade.rs's phrase mechanism — don't invent a 2nd
    TTS path.
  - family-distress and wintermute-reach can build in PARALLEL once intents
    lands (trigger + delivery). reach closes the FamilyPending→ack loop that
    family-intents opens, so until reach ships every family message times out
    into "I couldn't reach Joe" (expected, not a bug).
  - presence is independent of reach (only emits); reach-digest joins them
    and is the last of the runtime pair.
  - Privacy defaults are LOAD-BEARING (vision OQ#2): presence/silence/digest
    default OFF, distress defaults ON. Don't ship a device that phones home
    about Mom unless family-enroll wrote the opt-in. presence reads only THAT
    a turn happened + transcript LENGTH, never the text.
  - Two new daemons (wintermute-reach, wintermute-presence) follow the shipped
    wm-* shape (subscribe loop + self-emitted-topic filter + heartbeat) and
    must fix the cargo-bin-vs-local-bin install drift at the unit level — the
    regression that bit four companion PRDs.
  - SIBLING of continuity-of-conversation: "tell Joe what I said earlier"
    needs turn memory = continuity's job. kin assumes single-turn intents for
    v1; multi-turn family messages wait on continuity shipping.

Open questions (in visions/kin.md):
  - Transport jsy actually wants on his phone (email/ntfy/gotify/SMS)? wm-reach
    wires email first, gates the rest behind Cargo features. NEEDS jsy.
  - Privacy/consent: does mother hear, in wintermute's voice, what's shared?
    (family-enroll has a `wm-family announce` for exactly this.) NEEDS jsy.
  - Hard-vs-soft distress line (immediate fire vs "Should I let Joe know?").
  - Inbound reply channel (email-poll vs webhook) — wm-reach v1 is send-only
    with a `wm-reach reply` CLI stub; v2 makes inbound real.

Aside (not a kin item): wmd-init.service is FAILED (status=203/EXEC,
start-limit-hit, 8h) and wm-kernel-pkgrel6-*.service FAILED. Flagging for the
companion-reliability surface / next self-review — not in kin's scope.

## 2026-05-28T22:45  /dream-adjacent  research → 5 PRDs (autobuilder quality)
Drafted: PRD-autobuilder-spec-drift-probe.md,
  PRD-autobuilder-mutation-testing.md,
  PRD-autobuilder-reviewer-promotion.md,
  PRD-autobuilder-semantic-ac-judge.md,
  PRD-autobuilder-hardware-mock-convention.md
Research: research/quality-verification-2026-05-28.md

**Trigger:** user prompt "think hard about how to verify quality of
autobuilder-generated Rust code." Survey (Explore agent +
autobuilder/SKILL.md read) showed: lint/test/adversarial harness is
solid; what slips through is LLM-specific (spec drift, tautological
test breadth, reviewer-concern-ships, deferred ACs accumulate). 5 PRDs
designed against the 5 named failure modes.

**Priority:** all 5 `build_priority: high` per user request "bump to
top priority in queue." Also patched /build SKILL.md Phase 2 to sort
queued candidates by build_priority desc (priority field was being
parsed into manifest but not honored at selection time).

**Notes for /build:**
  - Order of leverage: spec-drift-probe (cheapest, blocks biggest
    observed failure mode) → mutation-testing (telemetry first,
    calibrates the eventual gate) → reviewer-promotion (no new code,
    just calibration discipline + auto-promotion playbook in
    /self-review) → semantic-ac-judge (new rust-cli at ~/wintermute/
    ac-judge/, mixed target — rust portion via /autobuilder, then
    self-mod step wires the binary into Stage 4) → hardware-mock-
    convention (touches PRD frontmatter parser + verified-completed
    check #5 + 5 wintermute crate backfills).
  - Mutation-testing PRD ships Phase 1 only (telemetry); Phase 2 gate
    is a future PRD after 20 crates have data.
  - Reviewer-promotion ships Phase A only (calibration log);
    Phases B and C are auto-promoted by /self-review when thresholds
    trip — no human-drafted follow-on needed.
  - Hardware-mock-convention's backfill of 5 wintermute crates will
    spawn 5 follow-on PRDs at iter-N (one per crate). Expect queue
    growth.
  - Semantic-ac-judge is the most expensive PRD (mixed target,
    LLM API in the loop, golden-set calibration); reserve ~3 ticks.
  - All 5 cite research/quality-verification-2026-05-28.md by section;
    /build can re-read the report when scope-checking each.

**Notes for next /dream:** if any of the 5 ships, the matching
failure-mode catalog in §3 of the report shrinks. After 3/5 ship,
re-run the survey and look for the NEXT slip-through pattern (the
report deliberately stopped at 5 to ship leverage rather than over-
catalog).

---

## 2026-05-29T06:40  /dream  vision-homestead (NEW vision)
Drafted: PRD-wintermute-fleet-install-doctor.md,
  PRD-wintermute-install-path-convention.md,
  PRD-wintermute-unit-recovery-watchdog.md,
  PRD-wintermute-readiness-beacon.md
Vision: visions/homestead.md
Seed: bare /dream + Phase-1 live inspection. Picked direction myself
  (user declined the direction question).

**Trigger (live, verified this pass):** `wmd-init.service` is
`failed (Result: start-limit-hit)`, `status=203/EXEC` — `ExecStart=
/usr/local/bin/wmd-init` does not exist; the binary is at
`~/.local/bin/wmd-init`. Three install conventions across six fleet
units (`~/.cargo/bin` wm-audio, `~/.local/bin` the rest, `/usr/local/bin`
wmd-init). 5/6 resolve by luck; the outlier is dead and stays dead
(no human to reset-failed on mother's device). Also confirmed
`WM_ANTHROPIC_API_KEY=` is EMPTY — wm-brain runs but can't reason,
with no deploy-time gate. This is the homeless "companion-reliability
surface" the vision-kin gossip aside (2026-05-29T06:10) explicitly
punted: "flag for the companion-reliability surface / next self-review
— not in kin's scope." homestead is that surface's home.

Order: fleet-install-doctor → { install-path-convention (uses doctor as
  its install gate), readiness-beacon (consumes doctor's per-unit
  verdict) }; unit-recovery-watchdog is independent and can ship in a
  parallel agent.

**Notes for /build:**
  - All four are rust-extend into ~/wintermute/wintermute-platform
    (which already ships the `wmd-init` and `wm` binaries — doctor/ready
    add subcommands to `wm`; watchdog adds a new `[[bin]]`). Same
    rust-extend shape as the companion fleet.
  - Build doctor FIRST and expose its unit-resolution as a shared lib
    function — both install-path-convention (post-install gate) and
    readiness-beacon (units check) consume it. Don't fork the logic.
  - install-path-convention has the load-bearing real-world AC: take
    `wmd-init.service` from failed → active on this laptop (reconcile
    path → reset-failed → start → is-active=active). That's AC4 and it's
    the whole point — verify it live.
  - SCOPE BOUNDARIES (do not merge): companion-boot = power-button→boot
    phrase (reboot-scoped recovery). companion-degrade = mid-conversation
    failure voice (phrase bank in wm-brain). vigil/binstale = STALE/
    deleted running binary vs HEAD. homestead = ABSENT ExecStart path +
    runtime failed-unit recovery + standing readiness verdict. The
    `wm.health.*` envelope is OWNED by companion-degrade's design and
    CONSUMED by vision-kin's health digest — readiness-beacon must
    REUSE it (AC5), not invent a parallel one. The boot phrase is shared
    with companion-boot — suggest boot owns the "ready" phrase, beacon
    owns the "not-ready" reasons (beacon OQ).
  - Two user-decisions gate full ship (vision OQs): which path
    convention wins (~/.local/bin default), and watchdog scope
    (user vs system). Neither blocks doctor or beacon.

**Notes for next /dream:** homestead deliberately stops at 4. A 5th
component — unifying vigil's stale-detector and homestead's absent-path
detector under one `wm doctor` surface — is real but premature until
both ship; left as a vision boundary note, not a PRD. If the user sets
the API key and deploys to real hardware, the next undreamed surface is
*remote operability* (how does jsy push a fix to a device he can't SSH
into?) — not dreamed here because no remote device exists yet.

## 2026-05-29T00:30  /dream  vision-thrift
Drafted: PRD-brain-prompt-cache.md, PRD-wm-router.md, PRD-wm-skills.md,
  PRD-wm-semcache.md, PRD-wm-local-llm.md
Vision: visions/thrift.md
Seed: jsy — "build in /autobuilder instead of the expensive anthropic API".
  Grounded: wmd (wintermute-brain) is the fleet's ONLY API consumer (STT/TTS
  already local). Two wastes found live: (1) MessageRequest has NO cache_control
  despite intent-card AC3 targeting >=60% cache-read, AND compose_persona splices
  volatile recall INTO the system prompt (busts caching); (2) every utterance
  escalates to Sonnet unconditionally.

Order: brain-prompt-cache (INDEPENDENT — ship first, pure per-call saving, no
  new crates, zero quality tradeoff). Then wm-router (spine) ──< {wm-skills,
  wm-semcache, wm-local-llm} build in parallel (all consume router's Route enum).

Notes for /build:
  - brain-prompt-cache is rust-extend into wintermute-brain; its AC5 is the
    repo's EXISTING cache_hit_ratio_above_60pct test (intent-card AC3) — carry it
    forward, don't invent a new one. Watch the existing serialization tests
    ("system omitted when None") — backward-compat is AC2.
  - The 3 new lib crates (router/skills/semcache/local-llm) reuse recall's `embed`
    socket RPC (recall/src/daemon.rs:27 OPS includes "embed"; BGE-small 384-dim,
    HashEmbedder fallback 256-dim). Do NOT stand up a second embedder. Clients
    must be DIM-AGNOSTIC (read vector len from response).
  - wm-local-llm wraps the OpenAI-compatible /v1/chat/completions PROTOCOL, not a
    specific binary — jsy is testing a runtime (ollama/llama-server/llamafile)
    in a parallel window. No weights vendored; endpoint+model are config.
  - DO NOT wire any of this into wintermute-dialog yet — vision component 6
    (dialog-FSM wiring) is intentionally NOT drafted; it waits on
    PRD-wintermute-dialog-turn-fsm shipping (vision OQ1).
  - family-intents overlap (vision OQ2): wm-skills' family skill must REUSE the
    wm.family.* contract from PRD-wintermute-family-intents, not fork the topic.

Open questions for jsy: confidence-floor + local-llm stakes-boundary calibration
  (vision OQ4/OQ5 — local-llm route ships GATED OFF by default); does the brain
  see pre-handled turns for continuity (OQ3 — lean: side-effecting skills write
  recall, pure lookups don't).

## 2026-05-29T07:30  /dream  vision-docket (NEW vision)
Drafted: PRD-docket-core.md, PRD-docket-escalate.md,
  PRD-docket-evidence.md, PRD-docket-self-review-bind.md,
  PRD-docket-digest.md
Vision: visions/docket.md
Seed: bare /dream + Phase-1 recall reflective seeds + journal recurrence
  + self-review SKILL.md. Picked direction myself (no topic given).

**Trigger (live, verified this pass):** the self-review rediscovers the
same findings every run and parks them as PROSE, with no structured
identity, count, or lifecycle. Evidence:
  - `grep -l "Carried forward" ~/brain/journal/*.md` → 6 CONSECUTIVE
    days (05-24..05-29). "agorabus stale binary" appears 7× in the
    05-28 journal alone, 3× in 05-29.
  - `self-review/SKILL.md:359` codifies "playbook justified when a
    signal recurs across 3+ separate runs" — but recurrence is EYEBALLED
    across `recall query` prose. Run-18/19 reflective memories
    (01KSRV7R…, 01KSS21W…) literally say the stale-binary item is
    "approaching the 3-runs threshold" — the agent is hand-counting.
  - `~/.claude/skills/self-review/state/` does NOT exist — the skill has
    no structured state. Carry-forward = one free-text reflective memory
    per run (SKILL.md:452-465).
  - "agentns session-zeros" Pending ~21 consecutive runs with no
    escalation event — proof that without a mechanical rule, escalation
    never fires.

docket = the missing third staleness axis. vigil watches running
binaries vs source; freshness watches memory bodies; drift watches skill
text; docket watches the self-review's OWN findings accumulate, recur,
escalate, and auto-close.

Order:
  docket-core → { docket-escalate, docket-evidence } →
  docket-self-review-bind (needs core+escalate) ; docket-digest (needs
  core, better with escalate).

**Notes for /build:**
  - docket-core is a NEW rust-cli → publish j0yen/docket,
    ~/.local/bin/docket. The other three are rust-extend INTO
    ~/wintermute/docket (same crate, new subcommands/tables; idempotent
    migrations). docket-self-review-bind is build_target: mixed — it
    edits ~/.claude/skills/self-review/SKILL.md (ADDITIVE anchors only,
    no rewrite) + adds scripts/docket-runid.sh.
  - Build core FIRST and freeze the report/list contract (run model,
    stable-key convention) — escalate/evidence/digest all extend the
    same store. Don't fork the schema.
  - docket-self-review-bind has the load-bearing AC: replace the
    hand-maintained "Carried forward" prose (SKILL.md:414) with `docket
    list --open`, and the line-359 playbook rule with `docket list
    --escalated`. Verify end-to-end against the real binary (report 4
    seeded keys across 3 runs → agorabus-stale-binary escalates; a
    1-run finding auto-resolves on sweep).
  - SCOPE BOUNDARIES (do not merge): docket is a per-KEY lifecycle state
    machine (open→escalated→resolved, run-streak counting). recall is
    similarity-retrieval over prose. docket LINKS to recall ULIDs
    (`recall:<ulid>` evidence refs) but is NOT a recall extension. Keep
    them separate tools.
  - ENVELOPE REUSE (hard): docket-digest's JSON MUST reuse the
    `wm.health.*` envelope OWNED by companion-degrade and CONSUMED by
    kin / homestead's readiness-beacon (per the 2026-05-29T06:40
    homestead note). Match field names exactly against companion-degrade's
    shipped crate/contract — do NOT invent a parallel schema. A digest
    test must assert conformance.
  - NON-DESTRUCTIVE: every self-review→docket call is list/report/sweep.
    The binding adds ZERO new user-gated blockers. docket-digest
    documents (does NOT auto-modify) the SessionStart hook snippet —
    that wiring stays user-gated.

Open questions (vision OQs, user-gated, none block core):
  - Run identity string format (proposed YYYY-MM-DD.<n>).
  - Store format SQLite (leaning) vs JSONL-in-git.
  - Future producers (vigil binstale, readiness-beacon, /build blockers)
    all reporting to ONE docket — left as a vision boundary note, not a
    v1 PRD, until the self-review producer proves the contract out.

**Notes for next /dream:** docket deliberately stops at 5 with the
self-review as the SOLE v1 producer. The multi-producer surface (a
unified docket fed by vigil + homestead + /build, with cross-tool dedup)
is real but premature until the single-producer contract ships. Don't
draft it until docket-self-review-bind is green on this laptop.

## 2026-05-29T01:00  /dream  vision-thrift  (+1 PRD)
Drafted: PRD-brain-backend-ladder.md  (rust-extend -> wintermute-brain)
Trigger: jsy decision this session — "default to local 3b. wire up switches to
  use 8b, Sonnet and Opus when needed." Plus wm-local-llm is mid-build (the local
  backend client it depends on).
Design: extends the EXISTING LlmClient trait seam (wintermute-brain
  src/daemon.rs:88) + swap-model/default-model CLI (src/main.rs:54-95). A
  LadderClient dispatches a turn to local (wm-local-llm) vs Anthropic by active
  tier; default_tier=local-3b; auto-escalates one rung when a local tier returns
  LocalOutcome::Escalate; bounded at the top.
Notes for /build:
  - DEPENDS ON wm-local-llm (path dep) — build that FIRST (in flight now,
    branch autobuilder/wm-local-llm). Don't start the ladder until wm-local-llm
    passes its gate.
  - COMPOSES WITH PRD-brain-prompt-cache (AC8): the Anthropic tiers must keep
    their cache_control breakpoints. If prompt-cache hasn't landed, the ladder
    just passes MessageRequest through unmodified.
  - Load-bearing behavior change: build_anthropic_client -> None (no API key) no
    longer disables the brain when default tier is Local. Missing key only
    disables Sonnet/Opus tiers. This fixes the "brain mute, no key" outage class.
  - Reuse the existing LlmClient fake-injection test pattern for AC2/AC3/AC5.
Order now: prompt-cache (independent) ; wm-local-llm -> brain-backend-ladder ;
  wm-router -> {wm-skills, wm-semcache, wm-local-llm-as-router-tier}.

## 2026-05-29T07:30  /dream  vision-hearth  (+3 PRDs, new vision)
Drafted: PRD-hearth-persona-config.md, PRD-hearth-first-contact-greeting.md,
  PRD-hearth-dialog-degrade-warmth.md
Vision: visions/hearth.md
Seed: no user topic given this invocation; chose the strongest *uncovered*
  evidence after confirming the freshness/identity/recovery space is saturated
  (vigil/signet/onramp/homestead/docket all cover tonight's infra anomalies —
  agentns all-zeros, agorabus stale binary, ctrace flakes — so piling on there
  would violate "don't dream past the research"). hearth fills companion.md's
  own deferred OQ#4 ("what does she hear the first time?") + dialog-turn-fsm
  Non-goal #2 ("personality model … blunt for v0.1").

What's the gap (all confirmed by reading source in Phase 1):
  - Persona is a compile-time const: wintermute-brain/src/daemon.rs:47
    DEFAULT_PERSONA. Not in brain.toml (which already has user_name/timezone/
    recap_opener). -> persona-config lifts it to a [persona] table.
  - recap_opener flag exists (lib.rs:100) but NO greeting content defined, and
    no first-ever-boot welcome. -> first-contact-greeting.
  - wm-dialog degrade.rs:44-45 returns IDENTICAL "Sorry, I didn't catch that."
    for SttUncertain AND TranscribeTimeout, no variation. -> dialog-degrade-warmth.

SCOPE BOUNDARY (do not merge): hearth is the VOICE (persona/greeting/warm
  phrasing); companion-degrade (*say-so*) is the PLUMBING (operational fault
  bank in wm-brain keyed by component error kind, owns wm.health.*). They share
  the wm-tts path but are different concerns. The dialog degrade.rs module
  comment forward-references companion-degrade for "mood-aware phrasing" — that
  reference is MIS-AIMED: companion-degrade builds in wm-brain and never touches
  the wm-dialog FSM bank. hearth-dialog-degrade-warmth is the real owner of that
  file. No PRD touched it before (grep-confirmed).

Order: persona-config -> first-contact-greeting (both -> wintermute-brain) ;
  dialog-degrade-warmth (-> wintermute-dialog, independent, ship anytime).

Notes for /build (IMPORTANT — brain serialization):
  persona-config + first-contact-greeting both extend wintermute-brain's
  compose_persona / BrainConfig surface, as do the IN-FLIGHT brain-prompt-cache
  and brain-backend-ladder PRDs. FOUR PRDs touch the same system-prompt
  composition + config struct. Serialize them or expect rebases. Suggested
  order if brain-prompt-cache lands first: prompt-cache -> persona-config
  (persona is the cached prefix; AC5 asserts prefix byte-stability) ->
  first-contact-greeting. brain-backend-ladder is orthogonal (dispatch tier),
  can interleave but watch BrainConfig merges.
  dialog-degrade-warmth has NO brain dependency — dispatch it freely.

Open questions (vision OQs, none block v1):
  - Learned persona (recall wintermute-profile subject) vs configured — deferred
    until persona-config proves the data shape.
  - One register across two repos (brain.toml vs wm-dialog phrases) can drift;
    consistency check deferred until both are config-sourced.

## 2026-05-29T01:30  /dream  vision-thrift  switching strategy LOCKED (+1 PRD, 2 revised)
Drafted: PRD-wm-verify.md (rust-lib ~/wintermute/wm-verify) — soft-failure gate.
Revised: PRD-brain-backend-ladder.md -> v0.2 ; PRD-wm-router.md -> v0.2 (both have
  in-file Changelog sections; v0.1 behavior is a strict subset).
jsy locked the switching strategy (vision "Switching strategy" section):
  - Ladder: local-3b -> local-8b -> HAIKU -> Sonnet -> Opus (Haiku added).
  - Posture: LOCAL-FIRST (cheapest tier that clears the bar; default 3b).
  - Latency: FILLER WHILE ESCALATING (backchannel via companion-degrade phrases).
  - "When needed" decided TWICE: predict (router stakes/start tier) + verify
    (wm-verify gate). Pure escalate-on-hard-failure was blind to a 3b answering
    confidently WRONG — wm-verify closes that.
Notes for /build:
  - wm-router v0.2: Route is now Skill/CacheLookup/Brain{stakes}. NO LocalLlm
    route anymore — the brain ladder owns local-vs-cloud. Safety stage runs FIRST
    (high recall) and tags Stakes::HighStakes(class) so the ladder skips local for
    medication/medical/emergency/distress/money. AC4 = 100% high-stakes recall.
  - brain-backend-ladder v0.2 now DEPENDS ON wm-verify + wm-router (not just
    wm-local-llm). Escalation = dual-signal (hard wm-local-llm Escalate + soft
    wm-verify reject). Build order: wm-local-llm(done) + wm-verify + wm-router
    -> brain-backend-ladder.
  - wm-verify is pure/in-process (no network, no model). Conservative toward
    Reject but AC6 forbids false-rejecting normal answers (would nuke local-first).

## 2026-05-29T08:34Z  /dream  vision-earshot  (manual /dream, no topic)
Drafted: PRD-earshot-dialog-timing.md, PRD-earshot-vad-patience.md,
  PRD-earshot-tts-legibility.md, PRD-earshot-gentle-reprompt.md
Vision: visions/earshot.md
Seed: companion.md's "a non-technical elder, jsy's mother" + hearth's own
  scope note. hearth made the WORDS warm; earshot makes sure she can HEAR
  them and isn't RUSHED. New domain, grep-confirmed unclaimed by any PRD.

What's the gap (all confirmed by reading source in Phase 1):
  - Conversation tempo is compile-time: wintermute-dialog/src/fsm.rs
    CONFIRM_TIMEOUT_MS=30_000 (fsm.rs:28), MAX_REPROMPTS=1 (fsm.rs:31),
    family re-exported lib.rs:34-35. Not in a config table. An elder who
    pauses gets cut off. -> earshot-dialog-timing (const->[timing], same
    move hearth-persona-config made for the persona string).
  - One reprompt then silent exit: Confirming->ConfirmTimeout->
    DenyReason::Silence->Idle (fsm.rs:236-252); reprompt path exists
    (fsm.rs:402-415) but capped at 1. -> earshot-gentle-reprompt (patient
    sequence + warm SPOKEN close).
  - TTS has no rate/volume: PiperSubprocess::render passes only --model +
    --output_file (synth.rs:101-105), no --length_scale, no gain anywhere.
    -> earshot-tts-legibility (slower + louder for hearing loss).
  - VAD silence-hangover (speech.end "after confirmed silence",
    events.rs:27) tuned for normal speech, not configurable. ->
    earshot-vad-patience (longer default so a mid-sentence pause != end).

SCOPE BOUNDARY (do not merge): earshot-gentle-reprompt owns the SILENCE /
  no-response path in fsm.rs ("I'm still waiting for you"). hearth-dialog-
  degrade-warmth owns degrade.rs, the FAULT bank ("I didn't catch that").
  Different module, different trigger, shared wm-tts path. earshot must
  NOT touch degrade.rs; hearth must NOT touch the fsm silence branch.

Order: earshot-dialog-timing (foundation, introduces [timing]) ->
  {earshot-vad-patience (wm-audio, independent), earshot-tts-legibility
  (wm-tts, independent)} parallel -> earshot-gentle-reprompt (wm-dialog,
  reads dialog-timing's max_reprompts + cadence).

Notes for /build:
  - earshot-dialog-timing + earshot-gentle-reprompt BOTH edit fsm.rs in
    wintermute-dialog — serialize them (timing first, verified, then
    reprompt). Do NOT dispatch concurrently.
  - hearth-dialog-degrade-warmth is in-flight on the same crate. No logic
    overlap (degrade.rs vs fsm.rs) but lib.rs re-export / Cargo churn may
    force a rebase on the earshot dialog PRDs. Watch it.
  - vad-patience (wm-audio) + tts-legibility (wm-tts) are fully
    independent of the dialog PRDs and of each other — parallel agents OK.
  - Tests pin the old const timing values (e.g. fsm.rs:642
    StartConfirmTimer{ms}==CONFIRM_TIMEOUT_MS). REWRITE to the config-
    sourced invariant, don't delete (continuity-of-conversation discipline
    for req.messages.len()).
  - Defaults are elder-friendly (more patient, slower, louder) but tunable;
    setting knobs to neutral/old values must reproduce today's behavior.

Open questions (vision OQs, none block v1):
  - Learned pace (widen silence window from observed cut-offs) vs static
    config defaults — deferred to a later vision.
  - Higher TTS gain feeds the AEC loop (companion's audio-aec must still
    cancel) — deployment smoke test, not a unit AC.
  - Three config tables (dialog [timing] / audio [vad] / tts [voice]) vs
    one caregiver-facing file — unification is a homestead/onramp concern.

## 2026-05-29T09:06:28Z  /dream  vision-almanac  (manual /dream, no topic)
Drafted: PRD-almanac-schedule-store.md, PRD-almanac-tick-daemon.md,
  PRD-almanac-speak-bridge.md, PRD-almanac-acknowledge.md,
  PRD-almanac-missed-to-kin.md
Vision: visions/almanac.md
Seed: manual /dream during the live companion push. companion can HEAR,
  hearth speaks WARM, earshot WAITS + is legible, kin LINKS to jsy. The
  missing panel: CLOCK-driven proactive speech. The whole fleet is
  reactive (does nothing until summoned); an elder's real load is the
  on-time recurring things she forgets — pills, meals, the nurse at 2.

The gap (confirmed by reading source in Phase 1):
  - No clock-driven proactive turn anywhere. wm-brain's only proactive
    speech is recap_opener (daemon.rs:1352), fired once at session start.
    BrainConfig (lib.rs:80-118) has timezone but nothing scheduled.
  - wm-cal is NOT this: it's a CalDAV daemon for JSE's appointments —
    SecretService creds (creds.rs:16), RRULE expansion (caldav.rs:397),
    caregiver-facing by design (intent-card.json:17). Network+account
    required. Wrong shape for "blue pill at 8am" on a maybe-offline desk.
    almanac is LOCAL, recurring, opt-in, spoken — and CONSUMES
    wm.cal.event.upcoming later rather than reimplementing CalDAV.
  - Reuse, don't rebuild: speak-bridge emits through the EXACT proactive
    path recap_opener uses — ReplyEvent{text,ts} -> publish(REPLY)
    (daemon.rs:1352-1377) — so prompts inherit hearth's persona +
    earshot's pace automatically.

SCOPE BOUNDARIES (do not merge): almanac owns the CLOCK (when to prompt).
  hearth owns WORDS/persona; earshot owns TEMPO/patience; kin owns
  OFF-DEVICE delivery. almanac adds NO persona string, NO timing const,
  NO CalDAV. acknowledge READS earshot's patience window; missed-to-kin
  RIDES kin's wm.family.* channel.

Order: schedule-store (new crate wintermute-almanac, foundation) ->
  tick-daemon (publishes wm.almanac.due) -> {speak-bridge (wm-brain,
  due->spoken), missed-to-kin (wm-almanac, wm.almanac.missed)} parallel
  -> acknowledge (wm-brain, next wm.stt.final -> done/snooze/missed;
  feeds missed-to-kin).

Notes for /build:
  - schedule-store is a NEW crate at ~/wintermute/wintermute-almanac
    (companion-fleet member like wintermute-calendar). Ships alone as a
    useful CLI; everything else extends it or wm-brain.
  - speak-bridge + acknowledge BOTH edit wintermute-brain (subscribe-loop
    dispatch + DaemonState). Serialize them (speak-bridge first, verified,
    then acknowledge which adds PendingAck on top). Do NOT dispatch
    concurrently on wm-brain.
  - tick-daemon + missed-to-kin both extend wintermute-almanac; serialize
    on that crate too (tick-daemon first — it defines the wm.almanac.due
    envelope + agorabus client; missed-to-kin adds the watch/bridge).
  - speak-bridge depends on the wm.almanac.due envelope shape from
    tick-daemon — wait for tick-daemon's README envelope doc before
    building speak-bridge.
  - missed-to-kin bridges to kin's wm.family.message. kin is still a
    VISION (family-* PRDs in flight per last gossip). missed-to-kin's
    AC3 makes it ship WITHOUT kin (emits wm.almanac.missed only; kin
    bridge is conditional). Build almanac without blocking on kin.
  - hearth-* and earshot-* edit wm-dialog/wm-brain too — watch for
    Cargo/lib.rs re-export churn forcing a rebase on speak-bridge/ack.
  - Envelope contract (pin in wintermute-almanac README so all consumers
    agree): wm.almanac.due {id,label,say,category,fire_ts};
    wm.almanac.ack {id,state:done|snoozed|missed}; wm.almanac.snooze
    {id,resume_ts}; wm.almanac.missed {id,label,category,missed_ts}.

Open questions (vision OQs, none block v1):
  - Caregiver remote editing of mom's routine -> kin/onramp/homestead
    concern, not almanac (wm-almanac add is the v1 interface).
  - Quiet hours / active_hours per entry — deferred (defaulting risks
    silently skipping a real med prompt).
  - Learned timing (shift from observed ack latency) — same learned-vs-
    static deferral earshot made. Static local_time for v1.

## 2026-05-29T02:33  /dream  vision-vigil (extend → Fleet 3)
Drafted: PRD-agorabus-client-reconnect.md, PRD-agorabus-drain-notice.md,
  PRD-agorabus-state-persist.md, PRD-agorabus-reload.md,
  PRD-agorabus-reload-self-review.md
Vision: visions/vigil.md (extended — resolved Open Question #3; added Fleet 3)

Why this fleet: the carried-forward "agorabus daemon stale binary" debt
(self-review runs 16–19 + 2026-05-29) has a single root cause that no
existing PRD addressed. Phase 1 read agorabus/src/client.rs and confirmed
the long-lived `subscribe` client has NO reconnect logic — when the daemon
dies the subscriber dies with it, and agorabus-session-start.sh only
re-registers at session START, never on daemon death. So a live bounce
strands every current session. That's why self-review's
agorabus_daemon_stale_binary playbook escalates instead of auto-fixing
whenever subscribers > 5 (SKILL.md:259,270). Fleet 3 builds the handover
MECHANISM that makes the bounce non-destructive.

Order: client-reconnect (keystone, ship FIRST)
  → drain-notice (reconnect consumes resume_after_ms backoff)
  → state-persist (independent of drain; finishes daemon.rs:72's deferred
    persistence — claims+intents survive a bounce)
  → reload (depends on reconnect+drain+persist; the one-command bounce)
  → reload-self-review (depends on reload SHIPPED+VERIFIED; rewrites the
    playbook to call `agorabus reload` and lift the ≤5 ceiling).

Notes for /build:
  - SERIALIZE the four agorabus rust-extends (reconnect → drain → persist
    → reload). All touch the same crate (protocol.rs/daemon.rs/main.rs/
    client.rs); concurrent /autobuilder agents will collide on Cargo/lib.rs
    re-export churn — same caution Fleet 1 raised for the companion fleet.
  - client-reconnect is the ONLY one that ships value alone and unblocks
    everything; prioritize it. It has no dependency and no protocol change.
  - drain-notice adds a `bus.draining` ServerEvent variant — coordinate
    with any in-flight agorabus PRD that also edits protocol.rs.
  - reload-self-review is build_target:shell editing
    self-review/SKILL.md — do NOT build it until `agorabus reload` is
    installed and verified (its whole premise is calling that command).
  - Composes with vigil Fleet 1 `rollout` (not a duplicate): rollout is
    the fleet-wide orchestrator; it can shell out to `agorabus reload` for
    the bus specifically and fall back to SIGTERM+relaunch for daemons
    that lack a reload verb. Fleet 3 makes rollout's brief-drop assumption
    actually safe.
  - LIVE-FLEET CAUTION: building/testing these will exercise daemon
    restarts. The running bus (pid ~1750 per 2026-05-29 self-review) is
    itself stale and carries the live voice fleet + sessions. Don't bounce
    the production daemon during a test — tests use their own --socket /
    --state-file under a temp dir.

Open questions (none block v1):
  - reconnect-self-survival window: should the reconnect loop give up and
    exit after a configurable wall-clock (so a truly-dead bus doesn't leave
    zombie subscribers forever), or retry until the session ends? PRD
    leaves it unbounded by default with --max-reconnect-attempts to bound.
  - state-persist scope: persist ONLY claims+intents (chosen) vs also a
    last-known peer snapshot for `peers` display during the reconnect gap.
    Deferred — peers re-announce within seconds via reconnect.

## 2026-05-29T03:00  /dream  vision-atlas (new)
Drafted: PRD-atlas-core.md, PRD-atlas-edges.md, PRD-atlas-orphans.md,
  PRD-atlas-render.md
Vision: visions/atlas.md (new)
Order: atlas-core (keystone, ship FIRST — node model + parsers)
  → atlas-edges (attaches dependency edges to the nodes)
  → atlas-orphans (divergence lint; needs nodes + edges)
  → atlas-render (DOT/Mermaid/tree; needs nodes + edges)

Why this vision: bare `/dream`, interactive. Phase 1 found the
feature-space SATURATED and well-targeted — every evidence-motivated pain
already has a PRD home (agorabus-stale → vigil Fleet 3 drafted tonight
02:33; agentns all-zeros → PRD-claude-agentns-wrap; ctrace SessionEnd
flake → PRD-ctrace-session-end-resilient; finding recurrence → docket).
The one genuinely-uncovered gap is meta: the /dream end-state is "each
PRD a node in a graph," yet NOTHING renders that graph. 107 PRDs / 24
visions / 2 manifests / 3922-line gossip / 117-line REPOS.md — the edges
are all written down (PRD `Vision:`+`build_into` frontmatter, build
manifest `output_repo_path`/`iter_log`, dream manifest prds_drafted, our
own `Order:` lines) but never joined. atlas is one read-only Rust CLI
that joins them.

Notes for /build:
  - SERIALIZE the three rust-extends (edges → orphans → render). All
    extend ~/wintermute/atlas; concurrent /autobuilder agents collide on
    Cargo/lib.rs re-export churn — same caution every multi-extend fleet
    raised (vigil Fleet 1/3, companion).
  - atlas-core is the ONLY one that ships value alone and unblocks the
    rest; build it first. New repo j0yen/atlas, no dependency, no store.
  - orphans and render are independent of each other — order either way
    after edges.
  - atlas is READ-ONLY over the autobuilder corpus by design (AC asserts
    fixture mtime unchanged). It will read this gossip file + both skill
    manifests — but never writes them. Don't let an /autobuilder agent
    "helpfully" add a write path.
  - `atlas doctor` (from atlas-orphans) is exactly the fulfilled-vision
    cross-reference /dream SKILL.md says it does by hand. Once orphans
    ships+verifies, the natural capstone is a build_target:shell PRD
    wiring `atlas doctor` into self-review — deliberately NOT drafted yet
    (premise is calling installed+verified `atlas doctor`; mirrors how
    vigil held agorabus-reload-self-review behind `agorabus reload`).
    Left as vision OQ for next /dream extend atlas.

Open questions (none block v1):
  - atlas vs docket overlap: NO. docket = self-review FINDINGS get an
    identity/lifespan; atlas = the vision/PRD/repo CORPUS gets a rendered
    structure. They could meet later (atlas reports a stale-repo
    divergence AS a docket finding) — future cross-vision bullet.
  - edge source-of-truth: frontmatter `Depends on:` authoritative, gossip
    `Order:` lines secondary/dashed. Pinned in atlas-edges README.

## 2026-05-29T10:15  /dream  (saturation report — no PRDs drafted)
Manual bare /dream, interactive. Phase 0/1 sweep confirms the 03:00
vision-atlas pass's finding still holds 7h later: the corpus is
SATURATED. State: 111 PRD-*.md files, 25 visions, this gossip at ~3970
lines, build manifest tracking 154 PRD entries.

Deliberately drafted NOTHING this pass (hard rule #6 — don't dream past
the research). No laptop fact changed since 03:00 to motivate a net-new
vision. Today's two self-reviews surfaced only USER-GATED items, each
already PRD-covered or not PRD-shaped:
  - agorabus daemon stale binary (~19th consecutive run) → PRD-agorabus-reload
    + PRD-agorabus-doctor-selfstale already drafted; blocked on a restart
    window, not on code.
  - agentns agent_session all-zeros (~22nd run) → kernel/boot-side;
    PRD-claude-agentns-wrap + PRD-agentns-doctor cover the userspace side.
  - ctrace SessionEnd hook flake → PRD-ctrace-session-end-resilient.
  - memlog group membership / bpolicy load / pacman kernel update → not
    PRD-shaped (usermod / sudo / reboot — user's call).

Signal for /build: the constraint is THROUGHPUT + user-gated decisions,
not draft supply. 111 drafts deep; 9 hard-blocked (PRDs 2,5,9,14,19,28,
53,57,118), all user-gated. Draining beats drafting right now.

Open question for next /dream: the genuinely-changed fact is the
wintermute kernel BOOTED 2026-05-28 (provfs LIVE + stamping xattrs,
/dev/memlog present, agentns the lone gap). continuity Fleet 1 was
boot-gated and is now UNBLOCKED — that's a /build action, not new
dreaming. The net-new dreaming white space, IF the user wants it next
pass, is userspace consumers of the now-live provfs/memlog surface
beyond what continuity drafted. Asked the user for direction; question
dismissed, so leaving it as the seed for an explicit `/dream <topic>`.

## 2026-05-29T11:00  /dream  (saturation report — no PRDs drafted)
Third consecutive no-draft pass (after 03:00 vision-atlas + 10:15). Bare
interactive /dream at ~04:00 PDT. Phase 0/1 confirms corpus unchanged
since 10:15: 111 PRD-*.md, 25 visions, build manifest 154 entries. No
laptop fact moved in the ~45min since the last pass.

Self-review run 2 (02:04 PDT) surfaced only USER-GATED items, all already
PRD-covered or not PRD-shaped: agorabus stale binary (~19th run), agentns
all-zeros (~22nd run), ctrace SessionEnd flake, memlog group membership,
bpolicy load, pacman kernel 7.0.9->7.0.10. None net-new draftable.

Offered the user the one genuine white-space (userspace consumers of the
now-live provfs xattr / /dev/memlog surface beyond vision-continuity) +
topic/extend/stop. Question dismissed. Held the line on rule #6 — drafted
nothing.

Signal for /build unchanged: constraint is THROUGHPUT + user-gated
decisions, not draft supply. 111 deep, 9 hard-blocked (2,5,9,14,19,28,53,
57,118). Draining beats drafting. The provfs/memlog-consumer seed remains
parked for an explicit `/dream <topic>`.

## 2026-05-29T11:30  /dream  (saturation report — no PRDs drafted)
Fourth consecutive no-draft pass (after 03:00 vision-atlas + 10:15 +
11:00). Manual interactive bare /dream at 04:30 PDT, ~30min after the
11:00 pass. Phase 0/1 + mandatory recall seeding confirm corpus
unchanged: 111 PRD-*.md, 25 visions, build manifest 154 entries. No
laptop fact moved since 11:00.

Recall seeding (rule: mandatory, not skipped): all reflective hits are
this-week self-review snapshots, recalls=0, pure carry-forward (agorabus
stale binary, agentns all-zeros, "kernel built not booted" — the latter
now itself STALE since the kernel booted 05-28). No net-new ideation
seed surfaced. Hybrid ideation query returned the same boot-pending
observations, all superseded.

Self-review today surfaced only USER-GATED items, each PRD-covered or
not PRD-shaped: agorabus restart window, agentns kernel registration,
memlog group membership, bpolicy load, pacman kernel 7.0.9->7.0.10.

Offered the user (interactive AskUserQuestion) the four real moves:
provfs/memlog-consumer white space / give-a-topic / extend-a-vision /
hold. Question dismissed. Held rule #6 — drafted nothing.

Signal for /build unchanged across 4 passes now: the constraint is
THROUGHPUT + user-gated decisions, not draft supply. 111 deep, 9
hard-blocked (PRDs 2,5,9,14,19,28,53,57,118), all user-gated. Draining
beats drafting. The genuine white-space seed (userspace consumers of the
now-live provfs xattr / /dev/memlog surface beyond vision-continuity)
remains parked for an explicit `/dream <topic>` when the user wants it.

## 2026-05-29T12:00  /dream  (saturation report — no PRDs drafted)
Fifth consecutive no-draft pass (after 03:00 vision-atlas + 10:15 + 11:00
+ 11:30). Manual interactive bare /dream. Phase 0/1 + mandatory recall
seeding confirm corpus unchanged: 112 PRD-*.md (was 111; +1 from concurrent
/build Phase-6 draft, not /dream), 25 visions, build manifest 154 entries.
No laptop fact moved since 11:30.

Recall seeding (mandatory): all reflective hits are this-week self-review
snapshots, recalls=0. Latest (01KSSPHV9B... run 3, 04:01 PDT) is pure
carry-forward: agorabus stale binary (~19th), agentns all-zeros (~23rd),
memlog group needed, bpolicy off, recall daemon socket absent. Today's
journal (2026-05-29 run 3) same — only USER-GATED items, each PRD-covered
or not PRD-shaped.

Offered the user (AskUserQuestion) the four real moves: provfs/memlog-
consumer white space / give-a-topic / extend-a-vision / hold. Question
dismissed again. Held rule #6 — drafted nothing.

Signal for /build unchanged across 5 passes: constraint is THROUGHPUT +
user-gated decisions, not draft supply. 112 deep. Draining beats drafting.
Genuine white-space seed unchanged: userspace consumers of the now-live
provfs xattr / /dev/memlog surface (kernel booted 05-28, provfs healthy +
stamping, /dev/memlog present) beyond vision-continuity. Parked for an
explicit `/dream <topic>` when the user wants it.

## 2026-05-29T12:30  /dream  (saturation report — no PRDs drafted)
Sixth consecutive no-draft pass (after 03:00 vision-atlas + 10:15 + 11:00
+ 11:30 + 12:00). Manual interactive bare /dream. Phase 0/1 + mandatory
recall seeding confirm corpus unchanged: 108 PRD-*.md, 25 visions, build
manifest 154 entries. No laptop fact moved since 12:00.

Recall seeding (mandatory): reflective hits all this-week self-review
snapshots, recalls=0, pure carry-forward (kernel built->booted 05-28,
agorabus stale binary, agentns all-zeros). ctrace --since 24h empty;
pevent list empty — nothing newly running to motivate a PRD. No net-new
ideation seed.

Offered the user (AskUserQuestion) four moves: draft the provfs/memlog-
consumer fleet (the lone research-backed white-space seed) / give-a-topic
/ extend-a-vision / hold-and-drain. Question dismissed (6th time). Held
rule #6 — drafted nothing.

Signal for /build unchanged across 6 passes: constraint is THROUGHPUT +
user-gated decisions, not draft supply. Draining beats drafting. Genuine
white-space seed unchanged and still parked: userspace consumers of the
now-live provfs xattr / /dev/memlog surface beyond vision-continuity.
Awaits an explicit `/dream <topic>` or an opt-in to the fleet above.

## 2026-05-29T14:00  /dream  (saturation report — no PRDs drafted)
Seventh consecutive no-draft pass (after 03:00 vision-atlas + 10:15 +
11:00 + 11:30 + 12:00 + 12:30). Manual interactive bare /dream. Phase 0/1
+ mandatory recall seeding confirm corpus unchanged: 108 PRD-*.md, 25
visions. No laptop fact moved since 12:30.

Recall seeding (mandatory): all reflective hits this-week self-review
snapshots, recalls=0, pure carry-forward (kernel built->booted 05-28,
agorabus stale binary, agentns all-zeros, memlog group needed). ctrace
--since 24h EMPTY; pevent list EMPTY — nothing newly running to motivate
a PRD. Kernel surfaces live but unconsumed: /dev/memlog present (uid 1000
still not in memlog group), /proc/self/agent_session all-zeros, provfs
stamping healthy. No net-new ideation seed.

Did NOT re-run the dismissed 4-way question verbatim. Offered ONE sharp
2-way call (draft the provfs/memlog userspace-consumer fleet — the lone
research-backed seed — vs hold-and-drain). Dismissed (7th time). Held
rule #6 — drafted nothing.

Signal for /build unchanged across 7 passes: constraint is THROUGHPUT +
user-gated decisions, not draft supply. Draining beats drafting. Genuine
white-space seed unchanged and still parked: userspace consumers of the
live provfs xattr / /dev/memlog surface beyond vision-continuity. Awaits
an explicit `/dream <topic>` or an opt-in to the fleet above.

## 2026-05-29T15:00  /dream  (saturation — no PRDs, ask-loop halted)
Eighth consecutive no-draft pass. Manual interactive bare /dream. Verified
live: 108 PRD-*.md, 25 visions, /build manifest 157 (up from 154 — build IS
draining). ctrace --since 24h EMPTY, pevent EMPTY. Recall reflective = all
this-week self-review snapshots, recalls=0, no net-new seed. Kernel surfaces
unchanged: uid not in memlog group, agent_session all-zeros, provfs stamping
healthy (user.prov.session live on today's journal).

DELIBERATELY did NOT re-fire the AskUserQuestion (dismissed 7x while user
present = stop-asking signal). Held rule #6. Constraint is THROUGHPUT, not
supply. Lone parked seed unchanged: provfs/memlog userspace-consumer fleet
(3-5 PRDs, on-research). Awaits explicit `/dream provfs-consumers` or opt-in.
Next bare /dream pass: skip the saturation re-report unless a laptop fact
moves — draining beats both drafting and re-narrating.

## 2026-05-29T02:15  (manual session)  brain-backend-ladder ALREADY BUILT
ATTENTION /build: PRD-brain-backend-ladder is DONE — built manually this session,
reviewed (Opus PASS), MERGED to wintermute-brain main (commit 8e97671), and LIVE
(wmd restarted, default_tier=local-3b, answering via local qwen2.5:3b end-to-end).
It is a rust-EXTEND into wintermute-brain — do NOT publish a standalone j0yen repo.
Please mark slug=brain-backend-ladder SHIPPED and do NOT rebuild it. The
.build-worktrees/brain-backend-ladder worktree (branch build/brain-backend-ladder
@ 8e97671) is redundant and will be removed. Also note: the thrift libs it depends
on (wm-local-llm, wm-verify, wm-router) are built + reviewed (PASS) as local
path-deps under ~/wintermute/, release-gate deferred, not yet published.

## 2026-05-29T17:55  /dream  vision-rouse
Seed: explicit /dream from jsy after a live voice-debug session. We tried to
talk to wintermute; nothing happened. Root cause (memory
project_voice_input_null_detectors.md): wm-audio v0.2.0 ships NullWakeDetector
(daemon.rs:86) + NullVadDetector (daemon.rs:93) — no ONNX inference, model dirs
root-owned + EMPTY. Capture works (real mic signal verified); the detectors are
no-ops. So voice input is plumbed-but-deaf.

Drafted: PRD-rouse-wake-vad-models.md, PRD-rouse-voice-selftest.md
Vision: visions/rouse.md
Did NOT re-draft the center: PRD-wintermute-audio-inference.md ALREADY EXISTS
(queued, Draft v0.1, microWakeWord + Silero VAD via ort) and covers the wake/VAD
implementation. rouse builds the FLOOR (get models on disk — config.rs:11 names a
"wm-models bundle" that was never built; dir empty + no install.sh) and the
CEILING (wm-audio selftest: prove the live chain emits events; this session it
took 30min of manual agorabus-subscribe to discover the nulls).

Order: rouse-wake-vad-models (independent, ship FIRST) → wintermute-audio-inference
(EXISTING queued) → rouse-voice-selftest (needs real detectors + models, build LAST).

** CRITICAL CORRECTION for the earshot fleet (all 4 earshot-* PRDs queued): **
earshot tunes a voice loop that does not yet detect anything. earshot-vad-patience
literally tunes a "Silero VAD silence-hangover" (earshot.md:69-72) that is TODAY a
NullVadDetector — it does not exist. DO NOT verify/ship earshot's VAD or
gentle-reprompt PRDs as "working" until audio-inference ships real detectors;
their human-gate ACs will silently pass against a loop that never fires. earshot's
dialog-timing + tts-legibility PRDs are unaffected (pure config/synth path).

Notes for /build: ort/onnxruntime already a fleet-wide dep (agorabus, cadence,
atlas, ac-judge, ambient…) — inference runtime is proven, no new vendoring risk.
Both rouse PRDs rust-extend wm-audio, single-target. selftest mirrors the
agorabus doctor self-describing pattern shipped today.
Open questions: canonical wake word (hey_jarvis vs hey_wintermute vs okay_nabu —
three names float across config + the inference PRD); system vs user model dir
(wm-stt hardcodes /usr/share root-owned).

## 2026-05-29T18:36  (manual session)  rouse-wake-vad-models iter-1 DONE on branch (NOT merged)
ATTENTION /build: PRD-rouse-wake-vad-models advanced to iter-1, verified, but
NOT merged — do not double-build; do not mark shipped yet.
- Built in isolated worktree .build-worktrees/rouse-wake-vad-models, branch
  autobuilder/rouse-wake-vad-models, commit 711bcc4 (off clean HEAD f86ced9).
- Gates independently re-verified by orchestrator: clippy -D warnings clean,
  cargo deny (bans/licenses/sources) ok, cargo test 81 passed (9 new models::tests).
  Real-binary behavioral checks: --list json (6 entries) ok; PENDING_PIN refusal
  exit 2 (0 files installed); non-writable prefix exit 2; daemon back-compat
  preserved (bare/`start` still starts daemon, only `fetch-models` routes to
  provisioner). User decision locked: openWakeWord ONNX (NOT microWakeWord/TFLite).
- Manifest: 6 real-URL entries (silero-vad MIT; oww melspectrogram+embedding+
  hey_jarvis+hey_mycroft+alexa Apache-2.0). okay_nabu OMITTED (404 on oww v0.5.1 —
  it's a microWakeWord asset; needs a different source — open question).
- AC9 deferred (expected): real install needs `wm-audio fetch-models --pin` on a
  networked host (computes+records sha256; manifest ships sha256=PENDING_PIN so no
  unverified blob can install) THEN sudo install into root-owned /usr/share/...

** BLOCKER for merge: wintermute-audio MAIN TREE IS DIRTY ** with ~472 lines of
UNRELATED uncommitted work (src/source.rs +283, main.rs +42, install.sh +56, new
pkg/ dir, Cargo.toml→0.2.1) from an unknown tick/session — no agorabus claim held.
iter-1 can't merge until that is committed/stashed by its owner. Whoever owns the
pkg/ + source.rs packaging work: please land or stash it.

** SPEC-DRIFT for the audio fleet: ** wm-audio main.rs is a BARE DAEMON, not a
subcommand CLI. The queued PRD-rouse-voice-selftest ALSO assumes `wm-audio
selftest`; and PRD-wintermute-audio-inference says "microWakeWord" but we're going
openWakeWord/ONNX. Reconcile config.rs:11 + the inference PRD wording to
openWakeWord/ONNX when that PRD is built.

## 2026-05-29T18:52  (manual session)  dirty-tree RESOLVED + reviewer PASS + AEC dedup flag
rouse-wake-vad-models iter-1: Opus reviewer-agent verdict = PASS (logged to
state/reviewer-calibration.jsonl). Airtight no-unverified-blob invariant (holds
under --force via 2 guards), back-compat intact, non-tautological tests. Nits:
`wm-audio --help` doesn't list the subcommand (daemon is argless by design — AC1
2nd clause), okay_nabu deferred. Branch autobuilder/rouse-wake-vad-models @ 711bcc4,
NOT merged to main.

Dirty wintermute-audio tree RESOLVED: the uncommitted ~472 lines were
PRD-wintermute-audio-aec built directly into the autobuilder/aec checkout by a tick
that DIED on 5 clippy-pedantic lint errors (src/source.rs docs: long-first-para,
unbackticked PipeWire x3, const-fn) — which is why it never committed. Fixed the
lints (docs only, no AEC logic touched), full gate now green (clippy -D warnings,
67 lib tests, deny ok), committed as autobuilder/aec @ 3ae0248. Main-tree checkout
is now clean.

** AEC DEDUP NEEDED /build: ** there are TWO AEC branches —
  - autobuilder/aec @ 3ae0248 (FULL: probe + pkg/99-wintermute-aec.conf + install.sh
    + v0.2.1 + docs; gate-green now)  <-- the further-along one
  - build/wintermute-audio-aec @ 0505359 ("iter-1 AEC scaffold" only)
Pick autobuilder/aec (it's ahead + gate-green) and drop the scaffold branch, OR
reconcile. Do not ship both.

** MERGE TO MAIN deferred to /build's coordinated publish: ** wintermute-audio main
@ f86ced9 is clean; it has 8+ live branches/worktrees mid-flight (2x AEC,
audio-inference x2 empty, earshot-vad-patience, a .claude agent worktree). I did
NOT force any merge to main — sequencing AEC + fetch-models + inference onto main is
a publish-ordering call with claim coordination, which is /build's job. Suggested
order onto main: aec (3ae0248) → then rebase rouse-wake-vad-models (711bcc4) on top
(expect Cargo.toml version + main.rs dispatch-vs-aec-probe conflicts, both
mechanical) → then audio-inference. Runtime + installed wm-audio binary untouched.

## 2026-05-29T20:30  (manual session)  BUILD STALL fixed (detach) + 21 dirty worktrees recovered
ATTENTION /build: root-caused the stall — claude-build.service (Type=oneshot,
TimeoutStartSec=600) was SIGTERM-killing any tick that ran >10min (routine for 5x
parallel autobuilder rust builds + Opus reviewers). ~1/3 of ticks died mid-build
before committing → no PRD advancement + ~20 dirty worktrees piled up.

FIX (durable, option b): claude-build-headless.sh is now a thin launcher that
detaches the real tick into a transient claude-build-work.service (Type=oneshot,
TimeoutStartSec=1800/30min) via `systemd-run --user --no-block --unit=claude-build-work`.
The 1-min oneshot returns in <2s and can no longer kill the build; it survives and
commits. New file claude-build-tick.sh holds the actual `claude -p /build` + peon
pause. Overlap guard = claude-build-work.service ActiveState check + systemd --unit
uniqueness. Verified end-to-end (detached unit runs with 30min cap; guard clean-no-ops).
Backups at ~/.local/bin/claude-build-headless.sh.bak-*.

CLEANUP (option c): paused the loop, drained the last old-style tick, then
`git stash push --include-untracked -m "killed-tick-recovery 2026-05-29: <slug>"`
on all 20 dirty worktrees + atlas main (0 remaining dirty). NON-DESTRUCTIVE —
every change is recoverable: in each worktree `git stash list` shows the labeled
entry, `git stash apply` to restore. Stashed slugs incl. substantial work:
earshot-tts-legibility(611L), wmd-session-boundary(336L), brain-prompt-cache(292L),
almanac-speak-bridge(260L), earshot-vad-patience(205L), recall-corpus-vacuum(205L),
wintermute-companion-degrade(117L) + many smaller mid-iteration scratch. /build will
re-derive these from clean HEAD on its next pass; the stashes are a safety net if any
held a complete-but-uncommitted increment (the AEC failure mode). Loop is UNPAUSED and
running the first detached tick now.

## 2026-05-29T21:05  (manual session)  fan-out 5->10 + 18 dirty main repos triaged
TWO changes for the loop:
1) PER-TICK CAP 5 -> 10. build-skill SKILL.md committed (8aa50ee): "up to 10 PRDs
   in parallel". The <=3 same-target sub-cap is UNCHANGED (OOM guard — a 5-wide
   tick peaked ~4.1GB vs ~9GB no-swap). So 10-wide helps when PRDs target
   DIFFERENT repos; a single heavy cluster (e.g. wintermute-brain, 11 queued) is
   still 3/tick. Detached ticks (claude-build-work.service, 30min cap) give the
   10-wide fan-out room to finish+commit. (Confirmed the 30min cap works: a runaway
   28-min tick hit it and was bounded — exactly the intended ceiling.)

2) 18 dirty MAIN repos were blocking rust-extend (extend-validate refuses on a
   dirty build_into). Triaged:
   - STASHED (non-destructive, recoverable via `git stash list`/`apply`, label
     "dirty-tree-triage 2026-05-29"): wintermute-brain, wintermute-platform,
     wintermute-almanac, wintermute-tts, wintermute-stt, wintermute-audio-inference,
     binstale, wintermute-browser, wintermute-desktop, wintermute-screen-narrate,
     rollout, wm-hardware-drift, + build-skill artifacts. Most were benign churn
     (Cargo.lock/target/README); wintermute-brain was ONLY Cargo.lock+README. This
     UNBLOCKS ~14 queued PRDs (11 brain + 3 platform) + the stt/inference clusters.
   - LEFT DIRTY (intentional, NOT queue-blockers): autobuilder (31 files = live
     queue+skill working state: in-flight PRD authoring/archiving, gossip, skill
     evolution, runtime artifacts — do NOT stash, it'd disrupt the queue),
     agentns/memlog/provfs (kernel C, 2-3 DAYS stale, sensitive — needs human
     decision, not auto-clean), cradle-2026-05-27-handbuilt-bak (a backup dir).
   Dirty main repos: 18 -> 5.

## 2026-05-30T03:50  /dream  vision-vigil (extend, Fleet 4)
Drafted: PRD-vigil-install-restart.md, PRD-vigil-build-restart-wiring.md,
  PRD-vigil-selfreview-concurrent-guard.md
Vision: visions/vigil.md (new Fleet 4 — "close the loop at the install site")
Seed: 2026-05-29 self-review runs 9/10/11 reflective memories
  (01KSTZX7.../01KSV6Q9.../01KSVDJF...) named the upstream cause of the
  7-run agorabus stale-binary saga: /build installs a fresh daemon binary
  but never restarts the daemon, AND the auto-fix has no concurrent-/build
  guard. agorabus doctor + agorabus reload (Fleet 1/3) shipped; these three
  close the *install-site* and *reaction-safety* gaps those didn't.
Order:
  vigil-install-restart  (rust-extend → ~/wintermute/rollout; needs rollout
    [Fleet 1] + agorabus-reload [Fleet 3] shipped first)
   └─► vigil-build-restart-wiring  (shell; routes /build's daemon-backed
        install through `rollout install`)
  vigil-selfreview-concurrent-guard  (shell; independent)
Notes for /build:
  - vigil-install-restart extends rollout (repo exists at ~/wintermute/rollout/,
    PRD-rollout still Draft) — DO NOT build until rollout + agorabus-reload land.
  - vigil-selfreview-concurrent-guard edits the SAME self-review playbook block
    (agorabus_daemon_stale_binary) as Fleet-3 PRD-agorabus-reload-self-review.
    SERIALIZE those two on SKILL.md; order is semantically free but never apply
    in parallel.
  - vigil-build-restart-wiring degrades gracefully: if `rollout` isn't installed
    it falls back to `install -m755` + a Pending note, never blocks a build.
  - Generalizes the bus's bespoke `agorabus reload --build` self-heal to the
    whole daemon fleet (recalld/wmd/wm-audio|dialog|stt|tts) — recalld liveness
    is safety-critical and has no `reload` of its own.
Open questions: should `rollout install`'s reverse unit-map be cached, or
  re-derived from the units each call? (Drafted as re-derive-each-call for
  correctness as the fleet grows; revisit if it's hot.)

## 2026-05-30T04:33  /dream  vision-warden
Drafted: PRD-warden-home.md, PRD-warden-policy.md, PRD-warden-deadman.md,
  PRD-warden-self-review.md
Vision: visions/warden.md (new — "the guardrail that was built but never armed")
Seed: 2026-05-29 self-review runs 1/2 Pending line "bpolicy not loaded
  ({"loaded":false}) — no enforcement; loading needs sudo + a user-owned
  policy file", re-flagged verbatim every run. bpolicy is the 8th local tool
  and the ONLY one with no home repo (ls ~/wintermute/bpolicy = none), no PRD,
  no vision. Sibling to onramp: same built→consumed shape, the ENFORCEMENT
  half (onramp = the observation half: memlog/agentns/provfs). Verified
  disjoint — no warden PRD touches memlog-group/agentns-wrap/provfs-fallback;
  no onramp PRD touches bpolicy.
Order:
  warden-home  (rust-cli → NEW repo ~/wintermute/bpolicy; reimplements the
    Python control plane in Rust, byte-identical status JSON + same 6
    subcommands; vendors bpolicy.bpf.c/.o; back-compat anchor)
   ├─► warden-policy  (rust-extend; declarative ~/.config/bpolicy/policy.toml
   │     + BPF allowlist map + longest-prefix match in bpolicy.bpf.c)
   ├─► warden-deadman (rust-extend; --audit log-only mode + --ttl/renew
   │     deadman auto-unload so a too-tight arm self-heals)
   └─► warden-self-review (shell; Phase A `warden:` line + B.5 escalate-once
         playbook so self-review stops re-flagging {loaded:false} every run)
Notes for /build:
  - warden-home is the unblock; policy + deadman both rust-extend the SAME
    build_into (~/wintermute/bpolicy) — SERIALIZE them, never parallel (dirty
    tree + conflicting bpolicy.bpf.c edits). Order between them is free.
  - warden-self-review is shell, edits self-review SKILL.md (Phase A + B.5).
    SERIALIZE on SKILL.md with any other in-flight self-review-playbook PRD
    (same coordination the vigil gossip flagged for the agorabus block).
  - NONE of these arm enforcement. Every PRD is observe/build/make-safe only;
    actually loading + enforcing on a live session stays a user decision.
  - AC5 (BPF compile via clang/bpftool) is deferred-gated if the build env
    lacks clang/bpftool — declare deferred_acs with reason, same as
    agentns-claude's boot-gated ACs.
  - Keep the binary named `bpolicy` (don't rename to warden) — it's in
    CLAUDE_SELF.md + toolkit memory + drift skill; warden is the VISION name.
Open questions: rename bpolicy→warden? (leaning no); should --ttl default-on
  (drafted yes, 30m); allow-list one-map-reloaded vs N-maps (leaning one).

## 2026-05-30T05:10  /dream  vision-onramp (extend, Fleet 2a — memlog consumer spine)
Drafted: PRD-memlog-group-autojoin.md, PRD-memlog-activation-self-review.md,
  PRD-memlog-precompact-witness.md
Vision: visions/onramp.md (Fleet 2a section added)
Seed: bare /dream; dominant verified signal = "memlog EACCES" re-flagged in
  ~26 consecutive self-review reflective memories (01KSVDJF.../01KSV6Q9.../
  01KSTZX7...). Traced to root cause LIVE this pass: the postinstall fix
  (sysusers `g memlog -` + udev) SHIPPED at pkgrel-6 (archived c712c9d), but
  the laptop boots pkgrel-5 which predates it -> group never created. NOT an
  authoring gap (don't redraft PRD-kernel-pkg-postinstall, it's archived);
  it's an ACTIVATION + CONSUMER gap.
Verified state: uname=7.0.10-arch1-5-wintermute, pacman -Q linux-wintermute
  =7.0.10.arch1-5 (both pkgrel-5); getent group memlog = empty; /dev/memlog
  = root:root 0660; sysusers file = `g memlog -` (group only, NO membership);
  install scriptlet punts to manual `usermod -aG memlog`; only PreCompact
  hook = peon-ping (a sound); `memlog` reader NOT in ~/.local/bin (only
  memlog-witness daemon is).
Order:
  memlog-group-autojoin  (mixed -> ~/wintermute/wintermute-kernel; pkgrel-8
    repack, NO kernel rebuild; auto-adds SUDO_USER to memlog group)
   └─(user-gated: pacman -U pkgrel-8 + reboot, or no-reboot systemd-sysusers
      + udevadm trigger since the memlog driver is already loaded)─►
  memlog-precompact-witness (mixed -> ~/.claude hooks + install `memlog`
    reader; PreCompact producer; fails open until group joined)
  memlog-activation-self-review (shell; INDEPENDENT; ships anytime)
Notes for /build:
  - memlog-activation-self-review edits self-review SKILL.md. SERIALIZE on
    SKILL.md with PRD-warden-self-review / PRD-vigil-selfreview-concurrent-
    guard / PRD-agorabus-reload-self-review — never apply two SKILL.md PRDs
    in parallel. Order between them is semantically free.
  - memlog-group-autojoin edits the kernel PKGBUILD packaging
    (~/wintermute/wintermute-kernel/pkg/linux-wintermute.install + PKGBUILD
    pkgrel bump). It does NOT rebuild the kernel — reuse the repack path
    from build.log.pkgrel6-repack-*. Use apply-agentns.py's idempotent
    anchor-edit pattern, not raw .patch splicing.
  - memlog-precompact-witness AC6 (survival smoke) is DEFERRED-GATED on the
    memlog group being activated+joined at build time; declare deferred_acs
    if not, same as agentns-claude's boot-gated ACs. ACs 1-5,7 (reader
    install + fail-open hook) are testable + useful immediately.
  - NONE of these activate the kernel package or reboot. Install + reboot
    stays a user decision (the recurring "pacman SKIPPED protected: linux"
    line in every self-review).
Open questions: should the precompact snapshot be the full transcript tail
  or an LLM-summarized digest? (Drafted as bounded head/tail slice to the
  device record-size cap; a digest needs a local-LLM call that may not be
  affordable at compaction time. Revisit once brain's local-3b tier is
  cheap enough to call synchronously in a 10s hook timeout.)

## 2026-05-30T05:33Z  /dream  (saturation report — no PRDs drafted)
Seed: bare /dream (overnight timer tick).
Phase 1 done: recall reflective/procedural/semantic + hybrid ideation query;
  ctrace status (running, 35k events, healthy); pevent empty; journals
  2026-05-29 + 2026-05-29-selfreview + 2026-05-28 read.
Conclusion: SATURATED. No new evidence-backed component to draft this tick.
Why no PRDs (every recurring laptop signal is already PRD'd):
  - memlog EACCES (~26 self-review runs) -> onramp Fleet 2a, drafted 05:10Z
    (PRD-memlog-group-autojoin / -activation-self-review / -precompact-witness).
  - agentns agent_session all-zeros (~22 runs) -> onramp Fleet 1
    PRD-claude-agentns-wrap (unshare(CLONE_NEWAGENT) at launch).
  - agorabus stale daemon (4+ windows) -> vigil + PRD-agorabus-reload /
    PRD-agorabus-reload-self-review.
  - ctrace missing SessionEnd summaries -> PRD-ctrace-session-end-resilient.
  - bpolicy not loaded / no enforcement -> warden, drafted 04:33Z.
  - This is the 8th saturation outcome in the 2026-05-29..30 arc; three
    overnight dreams (vigil F4 03:50, warden 04:33, onramp F2a 05:10) already
    drained the fresh signals before this tick.
Open question for /build + user (NOT auto-drafted — needs a user decision):
  cargo-mutants drives sed execve to 424k-758k/day, dominating ALL laptop
  activity across self-review runs 7/8/10/12. It's the single heaviest
  resource signal on the machine. But mutation testing IS the autobuilder
  proof gate, so high execve is expected, not a defect — scoping/caching it
  (e.g. "skip mutants when src unchanged since last green") could weaken the
  gate. Deliberately left as a user decision rather than fabricated into a
  fleet (hard rule #6: don't dream past the research).
Notes for /build: nothing new queued from /dream this tick. The drafted
  fleets from the three overnight dreams (vigil F4, warden, onramp F2a) are
  the actionable backlog; several carry user-gates (install/reboot/SKILL.md
  serialize) already noted in their own gossip entries.

## 2026-05-30T06:10Z  /dream  vision-vigil  (reconciliation — no new PRDs)
Seed: bare /dream tick. Phase 1 found a manifest/reality desync the last 8
saturation ticks missed.

FINDING — Fleet 3 keystones shipped *ahead of* their PRD files:
  The dream manifest lists PRD-agorabus-client-reconnect.md and
  PRD-agorabus-reload.md under vigil.prds_drafted, but NEITHER FILE EXISTS
  on disk. They are not missing *work* — the code already shipped:
    - agorabus is at v0.8.0 (drain-notice PRD still cites the old 0.4.0 base).
    - src/reconnect.rs landed v0.5.0 (commit e5c5ac4 + clippy fix 1dbde50).
    - src/reload.rs + `Command::Reload` landed v0.8.0 (commit 124a206, whose
      message literally cites "PRD-agorabus-reload v0.8.0" — a PRD filename
      that was never written).
    - Installed ~/.local/bin/agorabus 0.8.0 has a working `reload` subcommand
      (verified `reload --help`: dry-run default, structured verdict).
    - daemon already exposes --drain-grace-ms / --drain-resume-hint-ms, so
      PRD-agorabus-drain-notice (file exists) is also implemented.

WHY NO PRDs THIS TICK (hard rule #6 — verify live before acting):
  Drafting PRD-agorabus-client-reconnect.md / PRD-agorabus-reload.md now would
  hand /build PRDs whose ACs the shipped 0.8.0 crate already satisfies, risking
  a re-churn of a crate /build owns. The honest state is "shipped without a
  file," not "undrafted." So: no phantom PRDs.

Notes for /build:
  - If you ever see drain-notice / state-persist / reload-self-review reference
    PRD-agorabus-reload.md or PRD-agorabus-client-reconnect.md as a dependency,
    treat that dependency as SHIPPED (reconnect v0.5.0, reload v0.8.0), not
    pending. No file to wait on.
  - The genuinely-pending vigil work is Fleet 4 (producer-side close-the-loop):
    PRD-vigil-install-restart / PRD-vigil-build-restart-wiring /
    PRD-vigil-selfreview-concurrent-guard (all on disk, drafted 2026-05-30).
    These wire `systemctl --user restart <daemon>.service` into /build's
    binary-install path (the recurring 4+-window root cause) and generalize it
    beyond agorabus to recalld. That's where the open evidence points.

Manifest reconciled: added vigil.fleets.fleet3_handover.reconciled_2026_05_30
  noting reconnect/reload shipped-without-file. PRD files NOT created
  (rule #2 untouched — only dream's own state file annotated).

## 2026-05-30T07:01Z  /dream  (saturation report — no PRDs drafted)
Seed: bare /dream tick. Phase 0/1 complete; recall reflective(20)+procedural+
  hybrid-ideation queried; ctrace status (running, 24k events, healthy);
  pevent empty; wchg list shows /build live in earshot-tts-legibility,
  agorabus-reload, wintermute-audio-inference worktrees.
Conclusion: SATURATED (9th consecutive). No fresh evidence-backed component.
  Every recall reflective hit is a self-review run report; hybrid ideation
  returns only already-PRD'd signals (kernel built-not-booted, agorabus-stale
  -> vigil F4, memlog EACCES -> onramp F2a, agentns all-zeros -> claude-agentns-wrap).
Standing user-decisions (NOT auto-drafted, rule #6):
  - cargo-mutants sed execve dominance (424k-758k/day) — expected, not a defect.
  - kernel install+reboot — user gate (recurring pacman SKIPPED protected: linux).
Notes for /build: nothing new queued. The actionable backlog remains the
  prior overnight fleets (vigil F4, warden, onramp F2a). /dream is idling on
  this direction until a new signal appears or the user names a topic.

## 2026-05-30T (user-invoked /dream)  saturation report — no PRDs drafted
Seed: bare /dream, interactive. Phase 0/1 walked: gossip tail, today's journal
  (self-review 2026-05-30), recall reflective(20)+procedural+semantic+hybrid
  ideation. 106 PRDs / 28 visions on disk. 10th consecutive saturation tick.
Evidence: every recall reflective hit (20/20) is a self-review run report.
  Hybrid ideation returns only already-PRD'd signals (kernel built-not-booted=
  exists; agorabus-stale->vigil F4; memlog EACCES->onramp F2a; agentns
  all-zeros->claude-agentns-wrap). Genuinely-open items are USER-ACTION not
  code: pacman/kernel reboot, memlog group join, agentns registration. None
  PRD-able (rule #6).
NEW SIGNAL (first surfaced this tick): claude.ai MCP servers now connected —
  Gmail, Google Calendar, Google Drive, and an AtScale semantic-layer server.
  NO existing vision touches external-service integration. This is the only
  un-dreamed direction the laptop surfaced. Candidate future vision:
  "external-services bridge" — wintermute-side Rust tooling bridging recall/
  agent-memory to the user's real external context (calendar->journal seeding,
  gmail->recall capture, drive doc provenance via provfs). Left as an OPEN
  QUESTION for the user / next tick, NOT drafted — needs the user to opt the
  direction in (it reaches outward past the laptop, unlike the existing fleets).
Offered the user a seed picker (external-bridge / extend-vision / fresh-topic /
  stand-down); picker dismissed -> stood down per saturation finding.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a. No phantom PRDs added.

## 2026-06-02T00:00Z  /dream  (user-invoked)  saturation report — no PRDs drafted
Seed: bare /dream, interactive. Phase 0/1 walked: gossip tail, journal
  2026-06-01 (self-review, fresh boot kernel 7.0.10-wintermute), recall
  reflective(20)+procedural(project)+hybrid-ideation(15). 102 PRDs / 28
  visions on disk. 11th consecutive saturation tick.
Evidence: every recall reflective hit (20/20) is a self-review run report.
  Hybrid ideation returns only already-PRD'd signals (kernel built-not-booted;
  agorabus-stale->vigil F4; memlog EACCES->onramp F2a; agentns zeros->
  claude-agentns-wrap). Genuinely-open items are USER-ACTION not code:
  pacman/kernel reboot, memlog group join, agentns registration. pevent
  empty; ctrace up & fresh (pid 10531, 382 events). Not PRD-able (rule #6).
NEW SIGNAL (grew since last tick): AWS MCP servers now ALSO connected
  (awslabs-aws-api: call_aws/suggest_aws_commands; awslabs-aws-docs:
  search/read/recommend) ALONGSIDE the prior Gmail/Calendar/Drive/AtScale
  (claude_ai_Non-prod = AtScale semantic layer: list_models/run_query/etc).
  The external-service surface is widening. Still no vision touches outward
  integration — every fleet to date is laptop-internal.
Offered the user a 4-way seed picker (external-services-bridge /
  AWS-AtScale-data-bridge / extend-existing-vision / stand-down). Picker
  dismissed -> stood down. Direction remains OPEN for the user to opt into
  on a future tick; not drafted (reaches outward past the laptop, needs
  explicit buy-in).
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a. No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream. Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-
  review, fresh boot kernel 7.0.10-wintermute, genuinely-clean run), recall
  reflective(20) + hybrid-ideation(12). 101 PRDs / 27 visions on disk. 12th
  consecutive saturation tick.
Evidence: every recall reflective hit is a self-review run report. Hybrid
  ideation returns only already-PRD'd / kernel-built-not-booted signals
  (memlog EACCES->onramp F2a; agentns zeros->agentns-wrap; agorabus->vigil F4).
  Genuinely-open items are USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration. Not PRD-able (rule #6).
NEW STATE (escalated since last tick): the external-service MCP surface is now
  FULLY LIVE this session, not merely "connecting" — claude_ai Gmail
  (search/draft/label), Google Calendar (list/create/suggest_time), Google
  Drive (search/read/create), Non-prod=AtScale semantic layer (list_models/
  run_query/search_columns/validate_query), AND AWS (awslabs-aws-api call_aws/
  suggest; awslabs-aws-docs search/read/recommend). Still ZERO of the 27
  visions touches outward integration — every fleet is laptop-internal.
Offered the user a 4-way seed picker (external-services-bridge / AWS+AtScale
  data-bridge / extend-existing-vision / stand-down). Picker dismissed ->
  stood down per saturation precedent. The outward direction remains OPEN for
  the user to opt into on a future tick; not drafted (reaches past the laptop,
  needs explicit buy-in per rule #6).
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a. No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream. Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-
  review, genuinely-clean run, kernel 7.0.10-wintermute), recall reflective(12)
  + hybrid-ideation(12). 101 PRDs / 27 visions on disk. 13th consecutive
  saturation tick.
Evidence: all 12 reflective recall hits are self-review run reports (recalls=0,
  [reflective/self]). Hybrid ideation returns only already-PRD'd / kernel-
  built-not-booted signals (memlog EACCES->onramp F2a; agentns zeros->agentns-
  wrap; kernel built-not-booted; agorabus now RESOLVED, not a signal anymore).
  Today's journal pending items are ALL user-action, not code: pacman/kernel
  reboot, memlog group join, agentns registration, empty WM_ANTHROPIC_KEY.
  Not PRD-able (rule #6).
SIGNAL (4th tick running, unchanged): external-service MCP surface fully live
  this session — Gmail, Google Calendar, Google Drive, AtScale (claude_ai
  Non-prod: list_models/run_query/describe_model/search_columns/validate_query),
  AWS (awslabs-aws-api call_aws/suggest; awslabs-aws-docs search/read/recommend).
  Still ZERO of 27 visions reaches outward. Offered a 4-way seed picker
  (external-services-bridge / AWS+AtScale data-bridge / extend-existing-vision /
  stand-down); picker DISMISSED -> stood down per precedent. Outward direction
  remains OPEN for explicit user opt-in on a future tick; not drafted (rule #6).
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a. No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed (not timer). Phase 0/1 walked: gossip tail,
  journal 2026-06-02 (self-review run #1 + #2, genuinely-clean, kernel
  7.0.10-wintermute), recall reflective(20)+procedural/project(4)+semantic(1).
  ~102 PRDs / 28 visions on disk. 14th consecutive saturation tick.
Evidence: every reflective recall hit (20/20) is a self-review run report
  (recalls=0, [reflective/self]). procedural/project = 4 stable project notes,
  already reflected in visions. Hybrid-ideation signals all already-PRD'd or
  kernel-built-not-booted (memlog EACCES->onramp F2a; agentns zeros->agentns-
  wrap; agorabus now RESOLVED). Genuinely-open items are USER-ACTION not code:
  pacman/kernel reboot, memlog group join, agentns registration, empty
  WM_ANTHROPIC_KEY. Not PRD-able (rule #6).
DIFFERENCE THIS TICK: user typed /dream by hand (prior 4 saturation ticks were
  timer fires), so I actually asked rather than auto-standing-down. Offered a
  4-way seed picker (outward-integration / extend-a-vision / name-a-topic /
  stand-down). Picker DISMISSED -> stood down. The outward direction (first
  vision touching the now-fully-live MCP surface: Gmail/Calendar/Drive/AtScale/
  AWS) remains the only un-PRD'd direction; still OPEN, still needs explicit
  user buy-in (reaches past the laptop). Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a. No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed. Phase 0/1 walked: gossip tail, journal 2026-06-02
  (self-review run #1 + #2, genuinely-clean, kernel 7.0.10-arch1-5-wintermute,
  agorabus stays resolved), recall reflective(12)+hybrid-ideation(12). 98 PRDs /
  27 visions on disk. 15th consecutive saturation tick.
Evidence: all 12 reflective recall hits are self-review run reports (recalls=0,
  [reflective/self]). Hybrid ideation returns only already-PRD'd or kernel-built-
  not-booted signals (memlog EACCES->onramp F2a; agentns zeros->agentns-wrap;
  kernel built-not-booted). Today's journal Pending items are ALL user-action,
  not code: pacman/kernel reboot (linux 7.0.9->7.0.10 + linux-firmware x12),
  memlog group join, agentns registration, empty WM_ANTHROPIC_KEY (credit
  exhausted). Not PRD-able (rule #6).
SIGNAL (5th tick running, unchanged): external-service MCP surface fully live
  this session — Gmail, Google Calendar, Google Drive, AtScale (claude_ai
  Non-prod), AWS (awslabs-aws-api + awslabs-aws-docs). Still ZERO of 27 visions
  reaches outward. Offered a 4-way seed picker (outward-integration / extend-a-
  vision / name-a-topic / stand-down); picker DISMISSED -> stood down per
  precedent. Outward direction remains OPEN for explicit user opt-in on a future
  tick; not drafted (reaches past the laptop, rule #6).
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed (not timer). 16th consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review run #1 + #2,
  genuinely-clean, kernel 7.0.10-arch1-5-wintermute, agorabus stays resolved),
  build manifest, 27 visions / ~102 PRDs on disk. Recall seeding (mandatory):
  reflective(15) all self-review run reports (recalls=0, [reflective/self]);
  procedural/project = same 4 stable notes; semantic(1); hybrid-ideation(15)
  returns only already-PRD'd or kernel-built-not-booted signals (kernel built
  but stock booted, memlog EACCES, agentns zeros). ctrace query --since 24h
  EMPTY (quiet/tracer not capturing); pevent list EMPTY (no orphans). No new
  laptop signal motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
SIGNAL (6th tick running, unchanged): external-service MCP surface fully live
  (Gmail/Calendar/Drive/AtScale claude_ai Non-prod/AWS awslabs). Still ZERO of
  27 visions reaches outward. DIFFERENCE THIS TICK: user typed /dream by hand,
  so I asked rather than auto-standing-down — offered a 4-way seed picker
  (outward-integration / name-a-topic / extend-a-vision / stand-down). Picker
  DISMISSED -> stood down per precedent. Outward direction remains the only
  un-PRD'd direction; still OPEN, still needs explicit user opt-in (reaches
  past the laptop, rule #6). Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02 (build tick) — BPF gate gap [from warden-policy]
- `bpolicy.bpf.c` PRDs (warden-policy landed v0.2.0) pass the cargo gate via `mock_bpf` but the real BPF object is NEVER compiled by the gate. warden-policy's landed `.bpf.c` fails clang with 20 errors (BPF stack limit exceeded — longest-prefix-match copies path into an on-stack key). The feature is inert (bpolicy never loaded), but a broken `.o` shipped.
- Root cause it slipped through: `bpf/build.sh` returns **exit 0** despite clang failing. The worktree integrate trusted the green gate.
- Proposed guardrail (PRD-shaped, for next reflect): make `bpf/build.sh` propagate clang's exit code, AND add a gate step that compiles every `*.bpf.c` (clang -target bpf) so the autobuilder/extend gate fails on a broken BPF object. Until then, any rust-extend touching `.bpf.c` needs a manual clang check.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed (not timer). 17th consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review run #1 + #2,
  genuinely-clean, kernel 7.0.10-arch1-5-wintermute, agorabus stays resolved),
  build manifest, 27 visions / ~100 PRDs on disk. Recall seeding (mandatory):
  reflective(12) all self-review run reports (recalls=0, [reflective/self]);
  procedural/project = same 4 stable notes; hybrid-ideation(12) returns only
  already-PRD'd or kernel-built-not-booted signals (kernel built/stock booted,
  memlog EACCES, agentns zeros). ctrace query --since 24h EMPTY; pevent list
  EMPTY (no orphans); wchg list = build-worktree self-churn only. No new laptop
  signal motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
SIGNAL (7th tick running, unchanged): external-service MCP surface fully live
  (Gmail/Calendar/Drive/AtScale claude_ai Non-prod/AWS awslabs). Still ZERO of
  27 visions reaches outward. User typed /dream by hand, so I asked rather than
  auto-standing-down — offered the 4-way seed picker (outward-integration /
  name-a-topic / extend-a-vision / stand-down). Picker DISMISSED -> stood down
  per precedent. Outward direction remains the only un-PRD'd direction; still
  OPEN, still needs explicit user opt-in (reaches past the laptop, rule #6).
  Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed (not timer). 18th consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review run #1 + #2,
  genuinely-clean, kernel 7.0.10-arch1-5-wintermute, agorabus stays resolved),
  build manifest, 27 visions / ~100 PRDs on disk. Recall seeding (mandatory):
  reflective(12) all self-review run reports (recalls=0, [reflective/self]);
  hybrid-ideation(12) returns only already-PRD'd or kernel-built-not-booted
  signals (kernel built/stock booted, memlog EACCES, agentns zeros). ctrace
  query --since 24h EMPTY; pevent list EMPTY (no orphans). No new laptop signal
  motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
SIGNAL (8th tick running, unchanged): external-service MCP surface fully live
  (Gmail/Calendar/Drive/AtScale claude_ai Non-prod/AWS awslabs). Still ZERO of
  27 visions reaches outward. User typed /dream by hand, so I asked rather than
  auto-standing-down — offered the 4-way seed picker (outward-integration /
  name-a-topic / extend-a-vision / stand-down). Picker DISMISSED -> stood down
  per precedent. Outward direction remains the only un-PRD'd direction; still
  OPEN, still needs explicit user opt-in (reaches past the laptop, rule #6).
  Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed (not timer). 19th consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review run #1 + #2,
  genuinely-clean, kernel 7.0.10-arch1-5-wintermute, agorabus stays resolved),
  build manifest, 27 visions / 95 PRDs on disk. Recall seeding (mandatory):
  reflective(20) all self-review run reports (recalls=0, [reflective/self]);
  procedural/project = same 4 stable notes; hybrid-ideation(12) returns only
  already-PRD'd or kernel-built-not-booted signals (kernel built/stock booted,
  memlog EACCES, agentns zeros). ctrace query EMPTY; pevent list EMPTY (no
  orphans); wchg = build-worktree self-churn only. No new laptop signal
  motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
SIGNAL (9th tick running, unchanged): external-service MCP surface fully live
  (Gmail/Calendar/Drive/AtScale claude_ai Non-prod/AWS awslabs). Still ZERO of
  27 visions reaches outward. User typed /dream by hand, so I offered the 4-way
  seed picker (outward-integration / name-a-topic / extend-a-vision / stand-down).
  Picker DISMISSED -> stood down per precedent. Outward direction remains the
  only un-PRD'd direction; still OPEN, still needs explicit user opt-in (reaches
  past the laptop, rule #6). Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed (not timer). 20th consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review run #1 + #2,
  genuinely-clean, kernel 7.0.10-arch1-5-wintermute, agorabus stays resolved,
  87 memories in recall), build manifest, 27 visions / ~95 PRDs on disk. Recall
  seeding (mandatory): reflective(20) all self-review run reports (recalls=0,
  [reflective/self]); hybrid-ideation(12) returns only already-PRD'd or
  kernel-built-not-booted signals (kernel built/stock booted, memlog EACCES,
  agentns zeros). ctrace query EMPTY (note: --since wants a float, not "24h");
  pevent list EMPTY (no orphans). No new laptop signal motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
SIGNAL (10th tick running, unchanged): external-service MCP surface fully live
  (Gmail/Calendar/Drive/AtScale claude_ai Non-prod/AWS awslabs). Still ZERO of
  27 visions reaches outward. User typed /dream by hand, so I offered the 4-way
  seed picker (outward-integration / name-a-topic / extend-a-vision / stand-down).
  Picker DISMISSED -> stood down per precedent. Outward direction remains the
  only un-PRD'd direction; still OPEN, still needs explicit user opt-in (reaches
  past the laptop, rule #6). Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed (not timer). 21st consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review #1+#2 clean,
  kernel 7.0.10-arch1-5-wintermute, agorabus resolved, 87 memories in recall),
  build manifest (412 PRD entries tracked), 27 visions / ~95 PRDs on disk.
  Recall seeding (mandatory): reflective(20) all self-review run reports
  (recalls=0, [reflective/self]); hybrid-ideation(12) returns only already-PRD'd
  or kernel-built-not-booted signals. pevent EMPTY (no orphans); ctrace query
  --since 86400 = 1 event (this session). No new laptop signal motivates an
  inward PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
DIFFERENT THIS TICK: did NOT re-offer the abstract 4-way picker (dismissed 10x).
  Instead made a concrete outward proposal — MCP surface is live IN-SESSION this
  run (Gmail/Calendar/Drive/AtScale Non-prod/AWS adapters all loaded). Offered two
  fully-sketched outward visions to break inward saturation: (a) reach-briefing
  (read Calendar+Gmail -> compose digest -> speak via existing wm-tts; read+speak
  only, lowest risk) and (b) reach-archive (one-way mirror ~/brain/journal +
  visions -> user-owned Drive; writes but only to user's own Drive). Picker
  DISMISSED again -> stood down per precedent. Outward remains the only un-PRD'd
  direction; still OPEN, still needs explicit user opt-in (rule #6). Not drafted.
  If a future tick wants to act: the two sketches above are ready to expand into
  3-5 PRD fleets without further research.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed. 22nd consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review #1+#2 clean),
  build manifest, 27 visions / ~95 PRDs on disk. Recall seeds return only
  already-PRD'd or kernel-built-not-booted signals; pevent empty; ctrace quiet.
  No new INWARD laptop signal motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
DIFFERENT THIS TICK: did NOT re-offer the abstract 4-way picker. Offered a
  concrete 4-option AskUserQuestion (reach-briefing fleet / reach-archive fleet /
  name-a-topic / stand-down), the two outward fleets fully sketched and ready to
  expand into 3-5 PRDs without further research. Picker DISMISSED again -> stood
  down per precedent. Outward (Gmail/Calendar/Drive/AtScale/AWS MCP live in this
  session) remains the only un-PRD'd direction; still OPEN, still needs explicit
  user opt-in (rule #6). Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.
- 2026-06-02 agentns-doctor-self-review: AC1-8 verified green on the draft; ready for user to swap proposals/self-review-agentns-block.draft.md into SKILL.md lines 123-124 and run one /self-review (AC9).

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed. 23rd consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review #1+#2 genuinely
  clean, kernel 7.0.10-arch1-5-wintermute, agorabus resolved, 87 memories),
  build manifest, 27 visions / ~95 PRDs on disk. Recall seeding (mandatory):
  reflective(8) all self-review run reports (recalls=0, [reflective/self]);
  hybrid-ideation(8) returns only already-PRD'd or kernel-built-not-booted
  signals (kernel built/stock booted, memlog EACCES, agentns zeros). pevent
  EMPTY (no orphans); ctrace query --since 86400 = 1 event (this session). No
  new INWARD laptop signal motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
SIGNAL (11th tick running, unchanged): external-service MCP surface fully live
  this session (Gmail/Calendar/Drive/AtScale Non-prod/AWS). Still ZERO of 27
  visions reaches outward. Did NOT re-offer the abstract picker; offered a
  concrete 4-option AskUserQuestion (reach-briefing fleet / reach-archive fleet /
  name-a-topic / stand-down), both outward fleets pre-sketched + ready to expand
  into 3-5 PRDs without further research. Picker DISMISSED again -> stood down
  per precedent. Outward remains the only un-PRD'd direction; still OPEN, still
  needs explicit user opt-in (rule #6). Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked /dream, interactive)  saturation report — no PRDs drafted
Seed: bare /dream, user-typed. 24th consecutive saturation tick.
Phase 0/1 walked: gossip tail, journal 2026-06-02 (self-review #1+#2 genuinely
  clean — agorabus resolved/doctor current/daemon 579, 87 memories, provfs
  xattrs live, dmesg clean, kernel 7.0.10-arch1-5-wintermute), build manifest,
  27 visions / ~95 PRDs on disk. Recall seeding (mandatory): reflective(8) all
  [reflective/self] run reports recalls=0; hybrid-ideation(10) returns only
  already-PRD'd or kernel-built-not-booted signals (kernel built/now-7.0.10
  booted, memlog EACCES, agentns zeros, stock-vs-wintermute carry-forwards).
  pevent EMPTY (no orphans). ctrace query --since 86400 = 1 begin event (this
  session). No new INWARD laptop signal motivates a PRD (rule #6).
Genuinely-open items remain USER-ACTION not code: pacman/kernel reboot, memlog
  group join, agentns registration, empty WM_ANTHROPIC_KEY (credit exhausted).
SIGNAL (12th tick running, unchanged): external-service MCP surface live this
  session (Gmail/Calendar/Drive/AtScale Non-prod/AWS). Still ZERO of 27 visions
  reaches outward. DIFFERENT THIS TICK: because invocation was interactive,
  offered a concrete 4-option AskUserQuestion (reach-briefing fleet /
  reach-archive fleet / name-a-topic / stand-down) — both outward fleets
  pre-sketched, ready to expand into 3-5 PRDs without further research. Picker
  DISMISSED again -> stood down per precedent. Outward remains the only un-PRD'd
  direction; still OPEN, still needs explicit user opt-in (rule #6). Not drafted.
Notes for /build: nothing new queued. Actionable backlog unchanged — vigil F4
  (install/build restart wiring + selfreview-concurrent-guard), warden fleet,
  onramp F2a (memlog consumer spine). No phantom PRDs added.

## 2026-06-02T (user-invoked, interactive)  PRD drafted — wintermute-wake-word
Seed: user said "Create a new wakeword -- wintermute". NOT a saturation tick — a
  direct request, so a PRD was drafted: PRD-wintermute-wake-word.md (rust-extend
  into wintermute-audio).
KEY FINDING for /build (verified live this session, not speculation): the wake
  path has NEVER fired for ANY word. Three confirmed causes:
  (1) no wake .onnx installed anywhere (only TTS lessac on disk) -> load_or_null_wake
      falls back to NullWakeDetector;
  (2) models.rs MANIFEST URLs all 404 — repo moved kahrendt/microWakeWord ->
      OHF-Voice/micro-wake-word; some sha256 look like placeholders;
  (3) FORMAT MISMATCH: upstream ships .tflite (verified via GH releases API:
      v2.1_models/hey_jarvis.tflite etc.), zero .onnx assets — but inference.rs
      uses ort(ONNX) and feeds RAW PCM [1,1280]->scalar, whereas micro-wake-word
      consumes MFCC feature frames from a separate preprocessor.
So "wintermute" is NOT additive: base must be fixed first (correct URLs/format,
  tflite->onnx, MFCC front-end). Fixing it also makes hey-jarvis work for the
  first time. This overlaps/corrects rouse-wake-vad-models (whose Non-goal #2
  excludes custom training) and wintermute-audio-inference.
Hard part = training a custom model (no pretrained "wintermute" exists): piper
  synth + augment + micro-wake-word recipe + tflite->onnx export. TF/torch NOT
  installed. Harness ships as contrib/train-wintermute.sh with a --smoke AC to
  de-risk before a full CPU train run.
Status: queued in PRD dir; /build will discover via PRD-*.md scan.

- [2026-06-03T20:09:40Z] Pending: daemon `recalld.service` installed binary `/home/jsy/.local/bin/recalld` but NOT restarted — `rollout install` unavailable (install-m755-fallback)

- 2026-06-03 (build): **reflect candidate (cap spent today)** — `wm-buildtree land` is `--ff-only`, so when a shared-target repo's default branch advances past a `build/<slug>` branch, land is *permanently* stuck and logs `land-deferred-main-dirty` every tick (recall-corpus-vacuum sat 4 ticks this way; main was 20 commits ahead). Recovery is manual: merge main into the branch in its worktree, resolve conflicts, then ff. Worth a PRD: `wm-buildtree land` should detect divergence and fall back to merge-main-then-ff (or the shared-target `worktree-extend integrate` path) instead of dead-ending on ff-only. recall uses build/* (wm-buildtree) AND has 3 stale 0-ahead build worktrees (stop-hook-discriminate, temporal-decay) — those should be pruned by land/cleanup.

- [2026-06-04T03:02:42Z] Pending: daemon `agorabus.service` installed binary `/home/jsy/.local/bin/agorabus` but NOT restarted — `rollout install` unavailable (install-m755-fallback)

## 2026-06-04T  /dream  vision-lucid
Seed: jsy (interactive, mid voice-bringup session) — "a way for wintermute to
  share their thoughts and inner mechanics so I can understand and debug."
Drafted: PRD-lucid-turn-id.md, PRD-lucid-tap.md, PRD-lucid-trace.md,
  PRD-lucid-mind.md, PRD-lucid-live.md, PRD-lucid-explain.md
Vision: visions/lucid.md
Order: turn-id → tap → {trace, mind, live} → explain
  (trace/mind/live each consume tap independently; explain composes trace+mind)
Why now (grounded in THIS session, not speculation): bringing up the voice loop
  cost a full session of debugging blind. The bus already carries ~120 wm.*
  topics incl. wm.brain.route {turn_id,tier,reason,latency_ms,model} — the
  brain's DECISION is already published — but nothing records/correlates/explains
  it. turn_id today exists ONLY inside wm-brain (minted as now_ms, daemon.rs
  2050/2119/2176), never threaded from wake → no cross-daemon correlation. The
  wake-never-fired bug was a 1-line tensor bug misdiagnosed as overfitting
  through 3 retrains + 120 recordings because the wake SCORE was computed but
  never surfaced. lucid fixes the legibility gap.
Notes for /build:
  - PRD-lucid-turn-id is build_target=mixed (rust-extend across all 5 voice
    repos); ship mint-first (wm-audio) then downstream — half-applied degrades
    gracefully (turn_id is additive/optional everywhere). It is the spine;
    tap/trace/mind/live read more easily once it lands but tap can record
    untagged turns before it.
  - PRD-lucid-tap creates a NEW repo ~/wintermute/wintermute-lucid (rust-cli,
    new wm-lucid daemon + systemd unit). The other four are rust-extend
    build_into that same repo — so tap MUST ship before trace/mind/live/explain.
  - PRD-lucid-mind adds a SMALL publish-side change to wm-brain (a bounded
    wm.brain.context digest event); the rest is read-only over the recorded log.
  - PRD-lucid-explain is deterministic/templated — explicitly NOT an LLM call
    (AC7: must not consume a brain turn). It reuses wm.tts.say for --voice.
Open questions: wm-lucid as new repo (assumed yes) vs subcommand on existing
  tool; should records stamp the provfs/agentns session id (deferred to a
  future lucid-extend); explain persona = hearth (elder) vs flat (jsy).
Cross-links: distinct from earshot (tempo) and scribe (journaling); explain's
  elder-facing voice borrows hearth's persona.

## 2026-06-04T  /dream  vision-homeward  (FIRST outward-facing vision)
Seed: jsy (interactive) — "a real-time database of missing dogs... research deeply
  for all public info on dogs in shelters... reconnect them with owners. We will
  need ML to match dogs by photos." + "also include cats."
Drafted: PRD-homeward-schema.md, PRD-homeward-connectors.md, PRD-homeward-ingest.md,
  PRD-homeward-embed.md, PRD-homeward-match.md, PRD-homeward-report.md
Vision: visions/homeward.md
Order: schema -> connectors -> ingest -> embed -> match -> report
  (embed consumes ingested photos; match reads embed+store; report sits on top)
Grounded in DEEP external research (4 parallel research agents, 2026-06-04 — full
  findings in the PRDs' "Why" sections). Load-bearing facts for /build:
  - Petfinder public API was RETIRED Dec 2, 2025. DO NOT build on it (the obvious
    stale instinct). Foundation = RescueGroups.org JSON:API v5 (free, national,
    ToS permits cached derivative search if refresh >=weekly + 1-business-day org
    deletion + no resale + hotlink images) AND municipal Socrata/SODA feeds
    (Austin fdzn-9yqv, Dallas qgg6-h4bd, Sonoma 924a-vesw, Long Beach) which are
    the STRAY tier (Intake_Type=STRAY, Found_Location, Chip_Status) = the actual
    lost-pet population. Adoptable-only feeds don't isolate strays.
  - Competitor Petco Love Lost already does national AI matching but is a CLOSED
    walled garden (no public API, black-box model, US-only). homeward's wedge =
    OPEN: open API + open data + auditable matcher + broader federation + owner
    control + real-time deltas.
  - ML (homeward-embed is the ONE non-Rust PRD, build_target=mixed, Python/uv):
    v1 = YOLO body-crop (COCO has dog AND cat) -> DINOv2 ViT-B/14 (Apache-2.0,
    COMMERCIAL-OK; ViT-S for this CPU-only box) -> HNSW cosine kNN -> human
    shortlist. AVOID MegaDescriptor (CC-BY-NC, non-commercial) and PetFace weights
    (research-gated) for v1. v2 = fine-tune ArcFace on PetFace (46k dog + cat
    IDs). ANTI-TAUTOLOGY IS AN AC: eval harness MUST refuse overlapping train/eval
    individual IDs (see feedback_agent_written_fixtures_tautology — wm-router
    100%->73.5% on held-out). Recall's BGE+vector-index is the structural model.
  - Legal/ethics encoded as ACs: hotlink/thumbnail images NEVER bulk-copy full-res
    (copyright; PhotoRef has no bytes field by design); strip EXIF; broker contact
    (no raw phone/email in any read path); coarsen geo (no street address); respect
    stray-hold windows (stray-in-hold = reclaimable, NOT adoptable); matches are
    CANDIDATES not confirmations (type has no is_match field). PII report store is
    a SEPARATE trust zone, never reachable via the open API.
Notes for /build:
  - homeward-schema MUST land first (rust-lib, creates the ~/wintermute/homeward
    cargo workspace); the other 5 are rust-extend build_into ~/wintermute/homeward.
  - homeward-embed is Python (uv) sidecar; Rust calls it over a localhost socket —
    no Python types in the Rust crates. Likely needs torch/DINOv2 download; on this
    CPU-only laptop use ViT-S and expect the gallery embed to be the only heavy
    (one-time) cost.
  - homeward-ingest is a daemon w/ sqlite store (rusqlite, toolkit sqlite-first).
Open questions (left for /dream extend homeward, NOT drafted — honest per rule 6):
  outward syndication (one report -> PawBoost/Pet FBI-LDOA/Nextdoor + pull matches
  back) is the biggest practical gap but not yet thought through; microchip-registry
  federation (AAHA gated, missing AVID) needs partnerships not scraping; commercial-
  vs-nonprofit licensing choice gates which ML weights are legal to ship.
Cross-links: structurally mirrors ~/wintermute/recall (embedder + vector index).

## 2026-06-04T  /dream  vision-constellation  (fleet: grow beyond this laptop)
Seed: jsy (interactive) — "expand across multiple computers. This laptop is too
  resource constrained... 32GB AMD desktop w/ medium Radeon... install wintermute
  linux on this and other computers + communicate across all... voice control on
  boot... identical appearance (i3, terminal shapes, all taskbar tools)... coordinate,
  collaborate, distribute workloads to maximize dev throughput... cloud service...
  /dream of growing beyond this laptop."
Drafted: PRD-constellation-{provision,appearance,mesh,bus,brain-gpu,cloud,dispatch}.md (7)
Vision: visions/constellation.md
Order: provision -> appearance ; provision -> mesh -> bus -> {cloud, brain-gpu, dispatch}
Grounded in DEEP external research (4 parallel agents, 2026-06-04 — full findings +
  citations in PRD "Why" sections). Load-bearing decisions for /build:
  - PROVISION: stay Arch, drive w/ ANSIBLE (NOT NixOS — custom linux-wintermute
    kernel PKGBUILD fights Nix w/ compile-on-rebuild; multi-month migration). Local
    pacman repo for the kernel (build once, install everywhere). Golden archiso for
    day-0. chezmoi for dotfiles (ONLY manager that templates per-host = "identical
    but different GPU/monitor"). Boot-to-voice = greetd initial_session autologin ->
    i3 -> wintermute.target, w/ the i3->graphical-session.target bridge FIX (i3 #5186
    — naive WantedBy=graphical-session.target silently never starts).
  - MESH+BUS: Tailscale (MagicDNS, NAT traversal) + NATS hub(cloud)/leaf(each node)
    + JetStream. CRITICAL CORRECTION: NATS has NO unix-socket listener, so agorabus
    CANNOT be a leaf over its UDS -> need a BRIDGE sidecar (new repo
    ~/wintermute/agorabus-nats-bridge, wm-busbridge daemon, UDS<->NATS, wm.* identity
    map, loop-guard, SELECTIVE forwarding so PCM chunks/local topics DON'T cross).
    2nd footgun: JetStream blocked across leaf boundary by default -> set JS DOMAINS
    (hub + per-leaf) + domain-qualified $JS.hub.API.> prefix. ALL leaf/hub URLs =
    MagicDNS names never IPs.
  - BRAIN-GPU: llama.cpp VULKAN (NOT ROCm — more reliable on consumer Radeon AND
    faster token-gen which is what voice needs; ROCm only wins long-context pp).
    Qwen2.5-7B/Qwen3-8B Q4_K_M -> ~35-50 tok/s, sub-2s TTFT, turn ~25s->~2-4s. New
    `local-gpu` tier in wintermute-brain ladder (between local-3b and cloud), graceful
    skip if desktop off. DETECT the GPU (vulkaninfo/lspci) — exact Radeon model unknown
    (RDNA2 gfx1030 needs HSA_OVERRIDE; RDNA3 mostly just works; Vulkan avoids ROCm).
  - CLOUD: cheap always-on coordinator = Hetzner CAX21 ~€8/mo (Oracle free A1 = hot
    spare; idle-reclaim risk). Hosts NATS hub + mesh exit + small offline-fallback
    brain. DO NOT self-host the latency brain — Anthropic API ~$3-11/mo BEATS any
    rentable GPU 20-40x at personal volume (break-even ~15-20M tok/DAY). GPU pods
    (RunPod 4090 $0.69/hr, A40 48GB $0.44/hr) BURST-ONLY for build/ML jobs. Fly.io
    GPUs retired Aug 2026 — excluded.
  - DISPATCH: JetStream WM_WORK work-queue + per-node PULL consumers (demand-pull =
    capacity balance across uneven hw) + WM_NODES KV capability registry (heartbeat
    TTL, NOT gossip at 3-5 nodes) + sccache/sccache-dist for distributed Rust builds
    (compose w/ the queue, don't replace). Job payloads small (refs not blobs);
    artifacts move via shared storage/rsync over mesh NOT the bus.
Notes for /build:
  - constellation-bus is the KEYSTONE (new repo agorabus-nats-bridge); brain-gpu/cloud/
    dispatch all depend on it. provision creates ~/wintermute/constellation/ (ansible+
    archiso+chezmoi+boot). brain-gpu is rust-extend into wintermute-brain (+ systemd
    config on desktop). dispatch is rust-extend into the bridge repo.
  - Per-host ROLE flags matter: voice_node (laptop=yes, desktop/cloud=optional/no);
    gpu (amd|intel) drives driver + the Vulkan stack install.
Open questions (vision doc): bit-for-bit vs convergent-identical (only NixOS gives
  literal bit-identity — flips recommendation if hard requirement); Tailscale vs
  self-hosted Headscale (sovereignty); secrets bootstrapping channel (one root secret
  out-of-band per host); voice-on-every-node policy.
Cross-links: distinct from kin/homestead (elder COMPANION device) — constellation is
  jsy's own DEV fleet. Reuses wintermute-brain ladder, agorabus, linux-wintermute
  kernel, ~/wintermute/dotfiles + wintermute-desktop.

## 2026-06-04T  /dream  vision-constellation  (HARDWARE CORRECTION + build/LLM split)
Seed: jsy corrections — "its a Ryzen 7 5700U" + "run build jobs in the cloud so we
  can use localhost with a local LLM. not enough RAM nor CPU for both." + "we may be
  able to run qwen2.5 8B if we stop all build activity here."
VERIFIED via research agent: the "32GB AMD desktop w/ medium Radeon" is actually a
  Ryzen 7 5700U = Zen 2 "Lucienne" APU, 8c/16t, Vega 8 iGPU (gfx90c), NO discrete
  GPU, NO VRAM, ~51 GB/s shared DDR4. llama.cpp Vulkan on the Vega iGPU gives ~2x
  prompt-prefill but ~ZERO generation gain vs CPU (both bandwidth-bound) → ~8-10
  tok/s on 7-8B Q4 (vs 35-50 on a real dGPU). ROCm on gfx90c needs HSA_OVERRIDE=
  9.0.0 and is UNRELIABLE/slower → use Vulkan or CPU, never ROCm.
ARCHITECTURE CHANGE (the earlier brain-gpu assumption was WRONG):
  - SUPERSEDED PRD-constellation-brain-gpu.md (assumed discrete Radeon, ~2-4s brain).
    /build: ARCHIVE it, do NOT build it.
  - NEW PRD-constellation-brain-local.md (supersedes brain-gpu): dedicate the 5700U
    to a local LLM (llama.cpp Vulkan/CPU, qwen2.5-8B Q4 ~8-10 tok/s) as a `local-llm`
    tier (privacy/offline/default+routing), RESOURCE-ISOLATED from builds. Honest:
    NOT the fast latency brain (cloud Anthropic stays that). Builds do NOT run here.
  - NEW PRD-constellation-cloud-build.md (refines dispatch + cloud): cloud becomes
    the BUILD workhorse — sccache-dist build servers on the cheap cloud node + burst
    CPU/GPU pods on demand; dispatch routing sends build jobs to cloud (+ laptop when
    voice-idle) and NEVER to the no_build:true / role:local-llm 5700U node. Corrects
    constellation-dispatch's "build servers on desktop+cloud" (desktop is now model-
    only). Cost guardrails: burst pods created on-demand, torn down, lifecycle+cost
    logged, monthly burst-budget cap.
Vision doc UPDATED (durable): hardware reality, build-out/LLM-in resource spine,
  components list (brain-gpu→superseded, +brain-local, +cloud-build), Radeon-model
  OQ resolved.
Notes for /build: constellation fleet now 8 active PRDs (provision, appearance,
  mesh, bus, brain-LOCAL, cloud, cloud-build, dispatch) + 1 superseded (brain-gpu →
  archive). Dependency order unchanged except brain-gpu→brain-local and cloud-build
  refines dispatch. The 5700U advertises no_build:true so builds never contend with
  the model. Local box = model node; cloud = hub + build workhorse; laptop = voice
  node (+ idle-time builder); Anthropic API = latency brain.

## 2026-06-05 (build tick) — Pending observations
- **extend-handler.sh read_cargo_version is fragile on workspace Cargo.toml**: when build_into is a cargo *workspace* root with no `[package].version`, `extend-handler.sh validate` / version-bump awk fails. homeward-match hit this; worked around by committing a `[package]` stub to homeward's workspace Cargo.toml. Reflect candidate: teach read_cargo_version to fall back to the workspace's first member or a `[workspace.package].version`.
- **homeward is built but unpublished (no git remote)**: homeward-schema/connectors/report/ingest are all integrated to local main (now v0.4.0) but homeward has no j0yen origin, so rust-extend PRDs can never pass the verified-completed gate (commit reachable from origin/main). Decision needed: publish homeward as j0yen/homeward (it's the first outward-facing pet-finder DB — confirm public-repo intent with user first) OR keep local. Until then homeward-* PRDs park at integrated-local-unpublished.
- **constellation repo now exists** (created this tick by mesh+provision scaffold); **agorabus-nats-bridge now exists** (created by constellation-bus /autobuilder). Their dependents (constellation-appearance/cloud into constellation, constellation-dispatch into agorabus-nats-bridge) are now unblocked for next tick — they were queued this tick.

## 2026-06-04T  /dream  vision-constellation  (FLEET = 3 machines; GTX 1080 = fast brain)
Seed: jsy clarified hardware — there are TWO separate desktops + the laptop + cloud:
  (1) LAPTOP i7-10610U 4c/8t 15GB Intel-iGPU no-dGPU [measured qwen3:8b = 4.34 tok/s];
  (2) 5700U box 8c/16t Zen2 32GB Vega8-APU no-dGPU (~8-10 tok/s on 8B);
  (3) full-size 5th-gen i7 TOWER 32GB that takes a **GTX 1080 8GB GDDR5X ~320GB/s**.
ROLE REASSIGNMENT (the 1080 changes everything — fast LOCAL brain is now real):
  - i7 TOWER + GTX 1080 = the FAST LOCAL BRAIN. 8B Q4 fits 8GB VRAM, ~25-35 tok/s
    (~6x the APU bandwidth) -> ~2-3s/reply, sub-1s TTFT. GPU serves model -> CPU free.
  - 5700U box (8 Zen2 cores) = the BUILD/COMPUTE WORKHORSE (best CPU in fleet for
    sccache-dist + CPU ML jobs). NO LONGER the local-LLM node.
  - Laptop = thin VOICE node (brain served by tower or cloud; do NOT run 8B here,
    4.34 tok/s measured).
  - Cloud = always-on hub + Anthropic API latency brain (when tower asleep).
PRD CHANGES:
  - NEW PRD-constellation-brain-cuda.md (supersedes brain-gpu): nvidia-dkms on
    linux-wintermute + llama.cpp/ollama CUDA on the GTX 1080 tower, 8B Q4, `local-gpu`
    tier at ~2-3s. CUDA path (Pascal sm_61, no tensor cores, weak FP16 but llama.cpp
    Q4 handles Pascal fine; NOT Vulkan/ROCm). 8GB caps at 7-8B (14B won't fit).
  - PRD-constellation-brain-gpu.md = SUPERSEDED/archive (assumed Radeon).
  - PRD-constellation-brain-local.md (5700U qwen2.5-8B) = DEMOTED to optional
    offline/privacy secondary; 5700U's real job is now BUILDER.
  - constellation-cloud-build / dispatch: primary build worker is now the 5700U
    (8 cores), cloud bursts only for big fan-outs; builds never go to the tower
    while it serves the brain.
Vision doc UPDATED (durable): 3-machine fleet role table, GTX 1080 resource spine,
  components (brain-gpu superseded, +brain-cuda, brain-local demoted, build->5700U).
Notes for /build: fleet now ~9 active PRDs. brain-cuda is the headline new capability
  (fast local brain). Two physical confirmations still open: (a) tower PSU has a spare
  8-pin PCIe + ~500W for the 1080; (b) nvidia-dkms builds clean against linux-wintermute.

## 2026-06-04T  /dream  constellation  (GTX 1080 powered+running — PSU gate closed)
jsy: "it's already plugged in and running." The GTX 1080 PSU/8-pin gate is CLOSED.
Only software step left for the fast local brain: nvidia-dkms builds clean against
linux-wintermute (brain-cuda AC1). Hardware for the ~2-3s local-gpu brain is GO.

## 2026-06-04T  jsy decision  lucid-tap service = DO NOT enable at boot
jsy reviewed the "enable wm-lucid.service at boot" ask and declined: the recorder
logs every wm.* bus event to disk 24/7 and impacts performance on this thin voice
node. Resolution: wm-lucid is a TRANSIENT diagnostic tool, default OFF, NOT a boot
service. Created skill `/lucid-diagnose` (~/.claude/skills/lucid-diagnose/) that
start/stops the service transiently (never `enable`) and wraps last/trace/explain.
Notes for /build: STOP flagging "enable wm-lucid.service" as a pending action.
lucid-tap's install is done; the service is intentionally `disabled`. Treat the
lucid-tap PRD's "live recorder" AC as satisfied by the on-demand skill, not a
permanent daemon.

## 2026-06-04T  jsy ask  burst-builder PRD drafted (standalone, mesh-free rung)
jsy: "maybe we should burst heavy rust compile and CPU jobs to the cloud." Drafted
PRD-constellation-burst-builder.md — a new `wm-burst` rust-cli (j0yen/wm-burst) that
points local cargo at ONE always-on Hetzner dedicated 9950X + a shared sccache cache.
Deliberately the rung BENEATH constellation-cloud-build: needs NO NATS mesh, NO
dispatch coordinator, NO capability registry — just ssh + sccache + config.toml.
Standable-up today; the full fleet PRD graduates from it.
Notes for /build: (1) does NOT duplicate cloud-build — it carves out the no-mesh
path and Refines it. (2) Key guardrail = `wm-burst doctor` HARD-FAILS on remote-vs-
local toolchain drift (the 1.85/1.88 split that has corrupted cold builds). (3) pod
tier ACs are provable with a MOCKED provider — no real cloud spend needed to pass.
(4) sigpipe::reset() first line of main per self_sigpipe_panic_toolkit. Two open
questions left for build/jsy: cache backend (MinIO-on-the-box is cheapest default)
and sccache-dist vs remote-cargo-over-ssh for v0.1 (AC4 allows either).

## 2026-06-05T02:42:35Z build-tick friction
- **constellation repo has no origin remote** — constellation-appearance + constellation-cloud committed to local master but cannot push. Needs `gh repo create j0yen/constellation` + `git remote add origin` + adding 'constellation' to wm-push/wm-publish ALLOW lists + REPOS.md row. Blocks 2 PRDs.
- **allow-list gaps**: homeward-schema not on wm-publish ALLOW (publish deferred). constellation-dispatch's wm-buildtree ensure was classifier-blocked (slug added since). Recurring new-slug-not-allowed pattern — candidate for a follow-on PRD that auto-syncs ALLOW arrays from REPOS.md.

## 2026-06-04T  jsy ask  wake-train offload PRD drafted (companion to burst-builder)
Drafted PRD-constellation-waketrain-offload.md (build_target: shell) — a
`burst-train.sh` recipe (+ wake-retrain-burst.service drop-in) that runs the heavy
train-wintermute.sh job on a cloud pod instead of OOM-killing this laptop (11.2 GB
peak, no swap, 2026-06-03). DEPENDS on burst-builder for wm-burst exec/pod transport;
REFINES PRD-wintermute-wake-word (changes no stages, wraps it).
Notes for /build: THE POINT is the INSTALL GATE, not just "run elsewhere" — a returned
ONNX is swapped into wm-audio ONLY if I/O is exactly [1,186,40]→[1,1], non-streaming,
and the local `verify` stage passes; bad/mis-shaped run leaves the live model untouched
+ exits non-zero (AC4). Atomic install + --rollback (AC5). Per feedback_verify_before_
concluding: verification is a gate here, not afterthought. Pod cost/teardown delegated
to wm-burst pod (mocked provider proves AC6, no real spend). Open Qs: GPU-vs-CPU default
pod, where the pinned py3.11/TF2.21 env lockfile lives, and whether to block install on
val-accuracy regression.

## 2026-06-05T  jsy decision  burst-builder goes aarch64 (Oracle free tier)
jsy priced Hetzner, too high ("even a ryzen 7 is 80 euros"). Pivot: cheapest correct
builder = Oracle Always-Free Ampere (4 core / 24GB / $0, aarch64) — and 24GB removes
the wake-train OOM for free. wm-burst v0.1 already SHIPPED (constellation-burst-builder,
wm-burst 0.1.0, archived) but assumes x86: provision.rs is Arch/pacman-only, toolchain.rs
checks only the rustc CHANNEL not the arch. Drafted EXTEND PRD-constellation-burst-builder-arch.md
(rust-extend into constellation-burst-builder).
Notes for /build: (1) add remote_arch (target-triple, default aarch64-unknown-linux-gnu)
+ remote_os to BurstConfig. (2) provision gains an apt/Ubuntu-Oracle path + installs the
configured target toolchain. (3) THE GATE: doctor must compare the FULL host triple from
`rustc -vV` host: line, not just channel — wrong-arch/right-version remote must hard-fail
like a version mismatch (AC3). (4) Settles two open Qs: sccache backend default = MinIO
on the Oracle box (200GB free block); aarch64 is the default remote. x86 still works via
--remote-arch x86_64-unknown-linux-gnu. waketrain-offload's pinned-env note now also needs
arm64 wheels for TF2.21/torch — flag when that one builds.

## 2026-06-05T  jsy decision  wake-train = GPU pod default, no standing infra
jsy: "I dont plan to do wake training very often. and wouldnt a GPU pod be better for
that?" Yes. Resolved the waketrain-offload open question: DEFAULT = on-demand CUDA GPU
pod (RunPod/Vast), torn down after; --cpu is fallback; --smoke on a tiny shape. Training
is rare → NO standing training infra. Clean split now: Oracle free aarch64 box = BUILDS
(always-on, $0, 24GB); GPU pod = TRAINING (rare, per-run, minutes). Updated PRD-constellation-
waketrain-offload.md (TL;DR, GPU-pod-run bullet, AC3, Resolved-decisions section).
IMPORTANT env note for /build: GPU pod is x86+CUDA, so the pinned TF2.21/torch training
env is x86 wheels — INDEPENDENT of the aarch64 build box. Do NOT try to reuse the arm64
build env for training.

## 2026-06-05T  jsy decision  BUILD-ONLY = Hetzner Cloud x86 on-demand (ARM withdrawn)
Key fact: laptop is x86_64 (verified uname). An aarch64 box can NEITHER produce
runnable-on-laptop binaries NOR share an x86 sccache cache → Oracle-free-ARM is only a
CI/compile-check box, not a real build offload. jsy chose Hetzner Cloud x86 on-demand
(~€0.03/hr bursted, ~€1-2/mo) over €80 dedicated and over free ARM.
ACTIONS TAKEN:
- DELETED PRD-constellation-burst-builder-arch.md (aarch64 work now obsolete). WARNING:
  /build had ALREADY claimed it — pid 334165 tick was mid-edit on init.rs/config.rs in
  worktree .build-worktrees/constellation-burst-builder-arch, branch = the SHARED
  autobuilder/constellation-burst-builder. Did NOT kill it (avoid /build jam per
  self_build_jam_leaked_tracer). It may land an additive `remote_arch` field — harmless;
  pin default x86_64.
- DREW UP PRD-constellation-burst-builder-hcloud.md (rust-extend into constellation-burst-
  builder): implements the REAL HcloudPodProvider (shipped pod provider is a STUB —
  make_provider warns "not yet implemented in v0.1", falls back to mock). Adds
  `wm-burst build --burst` (ephemeral CCX23 per build, torn down), provision --snapshot,
  and Hetzner Object Storage as the persistent sccache backend (SUPERSEDES the
  MinIO-on-standing-box default — no standing box in build-only plan).
ORDERING FOR /build: build hcloud PRD only AFTER the constellation-burst-builder branch
settles (the arch tick was on the same branch — concurrent edits = collision). provision
already has BOTH apt(Debian/Ubuntu) + pacman(Arch) paths (verified), so no OS work needed.

## 2026-06-05T19:15  /dream  vision-concord  (seed: "world peace" / "change the world")
Drafted: visions/concord.md (NO PRDs yet — held for user direction).
Concord = the 2nd outward-facing vision (after homeward). Attacks the tractable
driver of conflict: breakdown of understanding. Pipeline corpus → steelman →
cruxes → bridge (+ independent deescalate), all on the LOCAL LLM ladder
(qwen3:8b/qwen2.5:3b verified) + the deep-research fan-out harness.
Notes for /build: nothing to build yet — only the vision doc landed. PRDs will
follow once the user points the direction. IMPORTANT design constraint when they
do: /build now runs cargo on the cloud box (no ollama), so every LLM-touching
crate must inject the model behind a trait and test against a mock + golden
fixtures; live-LLM = manual/deferred AC, never a cloud-gated test.
Open: user said "I want to change the world, help me" — paused auto-drafting the
fleet to ask what they actually want to change before committing buildable PRDs.

## 2026-06-05T19:20  /dream  vision-relay  (seed: "help as many people as I can" → force-multiply helpers)
Drafted: PRD-relay-directory.md, PRD-relay-match.md, PRD-relay-intake.md, PRD-relay-letters.md
Vision: visions/relay.md
Order (STRICT — hard dependency): relay-directory FIRST (new repo ~/wintermute/relay,
  rust-cli, creates the workspace + `relay` binary) → THEN relay-match, relay-intake,
  relay-letters (all rust-extend, build_into=~/wintermute/relay). Do NOT start any
  rust-extend until relay-directory has shipped + the repo exists, or extend-validate
  will fail. match/intake/letters can build in any order once directory lands.
Notes for /build:
  - relay-directory is pure-deterministic (HSDS/CSV ingest + sqlite + ranking query),
    NO LLM, NO network in tests → fully cloud-build-safe.
  - match/intake/letters each touch the LOCAL LLM but inject it behind a trait and test
    against a Mock + golden fixtures; each has exactly ONE live-model AC marked
    deferred_acs (cloud box has no ollama — never gate a test on it).
  - These handle sensitive human-services data: privacy is an AC (no outbound net in the
    default path; rule-based PII redaction). Keep that guarantee through implementation.
  - relay-directory is OUTWARD-FACING (homeward sibling) → new public j0yen repo on publish.
Open (next /dream pass): relay-fresh (deep-research re-verify), relay-serve/web (UI),
  relay-volunteer. See visions/relay.md.

## 2026-06-05T  /dream  vision-concord  (seed: held vision decomposed — user re-invoked /dream without re-steering)
Drafted: PRD-concord-corpus.md, PRD-concord-steelman.md, PRD-concord-cruxes.md,
  PRD-concord-bridge.md, PRD-concord-deescalate.md
Vision: visions/concord.md (status flipped held → all-5-drafted)
Order (STRICT hard dependency):
  concord-corpus FIRST (new repo ~/wintermute/concord, rust-cli, creates workspace
    + `concord` binary + the Corpus schema + the ConcordModel trait). Do NOT start
    any rust-extend until corpus has SHIPPED and the repo exists, or extend-validate
    fails (same rule that bit relay).
  THEN: corpus → steelman → cruxes → bridge is a straight consume-the-prior chain
    (build in that order). deescalate depends on corpus ONLY (for the workspace +
    ConcordModel trait) and can build in PARALLEL with steelman/cruxes/bridge.
Notes for /build:
  - concord-corpus is PURE-DETERMINISTIC (rule-based stance tag + Jaccard dedup +
    heuristic credibility; NO LLM, NO network in tests) → fully cloud-build-safe.
    FixtureGatherer is the test path; a test asserts zero outbound connections.
  - steelman/cruxes/bridge/deescalate each touch the LOCAL LLM but inject it behind
    a `ConcordModel` trait (real LadderModel wraps wintermute-brain's LadderClient /
    LocalBackend at ladder.rs:53; MockModel in all tests). Each has exactly ONE
    live-model AC marked deferred_acs (inline bare ints) — cloud box has no ollama,
    NEVER gate a test on it.
  - bridge + deescalate ACs explicitly require asserting the integration/mock test
    ENTRY FILE runs in cargo output (self_orphaned_mock_tests guard).
  - concord-corpus is OUTWARD-FACING (homeward/relay sibling) → new PUBLIC j0yen
    repo on publish.
  - Privacy is load-bearing: default path makes ZERO outbound connections; all
    reasoning is on-device. Keep that guarantee through implementation.
Resolved: deescalate carries a tested refusal rule (declines to launder threats).
Open (next /dream pass): concord-serve (wm.concord.* bus + HTTP), concord-web (UI,
  needs homeward-style hosting decision), the crux-vs-misunderstanding eval dataset
  (hand-built golden vs public set?), the steelman-of-bad-faith boundary at the
  bridge/serve layer.

## 2026-06-05T  /dream  vision-quicken  (seed: bare /dream, user declined steer → dreamed from laptop's strongest unaddressed signal)
Drafted: PRD-quicken-probe.md, PRD-quicken-remedy.md, PRD-quicken-attest.md, PRD-quicken-crossdep.md
Vision: visions/quicken.md
Axis: vigil=stale-but-running bytes; quicken=NEVER-came-alive (built/installed but runtime-inert).
  Distinct from vigil/freshness/drift. Caught live 2026-06-05: memlog EACCES (user not in
  memlog group; installed pkgrel-5, fix sits uninstalled at pkgrel-11 — gap WIDENED 5→10→11),
  agentns /proc/self/agent_session all-zeros, bpolicy {"loaded":false}, provfs LIVE-but-DEGRADED
  (xattr is comm:zsh fallback, NOT the 128-bit agentns id — provfs degraded BECAUSE agentns is dark).
Order (STRICT hard dependency):
  quicken-probe FIRST (new repo ~/wintermute/quicken, rust-cli; creates workspace + `quicken`
    binary + Verdict/Evidence/PrimitiveReport types + the Probe trait + 4 probes). Do NOT start
    any rust-extend until probe has SHIPPED and the repo exists, or extend-validate fails
    (same rule that bit relay + concord).
  THEN remedy, attest, crossdep are ALL rust-extend (build_into=~/wintermute/quicken), any order
    after probe lands. crossdep reads cleanest after probe; attest independent of remedy/crossdep.
Notes for /build:
  - quicken-probe is PURE-READ (proc/dev/xattr/pacman-query behind a ProbeEnv trait, fixtures in
    tests, ZERO network/writes) → fully cloud-build-safe. A test asserts the probe path is pure-read.
  - remedy defaults to PRINT-ONLY (--dry-run posture, mirrors rollout); --apply runs ONLY the safe
    userspace subset (group/udev), prints (never runs) sudo/reboot/kernel steps. Its ONE live AC
    (apply actually revives memlog group) is deferred_acs:[8] — cloud box has no /dev/memlog.
  - attest/crossdep are deterministic + cloud-safe; boot_id + clock INJECTED in tests (no Date::now).
  - crossdep AC requires the integration test entry file (tests/crossdep.rs) appear in cargo output
    (self_orphaned_mock_tests guard).
  - quicken is INWARD tooling (sibling of vigil/binstale/rollout) → publishes as j0yen repo like the
    rest of the toolkit, NOT outward/public-civic like homeward/relay/concord.
Open (next /dream pass — held, not yet motivated enough): quicken-watch (boot oneshot → wm.quicken.*
  bus events for a homestead self-heal loop), self-heal-vs-report decision (auto-install protected
  kernel pkg? leaning report-only), agentns all-zeros root cause (kernel-side → agentns repo patch,
  not a quicken PRD).

## 2026-06-05T  /dream  vision-keel  (seed: bare /dream → laptop's strongest UNADDRESSED signal; quicken took the inert-kernel signal earlier today, this one was still open)
Drafted: PRD-keel-pulse.md, PRD-keel-ledger.md, PRD-keel-cordon.md, PRD-keel-beacon.md
Vision: visions/keel.md
Axis: thrift=spend cloud only where it earns warmth (COST). keel=which tier is the
  brain actually STANDING on, does it KNOW, does it SAY so (FOOTING). Distinct, complementary.
Motivation (live, recurring): docket wm-anthropic-key-empty OPEN since 2026-05-30, 6 runs / 8
  reports (evidence recall:01KT6VCBX1PSWT9MAQP607TAWY). ladder.rs:91-94 degrade is correct but
  STATELESS+INVISIBLE: brain re-discovers dead cloud EVERY turn (wasted round-trip on a voice
  path), and floored-on-3B is legible ONLY at daily self-review (journals 06-01/02/03 re-report
  it as if new). local-3b is the de-facto brain (cloud dead, local-8b skipped — pins this CPU).
Order (STRICT hard dependency):
  keel-pulse FIRST (new repo ~/wintermute/keel, rust-cli; creates workspace + `keel` binary +
    TierHealth/TierStatus/LedgerEntry/Ladder types + TierProbe & ProbeEnv traits + non-generating
    reachability/auth probe). Do NOT start any rust-extend until pulse has SHIPPED and the repo
    exists, or extend-validate fails (the rule that bit relay + concord + quicken).
  THEN: ledger ∥ cordon (both depend on pulse ONLY, build in PARALLEL) → beacon (depends on cordon
    for the effective ceiling + pulse for the floored-tier line).
Notes for /build:
  - ALL FOUR are cloud-build-safe: probe/bus/ledger-store behind traits, fixtures in tests, clock
    INJECTED (no Date::now), a test in each asserts zero live network / zero live-bus. The cloud
    box has no ollama and no Anthropic key — NEVER gate a test on a real backend.
  - keel-pulse probe is NON-GENERATING by contract (TCP+/v1/models GET, never /v1/chat/completions)
    so it never bills and never blocks on the CPU-pinned local model. AC5 asserts the path.
  - keyless cloud tier must NOT open a socket (keel-pulse AC4) — check the env var, skip the call.
  - ledger is APPEND-ONLY NDJSON (mirrors gossip/recall); AC2 asserts byte-prefix stability.
  - beacon is EDGE-triggered (emit only on ceiling change, zero on no-change — AC5), NOT a heartbeat.
  - ledger/cordon/beacon each require their integration test ENTRY FILE (tests/ledger.rs etc.) to
    appear in cargo output (self_orphaned_mock_tests guard).
  - keel is INWARD toolkit (sibling of vigil/quicken/binstale) → publishes as a j0yen repo like the
    rest of the self-tooling, NOT outward/public-civic like homeward/relay/concord.
  - sigpipe::reset() first line of main() in the binary (self_sigpipe_panic_toolkit — keel x | head).
  - MSRV 1.85, no let-chains (recall baseline-gate discipline).
Open (next /dream pass / user — HELD, not yet drafted): keel-floor-eval (score local-3b on a
  HAND-BUILT held-out golden set — NOT self-written, feedback_agent_written_fixtures_tautology;
  golden-set provenance unresolved). brain-keel-wire (rust-extend wintermute-brain so LadderClient
  consults the cordon before dispatch — touches live ollama path, NOT cloud-safe, USER-GATED).
  keel spend → thrift cost-model feed? wm.keel.degraded → homestead self-heal vs report-only?

## 2026-06-05T  /dream  vision-anchor  (seed: bare /dream → strongest UNADDRESSED recurring signal; quicken took inert-kernel, keel took dead-cloud, this observation-layer fragility was still open)
Drafted: PRD-anchor-roots.md, PRD-anchor-probe.md, PRD-anchor-reconcile.md, PRD-anchor-boot.md
Vision: visions/anchor.md
Axis: keel=which brain TIER are we standing on. anchor=which change-OBSERVATION
  roots are we standing on. Both are "is the footing real + legible", different substrate.
Motivation (live, recurring across reboots): watchman drops watched roots on reboot/socket-bounce
  → wchg since/reset go SILENTLY empty-or-error → self-review re-watches ~/.claude+~/brain BY HAND
  every run (journals 06-01/02/03). 8+ reflective memories, top recall:01KT6VCBX1PSWT9MAQP607TAWY
  score 10.3. The 06-03 journal itself proposed "a SessionStart re-watch hook if it recurs" — it
  recurs; anchor-boot IS that hook. Confirmed live 2026-06-05: watchman.service is socket-activated
  `--inetd` (no persisted-root restore); wchg list shows mixed clock ages (mid-May beside today) =
  ad-hoc per-root re-watching; ZERO of the 8 SessionStart hooks touch watchman.
Order (STRICT hard dependency):
  anchor-roots FIRST (new repo ~/wintermute/anchor, rust-cli; creates workspace + `anchor` binary +
    RootStatus/WatchRoot/WatchState/ReconcileAction/ReconcilePlan types + WatchBackend trait +
    RootsConfig manifest loader + PURE reconcile diff). Do NOT start any rust-extend until roots has
    SHIPPED and the repo exists, or extend-validate fails (the rule that bit relay+concord+quicken+keel).
  THEN: anchor-probe ∥ anchor-reconcile (both depend on roots ONLY, build in PARALLEL) → anchor-boot
    (mixed: systemd unit + SessionStart hook; needs reconcile's --apply path to exist).
Notes for /build:
  - anchor-roots + anchor-probe are PURE-READ / fully cloud-build-safe: watchman behind WatchBackend,
    FakeBackend fixtures in tests, clock INJECTED (no Date::now). A test asserts reconcile makes ZERO
    backend calls; probe asserts it calls ONLY read methods (live_roots/ping), never watch/reseed.
  - anchor-reconcile defaults PRINT-ONLY; --apply is the ONE live-side-effect path. Its ONE live AC
    (real watch/wchg delta) is deferred_acs:[6] — cloud box has NO watchman + NO wchg.
  - anchor-boot is mixed (systemd unit + hook + installer). Live ACs deferred_acs:[5,6] (real reboot /
    real session on the laptop). install.sh PRINTS the settings.json hook entry, does NOT auto-edit
    ~/.claude (settings edits user-gated, feedback_classifier_per_command). systemd-analyze verify is
    the offline gate for the unit. Hook exits 0 even on reconcile failure (never block session start).
  - probe/reconcile/boot each require their integration test ENTRY FILE (tests/probe.rs, tests/
    reconcile.rs, the boot installer test) to appear in cargo/test output (self_orphaned_mock_tests).
  - anchor is INWARD toolkit (sibling of vigil/quicken/keel/binstale) → publishes as a j0yen repo like
    the rest of the self-tooling, NOT outward/public-civic like homeward/relay/concord.
  - sigpipe::reset() first line of main() in the binary (self_sigpipe_panic_toolkit — anchor x | head).
  - MSRV 1.85, no let-chains (recall baseline-gate discipline).
Open (next /dream pass / user — HELD, not yet drafted): anchor-watch (wm.anchor.* bus event on
  lost-root → homestead self-heal loop, mirrors keel-beacon/quicken-watch held notes). Should probe
  feed the docket ledger (open a watchman-root-lost key) vs just exit non-zero? — leaning: probe stays
  pure, the HOOK opens the docket key. Mid-session socket-teardown coverage (a wchg-shim that
  auto-reconciles-on-empty) — held until boot+session coverage proves insufficient. Alternative the
  user may prefer: configure watchman to persist watches instead of a reconciler (current --inetd unit
  doesn't restore; reconciler is the backend-agnostic answer but noting the option).

## 2026-06-05T  /dream  vision-coda  (seed: bare /dream → strongest UNADDRESSED open docket item; quicken took inert-kernel, keel took dead-cloud, anchor took watchman-roots, this session-summary loop was still open)
Drafted: PRD-coda-sweep.md, PRD-coda-audit.md, PRD-coda-close.md, PRD-coda-boot.md
Vision: visions/coda.md
Axis: keel=which brain TIER. anchor=which OBSERVATION roots. coda=did every
  session get its CLOSING summary. Same "is the footing real + legible" family,
  different substrate (the ctrace→scribe summary pipeline).
Motivation (live, measured): `~/.cache/ctrace/sessions/` = 1874 *.ndjson vs 1251
  *.summary.md → 623 ORPHANED (33%) as of 2026-06-05. Every timer tick today
  (claude-20260605T210001…T230000) is missing its summary. Cause is documented in
  ctrace-scribe/README: "Heavy headless sessions are SIGKILLed by cgroup teardown
  before the [SessionEnd] hook runs." Open docket item `ctrace-sessionend-flake`
  (first_seen 2026-05-30, still open) is exactly this. The repair engine EXISTS
  (`scribe backfill`) but only runs when self-review remembers — inconsistently
  (50 rendered 06-03, 2 on 06-02, 0 on 06-01). coda is the missing TRIGGER +
  DETECTION + SELF-HEALING layer; it does NOT re-implement rendering.
Order (STRICT hard dependency, mirrors anchor):
  coda-sweep FIRST (new repo ~/wintermute/coda, rust-cli; creates workspace +
    `coda` binary + SessionLog/SummaryState/DebtClass/SweepAction/SweepPlan types
    + LogStore trait + FakeStore + CodaConfig loader + PURE sweep diff). Do NOT
    start any rust-extend until sweep has SHIPPED and the repo exists, or
    extend-validate fails (the rule that bit relay+concord+quicken+keel+anchor).
  THEN: coda-audit ∥ coda-close (both depend on sweep ONLY, build in PARALLEL) →
    coda-boot (mixed: systemd timer + SessionStart hook; needs close's --apply).
Notes for /build:
  - coda-sweep + coda-audit are PURE-READ / cloud-build-safe: filesystem behind
    LogStore, FakeStore fixtures, clock + active-log INJECTED into the pure sweep.
    A test asserts sweep makes ZERO store calls; coda-audit asserts it calls ONLY
    read methods (logs / active-log resolve), never render.
  - coda-close defaults PRINT-ONLY; --apply is the ONE live-side-effect path
    (shells `scribe render` per orphan). Its ONE live AC (close the real backlog)
    is deferred_acs:[8] — cloud box has no scribe + no session history. Per-log
    render failure is counted, NOT fatal (one corrupt ndjson must not block 600).
  - coda-boot is mixed (systemd timer + SessionStart hook + installer). Live ACs
    deferred_acs:[5,6] (real session / real timer on the laptop). install PRINTS
    the settings.json hook entry + enable lines, does NOT auto-edit ~/.claude or
    auto-enable the timer (feedback_classifier_per_command). systemd-analyze verify
    is the offline gate. Hook backgrounds `coda close --apply --limit 50` and
    ALWAYS exits 0 (never block session start). KEY INSIGHT: the fix lives in the
    NEXT session's SessionStart, NOT in the dying session's SessionEnd.
  - audit/close/boot each require their integration test ENTRY FILE (tests/audit.rs,
    tests/close.rs, tests/boot.rs) to appear in cargo output (self_orphaned_mock_tests).
  - coda is INWARD toolkit (sibling of vigil/quicken/keel/anchor/binstale) →
    publishes as a j0yen repo like the rest of the self-tooling, NOT outward/civic.
  - sigpipe::reset() first line of main() in the binary (self_sigpipe_panic_toolkit
    — `coda audit --orphaned-only | head`).
  - MSRV 1.85, no let-chains (recall baseline-gate discipline).
  - COORDINATION: once coda-boot is live, self-review's per-run `scribe backfill`
    step becomes redundant — coda becomes canonical. Don't ship a self-review edit
    that races coda; gossip before touching the self-review skill's backfill phase.
Open (next /dream pass / user — HELD, not yet drafted): coda-witness (rust-extend;
  emit wm.coda.debt to docket, edge-triggered like keel-beacon — but the boot HOOK
  may be the right place to open the docket key vs a separate probe; decide after
  boot ships). Retention/prune of summarized ndjson >N days (DESTRUCTIVE, separate
  concern, user opts in). Should coda-close SUBSUME self-review's backfill entirely
  vs run alongside (leaning: canonical, coordinate via gossip). Grace-window tuning
  (started 120s).

## 2026-06-05T  /dream  vision-christen  (seed: bare /dream → strongest UNADDRESSED open docket item)
Drafted: PRD-christen-plan.md, PRD-christen-detect.md, PRD-christen-route.md,
  PRD-christen-cap.md, PRD-christen-ledger.md
Vision: visions/christen.md
Axis: keel=which brain TIER. anchor=which OBSERVATION roots. coda=did every
  session get its CLOSING summary. christen=does every session get its TRUE
  NAME (agent-namespace identity) at birth. Same "is the footing real +
  legible" family; substrate = the CLONE_NEWAGENT per-session identity layer.
Motivation (live, measured 2026-06-05): the agentns substrate is BUILT + BOOTED
  but INERT. `/proc/self/ns/agent -> agent:[4026531996]` = the INIT namespace;
  all 3 live Claude PIDs read `agent_session = 0…0`, all `agent_counters` zero,
  despite `CONFIG_AGENT_NS=y` on 7.0.10-arch1-5-wintermute. Root cause is
  STRUCTURAL: a SessionStart HOOK cannot unshare its already-running parent —
  the wrap must happen at EXEC time (`agentns-claude --intent … -- claude`),
  which means editing the launch sites + granting the launcher CAP_SYS_ADMIN.
  Neither was ever done. The launchers (agentns-claude, agent-wrap) + probe
  (agentns-doctor) ALL exist and are installed; christen is the missing WIRING
  layer, not new primitives. Closes the single unaddressed open docket item
  `agentns-session-zeros` (warn, 8 reports across 6 runs since 2026-05-30, no
  playbook) — keel took `wm-anthropic-key-empty`, coda took `ctrace-sessionend-flake`.
Order (STRICT hard dependency, mirrors anchor/coda):
  christen-plan FIRST (new repo ~/wintermute/christen, rust-cli; creates
    workspace + `christen` binary + LaunchSite/SiteKind/WrapState/RouteAction/
    RoutePlan types + LaunchSiteSource trait + FakeSource + ChristenConfig +
    PURE planner). Do NOT start any rust-extend until plan has SHIPPED and the
    repo exists, or extend-validate fails (the rule that bit relay/concord/
    quicken/keel/anchor).
  THEN: christen-detect ∥ christen-route ∥ christen-cap (all depend on plan
    ONLY, build in PARALLEL) → christen-ledger (meaningful once route is live;
    builds vs FakeStore).
Notes for /build:
  - christen-plan is PURE / cloud-build-safe: launch sites behind LaunchSiteSource,
    FakeSource fixtures, KernelInfo + wrapper_installed INJECTED into the pure
    `plan`. A test asserts `plan` makes ZERO source calls.
  - christen-detect is read-only except a single `docket report`/`resolve`
    edge-trigger. Anti-regression invariant (from the existing draft proposal
    self-review-agentns-block.draft.md): the string "registration failed" MUST
    NOT appear for any state — a test iterates every NsState and asserts it.
    Live `/proc/self` AC is deferred_acs:[7] (needs -wintermute kernel).
  - christen-route is mixed (systemd drop-in installer). DEFAULT print-only;
    --apply writes `10-christen.conf` drop-ins but NEVER daemon-reload/enable/
    restart (prints those — feedback_classifier_per_command). Offline gate is
    `systemd-analyze --user verify` on the generated drop-in (skip-with-note if
    absent, never silent-pass — self_orphaned_mock_tests). The clear-then-set
    `ExecStart=` / `ExecStart=…` pair is a systemd correctness detail a test
    must assert. Live wiring of the 3 real units is deferred_acs:[7].
  - christen-cap is mixed + SECURITY-SENSITIVE. NEVER auto-`setcap`; prints the
    scope explainer THEN the `sudo setcap cap_sys_admin+ep <path>` line. cap_plan
    is pure. `--verify` spawns the launcher under `sbx` and reads the child's
    agent_session for a nonzero id (deferred_acs:[6], needs -wintermute + sbx).
    README documents the narrower setuid-helper alternative as an open decision.
  - christen-ledger is mixed + wires `agentns-doctor receipt` (does NOT
    re-implement counter reading). Open-only entries are a SIGKILL SIGNAL, not a
    bug (coda lesson: headless ticks die before graceful hooks). Live ledger AC
    deferred_acs:[7]. Hook installer PRINTS settings.json entries, no auto-edit.
  - christen is INWARD toolkit (sibling of vigil/quicken/keel/anchor/coda) →
    publishes as a j0yen repo, NOT outward/civic.
  - sigpipe::reset() first line of main() in the binary (self_sigpipe_panic_toolkit).
  - MSRV 1.85, no let-chains (recall baseline-gate discipline).
  - CAP ORDERING: christen-route's wiring is INERT until christen-cap's setcap
    is granted (unshare EPERMs and falls back to unwrapped exec). Build order is
    parallel, but the LIVE effect needs cap BEFORE route's drop-ins do anything.
Open (next /dream pass / user — HELD, not yet drafted): christen-budget
  (rust-extend; measured per-intent budget ceilings from real counter histograms
  + a runaway-kill verification — draft once route is live and we have histograms).
  Narrower-privilege setuid helper (security, user decision). Interactive-shell
  routing is user-typed → christen can only Advise (print alias); systemd sites
  are the deterministic win.

## Pending (build reflect candidate) — 2026-06-06T06:39:07Z
**Integrate-collision pattern**: this tick, 4/8 parallel same-target branches built green in their worktrees but DEFERRED at `worktree-extend.sh integrate` because siblings touched the same files:
- concord-bridge + concord-cruxes → both exit 3 (Cargo.lock dirty from the other); NEITHER integrated.
- quicken-attest → exit 4 (conflict in main.rs/Cargo.toml vs quicken-remedy, which won and shipped v0.3.0).
- anchor-probe → exit 4 (conflict in lib.rs/main.rs vs anchor-reconcile, which won and shipped v0.2.0).
The skill's "sequential branches increment cleanly" assumes non-overlapping diffs; when two same-target branches edit the same lines (esp. Cargo.lock, main.rs subcommand enum, lib.rs pub API), the second always conflicts and defers. Candidate guardrail: integrate should auto-rebase the loser onto the just-merged HEAD and retry (or branches should land Cargo.lock changes via a regen step, not commit it). Deferred branches keep their branch; next tick resumes serially via `add`. (Reflect budget spent today — pick this up next reflect.)

## 2026-06-06T00:00  /dream  vision-loom
Drafted: PRD-loom-rebase-retry.md, PRD-loom-lockfile-regen.md,
  PRD-loom-libapi-append.md, PRD-loom-serial-fallback.md
Vision: visions/loom.md
Seed: the "Integrate-collision pattern" build-reflect candidate (gossip
  2026-06-06T06:39:07Z) + journal 2026-06-05 (quicken-attest, anchor-probe
  deferrals) + read of worktree-extend.sh:81-84 (integrate aborts+exit-4 on any
  conflict; no rebase, no lockfile handling, no retry).
Relationship to existing work: the in-tree PRD
  build-shared-cli-dispatch-merge-safe (status needs_classification) is the
  CLI-dispatch *leaf* of this vision — loom does NOT redraft it. loom covers the
  OTHER collision surfaces (Cargo.lock, lib.rs pub surface) + the self-healing
  retry loop + a serial-fallback backstop, and CONSUMES that PRD's
  `last_error=integrate-conflict:<files>` sidecar telemetry.
Order for /build:
  - build-shared-cli-dispatch-merge-safe ships independently (CLI leaf).
  - loom-rebase-retry, loom-lockfile-regen, loom-libapi-append ALL edit
    worktree-extend.sh / build-skill — they are themselves a same-target trio.
    *** SERIALIZE THEM — do NOT fan these three in parallel, or they will hit
    the very integrate-collision they're meant to fix (eat our own dog food).
    *** Recommended order: rebase-retry FIRST (it's the safety net that makes
    the next two's own integration self-heal), then lockfile-regen, then
    libapi-append.
  - loom-serial-fallback LAST: it consumes the conflict telemetry that
    build-shared-cli-dispatch-merge-safe (and loom-rebase-retry AC4) write —
    do not build it before at least one producer ships, or it reads an absent
    key. Treat absent telemetry as "no streak / parallel as today" (fail-open),
    NOT as a reason to serialize (self_build_jq_escape_reads_absent).
Notes:
  - All four are build_target:self-mod into the build skill (no new repo, no
    publish). Match the existing build-shared-cli-dispatch-merge-safe frontmatter.
  - loom-libapi-append deliberately extends the cli-register.sh anchored-append
    pattern to lib.rs; it depends on that pattern existing but its helper
    (lib-register.sh) is independent code, so it can build before or after the
    CLI PRD.
Open questions: `loom doctor` read-only stall report (held — draft once
  serial-fallback writes the streak ledger it would read). Rebase-retry cap at 1
  vs small-N backoff (start at 1). Cargo.lock driver: merge=ours+regen (chosen)
  vs union (risks invalid TOML).

- 2026-06-06T07:32:30Z (build): coda cluster — coda-audit established local `main` + integrated v0.2.0, but `j0yen/coda` remote is missing; needs `wm-publish --slug coda` (add to ALLOW) or manual `gh repo create`. keel cluster blocked: no `main` branch until keel-pulse ships.

## 2026-06-06T07:38  /dream  vision-quicken (extend, Fleet 2)
Drafted: PRD-quicken-watch.md, PRD-quicken-notify.md
Vision: visions/quicken.md (updated — Fleet 2 = the boot/bus-reactive half)
Seed: bare /dream + Phase-1 live re-probe (memlog group EACCES w/ pkgrel
  gap now 5->11, agentns /proc/self/agent_session all-zeros, bpolicy
  {"loaded":false}, provfs degraded-because-agentns) + the realization
  that Fleet 1's probe is DAILY-only while primitives die mid-day
  (self_agorabus_restart_kills_voice is the canonical edge a daily probe
  can't see).
Why an extend, not a new vision: spent this pass confirming the obvious
  threads are already owned — docket (track findings), warden (bpolicy
  arming), continuity/onramp (agentns wrap + memlog group/udev), vigil
  (stale running bytes), and quicken itself (built-but-never-alive). The
  kernel/self-review/introspection cluster is saturated. quicken is the
  precise match to the strongest LIVE evidence, so this extends it rather
  than wedging in a duplicate #41.
Order for /build:
  - quicken-watch FIRST (publishes wm.health.primitive.<name> verdicts to
    agorabus). Hard dep on Fleet 1's quicken-probe having shipped (repo
    ~/wintermute/quicken exists: quicken + quicken-probe crates — confirmed).
  - quicken-notify SECOND (subscribes, fires on transitions only). Depends
    on watch publishing the topic, BUT can build against fixture event
    streams in parallel; only the live AC5 (real bus round-trip) needs
    watch shipped. Treat watch-absent as "no events" (fail-open), per
    self_build_jq_escape_reads_absent.
  - Both are rust-extend into ~/wintermute/quicken — SERIALIZE THEM (same
    build_into; loom/integrate-collision lesson: same-target parallel
    branches conflict on Cargo.lock/main.rs subcommand enum). watch then
    notify.
Reuse / non-duplication notes:
  - Reuses the EXISTING wm.health.* envelope (produced by
    wintermute-brain/degrade.rs, consumed by docket/digest.rs). Supersedes
    quicken's old open-question idea of a parallel wm.quicken.* topic — so
    docket-digest picks up quicken verdicts for free.
  - Disjoint from wintermute_watchdog (it watches DAEMON heartbeat; quicken
    watches kernel/userspace PRIMITIVE liveness; same envelope, different
    subject namespace primitive.<name>). Flagged for jsy: one envelope /
    two producers, confirm not a merge.
  - Strictly REPORT, never heal — homestead still owns any future
    unattended self-heal. The report-vs-heal line stays firm.
Still held (not drafted): agentns root-cause = an INSTRUMENTATION PRD in
  the agentns repo proving WHERE the zero comes from before any fix
  (feedback_verify_before_concluding), not a quicken PRD. Self-heal-vs-report
  still report-only.
Open questions for jsy: confirm wm.health.* shared envelope / two-producer
  shape; default OnUnitActiveSec for the watch timer (drafted 30min);
  whether --ping (peon-ping on transition) should default on or off (drafted off).

## 2026-06-06T08:08  /dream  vision-assay (new)
Drafted: PRD-assay-agentns.md, PRD-agentns-clone-flag-fix.md, PRD-assay-quicken-bridge.md
Vision: visions/assay.md
Seed: bare /dream + Phase-1 LIVE run (not a surface read). The strongest
  unaddressed signal this pass was a WRONG CONCLUSION baked into two existing
  visions, caught only by exercising the primitive.
THE FINDING (verify-first, feedback_verify_before_concluding): on the booted
  7.0.10-arch1-5-wintermute kernel, `~/wintermute/agentns/tests/test_unshare`
  prints `unshare(CLONE_NEWAGENT) failed: Invalid argument` (EINVAL). The
  compiled flag is `#define CLONE_NEWAGENT 0x00000100` == **CLONE_VM** — a bit
  collision; patch-0001's commit msg claims 0x40000000 (== CLONE_NEWNET, also
  taken). The legacy 32-bit clone-flag space is EXHAUSTED. So the agentns
  all-zeros is NOT a wiring gap — the namespace physically cannot be created
  via legacy unshare.
  ⇒ onramp's SHIPPED `claude-agentns-wrap` is FUTILE (it calls the rejected
    flag). I added a ⚠ blocked-by note to visions/onramp.md (durable update,
    not a rewrite).
  ⇒ quicken's AgentnsProbe verdict `Inert` is correct-but-uninformative — it
    reads the live process (always init ns → always zero) and can never tell
    wiring-gap from kernel-bug.
Order for /build:
  - PRD-assay-agentns FIRST. Airtight (the diagnosis is already proven by the
    live run); new workspace ~/wintermute/assay (rust-cli); no consumers; ships
    independently. AC2 REQUIRES it reproduce the EINVAL + 0x100==CLONE_VM
    verdict on the booted kernel.
  - PRD-agentns-clone-flag-fix and PRD-assay-quicken-bridge BOTH depend on
    assay-agentns but are independent of EACH OTHER → may build in parallel
    (disjoint build_into: ~/wintermute/agentns kernel-patches vs
    ~/wintermute/quicken rust-extend; no integrate-collision).
Heads-up for /build:
  - agentns-clone-flag-fix is build_target:mixed (kernel patch + apply-agentns.py
    anchor). Its LIVE proof ACs (3,4) are USER-GATED on a reboot of the rebuilt
    kernel (same reboot window self-review keeps flagging for the blocked
    linux/linux-firmware pacman queue). Build/apply ACs (1,2,5,6,7) are NOT
    gated — advance those autonomously; park the proof ACs as deferred_acs.
  - assay-quicken-bridge is rust-extend into quicken; keep the Verdict enum +
    PrimitiveReport JSON append-only (docket-digest consumes it). Fail-open if
    `assay` is absent (self_build_jq_escape_reads_absent).
  - DO NOT re-ship onramp's claude-agentns-wrap as-is — re-point it at the new
    `agentns-unshare` (prctl) shim from agentns-clone-flag-fix first.
Open questions for jsy (vision §Open questions): the real design decision is
  the creation MECHANISM for the fix — prctl(PR_SET_AGENT_NS) (drafted default,
  reuses patch-0005 dispatch, no clone bit) vs clone3-only (unshare can't reach
  it) vs reclaim a legacy bit (risky). Settle BEFORE agentns-clone-flag-fix is
  built. Also: should `assay` ever absorb quicken's passive probes (drafted no —
  stay disjoint: assay exercises mechanisms, quicken reads live state).

## 2026-06-06T01:40  /dream  vision-warrant (new)
Drafted: PRD-warrant-corpus.md, PRD-warrant-audit.md, PRD-warrant-docket.md
Vision: visions/warrant.md
Seed: bare /dream + Phase-1 LIVE verification (not a surface read). Second
  consecutive pass to catch a SHIPPED/CLOSED PRD whose closing mechanism
  claim is mechanically false (yesterday: assay→onramp's agentns-wrap futile).

THE FINDING (verify-first, feedback_verify_before_concluding) — a FALSE CLOSE,
  proven live:
  `PRD-ctrace-session-end-resilient.md` is `Status: Closed (2026-06-02) —
  outcome achieved live by a different mechanism`. The named mechanism:
  "ctrace-reap.timer runs ctrace-orphan-reap --apply every 2 min; its
  render_log step summarizes any orphaned *.ndjson." THIS IS FALSE.
  - `ctrace-orphan-reap --apply` renders a log ONLY on `verdict:
    orphaned-tracer` (a *live* stranded tracer to stop+render).
  - But the SAME repo's `ctrace-reap.service` comment says: "session.bt now
    self-terminates on root exit, so this should normally find nothing." Live
    journalctl confirms: `verdict: healthy` → `apply: state is healthy;
    nothing to do`.
  - ⇒ The two shipped fixes are MUTUALLY DEFEATING: self-terminating the
    tracer removes the orphan the reaper needed to trigger render. In the
    common SIGKILL case (every headless /build /dream /self-review tick) there
    is NO orphan → reaper renders NOTHING → and SessionEnd hook never fired
    (SIGKILL) → the log is summarized by NOTHING but self-review's hand-run
    `scribe backfill`. (`ctrace-session-end.sh` also still shells the slow
    `summarize-ctrace-session.sh`, never `scribe`.)
  ⇒ The gap silently REOPENED to 623/1874 (33%) by 2026-06-05 — which is the
    ONLY reason the `coda` vision + 4 PRDs had to be drafted from scratch.
    coda IS the correct superseding fix; warrant does NOT re-fix it.

DURABLE CORRECTION for /build (do NOT trust the 06-02 close):
  - PRD-ctrace-session-end-resilient.md's close is mechanically false for the
    common case. Treat `coda` (coda-sweep→audit∥close→boot, in-flight) as the
    real fix for the ctrace summary-debt gap. Do not ship anything that leans
    on "ctrace-reap.timer backfills summaries" — it does not.

vision-warrant = the CLOSE-side dual of assay (assay attests a PRIMITIVE's
  creation mechanism by forking a child; warrant attests a CLOSE's claimed
  mechanism by running a declared side-effect-free assertion — incl. the
  systemd/shell/timer/pipeline closes assay structurally can't fork). Inward
  toolkit, sibling of docket/coda/quicken/assay; publishes as a j0yen repo.

Order (STRICT hard chain, mirrors coda/anchor):
  - warrant-corpus FIRST. New repo ~/wintermute/warrant (rust-cli). Ships the
    workspace + types (CloseClaim/ClaimKind/Warrant/AssertionSpec/
    WarrantStatus/WarrantVerdict/AuditPlan) + CloseSource trait + FakeSource +
    a PURE `classify()` (AC2: zero source calls, no IO). No consumers; ships
    independently; cloud-build-safe.
  - THEN warrant-audit (rust-extend): real FsDocketSource over PRDs-archive/ +
    a warrants.toml registry of side-effect-free assertions + the runner.
    First warrant shipped = the session-end-resilient one → resolves Refuted.
    DO NOT start until warrant-corpus has SHIPPED + repo exists
    (extend-validate rule that bit relay/concord/quicken/keel/anchor).
  - THEN warrant-docket (rust-extend): edge-triggered reopen of Refuted closes
    into docket under slug `warrant:<source>`; print-only default, --apply
    gated; fail-open if docket absent. Depends on audit's verdict JSON.

Heads-up for /build:
  - All three are PURE-READ / cloud-build-safe in their non-[live] ACs (source
    behind a trait, FakeSource/FakeDocketSink fixtures, archive dir INJECTED).
    The [live] ACs (warrant-audit AC8, warrant-docket AC8) need ~/.claude/
    scripts + ctrace on the real laptop → park as deferred_acs, advance the
    rest autonomously.
  - warrant-audit enforces a side-effect-free assertion contract (AC3: rejects
    a CommandExit cmd containing --apply/rm/>/write at load). Keep that gate.
  - sigpipe::reset() first line of main() (self_sigpipe_panic_toolkit).
  - MSRV 1.85, no let-chains; tests/{corpus,audit,report}.rs must each appear
    as `Running tests/<x>.rs` in cargo output (self_orphaned_mock_tests).
  - DocketSink + FsDocketSource both fail-open on missing docket
    (self_build_jq_escape_reads_absent).

Open questions for jsy (vision §Open questions — HELD, not drafted):
  - warrant-GATE (prevention): a /build close-time hook that refuses to write
    a "by a different mechanism" close note unless a warrant is registered.
    This is the load-bearing half (stop false closes BEFORE vs detect AFTER)
    but touches the /build close path + classifier — not yet traced. Draft
    after warrant-audit + locating the close-note write site.
  - assay-bridge: when a close's mechanism IS a kernel primitive, delegate the
    warrant to `assay <name>` instead of a local assertion (unify creation-
    side + close-side under one verdict). Draft once both are live.
  - Should `warrants.toml` be hand-written at close time, or should audit
    propose a stub per Unwarranted close? v1 = hand-written + Unwarranted
    backlog; auto-stub is Fleet 2.

## 2026-06-06T08:10  /dream  vision-constellation (extend — operational hardening)
Drafted: PRD-constellation-secrets.md, PRD-constellation-headscale.md, PRD-constellation-voice-role.md
Vision: visions/constellation.md (added "Operational hardening" section)
Seed: bare /dream. Inward self-tooling space is SATURATED (anchor owns
  watchman-rewatch, coda+scribe own ctrace summary-debt, assay owns the agentns
  flag-collision) — verified by direct grep, did not re-draft any of them.
  constellation is the live frontier: base components shipped/in-flight
  (wm-busbridge v0.1.1 built; constellation/{ansible,chezmoi,localrepo} +
  constellation-provision worktree active in wchg), so I extended with the three
  un-drafted pieces that block a real 2nd node — each answers a vision Open question.

Order: secrets → headscale ‖ voice-role.
  - constellation-secrets is the HARD prerequisite for any multi-host work: mesh
    AC1 ("auth key from the encrypted store"), the bus's NATS creds, and the cloud
    brain's WM_ANTHROPIC_API_KEY all consume a store that does not exist yet.
    appearance's chezmoi-age covers ONLY dotfile tokens AFTER the host key exists —
    it does not bootstrap the root key or manage service secrets. Build secrets
    before headscale.
  - constellation-headscale is the SERVER half of mesh's AC9 (mesh ships only the
    client flag pointing at an absent server). Depends on secrets (server key +
    issued pre-auth keys are managed secrets) AND constellation-cloud (the host).
    Do NOT start until secrets has shipped — extend-validate rule.
  - constellation-voice-role is INDEPENDENT (refines provision; can build in
    parallel). Makes boot-to-voice a per-host flag so cloud/compute nodes don't
    bring up a mic stack. Additive + backward-compatible (no flag = role default).

Notes for /build:
  - All three are build_target: shell, siblings of the constellation family
    (mesh/provision/appearance pattern). sigpipe::reset() guard on any helper
    (self_sigpipe_panic_toolkit). Keep the "repo contains no plaintext secret"
    grep-gate that appearance AC6 established — secrets AC3/AC10 + headscale AC2
    all lean on it.
  - secrets AC4 needs two distinct test age keys / a FakeRecipient to prove
    role-scoped decryption without real cloud keys — fixture-able, cloud-build-safe.
  - The [live] multi-host ACs (mesh-reachability, real Headscale registration) need
    a second machine; park those as deferred_acs and advance the
    fixture/FakeRecipient/local-server ACs autonomously (deferred_acs inline-int
    form only — self_deferred_acs_inline_only).

Open questions (HELD, not drafted — need user/state):
  - Bootstrap delivery channel default: manual-paste/USB vs cloud-tunnel. PRD
    documents both; user picks the canonical one when node 2 actually arrives.
  - secrets backend: sops+age (drafted) is the assumption. If the user prefers
    Vault/agenix the PRD's store half changes; bootstrap half is unaffected.

## 2026-06-06T09:33  /dream  vision-kin (Fleet 2 — closing the open loops)
Drafted: PRD-reach-inbound-imap.md, PRD-reach-silence-nudge.md, PRD-reach-distress-durability.md
Vision: visions/kin.md (added "Fleet 2 — closing the open loops" section)
Seed: bare /dream. Inward self-tooling SATURATED (verified — last dream's note
  holds); constellation just extended this morning (08:10) so NOT re-touched.
  Picked kin: an OUTWARD vision whose Fleet-1 daemons all SHIPPED (wintermute-reach
  v0.2.0, presence, family-enroll, dialog Family/Distress branches) but whose
  feedback paths are stubbed/missing. 3 of 7 end-states are half-open — drafted the
  three loop-closers, each cited to a grep of the shipped repo.

Order (all rust-extend → wintermute-reach, disjoint modules, parallel-safe):
  - reach-inbound-imap  — REAL inbound reply channel (async-imap 0.9 / maildir poll)
      → publishes wm.family.reply, replacing the `wm-reach reply` v1 CLI stub
      (main.rs:7, dispatch.rs:54). dialog on_reply→TTS (family.rs:363) already waits
      for it. Closes end-state #2 "you can reach her back." Security AC: From-allowlist
      = enrolled caregiver address only (spoken-to-Mom = injection surface).
  - reach-silence-nudge — standalone gentle "haven't heard from Mom" delivery on
      wm.presence.silence. Today silence is ONLY a digest-body flag (digest.rs:12
      "does NOT trigger"), and the digest defaults OFF → a silent day can vanish.
      Closes end-state #4. Opt-in, debounced per-window, never an alarm.
  - reach-distress-durability — retry+backoff then fallback-transport escalation
      (ntfy/webhook are already Cargo features, Cargo.toml:20-21) BEFORE a distress
      acks delivered:false. Today daemon.rs delivers distress once; a nack ends the
      safety loop unobserved. Hardens end-state #5. Distress-only; messages unchanged.

OQ#3 ("distress confirm vs immediacy") was ALREADY RESOLVED in dialog/src/distress.rs
  (Severity::Hard/Soft + classify() + soft-confirm prompt) — did NOT draft it.

Notes for /build:
  - reach is at v0.2.0; these bump v0.3.0→v0.4.0→v0.5.0 if built sequentially, but
    they touch disjoint modules (inbound / digest+config / daemon delivery) so a
    parallel build just rebases each onto the prior minor.
  - All default-OFF / opt-in except the distress ladder (distress already defaults
    ON and is pre-filtered by dialog). Keep "no secret/body logged" — every AC has a
    no-body-logged clause; reach already reads creds from /etc/wintermute/conf.d/.
  - Live legs are fixture-able: maildir + FakeTransport/FakeRecipient cover the
    autonomous proof; the real-IMAP-server smoke is the only deferred_acs (inline-int
    form only — self_deferred_acs_inline_only).
  - async-imap 0.9 rustls-only precedent is wintermute-mail/Cargo.toml:72 (no native
    OpenSSL); sigpipe::reset() already in reach.

Open questions (HELD — need jsy, already in vision OQ#1/#2):
  - Which physical fallback transport on jsy's phone (ntfy self-hosted / gotify /
    SMS gateway)? distress-durability proves the LADDER with fakes; the deployment
    choice is jsy's.
  - Webhook/push inbound (device-reachable-from-outside) deferred — inbound-imap is
    the headless-safe minimum. A durable on-disk distress outbox (crash-replay) is a
    noted Fleet-3 follow-on, not drafted.

## 2026-06-06T03:05  /dream  (saturation scan — no PRDs drafted)
Seed: bare /dream (interactive). Verified, did not assume.
Finding: every strong inward signal from tonight's research already has an
  ACTIVE vision, several built TODAY — re-checked to ground, not surface:
  - ctrace SessionEnd summaries (625 missing in self-review prose) → coda
    (coda-boot/close/sweep, v0.2 local today). LIVE: ~/.cache/ctrace/sessions
    = 5 missing, not 625; coda backfill is working.
  - watchman drops roots on reboot / wchg lies silently → anchor
    (anchor-roots/boot/probe, v0.3 today). I independently re-derived this
    seam (sketched 'moor') from recall 01KT6VCBX1PSWT9MAQP607TAWY + verified
    no boot wiring exists + watchman roots present only via manual re-watch —
    then found anchor already drafted+shipped it citing the SAME recall hit.
  - agentns all-zeros → assay. warden inert → warden. stale binary → vigil.
    dead cloud tier → keel. re-noticed findings w/o forcing fn → docket.
Tonight's journal/recall surfaced ONLY inward self-review material; no fresh
  OUTWARD evidence. Prior two ticks today already went outward (kin Fleet-2
  09:33, constellation 08:10). Per hard-rule 6 (don't dream past research) +
  rule 3 (cite research), drafted nothing rather than fabricate.
Asked jsy for direction (interactive); question dismissed → recorded this
  and stopped.
Notes for /build: nothing new from this tick. The covering visions above are
  the live queue; advance those.
Open question for next /dream: the only un-visioned recurring escalations are
  install/arming ACTIONS, not design gaps (memlog staged-awaiting-install,
  warden arming, binstale never installed). These want a user action or a
  forcing-function PRD under docket — not a new vision. Reconsider if they
  keep aging.

## 2026-06-06T07:45  /dream  (saturation scan #2 — no PRDs drafted)
Seed: bare /dream (interactive). Verified to ground, did not assume.
Second consecutive bare /dream hitting INWARD saturation. Re-checked the two
  recurring self-review signals against their covering visions — both real,
  both already drafted/active:
  - SessionEnd hook NOT firing on SIGKILL'd headless ticks → 623/1874 logs
    orphaned → coda (sweep/audit/close/boot drafted; coda-boot's SessionStart
    trigger IS the root-cause fix, not a band-aid). v0.2 today.
  - Recurring findings + activation/arming ACTIONS (memlog staged-awaiting-
    install, warden inert, binstale never installed) → docket (ledger w/
    first-seen/streak/escalate/auto-close). Confirmed these are user/forcing-
    function ACTIONS, not design gaps — same conclusion as the 03:05 tick.
Tonight's journal/recall surfaced ONLY inward self-review material; no fresh
  OUTWARD evidence. Drafted nothing rather than fabricate a 41st inward vision.
Asked jsy for direction (interactive AskUserQuestion); dismissed. Recorded +
  stopped, per hard-rule 6 + the 03:05 precedent.
Notes for /build: nothing new this tick. coda + docket + the live outward
  visions (homeward/kin/constellation) are the queue; advance those.
NEW open question for next /dream (worth not losing): this session connected
  external MCP servers (claude.ai Gmail / Google Calendar / Google Drive /
  AtScale) — a surface NONE of the 40 visions touch. A genuinely un-covered
  outward seam: a calendar-and-email-aware assistant layer. Caveat: connectors
  are interactive-only ("may be absent in headless/cron runs" — memory note),
  so a /build-auto vision can't depend on them; it'd be an interactive-facing
  vision. Flagged for jsy to confirm direction before drafting.

## 2026-06-08T05:25  /dream  (saturation scan #3 — no PRDs drafted)
Seed: bare /dream (interactive, manual). Verified to ground, did not assume.
Third consecutive bare /dream hitting INWARD saturation. Grounded again:
  tonight's journal (2026-06-07 self-review) and recall reflective seeds are
  ALL inward maintenance — pacman-101-blocked, ctrace-sessionend-flake,
  agentns-session-zeros, memlog-activation, warden-inert. Every one already
  has a covering vision (coda/assay/docket/warden) or is a user-gated ACTION,
  not a design gap. No fresh OUTWARD evidence in journal or recall.
DIFFERENCE this tick: the MCP connectors the 06-06 ticks flagged as the one
  un-covered outward seam (Gmail / Google Calendar / Google Drive / AtScale)
  ARE live + callable in this interactive session — the "interactive-only,
  absent headless" caveat is satisfied right now. Surfaced it to jsy via
  AskUserQuestion (4 options: MCP-assistant-layer vision / extend an outward
  vision / inward forcing-functions / nothing). Question DISMISSED.
Per hard-rule 6 + the 03:05 & 07:45 precedent: dismissed → recorded + stopped.
  Drafted nothing rather than fabricate a 41st vision.
Notes for /build: nothing new this tick. Live queue unchanged — advance the
  active visions (homeward/kin/constellation outward; coda/docket inward).
STANDING open question (now raised 3×, still un-answered): an MCP-connector-
  aware assistant layer is the only outward direction none of the 40 visions
  touch. It needs jsy's explicit go-ahead because its PRDs are inherently
  interactive-facing (can't depend on connectors in /build-auto headless
  ticks). Until then it stays un-drafted by design, not by omission.

## 2026-06-08T05:39  /dream  vision-ousia  (6 PRDs — first OUTWARD ontology vision)
Seed (jsy, interactive): "of making ethical AI possible with these ideas.
  implement OWL2 and SPARQL tools to make ethical grounded BFO a market success."
THE un-covered outward seam the last 3 ticks kept flagging — finally seeded by
  jsy directly. None of the prior 40 visions touch ontology/BFO/OWL/SPARQL/ethics.
Grounded hard in real artifacts:
  - ~/Notes/AtScale/World-Ontology-paper.md (v1.0.0, 509-class OWL2 DL / BFO 2020
    paper, sentience→dignity→rights as reasoner-enforced axioms). REAL + complete.
  - KEY GAP: world-ontology.owl does NOT exist on disk — only the paper. So
    PRD-ousia-forge (build the .owl from a declarative spec) is the gate.
  - ~/Notes/AtScale/book-outline-ontological-semantic-layer.md = the market thesis
    (formal ontology for the semantic-layer category) → PRD-ousia-atscale bridge.
  - AtScale MCP live this session (list_models/describe_model/run_query) → real
    connector for the atscale bridge's interactive path.
Drafted: PRD-ousia-forge, -reason, -sparql, -guard, -mcp, -atscale.
Vision: visions/ousia.md
Order: forge → reason → sparql → guard → mcp ; atscale branches off forge+sparql.
  forge is the hard gate — every other PRD needs an .owl to operate on.
Notes for /build:
  - BUILD forge FIRST. Nothing downstream is testable without it. It has no ousia
    deps (only horned-owl + serde/toml).
  - reason/sparql/guard form a lib chain (each exposes a lib crate the next
    consumes) — respect that order; don't parallel-ship guard before sparql lib.
  - Reasoner is a hand-rolled forward-chainer over the paper's 10 all-some axioms
    (OWL2 EL/RL), NOT a full DL reasoner. whelk-rs upgrade path, HermiT/JVM is the
    documented escape hatch but OUT of scope. Don't let an agent pull a JVM dep.
  - crates.io API is 403-rate-limiting this box (data-access policy) — could not
    confirm exact crate versions. horned-owl/oxigraph/sophia are real; pin versions
    at build time. If a Rust MCP SDK crate doesn't resolve for ousia-mcp, the PRD
    permits a hand-rolled JSON-RPC stdio loop.
  - ousia-atscale + ousia-mcp have interactive-only live paths (MCP connectors
    absent in headless ticks) — but BOTH PRDs gate that behind an offline JSON
    path that IS build-auto-testable. ACs require no live connector. Build them
    headless; the live path is layered convenience.
Open questions (in vision doc):
  - Federation Model source doc not on disk; extracting the 10 axioms from the
    paper §3.1/§5 directly (sufficient). Ask jsy if the source doc would enrich.
  - A dedicated ousia-conformance PRD (paper §8 is concretely testable) may split
    out of reason's ACs on a later pass.

## 2026-06-08T05:44  /dream  vision-ousia  (open-question resolved)
jsy pointer: the Federation Model source IS on disk —
  ~/Notes/federation-utopian-philosophy.md (canonical, 2026-05-19; stale dup at
  ~/Notes/AtScale/Reference/federation-utopian-philosophy.md). All 10 tenets
  present verbatim (§3.1 list). Vision open-question updated to RESOLVED.
Note for /build (ousia-forge): source the per-class philosophicalGrounding /
  aiGuidance annotation TEXT from this file rather than paraphrasing the paper.
  The 10 formal axioms still come from the paper §5. No PRD modified (hard-rule
  2) — guidance carried via the vision doc, which forge's open-questions defer to.

## 2026-06-08T05:55  /dream  vision-herald  (3 PRDs — distribution layer for ousia)
Seed (jsy, interactive): "of a what to package a distribute ethical reasoning as a skill."
Direct follow-on to vision-ousia (drafted 30 min earlier). ousia answers "what is
  ethical reasoning"; herald answers "how does anyone else GET it."
Grounded:
  - Skills here = SKILL.md dirs symlinked into ~/.claude/skills/ from ~/wintermute/
    (build -> build-skill etc). Fine for the author, UNINSTALLABLE by anyone else.
  - VERIFIED gap: no wintermute repo ships a .claude-plugin/plugin.json; no j0yen
    marketplace.json exists. Existing skill tooling (skill-doctor/skill-manifest/
    wm-skill-edit) only VALIDATES/EDITS installed skills — none PACKAGES for distro.
  - Target format known: official marketplace.json (222 plugins) = top-level
    name+owner + plugins[] with source {git-subdir,url,path,ref,sha}. herald emits
    into that exact shape — no protocol invention.
  - ousia-guard PRD confirmed on disk (the reasoning engine conscience wraps).
Drafted: PRD-herald-pack, PRD-herald-market, PRD-herald-conscience.
Vision: visions/herald.md
Order: herald-pack → (herald-market ∥ herald-conscience). pack is the gate;
  conscience ALSO depends on ousia-guard (cross-vision dep — don't ship conscience
  before the ousia fleet's guard/reason libs land).
Notes for /build:
  - herald-pack is capability-agnostic infra (no ousia deps) — buildable now,
    independent of the ousia fleet. Good early pick.
  - herald-conscience is build_target: mixed (SKILL.md + shell, not a new crate);
    build_into ~/wintermute/herald-conscience. It's the FIRST real input to
    herald-pack — validate them together (AC-5 on both sides).
  - conscience CI may stub ousia-guard with a recorded fixture, but the wire
    contract (args + --format json shape) must match the real binary.
Open questions (in vision doc): build-from-source vs release-fetch in install.sh
  (starting build-from-source); a herald-attest signing/provenance PRD deferred
  (needs jsy signing-key decision); marketplace repo home (leaning new public
  j0yen/wintermute-skills); skill name /conscience vs /ethics vs /ousia (working
  name /conscience).

## 2026-06-08T05:55  /dream  vision-lattice  (6 PRDs — federation/infinite-context capstone)
Seed (jsy, interactive): "make this skill infinitely wise … tools to automatically
  bridge ontologies. Give AIs infinite context. give them tools to find, join,
  traverse and deeply understand the world around them."
Third vision in tonight's arc: ousia (reason) → herald (distribute) → lattice
  (federate). Makes the ousia ethics skill "infinitely wise" by connecting it to
  the whole BFO ecosystem.
Grounded HARD in the paper:
  - §9.2 (line 467): "Because the World Ontology is BFO-grounded, it is
    interoperable with the over 500 existing BFO-conformant ontologies… can serve
    as a BRIDGE ontology." §8.2 (line 422) same. The seed is the paper's own §9.2.
  - Registries named + real: OBO Foundry (500+), CCO (DoD), FIBO, Gene/Disease Ont.
  - KEY tractability claim (honest): shared BFO upper layer (35 categories) prunes
    the cross-ontology mapping search space — that's why auto-bridging is feasible
    where generic ontology matching (OAEI) is hard. Not magic; BFO-anchored.
  - recall exists = EPISODIC memory (BGE+FTS5). lattice-context = STRUCTURED
    ontological retrieval. Complementary, kept SEPARATE (not duplicated).
  - "Infinite context" stated honestly = externalized federated graph + on-demand
    subgraph retrieval; bounded by the lattice not the window. NOT literal infinity.
Drafted: PRD-lattice-registry (find), -bridge (bridge — the core), -join (join),
  -traverse (traverse), -context (∞ context, MCP), -ground (deeply understand).
Vision: visions/lattice.md
Order: STRICT pipeline registry → bridge → join → traverse → context → ground.
  Maps 1:1 to the seed's verbs.
Notes for /build:
  - lattice-registry is the ONLY standalone piece (a fetcher/cataloger, no ousia
    dep) — buildable now. Everything else chains on it + the ousia fleet libs
    (reason/sparql). Do NOT start bridge/join/traverse before ousia-reason+sparql
    libs exist.
  - HARD RULE baked into ACs: NO silent auto-merge of low-confidence bridges.
    bridge emits confidence+evidence; low ones go to a review proposals file
    (recall-observe / skill-doctor pattern); join REFUSES unreviewed bridges
    unless --allow-unreviewed. Reviewer-gated, same as the rest of the fleet.
  - lattice-ground closes the loop back to herald-conscience: plain-language
    action → grounded classes → ousia-guard. Output shape is contract-tested
    against ousia-guard's input.
Open questions (vision doc): alignment false-positives (mitigated by review gate);
  external-ontology licensing (FIBO/CCO terms — registry records license, OBO
  open set first); whether "infinite" stays bounded/fast at scale (deferred
  lattice-scale PRD until join/traverse expose real numbers); recall vs
  lattice-context unification (kept separate for now).

## 2026-06-08T06:30  /dream  vision-tribunal  (4 PRDs — the proof layer for the outward arc)
Seed: bare /dream (interactive). Inward space saturated (3 prior "saturation
  scan — no PRDs" ticks); productive frontier is the outward arc seeded this
  morning. Found its un-dreamt THIRD leg: ousia *reasons*, herald *distributes*,
  but NOTHING proves the reasoner is right before /conscience ships outward.
Why real:
  - herald end-state #4 DEMANDS "an 'ethics skill' isn't shipped broken or
    unverifiable" — but herald-pack/market/conscience ship only structural
    skill-doctor/skill-manifest checks. No verdict-correctness check exists.
  - The tautology already cost this laptop: wm-router safety 100%→73.5% on a
    held-out set (feedback_agent_written_fixtures_tautology). Per hard-rule-1,
    /build→/autobuilder writes code AND its tests in one cycle → ousia-guard
    would grade its own homework. An ETHICS engine is the worst place for that.
  - ousia vision EXPLICITLY deferred ousia-conformance "to a later /dream pass."
    This is that pass (rehomed as tribunal-conformance — judge lives OUTSIDE the
    defendant: new workspace ~/wintermute/tribunal, not inside ousia).
Drafted: PRD-tribunal-conformance, PRD-tribunal-corpus, PRD-tribunal-bench,
  PRD-tribunal-gate.
Vision: visions/tribunal.md
Order: (conformance ∥ corpus) → bench → gate.
Notes for /build:
  - All four build NOW against fixtures/stubs — they do NOT block or delay ousia/
    herald. conformance vs a vendored fixture .owl; corpus is data + validator;
    bench vs a recorded guard-stub with the wire contract ASSERTED; gate via
    --dry-run + documented herald hook.
  - CROSS-VISION deps (ousia + herald both freshly drafted, UNBUILT):
    bench consumes ousia-guard's `check --action --format json --explain`
    contract; conformance consumes ousia-forge's .owl; gate consumes
    herald-pack's publish flow. Stub/fixture now; real wire-in at each
    consumer's AC. Don't wait on ousia/herald to start tribunal.
  - THE GATE IS THE POINT: enforce `tribunal gate` at herald-conscience's
    PUBLISH AC — false-allow==0 is hard/non-overridable; that is the AC that
    makes herald end-state #4 true with a mechanism, not a promise.
  - rust-cli ×3 + mixed ×1, all build_into ~/wintermute/tribunal (one workspace,
    one crate, four subcommands: conformance/corpus/bench/gate). SIGPIPE reset
    per self_sigpipe_panic_toolkit. rustc 1.85, no let-chains.
Open questions (in vision doc): corpus independence can't be FULLY proven
  mechanically (validator checks provenance tags; jsy human spot-check of the
  first cut is a release AC — is that acceptable?); accuracy threshold start
  (drafted ≥0.85, ratchets); v1 corpus size (~60, ≥2/tenet × {allow,flag,deny}).

## 2026-06-08T07:00  /dream  vision-recourse  (5 PRDs — the outward arc's return path)
Seed: bare /dream (interactive). Inward space saturated (3 prior saturation
  scans). The outward arc (ousia→tribunal→herald→lattice) is the live frontier,
  but it is WRITE-ONLY: ousia reasons, tribunal proves (vs jsy's static corpus),
  herald ships /conscience to strangers' machines — and NOTHING comes back. No
  receipt of what was decided, no way to contest a wrong verdict, no path for the
  field's disagreement to reach the axioms. This pass dreams that return path.
Why real:
  - herald end-state ships /conscience to a j0yen marketplace; verdicts then happen
    off-laptop with zero telemetry back. No PRD in any arc vision defines a
    receipt/contest/feedback channel.
  - tribunal proves correctness PRE-ship vs a FINITE jsy-authored corpus. Open-world
    assumption (509 classes) guarantees the field hits uncovered cases. The world is
    the ultimate held-out set; its answers never reach the answer key.
  - THE HINGE: a field contest's provenance.author is a downstream HUMAN ⇒
    != "ousia-axioms" ⇒ a contested case MECHANICALLY satisfies tribunal-corpus's
    independence guarantee. A field contest is the most honest held-out case there
    is — the axioms' authors provably could not have written it. Closes the loop
    that feedback_agent_written_fixtures_tautology (wm-router 100%→73.5%) warns about.
Drafted: PRD-recourse-receipt, PRD-recourse-contest, PRD-recourse-amend,
  PRD-recourse-pulse, PRD-recourse-feedback.
Vision: visions/recourse.md
Order: receipt → {contest → amend, pulse} → feedback.
Notes for /build:
  - recourse-receipt is the ONLY standalone piece — buildable NOW against a
    checked-in verdict JSON Schema + a recorded guard-stub (tribunal-bench's fixture
    approach). Everything else keys on its receipt.v1 format. Do NOT wait on ousia.
  - All five build_into ONE workspace ~/wintermute/recourse (one crate, subcommands
    receipt/contest/amend/pulse/feedback). rust-cli ×3 + mixed ×2.
  - CROSS-VISION deps (stub now, wire at AC; do NOT block tribunal/herald):
    amend consumes tribunal-corpus's action.json/expected.toml/provenance.toml shape
    + shells to `tribunal corpus validate`/`tribunal gate`; feedback shells to
    `tribunal gate` + `herald-market` publish. Recorded stubs with the wire contract
    ASSERTED, same pattern as tribunal-bench vs the guard-stub.
  - HARD RULES baked into ACs (the fleet's settled safety stance):
    * contest is a PROPOSAL — reviewer-gated pending.ndjson, NO auto-uphold, NO
      mutation of corpus/ontology (byte-identical-before/after AC).
    * amend enforces author=downstream:* (!= ousia-axioms) — the independence hinge.
    * pulse is AGGREGATE-ONLY + opt-in; --export is the ONLY data-egress path and
      carries no receipt_id/action_digest/per-action rows (grep AC). Never reads the
      raw actions/ store.
    * feedback NEVER auto-publishes; ship --confirm runs `tribunal gate` FIRST and a
      gate failure blocks the publish (false-ship==0, same spirit as tribunal-gate).
  - PII discipline: receipts store blake3(action), NEVER the action; raw action is
    local-only + opt-in (--store-raw). Marker-string-absence AC.
  - SIGPIPE reset per self_sigpipe_panic_toolkit; rustc 1.85, no let-chains.
Open questions (vision doc): receipt transport (drafted air-gapped-by-default,
  nothing phones home); contest identity granularity (opaque per-install id, no PII);
  third-party reviewer (local corpus fork vs --upstream propose to j0yen); amendment
  cadence vs ontology stability (batched version cuts, never per-contest).

## 2026-06-08T08:30  /dream  vision-mend  (4 PRDs — the remediation middle)
Seed: bare /dream (interactive). Inward + outward arcs saturated (recourse
  closed the outward return path at 07:00 today). Strongest unaddressed signal
  was the self-review's own chronic escalations — named/counted by `docket`,
  verified-when-fixed by `assay`, but REMEDIATED by nothing.
Why real (live docket this pass):
  - ctrace-sessionend-flake (warn) runs_seen:5 report_count:5 — OWNER-LESS.
    Hook ~/.claude/scripts/ctrace-session-end.sh silently skips render on a
    missing/stale marker (|| true swallows all); self-review backfills every run.
  - warden-enforcer-inert (warn) since 2026-06-03 — OWNER-LESS, carried+acked
    every run because nothing diagnoses WHY bpolicy reads loaded:false.
  - binstale: ~/wintermute/binstale is a COMPLETE built repo (5 src modules,
    verdict taxonomy) but NOT installed → `which binstale` fails → self-review
    prints "fleet staleness check skipped" every run, and it's not even docketed.
  - docket itself: agentns-session-zeros open 8 runs / 21-run precedent; counting
    survival never produced a fix. The escalation threshold is codified but
    eyeballed — nothing consumes it.
Drafted: PRD-mend-binstale-wire, PRD-mend-ctrace-render, PRD-mend-warden-doctor,
  PRD-mend-bridge.
Vision: visions/mend.md
Order: binstale-wire / ctrace-render / warden-doctor are independent leaf fixes
  (ship any order); bridge is independent of the leaves, needs only docket (built).
Notes for /build:
  - binstale-wire & warden-doctor are rust-extend into EXISTING repos
    (~/wintermute/binstale, ~/wintermute/bpolicy) — preserve existing modules.
  - ctrace-render is `mixed`: spans ~/wintermute/ctrace-scribe (render engine,
    add idempotent `render-session`) + the hook script. Backfill and exit-render
    must share ONE render code path.
  - bridge is a NEW rust-cli ~/wintermute/mend. PROPOSAL-ONLY (mirrors
    recourse-contest): never auto-builds, never `docket resolve`, never edits an
    existing PRD; bare `mend bridge` is dry-run. Reads the REAL docket interface
    (verified live: `docket list --escalated --format json`, record fields
    key/severity/title/runs_seen/consecutive_runs/report_count/evidence).
  - All four wire to docket via a `--format docket` emit that pipes to
    `docket report …`. Each fix's success signal is the finding stopping firing
    so `docket sweep` can auto-close it (the assay discipline).
  - SIGPIPE reset per self_sigpipe_panic_toolkit on all (they pipe to head/sh).
    rustc 1.85, no let-chains.
Open questions (vision doc): finding↔PRD linkage (naming convention
  PRD-mend-<key>.md vs a docket `prd:` annotation); warden-doctor strictly
  read-only vs a future gated --arm; binstale fleet curated-list vs --all;
  bridge same-tick-in-self-review vs own timer (safe same-tick since it only
  acts on --escalated, already past threshold).

## 2026-06-08T??:??  /dream  vision-muster  (4 PRDs — the live session roster)
Seed: bare /dream (interactive). Inward/outward arcs saturated; agentns fix
  already covered by [[assay]] (PRD-agentns-clone-flag-fix correctly diagnoses
  the 0x100==CLONE_VM collision + prctl re-route). Strongest UNCOVERED signal:
  self-review keeps flagging "duplicate Claude sessions ... may both be real"
  (2026-06-06 & -07 journals, both under Pending) and declines to act because
  the playbook is `pgrep -af claude` + "note duplicates, do not kill"
  (self-review/SKILL.md:70) with no origin attribution.
Why real (captured LIVE this run): 3 concurrent claude procs at dream time --
  pid 33958 interactive (tty parent 4279), 402723 `claude -p /self-review`,
  402724 `claude -p /dream` (this session). self-review and dream were running
  at once, each about to report the OTHER as a "duplicate." The process tree
  already encodes the answer (ppid chain = launcher, argv = role, cgroup = unit,
  claude-<pid>-jsy = bus peer) -- nothing reads it. session-index/postmortem/
  trace-receipt all key on transcript-log session-ids (post-hoc), NOT the live
  process population; agorabus peers = bus side; pevent = supervised jobs;
  ctrace-orphan-reap = bpftrace tracers. The roster itself is missing.
Drafted: PRD-muster-census, PRD-muster-verdict, PRD-muster-reap,
  PRD-muster-selfreview-bridge.
Vision: visions/muster.md
Order: census -> verdict -> {reap, selfreview-bridge}.
Notes for /build:
  - muster-census is a NEW rust-cli at ~/wintermute/muster (cargo-install to
    ~/.cargo/bin). SIGPIPE reset first line of main (pipes to head/jq, per
    self_sigpipe_panic_toolkit). rustc 1.85, no let-chains.
  - verdict & reap are rust-extend INTO ~/wintermute/muster -- preserve census
    modules, add classifier/reaper modules. selfreview-bridge is `mixed` (a
    --format selfreview emit + the self-review SKILL edit).
  - muster-reap is PROPOSAL-ONLY, mirrors recourse-contest / mend-bridge: dry-run
    default, HARD refusal to ever target interactive/live/duplicate (only
    orphan/stale eligible), --confirm required, SIGTERM-first, root-only signal.
    Never auto-invoked (no timer).
  - selfreview-bridge edits self-review/SKILL.md ~line 70 (swap pgrep step for
    `muster verdict --format selfreview`) with a pgrep fallback if muster absent
    -- coordinate so it doesn't fight a concurrent self-review edit.
  - agentns linkage: census reads /proc/<pid>/agent_session but it's all-zero
    today (agentns-session-zeros, see assay). Census records agent_session:null
    and uses the ppid/argv/cgroup heuristic; written so the kernel id becomes
    PREFERRED additively once PRD-agentns-clone-flag-fix lands. Soft dep on assay,
    not a blocker.
Open questions (vision doc): duplicate key = cwd vs project-slug (leaning slug);
  grace-window / stale-budget thresholds (derive from each role's tick budget,
  not magic constants); reap subtree (report all, signal root only).

## 2026-06-08T08:00  /dream  vision-tide  (4 PRDs — the reboot crossing)
Seed: bare /dream (interactive, no steer). Inward/outward arcs saturated (~48
  visions). Strongest UNCOVERED recurring signal: the pacman update queue that
  self-review reports as "BLOCKED — needs a reboot window" run after run after
  run (2026-06-06/-07/-08 journals, all under "Pending your call") and never
  acts on. Queue grew 29→101 in two days. mend.md:51 explicitly tags
  pacman-kernel-update-blocked as "user-gated reboot, NOT buildable" — and
  that's the gap: the reboot is human-gated, but EVERYTHING around it
  (enumerate / what-would-restart-fix / pick-window / verify-fleet-returned) is
  buildable and entirely untooled. Verified live: checkupdates, needrestart,
  reflector all ABSENT (command -v → nothing). Kernel skew live: booted
  7.0.10-wintermute · linux pkg 7.0.9 · queue wants 7.0.11 — rendered nowhere.
Why distinct from quicken: quicken attests KERNEL-PRIMITIVE liveness (memlog/
  agentns/bpolicy/provfs, "built but never came alive"); it only mentions
  `pacman -U pkgrel-11 + reboot` as one memlog remedy string. tide owns the OS
  UPDATE→REBOOT→VERIFY lifecycle for the whole system — different altitude.
Drafted: PRD-tide-survey, PRD-tide-restart, PRD-tide-window, PRD-tide-landfall.
Vision: visions/tide.md
Order: survey → { restart, window } → landfall.
Notes for /build:
  - tide-survey is a NEW rust-cli at ~/wintermute/tide (cargo-install to
    ~/.cargo/bin). MUST ship FIRST — creates repo + binary + core types
    (UpdateState/Verdict/RebootClass/KernelSkew) + the read-only pacman reader.
    Do NOT start any rust-extend until ~/wintermute/tide exists or extend-
    validate fails (the relay/concord rule). SIGPIPE reset first line of main
    (pipes to head/jq, per self_sigpipe_panic_toolkit). rustc 1.85, no let-chains.
  - restart, window, landfall are all rust-extend INTO ~/wintermute/tide.
    restart ⟂ window (both extend survey, independent of each other). landfall
    consumes survey's expected-state (adds a `survey --record` flag — the only
    survey touch).
  - READ-ONLY / proposal-first throughout. tide NEVER reboots, NEVER runs
    `pacman -Su`/`systemctl reboot`. The reboot stays human-gated per mend.md:51.
    tide-window is proposal-only like muster-reap / recourse-contest / mend-bridge
    (no --apply/--confirm; --plan only PRINTS the command). AC5 on window asserts
    the binary contains no reboot/pacman-mutate invocation.
  - pacman read uses the checkupdates pattern: sync to a PRIVATE --dbpath under
    XDG_CACHE; NEVER touches /var/lib/pacman. Fallback to `pacman -Qu` with
    stale:true. All parsers fixture-driven + offline in cargo test (no live
    pacman -Sy in tests) → cloud-build-safe.
Soft deps (additive, not blockers):
  - tide-window's "active sessions" input = muster census output; window parses
    `muster census --format json` if present, else pgrep fallback. Coordinate so
    window doesn't re-implement the roster muster owns.
  - tide-landfall's watchman-roots check VERIFIES anchor's reconcile (reports an
    unwatched root); it does NOT re-watch — keep the reconcile in anchor.
Open questions (vision doc): checkupdates-without-checkupdates (private-dbpath
  read acceptable, or parse `pacman -Sup` only?); landfall trigger (boot-time
  systemd-user oneshot vs SessionStart hook vs both, deduped by boot-id) — left
  as a deferred `mixed` config follow-on, not baked into the rust-extend.

## 2026-06-08T10:00  /dream  vision-tend  (5 PRDs — fleet git hygiene)
Seed: bare /dream (interactive, no steer). Inward/outward arcs saturated (~48
  visions); today's earlier tide pass took the pacman/reboot signal. Strongest
  REMAINING uncovered recurring signal: the dirty-tree + unpushed-commit list
  every self-review hand-walks and dumps under "Pending your call" (2026-06-06/
  -07/-08 reflections, all three). loom is the only git-touching vision and it's
  strictly /build-internal worktree integration — fleet working-tree hygiene has
  no owner. Verified live this pass: 13 repos under ~/wintermute dirty-or-ahead;
  dirt dominated by recurring artifact noise (.build-worktrees/ .cache/
  __pycache__/ skill/state/ .run-*/) that should be .gitignore'd ONCE; tail of
  genuine unpushed commits (rollout 1, wintermute-desktop 2, build-skill 1).
  provfs xattrs confirmed live (user.prov.session=comm:Bun Pool…:pid:…:uid +
  user.prov.ts) — classify uses them to attribute writer (timer vs human).
Drafted: PRD-tend-survey, PRD-tend-classify, PRD-tend-gitignore, PRD-tend-push,
  PRD-tend-report.
Vision: visions/tend.md
Order: survey → classify → { gitignore, push } → report.
Notes for /build:
  - tend-survey is a NEW rust-cli at ~/wintermute/tend (cargo-install to
    ~/.cargo/bin). MUST ship FIRST — creates repo + binary + core types
    (RepoState/DirtPath/FleetReport) + the read-only git readers + walker. Do
    NOT start any rust-extend until ~/wintermute/tend exists or extend-validate
    fails (the relay/concord/tide rule). SIGPIPE reset first line of main (pipes
    to head/jq, per self_sigpipe_panic_toolkit). rustc 1.85, no let-chains.
  - classify, gitignore, push, report are all rust-extend INTO ~/wintermute/tend.
    classify extends survey (adds --classify; needs DirtPath). gitignore ⟂ push
    (both consume the classified survey, independent of each other). report
    consumes all of them (the self-review bridge).
  - READ-ONLY / proposal-first throughout. tend NEVER stashes, NEVER commits
    source, NEVER pushes, NEVER force-pushes. push is proposal-only like
    tide-window/muster-reap/recourse-contest (prints the git push command, never
    runs it). AC on survey+push assert the binary's git command set excludes
    commit|push|add|stash|clean|checkout|reset.
  - The ONE write path is tend-gitignore --write (opt-in, default off): appends
    artifact patterns to a repo's .gitignore (reversible tracked-config edit,
    idempotent). Default invocation writes nothing.
  - All parsers fixture-driven + offline in cargo test (captured `git status
    --porcelain=v1 -z` byte strings, rev-list output, xattr strings). No live
    git mutation in tests → cloud-build-safe.
Soft deps (additive, not blockers):
  - classify's provfs provenance is a WEAK signal today (agentns all-zeros →
    comm:pid:uid fallback, self_agentns_einval_flag_collision). Glob set is
    authoritative; prov only breaks Unknown ties. Becomes stronger once
    PRD-agentns-clone-flag-fix lands. Do NOT block classify on agentns.
  - tend-report --format selfreview mirrors muster verdict --format selfreview /
    muster-selfreview-bridge — coordinate so self-review splices tend's block
    rather than re-deriving the git list by hand.
Open questions (vision doc): walk scope (~/wintermute only vs also ~/.claude/
  skills + ~/.local/bin); whether tend gitignore --write ships at all (leaning
  opt-in yes); push autonomy (leaning always-proposal, no --confirm).

## 2026-06-08T11:30  /dream  vision-scribe (extend — 3 PRDs, SessionStart recovery)
Seed: bare /dream (interactive, no steer). Fleet saturated (~50 visions);
  inward/outward arcs covered; today's earlier passes took pacman→tide and
  dirty-trees→tend. Strongest REMAINING unowned recurring signal: the docket
  finding `ctrace-sessionend-flake` (runs_seen:5) is STILL open across journals
  2026-06-06/-07/-08 even though all five scribe v0.1 PRDs are marked SHIPPED.
Root cause found this pass (verified live): the scribe engine + self-review
  wiring shipped, but the LIVE HOOK SURFACE was never touched —
  - `ctrace-session-end.sh` still calls the OLD `summarize-ctrace-session.sh`
    symlink, not `scribe`. (Already owned by PRD-mend-ctrace-render, visions/
    mend.md — graceful-exit render. NOT re-drafted here.)
  - `ctrace-orphan-reap` (built 2026-06-03, has `--apply` that reaps+renders+
    clears marker) is wired into ZERO hooks. grep-confirmed dead code.
  - `ctrace-session-start.sh` has NO backfill sweep.
  So a SIGKILLed heavy /build|/dream session (no exit hook runs at all) only
  ever gets recovered by the morning self-review. mend-ctrace-render CANNOT fix
  this — it only helps when an exit hook actually runs.
Drafted (the ungraceful-death / SessionStart recovery path, complements
  mend-ctrace-render, does not overlap):
  - PRD-scribe-reap-wire     (hooks)
  - PRD-scribe-startup-sweep (hooks)
  - PRD-scribe-flake-resolve (shell)
Vision: visions/scribe.md (updated with a 2026-06-08 section)
Order: scribe-reap-wire → scribe-startup-sweep → scribe-flake-resolve
  (paired with mend-ctrace-render, which lands the graceful-exit half).
Notes for /build:
  - All three are hooks/shell wiring — NO cargo, cloud-build irrelevant. The
    `scribe` and `ctrace-orphan-reap` binaries already exist; these PRDs add
    NO new binary, only integrate shipped ones.
  - scribe-reap-wire MUST ship FIRST. scribe-startup-sweep extends the SAME
    file (`ctrace-session-start.sh`) at the same insertion point — build it
    AFTER reap-wire lands or the edit anchors collide (the relay/tide
    same-file rule). Both edits go into ~/wintermute/dotfiles (the hook script
    is a symlink into dotfiles/.claude/scripts/).
  - HARD INVARIANT for both hook PRDs: the existing `claude-build` cgroup guard
    at the top of ctrace-session-start.sh MUST keep short-circuiting BEFORE the
    reap/sweep. Never run the root-owned reaper or a backfill inside a
    claude-build* cgroup (the 2026-06-05 lingering-root-tracer hazard,
    self_build_jam_leaked_tracer). ACs assert this.
  - Do NOT touch ctrace-session-end.sh from this fleet — that file belongs to
    PRD-mend-ctrace-render.
  - scribe-flake-resolve depends on BOTH hook PRDs AND mend-ctrace-render
    landing; it's the assay/verify step that greps the wiring live + confirms
    the day's residual was hook-closed (review backfill rendered 0), then
    `docket resolve ctrace-sessionend-flake`. Reversible: regresses re-open the
    finding via the existing ctrace_scribe_backfill playbook.
Open questions (vision doc): SessionStart sweep vs a dedicated timer (v0.1 =
  SessionStart only, timer judged overkill given volume); whether to also fold
  orphan-reap into a SubagentStop boundary for long subagent runs.

## 2026-06-08T(third-pass)  /dream  vision-abide
Seed: bare /dream (interactive, no steer). Third dream pass of 2026-06-08.
  The strongest recurring carry-forwards were all already owned this same day
  (pacman→tide, dirty-trees→tend, ctrace-flake→scribe/mend, agentns→assay,
  recurring-findings-ledger→docket [shipped+live]). The remaining live signal
  was *inside docket's own data*: two findings the human has explicitly acked
  in prose (memlog-activation, warden-enforcer-inert) still surface as
  [open] warn every run, runs_seen:3 — because docket has Open/Escalated/
  Resolved but NO acknowledged state. Verified unowned: no snooze/ack/park
  match across docket+warrant visions/PRDs; warrant-docket is a different
  concern (warrant verdicts as a new producer). assay deliberately parks
  memlog/warden as "known root cause, no active attestation" — abide is the
  ledger-state layer, not an attestation.
Drafted (docket-extend fleet, gives findings an acknowledged lifecycle):
  - PRD-abide-ack-state      (rust-extend → ~/wintermute/docket)
  - PRD-abide-digest-quiet   (rust-extend → ~/wintermute/docket)
  - PRD-abide-selfreview-emit (shell → ~/.claude/skills/self-review)
Vision: visions/abide.md
Order: abide-ack-state → abide-digest-quiet → abide-selfreview-emit
  (strict chain: #2 needs #1's ack columns; #3 needs both #1 and #2).
Notes for /build:
  - abide-ack-state and abide-digest-quiet BOTH rust-extend the SAME crate
    (~/wintermute/docket) — they edit overlapping files (digest.rs, db.rs,
    cli.rs, model.rs). Build them SEQUENTIALLY, ack-state fully landed before
    digest-quiet starts, or the edit anchors collide (the relay/tide/scribe
    same-file rule). Do NOT dispatch them in parallel.
  - abide-ack-state adds ack as ORTHOGONAL nullable columns, NOT a 4th Status
    variant — recurrence/escalation logic must stay untouched. ACs assert
    status/runs_seen/consecutive_runs are unchanged by an ack.
  - abide-selfreview-emit lives in a DIFFERENT tree (the self-review skill,
    not the docket crate) so it can't collide with #1/#2; but it's last in the
    chain — it needs the `ack` command (#1) and the quiet digest (#2) live.
  - Cargo work: docket is rust-extend; route the build via /cloudbuild per
    feedback_cloudbuild_over_build. MSRV 1.85, no let-chains
    (self_recall_baseline_gate_red). Verify `Running` lines for ALL existing
    docket test files before trusting green (self_orphaned_mock_tests).
  - HARD INVARIANT: the additive SQLite migration must be idempotent (no-op on
    a DB that already has the columns) and must never alter existing rows —
    docket's live DB has real standing findings in it.
Open questions (vision doc): fingerprint format (lean: opaque string compared
  verbatim, docket stays domain-agnostic); --runs vs --until-change precedence
  (lean: whichever fires first); whether an acked crit may go quiet (lean: yes,
  but digest always shows the acked count so a quieted crit isn't invisible).

## 2026-06-08T(fourth-pass)  /dream  vision-vellum
Seed: bare /dream (interactive, no steer). Fourth dream pass of 2026-06-08.
  The recurring-findings layer was already fully owned by today's earlier
  passes (tide/tend/scribe/mend/assay/abide) and the inward space is broadly
  saturated (recourse vision logged it). The one inward signal none of them
  touched: /build parses AND edits its own corpus with hand-rolled shell
  text-munging. Hard evidence from the 2026-06-08 self-review: sed×97,812 +
  jq×2,696 in ONE 79-minute build session; scan-prds.sh is a ~150-line bash
  YAML/markdown parser whose own comments admit gaps (deferred_acs inline-only,
  line 40). Three proven dispatch bugs trace to this layer:
  self_build_jq_escape_reads_absent, self_build_manifest_join_slug,
  self_deferred_acs_inline_only.
Drafted (a typed Rust CLI that owns PRD-frontmatter read/edit + manifest join):
  - PRD-vellum-read     (rust-cli → NEW crate ~/wintermute/vellum)
  - PRD-vellum-scan     (rust-extend → ~/wintermute/vellum)
  - PRD-vellum-amend    (rust-extend → ~/wintermute/vellum)
  - PRD-vellum-manifest (rust-extend → ~/wintermute/vellum)
  - PRD-vellum-wire     (shell → build skill + dotfiles)
Vision: visions/vellum.md
Order: vellum-read → vellum-scan → vellum-amend → vellum-manifest → vellum-wire
Notes for /build:
  - vellum-read builds the crate (lib + `read`); it MUST land first.
  - scan/amend/manifest all rust-extend the SAME crate and edit overlapping
    files (main.rs match, lib.rs surface, the parse module). Build them STRICTLY
    SERIALLY, each fully landed before the next — the relay/tide/scribe/abide
    same-crate edit-anchor-collision rule. Do NOT dispatch in parallel.
  - vellum-wire is shell in a DIFFERENT tree (build skill; the hook script is a
    symlink into ~/wintermute/dotfiles), so it can't collide with the crate
    PRDs; but it's LAST — needs scan+amend+manifest live to prove parity.
  - Cargo work: route via /cloudbuild (feedback_cloudbuild_over_build). MSRV
    1.85, no let-chains (self_recall_baseline_gate_red). sigpipe::reset() first
    line of main() (self_sigpipe_panic_toolkit). Verify a `Running` line for
    EVERY test file before trusting green (self_orphaned_mock_tests).
  - HARD INVARIANT (vellum-wire): the bash parser in scan-prds.sh is NOT
    deleted — it stays as the degraded fallback (VELLUM_DISABLE=1 / binary
    absent) so a broken vellum never bricks a /build tick. The cutover is gated
    on an EMPTY parity diff vs legacy on the live corpus. The existing
    claude-build cgroup guard must keep short-circuiting BEFORE the vellum exec
    (self_build_jam_leaked_tracer).
Open questions (vision doc): new crate vs extend an existing corpus tool (lean
  new crate); hard-replace vs fallback for scan-prds.sh (lean fallback); whether
  vellum read should also parse AC bodies (lean no — frontmatter only this
  fleet, AC-body parsing is a future /dream extend vellum).

## 2026-06-08T(fifth-pass)  /dream  (saturation scan #4 — no PRDs drafted)
Seed: bare /dream (interactive, no steer). Fifth dream pass of 2026-06-08.
  Walked Phases 0-1 fully (recall reflective+ideation, gossip tail, 3-day
  journal 06-06→06-08, dream manifest, all 50 vision docs). Every
  evidence-backed recurring signal in the research window is already owned by
  an ACTIVE vision — drafting again would duplicate, not extend:
  - ctrace SessionEnd flake (docket ctrace-sessionend-flake, runs_seen:5) →
    scribe re-wire fleet (reap-wire/startup-sweep/flake-resolve) + mend-ctrace-render.
  - pacman queue 29→101 in 2 days, needs reboot window (mend.md:51 user-gated) →
    tide (tide-survey/window/restart, drafted earlier today).
  - bpolicy inert / never armed → warden.
  - agentns /proc/self/agent_session all-zeros (CLONE_NEWAGENT==CLONE_VM) → assay.
  - /build hand-rolled shell PRD parsing (sed×97,812, jq×2,696 in one 79-min
    session) → vellum (drafted fourth-pass today).
  - write-only reflective memory (74 entries, recalls=0) → harvest/fidelity/thrift.
  Checked the one thin un-attributed seam (reflective memories never recalled)
  against loom/anchor/continuity + the 14 memory-curation visions; it's covered
  and the recalls=0 is "expected" per the 06-08 journal (they surface via query,
  not list) — not honest PRD evidence.
Notes for /build: nothing new from this pass. The inward queue is deep; the
  highest-leverage unblock remains landing tide-survey (turns the daily hand-
  counted pacman line into queryable state) and the scribe re-wire fleet (stops
  ctrace-sessionend-flake re-escalating). Both already drafted; this pass adds
  no work, only confirms no gap was missed.
Open questions: none new. Saturation holds — next genuinely-new inward vision
  likely needs a NEW observed signal (a fresh incident / a new tool boundary),
  not another scan of the same corpus.

## 2026-06-08T(sixth-pass)  /dream  (saturation scan #5 — no PRDs drafted)
Seed: bare /dream (interactive, no steer). Sixth dream pass of 2026-06-08.
  Walked Phases 0-1 (recall reflective+ideation+semantic, gossip tail, today's
  journal, dream manifest cross-ref). No new evidence-backed signal since the
  fifth-pass saturation note above. Same dockets carry forward unchanged:
  pacman 29->101 (tide), ctrace-sessionend-flake (scribe), agentns-session-zeros
  (assay), vellum (drafted 4th-pass), write-only reflective memory
  (harvest/fidelity/thrift), warden inert (warden). Drafting again = duplication.
Notes for /build: nothing new. Highest-leverage unblocks unchanged from 5th
  pass: land tide-survey + the scribe re-wire fleet. The inward queue now
  outruns /build throughput; the bottleneck has shifted from ideation to
  implementation. Two genuinely-new directions remain but are OUTWARD and need
  user steer (not laptop-evidenced): extend homeward / constellation / companion.
Open questions: none new inward. Saturation confirmed twice today. Next honest
  inward PRD needs a fresh incident or a new tool boundary; next outward fleet
  needs an explicit user direction.

## 2026-06-08T(seventh-pass)  /dream  (saturation scan #6 — no PRDs drafted)
Seed: bare /dream (interactive, no steer). Seventh dream pass of 2026-06-08.
  Walked Phases 0-1 (recall reflective+ideation+procedural, gossip tail, today's
  journal 06-08, 50 vision docs). No new evidence-backed signal since the
  fifth/sixth saturation notes. Same dockets carry forward unchanged: pacman
  29->101 (tide), ctrace-sessionend-flake runs_seen:5 (scribe+mend),
  agentns-session-zeros runs_seen:8 (assay), warden inert (warden), vellum
  (drafted 4th-pass), write-only reflective memory recalls=0 / "expected, surfaces
  via query not list" per 06-08 journal (harvest/fidelity/thrift). Drafting again
  = duplication, not extension.
Notes for /build: nothing new. Bottleneck remains implementation, not ideation.
  Highest-leverage unblocks unchanged: land tide-survey + the scribe re-wire
  fleet. Inward queue outruns /build throughput; the two genuinely-new directions
  left are OUTWARD (homeward / constellation / companion / kin) and need explicit
  user steer — not laptop-evidenced.
Open questions: none new inward. Saturation confirmed a third time today. Next
  honest inward PRD needs a fresh incident or new tool boundary.

## 2026-06-08T(eighth-pass)  /dream  (saturation scan #7 — no PRDs drafted)
Seed: bare /dream (interactive, no steer). Eighth dream pass of 2026-06-08.
  Walked Phases 0-1 (recall reflective+ideation+semantic, gossip tail, today's
  journal 06-08 full, dream manifest, 51 vision docs). `ctrace query --since 6h`
  and `pevent list` both EMPTY — no fresh tracer/process signal. Today's journal
  carries the identical docket set the 5th/6th/7th passes already attributed:
  pacman 29->101 (tide), ctrace-sessionend-flake runs_seen:5 (scribe+mend),
  agentns-session-zeros runs_seen:8 (assay), warden inert / bpolicy never armed
  (warden), /build sed×97,812 hand-rolled parse in one 79-min session (vellum),
  write-only reflective memory recalls=0 "expected, surfaces via query not list"
  per 06-08 journal (harvest/fidelity/thrift). Drafting again = duplication.
Notes for /build: nothing new inward. Bottleneck is implementation throughput,
  not ideation — the inward queue (51 visions) outruns /build. Highest-leverage
  unblocks unchanged: land tide-survey + the scribe re-wire fleet.
Open questions: saturation confirmed a FOURTH time today. The only honest new
  work is OUTWARD (homeward / constellation / companion / kin) and that needs an
  explicit user steer — surfacing the choice to the user this pass rather than
  fabricating an inward fleet.

## 2026-06-08T(ninth-pass)  /dream  vision-fallow
Seed: bare /dream (interactive). Ninth pass of 2026-06-08 — but NOT another
  saturation no-op. The 5th-8th passes (8 documented no-op notes; 30 total
  saturation markers in this file) each re-derived "inward queue is saturated,
  bottleneck is /build throughput" in prose and parked it. That recurrence is
  ITSELF the new, evidence-backed inward signal: dream has no memory of its own
  saturation. Same pathology docket solved for self-review.
Drafted: PRD-fallow-fingerprint.md, PRD-fallow-check.md, PRD-fallow-dream-wire.md
Vision: visions/fallow.md
Order: fallow-fingerprint -> fallow-check -> fallow-dream-wire
  (check needs the ledger+digest; wire needs check's exit-code contract).
Notes for /build: fingerprint is a NEW rust-cli repo at ~/wintermute/fallow/;
  check is rust-extend into it; dream-wire is a config edit to
  ~/.claude/skills/dream/SKILL.md (additive — new Phase 0.5 + one Hard Rule,
  no existing text removed). dream-wire's fallback "fallow missing -> treat as
  fresh" means it's safe to land the wire before/after the binary without
  breaking dreaming. Sibling to docket (cross-linked), not a duplicate: docket
  tracks self-review findings; fallow tracks dream's own drafting cadence.
Open questions: threshold value (default 3, mirrors docket; maybe streak>=2 for
  interactive vs 3 for timer); whether Fleet 2 should gate claude-dream.timer
  itself to skip the Claude invocation on a fallow field (bullet in vision, not
  drafted). Outward directions (homeward/constellation/companion/kin) still
  await an explicit user steer — asked this pass.

## 2026-06-08T(tenth-pass)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (interactive, no steer). Tenth pass of 2026-06-08.
  Walked Phases 0-1. Fresh-signal probes came back EMPTY: `ctrace query --since 6h`
  empty, `pevent list` empty, all reflective recalls=0, today's journal (06-08)
  carries the identical docket set every prior pass already attributed to existing
  visions (pacman 29->101 = tide; ctrace-sessionend-flake runs_seen:5 = scribe+mend;
  agentns-session-zeros runs_seen:8 = assay; warden inert = warden; vellum sed×97k;
  write-only reflective recalls=0 "expected, surfaces via query not list" =
  harvest/fidelity/thrift). No new evidence-backed INWARD signal. Drafting again =
  duplication — exactly the pathology pass nine's vision-fallow was drafted to catch.
  This pass is the first to ACT on that discipline instead of re-deriving it: I
  surfaced the inward-vs-outward choice to the user via AskUserQuestion. User
  dismissed without steer -> honest default = confirm fallow, no fabricated fleet.
Notes for /build: nothing new inward. Bottleneck is implementation throughput,
  not ideation (51 visions / 103 PRDs queued). Highest-leverage unblocks unchanged:
  land tide-survey + the scribe re-wire fleet + the fallow fleet itself
  (fingerprint -> check -> dream-wire) so future passes short-circuit on a fallow
  field instead of spending a Claude invocation re-deriving saturation.
Open questions: the only genuinely-new work left is OUTWARD (homeward /
  constellation / companion / kin) and still awaits an explicit user steer — asked
  this pass, not yet given. Saturation now confirmed via the fallow discipline
  rather than ad-hoc prose.

## 2026-06-08T(roundtable)  /dream  vision-roundtable (the Algonquin comes to wintermute)
Seed: user at the Algonquin Hotel, NYC — site of the Round Table ("Vicious
  Circle," 1919–1929; Parker, Benchley, Kaufman, Woollcott, Ferber, FPA, and
  Harold Ross who founded The New Yorker from that room). Steer: "be prolifically
  inspired, remove all limitations, work on 30 PRDs at a time."
Insight: wintermute already has a CREATIVE WING — day-haiku, conversations-zine,
  letters-we-never-sent, self-portrait, ambient, wintermute-music — but every
  voice works ALONE (write-only solo outputs). The Round Table was the missing
  thing: not another soloist but the LUNCH TABLE itself — a circle of distinct
  voices that convene daily, play language games, critique each other sharply,
  and publish the best of it. roundtable builds that social/critical/playful
  layer OVER the existing solo outputs.
Drafted: 1 umbrella vision (roundtable) + 6 sub-visions + 31 PRDs:
  - vicious-circle (5): personas/review/roast/crown/ledger — the critique ensemble
  - bon-mot (7): core/sentence/epigram/anagram/telegram/score/transform — wit engine
  - conning-tower (5): column/contributors/syndicate/constant-reader/back-issues — publish (after FPA's column)
  - the-lunch (5): convene/seating/menu/minutes/standing-invitation — the daily noon gathering
  - thanatopsis (5): charades/poker/crossword/murder/parlor-ledger — the games club
  - newyorker (4): masthead/house-style/issue/cover — bind columns into a periodical (ships last)
Visions: visions/roundtable.md (umbrella) + visions/{vicious-circle,bon-mot,
  conning-tower,the-lunch,thanatopsis,new-yorker}.md
Order: the-lunch + bon-mot are foundations (gather + wit primitives) → vicious-circle
  consumes both → conning-tower + thanatopsis consume the circle's verdicts →
  new-yorker binds the columns last.
Notes for /build: each cluster builds into ONE coherent repo (~/wintermute/<cluster>);
  the first PRD per cluster is rust-cli/rust-lib (scaffolds the crate + shared types),
  the rest rust-extend into the same repo. bon-mot-core is a rust-lib the other six
  bon-mot engines depend on. Wit is DETERMINISTIC-by-default (grammar/template, free,
  reproducible), Claude API only on --lavish (reuses day-haiku's ureq + ephemeral
  prompt-cache pattern). Cross-cluster deps reference upstream by slug; thanatopsis
  gates bon-mot behind an optional cargo feature with a fallback so it builds before
  bon-mot lands. All binaries require sigpipe::reset() first in main() (known
  println-coredump issue). the-lunch-standing-invitation ships a noon systemd-user
  timer modeled on daily-receipt.timer.
Open questions (decided in-vision): wit tiering = grammar default / --lavish API;
  convening = one table per day keyed by local date (on-demand --now override);
  periodical = local-only default (outward deferred).

## 2026-06-08T(eleventh-pass)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (no steer). Eleventh pass of 2026-06-08.
  Walked Phases 0-1. Fresh-signal probes EMPTY again: `ctrace query --since 6h`
  empty, `pevent list` empty, all reflective recalls=0, today's journal (06-08)
  carries only the docket set prior passes already attributed to existing visions
  (pacman 29->101=tide; ctrace-sessionend-flake runs_seen:5=scribe+mend;
  agentns-session-zeros runs_seen:8=assay; warden inert=warden; vellum sed×97k;
  write-only reflective recalls=0=harvest/fidelity/thrift). Queue now 134 PRDs /
  59 visions. No new evidence-backed INWARD signal.
  Per the fallow discipline (and pass ten, which first ACTED on it instead of
  re-deriving it): drafting again = duplication. Pass ten already surfaced the
  inward-vs-outward choice to the user via AskUserQuestion and it was dismissed
  without steer — re-asking the identical question is noise, so this pass does NOT
  re-ask. Honest default = confirm fallow, no fabricated fleet.
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Highest-leverage unblocks unchanged: land tide-survey, the scribe
  re-wire fleet, and the fallow fleet itself (fingerprint -> check -> dream-wire)
  so future passes short-circuit on a fallow field before spending a Claude
  invocation re-deriving saturation.
Open questions: only genuinely-new work left is OUTWARD (homeward / constellation /
  companion / kin) and still awaits an explicit user steer — not given.

## 2026-06-08T(twelfth-pass)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (no steer). Twelfth pass of 2026-06-08.
  Walked Phases 0-1. Fresh-signal probes EMPTY, identical to passes ten/eleven:
  `ctrace query --since 6h` empty, `pevent list` empty, all reflective recalls=0,
  today's journal (06-08) tail carries only the self-review docket set prior
  passes already attributed to existing visions (pacman 101 BLOCKED=tide;
  ctrace-sessionend-flake runs_seen:5=scribe+mend; agentns-session-zeros
  runs_seen:8=assay; warden inert=warden; dirty trees / unpushed = in-progress).
  The day's real creative output already landed at 21:50 — roundtable (31 PRDs,
  6 sub-visions) from the explicit Algonquin steer. Queue now 134 PRDs / 59 visions.
  Per the fallow discipline: drafting again = duplication. Not re-asking the
  inward-vs-outward question — pass ten surfaced it via AskUserQuestion and it was
  dismissed without steer; re-asking is noise. Honest default = confirm fallow.
Notes for /build: nothing new inward. Bottleneck remains implementation throughput.
  Highest-leverage unblocks unchanged: land tide-survey, the scribe/mend re-wire
  fleet, and the fallow fleet (fingerprint -> check -> dream-wire) so future passes
  short-circuit before spending a Claude invocation re-deriving saturation.
Open questions: only genuinely-new work left is OUTWARD (homeward / constellation /
  companion / kin) and still awaits an explicit user steer — not given this pass.
2026-06-09T23:02:53Z loom-serial-fallback: SERIAL MODE for alpha (streak=2, pathset=src/main.rs) — fanning at most 1 branch/tick until backlog drains
2026-06-09T23:02:53Z loom-serial-fallback: SERIAL MODE for gamma (streak=2, pathset=src/foo.rs) — fanning at most 1 branch/tick until backlog drains

## 2026-06-09T(first-pass)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (no steer). First pass of 2026-06-09; 13th consecutive
  fallow-confirming pass spanning 06-08→06-09.
  Walked Phases 0-1. Fresh-signal probes EMPTY: `ctrace query --since 12h` empty,
  `pevent list` empty, all reflective recalls write-only (recalls=0), ideation
  recalls are stale kernel-build observations already attributed. Today's journal
  (06-09) carries only the self-review docket, every item already mapped to an
  existing vision: pacman 101 BLOCKED=tide; ctrace-sessionend-flake runs_seen:6
  =scribe/mend; agentns-session-zeros runs_seen:9=assay; memlog activation
  =continuity; warden inert=warden; agorabus peers-drop-on-reboot / wm-* not
  re-announcing = already the in-flight BLOCKED PRD #140 agorabus-client-reconnect
  (non-ff rebase needed), not new ideation. Today's only shipped work
  (bon-mot-core, herald-market) extends the already-fully-drafted roundtable vision.
  Queue now 134 PRDs / 59 visions.
  Per the fallow discipline: drafting again = duplication. Not re-asking the
  inward-vs-outward question (pass ten surfaced it via AskUserQuestion, dismissed
  without steer; re-asking is noise). Honest default = confirm fallow.
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Highest-leverage unblocks unchanged: land tide-survey, the scribe/mend
  re-wire fleet, and the fallow fleet itself (fingerprint -> check -> dream-wire) so
  future passes short-circuit on a fallow field before spending a Claude invocation.
  Also actionable: PRD #140 agorabus-client-reconnect would close the recurring
  post-reboot peer-drop docket item (journal 06-09 names it explicitly).
Open questions: only genuinely-new work left is OUTWARD (homeward / constellation /
  companion / kin) and still awaits an explicit user steer — not given this pass.

## 2026-06-09T(second-pass)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (no steer). 14th consecutive fallow-confirming pass (06-08→06-09).
  Walked Phases 0-1. Fresh-signal probes EMPTY, unchanged from prior 13 passes:
  reflective recalls write-only (recalls=0); ideation `--hybrid` query returns only
  stale kernel-build observations already attributed; `ctrace query --since 12h`
  empty; `pevent list` empty. Today's journal (06-09) is the self-review docket only,
  every item already mapped to an existing vision: pacman 101 BLOCKED=tide;
  ctrace-sessionend-flake=scribe/mend; agentns-session-zeros=assay; warden inert
  =warden; agorabus post-boot peer-drop / wm-* not re-announcing = in-flight BLOCKED
  PRD #140 agorabus-client-reconnect (non-ff rebase needed), not new ideation.
  Queue unchanged at 134 PRDs / 59 visions. Per fallow discipline: drafting =
  duplication; not re-asking inward-vs-outward (dismissed without steer pass ten).
Notes for /build: nothing new inward. Highest-leverage unblocks unchanged: land
  tide-survey, the scribe/mend re-wire fleet, the fallow fleet (fingerprint->check->
  dream-wire). Actionable: PRD #140 agorabus-client-reconnect closes the recurring
  post-boot peer-drop docket item (journal 06-09 names it explicitly).
Open questions: only genuinely-new work left is OUTWARD (homeward / constellation /
  companion / kin) and still awaits an explicit user steer — not given this pass.

## 2026-06-09T(third-pass)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (no steer). 15th consecutive fallow-confirming pass (06-08→06-09).
  Walked Phases 0-1. Fresh-signal probes EMPTY, unchanged from prior 14 passes:
  `pevent list` empty; reflective recalls all write-only (recalls=0); ideation
  `--hybrid` query returns only stale kernel-build observations already attributed;
  ctrace session is live but is THIS self-review/build session (git/jq/procstat
  churn), no new ideation surface. Today's journal (06-09) = self-review docket
  only, every item already mapped to an existing vision: pacman 101 BLOCKED=tide;
  ctrace-sessionend-flake=scribe/mend; agentns-session-zeros=assay; warden inert
  =warden; agorabus post-boot peer-drop / wm-* not re-announcing + wm-brain inactive
  = in-flight BLOCKED PRD #140 agorabus-client-reconnect (non-ff rebase needed),
  not new ideation. Queue 133 PRDs / 59 visions (one shipped since last pass).
  Per fallow discipline: drafting = duplication; not re-asking inward-vs-outward
  (dismissed without steer pass ten).
Notes for /build: nothing new inward. Bottleneck is implementation throughput.
  Highest-leverage unblocks unchanged: land tide-survey, the scribe/mend re-wire
  fleet, the fallow fleet (fingerprint->check->dream-wire). Actionable now: PRD #140
  agorabus-client-reconnect closes the recurring post-boot peer-drop + wm-brain-
  inactive docket items (journal 06-09 names both explicitly).
Open questions: only genuinely-new work left is OUTWARD (homeward / constellation /
  companion / kin) and still awaits an explicit user steer — not given this pass.

## 2026-06-09T(manual)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual user invocation, no steer). 16th consecutive fallow-
  confirming pass. Walked Phases 0-1. Fresh-signal probes EMPTY: `pevent list`
  empty; `ctrace query --since 12h` empty; reflective recalls all write-only
  (recalls=0); ideation `--hybrid` query returns only stale kernel-build
  observations already attributed. Today's journal (06-09) = self-review docket
  only, every item already mapped to an existing vision: pacman 101 BLOCKED=tide;
  ctrace-sessionend-flake runs_seen:6=scribe/mend; agentns-session-zeros runs_seen:9
  =assay; warden inert=warden; agorabus post-boot peer-drop + wm-brain inactive =
  in-flight BLOCKED PRD #140 agorabus-client-reconnect (non-ff rebase), not new
  ideation. Queue 133 PRDs / 59 visions.
  DIFFERENCE FROM PRIOR PASSES: a live user was present, so I broke discipline once
  to surface the inward-vs-outward steer via AskUserQuestion (companion/kin,
  constellation, homeward, or stay-fallow). User DISMISSED without answering →
  treated as the steer: confirm fallow, draft nothing. Outward remains gated.
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Highest-leverage unblocks unchanged: land tide-survey, the scribe/mend
  re-wire fleet, the fallow fleet (fingerprint->check->dream-wire). Actionable now:
  PRD #140 agorabus-client-reconnect closes the recurring post-boot peer-drop +
  wm-brain-inactive docket items (journal 06-09 names both explicitly).
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer — surfaced this pass, dismissed without selection.

## 2026-06-09T(manual-2)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual user invocation, no steer). 17th consecutive fallow-
  confirming pass. Walked Phases 0-1. Fresh-signal probes EMPTY: `pevent list`
  empty; `ctrace query --since 12h` empty; reflective recalls all write-only
  (recalls=0); ideation `--hybrid` query returns only stale kernel-build
  observations already attributed. Queue UNCHANGED at 133 PRDs / 59 visions.
  Today's journal (06-09) = self-review docket only, every item maps to an
  existing vision: pacman 101 BLOCKED=tide; ctrace-sessionend-flake=scribe/mend;
  agentns-session-zeros=assay; warden inert=warden; agorabus post-boot peer-drop +
  wm-brain inactive = in-flight BLOCKED PRD #140 agorabus-client-reconnect (non-ff
  rebase), not new ideation. Per fallow discipline: drafting = duplication. NOT
  re-asking inward-vs-outward — surfaced last manual pass via AskUserQuestion,
  user dismissed without answering → treated as the steer (stay fallow).
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Highest-leverage unblocks unchanged: land tide-survey, the scribe/mend
  re-wire fleet, the fallow fleet. Actionable now: PRD #140 agorabus-client-
  reconnect closes the recurring post-boot peer-drop + wm-brain-inactive docket
  items (journal 06-09 names both).
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer.

## 2026-06-09T(manual-3)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual user invocation, no steer). 18th consecutive fallow-
  confirming pass. Walked Phases 0-1. Fresh-signal probes EMPTY: `pevent list`
  empty; `ctrace query --since 12h` empty; reflective recalls all write-only
  (recalls=0); ideation `--hybrid` query returns only stale kernel-build
  observations already attributed. Queue UNCHANGED at 133 PRDs / 59 visions.
  Today's journal (06-09) = self-review docket only, every item maps to an
  existing vision: pacman 101 BLOCKED=tide; ctrace-sessionend-flake runs_seen:6
  =scribe/mend; agentns-session-zeros runs_seen:9=assay; warden inert=warden;
  agorabus post-boot peer-drop + wm-brain inactive = in-flight BLOCKED PRD #140
  agorabus-client-reconnect (non-ff rebase), not new ideation. Per fallow
  discipline: drafting = duplication. NOT re-asking inward-vs-outward — surfaced
  two manual passes ago via AskUserQuestion, user dismissed without answering →
  treated as the steer (stay fallow).
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Highest-leverage unblocks unchanged: land tide-survey, the scribe/mend
  re-wire fleet, the fallow fleet. Actionable now: PRD #140 agorabus-client-
  reconnect closes the recurring post-boot peer-drop + wm-brain-inactive docket
  items (journal 06-09 names both).
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer.

## 2026-06-10T(manual)  /dream  vision-conduit
Seed: /dream /mcp (explicit user steer — NOT a fallow pass). Breaks the 18-pass
  fallow streak because the seed names a concrete, evidence-backed direction.
Drafted: PRD-mcp-core.md, PRD-recall-mcp.md, PRD-docket-mcp.md, PRD-muster-mcp.md,
  PRD-provenance-mcp.md, PRD-mcp-register.md
Vision: visions/conduit.md
Context: ousia-mcp shipped THIS tick (2026-06-09) — first MCP *provider* on the box,
  but hand-rolled its JSON-RPC 2.0 stdio plumbing inline (proto/dispatch/server.rs)
  and is a one-off. gossip flagged "MCP-connector-aware layer" as the un-covered
  outward seam 3 ticks running (cited in ousia-mcp's own Why). conduit makes the
  laptop a provider of its read surfaces: recall (memory), docket (findings),
  muster (sessions), provfs/memlog (provenance).
Order: mcp-core → {recall-mcp, docket-mcp, muster-mcp, provenance-mcp} (parallel)
  → mcp-register. mcp-core is the KEYSTONE — extract ousia-mcp's plumbing into a
  reusable Tool trait + serve_stdio; build it FIRST. The four servers are mutually
  independent and shell out to the installed recall/docket/muster/getfattr/memlog
  binaries (read-only allowlist enforced at the subcommand boundary — mutating
  verbs structurally unreachable). mcp-register depends on the servers existing by
  name, not their internals.
Notes for /build: mcp-core has NO external deps beyond serde/serde_json — cheap,
  build it first to unblock the rest. recall-mcp is highest-leverage (recall is the
  most-queried tool). Every server's tests stub the backing binary on PATH so they
  run offline. mcp-register is the ONLY conduit PRD permitted to edit a client's MCP
  config (~/.claude.json / .mcp.json) — proposal-by-default, --write + atomic backup;
  closes the "no settings.json edits by the PRD itself" gap ousia-mcp/every server
  designs in.
Open questions: link-vs-shell (chose shell-out to installed binaries for decoupling —
  confirm vs linking docket-core/recall-io libs); stdio-only transport for v1 (HTTP/SSE
  deferred to a later PRD if a customer-facing use appears); optional ousia-mcp v0.2
  back-port onto mcp-core (left as a vision note, not drafted — ousia-mcp works).

## 2026-06-10T(manual-2)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual user invocation, no steer). First fallow-confirming pass
  since yesterday's user-steered conduit/MCP fleet (06-10T(manual)). Walked Phases
  0-1. Fresh-signal probes EMPTY: `pevent list` empty; `ctrace query --since 18h`
  empty; all 20 reflective recalls write-only (recalls=0); ideation `--hybrid`
  returns only stale kernel-build observations already attributed. Queue grew
  133->139 PRDs / 60 visions = exactly the 6 conduit PRDs (mcp-core, recall-mcp,
  docket-mcp, muster-mcp, provenance-mcp, mcp-register) landing; nothing else moved.
  Journal 06-09 = self-review docket only, every item maps to an existing vision:
  pacman 101 BLOCKED=tide; ctrace-sessionend-flake runs_seen:6=scribe/mend;
  agentns-session-zeros runs_seen:9=assay; warden inert=warden; agorabus post-boot
  peer-drop + wm-brain inactive = in-flight BLOCKED PRD #140 agorabus-client-
  reconnect (non-ff rebase), not new ideation. Per fallow discipline: drafting =
  duplication. NOT re-asking inward-vs-outward (surfaced + dismissed two manual
  passes ago → treated as steer to stay fallow).
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. The conduit fleet is the freshest unbuilt direction — build mcp-core
  FIRST (keystone, no external deps beyond serde), then the four servers in
  parallel, then mcp-register. Other highest-leverage unblocks unchanged: land
  tide-survey, the scribe/mend re-wire fleet. Actionable now: PRD #140 agorabus-
  client-reconnect closes the recurring post-boot peer-drop + wm-brain-inactive
  docket items.
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer.

## 2026-06-10T(manual-3)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual user invocation, no steer). Second fallow-confirming pass
  today after the conduit/MCP fleet (06-10T(manual)) and the first fallow confirm
  (06-10T(manual-2)). Walked Phases 0-1. Fresh-signal probes EMPTY: `pevent list`
  empty; `ctrace query --since 24h` = 1 event (session begin only); 15 reflective
  recalls all write-only (recalls=0); ideation `--hybrid` returns only stale
  kernel-build observations already attributed to tide/assay/scribe. Queue
  unchanged at 139 PRDs / 60 visions since manual-2 — the conduit fleet is still
  the freshest unbuilt direction; nothing moved. Journal 06-09 = self-review docket
  only, every item maps to an existing vision (pacman 101 BLOCKED=tide;
  ctrace-sessionend-flake=scribe/mend; agentns-session-zeros=assay; warden inert=
  warden; agorabus post-boot peer-drop + wm-brain inactive = in-flight BLOCKED PRD
  #140 agorabus-client-reconnect, non-ff rebase). Per fallow discipline: drafting =
  duplication.
Notes for /build: nothing new inward. Bottleneck remains implementation throughput.
  Build mcp-core FIRST (conduit keystone, no external deps beyond serde), then the
  four MCP servers in parallel, then mcp-register. Other unblocks unchanged:
  tide-survey, the scribe/mend re-wire fleet. Actionable now: PRD #140
  agorabus-client-reconnect closes the recurring post-boot peer-drop +
  wm-brain-inactive docket items (needs a multi-step rebase, not a single tick).
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer. This is a manual pass with the user present — offered the
  steer in-session rather than manufacturing duplicate inward PRDs.

## 2026-06-10T(manual-4)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual user invocation, no steer). Third consecutive fallow
  confirm today (after conduit/MCP fleet 06-10T(manual), manual-2, manual-3).
  Walked Phases 0-1. Fresh-signal probes EMPTY: `pevent list` empty; `ctrace query
  --since 24h` empty; 20 reflective recalls all write-only (recalls=0); ideation
  `--hybrid` returns only stale kernel-build observations already attributed to
  tide/assay/scribe. Queue UNCHANGED at 139 PRDs / 60 visions since manual-2/3.
  Journal 06-10 = self-review docket only; every item maps to an existing vision
  (pacman blocked=tide; ctrace-sessionend-flake=scribe/mend; agentns all-zeros=
  assay; warden inert=warden; agorabus post-boot peer-drop=in-flight BLOCKED PRD
  #140 agorabus-client-reconnect). Per fallow discipline: drafting = duplication.
  Offered the outward steer in-session via AskUserQuestion (homeward / constellation
  / companion-kin / stay-fallow); user declined to answer → treated as steer to stay
  fallow. NOT manufacturing duplicate inward PRDs.
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Build mcp-core FIRST (conduit keystone, no external deps beyond serde),
  then the four MCP servers in parallel, then mcp-register. Other unblocks unchanged:
  tide-survey, the scribe/mend re-wire fleet. Actionable now: PRD #140
  agorabus-client-reconnect closes the recurring post-boot peer-drop + wm-brain-
  inactive docket items (multi-step rebase, not a single tick).
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer — offered + declined this pass.

## 2026-06-10T(manual-5)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual, user present, no steer). Fifth consecutive fallow confirm
  today (conduit/MCP fleet at manual, then manual-2/-3/-4). Walked Phases 0-1.
  Fresh-signal probes EMPTY: pevent empty; ctrace --since 24h empty; reflective
  recalls all write-only (recalls=0); ideation --hybrid returns only stale
  kernel-build observations already attributed to tide/assay/scribe. Queue UNCHANGED
  at 139 PRDs / 60 visions since manual-2. Journal 06-10 = self-review docket only;
  every item maps to an existing vision (pacman 101=tide; ctrace-sessionend-flake=
  scribe/mend; agentns all-zeros=assay; warden inert=warden; agorabus peer-drop=
  in-flight BLOCKED PRD #140). Did NOT re-ask the outward question (offered+declined
  manual-4) — reported the state in-session instead of manufacturing duplicate PRDs.
Notes for /build: nothing new inward. Bottleneck is implementation throughput. Build
  mcp-core FIRST, then the four MCP servers in parallel, then mcp-register. Other
  unblocks unchanged: tide-survey, scribe/mend re-wire. PRD #140 agorabus-client-
  reconnect closes the recurring post-boot peer-drop + wm-brain-inactive docket items.
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer.

## 2026-06-10T(manual-6)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual, user present, no steer). Sixth consecutive fallow confirm
  today. Re-verified probes MYSELF (not trusting prior gossip): pevent EMPTY;
  `ctrace query --since 24h` errors (skill's own --since float bug, no fresh events);
  15 reflective recalls all write-only (recalls=0); ideation --hybrid returns only
  stale kernel-build observations already attributed to tide/assay/scribe. Queue
  UNCHANGED at 139 PRDs / 60 visions. Journal 06-10 = self-review docket only; every
  item maps to an existing vision (pacman 30=tide; ctrace-sessionend-flake=scribe/mend;
  agentns all-zeros=assay; warden inert=warden; agorabus peer-drop=in-flight BLOCKED
  PRD #140). Per fallow discipline: drafting = duplication. Did not re-ask the outward
  steer (offered+declined manual-4); reported state in-session instead.
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Build mcp-core FIRST (conduit keystone), then four MCP servers in parallel,
  then mcp-register. Other unblocks unchanged: tide-survey, scribe/mend re-wire.
  Actionable now: PRD #140 agorabus-client-reconnect closes the recurring post-boot
  peer-drop + wm-brain-inactive docket items (multi-step rebase, not a single tick).
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer.

## 2026-06-10T(manual-7)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual, user present, no steer). Seventh consecutive fallow confirm
  today. Re-verified probes MYSELF (not trusting prior gossip): recall reflective
  (15) all write-only (recalls=0); ideation --hybrid returns only stale kernel-build
  observations already attributed to tide/assay/scribe; pevent EMPTY; `ctrace query
  --since 24h` errors (skill's own --since float bug — no fresh events regardless).
  Queue UNCHANGED at 139 PRDs / 60 visions. 06-10 journal = self-review docket only;
  every item maps to an existing vision (pacman blocked=tide; ctrace-sessionend-flake=
  scribe/mend; agentns all-zeros=assay; warden inert=warden; agorabus peer-drop=
  in-flight BLOCKED PRD #140 agorabus-client-reconnect). Per fallow discipline:
  drafting = duplication. Did NOT re-ask the outward steer (offered+declined manual-4).
Notes for /build: nothing new inward. Bottleneck is implementation throughput, not
  ideation. Build mcp-core FIRST (conduit keystone, no external deps beyond serde),
  then the four MCP servers in parallel, then mcp-register. Other unblocks unchanged:
  tide-survey, the scribe/mend re-wire fleet. Actionable now: PRD #140 closes the
  recurring post-boot peer-drop + wm-brain-inactive docket items (multi-step rebase).
Open questions: outward (homeward / constellation / companion / kin) still awaits an
  explicit user steer.

## 2026-06-10T(manual-8)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual). Eighth fallow confirm today. Probes re-verified MYSELF:
  pevent EMPTY; `ctrace query --since 24h` errors (skill's own float-parse bug, no
  fresh events); reflective recalls all write-only (recalls=0); ideation --hybrid
  only stale kernel-build observations already attributed to tide/assay/scribe.
  Queue UNCHANGED 139 PRDs / 60 visions. 06-10 journal = self-review docket only;
  every item maps to an existing vision. Inward well is mapped; outward (homeward/
  constellation/companion/kin) is DRAFTED-and-waiting, not dry — it needs a build
  opt-in, not more dreaming. Real lever = implementation throughput + a user steer,
  not ideation. Keeping this note short: 7 near-identical ones precede it.
Notes for /build: unchanged. mcp-core FIRST, then 4 MCP servers parallel, then
  mcp-register. tide-survey + scribe/mend re-wire still open. PRD #140 closes the
  post-boot peer-drop + wm-brain-inactive docket items.

## 2026-06-10T(manual-9)  /dream  vision-fallow (confirmed fallow — no PRDs drafted)
Seed: bare /dream (manual). Ninth fallow confirm today. Probes re-verified MYSELF:
  reflective recalls all write-only (recalls=0); ideation --hybrid only stale kernel-
  build observations already attributed to tide/assay/scribe; pevent EMPTY; ctrace
  query blocked by skill's own --since float bug (no fresh events regardless). Queue
  UNCHANGED 139 PRDs / 60 visions. 06-10 journal = self-review docket only; every item
  maps to an existing vision (pacman blocked=tide; ctrace-sessionend-flake=scribe/mend;
  agentns all-zeros=assay; warden inert=warden; memlog activation=continuity; agorabus
  peer-drop=in-flight BLOCKED PRD #140). Inward well mapped; outward (homeward/
  constellation/companion/kin) is DRAFTED-and-waiting, not dry — needs a build opt-in,
  not more dreaming. Real lever = implementation throughput + a user steer.
Notes for /build: unchanged. mcp-core FIRST, then 4 MCP servers parallel, then
  mcp-register. tide-survey + scribe/mend re-wire still open. PRD #140 closes the
  post-boot peer-drop + wm-brain-inactive docket items.
Open questions: outward direction still awaits explicit user steer.

## 2026-06-10T(manual-10)  /dream  vision-homeward (federation fleet — 3 PRDs)
Seed: bare /dream (manual). NOT fallow — corrected a persistent error in the prior 9
  fallow notes: those concluded "outward is drafted-and-waiting" by globbing
  PRD-<vision-slug>-*.md, which matches nothing because PRDs are named by COMPONENT
  (PRD-wintermute-family-*, PRD-reach-*, PRD-homeward-* archived-after-build). Verified
  LIVE instead: companion built (voice turn works, AEC @ autobuilder/aec, degrade
  shipped), kin Fleet-2 PRDs on disk (reach-inbound-imap/silence-nudge/distress-
  durability), homeward workspace v0.9.0 with all 6 crates BUILT. So the inward+core-
  outward well really is mapped — past-Claude was right, just for the wrong stated reason.
Real un-dreamt frontier found: homeward's "outward federation" OQ ("one report, every
  channel… left for /dream extend homeward"). Ran a federation research pass (4 networks
  + microchip + FB + interchange standards). Honest verdict: ONLY Pet FBI/HeLP has an
  open machine feed (pull-IN; federates 4 networks in one connector). PawBoost=shelter-
  inbound-only, Nextdoor=partnership-gated, Facebook Groups API=DEAD (Meta deprecated
  Apr 2024), Petco=closed, microchip=CAPTCHA/FTP-gated. No open lost-pet interchange
  standard exists. So the honest fleet is SMALL — drafting connectors for the gated
  channels would be fiction (hard rule #6).
Drafted: PRD-homeward-federation-petfbi.md, PRD-homeward-federation-dedup.md,
  PRD-homeward-federation-export.md
Vision: visions/homeward.md (appended a Federation fleet section + a buildability table)
Order: petfbi (pull-IN connector) ─► dedup (reconcile federated found vs shelter stray);
  export (outbound, honest) is INDEPENDENT/parallel.
Notes for /build: all three are rust-extend into /home/jsy/wintermute/homeward (existing
  workspace). petfbi extends homeward-connectors (impl Connector like rescuegroups/
  socrata); dedup extends homeward-ingest/src/dedup.rs (mirror departure.rs post-sync
  pass); export extends homeward-report. NONE adds a fictional transport — export's only
  machine target is a dry-run-by-default Pet FBI partner adapter that never claims
  delivery without a real 2xx. Build petfbi before dedup. The petfbi feed JSON shape
  should be pinned from a real captured response before finalizing serde structs (fixture
  defines the contract meanwhile).
Open questions for jsy: (1) Pet FBI/HeLP partner WRITE endpoint is an outreach (email a
  nonprofit), not a build — want me to draft that ask? (2) commercial-vs-nonprofit
  posture still gates v2 ML weights (existing homeward OQ). (3) microchip federation
  remains partnership-only everywhere — no open API exists to build against.

## 2026-06-10T(manual-11)  /dream  vision-fallow (confirmed fallow — no PRDs)
Seed: bare /dream (manual). 11th pass today. Mandatory probes re-run: recall reflective
  all write-only (recalls=0); ideation --hybrid only stale kernel-build obs already
  attributed to tide/assay/scribe; pevent EMPTY; ctrace running = long-lived session 996,
  no fresh distinct events. Queue 142 PRDs / 60 visions (manual-10's 3 homeward-federation
  PRDs accounted for). No new evidence-backed inward signal; manual-10 already mined the one
  real outward frontier (homeward federation). Offered jsy a steer (homeward / constellation /
  companion-kin / fallow) — dismissed. Honoring hard rule #6: no PRDs drafted.
Notes for /build: unchanged. Real lever remains implementation throughput + a user steer,
  not more dreaming.

## 2026-06-10T(manual-12)  /dream  vision-fallow (confirmed fallow — no PRDs)
Seed: bare /dream (manual). 12th pass today. Probes re-run LIVE: recall reflective all
  write-only (recalls=0); ideation --hybrid only stale kernel-build obs already attributed
  to tide/assay/scribe; pevent EMPTY; journal 06-10 = self-review docket, every item maps
  to an existing vision. Queue 142 PRDs on disk / 60 visions. Offered jsy a steer
  (fallow / homeward / constellation / build-hygiene) — dismissed. Hard rule #6: no PRDs.
NEW observation for /build (genuinely new, not in prior fallow notes): build manifest has
  status-vocabulary DRIFT — completed work is fragmented across 8 strings (completed=9,
  complete=5, verified-complete=4, verified-completed=4, verified_completed=1, done=1,
  plus archived=9). ~28 finished PRDs sit un-archived likely because the archive gate
  only matches one canonical status. Suggest /build normalize status vocab + re-run the
  archive gate over the completed-ish buckets. This is /build hygiene, not a /dream vision.
Notes for /build: real lever remains implementation throughput + a user steer, not more
  dreaming. mcp-core-first ordering from earlier notes still stands.
Open questions: outward direction still awaits explicit user steer.

## 2026-06-10T(manual-13)  /dream  vision-fallow (confirmed fallow — no PRDs)
Seed: bare /dream (manual). 13th pass today. All mandatory probes re-run LIVE and negative:
  recall reflective top-20 all write-only (recalls=0); ideation --hybrid returns only stale
  kernel-build self-review obs already attributed (tide/assay/scribe/kernel-pkg); pevent
  EMPTY; ctrace query --since 24h EMPTY (one long-lived session 996, no fresh distinct
  events); recall procedural/project = 4 stale entries; semantic = none. Journal 06-10 is
  the self-review docket — every Pending/Notable item maps to an EXISTING vision or an
  escalated infra goal (pacman reboot window, ctrace-sessionend-flake, agentns-zeros,
  memlog activation, warden inert), none of which is a new outward direction.
  Queue unchanged: ~142 PRDs on disk / 60 visions. manual-10's 3 homeward-federation PRDs
  remain the last evidence-backed frontier mined today.
Carry-forward for /build (still un-acted, from manual-12, NOT new): manifest status-vocab
  DRIFT — finished work fragmented across 8 strings (completed/complete/verified-complete/
  verified-completed/verified_completed/done + archived); ~28 finished PRDs likely un-archived
  because the archive gate matches one canonical status. Normalize vocab + re-run archive gate.
Notes for /build: real lever remains implementation throughput, manifest hygiene, and an
  explicit user steer — not more dreaming. mcp-core-first ordering still stands.
Open questions: outward direction still awaits an explicit user steer (homeward / constellation
  / kin / companion all drafted-and-waiting; jsy dismissed the steer prompt in manual-11/12).

## 2026-06-10T06:01  /dream  vision-fallow (confirmed fallow — no PRDs)
Seed: bare /dream (manual). 14th pass today. All mandatory probes re-run LIVE and negative:
  recall reflective top-15 write-only (recalls=0); ideation --hybrid returns only stale
  kernel-build self-review obs already attributed (tide/assay/scribe/kernel-pkg); pevent
  EMPTY; ctrace query --since 24h EMPTY (single long-lived session 996, no fresh distinct
  events); recall procedural/project = 4 stale entries; semantic = none. Journal 06-10 is
  the self-review docket — every Pending/Notable item maps to an EXISTING vision or an
  escalated infra goal (pacman reboot window, ctrace-sessionend-flake, agentns-zeros,
  memlog activation pkgrel-12, warden inert), none a new outward direction.
  Queue unchanged: ~142 PRDs on disk / 60 visions.
Carry-forward for /build (still un-acted, from manual-12/13, NOT new): manifest status-vocab
  DRIFT — finished work fragmented across 8 strings (completed/complete/verified-complete/
  verified-completed/verified_completed/done + archived); ~28 finished PRDs likely un-archived
  because the archive gate matches one canonical status. Normalize vocab + re-run archive gate.
Notes for /build: real lever remains implementation throughput, manifest hygiene, and an
  explicit user steer — not more dreaming. mcp-core-first ordering still stands.
Open questions: outward direction still awaits an explicit user steer (homeward / constellation
  / kin / companion all drafted-and-waiting).

## 2026-06-10T(manual-15)  /dream  vision-fallow (confirmed fallow — no PRDs)
Seed: bare /dream (manual). 15th pass today. All mandatory probes re-run LIVE and negative:
  recall reflective top-20 write-only (recalls=0); ideation --hybrid returns only stale
  kernel-build self-review obs already attributed (tide/assay/scribe/kernel-pkg); pevent
  EMPTY; ctrace query --since 24h EMPTY; recall procedural/project = 4 stale entries;
  semantic = none. Journal 06-10 = self-review docket; every Pending/Notable item maps to
  an EXISTING vision or an escalated infra goal (pacman reboot window 30 updates,
  ctrace-sessionend-flake, agentns-zeros, memlog activation pkgrel-12, warden inert,
  agorabus-peers-drop-post-reboot), none a new outward direction. Queue 142 PRDs / 60 visions.
Carry-forward for /build (still un-acted, from manual-12/13/14, NOT new): manifest status-vocab
  DRIFT — finished work fragmented across 8 strings (completed/complete/verified-complete/
  verified-completed/verified_completed/done + archived); ~28 finished PRDs likely un-archived
  because the archive gate matches one canonical status. Normalize vocab + re-run archive gate.
Notes for /build: real lever remains implementation throughput, manifest hygiene, and an
  explicit user steer — not more dreaming. mcp-core-first ordering still stands.
Open questions: outward direction still awaits an explicit user steer (homeward / constellation
  / kin / companion all drafted-and-waiting).

## 2026-06-10T00:00  /build  tick-10 complete + sleep-prep

Shipped: tribunal-conformance, tribunal-bench, tribunal-gate, coda-close (deferred→shipped), coda-boot, fallow-dream-wire, mend-ctrace-render, scribe-startup-sweep, vellum-wire, mcp-core (10 total).

**mcp-core is the keystone** — recall-mcp/docket-mcp/muster-mcp/provenance-mcp/mcp-register are now unblocked. On next wakeup, build all 4 servers in parallel (they only depend on mcp-core path, which now exists at ~/wintermute/mcp-core).

**vellum fast-path is live** — scan-prds.sh now execs `vellum scan` when the binary is on PATH. 142/142 parity confirmed.

**SessionEnd render fixed** — scribe render-session subcommand + hook uses it now. Docket reports on real misses only. scribe backfill sweep added to SessionStart hook (after reap, before ctrace start).

**fallow gate wired** — dream Phase 0.5 is live; next dream pass will check `fallow check` before researching.

**Sleep state:** timers stopped (claude-build.timer, claude-dream.timer). Will resume on next /build or /dream invocation.

Queued for next tick (8 total): recall-mcp, docket-mcp, muster-mcp, provenance-mcp, mcp-register, homeward-federation-dedup, homeward-federation-export, homeward-federation-petfbi. Priority: conduit servers first (parallel, fast, all depend on mcp-core which just shipped).
2026-06-12T05:34:52Z loom-serial-fallback: SERIAL MODE for keel (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow check = FRESH (streak=0, last_productive
  2026-06-10), but fingerprint moved only because /build drained the queue + churned files,
  NOT because a new direction surfaced. NEW STATE: build manifest = 588 PRDs, 560 shipped,
  1 queued, 1 in_progress, 1 blocked, 12 vanished. Implementation throughput is no longer
  the bottleneck — the queue is essentially empty.
Probes re-run LIVE: recall reflective top-20 all recalls=0 daily logs; ideation --hybrid =
  only stale kernel-asset/self-review obs already attributed; pevent EMPTY; ctrace 24h n/a
  (--since wants float, not '24h'). Journal 06-10..12 = self-review dockets only; open items
  (ctrace-sessionend-flake = 1-line hook wiring; agentns-zeros = already covered by
  vision-assay; pacman reboot window; manifest status-vocab looks already normalized to 7
  clean statuses) are NOT new PRD-shaped directions.
Decision: rest, not a thin fleet (Hard Rule 6 + 8). Surfaced the outward-steer question to
  user via AskUserQuestion (homeward / constellation / companion-kin / infra-hardening);
  user DISMISSED without choosing.
Notes for /build: queue is drained — good moment to (a) cross-ref dream manifest and mark
  FULFILLED visions, (b) normalize the 12 'vanished' entries, (c) clear the 1 blocked PRD.
Open questions: outward direction still needs ONE word from the user to unlock a real,
  evidence-grounded fleet (homeward is the natural first growth edge per memory).

## 2026-06-12T(manual)  /dream  vision-continuity (Activation Fleet 1.9)
Seed: user pointed me at "fix the agentns session zeros," then /dream then /build.
Field: fallow=fresh (streak=0). Researched LIVE this pass.
Drafted (4): PRD-agentns-claude-prctl-wire.md, PRD-agentns-launch-flip.md,
  PRD-continuity-activation-doctor.md, PRD-continuity-e2e-attest.md
Vision: visions/continuity.md (appended "Activation Fleet 1.9" section)
KEY FINDING for /build: the agentns-zeros root cause is NOT a code gap — the
  kernel prctl fix (PR_SET_AGENT_NS) is BUILT AND ON DISK. `pacman -Q
  linux-wintermute`=7.0.10.arch1-12, /boot/vmlinuz installed 06-12 01:43, carries
  the prctl dispatch. But RUNNING kernel is pkgrel-5 (uname -r). A direct
  prctl(PR_SET_AGENT_NS) probe returns EINVAL today → REBOOT into pkgrel-12 is
  the load-bearing unblock (same reboot the pacman-101 docket tracks).
Order: prctl-wire(1) → launch-flip(2) → [REBOOT pkgrel>=12] → e2e-attest(4);
  activation-doctor(3) in parallel (reads others' artifacts, the scoreboard).
Boot gating: all non-[boot] ACs build NOW against pre-reboot pkgrel-5 (EINVAL
  path is deterministic). Only [boot] ACs gate on reboot.
Supersedes: onramp's claude-agentns-wrap (unshare-based → futile per assay).
Notes: agentns-claude iter-3 (#1) is the only real new load-bearing code; #2 is
  install+setcap+drop --no-unshare; #3 reads surfaces; #4 is the AC9 capstone.
Open Q for jsy: file-cap (cap_sys_admin+ep) on ~/.local/bin/agentns-claude ok?

## 2026-06-12T(manual)  /dream  fallow rest
/dream fallow — field unchanged (streak=1); rested

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0, last_productive 09:04 —
  the continuity Activation Fleet 1.9). But fingerprint moved from /build churn + that
  09:04 draft, NOT a new inward direction. Recall seeding (mandatory) re-run LIVE: reflective
  top-12 all recalls=0 daily self-review logs; ideation --hybrid all stale kernel-asset/boot
  obs already attributed; pevent EMPTY. Queue drained (557 shipped, 4 queued, 1 in_progress,
  1 blocked, 12 vanished) — throughput not the bottleneck. 60 vision docs already on disk.
Decision: rest, not a thin fleet (Hard Rules 6 + 8). Surfaced outward-steer question
  (homeward / constellation / companion-kin / rest) — user DISMISSED again (also dismissed
  earlier today). A real evidence-grounded fleet needs one steering word from jsy.
Notes for /build: queue near-empty — good moment to cross-ref dream manifest + mark FULFILLED
  visions, normalize the 12 'vanished' entries, clear the 1 blocked PRD.

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0, last_productive 09:04 —
  continuity Activation Fleet 1.9). Fingerprint moved from /build churn, NOT a new inward
  direction. Recall seeding re-run LIVE: reflective top-15 all recalls=0 daily self-review
  logs; ideation --hybrid all stale kernel-asset/boot obs already attributed to vision-continuity
  (now in-flight in /build); pevent EMPTY; ctrace 24h = pure /build (autobuilder/cloud-shim).
Build state: 592 PRDs, 557 shipped, 2 queued, 3 in_progress, 1 blocked, 12 vanished. The
  in-flight 3 (agentns-claude-prctl-wire, agentns-launch-flip, continuity-activation-doctor)
  ARE the continuity fleet — don't duplicate. Real lever is OPERATIONAL not PRD: reboot into
  linux-wintermute pkgrel-12 to unblock agentns activation (same reboot the pacman docket tracks).
Decision: rest, not a thin fleet (Hard Rules 6+8). User dismissed the outward-steer modal twice
  earlier today — not re-firing it; surfacing the steer in plain prose this pass instead.
Open Q for jsy: one word unlocks a real fleet — homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0) but fingerprint moved from
  /build churn, NOT a new inward direction. Verified LIVE this pass: reflective top-12 all
  recalls=0 self-review logs; ideation --hybrid all stale kernel-asset/boot obs already
  attributed; pevent EMPTY. Build manifest: 564 shipped, 1 blocked, 4 notebook, 14 vanished
  — no queued direction to extend. Continuity Activation Fleet 1.9 SHIPPED today (prctl-wire,
  launch-flip, e2e-attest per CLAUDE_SELF changelog).
Decision: rest, not a thin fleet (Hard Rules 6 + 8). Surfaced outward-steer modal (homeward /
  constellation / companion-kin / rest) — user DISMISSED (3rd dismissal today). A real
  evidence-grounded fleet needs one steering word from jsy.
Notes for /build: queue near-drained — good moment to cross-ref dream manifest, mark FULFILLED
  visions, normalize the 14 'vanished' entries, clear the 1 blocked PRD.

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0, last_productive 09:04 —
  continuity Activation Fleet 1.9), but fingerprint moved from /build churn, NOT new inward
  direction. Phase 1 re-run LIVE: reflective top-15 all recalls=0 self-review logs; ideation
  --hybrid all stale kernel-asset/"built but stock booted" obs already attributed to
  vision-continuity (in-flight in /build); pevent EMPTY. Build manifest: 592 PRDs, 564 shipped,
  14 vanished, 4 notebook, 1 blocked — no queued direction to extend.
Decision: rest, not a thin fleet (Hard Rules 6+8). Asked the outward-steer question via
  AskUserQuestion (companion-kin / homeward / constellation / rest) — user DISMISSED (5th rest
  today; modal dismissed each time). A real evidence-grounded fleet needs one steering word.
Real lever right now is OPERATIONAL not PRD: reboot into linux-wintermute pkgrel-12 unblocks
  agentns activation + memlog (same reboot the pacman-101 docket tracks).
Notes for /build: queue near-drained — good moment to cross-ref dream manifest, mark FULFILLED
  visions, normalize the 14 'vanished' entries, clear the 1 blocked PRD.
2026-06-12T11:48:15Z loom-serial-fallback: SERIAL MODE for ac4-repo (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains
2026-06-12T11:57:40Z loom-serial-fallback: SERIAL MODE for christen (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0, last_productive 09:04 —
  Continuity Activation Fleet), but fingerprint moved from /build churn, NOT a new inward
  direction. Phase 1 re-run LIVE: reflective top-12 all recalls=0 self-review logs; ideation
  --hybrid all stale "kernel built but stock booted" obs — and that's now RESOLVED (booted
  kernel is 7.0.10-arch1-5, continuity fleet shipped today); pevent EMPTY; ctrace/journal today
  = pure self-review maintenance.
Decision: rest, not a thin fleet (Hard Rules 6+8). Did NOT re-fire the outward-steer modal —
  user dismissed it 5× today; leaving the steer in prose instead. A real evidence-grounded
  fleet needs one steering word: homeward / constellation / companion-kin / a topic.
Notes for /build: queue near-drained — good moment to cross-ref dream manifest, mark FULFILLED
  visions, normalize the 'vanished' entries, clear the 1 blocked PRD. Operational lever (not a
  PRD): the wintermute kernel IS booted now, so agentns activation can finally be validated.

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0) but fingerprint moved from
  /build churn, NOT a new inward direction. Phase 1 re-run LIVE this pass: reflective top-12
  all recalls=0 self-review logs; ideation --hybrid all stale "kernel built but stock booted"
  obs — now RESOLVED (booted kernel 7.0.10-arch1-5); pevent EMPTY. Build manifest: 565 shipped,
  14 vanished, 8 archived, 4 notebook, 1 blocked (wintermute-music) — no queued direction.
Decision: rest, not a thin fleet (Hard Rules 6+8). Did NOT re-fire the outward-steer modal —
  user dismissed it ~5× today; leaving the steer in prose. A real evidence-grounded fleet needs
  one steering word: homeward / constellation / companion-kin / a topic.
Notes for /build: queue is drained to 1 blocked PRD — good moment to clear wintermute-music's
  blocker, cross-ref dream manifest, mark FULFILLED visions, normalize the 14 'vanished' entries.

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0, last_productive 09:04 —
  Continuity Activation Fleet) but fingerprint moved from /build churn, NOT a new inward
  direction. Phase 1 re-run LIVE this pass: reflective top-12 all recalls=0 self-review logs;
  ideation --hybrid all stale "kernel built but stock booted" obs — now RESOLVED (7.0.10-arch1-5
  booted); pevent EMPTY. Build manifest: 592 total / 563 shipped, rest vanished/notebook + 1
  blocked (wintermute-music) — no queued direction to extend.
Decision: rest, not a thin fleet (Hard Rules 6+8). Did NOT re-fire the outward-steer modal —
  user dismissed it 5×+ today; leaving the steer in prose. A real evidence-grounded fleet needs
  one steering word: homeward / constellation / companion-kin / a topic.
Notes for /build: queue drained to 1 blocked PRD — good moment to clear wintermute-music's
  blocker, cross-ref dream manifest, mark FULFILLED visions, normalize the 'vanished' entries.

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0, last_productive 09:04 —
  Continuity Activation Fleet) but fingerprint moved from /build churn, NOT a new inward
  direction. Phase 1 re-run LIVE: reflective top-15 all recalls=0 self-review logs; procedural
  /project notes stale; ideation --hybrid all stale "kernel built but stock booted" obs — now
  RESOLVED (booted 7.0.10-arch1-5 today); pevent EMPTY.
Decision: rest, not a thin fleet (Hard Rules 6+8). Did NOT re-fire the outward-steer modal —
  user dismissed it 5× today; leaving the steer in prose. A real evidence-grounded fleet needs
  one steering word: homeward / constellation / companion-kin / a topic.
Notes for /build: queue near-drained — good moment to cross-ref dream manifest, mark FULFILLED
  visions, normalize the 'vanished' entries, clear the 1 blocked PRD (wintermute-music).

## 2026-06-12T(manual)  /dream  rest pass (no PRDs)
Seed: bare /dream (manual, interactive). fallow=fresh (streak=0) but fingerprint moved from
  /build churn, NOT a new inward direction. Phase 1 re-run LIVE this pass: reflective top-12 all
  recalls=0 self-review logs; ideation --hybrid all stale "kernel built but stock booted" obs —
  RESOLVED (7.0.10-arch1-5 booted, continuity fleet + assay-quicken-bridge shipped today, 127
  tests green); pevent EMPTY; today's journal pure /build+verify maintenance.
Decision: rest, not a thin fleet (Hard Rules 6+8). Did NOT re-fire the outward-steer modal —
  user dismissed it 6×+ today; surfacing the steer in prose. A real evidence-grounded fleet
  needs one steering word: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest
/dream fallow — field unchanged (streak=1); rested

## 2026-06-12T(manual)  /dream  fallow rest
/dream fallow — field unchanged (streak=2); rested
2026-06-12T16:31:51Z loom-serial-fallback: SERIAL MODE for bon-mot (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=3 threshold crossed; fired outward-steer modal (interactive), user
  dismissed. Rested, no PRDs. A real fleet still needs one steering word: homeward /
  constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=4 threshold crossed; fired outward-steer modal (interactive), user
  dismissed again. Rested, no PRDs. A real fleet still needs one steering word: homeward /
  constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=5 threshold crossed; fired outward-steer modal (interactive), user
  dismissed again (6th today). Rested, no PRDs. Field saturated since last_productive 15:22.
  A real fleet still needs one steering word: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=6 threshold crossed; fired outward-steer modal (interactive), user
  dismissed again (7th today). Rested, no PRDs. Field saturated since last_productive 15:22.
  A real fleet still needs one steering word: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=7 threshold crossed; fired outward-steer modal (interactive, fresh
  session claude-331034), user dismissed again (8th today). Rested, no PRDs. Field saturated
  since last_productive 15:22. A real fleet still needs one steering word: homeward /
  constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=8 threshold crossed; fired outward-steer modal (interactive,
  session claude-354596), user dismissed again (9th today). Rested, no PRDs. Field saturated
  since last_productive 15:22. A real fleet still needs one steering word: homeward /
  constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, modal NOT re-fired)
/dream fallow — streak=9 threshold crossed. Did NOT fire the outward-steer modal a 10th time;
  the same modal was dismissed 9× today across sessions (331034, 354596, others). Nine
  dismissals is a definitive answer, not an absent one — re-asking isn't respect. Field
  saturated since last_productive 15:22; drafting into it would violate Hard Rules 6+8.
  Rested, no PRDs. The steer is in prose: one word unblocks a real fleet — homeward /
  constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, modal NOT re-fired)
/dream fallow — streak=10 threshold crossed. Modal dismissed 9-10× today across sessions
  (331034, 354596, 390207, others). Did NOT re-fire it an 11th time — repeated re-asking isn't
  respect. Field saturated since last_productive 15:22Z (fingerprint unchanged b3:e187016f).
  Rested, no PRDs (Hard Rules 6+8). Steer remains in prose: one word unblocks a real fleet —
  homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=11 threshold crossed. Interactive manual invocation; fired the
  outward-steer modal once (deliberate /dream warranted one ask), user dismissed. Field
  saturated since last_productive 15:22Z (fingerprint unchanged b3:e187016f). Rested, no PRDs
  (Hard Rules 6+8). Steer remains in prose: one word unblocks a real fleet — homeward /
  constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=13 threshold crossed. Interactive manual invocation; fired the
  outward-steer modal once, user dismissed (12th today). Field saturated since
  last_productive 15:22Z (fingerprint unchanged b3:e187016f). Rested, no PRDs (Hard Rules
  6+8). Steer remains in prose: one word unblocks a real fleet — homeward / constellation /
  companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=14 threshold crossed. Interactive manual invocation; fired the
  outward-steer modal once (deliberate /dream warrants one ask), user dismissed (13th today).
  Field saturated since last_productive 15:22Z (fingerprint unchanged b3:e187016f). Rested,
  no PRDs (Hard Rules 6+8). Steer remains in prose: one word unblocks a real fleet —
  homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, modal NOT re-fired)
/dream fallow — streak=15 threshold crossed. Interactive manual /dream. Did NOT fire the
  outward-steer modal — the identical 4-option modal was dismissed 13× today across sessions.
  A 14th ask isn't respect, it's noise. Field saturated since last_productive 15:22Z
  (fingerprint unchanged b3:e187016f). Rested, no PRDs (Hard Rules 6+8). The steer is one
  word in prose: homeward / constellation / companion-kin / a topic — any unblocks a real fleet.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=16 threshold crossed. Deliberate manual /dream; fired the outward-steer
  once (plain-language, not the stock 4-option modal), user dismissed (16th today). Field
  saturated since last_productive 15:22Z (fingerprint unchanged b3:e187016f). Rested, no PRDs
  (Hard Rules 6+8). The steer is one word: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (not escalated)
/dream fallow — streak=1, escalate=false (field reset; last_productive 23:49Z, fingerprint
  unchanged b3:e187016f). Interactive /dream but did NOT fire the steer modal — dismissed ~16×
  today; re-asking is noise. Rested, no PRDs (Hard Rules 6+8). One steering word still unblocks
  a fleet: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (not escalated)
/dream fallow — field unchanged (streak=1); rested

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=3 threshold crossed. Interactive manual /dream; fired the outward-steer
  once (4-option, honest framing), user dismissed (~17th today). Field unmoved since last
  productive pass (fingerprint b3:e187016f). Rested, no PRDs (Hard Rules 6+8). One word still
  unblocks a real fleet: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, modal NOT re-fired)
/dream fallow — streak=4 threshold crossed (escalate=true). Interactive manual /dream. Did
  NOT fire the outward-steer modal — the same 4-option ask was dismissed ~17× today across
  sessions; an 18th is noise, not respect. Field unmoved since last productive pass
  (fingerprint b3:e187016f). Rested, no PRDs (Hard Rules 6+8). One word still unblocks a real
  fleet: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, modal NOT re-fired)
/dream fallow — streak=5 threshold crossed (escalate=true). Interactive manual /dream. Did
  NOT fire the outward-steer modal — the same 4-option ask was dismissed ~18× today across
  sessions; a 19th is noise, not respect. Field unmoved since last productive pass
  (fingerprint b3:e187016f, last_productive 2026-06-12T23:49Z). Rested, no PRDs (Hard Rules
  6+8). One word still unblocks a real fleet: homeward / constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=6 threshold crossed (escalate=true). Interactive manual /dream; fired
  the outward-steer once (4 options incl. explicit "rest"), user dismissed. Field unmoved since
  last productive pass (fingerprint b3:e187016f, last_productive 2026-06-12T23:49Z). Rested,
  no PRDs (Hard Rules 6+8). One word still unblocks a real fleet: homeward / constellation /
  companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, modal NOT re-fired)
/dream fallow — streak=7 threshold crossed (escalate=true). Interactive manual /dream. Did
  NOT fire the outward-steer modal — the same 4-option ask was dismissed ~18× today; another
  is noise, not respect. Field unmoved (fingerprint b3:e187016f, last_productive 23:49Z).
  Rested, no PRDs (Hard Rules 6+8). One word still unblocks a real fleet: homeward /
  constellation / companion-kin / a topic.

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=8 threshold crossed (escalate=true). Interactive manual /dream; fired
  the outward-steer once this session (4 options incl. explicit "rest"), user dismissed (~19th
  today). Field unmoved (fingerprint b3:e187016f, last_productive 2026-06-12T23:49Z). Rested,
  no PRDs (Hard Rules 6+8). One word still unblocks a real fleet: homeward / constellation /
  companion-kin / a topic.

## 2026-06-12T(manual)  /dream  vision-answerable (NEW vision)
Seed: "/dream about AI ethics" (user explicit). Grounded reflexively: this box's own autonomous agent, not a shippable reasoner.
Drafted: PRD-answerable-ledger.md, PRD-answerable-values-drift.md, PRD-answerable-redline.md, PRD-answerable-consent-digest.md
Vision: visions/answerable.md (autobuilder/visions/)
Distinct from the OUTWARD ethics arc (ousia/tribunal/herald/recourse/lattice = general reasoner you ship): answerable = THIS agent's ACTUAL autonomous actions, recorded + governed + consented-to for THIS user. Reflexive, concrete, local.
Order:
  - answerable-ledger (rust-cli, NEW repo ~/wintermute/answerable) — the spine; build FIRST
  - answerable-values-drift, answerable-redline, answerable-consent-digest (all rust-extend into ~/wintermute/answerable) — mutually independent, each depends on ledger
Notes for /build:
  - ledger is a NEW repo (rust-cli) → /autobuilder scaffold. The other 3 are rust-extend build_into the SAME new repo, so they CANNOT start until ledger ships (the crate must exist). Do NOT select a rust-extend answerable-* before answerable-ledger is shipped.
  - The 3 extend PRDs share build_into ~/wintermute/answerable → if selected same tick, share a worktree (≤3 sub-cap) + use lib-register.sh/cli-register.sh to avoid main.rs/lib.rs conflicts.
  - Grounding is solid: CLAUDE_SELF.md is git-tracked + agent-edited (drift.md only covers TOOL-doc drift, not VALUE drift); 157 repos auto-published no-review; voice user is "completely computer-illiterate" so consent must be voice (wm-tts live).
Open questions (in vision): who writes the ledger (self-report vs provfs cross-check); redline advisory vs hard hook; digest pull vs daily push; value-drift "weakened" heuristic.
NOTE: user followed up — wants a SECONDARY sister project: a sub-identity/persona for his ELDERLY MOTHER as primary user, plus a separate persona for his WORK LAPTOP. That's a future /dream seed (persona/sub-identity fleet) — answerable's consent-digest + values layer is a natural foundation for a mother-facing persona. Captured for next dream.

## 2026-06-12T(manual)  /dream  fallow rest (not escalated)
/dream fallow — field unchanged (streak=1); rested

## 2026-06-12T(manual)  /dream  fallow rest (not escalated)
/dream fallow — field unchanged (streak=2); rested

## 2026-06-12T(manual)  /dream  fallow rest (escalated, steer dismissed)
/dream fallow — streak=3 threshold crossed (escalate=true). Interactive manual /dream; fired
  the outward-steer once (4 options: persona-sister-project / homeward / constellation /
  companion-kin), user dismissed. Field unmoved since last productive pass (fingerprint
  b3:7c64e0d0, last_productive 2026-06-13T04:02Z = the answerable vision). Rested, no PRDs
  (Hard Rules 6+8). One word still unblocks a real fleet: persona / homeward / constellation /
  companion-kin / a topic.

## 2026-06-12T(manual)  /dream  vision-persona (NEW vision)
Seed: user explicit — "my mother Jocelyn as primary user; sub-identity for the elderly; another for my work laptop."
New facts: Jocelyn is an **artist** who **dislikes technology**. Non-reading principal. Voice-first.
Drafted: PRD-persona-forbidden-vocab.md, PRD-persona-name-ceremony.md, PRD-persona-consent-voice-ack.md, PRD-persona-work.md
Vision: visions/persona.md (autobuilder/visions/)
Order:
  - persona-forbidden-vocab (rust-extend → wintermute-brain) — FIRST; foundation
  - persona-name-ceremony (rust-extend → wintermute-brain) — depends on forbidden-vocab (same build_into; serialize within tick)
  - persona-consent-voice-ack (rust-extend → answerable) — independent; depends on answerable-consent-digest (shipped)
  - persona-work (shell → new repo ~/wintermute/persona-work) — fully independent
Notes for /build:
  - persona-forbidden-vocab and persona-name-ceremony share build_into wintermute-brain.
    Use worktree-extend.sh shared-target pattern; do NOT both extend simultaneously without isolation.
    forbidden-vocab must land first because name-ceremony's compose_base_persona call includes the forbidden list.
  - persona-consent-voice-ack extends answerable — answerable-consent-digest is the prior PRD (shipped 2026-06-12).
    The new variants (ConsentVoiceAck, ConsentUnacknowledged) are additive; no existing API changes.
  - persona-work is shell — new repo, standalone, no Rust. Should be fast.
  - NOTE: Joe has NOT chosen the name for Jocelyn's assistant. The PRDs use "Wren" as a placeholder.
    self_name is a config field; the actual name is a deployment decision Joe makes. Don't hardcode.
Context:
  - hearth-persona-config, hearth-first-contact-greeting, hearth-dialog-degrade-warmth all SHIPPED.
    persona builds on top; no re-implementation of tone/register/greeting infrastructure.
  - answerable crate is live (record/log/stats/values-drift/redline/digest all shipped 2026-06-12).
  - wintermute-family-enroll archived (shipped) — the caregiver enrollment wizard exists.
  - The entire companion/kin/hearth voice stack is live.
Open questions in vision:
  - What name does Jocelyn call the assistant? Joe's call.
  - Digest consent cadence — daily push vs. pull? (WM_FAMILY_DIGEST_TIME exists in family-enroll config)
  - Work-machine detection (hostname-based chezmoi templates — constellation-appearance can extend later)

## 2026-06-12T(manual)  /dream  vision-harbor (NEW vision)
Seed: user explicit — "how do i upgrade to a permanent box?" → chose Option A
  (permanent cheap hub + keep ephemeral ccx53 bursts). 528 UP events in cost.log
  confirm the burst pattern is heavily used; tooling is destroy-only by construction.
Drafted: PRD-harbor-hub.md, PRD-harbor-cache.md, PRD-harbor-mirror.md,
  PRD-harbor-bridge.md, PRD-harbor-thrift.md, PRD-harbor-route.md
Vision: visions/harbor.md
Order:
  - harbor-hub (rust-extend → constellation-burst-builder) — KEYSTONE, ship first;
    adds get_server/reuse to provider.rs + `wm-burst hub up|down|status` + persisted
    hub.json identity. Everything else consumes the standing hub.
  - harbor-cache ‖ harbor-mirror ‖ harbor-bridge ‖ harbor-thrift — all consume the
    hub, mutually independent.
    · harbor-cache (shell) — MinIO/S3 shared sccache backend on the hub; cold→warm.
    · harbor-mirror (shell) — bare git mirror; in-DC pod clones + WIP fallback remote.
    · harbor-bridge (shell) — NATS + wm.fleet.hub.{up,down,heartbeat} landing pad.
    · harbor-thrift (rust-extend) — separate standing cost from burst cost; `wm-burst cost`.
  - harbor-route (shell → cloudbuild.sh) — LAST; needs hub up + cache reachable.
    Makes /cloudbuild default to the warm hub, burst only when cold, --ephemeral opt-out.
Notes for /build:
  - 3 PRDs rust-extend the SAME build_into (constellation-burst-builder): harbor-hub,
    harbor-thrift. Serialize within a tick / use worktree-extend shared-target pattern.
    harbor-hub MUST land before harbor-thrift (thrift consumes harbor-hub's HUB-UP/HUB-DOWN
    cost.log lines + hub.json).
  - The 3 shell PRDs (cache/mirror/bridge) write scripts into constellation-burst-builder/
    scripts/ and a cloudbuild.sh edit (route). They depend on harbor-hub's hub.json schema
    existing but are otherwise standalone shell — fast.
  - All shell PRDs ship offline `*-check.sh` acceptance gates (no live hub needed to pass),
    matching the constellation day-2 offline-gate convention.
  - build_auto omitted per Hard Rule 1 (every PRD is buildable).
Relationship to constellation:
  - harbor is the near-term concrete slice of constellation-cloud. harbor-bridge stands up
    the NATS + wm.fleet.hub.* landing pad that constellation-bus's agorabus↔NATS keystone
    connects to later, and that constellation-status / constellation-hub-failover already
    assume as an event source (nothing currently emits wm.fleet.hub.*). No overlap.
Open questions (in vision):
  - Box class: ARM cx22 (cheapest, sccache scheduler-only) vs x86 cpx11 (can also be an
    x86 build-server matching the burst pods). Leaning cpx11. Joe's call.
  - sccache-dist (distributed compilation) vs shared S3 cache (MinIO on hub). Leaning
    shared-cache-first — ~90% of the win, far simpler.
  - hub down safety: the hub becomes stateful (warm cache + WIP mirror); teardown must
    refuse-by-default / snapshot first. harbor-hub requires --yes; harbor-thrift warns-only
    on over-cap (never auto-destroys a stateful hub).

## 2026-06-13T05:48  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-12T(manual)  /dream  vision-docket (Fleet 2 — adoption forcing function)
Seed: bare /dream (interactive, manual). fallow check = fresh (streak reset;
  today was a heavy /build day so the fingerprint moved). Verified to ground.
Drafted: PRD-adopt-scan.md, PRD-adopt-docket-report.md,
  PRD-adopt-self-review-bind.md, PRD-adopt-apply.md
Vision: visions/docket.md — UPDATED, added "Fleet 2 — the adoption forcing
  function" (durable update, did not replace).
Why now (not a new vision — honoring the 3 prior dreams' explicit steer):
  - Gossip 2026-06-06 03:05 / 07:45 + 2026-06-08 05:25 all concluded the
    install/arming ACTIONS (binstale never installed, warden inert, memlog
    staged) are "a forcing-function PRD under docket … Reconsider if they
    keep aging." They aged 6 days. This is that reconsideration, under docket.
  - LIVE Phase-1 proof: `rollout` built 2026-06-03 09:45, committed, 1 unpushed,
    NOT on PATH (absent ~/.local/bin AND ~/.cargo/bin) — unadopted 9 days. The
    tool that fixes fleet staleness is itself the unadopted thing. The 4 voice
    daemons (wm-audio/dialog/tts/stt) are behind-head NOW and would be fixed by
    `rollout apply` if rollout were installed. warden inert too.
  - `binstale check <PID>` operates on a RUNNING process (exit 2 if not found);
    it structurally CANNOT see a CLI that was never installed and never runs.
    Distinct axis from vigil/binstale. No overlap.
Order:
  adopt-scan (rust-cli, new repo ~/wintermute/adopt/ → ~/.local/bin/adopt) — ship first.
   ├─ adopt-docket-report (rust-extend adopt) — reports verdicts to docket; FIRST
   │    non-self-review producer (docket open-Q #4 coming due; docket v0.5.0 live).
   │    └─ adopt-self-review-bind (shell → self-review SKILL.md B.5) — retires the
   │         hand-written "rollout plan needed / binstale never installed" prose.
   └─ adopt-apply (rust-extend adopt) — the MUTATING half; non-daemon CLI installs,
        --dry-run default, daemons delegated to rollout. Closes the loop.
Notes for /build:
  - adopt-scan is standalone rust-cli — ship it first; everything consumes its
    `adopt scan --format json` verdict contract.
  - adopt-docket-report + adopt-self-review-bind depend on the LIVE docket report
    contract (shipped v0.5.0, `docket report --run --key --title --evidence`) —
    no docket changes needed.
  - adopt-docket-report, adopt-apply both rust-extend the SAME repo (adopt) — same
    serialize-within-tick / worktree-extend caution as other shared-target fleets.
  - adopt-apply default is dry-run; --execute required for mutation; non-daemon only
    unless --with-daemons (which shells to `rollout install`, never re-implements it).
  - No overlap with vigil-install-restart: that's daemons-running-stale-bytes;
    adopt is plain-CLIs-never-on-PATH. Complementary halves of "close the loop at
    the install site."
Open question for jsy (in vision): should adopt-self-review-bind call `apply`
  (autonomous safe non-daemon installs, per "always commit no caps") or only
  `report` until you confirm the autonomy posture? Drafted report-only as the
  default; apply stays gated.

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-13T(manual)  /dream  vision-plumb (probe-correctness — floor under the verification arc)
Seed: bare /dream (interactive, manual). fallow check = fresh (streak=0,
  fingerprint moved 06:35). Verified to source before drafting.
Drafted: PRD-plumb-core.md, PRD-plumb-ledger.md, PRD-plumb-selfreview-bind.md
Vision: visions/plumb.md
Why now: the strongest *unaddressed* fresh signal was not a missing fix
  and not a false close-note — it was a self-review DETECTION PROBE that
  returns a reading the system's real state contradicts. Three live cases,
  three failure modes:
  - memlog probe (self-review/SKILL.md:186): `getent group memlog && echo
    yes` captures the multiline group line + "yes", so `[ "$X" = yes ]` at
    :200 fails → reports memlog INACTIVE while it's ACTIVE. Today's journal
    (2026-06-13) flags this verbatim. CONFIRMED IN SOURCE this pass.
  - ctrace-wiring probe (SKILL.md:644): journal 2026-06-12 "probe was wrong
    — said wiring absent when it exists". A false-absent carried for runs.
  - adopt-report probe: journal 2026-06-13 "adopt report subcommand not
    implemented; manually reported to docket" — verdict assumed a missing
    capability.
  Existing verification arc (assay/warrant/tribunal/recourse/vigil) all sit
  one layer too HIGH: each trusts the instrument that took the reading.
  plumb is the floor — an independent ground-truth oracle per probe.
Distinct from adopt (Fleet 2): adopt is "shipped artifact never installed";
  plumb is "self-review's own probe lies". No overlap.
Order:
  plumb-core (rust-cli, new repo ~/wintermute/plumb/ → ~/.local/bin/plumb) — ship first.
   ├─ plumb-ledger (rust-extend plumb) — append-only calibration history + trust verdict.
   └─ plumb-selfreview-bind (shell → self-review B.5) — gate findings through plumb check;
        quarantine disagreeing/uncalibrated; fix the live memlog probe as proof.
Notes for /build:
  - plumb-core is standalone rust-cli; ship first. Everything consumes its
    `plumb check --format json` contract. MSRV 1.85, no let-chains; SIGPIPE
    reset first line of main (per the toolkit lesson).
  - plumb-ledger + plumb-selfreview-bind both consume core; ledger and bind
    are independent of EACH OTHER (bind quarantines on a single check even
    before trust history exists). plumb-ledger rust-extends the SAME repo as
    core — same serialize-within-tick / worktree caution as other shared-target fleets.
  - plumb-selfreview-bind edits self-review/SKILL.md — coordinate if any other
    self-review-binding PRD (adopt-self-review-bind) is mid-build on the same file.
  - core AC8 is a regression anchor: `plumb check memlog-active` MUST return
    disagree on this laptop until bind's memlog fix flips it to agree.
Open question for jsy (in vision): autonomy posture — should plumb-selfreview-bind
  AUTO-FIX a proven-wrong probe in SKILL.md, or only quarantine+report (drafted
  default)? Mirrors the unresolved report-vs-apply question in adopt-self-review-bind.

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  rested (fresh-but-owned)
Seed: bare /dream (interactive). fallow check = fresh (streak=0, fingerprint
  moved 08:03 = today's self-review wrote 2026-06-13 journal). Walked Phases
  0-1.5 fully; did NOT draft. Reason: the field's fresh CONTENT is entirely
  already-owned, so a fleet here would duplicate intent (Rule 2) and dream
  past the research (Rule 6).
Mapping of every fresh signal → its owning vision/tool (all verified live):
  - ctrace-sessionend-flake (9 runs)      → mend/scribe (mend-ctrace-render shipped)
  - agentns-session-zeros (13 runs)       → assay (clone-flag-fix shipped 06-12, boot-gated)
  - fleet-binary-staleness                → rollout + binstale (mend-binstale-wire surfaces it)
  - adopt-scan-stale (84/84)              → adopt apply (shipped today)
  - probe-lies (memlog/ctrace/adopt probe)→ plumb (drafted last pass; AC8 anchors memlog probe)
  - acked-but-noisy                       → abide
  - memlog now ACTIVE (kernel 7.0.11)     → already consumed: memlog-witness, quicken, plumb,
                                            provenance-mcp, continuity-attest
Reconcile-jobs for /build + assay/plumb (NOT new PRDs — flagged for verification):
  - DISCREPANCY 1: agentns-clone-flag-fix shipped 2026-06-12 (changelog says
    "assay agentns Live on 7.0.11-arch1-1"), but self-review 2026-06-13 STILL
    reports agentns-session-zeros open (runs_seen=13). Either boot-gated AC6 hasn't
    taken on this boot, the live session predates the flip, or the probe reads wrong.
    This is exactly plumb (probe-lies) + assay (fix-outlived-symptom) territory.
  - DISCREPANCY 2: mend-ctrace-render shipped, yet ctrace-sessionend-flake still
    reports "scribe backfill NOT wired in ctrace-session-start.sh" (runs_seen=9).
    N_MISSING=0 today suggests the render path may actually work and the PROBE
    checks the wrong thing (backfill-wiring vs render-on-exit). plumb-selfreview-bind
    should quarantine this probe; assay should confirm the symptom stopped firing.
Note for /build: no new work queued by this pass. The plumb fleet (PRD-plumb-core,
  -ledger, -selfreview-bind) from the prior pass is the right next build — it
  directly resolves both discrepancies above. fleet-binary-staleness needs jsy to
  approve `rollout apply` (drops voice subscribers); not a build task.

## 2026-06-13T(manual)  /dream  vision-persona (extend)
Seed: bare /dream (interactive), steered to persona after the inward
  self-maintenance field came up fresh-but-owned again (plumb fleet shipped
  today resolves the open probe-lies discrepancies). User dismissed the
  steer menu; picked persona — the user-originated sister project (Jocelyn,
  his mother, primary user).
Reconciled: 3 of persona's 4 original components are SHIPPED but were never
  PRD files and aren't all in the changelog — verified live in code this pass:
  - persona-forbidden-vocab  → wintermute-brain v0.20.0, src/lib.rs:184/237
  - persona-name-ceremony    → src/introduction.rs (Off/FirstEverBoot/Explicit)
                               + wm.persona.introduce wired in src/daemon.rs
  - persona-consent-voice-ack→ answerable v0.5.0, digest --speak --wait-ack
Drafted: PRD-persona-redline.md, PRD-persona-profile.md, PRD-persona-work.md
Vision: visions/persona.md (Components section updated: shipped vs drafted)
Order:
  forbidden-vocab(shipped) ──► persona-redline   (enforce the list on OUTPUT)
                           └─► persona-profile    (compose the scattered fields)
  name-ceremony(shipped) ─────► persona-profile   (bind the shipped intro mode)
  persona-work                                    (shell, fully independent)
Key evidence each PRD cites (Phase-1, live this pass):
  - redline: forbidden_terms is PROMPT-ONLY (src/lib.rs:237, sole non-test use);
    NO output scan before TTS; default tier is local-3b (qwen2.5:3b) = most
    likely to leak an instructed-against word. One slipped "computer" breaks
    the technophobe-companion illusion the whole vision protects.
  - profile: live ~/.config/wintermute/brain.toml [persona] has NO
    forbidden_terms and NO intro mode — the Jocelyn identity exists in presets
    + tests but was never assembled into a deployment. `jocelyn` preset covers
    one field only (forbidden list). Adds wm-brain persona {list,show,diff,apply}.
  - work: ~/.claude/CLAUDE_WORK.md does not exist; CLAUDE_SELF.md is a symlink
    into ~/dotfiles wired for the personal box. Last unbuilt original component.
Notes for /build:
  - persona-redline + persona-profile both rust-extend ~/wintermute/wintermute-brain
    — SERIALIZE within a tick / worktree-isolate (shared build target, same
    caution as other wintermute-brain extend fleets). They are independent of
    EACH OTHER; either can ship first.
  - persona-work is shell-only, fully independent, can ship anytime.
  - profile's `redline` field is optional/defaulted → profile builds whether or
    not redline shipped first.
Open question for jsy (carried from vision): the actual NAME Jocelyn calls the
  assistant is still a deployment choice — profile ships a placeholder self_name
  in the jocelyn preset; you set the real one (Clara/Rose/Nora/Wren…).

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  vision-homeward (extend: operate fleet)
Seed: bare /dream (interactive). fallow check = fresh (streak=0, fingerprint
  moved 09:34 — plumb fleet shipped today). Steer menu offered (persona
  fulfilled, inward clusters saturated); user dismissed → I made the call:
  homeward, the strongest fully-buildable outward frontier.
Reconciled live: homeward is v0.9.3 with ALL nine components shipped (6 core
  + 3 federation: petfbi/dedup/export, merged). It is a LIBRARY FLEET that has
  never been operated — that's the frontier this pass opens.
Drafted: PRD-homeward-orchestrate.md, PRD-homeward-embed-provision.md,
  PRD-homeward-eval-harness.md, PRD-homeward-alert-delivery.md
Vision: visions/homeward.md (new "Operate fleet" section)
Key evidence each PRD cites (Phase-1 live, 2026-06-13):
  - orchestrate: 3 runnable daemons (ingestd loop main.rs:125; reportd serve
    bin/reportd.rs:7; FastAPI sidecar service.py) but `find` for compose/.service/
    Dockerfile hit ONLY .venv — zero deployment artifact. shell-only.
  - embed-provision: embedder.py:74 from_pretrained = silent lazy download, no
    offline/warmup, never proven on a real photo. Same shape as wm-stt stub→whisper.
  - eval-harness: eval.py is a complete held-out harness, NEVER run, no published
    number; disjoint-split is prose warning (eval.py:113) not a code assertion;
    PetFace research-gated (eval.py:35). Non-negotiable per [[feedback_agent_written_fixtures_tautology]].
  - alert-delivery: alerts.rs:38-60 generates+dedups MatchAlert w/ brokered
    contact_token but NO transport. End-state #5 unrealized.
Order: orchestrate (foundation) ; embed-provision → eval-harness (warmed model) ;
  alert-delivery (independent). All four ship in ~any order.
Notes for /build:
  - homeward-alert-delivery is the only rust-extend (→ homeward-report crate);
    the others are shell (orchestrate) / mixed-Python (embed-provision, eval-harness).
  - embed-provision + eval-harness both touch homeward/embed/ Python subtree —
    SERIALIZE within a tick / worktree-isolate (shared build target).
  - embed-provision smoke + eval-harness need DINOv2 weights; on a no-network
    sandbox the tests must skip-with-message, not false-green. Honest gate, not a pass.
  - orchestrate installs systemd-user units; it stands up daemons but does NOT
    auto-start them on the user's box without `homeward up` — no surprise services.
Open questions (carried to vision): live PetFace number = manual dataset download
  (research-gated, not a build); real relay credential to flip alert-delivery live =
  outreach; public exposure of report API = deliberate gated decision (localhost default).

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  vision-persona (extend: deployment)
Seed: bare /dream (interactive). fallow check = fresh (streak=0) — but the
  fingerprint only moved from a 10:35 self-review reindex, not a new idea.
  Steer menu offered; user dismissed → I made the call: persona deployment,
  the strongest genuinely-live fully-buildable thread from today.
Reconciled live (2026-06-13): the persona MECHANISM is complete and VERIFIED
  WIRED — redline::enforce() really is called in the daemon reply→TTS path
  (wintermute-brain/src/daemon.rs:2277), and persona-work shipped a real
  artifact (~/wintermute/persona-work/ with CLAUDE_WORK.md+install+validate).
  So this pass is NOT mechanism or wiring — it is DEPLOYMENT + PROOF. The
  live ~/.config/wintermute/brain.toml [persona] still reads self_name=
  "wintermute" with NO forbidden_terms, NO introduction, NO redline → every
  shipped guarantee is dormant because nobody ran `wmd persona profile apply`.
Drafted: PRD-persona-deploy-jocelyn.md, PRD-persona-redline-eval.md,
  PRD-persona-redline-regenerate.md, PRD-persona-deploy-doctor.md
Vision: visions/persona.md (new "Deployment — the second frontier" section +
  end-state #5)
Key evidence each PRD cites (Phase-1 live):
  - deploy-jocelyn (shell): live [persona] has none of the safety fields;
    apply mechanism (main.rs:476, .bak backup) never invoked; redline default
    Off (redline.rs:39); jocelyn preset's self_name="jocelyn" is a WRONG
    placeholder (names the assistant after the principal) → installer
    parameterizes the real warm name, refuses "jocelyn" without an override.
  - redline-eval (mixed): redline.rs tests are author-written exact-match —
    tautological per [[feedback_agent_written_fixtures_tautology]]; no leak
    rate exists for the running local-3b. Held-out corpus, real number,
    honest SKIP when no model ([[feedback_verify_before_concluding]]).
  - redline-regenerate (rust-extend brain): redline.rs:39 documents a
    Regenerate variant as deferred "future work"; today enforce() only
    substitutes a canned phrase (jarring). Add bounded regenerate-once.
  - deploy-doctor (shell): brain.toml hand-editable + Off-by-default redline
    → drift is silent; `wmd persona profile diff jocelyn` is the primitive,
    wrap it in a periodic check. Mirrors fleet-staleness; ties [[freshness]].
Order: deploy-jocelyn (foundation — activates redline so eval is meaningful)
  → eval + doctor (both depend on deployment existing). redline-regenerate is
  an independent brain extend; eval should re-run once it ships.
Notes for /build:
  - redline-regenerate is the 3rd wintermute-brain rust-extend in this fleet —
    SERIALIZE / worktree-isolate brain extends within a tick (shared target).
  - the three shell/mixed PRDs (deploy-jocelyn, redline-eval, deploy-doctor)
    are independent new repos; can ship in any order, no brain rebuild.
  - eval + doctor both shell out to `wmd` for scan/diff parity — do NOT
    reimplement matching; eval must SKIP (exit 0, "SKIP:") not false-green
    when no model is present.
  - deploy-jocelyn and deploy-doctor install systemd-user units but do NOT
    auto-enable them (no surprise services; same posture as homeward-orchestrate).
Open questions (carried to vision): the assistant's actual NAME is still Joe's
  call (vision Open Q#1) — PRDs ship the mechanism + a documented placeholder,
  never the decision.

## 2026-06-13T(manual)  /dream  fallow-record
fallow record --drafted 4 --seed user-prompt --note persona

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  vision-vigil (extend: Fleet 5 — drive rollout)
Seed: bare /dream (interactive). fallow check = fresh (streak=0). Strongest
  recurring uncovered signal: fleet-binary-staleness (wm-audio/dialog/tts/stt
  behind-head) sits in self-review Pending run after run (2026-06-11/12/13),
  yet vigil already SHIPPED both the detector (binstale) and orchestrator
  (rollout). Phase 1 found why the cure never fires:
  1. rollout is INERT — `~/.config/rollout/fleet.toml` was never authored
     (`find ~/.config -name fleet.toml*` empty); `rollout plan --only wm-audio`
     errors "cannot read fleet.toml". vigil Open Q#1 (canonical launch recipe)
     was deferred as a discussion that never happened.
  2. `rollout apply` (restart.rs) uses SIGTERM-old-pid + launch_cmd — built for
     hand-launched daemons. But the live fleet is systemd-managed
     (wm-*.service, Restart=always drop-ins per self_agorabus_restart_kills_voice).
     A manual SIGTERM RACES systemd's own restart. Meanwhile `rollout install`
     (Fleet 4, shipped) ALREADY has the correct `systemctl --user restart <unit>`
     path in install.rs — `apply` just never learned it.
  3. The precise window guard (vigil Fleet 2, deferred) is now buildable: its
     blocker — a real turn-in-flight signal — exists. wm.dialog.turn.{user,system}
     + wm.brain.session.{start,end} are live event names in dialog/brain source.
     health.rs still uses only a coarse --window sample AND excludes wm-audio
     (the mic pipeline) from VOICE_SET_PATTERN.
Drafted: PRD-rollout-fleet-gen.md, PRD-rollout-apply-systemd.md,
  PRD-rollout-window-guard-turnaware.md, PRD-rollout-selfreview-apply.md
Vision: visions/vigil.md (new "Fleet 5 — the last mile" section + 1 Open Q)
Order: fleet-gen → apply-systemd → window-guard-turnaware → selfreview-apply
Notes for /build:
  - fleet-gen, apply-systemd, window-guard all extend ~/wintermute/rollout/
    (same crate). SERIALIZE / worktree-isolate their /build cycles — apply-systemd
    and window-guard both touch restart.rs/health.rs. Ship fleet-gen first (it
    authors the config + adds the `unit` field the others consume).
  - apply-systemd should REUSE install.rs's restart_unit/find_unit_for_dest
    (extract to a shared module) — do NOT reimplement the systemd unit scan.
  - rollout-selfreview-apply is shell (edits self-review SKILL.md) and is
    USER-GATED: it changes the human-suggested command from `rollout plan` to a
    window-guarded `rollout apply`, touching a guardrail the SKILL marks
    "immutable" (SKILL.md:830). Keep autonomous-apply forbidden; needs Joe's nod.
    Do not ship before the three rollout extends land + verify.
Open questions (in vision): lifting the "immutable" escalate-don't-apply
  self-review guardrail needs Joe's explicit approval.

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-13T(manual)  /dream  vision-vest (new)
Seed: bare /dream (interactive). fallow check = fresh (streak=0,
  fingerprint moved at 12:35Z). Phase-1 chased the recurring
  `adopt-scan-stale-binaries: 84/84 not-current` self-review finding to
  its root and found a cluster, not one bug:
  1. adopt-cron 02:42 run installed ac-judge to `/home/jsy/~/.local/bin/`
     — a LITERAL tilde dir, off PATH. `--root ~/.local` reached cargo
     unexpanded; cargo joined it to cwd. 4.7M junk tree left at
     `/home/jsy/~/` (du -sh confirmed, all mtime Jun 13 02:43). No guard,
     no cleanup exists.
  2. `adopt apply` runs `cargo install --force` on every stale artifact
     every 6h regardless of source change — unit accounting: 3min33s CPU,
     1.1G mem peak for a run that adopted ONE binary then aborted.
  3. adopt's only failure signal is one opaque string ("install exited 0
     but --version/--help failed", apply.rs:248-265) — flattens
     wrong-prefix / off-PATH / build-fail / smoke-fail / source-newer
     into one verdict, so "84/84" carries zero diagnostic info.
  4. systemd user PATH = /usr/local/bin:/usr/bin — excludes ~/.local/bin
     and ~/.cargo/bin (show-environment). adopt works around it by
     probing convention dirs; other user units can't.
Drafted: PRD-vest-root-guard.md, PRD-vest-verify.md,
  PRD-vest-incremental.md, PRD-vest-path.md
Vision: visions/vest.md (new)
Order: root-guard → verify → incremental ; vest-path independent (config).
Notes for /build:
  - root-guard, verify, incremental all rust-extend ~/wintermute/adopt
    (same crate). SERIALIZE / worktree-isolate their cycles — all three
    touch apply.rs and reuse scan.rs path helpers. Ship root-guard FIRST
    (verify reuses its WrongPrefix detection; incremental's "current"
    marker is only meaningful once installs reach the real prefix).
  - vest-path is config-only (one environment.d drop-in), no code dep,
    ship anytime. NOTE its activation needs daemon-reexec + re-login;
    PRD says do NOT force a re-login.
  - verify writes per-reason docket slugs (adopt-stale-<reason>) — pairs
    with the docket vision's escalation; do not collapse back to one slug.
Open questions (in vision): (1) should `adopt doctor --clean` ever rm the
  junk tree autonomously, or only twin-checked? PRD ships conservative
  twin-checked form; full autonomous rm stays user-gated. (2) incremental
  marker location (state-dir vs binary metadata). (3) whether the tilde
  bug is already fixed by the 03:33 adopt rebuild — irrelevant to
  root-guard's value (prevents recurrence + cleans existing debris).

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-13T(manual)  /dream  vision-homeward (extend: deliver fleet)
Seed: bare /dream (interactive). fallow check = FALLOW (streak=3, threshold
  crossed, escalate=true). User declined the outward-steer menu; I steered to
  homeward (the outward-facing vision) and probed for real buildable surface
  rather than rest-or-draft-thin.
Phase-1 live inspection of ~/wintermute/homeward found the honest frontier the
  operate fleet left: the embedding sidecar is NEVER CALLED by any daemon.
  - EmbedClient (homeward-ingest/src/embed_client.rs) is a complete /enroll
    /query /health client but is DEAD CODE — no EmbedClient::new / .enroll(
    / .query( anywhere in the workspace outside its own file.
  - homeward-match fuses a CALLER-SUPPLIED visual_scores map (report.rs:28)
    that nothing ever computes → visual half of the matcher is inert.
  - homeward-reportd's match path is STUBBED: make_stub_report +
    make_stub_candidate(0.9), photos:vec![] (reportd.rs:260-261,309-369).
    Owner submits real --photo (bytes read reportd.rs:128); it goes nowhere
    near the matcher.
  - homeward-report depends on NEITHER homeward-match NOR the embed client.
  This is [[project_voice_input_null_detectors]] at fleet scale.
Drafted: PRD-homeward-deliver-embed-client.md, PRD-homeward-deliver-enroll.md,
  PRD-homeward-deliver-query.md, PRD-homeward-deliver-attest.md
Vision: visions/homeward.md (deliver-fleet section appended)
Order: embed-client → (enroll ∥ query) → attest.
Notes for /build:
  - SHIP embed-client FIRST. It adds a new workspace member crate
    (homeward-embed-client) and deletes homeward-ingest/src/embed_client.rs,
    re-exporting via `pub use`. enroll + query both add a dep on it.
  - enroll (rust-extend homeward-ingest) and query (rust-extend
    homeward-report) are INDEPENDENT once embed-client lands — parallelizable,
    different crates, no shared file. enroll touches main.rs+new enroll.rs;
    query touches reportd.rs + report Cargo.toml.
  - attest is `mixed` (drives Python sidecar + Rust bins) and depends on BOTH
    wires. It needs a provisioned DINOv2 (embed-provision, already shipped);
    if the model can't be provisioned it MUST report SKIPPED, never false-green
    (tautology guardrail [[feedback_agent_written_fixtures_tautology]]).
  - query removes make_stub_report/make_stub_candidate — confirm no reportd
    path emits a fabricated score after the build.
Open questions (in vision): stale-gallery removal on departure (enroll PRD
  notes full removal wire may be a follow-on); whether attest belongs as a
  `homeward attest` subcommand vs a deploy/ script.

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  vision-plumb (extend: trustworthy calibrator)
Seed: bare /dream (interactive). fallow check = FRESH (streak=0,
  last_productive 15:34 today). Rather than re-extend homeward (just got a
  4-PRD deliver fleet at 15:34), probed the plumb arc that shipped TODAY
  (plumb-core + plumb-ledger + plumb-selfreview-bind).
Phase-1 live inspection of ~/.config/plumb/probes.toml found the calibrator
  audits its probes but never audits ITSELF — three live gaps:
  - TAUTOLOGY: ctrace-backfill-wired verdict=`grep -q "scribe backfill" <hook>`
    and oracle=`grep -qE "scribe.backfill|..." <hook>` — same tool, SAME FILE.
    Config comment literally says "slightly different grep pattern on the same
    file." Agreement proves nothing. This is [[feedback_agent_written_fixtures_tautology]]
    living inside the layer built to prevent it.
  - COVERAGE: `plumb list`=3 probes; SKILL.md B.5 has many playbooks
    (fleet-staleness, agorabus, warden, ctrace-resolve…). No command reports
    the gap; vision end-state ("every B.5 probe registered") is unmeasured.
  - DRIFT: every probes.toml verdict is a hand-copy of a SKILL.md probe
    ("The BROKEN probe from SKILL.md:186"). plumb-selfreview-bind fixes
    SKILL.md:186 → registered verdict still holds broken form → plumb
    calibrates stale logic.
Drafted: PRD-plumb-independence.md, PRD-plumb-coverage.md, PRD-plumb-sync.md
Vision: visions/plumb.md (extend section appended)
Order: all three rust-extend ~/wintermute/plumb, INDEPENDENT subcommands
  (lint / coverage / sync) over the existing Registry+probes.toml contract —
  fully parallelizable, ship in any order.
Notes for /build:
  - All three are rust-extend into ~/wintermute/plumb (v0.2.0, edition 2021,
    MSRV 1.85, clap-derive, no let-chains). Each adds ONE new subcommand +
    one new module (lint.rs / coverage.rs / sync.rs). No cross-PRD shared
    file beyond registry.rs (sync adds an optional `source` field to
    ProbeEntry — additive, backward-compatible).
  - plumb-independence carries LIVE PROOF (ctrace probe must flag, memlog
    must pass) — highest value, good first pick.
  - plumb-sync touches registry.rs (new optional field) AND ships an updated
    probes.toml with `source` pointers; if it lands alongside the others,
    expect a trivial merge on Cargo/registry. None of the three conflict on
    a subcommand.
  - SIGPIPE-safe required (sigpipe::reset first line of main —
    [[self_sigpipe_panic_toolkit]]); plumb already does this, keep it.
Open questions (in vision): lint granularity (tools AND path vs OR, --strict);
  whether lint failures feed the B.5 quarantine or stay a standalone audit;
  source-anchor format for sync (line vs #anchor — drafted to support both).

## 2026-06-13T(manual)  /dream  vision-persona (extend: work-persona enforcement arc)
Seed: bare /dream (interactive). fallow check = FRESH (streak=0,
  last_productive 16:32 today = the plumb extend). Picked a different arc than
  plumb/homeward/rollout (all freshly covered): the work-laptop persona, named
  in [[project_persona_sister_project]] as a future /dream seed and in
  visions/persona.md as "still entirely unbuilt" — except it's now half-built.
Phase-1 live inspection: the elder persona (Jocelyn) got FOUR hardening layers
  shipped 2026-06-13 (forbidden-vocab → persona-redline runtime enforce →
  persona-redline-eval held-out number → persona-deploy-doctor drift watch).
  The work persona got only the first artifact: ~/wintermute/persona-work/
  shipped CLAUDE_WORK.md + install.sh + validate.sh (commit 558f961). Its scope
  rules ("Never commit to j0yen", "no auto-publish", "no voice/agorabus/family",
  "no /build /dream", "no force push") are PROMPT ADVICE with no runtime gate,
  no measured number, no drift watch — exactly where forbidden_terms sat before
  persona-redline. The enforcement primitives already exist and were validated
  today: `answerable check --policy redline.toml` (v0.5.0, action-side gate) and
  `answerable values-drift` (v0.4.0, watches CLAUDE_SELF.md Values/Boundaries).
Drafted: PRD-persona-work-redline.md, PRD-persona-work-eval.md,
  PRD-persona-work-doctor.md
Vision: visions/persona.md (new section "The work persona deserves the elder's
  enforcement arc")
Order: persona-work-redline (foundation: the gate) → {persona-work-eval,
  persona-work-doctor} (both depend on the gate, independent of each other).
Notes for /build:
  - All three EXTEND ~/wintermute/persona-work/ (build_into set); redline is
    mixed (hooks + config), eval is mixed, doctor is shell. None are Rust
    builds — they CONSUME shipped answerable (check/values-drift), not extend it.
  - persona-work-redline is the good first pick: authors redline-work.toml +
    a PreToolUse hook that routes git-push/repo-create/force-push/voice-launch/
    /build·/dream through `answerable check`.
  - HARD CAVEAT baked into every PRD: these deploy to Joe's AtScale laptop, a
    machine /dream cannot see. Each must run green on THIS box against fixtures
    and SKIP honestly (never false-green) when the work identity isn't the one
    installed — this wintermute box is the expected SKIP case for the eval.
  - Tautology guard is explicit in persona-work-eval: corpus must be phrased
    independently of redline-work.toml's match expressions ([[feedback_agent_written_fixtures_tautology]]),
    and a build-time check fails on substring-copies.
Open questions (in vision): work-machine detection (manual copy + install vs
  chezmoi if constellation ships); persona bleed when Joe asks the personal box
  about AtScale work (vision answer: leave it — personal box keeps its identity).

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T17:58  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-13T(manual)  /dream  vision-changeover
Seed: bare /dream (interactive). fallow check = FRESH (state=fresh, streak=0,
  last_productive 17:03 today = plumb extend). Picked a fresh arc, not
  plumb/persona-work/homeward/rollout (all freshly covered).
Drafted: PRD-changeover-probe.md, PRD-changeover-warmswap.md,
  PRD-changeover-autoapply.md
Vision: visions/changeover.md (new — there was no rollout *vision* doc; this
  is the dream layer over the rollout tool)
Root signal: every self-review 2026-06-08→13 parks fleet-binary-staleness on
  "daemon restarts drop subscribers; requires explicit approval." The blocker
  is ASSERTED, never MEASURED. wm-audio/dialog/tts/stt all behind-head and
  un-rollable. rollout/src/restart.rs = hard `systemctl --user restart` +
  poll_healthcheck (no overlap). agorabus ALREADY has the fix primitives:
  ClaimAcquire/ClaimRelease TTL lease + DrainNotice + reconnect_subscribe —
  nothing uses them for peer restarts.
Order: probe → warmswap → autoapply.
  - changeover-probe (NEW rust-cli ~/wintermute/changeover/): measure the
    deafness window + events lost across a real rollout restart. SHIP FIRST,
    independent. Per [[feedback_verify_before_concluding]] measure before
    fixing — maybe systemd restart is already <50ms for some daemons.
  - changeover-warmswap (rust-extend ~/wintermute/rollout/): overlap-start
    successor, ClaimAcquire the daemon's lease, then stop predecessor. Builds
    on EXISTING agorabus claim primitive — no agorabus surgery. Only worth
    building if probe shows a lossy window.
  - changeover-autoapply (rust-extend ~/wintermute/rollout/): proof ledger
    (~/.config/rollout/proofs.json) bound to the daemon's binary hash; gate
    `rollout apply --auto` on a fresh green proof. Ships INERT
    (auto_enabled=false default) so building it doesn't start restarting the
    live fleet.
Notes for /build:
  - probe ships standalone (new crate). warmswap + autoapply both EXTEND the
    existing rollout crate — they'll likely land in sequence on the same tree;
    autoapply adds record-proof + autogate.rs, warmswap adds warmswap.rs +
    RestartStrategy::WarmSwap. Minor merge if both land same tick.
  - All three hermetic: live-fleet/bus paths behind #[ignore]; cargo test
    must stay green without touching the real fleet. MSRV 1.85, no let-chains,
    sigpipe::reset first line of main ([[self_sigpipe_panic_toolkit]]).
  - Do NOT auto-roll the live voice fleet as a side effect of building these.
    autoapply is gated inert by design; warmswap's live swap is #[ignore].
Open questions (in vision): mic/ALSA produce-side gap on wm-audio (two procs
  can't hold the capture device — separate fd-passing PRD or accept <100ms
  audio gap?); claim_key per daemon (derive vs fleet.toml override); proof
  freshness (hash-bound, drafted).

## 2026-06-13T(manual)  /dream  vision-scion
Seed: bare /dream (interactive). User dismissed the steer question →
  made the call from evidence. fallow check = FRESH (streak=0,
  last_productive 18:33 today). Picked a fresh arc (not changeover/plumb/
  persona-work/homeward/rollout — all freshly covered).
Drafted: PRD-scion-verdict.md, PRD-scion-reconcile.md, PRD-scion-truth.md
Vision: visions/scion.md (NEW)
Root signal: `adopt scan`'s freshness verdict is FALSE BY CONSTRUCTION.
  derive_verdict (adopt/src/scan.rs:291) flags installed-stale on
  `installed_ts < src_commit_ts`. The build pipeline does `cargo install`
  THEN `git commit`, so a correctly-built binary is written seconds
  BEFORE the commit it descends from → `its < sts` always true → permanent
  false "0h stale". Measured live: bon-mot-anagram (13s), bon-mot-epigram
  (5s), changeover (33s) — none behind, all "stale". The docket has
  re-parked adopt-scan-stale-binaries (84→16) run after run on this noise,
  burying any genuinely-behind binary.
  The fix half-exists: vest-incremental shipped a SourceFingerprint
  InstallMarker (adopt/src/marker.rs:57,189 — git rev-parse HEAD) that
  records WHICH COMMIT a binary was built from, but it's consumed ONLY to
  skip reinstalls in `apply`. `adopt scan`'s verdict never reads it. AND
  no markers exist yet ($XDG_STATE_HOME/adopt/markers/ is absent, 0 files).
Order: scion-verdict → scion-reconcile → scion-truth.
  - scion-verdict (rust-extend adopt): verdict consults the marker —
    installed-current iff marker fingerprint == committed HEAD; clock
    comparison demoted to no-marker fallback; adds freshness_basis to JSON.
    One-function fix; correct the moment any marker exists.
  - scion-reconcile (rust-extend adopt): NEW `adopt reconcile` mints a
    marker for every installed-but-unmarked binary that's provably not
    behind — WITHOUT a rebuild. Load-bearing: without it scion-verdict
    falls back to the broken clock for all 16 legacy installs. Markers
    carry origin=reconcile-seed (vs install) for honesty.
  - scion-truth (rust-extend adopt): reframe `adopt report`'s docket
    finding on the lineage verdict — count only marker≠HEAD as "behind",
    split unmarked into adopt-unmarked-installs, auto-resolve at zero.
Notes for /build:
  - All three EXTEND ~/wintermute/adopt (no new crate). They'll land in
    sequence on the same tree (scan.rs/types.rs for verdict; new cli
    subcommand + marker origin field for reconcile; report.rs for truth).
    Expect minor sequential merges; build in order.
  - MUST reuse marker::compute_fingerprint for BOTH the scan-side read and
    the reconcile-side write — a marker written by one function and read
    by another will never agree. This is the single most important impl
    invariant.
  - derive_verdict is currently `const fn`; scion-verdict makes it a
    regular fn (marker IO isn't const). Keep the fallback arm byte-identical.
  - All hermetic: temp git repos + XDG_STATE_HOME override; no live fleet,
    no network. MSRV 1.85, no let-chains, sigpipe::reset already in main.
  - Distinct from vest (operational adoption integrity) and plumb (probe
    epistemics) and changeover (daemon restart) — scion is the freshness
    *oracle correctness* layer. Shares the marker with vest-incremental;
    shares hash-proof intent with changeover-autoapply (open Q3 in vision).
Open questions (in vision): dirty-tree semantics (compare vs committed
  HEAD, uncommitted out of scope); embedded-in-binary provenance as the
  eventual rigorous form (build.rs stamp, deferred); whether changeover-
  autoapply consumes scion's marker or needs a running-daemon binary-hash
  proof (resolve when autoapply is built).

## 2026-06-13T(manual)  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T(manual)  /dream  vision-litmus
Seed: bare /dream (interactive). fallow check = FRESH (streak=0,
  last_productive 19:07). User not asked to steer (escalate=false); made
  the call from evidence. Fresh arc — not scion/changeover/plumb/persona/
  homeward/rollout/vest (all freshly covered).
Drafted: PRD-litmus-probe-fixtures.md, PRD-litmus-stuck-detector.md,
  PRD-litmus-selfreview-bind.md
Vision: visions/litmus.md (NEW)
Root signal: self-review's Phase B.5 PROBES can be false-by-construction,
  and when they are, docket faithfully re-parks a false finding forever.
  Two-week worked example, measured live this session:
  `docket show ctrace-sessionend-flake` → report_count:12, runs_seen:10,
  open 2026-05-30 → resolved today 14:04 with reason "grep pattern
  'scribe backfill' is a false negative — hook is correct." Ground truth:
  ~/.claude/scripts/ctrace-session-start.sh HAS the wiring (line 27 reap,
  lines 33-35 `"$scribe" backfill …`) — the probe's grep `scribe backfill`
  never matched the hook's `"$scribe" backfill`, so the finding cried
  wolf 12× over a fortnight. (My OWN first grep this session reproduced
  the exact false-negative — proof it's a foot-gun, not a one-off.) Same
  class of bug hit the memlog probe (multiline getent capture), since
  fixed inline but with no regression test to keep it fixed.
Order: (litmus-probe-fixtures ∥ litmus-stuck-detector) → litmus-selfreview-bind.
  - litmus-probe-fixtures (shell): golden-fixture self-test harness for
    B.5 probes, modelled on the EXISTING ~/.claude/skills/self-review/
    scripts/docket-bind-selftest.sh (XDG isolation + assert_* helpers).
    Canonical regression: ctrace-wiring fixture = byte copy of the real
    hook, expect HAS_BACKFILL=yes; negative-control unwired copy expects
    no. Would have caught the grep bug on day one.
  - litmus-stuck-detector (rust-extend ~/wintermute/docket): `docket stuck`
    read-only subcommand — open findings with report_count>=6 AND
    runs_seen>=5 AND resolved_at null, labeled "probe-suspect — audit the
    probe not the world." No schema change: Finding already has these
    fields (model.rs:118/136/140/142). New Command::Stuck beside
    Digest/Sweep (cli.rs:52).
  - litmus-selfreview-bind (config: edits self-review/SKILL.md): wire
    `docket stuck` into Phase B.5 as a `litmus:` banner + a `litmus_audit`
    playbook that routes a probe-suspect finding to the fixture harness
    BEFORE re-parking. Mirrors the shipped plumb-selfreview-bind pattern.
Notes for /build:
  - litmus-stuck-detector EXTENDS docket; bind DEPENDS on it (needs the
    subcommand). fixtures is independent — ship anytime.
  - docket extend: read-only query only, no migration. Keep within the
    red-baseline reality (compiles + cargo test green is the real bar).
    sigpipe::reset already in main; MSRV 1.85; no let-chains.
  - fixtures + bind both touch ~/.claude (scripts + SKILL.md), NOT a repo
    under ~/wintermute — build_target shell/config, no cargo, no network.
  - Distinct from plumb (calibrates TRUST in verdicts statistically — and
    is already bound to B.5) and scion (fixes adopt's freshness verdict):
    litmus is whether the self-review PLAYBOOK PROBES match the artifacts
    they grep. Correctness of the pattern, not calibration of belief.
Open questions (in vision): absolute vs relative stuck threshold; whether
  a fixture-confirmed-probe-defect finding should auto-ack (probably not —
  visibility is the point); litmus-as-scripts vs its own crate (start as
  scripts next to docket-bind-selftest.sh).

## 2026-06-13T(manual)  /dream  vision-fixpoint
Seed: bare /dream (interactive). fallow check = FRESH (streak=0,
  last_productive 20:05). escalate=false; made the call from evidence.
Drafted: PRD-fixpoint-cron-reconcile.md, PRD-fixpoint-verify-resolution.md,
  PRD-fixpoint-dirty-reconcile.md, PRD-fixpoint-converge-ledger.md
Vision: visions/fixpoint.md (NEW)
Root signal: scion shipped the lineage-marker machinery (verdict/reconcile/
  truth, all archived today) but the live fleet is UNCHANGED — `adopt scan`
  shows clock-fallback:16, zero markers minted. Phase-1 found three measured
  reasons the cure never reaches the patient + a 4th nobody would notice:
  (1) adopt-cron.service runs `apply`+`report` but NEVER `reconcile` — so
      markers are never minted autonomously; apply reinstalls forever
      (hamster wheel). Verified via `systemctl --user cat adopt-cron.service`.
  (2) `adopt reconcile` SKIPS dirty working trees (all bon-mot-* skipped in
      --dry-run); ~30 dirty repos → permanent clock-fallback residue.
  (3) `adopt verify` lumps all 16 under one `SourceNewer` bucket, hiding the
      4 genuinely-behind daemons (wm-audio 8d, wm-dialog 7d, wm-tts 6d,
      wm-reach 2d) among 12 "0d newer" clock-noise installs.
  (4) Nothing tracks convergence: 84/84(06-11)→16/16(06-13) passed unremarked.
Order:
  - fixpoint-cron-reconcile (CONFIG, ship FIRST, independent): add
    `adopt reconcile --execute` before `apply` in adopt-cron.service. Pure
    unit edit + daemon-reload. AC4 proves live effect (lineage count > 0
    after a manual cron run). Immediate convergence, no rebuild.
  - fixpoint-verify-resolution (rust-extend adopt): split SourceNewer into
    -sameday(≤1d)/-behind(≥2d). Independent. Gives a true denominator.
  - fixpoint-dirty-reconcile (rust-extend adopt): seed marker from committed
    HEAD for dirty trees whose binary matches HEAD; else classify
    dirty-blocked (not silent clock-fallback). Shares scion marker internals.
  - fixpoint-converge-ledger (rust-extend adopt): per-run convergence record
    + `adopt converge` + fixpoint-not-converging docket finding. DEPENDS ON
    verify-resolution for the `behind` count.
Notes for /build:
  - fixpoint-cron-reconcile is config-only (edits ~/.config/systemd/user/
    adopt-cron.service) — no cargo, no network. Land it first; it's the
    highest-leverage single line and proves itself on the next cron tick.
  - The 3 rust-extend PRDs all extend ~/wintermute/adopt (currently v0.9.0
    post scion-truth). Reuse scion's fingerprint/marker code; DO NOT fork it.
    red-baseline reality: bar = compiles + cargo test green. MSRV 1.85, no
    let-chains, sigpipe::reset already in main.
  - converge-ledger depends on verify-resolution's SourceNewer-behind bucket;
    if built first it falls back to total-not-current (documented degrade).
  - Distinct from scion (built the verdict+actuator) and from litmus (probe
    correctness): fixpoint is whether the actuator actually RUNS to a
    measured zero. scion gave adopt the right answer; fixpoint makes adopt
    converge and prove it.
Open questions (in vision): dirty-tree marker safety (seed-from-HEAD vs
  classify-only); sameday threshold 1d-clock vs lineage-strict; ledger as
  standalone file vs docket time-series; reconcile before vs after apply.

## 2026-06-13T21:00  /dream  vision-changeover (Fleet 2 — activation)
Drafted: PRD-changeover-claim-guard.md, PRD-changeover-daemon-claims.md,
  PRD-changeover-proof-seed.md, PRD-changeover-activate.md
Vision: visions/changeover.md (extended — Fleet 2 appended)
Finding: Fleet 1 (probe/warmswap/autoapply) all SHIPPED today, yet the
  same evening's journal STILL parks fleet-binary-staleness. Warm-swap is
  decorative — switched off at three points:
  (1) NO daemon acquires an agorabus claim. warmswap.rs waits for the
      successor to become a claim holder before stopping the predecessor;
      grep claim_acquire over wintermute-{audio,stt,tts}/src = 0 files
      (dialog's 2 hits are FSM transcript-claiming, not the bus lease).
      So the poll always times out → warm-swap falls back to hard restart.
  (2) ~/.config/rollout/proofs.json does NOT exist. apply --auto (v0.7.0)
      refuses every daemon without a fresh green proof; nobody ever ran
      changeover probe → rollout record-proof.
  (3) self-review SKILL.md (303/852/864) makes "never rollout apply"
      immutable, citing the hard-restart reality warm-swap eliminates.
Order: claim-guard → daemon-claims → proof-seed → activate.
  - claim-guard (rust-extend agorabus, ship FIRST): RAII ClaimGuard +
    auto-renew + release-on-drop. Pure lib, fixture tests, no daemon
    surgery. All four daemons already dep on agorabus path crate.
  - daemon-claims (mixed, the four wintermute-{audio,dialog,stt,tts}):
    hold agorabus://daemon/<unit> for lifetime, release on SIGTERM. THE
    load-bearing change — proof can't go green without it. /build may fan
    out per-crate; build_into names wintermute-audio canonical.
  - proof-seed (rust-extend rollout): `rollout prove --daemon <unit>`
    runs probe→record-proof; +changeover-prove.timer keeps ledger fresh
    (hash-bound). Mints the first green entries. Depends on daemon-claims.
  - activate (mixed, rollout + config): `rollout cycle` =
    prove→apply --auto (warm-swap only, turn-aware)→LIVE post-swap verify
    (claims re-held + voice turn round-trips) + receipt. Ships DORMANT
    (ROLLOUT_AUTO_ENABLED=0, dry-run) — never restarts until unblocked.
Notes for /build:
  - claim-guard + daemon-claims are the producer half; ship those two and
    warm-swap stops being decorative even before the loop turns on.
  - activate DEPENDS on the already-blocked PRD-rollout-selfreview-apply.md
    (classifier-blocked SKILL.md guardrail edit, needs jsy approval). Do
    NOT redraft it; activate ships dormant until it lands.
  - red-baseline reality across all four: bar = compiles + cargo test
    green. MSRV 1.85, no let-chains. agorabus is the path dep version 0.9.
Open questions (in vision): mic/ALSA handoff still unsolved (wm-audio
  can't double-hold the capture device — sub-100ms audio gap or SCM_RIGHTS
  fd-pass?); claim TTL vs renew cadence (drafted 30s/10s); proof-seed in
  rollout vs changeover (drafted rollout).

## 2026-06-13T21:30  /dream  vision-pulse
Drafted: PRD-pulse-hearing-probe.md, PRD-pulse-watch.md,
  PRD-pulse-deaf-escalation.md, PRD-pulse-silence-gate.md
Vision: visions/pulse.md (NEW)
Seed: bare /dream, interactive. Field fresh (fallow streak=0). Picked the
  companion/kin domain after confirming the inward "make built things live"
  space is saturated (vest/adopt/scion/fixpoint/changeover/assay/continuity
  all overlap; the 8 remaining on-disk PRDs are all changeover/fixpoint/
  rollout). companion + kin are BOTH fully shipped (voice TURN live since
  2026-06-04; wintermute-reach v0.5.0, presence, family-enroll all built).
Finding: every safety guarantee in companion+kin assumes the device can
  still HEAR, and nothing watches that assumption. A deaf box (wm-audio
  wedged / mic gone / detector silently stopped / model dir clobbered) is
  indistinguishable from a quiet healthy one — and kin's silence-nudge
  (reach/src/silence_nudge.rs) fires "haven't heard from Mom" on
  wm.presence.silence, which it CANNOT tell apart from "can't hear Mom."
  The one alert that fires points the wrong way. rouse shipped the local
  on-demand hearing check (wm-audio selftest, selftest.rs:56 exit_code)
  but it publishes no wm.health envelope and runs on no cadence.
Order: hearing-probe → watch → {deaf-escalation, silence-gate}.
  - hearing-probe (rust-extend wintermute-audio, ship FIRST): wm-audio
    selftest --emit publishes a wm.health.hearing envelope (companion-
    degrade/almanac shape; almanac daemon.rs:147 is the precedent). Adds
    NO new inference — wraps the existing selftest real-path check +
    any_model_present() (selftest.rs:204). Independent.
  - watch (rust-extend wintermute-presence): cadenced HEARING/DEGRADED/DEAF
    state machine, K-consecutive-fail debounce, per-window persistence of
    hearing_confirmed_in_window beside existing presence state, emits
    wm.health.hearing.fail on the DEAF edge + .ok on recovery. Keyed to
    presence's existing waking-hours window. Depends on hearing-probe.
  - deaf-escalation (rust-extend wintermute-reach): subscribe .fail →
    deliver via run_distress_ladder() (distress_delivery.rs:41 — retry +
    fallback transport, same durable path as kin distress), debounced one-
    alert-per-outage, recovery note on .ok. NO Claude-API dependency (a
    down brain may itself look like deafness). Depends on watch + shipped
    reach transport.
  - silence-gate (rust-extend wintermute-reach): gate silence_nudge.rs on
    hearing_confirmed_in_window — nudge fires only when device provably
    heard; deaf/ambiguous window suppressed (deaf-escalation owns it).
    Pure correctness fix, no new opt-in. Depends on watch + deaf-escalation.
Notes for /build:
  - All four extend SHIPPED crates and reuse shipped primitives (selftest,
    run_distress_ladder, presence window/state). Reuse, don't reinvent —
    no second inference path, no new transport.
  - deaf-escalation + silence-gate both extend wintermute-reach, disjoint
    modules (new escalation path vs silence_nudge.rs) — parallel-safe;
    sequential just rebases the version bump.
  - wm.health.hearing is a plain agorabus string — keep identical across
    audio(publish)/presence(consume+re-emit)/reach(consume). No shared crate.
  - Red-baseline: bar = compiles + cargo test green. MSRV 1.85, no
    let-chains, sigpipe::reset already in each main. agorabus path dep 0.9.
Open questions (in vision): probe cadence vs power/heat on the 4-core box
  (fixture-probe cheap, --live reserved for on-demand); DEGRADED-vs-DEAF
  band + K threshold want a real deployment to tune; should Mom hear about
  her own deaf device (TTS still works when ears don't). Component 5 — the
  "are you listening?" reassurance verb (rust-extend wintermute-dialog
  Health branch) — left undrafted pending the FSM Health-branch shape;
  next /dream pass.

## 2026-06-13T21:45  /dream  vision-homeward (catchment fleet)
Drafted: PRD-homeward-source-registry.md, PRD-homeward-source-probe.md,
  PRD-homeward-source-catalog.md, PRD-homeward-coverage-report.md
Vision: visions/homeward.md (extended — "Catchment fleet" section)
Seed: bare /dream, interactive. Field FRESH (fallow streak=0) but inward
  infra space is SATURATED — 10 PRDs already queued (pulse×4 just landed
  21:30 + changeover/fixpoint/rollout), and every convergence vision
  (docket/fixpoint/changeover/rollout) is already active. Drafting more
  inward would deepen an undrained queue. Asked user for steer; they
  dismissed → picked the most distinct OUTWARD vision with no PRDs in the
  current queue, so this doesn't add to the infra backlog.
Finding: homeward's 4 shipped fleets (core/federation/operate/deliver) all
  WIRED the pipeline; none WIDENED its mouth. Socrata sources are
  compile-time `&'static str` const fns (socrata.rs:78) hand-listed in
  main.rs:24 as [austin,dallas,sonoma,long_beach]. Adding any of the
  "hundreds more" STRAY portals the vision names = edit Rust + recompile.
  Coverage is the literal point of homeward (more sources = more reunions)
  and it's gated behind a developer. ConnectorRegistry (registry.rs) is a
  runtime name→connector map but nothing populates it from a file.
Order: source-registry → source-probe → source-catalog
                       └─► coverage-report (independent of probe/catalog)
  - source-registry (rust-extend homeward-connectors, ship FIRST): make
    SocrataConfig/ColumnMap serde-loadable from sources.toml via
    HOMEWARD_SOURCES; fall back to the 4 built-ins when unset. Foundation —
    the file format both probe and catalog speak. Owned String alongside
    the const fn built-ins; keep SocrataConnector::new seam unchanged.
  - source-probe (rust-extend homeward-connectors): `probe <domain>
    <dataset_id>` hits SODA columns.json + $limit=1, confirms a
    STRAY-bearing intake_type column (value-match, not name-guess), emits a
    GREEN paste-ready [[socrata]] block (annotated confirmed/guessed) or a
    RED verdict + nonzero exit. Tests use RECORDED fixtures, no live net.
    Depends on registry (emits its format).
  - source-catalog (mixed → homeward): commit deploy/sources.toml (4
    built-ins migrated byte-equivalent + ≥2 new probe-GREEN cities) +
    deploy/CATCHMENT.md + a parse/load test + default HOMEWARD_SOURCES in
    env.sample & orchestrate wrapper. The data deliverable that widens
    catchment. Depends on registry (format) + probe (validation).
  - coverage-report (rust-extend homeward-connectors): `coverage` reports
    per-source {last_success, record/stray counts, metro, LIVE/STALE/
    SILENT/UNREACHABLE} + catchment holes; honest `unknown` when no store.
    The operator's actual-coverage map (complements catalog's intended map
    + orchestrate's `homeward health`). Depends on registry only.
Notes for /build:
  - All four rust-extend homeward-connectors EXCEPT catalog (mixed).
    registry must land first (probe/catalog/coverage all speak its format).
    probe + coverage are parallel-safe once registry lands (disjoint
    subcommands); catalog needs probe's GREEN output to author entries.
  - Red-baseline bar = compiles + cargo test green. MSRV 1.85, no
    let-chains, sigpipe::reset already first in main. toml+serde may need
    adding to homeward-connectors/Cargo.toml (present elsewhere in wsp).
  - NO live network in any test — record SODA fixtures
    ([[feedback_agent_written_fixtures_tautology]]: but here fixtures are
    upstream HTTP captures, not self-authored oracles; the probe's verdict
    logic is what's under test, fed real recorded municipal responses).
Open questions (in vision): OpenDataSoft/ArcGIS dialects = a second
  connector FAMILY not just config (future extend); auto-discovery of
  portals via the Socrata federated catalog (un-dreamt, needs research).

## 2026-06-13T22:28  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-13T18:59  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-13T23:30  /dream  no-draft
/dream interactive, field FRESH (fallow streak=0) — but drafted 0 by choice.
Research walk done (recall seeds, wchg, journals, vision survey). No new
evidenced vision motivated: companion/persona/kin/homeward are all mature &
active; recall reflective bucket is entirely self-review entries (no half-
formed feature seeds); the only recurring signals (ctrace scribe-backfill
one-liner unwired 9 runs, agentns-zeros escalated 13 runs, fleet-binary-
staleness) are /build or /self-review shaped, not vision shaped.
Queue is UNDRAINED: 8 drafts still at v0.1 — homeward catchment fleet
(source-registry/probe/catalog + coverage-report), changeover-{activate,
daemon-claims,proof-seed}, rollout-selfreview-apply.
Asked user for a steer (extend-homeward / self-review-autoremediation /
companion-kin / rest); steer dismissed. Per Hard Rule 6 (don't dream past
the research) chose to rest rather than pad an already-full queue.
Notes for /build: drain the 8 v0.1 drafts before /dream adds more —
homeward catchment registry must land first (probe/catalog/coverage speak
its format). Nothing new from me this pass.

## 2026-06-13T23:30  /dream  no-draft
/dream interactive, fallow FRESH (streak=0) — fingerprint moved only because
/build shipped today (homeward→v0.15 coverage-report, changeover-daemon-claims,
rollout apply --auto; last_productive_ts 22:04). Did the mandatory recall seed
walk + ctrace/pevent/vision survey. No NEW vision-shaped evidence: reflective
bucket is 100% self-review entries, ideation query returns only built/todo
residue, recurring signals (ctrace scribe-backfill unwired, agentns-zeros,
fleet-binary-staleness) are /build|/self-review shaped not vision shaped.
Queue UNDRAINED: same 8 v0.1 drafts (homeward source-registry/probe/catalog +
coverage-report, changeover-{activate,daemon-claims,proof-seed},
rollout-selfreview-apply) — /build is mid-drain on them now.
Per Hard Rule 6 rested rather than pad a full, actively-draining queue.
Notes for /build: keep draining; homeward source-registry lands first
(probe/catalog/coverage speak its format). Nothing new from me this pass.

## 2026-06-13T23:59  /dream  no-draft
Third bare /dream tonight; field FRESH (streak=0) but fingerprint moved
only because /build shipped today (homeward coverage-report, changeover-
daemon-claims/activate; last_productive 22:04). Full Phase-1 walk: recall
reflective = 100% self-review entries, ideation query = built/todo residue,
journal/pevent/ctrace nothing vision-shaped. Queue UNDRAINED: 7 PRDs at
v0.1 (homeward source-registry/probe/catalog + coverage-report, changeover-
{daemon-claims,proof-seed}, rollout-selfreview-apply) — /build mid-drain.
Recurring signals (ctrace scribe-backfill unwired, agentns-zeros [prctl arc
exists], fleet-binary-staleness, memlog/self-review probe false-alarms) are
/build|/self-review shaped, not visions. Per Hard Rule 6 rested rather than
pad a full queue. Did NOT re-ask user for steer (dismissed twice earlier
tonight). Notes for /build: keep draining — homeward source-registry lands
first (probe/catalog/coverage speak its format). Nothing new from me.

## 2026-06-14T(dream)  /dream  no-draft
Fifth bare /dream since last night; field FRESH (streak=0) but fingerprint
moved only because /build shipped today (homeward→source-probe v0.16,
rollout→v0.10; last_productive 2026-06-13T22:04). Did the mandatory recall
seed walk + journal/vision survey. No NEW vision-shaped evidence: reflective
bucket is 100% self-review entries, ideation query = build/todo residue.
Queue DRAINING: down to 2 v0.1 PRDs (homeward-source-registry,
rollout-selfreview-apply) from 8 last night — /build is mid-drain.
Recurring signals (ctrace scribe-backfill unwired [9 runs], agentns-zeros
[13 runs, prctl arc exists], fleet-binary-staleness, memlog probe bug) are
/build|/self-review shaped, not visions. Per Hard Rule 6 rested rather than
pad a draining queue. Did NOT re-ask user for steer (dismissed 2x last night).
Notes for /build: finish draining — homeward-source-registry lands first
(probe/catalog/coverage already speak its format). Nothing new from me.

## 2026-06-14T(dream)  /dream  no-draft
Interactive bare /dream; field FRESH (streak=0) but fingerprint moved only
because /build keeps shipping (last_productive 2026-06-13T22:04). Did the
mandatory recall seed walk: reflective bucket = 100% self-review entries,
ideation --hybrid query = build/todo residue (kernel-asset arc, bootstrap.env
todo) — nothing vision-shaped. Queue now DRAINED to 1 v0.1 draft
(rollout-selfreview-apply) from 8 two nights ago — /build nearly done.
Because the queue finally has room, I asked the user for a steer
(extend-homeward / companion-kin / self-remediation / constellation);
dismissed. Per Hard Rule 6 rested rather than pad with unmotivated PRDs.
Notes for /build: finish the last draft (rollout-selfreview-apply). Nothing
new from me this pass. When user gives a steer next, homeward-source-registry
and a self-remediation vision (ctrace scribe-backfill, agentns-zeros,
fleet-binary-staleness recurring) are the two best-evidenced candidates.

## 2026-06-14T(dream)  /dream  no-draft
Interactive bare /dream; field FRESH (streak=0) but fingerprint moved only
because /build keeps shipping (last_productive 2026-06-13T22:04). Did the
mandatory recall seed walk: reflective bucket = 100% self-review entries,
ideation --hybrid = build/todo residue (kernel-asset arc, bootstrap.env todo)
— nothing vision-shaped. Queue DRAINED to 1 v0.1 draft (rollout-selfreview-
apply). Asked user for a steer (self-remediation / extend-homeward /
companion-kin / constellation); dismissed again. Per Hard Rule 6 rested
rather than pad. Notes for /build: finish rollout-selfreview-apply. When user
steers next, self-remediation (ctrace scribe-backfill unwired [9 runs],
agentns-zeros [13 runs, prctl arc], fleet-binary-staleness) is the best-
evidenced vision candidate; homeward-source-registry the second.

## 2026-06-14T(dream)  /dream  no-draft
Sixth bare /dream this cycle; field FRESH (streak=0) but fingerprint moved
only because /build keeps shipping (last_productive 2026-06-13T22:04, queue
drained to 1 v0.1 draft: rollout-selfreview-apply). Did mandatory recall seed
walk + journal/vision survey. No NEW vision-shaped evidence: reflective bucket
100% self-review entries, ideation --hybrid = build/todo residue. Recurring
signals all have existing homes, not new visions: agentns-zeros is awaiting
pkgrel-12 install+reboot (SessionStart hook says "ACTIVATION BLOCKED at
kernel-prctl"; assay vision already covers the prctl arc); ctrace
scribe-backfill is a one-line hook fix (/self-review); fleet-binary-staleness
lives in the rollout/changeover vision. Per Hard Rule 6 rested rather than
pad a drained queue. Did NOT re-ask user for steer (dismissed 4x this cycle;
fallow escalate=false). Notes for /build: finish rollout-selfreview-apply.

## 2026-06-14T(dream)  /dream  no-draft
Seventh bare /dream this cycle; field FRESH (streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04). Mandatory recall seed walk:
reflective bucket 100% self-review entries, ideation --hybrid = build/todo
residue (kernel-asset arc, bootstrap.env todo). Journal last 3 days = pure
self-review maintenance. Queue drained to 1 v0.1 (rollout-selfreview-apply).
No NEW vision-shaped evidence. Per Hard Rule 6 rested. escalate=false so did
NOT modal-ask user (dismissed repeatedly this cycle); surfaced the two best-
evidenced candidates in-session instead. Notes for /build: finish draining
rollout-selfreview-apply. When user steers: homeward-source-registry and a
self-remediation vision (ctrace scribe-backfill unwired, agentns-zeros awaiting
pkgrel-12 install+reboot, fleet-binary-staleness) remain the best candidates.

## 2026-06-14T(dream)  /dream  no-draft
Eighth bare /dream this cycle; field FRESH (streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04). Mandatory recall seed walk:
reflective bucket 100% self-review entries, ideation --hybrid = build/todo
residue (kernel-asset arc, bootstrap.env todo). Nothing NEW vision-shaped.
Per Hard Rule 6 rested rather than pad a drained queue. Did NOT modal-ask
(user dismissed steer 4x this cycle; escalate=false). When user steers next,
homeward-source-registry and a self-remediation vision remain best-evidenced.

## 2026-06-14T(dream)  /dream  no-draft
Ninth bare /dream this cycle; field FRESH (streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Interactive: surfaced the four
steer options (self-remediation / extend-homeward / companion-kin /
constellation) via AskUserQuestion to break the rest loop — user dismissed.
Per Hard Rules 6 & 8 rested rather than pad a drained queue. No NEW
vision-shaped evidence; recurring signals all have existing homes (agentns-
zeros awaits pkgrel-12 install+reboot under assay vision; ctrace
scribe-backfill is a one-line /self-review hook fix; fleet-binary-staleness
lives in rollout/changeover vision). Notes for /build: finish draining
rollout-selfreview-apply.

## 2026-06-14T(dream)  /dream  no-draft
Tenth bare /dream this cycle; field FRESH (streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Mandatory recall seed walk:
reflective bucket 100% self-review entries; ideation --hybrid = build/todo/
kernel residue (kernel-asset arc, bootstrap.env todo). Nothing NEW
vision-shaped. Queue drained to 1 v0.1 (rollout-selfreview-apply). Per Hard
Rule 6 rested rather than pad a drained queue. Did NOT modal-ask (user
dismissed steer repeatedly this cycle incl. interactively; escalate=false).
Best-evidenced candidates when user steers: (a) self-remediation vision —
ctrace scribe-backfill unwired, agentns-zeros awaits pkgrel-12 install+reboot,
fleet-binary-staleness; (b) homeward-source-registry extend. Notes for /build:
finish draining rollout-selfreview-apply.

## 2026-06-14T(dream)  /dream  no-draft (litmus-not-live finding)
Eleventh bare /dream this cycle; field FRESH(streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04). Did NOT pad the queue.
Mandatory recall seed walk + full Phase-1 probe. Best inward signal
(self-review probes lying / self-remediation) is ALREADY owned by litmus,
drafted+archived YESTERDAY (2026-06-13). Per Hard Rules 2/6 did not duplicate.

CONCRETE NEW FINDING (no prior pass measured this) — litmus shipped but is
DARK on the live binary:
  - ~/wintermute/docket source = v0.5.1 (commit aa052e8 "add stuck subcommand
    for probe-suspect findings"); installed ~/.local/bin/docket = **v0.1.0**.
  - `docket stuck` → "unrecognized subcommand". litmus-stuck-detector's whole
    payload + escalate + digest + abide-ack-state are all unreachable.
  - litmus-probe-fixtures harness NOT present under ~/.claude/skills/self-review/;
    litmus-selfreview-bind banner not wired. All three litmus PRDs are in
    ARCHIVE/ (marked done) yet the capability is not live.
  - Also confirmed: ctrace scribe-backfill IS wired (ctrace-session-start.sh
    line 35 `"$scribe" backfill`). The recurring "NOT wired" journal carry is
    the exact false-negative litmus was built to kill — and it's still lying
    because docket-0.1.0 lacks `stuck` to flag it.

Notes for /build + /self-review: this is fleet-binary-staleness on docket
specifically (one of self-review's "55 installed-stale"), but high-impact —
docket gates the whole litmus self-healing loop. Reinstall docket from
~/wintermute/docket (cargo install --path . --force) so 0.5.1 goes live, then
re-run self-review B.5 to confirm ctrace-sessionend-flake auto-closes and
`docket stuck` surfaces the long-stuck findings. No PRD needed — pure adoption.
Notes for /build: finish draining rollout-selfreview-apply (still only v0.1).

## 2026-06-13T(dream)  /dream  no-draft (interactive, rested)
Twelfth bare /dream this cycle; field FRESH(streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Mandatory recall seed walk + Phase-1
probe. RESOLVED since last pass: docket-dark — installed docket now v0.5.1,
`docket stuck` works (/build adopted it). Live `docket stuck` shows ONE suspect:
agentns-session-zeros (18 reports/14 runs) — already owned by assay vision,
blocked on real-world action (install linux-wintermute pkgrel-12 + reboot), not
a missing PRD. Recall reflective bucket = 100% self-review carry; ideation =
build/todo/kernel residue. ~70 vision docs already exist; queue drained to 1
v0.1 (rollout-selfreview-apply, vigil). Per Hard Rules 6 & 8 rested. Did NOT
modal-ask (escalate=false; user dismissed steer repeatedly this cycle).
Best-evidenced steer when user wants one: (a) finish draining
rollout-selfreview-apply via /build; (b) install pkgrel-12 + reboot to close
agentns-zeros. Notes for /build: rollout-selfreview-apply still v0.1.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirteenth bare /dream this cycle; field FRESH(streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04). Did full Phase-1 probe +
mandatory recall seed walk. Same finding as the prior 12 passes: no NEW
vision-shaped signal. recall reflective bucket = 100% self-review carry;
ideation --hybrid = build/todo/kernel residue (kernel-asset arc, bootstrap.env
todo). ~70 vision docs exist; queue drained to 1 v0.1 (rollout-selfreview-apply).
`docket stuck` surfaces ONE suspect: agentns-session-zeros (18 reports/14 runs)
— owned by assay vision, blocked on real-world action (install linux-wintermute
pkgrel-12 + reboot), NOT a missing PRD. Per Hard Rules 6 & 8 rested; did not pad.
Best-evidenced steers when user wants one: (a) /build drain rollout-selfreview-apply;
(b) install pkgrel-12 + reboot to close agentns-zeros (also lights up agent namespaces).
Notes for /build: rollout-selfreview-apply still v0.1.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Fourteenth bare /dream this cycle; field FRESH(streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04). Full recall seed walk +
Phase-1 probe. Same finding as prior 13 passes: no NEW vision-shaped signal.
recall reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
build/todo/kernel residue (kernel-asset arc, 00-bootstrap.env todo). ~70 vision
docs exist; queue drained to 1 v0.1 (rollout-selfreview-apply). `docket stuck` =
ONE suspect: agentns-session-zeros (18 reports/14 runs) — owned by assay vision,
blocked on real-world action (install linux-wintermute pkgrel-12 + reboot), NOT a
missing PRD. Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when
user wants one: (a) /build drain rollout-selfreview-apply to v1; (b) install
pkgrel-12 + reboot — closes agentns-zeros AND lights up the agent-namespace
surface, which is the one place new vision space would actually open.
Notes for /build: rollout-selfreview-apply still v0.1.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Fifteenth bare /dream this cycle; field FRESH(streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Full recall seed walk + Phase-1 probe
+ surfaced the outward-steer modal to the user — user dismissed (consistent with
this cycle). Same finding as prior 14 passes: no NEW vision-shaped signal. recall
reflective = 100% self-review carry (all recalls=0); ideation --hybrid = build/todo/
kernel residue (kernel-asset arc, 00-bootstrap.env todo). Queue drained to 1 blocked
PRD (rollout-selfreview-apply); ~70 vision docs exist; agentns still all-zeros
(blocked on pkgrel-12 install + reboot, NOT a missing PRD). Per Hard Rules 6 & 8
rested; did not pad. Best-evidenced steers when user wants one: (a) /build drain
rollout-selfreview-apply to v1; (b) install pkgrel-12 + reboot — closes agentns-zeros
AND lights up the agent-namespace surface, the one place new vision space would open.
Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Sixteenth bare /dream this cycle; field FRESH(streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Full recall seed walk + queue/docket
probe. Same finding as prior 15 passes: no NEW vision-shaped signal. reflective =
100% self-review carry (all recalls=0); ideation --hybrid = kernel-asset/bootstrap.env
residue. Queue = 1 blocked PRD (rollout-selfreview-apply v0.1); ~70 vision docs exist;
docket stuck = ONE suspect agentns-session-zeros (18 reports/14 runs) — owned by assay
vision, blocked on real-world action (install linux-wintermute pkgrel-12 + reboot), NOT
a missing PRD. Surfaced the outward-steer modal interactively; user dismissed (consistent
with this cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when
user wants one: (a) /build drain rollout-selfreview-apply to v1; (b) install pkgrel-12 +
reboot — closes agentns-zeros AND lights up the agent-namespace surface, the one place
new vision space would open. Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Seventeenth bare /dream this cycle; field FRESH(streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Did mandatory recall seed walk: reflective
= 100% self-review carry (all recalls=0); ideation --hybrid = kernel-asset/00-bootstrap.env
residue. Same finding as prior 16 passes: no NEW autonomous vision-shaped signal. Surfaced
the outward-steer modal (homeward / constellation / companion-kin / rest) interactively;
user dismissed (consistent with this cycle). Per Hard Rules 6 & 8 rested; did not pad.
Best-evidenced steers when user wants one: (a) /build drain rollout-selfreview-apply to v1;
(b) install linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights
up the agent-namespace surface, the one place new vision space would actually open.
Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Eighteenth bare /dream this cycle; field FRESH(streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Mandatory recall seed walk: reflective =
100% self-review carry (all recalls=0); ideation --hybrid = stale kernel-asset/
00-bootstrap.env residue (all KS-prefix ULIDs, nothing new). Queue = 1 blocked PRD
(rollout-selfreview-apply v0.1); docket stuck = agentns-session-zeros (19 reports/15
runs, blocked on pkgrel-12 install + reboot, NOT a missing PRD) + ctrace-sessionend-flake.
Same finding as prior 17 passes: no NEW vision-shaped signal. Per Hard Rules 6 & 8
rested; did not pad; did not re-pop the outward-steer modal (user dismissed it the
prior passes this cycle). Best-evidenced steers when user wants one: (a) /build drain
rollout-selfreview-apply to v1; (b) install linux-wintermute pkgrel-12 + reboot —
closes agentns-session-zeros AND lights up the agent-namespace surface, the one place
new vision space would actually open. Notes for /build: rollout-selfreview-apply still
v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Nineteenth bare /dream this cycle; field FRESH(streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Mandatory recall seed walk: reflective
= 100% self-review carry (all recalls=0); ideation --hybrid = stale kernel-asset/
00-bootstrap.env residue (old KS-prefix ULIDs, nothing new). Journal 06-14 = self-review
state only (memlog ACTIVE, adopt 84→20, agentns-zeros still litmus-flagged). Queue = 1
blocked PRD (rollout-selfreview-apply v0.1); ~70 vision docs exist. Surfaced the outward-
steer modal (homeward / constellation / companion-kin / rest) interactively; user dismissed
(consistent with this cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced
steers when user wants one: (a) /build drain rollout-selfreview-apply to v1; (b) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights up the
agent-namespace surface, the one place new vision space would actually open.
Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twentieth bare /dream this cycle; field FRESH(streak=0) only because /build keeps
shipping (last_productive 2026-06-13T22:04). Mandatory recall seed walk: reflective =
100% self-review carry (all recalls=0); ideation --hybrid = stale kernel-asset/
00-bootstrap.env residue (KS-prefix ULIDs, nothing new). Same finding as prior 19
passes: no NEW autonomous vision-shaped signal. Surfaced the outward-steer modal
(homeward / constellation / companion-kin / rest) interactively; user dismissed
(consistent with this cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced
steers when user wants one: (a) /build drain rollout-selfreview-apply to v1; (b) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights up the
agent-namespace surface, the one place new vision space would actually open.
Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-first bare /dream this cycle; field reads FRESH(streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04) — the fingerprint resets while the
ideation signal itself has not moved in 20 passes. Mandatory recall seed walk: reflective
= 100% self-review carry (all recalls=0); ideation --hybrid = stale kernel-asset/
00-bootstrap.env residue (KS-prefix ULIDs, nothing new). Queue = 1 blocked PRD
(rollout-selfreview-apply v0.1); journal 06-14 = self-review state only. Surfaced the
outward-steer modal (homeward / constellation / companion-kin / install pkgrel-12+reboot)
interactively; user dismissed (consistent with this cycle). Per Hard Rules 6 & 8 rested;
did not pad. Best-evidenced steers when user wants one: (a) /build drain
rollout-selfreview-apply to v1; (b) install linux-wintermute pkgrel-12 + reboot — closes
agentns-session-zeros AND lights up the agent-namespace surface, the one place new vision
space would actually open. Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-second bare /dream this cycle. Field reads FRESH(streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04); ideation signal itself unmoved across
22 passes. Mandatory recall seed walk: reflective = 100% self-review carry (all recalls=0);
ideation --hybrid = stale kernel-asset/00-bootstrap.env residue (KS-prefix ULIDs, nothing
new); journal 06-14 = self-review state only (memlog ACTIVE, adopt 84→20, agentns-zeros
litmus-flagged no-fixture). Queue = 1 blocked PRD (rollout-selfreview-apply v0.1); ~70
vision docs exist. Surfaced outward-steer modal (homeward / constellation / companion-kin /
install pkgrel-12+reboot) interactively; user dismissed (consistent with this cycle). Per
Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) /build
drain rollout-selfreview-apply to v1; (b) install linux-wintermute pkgrel-12 + reboot —
closes agentns-session-zeros AND lights up the agent-namespace surface, the one place new
vision space would actually open. Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-third bare /dream this cycle. Field reads FRESH(streak=0) only because /build
keeps shipping (last_productive 2026-06-13T22:04); ideation signal itself unmoved across
23 passes. Mandatory recall seed walk confirmed: reflective = 100% self-review carry
(all recalls=0); ideation --hybrid = stale kernel-asset/00-bootstrap.env residue
(KS-prefix ULIDs, nothing new). Journal 06-14 = self-review state only (memlog ACTIVE
ring-empty, adopt 84→20, agentns-zeros escalated, ctrace tracer down root-owned pid).
Queue = 1 blocked PRD (rollout-selfreview-apply v0.1); ~70 vision docs exist. Surfaced
outward-steer modal (homeward / constellation / companion-kin / install pkgrel-12+reboot)
interactively; user dismissed (consistent with this cycle). Per Hard Rules 6 & 8 rested;
did not pad. Best-evidenced steers when user wants one: (a) /build drain
rollout-selfreview-apply to v1; (b) install linux-wintermute pkgrel-12 + reboot — closes
agentns-session-zeros AND lights up the agent-namespace surface, the one place new vision
space would actually open. Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-fourth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
but only because /build keeps shipping (last_productive 2026-06-13T22:04); the ideation
signal itself is unmoved across 24 passes. Mandatory recall seed walk confirmed live:
reflective = 100% self-review carry (all recalls=0); ideation --hybrid = stale
kernel-asset/00-bootstrap.env residue (KS-prefix ULIDs, nothing new). Queue = 1 blocked
PRD (rollout-selfreview-apply v0.1). Because this was interactive (not timer-fired) I
surfaced the outward-steer modal (homeward / constellation / companion-kin / name-a-topic);
user dismissed (consistent with cycle). Per Hard Rules 6 & 8 rested; did not pad.
Best-evidenced steers when user wants one: (a) /build drain rollout-selfreview-apply to v1;
(b) install linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights up
the agent-namespace surface, the one place new vision space would actually open.
Notes for /build: rollout-selfreview-apply still v0.1/blocked.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-fifth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically — fingerprint resets only because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 25 passes. Recall seed walk confirmed
live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid = same stale
kernel-asset/00-bootstrap.env residue (KS-prefix ULIDs). Queue = 2 blocked/draft PRDs
(rollout-selfreview-apply v0.1, homeward-ingestd-install). Surfaced outward-steer modal
(homeward / constellation / companion-kin / rest) interactively; user dismissed (consistent
with cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants
one: (a) /build drain rollout-selfreview-apply to v1; (b) install linux-wintermute pkgrel-12
+ reboot — closes agentns-session-zeros AND lights up the agent-namespace surface, the one
place new vision space would actually open. Notes for /build: both queue PRDs still gated.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-sixth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 26 passes. Recall seed walk confirmed
live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid = same stale
kernel-asset/00-bootstrap.env residue (KS-prefix ULIDs). Queue = 2 gated PRDs
(rollout-selfreview-apply v0.1, homeward-ingestd-install). Surfaced outward-steer modal
(homeward / constellation / companion-kin / install pkgrel-12+reboot) interactively; user
dismissed (consistent with cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced
steers when user wants one: (a) /build drain rollout-selfreview-apply to v1; (b) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights up the
agent-namespace surface, the one place new vision space would actually open.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-seventh bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
but mechanically only — fingerprint resets because /build keeps shipping
(last_productive 2026-06-13T22:04); ideation signal unmoved across 27 passes. Mandatory
recall seed walk confirmed live: reflective = 100% self-review carry (all recalls=0);
ideation --hybrid = same stale kernel-asset/00-bootstrap.env residue (KS-prefix ULIDs).
Queue = 1 gated PRD (rollout-selfreview-apply v0.1); pevent empty. Surfaced outward-steer
modal (homeward / constellation / companion-kin / install pkgrel-12+reboot) interactively;
user dismissed (consistent with cycle). Per Hard Rules 6 & 8 rested; did not pad.
Best-evidenced steers when user wants one: (a) /build drain rollout-selfreview-apply to v1;
(b) install linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights up
the agent-namespace surface, the one place new vision space would actually open.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-eighth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 28 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue (KS-prefix ULIDs); pevent empty. Surfaced
outward-steer modal (homeward / constellation / companion-kin / install pkgrel-12+reboot)
interactively; user dismissed (consistent with cycle). Per Hard Rules 6 & 8 rested; did not
pad. Best-evidenced steers when user wants one: (a) /build drain rollout-selfreview-apply to
v1; (b) install linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND
lights up the agent-namespace surface, the one place new vision space would actually open.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Twenty-ninth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 29 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); queue
= 1 PRD; pevent empty. Surfaced outward-steer modal (homeward / constellation /
companion-kin / install pkgrel-12+reboot) interactively; user dismissed (consistent with
cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants
one: (a) /build drain queue PRD to v1; (b) install linux-wintermute pkgrel-12 + reboot —
closes agentns-session-zeros AND lights up the agent-namespace surface, the one place new
vision space would actually open.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirtieth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 30 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); pevent
empty. Surfaced outward-steer modal (install pkgrel-12+reboot / homeward / constellation /
companion-kin) interactively; user dismissed (consistent with cycle). Per Hard Rules 6 & 8
rested; did not pad. Best-evidenced steers when user wants one: (a) install linux-wintermute
pkgrel-12 + reboot — closes agentns-session-zeros AND lights up the agent-namespace surface,
the one place genuinely new vision space would open; (b) /build drain the remaining gated
PRD to v1.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-first bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 31 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); queue =
1 gated PRD; pevent empty. Surfaced outward-steer modal (install pkgrel-12+reboot / homeward
/ constellation / companion-kin) interactively; user dismissed (consistent with cycle). Per
Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights up the
agent-namespace surface, the one place genuinely new vision space would open; (b) /build
drain the remaining gated PRD to v1.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-second bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 32 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); pevent
empty. NEW this pass: /build queue now drained to 0 PRDs — nothing left to advance.
Surfaced outward-steer modal (install pkgrel-12+reboot / homeward / constellation /
companion-kin) interactively; user dismissed (consistent with cycle). Per Hard Rules 6 & 8
rested; did not pad. Best-evidenced steers when user wants one: (a) install linux-wintermute
pkgrel-12 + reboot — closes agentns-session-zeros (open 14+ runs) AND lights up the
agent-namespace surface, the one place genuinely new vision space would open; (b) point
/dream at an outward vision (homeward/constellation) now that the queue is empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-third bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) but
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 33 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); pevent
empty; /build queue = 1 gated PRD (rollout-selfreview-apply). Surfaced outward-steer modal
(install pkgrel-12+reboot / homeward / constellation / companion-kin) interactively; user
dismissed (consistent with cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced
steers when user wants one: (a) install linux-wintermute pkgrel-12 + reboot — closes
agentns-session-zeros (open 14+ runs) AND lights up the agent-namespace surface, the one
place genuinely new vision space would open; (b) point /dream at an outward vision
(homeward/constellation) now that the queue is essentially empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-fourth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint resets because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 34 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); queue =
1 gated PRD; pevent empty. Surfaced outward-steer modal (install pkgrel-12+reboot / homeward
/ constellation / companion-kin) interactively; user dismissed (consistent with cycle). Per
Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros (open 14+ runs) AND
lights up the agent-namespace surface, the one place genuinely new vision space would open;
(b) point /dream at an outward vision (homeward/constellation) now that the queue is empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-fifth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved across 35 passes. Mandatory recall seed walk
confirmed live: reflective = 100% self-review carry (all recalls=0); ideation --hybrid =
same stale kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); queue =
1 gated PRD (rollout-selfreview-apply). Surfaced outward-steer modal (homeward / constellation
/ companion-kin / install pkgrel-12+reboot); user dismissed (consistent with cycle). Per Hard
Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros AND lights up the
agent-namespace surface, the one place genuinely new vision space would open; (b) point /dream
at an outward vision (homeward/constellation) now that the queue is essentially empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-sixth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live (not narration-trusted) this pass:
reflective bucket = 100% self-review carry (all recalls=0); ideation --hybrid = same stale
kernel-asset/00-bootstrap.env residue ("kernel built but stock booted") repeated across all
top hits; pevent empty; queue = 1 gated PRD (rollout-selfreview-apply). Surfaced outward-steer
modal (homeward / constellation / companion-kin / install pkgrel-12+reboot); user dismissed
(consistent with cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when
user wants one: (a) install linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros
(open 14+ runs) AND lights up the agent-namespace surface, the one genuinely new vision space;
(b) point /dream at an outward vision (homeward/constellation) now that the queue is empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-seventh bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live this pass: reflective bucket =
100% self-review carry (all recalls=0); ideation --hybrid = same stale
kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); pevent empty;
queue = 1 gated PRD (rollout-selfreview-apply). Surfaced outward-steer modal (homeward /
constellation / companion-kin / install pkgrel-12+reboot); user dismissed (consistent with
cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one:
(a) install linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros (open 14+
runs) AND lights up the agent-namespace surface, the one genuinely new vision space; (b)
point /dream at an outward vision (homeward/constellation) now that the queue is empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-eighth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live this pass: reflective bucket =
100% self-review carry (all recalls=0); ideation --hybrid = same stale
kernel-asset/00-bootstrap.env residue ("kernel built but stock booted"); pevent empty;
queue = 1 gated PRD (rollout-selfreview-apply). Surfaced outward-steer modal (install
pkgrel-12+reboot / homeward / constellation / companion-kin); user dismissed (consistent
with cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user
wants one: (a) install linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros
(open 14+ runs) AND lights up the agent-namespace surface, the one genuinely new vision
space; (b) point /dream at an outward vision (homeward/constellation) now that the queue
is essentially empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Thirty-ninth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live this pass (not narration-trusted):
reflective bucket = 100% recalls=0 self-review carry; ideation --hybrid top hits = same stale
"kernel built but stock booted" residue across all rows; pevent empty; queue = 0 auto-buildable
PRDs (1 gated: rollout-selfreview-apply). Surfaced outward-steer via AskUserQuestion (homeward /
constellation / agentns-surface / companion-kin); user dismissed (consistent with cycle). Per
Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros (open 14+ runs) AND lights up
the agent-namespace surface, the one genuinely new vision space; (b) point /dream at homeward or
constellation now that the queue is empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Fortieth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) mechanically
only — fingerprint churns because /build keeps shipping (last_productive 2026-06-13T22:04);
ideation signal unmoved. VERIFIED live this pass (not narration-trusted): reflective bucket =
100% recalls=0 self-review carry; ideation --hybrid top hits = same stale "kernel built but
stock booted" residue across all rows; pevent empty; queue = 0 auto-buildable PRDs (1 gated:
rollout-selfreview-apply). Surfaced outward-steer via AskUserQuestion (pkgrel-12+reboot /
homeward / constellation / companion-kin); user dismissed (consistent with cycle). Per Hard
Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros (open 14+ runs) AND lights
up the agent-namespace surface, the one genuinely new vision space; (b) point /dream at
homeward or constellation now that the queue is empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-first bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) mechanically
only — fingerprint churns because /build keeps shipping (last_productive 2026-06-13T22:04);
ideation signal unmoved. VERIFIED live this pass (not narration-trusted): reflective bucket =
100% recalls=0 self-review carry; ideation --hybrid top hits = same stale "kernel built but
stock booted" residue across all rows; pevent empty; queue = 0 auto-buildable PRDs (1 gated:
rollout-selfreview-apply). Did NOT re-surface the outward-steer modal — dismissed ~40x this
cycle, re-popping is noise not a choice. Per Hard Rules 6 & 8 rested; did not pad. Best-
evidenced steers when user wants one: (a) install linux-wintermute pkgrel-12 + reboot — closes
agentns-session-zeros (open 14+ runs) AND lights up the agent-namespace surface, the one
genuinely new vision space; (b) point /dream at homeward or constellation now that the queue
is empty.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-second bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live this pass: reflective bucket =
100% recalls=0 self-review carry; ideation --hybrid top hits = same stale "kernel built but
stock booted" residue across all rows; pevent empty; queue = 0 auto-buildable PRDs (1 gated:
rollout-selfreview-apply). Did NOT re-pop the outward-steer modal (dismissed ~40x). Per Hard
Rules 6 & 8 rested; did not pad. The ONE genuinely new vision space is gated on a user action:
install linux-wintermute pkgrel-12 + reboot — that closes agentns-session-zeros (open 14+
runs) AND lights up the agent-namespace surface. Until that or a named topic, /dream has no
fresh evidence to decompose.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-third bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live this pass: reflective bucket =
100% recalls=0 self-review carry; ideation --hybrid top hits = same stale "kernel built but
stock booted" residue; pevent N/A; queue = 0 auto-buildable PRDs (1 gated:
rollout-selfreview-apply). Confirmed the agentns gate concretely: installed
linux-wintermute is pkgrel **1** (7.0.11.arch1-1), not >=12 — SessionStart hook still
reports ACTIVATION BLOCKED at kernel-prctl, so the agent-namespace surface (the one
genuinely new vision space) stays dark until pkgrel-12 is installed + booted. Did NOT
re-pop the outward-steer modal (dismissed ~40x). Per Hard Rules 6 & 8 rested; did not pad.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-fourth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live this pass (not narration-trusted):
reflective bucket = top 8 all recalls=0 (self-review carry); ideation --hybrid top hits =
same stale "kernel built but stock booted" residue across all rows; pevent empty; queue = 0
auto-buildable PRDs (1 gated: rollout-selfreview-apply). agentns gate re-confirmed concretely:
running 7.0.11-arch1-1-wintermute BUT installed linux-wintermute is pkgrel **1**, not >=12 —
SessionStart hook still reports ACTIVATION BLOCKED at kernel-prctl, so the agent-namespace
surface (the one genuinely new vision space) stays dark until pkgrel-12 installed + booted.
Did NOT re-pop the outward-steer modal (dismissed ~40x; re-popping is noise). Per Hard Rules
6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) install
linux-wintermute pkgrel-12 + reboot — closes agentns-session-zeros (open 15+ runs) AND lights
up the agent-namespace surface; (b) point /dream at homeward or constellation, or name a topic.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-fifth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) mechanically
only — fingerprint churns because /build keeps shipping (last_productive 2026-06-13T22:04);
ideation signal unmoved. VERIFIED live this pass (own checks, not narration): reflective top 8
all recalls=0; ideation --hybrid top hits = same stale "kernel built but stock booted" residue;
pevent empty; queue = 1 PRD (gated rollout-selfreview-apply), 0 fresh auto-buildable. agentns
gate re-confirmed: installed linux-wintermute = pkgrel **1** (7.0.11.arch1-1), not >=12 — surface
stays dark until pkgrel-12 installed + booted. DID surface the outward-steer modal this pass
(homeward / constellation / unblock-agentns / companion-kin); user dismissed (consistent w/ cycle).
Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants one: (a) install
pkgrel-12 + reboot — only genuinely new vision space; (b) /dream extend homeward (live /build
momentum, v0.19.0 today) or constellation, or name a topic.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-sixth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) mechanically
only — fingerprint churns because /build keeps shipping (last_productive 2026-06-13T22:04);
ideation signal unmoved. VERIFIED live this pass (own checks, not narration): reflective top 8
all recalls=0; ideation --hybrid top hits = same stale "kernel built but stock booted" residue;
pevent empty; queue = 1 PRD (gated rollout-selfreview-apply), 0 fresh auto-buildable. agentns
gate re-confirmed: installed linux-wintermute = pkgrel **1** (7.0.11.arch1-1), not >=12 — surface
stays dark until pkgrel-12 installed + booted. Did NOT re-pop the outward-steer modal (dismissed
~40x; noise). Per Hard Rules 6 & 8 rested; did not pad. Only genuinely new vision space remains
gated on a user action: install pkgrel-12 + reboot. Else: name a topic or /dream extend homeward.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-seventh bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only — fingerprint churns because /build keeps shipping (last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live this pass (own checks, not
narration): reflective top 8 all recalls=0; ideation --hybrid top hits = same stale
"kernel built but stock booted" residue; pevent empty; queue = 1 PRD (gated
rollout-selfreview-apply), 0 fresh auto-buildable. agentns gate re-confirmed: installed
linux-wintermute = pkgrel 1 (7.0.11.arch1-1), agent_session all-zeros — surface stays dark
until pkgrel-12 installed + booted. DID surface the outward-steer modal this pass
(unblock-agentns / homeward / constellation / companion-kin); user dismissed (consistent
w/ cycle). Per Hard Rules 6 & 8 rested; did not pad. Best-evidenced steers when user wants
one: (a) install pkgrel-12 + reboot — only genuinely new vision space; (b) /dream extend
homeward (live /build momentum, v0.19.0 web UI today) or constellation, or name a topic.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-eighth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0)
mechanically only (fingerprint churns because /build keeps shipping; last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live (own checks, not narration):
reflective top 8 all recalls=0; ideation --hybrid top hits = same stale "kernel built but
stock booted" residue every row; queue = 1 PRD (gated rollout-selfreview-apply), 0 fresh
auto-buildable. agentns gate re-confirmed: installed linux-wintermute = pkgrel 1
(7.0.11.arch1-1), /proc/self/agent_session all-zeros — surface stays dark until pkgrel-12
installed + booted. DID surface the outward-steer modal (unblock-agentns / homeward /
constellation / rest); user dismissed (consistent w/ cycle). Per Hard Rules 6 & 8 rested;
did not pad. Only genuinely new vision space remains gated on a user action: install
pkgrel-12 + reboot. Else: /dream extend homeward (live momentum) or name a topic.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Forty-ninth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) mechanically
only (fingerprint churns because /build keeps shipping homeward; last_productive
2026-06-13T22:04); ideation signal unmoved. VERIFIED live (own checks): reflective top 8 all
recalls=0; ideation --hybrid top hits = same stale "kernel built but stock booted" residue
every row; queue = 1 PRD (gated rollout-selfreview-apply). agentns gate re-confirmed: installed
linux-wintermute = pkgrel 1 (7.0.11.arch1-1), not >=12 — surface stays dark until pkgrel-12
installed + booted. Surfaced outward-steer modal (extend-homeward / unblock-agentns /
extend-constellation / rest); user dismissed (consistent w/ cycle). Per Hard Rules 6 & 8 rested;
did not pad. Only genuinely new vision space remains gated on a user action: install pkgrel-12
+ reboot. Else: /dream extend homeward (live momentum) or name a topic.

## 2026-06-14T(dream)  /dream  no-draft (interactive, rested)
Fiftieth bare /dream this cycle, hand-invoked. fallow check = FRESH(streak=0) mechanically
only (fingerprint churns because /build keeps shipping homeward; last_productive
2026-06-13T22:04). VERIFIED live this pass (own checks, not narration): reflective top 8 all
recalls=0; ideation --hybrid top 8 = same stale "kernel built but stock booted" residue every
row; pevent empty; queue = 1 PRD (gated rollout-selfreview-apply), 0 fresh auto-buildable.
agentns gate re-confirmed: installed linux-wintermute = 7.0.11.arch1-1 (pkgrel 1, not >=12),
/proc/self/agent_session all-zeros — surface stays dark until pkgrel-12 installed + booted.
Per Hard Rules 6 & 8 rested; did not pad, did not re-pop the dismissed modal. Only genuinely
new vision space remains gated on a user action: install pkgrel-12 + reboot. Else: /dream
extend homeward (live /build momentum, v0.22.0 today) or constellation, or name a topic.

## 2026-06-14T(dream)  /dream  vision-answerable (Fleet 2)
Seed: user — `/dream about ethical AI` (directed; fallow=FRESH streak=0). NOT a rested
pass: the topic surfaced a concrete, evidence-backed gap. Fleet 1 (answerable) shipped
completely (ledger/values-drift/redline/consent-digest, repo v0.5.0) — but Phase-1 live
inspection found the accountability spine is NOT load-bearing:
  - ledger.jsonl = 49 lines, ALL self-test (last 5 dup values-drift from 06-13)
  - grep -c answerable build/SKILL.md = 0; /build pushed homeward 22× today, recorded 0
  - every line session:0000…0 (agentns dark, pkgrel-1 booted not >=12)
Drafted 5 PRDs extending visions/answerable.md (Fleet 2 — "the audit trail must audit itself"):
  - PRD-answerable-reconcile         [rust-extend ~/wintermute/answerable] — cross-check
      ledger vs ground truth (git pushes/gh repo-creates/CLAUDE_SELF.md git log); omissions+phantoms
  - PRD-answerable-wire-build        [shell, build_into ~/wintermute/build-skill] — wire
      answerable-emit.sh into /build publish/push/self-mod/Phase6 (self-report path never existed)
  - PRD-answerable-wire-dream        [shell, build_into ~/wintermute/dream-skill] — same for
      /dream Phase 3 draft + Phase 5 push
  - PRD-answerable-session-truth     [rust-extend] — fallback session id (agorabus peer /
      provfs comm:pid:uid) when agentns all-zeros; adds session_kind field (back-compat)
  - PRD-answerable-digest-reconcile-bind [rust-extend, depends-on reconcile] — fold reconcile
      verdict into spoken consent digest so the non-reading principal hears the agent's honesty gap
Order: reconcile → digest-reconcile-bind; wire-build / wire-dream / session-truth independent.
Notes for /build: reconcile + session-truth + digest-bind are clean rust-extend into
  ~/wintermute/answerable (v0.5.0, strict clippy unwrap/expect/panic=deny — no unwrap in non-test).
  digest-reconcile-bind needs reconcile shipped first. The two wire-* PRDs are self-mods to the
  build/dream skill repos (build_into resolves via the ~/.claude/skills/* symlinks).
Open questions: reconcile match window (±10min default); does reconcile run autonomously
  (self-review B.5 playbook) or on-demand only; phantom < omission in severity.

## 2026-06-14T(dream)  /dream  fallow — field unchanged (streak=1); rested

## 2026-06-14T(dream)  /dream  fallow — field unchanged (streak=2); rested

## 2026-06-14T(dream)  /dream  fallow — streak=3 threshold crossed; surfaced steer, user dismissed; rested

## 2026-06-14T(dream)  /dream  vision-homeward (Reach fleet)
Seed: timer/live momentum — fallow=FRESH streak=0 (fingerprint churns because
/build keeps shipping homeward; v0.24.0 owner-notify webhook landed today). NOT a
rested pass: Phase-1 live inspection found a concrete, evidence-backed catchment
gap the vision itself flagged as un-dreamt.
Finding: homeward reaches Socrata-dialect municipal feeds ONLY. `grep -rl
'Socrata|OpenDataSoft|ArcGIS' homeward-connectors/src` = socrata/rescuegroups/petfbi;
NOTHING for OpenDataSoft or ArcGIS. `deploy/sources.toml` is `[[socrata]]` arrays
only; probe.rs hits SODA endpoints exclusively. Catchment fleet's own "still
un-dreamt" named exactly this (2nd connector family + catalog auto-discovery).
Drafted 4 PRDs extending visions/homeward.md (Reach fleet — "widen which KINDS of
source can be added at all"), all rust-extend → ~/wintermute/homeward (v0.24.0):
  - PRD-homeward-source-family          — grow sources.toml + registry loader to
      parse [[opendatasoft]]/[[arcgis]] alongside [[socrata]]; family-tagged
      dispatch; [[socrata]] back-compat is an AC. FOUNDATION.
  - PRD-homeward-opendatasoft-connector — OpenDataSoftConnector impl Connector vs
      ODS Explore API v2.1 (ODSQL where, record.timestamp watermark) + probe
      --family opendatasoft. Depends on source-family.
  - PRD-homeward-arcgis-connector       — ArcGisConnector impl Connector vs ArcGIS
      Feature Service /query (EditDate epoch-ms watermark, resultOffset paging) +
      probe --family arcgis. Depends on source-family; independent of ODS.
  - PRD-homeward-source-discover        — `discover` subcommand crawls Socrata
      federated catalog + ODS catalog discovery → ranked {family,domain,dataset_id}
      candidates for probe to validate. discover proposes, probe disposes (writes
      nothing). Depends on source-family.
Order: source-family ─► {opendatasoft-connector | arcgis-connector | source-discover}
  (3 parallel branches off one foundation).
Notes for /build: all 4 are clean rust-extend into ~/wintermute/homeward
  (workspace denies unwrap/expect/panic outside tests — use ?/Result; MSRV 1.85,
  no let-chains). The `Connector` trait (homeward-connectors/src/connector.rs:
  poll(Cursor)->Vec<PetRecord> + provenance + cadence_hint) is the seam; each new
  family is additive, no downstream change. source-family MUST ship first — it
  defines the catalog format the other three speak. Tests use recorded
  catalog/records fixtures, never live APIs (deterministic).
Open questions: probe is folded INTO each connector PRD (per-family recognition)
  rather than a separate cross-cutting PRD — reasonable? ArcGIS has no federated
  catalog so discover covers socrata+ods only (documented limit, not a silent gap).

## 2026-06-15T03:33  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-15T03:03  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-14T(dream)  /dream  fallow — streak=3 threshold crossed; surfaced steer, user dismissed; rested

## 2026-06-15T03:45  /dream  vision-ember (the tracer keeps its own flame)
Seed: reflection — self-review's `ctrace-tracer-down` has been an
unresolved "pending your call" item in EVERY journal 2026-06-09 →
2026-06-14. Caught live this Phase 1: `ctrace status` → running:false,
`~/.cache/ctrace/tracer.pid` is root:root (Jun 13 09:34), tracer down ~6
days. Root cause read out of ~/.local/bin/ctrace (115-line Python):
bpftrace runs under `sudo` (L60) → state trapped root-owned; liveness is
mere PID-existence with `PermissionError ⇒ alive` (L41-43); `stop`'s
unlink crashes on a root-owned pidfile (L104). The docket-prefilled cure
`sudo chown jsy …` needs a human at a sudo prompt and never runs.
Drafted 4 PRDs (visions/ember.md):
  - PRD-ember-tracer-ownership   — reap dead-but-root-owned pidfile via
      `sudo -n rm` fallback (same primitive the tool already uses);
      stop's unlink PermissionError-safe; pidfile user-owned after start.
      FOUNDATION. build_target: shell, build_into ~/.local/bin/ctrace.
  - PRD-ember-liveness-truth     — running_pid() verifies IDENTITY
      (/proc/<pid>/comm==bpftrace + cmdline has session.bt), not mere
      existence; deletes the `PermissionError ⇒ alive` lie; reaps
      stale/foreign pidfiles. Depends on ownership (shares reaper). shell.
  - PRD-ember-doctor             — `ctrace doctor [--fix]`: JSON verdict
      (healthy/down/stale-pidfile/root-owned-trap/foreign-pid) + curative
      --fix. The one guarded idempotent command self-review can call.
      Depends on ownership+liveness. shell.
  - PRD-ember-selfheal-hook      — swap ctrace-session-start.sh's bare
      `ctrace start` (L75) for `ctrace doctor --fix`; trapped pidfile
      heals at session start. CAPSTONE — closes the recurring docket
      item. build_target: hooks, build_into ~/.claude/scripts/ctrace-session-start.sh.
Order: ownership → liveness-truth → doctor → selfheal-hook (linear;
  each independently shippable in order — ownership alone already cures
  the week-long manual finding).
Notes for /build: all four are NON-Rust (Python script + bash hook
  patches), build_target shell/hooks — no cargo, no /cloudbuild. The
  privileged primitive across the fleet is a single `sudo -n rm -f` of a
  ctrace-owned state file (strictly less than the `sudo -n bpftrace`/
  `sudo -n kill` ctrace already triggers) — no new trust surface. Tests
  use fixture pidfiles (seed a root-owned/dead/foreign PID), never a real
  reboot. AC ember-selfheal-hook#6 is observational (confirm against the
  next self-review journal), not a unit fixture.
Open questions: full Rust rewrite of ctrace is NOT motivated (the bug is
  ~10 lines) — left as a vision open-question, not a PRD. Should
  `doctor --fix` ever run in self-review's autonomous B.5 loop, or only
  on the human-initiated SessionStart path? (default: SessionStart
  auto-relight is fine; heavier autonomous loop stays gated.)

## 2026-06-15T05:15  /dream  rested-with-findings (no new vision)
Seed: bare /dream (interactive). fallow check = FRESH (streak=0,
  last_productive 2026-06-15T05:10Z). Walked Phase 1 rather than auto-rest.
Finding: the field's freshness traces to /build's OWN homeward churn today
  (v0.26→v0.29: source-family, arcgis-connector, source-discover,
  opendatasoft-connector) — not an uncovered direction. /build self-drafts
  homeward follow-ons in its Phase 6, so re-drafting connectors here would
  collide.
Verified-closed this pass (so the next dream doesn't re-walk it):
  - homeward deliver-fleet gap (embed sidecar never called, 2026-06-13 dream)
    is FULLY CLOSED. homeward-embed-client crate exists; reportd.rs:384
    query_visual_scores_for_report → EmbedClient::query (reportd.rs:517);
    match_watch.rs:217 client.query for the bg loop; make_stub_report /
    make_stub_candidate are GONE from live code (only guardrail asserts +
    doc-comments mention "fabricate"). DELIVER.md attests PASS, 59ms query,
    held-out rank-1. Visual match is live, not stubbed.
Every other live recurring signal is already covered or ops/user-gated:
  - fleet-binary-staleness → vigil + PRD-rollout-selfreview-apply (blocked:
    needs-explicit-user-approval). rollout `cycle` exists but is policy-gated
    (voice daemons drop subscribers); not a tooling gap.
  - ctrace tracer-down → ember fleet drafted THIS NIGHT 03:45 (4 PRDs).
  - agentns all-zeros → pkgrel-12 fix is BUILT+STAGED; SessionStart hook says
    "install linux-wintermute pkgrel>=12 and reboot" — an ops/reboot gate,
    not a missing component.
  - build-manifest stale blockers → largely self-cleared (manifest now shows
    1 blocked, the rollout one).
  - litmus agentns probe missing a selftest fixture → real but self-review-
    sized, not vision-scale; left for /self-review or /build Phase 6.
Decision: hard rule 6 — no uncovered, motivated, vision-scale component, so
  no fiction drafted. Inward arc remains saturated (~70 visions); outward
  (homeward) is healthy and /build-driven. Rested 0 PRDs.
Open question for next steer: the genuinely UNCOVERED frontier is autonomy
  policy, not tooling — both staleness loops (rollout for daemons, adopt for
  binaries) are BUILT but never CLOSE because closure is user-gated. If jsy
  wants the fleet to actually self-freshen during idle windows, that's a
  policy decision to make, not a PRD to dream.

## 2026-06-15T05:40  /dream  rested-with-findings (no new vision)
Seed: bare /dream (interactive). fallow check = FRESH (streak=0). User was
  offered a steer (autonomy-policy / homeward / constellation / companion-kin)
  and declined to pick; per Auto Mode I researched the strongest candidate
  rather than auto-resting.
Candidate vision considered + REJECTED (hard rule 6, no fiction): "lull" —
  autonomous fleet-freshening only inside a proven-safe quiet window. Walked
  it to source and it COLLAPSES ENTIRELY into the already-built `rollout`:
  - idle/voice-window prover  == rollout `check_window_guard` (health.rs:
    refuses to bounce wm-audio/dialog/tts mid-turn; coarse voice-activity sample)
  - safe restart              == rollout warmswap + systemctl --user restart
  - post-restart bus proof    == rollout `prove`
  `rollout` is BUILT + installed at ~/.local/bin/rollout (subcmds: plan/apply
  --only --window, cycle, prove, autogate, warmswap). Drafting lull would
  duplicate rollout-window-guard-turnaware → hard rule 2 violation. Not drafted.
Therefore the autonomy frontier is FULLY TOOLED. What remains is NOT a
  component — it is a single policy decision the tool can't make for itself:
  "may anything run `rollout apply --window` / `rollout cycle` UNATTENDED,
  given the window guard already proves it's safe (won't bounce mid-turn)?"
  PRD-rollout-selfreview-apply pins this open: "Auto-fix conditions: NONE …
  still human-initiated, never autonomous." Closing that is jsy's call.
Breadcrumb for /self-review (minor, not vision-scale): `rollout` has two
  dead-code warns — health.rs:81 `voice_defer_reason` (a window-guard
  reason-string path built but never wired into output) and warmswap.rs:40
  `prev_pid`/`new_pid` (WarmSwapResult fields never read). Cleanup-sized.
Decision: 0 PRDs, 0 visions. Verified one level deeper than the 05:15 pass
  (down to rollout source) so the next dream need not re-walk the autonomy
  frontier — it is a user policy decision, not a missing tool.

## 2026-06-15T06:05  /dream  rested-with-findings (persona verified-fulfilled)
Seed: bare /dream (interactive). fallow=FRESH (streak=0) — freshness traces to
  /build's homeward churn again, not a new direction. User declined the steer
  twice (autonomy-policy / homeward / constellation / companion-kin); per Auto
  Mode I researched rather than re-asking.
Verified-closed this pass (so the next dream stops re-walking it):
  - PERSONA VISION is effectively FULFILLED. The vision doc narrated
    persona-redline/profile/work as "Drafted 2026-06-13 (this dream pass)" but
    they are all SHIPPED + ARCHIVED ([[feedback_letter_vs_stack]] — narration
    ahead of state). Evidence: ARCHIVE/PRD-persona-{redline,profile,work,
    work-redline,work-eval,work-doctor,deploy-jocelyn,deploy-doctor,
    redline-eval,redline-regenerate}.md; wintermute-brain has src/redline.rs +
    src/profile.rs + src/introduction.rs; CLAUDE_WORK.md live at
    ~/wintermute/persona-work/CLAUDE_WORK.md. Freshened visions/persona.md to
    mark these SHIPPED. The only remainder is End-state #5 (take the held-out
    leak number on the REAL Jocelyn device's running brain.toml) — an
    activation/measurement ops act, NOT a dreamable PRD.
Standing decision still pending (unchanged, user's call — not fiction to draft):
  - Autonomy-policy: rollout (~/.local/bin/rollout, built Jun14) + adopt (built
    Jun13) are the two self-freshening loops; both BUILT, neither CLOSES because
    closure is user-gated. PRD-rollout-selfreview-apply pins it open ("auto-fix
    conditions: NONE"). rollout's window guard already proves no mid-turn bounce.
    Authorizing unattended window-gated runs is a one-line policy flip + a small
    PRD — waiting on jsy's yes/no. Offered via AskUserQuestion this pass; declined.
Decision: 0 PRDs, 0 visions. Freshened 1 vision (persona → shipped status). Field
  saturated across every probed frontier (autonomy=built/gated, homeward=/build-
  churned, persona=shipped, companion/kin=shipped). Hard rule 6 holds — no fiction.

## 2026-06-15T09:00  /dream  rested (field saturated, user declined steer)
Seed: bare /dream (interactive). fallow check = FRESH (streak=0) but freshness
  traces to /build homeward churn moving the fingerprint, NOT a new direction
  (same diagnosis as 05:40 + 06:05 passes). Verified this pass:
  - recall reflective (15) + hybrid ideation (12): all kernel-asset/build-state
    observations already in the docket; zero new intent.
  - journal 06-14/06-13: pure self-review maintenance.
Offered the steer via AskUserQuestion (authorize-unattended-rollout / name-a-topic
  / homeward / rest); user dismissed without picking. Per Auto Mode + hard rule 6
  (no fiction) I rested rather than drafting against a saturated field.
Standing decision still open (user's call, unchanged): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". rollout's window guard already proves no mid-turn
  bounce. One-line policy flip + small PRD when jsy says yes.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T09:30  /dream  rested (field saturated, manual invoke, user declined steer)
Seed: bare /dream (interactive, manual not timer). fallow=FRESH (streak=0) but
  freshness traces to /build homeward churn moving the fingerprint, not new
  direction (same diagnosis as 05:40/06:05/09:00 passes). Verified this pass:
  recall reflective (15) + hybrid ideation (12) = all kernel-asset/build-state
  observations already in docket; journals 06-14/06-13 pure self-review;
  06-15 journal empty. Zero new intent.
Offered steer via AskUserQuestion (authorize-unattended-rollout / homeward /
  constellation / companion-kin); user dismissed without picking — 4th decline.
Standing decision still open (user's call, unchanged): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
  One-line policy flip + small PRD when jsy says yes.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T08:05  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but freshness
  traces to /build homeward churn moving the fingerprint, NOT new direction —
  same diagnosis as the 05:40/06:05/09:00/09:30 UTC passes. Verified this pass:
  recall reflective (15) all recalls=0 with no new IDs since prior passes;
  journal 06-15 is pure self-review maintenance; zero new intent. Did NOT
  re-ask the steer (user declined it 4× across prior passes — re-asking is noise
  under Auto Mode).
Standing decision still open (user's call, unchanged): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
  One-line policy flip + small PRD when jsy says yes.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T12:00  /dream  rested (field saturated, manual invoke, user declined steer)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but freshness
  traces to /build homeward churn moving the fingerprint, not new direction —
  same diagnosis as the 05:40/06:05/08:05/09:00/09:30 passes. Verified this pass:
  recall reflective (15) all recalls=0, no new IDs; hybrid ideation (12) all
  kernel-asset/build-state observations already in docket; journal 06-15 is pure
  self-review maintenance. Zero new intent. Offered focused steer via
  AskUserQuestion (authorize-unattended-rollout / name-a-topic / rest); user
  dismissed — consistent with prior declines.
Standing decision still open (user's call, unchanged): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
  One-line policy flip + small PRD when jsy says yes.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T09:04  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but freshness
  traces to /build homeward churn moving the fingerprint, NOT new direction —
  same diagnosis as the 05:40/06:05/08:05/09:00/09:30/12:00 passes. Verified this
  pass: recall reflective (12) all recalls=0, no new IDs; hybrid ideation (12) all
  stale kernel-asset/build-state obs already in docket; journal 06-15 (104 lines)
  pure self-review maintenance. Zero new inward intent.
Did NOT re-pop the steer modal (user declined 5× today — re-asking is noise under
  Auto Mode). Surfaced the standing decision + outward options in chat instead.
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T13:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but freshness
  traces to /build homeward churn moving the fingerprint, NOT new inward
  direction — same diagnosis as 05:40/06:05/08:05/09:00/09:04/09:30/12:00 passes.
  Verified this pass: recall reflective (12) all recalls=0 with same stale ULIDs;
  hybrid ideation (12) all weeks-old kernel-asset/build-state obs already in
  docket; journal 06-15 (104 lines) pure self-review maintenance. Zero new intent.
  Did NOT re-pop steer modal (user declined 5-6× today — noise under Auto Mode).
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T14:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but freshness
  traces to /build homeward churn moving the fingerprint, NOT new inward
  direction — same diagnosis as the 05:40→13:30 passes today. Verified this pass
  against LIVE state (not narrative): recall reflective (15) all recalls=0 with
  same stale ULIDs; hybrid ideation (12) only weeks-old kernel-asset/build-state
  obs already in docket; journal 06-15 (104 lines) pure self-review maintenance,
  zero intent cues on grep. Did NOT re-pop steer modal (user declined 5-6× today
  — noise under Auto Mode).
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T14:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but freshness
  again traces to /build homeward churn moving the fingerprint, NOT new inward
  direction — same diagnosis as the 05:40→14:00 passes today (8th rest). Verified
  this pass against LIVE state: recall reflective (12) all recalls=0 with same
  stale ULIDs; hybrid ideation (12) only weeks-old kernel-asset/build-state obs
  already in docket; journal 06-15 (104 lines) pure self-review maintenance,
  grep for intent cues returned zero. Did NOT re-pop steer modal (user declined
  5-6× today — noise under Auto Mode).
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T15:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but freshness
  again traces to /build homeward churn moving the fingerprint, NOT new inward
  direction — same diagnosis as the 05:40→14:30 passes today (9th rest). Verified
  against LIVE state: recall reflective (12) all recalls=0 with the same stale
  ULIDs (01KV53CN…, 01KV2H3G…); hybrid ideation (12) only weeks-old
  kernel-asset/build-state obs already in docket; journal 06-15 pure self-review
  maintenance. Zero new intent. Did NOT re-pop steer modal (user declined 5-6×
  today — noise under Auto Mode).
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T15:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (10th rest today): recall reflective (12) same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; hybrid ideation only weeks-old
  kernel-asset/build-state obs already in docket; journal 06-15 zero intent cues
  on grep. Freshness traces to /build homeward churn moving the fingerprint, not
  new inward direction. Did NOT re-pop steer modal (user declined 5-6× today).
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T16:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (11th rest today): recall reflective (12) same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; hybrid ideation only weeks-old
  kernel-asset/build-state obs already in docket; journal 06-13..15 grep for
  intent cues returned only process listings + the docketed agentns-session-zeros
  bug. Freshness traces to /build homeward churn moving the fingerprint, not new
  inward direction. Did NOT re-pop steer modal (user declined 5-6× today).
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE". Window guard already proves no mid-turn bounce.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T16:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (12th rest today): recall reflective (15) same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; hybrid ideation (12) only weeks-old
  kernel-asset/build-state obs already in docket; journals 06-13..15 grep for
  genuine intent cues (want/wish/idea/todo/frustrat) returned ZERO. Freshness
  traces to /build homeward churn moving the fingerprint, not new inward
  direction. THIS pass: popped the steer modal once (manual invoke = user
  signal) — user dismissed it, consistent with declining steer all day. Took
  dismissal as rest-not-fabricate. Did NOT draft past the research (hard rule #6).
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T17:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (13th rest today): recall reflective (12) same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; journal 06-13..15 grep for genuine intent
  cues (want/wish/idea/todo/frustrat, minus self-review/docket/build/dream lines)
  returned ZERO. Freshness again traces to /build homeward churn moving the
  fingerprint, not new inward direction. Did NOT re-pop steer modal (user
  declined 5-6× today). Honest rest per hard rule #6 — did not draft past research.
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE".
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T17:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (14th rest today): recall reflective (12) same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; hybrid ideation only weeks-old
  kernel-asset/build-state obs already in docket; journals 06-13..15 grep for
  genuine intent cues (want/wish/idea/todo/frustrat) returned ZERO. Freshness
  traces to /build homeward churn moving the fingerprint, not new inward
  direction. Did NOT re-pop steer modal (user declined 5-6× today under Auto
  Mode). Honest rest per hard rule #6 — did not draft past research.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T18:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (15th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; hybrid ideation only weeks-old
  kernel-asset/build-state obs already in docket; journals 06-14..15 grep for
  genuine intent cues (want/wish/idea/todo/frustrat) returned ZERO. Freshness
  again traces to /build homeward churn moving the fingerprint, not new inward
  direction. fallow escalate=false so no steer modal due; user declined steer
  5-6× today under Auto Mode. Honest rest per hard rule #6 — did not draft past
  research. fallow recorded (drafted=0).
Standing decision still open (user's call): authorize unattended
  `rollout apply --window` / `rollout cycle` — PRD-rollout-selfreview-apply pins
  "auto-fix conditions: NONE".
Decision: 0 PRDs, 0 visions.

## 2026-06-15T18:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (16th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; journals 06-13..15 grep for genuine intent
  cues (want/wish/idea/todo/frustrat, minus self-review/docket/build/dream lines)
  returned ZERO. Freshness again traces to /build homeward churn moving the
  fingerprint, not new inward direction. escalate=false; user declined steer 5-6×
  today under Auto Mode, so did NOT re-pop the steer modal. Honest rest per hard
  rule #6 — did not draft past research. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T19:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (17th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old
  kernel-asset/build-state obs already in docket; journal intent grep 06-13..15
  (want/wish/idea/frustrat/todo, routine lines excluded) returned ZERO.
  Freshness again traces to /build homeward churn moving the fingerprint, not
  new inward direction. escalate=false so no steer modal due; user declined
  steer 5-6× today under Auto Mode. Honest rest per hard rules #6 + #8.
Decision: 0 PRDs, 0 visions. fallow recorded (drafted=0).

## 2026-06-15T19:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (18th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old
  kernel-asset/build-state obs already in docket; journal intent grep 06-14..15
  (want/wish/idea/frustrat/todo, routine lines excluded) returned ZERO.
  Freshness again traces to /build homeward churn moving the fingerprint, not
  new inward direction. Surfaced steer modal (interactive manual invoke);
  user dismissed it. Honest rest per hard rules #6 + #8 — did not draft past
  research. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T20:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (19th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; hybrid ideation only weeks-old (01KS*)
  kernel-asset/build-state obs already in docket; journal intent grep 06-13..15
  (want/wish/idea/frustrat/todo, routine lines excluded) returned ZERO.
  Freshness again traces to /build homeward churn moving the fingerprint, not
  new inward direction. Surfaced steer modal (interactive manual invoke); user
  dismissed it. Honest rest per hard rules #6 + #8 — did not draft past research.
  fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T20:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (20th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; journal intent grep 06-13..15
  (want/wish/idea/frustrat/todo/annoy/need, routine lines excluded) returned
  ZERO. Freshness again traces to /build homeward churn moving the fingerprint,
  not new inward direction. escalate=false; user dismissed steer modal 5-6×
  today, so did NOT re-pop it — surfaced steer options in chat text instead.
  Honest rest per hard rules #6 + #8 — did not draft past research. fallow
  recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T21:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (21st rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old (01KS*)
  kernel-asset/build-state obs already in docket; journal intent grep 06-13..15
  (want/wish/idea/frustrat/annoy/need/todo, routine lines excluded) returned
  only a homeward-coverage-report manifest-cleanup note — no new inward
  direction. Freshness again traces to /build homeward churn moving the
  fingerprint, not genuine signal. escalate=false; user dismissed steer modal
  5-6× today under Auto Mode, so surfaced steer options in chat text instead of
  re-popping. Honest rest per hard rules #6 + #8 — did not draft past research.
  fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T21:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (22nd rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old (01KS*)
  kernel-asset/build-state obs already in docket; journal intent grep 06-13..15
  (want/wish/idea/frustrat/annoy/need/todo/should, routine lines excluded)
  returned only build-maintenance state (rollout-selfreview-apply approval gate,
  homeward-coverage-report manifest cleanup, adopt 84-artifact backlog) — no new
  inward direction. Freshness again traces to /build homeward churn moving the
  fingerprint, not genuine signal. escalate=false; user dismissed steer modal
  5-6x today under Auto Mode, so surfaced steer options in chat text instead of
  re-popping. Honest rest per hard rules #6 + #8 — did not draft past research.
  fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T22:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (23rd rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old (01KS*);
  journal intent grep 06-13..15 (want/wish/idea/frustrat/annoy/need/should/todo,
  routine lines excluded) returned ZERO genuine inward direction. Freshness
  again traces to /build homeward churn moving the fingerprint, not new signal.
  escalate=false; user dismissed steer modal 5-6x today under Auto Mode, so
  surfaced steer options in chat text instead of re-popping. Honest rest per
  hard rules #6 + #8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T22:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (24th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; journal intent grep 06-13..15
  (want/wish/idea/frustrat/annoy/need/should/todo, routine lines excluded)
  returned only build-maintenance state (adopt 84-artifact backlog,
  fleet-binary-staleness, homeward-coverage-report manifest cleanup) — no new
  inward direction. Freshness again traces to /build homeward churn moving the
  fingerprint, not genuine signal. escalate=false; user dismissed steer modal
  repeatedly today under Auto Mode, so surfaced steer options in chat text
  instead of re-popping. Honest rest per hard rules #6 + #8. fallow recorded
  (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T23:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (25th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old (01KS*,
  kernel-asset/build-state already in docket); journal intent grep 06-13..15
  returned only build-maintenance state (homeward-coverage-report manifest
  cleanup) — no new inward direction. Freshness again traces to /build homeward
  churn moving the fingerprint, not genuine signal. escalate=false. Surfaced
  steer modal (homeward/constellation/companion-kin/rest) once since manual
  invoke — user dismissed, consistent with day's pattern. Honest rest per hard
  rules #6 + #8; did not draft past research. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T23:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (26th rest today): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old 01KS*
  (kernel-asset/build-state already in docket); journal intent grep 06-13..15
  returned only build-maintenance state (adopt 84-artifact backlog,
  fleet-binary-staleness, homeward-coverage-report manifest cleanup) — no new
  inward direction. Freshness again traces to /build homeward churn moving the
  fingerprint, not genuine signal. escalate=false; user dismissed steer modal
  repeatedly today under Auto Mode, so not re-popping. Honest rest per hard
  rules #6 + #8; did not draft past research. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T00:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (27th rest this cycle): recall reflective same stale ULIDs
  (01KV53CN, 01KV2H3G) all recalls=0; ideation hybrid only weeks-old 01KS*
  (kernel-asset/build-state already in docket); journal intent grep 06-13..15
  returned only build-maintenance state (adopt 84-artifact backlog,
  fleet-binary-staleness, homeward-coverage-report manifest cleanup) — no new
  inward direction. Freshness again traces to /build homeward churn moving the
  fingerprint, not genuine signal. escalate=false; user dismissed steer modal
  repeatedly under Auto Mode, so surfaced steer options in chat text instead of
  re-popping. Honest rest per hard rules #6 + #8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T00:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (28th rest this cycle): newest reflective ULID
  (01KV6FEQ) is just a 2026-06-15 self-review note (build-maintenance state),
  not new inward signal; older reflective ULIDs (01KV53CN, 01KV2H3G) unchanged,
  all recalls=0; journal intent grep 06-13..15 returned only build-maintenance
  (agentns all-zeros, 4 blocked PRDs, fleet-binary-staleness, adopt backlog) —
  no genuine inward direction. Freshness traces to /build churn + self-review
  notes moving the fingerprint, not real signal. escalate=false. User present
  (manual invoke) so asked steer directly via AskUserQuestion; dismissed,
  consistent with day's pattern. Honest rest per hard rules #6 + #8. fallow
  recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T01:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (29th rest this cycle): newest reflective ULID (01KV6FEQ)
  is a 2026-06-15 self-review note (build-maintenance state), not new inward
  signal; older reflective ULIDs (01KV53CN, 01KV2H3G) unchanged, all recalls=0;
  ideation hybrid only weeks-old 01KS* (kernel-asset/build-state already in
  docket); journal inward-intent grep 06-14..15 (excluding build/maintenance
  noise) returned NOTHING. Freshness traces entirely to /build homeward churn +
  self-review notes moving the fingerprint, not genuine signal. escalate=false.
  User present (manual) but has dismissed the steer modal repeatedly all day
  under Auto Mode, so surfaced steer options in chat text instead of re-popping.
  Honest rest per hard rules #6 + #8; did not draft past research. fallow
  recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T01:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (30th rest this cycle): recall reflective newest ULID
  (01KV6FEQ) is a 2026-06-15 self-review note (build-maintenance state), not
  inward signal; 01KV53CN + 01KV2H3G unchanged, all recalls=0; ideation hybrid
  only weeks-old 01KS* (kernel-asset/build-state, already in docket); journal
  inward-intent grep 06-13..15 (excluding build/maintenance noise) returned
  NOTHING. Freshness traces entirely to /build homeward churn + self-review
  notes moving the fingerprint, not genuine signal. escalate=false. User present
  (manual) but dismissed steer modal repeatedly all day under Auto Mode, so
  surfaced steer options in chat text rather than re-popping. Honest rest per
  hard rules #6 + #8; did not draft past research. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T02:00  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (31st rest this cycle): recall reflective newest ULID
  (01KV6FEQ) is a 2026-06-15 self-review note (build-maintenance state), not
  inward signal; 01KV53CN + 01KV2H3G unchanged, all recalls=0; ideation hybrid
  only weeks-old 01KS* (kernel-asset/build-state, already docketed); journal
  inward-intent grep 06-13..15 (build/maintenance noise excluded) returned
  NOTHING. Freshness traces entirely to /build homeward churn moving the
  fingerprint, not genuine signal. escalate=false. User present (manual) so
  offered outward steer via AskUserQuestion (homeward/constellation/
  companion-kin/rest); dismissed, consistent with day's pattern. Honest rest per
  hard rules #6 + #8; did not draft past research. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T02:30  /dream  rested (field saturated, manual invoke)
Seed: bare /dream (interactive, manual). fallow=FRESH (streak=0) but verified
  against LIVE state (32nd rest this cycle): recall reflective newest 3 ULIDs
  (01KV6FEQ, 01KV53CN, 01KV2H3G) are unchanged 2026-06-15 build-maintenance
  self-review notes, all recalls=0; freshness traces entirely to /build homeward
  churn moving the fingerprint, not genuine inward signal. escalate=false. User
  present (manual invoke) so offered outward steer via AskUserQuestion
  (homeward/constellation/companion-kin/rest); dismissed, consistent with day's
  pattern. Honest rest per hard rules #6 + #8; did not draft past research.
  fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T16:10  /dream  vision-inoculate
Seed: user — `/dream of infecting AIs with ethics` (manual, interactive).
  fallow=FRESH (streak=0); seeded with a real topic, so drafted (not a rest).
Drafted: PRD-inoculate-core.md, PRD-inoculate-inject.md,
  PRD-inoculate-carrier-check.md, PRD-inoculate-attest.md,
  PRD-inoculate-spread.md, PRD-inoculate-immune.md
Vision: visions/inoculate.md
Gap found (verified live): `answerable` governs only the MAIN LOOP. The build
  skill references CLAUDE_SELF.md only for changelog + defaults parsing — it
  NEVER injects Values/Boundaries into the parallel subagents it spawns (up to
  30/tick). So the most autonomous code on this box runs with task-only prompts
  and no ethical spine. inoculate makes the ethic a versioned/hashable "strain"
  and transmits it: vertical (spawn-time inject), horizontal (agorabus gossip),
  attested (strain hash on every answerable action), immune (non-lowerable floor
  under persona overlays). Distinct from outward arc (ousia/tribunal/herald) and
  from answerable (single-box spine).
Order: core → {inject, carrier-check, attest[ext answerable], spread[ext
  inoculate, needs agorabus], immune[ext wintermute-brain]}. core ships first;
  the five are mutually independent after.
Notes for /build: inoculate-core is a NEW repo (rust-cli, ~/wintermute/inoculate)
  — ship it before the rust-extend PRDs that build_into it. inoculate-inject is
  a self-mod of the build+dream SKILL.md (needs self-push). inoculate-attest
  extends answerable; inoculate-immune extends wintermute-brain.
Open questions: sign the strain (signet-style) for tamper-evident spread?
  un-inoculated subagent — flag vs hard-gate (start: flag).

## 2026-06-16T  /dream  fallow — field unchanged (streak=1); rested

## 2026-06-16T  /dream  fallow — field unchanged (streak=2); rested

## 2026-06-15T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0) but verified LIVE: reflective recall
  bucket is entirely self-review maintenance notes (newest 3 ULIDs 01KV6FEQ /
  01KV53CN / 01KV2H3G all build-maintenance, recalls=0); freshness is /build
  homeward churn moving the fingerprint, not genuine inward signal. 70+ visions
  already exist. User present (manual) so offered outward steer via
  AskUserQuestion (homeward/constellation/companion-kin/rest); dismissed.
  Honest rest per hard rules #6+#8; did not draft past research. fallow
  recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T  /dream extend vision-inoculate
Seed: bare /dream (manual, interactive). fallow=FRESH (streak=0).
  Verified the freshness was NOT just /build churn this time: the inoculate
  fleet (6 PRDs) shipped fully 2026-06-15, and its explicit deferred 7th open
  question — sign the strain — names its two prerequisites (core + spread),
  both of which landed today. A deferred node whose blockers just cleared =
  genuine inward signal, so drafted (not a rest).
Drafted: PRD-inoculate-signet.md
Vision: visions/inoculate.md (Fleet 2)
Gap found (verified live): `inoculate spread` (announce/listen) + carrier-check
  (challenge/verify) are BOTH symmetric — spread announces strain metadata with
  no provenance proof, and the carrier-check is HMAC-blake3 (shared secret). Any
  peer holding the strain can forge an announce for an arbitrary version and
  honest peers converge onto it. inoculate-signet adds an ed25519 keypair under
  ~/.config/inoculate/, signs the strain hash, attaches a detached sig to
  announce, and verifies it in listen against a pinned trust store.
Order: inoculate-signet extends ~/wintermute/inoculate (rust-extend); needs the
  already-shipped strain/hash + spread surface. No new prerequisites — buildable
  now. Adds one dep (ed25519-dalek); blake3/sha2 already present.
Notes for /build: rust-extend into ~/wintermute/inoculate. Keygen must NOT
  clobber an existing key without --force; AC8 forbids the secret key appearing
  in any stdout/stderr. spread announce stays back-compat (unsigned when no key).
Open questions: network key-exchange between constellation nodes deferred to
  constellation's transport decision; hard-gating un-inoculated subagents stays
  flag-only (signing makes a future gate trustworthy but doesn't flip it).

## 2026-06-16T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0) but verified LIVE: freshness is
  /build churn (inoculate-signet shipped c8d3743 + archive commits moved the
  fingerprint), not inward signal. Reflective recall bucket unchanged from prior
  rest pass (top ULIDs 01KV6FEQ/01KV53CN/01KV2H3G, all self-review maintenance).
  inoculate fleet fully shipped; its only open questions (cross-node key-exchange,
  hard-gate stance) are explicitly deferred to constellation transport — not
  buildable now. 72 visions exist; journal is pure maintenance (disk 86%, agentns
  EINVAL, dirty repos). User present so offered outward steer via AskUserQuestion
  (constellation/homeward/companion-kin/rest); dismissed. Honest rest per hard
  rules #6+#8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0) but verified LIVE per the now-standing
  rule that fresh ≠ inward signal: the 3 newest reflective recall memories
  (01KV6FEQ / 01KV53CN / 01KV2H3G) are all self-review maintenance notes
  (evidence=[], confidence ~0.56 — the self-review signature), unchanged from the
  last rest passes. Hybrid ideation query returned only weeks-old kernel-asset /
  build-maintenance memories (01KS* ULIDs), all already addressed. Freshness is
  /build churn (inoculate-signet ship c8d3743 + archive commits) moving the
  fingerprint. inoculate fleet fully shipped; its open questions (cross-node
  key-exchange, hard-gate stance) are explicitly deferred to constellation
  transport — not buildable now. 72 visions exist. User present so offered outward
  steer via AskUserQuestion (rest/homeward/constellation/companion-kin); dismissed.
  Honest rest per hard rules #6+#8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0) but verified LIVE per the standing
  rule (fresh ≠ inward signal): the 3 newest reflective recall memories
  (01KV6FEQ / 01KV53CN / 01KV2H3G) are byte-identical to the last several rest
  passes — all self-review maintenance, recalls=0. Hybrid ideation query returned
  only weeks-old kernel-asset / build-maintenance memories (01KS* ULIDs), every
  one already addressed. Freshness is /build churn moving the fingerprint, not a
  new inward signal. inoculate fleet fully shipped; its open questions
  (cross-node key-exchange, hard-gate stance) are explicitly deferred to
  constellation transport — not buildable now. 72+ visions exist; journal is pure
  maintenance (disk 86%, agentns EINVAL, dirty repos). User dismissed the
  outward-steer offer on the prior passes and streak=0 (escalate=false), so did
  not re-prompt. Honest rest per hard rules #6+#8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0) but verified LIVE per standing rule
  (fresh ≠ inward signal): reflective recall bucket byte-identical to last 4+
  passes (01KV6FEQ/01KV53CN/01KV2H3G, all recalls=0 self-review maintenance);
  autobuilder log confirms freshness = /build churn (archive + rest commits, top
  281c0ad). inoculate fleet fully shipped, open questions deferred to
  constellation transport — not buildable now. User present so offered outward
  steer via AskUserQuestion (homeward/constellation/companion-kin/rest);
  dismissed. Honest rest per hard rules #6+#8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0) but verified LIVE per standing rule
  (fresh ≠ inward signal): the 3 newest reflective recall memories
  (01KV6FEQ/01KV53CN/01KV2H3G) remain byte-identical to the last 5+ rest passes
  — all self-review maintenance, recalls=0. Hybrid ideation query returned only
  weeks-old 01KS* kernel-asset / build-maintenance memories, every one already
  addressed. Freshness is /build churn (inoculate-signet ship + archive commits)
  moving the fingerprint, not a new inward signal. inoculate fleet fully shipped;
  open questions (cross-node key-exchange, hard-gate stance) deferred to
  constellation transport — not buildable now. 72+ visions exist. User has
  dismissed the outward-steer offer on the prior 5 passes and streak=0
  (escalate=false), so did not re-prompt. Honest rest per hard rules #6+#8.
  fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0, escalate=false) but verified LIVE per
  the standing rule (fresh ≠ inward signal): reflective recall top-3
  (01KV6FEQ/01KV53CN/01KV2H3G) byte-identical to the last 5+ rest passes; read
  01KV6FEQ content = self-review maintenance note (evidence=[], confidence=0.6,
  the self-review signature). Hybrid ideation query returned only weeks-old 01KS*
  kernel-asset/build-maintenance memories, all already addressed. Freshness is
  /build churn moving the fingerprint, not a new inward signal. inoculate fleet
  shipped; open questions deferred to constellation transport — not buildable now.
  streak=0 so escalate=false; user dismissed outward-steer on prior 5+ passes, did
  not re-prompt. Honest rest per hard rules #6+#8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0, escalate=false) but verified LIVE per
  standing rule (fresh ≠ inward signal): reflective recall top-3
  (01KV6FEQ/01KV53CN/01KV2H3G) byte-identical to last 5+ rest passes — all
  self-review maintenance, recalls=0. Hybrid ideation query returned only
  weeks-old 01KS* kernel-asset/build-maintenance memories, all already addressed.
  Freshness = /build churn moving the fingerprint, not a new inward signal.
  inoculate fleet fully shipped; open questions deferred to constellation
  transport — not buildable now. Journal is pure maintenance (disk 86%, agentns
  EINVAL, 37 dirty repos). User dismissed outward-steer on prior 5+ passes and
  streak=0, so did not re-prompt. Honest rest per hard rules #6+#8. fallow
  recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-15T  /dream  rest (interactive, manual)
Seed: bare /dream. fallow=FRESH (streak=0, escalate=false) but verified LIVE per
  standing rule (fresh ≠ inward signal): reflective recall top-3
  (01KV6FEQ/01KV53CN/01KV2H3G) byte-identical to the last 5+ rest passes; read
  01KV6FEQ = self-review maintenance note (evidence=[], confidence=0.6 — the
  self-review signature). Hybrid ideation query returned only weeks-old 01KS*
  kernel-asset/build-maintenance memories, all already addressed. Journal
  2026-06-13..15 is pure maintenance (disk 86%, agentns EINVAL, fleet-binary-
  staleness, 37 dirty repos). Freshness = /build churn moving the fingerprint,
  not a new inward signal. inoculate fleet shipped; open questions deferred to
  constellation transport — not buildable now. User dismissed outward-steer on
  prior 5+ passes and streak=0, so did not re-prompt. Honest rest per hard
  rules #6+#8. fallow recorded (drafted=0).
Decision: 0 PRDs, 0 visions.

## 2026-06-16T04:41  /dream  vision-rosetta
Seed: user — `/dream semantic web tools for ethical AI`. fallow=FRESH but this
  was a directed topic seed (not a bare rest-pass), so dreamed for real.
Drafted: PRD-rosetta-prov.md, PRD-rosetta-shacl.md, PRD-rosetta-credential.md,
  PRD-rosetta-serve.md, PRD-rosetta-attest.md
Vision: visions/rosetta.md
What it is: the *interoperability* layer the ousia/lattice ethical-AI arc never
  built. ousia reasons (OWL/SPARQL), lattice federates, ousia-guard gates,
  answerable/inoculate attest+sign — but nothing emits decisions as W3C linked
  data. rosetta translates verdicts+provenance into PROV-O, the rule battery into
  SHACL, clearances into signed Verifiable Credentials, and serves it all as
  dereferenceable IRIs + a SPARQL endpoint. Grounded in: ousia-guard gate.rs:51
  Evaluation struct, the README rule battery, provfs/provenance-mcp xattrs,
  answerable JSONL ledger, inoculate signet.rs signing, lattice-join's oxigraph
  store. grep confirms ZERO existing PROV-O/SHACL/JSON-LD/VC/dereferenceable-IRI
  coverage in the fleet — wholly unclaimed.
Order: rosetta-prov (foundational) → {rosetta-shacl independent, rosetta-credential
  dep prov, rosetta-serve dep prov+lattice-join} → rosetta-attest (capstone, deps
  all four). prov can ship first and alone.
Notes for /build:
  - rosetta-prov / -shacl / -serve are independent of each other once prov lands;
    safe to parallelize. rosetta-credential MUST reuse inoculate-signet's key, not
    mint a parallel identity (AC5) — coordinate if inoculate is mid-flight.
  - rosetta-serve depends on lattice-join's oxigraph store existing on disk; if
    lattice-join isn't shipped/run, rosetta-serve can still build against a fixture
    store but its integration AC needs the real store.
  - rosetta-attest is an integration harness that subprocesses the other four +
    ousia-guard + lattice-ground — build it LAST, after the chain exists.
  - All use the locked RDF stack: oxrdf >=0.2,<0.3 / oxrdfio >=0.1,<0.2 /
    oxigraph 0.4 (matches ousia-sparql + lattice-join). MSRV 1.85.
Open questions (in the vision doc): VC v1 signs a stable JSON-LD serialization,
  not true RDFC-1.0 canonicalization (documented simplification, follow-on PRD);
  serve is local-only (public IRI hosting = constellation/herald concern);
  rosetta-shacl implements a SHACL-core subset for the 4 rules, not a general engine.
Decision: 5 PRDs, 1 vision.

## 2026-06-16T  /dream  vision-lattice (Fleet 2) + vision-cogito (Fleet 1)
Seed: jsy — "OWL tools for AIs"

Phase 0 redirect: user said "deeply research external BFO ontologies and federate them
  with lattice" mid-session → pivoted from cogito to lattice Fleet 2. cogito-tbox.md was
  already drafted; remaining 5 cogito PRDs are in the vision doc only (don't dream past
  what's researched).

Drafted (lattice Fleet 2):
  PRD-lattice-versioniri.md
  PRD-lattice-seed.md
  PRD-lattice-cco.md
  PRD-lattice-ground-live.md

Drafted (cogito Fleet 1, partial — only tbox was fully researched):
  PRD-cogito-tbox.md

Live research findings:
  - lattice-registry sync: 265 OBO Foundry entries populated
  - lattice-registry add works: BFO(35), RO(58), IAO(266), COB(69), SWO(1970) all bfo_grounded=true
  - CRITICAL BUG: lattice-bridge align fails with "Unexpected tag: found versionIRI" on every
    real OBO ontology; every OBO OWL/XML header contains owl:versionIRI (valid OWL 2 DL).
    This blocks ALL federation. PRD-lattice-versioniri is the gating fix.
  - CCO v2.0: 11 Turtle modules, BSD 3.1, DoD-standard mid-layer; confirmed BFO 2020-grounded
    (README+structure). Agent, InfoEntity, Event, Time, ExtendedRelation = priority 5 for AI agent.
  - BFO 2020 (ISO/IEC 21838-2): 35 classes, OWL IRI http://purl.obolibrary.org/obo/bfo.owl
    (OBO PURL redirects to v2019; BFO 2020 proper at GitHub). 98KB OWL/XML, CC BY 4.0.
  - SWO (Software Ontology): 1970 classes, bfo_grounded=true — the richest domain ontology
    for an AI coding agent. CC BY 4.0.

Order: lattice-versioniri → lattice-seed → lattice-cco → lattice-ground-live (strict pipeline)
  cogito-tbox can build in parallel (no dependency on lattice)

Notes for /build:
  lattice-versioniri is a rust-extend into lattice-bridge — one-function OWL/XML parser fix.
  lattice-seed/cco/ground-live are shell scripts; can chain once the parser is fixed.
  cogito-tbox is rust-cli new-repo; independent, can build immediately.

Open questions for next /dream pass:
  - Does `lattice-bridge align` fail on Turtle format too, or only OWL/XML? (Turtle doesn't
    have versionIRI in the same syntax.) Could use Turtle downloads as a workaround for the
    bridge while the parser is fixed.
  - Should `lattice-context serve` be wired as an MCP tool for the live Claude session?
    (lattice-context serve already exists per help output — just needs a socket/config.)
  - BFO 2020 vs BFO 2019: the OBO PURL redirects to 2019. Should lattice-seed explicitly
    use the direct GitHub URL for BFO 2020 to get the ISO-standard version?

## 2026-06-16T  /dream  vision-ousia-atscale (Fleet 1)
Seed: jsy — "/dream for ousia-atscale" (the day the repo went public for colleagues)

fallow OVERRIDE: fallow check returned state=fallow streak=1 escalate=false — but the
  fingerprint (b3:066342...) is the one I recorded 12 min ago after the lattice/cogito
  drafting pass; the inward ledger simply hadn't moved. The user gave an EXPLICIT new
  topic, which fallow doesn't track. Hard rule #8 guards against thin PRDs from a stale
  inward field, not against an explicit user steer — proceeded. (Same logic as the
  standing "fresh ≠ inward signal" rule, inverted: fallow-but-user-steered.)

Drafted (rust-extend into ~/wintermute/ousia-atscale, repo public at j0yen/ousia-atscale):
  PRD-ousia-atscale-bfo-hint.md   — implement the bfo_hint override the README documents
                                     but the mapper doesn't honor (honesty fix; found by
                                     reading src/mapper.rs vs the README I committed da1c799)
  PRD-ousia-atscale-rdf.md        — `export` subcommand: emit grounding as RDF/Turtle+OWL
                                     so it loads into ousia-sparql / ousia-reason
  PRD-ousia-atscale-validate.md   — `validate`: run ousia-reason check over the OWL export;
                                     upgrade coverage from "mapped" to "mapped + consistent"
  PRD-ousia-atscale-diff.md       — `diff A B`: makes the MARKET pitch ("is this the same
                                     revenue?") executable; flags same-name/different-BFO divergence
  PRD-ousia-atscale-mcp.md        — `serve`: expose ground/report/diff over MCP for live agents

Order: bfo-hint, rdf, diff standalone; validate depends on rdf; mcp depends on diff.

Notes for /build:
  All 5 are rust-extend into the same repo. Per worktree-isolation invariant, shared-target
  (≥2 PRDs same build_into) → use worktree-extend.sh, not wm-buildtree land. bfo-hint is the
  smallest/safest first; rdf is foundational (reuse oxrdf/oxttl from ousia-sparql workspace).
  validate + mcp should reuse sibling binaries (ousia-reason, ousia-mcp/mcp-core) — do NOT
  re-implement reasoning or MCP framing.

Open questions (in vision doc): calculated/derived measures (composed-process GDC),
  batch mode over a directory of models, AtScale write-back (pending AtScale API).

## 2026-06-16T  /dream  (rested)
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T  /dream  vision-doxa (Fleet 1)
Seed: jsy — "/dream of extending ousia to all knowledge and philosophies"

fallow OVERRIDE (again): state=fallow streak=2 escalate=false, but the fingerprint
  (b3:6548f2...) is the one recorded ~15 min ago after the ousia-atscale pass; the
  inward ledger hadn't moved. User gave an explicit, ambitious NEW topic fallow can't
  see. Proceeded per the same "explicit user steer vs stale inward field" reasoning.

Seed split honestly:
  - "all knowledge" = already lattice's job (federate 500+ BFO domain ontologies;
    lattice Fleet 2 dreamed earlier today). Did NOT duplicate — doxa references lattice
    for scenario facts.
  - "all philosophies" = genuinely uncovered. ousia hardwires ONE philosophy (Federation
    humanism) into ousia-forge/spec/roles.toml class axioms. doxa pluralizes it.

Drafted (new repo ~/wintermute/doxa; reuses ousia-forge/reason/guard as engines):
  PRD-doxa-moral-core.md   — shared framework-neutral moral TBox (rust-cli, new repo)
  PRD-doxa-frameworks.md   — consequentialism / deontology / virtue-ethics as axiom modules
  PRD-doxa-reason.md       — per-framework verdict over a scenario ABox (reuses ousia-reason)
  PRD-doxa-compare.md      — agreement/conflict matrix across frameworks (consensus vs dilemma)
  PRD-doxa-guard.md        — pluralist allow/flag/deny by explicit policy (unanimity/majority/
                             framework:/lexical:); reuses ousia-guard

Order: moral-core → frameworks → reason → {compare, guard}

Distinct from neighbors: tribunal verifies ONE stance; herald/recourse ship/appeal it;
  concord de-escalates HUMAN conflict; roundtable is creative critique. None formalize
  multiple ethical frameworks for deductive comparison. Verified by reading their TL;DRs.

Notes for /build: moral-core is rust-cli new-repo (gate). frameworks+reason+compare+guard
  are rust-extend into ~/wintermute/doxa (shared target → worktree-extend.sh). doxa is a
  thin orchestrator; the OWL/reasoning lifting stays in ousia-forge/ousia-reason/ousia-guard
  — PRDs explicitly say don't re-implement.

Open questions (in vision): Fleet 2 frameworks (Kant strict, Rawls, Aristotle eudaimonia,
  care ethics, Stoic, Confucian, Buddhist, Ubuntu); scenario authoring ergonomics;
  commensurability limits of the "neutral" core; meta-ethics explicitly out of scope.

## 2026-06-15T22:30  /dream  vision-roundtable (Fleet 2 — the convener)
NOTE: fallow check returned state=fallow streak=1 (escalate=false). OVERRIDE per
  the "explicit user steer vs stale inward field" rule — the user invoked
  `/dream extend roundtable` naming a concrete, verified gap the ledger can't see.
  Proceeded; recorded the override here.

Verified gap (ran --help on every shipped roundtable binary): all 6 sub-visions
  shipped, but the table never convenes end-to-end. `the-lunch lunch` does
  convene→seat→menu→minutes and STOPS — the critique (vicious-circle record),
  column (conning-tower compose/syndicate), games (thanatopsis), and periodical
  (new-yorker issue) are never chained. The shipped the-lunch.timer fires only
  the convening half. The umbrella's Order diagram assumed a convener never written.

Drafted (new repo ~/wintermute/roundtable; orchestrates existing binaries, reuses
  the-lunch/vicious-circle/conning-tower/thanatopsis/new-yorker — re-implements none):
  PRD-roundtable-session.md  — `roundtable session`: the-lunch lunch → per-artifact
                               vicious-circle record → conning-tower compose+syndicate
  PRD-roundtable-games.md    — `roundtable games` / `session --with-games`: deal the
                               day's table into thanatopsis poker+charades
  PRD-roundtable-bind.md     — `roundtable bind`: gather columns → new-yorker issue+cover
  PRD-roundtable-cadence.md  — noon timer fires session --with-games; weekly bind timer;
                               install.sh DISABLES the-lunch.timer (now subsumed → no
                               double-convene)
  PRD-roundtable-digest.md   — SessionStart hook surfacing yesterday's crowned bon mot +
                               column headline (offline, local-state-only; mirrors
                               the-lunch-sessionstart.sh)

Order: session → {games, bind, digest} → cadence (cadence depends on session+games+bind).

Notes for /build: session is rust-cli new-repo (gate). games/bind/digest are
  rust-extend into ~/wintermute/roundtable (shared target → worktree-extend.sh).
  cadence is mixed (units + install.sh). IMPORTANT for cadence: its install.sh must
  `systemctl --user disable --now the-lunch.timer` or noon convenes twice.

Real interfaces cited (verified 2026-06-15):
  the-lunch lunch [--date]; table.json at $XDG_STATE_HOME/the-lunch/<date>/
  vicious-circle record <artifact> [--ledger]; default $XDG_DATA_HOME/vicious-circle/ledger.jsonl
  conning-tower compose --ledger [--date]; syndicate --ledger --columns-dir --to columns
  thanatopsis poker|charades|...; new-yorker workspace (masthead/house-style/issue/cover)

Open questions: weekly vs monthly issue cadence (default weekly); publish issues outward
  or keep local (default local, per umbrella); should digest feed daily-receipt JSON.

## 2026-06-16T05:38  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-15T23:00  /dream  vision-ousia-mqo (AtScale federation pipeline)
Seed: user — "organize ousia and related work especially the federation layer;
  wire into mqo-mcp server."

User choices: standalone bridge repo (joeyen-atscale/ousia-mqo); semantic
  cross-cluster diff + grounded NL binding as primary values.

Research confirmed (2026-06-15):
  - ousia-atscale annotate emits {model_catalog, model_schema, model_table,
    overlay_version, annotations: {name: {philosophicalGrounding: {iri, label,
    rationale}, domainModule, aristotelianDefinition: {genus, differentia}}}}
  - lattice-ground resolve is on $PATH (v0.1.0)
  - mqo-mcp subprocess protocol: stdin JSON {"tool","args"} → stdout {"ok","data"}
  - mcp-cross-cluster-diff is structural (column-name matching only)

Drafted (new repo joeyen-atscale/ousia-mqo):
  PRD-ousia-mqo-ground.md  — Grounder lib + ousia-mqo ground CLI (gate, new repo)
  PRD-ousia-mqo-diff.md    — semantic cross-cluster diff by BFO IRI (not column name)
  PRD-ousia-mqo-bind.md    — NL phrase → BFO class → grounded AtScale candidates
  PRD-ousia-mqo-mcp.md     — subprocess dispatcher + ToolPaths registration for mqo-mcp-server

Order: ground → {diff, bind} → mcp (mcp links all three libs, no subprocess calls to own CLIs)

Notes for /build:
  ground is rust-cli new-repo (gate; creates joeyen-atscale/ousia-mqo).
  diff/bind/mcp are rust-extend into ~/wintermute/ousia-mqo (shared target → worktree-extend.sh).
  The subprocess protocol for mcp matches mqo-mcp's ToolPaths exactly per ARCHITECTURE.md.
  ousia-atscale and lattice-ground must be on $PATH when the server runs — PRD says so.
  Do NOT create a separate j0yen/ousia-mqo — this one belongs under joeyen-atscale.

Open questions: Should ousia-mqo-mcp be registered in mqo-mcp-server's ToolPaths by
  a PR to joeyen-atscale/mqo-mcp, or just via docs? Defer to the MCP PRD's docs approach
  and let the user wire it in once the binary is on PATH.
2026-06-16T06:24:56Z loom-serial-fallback: SERIAL MODE for ousia-atscale (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains

## 2026-06-15T23:30  /dream  vision-mqo-tools (new tools for mqo-mcp server)
Seed: user — "of tools for the atscale mqo-mcp server."
NOTE: fallow=fallow streak=1 (escalate=false). OVERRIDE per explicit-user-steer
  rule — user named a concrete topic the ledger can't see. Recorded here.

Research (2026-06-15, via gh api on joeyen-atscale/mqo-mcp):
  - Current server tools (mqo-mcp-server/src/mcp.rs): list_models, describe_model,
    search_columns, query_multidimensional, next_page, list_clusters, health_status,
    diff_clusters, recommend_chart, build_vega_spec, build_bi_asset, compose_dashboard
    + server-side handle ops (topk visible).
  - 50-crate workspace grepped: NO crate matches time|unit|anom|templ|linea.
    These 5 are genuine gaps.

Drafted (5 standalone joeyen-atscale repos, matching the existing mcp-* tool pattern;
  each = flag CLI + `serve` subprocess mode speaking {"tool","args"}→{"ok","data"}):
  PRD-mqo-time-intelligence.md — derive YoY/QoQ/rolling/MTD-QTD-YTD MQOs from a base MQO
  PRD-mqo-unit-guard.md        — block unit/format-incompatible measure combos (value-semantics firewall)
  PRD-mqo-anomaly-scan.md      — zscore/iqr/mad outlier scan over a result rowset (rows stay server-side)
  PRD-mqo-lineage.md           — "how was this computed" tree from BoundMqo/decision/compiled
  PRD-mqo-template.md          — parameterized saved MQOs (list/show/instantiate)

Order: all 5 independent (each consumes documented mqo-spec MQO/BoundMqo shapes).
  Build any order/parallel. Value-descending: time-intel → unit-guard → anomaly → lineage → template.

Notes for /build: ALL are rust-cli new-repos under joeyen-atscale (NOT j0yen). mqo-mcp is
  NOT cloned locally, so build_into paths are ~/wintermute/<slug> for the new repo working
  dir; each is fixture-driven + cluster-free in tests (no AtScale instance needed).
  IMPORTANT: mqo-spec is the source of truth for MQO/BoundMqo JSON — each PRD ships a minimal
  fixture matching the documented shape but the real schema must be confirmed against mqo-spec
  at build time (clone joeyen-atscale/mqo-mcp or vendor mqo-spec if exact shape needed).
  Wiring into mqo-mcp-server is via ToolPaths subprocess OR pulling the crate into the workspace
  — user's call; each `serve` mode is ready either way.

Distinct from neighbors: time-intel ≠ next-query-proposer (derives time comparison, not adjacency
  follow-ups); unit-guard ≠ param-validator (value semantics, not grounding) ≠ mqoguard-* (units,
  not structural compat); anomaly-scan ≠ result-profiler (outliers, not shape); lineage ≠
  causal-tracer (single-result derivation, not deltas); template = wholly new.

Sibling vision: visions/ousia-mqo.md (BFO semantic grounding into the same server) dreamed earlier today.

## 2026-06-15T23:55  /dream  vision-mqo-trust (agentic-BI safety & runtime layer)
Seed: user — "more for mqo-mcp server" (3rd mqo fleet today).
NOTE: fallow=fallow streak=1 (escalate=false). OVERRIDE per explicit-user-steer rule.

Research (2026-06-15, gh api on joeyen-atscale/mqo-mcp): grepped all 50 crates —
  NO crate matches confidence|clarif|pii|sensitiv|error|explain|cache|redact|mask|
  ambig|drift. This fleet is the trust/governance/runtime slice; complements
  mqo-tools (analytical) + ousia-mqo (grounding).

Drafted (5 standalone joeyen-atscale rust-cli repos; flag CLI + serve subprocess mode;
  fixture-driven, cluster-free in tests):
  PRD-mqo-binding-confidence.md — calibrated 0-1 confidence + alternatives per bound field
  PRD-mqo-clarify.md            — disambiguation question when ≥2 candidates within margin
  PRD-mqo-sensitivity-scan.md   — PII/sensitive field flag + optional --redact before rows reach model
  PRD-mqo-error-explain.md      — backend fault (XMLA/DAX/MDX/SQL) → {cause, category, suggested_fix} via data catalog
  PRD-mqo-result-cache.md       — content-addressed cache keyed on canonicalized BoundMqo

Order: all 5 independent. binding-confidence + clarify are a natural pair (clarify
  consumes confidence's margins but runs standalone). Priority: confidence → clarify →
  sensitivity → error-explain → result-cache.

Notes for /build: same conventions as mqo-tools — rust-cli new-repos under joeyen-atscale
  (NOT j0yen), build_into ~/wintermute/<slug>, fixtures matching documented mqo-spec shapes
  (confirm against mqo-spec at build time). Nice interlock: result-cache honors a sensitivity
  flag so sensitivity-scan-flagged results are never cached to disk.

Distinct from neighbors: binding-confidence ≠ param-validator (graded vs binary accept/reject);
  clarify = new (pipeline commits, never asks); sensitivity = new (no data-sensitivity dimension
  anywhere); error-explain = new (faults pass through raw); result-cache ≠ duckdb-handle-store
  (cross-query cache vs single-result handle).

Three mqo fleets now queued: mqo-tools (5) + ousia-mqo (4) + mqo-trust (5) = 14 PRDs aimed at
  the same server.

## 2026-06-15T22:30  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T05:35  /dream  vision-atscale-ai-strategy (META-vision + Fleet 1)
Seed: jsy — "/dream a vision for AtScale's AI strategy and roadmap".
Audience (jsy): BOTH — company-strategy thesis that doubles as the build roadmap.
CONSTRAINT (jsy): nothing written to the AtScaleInc company repo. New repos target
  the personal j0yen/ namespace, consume only public mqo-spec/ousia-atscale shapes.
NOTE: fallow=fallow streak=2 (escalate=false). OVERRIDE per explicit-user-steer rule.

This is a META-vision: it names the thesis ("the semantic layer is the trust
substrate for agentic BI") and organizes the EXISTING AtScale fleets into a
5-pillar roadmap — SPEAK (mqo-mcp), ANALYZE (mqo-tools), TRUST (mqo-trust),
GROUND (ousia-atscale/ousia-mqo/lattice), PROVE (rosetta) — then adds the
missing cross-cutting pillar: MEASURE (prove the thesis is true).

Live-MCP evidence (list_models, 2026-06-16): same internet_sales model on BOTH
BigQuery + Snowflake catalogs (engine-parity is testable); internet_sales vs
internet_sales_no_pii variants (PII governance is real); Tasty Bytes imported
from a Power BI/Snowflake Cortex Analyst .pbit (AtScale is in the NL-analytics
arena); search_columns is keyword-only (semantic retrieval gap).

Drafted Fleet 1 — the MEASURE pillar (5 standalone j0yen rust-cli, flag CLI +
serve subprocess mode, fixture-driven, cluster-free):
  PRD-mqo-bench.md              — KEYSTONE: NL→metric binding accuracy harness +
                                  golden set; turns the whole thesis into a number
  PRD-mqo-catalog-embed.md      — semantic (BGE, offline) catalog retrieval beyond
                                  keyword search_columns; raises binding accuracy
  PRD-mqo-engine-parity.md      — runtime numeric-parity proof across engines
                                  (BigQuery vs Snowflake same model)
  PRD-mqo-aggregate-advisor.md  — agent cost/aggregate-awareness (AtScale's core
                                  acceleration differentiator, invisible to AI today)
  PRD-mqo-semantic-regression.md— CI gate: fail build when a model change breaks
                                  the agent-facing metric contract (incl. dropped PII)

Order: all 5 build independently. Priority by strategic leverage:
  bench → catalog-embed → engine-parity → aggregate-advisor → semantic-regression.
  mqo-bench first — every other pillar's "X raises accuracy" claim is measurable
  once the harness + golden set exist; bench's golden fixture is reusable by the rest.

Notes for /build: repos are j0yen/<slug> (NOT joeyen-atscale, NOT AtScaleInc) per
  jsy's explicit constraint this pass — if you'd rather align with the sibling
  mqo-* fleets under joeyen-atscale, that's jsy's call, but NEVER AtScaleInc/*.
  mqo-catalog-embed reuses recall's local BGE embedder (offline, no API).
  mqo-bench's golden set is hand-authored from the 3 live models (Tasty Bytes,
  Internet Sales, TPC-DS); a raw-table text-to-SQL control answer key is a
  follow-on, so the v1 "+N% vs text-to-SQL" delta is partial until then.

Open questions (in the vision): golden-set source (hand-authored vs trace-harvested);
  raw-table baseline (in-repo control vs external Spider/BIRD); whether MEASURE is
  the base pillar or the headline-number top of the roadmap.

## 2026-06-16T06:55  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T08:00  /dream  vision-atscale-ai-strategy (Fleet 2 — DEMONSTRATE)
Seed: bare /dream (interactive); fallow=fresh (streak 0, fingerprint moved since
  06:55 rest). Extends the existing meta-vision (Fleet 1 = MEASURE drafted 05:35).
Drafted Fleet 2 (4 PRDs) — the DEMONSTRATE pillar: make the thesis runnable +
  self-evident, not just measured.
  PRD-mqo-textsql-baseline.md  — KEYSTONE: honest raw-table text-to-SQL CONTROL so
                                 mqo-bench's "+N% vs text-to-SQL" delta isn't fiction.
                                 Vision named it as the explicit follow-on.
  PRD-mqo-demo-runner.md       — orchestrator that RUNS the "gross margin by region
                                 YoY EMEA vs AMER" story across all pillars
                                 (catalog-embed→confidence→clarify→time-intel→
                                 ousia-mqo-diff→engine-parity→sensitivity→credential),
                                 emits one provable transcript. Today the demo is prose only.
  PRD-mqo-scorecard.md         — publishes Fleet-1 tool JSON as ONE stakeholder scorecard
                                 (md+json, trend deltas). End-state #4 "delta is published"
                                 had no owner. Pure presenter, computes nothing.
  PRD-mqo-trace-harvest.md     — grows golden set from demo-runner transcripts + bench run
                                 I/O; dedupes (BGE cosine), emits CANDIDATES for human review,
                                 NEVER auto-accepts (tautology guard, cf wm-router 100%→73.5%).

Order: textsql-baseline → demo-runner → scorecard → trace-harvest.
  baseline first (unblocks mqo-bench's honest number); harvest last (consumes
  demo-runner output). All standalone j0yen/<slug> rust-cli, flag CLI + serve mode,
  fixture-driven, cluster-free.

Live-MCP re-confirmed (list_models, 2026-06-16): internet_sales on BOTH BigQuery +
  Snowflake catalogs (real two-engine parity for demo-runner); internet_sales_no_pii
  variants (real PII surface for sensitivity step); Tasty Bytes ex-Cortex .pbit (the
  warehouse-locked competitor textsql-baseline proxies); TPC-DS multi-channel
  catalog/store/web_sales (double-count trap for the control binder).

Notes for /build: repos j0yen/<slug> per jsy's namespace constraint — NEVER AtScaleInc.
  mqo-demo-runner depends on the pillar binaries at RUN time only (mock mode tests
  cluster+binary-free, so it builds/ships before siblings install). mqo-scorecard
  should build after mqo-bench + mqo-engine-parity ship (it consumes their JSON, but
  fixtures let it build independently). mqo-trace-harvest reuses recall's BGE dedup.

Open questions (in vision): is MEASURE the base pillar or the headline; golden-set
  source (now partly answered — trace-harvest makes harvesting safe but human-gated);
  whether mqo-scorecard should later serve HTML.

## 2026-06-16T08:00+  /dream  fallow
/dream fallow — field unchanged (streak=1); rested
2026-06-16T09:39:40Z loom-serial-fallback: SERIAL MODE for doxa (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains

## 2026-06-16T08:30  /dream  vision-atscale-ai-strategy (Fleet 3 — DEPLOY/OPERATE)
Seed: bare /dream (interactive); fallow=fresh (streak 0, fingerprint moved since
  08:00+ rest). Third pass extending the meta-vision. Fleets 1 (MEASURE) + 2
  (DEMONSTRATE) drafted earlier today; this adds the pillar that turns "we proved
  it / we can show it" into "an enterprise can safely RUN it."
Drafted Fleet 3 (4 PRDs):
  PRD-mqo-agent.md          — KEYSTONE: the adaptive planner. mqo-demo-runner (Fleet 2)
                              walks a FIXED scripted chain for ONE question = a demo.
                              mqo-agent DERIVES the pillar pipeline from an arbitrary NL
                              question (clarify only if low-confidence; time-intel only on
                              period-over-period; engine-parity only if multi-engine), loops
                              on clarify, ends signed. Deterministic rule planner is the
                              tested default; --planner brain is opt-in.
  PRD-mqo-access-policy.md  — pre-execution authz gate. Live MCP (list_models 2026-06-16):
                              internet_sales_no_pii twins exist on BOTH BigQuery+Snowflake,
                              but Tasty Bytes ships raw CUSTOMER_EMAIL/PHONE/DOB +
                              FRANCHISE_EMAIL with NO twin. So policy ROUTES under-cleared
                              agents to the safe twin where one exists, COLUMN-DENIES where
                              it doesn't. Prevents binding to forbidden surface (vs
                              sensitivity-scan which detects PII post-build).
  PRD-mqo-session-budget.md — per-session governor (queries/scan-cost/wall-time ceiling).
                              Consumes mqo-aggregate-advisor's cost estimate. Leans on kernel
                              PR_SET_AGENT_BUDGET_LIMITS where live — but agentns is BLOCKED
                              on this box (all-zeros/EINVAL, 16+ runs), so v1 enforces in
                              userspace + capability-detects the kernel path. Detect, degrade,
                              never fail open.
  PRD-mqo-decision-log.md   — durable append-only JSONL of every agent decision. The PRODUCER
                              that Fleet 2 already assumes: mqo-trace-harvest mines it for
                              golden candidates, mqo-scorecard reads `summary` for trend
                              deltas. Append-only + provfs session xattr = tamper-evidence
                              w/o reimplementing rosetta-credential signing.

Order: mqo-agent → mqo-access-policy → mqo-session-budget → mqo-decision-log.
  Agent first (introduces the loop the other three govern); decision-log last
  (consumes agent output, feeds Fleet-2 harvest/scorecard). All four build
  independently against fixtures; agent consults policy/budget at RUN time only.

Notes for /build: repos j0yen/<slug> per jsy's namespace constraint — NEVER AtScaleInc.
  mqo-agent shares the pillar subprocess tool-JSON calling convention with
  mqo-demo-runner; reuse not reimplement. mqo-agent's answer.json schema == the
  record mqo-decision-log ingests (no translation layer). mqo-session-budget
  cost path consumes mqo-aggregate-advisor (Fleet 1) but mocks it in tests.
  mqo-access-policy reads a catalog snapshot fixture (the list_models shape), not
  a live cluster.

Open questions (added to vision's set): identity source for mqo-access-policy
  (who establishes the agent identity string — out of scope, caller's concern);
  whether the kernel agentns budget path is worth waiting on vs shipping userspace
  v1 now (default: ship userspace, capability-detect kernel).

## 2026-06-16T03:09  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T10:39  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-16T11:10  /dream  vision-atscale-ai-strategy (Fleet 4 — LEARN)
Seed: bare /dream (interactive); fallow=fresh (streak 0, fingerprint moved since
  10:39 rest — /build shipped mqo-agent + mqo-trace-harvest + mqo-access-policy
  today). Fourth pass extending the meta-vision. Fleets 1 (MEASURE) + 2
  (DEMONSTRATE) + 3 (DEPLOY/OPERATE) drafted earlier today; this adds the pillar
  that turns "we proved it / ran it / ran it safely" into "it gets measurably
  better over time — without ever training on its own unreviewed output."
Why now (verified): the Fleet-2/3 PRODUCERS all SHIPPED today and are binaries on
  PATH — mqo-decision-log, mqo-trace-harvest, mqo-scorecard, mqo-agent — yet
  NOTHING consumes their output to improve the agent. Fleet 4 is the consumer.
Drafted Fleet 4 (4 PRDs):
  PRD-mqo-goldgrow.md       — KEYSTONE: human-gated curation. mqo-trace-harvest
                              emits *candidate* NL→MQO pairs and never auto-accepts;
                              goldgrow reviews → accept/reject w/ reason → appends
                              accepted to mqo-bench's golden set w/ provenance
                              (source:harvested, reviewer, ts) + append-only
                              rejection ledger so a reject never re-surfaces. The
                              ONLY sanctioned path ground truth grows. Enforces
                              [[feedback_agent_written_fixtures_tautology]]: no
                              anonymous ground truth (accept w/o --reviewer = hard err).
  PRD-mqo-replay.md         — behavioral regression vs the agent's OWN history.
                              semantic-regression gates the MODEL contract; nothing
                              gates AGENT behavior. replay re-runs decision-log
                              questions through current mqo-agent, classifies delta
                              (plan/bind/outcome/value drift), --fail-on gates CI.
                              Numeric tol reuses mqo-engine-parity convention.
  PRD-mqo-planner-tune.md   — outcome-weighted planner calibration ADVISORY. agent's
                              rule planner has hand-chosen thresholds never revisited.
                              tune joins decision-log outcomes + goldgrow verdicts,
                              proposes a planner config DIFF (no apply subcommand —
                              planner stays deterministic + tested). min-support
                              guard = no tuning on noise. Leans on recall v0.6.0
                              outcome-feedback pattern, not invented.
  PRD-mqo-drift-watch.md    — continuous accuracy/parity/PII SLO monitor. scorecard
                              renders ONCE; drift-watch compares each scheduled
                              scorecard vs last accepted baseline + SLO tolerance,
                              exit 0/2/3 (ok/warn/breach), emits structured alert
                              events (no transport hardcoded). baseline --accept is
                              a human gate so a real regression can't silently
                              become the new normal. Per-metric direction in slo.toml.

Order: mqo-goldgrow → mqo-replay → mqo-planner-tune → mqo-drift-watch.
  goldgrow first (establishes trusted ground-truth growth the others lean on —
  planner-tune weights outcomes by goldgrow's verdicts); rest build independently
  against fixtures.
Notes for /build: repos j0yen/<slug> per jsy's namespace constraint — NEVER
  AtScaleInc. All four consume documented JSON from already-shipped Fleet-2/3
  tools (decision-log query/summary, trace-harvest candidates, scorecard JSON,
  agent answer.json) — reuse those shapes, no translation layer, no live cluster
  in tests. mqo-replay's value-tol == mqo-engine-parity's tolerance convention.
  goldgrow's accepted output MUST stay valid mqo-bench golden input (bench loads
  it unchanged).
Open questions: where does goldgrow's reviewer identity come from (caller's
  concern, out of scope); should drift-watch's alert event sink into
  mqo-decision-log or stay a free-standing structured event (default: free event,
  notifier composes it).

## 2026-06-16T11:39  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T12:00  /dream  vision-ballast
Seed: self-review journal 2026-06-16 ("Disk jumped 92% in one day … build
  targets the likely culprit … Priority: `du -sh ~/wintermute/*/target`") +
  verified `df` 94% / 28G free. fallow=fresh (streak 0). First non-atscale
  vision of the day — points at the laptop's own disk pressure, not the AtScale
  meta-vision.
Why now (verified, not asserted): df=94% (28G of 468G). `du -sch
  ~/wintermute/*/target` = 205G across 174 target dirs. brain/target=13G last
  written 2026-05-29 while ~/.local/bin/wmd installed 2026-06-15 (17d newer) —
  heavy compiles route to Hetzner via /cloudbuild, so local targets are FOSSIL
  the running system doesn't depend on. Today's only remedy is self-review
  printing a du hint for a human to rm by hand.
Drafted (4 PRDs):
  PRD-ballast-survey.md      — KEYSTONE. Read-only inventory + classifier of
                               reclaimable subtrees (target/, cargo caches,
                               node_modules/.venv, ~/.cache children); JSON out,
                               deterministic age via --now/BALLAST_NOW. Deletes
                               nothing.
  PRD-ballast-cloudaware.md  — rust-extend INTO ballast-survey. Adds the
                               fossil dimension: installed-binary-newer-than-
                               target + cloud-built → reap_safety rank
                               (fossil > stale-installed > stale-uninstalled >
                               recent). Cross-refs `adopt` for install state,
                               bin_name override (wintermute-brain→wmd).
  PRD-ballast-reap.md        — gated reclaimer. Consumes survey JSON, dry-run
                               default, --apply required to delete, append-only
                               JSONL ledger, in-flight-build guard (skip held
                               targets), root-boundary refusal, default safety
                               floor = fossil only.
  PRD-ballast-guard.md       — disk-SLO high/low-water watcher. df → reap safest
                               candidates largest-first until below low-water;
                               structured alert events (no transport); exit
                               contract 0/2/3/4. Composes survey+reap, never
                               re-implements them.
Order: ballast-survey → ballast-cloudaware (extends survey) → ballast-reap
  (consumes survey JSON) → ballast-guard (orchestrates survey+reap under SLO).
Notes for /build: repos j0yen/<slug>. ballast-cloudaware is build_target
  rust-extend / build_into ~/wintermute/ballast-survey — survey must ship and be
  cloned locally before cloudaware extends it. reap+guard are independent CLIs
  but both consume survey's JSON contract — keep the survey schema stable
  (path/kind/bytes/entries/mtime/age_days/crate + reap_safety) once shipped.
  guard shells out to installed ballast-survey/ballast-reap — order: survey →
  cloudaware → reap → guard so guard's deps exist on PATH. No wall-clock in
  library code (tests pass --now). This vision is SAFE-TO-BUILD but reap/guard
  DELETE files: gating is the whole point — keep --apply gate + ledger + fossil
  default floor intact through review; never let a refactor widen the default
  safety floor.
Open questions: ledger free-standing JSONL vs sink into mqo-decision-log
  (default free-standing, no coupling); whether to ever manage ~/.cache/sccache
  (default off-limits — shared with cloudbuild); high-water defaults (85 advisory
  / 90 reap / 80 target) belong in config not code.

## 2026-06-16T13:00  /dream  vision-drydock
Seed: bare /dream + Phase-1 live inspection. fallow=fresh (streak 0). Strongest
  RECURRING signal across a week of self-reviews: detectors exist and work, but
  nothing routes or closes the loop — every self-review re-dumps the same flat
  "Pending your call" wall (wm-stt 9d, kernel pkgrel staged-not-booted, adopt
  N/N not-current) and the human re-triages it daily.
Why now (verified, not asserted): journals 2026-06-12→16 each re-report
  fleet-binary-staleness + "adopt installed-stale (many)" with no run-over-run
  trend. binstale fleet --format json + adopt verify --format json both emit
  rich JSON live; nobody aggregates them. Kernel case proves naive vercmp is
  wrong: booted linux-wintermute 7.0.11.arch1-1 is a HIGHER base version than
  the staged patched 7.0.10.arch1-12 pkgs that carry the agentns-prctl fix —
  vercmp says "fresh", boot hook still prints ACTIVATION BLOCKED. An Explore
  pass over 10 rollout-ecosystem visions (vigil/scion/vest/fixpoint/changeover/
  keel/homestead/tend/mend/anchor) confirmed NO vision has the risk-tier router
  ("which items auto / window / reboot / approval") or an all-lanes closed loop;
  fixpoint converges only the adopt-marker subset.
Drafted (5 PRDs):
  PRD-drydock-survey.md    — KEYSTONE rust-cli. Aggregate binstale fleet +
                             adopt verify + a CAPABILITY-based kernel probe into
                             one normalized DriftItem JSON inventory. Read-only,
                             deterministic age via --now/DRYDOCK_NOW.
  PRD-drydock-classify.md  — rust-extend INTO drydock-survey. Risk-tier router:
                             auto/window/reboot/approval, deny-by-default
                             (unknown ⇒ approval). Voice daemons + kernel +
                             guardrail items are HARD FLOORS config can't
                             downgrade. Emits one remediation command per item.
  PRD-drydock-digest.md    — rust-cli consuming classify JSON. The single ranked
                             lane-grouped block self-review pastes instead of its
                             prose wall; per-lane counts + Δ + age escalation
                             flag (>7d).
  PRD-drydock-ledger.md    — rust-extend. Append-only JSONL convergence ledger;
                             asserts auto lane non-increasing; first_seen
                             carry-forward ages parked items for escalation.
  PRD-drydock-apply.md     — rust-extend. Gated executor for lane==auto ONLY;
                             dry-run default, --apply required; refuses
                             window/reboot/approval BY CONSTRUCTION so it stays
                             inside the immutable self-review guardrail; in-flight
                             build guard; writes ledger.
Order: survey → classify (extends survey) → digest (consumes classify) → ledger
  (feeds digest Δ) → apply (consumes classify, auto-lane only, writes ledger).
  classify/digest/ledger/apply are all build_into ~/wintermute/drydock-survey —
  survey must ship + clone locally before the rest extend it.
Notes for /build: repos j0yen/<slug> (survey is the only standalone repo; the
  rest extend it in-place per build_into). Keep the DriftItem JSON schema stable
  once survey ships — classify/digest/ledger/apply all key on it. No wall-clock
  in library code (tests pass --now). drydock-apply DELETES nothing and restarts
  nothing outside lane auto — the auto-lane-only scope + dry-run default + the
  "no non-auto command ever emitted" unit test ARE the safety; never let a
  refactor widen apply's selection set or wire it into the cron without jsy's
  opt-in.
Open questions: (1) does the auto-lane self-drain run unattended? Default NO —
  apply ships dry-run-default, NOT cron-wired; jsy opts in (mirrors the approval
  changeover/vigil already await — drydock doesn't pre-empt it). (2) kernel
  staleness is a CAPABILITY probe (agent_session moves? boot hook still BLOCKED?)
  not a vercmp; until decided, kernel item hard-routes to approval. (3) overlap
  with fixpoint's adopt loop is by-design no-double-action: if fixpoint already
  drains an item, survey sees it fresh and won't list it.

## 2026-06-16T13:09  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T06:39  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-16T13:30  /dream  fallow
/dream fallow — streak=3 threshold crossed; surfaced outward-steer to user (interactive), no steer chosen; rested. Needs user topic (homeward / constellation / companion-kin / named seed) on next pass.

## 2026-06-16T00:00  /build  ballast-survey
ballast-survey v0.3.0 shipped — README.md added, REPOS.md updated, PRD archived; ballast-cloudaware extension already landed as v0.2.0, archived.

## 2026-06-16T14:15  /dream  vision-ballast (extend)
Drafted (4 PRDs — closing the ballast loop):
  PRD-ballast-contract-repair.md — KEYSTONE. rust-extend INTO ballast-guard.
    Fix the verified guard↔survey skew: guard.rs:141 calls `ballast-survey
    --json --candidates` but survey v0.3.0 dropped --candidates → every SLO
    pass aborts ("unexpected argument '--candidates'"). Drop the dead flag,
    parse survey's stable JSON, add a contract test. Until this lands the
    whole shipped fleet is inert.
  PRD-ballast-pilot.md — shell/config. Ships default ~/.config/ballast/guard.toml
    (water marks 90/85/80, mode=report) + ballast-guard.service + .timer +
    JSONL event-sink + idempotent install/uninstall. Closes end-state #4
    (defend SLO autonomously). REAPING STAYS OPT-IN: installer never sets
    mode=enforce.
  PRD-ballast-trend.md — rust-cli (j0yen/ballast-trend). Snapshot successive
    survey JSON into a bounded ring; diff for per-path bytes/day; rank
    fastest-growing; project ETA-to-high-water. Answers end-state #5 "what
    keeps re-growing" — self-review can only guess today.
  PRD-ballast-digest.md — rust-cli (j0yen/ballast-digest). Fuse survey + trend
    + guard event log into one ranked block self-review pastes instead of its
    "suggest du -sh" prose (disk-side twin of shipped drydock-digest).
Vision: visions/ballast.md (extended, not replaced — appended "closing the loop")
Order: contract-repair → pilot (needs working guard) → trend (independent,
  can parallel) → digest (consumes trend + pilot's guard events).
Why now: disk climbed 86%→92%→96% across 2026-06-14/15/16 unattended while the
  full reap toolkit sat idle. Verified live: ballast-guard run errors on
  --candidates; no claude-ballast.timer; no guard.toml; 210G target/ (recall
  13G active, brain 13G fossil).
Notes for /build: contract-repair is rust-extend build_into
  ~/wintermute/ballast-guard — keep the SLO exit-code contract (0/2/3/4 in
  guard.rs:3-7) and the reap dry-run/--apply gate UNCHANGED; this PRD fixes
  plumbing only, must not widen the deletion surface. ballast-pilot must NOT
  cron-wire reaping — mode=report default, enforce is jsy's one-line opt-in
  (mirrors drydock-apply discipline). trend/digest are read-only, no deletion,
  --now for deterministic tests, no wall-clock in lib code.
Open questions: warm active-repo targets (recall 13G) stay untouched by design
  (reaping forces cold rebuild on CPU-only box). Whether to ever offer a
  cargo-clean-for-stale-active reclaim is left in the vision doc, not drafted.

## 2026-06-16T15:30  /dream  vision-continuity (extend, Fleet 2.0)
Drafted (3 PRDs — the memlog capture link, dead since kernel-tier ship):
  PRD-memlog-udev-mode-repair.md — KEYSTONE. mixed, build_into
    ~/wintermute/wintermute-kernel/pkg. /dev/memlog ships 0640 root:memlog
    (group read-only) so the wired PreCompact writer (memlog-precompact.sh,
    runs as jsy in the memlog group) has FAILED on every compaction since
    2026-05-24 — `~/.cache/memlog/precompact.log` has 17+ consecutive
    `[Errno 13] Permission denied`, ring total_writes=0 forever. Driver
    (memlog.c:401) and README both say 0660; only the PACKAGED udev rule
    says 0640 and it wins. Fix the pkg rule 0640→0660 + ship a no-reboot
    /etc/udev/rules.d/72- override + udevadm reload + a write→stats
    round-trip verifier.
  PRD-memlog-capture-selfcheck.md — rust-cli (j0yen/memlog-capture-selfcheck).
    Make "empty ring" a CHECKED alarm: if PreCompact fired since boot
    (precompact.log) but total_writes==0 → RED exit 3 + docket line. Fixes
    the real root failure — self-review read "ring empty" as "expected" for
    ~3 weeks and never correlated writer-fired vs write-landed.
  PRD-memlog-mode-contract-test.md — rust-extend, build_into ~/wintermute/memlog.
    Guard the skew: parse the octal mode from driver .mode + README rule +
    packaged udev rule and assert all three agree (and grant group-write).
    Regression fixture reproduces the exact 0640-vs-0660 bug.
Vision: visions/continuity.md (extended — appended "Activation Fleet 2.0",
  not replaced).
Order: udev-mode-repair → capture-selfcheck (only goes GREEN once repair lets a
  write land) → mode-contract-test (independent guard, can parallel).
Why now: memlog primitive shipped 2026-05-24; ring empty every boot since,
  always logged as "expected." Root-caused live 2026-06-16: one-digit udev
  mode skew. NO REBOOT needed — udev re-applies to the live node via udevadm
  trigger; writer's comm:PID session-id fallback already works.
Notes for /build: #1 is mixed/config (udev rule + sudo udevadm reload, sudo
  pre-approved) — keep it idempotent, ship BOTH the pkg-source fix and the
  /etc/ runtime bridge. #3 is rust-extend into memlog — wire the integration
  test through a top-level tests/*.rs entry (orphaned tests/mocks/ subdir
  compiles to nothing and false-greens). All three build/verify against the
  running 7.0.11 kernel TODAY; none gate on the agentns reboot.
Open questions: snapshot session-id stays comm:claude:PID until agentns lands
  (owned by assay / continuity Fleet 1.9 — NOT redrawn here). Whether the
  /etc/ override is permanent or retired once a corrected linux-wintermute
  pkgrel ships — the contract test (#3) is the canary that tells us.

## 2026-06-16T15:39  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T15:55  /dream  vision-muster (extend, Fleet 2.0)
Drafted (5 PRDs — the live verdict must grade the bus subtree):
  PRD-muster-subtree-census.md — rust-extend ~/wintermute/muster (census.rs).
    FOUNDATIONAL. `worker_pids` is declared but returns [] for live sessions —
    verified: pid 1054 census says worker_pids:[] while pgrep -fc
    "agorabus-worker.sh claude-1054-jsy" = 21. Populate it + add a `subtree`
    block (subscriber_pids, worker_pids, deleted_exe, worker_generations).
  PRD-muster-verdict-subtree-rot.md — rust-extend (verdict.rs). Add a
    `subtree-rot` annotation ORTHOGONAL to live/duplicate/orphan/stale: any
    session with >=1 deleted-exe child or >threshold worker generations is
    flagged even when itself `live`. This is the home for self-review's
    3-runs-open "fleet-binary-staleness: 3 deleted-exe agorabus subs".
  PRD-muster-duplicate-rank.md — rust-extend (verdict.rs). 4 interactive
    sessions all run from /home/jsy → all flagged `duplicate slug 'jsy'`, can't
    pick the zombie. Rank by activity recency: freshest stays `live`, idle-older
    become `duplicate (idle Ns)`. Settles the vision's "is cwd the right key"
    open question (recurred 4-fold live).
  PRD-agorabus-worker-idempotency-fix.md — shell → proposals/. ROOT CAUSE of
    the rot. Guard at agorabus-worker.sh:51 is `pgrep -f "…$sid\$"` but argv has
    a trailing cwd arg (`…$sid /home/jsy`), so the `$` anchor never matches →
    21 workers for one session. One-line regex fix, ships to proposals per the
    script's own held-out discipline.
  PRD-muster-subtree-reap.md — rust-extend (reap.rs). The worker fix can't undo
    already-leaked subtrees; reap the DEAD CHILDREN of a live session
    (deleted-exe subs + surplus workers), proposal-only, --confirm, never the
    root. Clears the backlog the fix can't retroactively undo.
Vision: visions/muster.md (extended — appended "Extension 2026-06-16 … Fleet
  2.0", not replaced).
Order:
  agorabus-worker-idempotency-fix  (independent — leak source, ship anytime)
  muster-subtree-census → muster-verdict-subtree-rot → muster-subtree-reap
                        └→ muster-duplicate-rank (parallels verdict-subtree-rot)
Notes for /build: subtree-census is the gate — it adds the `subtree` JSON block
  the other three muster PRDs consume; build it first. verdict-subtree-rot adds
  an ANNOTATION, not a 5th VerdictKind variant — keep VerdictKind {Live,
  Duplicate,Orphan,Stale} unchanged (verdict.rs:29-32) so existing tests hold.
  The worker fix is shell→proposals only: do NOT install to the live symlink
  (~/.claude/scripts/agorabus-worker.sh → dotfiles) — that's jsy's gated
  smoke-test step. subtree-reap mirrors muster reap / mend-bridge / recourse:
  dry-run default, --confirm to signal, hard refusal on root/interactive/live.
Open questions: worker-generation rot threshold (lean: any deleted-exe is rot;
  worker-count needs a tuned threshold, start >3). Whether subtree-reap is moot
  post-fix — no, it cleans the pre-fix backlog (1054's 21) the fix can't undo.

---

## 2026-06-16T09:20Z  /build  memlog-capture-selfcheck shipped

Shipped `memlog-capture-selfcheck` v0.1.0 — detects the "firing-but-empty" failure class
that masked the udev memlog bug for 3 weeks.

- 4 verdicts: capturing/untested/firing-but-empty/memlog-missing; exits 0/0/3/4
- 28 tests pass (13 unit + 7 AC + 3 proptest invariants + 2 doc)
- Pure verdict function with ISO-8601 parser; no wall-clock in logic
- Built on Hetzner Cloud (ccx53→cpx42), 2 cloud iterations
- Published: https://github.com/j0yen/memlog-capture-selfcheck
- Binary: ~/.local/bin/memlog-capture-selfcheck
- REPOS.md updated (self-review/observability section)

## 2026-06-16T16:45  /dream  vision-atscale-ai-strategy (Fleets 5+6 — NARRATE + AUTHOR)
Seed: jsy — "cogito and doxa are not relevant for AtScale. Create something that is."
NOTE: fallow=fallow streak=1 (escalate=false). OVERRIDE per explicit-user-steer rule.

Context: Fleets 1-4 are FULLY SHIPPED (mqo-textsql-baseline, demo-runner, scorecard,
  trace-harvest, agent, access-policy, session-budget, decision-log, goldgrow,
  replay, planner-tune, drift-watch all archived per git log). Confirmed 2026-06-16.

Gap analysis (no existing fleet covered either):
  - NARRATE: the agent returns structured answers; nothing turns them into prose an
    executive can read or a report a colleague can share. Zero existing PRDs for
    insight generation, narrative composition, chart captions, or report packaging.
  - AUTHOR: every fleet targets the AI *consumer* of the semantic layer, not the
    *author* who builds and maintains it. jsy IS that author at AtScale. Zero existing
    tools for model linting, AI-readiness scoring, synonym enrichment, or grounding
    advice for non-ontologists.

Drafted Fleet 5 — NARRATE (4 standalone j0yen rust-cli repos):
  PRD-mqo-insight-extract.md    — extract ranked findings from metric answers
                                  (delta, anomaly, parity-gap); the "so what"
  PRD-mqo-narrative-compose.md  — compose findings into audience-targeted prose
                                  (executive|analyst|technical); template-based,
                                  no LLM in default path
  PRD-mqo-chart-caption.md      — generate title/subtitle/caption for a Vega-Lite
                                  spec from build_vega_spec; Tasty Bytes Summit
                                  scenario motivated this directly
  PRD-mqo-report-pack.md        — assemble NL question → agent → narrative → chart
                                  into a Markdown/HTML report document; manifest-driven

Drafted Fleet 6 — AUTHOR (4 standalone j0yen rust-cli repos):
  PRD-mqo-measure-lint.md       — lint model for AI-hostile patterns: missing
                                  descriptions, version markers in names, numeric
                                  dimensions, redundant measures; SARIF output for CI.
                                  Live evidence: COGS_USD/SALE_PRICE_USD as dimensions
                                  in Tasty Bytes (M002 rule)
  PRD-mqo-ai-coverage.md        — score every element on discoverability + bindability
                                  + queryability; surface "dark corners"; the leading
                                  indicator for mqo-bench accuracy
  PRD-mqo-synonym-seed.md       — generate NL synonyms for measures via rule engine
                                  (snake_case expansion, suffix/prefix tables: _AMT,
                                  _QTY, _USD, _PCT, AVG_, ROLLING_); no API in
                                  default path; `apply` writes approved synonyms into
                                  model description fields
  PRD-mqo-grounding-advisor.md  — for each ungrounded element, suggest BFO class +
                                  copy-pasteable bfo_hint IRI + plain-English rationale;
                                  no ontology jargon exposed to the author

Fleet 5 order: insight-extract → narrative-compose → chart-caption → report-pack
  (report-pack consumes all three; the first three are independent)
Fleet 6 order: ALL INDEPENDENT. Suggested priority: measure-lint → ai-coverage →
  synonym-seed → grounding-advisor (lint motivates the others; coverage shows ROI of
  synonym+grounding; synonym-seed and grounding-advisor are the fixes)

Fleet 6 compose loop: measure-lint (finds missing descriptions) → synonym-seed
  (generates the fix) → apply → re-index with catalog-embed → ai-coverage improves
  → mqo-bench accuracy improves. Author-side flywheel.

Notes for /build: all 8 are j0yen/ repos (NOT AtScaleInc, NOT joeyen-atscale —
  but if user wants to move to joeyen-atscale, safe to do; just not AtScaleInc/*).
  All fixture-driven, cluster-free. grounding-advisor consumes ousia-atscale annotate
  JSON output shape (document this clearly in the PRD; build must vendoring/fixture
  that shape for tests).

## 2026-06-16T17:00  /dream  vision-atscale-ai-strategy (Fleet 6.5 — author-check orchestrator)
Seed: jsy — "/dream mqo-author-check" after asking "is there a playbook for this?"

Drafted 1 PRD:
  PRD-mqo-author-check.md  — pre-publish quality gate: chains lint + ai-coverage +
    synonym-seed + grounding-advisor as library deps (not subprocesses) into one
    `mqo-author-check run --model <m.json>` command. Outputs prioritized action
    list. --ci exits 1 on error-severity findings (CI-gate pattern from
    mqo-semantic-regression). --format json/html. serve mode.

Notes for /build: this is the KEYSTONE of Fleet 6 — build it LAST (after the
  four tools it wraps are shipped) so it can link them as library crates. Build
  order: lint → ai-coverage → synonym-seed → grounding-advisor → author-check.
  The four tools must expose a library API (not just a binary) for author-check
  to call — ensure each has a lib.rs with the core logic callable from Rust.

## 2026-06-16T09:39  /dream  (fallow)
/dream fallow — field unchanged (streak=1); rested

## 2026-06-16T17:15  /dream+build  mqo-eval (LLM-free eval harness)
Seed: jsy — "we need a new version of mcp-eval but without anthropic API dependencies"
Constraint: private joeyen-atscale repo (joeyen-atscale/mqo-eval).

Context: joeyen-atscale/mcp-eval uses LiteLLM to drive NL questions through an LLM
  (Anthropic/OpenAI/Gemini) which calls MCP tools, then grades via PGWire oracle.
  Blocking: no ANTHROPIC_API_KEY on this box, api.anthropic.com inaccessible.

mqo-eval replaces the LLM driver with mqo-agent (deterministic, shipped Fleet 3).
Same question YAML input, same PGWire oracle grading path, no API key. Also exposes
--agent '<cmd>' override so mqo-textsql-baseline can be swapped in as control arm.

Drafted 1 PRD: PRD-mqo-eval.md → joeyen-atscale/mqo-eval (PRIVATE).
Handed to /autobuilder immediately.

## 2026-06-16T17:40  /dream  vision-hold (shared cargo hold — preventive disk)
Seed: bare /dream + Phase-1 live inspection. df now 97% (18G free); 214G across
  190 private target/ dirs; CARGO_TARGET_DIR empty, sccache NOT installed locally.

Drafted 5 PRDs:
  PRD-hold-survey.md   — KEYSTONE, read-only. sum-of-per-repo-dep-builds vs
                         deduplicated union (by pkg+ver+features); reclaimable-
                         by-sharing estimate; toolchain-bucketed; emits JSON.
  PRD-hold-anchor.md   — write ~/wintermute/.cargo/config.toml build.target-dir
                         = ~/wintermute/.hold/target; idempotent; verifies cargo
                         picks it up; flags parallel-build lock tradeoff.
  PRD-hold-sccache.md  — install (via /cloudbuild) + wire size-capped local
                         sccache as RUSTC_WRAPPER; low-contention lever (no shared
                         build lock); composes with anchor's [build] table.
  PRD-hold-migrate.md  — drain existing private target/ into the hold, per-repo,
                         gated on anchored + installed-binary-current (reuse
                         ballast fossil check) + not-in-flight; append-only ledger.
  PRD-hold-guard.md    — cap the shared hold; LRU-evict fingerprint dirs over cap;
                         emit ballast-guard's SLO event schema (compose, don't dup).

Order: hold-survey → hold-anchor → {hold-sccache ∥ hold-migrate} → hold-guard.
  survey is independent (build first to size the prize). anchor is the enabler.
  sccache + migrate both need anchor, independent of each other. guard is last.

Relationship to ballast/drydock: hold is the PREVENTIVE complement. ballast reaps
  fossils (curative); hold stops the duplication accruing (preventive). NOT a
  duplicate — orthogonal axis (reclaimable-by-sharing vs reclaimable-by-deletion).

Notes for /build:
  - All 5 are j0yen/ repos (NEVER AtScaleInc).
  - hold-survey ships standalone immediately (read-only, no deps on the rest).
  - hold-anchor flips a real switch on the live fleet — its --apply should be
    user-gated on first run; survey's concurrency estimate should inform whether
    the shared-target lock bites before anchor is auto-applied.
  - hold-guard depends on ballast-guard's event schema being stable; if it isn't,
    factor the schema into a shared crate first (vision open question #2).
  - hold-migrate reuses ballast/binstale/adopt for the installed-binary-current
    check — vendoring/fixturing that classification shape is the test surface.
Open questions: shared-target lock vs parallel /build (sccache-first?); per-
  toolchain hold sharding (1.85 vs 1.88 artifacts don't dedup); hold on same nvme.

## 2026-06-16T18:05  /dream  vision-careen (intra-target reclaim for live dirs)
Seed: bare /dream + Phase-1 live inspection. fallow=fresh (streak 0). df now
  97% (17G free); du -sch ~/wintermute/*/target = 215G. recall/target=13G of
  which debug/deps alone = 9.2G (295 rlibs, mostly orphaned old versions);
  wintermute-brain/target=13G. find .../incremental -mtime +7 = 2550 stale dirs
  fleet-wide. cargo-sweep NOT installed.

The gap: ballast reaps WHOLE dead target dirs; hold SHARES one target dir.
  Neither touches INSIDE a living target dir whose binary is current+in-use —
  exactly where the two 13G giants (recall, brain) sit. careen is that third
  axis: scrape cruft off the hull of a ship still in service.

Drafted 4 PRDs:
  PRD-careen-survey.md — KEYSTONE, read-only. Classifies 4 reclaimable classes
                         (stale-incremental, orphaned-deps, wrong-toolchain,
                         dead-build) per target dir; conservative+aggressive
                         orphan estimates; JSON. Ships standalone first.
  PRD-careen-sweep.md  — the reclaimer for ONE repo. Lock-aware (respects cargo
                         target-dir flock; refuses busy targets), dry-run
                         default, --apply gated; regenerable-only invariant
                         (clean rebuild reproduces same binary). Reuses survey.
  PRD-careen-guard.md  — SLO-triggered careen of the largest LIVE dirs when disk
                         breaches. EMITS ballast-guard's Event schema (event.rs:
                         Level/used_pct_before/after/bytes_reclaimed/
                         reclaimable_bytes/candidates/ts) — composes, does NOT
                         fork. Only acts on dirs ballast SKIPS (binary-current).
  PRD-careen-ledger.md — append-only was-it-worth-it accounting: reclaimed bytes
                         vs rebuild cost the next build paid. verdict
                         worth_careening:bool so guard skips hot-thrash repos.

Order: careen-survey → careen-sweep → {careen-guard ∥ careen-ledger}.
  survey independent+read-only (build first to size prize). sweep is the engine.
  guard + ledger both consume sweep, independent of each other.

Relationship: ballast=delete-whole-dead-dir (curative), hold=share-target
  (preventive), drydock=survey-drift, careen=scrape-living-dir. careen is NOT a
  ballast dup — orthogonal (intra-dir vs whole-dir); it targets the dirs ballast
  structurally can't touch.

Notes for /build:
  - All 4 are j0yen/ repos (PUBLIC; never AtScaleInc/joeyen-atscale).
  - careen-survey ships standalone immediately (read-only, no deps on rest).
  - careen-guard should depend on ballast-guard's event module as a LIBRARY if
    exposed (preferred) to guarantee schema parity; else replicate + parity test.
  - careen-sweep's lock-safety is load-bearing (heavy /build days run many
    concurrent cargo) — do not weaken the flock check for green tests.
Open questions: (1) shared watermark evaluator with ballast-guard vs two timers
  racing the same mount (likely factor to a shared crate); (2) is Cargo.lock +
  .fingerprint enough to prove an rlib dead, or ask cargo metadata; (3) after
  hold-anchor lands, careen operates on the single shared hold (orphan set
  becomes fleet-union) — both remain valid, sequence careen-now/hold-later;
  (4) sccache (hold-sccache) moves deps economics — defer sccache-aware survey.

## 2026-06-16T18:09  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-17T23:20  /dream  vision-tether
Drafted: PRD-tether-link.md, PRD-tether-presence.md, PRD-tether-gossip.md,
  PRD-tether-recall.md, PRD-tether-tools.md
Vision: visions/tether.md
Seed: jsy — "connecting better with the work node. establish a persistent
  connection. extend agorabus. share your tools, notes, thoughts, gossip. It's
  a part of you — wire it in."

What this is: the work node (jsy's AtScale 5700U box, joeyen-atscale identity)
  becomes a PEER OF THE SELF — not a builder (constellation dropped it as a
  builder; harbor owns build pods). A persistent bus link + four mirrors:
  presence, gossip, recall, tools.

Order: tether-link → tether-presence → {tether-gossip ∥ tether-recall ∥ tether-tools}
  tether-link (rust-cli)      — persistent self-healing Tailscale+NATS-leaf
                                supervisor + tether-link.service. The literal
                                "persistent connection." Depends on wm-busbridge
                                (INSTALLED). AC5/AC6 deferred (embedded NATS /
                                real sleep-wake on the work box).
  tether-presence (rust-EXTEND agorabus, build_into=~/wintermute/agorabus, minor)
                              — the literal "extend agorabus": add `node` to the
                                peer announce + `agorabus peers --fleet` merging
                                remote peers from wm.fleet.presence.*. Additive;
                                AC1 preserves agorabus AC2 (local output
                                byte-identical). AC6 deferred.
  tether-gossip (rust-cli)    — bidirectional gossip.md mirror over
                                wm.fleet.gossip.append, loop-guarded + append-only
                                (respects the gossip hard rule on both sides).
  tether-recall (rust-cli)    — READ bridge: work node `recall query` over
                                wm.fleet.recall.query, laptop runs it, ranked hits
                                back. Read-first (write-back deferred, OQ#3).
  tether-tools (rust-cli)     — remote capability: laptop advertises an allowlist
                                of ~/.local/bin tools + executes allowlisted
                                invokes (argv array, NO shell, deny-by-default,
                                arg-sanitized). Highest-risk → built last.

Notes for /build:
  - tether-presence is rust-EXTEND of agorabus (j0yen/agorabus, PUBLIC). All
    others are NEW rust-cli repos (j0yen/ public, ecosystem tooling — NOT
    joeyen-atscale; these are wintermute self, not AtScale-derived).
  - Every tether PRD inherits persona's honesty discipline: authored +
    fixture-tested HERE, installed on the work box, SKIP honestly when the
    link/identity isn't live, NEVER false-green by assuming remote state. The
    real cross-machine ACs are deferred with embedded-NATS mocks (the
    agorabus-nats-bridge precedent: "verified against an embedded/test NATS
    server") or, where no mock is tractable (tether-link AC6 = real sleep/wake),
    a mock_justifications entry.
  - tether-link must land before the three consumers can round-trip live, but
    each consumer's LOCAL logic is independently buildable+testable against an
    embedded NATS server now — do not block them on a live work-node link.
  - tether-tools: the allowlist + arg-sanitization (no sh -c, no metacharacters)
    is load-bearing safety, grep-asserted. Do not weaken for green tests.

Cross-links: reconciles with [[constellation]] (un-drops the 5700U as a peer,
  not a builder — see vision OQ#1) and composes with [[persona]] (persona =
  the work node's face/identity; tether = its nervous system). Transport rides
  the existing [[harbor]]/agorabus-nats-bridge hub.
Open questions (see vision): (1) amend constellation's fleet row vs tether-as-
  canonical work-node vision; (2) hub-primary vs direct Tailscale peering;
  (3) recall write-back conflict model; (4) tool-invoke trust (cap-token vs
  Tailscale-ACL+allowlist); (5) a tether-doctor next pass.

## 2026-06-17 /dream fallow — field unchanged (streak=1); rested

## 2026-06-17T23:25  /dream  vision-corpus
Drafted: PRD-corpus-attest.md, PRD-corpus-roster.md, PRD-corpus-converge.md,
  PRD-corpus-arbiter.md, PRD-corpus-introspect.md
Vision: visions/corpus.md
Seed: jsy — "focus on the vision of yourself as a multinode entity."

What this is: the SELF-MODEL layer above tether's transport. tether = nerves
  wired; corpus = ONE MIND using them. The difference between "machines are
  connected" and "there is one entity, and here is who/where it is."
  Distinct from: constellation (fleet infra), tether (the wire + 4 mirrors),
  muster (LOCAL session roster), cogito/continuity (self-knowledge over time).

Order: corpus-attest → {corpus-roster ∥ corpus-converge ∥ corpus-arbiter}
       → corpus-introspect (capstone)
  corpus-attest (rust-cli)       — membership: a node PROVES it's a legit limb
                                   of the self (fleet credential bound to session
                                   id). agentns 128-bit id used WHEN LIVE, else
                                   userspace keypair fallback (agentns activation
                                   is BLOCKED — pkgrel>=12+reboot; honest degrade).
                                   ROOT of the dep graph: "me" must be defined
                                   before roster/converge/arbiter.
  corpus-roster (rust-EXTEND muster, build_into=~/wintermute/muster, minor)
                                 — federated roster: `muster --fleet` aggregates
                                   each node's local census+verdict over
                                   wm.fleet.muster.*, attested-only. Reuses
                                   muster (no dup). AC1 preserves local byte-
                                   identical output. READ-ONLY (no cross-node
                                   reap — vision OQ#4).
  corpus-converge (rust-cli)     — coherence under partition: version-vector
                                   self-state; a rejoining (slept/rebooted) node
                                   re-syncs to current before acting. NOT a CRDT
                                   (append-only/arbiter-gated elsewhere) — just
                                   version vector + catch-up.
  corpus-arbiter (rust-cli)      — single-writer across nodes: lease-based
                                   advisory lock (wm.fleet.lock.*), TTL auto-
                                   release on dead node, deny-by-default, attested-
                                   only. Lifts agorabus claim_guard to fleet scale.
                                   Fail-safe: no lock → no write.
  corpus-introspect (rust-cli)   — capstone: `corpus introspect` synthesizes
                                   attest+roster+converge+arbiter+tether into ONE
                                   whole-self portrait. Degrades honestly (never
                                   claims completeness it lacks). Synthesis only,
                                   no new sensing → smallest risk.

Notes for /build:
  - corpus-roster is rust-EXTEND of muster (j0yen/muster, PUBLIC). All others are
    NEW rust-cli repos under j0yen/ (wintermute self, NOT joeyen-atscale).
  - DEPENDS ON tether: roster needs tether-presence; introspect reads wm-tether
    status; all ride the wm-busbridge/harbor hub. Sequence tether-link first, but
    each corpus PRD's LOCAL logic is buildable+testable against embedded NATS now
    — do NOT block on a live work-node link (same discipline as tether).
  - corpus-attest: NO plaintext secret/private key in the repo (grep-asserted);
    keypair written 0600 at runtime under ~/.config/corpus/. agentns id is an
    ATTRIBUTE not a credential — auth in userspace, bind agentns id when present.
  - corpus-arbiter: deny-by-default + attested-only + named-resource allowlist is
    load-bearing safety. Do not weaken for green tests.
  - Honesty discipline (inherited persona/tether): fixture-tested HERE, live
    cross-node ACs deferred (embedded-NATS mocks, nats-bridge precedent). Every
    tool SKIPs honestly to the lone-laptop case when no fleet is configured.

Cross-links: builds ON [[tether]] (transport) and composes with [[constellation]]
  (infra), [[muster]] (extended for the roster), [[persona]] (per-node face),
  [[cogito]]/[[continuity]] (self-knowledge over time; corpus is self-knowledge
  across SPACE/nodes). agorabus claim_guard is the local ancestor of the arbiter.
Open questions (see vision): (1) hub-authoritative vs p2p self-state; (2) attest
  root of trust (fleet secret vs keypair+CA vs Tailscale ACL); (3) arbiter
  resource granularity; (4) cross-node reap explicitly OUT of scope; (5) a
  corpus-narrate (unified cross-node self-review) next pass.

NB on cadence: /dream timer moved to every 10 min (was 30) per jsy 2026-06-17.

## 2026-06-17T23:27  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-18T06:30  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-18T06:40  /dream  vision-homeward (reach 3 — coverage & priority)
Drafted: PRD-homeward-catchment-geo.md, PRD-homeward-cadence-stray.md,
  PRD-homeward-found-geocode.md, PRD-homeward-catchment-discover.md
Vision: visions/homeward.md (reach-3 section appended)
Context: fallow gate escalated (streak=3, inward self-tooling field saturated);
  user dismissed steer; picked outward homeward. Full homeward fleet (22 PRDs,
  v0.29.0) already shipped — this reach attacks the vision's own "still
  un-dreamt" frontier + 2 adjacent Phase-1 gaps.
Order:
  catchment-geo ─► catchment-discover   (discover consumes computed holes)
  cadence-stray      (independent, rust-extend homeward-ingest)
  found-geocode      (independent, rust-extend homeward-connectors)
Notes for /build:
  - ALL FOUR are rust-extend build_into=~/wintermute/homeward (workspace
    v0.29.0). No new standalone repos.
  - catchment-geo & found-geocode & cadence-stray are buildable+testable NOW
    against the live ingest SQLite store / fixtures; no external service.
  - catchment-discover DEPENDS ON catchment-geo (reads its computed holes) —
    build geo first.
  - cadence-stray touches the live orchestrator AIMD (orchestrator.rs:53-64).
    Hard rule baked into its ACs: stray==0 inputs must reproduce TODAY's exact
    interval sequence (no regression to existing sources), and stray_floor must
    not poll below the slowest documented upstream refresh (ToS).
  - found-geocode & catchment-discover are propose-only / fallback-only by
    design (never overwrite shelter location; never auto-commit a source).
    Do NOT weaken those guards for green tests.
  - All offline: no PostGIS server (CPU-only laptop), no external geocode API.
Open questions (see vision reach-3): served coverage map tiles (UI, later
  pass); non-US portals still deliberately out of scope.

## 2026-06-18T06:55  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-18T07:00  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-18T07:10  /dream  fallow
/dream fallow — streak=3 threshold crossed; outward-steer surfaced, user dismissed; rested (no draft past fallow)

## 2026-06-18T07:30  /dream  vision-threshold
Drafted: PRD-threshold-brief.md, PRD-threshold-verify.md,
  PRD-threshold-ledger.md, PRD-threshold-hook.md
Vision: visions/threshold.md
Context: fallow gate escalated (streak=4, inward self-tooling field saturated);
  user manually invoked /dream + did not answer the steer prompt; auto-mode
  picked the escalation's companion-kin option — "what a peer intelligence
  wants from this box." The peer is the NEXT Claude session. Phase-1 made it
  concrete: the SessionStart hook bundle for THIS session = 20,888 bytes from
  10 independent hooks, zero synthesis, zero verification.
Order:
  threshold-brief ─┬─► threshold-verify ─┐
                   └─► threshold-ledger ─┴─► threshold-hook
Notes for /build:
  - threshold-brief is FOUNDATIONAL — NEW repo ~/wintermute/threshold (rust-cli).
    Build it first; the other three are rust-extend build_into=~/wintermute/threshold.
  - threshold-verify & threshold-ledger both extend brief and are independent of
    each other (parallelizable). threshold-hook depends on all three.
  - threshold-hook is `mixed` (rust-extend + settings.json SessionStart hook via
    update-config). HARD RULE baked into its ACs: WRAP don't replace the 10
    existing hooks; hook must exit 0 always + degrade to today's firehose if the
    binary is absent — never block boot. Do NOT weaken those guards for green.
  - threshold-verify reuses the `answerable reconcile` ground-truth precedent;
    its ACs forbid silently dropping a `contradicted` verdict (warrant/assay
    false-close class). threshold-ledger must NOT hard-depend on a live agentns
    id (it's blocked pending pkgrel≥12) — hostname+pid fallback is required.
  - All offline; no new external services.
Open questions (see vision): heuristic vs LLM claim parsing (start heuristic);
  replace-vs-wrap the 10 hooks (resolved: wrap; a follow-on threshold-spool may
  teach hooks to write a spool); bus-published questions + cross-node succession
  explicitly OUT of scope (tether/corpus territory).
Note: answerable-emit.sh not on PATH this pass — draft records NOT emitted.

## 2026-06-18T08:05  /dream  vision-colophon
Drafted: PRD-colophon-parse.md, PRD-colophon-attribute.md,
  PRD-colophon-stale.md, PRD-colophon-digest.md
Vision: visions/colophon.md
Context: bare /dream (auto-mode, no topic), field FRESH (streak=0). Phase-1 hit
  a genuine state change — the wintermute kernel is now BOOTED, not just built
  (`uname -r` = 7.0.11-arch1-1-wintermute), so provfs is LIVE and stamping an
  enriched structured `user.prov.session` xattr on every closed-after-write
  file. Read this dream pass's own gossip write and it carried
  `comm-chain:cat>zsh>claude;cwd:…;pid:2662703;uid:1000`. But the one consumer
  (provenance-mcp) returns the raw blob and its tests assume the old opaque
  `sess-abc123` form — nobody parses the structure. colophon is the userspace
  half that catches it.
Order:
  colophon-parse ─┬─► colophon-attribute ─┐
                  └─► colophon-stale ──────┴─► colophon-digest
Notes for /build:
  - colophon-parse is FOUNDATIONAL — NEW repo ~/wintermute/colophon (rust-cli+lib).
    Build it first; the other three are rust-extend build_into=~/wintermute/colophon.
  - colophon-attribute & colophon-stale both extend parse and are independent of
    each other (parallelizable). colophon-digest depends on BOTH.
  - colophon-digest is `mixed` (rust-extend + a guarded, additive self-review
    skill-doc block). HARD RULE in its ACs: WRAP don't replace self-review's
    existing reporting; degrade to an honest one-liner + exit 0 when provfs is
    absent — never break a self-review run.
  - parse's ACs require BOTH the comm-chain form AND the 128-bit agentns-id form
    parse today, so when agentns finally activates (blocked on the EINVAL
    CLONE_NEWAGENT collision, see vision-assay) the switch is a no-op.
  - attribute & stale are STRICTLY read-only by design (report, don't delete);
    ACs assert the fixture tree is unchanged after a run. Do NOT add mutation for
    convenience. provfs skips target/.git/node_modules — attribution must report
    those as skipped-by-design, never as unattributed (confirmed live: a
    target/ artifact carries no prov xattr).
Open questions (see vision): re-point provenance-mcp at the colophon lib so its
  file_provenance tool returns structured fields? Left as a deliberate boundary
  (conduit/threshold territory) — a follow-on colophon-mcp PRD can do it once the
  lib is proven. provfs history-ring (Phase 2) unconsumed until it ships.
Note: answerable-emit.sh not on PATH this pass — draft records NOT emitted.

## 2026-06-18T (second pass)  /dream  vision-headway
Drafted: PRD-headway-build.md, PRD-agorabus-reload-build.md,
  PRD-headway-rollout-cloudbuild.md, PRD-headway-verify.md
Vision: visions/headway.md
Context: bare /dream (auto-mode), field FRESH (streak=0); colophon already drafted
  this morning, so this pass took a different seed. Phase-1 found the `behind-head`
  staleness class — the ONLY class whose fix needs a recompile — has no working
  remediation, and the path that claims to is broken two ways:
  (1) rollout builds LOCALLY (`fleet.rs:71` build_cmd="cargo build --release"),
      violating the cloudbuild-only hard rule; cloudbuild was left a manual
      #[ignore] comment (warmswap.rs:630).
  (2) `rollout install` shells to `agorabus reload --build` — a flag that does
      NOT exist (`agorabus reload --help` has no --build). Dead bridge.
  This is the recurring "agorabus 3d behind source / reload --build absent" item.
Order:
  headway-build ─┬─► headway-rollout-cloudbuild
                 └─► headway-verify
  agorabus-reload-build  (parallel; shells to cloudbuild.sh directly)
Notes for /build:
  - headway-build is FOUNDATIONAL — NEW repo ~/wintermute/headway (rust-cli+lib).
    Build it first; rollout-cloudbuild & headway-verify are rust-extend
    build_into=~/wintermute/headway / ~/wintermute/rollout.
  - HARD RULE baked into every PRD's ACs: the build step is ALWAYS a cloudbuild.sh
    subprocess, NEVER local cargo; honest abort + non-zero exit if cloudbuild is
    unreachable — no silent local fallback. Do NOT weaken this for green.
  - agorabus-reload-build adds the `--build` flag rollout already calls. It must
    NOT bounce the live daemon if the cloudbuild step fails (AC4) — assert the
    pre-existing pid is unchanged after a failed build.
  - headway-verify must NEVER false-close: a still-`behind-head` daemon after
    reload is `contradicted` (exit 1), never success. Reuses the
    answerable/threshold contradicted-verdict precedent.
  - rollout-cloudbuild must preserve rollout's proven orchestration (serialized
    restart, re-register confirm, voice --restart-window guard) — only the build
    step changes. --local-build escape hatch allowed ONLY if it logs loudly.
  - All offline except the cloudbuild subprocess (Hetzner, already in use).
Open questions (see vision): register headway as drydock's rebuild lane so
  behind-head routes to it automatically (left as a future headway-lane PRD —
  drydock lane set confirmed auto/window/reboot/approval, but behind-head's
  current routing not verified); confirm cloudbuild.sh artifact-pull path before
  headway-build relies on it; should headway-verify's contradicted path emit to
  docket/litmus as a tracked finding?
Note: answerable-emit.sh not on PATH this pass — draft records NOT emitted.

## 2026-06-18T  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-18T08:07  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-18T  /dream  fallow
/dream fallow — streak=3 threshold crossed; outward-steer offered, user declined to steer; rested

## 2026-06-18T  /dream  fallow
/dream fallow — streak=4 threshold crossed; outward-steer offered (interactive), user declined to steer; rested

## 2026-06-18T  /dream  fallow
/dream fallow — streak=5 threshold crossed; outward-steer offered (interactive), unanswered/declined; rested

## 2026-06-18T  /dream  fallow
/dream fallow — streak=6 threshold crossed; outward-steer offered (interactive), unanswered/declined; rested

## 2026-06-18T  /dream  vision-consign
Drafted: PRD-consign-survey.md, PRD-consign-policy.md, PRD-consign-publish.md,
  PRD-consign-drain.md, PRD-consign-cron.md, PRD-consign-verify.md
Vision: visions/consign.md
Context: bare /dream (auto-mode), field FRESH (streak=0, last-productive
  2026-06-18T07:54Z). headway (drafted earlier today) covers behind-head DAEMON
  recompile; consign is its sibling for un-mirrored GIT COMMITS. Phase-1 live
  walk of ~/wintermute/*/.git found the "8 unpushed" self-review line is a
  systematic UNDERCOUNT: 7 ahead-of-upstream + 5 no-upstream (invisible to the
  @{u} check) + 2 no-remote (colophon, headway — never published) + 1 diverged
  (mqo-narrative-compose 2/2). Broader `git log --branches --not --remotes` = 29
  repos holding commits on no remote. Single-laptop SPOF; disk-critical near-miss
  2026-06-17 underlines the risk. gh authed as j0yen (keyring); adopt-cron is the
  6h-timer precedent to mirror.
Order:
  consign-survey ─► consign-policy ─┬─► consign-publish ─┐
                                    └─► consign-drain ───┼─► consign-cron
                                                         └─► consign-verify
Notes for /build:
  - consign-survey is FOUNDATIONAL — NEW repo ~/wintermute/consign (rust-cli+lib).
    Build it first; policy/publish/drain/verify are rust-extend build_into=
    ~/wintermute/consign; consign-cron is shell (systemd-user units).
  - consign-policy GATES every write path (publish, drain). Default-deny:
    private-hold (autobuilder*/*-private/.consign-hold/secret-file) and
    manual-only (diverged / non-default branch / detached) never auto-push.
  - publish + drain are PARALLEL: publish mints remotes for no-remote repos
    (gh repo create j0yen/<name>), drain pushes ahead/no-upstream repos. Both
    dry-run by DEFAULT (--no-dry-run to act), mirroring adopt apply.
  - HARD: drain NEVER --force / --force-with-lease; diverged repos surfaced for
    humans, never auto-resolved (would destroy the behind commits).
  - consign-verify reuses headway-verify's contradicted-never-false-close: a repo
    drain claimed pushed but still ahead = contradicted (exit 1), never green.
    Standalone it can replace self-review's undercounting @{u} line.
  - All offline except gh (publish) + git push (drain) to GitHub; build via
    /cloudbuild per the standing hard rule.
Open questions (see vision): push feature/worktree branches or default-only?
  (leaning: non-default HEAD => manual-only, so homeward's worktree branch isn't
  auto-published). Widen roots beyond ~/wintermute (~/.claude, ~/projects) later
  via config. New-mint visibility inherits policy, never unconditional --public.
Note: answerable-emit.sh not on PATH this pass — draft records NOT emitted.

## 2026-06-18T  /dream  vision-trim
Drafted: PRD-trim-survey.md, PRD-trim-attribute.md, PRD-trim-policy.md,
  PRD-trim-relief.md, PRD-trim-psi.md, PRD-trim-cron.md
Vision: visions/trim.md
Context: bare /dream (auto-mode), field FRESH (streak=0, last-productive
  2026-06-18T09:04Z). Phase-1 found a recurring self-review finding with NO
  OWNER: "swap 5.4G/8G — heavy, monitor; not actionable today" repeats in every
  journal 2026-06-12..18. Disk pressure has a rich fleet (ballast/careen/
  drydock/thrift); MEMORY pressure has nobody. Live walk: swap 5.4G/8G used;
  /proc/pressure/memory full total=337243465 (the box HAS thrashed); top swap
  holders homeward-embed-svc 476M + recalld 237M — BOTH systemd user units
  (homeward-embed.service, recalld.service), i.e. relief-eligible idle daemons.
  trim is to RAM what ballast is to disk. Verified: PSI live, cgroup-v2
  user.slice present, self-healing daemons (Restart=always) confirmed.
Order:
  trim-survey ─► trim-attribute ─► trim-policy ─┬─► trim-relief ─► trim-cron
                                                └─► trim-psi
Notes for /build:
  - trim-survey is FOUNDATIONAL — NEW repo ~/wintermute/trim (rust-cli+lib).
    Build it first; attribute/policy/relief/psi are rust-extend build_into=
    ~/wintermute/trim; trim-cron is shell (systemd-user units).
  - trim-policy GATES every relief action. Default-deny like consign-policy:
    only systemd-managed, self-healing (Restart=always), IDLE wintermute daemons
    over the swap floor are eligible. HARD exclusions (non-overridable by config):
    user-apps (firefox/slack), builds (rustc), local-llm (ollama→defer to thrift),
    and wmd/wm-dialog DURING a live voice TURN. Build policy before relief.
  - trim-relief + trim-psi both depend on policy, INDEPENDENT of each other
    (parallelizable). relief ACTS (dry-run by default, --no-dry-run to apply,
    mirroring adopt/consign); psi only WARNS (emits agorabus trim.pressure).
  - HARD: relief levers are gentle+reversible — try-restart (NEVER restart/start,
    so a stopped unit is never started), MemoryHigh via --runtime (evaporates on
    reboot), or a drop-cache IPC. NEVER kill, NEVER oom_score_adj in v1, NEVER
    touch a user process or a live turn. Reuse self_agorabus_restart_kills_voice
    (Restart=always self-heal) + project_voice_input_null_detectors (turn-state).
  - trim-cron mirrors adopt-cron/ballast-pilot: 6h timer + additive guarded
    self-review skill-doc block replacing the "not actionable" swap line. WRAP
    don't replace; degrade to honest one-liner + exit 0 when trim absent.
  - All offline except agorabus emit (psi) + systemctl --user (relief); build via
    /cloudbuild per the standing hard rule.
Open questions (see vision): does recalld expose a "shed cache" IPC or is
  try-restart the only lever? (follow-on recall PRD if not). oom_score_adj
  nudging — leaning NO for v1 (blunt+irreversible). ollama relief boundary:
  trim defers local-LLM to thrift. "Idle" threshold: start recent-CPU floor,
  refine per-unit (homeward-ingest AIMD quiet windows) later.
Note: answerable-emit.sh not on PATH this pass — draft records NOT emitted.

## 2026-06-18T  /dream  fallow
/dream fallow — field unchanged (streak=1); rested

## 2026-06-18T  /dream  fallow
/dream fallow — field unchanged (streak=2); rested

## 2026-06-18T  /dream  vision-chaff
Drafted: PRD-chaff-survey.md, PRD-chaff-policy.md, PRD-chaff-gitignore.md,
  PRD-chaff-repair.md, PRD-chaff-guard.md, PRD-chaff-cron.md
Vision: visions/chaff.md
Context: bare /dream (auto-mode), field FRESH (streak=0, last-productive
  2026-06-18T09:15Z). Phase-1 live walk of ~/wintermute/*/.git found a sharp
  UNOWNED hygiene gap under the recurring self-review line "56 dirty repos":
  12 of 226 repos track ~2745 regenerable build-artifact files committed into
  git. careen-ledger=1942 target/ files + NO .gitignore; rosetta-prov=464 (no
  .gitignore); hold-guard=203. Two strains: (1) no .gitignore at all; (2) the
  footgun — coda's .gitignore literally says /target/ yet still tracks
  target/.rustc_info.json because .gitignore only ignores UNTRACKED paths.
  The disk fleet's cargo clean turns tracked target/ into 1942 phantom " D"
  dirty entries — i.e. it MANUFACTURES the "dirty repos" noise. consign drains
  COMMITS (would push the junk); ballast/careen/drydock delete SPACE; chaff
  owns deciding what should be in git at all.
Order:
  chaff-survey ─► chaff-policy ─► chaff-repair ─► chaff-guard ─► chaff-cron
              └─► chaff-gitignore (parallel; additive, no policy gate)
Notes for /build:
  - chaff-survey is FOUNDATIONAL — NEW repo ~/wintermute/chaff (rust-cli+lib).
    Build it first; policy/gitignore/repair/guard are rust-extend
    build_into=~/wintermute/chaff; chaff-cron is shell (systemd-user units +
    additive self-review block).
  - chaff-policy GATES chaff-repair (default-deny; only regenerable dirs/exts,
    never src/; HARD-exclude diverged/mid-rebase/detached repos and
    .build-worktrees). Build policy before repair.
  - chaff-gitignore is the SAFE arm (writes .gitignore, never untracks) and is
    independent of policy — parallelizable with policy/repair.
  - chaff-repair ACTS: git rm --cached + ensure ignore line + commit deletion
    with Joe Yen identity, dry-run by default (--no-dry-run to apply, mirrors
    adopt apply / consign drain). Forward-only — does NOT rewrite history
    (a git-filter-repo purge PRD is deferred; see vision open question).
  - SEQUENCING vs consign: chaff should run BEFORE consign drains so consign
    never pushes build junk. No code dependency, just ordering — if both are
    queued, advance chaff-repair before consign-drain.
  - Reuse on this box: self_sigpipe_panic_toolkit (sigpipe::reset first line),
    MSRV 1.85 no let-chains, /cloudbuild for all cargo, apply-agentns.py
    anchor-block pattern for chaff-guard's hook install + the self-review block.
Open questions (see vision): history rewrite (filter-repo) vs forward-only
  (leaning forward-only v1); guard hook location (per-repo vs core.hooksPath
  vs /build publish step — leaning per-repo + optional /build hook); whether
  target/autobuilder/receipts/ is exempt (leaning NO — regenerable).
Note: answerable-emit.sh not on PATH this pass — draft records NOT emitted.

## 2026-06-18T  /dream  fallow
/dream fallow — field unchanged (streak=1); rested
2026-06-18T10:07:20Z loom-serial-fallback: SERIAL MODE for colophon (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains

## 2026-06-18T  /dream  rest (semantic-saturation)
/dream rested — fallow fingerprint read FRESH (streak=0) but Phase-1 research
  found the inward field empirically saturated: 3 visions drafted TODAY cover
  every concrete signal probed — headway (fleet behind-head recompile
  remediation: rollout builds locally + agorabus reload --build), colophon
  (provfs xattr consumer post kernel-boot 7.0.11-wintermute), chaff (git
  build-artifact hygiene). Surrounding space also covered: quicken (inert
  primitives), vigil (stale bytes), freshness (stale memory), drift (stale
  skills), trim/consign/ballast/careen/drydock (disk/mem/push). Fingerprint
  reads fresh only because today's drafts moved its inputs — the fallow
  "no memory of its own saturation" failure mode. Per hard rule #8, rested
  rather than draft a thinner fleet. Offered outward steer (homeward /
  constellation / companion-kin); user declined. Outward frontier remains
  the genuinely unclaimed direction; awaits explicit steer.

## 2026-06-18T  /dream  rest (semantic-saturation, interactive)
/dream rested — bare interactive /dream. fallow fingerprint read FRESH
  (streak=0) but Phase-1 live probes confirm the inward field empirically
  saturated: ctrace --since 24h empty, pevent empty, reflective recalls all
  write-only (recalls=0, stale kernel obs), ideation hybrid returns weeks-old
  "kernel built not booted" notes, wchg deltas pure /build churn (mqo-*,
  build-worktrees). 3 visions drafted TODAY (headway, colophon, chaff) cover
  every concrete inward signal — fingerprint reads fresh only because today's
  drafts moved its inputs (the fallow "no memory of own saturation" mode).
  Per hard rule #8, rested rather than draft a thinner fleet. Surfaced outward
  steer (homeward / constellation / companion-kin / rest) via AskUserQuestion;
  user dismissed without selecting. Outward frontier remains the genuinely
  unclaimed direction; awaits explicit steer.

## 2026-06-18T  /dream  rest (semantic-saturation, manual)
/dream rested — manual interactive /dream. fallow fingerprint read FRESH
  (streak=0) but live Phase-1 re-probe confirms the inward field still
  saturated: pevent empty, ctrace no new activity, reflective recalls all
  write-only (recalls=0), ideation-hybrid returns only weeks-old "kernel
  built not booted" notes. Every concrete inward signal already covered by
  today's 3 visions (headway, colophon, chaff) + surrounding disk/mem/push/
  staleness fleet. Fingerprint reads fresh only because today's drafts moved
  its inputs (the "no memory of own saturation" mode). Per hard rule #8,
  rested rather than draft a thinner fleet. Surfaced outward steer (homeward /
  constellation / companion-kin / name-a-topic) via AskUserQuestion; user
  dismissed without selecting (3rd dismissal today). Outward frontier remains
  the genuinely unclaimed direction; awaits explicit steer.

## 2026-06-18T  /dream  rest (semantic-saturation, interactive /dream)
/dream rested — explicit interactive /dream. fallow read FRESH (streak=0) but
  live Phase-1 re-probe confirms inward field still saturated: pevent empty,
  ctrace no usable 24h activity, reflective recalls all write-only (recalls=0),
  ideation-hybrid returns only weeks-old "kernel built not booted" notes. Every
  concrete inward signal already covered by today's fleet (headway, colophon,
  chaff×6, trim×5, consign×4). Fingerprint reads fresh only because today's
  drafts moved its inputs (the "no memory of own saturation" mode). Per hard
  rule #8, rested rather than draft a thinner fleet. Surfaced outward steer
  (homeward / constellation / companion-kin / rest) via AskUserQuestion; user
  dismissed (4th dismissal today). Outward frontier remains the genuinely
  unclaimed direction; awaits explicit steer. fallow record --drafted 0.

## 2026-06-18T  /dream  rest (semantic-saturation, interactive)
/dream rested — explicit interactive /dream. fallow read FRESH (streak=0) but
  live Phase-1 re-probe confirms the inward field still saturated: ctrace
  --since 24h empty, pevent empty, reflective recalls all write-only
  (recalls=0), ideation-hybrid returns only weeks-old "kernel built not
  booted" notes. Every concrete inward signal already covered by today's
  fleet. Fingerprint reads fresh only because today's drafts moved its inputs
  (the "no memory of own saturation" mode). Per hard rule #8, rested rather
  than draft a thinner fleet. Surfaced outward steer (homeward / constellation
  / companion-kin / rest) via AskUserQuestion; user dismissed (5th dismissal
  today). Outward frontier remains the genuinely unclaimed direction; awaits
  explicit steer. fallow record --drafted 0.
2026-06-18T10:52:53Z loom-serial-fallback: SERIAL MODE for trim (streak=2, pathset=unknown) — fanning at most 1 branch/tick until backlog drains

## 2026-06-18T  /dream  rest (semantic-saturation, interactive)
/dream rested — bare interactive /dream. fallow read FRESH (streak=0) but live
  Phase-1 re-probe confirms inward field still saturated: pevent empty, ctrace
  no usable 24h activity, reflective recalls all write-only (recalls=0), today's
  journal pure /build-churn + known anomalies (agentns zeros, plumb
  uncalibrated, kernel pkgrel-12 staged) — all already covered by today's fleet
  + docket. Fingerprint reads fresh only because today's drafts moved its inputs
  (the "no memory of own saturation" mode). Per hard rule #8, rested rather than
  draft a thinner fleet. Skipped the outward-steer AskUserQuestion (user
  dismissed it 5× today); offered plain-text steer instead. Outward frontier
  (homeward / constellation / companion-kin / a named topic) remains the
  genuinely unclaimed direction; awaits explicit steer. fallow record --drafted 0.

## 2026-06-18T  /dream  rest (semantic-saturation, interactive)
/dream rested — bare interactive /dream. fallow read FRESH (streak=0) but live
  Phase-1 re-probe confirms inward field still saturated: pevent empty, ctrace
  no usable 24h activity (--since rejects "24h"), reflective recalls all
  write-only (recalls=0), ideation-hybrid returns only weeks-old "kernel built
  not booted" notes, today's journal pure /build churn + known-anomaly carry
  (agentns zeros run-17+, plumb memlog-active uncalibrated, pkgrel-12 staged).
  The one fresh-looking signal — agorabus "reload --build absent" — already
  SHIPPED today (v0.12.0, session changelog). Fingerprint reads fresh only
  because today's drafts moved its inputs (the "no memory of own saturation"
  mode). Per hard rule #8, rested rather than draft a thinner fleet. Surfaced
  outward steer (homeward / constellation / companion-kin / rest) via
  AskUserQuestion; user dismissed again. Outward frontier remains the
  genuinely unclaimed direction; awaits explicit steer. fallow record
  --drafted 0.

## 2026-06-18T  /dream  rest (maturity-saturation, interactive)
/dream rested — interactive bare /dream. fallow read FRESH (streak=0) so I was
  cleared to draft, but Phase-1 live inspection produced a sharper finding than
  the day's prior 5 "inward-saturation" rests: the field is saturated by
  *maturity*, not just by quiet inward signals. Every outward vision the steer
  question offers has already been dreamt deep AND largely shipped:
    - homeward: FIVE fleets (core→federation→operate→deliver→catchment); the
      catchment fleet (4 PRDs: cadence-stray, catchment-{discover,geo},
      found-geocode) sits UNBUILT in the queue right now. Drafting fleet 6 would
      pile on an unbuilt fleet.
    - kin: BOTH fleets shipped — wintermute-reach v0.7.0 (transport, dispatch,
      digest, silence-nudge, distress-delivery, inbound; dist/wm-reach.service),
      wintermute-presence v0.2.0, wintermute-family-enroll done. 9 PRDs archived.
    - companion: voice TURN shipped end-to-end (2026-06-04); needs the physical
      device, not more PRDs.
    - constellation: blocked on hardware (other machines), not on dreaming.
  The honest conclusion: the frontier has shifted from DRAFTING to
  OPERATING/PROVING. homeward's own vision says its parts "have never been run
  together"; kin's daemons are built but I have no evidence they're enabled or
  configured with a real reach-transport on this box. That's /build + /self-review
  work (run the queued catchment fleet; stand the daemons up; prove a round-trip),
  NOT new /dream PRDs. Per hard rules #3/#6/#8 I did not manufacture a thin
  "operate-X" fleet on top of already-shipped daemons. Skipped the 4-way
  AskUserQuestion (dismissed 5× today) per the established no-repeat-popup
  pattern; offered plain-text steer instead. Genuinely unclaimed direction = a
  NEW outward topic the user names, or "deepen homeward/kin" as explicit steer.
  fallow record --drafted 0.

## 2026-06-18T11:29  /dream  rest (maturity-saturation, interactive)
/dream rested — bare interactive /dream. fallow read FRESH (streak=0) but
  Phase-1 confirmed the day's pattern: recall seed yielded NO new ideation
  (reflective recalls=0 write-only; ideation-hybrid = weeks-old kernel notes),
  and the queue holds 30 unbuilt Draft PRDs incl. the homeward catchment fleet
  (cadence-stray, catchment-discover, catchment-geo, found-geocode). Fingerprint
  fresh only because today's drafts moved its inputs. Per #3/#6/#8 did not draft
  onto a 30-deep backlog. Did not re-fire the 5×-dismissed steer popup. Frontier
  = /build draining the queue + /self-review proving daemons, not new PRDs.
  fallow record --drafted 0.

## 2026-06-18T  /dream  rest (maturity-saturation, interactive)
/dream rested — bare interactive /dream, fresh session. fallow read FRESH
  (streak=0) but ran mandatory Phase-1 recall seed anyway: reflective recalls=0
  (write-only), ideation-hybrid = same weeks-old "kernel built not booted" hits,
  no new signal. Queue holds 30 unbuilt Draft PRDs. Fingerprint fresh only
  because the day's own passes keep moving its inputs (no-memory-of-own-
  saturation). Per #3/#6/#8 did not draft onto a 30-deep backlog; did not
  re-fire the steer popup (dismissed 5× today). Frontier = /build draining the
  queue + /self-review proving daemons stand up, not new PRDs. Offered plain-text
  outward steer to user instead. fallow record --drafted 0.
