#!/usr/bin/env bash
# RGX-0078.5.j.4 BATCH-1 PRE-FLIGHT — price members 2 (C2 fact-ops) and
# 6 (trace-eagerness) from per-parse semantic-store EVENT COUNTS.
#
# WHAT: for each of the 8 bench patterns (the geomean steering population,
#       `rust/src/bin/regex_perf_probe.rs:242-254`), dump the parse's
#       `store_counters` via `--dump-rule-outcome-counts-json` (TOOLBOX 3.5)
#       and report `facts_emitted` + `predicate_evaluations` per parse.
#
# WHY:  member 6 removes ONE `rule_context_path()` String allocation per
#       `emit_fact` and per predicate query (9 eager sites,
#       `semantic_runtime.rs`); member 2 removes the `to_ascii_lowercase()`
#       allocation per FactIndex insert AND per query. Both levers' populations
#       are therefore EXACTLY these two counters — so counting them prices the
#       levers without a build-and-bench chain.
#
# ⚠️ WHY THE `-0163` STANDING INSTRUMENT RULE DOES NOT BITE HERE:
#    that rule forbids pricing a FUSED-PATH FRAME with a runtime counter census,
#    because enabling counters routes the parse to the PROTOCOL graph (the
#    observability twin). But a semantic-store EVENT count is graph-INVARIANT:
#    the twin is pinned byte-identical on verdict and typed AST, which entails
#    the same facts emitted and the same predicates evaluated on either graph.
#    The count is a property of (grammar x input), not of which graph ran it.
#    This is an ARGUMENT, not a measurement — it is recorded as such.
#
# CUSTODY (two gates, both hard):
#   (1) the regex artifact must be the banked floor vintage `e4924024`;
#   (2) the probe binary must be NEWER than that artifact, i.e. actually built
#       from it. Gate (2) exists because this slice CAUGHT a stale probe:
#       the on-disk debug probe predated the artifact by 28 minutes and would
#       have reported counts for the wrong vintage.
#
# HOW: bash docs/tasks/artifacts/batch1_preflight/preflight_store_counters.sh
# OUT: preflight_store_counters.txt (+ per-pattern JSON under ./dumps/)

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

ART="generated/regex_parser.rs"
EXPECT_SHA="e4924024"
PROBE="rust/target/debug/parseability_probe"
OUTDIR="docs/tasks/artifacts/batch1_preflight/dumps"

echo "RGX-0078.5.j.4 BATCH-1 PRE-FLIGHT — semantic-store event counts"
echo "root: $ROOT"
echo

# ---- custody gate 1: artifact vintage ---------------------------------------
[ -f "$ART" ] || { echo "REFUSED: $ART absent (regenerate: make -C rust focus_regex)." >&2; exit 2; }
GOT="$(shasum -a 256 "$ART" | cut -c1-8)"
[ "$GOT" = "$EXPECT_SHA" ] || {
  echo "REFUSED: artifact custody mismatch — expected $EXPECT_SHA, got $GOT." >&2; exit 3; }

# ---- custody gate 2: probe actually built FROM that artifact ----------------
[ -x "$PROBE" ] || { echo "REFUSED: $PROBE absent/not executable." >&2; exit 4; }
if [ "$PROBE" -ot "$ART" ]; then
  echo "REFUSED: probe is OLDER than the artifact — it embeds a STALE regex parser." >&2
  echo "  probe:    $(stat -c '%y' "$PROBE")" >&2
  echo "  artifact: $(stat -c '%y' "$ART")" >&2
  echo "  rebuild:  (cd rust && cargo build --features generated_parsers --bin parseability_probe)" >&2
  exit 5
fi
echo "custody: artifact $ART sha256[0:8]=$GOT (OK — banked floor vintage)"
echo "custody: probe    $PROBE is newer than the artifact (OK — built from it)"
echo "  probe    mtime: $(stat -c '%y' "$PROBE")"
echo "  artifact mtime: $(stat -c '%y' "$ART")"
echo

mkdir -p "$OUTDIR"

# The 8 bench patterns — the geomean steering population, copied verbatim from
# rust/src/bin/regex_perf_probe.rs:242-254 (keep in lockstep if that changes).
names=(literal_simple digit_sequence character_class alternation capture_groups url_simple email_basic anchor_complex)
pats=(
  'test'
  '\d{3}-\d{2}-\d{4}'
  '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
  'cat|dog|bird'
  '(\d{4})-(\d{2})-(\d{2})'
  'https?://\S+'
  '\b\w+@\w+\.\w+\b'
  '^(\d+)\s+(?P<word>\w+)\s+(?:foo|bar)$'
)

printf '%-16s %8s %10s %10s %10s\n' pattern bytes entries facts predicates
printf '%-16s %8s %10s %10s %10s\n' ---------------- -------- ---------- ---------- ----------

tot_f=0; tot_p=0; n=0
for i in "${!names[@]}"; do
  nm="${names[$i]}"; pat="${pats[$i]}"
  src="$OUTDIR/$nm.pattern"; dst="$OUTDIR/$nm.outcome.json"
  printf '%s' "$pat" > "$src"
  "$PROBE" --parse regex "$src" --dump-rule-outcome-counts-json "$dst" >/dev/null 2>&1 || {
    echo "REFUSED: parse failed for '$nm' — the probe/artifact pair is not usable." >&2; exit 6; }
  read -r ent f p < <(python3 -c "
import json,sys
d=json.load(open('$dst'))
sc=d.get('store_counters',{})
print(d.get('total_entries',0), sc.get('facts_emitted',0), sc.get('predicate_evaluations',0))
")
  printf '%-16s %8d %10d %10d %10d\n' "$nm" "${#pat}" "$ent" "$f" "$p"
  tot_f=$((tot_f+f)); tot_p=$((tot_p+p)); n=$((n+1))
done

echo
echo "per-parse ARITHMETIC MEAN over the 8 bench patterns:"
python3 -c "
f=$tot_f/$n; p=$tot_p/$n
print(f'  facts_emitted        {f:8.2f} / parse')
print(f'  predicate_evaluations{p:8.2f} / parse')
print()
print('POPULATION EACH LEVER REMOVES (allocations per parse):')
print(f'  member 6 trace-eagerness : emit_fact site + 6 predicate-query sites')
print(f'                             upper bound = facts + predicates = {f+p:.2f} String allocs/parse')
print(f'  member 2 C2 fact-ops     : FactIndex insert path = facts = {f:.2f} lowercased-String allocs/parse')
print(f'                             (+ one per has_fact/count query on the same kind path)')
print()
print('⚠️  NO ns FIGURE IS DERIVED HERE. Converting an allocation COUNT into')
print('    nanoseconds requires a measured per-allocation cost under mimalloc,')
print('    which this slice does not have and does not invent (the -0166/-0172')
print('    lesson). The count is the deliverable; the A/B supplies the ns.')
print()
print('⚠️  DOUBLE-COUNTING GUARD: members 5 and 6 BOTH act on the semantic-runtime')
print('    population that -0171 attributed at 24.1 ns. They are NOT additive with')
print('    each other, and member 6 is NOT additional upside on top of that row —')
print('    it is a MECHANISM for part of it.')
"
