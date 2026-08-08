#!/usr/bin/env bash
# stimuli/run_external_corpus.sh <sv|vhdl> [per_file_timeout_s] [jobs] [max_files]
#
# EXTERNAL-CORPUS.3.1 / .3.2 — bulk characterization of the external TEST CORPORA
# vendored under stimuli/<fam>/subs/ (git submodules, sparse-checked-out to their
# test dirs). Parses EVERY corpus file with the PGEN parser and tallies
# parse-pass / parse-fail / timeout per sub-corpus, then writes a dated
# characterization report.
#
# Doctrine (characterize, don't game — EXTERNAL-CORPUS / project_external_corpus_doctrine):
#   - The number reported is the HONEST raw parse outcome, not a forced "conformance pass".
#   - Many corpus files are INTENTIONALLY invalid (e.g. GHDL `gna` bug-regressions,
#     VESTS `non_compliant/`, sv-tests files with `:should_fail_because:`), so a
#     parse-FAIL is often the CORRECT outcome, not a parser bug. This runner reports
#     raw outcomes per sub-corpus; deeper expected-vs-actual adjudication is a follow-up.
#   - These corpora exercise the FULL grammars (not simplified subsets), so genuine
#     divergences are real parser gaps — a bug-finding oracle.
#
# Usage:
#   stimuli/run_external_corpus.sh sv            # all SV corpora, 20s/file, 8 jobs
#   stimuli/run_external_corpus.sh vhdl 30 6     # VHDL, 30s/file, 6 jobs
#   stimuli/run_external_corpus.sh sv 20 8 500   # cap at 500 files (smoke)
set -uo pipefail

FAM="${1:?usage: run_external_corpus.sh <sv|sv2005|vhdl> [timeout_s] [jobs] [max_files]}"
TIMEOUT_S="${2:-20}"
JOBS="${3:-8}"
MAX_FILES="${4:-0}"   # 0 = no cap

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROBE="${PGEN_PARSE_PROBE_BIN:-$ROOT/rust/target/debug/parseability_probe}"

ROOT_EARLY="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
case "$FAM" in
  sv)   GRAMMAR=systemverilog; export PROFILE_ARGS="--profile sv_2017";
        FIND_EXTS=( -name '*.sv' -o -name '*.svh' -o -name '*.v' );
        # SV-CORPUS-GRAD.8: fold the pre-submodule-vintage uvm-core vendoring
        # (plain tracked files) into the bulk universe as sub-corpus `uvm-core`.
        EXTRA_DIRS=( "$ROOT_EARLY/stimuli/sv/uvm" );;
  # SV-CORPUS-GRAD.8c: the verilog_2005 profile lane — re-parses ONLY the
  # adjudicator-derived lane files (single source of lane membership:
  # `v2005_lane_files.tsv`, emitted by adjudicate_external_corpus.py) under
  # the strict verilog_2005 profile.
  sv2005) GRAMMAR=systemverilog; export PROFILE_ARGS="--profile verilog_2005";
        FIND_EXTS=( -name '*.v' );  # unused (list-driven)
        EXTRA_DIRS=();
        LANE_LIST="$ROOT_EARLY/stimuli/sv/characterization/v2005_lane_files.tsv";;
  vhdl) GRAMMAR=vhdl;          export PROFILE_ARGS="";
        FIND_EXTS=( -name '*.vhd' -o -name '*.vhdl' );
        EXTRA_DIRS=();;
  *) echo "unknown family '$FAM' (expected sv|sv2005|vhdl)" >&2; exit 2;;
esac

[ -x "$PROBE" ] || { echo "parseability_probe not found/executable at $PROBE" >&2; exit 3; }

if [ "$FAM" = sv2005 ]; then
  SUBS="$ROOT/stimuli/sv/subs"
  OUTDIR="$ROOT/stimuli/sv/characterization"
  RESULTS="$OUTDIR/results_v2005.tsv"
  REPORT="$OUTDIR/characterization_v2005.md"
else
  SUBS="$ROOT/stimuli/$FAM/subs"
  OUTDIR="$ROOT/stimuli/$FAM/characterization"
  RESULTS="$OUTDIR/results.tsv"
  REPORT="$OUTDIR/characterization.md"
fi
mkdir -p "$OUTDIR"
: > "$RESULTS"

export PROBE GRAMMAR TIMEOUT_S

# Parse a single file; emit "<subcorpus>\t<status>\t<repo-root-relative path>".
#
# CORPUS-GRAD-ALL.2.1 — column 3 is REPO-ROOT-RELATIVE, never absolute. An absolute path bakes
# the checkout root into the artifact, which made the tracked VHDL characterization report
# unreproducible: every one of its 13 720 rows pointed at a DIFFERENT clone (a home-directory copy
# on another volume), so the report could be neither re-run nor diffed against a later one, and it
# breached both the repo-root-relative-paths rule and the same-volume data-locality policy. A
# relative column makes the artifact portable and diffable, which is the precondition for TRACKING
# it — an untracked measurement is the staleness defect class
# (project_all_parsers_fully_pass_stimuli_and_external_corpora clause 4).
#
# Backward-compatible with the consumers BY CONSTRUCTION: both `adjudicate_external_corpus.py` and
# the stuck-point clusterer key on the `/subs/<suite>/` (or `/stimuli/sv/uvm/`) INFIX and split
# there, which a relative path still contains — verified before changing this.
parse_one() {
  local f="$1" sub rc status rel
  sub="$(printf '%s' "$f" | sed -E 's#.*/subs/([^/]+)/.*#\1#; s#.*/stimuli/sv/uvm/.*#uvm-core#')"
  rel="${f#"$ROOT"/}"
  # shellcheck disable=SC2086
  timeout "$TIMEOUT_S" "$PROBE" --parse "$GRAMMAR" "$f" $PROFILE_ARGS >/dev/null 2>&1
  rc=$?
  if [ "$rc" -eq 0 ]; then status=pass
  elif [ "$rc" -eq 124 ]; then status=timeout
  elif [ "$rc" -ge 128 ]; then status=crash   # signal death (e.g. stack
                                              # overflow abort) is NOT a
                                              # graceful reject - own status
  else status=fail; fi
  printf '%s\t%s\t%s\n' "$sub" "$status" "$rel"
}
export -f parse_one
export ROOT

if [ "$FAM" = sv2005 ]; then
  [ -f "$LANE_LIST" ] || { echo "lane list not found: $LANE_LIST (run the adjudicator first)" >&2; exit 4; }
  echo "external-corpus[$FAM]: reading lane files from $LANE_LIST ..." >&2
  mapfile -t FILES < <(tail -n +2 "$LANE_LIST" | cut -f3 | sed "s#^#$ROOT/#" | sort)
else
  echo "external-corpus[$FAM]: collecting files under $SUBS ..." >&2
  mapfile -t FILES < <(find "$SUBS" ${EXTRA_DIRS[@]+"${EXTRA_DIRS[@]}"} -type f \( "${FIND_EXTS[@]}" \) 2>/dev/null | sort)
fi
TOTAL_FOUND=${#FILES[@]}
if [ "$MAX_FILES" -gt 0 ] && [ "$TOTAL_FOUND" -gt "$MAX_FILES" ]; then
  FILES=( "${FILES[@]:0:$MAX_FILES}" )
  echo "external-corpus[$FAM]: capping at $MAX_FILES of $TOTAL_FOUND files (smoke run)" >&2
fi
echo "external-corpus[$FAM]: parsing ${#FILES[@]} files (timeout=${TIMEOUT_S}s, jobs=$JOBS) ..." >&2

printf '%s\0' "${FILES[@]}" | xargs -0 -P "$JOBS" -I{} bash -c 'parse_one "$@"' _ {} >> "$RESULTS"

# ---- instrument identity (SV-CORPUS-GRAD.10) ----
#
# ⛔ A characterization report that does not name WHICH parser produced it cannot be checked
# for staleness. The axis-2 FRESHNESS AUDIT (2026-08-08) had to prove the tracked SV report
# stale by comparing the GIT COMMIT DATES of two OTHER files, because the report itself said
# only "against `parseability_probe`" — not which build, not which grammar, not which HEAD.
# Content hashes make the artifact self-dating: a later reader re-hashes the same three
# inputs and knows in ONE command whether the number still describes their tree. This closes
# the staleness defect class named by project_all_parsers_fully_pass_stimuli_and_external_corpora
# ("the first honest act is to re-measure them rather than to quote them") at the source, so
# the next reader does not have to reconstruct vintage from unrelated commit dates.
sha256_of() {
  [ -f "$1" ] || { printf '(absent)'; return; }
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
  elif command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  else printf '(no sha256 tool)'; fi
}

GRAMMAR_FILE="$ROOT/grammars/$GRAMMAR.ebnf"
GEN_PARSER="$ROOT/generated/${GRAMMAR}_parser.rs"
HEAD_ID="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || printf 'unknown')"
HEAD_DATE="$(git -C "$ROOT" log -1 --date=short --format=%ad 2>/dev/null || printf 'unknown')"
if [ -n "$(git -C "$ROOT" status --porcelain 2>/dev/null)" ]; then HEAD_ID="$HEAD_ID+dirty"; fi

# ---- aggregate ----
TOTAL=$(wc -l < "$RESULTS" | tr -d ' ')
PASS=$(awk -F'\t' '$2=="pass"' "$RESULTS" | wc -l | tr -d ' ')
FAIL=$(awk -F'\t' '$2=="fail"' "$RESULTS" | wc -l | tr -d ' ')
TMO=$(awk -F'\t' '$2=="timeout"' "$RESULTS" | wc -l | tr -d ' ')
CRASH=$(awk -F'\t' '$2=="crash"' "$RESULTS" | wc -l | tr -d ' ')
PCT=$(awk -v p="$PASS" -v t="$TOTAL" 'BEGIN{ if(t>0) printf "%.1f", 100*p/t; else print "0.0" }')

{
  echo "# External Test-Corpus Characterization — \`$FAM\` (EXTERNAL-CORPUS.3.$([ "$FAM" = sv ] && echo 1 || echo 2))"
  echo
  echo "Generated by \`stimuli/run_external_corpus.sh $FAM $TIMEOUT_S $JOBS $MAX_FILES\` against \`$(basename "$PROBE")\`."
  echo "Per-file parse via \`parseability_probe --parse $GRAMMAR <file> $PROFILE_ARGS\`, timeout ${TIMEOUT_S}s, $JOBS-way parallel."
  echo
  echo "> **Characterize, don't game.** Raw parse outcomes only. Many corpus files are"
  echo "> INTENTIONALLY invalid (GHDL \`gna\` regressions, VESTS \`non_compliant/\`, sv-tests"
  echo "> \`:should_fail_because:\`), so a parse-FAIL is frequently the CORRECT outcome — not a"
  echo "> parser bug. Expected-vs-actual adjudication per sub-corpus is the follow-up slice."
  echo
  echo "## Instrument identity (what produced this number)"
  echo
  echo "> Re-hash these three inputs; if any hash differs from the row below, **this report no"
  echo "> longer describes your tree** and the honest act is to re-measure, not to quote."
  echo
  echo "| input | repo-root-relative path | sha256 |"
  echo "|---|---|---|"
  echo "| parse binary | \`${PROBE#"$ROOT/"}\` | \`$(sha256_of "$PROBE")\` |"
  echo "| grammar | \`${GRAMMAR_FILE#"$ROOT/"}\` | \`$(sha256_of "$GRAMMAR_FILE")\` |"
  echo "| generated parser | \`${GEN_PARSER#"$ROOT/"}\` | \`$(sha256_of "$GEN_PARSER")\` |"
  echo
  echo "Measured at \`HEAD\` = \`$HEAD_ID\` ($HEAD_DATE)."
  echo
  echo "## Totals"
  echo
  echo "| files parsed | pass | fail | timeout | crash | pass-rate |"
  echo "|---|---|---|---|---|---|"
  echo "| $TOTAL | $PASS | $FAIL | $TMO | $CRASH | ${PCT}% |"
  echo
  echo "## Per sub-corpus"
  echo
  echo "| sub-corpus | files | pass | fail | timeout | crash | pass-rate |"
  echo "|---|---|---|---|---|---|---|"
  awk -F'\t' '
    { tot[$1]++; if($2=="pass")p[$1]++; else if($2=="timeout")to[$1]++; else if($2=="crash")c[$1]++; else f[$1]++ }
    END { for (s in tot) {
            r = (tot[s]>0)? 100*p[s]/tot[s] : 0;
            printf "| %s | %d | %d | %d | %d | %d | %.1f%% |\n", s, tot[s], p[s]+0, f[s]+0, to[s]+0, c[s]+0, r
          } }
  ' "$RESULTS" | sort
  echo
  echo "_Raw per-file results: \`${RESULTS#"$ROOT/"}\`._"
} > "$REPORT"

echo "external-corpus[$FAM]: $TOTAL parsed — pass=$PASS fail=$FAIL timeout=$TMO crash=$CRASH (${PCT}% pass). Report: $REPORT" >&2
