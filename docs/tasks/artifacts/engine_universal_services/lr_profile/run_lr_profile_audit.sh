#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/lr_profile/run_lr_profile_audit.sh
#
# ENGINE-UNIVERSAL-SERVICES.21 acceptance (e) — THE ENTRY POINT FOR THE LR PROFILE AUDIT.
#
# ⛔⛔ WHY THIS FILE EXISTS AT ALL. `.20` slice 4 and `.21` slice 1 published numbers — LR self-time
# `1.83-3.60 %`, inclusive `22.38-26.75 %`, ratio `6.9-13.6x`, a corpus family share of `2.7411 %`,
# a `24 of 192` sample-impact figure — produced by four python instruments and eight profile reports
# that all lived in **gitignored** `rust/target/audit_scratch/`. `git ls-files` returned **0**. Those
# numbers were therefore unreproducible by anyone, including their author after a `cargo clean`, and
# `GATE-REACHABILITY`'s founding sentence applies verbatim: a check nothing invokes is
# indistinguishable from one that does not exist.
#
# ⭐ PROMOTION IS NOT A COPY, AND THAT WAS THE FIRST FINDING. Two of the four instruments had
# RE-TYPED the LR family predicate, and their copies had already drifted from the one `.21` slice 1
# corrected. Every promoted instrument now IMPORTS `LR_FAMILY_RE` from its single home
# (`stimuli/sv/corpus_parse_cost.py`) and REFUSES if it cannot.
#
# TIERS — the cheap one is the default, because a check nobody minds running keeps running.
#   (default)  attribution + predicate A/B over retained sample reports          — seconds
#   --census   the full-corpus per-rule census that produces the family share    — ~10 min, 16 k files
#   --impact   the per-file census behind the `24 of 192` sample-impact figure   — ~10 min
#   --folding  the linker-folding evidence (needs a vmaddr, see bl_callers.py)   — seconds
#
# ⚠️ The 3.5-6 MB `sample` reports are NOT tracked; `prof_run.sh` beside this file regenerates them.
# Point the audit at a directory of reports with PGEN_LR_PROFILE_REPORTS=<dir> (default:
# rust/target/audit_scratch, where the originals still sit).
#
# Usage:
#   bash docs/tasks/artifacts/engine_universal_services/lr_profile/run_lr_profile_audit.sh [--census|--impact]
# Exit 0 iff every instrument's own controls pass and the predicate A/B agrees.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT"

REPORTS_DIR="${PGEN_LR_PROFILE_REPORTS:-rust/target/audit_scratch}"
mapfile -t REPORTS < <(ls "$REPORTS_DIR"/p[0-9].txt "$REPORTS_DIR"/c[0-9].txt 2>/dev/null)

printf '%s\n' "=============================================================================="
printf 'ENGINE-UNIVERSAL-SERVICES.21 (e) — LR profile audit (tracked instruments)\n'
printf '%s\n' "=============================================================================="
printf 'reports dir : %s\n' "$REPORTS_DIR"
printf 'reports     : %d\n' "${#REPORTS[@]}"

if [ "${#REPORTS[@]}" -eq 0 ]; then
  echo "⛔ no sample reports found. Regenerate them with, e.g.:" >&2
  echo "   bash $HERE/prof_run.sh stimuli/sv/subs/<file>.sv rust/target/lr_profile/p1.txt 6" >&2
  echo "   (the published set was 5 shipped-parser runs p1..p5 + 3 controls c1..c3)" >&2
  exit 2
fi

rc=0
printf '\n--- attribution (every report, all six controls enforced) ---------------------\n'
python3 "$HERE/lr_attribute.py" "${REPORTS[@]}" || rc=1

printf '\n--- predicate A/B: does `.21` slice 1'"'"'s correction move these numbers? ---------\n'
python3 "$HERE/lr_breakdown.py" "${REPORTS[@]}" | tail -12 || rc=1

IMPACT_TSV="rust/target/lr_profile/sample_impact/per_file.tsv"
case "${1:-}" in
  --census)
    printf '\n--- full-corpus family census (~10 min) --------------------------------------\n'
    python3 "$HERE/family_census.py" || rc=1 ;;
  --impact)
    printf '\n--- per-file sample-impact census (~10 min) ----------------------------------\n'
    python3 "$HERE/sample_impact.py" || rc=1 ;;
  *)
    if [ -f "$IMPACT_TSV" ]; then
      printf '\n--- sample-impact tiers, re-analysed from the retained census (seconds) ------\n'
      python3 "$HERE/sample_impact.py" --reuse 2>/dev/null | head -6 || rc=1
    else
      printf '\n--- sample-impact tiers: SKIPPED (no retained census at %s)\n' "$IMPACT_TSV"
      printf '    run with --impact to produce it (~10 min, 16 335 files).\n'
    fi ;;
esac

printf '\n%s\n' "------------------------------------------------------------------------------"
[ "$rc" -eq 0 ] && printf '✅ every instrument control passed; the published intervals re-derive.\n' \
                || printf '⛔ an instrument refused — read its REFUSE line above.\n'
exit "$rc"
