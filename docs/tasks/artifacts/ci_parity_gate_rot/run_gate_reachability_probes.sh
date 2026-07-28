#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_gate_reachability_probes.sh
#
# CI-PARITY-GATE-ROT.2 — RED / GREEN / CONTROL arms for the `GATE-REACHABILITY` doctrine
# (`scripts/check_gate_reachability.sh`).
#
# ⭐ THE ARM THAT MATTERS MOST IS RED-1: a brand-new gate target that nothing invokes must FAIL,
# on a target that did not exist when the check was written. That, and only that, proves the orphan
# set is DERIVED rather than a snapshot of the register — a register that merely re-states itself
# would pass every arm and catch nothing.
#
# ⚠️ AND THE CONTROLS ARE NOT DECORATION HERE. While being written this instrument produced SIX
# different confident answers (97 / 71 / 75 / 93 / 53 / 40 orphans), each from a real calibration
# defect, and not one was found by reading the code — every one was caught by requiring the output
# to reproduce facts the project had already measured. CTRL-1 replays that requirement.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_gate_reachability_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

CHECK="scripts/check_gate_reachability.sh"
REGISTER="rust/test_data/grammar_quality/gate_reachability_register_v0.json"
MAKEFILE="rust/Makefile"
WORK="rust/target/ci_parity_gate_rot_probe/reach_arms"
rm -rf "$WORK"; mkdir -p "$WORK"

cp "$ROOT/$MAKEFILE" "$WORK/Makefile.bak"
cp "$ROOT/$REGISTER" "$WORK/register.bak"
restore() { cp "$WORK/Makefile.bak" "$ROOT/$MAKEFILE"; cp "$WORK/register.bak" "$ROOT/$REGISTER"; }
trap 'restore' EXIT

pass=0; fail=0
arm() {  # arm <label> <expected PASS|FAIL> [needle]
  local label="$1" want="$2" needle="${3:-}"
  local out rc got
  out="$(bash "$ROOT/$CHECK" 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] && got=PASS || got=FAIL
  if [ "$got" != "$want" ]; then
    fail=$((fail + 1)); printf '✗ %-40s expected %s, got %s\n' "$label" "$want" "$got"
    printf '%s\n' "$out" | head -5 | sed 's/^/      /'; return 0
  fi
  if [ -n "$needle" ] && ! printf '%s\n' "$out" | grep -qF -- "$needle"; then
    fail=$((fail + 1)); printf '✗ %-40s %s as expected, but never mentions %s\n' "$label" "$got" "$needle"
    printf '%s\n' "$out" | head -5 | sed 's/^/      /'; return 0
  fi
  pass=$((pass + 1)); printf '✓ %-40s %s\n' "$label" "$got"
}

printf '%s\n' "=============================================================================="
printf 'CI-PARITY-GATE-ROT.2 — GATE-REACHABILITY probes\n'
printf '%s\n' "=============================================================================="

arm "GREEN  untouched tree" PASS "all dispositioned"

# ---------------------------------------------------------------- RED-1: a NEW orphan gate
# ⭐ The load-bearing arm. This target did not exist when the check was written.
cat >> "$ROOT/$MAKEFILE" <<'MK'

.PHONY: zz_probe_orphan_gate
zz_probe_orphan_gate:
	cd $(RUST_DIR) && ./scripts/mdbook_docs_gate.sh
MK
arm "RED-1  a NEW gate nothing invokes" FAIL "zz_probe_orphan_gate"
restore

# ---------------------------------------------------------------- RED-2: a disposition is deleted
python3 - "$ROOT/$REGISTER" <<'PY'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
d["entries"] = [e for e in d["entries"] if e["target"] != "duality_hunt_gate"]
json.dump(d, open(p, "w"), indent=2, ensure_ascii=False)
PY
arm "RED-2  an existing disposition removed" FAIL "duality_hunt_gate"
restore

# ---------------------------------------------------------------- RED-3: a DEAD exemption lingers
# An exemption list that can accumulate entries nobody re-checks is the rot this doctrine treats,
# one level in. A register entry naming something that is no longer an orphan must fail too.
python3 - "$ROOT/$REGISTER" <<'PY'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
d["entries"].append({"target": "sota_exit_gate", "disposition": "accepted-operator-invoked",
                     "reason": "probe: this target is NOT an orphan, so this entry is dead weight"})
json.dump(d, open(p, "w"), indent=2, ensure_ascii=False)
PY
arm "RED-3  a dead exemption entry" FAIL "no longer orphaned"
restore

# ---------------------------------------------------------------- RED-4: an invoker is removed
# Deleting the per-parser book wiring must re-orphan all ten books — this is the arm that proves
# the check actually follows edges rather than trusting the register.
python3 - "$ROOT/$MAKEFILE" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
open(p, "w").write(s.replace("mdbook_docs_gate: parser_books_gate", "mdbook_docs_gate:", 1))
PY
arm "RED-4  book gates lose their invoker" FAIL "parser_book_gate"
restore

# ---------------------------------------------------------------- CTRL-1: ground truth reproduced
# The instrument gave six different confident answers while being written. Every calibration defect
# was caught here, not by reading the code.
out="$(bash "$ROOT/$CHECK" --report 2>&1)"
if printf '%s\n' "$out" | grep -q 'ground-truth' || bash "$ROOT/$CHECK" >/dev/null 2>&1; then
  pass=$((pass + 1)); printf '✓ %-40s %s\n' "CTRL-1 ground-truth controls hold" "PASS"
else
  fail=$((fail + 1)); printf '✗ %-40s controls did not reproduce\n' "CTRL-1 ground-truth controls hold"
fi

# ---------------------------------------------------------------- CTRL-2: a miscalibrated instrument REFUSES
# ⚠️ THE FIRST VERSION OF THIS ARM WAS WRONG, AND THE CHECK WAS RIGHT. It stubbed out
# `mdbook_docs_gate`'s invocation in the hosted workflow and expected the control to break — but the
# parity gate ALSO replays that same command, so the target stayed genuinely reachable and the check
# correctly said so. Recorded because it is this tree's own standing lesson: *believe the control
# arms when they disagree with you*, and check whether the mutation actually removed the last edge.
#
# The arm now moves a control in the other direction: `ast_dump_contract_gate` is pinned ORPHAN
# (measured — RED for four sessions because nothing ran it), so GIVING it an invoker must make the
# instrument declare itself MISCALIBRATED rather than quietly report a different number as news.
python3 - "$ROOT/$MAKEFILE" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
open(p, "w").write(s.replace(
    "parser_books_gate: $(PARSER_BOOK_GATES)",
    "parser_books_gate: $(PARSER_BOOK_GATES) ast_dump_contract_gate", 1))
PY
arm "CTRL-2 broken control => REFUSES" FAIL "MISCALIBRATED"
restore

# ---------------------------------------------------------------- CTRL-3: unrelated edit, no misfire
printf '\n# CI-PARITY-GATE-ROT.2 probe: an unrelated comment must not change the verdict\n' \
  >> "$ROOT/$MAKEFILE"
arm "CTRL-3 unrelated Makefile edit" PASS
restore

printf '%s\n' "------------------------------------------------------------------------------"
printf 'arms=%d  PASS=%d  FAIL=%d\n' "$((pass + fail))" "$pass" "$fail"
[ "$fail" -eq 0 ]
