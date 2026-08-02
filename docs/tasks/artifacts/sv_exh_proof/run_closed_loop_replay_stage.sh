#!/usr/bin/env bash
# SV-EXH-PROOF.7.4.6.13 / .7.4.6.14 — reproduce the canonical SV gate's CLOSED-LOOP REPLAY
# stage standalone, for one LRM profile, at the gate's exact pinned configuration.
#
# ⭐ WHY THIS EXISTS: the full `sv_stimuli_quality_gate` is ~27 minutes and rebuilds the SV
# parser; this stage alone is ~4-10 minutes and is the ONLY stage the residual ratchet reads.
# It is a FAITHFUL oracle, not an approximation — `PGEN-SV-EXH-PROOF-0172` verified that its
# `profile_{2017,2023}_replay_{stimuli.sv,gap.json,gap.txt}` come out BYTE-IDENTICAL (`cmp`) to
# the canonical gate's own artifacts, with the residual reproducing at the pinned `2017: 0` /
# `2023: 0`. That is what makes a cheap A/B of a generator change trustworthy.
#
# ⛔ IT DEPENDS ON A PRIOR CANONICAL GATE RUN. Two inputs come from
# `rust/target/sv_stimuli_quality_gate/work/`:
#   * `systemverilog_gen_ast.json`      — the normalized grammar the gate generated from
#   * `profile_<P>_initial_gap.json`    — the initial-stage gap report the replay targets
# If they are absent, run `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate` once first
# (under `scripts/run_with_memory_guard.sh`).
#
# ⛔ AND IT DEPENDS ON THE DUAL-FEATURE BINARY:
#   (cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline)
#
# The pinned configuration below is `closed_loop_replay_target_ceiling_configuration` from the
# gate's own `summary.txt` — the string the two-sided ratchet is pinned against. Change any of
# it and the measured residual is no longer comparable to the pins.
#
# Usage (from the repository root):
#   bash docs/tasks/artifacts/sv_exh_proof/run_closed_loop_replay_stage.sh 2017 rust/target/my_arm
#   python3 docs/tasks/artifacts/sv_exh_proof/depth_slack_census.py 20 rust/target/my_arm/profile_2017_replay_coverage.json
#   python3 -c "import json;print(len(json.load(open('rust/target/my_arm/profile_2017_replay_gap.json'))['targets']))"
set -euo pipefail

profile="${1:?usage: run_closed_loop_replay_stage.sh <2017|2023> <out_dir>}"
out_dir="${2:?usage: run_closed_loop_replay_stage.sh <2017|2023> <out_dir>}"

# All paths are repository-root relative, derived at runtime from this script's own location,
# so the checkout can move (repo policy §12).
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../../.." && pwd)"

case "$profile" in
    2017) profile_idx=0 ;;
    2023) profile_idx=1 ;;
    *) echo "error: unknown profile '$profile' (expected 2017 or 2023)" >&2; exit 2 ;;
esac

# Seeds mirror sv_stimuli_quality_gate.sh: profile_seed_base = seed_base + idx*1000000,
# replay_seed = profile_seed_base + 700000.
seed_base=12001
replay_seed=$((seed_base + profile_idx * 1000000 + 700000))

work="$repo_root/rust/target/sv_stimuli_quality_gate/work"
gen_ast="$work/systemverilog_gen_ast.json"
initial_gap="$work/profile_${profile}_initial_gap.json"
binary="$repo_root/rust/target/debug/ast_pipeline"

for required in "$gen_ast" "$initial_gap" "$binary"; do
    if [[ ! -s "$required" ]]; then
        echo "error: missing required input '$required'" >&2
        echo "note: see this script's header — run the canonical gate once, and build the" >&2
        echo "      dual-feature ast_pipeline, before using this probe." >&2
        exit 2
    fi
done

mkdir -p "$out_dir"

exec env PGEN_TRACE_VERBOSITY=none \
    "$binary" "$gen_ast" \
    --generate-stimuli \
    --grammar-profile "$profile" \
    --enforce-word-boundary-spacing \
    --count 8 \
    --seed "$replay_seed" \
    --entry-rule systemverilog_file \
    --max-depth 20 \
    --max-repeat 2 \
    --recovery-stimuli-mode baseline \
    --target-generation-timeout-ms 5 \
    --output "$out_dir/profile_${profile}_replay_stimuli.sv" \
    --coverage-output "$out_dir/profile_${profile}_replay_coverage.json" \
    --gap-report-json "$out_dir/profile_${profile}_replay_gap.json" \
    --gap-report-text "$out_dir/profile_${profile}_replay_gap.txt" \
    --gap-report-threshold 1 \
    --target-max-attempts 5000 \
    --target-report-input "$initial_gap"
