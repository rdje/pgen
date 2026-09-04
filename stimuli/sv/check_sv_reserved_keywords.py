#!/usr/bin/env python3
"""check_sv_reserved_keywords.py — the SV grammar's reserved-word sets ARE the LRMs' Annex B tables.

SV-CORPUS-GRAD.3.37 (PGEN-SV-CORPUS-GRAD-0366). `grammars/systemverilog.ebnf` refuses a keyword as an
identifier through two regex alternations, `reserved_non_keyword_identifier_sv` (profiles sv_2017 and
sv_2023) and `reserved_non_keyword_identifier_v2005` (verilog_2005). Until 1.0.211 the SV list was typed
by hand and held 173 of IEEE 1800-2017 Table B.1's 248 words — `reg global;`, `i.super.i = 1;`,
`function new; input x;` and `var final;` all parsed with the keyword bound as an identifier. This
instrument derives the tables from the in-repo LRM text and compares SETS, both directions:

  gate mode (default)   derive + compare; exit 1 naming every word missing from or foreign to a regex
  --write               regenerate the two regex lines from the tables (alphabetical), keep everything else
  --self-test           a planted missing word and a planted foreign word are each REFUSED

Sources (never inferred, always these files):
  1800-2017 Table B.1   docs/systemverilog/2017/md/section-41-data-read-api.md   ("Table B.1—Reserved keywords" .. "Annex C")
  1800-2023 Table B.1   docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md
  1364-2005 Annex B     docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md
The 2017 and 2023 tables must agree (they do: 248 = 248), because ONE regex serves both SV profiles.

Extraction: a table row is a line holding exactly one identifier-shaped token. The PDF's page furniture
inside the table (running headers, "Authorized licensed use", "Copyright", a bare page number, the
"(continued)" header) is skipped by pattern; prose before the first row is skipped; the FIRST other non-token
line after the rows ends the list — 1364-2005's
table ends at its own footnote line `1unsigned`, after which the prose "is / reserved / for" is wrapped one
word per line and would otherwise read as three keywords. The tables are NOT strictly alphabetical
(Table B.1 lists `endspecify` before `endsequence`), so order is not used. A trailing footnote digit
(1364-2005 prints `unsigned1`) is stripped only when the digit form is not itself a 1800-2017 keyword but
the stem is (`tri0`, `unique0`, `supply1` ... are keywords and stay).
"""
import re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
SRC = {
    "1800-2017": (ROOT / "docs/systemverilog/2017/md/section-41-data-read-api.md", "Table B.1—Reserved keywords", "Annex C"),
    "1800-2023": (ROOT / "docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md", "Table B.1—Reserved keywords", None),
    "1364-2005": (ROOT / "docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md", "List of keywords", None),
}
TOKEN = re.compile(r"[a-z_][a-z0-9_$]*")
RULES = {"sv": "reserved_non_keyword_identifier_sv", "v2005": "reserved_non_keyword_identifier_v2005"}


FURNITURE = re.compile(r"^(Authorized licensed use|IEEE Std |IEEE Standard for|Copyright ©|\d+$)")


def table(edition, reference=None):
    path, start, end = SRC[edition]
    lines = path.read_text(encoding="utf-8").split("\n")
    s = next(i for i, l in enumerate(lines) if l.startswith(start))
    e = next((i for i, l in enumerate(lines) if i > s and end and l.startswith(end)), len(lines))
    words = []
    for raw in lines[s + 1:e]:
        t = raw.strip()
        if not t or raw.startswith(start) or FURNITURE.match(t):
            continue                                   # blank, "(continued)" header, page furniture
        if not TOKEN.fullmatch(t):
            if words:
                break                                  # the first real non-row line AFTER the rows ends the table
            continue                                   # the sentence(s) between the heading and the first row
        if reference is not None and t not in reference and t[-1].isdigit() and t[:-1] in reference:
            t = t[:-1]                                 # footnote digit
        words.append(t)
    return words


def regex_set(text, rule):
    line = next(l for l in text.split("\n") if l.startswith(rule + " :="))
    m = re.search(r"/\(\?:(.*)\)\\b/", line)
    return set(m.group(1).split("|")), line


def compare(text):
    kw17, kw23 = table("1800-2017"), table("1800-2023")
    kw05 = table("1364-2005", reference=set(kw17))
    problems = []
    if kw17 != kw23:
        problems.append(f"1800-2017 and 1800-2023 Table B.1 differ: only-2017={sorted(set(kw17)-set(kw23))} only-2023={sorted(set(kw23)-set(kw17))}")
    for key, words in (("sv", kw17), ("v2005", kw05)):
        have, _ = regex_set(text, RULES[key]); want = set(words)
        miss, extra = sorted(want - have), sorted(have - want)
        if miss:
            problems.append(f"{RULES[key]}: {len(miss)} Annex B word(s) NOT reserved: {' '.join(miss)}")
        if extra:
            problems.append(f"{RULES[key]}: {len(extra)} reserved word(s) NOT in Annex B: {' '.join(extra)}")
    return problems, {"1800-2017": len(kw17), "1800-2023": len(kw23), "1364-2005": len(kw05)}


def write(text):
    kw17 = table("1800-2017"); kw05 = table("1364-2005", reference=set(kw17))
    for key, words in (("sv", kw17), ("v2005", kw05)):
        _, line = regex_set(text, RULES[key])
        text = text.replace(line, f"{RULES[key]} := trivia /(?:" + "|".join(words) + r")\b/", 1)
    return text


def self_test(text):
    fails = []
    ok, counts = compare(text)
    if ok:
        fails.append("control: the tracked grammar must pass before planting (" + "; ".join(ok) + ")")
    planted = text.replace("|global|", "|", 1)
    if not any("NOT reserved: global" in p for p in compare(planted)[0]):
        fails.append("planted MISSING word `global` was not refused")
    planted = text.replace("|global|", "|global|frobnicate|", 1)
    if not any("NOT in Annex B: frobnicate" in p for p in compare(planted)[0]):
        fails.append("planted FOREIGN word `frobnicate` was not refused")
    if counts != {"1800-2017": 248, "1800-2023": 248, "1364-2005": 124}:
        fails.append(f"table sizes moved: {counts} (expected 248 / 248 / 124 — re-read the tables before trusting either side)")
    print(f"sv-reserved-keywords self-test: {'PASS' if not fails else 'FAIL'} ({len(fails)} failing)")
    for f in fails:
        print("  - " + f)
    return 1 if fails else 0


def main(argv):
    text = GRAMMAR.read_text(encoding="utf-8")
    if "--self-test" in argv:
        return self_test(text)
    if "--write" in argv:
        GRAMMAR.write_text(write(text), encoding="utf-8")
        text = GRAMMAR.read_text(encoding="utf-8")
    problems, counts = compare(text)
    tag = " ".join(f"{k}={v}" for k, v in counts.items())
    if problems:
        print(f"SV-RESERVED-KEYWORDS: FAIL ({tag})")
        for p in problems:
            print("  - " + p)
        return 1
    print(f"SV-RESERVED-KEYWORDS: OK — {RULES['sv']} == Table B.1 and {RULES['v2005']} == 1364-2005 Annex B, both directions ({tag})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
