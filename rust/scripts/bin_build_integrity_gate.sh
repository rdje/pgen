#!/usr/bin/env bash
# BIN-BUILD-INTEGRITY.2 — every tracked binary must still COMPILE, in every
# configuration this repository actually builds.
#
# WHY THIS GATE EXISTS (the blind spot it closes):
#   The maintained batteries build `--lib` / `--tests`, never `--bins`, and
#   they build under DEFAULT features only. A `[[bin]]` is therefore invisible
#   to them twice over — always (no battery asks for bins at all) and again
#   when it declares `required-features`, which excludes it from every
#   default-feature build. `ebnf_dual_run_diff` rotted in exactly that hole
#   for a week (BIN-BUILD-INTEGRITY.1): a type migration landed, the full
#   battery passed, clippy passed, and the binary simply did not compile.
#   A build-coverage hole is not a failure, it is a SILENCE.
#
# WHAT IT PROVES (two things, both mechanical):
#   1. COVERAGE — the binary census is derived from `cargo metadata`, not from
#      a hand-maintained list, and every declared binary is proven to be
#      included in at least one checked configuration. A new binary whose
#      feature requirements no configuration satisfies FAILS the gate by name,
#      so the census can never silently drift out from under the check.
#   2. COMPILATION — each configuration is `cargo check`ed with `--all-targets`
#      (bins + lib + tests + benches + examples), so a feature-gated test
#      module that no run configuration compiles is caught by the same sweep.
#
# WHY A SET OF CONFIGURATIONS AND NOT ONE UNION BUILD:
#   The union is not honest here and is not even expressible. `--all-features`
#   is invalid (`mimalloc_perf` and `never_free_arena_perf` both install a
#   `#[global_allocator]`), and a union build hides exactly the failure mode
#   that rotted: code reachable only under ONE feature. The configurations
#   below are not a matrix for its own sake — each is a configuration this
#   repository genuinely builds (the Makefile's bootstrap target, the
#   `focus_*` regen path, the maintained gates, the documented toolbox
#   binary), so each must compile.
#
# COST: this is a HEAVY job (one `cargo check` per configuration, and distinct
# feature sets do not share build artifacts). It is a maintained gate, not a
# pre-commit hook. Run it under the memory guard, per the host-RAM directive:
#   scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 5400 -- \
#     make -C rust SHELL=/bin/bash bin_build_integrity_gate
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
STATE_DIR="${PGEN_BIN_BUILD_INTEGRITY_STATE_DIR:-$RUST_DIR/target/bin_build_integrity_gate}"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"
COVERAGE_JSON="$STATE_DIR/coverage.json"

mkdir -p "$STATE_DIR" "$LOG_DIR"

# The configuration plan: label | cargo feature flags | why the repo builds it.
# Keep this in step with the build commands the repository actually issues; a
# configuration that no longer exists should be removed, not left to rot.
CONFIG_LABELS=(
    "default"
    "bootstrap"
    "generated_parsers"
    "ebnf_dual_run"
    "generated_parsers+ebnf_dual_run"
)
CONFIG_FLAGS=(
    ""
    "--no-default-features --features bootstrap"
    "--features generated_parsers"
    "--features ebnf_dual_run"
    # Comma-separated, NOT space-separated: this string is word-split into
    # argv below, and `--features a b` would make `b` a stray positional
    # (cargo's --features takes ONE value token). Comma-joining keeps the
    # feature set a single argv element that survives the split.
    "--features generated_parsers,ebnf_dual_run"
)
CONFIG_WHY=(
    "the default build (feature 'normal') — every maintained battery and the plain cargo flow"
    "rust/Makefile ast_pipeline_bootstrap — the annotation-parser bootstrap path"
    "the canonical regen path: make focus_<grammar> builds the pipeline with this feature set"
    "the .ebnf frontend path used by the EBNF dual-run differential harness"
    "the documented toolbox binary (TOOLBOX.md 'The two binaries') — ast_pipeline for cert-coverage + .ebnf input"
)
# The feature set each configuration ENABLES, used for the coverage proof.
# 'normal' is the package default, so every configuration that does not pass
# --no-default-features has it on.
CONFIG_FEATURES=(
    "normal"
    "bootstrap"
    "normal,generated_parsers"
    "normal,ebnf_dual_run"
    "normal,generated_parsers,ebnf_dual_run"
)

print_log_excerpt() {
    local label="$1"
    local log_path="$2"
    local status="$3"
    local max_head_lines="${4:-40}"
    local max_tail_lines="${5:-60}"

    echo "error: configuration '${label}' failed cargo check with exit code ${status}" >&2
    echo "log: ${log_path}" >&2
    if [[ ! -s "$log_path" ]]; then
        echo "(log file missing or empty)" >&2
        return
    fi

    local total_lines
    total_lines=$(wc -l <"$log_path" | tr -d ' ')
    echo "--- begin ${label} log ---" >&2
    if [[ "$total_lines" -le $((max_head_lines + max_tail_lines)) ]]; then
        cat "$log_path" >&2
    else
        sed -n "1,${max_head_lines}p" "$log_path" >&2
        echo "--- log truncated; showing last ${max_tail_lines} lines of ${total_lines} ---" >&2
        tail -n "$max_tail_lines" "$log_path" >&2
    fi
    echo "--- end ${label} log ---" >&2
}

# STEP 1 — the coverage proof, BEFORE spending any build time. Derive the
# binary census from cargo metadata and assert every binary is included in at
# least one planned configuration. This is what stops the census from drifting
# out from under the gate: a new required-features binary nobody planned for
# fails here, by name, in seconds.
echo "==> Deriving the binary census from cargo metadata"
metadata_json="$STATE_DIR/cargo_metadata.json"
(cd "$RUST_DIR" && cargo metadata --no-deps --format-version 1) >"$metadata_json"

coverage_status=0
python3 - "$metadata_json" "$COVERAGE_JSON" "$(IFS='|'; echo "${CONFIG_LABELS[*]}")" "$(IFS='|'; echo "${CONFIG_FEATURES[*]}")" <<'PY' || coverage_status=$?
import json
import sys

metadata_path, out_path, labels_raw, features_raw = sys.argv[1:5]

labels = labels_raw.split("|")
config_features = [set(f.split(",")) for f in features_raw.split("|")]

with open(metadata_path) as fh:
    metadata = json.load(fh)

bins = []
for package in metadata.get("packages", []):
    for target in package.get("targets", []):
        if "bin" in target.get("kind", []):
            bins.append(
                {
                    "name": target["name"],
                    "required_features": sorted(target.get("required-features", []) or []),
                }
            )
bins.sort(key=lambda b: b["name"])

rows = []
uncovered = []
for binary in bins:
    required = set(binary["required_features"])
    covering = [
        label
        for label, enabled in zip(labels, config_features)
        if required.issubset(enabled)
    ]
    rows.append({**binary, "covered_by": covering})
    if not covering:
        uncovered.append(binary["name"])

payload = {
    "bin_count": len(bins),
    "configurations": labels,
    "uncovered_bins": uncovered,
    "bins": rows,
}
with open(out_path, "w") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")

print(f"binaries declared: {len(bins)}")
for row in rows:
    req = ",".join(row["required_features"]) or "-"
    print(f"  {row['name']:42s} required-features={req:34s} covered_by={len(row['covered_by'])}")

if uncovered:
    print(
        "error: no planned configuration satisfies the required-features of: "
        + ", ".join(uncovered),
        file=sys.stderr,
    )
    print(
        "       add a configuration to CONFIG_* in rust/scripts/bin_build_integrity_gate.sh "
        "(or correct the binary's required-features)",
        file=sys.stderr,
    )
    raise SystemExit(1)
PY

if [[ "$coverage_status" -ne 0 ]]; then
    echo "❌ binary-coverage proof FAILED — the gate does not check every declared binary." >&2
    exit 1
fi

# STEP 2 — check every configuration. Failures are collected, not fatal on the
# first one: knowing that three configurations broke is worth more than
# stopping at the first.
failures=0
results=()
for index in "${!CONFIG_LABELS[@]}"; do
    label="${CONFIG_LABELS[$index]}"
    flags="${CONFIG_FLAGS[$index]}"
    why="${CONFIG_WHY[$index]}"
    log_path="$LOG_DIR/$(printf '%s' "$label" | tr '+' '_').log"

    echo "==> cargo check --all-targets ${flags:-(default features)}"
    echo "    why this configuration exists: ${why}"

    status=0
    # shellcheck disable=SC2086 -- word splitting of the flag string is intended
    (cd "$RUST_DIR" && cargo check --all-targets $flags) >"$log_path" 2>&1 || status=$?

    if [[ "$status" -eq 0 ]]; then
        echo "    ok"
        results+=("$label|pass|0")
    else
        print_log_excerpt "$label" "$log_path" "$status"
        results+=("$label|fail|$status")
        failures=$((failures + 1))
    fi
done

python3 - "$SUMMARY_JSON" "$COVERAGE_JSON" "$failures" "${results[@]}" <<'PY'
import json
import sys

out_path, coverage_path, failures = sys.argv[1:4]
rows = []
for raw in sys.argv[4:]:
    label, verdict, status = raw.split("|")
    rows.append({"configuration": label, "verdict": verdict, "exit_code": int(status)})

with open(coverage_path) as fh:
    coverage = json.load(fh)

payload = {
    "failing_configurations": int(failures),
    "configurations": rows,
    "bin_count": coverage["bin_count"],
    "uncovered_bins": coverage["uncovered_bins"],
}
with open(out_path, "w") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

{
    echo "PGEN binary build-integrity gate"
    echo "state_dir: $STATE_DIR"
    echo
    printf '%-34s %s\n' "configuration" "verdict"
    for row in "${results[@]}"; do
        IFS='|' read -r label verdict status <<<"$row"
        if [[ "$verdict" == "pass" ]]; then
            printf '%-34s %s\n' "$label" "pass"
        else
            printf '%-34s %s (exit %s)\n' "$label" "fail" "$status"
        fi
    done
    echo
    echo "Coverage: $(python3 -c "import json,sys;d=json.load(open('$COVERAGE_JSON'));print(f\"{d['bin_count']} binaries, all covered by at least one configuration\")")"
    echo "Summary JSON: $SUMMARY_JSON"
    echo "Coverage JSON: $COVERAGE_JSON"
    echo "Logs: $LOG_DIR"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if [[ "$failures" -ne 0 ]]; then
    echo "❌ $failures configuration(s) do not compile." >&2
    exit 1
fi

echo "✅ every declared binary compiles in every configuration this repository builds."
