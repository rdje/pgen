#!/usr/bin/env python3
r"""SV-CORPUS-GRAD.3.43 — classify every DPI import function RESULT type a corpus writes against IEEE 1800-2017
§35.5.5 (docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493; 1800-2023 :493-501 reads
the same): legal results are `void, byte, shortint, int, longint, real, shortreal, chandle, string` and SCALAR
`bit` / `logic`.  Input: the TSV of `ast_field_census.py` (kind=import_function field=proto.body.body.return_type).

Categories (the grammar's decision, `dpi_function_result_type`):
  legal_keyword   one of the eleven listed types (signing on the atom / vector types admitted — §35.5.5 is silent on it)
  named           a type identifier (scoped or a known typedef) or a `type(...)` reference with NO packed dimension —
                  what it resolves to is a SEMANTIC question the grammar cannot decide, so it is admitted
  ILLEGAL         everything else the letter excludes and a parser can see: a packed dimension on any base,
                  `reg`, `integer`, `time`, `realtime`, an enum / struct / union literal, `event`, a virtual interface,
                  a class type, a covergroup type
  unclassified    a rendering this classifier does not know (printed; a census with any of these is not finished)

Usage: python3 docs/tasks/artifacts/sv_corpus_grad/dpi_result_census/classify.py <census.tsv> [<census2.tsv> ...]
Prints `DPI-RESULT-CLASSIFY: rows=N parser=P regex=R legal_keyword=A named=B ILLEGAL=C unclassified=D` per file,
then every ILLEGAL and unclassified row (these are the pre-registered `pass -> fail` corpus rows when they sit in an
ACCEPTING file — a `regex` row sits in a file the shipped parser already rejects, so it cannot move).
"""
import json, re, sys, collections

ATOM_OK = {"byte", "shortint", "int", "longint"}
VEC_OK = {"bit", "logic"}
NONINT_OK = {"real", "shortreal"}

def classify_json(v):
    if v == {"kind": "void"}:
        return "legal_keyword", "void"
    if not (isinstance(v, dict) and v.get("kind") == "data_type"):
        return "unclassified", json.dumps(v)[:80]
    b = v.get("body", {})
    k = b.get("kind")
    base = (b.get("base") or {}).get("kind")
    dims = b.get("dims", [])
    sig = b.get("signing")
    sigtxt = "" if not sig else (" " + sig.get("kind", "?"))
    if k == "integer_atom":
        return ("legal_keyword" if base in ATOM_OK else "ILLEGAL"), base + sigtxt
    if k == "integer_vector":
        if dims:
            return "ILLEGAL", "%s%s [packed x%d]" % (base, sigtxt, len(dims))
        return ("legal_keyword" if base in VEC_OK else "ILLEGAL"), base + sigtxt
    if k == "non_integer":
        return ("legal_keyword" if base in NONINT_OK else "ILLEGAL"), base
    if k in ("string", "chandle"):
        return "legal_keyword", k
    if k in ("scoped_data_type", "known_unscoped_data_type"):
        if dims:
            return "ILLEGAL", "%s [packed x%d]" % (k, len(dims))
        return "named", k
    if k == "type_reference":
        return "named", "type(...)"
    if k in ("class_type", "provisional_class_type", "enum", "struct_union", "event", "virtual_interface", "covergroup"):
        return "ILLEGAL", k
    return "unclassified", json.dumps(v)[:80]

def classify_text(t):
    t = " ".join(t.split())
    if re.fullmatch(r"void", t): return "legal_keyword", t
    if re.fullmatch(r"(byte|shortint|int|longint)( (un)?signed)?", t): return "legal_keyword", t
    if re.fullmatch(r"(integer|time)( (un)?signed)?", t): return "ILLEGAL", t
    if re.fullmatch(r"(bit|logic)( (un)?signed)?", t): return "legal_keyword", t
    if re.fullmatch(r"(bit|logic|reg)( (un)?signed)?( ?\[[^\]]*\])+", t): return "ILLEGAL", t + "  (packed)"
    if re.fullmatch(r"reg( (un)?signed)?", t): return "ILLEGAL", t
    if re.fullmatch(r"(real|shortreal)", t): return "legal_keyword", t
    if re.fullmatch(r"realtime", t): return "ILLEGAL", t
    if re.fullmatch(r"(string|chandle)", t): return "legal_keyword", t
    if re.match(r"(enum|struct|union|event|virtual)\b", t): return "ILLEGAL", t
    if re.fullmatch(r"(\w+::)*\w+( ?\[[^\]]*\])+", t): return "ILLEGAL", t + "  (typedef with packed dims)"
    if re.fullmatch(r"(\w+::)*\w+(\s*#\s*\(.*\))?", t):
        return ("ILLEGAL" if "#" in t else "named"), t
    if re.fullmatch(r"type\s*\(.*\)", t): return "named", t
    return "unclassified", t

def main():
    for path in sys.argv[1:]:
        rows = [l.rstrip("\n").split("\t") for l in open(path).read().splitlines()[1:]]
        cnt = collections.Counter(); src = collections.Counter(); flagged = []; renderings = collections.Counter()
        for f, status, idx, val in rows:
            if idx == "-":
                continue
            if idx.startswith("regex"):
                cat, r = classify_text(val); src["regex"] += 1
            else:
                cat, r = classify_json(json.loads(val)); src["parser"] += 1
            cnt[cat] += 1; renderings[(cat, r)] += 1
            if cat in ("ILLEGAL", "unclassified"):
                flagged.append((cat, status, f, r))
        print("DPI-RESULT-CLASSIFY: %s rows=%d parser=%d regex=%d legal_keyword=%d named=%d ILLEGAL=%d unclassified=%d"
              % (path, src["parser"] + src["regex"], src["parser"], src["regex"], cnt["legal_keyword"], cnt["named"], cnt["ILLEGAL"], cnt["unclassified"]))
        for (cat, r), n in sorted(renderings.items(), key=lambda x: (-x[1], x[0])):
            print("  %6d  %-14s %s" % (n, cat, r))
        for cat, status, f, r in flagged:
            print("  %s\t%s\t%s\t%s" % (cat, status, f, r))

if __name__ == "__main__":
    main()
