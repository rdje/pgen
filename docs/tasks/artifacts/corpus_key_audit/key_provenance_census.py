#!/usr/bin/env python3
"""CORPUS-KEY-AUDIT.2 — the answer-key PROVENANCE census.

WHAT IT ANSWERS
  For every keyed row in the SV adjudication manifests: what KIND of evidence produced its
  expected verdict? A clause of the standard, the observed OUTPUT of an upstream tool, or a
  suite's own METADATA. ⛔ Only the first cannot be wrong the way `SV-CORPUS-GRAD.13e.3`
  found twice in three days, so **the size of the other two IS the trust bound on the
  published defect bar** — and that number was unpublished.

WHY IT EXISTS
  `SV-CORPUS-DENOMINATOR` publishes the bar beside its denominator, because a bar without one
  is not a claim about the corpus. This census is the layer under that: a denominator whose
  expectations rest on tool testimony is only as good as the tool, and the corpus already
  contains two measured cases where it was not good enough — a driver key that never read the
  flag deciding the dialect (`.13e.3`(a)) and a golden read by one phrase out of many
  (`.13e.3`(b) / ledger `SV-0068`).

⛔⛔ THE CLASSIFIER WAS WRONG TWICE BEFORE IT WAS RIGHT, IN BOTH DIRECTIONS, AND BOTH
    CORRECTIONS ARE PINNED AS SELF-TEST ARMS.
  1. OVER-COUNTING. A first cut called any basis naming `IEEE 1800-2017` clause-cited. That
     swept in all **623** Surelog rows, whose basis reads *"...parses under Surelog's IEEE
     1800-2017 grammar"* — an edition naming the upstream TOOL's grammar, i.e. the purest tool
     testimony in the corpus, classified as its opposite.
  2. UNDER-COUNTING. Demanding an `A.n.n` production then MISSED rows citing a numbered clause
     directly — *"IEEE 1800-2017 22.8 / IEEE 1364-2005 19.2 permit it only OUTSIDE..."*.
  ⇒ the rule is not "does it mention the standard" but "does it point at a PLACE in it".
  Resolved by ENUMERATING what actually follows every edition mention rather than guessing a
  third time: 623 `grammar`, 72 a dotted clause number, 63 `clauses`/`Annex`/`production`,
  and nothing else — an exhaustive partition of all 758, which is why the split is trustworthy.
  (The same lesson, in another corpus:
  docs/knowledge/a-heading-census-is-only-as-good-as-the-heading-grammar.md.)

PRECEDENCE, STATED BECAUSE A CLASSIFIER WITHOUT ONE IS AMBIGUOUS ON EVERY OVERLAPPING ROW
  clause-cited > tool-testimony > suite-convention. The first is the north star's own ranking
  (the spec outranks tool testimony, and `pr1704726a`'s basis says so in its own words). The
  second over the third because reading a tool's actual OUTPUT is a more specific claim than
  reading a metadata field, so when a basis does both, the output is what it rests on.

Usage:
  python3 docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py [--md OUT]
  python3 docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py --self-test
"""
import argparse
import collections
import csv
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
MANIFESTS = {
    "sv_2017": ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
    "verilog_2005": ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
}
VERDICTS = ("must_accept", "must_reject")

# A clause cite POINTS AT A PLACE in the standard: an Annex A production, an LRM doc path, a
# § / "clause N", or an edition immediately followed by a numbered clause / Annex / production.
LOCATOR = re.compile(r"\bA\.\d+\.\d+|docs/(?:verilog|systemverilog)/|§\s*\d|\bclause\s+\d"
                     # a PARENTHESIZED dotted clause number, the shape the `.8b.2`/`.8b.3` pinned
                     # rulings use with no edition prefix: "... forward reference (6.21) - name
                     # resolution". ⛔ Parenthesized deliberately: a bare dotted number also spells
                     # a task-leaf id (`.8b.3`), and matching those would classify every pinned row
                     # as clause-cited on its own bookkeeping.
                     r"|\(\d+(?:\.\d+)+[^)]*\)"
                     # the `LRM 5.7.1` spelling the sv-tests pins use, with no edition word
                     r"|\bLRM\s+\d+(?:\.\d+)*")
EDITION_CITE = re.compile(
    r"IEEE\s*1(?:364|800)-\d{4}\s+(?:\d+(?:\.\d+)+|clauses?\b|Annex\b|production\b)")
# ⛔ NOT a cite: "<Tool>'s IEEE 1800-2017 grammar" names the upstream tool's grammar. 623 rows.

# Observed upstream OUTPUT — the tool was RUN and what it said was read.
TOOL_OUTPUT = re.compile(r"golden|\.gold\b|\blog\b|error-suite|\bCE\b|exit status", re.I)
# Suite METADATA only — the suite DECLARES an intent; nothing was run and nothing was read.
# ⛔ Every alternative here was ENUMERATED from the population, never guessed: the first cut used
# the ivtest lane's vocabulary alone and left 4 961 rows (49 %) unclassified, which would have made
# the published trust bound a lower bound wearing the clothes of a measurement.
SUITE_META = re.compile(
    r"regress-\w+\.list|type (?:normal|CO|NI)\b|vvp_tests|descriptor|:tags:|\bdirectory\b"
    r"|driver \S+\.py|positive test|negative test|clause-keyed|suite contract|fixture"
    r"|:should_fail_because:|TYPE:\s*(?:POSITIVE|NEGATIVE)"
    # an annotation the corpus file carries about ITSELF is still the suite declaring intent
    r"|in-file comment|header says", re.I)

CLAUSE, TOOL, SUITE, UNCLASSIFIED = (
    "clause-cited", "tool-testimony", "suite-convention", "UNCLASSIFIED")


def classify(basis: str) -> str:
    """The provenance of ONE expectation. Precedence is documented in the module docstring."""
    if LOCATOR.search(basis) or EDITION_CITE.search(basis):
        return CLAUSE
    if TOOL_OUTPUT.search(basis):
        return TOOL
    if SUITE_META.search(basis):
        return SUITE
    return UNCLASSIFIED


def load_rows():
    rows = []
    for lane, path in MANIFESTS.items():
        if not path.is_file():
            continue
        with path.open(encoding="utf-8") as fh:
            for r in csv.DictReader(fh, delimiter="\t"):
                if r["expected"] in VERDICTS:
                    rows.append({"lane": lane, "suite": r["suite"], "expected": r["expected"],
                                 "basis": r["basis"], "relpath": r["relpath"],
                                 "adjudication": r["adjudication"]})
    return rows


def census(rows):
    by = collections.Counter()
    per_suite = collections.defaultdict(collections.Counter)
    samples = collections.defaultdict(list)
    residual = []
    for r in rows:
        c = classify(r["basis"])
        by[(r["lane"], c, r["expected"])] += 1
        per_suite[r["suite"]][c] += 1
        if len(samples[c]) < 3:
            samples[c].append((r["suite"], r["expected"], r["basis"][:150]))
        if c == UNCLASSIFIED:
            residual.append((r["suite"], r["expected"], r["basis"]))
    # ⛔ THE BAR'S OWN ROWS ARE A DIFFERENT POPULATION FROM THE WHOLE KEY, AND CONFLATING THEM
    # OVERSTATES THE CLAIM. The published defect bar is the `divergence:unexplained_*` subset, and
    # a divergence tends to get INVESTIGATED, which is exactly the event that produces a clause
    # cite. So the bar is measurably better-evidenced than the key as a whole, and the honest
    # sentence attaches the right number to the right population.
    # ⛔ sv_2017 ONLY. `SV-CORPUS-DENOMINATOR` publishes `bar` as the sv_2017 lane's
    # unexplained divergences; taking BOTH lanes gives 338, not the published 274, and a
    # "bar" that is not the bar anyone reads is worse than no number. The first cut of this
    # block did exactly that, and the two numbers disagreeing on sight is how it was caught.
    bar = [r for r in rows if r["lane"] == "sv_2017"
           and r["adjudication"].startswith("divergence:unexplained")]
    bar_by = collections.Counter(classify(r["basis"]) for r in bar)
    return {"by": by, "per_suite": per_suite, "samples": samples, "rows": len(rows),
            "residual": residual, "bar_rows": len(bar), "bar_by": bar_by}


def render(res, rows):
    lane_tot = collections.Counter(r["lane"] for r in rows)
    cls_tot = collections.Counter()
    for (lane, c, _e), n in res["by"].items():
        cls_tot[(lane, c)] += n
    L = ["# Answer-key PROVENANCE census — CORPUS-KEY-AUDIT.2", "",
         "> DERIVED. Re-run: `python3 "
         "docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py`", ">",
         "> ⛔ **THIS IS A TRUST BOUND, NOT A DEFECT COUNT.** A `tool-testimony` or",
         "> `suite-convention` expectation is not wrong; it is *capable of being wrong in the way",
         "> `SV-CORPUS-GRAD.13e.3` measured twice*, which a `clause-cited` one is not. The number",
         "> below says how much of the published bar rests on evidence outside the standard.", ""]

    for lane in sorted(lane_tot):
        tot = lane_tot[lane]
        clause = cls_tot[(lane, CLAUSE)]
        L += [f"## `{lane}` — {tot} keyed rows", "",
              "| provenance | rows | share | `must_accept` | `must_reject` |", "|---|---|---|---|---|"]
        for c in (CLAUSE, TOOL, SUITE, UNCLASSIFIED):
            n = cls_tot[(lane, c)]
            if not n and c == UNCLASSIFIED:
                L.append(f"| `{c}` | 0 | — | 0 | 0 |")
                continue
            L.append(f"| `{c}` | **{n}** | {n / tot:.1%} | "
                     f"{res['by'][(lane, c, 'must_accept')]} | "
                     f"{res['by'][(lane, c, 'must_reject')]} |")
        L += ["", f"⇒ **trust bound: {tot - clause} of {tot} ({(tot - clause) / tot:.1%})** of this "
              f"lane's expectations rest on evidence OUTSIDE the standard.", ""]

    n_bar = res["bar_rows"]
    cls_clause = sum(n for (_l, c, _e), n in res["by"].items() if c == CLAUSE)
    if n_bar:
        bar_clause = res["bar_by"][CLAUSE]
        L += [f"## ⛔ The BAR's own rows ({n_bar}, `sv_2017`) — a different population, a different number",
              "",
              "The published defect bar is the `sv_2017` `divergence:unexplained_*` subset, **not** the whole",
              "key. A divergence tends to get INVESTIGATED, and investigation is exactly the event",
              "that produces a clause cite — so the bar is measurably **better**-evidenced than the",
              "key as a whole. Attaching the key-wide share to the bar overstates the claim.", "",
              "| provenance | bar rows | share |", "|---|---|---|"]
        for c in (CLAUSE, TOOL, SUITE, UNCLASSIFIED):
            L.append(f"| `{c}` | {res['bar_by'][c]} | {res['bar_by'][c] / n_bar:.1%} |")
        L += ["", f"⇒ **{n_bar - bar_clause} of {n_bar} ({(n_bar - bar_clause) / n_bar:.1%})** of the "
              "rows *behind the published bar* rest on evidence outside the standard, against "
              f"**{(res['rows'] - cls_clause) / res['rows']:.1%}** key-wide. ⭐ Use **this** number "
              "when speaking about the BAR and the key-wide one when speaking about the ANSWER KEY: "
              "they are different populations, and the bar is the better-evidenced of the two.", ""]

    L += ["## By suite — where the non-clause evidence actually lives", "",
          "| suite | rows | `clause-cited` | `tool-testimony` | `suite-convention` | `UNCLASSIFIED` |",
          "|---|---|---|---|---|---|"]
    for suite in sorted(res["per_suite"], key=lambda s: -sum(res["per_suite"][s].values())):
        c = res["per_suite"][suite]
        n = sum(c.values())
        L.append(f"| `{suite}` | {n} | {c[CLAUSE]} | {c[TOOL]} | {c[SUITE]} | "
                 f"{'⛔ ' if c[UNCLASSIFIED] else ''}{c[UNCLASSIFIED]} |")
    L.append("")

    L += ["## ⛔ What the classifier got wrong before it got this right", "",
          "Both corrections are pinned as `--self-test` arms, because a classifier's own history is",
          "the only evidence its current rule is not the next mistake.", "",
          "| # | the rule | what it did | rows |", "|---|---|---|---|",
          "| 1 | *any mention of `IEEE 1800-2017` is a clause cite* | swept in every Surelog row, "
          "whose basis reads *\"parses under **Surelog's** IEEE 1800-2017 grammar\"* — an edition "
          "naming the upstream TOOL's grammar, i.e. the purest tool testimony in the corpus, "
          "classified as its opposite | **623 over-counted** |",
          "| 2 | *a clause cite needs an `A.n.n` production* | missed rows citing a numbered clause "
          "directly — *\"IEEE 1800-2017 22.8 / IEEE 1364-2005 19.2 permit it only OUTSIDE…\"* | "
          "**72 under-counted** |", "",
          "⇒ the rule is not *does it mention the standard* but ***does it point at a PLACE in it***.",
          "Settled by ENUMERATING what follows every edition mention rather than guessing a third",
          "time — 623 `grammar`, 72 a dotted clause number, 63 `clauses`/`Annex`/`production`, and",
          "nothing else. That the partition is EXHAUSTIVE is why the split is trustworthy.", ""]

    L += ["## ⛔ The residual, VERBATIM — a census that hides its leftovers is a census you cannot check",
          ""]
    if res["residual"]:
        L += [f"**{len(res['residual'])} of {res['rows']} rows** ({len(res['residual']) / res['rows']:.2%}) "
              "match no rule. Every one is printed here rather than folded into a class, because a",
              "residual absorbed into the nearest bucket is exactly how a classifier stops being",
              "checkable. The first cut left **4 961** unclassified (49 %) and the vocabulary was",
              "extended by ENUMERATING those shapes, not by widening a rule until the number fell.", ""]
        for suite, exp, b in res["residual"]:
            L.append(f"- `{suite}` / `{exp}` — {b}")
        L += ["", "⚠️ **TWO of these ARE clause cites, so `clause-cited` is a LOWER bound, short by "
              "exactly 2.** They spell the clause as a bare dotted number (*\"…pinned .8c.1: 12.8.2 "
              "early defparam…\"*, *\"…- 19.11 selects the 1364-2001 keyword set…\"*), and a general "
              "bare-dotted-number rule is **deliberately not added**: `.8b.3` and `.13e.2` are "
              "task-leaf ids in the very same strings, so such a rule would classify every pinned row "
              "as clause-cited on its own bookkeeping. Two rows misfiled in the conservative direction "
              "is the cheaper error. The other two cite nothing external at all — they are the "
              "adjudicator's own reading of the file — which is a third trust profile, too small a "
              "population (2) to justify naming a class for.", ""]
    else:
        L += ["No row is unclassified.", ""]

    L += ["## Samples, one per class — so the classification is auditable, not asserted", ""]
    for c in (CLAUSE, TOOL, SUITE, UNCLASSIFIED):
        if not res["samples"][c]:
            continue
        L.append(f"**`{c}`**")
        L.append("")
        for suite, exp, b in res["samples"][c]:
            L.append(f"- `{suite}` / `{exp}` — {b}")
        L.append("")
    return L


def self_test() -> int:
    """Every arm is a basis string this corpus actually contains, or the exact trap it produced."""
    cases = [
        # the two measured mistakes, pinned in both directions
        ("Surelog: golden log Assert.log completes with no [SNT:]/[FTL:] - upstream testimony that "
         "the unit's single source parses under Surelog's IEEE 1800-2017 grammar", TOOL,
         "an edition naming the TOOL's grammar is NOT a clause cite (623-row over-count)"),
        ("pinned .3.14a: `default_nettype wire inside module t (line 11) - IEEE 1800-2017 22.8 / "
         "IEEE 1364-2005 19.2 permit it only OUTSIDE a design element", CLAUSE,
         "a numbered clause right after the edition IS a cite (72-row under-count)"),
        # the ordinary shapes
        ("pinned .13e.2: line 3 gives a NET a data type. IEEE 1364-2005 A.2.1.3 net_declaration "
         "(docs/verilog/2005/txt/section-Annex_A...:153) admits only", CLAUSE,
         "an Annex A production plus an LRM doc path"),
        ("ivtest: vlg CE with golden br1027a.gold reporting a PARSE-stage refusal", TOOL,
         "a golden log read for its message"),
        ("ivtest: regress-vlg.list type normal - compiles under the iverilog plain-Verilog dialect",
         SUITE, "a driver LIST metadata field, no tool output inspected"),
        ("ivtest: vvp_tests descriptor(s) br_gh1087a3.json - runs under an explicit SV generation",
         SUITE, "a descriptor, still metadata"),
        ("sv2v error-suite key: declaration grammar violation", TOOL, "an error-suite key"),
        ("pinned: LRM 5.7.1 lexical rule - '4af' needs 'h", CLAUSE,
         "the bare `LRM n.n.n` spelling the sv-tests pins use"),
        ("verilator: driver t_lparam_assign_1.py expects success", SUITE,
         "a suite DRIVER SCRIPT declaring an outcome is metadata, not observed output"),
        ("ispras: '// ! TYPE: POSITIVE' clause-keyed valid example", SUITE,
         "a suite TAG — `clause-keyed` names a tag, and must not read as a clause cite"),
        ("pinned: in-file comment 'lexer should reject invalid identifier'", SUITE,
         "an annotation the corpus file carries about itself is still the suite declaring intent"),
        ("pinned .8b.2: block-local initializer without static/automatic (6.21) - lifetime prose",
         CLAUSE, "a PARENTHESIZED clause number, with no edition word anywhere"),
        ("pinned .8b.3: some ruling with no cite at all", UNCLASSIFIED,
         "a task-leaf id is NOT a clause number — the rule must not fire on `.8b.3`"),
        ("some future basis nobody has a rule for", UNCLASSIFIED,
         "an unknown shape is REPORTED, never folded into a class"),
        # precedence, both edges
        ("ivtest: regress-vlg.list type normal but IEEE 1364-2005 A.4.2 admits no such form",
         CLAUSE, "clause OUTRANKS suite metadata on a row carrying both"),
        ("golden log shows nothing, and regress-vlg.list type normal", TOOL,
         "observed output OUTRANKS metadata on a row carrying both"),
    ]
    failed = 0
    print("key-provenance --self-test:")
    for basis, want, why in cases:
        got = classify(basis)
        ok = got == want
        failed += not ok
        print(f"  {'✅' if ok else '❌'} {why}"
              + ("" if ok else f"   [got {got}, wanted {want}]"))
    print(f"KEY-PROVENANCE-CENSUS SELF-TEST: arms={len(cases)} failed={failed}")
    return 1 if failed else 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--md", type=Path, default=Path(__file__).with_name("provenance.md"))
    ap.add_argument("--self-test", action="store_true",
                    help="run the classifier arms only; reads no manifest, writes nothing")
    args = ap.parse_args()
    if args.self_test:
        return self_test()

    missing = [str(p.relative_to(ROOT)) for p in MANIFESTS.values() if not p.is_file()]
    if missing:
        print(f"⛔ REFUSING: manifest(s) absent: {', '.join(missing)}. This census is NOT "
              "EVALUATED — never report that as a pass.", file=sys.stderr)
        return 2

    rows = load_rows()
    res = census(rows)
    args.md.write_text("\n".join(render(res, rows)) + "\n", encoding="utf-8")
    cls = collections.Counter()
    for (_lane, c, _e), n in res["by"].items():
        cls[c] += n
    print(f"KEY-PROVENANCE-CENSUS: rows={res['rows']} clause_cited={cls[CLAUSE]} "
          f"tool_testimony={cls[TOOL]} suite_convention={cls[SUITE]} "
          f"unclassified={cls[UNCLASSIFIED]} "
          f"trust_bound={res['rows'] - cls[CLAUSE]}/{res['rows']} "
          f"bar_trust_bound={res['bar_rows'] - res['bar_by'][CLAUSE]}/{res['bar_rows']}")
    try:
        shown = args.md.resolve().relative_to(ROOT)
    except ValueError:
        shown = args.md
    print(f"wrote {shown}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
