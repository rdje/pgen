#!/bin/bash
set -euo pipefail

fail() {
  echo "error: $*" >&2
  exit 1
}

normalize_bool() {
  local raw="${1:-}"
  local name="${2:-boolean flag}"
  local lowered
  lowered="$(printf '%s' "$raw" | tr '[:upper:]' '[:lower:]')"
  case "$lowered" in
    1|true|yes|on)
      printf 'true'
      ;;
    0|false|no|off|'')
      printf 'false'
      ;;
    *)
      fail "invalid boolean value for $name: '$raw'"
      ;;
  esac
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
STATE_DIR="${PGEN_CI_WORKFLOW_LOCAL_STATE_DIR:-$RUST_DIR/target/ci_workflow_local_gate}"
mkdir -p "$STATE_DIR"
RUN_DIR="$(mktemp -d "$STATE_DIR/run.XXXXXX")"
EXPORT_DIR="$RUN_DIR/export"
LOG_DIR="$RUN_DIR/logs"
SUMMARY_FILE="$RUN_DIR/summary.txt"
FILTER_RAW="${PGEN_CI_WORKFLOW_LOCAL_FILTER:-}"
CARGO_OFFLINE_RAW="${PGEN_CI_WORKFLOW_LOCAL_CARGO_OFFLINE:-true}"
KEEP_RUNS_RAW="${PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS:-false}"
KEEP_RUNS_NORMALIZED="$(normalize_bool "$KEEP_RUNS_RAW" "PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS")"
# ⭐ CI-PARITY-GATE-ROT (director ruling 2026-07-28): DEFAULTS TO `true` SINCE `.4` LANDED.
#
# `.3` shipped this knob defaulting to `false` on purpose and said why: defaulting it to `true`
# then would have made the LOCAL gate green while the HOSTED side stayed broken — 14 of the 15
# tracked workflows had no regeneration step — and **false parity is worse than the visible red the
# gate reported.** `.4` removed that condition: 11 of 15 workflows need generated parsers and all 11
# now regenerate through `.github/actions/regenerate-parsers`, so a local green and a hosted green
# now mean the same thing. Flipping the default is the last step of the director's ordered scope.
#
# Cost when it engages: ~236 s, and ONLY when the export dir is missing an artifact
# `rust/src/lib.rs` includes by literal path — a run that needs nothing pays nothing
# (`preflight_generated_artifacts` returns early). Set `PGEN_CI_WORKFLOW_LOCAL_PREPARE=0` to opt out
# deliberately; the gate then warns loudly instead of preparing, and any replay that compiles the
# crate fails with the cause attached.
PREPARE_RAW="${PGEN_CI_WORKFLOW_LOCAL_PREPARE:-true}"
PREPARE_NORMALIZED="$(normalize_bool "$PREPARE_RAW" "PGEN_CI_WORKFLOW_LOCAL_PREPARE")"

# CI-PARITY-GATE-ROT.3 — the roster of replayable workflow names, and how many actually RAN.
# Both are DERIVED by `run_workflow` from its own call sites; nothing here is a hand-kept list that
# could drift away from `main()`. They exist so the gate can answer two questions it previously
# could not: "was that filter name real?" and "did this run replay anything at all?".
WORKFLOW_ROSTER=""
WORKFLOWS_RUN_COUNT=0

mkdir -p "$EXPORT_DIR" "$LOG_DIR"

note() {
  echo "$*" | tee -a "$SUMMARY_FILE"
}

cleanup_run_dir_on_exit() {
  local exit_code=$?

  if [[ ! -d "$RUN_DIR" ]]; then
    return "$exit_code"
  fi

  if [[ "$exit_code" -eq 0 ]]; then
    if [[ "$KEEP_RUNS_NORMALIZED" == "true" ]]; then
      echo "retaining successful run_dir: $RUN_DIR (PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS=$KEEP_RUNS_RAW)"
    else
      echo "removing successful run_dir: $RUN_DIR"
      rm -rf "$RUN_DIR" || echo "warning: failed to remove successful run_dir: $RUN_DIR" >&2
    fi
  else
    echo "retaining failed run_dir: $RUN_DIR" >&2
  fi

  return "$exit_code"
}

trap cleanup_run_dir_on_exit EXIT

require_tool() {
  local tool="$1"
  command -v "$tool" >/dev/null 2>&1 || fail "required tool not found on PATH: $tool"
}

is_selected() {
  local workflow_name="$1"
  local item
  if [[ -z "$FILTER_RAW" ]]; then
    return 0
  fi
  IFS=',' read -r -a items <<<"$FILTER_RAW"
  for item in "${items[@]}"; do
    if [[ "$workflow_name" == "$item" ]]; then
      return 0
    fi
  done
  return 1
}

assert_tracked() {
  local repo_rel="$1"
  (cd "$ROOT_DIR" && git ls-files --error-unmatch -- "$repo_rel" >/dev/null 2>&1) || \
    fail "required tracked file missing from git index: $repo_rel"
}

# GENERATED-LINT-CORRECTNESS.3 — `generated/` is pipeline output and is NOT tracked
# (`.gitignore:24`, README "Key Project Paths", COMMIT.md). Asserting it is IN THE GIT INDEX is
# therefore always false, and it had been doing so at 7 call sites since `0ed2b2ad`
# ("Slice 5: stop tracking generated/* in git", 2026-04-29) — 1,371 commits during which this
# gate could not get past its FIRST audit. The audit's real intent is that the artifact is
# AVAILABLE to the include!() sites, which is presence on disk, exactly the condition
# `rust/build.rs` tests with `is_file()`. Refuse loudly and name the regeneration command rather
# than asserting a condition the repository policy guarantees can never hold.
assert_generated_artifact() {
  local repo_rel="$1"
  [ -s "$ROOT_DIR/$repo_rel" ] || fail "required generated artifact missing or empty: $repo_rel
  generated/ is untracked pipeline output — regenerate it before running this gate, e.g.
    make -C rust SHELL=/bin/bash regex_parser_bootstrap   # seeds generated/ebnf.rs on a cold clone
    make -C rust SHELL=/bin/bash annotation_parsers       # the two annotation parsers
    make -C rust SHELL=/bin/bash focus_<grammar>          # one shipped parser
  (This is a REFUSAL, not a pass: the gate cannot audit include! wiring it cannot see.)"
}

assert_workflow_contains() {
  local workflow_file="$1"
  local expected="$2"
  grep -F -- "$expected" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1 || \
    fail "workflow content drift detected in $workflow_file: expected '$expected'"
}

assert_workflow_not_contains() {
  local workflow_file="$1"
  local forbidden="$2"
  if grep -F -- "$forbidden" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1; then
    fail "unexpected workflow content in $workflow_file: found '$forbidden'"
  fi
}

assert_file_contains() {
  local repo_file="$1"
  local expected="$2"
  grep -F -- "$expected" "$ROOT_DIR/$repo_file" >/dev/null 2>&1 || \
    fail "file content drift detected in $repo_file: expected '$expected'"
}

# Regex form of assert_file_contains.
# CI-PARITY-GATE-ROT.1 (2026-07-27): added for surfaces whose content is DESIGNED to move —
# release/contract version constants in particular. Pinning such a value as a literal guarantees
# the audit fails on every legitimate bump, which is how three of these rotted silently
# (EMBEDDING_API_VERSION pinned 1.2.0, live 1.3.1; the two regex constants pinned 1.1.31/1.1.29,
# live 1.1.109/1.1.106). Asserting SHAPE keeps the real invariant — the public surface still
# declares these constants, well-formed — without re-pinning a moving target.
assert_file_matches() {
  local repo_file="$1"
  local pattern="$2"
  grep -Eq -- "$pattern" "$ROOT_DIR/$repo_file" 2>/dev/null || \
    fail "file content drift detected in $repo_file: nothing matches /$pattern/"
}

# assert_file_contains_count <repo_file> <literal> <expected_count>
# Presence is not enough when a surface has N instances that must EACH hold.
# QUANT-PLUS-ITER.4 (2026-07-27): added because this leaf's own RED arm caught its own weak
# assertion — `ast_dump_contract_gate.sh` carries TWO raw-AST fixtures (`mini`, `mini_large`) and a
# plain `assert_file_contains` for the mandatory `@entry` declaration still PASSED after one of the
# two was deleted. An audit that proves "at least one of N is correct" is a vacuity trap wearing a
# green tick — the same class as `.3`'s "lint a directory that compiled zero parsers and exit 0".
assert_file_contains_count() {
  local repo_file="$1"
  local expected="$2"
  local want="$3"
  local got
  got="$(grep -F -c -- "$expected" "$ROOT_DIR/$repo_file" 2>/dev/null || true)"
  [ -n "$got" ] || got=0
  if [ "$got" != "$want" ]; then
    fail "file content drift in $repo_file: expected exactly $want occurrence(s) of '$expected', found $got"
  fi
}

assert_file_not_contains() {
  local repo_file="$1"
  local forbidden="$2"
  if grep -F -- "$forbidden" "$ROOT_DIR/$repo_file" >/dev/null 2>&1; then
    fail "unexpected file content in $repo_file: found '$forbidden'"
  fi
}

# assert_markdown_section_not_contains <repo_file> <section-heading-regex> <forbidden>
# Forbid a string inside ONE `##` section of a markdown file (heading line through to the next
# `## `, or EOF).
#
# CI-PARITY-GATE-ROT.1b (2026-07-27), on the DIRECTOR RULING *"I agree we shouldn't cite either
# regex.json or regex.ebnf"*. The boundary being protected is real and STANDS: a downstream
# contract must not INSTRUCT a consumer to reach into PGEN-internal build inputs — consumers
# integrate through `pgen::embedding_api` and the supported `make` target.
#
# ⛔ WHY THIS IS NOT A RELAXATION. The previous whole-file assertion could only be satisfied by
# deleting ~35 occurrences that are HISTORICAL PROVENANCE — "Maintenance Update … the bound is now
# encoded structurally inside `grammars/regex.ebnf`" — i.e. the record of which grammar rule
# changed in a released slice. Erasing those to satisfy a check would back-date the maintenance
# history, which this project refuses. The invariant is about INSTRUCTIONS, not about whether
# history may name a file, so the assertion is scoped to the section that carries instructions.
# A citation re-added to the consumer-facing recipe still FAILS — verified by probe.
assert_markdown_section_not_contains() {
  local repo_file="$1"
  local heading_re="$2"
  local forbidden="$3"
  local section
  section="$(awk -v re="$heading_re" '
    $0 ~ /^## / { insec = ($0 ~ re) ? 1 : 0 }
    insec { print }
  ' "$ROOT_DIR/$repo_file" 2>/dev/null)"
  if [ -z "$section" ]; then
    fail "section matching /$heading_re/ not found in $repo_file (audit is stale or the section was renamed)"
    return
  fi
  if printf '%s\n' "$section" | grep -F -- "$forbidden" >/dev/null 2>&1; then
    fail "unexpected content in $repo_file section /$heading_re/: found '$forbidden' — the downstream contract must not instruct consumers to use PGEN-internal build inputs"
  fi
}

# Like assert_file_not_contains, but ignores COMMENT lines (`#`-led, leading whitespace allowed).
# CI-PARITY-GATE-ROT.1 (2026-07-27): needed because "this surface does not INVOKE X" is a claim
# about executable lines, and the whole-file form was failing on a comment that documents why the
# retired path once existed. Prose describing history is not a routing decision.
# ⚠️ Deliberately narrow: it strips only whole-line comments, so an invocation with a trailing
# comment on the same line is still caught. It is NOT a general "ignore anything that looks like a
# comment" relaxation, and it must not be used for assertions about documentation content.
assert_file_not_contains_uncommented() {
  local repo_file="$1"
  local forbidden="$2"
  if grep -v -E '^[[:space:]]*#' "$ROOT_DIR/$repo_file" 2>/dev/null |
       grep -F -- "$forbidden" >/dev/null 2>&1; then
    fail "unexpected file content in $repo_file: found '$forbidden' on a non-comment line"
  fi
}

copy_tracked_worktree() {
  local repo_rel
  note "exporting tracked working tree into $EXPORT_DIR"
  while IFS= read -r -d '' repo_rel; do
    mkdir -p "$EXPORT_DIR/$(dirname "$repo_rel")"
    cp -a "$ROOT_DIR/$repo_rel" "$EXPORT_DIR/$repo_rel"
  done < <(cd "$ROOT_DIR" && git ls-files -z)
}

audit_static_include_paths() {
  note "auditing include!(...) literals"
  if (cd "$ROOT_DIR" && rg -n 'include!\\(\"/' rust/src rust/src/bin -g '*.rs' >/dev/null 2>&1); then
    fail "absolute include!(...) literal found in rust/src or rust/src/bin"
  fi

  assert_generated_artifact "generated/ebnf.rs"
  assert_generated_artifact "generated/return_annotation_parser.rs"
  assert_generated_artifact "generated/semantic_annotation_parser.rs"
}

audit_markdown_repo_relative_paths() {
  note "auditing markdown repo-path policy"
  if (cd "$ROOT_DIR" && rg -n --glob '*.md' '/Users/richarddje/Documents/github/pgen/' . >/dev/null 2>&1); then
    fail "absolute PGEN checkout path found in markdown docs; use relative repo paths"
  fi
}

audit_root_markdown_surface() {
  local -a expected_root_md=(
    "AGENTS.md"
    "CHANGES.md"
    "CLAUDE.md"
    "COMMIT.md"
    "DEVELOPMENT_NOTES.md"
    "DOCTRINE_ENFORCEMENT.md"
    "GEMINI.md"
    "KNOWLEDGE_MAP.md"
    # LIVE-MEANS-LIVE.1c3 (2026-07-31): LIVE_ACHIEVEMENT_STATUS.md is DELETED. Its one load-bearing
    # value — the hand-authored family-status claim — moved to `claimed_status` in
    # rust/test_data/grammar_quality/done_bar_family_register_v0.json (.1a), and its human view to
    # docs/book/src/roadmap-and-live-status.md under a gate (.1c1). The file was 1,547,057 B of which
    # 94.7 % was 856 dated tracker notes; all 467 slice IDs it cited were re-measured as 467/467
    # reachable from a durable layer without it, so the delete is provably lossless.
    # ⚠️ This roster is an EXACT-SET comparison against `git ls-files`, so this entry had to be
    # removed in the SAME commit as the file — earlier and the audit fails, later and it also fails.
    "MEMORY_ARCHITECTURE.md"
    "MEMORY.md"
    "PGEN_USER_GUIDE.md"
    "QUICKSTART_AI_ONBOARDING.md"
    # README-POLICY.4 (2026-07-30, DIRECT DIRECTOR ORDER): the project-NEUTRAL README Stability
    # Policy, copied verbatim (byte-identical, 2,425 B) from the sibling repo that authored it.
    # It sits at the root beside the other PORTABLE standards it is a sibling of —
    # MEMORY_ARCHITECTURE.md, DOCTRINE_ENFORCEMENT.md, TOOLBOX.md — while PGEN's own INSTANCE of
    # it (routing table, chosen caps, adoption evidence) stays at
    # docs/reference/PGEN_README_STABILITY_POLICY.md. Standard at the root, instance under docs/.
    # ⚠️ Ordering is `sort`'s, not ASCII's: this sorts BEFORE README.md, exactly as
    # MEMORY_ARCHITECTURE.md sorts before MEMORY.md. Verified by running this audit's own
    # pipeline rather than by reasoning about collation.
    "README_POLICY.md"
    "README.md"
    "SESSION_BOOTSTRAP.md"
    "TOOLBOX.md"
  )
  local -a actual_root_md=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing root markdown allowlist"
  while IFS= read -r line; do
    actual_root_md+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\A[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_root_md[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_root_md[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected root markdown surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual root markdown surface:\n%s\n' "$actual_snapshot" >&2
    fail "root markdown allowlist drift detected; rehome stale root docs or update the tracked policy deliberately"
  fi
}

audit_top_level_docs_surface() {
  local -a expected_top_level_docs=(
    "docs/AST_GENERATOR_ARCHITECTURE.md"
    "docs/ast_transformation_pipeline.md"
    "docs/BOOTSTRAP_MODE_SPECIFICATION.md"
    "docs/EBNF_INCLUDE_SYSTEM.md"
    "docs/parser_architecture_evolution.md"
    # CI-PARITY-GATE-ROT.1 (2026-07-27): the four entries below were live, heavily-referenced
    # docs that this allowlist had never been updated for — the audit was RIGHT that no
    # deliberate policy update had happened, and this IS that deliberate update. Each was
    # verified before admission (added 2026-05-14/17, i.e. long-established; reference counts
    # measured across tracked markdown): TASK_TREE 70 refs and mandated by CLAUDE.md item 6,
    # TASK_TREE_README 9, POST_SV_AUDIT_LEDGER 23, SV_EXH_PROOF_BASELINE 6. None is a stray.
    # ⚠️ ORDER IS LOAD-BEARING: the expected list is compared verbatim against `sort` output, and
    # this locale sorts `TASK_TREE_README.md` BEFORE `TASK_TREE.md`. Keep entries in `sort` order.
    "docs/POST_SV_AUDIT_LEDGER.md"
    "docs/RETURN_ANNOTATIONS_REFERENCE.md"
    "docs/SV_EXH_PROOF_BASELINE.md"
    "docs/TASK_TREE_README.md"
    "docs/TASK_TREE.md"
    "docs/TEST_INFRASTRUCTURE.md"
  )
  local -a actual_top_level_docs=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing top-level docs allowlist"
  while IFS= read -r line; do
    actual_top_level_docs+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\Adocs\/[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_top_level_docs[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_top_level_docs[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected top-level docs surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual top-level docs surface:\n%s\n' "$actual_snapshot" >&2
    fail "top-level docs allowlist drift detected; prune stale docs or update the tracked policy deliberately"
  fi
}

audit_contract_docs_surface() {
  local -a expected_contract_docs=(
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md"
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md"
    "docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md"
    "docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
    # CI-PARITY-GATE-ROT.1 (2026-07-27): deliberate policy update, same class as the top-level
    # list above. The two RTL entries are the integration contracts for shipped parser families
    # README already documents (14 refs each, added 2026-05-15); the four SEMANTIC_STORE entries
    # are the store's contract family (added 2026-05-21, 2-6 refs each). All six long-established
    # and referenced; none is a stray awaiting rehome.
    "docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md"
    "docs/contracts/PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md"
    "docs/contracts/PGEN_SEMANTIC_STORE_SCHEMA_LANGUAGE_SPEC.md"
    "docs/contracts/PGEN_SEMANTIC_STORE_TEST_PLAN.md"
    "docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md"
    "docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md"
  )
  local -a actual_contract_docs=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing contract docs allowlist"
  while IFS= read -r line; do
    actual_contract_docs+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\Adocs\/contracts\/[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_contract_docs[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_contract_docs[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected contract docs surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual contract docs surface:\n%s\n' "$actual_snapshot" >&2
    fail "contract docs allowlist drift detected; rehome unexpected docs or update the tracked policy deliberately"
  fi
}

audit_reference_docs_surface() {
  local -a expected_reference_docs=(
    # CI-PARITY-GATE-ROT.1 (2026-07-27): deliberate policy update. PARSEABILITY_PROBE (added
    # 2026-05-04, 9 refs) documents a maintained TOOLBOX surface; SV_EXH_PROOF_DEFECT_TAXONOMY
    # (added 2026-05-23, 7 refs) belongs to the SV exhaustive-proof reference set.
    "docs/reference/PARSEABILITY_PROBE.md"
    "docs/reference/PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md"
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md"
    "docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md"
    "docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md"
    # README-POLICY.1 (2026-07-30): the adopted README Stability Policy. Tracked in-repo rather
    # than referenced across a volume boundary (CLAUDE.md §12/§13). ⚠️ ORDER IS LOAD-BEARING —
    # this list is compared verbatim against `sort` output, and README sorts before RELEASE.
    "docs/reference/PGEN_README_STABILITY_POLICY.md"
    "docs/reference/PGEN_RELEASE_POLICY.md"
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md"
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md"
    "docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md"
    "docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md"
    "docs/reference/RUST_CODEBASE_ANALYSIS.md"
    "docs/reference/STRESS_TEST_STANDARDIZATION.md"
    "docs/reference/SV_EXH_PROOF_DEFECT_TAXONOMY.md"
    "docs/reference/SV_GRAMMAR_COVERAGE_MATRIX.md"
  )
  local -a actual_reference_docs=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing reference docs allowlist"
  while IFS= read -r line; do
    actual_reference_docs+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\Adocs\/reference\/[^\/]+\.md\z/ }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' "${expected_reference_docs[@]}")"
  actual_snapshot="$(printf '%s\n' "${actual_reference_docs[@]}")"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'expected reference docs surface:\n%s\n' "$expected_snapshot" >&2
    printf 'actual reference docs surface:\n%s\n' "$actual_snapshot" >&2
    fail "reference docs allowlist drift detected; rehome unexpected docs or update the tracked policy deliberately"
  fi
}

audit_docs_book_surface() {
  # The expected set is DERIVED from docs/book/src/SUMMARY.md — the curation source
  # of truth (mdbook builds exactly the chapters SUMMARY references). A tracked
  # chapter file is legitimate iff SUMMARY.md references it (no stray uncurated
  # chapters), and every SUMMARY.md chapter reference must be a tracked file
  # (mdbook silently CREATES a missing chapter file at build time, so dangling
  # references matter too). The former hand-maintained allowlist here drifted 9
  # chapters behind the tracked surface — the duplicated-metadata class: a hand
  # copy of a derivable set silently rots (RGX-0078.8.t1).
  local -a summary_refs=()
  local -a tracked_chapters=()
  local expected_snapshot
  local actual_snapshot
  local line

  note "auditing docs/book surface (derived from SUMMARY.md)"

  for line in docs/book/book.toml docs/book/src/SUMMARY.md; do
    if ! (cd "$ROOT_DIR" && git ls-files --error-unmatch "$line" >/dev/null 2>&1); then
      fail "docs/book surface: required file '$line' is not tracked"
    fi
  done

  while IFS= read -r line; do
    summary_refs+=("docs/book/src/$line")
  done < <(
    cd "$ROOT_DIR" &&
      perl -ne 'print "$1\n" while /\]\(([^()\s]+\.md)\)/g' docs/book/src/SUMMARY.md |
      sort -u
  )

  while IFS= read -r line; do
    tracked_chapters+=("$line")
  done < <(
    cd "$ROOT_DIR" &&
      git ls-files -z |
      perl -0ne 'for (split /\0/) { print "$_\n" if /\Adocs\/book\/src\/.+\.md\z/ && $_ ne "docs/book/src/SUMMARY.md" }' |
      sort
  )

  expected_snapshot="$(printf '%s\n' ${summary_refs[@]+"${summary_refs[@]}"})"
  actual_snapshot="$(printf '%s\n' ${tracked_chapters[@]+"${tracked_chapters[@]}"})"

  if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
    printf 'SUMMARY.md-referenced chapters:\n%s\n' "$expected_snapshot" >&2
    printf 'tracked docs/book/src chapters:\n%s\n' "$actual_snapshot" >&2
    fail "docs/book surface drift: tracked chapters and SUMMARY.md chapter references must match exactly"
  fi
}

audit_active_docs_rehome_paths() {
  note "auditing active docs rehome paths"
  if (
    cd "$ROOT_DIR" &&
      rg -n \
        '(^|[^/])PGEN_SOTA_IMPLEMENTATION_ROADMAP\.md|(^|[^/])RUST_CODEBASE_ANALYSIS\.md|(^|[^/])PGEN_PARSER_INTEGRATION_CONTRACTS\.md|(^|[^/])PGEN_PARSER_ISSUE_REPORTING_PROTOCOL\.md|(^|[^/])PGEN_RELEASED_PARSER_BUG_LEDGER\.md|(^|[^/])PGEN_REGEX_PARSER_INTEGRATION_CONTRACT\.md|(^|[^/])PGEN_ANNOTATION_NORMATIVE_SPEC\.md|(^|[^/])PGEN_RELEASE_POLICY\.md|(^|[^/])PGEN_SEMANTIC_STEERING_CONTROL_MATRIX\.md|(^|[^/])PGEN_STIMULI_MODULE_NORMATIVE_SPEC\.md' \
        README.md \
        SESSION_BOOTSTRAP.md \
        QUICKSTART_AI_ONBOARDING.md \
        PGEN_USER_GUIDE.md \
        COMMIT.md \
        rust/docs/TECHNICAL_ARCHITECTURE.md \
        rust/docs/EMBEDDING_API_CONTRACT.md \
        rust/scripts/ci_workflow_local_gate.sh \
        docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md \
        docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md \
        docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md \
        docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md \
        docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md \
        docs/reference/PGEN_RELEASE_POLICY.md \
        docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md \
        docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md \
        docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md \
        docs/reference/RUST_CODEBASE_ANALYSIS.md \
        >/dev/null 2>&1
  ); then
    fail "active docs path drift detected; update live docs to use rehomed docs/contracts and docs/reference paths"
  fi
}

audit_workflow_surface() {
  local workflow_file
  note "auditing tracked workflow surface"
  for workflow_file in \
    .github/workflows/annotation-contract-gate.yml \
    .github/workflows/annotation-nonbootstrap-e2e-gate.yml \
    .github/workflows/branch-protection-contract-gate.yml \
    .github/workflows/differential-regression-gate.yml \
    .github/workflows/ebnf-frontend-dual-run-diff.yml \
    .github/workflows/fixed-point-gate.yml \
    .github/workflows/generated-clippy-correctness-gate.yml \
    .github/workflows/mdbook-docs-gate.yml \
    .github/workflows/performance-gate.yml \
    .github/workflows/stimuli-cross-family-platform-gate.yml \
    .github/workflows/sota-exit-gate.yml; do
    assert_tracked "$workflow_file"
  done

  assert_tracked "rust/config/branch_protection_policy.json"
  assert_tracked "rust/config/sota_exit_policy.env"
  assert_tracked "rust/scripts/annotation_nonbootstrap_e2e_gate.sh"
  assert_tracked "rust/scripts/branch_protection_contract_gate.sh"
  assert_tracked "rust/scripts/ebnf_frontend_dual_run_diff_gate.sh"
  assert_tracked "rust/scripts/fixed_point_bootstrap_gate.sh"
  assert_tracked "rust/scripts/performance_gate.sh"
  assert_tracked "rust/scripts/sota_exit_gate.sh"

  assert_workflow_contains \
    ".github/workflows/ebnf-frontend-dual-run-diff.yml" \
    "Verify Perl runtime for Perl-vs-Rust dual-run"
  assert_workflow_contains \
    ".github/workflows/sota-exit-gate.yml" \
    "Verify Perl runtime for SOTA dual-run surfaces"

  for workflow_file in \
    .github/workflows/annotation-contract-gate.yml \
    .github/workflows/annotation-nonbootstrap-e2e-gate.yml \
    .github/workflows/branch-protection-contract-gate.yml \
    .github/workflows/differential-regression-gate.yml \
    .github/workflows/fixed-point-gate.yml \
    .github/workflows/mdbook-docs-gate.yml \
    .github/workflows/performance-gate.yml \
    .github/workflows/stimuli-cross-family-platform-gate.yml; do
    assert_workflow_not_contains "$workflow_file" "Verify Perl runtime"
  done
}

# CI-PARITY-GATE-ROT.4 — the composite action every hosted workflow that compiles the crate must
# reference, and the make target that is the single definition of the sequence it runs.
REGENERATION_ACTION_PATH=".github/actions/regenerate-parsers/action.yml"
REGENERATION_ACTION_REF="uses: ./.github/actions/regenerate-parsers"
REGENERATION_MAKE_TARGET="regenerate_generated_parsers"

# ⭐ THE EXEMPTION SET — SMALL, EXPLICIT, AND EACH ENTRY MEASURED, NEVER ASSUMED.
# A workflow lands here only because a run against a tracked-files-only tree — the exact shape
# `actions/checkout` and `copy_tracked_worktree` both produce — was observed to PASS without any
# generated artifact. Everything else defaults to REQUIRING the step, which is the safe direction:
# a workflow added tomorrow fails this audit until someone either wires the step or measures it
# into this list. `.3`'s unprepared census (`docs/tasks/artifacts/ci_parity_gate_rot/`) is the
# evidence for all three.
#
#   branch-protection-contract-gate  PASS 0s   — shell + jq, never invokes cargo
#   fixed-point-gate                 PASS 23s  — builds ast_pipeline_bootstrap WITHOUT
#                                                --features generated_parsers
#   mdbook-docs-gate                 PASS 1s   — mdbook only
#
# ⛔ `.4`'s scope is explicit that "the 3 that pass must NOT pay for it", so exemption is asserted
# in BOTH directions: an exempt workflow that acquires the step also fails this audit. A 258 s
# regeneration bolted onto a 1 s shell check is a real cost regression, not a harmless extra.
# ⭐⭐ CI-PARITY-GATE-ROT.8 — THE EXEMPTION SET IS READ FROM THE SHARED REGISTER, NOT KEPT HERE.
# It used to be a `case` block in this file AND the same knowledge in the doctrine check, i.e. two
# lists that must agree — which is two lists that can disagree. `.1` found twelve assertions rotted
# on exactly that shape. Both readers now consult
# `rust/test_data/grammar_quality/flow_integrity_register_v0.json`, so "which workflows are exempt"
# has one definition and a change to it is visible to both at once.
# ⛔ REFUSES rather than defaulting to "not exempt" when the register cannot be read: silently
# treating every workflow as non-exempt would make the audit demand a 236s step of three jobs
# measured not to need one — a check that cannot read its own input must say so.
FLOW_INTEGRITY_REGISTER="rust/test_data/grammar_quality/flow_integrity_register_v0.json"

workflow_is_regeneration_exempt() {
  local workflow_file="$1"
  local exempt_list
  exempt_list="$(jq -er '.regeneration_exempt_workflows | keys[] | select(startswith("_") | not)' \
    "$ROOT_DIR/$FLOW_INTEGRITY_REGISTER" 2>/dev/null)" || \
    fail "cannot read the regeneration exemption set from $FLOW_INTEGRITY_REGISTER
  That register is the single source for which workflows were MEASURED not to need generated
  parsers. Without it this audit cannot tell an exempt workflow from a missing step."
  grep -qxF -- "$workflow_file" <<<"$exempt_list"
}

audit_workflow_regeneration_surface() {
  # CI-PARITY-GATE-ROT.4.
  #
  # ⭐⭐ THE DEFECT THIS EXISTS TO STOP RECURRING A FOURTH TIME. `generated/` is untracked
  # (`0ed2b2ad`, 2026-04-29), `actions/checkout` yields a tracked-files-only tree, and
  # `rust/src/lib.rs:72,78` include the annotation parsers by LITERAL path with no
  # `has_generated_*` cfg — so under `--features generated_parsers` their absence is a hard rustc
  # error that takes the whole crate down. Measured by `.3` against that same tree shape: 8 of the
  # 11 replayed workflow commands died on exactly this; measured by `.4`: 11 of the 15 tracked
  # workflows need the step, and before `.4` only ONE declared it.
  #
  # ⭐ THE ROSTER IS DERIVED, NOT HAND-LISTED — and that is the direct fix for how the last two
  # stayed unmeasured. `.3`'s census could only see the 11 workflows the parity gate replays, so
  # `rtl-const-expr-cert-gate` and `sv-cert-recognized-union-gate` sat outside every instrument
  # until `.4` measured them (both FAIL: `No rule to make target '../generated/ebnf.rs'`). This
  # audit walks `git ls-files .github/workflows`, so a workflow cannot be outside it.
  #
  # ⭐ AND THE PREDICATE IS FAIL-SAFE: needing the step is the DEFAULT, established by running a
  # `make -C rust` gate at all; not needing it requires a measured entry in the exemption set
  # above. The opposite polarity — a hand-list of workflows that need it — is the duplicated-
  # moving-metadata shape that rotted twelve assertions in `.1`.
  local workflow_file
  local needs_step=0
  local exempt=0
  note "auditing hosted-workflow regeneration surface"

  # (a) ONE HOME, asserted at both ends: the action must exist and must delegate to the make
  #     target, and the make target must exist. Neither may hold the sequence alone.
  assert_tracked "$REGENERATION_ACTION_PATH"
  assert_file_contains "$REGENERATION_ACTION_PATH" \
    "make -C rust SHELL=/bin/bash $REGENERATION_MAKE_TARGET"
  assert_file_contains "rust/Makefile" "$REGENERATION_MAKE_TARGET:"
  assert_file_contains "rust/Makefile" "GENERATED_PARSER_FAMILIES ="

  # (b) The local parity gate must call the SAME target, so its preparation and the hosted side
  #     cannot diverge. Before `.4` the sequence was written out twice and wiring ten more
  #     workflows would have made twelve copies whose drift nothing could detect.
  assert_file_contains "rust/scripts/ci_workflow_local_gate.sh" \
    "make -C rust SHELL=/bin/bash \"\$REGENERATION_MAKE_TARGET\""

  # (b2) …and preparation must stay ON BY DEFAULT. ⭐ This exists because the project has already
  #      watched this exact erosion once: `PGEN_CLIPPY_GENERATED_STRICT` defaulted to `0` and was
  #      set by no gate, no aggregate and no workflow, so a 291 → 0 correctness win was unguarded
  #      from the moment it landed (`GENERATED-LINT-CORRECTNESS.3`). A silent flip of this default
  #      back to `false` restores the state `.3` measured: a workflow phase that looks like it runs
  #      while eight of eleven replays cannot compile.
  #
  # ⚠️⚠️ AND IT IS WRITTEN THIS WAY BECAUSE THE OBVIOUS WAY IS UNSOUND — caught by this leaf's own
  # probe arms. A `assert_file_not_contains <this file> '<the forbidden literal>'` puts that literal
  # INTO the file it forbids it from, so the audit trips on its own source (measured: 8 of 12 arms
  # failed on `found 'PREPARE_RAW=…:-false…'`). The positive form is no better — it would match its
  # own text and pass vacuously. **An assertion about a file cannot live inside that file as a
  # literal**; the `PGEN_CLIPPY_GENERATED_STRICT` precedent only works because the asserted file is
  # a DIFFERENT one. So instead of matching text, extract the declared default and compare the
  # VALUE. The pattern is anchored at column 0 on `PREPARE_RAW=`, and this comment/extraction line
  # is indented, so neither can match itself.
  local prepare_default
  prepare_default="$(grep -oE '^PREPARE_RAW="\$\{PGEN_CI_WORKFLOW_LOCAL_PREPARE:-[a-z]+' \
    "$ROOT_DIR/rust/scripts/ci_workflow_local_gate.sh" | sed 's/.*:-//' | head -n 1)"
  if [[ "$prepare_default" != "true" ]]; then
    fail "PGEN_CI_WORKFLOW_LOCAL_PREPARE defaults to '${prepare_default:-<unparseable>}', expected 'true'.
  Since CI-PARITY-GATE-ROT.4 the hosted workflows regenerate generated/ themselves, so the local
  gate must too or it certifies a parity it is not testing: without preparation, 8 of the 11 replays
  cannot compile the crate at all. Set the variable to 0 for one run if you need to skip it; do not
  change the default. (docs/tasks/CI-PARITY-GATE-ROT.md leaves .3 and .4.)"
  fi

  # (c) No workflow may re-inline the recipe. `regex_parser_bootstrap` is its first step and its
  #     unambiguous signature; a copy pasted back into a workflow file is a second home.
  for workflow_file in $(cd "$ROOT_DIR" && git ls-files '.github/workflows/*.yml'); do
    assert_workflow_not_contains "$workflow_file" "regex_parser_bootstrap"
  done

  # (d) The derived roster: every tracked workflow that runs a `make -C rust` gate declares the
  #     step, and every measured-exempt one does not.
  for workflow_file in $(cd "$ROOT_DIR" && git ls-files '.github/workflows/*.yml'); do
    if workflow_is_regeneration_exempt "$workflow_file"; then
      exempt=$((exempt + 1))
      assert_workflow_not_contains "$workflow_file" "$REGENERATION_ACTION_REF"
      continue
    fi
    grep -F -- "make -C rust" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1 || continue
    needs_step=$((needs_step + 1))
    grep -F -- "$REGENERATION_ACTION_REF" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1 || \
      fail "workflow $workflow_file runs a 'make -C rust' gate but declares no regeneration step.
  generated/ is untracked, so actions/checkout gives this job a tree with no generated parsers and
  any command that compiles the crate with --features generated_parsers dies on
    error: couldn't read \`src/../../generated/return_annotation_parser.rs\`
  fix:  add this step before the gate step —
          - name: Regenerate the generated parsers
            $REGENERATION_ACTION_REF
  or, if this workflow genuinely needs no generated artifact, MEASURE that against a
  tracked-files-only tree and add it to workflow_is_regeneration_exempt with the measurement.
  (See docs/tasks/CI-PARITY-GATE-ROT.md leaf .4.)"

    # (e) A regeneration budget that does not fit the job's timeout is a workflow that cannot
    #     complete — the same class this tree keeps finding, one layer out. The step alone
    #     measured 258 s locally, from a WARM cargo registry, on a 3.6 TB volume; a hosted runner
    #     starts cold and is slower. 30 minutes is the floor for any job that carries it.
    #     ⚠️ This is a FLOOR, not a per-gate cost model: pinning each workflow's measured runtime
    #     here would be exactly the moving-value duplication `.1` found rotting twelve times.
    assert_workflow_timeout_at_least "$workflow_file" 30
  done

  note "regeneration surface: $needs_step workflow(s) require the step, $exempt measured-exempt"
  if [ "$needs_step" -lt 1 ]; then
    fail "audit_workflow_regeneration_surface matched no workflows at all — the roster derivation
  is broken (it walks 'git ls-files .github/workflows/*.yml'). A surface audit that inspects
  nothing must refuse, not pass: that vacuous-green shape is what this tree exists to remove."
  fi
}

# assert_workflow_timeout_at_least <workflow_file> <minutes>
# CI-PARITY-GATE-ROT.4. A missing `timeout-minutes` is also a failure: GitHub's default is 360
# minutes, so omitting it is not "unbounded is fine", it is "nobody priced this job".
assert_workflow_timeout_at_least() {
  local workflow_file="$1"
  local want="$2"
  local got
  got="$(grep -Eo '^[[:space:]]*timeout-minutes:[[:space:]]*[0-9]+' "$ROOT_DIR/$workflow_file" 2>/dev/null |
    grep -Eo '[0-9]+$' | head -n 1)"
  [ -n "$got" ] || \
    fail "workflow $workflow_file declares the regeneration step but no timeout-minutes; price the job"
  if [ "$got" -lt "$want" ]; then
    fail "workflow $workflow_file budgets timeout-minutes: $got, below the $want-minute floor for a
  job that regenerates generated/ first (the regeneration alone measured 258 s locally, warm, on a
  fast volume; hosted runners start cold). Raise it or the job cannot complete."
  fi
}

audit_ebnf_frontend_conversion_surface() {
  local repo_file
  note "auditing ebnf_to_json conversion surface"

  for repo_file in \
    rust/Makefile \
    rust/scripts/annotation_nonbootstrap_e2e_gate.sh \
    rust/scripts/fixed_point_bootstrap_gate.sh \
    rust/scripts/hdl_frontend_readiness_gate.sh \
    rust/scripts/stimuli_module_parity_gate.sh \
    rust/scripts/sv_external_corpus_triage_gate.sh \
    rust/scripts/sv_preprocessor_quality_gate.sh \
    rust/scripts/sv_stimuli_quality_gate.sh \
    rust/scripts/sv_syntax_closure_gate.sh \
    rust/scripts/vhdl_external_corpus_triage_gate.sh \
    rust/scripts/vhdl_stimuli_quality_gate.sh; do
    assert_tracked "$repo_file"
    # CI-PARITY-GATE-ROT.1 (2026-07-27): this audit exists to prove these surfaces do not ROUTE
    # through the retired Perl converter. It was failing on `rust/Makefile:782`, where the sole
    # occurrence is a COMMENT documenting the bootstrap seed strategy — measured: zero
    # non-comment occurrences in that file, i.e. the migration the audit checks for is COMPLETE
    # and the audit was reporting its own documentation back to it.
    # ⛔ NOT force-greened, and deliberately NOT scoped down to "the Makefile is exempt": the
    # assertion is narrowed to EXECUTABLE lines for every file in the list, so a real invocation
    # re-added to ANY of them still fails. Comment text stays free to describe history.
    # ⭐ LANG-CAPABILITY-AUDIT.10.6 (2026-07-30) — the carve-out that used to sit here is GONE.
    # It read: "the Perl path is still live in the hybrid flow (README.md 'perl/: legacy/frontend
    # EBNF-to-JSON path ... still used in hybrid flow'; the active consumer is
    # rust/scripts/ebnf_stimuli_quality_gate.sh, deliberately not in this list)". Two things were
    # wrong with it: README.md has not mentioned Perl since README-POLICY.1 trimmed the path
    # inventory out, so the pin rested on a citation to a line that does not exist; and the three
    # carved-out scripts were not merely exempt, they were POSITIVELY ASSERTED to keep calling
    # Perl (see the inverted block below), which is why a component the project describes as
    # retired survived as a REQUIRED stage of sota_exit_gate.
    assert_file_not_contains_uncommented "$repo_file" "ebnf_to_json.pl"
  done

  # ⭐⭐ LANG-CAPABILITY-AUDIT.10.6 — THE PINS ARE INVERTED. These three scripts used to be
  # `assert_file_contains`'d on their Perl invocations, i.e. removing Perl FAILED this gate.
  # They are now asserted to be Perl-FREE on executable lines, so the gate enforces the
  # project's intent instead of the residue it left behind. The de-Perl campaign had in fact
  # landed everywhere except one hard-coded line in ebnf_stimuli_quality_gate.sh, which
  # bypassed that script's own run_frontend_to_json() helper.
  for repo_file in \
    rust/scripts/ebnf_frontend_dual_run_diff_gate.sh \
    rust/scripts/ebnf_frontend_readiness_gate.sh \
    rust/scripts/ebnf_stimuli_quality_gate.sh; do
    assert_tracked "$repo_file"
    assert_file_not_contains_uncommented "$repo_file" "ebnf_to_json.pl"
    assert_file_not_contains_uncommented "$repo_file" "EBNF_TO_JSON"
  done

  # The retired knob must stay REJECTED rather than silently ignored: a stale
  # `PGEN_EBNF_FRONTEND_IMPL=perl` in some CI job has to fail loudly, not pick a default.
  assert_file_contains \
    "rust/scripts/ebnf_frontend_readiness_gate.sh" \
    'if [[ "$FRONTEND_IMPL" != "rust" ]]; then'
  assert_file_contains \
    "rust/scripts/ebnf_stimuli_quality_gate.sh" \
    'if [[ "$FRONTEND_IMPL" != "rust" ]]; then'
}

audit_embedding_api_surface() {
  note "auditing public embedding API surface"

  assert_generated_artifact "generated/regex.json"
  assert_generated_artifact "generated/regex_parser.rs"
  assert_tracked "rust/src/embedding_api.rs"
  assert_tracked "rust/docs/EMBEDDING_API_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md"
  assert_tracked "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md"
  assert_tracked "docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md"
  assert_tracked "docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md"
  assert_tracked "rust/scripts/regex_parser_integration_contract_gate.sh"
  assert_tracked "rust/scripts/regex_embedded_code_block_contract_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/regex_parser_integration_contract_v1.json"
  assert_tracked "rust/test_data/grammar_quality/regex_embedded_code_block_contract_v0.json"

  # CI-PARITY-GATE-ROT.1 (2026-07-27): these three were pinned as LITERALS (1.2.0 / 1.1.31 /
  # 1.1.29) against constants the release policy moves deliberately — live values at adjudication
  # were 1.3.1 / 1.1.109 / 1.1.106. A parity audit that hard-pins a version it expects to change
  # fails on every legitimate bump, so it rots by construction and gets ignored. ⛔ NOT deleted:
  # the invariant this audit really owns is "the public embedding surface still DECLARES these
  # constants, well-formed", which the shape assertion below enforces exactly. The VALUES are
  # already owned by the release-policy gates and the integration-contract identity blocks — this
  # audit was duplicating that ownership, which is what made it drift.
  assert_file_matches \
    "rust/src/embedding_api.rs" \
    'pub const EMBEDDING_API_VERSION: &str = "[0-9]+\.[0-9]+\.[0-9]+";'
  assert_file_matches \
    "rust/src/embedding_api.rs" \
    'pub const REGEX_PARSER_INTEGRATION_CONTRACT_VERSION: &str = "[0-9]+\.[0-9]+\.[0-9]+";'
  assert_file_matches \
    "rust/src/embedding_api.rs" \
    'pub const REGEX_PARSER_RELEASE_VERSION: &str = "[0-9]+\.[0-9]+\.[0-9]+";'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub const REGEX_AST_DUMP_SCHEMA_VERSION: u32 = 1;'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    '#[serde(rename = "regex")]'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    '#[serde(rename = "regex_default")]'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub struct ParseDiagnosticLocation {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub supports_regex_generated_backend: bool,'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub stable_diagnostic_location_fields: Vec<String>,'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub regex_parser_release_version: String,'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub fn parse_regex_default(input: &str) -> GrammarParseOutcome {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'pub fn parse_regex_default_ast_dump('
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'GrammarFamily::Regex => parse_generated_regex(input),'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_generated_backend_enabled() -> bool {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_metadata_is_stable() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_failures_are_machine_localizable() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_accepts_declared_success_samples() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_classifies_whole_pattern_recursion_as_subroutine_call() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_classifies_returned_capture_subroutine() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_classifies_numeric_backreferences() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_preserves_conditional_false_branch() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_binds_quantifier_to_final_literal_atom() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn parse_diagnostic_location_is_one_based_and_clamped_to_utf8_boundaries() {'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'fn regex_parser_integration_contract_rejects_declared_failure_samples() {'

  assert_file_contains \
    "rust/Makefile" \
    'regex_parser_integration_contract_gate - Validate regex parser integration contract (consumer-facing convenience API + diagnostics)'
  assert_file_contains \
    "rust/Makefile" \
    'regex_embedded_code_block_contract_gate - Validate regex embedded code-block structural contract over the checked-in synthetic corpus'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) regex_parser_integration_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_embedded_code_block_contract_gate.sh'
  assert_file_contains \
    "rust/scripts/regex_parser_integration_contract_gate.sh" \
    'cargo test --lib regex_parser_integration_contract_'
  assert_file_contains \
    "rust/scripts/regex_parser_integration_contract_gate.sh" \
    'cargo test --features generated_parsers --lib regex_parser_integration_contract_'
  assert_file_contains \
    "rust/scripts/regex_embedded_code_block_contract_gate.sh" \
    'regex_embedded_code_block_contract_v0.json'
  assert_file_contains \
    "rust/scripts/regex_embedded_code_block_contract_gate.sh" \
    '--parse "$expected_parser_type" "$case_input_file" --profile "$expected_profile"'

  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    'Provide a stable, versioned surface for external projects embedding PGEN annotation parsing and selected grammar parsing'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`parse_regex_default(...)`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`parse_regex_default_ast_dump(...)`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`make -C rust regex_parser_integration_contract_gate`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`GrammarFamily`: `systemverilog | vhdl | regex`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`GrammarProfile`: `sv_2017 | sv_2023 | verilog_2005 | vhdl_1076_2019 | regex_default`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`ParseDiagnostic`: stable `code` + human-readable `message` + optional `location`.'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`regex_parser_release_version`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`regex_generated_backend_required_feature`'
  assert_file_contains \
    "rust/docs/EMBEDDING_API_CONTRACT.md" \
    '`embedding_api_gate` now covers the public regex parser/profile surface too.'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- `regex`: `regex_default`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- `parse_regex_default(...)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- `parse_regex_default_ast_dump(...)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'the public embedding API now exposes regex through `regex_default`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust regex_parser_integration_contract_gate'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '### Regex Parser Flavor'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`parseability_attempts_total=5911`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?<name>[a-z]+)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`[[:^alnum:]]+`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?<A>foo)-\\k{A}`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`a{,4}`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?1(1))`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?{lua: return x + 1})`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`(?{javascript:return x + 1;})`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'plain `(?{...})` is preserved as opaque generic payload'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'language tags `lua`, `js`, `javascript`, and `rhai` are preserved as opaque source-body payloads'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust regex_embedded_code_block_contract_gate'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'this is not a host-language regex literal parser for wrapper forms such as `/pattern/flags`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- whole-pattern recursion via `(?R)`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '- explicit false branches are preserved separately, so `(?(1)a|b)` transports `a` and `b` as distinct yes/no branches'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '  - `(?R)` now appears as `subroutine_call` / `subroutine_target`, not `inline_modifiers`'

  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    'Every current and future parser family that PGEN publishes for downstream consumption must have a tracked integration-contract document.'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    'its family document must publish:'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    '- `Contract Identity`'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    '| `regex` | `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` |'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    '`docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'Accepted reports should then be logged in:'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'If one or more downstream consumer repos also track the same issue locally'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'The parser family/profile is the primary tracking axis for released-parser support.'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'for regex, copy `parser_embedding_api_contract().regex_parser_release_version`'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md" \
    'for regex, copy `parser_embedding_api_contract().regex_integration_contract_version`'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    'Every downstream bug report against a released parser family must receive a stable report ID.'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    '`Downstream Tracking Refs`'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    '`Reported Against Parser Release`'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    'The primary index for this ledger is `Parser Family/Profile`.'
  assert_file_contains \
    "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" \
    'the parser release version containing the fix'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'This is the document downstream projects such as RGX should read first when deciding how to embed the PGEN regex parser.'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '- Contract version:'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '- Parser release version:'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'make -C rust regex_parser_integration_contract_gate'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'Issue Reporting Quick Path'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'PGEN_TRACE_VERBOSITY=debug'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'Regex AST-dump schema version:'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'stable optional machine-localizable location object'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'Published Regex Flavor Summary'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '## Release 1.1.29 / Contract 1.1.31 Highlights'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'raw regex bodies, not host-language delimiter wrappers'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '`(?1(1))`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'braced named backreferences such as `\k{name}`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'subroutine-reference forms such as `\g{1}` and `\g<1>`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'whole-pattern recursion `(?R)` now classifies as `subroutine_call` / `subroutine_target`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'numeric backreferences such as `\1` now classify as `backreference` instead of generic `escape`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'negated POSIX classes such as `[[:^alnum:]]`'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'explicit conditional false branches such as `(?(1)a|b)` now preserve separate `yes_branch` and `no_branch` spans'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'plain `(?{...})` is preserved as opaque generic payload'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '`lua`, `js`, `javascript`, and `rhai` payloads are preserved as opaque source-body payloads'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'make -C rust regex_embedded_code_block_contract_gate'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    'make -C rust regex_pcre2_compile_oracle_gate'
  assert_file_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`'
  # CI-PARITY-GATE-ROT.1b (2026-07-27, DIRECTOR RULING: the boundary STANDS). Scoped from
  # whole-file to the consumer-facing build recipe — see assert_markdown_section_not_contains for
  # why the whole-file form was unsatisfiable without erasing ~35 historical provenance notes.
  assert_markdown_section_not_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '^## Generated Parser Build Recipe' \
    'generated/regex.json'
  assert_markdown_section_not_contains \
    "docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md" \
    '^## Generated Parser Build Recipe' \
    'grammars/regex.ebnf'
  assert_file_contains \
    "docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md" \
    'local git-tracked records in PGEN plus zero-or-more downstream consumer repos are sufficient'
  assert_file_contains \
    "README.md" \
    '`docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`'
  assert_file_contains \
    "README.md" \
    '`docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`'
  assert_file_contains \
    "README.md" \
    '`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`'
  assert_file_contains \
    "COMMIT.md" \
    '`docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md` and `docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md`'
  assert_file_contains \
    "COMMIT.md" \
    '`docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`'

  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'internal parser-registry or probe availability automatically means the same family already has a public embedding contract'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'It now also has a public embedding seam in `embedding_api.rs`, but that public surface should not be mistaken for complete parser-family closure by itself'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'dedicated downstream integration contract doc plus a regex-specific host contract gate'
}

audit_rtl_frontend_generated_contract_surface() {
  note "auditing rtl_frontend generated contract surface"

  assert_tracked ".github/workflows/rtl-frontend-generated-contract-gate.yml"
  assert_generated_artifact "generated/rtl_frontend.json"
  assert_generated_artifact "generated/rtl_frontend_parser.rs"
  assert_tracked "grammars/rtl_frontend.ebnf"
  assert_tracked "rtl_frontend/Cargo.toml"
  assert_tracked "rtl_frontend/src/lib.rs"
  assert_tracked "rust/src/bin/rtl_frontend_generated_contract_probe.rs"
  assert_tracked "rust/src/parser_registry.rs"
  assert_tracked "rust/scripts/rtl_frontend_generated_contract_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json"

  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'rtl_frontend_generated_parity_contract_v0.json'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'parse_sample("rtl_frontend", &sample.sample)'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'parse_sample_ast_json("rtl_frontend", &sample.sample)'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'required_rule_names'
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'forbidden_rule_names'
  # CI-PARITY-GATE-ROT.1 (2026-07-27): was pinned to `expected_rule_texts`, which is the `0.1.0`
  # RAW-ENVELOPE retention layer. The `0.2.0` typed-AST migration RETIRED that layer — README
  # records it as having become "structurally unsatisfiable once the released typed-AST campaign
  # (schema 3) folded the structural rules into the typed carrier" — and the probe now carries 0
  # occurrences of the old token. Re-pinned to the token that replaced it (measured: 4
  # occurrences), so the audit tracks the layer that actually exists.
  assert_file_contains \
    "rust/src/bin/rtl_frontend_generated_contract_probe.rs" \
    'required_typed_string_values'
  assert_file_contains \
    "rust/src/parser_registry.rs" \
    'fn rtl_frontend_generated_contract_metadata_is_stable() {'
  assert_file_contains \
    "rust/src/parser_registry.rs" \
    'fn rtl_frontend_generated_contract_samples_hold() {'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"required_rule_names"'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"forbidden_rule_names"'
  # CI-PARITY-GATE-ROT.1 (2026-07-27): the same `0.1.0` retirement as the probe pin above, and a
  # perfect illustration of the class: the ONE surviving occurrence of `expected_rule_texts` in
  # this contract is inside its own `provenance` narrative — the sentence recording that the lock
  # was RETIRED ("required_rule_texts/expected_rule_texts span locks ... retired; their curated
  # texts were re-expressed as required_typed_string_values"). The audit was pinned to a token that
  # survives only in the explanation of its own removal. Re-pinned to the live successor
  # (76 occurrences).
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"required_typed_string_values"'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"expected_elaboration"'
  assert_file_contains \
    "rtl_frontend/src/lib.rs" \
    'expected_handwritten_parse_ok'
  assert_file_contains \
    "rtl_frontend/src/lib.rs" \
    'expected_elaboration'
  # CI-PARITY-GATE-ROT.1 (2026-07-27): pinned the RATCHET value 54; live is 59. A ratchet exists to
  # be raised as coverage grows, so pinning its exact value guarantees the audit fails on every
  # legitimate ratchet bump - the same design flaw as the version pins above. The invariant this
  # audit owns is "the elaboration ratchet is still DECLARED"; its VALUE is owned and enforced by
  # the handwritten replay tests that ratchet it.
  assert_file_matches \
    "rtl_frontend/src/lib.rs" \
    'MIN_GENERATED_CONTRACT_ELABORATION_SAMPLES: usize = [0-9]+'
  assert_file_matches \
    "rtl_frontend/src/lib.rs" \
    'MIN_GENERATED_CONTRACT_ELABORATION_ACCEPTS: usize = [0-9]+'
  assert_file_matches \
    "rtl_frontend/src/lib.rs" \
    'MIN_GENERATED_CONTRACT_ELABORATION_REJECTS: usize = [0-9]+'
  assert_file_matches \
    "rtl_frontend/src/lib.rs" \
    'MIN_GENERATED_CONTRACT_ELABORATION_CHILD_PATH_SAMPLES: usize = [0-9]+'
  assert_file_matches \
    "rtl_frontend/src/lib.rs" \
    'MIN_GENERATED_CONTRACT_ELABORATION_TOP_PARAMETER_CHECKS: usize = [0-9]+'
  assert_file_matches \
    "rtl_frontend/src/lib.rs" \
    'MIN_GENERATED_CONTRACT_ELABORATION_CHILD_PARAMETER_CHECKS: usize = [0-9]+'
  assert_file_matches \
    "rtl_frontend/src/lib.rs" \
    'MIN_GENERATED_CONTRACT_ELABORATION_CHILD_PORT_BINDING_CHECKS: usize = [0-9]+'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"top_parameters"'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"child_paths"'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"child_parameters"'
  assert_file_contains \
    "rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json" \
    '"child_port_bindings"'
  assert_file_contains \
    "rtl_frontend/src/lib.rs" \
    'current rtl_frontend generated contract samples should parse the same way'
  assert_file_contains \
    "rtl_frontend/src/lib.rs" \
    'fn generated_contract_manifest_matches_handwritten_elaboration_surface()'
  assert_file_contains \
    "rust/scripts/rtl_frontend_generated_contract_gate.sh" \
    'cargo run --features generated_parsers --bin rtl_frontend_generated_contract_probe'
  assert_file_contains \
    "rust/scripts/rtl_frontend_generated_contract_gate.sh" \
    'cargo test --manifest-path ../rtl_frontend/Cargo.toml'
  assert_file_contains \
    "rtl_frontend/src/lib.rs" \
    'fn generated_contract_manifest_matches_handwritten_parse_surface()'
  assert_file_contains \
    "rust/scripts/rtl_frontend_generated_contract_gate.sh" \
    'generated_contract_manifest_matches_handwritten --lib'
  assert_file_contains \
    "rtl_frontend/src/lib.rs" \
    '../rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json'
  assert_file_contains \
    ".github/workflows/rtl-frontend-generated-contract-gate.yml" \
    'make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate'
  assert_file_contains \
    ".github/workflows/rtl-frontend-generated-contract-gate.yml" \
    'path: rust/target/rtl_frontend_generated_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    'rtl_frontend_generated_contract_gate - Validate curated rtl_frontend generated and handwritten contract samples'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/rtl_frontend_generated_contract_gate.sh'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate`'
}

audit_stimuli_cross_family_platform_surface() {
  note "auditing cross-family stimuli platform surface"

  assert_tracked ".github/workflows/stimuli-cross-family-platform-gate.yml"
  assert_tracked "rust/scripts/stimuli_cross_family_platform_gate.sh"
  assert_tracked "rust/scripts/ebnf_stimuli_quality_gate.sh"
  assert_tracked "rust/scripts/vhdl_stimuli_quality_gate.sh"
  assert_tracked "rust/scripts/sv_stimuli_quality_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/regex_family_stimuli_contract.json"
  assert_tracked "rust/test_data/grammar_quality/vhdl_stimuli_cross_family_platform_contract_v0.json"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_stimuli_cross_family_platform_contract_v0.json"
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_stimuli_cross_family_platform_contract_v0.json" \
    '"default_mode": "sv_parseable_file"'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_stimuli_cross_family_platform_contract_v0.json" \
    '"target_max_attempts": 50'

  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'SUMMARY_JSON="${REPORT_DIR}/summary.json"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_CONTRACT="${REGEX_CONTRACT_FILE}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'echo "generated_at_utc: ${generated_at_utc}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'echo "summary_json: ${SUMMARY_JSON}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_COUNT="${REGEX_COUNT}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${REGEX_TARGET_MAX_ATTEMPTS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_VHDL_STIMULI_QUALITY_CONTRACT="${VHDL_CONTRACT_FILE}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_VHDL_STIMULI_CARGO_TARGET_DIR=target'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${VHDL_TARGET_MAX_ATTEMPTS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_QUALITY_CONTRACT="${SV_CONTRACT_FILE}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_QUALITY_LRM_PROFILES="${SV_LRM_PROFILES}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    '--arg summary_json "${SUMMARY_JSON}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'summary_json: $summary_json'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'CROSS_FAMILY_CARGO_BUILD_JOBS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_CARGO_BUILD_JOBS:-1}"'
  assert_file_contains \
    "rust/scripts/stimuli_cross_family_platform_gate.sh" \
    'PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${SV_TARGET_MAX_ATTEMPTS}"'
  assert_file_contains \
    ".github/workflows/stimuli-cross-family-platform-gate.yml" \
    'make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate'
  assert_file_contains \
    ".github/workflows/stimuli-cross-family-platform-gate.yml" \
    'path: rust/target/stimuli_cross_family_platform_gate'
  assert_file_contains \
    "rust/Makefile" \
    'stimuli_cross_family_platform_gate - Validate bounded shared stimuli-platform replay across regex, VHDL, and SystemVerilog'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/stimuli_cross_family_platform_gate.sh'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate`'
}

audit_annotation_aggregate_contract_surface() {
  note "auditing aggregate annotation contract gate surface"

  assert_tracked "rust/scripts/annotation_robustness_gate.sh"
  assert_tracked "rust/scripts/annotation_stimuli_quality_gate.sh"

  assert_file_contains \
    "rust/Makefile" \
    'annotation_contract_gate - Enforce normative bootstrap annotation contracts + validator diagnostics'
  assert_file_contains \
    "rust/Makefile" \
    'annotation_shared_contract_gate - Enforce shared bootstrap/generated annotation contracts'
  assert_file_contains \
    "rust/Makefile" \
    'annotation_robustness_gate - Enforce advanced annotation suites + generated parseability/coverage checks'
  assert_file_contains \
    "rust/Makefile" \
    'annotation_stimuli_quality_gate - Enforce strict closed-loop stimuli/coverage/gap checks (no-regression) for annotation grammars'
  assert_file_contains \
    "rust/Makefile" \
    'semantic_runtime_contract_gate - Enforce semantic runtime/typed-AST contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'semantic_ast_roundtrip_gate - Enforce semantic AST round-trip contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'semantic_full_contract_gate - Enforce aggregate semantic contract gate (runtime + round-trip + regression)'
  assert_file_contains \
    "rust/Makefile" \
    'return_annotation_support_gate - Enforce aggregate 100% return-annotation support proof (audit + contract + stimuli)'
  assert_file_contains \
    "rust/Makefile" \
    'return_runtime_semantics_gate - Enforce typed return AST/runtime transform contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'return_ast_roundtrip_gate - Enforce canonical return AST round-trip contract checks'
  assert_file_contains \
    "rust/Makefile" \
    'return_parity_gate - Enforce zero return mismatches on comparable (expectation-aligned) differential corpus'
  assert_file_contains \
    "rust/Makefile" \
    'return_full_contract_gate - Enforce aggregate return contract gate (runtime + round-trip + parity)'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib annotation_validator'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_builtin_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_builtin_contract'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) annotation_shared_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_full_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_full_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) annotation_robustness_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) annotation_stimuli_quality_gate'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_normative_shared_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser return --suite return_annotation_normative_shared_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib semantic_validator_'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --features generated_parsers --lib generated_semantic_tree_to_ast_'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_usage_gate'

  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_runtime_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_ast_roundtrip_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) semantic_differential_regression_gate'

  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib unified_return_ast'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --features generated_parsers --lib generated_return_tree_to_typed_ast_'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib return_validator'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && cargo test --lib test_round_trip_runner'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_runtime_semantics_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_ast_roundtrip_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) return_parity_gate'

  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_shared_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_robustness_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_runtime_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_ast_roundtrip_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust semantic_full_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_runtime_semantics_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_ast_roundtrip_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust return_full_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust annotation_stimuli_quality_gate`'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_shared_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_usage_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_runtime_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_ast_roundtrip_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`semantic_full_contract_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_robustness_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`annotation_stimuli_quality_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_annotation_support_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_runtime_semantics_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_parity_gate` (local gate target)'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`return_full_contract_gate` (local gate target)'

  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash annotation_contract_gate`'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash semantic_full_contract_gate`'
  assert_file_contains \
    "README.md" \
    '`make -C rust SHELL=/bin/bash return_annotation_support_gate`'

  assert_file_contains \
    "QUICKSTART_AI_ONBOARDING.md" \
    '`annotation_contract_gate`, `semantic_full_contract_gate`,'
  assert_file_contains \
    "QUICKSTART_AI_ONBOARDING.md" \
    '`return_annotation_support_gate`, and `annotation_stimuli_quality_gate`.'

  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '## Rust-To-Shell Contract Seams'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '- Aggregate annotation proof seam'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '- `annotation_contract_gate`'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '- `return_annotation_support_gate`'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'high-level entrypoints into aggregate annotation proof surfaces'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'operator-facing map of aggregate annotation / semantic / return local gates'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'annotation proof obligations and gate targets behind aggregate annotation claims'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'If an annotation leaf suite or one SC gate passes, the repo-level annotation proof claim is done.'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Aggregate annotation proof composition'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Operator-facing annotation gate map'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Aggregate annotation proof contract'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '### If the task is return/semantic annotation parsing or validation'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'And pick the nearest aggregate proof surface:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '### If the task is proof plumbing, contract sidecars, or release-gate behavior'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'For annotation-specific proof plumbing, narrow quickly to:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'the nearest aggregate annotation proof surface:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'if the parser is `return_annotation` or `semantic_annotation`, usually also add:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'for annotation-focused stimuli work, usually also add:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'for annotation-proof changes, the practical aggregate readers are usually:'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Symptom: Annotation-focused unit tests or leaf suites pass, but the repo-level annotation proof still feels wrong or incomplete'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'Annotation proof / closure problem'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'stopping at the leaf suite that passed instead of checking which aggregate proof claim the repo is actually making'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    'patching the top-level proof claim before the nearest annotation seam is correct tends to hide whether the real drift is semantic behavior, closure evidence, or just the proof map'
}

audit_annotation_semantic_contract_surface() {
  note "auditing annotation semantic contract surface"

  assert_tracked "rust/scripts/sc01_contract_gate.sh"
  assert_tracked "rust/scripts/sc02_contract_gate.sh"
  assert_tracked "rust/scripts/sc03_contract_gate.sh"
  assert_tracked "rust/scripts/sc04_contract_gate.sh"
  assert_tracked "rust/scripts/sc05_contract_gate.sh"
  assert_tracked "rust/scripts/sc06_contract_gate.sh"
  assert_tracked "rust/scripts/sc07_contract_gate.sh"
  assert_tracked "rust/scripts/sc08_contract_gate.sh"
  assert_tracked "rust/scripts/sc09_contract_gate.sh"
  assert_tracked "rust/scripts/sc10_contract_gate.sh"
  assert_tracked "rust/scripts/sc11_contract_gate.sh"
  assert_tracked "rust/scripts/sc12_contract_gate.sh"
  assert_tracked "rust/scripts/sc13_contract_gate.sh"
  assert_tracked "rust/test_data/semantic_annotation/sc01_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc02_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc03_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc04_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc05_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc06_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc07_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc08_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc09_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc10_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc11_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc12_contract.json"
  assert_tracked "rust/test_data/semantic_annotation/sc13_contract.json"

  assert_file_contains \
    "rust/Makefile" \
    'sc01_contract_gate - Enforce SC-01 canonical-transform Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc02_contract_gate - Enforce SC-02 raw literal sample-hint Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc03_contract_gate - Enforce SC-03 directive routing + strict policy contract slices and differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc04_contract_gate - Enforce SC-04 token steering Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc05_contract_gate - Enforce SC-05 precedence/associativity Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc06_contract_gate - Enforce SC-06 branch weighting/selection Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc07_contract_gate - Enforce SC-07 recovery/sync Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc08_contract_gate - Enforce SC-08 value-domain Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc09_contract_gate - Enforce SC-09 relational-constraint Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc10_contract_gate - Enforce SC-10 coverage-target Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc11_contract_gate - Enforce SC-11 negative-case Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc12_contract_gate - Enforce SC-12 deterministic-partition Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    'sc13_contract_gate - Enforce SC-13 profiles/runtime-scaffold Tier-4 contract slices + differential taxonomy checks'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc01_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc02_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc03_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc04_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc05_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc06_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc07_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc08_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc09_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc10_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc11_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc12_contract_gate'
  assert_file_contains \
    "rust/Makefile" \
    '@$(MAKE) -C $(RUST_DIR) sc13_contract_gate'

  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc01_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc02_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc03_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc04_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc05_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc06_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc07_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc08_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc09_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc10_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc11_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc12_contract_gate`'
  assert_file_contains \
    "docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md" \
    '`make -C rust sc13_contract_gate`'

  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc01_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc02_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc03_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc04_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc05_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc06_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc07_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc08_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc09_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc10_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc11_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc12_contract'
  assert_file_contains \
    "docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md" \
    'semantic_annotation_sc13_contract'

  assert_file_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    'semantic_annotation_sc01_contract'
  assert_file_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    'target/sc01_contract_gate/work/sc01_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    '(.mismatched_cases == 0)'
  assert_file_not_contains \
    "rust/scripts/sc01_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc02_contract_gate.sh" \
    'semantic_annotation_sc02_contract'
  assert_file_contains \
    "rust/scripts/sc02_contract_gate.sh" \
    'target/sc02_contract_gate/work/sc02_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc02_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc03_contract_gate.sh" \
    'semantic_annotation_sc03_contract'
  assert_file_contains \
    "rust/scripts/sc03_contract_gate.sh" \
    'target/sc03_contract_gate/work/sc03_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc03_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc04_contract_gate.sh" \
    'semantic_annotation_sc04_contract'
  assert_file_contains \
    "rust/scripts/sc04_contract_gate.sh" \
    'target/sc04_contract_gate/work/sc04_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc04_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc05_contract_gate.sh" \
    'semantic_annotation_sc05_contract'
  assert_file_contains \
    "rust/scripts/sc05_contract_gate.sh" \
    'target/sc05_contract_gate/work/sc05_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc05_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc06_contract_gate.sh" \
    'semantic_annotation_sc06_contract'
  assert_file_contains \
    "rust/scripts/sc06_contract_gate.sh" \
    'target/sc06_contract_gate/work/sc06_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc06_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc07_contract_gate.sh" \
    'semantic_annotation_sc07_contract'
  assert_file_contains \
    "rust/scripts/sc07_contract_gate.sh" \
    'target/sc07_contract_gate/work/sc07_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc07_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc08_contract_gate.sh" \
    'semantic_annotation_sc08_contract'
  assert_file_contains \
    "rust/scripts/sc08_contract_gate.sh" \
    'target/sc08_contract_gate/work/sc08_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc08_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc09_contract_gate.sh" \
    'semantic_annotation_sc09_contract'
  assert_file_contains \
    "rust/scripts/sc09_contract_gate.sh" \
    'target/sc09_contract_gate/work/sc09_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc09_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc10_contract_gate.sh" \
    'semantic_annotation_sc10_contract'
  assert_file_contains \
    "rust/scripts/sc10_contract_gate.sh" \
    'target/sc10_contract_gate/work/sc10_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc10_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc11_contract_gate.sh" \
    'semantic_annotation_sc11_contract'
  assert_file_contains \
    "rust/scripts/sc11_contract_gate.sh" \
    'target/sc11_contract_gate/work/sc11_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc11_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc12_contract_gate.sh" \
    'semantic_annotation_sc12_contract'
  assert_file_contains \
    "rust/scripts/sc12_contract_gate.sh" \
    'target/sc12_contract_gate/work/sc12_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc12_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/scripts/sc13_contract_gate.sh" \
    'semantic_annotation_sc13_contract'
  assert_file_contains \
    "rust/scripts/sc13_contract_gate.sh" \
    'target/sc13_contract_gate/work/sc13_semantic_differential_report.json'
  assert_file_contains \
    "rust/scripts/sc13_contract_gate.sh" \
    '(.total_cases > 0) and'

  assert_file_contains \
    "rust/test_data/semantic_annotation/sc01_contract.json" \
    '"generated_parser": "expected_fail"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc02_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc03_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc04_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc05_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc06_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc07_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc08_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc09_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc10_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc11_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc12_contract.json" \
    '"generated_parser": "pass"'
  assert_file_contains \
    "rust/test_data/semantic_annotation/sc13_contract.json" \
    '"generated_parser": "pass"'
}

audit_sota_json_consumption_surface() {
  note "auditing aggregate SOTA summary.json consumption surface"

  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/regex_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_combined_telemetry_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sota_summary_json="$sota_state_dir/summary.json"'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.counts.required_failures'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.primary_unmet_closure_criterion'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_failure_context_contract_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_failure_context_contract_summary_txt'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_failure_context_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_roundtrip_contract_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_roundtrip_contract_summary_txt'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.sv_roundtrip_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.proof_surfaces.failure_context_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.proof_surfaces.roundtrip_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status.systemverilog.proof_surfaces.parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.failure_context_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.systemverilog.proof_surfaces.roundtrip_contract_summary_json'

  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'sota_summary_json="$sota_state_dir/summary.json"'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.counts.required_failures'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.primary_unmet_closure_criterion'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.primary_unmet_closure_criterion // "<none>"'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.proof_surfaces.stimuli_parseability_report_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.proof_surfaces.dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status.regex.proof_surfaces.formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.regex.proof_surfaces.family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.regex.primary_unmet_detail_criterion // "<none>"'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.regex.proof_surfaces.formal_exhaustive_closure_summary_json'

  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    'sota_summary_json="$sota_state_dir/summary.json"'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.proof_surfaces.summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.counts.required_failures'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status.vhdl.primary_unmet_closure_criterion'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status.vhdl.proof_surfaces.quality_parseability_report_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.vhdl.proof_surfaces.family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status.vhdl.proof_surfaces.formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    '.family_status_contract.vhdl.proof_surfaces.formal_exhaustive_closure_summary_json'
}

audit_sota_nested_family_emission_surface() {
  note "auditing nested family SOTA summary.json emission surface"

  assert_tracked "rust/scripts/sota_exit_gate.sh"

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'systemverilog: family_status_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'parser_aggregate_summary_json: maybe_path($sv_family_status_systemverilog_parser_aggregate_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($sv_family_status_systemverilog_formal_exhaustive_closure_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'failure_context_contract_summary_json: maybe_path($sv_failure_context_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'roundtrip_contract_summary_json: maybe_path($sv_roundtrip_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'systemverilog: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'parser_aggregate_summary_json: maybe_path($sv_family_status_contract_systemverilog_parser_aggregate_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'semantic_scope_contract_summary_json: maybe_path($sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($sv_family_status_contract_systemverilog_formal_exhaustive_closure_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'failure_context_contract_summary_json: maybe_path($sv_failure_context_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'roundtrip_contract_summary_json: maybe_path($sv_roundtrip_contract_summary_json)'

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'regex: family_status_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'stimuli_parseability_report_json: maybe_path($regex_family_stimuli_parseability_report_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'stimuli_parseability_counterexample_triage_json: maybe_path($regex_family_stimuli_parseability_counterexample_triage_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'dual_run_summary_json: maybe_path($regex_family_dual_run_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_state_dir: maybe_path($regex_family_status_regex_formal_exhaustive_closure_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($regex_family_status_regex_formal_exhaustive_closure_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'regex: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'family_contract_summary_json: maybe_path($regex_family_status_contract_regex_family_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_state_dir: maybe_path($regex_family_status_contract_regex_formal_exhaustive_closure_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($regex_family_status_contract_regex_formal_exhaustive_closure_summary_json)'

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'vhdl: family_status_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'quality_parseability_report_json: maybe_path($vhdl_family_quality_parseability_report_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($vhdl_family_status_vhdl_formal_exhaustive_closure_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'vhdl: family_status_contract_entry('
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'family_contract_summary_json: maybe_path($vhdl_family_status_contract_vhdl_family_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'formal_exhaustive_closure_summary_json: maybe_path($vhdl_family_status_contract_vhdl_formal_exhaustive_closure_summary_json)'
}

audit_combined_telemetry_nested_provenance_surface() {
  note "auditing combined telemetry nested provenance emission surface"

  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/regex_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_combined_telemetry_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'state_dir: $sv_failure_context_contract_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'failure_context_contract_summary_json: $sv_failure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'parser_aggregate_summary_json: $sv_family_status_systemverilog_parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'formal_exhaustive_closure_summary_json: $sv_family_status_systemverilog_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'roundtrip_contract_summary_json: $sv_roundtrip_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'parser_aggregate_summary_json: $sv_family_status_contract_systemverilog_parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'semantic_scope_contract_summary_json: $sv_family_status_contract_systemverilog_semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'formal_exhaustive_closure_summary_json: $sv_family_status_contract_systemverilog_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'state_dir: $sv_roundtrip_contract_state_dir'

  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'dual_run_summary_json: $regex_family_dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_report_json: $regex_family_stimuli_parseability_report_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_counterexample_triage_json: $regex_family_stimuli_parseability_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_parser_rejections_zero: $regex_family_status_regex_stimuli_parseability_parser_rejections_zero'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'stimuli_parseability_counterexample_primary_parser_error: $regex_family_stimuli_parseability_counterexample_primary_parser_error'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'summary_json: $regex_family_status_contract_regex_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'state_dir: $regex_family_status_regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'formal_exhaustive_closure_summary_json: $regex_family_status_regex_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'state_dir: $regex_family_status_contract_regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_combined_telemetry_contract_gate.sh" \
    'summary_json: $regex_family_status_contract_regex_formal_exhaustive_closure_summary_json'

  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    'quality_parseability_report_json: $vhdl_family_quality_parseability_report_json'
  assert_file_contains \
    "rust/scripts/vhdl_combined_telemetry_contract_gate.sh" \
    'summary_json: $vhdl_family_status_contract_vhdl_family_contract_summary_json'
}

audit_sv_auxiliary_contract_surface() {
  note "auditing SV auxiliary-contract summary surface"

  assert_tracked "rust/scripts/sota_exit_gate.sh"
  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/sv_failure_context_contract_gate.sh"
  assert_tracked "rust/scripts/sv_roundtrip_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_failure_context_contract_state_dir: maybe_path($sv_failure_context_contract_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_failure_context_contract_summary_txt: maybe_path($sv_failure_context_contract_summary_txt)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_failure_context_contract_summary_json: maybe_path($sv_failure_context_contract_summary_json)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_roundtrip_contract_state_dir: maybe_path($sv_roundtrip_contract_state_dir)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_roundtrip_contract_summary_txt: maybe_path($sv_roundtrip_contract_summary_txt)'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_roundtrip_contract_summary_json: maybe_path($sv_roundtrip_contract_summary_json)'

  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_failure_context_contract_summary_json: $sv_failure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_roundtrip_contract_summary_json: $sv_roundtrip_summary_json'

  assert_file_contains \
    "rust/scripts/sv_failure_context_contract_gate.sh" \
    'systemverilog_generation_counterexample_triage_json: $systemverilog_generation_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_failure_context_contract_gate.sh" \
    'systemverilog_preprocessor_counterexample_triage_json: $systemverilog_preprocessor_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_roundtrip_contract_gate.sh" \
    'systemverilog_roundtrip_initial_targets: $systemverilog_roundtrip_initial_targets'
  assert_file_contains \
    "rust/scripts/sv_roundtrip_contract_gate.sh" \
    'systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches: $systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches'
}

audit_family_layer_provenance_surface() {
  note "auditing family-layer provenance emission surface"

  assert_tracked "rust/scripts/sv_parser_family_status_gate.sh"
  assert_tracked "rust/scripts/sv_parser_family_status_contract_gate.sh"
  assert_tracked "rust/scripts/regex_parser_family_status_gate.sh"
  assert_tracked "rust/scripts/regex_parser_family_status_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_parser_family_status_gate.sh"
  assert_tracked "rust/scripts/vhdl_parser_family_status_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'parser_aggregate_summary_json: $sv_parser_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'semantic_scope_contract_summary_json: $sv_semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_SKIP_FAMILY_STATUS=1'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_summary_json: $sv_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'aggregate_summary_json: $svpp_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'parser_aggregate_summary_json: $systemverilog_parser_aggregate_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'semantic_scope_contract_summary_json: $systemverilog_semantic_scope_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'formal_exhaustive_closure_summary_json: $systemverilog_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'aggregate_summary_json: $systemverilog_preprocessor_aggregate_summary_json'

  assert_file_contains \
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'family_contract_state_dir: $regex_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'family_contract_summary_json: $regex_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_state_dir: $regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_summary_json: $regex_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'state_dir: $regex_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'summary_json: $regex_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'state_dir: $regex_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_status_contract_gate.sh" \
    'summary_json: $regex_formal_exhaustive_closure_summary_json'

  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'family_contract_state_dir: $vhdl_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'family_contract_summary_json: $vhdl_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_state_dir: $vhdl_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_summary_json: $vhdl_formal_exhaustive_closure_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'state_dir: $vhdl_family_contract_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'summary_json: $vhdl_family_contract_summary_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'state_dir: $vhdl_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_status_contract_gate.sh" \
    'summary_json: $vhdl_formal_exhaustive_closure_summary_json'
}

audit_family_summary_identity_surface() {
  note "auditing family-sidecar summary identity surface"

  for repo_file in \
    rust/scripts/sv_parser_aggregate_contract_gate.sh \
    rust/scripts/sv_preprocessor_aggregate_contract_gate.sh \
    rust/scripts/sv_formal_exhaustive_closure_gate.sh \
    rust/scripts/regex_parser_family_contract_gate.sh \
    rust/scripts/regex_broader_corpus_proof_gate.sh \
    rust/scripts/regex_formal_exhaustive_closure_gate.sh \
    rust/scripts/vhdl_parser_family_contract_gate.sh \
    rust/scripts/vhdl_formal_exhaustive_closure_gate.sh \
    rust/scripts/sv_parser_family_status_gate.sh \
    rust/scripts/regex_parser_family_status_gate.sh \
    rust/scripts/vhdl_parser_family_status_gate.sh \
    rust/scripts/sv_parser_family_status_contract_gate.sh \
    rust/scripts/regex_parser_family_status_contract_gate.sh \
    rust/scripts/vhdl_parser_family_status_contract_gate.sh; do
    assert_tracked "$repo_file"
    assert_file_contains "$repo_file" 'SUMMARY_JSON="$STATE_DIR/summary.json"'
    assert_file_contains "$repo_file" 'echo "summary_json: $SUMMARY_JSON"'
    assert_file_contains "$repo_file" '--arg summary_json "$SUMMARY_JSON"'
    assert_file_contains "$repo_file" 'state_dir: $state_dir'
    assert_file_contains "$repo_file" 'summary_txt: $summary_txt'
    assert_file_contains "$repo_file" 'summary_json: $summary_json'
  done
}

audit_family_contract_proof_surface() {
  note "auditing family-contract proof-surface emission surface"

  assert_tracked "rust/scripts/regex_parser_family_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_parser_family_contract_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/regex_family_stimuli_contract.json"

  assert_file_contains \
    "rust/test_data/grammar_quality/regex_family_stimuli_contract.json" \
    '"grammar_name": "regex"'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_family_stimuli_contract.json" \
    '"require_parseability": true'

  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'STIMULI_CONTRACT_FILE="${PGEN_REGEX_FAMILY_CONTRACT_STIMULI_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/regex_family_stimuli_contract.json}"'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'require_file "$STIMULI_CONTRACT_FILE"'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_CONTRACT="$STIMULI_CONTRACT_FILE"'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="$STIMULI_TARGET_MAX_ATTEMPTS"'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'frontend_state_dir: $frontend_state_dir'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_contract_file: $stimuli_contract_file'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_target_max_attempts: $STIMULI_TARGET_MAX_ATTEMPTS'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'dual_run_summary_json: $dual_run_summary_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_summary_csv: $stimuli_summary_csv'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_parseability_report_json: $stimuli_regex_parseability_report_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_parseability_counterexample_triage_json: $stimuli_regex_parseability_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_regex_parseability_parser_rejections_total: $stimuli_regex_parseability_parser_rejections_total'
  assert_file_contains \
    "rust/scripts/regex_parser_family_contract_gate.sh" \
    'stimuli_regex_parseability_counterexample_primary_parser_error: $stimuli_regex_parseability_counterexample_primary_parser_error'

  assert_file_contains \
    "rust/scripts/vhdl_parser_family_contract_gate.sh" \
    'quality_parseability_report_json: $quality_parseability_report_json'
  assert_file_contains \
    "rust/scripts/vhdl_parser_family_contract_gate.sh" \
    'strict_promotion_report_json: $strict_promotion_report_json'
}

audit_regex_corpus_bundle_surface() {
  note "auditing regex corpus bundle surface"

  assert_tracked "regex_corpus_bundle/README.md"
  assert_tracked "regex_corpus_bundle/docs/regex_corpus_plan.md"
  assert_tracked "regex_corpus_bundle/manifests/upstreams.lock.json"
  assert_tracked "regex_corpus_bundle/manifests/licenses.json"
  assert_tracked "regex_corpus_bundle/schemas/regex_case.schema.json"
  assert_tracked "regex_corpus_bundle/scripts/fetch_regex_corpora.py"
  assert_tracked "regex_corpus_bundle/corpus/pcre2/invalid/README.md"
  assert_tracked "regex_corpus_bundle/corpus/pcre2/quarantine/README.md"
  assert_tracked "regex_corpus_bundle/oracle/pcre2/README.md"
  assert_tracked "rust/scripts/regex_corpus_bundle_contract_gate.sh"

  assert_file_contains \
    "rust/Makefile" \
    'regex_corpus_bundle_contract_gate - Validate the tracked PCRE2-first regex corpus acquisition bundle contract'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_corpus_bundle_contract_gate.sh'

  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'BUNDLE_DIR="$ROOT_DIR/regex_corpus_bundle"'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'LOCKFILE="$BUNDLE_DIR/manifests/upstreams.lock.json"'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'SCHEMA_FILE="$BUNDLE_DIR/schemas/regex_case.schema.json"'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'run_logged "fetch_regex_corpora_help" python3 "$FETCH_SCRIPT" --help'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'status_effect: does_not_reopen_closed_regex_family_row'
  assert_file_contains \
    "rust/scripts/regex_corpus_bundle_contract_gate.sh" \
    'maintained PCRE2-first acquisition/inventory starter for future regex hardening'

  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'Repo-ready starter bundle for a **PCRE2-first** regex corpus pipeline.'
  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'make -C rust regex_corpus_bundle_contract_gate'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '1. **Canonical syntax and behavior source:** PCRE2 upstream `testdata/testinput*` and related files.'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '2. **Secondary PCRE2-relevant source:** PHP `ext/pcre/tests`, because PHP uses PCRE2 but wraps patterns in PHP-specific delimiters and modifiers.'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '3. **Non-goal for phase 1:** treating non-PCRE2 engines as syntax ground truth.'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    'make -C rust regex_corpus_bundle_contract_gate'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '### Regex External Corpus Hardening'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`regex_corpus_bundle/`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust regex_corpus_bundle_contract_gate'
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`regex_corpus_bundle/` is the canonical PCRE2-first starter for widening regex evidence'
  # LIVE-MEANS-LIVE.1c2: the LIVE_ACHIEVEMENT_STATUS.md arm is RETIRED (the file is deleted by
  # `.1c3`). It was 1 arm of 21 here; the same feature stays asserted on PGEN_USER_GUIDE.md ×3, the
  # roadmap, RUST_CODEBASE_ANALYSIS.md and README.md — measured, not assumed, before dropping it.
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`regex_corpus_bundle/`'
  assert_file_contains \
    "README.md" \
    '`regex_corpus_bundle/`: PCRE2-first regex corpus acquisition/inventory starter for future regex hardening'
}

audit_regex_pcre2_compile_oracle_surface() {
  note "auditing regex PCRE2 compile-oracle surface"

  assert_tracked "regex_corpus_bundle/scripts/normalize_pcre2_compile_oracle.py"
  assert_tracked "rust/scripts/regex_pcre2_compile_oracle_gate.sh"
  assert_tracked "rust/src/bin/regex_corpus_probe.rs"
  assert_tracked "rust/src/regex_compile_validation.rs"
  assert_tracked "rust/test_data/grammar_quality/regex_pcre2_compile_oracle_lightweight_v0.env"

  assert_file_contains \
    "rust/Makefile" \
    'regex_pcre2_compile_oracle_gate - Normalize canonical PCRE2 compile-oracle cases from testinput2/testoutput2 and enforce tracked mismatch ceilings'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_pcre2_compile_oracle_gate.sh'

  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'scripts/normalize_pcre2_compile_oracle.py'
  assert_file_contains \
    "regex_corpus_bundle/README.md" \
    'make -C rust regex_pcre2_compile_oracle_gate'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    '`normalize_pcre2_compile_oracle.py`'
  assert_file_contains \
    "regex_corpus_bundle/docs/regex_corpus_plan.md" \
    'make -C rust regex_pcre2_compile_oracle_gate'

  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'NORMALIZER="$BUNDLE_DIR/scripts/normalize_pcre2_compile_oracle.py"'
  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'BASELINE_ENV="${PGEN_REGEX_PCRE2_COMPILE_ORACLE_BASELINE_ENV:-$RUST_DIR/test_data/grammar_quality/regex_pcre2_compile_oracle_lightweight_v0.env}"'
  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'expected_parse_ok_total: $expected_parse_ok_total'
  assert_file_contains \
    "rust/scripts/regex_pcre2_compile_oracle_gate.sh" \
    'false_accept_total: $false_accept_total'
  assert_file_contains \
    "rust/src/regex_compile_validation.rs" \
    'pub fn validate_regex_compile_contract(input: &str) -> Result<(), RegexCompileValidationError> {'
  assert_file_contains \
    "rust/src/parser_registry.rs" \
    'validate_regex_compile_contract(&owned_sample).map_err(|err| err.message)'
  assert_file_contains \
    "rust/src/embedding_api.rs" \
    'validate_regex_compile_contract(&owned_input)'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    '`make -C rust regex_pcre2_compile_oracle_gate`'
  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'the compile-oracle gate is the first external-corpus lane that actually measures expected compile outcomes against PCRE2 source truth'
  # LIVE-MEANS-LIVE.1c2: retired tracker arm (1 of 20 here; guide ×2, roadmap,
  # RUST_CODEBASE_ANALYSIS.md ×2 and README.md still assert this feature).
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`make -C rust regex_pcre2_compile_oracle_gate` consumes the new normalizer `regex_corpus_bundle/scripts/normalize_pcre2_compile_oracle.py`'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`regex_pcre2_compile_oracle_gate` for compile-truth comparison against pinned PCRE2 source truth'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`rust/src/regex_compile_validation.rs`'
  assert_file_contains \
    "README.md" \
    '`make -C rust regex_pcre2_compile_oracle_gate`'
}

audit_regex_formal_exhaustive_closure_surface() {
  note "auditing regex formal exhaustive-closure surface"

  assert_tracked "rust/scripts/regex_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/regex_broader_corpus_proof_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/regex_formal_exhaustive_closure_contract.json"
  assert_tracked "rust/test_data/grammar_quality/regex_broader_corpus_v0.json"

  assert_file_contains \
    "rust/Makefile" \
    'regex_broader_corpus_proof_gate - Run deterministic broader regex corpus proof over the checked-in regex stress corpus'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_broader_corpus_proof_gate.sh'
  assert_file_contains \
    "rust/Makefile" \
    'regex_formal_exhaustive_closure_gate - Compute the explicit regex exhaustive-closure proof surface status from the family sidecar'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/regex_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/regex_broader_corpus_v0.json" \
    '"source_file": "rust/test_data/regex/stress_tests.json"'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_broader_corpus_v0.json" \
    '"expected_case_count": 44'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_broader_corpus_v0.json" \
    '"expected_parser_type": "regex"'

  assert_file_contains \
    "rust/test_data/grammar_quality/regex_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "broader_corpus_backed_proof_surface"'
  assert_file_contains \
    "rust/test_data/grammar_quality/regex_formal_exhaustive_closure_contract.json" \
    '"required_surface_missing_detail": "Regex still lacks a checked-in broader corpus-backed proof surface'

  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'MANIFEST_FILE="${PGEN_REGEX_BROADER_CORPUS_PROOF_MANIFEST:-$RUST_DIR/test_data/grammar_quality/regex_broader_corpus_v0.json}"'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'cargo build --features generated_parsers --bin parseability_probe'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    '"$PARSE_PROBE_BIN" --supports regex'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'parse_fail_total'
  assert_file_contains \
    "rust/scripts/regex_broader_corpus_proof_gate.sh" \
    'primary_parse_failure_case'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'REGEX_BROADER_CORPUS_PROOF_GATE="$RUST_DIR/scripts/regex_broader_corpus_proof_gate.sh"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'run_logged "regex_broader_corpus_proof_gate"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'regex_broader_corpus_backed_proof_parse_fail_total'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'REGEX_FAMILY_CONTRACT_GATE="$RUST_DIR/scripts/regex_parser_family_contract_gate.sh"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'EXISTING_BROADER_CORPUS_PROOF_STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_BROADER_CORPUS_PROOF_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'broader_corpus_backed_proof_surface_present'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'regex_unmet+=("${required_surface_key}=missing")'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'regex_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/regex_formal_exhaustive_closure_gate.sh" \
    'broader_corpus_backed_proof_summary_json'
}

audit_vhdl_formal_exhaustive_closure_surface() {
  note "auditing VHDL formal exhaustive-closure surface"

  assert_tracked "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/vhdl_external_corpus_triage_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/vhdl_formal_exhaustive_closure_contract.json"
  assert_tracked "rust/test_data/grammar_quality/vhdl_external_corpus_triage_v0.json"

  assert_file_contains \
    "rust/Makefile" \
    'vhdl_formal_exhaustive_closure_gate - Compute the explicit VHDL exhaustive-closure proof surface status from family and external-corpus sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/vhdl_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/vhdl_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "external_corpus_backed_proof_surface"'
  assert_file_contains \
    "rust/test_data/grammar_quality/vhdl_formal_exhaustive_closure_contract.json" \
    '"required_surface_missing_detail": "VHDL still lacks a checked-in external corpus-backed proof surface'

  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'VHDL_EXTERNAL_CORPUS_TRIAGE_GATE="$RUST_DIR/scripts/vhdl_external_corpus_triage_gate.sh"'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'run_logged "vhdl_external_corpus_triage_gate"'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'vhdl_external_corpus_backed_proof_parse_fail_total'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'vhdl_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/vhdl_formal_exhaustive_closure_gate.sh" \
    'external_corpus_backed_proof_summary_json'
}

audit_sv_formal_exhaustive_closure_surface() {
  note "auditing SV formal exhaustive-closure surface"

  assert_tracked "rust/scripts/sv_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/sv_external_corpus_triage_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_formal_exhaustive_closure_contract.json"

  assert_file_contains \
    "rust/Makefile" \
    'sv_formal_exhaustive_closure_gate - Compute the explicit SV exhaustive-closure proof surface status from family-status and external-corpus sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/sv_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "external_corpus_backed_proof_surface"'
  # CI-PARITY-GATE-ROT.1 (2026-07-27): was pinned to the prose "SystemVerilog still LACKS an
  # explicit checked-in external corpus-backed proof surface sidecar" — a description of an
  # EARLIER state of the world. The sidecar now exists and the contract's requirement moved on to
  # "must MATCH the live triage gate output exactly". ⛔ Deliberately re-pinned to the stable
  # SUBJECT of the requirement (the sidecar key) rather than to the full sentence: pinning
  # narrative prose is what made this rot, and the key is what the gate actually consumes.
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_formal_exhaustive_closure_contract.json" \
    'expected_proof_surface_sidecar'

  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'SV_EXTERNAL_CORPUS_TRIAGE_GATE="$RUST_DIR/scripts/sv_external_corpus_triage_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'SV_FAMILY_STATUS_GATE="$RUST_DIR/scripts/sv_parser_family_status_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'run_logged "sv_external_corpus_triage_gate"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="${PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'systemverilog_external_corpus_backed_proof_parse_fail_total'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'systemverilog_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/sv_formal_exhaustive_closure_gate.sh" \
    'systemverilog_external_corpus_backed_proof_summary_json'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust SHELL=/bin/bash sv_formal_exhaustive_closure_gate'
  # LIVE-MEANS-LIVE.1c2: retired tracker arm (1 of 15 here; guide, roadmap and
  # RUST_CODEBASE_ANALYSIS.md still assert this feature).
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`sv_formal_exhaustive_closure_gate` now makes the missing-vs-present SystemVerilog external-corpus proof surface explicit'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`sv_formal_exhaustive_closure_gate` when the task is SystemVerilog external-corpus proof normalization'
}

audit_sv_preprocessor_formal_exhaustive_closure_surface() {
  note "auditing SV preprocessor formal exhaustive-closure surface"

  assert_tracked "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_syntax_closure_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_reachability_closure_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_preprocessor_formal_exhaustive_closure_contract.json"
  assert_tracked "rust/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json"

  assert_file_contains \
    "rust/Makefile" \
    'sv_preprocessor_zero_plausible_gap_proof_gate - Prove the SV-preprocessor helper-only syntax-unreachable whitelist over the retained aggregate/reachability sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'sv_preprocessor_formal_exhaustive_closure_gate - Compute the explicit SV-preprocessor exhaustive-closure proof surface status from syntax, aggregate, and reachability sidecars'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh'
  assert_file_contains \
    "rust/Makefile" \
    'cd $(RUST_DIR) && ./scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh'

  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_formal_exhaustive_closure_contract.json" \
    '"required_surface_key": "zero_plausible_grammar_level_gap_proof_surface"'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_formal_exhaustive_closure_contract.json" \
    '"required_surface_missing_detail": "SystemVerilog preprocessor still lacks an explicit grammar-level exhaustive proof surface'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json" \
    '"allowed_unreachable_rules":'
  assert_file_contains \
    "rust/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json" \
    '"helper_only_whitelist_detail": "SystemVerilog preprocessor zero-plausible-gap proof requires the syntax-unreachable surface'

  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_syntax_closure_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'AGGREGATE_CONTRACT_GATE="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'REACHABILITY_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_reachability_closure_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'ZERO_GAP_PROOF_GATE="$RUST_DIR/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_SYNTAX_CLOSURE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_AGGREGATE_CONTRACT_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_AGGREGATE_CONTRACT_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_REACHABILITY_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_REACHABILITY_CLOSURE_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'EXISTING_ZERO_GAP_PROOF_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_ZERO_PLAUSIBLE_GAP_PROOF_STATE_DIR:-}"'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'aggregate_contract_summary_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'reachability_closure_summary_txt'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'zero_gap_proof_summary_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'zero_plausible_grammar_level_gap_proof_surface'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh" \
    'systemverilog_preprocessor_formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh" \
    'helper_only_unreachable_surface_green'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh" \
    'zero_plausible_grammar_level_gap_proof_surface'

  assert_file_contains \
    "PGEN_USER_GUIDE.md" \
    'make -C rust SHELL=/bin/bash sv_preprocessor_formal_exhaustive_closure_gate'
  # LIVE-MEANS-LIVE.1c2: retired tracker arm (1 of 37 here; guide, roadmap and
  # RUST_CODEBASE_ANALYSIS.md still assert this feature).
  assert_file_contains \
    "docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md" \
    '`sv_preprocessor_formal_exhaustive_closure_gate` now makes the missing-vs-present SystemVerilog-preprocessor grammar-level proof surface explicit'
  assert_file_contains \
    "docs/reference/RUST_CODEBASE_ANALYSIS.md" \
    '`sv_preprocessor_formal_exhaustive_closure_gate` when the task is SystemVerilog-preprocessor formal-closure proof normalization'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'formal_exhaustive_closure_surface_green'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_gate.sh" \
    'systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'systemverilog_preprocessor_formal_exhaustive_closure_gate'
  assert_file_contains \
    "rust/scripts/sv_parser_family_status_contract_gate.sh" \
    'formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_family_status_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sota_exit_gate.sh" \
    'sv_family_status_contract_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_family_status_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
  assert_file_contains \
    "rust/scripts/sv_combined_telemetry_contract_gate.sh" \
    'sv_family_status_contract_systemverilog_preprocessor_formal_exhaustive_closure_state_dir'
}

audit_sv_aggregate_contract_proof_surface() {
  note "auditing SV aggregate-contract proof-surface emission surface"

  assert_tracked "rust/scripts/sv_parser_aggregate_contract_gate.sh"
  assert_tracked "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh"

  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'generation_report_json: $generation_report_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'shadow_report_json: $shadow_report_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'generation_counterexample_triage_json: $generation_counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'replay_gap_target_triage_json: $replay_gap_target_triage_json'
  assert_file_contains \
    "rust/scripts/sv_parser_aggregate_contract_gate.sh" \
    'source_gap_json: $source_gap_json'

  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'quality_state_dir: $quality_state_dir'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'parseability_report_json: $parseability_report_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'counterexample_triage_json: $counterexample_triage_json'
  assert_file_contains \
    "rust/scripts/sv_preprocessor_aggregate_contract_gate.sh" \
    'gap_stage3_json: $gap_stage3_json'
}

audit_generated_clippy_correctness_surface() {
  # GENERATED-LINT-CORRECTNESS.3.
  #
  # ⛔ This is a SURFACE audit, not a run — and that is a measured decision, not a shortcut.
  # `copy_tracked_worktree` exports `git ls-files` only, and `generated/` is untracked by
  # repository policy, so the export dir has NO generated parsers. `rust/build.rs` sets each
  # `has_generated_<n>_parser` cfg only when the artifact `is_file()`, so a lint run in there
  # would compile ZERO generated parsers and report 0 findings — a vacuous green, which is the
  # exact failure mode this tree exists to remove. The gate itself refuses in that situation
  # (exit 2), so running it here would fail the whole local-parity gate by design.
  # The real run is `make -C rust SHELL=/bin/bash generated_clippy_correctness_gate` where the
  # artifacts exist, the commit workflow's strict generated stage, and the hosted workflow
  # (which regenerates the parsers first).
  note "auditing generated-parser clippy correctness surface"

  assert_tracked ".github/workflows/generated-clippy-correctness-gate.yml"
  assert_tracked "rust/scripts/generated_clippy_correctness_gate.sh"
  assert_tracked "rust/scripts/clippy_on_rust_change.sh"
  assert_tracked "rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json"

  # The contract must still pin the roster BY NAME (the anti-narrowing mechanism) and must still
  # carry both anchor lints — the two that produced the 291 errors this tree opened on.
  assert_file_contains \
    "rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json" \
    '"correctness_roster"'
  assert_file_contains \
    "rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json" \
    '"eq_op"'
  assert_file_contains \
    "rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json" \
    '"overly_complex_bool_expr"'

  # The two anti-vacuity checks must still be wired.
  assert_file_contains \
    "rust/scripts/generated_clippy_correctness_gate.sh" \
    'build-script-executed'
  assert_file_contains \
    "rust/scripts/generated_clippy_correctness_gate.sh" \
    'cannot lint generated parsers that are not on disk'

  # The generated stage must remain STRICT BY DEFAULT — a silent flip back to 0 is precisely
  # how the 291 accumulated unnoticed.
  assert_file_contains \
    "rust/scripts/clippy_on_rust_change.sh" \
    'GENERATED_STRICT="${PGEN_CLIPPY_GENERATED_STRICT:-1}"'
  assert_file_not_contains \
    "rust/scripts/clippy_on_rust_change.sh" \
    'GENERATED_STRICT="${PGEN_CLIPPY_GENERATED_STRICT:-0}"'

  # The Makefile lane must exist so the gate has a repo-standard entry point.
  assert_file_contains "rust/Makefile" "generated_clippy_correctness_gate:"
  assert_file_contains "rust/Makefile" "generated_clippy_correctness_policy:"

  # The roster pin is only meaningful while the enforcer recognizes codegen-emission evidence.
  assert_file_contains \
    "scripts/check_diagnosis_evidence.sh" \
    'GENERATED-CLIPPY-CORRECTNESS:'
}

audit_ast_dump_contract_surface() {
  note "auditing AST dump contract surface"

  # QUANT-PLUS-ITER.4 (2026-07-27). ⭐ THIS AUDIT EXISTS BECAUSE THE GATE IT WATCHES ROTTED FOR
  # FOUR SESSIONS WITH NOBODY NOTICING. `make -C rust ast_dump_contract_gate` was RED from
  # `7219547c` (the `@entry: true` mandate) until this leaf, and the reason it stayed red is that
  # it is referenced by NO aggregate and NO CI workflow — measured: outside its own Makefile
  # recipe and the help text, the string `ast_dump_contract_gate` appeared nowhere.
  # *A check that nothing INVOKES is indistinguishable from a check that does not exist.*
  # Fixing the gate without giving something a reason to run it would simply schedule the next rot.
  assert_tracked "rust/scripts/ast_dump_contract_gate.sh"
  assert_file_contains "rust/Makefile" 'ast_dump_contract_gate:'
  assert_file_contains "rust/Makefile" 'cd $(RUST_DIR) && ./scripts/ast_dump_contract_gate.sh'

  # Pin the EXACT regression that happened: both raw-AST fixtures must declare the mandatory entry
  # rule. These are JSON heredocs, not `.ebnf` files, so the `.2` step-B migration sweep was
  # structurally blind to them — an `.ebnf`-shaped sweep cannot see a grammar expressed as raw AST.
  # ⛔ COUNT, not presence: there are TWO fixtures (`mini` and `mini_large`) and BOTH must declare
  # the mandatory entry. A presence check passed with one of them deleted — caught by this leaf's
  # own RED arm, which is precisely why the RED arm exists.
  assert_file_contains_count "rust/scripts/ast_dump_contract_gate.sh" '["semantic_annotation", ["entry", "true"]]' 2
  # ... and it must be the CANONICAL annotation backend, not the bootstrap fallback the pipeline
  # itself refuses (non-canonical artifacts).
  assert_file_contains "rust/scripts/ast_dump_contract_gate.sh" \
    'run_logged_rust "build_ast_pipeline" cargo build --features generated_parsers --bin ast_pipeline'
  # ⚠️ UNCOMMENTED form — the whole-file assertion fired on this leaf's OWN comment explaining why
  # the fallback is refused. Second instance in one session of an audit reporting documentation
  # back to itself (the `ebnf_to_json.pl` Makefile comment was the first); the helper built for
  # that one applies unchanged here.
  assert_file_not_contains_uncommented "rust/scripts/ast_dump_contract_gate.sh" 'PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1'
}

audit_summary_json_emission_surface() {
  note "auditing top-level proof summary.json emission surface"

  assert_tracked "rust/scripts/sota_exit_gate.sh"
  assert_tracked "rust/scripts/sv_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/regex_combined_telemetry_contract_gate.sh"
  assert_tracked "rust/scripts/vhdl_combined_telemetry_contract_gate.sh"

  for repo_file in \
    rust/scripts/sv_failure_context_contract_gate.sh \
    rust/scripts/sv_roundtrip_contract_gate.sh \
    rust/scripts/sota_exit_gate.sh \
    rust/scripts/sv_combined_telemetry_contract_gate.sh \
    rust/scripts/regex_combined_telemetry_contract_gate.sh \
    rust/scripts/vhdl_combined_telemetry_contract_gate.sh; do
    assert_file_contains "$repo_file" 'SUMMARY_JSON="$STATE_DIR/summary.json"'
    assert_file_contains "$repo_file" 'echo "generated_at_utc: $generated_at_utc"'
    assert_file_contains "$repo_file" 'echo "summary_json: $SUMMARY_JSON"'
    assert_file_contains "$repo_file" '--arg generated_at_utc "$generated_at_utc"'
    assert_file_contains "$repo_file" '--arg summary_json "$SUMMARY_JSON"'
    assert_file_contains "$repo_file" 'summary_json: $summary_json'
  done
}

assert_workflow_command() {
  local workflow_file="$1"
  local expected="$2"
  grep -F "$expected" "$ROOT_DIR/$workflow_file" >/dev/null 2>&1 || \
    fail "workflow command drift detected in $workflow_file: expected '$expected'"
}

run_workflow() {
  local workflow_name="$1"
  local workflow_file="$2"
  local command_marker="$3"
  local command_line="$4"
  local log_file="$LOG_DIR/${workflow_name}.log"

  # CI-PARITY-GATE-ROT.3 — record the name BEFORE the filter test, so the roster is every workflow
  # this gate knows how to replay rather than only the selected ones. That is what makes
  # `assert_workflow_selection_was_real` able to tell a typo from a deliberate narrowing.
  WORKFLOW_ROSTER="$WORKFLOW_ROSTER $workflow_name"

  if ! is_selected "$workflow_name"; then
    note "skip $workflow_name (filtered)"
    return 0
  fi

  assert_workflow_command "$workflow_file" "$command_marker"
  WORKFLOWS_RUN_COUNT=$((WORKFLOWS_RUN_COUNT + 1))
  note "run $workflow_name"
  if (
    cd "$EXPORT_DIR"
    export CARGO_NET_OFFLINE="$CARGO_OFFLINE_RAW"
    eval "$command_line"
  ) >"$log_file" 2>&1; then
    note "ok $workflow_name ($log_file)"
  else
    tail -n 120 "$log_file" >&2 || true
    explain_missing_generated_failure "$log_file"
    fail "workflow local parity failed: $workflow_name (log: $log_file)"
  fi
}

assert_workflow_selection_was_real() {
  # CI-PARITY-GATE-ROT.3. ⭐⭐ THIS CLOSES A MEASURED VACUOUS GREEN.
  #
  # `is_selected` accepted ANY string, and a name matching nothing simply skipped all eleven
  # replays — after which `main` printed "all selected local workflow commands passed" and the
  # Makefile printed "✅ Local GitHub workflow parity gate passed", exit 0. Measured verbatim with
  # PGEN_CI_WORKFLOW_LOCAL_FILTER=typo-that-matches-nothing: eleven "skip … (filtered)" lines and a
  # green verdict. A typo in the filter therefore CERTIFIED PARITY WHILE REPLAYING NOTHING.
  #
  # That is this tree's own signature defect one level in: *a check that cannot run must SAY SO,
  # not return green*. The audits still ran, so the run was not entirely empty — which is exactly
  # what made it convincing, and exactly why the closing message must not speak for a phase that
  # did not execute.
  local item matched name
  local -a filter_items

  if [[ -n "$FILTER_RAW" ]]; then
    IFS=',' read -r -a filter_items <<<"$FILTER_RAW"
    for item in "${filter_items[@]}"; do
      matched=0
      for name in $WORKFLOW_ROSTER; do
        if [[ "$name" == "$item" ]]; then
          matched=1
          break
        fi
      done
      if [[ "$matched" -eq 0 ]]; then
        fail "unknown PGEN_CI_WORKFLOW_LOCAL_FILTER entry '$item': no such replayable workflow, so nothing ran for it. Known workflows:$WORKFLOW_ROSTER"
      fi
    done
  fi

  if [[ "$WORKFLOWS_RUN_COUNT" -eq 0 ]]; then
    fail "workflow phase replayed 0 workflows — refusing to report local parity. Known workflows:$WORKFLOW_ROSTER"
  fi
}

required_unguarded_generated_artifacts() {
  # CI-PARITY-GATE-ROT.3 — DERIVE, do not hand-list.
  #
  # `rust/src/lib.rs` includes generated parsers two different ways, and the difference is the
  # whole story:
  #   - `include!(env!("PGEN_<X>_PARSER_PATH_RESOLVED"))` sites sit behind a `has_generated_*` cfg
  #     that `rust/build.rs` sets only when the artifact `is_file()` — absence merely DISABLES them;
  #   - `include!("../../generated/<x>.rs")` LITERAL-path sites have no such cfg, so absence is a
  #     hard `error: couldn't read …` and the whole crate fails to compile.
  # Only the literal form can break the build, and grepping for exactly that form means this check
  # can never drift away from what `lib.rs` actually does.
  (cd "$ROOT_DIR" && grep -oE 'include!\("\.\./\.\./generated/[a-z_]+\.rs"\)' rust/src/lib.rs 2>/dev/null) |
    sed -E 's|.*(generated/[a-z_]+\.rs).*|\1|' | sort -u
}

prepare_generated_artifacts() {
  # CI-PARITY-GATE-ROT.3 — materialise `generated/` inside the export dir.
  #
  # ⛔ NOT a shortcut and NOT a copy from the developer's tree. This replays the repository's OWN
  # cold-clone bootstrap recipe — `rust/Makefile`'s `regex_parser_bootstrap` ("Bootstrap regex
  # parser from cold clone", which seeds `generated/ebnf.rs` when it is missing) followed by the
  # annotation pair and the per-grammar `focus_*` targets.
  #
  # ⭐ CI-PARITY-GATE-ROT.4 — IT NO LONGER SPELLS THAT SEQUENCE OUT. It calls
  # `rust/Makefile`'s `regenerate_generated_parsers`, which is now the ONE definition of the
  # recipe; the hosted side reaches the same target through the composite action
  # `.github/actions/regenerate-parsers`. Before `.4` the sequence was written out twice (here and
  # in `generated-clippy-correctness-gate.yml`) and wiring the ten further workflows that need it
  # would have produced twelve copies whose drift nothing could detect.
  # `audit_workflow_regeneration_surface` asserts both ends of that wiring.
  #
  # ⛔ Copying the developer's `generated/` into the export dir was REJECTED: it would make this
  # gate green against artifacts a fresh checkout does not have, which is the vacuity class this
  # tree exists to remove.
  # ⚠️ THE LOG IS DELIBERATELY BOUNDED TO ITS TAIL, AND THAT IS A MEASURED DECISION.
  # `rust/Makefile:93-94` runs the generator as `--generate-parser --debug --trace …`, so the
  # cold-clone sequence emits PGEN's own `[PGEN][LOW]`/`[HIGH]`/`[DBG]` trace for seven grammars.
  # Captured in full it measured **7.1 GB** for one preparation — fine on this 3.6 TB volume,
  # fatal on a hosted runner with ~14 GB free. `make` stops AT the failing step, so the tail is
  # exactly where a failure's evidence lives; `tail -c` also bounds memory to the window itself.
  # ⛔ The alternative — quietening the recipe — was rejected as out of scope: that recipe is
  # SHARED with the tracked hosted workflow, and changing what evidence it leaves is a different
  # change with a different owner.
  local log_file="$LOG_DIR/00-prepare-generated.log"
  local log_tail_bytes=4194304
  note "preparing generated/ inside the export dir (cold-clone bootstrap; log: $log_file, last ${log_tail_bytes}B)"
  if (
    cd "$EXPORT_DIR"
    export CARGO_NET_OFFLINE="$CARGO_OFFLINE_RAW"
    set -e
    make -C rust SHELL=/bin/bash "$REGENERATION_MAKE_TARGET"
  ) 2>&1 | tail -c "$log_tail_bytes" >"$log_file"; then
    note "ok prepare generated/ ($log_file)"
  else
    tail -n 60 "$log_file" >&2 || true
    fail "could not prepare generated/ in the export dir (log: $log_file); the workflow phase cannot be replayed against an unprepared tree"
  fi
}

preflight_generated_artifacts() {
  # CI-PARITY-GATE-ROT.3. ⭐⭐ THE MEASURED HEADLINE OF THIS LEAF.
  #
  # `copy_tracked_worktree` exports `git ls-files` output ONLY, and `generated/` has been untracked
  # since `0ed2b2ad` ("Slice 5: stop tracking generated/* in git", 2026-04-29). So the export dir
  # has no generated parsers at all, and the first replay that compiles the crate dies with
  #
  #   error: couldn't read `src/../../generated/return_annotation_parser.rs`: No such file or
  #   directory (os error 2)  --> src/lib.rs:72:9
  #
  # Measured over all eleven replays (`docs/tasks/artifacts/ci_parity_gate_rot/`): 3 PASS / 8 FAIL,
  # and every one of the eight resolves to that single cause. The three that pass are exactly the
  # three that never need a generated parser (`branch-protection-contract-gate` is shell+jq,
  # `mdbook-docs-gate` is mdbook, `fixed-point-gate` builds the bootstrap binary WITHOUT
  # `--features generated_parsers`).
  #
  # ⛔ Without this preflight the operator's evidence is eight identical `could not compile pgen`
  # cascades, which read as "the Rust code is broken" — the diagnosis is aimed at the wrong thing
  # entirely. A gate that cannot run must say WHY, and must not fail as though the subject under
  # test were the broken party.
  #
  # ⚠️ WARN-AND-CONTINUE, NOT REFUSE — and that is a CORRECTION, caught by a control arm.
  # The first cut of this function exited 2 up front. That would have broken a case which
  # currently WORKS: `PGEN_CI_WORKFLOW_LOCAL_FILTER=branch-protection-contract-gate` passes today
  # (measured, 56s), because that replay is shell+jq and needs no generated parser at all. Three of
  # the eleven replays are in that class. Refusing for all of them would have traded a bad
  # diagnosis for a lost capability — a gate must not fail runs it can genuinely complete. So the
  # missing state is announced loudly here, and the CAUSE is re-attached in `run_workflow`'s
  # failure path, which is where the operator is actually looking when it bites.
  local missing=""
  local artifact

  for artifact in $(required_unguarded_generated_artifacts); do
    if [[ ! -f "$EXPORT_DIR/$artifact" ]]; then
      missing="$missing $artifact"
    fi
  done

  if [[ -z "$missing" ]]; then
    note "preflight: generated/ artifacts present in the export dir"
    return 0
  fi

  if [[ "$PREPARE_NORMALIZED" == "true" ]]; then
    # ⚠️ AN HONEST COST, STATED RATHER THAN HIDDEN. Since the default flipped to `true`, EVERY run
    # pays the ~236 s preparation, because the export dir is `git ls-files` output and therefore
    # always lacks `generated/`. For a full gate run that is exactly right. For a NARROWED run it
    # can be waste: three replays (`branch-protection-contract-gate`, `mdbook-docs-gate`,
    # `fixed-point-gate`) never compile the crate and need nothing. The gate cannot decide this for
    # the operator — the replay roster is only complete once every `run_workflow` call site has been
    # reached, which is after they have run — so it says so instead of guessing.
    if [[ -n "$FILTER_RAW" ]]; then
      note "preflight: filtered run — if your selection does not compile the crate, skip this ~236s"
      note "preflight: preparation with PGEN_CI_WORKFLOW_LOCAL_PREPARE=0"
    fi
    prepare_generated_artifacts
    missing=""
    for artifact in $(required_unguarded_generated_artifacts); do
      if [[ ! -f "$EXPORT_DIR/$artifact" ]]; then
        missing="$missing $artifact"
      fi
    done
    if [[ -z "$missing" ]]; then
      note "preflight: generated/ artifacts present after preparation"
      return 0
    fi
    fail "preparation reported success but these artifacts are still absent from the export dir:$missing"
  fi

  note "preflight: ⚠️  export dir has NO generated parsers -- missing:$missing"
  note "preflight: any replay that compiles the crate WILL fail; re-run with PGEN_CI_WORKFLOW_LOCAL_PREPARE=1"
}

explain_missing_generated_failure() {
  # CI-PARITY-GATE-ROT.3 — attach the cause to the failure, not to a header the operator scrolled
  # past twenty audits ago. Only fires when the replay's own log carries the signature, so a
  # genuine gate failure is never mislabelled as an environment problem.
  local log_file="$1"
  grep -qE "couldn't read \`src/\.\./\.\./generated/|missing return annotation JSON at" "$log_file" 2>/dev/null || return 0

  echo "" >&2
  echo "  ^^^ this is NOT a defect in the Rust sources. The export dir is 'git ls-files' output" >&2
  echo "      ONLY, and generated/ has been untracked since 0ed2b2ad ('Slice 5: stop tracking" >&2
  echo "      generated/* in git', 2026-04-29). rust/src/lib.rs includes the two annotation" >&2
  echo "      parsers by LITERAL path with no has_generated_* cfg, so their absence is a hard" >&2
  echo "      rustc error rather than a disabled feature." >&2
  echo "      fix:  re-run with PGEN_CI_WORKFLOW_LOCAL_PREPARE=1 (replays the repository's own" >&2
  echo "            cold-clone bootstrap inside the export dir before the replays)." >&2
  echo "      note: 8 of the 11 replayed workflows hit this, and only" >&2
  echo "            .github/workflows/generated-clippy-correctness-gate.yml declares a" >&2
  echo "            regeneration step -- see docs/tasks/CI-PARITY-GATE-ROT.md leaves .3 and .4." >&2
}

main() {
  note "PGEN local CI workflow gate"
  note "run_dir: $RUN_DIR"
  note "filter: ${FILTER_RAW:-<all>}"
  note "cargo_offline: $CARGO_OFFLINE_RAW"
  note "keep_success_runs: $KEEP_RUNS_NORMALIZED"
  note "prepare_generated: $PREPARE_NORMALIZED"

  require_tool git
  require_tool cargo
  require_tool make
  require_tool mdbook
  require_tool perl
  require_tool jq

  copy_tracked_worktree
  audit_static_include_paths
  audit_markdown_repo_relative_paths
  audit_root_markdown_surface
  audit_top_level_docs_surface
  audit_contract_docs_surface
  audit_reference_docs_surface
  audit_docs_book_surface
  audit_active_docs_rehome_paths
  audit_workflow_surface
  audit_workflow_regeneration_surface
  audit_ebnf_frontend_conversion_surface
  audit_embedding_api_surface
  audit_rtl_frontend_generated_contract_surface
  audit_stimuli_cross_family_platform_surface
  audit_annotation_aggregate_contract_surface
  audit_annotation_semantic_contract_surface
  audit_sota_json_consumption_surface
  audit_sota_nested_family_emission_surface
  audit_combined_telemetry_nested_provenance_surface
  audit_sv_auxiliary_contract_surface
  audit_family_layer_provenance_surface
  audit_family_summary_identity_surface
  audit_family_contract_proof_surface
  audit_regex_corpus_bundle_surface
  audit_regex_pcre2_compile_oracle_surface
  audit_regex_formal_exhaustive_closure_surface
  audit_sv_formal_exhaustive_closure_surface
  audit_sv_preprocessor_formal_exhaustive_closure_surface
  audit_vhdl_formal_exhaustive_closure_surface
  audit_sv_aggregate_contract_proof_surface
  audit_generated_clippy_correctness_surface
  audit_ast_dump_contract_surface
  audit_summary_json_emission_surface

  # CI-PARITY-GATE-ROT.3 — the audit phase is now 32/32, but that is NOT "the gate completes".
  # Everything above reads files; everything below RUNS them, and the export dir has to be able to
  # build before a single replay is meaningful. Refuse here, once, with the real cause — rather
  # than eight identical rustc cascades that misdirect the diagnosis onto the Rust sources.
  # ⚠️ Ordering note: with PGEN_CI_WORKFLOW_LOCAL_PREPARE=1 this runs BEFORE the filter names are
  # validated (the roster is only complete once every run_workflow call site has been reached), so
  # a mistyped filter costs one preparation before `assert_workflow_selection_was_real` rejects it.
  # Deliberate: a second, source-grepped roster would be a parallel mechanism that can drift, and
  # the default (PREPARE=false) refuses immediately anyway.
  preflight_generated_artifacts

  run_workflow \
    "annotation-contract-gate" \
    ".github/workflows/annotation-contract-gate.yml" \
    "make -C rust SHELL=/bin/bash annotation_contract_gate" \
    "make -C rust SHELL=/bin/bash annotation_contract_gate"
  run_workflow \
    "annotation-nonbootstrap-e2e-gate" \
    ".github/workflows/annotation-nonbootstrap-e2e-gate.yml" \
    "make -C rust SHELL=/bin/bash annotation_nonbootstrap_e2e_gate" \
    "PGEN_STRICT_ANNOTATION_VALIDATION=1 make -C rust SHELL=/bin/bash annotation_nonbootstrap_e2e_gate"
  run_workflow \
    "branch-protection-contract-gate" \
    ".github/workflows/branch-protection-contract-gate.yml" \
    "make -C rust SHELL=/bin/bash branch_protection_contract_gate" \
    "make -C rust SHELL=/bin/bash branch_protection_contract_gate"
  run_workflow \
    "differential-regression-gate" \
    ".github/workflows/differential-regression-gate.yml" \
    "make -C rust SHELL=/bin/bash differential_regression_gate" \
    "make -C rust SHELL=/bin/bash differential_regression_gate"
  run_workflow \
    "ebnf-frontend-dual-run-diff" \
    ".github/workflows/ebnf-frontend-dual-run-diff.yml" \
    "make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff" \
    "make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff"
  run_workflow \
    "rtl-frontend-generated-contract-gate" \
    ".github/workflows/rtl-frontend-generated-contract-gate.yml" \
    "make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate" \
    "make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate"
  run_workflow \
    "stimuli-cross-family-platform-gate" \
    ".github/workflows/stimuli-cross-family-platform-gate.yml" \
    "make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate" \
    "make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate"
  run_workflow \
    "mdbook-docs-gate" \
    ".github/workflows/mdbook-docs-gate.yml" \
    "make -C rust SHELL=/bin/bash mdbook_docs_gate" \
    "make -C rust SHELL=/bin/bash mdbook_docs_gate"
  run_workflow \
    "fixed-point-gate" \
    ".github/workflows/fixed-point-gate.yml" \
    "make -C rust SHELL=/bin/bash fixed_point_gate" \
    "PGEN_STRICT_ANNOTATION_VALIDATION=1 make -C rust SHELL=/bin/bash fixed_point_gate"
  run_workflow \
    "performance-gate" \
    ".github/workflows/performance-gate.yml" \
    "make -C rust SHELL=/bin/bash performance_gate" \
    "make -C rust SHELL=/bin/bash performance_gate"
  run_workflow \
    "sota-exit-gate" \
    ".github/workflows/sota-exit-gate.yml" \
    "make -C rust SHELL=/bin/bash sota_exit_gate" \
    "PGEN_STRICT_ANNOTATION_VALIDATION=1 PGEN_SOTA_POLICY_FILE=$EXPORT_DIR/rust/config/sota_exit_policy.env PGEN_SOTA_RUN_EBNF_READINESS=1 PGEN_SOTA_REQUIRE_EBNF_STRICT=0 PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF=1 PGEN_SOTA_REQUIRE_EBNF_DUAL_RUN_STRICT=0 make -C rust SHELL=/bin/bash sota_exit_gate"

  assert_workflow_selection_was_real
  note "all selected local workflow commands passed ($WORKFLOWS_RUN_COUNT replayed)"
}

main "$@"
