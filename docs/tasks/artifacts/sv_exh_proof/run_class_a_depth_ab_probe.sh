#!/usr/bin/env bash
# SV-EXH-PROOF.7.4.6.9 — the class-A DEPTH A/B, with its own ground-truth control.
#
# The static instruments (`class_a_min_derivation_depth.py`,
# `class_a_residual_depth_population.py`) predict that every class-A residual branch is a
# BUDGET failure: `construct_mode` commits to a min-TERMINAL-LENGTH derivation 48-54 deep
# while the witness pass runs at `--max-depth 20 x 2 = 40`.  This script tests that
# prediction against the running engine.
#
# It reproduces EXACTLY the `profile_<P>_closed_loop_replay` invocation of
# rust/scripts/sv_stimuli_quality_gate.sh:2290-2311 (the parameters come from that gate's own
# `closed_loop_replay_target_ceiling_configuration` summary line), then varies ONE knob.
#
#   ARM A  --max-depth 20   the gate's configuration -> the POSITIVE CONTROL.  It must
#                           reproduce the pinned residual (42 for profile_2017) and a
#                           witness-pass line identical to the gate's own log.  If arm A
#                           does not reproduce, arm B measures nothing.
#   ARM B  --max-depth 30   witness budget 60 >= the predicted 48-54.  If the depth model
#                           is right the class-A residual collapses; if it does not, the
#                           model is refuted.
#
# Requires a prior `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` run, whose work
# directory supplies the normalized grammar and the initial target report.  Every path is
# repo-root relative and every output stays on the repository volume.
#
#   Usage: bash docs/tasks/artifacts/sv_exh_proof/run_class_a_depth_ab_probe.sh [PROFILE] [DEPTH_A] [DEPTH_B]
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT_DIR" || exit 2

PROFILE="${1:-2017}"
DEPTH_A="${2:-20}"
DEPTH_B="${3:-30}"

OUT_DIR="rust/target/sv_exh_proof_7_4_6_9"
GATE_WORK="rust/target/sv_stimuli_quality_gate/work"
BIN="rust/target/debug/ast_pipeline"
GRAMMAR="$GATE_WORK/systemverilog_gen_ast.json"
TARGETS_IN="$GATE_WORK/profile_${PROFILE}_initial_gap.json"

mkdir -p "$OUT_DIR"
for f in "$BIN" "$GRAMMAR" "$TARGETS_IN"; do
    [[ -s "$f" ]] || {
        echo "error: missing input '$f' — run the SV stimuli quality gate first" >&2
        exit 2
    }
done

# The gate's own closed-loop replay parameters: seed_base 12001 + profile_index * 1000000,
# replay seed = profile_seed_base + 700000; count 8; max_repeat 2; target timeout 5 ms;
# helper timeout left at its 1000 ms default; target_max_attempts 5000.
case "$PROFILE" in
    2017) SEED=712001 ;;
    2023) SEED=1712001 ;;
    *) echo "error: unknown profile '$PROFILE' (expected 2017 or 2023)" >&2; exit 2 ;;
esac

run_arm() {
    local arm="$1" depth="$2"
    local tag="arm_${arm}_depth${depth}"
    echo "=== ${tag}: starting at $(date -u '+%Y-%m-%dT%H:%M:%SZ') ==="
    local start; start=$(date +%s)
    PGEN_TRACE_VERBOSITY=none "$BIN" "$GRAMMAR" \
        --generate-stimuli \
        --grammar-profile "sv_${PROFILE}" \
        --enforce-word-boundary-spacing \
        --count 8 \
        --seed "$SEED" \
        --entry-rule systemverilog_file \
        --max-depth "$depth" \
        --max-repeat 2 \
        --recovery-stimuli-mode baseline \
        --target-generation-timeout-ms 5 \
        --output "$OUT_DIR/${tag}_stimuli.txt" \
        --coverage-output "$OUT_DIR/${tag}_coverage.json" \
        --gap-report-json "$OUT_DIR/${tag}_replay_gap.json" \
        --gap-report-text "$OUT_DIR/${tag}_replay_gap.txt" \
        --gap-report-threshold 1 \
        --target-max-attempts 5000 \
        --target-report-input "$TARGETS_IN" \
        > "$OUT_DIR/${tag}.log" 2>&1
    local rc=$?
    local elapsed=$(( $(date +%s) - start ))
    local residual="n/a"
    if [[ -s "$OUT_DIR/${tag}_replay_gap.json" ]]; then
        residual="$(jq -er '(.targets // []) | length' "$OUT_DIR/${tag}_replay_gap.json" 2>/dev/null || echo parse-error)"
    fi
    echo "=== ${tag}: rc=${rc} elapsed=${elapsed}s residual_targets=${residual} ==="
    grep -E '^(Witness pass|Target-driven generation):' "$OUT_DIR/${tag}.log" || true
}

run_arm A "$DEPTH_A"
run_arm B "$DEPTH_B"
echo "=== probe complete at $(date -u '+%Y-%m-%dT%H:%M:%SZ') ==="
