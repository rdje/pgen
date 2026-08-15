#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/family_share_gate/probe.sh
#
# ENGINE-UNIVERSAL-SERVICES.21 acceptance (f) — THE RED PROBES FOR THE FAMILY-SHARE GATE.
#
#   A control never observed failing is not known to work (`docs/CLAIM_VERIFICATION.md` §6).
#
# `--verify-family-share` is a gate whose whole job is to REFUSE. Every arm below is driven
# against a MUTATED COPY of the tracked artifact (never the tracked one — that is why the mode
# takes `--family-share-artifact` as a parameter), and each is asserted to exit non-zero for the
# right reason. One GREEN control proves the suite is not simply failing everything.
#
# ⭐ ARM 6 is the one that matters most: it replays the pre-`.21` classifier — the predicate that
# produced the wrong `0.681 %` and the wrong `~35x` bound that four surfaces carried — and shows
# this gate refuses it. That is the difference between "the number is right now" and "the number
# cannot go wrong unnoticed".
#
# Usage:  bash docs/tasks/artifacts/engine_universal_services/family_share_gate/probe.sh
# Output: docs/tasks/artifacts/engine_universal_services/family_share_gate/probe.txt
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT" || exit 2
OUT="$HERE/probe.txt"

INSTRUMENT="stimuli/sv/corpus_parse_cost.py"
TRACKED="docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/family_share.json"
WORK="rust/target/family_share_gate"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0; fail=0

# $1 arm name · $2 expected exit code · $3 artifact path · $4 what it proves
arm() {
  local name="$1" want="$2" art="$3" proves="$4"
  local out rc
  out="$(python3 "$INSTRUMENT" --verify-family-share --family-share-artifact "$art" 2>&1)"
  rc=$?
  if [ "$rc" -eq "$want" ]; then
    printf '  ✓ %-34s exit %d (expected %d) — %s\n' "$name" "$rc" "$want" "$proves"
    pass=$((pass + 1))
  else
    printf '  ✗ %-34s exit %d (expected %d) — %s\n' "$name" "$rc" "$want" "$proves"
    printf '%s\n' "$out" | sed 's/^/        /'
    fail=$((fail + 1))
  fi
  printf '%s\n' "$out" | sed 's/^/        | /' >> "$WORK/detail.txt"
}

# mutate one JSON field of a copy of the tracked artifact
mutate() {  # $1 out-name · $2 python expression body over `d`
  local out="$WORK/$1"
  python3 - "$TRACKED" "$out" "$2" <<'PY'
import json, sys
src, dst, expr = sys.argv[1], sys.argv[2], sys.argv[3]
with open(src, encoding="utf-8") as fh:
    d = json.load(fh)
exec(expr, {"d": d})
with open(dst, "w", encoding="utf-8") as fh:
    json.dump(d, fh, indent=2, sort_keys=True)
PY
  printf '%s' "$out"
}

# ⛔ NOT `{ … } | tee`. A pipeline runs its left side in a SUBSHELL, so the pass/fail counters
# would be discarded and this script would exit 0 while printing "8 failed" — the exact
# report-says-one-thing / exit-code-says-another shape `.20` slice 4's aggregator shipped with.
exec > >(tee "$OUT") 2>&1

printf '===========================================================================\n'
printf 'ENGINE-UNIVERSAL-SERVICES.21 (f) — family-share gate, RED probes\n'
printf '===========================================================================\n'
printf 'instrument: %s\n' "$INSTRUMENT"
printf 'artifact:   %s\n\n' "$TRACKED"

arm "GREEN tracked artifact" 0 "$TRACKED" \
    "the gate passes on the real tree, so the REDs below are not vacuous"

arm "RED 1 artifact absent" 2 "$WORK/does_not_exist.json" \
    "a missing derivation REFUSES; it never reads as 'nothing to check'"

a="$(mutate red2.json 'd["schema"] = "pgen.parse_cost.family_share/v0"')"
arm "RED 2 schema moved" 2 "$a" \
    "a reader that outlived its producer's schema refuses instead of unpacking"

a="$(mutate red3.json 'd["corpus_family_share_pct"] = "2.900"')"
arm "RED 3 carried != derived" 1 "$a" \
    "the carried CORPUS_FAMILY_SHARE_PCT can no longer be edited unopposed"

a="$(mutate red4.json 'd["blind_spot_factor"] = "35.7"')"
arm "RED 4 blind-spot factor stale" 1 "$a" \
    "the pre-.21 ~35x bound cannot survive beside a corrected share"

a="$(mutate red5.json 'd["identity"].pop("corpus inputs")')"
arm "RED 5 identity row missing" 1 "$a" \
    "an input with no row is UNGUARDED and says so, rather than passing"

a="$(mutate red6.json 'd["identity"]["classifier"]["sha256"] = "'"$(printf '_lr_base$|_lr_suffix(_r\\d+)?$' | shasum -a 256 | cut -d" " -f1)"'"')"
arm "RED 6 pre-.21 classifier" 1 "$a" \
    "THE FOUNDING DEFECT: the narrow predicate that produced 0.681 % is refused"

a="$(mutate red7.json 'd["identity"]["grammar"]["sha256"] = "0" * 64')"
arm "RED 7 grammar moved" 1 "$a" \
    "a grammar edit stales the share — the event .21 said would rot it silently"

a="$(mutate red8.json 'd["identity"]["generated parser"]["sha256"] = "0" * 64')"
arm "RED 8 generated parser moved" 1 "$a" \
    "a regenerated parser stales it too, at the moment cost can move"

a="$(mutate red9.json 'd["identity"]["corpus inputs"]["sha256"] = "0" * 64')"
arm "RED 9 corpus moved" 1 "$a" \
    "a submodule bump changes what was measured without touching one PGEN byte"

printf '\n%d passed, %d failed.\n' "$pass" "$fail"
if [ "$fail" -eq 0 ]; then
  printf '⭐ every refusal in the family-share gate has now been OBSERVED firing.\n'
else
  printf '⛔ an arm did not behave as declared — the gate is not proven.\n'
fi

# let the tee-writer drain before the shell exits, or the tail of the report is lost
exec 1>&- 2>&-
wait
exit "$fail"
