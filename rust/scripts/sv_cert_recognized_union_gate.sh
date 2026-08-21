#!/usr/bin/env bash
set -euo pipefail

# sv_cert_recognized_union_gate.sh
#
# GRAMMAR-WELLFORMED.H.12.8.5.2 — the recognized SystemVerilog certificate-coverage
# UNION accounting basis, as a re-runnable, deterministic, regression-locked oracle.
#
# It runs `ast_pipeline --report-certificate-coverage` with the complete multi-config
# `--cert-union-config` set FOR EACH declared seed, parses the canonical
# `CERTIFICATE-COVERAGE:` line and the `CERTIFICATE-COVERAGE-UNION:` line (+ the union
# residual rule set), and asserts every field equals the tracked contract — the canonical
# UNKNOWN, the union witness/UNKNOWN, the union residual rule set (order-insensitive),
# sample_parse_failures=0, and that all seeds agree (determinism). Emits summary.txt +
# summary.json; exits nonzero on any drift.
#
# Parser-agnostic by construction: the union is driven entirely by the CLI config list in
# the contract — no grammar names are baked into engine code. Heavy (~2 min/seed), so this
# is a CI/`make` oracle, not a `cargo test --lib` unit.
#
# Exit codes: 0 green · 1 drift or a broken precondition · 2 REFUSES because the contract's
# BASELINE-IDENTITY block did not verify, so this gate cannot tell a regression from a stale
# baseline (the published refusal code — docs/book/src/gate-flow.md §1).

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
GRAMMARS_DIR="$ROOT_DIR/grammars"

STATE_DIR="${PGEN_SV_CERT_RECOGNIZED_UNION_STATE_DIR:-$RUST_DIR/target/sv_cert_recognized_union_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_SV_CERT_RECOGNIZED_UNION_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json}"
GRAMMAR_FILE="$GRAMMARS_DIR/systemverilog.ebnf"
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

# Stage logs (e.g. the focus_systemverilog regen, ~5 GB of generation debug output per run)
# are scratch, not evidence: keep only a bounded tail. Per-seed cert logs are NOT routed
# through these helpers and stay complete — they are the parsed evidence artifacts.
PRUNE_THRESHOLD_BYTES=10485760

prune_log() {
    local log_file="$1"
    if [[ -f "$log_file" ]] && [[ "$(wc -c <"$log_file")" -gt "$PRUNE_THRESHOLD_BYTES" ]]; then
        local tmp="${log_file}.tail"
        {
            echo "[pruned by sv_cert_recognized_union_gate: original log exceeded ${PRUNE_THRESHOLD_BYTES} bytes; last 2000 lines retained]"
            tail -n 2000 "$log_file"
        } >"$tmp" && mv "$tmp" "$log_file"
    fi
}

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        prune_log "$log_file"
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        prune_log "$log_file"
        exit 1
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

require_tool jq
require_file "$CONTRACT_FILE"
require_file "$GRAMMAR_FILE"

# --- contract schema validation -------------------------------------------------------
jq -e '
    .family == "systemverilog"
    and (.gate == "sv_cert_recognized_union_gate")
    and ((.version | type) == "number")
    and ((.base_entry | type) == "string" and (.base_entry | length) > 0)
    and ((.base_profile | type) == "string" and (.base_profile | length) > 0)
    and ((.samples | type) == "number")
    and ((.seeds | type) == "array" and (.seeds | length) > 0)
    and ((.union_configs | type) == "array" and (.union_configs | length) > 0)
    and ((.expected_total | type) == "number")
    and ((.expected_proof | type) == "number")
    and ((.expected_canonical_witness | type) == "number")
    and ((.expected_canonical_unknown | type) == "number")
    and ((.expected_union_witness | type) == "number")
    and ((.expected_union_unknown | type) == "number")
    and ((.expected_union_residual_rules | type) == "array")
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
' "$CONTRACT_FILE" >/dev/null || {
    echo "error: contract schema invalid: $CONTRACT_FILE" >&2
    exit 1
}

# --- BASELINE IDENTITY: is this contract still describing THIS tree? -------------------
#
# Doctrine `BASELINE-IDENTITY` (SV-CORPUS-GRAD.13c.2x.2, director-ordered 2026-08-20). Every
# expected_* field below is an exact function of the SV grammar, the generated parser it
# certifies through, and the three engine surfaces the contract's own identity block declares.
#
# ⛔ THIS RUNS BEFORE ANY MEASUREMENT, DELIBERATELY. SV-CORPUS-GRAD.13c.2x measured this gate
# RED at HEAD by 71 rules and 53 UNKNOWNs after ELEVEN revisions of grammars/systemverilog.ebnf
# — and with no identity block, that RED could say "something is wrong" but never "your
# baseline is eleven revisions old". Spending ~2 minutes per seed to arrive at an
# undiagnosable verdict is the waste. Saying it in under a second is the fix.
#
# ⛔ AND THE REFUSAL IS THE POINT, NOT A COURTESY. SV-CORPUS-GRAD.13i found six oracles
# carrying an identity block that NOTHING read, four of them measurably stale. A block with no
# consumer passes every check that only asks whether it exists — so this gate is the consumer,
# and it REFUSES (exit 2, the published code for "cannot see what it is meant to check") rather
# than reporting a drift it could not attribute.
# ⛔⛔ THE FULL MATRIX (`SV-CORPUS-GRAD.13c.2x.4`). An identity DISAMBIGUATES this gate's verdict;
# it does not gate the measurement. Refusing on any divergence was a design error — it refused in
# the very case where running resolves everything, and blocked commits repo-wide.
#   0 fresh+confirmed -> measure
#   1 STALE           -> measure anyway; GREEN means the baseline was stale and still correct
#   3 UNCONFIRMED     -> refuse BEFORE the ~2 min/seed work: a person already ruled the numbers
#                        wrong, so measuring teaches nothing
identity_rc=0
identity_stale=0
"$ROOT_DIR/scripts/check_baseline_identity.sh" --verify "${CONTRACT_FILE#"$ROOT_DIR/"}" \
    || identity_rc=$?
case "$identity_rc" in
    0) ;;
    1) identity_stale=1 ;;
    3)
        {
            echo ""
            echo "sv_cert_recognized_union_gate: REFUSING TO MEASURE — the contract's expectations"
            echo "  are UNCONFIRMED (see above). A person has already recorded that these numbers"
            echo "  do not describe the recorded tree, so ~2 minutes per seed would resolve"
            echo "  nothing. Adjudicating them is SV-CORPUS-GRAD.13c.2x(a)."
        } >&2
        exit 2
        ;;
    *)
        {
            echo ""
            echo "sv_cert_recognized_union_gate: REFUSING TO MEASURE — the contract's identity"
            echo "  could not be read (rc=$identity_rc). Resolve the refusal above first."
        } >&2
        exit 2
        ;;
esac
# ⭐⭐ DELIVERED BY `SV-CORPUS-GRAD.13c.2x`(c), which is the slice that made the cell reachable: on
# STALE + GREEN this run RE-STAMPS the baseline itself, on `sv_syntax_closure_gate`'s model. Every
# expectation was re-derived against the current tree and every one held, so "stale and still
# correct" is a DERIVED fact needing no human judgement — and re-stamping it here is what removes
# the adoption friction that leaves a block unread. The guard that keeps it honest: the re-stamp is
# reached ONLY after `recognized_basis_green == true`, and every stage above exits non-zero rather
# than skipping, so a vacuous green cannot arrive there.
# ⛔ STALE + RED is the one cell that genuinely needs a person, and this gate now says so rather
# than letting the RED read as "the tree regressed" — the exact wrong diagnosis when the baseline
# is the stale half, which is the shape that left this contract wrong for eleven revisions.

base_entry="$(jq -r '.base_entry' "$CONTRACT_FILE")"
base_profile="$(jq -r '.base_profile' "$CONTRACT_FILE")"
samples="$(jq -r '.samples' "$CONTRACT_FILE")"
expected_total="$(jq -r '.expected_total' "$CONTRACT_FILE")"
expected_proof="$(jq -r '.expected_proof' "$CONTRACT_FILE")"
expected_canonical_witness="$(jq -r '.expected_canonical_witness' "$CONTRACT_FILE")"
expected_canonical_unknown="$(jq -r '.expected_canonical_unknown' "$CONTRACT_FILE")"
expected_union_witness="$(jq -r '.expected_union_witness' "$CONTRACT_FILE")"
expected_union_unknown="$(jq -r '.expected_union_unknown' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"

# canonical sorted residual set from the contract (order-insensitive comparison basis)
expected_residual_sorted="$(jq -c '.expected_union_residual_rules | sort' "$CONTRACT_FILE")"

mapfile -t seeds < <(jq -r '.seeds[]' "$CONTRACT_FILE")
mapfile -t union_configs < <(jq -r '.union_configs[]' "$CONTRACT_FILE")

# build the repeatable --cert-union-config arg vector
union_args=()
for cfg in "${union_configs[@]}"; do
    union_args+=(--cert-union-config "$cfg")
done

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

# --- ensure the generated SV parser, then build the DEBUG ast_pipeline -----------------
# build.rs sets has_generated_systemverilog_parser cfg when generated/systemverilog_parser.rs
# exists; cert-coverage verifies witnesses through that real generated parser.
run_logged_rust "ensure_generated_systemverilog_parser" \
    make -C "$RUST_DIR" SHELL=/bin/bash focus_systemverilog
run_logged_rust "build_debug_ast_pipeline" \
    cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

# --- per-seed cert-union run + parse ---------------------------------------------------
declare -a unmet=()
declare -a per_seed_json=()

first_seed_signature=""
first_seed_input_sha=""
first_seed_id=""
overall_canonical_unknown=""
overall_union_unknown=""
overall_union_witness=""
overall_residual_sorted=""

# ⛔⛔ THE INPUT IS PINNED PER ITERATION (SV-CORPUS-GRAD.13c.2x.1 (d)). This loop re-reads
# `$GRAMMAR_FILE` FROM DISK once per seed and then asserts that every seed agreed. That is a claim
# about the TOOL, and it only means that if the INPUT was the same each time — which nothing here
# recorded. Measured consequence, 2026-08-20: this gate reported `canonical total` 1434 at seed 0
# and 1433 at seeds 7/42 and fired `seed=7 signature drift vs seed 0`; TWO hypotheses were written
# down, both about the tool (per-process nondeterminism, a seeded append site), and BOTH were false.
# `certificate_coverage()` is pure and its `total` is `grammar.rule_order.len()`; re-measured on the
# exhibiting arm it read 1434 six times out of six across three processes AND three seeds. What had
# actually differed was the FILE: a multi-seed run takes minutes, and an ordinary apply/revert of an
# experimental grammar arm lands inside that window.
#
# ⭐ THE DIGEST IS EVIDENCE, NOT A NEW FAILURE CONDITION, and that distinction is deliberate. A
# byte digest that FAILED on its own would fire on a comment-only edit the frontend strips — the
# `SV-CORPUS-GRAD.13c.2x.4` defect, where one comment line blocked every commit. So the digest is
# recorded every iteration and consulted only when the signature has ALREADY drifted, where it
# turns an unattributable RED into a named cause. Neither branch can newly fail a run that would
# otherwise have passed.
# ⭐ ONE implementation of the attribution, shared rather than re-spelled per gate.
source "$ROOT_DIR/rust/scripts/lib/drift_attribution.sh"

for seed in "${seeds[@]}"; do
    cert_log="$LOG_DIR/cert_seed_${seed}.log"
    # ⛔ The digest is taken INLINE, at the point of use, deliberately. Hiding it behind a helper
    # defined above the loop makes `gate_input_pin_census.sh` — which scans from the loop opener
    # down — report this gate as BLIND, and tuning the CENSUS to recognise the helper would be
    # tuning the instrument to flatter the change it is measuring. The code moves; the measure does not.
    seed_input_sha="$(shasum -a 256 "$GRAMMAR_FILE" 2>/dev/null | awk '{print $1}')"
    echo "==> cert_union seed=${seed} (grammar ${seed_input_sha:0:16}…)"
    if ! "$AST_PIPELINE_BIN" "$GRAMMAR_FILE" --report-certificate-coverage \
        --grammar-profile "$base_profile" --entry-rule "$base_entry" \
        --count "$samples" --seed "$seed" \
        "${union_args[@]}" >"$cert_log" 2>&1; then
        echo "error: cert-coverage run failed for seed ${seed} (log: $cert_log)" >&2
        tail -n 80 "$cert_log" >&2 || true
        exit 1
    fi

    canon_line="$(grep -E '^CERTIFICATE-COVERAGE: ' "$cert_log" | head -n 1 || true)"
    union_line="$(grep -E '^CERTIFICATE-COVERAGE-UNION: ' "$cert_log" | head -n 1 || true)"
    if [[ -z "$canon_line" || -z "$union_line" ]]; then
        echo "error: missing cert headline line(s) for seed ${seed} (log: $cert_log)" >&2
        exit 1
    fi

    canon_total="$(field_int "$canon_line" total)"
    canon_proof="$(field_int "$canon_line" proof)"
    canon_witness="$(field_int "$canon_line" witness)"
    canon_unknown="$(field_int "$canon_line" UNKNOWN)"
    canon_spf="$(field_int "$canon_line" sample_parse_failures)"

    union_total="$(field_int "$union_line" total)"
    union_proof="$(field_int "$union_line" proof)"
    union_witness="$(field_int "$union_line" witness)"
    union_unknown="$(field_int "$union_line" UNKNOWN)"

    # union residual rule set (order-insensitive). The "UNION UNKNOWN rules (...)" line is
    # present only when union UNKNOWN > 0; when it reaches 0 the residual is the empty set.
    residual_raw="$(grep -E 'UNION UNKNOWN rules \([0-9]+ of [0-9]+ shown\): ' "$cert_log" \
        | head -n 1 | sed -E 's/.*shown\): //' || true)"
    if [[ -z "$residual_raw" ]]; then
        residual_sorted='[]'
    else
        residual_sorted="$(printf '%s' "$residual_raw" | jq -c 'sort')"
    fi

    # per-seed contract assertions
    [[ "$canon_total" == "$expected_total" ]] || unmet+=("seed=${seed} canonical total=${canon_total} (expected ${expected_total})")
    [[ "$canon_proof" == "$expected_proof" ]] || unmet+=("seed=${seed} canonical proof=${canon_proof} (expected ${expected_proof})")
    [[ "$canon_witness" == "$expected_canonical_witness" ]] || unmet+=("seed=${seed} canonical witness=${canon_witness} (expected ${expected_canonical_witness})")
    [[ "$canon_unknown" == "$expected_canonical_unknown" ]] || unmet+=("seed=${seed} canonical UNKNOWN=${canon_unknown} (expected ${expected_canonical_unknown})")
    [[ "$canon_spf" == "0" ]] || unmet+=("seed=${seed} canonical sample_parse_failures=${canon_spf} (expected 0)")
    [[ "$union_total" == "$expected_total" ]] || unmet+=("seed=${seed} union total=${union_total} (expected ${expected_total})")
    [[ "$union_proof" == "$expected_proof" ]] || unmet+=("seed=${seed} union proof=${union_proof} (expected ${expected_proof})")
    [[ "$union_witness" == "$expected_union_witness" ]] || unmet+=("seed=${seed} union witness=${union_witness} (expected ${expected_union_witness})")
    [[ "$union_unknown" == "$expected_union_unknown" ]] || unmet+=("seed=${seed} union UNKNOWN=${union_unknown} (expected ${expected_union_unknown})")
    [[ "$residual_sorted" == "$expected_residual_sorted" ]] || unmet+=("seed=${seed} union residual=${residual_sorted} (expected ${expected_residual_sorted})")

    # determinism: every seed must agree with the first seed's full signature
    signature="${canon_total}|${canon_proof}|${canon_witness}|${canon_unknown}|${canon_spf}|${union_total}|${union_proof}|${union_witness}|${union_unknown}|${residual_sorted}"
    if [[ -z "$first_seed_signature" ]]; then
        first_seed_signature="$signature"
        first_seed_input_sha="$seed_input_sha"
        first_seed_id="$seed"
        overall_canonical_unknown="$canon_unknown"
        overall_union_unknown="$union_unknown"
        overall_union_witness="$union_witness"
        overall_residual_sorted="$residual_sorted"
    elif [[ "$signature" != "$first_seed_signature" ]]; then
        # ⛔ ATTRIBUTE THE DRIFT, in the ONE place that knows how. The two causes send a reader to
        # opposite places, and the old message named only the axis this loop happened to vary.
        unmet+=("$(attribute_signature_drift "$seed" "$first_seed_id" "$seed_input_sha" \
            "$first_seed_input_sha" "${GRAMMAR_FILE#"$ROOT_DIR/"}" "$signature" "$first_seed_signature")")
    fi

    per_seed_json+=("$(jq -nc \
        --argjson seed "$seed" \
        --argjson canonical_total "$canon_total" \
        --argjson canonical_proof "$canon_proof" \
        --argjson canonical_witness "$canon_witness" \
        --argjson canonical_unknown "$canon_unknown" \
        --argjson canonical_sample_parse_failures "$canon_spf" \
        --argjson union_total "$union_total" \
        --argjson union_proof "$union_proof" \
        --argjson union_witness "$union_witness" \
        --argjson union_unknown "$union_unknown" \
        --argjson union_residual_rules "$residual_sorted" \
        --arg grammar_sha256 "$seed_input_sha" \
        '{seed: $seed, grammar_sha256: $grammar_sha256, canonical: {total: $canonical_total, proof: $canonical_proof, witness: $canonical_witness, unknown: $canonical_unknown, sample_parse_failures: $canonical_sample_parse_failures}, union: {total: $union_total, proof: $union_proof, witness: $union_witness, unknown: $union_unknown, residual_rules: $union_residual_rules}}')")
done

recognized_basis_green=false
unmet_count="${#unmet[@]}"
if [[ "$unmet_count" -eq 0 ]]; then
    recognized_basis_green=true
fi
fully_certified_via_union=false
if [[ "$overall_union_unknown" == "0" ]]; then
    fully_certified_via_union=true
fi

primary_unmet="<none>"
if [[ "$unmet_count" -gt 0 ]]; then
    primary_unmet="${unmet[0]}"
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
unmet_json="$(printf '%s\n' "${unmet[@]:-}" | jq -R . | jq -sc 'map(select(length > 0))')"
# per_seed_json entries are already compact JSON objects (one line each) — slurp directly.
per_seed_array_json="$(printf '%s\n' "${per_seed_json[@]}" | jq -sc 'map(select(type == "object"))')"
seeds_json="$(jq -c '.seeds' "$CONTRACT_FILE")"
union_configs_json="$(jq -c '.union_configs' "$CONTRACT_FILE")"

{
    echo "SV Recognized Certificate-Coverage Union Gate Summary"
    echo "gate: sv_cert_recognized_union_gate"
    echo "version: 1"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "contract_file: $CONTRACT_FILE"
    echo "status_rule_done: $done_rule"
    echo "base_entry: $base_entry"
    echo "base_profile: $base_profile"
    echo "samples: $samples"
    echo "seeds: $seeds_json"
    echo "union_configs: $union_configs_json"
    echo "recognized_basis_green: $recognized_basis_green"
    echo "fully_certified_via_union: $fully_certified_via_union"
    echo "canonical_unknown: $overall_canonical_unknown"
    echo "union_unknown: $overall_union_unknown"
    echo "union_witness: $overall_union_witness"
    echo "union_residual_rules: $overall_residual_sorted"
    echo "expected_canonical_unknown: $expected_canonical_unknown"
    echo "expected_union_unknown: $expected_union_unknown"
    echo "expected_union_witness: $expected_union_witness"
    echo "expected_union_residual_rules: $expected_residual_sorted"
    echo "unmet_criteria_count: $unmet_count"
    echo "primary_unmet_criterion: $primary_unmet"
    echo "unmet_criteria_json: $unmet_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "sv_cert_recognized_union_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg contract_file "$CONTRACT_FILE" \
    --arg status_rule_done "$done_rule" \
    --arg base_entry "$base_entry" \
    --arg base_profile "$base_profile" \
    --argjson samples "$samples" \
    --argjson seeds "$seeds_json" \
    --argjson union_configs "$union_configs_json" \
    --argjson recognized_basis_green "$recognized_basis_green" \
    --argjson fully_certified_via_union "$fully_certified_via_union" \
    --argjson canonical_unknown "${overall_canonical_unknown:-null}" \
    --argjson union_unknown "${overall_union_unknown:-null}" \
    --argjson union_witness "${overall_union_witness:-null}" \
    --argjson union_residual_rules "${overall_residual_sorted:-[]}" \
    --argjson expected_canonical_unknown "$expected_canonical_unknown" \
    --argjson expected_union_unknown "$expected_union_unknown" \
    --argjson expected_union_witness "$expected_union_witness" \
    --argjson expected_union_residual_rules "$expected_residual_sorted" \
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
      base_entry: $base_entry,
      base_profile: $base_profile,
      samples: $samples,
      seeds: $seeds,
      union_configs: $union_configs,
      recognized_basis_green: $recognized_basis_green,
      fully_certified_via_union: $fully_certified_via_union,
      observed: {
        canonical_unknown: $canonical_unknown,
        union_unknown: $union_unknown,
        union_witness: $union_witness,
        union_residual_rules: $union_residual_rules
      },
      expected: {
        canonical_unknown: $expected_canonical_unknown,
        union_unknown: $expected_union_unknown,
        union_witness: $expected_union_witness,
        union_residual_rules: $expected_union_residual_rules
      },
      unmet_criteria_count: $unmet_criteria_count,
      primary_unmet_criterion: $primary_unmet_criterion,
      unmet_criteria: $unmet_criteria,
      per_seed: $per_seed
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

if [[ "$recognized_basis_green" != "true" ]]; then
    echo "❌ SV recognized cert-coverage union basis NOT green: primary_unmet=${primary_unmet}" >&2
    echo "   (see $SUMMARY_TXT / $SUMMARY_JSON and per-seed logs under $LOG_DIR)" >&2
    if (( identity_stale == 1 )); then
        {
            echo ""
            echo "  ⛔ AND THIS BASELINE IS STALE: a declared input moved since it was confirmed."
            echo "     So the failure above is AMBIGUOUS — it may be a real regression, or the"
            echo "     contract may simply no longer describe this tree. Adjudicate it; do NOT"
            echo "     re-stamp to make it green, because a stamp asserts the numbers were"
            echo "     RE-DERIVED and matched, which is precisely what just did not happen."
        } >&2
    fi
    exit 1
fi

# ⭐⭐ STALE + GREEN => THIS RUN IS THE CONFIRMING RUN (see the matrix at the identity check above).
if (( identity_stale == 1 )); then
    echo "==> baseline_identity_restamp"
    if "$ROOT_DIR/scripts/check_baseline_identity.sh" --stamp "${CONTRACT_FILE#"$ROOT_DIR/"}" \
        --confirmed-by "auto re-stamped by a GREEN sv_cert_recognized_union_gate run: \
canonical=${expected_total}/${expected_proof}/${expected_canonical_witness}/${overall_canonical_unknown} \
union_witness=${overall_union_witness} union_unknown=${overall_union_unknown} \
residual=${overall_residual_sorted} seeds=${seeds_json} sample_parse_failures=0"; then
        echo "    the baseline was STALE and every expectation still held, so this run re-stamped it"
    else
        echo "    ⛔ the re-stamp FAILED — the gate's verdict stands, but the baseline is still stale" >&2
        exit 1
    fi
fi

echo "✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=${overall_canonical_unknown}, union UNKNOWN=${overall_union_unknown}, residual=${overall_residual_sorted}, deterministic across seeds ${seeds_json})."
