#!/usr/bin/env bash
# audit.sh-trace: emits ::audit-start marker for harness instrumentation
# audit.sh — runs the BAD_RUST audit against this project.
# READ-ONLY: the edit-agent must not modify this file.
#
# Uses ~/.claude/skills/rustbuild/rules/audit-checks.sh when present (a
# build box), else the vendored copy at rules/audit-checks.sh (CI has no
# ~/.claude — mirrors mcphost's scripts/audit.sh convention).
# PRD-build-gate-producers: rustbuild had no scripts/audit.sh at all before
# this — risk-gate.json came back empty on every fresh run (confirmed in
# agent/gate-baseline.json's inherited blockers). This gives it the same
# audit wiring mcphost/wm-node/adopt already carry, pointed at the live
# (renamed) skill path from the start.
# Output: target/autobuilder/receipts/risk-gate.json (the gate's expected
# location — eliminates the previous manual `cp audit.json risk-gate.json`
# step every standalone build needed).
# Exit 0 if no BLOCKING findings, 1 otherwise.

set -euo pipefail
cd "$(dirname "$0")/.."

AUDIT="$HOME/.claude/skills/rustbuild/rules/audit-checks.sh"
[ -x "$AUDIT" ] || AUDIT="rules/audit-checks.sh"
mkdir -p target/autobuilder/receipts

if [ ! -x "$AUDIT" ]; then
  echo "audit: no audit-checks.sh (skill copy absent and no vendored rules/audit-checks.sh)" >&2
  exit 1
fi

"$AUDIT" . > target/autobuilder/receipts/risk-gate.json
