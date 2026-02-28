#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  sv_preprocessor_reference_runner.sh <input_sv_file> <output_sv_file> <diagnostics_json_file>

Environment:
  PGEN_SV_PREPROCESSOR_REFERENCE_BACKEND=auto|iverilog|verilator   (default: auto)
  PGEN_SV_PREPROCESSOR_REFERENCE_IVERILOG_BIN=<path>               (default: iverilog)
  PGEN_SV_PREPROCESSOR_REFERENCE_VERILATOR_BIN=<path>              (default: verilator)
  PGEN_SV_PREPROCESSOR_REFERENCE_LANGUAGE=<profile>                (default: 1800-2017)
  PGEN_SV_PREPROCESSOR_REFERENCE_INCLUDE_DIRS=<csv dirs>           (optional)
  PGEN_SV_PREPROCESSOR_REFERENCE_DEFINES=<csv defs>                (optional)

Notes:
  - Auto backend prefers iverilog, then verilator.
  - Diagnostics file is always emitted as a JSON array.
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage
    exit 0
fi

if [[ "$#" -ne 3 ]]; then
    usage >&2
    exit 2
fi

input_file="$1"
output_file="$2"
diagnostics_file="$3"

if [[ ! -f "$input_file" ]]; then
    echo "error: input file not found: $input_file" >&2
    exit 2
fi

mkdir -p "$(dirname "$output_file")" "$(dirname "$diagnostics_file")"

backend_mode="${PGEN_SV_PREPROCESSOR_REFERENCE_BACKEND:-auto}"
iverilog_bin="${PGEN_SV_PREPROCESSOR_REFERENCE_IVERILOG_BIN:-iverilog}"
verilator_bin="${PGEN_SV_PREPROCESSOR_REFERENCE_VERILATOR_BIN:-verilator}"
language_profile="${PGEN_SV_PREPROCESSOR_REFERENCE_LANGUAGE:-1800-2017}"
include_dirs_csv="${PGEN_SV_PREPROCESSOR_REFERENCE_INCLUDE_DIRS:-}"
defines_csv="${PGEN_SV_PREPROCESSOR_REFERENCE_DEFINES:-}"

case "$backend_mode" in
    auto|iverilog|verilator)
        ;;
    *)
        echo "error: PGEN_SV_PREPROCESSOR_REFERENCE_BACKEND must be one of: auto, iverilog, verilator" >&2
        exit 2
        ;;
esac

stderr_file="$(mktemp "${TMPDIR:-/tmp}/svpp_reference_runner_stderr.XXXXXX")"
trap 'rm -f "$stderr_file"' EXIT

emit_empty_diagnostics() {
    printf '[]\n' >"$diagnostics_file"
}

emit_stderr_diagnostics() {
    local selected_backend="$1"
    local fallback_message="$2"
    if [[ -s "$stderr_file" ]]; then
        jq -R -s --arg backend "$selected_backend" '
            split("\n")
            | map(select(length > 0))
            | map({
                code: (if test("(?i)warning") then "reference_warning" else "reference_error" end),
                severity: (if test("(?i)warning") then "warning" else "error" end),
                file: "",
                line: 0,
                message: .,
                detail: ("trusted reference runner (" + $backend + ")")
            })
        ' "$stderr_file" >"$diagnostics_file"
    else
        jq -n --arg backend "$selected_backend" --arg message "$fallback_message" '
            [{
                code: "reference_error",
                severity: "error",
                file: "",
                line: 0,
                message: $message,
                detail: ("trusted reference runner (" + $backend + ")")
            }]
        ' >"$diagnostics_file"
    fi
}

build_common_flags() {
    local -n out_flags_ref="$1"
    out_flags_ref=()

    if [[ -n "$include_dirs_csv" ]]; then
        local old_ifs="$IFS"
        IFS=','
        read -r -a _include_dirs <<<"$include_dirs_csv"
        IFS="$old_ifs"
        for dir in "${_include_dirs[@]}"; do
            [[ -z "$dir" ]] && continue
            out_flags_ref+=("-I" "$dir")
        done
    fi

    if [[ -n "$defines_csv" ]]; then
        local old_ifs="$IFS"
        IFS=','
        read -r -a _defines <<<"$defines_csv"
        IFS="$old_ifs"
        for def in "${_defines[@]}"; do
            [[ -z "$def" ]] && continue
            out_flags_ref+=("-D" "$def")
        done
    fi
}

run_iverilog() {
    local -a common_flags=()
    build_common_flags common_flags
    "$iverilog_bin" -g2012 -E "${common_flags[@]}" "$input_file" >"$output_file" 2>"$stderr_file"
}

run_verilator() {
    local -a common_flags=()
    build_common_flags common_flags
    "$verilator_bin" -E --language "$language_profile" "${common_flags[@]}" "$input_file" >"$output_file" 2>"$stderr_file"
}

selected_backend=""
case "$backend_mode" in
    auto)
        if command -v "$iverilog_bin" >/dev/null 2>&1; then
            selected_backend="iverilog"
        elif command -v "$verilator_bin" >/dev/null 2>&1; then
            selected_backend="verilator"
        else
            selected_backend="none"
        fi
        ;;
    iverilog)
        selected_backend="iverilog"
        ;;
    verilator)
        selected_backend="verilator"
        ;;
esac

if [[ "$selected_backend" == "none" ]]; then
    emit_stderr_diagnostics "auto" "no supported trusted-reference preprocessor backend found (install iverilog or verilator)"
    exit 1
fi

if [[ "$selected_backend" == "iverilog" ]] && ! command -v "$iverilog_bin" >/dev/null 2>&1; then
    emit_stderr_diagnostics "iverilog" "iverilog backend not found: $iverilog_bin"
    exit 1
fi

if [[ "$selected_backend" == "verilator" ]] && ! command -v "$verilator_bin" >/dev/null 2>&1; then
    emit_stderr_diagnostics "verilator" "verilator backend not found: $verilator_bin"
    exit 1
fi

set +e
if [[ "$selected_backend" == "iverilog" ]]; then
    run_iverilog
    runner_exit=$?
else
    run_verilator
    runner_exit=$?
fi
set -e

if [[ "$runner_exit" -eq 0 ]]; then
    [[ -f "$output_file" ]] || : >"$output_file"
    if [[ -s "$stderr_file" ]]; then
        emit_stderr_diagnostics "$selected_backend" "trusted reference backend emitted diagnostics"
    else
        emit_empty_diagnostics
    fi
    exit 0
fi

emit_stderr_diagnostics "$selected_backend" "trusted reference backend failed with exit code $runner_exit"
exit "$runner_exit"
