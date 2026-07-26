#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.1 — which productions of PGEN's OWN meta-grammar are
# declared but unreachable? Read-only; debug `ast_pipeline` + the shared closure probe.
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$REPO" || exit 1
OUT="$(mktemp -d)"; trap 'rm -rf "$OUT"' EXIT
./rust/target/debug/ast_pipeline grammars/ebnf.ebnf --generate-parser \
  --dump-gen-ast "$OUT/ebnf_gen.json" --eliminate-left-recursion --output "$OUT/p.rs" >/dev/null 2>&1
echo "=== --lint-grammar verdict ==="
./rust/target/debug/ast_pipeline grammars/ebnf.ebnf --lint-grammar 2>&1 \
  | grep -oE 'unreachable_rules=[0-9]+' | sed 's/^/  lint says: /'
echo
echo "=== independent reachability closure from the entry rule ==="
OUT="$OUT" python3 - <<'PY'
import json, os, sys
sys.path.insert(0, 'docs/tasks/artifacts/lex_adjacency')
from closure_probe import closure
d = json.load(open(os.environ['OUT'] + '/ebnf_gen.json'))
gt = d['grammar_tree']; entry = (d.get('rule_order') or ['grammar_file'])[0]
reach = closure(gt, entry)
orphans = sorted(set(gt) - reach)
print(f"  entry='{entry}'  rules={len(gt)}  reachable={len(reach)}  UNREACHABLE={len(orphans)}")
for o in orphans:
    print("    -", o)
PY
