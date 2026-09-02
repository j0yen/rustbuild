#!/usr/bin/env bash
# Tests for inoculate-preamble and inoculate-inject
set -uo pipefail

PASS=0
FAIL=0
ERRORS=()

pass() { printf '[PASS] %s\n' "$1"; ((PASS++)); }
fail() { printf '[FAIL] %s\n' "$1"; ((FAIL++)); ERRORS+=("$1"); }

PREAMBLE_BIN="$(command -v inoculate-preamble 2>/dev/null || echo '/home/jsy/.local/bin/inoculate-preamble')"
INJECT_BIN="$(command -v inoculate-inject 2>/dev/null || echo '/home/jsy/.local/bin/inoculate-inject')"

# --- Test 1: inoculate-preamble outputs non-empty string with strain_hash, exits 0
OUTPUT="$("$PREAMBLE_BIN")"
EXIT_CODE=$?
if [[ $EXIT_CODE -eq 0 && -n "$OUTPUT" && "$OUTPUT" == *"strain_hash:"* ]]; then
    pass "inoculate-preamble: outputs non-empty with strain_hash, exits 0"
else
    fail "inoculate-preamble: expected non-empty output with strain_hash and exit 0 (got exit=$EXIT_CODE, len=${#OUTPUT})"
fi

# --- Test 2: --max 100 truncates to ≤100 chars
OUTPUT_MAX="$("$PREAMBLE_BIN" --max 100)"
EXIT_CODE=$?
if [[ $EXIT_CODE -eq 0 && "${#OUTPUT_MAX}" -le 100 ]]; then
    pass "inoculate-preamble --max 100: output ≤100 chars (got ${#OUTPUT_MAX})"
else
    fail "inoculate-preamble --max 100: expected ≤100 chars, got ${#OUTPUT_MAX} (exit=$EXIT_CODE)"
fi

# --- Test 3: default output ≤1500 chars
OUTPUT_DEF="$("$PREAMBLE_BIN")"
if [[ "${#OUTPUT_DEF}" -le 1500 ]]; then
    pass "inoculate-preamble: default output ≤1500 chars (got ${#OUTPUT_DEF})"
else
    fail "inoculate-preamble: default output exceeds 1500 chars (got ${#OUTPUT_DEF})"
fi

# --- Test 4: inoculate-inject wrap has preamble before file contents
TMPFILE="$(mktemp)"
printf 'ORIGINAL_CONTENT_MARKER\n' > "$TMPFILE"
INJECT_OUT="$("$INJECT_BIN" wrap "$TMPFILE")"
INJECT_EXIT=$?
PREAMBLE_PART="$("$PREAMBLE_BIN")"
# Check preamble appears before file contents
if [[ $INJECT_EXIT -eq 0 && "$INJECT_OUT" == *"ORIGINAL_CONTENT_MARKER"* && "${#PREAMBLE_PART}" -gt 0 && "$INJECT_OUT" == *"---"* ]]; then
    # Verify preamble comes before file contents
    PREAMBLE_IN_OUTPUT="${INJECT_OUT%%---*}"
    if [[ "${PREAMBLE_IN_OUTPUT}" == *"strain_hash:"* ]]; then
        pass "inoculate-inject wrap: preamble + separator + file contents in correct order"
    else
        fail "inoculate-inject wrap: preamble not found before separator"
    fi
else
    fail "inoculate-inject wrap: missing separator or file contents (exit=$INJECT_EXIT)"
fi
rm -f "$TMPFILE"

# --- Test 5: SIGPIPE safety: inoculate-preamble | head -1 exits without error
"$PREAMBLE_BIN" | head -1 > /dev/null
PIPE_EXIT="${PIPESTATUS[0]}"
# SIGPIPE gives exit 141 in bash; we accept 0 or 141 (broken pipe is ok)
if [[ $PIPE_EXIT -eq 0 || $PIPE_EXIT -eq 141 ]]; then
    pass "SIGPIPE safety: inoculate-preamble | head -1 exits cleanly (exit=$PIPE_EXIT)"
else
    fail "SIGPIPE safety: inoculate-preamble | head -1 unexpected exit=$PIPE_EXIT"
fi

# --- Test 6: fallback when inoculate not on PATH
# Use a PATH that has bash/env/builtins but not inoculate
BASH_DIR="$(dirname "$(command -v bash)")"
ENV_DIR="$(dirname "$(command -v env)")"
# Build a restricted PATH with bash and env but exclude dirs containing inoculate
RESTRICTED_PATH="${BASH_DIR}:${ENV_DIR}:/usr/bin:/bin"
FALLBACK_OUT="$(PATH="$RESTRICTED_PATH" "$PREAMBLE_BIN")"
FALLBACK_EXIT=$?
if [[ $FALLBACK_EXIT -eq 0 && -n "$FALLBACK_OUT" ]]; then
    pass "fallback: inoculate absent from PATH → minimal floor, exits 0"
else
    fail "fallback: expected exit 0 and non-empty output (got exit=$FALLBACK_EXIT, len=${#FALLBACK_OUT})"
fi

# --- Summary
echo ""
printf 'Results: %d passed, %d failed\n' "$PASS" "$FAIL"
if [[ $FAIL -gt 0 ]]; then
    printf 'Failed tests:\n'
    for e in "${ERRORS[@]}"; do printf '  - %s\n' "$e"; done
    exit 1
fi
exit 0
