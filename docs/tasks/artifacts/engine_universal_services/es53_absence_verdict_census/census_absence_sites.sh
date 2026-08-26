#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.53 (c) — the ABSENCE-SITE census, made re-runnable.
#
# WHAT IT ANSWERS. `grammar_tree` is the ACTIVE (profile-filtered) rule map, so a name absent from
# it is absent for one of two reasons the generator cannot tell apart: the profile filter removed it
# (`apply_grammar_profile_filter` prunes `rule_order` too), or it was never defined in this grammar
# at all — an `include(...)` external reference, which `node_satisfiable` deliberately treats as ⊤
# "no false positives". `ENGINE-UNIVERSAL-SERVICES.51` shipped a fix for one site that read the
# first reason into the second; `.53` owns the survivor. The question this census keeps answerable
# is not "how many lookups are there" but "which lookups turn an ABSENCE into a VERDICT".
#
# ⛔ WHY A MANIFEST AND NOT A GREP COUNT. The raw population is large and almost entirely benign:
# publishing it as a defect count would be the census-claim error in its other direction — a
# POPULATION mistaken for a DEFECT COUNT. Publishing only the verdict sites would leave the rest
# silently unexamined, which is the error in the first direction. So every site is pinned with an
# explicit class, and a site the tree gains or loses fails this check until it is classified.
#
# CLASSIFICATION RULE (the one this manifest was built with — apply it verbatim to a new row):
#   verdict — the ABSENCE branch produces the function's own returned classification of the grammar
#             (dead / gated / leads-with / reachable). An absent name is ASSERTED to have a property.
#   guard   — the ABSENCE branch only declines to act: `continue`, an early `return`/`?` that yields
#             no answer, a `.filter(..)`, a name list, an error. An absent name is SKIPPED, never
#             characterised.
#   fixture — the site is inside `#[cfg(test)]`: test plumbing, not engine logic.
#
# USAGE
#   bash docs/tasks/artifacts/engine_universal_services/es53_absence_verdict_census/census_absence_sites.sh
#   … --list     print the derived rows and exit 0 (no manifest comparison)
#   … --verdicts print only the rows classified `verdict`
# Exit 0 = the tree matches the manifest. Exit 1 = drift (a row added, removed, or re-shaped).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

SOURCE="rust/src/ast_pipeline/stimuli_generator.rs"
MANIFEST="docs/tasks/artifacts/engine_universal_services/es53_absence_verdict_census/absence_sites_v1.tsv"

[ -f "$SOURCE" ] || { echo "ABSENCE-SITE-CENSUS: source not found: $SOURCE" >&2; exit 1; }

# Derive `scope <TAB> fn <TAB> count <TAB> normalized-source` for every `grammar_tree` lookup.
# The key is (scope, fn, text) — never a line number, which drifts on any edit above the site.
derive() {
  awk '
    /^#\[cfg\(test\)\]/ { scope = "test" }
    /^[ \t]*(pub[( ]*[a-z]*[)]* )?(async )?fn [a-z_0-9]+/ {
      f = $0
      sub(/^[ \t]+/, "", f); sub(/^pub[( ]*[a-z]*[)]* /, "", f); sub(/^async /, "", f)
      sub(/^fn /, "", f); sub(/[(<].*/, "", f)
      curfn = f
    }
    /grammar_tree\.contains_key|grammar_tree\.get/ {
      t = $0
      gsub(/^[ \t]+/, "", t); gsub(/[ \t]+$/, "", t); gsub(/[ \t]+/, " ", t)
      key = ((scope == "test") ? "test" : "engine") "\t" curfn "\t" t
      if (!(key in seen)) { order[++n] = key }
      seen[key]++
    }
    END {
      for (i = 1; i <= n; i++) {
        split(order[i], parts, "\t")
        printf "%s\t%s\t%d\t%s\n", parts[1], parts[2], seen[order[i]], parts[3]
      }
    }
  ' "$SOURCE" | sort
}

case "${1:-}" in
  --list) derive; exit 0 ;;
  --verdicts)
    [ -f "$MANIFEST" ] || { echo "ABSENCE-SITE-CENSUS: manifest not found: $MANIFEST" >&2; exit 1; }
    grep -v '^#' "$MANIFEST" | awk -F'\t' '$1 == "verdict"'
    exit 0 ;;
esac

[ -f "$MANIFEST" ] || { echo "ABSENCE-SITE-CENSUS: manifest not found: $MANIFEST" >&2; exit 1; }

derived="$(derive)"
pinned="$(grep -v '^#' "$MANIFEST" | grep -v '^[[:space:]]*$' | cut -f2- | sort)"

if [ "$derived" != "$pinned" ]; then
  echo "ABSENCE-SITE-CENSUS: FAIL — $SOURCE no longer matches the pinned site set." >&2
  echo "  Rows only in the TREE (classify each one, then add it to the manifest):" >&2
  comm -23 <(printf '%s\n' "$derived") <(printf '%s\n' "$pinned") | sed 's/^/    + /' >&2
  echo "  Rows only in the MANIFEST (the site moved or went away — drop or re-pin it):" >&2
  comm -13 <(printf '%s\n' "$derived") <(printf '%s\n' "$pinned") | sed 's/^/    - /' >&2
  echo "  A NEW row in the 'verdict' class is the one that matters: it is another site reading" >&2
  echo "  'absent from grammar_tree' as a property of the grammar. See ENGINE-UNIVERSAL-SERVICES.53." >&2
  exit 1
fi

engine_rows=$(printf '%s\n' "$pinned" | awk -F'\t' '$1 == "engine"' | awk -F'\t' '{ s += $3 } END { print s + 0 }')
test_rows=$(printf '%s\n' "$pinned" | awk -F'\t' '$1 == "test"' | awk -F'\t' '{ s += $3 } END { print s + 0 }')
verdicts=$(grep -v '^#' "$MANIFEST" | awk -F'\t' '$1 == "verdict" { s += $4 } END { print s + 0 }')
guards=$(grep -v '^#' "$MANIFEST" | awk -F'\t' '$1 == "guard" { s += $4 } END { print s + 0 }')

echo "ABSENCE-SITE-CENSUS: OK — engine sites=${engine_rows} (verdict=${verdicts}, guard=${guards}), test-fixture sites=${test_rows}"
