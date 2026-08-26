#!/usr/bin/env bash
# registry_escaping_arms.sh — RED/GREEN control for the `<meta:registry-escaping>` arm of
# `scripts/check_doctrines.sh`. Owning leaf: `DOCTRINE-GAP-OWNERSHIP.16` (b).
#
# ⭐ WHY IT IS A SEPARATE SCRIPT RATHER THAN A `--self-test` INSIDE THE DRIVER. The driver's whole
#   contract is "run every doctrine and exit nonzero iff any failed"; giving it a mode that
#   deliberately fails would put a self-failing path inside the thing every commit runs. Instead the
#   arms run the REAL driver against a MUTATED COPY, so what is proven is the shipped code.
#
# ⛔ THE RED ARM RECONSTRUCTS THE FOUNDING DEFECT VERBATIM — the two command substitutions that ran
#   from the `CORPUS-KEY-INTEGRITY` row on 2026-08-25, printing "match: command not found" and
#   "syntax: command not found" while the driver exited 0.
#
# ⚠️ THE GREEN ARM IS NOT OPTIONAL. Without it, a RED arm proves only that SOMETHING about the copy
#   is refused — not that the ESCAPING is what did it ([[a-control-that-cannot-fail-is-not-a-control]]).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT" || exit 1

WORK="rust/target/registry_escaping_arms"
mkdir -p "$WORK"
DRIVER="scripts/check_doctrines.sh"
trap 'rm -f "$ROOT/$WORK/red.sh" "$ROOT/$WORK/green.sh"' EXIT

arms=0; passed=0

# The arm reads its own findings out of the driver's report line, so an unrelated doctrine failing in
# the caller's tree cannot be mistaken for this arm firing.
verdict() { grep -o '[✓✗] [A-Z]*  <meta:registry-escaping>' "$1" | head -1; }

# ── GREEN — the shipped registry, unmutated, must PASS this arm.
arms=$((arms + 1))
cp "$DRIVER" "$WORK/green.sh"
bash "$WORK/green.sh" >"$WORK/green.log" 2>&1
if verdict "$WORK/green.log" | grep -q '✓'; then
  passed=$((passed + 1)); echo "REGISTRY-ESCAPING-ARMS: GREEN — the shipped registry passes the escaping arm"
else
  echo "REGISTRY-ESCAPING-ARMS: GREEN FAILED — the real registry must pass; see $WORK/green.log" >&2
fi

# ── RED — re-introduce the founding defect: UNESCAPE the backticks in the CORPUS-KEY-INTEGRITY row.
arms=$((arms + 1))
awk '
  /^DOCTRINES=\(/ { inreg = 1 }
  inreg && /^  "CORPUS-KEY-INTEGRITY\|/ { gsub(/\\`/, "`") }
  { print }
' "$DRIVER" > "$WORK/red.sh"
if cmp -s "$DRIVER" "$WORK/red.sh"; then
  echo "REGISTRY-ESCAPING-ARMS: RED NOT EVALUATED — the fixture edit changed nothing; the row moved" >&2
else
  bash "$WORK/red.sh" >"$WORK/red.log" 2>&1
  if verdict "$WORK/red.log" | grep -q '✗'; then
    passed=$((passed + 1)); echo "REGISTRY-ESCAPING-ARMS: RED — un-escaping the founding row is REFUSED by the arm"
  else
    echo "REGISTRY-ESCAPING-ARMS: RED FAILED — the founding defect was not refused; see $WORK/red.log" >&2
  fi
fi

# ── RED-2 — the defect the third fix's guard could not see, and the reason this one reads SOURCE.
# The `|`-count guard passes here: the row's separator count is untouched. Only the escaping arm fires.
arms=$((arms + 1))
if [ -f "$WORK/red.log" ]; then
  if grep -q 'command not found' "$WORK/red.log" && grep -q 'ALL 30 enforced doctrines PASS\|<meta:registry> —' "$WORK/red.log"; then
    passed=$((passed + 1)); echo "REGISTRY-ESCAPING-ARMS: RED-2 — the substitution really RAN, and the |-count guard did not notice"
  elif grep -q 'command not found' "$WORK/red.log"; then
    passed=$((passed + 1)); echo "REGISTRY-ESCAPING-ARMS: RED-2 — the substitution really RAN (bash printed 'command not found')"
  else
    echo "REGISTRY-ESCAPING-ARMS: RED-2 FAILED — no substitution observed, so the fixture is not the founding defect" >&2
  fi
else
  echo "REGISTRY-ESCAPING-ARMS: RED-2 NOT EVALUATED — the RED arm produced no log" >&2
fi

echo "REGISTRY-ESCAPING-ARMS: arms=${passed}/${arms}"
[ "$passed" = "$arms" ] || exit 1
exit 0
