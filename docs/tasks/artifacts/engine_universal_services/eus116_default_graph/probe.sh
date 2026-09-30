#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.116` — WHICH GRAPH does a registry parse with NO entry requested run?
#
# Every generated parser carries two graphs: the fused bare-parse cascade graph (the speed campaign's
# product, entered when `bare_parse` is computed true by `parse()`) and the protocol graph (every
# diagnostic consumer, and `parse_from(entry)`, which pins `bare_parse = false`). The two are
# byte-identical in verdict and AST by contract, so a verdict sweep cannot tell them apart. They are
# NOT identical in what they allocate: the fused MTB graph builds values ONCE over the derivation tape,
# the protocol graph allocates a `ParseNode` per rule frame — so the arena node count
# (`PGEN_REPORT_PARSE_FOOTPRINT`) is a deterministic, in-process fingerprint of the graph that ran.
#
# The control: the SAME input parsed twice — once as an operator's default `--parse` (must be the fused
# graph), once under `PGEN_REPORT_MEMO_STATS=1` (which clears `bare_parse` and runs the protocol graph BY
# CONSTRUCTION, TOOLBOX.md §3.3b). Equal node counts mean the default parse IS the protocol parse —
# the `.112` slice-2 regression (`f04f5b807`, 2026-09-28: the derived registry template called
# `parse_full_from(GRAMMAR_ENTRY_RULE)` for every parse). Fewer nodes on the default arm means the
# fused graph ran.
#
# Usage:  bash <this script> <path-to-parseability_probe> [--levels N]
#   exit 0  every family's default parse reads FEWER arena nodes than its forced-protocol parse (fused)
#   exit 1  at least one family's default parse reads the SAME node count (the protocol graph)
#   exit 2  usage / a parse did not report its footprint
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
probe="${1:?usage: probe.sh <path-to-parseability_probe> [--levels N]}"; shift
levels=1024
while [ $# -gt 0 ]; do case "$1" in --levels) levels="$2"; shift 2;; *) echo "unknown argument: $1" >&2; exit 2;; esac; done
gen="docs/tasks/artifacts/engine_universal_services/eus79a_per_frame_census/gen_deep_input.py"
out="tmp/eus116"; mkdir -p "$out"

nodes_of() {  # $1 = env prefix (may be empty), $2 = family, $3 = file, $4 = profile-or-empty
  local envp="$1" fam="$2" f="$3" prof="$4" pa=()
  [ -n "$prof" ] && pa=(--profile "$prof")
  env $envp PGEN_REPORT_PARSE_FOOTPRINT=1 "$probe" --parse "$fam" "$f" "${pa[@]}" 2>&1 >/dev/null \
    | grep -o 'nodes=[0-9]* (' | head -1 | tr -dc '0-9'
}

fail=0; checked=0
for spec in "return_annotation:" "systemverilog:sv_2017"; do
  fam="${spec%%:*}"; prof="${spec##*:}"
  src="$(python3 "$gen" "$fam" "$levels" --out-dir "$out")" || { echo "DEFAULT-GRAPH: cannot generate the $fam input" >&2; exit 2; }
  d="$(nodes_of "" "$fam" "$src" "$prof")"
  p="$(nodes_of "PGEN_REPORT_MEMO_STATS=1" "$fam" "$src" "$prof")"
  if [ -z "$d" ] || [ -z "$p" ]; then echo "DEFAULT-GRAPH: $fam did not report a footprint (default='$d' protocol='$p')" >&2; exit 2; fi
  checked=$((checked + 1))
  if [ "$d" -lt "$p" ]; then verdict=FUSED; else verdict=PROTOCOL; fail=1; fi
  printf 'DEFAULT-GRAPH: family=%s levels=%s default_nodes=%s protocol_nodes=%s verdict=%s\n' "$fam" "$levels" "$d" "$p" "$verdict"
done
if [ "$fail" -eq 0 ]; then
  echo "DEFAULT-GRAPH: OK ($checked families: the default registry parse runs the fused graph — fewer arena nodes than the forced-protocol parse on the same input)"
else
  echo "DEFAULT-GRAPH: FAIL — a default parse read the SAME arena node count as the forced-protocol parse: it IS the protocol graph (ENGINE-UNIVERSAL-SERVICES.116)" >&2
fi
exit $fail
