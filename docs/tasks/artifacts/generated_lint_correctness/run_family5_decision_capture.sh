#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_family5_decision_capture.sh
#
# GENERATED-LINT-CORRECTNESS.4 — regenerate `family5_decision_capture.txt`, the single log behind
# every number in the `.4` decision.
#
# ⭐ WHY A SCRIPT AND NOT A PASTED LOG. Two numbers in the first hand-run capture were WRONG and the
# script is what makes that impossible to repeat:
#   (1) the calibration rows extracted the census TABLE HEADER (which also contains the word
#       "BACKED") instead of the summary line, and printed blank;
#   (2) the calibration was run against the WORKING TREE, which by then already contained this
#       leaf's own acceptance checklist — i.e. the leaf was inflating its own justification by one
#       box. The calibration now runs in a PRISTINE `git worktree` at the given baseline commit.
#
# Usage: bash docs/tasks/artifacts/generated_lint_correctness/run_family5_decision_capture.sh [BASELINE]
#   BASELINE defaults to the commit this leaf was measured against.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"
BASELINE="${1:-2eed59b6}"
CENSUS="docs/tasks/artifacts/generated_lint_correctness/run_root_cause_box_census.sh"

SITE='[A-Za-z0-9_./-]+\.(rs|ebnf|sh|toml|json|py|pl):[0-9]+'
CONTRAST='[0-9][0-9,.]*[[:space:]]*(->|→|=>)[[:space:]]*[0-9]|ACCEPT[[:space:]]*(->|→)[[:space:]]*REJECT|REJECT[[:space:]]*(->|→)[[:space:]]*ACCEPT|byte-identical|diverge=|[-+][0-9]+\.[0-9]+%'
G1='CERTIFICATE-COVERAGE:|\[plannable-probe\]|rejected by post predicate|furthest_position=|witnessed_target=(true|false)|PGEN_CERT_COVERAGE_(DUMP_ALL|DEBUG_PROBES)|PGEN_REACH_PATH_DUMP|--report-certificate-coverage|--trace-rules|--dump-rule-call-counts|--lint-grammar|--parse-dump-ast'
G2OLD='self-time|call-graph attribution|call-graph samples|cargo flamegraph|flamegraph'
G2NEW="$G2OLD|/usr/bin/sample|\botool\b|\bspindump\b|\bfiltercalltree\b|\bITIMER_PROF\b|--dump-rule-outcome-counts"
G3='error\[E[0-9]{4}\]|could not compile'
G4='GENERATED-CLIPPY-CORRECTNESS:|clippy::[a-z_]{3,}|PGEN_CLIPPY_GENERATED_STRICT'
G5='git (ls-files|log -S|log --all -S|rev-list|fsck|reflog|diff-tree|merge-base|cat-file)|\bshellcheck\b|bash -n |sh -n |make -n |make --dry-run|\bE2BIG\b|\bENOSPC\b|\bEACCES\b|\bARG_MAX\b|guard\.[0-9]+\.marker|reason=(none|rss-budget|free-floor|disk-floor|timeout)'

# A PRISTINE worktree at BASELINE, carrying the NEW census tool + NEW enforcer but the OLD task
# corpus — so the corpus numbers describe what was there BEFORE this leaf edited anything.
WT="$ROOT/rust/target/glc4_baseline"
rm -rf "$WT"; git worktree prune
git worktree add --detach -q "$WT" "$BASELINE" || { echo "capture: could not create worktree" >&2; exit 1; }
mkdir -p "$WT/rust/target"
cp "$CENSUS" "$WT/$CENSUS"
cp "$ROOT/scripts/check_diagnosis_evidence.sh" "$WT/scripts/check_diagnosis_evidence.sh"
OLDSIG="$(git show "${BASELINE}:scripts/check_diagnosis_evidence.sh" | grep -E '^DIAGNOSIS_SIG=' | sed "s/^DIAGNOSIS_SIG='//; s/'$//")"

in_wt() { ( cd "$WT" && "$@" ); }
backs()   { in_wt "$CENSUS" --census --sig "$1" 2>&1 | grep 'BACKED (signature' | grep -oE '[0-9]+$'; }
admits()  { in_wt "$CENSUS" --census --sig "$OLDSIG" --alt "$1" 2>&1 | tail -1 | grep -oE 'admit [0-9]+' | grep -oE '[0-9]+'; }

printf 'GENERATED-LINT-CORRECTNESS.4 — decision capture\n'
printf 'baseline commit: %s   (%s)\n' "$BASELINE" "$(git log -1 --format=%cs "$BASELINE")"
printf 'All corpus numbers are measured in a PRISTINE worktree at that commit, so this leaf\n'
printf 'cannot inflate its own justification.\n'
printf '%s\n' '================================================================================'

printf '\n[1] BOX CENSUS — the rule BEFORE .4 vs AFTER .4, same corpus\n\n'
printf '  BEFORE (pre-.4 DIAGNOSIS_SIG):\n'
in_wt "$CENSUS" --census --sig "$OLDSIG" 2>&1 | tail -4 | sed 's/^/    /'
printf '\n  AFTER (shipped .4 DIAGNOSIS_SIG):\n'
in_wt "$CENSUS" --census 2>&1 | tail -4 | sed 's/^/    /'

printf '\n[2] PLACEMENT SPLIT — where the evidence actually is (pre-.4 rule)\n\n'
in_wt "$CENSUS" --placement --sig "$OLDSIG" 2>&1 | tail -3 | sed 's/^/  /'

printf '\n[3] THE CHARTER HYPOTHESIS, PRICED AGAINST THE CORPUS\n'
printf '    denominator = the 304 boxes the charter saw under the pre-.4 rule\n\n'
printf '  %-52s %3s of 304\n' "SITE alone (FORBIDDEN — 'cite a line number')" "$(admits "$SITE")"
printf '  %-52s %3s of 304\n' "CONTRAST alone"                                "$(admits "$CONTRAST")"
printf '  %-52s %3s of 304   <== THE CHARTER'"'"'S HYPOTHESIS\n' "SITE **AND** CONTRAST" \
  "$(admits "($SITE)(.|\n)*($CONTRAST)|($CONTRAST)(.|\n)*($SITE)")"
printf '  %-52s %3s of 304\n' "corrected macOS/native profiler vocabulary"    "$(admits "$G2NEW")"
printf '  %-52s %3s of 304\n' "ops/build-flow family (as shipped)"            "$(admits "$G5")"
printf '  %-52s %3s of 304\n' "bare 'grep'  (EXCLUDED as too loose)"          "$(admits '\bgrep\b')"

printf '\n[4] CALIBRATION — how many boxes each signature family backs\n'
printf '    NOTE: [3] counts how many CURRENTLY-UNBACKED boxes a candidate would NEWLY admit;\n'
printf '    [4] counts how many boxes a family backs IN TOTAL. They differ whenever a box is\n'
printf '    already backed by another family (why ops reads 5 in [3] and 7 in [4]).\n\n'
printf '  group 1 correctness ............................... %s\n' "$(backs "$G1")"
printf '  group 2 performance   BEFORE the correction ....... %s\n' "$(backs "$G2OLD")"
printf '  group 2 performance   AFTER  the correction ....... %s\n' "$(backs "$G2NEW")"
printf '  group 3 build-integrity ........................... %s\n' "$(backs "$G3")"
printf '  group 4 codegen-emission .......................... %s\n' "$(backs "$G4")"
printf '  group 5 ops/build-flow  (NEW in .4) ............... %s\n' "$(backs "$G5")"

printf '\n[5] WHAT THE UNBACKED BOXES ACTUALLY CONTAIN (descriptive probes, overlapping)\n\n'
in_wt "$CENSUS" --classify --sig "$OLDSIG" 2>&1 | tail -13 | sed 's/^/  /'

printf '\n[6] ROUTED TO .5 — acceptance-gate coverage of the OPS surface, all history\n\n'
tot=0; ops=0; opsonly=0
while read -r c; do
  files=$(git diff-tree --no-commit-id --name-only -r "$c"); code=0; o=0
  while IFS= read -r f; do [ -z "$f" ] && continue
    case "$f" in grammars/*.ebnf|rust/src/*|generated/*|rust/test_data/ast_shape_contract/*.json) code=1 ;; esac
    case "$f" in scripts/*|rust/scripts/*|rust/Makefile|Makefile|.githooks/*|.github/workflows/*|rust/build.rs) o=1 ;; esac
  done <<< "$files"
  tot=$((tot+1)); [ "$o" -eq 1 ] && ops=$((ops+1))
  [ "$o" -eq 1 ] && [ "$code" -eq 0 ] && opsonly=$((opsonly+1))
done < <(git log --format=%h "$BASELINE")
printf '  commits total ..................................... %d\n' "$tot"
printf '  touching the OPS surface .......................... %d\n' "$ops"
printf '  ... of those, gate SILENT (no code_changed path) ... %d  (%d%%)\n' \
  "$opsonly" "$((ops==0?0:opsonly*100/ops))"

git worktree remove --force "$WT" >/dev/null 2>&1; git worktree prune
exit 0
