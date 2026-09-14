#!/usr/bin/env bash
# scripts/check_lookahead_slot_templates.sh — doctrine LOOKAHEAD-SLOT-TEMPLATES (SV-CORPUS-GRAD.3.61).
#
# ⛔ WHAT THIS PROVES. No return template in a scoped grammar binds a LOOKAHEAD child: a lookahead is a
# positional child that binds `[]`, so `$p` on it, a `$N::K*` whose K-th child is the guard or the
# separator the guard shifted, and a guard-opened group bound raw are all shapes no consumer can read.
# ⛔ WHY (`SV-CORPUS-GRAD.3.61`, 2026-09-10): ten such sites shipped in the SV grammar across three
# releases (`1.0.79`, `1.0.191`, `1.0.210`) and stayed green under every gate — the accept set never
# moved and a consistently wrong tree is byte-identical to itself release after release.
# The audit first proves it can fail (`--self-test`: the three finding kinds + a clean control), then
# reads the scoped grammars; the scope is a LIST in the auditor, and the bootstrap-chain grammars are
# outside it by measurement (see the auditor's header).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if ! out="$(python3 "$ROOT/scripts/lookahead_slot_audit.py" --self-test 2>&1)"; then
  printf '%s\n' "$out"; echo "lookahead-slot-templates: FAIL — the auditor's self-test did not pass, so a green reading would be meaningless"; exit 1
fi
if ! out="$(python3 "$ROOT/scripts/lookahead_slot_audit.py" 2>&1)"; then
  printf '%s\n' "$out"; echo "lookahead-slot-templates: FAIL — a return template binds a lookahead child (fix the template's indices; see the findings above)"; exit 1
fi
echo "lookahead-slot-templates: OK — $(printf '%s\n' "$out" | tail -1)"
