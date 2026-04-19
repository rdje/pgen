#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_STIMULI_QUALITY_STATE_DIR:-$RUST_DIR/target/sv_stimuli_quality_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

CONTRACT_FILE="${PGEN_SV_STIMULI_QUALITY_CONTRACT:-$RUST_DIR/test_data/grammar_quality/systemverilog_core_v0_contract.json}"
PARSE_FULL_MODE="${PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE:-auto}"
PARSE_FULL_MIN_PASS_RATIO_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO:-}"
PARSE_FULL_ENFORCE_MIN_PASS_RATIO_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO:-}"
SAMPLE_COUNT_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_COUNT:-}"
SEED_BASE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_SEED_BASE:-}"
TARGET_MAX_ATTEMPTS_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS:-}"
PENDING_FRONTIER_EXTRA_STAGNATION_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_PENDING_FRONTIER_EXTRA_STAGNATION:-}"
CARGO_BUILD_JOBS_OVERRIDE="${PGEN_SV_STIMULI_CARGO_BUILD_JOBS:-}"
LRM_PROFILE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_LRM_PROFILE:-}"
LRM_PROFILES_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_LRM_PROFILES:-}"
STIMULI_MODE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_MODE:-}"
SEMANTIC_CLOSURE_MODE="${PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE:-0}"
DECLARED_SHADOW_MODE="${PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_MODE:-auto}"
DECLARED_SHADOW_PARSEABLE_ONLY="${PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_PARSEABLE_ONLY:-0}"
DECLARED_IDENTIFIER_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_DECLARED_IDENTIFIER_SUITE:-}"
ENFORCE_DECLARED_IDENTIFIER_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_ENFORCE_DECLARED_IDENTIFIER_SUITE:-}"
WIDTH_COMPAT_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_WIDTH_COMPAT_SUITE:-}"
ENFORCE_WIDTH_COMPAT_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_ENFORCE_WIDTH_COMPAT_SUITE:-}"
PORT_BINDING_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_PORT_BINDING_SUITE:-}"
ENFORCE_PORT_BINDING_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_ENFORCE_PORT_BINDING_SUITE:-}"
PACKAGE_QUAL_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_PACKAGE_QUAL_SUITE:-}"
ENFORCE_PACKAGE_QUAL_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_ENFORCE_PACKAGE_QUAL_SUITE:-}"
CONTEXT_LEGALITY_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_CONTEXT_LEGALITY_SUITE:-}"
ENFORCE_CONTEXT_LEGALITY_SUITE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_ENFORCE_CONTEXT_LEGALITY_SUITE:-}"
DIFF_MODE="${PGEN_SV_STIMULI_DIFF_MODE:-auto}"
DIFF_MAX_SAMPLES="${PGEN_SV_STIMULI_DIFF_MAX_SAMPLES:-8}"
DIFF_REFERENCE_RUNNER="${PGEN_SV_STIMULI_REFERENCE_RUNNER:-}"
PERF_BUDGET_MODE="${PGEN_SV_STIMULI_PERF_BUDGET_MODE:-auto}"
REALISTIC_CORPUS_MODE="${PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE:-auto}"
REALISTIC_CORPUS_OVERRIDE="${PGEN_SV_STIMULI_REALISTIC_CORPUS:-}"
REALISTIC_CORPUS_MAX_CASES="${PGEN_SV_STIMULI_REALISTIC_CORPUS_MAX_CASES:-0}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"

declared_identifier_suite_status="skip"
declared_identifier_suite_total=0
declared_identifier_suite_passed=0
declared_identifier_suite_failed=0
width_compat_suite_status="skip"
width_compat_suite_total=0
width_compat_suite_passed=0
width_compat_suite_failed=0
port_binding_suite_status="skip"
port_binding_suite_total=0
port_binding_suite_passed=0
port_binding_suite_failed=0
package_qual_suite_status="skip"
package_qual_suite_total=0
package_qual_suite_passed=0
package_qual_suite_failed=0
context_legality_suite_status="skip"
context_legality_suite_total=0
context_legality_suite_passed=0
context_legality_suite_failed=0

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

parseability_summary_field_u64() {
    local path="$1"
    local field="$2"
    jq -er ".summary.${field} | numbers" "$path"
}

parseability_target_drive_field_u64() {
    local path="$1"
    local field="$2"
    jq -er "(.target_drive_validation.${field} // 0) | numbers" "$path"
}

parseability_acceptance_rate_percent() {
    local path="$1"
    local attempts accepted
    attempts="$(parseability_summary_field_u64 "$path" "attempts")"
    accepted="$(parseability_summary_field_u64 "$path" "accepted")"
    perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$accepted" "$attempts"
}

now_ms() {
    perl -MTime::HiRes=time -e 'printf "%.0f\n", time()*1000'
}

file_size_bytes() {
    perl -e 'my $f = shift; my $s = -s $f; print defined($s) ? $s : 0;' "$1"
}

resolve_path() {
    local raw="$1"
    if [[ "$raw" == /* ]]; then
        printf '%s\n' "$raw"
    else
        printf '%s\n' "$ROOT_DIR/$raw"
    fi
}

enforce_threshold_le() {
    local enabled="$1"
    local metric="$2"
    local value="$3"
    local max_allowed="$4"
    local context="$5"
    if [[ "$enabled" -eq 1 && "$max_allowed" -gt 0 && "$value" -gt "$max_allowed" ]]; then
        echo "error: ${metric} budget exceeded for ${context} (${value} > ${max_allowed})" >&2
        exit 1
    fi
}

canonicalize_json() {
    local source="$1"
    local target="$2"
    jq -S . "$source" >"$target"
}

assert_same_text() {
    local left="$1"
    local right="$2"
    local context="$3"
    if ! cmp -s "$left" "$right"; then
        echo "error: deterministic replay mismatch for $context" >&2
        diff -u "$left" "$right" | head -n 120 >&2 || true
        exit 1
    fi
}

assert_same_json() {
    local left="$1"
    local right="$2"
    local context="$3"
    local left_norm="${left}.norm.json"
    local right_norm="${right}.norm.json"
    canonicalize_json "$left" "$left_norm"
    canonicalize_json "$right" "$right_norm"
    assert_same_text "$left_norm" "$right_norm" "$context"
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
            $text =~ s/"(?:\\.|[^"\\])*"/""/gs;
            $text =~ s/\b(?:timeunit|timeprecision)\b[^;]*;//g;
            $text =~ s/`[A-Za-z_][A-Za-z0-9_]*(?:\([^\n]*\))?//g;

            my %keywords = map { $_ => 1 } qw(
                module endmodule interface endinterface program endprogram package endpackage class endclass
                begin end if else case endcase for foreach while repeat do
                always always_ff always_comb always_latch
                assign wire logic reg bit byte shortint int integer longint string chandle event time realtime
                signed unsigned input output inout ref var parameter localparam type typedef enum struct union
                packed unpacked function endfunction task endtask import export virtual static automatic const
                generate endgenerate genvar return break continue default initial final disable wait fork join
                join_any join_none checker endchecker primitive endprimitive
                this super null inside matches with let assert assume cover property sequence rand randc
                timeunit timeprecision clocking endclocking modport randsequence endsequence
                constraint solve before dist bins binsof cross covergroup endgroup coverpoint
                posedge negedge iff and or xor xnor not
            );

            my %declared;
            while ($text =~ /\b(?:module|interface|program|package|class|checker|primitive|function|task)\s+([A-Za-z_][A-Za-z0-9_]*)/g) {
                $declared{$1} = 1;
            }
            while ($text =~ /\bimport\s+([A-Za-z_][A-Za-z0-9_]*)::(?:\*|[A-Za-z_][A-Za-z0-9_]*)/g) {
                $declared{$1} = 1;
            }
            while ($text =~ /\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:=|[,;)])/g) {
                $declared{$1} = 1;
            }
            while ($text =~ /\b(?:input|output|inout|ref|var|logic|reg|wire|bit|byte|shortint|int|integer|longint|string|chandle|event|time|realtime)\b(?:\s+(?:signed|unsigned))?(?:\s*\[[^\]]+\])?\s+([^;]+)/g) {
                my $tail = $1;
                while ($tail =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                    my $id = $1;
                    next if $keywords{$id};
                    $declared{$id} = 1;
                }
            }
            while ($text =~ /\b(?:parameter|localparam|genvar|typedef)\b([^;]*);/g) {
                my $tail = $1;
                while ($tail =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                    my $id = $1;
                    next if $keywords{$id};
                    $declared{$id} = 1;
                }
            }
            while ($text =~ /\bfor\s*\(\s*(?:int|integer|longint|shortint|byte|bit)\s+([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                $declared{$1} = 1;
            }
            while ($text =~ /\bforeach\s*\(([^)]*)\)/g) {
                my $foreach_head = $1;
                while ($foreach_head =~ /\[\s*([A-Za-z_][A-Za-z0-9_]*)\s*\]/g) {
                    $declared{$1} = 1;
                }
            }
            while ($text =~ /\b([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:#\s*\([^;{}]*\)\s*)?\(/g) {
                my ($type_name, $inst_name) = ($1, $2);
                next if $keywords{$type_name};
                $declared{$type_name} = 1;
                $declared{$inst_name} = 1;
            }

            my %used;

            sub mark_chunk_uses {
                my ($chunk, $used_ref, $keywords_ref) = @_;
                return if !defined $chunk || $chunk eq "";

                # Remove common literal forms that can look like identifiers in noisy samples.
                $chunk =~ s/\b\d+\s*'\''\s*[sS]?[bBoOdDhH]\s*[0-9a-fA-F_xXzZ]+\b/ /g;
                $chunk =~ s/\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?(?:[a-zA-Z_][A-Za-z0-9_]*)\b/ /g;

                while ($chunk =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                    my $id = $1;
                    next if $keywords_ref->{$id};
                    next if $id =~ /^(?:x|z)$/i;
                    my $start = $-[1];
                    my $end = $+[1];
                    my $prev = $start > 0 ? substr($chunk, $start - 1, 1) : "";
                    my $prev2 = $start > 1 ? substr($chunk, $start - 2, 2) : "";
                    my $next = substr($chunk, $end, 1);
                    my $next2 = substr($chunk, $end, 2);
                    next if $prev =~ /[0-9]/;
                    next if $prev eq "." || $prev2 eq "::" || $prev2 eq "->";
                    next if $next2 eq "::";
                    next if $next eq ":";
                    next if $prev eq "`";
                    $used_ref->{$id} = 1;
                }
            }

            # Assignment/use contexts (structured-only to avoid lexical noise false positives).
            while ($text =~ /\b([A-Za-z_][A-Za-z0-9_]*)(\s*\[[^\]]+\])?\s*(?:<=|=(?!=))/g) {
                my $lhs = $1;
                my $lhs_index_expr = $2 // "";
                next if $keywords{$lhs};
                $used{$lhs} = 1;
                if ($lhs_index_expr ne "") {
                    mark_chunk_uses($lhs_index_expr, \%used, \%keywords);
                }
            }
            while ($text =~ /(?:<=|=(?!=))\s*([^;]+)/g) {
                mark_chunk_uses($1, \%used, \%keywords);
            }
            while ($text =~ /\b(?:if|while|assert|assume|cover)\s*\(([^)]*)\)/g) {
                mark_chunk_uses($1, \%used, \%keywords);
            }
            while ($text =~ /\bfor\s*\(([^;]*);([^;]*);([^\)]*)\)/g) {
                mark_chunk_uses("$1 $2 $3", \%used, \%keywords);
            }
            while ($text =~ /\bforeach\s*\(([^)]*)\)/g) {
                mark_chunk_uses($1, \%used, \%keywords);
            }
            while ($text =~ /\@\s*\(([^)]*)\)/g) {
                mark_chunk_uses($1, \%used, \%keywords);
            }
            while ($text =~ /\.\s*[A-Za-z_][A-Za-z0-9_]*\s*\(([^)]*)\)/g) {
                mark_chunk_uses($1, \%used, \%keywords);
            }

            for my $id (sort keys %used) {
                next if $declared{$id};
                next if $id =~ /^(?:[A-Z_][A-Z0-9_]*)$/;
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
            my %keywords = map { $_ => 1 } qw(
                signed unsigned wire logic reg bit input output inout ref var
            );

            while ($text =~ /\b(?:logic|reg|wire|bit)\b(?:\s+(?:signed|unsigned))?\s*\[\s*(\d+)\s*:\s*(\d+)\s*\]\s*([^;]+);/g) {
                my ($msb, $lsb, $tail) = ($1, $2, $3);
                my $width = $msb >= $lsb ? ($msb - $lsb + 1) : ($lsb - $msb + 1);
                while ($tail =~ /\b([A-Za-z_][A-Za-z0-9_]*)\b/g) {
                    my $name = $1;
                    next if $keywords{$name};
                    $width_of{$name} = $width;
                }
            }
            while ($text =~ /\b([A-Za-z_][A-Za-z0-9_]*)(?:\s*\[[^\]]+\])?\s*(?:<=|=)\s*(\d+)\s*'\''[bBoOdDhH][0-9a-fA-F_xXzZ]+/g) {
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
    local parse_status="${3:-pass}"
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
        if [[ "$require_declared_identifiers_parseable_only" -eq 1 && "$parse_status" != "pass" ]]; then
            SEMANTIC_EVAL_NOTE="declared identifier runtime check skipped because parse_full status is '${parse_status}' and parseable-only guard is enabled"
        else
            if ! SEMANTIC_EVAL_NOTE="$(check_declared_identifiers_before_use "$preprocessed_file")"; then
                return 1
            fi
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
    local parse_status="${3:-pass}"
    if evaluate_semantic_baseline "$candidate_file" "$preprocess_error_count" "$parse_status"; then
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

run_sv_cargo_build() {
    if [[ -n "$CARGO_BUILD_JOBS_OVERRIDE" ]]; then
        env CARGO_BUILD_JOBS="$CARGO_BUILD_JOBS_OVERRIDE" "$@"
    else
        "$@"
    fi
}

run_declared_identifier_contract_suite() {
    local suite_file="$1"
    local enforce="$2"
    local suite_summary_csv="$WORK_DIR/declared_identifier_contract_summary.csv"
    local idx=0
    local case_json

    declared_identifier_suite_status="skip"
    declared_identifier_suite_total=0
    declared_identifier_suite_passed=0
    declared_identifier_suite_failed=0
    echo "case,expect,actual,status,notes" >"$suite_summary_csv"

    if [[ "$enforce" -ne 1 ]]; then
        return 0
    fi

    if [[ -z "$suite_file" ]]; then
        echo "declared identifier contract suite is enforced but no suite path is configured"
        return 1
    fi
    require_file "$suite_file"

    while IFS= read -r case_json; do
        idx=$((idx + 1))
        declared_identifier_suite_total=$((declared_identifier_suite_total + 1))

        local case_name
        local case_expect_pass
        local case_input
        local case_file
        local case_actual_pass
        local case_check_note
        local case_expected_label
        local case_actual_label
        local case_status
        local case_note

        case_name="$(jq -er '.name | strings' <<<"$case_json")"
        case_expect_pass="$(jq -er 'if (.expect_pass // false) then 1 else 0 end' <<<"$case_json")"
        case_input="$(jq -er '.input | strings' <<<"$case_json")"
        case_file="$WORK_DIR/declared_identifier_case_${idx}.sv"

        printf '%s\n' "$case_input" >"$case_file"

        if case_check_note="$(check_declared_identifiers_before_use "$case_file" 2>&1)"; then
            case_actual_pass=1
            if [[ -z "$case_check_note" ]]; then
                case_check_note="declared identifier check passed"
            fi
        else
            case_actual_pass=0
            if [[ -z "$case_check_note" ]]; then
                case_check_note="declared identifier check failed"
            fi
        fi

        if [[ "$case_expect_pass" -eq 1 ]]; then
            case_expected_label="pass"
        else
            case_expected_label="fail"
        fi
        if [[ "$case_actual_pass" -eq 1 ]]; then
            case_actual_label="pass"
        else
            case_actual_label="fail"
        fi

        if [[ "$case_expect_pass" -eq "$case_actual_pass" ]]; then
            case_status="pass"
            case_note="$case_check_note"
            declared_identifier_suite_passed=$((declared_identifier_suite_passed + 1))
        else
            case_status="fail"
            case_note="$case_check_note"
            declared_identifier_suite_failed=$((declared_identifier_suite_failed + 1))
            echo "declared identifier contract mismatch: case='${case_name}' expected=${case_expected_label} actual=${case_actual_label}" >&2
        fi

        echo "${case_name},${case_expected_label},${case_actual_label},${case_status},$(csv_sanitize "$case_note")" >>"$suite_summary_csv"
    done < <(jq -c '.cases[]' "$suite_file")

    if (( declared_identifier_suite_total == 0 )); then
        echo "declared identifier contract suite has zero cases: $suite_file" >&2
        declared_identifier_suite_status="fail"
        return 1
    fi

    if (( declared_identifier_suite_failed > 0 )); then
        declared_identifier_suite_status="fail"
        echo "declared identifier contract suite failed: $declared_identifier_suite_failed/$declared_identifier_suite_total mismatches (summary: $suite_summary_csv)" >&2
        return 1
    fi

    declared_identifier_suite_status="pass"
    return 0
}

run_width_compatibility_contract_suite() {
    local suite_file="$1"
    local enforce="$2"
    local suite_summary_csv="$WORK_DIR/width_compatibility_contract_summary.csv"
    local idx=0
    local case_json

    width_compat_suite_status="skip"
    width_compat_suite_total=0
    width_compat_suite_passed=0
    width_compat_suite_failed=0
    echo "case,expect,actual,status,notes" >"$suite_summary_csv"

    if [[ "$enforce" -ne 1 ]]; then
        return 0
    fi

    if [[ -z "$suite_file" ]]; then
        echo "width compatibility contract suite is enforced but no suite path is configured"
        return 1
    fi
    require_file "$suite_file"

    while IFS= read -r case_json; do
        idx=$((idx + 1))
        width_compat_suite_total=$((width_compat_suite_total + 1))

        local case_name
        local case_expect_pass
        local case_input
        local case_file
        local case_actual_pass
        local case_check_note
        local case_expected_label
        local case_actual_label
        local case_status
        local case_note

        case_name="$(jq -er '.name | strings' <<<"$case_json")"
        case_expect_pass="$(jq -er 'if (.expect_pass // false) then 1 else 0 end' <<<"$case_json")"
        case_input="$(jq -er '.input | strings' <<<"$case_json")"
        case_file="$WORK_DIR/width_compat_case_${idx}.sv"

        printf '%s\n' "$case_input" >"$case_file"

        if case_check_note="$(check_width_compatibility_simple "$case_file" 2>&1)"; then
            case_actual_pass=1
            if [[ -z "$case_check_note" ]]; then
                case_check_note="width compatibility check passed"
            fi
        else
            case_actual_pass=0
            if [[ -z "$case_check_note" ]]; then
                case_check_note="width compatibility check failed"
            fi
        fi

        if [[ "$case_expect_pass" -eq 1 ]]; then
            case_expected_label="pass"
        else
            case_expected_label="fail"
        fi
        if [[ "$case_actual_pass" -eq 1 ]]; then
            case_actual_label="pass"
        else
            case_actual_label="fail"
        fi

        if [[ "$case_expect_pass" -eq "$case_actual_pass" ]]; then
            case_status="pass"
            case_note="$case_check_note"
            width_compat_suite_passed=$((width_compat_suite_passed + 1))
        else
            case_status="fail"
            case_note="$case_check_note"
            width_compat_suite_failed=$((width_compat_suite_failed + 1))
            echo "width compatibility contract mismatch: case='${case_name}' expected=${case_expected_label} actual=${case_actual_label}" >&2
        fi

        echo "${case_name},${case_expected_label},${case_actual_label},${case_status},$(csv_sanitize "$case_note")" >>"$suite_summary_csv"
    done < <(jq -c '.cases[]' "$suite_file")

    if (( width_compat_suite_total == 0 )); then
        echo "width compatibility contract suite has zero cases: $suite_file" >&2
        width_compat_suite_status="fail"
        return 1
    fi

    if (( width_compat_suite_failed > 0 )); then
        width_compat_suite_status="fail"
        echo "width compatibility contract suite failed: $width_compat_suite_failed/$width_compat_suite_total mismatches (summary: $suite_summary_csv)" >&2
        return 1
    fi

    width_compat_suite_status="pass"
    return 0
}

run_port_binding_legality_contract_suite() {
    local suite_file="$1"
    local enforce="$2"
    local suite_summary_csv="$WORK_DIR/port_binding_legality_contract_summary.csv"
    local idx=0
    local case_json

    port_binding_suite_status="skip"
    port_binding_suite_total=0
    port_binding_suite_passed=0
    port_binding_suite_failed=0
    echo "case,expect,actual,status,notes" >"$suite_summary_csv"

    if [[ "$enforce" -ne 1 ]]; then
        return 0
    fi

    if [[ -z "$suite_file" ]]; then
        echo "port binding legality contract suite is enforced but no suite path is configured"
        return 1
    fi
    require_file "$suite_file"

    while IFS= read -r case_json; do
        idx=$((idx + 1))
        port_binding_suite_total=$((port_binding_suite_total + 1))

        local case_name
        local case_expect_pass
        local case_input
        local case_file
        local case_actual_pass
        local case_check_note
        local case_expected_label
        local case_actual_label
        local case_status
        local case_note

        case_name="$(jq -er '.name | strings' <<<"$case_json")"
        case_expect_pass="$(jq -er 'if (.expect_pass // false) then 1 else 0 end' <<<"$case_json")"
        case_input="$(jq -er '.input | strings' <<<"$case_json")"
        case_file="$WORK_DIR/port_binding_case_${idx}.sv"

        printf '%s\n' "$case_input" >"$case_file"

        if case_check_note="$(check_port_binding_legality_basic "$case_file" 2>&1)"; then
            case_actual_pass=1
            if [[ -z "$case_check_note" ]]; then
                case_check_note="port binding legality check passed"
            fi
        else
            case_actual_pass=0
            if [[ -z "$case_check_note" ]]; then
                case_check_note="port binding legality check failed"
            fi
        fi

        if [[ "$case_expect_pass" -eq 1 ]]; then
            case_expected_label="pass"
        else
            case_expected_label="fail"
        fi
        if [[ "$case_actual_pass" -eq 1 ]]; then
            case_actual_label="pass"
        else
            case_actual_label="fail"
        fi

        if [[ "$case_expect_pass" -eq "$case_actual_pass" ]]; then
            case_status="pass"
            case_note="$case_check_note"
            port_binding_suite_passed=$((port_binding_suite_passed + 1))
        else
            case_status="fail"
            case_note="$case_check_note"
            port_binding_suite_failed=$((port_binding_suite_failed + 1))
            echo "port binding legality contract mismatch: case='${case_name}' expected=${case_expected_label} actual=${case_actual_label}" >&2
        fi

        echo "${case_name},${case_expected_label},${case_actual_label},${case_status},$(csv_sanitize "$case_note")" >>"$suite_summary_csv"
    done < <(jq -c '.cases[]' "$suite_file")

    if (( port_binding_suite_total == 0 )); then
        echo "port binding legality contract suite has zero cases: $suite_file" >&2
        port_binding_suite_status="fail"
        return 1
    fi

    if (( port_binding_suite_failed > 0 )); then
        port_binding_suite_status="fail"
        echo "port binding legality contract suite failed: $port_binding_suite_failed/$port_binding_suite_total mismatches (summary: $suite_summary_csv)" >&2
        return 1
    fi

    port_binding_suite_status="pass"
    return 0
}

run_package_qualification_contract_suite() {
    local suite_file="$1"
    local enforce="$2"
    local suite_summary_csv="$WORK_DIR/package_qualification_contract_summary.csv"
    local idx=0
    local case_json

    package_qual_suite_status="skip"
    package_qual_suite_total=0
    package_qual_suite_passed=0
    package_qual_suite_failed=0
    echo "case,expect,actual,status,notes" >"$suite_summary_csv"

    if [[ "$enforce" -ne 1 ]]; then
        return 0
    fi

    if [[ -z "$suite_file" ]]; then
        echo "package qualification contract suite is enforced but no suite path is configured"
        return 1
    fi
    require_file "$suite_file"

    while IFS= read -r case_json; do
        idx=$((idx + 1))
        package_qual_suite_total=$((package_qual_suite_total + 1))

        local case_name
        local case_expect_pass
        local case_input
        local case_file
        local case_actual_pass
        local case_check_note
        local case_expected_label
        local case_actual_label
        local case_status
        local case_note

        case_name="$(jq -er '.name | strings' <<<"$case_json")"
        case_expect_pass="$(jq -er 'if (.expect_pass // false) then 1 else 0 end' <<<"$case_json")"
        case_input="$(jq -er '.input | strings' <<<"$case_json")"
        case_file="$WORK_DIR/package_qualification_case_${idx}.sv"

        printf '%s\n' "$case_input" >"$case_file"

        if case_check_note="$(check_package_qualification_resolution "$case_file" 2>&1)"; then
            case_actual_pass=1
            if [[ -z "$case_check_note" ]]; then
                case_check_note="package qualification check passed"
            fi
        else
            case_actual_pass=0
            if [[ -z "$case_check_note" ]]; then
                case_check_note="package qualification check failed"
            fi
        fi

        if [[ "$case_expect_pass" -eq 1 ]]; then
            case_expected_label="pass"
        else
            case_expected_label="fail"
        fi
        if [[ "$case_actual_pass" -eq 1 ]]; then
            case_actual_label="pass"
        else
            case_actual_label="fail"
        fi

        if [[ "$case_expect_pass" -eq "$case_actual_pass" ]]; then
            case_status="pass"
            case_note="$case_check_note"
            package_qual_suite_passed=$((package_qual_suite_passed + 1))
        else
            case_status="fail"
            case_note="$case_check_note"
            package_qual_suite_failed=$((package_qual_suite_failed + 1))
            echo "package qualification contract mismatch: case='${case_name}' expected=${case_expected_label} actual=${case_actual_label}" >&2
        fi

        echo "${case_name},${case_expected_label},${case_actual_label},${case_status},$(csv_sanitize "$case_note")" >>"$suite_summary_csv"
    done < <(jq -c '.cases[]' "$suite_file")

    if (( package_qual_suite_total == 0 )); then
        echo "package qualification contract suite has zero cases: $suite_file" >&2
        package_qual_suite_status="fail"
        return 1
    fi

    if (( package_qual_suite_failed > 0 )); then
        package_qual_suite_status="fail"
        echo "package qualification contract suite failed: $package_qual_suite_failed/$package_qual_suite_total mismatches (summary: $suite_summary_csv)" >&2
        return 1
    fi

    package_qual_suite_status="pass"
    return 0
}

run_context_legality_contract_suite() {
    local suite_file="$1"
    local enforce="$2"
    local suite_summary_csv="$WORK_DIR/context_legality_contract_summary.csv"
    local idx=0
    local case_json

    context_legality_suite_status="skip"
    context_legality_suite_total=0
    context_legality_suite_passed=0
    context_legality_suite_failed=0
    echo "case,expect,actual,status,notes" >"$suite_summary_csv"

    if [[ "$enforce" -ne 1 ]]; then
        return 0
    fi

    if [[ -z "$suite_file" ]]; then
        echo "context legality contract suite is enforced but no suite path is configured"
        return 1
    fi
    require_file "$suite_file"

    while IFS= read -r case_json; do
        idx=$((idx + 1))
        context_legality_suite_total=$((context_legality_suite_total + 1))

        local case_name
        local case_expect_pass
        local case_input
        local case_file
        local case_actual_pass
        local case_check_note
        local case_expected_label
        local case_actual_label
        local case_status
        local case_note

        case_name="$(jq -er '.name | strings' <<<"$case_json")"
        case_expect_pass="$(jq -er 'if (.expect_pass // false) then 1 else 0 end' <<<"$case_json")"
        case_input="$(jq -er '.input | strings' <<<"$case_json")"
        case_file="$WORK_DIR/context_legality_case_${idx}.sv"

        printf '%s\n' "$case_input" >"$case_file"

        if case_check_note="$(check_context_legality_basic "$case_file" 2>&1)"; then
            case_actual_pass=1
            if [[ -z "$case_check_note" ]]; then
                case_check_note="context legality check passed"
            fi
        else
            case_actual_pass=0
            if [[ -z "$case_check_note" ]]; then
                case_check_note="context legality check failed"
            fi
        fi

        if [[ "$case_expect_pass" -eq 1 ]]; then
            case_expected_label="pass"
        else
            case_expected_label="fail"
        fi
        if [[ "$case_actual_pass" -eq 1 ]]; then
            case_actual_label="pass"
        else
            case_actual_label="fail"
        fi

        if [[ "$case_expect_pass" -eq "$case_actual_pass" ]]; then
            case_status="pass"
            case_note="$case_check_note"
            context_legality_suite_passed=$((context_legality_suite_passed + 1))
        else
            case_status="fail"
            case_note="$case_check_note"
            context_legality_suite_failed=$((context_legality_suite_failed + 1))
            echo "context legality contract mismatch: case='${case_name}' expected=${case_expected_label} actual=${case_actual_label}" >&2
        fi

        echo "${case_name},${case_expected_label},${case_actual_label},${case_status},$(csv_sanitize "$case_note")" >>"$suite_summary_csv"
    done < <(jq -c '.cases[]' "$suite_file")

    if (( context_legality_suite_total == 0 )); then
        echo "context legality contract suite has zero cases: $suite_file" >&2
        context_legality_suite_status="fail"
        return 1
    fi

    if (( context_legality_suite_failed > 0 )); then
        context_legality_suite_status="fail"
        echo "context legality contract suite failed: $context_legality_suite_failed/$context_legality_suite_total mismatches (summary: $suite_summary_csv)" >&2
        return 1
    fi

    context_legality_suite_status="pass"
    return 0
}

if [[ "$PARSE_FULL_MODE" != "auto" && "$PARSE_FULL_MODE" != "0" && "$PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ -n "$PARSE_FULL_ENFORCE_MIN_PASS_RATIO_OVERRIDE" ]] && [[ "$PARSE_FULL_ENFORCE_MIN_PASS_RATIO_OVERRIDE" != "0" && "$PARSE_FULL_ENFORCE_MIN_PASS_RATIO_OVERRIDE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO must be 0 or 1 when set" >&2
    exit 2
fi
if [[ -n "$PARSE_FULL_MIN_PASS_RATIO_OVERRIDE" ]]; then
    if ! [[ "$PARSE_FULL_MIN_PASS_RATIO_OVERRIDE" =~ ^[0-9]+$ ]] || [[ "$PARSE_FULL_MIN_PASS_RATIO_OVERRIDE" -lt 0 ]] || [[ "$PARSE_FULL_MIN_PASS_RATIO_OVERRIDE" -gt 100 ]]; then
        echo "error: PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO must be an integer between 0 and 100 when set" >&2
        exit 2
    fi
fi
if [[ "$DIFF_MODE" != "auto" && "$DIFF_MODE" != "0" && "$DIFF_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_DIFF_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ "$PERF_BUDGET_MODE" != "auto" && "$PERF_BUDGET_MODE" != "0" && "$PERF_BUDGET_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_PERF_BUDGET_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ "$REALISTIC_CORPUS_MODE" != "auto" && "$REALISTIC_CORPUS_MODE" != "0" && "$REALISTIC_CORPUS_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ "$DECLARED_SHADOW_MODE" != "auto" && "$DECLARED_SHADOW_MODE" != "0" && "$DECLARED_SHADOW_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ "$DECLARED_SHADOW_PARSEABLE_ONLY" != "0" && "$DECLARED_SHADOW_PARSEABLE_ONLY" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_DECLARED_SHADOW_PARSEABLE_ONLY must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$DIFF_MAX_SAMPLES" =~ ^[0-9]+$ ]] || [[ "$DIFF_MAX_SAMPLES" -lt 1 ]]; then
    echo "error: PGEN_SV_STIMULI_DIFF_MAX_SAMPLES must be an integer >= 1" >&2
    exit 2
fi
if [[ -n "$CARGO_BUILD_JOBS_OVERRIDE" ]]; then
    if ! [[ "$CARGO_BUILD_JOBS_OVERRIDE" =~ ^[0-9]+$ ]] || [[ "$CARGO_BUILD_JOBS_OVERRIDE" -lt 1 ]]; then
        echo "error: PGEN_SV_STIMULI_CARGO_BUILD_JOBS must be an integer >= 1 when set" >&2
        exit 2
    fi
fi
if [[ -n "$PENDING_FRONTIER_EXTRA_STAGNATION_OVERRIDE" ]]; then
    if ! [[ "$PENDING_FRONTIER_EXTRA_STAGNATION_OVERRIDE" =~ ^[0-9]+$ ]]; then
        echo "error: PGEN_SV_STIMULI_QUALITY_PENDING_FRONTIER_EXTRA_STAGNATION must be an integer >= 0 when set" >&2
        exit 2
    fi
fi
if ! [[ "$REALISTIC_CORPUS_MAX_CASES" =~ ^[0-9]+$ ]] || [[ "$REALISTIC_CORPUS_MAX_CASES" -lt 0 ]]; then
    echo "error: PGEN_SV_STIMULI_REALISTIC_CORPUS_MAX_CASES must be an integer >= 0" >&2
    exit 2
fi
if [[ "$SEMANTIC_CLOSURE_MODE" != "0" && "$SEMANTIC_CLOSURE_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE must be 0 or 1" >&2
    exit 2
fi
if [[ -n "$ENFORCE_DECLARED_IDENTIFIER_SUITE_OVERRIDE" ]] && [[ "$ENFORCE_DECLARED_IDENTIFIER_SUITE_OVERRIDE" != "0" && "$ENFORCE_DECLARED_IDENTIFIER_SUITE_OVERRIDE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_ENFORCE_DECLARED_IDENTIFIER_SUITE must be 0 or 1 when set" >&2
    exit 2
fi
if [[ -n "$ENFORCE_WIDTH_COMPAT_SUITE_OVERRIDE" ]] && [[ "$ENFORCE_WIDTH_COMPAT_SUITE_OVERRIDE" != "0" && "$ENFORCE_WIDTH_COMPAT_SUITE_OVERRIDE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_ENFORCE_WIDTH_COMPAT_SUITE must be 0 or 1 when set" >&2
    exit 2
fi
if [[ -n "$ENFORCE_PORT_BINDING_SUITE_OVERRIDE" ]] && [[ "$ENFORCE_PORT_BINDING_SUITE_OVERRIDE" != "0" && "$ENFORCE_PORT_BINDING_SUITE_OVERRIDE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_ENFORCE_PORT_BINDING_SUITE must be 0 or 1 when set" >&2
    exit 2
fi
if [[ -n "$ENFORCE_PACKAGE_QUAL_SUITE_OVERRIDE" ]] && [[ "$ENFORCE_PACKAGE_QUAL_SUITE_OVERRIDE" != "0" && "$ENFORCE_PACKAGE_QUAL_SUITE_OVERRIDE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_ENFORCE_PACKAGE_QUAL_SUITE must be 0 or 1 when set" >&2
    exit 2
fi
if [[ -n "$ENFORCE_CONTEXT_LEGALITY_SUITE_OVERRIDE" ]] && [[ "$ENFORCE_CONTEXT_LEGALITY_SUITE_OVERRIDE" != "0" && "$ENFORCE_CONTEXT_LEGALITY_SUITE_OVERRIDE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_ENFORCE_CONTEXT_LEGALITY_SUITE must be 0 or 1 when set" >&2
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

contract_version="$(jq -er '.version | numbers' "$CONTRACT_FILE")"
grammar_name="$(jq -er '.grammar_name | strings' "$CONTRACT_FILE")"
ebnf_path_rel="$(jq -er '.ebnf_path | strings' "$CONTRACT_FILE")"
default_sample_count="$(jq -er '.sample_count | numbers' "$CONTRACT_FILE")"
default_seed_base="$(jq -er '.seed_base | numbers' "$CONTRACT_FILE")"
default_lrm_profile="$(jq -er '(.lrm_profiles.default_profile // "2017") | strings' "$CONTRACT_FILE")"
supported_lrm_profiles_csv="$(jq -er '(.lrm_profiles.supported_profiles // ["2017","2023"]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"
required_lrm_profiles_csv="$(jq -er '(.lrm_profiles.required_profiles // [(.lrm_profiles.default_profile // "2017")]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"
default_stimuli_mode="$(jq -er '(.stimuli_modes.default_mode // "sv_file") | strings' "$CONTRACT_FILE")"
supported_stimuli_modes_csv="$(jq -er '(.stimuli_modes.supported_modes // ["sv_file","sv_snippet","sv_pp_file","sv_pp_snippet","sv_semantic_file"]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"
declared_identifier_suite_rel="$(jq -er '(.semantic_contracts.declared_identifier_suite_path // "") | strings' "$CONTRACT_FILE")"
enforce_declared_identifier_suite="$(jq -er 'if (.semantic_contracts.enforce_declared_identifier_suite // false) then 1 else 0 end' "$CONTRACT_FILE")"
width_compat_suite_rel="$(jq -er '(.semantic_contracts.width_compatibility_suite_path // "") | strings' "$CONTRACT_FILE")"
enforce_width_compat_suite="$(jq -er 'if (.semantic_contracts.enforce_width_compatibility_suite // false) then 1 else 0 end' "$CONTRACT_FILE")"
port_binding_suite_rel="$(jq -er '(.semantic_contracts.port_binding_legality_suite_path // "") | strings' "$CONTRACT_FILE")"
enforce_port_binding_suite="$(jq -er 'if (.semantic_contracts.enforce_port_binding_legality_suite // false) then 1 else 0 end' "$CONTRACT_FILE")"
package_qual_suite_rel="$(jq -er '(.semantic_contracts.package_qualification_suite_path // "") | strings' "$CONTRACT_FILE")"
enforce_package_qual_suite="$(jq -er 'if (.semantic_contracts.enforce_package_qualification_suite // false) then 1 else 0 end' "$CONTRACT_FILE")"
context_legality_suite_rel="$(jq -er '(.semantic_contracts.context_legality_suite_path // "") | strings' "$CONTRACT_FILE")"
enforce_context_legality_suite="$(jq -er 'if (.semantic_contracts.enforce_context_legality_suite // false) then 1 else 0 end' "$CONTRACT_FILE")"
closed_loop_enabled="$(jq -er 'if (.closed_loop.enabled // true) then 1 else 0 end' "$CONTRACT_FILE")"
gap_report_threshold="$(jq -er '(.closed_loop.gap_report_threshold // 1) | numbers' "$CONTRACT_FILE")"
target_max_attempts="$(jq -er '(.closed_loop.target_max_attempts // 5000) | numbers' "$CONTRACT_FILE")"
require_non_increasing_target_debt="$(jq -er 'if (.closed_loop.require_non_increasing_target_debt // true) then 1 else 0 end' "$CONTRACT_FILE")"
parseability_shadow_contract_enabled="$(jq -er 'if (.closed_loop.parseability_shadow_enabled // false) then 1 else 0 end' "$CONTRACT_FILE")"
failure_replay_enabled="$(jq -er 'if (.failure_replay.enabled // true) then 1 else 0 end' "$CONTRACT_FILE")"
shrink_semantic_failures="$(jq -er 'if (.failure_replay.shrink_semantic_failures // true) then 1 else 0 end' "$CONTRACT_FILE")"
shrink_parse_full_failures="$(jq -er 'if (.failure_replay.shrink_parse_full_failures // true) then 1 else 0 end' "$CONTRACT_FILE")"
failure_shrink_max_iterations="$(jq -er '(.failure_replay.shrink_max_iterations // 24) | numbers' "$CONTRACT_FILE")"
perf_budget_contract_enforced="$(jq -er 'if (.performance_budgets.enforce // false) then 1 else 0 end' "$CONTRACT_FILE")"
perf_max_generate_ms_per_sample="$(jq -er '(.performance_budgets.max_generate_ms_per_sample // 0) | numbers' "$CONTRACT_FILE")"
perf_max_preprocess_ms_per_sample="$(jq -er '(.performance_budgets.max_preprocess_ms_per_sample // 0) | numbers' "$CONTRACT_FILE")"
perf_max_parse_full_ms_per_sample="$(jq -er '(.performance_budgets.max_parse_full_ms_per_sample // 0) | numbers' "$CONTRACT_FILE")"
perf_max_sample_bytes="$(jq -er '(.performance_budgets.max_sample_bytes // 0) | numbers' "$CONTRACT_FILE")"
perf_max_preprocessed_bytes="$(jq -er '(.performance_budgets.max_preprocessed_bytes // 0) | numbers' "$CONTRACT_FILE")"
realistic_corpus_contract_enforced="$(jq -er 'if (.nexsim_realistic_corpus.enforce // false) then 1 else 0 end' "$CONTRACT_FILE")"
realistic_corpus_rel_default="$(jq -er '(.nexsim_realistic_corpus.cases_path // "") | strings' "$CONTRACT_FILE")"
realistic_max_preprocess_ms_per_case="$(jq -er '(.nexsim_realistic_corpus.max_preprocess_ms_per_case // 0) | numbers' "$CONTRACT_FILE")"
realistic_max_parse_full_ms_per_case="$(jq -er '(.nexsim_realistic_corpus.max_parse_full_ms_per_case // 0) | numbers' "$CONTRACT_FILE")"
realistic_max_sample_bytes="$(jq -er '(.nexsim_realistic_corpus.max_sample_bytes // 0) | numbers' "$CONTRACT_FILE")"
realistic_max_preprocessed_bytes="$(jq -er '(.nexsim_realistic_corpus.max_preprocessed_bytes // 0) | numbers' "$CONTRACT_FILE")"
realistic_require_no_preprocess_errors="$(jq -er 'if (.nexsim_realistic_corpus.require_no_preprocess_errors // true) then 1 else 0 end' "$CONTRACT_FILE")"
parse_full_quality_contract_enforced="$(jq -er 'if (.parse_full_quality.enforce_min_pass_ratio // false) then 1 else 0 end' "$CONTRACT_FILE")"
parse_full_quality_min_pass_ratio="$(jq -er '(.parse_full_quality.min_pass_ratio // 0) | numbers' "$CONTRACT_FILE")"
declared_shadow_contract_enabled="$(jq -er 'if (.semantic_promotion.declared_identifier_shadow_enabled // true) then 1 else 0 end' "$CONTRACT_FILE")"
declared_shadow_contract_strict="$(jq -er 'if (.semantic_promotion.declared_identifier_shadow_strict // false) then 1 else 0 end' "$CONTRACT_FILE")"

sample_count="${SAMPLE_COUNT_OVERRIDE:-$default_sample_count}"
seed_base="${SEED_BASE_OVERRIDE:-$default_seed_base}"
replay_sample_count="$(jq -er --argjson fallback "$sample_count" '(.closed_loop.replay_sample_count // $fallback) | numbers' "$CONTRACT_FILE")"
target_max_attempts_source="contract"
if [[ -n "$TARGET_MAX_ATTEMPTS_OVERRIDE" ]]; then
    target_max_attempts="$TARGET_MAX_ATTEMPTS_OVERRIDE"
    target_max_attempts_source="env_override"
fi
if [[ -n "$STIMULI_MODE_OVERRIDE" ]]; then
    stimuli_mode="$STIMULI_MODE_OVERRIDE"
elif [[ "$SEMANTIC_CLOSURE_MODE" == "1" ]]; then
    stimuli_mode="sv_semantic_file"
else
    stimuli_mode="$default_stimuli_mode"
fi
if [[ -n "$DECLARED_IDENTIFIER_SUITE_OVERRIDE" ]]; then
    declared_identifier_suite_rel="$DECLARED_IDENTIFIER_SUITE_OVERRIDE"
fi
if [[ -n "$ENFORCE_DECLARED_IDENTIFIER_SUITE_OVERRIDE" ]]; then
    enforce_declared_identifier_suite="$ENFORCE_DECLARED_IDENTIFIER_SUITE_OVERRIDE"
fi
if [[ -n "$WIDTH_COMPAT_SUITE_OVERRIDE" ]]; then
    width_compat_suite_rel="$WIDTH_COMPAT_SUITE_OVERRIDE"
fi
if [[ -n "$ENFORCE_WIDTH_COMPAT_SUITE_OVERRIDE" ]]; then
    enforce_width_compat_suite="$ENFORCE_WIDTH_COMPAT_SUITE_OVERRIDE"
fi
if [[ -n "$PORT_BINDING_SUITE_OVERRIDE" ]]; then
    port_binding_suite_rel="$PORT_BINDING_SUITE_OVERRIDE"
fi
if [[ -n "$ENFORCE_PORT_BINDING_SUITE_OVERRIDE" ]]; then
    enforce_port_binding_suite="$ENFORCE_PORT_BINDING_SUITE_OVERRIDE"
fi
if [[ -n "$PACKAGE_QUAL_SUITE_OVERRIDE" ]]; then
    package_qual_suite_rel="$PACKAGE_QUAL_SUITE_OVERRIDE"
fi
if [[ -n "$ENFORCE_PACKAGE_QUAL_SUITE_OVERRIDE" ]]; then
    enforce_package_qual_suite="$ENFORCE_PACKAGE_QUAL_SUITE_OVERRIDE"
fi
if [[ -n "$CONTEXT_LEGALITY_SUITE_OVERRIDE" ]]; then
    context_legality_suite_rel="$CONTEXT_LEGALITY_SUITE_OVERRIDE"
fi
if [[ -n "$ENFORCE_CONTEXT_LEGALITY_SUITE_OVERRIDE" ]]; then
    enforce_context_legality_suite="$ENFORCE_CONTEXT_LEGALITY_SUITE_OVERRIDE"
fi
declared_identifier_suite_path=""
if [[ -n "$declared_identifier_suite_rel" ]]; then
    if [[ "$declared_identifier_suite_rel" == /* ]]; then
        declared_identifier_suite_path="$declared_identifier_suite_rel"
    else
        declared_identifier_suite_path="$ROOT_DIR/$declared_identifier_suite_rel"
    fi
fi
width_compat_suite_path=""
if [[ -n "$width_compat_suite_rel" ]]; then
    if [[ "$width_compat_suite_rel" == /* ]]; then
        width_compat_suite_path="$width_compat_suite_rel"
    else
        width_compat_suite_path="$ROOT_DIR/$width_compat_suite_rel"
    fi
fi
port_binding_suite_path=""
if [[ -n "$port_binding_suite_rel" ]]; then
    if [[ "$port_binding_suite_rel" == /* ]]; then
        port_binding_suite_path="$port_binding_suite_rel"
    else
        port_binding_suite_path="$ROOT_DIR/$port_binding_suite_rel"
    fi
fi
package_qual_suite_path=""
if [[ -n "$package_qual_suite_rel" ]]; then
    if [[ "$package_qual_suite_rel" == /* ]]; then
        package_qual_suite_path="$package_qual_suite_rel"
    else
        package_qual_suite_path="$ROOT_DIR/$package_qual_suite_rel"
    fi
fi
context_legality_suite_path=""
if [[ -n "$context_legality_suite_rel" ]]; then
    if [[ "$context_legality_suite_rel" == /* ]]; then
        context_legality_suite_path="$context_legality_suite_rel"
    else
        context_legality_suite_path="$ROOT_DIR/$context_legality_suite_rel"
    fi
fi
realistic_corpus_rel="$realistic_corpus_rel_default"
if [[ -n "$REALISTIC_CORPUS_OVERRIDE" ]]; then
    realistic_corpus_rel="$REALISTIC_CORPUS_OVERRIDE"
fi
realistic_corpus_path=""
if [[ -n "$realistic_corpus_rel" ]]; then
    realistic_corpus_path="$(resolve_path "$realistic_corpus_rel")"
fi

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
pending_frontier_extra_stagnation_args=()
if [[ -n "$PENDING_FRONTIER_EXTRA_STAGNATION_OVERRIDE" ]]; then
    pending_frontier_extra_stagnation_args=(
        --target-pending-frontier-extra-stagnation
        "$PENDING_FRONTIER_EXTRA_STAGNATION_OVERRIDE"
    )
fi
if ! [[ "$failure_shrink_max_iterations" =~ ^[0-9]+$ ]] || [[ "$failure_shrink_max_iterations" -lt 1 ]]; then
    echo "error: failure_replay.shrink_max_iterations must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$perf_max_generate_ms_per_sample" =~ ^[0-9]+$ ]]; then
    echo "error: performance_budgets.max_generate_ms_per_sample must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$perf_max_preprocess_ms_per_sample" =~ ^[0-9]+$ ]]; then
    echo "error: performance_budgets.max_preprocess_ms_per_sample must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$perf_max_parse_full_ms_per_sample" =~ ^[0-9]+$ ]]; then
    echo "error: performance_budgets.max_parse_full_ms_per_sample must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$perf_max_sample_bytes" =~ ^[0-9]+$ ]]; then
    echo "error: performance_budgets.max_sample_bytes must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$perf_max_preprocessed_bytes" =~ ^[0-9]+$ ]]; then
    echo "error: performance_budgets.max_preprocessed_bytes must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$realistic_max_preprocess_ms_per_case" =~ ^[0-9]+$ ]]; then
    echo "error: nexsim_realistic_corpus.max_preprocess_ms_per_case must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$realistic_max_parse_full_ms_per_case" =~ ^[0-9]+$ ]]; then
    echo "error: nexsim_realistic_corpus.max_parse_full_ms_per_case must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$realistic_max_sample_bytes" =~ ^[0-9]+$ ]]; then
    echo "error: nexsim_realistic_corpus.max_sample_bytes must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$realistic_max_preprocessed_bytes" =~ ^[0-9]+$ ]]; then
    echo "error: nexsim_realistic_corpus.max_preprocessed_bytes must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$parse_full_quality_min_pass_ratio" =~ ^[0-9]+$ ]] || [[ "$parse_full_quality_min_pass_ratio" -lt 0 ]] || [[ "$parse_full_quality_min_pass_ratio" -gt 100 ]]; then
    echo "error: parse_full_quality.min_pass_ratio must be an integer between 0 and 100" >&2
    exit 2
fi
if [[ -n "$PARSE_FULL_ENFORCE_MIN_PASS_RATIO_OVERRIDE" ]]; then
    parse_full_quality_contract_enforced="$PARSE_FULL_ENFORCE_MIN_PASS_RATIO_OVERRIDE"
fi
if [[ -n "$PARSE_FULL_MIN_PASS_RATIO_OVERRIDE" ]]; then
    parse_full_quality_min_pass_ratio="$PARSE_FULL_MIN_PASS_RATIO_OVERRIDE"
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
mode_parse_full_eligible="$(jq -er --arg mode "$stimuli_mode" 'if (.stimuli_modes.profiles[$mode].parse_full_eligible // ($mode == "sv_file" or $mode == "sv_pp_file" or $mode == "sv_semantic_file")) then 1 else 0 end' "$CONTRACT_FILE")"
mode_recovery_stimuli_mode="$(jq -er --arg mode "$stimuli_mode" '(.stimuli_modes.profiles[$mode].recovery_stimuli_mode // "baseline") | strings' "$CONTRACT_FILE")"
mode_max_depth="$(jq -er --arg mode "$stimuli_mode" '(.stimuli_modes.profiles[$mode].max_depth // 24) | numbers' "$CONTRACT_FILE")"
mode_max_repeat="$(jq -er --arg mode "$stimuli_mode" '(.stimuli_modes.profiles[$mode].max_repeat // 4) | numbers' "$CONTRACT_FILE")"

if [[ "$mode_recovery_stimuli_mode" != "baseline" && "$mode_recovery_stimuli_mode" != "recovery_biased" && "$mode_recovery_stimuli_mode" != "near_sync_negative" ]]; then
    echo "error: unsupported recovery stimuli mode '$mode_recovery_stimuli_mode' for stimuli mode '$stimuli_mode' (supported: baseline, recovery_biased, near_sync_negative)" >&2
    exit 2
fi
if ! [[ "$mode_max_depth" =~ ^[0-9]+$ ]] || [[ "$mode_max_depth" -lt 1 ]]; then
    echo "error: stimuli mode '$stimuli_mode' max_depth must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$mode_max_repeat" =~ ^[0-9]+$ ]] || [[ "$mode_max_repeat" -lt 1 ]]; then
    echo "error: stimuli mode '$stimuli_mode' max_repeat must be an integer >= 1" >&2
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
require_declared_identifiers_parseable_only="$(jq -er 'if (.semantic_baseline.require_declared_identifiers_parseable_only // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_package_qualification_resolution="$(jq -er 'if (.semantic_baseline.require_package_qualification_resolution // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_width_compatibility_simple="$(jq -er 'if (.semantic_baseline.require_width_compatibility_simple // false) then 1 else 0 end' "$CONTRACT_FILE")"
require_context_legality_basic="$(jq -er 'if (.semantic_baseline.require_context_legality_basic // false) then 1 else 0 end' "$CONTRACT_FILE")"

require_nonempty_preprocessed_output="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_nonempty_preprocessed_output" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_nonempty_preprocessed_output // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_nonempty_preprocessed_output then 1 else 0 end) end' "$CONTRACT_FILE")"
require_no_preprocess_errors="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_no_preprocess_errors" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_no_preprocess_errors // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_no_preprocess_errors then 1 else 0 end) end' "$CONTRACT_FILE")"
require_balanced_structural_keywords="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_balanced_structural_keywords" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_balanced_structural_keywords // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_balanced_structural_keywords then 1 else 0 end) end' "$CONTRACT_FILE")"
require_unique_named_port_bindings="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_unique_named_port_bindings" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_unique_named_port_bindings // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_unique_named_port_bindings then 1 else 0 end) end' "$CONTRACT_FILE")"
require_port_binding_legality_basic="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_port_binding_legality_basic" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_port_binding_legality_basic // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_port_binding_legality_basic then 1 else 0 end) end' "$CONTRACT_FILE")"
require_declared_identifiers_before_use="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_declared_identifiers_before_use" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_declared_identifiers_before_use // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_declared_identifiers_before_use then 1 else 0 end) end' "$CONTRACT_FILE")"
require_declared_identifiers_parseable_only="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_declared_identifiers_parseable_only" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_declared_identifiers_parseable_only // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_declared_identifiers_parseable_only then 1 else 0 end) end' "$CONTRACT_FILE")"
require_package_qualification_resolution="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_package_qualification_resolution" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_package_qualification_resolution // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_package_qualification_resolution then 1 else 0 end) end' "$CONTRACT_FILE")"
require_width_compatibility_simple="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_width_compatibility_simple" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_width_compatibility_simple // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_width_compatibility_simple then 1 else 0 end) end' "$CONTRACT_FILE")"
require_context_legality_basic="$(jq -er --arg mode "$stimuli_mode" --argjson fallback "$require_context_legality_basic" 'if (.stimuli_modes.profiles[$mode].semantic_overrides.require_context_legality_basic // null) == null then $fallback else (if .stimuli_modes.profiles[$mode].semantic_overrides.require_context_legality_basic then 1 else 0 end) end' "$CONTRACT_FILE")"

if ! [[ "$include_max_depth" =~ ^[0-9]+$ ]] || [[ "$include_max_depth" -lt 1 ]]; then
    echo "error: preprocess.include_max_depth must be an integer >= 1" >&2
    exit 2
fi

grammar_file="$ROOT_DIR/$ebnf_path_rel"
grammar_json="$WORK_DIR/${grammar_name}.json"
grammar_input="$grammar_file"
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
echo "semantic_closure_mode: $SEMANTIC_CLOSURE_MODE"
echo "stimuli_mode_entry_rule: $mode_entry_rule"
echo "stimuli_mode_closed_loop_enabled: $mode_closed_loop_enabled"
echo "stimuli_mode_parse_full_eligible: $mode_parse_full_eligible"
echo "stimuli_mode_recovery_stimuli_mode: $mode_recovery_stimuli_mode"
echo "stimuli_mode_max_depth: $mode_max_depth"
echo "stimuli_mode_max_repeat: $mode_max_repeat"
echo "closed_loop_enabled: $closed_loop_enabled"
echo "closed_loop_effective_enabled: $closed_loop_effective_enabled"
echo "closed_loop_gap_report_threshold: $gap_report_threshold"
echo "closed_loop_target_max_attempts: $target_max_attempts"
echo "closed_loop_target_max_attempts_source: $target_max_attempts_source"
echo "cargo_build_jobs: ${CARGO_BUILD_JOBS_OVERRIDE:-<default>}"
echo "closed_loop_replay_sample_count: $replay_sample_count"
echo "closed_loop_require_non_increasing_target_debt: $require_non_increasing_target_debt"
echo "closed_loop_parseability_shadow_contract_enabled: $parseability_shadow_contract_enabled"
echo "failure_replay_enabled: $failure_replay_enabled"
echo "failure_replay_shrink_semantic_failures: $shrink_semantic_failures"
echo "failure_replay_shrink_parse_full_failures: $shrink_parse_full_failures"
echo "failure_replay_shrink_max_iterations: $failure_shrink_max_iterations"
echo "semantic_require_port_binding_legality_basic: $require_port_binding_legality_basic"
echo "semantic_require_declared_identifiers_before_use: $require_declared_identifiers_before_use"
echo "semantic_require_declared_identifiers_parseable_only: $require_declared_identifiers_parseable_only"
echo "semantic_require_package_qualification_resolution: $require_package_qualification_resolution"
echo "semantic_require_width_compatibility_simple: $require_width_compatibility_simple"
echo "semantic_require_context_legality_basic: $require_context_legality_basic"
echo "declared_identifier_suite_enforced: $enforce_declared_identifier_suite"
echo "declared_identifier_suite_path: ${declared_identifier_suite_path:-<unset>}"
echo "width_compatibility_suite_enforced: $enforce_width_compat_suite"
echo "width_compatibility_suite_path: ${width_compat_suite_path:-<unset>}"
echo "port_binding_legality_suite_enforced: $enforce_port_binding_suite"
echo "port_binding_legality_suite_path: ${port_binding_suite_path:-<unset>}"
echo "package_qualification_suite_enforced: $enforce_package_qual_suite"
echo "package_qualification_suite_path: ${package_qual_suite_path:-<unset>}"
echo "context_legality_suite_enforced: $enforce_context_legality_suite"
echo "context_legality_suite_path: ${context_legality_suite_path:-<unset>}"
echo "differential_mode: $DIFF_MODE"
echo "differential_max_samples: $DIFF_MAX_SAMPLES"
echo "differential_reference_runner: ${DIFF_REFERENCE_RUNNER:-<unset>}"
echo "performance_budget_mode: $PERF_BUDGET_MODE"
echo "performance_budget_contract_enforced: $perf_budget_contract_enforced"
echo "performance_max_generate_ms_per_sample: $perf_max_generate_ms_per_sample"
echo "performance_max_preprocess_ms_per_sample: $perf_max_preprocess_ms_per_sample"
echo "performance_max_parse_full_ms_per_sample: $perf_max_parse_full_ms_per_sample"
echo "performance_max_sample_bytes: $perf_max_sample_bytes"
echo "performance_max_preprocessed_bytes: $perf_max_preprocessed_bytes"
echo "realistic_corpus_mode: $REALISTIC_CORPUS_MODE"
echo "realistic_corpus_contract_enforced: $realistic_corpus_contract_enforced"
echo "realistic_corpus_path: ${realistic_corpus_path:-<unset>}"
echo "realistic_corpus_max_cases: $REALISTIC_CORPUS_MAX_CASES"
echo "realistic_corpus_max_preprocess_ms_per_case: $realistic_max_preprocess_ms_per_case"
echo "realistic_corpus_max_parse_full_ms_per_case: $realistic_max_parse_full_ms_per_case"
echo "realistic_corpus_max_sample_bytes: $realistic_max_sample_bytes"
echo "realistic_corpus_max_preprocessed_bytes: $realistic_max_preprocessed_bytes"
echo "realistic_corpus_require_no_preprocess_errors: $realistic_require_no_preprocess_errors"
echo "parse_full_quality_enforced: $parse_full_quality_contract_enforced"
echo "parse_full_quality_min_pass_ratio: $parse_full_quality_min_pass_ratio"
echo "declared_shadow_mode: $DECLARED_SHADOW_MODE"
echo "declared_shadow_parseable_only: $DECLARED_SHADOW_PARSEABLE_ONLY"
echo "declared_shadow_contract_enabled: $declared_shadow_contract_enabled"
echo "declared_shadow_contract_strict: $declared_shadow_contract_strict"

run_logged "declared_identifier_contract_suite" \
    run_declared_identifier_contract_suite "$declared_identifier_suite_path" "$enforce_declared_identifier_suite"
run_logged "width_compatibility_contract_suite" \
    run_width_compatibility_contract_suite "$width_compat_suite_path" "$enforce_width_compat_suite"
run_logged "port_binding_legality_contract_suite" \
    run_port_binding_legality_contract_suite "$port_binding_suite_path" "$enforce_port_binding_suite"
run_logged "package_qualification_contract_suite" \
    run_package_qualification_contract_suite "$package_qual_suite_path" "$enforce_package_qual_suite"
run_logged "context_legality_contract_suite" \
    run_context_legality_contract_suite "$context_legality_suite_path" "$enforce_context_legality_suite"

echo "profile,sample,seed,coverage_gap_initial,gap_replay,stimuli_generate,parseability_attempts,parseability_accepted,parseability_rejected,parseability_parser_rejections,parseability_generation_errors,parseability_empty_generations,parseability_acceptance_rate_percent,preprocess,semantic_validate,parse_full,warnings,errors,status,notes" >"$SUMMARY_CSV"

run_logged_rust "build_ast_pipeline_for_sv_generation" \
    run_sv_cargo_build cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi
run_logged "frontend_rust_raw_ast_export" \
    "$AST_PIPELINE_BIN" "$grammar_input" \
    --emit-raw-ast-json "$grammar_json"
require_nonempty_file "$grammar_json"

run_logged "generate_sv_parser" \
    "$AST_PIPELINE_BIN" "$grammar_input" \
    --generate-parser \
    --emit-raw-ast-json "$grammar_json" \
    --eliminate-left-recursion \
    --output "$parser_out"
require_nonempty_file "$parser_out"

run_logged_rust "build_ast_pipeline_and_parseability_probe_with_systemverilog_adapter" \
    run_sv_cargo_build env PGEN_SYSTEMVERILOG_PARSER_PATH="$parser_out" \
    cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline --bin parseability_probe
if [[ ! -x "$PARSE_PROBE_BIN" ]]; then
    echo "error: parseability_probe binary is missing at '$PARSE_PROBE_BIN' after adapter build" >&2
    exit 1
fi
if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after adapter build" >&2
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

parseability_generation_enabled=0
parseability_generation_note="generation uses raw stimuli only"
parseability_generation_args=()
if [[ "$parse_full_enabled" -eq 1 ]]; then
    parseability_generation_enabled=1
    parseability_generation_note="generation retries until generated parser accepts the sample"
    parseability_generation_args=(--validate-parseability)
fi

closed_loop_parseability_shadow_enabled=0
closed_loop_parseability_shadow_effective="disabled_by_contract"
closed_loop_parseability_shadow_note="closed-loop replay parseability shadow disabled by contract"
if [[ "$parseability_shadow_contract_enabled" -eq 1 ]]; then
    if [[ "$closed_loop_effective_enabled" -ne 1 ]]; then
        closed_loop_parseability_shadow_effective="disabled_by_closed_loop"
        closed_loop_parseability_shadow_note="closed-loop replay parseability shadow skipped because closed-loop replay is disabled"
    elif [[ "$parse_full_enabled" -ne 1 ]]; then
        closed_loop_parseability_shadow_effective="disabled_by_parse_full"
        closed_loop_parseability_shadow_note="closed-loop replay parseability shadow skipped because parser-backed validation is unavailable (${parse_full_effective})"
    else
        closed_loop_parseability_shadow_enabled=1
        closed_loop_parseability_shadow_effective="enabled"
        closed_loop_parseability_shadow_note="closed-loop replay parseability shadow enabled"
    fi
fi

perf_budget_enabled=0
perf_budget_effective="disabled_by_mode"
perf_budget_note="performance budget checks disabled by mode"
if [[ "$PERF_BUDGET_MODE" == "1" ]]; then
    perf_budget_enabled=1
    perf_budget_effective="enabled"
    perf_budget_note="performance budget checks enabled by strict mode"
elif [[ "$PERF_BUDGET_MODE" == "auto" ]]; then
    if [[ "$perf_budget_contract_enforced" -eq 1 ]]; then
        perf_budget_enabled=1
        perf_budget_effective="enabled"
        perf_budget_note="performance budget checks enabled by contract"
    else
        perf_budget_enabled=0
        perf_budget_effective="disabled_by_contract"
        perf_budget_note="performance budget checks disabled by contract"
    fi
fi
if [[ "$perf_budget_enabled" -eq 1 && "$parse_full_enabled" -ne 1 && "$perf_max_parse_full_ms_per_sample" -gt 0 ]]; then
    perf_budget_note="${perf_budget_note}; parse_full timing budget skipped (${parse_full_effective})"
fi

realistic_corpus_enabled=0
realistic_corpus_effective="disabled_by_mode"
realistic_corpus_note="realistic corpus validation disabled by mode"
if [[ "$REALISTIC_CORPUS_MODE" == "1" ]]; then
    realistic_corpus_enabled=1
    realistic_corpus_effective="enabled"
    realistic_corpus_note="realistic corpus validation enabled by strict mode"
elif [[ "$REALISTIC_CORPUS_MODE" == "auto" ]]; then
    if [[ "$realistic_corpus_contract_enforced" -eq 1 ]]; then
        realistic_corpus_enabled=1
        realistic_corpus_effective="enabled"
        realistic_corpus_note="realistic corpus validation enabled by contract"
    else
        realistic_corpus_enabled=0
        realistic_corpus_effective="disabled_by_contract"
        realistic_corpus_note="realistic corpus validation disabled by contract"
    fi
fi
if [[ "$realistic_corpus_enabled" -eq 1 && "$parse_full_supported" -ne 1 ]]; then
    echo "error: realistic corpus validation requires generated parser adapter for '$grammar_name'" >&2
    exit 1
fi
if [[ "$realistic_corpus_enabled" -eq 1 ]]; then
    if [[ -z "$realistic_corpus_path" ]]; then
        echo "error: realistic corpus validation is enabled but no corpus path is configured" >&2
        exit 1
    fi
    require_file "$realistic_corpus_path"
fi

declared_shadow_enabled=0
declared_shadow_strict=0
declared_shadow_effective="disabled_by_mode"
declared_shadow_note="declared-identifier shadow burn-down is disabled by mode"
if [[ "$require_declared_identifiers_before_use" -eq 1 ]]; then
    declared_shadow_enabled=0
    declared_shadow_strict=0
    declared_shadow_effective="disabled_by_runtime_requirement"
    declared_shadow_note="shadow burn-down skipped because require_declared_identifiers_before_use is already enforced"
elif [[ "$DECLARED_SHADOW_MODE" == "1" ]]; then
    declared_shadow_enabled=1
    declared_shadow_strict=1
    declared_shadow_effective="enabled"
    declared_shadow_note="declared-identifier shadow burn-down enabled by strict mode"
elif [[ "$DECLARED_SHADOW_MODE" == "auto" ]]; then
    if [[ "$declared_shadow_contract_enabled" -eq 1 ]]; then
        declared_shadow_enabled=1
        declared_shadow_effective="enabled"
        declared_shadow_note="declared-identifier shadow burn-down enabled by contract"
        if [[ "$declared_shadow_contract_strict" -eq 1 ]]; then
            declared_shadow_strict=1
            declared_shadow_note="${declared_shadow_note}; strict policy enabled by contract"
        fi
    else
        declared_shadow_enabled=0
        declared_shadow_effective="disabled_by_contract"
        declared_shadow_note="declared-identifier shadow burn-down disabled by contract"
    fi
fi

semantic_pass_count=0
parse_full_pass_count=0
parse_full_skip_count=0
parse_full_fail_count=0
closed_loop_profile_pass_count=0
closed_loop_profile_skip_count=0
closed_loop_initial_replay_determinism_pass_count=0
closed_loop_initial_targets_total=0
closed_loop_replay_targets_total=0
closed_loop_initial_preprocess_warnings_total=0
closed_loop_initial_preprocess_errors_total=0
closed_loop_replay_preprocess_warnings_total=0
closed_loop_replay_preprocess_errors_total=0
closed_loop_parseability_shadow_requested_total=0
closed_loop_parseability_shadow_attempts_total=0
closed_loop_parseability_shadow_accepted_total=0
closed_loop_parseability_shadow_rejected_total=0
closed_loop_parseability_shadow_parser_rejections_total=0
closed_loop_parseability_shadow_generation_errors_total=0
closed_loop_parseability_shadow_empty_generations_total=0
closed_loop_parseability_shadow_acceptance_rate_percent="0.00"
closed_loop_parseability_shadow_counterexamples_captured_total=0
total_warnings=0
total_errors=0
semantic_shrink_count=0
parse_full_shrink_count=0
perf_generate_total_ms=0
perf_generate_max_ms=0
perf_preprocess_total_ms=0
perf_preprocess_max_ms=0
perf_parse_full_total_ms=0
perf_parse_full_max_ms=0
perf_parse_full_samples=0
perf_sample_bytes_max=0
perf_preprocessed_bytes_max=0
parseability_generation_requested_total=0
parseability_generation_accepted_total=0
parseability_generation_rejected_total=0
parseability_generation_attempts_total=0
parseability_generation_parser_rejections_total=0
parseability_generation_errors_total=0
parseability_generation_empty_generations_total=0
parseability_generation_counterexamples_captured_total=0
realistic_cases_declared=0
realistic_cases_executed=0
realistic_expected_pass_total=0
realistic_expected_fail_total=0
realistic_parse_pass_total=0
realistic_parse_fail_total=0
realistic_expected_fail_parse_pass_total=0
realistic_preprocess_warning_total=0
realistic_preprocess_error_total=0
realistic_preprocess_total_ms=0
realistic_preprocess_max_ms=0
realistic_parse_total_ms=0
realistic_parse_max_ms=0
realistic_sample_bytes_max=0
realistic_preprocessed_bytes_max=0
declared_shadow_total=0
declared_shadow_passed=0
declared_shadow_failed=0
declared_shadow_skipped_unparseable=0
declared_shadow_report_json="$WORK_DIR/${grammar_name}_declared_identifier_shadow_report.json"
declared_shadow_cases_jsonl="$WORK_DIR/${grammar_name}_declared_identifier_shadow_cases.jsonl"
realistic_report_json="$WORK_DIR/${grammar_name}_nexsim_realistic_corpus_report.json"
realistic_cases_jsonl="$WORK_DIR/${grammar_name}_nexsim_realistic_corpus_cases.jsonl"
closed_loop_parseability_shadow_report_json="$WORK_DIR/${grammar_name}_closed_loop_parseability_shadow_report.json"
closed_loop_parseability_shadow_profiles_jsonl="$WORK_DIR/${grammar_name}_closed_loop_parseability_shadow_profiles.jsonl"
closed_loop_parseability_shadow_counterexamples_jsonl="$WORK_DIR/${grammar_name}_closed_loop_parseability_shadow_counterexamples.jsonl"
parseability_generation_counterexamples_jsonl="$WORK_DIR/${grammar_name}_parseability_generation_counterexamples.jsonl"
closed_loop_parseability_shadow_primary_entry_attempts_total=0
closed_loop_parseability_shadow_primary_entry_accepted_outputs_total=0
closed_loop_parseability_shadow_primary_entry_rejected_outputs_total=0
closed_loop_parseability_shadow_alternate_entry_attempts_total=0
closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total=0
closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total=0
if [[ "$declared_shadow_enabled" -eq 1 ]]; then
    : >"$declared_shadow_cases_jsonl"
fi
: >"$closed_loop_parseability_shadow_profiles_jsonl"
: >"$closed_loop_parseability_shadow_counterexamples_jsonl"
: >"$parseability_generation_counterexamples_jsonl"
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
        closed_loop_initial_replay_stimuli="$WORK_DIR/profile_${profile_key}_initial_replay_stimuli.sv"
        closed_loop_initial_replay_coverage="$WORK_DIR/profile_${profile_key}_initial_replay_coverage.json"
        closed_loop_initial_replay_gap_json="$WORK_DIR/profile_${profile_key}_initial_replay_gap.json"
        closed_loop_initial_replay_gap_text="$WORK_DIR/profile_${profile_key}_initial_replay_gap.txt"
        closed_loop_replay_stimuli="$WORK_DIR/profile_${profile_key}_replay_stimuli.sv"
        closed_loop_replay_coverage="$WORK_DIR/profile_${profile_key}_replay_coverage.json"
        closed_loop_replay_gap_json="$WORK_DIR/profile_${profile_key}_replay_gap.json"
        closed_loop_replay_gap_text="$WORK_DIR/profile_${profile_key}_replay_gap.txt"
        closed_loop_replay_seed=$((profile_seed_base + 700000))

        run_logged "profile_${profile_key}_closed_loop_initial" \
            "$AST_PIPELINE_BIN" "$grammar_input" \
            --generate-stimuli \
            --grammar-profile "$lrm_profile" \
            --enforce-word-boundary-spacing \
            --count "$sample_count" \
            --seed "$profile_seed_base" \
            --entry-rule "$mode_entry_rule" \
            --max-depth "$mode_max_depth" \
            --max-repeat "$mode_max_repeat" \
            --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
            "${pending_frontier_extra_stagnation_args[@]}" \
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

        run_logged "profile_${profile_key}_closed_loop_initial_replay" \
            "$AST_PIPELINE_BIN" "$grammar_input" \
            --generate-stimuli \
            --grammar-profile "$lrm_profile" \
            --enforce-word-boundary-spacing \
            --count "$sample_count" \
            --seed "$profile_seed_base" \
            --entry-rule "$mode_entry_rule" \
            --max-depth "$mode_max_depth" \
            --max-repeat "$mode_max_repeat" \
            --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
            "${pending_frontier_extra_stagnation_args[@]}" \
            --output "$closed_loop_initial_replay_stimuli" \
            --coverage-output "$closed_loop_initial_replay_coverage" \
            --gap-report-json "$closed_loop_initial_replay_gap_json" \
            --gap-report-text "$closed_loop_initial_replay_gap_text" \
            --gap-report-threshold "$gap_report_threshold"
        require_nonempty_file "$closed_loop_initial_replay_stimuli"
        require_nonempty_file "$closed_loop_initial_replay_coverage"
        require_nonempty_file "$closed_loop_initial_replay_gap_json"
        require_nonempty_file "$closed_loop_initial_replay_gap_text"
        assert_same_text "$closed_loop_initial_stimuli" "$closed_loop_initial_replay_stimuli" "sv closed-loop initial stimuli replay (${lrm_profile})"
        assert_same_json "$closed_loop_initial_coverage" "$closed_loop_initial_replay_coverage" "sv closed-loop initial coverage replay (${lrm_profile})"
        assert_same_json "$closed_loop_initial_gap_json" "$closed_loop_initial_replay_gap_json" "sv closed-loop initial gap replay (${lrm_profile})"
        assert_same_text "$closed_loop_initial_gap_text" "$closed_loop_initial_replay_gap_text" "sv closed-loop initial gap text replay (${lrm_profile})"
        closed_loop_initial_replay_determinism_pass_count=$((closed_loop_initial_replay_determinism_pass_count + 1))

        run_logged "profile_${profile_key}_closed_loop_replay" \
            env PGEN_TRACE_VERBOSITY="${PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY:-none}" \
            "$AST_PIPELINE_BIN" "$grammar_input" \
            --generate-stimuli \
            --grammar-profile "$lrm_profile" \
            --enforce-word-boundary-spacing \
            --count "$replay_sample_count" \
            --seed "$closed_loop_replay_seed" \
            --entry-rule "$mode_entry_rule" \
            --max-depth "$mode_max_depth" \
            --max-repeat "$mode_max_repeat" \
            --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
            "${pending_frontier_extra_stagnation_args[@]}" \
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

        if [[ "$closed_loop_parseability_shadow_enabled" -eq 1 ]]; then
            closed_loop_replay_parseability_shadow_stimuli="$WORK_DIR/profile_${profile_key}_replay_parseability_shadow.sv"
            closed_loop_replay_parseability_shadow_report="$WORK_DIR/profile_${profile_key}_replay_parseability_shadow_report.json"

            run_logged "profile_${profile_key}_closed_loop_replay_parseability_shadow" \
                env PGEN_TRACE_VERBOSITY="${PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY:-none}" \
                "$AST_PIPELINE_BIN" "$grammar_input" \
                --generate-stimuli \
                --grammar-profile "$lrm_profile" \
                --enforce-word-boundary-spacing \
                --count "$replay_sample_count" \
                --seed "$closed_loop_replay_seed" \
                --entry-rule "$mode_entry_rule" \
                --max-depth "$mode_max_depth" \
                --max-repeat "$mode_max_repeat" \
                --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
                "${pending_frontier_extra_stagnation_args[@]}" \
                --output "$closed_loop_replay_parseability_shadow_stimuli" \
                --target-max-attempts "$target_max_attempts" \
                --target-report-input "$closed_loop_initial_gap_json" \
                --validate-parseability \
                --parseability-report-json "$closed_loop_replay_parseability_shadow_report"
            require_nonempty_file "$closed_loop_replay_parseability_shadow_report"
            if ! jq -e ".grammar_name == \"$grammar_name\" and .summary.attempts == .summary.requested and .summary.accepted <= .summary.requested and .summary.rejected == (.summary.attempts - .summary.accepted)" "$closed_loop_replay_parseability_shadow_report" >/dev/null; then
                echo "error: closed-loop replay parseability shadow report validation failed for profile '${lrm_profile}': $closed_loop_replay_parseability_shadow_report" >&2
                exit 1
            fi

            shadow_requested="$(parseability_summary_field_u64 "$closed_loop_replay_parseability_shadow_report" "requested")"
            shadow_attempts="$(parseability_summary_field_u64 "$closed_loop_replay_parseability_shadow_report" "attempts")"
            shadow_accepted="$(parseability_summary_field_u64 "$closed_loop_replay_parseability_shadow_report" "accepted")"
            shadow_rejected="$(parseability_summary_field_u64 "$closed_loop_replay_parseability_shadow_report" "rejected")"
            shadow_parser_rejections="$(parseability_summary_field_u64 "$closed_loop_replay_parseability_shadow_report" "parser_rejections")"
            shadow_generation_errors="$(parseability_summary_field_u64 "$closed_loop_replay_parseability_shadow_report" "generation_errors")"
            shadow_empty_generations="$(parseability_summary_field_u64 "$closed_loop_replay_parseability_shadow_report" "empty_generations")"
            shadow_acceptance_rate_percent="$(parseability_acceptance_rate_percent "$closed_loop_replay_parseability_shadow_report")"
            shadow_counterexamples_captured="$(jq -er '((.counterexamples // []) | length) | numbers' "$closed_loop_replay_parseability_shadow_report")"
            shadow_primary_entry_attempts="$(parseability_target_drive_field_u64 "$closed_loop_replay_parseability_shadow_report" "primary_entry_attempts")"
            shadow_primary_entry_accepted_outputs="$(parseability_target_drive_field_u64 "$closed_loop_replay_parseability_shadow_report" "primary_entry_accepted_outputs")"
            shadow_primary_entry_rejected_outputs="$(parseability_target_drive_field_u64 "$closed_loop_replay_parseability_shadow_report" "primary_entry_rejected_outputs")"
            shadow_alternate_entry_attempts="$(parseability_target_drive_field_u64 "$closed_loop_replay_parseability_shadow_report" "alternate_entry_attempts")"
            shadow_alternate_entry_accepted_outputs="$(parseability_target_drive_field_u64 "$closed_loop_replay_parseability_shadow_report" "alternate_entry_accepted_outputs")"
            shadow_alternate_entry_rejected_outputs="$(parseability_target_drive_field_u64 "$closed_loop_replay_parseability_shadow_report" "alternate_entry_rejected_outputs")"

            closed_loop_parseability_shadow_requested_total=$((closed_loop_parseability_shadow_requested_total + shadow_requested))
            closed_loop_parseability_shadow_attempts_total=$((closed_loop_parseability_shadow_attempts_total + shadow_attempts))
            closed_loop_parseability_shadow_accepted_total=$((closed_loop_parseability_shadow_accepted_total + shadow_accepted))
            closed_loop_parseability_shadow_rejected_total=$((closed_loop_parseability_shadow_rejected_total + shadow_rejected))
            closed_loop_parseability_shadow_parser_rejections_total=$((closed_loop_parseability_shadow_parser_rejections_total + shadow_parser_rejections))
            closed_loop_parseability_shadow_generation_errors_total=$((closed_loop_parseability_shadow_generation_errors_total + shadow_generation_errors))
            closed_loop_parseability_shadow_empty_generations_total=$((closed_loop_parseability_shadow_empty_generations_total + shadow_empty_generations))
            closed_loop_parseability_shadow_counterexamples_captured_total=$((closed_loop_parseability_shadow_counterexamples_captured_total + shadow_counterexamples_captured))
            closed_loop_parseability_shadow_primary_entry_attempts_total=$((closed_loop_parseability_shadow_primary_entry_attempts_total + shadow_primary_entry_attempts))
            closed_loop_parseability_shadow_primary_entry_accepted_outputs_total=$((closed_loop_parseability_shadow_primary_entry_accepted_outputs_total + shadow_primary_entry_accepted_outputs))
            closed_loop_parseability_shadow_primary_entry_rejected_outputs_total=$((closed_loop_parseability_shadow_primary_entry_rejected_outputs_total + shadow_primary_entry_rejected_outputs))
            closed_loop_parseability_shadow_alternate_entry_attempts_total=$((closed_loop_parseability_shadow_alternate_entry_attempts_total + shadow_alternate_entry_attempts))
            closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total=$((closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total + shadow_alternate_entry_accepted_outputs))
            closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total=$((closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total + shadow_alternate_entry_rejected_outputs))
            if (( shadow_counterexamples_captured > 0 )); then
                jq -c \
                    --arg profile "$lrm_profile" \
                    '(.counterexamples // [])[] | . + {profile: $profile}' \
                    "$closed_loop_replay_parseability_shadow_report" >>"$closed_loop_parseability_shadow_counterexamples_jsonl"
            fi

            jq -cn \
                --arg profile "$lrm_profile" \
                --arg report_json "$closed_loop_replay_parseability_shadow_report" \
                --argjson requested "$shadow_requested" \
                --argjson attempts "$shadow_attempts" \
                --argjson accepted "$shadow_accepted" \
                --argjson rejected "$shadow_rejected" \
                --argjson parser_rejections "$shadow_parser_rejections" \
                --argjson generation_errors "$shadow_generation_errors" \
                --argjson empty_generations "$shadow_empty_generations" \
                --argjson acceptance_rate_percent "$shadow_acceptance_rate_percent" \
                --argjson counterexamples_captured "$shadow_counterexamples_captured" \
                --argjson primary_entry_attempts "$shadow_primary_entry_attempts" \
                --argjson primary_entry_accepted_outputs "$shadow_primary_entry_accepted_outputs" \
                --argjson primary_entry_rejected_outputs "$shadow_primary_entry_rejected_outputs" \
                --argjson alternate_entry_attempts "$shadow_alternate_entry_attempts" \
                --argjson alternate_entry_accepted_outputs "$shadow_alternate_entry_accepted_outputs" \
                --argjson alternate_entry_rejected_outputs "$shadow_alternate_entry_rejected_outputs" \
                '{
                    profile: $profile,
                    report_json: $report_json,
                    observed: {
                        requested: $requested,
                        attempts: $attempts,
                        accepted: $accepted,
                        rejected: $rejected,
                        parser_rejections: $parser_rejections,
                        generation_errors: $generation_errors,
                        empty_generations: $empty_generations,
                        acceptance_rate_percent: $acceptance_rate_percent
                    },
                    counterexamples_captured: $counterexamples_captured,
                    target_drive_validation: {
                        primary_entry_attempts: $primary_entry_attempts,
                        primary_entry_accepted_outputs: $primary_entry_accepted_outputs,
                        primary_entry_rejected_outputs: $primary_entry_rejected_outputs,
                        alternate_entry_attempts: $alternate_entry_attempts,
                        alternate_entry_accepted_outputs: $alternate_entry_accepted_outputs,
                        alternate_entry_rejected_outputs: $alternate_entry_rejected_outputs
                    }
                }' >>"$closed_loop_parseability_shadow_profiles_jsonl"
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
        parseability_report_json="$WORK_DIR/sample_${profile_key}_${idx}.parseability_generation.json"
        parseability_attempts=0
        parseability_accepted=0
        parseability_rejected=0
        parseability_parser_rejections=0
        parseability_generation_errors=0
        parseability_empty_generations=0
        parseability_acceptance_rate_percent="0"

        generate_started_ms="$(now_ms)"
        parseability_report_args=()
        if [[ "$parseability_generation_enabled" -eq 1 ]]; then
            parseability_report_args=(--parseability-report-json "$parseability_report_json")
        fi
        run_logged "sample_${profile_key}_${idx}_generate_stimulus" \
            "$AST_PIPELINE_BIN" "$grammar_input" \
            --generate-stimuli \
            --grammar-profile "$lrm_profile" \
            "${parseability_generation_args[@]}" \
            "${parseability_report_args[@]}" \
            --enforce-word-boundary-spacing \
            --count 1 \
            --seed "$seed" \
            --entry-rule "$mode_entry_rule" \
            --max-depth "$mode_max_depth" \
            --max-repeat "$mode_max_repeat" \
            --recovery-stimuli-mode "$mode_recovery_stimuli_mode" \
            "${pending_frontier_extra_stagnation_args[@]}" \
            --output "$sample_file"
        require_nonempty_file "$sample_file"
        generate_elapsed_ms=$(( $(now_ms) - generate_started_ms ))
        perf_generate_total_ms=$((perf_generate_total_ms + generate_elapsed_ms))
        if (( generate_elapsed_ms > perf_generate_max_ms )); then
            perf_generate_max_ms="$generate_elapsed_ms"
        fi
        sample_size_bytes="$(file_size_bytes "$sample_file")"
        if (( sample_size_bytes > perf_sample_bytes_max )); then
            perf_sample_bytes_max="$sample_size_bytes"
        fi
        enforce_threshold_le "$perf_budget_enabled" "stimuli_generate_ms_per_sample" "$generate_elapsed_ms" "$perf_max_generate_ms_per_sample" "profile=${lrm_profile},sample=${idx}"
        enforce_threshold_le "$perf_budget_enabled" "stimuli_sample_bytes" "$sample_size_bytes" "$perf_max_sample_bytes" "profile=${lrm_profile},sample=${idx}"
        if [[ "$parseability_generation_enabled" -eq 1 ]]; then
            require_file "$parseability_report_json"
            parseability_requested="$(jq -er '.summary.requested | numbers' "$parseability_report_json")"
            parseability_attempts="$(jq -er '.summary.attempts | numbers' "$parseability_report_json")"
            parseability_accepted="$(jq -er '.summary.accepted | numbers' "$parseability_report_json")"
            parseability_rejected="$(jq -er '.summary.rejected | numbers' "$parseability_report_json")"
            parseability_parser_rejections="$(jq -er '.summary.parser_rejections | numbers' "$parseability_report_json")"
            parseability_generation_errors="$(jq -er '.summary.generation_errors | numbers' "$parseability_report_json")"
            parseability_empty_generations="$(jq -er '.summary.empty_generations | numbers' "$parseability_report_json")"
            parseability_acceptance_rate_percent="$(jq -er '.summary | if .attempts == 0 then 0 else ((.accepted * 100.0) / .attempts) end' "$parseability_report_json")"
            parseability_generation_requested_total=$((parseability_generation_requested_total + parseability_requested))
            parseability_generation_attempts_total=$((parseability_generation_attempts_total + parseability_attempts))
            parseability_generation_accepted_total=$((parseability_generation_accepted_total + parseability_accepted))
            parseability_generation_rejected_total=$((parseability_generation_rejected_total + parseability_rejected))
            parseability_generation_parser_rejections_total=$((parseability_generation_parser_rejections_total + parseability_parser_rejections))
            parseability_generation_errors_total=$((parseability_generation_errors_total + parseability_generation_errors))
            parseability_generation_empty_generations_total=$((parseability_generation_empty_generations_total + parseability_empty_generations))
            sample_parseability_counterexamples="$(jq -er '((.counterexamples // []) | length) | numbers' "$parseability_report_json")"
            parseability_generation_counterexamples_captured_total=$((parseability_generation_counterexamples_captured_total + sample_parseability_counterexamples))
            if (( sample_parseability_counterexamples > 0 )); then
                jq -c \
                    --arg profile "$lrm_profile" \
                    --argjson sample_index "$idx" \
                    --argjson seed "$seed" \
                    '(.counterexamples // [])[] | . + {profile: $profile, sample_index: $sample_index, seed: $seed}' \
                    "$parseability_report_json" >>"$parseability_generation_counterexamples_jsonl"
            fi
        fi

        preprocess_started_ms="$(now_ms)"
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
        preprocess_elapsed_ms=$(( $(now_ms) - preprocess_started_ms ))
        perf_preprocess_total_ms=$((perf_preprocess_total_ms + preprocess_elapsed_ms))
        if (( preprocess_elapsed_ms > perf_preprocess_max_ms )); then
            perf_preprocess_max_ms="$preprocess_elapsed_ms"
        fi
        preprocessed_size_bytes="$(file_size_bytes "$preprocessed_file")"
        if (( preprocessed_size_bytes > perf_preprocessed_bytes_max )); then
            perf_preprocessed_bytes_max="$preprocessed_size_bytes"
        fi
        enforce_threshold_le "$perf_budget_enabled" "preprocess_ms_per_sample" "$preprocess_elapsed_ms" "$perf_max_preprocess_ms_per_sample" "profile=${lrm_profile},sample=${idx}"
        enforce_threshold_le "$perf_budget_enabled" "preprocessed_sample_bytes" "$preprocessed_size_bytes" "$perf_max_preprocessed_bytes" "profile=${lrm_profile},sample=${idx}"

        require_file "$diagnostics_json"
        warning_count="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$diagnostics_json")"
        error_count="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$diagnostics_json")"
        total_warnings=$((total_warnings + warning_count))
        total_errors=$((total_errors + error_count))

        parse_status="skip"
        parse_note="parse_full stage skipped"
        if [[ "$parse_full_enabled" -eq 1 ]]; then
            parse_log="$LOG_DIR/sample_${profile_key}_${idx}_parse_full.log"
            echo "==> sample_${profile_key}_${idx}_parse_full"
            parse_started_ms="$(now_ms)"
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
            parse_elapsed_ms=$(( $(now_ms) - parse_started_ms ))
            perf_parse_full_total_ms=$((perf_parse_full_total_ms + parse_elapsed_ms))
            perf_parse_full_samples=$((perf_parse_full_samples + 1))
            if (( parse_elapsed_ms > perf_parse_full_max_ms )); then
                perf_parse_full_max_ms="$parse_elapsed_ms"
            fi
            enforce_threshold_le "$perf_budget_enabled" "parse_full_ms_per_sample" "$parse_elapsed_ms" "$perf_max_parse_full_ms_per_sample" "profile=${lrm_profile},sample=${idx}"
        else
            parse_full_skip_count=$((parse_full_skip_count + 1))
            parse_note="parse_full unavailable (${parse_full_effective})"
        fi

        semantic_status="pass"
        semantic_note="baseline semantic validation passed"
        if ! evaluate_semantic_baseline "$preprocessed_file" "$error_count" "$parse_status"; then
            semantic_status="fail"
            semantic_note="$SEMANTIC_EVAL_NOTE"
        fi

        if [[ "$semantic_status" != "pass" ]]; then
            if [[ "$failure_replay_enabled" -eq 1 ]] && [[ "$shrink_semantic_failures" -eq 1 ]]; then
                semantic_shrink_file="$WORK_DIR/sample_${profile_key}_${idx}.semantic.shrunk.sv"
                semantic_shrink_lines="$(deterministic_prefix_shrink "$preprocessed_file" "$semantic_shrink_file" "$failure_shrink_max_iterations" semantic_failure_predicate "$error_count" "$parse_status")"
                semantic_note="${semantic_note}; shrunk_failure=${semantic_shrink_file}; shrunk_lines=${semantic_shrink_lines}"
                semantic_shrink_count=$((semantic_shrink_count + 1))
            fi
            echo "${lrm_profile},${idx},${seed},${profile_closed_loop_initial_status},${profile_closed_loop_replay_status},pass,${parseability_attempts},${parseability_accepted},${parseability_rejected},${parseability_parser_rejections},${parseability_generation_errors},${parseability_empty_generations},${parseability_acceptance_rate_percent},pass,fail,${parse_status},${warning_count},${error_count},fail,$(csv_sanitize "$semantic_note")" >>"$SUMMARY_CSV"
            echo "error: semantic baseline validation failed for profile '${lrm_profile}' sample_${idx}: ${semantic_note}" >&2
            exit 1
        fi
        semantic_pass_count=$((semantic_pass_count + 1))

        if [[ "$declared_shadow_enabled" -eq 1 ]]; then
            declared_shadow_case_note=""
            declared_shadow_case_status="pass"
            if [[ "$DECLARED_SHADOW_PARSEABLE_ONLY" -eq 1 && "$parse_status" != "pass" ]]; then
                declared_shadow_case_status="skip_unparseable"
                declared_shadow_case_note="declared identifier shadow check skipped because parse_full status is '${parse_status}'"
                declared_shadow_skipped_unparseable=$((declared_shadow_skipped_unparseable + 1))
            else
                declared_shadow_total=$((declared_shadow_total + 1))
                if declared_shadow_case_note="$(check_declared_identifiers_before_use "$preprocessed_file" 2>&1)"; then
                    declared_shadow_passed=$((declared_shadow_passed + 1))
                    if [[ -z "$declared_shadow_case_note" ]]; then
                        declared_shadow_case_note="declared identifier shadow check passed"
                    fi
                else
                    declared_shadow_case_status="fail"
                    declared_shadow_failed=$((declared_shadow_failed + 1))
                    if [[ -z "$declared_shadow_case_note" ]]; then
                        declared_shadow_case_note="declared identifier shadow check failed"
                    fi
                fi
            fi

            jq -n \
                --arg profile "$lrm_profile" \
                --argjson sample_index "$idx" \
                --argjson seed "$seed" \
                --arg status "$declared_shadow_case_status" \
                --arg note "$declared_shadow_case_note" \
                --arg sample_file "$preprocessed_file" \
                '{
                    profile: $profile,
                    sample_index: $sample_index,
                    seed: $seed,
                    status: $status,
                    note: $note,
                    sample_file: $sample_file
                }' >>"$declared_shadow_cases_jsonl"
        fi

        final_note="$parse_note"
        echo "${lrm_profile},${idx},${seed},${profile_closed_loop_initial_status},${profile_closed_loop_replay_status},pass,${parseability_attempts},${parseability_accepted},${parseability_rejected},${parseability_parser_rejections},${parseability_generation_errors},${parseability_empty_generations},${parseability_acceptance_rate_percent},pass,${semantic_status},${parse_status},${warning_count},${error_count},pass,$(csv_sanitize "$final_note")" >>"$SUMMARY_CSV"
    done
done

if [[ -s "$declared_shadow_cases_jsonl" ]]; then
    declared_shadow_cases_json="$(jq -s '.' "$declared_shadow_cases_jsonl")"
else
    declared_shadow_cases_json='[]'
fi

jq -n \
    --arg grammar_name "$grammar_name" \
    --arg requested_mode "$DECLARED_SHADOW_MODE" \
    --arg effective_mode "$declared_shadow_effective" \
    --argjson parseable_only "$DECLARED_SHADOW_PARSEABLE_ONLY" \
    --arg note "$declared_shadow_note" \
    --argjson enabled "$declared_shadow_enabled" \
    --argjson strict "$declared_shadow_strict" \
    --argjson total "$declared_shadow_total" \
    --argjson passed "$declared_shadow_passed" \
    --argjson failed "$declared_shadow_failed" \
    --argjson skipped_unparseable "$declared_shadow_skipped_unparseable" \
    --argjson cases "$declared_shadow_cases_json" \
    '{
        grammar_name: $grammar_name,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        parseable_only: ($parseable_only == 1),
        note: $note,
        enabled: $enabled,
        strict: $strict,
        totals: {
            checked: $total,
            passed: $passed,
            failed: $failed,
            skipped_unparseable: $skipped_unparseable
        },
        cases: $cases
    }' >"$declared_shadow_report_json"

if [[ "$declared_shadow_enabled" -eq 1 && "$declared_shadow_strict" -eq 1 && "$DECLARED_SHADOW_PARSEABLE_ONLY" -eq 1 && "$declared_shadow_total" -eq 0 ]]; then
    echo "error: strict declared-identifier shadow mode requires at least one parseable sample when parseable-only filtering is enabled" >&2
    cat "$declared_shadow_report_json" >&2
    exit 1
fi

if [[ "$declared_shadow_enabled" -eq 1 && "$declared_shadow_strict" -eq 1 && "$declared_shadow_failed" -gt 0 ]]; then
    echo "error: strict declared-identifier shadow mode detected failures ($declared_shadow_failed/$declared_shadow_total)" >&2
    cat "$declared_shadow_report_json" >&2
    exit 1
fi

realistic_cases_json='[]'
if [[ "$realistic_corpus_enabled" -eq 1 ]]; then
    : >"$realistic_cases_jsonl"
    mapfile -t realistic_case_rows < <(jq -c '.cases[]?' "$realistic_corpus_path")
    realistic_cases_declared="${#realistic_case_rows[@]}"
    if (( realistic_cases_declared == 0 )); then
        echo "error: realistic corpus has zero cases: $realistic_corpus_path" >&2
        exit 1
    fi

    realistic_case_manifest_idx=0
    for case_json in "${realistic_case_rows[@]}"; do
        if (( REALISTIC_CORPUS_MAX_CASES > 0 && realistic_case_manifest_idx >= REALISTIC_CORPUS_MAX_CASES )); then
            break
        fi
        realistic_case_manifest_idx=$((realistic_case_manifest_idx + 1))

        case_name="$(jq -er '.name | strings' <<<"$case_json")"
        case_source_rel="$(jq -er '.path | strings' <<<"$case_json")"
        case_expect_parse_full_pass="$(jq -er 'if has("expect_parse_full_pass") then (if .expect_parse_full_pass then 1 else 0 end) else 1 end' <<<"$case_json")"
        case_profiles_csv="$(jq -er '(.profiles // []) | map(select(type=="string")) | join(",")' <<<"$case_json")"
        case_source_path="$(resolve_path "$case_source_rel")"
        case_source_dir="$(dirname "$case_source_path")"
        require_file "$case_source_path"

        if [[ "$case_expect_parse_full_pass" -eq 1 ]]; then
            realistic_expected_pass_total=$((realistic_expected_pass_total + 1))
        else
            realistic_expected_fail_total=$((realistic_expected_fail_total + 1))
        fi

        declare -a case_profiles=()
        if [[ -n "$case_profiles_csv" ]]; then
            IFS=',' read -r -a _case_profiles_raw <<< "$case_profiles_csv"
            for _raw_profile in "${_case_profiles_raw[@]}"; do
                _profile="$(echo "$_raw_profile" | tr -d '[:space:]')"
                if [[ -n "$_profile" ]]; then
                    case_profiles+=("$_profile")
                fi
            done
        fi
        if [[ "${#case_profiles[@]}" -eq 0 ]]; then
            case_profiles=("${run_profiles[@]}")
        fi

        case_name_key="$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_' '_')"
        for case_profile in "${case_profiles[@]}"; do
            if [[ -z "${supported_profiles_map[$case_profile]:-}" ]]; then
                echo "error: realistic corpus case '$case_name' uses unsupported LRM profile '$case_profile'" >&2
                exit 1
            fi

            case_profile_key="$(printf '%s' "$case_profile" | tr -c 'A-Za-z0-9_' '_')"
            case_input_file="$WORK_DIR/realistic_case_${case_name_key}_${case_profile_key}.sv"
            case_preprocessed_file="$WORK_DIR/realistic_case_${case_name_key}_${case_profile_key}.preprocessed.sv"
            case_diagnostics_json="$WORK_DIR/realistic_case_${case_name_key}_${case_profile_key}.diagnostics.json"
            case_parse_log="$LOG_DIR/realistic_case_${case_name_key}_${case_profile_key}_parse_full.log"

            cp "$case_source_path" "$case_input_file"

            case_sample_bytes="$(file_size_bytes "$case_input_file")"
            if (( case_sample_bytes > realistic_sample_bytes_max )); then
                realistic_sample_bytes_max="$case_sample_bytes"
            fi
            enforce_threshold_le 1 "realistic_sample_bytes" "$case_sample_bytes" "$realistic_max_sample_bytes" "case=${case_name},profile=${case_profile}"

            case_preprocess_started_ms="$(now_ms)"
            run_logged "realistic_case_${case_name_key}_${case_profile_key}_preprocess" \
                "$AST_PIPELINE_BIN" "$case_input_file" \
                --preprocess-systemverilog \
                --output "$case_preprocessed_file" \
                --sv-diagnostics-json "$case_diagnostics_json" \
                --sv-include-dir "$case_source_dir" \
                --sv-include-max-depth "$include_max_depth" \
                --sv-include-path-policy "$include_path_policy" \
                --sv-macro-redefine-policy "$macro_redefine_policy" \
                --sv-conditional-symbol-policy "$conditional_symbol_policy" \
                --sv-conditional-expr-policy "$conditional_expr_policy" \
                --sv-strict-warning-codes "$strict_warning_codes"
            case_preprocess_elapsed_ms=$(( $(now_ms) - case_preprocess_started_ms ))
            realistic_preprocess_total_ms=$((realistic_preprocess_total_ms + case_preprocess_elapsed_ms))
            if (( case_preprocess_elapsed_ms > realistic_preprocess_max_ms )); then
                realistic_preprocess_max_ms="$case_preprocess_elapsed_ms"
            fi
            enforce_threshold_le 1 "realistic_preprocess_ms_per_case" "$case_preprocess_elapsed_ms" "$realistic_max_preprocess_ms_per_case" "case=${case_name},profile=${case_profile}"

            case_preprocessed_bytes="$(file_size_bytes "$case_preprocessed_file")"
            if (( case_preprocessed_bytes > realistic_preprocessed_bytes_max )); then
                realistic_preprocessed_bytes_max="$case_preprocessed_bytes"
            fi
            enforce_threshold_le 1 "realistic_preprocessed_bytes" "$case_preprocessed_bytes" "$realistic_max_preprocessed_bytes" "case=${case_name},profile=${case_profile}"

            case_warning_count="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$case_diagnostics_json")"
            case_error_count="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$case_diagnostics_json")"
            realistic_preprocess_warning_total=$((realistic_preprocess_warning_total + case_warning_count))
            realistic_preprocess_error_total=$((realistic_preprocess_error_total + case_error_count))
            if [[ "$realistic_require_no_preprocess_errors" -eq 1 && "$case_error_count" -gt 0 ]]; then
                echo "error: realistic corpus case '$case_name' profile '$case_profile' has preprocess diagnostics errors ($case_error_count)" >&2
                exit 1
            fi

            case_parse_started_ms="$(now_ms)"
            if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$case_preprocessed_file" >"$case_parse_log" 2>&1; then
                case_parse_status="pass"
                realistic_parse_pass_total=$((realistic_parse_pass_total + 1))
            else
                case_parse_status="fail"
                realistic_parse_fail_total=$((realistic_parse_fail_total + 1))
            fi
            case_parse_elapsed_ms=$(( $(now_ms) - case_parse_started_ms ))
            realistic_parse_total_ms=$((realistic_parse_total_ms + case_parse_elapsed_ms))
            if (( case_parse_elapsed_ms > realistic_parse_max_ms )); then
                realistic_parse_max_ms="$case_parse_elapsed_ms"
            fi
            enforce_threshold_le 1 "realistic_parse_full_ms_per_case" "$case_parse_elapsed_ms" "$realistic_max_parse_full_ms_per_case" "case=${case_name},profile=${case_profile}"

            case_status="pass"
            case_note="parse_full status '${case_parse_status}' matched minimum expectation"
            if [[ "$case_expect_parse_full_pass" -eq 1 && "$case_parse_status" != "pass" ]]; then
                case_status="fail"
                case_note="expected parse_full pass but observed fail"
                echo "error: realistic corpus case '$case_name' profile '$case_profile' failed required parse_full pass" >&2
                tail -n 80 "$case_parse_log" >&2 || true
                exit 1
            elif [[ "$case_expect_parse_full_pass" -eq 0 && "$case_parse_status" == "pass" ]]; then
                realistic_expected_fail_parse_pass_total=$((realistic_expected_fail_parse_pass_total + 1))
                case_note="parse_full passed on expected-fail case (coverage improvement signal)"
            fi

            jq -n \
                --arg case_name "$case_name" \
                --arg profile "$case_profile" \
                --arg source_file "$case_source_path" \
                --arg sample_file "$case_input_file" \
                --arg preprocessed_file "$case_preprocessed_file" \
                --arg diagnostics_file "$case_diagnostics_json" \
                --arg parse_log_file "$case_parse_log" \
                --arg parse_status "$case_parse_status" \
                --arg status "$case_status" \
                --arg note "$case_note" \
                --argjson expect_parse_full_pass "$case_expect_parse_full_pass" \
                --argjson preprocess_ms "$case_preprocess_elapsed_ms" \
                --argjson parse_full_ms "$case_parse_elapsed_ms" \
                --argjson sample_bytes "$case_sample_bytes" \
                --argjson preprocessed_bytes "$case_preprocessed_bytes" \
                --argjson preprocess_warnings "$case_warning_count" \
                --argjson preprocess_errors "$case_error_count" \
                '{
                    case_name: $case_name,
                    profile: $profile,
                    source_file: $source_file,
                    sample_file: $sample_file,
                    preprocessed_file: $preprocessed_file,
                    diagnostics_file: $diagnostics_file,
                    parse_log_file: $parse_log_file,
                    expect_parse_full_pass: ($expect_parse_full_pass == 1),
                    parse_status: $parse_status,
                    status: $status,
                    note: $note,
                    observed: {
                        preprocess_ms: $preprocess_ms,
                        parse_full_ms: $parse_full_ms,
                        sample_bytes: $sample_bytes,
                        preprocessed_bytes: $preprocessed_bytes,
                        preprocess_warnings: $preprocess_warnings,
                        preprocess_errors: $preprocess_errors
                    }
                }' >>"$realistic_cases_jsonl"

            realistic_cases_executed=$((realistic_cases_executed + 1))
        done
    done
fi

if [[ -s "$realistic_cases_jsonl" ]]; then
    realistic_cases_json="$(jq -s '.' "$realistic_cases_jsonl")"
fi

jq -n \
    --arg grammar_name "$grammar_name" \
    --arg requested_mode "$REALISTIC_CORPUS_MODE" \
    --arg effective_mode "$realistic_corpus_effective" \
    --arg note "$realistic_corpus_note" \
    --arg corpus_path "${realistic_corpus_path:-}" \
    --argjson max_cases "$REALISTIC_CORPUS_MAX_CASES" \
    --argjson enabled "$realistic_corpus_enabled" \
    --argjson contract_enforced "$realistic_corpus_contract_enforced" \
    --argjson cases_declared "$realistic_cases_declared" \
    --argjson cases_executed "$realistic_cases_executed" \
    --argjson expected_pass_total "$realistic_expected_pass_total" \
    --argjson expected_fail_total "$realistic_expected_fail_total" \
    --argjson observed_parse_pass_total "$realistic_parse_pass_total" \
    --argjson observed_parse_fail_total "$realistic_parse_fail_total" \
    --argjson expected_fail_parse_pass_total "$realistic_expected_fail_parse_pass_total" \
    --argjson preprocess_warning_total "$realistic_preprocess_warning_total" \
    --argjson preprocess_error_total "$realistic_preprocess_error_total" \
    --argjson preprocess_total_ms "$realistic_preprocess_total_ms" \
    --argjson preprocess_max_ms "$realistic_preprocess_max_ms" \
    --argjson parse_total_ms "$realistic_parse_total_ms" \
    --argjson parse_max_ms "$realistic_parse_max_ms" \
    --argjson sample_bytes_max "$realistic_sample_bytes_max" \
    --argjson preprocessed_bytes_max "$realistic_preprocessed_bytes_max" \
    --argjson max_preprocess_ms_per_case "$realistic_max_preprocess_ms_per_case" \
    --argjson max_parse_full_ms_per_case "$realistic_max_parse_full_ms_per_case" \
    --argjson max_sample_bytes "$realistic_max_sample_bytes" \
    --argjson max_preprocessed_bytes "$realistic_max_preprocessed_bytes" \
    --argjson require_no_preprocess_errors "$realistic_require_no_preprocess_errors" \
    --argjson cases "$realistic_cases_json" \
    '{
        grammar_name: $grammar_name,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        note: $note,
        corpus_path: $corpus_path,
        max_cases: $max_cases,
        enabled: $enabled,
        contract_enforced: $contract_enforced,
        thresholds: {
            max_preprocess_ms_per_case: $max_preprocess_ms_per_case,
            max_parse_full_ms_per_case: $max_parse_full_ms_per_case,
            max_sample_bytes: $max_sample_bytes,
            max_preprocessed_bytes: $max_preprocessed_bytes,
            require_no_preprocess_errors: ($require_no_preprocess_errors == 1)
        },
        totals: {
            cases_declared: $cases_declared,
            cases_executed: $cases_executed,
            expected_pass_total: $expected_pass_total,
            expected_fail_total: $expected_fail_total,
            observed_parse_pass_total: $observed_parse_pass_total,
            observed_parse_fail_total: $observed_parse_fail_total,
            expected_fail_parse_pass_total: $expected_fail_parse_pass_total,
            preprocess_warning_total: $preprocess_warning_total,
            preprocess_error_total: $preprocess_error_total,
            preprocess_total_ms: $preprocess_total_ms,
            preprocess_max_ms: $preprocess_max_ms,
            parse_total_ms: $parse_total_ms,
            parse_max_ms: $parse_max_ms,
            sample_bytes_max: $sample_bytes_max,
            preprocessed_bytes_max: $preprocessed_bytes_max
        },
        cases: $cases
    }' >"$realistic_report_json"

if [[ "$realistic_corpus_enabled" -eq 1 && "$realistic_cases_executed" -eq 0 ]]; then
    echo "error: realistic corpus validation is enabled but no cases executed" >&2
    exit 1
fi

diff_report_json="$WORK_DIR/${grammar_name}_differential_report.json"
diff_cases_jsonl="$WORK_DIR/${grammar_name}_differential_cases.jsonl"
diff_effective_mode="disabled"
diff_note="trusted-reference differential disabled by configuration"
diff_reference_runner="$DIFF_REFERENCE_RUNNER"
diff_total_samples_seen=0
diff_samples_checked=0
diff_mismatch_count=0
diff_match_count=0
diff_rust_failed_reference_passed_count=0
diff_reference_failed_rust_passed_count=0
diff_both_failed_count=0
diff_reference_artifact_missing_count=0

if [[ "$DIFF_MODE" != "0" ]]; then
    if [[ "$mode_parse_full_eligible" -ne 1 ]]; then
        if [[ "$DIFF_MODE" == "1" ]]; then
            echo "error: strict differential mode requires parse_full-eligible stimuli mode (current: '$stimuli_mode')" >&2
            exit 1
        fi
        diff_effective_mode="disabled_by_stimuli_mode"
        diff_note="differential parse taxonomy disabled because stimuli mode '$stimuli_mode' is not parse_full-eligible"
    elif [[ "$parse_full_supported" -ne 1 ]]; then
        if [[ "$DIFF_MODE" == "1" ]]; then
            echo "error: strict differential mode requires generated parser adapter for '$grammar_name'" >&2
            exit 1
        fi
        diff_effective_mode="unsupported_adapter"
        diff_note="differential parse taxonomy disabled because generated parser adapter is unavailable"
    elif [[ -z "$DIFF_REFERENCE_RUNNER" ]]; then
        if [[ "$DIFF_MODE" == "1" ]]; then
            echo "error: strict differential mode requires PGEN_SV_STIMULI_REFERENCE_RUNNER" >&2
            exit 1
        fi
        diff_effective_mode="unsupported_reference_runner"
        diff_note="trusted-reference runner not configured; set PGEN_SV_STIMULI_REFERENCE_RUNNER"
    elif [[ ! -x "$DIFF_REFERENCE_RUNNER" ]]; then
        if [[ "$DIFF_MODE" == "1" ]]; then
            echo "error: strict differential mode requires executable trusted-reference runner at '$DIFF_REFERENCE_RUNNER'" >&2
            exit 1
        fi
        diff_effective_mode="unsupported_reference_runner"
        diff_note="trusted-reference runner path is not executable: $DIFF_REFERENCE_RUNNER"
    else
        diff_effective_mode="enabled"
        diff_note="trusted-reference parse differential classification enabled"
        : >"$diff_cases_jsonl"
        diff_case_index=0
        for profile_idx in "${!run_profiles[@]}"; do
            lrm_profile="${run_profiles[$profile_idx]}"
            profile_key="${lrm_profile//[^A-Za-z0-9_]/_}"
            for ((idx = 0; idx < sample_count; idx++)); do
                sample_preprocessed="$WORK_DIR/sample_${profile_key}_${idx}.preprocessed.sv"
                if [[ ! -s "$sample_preprocessed" ]]; then
                    continue
                fi
                diff_total_samples_seen=$((diff_total_samples_seen + 1))
                if (( diff_samples_checked >= DIFF_MAX_SAMPLES )); then
                    continue
                fi

                diff_rust_log="$LOG_DIR/diff_sample_${profile_key}_${idx}.rust.log"
                diff_ref_log="$LOG_DIR/diff_sample_${profile_key}_${idx}.reference.log"
                diff_ref_ast="$WORK_DIR/diff_sample_${profile_key}_${idx}.reference.ast.json"
                diff_ref_diag="$WORK_DIR/diff_sample_${profile_key}_${idx}.reference.diagnostics.json"

                rust_exit=0
                if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$sample_preprocessed" >"$diff_rust_log" 2>&1; then
                    rust_exit=0
                else
                    rust_exit=$?
                fi

                ref_exit=0
                if "$DIFF_REFERENCE_RUNNER" "$sample_preprocessed" "$diff_ref_ast" "$diff_ref_diag" >"$diff_ref_log" 2>&1; then
                    ref_exit=0
                else
                    ref_exit=$?
                fi

                category=""
                if (( rust_exit == 0 && ref_exit == 0 )); then
                    if [[ ! -s "$diff_ref_diag" ]]; then
                        category="reference_artifact_missing"
                    elif ! jq -e 'type == "array"' "$diff_ref_diag" >/dev/null 2>&1; then
                        category="reference_artifact_missing"
                    else
                        category="match"
                    fi
                elif (( rust_exit != 0 && ref_exit == 0 )); then
                    category="rust_failed_reference_passed"
                elif (( rust_exit == 0 && ref_exit != 0 )); then
                    category="reference_failed_rust_passed"
                else
                    category="both_failed"
                fi

                case "$category" in
                    match)
                        diff_match_count=$((diff_match_count + 1))
                        ;;
                    rust_failed_reference_passed)
                        diff_rust_failed_reference_passed_count=$((diff_rust_failed_reference_passed_count + 1))
                        diff_mismatch_count=$((diff_mismatch_count + 1))
                        ;;
                    reference_failed_rust_passed)
                        diff_reference_failed_rust_passed_count=$((diff_reference_failed_rust_passed_count + 1))
                        diff_mismatch_count=$((diff_mismatch_count + 1))
                        ;;
                    both_failed)
                        diff_both_failed_count=$((diff_both_failed_count + 1))
                        ;;
                    reference_artifact_missing)
                        diff_reference_artifact_missing_count=$((diff_reference_artifact_missing_count + 1))
                        diff_mismatch_count=$((diff_mismatch_count + 1))
                        ;;
                    *)
                        echo "error: unknown differential mismatch category '$category'" >&2
                        exit 1
                        ;;
                esac

                jq -n \
                    --argjson index "$diff_case_index" \
                    --arg profile "$lrm_profile" \
                    --argjson sample_index "$idx" \
                    --arg category "$category" \
                    --arg sample_file "$sample_preprocessed" \
                    --arg rust_log "$diff_rust_log" \
                    --arg reference_log "$diff_ref_log" \
                    --arg reference_ast "$diff_ref_ast" \
                    --arg reference_diagnostics "$diff_ref_diag" \
                    --argjson rust_exit "$rust_exit" \
                    --argjson reference_exit "$ref_exit" \
                    '{
                        index: $index,
                        profile: $profile,
                        sample_index: $sample_index,
                        category: $category,
                        sample_file: $sample_file,
                        rust: {
                            log_file: $rust_log,
                            exit_code: $rust_exit
                        },
                        reference: {
                            log_file: $reference_log,
                            ast_file: $reference_ast,
                            diagnostics_file: $reference_diagnostics,
                            exit_code: $reference_exit
                        }
                    }' >>"$diff_cases_jsonl"

                diff_samples_checked=$((diff_samples_checked + 1))
                diff_case_index=$((diff_case_index + 1))
            done
        done
    fi
fi

if [[ -s "$diff_cases_jsonl" ]]; then
    diff_cases_json="$(jq -s '.' "$diff_cases_jsonl")"
else
    diff_cases_json='[]'
fi

jq -n \
    --arg grammar_name "$grammar_name" \
    --arg requested_mode "$DIFF_MODE" \
    --arg effective_mode "$diff_effective_mode" \
    --arg note "$diff_note" \
    --arg reference_runner "$diff_reference_runner" \
    --argjson total_samples_seen "$diff_total_samples_seen" \
    --argjson max_samples "$DIFF_MAX_SAMPLES" \
    --argjson samples_checked "$diff_samples_checked" \
    --argjson mismatch_count "$diff_mismatch_count" \
    --argjson match_count "$diff_match_count" \
    --argjson rust_failed_reference_passed_count "$diff_rust_failed_reference_passed_count" \
    --argjson reference_failed_rust_passed_count "$diff_reference_failed_rust_passed_count" \
    --argjson both_failed_count "$diff_both_failed_count" \
    --argjson reference_artifact_missing_count "$diff_reference_artifact_missing_count" \
    --argjson cases "$diff_cases_json" \
    '{
        grammar_name: $grammar_name,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        note: $note,
        reference_runner: $reference_runner,
        total_samples_seen: $total_samples_seen,
        max_samples: $max_samples,
        samples_checked: $samples_checked,
        mismatch_count: $mismatch_count,
        taxonomy_counts: {
            match: $match_count,
            rust_failed_reference_passed: $rust_failed_reference_passed_count,
            reference_failed_rust_passed: $reference_failed_rust_passed_count,
            both_failed: $both_failed_count,
            reference_artifact_missing: $reference_artifact_missing_count
        },
        cases: $cases
    }' >"$diff_report_json"

if [[ "$DIFF_MODE" == "1" && "$diff_effective_mode" == "enabled" && "$diff_mismatch_count" -gt 0 ]]; then
    echo "error: strict SV stimuli differential mode detected mismatches ($diff_mismatch_count)" >&2
    cat "$diff_report_json" >&2
    exit 1
fi

perf_generate_avg_ms=0
perf_preprocess_avg_ms=0
perf_parse_full_avg_ms=0
parse_full_samples_total=0
parse_full_pass_ratio_percent=0
if (( total_samples > 0 )); then
    perf_generate_avg_ms=$((perf_generate_total_ms / total_samples))
    perf_preprocess_avg_ms=$((perf_preprocess_total_ms / total_samples))
fi
if (( perf_parse_full_samples > 0 )); then
    perf_parse_full_avg_ms=$((perf_parse_full_total_ms / perf_parse_full_samples))
fi
parse_full_samples_total=$((parse_full_pass_count + parse_full_fail_count))
if (( parse_full_samples_total > 0 )); then
    parse_full_pass_ratio_percent=$((parse_full_pass_count * 100 / parse_full_samples_total))
fi
parse_full_quality_effective="observed_only"
parse_full_quality_note="parse_full pass ratio is reported for telemetry only"
if [[ "$parse_full_quality_contract_enforced" -eq 1 ]]; then
    if [[ "$parse_full_enabled" -ne 1 ]]; then
        parse_full_quality_effective="strict_unavailable_parse_full"
        parse_full_quality_note="strict parse_full ratio enforcement requested but parse_full stage is unavailable (${parse_full_effective})"
        echo "error: ${parse_full_quality_note}" >&2
        exit 1
    fi
    parse_full_quality_effective="strict_enforced"
    parse_full_quality_note="strict parse_full ratio enforcement is enabled"
    if (( parse_full_pass_ratio_percent < parse_full_quality_min_pass_ratio )); then
        echo "error: strict parse_full pass ratio check failed (${parse_full_pass_ratio_percent}% < ${parse_full_quality_min_pass_ratio}%)" >&2
        exit 1
    fi
else
    if [[ "$parse_full_enabled" -ne 1 ]]; then
        parse_full_quality_effective="skipped_parse_full_unavailable"
        parse_full_quality_note="parse_full ratio telemetry skipped because parse_full stage is unavailable (${parse_full_effective})"
    fi
fi

parse_full_quality_report_json="$WORK_DIR/${grammar_name}_parse_full_quality_report.json"
parseability_generation_acceptance_rate_percent="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$parseability_generation_accepted_total" "$parseability_generation_attempts_total")"
parseability_generation_report_json="$WORK_DIR/${grammar_name}_parseability_generation_report.json"
parseability_generation_counterexamples_json="[]"
if [[ -s "$parseability_generation_counterexamples_jsonl" ]]; then
    parseability_generation_counterexamples_json="$(jq -s '.[0:20]' "$parseability_generation_counterexamples_jsonl")"
fi
closed_loop_parseability_shadow_acceptance_rate_percent="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$closed_loop_parseability_shadow_accepted_total" "$closed_loop_parseability_shadow_attempts_total")"
closed_loop_parseability_shadow_counterexamples_json="[]"
if [[ -s "$closed_loop_parseability_shadow_counterexamples_jsonl" ]]; then
    closed_loop_parseability_shadow_counterexamples_json="$(jq -s '.[0:20]' "$closed_loop_parseability_shadow_counterexamples_jsonl")"
fi
closed_loop_parseability_shadow_profiles_json="[]"
if [[ -s "$closed_loop_parseability_shadow_profiles_jsonl" ]]; then
    closed_loop_parseability_shadow_profiles_json="$(jq -s '.' "$closed_loop_parseability_shadow_profiles_jsonl")"
fi
jq -n \
    --arg grammar_name "$grammar_name" \
    --arg effective_mode "$closed_loop_parseability_shadow_effective" \
    --arg note "$closed_loop_parseability_shadow_note" \
    --argjson enabled "$closed_loop_parseability_shadow_enabled" \
    --argjson requested_total "$closed_loop_parseability_shadow_requested_total" \
    --argjson attempts_total "$closed_loop_parseability_shadow_attempts_total" \
    --argjson accepted_total "$closed_loop_parseability_shadow_accepted_total" \
    --argjson rejected_total "$closed_loop_parseability_shadow_rejected_total" \
    --argjson parser_rejections_total "$closed_loop_parseability_shadow_parser_rejections_total" \
    --argjson generation_errors_total "$closed_loop_parseability_shadow_generation_errors_total" \
    --argjson empty_generations_total "$closed_loop_parseability_shadow_empty_generations_total" \
    --argjson acceptance_rate_percent "$closed_loop_parseability_shadow_acceptance_rate_percent" \
    --argjson counterexamples_captured_total "$closed_loop_parseability_shadow_counterexamples_captured_total" \
    --argjson counterexamples "$closed_loop_parseability_shadow_counterexamples_json" \
    --argjson primary_entry_attempts_total "$closed_loop_parseability_shadow_primary_entry_attempts_total" \
    --argjson primary_entry_accepted_outputs_total "$closed_loop_parseability_shadow_primary_entry_accepted_outputs_total" \
    --argjson primary_entry_rejected_outputs_total "$closed_loop_parseability_shadow_primary_entry_rejected_outputs_total" \
    --argjson alternate_entry_attempts_total "$closed_loop_parseability_shadow_alternate_entry_attempts_total" \
    --argjson alternate_entry_accepted_outputs_total "$closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total" \
    --argjson alternate_entry_rejected_outputs_total "$closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total" \
    --argjson profiles "$closed_loop_parseability_shadow_profiles_json" \
    '{
        grammar_name: $grammar_name,
        enabled: ($enabled == 1),
        effective_mode: $effective_mode,
        note: $note,
        observed: {
            requested_total: $requested_total,
            attempts_total: $attempts_total,
            accepted_total: $accepted_total,
            rejected_total: $rejected_total,
            parser_rejections_total: $parser_rejections_total,
            generation_errors_total: $generation_errors_total,
            empty_generations_total: $empty_generations_total,
            acceptance_rate_percent: $acceptance_rate_percent
        },
        counterexamples_captured_total: $counterexamples_captured_total,
        counterexamples: $counterexamples,
        target_drive_validation: {
            primary_entry_attempts_total: $primary_entry_attempts_total,
            primary_entry_accepted_outputs_total: $primary_entry_accepted_outputs_total,
            primary_entry_rejected_outputs_total: $primary_entry_rejected_outputs_total,
            alternate_entry_attempts_total: $alternate_entry_attempts_total,
            alternate_entry_accepted_outputs_total: $alternate_entry_accepted_outputs_total,
            alternate_entry_rejected_outputs_total: $alternate_entry_rejected_outputs_total
        },
        profiles: $profiles
    }' >"$closed_loop_parseability_shadow_report_json"
jq -n \
    --arg grammar_name "$grammar_name" \
    --arg note "$parseability_generation_note" \
    --argjson enabled "$parseability_generation_enabled" \
    --argjson requested_total "$parseability_generation_requested_total" \
    --argjson accepted_total "$parseability_generation_accepted_total" \
    --argjson rejected_total "$parseability_generation_rejected_total" \
    --argjson attempts_total "$parseability_generation_attempts_total" \
    --argjson parser_rejections_total "$parseability_generation_parser_rejections_total" \
    --argjson generation_errors_total "$parseability_generation_errors_total" \
    --argjson empty_generations_total "$parseability_generation_empty_generations_total" \
    --argjson acceptance_rate_percent "$parseability_generation_acceptance_rate_percent" \
    --argjson counterexamples "$parseability_generation_counterexamples_json" \
    '{
        grammar_name: $grammar_name,
        enabled: ($enabled == 1),
        note: $note,
        observed: {
            requested_total: $requested_total,
            accepted_total: $accepted_total,
            rejected_total: $rejected_total,
            attempts_total: $attempts_total,
            parser_rejections_total: $parser_rejections_total,
            generation_errors_total: $generation_errors_total,
            empty_generations_total: $empty_generations_total,
            acceptance_rate_percent: $acceptance_rate_percent
        },
        counterexamples: $counterexamples
    }' >"$parseability_generation_report_json"

jq -n \
    --arg grammar_name "$grammar_name" \
    --arg effective_mode "$parse_full_quality_effective" \
    --arg note "$parse_full_quality_note" \
    --arg parse_full_effective "$parse_full_effective" \
    --argjson enforced "$parse_full_quality_contract_enforced" \
    --argjson min_pass_ratio "$parse_full_quality_min_pass_ratio" \
    --argjson pass_count "$parse_full_pass_count" \
    --argjson fail_count "$parse_full_fail_count" \
    --argjson skip_count "$parse_full_skip_count" \
    --argjson samples_total "$parse_full_samples_total" \
    --argjson pass_ratio_percent "$parse_full_pass_ratio_percent" \
    '{
        grammar_name: $grammar_name,
        enforced: $enforced,
        effective_mode: $effective_mode,
        note: $note,
        min_pass_ratio: $min_pass_ratio,
        parse_full_effective: $parse_full_effective,
        observed: {
            pass_count: $pass_count,
            fail_count: $fail_count,
            skip_count: $skip_count,
            samples_total: $samples_total,
            pass_ratio_percent: $pass_ratio_percent
        }
    }' >"$parse_full_quality_report_json"

perf_report_json="$WORK_DIR/${grammar_name}_performance_report.json"
jq -n \
    --arg grammar_name "$grammar_name" \
    --arg requested_mode "$PERF_BUDGET_MODE" \
    --arg effective_mode "$perf_budget_effective" \
    --arg note "$perf_budget_note" \
    --argjson enabled "$perf_budget_enabled" \
    --argjson sample_count "$total_samples" \
    --argjson parse_full_samples "$perf_parse_full_samples" \
    --argjson max_generate_ms_per_sample "$perf_max_generate_ms_per_sample" \
    --argjson max_preprocess_ms_per_sample "$perf_max_preprocess_ms_per_sample" \
    --argjson max_parse_full_ms_per_sample "$perf_max_parse_full_ms_per_sample" \
    --argjson max_sample_bytes "$perf_max_sample_bytes" \
    --argjson max_preprocessed_bytes "$perf_max_preprocessed_bytes" \
    --argjson observed_generate_total_ms "$perf_generate_total_ms" \
    --argjson observed_generate_avg_ms "$perf_generate_avg_ms" \
    --argjson observed_generate_max_ms "$perf_generate_max_ms" \
    --argjson observed_preprocess_total_ms "$perf_preprocess_total_ms" \
    --argjson observed_preprocess_avg_ms "$perf_preprocess_avg_ms" \
    --argjson observed_preprocess_max_ms "$perf_preprocess_max_ms" \
    --argjson observed_parse_full_total_ms "$perf_parse_full_total_ms" \
    --argjson observed_parse_full_avg_ms "$perf_parse_full_avg_ms" \
    --argjson observed_parse_full_max_ms "$perf_parse_full_max_ms" \
    --argjson observed_sample_bytes_max "$perf_sample_bytes_max" \
    --argjson observed_preprocessed_bytes_max "$perf_preprocessed_bytes_max" \
    --argjson realistic_enabled "$realistic_corpus_enabled" \
    --arg realistic_effective_mode "$realistic_corpus_effective" \
    --arg realistic_note "$realistic_corpus_note" \
    --arg realistic_corpus_path "${realistic_corpus_path:-}" \
    --argjson realistic_cases_declared "$realistic_cases_declared" \
    --argjson realistic_cases_executed "$realistic_cases_executed" \
    --argjson realistic_expected_pass_total "$realistic_expected_pass_total" \
    --argjson realistic_expected_fail_total "$realistic_expected_fail_total" \
    --argjson realistic_observed_parse_pass_total "$realistic_parse_pass_total" \
    --argjson realistic_observed_parse_fail_total "$realistic_parse_fail_total" \
    --argjson realistic_expected_fail_parse_pass_total "$realistic_expected_fail_parse_pass_total" \
    --argjson realistic_preprocess_warning_total "$realistic_preprocess_warning_total" \
    --argjson realistic_preprocess_error_total "$realistic_preprocess_error_total" \
    --argjson realistic_preprocess_total_ms "$realistic_preprocess_total_ms" \
    --argjson realistic_preprocess_max_ms "$realistic_preprocess_max_ms" \
    --argjson realistic_parse_total_ms "$realistic_parse_total_ms" \
    --argjson realistic_parse_max_ms "$realistic_parse_max_ms" \
    --argjson realistic_sample_bytes_max "$realistic_sample_bytes_max" \
    --argjson realistic_preprocessed_bytes_max "$realistic_preprocessed_bytes_max" \
    --argjson realistic_max_preprocess_ms_per_case "$realistic_max_preprocess_ms_per_case" \
    --argjson realistic_max_parse_full_ms_per_case "$realistic_max_parse_full_ms_per_case" \
    --argjson realistic_max_sample_bytes "$realistic_max_sample_bytes" \
    --argjson realistic_max_preprocessed_bytes "$realistic_max_preprocessed_bytes" \
    --argjson realistic_require_no_preprocess_errors "$realistic_require_no_preprocess_errors" \
    --arg realistic_report_json "$realistic_report_json" \
    '{
        grammar_name: $grammar_name,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        note: $note,
        enabled: $enabled,
        thresholds: {
            max_generate_ms_per_sample: $max_generate_ms_per_sample,
            max_preprocess_ms_per_sample: $max_preprocess_ms_per_sample,
            max_parse_full_ms_per_sample: $max_parse_full_ms_per_sample,
            max_sample_bytes: $max_sample_bytes,
            max_preprocessed_bytes: $max_preprocessed_bytes
        },
        observed: {
            sample_count: $sample_count,
            parse_full_samples: $parse_full_samples,
            generate_total_ms: $observed_generate_total_ms,
            generate_avg_ms: $observed_generate_avg_ms,
            generate_max_ms: $observed_generate_max_ms,
            preprocess_total_ms: $observed_preprocess_total_ms,
            preprocess_avg_ms: $observed_preprocess_avg_ms,
            preprocess_max_ms: $observed_preprocess_max_ms,
            parse_full_total_ms: $observed_parse_full_total_ms,
            parse_full_avg_ms: $observed_parse_full_avg_ms,
            parse_full_max_ms: $observed_parse_full_max_ms,
            sample_bytes_max: $observed_sample_bytes_max,
            preprocessed_bytes_max: $observed_preprocessed_bytes_max
        },
        realistic_corpus: {
            enabled: ($realistic_enabled == 1),
            effective_mode: $realistic_effective_mode,
            note: $realistic_note,
            corpus_path: $realistic_corpus_path,
            report_json: $realistic_report_json,
            thresholds: {
                max_preprocess_ms_per_case: $realistic_max_preprocess_ms_per_case,
                max_parse_full_ms_per_case: $realistic_max_parse_full_ms_per_case,
                max_sample_bytes: $realistic_max_sample_bytes,
                max_preprocessed_bytes: $realistic_max_preprocessed_bytes,
                require_no_preprocess_errors: ($realistic_require_no_preprocess_errors == 1)
            },
            observed: {
                cases_declared: $realistic_cases_declared,
                cases_executed: $realistic_cases_executed,
                expected_pass_total: $realistic_expected_pass_total,
                expected_fail_total: $realistic_expected_fail_total,
                observed_parse_pass_total: $realistic_observed_parse_pass_total,
                observed_parse_fail_total: $realistic_observed_parse_fail_total,
                expected_fail_parse_pass_total: $realistic_expected_fail_parse_pass_total,
                preprocess_warning_total: $realistic_preprocess_warning_total,
                preprocess_error_total: $realistic_preprocess_error_total,
                preprocess_total_ms: $realistic_preprocess_total_ms,
                preprocess_max_ms: $realistic_preprocess_max_ms,
                parse_total_ms: $realistic_parse_total_ms,
                parse_max_ms: $realistic_parse_max_ms,
                sample_bytes_max: $realistic_sample_bytes_max,
                preprocessed_bytes_max: $realistic_preprocessed_bytes_max
            }
        }
    }' >"$perf_report_json"

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
    echo "closed_loop_target_max_attempts_source: $target_max_attempts_source"
    echo "closed_loop_replay_sample_count: $replay_sample_count"
    echo "closed_loop_profiles_passed: $closed_loop_profile_pass_count/$profile_count"
    echo "closed_loop_profiles_skipped: $closed_loop_profile_skip_count/$profile_count"
    echo "closed_loop_initial_replay_determinism_passes: $closed_loop_initial_replay_determinism_pass_count/$profile_count"
    echo "closed_loop_initial_targets_total: $closed_loop_initial_targets_total"
    echo "closed_loop_replay_targets_total: $closed_loop_replay_targets_total"
    echo "closed_loop_initial_preprocess_warnings_total: $closed_loop_initial_preprocess_warnings_total"
    echo "closed_loop_initial_preprocess_errors_total: $closed_loop_initial_preprocess_errors_total"
    echo "closed_loop_replay_preprocess_warnings_total: $closed_loop_replay_preprocess_warnings_total"
    echo "closed_loop_replay_preprocess_errors_total: $closed_loop_replay_preprocess_errors_total"
    echo "closed_loop_parseability_shadow_enabled: $closed_loop_parseability_shadow_enabled"
    echo "closed_loop_parseability_shadow_effective: $closed_loop_parseability_shadow_effective"
    echo "closed_loop_parseability_shadow_note: $closed_loop_parseability_shadow_note"
    echo "closed_loop_parseability_shadow_requested_total: $closed_loop_parseability_shadow_requested_total"
    echo "closed_loop_parseability_shadow_attempts_total: $closed_loop_parseability_shadow_attempts_total"
    echo "closed_loop_parseability_shadow_accepted_total: $closed_loop_parseability_shadow_accepted_total"
    echo "closed_loop_parseability_shadow_rejected_total: $closed_loop_parseability_shadow_rejected_total"
    echo "closed_loop_parseability_shadow_parser_rejections_total: $closed_loop_parseability_shadow_parser_rejections_total"
    echo "closed_loop_parseability_shadow_generation_errors_total: $closed_loop_parseability_shadow_generation_errors_total"
    echo "closed_loop_parseability_shadow_empty_generations_total: $closed_loop_parseability_shadow_empty_generations_total"
    echo "closed_loop_parseability_shadow_acceptance_rate_percent: $closed_loop_parseability_shadow_acceptance_rate_percent"
    echo "closed_loop_parseability_shadow_counterexamples_captured_total: $closed_loop_parseability_shadow_counterexamples_captured_total"
    echo "closed_loop_parseability_shadow_primary_entry_attempts_total: $closed_loop_parseability_shadow_primary_entry_attempts_total"
    echo "closed_loop_parseability_shadow_primary_entry_accepted_outputs_total: $closed_loop_parseability_shadow_primary_entry_accepted_outputs_total"
    echo "closed_loop_parseability_shadow_primary_entry_rejected_outputs_total: $closed_loop_parseability_shadow_primary_entry_rejected_outputs_total"
    echo "closed_loop_parseability_shadow_alternate_entry_attempts_total: $closed_loop_parseability_shadow_alternate_entry_attempts_total"
    echo "closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total: $closed_loop_parseability_shadow_alternate_entry_accepted_outputs_total"
    echo "closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total: $closed_loop_parseability_shadow_alternate_entry_rejected_outputs_total"
    echo "closed_loop_parseability_shadow_report_json: $closed_loop_parseability_shadow_report_json"
    echo "declared_identifier_suite_status: $declared_identifier_suite_status"
    echo "declared_identifier_suite_total: $declared_identifier_suite_total"
    echo "declared_identifier_suite_passed: $declared_identifier_suite_passed"
    echo "declared_identifier_suite_failed: $declared_identifier_suite_failed"
    echo "width_compatibility_suite_status: $width_compat_suite_status"
    echo "width_compatibility_suite_total: $width_compat_suite_total"
    echo "width_compatibility_suite_passed: $width_compat_suite_passed"
    echo "width_compatibility_suite_failed: $width_compat_suite_failed"
    echo "port_binding_suite_status: $port_binding_suite_status"
    echo "port_binding_suite_total: $port_binding_suite_total"
    echo "port_binding_suite_passed: $port_binding_suite_passed"
    echo "port_binding_suite_failed: $port_binding_suite_failed"
    echo "package_qualification_suite_status: $package_qual_suite_status"
    echo "package_qualification_suite_total: $package_qual_suite_total"
    echo "package_qualification_suite_passed: $package_qual_suite_passed"
    echo "package_qualification_suite_failed: $package_qual_suite_failed"
    echo "context_legality_suite_status: $context_legality_suite_status"
    echo "context_legality_suite_total: $context_legality_suite_total"
    echo "context_legality_suite_passed: $context_legality_suite_passed"
    echo "context_legality_suite_failed: $context_legality_suite_failed"
    echo "parse_full_mode: $PARSE_FULL_MODE"
    echo "parse_full_effective: $parse_full_effective"
    echo "parseability_generation_enabled: $parseability_generation_enabled"
    echo "parseability_generation_note: $parseability_generation_note"
    echo "parseability_generation_requested_total: $parseability_generation_requested_total"
    echo "parseability_generation_accepted_total: $parseability_generation_accepted_total"
    echo "parseability_generation_rejected_total: $parseability_generation_rejected_total"
    echo "parseability_generation_attempts_total: $parseability_generation_attempts_total"
    echo "parseability_generation_parser_rejections_total: $parseability_generation_parser_rejections_total"
    echo "parseability_generation_generation_errors_total: $parseability_generation_errors_total"
    echo "parseability_generation_empty_generations_total: $parseability_generation_empty_generations_total"
    echo "parseability_generation_counterexamples_captured_total: $parseability_generation_counterexamples_captured_total"
    echo "parseability_generation_acceptance_rate_percent: $parseability_generation_acceptance_rate_percent"
    echo "parseability_generation_report_json: $parseability_generation_report_json"
    echo "parse_full_quality_enforced: $parse_full_quality_contract_enforced"
    echo "parse_full_quality_effective: $parse_full_quality_effective"
    echo "parse_full_quality_note: $parse_full_quality_note"
    echo "parse_full_quality_min_pass_ratio: $parse_full_quality_min_pass_ratio"
    echo "parse_full_pass_ratio_percent: $parse_full_pass_ratio_percent"
    echo "parse_full_quality_report_json: $parse_full_quality_report_json"
    echo "declared_shadow_mode: $DECLARED_SHADOW_MODE"
    echo "declared_shadow_parseable_only: $DECLARED_SHADOW_PARSEABLE_ONLY"
    echo "declared_shadow_effective: $declared_shadow_effective"
    echo "declared_shadow_note: $declared_shadow_note"
    echo "declared_shadow_checked: $declared_shadow_total"
    echo "declared_shadow_passed: $declared_shadow_passed"
    echo "declared_shadow_failed: $declared_shadow_failed"
    echo "declared_shadow_skipped_unparseable: $declared_shadow_skipped_unparseable"
    echo "declared_shadow_report_json: $declared_shadow_report_json"
    echo "perf_budget_mode: $PERF_BUDGET_MODE"
    echo "perf_budget_effective: $perf_budget_effective"
    echo "perf_budget_note: $perf_budget_note"
    echo "perf_threshold_generate_ms_per_sample: $perf_max_generate_ms_per_sample"
    echo "perf_threshold_preprocess_ms_per_sample: $perf_max_preprocess_ms_per_sample"
    echo "perf_threshold_parse_full_ms_per_sample: $perf_max_parse_full_ms_per_sample"
    echo "perf_threshold_sample_bytes: $perf_max_sample_bytes"
    echo "perf_threshold_preprocessed_bytes: $perf_max_preprocessed_bytes"
    echo "perf_observed_generate_total_ms: $perf_generate_total_ms"
    echo "perf_observed_generate_avg_ms: $perf_generate_avg_ms"
    echo "perf_observed_generate_max_ms: $perf_generate_max_ms"
    echo "perf_observed_preprocess_total_ms: $perf_preprocess_total_ms"
    echo "perf_observed_preprocess_avg_ms: $perf_preprocess_avg_ms"
    echo "perf_observed_preprocess_max_ms: $perf_preprocess_max_ms"
    echo "perf_observed_parse_full_samples: $perf_parse_full_samples"
    echo "perf_observed_parse_full_total_ms: $perf_parse_full_total_ms"
    echo "perf_observed_parse_full_avg_ms: $perf_parse_full_avg_ms"
    echo "perf_observed_parse_full_max_ms: $perf_parse_full_max_ms"
    echo "perf_observed_sample_bytes_max: $perf_sample_bytes_max"
    echo "perf_observed_preprocessed_bytes_max: $perf_preprocessed_bytes_max"
    echo "perf_report_json: $perf_report_json"
    echo "realistic_corpus_effective: $realistic_corpus_effective"
    echo "realistic_corpus_note: $realistic_corpus_note"
    echo "realistic_corpus_path: ${realistic_corpus_path:-}"
    echo "realistic_corpus_cases_declared: $realistic_cases_declared"
    echo "realistic_corpus_cases_executed: $realistic_cases_executed"
    echo "realistic_corpus_expected_pass_total: $realistic_expected_pass_total"
    echo "realistic_corpus_expected_fail_total: $realistic_expected_fail_total"
    echo "realistic_corpus_observed_parse_pass_total: $realistic_parse_pass_total"
    echo "realistic_corpus_observed_parse_fail_total: $realistic_parse_fail_total"
    echo "realistic_corpus_expected_fail_parse_pass_total: $realistic_expected_fail_parse_pass_total"
    echo "realistic_corpus_preprocess_warning_total: $realistic_preprocess_warning_total"
    echo "realistic_corpus_preprocess_error_total: $realistic_preprocess_error_total"
    echo "realistic_corpus_preprocess_total_ms: $realistic_preprocess_total_ms"
    echo "realistic_corpus_preprocess_max_ms: $realistic_preprocess_max_ms"
    echo "realistic_corpus_parse_total_ms: $realistic_parse_total_ms"
    echo "realistic_corpus_parse_max_ms: $realistic_parse_max_ms"
    echo "realistic_corpus_sample_bytes_max: $realistic_sample_bytes_max"
    echo "realistic_corpus_preprocessed_bytes_max: $realistic_preprocessed_bytes_max"
    echo "realistic_corpus_report_json: $realistic_report_json"
    echo "semantic_baseline_passes: $semantic_pass_count/$total_samples"
    echo "parse_full_passes: $parse_full_pass_count/$total_samples"
    echo "parse_full_failures: $parse_full_fail_count"
    echo "parse_full_skips: $parse_full_skip_count"
    echo "diff_mode: $DIFF_MODE"
    echo "diff_effective_mode: $diff_effective_mode"
    echo "diff_note: $diff_note"
    echo "diff_reference_runner: $diff_reference_runner"
    echo "diff_samples_checked: $diff_samples_checked/$diff_total_samples_seen (max=$DIFF_MAX_SAMPLES)"
    echo "diff_mismatch_count: $diff_mismatch_count"
    echo "diff_taxonomy_match: $diff_match_count"
    echo "diff_taxonomy_rust_failed_reference_passed: $diff_rust_failed_reference_passed_count"
    echo "diff_taxonomy_reference_failed_rust_passed: $diff_reference_failed_rust_passed_count"
    echo "diff_taxonomy_both_failed: $diff_both_failed_count"
    echo "diff_taxonomy_reference_artifact_missing: $diff_reference_artifact_missing_count"
    echo "diff_report_json: $diff_report_json"
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
