#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2i — the divergence-list cap escape, and the two-sided ratchet it feeds.
#
# ⛔ WHY. `ebnf_frontend_dual_run_gate` was RED on `systemverilog` at `151 > ceiling 150` and the
# report could not say WHICH divergence was the extra one: its `divergences` list is capped at 40
# with no override, so a set-diff of the ceiling-era report against today's came back EMPTY ON BOTH
# SIDES — same first 40 rows, and the new one past the cap. `PGEN_ENVELOPE_DUMP_ALL=1` lifts the cap
# (deliberately named after `PGEN_LINT_DUMP_ALL`, which exists for the identical reason on the lint).
#
#   bash docs/tasks/artifacts/sv_corpus_grad/es13c2i_envelope_cap/probe.sh
#   # ENVELOPE-CAP: N/N as declared
#
# The total is DERIVED from the arms that ran — never a stored number.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

DIFF="rust/target/debug/ebnf_dual_run_diff"
GATE="rust/scripts/ebnf_frontend_dual_run_diff_gate.sh"
SV="grammars/systemverilog.ebnf"
W="rust/target/es13c2i_cap_probe"
rm -rf "$W"; mkdir -p "$W"
pass=0; total=0

[ -x "$DIFF" ] || { echo "probe: $DIFF is missing — build it first:"; \
  echo "  (cd rust && cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff)"; exit 2; }

field() { python3 -c "
import json,sys
d=json.load(open(sys.argv[1]))['envelope_differential']
v=d[sys.argv[2]]
print(len(v) if isinstance(v,list) else v)" "$1" "$2"; }

run_diff() {  # run_diff <tag> [env…]
  local tag="$1"; shift
  env "$@" "$DIFF" --input "$SV" --output "$W/$tag.out.json" \
      --envelope-differential "$W/$tag.env.json" >/dev/null 2>&1
}

ok() { total=$((total+1)); if [ "$2" = "$3" ]; then pass=$((pass+1)); printf '  ✓ %-52s %s\n' "$1" "$2";
       else printf '  ✗ %-52s got %s, wanted %s\n' "$1" "$2" "$3"; fi }

echo "ENVELOPE-CAP — SV-CORPUS-GRAD.13c.2i"
echo

run_diff capped
run_diff dumpall PGEN_ENVELOPE_DUMP_ALL=1
ok "A1 default run truncates the list at the cap"  "$(field "$W/capped.env.json"  divergences)" "40"
ok "A2 PGEN_ENVELOPE_DUMP_ALL lists every one"     "$(field "$W/dumpall.env.json" divergences)" "$(field "$W/dumpall.env.json" divergence_total)"

# ⭐ THE BINDING COUNTS MUST NOT MOVE. A reporting knob that perturbs the measurement is not a
# reporting knob — every ratchet in the gate keys on these four.
for f in divergence_total tokens_compared token_matches kind_divergences; do
  ok "A3 $f is identical with the cap lifted" \
     "$(field "$W/capped.env.json" "$f")" "$(field "$W/dumpall.env.json" "$f")"
done

# ⭐ A4 — `0` and empty must mean OFF, so the variable cannot be enabled by accident.
run_diff off0    PGEN_ENVELOPE_DUMP_ALL=0
run_diff offnull PGEN_ENVELOPE_DUMP_ALL=
ok "A4a =0 leaves the cap in force"     "$(field "$W/off0.env.json"    divergences)" "40"
ok "A4b =<empty> leaves the cap in force" "$(field "$W/offnull.env.json" divergences)" "40"

# ── A5/A6 THE TWO-SIDED RATCHET — a ceiling is only a ratchet if BOTH directions fire ────────────
# ⛔ Run against a COPY of the gate script; the arms edit the ceiling, and editing the tracked one
# would leave the repository's own gate mis-set if this probe were interrupted.
sv_total="$(field "$W/dumpall.env.json" divergence_total)"
declared="$(bash -c "source $GATE >/dev/null 2>&1; envelope_divergence_ceiling systemverilog" 2>/dev/null \
            || sed -n 's/^ *systemverilog) echo \([0-9]*\) ;;/\1/p' "$GATE" | head -1)"
ok "A5 the tracked ceiling EQUALS the measured total" "$declared" "$sv_total"

total=$((total+1))
if grep -q 'REGRESSED' "$GATE" && grep -qi 'lower the ceiling\|BELOW' "$GATE"; then
  pass=$((pass+1)); printf '  ✓ %-52s\n' "A6 the ratchet has both a rise and a fall arm"
else printf '  ✗ %-52s — one direction is missing\n' "A6 the ratchet has both a rise and a fall arm"; fi

# ── A7 the extra divergence is NAMEABLE, which is the whole point of the escape ───────────────────
total=$((total+1))
n="$(python3 - <<'PY'
import json
d=json.load(open('rust/target/es13c2i_cap_probe/dumpall.env.json'))['envelope_differential']
print(sum(1 for x in d['divergences']
          if (x.get('arm1') or {}).get('kind') == 'semantic_annotation_inline'
          and (x.get('arm2') or {}).get('kind') == 'semantic_annotation'))
PY
)"
if [ "$n" -ge 1 ] 2>/dev/null; then
  pass=$((pass+1))
  printf '  ✓ %-52s %s instances\n' "A7 the inline-annotation class is nameable" "$n"
else printf '  ✗ %-52s (%s)\n' "A7 the inline-annotation class is nameable" "$n"; fi

echo
echo "ENVELOPE-CAP: $pass/$total as declared"
[ "$pass" = "$total" ]
