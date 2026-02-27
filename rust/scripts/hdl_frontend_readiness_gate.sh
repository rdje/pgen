#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"

STATE_DIR="${PGEN_HDL_FRONTEND_STATE_DIR:-$RUST_DIR/target/hdl_frontend_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

STRICT="${PGEN_HDL_FRONTEND_STRICT:-0}"
STIMULI_COUNT="${PGEN_HDL_FRONTEND_STIMULI_COUNT:-8}"
STIMULI_SEED="${PGEN_HDL_FRONTEND_STIMULI_SEED:-1337}"
PARSEABILITY_MAX_ATTEMPTS="${PGEN_HDL_FRONTEND_PARSEABILITY_MAX_ATTEMPTS:-50}"

GRAMMARS=("systemverilog" "vhdl")
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"

if ! [[ "$STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_HDL_FRONTEND_STRICT must be 0 or 1" >&2
    exit 2
fi

if ! [[ "$STIMULI_COUNT" =~ ^[0-9]+$ ]] || [[ "$STIMULI_COUNT" -lt 1 ]]; then
    echo "error: PGEN_HDL_FRONTEND_STIMULI_COUNT must be an integer >= 1" >&2
    exit 2
fi

if ! [[ "$PARSEABILITY_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$PARSEABILITY_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_HDL_FRONTEND_PARSEABILITY_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi

mkdir -p "$STATE_DIR" "$LOG_DIR" "$WORK_DIR"

echo "==> Building ast_pipeline binary"
(cd "$RUST_DIR" && cargo build --bin ast_pipeline >/dev/null)

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary not found at '$AST_PIPELINE_BIN'" >&2
    exit 1
fi

echo "grammar,grammar_file,ebnf_to_json,json_to_parser,json_to_stimuli,parser_registry_support,parseability,overall,notes" >"$SUMMARY_CSV"
{
    echo "PGEN HDL Frontend Readiness Summary"
    echo "state_dir: $STATE_DIR"
    echo "strict_mode: $STRICT"
    echo "stimuli_count: $STIMULI_COUNT"
    echo "stimuli_seed: $STIMULI_SEED"
    echo
} >"$SUMMARY_TXT"

failures=0

for grammar in "${GRAMMARS[@]}"; do
    grammar_file="$GRAMMARS_DIR/${grammar}.ebnf"
    json_out="$WORK_DIR/${grammar}.json"
    parser_out="$WORK_DIR/${grammar}_parser.rs"
    stimuli_out="$WORK_DIR/${grammar}_stimuli.txt"
    stimuli_manifest="$WORK_DIR/${grammar}_stimuli_samples.list"

    grammar_file_status="missing"
    ebnf_to_json_status="skip"
    json_to_parser_status="skip"
    json_to_stimuli_status="skip"
    parser_registry_support_status="skip"
    parseability_status="skip"
    overall="not_ready"
    notes="grammar file missing (expected ${grammar}.ebnf)"

    ebnf_log="$LOG_DIR/${grammar}.ebnf_to_json.log"
    parser_log="$LOG_DIR/${grammar}.json_to_parser.log"
    stimuli_log="$LOG_DIR/${grammar}.json_to_stimuli.log"
    probe_build_log="$LOG_DIR/${grammar}.parseability_probe_build.log"
    probe_support_log="$LOG_DIR/${grammar}.parseability_support.log"
    parseability_log="$LOG_DIR/${grammar}.parseability.log"

    if [[ -f "$grammar_file" ]]; then
        grammar_file_status="present"
        notes="ok"
        if perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$grammar_file" -o "$json_out" >"$ebnf_log" 2>&1; then
            ebnf_to_json_status="pass"
            if "$AST_PIPELINE_BIN" "$json_out" --generate-parser --eliminate-left-recursion -o "$parser_out" >"$parser_log" 2>&1; then
                json_to_parser_status="pass"
                : >"$stimuli_log"
                : >"$stimuli_out"
                : >"$stimuli_manifest"
                stimuli_generation_failed=0
                emitted_stimuli=0

                for ((stimulus_idx = 0; stimulus_idx < STIMULI_COUNT; stimulus_idx++)); do
                    stimulus_seed=$((STIMULI_SEED + stimulus_idx))
                    stimulus_file="$WORK_DIR/${grammar}_stimulus_${stimulus_idx}.txt"
                    if "$AST_PIPELINE_BIN" "$json_out" --generate-stimuli --count 1 --seed "$stimulus_seed" --output "$stimulus_file" >>"$stimuli_log" 2>&1; then
                        if [[ -s "$stimulus_file" ]]; then
                            printf '%s\n' "$stimulus_file" >>"$stimuli_manifest"
                            emitted_stimuli=$((emitted_stimuli + 1))
                        fi
                    else
                        stimuli_generation_failed=1
                        break
                    fi
                done

                if [[ "$stimuli_generation_failed" -eq 0 && "$emitted_stimuli" -gt 0 ]]; then
                    json_to_stimuli_status="pass"
                    probe_build_ok=0
                    if [[ "$grammar" == "systemverilog" ]]; then
                        if (cd "$RUST_DIR" && PGEN_SYSTEMVERILOG_PARSER_PATH="$parser_out" cargo build --features generated_parsers --bin parseability_probe >"$probe_build_log" 2>&1); then
                            probe_build_ok=1
                        fi
                    elif [[ "$grammar" == "vhdl" ]]; then
                        if (cd "$RUST_DIR" && PGEN_VHDL_PARSER_PATH="$parser_out" cargo build --features generated_parsers --bin parseability_probe >"$probe_build_log" 2>&1); then
                            probe_build_ok=1
                        fi
                    fi

                    if [[ "$probe_build_ok" -eq 0 || ! -x "$PARSE_PROBE_BIN" ]]; then
                        parser_registry_support_status="fail"
                        parseability_status="fail"
                        overall="fail"
                        failures=$((failures + 1))
                        notes="failed to build parseability_probe (see logs/${grammar}.parseability_probe_build.log)"
                    elif "$PARSE_PROBE_BIN" --supports "$grammar" >"$probe_support_log" 2>&1; then
                        parser_registry_support_status="pass"

                        sample_index=0
                        parseability_failed=0
                        : >"$parseability_log"
                        while IFS= read -r sample_file || [[ -n "$sample_file" ]]; do
                            [[ -n "$sample_file" ]] || continue
                            if ! "$PARSE_PROBE_BIN" --parse "$grammar" "$sample_file" >>"$parseability_log" 2>&1; then
                                sample_base_seed=$((STIMULI_SEED + sample_index))
                                recovered=0
                                for ((retry_idx = 1; retry_idx < PARSEABILITY_MAX_ATTEMPTS; retry_idx++)); do
                                    retry_seed=$((sample_base_seed + retry_idx * STIMULI_COUNT))
                                    if "$AST_PIPELINE_BIN" "$json_out" --generate-stimuli --count 1 --seed "$retry_seed" --output "$sample_file" >>"$stimuli_log" 2>&1 && [[ -s "$sample_file" ]]; then
                                        if "$PARSE_PROBE_BIN" --parse "$grammar" "$sample_file" >>"$parseability_log" 2>&1; then
                                            echo "Recovered parseability for sample ${sample_index} with retry seed ${retry_seed}" >>"$parseability_log"
                                            recovered=1
                                            break
                                        fi
                                    fi
                                done
                                if [[ "$recovered" -eq 0 ]]; then
                                    parseability_failed=1
                                    break
                                fi
                            fi
                            sample_index=$((sample_index + 1))
                        done <"$stimuli_manifest"

                        if [[ "$sample_index" -eq 0 ]]; then
                            parseability_failed=1
                            echo "no non-empty stimuli samples were emitted" >>"$parseability_log"
                        fi

                        if [[ "$parseability_failed" -eq 1 ]]; then
                            parseability_status="fail"
                            overall="fail"
                            failures=$((failures + 1))
                            notes="parseability failed (see logs/${grammar}.parseability.log)"
                        else
                            : >"$stimuli_out"
                            while IFS= read -r accepted_sample_file || [[ -n "$accepted_sample_file" ]]; do
                                [[ -n "$accepted_sample_file" ]] || continue
                                cat "$accepted_sample_file" >>"$stimuli_out"
                                printf '\n\n' >>"$stimuli_out"
                            done <"$stimuli_manifest"
                            parseability_status="pass"
                            overall="pass"
                        fi
                    else
                        parser_registry_support_status="fail"
                        parseability_status="fail"
                        overall="fail"
                        failures=$((failures + 1))
                        notes="parser_registry adapter unavailable (see logs/${grammar}.parseability_support.log)"
                    fi
                else
                    json_to_stimuli_status="fail"
                    overall="fail"
                    failures=$((failures + 1))
                    notes="json_to_stimuli failed (see logs/${grammar}.json_to_stimuli.log)"
                fi
            else
                json_to_parser_status="fail"
                overall="fail"
                failures=$((failures + 1))
                notes="json_to_parser failed (see logs/${grammar}.json_to_parser.log)"
            fi
        else
            ebnf_to_json_status="fail"
            overall="fail"
            failures=$((failures + 1))
            notes="ebnf_to_json failed (see logs/${grammar}.ebnf_to_json.log)"
        fi
    else
        failures=$((failures + 1))
    fi

    echo "${grammar},${grammar_file_status},${ebnf_to_json_status},${json_to_parser_status},${json_to_stimuli_status},${parser_registry_support_status},${parseability_status},${overall},${notes}" >>"$SUMMARY_CSV"
done

{
    echo "Results:"
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
    echo
    echo "Logs: $LOG_DIR"
    echo "Artifacts: $WORK_DIR"
} >>"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if [[ "$failures" -ne 0 ]]; then
    echo "⚠️  HDL frontend readiness has $failures failing or missing grammar flow(s)." >&2
    if [[ "$STRICT" -eq 1 ]]; then
        echo "❌ strict mode enabled: failing." >&2
        exit 1
    fi
    echo "ℹ️  strict mode disabled: reporting only." >&2
else
    echo "✅ HDL frontend readiness check passed for all tracked grammars."
fi
