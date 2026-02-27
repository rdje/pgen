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
closed_loop_enabled="$(jq -er 'if (.closed_loop.enabled // true) then 1 else 0 end' "$CONTRACT_FILE")"
gap_report_threshold="$(jq -er '(.closed_loop.gap_report_threshold // 1) | numbers' "$CONTRACT_FILE")"
target_max_attempts="$(jq -er '(.closed_loop.target_max_attempts // 5000) | numbers' "$CONTRACT_FILE")"
require_non_increasing_target_debt="$(jq -er 'if (.closed_loop.require_non_increasing_target_debt // true) then 1 else 0 end' "$CONTRACT_FILE")"

sample_count="${SAMPLE_COUNT_OVERRIDE:-$default_sample_count}"
seed_base="${SEED_BASE_OVERRIDE:-$default_seed_base}"
replay_sample_count="$(jq -er --argjson fallback "$sample_count" '(.closed_loop.replay_sample_count // $fallback) | numbers' "$CONTRACT_FILE")"

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
require_declared_identifiers_before_use="$(jq -er 'if (.semantic_baseline.require_declared_identifiers_before_use // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_package_qualification_resolution="$(jq -er 'if (.semantic_baseline.require_package_qualification_resolution // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_width_compatibility_simple="$(jq -er 'if (.semantic_baseline.require_width_compatibility_simple // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_context_legality_basic="$(jq -er 'if (.semantic_baseline.require_context_legality_basic // false) then 1 else 0 end' "$CONTRACT_FILE")"

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
echo "closed_loop_enabled: $closed_loop_enabled"
echo "closed_loop_gap_report_threshold: $gap_report_threshold"
echo "closed_loop_target_max_attempts: $target_max_attempts"
echo "closed_loop_replay_sample_count: $replay_sample_count"
echo "closed_loop_require_non_increasing_target_debt: $require_non_increasing_target_debt"
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
    if [[ "$parse_full_supported" -eq 0 ]]; then
        echo "error: parse_full mode is strict (1) but no generated parser adapter is registered for '$grammar_name'" >&2
        exit 1
    fi
    parse_full_enabled=1
    parse_full_effective="enabled"
else
    if [[ "$parse_full_supported" -eq 1 ]]; then
        parse_full_enabled=1
        parse_full_effective="enabled"
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
total_warnings=0
total_errors=0
profile_count="${#run_profiles[@]}"
total_samples=$((sample_count * profile_count))

for profile_idx in "${!run_profiles[@]}"; do
    lrm_profile="${run_profiles[$profile_idx]}"
    profile_seed_base=$((seed_base + (profile_idx * 1000000)))
    profile_key="${lrm_profile//[^A-Za-z0-9_]/_}"

    profile_closed_loop_initial_status="skip"
    profile_closed_loop_replay_status="skip"
    profile_closed_loop_note="closed-loop disabled by contract"
    if [[ "$closed_loop_enabled" -eq 1 ]]; then
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
        profile_closed_loop_note="initial_targets=${initial_target_count}; replay_targets=${replay_target_count}"

        if [[ "$require_non_increasing_target_debt" -eq 1 ]] && (( replay_target_count > initial_target_count )); then
            echo "error: closed-loop replay increased target debt for profile '${lrm_profile}' (${initial_target_count} -> ${replay_target_count})" >&2
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

        if [[ "$require_nonempty_preprocessed_output" -eq 1 ]] && [[ ! -s "$preprocessed_file" ]]; then
            semantic_status="fail"
            semantic_note="preprocessed output is empty"
        fi
        if [[ "$require_no_preprocess_errors" -eq 1 ]] && (( error_count > 0 )); then
            semantic_status="fail"
            semantic_note="preprocessor diagnostics contain error severity entries"
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_balanced_structural_keywords" -eq 1 ]]; then
            if ! semantic_note="$(check_balanced_structural_keywords "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_unique_named_port_bindings" -eq 1 ]]; then
            if ! semantic_note="$(check_unique_named_port_bindings "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_declared_identifiers_before_use" -eq 1 ]]; then
            if ! semantic_note="$(check_declared_identifiers_before_use "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_package_qualification_resolution" -eq 1 ]]; then
            if ! semantic_note="$(check_package_qualification_resolution "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_width_compatibility_simple" -eq 1 ]]; then
            if ! semantic_note="$(check_width_compatibility_simple "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_context_legality_basic" -eq 1 ]]; then
            if ! semantic_note="$(check_context_legality_basic "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi

        if [[ "$semantic_status" != "pass" ]]; then
            echo "${lrm_profile},${idx},${seed},${profile_closed_loop_initial_status},${profile_closed_loop_replay_status},pass,pass,fail,skip,${warning_count},${error_count},fail,${semantic_note}" >>"$SUMMARY_CSV"
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

        echo "${lrm_profile},${idx},${seed},${profile_closed_loop_initial_status},${profile_closed_loop_replay_status},pass,pass,${semantic_status},${parse_status},${warning_count},${error_count},pass,${parse_note}" >>"$SUMMARY_CSV"
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
    echo "parse_full_mode: $PARSE_FULL_MODE"
    echo "parse_full_effective: $parse_full_effective"
    echo "semantic_baseline_passes: $semantic_pass_count/$total_samples"
    echo "parse_full_passes: $parse_full_pass_count/$total_samples"
    echo "parse_full_failures: $parse_full_fail_count"
    echo "parse_full_skips: $parse_full_skip_count"
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
