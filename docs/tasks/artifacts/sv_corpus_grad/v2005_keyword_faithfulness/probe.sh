#!/usr/bin/env bash
# SV-CORPUS-GRAD.13e.7 (c) — refusal + control probes for the `verilog_2005` keyword-faithfulness
# census (`stimuli/sv/v2005_keyword_faithfulness_census.py`).
#
# WHY THIS FILE EXISTS
# --------------------
# An instrument that can only ever return one reading is not a measurement. This probe drives the
# census until every refusal path fires and until the VERDICT COLUMN MOVES, so a green baseline
# means "measured and clean" rather than "the code never reached the interesting branch".
#
# Arm 2 is the load-bearing one: switching the censused profile to `sv_2017` must make the
# candidate population EXPLODE, because `sv_2017` is where those keywords legitimately live. If
# the reading did not move, the reachability computation would not be reading the profile at all.
#
# Every mutation happens in a scratch copy under `rust/target/` — no tracked file is ever written.
#
# Usage: bash docs/tasks/artifacts/sv_corpus_grad/v2005_keyword_faithfulness/probe.sh

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT_DIR" || exit 2

CENSUS="stimuli/sv/v2005_keyword_faithfulness_census.py"
ARTIFACT="docs/tasks/artifacts/sv_corpus_grad/v2005_keyword_faithfulness"
WITNESSES="$ARTIFACT/witnesses"
SCRATCH="rust/target/v2005_keyword_faithfulness_probe"

pass=0
fail=0

rm -rf "$SCRATCH"
mkdir -p "$SCRATCH"
trap 'rm -rf "$SCRATCH"' EXIT

# check <name> <expected-rc> <must-appear-in-output> -- <command...>
check() {
    local name="$1" want_rc="$2" want_text="$3"; shift 4
    local out rc
    out="$("$@" 2>&1)"; rc=$?
    if [[ "$rc" == "$want_rc" ]] && [[ -z "$want_text" || "$out" == *"$want_text"* ]]; then
        printf '  ✅ %-58s rc=%s\n' "$name" "$rc"; pass=$((pass + 1))
    else
        printf '  ❌ %-58s rc=%s (wanted %s / %s)\n' "$name" "$rc" "$want_rc" "$want_text"
        printf '     %s\n' "${out:0:400}"; fail=$((fail + 1))
    fi
}

echo "V2005-KEYWORD-FAITHFULNESS PROBE — $ROOT_DIR"
echo

echo "[1] the baseline is GREEN and the tracked artifact is current"
check "baseline gate mode" 0 "over_acceptances=" -- \
    python3 "$CENSUS"

echo
echo "[2] CONTROL — the reading MOVES with the profile (it is not stuck)"
# The SAME code, over the SAME grammar and the SAME oracle, must report a different population
# once only the profile changes. `verilog_2005` reaches 24 word-shaped IEEE-1800-only keywords;
# `sv_2017`, where those keywords are native, reaches 136. A census blind to the profile would
# print one of those numbers twice.
check "verilog_2005 reaches 24 IEEE-1800-only keywords" 0 "candidates=24" -- \
    python3 "$CENSUS"
check "sv_2017 reaches 136 — the same code, a different reading" 1 "\`sv_2017\` reaches 136" -- \
    python3 "$CENSUS" --profile sv_2017
# Censusing the CONTROL profile itself is a degenerate self-comparison, and the census names it
# rather than reporting a flattering "leaks 0".
check "the degenerate self-comparison refuses by name" 1 "not reading the profile" -- \
    python3 "$CENSUS" --profile sv_2017

echo
echo "[3] a CONTROL run can never become the baseline"
check "--write refused under --profile" 2 "must never become the baseline" -- \
    python3 "$CENSUS" --profile sv_2017 --write
check "--write refused under --witness-dir" 2 "must never become the baseline" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/none" --write

echo
echo "[4] a NEW reachable IEEE-1800-only keyword cannot land silently"
cp -R "$WITNESSES" "$SCRATCH/dropped_row"
grep -v $'^chandle\t' "$WITNESSES/MANIFEST.tsv" > "$SCRATCH/dropped_row/MANIFEST.tsv"
check "manifest row removed for a reachable keyword" 1 "have no witness" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/dropped_row"

echo
echo "[5] a manifest row whose witness file is absent REFUSES"
cp -R "$WITNESSES" "$SCRATCH/missing_file"
rm "$SCRATCH/missing_file/chandle.sv"
check "witness named by the manifest does not exist" 1 "does not exist" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/missing_file"

echo
echo "[6] a witness that cannot run LEG 3 REFUSES (the row could not be earned)"
cp -R "$WITNESSES" "$SCRATCH/no_keyword"
printf 'module m; integer i; endmodule\n' > "$SCRATCH/no_keyword/chandle.sv"
check "witness does not contain its own keyword" 1 "cannot be run" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/no_keyword"

echo
echo "[7] a reachable keyword whose sv_2017 CONTROL fails is an unmeasured HOLE, not 'clean'"
cp -R "$WITNESSES" "$SCRATCH/broken_control"
printf 'module m; chandle chandle chandle; endmodule\n' > "$SCRATCH/broken_control/chandle.sv"
check "sv_2017 control fails on a reachable keyword" 1 "nothing was measured" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/broken_control"

echo
echo "[8] gate mode REFUSES a stale tracked artifact"
cp "$ARTIFACT/census.tsv" "$SCRATCH/census.tsv.bak"
printf 'stale\n' >> "$ARTIFACT/census.tsv"
check "census.tsv drifted from a fresh derivation" 1 "DRIFT" -- \
    python3 "$CENSUS"
cp "$SCRATCH/census.tsv.bak" "$ARTIFACT/census.tsv"
check "artifact restored, gate green again" 0 "over_acceptances=" -- \
    python3 "$CENSUS"

echo
echo "[9] the census is DETERMINISTIC"
python3 "$CENSUS" --write > /dev/null 2>&1
cp "$ARTIFACT/census.tsv" "$SCRATCH/run1.tsv"
python3 "$CENSUS" --write > /dev/null 2>&1
if cmp -s "$SCRATCH/run1.tsv" "$ARTIFACT/census.tsv"; then
    printf '  ✅ %-58s\n' "two writes are byte-identical"; pass=$((pass + 1))
else
    printf '  ❌ %-58s\n' "two writes DIFFER"; fail=$((fail + 1))
fi

echo
echo "V2005-KEYWORD-FAITHFULNESS-PROBE: pass=$pass fail=$fail"
[[ "$fail" -eq 0 ]] || exit 1
