#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.36 — the adversarial probe for PARSE-COST-RATCHET's TYPED ACCEPTANCE.
#
# ⛔ WHY IT EXISTS. `accepted_rises.tsv` is the first mechanism in this gate that lets a measured
# RISE pass. A mechanism that only ever says yes is indistinguishable from a hole
# ([[a-check-whose-inputs-all-pass-has-not-been-tested]]), so every way it must REFUSE is fired
# here and observed.
#
# Each arm perturbs ONE input, runs the full ratchet (tier 2, ~2.5 min), asserts the expected exit
# code and message, then restores. The tracked baseline and the acceptance record are backed up
# first and restored in a trap, because the promoted baseline is uncommitted while this runs and
# `git checkout` would discard it.
#
#   bash docs/tasks/artifacts/engine_universal_services/accepted_rise_gate/probe.sh
#
# Exit 0 = every arm refused as specified. Exit 1 = an arm did NOT refuse (a hole).
set -uo pipefail
# ⛔ FIVE levels, not four: accepted_rise_gate -> engine_universal_services -> artifacts -> tasks
# -> docs -> repo root. The first cut of this probe used four, resolved ROOT to `docs/`, and every
# arm then reported rc=127 "No such file or directory" — which its own expected-message assertion
# refused rather than scoring as a refusal. A probe that cannot find the gate must not read as a
# probe that found a hole.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -f scripts/check_parse_cost_ratchet.sh ] || { echo "probe: not at the repo root ($ROOT)" >&2; exit 2; }

ART="docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet"
ACCEPTED="$ART/accepted_rises.tsv"
BACKUP="$(mktemp -d "${TMPDIR:-/tmp}/accepted_rise_probe.XXXXXX")"
LOG="$BACKUP/run.log"
PASS=0
FAIL=0

cleanup() {
  cp -f "$BACKUP/accepted_rises.tsv" "$ACCEPTED" 2>/dev/null
  cp -f "$BACKUP/cost.md" "$ART/cost.md" 2>/dev/null
  cp -f "$BACKUP/entries.tsv" "$ART/entries.tsv" 2>/dev/null
  echo "probe: restored $ART from $BACKUP"
}
trap cleanup EXIT

cp -f "$ACCEPTED" "$BACKUP/accepted_rises.tsv"
cp -f "$ART/cost.md" "$BACKUP/cost.md"
cp -f "$ART/entries.tsv" "$BACKUP/entries.tsv"
echo "probe: backed up the baseline + acceptance record to $BACKUP"

# Run the ratchet's tier 2 and check the outcome.
#   $1 arm name   $2 expected exit code   $3 a string the output must contain
arm() {
  local name="$1" want_rc="$2" want_msg="$3" rc
  PGEN_PARSE_COST_REMEASURE=1 bash scripts/check_parse_cost_ratchet.sh > "$LOG" 2>&1
  rc=$?
  if [ "$rc" = "$want_rc" ] && grep -qF "$want_msg" "$LOG"; then
    echo "  ✓ $name — rc=$rc and the message names it"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $name — expected rc=$want_rc containing:"
    echo "      $want_msg"
    echo "    got rc=$rc:"
    sed 's/^/      /' "$LOG" | tail -12
    FAIL=$((FAIL + 1))
  fi
}

# ── arm 1: a rise with NO row at all must FAIL ──────────────────────────────────────────────────
# The baseline is lowered so this tree measures as a rise, and the acceptance record is emptied.
echo "arm 1: an unaccepted rise"
grep -v $'^entries\t\|^memo_hits\t' "$BACKUP/accepted_rises.tsv" > "$ACCEPTED"
python3 "$(dirname "${BASH_SOURCE[0]}")/perturb.py" "$ART/entries.tsv" entries 1000 > "$BACKUP/p1" || exit 2
arm "an unaccepted rise is REFUSED" 1 "Costs are REJECTED, not traded"

# ── arm 2: a row whose INVARIANT no longer holds must FAIL ───────────────────────────────────────
# The entries rise from arm 1 is now COVERED by a row — and `committed` is lowered too, so the fresh
# run shows committed rising, which is exactly what `pure_memo_lookups` forbids. The gate must refuse
# its own acceptance rather than honour it.
echo "arm 2: an acceptance whose invariant is broken"
cp -f "$BACKUP/accepted_rises.tsv" "$ACCEPTED"
python3 "$(dirname "${BASH_SOURCE[0]}")/perturb.py" "$ART/entries.tsv" committed 500 > "$BACKUP/p2" || exit 2
E_FROM="$(cut -f1 "$BACKUP/p1")"; E_TO="$(cut -f2 "$BACKUP/p1")"
printf 'entries\t%s\t%s\tpure_memo_lookups\tPROBE\tarm 2: committed moved too, so this acceptance must be refused\n' \
  "$E_FROM" "$E_TO" >> "$ACCEPTED"
arm "a broken invariant is REFUSED" 1 "that invariant NO LONGER HOLDS"

# ── arm 3: a row naming an UNKNOWN invariant must FAIL ───────────────────────────────────────────
echo "arm 3: an unknown invariant name"
cp -f "$BACKUP/accepted_rises.tsv" "$ACCEPTED"
printf 'entries\t1\t2\tbecause_i_said_so\tPROBE\tarm 3\n' >> "$ACCEPTED"
arm "an unknown invariant is REFUSED" 1 "is not coded in this gate"

# ── arm 4: a MALFORMED row must FAIL rather than be skipped ─────────────────────────────────────
echo "arm 4: a malformed row"
cp -f "$BACKUP/accepted_rises.tsv" "$ACCEPTED"
printf 'entries\tnot-a-number\t2\tpure_memo_lookups\tPROBE\tarm 4\n' >> "$ACCEPTED"
arm "a malformed row is REFUSED" 1 "from/to are not integers"

# ── arm 5: the CONTROL — the tree as it stands must be GREEN ─────────────────────────────────────
# Without this arm a probe that broke the gate outright would score 4/4.
echo "arm 5: control — the unperturbed tree"
cp -f "$BACKUP/accepted_rises.tsv" "$ACCEPTED"
cp -f "$BACKUP/cost.md" "$ART/cost.md"
cp -f "$BACKUP/entries.tsv" "$ART/entries.tsv"
arm "the unperturbed tree HOLDS" 0 "parse-cost-ratchet: OK"

echo "ACCEPTED-RISE-GATE: arms_passed=$PASS arms_failed=$FAIL"
[ "$FAIL" = 0 ] || exit 1
