#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2x.1 / .13c.2x.3 — is the POST-ELIMINATION rule count deterministic?
#
# ⛔ WHY IT EXISTS. `.13c.2x.1` observed `sv_cert_recognized_union_gate` reporting `canonical total`
# = 1434 at seed 0 and 1433 at seeds 7 and 42, and recorded TWO hypotheses without adopting either:
#   H1  per-process nondeterminism — each seed is a separate PROCESS, so seed and process are
#       CONFOUNDED in that observation, and H1 would touch EVERY rule-count baseline in the repo;
#   H2  real seed-dependence.
# The counted population is the post-left-recursion-elimination rule set, which includes the rules
# PGEN SYNTHESIZES (`property_expr_lr_suffix_r0…r79`, `casting_type_lr_*`) and which appear zero
# times in the grammar source. If that synthesis were nondeterministic, every rule-count contract
# in `rust/test_data/grammar_quality/` would be pinning noise — including the one
# `SV-CORPUS-GRAD.13c.2x.3` stamped `expectations: confirmed`.
#
# ⭐ THIS PROBE DE-CONFOUNDS THE TWO AXES, on a DIFFERENT instrument that counts the SAME
# population: the syntax-probe gap report's `total_rules`.
#   axis A — five separate PROCESSES at ONE fixed seed   (the H1 axis, held clean of seed)
#   axis B — one process per seed across FOUR seeds      (the H2 axis)
#
# ⚠️ HONEST BOUND, stated rather than discovered: this is the GAP REPORT, not
# `certificate_coverage()`, and it runs on HEAD rather than on `.13c.2x.1`'s `SV-0065` arm. It
# therefore CANNOT close `.13c.2x.1`. What it can do — and does — is test whether the rule
# SYNTHESIS shared by both instruments is itself unstable, which is the part of H1 that would make
# the defect repo-wide.
#
#   bash docs/tasks/artifacts/sv_corpus_grad/rule_count_determinism/probe.sh
#
# Exit 0 = every run agreed on both axes. Exit 1 = a divergence (that IS the finding).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -f grammars/systemverilog.ebnf ] || { echo "probe: not at the repo root ($ROOT)" >&2; exit 2; }

BIN="rust/target/debug/ast_pipeline"
# The gate's own pre-elimination raw_ast export. Regenerate it with:
#   make -C rust SHELL=/bin/bash sv_syntax_closure_gate
GRAMMAR_JSON="rust/target/sv_syntax_closure_gate/work/systemverilog.json"
WORK="rust/target/rule_count_determinism_probe"      # on-volume by policy (CLAUDE.md §13)

for f in "$BIN" "$GRAMMAR_JSON"; do
  [ -f "$f" ] || { echo "probe: REFUSING — missing $f. Run 'make -C rust SHELL=/bin/bash sv_syntax_closure_gate' first." >&2; exit 2; }
done
rm -rf "$WORK"; mkdir -p "$WORK" || exit 2

run() {   # $1 tag   $2 seed
  "$BIN" "$GRAMMAR_JSON" --generate-stimuli --count 1 --seed "$2" \
      --entry-rule sv_multi_entry_root --output "$WORK/stim.txt" \
      --coverage-output "$WORK/cov.json" --gap-report-json "$WORK/gap_$1.json" \
      --gap-report-threshold 1 >"$WORK/run_$1.log" 2>&1 \
    || { echo "probe: the generator failed on $1 (see $WORK/run_$1.log)" >&2; return 1; }
  # ⛔⛔ TWO FIELD GROUPS, REPORTED AND COMPARED SEPARATELY — the first cut of this probe printed
  # them on one line and compared the line, which is why it announced "H2 holds for this counter"
  # when what actually moved was BRANCH DEBT, not any rule count. A probe that conflates a
  # STRUCTURAL property (graph reachability over the rule set) with a SAMPLING one (which branches
  # this seed's stimuli happened to exercise) cannot attribute its own finding.
  python3 - "$WORK/gap_$1.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); s = d["summary"]
u = ",".join(sorted(r["rule_name"] for r in d.get("unreachable_rule_debt", [])))
# STRUCTURAL — a function of the grammar alone; must not move on either axis.
print("STRUCT total=%d reachable=%d unreachable=%d total_branches=%d [%s]" % (
    s["total_rules"], s["reachable_rules"], s["unreachable_rules"], s["total_branches"], u))
# SAMPLED — a function of what this seed's stimuli exercised; seed-dependence here is EXPECTED.
print("SAMPLE unreachable_branches=%d" % s["unreachable_branches"])
PY
}

struct_of() { printf '%s\n' "$1" | grep '^STRUCT '; }
sample_of() { printf '%s\n' "$1" | grep '^SAMPLE '; }

fail=0

echo "axis A — five separate PROCESSES, one fixed seed (24001): the H1 axis"
a_struct=(); a_sample=()
for i in 1 2 3 4 5; do
  out="$(run "a$i" 24001)" || exit 2
  echo "  process $i: $(struct_of "$out")"
  a_struct+=("$(struct_of "$out")"); a_sample+=("$(sample_of "$out")")
done
if [ "$(printf '%s\n' "${a_struct[@]}" | sort -u | wc -l | tr -d ' ')" = "1" ]; then
  echo "  ✓ STRUCTURAL: all five processes agree — H1 (per-process nondeterminism of rule"
  echo "    synthesis) is REFUTED for this counter"
else
  echo "  ✗ STRUCTURAL: processes DISAGREE at a fixed seed — H1 holds, and EVERY rule-count" >&2
  echo "    baseline in rust/test_data/grammar_quality/ is pinning noise" >&2
  fail=1
fi
if [ "$(printf '%s\n' "${a_sample[@]}" | sort -u | wc -l | tr -d ' ')" != "1" ]; then
  echo "  ✗ SAMPLED: branch debt moved between processes at ONE seed — the generator is not" >&2
  echo "    reproducible from its seed, which is a separate and more serious defect" >&2
  fail=1
fi

echo "axis B — one process per seed, four seeds: the H2 axis"
b_struct=(); b_sample=()
for s in 0 7 42 24001; do
  out="$(run "b$s" "$s")" || exit 2
  echo "  seed $s: $(struct_of "$out")  $(sample_of "$out")"
  b_struct+=("$(struct_of "$out")"); b_sample+=("$(sample_of "$out")")
done
if [ "$(printf '%s\n' "${b_struct[@]}" | sort -u | wc -l | tr -d ' ')" = "1" ]; then
  echo "  ✓ STRUCTURAL: all four seeds agree — H2 (seed-dependence of the rule count) is REFUTED"
else
  echo "  ✗ STRUCTURAL: the RULE COUNT moved with the seed — H2 holds" >&2
  fail=1
fi
b_sample_distinct="$(printf '%s\n' "${b_sample[@]}" | sort -u | wc -l | tr -d ' ')"
echo ""
echo "⚠️ SAMPLED branch debt across those four seeds: $b_sample_distinct distinct value(s)."
if [ "$b_sample_distinct" != "1" ]; then
  # ⛔ NOT counted as a failure of THIS probe's question, and the distinction is the whole point.
  # Branch DEBT is what a seed's stimuli happened not to exercise — seed-dependence there is
  # expected and correct. It is reported loudly anyway because a CONTRACT that pins it against a
  # fixed ceiling is pinning a sampling artifact, and that is a finding about the contract.
  printf '%s\n' "${b_sample[@]}" | sed 's/^/     /'
  echo "   ⇒ EXPECTED for a sampling metric, and a FINDING about any contract that pins it:"
  echo "     systemverilog_syntax_closure_contract.json declares stimuli_seed 24001 and"
  echo "     max_unreachable_branches, so its ceiling holds ONLY at that seed."
fi

echo ""
if [ "$fail" -eq 0 ]; then
  echo "probe: the post-elimination RULE count is process-deterministic AND seed-independent here."
  echo "       ⚠️ That does NOT close SV-CORPUS-GRAD.13c.2x.1 — different instrument, different"
  echo "       tree state. It removes the repo-wide hypothesis, not the observation."
else
  echo "probe: a divergence was observed in a field that must not move. THAT is the finding."
fi
exit "$fail"
