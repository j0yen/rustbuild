#!/usr/bin/env bash
# cargo-on-redbaron.sh — run cargo for a crate on RedBaron, the fleet's Rust
# build machine (not Wintermute Hub, the Hetzner NATS box). 'hub'/hub.json
# below is this script's legacy name for RedBaron, the standing build machine.
# The Hetzner burst path was retired 2026-09-01; RedBaron is the only
# destination now.
#
# Config (required): ~/.config/wm-burst/hub.json
#   { "ip": "<redbaron-tailscale-ip>", "user": "jsy",
#     "build_root": "/path/to/build", "sccache_dir": "/path/to/.sccache" }
#   ip is required; user/build_root/sccache_dir default to
#   root//root/build//root/.sccache if omitted.
#
# Optional (same directory): .env sourced if present, e.g. for SSH_KEY
# (default ~/.ssh/id_ed25519); cache.env for SCCACHE_* vars for a shared cache.
#
# Subcommands:
#   status                            show RedBaron reachability
#   doctor                            verify toolchain/arch on RedBaron
#   route <crate> [--dry-run]         print routing decision without doing work
#   sync  <crate>                     rsync a crate up to RedBaron (no build)
#   build <crate> [-- <cargo args>]   cargo build on RedBaron, pull target/ back
#   test  <crate> [-- <cargo args>]   cargo test on RedBaron
#   cargo <crate> -- <args...>        any cargo command on RedBaron (used by skill/bin/cargo)
#   ssh   [cmd]                       ssh into RedBaron
#   up|down|keep-build|fleet|session-start|session-end
#                                      retired 2026-09-01; error stub only
#
# Routing: RedBaron via hub.json is the only destination. No hub.json, or
# RedBaron unreachable, is an error — never falls back, never builds locally.
set -uo pipefail

STATE_DIR="$HOME/.config/wm-burst"
ENV_FILE="${WM_BURST_ENV:-$STATE_DIR/.env}"
if [ -f "$ENV_FILE" ]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
fi
SSH_KEY="${SSH_KEY:-$HOME/.ssh/id_ed25519}"

HUB_JSON="${STATE_DIR}/hub.json"
# hub.json fields: ip (required), user (default root),
# build_root (default /root/build), sccache_dir (default /root/.sccache).
hub_field(){ # hub_field <key> <default>
  [ -f "$HUB_JSON" ] || { echo "$2"; return; }
  python3 -c "import json,sys; d=json.load(open('$HUB_JSON')); print(d.get(sys.argv[1]) or sys.argv[2])" "$1" "$2" 2>/dev/null || echo "$2"
}
HUB_USER="$(hub_field user root)"
HUB_BUILD_ROOT="$(hub_field build_root /root/build)"
HUB_SCCACHE_DIR="$(hub_field sccache_dir /root/.sccache)"
CACHE_ENV="${STATE_DIR}/cache.env"

log(){ echo "[cargo-on-redbaron $(date +%H:%M:%S)] $*" >&2; }

SSH_CMD="ssh -o ConnectTimeout=8 -o ServerAliveInterval=30 -o ServerAliveCountMax=60 -i $SSH_KEY -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o LogLevel=ERROR"
ssh_hub(){ $SSH_CMD "$HUB_USER@$1" "${@:2}"; }

do_sync(){ # do_sync <ip> <crate_path> [user] [build_root]
  local ip="$1" crate="$2" user="${3:-root}" root="${4:-/root/build}" base; base="$(basename "$crate")"
  $SSH_CMD "$user@$ip" "mkdir -p $root/$base"
  rsync -az --delete --exclude target --exclude .git -e "$SSH_CMD" \
    "$crate/" "$user@$ip:$root/$base/"
  echo "$base"
}

resolve_crate(){ # accept a path or a bare crate name under ~/wintermute
  local c="$1"
  [ -d "$c" ] && { echo "$(cd "$c" && pwd)"; return; }
  [ -d "$HOME/wintermute/$c" ] && { echo "$HOME/wintermute/$c"; return; }
  echo "$c"
}

# ─── hub routing ─────────────────────────────────────────────────────────────

hub_ip(){ # read ip from hub.json; empty string if absent
  [ -f "$HUB_JSON" ] || { echo ""; return; }
  python3 -c "import json,sys; d=json.load(open('$HUB_JSON')); print(d.get('ip',''))" 2>/dev/null || true
}

hub_reachable(){ # hub_reachable <ip>; rc 0 = up, rc 1 = down
  local ip="$1"
  [ -n "$ip" ] || return 1
  timeout 4 bash -c "cat </dev/null >/dev/tcp/$ip/22" 2>/dev/null
}

# route_decision — sets ROUTE_IP on success. Must be called directly (never
# via `$(...)`), since it communicates through a global, not stdout, and a
# command-substitution subshell can't export that back to the caller.
# rc 2 = no route (missing hub.json, or RedBaron unreachable) — never falls
# back and never builds locally.
route_decision(){
  ROUTE_IP=""
  local ip; ip="$(hub_ip)"
  if [ -z "$ip" ]; then
    echo "ERROR: no hub.json at $HUB_JSON and the Hetzner burst box was retired (2026-09-01). Write hub.json pointing at RedBaron (hub.json is this script's legacy name for the build-machine config; it is not Wintermute Hub)." >&2
    return 2
  fi
  if ! hub_reachable "$ip"; then
    echo "ERROR: RedBaron ($HUB_USER@$ip) unreachable (SSH timeout) — bring RedBaron up." >&2
    return 2
  fi
  ROUTE_IP="$ip"
}

cmd_route(){ # route <crate> [--dry-run]
  local crate_arg="${1:-}"; shift || true
  local dry_run=0
  [ "${1:-}" = "--dry-run" ] && dry_run=1
  [ -n "$crate_arg" ] || { echo "usage: cargo-on-redbaron route <crate> [--dry-run]" >&2; return 2; }
  route_decision || return $?
  echo "route: warm hub @ $ROUTE_IP"
  [ $dry_run -eq 1 ] && return 0
  return 0
}

cmd_status(){
  local hub_ip_val; hub_ip_val="$(hub_ip)"
  if [ -n "$hub_ip_val" ]; then
    if hub_reachable "$hub_ip_val"; then
      echo "hub: $HUB_USER@$hub_ip_val = RedBaron (reachable, build_root=$HUB_BUILD_ROOT)"
    else
      echo "hub: $hub_ip_val (UNREACHABLE — SSH timeout)"
    fi
  else
    echo "hub: not configured (no hub.json at $HUB_JSON)"
  fi
}

cmd_doctor(){
  local ip; ip="$(hub_ip)"
  [ -n "$ip" ] || { echo "cargo-on-redbaron: no hub.json at $HUB_JSON" >&2; return 1; }
  hub_reachable "$ip" || { echo "cargo-on-redbaron: RedBaron ($HUB_USER@$ip) unreachable" >&2; return 1; }
  ssh_hub "$ip" '. ~/.cargo/env 2>/dev/null; echo "arch: $(uname -m)"; rustc --version; rustup toolchain list; sccache --version 2>/dev/null || echo "sccache: MISSING"'
}

cmd_sync(){
  local crate; crate="$(resolve_crate "$1")"
  route_decision || return $?
  do_sync "$ROUTE_IP" "$crate" "$HUB_USER" "$HUB_BUILD_ROOT" >/dev/null && log "synced $1"
}

# Load harbor-cache SCCACHE_* env block when hub + shared cache config exist.
_maybe_export_cache_env(){
  if [ -f "$HUB_JSON" ] && [ -f "$CACHE_ENV" ]; then
    log "exporting harbor-cache SCCACHE_* block from $CACHE_ENV"
    # shellcheck disable=SC1090
    source "$CACHE_ENV" 2>/dev/null && return 0
    log "WARN: failed to source $CACHE_ENV — proceeding without shared cache"
  fi
}

_build_or_test_hub(){ # _build_or_test_hub <build|test|cargo> <hub_ip> <crate> [cargo args...]
  local action="$1" ip="$2" crate="$3"; shift 3
  [ "${1:-}" = "--" ] && shift
  local base; base="$(do_sync "$ip" "$crate" "$HUB_USER" "$HUB_BUILD_ROOT")"
  local cargocmd
  case "$action" in
    test)  cargocmd="cargo test $*";;
    cargo) cargocmd="cargo $*";;
    *)     cargocmd="cargo build $*";;
  esac
  log "RedBaron $HUB_USER@$ip: $cargocmd  (in $HUB_BUILD_ROOT/$base)"
  local start rc; start=$(date +%s)
  # Hub uses shared sccache; source cache.env if available for a shared S3-style bucket.
  local sccache_block=". ~/.cargo/env 2>/dev/null; export RUSTC_WRAPPER=sccache SCCACHE_DIR=$HUB_SCCACHE_DIR"
  ssh_hub "$ip" "$sccache_block; cd $HUB_BUILD_ROOT/$base; $cargocmd"
  rc=$?
  log "hub $action exit=$rc in $(($(date +%s)-start))s"
  ssh_hub "$ip" ". ~/.cargo/env 2>/dev/null; sccache --show-stats 2>/dev/null | grep -iE 'cache hits rate|compile requests executed'" 2>/dev/null || true
  if [ "$action" != "test" ] && [ $rc -eq 0 ]; then
    rsync -az -e "$SSH_CMD" \
      "$HUB_USER@$ip:$HUB_BUILD_ROOT/$base/target/" "$crate/target/" 2>/dev/null && log "pulled artifacts to $crate/target/" || log "(no artifacts pulled)"
  fi
  return $rc
}

_build_or_test(){ # _build_or_test <build|test|cargo> <crate> [-- <cargo args>]
  local action="$1"; shift
  local crate; crate="$(resolve_crate "$1")"; shift
  [ "${1:-}" = "--" ] && shift   # drop the CLI separator; the rest are cargo args
  [ -d "$crate" ] || { echo "cargo-on-redbaron: crate not found: $crate" >&2; return 1; }

  route_decision || return $?
  log "routing: warm hub @ $ROUTE_IP"
  _build_or_test_hub "$action" "$ROUTE_IP" "$crate" "$@"
}

cmd_build(){ _build_or_test build "$@"; }
cmd_test(){ _build_or_test test "$@"; }

cmd_ssh(){
  local ip; ip="$(hub_ip)"
  [ -n "$ip" ] || { echo "cargo-on-redbaron: no hub.json at $HUB_JSON" >&2; return 1; }
  hub_reachable "$ip" || { echo "cargo-on-redbaron: RedBaron ($HUB_USER@$ip) unreachable" >&2; return 1; }
  ssh_hub "$ip" "$@"
}

main(){
  local sub="${1:-status}"; shift || true
  case "$sub" in
    up|down|keep-build|fleet|session-start|session-end)
      echo "cargo-on-redbaron: '$sub' retired 2026-09-01; builds run on RedBaron" >&2; exit 2 ;;
    status) cmd_status ;;
    doctor) cmd_doctor ;;
    sync) cmd_sync "$@" ;;
    build) cmd_build "$@" ;;
    test) cmd_test "$@" ;;
    cargo) _build_or_test cargo "$@" ;;
    route) cmd_route "$@" ;;
    ssh) cmd_ssh "$@" ;;
    -h|--help|help) sed -n '2,30p' "$0" | sed 's/^# \?//' ;;
    *) echo "cargo-on-redbaron: unknown subcommand '$sub' (try --help)" >&2; exit 2 ;;
  esac
}
main "$@"
