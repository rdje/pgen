#!/usr/bin/env python3
"""SV-CORPUS-GRAD.14b.3 — PRODUCTION PARITY between IEEE 1800-2017 and 1800-2023 Annex A, for the
`must_reject` rows the sv_2023 lane INHERITS from the 2017 key on a cited 2017 production.

A 2017-lane `must_reject` whose basis says "1800-2017 Annex A <production> derives nothing for this
text" carries over to the 2023 lane ONLY if the 2023 production reads the same. This instrument
extracts every production name a row's basis cites (`name ::=`), pulls both editions' bodies out of
the in-repo LRM texts, normalises them (footnote digits, whitespace, code fences) and reports
IDENTICAL or DIFFERS per production — the mechanical half of `.14b.3`; a DIFFERS is read by hand.

Usage: python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/production_parity.py [--self-test]
"""
from __future__ import annotations
import csv, re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[5]
A17 = ROOT / "docs/systemverilog/2017/md/section-41-data-read-api.md"
A23 = ROOT / "docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md"
MANIFEST = ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv"
HEAD_RE = re.compile(r"^([a-z_][a-z_0-9]*)\s*::=\s*(.*)$")


FOOTER_RE = re.compile(r"Authorized licensed use limited to|Downloaded on .* from IEEE Xplore|Restrictions apply|^IEEE Std 1800-20\d\d|IEEE Standard for SystemVerilog|^Copyright |^\d{1,4}$|^A\.\d+(\.\d+)*\s|^#+ |^Footnotes")


def productions(text: str) -> tuple[dict[str, str], list[tuple[int, str]]]:
    """(name -> normalised body, [(line_no, name)] heads in order). A body runs from `name ::=` to the
    next head; code fences and the PDF page furniture (the IEEE footer/header lines) are dropped. A head
    carrying a footnote digit (`list_of_port_connections29`) is ALSO registered under its plain name."""
    out, heads, cur, body = {}, [], None, []
    def close():
        if cur:
            b = normalise(" ".join(body)); out.setdefault(cur, b)
            plain = re.sub(r"(?<=[a-z_])\d+$", "", cur)
            if plain != cur and plain not in out: out[plain] = b
    for i, line in enumerate(text.splitlines(), 1):
        st = line.strip()
        m = HEAD_RE.match(st)
        if m:
            close(); cur, body = m.group(1), [m.group(2)]; heads.append((i, cur)); continue
        if st.startswith("```") or FOOTER_RE.search(st):
            continue
        if cur is not None:
            body.append(st)
    close()
    return out, heads


def enclosing(heads: list[tuple[int, str]], line: int) -> str | None:
    """The production whose head is the nearest at or before `line`."""
    name = None
    for ln, n in heads:
        if ln <= line: name = n
        else: break
    return re.sub(r"(?<=[a-z_])\d+$", "", name) if name else None


def normalise(s: str) -> str:
    s = re.sub(r"([a-z_\]\)])\d+\b", r"\1", s)      # footnote digits glued to names or brackets (this46, ]15)
    s = re.sub(r"\s*([()\[\]{}|.*])\s*", r" \1 ", s)  # one spacing for punctuation ("@@(" == "@@ (", ".*" == ". *")
    s = re.sub(r"\s+_", "_", s)                         # a name broken across a PDF line ("constant _expression")
    s = re.sub(r"\s+", " ", s).strip()
    s = re.sub(r"\s\d{1,3}$", "", s)                    # a footnote index left standing after the body ('z_or_x 48)
    return s


def cited_names(basis: str, heads17: list[tuple[int, str]] | None = None) -> list[str]:
    """Production names the basis cites — by `name ::=`, and by the 2017 LINE RANGES it cites
    (`section-41-data-read-api.md:N-M` -> the production enclosing each cited line)."""
    names = re.findall(r"\b([a-z_][a-z_0-9]*)\s*::=", basis)
    if heads17:
        for a, b in re.findall(r"section-41-data-read-api\.md:(\d+)(?:-(\d+))?", basis):
            lo, hi = int(a), int(b or a)
            for ln in range(lo, min(hi, lo + 40) + 1):
                n = enclosing(heads17, ln)
                if n: names.append(n)
    seen, out = set(), []
    for n in names:
        n = re.sub(r"(?<=[a-z_])\d+$", "", n) if n not in ("delay2", "delay3") else n
        if n not in seen: seen.add(n); out.append(n)
    return out


def main() -> int:
    if "--self-test" in sys.argv:
        p, heads = productions("x ::=\n```\n\na | b3\n```ebnf\nAuthorized licensed use limited to: nobody\ny22 ::= c ( d )\n")
        ok = (p["x"] == "a | b" and p["y22"] == p["y"] == "c ( d )" and heads == [(1, "x"), (7, "y22")]
              and cited_names("IEEE foo ::= a | b (x:1); bar ::= c") == ["foo", "bar"]
              and enclosing(heads, 3) == "x" and enclosing(heads, 8) == "y"
              and normalise("@@( e )15") == normalise("@@ ( e )")
              and productions("z ::= q\n1176\nA.8.8 Strings\n| r\n")[0]["z"] == "q | r")
        if not ok: print(p, heads)
        print(f"PRODUCTION-PARITY-SELFTEST: {'PASS' if ok else 'FAIL'}"); return 0 if ok else 1
    (p17, heads17), (p23, _h23) = productions(A17.read_text(encoding="utf-8")), productions(A23.read_text(encoding="utf-8"))
    rows = [r for r in csv.DictReader(MANIFEST.open(encoding="utf-8"), delimiter="\t")
            if r["expected"] == "must_reject" and "section-41-data-read-api.md" in r["basis"] and "EDITION CONTRAST" not in r["basis"]]
    verdicts, per_name = [], {}
    for r in rows:
        names = cited_names(r["basis"], heads17)
        vs = []
        for n in names:
            if n not in per_name:
                a, b = p17.get(n), p23.get(n)
                per_name[n] = ("MISSING-2017" if a is None else "MISSING-2023" if b is None else "IDENTICAL" if a == b else "DIFFERS")
            vs.append(f"{n}:{per_name[n]}")
        verdicts.append((r["suite"], r["relpath"], vs))
    n_rows = len(verdicts); n_nocite = sum(1 for _, _, v in verdicts if not v)
    n_all_ident = sum(1 for _, _, v in verdicts if v and all(x.endswith(":IDENTICAL") for x in v))
    n_diff = sum(1 for _, _, v in verdicts if any(":DIFFERS" in x for x in v))
    n_missing = sum(1 for _, _, v in verdicts if any("MISSING" in x for x in v))
    print(f"PRODUCTION-PARITY: rows={n_rows} all_cited_identical={n_all_ident} some_differs={n_diff} some_missing={n_missing} no_production_named={n_nocite} productions={len(per_name)}")
    for name, v in sorted(per_name.items()):
        if v != "IDENTICAL": print(f"  {v:13s} {name}")
    for s, rel, v in verdicts:
        if not v or any(":DIFFERS" in x or "MISSING" in x for x in v):
            print(f"  ROW {s} {rel}: {' '.join(v) or '(no production named in the basis)'}")
    if "--write" in sys.argv or "--check" in sys.argv:
        import hashlib
        def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
        tsv = ["suite\trelpath\tverdict\tproductions"]
        for s_, rel, v in verdicts:
            verdict = ("IDENTICAL" if v and all(x.endswith(":IDENTICAL") for x in v) else "DIFFERS" if any(":DIFFERS" in x for x in v)
                       else "MISSING" if any("MISSING" in x for x in v) else "UNCITED")
            tsv.append(f"{s_}\t{rel}\t{verdict}\t{' '.join(v)}")
        md = ["# Production parity, IEEE 1800-2017 vs 1800-2023, for the sv_2023 lane's inherited `must_reject` rulings (SV-CORPUS-GRAD.14b.3)", "",
              "> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/production_parity.py --write`; `--check` re-derives and diffs.", "",
              "A 2017-lane `must_reject` whose basis names a 1800-2017 production carries over to the 2023 lane mechanically when every cited production reads IDENTICALLY in 1800-2023 Annex A (footnote digits, PDF page furniture and spacing normalised); a DIFFERS or MISSING production is read by hand (`SV2023_REREAD_BY_HAND` in the adjudicator).", "",
              "## Instrument identity", "", "> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.", "",
              "| input | repo-root-relative path | sha256 |", "|---|---|---|",
              f"| sv_2017 manifest | `stimuli/sv/characterization/adjudication_manifest.tsv` | `{sha(MANIFEST)}` |",
              f"| IEEE 1800-2017 Annex A | `{A17.relative_to(ROOT)}` | `{sha(A17)}` |",
              f"| IEEE 1800-2023 Annex A | `{A23.relative_to(ROOT)}` | `{sha(A23)}` |",
              f"| instrument | `{Path(__file__).resolve().relative_to(ROOT)}` | `{sha(__file__)}` |", "",
              "## The census", "",
              f"`PRODUCTION-PARITY: rows={n_rows} all_cited_identical={n_all_ident} some_differs={n_diff} some_missing={n_missing} no_production_named={n_nocite} productions={len(per_name)}`", "",
              "| production | verdict |", "|---|---|"] + [f"| `{n}` | {v} |" for n, v in sorted(per_name.items()) if v != "IDENTICAL"] + ["",
              "| suite | relpath | verdict | productions |", "|---|---|---|---|"] + [f"| `{l.split(chr(9))[0]}` | `{l.split(chr(9))[1]}` | {l.split(chr(9))[2]} | `{l.split(chr(9))[3]}` |" for l in tsv[1:]] + [""]
        out_tsv, out_md = Path(__file__).resolve().parent / "parity.tsv", Path(__file__).resolve().parent / "parity.md"
        new_tsv, new_md = "\n".join(tsv) + "\n", "\n".join(md)
        if "--check" in sys.argv:
            cur = (out_tsv.read_text() if out_tsv.is_file() else "", out_md.read_text() if out_md.is_file() else "")
            if cur != (new_tsv, new_md):
                print("production-parity: STALE — parity.tsv / parity.md differ from a fresh derivation; re-run with --write", file=sys.stderr); return 1
            print("production-parity: current"); return 0
        out_tsv.write_text(new_tsv); out_md.write_text(new_md); print(f"wrote {out_tsv.relative_to(ROOT)} and {out_md.relative_to(ROOT)}")
    if "--show" in sys.argv:
        for name, v in per_name.items():
            if v == "DIFFERS":
                a, b = p17[name].split(" "), p23[name].split(" ")
                i = 0
                while i < min(len(a), len(b)) and a[i] == b[i]: i += 1
                print(f"\n== {name} (first difference at token {i} of {len(a)}/{len(b)})\n 2017: …{' '.join(a[max(0, i - 6):i + 14])}\n 2023: …{' '.join(b[max(0, i - 6):i + 14])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
