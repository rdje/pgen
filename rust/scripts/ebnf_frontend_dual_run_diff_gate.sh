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

# LANG-CAPABILITY-AUDIT.10.6 — the PERL arm is RETIRED.
#
# This gate was born as a Perl->Rust migration oracle: it diffed `tools/ebnf_to_json.pl`
# against the hand-written Rust frontend. That migration is COMPLETE, and the arm had
# decayed into a liability — measured, it was blind to 25 of regex.ebnf's 276 rules (the
# whole modern `code_*` family) and the gate PASSED that as `perl_under_reports`. It also
# compared `sorted(set(rule_names))` only, so two frontends could tokenize every rule body
# differently and still report `parity`.
#
# The "dual run" is now the duality that actually matters and the name is finally accurate:
#   arm 1 — the hand-written Rust frontend (`ast_pipeline --emit-raw-ast-json`)
#   arm 2 — the parser GENERATED from grammars/ebnf.ebnf (`ebnf_dual_run_diff`)
# Arm 2's verdict is the SELF-HOSTING measurement, and it is what this gate asserts.
#
# ⚠️ HONEST BOUND, stated rather than implied: retiring the Perl arm removes this gate's
# only OUTPUT-level comparison. Arm 1 yields a raw-AST envelope and arm 2 yields a verdict,
# so they are not directly diffable yet. Building that raw-AST differential — the evidence
# a frontend REPLACEMENT would need — is the remaining half of LANG-CAPABILITY-AUDIT.10.6.

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
RUST_DIFF_BIN="$RUST_DIR/target/debug/ebnf_dual_run_diff"
BOOTSTRAP_EBNF_JSON="$WORK_DIR/bootstrap_ebnf.json"
BOOTSTRAP_EBNF_PARSER_RS="$WORK_DIR/bootstrap_ebnf.rs"

if ! [[ "$STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_EBNF_DUAL_RUN_STRICT must be 0 or 1" >&2
    exit 2
fi

mkdir -p "$STATE_DIR" "$LOG_DIR" "$WORK_DIR"

print_log_excerpt_on_failure() {
    local label="$1"
    local log_path="$2"
    local status="$3"
    local max_head_lines="${4:-120}"
    local max_tail_lines="${5:-40}"

    echo "error: ${label} failed with exit code ${status}" >&2
    echo "log: ${log_path}" >&2
    if [[ ! -f "$log_path" ]]; then
        echo "(log file missing)" >&2
        return
    fi
    if [[ ! -s "$log_path" ]]; then
        echo "(log file is empty)" >&2
        return
    fi

    local total_lines
    total_lines=$(wc -l <"$log_path" | tr -d ' ')
    echo "--- begin ${label} log ---" >&2
    if [[ "$total_lines" -le "$max_head_lines" ]]; then
        cat "$log_path" >&2
    else
        sed -n "1,${max_head_lines}p" "$log_path" >&2
        echo "--- log truncated; showing last ${max_tail_lines} lines of ${total_lines} ---" >&2
        tail -n "$max_tail_lines" "$log_path" >&2
    fi
    echo "--- end ${label} log ---" >&2
}

run_logged_or_dump() {
    local label="$1"
    local log_path="$2"
    shift 2

    # `set -e` is active for this script, so an UNGUARDED `"$@" >log 2>&1`
    # aborts the whole script AT the failing command: every line below it —
    # including the failure excerpt the 2026-04-06 CI-observability fix added —
    # would never run, and a failing bootstrap step would surface as a bare
    # `make ... Error 1` with the real stderr still hidden in the log file.
    # Capturing the status with `|| status=$?` keeps errexit from firing here;
    # the real exit code is still returned, so the caller still fails fast.
    local status=0
    "$@" >"$log_path" 2>&1 || status=$?
    if [[ "$status" -eq 0 ]]; then
        return 0
    fi

    print_log_excerpt_on_failure "$label" "$log_path" "$status"
    return "$status"
}

# BIN-BUILD-INTEGRITY.3 — the CANONICAL annotation backend, deliberately.
# This binary does three jobs below: it exports the raw AST of `grammars/ebnf.ebnf`
# (the bootstrap seed), generates a parser from that JSON, and exports each tracked
# grammar's raw AST. Since commit 200cae5b an `ebnf_dual_run`-ONLY binary hard-REFUSES
# the generation job, because without `generated_parsers` annotation parsing would
# silently fall back to the hand-rolled bootstrap subset. Building with BOTH features is
# the right answer rather than opting into that fallback: this is a differential harness,
# so the ONE thing it may vary is the arm under test — a degradable annotation backend
# would be a second, uncontrolled variable, and the fallback's own license only covers
# artifacts that are re-derived canonically before being trusted, which a standing gate
# cannot do. It is also the feature set TOOLBOX.md documents for this shared binary path,
# so running this gate no longer replaces the toolbox's dual-feature `ast_pipeline` with
# a single-feature one.
# ⭐ LANG-CAPABILITY-AUDIT.10.6 — building with `ebnf_dual_run` is ALSO what let the Perl
# bootstrap step retire: this binary can now read `.ebnf` directly, so nothing here needs
# a second frontend implementation just to get started.
echo "==> Building ast_pipeline (generated_parsers + ebnf_dual_run path)"
(cd "$RUST_DIR" && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline >/dev/null)

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary missing at '$AST_PIPELINE_BIN'" >&2
    exit 1
fi

echo "==> Regenerating EBNF frontend artifacts for dual-run harness"
# The binary above is built with BOTH features, so it reads `.ebnf` directly and the Perl
# bootstrap step is unnecessary. (Even on a cold clone this holds: the generated-parser
# cross-check is cfg-gated on `has_generated_ebnf_parser`, so `--features ebnf_dual_run`
# compiles and runs with no `generated/ebnf.rs` present — the same chicken-and-egg breaker
# `rust/Makefile`'s own `regex_parser_bootstrap` relies on.)
run_logged_or_dump \
    "bootstrap EBNF raw-AST export (Rust frontend)" \
    "$LOG_DIR/bootstrap_ebnf_raw_ast.log" \
    "$AST_PIPELINE_BIN" "$GRAMMARS_DIR/ebnf.ebnf" --emit-raw-ast-json "$BOOTSTRAP_EBNF_JSON"
run_logged_or_dump \
    "bootstrap EBNF parser generation" \
    "$LOG_DIR/bootstrap_generate_ebnf_parser.log" \
    "$AST_PIPELINE_BIN" "$BOOTSTRAP_EBNF_JSON" --generate-parser --output "$BOOTSTRAP_EBNF_PARSER_RS"

echo "==> Building Rust dual-run report binary"
(cd "$RUST_DIR" && PGEN_EBNF_PARSER_PATH="$BOOTSTRAP_EBNF_PARSER_RS" cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff >/dev/null)

if [[ ! -x "$RUST_DIFF_BIN" ]]; then
    echo "error: Rust dual-run binary missing at '$RUST_DIFF_BIN'" >&2
    exit 1
fi

echo "grammar,rust_parse,rust_parse_full,rust_rule_count,raw_ast_status,parse_end,input_bytes,consumed_pct,overall,notes" >"$SUMMARY_CSV"
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
    rust_json="$WORK_DIR/${grammar}.rust_parse_report.json"
    rust_raw_ast_json="$WORK_DIR/${grammar}.rust_raw_ast.json"
    diff_json="$WORK_DIR/${grammar}.dual_run_diff.json"

    rust_log="$LOG_DIR/${grammar}.rust_parse.log"
    rust_raw_ast_log="$LOG_DIR/${grammar}.rust_raw_ast.log"

    rust_parse="fail"
    rust_parse_full="fail"
    rust_rule_count="-"
    raw_ast_status="skip"
    parse_end="-"
    input_bytes="-"
    consumed_pct="-"
    overall="fail"
    notes="internal error"

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

    # Arm 1 - the hand-written Rust frontend's raw-AST envelope. With the Perl arm retired
    # there is no second envelope to diff against yet, so this records the rule count as an
    # artifact and asserts only that the export SUCCEEDS. The frontend<->meta-parser envelope
    # differential is the remaining half of LANG-CAPABILITY-AUDIT.10.6.
    if "$AST_PIPELINE_BIN" "$grammar_file" --emit-raw-ast-json "$rust_raw_ast_json" >"$rust_raw_ast_log" 2>&1; then
        rust_rule_count="$(python3 - "$rust_raw_ast_json" <<'RAWAST'
import json,sys
payload = json.load(open(sys.argv[1]))
names = []
for rule in payload.get("raw_ast", []):
    if isinstance(rule, list) and rule:
        head = rule[0]
        if isinstance(head, list) and len(head) >= 2 and head[0] == "rule" and isinstance(head[1], str):
            names.append(head[1])
print(len(names))
RAWAST
)"
        raw_ast_status="exported"
    else
        notes="rust raw_ast export failed (see logs/${grammar}.rust_raw_ast.log)"
        any_internal_errors=1
        failures=$((failures + 1))
    fi

    # The self-hosting assertion: the GENERATED meta-parser must fully consume the grammar,
    # and the hand-written frontend must export its envelope. (Before .10.6 this also
    # required Perl parity, and TOLERATED `perl_under_reports` - a pass that accepted the
    # Perl arm being blind to 25 of regex.ebnf's 276 rules.)
    if [[ "$rust_parse" == "pass" && "$rust_parse_full" == "pass" && "$raw_ast_status" == "exported" ]]; then
        overall="pass"
        notes="self-hosting: generated parser fully consumed the grammar; frontend envelope exported (${rust_rule_count} rules)"
    else
        if [[ "$notes" == "internal error" ]]; then
            notes="generated meta-parser did not fully consume the grammar"
        fi
        overall="fail"
        failures=$((failures + 1))
    fi

    python3 - "$grammar" "$rust_parse" "$rust_parse_full" "$overall" "$notes" "$rust_rule_count" "$raw_ast_status" "$parse_end" "$input_bytes" "$consumed_pct" "$rust_json" "$rust_raw_ast_json" "$diff_json" <<'ENTRY'
import json,sys,os
(
    grammar, rust_parse, rust_parse_full, overall, notes,
    rust_rule_count, raw_ast_status,
    parse_end, input_bytes, consumed_pct,
    rust_json_path, rust_raw_ast_json_path, out_path
) = sys.argv[1:]

payload = {
    "grammar": grammar,
    "rust_parse": rust_parse,
    "rust_parse_full": rust_parse_full,
    "overall": overall,
    "notes": notes,
    "rust_rule_count": None if rust_rule_count == "-" else int(rust_rule_count),
    "raw_ast_status": raw_ast_status,
    "rust_parse_end": None if parse_end == "-" else int(parse_end),
    "input_bytes": None if input_bytes == "-" else int(input_bytes),
    "consumed_pct": None if consumed_pct == "-" else float(consumed_pct),
    "artifacts": {
        "rust_json": rust_json_path if os.path.exists(rust_json_path) else None,
        "rust_raw_ast_json": rust_raw_ast_json_path if os.path.exists(rust_raw_ast_json_path) else None,
    },
}

if os.path.exists(rust_json_path):
    payload["rust_report"] = json.load(open(rust_json_path))

with open(out_path, "w") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
ENTRY

    echo "${grammar},${rust_parse},${rust_parse_full},${rust_rule_count},${raw_ast_status},${parse_end},${input_bytes},${consumed_pct},${overall},${notes}" >>"$SUMMARY_CSV"
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
