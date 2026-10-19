#!/usr/bin/env bash
# scripts/check_ci_mirror_docs_surfaces.sh — CI-MIRROR-DOCS-SURFACES: the push mirror's six WORKING-TREE
# docs-surface audits hold at the COMMIT that changes a doc, not first at the push.
#
# PROVENANCE — CI-PARITY-GATE-ROT.119(d), 2026-10-19 (PGEN-CI-PARITY-GATE-ROT-0124). The push mirror
# (rust/scripts/ci_workflow_local_gate.sh, ≈5.5 h per run) opens with 33 pre-flight audits. Six of them read
# ONLY the working tree — the root-markdown, top-level-docs, contract-docs and reference-docs allowlists
# (EXACT-SET comparisons against `git ls-files`), the docs/book surface (derived from SUMMARY.md) and the
# active-docs rehome-path audit (an `rg` over a fixed set of live docs) — and nothing but the mirror ran
# them. Measured that day: run 5 was refused 65 s in (a new contract doc, -0470, absent from the inline
# allowlist) and run 6 71 s in (a provenance comment naming a rehomed doc by its bare basename), each
# costing a full re-run of the gate the push waits on. Every pre-commit doctrine had passed both commits.
#
# ONE DEFINITION, NO FORK. This script does not re-implement the audits: it runs the mirror script in its
# AUDITS-ONLY mode (PGEN_CI_WORKFLOW_LOCAL_AUDITS_ONLY names the six functions; the mode refuses any other
# name, because the other 27 audits read the export this mode never produces). The functions that refuse
# here are the very functions the full gate runs, so the two cannot disagree. The mirror's own
# PGEN_CI_WORKFLOW_LOCAL_ROOT_DIR seam is what lets the RED arms below point the audits at a scratch tree.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit 0 holds / 1 breach / 2 refuses to evaluate; explains on
# stderr (the audit prints the expected and the actual surface, then its error line); deterministic; reads
# the repository and git only and mutates nothing tracked (the mirror's run dir goes under rust/target/ and
# is removed here); path-agnostic; no cargo, no network. Well under a second.
# ⚠️ A MISSING TOOL MUST NOT BE A PASS (CI-PARITY-GATE-ROT.20a: an audit whose `rg` was absent silently
# PASSED): git, perl and rg are required and their absence REFUSES (2).
#   bash scripts/check_ci_mirror_docs_surfaces.sh              # the doctrine
#   bash scripts/check_ci_mirror_docs_surfaces.sh --self-test  # prove each arm can fail (4 arms)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2
TAG="ci-mirror-docs-surfaces"
MIRROR="rust/scripts/ci_workflow_local_gate.sh"
AUDITS="audit_root_markdown_surface audit_top_level_docs_surface audit_contract_docs_surface audit_reference_docs_surface audit_docs_book_surface audit_active_docs_rehome_paths"
SCRATCH="$ROOT/rust/target/check_ci_mirror_docs_surfaces"   # DATA-LOCALITY: scratch stays on the repo volume

refuse() { printf '%s: REFUSED (2) - %s\n' "$TAG" "$*" >&2; exit 2; }
for tool in git perl rg; do
  command -v "$tool" >/dev/null 2>&1 || refuse "$tool not found on PATH; a check that cannot inspect its subject must SAY SO, not return green"
done
[ -r "$MIRROR" ] || refuse "$MIRROR is unreadable"
grep -q 'PGEN_CI_WORKFLOW_LOCAL_AUDITS_ONLY' "$MIRROR" || refuse "$MIRROR carries no audits-only mode (PGEN_CI_WORKFLOW_LOCAL_AUDITS_ONLY) - the seam this doctrine runs through is gone"
mkdir -p "$SCRATCH" || refuse "cannot create $SCRATCH"

# judge ROOT_DIR AUDIT... — run the named audits through the mirror script itself against ROOT_DIR.
# Returns the mirror's exit code; on a breach its stderr (the audit's expected/actual surfaces and error
# line) is passed through. JUDGE_STDERR holds that stderr for the self-test's assertions. The mirror's
# run dir lands under a scratch state dir that is removed here whatever the verdict.
JUDGE_STDERR=""
judge() {
  local root="$1"; shift
  local state rc
  state="$(mktemp -d "$SCRATCH/state.XXXXXX")" || return 2
  PGEN_CI_WORKFLOW_LOCAL_ROOT_DIR="$root" PGEN_CI_WORKFLOW_LOCAL_STATE_DIR="$state" \
    PGEN_CI_WORKFLOW_LOCAL_AUDITS_ONLY="$*" bash "$ROOT/$MIRROR" >"$state/stdout" 2>"$state/stderr"
  rc=$?
  JUDGE_STDERR="$(cat "$state/stderr")"
  [ "$rc" -eq 0 ] || cat "$state/stderr" >&2
  rm -rf "$state"
  return "$rc"
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  arm() { if [ "$2" -eq 0 ]; then echo "  PASS  $1"; else echo "  FAIL  $1"; fails=$((fails+1)); fi; }

  # (1) GREEN — the six audits hold on this tree.
  judge "$ROOT" $AUDITS >/dev/null 2>&1; [ $? -eq 0 ]
  arm "the six working-tree audits hold on this tree (0)" $?

  # (2) RED — a tracked stray under docs/contracts/ is refused. The stray is planted in a COPY of the
  #     index (GIT_INDEX_FILE), never in the real one: `git ls-files`, the allowlists' only input, reads
  #     the copy, so the arm is exact and the repository is untouched.
  idx="$SCRATCH/index.selftest.$$"
  cp "$(git rev-parse --git-path index)" "$idx" || refuse "cannot copy the index for the RED arm"
  GIT_INDEX_FILE="$idx" git update-index --add \
    --cacheinfo "100644,e69de29bb2d1d6434b8b29ae775ad8c2e48c5391,docs/contracts/ZZZ_STRAY_SELFTEST.md" \
    || refuse "cannot plant the stray in the index copy"
  GIT_INDEX_FILE="$idx" judge "$ROOT" $AUDITS >/dev/null 2>&1; rc=$?
  rm -f "$idx"
  [ "$rc" -eq 1 ] && printf '%s' "$JUDGE_STDERR" | grep -q 'contract docs allowlist drift'
  arm "a tracked stray contract doc is refused by the contract-docs allowlist (1)" $?

  # (3) RED — a live doc naming a rehomed doc by its bare basename is refused. A scratch tree holds the
  #     audit's 18 live docs (empty) and a README.md carrying the bare name; the basename is spelled in
  #     two halves so this script never carries it whole.
  tree="$(mktemp -d "$SCRATCH/tree.XXXXXX")" || refuse "cannot create the scratch tree"
  for f in README.md SESSION_BOOTSTRAP.md QUICKSTART_AI_ONBOARDING.md PGEN_USER_GUIDE.md COMMIT.md \
           rust/docs/TECHNICAL_ARCHITECTURE.md rust/docs/EMBEDDING_API_CONTRACT.md \
           rust/scripts/ci_workflow_local_gate.sh \
           docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md \
           docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md \
           docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md docs/reference/PGEN_RELEASE_POLICY.md \
           docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md \
           docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md docs/reference/RUST_CODEBASE_ANALYSIS.md; do
    mkdir -p "$tree/$(dirname "$f")"; : > "$tree/$f"
  done
  printf 'see PGEN_PARSER_INTEGRATION_%s for the index\n' "CONTRACTS.md" > "$tree/README.md"
  judge "$tree" audit_active_docs_rehome_paths >/dev/null 2>&1; rc=$?
  rm -rf "$tree"
  [ "$rc" -eq 1 ] && printf '%s' "$JUDGE_STDERR" | grep -q 'active docs path drift'
  arm "a live doc naming a rehomed doc by its bare basename is refused by the rehome-path audit (1)" $?

  # (4) REFUSAL — an audit name outside the six is refused by the mode itself, never run: the other 27
  #     audits read the export this mode does not produce, and a verdict about an absent tree is nothing.
  judge "$ROOT" audit_embedding_api_surface >/dev/null 2>&1; rc=$?
  [ "$rc" -ne 0 ] && printf '%s' "$JUDGE_STDERR" | grep -q 'not a working-tree-only audit'
  arm "an export-reading audit name is refused by the audits-only mode, not run (1)" $?

  echo "$TAG self-test: $((4 - fails))/4 arms (fail=$fails)"
  [ "$fails" -eq 0 ] || exit 1
  exit 0
fi

if judge "$ROOT" $AUDITS; then
  echo "$TAG: OK - the push mirror's 6 working-tree docs-surface audits hold (4 allowlists, docs/book surface, rehome paths)"
  exit 0
fi
printf '%s: FAIL - a docs-surface audit the push mirror runs in its first minute refuses THIS tree (its expected and actual surfaces are above). Fix the doc, or update the policy in %s deliberately in the same commit.\n' "$TAG" "$MIRROR" >&2
exit 1
