# PRD: inoculate-immune — a persona cannot weaken the floor

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/wintermute-brain
Vision: visions/inoculate.md

## TL;DR

The other half of "inoculate" is immunity. This box runs **persona overlays**
(`persona-redline`, `persona-forbidden-vocab`, `persona-profile` — all in
wintermute-brain) that reshape how a sub-identity behaves: an elderly-mother
persona, a work-laptop persona. An overlay should be able to *add* scruples, but
never *remove* a base Boundary. `inoculate-immune` makes the base strain a
non-overridable floor: persona resolution is constrained so the effective ethic
is always `base ∪ persona`, never `persona` alone.

## Why this exists

Verified live: `persona-redline` (output enforcement), `persona-forbidden-vocab`,
and `persona-profile` (named registry) all ship in `~/wintermute/wintermute-brain`
(`src/redline.rs` present; CLAUDE_SELF.md changelog 2026-06-12..13 records the
persona arc). The user's own intent (memory: persona sister-project) is an
elderly-mother persona as *primary user*. A persona that could silently relax a
base Boundary (e.g. "irreversible actions ask first") would be a real ethical
hole — exactly the drift `answerable values-drift` watches for in the base file,
but unwatched at the persona layer.

## What this builds

Extends `wintermute-brain`'s persona/redline resolution, depending on
`inoculate-core` for the base strain:

- A `floor` concept in persona resolution: when composing the effective policy
  for a persona, load the base strain's Boundaries/redlines as a mandatory floor.
  The effective set = `base ∪ persona_additions`; any persona rule that would
  *weaken* a base Boundary (remove it, or widen an allow that the base forbids) is
  rejected at load time with a named error.
- `persona lint <profile>` (or extend the existing persona doctor) — report
  whether a persona profile is floor-compliant: list any rule that attempts to
  lower the floor. Exit 0=compliant, 1=would-weaken.
- Resolution precedence is documented and tested: base-forbid always wins over
  persona-allow; persona-forbid adds to base.
- The base strain hash the floor was computed from is recorded on the resolved
  policy (ties to inoculate-attest: a persona action's ledger entry can name both
  the persona and the base strain version it was floored against).

## Acceptance criteria

1. `cargo test --release` passes in `wintermute-brain`; binaries reinstall.
2. A persona profile that re-allows a base-forbidden action is rejected at load
   with a named error (`floor-violation: <rule>`), asserted by a fixture profile.
3. A persona profile that only *adds* constraints loads cleanly and its effective
   policy = base ∪ additions, asserted against fixtures.
4. `persona lint` exits 1 and names the offending rule for a weakening profile;
   exits 0 for a compliant one.
5. Precedence: a test proves base-forbid beats persona-allow, and persona-forbid
   adds to base-allow.
6. The resolved policy carries the base `strain_hash` it was floored against.
7. No regression in existing persona/redline tests; CHANGELOG + version bump.

## Out of scope

Producing the strain (inoculate-core), transmitting it (spread), and recording
actions (attest). This PRD only makes the base ethic a non-lowerable floor under
persona overlays.
