# PRD: inoculate-signet — provenance-signed strains

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/inoculate
Vision: visions/inoculate.md

## TL;DR

`inoculate spread` now transmits the ethics strain horizontally over agorabus,
and the carrier-check (`challenge`/`verify`) proves a peer *knows* the strain
content. But both are **symmetric**: the carrier-check is HMAC-blake3 (a shared
secret), and `spread announce` publishes strain metadata with no proof of
*origin*. Any peer holding the strain can forge an `announce` for an arbitrary
version, and any peer can answer a challenge. Horizontal spread is therefore
**tamper-evident in content but not in provenance** — a hostile or buggy peer
can announce a "newer" strain and have honest peers converge onto it. This PRD
adds **asymmetric signing**: this box holds an ed25519 keypair, signs the strain
hash, and attaches a detached signature to every `spread announce`. Peers verify
the signature against a known public key before caching or converging. The
strain becomes provenance-attested: "this strain came from this box," not merely
"this strain has this content."

## Why this exists

Verified live 2026-06-15 on this box:

- `inoculate spread announce` (per `inoculate spread --help`) publishes to
  `inoculate.strain.announce` and `spread listen` "subscribe to peer
  announcements; cache newer strains locally." Nothing in that path
  authenticates the announcer — convergence is on version/content alone.
- The carrier-check is explicitly "HMAC-blake3 form" (`inoculate --help`:
  `challenge`/`verify`/`self-attest`). HMAC is a *shared-secret* MAC: it proves
  the responder holds the same strain bytes, not that any particular box
  authored or vouches for them. It cannot distinguish "the real strain from
  this box" from "an identical-looking strain a peer fabricated."
- `inoculate`'s `Cargo.toml` already pulls `blake3 = "1"` and `sha2 = "0.10"`
  (read this session) — hashing is in place; signing is not. No ed25519 / ring /
  signature crate is present.
- No signing key material exists on the box for this purpose: `ls ~/wintermute/signet`
  is absent, `~/.local/bin` has no `sign`/`signet`/`key` tool, and only
  `/usr/sbin/ssh-keygen` is installed (no `minisign`/`age`). So this PRD must
  *create and manage* the strain-signing keypair, not assume one.
- The `inoculate` vision's open questions (read this session, `visions/inoculate.md`)
  name this exactly: *"Should the strain be **signed** (a `signet`-style key) so
  a carrier check can prove provenance (this strain came from this box), not just
  content? … likely a 7th PRD, deferred until core + spread expose the shape."*
  As of the 2026-06-15 changelog (`~/.claude/CLAUDE_SELF.md`), both prerequisites
  — `inoculate-core` (`strain`/`hash`) and `inoculate-spread` (`announce`/`listen`)
  — have shipped. The shape is now exposed; the deferred PRD is unblocked.
- A sibling `signet` vision already exists (`visions/signet.md`) but governs the
  kernel `agent_session_id` reading surface — a different concept. This PRD
  reuses the *name idea* (a cryptographic signet of provenance) for strain
  authorship, and should cross-reference but not depend on it.

## What this builds

A `rust-extend` of `~/wintermute/inoculate` adding signed-provenance to the
strain, built from the existing `blake3`/`sha2` foundation plus one signing dep.

### New dependency

- `ed25519-dalek` (with the `rand_core` feature for keygen). Detached
  signatures over the strain hash; small, audited, no-network. (If the box's
  pinned toolchain forces a specific version, the autobuilder loop resolves it;
  the AC is "ed25519 detached signatures," not a version.)

### Key material

- A strain-signing keypair stored under `~/.config/inoculate/`:
  - `signing.key` — the ed25519 secret key (mode `0600`).
  - `signing.pub` — the ed25519 public key, world-readable, also publishable on
    the bus so peers can pin it.
- Keys are created on first `sign`/`keygen` if absent; never overwritten
  silently (refuse + nonzero exit if a key already exists, unless `--force`).
- The public key is identified by its own blake3 fingerprint (`pubkey_fp`) so
  peers can refer to "the key that signed this" without transmitting raw bytes
  inline.

### New / extended subcommands

- `inoculate keygen` — generate the strain-signing keypair if absent; print the
  public key + `pubkey_fp`. Idempotent guard (refuse to clobber without
  `--force`).
- `inoculate sign` — compute the current strain hash (reusing the existing
  `hash` path) and emit a detached signature object as JSON:
  `{ strain_version, strain_hash, pubkey_fp, sig (base64), signed_at }`.
  (`signed_at` is supplied by the caller/env to stay deterministic-testable; do
  not call wall-clock inside the signing core.)
- `inoculate verify-sig` — given a signature object (stdin or `--file`) and a
  public key (`--pub <path>` or a pinned trust store under
  `~/.config/inoculate/trusted/`), verify (a) the signature is valid for the
  embedded `strain_hash` under that pubkey, and (b) the embedded `strain_hash`
  matches the *locally computed* strain hash when `--match-local` is passed.
  Exit `0` = valid+provenance-trusted, `1` = valid sig but untrusted/unpinned
  key, `2` = bad signature or hash mismatch. (Mirrors the existing
  allow/flag/redline exit convention.)
- `inoculate spread announce` — **extended** to attach the detached signature to
  the published metadata when a signing key is present (omit it, with a stderr
  notice, when no key exists — announce still works unsigned for back-compat).
- `inoculate spread listen` — **extended** to verify a peer announcement's
  signature before caching: an announcement with a missing/invalid signature, or
  one signed by an unpinned key, is logged and **not** converged onto (it may
  still be cached in a quarantined `unverified/` area for inspection). Add a
  `--require-signed` flag (default on once a trust store is non-empty) so the
  convergence path refuses unsigned strains.
- `inoculate spread status` — **extended** to show, per heard peer, whether its
  strain was signature-verified and which `pubkey_fp` signed it.

### Trust store

- `~/.config/inoculate/trusted/<pubkey_fp>.pub` — a directory of pinned peer
  public keys. `inoculate trust add <path>` / `inoculate trust list` manage it.
  On a single box today the only pinned key is this box's own; the design is
  forward-compatible with `constellation` (each node pins its peers' keys), but
  this PRD does NOT build any network key-exchange — pinning is manual/local,
  matching the current single-box reality.

## Acceptance criteria

1. `inoculate keygen` creates `~/.config/inoculate/signing.key` (mode `0600`)
   and `signing.pub` when absent, prints the public key and its `pubkey_fp`, and
   refuses to overwrite an existing key without `--force` (nonzero exit, key
   left intact).
2. `inoculate sign` emits a JSON signature object containing `strain_version`,
   `strain_hash` (equal to `inoculate hash` output), `pubkey_fp`, a base64
   `sig`, and the caller-supplied `signed_at`; signing twice over the same
   strain + key + `signed_at` is byte-identical (deterministic).
3. `inoculate verify-sig` returns exit `0` for a signature produced by `sign`
   under a pinned/trusted key, exit `2` if the signature bytes or `strain_hash`
   are tampered, and exit `1` for a valid signature whose `pubkey_fp` is not in
   the trust store.
4. `inoculate verify-sig --match-local` additionally fails (exit `2`) when the
   signature's embedded `strain_hash` does not match the locally computed strain
   hash, and passes when it does.
5. `inoculate spread announce` includes the detached signature in its published
   metadata when a signing key exists, and still announces (unsigned, with a
   stderr notice) when no key is present.
6. `inoculate spread listen` with a non-empty trust store does **not** converge
   onto an announcement carrying a missing, invalid, or untrusted-key signature
   (verified via a crafted bad-signature fixture); a validly signed,
   trusted-key, higher-version announcement **does** get cached/converged.
7. `inoculate trust add` / `trust list` manage `~/.config/inoculate/trusted/`,
   and `inoculate spread status` reports per-peer signature-verification state
   and signing `pubkey_fp`.
8. The secret key is never printed by any subcommand, never logged, and is not
   included in `sign`/`announce` output (only the public key / `pubkey_fp` and
   signature appear); a test asserts the secret bytes do not appear in any
   command's stdout/stderr.

## Out of scope (left for the vision)

- Network key exchange / PKI between constellation nodes — deferred to the
  `constellation` transport decision (open question in the vision).
- Hard-gating un-inoculated subagents — still flag-only per the vision's
  "start with flag" decision; signing makes a future hard gate *trustworthy* but
  does not flip it.
- Key rotation/revocation lists — note as a future PRD once a second box exists
  to rotate against.
