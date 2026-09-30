#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.97` — the JSON-pointer pin, RED on the 1.0.214 tree FOREVER, GREEN on HEAD.
#
# The corruption this leaf was opened on carries every `kind` the correct tree carries, in the wrong node:
#   1.0.214  scope: {name: {kind: "dot"}, scope: {kind: "this"}[, select: "<invalid_sequence_access>"]}
#   1.0.215+ scope: [{kind: "this"}, {kind: "dot"}]
# so a `kind`-chain `arm` and the contract's root-key claims hold on BOTH. A JSON pointer names WHERE a
# value must sit: `/scope/0/kind = "this"`, `/scope/1/kind = "dot"`, and `/scope/select` MUST NOT resolve.
#
# This probe dumps the typed tree of each control input at its census rule through the grammar-AST
# INTERPRETER (TOOLBOX §1.5b — byte-identical to the generated parser by §1.6, and it reads any grammar
# revision straight from git, so the RED arm needs no historical binary), once on the 1.0.214 grammar
# (`git show 15de0cd1:grammars/systemverilog.ebnf`) and once on HEAD's, and evaluates the SAME claims
# the tracked manifest samples spell (`rust/test_data/ast_shape_contract/systemverilog_v1.json`,
# samples `eus97_*`) with a minimal RFC 6901 mirror of `check_json_pointer_claims`.
#
# Usage:  bash <this script> [<path-to-debug-ast_pipeline>]
#   exit 0  every claim FAILS on 1.0.214 (RED) and HOLDS on HEAD (GREEN), for every control
#   exit 1  a control did not read as pre-registered
#   exit 2  usage / a dump failed
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
ap="${1:-rust/target/debug/ast_pipeline}"
[ -x "$ap" ] || { echo "JSON-POINTER-PIN: no ast_pipeline at $ap (build: cd rust && cargo build --features \"generated_parsers ebnf_dual_run\")" >&2; exit 2; }
out="tmp/eus97"; mkdir -p "$out"
git show 15de0cd1:grammars/systemverilog.ebnf > "$out/sv_1_0_214.ebnf" || exit 2
printf 'this.arr' > "$out/this_arr.txt"; printf 'this.x' > "$out/this_x.txt"

dump() {  # $1 grammar, $2 rule, $3 input, $4 out.json
  "$ap" "$1" --interpret-parse "$3" --interpret-entry-rule "$2" --interpret-parse-ast-json "$4" --grammar-profile sv_2017 > "$4.log" 2>&1 \
    && grep -q 'accepted=true' "$4.log"
}

fail=0; checked=0
# rule | input | claims (pointer=json ; !pointer = must be absent)
while IFS='|' read -r rule inp claims; do
  [ -z "$rule" ] && continue
  for arm in old head; do
    g="$out/sv_1_0_214.ebnf"; [ "$arm" = head ] && g="grammars/systemverilog.ebnf"
    j="$out/${arm}_${rule}.json"
    dump "$g" "$rule" "$out/$inp.txt" "$j" || { echo "JSON-POINTER-PIN: dump failed: arm=$arm rule=$rule ($(tail -1 "$j.log"))" >&2; exit 2; }
  done
  verdict="$(python3 - "$out/old_${rule}.json" "$out/head_${rule}.json" "$claims" <<'PY'
import json, sys
def resolve(v, ptr):
    if ptr == "": return True, v
    for tok in ptr.split("/")[1:]:
        tok = tok.replace("~1", "/").replace("~0", "~")
        if isinstance(v, dict) and tok in v: v = v[tok]
        elif isinstance(v, list) and tok.isdigit() and int(tok) < len(v): v = v[int(tok)]
        else: return False, None
    return True, v
def failures(tree, claims):
    out = []
    for c in claims.split(";"):
        c = c.strip()
        if not c: continue
        if c.startswith("!"):
            ok, _ = resolve(tree, c[1:])
            if ok: out.append(f"{c[1:]} must not resolve")
        else:
            ptr, exp = c.split("=", 1)
            ok, got = resolve(tree, ptr)
            if not ok: out.append(f"{ptr} does not resolve")
            elif got != json.loads(exp): out.append(f"{ptr} = {json.dumps(got)} != {exp}")
    return out
old = json.load(open(sys.argv[1]))["content"]["Json"]; head = json.load(open(sys.argv[2]))["content"]["Json"]
r = failures(old, sys.argv[3]); g = failures(head, sys.argv[3])
print(f"red_failures={len(r)} green_failures={len(g)} red_detail={'; '.join(r)}")
PY
)"
  checked=$((checked + 1))
  rf="$(printf '%s' "$verdict" | grep -o 'red_failures=[0-9]*' | tr -dc '0-9')"; gf="$(printf '%s' "$verdict" | grep -o 'green_failures=[0-9]*' | tr -dc '0-9')"
  if [ "$rf" -gt 0 ] && [ "$gf" -eq 0 ]; then st="OK (RED on 1.0.214, GREEN on HEAD)"; else st="NOT AS PRE-REGISTERED"; fail=1; fi
  echo "JSON-POINTER-PIN: rule=$rule input=$inp $verdict :: $st"
done <<'ROWS'
ps_or_hierarchical_array_identifier|this_arr|/scope/0/kind="this";/scope/1/kind="dot";/name/name/body="arr";!/scope/select;!/scope/name
nonrange_variable_lvalue|this_arr|/scope/0/kind="this";/scope/1/kind="dot";!/scope/select;!/scope/name
constraint_primary|this_x|/body/scope/0/kind="this";/body/scope/1/kind="dot";/body/name/name/body="x";!/body/scope/select;!/body/scope/name
ROWS
if [ "$fail" -eq 0 ]; then echo "JSON-POINTER-PIN: OK ($checked controls: every claim fails on the 1.0.214 tree and holds on HEAD)"; else echo "JSON-POINTER-PIN: FAIL" >&2; fi
exit $fail
