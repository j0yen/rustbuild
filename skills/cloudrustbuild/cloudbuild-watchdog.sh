#!/usr/bin/env bash
# cloudbuild-watchdog.sh — kill orphaned burst builders older than MAX_AGE_HOURS.
#
# Runs as a systemd timer (every 30 min). Destroys any Hetzner server labelled
# role=rust-builder that has been running longer than MAX_AGE_HOURS (default 2).
# The hub (labelled role=harbor-hub or unlabelled monthly) is never touched.
#
# Usage: cloudbuild-watchdog.sh [--max-age-hours N] [--dry-run]
set -uo pipefail

ENV_FILE="${WM_BURST_ENV:-$HOME/.config/wm-burst/.env}"
[ -f "$ENV_FILE" ] || { echo "watchdog: missing $ENV_FILE" >&2; exit 1; }
# shellcheck disable=SC1090
source "$ENV_FILE"
: "${HCLOUD_TOKEN:?watchdog: HCLOUD_TOKEN not set}"

MAX_AGE_HOURS=2
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --max-age-hours) MAX_AGE_HOURS="$2"; shift 2 ;;
    --dry-run)       DRY_RUN=1; shift ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

api_get() { curl -sf -H "Authorization: Bearer $HCLOUD_TOKEN" "https://api.hetzner.cloud/v1$1"; }
api_delete() { curl -sf -X DELETE -H "Authorization: Bearer $HCLOUD_TOKEN" "https://api.hetzner.cloud/v1$1"; }

ts() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }
log() { echo "$(ts) [cloudbuild-watchdog] $*"; }

# Fetch all rust-builder labelled servers
servers=$(api_get "/servers?label_selector=role%3Drust-builder&per_page=50")
if [[ -z "$servers" ]]; then
  log "ERROR: failed to query Hetzner API"
  exit 1
fi

count=$(echo "$servers" | python3 -c "import json,sys; print(len(json.load(sys.stdin)['servers']))")
log "found $count rust-builder server(s)"

if [[ "$count" -eq 0 ]]; then
  log "nothing to do"
  exit 0
fi

now_epoch=$(date -u +%s)
max_age_secs=$(( MAX_AGE_HOURS * 3600 ))
killed=0

while IFS= read -r line; do
  server_id=$(echo "$line" | cut -d'|' -f1)
  server_name=$(echo "$line" | cut -d'|' -f2)
  server_type=$(echo "$line" | cut -d'|' -f3)
  created=$(echo "$line" | cut -d'|' -f4)
  status=$(echo "$line" | cut -d'|' -f5)

  # Parse created timestamp to epoch
  created_epoch=$(date -u -d "$created" +%s 2>/dev/null || python3 -c "
import sys
from datetime import datetime, timezone
ts = '$created'.replace('Z','+00:00')
dt = datetime.fromisoformat(ts)
print(int(dt.timestamp()))
")
  age_secs=$(( now_epoch - created_epoch ))
  age_hours=$(python3 -c "print(f'{$age_secs/3600:.1f}')")

  if [[ $age_secs -gt $max_age_secs ]]; then
    if [[ $DRY_RUN -eq 1 ]]; then
      log "DRY-RUN: would destroy $server_id ($server_name, $server_type, age=${age_hours}h, status=$status)"
    else
      log "destroying $server_id ($server_name, $server_type, age=${age_hours}h, status=$status) — ORPHAN"
      result=$(api_delete "/servers/$server_id")
      if echo "$result" | python3 -c "import json,sys; d=json.load(sys.stdin); sys.exit(0 if 'action' in d else 1)" 2>/dev/null; then
        log "destroyed $server_id ✓ billing stopped"
        killed=$(( killed + 1 ))
      else
        log "ERROR: failed to destroy $server_id — check Hetzner console"
      fi
    fi
  else
    log "ok: $server_id ($server_name, age=${age_hours}h) — within ${MAX_AGE_HOURS}h limit"
  fi
done < <(echo "$servers" | python3 -c "
import json, sys
for s in json.load(sys.stdin)['servers']:
    print(f\"{s['id']}|{s['name']}|{s['server_type']['name']}|{s['created']}|{s['status']}\")
")

log "done — killed=$killed dry_run=$DRY_RUN"
