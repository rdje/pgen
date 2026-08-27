#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.44 (b) STEP 2 — the CENSUS the leaf owes before any emission change.
#
# THE TWO QUESTIONS (`.44` (b) step 1, verbatim): "which choice sites issue the 18.9 M calls, and
# whether the engine's existing `QUANT-SITE-CENSUS` / `DEGENERACY-CENSUS` / `PREFIX2-CENSUS`
# machinery (`TOOLBOX.md` 5.x) already prices that elision on SystemVerilog."
#
# FOUR ARMS, in the order a reader should trust them:
#   A  STATIC   — the four attempt-elision gates on SystemVerilog + their blocker histograms.
#   B  CONTROL  — the SAME instrument on regex. It MUST report large non-zero numbers, or arm A's
#                 zero is a dead instrument rather than a fact about SystemVerilog. This arm is the
#                 RED/GREEN control: the census must be seen able to disagree with itself.
#   C  DYNAMIC  — the census joined to REAL over-ceiling outcome counts (`--fusibility-outcome-counts`),
#                 so the elision surface is priced on THIS workload and not on a regular corpus.
#   D  MEMO     — the per-rule memo-HIT counter, which prices the OUTCOME half of the two failure-store
#                 suppression paths WITHOUT the codegen change (b) step 1 priced for the input half.
#
# ⛔ THE OBSERVER CHANGES THE EXECUTION AND THAT IS PRIOR ART, NOT A CAVEAT (`TOOLBOX.md` §2.1/§3.5):
#   the outcome dump enables transactional coverage, so every arm-C/D parse runs the PROTOCOL graph.
#   That is the correct graph here — it is the one whose `memoized_call` is under study, and the
#   fused region carries zero `memoized_call` references. `-0101` measured the two graphs within 8 %
#   at depth 300 and indistinguishable at depth 800, which licenses reading these as THE COST.
#
# USAGE:  census_attempt_elision.sh [depth ...]      default: 300 800
# Exit 0 always: this is a MEASUREMENT, not a gate. It refuses (exit 2) only when it cannot be honest.
set -uo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT" || exit 1
P="rust/target/release/parseability_probe"
A="rust/target/debug/ast_pipeline"
[ -x "$P" ] || { echo "missing $P — build it with:"; \
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)"; exit 2; }
[ -x "$A" ] || { echo "missing $A — build it with:  (cd rust && cargo build --bin ast_pipeline)"; exit 2; }

# ⛔ The fingerprint is CHECKED, not printed and ignored: a probe embedding a different parser than
# the one on disk attributes cost to code that would not ship.
PROBE_SV="$("$P" --parser-fingerprint 2>&1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["parsers"]["systemverilog"])')"
DISK_SV="$(shasum -a 256 generated/systemverilog_parser.rs | cut -d' ' -f1)"
echo "# probe systemverilog fingerprint: $PROBE_SV"
if [ "$PROBE_SV" != "$DISK_SV" ]; then
  echo "# ⛔ NOT EVALUATED — probe embeds $PROBE_SV but generated/systemverilog_parser.rs is $DISK_SV"
  exit 2
fi
echo "# fingerprint MATCHES generated/systemverilog_parser.rs"

W="rust/target/attempt_elision_census"; rm -rf "$W"; mkdir -p "$W"
DEPTHS=("$@"); [ "${#DEPTHS[@]}" -eq 0 ] && DEPTHS=(300 800)
GATES='^(FUSIBILITY-CENSUS|CHOICE-SITE-CENSUS|DEGENERACY-CENSUS|PREFIX2-CENSUS|FIRSTK-CENSUS|QUANT-SITE-CENSUS|CASCADE-CENSUS)'

echo
echo "=============================================================================="
echo "ARM A — the four ATTEMPT-ELISION gates on SystemVerilog, and WHY they refuse"
echo "=============================================================================="
"$A" grammars/systemverilog.ebnf --report-fusibility-census > "$W/sv_static.txt" 2>&1 || true
grep -E "$GATES" "$W/sv_static.txt"
echo
echo "-- blocker histograms (the WHY: read the count against the site total on the line above) --"
sed -n '/^DEGENERACY-CENSUS/,/^CASCADE-CENSUS/p' "$W/sv_static.txt" | grep -E '^ +[0-9]+ ' | sort -u

echo
echo "=============================================================================="
echo "ARM B — RED/GREEN CONTROL: the SAME instrument on regex must report NON-ZERO"
echo "=============================================================================="
"$A" grammars/regex.ebnf --report-fusibility-census > "$W/rgx_static.txt" 2>&1 || true
grep -E "$GATES" "$W/rgx_static.txt"
echo "   ⇒ non-zero here + zero in arm A means arm A measured SystemVerilog, not a dead instrument."

for d in "${DEPTHS[@]}"; do
  IN="$W/in_$d.sv"
  python3 -c "import sys;n=$d;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*n, ')'*n))" > "$IN"
  "$P" --parse systemverilog "$IN" --profile sv_2017 \
       --dump-rule-outcome-counts-json "$W/out_$d.json" > "$W/parse_$d.log" 2>&1

  echo
  echo "=============================================================================="
  echo "ARM C — the elision surface priced on the REAL depth-$d workload"
  echo "=============================================================================="
  "$A" grammars/systemverilog.ebnf --report-fusibility-census \
       --fusibility-outcome-counts "$W/out_$d.json" 2>&1 \
    | grep -E '^(OUTCOME-SHARE|DEGENERACY-EXPOSURE|PREFIX2-EXPOSURE|QUANT-EXPOSURE|CASCADE-EXPOSURE)'

  echo
  echo "=============================================================================="
  echo "ARM D — where the calls actually go, and which rules the memo NEVER serves (depth $d)"
  echo "=============================================================================="
  IN="$IN" python3 - "$W/out_$d.json" <<'PY'
import json, os, sys, collections
d = json.load(open(sys.argv[1]))
e = d['rule_entry_counts']; h = d.get('rule_memo_hit_counts', {})
tot = sum(e.values()); toth = sum(h.values())
nbytes = os.path.getsize(os.environ['IN']); keys = nbytes + 1
print("input %d B  =>  at most %d distinct (rule,position) keys PER RULE" % (nbytes, keys))
print("rule-method entries : %12d   (the call population)" % tot)
print("memo HITS           : %12d   (%.1f%% — served from the memo, returns Backtrack)"
      % (toth, 100.0 * toth / tot))
print("committed           : %12d   discarded %d (%.5f%%)"
      % (d['total_committed'], tot - d['total_committed'], 100.0 * (tot - d['total_committed']) / tot))
zero = [(r, c) for r, c in e.items() if h.get(r, 0) == 0]
zc = sum(c for _, c in zero)
print()
print("rules the memo NEVER serves: %d of %d rules, carrying %d entries (%.1f%% of ALL calls)"
      % (len(zero), len(e), zc, 100.0 * zc / tot))
b = collections.Counter(); be = collections.Counter()
for r, c in e.items():
    hr = 100.0 * h.get(r, 0) / c
    k = "0%" if hr == 0 else ("<50%" if hr < 50 else ("50-90%" if hr < 90 else
        ("90-99.9%" if hr < 99.9 else ">=99.9%")))
    b[k] += 1; be[k] += c
print()
print("%-10s %6s %14s %8s   <- a BIMODAL split is the finding: a rule is served or it is not"
      % ("hit-rate", "rules", "entries", "share"))
for k in ["0%", "<50%", "50-90%", "90-99.9%", ">=99.9%"]:
    print("%-10s %6d %14d %7.1f%%" % (k, b[k], be[k], 100.0 * be[k] / tot))
print()
print("top zero-hit rules — `calls/key` is the PIGEONHOLE bound: with at most %d keys, a rule called" % keys)
print("this many times per key and still reporting ZERO hits has never had a result RETAINED.")
print("%-38s %12s %8s %11s" % ("rule", "calls", "hits", "calls/key"))
for r, c in sorted(zero, key=lambda x: -x[1])[:8]:
    print("%-38s %12d %8d %11.1f" % (r, c, h.get(r, 0), c / keys))
PY
done

cat <<'NOTE'

==============================================================================
READ IT LIKE THIS
==============================================================================
ARM A/B  A gate reporting 0 is only a FACT about the grammar once arm B shows the same gate
         reporting non-zero elsewhere. Read each blocker count against its own site total: a
         blocker firing on 100 % of sites is one grammar-level property, not N site verdicts.
ARM C    `*-EXPOSURE: … entries=0` prices the elision on the workload itself. `OUTCOME-SHARE`'s
         `on_encodable` vs `on_structural` split says whether the shape-encodable lanes can matter
         here at all; `ceiling≈1.00x` means they cannot.
ARM D    `calls/key` is a COUNTING argument, not an inference: a rule cannot be called hundreds of
         times per available key, report zero hits, and still be having its results retained. It is
         how the failure-store suppression is priced from the OUTCOME side with no codegen change.
NOTE
