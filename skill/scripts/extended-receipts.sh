#!/usr/bin/env bash
# extended-receipts.sh — run the 17 extended-gates producers against a crate, in parallel.
#   usage: extended-receipts.sh <crate-dir> [parallelism=6]
# Each producer writes target/autobuilder/receipts/<name>-receipt.json and prints its verdict.
# Producers are independent binaries (cargo install --path autobuilder/crates/extended-gates);
# determinism/cold-build-time build in their own temp target dirs, so they can overlap.
# Run this on the final HEAD, after the audit (scripts/audit.sh) has finished — not concurrently
# with it (cargo-deny inside the audit contends with producer builds and can fail spuriously).
# Exit 0 when every producer wrote a receipt whose verdict is pass|skipped; 1 otherwise.
set -uo pipefail
crate="${1:?crate dir}"; par="${2:-6}"; export PATH="$HOME/.cargo/bin:$PATH"
# Keep the cargo route shims (cargo-budget-bin, burst-lane-bin) AHEAD of rustup's
# cargo: prepending ~/.cargo/bin above shadowed them, so every producer's cargo
# (flake-audit's 3x full test suite, mutation-kill, ...) ran local and unbudgeted
# even with the burst lane enabled — 2026-09-16, gates 30+ min local on RedBaron.
_shims="$(printf '%s' "$PATH" | tr ':' '\n' | grep -E '/(cargo-budget-bin|burst-lane-bin)$' | paste -sd: -)"
[ -n "$_shims" ] && export PATH="$_shims:$PATH"
producers="supply-audit license-audit secrets-scan sbom msrv-verify binary-size semver-check cli-surface schema-compat ac-traceability flake-audit hermetic-build determinism cold-build-time bench-delta mutation-kill experiment"
for p in $producers; do command -v "$p" >/dev/null || { echo "missing producer bin: $p (cargo install --path ~/wintermute/rustbuild/autobuilder/crates/extended-gates --locked)" >&2; exit 2; }; done

# PRD-build-diff-scoped-gate requirement 1 (P0, AC1/AC3): each producer's
# class — `tree` (hermetic, keyed only by the branch tree — safe to run
# at extend-gate.sh's --scope branch) or `history-infra` (reads a
# previous/published baseline, an external registry, or other non-tree
# state — deferred at --scope branch, per extend-gate.sh's own
# risk-gate/intake/rollback-plan/ci-checks table). This is a declaration
# only — extend-gate.sh does not yet read it to skip history-infra
# producers at branch scope (that consumer wiring is a separate step);
# today this script still runs every producer regardless of scope,
# unchanged. A producer with NO entry here is history-infra by
# construction (producer_class's own fallback below) — fail safe, so an
# undeclared producer can never wrongly get treated as safe-to-run at
# branch scope once a consumer does read this.
#   tree: secrets-scan, sbom, msrv-verify, binary-size, cli-surface,
#         ac-traceability, flake-audit, hermetic-build, determinism,
#         cold-build-time, mutation-kill — each reads/builds only the
#         crate's own tree (no comparison against a prior release, no
#         registry/network lookup beyond what `cargo build` itself needs).
#   history-infra: supply-audit, license-audit (query the crates.io/RUSTSEC
#         registry, not just the tree), semver-check, schema-compat,
#         bench-delta (each compares HEAD against a previous published
#         version or a stored baseline — history, not tree).
#   `experiment` is deliberately NOT declared: its inputs vary per use and
#   no single class is safe to assume — it falls through to history-infra
#   by the fail-safe default below rather than a guessed classification.
declare -A producer_class=(
  [secrets-scan]=tree [sbom]=tree [msrv-verify]=tree [binary-size]=tree
  [cli-surface]=tree [ac-traceability]=tree [flake-audit]=tree
  [hermetic-build]=tree [determinism]=tree [cold-build-time]=tree
  [mutation-kill]=tree
  [supply-audit]=history-infra [license-audit]=history-infra
  [semver-check]=history-infra [schema-compat]=history-infra
  [bench-delta]=history-infra
)
producer_class_of() {  # $1=producer name -> prints its class, defaults history-infra
  printf '%s\n' "${producer_class[$1]:-history-infra}"
}
t0=$(date +%s)
# PRD-build-burst-gate-canary-invariant R15: tag each producer's own cargo
# call with its own name, not the blanket "extended-receipts" step
# extend-gate.sh's run_unslotted_producer() sets for this whole script's
# process tree — cargo-budget-bin/cargo already forwards whatever
# CARGO_BUDGET_PARENT_STEP it inherits straight into the ledger row, so
# this per-producer override (env-var prefix on the `timeout` command,
# applies to the whole subtree timeout execs) is the only change needed:
# a burst-intended gate's R15 check can now attribute a ledger row to
# flake-audit/mutation-kill/etc. by name instead of only ever seeing
# "extended-receipts" and having no per-producer signal at all.
printf '%s\n' $producers | xargs -P "$par" -I{} bash -c 'CARGO_BUDGET_PARENT_STEP="{}" timeout 1500 {} --project "$1" >/dev/null 2>&1; echo "{} rc=$?"' _ "$crate" | sort > /tmp/extended-receipts.$$
fail=0
for p in $producers; do
  v=$(python3 -c "import json,sys;print(json.load(open('$crate/target/autobuilder/receipts/$p-receipt.json'))['verdict'])" 2>/dev/null || echo "no-receipt")
  printf '%-16s %s\n' "$p" "$v"; case "$v" in pass|skipped) ;; *) fail=1;; esac
done
rm -f /tmp/extended-receipts.$$; echo "extended-receipts: $(( $(date +%s)-t0 ))s parallel=$par"; exit $fail
