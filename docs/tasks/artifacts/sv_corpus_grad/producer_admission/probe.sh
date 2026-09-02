#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/producer_admission/probe.sh
#
# SV-CORPUS-GRAD.13l — the RED/GREEN bank for TASK-ACCEPTANCE's DERIVED producer admission.
# Every arm stages a mutation and REVERTS it inside one trap-guarded step (a deliberate mutation
# must never outlive the tool call that applied it — ENGINE-UNIVERSAL-SERVICES.87). The index is
# expected EMPTY on entry and is left empty on exit; the bank refuses to run on a dirty index.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
GATE=scripts/check_diagnosis_evidence.sh
[ -z "$(git diff --cached --name-only)" ] || { echo "REFUSE: the index is not empty; this bank stages and unstages probes"; exit 2; }
PRODUCER=stimuli/run_external_corpus.sh          # hop 1: named by check_corpus_census.sh
HOP2=stimuli/sv/adjudicate_external_corpus.py    # hop 2: named only by the runner
CONTROL=stimuli/sv/audit_explained_svpp.py       # no oracle names it
for f in "$PRODUCER" "$HOP2" "$CONTROL"; do git diff --quiet -- "$f" || { echo "REFUSE: $f has uncommitted changes"; exit 2; }; done
restore() { git reset -q -- "$PRODUCER" "$HOP2" "$CONTROL" 2>/dev/null; git checkout -q -- "$PRODUCER" "$HOP2" "$CONTROL" 2>/dev/null; }
trap restore EXIT INT TERM HUP
pass=0; fail=0
say(){ if [ "$1" = ok ]; then pass=$((pass+1)); printf 'PASS  %s\n' "$2"; else fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$2" "$3"; fi; }
probe(){ # probe <file> -> stages a comment-only edit, runs the gate, restores; echoes "rc|first line"
  printf '\n# probe: comment-only edit (producer_admission bank)\n' >> "$1"; git add -- "$1"
  local out rc; out="$(bash "$GATE" 2>&1)"; rc=$?; restore
  printf '%s|%s' "$rc" "$(printf '%s' "$out" | head -1)"
}
# ---- 0: the derived set is derived — hop 1 AND hop 2 present, control absent --------------------
set_out="$(bash "$GATE" --producers 2>&1)"
if printf '%s' "$set_out" | grep -qF "$PRODUCER" && printf '%s' "$set_out" | grep -qF "$HOP2" && ! printf '%s' "$set_out" | grep -qF "$CONTROL"; then
  say ok "0  --producers derives hop 1 ($PRODUCER) AND hop 2 ($HOP2), and excludes an un-referenced helper"
else say no "0  --producers derivation" "$set_out"; fi
n=$(printf '%s' "$set_out" | grep -c '^  stimuli/'); echo "      derived set size: $n"
# ---- 1: RED — a producer edit with no checklist is a CODE change and the gate refuses ------------
r="$(probe "$PRODUCER")"; rc="${r%%|*}"
if [ "$rc" -ne 0 ]; then say ok "1  RED: comment-only edit to $PRODUCER without a checklist -> gate FAILS (rc=$rc)"; else say no "1  RED producer edit" "$r"; fi
# ---- 2: RED — the hop-2 producer too -----------------------------------------------------------
r="$(probe "$HOP2")"; rc="${r%%|*}"
if [ "$rc" -ne 0 ]; then say ok "2  RED: comment-only edit to $HOP2 (hop 2) -> gate FAILS (rc=$rc)"; else say no "2  RED hop-2 edit" "$r"; fi
# ---- 3: GREEN — an un-referenced stimuli helper stays out of scope ------------------------------
r="$(probe "$CONTROL")"; rc="${r%%|*}"; line="${r#*|}"
if [ "$rc" -eq 0 ] && printf '%s' "$line" | grep -q 'no code change staged'; then say ok "3  GREEN: comment-only edit to $CONTROL -> 'no code change staged' (rc=0)"; else say no "3  non-producer control" "$r"; fi
# ---- 4: GREEN — the empty index -----------------------------------------------------------------
out="$(bash "$GATE" 2>&1)"; rc=$?
if [ "$rc" -eq 0 ]; then say ok "4  GREEN: empty index -> rc=0"; else say no "4  empty index" "$out"; fi
[ -z "$(git diff --cached --name-only)" ] && git diff --quiet -- "$PRODUCER" "$HOP2" "$CONTROL" && say ok "5  the tree and index are restored byte-for-byte" || say no "5  restore" "$(git status --short)"
printf 'PRODUCER-ADMISSION: %d/%d as declared\n' "$pass" "$((pass+fail))"
[ "$fail" -eq 0 ]
