#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2k (a)+(b) — build and measure ONE experimental parser arm, end to end.
#
# ⛔ WHY A DRIVER. An arm is only a measurement if the GRAMMAR, the GENERATED PARSER and the
# EXECUTABLE all moved together. `ENGINE-UNIVERSAL-SERVICES.20` slice 4 measured a probe built
# from a different arm than the one it named and the gate printed "the measurement cannot have
# moved" (14.7x off, in the passing direction); `CI-PARITY-GATE-ROT.32` then found that GNU Make
# 3.81 compares mtimes at WHOLE SECONDS, so a regeneration can be silently SKIPPED at exit 0.
# This driver therefore does the three steps in order, and the arm file it produces records the
# probe's own `--parser-fingerprint` so the parser inside the executable is never assumed.
#
# USAGE  bash docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/measure_arm.sh <arm>
#        <arm> ∈ {t_only, designB, designA}   (see apply_arm.py for what each one is)
# EXIT   0 = arm measured · non-zero = refused somewhere in the chain (the grammar is RESTORED)
set -euo pipefail

# ⚠️ NO BRACES in this message: the first `}` inside `${1:?…}` CLOSES the expansion, so a
# braced usage string appends a literal `}` to the value and the arm name arrives as
# `t_only}`. Caught by apply_arm.py's choice check rather than by reading.
ARM="${1:?usage: measure_arm.sh t_only|designB|designA}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
# ⛔ A ROOT GUARD, because this directory has now produced this exact bug twice: the
# `accepts_invalid_class` probe shipped one level short (`.13c.2k`) and so did the first cut
# of THIS driver. An unguarded wrong root fails as a confusing "No such file".
[ -d "$ROOT/grammars" ] || { echo "measure_arm: not at the repo root (derived $ROOT)" >&2; exit 2; }
cd "$ROOT"
HERE="docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms"
PROBE="rust/target/debug/parseability_probe"

cleanup() { python3 "$HERE/apply_arm.py" --restore || true; }
trap cleanup EXIT

echo "── [$ARM] 1/4  apply the grammar arm"
python3 "$HERE/apply_arm.py" --arm "$ARM"

echo "── [$ARM] 2/4  regenerate the SystemVerilog parser"
make -C rust SHELL=/bin/bash focus_systemverilog

PARSER_SHA="$(shasum -a 256 generated/systemverilog_parser.rs | awk '{print $1}')"
echo "── [$ARM]      generated parser sha256 = $PARSER_SHA"

echo "── [$ARM] 3/4  rebuild the debug probe (debug and release counters are byte-identical —"
echo "                 verified this slice at HEAD across all 1,077 rules)"
( cd rust && cargo build --features generated_parsers --bin parseability_probe )

echo "── [$ARM] 4/4  measure over the pinned 192-file sample"
python3 "$HERE/arm_cost.py" --measure --arm "$ARM" --probe "$PROBE" --jobs 6

echo "── [$ARM] done. Grammar restored to HEAD by the exit trap."
