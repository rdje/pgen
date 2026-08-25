#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.46 (d) — RE-AUDIT EVERY LIVE SITE UNDER EVERY PROFILE.
#
# WHY THIS FILE EXISTS
# --------------------
# `.46`'s routing evidence audited the live SystemVerilog sites under ONE profile (`verilog_2005`)
# and published two claims off that single reading:
#
#   (1) "LIVE POPULATION IN THE SV GRAMMAR TODAY: 3 sites"
#   (2) "None produces an over-acceptance today — `p::b` correctly REJECTS under `verilog_2005`,
#        because nothing else in the v2005 grammar can consume `::`"
#
# Claim (2) is a CENSUS claim ("nothing else can consume X") and no census stood behind it. This
# script runs the census, and enumerates the sites and the PROFILE UNIVERSE by tool rather than by
# hand — the universe is where the single-profile reading actually went wrong, because it is FIVE
# states wide, not one:
#
#     unspecified · sv_2017 · sv_2023 · verilog_2005 · an UNDECLARED spelling
#
# `unspecified` and `undeclared` are runtime states no `@profiles` list mentions, and the second of
# them is the state in which EVERY gated rule is absent at once.
#
# ⛔ A SITE OR A GRAMMAR THIS SCRIPT CANNOT MEASURE IS REPORTED AS `not-measured`, NEVER AS CLEAN.
# `generated/` is untracked, so on a fresh clone the emitted-artifact leg has nothing to read; it
# says so instead of scoring the leg green from an absent file.
#
# Usage: bash docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/site_profile_audit.sh

set -uo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT_DIR" || exit 2

PIPE="rust/target/debug/ast_pipeline"
SV="grammars/systemverilog.ebnf"
EMITTED="generated/systemverilog_parser.rs"
WORK="rust/target/es46d_audit"

if [[ ! -x "$PIPE" ]]; then
    printf 'REFUSED: %s is absent — build it first:\n' "$PIPE" >&2
    printf '  cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline\n' >&2
    exit 2
fi

rm -rf "$WORK"; mkdir -p "$WORK"
trap 'rm -rf "$WORK"' EXIT
pass=0; fail=0; notmeasured=0

expect() { # expect <label> <actual> <wanted>
    if [[ "$2" == "$3" ]]; then printf '  ✅ %-64s %s\n' "$1" "$2"; pass=$((pass+1))
    else printf '  ❌ %-64s %s (wanted %s)\n' "$1" "$2" "$3"; fail=$((fail+1)); fi
}
verdict() { # verdict <file> <profile-or-empty>
    if [[ -z "$2" ]]; then
        "$PIPE" "$SV" --interpret-parse "$1" 2>&1 | grep -o 'accepted=[a-z]*' | head -1
    else
        "$PIPE" "$SV" --interpret-parse "$1" --grammar-profile "$2" 2>&1 | grep -o 'accepted=[a-z]*' | head -1
    fi
}

echo "SITE × PROFILE AUDIT — ENGINE-UNIVERSAL-SERVICES.46 (d)"
echo

# ── [1] THE PROFILE UNIVERSE, DERIVED ─────────────────────────────────────────────────────────────
# The universe is read off the grammar, never typed here: a profile added to `systemverilog.ebnf`
# must make this arm fail rather than quietly go unaudited.
echo "[1] THE PROFILE UNIVERSE — derived from the grammar, not typed into this script"
LINT="$WORK/lint.txt"
"$PIPE" "$SV" --lint-grammar > "$LINT" 2>&1
DECLARED="$(grep -o 'profiles=\[[^]]*\]' "$LINT" | head -1 | sed 's/profiles=\[//; s/\]//; s/"//g; s/, /,/g')"
printf '    declared @profiles universe: %s\n' "$DECLARED"
expect "the declared universe is exactly the three SV dialects" "$DECLARED" "sv_2017,sv_2023,verilog_2005"
echo "    ⭐ the RUNTIME universe is two states WIDER — both are un-nameable in a @profiles list:"
echo "       · unspecified  — no --grammar-profile at all ⇒ NO gate is applied ⇒ every rule live"
echo "       · undeclared   — an unknown spelling passes through UN-COERCED (the pinned"
echo "                        'profile_alias_unknown_passthrough' combinator) ⇒ EVERY gated rule absent"
echo

# ── [2] THE SITE CENSUS — the lint's SOURCE sites against the SHIPPED parser's EMITTED wrappers ───
echo "[2] SITE CENSUS — source sites (lint) reconciled against emitted sites (shipped parser)"
NEG="$(grep -o 'profile_gated_negative_lookaheads=[0-9]*' "$LINT" | grep -o '[0-9]*$' | head -1)"
POS="$(grep -o 'profile_gated_positive_lookaheads=[0-9]*' "$LINT" | grep -o '[0-9]*$' | head -1)"
printf '    lint notes: negative=%s positive=%s   (a note is one (rule, path, PROFILE) triple)\n' "$NEG" "$POS"
expect "negative notes"  "$NEG" "7"
expect "positive notes"  "$POS" "1"
grep -o "rule '[a-z_]*' is LIVE under profile '[^']*'" "$LINT" \
    | sed "s/rule '//; s/' is LIVE under profile '/  ×  /; s/'$//" | sort > "$WORK/notes.txt"
echo "    the (rule × profile) pairs the notes cover:"
sed 's/^/      /' "$WORK/notes.txt"
DISTINCT_RULES="$(sed 's/  ×.*//' "$WORK/notes.txt" | sort -u | wc -l | tr -d ' ')"
expect "distinct RULES carrying a note" "$DISTINCT_RULES" "5"

if [[ -r "$EMITTED" ]]; then
    EMIT_N="$(grep -c 'profile_gate_bypass_applies(&' "$EMITTED")"
    printf '    emitted bypass wrappers in %s: %s\n' "$EMITTED" "$EMIT_N"
    # ⭐ 5 emitted against 4 source NEGATIVE sites is CORRECT and is the reconciliation this arm
    # exists to state: codegen INLINES `simple_identifier_no_scope` into `scope_free_identifier`'s
    # first alternative, so that one source site is emitted TWICE. The lint counts SOURCE sites; the
    # parser emits per INLINED site. A reader comparing the two numbers without this line reads a
    # missing repair.
    expect "emitted wrappers (4 source negative sites, one inlined twice)" "$EMIT_N" "5"
    grep -o 'profile_gate_bypass_applies(&\[[^]]*\], *[a-z]*)' "$EMITTED" \
        | sed 's/profile_gate_bypass_applies(//' | sort | uniq -c | sed 's/^/      /'
    echo "      ⇒ every wrapper carries when_undeclared=true, and four carry the vacuous profile"
    echo "        \"verilog_2005\"; the fifth (non_keyword_identifier) carries NONE, because it is"
    echo "        the SELECTION site — vacuous ONLY where both dialect lists vanish at once."
else
    printf '    %-64s not-measured (%s absent — generated/ is untracked)\n' "emitted bypass wrappers" "$EMITTED"
    notmeasured=$((notmeasured + 1))
fi
echo

# ── [3] THE CENSUS BEHIND THE ROUTING CLAIM — "nothing else can consume `::`" ─────────────────────
echo "[3] THE '::' CONSUMER CENSUS — the claim '.46' published by hand, now run"
"$PIPE" "$SV" --generate-parser --output "$WORK/throwaway.rs" --dump-gen-ast "$WORK/gen_ast.json" >/dev/null 2>&1
"$PIPE" "$SV" --dump-rule-profiles "$WORK/rule_profiles.json" >/dev/null 2>&1
if [[ -r "$WORK/gen_ast.json" && -r "$WORK/rule_profiles.json" ]]; then
    python3 - "$WORK/gen_ast.json" "$WORK/rule_profiles.json" <<'PY' > "$WORK/colon_census.txt"
import json, re, sys
gen = json.load(open(sys.argv[1])); prof = json.load(open(sys.argv[2]))
gt = gen['grammar_tree']; rules = prof['rules']
def terminals(n, out):
    if isinstance(n, dict):
        for k, v in n.items():
            if k == 'Token' and isinstance(v, list) and len(v) == 2:
                kind = v[0].get('String'); val = v[1].get('String')
                if kind in ('quoted_string', 'regex'): out.append((kind, val))
            terminals(v, out)
    elif isinstance(n, list):
        for x in n: terminals(x, out)
def consumes(kind, val):
    if kind == 'quoted_string': return val.startswith(':')
    try:
        m = re.match(val, '::'); return bool(m) and m.end() > 0
    except re.error:
        return None            # UNKNOWN — reported, never scored clean
hits, unknown, total = {}, [], 0
for r, b in gt.items():
    ts = []; terminals(b, ts); total += len(ts)
    for kind, val in ts:
        c = consumes(kind, val)
        if c is None: unknown.append((r, kind, val))
        elif c: hits.setdefault(r, []).append((kind, val))
print(f"terminals_scanned={total} consumers={len(hits)} uncompilable={len(unknown)}")
for r in sorted(hits):
    p = rules.get(r, {})
    print(f"  {r}|{p.get('declared_profiles')}|{p.get('satisfiable_under')}|{hits[r]}")
PY
    sed 's/^/    /' "$WORK/colon_census.txt"
    SCANNED="$(grep -o 'terminals_scanned=[0-9]*' "$WORK/colon_census.txt" | grep -o '[0-9]*$')"
    CONSUMERS="$(grep -o 'consumers=[0-9]*' "$WORK/colon_census.txt" | grep -o '[0-9]*$')"
    UNCOMP="$(grep -o 'uncompilable=[0-9]*' "$WORK/colon_census.txt" | grep -o '[0-9]*$')"
    expect "every terminal in the grammar was scanned, none skipped" "$UNCOMP" "0"
    expect "rules with a terminal that can consume a leading ':'" "$CONSUMERS" "2"
    echo "    ⛔⛔ THE ROUTING CLAIM IS UNSUPPORTED — AND NOTE THAT IS *NOT* THE SAME AS REFUTED."
    echo "        '.46' published *\"nothing else in the v2005 grammar can consume ::\"* as the REASON"
    echo "        the live sites were safe, and no census stood behind it. Run: exactly TWO rules"
    echo "        carry a terminal that can consume a leading ':', and one of them — \`colon\`, UNGATED"
    echo "        — is LIVE under verilog_2005."
    echo "    ⚠️ THAT DOES NOT REFUTE THE SENTENCE, and this arm must not claim it does. \`colon\`"
    echo "        consumes ONE ':'; consuming '::' that way needs TWO adjacent \`colon\`s at the"
    echo "        guarded position, and whether the grammar admits that is UNMEASURED here."
    echo "    ⇒ what IS established: the claim rested on a census nobody ran, and the nearest hazard"
    echo "      is live. The margin was an argument rather than a measurement. Moot for soundness"
    echo "      only because (c) shipped and the guard is now enforced."
else
    printf '    %-64s not-measured (gen-AST dump failed)\n' "':' consumer census"
    notmeasured=$((notmeasured + 1))
fi
echo

# ── [4] THE BEHAVIOURAL MATRIX — every site, every profile, on the REAL grammar ───────────────────
echo "[4] BEHAVIOURAL MATRIX — real grammar, all five runtime profile states"
mkdir -p "$WORK/in"
printf 'module m;\n  reg foo;\nendmodule\n'              > "$WORK/in/ctl_plain.sv"
printf 'module m;\n  reg class;\nendmodule\n'            > "$WORK/in/sel_class.sv"
printf 'module m;\n  reg macromodule;\nendmodule\n'      > "$WORK/in/sel_macromodule.sv"
printf 'module m;\n  assign x = p::b;\nendmodule\n'      > "$WORK/in/nar_simple_ident.sv"
printf 'module m;\n  assign x = \\foo ;\nendmodule\n'    > "$WORK/in/ctl_escaped.sv"
printf 'module m;\n  assign x = \\foo ::b;\nendmodule\n' > "$WORK/in/nar_escaped_ident.sv"
printf 'module m;\n  initial $display(1);\nendmodule\n'  > "$WORK/in/ctl_systf.sv"
printf 'module m;\n  initial $unit::f();\nendmodule\n'   > "$WORK/in/nar_systf.sv"
printf 'module m;\n  C::T x;\nendmodule\n'               > "$WORK/in/pos_class_scope.sv"
printf 'module m;\nendmodule\n'                          > "$WORK/in/ctl_bare_module.sv"

printf '    %-24s %-9s %-9s %-9s %-9s %-9s\n' "input" "unspec" "v2005" "sv_2017" "sv_2023" "undecl"
printf '    %-24s %-9s %-9s %-9s %-9s %-9s\n' "------------------------" "--------" "--------" "--------" "--------" "--------"
for f in "$WORK"/in/*.sv; do
    row=""
    for P in "" verilog_2005 sv_2017 sv_2023 pgen_audit_undeclared_profile; do
        v="$(verdict "$f" "$P")"; row="$row ${v#accepted=}"
    done
    # shellcheck disable=SC2086
    set -- $row
    printf '    %-24s %-9s %-9s %-9s %-9s %-9s\n' "$(basename "$f" .sv)" "$1" "$2" "$3" "$4" "$5"
done
echo
echo "    the arms that DECIDE, pinned:"
expect "NARROWING  p::b            rejects under verilog_2005"    "$(verdict "$WORK/in/nar_simple_ident.sv" verilog_2005)" accepted=false
expect "NARROWING  p::b            accepts under sv_2017"         "$(verdict "$WORK/in/nar_simple_ident.sv" sv_2017)"      accepted=true
expect "NARROWING  p::b            accepts under sv_2023"         "$(verdict "$WORK/in/nar_simple_ident.sv" sv_2023)"      accepted=true
expect "NARROWING  \\foo ::b        rejects under verilog_2005"    "$(verdict "$WORK/in/nar_escaped_ident.sv" verilog_2005)" accepted=false
expect "NARROWING  \$unit::f()      rejects under verilog_2005"    "$(verdict "$WORK/in/nar_systf.sv" verilog_2005)"       accepted=false
expect "POSITIVE   C::T x          rejects under verilog_2005"    "$(verdict "$WORK/in/pos_class_scope.sv" verilog_2005)"  accepted=false
expect "SELECTION  reg class;      accepts under verilog_2005"    "$(verdict "$WORK/in/sel_class.sv" verilog_2005)"        accepted=true
expect "SELECTION  reg class;      rejects under sv_2017"         "$(verdict "$WORK/in/sel_class.sv" sv_2017)"             accepted=false
expect "SELECTION  reg macromodule; rejects under verilog_2005"   "$(verdict "$WORK/in/sel_macromodule.sv" verilog_2005)"  accepted=false
echo "    the two runtime states '.46' never audited:"
expect "UNSPECIFIED — no gate at all, so reg class; is RESERVED"  "$(verdict "$WORK/in/sel_class.sv" "")"                  accepted=false
expect "UNSPECIFIED — no gate at all, so p::b PARSES"             "$(verdict "$WORK/in/nar_simple_ident.sv" "")"           accepted=true
expect "UNDECLARED  — even the bare module rejects"               "$(verdict "$WORK/in/ctl_bare_module.sv" pgen_audit_undeclared_profile)" accepted=false
expect "UNDECLARED  — and the plain control rejects too"          "$(verdict "$WORK/in/ctl_plain.sv" pgen_audit_undeclared_profile)"       accepted=false
echo "    ⇒ UNDER AN UNDECLARED PROFILE 427 OF 1 537 RULES VANISH AT ONCE and the minimal legal"
echo "      module no longer parses. The four sentinel-profile notes in [2] are therefore STATICALLY"
echo "      real and BEHAVIOURALLY inert: the guard they describe is enforced, on a profile that"
echo "      accepts no module. Reported rather than dropped — a note is cheaper than the census a"
echo "      future reader would otherwise have to redo."
echo

# ── [5] THE MONOTONICITY READING, INCLUDING ITS MEASURED EXCEPTION ────────────────────────────────
echo "[5] WHAT THE MATRIX SAYS ABOUT THE INVARIANT '.46' SHIPPED"
echo "    ⛔⛔ '.46' published the invariant UNQUALIFIED — *\"gating a rule out of a profile must only"
echo "        ever REMOVE strings from the language, never ADD them\"* — and this grammar VIOLATES it"
echo "        deliberately, by measurement, at the SELECTION site:"
echo "            reg class;   unspecified (NO gate) = reject      verilog_2005 (gated) = ACCEPT"
echo "        Applying the verilog_2005 gate ADDED that string. That is not a defect: a SELECTION"
echo "        dispatcher swaps one dialect's alternative for another's, and two dialects' languages"
echo "        are incomparable by construction."
echo "    ⭐ THE INVARIANT THE REPAIR ACTUALLY IMPLEMENTS is the narrower one, and it is the one that"
echo "      survives this audit:  GATING MAY REMOVE A PRODUCTION; IT MAY NEVER DELETE A CONSTRAINT."
echo "      A negative lookahead derives nothing, so emptying its body is constraint deletion — the"
echo "      inversion. Swapping a live alternative for a sibling is production replacement — intended."
echo

printf 'RESULT: pass=%d fail=%d not-measured=%d\n' "$pass" "$fail" "$notmeasured"
[[ $fail -eq 0 ]] || exit 1
exit 0
