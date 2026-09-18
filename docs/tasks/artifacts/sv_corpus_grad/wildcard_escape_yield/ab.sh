#!/bin/bash
# SV-CORPUS-GRAD.27 — the yield-site A/B: every probe here and every pin named for interface ports, nettypes or the wildcard
# escape, parsed under the interpreter with THREE grammars (the released HEAD, the first .27 candidate, the final) on the SV
# profiles; each row prints rc:sha8 per grammar and the relation of the final tree to the other two.
#   bash docs/tasks/artifacts/sv_corpus_grad/wildcard_escape_yield/ab.sh HEAD.ebnf FIRST.ebnf FINAL.ebnf ast_pipeline_bin OUT_DIR
# Reading: "fix=head fix=cand" = identity; "fix=head" alone = a tree the first candidate moved and the final restored;
# "fix=cand" alone = a mover (HEAD rejects); "FIX-DIFFERS-FROM-BOTH" = a mixed probe (HEAD rejects it, the final differs
# from the first candidate in its direction-less ports alone — classify with a JSON diff, as the leaf record did).
set -u
cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." || exit 1
H=$1; C=$2; F=$3; B=$4; O=$5; mkdir -p "$O"
D=docs/tasks/artifacts/sv_corpus_grad/wildcard_escape_yield/probes; R=stimuli/sv/adjudication_repros
run(){ f=$1; g=$2; tag=$3; prof=$4; n=$(basename "$f" .sv); out=$O/$n.$prof.$tag.json; "$B" "$g" --grammar-profile "$prof" --interpret-parse "$f" --interpret-parse-ast-json "$out" >/dev/null 2>&1; rc=$?; h=$( [ -s "$out" ] && shasum -a 256 "$out" | cut -c1-8 || echo none ); echo "$rc:$h"; }
printf "%-58s %-8s %-12s %-12s %-12s %s\n" file prof head cand fix relation
{ ls $R | grep -E 'interface_port|nettype|wildcard' | sed "s|^|$R/|"; ls $D/*.sv; } | while read -r f; do n=$(basename "$f"); case $n in *sv2023*) profs="sv_2023";; *) profs="sv_2017 sv_2023";; esac
  for prof in $profs; do HH=$(run "$f" "$H" head $prof); CC=$(run "$f" "$C" cand $prof); FF=$(run "$f" "$F" fix $prof); rel=""; [ "$FF" = "$HH" ] && rel="fix=head"; [ "$FF" = "$CC" ] && rel="$rel fix=cand"; [ -z "$rel" ] && rel="FIX-DIFFERS-FROM-BOTH"; printf "%-58s %-8s %-12s %-12s %-12s %s\n" "$n" $prof "$HH" "$CC" "$FF" "$rel"; done; done
