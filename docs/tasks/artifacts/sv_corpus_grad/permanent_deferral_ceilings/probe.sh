#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/permanent_deferral_ceilings/probe.sh — SV-CORPUS-GRAD.13f's bank for the
# PERMANENT-deferral ceiling bound in corpus_verdict_coverage.py. Every RED arm asserts the refusal TEXT, not
# only rc 1: the first RED run of this bound returned rc 1 from a NameError, and only reading the message
# told a crash from a refusal. Scratch registers + scratch outdirs under rust/target; nothing tracked mutated.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
I=stimuli/sv/corpus_verdict_coverage.py; REG=docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/permanent_deferral_ceilings.json
W=rust/target/permanent_deferral_ceilings_probe; rm -rf "$W"; mkdir -p "$W"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0
arm(){ local label="$1" regf="$2" want_rc="$3" token="$4"; local out rc; out="$(python3 "$I" --ceilings "$regf" --outdir "$W/out_$pass$fail" 2>&1)"; rc=$?
  if [ "$rc" -eq "$want_rc" ] && printf '%s' "$out" | grep -qF -- "$token"; then pass=$((pass+1)); printf 'PASS  %-66s rc=%d\n' "$label" "$rc"; else fail=$((fail+1)); printf 'FAIL  %-66s rc=%d (want %d, token %q)\n%s\n' "$label" "$rc" "$want_rc" "$token" "$(printf '%s' "$out" | tail -3)"; fi; }
python3 -c "import json;d=json.load(open('$REG'));d['ceilings']['deferred:impl_varying']['ceiling']=89;json.dump(d,open('$W/low.json','w'))"
arm "1  RED: impl_varying ceiling 89 < 90 rows -> REFUSE naming the class" "$W/low.json" 1 "above its recorded ceiling"
python3 -c "import json;d=json.load(open('$REG'));del d['ceilings']['deferred:ni_unimplemented'];json.dump(d,open('$W/missing.json','w'))"
arm "2  RED: a permanent class with no ceiling -> REFUSE" "$W/missing.json" 1 "records no ceiling"
printf 'not json' > "$W/bad.json"
arm "3  RED: an unreadable register -> REFUSE" "$W/bad.json" 1 "unreadable"
arm "4  GREEN: the tracked register -> rc 0" "$REG" 0 "wrote"
if cmp -s "$W/out_30/coverage.md" docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/coverage.md 2>/dev/null || cmp -s "$(ls -d $W/out_* | tail -1)/coverage.md" docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/coverage.md; then pass=$((pass+1)); echo "PASS  5  GREEN: the scratch coverage.md is byte-identical to the tracked one"; else fail=$((fail+1)); echo "FAIL  5  scratch vs tracked coverage.md differ"; fi
# 6: the bound is exactly TIGHT today — every permanent class sits AT its ceiling (a ceiling with slack is a bound about to be silently spent)
python3 - "$REG" <<'PY' && pass=$((pass+1)) || fail=$((fail+1))
import json, sys, csv
reg=json.load(open(sys.argv[1]))["ceilings"]
counts={}
for row in csv.DictReader(open("stimuli/sv/characterization/adjudication_manifest.tsv"), delimiter="\t"):
    counts[row["adjudication"]]=counts.get(row["adjudication"],0)+1
slack={c:int(v["ceiling"])-counts.get(c,0) for c,v in reg.items()}
ok=all(s==0 for s in slack.values())
print(("PASS  " if ok else "FAIL  ")+f"6  every permanent class sits AT its ceiling (slack={slack})")
sys.exit(0 if ok else 1)
PY
printf 'PERMANENT-DEFERRAL-CEILINGS: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
