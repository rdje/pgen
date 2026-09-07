#!/usr/bin/env python3
r"""SV-CORPUS-GRAD.3.39 — the compiler-directive census classifier (`DIRECTIVE-CLASSIFY:`).

Reads an `ast_field_census.py` TSV taken at `--kind compiler_directive --field body` (one row per backtick
line the parser read as a `compiler_directive` node, plus one `regex` row per directive line in a file the
parser rejects) and buckets every row against the DIRECTIVE SYNTAX IEEE 1800-2017 clause 22 / IEEE
1800-2023 clause 22 / IEEE 1364-2005 clause 19 define for the thirteen names whose arguments the LRM
gives a syntax (`resetall`, `undef`, `undefineall`, `timescale`, `default_nettype`, `unconnected_drive`,
`nounconnected_drive`, `celldefine`, `endcelldefine`, `pragma`, `line`, `begin_keywords`, `end_keywords`):

    typed:<kind>              (release 1.0.217+) the parser read the directive as its TYPED node — legal by the clause's
                              own production, by construction; an `opaque` node is judged by its `text` below
    legal:<name>              the arguments match the clause's syntax box
    ILLEGAL:<name>:<reason>   they do not — the typed grammar REFUSES the line
    TRAILING:<name>           the directive is legal and non-comment text follows it on the same line
                              (the grammar hands that text to the NEXT item; the row is listed for reading)
    no_node                   the file was selected by text and the parser read NO directive node in it
                              (its backtick lines sit inside a comment or a string) — never a mover
    opaque:<name>             a name the parser tolerates as an opaque line (`include`, `define`, the
                              conditional family, a text-macro use, a vendor directive) — unchanged by .3.39
    macro_body:<name>         the line is a CONTINUATION LINE of a `define (the previous physical line ends
                              with `\`): macro text, not a directive at all; the file is preprocessor-owned
    unclassified              the reader could not even name the directive — a defect in this classifier

Every `ILLEGAL` / `TRAILING` row is printed with its file and text, and the PREDICTION block names each
ACCEPTED file that carries one (the release build must move exactly those, `pass -> fail`) and each already
REJECTED file whose regex rows carry one (the failure position may move; the adjudicator's explanation must
be re-read).

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/directive_census/classify.py <census.tsv> --profile sv_2017
  python3 docs/tasks/artifacts/sv_corpus_grad/directive_census/classify.py --self-test

Exit 0; exit 2 on an unreadable TSV; exit 7 on a self-test miss.  The version_specifier list is the
profile's edition: `sv_2017` = IEEE 1800-2017 Syntax 22-10, `sv_2023` = IEEE 1800-2023 Syntax 22-10 (adds
1800-2023), `verilog_2005` = IEEE 1364-2005 Syntax 19-10.

HONEST BOUND: this is a second transcription of the same clauses the grammar transcribes; agreement between
the two catches a transcription slip in either, never a shared misreading of the clause. The LRM lines are
cited beside each validator so a reader can re-read the clause, not the classifier.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[5]

TYPED = (
    "resetall", "undefineall", "undef", "timescale", "default_nettype", "unconnected_drive",
    "nounconnected_drive", "celldefine", "endcelldefine", "pragma", "line", "begin_keywords", "end_keywords",
)
NAME_RE = re.compile(r"^\s*`([A-Za-z_][A-Za-z0-9_$]*)(.*)$", re.S)
# A trailing comment is layout everywhere except after `line (22.12: "Comments are not allowed on the
# same line as a `line directive").
TRAIL_RE = re.compile(r"^\s*(?://.*|/\*.*)?$", re.S)
SIMPLE_ID = r"[A-Za-z_][A-Za-z0-9_$]*"
ESCAPED_ID = r"\\[!-~]+"
IDENT_RE = re.compile(rf"^\s+({SIMPLE_ID}|{ESCAPED_ID})(.*)$", re.S)
# 22.7 (docs/systemverilog/2017/md/section-22-compiler-directives.md:615-633): `timescale time_unit /
# time_precision; "the valid integers are 1, 10, and 100 ... the valid character strings are s, ms, us, ns,
# ps, and fs"; the clause's own examples put white space between the integer and the unit (`1 ns / 1 ps`).
TIME_RE = re.compile(r"^\s*(100|10|1)\s*(s|ms|us|ns|ps|fs)\b(.*)$", re.S)
# 22.8 (:681-686) / 19.2 (docs/verilog/2005/md/section-19-compiler-directives.md:114-121).
NETTYPES = ("wire", "tri", "tri0", "tri1", "wand", "triand", "wor", "trior", "trireg", "uwire", "none")
# 22.14 (:851-862) / 1800-2023 (docs/systemverilog/2023/md/section-22-compiler-directives.md:922-935) /
# 19.11 (:723-729).
VERSIONS = {
    "sv_2017": ("1800-2017", "1800-2012", "1800-2009", "1800-2005", "1364-2005", "1364-2001",
                "1364-2001-noconfig", "1364-1995"),
    "sv_2023": ("1800-2023", "1800-2017", "1800-2012", "1800-2009", "1800-2005", "1364-2005", "1364-2001",
                "1364-2001-noconfig", "1364-1995"),
    "verilog_2005": ("1364-1995", "1364-2001", "1364-2001-noconfig", "1364-2005"),
}
STRING_RE = re.compile(r'^\s*("(?:[^"\\]|\\.)*")(.*)$', re.S)
# 22.11 Syntax 22-8 (:734-763) / 19.10 Syntax 19-9 (:662-677): number | string | identifier | ( list ).
NUMBER_RE = re.compile(r"^\s*((?:[0-9][0-9_]*\s*)?'[sS]?[dDhHoObB]\s*[0-9a-fA-FxXzZ?_]+|[0-9][0-9_]*(?:\.[0-9][0-9_]*)?(?:[eE][+-]?[0-9][0-9_]*)?)(.*)$", re.S)
PRAGMA_ID_RE = re.compile(rf"^\s*({SIMPLE_ID})(.*)$", re.S)


def _trail(rest: str, name: str):
    """Classify what follows a complete directive: layout, or text for the next item."""
    if TRAIL_RE.match(rest):
        return f"legal:{name}"
    return f"TRAILING:{name}"


def _no_args(rest: str, name: str):
    return _trail(rest, name)


def _undef(rest: str):
    m = IDENT_RE.match(rest)
    if not m:
        return "ILLEGAL:undef:missing_identifier"
    return _trail(m.group(2), "undef")


def _timescale(rest: str):
    m = TIME_RE.match(rest)
    if not m:
        return "ILLEGAL:timescale:bad_time_unit"
    rest = m.group(3)
    m2 = re.match(r"^\s*/(.*)$", rest, re.S)
    if not m2:
        return "ILLEGAL:timescale:missing_precision"
    m3 = TIME_RE.match(m2.group(1))
    if not m3:
        return "ILLEGAL:timescale:bad_time_precision"
    return _trail(m3.group(3), "timescale")


def _default_nettype(rest: str):
    m = PRAGMA_ID_RE.match(rest)
    if not m:
        return "ILLEGAL:default_nettype:missing_value"
    if m.group(1) not in NETTYPES:
        return "ILLEGAL:default_nettype:bad_value"
    return _trail(m.group(2), "default_nettype")


def _unconnected_drive(rest: str):
    m = PRAGMA_ID_RE.match(rest)
    if not m:
        return "ILLEGAL:unconnected_drive:missing_value"
    if m.group(1) not in ("pull0", "pull1"):
        return "ILLEGAL:unconnected_drive:bad_value"
    return _trail(m.group(2), "unconnected_drive")


def _begin_keywords(rest: str, profile: str):
    m = STRING_RE.match(rest)
    if not m:
        return "ILLEGAL:begin_keywords:missing_or_unquoted_version"
    if m.group(1)[1:-1] not in VERSIONS[profile]:
        return "ILLEGAL:begin_keywords:unknown_version"
    return _trail(m.group(2), "begin_keywords")


def _line(rest: str):
    m = re.match(r"^\s+([1-9][0-9]*)\b(.*)$", rest, re.S)
    if not m:
        return "ILLEGAL:line:bad_number"
    m2 = STRING_RE.match(m.group(2))
    if not m2:
        return "ILLEGAL:line:bad_filename"
    m3 = re.match(r"^\s+([012])\b(.*)$", m2.group(2), re.S)
    if not m3:
        return "ILLEGAL:line:bad_level"
    if not re.match(r"^\s*$", m3.group(2)):
        # 22.12: only white space may appear on the same line; comments are not allowed either.
        return "ILLEGAL:line:text_after_level"
    return "legal:line"


def _pragma_value(s: str):
    """Consume one pragma_value; return the remainder or None."""
    m = re.match(r"^\s*\((.*)$", s, re.S)
    if m:
        rest = _pragma_expression(m.group(1))
        if rest is None:
            return None
        while True:
            m2 = re.match(r"^\s*,(.*)$", rest, re.S)
            if not m2:
                break
            rest = _pragma_expression(m2.group(1))
            if rest is None:
                return None
        m3 = re.match(r"^\s*\)(.*)$", rest, re.S)
        return m3.group(1) if m3 else None
    m = NUMBER_RE.match(s)
    if m:
        return m.group(2)
    m = STRING_RE.match(s)
    if m:
        return m.group(2)
    m = PRAGMA_ID_RE.match(s)
    if m:
        return m.group(2)
    return None


def _pragma_expression(s: str):
    """pragma_keyword = pragma_value | pragma_keyword | pragma_value (ordered as the grammar orders it)."""
    m = PRAGMA_ID_RE.match(s)
    if m:
        m2 = re.match(r"^\s*=(.*)$", m.group(2), re.S)
        if m2:
            return _pragma_value(m2.group(1))
        return m.group(2)
    return _pragma_value(s)


def _pragma(rest: str):
    m = PRAGMA_ID_RE.match(rest)
    if not m:
        return "ILLEGAL:pragma:missing_name"
    rest = m.group(2)
    if TRAIL_RE.match(rest):
        return "legal:pragma"
    rest2 = _pragma_expression(rest)
    if rest2 is None:
        return "ILLEGAL:pragma:bad_expression"
    while True:
        m2 = re.match(r"^\s*,(.*)$", rest2, re.S)
        if not m2:
            break
        rest2 = _pragma_expression(m2.group(1))
        if rest2 is None:
            return "ILLEGAL:pragma:bad_expression"
    return _trail(rest2, "pragma")


def classify(text: str, profile: str) -> str:
    m = NAME_RE.match(text)
    if not m:
        return "unclassified"
    name, rest = m.group(1), m.group(2)
    if name in ("resetall", "celldefine", "endcelldefine", "nounconnected_drive", "end_keywords"):
        return _no_args(rest, name)
    if name == "undefineall":
        # SV-only (22.5.3); absent from 1364-2005 clause 19, where it is a text-macro use.
        return _no_args(rest, name) if profile != "verilog_2005" else "opaque:undefineall"
    if name == "undef":
        return _undef(rest)
    if name == "timescale":
        return _timescale(rest)
    if name == "default_nettype":
        return _default_nettype(rest)
    if name == "unconnected_drive":
        return _unconnected_drive(rest)
    if name == "begin_keywords":
        return _begin_keywords(rest, profile)
    if name == "line":
        return _line(rest)
    if name == "pragma":
        return _pragma(rest)
    return f"opaque:{name}"


def is_macro_body(path: Path, text: str) -> bool:
    """Does this directive line sit on a `define continuation line (previous physical line ends in `\\`)?"""
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").split("\n")
    except OSError:
        return False
    key = text.strip()
    for i, ln in enumerate(lines):
        if ln.strip() == key or ln.strip().startswith(key):
            j = i - 1
            while j >= 0 and not lines[j].strip():
                j -= 1
            if j >= 0 and lines[j].rstrip().endswith("\\"):
                return True
    return False


def read_rows(tsv: Path):
    rows = []
    with tsv.open(encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        if header != ["file", "status", "index", "value"]:
            raise SystemExit(f"⛔ not an ast_field_census TSV (header {header!r}): {tsv}")
        for ln in fh:
            parts = ln.rstrip("\n").split("\t", 3)
            if len(parts) != 4:
                continue
            rows.append(parts)
    return rows


def run(tsv: Path, profile: str, root: Path):
    rows = read_rows(tsv)
    buckets: Counter = Counter()
    listed: list[tuple[str, str, str, str]] = []
    per_file_status: dict[str, str] = {}
    movers_accepted: dict[str, list[str]] = defaultdict(list)
    movers_rejected: dict[str, list[str]] = defaultdict(list)
    n_parser = n_regex = 0
    for f, status, index, value in rows:
        per_file_status[f] = status
        if value == "-" or value.startswith("<"):
            if value == "<no compiler_directive node>":
                # The file list is a TEXT selection; the parser read no directive node in this accepted
                # file (its backtick lines sit inside a comment or a string). Counted, never a mover.
                buckets["no_node"] += 1
                listed.append(("no_node", f, status, value))
            elif value.startswith("<"):
                buckets["unclassified"] += 1
                listed.append(("unclassified", f, status, value))
            continue
        if index.startswith("regex"):
            text = value
            n_regex += 1
        else:
            try:
                text = json.loads(value)
            except json.JSONDecodeError:
                buckets["unclassified"] += 1
                listed.append(("unclassified", f, status, value))
                continue
            if isinstance(text, dict):
                # Release 1.0.217 and later: the body IS the typed node. A typed kind was accepted by the
                # clause's own production, so it is legal by construction — `typed:<kind>`; an `opaque` node
                # carries the residue line as `text` and is judged textually, and a TYPED NAME arriving as
                # `opaque` would be a grammar defect (the guard let a typed name into the residue).
                n_parser += 1
                kind = text.get("kind")
                if kind == "opaque" and isinstance(text.get("text"), str):
                    verdict = classify(text["text"], profile)
                    if verdict == "unclassified":
                        # a backtick followed by something that is not an identifier — the `" / `` / `\" macro-text
                        # operators of 22.5.1 (sv2v string_directive.sv: `"asdf `line`") — residue by definition
                        verdict = "opaque:non_identifier"
                    elif not verdict.startswith("opaque"):
                        verdict = "ILLEGAL:opaque_carries_typed_name:" + verdict
                        listed.append((verdict, f, status, text["text"].strip()))
                    buckets[verdict.split(":")[0] + ":" + (verdict.split(":")[1] if ":" in verdict else "")] += 1
                    continue
                buckets[f"typed:{kind}"] += 1
                continue
            if not isinstance(text, str):
                buckets["unclassified"] += 1
                listed.append(("unclassified", f, status, value))
                continue
            n_parser += 1
        verdict = classify(text, profile)
        name = verdict.split(":")[1] if ":" in verdict else ""
        if (verdict.startswith("ILLEGAL") or verdict.startswith("TRAILING")) and is_macro_body(root / f, text):
            verdict = f"macro_body:{name}"
        buckets[verdict.split(":")[0] + (":" + name if name else "")] += 1
        if verdict.startswith(("ILLEGAL", "TRAILING", "macro_body")):
            listed.append((verdict, f, status, text.strip()))
            # A macro_body line is refused by the typed grammar exactly like an ILLEGAL one (its argument is a
            # macro formal), so the file's failure position may move onto it; it never sits in an ACCEPTED file
            # (a `define continuation line is macro text the isolation parse cannot read past).
            (movers_accepted if status == "accepted" else movers_rejected)[f].append(f"{verdict} :: {text.strip()}")
    typed = sum(v for k, v in buckets.items() if k.split(":")[0] in ("legal", "ILLEGAL", "TRAILING", "macro_body", "typed"))
    by_class = Counter(k.split(":")[0] for k in buckets.elements())
    print(f"DIRECTIVE-CLASSIFY: {tsv} profile={profile} rows={len(rows)} parser_rows={n_parser} regex_rows={n_regex} "
          f"typed={typed} typed_nodes={by_class['typed']} legal={by_class['legal']} ILLEGAL={by_class['ILLEGAL']} TRAILING={by_class['TRAILING']} "
          f"opaque={by_class['opaque']} macro_body={by_class['macro_body']} no_node={by_class['no_node']} "
          f"unclassified={by_class['unclassified']}")
    for k in sorted(buckets):
        print(f"  {k:48} {buckets[k]}")
    if listed:
        print("ROWS (every ILLEGAL / TRAILING / macro_body / no_node / unclassified row, file status in brackets):")
        for verdict, f, status, text in listed:
            print(f"  {verdict:44} [{status}] {f} :: {text}")
    print(f"PREDICTION accepted files that must move pass -> fail: {len(movers_accepted)}")
    for f in sorted(movers_accepted):
        for why in movers_accepted[f]:
            print(f"  MOVE  {f} :: {why}")
    print(f"PREDICTION rejected files whose failure position may move: {len(movers_rejected)}")
    for f in sorted(movers_rejected):
        for why in movers_rejected[f]:
            print(f"  STAYS {f} :: {why}")
    return by_class["unclassified"] == 0


SELF_TEST = [
    # (text, profile, expected verdict)
    ("`resetall", "sv_2017", "legal:resetall"),
    ("`resetall // a comment", "sv_2017", "legal:resetall"),
    ("`resetall junk", "sv_2017", "TRAILING:resetall"),
    ("`celldefine", "verilog_2005", "legal:celldefine"),
    ("`endcelldefine", "sv_2023", "legal:endcelldefine"),
    ("`undefineall", "sv_2017", "legal:undefineall"),
    ("`undefineall", "verilog_2005", "opaque:undefineall"),
    ("`undef FOO", "sv_2017", "legal:undef"),
    ("`undef \\a$b ", "sv_2017", "legal:undef"),
    ("`undef", "sv_2017", "ILLEGAL:undef:missing_identifier"),
    ("`timescale 1ns/1ps", "sv_2017", "legal:timescale"),
    ("`timescale 1 ns / 1 ps", "verilog_2005", "legal:timescale"),
    ("`timescale 10 us / 100 ns // c", "sv_2023", "legal:timescale"),
    ("`timescale 1ps/1ns", "sv_2017", "legal:timescale"),          # ordering is a VALUE rule (22.7 prose), not syntax
    ("`timescale frump", "sv_2017", "ILLEGAL:timescale:bad_time_unit"),
    ("`timescale 1xs", "sv_2017", "ILLEGAL:timescale:bad_time_unit"),
    ("`timescale 2ps", "sv_2017", "ILLEGAL:timescale:bad_time_unit"),
    ("`timescale 9 ns / 1 ps", "sv_2017", "ILLEGAL:timescale:bad_time_unit"),
    ("`timescale 1ns", "sv_2017", "ILLEGAL:timescale:missing_precision"),
    ("`timescale 1ns / frump", "sv_2017", "ILLEGAL:timescale:bad_time_precision"),
    ("`timescale 1ns / 1ps /extra", "sv_2017", "TRAILING:timescale"),
    ("`default_nettype none", "sv_2017", "legal:default_nettype"),
    ("`default_nettype uwire", "verilog_2005", "legal:default_nettype"),
    ("`default_nettype supply0", "sv_2017", "ILLEGAL:default_nettype:bad_value"),
    ("`default_nettype", "sv_2017", "ILLEGAL:default_nettype:missing_value"),
    ("`unconnected_drive pull1", "sv_2017", "legal:unconnected_drive"),
    ("`unconnected_drive pull2", "sv_2017", "ILLEGAL:unconnected_drive:bad_value"),
    ("`unconnected_drive", "sv_2017", "ILLEGAL:unconnected_drive:missing_value"),
    ("`nounconnected_drive", "sv_2017", "legal:nounconnected_drive"),
    ("`nounconnected_drive pull0", "sv_2017", "TRAILING:nounconnected_drive"),
    ("`pragma protect begin", "sv_2017", "legal:pragma"),
    ("`pragma protect encoding = (enctype = \"BASE64\", line_length = 76, bytes = 128)", "sv_2017", "legal:pragma"),
    ("`pragma foo something, somethingelse = 7, \"abcdef\", ( hello, ( \"world\", pr = 4, \"gm\" ) ), a", "sv_2017", "legal:pragma"),
    ("`pragma once", "sv_2017", "legal:pragma"),
    ("`pragma", "sv_2017", "ILLEGAL:pragma:missing_name"),
    ("`pragma foo = 1", "sv_2017", "ILLEGAL:pragma:bad_expression"),
    ("`pragma foo a,", "sv_2017", "ILLEGAL:pragma:bad_expression"),
    ("`line 3 \"orig.v\" 2", "sv_2017", "legal:line"),
    ("`line 1 \"somefile\" 3", "sv_2017", "ILLEGAL:line:bad_level"),
    ("`line 1 somefile 2", "sv_2017", "ILLEGAL:line:bad_filename"),
    ("`line -12 \"somefile\" 3", "sv_2017", "ILLEGAL:line:bad_number"),
    ("`line 1 \"somefile\"", "sv_2017", "ILLEGAL:line:bad_level"),
    ("`line 1", "sv_2017", "ILLEGAL:line:bad_filename"),
    ("`line", "sv_2017", "ILLEGAL:line:bad_number"),
    ("`line 3 \"orig.v\" 2 // no comment allowed here", "sv_2017", "ILLEGAL:line:text_after_level"),
    ("`line `__LINE__ \"f\" 1", "sv_2017", "ILLEGAL:line:bad_number"),
    ("`begin_keywords \"1800-2017\"", "sv_2017", "legal:begin_keywords"),
    ("`begin_keywords \"1800-2023\"", "sv_2023", "legal:begin_keywords"),
    ("`begin_keywords \"1800-2023\"", "sv_2017", "ILLEGAL:begin_keywords:unknown_version"),
    ("`begin_keywords \"1800-2017\"", "verilog_2005", "ILLEGAL:begin_keywords:unknown_version"),
    ("`begin_keywords \"1364-2001-noconfig\"", "verilog_2005", "legal:begin_keywords"),
    ("`begin_keywords \"VAMS-2.3\"", "sv_2017", "ILLEGAL:begin_keywords:unknown_version"),
    ("`begin_keywords 1800-2017", "sv_2017", "ILLEGAL:begin_keywords:missing_or_unquoted_version"),
    ("`end_keywords", "sv_2017", "legal:end_keywords"),
    ("`include \"x.svh\"", "sv_2017", "opaque:include"),
    ("`define X 1", "sv_2017", "opaque:define"),
    ("`MY_MACRO(a, b)", "sv_2017", "opaque:MY_MACRO"),
    ("`default_trireg_strength this_is_optional", "sv_2017", "opaque:default_trireg_strength"),
    ("junk without a backtick", "sv_2017", "unclassified"),
]


def self_test() -> int:
    misses = 0
    for text, profile, expected in SELF_TEST:
        got = classify(text, profile)
        if got != expected:
            misses += 1
            print(f"  MISS {text!r} ({profile}): expected {expected} got {got}")
    # RED arm: an illegal form must never read legal for any name.
    red = [(t, p) for t, p, e in SELF_TEST if e.startswith("ILLEGAL")]
    if any(classify(t, p).startswith("legal") for t, p in red):
        misses += 1
        print("  MISS red arm: an ILLEGAL form classified legal")
    # macro_body arm on a synthetic file.
    import tempfile
    with tempfile.TemporaryDirectory(dir=str(REPO_ROOT / "rust/target")) as td:
        p = Path(td) / "m.sv"
        p.write_text("`define T(t) \\\n    `default_nettype t \\\n    module m; endmodule\n`default_nettype none\n")
        if not is_macro_body(p, "`default_nettype t \\"):
            misses += 1; print("  MISS macro_body: continuation line not recognised")
        if is_macro_body(p, "`default_nettype none"):
            misses += 1; print("  MISS macro_body: a top-level directive read as macro text")
    # typed-node arms (release 1.0.217+ parser rows): a typed node is `typed:<kind>`; an opaque node carrying a
    # typed name is a DEFECT the census must surface, never a legal row.
    with tempfile.TemporaryDirectory(dir=str(REPO_ROOT / "rust/target")) as td:
        tsv = Path(td) / "c.tsv"
        tsv.write_text("file\tstatus\tindex\tvalue\n"
                       f"{td}/x.sv\taccepted\t0\t" + json.dumps({"kind": "timescale", "unit": {}, "precision": {}}) + "\n"
                       f"{td}/x.sv\taccepted\t1\t" + json.dumps({"kind": "opaque", "text": "`include \"x.svh\""}) + "\n"
                       f"{td}/x.sv\taccepted\t2\t" + json.dumps({"kind": "opaque", "text": "`timescale frump"}) + "\n"
                       f"{td}/x.sv\taccepted\t3\t" + json.dumps({"kind": "opaque", "text": "`\"asdf `line`\""}) + "\n")
        import io, contextlib
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            run(tsv, "sv_2017", Path(td))
        out = buf.getvalue()
        if "typed:timescale" not in out or "opaque:include" not in out:
            misses += 1; print("  MISS typed-node arm: typed / opaque rows not bucketed")
        if "ILLEGAL:opaque_carries_typed_name" not in out:
            misses += 1; print("  MISS red arm: an opaque node carrying a typed name must be ILLEGAL")
        if "opaque:non_identifier" not in out or out.count("ILLEGAL:opaque_carries_typed_name") != 2:
            misses += 1; print("  MISS quote-operator arm: `\" is residue, never a typed-name defect")
    print(f"DIRECTIVE-CLASSIFY-SELF-TEST: arms={len(SELF_TEST) + 6} misses={misses}")
    return 7 if misses else 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("tsv", nargs="?")
    ap.add_argument("--profile", default="sv_2017", choices=sorted(VERSIONS))
    ap.add_argument("--root", default=str(REPO_ROOT))
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if not args.tsv:
        ap.error("a census TSV or --self-test is required")
    tsv = Path(args.tsv)
    if not tsv.is_file():
        print(f"⛔ no such census: {tsv}", file=sys.stderr)
        return 2
    ok = run(tsv, args.profile, Path(args.root))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
