#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.6f — the SELF-REJECTION census across every registered family,
# using the instrument the repo ALREADY HAS.
#
# ⭐ PRIOR ART, found before anything was designed (`feedback_read_prior_art_before_designing`):
# `--directed-generation-goal duality_break` (STIMULI-SIGNOFF.4.4, goal G2) already hunts
# "generator-emitted-but-parser-REJECTED samples against the real generated parser", steers
# generation ADVERSARIALLY toward new rejection signatures, and shrinks each break. It is
# strictly stronger than the blind own-corpus sampling H.16.6c–H.16.6e used, it scores against
# the SHIPPED parser rather than the interpreter, and it is fast: 1 000 samples per family in
# ~4 s, 4 000 in ~16 s. ⇒ this leaf is WIRING, and the wiring is even cheaper than it looked.
#
# ⛔ A `0/0` ROW IS NOT A PASS. `rtl_const_expr` reports `rejected 0/0 unique_breaks=0`, which
# reads as clean; plain generation on that grammar exits with
# `Error: Stimuli generation depth exceeded max_depth=24 while expanding rule 'primary_expr'`.
# A generation failure is reported in the PASSING direction, so this script prints the sample
# COUNT beside every verdict and flags a zero-sample row explicitly.
#
# Read-only. usage: duality_break_sweep.sh [rounds] [samples_per_round]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && while [ ! -d .git ] && [ "$PWD" != / ]; do cd ..; done; pwd)"
cd "$ROOT" || exit 2
BIN="rust/target/debug/ast_pipeline"
[ -x "$BIN" ] || { echo "duality_break_sweep: REFUSED — no ast_pipeline at $BIN"; exit 2; }
R="${1:-5}"; S="${2:-200}"
OUT="rust/target/h1666f"; mkdir -p "$OUT"

echo "DUALITY-BREAK-SWEEP  rounds=$R samples_per_round=$S seed=0"
echo "  a row is the SHIPPED parser's verdict on stimuli the grammar's own generator emitted"
printf '  %-24s %-10s %-10s %-8s %s\n' family directed diverse breaks note
zero=0
for g in json regex ebnf return_annotation semantic_annotation vhdl rtl_const_expr rtl_frontend; do
  f="grammars/$g.ebnf"
  [ -f "$f" ] || { printf '  %-24s %s\n' "$g" "(no grammar)"; continue; }
  line=$("$BIN" "$f" --generate-stimuli --count 40 --seed 0 \
          --directed-generation-goal duality_break --directed-rounds "$R" \
          --directed-samples-per-round "$S" -o "$OUT/dg_$g.txt" 2>&1 \
        | grep -E '^DIRECTED-GENERATION' | head -1)
  d=$(printf '%s' "$line"  | sed -nE 's/.*directed rejected ([0-9]+\/[0-9]+).*/\1/p')
  b=$(printf '%s' "$line"  | sed -nE 's/.*unique_breaks=([0-9]+).*/\1/p')
  v=$(printf '%s' "$line"  | sed -nE 's/.*baseline rejected ([0-9]+\/[0-9]+).*/\1/p')
  note=""
  case "$d" in
    ""|0/0) note="⛔ ZERO SAMPLES — vacuous, not clean"; zero=$((zero+1)) ;;
    0/*)    note="clean" ;;
    *)      note="⚠️ SELF-REJECTS" ;;
  esac
  printf '  %-24s %-10s %-10s %-8s %s\n' "$g" "${d:-?}" "${v:-?}" "${b:-?}" "$note"
done
echo
echo "  ⛔ systemverilog / systemverilog_preprocessor are NOT swept here — they are the locked"
echo "     lane and their sweep is priced separately; their absence is a STATED gap, not a pass."
echo "DUALITY-BREAK-SWEEP: zero_sample_families=$zero"
