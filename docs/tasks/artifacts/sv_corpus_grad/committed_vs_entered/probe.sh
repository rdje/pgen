#!/usr/bin/env bash
# SV-CORPUS-GRAD.13e.12 — why `rule_committed_counts` can EXCEED `rule_entry_counts`,
# re-derived by command, plus the RED control that makes `.13e.10`(c)'s leg-3 oracle
# a MEASUREMENT rather than a reading.
#
# WHY THIS FILE EXISTS
# --------------------
# `.13e.10`(c) promoted `--dump-rule-outcome-counts-json`'s `rule_committed_counts` to the
# leg-3 oracle for an entire lens: "the operator's own terminal COMMITS under `verilog_2005`".
# The first run of that oracle produced `dot_star` committed=3 against entered=1 — a rule
# committed more often than it was entered, in one parse of one file. An oracle whose two
# counters disagree in an unexplained direction is a trust question for every verdict built
# on it, so the number was ROUTED here rather than explained away.
#
# THE ANSWER, AND WHY IT IS NOT A DEFECT
# --------------------------------------
# The two maps count DIFFERENT POPULATIONS, and neither bounds the other:
#
#   rule_entry_counts[R]     = INVOCATIONS of R's generated rule method. One monotone
#                              `fetch_add` at the top of the method (and at
#                              `inlined_frame_call`), never rolled back, reported as a
#                              delta past a pre-parse baseline. A memo HIT at R still
#                              invokes R's method, so it counts.
#
#   rule_committed_counts[R] = OCCURRENCES of R in the FULLY EXPANDED derivation tree of
#                              the accepted parse. Not a counter at all: a post-parse fold
#                              (`exercised_rule_entry_counts`) over `coverage_stack` +
#                              `coverage_deltas`, where a memoized body's slots are moved
#                              into a side table and replaced by ONE `REPLAY|d` marker that
#                              the fold expands with a multiplicity.
#
# ⇒ a memo hit on an ANCESTOR of R adds an occurrence of R WITHOUT invoking R at all.
#   Entries therefore do not bound commits BY CONSTRUCTION.
#
# Arm 2 is the load-bearing one: the multiplicity law. Arm 4 is the one the SV lane needs —
# the leg-3 oracle must be able to read ZERO, or "committed >= 1" is not a measurement.
#
# Every scratch byte is written under `rust/target/` — no tracked file is ever touched.
#
# Usage: bash docs/tasks/artifacts/sv_corpus_grad/committed_vs_entered/probe.sh

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT_DIR" || exit 2

PROBE="rust/target/release/parseability_probe"
SCRATCH="rust/target/committed_vs_entered_probe"

# ⛔ A probe that cannot run must REFUSE, never report "0 failures". A missing binary is
# indistinguishable from a clean run to anything that only reads the exit code.
if [[ ! -x "$PROBE" ]]; then
    printf 'REFUSED: %s is absent — build it first (make -C rust SHELL=/bin/bash release_parseability_probe).\n' "$PROBE" >&2
    printf 'A probe that cannot measure must refuse rather than report a flattering zero.\n' >&2
    exit 2
fi

rm -rf "$SCRATCH"; mkdir -p "$SCRATCH"
trap 'rm -rf "$SCRATCH"' EXIT

pass=0
fail=0

note() { printf '  %s %-64s %s\n' "$1" "$2" "$3"; }
ok()   { note '✅' "$1" "$2"; pass=$((pass + 1)); }
bad()  { note '❌' "$1" "$2"; fail=$((fail + 1)); }

# dump <name> <source-text>  ->  writes $SCRATCH/<name>.json
dump() {
    local name="$1" text="$2"
    printf '%s\n' "$text" > "$SCRATCH/$name.sv"
    "$PROBE" --parse systemverilog "$SCRATCH/$name.sv" --profile verilog_2005 \
        --dump-rule-outcome-counts-json "$SCRATCH/$name.json" >/dev/null 2>&1
}

# read <file> <map> <rule>  ->  prints the count (0 when absent)
read_count() {
    python3 -c '
import json, sys
d = json.load(open(sys.argv[1]))
print(d[sys.argv[2]].get(sys.argv[3], 0))
' "$SCRATCH/$1.json" "$2" "$3"
}

# expect <label> <actual> <wanted>
expect() {
    if [[ "$2" == "$3" ]]; then ok "$1" "= $2"; else bad "$1" "= $2 (wanted $3)"; fi
}

echo "COMMITTED-vs-ENTERED PROBE — $ROOT_DIR"
echo

echo "[1] REPRODUCE — one \`.*\` site: dot_star is COMMITTED 3x while ENTERED once"
dump dot_star1 'module s; endmodule
module m; s u(.*); endmodule'
expect "dot_star entered"                     "$(read_count dot_star1 rule_entry_counts     dot_star)"                 1
expect "dot_star committed"                   "$(read_count dot_star1 rule_committed_counts dot_star)"                 3
# The multiplier is the ENCLOSING memoized rule's reach count, not anything about dot_star:
# list_of_port_connections is invoked 3x — one memo MISS plus 2 memo HITS — and its body
# delta contains dot_star exactly once.
expect "list_of_port_connections entered"     "$(read_count dot_star1 rule_entry_counts     list_of_port_connections)" 3
expect "list_of_port_connections memo hits"   "$(read_count dot_star1 rule_memo_hit_counts  list_of_port_connections)" 2
expect "list_of_port_connections committed"   "$(read_count dot_star1 rule_committed_counts list_of_port_connections)" 3

echo
echo "[2] CONTROL — the MULTIPLICITY LAW: double the sites, double the committed count"
# A defect in a counter has no reason to land on exactly 2x. An occurrence count does.
dump dot_star2 'module s; endmodule
module m; s u(.*); s v(.*); endmodule'
expect "dot_star entered (2 sites)"           "$(read_count dot_star2 rule_entry_counts     dot_star)"                 2
expect "dot_star committed (2 sites)"         "$(read_count dot_star2 rule_committed_counts dot_star)"                 6
expect "enclosing memo hits (2 sites)"        "$(read_count dot_star2 rule_memo_hit_counts  list_of_port_connections)" 4

echo
echo "[3] the two maps are NESTED, not disjoint — every committed rule was also entered"
# 298 rules are invoked; only 35 survive into the accepted tree. That direction is the
# ordinary one (failed speculation is truncated by try_parse) and is NOT the finding.
python3 - "$SCRATCH/dot_star1.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
e, c = d['rule_entry_counts'], d['rule_committed_counts']
orphans = [k for k in c if k not in e]
over = sorted(k for k, v in c.items() if v > e.get(k, 0))
print(f'  entry keys={len(e)}  committed keys={len(c)}  committed-not-entered={len(orphans)}')
print(f'  rules with committed > entered: {len(over)} -> {", ".join(over)}')
sys.exit(0 if not orphans else 1)
PY
if [[ $? -eq 0 ]]; then
    ok "committed keys are a SUBSET of entry keys" "no orphan rule"
else
    bad "committed keys are a SUBSET of entry keys" "orphan rule found"
fi

echo
echo "[4] ⭐ THE RED CONTROL the SV lane needs — the leg-3 oracle can read ZERO"
# `.13e.10`(c) reads "committed >= 1" as proof that the ACCEPT was carried by the operator
# itself. That is only a measurement if the same instrument returns 0 when the operator is
# absent. Each row below is a matched pair over the SAME enclosing construct.
dump plus_assign        'module m; initial begin a += 1; end endmodule'
dump plus_assign_ctl    'module m; initial begin a = 1; end endmodule'
dump shift_left_assign  'module m; initial begin a <<= 1; end endmodule'
dump wildcard_equal     'module m; initial begin if (a ==? b) ; end endmodule'
dump wildcard_equal_ctl 'module m; initial begin if (a == b) ; end endmodule'
dump dot_star_ctl       'module s; endmodule
module m; s u(.a(1)); endmodule'

for pair in "plus_assign:plus_assign:plus_assign_ctl" \
            "shift_left_assign:shift_left_assign:plus_assign_ctl" \
            "wildcard_equal:wildcard_equal:wildcard_equal_ctl" \
            "dot_star:dot_star1:dot_star_ctl"; do
    IFS=: read -r rule witness control <<< "$pair"
    w="$(read_count "$witness" rule_committed_counts "$rule")"
    c="$(read_count "$control" rule_committed_counts "$rule")"
    if [[ "$w" -ge 1 && "$c" -eq 0 ]]; then
        ok "$rule: witness committed=$w, control committed=0" "two-sided"
    else
        bad "$rule: witness committed=$w, control committed=$c" "wanted >=1 / 0"
    fi
done

echo
echo "[5] ⛔ and the ENTRY counter CANNOT do this job — the same reading in both arms"
# wildcard_equal is ENTERED twice whether or not `==?` is present: the parser speculates the
# branch either way. Only the COMMITTED map separates the accept from the failed probe. This
# is why `.13e.10`(c)'s leg 3 reads rule_committed_counts and never rule_entry_counts.
we="$(read_count wildcard_equal     rule_entry_counts wildcard_equal)"
ce="$(read_count wildcard_equal_ctl rule_entry_counts wildcard_equal)"
if [[ "$we" == "$ce" && "$we" -gt 0 ]]; then
    ok "wildcard_equal ENTERED $we in BOTH arms — indistinguishable" "committed separates them"
else
    bad "wildcard_equal entered witness=$we control=$ce" "wanted equal and > 0"
fi

echo
echo "[6] RED CONTROL ON THIS PROBE ITSELF — a wrong expectation must FAIL"
# A harness that has never been observed failing is not evidence of anything.
before=$fail
expect "(deliberate) dot_star committed"      "$(read_count dot_star1 rule_committed_counts dot_star)"                 99
if [[ $fail -eq $((before + 1)) ]]; then
    fail=$((fail - 1)); pass=$((pass + 1))
    ok "the deliberate-wrong arm FAILED as designed" "harness can go RED"
else
    bad "the deliberate-wrong arm did not fail" "harness cannot go RED"
fi

echo
printf 'RESULT: pass=%d fail=%d\n' "$pass" "$fail"
[[ $fail -eq 0 ]] || exit 1
exit 0
