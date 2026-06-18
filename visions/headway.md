# Vision: headway — the fleet catches up to its own source, the right way

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-18
**Status:** active
**Seed:** bare `/dream` (auto-mode, no topic), field FRESH (streak=0). Phase-1
live inspection found the one staleness class that has survived a full week of
self-reviews — `behind-head` (installed binary predates source HEAD) — has no
working automated remediation, and the path that claims to remediate it is
broken in two specific, citable ways.

## TL;DR

The laptop has a mature staleness *detection* toolchain — `binstale` classifies
running daemons (`fresh | deleted-exe | inode-drift | prov-stale | behind-head`),
`drydock` routes items into lanes (auto / window / reboot / approval), `rollout`
does serialized safe restarts, `agorabus doctor`/`reload` bounce a daemon to the
installed binary. Every other staleness class has a remediation that actually
runs. **`behind-head` does not** — it is the only class whose fix requires a
*recompile*, and the recompile path is broken:

1. **`rollout` builds locally.** Its default is `build_cmd = "cargo build
   --release"` (`rollout/src/fleet.rs:71`), a direct violation of the standing
   hard rule that *all* cargo routes through cloudbuild (Hetzner). The cloudbuild
   path exists only as a manual `#[ignore]` comment
   (`rollout/src/warmswap.rs:630`), never wired in.
2. **`rollout install` calls a flag that does not exist.** It shells to
   `agorabus reload --build` for the agorabus daemon (`rollout install --help`),
   but `agorabus reload` has no `--build` flag (`agorabus reload --help` lists
   only `--apply/--dry-run/--require-fresh/--start-if-absent/--format/--socket`
   and the timeouts). So the agorabus rebuild→reload bridge is a dead end.

This is exactly the item that has carried across self-reviews unchanged:
"agorabus 3d behind source; reload --build flag absent — needs cloudbuild +
reload --apply" (journal 2026-06-18, repeated 2026-06-17). The human re-runs the
manual `cloudbuild → cargo install → reload --apply` dance every time.

headway builds the sanctioned recompile-and-reload loop: one cloudbuild-backed
build primitive, the missing `agorabus reload --build` flag, a rollout that
routes its build through cloudbuild instead of local cargo, and an honest
convergence check that proves the daemon actually flipped `behind-head → fresh`
(and refuses to false-close when it didn't).

## End-state

When `binstale fleet` reports a daemon `behind-head`, a single command
recompiles it **through cloudbuild** (never local cargo), installs the fresh
artifact, reloads the owning daemon (bus reload for agorabus, `systemctl --user`
for the rest), then re-checks `binstale` and emits a receipt that either proves
the verdict flipped to `fresh` or surfaces a *contradicted* verdict for triage —
never a silent green. The `behind-head` row stops surviving self-reviews.

## Components (one bullet per future PRD)

- **headway-build** — NEW repo `~/wintermute/headway/` (rust-cli + lib). The
  sanctioned build primitive: given a crate dir, route the build through
  `~/.claude/skills/cloudbuild/cloudbuild.sh build <crate>`, pull the artifact,
  and emit a structured verdict (crate, artifact path, version/commit before →
  after, cloudbuild status). HARD: never local `cargo build`; abort + log a
  structured error if cloudbuild is unreachable — never a silent local fallback.
- **agorabus-reload-build** — rust-extend `build_into=~/wintermute/agorabus`. Add
  the `reload --build` flag the journal asked for and that `rollout install`
  already calls. On `--build`: detect `behind-head`, route the rebuild through
  cloudbuild (shell to `cloudbuild.sh`, NOT in-process cargo), install, then run
  the existing reload bounce. No-op when already fresh; honest abort when
  cloudbuild is down. Closes the phantom-flag bug.
- **headway-rollout-cloudbuild** — rust-extend `build_into=~/wintermute/rollout`.
  Replace rollout's local `cargo build --release` default with the cloudbuild
  route (consume headway-build). Keep rollout's proven orchestration (serialized,
  re-register confirm, voice `--restart-window` guard). A `--local-build` escape
  hatch is allowed only if it logs loudly; the default must never build locally.
- **headway-verify** — rust-extend `build_into=~/wintermute/headway`. Close the
  loop honestly: after install + reload, re-run `binstale check` on the daemon
  and assert the verdict flipped `behind-head → fresh`; emit a receipt. If still
  behind-head, surface a `contradicted` verdict (reuse the answerable/threshold
  "never silently drop a contradicted verdict" precedent) — do not false-close.

## Order

```
headway-build ─┬─► headway-rollout-cloudbuild
               └─► headway-verify
agorabus-reload-build   (parallel; shells to cloudbuild.sh directly,
                         no hard dep on headway-build being installed)
```

headway-build is foundational. rollout-cloudbuild and headway-verify both extend
/ consume it and are independent of each other (parallelizable).
agorabus-reload-build is independent — agorabus must not hard-depend on a sibling
CLI being installed, so it shells to `cloudbuild.sh` directly.

## Open questions

- Should headway register as drydock's *rebuild lane* so `behind-head` items
  route to it automatically (instead of landing in window/approval with no
  actionable fix)? Left as a future **headway-lane** PRD — drydock's lane set
  (auto/window/reboot/approval) is confirmed, but whether `behind-head` currently
  routes anywhere actionable is not, so this stays an open question, not a draft.
- cloudbuild artifact-pull path + naming: headway-build must learn where
  `cloudbuild.sh build` deposits the pulled binary. Confirm against the `build`
  verb (`cloudbuild.sh:512`, "up → build → pull → DOWN") before relying on it.
- Should `headway-verify`'s contradicted-verdict path emit to docket/litmus so a
  stuck `behind-head` becomes a tracked finding rather than a one-shot receipt?
