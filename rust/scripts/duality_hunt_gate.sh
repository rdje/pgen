#!/usr/bin/env bash
# duality_hunt_gate.sh — the plain-config duality-hunt gate (STIMULI-SIGNOFF.13.3).
#
# Runs the .4.4 duality-break hunter over every lane pinned in the tracked contract
# (rust/test_data/grammar_quality/duality_hunt_gate_contract_v0.json) and asserts the
# observed break-signature set equals the pinned set per (grammar, seed, budget).
# This is the HONEST plain-configuration duality coverage mandated by the
# STIMULI-SIGNOFF.13.1 §6 cert-spf adjudication (the cert pass's sample_parse_failures=0
# is config-scoped; this lane is what re-earns the duality picture).
#
#   - a NOVEL signature (observed, not pinned)  -> FAIL: route the new break class to a
#     task-tree leaf (the be-alert doctrine), then pin it with its owner.
#   - a VANISHED signature (pinned, not observed) -> FAIL: the contract is stale — the
#     closing slice re-baselines the contract same-commit (closure progress is recorded).
#
# Determinism tripwire: the first lane is run twice and the two JSON reports must be
# byte-identical.
#
# Env overrides:
#   PGEN_DUALITY_HUNT_CONTRACT   — alternate contract path (testability hook)
#   PGEN_DUALITY_HUNT_STATE_DIR  — alternate state dir (default rust/target/duality_hunt_gate)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

CONTRACT="${PGEN_DUALITY_HUNT_CONTRACT:-$RUST_DIR/test_data/grammar_quality/duality_hunt_gate_contract_v0.json}"
STATE_DIR="${PGEN_DUALITY_HUNT_STATE_DIR:-$RUST_DIR/target/duality_hunt_gate}"
LOG_DIR="$STATE_DIR/logs"
REPORT_DIR="$STATE_DIR/reports"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

AST_PIPELINE="$RUST_DIR/target/debug/ast_pipeline"

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

require_tool python3
require_file "$CONTRACT"

mkdir -p "$STATE_DIR" "$LOG_DIR" "$REPORT_DIR"
: >"$SUMMARY_TXT"

echo "==> building dual-feature ast_pipeline (generated_parsers + ebnf_dual_run)"
build_log="$LOG_DIR/build.log"
if ! (cd "$RUST_DIR" && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline) >"$build_log" 2>&1; then
    echo "error: ast_pipeline build failed (log: $build_log)" >&2
    tail -n 60 "$build_log" >&2 || true
    exit 1
fi

# Enumerate lanes as tab-separated rows from the contract.
lanes_tsv="$STATE_DIR/lanes.tsv"
python3 - "$CONTRACT" >"$lanes_tsv" <<'PY'
import json, sys
contract = json.load(open(sys.argv[1]))
for lane in contract["lanes"]:
    print("\t".join([
        lane["lane_id"], lane["grammar"], lane["grammar_file"],
        str(lane["rounds"]), str(lane["samples_per_round"]), str(lane["seed"]),
    ]))
PY

run_lane() {
    local lane_id="$1" grammar_file="$2" rounds="$3" spr="$4" seed="$5" report="$6"
    local log_file="$LOG_DIR/$(basename "$report" .json).log"
    echo "==> lane ${lane_id} (rounds=${rounds} samples_per_round=${spr} seed=${seed})"
    if ! "$AST_PIPELINE" "$ROOT_DIR/$grammar_file" \
        --directed-generation-goal duality_break \
        --directed-rounds "$rounds" --directed-samples-per-round "$spr" --seed "$seed" \
        --directed-report-json "$report" >"$log_file" 2>&1; then
        echo "error: lane '$lane_id' hunter run failed (log: $log_file)" >&2
        tail -n 40 "$log_file" >&2 || true
        exit 1
    fi
}

first_lane_id=""
while IFS=$'\t' read -r lane_id grammar grammar_file rounds spr seed; do
    [[ -n "$lane_id" ]] || continue
    if [[ -z "$first_lane_id" ]]; then
        first_lane_id="$lane_id"
    fi
    run_lane "$lane_id" "$grammar_file" "$rounds" "$spr" "$seed" "$REPORT_DIR/${lane_id}.json"
done <"$lanes_tsv"

# Determinism tripwire: re-run the first lane and byte-compare the JSON reports.
if [[ -n "$first_lane_id" ]]; then
    IFS=$'\t' read -r lane_id grammar grammar_file rounds spr seed \
        < <(head -n 1 "$lanes_tsv")
    echo "==> determinism tripwire: re-running lane ${first_lane_id}"
    run_lane "$first_lane_id" "$grammar_file" "$rounds" "$spr" "$seed" "$REPORT_DIR/${first_lane_id}_repeat.json"
    if ! cmp -s "$REPORT_DIR/${first_lane_id}.json" "$REPORT_DIR/${first_lane_id}_repeat.json"; then
        echo "error: determinism tripwire FAILED — lane '${first_lane_id}' produced different JSON reports on repeat" >&2
        exit 1
    fi
    echo "    byte-identical repeat ✓"
fi

# Compare every lane report against the pinned contract.
echo "==> comparing observed signature sets against the pinned contract"
python3 - "$CONTRACT" "$REPORT_DIR" "$SUMMARY_TXT" "$SUMMARY_JSON" <<'PY'
import json, os, sys

contract_path, report_dir, summary_txt, summary_json = sys.argv[1:5]
contract = json.load(open(contract_path))

failures = []
lane_rows = []
for lane in contract["lanes"]:
    lane_id = lane["lane_id"]
    report = json.load(open(os.path.join(report_dir, f"{lane_id}.json")))

    # Echo-back checks: the report must describe exactly the pinned run.
    for key, want in (
        ("goal", "duality_break"),
        ("grammar_name", lane["grammar"]),
        ("rounds", lane["rounds"]),
        ("samples_per_round", lane["samples_per_round"]),
        ("seed", lane["seed"]),
    ):
        got = report.get(key)
        if got != want:
            failures.append(f"[{lane_id}] report echo-back mismatch: {key}={got!r}, contract pins {want!r}")

    expected = {e["signature"]: e for e in lane["expected_signatures"]}
    observed = {b["signature"]: b for b in report.get("breaks", [])}

    for sig, brk in sorted(observed.items()):
        if sig not in expected:
            failures.append(
                f"[{lane_id}] NOVEL duality-break signature (not pinned): \"{sig}\" "
                f"(shrunk_reproducer={brk.get('shrunk_reproducer')!r}, occurrences={brk.get('occurrences')}) "
                f"— a new break class: route it to a task-tree leaf (be-alert doctrine), then pin it with its owner."
            )
    for sig, ent in sorted(expected.items()):
        if sig not in observed:
            failures.append(
                f"[{lane_id}] VANISHED pinned signature: \"{sig}\" (owner {ent['owner']}) "
                f"— the contract is stale: re-baseline duality_hunt_gate_contract_v0.json same-commit "
                f"with the change that closed it."
            )

    lane_rows.append({
        "lane_id": lane_id,
        "grammar": lane["grammar"],
        "rounds": lane["rounds"],
        "samples_per_round": lane["samples_per_round"],
        "seed": lane["seed"],
        "directed_generated_samples": report.get("directed_generated_samples"),
        "directed_rejected_samples": report.get("directed_rejected_samples"),
        "diverse_baseline_rejected_samples": report.get("diverse_baseline_rejected_samples"),
        "observed_signatures": sorted(observed),
        "pinned_signatures": sorted(expected),
        "clean": all(not f.startswith(f"[{lane_id}]") for f in failures),
    })

status = "pass" if not failures else "fail"
with open(summary_txt, "w") as f:
    f.write("gate: duality_hunt_gate\n")
    f.write(f"contract: {contract_path}\n")
    f.write(f"status: {status}\n")
    for row in lane_rows:
        f.write(
            f"lane {row['lane_id']}: rejected {row['directed_rejected_samples']}"
            f"/{row['directed_generated_samples']} "
            f"(diverse baseline {row['diverse_baseline_rejected_samples']}), "
            f"signatures observed={row['observed_signatures']} pinned={row['pinned_signatures']}\n"
        )
    for failure in failures:
        f.write(f"FAIL: {failure}\n")

with open(summary_json, "w") as f:
    json.dump({
        "gate": "duality_hunt_gate",
        "contract": contract_path,
        "status": status,
        "lanes": lane_rows,
        "failures": failures,
    }, f, indent=2, sort_keys=True)
    f.write("\n")

for row in lane_rows:
    print(
        f"    lane {row['lane_id']}: rejected {row['directed_rejected_samples']}"
        f"/{row['directed_generated_samples']}, "
        f"signatures {len(row['observed_signatures'])} observed / {len(row['pinned_signatures'])} pinned"
    )
if failures:
    print()
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    sys.exit(1)
PY

echo "✅ duality-hunt gate passed."
echo "Summary TXT: $SUMMARY_TXT"
echo "Summary JSON: $SUMMARY_JSON"
