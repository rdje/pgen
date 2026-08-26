#!/usr/bin/env bash
# gap_claim_census_replay.sh — the CALIBRATION harness behind `DOCTRINE-GAP-OWNERSHIP.15`
# and the `GAP-CLAIM-CENSUS` doctrine (`scripts/check_gap_claims.sh`).
#
# ⛔ WHY THIS IS TRACKED RATHER THAN A SCRATCH SCRIPT: `ENGINE-UNIVERSAL-SERVICES.54` (`-0097`)
# closed on exactly this — a leaf whose bisect evidence was cited from an IGNORED path, so the
# citation would have dangled. Every number the leaf publishes is re-derivable from HERE.
#
# It answers the question a guard must answer BEFORE it is made blocking:
#   *replayed over real history, what would this have done?*
# `ROUTING-EVIDENCE`'s first cut would not have caught its own founding incident because it was
# calibrated against a phrasing nobody had written. This harness judges each commit's ADDED lines
# against THAT COMMIT'S OWN version of the file, which is what the pre-commit hook sees.
#
# The predicate and the discharge are READ OUT OF THE SHIPPED ENFORCER, never re-typed here — a
# re-typed regex is a second source of truth that drifts silently (DERIVED_STATE_CONTAINMENT R1/R3).
#
# USAGE
#   docs/tasks/artifacts/doctrine_gap_ownership/gap_claim_census_replay.sh [N] [--dump]
#     N        how many commits back to replay (default 150 — the window the leaf publishes)
#     --dump   print every line that WOULD BLOCK, for one-by-one adjudication (the `.15`(b) input)
#   ...--narrow-discharge   re-run with the discharge cut back to a bare grep/git list. This is the
#                           CONTROL for the widening ruling: it must report MORE blocks than the
#                           shipped list, or the widening bought nothing.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT" || exit 1

ENFORCER="scripts/check_gap_claims.sh"
[ -r "$ENFORCER" ] || { echo "replay: $ENFORCER is missing — nothing to calibrate" >&2; exit 2; }

CLAIM_RE="$(sed -n "s/^CLAIM_RE='\(.*\)'$/\1/p"  "$ENFORCER")"
CENSUS_RE="$(sed -n "s/^CENSUS_RE='\(.*\)'$/\1/p" "$ENFORCER")"
[ -n "$CLAIM_RE" ] && [ -n "$CENSUS_RE" ] || {
  echo "replay: could not read CLAIM_RE/CENSUS_RE out of $ENFORCER — the assignment spelling moved." >&2
  echo "        REFUSING rather than replaying with an empty predicate: an empty regex matches" >&2
  echo "        every line, and the failure would be silent in the ALARMING direction." >&2
  exit 2; }

N="${1:-150}"; case "$N" in ''|*[!0-9]*) N=150 ;; esac
DUMP=""; NARROW=""
for a in "$@"; do
  [ "$a" = "--dump" ] && DUMP=1
  [ "$a" = "--narrow-discharge" ] && NARROW=1
done
# The narrow control: the discharge list as it stood BEFORE the widening ruling.
[ -n "$NARROW" ] && CENSUS_RE='git grep|grep -r|grep -rn|grep -c|grep -l|git ls-files|git log -s|git log --grep|rg -|--dump-|--report-|scripts/check_|make -c rust|comm -'

WORK="rust/target/gap_claim_census/replay"; mkdir -p "$WORK"

fired=0; blocked=0; c_fired=0; c_blocked=0; seen=0
while read -r h; do
  seen=$((seen + 1)); cf=0; cb=0
  for f in $(git show "$h" --name-only --format='' --diff-filter=ACM 2>/dev/null \
             | grep -E '^docs/tasks/.*\.md$' || true); do
    git show "$h:$f" > "$WORK/new.md" 2>/dev/null || continue
    git show "$h" -U0 -- "$f" 2>/dev/null | awk '
      /^@@/ { split($3,a,","); ln=a[1]+0; next }
      /^\+\+\+/ { next }
      /^\+/ { print ln; ln++; next }' | sort -un > "$WORK/nums.txt"
    [ -s "$WORK/nums.txt" ] || continue
    out="$(awk -v CLAIM_RE="$CLAIM_RE" -v CENSUS_RE="$CENSUS_RE" -v F="$f" '
      FNR==NR { added[$1+0]=1; next }
      { line[FNR]=$0; low[FNR]=tolower($0) }
      END{
        for(i=1;i<=FNR;i++){ if(line[i] ~ /^#{1,6} /) cur=i; sec[i]=cur }
        for(i=1;i<=FNR;i++){ if(low[i] ~ CENSUS_RE) has[sec[i]]=1 }
        for(i=1;i<=FNR;i++){
          if(added[i] && low[i] ~ CLAIM_RE)
            printf "%s\t%s:%d\t%s\n", (has[sec[i]] ? "BACKED" : "BLOCKED"), F, i, substr(line[i],1,170)
        }
      }' "$WORK/nums.txt" "$WORK/new.md")"
    [ -n "$out" ] || continue
    cf=$((cf + $(printf '%s\n' "$out" | grep -c .)))
    nb=$(printf '%s\n' "$out" | grep -c '^BLOCKED' || true)
    cb=$((cb + nb))
    [ -n "$DUMP" ] && printf '%s\n' "$out" | grep '^BLOCKED' | sed "s|^|${h:0:8}\t|"
  done
  fired=$((fired + cf)); blocked=$((blocked + cb))
  [ "$cf" -gt 0 ] && c_fired=$((c_fired + 1))
  [ "$cb" -gt 0 ] && c_blocked=$((c_blocked + 1))
done < <(git log --format='%H' -"$N")

printf '\n== GAP-CLAIM-CENSUS replay over %d commits%s ==\n' "$seen" "${NARROW:+  (NARROW-DISCHARGE CONTROL)}"
printf '  claim lines ADDED to docs/tasks : %d, in %d of %d commits\n' "$fired" "$c_fired" "$seen"
printf '  WOULD HAVE BLOCKED              : %d line(s), in %d of %d commits (%s%%)\n' \
  "$blocked" "$c_blocked" "$seen" "$(awk -v a="$c_blocked" -v b="$seen" 'BEGIN{printf "%.1f", (b?100*a/b:0)}')"
