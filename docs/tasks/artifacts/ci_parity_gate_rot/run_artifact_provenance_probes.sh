#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_artifact_provenance_probes.sh
#
# CI-PARITY-GATE-ROT.7 — RED/GREEN/CONTROL arms for `require_supplied_state_dir`, the guard that
# makes an EXISTING-artifact hand-off prove its provenance instead of being trusted.
#
# ⭐ RED-2 IS THE ARM THAT MATTERS. It hands the guard an artifact stamped 2026-07-26 00:36 — the
# real mtime of the leftover a `sota_exit_gate` run consumed as current proof — and requires refusal.
# That silent path is the dangerous one: the three ABSENT sibling artifacts made the run die loudly,
# but a machine with all four present would have gone fully GREEN on evidence of unknown vintage.
#
# ⚠️ THESE ARMS CAUGHT A REAL PORTABILITY DEFECT IN THE GUARD'S FIRST CUT. BSD stat spells mtime
# `-f %m`; GNU coreutils spells it `-c %Y` and reads `-f` as "file SYSTEM information", which
# SUCCEEDS at printing six lines of block counts. This host has GNU stat, so a BSD-first chain never
# fell through — it captured that block as the "timestamp". RED-2 and GREEN-1 both failed with
# `File: unbound variable`. The fix validates the result is a bare integer rather than betting on a
# variant.
#
# ⛔ The helper is EXTRACTED FROM THE LIVE GATE, never re-typed, so this cannot test a rule the gate
# does not apply.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_artifact_provenance_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"
W="rust/target/ci_parity_gate_rot_probe/p7"; rm -rf "$W"; mkdir -p "$W/stale/"; mkdir -p "$W/fresh"
# extract the live helper (never re-typed)
G=rust/scripts/sv_parser_family_status_gate.sh
{ echo '#!/usr/bin/env bash'; echo 'set -uo pipefail'
  sed -n '/^require_nonempty_file() {/,/^}/p' "$G"
  sed -n '/^require_supplied_state_dir() {/,/^}/p' "$G"; } > "$W/helper.sh"
grep -q '^require_supplied_state_dir() {' "$W/helper.sh" || { echo "FATAL: could not extract helper"; exit 1; }
printf 'x\n' > "$W/stale/summary.txt"; touch -t 202607260036 "$W/stale/summary.txt"
printf 'x\n' > "$W/fresh/summary.txt"
NOW=$(date +%s)
pass=0; fail=0
arm(){ local l="$1" want="$2" needle="$3"; shift 3
  out=$( ( set +e; . "$W/helper.sh"; "$@" ) 2>&1 ); rc=$?
  [ $rc -eq 0 ] && got=PASS || got=FAIL
  if [ "$got" != "$want" ]; then fail=$((fail+1)); printf '✗ %-38s expected %s got %s\n' "$l" "$want" "$got"; echo "$out"|sed 's/^/    /'; return; fi
  if [ -n "$needle" ] && ! echo "$out" | grep -qF -- "$needle"; then fail=$((fail+1)); printf '✗ %-38s %s but no "%s"\n' "$l" "$got" "$needle"; echo "$out"|sed 's/^/    /'; return; fi
  pass=$((pass+1)); printf '✓ %-38s %s\n' "$l" "$got"; }
echo "=== CI-PARITY-GATE-ROT.7 — artifact provenance probes ==="
PGEN_GATE_ARTIFACT_MIN_EPOCH="$NOW" arm "RED-1 absent dir refused" FAIL "does not exist" require_supplied_state_dir lbl "$W/nope" VAR
PGEN_GATE_ARTIFACT_MIN_EPOCH="$NOW" arm "RED-2 STALE artifact refused" FAIL "STALE artifact refused" require_supplied_state_dir lbl "$W/stale" VAR
PGEN_GATE_ARTIFACT_MIN_EPOCH="$NOW" arm "GREEN-1 in-run artifact accepted" PASS "" require_supplied_state_dir lbl "$W/fresh" VAR
arm "CTRL-1 no epoch => stale accepted" PASS "" require_supplied_state_dir lbl "$W/stale" VAR
rm -f "$W/fresh/summary.txt"; touch "$W/fresh/summary.txt"
PGEN_GATE_ARTIFACT_MIN_EPOCH="$NOW" arm "RED-3 empty summary refused" FAIL "non-empty" require_supplied_state_dir lbl "$W/fresh" VAR
echo "---"; printf 'arms=%d PASS=%d FAIL=%d\n' $((pass+fail)) $pass $fail; [ $fail -eq 0 ]
