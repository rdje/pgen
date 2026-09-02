#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/outcome_staleness_census/probe_b.sh
#
# SV-CORPUS-GRAD.13e.6(b) — the RED control behind the ruling that a grammar revision without a
# re-measure is BLOCKED, not merely loud: `CORPUS-CENSUS-CURRENCY` arm (3) refuses a report whose
# recorded grammar digest differs from the grammar in the tree, and that gate runs from
# `.githooks/pre-commit` on every commit for both SV lanes since `.13j`(c) / `.13m`.
#
# Driven through the gate's `PGEN_FAMILY_STATUS_SCHEMA` seam against a COPY of the real report with
# its grammar row zeroed — nothing tracked is mutated. Then the GREEN control on the real schema.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
W=rust/target/outcome_staleness_probe_b; rm -rf "$W"; mkdir -p "$W"
trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0
for fam in systemverilog_raw_parse systemverilog_v2005_raw_parse; do
  rep="$(python3 -c "import json;print(json.load(open('rust/test_data/grammar_quality/family_status_block_v0.json'))['corpus_census']['$fam']['report'])")"
  sed 's/^| grammar | `grammars\/systemverilog.ebnf` | `[0-9a-f]*` |/| grammar | `grammars\/systemverilog.ebnf` | `0000000000000000000000000000000000000000000000000000000000000000` |/' "$rep" > "$W/$fam.md"
  python3 - "$W" "$fam" <<'PY'
import json,sys
w,fam=sys.argv[1],sys.argv[2]
d=json.load(open("rust/test_data/grammar_quality/family_status_block_v0.json"))
e=dict(d["corpus_census"][fam]); e["report"]=f"{w}/{fam}.md"
json.dump({"corpus_census":{fam:e}}, open(f"{w}/{fam}.json","w"), indent=2)
PY
  out="$(PGEN_FAMILY_STATUS_SCHEMA="$W/$fam.json" bash scripts/check_corpus_census.sh "$fam" 2>&1)"; rc=$?
  if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'has MOVED since this census was measured'; then
    pass=$((pass+1)); printf 'PASS  RED   %-32s a moved grammar digest is a BREACH (rc=1)\n' "$fam"
  else
    fail=$((fail+1)); printf 'FAIL  RED   %-32s rc=%d\n%s\n' "$fam" "$rc" "$out"
  fi
  out="$(bash scripts/check_corpus_census.sh "$fam" 2>&1)"; rc=$?
  if [ "$rc" -eq 0 ]; then pass=$((pass+1)); printf 'PASS  GREEN %-32s the tracked report is fresh (rc=0)\n' "$fam"
  else fail=$((fail+1)); printf 'FAIL  GREEN %-32s rc=%d\n%s\n' "$fam" "$rc" "$out"; fi
done
grep -q 'check_doctrines.sh' .githooks/pre-commit && grep -q '"CORPUS-CENSUS-CURRENCY|' scripts/check_doctrines.sh \
  && { pass=$((pass+1)); echo "PASS  WIRED the gate is registered and the pre-commit hook runs the registry"; } \
  || { fail=$((fail+1)); echo "FAIL  WIRED the gate is not on the commit path"; }
printf 'OUTCOME-STALENESS-BLOCKS: %d/%d as declared\n' "$pass" "$((pass+fail))"
[ "$fail" -eq 0 ]
