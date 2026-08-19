#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2s — the probe matrix that chose the fix, and the arms it chose between.
#
# ⛔ WHY IT IS TRACKED. `docs/CLAIM_VERIFICATION.md` leg 3: a measured number whose instrument lives
# in a scratch directory is a "trust me" with extra steps. Every verdict this leaf published — the
# defect, the seven legal forms, the four illegal ones, and the finding that three candidate
# spellings are behaviourally IDENTICAL — comes from this script.
#
#   bash docs/tasks/artifacts/sv_corpus_grad/systf_clocking_event/probe_matrix.sh
#
# With no argument it probes the SHIPPED grammar. Pass grammar paths to probe arms instead:
#   bash .../probe_matrix.sh armA.ebnf armB.ebnf armC.ebnf
#
# Exit 0 = every expectation held. Exit 1 = one did not.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -f grammars/systemverilog.ebnf ] || { echo "probe: not at the repo root ($ROOT)" >&2; exit 2; }

PIPELINE=rust/target/debug/ast_pipeline
[ -x "$PIPELINE" ] || { echo "probe: $PIPELINE is not built (TOOLBOX.md §1.5b)" >&2; exit 2; }
WORK="$(mktemp -d "${TMPDIR:-/tmp}/systf_probe.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
PASS=0; FAIL=0

# name|expected|body   — `expected` is the verdict IEEE 1800-2017 A.8.2 / A.2.10 require.
CASES=(
  'clocking_event|ACCEPT|$rose(a, @(posedge clk))'
  'plain_call|ACCEPT|$rose(a)'
  'two_expressions|ACCEPT|$rose(a, b)'
  'empty_then_clocking|ACCEPT|$rose(a, , @(posedge clk))'
  'trailing_empty|ACCEPT|$rose(a, )'
  'past_full_form|ACCEPT|$past(a, 2, en, @(posedge clk))'
  'clocking_identifier|ACCEPT|$rose(a, @clk)'
  'two_empty_commas|ACCEPT|$rose(a, , )'
  'clocking_not_last|REJECT|$rose(a, @(posedge clk), b)'
  'at_as_first_arg|REJECT|$rose(@x)'
  'bare_at|REJECT|$rose(a, @)'
  'two_clocking_events|REJECT|$rose(a, @(posedge clk), @(posedge clk))'
)

probe() {  # $1 = grammar, $2 = file
  "$PIPELINE" "$1" --interpret-parse "$2" --grammar-profile sv_2017 >/dev/null 2>&1 \
    && echo ACCEPT || echo REJECT
}

for grammar in "${@:-grammars/systemverilog.ebnf}"; do
  echo "── $grammar"
  for spec in "${CASES[@]}"; do
    name="${spec%%|*}"; rest="${spec#*|}"; want="${rest%%|*}"; body="${rest#*|}"
    f="$WORK/$name.sv"
    { echo 'module m;'
      echo '  logic clk, a, b, en, x;'
      echo "  always @(posedge clk) x <= $body;"
      echo 'endmodule'; } > "$f"
    got="$(probe "$grammar" "$f")"
    if [ "$got" = "$want" ]; then
      printf '  ✓ %-22s %-6s  %s\n' "$name" "$got" "$body"; PASS=$((PASS+1))
    else
      printf '  ✗ %-22s want %s got %s  %s\n' "$name" "$want" "$got" "$body"; FAIL=$((FAIL+1))
    fi
  done
done

echo "SYSTF-CLOCKING-PROBE: passed=$PASS failed=$FAIL"
[ "$FAIL" = 0 ] || exit 1
