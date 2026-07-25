#!/usr/bin/env bash
# SV-CORPUS-GRAD.3.10 — the §5.7.1 spaced-number-literal repro matrix.
#
# Regenerates every probe row for the leaf from one place so the BEFORE and AFTER
# artifacts are produced by an IDENTICAL driver (the `.3.9` discipline: a repro
# matrix that is re-run, not re-typed).
#
# Usage:  bash run_matrix.sh <out.txt>
# Probe:  rust/target/release/parseability_probe (release; asserted newer than the
#         generated parser by the caller).
set -uo pipefail

REPO=/Volumes/SSD/Documents/github/pgen
PROBE="$REPO/rust/target/release/parseability_probe"
OUT="${1:?usage: run_matrix.sh <out.txt>}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# ---------------------------------------------------------------------------
# wrappers — the two consumer rules of the 12 fused number-literal tokens
# ---------------------------------------------------------------------------
# init_val  : IEEE 1800-2017 A.5.2  udp_initial_statement / sequential_body
udp_sample () { # $1 = the init_val spelling
  cat <<EOF
primitive p (q, clk, d);
  output q;
  reg q;
  input clk, d;
  initial q = $1;
  table
    r 0 : ? : 0 ;
  endtable
endprimitive
EOF
}
# scalar_constant : IEEE 1800-2017 A.7.5.3, reached via a timing-check condition
spec_sample () { # $1 = the scalar_constant spelling
  cat <<EOF
module m (input clk, input d, input cond);
  specify
    \$setup(posedge clk &&& cond == $1, d, 10);
  endspecify
endmodule
EOF
}

verdict () { # $1 = file, $2 = profile -> ACCEPT | REJECT
  if "$PROBE" --parse systemverilog "$1" --profile "$2" >/dev/null 2>&1; then
    echo ACCEPT
  else
    echo REJECT
  fi
}

# scalar_timing_check_condition kind, as EMITTED (proves the typed shape, which is
# the `scalar_constant` face of this defect — it degrades silently, it never rejects)
cond_kind () { # $1 = file
  local dump="$WORK/ast.json"
  "$PROBE" --parse-dump-ast systemverilog "$1" "$dump" --profile sv_2017 >/dev/null 2>&1 || { echo "<reject>"; return; }
  python3 - "$dump" <<'PY'
import json, sys
found = []
def walk(n):
    if isinstance(n, dict):
        if n.get('kind') in ('eq', 'case_eq', 'ne', 'case_ne') and 'rhs' in n:
            found.append("%s/rhs=%s" % (n['kind'], json.dumps(n['rhs'], separators=(',', ':'))))
        elif n.get('kind') == 'expression' and 'body' in n and 'lhs' not in n:
            found.append('expression')
        for v in n.values():
            walk(v)
    elif isinstance(n, list):
        for v in n:
            walk(v)
walk(json.load(open(sys.argv[1])))
print(found[0] if found else '<none>')
PY
}

row () { # $1 = id, $2 = wrapper(udp|spec), $3 = spelling, $4 = expectation note
  local f="$WORK/$1.sv"
  case "$2" in
    udp)  udp_sample  "$3" > "$f" ;;
    spec) spec_sample "$3" > "$f" ;;
  esac
  printf '  %-6s %-14s %-16s %-7s %-8s %s\n' \
    "$1" "$2" "$(printf '%q' "$3")" \
    "$(verdict "$f" sv_2017)" "$(verdict "$f" verilog_2005)" "$4"
}

{
  echo "# SV-CORPUS-GRAD.3.10 — §5.7.1 spaced number-literal repro matrix"
  echo "# probe: $PROBE"
  echo "# probe mtime : $(date -r "$PROBE" '+%Y-%m-%d %H:%M:%S')"
  echo "# parser mtime: $(date -r "$REPO/generated/systemverilog_parser.rs" '+%Y-%m-%d %H:%M:%S')"
  echo "# grammar sha : $(shasum -a 256 "$REPO/grammars/systemverilog.ebnf" | cut -c1-16)"
  echo
  echo "IEEE 1800-2017 §5.7.1, verbatim:"
  echo '  "The apostrophe character and the base format character shall not be'
  echo '   separated by any white space."                     <- seam CLOSED'
  echo '  "The unsigned number token shall immediately follow the base format,'
  echo '   optionally preceded by white space."               <- seam OPEN'
  echo '  a based literal "shall be composed of up to three tokens" (size /'
  echo '  base_format / value) => §5.3 free-form separation applies to the'
  echo '  size<->apostrophe seam.                             <- seam OPEN'
  echo
  echo "================================================================================"
  echo "MUST ACCEPT — the LRM-legal spellings (A2..A11 are the defect: REJECT before)"
  echo "================================================================================"
  printf '  %-6s %-14s %-16s %-7s %-8s %s\n' id consumer spelling sv_2017 v2005 note
  row A1  udp  "1'b1"      "control — tight, ACCEPTs at baseline"
  row A2  udp  "1 'b 1"    "DEFECT — both seams spaced"
  row A3  udp  "1 'b1"     "DEFECT — size seam only"
  row A4  udp  "1'b 1"     "DEFECT — value seam only"
  row A5  udp  "1	'b	0"  "DEFECT — tabs at both seams"
  row A6  udp  "1 'B 1"    "DEFECT — uppercase base"
  row A7  udp  "1 'b x"    "DEFECT — x value (string-literal token)"
  row A8  udp  "1 'b X"    "DEFECT — X value (string-literal token)"
  row A9  udp  "1  'b  0"  "DEFECT — multi-space"
  row A10 udp  "1"         "control — bare 1, LRM alt 9"
  row A11 udp  "0"         "control — bare 0, LRM alt 10"
  row A12 spec "1'b1"      "control — tight scalar_constant"
  row A13 spec "1 'b 1"    "DEFECT (shape) — accepts, but typed shape lost"
  row A14 spec "'b1"       "control — unsized tight"
  row A15 spec "'b 1"      "DEFECT (shape) — unsized, value seam"
  row A16 spec "'B 0"      "DEFECT (shape) — unsized uppercase"
  echo
  echo "================================================================================"
  echo "MUST REJECT — the closed set must STAY closed (over-permissiveness guard)"
  echo "================================================================================"
  printf '  %-6s %-14s %-16s %-7s %-8s %s\n' id consumer spelling sv_2017 v2005 note
  row R1  udp  "2'b1"      "size != 1 — NOT in the LRM closed set"
  row R2  udp  "1' b 1"    "§5.7.1 PROHIBITS ws between ' and base char"
  row R3  udp  "1 ' b 1"   "same prohibited seam, both sides spaced"
  row R4  udp  "1'b2"      "value 2 — not a binary digit"
  row R5  udp  "1'bz"      "z — NOT in init_val's closed set"
  row R6  udp  "1'b"       "digit-less (the SV-0030 defect must stay fixed)"
  row R7  udp  "1 'b"      "digit-less + spaced"
  row R8  udp  "1'b 11"    "2-bit value — not the closed 1-bit set"
  row R9  udp  "'b1"       "unsized — NOT an init_val alternative (scalar_constant only)"
  echo
  echo "================================================================================"
  echo "TYPED-SHAPE PROOF — scalar_timing_check_condition kind as EMITTED"
  echo "================================================================================"
  echo "# 'eq/rhs={\"kind\":\"1'b1\"}' = the typed compare shape reached scalar_constant."
  echo "# 'expression'              = scalar_constant FAILED and the flat fallback won."
  for spelling in "1'b1" "1 'b 1" "'b1" "'b 1"; do
    spec_sample "$spelling" > "$WORK/shape.sv"
    printf '  %-16s -> %s\n' "$(printf '%q' "$spelling")" "$(cond_kind "$WORK/shape.sv")"
  done
  echo
  echo "================================================================================"
  echo "KNOWN DEFERRED GAP (inherited from .3.5 / SV-0041, deliberately NOT closed here)"
  echo "================================================================================"
  row D1  udp  "1
'b 1"       "newline at the size seam — §5.3-legal, deferred"
} > "$OUT"

cat "$OUT"
