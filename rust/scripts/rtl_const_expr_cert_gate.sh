#!/usr/bin/env bash
set -euo pipefail

# rtl_const_expr_cert_gate.sh
#
# CERT-GEN-BUDGET.4 — the rtl_const_expr canonical certificate-coverage baseline as a
# re-runnable, deterministic, regression-locked oracle.
#
# The fully-certified-6 roster membership for rtl_const_expr was doc-asserted only; the
# CERT-GEN-BUDGET.3 adjudication showed exactly how that survives wording confusion (a
# sub-entry probe config mislabeled "canonical"). This gate pins the CANONICAL lane
# mechanically: the DEFAULT entry rule (no --entry-rule override), --max-depth 32,
# --count 40, the DEFAULT diverse generation step-budget, for each declared seed. It
# parses the `CERTIFICATE-COVERAGE:` headline and asserts every field equals the tracked
# contract (total/proof/witness/UNKNOWN/fully_certified, sample_parse_failures=0,
# proof_reverify_failures=0, grammar/entry identity) and that all seeds agree
# (determinism). Emits summary.txt + summary.json; exits nonzero on any drift.
#
# Behavior-affecting PGEN_* generation knobs are explicitly unset for the cert runs so
# the pinned baseline is the true default posture, never an inherited environment.
# Cheap (~30 s per seed) — suitable for CI and local `make`.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_RTL_CONST_EXPR_CERT_STATE_DIR:-$RUST_DIR/target/rtl_const_expr_cert_gate}"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_RTL_CONST_EXPR_CERT_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/rtl_const_expr_cert_contract.json}"
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"

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

# Stage logs (e.g. the focus_rtl_const_expr regen, ~40 MB per run) are scratch, not
# evidence: keep only a bounded tail. Per-seed cert logs are NOT routed through this
# helper and stay complete — they are the parsed evidence artifacts.
PRUNE_THRESHOLD_BYTES=10485760

prune_log() {
    local log_file="$1"
    if [[ -f "$log_file" ]] && [[ "$(wc -c <"$log_file")" -gt "$PRUNE_THRESHOLD_BYTES" ]]; then
        local tmp="${log_file}.tail"
        {
            echo "[pruned by rtl_const_expr_cert_gate: original log exceeded ${PRUNE_THRESHOLD_BYTES} bytes; last 2000 lines retained]"
            tail -n 2000 "$log_file"
        } >"$tmp" && mv "$tmp" "$log_file"
    fi
}

run_logged_rust() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (cd "$RUST_DIR" && "$@") >"$log_file" 2>&1; then
        prune_log "$log_file"
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        prune_log "$log_file"
        exit 1
    fi
}

# Extract `key=<int>` from a single cert headline line.
field_int() {
    local line="$1"
    local key="$2"
    local v
    v="$(printf '%s\n' "$line" | grep -oE "${key}=[0-9]+" | head -n 1 | cut -d= -f2 || true)"
    if [[ -z "$v" ]]; then
        echo "error: could not parse '${key}=' from line: $line" >&2
        exit 1
    fi
    printf '%s\n' "$v"
}

# Extract `key=<true|false>` from a single cert headline line.
field_bool() {
    local line="$1"
    local key="$2"
    local v
    v="$(printf '%s\n' "$line" | grep -oE "${key}=(true|false)" | head -n 1 | cut -d= -f2 || true)"
    if [[ -z "$v" ]]; then
        echo "error: could not parse '${key}=' from line: $line" >&2
        exit 1
    fi
    printf '%s\n' "$v"
}

# Extract `key='<value>'` from a single cert headline line.
field_quoted() {
    local line="$1"
    local key="$2"
    local v
    v="$(printf '%s\n' "$line" | grep -oE "${key}='[^']*'" | head -n 1 | sed -E "s/^${key}='(.*)'\$/\\1/" || true)"
    if [[ -z "$v" ]]; then
        echo "error: could not parse '${key}=' from line: $line" >&2
        exit 1
    fi
    printf '%s\n' "$v"
}

require_tool jq
require_file "$CONTRACT_FILE"

# --- contract schema validation -------------------------------------------------------
jq -e '
    .family == "rtl_const_expr"
    and (.gate == "rtl_const_expr_cert_gate")
    and ((.version | type) == "number")
    and ((.grammar_input | type) == "string" and (.grammar_input | length) > 0)
    and (.entry_rule_arg == "default")
    and ((.expected_grammar | type) == "string" and (.expected_grammar | length) > 0)
    and ((.expected_entry | type) == "string" and (.expected_entry | length) > 0)
    and ((.max_depth | type) == "number")
    and ((.samples | type) == "number")
    and ((.seeds | type) == "array" and (.seeds | length) > 0)
    and (.diverse_generation_budget == "default")
    and ((.expected_total | type) == "number")
    and ((.expected_proof | type) == "number")
    and ((.expected_witness | type) == "number")
    and ((.expected_unknown | type) == "number")
    and ((.expected_fully_certified | type) == "boolean")
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
' "$CONTRACT_FILE" >/dev/null || {
    echo "error: contract schema invalid: $CONTRACT_FILE" >&2
    exit 1
}

grammar_input="$(jq -r '.grammar_input' "$CONTRACT_FILE")"
expected_grammar="$(jq -r '.expected_grammar' "$CONTRACT_FILE")"
expected_entry="$(jq -r '.expected_entry' "$CONTRACT_FILE")"
max_depth="$(jq -r '.max_depth' "$CONTRACT_FILE")"
samples="$(jq -r '.samples' "$CONTRACT_FILE")"
expected_total="$(jq -r '.expected_total' "$CONTRACT_FILE")"
expected_proof="$(jq -r '.expected_proof' "$CONTRACT_FILE")"
expected_witness="$(jq -r '.expected_witness' "$CONTRACT_FILE")"
expected_unknown="$(jq -r '.expected_unknown' "$CONTRACT_FILE")"
expected_fully_certified="$(jq -r '.expected_fully_certified' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"

mapfile -t seeds < <(jq -r '.seeds[]' "$CONTRACT_FILE")

GRAMMAR_INPUT_PATH="$ROOT_DIR/$grammar_input"

mkdir -p "$LOG_DIR"
: >"$SUMMARY_TXT"

# --- ensure the generated rtl_const_expr artifacts, then build the DEBUG ast_pipeline ---
# build.rs sets has_generated_rtl_const_expr_parser when generated/rtl_const_expr_parser.rs
# exists; cert-coverage verifies witnesses through that real generated parser.
run_logged_rust "ensure_generated_rtl_const_expr_artifacts" \
    make -C "$RUST_DIR" SHELL=/bin/bash focus_rtl_const_expr
run_logged_rust "build_debug_ast_pipeline" \
    cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi
require_file "$GRAMMAR_INPUT_PATH"

# --- per-seed canonical cert run + parse -----------------------------------------------
declare -a unmet=()
declare -a per_seed_json=()

first_seed_signature=""
overall_total=""
overall_proof=""
overall_witness=""
overall_unknown=""
overall_fully_certified=""

for seed in "${seeds[@]}"; do
    cert_log="$LOG_DIR/cert_seed_${seed}.log"
    echo "==> canonical_cert seed=${seed}"
    # The contract pins the DEFAULT generation posture: strip every behavior-affecting
    # PGEN_* generation knob so an inherited environment cannot skew the baseline.
    if ! env -u PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS \
             -u PGEN_WITNESS_NO_PURDOM \
             -u PGEN_WITNESS_TIMEOUT_FLOOR_MS \
             -u PGEN_GENERATION_STEPS_PER_MS \
        "$AST_PIPELINE_BIN" "$GRAMMAR_INPUT_PATH" --report-certificate-coverage \
        --count "$samples" --max-depth "$max_depth" --seed "$seed" >"$cert_log" 2>&1; then
        echo "error: cert-coverage run failed for seed ${seed} (log: $cert_log)" >&2
        tail -n 80 "$cert_log" >&2 || true
        exit 1
    fi

    cert_line="$(grep -E '^CERTIFICATE-COVERAGE: ' "$cert_log" | head -n 1 || true)"
    if [[ -z "$cert_line" ]]; then
        echo "error: missing CERTIFICATE-COVERAGE headline for seed ${seed} (log: $cert_log)" >&2
        exit 1
    fi

    grammar_name="$(field_quoted "$cert_line" grammar)"
    entry_name="$(field_quoted "$cert_line" entry)"
    total="$(field_int "$cert_line" total)"
    proof="$(field_int "$cert_line" proof)"
    witness="$(field_int "$cert_line" witness)"
    unknown="$(field_int "$cert_line" UNKNOWN)"
    fully_certified="$(field_bool "$cert_line" fully_certified)"
    spf="$(field_int "$cert_line" sample_parse_failures)"
    prf="$(field_int "$cert_line" proof_reverify_failures)"

    # per-seed contract assertions
    [[ "$grammar_name" == "$expected_grammar" ]] || unmet+=("seed=${seed} grammar='${grammar_name}' (expected '${expected_grammar}')")
    [[ "$entry_name" == "$expected_entry" ]] || unmet+=("seed=${seed} entry='${entry_name}' (expected default entry '${expected_entry}')")
    [[ "$total" == "$expected_total" ]] || unmet+=("seed=${seed} total=${total} (expected ${expected_total})")
    [[ "$proof" == "$expected_proof" ]] || unmet+=("seed=${seed} proof=${proof} (expected ${expected_proof})")
    [[ "$witness" == "$expected_witness" ]] || unmet+=("seed=${seed} witness=${witness} (expected ${expected_witness})")
    [[ "$unknown" == "$expected_unknown" ]] || unmet+=("seed=${seed} UNKNOWN=${unknown} (expected ${expected_unknown})")
    [[ "$fully_certified" == "$expected_fully_certified" ]] || unmet+=("seed=${seed} fully_certified=${fully_certified} (expected ${expected_fully_certified})")
    [[ "$spf" == "0" ]] || unmet+=("seed=${seed} sample_parse_failures=${spf} (expected 0)")
    [[ "$prf" == "0" ]] || unmet+=("seed=${seed} proof_reverify_failures=${prf} (expected 0)")

    # determinism: every seed must agree with the first seed's full signature
    signature="${grammar_name}|${entry_name}|${total}|${proof}|${witness}|${unknown}|${fully_certified}|${spf}|${prf}"
    if [[ -z "$first_seed_signature" ]]; then
        first_seed_signature="$signature"
        overall_total="$total"
        overall_proof="$proof"
        overall_witness="$witness"
        overall_unknown="$unknown"
        overall_fully_certified="$fully_certified"
    elif [[ "$signature" != "$first_seed_signature" ]]; then
        unmet+=("seed=${seed} signature drift vs seed ${seeds[0]}: '${signature}' != '${first_seed_signature}'")
    fi

    per_seed_json+=("$(jq -nc \
        --argjson seed "$seed" \
        --arg grammar "$grammar_name" \
        --arg entry "$entry_name" \
        --argjson total "$total" \
        --argjson proof "$proof" \
        --argjson witness "$witness" \
        --argjson unknown "$unknown" \
        --argjson fully_certified "$fully_certified" \
        --argjson sample_parse_failures "$spf" \
        --argjson proof_reverify_failures "$prf" \
        '{seed: $seed, grammar: $grammar, entry: $entry, total: $total, proof: $proof, witness: $witness, unknown: $unknown, fully_certified: $fully_certified, sample_parse_failures: $sample_parse_failures, proof_reverify_failures: $proof_reverify_failures}')")
done

cert_baseline_green=false
unmet_count="${#unmet[@]}"
if [[ "$unmet_count" -eq 0 ]]; then
    cert_baseline_green=true
fi

primary_unmet="<none>"
if [[ "$unmet_count" -gt 0 ]]; then
    primary_unmet="${unmet[0]}"
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
unmet_json="$(printf '%s\n' "${unmet[@]:-}" | jq -R . | jq -sc 'map(select(length > 0))')"
per_seed_array_json="$(printf '%s\n' "${per_seed_json[@]}" | jq -sc 'map(select(type == "object"))')"
seeds_json="$(jq -c '.seeds' "$CONTRACT_FILE")"

{
    echo "rtl_const_expr Canonical Certificate-Coverage Gate Summary"
    echo "gate: rtl_const_expr_cert_gate"
    echo "version: 1"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "contract_file: $CONTRACT_FILE"
    echo "status_rule_done: $done_rule"
    echo "grammar_input: $grammar_input"
    echo "entry_rule_arg: default"
    echo "max_depth: $max_depth"
    echo "samples: $samples"
    echo "seeds: $seeds_json"
    echo "diverse_generation_budget: default"
    echo "cert_baseline_green: $cert_baseline_green"
    echo "total: $overall_total"
    echo "proof: $overall_proof"
    echo "witness: $overall_witness"
    echo "unknown: $overall_unknown"
    echo "fully_certified: $overall_fully_certified"
    echo "expected_total: $expected_total"
    echo "expected_proof: $expected_proof"
    echo "expected_witness: $expected_witness"
    echo "expected_unknown: $expected_unknown"
    echo "expected_fully_certified: $expected_fully_certified"
    echo "unmet_criteria_count: $unmet_count"
    echo "primary_unmet_criterion: $primary_unmet"
    echo "unmet_criteria_json: $unmet_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "rtl_const_expr_cert_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg contract_file "$CONTRACT_FILE" \
    --arg status_rule_done "$done_rule" \
    --arg grammar_input "$grammar_input" \
    --arg entry_rule_arg "default" \
    --argjson max_depth "$max_depth" \
    --argjson samples "$samples" \
    --argjson seeds "$seeds_json" \
    --arg diverse_generation_budget "default" \
    --argjson cert_baseline_green "$cert_baseline_green" \
    --argjson total "${overall_total:-null}" \
    --argjson proof "${overall_proof:-null}" \
    --argjson witness "${overall_witness:-null}" \
    --argjson unknown "${overall_unknown:-null}" \
    --argjson fully_certified "${overall_fully_certified:-null}" \
    --argjson expected_total "$expected_total" \
    --argjson expected_proof "$expected_proof" \
    --argjson expected_witness "$expected_witness" \
    --argjson expected_unknown "$expected_unknown" \
    --argjson expected_fully_certified "$expected_fully_certified" \
    --argjson unmet_criteria_count "$unmet_count" \
    --arg primary_unmet_criterion "$primary_unmet" \
    --argjson unmet_criteria "$unmet_json" \
    --argjson per_seed "$per_seed_array_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      contract_file: $contract_file,
      status_rule_done: $status_rule_done,
      grammar_input: $grammar_input,
      entry_rule_arg: $entry_rule_arg,
      max_depth: $max_depth,
      samples: $samples,
      seeds: $seeds,
      diverse_generation_budget: $diverse_generation_budget,
      cert_baseline_green: $cert_baseline_green,
      observed: {
        total: $total,
        proof: $proof,
        witness: $witness,
        unknown: $unknown,
        fully_certified: $fully_certified
      },
      expected: {
        total: $expected_total,
        proof: $expected_proof,
        witness: $expected_witness,
        unknown: $expected_unknown,
        fully_certified: $expected_fully_certified
      },
      unmet_criteria_count: $unmet_criteria_count,
      primary_unmet_criterion: $primary_unmet_criterion,
      unmet_criteria: $unmet_criteria,
      per_seed: $per_seed
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

if [[ "$cert_baseline_green" != "true" ]]; then
    echo "❌ rtl_const_expr canonical certificate-coverage baseline NOT green: primary_unmet=${primary_unmet}" >&2
    echo "   (see $SUMMARY_TXT / $SUMMARY_JSON and per-seed logs under $LOG_DIR)" >&2
    exit 1
fi

echo "✅ rtl_const_expr canonical certificate-coverage gate passed (total=${overall_total} proof=${overall_proof} witness=${overall_witness} UNKNOWN=${overall_unknown} fully_certified=${overall_fully_certified}, deterministic across seeds ${seeds_json})."
