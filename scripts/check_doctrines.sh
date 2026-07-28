#!/usr/bin/env bash
# scripts/check_doctrines.sh — THE GENERAL DOCTRINE ENFORCER (driver + registry).
#
# PGEN-DIAG-TOOLBOX-0002. Director directive (2026-06-22): make doctrine compliance
# "provable, trackable ... so we do not have to trust your words — rule enforcers that
# literally force the rule, no matter what." This is the single driver that runs EVERY
# mechanizable doctrine check, reports per-doctrine PASS/FAIL, and exits NONZERO on any
# breach. It formalizes the previously ad-hoc `.githooks/pre-commit` check stack into one
# registered, self-reporting framework.
#
# Enforcement layering (MEMORY_ARCHITECTURE.md §9 — defense in depth):
#   E1 discovery   : the doctrine docs (README, TOOLBOX.md, docs/decisions/, the books).
#   E2 self-check  : THIS script + each registered scripts/check_*.sh (single source of truth).
#   E3 git hook    : .githooks/pre-commit calls this (fast local gate; activated via
#                    `git config core.hooksPath .githooks`).
#   E4 CI          : the same script runs server-side (un-bypassable backstop). HONEST GAP:
#                    hosted CI is currently manual-only (workflow_dispatch) to conserve
#                    Actions minutes — so the un-bypassable leg is only as strong as the
#                    next manual/CI run. Re-enabling an auto CI doctrine-gate is the true
#                    "no matter what" backstop (a local hook can be `--no-verify`'d).
#
# What "provable / not trust-me-bro" means here, honestly:
#   - STRUCTURAL doctrines (memory-arch, docpaths, ebnf-SoT, knowledge-map) — the check
#     re-derives the invariant from the tree; pass/fail is a fact about the files.
#   - DETERMINISTIC-ORACLE doctrines (cert-coverage, ast_shape_contract, syntax-closure —
#     run by the broader make gates / CI) re-RUN the real tool at fixed seeds, so a claimed
#     number that does not reproduce FAILS. This is the strongest leg.
#   - EVIDENCE doctrines (diag-toolbox-evidence) require a re-checkable artifact (pasted tool
#     output) in the owning task leaf. A hook cannot prove the agent REASONED from it, but it
#     makes landing a code change with no pasted, reproducible tool output impossible at the gate.
#
# Registry below = the source of truth for "which doctrines are enforced by what". The
# human-readable mirror is DOCTRINE_ENFORCEMENT.md §10 (kept in lockstep).
set -uo pipefail   # deliberately NOT `-e`: run ALL checks, collect every result, then report.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

# Each entry: "ID|what it proves|relative/path/to/check.sh"
# Add a new doctrine here AND in docs/decisions/DOCTRINE_ENFORCEMENT.md (a meta-check below
# asserts every registered enforcer exists + is executable, so a registry entry cannot be a
# dangling promise).
DOCTRINES=(
  "MEMORY-ARCH|durable 4-layer memory architecture invariants (MEMORY_ARCHITECTURE.md §9)|scripts/check_memory_architecture.sh"
  "DIAG-SEVERITY+DOCPATH|severity never masked by verbosity + repo-root-relative live-doc paths|scripts/check_diagnostics_and_docpaths.sh"
  "EBNF-SOURCE-OF-TRUTH|no new out-of-band acceptance validator wired outside the EBNF|scripts/check_ebnf_source_of_truth.sh"
  "REGEX-SELF-HOSTING|the regex grammar self-hosts (gen<->parse duality holds)|scripts/check_regex_self_hosting.sh"
  "KNOWLEDGE-MAP|the derived Knowledge Map is in sync with its fact sources|knowledge-map/scripts/check_knowledge_map.sh"
  "TASK-ACCEPTANCE|a code change's task leaf passes the acceptance checklist (root-cause + addressed + no-regression boxes ticked + evidence-backed)|scripts/check_diagnosis_evidence.sh"
  "REGEX-ORACLE-ANCHOR-SYNC|the live oracle-tuple anchors agree with each other + the tracked ratchet bounds|scripts/check_regex_oracle_anchor_sync.sh"
  "DESIGN-PRIOR-ART|a task leaf proposing a NEW annotation/directive surface records a prior-art search first (docs/decisions/feedback_read_prior_art_before_designing.md)|scripts/check_design_prior_art.sh"
  "WAIVER-ROUTING|a task leaf claiming a gate does not apply names the leaf that owns fixing the gate (an unrouted waiver is a bug report about the gate, left inert)|scripts/check_waiver_routing.sh"
  "DESTRUCTIVE-TARGET-GUARD|destructive make targets refuse without PGEN_CONFIRM_CLEAN=1 + no innocuous alias routes into them (OPS-MEMSAFE.3)|scripts/check_destructive_target_guard.sh"
  "GATE-REACHABILITY|every tracked gate target is invoked by something that RUNS, or carries a deliberate disposition — a check nothing invokes is indistinguishable from one that does not exist (CI-PARITY-GATE-ROT.2)|scripts/check_gate_reachability.sh"
  "FLOW-INTEGRITY|the gate flow cannot drift back: workflows that need generated parsers regenerate them and budget for it, the recipe keeps one home, artifact hand-offs never point at a standalone default, no assertion requires a defect to pass, and hand-off provenance coverage only improves (CI-PARITY-GATE-ROT.8)|scripts/check_flow_integrity.sh"
)

fail=0
declare -a report=()

for entry in "${DOCTRINES[@]}"; do
  IFS='|' read -r id proves script <<< "$entry"
  if [ ! -x "$ROOT/$script" ]; then
    report+=("✗ FAIL  ${id} — registered enforcer missing or not executable: ${script}")
    fail=1
    continue
  fi
  if out="$("$ROOT/$script" 2>&1)"; then
    report+=("✓ PASS  ${id} — ${proves}")
  else
    report+=("✗ FAIL  ${id} — ${proves}")
    printf '%s\n' "$out" >&2
    fail=1
  fi
done

printf '\n================ DOCTRINE ENFORCEMENT REPORT ================\n' >&2
for line in "${report[@]}"; do printf '  %s\n' "$line" >&2; done
printf '============================================================\n' >&2
if [ "$fail" -eq 0 ]; then
  printf 'doctrines: ALL %d enforced doctrines PASS.\n' "${#DOCTRINES[@]}" >&2
else
  printf 'doctrines: ✗ one or more doctrines FAILED — commit/merge blocked. Fix above, do not bypass.\n' >&2
fi
exit "$fail"
