#!/usr/bin/env bash
# run_published_version_currency_probes.sh — prove the PUBLISHED-VERSION-CURRENCY doctrine check
# (task-tree leaf `DONE-BAR.5a`; scripts/check_published_version_currency.sh) actually bites.
#
# Every arm asserts BOTH the exit code AND a substring of the message (CI-PARITY-GATE-ROT.4's rule:
# pass/fail alone hides arms that reach the right verdict for the wrong reason). The RED-1 arm is
# the HISTORICAL defect replayed verbatim: a guide publishing `1.1.29`/`1.1.31` — the exact stale
# pair this doctrine was created to make impossible to hold silently.
#
#   bash docs/tasks/artifacts/done_bar/run_published_version_currency_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

CHECK="$ROOT/scripts/check_published_version_currency.sh"
WORK="$ROOT/rust/target/done_bar_audit/pvc_probe"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0
fail=0
arm() {
    local label="$1" want_rc="$2" want_substr="$3"
    shift 3
    local out rc
    out="$(env "$@" bash "$CHECK" 2>&1)"; rc=$?
    if [[ "$rc" == "$want_rc" && "$out" == *"$want_substr"* ]]; then
        printf '  ✅ %-56s exit=%s, message matched\n' "$label" "$rc"
        pass=$((pass + 1))
    else
        printf '  ⛔ %-56s exit=%s (want %s)\n' "$label" "$rc" "$want_rc"
        printf '     wanted substring: %s\n' "$want_substr"
        printf '%s\n' "$out" | tail -n 6 | sed 's/^/     | /'
        fail=$((fail + 1))
    fi
}

echo "=============================================================================="
echo "PUBLISHED-VERSION-CURRENCY probe arms — scripts/check_published_version_currency.sh"
echo "=============================================================================="

# CTRL-1 — the untouched tree passes (the .5a fix landed: guide == contract identity == tracker).
arm "CTRL-1 untouched tree passes" 0 "published-version-currency: OK" --

# RED-1 — THE HISTORICAL DEFECT, verbatim: a guide still publishing the stale 1.1.29/1.1.31 pair.
sed -e 's/`1\.1\.106`/`1.1.29`/' -e 's/`1\.1\.109`/`1.1.31`/' PGEN_USER_GUIDE.md >"$WORK/guide_stale_pair.md"
arm "RED-1 stale published pair fails, named" 1 "publishes parser release '1.1.29'" \
    -- "PGEN_PVC_GUIDE=$WORK/guide_stale_pair.md"

# RED-2 — a stale published STATUS fails: the guide claims `Done` against an `In Progress` row.
sed -e '/^### Regex Parser Flavor/,/^### / s/^  - `In Progress`$/  - `Done`/' PGEN_USER_GUIDE.md >"$WORK/guide_stale_status.md"
arm "RED-2 stale published status fails, named" 1 "publishes regex family status 'Done'" \
    -- "PGEN_PVC_GUIDE=$WORK/guide_stale_status.md"

# RED-3 — an EMPTY extraction fails loudly, never comparing empty strings (the vacuous-green class).
printf '# a guide with no Regex Parser Flavor section\n' >"$WORK/guide_empty.md"
arm "RED-3 empty extraction refuses, never vacuous" 1 "could not extract the guide's published parser release version" \
    -- "PGEN_PVC_GUIDE=$WORK/guide_empty.md"

# RED-4 — the CONTRACT side moving without the guide fails too (the drift direction that actually
# happened: releases advanced ~77 times while the guide stood still).
sed -e '/^## Contract Identity/,/^## / s/`1\.1\.106`/`1.1.107`/' \
    docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md >"$WORK/contract_advanced.md"
arm "RED-4 contract advance without guide fails" 1 "declares '1.1.107'" \
    -- "PGEN_PVC_CONTRACT=$WORK/contract_advanced.md"

echo "------------------------------------------------------------------------------"
printf 'published-version-currency probes: %d/%d passed\n' "$pass" "$((pass + fail))"
[[ "$fail" -eq 0 ]] || exit 1
