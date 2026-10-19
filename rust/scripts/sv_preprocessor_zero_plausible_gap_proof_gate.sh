#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_zero_plausible_gap_proof_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json}"
SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_syntax_closure_gate.sh"
AGGREGATE_CONTRACT_GATE="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"
REACHABILITY_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_reachability_closure_gate.sh"

EXISTING_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_EXISTING_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_AGGREGATE_CONTRACT_STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_EXISTING_AGGREGATE_CONTRACT_STATE_DIR:-}"
EXISTING_REACHABILITY_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_EXISTING_REACHABILITY_CLOSURE_STATE_DIR:-}"

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

top_level_summary_value_from_txt() {
    local key="$1"
    local path="$2"
    local line
    line="$(awk -v key="$key" 'index($0, key ": ") == 1 { print; exit }' "$path")"
    if [[ -z "$line" ]]; then
        echo "error: missing top-level key '${key}' in summary '$path'" >&2
        exit 1
    fi
    printf '%s\n' "${line#${key}: }"
}

summary_value_from_txt() {
    local key="$1"
    local path="$2"
    local line
    line="$(awk -v key="$key" '
        {
            trimmed = $0
            sub(/^[[:space:]]+/, "", trimmed)
            if (index(trimmed, key ": ") == 1) {
                print trimmed
                exit
            }
        }
    ' "$path")"
    if [[ -z "$line" ]]; then
        echo "error: missing key '${key}' in summary '$path'" >&2
        exit 1
    fi
    printf '%s\n' "${line#${key}: }"
}

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

fraction_is_full() {
    local value="$1"
    if [[ "$value" =~ ^([0-9]+)/([0-9]+)$ ]]; then
        [[ "${BASH_REMATCH[1]}" == "${BASH_REMATCH[2]}" ]]
        return
    fi
    return 1
}

require_tool jq
require_file "$CONTRACT_FILE"
require_file "$SYNTAX_CLOSURE_GATE"
require_file "$AGGREGATE_CONTRACT_GATE"
require_file "$REACHABILITY_CLOSURE_GATE"

# Contract version 4 (SV-CORPUS-GRAD.13c.2v.21, 2026-10-19) adds the per-class pins; a contract missing one is
# REFUSED BY NAME (v3 refused silently — a `jq -e` under `set -e` with no message, the same shape the
# syntax-closure gate fixed at SV-CORPUS-GRAD.13c.2x.3).
if ! jq -e '
    .family == "systemverilog_preprocessor"
    and ((.version | type) == "number")
    and ((.description | type) == "string" and (.description | length) > 0)
    and ((.grammar_name | type) == "string" and (.grammar_name | length) > 0)
    and ((.entry_rule | type) == "string" and (.entry_rule | length) > 0)
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
    and ((.allowed_unreachable_rules | type) == "array")
    and ((.allowed_unreachable_branches | type) == "array")
    and ((.required_unreachable_rule_reason | type) == "string" and (.required_unreachable_rule_reason | length) > 0)
    and ((.required_unreachable_branch_rule_name | type) == "string" and (.required_unreachable_branch_rule_name | length) > 0)
    and ((.required_unreachable_branch_reason | type) == "string" and (.required_unreachable_branch_reason | length) > 0)
    and ((.helper_only_whitelist_detail | type) == "string" and (.helper_only_whitelist_detail | length) > 0)
    and ((.default_profile | type) == "string" and (.default_profile | length) > 0)
    and ((.expected_lookahead_only_rules | type) == "array")
    and ((.expected_inside_lookahead_branches | type) == "number")
    and ((.expected_edition_gated_branches | type) == "array")
    and all(.expected_edition_gated_branches[];
            (.branch_id | type) == "string" and (.rule_name | type) == "string"
            and (.gated_rule | type) == "string" and (.profiles | type) == "array" and (.profiles | length) > 0)
' "$CONTRACT_FILE" >/dev/null; then
    echo "error: $CONTRACT_FILE is not a version-4 zero-plausible-gap proof contract — it needs family, version, description, grammar_name, entry_rule, done_rule, allowed_unreachable_rules, allowed_unreachable_branches, required_unreachable_rule_reason, required_unreachable_branch_rule_name, required_unreachable_branch_reason, helper_only_whitelist_detail (the v3 fields) and default_profile, expected_lookahead_only_rules, expected_inside_lookahead_branches, expected_edition_gated_branches (each {branch_id, rule_name, gated_rule, profiles}); NAME the missing pin — a missing pin is a refusal, never an empty class" >&2
    exit 1
fi

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

syntax_state_dir="${EXISTING_SYNTAX_CLOSURE_STATE_DIR:-$WORK_DIR/sv_preprocessor_syntax_closure_gate}"
if [[ -z "$EXISTING_SYNTAX_CLOSURE_STATE_DIR" ]]; then
    run_logged "sv_preprocessor_syntax_closure_gate" \
        env \
            PGEN_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="$syntax_state_dir" \
            "$SYNTAX_CLOSURE_GATE"
else
    syntax_state_dir="$(cd "$syntax_state_dir" && pwd)"
fi

aggregate_state_dir="${EXISTING_AGGREGATE_CONTRACT_STATE_DIR:-$WORK_DIR/sv_preprocessor_aggregate_contract_gate}"
if [[ -z "$EXISTING_AGGREGATE_CONTRACT_STATE_DIR" ]]; then
    run_logged "sv_preprocessor_aggregate_contract_gate" \
        env \
            PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR="$aggregate_state_dir" \
            "$AGGREGATE_CONTRACT_GATE"
else
    aggregate_state_dir="$(cd "$aggregate_state_dir" && pwd)"
fi

reachability_state_dir="${EXISTING_REACHABILITY_CLOSURE_STATE_DIR:-$WORK_DIR/sv_preprocessor_reachability_closure_gate}"
if [[ -z "$EXISTING_REACHABILITY_CLOSURE_STATE_DIR" ]]; then
    run_logged "sv_preprocessor_reachability_closure_gate" \
        env \
            PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_STATE_DIR="$reachability_state_dir" \
            "$REACHABILITY_CLOSURE_GATE"
else
    reachability_state_dir="$(cd "$reachability_state_dir" && pwd)"
fi

syntax_summary_txt="$syntax_state_dir/summary.txt"
syntax_gap_json="$syntax_state_dir/work/systemverilog_preprocessor_syntax_probe_gap.json"
aggregate_summary_txt="$aggregate_state_dir/summary.txt"
reachability_summary_txt="$reachability_state_dir/summary.txt"

require_nonempty_file "$syntax_summary_txt"
require_nonempty_file "$syntax_gap_json"
require_nonempty_file "$aggregate_summary_txt"
require_nonempty_file "$reachability_summary_txt"

contract_version="$(jq -r '.version' "$CONTRACT_FILE")"
family="$(jq -r '.family' "$CONTRACT_FILE")"
grammar_name="$(jq -r '.grammar_name' "$CONTRACT_FILE")"
entry_rule="$(jq -r '.entry_rule' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"
helper_only_whitelist_detail="$(jq -r '.helper_only_whitelist_detail' "$CONTRACT_FILE")"
required_unreachable_rule_reason="$(jq -r '.required_unreachable_rule_reason' "$CONTRACT_FILE")"
required_unreachable_branch_rule_name="$(jq -r '.required_unreachable_branch_rule_name' "$CONTRACT_FILE")"
required_unreachable_branch_reason="$(jq -r '.required_unreachable_branch_reason' "$CONTRACT_FILE")"
allowed_unreachable_rules_json="$(jq -cer '.allowed_unreachable_rules | sort' "$CONTRACT_FILE")"
allowed_unreachable_branches_json="$(jq -cer '.allowed_unreachable_branches | sort' "$CONTRACT_FILE")"
default_profile="$(jq -r '.default_profile' "$CONTRACT_FILE")"
expected_lookahead_only_rules_json="$(jq -cer '.expected_lookahead_only_rules | sort' "$CONTRACT_FILE")"
expected_inside_lookahead_branches="$(jq -r '.expected_inside_lookahead_branches' "$CONTRACT_FILE")"
expected_edition_gated_branches_json="$(jq -cer '.expected_edition_gated_branches | sort_by(.branch_id)' "$CONTRACT_FILE")"
expected_edition_gated_branch_ids_json="$(jq -cer '[.expected_edition_gated_branches[].branch_id] | sort' "$CONTRACT_FILE")"

# The syntax-closure gate's own products for THIS grammar, read from its summary: its contract (the lockstep
# partner of the guard / interior pins — a release that re-pins one contract and not the other is refused
# here by name, the 2026-10-18 shape) and the raw AST envelope (the engine's parse of the EBNF: every
# rule's @profiles and the grammar's @default_profile, the oracle for the edition-gated class).
syntax_contract_file="$(top_level_summary_value_from_txt "contract_file" "$syntax_summary_txt")"
syntax_raw_ast_json="$(top_level_summary_value_from_txt "grammar_raw_ast_json" "$syntax_summary_txt")"
require_file "$syntax_contract_file"
require_nonempty_file "$syntax_raw_ast_json"
if ! syntax_contract_lookahead_only_rules_json="$(jq -cer '.constraints.expected_lookahead_only_rules | select(type == "array") | sort' "$syntax_contract_file" 2>/dev/null)"; then
    echo "error: constraints.expected_lookahead_only_rules is ABSENT from $syntax_contract_file — the guard set is pinned two-sided in BOTH contracts (this proof's expected_lookahead_only_rules reads ${expected_lookahead_only_rules_json}); NAME it there" >&2
    exit 1
fi
if ! syntax_contract_inside_lookahead_branches="$(jq -er '.constraints.expected_inside_lookahead_branches | numbers' "$syntax_contract_file" 2>/dev/null)"; then
    echo "error: constraints.expected_inside_lookahead_branches is ABSENT from $syntax_contract_file — the interior count is pinned two-sided in BOTH contracts (this proof's expected_inside_lookahead_branches reads ${expected_inside_lookahead_branches}); NAME it there" >&2
    exit 1
fi

syntax_summary_state_dir="$(top_level_summary_value_from_txt "state_dir" "$syntax_summary_txt")"
aggregate_summary_state_dir="$(top_level_summary_value_from_txt "state_dir" "$aggregate_summary_txt")"
reachability_summary_state_dir="$(top_level_summary_value_from_txt "state_dir" "$reachability_summary_txt")"

if [[ "$syntax_summary_state_dir" != "$syntax_state_dir" ]]; then
    echo "error: preprocessor syntax-closure state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$aggregate_summary_state_dir" != "$aggregate_state_dir" ]]; then
    echo "error: preprocessor aggregate-contract state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$reachability_summary_state_dir" != "$reachability_state_dir" ]]; then
    echo "error: preprocessor reachability-closure state_dir mismatch in summary.txt" >&2
    exit 1
fi

jq -e --arg grammar_name "$grammar_name" --arg entry_rule "$entry_rule" '
    .grammar_name == $grammar_name
    and .entry_rule == $entry_rule
    and (.summary | type == "object")
    and (.unreachable_rule_debt | type == "array")
    and (.unreachable_branch_debt | type == "array")
' "$syntax_gap_json" >/dev/null

syntax_status="pass"
syntax_failure_count="0"
syntax_unresolved_rule_reference_count="$(summary_value_from_txt "unresolved_rule_reference_count" "$syntax_summary_txt")"

aggregate_parseability_parser_rejections_total="$(summary_value_from_txt "parseability_parser_rejections_total" "$aggregate_summary_txt")"
aggregate_parseability_rejected_total="$(summary_value_from_txt "parseability_rejected_total" "$aggregate_summary_txt")"
aggregate_final_targets="$(summary_value_from_txt "final_targets" "$aggregate_summary_txt")"
aggregate_covered_reachable_rules="$(summary_value_from_txt "covered_reachable_rules" "$aggregate_summary_txt")"
aggregate_covered_reachable_branches="$(summary_value_from_txt "covered_reachable_branches" "$aggregate_summary_txt")"

reachability_stage3_targets="$(summary_value_from_txt "stage3_targets" "$reachability_summary_txt")"
reachability_stage4_targets="$(summary_value_from_txt "stage4_targets" "$reachability_summary_txt")"
reachability_stage3_rules="$(summary_value_from_txt "stage3_covered_reachable_rules" "$reachability_summary_txt")"
reachability_stage4_rules="$(summary_value_from_txt "stage4_covered_reachable_rules" "$reachability_summary_txt")"
reachability_stage3_branches="$(summary_value_from_txt "stage3_covered_reachable_branches" "$reachability_summary_txt")"
reachability_stage4_branches="$(summary_value_from_txt "stage4_covered_reachable_branches" "$reachability_summary_txt")"

observed_unreachable_rules_json="$(jq -cer '.unreachable_rule_debt | map(.rule_name) | sort' "$syntax_gap_json")"
observed_unreachable_branches_json="$(jq -cer '.unreachable_branch_debt | map(.branch_id) | sort' "$syntax_gap_json")"

# --- contract v4 (SV-CORPUS-GRAD.13c.2v.21, 2026-10-19): the unreachable surface PARTITIONED BY THE GAP
# REPORT'S OWN REASON, each class judged against its oracle. v3 compared the whole observed set with one
# EMPTY allow-list, so the first guard rule release 1.0.9 added (2026-10-18) turned the proof false — and
# the umbrella above still printed `passed`. The four classes (rust/src/ast_pipeline/stimuli_generator.rs
# assigns the reasons): DEAD, `unreachable_from_entry` — the allow-lists, EMPTY (the literal-zero doctrine);
# GUARD, `reachable_only_inside_lookahead` — pinned two-sided here AND in the syntax-closure contract;
# INTERIOR, `inside_lookahead` — every one inside a pinned guard, the count pinned two-sided in both
# contracts; EDITION-GATED, `references_rule_missing_from_active_grammar` — pinned by id, each verified
# against the gap entry's rule_references and the grammar's @profiles / @default_profile as the raw AST
# carries them. Any other reason is unclassified and refuses the proof.
observed_dead_rules_json="$(jq -cer --arg reason "$required_unreachable_rule_reason" \
    '[.unreachable_rule_debt[]? | select(.reason == $reason) | .rule_name] | sort' "$syntax_gap_json")"
observed_guard_rules_json="$(jq -cer \
    '[.unreachable_rule_debt[]? | select(.reason == "reachable_only_inside_lookahead") | .rule_name] | sort' "$syntax_gap_json")"
observed_unclassified_rules_json="$(jq -cer --arg reason "$required_unreachable_rule_reason" \
    '[.unreachable_rule_debt[]? | select(.reason != $reason and .reason != "reachable_only_inside_lookahead") | {rule_name, reason}] | sort_by(.rule_name)' "$syntax_gap_json")"
observed_dead_branches_json="$(jq -cer --arg reason "$required_unreachable_branch_reason" \
    '[.unreachable_branch_debt[]? | select(.reason == $reason) | .branch_id] | sort' "$syntax_gap_json")"
observed_interior_branches_json="$(jq -cer \
    '[.unreachable_branch_debt[]? | select(.reason == "inside_lookahead") | .branch_id] | sort' "$syntax_gap_json")"
observed_interior_branch_count="$(jq -er 'length' <<<"$observed_interior_branches_json")"
observed_interior_branches_outside_guards_json="$(jq -cer --argjson guards "$expected_lookahead_only_rules_json" \
    '[.unreachable_branch_debt[]? | select(.reason == "inside_lookahead") | select(.rule_name as $r | ($guards | index($r)) == null) | .branch_id] | sort' "$syntax_gap_json")"
observed_edition_gated_branch_ids_json="$(jq -cer \
    '[.unreachable_branch_debt[]? | select(.reason == "references_rule_missing_from_active_grammar") | .branch_id] | sort' "$syntax_gap_json")"
observed_unclassified_branches_json="$(jq -cer --arg reason "$required_unreachable_branch_reason" \
    '[.unreachable_branch_debt[]? | select(.reason != $reason and .reason != "inside_lookahead" and .reason != "references_rule_missing_from_active_grammar") | {branch_id, reason}] | sort_by(.branch_id)' "$syntax_gap_json")"

# The grammar's own reading — @default_profile and each rule's @profiles — from the raw AST envelope the
# syntax-closure gate wrote for THIS grammar (each entry is [["rule", name], ["semantic_annotation",
# [name, value]] …, body]); the engine's parse of the EBNF, not a grep of its text.
grammar_default_profile="$(jq -r \
    '[.raw_ast[] | .[] | select(type == "array" and (.[0] | type) == "string" and .[0] == "semantic_annotation" and (.[1] | type) == "array" and .[1][0] == "default_profile") | .[1][1]] | unique | if length == 1 then .[0] else "<" + (length | tostring) + " default_profile annotations>" end' \
    "$syntax_raw_ast_json")"
grammar_rule_profiles_json="$(jq -c \
    '[.raw_ast[] | select((.[0] | type) == "array" and .[0][0] == "rule") | {name: .[0][1], profiles: ([.[] | select(type == "array" and (.[0] | type) == "string" and .[0] == "semantic_annotation" and (.[1] | type) == "array" and .[1][0] == "profiles") | .[1][1] | fromjson] | first)}]' \
    "$syntax_raw_ast_json")"
edition_gated_verification_json="$(jq -c \
    --argjson pins "$expected_edition_gated_branches_json" \
    --argjson grammar "$grammar_rule_profiles_json" \
    --arg default_profile "$default_profile" '
    . as $gap
    | [ $pins[] as $p
        | ($gap.unreachable_branch_debt | map(select(.branch_id == $p.branch_id)) | first) as $e
        | ($grammar | map(select(.name == $p.gated_rule)) | first) as $g
        | { branch_id: $p.branch_id,
            gated_rule: $p.gated_rule,
            observed: ($e != null),
            reason_ok: ($e != null and $e.reason == "references_rule_missing_from_active_grammar"),
            rule_name_ok: ($e != null and $e.rule_name == $p.rule_name),
            references_gated_rule: ($e != null and ((($e.rule_references // []) | index($p.gated_rule)) != null)),
            gated_rule_defined: ($g != null),
            grammar_profiles: (if $g != null then $g.profiles else null end),
            profiles_match: ($g != null and $g.profiles != null and (($g.profiles | sort) == ($p.profiles | sort))),
            default_profile_excluded: ($g != null and $g.profiles != null and (($g.profiles | index($default_profile)) == null)) }
        | .ok = (.observed and .reason_ok and .rule_name_ok and .references_gated_rule and .gated_rule_defined and .profiles_match and .default_profile_excluded) ]' \
    "$syntax_gap_json")"
edition_gated_failures_json="$(jq -c '[.[] | select(.ok | not)]' <<<"$edition_gated_verification_json")"

syntax_preconditions_green=false
aggregate_preconditions_green=false
reachability_preconditions_green=false
dead_rules_green=false
dead_branches_green=false
guard_rules_green=false
interior_branches_green=false
edition_gated_branches_green=false
unclassified_green=false
helper_only_unreachable_rules_green=false
helper_only_unreachable_branches_green=false
helper_only_unreachable_surface_green=false
declare -a class_unmet=()

if [[ "$syntax_status" == "pass" && "$syntax_failure_count" == "0" && "$syntax_unresolved_rule_reference_count" == "0" ]]; then
    syntax_preconditions_green=true
fi

if [[ "$aggregate_parseability_parser_rejections_total" == "0" \
   && "$aggregate_parseability_rejected_total" == "0" \
   && "$aggregate_final_targets" == "0" ]] \
   && fraction_is_full "$aggregate_covered_reachable_rules" \
   && fraction_is_full "$aggregate_covered_reachable_branches"; then
    aggregate_preconditions_green=true
fi

if [[ "$reachability_stage3_targets" == "0" \
   && "$reachability_stage4_targets" == "0" ]] \
   && fraction_is_full "$reachability_stage3_rules" \
   && fraction_is_full "$reachability_stage4_rules" \
   && fraction_is_full "$reachability_stage3_branches" \
   && fraction_is_full "$reachability_stage4_branches"; then
    reachability_preconditions_green=true
fi

# DEAD — the literal-zero doctrine: a dead rule or branch is removed at source, never absorbed.
if [[ "$observed_dead_rules_json" == "$allowed_unreachable_rules_json" ]]; then
    dead_rules_green=true
else
    class_unmet+=("dead rules (reason ${required_unreachable_rule_reason}) differ from allowed_unreachable_rules: observed=${observed_dead_rules_json} allowed=${allowed_unreachable_rules_json} — a dead rule is removed at source (GRAMMAR-WELLFORMED literal zero), never absorbed")
fi
if [[ "$observed_dead_branches_json" == "$allowed_unreachable_branches_json" ]] \
   && { [[ "$allowed_unreachable_branches_json" == "[]" ]] \
        || jq -e --arg rule_name "$required_unreachable_branch_rule_name" --arg reason "$required_unreachable_branch_reason" '
            all(.unreachable_branch_debt[]? | select(.reason == $reason);
                .reachable == false
                and .rule_name == $rule_name
            )
           ' "$syntax_gap_json" >/dev/null; }; then
    dead_branches_green=true
else
    class_unmet+=("dead branches (reason ${required_unreachable_branch_reason}) differ from allowed_unreachable_branches: observed=${observed_dead_branches_json} allowed=${allowed_unreachable_branches_json} — a dead branch is removed at source, never absorbed")
fi

# GUARD — pinned two-sided here AND in the syntax-closure contract (one adjudication, two readers).
if [[ "$observed_guard_rules_json" == "$expected_lookahead_only_rules_json" \
      && "$expected_lookahead_only_rules_json" == "$syntax_contract_lookahead_only_rules_json" ]]; then
    guard_rules_green=true
else
    class_unmet+=("guard rules (reason reachable_only_inside_lookahead) are not the two-sided pin of both contracts: observed=${observed_guard_rules_json} expected_lookahead_only_rules=${expected_lookahead_only_rules_json} syntax-closure contract=${syntax_contract_lookahead_only_rules_json} — a guard rule was added, removed or gained a positive use-site, or one contract was re-pinned without the other; re-pin BOTH, naming it")
fi

# INTERIOR — every alternative inside a lookahead belongs to a pinned guard; the count is pinned in both.
if [[ "$observed_interior_branch_count" == "$expected_inside_lookahead_branches" \
      && "$expected_inside_lookahead_branches" == "$syntax_contract_inside_lookahead_branches" \
      && "$observed_interior_branches_outside_guards_json" == "[]" ]]; then
    interior_branches_green=true
else
    class_unmet+=("interior alternatives (reason inside_lookahead): observed=${observed_interior_branch_count} expected_inside_lookahead_branches=${expected_inside_lookahead_branches} syntax-closure contract=${syntax_contract_inside_lookahead_branches} outside a pinned guard=${observed_interior_branches_outside_guards_json} — an alternation inside a lookahead was added or removed; re-measure and re-pin BOTH contracts, naming the rule")
fi

# EDITION-GATED — pinned by id; the gap report must name the gated rule as the branch's reference and the
# grammar's raw AST must carry exactly the pinned @profiles on that rule, @default_profile not among them.
if [[ "$observed_edition_gated_branch_ids_json" == "$expected_edition_gated_branch_ids_json" \
      && "$grammar_default_profile" == "$default_profile" \
      && "$edition_gated_failures_json" == "[]" ]]; then
    edition_gated_branches_green=true
else
    class_unmet+=("edition-gated alternatives (reason references_rule_missing_from_active_grammar): observed=${observed_edition_gated_branch_ids_json} pinned=${expected_edition_gated_branch_ids_json} default_profile=${default_profile} grammar @default_profile=${grammar_default_profile} pins the grammar does not bear out=${edition_gated_failures_json} — an edition form was added or removed, or a gated rule's @profiles moved; re-pin by id with the rule and its profiles")
fi

# UNCLASSIFIED — a reason this proof has no class for refuses it; the gap report grew, so must the proof.
if [[ "$observed_unclassified_rules_json" == "[]" && "$observed_unclassified_branches_json" == "[]" ]]; then
    unclassified_green=true
else
    class_unmet+=("unreachable entries with a reason this proof has no class for: rules=${observed_unclassified_rules_json} branches=${observed_unclassified_branches_json} — the gap report grew a reason; teach the proof the class or fix the grammar")
fi

if [[ "$dead_rules_green" == true && "$guard_rules_green" == true && "$observed_unclassified_rules_json" == "[]" ]]; then
    helper_only_unreachable_rules_green=true
fi
if [[ "$dead_branches_green" == true && "$interior_branches_green" == true \
      && "$edition_gated_branches_green" == true && "$observed_unclassified_branches_json" == "[]" ]]; then
    helper_only_unreachable_branches_green=true
fi
if [[ "$helper_only_unreachable_rules_green" == true && "$helper_only_unreachable_branches_green" == true ]]; then
    helper_only_unreachable_surface_green=true
fi

zero_plausible_grammar_level_gap_proof_surface=false
declare -a unmet_proof_criteria=()

if [[ "$syntax_preconditions_green" != true ]]; then
    unmet_proof_criteria+=("Syntax preconditions regressed: syntax_status=${syntax_status}, syntax_failure_count=${syntax_failure_count}, syntax_unresolved_rule_reference_count=${syntax_unresolved_rule_reference_count}.")
fi
if [[ "$aggregate_preconditions_green" != true ]]; then
    unmet_proof_criteria+=("Aggregate preconditions regressed: parseability_parser_rejections_total=${aggregate_parseability_parser_rejections_total}, parseability_rejected_total=${aggregate_parseability_rejected_total}, final_targets=${aggregate_final_targets}, covered_reachable_rules=${aggregate_covered_reachable_rules}, covered_reachable_branches=${aggregate_covered_reachable_branches}.")
fi
if [[ "$reachability_preconditions_green" != true ]]; then
    unmet_proof_criteria+=("Reachability preconditions regressed: stage3_targets=${reachability_stage3_targets}, stage4_targets=${reachability_stage4_targets}, stage3_rules=${reachability_stage3_rules}, stage4_rules=${reachability_stage4_rules}, stage3_branches=${reachability_stage3_branches}, stage4_branches=${reachability_stage4_branches}.")
fi
if [[ "$helper_only_unreachable_surface_green" != true ]]; then
    # The failing class names itself first (the family-status gate surfaces the FIRST criterion); the
    # contract's own sentence closes the list.
    if (( ${#class_unmet[@]} > 0 )); then
        unmet_proof_criteria+=("${class_unmet[@]}")
    fi
    unmet_proof_criteria+=("$helper_only_whitelist_detail")
fi

if [[ "${#unmet_proof_criteria[@]}" -eq 0 ]]; then
    zero_plausible_grammar_level_gap_proof_surface=true
fi

unmet_proof_criteria_count="${#unmet_proof_criteria[@]}"
primary_unmet_proof_criterion="<none>"
if [[ "$unmet_proof_criteria_count" -gt 0 ]]; then
    primary_unmet_proof_criterion="${unmet_proof_criteria[0]}"
fi
unmet_proof_criteria_json="$(jq -cn '$ARGS.positional' --args "${unmet_proof_criteria[@]}")"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "SV Preprocessor Zero Plausible Gap Proof Gate Summary"
    echo "gate: sv_preprocessor_zero_plausible_gap_proof_gate"
    echo "version: 2"
    echo "generated_at_utc: $generated_at_utc"
    echo "state_dir: $STATE_DIR"
    echo "summary_txt: $SUMMARY_TXT"
    echo "summary_json: $SUMMARY_JSON"
    echo "family: $family"
    echo "contract_file: $CONTRACT_FILE"
    echo "contract_version: $contract_version"
    echo "grammar_name: $grammar_name"
    echo "entry_rule: $entry_rule"
    echo "done_rule: $done_rule"
    echo "syntax_closure_state_dir: $syntax_state_dir"
    echo "syntax_closure_summary_txt: $syntax_summary_txt"
    echo "syntax_gap_json: $syntax_gap_json"
    echo "aggregate_contract_state_dir: $aggregate_state_dir"
    echo "aggregate_contract_summary_txt: $aggregate_summary_txt"
    echo "reachability_closure_state_dir: $reachability_state_dir"
    echo "reachability_closure_summary_txt: $reachability_summary_txt"
    echo "syntax_preconditions_green: $syntax_preconditions_green"
    echo "aggregate_preconditions_green: $aggregate_preconditions_green"
    echo "reachability_preconditions_green: $reachability_preconditions_green"
    echo "helper_only_unreachable_rules_green: $helper_only_unreachable_rules_green"
    echo "helper_only_unreachable_branches_green: $helper_only_unreachable_branches_green"
    echo "helper_only_unreachable_surface_green: $helper_only_unreachable_surface_green"
    echo "zero_plausible_grammar_level_gap_proof_surface: $zero_plausible_grammar_level_gap_proof_surface"
    echo "unmet_proof_criteria_count: $unmet_proof_criteria_count"
    echo "primary_unmet_proof_criterion: $primary_unmet_proof_criterion"
    echo "unmet_proof_criteria_json: $unmet_proof_criteria_json"
    echo "allowed_unreachable_rules_json: $allowed_unreachable_rules_json"
    echo "observed_unreachable_rules_json: $observed_unreachable_rules_json"
    echo "allowed_unreachable_branches_json: $allowed_unreachable_branches_json"
    echo "observed_unreachable_branches_json: $observed_unreachable_branches_json"
    echo "default_profile: $default_profile"
    echo "grammar_default_profile: $grammar_default_profile"
    echo "syntax_contract_file: $syntax_contract_file"
    echo "syntax_raw_ast_json: $syntax_raw_ast_json"
    echo "dead_rules_green: $dead_rules_green"
    echo "dead_branches_green: $dead_branches_green"
    echo "guard_rules_green: $guard_rules_green"
    echo "interior_branches_green: $interior_branches_green"
    echo "edition_gated_branches_green: $edition_gated_branches_green"
    echo "unclassified_green: $unclassified_green"
    echo "observed_dead_rules_json: $observed_dead_rules_json"
    echo "observed_dead_branches_json: $observed_dead_branches_json"
    echo "expected_lookahead_only_rules_json: $expected_lookahead_only_rules_json"
    echo "syntax_contract_lookahead_only_rules_json: $syntax_contract_lookahead_only_rules_json"
    echo "observed_guard_rules_json: $observed_guard_rules_json"
    echo "expected_inside_lookahead_branches: $expected_inside_lookahead_branches"
    echo "syntax_contract_inside_lookahead_branches: $syntax_contract_inside_lookahead_branches"
    echo "observed_inside_lookahead_branches: $observed_interior_branch_count"
    echo "observed_inside_lookahead_branches_outside_guards_json: $observed_interior_branches_outside_guards_json"
    echo "expected_edition_gated_branch_ids_json: $expected_edition_gated_branch_ids_json"
    echo "observed_edition_gated_branch_ids_json: $observed_edition_gated_branch_ids_json"
    echo "edition_gated_failures_json: $edition_gated_failures_json"
    echo "observed_unclassified_rules_json: $observed_unclassified_rules_json"
    echo "observed_unclassified_branches_json: $observed_unclassified_branches_json"
} >"$SUMMARY_TXT"

jq -n \
    --arg gate "sv_preprocessor_zero_plausible_gap_proof_gate" \
    --argjson version 2 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg family "$family" \
    --arg contract_file "$CONTRACT_FILE" \
    --argjson contract_version "$contract_version" \
    --arg grammar_name "$grammar_name" \
    --arg entry_rule "$entry_rule" \
    --arg done_rule "$done_rule" \
    --arg syntax_closure_state_dir "$syntax_state_dir" \
    --arg syntax_closure_summary_txt "$syntax_summary_txt" \
    --arg syntax_gap_json "$syntax_gap_json" \
    --arg aggregate_contract_state_dir "$aggregate_state_dir" \
    --arg aggregate_contract_summary_txt "$aggregate_summary_txt" \
    --arg reachability_closure_state_dir "$reachability_state_dir" \
    --arg reachability_closure_summary_txt "$reachability_summary_txt" \
    --argjson syntax_preconditions_green "$syntax_preconditions_green" \
    --argjson aggregate_preconditions_green "$aggregate_preconditions_green" \
    --argjson reachability_preconditions_green "$reachability_preconditions_green" \
    --argjson helper_only_unreachable_rules_green "$helper_only_unreachable_rules_green" \
    --argjson helper_only_unreachable_branches_green "$helper_only_unreachable_branches_green" \
    --argjson helper_only_unreachable_surface_green "$helper_only_unreachable_surface_green" \
    --argjson zero_plausible_grammar_level_gap_proof_surface "$zero_plausible_grammar_level_gap_proof_surface" \
    --argjson unmet_proof_criteria_count "$unmet_proof_criteria_count" \
    --arg primary_unmet_proof_criterion "$primary_unmet_proof_criterion" \
    --argjson unmet_proof_criteria "$unmet_proof_criteria_json" \
    --argjson allowed_unreachable_rules "$allowed_unreachable_rules_json" \
    --argjson observed_unreachable_rules "$observed_unreachable_rules_json" \
    --argjson allowed_unreachable_branches "$allowed_unreachable_branches_json" \
    --argjson observed_unreachable_branches "$observed_unreachable_branches_json" \
    --arg default_profile "$default_profile" \
    --arg grammar_default_profile "$grammar_default_profile" \
    --arg syntax_contract_file "$syntax_contract_file" \
    --arg syntax_raw_ast_json "$syntax_raw_ast_json" \
    --argjson dead_rules_green "$dead_rules_green" \
    --argjson dead_branches_green "$dead_branches_green" \
    --argjson guard_rules_green "$guard_rules_green" \
    --argjson interior_branches_green "$interior_branches_green" \
    --argjson edition_gated_branches_green "$edition_gated_branches_green" \
    --argjson unclassified_green "$unclassified_green" \
    --argjson observed_dead_rules "$observed_dead_rules_json" \
    --argjson observed_dead_branches "$observed_dead_branches_json" \
    --argjson expected_lookahead_only_rules "$expected_lookahead_only_rules_json" \
    --argjson syntax_contract_lookahead_only_rules "$syntax_contract_lookahead_only_rules_json" \
    --argjson observed_guard_rules "$observed_guard_rules_json" \
    --argjson expected_inside_lookahead_branches "$expected_inside_lookahead_branches" \
    --argjson syntax_contract_inside_lookahead_branches "$syntax_contract_inside_lookahead_branches" \
    --argjson observed_inside_lookahead_branches "$observed_interior_branch_count" \
    --argjson observed_inside_lookahead_branches_outside_guards "$observed_interior_branches_outside_guards_json" \
    --argjson expected_edition_gated_branches "$expected_edition_gated_branches_json" \
    --argjson observed_edition_gated_branch_ids "$observed_edition_gated_branch_ids_json" \
    --argjson edition_gated_verification "$edition_gated_verification_json" \
    --argjson observed_unclassified_rules "$observed_unclassified_rules_json" \
    --argjson observed_unclassified_branches "$observed_unclassified_branches_json" \
    '{
        gate: $gate,
        version: $version,
        generated_at_utc: $generated_at_utc,
        state_dir: $state_dir,
        summary_txt: $summary_txt,
        summary_json: $summary_json,
        family: $family,
        contract_file: $contract_file,
        contract_version: $contract_version,
        grammar_name: $grammar_name,
        entry_rule: $entry_rule,
        done_rule: $done_rule,
        zero_plausible_grammar_level_gap_proof_surface: $zero_plausible_grammar_level_gap_proof_surface,
        unmet_proof_criteria_count: $unmet_proof_criteria_count,
        primary_unmet_proof_criterion: $primary_unmet_proof_criterion,
        unmet_proof_criteria: $unmet_proof_criteria,
        proof_surfaces: {
            syntax_closure_state_dir: $syntax_closure_state_dir,
            syntax_closure_summary_txt: $syntax_closure_summary_txt,
            syntax_gap_json: $syntax_gap_json,
            aggregate_contract_state_dir: $aggregate_contract_state_dir,
            aggregate_contract_summary_txt: $aggregate_contract_summary_txt,
            reachability_closure_state_dir: $reachability_closure_state_dir,
            reachability_closure_summary_txt: $reachability_closure_summary_txt
        },
        criteria: {
            syntax_preconditions_green: $syntax_preconditions_green,
            aggregate_preconditions_green: $aggregate_preconditions_green,
            reachability_preconditions_green: $reachability_preconditions_green,
            helper_only_unreachable_rules_green: $helper_only_unreachable_rules_green,
            helper_only_unreachable_branches_green: $helper_only_unreachable_branches_green,
            helper_only_unreachable_surface_green: $helper_only_unreachable_surface_green
        },
        expected: {
            allowed_unreachable_rules: $allowed_unreachable_rules,
            allowed_unreachable_branches: $allowed_unreachable_branches
        },
        observed: {
            unreachable_rules: $observed_unreachable_rules,
            unreachable_branches: $observed_unreachable_branches
        },
        classes: {
            default_profile: $default_profile,
            grammar_default_profile: $grammar_default_profile,
            syntax_contract_file: $syntax_contract_file,
            syntax_raw_ast_json: $syntax_raw_ast_json,
            dead: {
                green: ($dead_rules_green and $dead_branches_green),
                rules_green: $dead_rules_green,
                branches_green: $dead_branches_green,
                observed_rules: $observed_dead_rules,
                observed_branches: $observed_dead_branches
            },
            guard: {
                green: $guard_rules_green,
                expected: $expected_lookahead_only_rules,
                syntax_contract: $syntax_contract_lookahead_only_rules,
                observed: $observed_guard_rules
            },
            interior: {
                green: $interior_branches_green,
                expected: $expected_inside_lookahead_branches,
                syntax_contract: $syntax_contract_inside_lookahead_branches,
                observed: $observed_inside_lookahead_branches,
                outside_guards: $observed_inside_lookahead_branches_outside_guards
            },
            edition_gated: {
                green: $edition_gated_branches_green,
                expected: $expected_edition_gated_branches,
                observed_ids: $observed_edition_gated_branch_ids,
                verification: $edition_gated_verification
            },
            unclassified: {
                green: $unclassified_green,
                rules: $observed_unclassified_rules,
                branches: $observed_unclassified_branches
            }
        }
    }' >"$SUMMARY_JSON"

echo "SV preprocessor zero-plausible-gap proof gate completed."
