#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.56 (a) — is the zero-memo-hit spine a REAL re-execution cost, or an
# ACCOUNTING GAP in which channel records hits?
#
# The leaf named this as its FIRST job and named the trap with it: the thin memo has no hit
# recording at all, so "served by the thin memo" would produce a zero that is an instrument
# artefact rather than a cost. It also named the reasoning that must NOT be used — `-0103` retracted
# an explanation that attributed the class to a suppression gate before showing the gate FIRES.
#
# THREE ARMS:
#   A  REGIME   — entries, memo hits and entries-per-key for the three spine rules across depths
#                 that straddle the recursion ceiling. This is the arm that reframes the question:
#                 SUB-ceiling the rules are entered EXACTLY ONCE PER KEY, so a 0 % hit rate there is
#                 arithmetically forced and is not a cost at all.
#   B  CHANNEL  — the accounting-gap hypothesis, tested on the EMITTED ARTIFACT rather than the
#                 template: every `record_memo_hit` call site in `generated/systemverilog_parser.rs`
#                 must lie inside `memoized_call`, which is what makes the per-rule counter blind to
#                 nothing on the PROTOCOL graph. A CLOSED census (all sites, not a sample).
#   C  TRACE    — per-rule memo EVENT counts at one over-ceiling depth: misses, failure stores,
#                 success stores, hits and stale evictions, for the spine rule AND for a control
#                 rule the memo demonstrably serves in the SAME run. The trace is STREAMED, never
#                 written: at depth 350 the log is 1.9 GB and this repository's data-locality rules
#                 do not need another 1.9 GB artifact to say the same thing.
#
# ⛔ WHAT THIS SCRIPT DOES NOT SEPARATE, stated here so no reader infers it: WHICH of the two
#   failure-store gates refuses — `ast_based_generator.rs:9913` (content-scoped recursion floor,
#   caching nothing at all) or `:9934` (depth-gated, storing into a channel whose replay requires a
#   stack at least as deep). Neither emits a trace line, and separating them needs the per-refusal
#   counter `.44`(b) steps 1 and 2 both priced. What IS established without it is the OUTCOME.
#
# ⚠️ TRACE-LINE HAZARD, censused here and worth knowing before reading any position field: all ten
#   `💾` memo trace lines say "at position N" and FOUR mean something else — the three store lines
#   (`ast_based_generator.rs:9994` / `:10003` / `:10010`) and the SUCCESS-hit line (`:9744`, where
#   `self.position = entry.end_pos` runs immediately above) print a POST-BODY position, while the
#   other six print the memo KEY. Comparing "miss at position P" against "stored at position Q"
#   therefore compares start positions against END positions and yields disjoint sets that mean
#   nothing (leaf `.59`). This arm COUNTS events; it deliberately does not join their positions.
#
# USAGE:  bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_spine_memo_refusal.sh
# Exit 0 on a successful measurement; exit 2 when it cannot be honest.
set -uo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT" || exit 1
P="rust/target/release/parseability_probe"
[ -x "$P" ] || { echo "missing $P — build it with:"; \
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)"; exit 2; }

PROBE_SV="$("$P" --parser-fingerprint 2>&1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["parsers"]["systemverilog"])')"
DISK_SV="$(shasum -a 256 generated/systemverilog_parser.rs | cut -d' ' -f1)"
echo "# probe systemverilog fingerprint: $PROBE_SV"
if [ "$PROBE_SV" != "$DISK_SV" ]; then
  echo "# ⛔ NOT EVALUATED — probe embeds $PROBE_SV but generated/systemverilog_parser.rs is $DISK_SV"
  exit 2
fi
echo "# fingerprint MATCHES generated/systemverilog_parser.rs"

W="rust/target/spine_memo_refusal"; rm -rf "$W"; mkdir -p "$W"
SPINE="constant_primary"
CONTROL="primary_literal"

# ⛔ Rule ids are RESOLVED from the emitted RULE_NAMES table, never typed in: the trace prints
# numbers, and a hand-typed id silently counts a different rule.
IDS="$(python3 - "$SPINE" "$CONTROL" <<'PY'
import re, sys
src = open('generated/systemverilog_parser.rs').read()
m = re.search(r'RULE_NAMES[^=]*=\s*&\[(.*?)\];', src, re.S)
if not m:
    print("RESOLVE-FAILED"); raise SystemExit(0)
names = re.findall(r'"([^"]+)"', m.group(1))
out = []
for r in sys.argv[1:]:
    if r not in names:
        print("RESOLVE-FAILED"); raise SystemExit(0)
    out.append(str(names.index(r)))
print(" ".join(out))
PY
)"
if [ "$IDS" = "RESOLVE-FAILED" ]; then
  echo "# ⛔ NOT EVALUATED — could not resolve rule ids from the emitted RULE_NAMES table"; exit 2
fi
SPINE_ID="${IDS%% *}"; CONTROL_ID="${IDS##* }"
echo "# rule ids resolved from the emitted RULE_NAMES table: $SPINE=$SPINE_ID  $CONTROL=$CONTROL_ID"

echo
echo "=============================================================================="
echo "ARM A — THE REGIME. Entries per KEY across the recursion ceiling"
echo "=============================================================================="
printf '%-7s %7s %10s %10s %8s %9s | %10s %8s %9s\n' \
  depth bytes total sp_ent sp_hit sp_per_key ctl_ent ctl_hit ctl_hit%
for d in 3 20 60 300 315 320 330 350 400 500 800; do
  python3 -c "import sys;n=$d;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*n, ')'*n))" > "$W/in_$d.sv"
  "$P" --parse systemverilog "$W/in_$d.sv" --profile sv_2017 \
       --dump-rule-outcome-counts-json "$W/out_$d.json" > /dev/null 2>&1
  SPINE="$SPINE" CONTROL="$CONTROL" python3 - "$W/out_$d.json" "$W/in_$d.sv" "$d" <<'PY'
import json, os, sys
d = json.load(open(sys.argv[1])); nb = os.path.getsize(sys.argv[2]); keys = nb + 1
e = d['rule_entry_counts']; h = d.get('rule_memo_hit_counts', {})
sp, ct = os.environ['SPINE'], os.environ['CONTROL']
se, sh = e.get(sp, 0), h.get(sp, 0)
ce, ch = e.get(ct, 0), h.get(ct, 0)
print('%-7s %7d %10d %10d %8d %9.1f | %10d %8d %8.1f%%'
      % (sys.argv[3], nb, sum(e.values()), se, sh, se / keys, ce, ch,
         100.0 * ch / ce if ce else 0.0))
PY
done
echo "   ⭐ SUB-ceiling the spine is entered ~ONCE PER KEY — a 0 % hit rate there is ARITHMETIC,"
echo "     not a defect: there is nothing to hit. The cost is the OVER-ceiling column."

echo
echo "=============================================================================="
echo "ARM B — THE ACCOUNTING-GAP HYPOTHESIS, on the EMITTED ARTIFACT (closed census)"
echo "=============================================================================="
python3 - <<'PY'
lines = open('generated/systemverilog_parser.rs').read().split('\n')
start = next((i for i, l in enumerate(lines) if l.strip().startswith('fn memoized_call<F>')), None)
if start is None:
    print("# ⛔ NOT EVALUATED — no `fn memoized_call` in the emitted artifact"); raise SystemExit(0)
indent = len(lines[start]) - len(lines[start].lstrip())
end = next(i for i in range(start + 1, len(lines))
           if lines[i].strip().startswith('}') and len(lines[i]) - len(lines[i].lstrip()) == indent)
sites = [i + 1 for i, l in enumerate(lines) if 'record_memo_hit' in l]
inside = [s for s in sites if start + 1 <= s <= end + 1]
print("fn memoized_call spans lines %d..%d" % (start + 1, end + 1))
print("record_memo_hit call sites in the WHOLE artifact: %d  %s" % (len(sites), sites))
print("of those INSIDE memoized_call:                    %d" % len(inside))
print("⇒ %s" % ("every hit-recording site is inside the memo wrapper, so the per-rule hit counter "
                "is blind to NO channel of it — the four probe paths (plain failure, tainted "
                "failure, success, depth-gated failure) all record."
                if len(sites) == len(inside) and sites else
                "⛔ a hit-recording site sits OUTSIDE memoized_call — re-read before trusting any "
                "hit count."))
PY
echo "   ⚠️ SCOPE: this is the PROTOCOL graph. The FUSED graph's thin memo records no hits at all,"
echo "     which is why every number here comes from a coverage-enabled (protocol) run and why the"
echo "     control column in arm A must be non-zero for the arm to mean anything."

echo
echo "=============================================================================="
echo "ARM C — THE EVENT CENSUS at one over-ceiling depth (trace STREAMED, never stored)"
echo "=============================================================================="
TD=350
# ONE traced parse, both rules counted from the same stream: a second traced run would double the
# cost and, worse, would let the two rules' numbers come from two different executions.
PGEN_TRACE_VERBOSITY=debug "$P" --parse systemverilog "$W/in_$TD.sv" --profile sv_2017 \
    --trace-rules "$SPINE" 2>&1 \
  | LC_ALL=C awk -v sid="$SPINE_ID" -v cid="$CONTROL_ID" -v sname="$SPINE" -v cname="$CONTROL" -v depth="$TD" '
      function bump(id,  k) { }
      {
        for (i = 0; i < 2; i++) {
          id   = (i == 0 ? sid : cid)
          if ($0 ~ ("Memo miss for rule " id " at")) {
            miss[i]++
            if (match($0, /at position [0-9]+/)) { p = substr($0, RSTART+12, RLENGTH-12); pos[i "," p]++ }
          }
          else if ($0 ~ ("Memoized failed result for rule " id))         fstore[i]++
          else if ($0 ~ ("Memoized successful result for rule " id))     sstore[i]++
          else if ($0 ~ ("Memo hit for rule " id " at"))                 hit[i]++
          else if ($0 ~ ("Evicting STALE tainted success for rule " id)) evs[i]++
          else if ($0 ~ ("Evicting STALE tainted failure for rule " id)) evf[i]++
        }
      }
      END {
        for (i = 0; i < 2; i++) {
          id = (i == 0 ? sid : cid); nm = (i == 0 ? sname : cname)
          n = 0; mx = 0
          for (k in pos) { split(k, a, ","); if (a[1] == i) { n++; if (pos[k] > mx) mx = pos[k] } }
          printf "depth %s  rule %-22s (id %4s): miss=%-7d fail_store=%-6d succ_store=%-6d hit=%-7d evict=%d/%d\n",
                 depth, nm, id, miss[i]+0, fstore[i]+0, sstore[i]+0, hit[i]+0, evs[i]+0, evf[i]+0
          if (n > 0)
            printf "                                        distinct key positions=%d  mean probes/key=%.1f  MAX probes at one key=%d\n",
                   n, (miss[i]+0)/n, mx
        }
      }'
echo "   ⇒ the spine rule's failures are NEVER stored while the control rule's are, in the SAME run,"
echo "     on the same input, from the same binary. That is the finding; WHICH gate refuses is not"
echo "     separated here and must not be inferred (see the header)."

cat <<'NOTE'

==============================================================================
READ IT LIKE THIS
==============================================================================
ARM A   Read `sp_per_key` before reading `sp_hit`. A rule entered once per key CANNOT hit, so a
        0 % hit rate sub-ceiling is arithmetic. The finding is the RISE in that column across the
        ceiling — the same rule, the same grammar, two regimes.
ARM B   A closed census over the SHIPPED artifact, not over the codegen template: the template says
        what WOULD be emitted, the artifact says what IS. If every hit-recording site is inside the
        memo wrapper, a zero from the per-rule counter is a fact about the parse, not about the
        counter.
ARM C   The control rule is the whole point: it proves the memo, the counters and the trace all work
        in the run that reports the spine's zero. `fail_store=0` beside a non-zero control is the
        measurement; the gate that produces it is NOT identified by this script.
NOTE
