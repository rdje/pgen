#!/usr/bin/env bash
# SV-EXH-PROOF.7.4.6.13 defect (ii) — is the DECLARED CEILING inert for every grammar but SV?
#
# The ceiling (`DEPTH_SLACK_RETRY_CEILING_MULTIPLE`, rust/src/ast_pipeline/stimuli_generator.rs) is
# a GENERATOR change, so "it only affects SystemVerilog" is a claim that has to be measured, not
# argued. For each grammar this runs three generations and checks two things:
#
#   * DETERMINISM control  — ceiling OFF twice must be byte-identical, else the arm comparison
#     below means nothing.
#   * INERTNESS measurement — ceiling OFF vs ON must be byte-identical for any grammar whose
#     ladder never reaches the ceiling.
#   * NO-OUTPUT refusal    — a run that produced no file is REPORTED, never scored as a difference.
#
# ⛔ THAT THIRD CHECK IS NOT DEFENSIVE PADDING — it is the defect this instrument was born with.
# The first cut lacked it and read `rtl_const_expr` as DIVERGING on all three seeds. It was `cmp`
# on two files neither arm ever wrote: that grammar fails outright at the default depth
# (`Stimuli generation depth exceeded max_depth=24 while expanding rule 'primary_expr'`,
# pre-existing and documented at TOOLBOX 1.6 / PARSE-HARNESS §18), and its depth-slack retry never
# fires at all — no census line is emitted in EITHER arm — so the ceiling provably cannot reach it.
# A comparison instrument without an existence check reports ABSENCE as DIVERGENCE.
#
# The sweep's own positive control is external and deliberate: the SV closed-loop replay A/B
# (`rust/target/sv_exh_proof_7_4_6_13_ceiling/`) DOES diverge between these same two arms, with
# `declared-ceiling: … refusals=2429` / `1116`. A sweep that can only ever print IDENTICAL would
# be indistinguishable from one that is not comparing anything.
#
# Usage (from anywhere; all paths derive from this script's own location, repo policy §12):
#   bash docs/tasks/artifacts/sv_exh_proof/ceiling_inertness_sweep.sh
#
# Prerequisite: the dual-feature binary —
#   (cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline)
set -uo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../../.." && pwd)"
bin="$repo_root/rust/target/debug/ast_pipeline"
# Project-owned scratch stays on the repository's own volume (repo policy §13).
out="$repo_root/rust/target/sv_exh_proof_7_4_6_13_ceiling/inertness_sweep"

if [[ ! -x "$bin" ]]; then
    echo "error: missing '$bin'" >&2
    echo "note: (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2
    exit 2
fi

rm -rf "$out"; mkdir -p "$out"

GRAMMARS=(json regex vhdl rtl_frontend ebnf semantic_annotation return_annotation
          systemverilog_preprocessor rtl_const_expr)
SEEDS=(0 7 42)

identical=0; diverged=0; nondet=0; refused=0
for grammar in "${GRAMMARS[@]}"; do
    for seed in "${SEEDS[@]}"; do
        off1="$out/${grammar}_${seed}_off1"
        on="$out/${grammar}_${seed}_on"
        off2="$out/${grammar}_${seed}_off2"

        PGEN_DEPTH_SLACK_CEILING_MULTIPLE=0 "$bin" "$repo_root/grammars/$grammar.ebnf" \
            --generate-stimuli --count 12 --seed "$seed" --output "$off1" >/dev/null 2>&1
        "$bin" "$repo_root/grammars/$grammar.ebnf" \
            --generate-stimuli --count 12 --seed "$seed" --output "$on" >/dev/null 2>&1
        PGEN_DEPTH_SLACK_CEILING_MULTIPLE=0 "$bin" "$repo_root/grammars/$grammar.ebnf" \
            --generate-stimuli --count 12 --seed "$seed" --output "$off2" >/dev/null 2>&1

        if [[ ! -s "$off1" || ! -s "$on" || ! -s "$off2" ]]; then
            printf "%-27s seed %-3s REFUSED — the generator produced no stimuli in one or more arms\n" \
                "$grammar" "$seed"
            refused=$((refused + 1)); continue
        fi
        if ! cmp -s "$off1" "$off2"; then
            printf "%-27s seed %-3s *** NON-DETERMINISTIC *** (OFF vs OFF differ)\n" "$grammar" "$seed"
            nondet=$((nondet + 1)); continue
        fi
        if cmp -s "$off1" "$on"; then
            printf "%-27s seed %-3s IDENTICAL (determinism control held)\n" "$grammar" "$seed"
            identical=$((identical + 1))
        else
            printf "%-27s seed %-3s *** DIVERGES — the ceiling is NOT inert here ***\n" "$grammar" "$seed"
            diverged=$((diverged + 1))
        fi
    done
done

echo
echo "identical=$identical diverged=$diverged non_deterministic=$nondet refused_no_output=$refused"
# Measured at PGEN-SV-EXH-PROOF-0185: identical=24 diverged=0 non_deterministic=0 refused_no_output=3
# (the 3 are rtl_const_expr, for the pre-existing reason in this header).
[[ $diverged -eq 0 && $nondet -eq 0 ]]
