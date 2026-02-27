#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"

STATE_DIR="${PGEN_SV_STIMULI_QUALITY_STATE_DIR:-$RUST_DIR/target/sv_stimuli_quality_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

CONTRACT_FILE="${PGEN_SV_STIMULI_QUALITY_CONTRACT:-$RUST_DIR/test_data/grammar_quality/systemverilog_core_v0_contract.json}"
PARSE_FULL_MODE="${PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE:-auto}"
SAMPLE_COUNT_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_COUNT:-}"
SEED_BASE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_SEED_BASE:-}"
LRM_PROFILE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_LRM_PROFILE:-}"
LRM_PROFILES_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_LRM_PROFILES:-}"
STIMULI_MODE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_MODE:-}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool '$tool' is not available in PATH" >&2
        exit 1
    fi
}

require_file() {
    local path="$1"
    if [[ ! -f "$path" ]]; then
        echo "error: missing required file '$path'" >&2
        exit 1
    fi
}

require_nonempty_file() {
    local path="$1"
    if [[ ! -s "$path" ]]; then
        echo "error: expected non-empty artifact '$path'" >&2
        exit 1
    fi
}

check_balanced_structural_keywords() {
    local file="$1"
    local pair
    for pair in \
        "module:endmodule" \
        "interface:endinterface" \
        "program:endprogram" \
        "package:endpackage" \
        "class:endclass" \
        "begin:end"; do
        local open_kw="${pair%%:*}"
        local close_kw="${pair##*:}"
        local open_count
        local close_count
        open_count="$(perl -ne "while (/\\b\\Q${open_kw}\\E\\b/g) { ++\$c } END { print(\$c // 0) }" "$file")"
        close_count="$(perl -ne "while (/\\b\\Q${close_kw}\\E\\b/g) { ++\$c } END { print(\$c // 0) }" "$file")"
        if [[ "$open_count" != "$close_count" ]]; then
            echo "unbalanced structural keywords: ${open_kw}=${open_count}, ${close_kw}=${close_count}"
            return 1
        fi
    done
    return 0
}

check_unique_named_port_bindings() {
    local file="$1"
    local out
    if ! out="$(
        perl -0777 -e '
            use strict;
            use warnings;
            local $/;
            my $text = <>;
            my @stmts = split /;/, $text;
            my $stmt_idx = 0;
            for my $stmt (@stmts) {
                $stmt_idx++;
                my @names = ($stmt =~ /\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/g);
                next unless @names > 1;
                my %seen;
                for my $name (@names) {
                    if ($seen{$name}++) {
                        print "duplicate named port binding \"$name\" in statement $stmt_idx\n";
                        exit 1;
                    }
                }
            }
            exit 0;
        ' "$file" 2>&1
    )"; then
        echo "$out"
        return 1
    fi
    return 0
}

check_port_binding_legality_basic() {
    local file="$1"
    local out
    if ! out="$(
        perl -0777 -e '
            use strict;
            use warnings;
            local $/;
            my $text = <>;
            $text =~ s!/\*.*?\*/!!gs;
            $text =~ s!//.*?$!!gm;

            my %keywords = map { $_ => 1 } qw(
                module endmodule interface endinterface program endprogram package endpackage class endclass
                if else for foreach while case endcase begin end assign always always_ff always_comb always_latch
                function endfunction task endtask generate endgenerate
                input output inout ref wire logic reg bit byte shortint int integer longint
                signed unsigned var parameter localparam typedef struct union enum
            );

            my %module_ports;
            while ($text =~ /\bmodule\s+([A-Za-z_][A-Za-z0-9_]*)\b(.*?)\bendmodule\b/sg) {
                my ($module_name, $module_body) = ($1, $2);
                my $header = "";
                if ($module_body =~ /(.*?;)/s) {
                    $header = $1;
                }
                next unless $header =~ /\((.*)\)\s*;/s;
                my $ports_block = $1;
                my @segments = split /,/, $ports_block;
                for my $seg (@segments) {
                    my @ids = ($seg =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g);
                    next unless @ids;
                    my $candidate = $ids[-1];
                    next if $keywords{$candidate};
                    $module_ports{$module_name}{$candidate} = 1;
                }
            }

            while ($text =~ /\b([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\((.*?)\)\s*;/sg) {
                my ($module_type, $inst_name, $bindings) = ($1, $2, $3);
                next if $keywords{$module_type};
                next unless exists $module_ports{$module_type};
                while ($bindings =~ /\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/g) {
                    my $bound_port = $1;
                    next if $bound_port eq "*";
                    if (!exists $module_ports{$module_type}{$bound_port}) {
                        print "illegal named port binding: ${module_type}.${bound_port} in instance ${inst_name}\n";
                        exit 1;
                    }
                }
            }
            exit 0;
        ' "$file" 2>&1
    )"; then
        echo "$out"
        return 1
    fi
    return 0
}

check_declared_identifiers_before_use() {
    local file="$1"
    local out
    if ! out="$(
        perl -0777 -e '
            use strict;
            use warnings;
            local $/;
            my $text = <>;
            $text =~ s!/\*.*?\*/!!gs;
            $text =~ s!//.*?$!!gm;

            my %keywords = map { $_ => 1 } qw(
                module endmodule interface endinterface program endprogram package endpackage class endclass
                begin end if else case endcase for foreach while repeat do
                always always_ff always_comb always_latch
                assign wire logic reg bit byte shortint int integer longint string chandle event time realtime
                signed unsigned input output inout ref var parameter localparam type typedef enum struct union
                packed unpacked function endfunction task endtask import export virtual static automatic const
                generate endgenerate genvar return break continue default initial final disable wait fork join
                join_any join_none
            );

            my %declared;
            while ($text =~ /\b(?:module|interface|program|package|class|function|task)\s+([A-Za-z_][A-Za-z0-9_]*)/g) {
                $declared{$1} = 1;
            }
            while ($text =~ /\b(?:logic|reg|wire|bit|byte|shortint|int|integer|longint|string|chandle|event|time|realtime)\b(?:\s+(?:signed|unsigned))?\s+([^;]+)/g) {
                my $tail = $1;
                while ($tail =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                    $declared{$1} = 1;
                }
            }
            while ($text =~ /\b(?:parameter|localparam|genvar|typedef)\b([^;]*);/g) {
                my $tail = $1;
                while ($tail =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                    $declared{$1} = 1;
                }
            }

            while ($text =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                my $id = $1;
                next if $keywords{$id};
                next if $declared{$id};
                next if $id =~ /^(?:[A-Z_][A-Z0-9_]*|x|z)$/;
                print "undeclared identifier use detected (first=\"$id\")\n";
                exit 1;
            }
            exit 0;
        ' "$file" 2>&1
    )"; then
        echo "$out"
        return 1
    fi
    return 0
}

check_package_qualification_resolution() {
    local file="$1"
    local out
    if ! out="$(
        perl -0777 -e '
            use strict;
            use warnings;
            local $/;
            my $text = <>;
            $text =~ s!/\*.*?\*/!!gs;
            $text =~ s!//.*?$!!gm;

            my %pkg_declared;
            while ($text =~ /\bpackage\s+([A-Za-z_][A-Za-z0-9_]*)/g) {
                $pkg_declared{$1} = 1;
            }
            my %pkg_imported;
            while ($text =~ /\bimport\s+([A-Za-z_][A-Za-z0-9_]*)::(?:\*|[A-Za-z_][A-Za-z0-9_]*)/g) {
                $pkg_imported{$1} = 1;
            }

            while ($text =~ /\b([A-Za-z_][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*)/g) {
                my ($pkg, $symbol) = ($1, $2);
                next if $pkg_declared{$pkg};
                next if $pkg_imported{$pkg};
                print "unresolved package qualification: ${pkg}::${symbol}\n";
                exit 1;
            }
            exit 0;
        ' "$file" 2>&1
    )"; then
        echo "$out"
        return 1
    fi
    return 0
}

check_width_compatibility_simple() {
    local file="$1"
    local out
    if ! out="$(
        perl -0777 -e '
            use strict;
            use warnings;
            local $/;
            my $text = <>;
            $text =~ s!/\*.*?\*/!!gs;
            $text =~ s!//.*?$!!gm;

            my %width_of;
            while ($text =~ /\blogic\s*\[\s*(\d+)\s*:\s*(\d+)\s*\]\s*([A-Za-z_][A-Za-z0-9_]*)/g) {
                my ($msb, $lsb, $name) = ($1, $2, $3);
                my $width = $msb >= $lsb ? ($msb - $lsb + 1) : ($lsb - $msb + 1);
                $width_of{$name} = $width;
            }
            while ($text =~ /\b([A-Za-z_][A-Za-z0-9_]*)\s*(?:<=|=)\s*(\d+)\s*'\''[bBoOdDhH][0-9a-fA-F_xXzZ]+/g) {
                my ($lhs, $lit_width) = ($1, $2);
                next if !exists $width_of{$lhs};
                if ($lit_width > $width_of{$lhs}) {
                    print "literal width overflow: ${lhs} width=${width_of{$lhs}} literal_width=${lit_width}\n";
                    exit 1;
                }
            }
            exit 0;
        ' "$file" 2>&1
    )"; then
        echo "$out"
        return 1
    fi
    return 0
}

check_context_legality_basic() {
    local file="$1"
    local out
    if ! out="$(
        perl -0777 -e '
            use strict;
            use warnings;
            local $/;
            my $text = <>;
            $text =~ s!/\*.*?\*/!!gs;
            $text =~ s!//.*?$!!gm;

            my %genvar_declared;
            while ($text =~ /\bgenvar\b([^;]*);/g) {
                my $tail = $1;
                while ($tail =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                    $genvar_declared{$1} = 1;
                }
            }
            while ($text =~ /\bgenerate\b(.*?)\bendgenerate\b/sg) {
                my $blk = $1;
                while ($blk =~ /\bfor\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*=/g) {
                    my $iter = $1;
                    if (!$genvar_declared{$iter}) {
                        print "context legality violation: generate for iterator \"$iter\" is not declared as genvar\n";
                        exit 1;
                    }
                }
            }

            while ($text =~ /\balways_comb\b(.*?)(?:\bend\b|;)/sg) {
                my $blk = $1;
                if ($blk =~ /\@\s*\(/) {
                    print "context legality violation: always_comb contains event control\n";
                    exit 1;
                }
            }
            while ($text =~ /\balways_ff\b(.*?)(?:\bend\b|;)/sg) {
                my $blk = $1;
                if ($blk =~ /\b[A-Za-z_][A-Za-z0-9_]*(?:\[[^\]]+\])?\s*(?<!<)=(?!=)/) {
                    print "context legality violation: always_ff contains blocking assignment\n";
                    exit 1;
                }
            }
            exit 0;
        ' "$file" 2>&1
    )"; then
        echo "$out"
        return 1
    fi
    return 0
}

csv_sanitize() {
    local text="$1"
    text="${text//$'\r'/ }"
    text="${text//$'\n'/ }"
    text="${text//,/;}"
    printf '%s' "$text"
}

evaluate_semantic_baseline() {
    local preprocessed_file="$1"
    local preprocess_error_count="$2"
    SEMANTIC_EVAL_NOTE="baseline semantic validation passed"

    if [[ "$require_nonempty_preprocessed_output" -eq 1 ]] && [[ ! -s "$preprocessed_file" ]]; then
        SEMANTIC_EVAL_NOTE="preprocessed output is empty"
        return 1
    fi
    if [[ "$require_no_preprocess_errors" -eq 1 ]] && (( preprocess_error_count > 0 )); then
        SEMANTIC_EVAL_NOTE="preprocessor diagnostics contain error severity entries"
        return 1
    fi
    if [[ "$require_balanced_structural_keywords" -eq 1 ]]; then
        if ! SEMANTIC_EVAL_NOTE="$(check_balanced_structural_keywords "$preprocessed_file")"; then
            return 1
        fi
    fi
    if [[ "$require_unique_named_port_bindings" -eq 1 ]]; then
        if ! SEMANTIC_EVAL_NOTE="$(check_unique_named_port_bindings "$preprocessed_file")"; then
            return 1
        fi
    fi
    if [[ "$require_port_binding_legality_basic" -eq 1 ]]; then
        if ! SEMANTIC_EVAL_NOTE="$(check_port_binding_legality_basic "$preprocessed_file")"; then
            return 1
        fi
    fi
    if [[ "$require_declared_identifiers_before_use" -eq 1 ]]; then
        if ! SEMANTIC_EVAL_NOTE="$(check_declared_identifiers_before_use "$preprocessed_file")"; then
            return 1
        fi
    fi
    if [[ "$require_package_qualification_resolution" -eq 1 ]]; then
        if ! SEMANTIC_EVAL_NOTE="$(check_package_qualification_resolution "$preprocessed_file")"; then
            return 1
        fi
    fi
    if [[ "$require_width_compatibility_simple" -eq 1 ]]; then
        if ! SEMANTIC_EVAL_NOTE="$(check_width_compatibility_simple "$preprocessed_file")"; then
            return 1
        fi
    fi
    if [[ "$require_context_legality_basic" -eq 1 ]]; then
        if ! SEMANTIC_EVAL_NOTE="$(check_context_legality_basic "$preprocessed_file")"; then
            return 1
        fi
    fi
    return 0
}

semantic_failure_predicate() {
    local candidate_file="$1"
    local preprocess_error_count="$2"
    if evaluate_semantic_baseline "$candidate_file" "$preprocess_error_count"; then
        return 1
    fi
    return 0
}

parse_full_failure_predicate() {
    local candidate_file="$1"
    if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$candidate_file" >/dev/null 2>&1; then
        return 1
    fi
    return 0
}

deterministic_prefix_shrink() {
    local input_file="$1"
    local output_file="$2"
    local max_iterations="$3"
    local predicate_fn="$4"
    shift 4
    local predicate_args=("$@")

    local -a _shrink_lines=()
    mapfile -t _shrink_lines < "$input_file"
    local total_lines="${#_shrink_lines[@]}"
    if (( total_lines == 0 )); then
        cp "$input_file" "$output_file"
        echo 0
        return 0
    fi

    cp "$input_file" "$output_file"
    local lo=1
    local hi="$total_lines"
    local best_lines="$total_lines"
    local iterations=0
    local tmp_candidate="$WORK_DIR/.shrink_candidate.$$.$RANDOM.sv"

    while (( lo <= hi )) && (( iterations < max_iterations )); do
        iterations=$((iterations + 1))
        local mid=$(((lo + hi) / 2))
        : >"$tmp_candidate"
        local i
        for ((i = 0; i < mid; i++)); do
            printf '%s\n' "${_shrink_lines[$i]}" >>"$tmp_candidate"
        done
        if "$predicate_fn" "$tmp_candidate" "${predicate_args[@]}"; then
            best_lines="$mid"
            cp "$tmp_candidate" "$output_file"
            hi=$((mid - 1))
        else
            lo=$((mid + 1))
        fi
    done

    rm -f "$tmp_candidate"
    echo "$best_lines"
    return 0
}

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
}

run_logged_rust() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        cd "$RUST_DIR"
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
}

if [[ "$PARSE_FULL_MODE" != "auto" && "$PARSE_FULL_MODE" != "0" && "$PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ -n "$LRM_PROFILE_OVERRIDE" && -n "$LRM_PROFILES_OVERRIDE" ]]; then
    echo "error: set either PGEN_SV_STIMULI_QUALITY_LRM_PROFILE or PGEN_SV_STIMULI_QUALITY_LRM_PROFILES, not both" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

require_tool jq
require_tool perl
require_file "$CONTRACT_FILE"
require_file "$EBNF_TO_JSON"

contract_version="$(jq -er '.version | numbers' "$CONTRACT_FILE")"
grammar_name="$(jq -er '.grammar_name | strings' "$CONTRACT_FILE")"
ebnf_path_rel="$(jq -er '.ebnf_path | strings' "$CONTRACT_FILE")"
default_sample_count="$(jq -er '.sample_count | numbers' "$CONTRACT_FILE")"
default_seed_base="$(jq -er '.seed_base | numbers' "$CONTRACT_FILE")"
default_lrm_profile="$(jq -er '(.lrm_profiles.default_profile // "2017") | strings' "$CONTRACT_FILE")"
supported_lrm_profiles_csv="$(jq -er '(.lrm_profiles.supported_profiles // ["2017","2023"]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"
required_lrm_profiles_csv="$(jq -er '(.lrm_profiles.required_profiles // [(.lrm_profiles.default_profile // "2017")]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"
default_stimuli_mode="$(jq -er '(.stimuli_modes.default_mode // "sv_file") | strings' "$CONTRACT_FILE")"
supported_stimuli_modes_csv="$(jq -er '(.stimuli_modes.supported_modes // ["sv_file","sv_snippet","sv_pp_file","sv_pp_snippet"]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"
closed_loop_enabled="$(jq -er 'if (.closed_loop.enabled // true) then 1 else 0 end' "$CONTRACT_FILE")"
gap_report_threshold="$(jq -er '(.closed_loop.gap_report_threshold // 1) | numbers' "$CONTRACT_FILE")"
target_max_attempts="$(jq -er '(.closed_loop.target_max_attempts // 5000) | numbers' "$CONTRACT_FILE")"
require_non_increasing_target_debt="$(jq -er 'if (.closed_loop.require_non_increasing_target_debt // true) then 1 else 0 end' "$CONTRACT_FILE")"
failure_replay_enabled="$(jq -er 'if (.failure_replay.enabled // true) then 1 else 0 end' "$CONTRACT_FILE")"
shrink_semantic_failures="$(jq -er 'if (.failure_replay.shrink_semantic_failures // true) then 1 else 0 end' "$CONTRACT_FILE")"
shrink_parse_full_failures="$(jq -er 'if (.failure_replay.shrink_parse_full_failures // true) then 1 else 0 end' "$CONTRACT_FILE")"
failure_shrink_max_iterations="$(jq -er '(.failure_replay.shrink_max_iterations // 24) | numbers' "$CONTRACT_FILE")"

sample_count="${SAMPLE_COUNT_OVERRIDE:-$default_sample_count}"
seed_base="${SEED_BASE_OVERRIDE:-$default_seed_base}"
replay_sample_count="$(jq -er --argjson fallback "$sample_count" '(.closed_loop.replay_sample_count // $fallback) | numbers' "$CONTRACT_FILE")"
stimuli_mode="${STIMULI_MODE_OVERRIDE:-$default_stimuli_mode}"

if ! [[ "$sample_count" =~ ^[0-9]+$ ]] || [[ "$sample_count" -lt 1 ]]; then
    echo "error: sample_count must be an integer >= 1 (effective value: '$sample_count')" >&2
    exit 2
fi
if ! [[ "$seed_base" =~ ^[0-9]+$ ]]; then
    echo "error: seed_base must be an integer >= 0 (effective value: '$seed_base')" >&2
    exit 2
fi
if ! [[ "$gap_report_threshold" =~ ^[0-9]+$ ]] || [[ "$gap_report_threshold" -lt 1 ]]; then
    echo "error: closed_loop.gap_report_threshold must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$target_max_attempts" =~ ^[0-9]+$ ]] || [[ "$target_max_attempts" -lt 1 ]]; then
    echo "error: closed_loop.target_max_attempts must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$replay_sample_count" =~ ^[0-9]+$ ]] || [[ "$replay_sample_count" -lt 1 ]]; then
    echo "error: closed_loop.replay_sample_count must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$failure_shrink_max_iterations" =~ ^[0-9]+$ ]] || [[ "$failure_shrink_max_iterations" -lt 1 ]]; then
    echo "error: failure_replay.shrink_max_iterations must be an integer >= 1" >&2
    exit 2
fi

declare -a supported_stimuli_modes=()
declare -A supported_stimuli_modes_map=()
IFS=',' read -r -a _supported_modes_raw <<< "$supported_stimuli_modes_csv"
for _m in "${_supported_modes_raw[@]}"; do
    mode="$(echo "$_m" | tr -d '[:space:]')"
    if [[ -n "$mode" ]]; then
        supported_stimuli_modes+=("$mode")
        supported_stimuli_modes_map["$mode"]=1
    fi
done
if [[ "${#supported_stimuli_modes[@]}" -eq 0 ]]; then
    echo "error: no supported stimuli modes defined in contract" >&2
    exit 2
fi
if [[ -z "${supported_stimuli_modes_map[$stimuli_mode]:-}" ]]; then
    echo "error: unsupported stimuli mode '$stimuli_mode' (supported: ${supported_stimuli_modes[*]})" >&2
    exit 2
fi

mode_entry_rule="$(jq -er --arg mode "$stimuli_mode" '(.stimuli_modes.profiles[$mode].entry_rule // (if ($mode == "sv_snippet" or $mode == "sv_pp_snippet") then "source_item" else "systemverilog_file" end)) | strings' "$CONTRACT_FILE")"
mode_closed_loop_enabled="$(jq -er --arg mode "$stimuli_mode" 'if (.stimuli_modes.profiles[$mode].closed_loop_enabled // ($mode != "sv_snippet" and $mode != "sv_pp_snippet")) then 1 else 0 end' "$CONTRACT_FILE")"
mode_parse_full_eligible="$(jq -er --arg mode "$stimuli_mode" 'if (.stimuli_modes.profiles[$mode].parse_full_eligible // ($mode == "sv_file" or $mode == "sv_pp_file")) then 1 else 0 end' "$CONTRACT_FILE")"
mode_recovery_stimuli_mode="$(jq -er --arg mode "$stimuli_mode" '(.stimuli_modes.profiles[$mode].recovery_stimuli_mode // "baseline") | strings' "$CONTRACT_FILE")"

if [[ "$mode_recovery_stimuli_mode" != "baseline" && "$mode_recovery_stimuli_mode" != "recovery_biased" && "$mode_recovery_stimuli_mode" != "near_sync_negative" ]]; then
    echo "error: unsupported recovery stimuli mode '$mode_recovery_stimuli_mode' for stimuli mode '$stimuli_mode' (supported: baseline, recovery_biased, near_sync_negative)" >&2
    exit 2
fi

closed_loop_effective_enabled=0
if [[ "$closed_loop_enabled" -eq 1 && "$mode_closed_loop_enabled" -eq 1 ]]; then
    closed_loop_effective_enabled=1
fi

include_max_depth="$(jq -er '.preprocess.include_max_depth | numbers' "$CONTRACT_FILE")"
include_path_policy="$(jq -er '.preprocess.include_path_policy | strings' "$CONTRACT_FILE")"
macro_redefine_policy="$(jq -er '.preprocess.macro_redefine_policy | strings' "$CONTRACT_FILE")"
conditional_symbol_policy="$(jq -er '.preprocess.conditional_symbol_policy | strings' "$CONTRACT_FILE")"
conditional_expr_policy="$(jq -er '.preprocess.conditional_expr_policy | strings' "$CONTRACT_FILE")"
strict_warning_codes="$(jq -er '.preprocess.strict_warning_codes | strings' "$CONTRACT_FILE")"

require_nonempty_preprocessed_output="$(jq -er 'if .semantic_baseline.require_nonempty_preprocessed_output then 1 else 0 end' "$CONTRACT_FILE")"
require_no_preprocess_errors="$(jq -er 'if .semantic_baseline.require_no_preprocess_errors then 1 else 0 end' "$CONTRACT_FILE")"
require_balanced_structural_keywords="$(jq -er 'if .semantic_baseline.require_balanced_structural_keywords then 1 else 0 end' "$CONTRACT_FILE")"
require_unique_named_port_bindings="$(jq -er 'if .semantic_baseline.require_unique_named_port_bindings then 1 else 0 end' "$CONTRACT_FILE")"
require_port_binding_legality_basic="$(jq -er 'if (.semantic_baseline.require_port_binding_legality_basic // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_declared_identifiers_before_use="$(jq -er 'if (.semantic_baseline.require_declared_identifiers_before_use // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_package_qualification_resolution="$(jq -er 'if (.semantic_baseline.require_package_qualification_resolution // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_width_compatibility_simple="$(jq -er 'if (.semantic_baseline.require_width_compatibility_simple // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_context_legality_basic="$(jq -er 'if (.semantic_baseline.require_context_legality_basic // false) then 1 else 0 end' "$CONTRACT_FILE")"

require_nonempty_preprocessed_output="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_nonempty_preprocessed_output" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_nonempty_preprocessed_output // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_nonempty_preprocessed_output then 1 else 0 end) end' "$CONTRACT_FILE")"
require_no_preprocess_errors="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_no_preprocess_errors" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_no_preprocess_errors // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_no_preprocess_errors then 1 else 0 end) end' "$CONTRACT_FILE")"
require_balanced_structural_keywords="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_balanced_structural_keywords" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_balanced_structural_keywords // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_balanced_structural_keywords then 1 else 0 end) end' "$CONTRACT_FILE")"
require_unique_named_port_bindings="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_unique_named_port_bindings" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_unique_named_port_bindings // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_unique_named_port_bindings then 1 else 0 end) end' "$CONTRACT_FILE")"
require_port_binding_legality_basic="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_port_binding_legality_basic" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_port_binding_legality_basic // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_port_binding_legality_basic then 1 else 0 end) end' "$CONTRACT_FILE")"
require_declared_identifiers_before_use="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_declared_identifiers_before_use" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_declared_identifiers_before_use // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_declared_identifiers_before_use then 1 else 0 end) end' "$CONTRACT_FILE")"
require_package_qualification_resolution="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_package_qualification_resolution" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_package_qualification_resolution // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_package_qualification_resolution then 1 else 0 end) end' "$CONTRACT_FILE")"
require_width_compatibility_simple="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_width_compatibility_simple" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_width_compatibility_simple // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_width_compatibility_simple then 1 else 0 end) end' "$CONTRACT_FILE")"
require_context_legality_basic="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_context_legality_basic" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_context_legality_basic // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_context_legality_basic then 1 else 0 end) end' "$CONTRACT_FILE")"

if ! [[ "$include_max_depth" =~ ^[0-9]+$ ]] || [[ "$include_max_depth" -lt 1 ]]; then
    echo "error: preprocess.include_max_depth must be an integer >= 1" >&2
    exit 2
fi

grammar_file="$ROOT_DIR/$ebnf_path_rel"
grammar_json="$WORK_DIR/${grammar_name}.json"
parser_out="$WORK_DIR/${grammar_name}_parser.rs"

require_file "$grammar_file"

profiles_csv="$required_lrm_profiles_csv"
if [[ -n "$LRM_PROFILE_OVERRIDE" ]]; then
    profiles_csv="$LRM_PROFILE_OVERRIDE"
elif [[ -n "$LRM_PROFILES_OVERRIDE" ]]; then
    profiles_csv="$LRM_PROFILES_OVERRIDE"
fi

declare -a supported_profiles=()
declare -A supported_profiles_map=()
IFS=',' read -r -a _supported_raw <<< "$supported_lrm_profiles_csv"
for _p in "${_supported_raw[@]}"; do
    p="$(echo "$_p" | tr -d '[:space:]')"
    if [[ -n "$p" ]]; then
        supported_profiles+=("$p")
        supported_profiles_map["$p"]=1
    fi
done
if [[ "${#supported_profiles[@]}" -eq 0 ]]; then
    echo "error: no supported lrm profiles defined in contract" >&2
    exit 2
fi

declare -a run_profiles=()
declare -A run_profiles_map=()
IFS=',' read -r -a _run_raw <<< "$profiles_csv"
for _p in "${_run_raw[@]}"; do
    p="$(echo "$_p" | tr -d '[:space:]')"
    [[ -n "$p" ]] || continue
    if [[ -n "${run_profiles_map[$p]:-}" ]]; then
        continue
    fi
    if [[ -z "${supported_profiles_map[$p]:-}" ]]; then
        echo "error: unsupported lrm profile '$p' (supported: ${supported_profiles[*]})" >&2
        exit 2
    fi
    run_profiles+=("$p")
    run_profiles_map["$p"]=1
done
if [[ "${#run_profiles[@]}" -eq 0 ]]; then
    echo "error: no runnable lrm profile selected" >&2
    exit 2
fi

echo "==> SystemVerilog stimuli quality gate"
echo "state_dir: $STATE_DIR"
echo "contract_file: $CONTRACT_FILE"
echo "contract_version: $contract_version"
echo "grammar_name: $grammar_name"
echo "grammar_file: $grammar_file"
echo "sample_count: $sample_count"
echo "seed_base: $seed_base"
echo "parse_full_mode: $PARSE_FULL_MODE"
echo "lrm_default_profile: $default_lrm_profile"
echo "lrm_supported_profiles: ${supported_profiles[*]}"
echo "lrm_run_profiles: ${run_profiles[*]}"
echo "stimuli_mode: $stimuli_mode"
echo "stimuli_mode_entry_rule: $mode_entry_rule"
echo "stimuli_mode_closed_loop_enabled: $mode_closed_loop_enabled"
echo "stimuli_mode_parse_full_eligible: $mode_parse_full_eligible"
echo "stimuli_mode_recovery_stimuli_mode: $mode_recovery_stimuli_mode"
echo "closed_loop_enabled: $closed_loop_enabled"
echo "closed_loop_effective_enabled: $closed_loop_effective_enabled"
echo "closed_loop_gap_report_threshold: $gap_report_threshold"
echo "closed_loop_target_max_attempts: $target_max_attempts"
echo "closed_loop_replay_sample_count: $replay_sample_count"
echo "closed_loop_require_non_increasing_target_debt: $require_non_increasing_target_debt"
echo "failure_replay_enabled: $failure_replay_enabled"
echo "failure_replay_shrink_semantic_failures: $shrink_semantic_failures"
echo "failure_replay_shrink_parse_full_failures: $shrink_parse_full_failures"
echo "failure_replay_shrink_max_iterations: $failure_shrink_max_iterations"
echo "semantic_require_port_binding_legality_basic: $require_port_binding_legality_basic"
echo "semantic_require_declared_identifiers_before_use: $require_declared_identifiers_before_use"
echo "semantic_require_package_qualification_resolution: $require_package_qualification_resolution"
echo "semantic_require_width_compatibility_simple: $require_width_compatibility_simple"
echo "semantic_require_context_legality_basic: $require_context_legality_basic"

echo "profile,sample,seed,coverage_gap_initial,gap_replay,stimuli_generate,preprocess,semantic_validate,parse_full,warnings,errors,status,notes" >"$SUMMARY_CSV"

run_logged_rust "build_ast_pipeline_for_sv_generation" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi
run_logged "frontend_ebnf_to_json" \
    perl "$EBNF_TO_JSON" --pretty --quiet "$grammar_file" -o "$grammar_json"
require_nonempty_file "$grammar_json"

run_logged "generate_sv_parser" \
    "$AST_PIPELINE_BIN" "$grammar_json" \
    --generate-parser \
    --eliminate-left-recursion \
    --output "$parser_out"
require_nonempty_file "$parser_out"

run_logged_rust "build_parseability_probe_with_systemverilog_adapter" \
    env PGEN_SYSTEMVERILOG_PARSER_PATH="$parser_out" \
    cargo build --features generated_parsers --bin parseability_probe
if [[ ! -x "$PARSE_PROBE_BIN" ]]; then
    echo "error: parseability_probe binary is missing at '$PARSE_PROBE_BIN' after adapter build" >&2
    exit 1
fi

parse_full_supported=0
probe_log="$LOG_DIR/probe_parse_full_support.log"
echo "==> probe_parse_full_support"
if "$PARSE_PROBE_BIN" --supports "$grammar_name" >"$probe_log" 2>&1; then
    echo "    ok (${probe_log})"
    parse_full_supported=1
else
    echo "    skip (${probe_log})"
fi

parse_full_enabled=0
parse_full_effective="disabled"
if [[ "$PARSE_FULL_MODE" == "0" ]]; then
    parse_full_enabled=0
    parse_full_effective="disabled_by_mode"
elif [[ "$PARSE_FULL_MODE" == "1" ]]; then
    if [[ "$mode_parse_full_eligible" -eq 0 ]]; then
        echo "error: parse_full mode is strict (1) but stimuli mode '$stimuli_mode' is not parse_full-eligible" >&2
        exit 1
    fi
    if [[ "$parse_full_supported" -eq 0 ]]; then
        echo "error: parse_full mode is strict (1) but no generated parser adapter is registered for '$grammar_name'" >&2
        exit 1
    fi
    parse_full_enabled=1
    parse_full_effective="enabled"
else
    if [[ "$mode_parse_full_eligible" -eq 1 && "$parse_full_supported" -eq 1 ]]; then
        parse_full_enabled=1
        parse_full_effective="enabled"
    elif [[ "$mode_parse_full_eligible" -eq 0 ]]; then
        parse_full_enabled=0
        parse_full_effective="disabled_by_stimuli_mode"
    else
        parse_full_enabled=0
        parse_full_effective="unsupported_adapter"
    fi
fi

semantic_pass_count=0
parse_full_pass_count=0
parse_full_skip_count=0
parse_full_fail_count=0
closed_loop_profile_pass_count=0
closed_loop_profile_skip_count=0
closed_loop_initial_targets_total=0
closed_loop_replay_targets_total=0
closed_loop_initial_preprocess_warnings_total=0
closed_loop_initial_preprocess_errors_total=0
closed_loop_replay_preprocess_warnings_total=0
closed_loop_replay_preprocess_errors_total=0
total_warnings=0
total_errors=0
semantic_shrink_count=0
parse_full_shrink_count=0
profile_count="${#run_profiles[@]}"
total_samples=$((sample_count * profile_count))

for profile_idx in "${!run_profiles[@]}"; do
    lrm_profile="${run_profiles[$profile_idx]}"
    profile_seed_base=$((seed_base + (profile_idx * 1000000)))
    profile_key="${lrm_profile//[^A-Za-z0-9_]/_}"

    profile_closed_loop_initial_status="skip"
    profile_closed_loop_replay_status="skip"
    profile_closed_loop_note="closed-loop disabled by contract"
    if [[ "$closed_loop_effective_enabled" -eq 1 ]]; then
        closed_loop_initial_stimuli="$WORK_DIR/profile_${profile_key}_initial_stimuli.sv"
        closed_loop_initial_coverage="$WORK_DIR/profile_${profile_key}_initial_coverage.json"
        closed_loop_initial_gap_json="$WORK_DIR/profile_${profile_key}_initial_gap.json"
        closed_loop_initial_gap_text="$WORK_DIR/profile_${profile_key}_initial_gap.txt"
        closed_loop_replay_stimuli="$WORK_DIR/profile_${profile_key}_replay_stimuli.sv"
        closed_loop_replay_coverage="$WORK_DIR/profile_${profile_key}_replay_coverage.json"
        closed_loop_replay_gap_json="$WORK_DIR/profile_${profile_key}_replay_gap.json"
        closed_loop_replay_gap_text="$WORK_DIR/profile_${profile_key}_replay_gap.txt"
        closed_loop_replay_seed=$((profile_seed_base + 700000))

        run_logged "profile_${profile_key}_closed_loop_initial" \
            "$AST_PIPELINE_BIN" "$grammar_json" \
            --generate-stimuli \
            --count "$sample_count" \
            --seed "$profile_seed_base" \
            --entry-rule "$mode_entry_rule" \
            --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
            --output "$closed_loop_initial_stimuli" \
            --coverage-output "$closed_loop_initial_coverage" \
            --gap-report-json "$closed_loop_initial_gap_json" \
            --gap-report-text "$closed_loop_initial_gap_text" \
            --gap-report-threshold "$gap_report_threshold"
        require_nonempty_file "$closed_loop_initial_stimuli"
        require_nonempty_file "$closed_loop_initial_coverage"
        require_nonempty_file "$closed_loop_initial_gap_json"
        require_nonempty_file "$closed_loop_initial_gap_text"
        initial_target_count="$(jq -er '(.targets // []) | length | numbers' "$closed_loop_initial_gap_json")"
        closed_loop_initial_targets_total=$((closed_loop_initial_targets_total + initial_target_count))
        profile_closed_loop_initial_status="pass"

        run_logged "profile_${profile_key}_closed_loop_replay" \
            "$AST_PIPELINE_BIN" "$grammar_json" \
            --generate-stimuli \
            --count "$replay_sample_count" \
            --seed "$closed_loop_replay_seed" \
            --entry-rule "$mode_entry_rule" \
            --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
            --output "$closed_loop_replay_stimuli" \
            --coverage-output "$closed_loop_replay_coverage" \
            --gap-report-json "$closed_loop_replay_gap_json" \
            --gap-report-text "$closed_loop_replay_gap_text" \
            --gap-report-threshold "$gap_report_threshold" \
            --target-max-attempts "$target_max_attempts" \
            --target-report-input "$closed_loop_initial_gap_json"
        require_nonempty_file "$closed_loop_replay_stimuli"
        require_nonempty_file "$closed_loop_replay_coverage"
        require_nonempty_file "$closed_loop_replay_gap_json"
        require_nonempty_file "$closed_loop_replay_gap_text"
        replay_target_count="$(jq -er '(.targets // []) | length | numbers' "$closed_loop_replay_gap_json")"
        closed_loop_replay_targets_total=$((closed_loop_replay_targets_total + replay_target_count))
        profile_closed_loop_replay_status="pass"

        closed_loop_initial_preprocessed="$WORK_DIR/profile_${profile_key}_initial.preprocessed.sv"
        closed_loop_initial_diagnostics="$WORK_DIR/profile_${profile_key}_initial.diagnostics.json"
        closed_loop_replay_preprocessed="$WORK_DIR/profile_${profile_key}_replay.preprocessed.sv"
        closed_loop_replay_diagnostics="$WORK_DIR/profile_${profile_key}_replay.diagnostics.json"

        run_logged "profile_${profile_key}_closed_loop_initial_preprocess" \
            "$AST_PIPELINE_BIN" "$closed_loop_initial_stimuli" \
            --preprocess-systemverilog \
            --output "$closed_loop_initial_preprocessed" \
            --sv-diagnostics-json "$closed_loop_initial_diagnostics" \
            --sv-include-max-depth "$include_max_depth" \
            --sv-include-path-policy "$include_path_policy" \
            --sv-macro-redefine-policy "$macro_redefine_policy" \
            --sv-conditional-symbol-policy "$conditional_symbol_policy" \
            --sv-conditional-expr-policy "$conditional_expr_policy" \
            --sv-strict-warning-codes "$strict_warning_codes"
        run_logged "profile_${profile_key}_closed_loop_replay_preprocess" \
            "$AST_PIPELINE_BIN" "$closed_loop_replay_stimuli" \
            --preprocess-systemverilog \
            --output "$closed_loop_replay_preprocessed" \
            --sv-diagnostics-json "$closed_loop_replay_diagnostics" \
            --sv-include-max-depth "$include_max_depth" \
            --sv-include-path-policy "$include_path_policy" \
            --sv-macro-redefine-policy "$macro_redefine_policy" \
            --sv-conditional-symbol-policy "$conditional_symbol_policy" \
            --sv-conditional-expr-policy "$conditional_expr_policy" \
            --sv-strict-warning-codes "$strict_warning_codes"

        require_file "$closed_loop_initial_diagnostics"
        require_file "$closed_loop_replay_diagnostics"
        initial_preprocess_warnings="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$closed_loop_initial_diagnostics")"
        initial_preprocess_errors="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$closed_loop_initial_diagnostics")"
        replay_preprocess_warnings="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$closed_loop_replay_diagnostics")"
        replay_preprocess_errors="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$closed_loop_replay_diagnostics")"
        closed_loop_initial_preprocess_warnings_total=$((closed_loop_initial_preprocess_warnings_total + initial_preprocess_warnings))
        closed_loop_initial_preprocess_errors_total=$((closed_loop_initial_preprocess_errors_total + initial_preprocess_errors))
        closed_loop_replay_preprocess_warnings_total=$((closed_loop_replay_preprocess_warnings_total + replay_preprocess_warnings))
        closed_loop_replay_preprocess_errors_total=$((closed_loop_replay_preprocess_errors_total + replay_preprocess_errors))
        profile_closed_loop_note="initial_targets=${initial_target_count}; replay_targets=${replay_target_count}; initial_preprocess_errors=${initial_preprocess_errors}; replay_preprocess_errors=${replay_preprocess_errors}"

        if [[ "$require_non_increasing_target_debt" -eq 1 ]] && (( replay_target_count > initial_target_count )); then
            echo "error: closed-loop replay increased target debt for profile '${lrm_profile}' (${initial_target_count} -> ${replay_target_count})" >&2
            exit 1
        fi
        if [[ "$require_non_increasing_target_debt" -eq 1 ]] && (( replay_preprocess_errors > initial_preprocess_errors )); then
            echo "error: closed-loop replay increased preprocess error debt for profile '${lrm_profile}' (${initial_preprocess_errors} -> ${replay_preprocess_errors})" >&2
            exit 1
        fi

        closed_loop_profile_pass_count=$((closed_loop_profile_pass_count + 1))
    else
        closed_loop_profile_skip_count=$((closed_loop_profile_skip_count + 1))
    fi

    for ((idx = 0; idx < sample_count; idx++)); do
        seed=$((profile_seed_base + idx))
        sample_file="$WORK_DIR/sample_${profile_key}_${idx}.sv"
        preprocessed_file="$WORK_DIR/sample_${profile_key}_${idx}.preprocessed.sv"
        diagnostics_json="$WORK_DIR/sample_${profile_key}_${idx}.diagnostics.json"

        run_logged "sample_${profile_key}_${idx}_generate_stimulus" \
            "$AST_PIPELINE_BIN" "$grammar_json" \
            --generate-stimuli \
            --count 1 \
            --seed "$seed" \
            --entry-rule "$mode_entry_rule" \
            --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
            --output "$sample_file"
        require_nonempty_file "$sample_file"

        run_logged "sample_${profile_key}_${idx}_preprocess" \
            "$AST_PIPELINE_BIN" "$sample_file" \
            --preprocess-systemverilog \
            --output "$preprocessed_file" \
            --sv-diagnostics-json "$diagnostics_json" \
            --sv-include-max-depth "$include_max_depth" \
            --sv-include-path-policy "$include_path_policy" \
            --sv-macro-redefine-policy "$macro_redefine_policy" \
            --sv-conditional-symbol-policy "$conditional_symbol_policy" \
            --sv-conditional-expr-policy "$conditional_expr_policy" \
            --sv-strict-warning-codes "$strict_warning_codes"

        require_file "$diagnostics_json"
        warning_count="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$diagnostics_json")"
        error_count="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$diagnostics_json")"
        total_warnings=$((total_warnings + warning_count))
        total_errors=$((total_errors + error_count))

        semantic_status="pass"
        semantic_note="baseline semantic validation passed"
        if ! evaluate_semantic_baseline "$preprocessed_file" "$error_count"; then
            semantic_status="fail"
            semantic_note="$SEMANTIC_EVAL_NOTE"
        fi

        if [[ "$semantic_status" != "pass" ]]; then
            if [[ "$failure_replay_enabled" -eq 1 ]] && [[ "$shrink_semantic_failures" -eq 1 ]]; then
                semantic_shrink_file="$WORK_DIR/sample_${profile_key}_${idx}.semantic.shrunk.sv"
                semantic_shrink_lines="$(deterministic_prefix_shrink "$preprocessed_file" "$semantic_shrink_file" "$failure_shrink_max_iterations" semantic_failure_predicate "$error_count")"
                semantic_note="${semantic_note}; shrunk_failure=${semantic_shrink_file}; shrunk_lines=${semantic_shrink_lines}"
                semantic_shrink_count=$((semantic_shrink_count + 1))
            fi
            echo "${lrm_profile},${idx},${seed},${profile_closed_loop_initial_status},${profile_closed_loop_replay_status},pass,pass,fail,skip,${warning_count},${error_count},fail,$(csv_sanitize "$semantic_note")" >>"$SUMMARY_CSV"
            echo "error: semantic baseline validation failed for profile '${lrm_profile}' sample_${idx}: ${semantic_note}" >&2
            exit 1
        fi
        semantic_pass_count=$((semantic_pass_count + 1))

        parse_status="skip"
        parse_note="parse_full stage skipped"
        if [[ "$parse_full_enabled" -eq 1 ]]; then
            parse_log="$LOG_DIR/sample_${profile_key}_${idx}_parse_full.log"
            echo "==> sample_${profile_key}_${idx}_parse_full"
            if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$preprocessed_file" >"$parse_log" 2>&1; then
                echo "    ok (${parse_log})"
                parse_status="pass"
                parse_note="parse_full accepted preprocessed sample"
                parse_full_pass_count=$((parse_full_pass_count + 1))
            else
                parse_status="fail"
                parse_note="parse_full rejected preprocessed sample"
                parse_full_fail_count=$((parse_full_fail_count + 1))
                if [[ "$failure_replay_enabled" -eq 1 ]] && [[ "$shrink_parse_full_failures" -eq 1 ]]; then
                    parse_shrink_file="$WORK_DIR/sample_${profile_key}_${idx}.parse_full.shrunk.sv"
                    parse_shrink_lines="$(deterministic_prefix_shrink "$preprocessed_file" "$parse_shrink_file" "$failure_shrink_max_iterations" parse_full_failure_predicate)"
                    parse_note="${parse_note}; shrunk_failure=${parse_shrink_file}; shrunk_lines=${parse_shrink_lines}"
                    parse_full_shrink_count=$((parse_full_shrink_count + 1))
                fi
                if [[ "$PARSE_FULL_MODE" == "1" ]]; then
                    echo "    fail (${parse_log})" >&2
                    tail -n 80 "$parse_log" >&2 || true
                    echo "error: strict parse_full mode requires all samples to pass parse_full" >&2
                    exit 1
                fi
                echo "    soft-fail (${parse_log})"
            fi
        else
            parse_full_skip_count=$((parse_full_skip_count + 1))
            parse_note="parse_full unavailable (${parse_full_effective})"
        fi

        echo "${lrm_profile},${idx},${seed},${profile_closed_loop_initial_status},${profile_closed_loop_replay_status},pass,pass,${semantic_status},${parse_status},${warning_count},${error_count},pass,$(csv_sanitize "$parse_note")" >>"$SUMMARY_CSV"
    done
done

{
    echo "PGEN SV Stimuli Quality Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "contract_file: $CONTRACT_FILE"
    echo "grammar_name: $grammar_name"
    echo "sample_count: $sample_count"
    echo "profile_count: $profile_count"
    echo "run_profiles: ${run_profiles[*]}"
    echo "seed_base: $seed_base"
    echo "closed_loop_enabled: $closed_loop_enabled"
    echo "closed_loop_gap_report_threshold: $gap_report_threshold"
    echo "closed_loop_target_max_attempts: $target_max_attempts"
    echo "closed_loop_replay_sample_count: $replay_sample_count"
    echo "closed_loop_profiles_passed: $closed_loop_profile_pass_count/$profile_count"
    echo "closed_loop_profiles_skipped: $closed_loop_profile_skip_count/$profile_count"
    echo "closed_loop_initial_targets_total: $closed_loop_initial_targets_total"
    echo "closed_loop_replay_targets_total: $closed_loop_replay_targets_total"
    echo "closed_loop_initial_preprocess_warnings_total: $closed_loop_initial_preprocess_warnings_total"
    echo "closed_loop_initial_preprocess_errors_total: $closed_loop_initial_preprocess_errors_total"
    echo "closed_loop_replay_preprocess_warnings_total: $closed_loop_replay_preprocess_warnings_total"
    echo "closed_loop_replay_preprocess_errors_total: $closed_loop_replay_preprocess_errors_total"
    echo "parse_full_mode: $PARSE_FULL_MODE"
    echo "parse_full_effective: $parse_full_effective"
    echo "semantic_baseline_passes: $semantic_pass_count/$total_samples"
    echo "parse_full_passes: $parse_full_pass_count/$total_samples"
    echo "parse_full_failures: $parse_full_fail_count"
    echo "parse_full_skips: $parse_full_skip_count"
    echo "semantic_failures_shrunk: $semantic_shrink_count"
    echo "parse_full_failures_shrunk: $parse_full_shrink_count"
    echo "total_warnings: $total_warnings"
    echo "total_errors: $total_errors"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ SV stimuli quality gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
