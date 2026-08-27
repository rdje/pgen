#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.44 (b) STEP 3 — the census that PRICES the layout-aware dispatch
# DESIGN before a line of it is emitted, and the CONTROL that tests its premise.
#
# WHAT STEP 2 LEFT: "the existing elision machinery prices SystemVerilog at EXACTLY ZERO on all four
# gates, for ONE reason — `terminals skip leading layout (R2 raw-byte peek unsound)`. Step 3 owes the
# layout-aware dispatch design." Step 2 measured that zero against ONE control, regex, which differs
# from SystemVerilog in TWO ways at once (layout policy AND grammar scale) — the exact shape step 1
# already recorded as un-attributable ("an instrument that changes TWO things at once cannot
# attribute an effect to either of them"). This script adds the missing controls.
#
# FIVE ARMS:
#   A  HEADROOM   — the four gates on the TRACKED SystemVerilog grammar vs. a copy differing by ONE
#                   line (`@whitespace_sensitive: { terminals: true, regex_tokens: true }`). That
#                   flip is exactly the hypothetical "a raw byte peek at the dispatch point is
#                   sound", so the delta IS the layout-aware headroom. The one-line diff is
#                   ASSERTED, not assumed — the arm refuses if the copy differs by anything else.
#   B  CONTROL    — the same instrument on regex, which must report large NON-ZERO numbers, or
#                   arm A's tracked-side zero is a dead instrument rather than a grammar fact.
#   C  WORKLOAD   — both facets joined to REAL depth-800 over-ceiling outcome counts, so the
#                   elision surface is priced on the plateau workload and not on a regular corpus.
#   D  SITE       — the dominant site's per-branch first-byte verdict at the workload's own dispatch
#                   byte `(`, with THREE attribution bounds (exclusive-at-site, named-only-by-refuted,
#                   named-by-any) so the reader can see the attribution's width instead of one number.
#   E  BAR        — `.44` (d)'s ≤2× ratio measured across FIVE families. This is the falsification
#                   leg: it tests whether the elision machinery is what makes a reject cheap, using
#                   parsers this leaf did not build. json and rtl_const_expr are layout-SKIPPING
#                   (all four gates read 0, exactly like SystemVerilog) — if they still reject at
#                   ≈1×, the machinery is NOT the discriminator and arm A's headroom, however large,
#                   is not by itself the fix.
#
# ⛔ THE OBSERVER CHANGES THE EXECUTION AND THAT IS PRIOR ART, NOT A CAVEAT (`TOOLBOX.md` §2.1/§3.5):
#   the outcome dump enables transactional coverage, so arm C/D parses run the PROTOCOL graph.
#   `-0101` measured the two graphs within 8 % at depth 300 and indistinguishable at 800, which
#   licenses reading these as THE COST but not as the fused graph's internal SHAPE. ⭐ Arm E runs
#   PLAIN (no dump, no counter, no env) — it is the FUSED graph an embedder actually executes.
#
# USAGE:  bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_layout_aware_dispatch.sh
# Exit 0 always on a successful measurement; exit 2 when it cannot be honest (missing binary,
# fingerprint mismatch, or a probe grammar that is not the tracked grammar plus exactly one line).
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

W="rust/target/layout_aware_dispatch_census"; rm -rf "$W"; mkdir -p "$W/tracked" "$W/layout_aware"
GATES='^(FUSIBILITY-CENSUS|CHOICE-SITE-CENSUS|DEGENERACY-CENSUS|PREFIX2-CENSUS|FIRSTK-CENSUS|QUANT-SITE-CENSUS)'

# ⛔ The census names the grammar after the FILE STEM and refuses an outcome-counts join whose
# grammar name differs. Both copies therefore keep the stem `systemverilog`; they are told apart by
# their DIRECTORY, and every arm prints which one it is reading.
cp grammars/systemverilog.ebnf "$W/tracked/systemverilog.ebnf"
FLIP='@whitespace_sensitive: { terminals: true, regex_tokens: true }'
awk -v line="$FLIP" 'NR==196{print line}{print}' grammars/systemverilog.ebnf > "$W/layout_aware/systemverilog.ebnf"

echo
echo "=============================================================================="
echo "ARM A — HEADROOM: the four gates under the TRACKED layout facet vs. the"
echo "        LAYOUT-AWARE hypothetical (the tracked grammar + exactly one line)"
echo "=============================================================================="
# The probe grammar is only evidence if it is provably the tracked grammar plus the flip.
DIFF_ADDED="$(diff "$W/tracked/systemverilog.ebnf" "$W/layout_aware/systemverilog.ebnf" | grep -c '^>')"
DIFF_REMOVED="$(diff "$W/tracked/systemverilog.ebnf" "$W/layout_aware/systemverilog.ebnf" | grep -c '^<')"
DIFF_LINE="$(diff "$W/tracked/systemverilog.ebnf" "$W/layout_aware/systemverilog.ebnf" | sed -n 's/^> //p')"
echo "probe-grammar diff vs tracked: +${DIFF_ADDED} -${DIFF_REMOVED}   added: ${DIFF_LINE}"
if [ "$DIFF_ADDED" != "1" ] || [ "$DIFF_REMOVED" != "0" ] || [ "$DIFF_LINE" != "$FLIP" ]; then
  echo "# ⛔ NOT EVALUATED — the probe grammar is not the tracked grammar plus exactly the flip line."
  exit 2
fi

"$A" "$W/tracked/systemverilog.ebnf"      --report-fusibility-census \
     --fusibility-census-json "$W/tracked.json"      > "$W/tracked.txt"      2>&1 || true
"$A" "$W/layout_aware/systemverilog.ebnf" --report-fusibility-census \
     --fusibility-census-json "$W/layout_aware.json" > "$W/layout_aware.txt" 2>&1 || true

echo
echo "-- TRACKED facet (terminals skip layout; regex tokens skip layout) --"
grep -E "$GATES" "$W/tracked.txt"
echo
echo "-- LAYOUT-AWARE facet (the hypothetical: skip layout ONCE at the dispatch point, then peek) --"
grep -E "$GATES" "$W/layout_aware.txt"
echo
echo "-- WHY the tracked side reads zero, and what is LEFT once the layout reason is removed --"
echo "   (FIRSTK first: its gate is the shared level-1 admission predicate and nothing else, so it"
echo "    isolates the layout fact alone. The other three carry co-firing blockers.)"
for facet in tracked layout_aware; do
  for gate in FIRSTK-CENSUS DEGENERACY-CENSUS PREFIX2-CENSUS QUANT-SITE-CENSUS; do
    case "$gate" in
      FIRSTK-CENSUS)     end='^QUANT-SITE-CENSUS' ;;
      DEGENERACY-CENSUS) end='^PREFIX2-CENSUS' ;;
      PREFIX2-CENSUS)    end='^FIRSTK-CENSUS' ;;
      QUANT-SITE-CENSUS) end='^CASCADE-CENSUS' ;;
    esac
    echo "   [$facet] $gate residual blockers:"
    sed -n "/^$gate/,/$end/p" "$W/$facet.txt" | grep -E '^ +[0-9]+  ' | sed 's/^/     /'
  done
done

echo
echo "=============================================================================="
echo "ARM B — RED/GREEN CONTROL: the SAME instrument on regex must report NON-ZERO"
echo "=============================================================================="
"$A" grammars/regex.ebnf --report-fusibility-census > "$W/rgx.txt" 2>&1 || true
grep -E "$GATES" "$W/rgx.txt"
echo "   ⇒ non-zero here + zero on arm A's tracked side means arm A measured the grammar, not a"
echo "     dead instrument. ⚠️ It does NOT license reading regex as a control for arm E — see there."

echo
echo "=============================================================================="
echo "ARM C — the elision surface priced on the REAL depth-800 over-ceiling workload"
echo "=============================================================================="
IN="$W/in_800.sv"
python3 -c "import sys;n=800;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*n, ')'*n))" > "$IN"
"$P" --parse systemverilog "$IN" --profile sv_2017 \
     --dump-rule-outcome-counts-json "$W/out_800.json" > "$W/parse_800.log" 2>&1
for facet in tracked layout_aware; do
  echo "-- [$facet] --"
  "$A" "$W/$facet/systemverilog.ebnf" --report-fusibility-census \
       --fusibility-outcome-counts "$W/out_800.json" 2>&1 \
    | grep -E '^(OUTCOME-SHARE|DEGENERACY-EXPOSURE|PREFIX2-EXPOSURE|QUANT-EXPOSURE)' | sed 's/^/   /'
done

echo
echo "=============================================================================="
echo "ARM D — the DOMINANT SITE, branch by branch, at the workload's own dispatch byte"
echo "=============================================================================="
python3 - "$W/layout_aware.json" "$W/out_800.json" <<'PY'
import json, sys, collections
cen = json.load(open(sys.argv[1])); out = json.load(open(sys.argv[2]))
e = out['rule_entry_counts']; h = out.get('rule_memo_hit_counts', {})
tot = sum(e.values()); BYTE = 0x28   # '(' — 800 of the input's 1633 bytes
sites = [s for s in cen['choice_sites'] if s['top_level']]

def site_split(s):
    ref, adm = set(), set(); nref = 0
    for b in s['branch_verdicts']:
        fb = b['first_bytes']
        refuted = fb is not None and BYTE not in fb
        (ref if refuted else adm).update(b['direct_refs'])
        nref += 1 if refuted else 0
    return ref - adm, nref

def site_kill(s):
    """Entries a level-1 guard at THIS site would elide, capped by what the site can ISSUE.

    The static half is the entries on rules named ONLY by this site's '('-refuted branches (no
    cross-site double counting). The cap is the DYNAMIC half and it is what makes the ranking
    honest: a site cannot elide more attempts than it makes, and it makes at most one attempt per
    refuted branch per entry of its own rule. Without the cap the ranking is blind to execution —
    it tied four clone sites (the sv_2017/sv_2023 profile twins and their LR-elimination seed
    copies) that name identical rules, and then picked the seed copy, which is entered 4 558 times
    against the twin's 601 658. The ~1.0x per-attempt check below is what exposed that.
    """
    excl, nref = site_split(s)
    return min(sum(e.get(r, 0) for r in excl), e.get(s['rule'], 0) * nref)

# ⛔ The dominant site is DERIVED by the quantity this arm is ABOUT — how many entries a level-1
# guard here would elide — never by hand and never by "which rule takes the most entries". The
# first cut of this script maximised the latter and picked `non_keyword_identifier`, a 2-branch
# site that is 100 % memo-served: the biggest rule is not the biggest tournament.
ranked = sorted(sites, key=site_kill, reverse=True)
print("top-6 top-level sites by elidable entries at byte '(' (the DERIVED ranking):")
for s in ranked[:6]:
    ran = e.get(s['rule'], 0)
    print("   %-46s %-6s branches=%-3d elidable=%10d (%4.1f%%)  entries_on_rule=%d%s"
          % (s['rule'], s['site'], s['branches'], site_kill(s), 100.0 * site_kill(s) / tot,
             ran, "" if ran else "   <- NEVER ENTERED on this input"))
# The static ranking cannot tell four CLONES apart: the sv_2017/sv_2023 profile twins and their
# LR-elimination seed copies name the same referenced rules, so they tie exactly. Only the one the
# parse actually enters can own the cost — attributing it statically would over-count it 4x. The
# filter is therefore PRINTED, never silent.
executed = [s for s in ranked if e.get(s['rule'], 0) > 0]
dropped = len(ranked) - len(executed)
print("   ⛔ %d of %d top-level sites were NEVER ENTERED on this input (profile twins and"
      % (dropped, len(ranked)))
print("      LR-seed clones tie exactly on the static ranking); the dominant site is taken from")
print("      the %d that WERE entered, so the cost is attributed to code that ran." % len(executed))
if not executed:
    print("   # NOT EVALUATED — no top-level site was entered on this input.")
    raise SystemExit(0)
dom = executed[0]
print()
print("dominant EXECUTED top-level site: %s %s — %d branches, %d entries on the rule itself"
      % (dom['rule'], dom['site'], dom['branches'], e.get(dom['rule'], 0)))
adm = ref = und = 0
print("   %-3s %-46s %-16s %12s" % ("br", "direct refs", "at byte '('", "refs entries"))
for b in dom['branch_verdicts']:
    fb = b['first_bytes']; refs = ','.join(b['direct_refs'])
    ent = sum(e.get(x, 0) for x in b['direct_refs'])
    if fb is None:      v = "UNDECIDED"; und += 1
    elif BYTE in fb:    v = "admits";    adm += 1
    else:               v = "REFUTED";   ref += 1
    print("   b%-2d %-46s %-16s %12d" % (b['index'], refs[:46], v, ent))
print("   ⇒ %d admit, %d REFUTED on the first byte, %d undecided" % (adm, ref, und))

excl, nref = site_split(dom)
n1_raw = sum(e.get(r, 0) for r in excl)
n1 = min(n1_raw, e.get(dom['rule'], 0) * nref)
print()
print("BOUND 1 (this site only, no cross-site double counting): %d rules named ONLY by its"
      % len(excl))
print("        '('-refuted branches carry %d entries = %.1f%% of all %d"
      % (n1_raw, 100.0 * n1_raw / tot, tot))
print("        (issue cap %d entries x %d refuted branches = %d ⇒ BOUND 1 = %d = %.1f%%)."
      % (e.get(dom['rule'], 0), nref, e.get(dom['rule'], 0) * nref, n1, 100.0 * n1 / tot))
site_entries = e.get(dom['rule'], 0)
print("        ⭐ ATTRIBUTION IS ARITHMETIC, NOT ASSERTION: each is entered ≈ONCE per site entry")
print("        (site entries = %d), which is what a per-ATTEMPT entry looks like:" % site_entries)
for r in sorted(excl, key=lambda x: -e.get(x, 0))[:6]:
    if e.get(r, 0):
        print("          %-44s %9d  = %.3f× the site" % (r, e[r], e[r] / site_entries))

named = collections.defaultdict(lambda: [0, 0])
for s in sites:
    for b in s['branch_verdicts']:
        fb = b['first_bytes']
        i = 0 if (fb is not None and BYTE not in fb) else 1
        for r in b['direct_refs']:
            named[r][i] += 1
only_ref = [r for r, (a, o) in named.items() if a > 0 and o == 0]
any_ref  = [r for r, (a, o) in named.items() if a > 0]
n2 = sum(e.get(r, 0) for r in only_ref); n3 = sum(e.get(r, 0) for r in any_ref)
print()
print("BOUND 2 (grammar-wide, rules named ONLY by '('-refuted top-level branches): %d rules, "
      "%d entries = %.1f%%" % (len(only_ref), n2, 100.0 * n2 / tot))
print("BOUND 3 (grammar-wide, rules named by ANY '('-refuted top-level branch): %d rules, "
      "%d entries = %.1f%%  ⚠️ UPPER — a rule reachable from an admitting branch too is counted here"
      % (len(any_ref), n3, 100.0 * n3 / tot))
print()
print("⛔ WHAT NONE OF THE THREE BOUNDS COVER: an entry issued from a SEQUENCE element or a NESTED")
print("   Or site. `direct_refs` here spans top-level Or branches only, which is the population a")
print("   level-1 dispatch guard can refute. Bounds 1-2 are therefore not strict lower bounds on")
print("   the rule's entries; they are the share ATTRIBUTABLE to guardable branches on this input.")

print()
print("THE SECOND MULTIPLIER, ON THE SAME SITE — repetition, which dispatch elision does not touch:")
print("   %-46s %10s %10s %8s %10s" % ("rule", "entries", "memo hits", "hit%", "calls/key"))
nbytes = len(open(sys.argv[2].replace('out_800.json', 'in_800.sv'), 'rb').read()); keys = nbytes + 1
watch = [dom['rule']] + [x for x in ('constant_primary', 'constant_expression_operand',
                                     'declaration_identifier', 'non_keyword_identifier',
                                     'lparen', 'trivia') if x in e]
for r in dict.fromkeys(watch):
    c = e.get(r, 0); hh = h.get(r, 0)
    print("   %-46s %10d %10d %7.1f%% %10.1f"
          % (r, c, hh, 100.0 * hh / c if c else 0.0, c / keys))
print("   ⇒ input is %d B ⇒ at most %d distinct (rule,position) keys. A rule entered hundreds of"
      % (nbytes, keys))
print("     times per key with ZERO hits is being RE-DERIVED, not probed — and it re-runs its whole")
print("     tournament every time. The two multipliers COMPOSE: breadth × repetition.")
PY

echo
echo "=============================================================================="
echo 'ARM E — .44 (d) THE <=2x BAR ACROSS FIVE FAMILIES (plain runs: the FUSED graph)'
echo "=============================================================================="
echo "⛔ THE FALSIFICATION LEG. json and rtl_const_expr are layout-SKIPPING: all four elision gates"
echo "   read 0 on them exactly as on SystemVerilog. If they reject at ≈1× anyway, the elision"
echo "   machinery is NOT what makes a reject cheap, and arm A's headroom is not by itself the fix."
printf '%-16s %8s %10s %8s %10s %8s  %s\n' family accepted cost reject cost ratio verdict
# spec = family : template : open+close pair : profile : depth ladder
for spec in "json:%s1%s:[]::400 800 1200 1600 2400" \
            "regex:%sa%s:()::200 400 600 800 1200" \
            "rtl_const_expr:%s1%s:()::100 200 300 400 600" \
            "systemverilog:module m; assign x = %s1%s; endmodule:():sv_2017:200 300 320 400 800" \
            "vhdl:entity e is end entity; architecture a of e is begin s <= %s1%s; end architecture;:():vhdl_1076_2019:200 300 320 400 800"; do
  fam="${spec%%:*}"; rest="${spec#*:}"
  tmpl="${rest%%:*}"; rest="${rest#*:}"
  pair="${rest%%:*}"; rest="${rest#*:}"
  prof="${rest%%:*}"; ladder="${rest#*:}"
  op="${pair:0:1}"; cl="${pair:1:1}"
  best_n=""; best_t=""; rej_n=""; rej_t=""
  for n in $ladder; do
    python3 -c "import sys;n=$n;sys.stdout.write('''$tmpl''' % ('$op'*n, '$cl'*n))" > "$W/e_in.txt"
    st=$(python3 -c 'import time;print(time.monotonic())')
    if [ -n "$prof" ]; then "$P" --parse "$fam" "$W/e_in.txt" --profile "$prof" >/dev/null 2>&1
    else                    "$P" --parse "$fam" "$W/e_in.txt"                    >/dev/null 2>&1; fi
    rc=$?
    el=$(python3 -c "import time;print(f'{time.monotonic()-$st:.3f}')")
    if [ "$rc" = "0" ]; then best_n="$n"; best_t="$el"; else
      rej_n="$n"; rej_t="$el"   # keep the DEEPEST (saturated) reject — the plateau, not the cliff
    fi
  done
  if [ -z "$best_t" ] || [ -z "$rej_t" ]; then
    printf '%-16s %8s %10s %8s %10s %8s  %s\n' "$fam" "${best_n:--}" "${best_t:--}" "${rej_n:--}" "${rej_t:--}" "-" \
      "NOT EVALUATED (ladder never crossed the cliff)"
    continue
  fi
  read -r ratio verdict <<<"$(python3 -c "
r = $rej_t / $best_t
print('%.1fx' % r, 'PASS(<=2x)' if r <= 2.0 else 'RED')")"
  printf '%-16s %8s %9ss %8s %9ss %8s  %s\n' "$fam" "$best_n" "$best_t" "$rej_n" "$rej_t" "$ratio" "$verdict"
done
echo "   (accepted = deepest depth ON THE LADDER that parsed; reject = the DEEPEST over-ceiling"
echo "    depth, i.e. the saturated plateau, which is what a pathological input reaches.)"
echo "   WARNING: a ladder brackets the accept anchor from BELOW, and a shallower anchor is a"
echo "      CHEAPER anchor, so every ratio printed here is an UPPER bound. A RED row therefore"
echo "      cannot be a ladder artefact, and a PASS row would only improve with a finer ladder."

cat <<'NOTE'

==============================================================================
READ IT LIKE THIS
==============================================================================
ARM A   The delta between the two facets IS the layout-aware headroom, because the two grammars
        differ by one asserted line. A gate that stays 0 on BOTH sides (degeneracy) is blocked by
        something layout-awareness cannot fix; read its blocker histogram to see by what.
ARM B   Arm A's tracked-side zero is a grammar fact only because this arm shows the same gates
        reporting large numbers elsewhere from the same binary.
ARM C   `PREFIX2-EXPOSURE` moving off zero prices the D1 lane on the plateau workload itself.
ARM D   Bound 1 has no cross-site double counting and its attribution is CHECKED by the ≈1.0×
        entries-per-site-entry ratio, not asserted. Bounds 2/3 bracket the grammar-wide figure.
        The repetition table is the term dispatch elision CANNOT reach — read it before concluding
        that a bigger elision surface closes the ratio.
ARM E   Two families with ZERO elision surface reject at ≈1×. That is the control step 2 lacked,
        and it is what turns "the machinery prices SV at zero" from a diagnosis into a symptom.
NOTE
