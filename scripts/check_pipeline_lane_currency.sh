#!/usr/bin/env bash
# scripts/check_pipeline_lane_currency.sh
#
# DOCTRINE `PIPELINE-LANE-CURRENCY` (structural, derive-and-diff) — SVPP-EXPANSION.4b.
#
#   The preprocess→parse lane's published numbers are a RENDER of two dumps under a DERIVED context;
#   a copy of any of the three that nobody re-derives is a number about a tree that no longer exists.
#
# WHY A DOCTRINE. `stimuli/sv/characterization/pipeline_characterization.md` is the TRACKED claim of
# what the SystemVerilog front end recognises once macros are expanded — the release reading the
# raw lane cannot give (SVPP-EXPANSION.4: 4 157 UVM + OpenTitan rows carry NO verdict in the raw
# answer key). It is produced by `stimuli/sv/pipeline_vs_raw.py` from the lane's dump and the raw
# lane's dump, under the include context `stimuli/sv/pipeline_context.tsv` (+ the per-file rows of
# `pipeline_context_files.tsv`, + the per-file DEFINES of `pipeline_context_defines.tsv` that a file's
# own driver passes — SVPP-EXPANSION.4c.1, + the per-file UNIT PREDECESSORS of `pipeline_context_units.tsv`
# that a file's own harness compiles first — .4c.2) that `stimuli/sv/pipeline_context.py` DERIVES from the corpus,
# and after the LIBRARY PASS (.6d.1: the design suites' packages parsed in dependency order, exporting their
# facts; its per-package dump `pipeline_library.tsv` and the unit-membership dump `pipeline_library_members.tsv`
# (.6d.2) are summarised in the report and read by the re-adjudication, .gitignored like the lane's). Three
# things can rot independently: the context (a vendored suite gains a header directory and the
# tracked table is not re-derived), the report (a lane re-run whose numbers nobody published), and
# the identity block (already `IDENTITY-CARRIERS`' business — this doctrine does NOT re-check it).
#
# WHAT IT ASSERTS, two arms, each blind to what the other sees:
#   (1) CONTEXT — `pipeline_context.py --check`: all four tracked context tables equal a fresh derivation
#       from the tree. Always evaluable (the corpus submodules being absent is itself reported by
#       the tool, per suite, as NOT EVALUATED). HARD on drift.
#   (3) READJUDICATION — `pipeline_readjudication.py --check` (SVPP-EXPANSION.6a): the tracked
#       re-adjudication report + per-row table (the deferred population re-read under the pipeline)
#       equal a fresh render; the same NOT EVALUATED / REFUSED handling as arm (2).
#   (4) REJECTS-OWN-FILE CENSUS — `docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.py --check`
#       (SVPP-EXPANSION.6e): the tracked census of the rows the parser rejects in their own file was derived
#       from THIS re-adjudication table and THESE dumps — its identity block re-hashes (the re-expansion
#       itself is `--check --full`, run by the lane chain, not here: 20 s of ast_pipeline is not a hook's
#       budget); an absent dump is NOT EVALUATED; a moved input is DRIFT.
#   (2) REPORT — `pipeline_vs_raw.py --check`: the tracked report equals a fresh render of the two
#       dumps on this tree, the `parse binary` digest row and the `Measured at HEAD` line masked (a
#       rebuilt probe and the commit the render ran on are provenance, not staleness — the same
#       reading IDENTITY-CARRIERS gives that label; without the HEAD mask the tracked report was DRIFT
#       on every commit after the one that published it — SVPP-EXPANSION.5(g)(h)). The dumps are .gitignored,
#       so where either is absent the arm reports NOT EVALUATED, loudly, and passes — a clone that
#       has not run the lanes has nothing to compare, and saying so is the honest verdict.
#
# HONEST LIMIT: arm (2) proves the report is the render of the dumps ON DISK; whether those dumps
# were produced by the probe the identity block names is IDENTITY-CARRIERS' (the `parse binary`
# row) and, for the raw dump, CORPUS-CENSUS-CURRENCY's question. The three compose; none re-derives
# another's number.
#
# Exit codes (the §4 contract): 0 holds (incl. NOT EVALUATED, which is printed), 1 breach, 2 refuse
# (a tool missing). `--self-test` drives four arms — a GREEN control on the real tree, a RED context
# mutation, a RED report mutation, and the absent-dump NOT EVALUATED path — each on a COPY, restoring
# every file it touches; a miss exits 7.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
TAG="PIPELINE-LANE-CURRENCY"
CTX_TOOL=stimuli/sv/pipeline_context.py
REPORT_TOOL=stimuli/sv/pipeline_vs_raw.py
READJ_TOOL=stimuli/sv/pipeline_readjudication.py   # SVPP-EXPANSION.6a: the deferred population re-read (arm 3)
CENSUS_TOOL=docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.py   # SVPP-EXPANSION.6e (arm 4)
[ -f "$CTX_TOOL" ] && [ -f "$REPORT_TOOL" ] && [ -f "$READJ_TOOL" ] && [ -f "$CENSUS_TOOL" ] || { echo "$TAG: REFUSE — $CTX_TOOL, $REPORT_TOOL, $READJ_TOOL or $CENSUS_TOOL is missing" >&2; exit 2; }

run_arms() {
  local rc=0 out
  out="$(python3 "$CTX_TOOL" --check 2>&1)" || { echo "$TAG: ✗ (1) CONTEXT — $(printf '%s' "$out" | tail -1)" >&2; rc=1; }
  [ $rc -eq 0 ] && echo "$TAG: (1) CONTEXT — $(printf '%s' "$out" | grep -E '^PIPELINE-CONTEXT: (OK|DRIFT)' | tail -1 | cut -c1-140)"
  out="$(python3 "$REPORT_TOOL" --check 2>&1)"; local r2=$?
  if [ $r2 -ne 0 ]; then echo "$TAG: ✗ (2) REPORT — $(printf '%s' "$out" | grep -E 'DRIFT|REFUSED' | tail -1 | cut -c1-200)" >&2; rc=1
  else echo "$TAG: (2) REPORT — $(printf '%s' "$out" | grep -E '^PIPELINE-VS-RAW: (OK|NOT EVALUATED)' | tail -1 | cut -c1-160)"; fi
  # arm (3) — SVPP-EXPANSION.6a: the tracked re-adjudication report + per-row table equal a fresh render
  out="$(python3 "$READJ_TOOL" --check 2>&1)"; local r3=$?
  if [ $r3 -ne 0 ]; then echo "$TAG: ✗ (3) READJUDICATION — $(printf '%s' "$out" | grep -E 'DRIFT|REFUSED' | tail -1 | cut -c1-200)" >&2; rc=1
  else echo "$TAG: (3) READJUDICATION — $(printf '%s' "$out" | grep -E '^PIPELINE-READJUDICATION: (OK|NOT EVALUATED)' | tail -1 | cut -c1-160)"; fi
  # arm (4) — SVPP-EXPANSION.6e: the rejects-own-file census's identity names the table and dumps on disk
  out="$(python3 "$CENSUS_TOOL" --check 2>&1)"; local r4=$?
  if [ $r4 -ne 0 ]; then echo "$TAG: ✗ (4) REJECTS-OWN-FILE CENSUS — $(printf '%s' "$out" | grep -E 'DRIFT|REFUSED|no tracked' | tail -1 | cut -c1-200)" >&2; rc=1
  else echo "$TAG: (4) REJECTS-OWN-FILE CENSUS — $(printf '%s' "$out" | grep -E '^REJECTS-OWN-FILE-CENSUS: (OK|NOT EVALUATED)' | tail -1 | cut -c1-160)"; fi
  return $rc
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  CTX=stimuli/sv/pipeline_context.tsv; REP=stimuli/sv/characterization/pipeline_characterization.md; DUMP=stimuli/sv/characterization/pipeline_results.tsv
  READJ=stimuli/sv/characterization/pipeline_readjudication.tsv
  cp "$CTX" "$CTX.selftest.bak"; cp "$REP" "$REP.selftest.bak"; cp "$READJ" "$READJ.selftest.bak"; [ -f "$DUMP" ] && cp "$DUMP" "$DUMP.selftest.bak"
  restore() { mv -f "$CTX.selftest.bak" "$CTX"; mv -f "$REP.selftest.bak" "$REP"; mv -f "$READJ.selftest.bak" "$READJ"; [ -f "$DUMP.selftest.bak" ] && mv -f "$DUMP.selftest.bak" "$DUMP"; return 0; }
  trap restore EXIT
  # arm A — GREEN control on the real tree
  if run_arms >/dev/null 2>&1; then echo "  PASS A green control (the real tree holds)"; else echo "  FAIL A green control"; fails=$((fails+1)); fi
  # arm B — RED: a context row deleted must be DRIFT
  sed -i.tmp '2d' "$CTX" && rm -f "$CTX.tmp"
  if run_arms >/dev/null 2>&1; then echo "  FAIL B a deleted context row was NOT refused"; fails=$((fails+1)); else echo "  PASS B a deleted context row is refused"; fi
  cp "$CTX.selftest.bak" "$CTX"
  # arm C — RED: a report number edited must be DRIFT (only when the dumps are present)
  if [ -f "$DUMP" ]; then
    python3 - "$REP" <<'PY'
import sys,re,pathlib
p=pathlib.Path(sys.argv[1]); s=p.read_text()
s=re.sub(r"\| \*\*total\*\* \| (\d+) \|", lambda m: f"| **total** | {int(m.group(1))+1} |", s, count=1)
p.write_text(s)
PY
    if run_arms >/dev/null 2>&1; then echo "  FAIL C an edited report number was NOT refused"; fails=$((fails+1)); else echo "  PASS C an edited report number is refused"; fi
    cp "$REP.selftest.bak" "$REP"
    # arm D — the absent-dump path is NOT EVALUATED and passes
    mv "$DUMP" "$DUMP.selftest.gone"
    # (captured first: `grep -q` closing the pipe after its first match made the third arm's later write
    # a SIGPIPE, and pipefail turned a correct NOT EVALUATED into a FAIL — measured when arm (3) landed)
    d_out="$(run_arms 2>&1)"
    if printf '%s' "$d_out" | grep -q 'NOT EVALUATED'; then echo "  PASS D an absent dump is NOT EVALUATED, loudly"; else echo "  FAIL D an absent dump was not reported"; fails=$((fails+1)); fi
    mv "$DUMP.selftest.gone" "$DUMP"
    # arm E — RED: a re-adjudication verdict edited in the per-row table must be DRIFT (SVPP-EXPANSION.6a)
    sed -i.tmp '2s/\t[a-z_:]*\t\([^\t]*\)$/\tselftest_edited\t\1/' "$READJ" && rm -f "$READJ.tmp"
    if run_arms >/dev/null 2>&1; then echo "  FAIL E an edited re-adjudication verdict was NOT refused"; fails=$((fails+1)); else echo "  PASS E an edited re-adjudication verdict is refused"; fi
    cp "$READJ.selftest.bak" "$READJ"
    # arm F — RED: the census's recorded identity for the re-adjudication table no longer hashes (SVPP-EXPANSION.6e):
    # the table is edited in a way arm (3) does not see (a trailing comment line) so only arm (4) can refuse it
    printf '# selftest\n' >> "$READJ"
    f_out="$(run_arms 2>&1)"
    if printf '%s' "$f_out" | grep -q '(4) REJECTS-OWN-FILE CENSUS — .*DRIFT'; then echo "  PASS F a re-adjudication table the census did not read is refused by arm (4)"; else echo "  FAIL F the census arm did not refuse a moved input"; fails=$((fails+1)); fi
    cp "$READJ.selftest.bak" "$READJ"
  else
    echo "  SKIP C/D/E/F (no dump on this tree — the arms need one to mutate)"
  fi
  echo "$TAG-SELF-TEST: arms=$([ -f "$DUMP" ] && echo 6 || echo 2) failed=$fails"
  [ $fails -eq 0 ] || exit 7
  exit 0
fi

if run_arms; then echo "$TAG: OK"; exit 0; else echo "$TAG: ✗ FAIL — the lane's published context or report no longer describes this tree; re-derive (pipeline_context.py --write; run_pipeline_lane.sh all; pipeline_vs_raw.py)" >&2; exit 1; fi
