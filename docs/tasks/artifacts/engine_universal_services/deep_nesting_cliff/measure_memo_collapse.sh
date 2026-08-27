#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.44 (b) STEP 1 — the CENSUS the leaf owes before any emission change.
#
# THE QUESTION (`.44` (d), verbatim): "why does SystemVerilog still spend ~72 % re-descending when
# those channels exist -- are they not reached on SV's cascade shape, keyed too narrowly to collapse
# sibling alternatives, or reached and working with the residue genuinely elsewhere?"
#
# WHAT THIS MEASURES, per depth, on the SHIPPED release probe:
#   * MEMO STATS  -> entries per channel (plain / store-tainted / depth-gated) + ceiling rejections
#   * trace       -> HITS per channel + misses          (streamed, never written to disk)
#   * call counts -> total rule CALLS
#   * derived     -> calls-per-distinct-key, and hits-per-entry for each memo channel
#
# ⛔⛔ THE OBSERVER CHANGES THE EXECUTION, AND THAT IS A MEASURED FACT, NOT A CAVEAT.
#   `ast_based_generator.rs:2252` computes
#       bare_parse = !coverage_enabled && !logger_enabled && !counters_observed && !report_memo_stats_enabled()
#   so EVERY instrument here (memo stats, --trace, --dump-rule-call-counts) DISABLES the fused
#   `cascade_*` graph and measures the PROTOCOL graph instead. The emitted comment says so:
#   "fused internal rules skip their memo lane, so memo stats are truthful only on the protocol graph".
#   ⇒ These numbers describe the protocol path. The script therefore ALSO times the plain
#   (fused, shipping) run beside the instrumented one so the reader can see whether the two cost the
#   same. MEASURED 2026-08-27 at depth 800, 3 runs each: plain 12.12/12.40/12.42 s,
#   memo-stats 12.27/12.15/12.14 s, call-counts 12.51/12.25/12.25 s -- indistinguishable, so the
#   fused graph buys nothing on the over-ceiling path and the protocol numbers are representative
#   OF THE COST. They are not thereby proven representative of the fused graph's INTERNAL shape.
#
# ⚠️ TRACING IS THE EXPENSIVE ARM: depth 350 emits ~4.9 GB of trace text. It is streamed through
#   `awk` and never stored. Depths past ~450 are impractical; the script says so rather than hanging.
#
# USAGE:  measure_memo_collapse.sh [depth ...]      default: 300 325 350 375 400
# Exit 0 always: this is a MEASUREMENT, not a gate.
set -uo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT" || exit 1
P="rust/target/release/parseability_probe"
[ -x "$P" ] || { echo "missing $P — build it with:"; \
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)"; exit 2; }

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

W="rust/target/memo_collapse"; rm -rf "$W"; mkdir -p "$W"
DEPTHS=("$@"); [ "${#DEPTHS[@]}" -eq 0 ] && DEPTHS=(300 325 350 375 400)

printf '\n%-6s %10s %10s %10s %10s %10s %10s %9s %9s %9s\n' \
  depth calls ceil-rej ent-plain ent-depth hit-plain hit-depth miss cal/key h/e-depth

for d in "${DEPTHS[@]}"; do
  IN="$W/in_$d.sv"
  python3 -c "import sys;n=$d;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*n, ')'*n))" > "$IN"

  # --- entries per channel + ceiling rejections (protocol graph) ---
  STATS="$(PGEN_REPORT_MEMO_STATS=1 "$P" --parse systemverilog "$IN" --profile sv_2017 2>&1 \
           | grep -m1 '^=== MEMO STATS')"
  read -r ENT_OK ENT_FAIL ENT_TAINT ENT_DEPTH CEIL <<<"$(python3 - <<PY
import re
s = """$STATS"""
m = re.search(r'MEMO STATS: (\d+) success entries \((\d+) tainted\) \+ (\d+) cached failures \((\d+) store-tainted, (\d+) depth-gated\) = (\d+) total, \d+ subtree-nodes, \d+ distinct rules, (\d+) depth-ceiling rejections', s)
print(*( (m.group(1), m.group(3), m.group(4), m.group(5), m.group(7)) if m else ('NA',)*5 ))
PY
)"
  if [ "$ENT_OK" = "NA" ]; then echo "  depth $d: NOT EVALUATED — no MEMO STATS headline"; continue; fi
  ENT_PLAIN=$(( ENT_FAIL - ENT_TAINT - ENT_DEPTH ))
  ENT_TOTAL=$(( ENT_OK + ENT_FAIL ))

  # --- total rule CALLS (protocol graph) ---
  CALLS="$("$P" --parse systemverilog "$IN" --profile sv_2017 --dump-rule-call-counts 3000 2>&1 \
      | tr '\r' '\n' | sed 's/\x1b\[[0-9;]*[A-Za-z]//g' | python3 -c '
import re,sys
blocks=sys.stdin.read().split("=== Rule call counts")
rows={}
for line in blocks[-1].split("\n"):
    m=re.match(r"\s*(\w+)\s+([\d,]+)\s*$", line)
    if m: rows[m.group(1)]=int(m.group(2).replace(",",""))
print(sum(rows.values()) if rows else "NA")')"

  # --- HITS per channel + misses, STREAMED (a depth-350 trace is ~4.9 GB; none of it is stored) ---
  read -r HIT_DEPTH HIT_PLAIN MISS <<<"$(PGEN_TRACE_VERBOSITY=debug "$P" --parse systemverilog "$IN" \
      --profile sv_2017 --trace 2>&1 | LC_ALL=C awk '
      /cached depth-gated failure/ {dg++; next}
      /cached failure/            {pf++; next}
      /Memo miss for rule/        {ms++; next}
      END {printf "%d %d %d\n", dg+0, pf+0, ms+0}')"

  CK=$(python3 -c "print('%.1f' % ($CALLS/$ENT_TOTAL) if '$CALLS'!='NA' and $ENT_TOTAL else 'NA')" 2>/dev/null || echo NA)
  HE=$(python3 -c "print('%.2f' % ($HIT_DEPTH/$ENT_DEPTH) if $ENT_DEPTH else 0.0)")
  printf '%-6s %10s %10s %10s %10s %10s %10s %9s %9s %9s\n' \
    "$d" "$CALLS" "$CEIL" "$ENT_PLAIN" "$ENT_DEPTH" "$HIT_PLAIN" "$HIT_DEPTH" "$MISS" "$CK" "$HE"
done

# ---------------------------------------------------------------------------------------------
# THE OBSERVER-EFFECT ARM. Proven BEHAVIOURALLY, by an instrument that does not itself set the flag:
# `/usr/bin/sample` is attached to a plain run and to a `PGEN_REPORT_MEMO_STATS=1` run of the SAME
# binary on the SAME input, and the fused `cascade_match_*` frames are counted in each. It is its own
# RED control -- the same instrument must report a non-zero count in one mode and exactly zero in the
# other, so it can be seen to disagree with itself.
# MEASURED 2026-08-27, depth 800, 5 s: plain 12 562 cascade frames / 42 720 parse frames;
#                                      memo-stats **0** cascade frames / 58 184 parse frames.
if command -v /usr/bin/sample >/dev/null 2>&1; then
  echo
  echo "== observer-effect arm (depth 800): does an instrument delete the fused graph? =="
  OW="$W/observer"; mkdir -p "$OW"
  python3 -c "import sys;n=800;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*n, ')'*n))" > "$OW/in.sv"
  for mode in plain memostats; do
    if [ "$mode" = plain ]; then
      "$P" --parse systemverilog "$OW/in.sv" --profile sv_2017 >/dev/null 2>&1 &
    else
      PGEN_REPORT_MEMO_STATS=1 "$P" --parse systemverilog "$OW/in.sv" --profile sv_2017 >/dev/null 2>&1 &
    fi
    pid=$!; sleep 2; /usr/bin/sample "$pid" 5 -file "$OW/$mode.txt" >/dev/null 2>&1; wait "$pid"
    printf '   %-10s cascade_match_ frames=%-7s  parse_ frames=%s\n' "$mode" \
      "$(grep -c 'cascade_match_' "$OW/$mode.txt")" "$(grep -c '::parse_' "$OW/$mode.txt")"
  done
  echo "   ⇒ a ZERO in the memostats row means every number above describes the PROTOCOL graph."
else
  echo; echo "== observer-effect arm: NOT EVALUATED — /usr/bin/sample absent =="
fi

cat <<'NOTE'

READ IT LIKE THIS:
  cal/key    = total rule CALLS / total distinct memo entries. It is the collapse ratio: how many
               calls the memo has to absorb per key it stores. It rising with depth is the blow-up.
  h/e-depth  = HITS per depth-gated ENTRY. A memo entry that is replayed ~once has paid for exactly
               one avoided re-descent; the plain channel's ratio is the control to compare it against.
NOTE
