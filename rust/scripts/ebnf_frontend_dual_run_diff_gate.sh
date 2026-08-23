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

# LANG-CAPABILITY-AUDIT.10.6 part 2 — widened from ("ebnf" "json" "regex") to every tracked
# grammar the generated meta-parser fully consumes. Measured before widening: all 14 already
# passed the verdict assertions this gate has always made, and the whole 14-grammar envelope
# differential costs 3.8 s. Covering 3 of 14 was leaving measurement on the table.
# ⚠️ The three `*_lrm_extracted` grammars are deliberately absent: the hand-written frontend
# (arm 1) rejects them, so there is no pair to diff.
GRAMMARS=(
    "builtin_return_annotation"
    "builtin_semantic_annotation"
    "ebnf"
    "json"
    "regex"
    "return_annotation"
    "rtl_const_expr"
    "rtl_frontend"
    "semantic_annotation"
    "systemverilog"
    "systemverilog_lrm_profiled_generated"
    "systemverilog_lrm_profiled_wrapper"
    "systemverilog_preprocessor"
    "vhdl"
)

# LANG-CAPABILITY-AUDIT.10.6 part 2 — the ENVELOPE RATCHET.
#
# The differential's first run found real, located divergences between the hand-written
# frontend and the parser generated from `grammars/ebnf.ebnf`. They are defects in
# `ebnf.ebnf`, they are ROUTED to their own leaves, and they are NOT this gate's to fix — but
# they must not be allowed to grow, and the six grammars that ARE envelope-equivalent must not
# be allowed to lose it.
#
# So each grammar carries a labelled ceiling instead of a blanket pass. A count ABOVE the
# ceiling fails the gate; a count BELOW it fails too, with a message to lower the ceiling —
# a floor that can only be met and never tightened is the exact failure `.10.9` had to
# replace, and it is not being reintroduced here.
#
# ⛔ A ceiling is LOWERED as the owning leaf lands its fix. It is never RAISED to land a change.
envelope_divergence_ceiling() {
    case "$1" in
        # ✅ ENVELOPE-EQUIVALENT — the meta-parser reproduces the hand-written frontend's
        # envelope exactly. `ebnf` itself is in this set: the meta-grammar is self-hosting at
        # OUTPUT level, not merely at verdict level.
        builtin_return_annotation) echo 0 ;;
        builtin_semantic_annotation) echo 0 ;;
        ebnf) echo 0 ;;
        rtl_const_expr) echo 0 ;;
        rtl_frontend) echo 0 ;;
        vhdl) echo 0 ;;
        # LANG-CAPABILITY-AUDIT.10.13 — `regex_flags := /([gimsuyx]*)/` matches ACROSS trivia
        # and eats the leading `[gimsuyx]` run of the next token (`members` → `embers`).
        json) echo 2 ;;
        semantic_annotation) echo 6 ;;
        # .10.13 plus LANG-CAPABILITY-AUDIT.10.14 — a leading `@annotation` binds to the
        # PREVIOUS rule's expression instead of the rule it precedes.
        return_annotation) echo 6 ;;
        regex) echo 38 ;;
        # SV-CORPUS-GRAD.13c.2i — 150 -> 151, and the extra divergence is NAMED rather than bumped.
        # The ceiling was set at 150/13 880 tokens on 2026-07-31; `019e1739` (2026-08-09,
        # SV-CORPUS-GRAD.3.20) then added `use_clause_param_override_sv_only` carrying an INLINE
        # `@probe_sample:`, and arm 1 emits `semantic_annotation_inline` where arm 2's projection
        # emits `semantic_annotation` — the SAME blind spot already named on the svpp row below,
        # whose population here is 37. ⇒ one more instance of an accepted asymmetry, not a new
        # fidelity loss. ⛔ It was invisible for ~2.5 weeks because the report's divergence list is
        # capped at 40 and this one sits past it; `PGEN_ENVELOPE_DUMP_ALL=1` now lifts that cap,
        # which is how the row above was identified at all.
        #
        # GRAMMAR-WELLFORMED.H.20 — 151 -> 155, and all FOUR extra divergences are NAMED,
        # ATTRIBUTED to an exact commit, and each MECHANISM proven by a surgical one-edit control.
        # Nothing is being landed by this raise: all ten commits that touched
        # `grammars/systemverilog.ebnf` between 2026-08-18 and 2026-08-21 shipped under their own
        # leaves and their own gates, and this is the ADJUDICATION of already-shipped work — the
        # same shape as the .13c.2i entry above. The rows, from a per-vintage census run with the
        # binary PINNED (so arm 2 could not vary):
        #
        #   `4a2703cf` (.13c.2o) 151 -> 152  covergroup_declaration_sv_2023
        #        return_scalar -> return_object. ⛔ NOT a projection asymmetry — a LIVE frontend
        #        defect: `rust/src/ebnf_frontend.rs:713` extract_inline_return_annotation_payload
        #        TRACKS an in-body comment (so a `|` inside one cannot terminate the payload) but
        #        never TRIMS it, so the comment lands inside the payload text and
        #        classify_return_annotation (:1472) reads a string that starts `{` and does not end
        #        `}`. It ships verbatim in generated/systemverilog_return_annotations.json.
        #        Owned by GRAMMAR-WELLFORMED.H.20.1, which LOWERS this ceiling when it lands.
        #   `222f7ddb` (.13c.2v) 152 -> 153  primary_dollar_sv_only
        #        semantic_annotation -> rule_reference — LANG-CAPABILITY-AUDIT.10.14.
        #   `958fcc24` (.13c.2y) 153 -> 155  kw_implements_e133e2cb + kw_implies_470cec58
        #        <absent> -> semantic_annotation AND semantic_annotation -> rule_reference. These
        #        two rows are ONE .10.14 defect: deleting the single `@profiles:` line above
        #        kw_implies_470cec58 removes BOTH and nothing else.
        #
        # ⇒ NO new divergence CLASS was created and NO site was lost; the three moved classes went
        # 56->58, 46->47 and 5->6. ⛔ The argument for raising rather than leaving it red: a gate
        # that is red for a KNOWN reason cannot report an UNKNOWN one, so while this row sat red an
        # envelope regression on any of the other thirteen grammars was invisible behind it.
        systemverilog) echo 155 ;;
        systemverilog_lrm_profiled_generated) echo 317 ;;
        # .10.14 plus the two documented arm-2 blind spots: `[> …]` lexical annotations (which
        # arm 2 misparses as character classes) and arm 1's inline/rule-level annotation
        # distinction, which arm 2 does not represent.
        systemverilog_preprocessor) echo 15 ;;
        # ⚠️ NOT a defect count — arm 1 RESOLVES `include(…)` and splices in 1398 rules while
        # arm 2 stops at the directive, so the two arms describe different rule sets. This is
        # the one grammar where the token counts cannot be read as parity, and the report's
        # `unresolved_include_directives` field says so on its own.
        systemverilog_lrm_profiled_wrapper) echo 1399 ;;
        *) echo -1 ;;
    esac
}

# Arm-2 constructs the projection has no mapping for. Same ratchet discipline: a NEW unmapped
# construct is a blind spot in the measurement itself, so it must fail rather than pass quietly.
envelope_unmapped_ceiling() {
    case "$1" in
        # The 12 `[> …]` lexical annotations arm 2 reads as character classes.
        systemverilog_preprocessor) echo 12 ;;
        *) echo 0 ;;
    esac
}

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
# ⭐ LANG-CAPABILITY-AUDIT.10.6 PART 2 — the OUTPUT-level comparison is now BUILT.
#
# Retiring the Perl arm had left this gate with only a verdict: arm 1 yields a raw-AST
# envelope and arm 2 yielded `Ok`/`Err`, so nothing compared what the two arms actually
# PRODUCE. `pgen::ebnf_envelope_differential` closes that: it projects arm 2's typed AST into
# arm 1's `raw_ast` token vocabulary and diffs them token by token, in ONE process that runs
# both arms (so the two sides can never be compared across stale artifacts). Every run also
# executes the differential's positive AND negative ground-truth controls first and REFUSES
# to report if either misses.
#
# The `envelope_*` columns below are that measurement. `envelope_equiv` is the real
# frontend-REPLACEMENT verdict — a verdict-level pass says the meta-parser READS a grammar;
# only envelope equivalence says it reads it the SAME WAY.

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

echo "grammar,rust_parse,rust_parse_full,rust_rule_count,raw_ast_status,parse_end,input_bytes,consumed_pct,envelope_tokens,envelope_agree_pct,envelope_div,envelope_ceiling,envelope_equiv,overall,notes" >"$SUMMARY_CSV"
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
    envelope_json="$WORK_DIR/${grammar}.envelope_differential.json"
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
    envelope_tokens="-"
    envelope_agree_pct="-"
    envelope_div="-"
    envelope_equiv="-"
    envelope_ceiling="$(envelope_divergence_ceiling "$grammar")"
    unmapped_ceiling="$(envelope_unmapped_ceiling "$grammar")"
    overall="fail"
    notes="internal error"

    # ⭐ The envelope differential runs in the SAME invocation as the parse report, so both arms
    # come from one process and one input read. `--envelope-differential` exits non-zero if
    # either ground-truth control fails, which is why a control miss aborts the grammar rather
    # than producing an unvalidated number.
    if "$RUST_DIFF_BIN" --input "$grammar_file" --output "$rust_json" \
        --envelope-differential "$envelope_json" >"$rust_log" 2>&1; then
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

    # ⭐ LANG-CAPABILITY-AUDIT.10.6 part 2 — the frontend⟷meta-parser ENVELOPE differential.
    # This is the arm-vs-arm OUTPUT comparison the gate never had; everything above it is a
    # verdict. `envelope_equiv` is the frontend-REPLACEMENT verdict.
    if [[ -f "$envelope_json" ]]; then
        envelope_summary="$(python3 - "$envelope_json" "$envelope_ceiling" "$unmapped_ceiling" <<'ENVELOPE'
import json, sys

report = json.load(open(sys.argv[1]))["envelope_differential"]
divergence_ceiling, unmapped_ceiling = int(sys.argv[2]), int(sys.argv[3])

compared = report["tokens_compared"]
# A not-comparable payload agrees on everything that WAS checked — its kind — so it counts as
# agreement here. The count travels in the report under its own name, so the bound stays visible.
agreed = report["token_matches"] + report["payload_not_comparable"]
percent = (100.0 * agreed / compared) if compared else 0.0
divergences = report["divergence_total"]
unmapped = sum(report["unmapped_arm2_constructs"].values())

problems = []
if divergence_ceiling < 0:
    problems.append("no envelope ceiling is declared for this grammar")
elif divergences > divergence_ceiling:
    problems.append(
        "envelope divergences REGRESSED: %d > ceiling %d" % (divergences, divergence_ceiling)
    )
elif divergences < divergence_ceiling:
    problems.append(
        "envelope divergences IMPROVED to %d (ceiling %d) - lower the ceiling in "
        "envelope_divergence_ceiling() so the gain is locked in"
        % (divergences, divergence_ceiling)
    )
if unmapped > unmapped_ceiling:
    problems.append(
        "the projection met %d arm-2 construct(s) it cannot map (ceiling %d): %s - the "
        "measurement itself has a new blind spot"
        % (unmapped, unmapped_ceiling,
           " ".join(sorted(report["unmapped_arm2_constructs"])))
    )
elif unmapped < unmapped_ceiling:
    problems.append(
        "unmapped arm-2 constructs IMPROVED to %d (ceiling %d) - lower the ceiling in "
        "envelope_unmapped_ceiling()" % (unmapped, unmapped_ceiling)
    )

# `,` is the summary file's separator, so it may never appear in a field.
print("%d|%.2f|%d|%s|%s" % (
    compared,
    percent,
    divergences,
    "yes" if report["is_envelope_equivalent"] else "no",
    "; ".join(problems).replace(",", ";"),
))
ENVELOPE
)"
        IFS='|' read -r envelope_tokens envelope_agree_pct envelope_div envelope_equiv \
            envelope_problems <<<"$envelope_summary"
        if [[ -n "$envelope_problems" ]]; then
            notes="envelope ratchet: ${envelope_problems}"
            failures=$((failures + 1))
        fi
    else
        notes="envelope differential produced no report (see logs/${grammar}.rust_parse.log)"
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
    # the hand-written frontend must export its envelope, and — since
    # LANG-CAPABILITY-AUDIT.10.6 part 2 — the two arms' ENVELOPES must diff within this
    # grammar's declared ceiling. (Before .10.6 this also required Perl parity, and TOLERATED
    # `perl_under_reports` - a pass that accepted the Perl arm being blind to 25 of
    # regex.ebnf's 276 rules.)
    if [[ "$rust_parse" == "pass" && "$rust_parse_full" == "pass" \
        && "$raw_ast_status" == "exported" && "$notes" != "envelope ratchet: "* ]]; then
        overall="pass"
        if [[ "$envelope_equiv" == "yes" ]]; then
            # ⚠️ No commas in a note: this file is comma-separated and `column -s,` renders it.
            notes="ENVELOPE-EQUIVALENT - ${envelope_tokens} tokens / 0 divergences; the meta-parser reproduces the hand-written frontend exactly (${rust_rule_count} rules)"
        else
            notes="self-hosting verdict pass; envelope ${envelope_agree_pct}% over ${envelope_tokens} tokens; ${envelope_div} divergence(s) at the declared ceiling ${envelope_ceiling} (${rust_rule_count} rules)"
        fi
    else
        if [[ "$notes" == "internal error" ]]; then
            notes="generated meta-parser did not fully consume the grammar"
        fi
        overall="fail"
        # An envelope-ratchet failure has already been counted where it was detected; counting
        # it again here would double-report one problem.
        if [[ "$notes" != "envelope ratchet: "* ]]; then
            failures=$((failures + 1))
        fi
    fi

    python3 - "$grammar" "$rust_parse" "$rust_parse_full" "$overall" "$notes" "$rust_rule_count" "$raw_ast_status" "$parse_end" "$input_bytes" "$consumed_pct" "$rust_json" "$rust_raw_ast_json" "$envelope_json" "$envelope_ceiling" "$diff_json" <<'ENTRY'
import json,sys,os
(
    grammar, rust_parse, rust_parse_full, overall, notes,
    rust_rule_count, raw_ast_status,
    parse_end, input_bytes, consumed_pct,
    rust_json_path, rust_raw_ast_json_path, envelope_json_path, envelope_ceiling, out_path
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
    "envelope_divergence_ceiling": int(envelope_ceiling),
    "artifacts": {
        "rust_json": rust_json_path if os.path.exists(rust_json_path) else None,
        "rust_raw_ast_json": rust_raw_ast_json_path if os.path.exists(rust_raw_ast_json_path) else None,
        "envelope_differential_json": envelope_json_path if os.path.exists(envelope_json_path) else None,
    },
}

# LANG-CAPABILITY-AUDIT.10.6 part 2 - carry the FULL envelope differential, divergence list and
# ground-truth verdicts included, so a failing run is diagnosable straight from the artifact
# instead of needing the gate re-run by hand.
if os.path.exists(envelope_json_path):
    payload["envelope_differential"] = json.load(open(envelope_json_path))

if os.path.exists(rust_json_path):
    payload["rust_report"] = json.load(open(rust_json_path))

with open(out_path, "w") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
ENTRY

    echo "${grammar},${rust_parse},${rust_parse_full},${rust_rule_count},${raw_ast_status},${parse_end},${input_bytes},${consumed_pct},${envelope_tokens},${envelope_agree_pct},${envelope_div},${envelope_ceiling},${envelope_equiv},${overall},${notes}" >>"$SUMMARY_CSV"
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
