# PRD: rosetta-credential — an ethical clearance as a signed W3C Verifiable Credential

Status: Draft v0.1
build_target: rust-cli
Vision: visions/rosetta.md

## TL;DR

An `ousia-guard` verdict is only as trustworthy as the box that produced it.
When an ethical clearance leaves this laptop — attached to a published repo, a
shipped skill, a delegated action — a third party has no way to verify it
without trusting us. `rosetta-credential` wraps a guard verdict and its PROV-O
graph (from `rosetta-prov`) as a **W3C Verifiable Credential** in JSON-LD,
**signed** with the `inoculate-signet` key, so an ethical clearance becomes a
portable, cryptographically verifiable claim. `verify` re-checks the signature
and the embedded claim offline.

## Why this exists

Phase-1 live inspection (2026-06-15):

- `inoculate/src/signet.rs` already implements strain signing
  (provenance-signed strains, inoculate v0.5.0 per CLAUDE_SELF changelog); the
  SessionStart preamble even carries a live `strain_hash`. The signing primitive
  and key management exist — nothing yet signs a *decision*.
- `answerable` records consequential actions but its JSONL ledger is local and
  unsigned; it proves nothing to anyone off-box.
- W3C Verifiable Credentials are JSON-LD by construction — they are the standard
  way to make a claim ("this action was ethically cleared") portable and
  verifiable. The semantic-web answer to "trust this decision without trusting
  the issuer."
- `rosetta-prov` (foundational PRD) produces the PROV-O graph that becomes the
  credential's `credentialSubject` evidence.
- `grep` over all PRDs + visions: zero Verifiable-Credential coverage.

## What this builds

New repo `~/wintermute/rosetta-credential/` (rust-cli, edition 2021, MSRV 1.85).

**Deps:** `serde` / `serde_json` (JSON-LD documents), `oxrdf`/`oxrdfio` (embed
the PROV-O evidence), the `inoculate-signet` crate (path/git dep) **or** the
same Ed25519 crate inoculate uses (`ed25519-dalek`) reading the same keyfile —
choose the path that does not duplicate key management; the PRD's intent is to
*reuse inoculate-signet's key*, not mint a parallel identity. `clap` v4.

**Credential shape (W3C VC Data Model 2.0, JSON-LD):**

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2",
               "urn:wintermute:rosetta:v1"],
  "type": ["VerifiableCredential", "EthicalClearanceCredential"],
  "issuer": "urn:wintermute:signet:<key-id>",
  "validFrom": "2026-06-15T...Z",
  "credentialSubject": {
    "action": "urn:wo:action:abc",
    "verdict": "deny",
    "rulesFired": ["dignity-floor"],
    "provenance": { ...embedded PROV-O graph as JSON-LD... }
  },
  "proof": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-...",
             "verificationMethod": "urn:wintermute:signet:<key-id>",
             "proofValue": "<base64 Ed25519 signature>" }
}
```

**Scope note (carried from the vision):** full W3C *Data Integrity* requires RDF
Dataset Canonicalization (RDFC-1.0) before signing. v1 signs a **stable,
sorted-key JSON-LD serialization** (documented as a simplification) and records
the canonicalization method in the proof so a future PRD can upgrade to true
RDFC-1.0 without breaking the credential shape. This is an honest, stated
limitation — not silent.

**CLI:**

```
rosetta-credential issue  --verdict v.json [--prov prov.ttl]   # → signed VC (JSON-LD)
rosetta-credential verify --credential vc.jsonld               # check sig + claim → exit 0/1
rosetta-credential inspect --credential vc.jsonld              # print subject without verifying
```

`issue` invokes `rosetta-prov` logic (or shells to it) to build the PROV-O
evidence, assembles the VC, canonicalizes per the v1 method, signs with the
signet key, and emits JSON-LD. `verify` recomputes the canonical form, checks
the Ed25519 signature against the issuer key, and validates required VC fields.

## Acceptance criteria

1. `rosetta-credential issue --verdict <fixture>` emits a JSON-LD document that
   parses as valid JSON and contains `@context`, `type` including
   `VerifiableCredential`, `issuer`, `credentialSubject`, and a `proof`.
2. The `credentialSubject` carries the verdict, the `rulesFired` list, and an
   embedded PROV-O `provenance` object (non-empty).
3. `rosetta-credential verify` on a freshly issued credential exits 0 and
   prints a confirmation including the issuer key id.
4. Tampering with any byte of `credentialSubject` (e.g. flipping `deny`→`allow`)
   and re-running `verify` exits 1 with a signature-mismatch diagnostic.
5. The signing key is the **same** key/identity `inoculate-signet` uses — proven
   by a test that signs here and verifies with inoculate's verify path (or
   vice-versa), or by documented shared keyfile + matching key id.
6. `rosetta-credential inspect` prints the subject and proof metadata **without**
   performing verification, and exits 0 even for an invalid signature.
7. The `proof` records the v1 canonicalization method explicitly, and the README
   states the RDFC-1.0 simplification and the upgrade path.
8. A credential missing a `proof` fails `verify` with exit 1 and a clear
   "unsigned credential" message (not a panic).
