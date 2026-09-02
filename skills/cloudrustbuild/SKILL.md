---
name: cloudrustbuild
description: Run cargo build/test for a Rust crate on RedBaron (the fleet's Rust build machine) from carbon or ryzen7, over Tailscale SSH, with RedBaron's sccache and mold, and pull target/ back. On RedBaron itself cargo runs locally. Not related to Wintermute Hub (the Hetzner box that runs NATS). The Hetzner burst box was retired on 2026-09-01 — nothing here rents a server.
user_invocable: true
---

# /cloudrustbuild — cargo on RedBaron, from carbon or ryzen7

Fleet names: **RedBaron** (this Rust build machine: i7-11700KF, 16 threads,
30 GB, sccache + mold), **carbon**, **ryzen7**, and **Wintermute Hub** (the
Hetzner box that runs NATS; it builds nothing). This skill lets carbon and
ryzen7 build on RedBaron: `cloudbuild.sh` syncs
the crate to `~/build/<crate>` on RedBaron, runs the cargo command there as `jsy`,
prints the sccache hit rate, and rsyncs `target/` back. All three machines are
Ubuntu 26.04 x86_64, so the binary runs where it was requested.

On RedBaron, do not use this skill for RedBaron's own builds; run `cargo` directly.

**Retired (2026-09-01):** the Hetzner burst box, snapshots, `session-start`/`-end`,
`up`/`down`/`keep-build`/`fleet`, the cost log and the watchdog. Those entry points
now error (or no-op for `session-*`) so a stale caller fails loudly instead of
renting a server. If RedBaron is unreachable the script exits 2; it never bursts
and never falls back to a local build on its own.

The script's config file is still called `hub.json` and its log lines say
`warm hub` — that is the script's old name for "the standing build machine",
and it means RedBaron. It has nothing to do with Wintermute Hub.

## Config

`~/.config/wm-burst/hub.json` on each client:

```json
{"ip":"100.73.175.108","user":"jsy","build_root":"/home/jsy/build",
 "sccache_dir":"/home/jsy/.cache/sccache","prefer":"always"}
```

`prefer: always` routes cold and incremental builds alike to RedBaron. No token,
no `.env` is needed.

## Commands

```sh
SK=~/.claude/skills/cloudrustbuild/cloudbuild.sh
bash "$SK" build <crate> [-- <cargo args>]   # sync → cargo build on RedBaron → pull target/
bash "$SK" test  <crate> [-- <cargo args>]   # sync → cargo test on RedBaron
bash "$SK" status                            # RedBaron reachable? sccache stats
bash "$SK" doctor                            # toolchain check on RedBaron
bash "$SK" route <crate>                     # print the routing decision without building
bash "$SK" sync  <crate>                     # sync only
bash "$SK" ssh [cmd]                         # shell on RedBaron
```

`<crate>` is an absolute path or a bare name resolved under `~/wintermute/`.

## How /build uses it

Branch agents on carbon and ryzen7 export `AUTOBUILDER_CLOUD=1` so /rustbuild's
`cargo-cloud` shim routes every cargo invocation through `cloudbuild.sh build`.
There is no session to start or tear down. If `status` reports RedBaron is down,
the branch marks its PRD blocked with "RedBaron unreachable" and stops.
