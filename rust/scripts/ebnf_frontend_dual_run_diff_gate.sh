#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"
GENERATED_DIR="$ROOT_DIR/generated"

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
EBNF_JSON="$GENERATED_DIR/ebnf.json"
EBNF_PARSER_RS="$GENERATED_DIR/ebnf.rs"

if ! [[ "$STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_EBNF_DUAL_RUN_STRICT must be 0 or 1" >&2
    exit 2
fi

mkdir -p "$STATE_DIR" "$LOG_DIR" "$WORK_DIR" "$GENERATED_DIR"

echo "==> Building ast_pipeline (non-bootstrap path)"
(cd "$RUST_DIR" && cargo build --bin ast_pipeline >/dev/null)

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary missing at '$AST_PIPELINE_BIN'" >&2
    exit 1
fi

echo "==> Regenerating EBNF frontend artifacts for dual-run harness"
perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$GRAMMARS_DIR/ebnf.ebnf" -o "$EBNF_JSON" \
    >"$LOG_DIR/bootstrap_ebnf_to_json.log" 2>&1
"$AST_PIPELINE_BIN" "$EBNF_JSON" --generate-parser --output "$EBNF_PARSER_RS" \
    >"$LOG_DIR/bootstrap_generate_ebnf_parser.log" 2>&1

echo "==> Building Rust dual-run report binary"
(cd "$RUST_DIR" && cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff >/dev/null)

if [[ ! -x "$RUST_DIFF_BIN" ]]; then
    echo "error: Rust dual-run binary missing at '$RUST_DIFF_BIN'" >&2
    exit 1
fi

echo "grammar,perl_ebnf_to_json,rust_parse,rust_parse_full,perl_rule_count,parse_end,input_bytes,consumed_pct,overall,notes" >"$SUMMARY_CSV"
echo "[]" >"$SUMMARY_JSON"

{
    echo "PGEN EBNF Frontend Dual-Run Differential Summary"
    echo "state_dir: $STATE_DIR"
    echo "strict_mode: $STRICT"
    echo "generated_ebnf_json: $EBNF_JSON"
    echo "generated_ebnf_parser: $EBNF_PARSER_RS"
    echo
} >"$SUMMARY_TXT"

failures=0
any_internal_errors=0

for grammar in "${GRAMMARS[@]}"; do
    grammar_file="$GRAMMARS_DIR/${grammar}.ebnf"
    perl_json="$WORK_DIR/${grammar}.perl_raw_ast.json"
    rust_json="$WORK_DIR/${grammar}.rust_parse_report.json"
    diff_json="$WORK_DIR/${grammar}.dual_run_diff.json"

    perl_log="$LOG_DIR/${grammar}.perl_ebnf_to_json.log"
    rust_log="$LOG_DIR/${grammar}.rust_parse.log"

    perl_status="fail"
    rust_parse="fail"
    rust_parse_full="fail"
    perl_rule_count="-"
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

    if [[ "$perl_status" == "pass" ]]; then
        perl_rule_count="$(python3 - "$perl_json" <<'PY'
import json,sys
path=sys.argv[1]
data=json.load(open(path))
raw=data.get("raw_ast",[])
print(len(raw) if isinstance(raw,list) else "-")
PY
)"
    fi

    if [[ "$perl_status" == "pass" && "$rust_parse" == "pass" && "$rust_parse_full" == "pass" ]]; then
        overall="pass"
        notes="full parse parity"
    else
        if [[ "$notes" == "internal error" ]]; then
            notes="perl/rust parity mismatch"
        fi
        overall="fail"
        failures=$((failures + 1))
    fi

    python3 - "$grammar" "$perl_status" "$rust_parse" "$rust_parse_full" "$overall" "$notes" "$perl_rule_count" "$parse_end" "$input_bytes" "$consumed_pct" "$perl_json" "$rust_json" "$diff_json" <<'PY'
import json,sys,os
(
    grammar, perl_status, rust_parse, rust_parse_full, overall, notes,
    perl_rule_count, parse_end, input_bytes, consumed_pct,
    perl_json_path, rust_json_path, out_path
) = sys.argv[1:]

payload = {
    "grammar": grammar,
    "perl_ebnf_to_json": perl_status,
    "rust_parse": rust_parse,
    "rust_parse_full": rust_parse_full,
    "overall": overall,
    "notes": notes,
    "perl_rule_count": None if perl_rule_count == "-" else int(perl_rule_count),
    "rust_parse_end": None if parse_end == "-" else int(parse_end),
    "input_bytes": None if input_bytes == "-" else int(input_bytes),
    "consumed_pct": None if consumed_pct == "-" else float(consumed_pct),
    "artifacts": {
        "perl_json": perl_json_path if os.path.exists(perl_json_path) else None,
        "rust_json": rust_json_path if os.path.exists(rust_json_path) else None,
    },
}

if os.path.exists(rust_json_path):
    payload["rust_report"] = json.load(open(rust_json_path))
if os.path.exists(perl_json_path):
    payload["perl_report"] = json.load(open(perl_json_path))

with open(out_path, "w") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

    echo "${grammar},${perl_status},${rust_parse},${rust_parse_full},${perl_rule_count},${parse_end},${input_bytes},${consumed_pct},${overall},${notes}" >>"$SUMMARY_CSV"
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
