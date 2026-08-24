#!/usr/bin/env bash
# ctrace-session-start-backfill — SessionStart sweep for orphaned ndjson logs.
#
# At session start, scan ~/.cache/ctrace/sessions/ for any *.ndjson that has
# no sibling *.summary.md (i.e. it was orphaned by a SIGKILL'd session).
# Render them via `scribe backfill` if available; otherwise use the shell
# summarizer in a bounded loop.
#
# Design constraints:
#   - Must never block session start: cheap stat walk on the common path.
#   - If backfill touches > BATCH_THRESHOLD logs, run detached (background).
#   - Always exits 0.
#
# Usage: called from a SessionStart hook in ~/.claude/settings.json
#   OR   tested standalone: ./ctrace-session-start-backfill.draft.sh
#
# PRD: PRD-ctrace-session-end-resilient.md

set -uo pipefail

SESSIONS_DIR="${CTRACE_SESSIONS_DIR:-/home/jsy/.cache/ctrace/sessions}"
SUMMARIZE="${CTRACE_SUMMARIZE:-/home/jsy/.claude/scripts/summarize-ctrace-session.sh}"
LOG_FILE="${CTRACE_BACKFILL_LOG:-/home/jsy/.cache/ctrace/backfill.log}"
# If more than this many orphans need rendering, run detached so we don't delay start.
BATCH_THRESHOLD="${CTRACE_BACKFILL_THRESHOLD:-3}"

# --- helpers -----------------------------------------------------------------

log_msg() {
    printf '%s  ctrace-backfill  %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >> "$LOG_FILE" 2>/dev/null || true
}

# Collect all *.ndjson files with no sibling *.summary.md
find_orphans() {
    local dir="$1"
    local orphans=()
    # Use a glob; fall back gracefully if dir is empty or missing.
    if [ ! -d "$dir" ]; then
        return 0
    fi
    for ndjson in "$dir"/*.ndjson; do
        [ -e "$ndjson" ] || continue               # glob expanded to literal if no match
        local base="${ndjson%.ndjson}"
        [ -f "${base}.summary.md" ] && continue    # already summarized
        orphans+=("$ndjson")
    done
    (( ${#orphans[@]} == 0 )) && return 0
    printf '%s\n' "${orphans[@]}"
}

# Render one log via scribe (if available) or the shell summarizer fallback.
# Returns 0 on success, non-zero on failure.
render_one() {
    local log_path="$1"
    local summary="${log_path%.ndjson}.summary.md"

    # Prefer scribe when available
    if command -v scribe >/dev/null 2>&1; then
        if scribe render "$log_path" -o "$summary" 2>/dev/null; then
            return 0
        fi
        # scribe failed — fall through to shell summarizer
        log_msg "WARN scribe render failed for $log_path; falling back to shell summarizer"
    fi

    # Shell summarizer fallback
    if [ -x "$SUMMARIZE" ]; then
        "$SUMMARIZE" "$log_path" >/dev/null 2>>"$LOG_FILE" && return 0
        log_msg "ERROR shell summarizer failed for $log_path (exit $?)"
        return 1
    fi

    log_msg "ERROR no renderer available for $log_path (scribe absent, summarize not executable)"
    return 1
}

# Render a list of orphan paths, logging success/failure per item.
render_list() {
    local rendered=0 failed=0
    local f
    while IFS= read -r f; do
        [ -z "$f" ] && continue
        if render_one "$f"; then
            log_msg "OK rendered $f"
            (( rendered++ )) || true
        else
            (( failed++ )) || true
        fi
    done
    log_msg "backfill complete: rendered=$rendered failed=$failed"
}

# --- main --------------------------------------------------------------------

main() {
    # Fast mtime stat-walk: collect orphans
    mapfile -t orphans < <(find_orphans "$SESSIONS_DIR")
    local count="${#orphans[@]}"

    if (( count == 0 )); then
        # Common path — nothing to do, no log noise
        exit 0
    fi

    log_msg "found $count orphaned ndjson(s); BATCH_THRESHOLD=$BATCH_THRESHOLD"

    if (( count > BATCH_THRESHOLD )); then
        # Detach so we don't delay session start: re-invoke this same script
        # with CTRACE_BACKFILL_THRESHOLD set very high so the detached copy
        # renders inline without spawning another generation.
        log_msg "count > threshold; running detached (PID TBD)"
        CTRACE_BACKFILL_THRESHOLD=99999 \
        CTRACE_SESSIONS_DIR="$SESSIONS_DIR" \
        CTRACE_SUMMARIZE="$SUMMARIZE" \
        CTRACE_BACKFILL_LOG="$LOG_FILE" \
            nohup bash "${BASH_SOURCE[0]}" >> "$LOG_FILE" 2>&1 &
        log_msg "detached backfill PID=$!"
    else
        # Small batch — run inline (still bounded)
        printf '%s\n' "${orphans[@]}" | render_list
    fi
}

main
exit 0
