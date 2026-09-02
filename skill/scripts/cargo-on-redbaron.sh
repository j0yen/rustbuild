#!/usr/bin/env bash
# cloudbuild.sh — run cargo for a crate on RedBaron, the fleet's Rust build machine
# (not Wintermute Hub, the Hetzner NATS box). 'hub'/hub.json below is this
# script's legacy name for the standing build machine.
# The Hetzner burst path was retired on 2026-09-01; every burst entry point
# below now errors or no-ops. Hub config: ~/.config/wm-burst/hub.json.
#
# Lifecycle: warm builds go to the standing hub (harbor-hub) when reachable;
# cold/full compiles burst a ccx53 pod. Destroy-only is the explicit --ephemeral
# opt-out, not the default.
#
# Config + secrets come from ~/.config/wm-burst/.env:
#   HCLOUD_TOKEN   (required)  Hetzner Cloud API token
#   SNAPSHOT_ID    (required)  provisioned image: rustc 1.85+1.88 + sccache
#   BUILDER_TYPE   (opt, ccx33)  server type
#   BUILDER_LOC    (opt, fsn1)   location
#   SSH_KEY_NAME   (opt, wintermute-build)  uploaded key name
#   SSH_KEY        (opt, ~/.ssh/id_ed25519)  local private key
#
# Hub config (optional — harbor-route):
#   ~/.config/wm-burst/hub.json   { "ip": "<hub-ip>", "id": "<server-id>" }
#   ~/.config/wm-burst/cache.env  SCCACHE_* vars for the shared cache
#
# Subcommands:
#   up                 create server from snapshot, wait until SSH+toolchain ready
#   status             show running builder (if any) + hub status + accrued cost estimate
#   build <crate> [-- <cargo args>]   cargo build on RedBaron, pull target/ back
#   cargo <crate> -- <args...>        any cargo command on RedBaron (used by skill/bin/cargo)
#   test  <crate> [-- <cargo args>]   hub-aware test run
#   route <crate> [--dry-run]         print routing decision without doing work
#   fleet [--type cpx41] [--max N] <crate>... [-- <cargo args>]
#                      PARALLEL: one ephemeral cpx box per crate, concurrent,
#                      each torn down; self-limits to the account's core cap
#   keep-build <crate> [-- ...]       build but DO NOT destroy (reuse the box)
#   sync  <crate>      rsync a crate up (no build)
#   ssh   [cmd]        ssh into the running builder
#   down               destroy the running BURST builder (stops billing; hub not touched)
#   doctor             verify toolchain/arch on the running builder
#
# Routing flags (build/test):
#   --hub         force warm-hub path; error clearly if hub unreachable
#   --ephemeral   force old create→build→destroy burst path (ignores hub)
#
# Safety: build/test trap-destroy on exit for burst pods. Hub is NEVER destroyed
# by this script — use `wm-burst pod down` or Hetzner console for that.
set -uo pipefail

ENV_FILE="${WM_BURST_ENV:-$HOME/.config/wm-burst/.env}"
if [ -f "$ENV_FILE" ]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
elif [ ! -f "$HOME/.config/wm-burst/hub.json" ]; then
  echo "cloudbuild: missing $ENV_FILE (run setup / wm-burst init)" >&2; exit 1
fi
# HCLOUD_TOKEN is only needed for burst (Hetzner) paths; hub-only machines may omit .env.
HCLOUD_TOKEN="${HCLOUD_TOKEN:-}"
BUILDER_TYPE="${BUILDER_TYPE:-ccx33}"
BUILDER_LOC="${BUILDER_LOC:-fsn1}"
SSH_KEY_NAME="${SSH_KEY_NAME:-wintermute-build}"
SSH_KEY="${SSH_KEY:-$HOME/.ssh/id_ed25519}"
NAME="wintermute-builder"
API="https://api.hetzner.cloud/v1"
STATE_DIR="$HOME/.config/wm-burst"
COST_LOG="$STATE_DIR/cost.log"

HUB_JSON="${STATE_DIR}/hub.json"
# hub.json fields (harbor-route): ip (required), user (default root),
# build_root (default /root/build), sccache_dir (default /root/.sccache),
# prefer ("incremental" = classic heuristic, "always" = hub whenever reachable).
hub_field(){ # hub_field <key> <default>
  [ -f "$HUB_JSON" ] || { echo "$2"; return; }
  python3 -c "import json,sys; d=json.load(open('$HUB_JSON')); print(d.get(sys.argv[1]) or sys.argv[2])" "$1" "$2" 2>/dev/null || echo "$2"
}
HUB_USER="$(hub_field user root)"
HUB_BUILD_ROOT="$(hub_field build_root /root/build)"
HUB_SCCACHE_DIR="$(hub_field sccache_dir /root/.sccache)"
HUB_PREFER="$(hub_field prefer incremental)"
CACHE_ENV="${STATE_DIR}/cache.env"
HARBOR_CACHE_CLIENT="${HOME}/wintermute/constellation-burst-builder/scripts/harbor-cache-client-env.sh"
SESSION_LOCK="${STATE_DIR}/session.lock"

log(){ echo "[cloudbuild $(date +%H:%M:%S)] $*" >&2; }
api(){ # api METHOD PATH [json]
  local method="$1" path="$2" body="${3:-}"
  [ -n "$HCLOUD_TOKEN" ] || { echo "cloudbuild: HCLOUD_TOKEN not set (no $ENV_FILE) — burst path unavailable, only the hub route works on this machine" >&2; return 1; }
  if [ -n "$body" ]; then
    curl -fsS -X "$method" -H "Authorization: Bearer $HCLOUD_TOKEN" \
      -H "Content-Type: application/json" -d "$body" "$API$path"
  else
    curl -fsS -X "$method" -H "Authorization: Bearer $HCLOUD_TOKEN" "$API$path"
  fi
}
jqget(){ python3 -c 'import json,sys;d=json.load(sys.stdin);print(eval(sys.argv[1]))' "$1"; }

server_json(){ api GET "/servers?name=$NAME" 2>/dev/null; }
server_id(){ server_json | python3 -c 'import json,sys;s=json.load(sys.stdin)["servers"];print(s[0]["id"] if s else "")'; }
server_ip(){ server_json | python3 -c 'import json,sys;s=json.load(sys.stdin)["servers"];print(s[0]["public_net"]["ipv4"]["ip"] if s else "")'; }

# Ephemeral boxes reuse Hetzner IPs, so do NOT persist/check host keys (a recycled
# IP with a new host key would otherwise abort SSH). Throwaway known_hosts.
SSH_CMD="ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -o ConnectTimeout=8 -o ServerAliveInterval=30 -o ServerAliveCountMax=60 -i $SSH_KEY -o BatchMode=yes -o StrictHostKeyChecking=accept-new"
ssh_box(){ $SSH_CMD "root@$1" "${@:2}"; }
ssh_hub(){ $SSH_CMD "$HUB_USER@$1" "${@:2}"; }

hourly_rate(){ # for BUILDER_TYPE at BUILDER_LOC
  api GET "/server_types?per_page=50" | python3 -c '
import json,sys
d=json.load(sys.stdin); t=sys.argv[1]; loc=sys.argv[2]
for s in d["server_types"]:
    if s["name"]==t:
        for p in s["prices"]:
            if p["location"]==loc: print(p["price_hourly"]["net"]); break
        break' "$BUILDER_TYPE" "$BUILDER_LOC"
}

cmd_up(){
  local id ip
  id="$(server_id)"
  if [ -n "$id" ]; then ip="$(server_ip)"; log "builder already up: id=$id ip=$ip"; echo "$ip"; return 0; fi
  : "${SNAPSHOT_ID:?cloudbuild: SNAPSHOT_ID not set; provision a box and snapshot it first}"
  log "creating $BUILDER_TYPE from snapshot $SNAPSHOT_ID in $BUILDER_LOC ..."
  local body resp
  body=$(python3 -c 'import json,sys;print(json.dumps({"name":sys.argv[1],"server_type":sys.argv[2],"image":int(sys.argv[3]),"location":sys.argv[4],"ssh_keys":[sys.argv[5]],"labels":{"role":"rust-builder","project":"wintermute"}}))' \
    "$NAME" "$BUILDER_TYPE" "$SNAPSHOT_ID" "$BUILDER_LOC" "$SSH_KEY_NAME")
  resp=$(api POST "/servers" "$body") || { log "create failed"; return 1; }
  ip=$(echo "$resp" | python3 -c 'import json,sys;print(json.load(sys.stdin)["server"]["public_net"]["ipv4"]["ip"])')
  id=$(echo "$resp" | python3 -c 'import json,sys;print(json.load(sys.stdin)["server"]["id"])')
  echo "$(date -Iseconds) UP id=$id type=$BUILDER_TYPE ip=$ip" >> "$COST_LOG"
  log "server id=$id ip=$ip — waiting for SSH ..."
  local i
  for i in $(seq 1 30); do
    if timeout 4 bash -c "cat < /dev/null > /dev/tcp/$ip/22" 2>/dev/null; then break; fi
    sleep 4
  done
  for i in $(seq 1 20); do
    if ssh_box "$ip" 'test -x /root/.cargo/bin/rustc' 2>/dev/null; then
      log "ready ✓ ($(ssh_box "$ip" '. ~/.cargo/env; rustc --version' 2>/dev/null))"; echo "$ip"; return 0
    fi
    sleep 4
  done
  log "WARN: SSH/toolchain not confirmed; server is up at $ip — check manually"; echo "$ip"
}

cmd_down(){
  local id; id="$(server_id)"
  [ -z "$id" ] && { log "no builder running"; return 0; }
  log "destroying server $id (stops billing) ..."
  api DELETE "/servers/$id" >/dev/null && {
    echo "$(date -Iseconds) DOWN id=$id" >> "$COST_LOG"; log "destroyed ✓ billing stopped"
  } || log "delete failed — CHECK CONSOLE, server may still bill"
}

# ─── session warm mode ──────────────────────────────────────────────────────
# session-start: bring up a server once; write SESSION_LOCK so subsequent
# build/test calls reuse it instead of creating and destroying per build.
# session-end: tear down and remove the lock.
# The watchdog (max 2h default) is the safety net if session-end is never called.

session_read_ip(){
  [ -f "$SESSION_LOCK" ] || { echo ""; return; }
  python3 -c "import json; d=json.load(open('$SESSION_LOCK')); print(d.get('ip',''))" 2>/dev/null || echo ""
}

session_active(){
  local ip; ip="$(session_read_ip)"
  [ -n "$ip" ] && hub_reachable "$ip"
}

cmd_session_start(){
  local ttl="${1:-4}"
  local ip; ip="$(cmd_up)" || return 1
  local id; id="$(server_id)"
  python3 -c "
import json, sys, datetime
d = {'ip': sys.argv[1], 'id': sys.argv[2],
     'started': datetime.datetime.now(datetime.timezone.utc).isoformat(),
     'ttl_hours': int(sys.argv[3])}
print(json.dumps(d))
" "$ip" "$id" "$ttl" > "$SESSION_LOCK"
  log "session started: builder @ $ip warm (ttl=${ttl}h) — builds reuse this server until 'session-end'"
  echo "$ip"
}

cmd_session_end(){
  rm -f "$SESSION_LOCK"
  cmd_down
  log "session ended"
}

cmd_status(){
  local id ip; id="$(server_id)"
  if [ -z "$id" ]; then echo "burst builder: none running"; else
    ip="$(server_ip)"
    local rate; rate="$(hourly_rate)"
    echo "burst builder: id=$id ip=$ip type=$BUILDER_TYPE rate=€${rate}/hr (RUNNING — bills until 'down')"
  fi
  if [ -f "$SESSION_LOCK" ]; then
    local sip; sip="$(session_read_ip)"
    if session_active; then
      echo "session: WARM @ $sip — builds reuse this server (run 'session-end' to tear down)"
    else
      echo "session: lock exists ($(cat "$SESSION_LOCK" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('started','?'))")) but server UNREACHABLE — removing stale lock"
      rm -f "$SESSION_LOCK"
    fi
  else
    echo "session: none (run 'session-start' to warm a server for a build batch)"
  fi
  # Hub status (harbor-route)
  local hub_ip_val; hub_ip_val="$(hub_ip)"
  if [ -n "$hub_ip_val" ]; then
    if hub_reachable "$hub_ip_val"; then
      echo "hub: $HUB_USER@$hub_ip_val (reachable, prefer=$HUB_PREFER, build_root=$HUB_BUILD_ROOT)"
    else
      echo "hub: $hub_ip_val (UNREACHABLE — SSH timeout)"
    fi
  else
    echo "hub: not configured (no hub.json — run harbor-hub setup to enable warm builds)"
  fi
  [ -f "$COST_LOG" ] && { echo "--- recent lifecycle ---"; tail -5 "$COST_LOG"; }
}

cmd_doctor(){
  local ip; ip="$(server_ip)"; [ -z "$ip" ] && { echo "no builder running; 'up' first" >&2; return 1; }
  ssh_box "$ip" '. ~/.cargo/env; echo "arch: $(uname -m)"; rustc --version; rustup toolchain list; sccache --version 2>/dev/null || echo "sccache: MISSING"'
}

do_sync(){ # do_sync <ip> <crate_path> [user] [build_root]
  local ip="$1" crate="$2" user="${3:-root}" root="${4:-/root/build}" base; base="$(basename "$crate")"
  $SSH_CMD "$user@$ip" "mkdir -p $root/$base"
  rsync -az --delete --exclude target --exclude .git -e "$SSH_CMD" \
    "$crate/" "$user@$ip:$root/$base/"
  echo "$base"
}

cmd_sync(){ local ip; ip="$(cmd_up)"; do_sync "$ip" "$(resolve_crate "$1")" >/dev/null && log "synced $1"; }

resolve_crate(){ # accept a path or a bare crate name under ~/wintermute
  local c="$1"
  [ -d "$c" ] && { echo "$(cd "$c" && pwd)"; return; }
  [ -d "$HOME/wintermute/$c" ] && { echo "$HOME/wintermute/$c"; return; }
  echo "$c"
}

# ─── hub routing (harbor-route) ──────────────────────────────────────────────

hub_ip(){ # read ip from hub.json; empty string if absent
  [ -f "$HUB_JSON" ] || { echo ""; return; }
  python3 -c "import json,sys; d=json.load(open('$HUB_JSON')); print(d.get('ip',''))" 2>/dev/null || true
}

hub_reachable(){ # hub_reachable <ip>; rc 0 = up, rc 1 = down
  local ip="$1"
  [ -n "$ip" ] || return 1
  timeout 4 bash -c "cat </dev/null >/dev/tcp/$ip/22" 2>/dev/null
}

is_incremental(){ # is_incremental <crate_path>; rc 0 = likely incremental
  # Heuristic: target/ dir exists and was modified in the last 7 days.
  local crate="$1"
  [ -d "$crate/target" ] && find "$crate/target" -maxdepth 1 -newer "$crate/Cargo.toml" -name "*.d" 2>/dev/null | grep -q .
}

# route_decision — echoes "hub <ip>" or "burst" and a reason.
# Sets global ROUTE_DEST=hub|burst, ROUTE_IP (if hub).
route_decision(){ # route_decision <crate_path> [--force-hub] [--force-burst]
  local crate="$1" force_hub=0 force_burst=0
  shift
  while [ $# -gt 0 ]; do
    case "$1" in --force-hub) force_hub=1; shift;; --force-burst) force_burst=1; shift;; *) shift;; esac
  done

  ROUTE_DEST=burst; ROUTE_IP=""

  if [ $force_burst -eq 1 ]; then
    echo "ERROR: --ephemeral requested, but the Hetzner burst box was retired on 2026-09-01; builds run on RedBaron (the fleet's Rust build machine) only." >&2; return 2
  fi

  local ip; ip="$(hub_ip)"
  if [ -z "$ip" ]; then
    echo "ERROR: no hub.json at $HUB_JSON and the Hetzner burst box was retired (2026-09-01). Write hub.json pointing at RedBaron (hub.json is this script's legacy name for the build-machine config; it is not Wintermute Hub)." >&2; return 2
  fi

  if [ $force_hub -eq 1 ]; then
    if hub_reachable "$ip"; then
      ROUTE_DEST=hub; ROUTE_IP="$ip"
      echo "warm hub @ $ip (--hub forced)"; return
    else
      echo "ERROR: --hub forced but hub $ip is unreachable" >&2
      return 1
    fi
  fi

  if ! hub_reachable "$ip"; then
    if [ "$HUB_PREFER" = "always" ]; then
      echo "ERROR: hub $HUB_USER@$ip is unreachable and hub.json has prefer=always — refusing to burst to Hetzner. Bring the hub up, or pass --ephemeral to burst explicitly." >&2
      return 1
    fi
    echo "ERROR: RedBaron ($HUB_USER@$ip) unreachable (SSH timeout) and the Hetzner burst box was retired (2026-09-01) — bring RedBaron up." >&2; return 2
  fi

  if [ "$HUB_PREFER" = "always" ]; then
    ROUTE_DEST=hub; ROUTE_IP="$ip"
    echo "warm hub @ $ip (prefer=always in hub.json)"; return
  fi

  if is_incremental "$crate"; then
    ROUTE_DEST=hub; ROUTE_IP="$ip"
    echo "warm hub @ $ip (incremental — target/ dir exists)"; return
  fi

  echo "warm hub $HUB_USER@$ip = RedBaron (cold/full compile also builds there; the Hetzner burst box was retired 2026-09-01)"; ROUTE_DEST=hub; ROUTE_IP="$ip"; return
}

cmd_route(){ # route <crate> [--dry-run]
  local crate_arg="${1:-}"; shift || true
  local dry_run=0
  [ "${1:-}" = "--dry-run" ] && dry_run=1
  [ -n "$crate_arg" ] || { echo "usage: cloudbuild route <crate> [--dry-run]" >&2; return 2; }
  local crate; crate="$(resolve_crate "$crate_arg")"
  local decision; decision="$(route_decision "$crate")" || return $?
  echo "route: $decision"
  [ $dry_run -eq 1 ] && return 0
  return 0
}

# Load harbor-cache SCCACHE_* env block for burst pods when hub+cache available.
_maybe_export_cache_env(){
  if [ -f "$HUB_JSON" ] && [ -f "$CACHE_ENV" ]; then
    log "exporting harbor-cache SCCACHE_* block from $CACHE_ENV"
    # shellcheck disable=SC1090
    source "$CACHE_ENV" 2>/dev/null && return 0
    log "WARN: failed to source $CACHE_ENV — proceeding without shared cache"
  elif [ -f "$HUB_JSON" ] && [ -x "$HARBOR_CACHE_CLIENT" ]; then
    local env_output; env_output="$("$HARBOR_CACHE_CLIENT" 2>/dev/null)" || true
    [ -n "$env_output" ] && eval "$env_output"
  fi
}

_build_or_test_hub(){ # _build_or_test_hub <build|test> <hub_ip> <crate> [cargo args...]
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

_build_or_test(){ # _build_or_test <build|test> <keep?> <crate> [cargo args...]
  # Flags --hub and --ephemeral must precede <crate>.
  local action="$1" keep="$2"; shift 2
  local force_hub=0 force_burst=0
  while [ $# -gt 0 ]; do
    case "$1" in
      --hub)       force_hub=1;   shift;;
      --ephemeral) force_burst=1; shift;;
      *) break;;
    esac
  done
  local crate; crate="$(resolve_crate "$1")"; shift
  [ "${1:-}" = "--" ] && shift   # drop the CLI separator; the rest are cargo args
  [ -d "$crate" ] || { echo "cloudbuild: crate not found: $crate" >&2; return 1; }

  # Determine route.
  # Initialize ROUTE_DEST/ROUTE_IP before the subshell call so set -u doesn't
  # fire when route_decision returns "burst" (the subshell can't export to us).
  ROUTE_DEST=burst; ROUTE_IP=""
  local decision; decision="$(route_decision "$crate" \
    $([ $force_hub -eq 1 ] && echo --force-hub) \
    $([ $force_burst -eq 1 ] && echo --force-burst))" || return $?
  log "routing: $decision"
  # Re-parse ROUTE_DEST from the decision string since route_decision runs in a subshell.
  case "$decision" in
    "warm hub"*) ROUTE_DEST=hub; ROUTE_IP="$(echo "$decision" | grep -oP '(?<=@ )[0-9.]+' || true)";;
    *) ROUTE_DEST=burst; ROUTE_IP="";;
  esac

  if [ "$ROUTE_DEST" = "hub" ]; then
    # Warm hub path — no create/destroy.
    _build_or_test_hub "$action" "$ROUTE_IP" "$crate" "$@"
    return $?
  fi

  echo "ERROR: no route to RedBaron ($decision); the Hetzner burst path was retired 2026-09-01." >&2
  return 2
}

cmd_build(){ _build_or_test build destroy "$@"; }
cmd_test(){ _build_or_test test destroy "$@"; }
cmd_keepbuild(){ _build_or_test build keep "$@"; }

cmd_ssh(){ local ip; ip="$(server_ip)"; [ -z "$ip" ] && { echo "no builder running" >&2; return 1; }; ssh_box "$ip" "$@"; }

# ─── parallel fleet ─────────────────────────────────────────────────────────
# Build N crates concurrently, each on its own ephemeral cpx (SHARED-vCPU) box.
# Shared vCPU dodges the dedicated_core_limit that caps ccx fleets to ~1 box and
# is ~half the price. Each worker tears down its own box; a global EXIT trap
# reaps ANY leftover fleet box (label fleet=1) as a backstop. Concurrency is
# capped by --max AND self-limited at runtime: a worker that hits
# resource_limit_exceeded waits and retries, so the fleet auto-fits the account.
# cpx42: 8 shared vCPU, 16G, 320G disk, €0.048/hr. MUST be the "x2" generation —
# x1 types (cpx41 etc.) are phased out ("unsupported location"), and cpx22/cpx32
# have <240G disks so the ccx33-derived snapshot won't deploy. cpx42 is the cheap floor.
FLEET_TYPE_DEFAULT="cpx42"
FLEET_MAX_DEFAULT=5            # soft cost/concurrency cap
FLEET_PREFIX="wintermute-fleet"

create_named(){ # create_named <name> <type> -> echoes "id ip"; rc 2 = limit-exceeded
  local name="$1" stype="$2" body tmpf code
  body=$(python3 -c 'import json,sys;print(json.dumps({"name":sys.argv[1],"server_type":sys.argv[2],"image":int(sys.argv[3]),"location":sys.argv[4],"ssh_keys":[sys.argv[5]],"labels":{"role":"rust-builder","fleet":"1"}}))' \
    "$name" "$stype" "$SNAPSHOT_ID" "$BUILDER_LOC" "$SSH_KEY_NAME")
  tmpf=$(mktemp)
  code=$(curl -s -o "$tmpf" -w '%{http_code}' -X POST -H "Authorization: Bearer $HCLOUD_TOKEN" \
    -H "Content-Type: application/json" -d "$body" "$API/servers")
  if [ "$code" = "201" ]; then
    python3 -c 'import json;d=json.load(open("'"$tmpf"'"));print(d["server"]["id"],d["server"]["public_net"]["ipv4"]["ip"])'
    rm -f "$tmpf"; return 0
  fi
  if grep -q resource_limit_exceeded "$tmpf" 2>/dev/null; then rm -f "$tmpf"; return 2; fi
  log "[fleet] create '$name' failed (HTTP $code): $(head -c160 "$tmpf")"; rm -f "$tmpf"; return 1
}

destroy_by_name(){ local id; id=$(api GET "/servers?name=$1" 2>/dev/null | python3 -c 'import json,sys;s=json.load(sys.stdin)["servers"];print(s[0]["id"] if s else "")'); [ -n "$id" ] && api DELETE "/servers/$id" >/dev/null 2>&1; }

destroy_all_fleet(){ # backstop: reap every box labelled fleet=1
  api GET "/servers?label_selector=fleet%3D1" 2>/dev/null \
    | python3 -c 'import json,sys
for s in json.load(sys.stdin)["servers"]: print(s["id"])' 2>/dev/null \
    | while read -r id; do [ -n "$id" ] && api DELETE "/servers/$id" >/dev/null 2>&1 && log "[fleet] reaped leftover server $id"; done
}

fleet_worker(){ # fleet_worker <idx> <crate> <type> <results_dir> [cargo args...]
  local idx="$1" crate="$2" stype="$3" rdir="$4"; shift 4
  local name="$FLEET_PREFIX-$idx" base; base="$(basename "$crate")"
  local idip id ip tries=0
  while :; do
    if idip=$(create_named "$name" "$stype"); then id=${idip%% *}; ip=${idip##* }; break; fi
    [ $? -eq 2 ] || { echo "$base CREATE-FAIL 0" >"$rdir/$idx"; return 1; }
    tries=$((tries+1)); [ $tries -gt 90 ] && { echo "$base LIMIT-TIMEOUT 0" >"$rdir/$idx"; return 1; }
    sleep 10   # account limit hit; wait for a sibling to free a slot
  done
  trap "destroy_by_name '$name'" EXIT   # this subshell owns this box
  log "[fleet $idx] $base -> $name ($stype) ip=$ip"
  local i; for i in $(seq 1 40); do timeout 4 bash -c "cat </dev/null >/dev/tcp/$ip/22" 2>/dev/null && break; sleep 4; done
  for i in $(seq 1 20); do ssh_box "$ip" 'test -x /root/.cargo/bin/rustc' 2>/dev/null && break; sleep 4; done
  do_sync "$ip" "$crate" >/dev/null 2>&1
  local start rc; start=$(date +%s)
  ssh_box "$ip" ". ~/.cargo/env; export RUSTC_WRAPPER=sccache SCCACHE_DIR=/root/.sccache; cd /root/build/$base; cargo build $*" >"$rdir/$idx.log" 2>&1
  rc=$?
  local dur=$(($(date +%s)-start))
  if [ $rc -eq 0 ]; then
    rsync -az -e "$SSH_CMD" "root@$ip:/root/build/$base/target/" "$crate/target/" 2>/dev/null
    echo "$base OK $dur" >"$rdir/$idx"
  else
    echo "$base FAIL $dur" >"$rdir/$idx"
  fi
}

cmd_fleet(){
  local stype="${FLEET_TYPE:-$FLEET_TYPE_DEFAULT}" max="${FLEET_MAX:-$FLEET_MAX_DEFAULT}"
  local cargs=() crates=()
  while [ $# -gt 0 ]; do case "$1" in
    --type) stype="$2"; shift 2;;
    --max)  max="$2";  shift 2;;
    --) shift; cargs=("$@"); break;;
    *) crates+=("$1"); shift;;
  esac; done
  [ ${#crates[@]} -gt 0 ] || { echo "usage: cloudbuild fleet [--type cpx41] [--max N] <crate>... [-- <cargo args>]" >&2; return 2; }
  : "${SNAPSHOT_ID:?cloudbuild: SNAPSHOT_ID not set}"
  local rdir; rdir="$(mktemp -d)"
  # backstop reap; ${rdir:-} guard so the EXIT trap can't trip set -u once the
  # local has gone out of scope after cmd_fleet returns.
  trap 'destroy_all_fleet; rm -rf "${rdir:-}"' EXIT INT TERM
  log "fleet: ${#crates[@]} crate(s) on $stype, max $max concurrent (cargo build ${cargs[*]:-})"
  local idx=0 running=0
  for c in "${crates[@]}"; do
    local cp; cp="$(resolve_crate "$c")"
    [ -d "$cp" ] || { echo "skip (not found): $c" >&2; echo "$c NOTFOUND 0" >"$rdir/$idx"; idx=$((idx+1)); continue; }
    ( fleet_worker "$idx" "$cp" "$stype" "$rdir" "${cargs[@]}" ) &
    idx=$((idx+1)); running=$((running+1))
    [ $running -ge $max ] && { wait -n 2>/dev/null || wait; running=$((running-1)); }
  done
  wait
  echo "=== fleet results ==="
  local ok=0 fail=0 r
  for r in $(seq 0 $((idx-1))); do
    [ -f "$rdir/$r" ] || { echo "  [#$r] (no result)"; continue; }
    read -r cr st du <"$rdir/$r"
    printf "  %-32s %-12s %ss\n" "$cr" "$st" "$du"
    [ "$st" = OK ] && ok=$((ok+1)) || fail=$((fail+1))
  done
  echo "=== $ok ok, $fail failed ==="
  destroy_all_fleet                       # explicit reap
  rm -rf "$rdir"; trap - EXIT INT TERM     # clean exit: disarm the backstop
  [ $fail -eq 0 ]
}

main(){
  local sub="${1:-status}"; shift || true
  case "$sub" in
    up|down|keep-build|fleet) echo "cloudbuild: '$1' is retired — the Hetzner burst box is no longer used (2026-09-01); builds run on RedBaron (build/test/status/doctor/sync/route/ssh)." >&2; exit 2 ;;
    session-start|session-end) echo "cloudbuild: '$1' is a no-op — no burst sessions since 2026-09-01; RedBaron is always on." ;;
    status) cmd_status ;;
    doctor) cmd_doctor ;;
    sync) cmd_sync "$@" ;;
    build) cmd_build "$@" ;;
    test) cmd_test "$@" ;;
    cargo) _build_or_test cargo nokeep "$@" ;;
    route) cmd_route "$@" ;;
    ssh) cmd_ssh "$@" ;;
    -h|--help|help) sed -n '2,55p' "$0" | sed 's/^# \?//' ;;
    *) echo "cloudbuild: unknown subcommand '$sub' (try --help)" >&2; exit 2 ;;
  esac
}
main "$@"
