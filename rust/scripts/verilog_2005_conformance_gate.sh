#!/usr/bin/env bash
set -euo pipefail

# verilog_2005_conformance_gate.sh
#
# VERILOG-2005-PROFILE.4.3 — the repo-standard IEEE 1364-2005 conformance oracle for the
# strict `verilog_2005` dialect profile on `grammars/systemverilog.ebnf`, as a re-runnable,
# deterministic, regression-locked gate. Three stages, all asserted against the tracked
# contract (rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json):
#
#   1. WELLFORMEDNESS LINT LOCK — `ast_pipeline --lint-grammar` must exit 0 with exactly the
#      contract's `verilog_2005` profile-orphan count (0): the profile stays built-to-coherence
#      (the VERILOG-2005-PROFILE.3 standing sub-rule, mechanized).
#   2. CONFORMANCE CORPUS MATRIX — every curated corpus file is parsed under EVERY profile in
#      its per-file expectation map (verilog_2005 / sv_2017 / sv_2023) via the release
#      `parseability_probe`; accept/reject must match the contract exactly (strictness proven
#      both ways: the Verilog-2005 surface ACCEPTS, the SV-only surface REJECTS while still
#      ACCEPTING under the SV profiles). Profile-alias normalization is spot-locked too.
#   3. PROFILED CERT BASELINE — `--report-certificate-coverage --grammar-profile verilog_2005`
#      per contract seed; every headline field must equal the pinned baseline and all seeds
#      must agree (determinism). The pins are a BASELINE, not a closure claim: the gated
#      SV-only surface is profile-unreachable by design (see the contract's baseline_note).
#
# Parser-agnostic by construction: grammar file, profiles, corpus, expectations, and cert pins
# all come from the contract. Heavy (~3 cert runs), so this is a CI/`make` oracle, not a
# `cargo test --lib` unit.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VERILOG_2005_CONFORMANCE_STATE_DIR:-$RUST_DIR/target/verilog_2005_conformance_gate}"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_VERILOG_2005_CONFORMANCE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json}"
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PROBE_BIN="$RUST_DIR/target/release/parseability_probe"

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

# Bulky stage logs are pruned to their last 2000 lines (the focus_systemverilog regen
# stage streams ~5 GB of codegen output per run; the failure-triage tail is what matters,
# and the repo disk-space doctrine forbids retaining multi-GB scratch logs).
PRUNE_THRESHOLD_BYTES=10485760

prune_log() {
    local log_file="$1"
    if [[ -f "$log_file" ]] && [[ "$(wc -c <"$log_file")" -gt "$PRUNE_THRESHOLD_BYTES" ]]; then
        local tmp="${log_file}.tail"
        {
            echo "[pruned by verilog_2005_conformance_gate: original log exceeded ${PRUNE_THRESHOLD_BYTES} bytes; last 2000 lines retained]"
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

# Extract `key=<int>` from a single headline line.
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

require_tool jq
require_file "$CONTRACT_FILE"

# --- contract schema validation -------------------------------------------------------
jq -e '
    .family == "systemverilog"
    and (.gate == "verilog_2005_conformance_gate")
    and ((.version | type) == "number")
    and ((.grammar_file | type) == "string" and (.grammar_file | length) > 0)
    and ((.corpus_dir | type) == "string" and (.corpus_dir | length) > 0)
    and ((.profiles | type) == "array" and (.profiles | length) > 0)
    and ((.alias_checks | type) == "array")
    and ((.lint.expected_exit_code | type) == "number")
    and ((.lint.orphan_profile | type) == "string")
    and ((.lint.expected_profile_orphans | type) == "number")
    and ((.cert.entry_rule | type) == "string")
    and ((.cert.grammar_profile | type) == "string")
    and ((.cert.samples | type) == "number")
    and ((.cert.seeds | type) == "array" and (.cert.seeds | length) > 0)
    and ((.cert.expected_total | type) == "number")
    and ((.cert.expected_proof | type) == "number")
    and ((.cert.expected_witness | type) == "number")
    and ((.cert.expected_unknown | type) == "number")
    and ((.cert.expected_sample_parse_failures | type) == "number")
    and ((.cert.expected_proof_reverify_failures | type) == "number")
    and ((.cases | type) == "array" and (.cases | length) > 0)
    and ([.cases[] | select((.file | type) != "string" or (.expect | type) != "object")] | length == 0)
' "$CONTRACT_FILE" >/dev/null || {
    echo "error: contract schema invalid: $CONTRACT_FILE" >&2
    exit 1
}

GRAMMAR_FILE="$ROOT_DIR/$(jq -r '.grammar_file' "$CONTRACT_FILE")"
CORPUS_DIR="$RUST_DIR/$(jq -r '.corpus_dir' "$CONTRACT_FILE")"
require_file "$GRAMMAR_FILE"
if [[ ! -d "$CORPUS_DIR" ]]; then
    echo "error: missing corpus directory '$CORPUS_DIR'" >&2
    exit 1
fi

lint_expected_rc="$(jq -r '.lint.expected_exit_code' "$CONTRACT_FILE")"
lint_orphan_profile="$(jq -r '.lint.orphan_profile' "$CONTRACT_FILE")"
lint_expected_orphans="$(jq -r '.lint.expected_profile_orphans' "$CONTRACT_FILE")"

cert_entry="$(jq -r '.cert.entry_rule' "$CONTRACT_FILE")"
cert_profile="$(jq -r '.cert.grammar_profile' "$CONTRACT_FILE")"
cert_samples="$(jq -r '.cert.samples' "$CONTRACT_FILE")"
cert_expected_total="$(jq -r '.cert.expected_total' "$CONTRACT_FILE")"
cert_expected_proof="$(jq -r '.cert.expected_proof' "$CONTRACT_FILE")"
cert_expected_witness="$(jq -r '.cert.expected_witness' "$CONTRACT_FILE")"
cert_expected_unknown="$(jq -r '.cert.expected_unknown' "$CONTRACT_FILE")"
cert_expected_spf="$(jq -r '.cert.expected_sample_parse_failures' "$CONTRACT_FILE")"
cert_expected_prf="$(jq -r '.cert.expected_proof_reverify_failures' "$CONTRACT_FILE")"

mapfile -t cert_seeds < <(jq -r '.cert.seeds[]' "$CONTRACT_FILE")

# VERILOG-2005-PROFILE.6.7: the DECLARED ENTRY UNIVERSE for the profiled cert's per-profile proof
# gathering (P1/P2). Without these the single-entry universe would falsely prove the entry-relative
# library cohort (`library_text`, …) dead — exactly what `.6.5` forbids. Absent ⇒ empty (byte-identical
# legacy behaviour).
mapfile -t cert_union_configs < <(jq -r '.cert.union_configs[]? // empty' "$CONTRACT_FILE")
declare -a cert_union_args=()
for cfg in "${cert_union_configs[@]:-}"; do
    [[ -n "$cfg" ]] && cert_union_args+=(--cert-union-config "$cfg")
done

mkdir -p "$LOG_DIR"
: >"$SUMMARY_TXT"

# --- ensure the generated SV parser, then build both binaries --------------------------
run_logged_rust "ensure_generated_systemverilog_parser" \
    make -C "$RUST_DIR" SHELL=/bin/bash focus_systemverilog
run_logged_rust "build_debug_ast_pipeline" \
    cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline
run_logged_rust "build_release_parseability_probe" \
    cargo build --release --features generated_parsers --bin parseability_probe

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi
if [[ ! -x "$PROBE_BIN" ]]; then
    echo "error: parseability_probe binary is missing at '$PROBE_BIN' after build" >&2
    exit 1
fi

declare -a unmet=()

# --- stage 1: wellformedness lint lock --------------------------------------------------
lint_log="$LOG_DIR/lint_grammar.log"
echo "==> lint_grammar (0 ${lint_orphan_profile} profile-orphan lock)"
lint_rc=0
"$AST_PIPELINE_BIN" "$GRAMMAR_FILE" --lint-grammar >"$lint_log" 2>&1 || lint_rc=$?

lint_headline="$(grep -E '^grammar lint: ' "$lint_log" | head -n 1 || true)"
if [[ -z "$lint_headline" ]]; then
    echo "error: missing 'grammar lint:' headline (log: $lint_log)" >&2
    tail -n 40 "$lint_log" >&2 || true
    exit 1
fi
lint_orphans_headline="$(field_int "$lint_headline" profile_orphans)"
lint_orphans_lines="$(grep -c "is present under profile '${lint_orphan_profile}'" "$lint_log" || true)"

[[ "$lint_rc" == "$lint_expected_rc" ]] || unmet+=("lint exit code=${lint_rc} (expected ${lint_expected_rc})")
[[ "$lint_orphans_headline" == "$lint_expected_orphans" ]] || unmet+=("lint headline profile_orphans=${lint_orphans_headline} (expected ${lint_expected_orphans})")
[[ "$lint_orphans_lines" == "$lint_expected_orphans" ]] || unmet+=("lint '${lint_orphan_profile}' orphan error lines=${lint_orphans_lines} (expected ${lint_expected_orphans})")
echo "    rc=${lint_rc} profile_orphans=${lint_orphans_headline} (${lint_orphan_profile} orphan lines=${lint_orphans_lines})"

# --- stage 2: conformance corpus matrix -------------------------------------------------
probe_verdict() {
    # accept | reject for one file under one profile string (alias-safe).
    local file="$1"
    local profile="$2"
    if "$PROBE_BIN" --parse systemverilog "$file" --profile "$profile" >/dev/null 2>&1; then
        printf 'accept\n'
    else
        printf 'reject\n'
    fi
}

matrix_checked=0
matrix_mismatched=0
matrix_log="$LOG_DIR/corpus_matrix.tsv"
: >"$matrix_log"
declare -a case_results_json=()

while IFS= read -r case_json; do
    rel_file="$(jq -r '.file' <<<"$case_json")"
    abs_file="$CORPUS_DIR/$rel_file"
    if [[ ! -f "$abs_file" ]]; then
        unmet+=("corpus case file missing: ${rel_file}")
        continue
    fi
    while IFS=$'\t' read -r prof expected; do
        observed="$(probe_verdict "$abs_file" "$prof")"
        matrix_checked=$((matrix_checked + 1))
        printf '%s\t%s\texpected=%s\tobserved=%s\n' "$rel_file" "$prof" "$expected" "$observed" >>"$matrix_log"
        if [[ "$observed" != "$expected" ]]; then
            matrix_mismatched=$((matrix_mismatched + 1))
            unmet+=("corpus ${rel_file} under ${prof}: observed=${observed} (expected ${expected})")
        fi
        case_results_json+=("$(jq -nc --arg file "$rel_file" --arg profile "$prof" \
            --arg expected "$expected" --arg observed "$observed" \
            '{file: $file, profile: $profile, expected: $expected, observed: $observed}')")
    done < <(jq -r '.expect | to_entries[] | [.key, .value] | @tsv' <<<"$case_json")
done < <(jq -c '.cases[]' "$CONTRACT_FILE")

echo "==> corpus matrix: ${matrix_checked} file×profile checks, ${matrix_mismatched} mismatches (${matrix_log})"

alias_checked=0
while IFS= read -r alias_json; do
    rel_file="$(jq -r '.file' <<<"$alias_json")"
    alias_name="$(jq -r '.profile_alias' <<<"$alias_json")"
    expected="$(jq -r '.expect' <<<"$alias_json")"
    abs_file="$CORPUS_DIR/$rel_file"
    if [[ ! -f "$abs_file" ]]; then
        unmet+=("alias-check file missing: ${rel_file}")
        continue
    fi
    observed="$(probe_verdict "$abs_file" "$alias_name")"
    alias_checked=$((alias_checked + 1))
    if [[ "$observed" != "$expected" ]]; then
        unmet+=("alias '${alias_name}' on ${rel_file}: observed=${observed} (expected ${expected})")
    fi
done < <(jq -c '.alias_checks[]' "$CONTRACT_FILE")
echo "==> alias normalization: ${alias_checked} checks"

# --- stage 3: profiled cert-coverage baseline (deterministic across seeds) --------------
first_seed_signature=""
overall_headline=""
declare -a per_seed_json=()

for seed in "${cert_seeds[@]}"; do
    cert_log="$LOG_DIR/cert_${cert_profile}_seed_${seed}.log"
    echo "==> cert baseline profile=${cert_profile} seed=${seed}"
    if ! "$AST_PIPELINE_BIN" "$GRAMMAR_FILE" --report-certificate-coverage \
        --grammar-profile "$cert_profile" --entry-rule "$cert_entry" \
        --count "$cert_samples" --seed "$seed" \
        ${cert_union_args[@]+"${cert_union_args[@]}"} >"$cert_log" 2>&1; then
        echo "error: cert-coverage run failed for seed ${seed} (log: $cert_log)" >&2
        tail -n 80 "$cert_log" >&2 || true
        exit 1
    fi

    headline="$(grep -E '^CERTIFICATE-COVERAGE: ' "$cert_log" | head -n 1 || true)"
    if [[ -z "$headline" ]]; then
        echo "error: missing cert headline for seed ${seed} (log: $cert_log)" >&2
        exit 1
    fi

    c_total="$(field_int "$headline" total)"
    c_proof="$(field_int "$headline" proof)"
    c_witness="$(field_int "$headline" witness)"
    c_unknown="$(field_int "$headline" UNKNOWN)"
    c_spf="$(field_int "$headline" sample_parse_failures)"
    c_prf="$(field_int "$headline" proof_reverify_failures)"

    [[ "$c_total" == "$cert_expected_total" ]] || unmet+=("seed=${seed} cert total=${c_total} (expected ${cert_expected_total})")
    [[ "$c_proof" == "$cert_expected_proof" ]] || unmet+=("seed=${seed} cert proof=${c_proof} (expected ${cert_expected_proof})")
    [[ "$c_witness" == "$cert_expected_witness" ]] || unmet+=("seed=${seed} cert witness=${c_witness} (expected ${cert_expected_witness})")
    [[ "$c_unknown" == "$cert_expected_unknown" ]] || unmet+=("seed=${seed} cert UNKNOWN=${c_unknown} (expected ${cert_expected_unknown})")
    [[ "$c_spf" == "$cert_expected_spf" ]] || unmet+=("seed=${seed} cert sample_parse_failures=${c_spf} (expected ${cert_expected_spf})")
    [[ "$c_prf" == "$cert_expected_prf" ]] || unmet+=("seed=${seed} cert proof_reverify_failures=${c_prf} (expected ${cert_expected_prf})")

    signature="${c_total}|${c_proof}|${c_witness}|${c_unknown}|${c_spf}|${c_prf}"
    if [[ -z "$first_seed_signature" ]]; then
        first_seed_signature="$signature"
        overall_headline="$headline"
    elif [[ "$signature" != "$first_seed_signature" ]]; then
        unmet+=("seed=${seed} cert signature drift vs seed ${cert_seeds[0]}: '${signature}' != '${first_seed_signature}'")
    fi

    per_seed_json+=("$(jq -nc --argjson seed "$seed" --argjson total "$c_total" \
        --argjson proof "$c_proof" --argjson witness "$c_witness" --argjson unknown "$c_unknown" \
        --argjson sample_parse_failures "$c_spf" --argjson proof_reverify_failures "$c_prf" \
        '{seed: $seed, total: $total, proof: $proof, witness: $witness, unknown: $unknown, sample_parse_failures: $sample_parse_failures, proof_reverify_failures: $proof_reverify_failures}')")
done

# --- verdict + summaries -----------------------------------------------------------------
gate_green=false
unmet_count="${#unmet[@]}"
if [[ "$unmet_count" -eq 0 ]]; then
    gate_green=true
fi
primary_unmet="<none>"
if [[ "$unmet_count" -gt 0 ]]; then
    primary_unmet="${unmet[0]}"
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
unmet_json="$(printf '%s\n' "${unmet[@]:-}" | jq -R . | jq -sc 'map(select(length > 0))')"
per_seed_array_json="$(printf '%s\n' "${per_seed_json[@]}" | jq -sc 'map(select(type == "object"))')"
case_results_array_json="$(printf '%s\n' "${case_results_json[@]}" | jq -sc 'map(select(type == "object"))')"
seeds_json="$(jq -c '.cert.seeds' "$CONTRACT_FILE")"
profiles_json="$(jq -c '.profiles' "$CONTRACT_FILE")"

{
    echo "Verilog-2005 Conformance Gate Summary"
    echo "gate: verilog_2005_conformance_gate"
    echo "version: 1"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "contract_file: $CONTRACT_FILE"
    echo "grammar_file: $GRAMMAR_FILE"
    echo "corpus_dir: $CORPUS_DIR"
    echo "profiles: $profiles_json"
    echo "lint_exit_code: $lint_rc (expected $lint_expected_rc)"
    echo "lint_profile_orphans: $lint_orphans_headline (expected $lint_expected_orphans; '${lint_orphan_profile}' orphan lines=$lint_orphans_lines)"
    echo "corpus_checks: $matrix_checked"
    echo "corpus_mismatches: $matrix_mismatched"
    echo "alias_checks: $alias_checked"
    echo "cert_headline: $overall_headline"
    echo "cert_seeds: $seeds_json"
    echo "gate_green: $gate_green"
    echo "unmet_criteria_count: $unmet_count"
    echo "primary_unmet_criterion: $primary_unmet"
    echo "unmet_criteria_json: $unmet_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "verilog_2005_conformance_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg contract_file "$CONTRACT_FILE" \
    --arg grammar_file "$GRAMMAR_FILE" \
    --arg corpus_dir "$CORPUS_DIR" \
    --argjson profiles "$profiles_json" \
    --argjson lint_exit_code "$lint_rc" \
    --argjson lint_expected_exit_code "$lint_expected_rc" \
    --argjson lint_profile_orphans "$lint_orphans_headline" \
    --argjson lint_expected_profile_orphans "$lint_expected_orphans" \
    --argjson corpus_checks "$matrix_checked" \
    --argjson corpus_mismatches "$matrix_mismatched" \
    --argjson alias_checks "$alias_checked" \
    --argjson cert_seeds "$seeds_json" \
    --argjson per_seed "$per_seed_array_json" \
    --argjson case_results "$case_results_array_json" \
    --argjson gate_green "$gate_green" \
    --argjson unmet_criteria_count "$unmet_count" \
    --arg primary_unmet_criterion "$primary_unmet" \
    --argjson unmet_criteria "$unmet_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      contract_file: $contract_file,
      grammar_file: $grammar_file,
      corpus_dir: $corpus_dir,
      profiles: $profiles,
      lint: {
        exit_code: $lint_exit_code,
        expected_exit_code: $lint_expected_exit_code,
        profile_orphans: $lint_profile_orphans,
        expected_profile_orphans: $lint_expected_profile_orphans
      },
      corpus: {
        checks: $corpus_checks,
        mismatches: $corpus_mismatches,
        alias_checks: $alias_checks,
        case_results: $case_results
      },
      cert: {
        seeds: $cert_seeds,
        per_seed: $per_seed
      },
      gate_green: $gate_green,
      unmet_criteria_count: $unmet_criteria_count,
      primary_unmet_criterion: $primary_unmet_criterion,
      unmet_criteria: $unmet_criteria
    }' >"$SUMMARY_JSON"

if [[ "$gate_green" != "true" ]]; then
    echo "❌ verilog_2005 conformance gate NOT green: primary_unmet=${primary_unmet}" >&2
    echo "   (see $SUMMARY_TXT / $SUMMARY_JSON and logs under $LOG_DIR)" >&2
    exit 1
fi

echo "✅ verilog_2005 conformance gate passed (lint orphans=${lint_orphans_headline}, corpus ${matrix_checked} checks/0 mismatches, aliases ${alias_checked}, cert deterministic across seeds ${seeds_json})."
