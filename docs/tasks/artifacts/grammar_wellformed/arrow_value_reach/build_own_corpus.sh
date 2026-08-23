#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.6e — build the OWN-CORPUS sample at a named grammar vintage.
#
# `H.16.6c` established that "the grammar self-rejects N of its own stimuli" has TWO readings,
# and that the OWN-CORPUS one (score each grammar against stimuli IT generated) is the only
# reading under which a NARROWING fix is not charged for rows its own generator can no longer
# emit. This builds that corpus, deterministically, from a NAMED seed set.
#
# usage: build_own_corpus.sh <grammar.ebnf> <out.txt> [count] [seed ...]
set -eu
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && while [ ! -d .git ] && [ "$PWD" != / ]; do cd ..; done; pwd)"
BIN="$ROOT/rust/target/debug/ast_pipeline"
G="$1"; OUT="$2"; COUNT="${3:-200}"; shift 3 || shift $#
SEEDS=("$@")
[ ${#SEEDS[@]} -gt 0 ] || SEEDS=(0 7 42 123 999 1 2 3 5 11 13 17 23 31 37 41 53 61 71 89 97 \
                                 101 103 107 109 113 127 131 137 139 149 151 157 163 167 173 179 181 191 193)
[ -x "$BIN" ] || { echo "build_own_corpus: REFUSED — no ast_pipeline at $BIN"; exit 2; }
[ -f "$G" ]   || { echo "build_own_corpus: REFUSED — no grammar at $G"; exit 2; }
TMPD="$ROOT/rust/target/h1666e/corpus_$(basename "$OUT" .txt)"
rm -rf "$TMPD"; mkdir -p "$TMPD" "$(dirname "$OUT")"
for s in "${SEEDS[@]}"; do
  "$BIN" "$G" --generate-stimuli --count "$COUNT" --seed "$s" -o "$TMPD/s_$s.txt" >/dev/null
done
cat "$TMPD"/s_*.txt | grep -v '^[[:space:]]*$' | awk '!seen[$0]++' > "$OUT"
n=$(wc -l < "$OUT")
echo "OWN-CORPUS: grammar=$(basename "$G") seeds=${#SEEDS[@]} count=$COUNT unique=$n -> $OUT"
# ⛔ The DENSITY control: a sample with no arrows in it proves nothing about arrows.
echo "  density: rows_with_arrow=$(grep -c '=>' "$OUT") rows_with_two_arrows=$(grep -cE '=>.*=>' "$OUT")" \
     "rows_with_chained_arrow=$(grep -cE '=>[^,]*=>' "$OUT")"
