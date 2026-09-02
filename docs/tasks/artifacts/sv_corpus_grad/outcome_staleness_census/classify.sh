#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/outcome_staleness_census/classify.sh
#
# SV-CORPUS-GRAD.13e.6(c), second leg — a STALE report is stale by BYTES; was it stale by MEANING?
#
# `census.sh` scores a grammar revision `stale` when the tracked report's recorded grammar digest
# (bytes) differs from the grammar beside it. A comment-only grammar edit produces exactly that
# reading while the LANGUAGE is unchanged — `.13r`'s class. So for every stale revision this asks
# the sharper question: does the SEMANTIC digest of the grammar at that revision equal the semantic
# digest of the grammar the report was actually measured against? The semantic digest is the EBNF
# frontend's own `raw_ast` envelope — what the code generator consumes — computed with the same
# `--emit-raw-ast-json` + canonical-JSON sha256 recipe `scripts/check_sv_contract_currency.sh`
# uses, so a comment cannot move it.
#
#   SAME_LANGUAGE       stale by bytes only: the report still described the language in the tree
#   DIFFERENT_LANGUAGE  the published numbers described a grammar accepting a DIFFERENT language
#
# ⛔ The MEASURED revision is recovered, not guessed: the report's recorded byte-sha is looked up
# against the byte-sha of the grammar blob at every revision in history, and the newest match is the
# revision the report was measured against. A recorded sha matching no blob is `UNKNOWN` and refused
# from classification rather than scored.
#
# Reads git objects and a debug `ast_pipeline` (with `ebnf_dual_run`); writes only under rust/target.
# Deterministic, ~35 s for 11 stale rows. Usage: classify.sh [<census.tsv>]  (default: re-run census)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
HERE=docs/tasks/artifacts/sv_corpus_grad/outcome_staleness_census
P=rust/target/debug/ast_pipeline
bash scripts/require_ast_pipeline_features.sh "$P" ebnf_dual_run >/dev/null 2>&1 || {
  echo "classify: REFUSE - $P is absent or lacks ebnf_dual_run; the semantic digest cannot be computed" >&2; exit 2; }
W=rust/target/outcome_staleness_classify; rm -rf "$W"; mkdir -p "$W"
TSV="${1:-}"; [ -n "$TSV" ] || { TSV="$W/census.tsv"; bash "$HERE/census.sh" > "$TSV" 2>/dev/null; }

semdig() { # $1 = revision -> 12-hex semantic digest of the grammar at that revision, or LOAD_FAIL
  git show "$1:grammars/systemverilog.ebnf" > "$W/g.ebnf" 2>/dev/null || { echo NO_BLOB; return; }
  "$P" "$W/g.ebnf" --emit-raw-ast-json "$W/raw.json" >/dev/null 2>&1 || { echo LOAD_FAIL; return; }
  python3 -c "import json,hashlib;print(hashlib.sha256(json.dumps(json.load(open('$W/raw.json'))['raw_ast'],sort_keys=True,separators=(',',':')).encode()).hexdigest()[:12])"
}

declare -A REV_OF_SHA
while IFS=$'\t' read -r rev date verdict rec act touched; do
  [ "$rev" = rev ] && continue
  REV_OF_SHA["$act"]="${REV_OF_SHA[$act]:-$rev}"      # newest first ⇒ first seen = newest
done < "$TSV"

same=0; diff=0; unknown=0
printf 'rev\tdate\tmeasured_rev\tsemantic_at_rev\tsemantic_at_measured\tverdict\n'
while IFS=$'\t' read -r rev date verdict rec act touched; do
  [ "$verdict" = stale ] || continue
  mrev="${REV_OF_SHA[$rec]:-UNKNOWN}"
  if [ "$mrev" = UNKNOWN ]; then unknown=$((unknown+1)); printf '%s\t%s\tUNKNOWN\t-\t-\tUNCLASSIFIED\n' "$rev" "$date"; continue; fi
  s_rev="$(semdig "$rev")"; s_m="$(semdig "$mrev")"
  if [ "$s_rev" = "$s_m" ]; then cls=SAME_LANGUAGE; same=$((same+1)); else cls=DIFFERENT_LANGUAGE; diff=$((diff+1)); fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$rev" "$date" "$mrev" "$s_rev" "$s_m" "$cls"
done < "$TSV"
printf 'OUTCOME-STALENESS-CLASS: stale=%d different_language=%d same_language=%d unclassified=%d\n' \
  "$((same+diff+unknown))" "$diff" "$same" "$unknown" >&2
