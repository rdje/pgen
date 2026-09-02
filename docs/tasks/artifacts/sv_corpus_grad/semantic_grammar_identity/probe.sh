#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/semantic_grammar_identity/probe.sh
#
# SV-CORPUS-GRAD.13r — the RED/GREEN bank for `CORPUS-CENSUS-CURRENCY` arm (3) keyed on the SEMANTIC
# grammar digest, driven on the FOUNDING PAIR the leaf named:
#   `.13q` (03ca8d58) — one `#` comment: bytes moved, raw_ast unchanged  -> must be a NOTE, rc 0
#   `.13p` (c4b2fe1b) — a real grammar fix: raw_ast moved                -> must FAIL, rc 1
# ⛔ Every digest is DERIVED from the git blobs at run time (`git show <rev>:grammars/systemverilog.ebnf`
# through the ONE digester `check_baseline_identity.sh --digest`), never typed here — a bank that
# carries its own expected digests rots the day the digester's canonicalisation moves.
# ⛔ Nothing tracked is mutated: every arm is a report FIXTURE (a copy of the real SV report with its
# identity rows rewritten) named by a scratch schema through `PGEN_FAMILY_STATUS_SCHEMA`.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
FAM=systemverilog_raw_parse; GRAMMAR=grammars/systemverilog.ebnf
W=rust/target/semantic_grammar_identity_probe; rm -rf "$W"; mkdir -p "$W"
trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
dig() { bash scripts/check_baseline_identity.sh --digest "$1" "$2" | cut -d' ' -f1; }
blob() { git show "$1:$GRAMMAR" > "$W/g_$2.ebnf" && echo "$W/g_$2.ebnf"; }
Q0=$(blob 03ca8d58^ q0); Q1=$(blob 03ca8d58 q1); P0=$(blob c4b2fe1b^ p0)
[ -n "$Q0" ] && [ -n "$Q1" ] && [ -n "$P0" ] || { echo "REFUSE: could not read the founding blobs from git"; exit 2; }
LIVE_B=$(dig bytes "$GRAMMAR"); LIVE_S=$(dig ebnf_raw_ast "$GRAMMAR")
Q0_B=$(dig bytes "$Q0"); Q0_S=$(dig ebnf_raw_ast "$Q0"); Q1_B=$(dig bytes "$Q1"); Q1_S=$(dig ebnf_raw_ast "$Q1")
P0_S=$(dig ebnf_raw_ast "$P0")
[ -n "$LIVE_S" ] && [ -n "$Q0_S" ] && [ -n "$P0_S" ] || { echo "REFUSE: the semantic digester is unavailable (build rust/target/debug/ast_pipeline) — the bank cannot judge"; exit 2; }
echo "founding pair, derived: .13q bytes $Q0_B -> $Q1_B ; raw_ast ${Q0_S:0:12} -> ${Q1_S:0:12} | .13p raw_ast ${P0_S:0:12} -> ${Q0_S:0:12} | live bytes ${LIVE_B:0:12} raw_ast ${LIVE_S:0:12}"
# the pair must be what the leaf says it is, or the arms below prove nothing
[ "$Q0_B" != "$Q1_B" ] && [ "$Q0_S" = "$Q1_S" ] || { echo "REFUSE: .13q is not a bytes-only move in this history"; exit 2; }
[ "$P0_S" != "$Q0_S" ] || { echo "REFUSE: .13p did not move the semantic digest in this history"; exit 2; }

REPORT=$(python3 -c "import json;print(json.load(open('rust/test_data/grammar_quality/family_status_block_v0.json'))['corpus_census']['$FAM']['report'])")
fixture() { # fixture <name> <bytes-sha> <raw-ast-sha-or-NONE>
  python3 - "$REPORT" "$W/$1.md" "$2" "$3" <<'PY'
import re, sys
src, out, b, s = sys.argv[1:]
lines = open(src, encoding="utf-8").read().splitlines()
res = []
for l in lines:
    if l.startswith("| grammar |"):
        res.append(f"| grammar | `grammars/systemverilog.ebnf` | `{b}` |")
        if s != "NONE":
            res.append(f"| grammar raw ast | `grammars/systemverilog.ebnf` | `{s}` |")
    elif l.startswith("| grammar raw ast |"):
        continue  # replaced above (or dropped for NONE)
    else:
        res.append(l)
open(out, "w", encoding="utf-8").write("\n".join(res) + "\n")
PY
  python3 - "$W/$1.md" "$W/$1.json" "$FAM" <<'PY'
import json, sys
rep, out, fam = sys.argv[1:]
d = json.load(open("rust/test_data/grammar_quality/family_status_block_v0.json"))
e = dict(d["corpus_census"][fam]); e["report"] = rep
json.dump({"corpus_census": {fam: e}}, open(out, "w"), indent=2)
PY
}
pass=0; fail=0
arm() { # arm <label> <fixture> <expect-rc> <must-contain>
  local out rc; out="$(PGEN_FAMILY_STATUS_SCHEMA="$W/$2.json" bash scripts/check_corpus_census.sh "$FAM" 2>&1)"; rc=$?
  if [ "$rc" -eq "$3" ] && printf '%s' "$out" | grep -qF -- "$4"; then pass=$((pass+1)); printf 'PASS  %-62s rc=%d\n' "$1" "$rc"
  else fail=$((fail+1)); printf 'FAIL  %-62s rc=%d (want %d, token %q)\n%s\n' "$1" "$rc" "$3" "$4" "$(printf '%s' "$out" | grep -E 'NOTE|✗|breach' | head -3)"; fi
}
# 1  .13q shape: the report was measured at 03ca8d58^ bytes, the tree is at today's bytes; same language
fixture q "$Q0_B" "$LIVE_S";      arm "1  .13q comment-only: bytes moved, raw_ast equal -> NOTE, rc 0" q 0 "moved by BYTES only"
# 2  .13p shape: the report was measured against the pre-fix language
fixture p "$Q0_B" "$P0_S";        arm "2  .13p grammar fix: raw_ast moved -> FAIL" p 1 "semantically"
# 3  pre-.13r report (no semantic row), bytes moved -> the old HARD failure, unchanged
fixture n "$Q0_B" NONE;           arm "3  no semantic row + bytes moved -> FAIL (pre-.13r behaviour)" n 1 "the next re-measure stamps it"
# 4  a report whose two grammar rows name different files cannot describe one grammar
python3 - "$W/q.md" "$W/x.md" <<'PY'
import sys; s=open(sys.argv[1],encoding='utf-8').read().replace("| grammar raw ast | `grammars/systemverilog.ebnf` |","| grammar raw ast | `grammars/other.ebnf` |",1); open(sys.argv[2],'w',encoding='utf-8').write(s)
PY
python3 - "$W/x.md" "$W/x.json" "$FAM" <<'PY'
import json,sys; rep,out,fam=sys.argv[1:]; d=json.load(open("rust/test_data/grammar_quality/family_status_block_v0.json")); e=dict(d["corpus_census"][fam]); e["report"]=rep; json.dump({"corpus_census":{fam:e}},open(out,"w"),indent=2)
PY
arm "4  semantic row names a DIFFERENT file -> FAIL" x 1 "cannot describe one grammar"
# 5  GREEN: the real schema on the real tree
out="$(bash scripts/check_corpus_census.sh "$FAM" 2>&1)"; rc=$?
if [ "$rc" -eq 0 ]; then pass=$((pass+1)); printf 'PASS  %-62s rc=0\n' "5  GREEN control: the tracked report on the real schema"; else fail=$((fail+1)); printf 'FAIL  %-62s rc=%d\n' "5  GREEN control" "$rc"; fi
# 6  RED for the NOTE path itself: the same .13q fixture with the semantic row REMOVED must turn the NOTE into a breach
fixture qn "$Q0_B" NONE;          arm "6  the NOTE is EARNED by the row: remove it and .13q fails again" qn 1 "has MOVED"
printf 'SEMANTIC-GRAMMAR-IDENTITY: %d/%d as declared\n' "$pass" "$((pass+fail))"
[ "$fail" -eq 0 ]
