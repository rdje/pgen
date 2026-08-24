#!/usr/bin/env python3
"""CORPUS-KEY-AUDIT.1 — the answer-key CONTRADICTION census.

WHAT IT ANSWERS
  Do any two corpus rows, keyed from the SAME upstream evidence, carry OPPOSITE expected
  verdicts? A corpus large enough to contain a contradiction is an oracle you already own,
  and the question is mechanical: group rows by the upstream messages their key reads, and
  flag any message class whose rows disagree about `must_accept` vs `must_reject`.

WHY IT EXISTS
  `SV-CORPUS-GRAD.13e.3` found `br_gh552.v` and `real_invalid_ops.v` — the SAME operator,
  OPPOSITE expectations — BY HAND, and only because one of them happened to be adjudicated
  for another reason. The disagreement had been sitting in one tracked file for the life of
  the campaign, and nothing in the repository was asking the question.

⛔⛔ THE FOUNDING VERSION OVER-REPORTED BY CONSTRUCTION — `.1`(a) FIXED IT, AND THE
   FIX IS NOT THE ONE THE LEAF PLANNED.
  A row carries ONE expected verdict; its golden may carry SEVERAL messages, so attributing
  EVERY message to the row's verdict manufactures a disagreement whenever a file was pinned
  for a reason unrelated to most of what its golden says. `.1` planned to narrow that with
  the parser's own `furthest_position`. ⛔ MEASURED, THAT PLAN WOULD NOT HAVE WORKED AND IS
  ALSO CIRCULAR: `br_gh1087b` is pinned at line 3 while the only message in its golden sits
  at line 6, so positional narrowing finds nothing to attribute and falls back — and using
  the parser under test to interpret its own answer key is not evidence.

  ⭐ THE KEY ALREADY RECORDS ITS OWN DECIDING EVIDENCE, IN THE BASIS STRING. Three shapes,
  all mechanical, none needing a parse:
    * CLAUSE-CITED  — the basis names an IEEE clause / Annex A production / an LRM doc path.
      The key rests on the CLAUSE, not on the golden, so the row attributes NO message.
      `pr1704726a`'s basis says it outright: *"spec outranks the ivtest driver key"*.
    * QUOTED-DECIDER — the basis quotes the deciding message verbatim, as
      `PARSE-stage refusal ('…')`. The row attributes ONLY the golden messages containing
      that fragment. If the fragment is ABSENT from the golden the key contradicts its own
      cited evidence: that is reported as a KEY-INTEGRITY finding and the row falls back to
      the conservative whole-golden attribution, so a broken quote can never SILENCE a row.
    * WHOLE-GOLDEN  — everything else. For `must_accept` this is exact rather than a
      fallback: the accept key's claim IS that every message present is post-parse.

⛔ THE EXTRACTOR ITSELF WAS BLIND, AND THAT IS THE SHARPER HALF OF `.1`(a).
  The founding `MSG` regex required a literal `error:`/`sorry:` tag, but iverilog emits its
  bare parse refusal untagged — `./ivltests/br_gh79.v:6: syntax error`. So the ONE message
  class that most directly answers a parse-stage question was absent from the vocabulary the
  founding `messages=168` was measured over, together with `Net data type requires
  SystemVerilog or -gxtypes.` — the dialect-gate class `SV-CORPUS-GRAD.13e.3`(a) had just
  caught the key mis-reading. Corrected: **176** classes, +8, none dropped. `warning:` lines
  and `:      : It was declared here …` continuations stay excluded — a warning is not a
  refusal and a continuation annotates the message above it rather than standing alone.

Usage:
  python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py [--md OUT]
  python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py --self-test
"""
import argparse
import collections
import csv
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
IVTEST = ROOT / "stimuli/sv/subs/iverilog/ivtest"
GOLD = IVTEST / "gold"
MANIFESTS = {
    "sv_2017": ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
    "verilog_2005": ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
}
# One upstream message: `<file>.v[.ext]:<line>: [error:|sorry:] <text>`.
# ⛔ The tag is OPTIONAL — requiring it hid `syntax error`, the most parse-relevant class in
# the corpus (see the module docstring). Two exclusions, both deliberate:
#   `(?![: \t])` drops `:  : It was declared here …` continuation lines, which annotate the
#                 message above them and are not independent evidence;
#   `(?!warning:)` drops warnings, which are not refusals.
MSG = re.compile(
    r"^[^:]*\.v(?:\.\w+)?:(\d+):[ \t]*(?![: \t])(?!warning:)"
    r"(?:(?:error|sorry):[ \t]*)?(.+?)[ \t]*$", re.M)
VERDICTS = ("must_accept", "must_reject")

# A basis that names a normative clause did not read the golden to reach its verdict.
CLAUSE_CITE = re.compile(r"IEEE\s*1(?:364|800)-\d{4}|docs/(?:verilog|systemverilog)/"
                         r"|\bA\.\d+\.\d+\b")
# A basis that quotes the upstream message it read: `PARSE-stage refusal ('…')`.
QUOTED_DECIDER = re.compile(r"PARSE-stage refusal \('(.+?)'\)")

CLAUSE = "clause-cited"
QUOTED = "quoted-decider"
QUOTED_BAD = "quoted-decider-UNRESOLVED"
ACCEPT_CLAIM = "whole-golden:accept-claim"
UNDETERMINED = "whole-golden:UNDETERMINED"


def deciding_messages(expected: str, basis: str, msgs):
    """Which of a golden's messages actually keyed this row? -> (texts, provenance).

    ⭐ Read from the key's OWN recorded evidence, never from the parser — a census that asks
    the instrument under test to interpret its answer key proves nothing about either.
    """
    every = [t for _line, t in msgs]
    if CLAUSE_CITE.search(basis):
        return [], CLAUSE
    m = QUOTED_DECIDER.search(basis)
    if m:
        selected = [t for t in every if m.group(1) in t]
        if selected:
            return selected, QUOTED
        # The quote names evidence its own golden does not contain. Report it, and keep the
        # WIDEST attribution: a broken quote must not be able to silence a row.
        return every, QUOTED_BAD
    return every, (ACCEPT_CLAIM if expected == "must_accept" else UNDETERMINED)


def normalize(msg: str) -> str:
    """Collapse the parts an upstream message varies per file: quoted names and integers.

    ⛔ Deliberately NOT a prefix match. `SV-CORPUS-GRAD.3.24` measured a prefix rule sweeping
    preprocessor and command-file messages into a parse verdict; the same hazard applies here.
    """
    msg = re.sub(r"[`'\"][^`'\"]*[`'\"]", "X", msg)
    return re.sub(r"\b\d+\b", "N", msg).strip()


def gold_text(stem: str) -> str:
    text = ""
    for chan in ("iverilog-stderr", "iverilog-stdout"):
        f = GOLD / f"{stem}-{chan}.gold"
        if f.is_file():
            text += f.read_text(errors="replace")
    plain = GOLD / f"{stem}.gold"
    if plain.is_file():
        text += plain.read_text(errors="replace")
    return text


def gold_index() -> dict:
    """source stem -> golden stem, from BOTH descriptor sources the ivtest harness uses."""
    out = {}
    for jf in sorted((IVTEST / "vvp_tests").glob("*.json")):
        try:
            d = json.loads(jf.read_text(errors="replace"))
        except ValueError:
            continue
        if d.get("gold") and d.get("source"):
            out.setdefault(Path(d["source"]).stem, Path(d["gold"]).stem)
    lst = IVTEST / "regress-vlg.list"
    if lst.is_file():
        for line in lst.read_text(errors="replace").splitlines():
            parts = line.strip().split()
            if len(parts) < 2:
                continue
            g = next((p.split("=", 1)[1] for p in parts if p.startswith("gold=")), None)
            if g:
                out.setdefault(parts[0], Path(g).stem)
    return out


def census(rows):
    """One pass over already-loaded rows. Returns the naive AND the refined attribution.

    Both are computed here on purpose: the refined number means nothing without the number it
    replaced, and a false-positive rate quoted from a previous run is a carried number.
    """
    naive = collections.defaultdict(lambda: collections.defaultdict(set))
    refined = collections.defaultdict(lambda: collections.defaultdict(set))
    vocabulary = set()
    provenance = collections.Counter()
    integrity = []
    for r in rows:
        ident = (r["lane"], r["stem"])
        for _line, text in r["msgs"]:
            key = normalize(text)
            vocabulary.add(key)
            naive[key][r["expected"]].add(ident)
        texts, prov = deciding_messages(r["expected"], r["basis"], r["msgs"])
        provenance[(r["expected"], prov)] += 1
        if prov == QUOTED_BAD:
            integrity.append((*ident, QUOTED_DECIDER.search(r["basis"]).group(1)))
        for text in texts:
            refined[normalize(text)][r["expected"]].add(ident)

    def contradictory(table):
        return {m: v for m, v in table.items() if len([e for e in v if e in VERDICTS]) > 1}

    # ⛔ Per-side ROW COUNTS over the FULL refined table, not over the contradictory subset —
    # reading the subset makes every non-contradictory class report zero rows on both sides,
    # i.e. a power-bound table that silently says the census had no reach at all.
    sides = {e: {m: len(v[e]) for m, v in refined.items() if v.get(e)} for e in VERDICTS}
    return {"naive": contradictory(naive), "refined": contradictory(refined),
            "vocabulary": vocabulary, "provenance": provenance, "integrity": integrity,
            "rows": len(rows), "sides": sides}


def load_rows():
    """Every keyed iverilog row that HAS a golden, with that golden's messages attached."""
    gold_of = gold_index()
    rows = []
    for lane, path in MANIFESTS.items():
        if not path.is_file():
            continue
        with path.open(encoding="utf-8") as fh:
            for r in csv.DictReader(fh, delimiter="\t"):
                if r["suite"] != "iverilog" or r["expected"] not in VERDICTS:
                    continue
                stem = Path(r["relpath"]).stem
                gs = gold_of.get(stem)
                if not gs:
                    continue
                rows.append({"lane": lane, "stem": stem, "expected": r["expected"],
                             "basis": r["basis"],
                             "msgs": [(int(m.group(1)), m.group(2))
                                      for m in MSG.finditer(gold_text(gs))]})
    return rows


def _class_table(contra):
    L = ["| upstream message (normalized) | `must_accept` rows | `must_reject` rows |",
         "|---|---|---|"]
    for m, v in sorted(contra.items(), key=lambda kv: -sum(len(x) for x in kv[1].values())):
        cell = {}
        for e in VERDICTS:
            names = sorted(stem for _l, stem in v.get(e, ()))
            cell[e] = (f"**{len(names)}** — " + ", ".join(f"`{n}`" for n in names[:4])
                       + ("…" if len(names) > 4 else "")) if names else "—"
        L.append(f"| `{m}` | {cell['must_accept']} | {cell['must_reject']} |")
    return L + [""]


def render(res, rows):
    lanes = collections.Counter(r["lane"] for r in rows)
    naive, refined = len(res["naive"]), len(res["refined"])
    removed = naive - refined
    L = ["# Answer-key contradiction census — CORPUS-KEY-AUDIT.1", "",
         "> DERIVED. Re-run: `python3 "
         "docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`", ">",
         "> ⛔ **STILL A WORKLIST, NOT A DEFECT COUNT.** `.1`(a) attributes each row to the",
         "> evidence its own basis records — a CLAUSE cite keys nothing from the golden, a quoted",
         "> `PARSE-stage refusal ('…')` keys only the message it quotes — which removes the",
         "> founding over-report at its cause. What survives is a candidate: only adjudication",
         "> against the LRM decides, and the census never asks the parser under test.", "",
         f"- keyed rows scanned: **{res['rows']}** "
         f"({', '.join(f'{k} {v}' for k, v in sorted(lanes.items()))})",
         f"- distinct normalized upstream messages: **{len(res['vocabulary'])}**",
         f"- contradictory classes, NAIVE attribution (every message ← the row's verdict): "
         f"**{naive}**",
         f"- contradictory classes, DECIDING-EVIDENCE attribution: **{refined}**",
         f"- ⇒ false positives removed: **{removed}** of {naive}"
         + (f" (**{removed / naive:.0%}** of the founding report)" if naive else ""), ""]

    L += ["## Attribution provenance — where each row's key actually came from", "",
          "| expected | provenance | rows |", "|---|---|---|"]
    for (exp, prov), n in sorted(res["provenance"].items()):
        L.append(f"| `{exp}` | `{prov}` | {n} |")
    L += ["", "`clause-cited` rows contribute NO message class: their key rests on the clause, "
          "which the north star ranks above tool testimony (`pr1704726a`'s basis says so in "
          "its own words). `whole-golden:accept-claim` is exact rather than a fallback — a "
          "`must_accept` key asserts that EVERY message present is post-parse. "
          "`whole-golden:UNDETERMINED` is the conservative fallback for a `must_reject` row "
          "whose basis records no quotable decider; it keeps the row in the worklist.", ""]

    reject_side = sorted(res["sides"]["must_reject"])
    accept_side = res["sides"]["must_accept"]
    L += ["## ⛔ Honest power bound — how much could this census have found?", "",
          "A contradiction needs a class on BOTH sides, so the census can only ever reach the",
          f"intersection. After attribution the reject side is **{len(reject_side)} classes** "
          f"wide against **{len(accept_side)}** on the accept side — quoting `0` without that "
          "ratio would read as a far stronger clean bill than it is.", "",
          "| reject-side deciding class | `must_reject` rows | `must_accept` rows carrying it |",
          "|---|---|---|"]
    for m in reject_side:
        hits = accept_side.get(m, 0)
        L.append(f"| `{m}` | {res['sides']['must_reject'][m]} "
                 f"| {'⛔ ' if hits else ''}{hits} |")
    L += ["", "⭐ The sharpest row is `syntax error`: it is the upstream's own bare parse refusal, "
          "so a `must_accept` row whose golden carried one would be a near-certain key defect — "
          "and the founding extractor could not see the class at all, which made the question "
          "unaskable rather than answered. It is now asked, over the whole accept population.", ""]

    L += ["## Key integrity — does a basis's quoted evidence exist in the golden it names?", ""]
    if res["integrity"]:
        L += [f"⛔ **{len(res['integrity'])} row(s) quote a message their own golden does not "
              "contain.** Each keeps the WIDEST attribution here, so a broken quote cannot "
              "silence a row.", "",
              "| lane | row | quoted fragment |", "|---|---|---|"]
        L += [f"| `{lane}` | `{stem}` | `{frag}` |"
              for lane, stem, frag in sorted(res["integrity"])]
        L.append("")
    else:
        L += ["Every quoted decider resolves against its own golden.", ""]

    L += ["## Contradictory classes under the deciding-evidence attribution", ""]
    L += _class_table(res["refined"]) if res["refined"] else \
         ["No message class carries contradictory expectations.", ""]
    if res["naive"]:
        L += ["## For comparison — the classes the NAIVE attribution reported", "",
              "Kept so the correction is auditable rather than asserted.", ""]
        L += _class_table(res["naive"])
    return L


def self_test() -> int:
    """Prove the control can go RED, and that the EXCLUSION is what turns it green.

    ⛔ Fully synthetic, so it runs with the corpora absent and cannot be quietly satisfied by
    whatever the manifests happen to contain today. A control that only ever passes is not a
    control — `.1`'s own founding number is here because nothing was checking the key.
    """
    shared = "Unresolved wire 'w' cannot have multiple drivers."
    tool = "ivtest: vlg CE but golden x.gold shows only post-parse errors"
    def row(stem, expected, basis, msgs=((5, shared),)):
        return {"lane": "synthetic", "stem": stem, "expected": expected,
                "basis": basis, "msgs": list(msgs)}

    arms = [
        ("detector FIRES on a real disagreement",
         [row("accepts", "must_accept", tool),
          row("rejects", "must_reject", tool)],
         lambda r: len(r["refined"]) == 1 and not r["integrity"]),
        ("a CLAUSE cite on the reject side silences it — the exclusion does the work",
         [row("accepts", "must_accept", tool),
          row("rejects", "must_reject",
              "pinned: IEEE 1364-2005 A.2.1.3 net_declaration admits no net data type")],
         lambda r: len(r["refined"]) == 0 and len(r["naive"]) == 1),
        ("a quoted decider narrows to the message it quotes",
         [row("accepts", "must_accept", tool,
              ((5, shared), (6, "syntax error"))),
          row("rejects", "must_reject",
              tool + " reporting a PARSE-stage refusal ('syntax error')",
              ((5, shared), (6, "syntax error")))],
         lambda r: {m for m in r["refined"]} == {"syntax error"} and len(r["naive"]) == 2),
        ("a quote absent from the golden is REPORTED and does not silence the row",
         [row("accepts", "must_accept", tool),
          row("rejects", "must_reject",
              tool + " reporting a PARSE-stage refusal ('no such message')")],
         lambda r: len(r["integrity"]) == 1 and len(r["refined"]) == 1),
        ("an untagged `syntax error` is IN the vocabulary (the founding regex missed it)",
         [row("accepts", "must_accept", tool, ((6, "syntax error"),))],
         lambda r: "syntax error" in r["vocabulary"]),
    ]
    failed = 0
    for name, rows, check in arms:
        res = census(rows)
        ok = check(res)
        failed += not ok
        print(f"  {'✅' if ok else '⛔'} {name}"
              + ("" if ok else f"   [naive={len(res['naive'])} refined={len(res['refined'])} "
                               f"integrity={len(res['integrity'])} vocab={sorted(res['vocabulary'])}]"))
    print(f"KEY-CONTRADICTION-CENSUS SELF-TEST: arms={len(arms)} failed={failed}")
    return 1 if failed else 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--md", type=Path, default=Path(__file__).with_name("census.md"))
    ap.add_argument("--self-test", action="store_true",
                    help="run the synthetic controls only; touches no tracked artifact")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    if not IVTEST.is_dir():
        print(f"⛔ REFUSING: {IVTEST} is absent (the vendored corpora are git submodules). "
              "This census is NOT EVALUATED — never report that as a pass.", file=sys.stderr)
        return 2

    rows = load_rows()
    res = census(rows)
    args.md.write_text("\n".join(render(res, rows)) + "\n", encoding="utf-8")
    print(f"KEY-CONTRADICTION-CENSUS: rows={res['rows']} messages={len(res['vocabulary'])} "
          f"contradictory_classes={len(res['refined'])} "
          f"(naive={len(res['naive'])}) key_integrity_findings={len(res['integrity'])}")
    print(f"wrote {args.md.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
