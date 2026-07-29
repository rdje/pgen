#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_routing_evidence_probes.sh
#
# CI-PARITY-GATE-ROT.12 — RED/GREEN/CONTROL arms for `scripts/check_routing_evidence.sh`,
# the 13th enforced doctrine.
#
# ⭐ RED-1 IS THE ARM THAT MATTERS: it stages the VERBATIM added lines of commit 59f810e1
# (`PGEN-CI-PARITY-GATE-ROT-0015`) — the routing that sent a shared-gate defect to the regex
# family and cost a session — and requires the check to BLOCK. The founding incident is
# REPLAYED from git, not paraphrased, so this cannot test a rule the incident would have
# escaped.
#
# ⚠️ THIS ARM ALREADY EARNED ITS KEEP: the FIRST cut of the check required the destination
# TREE ID on the routing line. `-0015` names a FAMILY ("BELONGS TO ANOTHER FAMILY", "Filed
# against the regex family"), never the tree file, so the first cut PASSED its own founding
# incident. Caught here, before shipping.
#
# ⛔ Runs entirely inside a scratch git repo — it stages fixtures, so it must never touch the
# real repository's index.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_routing_evidence_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECK="$ROOT/scripts/check_routing_evidence.sh"
[ -r "$CHECK" ] || { echo "FATAL: $CHECK not readable"; exit 1; }

W="$ROOT/rust/target/ci_parity_gate_rot_probe/p12"; rm -rf "$W"; mkdir -p "$W/repo/scripts" "$W/repo/docs/tasks"
R="$W/repo"
cp "$CHECK" "$R/scripts/check_routing_evidence.sh"

# The founding incident, extracted from git — never re-typed.
git -C "$ROOT" show 59f810e1 -- docs/tasks/CI-PARITY-GATE-ROT.md \
  | grep '^+' | grep -v '^+++' | sed 's/^+//' > "$W/incident.txt"
[ -s "$W/incident.txt" ] || { echo "FATAL: could not extract the -0015 added lines"; exit 1; }
grep -qi 'belongs to another family' "$W/incident.txt" \
  || { echo "FATAL: extracted text lacks the routing statement — wrong commit?"; exit 1; }

git -C "$R" init -q
git -C "$R" config user.email probe@local; git -C "$R" config user.name probe
: > "$R/docs/tasks/SEED.md"; git -C "$R" add -A; git -C "$R" commit -qm seed

reset_index(){ git -C "$R" reset -q; rm -f "$R"/docs/tasks/*.md; : > "$R/docs/tasks/SEED.md"; }
run_check(){ ( cd "$R" && bash scripts/check_routing_evidence.sh ) 2>&1; }

pass=0; fail=0
arm(){ local l="$1" want="$2" needle="$3"
  out="$(run_check)"; rc=$?
  [ $rc -eq 0 ] && got=PASS || got=BLOCK
  if [ "$got" != "$want" ]; then fail=$((fail+1)); printf '✗ %-46s expected %s got %s\n' "$l" "$want" "$got"; echo "$out"|sed 's/^/    /'; return; fi
  if [ -n "$needle" ] && ! echo "$out" | grep -qF -- "$needle"; then fail=$((fail+1)); printf '✗ %-46s %s but no "%s"\n' "$l" "$got" "$needle"; echo "$out"|sed 's/^/    /'; return; fi
  pass=$((pass+1)); printf '✓ %-46s %s\n' "$l" "$got"; }

echo "=== CI-PARITY-GATE-ROT.12 — ROUTING-EVIDENCE probes ==="

# RED-1 — the real -0015 routing, replayed verbatim, with no recorded evidence.
reset_index; cp "$W/incident.txt" "$R/docs/tasks/CI-PARITY-GATE-ROT.md"; git -C "$R" add -A
arm "RED-1 the REAL -0015 routing => BLOCK" BLOCK "no recorded routing evidence"

# GREEN-1 — same text, plus the section the check asks for.
reset_index; { cat "$W/incident.txt"; printf '\n## ROUTING EVIDENCE\n\nReproduced outside the family: yes, the ebnf row fails identically.\n'; } > "$R/docs/tasks/CI-PARITY-GATE-ROT.md"; git -C "$R" add -A
arm "GREEN-1 same text + ROUTING EVIDENCE => pass" PASS ""

# RED-2 — a DIFFERENT phrasing, to prove the rule is not pinned to one incident's words.
reset_index; printf -- '- The residual is not ours: filed against the vhdl family with the capture.\n' > "$R/docs/tasks/SOME-TREE.md"; git -C "$R" add -A
arm "RED-2 different phrasing => BLOCK" BLOCK "no recorded routing evidence"

# CTRL-1 — intra-tree routing, the measured common case (17 in the real corpus): must not fire.
reset_index; printf -- '- Three orphans routed to `.3`, `.4` and `.5`; the frontier moves to `.4`.\n' > "$R/docs/tasks/SOME-TREE.md"; git -C "$R" add -A
arm "CTRL-1 intra-tree routing => pass" PASS ""

# CTRL-2 — the RECEIVING side records what it was handed; not the decision this governs.
reset_index; printf -- '### `ROUTED-IN` — routed in from CI-PARITY-GATE-ROT.7 with evidence.\n' > "$R/docs/tasks/SOME-TREE.md"; git -C "$R" add -A
arm "CTRL-2 ROUTED-IN receiving side => pass" PASS ""

# CTRL-3 — a routing statement that is present but NOT staged (staged-scope-aware).
reset_index; git -C "$R" add -A; git -C "$R" commit -qm base >/dev/null 2>&1 || true
printf -- '- ROUTED OUT: it belongs to another family.\n' > "$R/docs/tasks/UNSTAGED.md"
arm "CTRL-3 unstaged routing => pass" PASS ""

# CTRL-4 — nothing staged at all: nothing to judge.
reset_index; rm -f "$R/docs/tasks/UNSTAGED.md"
arm "CTRL-4 empty staged set => pass" PASS ""

# CTRL-5 — a staged NON-task file carrying the language must not fire.
reset_index; printf -- '- ROUTED OUT: it belongs to another family.\n' > "$R/CHANGES.md"; git -C "$R" add -A
arm "CTRL-5 routing text outside docs/tasks => pass" PASS ""

echo "---"; printf 'arms=%d PASS=%d FAIL=%d\n' $((pass+fail)) $pass $fail; [ $fail -eq 0 ]
