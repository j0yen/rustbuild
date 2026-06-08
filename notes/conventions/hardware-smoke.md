# Convention: hardware-smoke tests for wm-* daemons

**Established:** 2026-05-28 via PRD-wintermute-hardware-smoke-convention
**Status:** active — applies to every wm-* daemon whose ACs include
ground-truth-required measurements (mic latency, speaker timing, RSS
soak, AEC behavior, network round-trip, systemd recovery)
**Canonical prior art:** `wintermute-tts/tests/hardware_acs.rs` (lines 1–105)

## Why this exists

The four Fleet 1 daemons that survive past `cargo test --release` still
encounter `/build`'s verified-completed check #5 ("every AC paired with
a passing cargo test or smoke command"). Many of their ACs measure
real-hardware behavior — audio sink latency to the speaker, mic capture
under AEC, false-accept rate over 60 minutes, PipeWire restart recovery
— which no in-process Rust test can assert ground truth on without an
operator-side witness.

`wintermute-tts` shipped 2026-05-28 by pairing AC1/AC3/AC5/AC7 with
`#[test] #[ignore]` stubs in `tests/hardware_acs.rs`. Each stub's body
is `require_hardware_witness("ACx")`, which panics unless
`WM_TTS_HARDWARE_SMOKE=1` is set. The doc-comment is the manual
procedure verbatim — the test is documentation that happens to compile.

`/build`'s check #5 accepts this: the AC *is* paired with a named cargo
test. The test is a contract that an operator-attested run can fail
against, not a substitute for that run.

This convention codifies that pattern and makes it the default for any
fleet daemon whose ACs cannot be satisfied from inside `cargo test`
alone.

## Where

`tests/hardware_acs.rs` in the repo root. One file per repo. Never
per-AC files; group all hardware-gated ACs into the single file so the
doc-comments live next to each other.

If the daemon also has a deterministic bus-smoke or unit-paired path
for some ACs (e.g., wake-event-emission paired by `wake_bus_smoke`),
those ACs stay paired by their existing tests; `hardware_acs.rs`
covers only the residual hardware-witnessed claims.

## Env-witness naming

`WM_<UPPERSLUG>_HARDWARE_SMOKE=1`

The slug is the binary slug minus the `wintermute-` prefix:

| Repo | Env var |
|---|---|
| wintermute-tts | `WM_TTS_HARDWARE_SMOKE` |
| wintermute-stt | `WM_STT_HARDWARE_SMOKE` |
| wintermute-audio | `WM_AUDIO_HARDWARE_SMOKE` |
| wintermute-platform | `WM_PLATFORM_HARDWARE_SMOKE` |
| wintermute-dialog | `WM_DIALOG_HARDWARE_SMOKE` (reserved; dialog's ACs are software-timed today) |
| wintermute-brain | `WM_BRAIN_HARDWARE_SMOKE` (reserved) |

Only `=1` is accepted. Anything else (including `=0`, `=true`, unset)
panics. The check is intentionally narrow so the env var cannot drift
into "kinda accidentally set" in CI or a stray shell session.

## Required shape

```rust
//! Hardware-dependent acceptance tests for <list AC numbers>.
//!
//! These ACs measure <one-sentence what>. The deterministic, in-process
//! portion of each path is already covered by <name the unit / smoke
//! tests>. What lives here is the contract that the rest of each AC —
//! <one-line what the witness covers> — is exercised manually on a
//! machine with the relevant hardware.
//!
//! Each test is `#[ignore]`-gated. Run with:
//!
//!     cargo test --release --test hardware_acs -- --ignored --nocapture
//!
//! The doc-comments on each test name the operator-side procedure;
//! the test body itself is a sentinel that fails if invoked without
//! a `WM_<SLUG>_HARDWARE_SMOKE=1` environment witness, so an
//! accidental `--ignored` run on CI cannot silently report
//! "all passed".

#![allow(clippy::expect_used, clippy::panic, clippy::missing_panics_doc)]

use std::env;

fn require_hardware_witness(ac: &str) {
    let witness = env::var("WM_<SLUG>_HARDWARE_SMOKE").unwrap_or_default();
    assert_eq!(
        witness, "1",
        "{ac}: this is a hardware-timing smoke test. \
         Set WM_<SLUG>_HARDWARE_SMOKE=1 and run on a machine with \
         <required hardware>. See doc-comment for the manual procedure."
    );
}

/// AC<N> — <verbatim AC headline from PRD §4>.
///
/// Manual procedure:
///   1. <step>
///   2. <step>
///   3. <assertion + receipt path>
#[test]
#[ignore = "hardware: <one-line reason>"]
fn <descriptive_snake_case_name>() {
    require_hardware_witness("AC<N>");
}
```

Doc-comments are not optional. They are the only documentation an
operator has for the procedure. Reference verbatim AC text from the
PRD §Acceptance section where possible; an AC sentence and its
hardware procedure must be readable side-by-side.

## What counts as "hardware-gated"

An AC belongs in `hardware_acs.rs` when its assertion needs at least
one of:

- A live audio input (mic) or output (speaker) — first-frame latency,
  AEC, NoiseTorch suppression, false-accept rate, soak.
- A live network round-trip — cloud TTS/STT first-byte, OAuth flow.
- A live systemd-user instance — `systemctl restart` recovery, target
  ordering, restart-storm backoff.
- A real model artifact on disk that the repo deliberately doesn't
  pre-cache — distil-small whisper, Piper voicepack warmup.
- A multi-process orchestration with another daemon that's only
  available at runtime — `wm-audio` ↔ `wm-stt` resubscribe.

ACs that can be exercised against in-process synthetic fixtures
(intent decode, state machine, queue math, parser correctness) do NOT
belong here. They get unit or smoke tests in the lib or
`tests/<feature>_smoke.rs`.

## Promotion path

When an AC's procedure becomes automatable (e.g., a CI mic loopback,
a recorded-PCM replay harness, a containerized PipeWire instance),
the stub's body changes from `require_hardware_witness(...)` to the
real assertion. The doc-comment stays — it's now both
machine-checked and human-readable. The `#[ignore]` attribute can
be dropped only if the test runs deterministically without external
state.

Until that happens, the stub is the contract. `/build` treats a
witness-gated stub as a valid AC pairing for verified-completed
check #5.

## What `/build` checks

When `/build` evaluates verified-completed check #5 for a PRD that
declares hardware-gated ACs, it looks for:

1. A `tests/hardware_acs.rs` in the target repo's root.
2. A `#[test]` whose `#[ignore]` attribute mentions `hardware:`
   for each declared hardware AC.
3. A `require_hardware_witness` call (or equivalent panic-on-missing-
   env-var sentinel) inside the test body.

The presence of these three things satisfies check #5 for that AC,
without `/build` ever running the test under `--ignored`. The
operator-witnessed run is jsy's responsibility (and the doc-comment's
procedure is the contract for it).

## Receipt convention

When the operator runs `cargo test --release --test hardware_acs --
--ignored` with the env witness set, each stub that has a real
measurement target should write a JSON receipt to
`target/ac<N>_<descriptive>.json` (see wintermute-tts AC1/AC3/AC5/AC7
for canonical filenames). The receipt fields are:

```json
{
  "ac": "ACx",
  "ts_utc": "2026-05-28T12:34:56Z",
  "host": "wintermute",
  "git_sha": "deadbeef",
  "measurements": [...],
  "verdict": "pass|fail",
  "notes": "..."
}
```

This is captured-receipt territory; `/build` does not require it for
check #5, but future `wm-verify` (Fleet 1.5 bullet) will index these
receipts to build attestation logs.

## Cousin conventions

- `bus-smoke.md` — in-process agorabus smoke tests for daemons. The
  hardware-smoke pattern is the *operator-witnessed* counterpart for
  ACs that cannot be exercised against an in-process bus.
- `PRD-build-deferred-acs.md` (queued) — frontmatter-level deferral
  for ACs that have no natural Rust pairing surface at all (e.g.,
  install-script success, third-party OAuth). Hardware-smoke is for
  ACs that *can* be paired with a Rust stub; deferred-acs is for ACs
  that cannot. The two are complements, not competitors.
