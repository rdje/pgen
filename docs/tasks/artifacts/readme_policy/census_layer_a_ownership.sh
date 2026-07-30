#!/usr/bin/env bash
# docs/tasks/artifacts/readme_policy/census_layer_a_ownership.sh
# README-POLICY.2 — measured census of layer-A `MEMORY.md`, answering the ONE question a
# trim must not assume: IS EACH ENTRY DURABLY OWNED SOMEWHERE ELSE?
#
# ⛔ WHY THIS EXISTS, AND WHY IT IS NOT A SIZE REPORT.
#   `.1` shipped a README byte cap after routing every removed content class to a home it had
#   VERIFIED. Its most expensive lesson was the row it got wrong: `Key Project Paths` was routed
#   to `docs/book/src/source-map.md` on the strength of that chapter's NAME, and measuring found
#   0 path hits there — the repository's only path inventory would have been deleted into a
#   chapter that could not receive it. ⭐ A canonical home named from a plausible title is not a
#   verified destination. Layer A is the same shape at larger scale: 34 narrative entries whose
#   safety rests entirely on the claim "the task-trees and git already hold this."
#   This instrument turns that claim into a per-entry measurement.
#
# WHAT IT MEASURES
#   A. Block census — head / how-to-resume / north-star / current-state, lines + bytes + share.
#   B. Per-entry ownership in the "Current state" block: every `PGEN-<FAMILY>-<NNNN>` slice id is
#      resolved against git commit subjects (layer D) AND against `docs/tasks/` (layer B).
#      An entry naming NO slice id is reported UNANCHORED — it is not thereby unowned, but it
#      cannot be pruned on this instrument's evidence alone.
#   C. North-star `[[record]]` links resolved against `docs/decisions/` (layer C).
#   D. CALIBRATION, both polarities, REFUSING on a miss (feedback_instrument_needs_ground_truth):
#      a census that reports "everything is owned" is indistinguishable from one that cannot
#      see, so it must also correctly report a control that is NOT owned.
#
# Read-only. Deterministic apart from `git log`, which only grows. Exit 2 = REFUSED (cannot
# judge); exit 1 = calibration failed; exit 0 = census printed and calibrated.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"
TARGET="${1:-MEMORY.md}"

[ -f "$TARGET" ] || { printf 'layer-a-census: REFUSED — %s not found (ROOT=%s)\n' "$TARGET" "$ROOT" >&2; exit 2; }
[ -d docs/tasks ] || { printf 'layer-a-census: REFUSED — docs/tasks/ (layer B) missing; ownership is unjudgeable\n' >&2; exit 2; }
[ -d docs/decisions ] || { printf 'layer-a-census: REFUSED — docs/decisions/ (layer C) missing; ownership is unjudgeable\n' >&2; exit 2; }

# One git read, reused: every commit subject in the repository, materialised as a FILE.
#
# ⛔ DO NOT rewrite this as `printf '%s\n' "$SUBJECTS" | grep -q …`. Under `set -o pipefail`,
# `grep -q` exits at its FIRST match and closes the pipe; the upstream `printf` — still holding
# ~549 KB of subjects against a ~64 KB pipe buffer — then takes SIGPIPE (rc 141), and pipefail
# promotes that to the pipeline's status. The match is reported as a MISS. Measured here while
# writing this census: `PIPESTATUS=(141 0)` — grep said FOUND, printf's corpse set the verdict.
# ⭐ The polarity is inverted exactly where it hurts: the EARLIER the match (i.e. the more
# recently the slice was committed), the more reliably it is reported as NOT owned in git.
# This is the class already root-caused at `scripts/check_diagnosis_evidence.sh:101-109`
# (STORE-AWARE-GEN.4b.12) and fixed there by the same means used here: give grep a FILE, so
# there is no upstream writer to kill. Reproduced 5/5 — deterministic, not flaky.
SUBJECTS_FILE="$(mktemp)"
trap 'rm -f "$SUBJECTS_FILE" "${tmp_unowned:-}"' EXIT
git log --format='%s' > "$SUBJECTS_FILE" 2>/dev/null || true
[ -s "$SUBJECTS_FILE" ] || { printf 'layer-a-census: REFUSED — no git history readable; layer D cannot be consulted\n' >&2; exit 2; }

total_lines=$(wc -l < "$TARGET" | tr -d ' ')
total_bytes=$(wc -c < "$TARGET" | tr -d ' ')

printf '== layer-A census: %s ==\n' "$TARGET"
printf 'total: %s lines, %s bytes (%s bytes per line)\n\n' \
  "$total_lines" "$total_bytes" "$(( total_bytes / (total_lines > 0 ? total_lines : 1) ))"

# ---------------------------------------------------------------- A. block census
printf '== A. block census ==\n'
printf '%-6s %-6s %-9s %-7s  %s\n' 'start' 'lines' 'bytes' 'share' 'block'
printf '%-6s %-6s %-9s %-7s  %s\n' '-----' '-----' '-----' '-----' '-----'
awk -v tot="$total_bytes" '
  /^## / {
    if (name != "") printf "%-6d %-6d %-9d %6.1f%%  %s\n", start, NR-start, bytes, 100*bytes/tot, name
    name = substr($0, 4); start = NR; bytes = 0
  }
  {
    if (name == "") { hstart = (hstart ? hstart : 1); hbytes += length($0) + 1; hlines = NR }
    else bytes += length($0) + 1
  }
  END {
    if (name != "") printf "%-6d %-6d %-9d %6.1f%%  %s\n", start, NR-start+1, bytes, 100*bytes/tot, name
    printf "%-6d %-6d %-9d %6.1f%%  %s\n", 1, hlines, hbytes, 100*hbytes/tot, "(header — the layer-A contract statement)"
  }
' "$TARGET"

# Locate the "Current state" block — the one the file's OWN heading says must be overwritten.
cs_start=$(grep -nE '^## Current state' "$TARGET" | head -1 | cut -d: -f1 || true)
[ -n "${cs_start:-}" ] || { printf 'layer-a-census: REFUSED — no "## Current state" heading in %s\n' "$TARGET" >&2; exit 2; }
cs_heading=$(sed -n "${cs_start}p" "$TARGET")

printf '\n== B. "Current state" block — per-entry durable ownership ==\n'
printf 'block heading (verbatim): %s\n' "$cs_heading"
printf 'entries start at line %s\n\n' "$((cs_start + 1))"

owned=0; unanchored=0; unowned=0; entries=0
tmp_unowned="$(mktemp)"

while IFS= read -r ln; do
  line=$(sed -n "${ln}p" "$TARGET")
  case "$line" in ''|'#'*) continue ;; esac
  entries=$((entries + 1))
  bytes=$(( ${#line} + 1 ))
  ids=$(printf '%s\n' "$line" | grep -oE 'PGEN-[A-Z0-9]+(-[A-Z0-9]+)*-[0-9]{4}' | sort -u || true)
  label=$(printf '%s' "$line" | sed -E 's/^- //; s/\*\*//g' | cut -c1-58)
  if [ -z "$ids" ]; then
    unanchored=$((unanchored + 1))
    printf '  %-4s %7dB  UNANCHORED (no slice id)   %s\n' "L$ln" "$bytes" "$label"
    continue
  fi
  miss=""
  for id in $ids; do
    grep -qF "$id" "$SUBJECTS_FILE" || miss="$miss $id"
  done
  n_ids=$(printf '%s\n' "$ids" | wc -l | tr -d ' ')
  if [ -n "$miss" ]; then
    unowned=$((unowned + 1))
    printf '  %-4s %7dB  ⛔ NOT IN GIT:%s  %s\n' "L$ln" "$bytes" "$miss" "$label"
    printf 'L%s:%s\n' "$ln" "$miss" >> "$tmp_unowned"
  else
    owned=$((owned + 1))
    printf '  %-4s %7dB  ✅ owned (%s slice id(s) in git)  %s\n' "L$ln" "$bytes" "$n_ids" "$label"
  fi
done < <(seq "$((cs_start + 1))" "$total_lines")

printf '\n  entries=%s  owned-in-git=%s  unanchored=%s  NOT-in-git=%s\n' \
  "$entries" "$owned" "$unanchored" "$unowned"

# Tree-file existence for every family named by a resolved slice id (layer B, not just D).
printf '\n  layer-B tree files for the families named in this block:\n'
sed -n "$((cs_start + 1)),${total_lines}p" "$TARGET" \
  | grep -oE 'PGEN-[A-Z0-9]+(-[A-Z0-9]+)*-[0-9]{4}' \
  | sed -E 's/^PGEN-//; s/-[0-9]{4}$//' | sort -u \
  | while read -r fam; do
      if [ -f "docs/tasks/${fam}.md" ]; then
        printf '    ✅ docs/tasks/%s.md\n' "$fam"
      else
        printf '    ⚠️  docs/tasks/%s.md  ABSENT — family named but no tree file\n' "$fam"
      fi
    done

# ---------------------------------------------------------------- C. layer-C links
printf '\n== C. north-star [[record]] links → docs/decisions/ (layer C) ==\n'
c_missing=0
while read -r rec; do
  [ -n "$rec" ] || continue
  if [ -f "docs/decisions/${rec}.md" ]; then
    printf '    ✅ %s\n' "$rec"
  else
    printf '    ⛔ %s — NO record file\n' "$rec"
    c_missing=$((c_missing + 1))
  fi
done < <(grep -oE '\[\[[a-z0-9_]+\]\]' "$TARGET" | tr -d '[]' | sort -u)
printf '    missing=%s\n' "$c_missing"

# ---------------------------------------------------------------- D. calibration
# ⭐ THE ARMS THAT MAKE THE ABOVE MEAN ANYTHING. A census reporting "all owned" is
# indistinguishable from a census that cannot see, so it must also get a KNOWN-ABSENT
# control right. Both polarities, or this instrument REFUSES its own output.
printf '\n== D. calibration (both polarities — REFUSES on a miss) ==\n'
cal_fail=0
cal() { # name expected actual
  if [ "$2" = "$3" ]; then printf '    ✅ %-46s %s\n' "$1" "$3"
  else printf '    ⛔ %-46s expected=%s actual=%s\n' "$1" "$2" "$3"; cal_fail=$((cal_fail + 1)); fi
}

# POSITIVE — a slice id known to be in git (this tree's own first commit).
in_git=no; grep -qF 'PGEN-README-POLICY-0001' "$SUBJECTS_FILE" && in_git=yes
cal 'positive: PGEN-README-POLICY-0001 in git' 'yes' "$in_git"

# NEGATIVE — a well-formed slice id that CANNOT exist. If this resolves, the matcher is blind.
neg=no; grep -qF 'PGEN-NO-SUCH-FAMILY-9999' "$SUBJECTS_FILE" && neg=yes
cal 'negative: PGEN-NO-SUCH-FAMILY-9999 in git' 'no' "$neg"

# POSITIVE — a layer-C record this file links and that exists.
recpos=no; [ -f docs/decisions/feedback_done_bar_is_first_tier_only.md ] && recpos=yes
cal 'positive: layer-C record resolves' 'yes' "$recpos"

# NEGATIVE — a record that must not exist.
recneg=no; [ -f docs/decisions/zz_control_record_that_must_not_exist.md ] && recneg=yes
cal 'negative: bogus layer-C record resolves' 'no' "$recneg"

# STRUCTURAL — the block this census prunes is the one the file itself says not to append to.
selfcontra=no
case "$cs_heading" in *'do not append'*) selfcontra=yes ;; esac
cal 'structural: block heading says "do not append"' 'yes' "$selfcontra"

if [ "$cal_fail" -ne 0 ]; then
  printf '\nlayer-a-census: CALIBRATION FAILED (%s arm(s)) — census above is NOT trustworthy.\n' "$cal_fail" >&2
  exit 1
fi
printf '\nlayer-a-census: OK — calibrated %s/%s arms.\n' 5 5
