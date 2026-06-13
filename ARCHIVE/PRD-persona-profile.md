# PRD: persona-profile — one named declaration becomes a whole self

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/wintermute-brain
Vision: visions/persona.md

## TL;DR

A persona is currently **scattered**: register, self_name, wake_word,
max_sentences, greeting, `forbidden_terms`, and the introduction mode all live
as separate knobs in the `[persona]` table of `brain.toml` (and the intro mode
in its own sub-table). The shipped `jocelyn` work covers exactly one of these
fields — the forbidden-vocabulary defaults — and leaves the rest to be
hand-assembled. There is no single artifact that says "this device *is*
Jocelyn": warm-elder register **and** her forbidden list **and** the name
ceremony **and** a brief reply cap, as one coherent identity. `persona-profile`
adds a **named profile registry** to `wintermute-brain` and a
`wm-brain persona` subcommand that materializes a complete, internally
consistent `[persona]` configuration from one name — print it, diff it against
the live file, or apply it. Deploying Jocelyn's device becomes
`wm-brain persona apply jocelyn`, not a dozen manual TOML edits.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The live config proves the assembly gap.** `~/.config/wintermute/brain.toml`
  `[persona]` table read this session: `self_name="wintermute"`,
  `register="warm-elder"`, `addresses_user=true`, `max_sentences=3`,
  `greeting="off"`, `wake_word="hey wintermute"`. It contains **no
  `forbidden_terms`** and **no introduction mode** — the very fields the
  shipped persona PRDs added. The Jocelyn identity exists in code presets and
  tests but has never been assembled into a usable deployment.
- **`jocelyn` is preset for one field only.** `wintermute-brain/src/lib.rs`
  test `forbidden_terms_jocelyn_preset` (≈line 1753) defines the forbidden
  list, but nothing pairs it with the warm-elder register, a self_name, the
  `FirstEverBoot` introduction mode, or the brevity cap. The pieces ship
  separately; the *person* is never composed.
- **The mechanisms are all built and waiting.** `Register::{WarmElder,Plain,
  Brisk}`, `IntroductionMode::{Off,FirstEverBoot,Explicit}`
  (`src/introduction.rs`, with `wm.persona.introduce` wired in
  `src/daemon.rs`), and `forbidden_terms` (`src/lib.rs:184`) all exist. What is
  missing is the layer that **binds them into one named whole** — the
  difference between a parts bin and an identity.
- **The vision asked for this in spirit.** `visions/persona.md` open question
  1 ("what name does Jocelyn call the assistant?") and the `persona-work`
  component both describe materializing a *named identity* into a file. This
  PRD is that materialization for the **voice** principal; `persona-work` is
  its sibling for the **work** machine.

## What this builds

A profile registry + a thin CLI surface in `wintermute-brain`. Read-only by
default; writing the live config is explicit and reversible.

- **New module `src/profile.rs`.**
  - `struct PersonaProfile { name, register, self_name, wake_word,
    max_sentences, addresses_user, forbidden_terms, introduction,
    redline }` — a complete persona, all fields the daemon already understands.
    (`redline` couples to PRD-persona-redline if shipped; the field is
    optional/defaulted so this PRD builds independently.)
  - `fn builtin(name: &str) -> Option<PersonaProfile>` with at least two
    presets:
    - **`jocelyn`** — `WarmElder`, `self_name` placeholder (deployment sets
      the real name per vision open-question 1), the shipped forbidden list,
      `max_sentences = 2`, `IntroductionMode::FirstEverBoot { ack_timeout_secs:
      25 }`.
    - **`default`** — today's live values (`wintermute`, warm-elder, no
      forbidden terms, intro `Off`) so `apply default` is a documented identity,
      not a special case.
  - `fn to_toml_fragment(&self) -> String` — emits a valid `[persona]` (+ intro
    sub-table) TOML block that round-trips through the existing serde loader.
- **CLI subcommand `wm-brain persona`** (or a sibling `persona` binary in the
  crate — whichever matches the crate's existing bin layout):
  - `persona list` — names + one-line descriptions of built-in profiles.
  - `persona show <name>` — print the composed `[persona]` TOML to stdout.
  - `persona diff <name> [--config <path>]` — show a field-level diff between
    the named profile and the live `brain.toml` (default
    `~/.config/wintermute/brain.toml`); exit 0 if identical, 1 if they differ.
  - `persona apply <name> [--config <path>] [--write]` — without `--write`,
    print exactly what *would* change (the diff) and exit without touching the
    file; with `--write`, back up the existing file to `brain.toml.bak` first,
    then replace only the `[persona]` and introduction sections, preserving all
    other tables (`[routing]`, etc.) byte-for-byte.
- **Safety.** `apply --write` never edits any table other than persona/intro;
  it round-trips the rest of the document unchanged (parse → mutate the two
  sections → re-serialize the whole doc, asserting non-persona tables are
  identical). `sigpipe::reset()` first line of the bin's `main`.

Deps: a TOML editing crate that preserves unrelated tables (`toml_edit`) if not
already present; serde already in-tree. MSRV 1.85, no let-chains.

## Acceptance criteria

1. `persona list` prints at least `jocelyn` and `default` with descriptions.
2. `persona show jocelyn` emits a `[persona]` block whose `register` is
   `warm-elder`, whose `forbidden_terms` is the shipped Jocelyn list
   (non-empty), and whose introduction mode is `FirstEverBoot`.
3. The TOML emitted by `persona show <name>` parses back through the existing
   `PersonaConfig` serde loader without error and composes a base persona
   (no `{self_name}`/`{user_name}` tokens leak).
4. `persona diff default` against a `brain.toml` holding today's live values
   exits 0 (identical); `persona diff jocelyn` against the same file exits 1
   and lists `forbidden_terms` and the introduction mode among the differences.
5. `persona apply jocelyn` **without** `--write` prints the would-change diff
   and leaves the target file unmodified (byte-compare before/after).
6. `persona apply jocelyn --write --config <tmp>` writes the Jocelyn
   `[persona]` + intro sections, creates `<tmp>.bak`, and leaves a pre-existing
   `[routing]` table in `<tmp>` byte-for-byte identical.
7. Round-trip: after `apply --write`, `persona diff jocelyn --config <tmp>`
   exits 0.
8. An unknown name (`persona show nobody`) exits non-zero with a clear message
   and changes nothing.
9. `cargo test` green; `cargo build` clean; existing persona/register/intro
   tests unchanged.
