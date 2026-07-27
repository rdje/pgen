#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_root_cause_box_census.sh
#
# GENERATED-LINT-CORRECTNESS.4 — census every ticked ROOT CAUSE box in docs/tasks/*.md and
# report whether the box's OWN BODY carries a diagnosis-tool signature.
#
# ⭐ WHY THIS IS NOT A FRESH GREP. The whole point of `.3`'s box-scoping hardening is that the
# signature must sit INSIDE the ticked box's own bullet. A census written with an independent
# regex would measure a DIFFERENT rule than the one the enforcer applies, and the resulting
# number would be an estimate rather than a fact about the gate. So this driver:
#   - SOURCES `DIAGNOSIS_SIG` verbatim out of `scripts/check_diagnosis_evidence.sh` (so the two
#     can never drift), and
#   - reuses the enforcer's exact header regex and `box_body` extent rule (copied with the same
#     semantics: body runs to the next same-or-shallower box, the next `#` heading, or EOF).
#
# Modes:
#   --census                census all files; per-file backed/unbacked box counts + totals
#   --dump <task-file>      print every UNBACKED ticked ROOT CAUSE box body from that file
#   --dump-backed <file>    print every BACKED box body (control: what a passing box looks like)
#   --sig <regex>           evaluate a CANDIDATE signature regex instead of the shipped one;
#                           combine with --census to price a proposed 5th family against the
#                           real corpus of unbacked boxes.
#   --alt <regex>           with --census: report how many CURRENTLY-UNBACKED boxes the
#                           candidate <regex> would newly admit (the "what does it buy" number).
#   --classify              bucket every UNBACKED box by what it ACTUALLY contains. This is the
#                           instrument that answers "what SHAPE is the gap?" instead of guessing
#                           a regex and hoping. Buckets are deliberately overlapping — a box can
#                           be both profiler-worded and site-citing; the point is the profile of
#                           the corpus, not a partition.
#   --dump-bucket <name>    print every unbacked box matching one --classify bucket (read the
#                           source, do not infer from the tally).
#   --placement             THE DECISIVE SPLIT. For every UNBACKED box, ask whether its own FILE
#                           carries a diagnosis signature somewhere else. That partitions the gap
#                           into:
#                             OUT-OF-BOX  the leaf DOES hold tool evidence, just not inside the
#                                         ticked bullet — i.e. exactly what `.3`'s box-scoping
#                                         newly rejects. A PLACEMENT problem.
#                             NO-EVIDENCE the whole leaf file carries no diagnosis signature at
#                                         all — a genuine evidence gap.
#                           This is the number that decides whether a fifth SIGNATURE family is
#                           the right instrument at all.
#
# Exit 0 always on a successful census (this is a measuring instrument, not a gate).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

ENFORCER="scripts/check_diagnosis_evidence.sh"
[ -f "$ENFORCER" ] || { echo "census: ✗ enforcer not found: $ENFORCER" >&2; exit 1; }

# Source the live signature + keyword definitions verbatim from the enforcer. If either line
# ever stops being a single self-contained assignment this fails loudly rather than silently
# measuring a stale rule.
eval "$(grep -E "^DIAGNOSIS_SIG=" "$ENFORCER")"
eval "$(grep -E "^ROOT_KW=" "$ENFORCER")"
[ -n "${DIAGNOSIS_SIG:-}" ] || { echo "census: ✗ could not extract DIAGNOSIS_SIG from $ENFORCER" >&2; exit 1; }
[ -n "${ROOT_KW:-}" ]       || { echo "census: ✗ could not extract ROOT_KW from $ENFORCER" >&2; exit 1; }

HDR_RE="^[[:space:]]*[-*][[:space:]]*\[[xX]\][[:space:]].*($ROOT_KW)"

WORK="$(mktemp -d "$ROOT/rust/target/root_cause_box_census.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# box_body <start-line> <file> — identical extent rule to the enforcer's.
box_body() {
  awk -v start="$1" '
    function isbox(l)     { return match(l, /^[ \t]*[-*][ \t]*\[[ xX]\][ \t]/) }
    function indent(l, i) { i = match(l, /[^ \t]/); return (i == 0 ? 0 : i - 1) }
    NR <  start { next }
    NR == start { boxind = indent($0); print; next }
    {
      if (isbox($0) && indent($0) <= boxind) exit
      if ($0 ~ /^#/) exit
      print
    }
  ' "$2"
}

# ---------------------------------------------------------------------------------------------
# CLASSIFY buckets. Each entry: "name|what it means|regex".
# ⚠️ These are DESCRIPTIVE probes over the existing corpus, NOT a proposed gate. Their job is to
# answer "what do the unbacked boxes actually contain?" before any signature is designed —
# the [[feedback_read_prior_art_before_designing]] RE-MEASURE clause applied to the gate's own
# input corpus.
# ---------------------------------------------------------------------------------------------
BUCKETS=(
  "macos-sampler|macOS /usr/bin/sample profiler run (the tool this repo's SPEED campaign actually uses)|/usr/bin/sample|\bsample[0-9]+_|sample_nfa|sample_mi|decomposition[0-9]*\.txt|categorize[0-9]*\.py"
  "disassembly|instruction-level evidence (otool -tV / annotated disassembly)|otool|disassembl|\basm\b|instruction-decisive"
  "counter-dump|PGEN's own runtime counter/outcome dumps|--dump-rule-outcome-counts|counter dump|counters? (dump|series)|outcome dumps?|facts_emitted|rollbacks=|memo_census"
  "artifact-path|cites a banked evidence artifact under docs/tasks/artifacts/|docs/tasks/artifacts/"
  "site-citation|names a concrete source site file.ext:LINE|[A-Za-z0-9_./-]+\.(rs|ebnf|sh|toml|json|py|pl):[0-9]+"
  "contrast|states a before->after / arm-vs-arm outcome difference|[0-9][0-9,.]*[[:space:]]*(->|→|=>)[[:space:]]*[0-9]|ACCEPT[[:space:]]*(->|→)[[:space:]]*REJECT|REJECT[[:space:]]*(->|→)[[:space:]]*ACCEPT|byte-identical|diverge=|[-+][0-9]+\.[0-9]+%"
  "carried-forward|root cause established in a NAMED earlier slice and inherited here|carried (in )?from|carried from|the .-0[0-9]{3}. (root cause|design|census|section)"
  "not-a-defect|self-declared N/A: a measurement / docs / planning slice, no defect to diagnose|n/a.?defect|N/A defect-wise|measurement slice|docs re-baseline"
  "census-population|quantifies the affected population with a measured count|census|[0-9]+/[0-9]+ (sites|entries|rules|discards)|population"
)

MODE="census"; TARGET=""; SIG="$DIAGNOSIS_SIG"; ALT=""; BUCKET=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --census)       MODE="census"; shift ;;
    --classify)     MODE="classify"; shift ;;
    --placement)    MODE="placement"; shift ;;
    --dump)         MODE="dump";        TARGET="${2:?--dump needs a task file}"; shift 2 ;;
    --dump-backed)  MODE="dump-backed"; TARGET="${2:?--dump-backed needs a task file}"; shift 2 ;;
    --dump-bucket)  MODE="dump-bucket"; BUCKET="${2:?--dump-bucket needs a bucket name}"; shift 2 ;;
    --sig)          SIG="${2:?--sig needs a regex}"; shift 2 ;;
    --alt)          ALT="${2:?--alt needs a regex}"; shift 2 ;;
    *) echo "census: ✗ unknown argument: $1" >&2; exit 1 ;;
  esac
done

case "$MODE" in
  census|classify|dump-bucket|placement) mapfile -t FILES < <(git ls-files 'docs/tasks/*.md' | sort) ;;
  *)                                     FILES=("$TARGET") ;;
esac

if [ "$MODE" = "placement" ]; then
  oob=0; noev=0; total=0
  : > "$WORK/placement.txt"
  for f in "${FILES[@]}"; do
    [ -f "$f" ] || continue
    grep -nEi -- "$HDR_RE" "$f" > "$WORK/hdr.txt" 2>/dev/null || true
    [ -s "$WORK/hdr.txt" ] || continue
    # Does the FILE carry a diagnosis signature ANYWHERE? (= the pre-`.3` whole-file rule.)
    file_has_sig=0
    grep -Eiq -- "$SIG" "$f" && file_has_sig=1
    fo=0; fn=0
    while IFS= read -r ln; do
      [ -n "$ln" ] || continue
      box_body "$ln" "$f" > "$WORK/body.txt"
      grep -Eiq -- "$SIG" "$WORK/body.txt" && continue
      total=$((total + 1))
      if [ "$file_has_sig" -eq 1 ]; then oob=$((oob + 1)); fo=$((fo + 1))
      else                               noev=$((noev + 1)); fn=$((fn + 1)); fi
    done < <(cut -d: -f1 "$WORK/hdr.txt")
    [ $((fo + fn)) -gt 0 ] && printf '%6d %6d  %s\n' "$fn" "$fo" "$f" >> "$WORK/placement.txt"
  done
  echo "=============================================================================="
  echo "WHERE THE EVIDENCE ACTUALLY IS, for the $total unbacked boxes"
  echo "=============================================================================="
  printf '%6s %6s  %s\n' "NO-EV" "OUTBOX" "FILE"
  sort -rn "$WORK/placement.txt"
  echo "------------------------------------------------------------------------------"
  printf 'OUT-OF-BOX  (leaf HAS tool evidence, outside the ticked bullet) .. %4d  %3d%%\n' \
    "$oob" "$(( total == 0 ? 0 : oob * 100 / total ))"
  printf 'NO-EVIDENCE (leaf file carries no diagnosis signature at all) ... %4d  %3d%%\n' \
    "$noev" "$(( total == 0 ? 0 : noev * 100 / total ))"
  exit 0
fi

if [ "$MODE" = "classify" ] || [ "$MODE" = "dump-bucket" ]; then
  declare -A HITS=()
  for e in "${BUCKETS[@]}"; do HITS["${e%%|*}"]=0; done
  none=0; total=0
  for f in "${FILES[@]}"; do
    [ -f "$f" ] || continue
    grep -nEi -- "$HDR_RE" "$f" > "$WORK/hdr.txt" 2>/dev/null || true
    [ -s "$WORK/hdr.txt" ] || continue
    while IFS= read -r ln; do
      [ -n "$ln" ] || continue
      box_body "$ln" "$f" > "$WORK/body.txt"
      grep -Eiq -- "$SIG" "$WORK/body.txt" && continue   # backed already; not part of the gap
      total=$((total + 1)); matched_any=0
      for e in "${BUCKETS[@]}"; do
        name="${e%%|*}"; rest="${e#*|}"; re="${rest#*|}"
        if grep -Eiq -- "$re" "$WORK/body.txt"; then
          HITS["$name"]=$(( HITS["$name"] + 1 )); matched_any=1
          if [ "$MODE" = "dump-bucket" ] && [ "$BUCKET" = "$name" ]; then
            printf '\n===== %s:%s  [%s] =====\n' "$f" "$ln" "$name"; cat "$WORK/body.txt"
          fi
        fi
      done
      if [ "$matched_any" -eq 0 ]; then
        none=$((none + 1))
        if [ "$MODE" = "dump-bucket" ] && [ "$BUCKET" = "NONE" ]; then
          printf '\n===== %s:%s  [NO BUCKET] =====\n' "$f" "$ln"; cat "$WORK/body.txt"
        fi
      fi
    done < <(cut -d: -f1 "$WORK/hdr.txt")
  done
  if [ "$MODE" = "classify" ]; then
    echo "=============================================================================="
    echo "WHAT THE $total UNBACKED ROOT CAUSE BOXES ACTUALLY CONTAIN (buckets overlap)"
    echo "=============================================================================="
    for e in "${BUCKETS[@]}"; do
      name="${e%%|*}"; rest="${e#*|}"; desc="${rest%%|*}"
      printf '%5d  %3d%%  %-18s %s\n' "${HITS[$name]}" \
        "$(( total == 0 ? 0 : HITS[$name] * 100 / total ))" "$name" "$desc"
    done
    echo "------------------------------------------------------------------------------"
    printf '%5d  %3d%%  %-18s %s\n' "$none" "$(( total == 0 ? 0 : none * 100 / total ))" \
      "(no bucket)" "matched none of the probes above — read these with --dump-bucket NONE"
  fi
  exit 0
fi

total_boxes=0; total_backed=0; total_unbacked=0; total_alt=0
: > "$WORK/per_file.txt"

for f in "${FILES[@]}"; do
  [ -f "$f" ] || continue
  grep -nEi -- "$HDR_RE" "$f" > "$WORK/hdr.txt" 2>/dev/null || true
  [ -s "$WORK/hdr.txt" ] || continue
  boxes=0; backed=0; unbacked=0; alt=0
  while IFS= read -r ln; do
    [ -n "$ln" ] || continue
    boxes=$((boxes + 1))
    box_body "$ln" "$f" > "$WORK/body.txt"
    if grep -Eiq -- "$SIG" "$WORK/body.txt"; then
      backed=$((backed + 1))
      if [ "$MODE" = "dump-backed" ]; then
        printf '\n===== %s:%s  [BACKED] =====\n' "$f" "$ln"; cat "$WORK/body.txt"
      fi
    else
      unbacked=$((unbacked + 1))
      if [ -n "$ALT" ] && grep -Eiq -- "$ALT" "$WORK/body.txt"; then alt=$((alt + 1)); fi
      if [ "$MODE" = "dump" ]; then
        printf '\n===== %s:%s  [UNBACKED] =====\n' "$f" "$ln"; cat "$WORK/body.txt"
      fi
    fi
  done < <(cut -d: -f1 "$WORK/hdr.txt")
  total_boxes=$((total_boxes + boxes))
  total_backed=$((total_backed + backed))
  total_unbacked=$((total_unbacked + unbacked))
  total_alt=$((total_alt + alt))
  [ "$unbacked" -gt 0 ] && printf '%6d %6d %6d  %s\n' "$unbacked" "$backed" "$boxes" "$f" >> "$WORK/per_file.txt"
done

if [ "$MODE" = "census" ]; then
  echo "=========================================================================="
  echo "ROOT CAUSE box census — signature rule sourced live from $ENFORCER"
  [ "$SIG" != "$DIAGNOSIS_SIG" ] && echo "(CANDIDATE signature in use, NOT the shipped one)"
  echo "=========================================================================="
  printf '%6s %6s %6s  %s\n' "UNBACK" "BACKED" "BOXES" "FILE"
  sort -rn "$WORK/per_file.txt"
  echo "--------------------------------------------------------------------------"
  printf 'TOTAL ticked ROOT CAUSE boxes ......... %d\n' "$total_boxes"
  printf 'BACKED (signature inside the box) ..... %d\n' "$total_backed"
  printf 'UNBACKED .............................. %d\n' "$total_unbacked"
  printf 'FILES holding >=1 unbacked box ........ %d\n' "$(wc -l < "$WORK/per_file.txt" | tr -d ' ')"
  if [ -n "$ALT" ]; then
    echo "--------------------------------------------------------------------------"
    printf 'CANDIDATE --alt regex would newly admit %d of the %d unbacked boxes (%d%%).\n' \
      "$total_alt" "$total_unbacked" "$(( total_unbacked == 0 ? 0 : total_alt * 100 / total_unbacked ))"
  fi
fi
exit 0
