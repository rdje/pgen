#!/usr/bin/env bash
# run_corpus_scale_census.sh — how much of each vendored external corpus does its gate actually
# assert? (task-tree leaf `DONE-BAR.1`, leg 3.)
#
# WHY. `audit_done_bar.sh` answers a yes/no question about leg 3 — is the corpus lane a conformance
# gate, is it external-backed, does anything run it. It deliberately does NOT price the lane. This
# census does, because the two triage gates report `parse_pass_total` equal to `cases_declared` and
# a reader can mistake that for "the corpus passes". The bar says ALL the external test corpus,
# passing; the ratio below is what "all" currently means.
#
# Everything here is DERIVED at run time: the case counts come from the tracked manifests, and the
# file counts from the vendored trees. Nothing is quoted from a previous run.
#
#   bash docs/tasks/artifacts/done_bar/run_corpus_scale_census.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

echo "=============================================================================="
echo "EXTERNAL-CORPUS SCALE CENSUS — declared cases vs vendored corpus (derived this run)"
echo "=============================================================================="
echo

# census <family> <manifest> <space-separated corpus roots> <extension globs...>
# ⛔ The roots are read from the tracked register, so the denominator is EVERY tree the family's
# corpus lane draws on. Counting one root when the manifest references two understates the corpus
# and makes coverage look better — the wrong direction for an audit to be wrong in.
census() {
    local family="$1" manifest="$2" roots_csv="$3"
    shift 3
    local exts=("$@")
    read -r -a roots <<<"$roots_csv"

    local declared
    declared="$(jq -r '.cases | length' "$manifest")"

    local find_args=()
    local first=1
    for e in "${exts[@]}"; do
        if [[ $first == 1 ]]; then find_args+=(-name "$e"); first=0
        else find_args+=(-o -name "$e"); fi
    done

    local files=0 subdirs=0 referenced=0
    for root in "${roots[@]}"; do
        [[ -d "$root" ]] || continue
        files=$((files + $(find "$root" -type f \( "${find_args[@]}" \) 2>/dev/null | wc -l | tr -d ' ')))
        subdirs=$((subdirs + $(find "$root" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')))
        referenced=$((referenced + $(grep -oE "${root}/[A-Za-z0-9_.-]+" "$manifest" | sort -u | wc -l | tr -d ' ')))
    done

    printf '%s\n' "$family"
    printf '  manifest              %s\n' "${manifest#"$ROOT"/}"
    printf '  declared cases        %s\n' "$declared"
    printf '  vendored source files %s   (under %s)\n' "$files" "$roots_csv"
    printf '  vendored corpora      %s, of which the manifest references %s\n' "$subdirs" "$referenced"
    if [[ "$files" -gt 0 ]]; then
        printf '  coverage              %s\n' \
            "$(python3 -c "print(f'{$declared/$files*100:.3f}% of files')")"
    fi
    echo
}

# Roots come from the register, not from this script, so the two cannot drift apart.
roots_for() {
    jq -r --arg f "$1" '.families[$f].corpus_roots | join(" ")' \
        "$ROOT/rust/test_data/grammar_quality/done_bar_family_register_v0.json"
}

census "vhdl" \
    "$ROOT/rust/test_data/grammar_quality/vhdl_external_corpus_triage_v0.json" \
    "$(roots_for vhdl)" '*.vhd' '*.vhdl'

census "systemverilog" \
    "$ROOT/rust/test_data/grammar_quality/systemverilog_external_corpus_triage_v0.json" \
    "$(roots_for systemverilog)" '*.sv' '*.svh' '*.v' '*.vh'

echo "regex"
echo "  the regex family has no triage manifest; its lanes are priced directly:"
printf '  PCRE2 oracle cases (canonical)   %s\n' \
    "$(wc -l < regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl | tr -d ' ')"
printf '  PCRE2 answer key (expected)      %s\n' \
    "$(jq -c '.expected_parse_counts' regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_summary.json)"
printf '  regex_broader_corpus_proof_gate reads  %s (%s cases)\n' \
    "$(jq -r '.source_file' rust/test_data/grammar_quality/regex_broader_corpus_v0.json)" \
    "$(jq -r '.expected_case_count' rust/test_data/grammar_quality/regex_broader_corpus_v0.json)"
echo "  ⇒ the lane that RUNS reads a repo-authored fixture; the 2,189-case PCRE2 corpus is"
echo "    read only by gates nothing invokes (see audit_report.txt, regex leg 3)."
echo

echo "------------------------------------------------------------------------------"
echo "families claiming \`Done\` with NO external corpus vendored at all:"
for f in systemverilog_preprocessor return_annotation rtl_frontend; do
    printf '  %-28s corpus roots declared in the register: %s\n' "$f" \
        "$(jq -r --arg f "$f" '.families[$f].corpus_roots | if length == 0 then "NONE" else join(",") end' \
            rust/test_data/grammar_quality/done_bar_family_register_v0.json)"
done
