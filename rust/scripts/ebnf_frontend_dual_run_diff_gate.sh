#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"
STATE_DIR="${PGEN_EBNF_DUAL_RUN_STATE_DIR:-$RUST_DIR/target/ebnf_frontend_dual_run_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

STRICT="${PGEN_EBNF_DUAL_RUN_STRICT:-0}"
GRAMMARS=("ebnf" "json" "regex")

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
RUST_DIFF_BIN="$RUST_DIR/target/debug/ebnf_dual_run_diff"
BOOTSTRAP_EBNF_JSON="$WORK_DIR/bootstrap_ebnf.json"
BOOTSTRAP_EBNF_PARSER_RS="$WORK_DIR/bootstrap_ebnf.rs"

if ! [[ "$STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_EBNF_DUAL_RUN_STRICT must be 0 or 1" >&2
    exit 2
fi

mkdir -p "$STATE_DIR" "$LOG_DIR" "$WORK_DIR"

echo "==> Building ast_pipeline (ebnf_dual_run path)"
(cd "$RUST_DIR" && cargo build --features ebnf_dual_run --bin ast_pipeline >/dev/null)

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary missing at '$AST_PIPELINE_BIN'" >&2
    exit 1
fi

echo "==> Regenerating EBNF frontend artifacts for dual-run harness"
perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$GRAMMARS_DIR/ebnf.ebnf" -o "$BOOTSTRAP_EBNF_JSON" \
    >"$LOG_DIR/bootstrap_ebnf_to_json.log" 2>&1
"$AST_PIPELINE_BIN" "$BOOTSTRAP_EBNF_JSON" --generate-parser --output "$BOOTSTRAP_EBNF_PARSER_RS" \
    >"$LOG_DIR/bootstrap_generate_ebnf_parser.log" 2>&1

echo "==> Building Rust dual-run report binary"
(cd "$RUST_DIR" && PGEN_EBNF_PARSER_PATH="$BOOTSTRAP_EBNF_PARSER_RS" cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff >/dev/null)

if [[ ! -x "$RUST_DIFF_BIN" ]]; then
    echo "error: Rust dual-run binary missing at '$RUST_DIFF_BIN'" >&2
    exit 1
fi

echo "grammar,perl_ebnf_to_json,rust_parse,rust_parse_full,perl_rule_count,rust_rule_count,raw_ast_status,raw_ast_missing_on_perl_count,raw_ast_missing_on_rust_count,parse_end,input_bytes,consumed_pct,overall,notes" >"$SUMMARY_CSV"
echo "[]" >"$SUMMARY_JSON"

{
    echo "PGEN EBNF Frontend Dual-Run Differential Summary"
    echo "state_dir: $STATE_DIR"
    echo "strict_mode: $STRICT"
    echo "bootstrap_ebnf_json: $BOOTSTRAP_EBNF_JSON"
    echo "bootstrap_ebnf_parser: $BOOTSTRAP_EBNF_PARSER_RS"
    echo
} >"$SUMMARY_TXT"

failures=0
any_internal_errors=0

for grammar in "${GRAMMARS[@]}"; do
    grammar_file="$GRAMMARS_DIR/${grammar}.ebnf"
    perl_json="$WORK_DIR/${grammar}.perl_raw_ast.json"
    rust_json="$WORK_DIR/${grammar}.rust_parse_report.json"
    rust_raw_ast_json="$WORK_DIR/${grammar}.rust_raw_ast.json"
    raw_ast_compare_json="$WORK_DIR/${grammar}.raw_ast_compare.json"
    diff_json="$WORK_DIR/${grammar}.dual_run_diff.json"

    perl_log="$LOG_DIR/${grammar}.perl_ebnf_to_json.log"
    rust_log="$LOG_DIR/${grammar}.rust_parse.log"
    rust_raw_ast_log="$LOG_DIR/${grammar}.rust_raw_ast.log"

    perl_status="fail"
    rust_parse="fail"
    rust_parse_full="fail"
    perl_rule_count="-"
    rust_rule_count="-"
    raw_ast_status="skip"
    raw_ast_missing_on_perl_count="-"
    raw_ast_missing_on_rust_count="-"
    raw_ast_missing_on_perl_names="-"
    raw_ast_missing_on_rust_names="-"
    parse_end="-"
    input_bytes="-"
    consumed_pct="-"
    overall="fail"
    notes="internal error"

    if perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$grammar_file" -o "$perl_json" >"$perl_log" 2>&1; then
        perl_status="pass"
    else
        notes="perl ebnf_to_json failed (see logs/${grammar}.perl_ebnf_to_json.log)"
        failures=$((failures + 1))
    fi

    if "$RUST_DIFF_BIN" --input "$grammar_file" --output "$rust_json" >"$rust_log" 2>&1; then
        rust_parse="$(python3 - "$rust_json" <<'PY'
import json,sys
path=sys.argv[1]
data=json.load(open(path))
print("pass" if data.get("parse",{}).get("ok") else "fail")
PY
)"
        rust_parse_full="$(python3 - "$rust_json" <<'PY'
import json,sys
path=sys.argv[1]
data=json.load(open(path))
print("pass" if data.get("parse_full",{}).get("ok") else "fail")
PY
)"
        parse_end="$(python3 - "$rust_json" <<'PY'
import json,sys
path=sys.argv[1]
data=json.load(open(path))
end=data.get("parse",{}).get("span_end")
print(end if end is not None else "-")
PY
)"
        input_bytes="$(python3 - "$rust_json" <<'PY'
import json,sys
path=sys.argv[1]
data=json.load(open(path))
print(data.get("input_bytes","-"))
PY
)"
        consumed_pct="$(python3 - "$rust_json" <<'PY'
import json,sys
path=sys.argv[1]
data=json.load(open(path))
end=data.get("parse",{}).get("span_end")
size=data.get("input_bytes")
if isinstance(end,int) and isinstance(size,int) and size>0:
    print(f"{(end*100.0)/size:.2f}")
else:
    print("-")
PY
)"
    else
        notes="rust parser run failed (see logs/${grammar}.rust_parse.log)"
        any_internal_errors=1
        failures=$((failures + 1))
    fi

    if "$AST_PIPELINE_BIN" "$grammar_file" --emit-raw-ast-json "$rust_raw_ast_json" >"$rust_raw_ast_log" 2>&1; then
        if [[ "$perl_status" == "pass" ]]; then
            raw_ast_fields="$(python3 - "$perl_json" "$rust_raw_ast_json" "$raw_ast_compare_json" <<'PY'
import json
import sys

perl_path, rust_path, out_path = sys.argv[1:]

def rule_names(payload):
    raw_ast = payload.get("raw_ast", [])
    if not isinstance(raw_ast, list):
        raise SystemExit("raw_ast payload is not a list")
    names = []
    for rule in raw_ast:
        if not isinstance(rule, list) or not rule:
            continue
        head = rule[0]
        if (
            isinstance(head, list)
            and len(head) >= 2
            and head[0] == "rule"
            and isinstance(head[1], str)
        ):
            names.append(head[1])
    return names

with open(perl_path) as fh:
    perl_payload = json.load(fh)
with open(rust_path) as fh:
    rust_payload = json.load(fh)

perl_names = rule_names(perl_payload)
rust_names = rule_names(rust_payload)
perl_unique = sorted(set(perl_names))
rust_unique = sorted(set(rust_names))

missing_on_perl = [name for name in rust_unique if name not in perl_unique]
missing_on_rust = [name for name in perl_unique if name not in rust_unique]

if not missing_on_perl and not missing_on_rust:
    status = "parity"
elif missing_on_perl and not missing_on_rust:
    status = "perl_under_reports"
elif missing_on_rust and not missing_on_perl:
    status = "rust_under_reports"
else:
    status = "divergent"

comparison = {
    "status": status,
    "perl_rule_count": len(perl_names),
    "rust_rule_count": len(rust_names),
    "perl_unique_rule_count": len(perl_unique),
    "rust_unique_rule_count": len(rust_unique),
    "missing_on_perl_count": len(missing_on_perl),
    "missing_on_perl_names": missing_on_perl,
    "missing_on_rust_count": len(missing_on_rust),
    "missing_on_rust_names": missing_on_rust,
}

with open(out_path, "w") as fh:
    json.dump(comparison, fh, indent=2, sort_keys=True)
    fh.write("\n")

print(
    "\t".join(
        [
            str(len(perl_names)),
            str(len(rust_names)),
            status,
            str(len(missing_on_perl)),
            str(len(missing_on_rust)),
            "|".join(missing_on_perl) if missing_on_perl else "-",
            "|".join(missing_on_rust) if missing_on_rust else "-",
        ]
    )
)
PY
)"
            IFS=$'\t' read -r perl_rule_count rust_rule_count raw_ast_status raw_ast_missing_on_perl_count raw_ast_missing_on_rust_count raw_ast_missing_on_perl_names raw_ast_missing_on_rust_names <<<"$raw_ast_fields"
        fi
    else
        notes="rust raw_ast export failed (see logs/${grammar}.rust_raw_ast.log)"
        any_internal_errors=1
        failures=$((failures + 1))
    fi

    if [[ "$perl_status" == "pass" && "$rust_parse" == "pass" && "$rust_parse_full" == "pass" && ( "$raw_ast_status" == "parity" || "$raw_ast_status" == "perl_under_reports" ) ]]; then
        overall="pass"
        if [[ "$raw_ast_status" == "perl_under_reports" ]]; then
            notes="full parse parity; perl raw_ast under-reports ${raw_ast_missing_on_perl_count} unique rule(s)"
        else
            notes="full parse parity; raw_ast parity"
        fi
    else
        if [[ "$notes" == "internal error" ]]; then
            if [[ "$raw_ast_status" == "rust_under_reports" || "$raw_ast_status" == "divergent" ]]; then
                notes="unexpected raw_ast divergence (see work/${grammar}.raw_ast_compare.json)"
            else
                notes="perl/rust parity mismatch"
            fi
        fi
        overall="fail"
        failures=$((failures + 1))
    fi

    python3 - "$grammar" "$perl_status" "$rust_parse" "$rust_parse_full" "$overall" "$notes" "$perl_rule_count" "$rust_rule_count" "$raw_ast_status" "$raw_ast_missing_on_perl_count" "$raw_ast_missing_on_rust_count" "$parse_end" "$input_bytes" "$consumed_pct" "$perl_json" "$rust_json" "$rust_raw_ast_json" "$raw_ast_compare_json" "$diff_json" <<'PY'
import json,sys,os
(
    grammar, perl_status, rust_parse, rust_parse_full, overall, notes,
    perl_rule_count, rust_rule_count, raw_ast_status,
    raw_ast_missing_on_perl_count, raw_ast_missing_on_rust_count,
    parse_end, input_bytes, consumed_pct,
    perl_json_path, rust_json_path, rust_raw_ast_json_path, raw_ast_compare_json_path, out_path
) = sys.argv[1:]

payload = {
    "grammar": grammar,
    "perl_ebnf_to_json": perl_status,
    "rust_parse": rust_parse,
    "rust_parse_full": rust_parse_full,
    "overall": overall,
    "notes": notes,
    "perl_rule_count": None if perl_rule_count == "-" else int(perl_rule_count),
    "rust_rule_count": None if rust_rule_count == "-" else int(rust_rule_count),
    "raw_ast_status": raw_ast_status,
    "raw_ast_missing_on_perl_count": None if raw_ast_missing_on_perl_count == "-" else int(raw_ast_missing_on_perl_count),
    "raw_ast_missing_on_rust_count": None if raw_ast_missing_on_rust_count == "-" else int(raw_ast_missing_on_rust_count),
    "rust_parse_end": None if parse_end == "-" else int(parse_end),
    "input_bytes": None if input_bytes == "-" else int(input_bytes),
    "consumed_pct": None if consumed_pct == "-" else float(consumed_pct),
    "artifacts": {
        "perl_json": perl_json_path if os.path.exists(perl_json_path) else None,
        "rust_json": rust_json_path if os.path.exists(rust_json_path) else None,
        "rust_raw_ast_json": rust_raw_ast_json_path if os.path.exists(rust_raw_ast_json_path) else None,
        "raw_ast_compare_json": raw_ast_compare_json_path if os.path.exists(raw_ast_compare_json_path) else None,
    },
}

if os.path.exists(rust_json_path):
    payload["rust_report"] = json.load(open(rust_json_path))
if os.path.exists(perl_json_path):
    payload["perl_report"] = json.load(open(perl_json_path))
if os.path.exists(raw_ast_compare_json_path):
    payload["raw_ast_comparison"] = json.load(open(raw_ast_compare_json_path))
if os.path.exists(rust_raw_ast_json_path):
    payload["rust_raw_ast_report"] = json.load(open(rust_raw_ast_json_path))

with open(out_path, "w") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

    echo "${grammar},${perl_status},${rust_parse},${rust_parse_full},${perl_rule_count},${rust_rule_count},${raw_ast_status},${raw_ast_missing_on_perl_count},${raw_ast_missing_on_rust_count},${parse_end},${input_bytes},${consumed_pct},${overall},${notes}" >>"$SUMMARY_CSV"
done

python3 - "$WORK_DIR" "$SUMMARY_JSON" <<'PY'
import json,glob,os,sys
work_dir, out_json = sys.argv[1], sys.argv[2]
rows = []
for path in sorted(glob.glob(os.path.join(work_dir, "*.dual_run_diff.json"))):
    rows.append(json.load(open(path)))
with open(out_json, "w") as fh:
    json.dump({"entries": rows}, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

{
    echo "Results:"
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
    echo
    echo "Summary JSON: $SUMMARY_JSON"
    echo "Logs: $LOG_DIR"
    echo "Artifacts: $WORK_DIR"
} >>"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if [[ "$any_internal_errors" -ne 0 ]]; then
    echo "❌ Internal dual-run harness error(s) detected." >&2
    exit 1
fi

if [[ "$failures" -ne 0 ]]; then
    echo "⚠️  EBNF dual-run differential has $failures failing grammar flow(s)." >&2
    if [[ "$STRICT" -eq 1 ]]; then
        echo "❌ strict mode enabled: failing." >&2
        exit 1
    fi
    echo "ℹ️  strict mode disabled: report-only mode." >&2
else
    echo "✅ EBNF dual-run differential passed for all tracked grammars."
fi
