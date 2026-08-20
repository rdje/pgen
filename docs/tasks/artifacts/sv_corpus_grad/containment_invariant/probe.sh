#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2w (d) — the adversarial probe for PARSE-COST-RATCHET's CONTAINMENT invariant.
#
# ⛔ WHY IT EXISTS. `contained_in_introduced_subgraph` is the first invariant in this gate that is
# not an arithmetic identity over three totals: it reads the PER-RULE measurement and asks
# reachability in the measured grammar's own reference graph. A mechanism that only ever says yes is
# indistinguishable from a hole ([[a-check-whose-inputs-all-pass-has-not-been-tested]]), so every
# way it must REFUSE is fired here and observed — and so is the one way it must ACCEPT, because an
# invariant that can only refuse is a different hole.
#
# ⭐ NOTHING IN THIS PROBE IS TYPED FROM MEMORY. The rules it perturbs are DERIVED by
# `perturb_rule.py` from the run's own graph and `rule_costs.tsv`: a probe that hard-coded "this
# rule is inside the sub-graph" would assert the very reachability fact under test, and would pass
# whenever probe and predicate were wrong the same way.
#
#   bash docs/tasks/artifacts/sv_corpus_grad/containment_invariant/probe.sh
#
# ⚠️ Each arm runs the ratchet's TIER 2 — a full ~2.3-minute re-measure of the pinned 192-file
# sample — because the fresh half of every comparison must be a real measurement. Nine arms is
# ~21 minutes. That is the honest cost of an on-demand probe and is not paid on a commit.
#
# Exit 0 = every arm behaved as specified. Exit 1 = an arm did not (a hole, or a broken gate).
set -uo pipefail
# ⛔ FIVE `..` segments: containment_invariant -> sv_corpus_grad -> artifacts -> tasks -> docs ->
# repo root. ⚠️ THIS PROBE SHIPPED WITH FOUR AND WAS CAUGHT BY ITS OWN PRE-FLIGHT on its first
# execution, resolving ROOT to `docs/` — the same defect the `accepted_rise_gate` probe hit, where
# every arm reported rc=127 and its expected-message assertion refused rather than scoring a
# refusal. A probe that cannot find the gate must not read as a probe that found a hole, and the
# pre-flight below is what makes that true rather than hoped.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -f scripts/check_parse_cost_ratchet.sh ] || { echo "probe: not at the repo root ($ROOT)" >&2; exit 2; }

HERE="docs/tasks/artifacts/sv_corpus_grad/containment_invariant"
ART="docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet"
ACCEPTED="$ART/accepted_rises.tsv"
COSTS="$ART/rule_costs.tsv"
ENTRIES="$ART/entries.tsv"
PERTURB_TOTAL="docs/tasks/artifacts/engine_universal_services/accepted_rise_gate/perturb.py"
# ⛔ ON-VOLUME BY POLICY (CLAUDE.md §13): scratch is derived from the repo root, never $TMPDIR.
BACKUP="$ROOT/rust/target/containment_probe"
GRAPH="$BACKUP/rule_graph.json"
LOG="$BACKUP/run.log"
DELTA=100000
INTRODUCED="randomize_call"
PASS=0
FAIL=0

rm -rf "$BACKUP"; mkdir -p "$BACKUP" || exit 2

restore() { for f in accepted_rises.tsv rule_costs.tsv entries.tsv cost.md advisory.json; do
              cp -f "$BACKUP/$f" "$ART/$f" 2>/dev/null; done; }
cleanup() { restore; echo "probe: restored $ART from $BACKUP"; }
trap cleanup EXIT

for f in accepted_rises.tsv rule_costs.tsv entries.tsv cost.md advisory.json; do
  cp -f "$ART/$f" "$BACKUP/$f" || exit 2
done
echo "probe: backed up the baseline + acceptance record to $BACKUP"

# The reference graph of the grammar this tree measures — the same derivation the gate reads.
python3 scripts/parse_cost_containment.py --graph grammars/systemverilog.ebnf --out "$GRAPH" || exit 2

pick() { python3 "$HERE/perturb_rule.py" --graph "$GRAPH" --costs "$BACKUP/rule_costs.tsv" "$@"; }
bend() { python3 "$HERE/perturb_rule.py" --graph "$GRAPH" --costs "$COSTS" --apply "$@"; }

INSIDE="$(pick --pick inside  --introduced $INTRODUCED --min-entries $((DELTA * 2)))" || exit 2
OUTSIDE="$(pick --pick outside --introduced $INTRODUCED --min-entries $((DELTA * 2)))" || exit 2
ALT="$(pick --pick-alt-introduced --exclude "$INSIDE")" || exit 2
echo "probe: derived targets — inside=$INSIDE outside=$OUTSIDE alt_introduced=$ALT"

# Reset to the pristine baseline, then lower the TOTAL entries so this tree measures as a rise.
# Prints "from<TAB>to" — the exact integers an acceptance row must name.
rise() {
  restore
  python3 "$PERTURB_TOTAL" "$ENTRIES" entries "$DELTA" > "$BACKUP/rise.tsv" || exit 2
  E_FROM="$(cut -f1 "$BACKUP/rise.tsv")"; E_TO="$(cut -f2 "$BACKUP/rise.tsv")"
}

# $1 metric  $2 invariant  $3 introduced-column  $4 why
row() { printf '%s\t%s\t%s\t%s\tPROBE\t%s\t%s\n' "$1" "$E_FROM" "$E_TO" "$2" "$4" "$3" >> "$ACCEPTED"; }

# $1 arm name   $2 expected exit code   $3 a string the output must contain
arm() {
  local name="$1" want_rc="$2" want_msg="$3" rc
  PGEN_PARSE_COST_REMEASURE=1 bash scripts/check_parse_cost_ratchet.sh > "$LOG" 2>&1
  rc=$?
  if [ "$rc" = "$want_rc" ] && grep -qF "$want_msg" "$LOG"; then
    echo "  ✓ $name — rc=$rc and the message names it"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $name — expected rc=$want_rc containing:"
    echo "      $want_msg"
    echo "    got rc=$rc:"
    sed 's/^/      /' "$LOG" | tail -14
    FAIL=$((FAIL + 1))
  fi
}

# ── arm 1: the CONTROL — the unperturbed tree must be GREEN ─────────────────────────────────────
# Without it, a probe that broke the gate outright would score 8/8 refusals.
echo "arm 1: control — the unperturbed tree"
restore
arm "the unperturbed tree HOLDS" 0 "parse-cost-ratchet: OK"

# ── arm 2: a CONTAINED rise must be ACCEPTED ────────────────────────────────────────────────────
# The only per-rule movement is a rule INSIDE the sub-graph, so every leg holds. An invariant that
# can only refuse would fail here, and that is a hole of its own.
echo "arm 2: a contained rise is ACCEPTED"
rise
bend --rule "$INSIDE" --delta "$DELTA" --op lower || exit 2
row entries contained_in_introduced_subgraph "$INTRODUCED" "arm 2: the whole rise is on $INSIDE, inside the sub-graph"
arm "a CONTAINED rise is accepted" 0 "ACCEPTED as attributed"

# ── arm 3: an ESCAPING rise must be REFUSED ─────────────────────────────────────────────────────
echo "arm 3: a rise outside the sub-graph"
rise
bend --rule "$OUTSIDE" --delta "$DELTA" --op lower || exit 2
row entries contained_in_introduced_subgraph "$INTRODUCED" "arm 3: the rise is on $OUTSIDE, outside the sub-graph"
arm "an ESCAPING rise is REFUSED" 1 "rose OUTSIDE the sub-graph"

# ── arm 4: a rule that LOST entries must be REFUSED ─────────────────────────────────────────────
# Containment asserts the change is purely ADDITIVE; a fall means entries were re-routed, and a
# per-rule rise can no longer be attributed to the introduced construct.
echo "arm 4: a rule whose entries fell"
rise
bend --rule "$INSIDE" --delta "$DELTA" --op lower || exit 2
bend --rule "$OUTSIDE" --delta "$DELTA" --op raise || exit 2
row entries contained_in_introduced_subgraph "$INTRODUCED" "arm 4: $OUTSIDE loses entries, so the change is not additive"
arm "a FALLING rule is REFUSED" 1 "LOST entries"

# ── arm 5: THE SAME MEASUREMENT, A PERTURBED `introduced` SET, MUST FLIP THE VERDICT ────────────
# Byte-for-byte arm 2's perturbation. Only the row's declared scope changes. If the verdict did not
# move, the scope column would be decoration and the acceptance would not be scoped at all.
echo "arm 5: arm 2's numbers with a perturbed introduced set"
rise
bend --rule "$INSIDE" --delta "$DELTA" --op lower || exit 2
row entries contained_in_introduced_subgraph "$ALT" "arm 5: same rise, scope narrowed to $ALT"
arm "a PERTURBED introduced set REFUSES" 1 "rose OUTSIDE the sub-graph"

# ── arm 6: a scoped invariant with NO scope must be REFUSED ─────────────────────────────────────
echo "arm 6: a containment row declaring no introduced set"
rise
bend --rule "$INSIDE" --delta "$DELTA" --op lower || exit 2
row entries contained_in_introduced_subgraph "-" "arm 6: no scope declared"
arm "an UNSCOPED containment row is REFUSED" 1 "declares no \`introduced\` rule set"

# ── arm 7: an UNSCOPED invariant carrying a scope must be REFUSED ───────────────────────────────
# A row that names rules an invariant ignores reads as a narrower acceptance than the gate applies.
echo "arm 7: an unscoped invariant carrying an introduced set"
rise
row entries pure_memo_lookups "$INTRODUCED" "arm 7: pure_memo_lookups ignores a scope"
arm "a SCOPE on an unscoped invariant is REFUSED" 1 "which ignores it"

# ── arm 8: an `introduced` rule the grammar does not have must be REFUSED ───────────────────────
echo "arm 8: an introduced rule that is not in the grammar"
rise
bend --rule "$INSIDE" --delta "$DELTA" --op lower || exit 2
row entries contained_in_introduced_subgraph "no_such_rule_in_any_grammar" "arm 8: a typo'd scope"
arm "an UNKNOWN introduced rule is REFUSED" 1 "name no rule in the grammar's reference graph"

# ── arm 9: a MISSING per-rule baseline must REFUSE, never pass ──────────────────────────────────
# The predicate is per-RULE. If its evidence is absent the acceptance is un-evaluable, and an
# un-evaluable acceptance covering a measured rise must not read GREEN.
echo "arm 9: the per-rule baseline is missing"
rise
row entries contained_in_introduced_subgraph "$INTRODUCED" "arm 9: rule_costs.tsv is gone"
rm -f "$COSTS"
arm "a MISSING per-rule baseline REFUSES" 1 "could NOT BE EVALUATED"

restore
echo "CONTAINMENT-INVARIANT-PROBE: arms_passed=$PASS arms_failed=$FAIL"
[ "$FAIL" = 0 ] || exit 1
