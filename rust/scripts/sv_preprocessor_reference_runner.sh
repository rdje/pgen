#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  sv_preprocessor_reference_runner.sh --probe
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
  - --probe checks whether a trusted-reference backend is available and exits 0/1.
  - Diagnostics file is always emitted as a JSON array.
EOF
}

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

resolve_selected_backend() {
    case "$backend_mode" in
        auto)
            if command -v "$iverilog_bin" >/dev/null 2>&1; then
                echo "iverilog"
            elif command -v "$verilator_bin" >/dev/null 2>&1; then
                echo "verilator"
            else
                echo "none"
            fi
            ;;
        iverilog)
            echo "iverilog"
            ;;
        verilator)
            echo "verilator"
            ;;
    esac
}

ensure_backend_available() {
    local selected="$1"
    if [[ "$selected" == "none" ]]; then
        echo "no supported trusted-reference preprocessor backend found (install iverilog or verilator)" >&2
        return 1
    fi
    if [[ "$selected" == "iverilog" ]] && ! command -v "$iverilog_bin" >/dev/null 2>&1; then
        echo "iverilog backend not found: $iverilog_bin" >&2
        return 1
    fi
    if [[ "$selected" == "verilator" ]] && ! command -v "$verilator_bin" >/dev/null 2>&1; then
        echo "verilator backend not found: $verilator_bin" >&2
        return 1
    fi
    return 0
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage
    exit 0
fi

if [[ "${1:-}" == "--probe" ]]; then
    if [[ "$#" -ne 1 ]]; then
        usage >&2
        exit 2
    fi
    selected_backend="$(resolve_selected_backend)"
    if ensure_backend_available "$selected_backend"; then
        echo "$selected_backend"
        exit 0
    fi
    exit 1
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

selected_backend="$(resolve_selected_backend)"
if ! ensure_backend_available "$selected_backend" >"$stderr_file" 2>&1; then
    emit_stderr_diagnostics "$backend_mode" "trusted reference backend unavailable"
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
