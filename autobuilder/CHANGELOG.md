# Changelog

## v0.3.0 — 2026-09-05

The hermetic-build producer now attributes outbound sockets to the build's
own process tree (spawned cargo + its descendants, sampled via /proc every
150ms) instead of diffing the whole machine's /proc/net/tcp before/after —
so a cohabiting process's connection (agorabus, tailscale, a Claude
session, snap refreshes) never false-blocks the gate, and a build-tree
connection that opens and closes mid-build is still caught. Receipt schema
bumps to v2 (structured pid/comm/remote per socket; a v1 receipt on disk
is still accepted during the transition). Loopback and unix-domain traffic
stay ignored by default; `--strict` also attributes loopback.

## v0.2.0 — 2026-05-30

Add `autobuilder publish` subcommand that codifies the manual Stage-6 publish pipeline
(README/LICENSE generation, branch normalize to `main`, repo create via `wm-publish`,
push via `wm-push`, `REPOS.md` update) into a deterministic, idempotent, dry-run-capable
command. Shells out to safety wrappers; never calls `gh repo create` or `git push` directly.
Writes a `publish-receipt/v1` receipt to `target/autobuilder/receipts/publish-receipt.json`.
ACs 1–9 hermetic-green; AC10 (live network) deferred per PRD.
