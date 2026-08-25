#!/usr/bin/env bash
# rust/scripts/cold_clone_build_probe.sh — CI-PARITY-GATE-ROT.40 (b).
#
# ⛔ WHAT THIS PROVES: the crate compiles in the **COLD-CLONE ARTIFACT CONFIGURATION** — the state a
# fresh clone is in when `make -C rust regenerate_generated_parsers` runs its first build. In that
# state no `generated/*_parser.rs` family artifact exists yet, so `build.rs` sets **no**
# `has_generated_*` cfg, while the FEATURE set is already the full bootstrap one
# (`generated_parsers ebnf_dual_run`).
#
# ⛔ THE DEFECT CLASS IT CATCHES: PGEN gates code on TWO INDEPENDENT AXES —
#   * FEATURES        (`--features "generated_parsers ebnf_dual_run"`), chosen by the caller, and
#   * ARTIFACT cfgs   (`has_generated_systemverilog_parser`, …), set by `build.rs` from what is
#                     present on disk.
# An item gated on ARTIFACT presence that is referenced from FEATURE-gated code compiles fine on
# every developer tree (where `generated/` is populated) and makes the crate UNBUILDABLE on a fresh
# clone. That is exactly what happened: `0099d0d3` (2026-08-11, `CI-PARITY-GATE-ROT.24` slice 2)
# added `ebnf_dual_run` to the canonical `ast_pipeline` recipe, which pulled
# `parse_harness_equivalence` into the bootstrap build, which references
# `parser_registry::active_grammar_profile` — then gated on
# `any(has_generated_systemverilog_parser, has_generated_regex_parser)`. The bootstrap died with
# ONE `E0425` — UNDETECTED for 10 days (found 2026-08-21), UNREPAIRED for 14 (fixed 2026-08-25) —
# because the target that CREATES the first family artifact has, as a prerequisite, a build that
# needed one to already exist.
#
# ⛔ WHY NOTHING NOTICED: all 11 hosted workflows using the composite regeneration action are
# `workflow_dispatch` only (deliberate Actions-minutes policy), and the one local instrument that
# replays a tracked-files-only tree (`prepare_generated_artifacts` in `ci_workflow_local_gate.sh`)
# is operator-invoked. Every developer and agent run has a populated `generated/`, where the path is
# green. This probe exists so the cold path is proven by something that RUNS.
#
# ⭐ HOW THE COLD STATE IS REACHED WITHOUT TOUCHING `generated/`: `build.rs` resolves every family
# artifact from a `PGEN_*_PARSER_PATH` env var (default: under `../generated/`). Pointing all of
# them at a path that does not exist reproduces the cold cfg state EXACTLY — same `build.rs` code
# path, same absent cfgs — while the developer's tree and warm build cache stay untouched. Builds go
# to a SEPARATE `CARGO_TARGET_DIR`. Nothing is moved, removed or restored.
#
#   ⚠️ SCOPE BOUND, stated because the probe cannot see past it: the two ANNOTATION parsers
#   (`generated/return_annotation_parser.rs`, `generated/semantic_annotation_parser.rs`) are
#   `include!`d by HARDCODED relative path under `#[cfg(feature = "generated_parsers")]`
#   (`rust/src/lib.rs:84,90`) — no artifact cfg guards them. The bootstrap generates that pair
#   BEFORE any family parser, so this probe models the bootstrap's state AT THE FAILING STEP:
#   annotation pair present, families absent. It does NOT model the earlier stage where the
#   annotation pair itself is missing.
#
# ARMS (numbered at RUN TIME in the order they execute — the count depends on the flags, so no
# number is hard-coded in this header, where it would rot the first time an arm is inserted)
#   always:
#     * cold `cargo check --lib`              — the library in the cold configuration
#     * cold `cargo check --bin ast_pipeline` — the EXACT build `regenerate_generated_parsers` runs
#   --census (the FEATURE-SET WIDENING, ~2 min: the two arms above close the census over the
#             BOOTSTRAP feature set only — this widens it to every feature set the crate declares,
#             because "it is the only such reference" is a census claim and a census in prose rots):
#     * cold `cargo check --lib` under default features
#     * cold `cargo check --lib --features normal`
#     * cold `cargo check --lib --features generated_parsers`
#     * cold `cargo check --lib --features ebnf_dual_run`
#     * cold `cargo check --bin ast_pipeline_bootstrap --features bootstrap` — the bootstrap's
#       FIRST build, the one that seeds generated/ebnf.rs
#     ⚠️ BOUND: `mimalloc_perf` and `never_free_arena_perf` are excluded — they only install a
#     global allocator and are not on any bootstrap path.
#   --self-test (falsifiability — `CI-PARITY-GATE-ROT.27`: a check that cannot fail proves nothing):
#     * RED CONTROL   — the founding defect re-injected into an ISOLATED COPY of the crate; the
#                       probe MUST go red with `E0425` on `active_grammar_profile`
#     * GREEN CONTROL — the same isolated copy, unpatched; MUST pass, proving the RED arm's failure
#                       comes from the re-injected gate and not from the copy itself
#
# EXIT CODES
#   0  every arm behaved as required
#   1  an arm failed (the cold configuration does not build, or a control misbehaved)
#   2  usage error / prerequisite missing
#
# USAGE
#   rust/scripts/cold_clone_build_probe.sh [--census] [--self-test] [--quiet]

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}" || exit 2

SELF_TEST=0
CENSUS=0
QUIET=0
for arg in "$@"; do
    case "${arg}" in
        --self-test) SELF_TEST=1 ;;
        --census)    CENSUS=1 ;;
        --quiet)     QUIET=1 ;;
        -h|--help)   sed -n '2,75p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) echo "usage: $(basename "${BASH_SOURCE[0]}") [--self-test] [--census] [--quiet]" >&2; exit 2 ;;
    esac
done

# All paths repo-root-relative (director policy 12) and on the repository's own volume (policy 13).
readonly BOOTSTRAP_FEATURES="generated_parsers ebnf_dual_run"
readonly PROBE_TARGET_DIR="rust/target/coldprobe"
readonly SELFTEST_ROOT="tmp/cold_clone_build_probe_selftest"
readonly LOG_DIR="rust/target/generated_logs/cold_clone_build_probe"
# A path that must NOT exist: resolved by build.rs relative to `rust/`, so no `has_generated_*` cfg
# is emitted for any family parser.
readonly ABSENT_ARTIFACT="../generated/__cold_clone_probe_absent__.rs"

mkdir -p "${LOG_DIR}" || exit 2

say() { [[ "${QUIET}" -eq 1 ]] || echo "$@"; }

fail_count=0
arm_no=0

# Guard against the probe silently testing a WARM tree: if the sentinel ever exists, the redirect is
# not a redirect and every arm below is vacuous.
if [[ -e "rust/${ABSENT_ARTIFACT#../}" || -e "generated/__cold_clone_probe_absent__.rs" ]]; then
    echo "❌ PREREQUISITE: the sentinel artifact path exists on disk — the cold redirect would be" >&2
    echo "   vacuous and every arm would test a WARM tree. Remove it and re-run." >&2
    exit 2
fi

# Run one cargo invocation in the COLD artifact configuration.
#   $1 = manifest path   $2 = target dir   $3 = log file   $4.. = extra cargo args
cold_cargo_check() {
    local manifest="$1" target_dir="$2" logfile="$3"
    shift 3
    env \
        CARGO_TARGET_DIR="${target_dir}" \
        PGEN_JSON_PARSER_PATH="${ABSENT_ARTIFACT}" \
        PGEN_REGEX_PARSER_PATH="${ABSENT_ARTIFACT}" \
        PGEN_SYSTEMVERILOG_PARSER_PATH="${ABSENT_ARTIFACT}" \
        PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH="${ABSENT_ARTIFACT}" \
        PGEN_VHDL_PARSER_PATH="${ABSENT_ARTIFACT}" \
        PGEN_RTL_CONST_EXPR_PARSER_PATH="${ABSENT_ARTIFACT}" \
        PGEN_RTL_FRONTEND_PARSER_PATH="${ABSENT_ARTIFACT}" \
        PGEN_SCRATCH_PARSER_PATH="${ABSENT_ARTIFACT}" \
        cargo check --manifest-path "${manifest}" \
            --message-format short "$@" \
        >"${logfile}" 2>&1
}

# $1 = human label   $2 = expected outcome (pass|fail)   $3 = manifest   $4 = target dir   $5.. cargo args
run_arm() {
    local label="$1" expect="$2" manifest="$3" target_dir="$4"
    shift 4
    arm_no=$((arm_no + 1))
    local logfile="${LOG_DIR}/arm${arm_no}.log"
    say "   arm ${arm_no}: ${label} (expect ${expect})"
    cold_cargo_check "${manifest}" "${target_dir}" "${logfile}" "$@"
    local rc=$?
    local errors
    errors="$(grep -cE 'error\[|^error(:| )' "${logfile}" 2>/dev/null || true)"
    if [[ "${expect}" == "pass" ]]; then
        if [[ ${rc} -eq 0 ]]; then
            say "           ✅ exit=0, 0 errors"
        else
            fail_count=$((fail_count + 1))
            echo "           ❌ exit=${rc}, ${errors} error line(s) — the COLD configuration does NOT build."
            echo "           ── first errors ──"
            grep -E 'error\[|^error(:| )' "${logfile}" | head -5 | sed 's/^/           /'
            echo "           full log: ${logfile}"
        fi
    else
        if [[ ${rc} -ne 0 ]]; then
            say "           ✅ exit=${rc}, ${errors} error line(s) — the control went RED as required"
            grep -E 'error\[' "${logfile}" | head -2 | sed 's/^/           /' | { [[ "${QUIET}" -eq 1 ]] && cat >/dev/null || cat; }
        else
            fail_count=$((fail_count + 1))
            echo "           ❌ exit=0 — the RED CONTROL PASSED. This probe cannot detect its own"
            echo "              founding defect and must not be trusted (log: ${logfile})."
        fi
    fi
}

say "🧊 COLD-CLONE BUILD PROBE (CI-PARITY-GATE-ROT.40)"
say "   Features: ${BOOTSTRAP_FEATURES}   |   every has_generated_* family cfg: ABSENT"
say "   Target dir: ${PROBE_TARGET_DIR} (separate — the working build cache is untouched)"

run_arm "cold cargo check --lib" pass \
    "rust/Cargo.toml" "${PROBE_TARGET_DIR}" --features "${BOOTSTRAP_FEATURES}" --lib
run_arm "cold cargo check --bin ast_pipeline (the bootstrap's own build)" pass \
    "rust/Cargo.toml" "${PROBE_TARGET_DIR}" --features "${BOOTSTRAP_FEATURES}" --bin ast_pipeline

if [[ "${CENSUS}" -eq 1 ]]; then
    say ""
    say "📋 CENSUS — the same cold configuration under every feature set the crate declares"
    run_arm "cold --lib, DEFAULT features" pass \
        "rust/Cargo.toml" "${PROBE_TARGET_DIR}" --lib
    run_arm "cold --lib --features normal" pass \
        "rust/Cargo.toml" "${PROBE_TARGET_DIR}" --features normal --lib
    run_arm "cold --lib --features generated_parsers" pass \
        "rust/Cargo.toml" "${PROBE_TARGET_DIR}" --features generated_parsers --lib
    run_arm "cold --lib --features ebnf_dual_run" pass \
        "rust/Cargo.toml" "${PROBE_TARGET_DIR}" --features ebnf_dual_run --lib
    run_arm "cold --bin ast_pipeline_bootstrap --features bootstrap (seeds generated/ebnf.rs)" pass \
        "rust/Cargo.toml" "${PROBE_TARGET_DIR}" --features bootstrap --bin ast_pipeline_bootstrap
fi

if [[ "${SELF_TEST}" -eq 1 ]]; then
    say ""
    say "🔬 SELF-TEST — re-injecting the founding defect into an isolated copy"
    rm -rf "${SELFTEST_ROOT}"
    mkdir -p "${SELFTEST_ROOT}" || exit 2
    # Copy the crate WITHOUT its build dir; symlink `generated/` so the hardcoded annotation
    # `include!` paths (`../../generated/...`) resolve to the real, read-only artifacts.
    rsync -a --exclude 'target/' --exclude '*.log' rust/ "${SELFTEST_ROOT}/rust/" || exit 2
    ln -s "${REPO_ROOT}/generated" "${SELFTEST_ROOT}/generated" || exit 2

    local_registry="${SELFTEST_ROOT}/rust/src/parser_registry.rs"
    if ! grep -q '^pub fn active_grammar_profile' "${local_registry}"; then
        echo "❌ SELF-TEST PREREQUISITE: \`active_grammar_profile\` not found where the defect is" >&2
        echo "   re-injected. The control is stale — update it before trusting this probe." >&2
        exit 2
    fi
    # Re-inject the EXACT gate `0099d0d3`'s bootstrap tripped over.
    perl -0pi -e 's/^pub fn active_grammar_profile/#[cfg(any(has_generated_systemverilog_parser, has_generated_regex_parser))]\npub fn active_grammar_profile/m' \
        "${local_registry}" || exit 2
    if ! grep -q 'has_generated_systemverilog_parser, has_generated_regex_parser))\]$' "${local_registry}"; then
        echo "❌ SELF-TEST PREREQUISITE: the defect re-injection did not apply." >&2
        exit 2
    fi

    run_arm "RED CONTROL — artifact gate re-added to active_grammar_profile" fail \
        "${SELFTEST_ROOT}/rust/Cargo.toml" "${SELFTEST_ROOT}/target" \
        --features "${BOOTSTRAP_FEATURES}" --lib

    # Undo the injection in the copy: the same tree must now be GREEN, isolating the cause.
    perl -0pi -e 's/^#\[cfg\(any\(has_generated_systemverilog_parser, has_generated_regex_parser\)\)\]\npub fn active_grammar_profile/pub fn active_grammar_profile/m' \
        "${local_registry}" || exit 2
    run_arm "GREEN CONTROL — same copy, injection removed" pass \
        "${SELFTEST_ROOT}/rust/Cargo.toml" "${SELFTEST_ROOT}/target" \
        --features "${BOOTSTRAP_FEATURES}" --lib

    rm -rf "${SELFTEST_ROOT}"
fi

say ""
if [[ ${fail_count} -eq 0 ]]; then
    say "✅ COLD-CLONE BUILD PROBE PASSED — ${arm_no}/${arm_no} arms."
    exit 0
fi
echo "❌ COLD-CLONE BUILD PROBE FAILED — ${fail_count} of ${arm_no} arms."
echo "   A fresh clone cannot run README.md's Quick Start, and every hosted workflow that"
echo "   regenerates parsers dies at its first build step."
exit 1
