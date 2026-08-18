#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2k — the adversarial probe for the `accepts_invalid` manifest class.
#
# ⛔ WHY IT EXISTS. The repro manifest gained a FIFTH class, and a class whose rows all pass today
# proves nothing ([[a-check-whose-inputs-all-pass-has-not-been-tested]]). The whole point of
# `accepts_invalid` is that it goes RED *when the defect is FIXED* — the mirror of `defect` — so that
# direction has to be observed, not assumed. The two coherence guards added alongside it are fired
# here too, because a manifest is an oracle and an unreadable row in it must never read as green.
#
# Each arm perturbs the manifest, runs the oracle, asserts exit code + message, then restores from a
# backup (never from git: the manifest is uncommitted while this runs).
#
#   bash docs/tasks/artifacts/sv_corpus_grad/accepts_invalid_class/probe.sh
#
# Exit 0 = every arm behaved as specified. Exit 1 = an arm did not.
set -uo pipefail
# FIVE levels: accepts_invalid_class -> sv_corpus_grad -> artifacts -> tasks -> docs -> root.
# The guard below is not decoration: this probe was written with four and the guard caught it.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -f stimuli/sv/run_adjudication_repros.py ] || { echo "probe: not at the repo root ($ROOT)" >&2; exit 2; }

MANIFEST="stimuli/sv/adjudication_repros/MANIFEST.tsv"
BACKUP="$(mktemp -d "${TMPDIR:-/tmp}/accepts_invalid_probe.XXXXXX")"
LOG="$BACKUP/run.log"
PASS=0
FAIL=0

cleanup() { cp -f "$BACKUP/MANIFEST.tsv" "$MANIFEST" 2>/dev/null; echo "probe: restored $MANIFEST"; }
trap cleanup EXIT
cp -f "$MANIFEST" "$BACKUP/MANIFEST.tsv"

arm() {
  local name="$1" want_rc="$2" want_msg="$3" rc
  python3 stimuli/sv/run_adjudication_repros.py > "$LOG" 2>&1
  rc=$?
  if [ "$rc" = "$want_rc" ] && grep -qF "$want_msg" "$LOG"; then
    echo "  ✓ $name — rc=$rc and the message names it"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $name — expected rc=$want_rc containing:"
    echo "      $want_msg"
    echo "    got rc=$rc:"
    sed 's/^/      /' "$LOG" | tail -10
    FAIL=$((FAIL + 1))
  fi
}

# ── arm 1: an `accepts_invalid` row that starts REJECTING must FAIL, naming the round trip ───────
# Simulated by pointing the row at text the parser already refuses — the same observable state the
# owning fix will produce.
echo "arm 1: an over-acceptance that got FIXED"
python3 - "$MANIFEST" <<'PY'
import sys
path = sys.argv[1]
lines = open(path, encoding="utf-8").read().splitlines()
head = lines[0].split("\t")
i_id, i_cls = head.index("id"), head.index("class")
out = []
for line in lines:
    f = line.split("\t")
    if len(f) > i_cls and f[i_cls] == "accepts_invalid" and f[i_id].startswith("accepts_invalid_keyword_component_indexed"):
        # re-point this row's id at a reproducer that REJECTS today, which is what "fixed" looks like
        f[i_id] = "invalid_keyword_method_name.sv"
        line = "\t".join(f)
    out.append(line)
open(path, "w", encoding="utf-8").write("\n".join(out) + "\n")
PY
arm "a FIXED over-acceptance is REPORTED" 1 "flip its \`expect\` to REJECT"

# ── arm 2: an UNKNOWN class must be refused, not skipped ──────────────────────────────────────────
echo "arm 2: an unknown class"
cp -f "$BACKUP/MANIFEST.tsv" "$MANIFEST"
python3 - "$MANIFEST" <<'PY'
import sys
path = sys.argv[1]
lines = open(path, encoding="utf-8").read().splitlines()
head = lines[0].split("\t")
i_cls = head.index("class")
f = lines[1].split("\t"); f[i_cls] = "acepts_invalid"      # a plausible typo
lines[1] = "\t".join(f)
open(path, "w", encoding="utf-8").write("\n".join(lines) + "\n")
PY
arm "an unknown class is REFUSED" 1 "is not one of"

# ── arm 3: an INCOHERENT class/expect pair must be refused ────────────────────────────────────────
echo "arm 3: a class that contradicts its expectation"
cp -f "$BACKUP/MANIFEST.tsv" "$MANIFEST"
python3 - "$MANIFEST" <<'PY'
import sys
path = sys.argv[1]
lines = open(path, encoding="utf-8").read().splitlines()
head = lines[0].split("\t")
i_exp, i_cls = head.index("expect"), head.index("class")
for n, line in enumerate(lines[1:], 1):
    f = line.split("\t")
    if f[i_cls] == "accepts_invalid":
        f[i_exp] = "REJECT"      # the incoherent pair the class exists to prevent
        lines[n] = "\t".join(f)
        break
open(path, "w", encoding="utf-8").write("\n".join(lines) + "\n")
PY
arm "an incoherent class/expect pair is REFUSED" 1 "The class IS the claim"

# ── arm 4: the CONTROL — the unperturbed manifest must be GREEN ───────────────────────────────────
echo "arm 4: control — the unperturbed manifest"
cp -f "$BACKUP/MANIFEST.tsv" "$MANIFEST"
arm "the unperturbed manifest HOLDS" 0 "failures=0"

echo "ACCEPTS-INVALID-CLASS: arms_passed=$PASS arms_failed=$FAIL"
[ "$FAIL" = 0 ] || exit 1
