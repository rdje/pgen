#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13k(b) — the v2005 REJECTS-VALID census, keyed by the construct that stops
the parse.

WHAT IT ANSWERS
  `.13k` opened on three rows of `adjudication_manifest_v2005.tsv` that are keyed `must_accept`
  on a SUITE's word and that the parser rejects, and it asserted *"three is a sample"*. This
  census enumerates the whole population that claim is about — every row adjudicated
  `divergence:unexplained_rejects_valid` in the `verilog_2005` lane — and reports, per row, the
  furthest byte the parser reached and the text sitting there.

⛔⛔ WHY THE BASIS CENSUS IS THE WRONG POPULATION, MEASURED
  `.13k`(b) proposed classifying by BASIS: every `must_accept` row resting on a suite contract
  rather than a clause citation. Re-using `key_provenance_census.py`'s classifier (never a second
  copy of it) that population is **2 322 of 2 479 rows, 93.7 %** — which is, to within a tenth of
  a point, the corpus-wide trust bound that census already publishes (93.8 %). It is a true number
  and it is not a defect list: 2 085 of those rows sit at `match`, i.e. the parser agrees with
  them. ⇒ the basis is a TRUST BOUND, not a fault; the DECIDABLE class is the rows where the key
  and the parser actually disagree, and there are 53 of them.

⭐⭐ AND THE 53 ARE NOT ONE DEFECT. Grouped by the construct at the stop point and adjudicated
  against the IEEE 1364-2005 text vendored in `docs/verilog/2005/md/`:
    - text that Annex A CANNOT derive (`++`, `+=`, `-=`, `|=`, `logic`, …) ⇒ the ANSWER KEY is
      wrong and the parser is right, which is `.13k`'s class;
    - text Annex A DOES define (`tri0`/`tri1` at A.2.2.1:219, a UDP `table` at A.5.3:594) ⇒ the
      `verilog_2005` PROFILE is under-accepting, which is a parser defect and a different lane.
  Both verdicts are re-derivable from this repository: `net_type ::= … tri0 | tri1` is at
  `section-Annex_A-…:219`, while `grep -c '\blogic\b'` and a grep for `++` / `+=` over the same
  Annex return **0**.

DETERMINISTIC: output ordering is (suite, relpath); content-derived only; no network, no cargo.

Usage:
  python3 stimuli/sv/v2005_rejects_valid_census.py [--outdir DIR]
  python3 stimuli/sv/v2005_rejects_valid_census.py --self-test
"""
import argparse
import collections
import csv
import io
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv"
POSITIONS = ROOT / "stimuli/sv/characterization/positions_v2005.tsv"
SUBS = ROOT / "stimuli/sv/subs"
TARGET = "divergence:unexplained_rejects_valid"
# ⛔ THE WINDOW LOOKS BOTH WAYS, AND THAT IS A CORRECTION, NOT A PREFERENCE. `furthest_position`
# is where the parser STOPPED, so the construct that stopped it frequently ENDS at that byte
# rather than beginning there. A forward-only 48-byte window left 10 of 53 rows UNCLASSIFIED and
# mis-framed them all in the same direction: `br_gh1178a.v` reads `wire [7:0] replicated;` forward
# of its stop point while the `logic` that the verilog_2005 profile cannot derive sits BEHIND it —
# and its two sibling files, whose stop points happen to fall a few bytes earlier, classified
# correctly on the same construct. A window is a frame, and this one was measuring the text after
# the question rather than the question.
WINDOW_BACK = 56
WINDOW_FWD = 48

# ── the construct vocabulary ─────────────────────────────────────────────────────────────────
#
# ⛔ ENUMERATED FROM THE POPULATION, NEVER GUESSED — the discipline `key_provenance_census.py`
# paid for twice. Each probe carries the EDITION verdict and the in-repo citation that settles
# it, so a reader re-derives the adjudication instead of trusting this table.
#
# `edition` is the only judgement here:
#   "1800_only"  Annex A of IEEE 1364-2005 cannot derive this text ⇒ the KEY is wrong.
#   "1364_legal" Annex A DOES define it ⇒ the verilog_2005 PROFILE is under-accepting.
#   "preproc"    a `define / macro question, owned by the svpp lane, not by either of the above.
PROBES = [
    ("inc_dec_operator",       r"\+\+|--",
     "1800_only",  "IEEE 1800 A.6.2 inc_or_dec_operator; grep for '++' over 1364-2005 Annex A = 0"),
    ("assignment_operator",    r"[-+*/%&|^]=|<<=|>>=|<<<=|>>>=",
     "1800_only",  "IEEE 1800 A.6.2 assignment_operator; 1364-2005 A.6.2 is 'variable_lvalue = expression'"),
    ("logic_keyword",          r"\blogic\b",
     "1800_only",  "IEEE 1800 A.2.2.1 integer_vector_type; grep '\\blogic\\b' over 1364-2005 Annex A = 0"),
    ("inside_operator",        r"\binside\b",
     "1800_only",  "IEEE 1800 A.8.3 inside_expression; absent from 1364-2005 Annex A"),
    ("net_type_tri01",         r"\btri[01]\b",
     "1364_legal", "IEEE 1364-2005 Annex A A.2.2.1:219 net_type ::= ... | tri0 | tri1"),
    ("udp_table_entry",        r"\?\s*:|\(\s*[01x?*]{2}\s*\)|\bendtable\b",
     "1364_legal", "IEEE 1364-2005 Annex A A.5.3:594 udp_body ::= combinational_body | sequential_body"),
    ("specify_path_delay",     r"=>|\bendspecify\b|\bspecparam\b",
     "1364_legal", "IEEE 1364-2005 Annex A A.7.4 path_delay_value / A.7.2 specify_block"),
    ("gate_strength",          r"\b(?:pulldown|pullup|pull[01]|weak[01]|strong[01]|supply[01])\b",
     "1364_legal", "IEEE 1364-2005 Annex A A.3.1 pulldown/pullup + A.2.2.2 drive_strength"),
    ("macro_directive",        r"`\w+",
     "preproc",    "IEEE 1364-2005 clause 19 compiler directives; owned by the svpp lane"),
    ("block_comment",          r"/\*",
     "1364_legal", "IEEE 1364-2005 clause 3.7 comments; a lexical question, not a grammar one"),
]
EDITION_ORDER = ("1800_only", "1364_legal", "preproc", "UNCLASSIFIED")


def stop_text(path: Path, pos: int):
    """The text AROUND the furthest byte the parser reached, plus the 1-based line it sits on."""
    t = io.open(path, encoding="utf-8", errors="replace").read()
    return t[max(0, pos - WINDOW_BACK):pos + WINDOW_FWD], t.count("\n", 0, pos) + 1


def classify(fragment: str):
    """First matching probe wins; PROBES is ordered most-specific-first.

    ⛔ Returns UNCLASSIFIED rather than a default. A row nobody classified is not a row that
    passed — the founding lesson of every census in this tree.
    """
    for name, pat, edition, cite in PROBES:
        if re.search(pat, fragment):
            return name, edition, cite
    return "(unclassified)", "UNCLASSIFIED", "no probe matched — adjudicate by hand and add one"


def load():
    pos = {}
    for line in io.open(POSITIONS, encoding="utf-8"):
        p = line.rstrip("\n").split("\t")
        if len(p) >= 3:
            pos[p[1]] = int(p[2])
    out = []
    with io.open(MANIFEST, encoding="utf-8") as fh:
        for r in csv.DictReader(fh, delimiter="\t"):
            if r["adjudication"] != TARGET:
                continue
            rel = f"stimuli/sv/subs/{r['suite']}/{r['relpath']}"
            p = pos.get(rel)
            frag, line_no = ("", None)
            if p is not None and (ROOT / rel).is_file():
                frag, line_no = stop_text(ROOT / rel, p)
            name, edition, cite = classify(frag)
            out.append(dict(suite=r["suite"], relpath=r["relpath"], pos=p, line=line_no,
                            construct=name, edition=edition, cite=cite,
                            fragment=frag.replace("\n", "\\n").replace("\t", " ").strip(),
                            basis=r["basis"]))
    out.sort(key=lambda d: (d["suite"], d["relpath"]))
    return out


def code(text):
    """Wrap `text` in a code span that can actually HOLD it.

    ⛔ MEASURED, not defensive: corpus fragments contain backticks — a Verilog compiler directive
    is spelled `` `define ``, and the census population has five preprocessor rows. A
    single-backtick span cannot contain a backtick, so `f"`{fragment}`"` produced **6 cells with an
    unpaired backtick run**, which moves NO cell count and is therefore invisible to an arity
    check. `TABLE-ARITY-RATCHET`'s second arm caught it; my own arity pass over the same file
    reported 0 mismatches, which is exactly why that arm exists (LIVE-DOC-CONTAINMENT.11: a count
    is not a schema).

    The rule CommonMark gives: the delimiter must be LONGER than the longest backtick run inside,
    and the content is padded with one space on each side when it starts or ends with a backtick.
    """
    runs = [len(r) for r in re.findall(r"`+", text)]
    fence = "`" * ((max(runs) + 1) if runs else 1)
    pad = " " if text.startswith("`") or text.endswith("`") else ""
    return f"{fence}{pad}{text}{pad}{fence}"


def cell(text):
    """A markdown table cell. ⛔ `|` is ESCAPED, never trusted to be absent: two of the citations
    below are BNF alternations (`net_type ::= ... | tri0 | tri1`) whose own text is a pipe, and an
    unescaped one silently splits the row — LIVE-DOC-CONTAINMENT.8's founding defect, which
    TABLE-ARITY-RATCHET now blocks."""
    return text.replace("|", "\\|")


def render(rows):
    by_ed = collections.Counter(r["edition"] for r in rows)
    by_con = collections.Counter(r["construct"] for r in rows)
    L = ["# The `verilog_2005` rejects-valid census — keyed by the construct that stops the parse",
         "",
         "> Produced by `stimuli/sv/v2005_rejects_valid_census.py` from the tracked v2005",
         "> adjudication manifest and the `positions_v2005.tsv` sidecar. Deterministic and",
         "> content-derived: re-run it and diff.",
         "",
         "⛔ **This is the population `SV-CORPUS-GRAD.13k` is about** — every row the",
         "`verilog_2005` answer key calls `must_accept` and the parser rejects. `.13k` opened on",
         "three of them and said *\"three is a sample\"*. It is, and the sample was **not**",
         "representative: the class is TWO defects, not one.",
         "",
         "## The split",
         "",
         "| edition verdict | rows | what it means |",
         "|---|---|---|",
         f"| `1800_only` | {by_ed['1800_only']} | Annex A of IEEE 1364-2005 cannot derive the text ⇒ **the ANSWER KEY is wrong** and the parser is right |",
         f"| `1364_legal` | {by_ed['1364_legal']} | Annex A **does** define it ⇒ **the `verilog_2005` PROFILE is under-accepting** — a parser defect, a different lane |",
         f"| `preproc` | {by_ed['preproc']} | a compiler-directive question, owned by the svpp lane |",
         f"| `UNCLASSIFIED` | {by_ed['UNCLASSIFIED']} | no probe matched; adjudicate by hand and add one |",
         f"| **total** | **{len(rows)}** | |",
         "",
         "## By construct",
         "",
         "| construct | rows | edition | in-repo citation that settles it |",
         "|---|---|---|---|"]
    seen = {}
    for r in rows:
        seen.setdefault(r["construct"], (r["edition"], r["cite"]))
    for c, n in sorted(by_con.items(), key=lambda kv: (-kv[1], kv[0])):
        ed, cite = seen[c]
        L.append(f"| {code(c)} | {n} | {code(ed)} | {cell(cite)} |")
    L += ["", "## Every row", "",
          "| suite | file | line | construct | edition | text at the stop point |",
          "|---|---|---|---|---|---|"]
    for r in rows:
        # ⛔ TRUNCATE FIRST, THEN FENCE. Fencing then slicing can cut the closing delimiter off
        # and re-create the unpaired run this helper exists to prevent.
        frag = cell(r["fragment"])[:56]
        L.append(f"| {r['suite']} | {code(r['relpath'])} | {r['line']} | {code(r['construct'])} "
                 f"| {code(r['edition'])} | {code(frag)} |")
    L += ["", "⛔ **This census adjudicates the CONSTRUCT, not the row.** Whether a given file is",
          "`must_reject` or `divergence:explained` with an edition-contrast reason is `.13k`(a)",
          "and `(c)`; editing the key so it agrees with the parser is the gaming",
          "`EXTERNAL-CORPUS` forbids, and it is not done here.", ""]
    return "\n".join(L)


def self_test():
    cases = [
        ("for (i = 0; i < 3; i++)", "inc_dec_operator", "1800_only"),
        ("counter += 1;", "assignment_operator", "1800_only"),
        ("tmp |= i[3:0];", "assignment_operator", "1800_only"),
        ("logic [3:0] bus;", "logic_keyword", "1800_only"),
        ("tri0 t0 = enable ? val : 1'bz;", "net_type_tri01", "1364_legal"),
        ("(01) 0 ? : ? : 0 ;", "udp_table_entry", "1364_legal"),
        ("(clk => q) = (tR, tF);", "specify_path_delay", "1364_legal"),
        ("pulldown (weak0) pd2 (r2a);", "gate_strength", "1364_legal"),
        ("`define MACRO 1", "macro_directive", "preproc"),
        # ⛔ the NEGATIVE control: ordinary 1364-2005 text no probe should claim
        ("wire [7:0] replicated;", "(unclassified)", "UNCLASSIFIED"),
        # ⛔ PRECEDENCE: `++` before the compound-assignment probe, since `+=` and `++` overlap
        # on neither pattern but a looser assignment regex would swallow `++` and mislabel it.
        ("i++) begin", "inc_dec_operator", "1800_only"),
        # ⛔ a 1800-only construct must WIN over a 1364-legal one in the same window, because the
        # stop point is what the parser could not derive and the 1800 token is why.
        ("logic [3:0] tmp; tri0 x;", "logic_keyword", "1800_only"),
    ]
    misses = 0
    for text, want_con, want_ed in cases:
        con, ed, _ = classify(text)
        if (con, ed) != (want_con, want_ed):
            print(f"  ✗ {text!r}: want ({want_con}, {want_ed}) got ({con}, {ed})", file=sys.stderr)
            misses += 1
    print(f"V2005-REJECTS-VALID-CENSUS SELF-TEST: arms={len(cases)} failed={misses}",
          file=sys.stderr)
    return 2 if misses else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--outdir",
                    default=ROOT / "docs/tasks/artifacts/sv_corpus_grad/v2005_rejects_valid",
                    type=Path)
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if self_test():
        print("census: the classifier does not discriminate; refusing", file=sys.stderr)
        return 2
    rows = load()
    args.outdir.mkdir(parents=True, exist_ok=True)
    (args.outdir / "census.md").write_text(render(rows) + "\n", encoding="utf-8")
    with (args.outdir / "census.tsv").open("w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["suite", "relpath", "position", "line", "construct", "edition", "fragment"])
        for r in rows:
            w.writerow([r["suite"], r["relpath"], r["pos"], r["line"], r["construct"],
                        r["edition"], r["fragment"]])
    by = collections.Counter(r["edition"] for r in rows)
    print("V2005-REJECTS-VALID-CENSUS: rows={} {}".format(
        len(rows), " ".join(f"{e}={by[e]}" for e in EDITION_ORDER)))
    print(f"wrote {(args.outdir / 'census.md').relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
