---
name: cloudrustbuild
description: Run cargo build/test for a Rust crate on the fleet's Rust build hub (RedBaron) from any other node, over Tailscale SSH, with RedBaron's sccache and mold, and pull target/ back. Use from carbon or ryzen7 when a /rustbuild step or the user needs a compile; on RedBaron itself cargo runs locally. The Hetzner burst box was retired on 2026-09-01 — nothing here rents a server.
user_invocable: true
---

# /cloudrustbuild — cargo on the RedBaron hub, from any node

RedBaron is the fleet's Rust build machine (i7-11700KF, 16 threads, 30 GB,
sccache + mold). This skill lets carbon and ryzen7 use it: `cloudbuild.sh` syncs
the crate to `~/build/<crate>` on RedBaron, runs the cargo command there as `jsy`,
prints the sccache hit rate, and rsyncs `target/` back. All three machines are
Ubuntu 26.04 x86_64, so the binary runs where it was requested.

On RedBaron, do not use this skill for RedBaron's own builds; run `cargo` directly.

**Retired (2026-09-01):** the Hetzner burst box, snapshots, `session-start`/`-end`,
`up`/`down`/`keep-build`/`fleet`, the cost log and the watchdog. Those entry points
now error (or no-op for `session-*`) so a stale caller fails loudly instead of
renting a server. If RedBaron is unreachable the script exits 2; it never bursts
and never falls back to a local build on its own.

## Config

`~/.config/wm-burst/hub.json` on each client:

```json
{"ip":"100.73.175.108","user":"jsy","build_root":"/home/jsy/build",
 "sccache_dir":"/home/jsy/.cache/sccache","prefer":"always"}
```

`prefer: always` routes cold and incremental builds alike to the hub. No token,
no `.env` is needed.

## Commands

```sh
SK=~/.claude/skills/cloudrustbuild/cloudbuild.sh
bash "$SK" build <crate> [-- <cargo args>]   # sync → cargo build on RedBaron → pull target/
bash "$SK" test  <crate> [-- <cargo args>]   # sync → cargo test on RedBaron
bash "$SK" status                            # hub reachable? sccache stats
bash "$SK" doctor                            # toolchain check on the hub
bash "$SK" route <crate>                     # print the routing decision without building
bash "$SK" sync  <crate>                     # sync only
bash "$SK" ssh [cmd]                         # shell on the hub
```

`<crate>` is an absolute path or a bare name resolved under `~/wintermute/`.

## How /build uses it

Branch agents on carbon and ryzen7 export `AUTOBUILDER_CLOUD=1` so /rustbuild's
`cargo-cloud` shim routes every cargo invocation through `cloudbuild.sh build`.
There is no session to start or tear down. If `status` reports the hub is down,
the branch marks its PRD blocked with "build hub unreachable" and stops.
