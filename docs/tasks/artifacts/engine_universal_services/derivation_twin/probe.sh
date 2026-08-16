#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/derivation_twin/probe.sh
#
# ENGINE-UNIVERSAL-SERVICES.22 acceptance (b) — IS A VERDICT AGREEMENT A DERIVATION AGREEMENT?
#
#   (b) decide whether the file's `accepted: True` under 3.4 and its `pass` under a bare parse
#       are the same derivation (a verdict agreement is not a derivation agreement).
#
# PGEN runs a parse through one of two graphs (the "observability twin", TOOLBOX 2.1 / 3.4
# ROUTING). A BARE parse runs the fused `cascade_*` functions; a parse with any diagnostic consumer
# attached runs the PROTOCOL graph, and the 3.4/3.5 dumps additionally switch on the transactional
# COVERAGE recorder. `.22` is a record of that recorder being catastrophically wrong about cost —
# so "does it also change the ANSWER" is not a rhetorical question.
#
# THREE ARMS, ONE BINARY, and each arm must PROVE it is the configuration it claims:
#
#   A  bare                                   -> FUSED graph      tell: NO stderr at all
#   B  PGEN_REPORT_MEMO_STATS=1               -> PROTOCOL graph   tell: a `=== MEMO STATS:` block
#   C  --dump-ast-with-coverage               -> PROTOCOL + the   tell: `COVERAGE-DUMP-AST:
#                                                COVERAGE recorder       enable_coverage=true
#                                                                        exercised_rules=N`, N>0
#
# ⛔⛔ THE TELLS ARE THE POINT, AND TWO EARLIER CANDIDATES WERE MEASURED AND REJECTED. (1) Passing
# `--dump-rule-outcome-counts-json` beside `--parse-dump-ast` LOOKS like it routes the parse — the
# global is applied before dispatch — but the `enable_coverage()` call lives in the `--parse` detail
# macro, so no counts file is written and BOTH arms come out bare. (2) `PGEN_REPORT_MEMO_STATS=1`
# was then proposed as the coverage tell: its aggregate header is BYTE-IDENTICAL with and without
# coverage (`5710 success entries … 29378 total, 6170 subtree-nodes, 369 distinct rules`), and the
# only difference is which members of a tie group the top-30 cutoff prints. An A/B built on either
# would have agreed for the wrong reason and reported a PASS.
# ⇒ arm C's tell is the coverage recorder's OWN read-back, which is zero by construction when
# coverage is off, so it cannot be faked by a flag that was merely typed.
#
# Usage:  bash docs/tasks/artifacts/engine_universal_services/derivation_twin/probe.sh [N]
#           N = how many pinned-sample files to check (default: all of them)
# Output: result.txt beside this script. Exit 0 = every file's three derivations are byte-identical.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# derivation_twin -> engine_universal_services -> artifacts -> tasks -> docs -> ROOT (FIVE levels).
# ⛔ Checked, not counted: a wrong `..` depth resolves to a real directory that merely lacks the
# repo, and the first symptom is a confusing "no probe here" — which is what it was, twice in one
# session at this exact depth. The marker file makes the mistake self-announcing.
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT" || exit 2
if [ ! -f "$ROOT/stimuli/sv/parse_cost_sample.tsv" ]; then
  echo "derivation-twin: REFUSING — repo root did not resolve from $HERE (got $ROOT)" >&2
  exit 2
fi
OUT="$HERE/result.txt"
WORK="rust/target/derivation_twin"; rm -rf "$WORK"; mkdir -p "$WORK"

MANIFEST="stimuli/sv/parse_cost_sample.tsv"
# ⛔ The DEBUG probe is the default and that is deliberate, not laziness: `.20` established the
# entry counters are byte-identical between the debug and release builds, and this check compares
# a parse against ITSELF under three configurations — the same binary in all three arms is what
# makes the comparison mean anything. A release build would be faster and prove exactly as much.
PROBE="${PGEN_PROBE:-rust/target/debug/parseability_probe}"
LIMIT="${1:-0}"

if [ ! -x "$PROBE" ]; then
  echo "derivation-twin: REFUSING — no probe at $PROBE. Build it:" >&2
  echo "    cd rust && cargo build --bin parseability_probe --features generated_parsers" >&2
  exit 2
fi

# ⛔ THE PATHOLOGICAL FILE GOES FIRST, ALWAYS. It is the file `.22` is about — the one the coverage
# stack never terminated on — so a run that checked everything EXCEPT it would answer a question
# nobody asked.
PATHOLOGICAL="stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv"

files=("$PATHOLOGICAL")
while IFS=$'\t' read -r _tier path; do
  case "$_tier" in \#*|"") continue ;; esac
  [ -n "${path:-}" ] || continue
  [ "$path" = "$PATHOLOGICAL" ] && continue
  files+=("$path")
done < "$MANIFEST"
# ⛔⛔ A LIMIT MUST NOT SILENTLY MEAN "THE FIRST TIER". The manifest is ordered hot(40), lr(40),
# breadth(112), so a prefix cut of N<40 checks ONLY the heaviest files — and would report
# "40 files checked, all identical" while never touching a `breadth` row. That is a coverage lie
# in the passing direction, which is the defect class this whole tree exists to remove. The bound
# is therefore spent as a deterministic STRIDE across the manifest, and the tier census of what
# was actually checked is printed in the result.
if [ "$LIMIT" -gt 0 ] 2>/dev/null && [ "$LIMIT" -lt "${#files[@]}" ]; then
  stride=$(( ${#files[@]} / LIMIT )); [ "$stride" -lt 1 ] && stride=1
  strided=("$PATHOLOGICAL")
  i=1
  while [ "$i" -lt "${#files[@]}" ] && [ "${#strided[@]}" -lt "$LIMIT" ]; do
    strided+=("${files[$i]}")
    i=$(( i + stride ))
  done
  files=("${strided[@]}")
fi

{
  echo "=============================================================================="
  echo "ENGINE-UNIVERSAL-SERVICES.22 (b) — verdict agreement vs DERIVATION agreement"
  echo "=============================================================================="
  echo "probe:  $PROBE"
  echo "files:  ${#files[@]} (the pathological file first, then a deterministic stride over the"
  # ⛔ Single-quoted on purpose: backticks inside a DOUBLE-quoted echo are a command substitution,
  # and this line originally ran `hot` as a command and printed the empty result. Harmless here,
  # silent by nature, and exactly the class of thing a report is supposed not to do.
  echo '        pinned parse-cost sample — never a prefix, which would be `hot` tier only)'
  echo
} > "$OUT"

# The tier census of what is ACTUALLY checked, so the population can never be over-read.
declare -A tier_of=()
while IFS=$'\t' read -r _t _p; do
  case "$_t" in \#*|"") continue ;; esac
  [ -n "${_p:-}" ] && tier_of["$_p"]="$_t"
done < "$MANIFEST"

same=0; differ=0; skipped=0; tell_fail=0
first_mismatch=""

for rel in "${files[@]}"; do
  [ -f "$rel" ] || { skipped=$((skipped + 1)); continue; }
  stem="$WORK/$(printf '%s' "$rel" | tr '/.' '__')"

  "$PROBE" --parse-dump-ast systemverilog "$rel" "${stem}.A.json" --profile sv_2017 \
      >/dev/null 2>"${stem}.A.err"
  rcA=$?
  PGEN_REPORT_MEMO_STATS=1 "$PROBE" --parse-dump-ast systemverilog "$rel" "${stem}.B.json" \
      --profile sv_2017 >/dev/null 2>"${stem}.B.err"
  rcB=$?
  "$PROBE" --parse-dump-ast systemverilog "$rel" "${stem}.C.json" --profile sv_2017 \
      --dump-ast-with-coverage >/dev/null 2>"${stem}.C.err"
  rcC=$?

  # ⛔ A REJECTED file has no derivation to compare, and silently counting it as "same" would
  # inflate the pass count with rows that were never tested. Reported separately.
  if [ $rcA -ne 0 ] || [ $rcB -ne 0 ] || [ $rcC -ne 0 ]; then
    skipped=$((skipped + 1))
    continue
  fi

  # The arm-identity assertions. A miss here is a REFUSAL, not a mismatch: it means the three arms
  # were not three configurations, and the comparison below would be meaningless.
  if [ -s "${stem}.A.err" ] \
     || ! grep -q 'MEMO STATS' "${stem}.B.err" \
     || ! grep -qE 'COVERAGE-DUMP-AST: enable_coverage=true exercised_rules=[1-9]' "${stem}.C.err"
  then
    tell_fail=$((tell_fail + 1))
    { echo "⛔ ARM IDENTITY FAILED for $rel — the three arms are not three configurations:"
      echo "     A stderr bytes: $(wc -c < "${stem}.A.err" | tr -d ' ') (want 0)"
      echo "     B memo-stats:   $(grep -c 'MEMO STATS' "${stem}.B.err") (want 1)"
      echo "     C coverage:     $(grep -c 'COVERAGE-DUMP-AST' "${stem}.C.err") (want 1, exercised>0)"
    } >> "$OUT"
    continue
  fi

  if cmp -s "${stem}.A.json" "${stem}.B.json" && cmp -s "${stem}.A.json" "${stem}.C.json"; then
    same=$((same + 1))
  else
    differ=$((differ + 1))
    [ -z "$first_mismatch" ] && first_mismatch="$rel"
    { echo "⛔ DERIVATIONS DIFFER for $rel"
      echo "     A sha $(shasum -a 256 < "${stem}.A.json" | cut -d' ' -f1)"
      echo "     B sha $(shasum -a 256 < "${stem}.B.json" | cut -d' ' -f1)"
      echo "     C sha $(shasum -a 256 < "${stem}.C.json" | cut -d' ' -f1)"
    } >> "$OUT"
  fi
  rm -f "${stem}".*.json
done

tier_census=""
for t in hot lr breadth; do
  n=0
  for rel in "${files[@]}"; do [ "${tier_of[$rel]:-}" = "$t" ] && n=$(( n + 1 )); done
  tier_census="$tier_census $t=$n"
done
# The pathological file is not in the manifest, so it belongs to no tier and is named separately.
tier_census="$tier_census (+the pathological file)"

{
  echo "RESULT"
  echo "  files whose THREE derivations are byte-identical : $same"
  echo "  files whose derivations DIFFER                   : $differ"
  echo "  files skipped (rejected parse / absent)          : $skipped"
  echo "  files whose ARM IDENTITY could not be proven     : $tell_fail"
  echo "  tiers actually covered                           : $tier_census"
  echo
  if [ "$tell_fail" -gt 0 ]; then
    echo "⛔ REFUSED — at least one row could not prove its three arms were three configurations."
    echo "   A comparison whose arms may be identical agrees for the wrong reason."
  elif [ "$differ" -gt 0 ]; then
    echo "⛔ THE OBSERVABILITY TWIN CHANGES THE DERIVATION — first: $first_mismatch"
    echo "   That is an ENGINE defect, not a measurement artifact: every counter-based instrument"
    echo "   in TOOLBOX 3.1-3.6 would then be describing a parse a production run never performs."
  else
    echo "✅ (b) ANSWERED — on every file checked, the FUSED graph, the PROTOCOL graph and the"
    echo "   PROTOCOL graph WITH the transactional coverage recorder produce a BYTE-IDENTICAL AST."
    echo "   The verdict agreement is backed by a derivation agreement, and the 3.4/3.5 dumps"
    echo "   describe the derivation a production parse actually performs."
  fi
  echo "=============================================================================="
} >> "$OUT"

cat "$OUT"
[ "$tell_fail" -eq 0 ] && [ "$differ" -eq 0 ]
