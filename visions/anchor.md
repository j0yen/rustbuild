# Vision: anchor — the change-observation layer stays anchored across reboot

## TL;DR

`wchg`/watchman is the substrate the daily self-review stands on to know
*where the laptop actually changed this week*. But watchman runs `--inetd`
socket-activated with its root list held only in memory, so a reboot (or a
socket bounce) silently drops every watched root. The delta tooling then
returns empty or errors **without saying why**, and self-review re-watches
`~/.claude` + `~/brain` by hand, run after run. **anchor** makes the
observation layer durable: a declared set of watch roots, an idempotent
reconciler that re-asserts them at boot and at session start, and a health
probe so the delta layer fails *loud* instead of silent. The 2026-06-03
journal proposed "a SessionStart re-watch hook if it recurs" — it recurs;
anchor is that hook, generalized into a small tool.

## Why now (evidence)

- **8+ reflective memories** record the same loss, recurring across reboots —
  top hit `recall:01KT6VCBX1PSWT9MAQP607TAWY` score 10.3:
  *"watchman drops watched roots across reboot — wchg since/reset silently
  empty/error until re-watch."* Also `01KSTJ29…`, `01KSTRY6…`, `01KSGR03…`
  (*"socket teardown happens mid-session, not just at reboot"*), `01KSMZNQ…`,
  `01KSSFPE…`, `01KS95K6…`.
- **All three most recent journals** (06-01, 06-02, 06-03) re-derive the same
  manual fix. 06-03 Notable: *"watchman drops watched roots across a reboot —
  `wchg since`/`reset` silently returned empty/errored until re-`watch`. Worth
  a SessionStart re-watch hook if it recurs."* It recurs.
- **Live state confirms the mechanism**: `watchman.service` is
  `socket`-triggered `/usr/bin/watchman --foreground --inetd` (no persisted
  root restore in this config); `wchg list` shows mixed clock ages
  (`c:1779915649…` from mid-May beside `c:1780493700…`), the fingerprint of
  ad-hoc per-root re-watching rather than a coherent declared set.
- **No SessionStart hook re-watches roots today** (8 SessionStart hooks wired;
  none touch watchman). The proposed fix was never built.

## End-state

When this is done:

- There is a **canonical declared set** of watch roots at
  `~/.config/anchor/roots.toml` — the roots self-review and other delta
  consumers depend on (`~/.claude`, `~/brain`, `~/wintermute`, `~/.local/bin`,
  `~/.config/systemd/user`, …) — versioned, not implicit.
- A **boot oneshot** (After watchman.service) and a **SessionStart hook**
  run `anchor reconcile --apply`, which idempotently re-asserts any missing
  watch and re-seeds the `wchg` cursor, so a reboot no longer costs a
  hand-re-watch and no longer silently zeroes the week's delta.
- `anchor probe` lets self-review (and any wchg consumer) **fail loud**: it
  reports lost roots, stale clocks, and a dead watchman socket as structured
  JSON with a non-zero exit, so "empty delta" can never again be mistaken for
  "nothing changed."
- anchor is **inward toolkit** — sibling of vigil/quicken/keel/binstale —
  published as a `j0yen` repo like the rest of the self-tooling.

## Components (one bullet per future PRD)

- **anchor-roots** (rust-cli, NEW repo) — creates the `anchor` workspace +
  binary; the declared-roots manifest loader (`RootsConfig`), the shared types
  (`WatchRoot`/`WatchState`/`RootStatus`/`ReconcilePlan`/`ReconcileAction`), the
  `WatchBackend` trait, and a **pure** `reconcile` that diffs declared-vs-live
  and emits a print-only plan. `anchor plan`. This is anchor's corpus.
- **anchor-probe** (rust-extend) — pure-read health probe: lost roots, stale
  clocks (> threshold vs declared freshness), dead watchman socket. Structured
  JSON + non-zero exit so wchg consumers stop failing silently.
- **anchor-reconcile** (rust-extend) — `anchor reconcile --apply`: idempotently
  re-assert missing watches and re-seed wchg cursors through the `WatchBackend`.
  Print-only by default; `--apply` is the one live-side-effect path.
- **anchor-boot** (shell/hooks/config) — the systemd-user boot oneshot
  (`anchor-reconcile.service`, After/Wants watchman.service) **and** the
  SessionStart hook that run `anchor reconcile --apply`, retiring the manual
  re-watch dance. This is the journal's proposed fix, finally shipped.

## Order (STRICT hard dependency)

```
anchor-roots  (FIRST — creates the repo + types + WatchBackend trait + pure reconcile)
      │
      ├──> anchor-probe     ┐  (both depend on anchor-roots ONLY; build in PARALLEL)
      └──> anchor-reconcile ┘
                  │
                  └──> anchor-boot  (wires `anchor reconcile --apply`; needs the --apply path to exist)
```

Same hard rule that bit relay + concord + quicken + keel: **no rust-extend
starts until anchor-roots has SHIPPED and the repo exists**, or
extend-validate fails.

## Open questions (held for next /dream pass or the user)

- **anchor-watch** (a wm.anchor.* bus emitter on lost-root, for a homestead
  self-heal loop) — held; not yet motivated beyond the boot oneshot. Mirrors
  the keel-beacon / quicken-watch "held" notes.
- Should `anchor probe` feed the existing **docket** ledger (open a
  `watchman-root-lost` key when a declared root is missing at session start)
  rather than just exit non-zero? Leaning: probe stays pure/legible, the
  *hook* opens the docket key.
- Mid-session socket teardown (`01KSGR03…`) — the boot oneshot + SessionStart
  hook cover reboot and new-session, but not a watchman bounce mid-session.
  A `wchg`-shim that auto-reconciles-on-empty is a possible later PRD; held
  until the boot/session coverage proves insufficient.
- Watchman *can* persist watches via a state file — is the right fix actually
  configuring `watchman --persist` instead of a reconciler? Investigated
  enough to know the current `--inetd` unit doesn't restore; the reconciler is
  the robust, backend-agnostic answer (survives a watchman that forgets for
  *any* reason), but worth noting as an alternative the user may prefer.
