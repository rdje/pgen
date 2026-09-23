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
# once only the profile changes. `verilog_2005` reaches 13 word-shaped IEEE-1800-only keywords
# (14 until SV-CORPUS-GRAD.13c.2m, release 1.0.244: `class_qualifier` left the candidate set — it was never a
# keyword at all but IEEE 1800 Annex A's NONTERMINAL, welded into a literal token by the grammar extractor
# and reached under `verilog_2005` through `primary_hier_scope_prefix` and the welded `null class_qualifier := …`
# alternative of `primary_sv_2017`; the token, the alternatives and the literal arms are deleted, so the
# census's roster no longer carries a name that Annex B never reserved; measured: candidates 14 → 13,
# `sv_2017` 135 → 134, its manifest row and witness retired with it;
# 15 until SV-CORPUS-GRAD.3.66, release 1.0.234: `ref` left the reachable set when the ANSI port
# header and the function port took `verilog_2005` twins whose directions are input / output /
# inout only — IEEE 1364-2005 A.2.1.2, A.2.6 — so `kw_ref_69449f99` is reached by the sv twins alone;
# measured by the census module on the 1.0.233 and 1.0.234 grammars: removed=['ref'], added=[]. The 15 were
# (11 + the five `timescale time units SV-CORPUS-GRAD.3.39's typed directives made reachable, minus
# `tx_path_delay_expression` — an Annex A production NAME shipped as a literal terminal in
# list_of_path_delay_expressions' non-LRM seventh alternative, counted here as an identifier-consumed
# candidate until SV-CORPUS-GRAD.3.45 deleted the alternative and its terminal, release 1.0.219);
# `sv_2017`, where those keywords are native, reaches 134 for the same reason (135 before .13c.2m, 136 before .3.45). A
# census blind to the profile would print one of those numbers twice. ⛔ These two constants are a
# tripwire, not a budget: a reachable IEEE-1800-only keyword added or removed moves them by one, and
# the slice that moves them owes this comment its reason.
check "verilog_2005 reaches 13 IEEE-1800-only keywords" 0 "candidates=13" -- \
    python3 "$CENSUS"
check "sv_2017 reaches 134 — the same code, a different reading" 1 "\`sv_2017\` reaches 134" -- \
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
# `this` is one of the 13 words the profile-filtered reach analysis arrives at (SV-CORPUS-GRAD.13c.2m: until 1.0.244 these two
# arms used `class_qualifier`, the extractor's welded token, which left the roster with the weld).
cp -R "$WITNESSES" "$SCRATCH/dropped_row"
grep -v $'^this\t' "$WITNESSES/MANIFEST.tsv" > "$SCRATCH/dropped_row/MANIFEST.tsv"
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
printf 'module top; integer y; this this; endmodule\n' > "$SCRATCH/broken_control/this.sv"
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
echo "[9] LEG 4 (SV-CORPUS-GRAD.3.39) — a directive-argument row is EARNED, never declared"
# `ns` is consumed as a keyword inside `` `timescale 1 ns / 1 ns `` (leg 3 rejects the substituted
# witness) and IEEE 1364-2005 19.8 gives it exactly that role; the manifest's fourth column names
# an identifier witness that must accept before AND after the word is swapped for a fresh
# identifier. Each arm below breaks one of the leg's conditions and the census must SAY which.
cp -R "$WITNESSES" "$SCRATCH/leg4_missing"
rm "$SCRATCH/leg4_missing/ns_identifier.sv"
check "identifier witness named by the manifest does not exist" 1 "identifier witness" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/leg4_missing"
cp -R "$WITNESSES" "$SCRATCH/leg4_reserved"
printf 'module m; wire ns ns; endmodule\n' > "$SCRATCH/leg4_reserved/ns_identifier.sv"
check "identifier witness REJECTS: the verdict column moves to over_acceptance" 0 "over_acceptances=1" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/leg4_reserved"
cp -R "$WITNESSES" "$SCRATCH/leg4_off_line"
printf '`timescale 1 ns / 1 ns\nmodule m; wire x; assign x = ns; endmodule\n' > "$SCRATCH/leg4_off_line/ns.sv"
check "keyword used OFF a backtick-led line refuses the row" 1 "off a backtick-led line" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/leg4_off_line"
cp -R "$WITNESSES" "$SCRATCH/leg4_load_bearing"
cp "$WITNESSES/ns.sv" "$SCRATCH/leg4_load_bearing/ns_identifier.sv"
check "identifier witness where the word is load-bearing refuses the row" 1 "load-bearing" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/leg4_load_bearing"
cp -R "$WITNESSES" "$SCRATCH/leg4_no_column"
python3 - "$SCRATCH/leg4_no_column/MANIFEST.tsv" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); rows = []
for line in p.read_text().splitlines():
    f = line.split("\t")
    rows.append("\t".join(f[:3]) if f and f[0] == "ns" else line)
p.write_text("\n".join(rows) + "\n")
PY
check "manifest row WITHOUT the fourth column: leg 3 alone reads over_acceptance" 0 "over_acceptances=1" -- \
    python3 "$CENSUS" --witness-dir "$SCRATCH/leg4_no_column"

echo
echo "[10] the census is DETERMINISTIC"
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
