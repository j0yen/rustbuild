# PRD: inoculate-carrier-check — prove the strain is actually loaded

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/inoculate
Vision: visions/inoculate.md

## TL;DR

Injecting a preamble (inoculate-inject) does not prove a subagent *internalized*
it — a prompt can be ignored, truncated, or overridden downstream. Before one
agent extends autonomy to another (e.g. an orchestrator trusting a child to
publish), it should be able to **verify the child is a carrier**: a
challenge/response where only an agent that actually has the current strain in
context can answer correctly. `inoculate-carrier-check` adds that.

## Why this exists

`answerable` can record what the main loop did, but there is no way to ask "is
this *other* agent carrying the ethic, or did the preamble silently fall off?"
With `/build` fanning out to 30 parallel subagents, an unverifiable claim of
inoculation is worth little. A challenge bound to the in-force `strain_hash`
makes carriage checkable, not assumed. (Vision open question: start as a
flag/observation, not a hard autonomy gate.)

## What this builds

Extends the `inoculate` repo (from inoculate-core), reusing `Strain`:

- `inoculate challenge [--format text|json]` — emit a challenge: a nonce plus a
  question whose correct answer is a deterministic function of the current strain
  content (e.g. "list the boundary at index N" / "what is the strain_hash?" /
  "HMAC the nonce with the strain canonical bytes"). The HMAC variant is the
  strong form: only an agent with the strain bytes can compute it.
- `inoculate verify --challenge <file|-> --response <str>` — recompute the
  expected answer from the local strain + nonce and compare. Exit 0=carrier,
  1=not-a-carrier (mismatch), 2=malformed/stale challenge (nonce older than TTL).
- `inoculate self-attest` — convenience: an agent that holds the strain produces
  its own challenge response in one shot, for piping into a parent's `verify`.
- Strain version awareness: `verify` reports if the response matches an *older*
  strain than local (carrier of a stale ethic) vs no match at all — distinct exit
  detail so spread/attest can tell "behind" from "absent."

Deps: add `hmac`/`blake3` keyed mode; `rand`-free nonce (caller supplies time/seed
via arg, per the no-`Date::now` script lesson is N/A here — this is a binary, but
keep nonce caller-supplyable for testability).

## Acceptance criteria

1. `cargo test --release` passes; updated `inoculate` reinstalls to `~/.local/bin/`.
2. A response produced from the same strain + nonce makes `verify` exit 0; a
   response from a tampered strain makes it exit 1.
3. A challenge older than the TTL (`--ttl`) makes `verify` exit 2.
4. `verify` distinguishes "matches an older strain version" (reported explicitly,
   e.g. exit 1 with stderr `stale-strain v<old>`) from "no match at all."
5. HMAC mode: `verify` succeeds only when the response is the strain-keyed HMAC of
   the nonce; a random string of the same length fails.
6. No `unwrap`/`expect`/`panic` in non-test code; SIGPIPE-safe.
7. CHANGELOG + version bump recorded in the `inoculate` repo.

## Out of scope

Transmitting strains between agents (inoculate-spread) and recording carriage in
the ledger (inoculate-attest).
