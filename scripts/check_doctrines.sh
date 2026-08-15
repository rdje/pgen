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
#   E4 CI          : the same script runs server-side (un-bypassable backstop) — invoked by
#                    `.github/workflows/memory-architecture-gate.yml`, the one workflow that
#                    still runs on `push`/`pull_request`. ⭐ IT MUST INVOKE THIS DRIVER, NOT
#                    INDIVIDUAL ENFORCERS: it named five of them until 2026-07-29
#                    (CI-PARITY-GATE-ROT.15), so 8 of the 13 registered doctrines had NO
#                    automatic lane and a doctrine added tomorrow silently got none. The
#                    registry is the single source of the roster, so calling the driver makes
#                    a new doctrine inherit the lane by construction. `FLOW-INTEGRITY`
#                    invariant (8) enforces that; the other 14 workflows stay manual-only
#                    (workflow_dispatch) to conserve Actions minutes.
#                    HONEST GAP: a local hook can be `--no-verify`'d, and the four
#                    STAGED-SCOPE doctrines below evaluate nothing on a hosted push — which
#                    this driver now SAYS rather than passing silently.
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
  "MEMORY-ARCH|durable 4-layer memory architecture invariants (MEMORY_ARCHITECTURE.md §9), incl. layer A bounded by BOTH a line cap and a byte cap — the line-only form passed 60 lines / 138,403 bytes (README-POLICY.2)|scripts/check_memory_architecture.sh"
  "DIAG-SEVERITY+DOCPATH|severity never masked by verbosity + repo-root-relative live-doc paths|scripts/check_diagnostics_and_docpaths.sh"
  "EBNF-SOURCE-OF-TRUTH|no new out-of-band acceptance validator wired outside the EBNF|scripts/check_ebnf_source_of_truth.sh"
  "REGEX-SELF-HOSTING|the regex grammar self-hosts (gen<->parse duality holds)|scripts/check_regex_self_hosting.sh"
  "KNOWLEDGE-MAP|the derived Knowledge Map is in sync with its fact sources|knowledge-map/scripts/check_knowledge_map.sh"
  "TASK-ACCEPTANCE|a code change's task leaf passes the acceptance checklist (root-cause + addressed + no-regression boxes ticked + evidence-backed)|scripts/check_diagnosis_evidence.sh"
  "REGEX-ORACLE-ANCHOR-SYNC|the live oracle-tuple anchors agree with each other + the tracked ratchet bounds|scripts/check_regex_oracle_anchor_sync.sh"
  "LESSON-PROMOTION|a durable lesson written to DEVELOPMENT_NOTES.md is either PROMOTED into the retrievable layer or EXPLICITLY DECLINED, never silently dropped — the promotion mechanism existed, was wired, and was skipped 1592 times because no gate asked (LESSON-RETRIEVAL.4; KNOWLEDGE-MAP checks the map is in sync with its SOURCES, never that a lesson REACHED a source)|scripts/check_lesson_promotion.sh"
  "DESIGN-PRIOR-ART|a task leaf proposing a NEW annotation/directive surface records a prior-art search first (docs/decisions/feedback_read_prior_art_before_designing.md)|scripts/check_design_prior_art.sh"
  "WAIVER-ROUTING|a task leaf claiming a gate does not apply names the leaf that owns fixing the gate (an unrouted waiver is a bug report about the gate, left inert)|scripts/check_waiver_routing.sh"
  "ROUTING-EVIDENCE|a task leaf routing a finding OUT to another tree records what it MEASURED — above all whether the finding reproduces outside the family it is being sent to (CI-PARITY-GATE-ROT.9: a shared-gate defect was routed to the regex family on a plausible reading, and the deciding evidence was already on disk)|scripts/check_routing_evidence.sh"
  "DESTRUCTIVE-TARGET-GUARD|destructive make targets refuse without PGEN_CONFIRM_CLEAN=1 + no innocuous alias routes into them (OPS-MEMSAFE.3)|scripts/check_destructive_target_guard.sh"
  "GATE-REACHABILITY|every tracked gate target is invoked by something that RUNS, or carries a deliberate disposition — a check nothing invokes is indistinguishable from one that does not exist (CI-PARITY-GATE-ROT.2)|scripts/check_gate_reachability.sh"
  "FLOW-INTEGRITY|the gate flow cannot drift back: workflows that need generated parsers regenerate them and budget for it, the recipe keeps one home, artifact hand-offs never point at a standalone default, no assertion requires a defect to pass, hand-off provenance coverage only improves (CI-PARITY-GATE-ROT.8), the doctrine roster keeps an AUTOMATIC lane through THIS driver rather than a re-typed list of enforcers (CI-PARITY-GATE-ROT.15), a guard tests the artifact it actually READS (CI-PARITY-GATE-ROT.14), and the SHIPPING generation recipe stays QUIET — no tracked Makefile invokes --generate-parser carrying --debug/--trace, which cost 6.89 GB of log per regeneration for artifacts the flags provably cannot change (CI-PARITY-GATE-ROT.31, director-ruled)|scripts/check_flow_integrity.sh"
  "PUBLISHED-VERSION-CURRENCY|the user guide's published regex identity pair equals the contract's Contract Identity block and its published family status equals the live tracker row — a Provisional/any parser SHIPS on its published state, so the published state must be gate-held true (DONE-BAR.5a; measured ~77 releases stale with no gate reading it)|scripts/check_published_version_currency.sh"
  "LIVE-DOC-CURRENCY|a live document is currently TRUE, not merely bounded: instrument A counts DISTINCT dates per surface PAIRED WITH ITS DECLARED CHARTER (so it classifies rather than alarms — CHANGES.md scores 175 and is correct), instrument B catches a self-declared \`Last updated:\` refuted by the file's own newest content date (no baseline, no threshold), and route closure requires every .md destination a capped enforcer NAMES to be a watched surface — the unwatched-overflow edge is how LIVE_ACHIEVEMENT_STATUS.md reached 1 547 057 B beside a README capped on two axes (LIVE-MEANS-LIVE.2). ⛔ B REFUSES on any unclassified declaration spelling rather than enumerating spellings: enumeration is what made the same population measure 10, then 16, then 18, every miss silent in the passing direction|scripts/check_live_document_currency.sh"
  "SV-CORPUS-DENOMINATOR|the SV corpus DENOMINATOR is re-derived and published beside the defect bar: the tracked verdict-coverage artifacts are byte-identical to a fresh census re-run, and every designated live surface carries the derived adjudicated/routed/no-verdict/dark/bar tuple — a bar without its denominator is not a claim about the corpus (SV-CORPUS-GRAD.13b; the artifact was MEASURED stale one day after landing, published bar 319 vs HEAD's 318, with NOTHING in the repo invoking the instrument)|scripts/check_sv_corpus_denominator.sh"
  "PARSE-COST-RATCHET|the SV parser's parse COST is measured and cannot rise unwatched: the tracked baseline's instrument identity (grammar + generated parser + a digest of the sampled corpus files) is re-hashed every run, and a re-measure refuses on any rise in the binding counters — entries / committed / memo-hits, each verified deterministic and build-mode-independent before being made binding (ENGINE-UNIVERSAL-SERVICES.20 acceptance (d); the guarded admission cost +24.3 % parse time while the lint, the two-sided repro ratchet, the corpus pass/fail count and every registered doctrine stayed GREEN, because nothing measured parse cost at all)|scripts/check_parse_cost_ratchet.sh"
  "SCRATCH-SLOT-HEADER|the blessed throwaway scratch slot keeps its OPERATING MANUAL — the header block that is the only record of how to regenerate, drive and restore the slot, and of the rebuild-ast_pipeline-AFTER ordering trap — because the body is meant to be overwritten and the header is not (PARSE-HARNESS.11; measured 2026-08-15: an agent replaced the whole file, and the correct workflow ends in \`git checkout\`, so the loss can NEVER reach a commit for a commit-time gate to see — the probe-time tier in \`make focus_scratch\` is the one that catches it, and this tier catches a removal that IS committed)|scripts/check_scratch_slot_header.sh"
  "README-STABILITY|README.md stays a stable LANDING PAGE — both a line cap and a byte cap, because a line cap alone is measurably bypassable — at adoption layer-A MEMORY.md passed its 60-line cap carrying 138,403 unbounded bytes, a bypass since CLOSED by giving MEMORY-ARCH the same two caps (README-POLICY.2) — plus a changelog-leakage tripwire and a link back to the reviewed policy (README-POLICY.1; adopted at 510 lines/48,811 bytes with NO instrument watching size: the two guards that touch README.md audit doc PATHS and the root file SET)|scripts/check_readme_stability.sh"
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

# ------------------------------------------------------------------ meta-check: the human mirror
# The registry above is the SOURCE OF TRUTH and the header has always CLAIMED that
# DOCTRINE_ENFORCEMENT.md §10 is "kept in lockstep" with it. MEASURED 2026-07-29
# (CI-PARITY-GATE-ROT.15): it was not — 3 registered doctrines had no row (`TASK-ACCEPTANCE`,
# `ROUTING-EVIDENCE`, `DESTRUCTIVE-TARGET-GUARD`) and 1 row named `DIAG-TOOLBOX-EVIDENCE`, an id the
# registry no longer carries. A documented promise nobody checks is the exact rot this driver exists
# to end, one level up. ⛔ The assertion lives HERE and is about a DIFFERENT file, deliberately: an
# assertion about a file cannot live inside that file as a literal (docs/decisions/
# reference_self_referential_assertion_is_unsound.md).
# ⚠️ Honest limit, stated not discovered: this proves the ID SETS agree, not that each row's prose is
# accurate. Row text is reviewed, not gated.
MIRROR="DOCTRINE_ENFORCEMENT.md"
mirror_ids="$(sed -n '/^## 10\. The live PGEN instance/,/^## 11\./p' "$ROOT/$MIRROR" 2>/dev/null \
              | sed -n 's/^| `\([A-Z][A-Z0-9+-]*\)` |.*/\1/p' | LC_ALL=C sort -u)"
registry_ids="$(printf '%s\n' "${DOCTRINES[@]}" | cut -d'|' -f1 | LC_ALL=C sort -u)"
if [ -z "$mirror_ids" ]; then
  report+=("✗ FAIL  <meta:mirror> — ${MIRROR} §10 yielded ZERO doctrine rows. A check that cannot"
           "        inspect its subject must SAY SO, not pass: either the section was renamed or its"
           "        table shape changed. Fix the extraction or the section, do not delete the check.")
  fail=1
else
  only_registry="$(LC_ALL=C comm -23 <(printf '%s\n' "$registry_ids") <(printf '%s\n' "$mirror_ids"))"
  only_mirror="$(LC_ALL=C comm -13 <(printf '%s\n' "$registry_ids") <(printf '%s\n' "$mirror_ids"))"
  if [ -n "$only_registry" ] || [ -n "$only_mirror" ]; then
    [ -n "$only_registry" ] && printf 'doctrine registered but ABSENT from %s §10: %s\n' \
      "$MIRROR" "$(printf '%s' "$only_registry" | tr '\n' ' ')" >&2
    [ -n "$only_mirror" ] && printf 'row in %s §10 naming a doctrine NOT registered: %s\n' \
      "$MIRROR" "$(printf '%s' "$only_mirror" | tr '\n' ' ')" >&2
    report+=("✗ FAIL  <meta:mirror> — the registry and its ${MIRROR} §10 mirror disagree (see above)")
    fail=1
  else
    report+=("✓ PASS  <meta:mirror> — ${MIRROR} §10 lists exactly the ${#DOCTRINES[@]} registered doctrines")
  fi
fi

printf '\n================ DOCTRINE ENFORCEMENT REPORT ================\n' >&2
for line in "${report[@]}"; do printf '  %s\n' "$line" >&2; done
printf '============================================================\n' >&2

# ------------------------------------------------------------------ scope note: what did NOT run
# ⭐ THIS TREE'S FOUNDING PRINCIPLE, APPLIED TO THE DRIVER ITSELF: *a check that cannot run must SAY
# SO, not return green.* Some registered enforcers judge the STAGED diff. Under the pre-commit hook
# that is the change being made; on a hosted `push` (E4) the index is EMPTY, so they exit 0 having
# evaluated nothing — and a green tick would otherwise imply they were satisfied.
# ⛔ DERIVED from each enforcer's own source, not hand-listed, so a staged-scope doctrine added later
# is named here by construction — the same reason E4 must call this driver rather than enumerate it.
# ⚠️ Honest limit: the classifier is a source-level heuristic (does the enforcer read the index?). A
# mislabel costs an inaccurate NAME in an informational note; it never changes a verdict.
staged_count="$(git -C "$ROOT" diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')"
if [ "${staged_count:-0}" -eq 0 ]; then
  vacuous=()
  for entry in "${DOCTRINES[@]}"; do
    IFS='|' read -r id _proves script <<< "$entry"
    if grep -qE -- '--cached|--staged|diff-index' "$ROOT/$script" 2>/dev/null; then
      vacuous+=("$id")
    fi
  done
  if [ "${#vacuous[@]}" -gt 0 ]; then
    printf 'scope: no staged changes — %d of %d doctrines are scoped to the STAGED diff and therefore\n' \
      "${#vacuous[@]}" "${#DOCTRINES[@]}" >&2
    printf '       evaluated NOTHING here: %s\n' "$(printf '%s ' "${vacuous[@]}")" >&2
    printf '       Their PASS above is vacuous, not evidence. They bind at commit time (E3) and on a\n' >&2
    printf '       hosted push only for whatever that push actually stages.\n' >&2
  fi
fi
if [ "$fail" -eq 0 ]; then
  printf 'doctrines: ALL %d enforced doctrines PASS.\n' "${#DOCTRINES[@]}" >&2
else
  printf 'doctrines: ✗ one or more doctrines FAILED — commit/merge blocked. Fix above, do not bypass.\n' >&2
fi
exit "$fail"
