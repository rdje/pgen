#!/usr/bin/env bash
# run_replay_trace_cost_ab.sh — price the SV closed-loop replay trace (`DONE-BAR.5f` scope item 2,
# which the leaf recorded as SUSPECTED-BUT-NOT-MEASURED with an explicit "do not quote a number
# until it is measured").
#
# Runs the SAME shadow-replay-shaped generation twice — once at `PGEN_TRACE_VERBOSITY=low` (the
# default `sv_stimuli_quality_gate.sh` carried from 2026-04-21 to 2026-07-30) and once at `none` —
# and reports wall time, log bytes, log lines, and the backtrack-noise share. It then asserts the
# two arms produce BYTE-IDENTICAL stimuli and an IDENTICAL parseability report, which is what makes
# the trace pure overhead rather than a behavioural difference.
#
# ⛔ REFUSES (exit 2) rather than reporting a partial verdict when the inputs are absent: the
# generation input is `generated/systemverilog.json`, and `generated/` is untracked, so a naive run
# in a clean checkout would otherwise measure nothing and look like a pass.
#
# ⭐ Generation reads the raw-AST JSON, not the `.ebnf`, so the run does NOT need an
# `ebnf_dual_run`-featured binary — the `DONE-BAR.5c` lesson about a gate whose verdict depended on
# which make target built `rust/target/debug/ast_pipeline` last.
#
#   bash docs/tasks/artifacts/done_bar/run_replay_trace_cost_ab.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

GRAMMAR_JSON="generated/systemverilog.json"
BIN="rust/target/debug/ast_pipeline"
GUARD="scripts/run_with_memory_guard.sh"
WORK="rust/target/done_bar_5f/ab"

refuse() { printf 'REFUSED: %s\n' "$1" >&2; exit 2; }

[[ -s "$GRAMMAR_JSON" ]] || refuse "$GRAMMAR_JSON absent — run 'make -C rust SHELL=/bin/bash regenerate_generated_parsers' first"
[[ -x "$BIN" ]]          || refuse "$BIN absent — build it with --features generated_parsers"
[[ -x "$GUARD" ]]        || refuse "$GUARD absent (HOST-RAM budget directive requires it)"

rm -rf "$WORK"; mkdir -p "$WORK"

# Pinned to the gate's own shadow-replay shape: sv_2017, the contract's entry rule / depth / repeat,
# and the closed-loop replay seed base (profile_seed_base 0 + 700000).
COUNT=8
SEED=700000

run_arm() {
    local arm="$1" verbosity="$2"
    local log="$WORK/$arm.log"
    local started ended
    started=$(date +%s)
    "$GUARD" --budget-mb 12288 --timeout-s 1800 --marker "$WORK/guard_$arm.marker" -- \
        env PGEN_TRACE_VERBOSITY="$verbosity" "$BIN" "$GRAMMAR_JSON" \
        --generate-stimuli \
        --grammar-profile sv_2017 \
        --enforce-word-boundary-spacing \
        --count "$COUNT" \
        --seed "$SEED" \
        --entry-rule systemverilog_file \
        --max-depth 24 \
        --max-repeat 4 \
        --recovery-stimuli-mode baseline \
        --output "$WORK/$arm.sv" \
        --validate-parseability \
        --parseability-report-json "$WORK/${arm}_report.json" \
        > "$log" 2>&1
    local rc=$?
    ended=$(date +%s)
    [[ "$rc" -eq 0 ]] || refuse "arm '$arm' exited $rc — see $log"
    local bytes lines noise
    bytes=$(wc -c < "$log" | tr -d ' ')
    lines=$(wc -l < "$log" | tr -d ' ')
    noise=$(grep -c '🧭' "$log" || true)
    printf '%s|%s|%s|%s|%s\n' "$((ended - started))" "$bytes" "$lines" "${noise:-0}" "$verbosity"
}

echo "=============================================================================="
echo "DONE-BAR.5f — SV closed-loop replay trace A/B (count=$COUNT, sv_2017, seed=$SEED)"
echo "=============================================================================="
echo "input:  $GRAMMAR_JSON  (raw-AST JSON — no ebnf_dual_run feature needed)"
"$BIN" --report-feature-surface 2>&1 | sed 's/^/binary: /'
echo

echo "==> arm A: PGEN_TRACE_VERBOSITY=low   (the retired default)"
IFS='|' read -r a_secs a_bytes a_lines a_noise _ < <(run_arm low low)
echo "==> arm B: PGEN_TRACE_VERBOSITY=none  (the shipped default)"
IFS='|' read -r b_secs b_bytes b_lines b_noise _ < <(run_arm none none)
echo

printf '%-14s %12s %18s %14s %14s\n' arm elapsed_s log_bytes log_lines noise_lines
printf '%-14s %12s %18s %14s %14s\n' "low (before)" "$a_secs" "$a_bytes" "$a_lines" "$a_noise"
printf '%-14s %12s %18s %14s %14s\n' "none (after)" "$b_secs" "$b_bytes" "$b_lines" "$b_noise"
echo

awk -v as="$a_secs" -v bs="$b_secs" -v ab="$a_bytes" -v bb="$b_bytes" -v al="$a_lines" -v an="$a_noise" '
BEGIN {
    if (as > 0) printf "trace share of wall time : %.1f%% (%ds of %ds)\n", (as - bs) * 100.0 / as, as - bs, as
    if (bs > 0) printf "slowdown factor          : %.1fx\n", as / bs
    if (bb > 0) printf "log-byte ratio           : %.0fx (%.2f GB vs %d B)\n", ab / bb, ab / 1073741824, bb
    if (al > 0) printf "noise share of the log   : %.2f%% (%d of %d lines)\n", an * 100.0 / al, an, al
    if (as > 0) printf "trace write rate         : %.1f MB/s, %.0f lines/s\n", ab / as / 1048576, an / as
}'
echo

fail=0
if cmp -s "$WORK/low.sv" "$WORK/none.sv"; then
    echo "✅ stimuli BYTE-IDENTICAL across the two arms"
else
    echo "⛔ stimuli DIFFER across the two arms — the trace is NOT behaviour-neutral"; fail=1
fi
if diff -q <(jq -S . "$WORK/low_report.json") <(jq -S . "$WORK/none_report.json") >/dev/null 2>&1; then
    echo "✅ parseability report IDENTICAL across the two arms"
else
    echo "⛔ parseability reports DIFFER — the trace is NOT behaviour-neutral"; fail=1
fi

# The 1.89 GB arm-A log has served its purpose; leave the small one for inspection.
rm -f "$WORK/low.log"
echo
echo "(arm A's log removed after measurement — it is ~1.9 GB and nothing reads it, which is the finding)"
echo "=============================================================================="
[[ "$fail" -eq 0 ]]
