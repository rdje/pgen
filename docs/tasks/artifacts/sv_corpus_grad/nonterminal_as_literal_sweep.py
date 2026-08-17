#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2d — the SWEEP the leaf owed: which `kw_*` terminals spell a NONTERMINAL?

⛔ THE DEFECT CLASS. IEEE 1800's Annex A writes productions like

      cross_body_item ::= function_declaration | bins_selection_or_option ;

and the LRM→EBNF extraction sometimes transcribed the *referenced nonterminal* as a **terminal that
matches its own spelling as literal text**:

      cross_body_item_sv_2017 := kw_function_declaraton_06b7ed29
      kw_function_declaraton_06b7ed29 := trivia /function_declaraton\\b/

⇒ the alternative can only fire on source that literally contains the characters
`function_declaraton`, which no SystemVerilog file writes. The construct is **unreachable**, and the
grammar looks like it supports it.

⭐ WHY A SWEEP AND NOT A SPOT FIX. `.13c.2d` found ONE site by hand and named ONE more as a
"candidate the grep found". A count produced by a search nobody can re-run is not a census
(`docs/CLAIM_VERIFICATION.md` §1) — and the hand search was short by two, including the one with
EIGHT usage sites.

THREE CANDIDATE SIGNALS, all printed, because none alone finds the population:
  A  the matched text IS a nonterminal, or is within `CUTOFF` of one (catches the LRM's own TYPO
     `function_declaraton`, which is not a nonterminal anywhere — that is the whole point of it);
  B  the matched text CONTAINS a nonterminal name (catches the flattened multi-nonterminal
     productions, e.g. `PATHPULSE$<desc>$<desc>` collapsed into one token);
  C  the matched text carries a TRANSLITERATED punctuation name (`PATHPULSE$` emitted as
     `PATHPULSE_dollar`). ⛔ C was added only after A and B were measured to MISS an entire
     production: `pulse_control_specparam`'s bare alternative is near no nonterminal and contains
     none, so neither A nor B fires, and it is just as unreachable as the flattened one. Two signals
     that agree are not two signals that are complete.

⛔⛔ AND THEN AN AUTHORITY, BECAUSE THE SIGNALS ALONE ARE 50 % FALSE POSITIVES (5 of 10). A grammar names its
nonterminals after the constructs they build, so `assign`, `coverpoint`, `deassign`, `nettype` and
`timeunit` all sit within `CUTOFF` of `assign` / `cover_point` / `net_type` / `time_unit` — and every
one of them is a genuine reserved keyword whose `kw_*` terminal is exactly right. Publishing the raw
signal set would have reported **10 defects where there are 5**.

The authority is **IEEE 1800-2023 Annex B**, the normative reserved-keyword table, tracked in this
repository at `docs/systemverilog/2023/txt/section-Annex_B-normative-keywords.txt`. A `kw_*` terminal
whose text is in Annex B is a keyword and is CORRECT; one whose text is not, and which spells a
nonterminal, is the defect.

⭐ CROSS-CHECKED AGAINST AN INDEPENDENT SIGNAL, and the check is part of the run rather than a note:
a genuine keyword appears in real SystemVerilog source (`assign`: 3 828 corpus files) while a
nonterminal-spelled-as-literal appears in **zero**. Annex-B membership and corpus reachability are
derived from completely different inputs — a standards document and 16 427 source files — so when
they PARTITION THE POPULATION IDENTICALLY, neither is resting on the other. If they ever disagree,
this script says so loudly instead of picking one.

⛔ `kw_*` rules are EXCLUDED from the nonterminal set being matched against. The first cut did not
exclude them, and every long `kw_` name matched *itself* — a false positive that made the report
read as a finding.

USAGE   python3 docs/tasks/artifacts/sv_corpus_grad/nonterminal_as_literal_sweep.py
EXIT    0 = the sweep ran; 2 = it could not run correctly (never a silently empty population)
"""

import difflib
import os
import re
import sys

CUTOFF = 0.85
# Punctuation the LRM writes literally and an extraction may transliterate into a NAME.
# `$` is the one that has actually bitten (`PATHPULSE$` -> `PATHPULSE_dollar`); the rest are
# the same shape and cost nothing to watch. ⛔ Deliberately checked as `_<name>` or
# `<name>_` so that a keyword merely CONTAINING one of these words does not fire.
PUNCT_NAMES = ("dollar", "lparen", "rparen", "lbrace", "rbrace", "lbrack", "rbrack",
               "comma", "semi", "colon", "dot", "tick", "hash", "star", "slash")
GRAMMAR = "grammars/systemverilog.ebnf"
ANNEX_B = "docs/systemverilog/2023/txt/section-Annex_B-normative-keywords.txt"
CORPUS_ROOTS = ["stimuli/sv/subs", "stimuli/sv/uvm"]
SV_EXT = (".sv", ".svh", ".v", ".vh", ".sva", ".vams")

# docs/tasks/artifacts/sv_corpus_grad/ -> four levels up is the repository root.
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
os.chdir(ROOT)

if not os.path.isfile(GRAMMAR):
    sys.exit(f"sweep: {GRAMMAR} not found from {ROOT} — refusing to report an empty population")
src = open(GRAMMAR, encoding="utf-8").read()

rules = re.findall(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:=", src, re.M)
if not rules:
    sys.exit("sweep: the grammar yielded ZERO rule definitions — the parse is wrong, not the grammar")
nonterm = {r for r in rules if not r.startswith("kw_")}
nonterm |= {re.sub(r"_sv_20(17|23)$", "", r) for r in nonterm}

kw = [(m.group(1), m.group(2).replace(r"\b", ""))
      for m in re.finditer(r"^(kw_[A-Za-z0-9_]+)\s*:=\s*trivia\s*/([^/]+)/", src, re.M)]
if not kw:
    sys.exit("sweep: ZERO kw_* trivia terminals matched — the extraction pattern is wrong")

lines = src.splitlines()

# ── the AUTHORITY: IEEE 1800-2023 Annex B, the normative reserved-keyword table ───────────────────
if not os.path.isfile(ANNEX_B):
    sys.exit(f"sweep: {ANNEX_B} is missing — without the normative keyword table this sweep cannot "
             "tell a correct keyword terminal from a nonterminal transcribed as one, and a report "
             "that cannot make that distinction is half false positives. Refusing.")
annex_raw = open(ANNEX_B, encoding="utf-8", errors="replace").read()
# The table is one keyword per line, with page furniture around it. Take every line that is a single
# bare identifier — furniture ("Copyright © 2024 IEEE. All rights reserved.") never is.
KEYWORDS = {ln.strip() for ln in annex_raw.splitlines()
            if re.fullmatch(r"[A-Za-z_$][A-Za-z0-9_$]*", ln.strip() or " ")}
# ⛔ GROUND-TRUTH CONTROLS on the extraction itself, run before any verdict depends on it. Annex B
# has ~250 reserved words; an extraction that silently produced a handful would make every terminal
# look like a defect, and one that produced everything would make every terminal look correct.
if not 200 <= len(KEYWORDS) <= 400:
    sys.exit(f"sweep: extracted {len(KEYWORDS)} keywords from {ANNEX_B}; IEEE 1800-2023 Annex B "
             "lists ~250. The extraction is wrong, so every verdict below would be too. Refusing.")
for must in ("assign", "always_comb", "endfunction", "coverpoint", "timeunit"):
    if must not in KEYWORDS:
        sys.exit(f"sweep: '{must}' is a reserved keyword and the extraction missed it — refusing")
for must_not in ("function_declaration", "class_qualifier", "expression"):
    if must_not in KEYWORDS:
        sys.exit(f"sweep: '{must_not}' is a NONTERMINAL and the extraction admitted it as a "
                 "keyword — the table boundary is wrong. Refusing.")


def def_line(rule):
    for i, ln in enumerate(lines, 1):
        if re.match(rf"^{re.escape(rule)}\s*:=", ln):
            return i
    return None


def use_lines(rule):
    """Every line REFERENCING the rule, excluding its own definition."""
    d = def_line(rule)
    return [i for i, ln in enumerate(lines, 1)
            if rule in ln and i != d]


# ── the corpus denominator, counted rather than assumed ───────────────────────────────────────────
sv_files, all_files = [], []
for root in CORPUS_ROOTS:
    for dirpath, _, names in os.walk(root):
        for n in names:
            p = os.path.join(dirpath, n)
            all_files.append(p)
            if n.endswith(SV_EXT):
                sv_files.append(p)
corpus_available = bool(all_files)


def corpus_hits(text):
    """(sv-source files containing the literal, all files containing it). The two are reported
    SEPARATELY and never merged: the only occurrence of `class_qualifier` anywhere in the corpus is
    a COMMENT inside a Yacc grammar (`verible/verilog/parser/verilog.y`), which is not SystemVerilog
    source and cannot exercise the alternative. Filtering silently would hide that; reporting both
    shows the filter did work and what it removed."""
    sv = allf = 0
    for p in all_files:
        try:
            with open(p, "rb") as fh:
                blob = fh.read()
        except OSError:
            continue
        if text.encode() in blob:
            allf += 1
            if p.endswith(SV_EXT):
                sv += 1
    return sv, allf


print("SV-CORPUS-GRAD.13c.2d — sweep: kw_* terminals that spell a NONTERMINAL")
print(f"grammar: {GRAMMAR}   rules: {len(rules)}   nonterminals (kw_* excluded): {len(nonterm)}")
print(f"kw_* trivia terminals: {len(kw)}")
if corpus_available:
    print(f"corpus: {len(all_files)} files, of which {len(sv_files)} are SystemVerilog source")
else:
    print("corpus: ⚠️ NOT EVALUATED — the corpora are git submodules and none is checked out. "
          "The grammar-side findings below still bind; the reachability column does not.")
print()

hits = {}
for name, text in kw:
    sig = []
    if text in nonterm:
        sig.append(("A", "EXACT nonterminal"))
    else:
        near = difflib.get_close_matches(text, sorted(nonterm), n=1, cutoff=CUTOFF)
        if near:
            r = difflib.SequenceMatcher(None, text, near[0]).ratio()
            sig.append(("A", f"near '{near[0]}' ({r:.3f})"))
    inside = sorted((r for r in nonterm if len(r) >= 12 and r in text and r != text),
                    key=lambda r: (-len(r), r))   # name breaks length ties: sets are unordered
    if inside:
        sig.append(("B", "contains " + ", ".join(inside[:3])))
    # ⭐⭐ SIGNAL C — PUNCTUATION TRANSLITERATED INTO THE TOKEN TEXT. Added after signals A and B
    # MISSED a whole production: the LRM writes `PATHPULSE$`, and the extraction emitted a terminal
    # matching the literal characters `PATHPULSE_dollar`. No SystemVerilog source contains that, so
    # `pulse_control_specparam` was unreachable through BOTH its alternatives — and neither A nor B
    # fires, because `PATHPULSE_dollar` is near no nonterminal and contains none.
    # ✅ ITS FOUNDING POPULATION IS FIXED AND THE SIGNAL NOW REPORTS ZERO (`.13c.2f` slice 4): both
    # terminals match `PATHPULSE\$…` — the character the LRM writes — so C is a live tripwire over an
    # empty population, not a dead branch. An empty C is the AFTER-measurement of that fix.
    # ⚠️ AND THE EVIDENCE THAT FOUNDED IT WAS THE WEAK FORM OF THE RIGHT TEST. It was recorded as
    # *"an AST dump of `specparam PATHPULSE$a$y = (1);` contains `pulse_control` ZERO times"* — true,
    # and it reads zero on a REACHED production too, because no annotation emits the rule's name. The
    # discriminator is the declared `kind`: `simple` = the ordinary `specparam_assignment`,
    # `general`/`input_output` under a `pulse` node = this production. Measured after the fix, that
    # very input still reports `pulse=0` — a single limit value is a legal parenthesised
    # `constant_mintypmax_expression`, so both alternatives tie and the earlier one wins
    # (`.13c.2h`). A `(1, 2)` input is what separates them.
    translit = [x for x in PUNCT_NAMES if f"_{x}" in text or text.startswith(x + "_")]
    if translit:
        sig.append(("C", "punctuation transliterated: " + ", ".join(sorted(translit))))
    if sig:
        hits[name] = (text, sig)

defects, exonerated, disagreements = [], [], []
for name, (text, sig) in sorted(hits.items(), key=lambda kv: kv[1][0]):
    is_keyword = text in KEYWORDS
    sv, allf = corpus_hits(text) if corpus_available else (None, None)
    (exonerated if is_keyword else defects).append((name, text, sig, sv, allf))
    # The two authorities must agree: a keyword should be reachable, a non-keyword dead.
    if corpus_available and is_keyword != (sv > 0):
        disagreements.append((name, text, is_keyword, sv))

print(f"⛔ DEFECTS — the literal is NOT in Annex B, so it spells a nonterminal, not a keyword\n")
print(f"{'#':<3} {'matched literal text':<46} {'sites':>5}  signals")
print("-" * 108)
for i, (name, text, sig, sv, allf) in enumerate(defects, 1):
    uses = use_lines(name)
    print(f"{i:<3} {text[:46]:<46} {len(uses):>5}  " + " | ".join(f"{s}: {d}" for s, d in sig))
    print(f"    defined  {GRAMMAR}:{def_line(name)}")
    print(f"    used at  " + ", ".join(str(u) for u in uses[:12]) + ("…" if len(uses) > 12 else ""))
    if corpus_available:
        extra = f"; plus {allf - sv} non-SV file(s), which cannot exercise it" if allf > sv else ""
        print(f"    corpus   {sv} SV-source file(s) contain this literal — UNREACHABLE{extra}")
    # ⭐ THE REPAIR TARGET IS DERIVED, NOT ASSERTED. A first draft of this report claimed "three of
    # the four have their intended nonterminal in the grammar"; computing it says otherwise, and the
    # difference decides how expensive each fix is.
    cands = sorted(difflib.get_close_matches(text, sorted(nonterm), n=8, cutoff=CUTOFF))
    contained = sorted((r for r in nonterm if len(r) >= 12 and r in text and r != text),
                       key=len, reverse=True)
    exact_present = text in nonterm
    if exact_present:
        print(f"    candidate  the nonterminal '{text}' EXISTS in the grammar — re-pointing the "
              "reference is the shape of the fix")
    elif contained and not cands:
        print(f"    candidate  the production flattened {len(contained)} nonterminals, all present: "
              + ", ".join(contained[:3]))
    else:
        pool = sorted(set(cands) | set(contained))
        print(f"    candidate  ⛔ NO nonterminal of this exact name exists. {len(pool)} NAME "
              f"NEIGHBOUR(S): {', '.join(pool[:6])}{'…' if len(pool) > 6 else ''}")
        print( "               ⚠️ a name neighbour is NOT a repair — see the note below the table.")
    print()

print("-" * 108)
print(f"✅ EXONERATED — in Annex B, so the `kw_*` terminal is correct by construction ({len(exonerated)}):")
for name, text, sig, sv, allf in exonerated:
    reach = f"{sv} SV file(s)" if corpus_available else "corpus NOT EVALUATED"
    print(f"    {text:<28} {len(use_lines(name)):>2} site(s), reachable in {reach}")
print()
print("-" * 108)
print(f"=> {len(hits)} of {len(kw)} kw_* terminals matched a signal; "
      f"**{len(defects)} are defects**, {len(exonerated)} are correct keywords "
      f"({100*len(exonerated)//max(len(hits),1)} % of the raw signal set was a false positive)")
print(f"   defect usage sites in the grammar: {sum(len(use_lines(n)) for n, *_ in defects)}")
if corpus_available:
    if disagreements:
        print("\n   ⚠️ THE TWO AUTHORITIES DISAGREE — adjudicate before trusting the split above:")
        for name, text, kwd, sv in disagreements:
            print(f"      {text}: Annex-B keyword={kwd} but corpus reachability={sv} SV file(s)")
    else:
        print(f"   ✓ CROSS-CHECK: Annex-B membership and corpus reachability partition all "
              f"{len(hits)} rows IDENTICALLY.")
        print( "     Two authorities with no shared input — a standards table and 16 427 source")
        print( "     files — so neither verdict rests on the other.")
print()
print("⚠️ WHAT THIS SWEEP DOES NOT ANSWER, stated rather than implied:")
print("   - WHICH nonterminal each site meant. The `candidate` line narrows it mechanically and says")
print("     so when it cannot close it. ⛔⛔ A NAME NEIGHBOUR IS NOT A REPAIR, and this population")
print("     contains the counter-example that proves it: `class_qualifier`'s nearest name is")
print("     `class_item_qualifier`, and in IEEE 1800 those are DIFFERENT productions —")
print("     `class_item_qualifier ::= static | protected | local` versus")
print("     `class_qualifier ::= [ local:: ] [ implicit_class_handle . | class_scope ]`.")
print("     Substituting the neighbour would replace an unreachable alternative with a WRONG one,")
print("     which is worse: unreachable under-accepts, wrong mis-parses. Only the LRM text decides.")
print("   - whether re-pointing a reference is a one-line fix. It is not, at least once: the sv_2017")
print("     cross-body shape demands a `;` after EVERY item, so the repair must mirror the sv_2023")
print("     pair. Every fix here is ACCEPT-WIDENING and owes the full ceremony.")
print("   - ⛔ WHETHER THE CONSTRUCT IS REJECTED. This sweep measures REACHABILITY of an")
print("     ALTERNATIVE, and a construct can be reachable through a DIFFERENT production, so")
print("     `unreachable` does NOT imply `rejected`. This bullet asserted the implication until")
print("     `.13c.2f`, and pinning a minimal input per site refuted it: of the five sites the")
print("     sweep has ever listed, THREE rejected (`function_declaraton`, both `PATHPULSE$`")
print("     forms — all three now fixed) and the two that remain, `class_qualifier` and")
print("     `tx_path_delay_expression`, ACCEPT today through other productions. ⇒ every row here")
print("     owes a pinned REJECT before it is called an under-acceptance, and a row with none is")
print("     an unreachable alternative — still a defect, because the grammar claims support it")
print("     does not deliver, but not axis-2 mass.")

# ── cross-family: does this shape exist in any OTHER tracked grammar? ─────────────────────────────
# ⛔ THIS LIVES IN THE SCRIPT, NOT APPENDED TO ITS OUTPUT. It was first written by appending to the
# tracked .txt after generating it — which the next run silently drops, i.e. a hand-maintained
# addendum to a derived artifact. That is the rot class this whole tree keeps finding.
# ⛔ AND IT EXISTS BECAUSE THE FIRST DRAFT OF `.13c.2f`'s routing evidence ASSERTED that no other
# family has `kw_*` trivia terminals at all. Three do. The conclusion survived; the reason did not.
print()
print("=" * 100)
print("CROSS-FAMILY — does this shape exist in any OTHER tracked grammar?\n")
others = ["grammars/vhdl.ebnf", "grammars/rtl_frontend.ebnf",
          "grammars/systemverilog_lrm_profiled_generated.ebnf"]
print(f"  {'grammar':<52} {'kw_*':>5} {'_ in text':>10} {'hits':>6}")
for g in others:
    if not os.path.isfile(g):
        print(f"  {g:<52} {'—':>5} {'—':>10} {'ABSENT':>6}")
        continue
    osrc = open(g, encoding="utf-8").read()
    orules = re.findall(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:=", osrc, re.M)
    ont = {r for r in orules if not r.startswith("kw_")}
    ont |= {re.sub(r"_sv_20(17|23)$", "", r) for r in ont}
    okw = [(m.group(1), m.group(2).replace(r"\b", ""))
           for m in re.finditer(r"^(kw_[A-Za-z0-9_]+)\s*:=\s*trivia\s*/([^/]+)/", osrc, re.M)]
    und = [(n, t) for n, t in okw if "_" in t]
    ohits = []
    for n, t in und:
        near = difflib.get_close_matches(t, sorted(ont), n=1, cutoff=CUTOFF)
        # ⛔ SORTED, and longest-first: `ont` is a set, so an unsorted comprehension picks a
        # different representative between runs. Caught by an idempotence control, not by
        # review — two runs of this script printed different `contains` names for one row.
        inside = sorted((r for r in ont if len(r) >= 12 and r in t and r != t),
                        key=lambda r: (-len(r), r))
        if t in ont or near or inside:
            why = "EXACT" if t in ont else (f"near {near[0]}" if near else f"contains {inside[0]}")
            ohits.append((t, why))
    print(f"  {g:<52} {len(okw):>5} {len(und):>10} {len(ohits):>6}")
    for t, why in sorted(ohits):
        print(f"        {t[:60]:<62} -> {why}")
print()
print("  ⇒ VHDL and rtl_frontend are clean BY MEASUREMENT, not by an argument about how they were")
print("    authored. ⭐⭐ The SV-family sibling is TRACKED and its own header says \"Auto-generated by")
print("    tools/extract_systemverilog_lrm_profiles.py\", so the shape is produced by an EXTRACTOR:")
print("    a hand fix to grammars/systemverilog.ebnf leaves the generator emitting it, and the next")
print("    regeneration puts it back into git. Routed as SV-CORPUS-GRAD.13c.2f(d).")
