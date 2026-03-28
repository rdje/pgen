#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
BUNDLE_DIR="$ROOT_DIR/regex_corpus_bundle"

STATE_DIR="${PGEN_REGEX_CORPUS_BUNDLE_CONTRACT_STATE_DIR:-$RUST_DIR/target/regex_corpus_bundle_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

README_MD="$BUNDLE_DIR/README.md"
PLAN_MD="$BUNDLE_DIR/docs/regex_corpus_plan.md"
LOCKFILE="$BUNDLE_DIR/manifests/upstreams.lock.json"
LICENSES_FILE="$BUNDLE_DIR/manifests/licenses.json"
SCHEMA_FILE="$BUNDLE_DIR/schemas/regex_case.schema.json"
FETCH_SCRIPT="$BUNDLE_DIR/scripts/fetch_regex_corpora.py"
INVALID_README="$BUNDLE_DIR/corpus/pcre2/invalid/README.md"
QUARANTINE_README="$BUNDLE_DIR/corpus/pcre2/quarantine/README.md"
ORACLE_README="$BUNDLE_DIR/oracle/pcre2/README.md"

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

resolve_bundle_path() {
    local rel="$1"
    if [[ "$rel" == /* ]]; then
        printf '%s\n' "$rel"
    else
        printf '%s\n' "$BUNDLE_DIR/$rel"
    fi
}

bool_json() {
    if [[ "$1" == "true" ]]; then
        printf 'true\n'
    else
        printf 'false\n'
    fi
}

require_tool jq
require_tool python3

mkdir -p "$STATE_DIR" "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

require_file "$README_MD"
require_file "$PLAN_MD"
require_file "$LOCKFILE"
require_file "$LICENSES_FILE"
require_file "$SCHEMA_FILE"
require_file "$FETCH_SCRIPT"
require_file "$INVALID_README"
require_file "$QUARANTINE_README"
require_file "$ORACLE_README"

jq -e '
    (.lock_version == 1)
    and ((.created_for | type) == "string" and (.created_for | contains("PCRE2-flavor")))
    and (.upstreams | type == "array" and length == 2)
    and ((.upstreams | map(.name) | unique | length) == 2)
    and any(
        .upstreams[];
        .name == "pcre2"
        and .tier == "canonical"
        and .repo == "PCRE2Project/pcre2"
        and .ref == "pcre2-10.47"
        and .inventory.kind == "pcre2-testdata"
        and .inventory.output_json == "corpus/pcre2/canonical/pcre2_inventory.json"
        and .inventory.output_jsonl == "corpus/pcre2/canonical/pcre2_testfiles.jsonl"
        and .license_key == "pcre2"
    )
    and any(
        .upstreams[];
        .name == "php-src"
        and .tier == "secondary"
        and .repo == "php/php-src"
        and .ref == "php-8.4.19"
        and .inventory.kind == "php-phpt"
        and .inventory.output_json == "corpus/pcre2/php/php_inventory.json"
        and .inventory.output_jsonl == "corpus/pcre2/php/php_phpt_inventory.jsonl"
        and .license_key == "php"
    )
' "$LOCKFILE" >/dev/null

jq -e '
    .licenses.pcre2.source == "PCRE2Project/pcre2"
    and (.licenses.pcre2.summary | contains("testdata"))
    and (.licenses.pcre2.tracked_files | index("third_party/upstream/pcre2/pcre2-10.47/LICENCE.md"))
    and .licenses.php.source == "php/php-src"
    and (.licenses.php.summary | contains("PHP License"))
    and (.licenses.php.tracked_files | index("third_party/upstream/php-src/php-8.4.19/LICENSE"))
' "$LICENSES_FILE" >/dev/null

jq -e '
    .title == "Normalized PCRE2 regex test case"
    and .properties.flavor.const == "pcre2"
    and (.properties.tier.enum | index("canonical"))
    and (.properties.tier.enum | index("secondary"))
    and (.properties.tier.enum | index("derived-invalid"))
    and (.properties.tier.enum | index("quarantine"))
    and (.properties.wrapper.properties.kind.enum | index("php-preg"))
    and (.properties.wrapper.properties.kind.enum | index("pcre2test"))
    and (.properties.wrapper.properties.kind.enum | index("none"))
    and (.properties.expected.properties.parse.enum | index("ok"))
    and (.properties.expected.properties.parse.enum | index("error"))
    and (.properties.expected.properties.parse.enum | index("unknown"))
' "$SCHEMA_FILE" >/dev/null

run_logged "fetch_regex_corpora_help" python3 "$FETCH_SCRIPT" --help

pcre2_ref="$(jq -er '.upstreams[] | select(.name == "pcre2") | .ref' "$LOCKFILE")"
php_ref="$(jq -er '.upstreams[] | select(.name == "php-src") | .ref' "$LOCKFILE")"
pcre2_destination_rel="$(jq -er '.upstreams[] | select(.name == "pcre2") | .destination' "$LOCKFILE")"
php_destination_rel="$(jq -er '.upstreams[] | select(.name == "php-src") | .destination' "$LOCKFILE")"
pcre2_inventory_json_rel="$(jq -er '.upstreams[] | select(.name == "pcre2") | .inventory.output_json' "$LOCKFILE")"
pcre2_inventory_jsonl_rel="$(jq -er '.upstreams[] | select(.name == "pcre2") | .inventory.output_jsonl' "$LOCKFILE")"
php_inventory_json_rel="$(jq -er '.upstreams[] | select(.name == "php-src") | .inventory.output_json' "$LOCKFILE")"
php_inventory_jsonl_rel="$(jq -er '.upstreams[] | select(.name == "php-src") | .inventory.output_jsonl' "$LOCKFILE")"

pcre2_destination="$(resolve_bundle_path "$pcre2_destination_rel")"
php_destination="$(resolve_bundle_path "$php_destination_rel")"
pcre2_inventory_json="$(resolve_bundle_path "$pcre2_inventory_json_rel")"
pcre2_inventory_jsonl="$(resolve_bundle_path "$pcre2_inventory_jsonl_rel")"
php_inventory_json="$(resolve_bundle_path "$php_inventory_json_rel")"
php_inventory_jsonl="$(resolve_bundle_path "$php_inventory_jsonl_rel")"

pcre2_snapshot_present=false
php_snapshot_present=false
pcre2_inventory_json_present=false
pcre2_inventory_jsonl_present=false
php_inventory_json_present=false
php_inventory_jsonl_present=false

if [[ -d "$pcre2_destination" ]]; then
    pcre2_snapshot_present=true
fi
if [[ -d "$php_destination" ]]; then
    php_snapshot_present=true
fi
if [[ -f "$pcre2_inventory_json" ]]; then
    pcre2_inventory_json_present=true
    jq -e --arg ref "$pcre2_ref" '.source == "pcre2" and .ref == $ref' "$pcre2_inventory_json" >/dev/null
fi
if [[ -f "$pcre2_inventory_jsonl" ]]; then
    pcre2_inventory_jsonl_present=true
fi
if [[ -f "$php_inventory_json" ]]; then
    php_inventory_json_present=true
    jq -e --arg ref "$php_ref" '.source == "php-src" and .ref == $ref' "$php_inventory_json" >/dev/null
fi
if [[ -f "$php_inventory_jsonl" ]]; then
    php_inventory_jsonl_present=true
fi

pcre2_files_indexed="0"
php_files_indexed="0"
if [[ "$pcre2_inventory_json_present" == "true" ]]; then
    pcre2_files_indexed="$(jq -er '.files_indexed | numbers' "$pcre2_inventory_json")"
fi
if [[ "$php_inventory_json_present" == "true" ]]; then
    php_files_indexed="$(jq -er '.files_indexed | numbers' "$php_inventory_json")"
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat >"$SUMMARY_TXT" <<EOF
gate: regex_corpus_bundle_contract_gate
version: 1
generated_at_utc: $generated_at_utc
bundle_dir: $BUNDLE_DIR
lockfile: $LOCKFILE
schema_file: $SCHEMA_FILE
fetch_script: $FETCH_SCRIPT
pcre2_ref: $pcre2_ref
php_ref: $php_ref
pcre2_snapshot_present: $pcre2_snapshot_present
php_snapshot_present: $php_snapshot_present
pcre2_inventory_json_present: $pcre2_inventory_json_present
pcre2_inventory_jsonl_present: $pcre2_inventory_jsonl_present
php_inventory_json_present: $php_inventory_json_present
php_inventory_jsonl_present: $php_inventory_jsonl_present
pcre2_files_indexed: $pcre2_files_indexed
php_files_indexed: $php_files_indexed
current_role: maintained PCRE2-first acquisition/inventory starter for future regex hardening
status_effect: does_not_reopen_closed_regex_family_row
EOF

jq -n \
    --arg gate "regex_corpus_bundle_contract_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg bundle_dir "$BUNDLE_DIR" \
    --arg lockfile "$LOCKFILE" \
    --arg schema_file "$SCHEMA_FILE" \
    --arg fetch_script "$FETCH_SCRIPT" \
    --arg pcre2_ref "$pcre2_ref" \
    --arg php_ref "$php_ref" \
    --argjson pcre2_snapshot_present "$(bool_json "$pcre2_snapshot_present")" \
    --argjson php_snapshot_present "$(bool_json "$php_snapshot_present")" \
    --argjson pcre2_inventory_json_present "$(bool_json "$pcre2_inventory_json_present")" \
    --argjson pcre2_inventory_jsonl_present "$(bool_json "$pcre2_inventory_jsonl_present")" \
    --argjson php_inventory_json_present "$(bool_json "$php_inventory_json_present")" \
    --argjson php_inventory_jsonl_present "$(bool_json "$php_inventory_jsonl_present")" \
    --argjson pcre2_files_indexed "$pcre2_files_indexed" \
    --argjson php_files_indexed "$php_files_indexed" \
    '{
        gate: $gate,
        version: $version,
        generated_at_utc: $generated_at_utc,
        bundle_dir: $bundle_dir,
        lockfile: $lockfile,
        schema_file: $schema_file,
        fetch_script: $fetch_script,
        current_role: "maintained PCRE2-first acquisition/inventory starter for future regex hardening",
        status_effect: "does_not_reopen_closed_regex_family_row",
        upstreams: {
            pcre2: {
                ref: $pcre2_ref,
                snapshot_present: $pcre2_snapshot_present,
                inventory_json_present: $pcre2_inventory_json_present,
                inventory_jsonl_present: $pcre2_inventory_jsonl_present,
                files_indexed: $pcre2_files_indexed
            },
            php_src: {
                ref: $php_ref,
                snapshot_present: $php_snapshot_present,
                inventory_json_present: $php_inventory_json_present,
                inventory_jsonl_present: $php_inventory_jsonl_present,
                files_indexed: $php_files_indexed
            }
        }
    }' >"$SUMMARY_JSON"

echo "✅ regex corpus bundle contract gate passed."
echo "Summary TXT: $SUMMARY_TXT"
echo "Summary JSON: $SUMMARY_JSON"
