#!/usr/bin/env bash
# require_ast_pipeline_features.sh — refuse to MEASURE with an under-featured `ast_pipeline`.
#
# CI-PARITY-GATE-ROT.24. `rust/target/debug/ast_pipeline` is written by THREE different recipes
# with three different feature sets (`generated_parsers` alone — the canonical `$(RUST_AST_PIPELINE)`
# file rule that 21 targets depend on; `ebnf_dual_run` alone; and both), so what that path can do is
# a function of which target ran most recently. Nothing declares what it is SUPPOSED to be.
#
# ⛔ THE FAILURE THIS EXISTS TO STOP IS SILENT AND IT FAILS IN THE PASSING DIRECTION.
# An under-featured binary does not crash: it prints
#   Error: EBNF input 'grammars/json.ebnf' requires building with --features ebnf_dual_run
# on STDERR and exits non-zero, while a sweep scraping STDOUT for `CERTIFICATE-COVERAGE:` records an
# EMPTY row. Measured, in the session that opened `.24`: a before/after cert sweep produced 18 empty
# rows on EACH side — and TWO EMPTY SETS DIFF CLEAN, so it would have published "zero drift across
# every grammar x seeds 0/7/42" backed by nothing at all.
#
# USAGE
#   scripts/require_ast_pipeline_features.sh <bin> <feature>...
#   scripts/require_ast_pipeline_features.sh --self-test
#
#   e.g.  scripts/require_ast_pipeline_features.sh rust/target/debug/ast_pipeline \
#             generated_parsers ebnf_dual_run
#
# Exits 0 only when the binary EXISTS, answers `--report-feature-surface`, and reports every
# requested feature as `true`. Any other outcome is a hard refusal naming the exact rebuild command.
#
# ⭐ CONTRACT: this guard REFUSES rather than guesses. An unreadable surface, an unparseable line or
# an unknown feature name is a refusal, never a pass — the whole point is that "I could not tell" and
# "it is fine" must never be the same exit code.

set -euo pipefail

SURFACE_PREFIX="AST-PIPELINE-FEATURE-SURFACE:"
GUARD_PREFIX="AST-PIPELINE-FEATURE-GUARD:"

usage() {
    cat >&2 <<'USAGE'
usage: require_ast_pipeline_features.sh <bin> <feature>...
       require_ast_pipeline_features.sh --self-test

Verifies an `ast_pipeline` binary was built with the features a measurement needs,
so an under-featured binary cannot silently yield an empty metric.
USAGE
}

# Print the rebuild command that would satisfy the requested feature set.
rebuild_hint() {
    printf '  cd rust && cargo build --features "%s" --bin ast_pipeline\n' "$*"
}

require_features() {
    local bin="$1"
    shift
    local -a wanted=("$@")

    if [[ ${#wanted[@]} -eq 0 ]]; then
        echo "${GUARD_PREFIX} REFUSED — no features requested; a guard that checks nothing is worse than none" >&2
        return 2
    fi

    if [[ ! -x "$bin" ]]; then
        echo "${GUARD_PREFIX} REFUSED — binary '$bin' is missing or not executable." >&2
        echo "  build it:" >&2
        rebuild_hint "${wanted[@]}" >&2
        return 1
    fi

    # `--report-feature-surface` is answered PRE-clap and is itself feature-independent
    # (PARSE-HARNESS.10), so it is readable from any build of the binary — including the
    # under-featured one this guard exists to catch.
    local surface
    if ! surface="$("$bin" --report-feature-surface 2>/dev/null)"; then
        echo "${GUARD_PREFIX} REFUSED — '$bin' did not answer --report-feature-surface." >&2
        echo "  It is too old to describe itself, so its capability cannot be established. Rebuild:" >&2
        rebuild_hint "${wanted[@]}" >&2
        return 1
    fi

    if [[ "$surface" != *"${SURFACE_PREFIX}"* ]]; then
        echo "${GUARD_PREFIX} REFUSED — unparseable feature surface from '$bin':" >&2
        printf '    %s\n' "$surface" >&2
        return 1
    fi

    local -a missing=()
    local feature
    for feature in "${wanted[@]}"; do
        # Match `<feature>=true` as a whole token so `ebnf_dual_run=false` can never
        # satisfy a request for `ebnf_dual_run`.
        if [[ ! " $surface " == *" ${feature}=true "* ]]; then
            if [[ ! " $surface " == *" ${feature}=false "* ]]; then
                echo "${GUARD_PREFIX} REFUSED — '$bin' does not report a feature named '${feature}'." >&2
                echo "    surface: ${surface}" >&2
                echo "  A feature this guard cannot see is not a feature it can vouch for." >&2
                return 1
            fi
            missing+=("$feature")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "${GUARD_PREFIX} REFUSED — '$bin' is UNDER-FEATURED for this measurement." >&2
        echo "    required: ${wanted[*]}" >&2
        echo "    missing:  ${missing[*]}" >&2
        echo "    surface:  ${surface}" >&2
        echo "  ⛔ Measuring anyway yields EMPTY metric rows, not errors — and two empty" >&2
        echo "     result sets compare EQUAL. Rebuild before measuring:" >&2
        rebuild_hint "${wanted[@]}" >&2
        return 1
    fi

    echo "${GUARD_PREFIX} ok — '$bin' carries [${wanted[*]}]"
    return 0
}

# ── Self-test: prove the guard FIRES, both directions, before it is trusted ────────────────────
# A guard nobody has seen refuse is indistinguishable from `true`. This builds stub binaries that
# imitate `--report-feature-surface` and asserts the guard PASSES the complete one and REFUSES the
# under-featured one — the same positive+negative control discipline the envelope differential
# (TOOLBOX 1.9) runs before it publishes a number.
self_test() {
    local tmp
    tmp="$(mktemp -d)"
    # shellcheck disable=SC2064
    trap "rm -rf '$tmp'" RETURN

    local full="$tmp/full" partial="$tmp/partial" mute="$tmp/mute"
    printf '#!/usr/bin/env bash\necho "%s ebnf_dual_run=true generated_parsers=true"\n' \
        "$SURFACE_PREFIX" > "$full"
    printf '#!/usr/bin/env bash\necho "%s ebnf_dual_run=false generated_parsers=true"\n' \
        "$SURFACE_PREFIX" > "$partial"
    printf '#!/usr/bin/env bash\nexit 3\n' > "$mute"
    chmod +x "$full" "$partial" "$mute"

    local failures=0

    if require_features "$full" generated_parsers ebnf_dual_run >/dev/null 2>&1; then
        echo "  self-test POSITIVE control: pass (a complete binary is accepted)"
    else
        echo "  self-test POSITIVE control: FAIL — the guard refused a complete binary" >&2
        failures=$((failures + 1))
    fi

    if require_features "$partial" generated_parsers ebnf_dual_run >/dev/null 2>&1; then
        echo "  self-test NEGATIVE control: FAIL — the guard PASSED an under-featured binary" >&2
        failures=$((failures + 1))
    else
        echo "  self-test NEGATIVE control: pass (ebnf_dual_run=false is refused)"
    fi

    if require_features "$tmp/does-not-exist" generated_parsers >/dev/null 2>&1; then
        echo "  self-test MISSING-BINARY control: FAIL — a missing binary passed" >&2
        failures=$((failures + 1))
    else
        echo "  self-test MISSING-BINARY control: pass"
    fi

    if require_features "$mute" generated_parsers >/dev/null 2>&1; then
        echo "  self-test MUTE-BINARY control: FAIL — a binary that cannot describe itself passed" >&2
        failures=$((failures + 1))
    else
        echo "  self-test MUTE-BINARY control: pass (cannot-tell is not the same as fine)"
    fi

    if [[ $failures -eq 0 ]]; then
        echo "${GUARD_PREFIX} self-test ok — 4/4 controls (the guard is proven to fire)"
        return 0
    fi
    echo "${GUARD_PREFIX} self-test FAILED — $failures control(s) did not behave" >&2
    return 1
}

main() {
    if [[ $# -eq 0 ]]; then
        usage
        exit 2
    fi
    if [[ "$1" == "--self-test" ]]; then
        self_test
        exit $?
    fi
    if [[ "$1" == "-h" || "$1" == "--help" ]]; then
        usage
        exit 0
    fi
    if [[ $# -lt 2 ]]; then
        usage
        exit 2
    fi
    require_features "$@"
}

main "$@"
