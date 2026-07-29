#!/usr/bin/env bash
# run_demotion_impact_probe.sh — what happens to the family-status gates when DONE-BAR.2 demotes a
# row? (task-tree leaf `DONE-BAR.2`.)
#
# THE QUESTION. `.2` is chartered as "demote what does not meet the bar". Before editing a single
# tracker row, this probe answers the thing that decides `.2`'s SHAPE: the family-status gates assert
# `tracker_alignment_ok` by exact string equality against the tracker row and `exit 1` on mismatch.
# If they cannot COMPUTE the status an honest demotion writes, then demoting first does not record
# the truth — it manufactures two RED gates and blocks CI-PARITY-GATE-ROT.7 harder.
#
# ⛔ THE ALIGNMENT LOGIC IS REPLAYED, NOT DESCRIBED. The reader function and the comparison are
# extracted from the LIVE gate scripts, so this probe cannot test a rule the gates do not apply.
#
# ⚠️ SUPERSEDED AS A LIVE INSTRUMENT (2026-07-29, `PGEN-DONE-BAR-0010`): this probe is the
# `.2`-shaping BEFORE record. Its step-2 "gate COMPUTES (run 3)" column is a SNAPSHOT of aggregate
# run 3 under the OLD gate logic, and its verdict prose describes the pre-`.2a` gates. `.2a` has
# since landed exactly what this probe demanded (the gates now compute the qualified `Provisional`
# and carry a leg-3 criterion — note step 1's own "Provisional anywhere" count moved 0 → 4).
# The live AFTER instrument is run_family_status_bar_probes.sh.
#
#   bash docs/tasks/artifacts/done_bar/run_demotion_impact_probe.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

# ⛔ HISTORICAL GUARD (DONE-BAR.2b): this probe's step-2 replay mutates the three `| Done |` rows it
# was written against. Those rows were demoted by `.2b`, so the mutation targets no longer exist —
# and re-running the BEFORE experiment against the demoted tracker would MISCALIBRATE for a reason
# that is the fix landing, not an instrument defect. Say so and stop, instead of failing.
if ! grep -qF '| `vhdl` parser family | Done |' "$ROOT/LIVE_ACHIEVEMENT_STATUS.md"; then
    echo "=============================================================================="
    echo "DEMOTION IMPACT PROBE — HISTORICAL (superseded by DONE-BAR.2a/.2b)"
    echo "=============================================================================="
    echo
    echo "The demotion this probe was written to shape has LANDED: the family-status gates were"
    echo "taught the qualified Provisional vocabulary + the leg-3 criterion first (.2a,"
    echo "PGEN-DONE-BAR-0010), and the tracker rows then moved (.2b, PGEN-DONE-BAR-0011)."
    echo "The captured BEFORE record is demotion_impact_probe.txt; the live AFTER instrument is"
    echo "run_family_status_bar_probes.sh."
    exit 0
fi

WORK="$ROOT/rust/target/done_bar_audit/demotion_probe"
rm -rf "$WORK"; mkdir -p "$WORK"

echo "=============================================================================="
echo "DEMOTION IMPACT PROBE — can the family-status gates express an honest demotion?"
echo "=============================================================================="
echo

# --- 1. The status vocabulary each gate can COMPUTE, derived from the live scripts ---------------
echo "1. STATUS VOCABULARY each family-status gate can compute (derived from the live scripts)"
echo
for s in "$ROOT"/rust/scripts/*_parser_family_status_gate.sh; do
    printf '   %-42s %s\n' "$(basename "$s")" \
        "$(grep -oE '_status="[A-Za-z ]+"' "$s" | sed 's/_status=//;s/"//g' | sort -u | paste -sd'/' -)"
done
echo
printf '   gate scripts mentioning "Provisional" anywhere: %s\n' \
    "$(grep -rl 'Provisional' "$ROOT"/rust/scripts/ 2>/dev/null | wc -l | tr -d ' ')"
echo

# --- 2. Replay the LIVE alignment comparison against a demoted tracker --------------------------
# markdown_table_status_for_row was byte-identical across the three status gates until DONE-BAR.2a
# moved it into its single home, rust/scripts/lib/parser_family_status_bar.sh (sourced by all
# three); extract it from there and drive it with the tracker rows each gate actually matches on.
echo "2. REPLAY of the live tracker reader + alignment comparison against a DEMOTED tracker"
echo

sed -n '/^markdown_table_status_for_row() {/,/^}/p' \
    "$ROOT/rust/scripts/lib/parser_family_status_bar.sh" >"$WORK/reader.sh"
if [[ ! -s "$WORK/reader.sh" ]]; then
    echo "   ⛔ MISCALIBRATED: could not extract markdown_table_status_for_row from the live gate." >&2
    exit 3
fi

# The row matchers are the ones the gates themselves grep for. Tab-separated: the matcher text
# contains spaces, pipes and backticks, so any of those would corrupt the split.
MATCHERS=$(cat <<-'SPEC'
	regex	| `regex` parser family |	In Progress
	vhdl	| `vhdl` parser family |	Done
	systemverilog_preprocessor	| `systemverilog_preprocessor` frontend (`Phase Q`) |	Done
SPEC
)

cp "$ROOT/LIVE_ACHIEVEMENT_STATUS.md" "$WORK/tracker_demoted.md"
# The demotion `.2` would write, per the audit's derived qualifiers.
python3 - "$WORK/tracker_demoted.md" <<'PY'
import re, sys
p = sys.argv[1]
t = open(p, encoding="utf-8").read()
for row, new in (
    ("| `regex` parser family | Done |", "| `regex` parser family | Provisional (corpus pending) |"),
    ("| `vhdl` parser family | Done |", "| `vhdl` parser family | Provisional (corpus pending) |"),
    ("| `systemverilog_preprocessor` frontend (`Phase Q`) | Done |",
     "| `systemverilog_preprocessor` frontend (`Phase Q`) | Provisional (corpus pending) |"),
):
    if row not in t:
        raise SystemExit(f"MISCALIBRATED: tracker row not found verbatim: {row}")
    t = t.replace(row, new, 1)
open(p, "w", encoding="utf-8").write(t)
PY
rc=$?
if [[ $rc -ne 0 ]]; then
    echo "   ⛔ MISCALIBRATED: the tracker rows the gates match on are not where this probe expects." >&2
    exit 3
fi

printf '   %-28s %-28s %-28s %s\n' "family" "gate COMPUTES (run 3)" "tracker AFTER demotion" "alignment"
printf '   %s\n' "$(printf -- '-%.0s' {1..106})"
breaks=0
rows=0
while IFS=$'\t' read -r family matcher computed; do
    [[ -n "$family" ]] || continue
    rows=$((rows + 1))
    tracker_after="$(
        # shellcheck disable=SC1090
        source "$WORK/reader.sh"
        markdown_table_status_for_row "$matcher" "$WORK/tracker_demoted.md"
    )"
    # ⛔ THE GUARD THIS PROBE'S OWN FIRST CUT NEEDED. A broken field split made both sides EMPTY,
    # and empty == empty reported "✅ aligns" for all three rows — a comfortable answer derived from
    # no data at all. Two empty strings are never evidence of agreement.
    if [[ -z "$computed" || -z "$tracker_after" ]]; then
        echo "   ⛔ MISCALIBRATED: empty side for '$family' (computed='$computed' tracker='$tracker_after')" >&2
        exit 3
    fi
    if [[ "$computed" == "$tracker_after" ]]; then
        verdict="✅ aligns"
    else
        verdict="⛔ MISMATCH ⇒ gate exit 1"
        breaks=$((breaks + 1))
    fi
    printf '   %-28s %-28s %-28s %s\n' "$family" "$computed" "$tracker_after" "$verdict"
done <<<"$MATCHERS"
if [[ "$rows" -ne 3 ]]; then
    echo "   ⛔ MISCALIBRATED: expected 3 replayed rows, parsed $rows" >&2
    exit 3
fi
echo

# --- 3. Verdict ---------------------------------------------------------------------------------
echo "3. VERDICT"
echo
if [[ "$breaks" -eq 0 ]]; then
    echo '   The gates can express the demotion. `.2` may edit the tracker directly.'
else
    cat <<EOF
   ⛔ ${breaks} of 3 family-status gates would FAIL on an honest demotion.

   The gates implement the OLD bar: they compute a status from legs 1-2 only, have no leg-3
   criterion, and \`Provisional\` is not in their vocabulary at all. So the demotion \`.2\` must
   write is a status no gate can COMPUTE, and every gate compares by exact string equality then
   exits 1.

   ⇒ Demoting the tracker FIRST does not record the truth; it manufactures RED gates -- including
     two (\`vhdl\`, \`systemverilog_preprocessor\`) that pass today -- and blocks
     CI-PARITY-GATE-ROT.7 harder than the blocker it was meant to clear.

   ⇒ \`.2\` SPLITS: teach the gates the \`Provisional\` vocabulary and a leg-3 criterion FIRST
     (\`.2a\`), so a demoted row is what the gate COMPUTES rather than a disagreement with it;
     then move the rows (\`.2b\`).
EOF
    exit 1
fi
