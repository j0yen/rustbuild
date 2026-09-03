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
producers="supply-audit license-audit secrets-scan sbom msrv-verify binary-size semver-check cli-surface schema-compat ac-traceability flake-audit hermetic-build determinism cold-build-time bench-delta mutation-kill experiment"
for p in $producers; do command -v "$p" >/dev/null || { echo "missing producer bin: $p (cargo install --path ~/wintermute/rustbuild/autobuilder/crates/extended-gates --locked)" >&2; exit 2; }; done
t0=$(date +%s)
printf '%s\n' $producers | xargs -P "$par" -I{} bash -c 'timeout 1500 {} --project "$1" >/dev/null 2>&1; echo "{} rc=$?"' _ "$crate" | sort > /tmp/extended-receipts.$$ 
fail=0
for p in $producers; do
  v=$(python3 -c "import json,sys;print(json.load(open('$crate/target/autobuilder/receipts/$p-receipt.json'))['verdict'])" 2>/dev/null || echo "no-receipt")
  printf '%-16s %s\n' "$p" "$v"; case "$v" in pass|skipped) ;; *) fail=1;; esac
done
rm -f /tmp/extended-receipts.$$; echo "extended-receipts: $(( $(date +%s)-t0 ))s parallel=$par"; exit $fail
