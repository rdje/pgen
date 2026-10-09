#!/usr/bin/env python3
"""SV-CORPUS-GRAD.14b.3 — PRODUCTION PARITY between IEEE 1800-2017 and 1800-2023 Annex A, for the
`must_reject` rows the sv_2023 lane INHERITS from the 2017 key on a cited 2017 production.

A 2017-lane `must_reject` whose basis says "1800-2017 Annex A <production> derives nothing for this
text" carries over to the 2023 lane ONLY if the 2023 production reads the same. This instrument
extracts every production name a row's basis cites (`name ::=`), pulls both editions' bodies out of
the in-repo LRM texts, normalises them (footnote digits, whitespace, code fences) and reports
IDENTICAL or DIFFERS per production — the mechanical half of `.14b.3`; a DIFFERS is read by hand.

Usage: python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/production_parity.py [--self-test]
       … --write    re-derive parity.tsv / parity.md
       … --check    (1) the two artifacts equal a fresh derivation AND (2) the sv_2023 manifest AGREES with it —
                    every IDENTICAL row carries over, no `re-read is owed` row is IDENTICAL (SV-CORPUS-GRAD.54;
                    run by CORPUS-KEY-INTEGRITY tier A5 on every commit). rc 0 both hold, 1 either fails, 2 refused.
       PGEN_PARITY_MANIFEST / PGEN_PARITY_SV2023_MANIFEST  override the two manifests (the doctrine's red control
                    replays the pair that lagged at PGEN-SV-CORPUS-GRAD-0439 from git history)
"""
from __future__ import annotations
import csv, os, re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[5]
A17 = ROOT / "docs/systemverilog/2017/md/section-41-data-read-api.md"
A23 = ROOT / "docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md"
MANIFEST = Path(os.environ.get("PGEN_PARITY_MANIFEST") or ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv")
SV2023_MANIFEST = Path(os.environ.get("PGEN_PARITY_SV2023_MANIFEST") or ROOT / "stimuli/sv/characterization/adjudication_manifest_sv2023.tsv")
# SV-CORPUS-GRAD.54: the two markers the adjudicator writes into an sv_2023 basis (adjudicate_external_corpus.py,
# the .14b.3 carry-over): a row this artifact decides IDENTICAL must read CARRY_MARK; a row still reading OWED_MARK
# while IDENTICAL here was adjudicated BEFORE this artifact learnt its citation — the one-pass lag.
CARRY_MARK = "CARRIES OVER"
OWED_MARK = "its 1800-2023 re-read is owed (.14b.3)"
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
    s = re.sub(r"([a-z_\]\)}])\d+\b", r"\1", s)     # footnote digits glued to names or brackets (this46, ]15, }13)
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
                # a line inside A.10 (the footnotes) sits after the last head, `white_space`; it cites a
                # FOOTNOTE, not that production — footnotes are read by hand, never compared as a body
                if n and n != "white_space": names.append(n)
    seen, out = set(), []
    for n in names:
        n = re.sub(r"(?<=[a-z_])\d+$", "", n) if n not in ("delay2", "delay3") else n
        if n not in seen: seen.add(n); out.append(n)
    return out


def manifest_agreement(verdicts, sv2023_rows):
    """SV-CORPUS-GRAD.54 — does the sv_2023 manifest AGREE with this derivation? The adjudicator reads parity.tsv to
    carry a 2017 ruling over, and parity.tsv reads the sv_2017 manifest's citations, so a NEW pin needs two passes
    (adjudicate -> parity -> adjudicate); PGEN-SV-CORPUS-GRAD-0439 stopped after one and published three rows as
    `re-read is owed` beside an artifact saying IDENTICAL. Returns (ok, summary line, detail lines): every row
    decided IDENTICAL must exist in the sv_2023 manifest and carry CARRY_MARK in its basis."""
    basis = {(r["suite"], r["relpath"]): r["basis"] for r in sv2023_rows}
    identical = [(s_, rel) for s_, rel, v in verdicts if v and all(x.endswith(":IDENTICAL") for x in v)]
    missing = [k for k in identical if k not in basis]
    not_carried = [k for k in identical if k in basis and CARRY_MARK not in basis[k]]
    owed = [k for k in not_carried if OWED_MARK in basis[k]]
    ok = not missing and not not_carried
    summary = (f"PARITY-MANIFEST-AGREEMENT: identical={len(identical)} carried_over={len(identical) - len(not_carried) - len(missing)} "
               f"owed_but_identical={len(owed)} not_carried={len(not_carried)} missing_in_sv2023={len(missing)} {'OK' if ok else 'STALE'}")
    details = [f"  STALE {s_} {rel}: parity says IDENTICAL, the sv_2023 row reads {'re-read is owed' if (s_, rel) in owed else 'neither marker'} "
               "- run the adjudicator again (the fixed point is adjudicate -> parity --write -> adjudicate)" for s_, rel in not_carried]
    details += [f"  STALE {s_} {rel}: parity decides it, the sv_2023 manifest has no such row - the manifests are out of step" for s_, rel in missing]
    return ok, summary, details


def main() -> int:
    if "--self-test" in sys.argv:
        p, heads = productions("x ::=\n```\n\na | b3\n```ebnf\nAuthorized licensed use limited to: nobody\ny22 ::= c ( d )\n")
        ok = (p["x"] == "a | b" and p["y22"] == p["y"] == "c ( d )" and heads == [(1, "x"), (7, "y22")]
              and cited_names("IEEE foo ::= a | b (x:1); bar ::= c") == ["foo", "bar"]
              and enclosing(heads, 3) == "x" and enclosing(heads, 8) == "y"
              and normalise("@@( e )15") == normalise("@@ ( e )")
              and productions("z ::= q\n1176\nA.8.8 Strings\n| r\n")[0]["z"] == "q | r"
              and normalise("{ packed_dimension }13") == normalise("{ packed_dimension }17"))
        # SV-CORPUS-GRAD.54: the agreement arm on literals — GREEN when every IDENTICAL row carries over; RED on an
        # owed row, on a row with neither marker, on a row the sv_2023 manifest lacks; a DIFFERS row is never asked.
        vs = [("s", "a", ["p:IDENTICAL"]), ("s", "b", ["q:DIFFERS"]), ("s", "c", ["r:IDENTICAL"]), ("s", "d", ["t:IDENTICAL"]), ("s", "e", ["u:IDENTICAL", "v:IDENTICAL"])]
        rows = [{"suite": "s", "relpath": "a", "basis": "sv_2023 lane (.14b.3): the 2017 ruling CARRIES OVER mechanically"},
                {"suite": "s", "relpath": "b", "basis": "sv_2023 lane (.14b.2): INHERITED ... " + OWED_MARK},
                {"suite": "s", "relpath": "c", "basis": "sv_2023 lane (.14b.2): INHERITED ... " + OWED_MARK},
                {"suite": "s", "relpath": "e", "basis": "the lane's own ruling, neither marker"}]
        ok1, line1, det1 = manifest_agreement(vs, rows)
        rows_green = rows[:2] + [{"suite": "s", "relpath": "c", "basis": "CARRIES OVER, re-read by hand"}, {"suite": "s", "relpath": "d", "basis": "CARRIES OVER mechanically"}, {"suite": "s", "relpath": "e", "basis": "CARRIES OVER mechanically"}]
        ok2, line2, det2 = manifest_agreement(vs, rows_green)
        ok = ok and (not ok1 and line1 == "PARITY-MANIFEST-AGREEMENT: identical=4 carried_over=1 owed_but_identical=1 not_carried=2 missing_in_sv2023=1 STALE"
                     and len(det1) == 3 and any("s c: parity says IDENTICAL, the sv_2023 row reads re-read is owed" in d for d in det1)
                     and any("s e: parity says IDENTICAL, the sv_2023 row reads neither marker" in d for d in det1) and any("s d: parity decides it" in d for d in det1)
                     and ok2 and line2 == "PARITY-MANIFEST-AGREEMENT: identical=4 carried_over=4 owed_but_identical=0 not_carried=0 missing_in_sv2023=0 OK" and not det2)
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
              "| suite | relpath | verdict | productions |", "|---|---|---|---|"] + [f"| `{l.split(chr(9))[0]}` | `{l.split(chr(9))[1]}` | {l.split(chr(9))[2]} | {md_code(l.split(chr(9))[3])} |" for l in tsv[1:]] + [""]
        out_tsv, out_md = Path(__file__).resolve().parent / "parity.tsv", Path(__file__).resolve().parent / "parity.md"
        new_tsv, new_md = "\n".join(tsv) + "\n", "\n".join(md)
        if "--check" in sys.argv:
            cur = (out_tsv.read_text() if out_tsv.is_file() else "", out_md.read_text() if out_md.is_file() else "")
            stale = cur != (new_tsv, new_md)
            if stale:
                print("production-parity: STALE — parity.tsv / parity.md differ from a fresh derivation; re-run with --write", file=sys.stderr)
            else:
                print("production-parity: current")
            # SV-CORPUS-GRAD.54: the manifest-side arm — evaluated whether or not the artifact is current, so a stale
            # artifact and a lagging manifest are both named in one run
            if not SV2023_MANIFEST.is_file():
                print(f"production-parity: REFUSED - no sv_2023 manifest at {SV2023_MANIFEST}", file=sys.stderr); return 2
            ok, line, details = manifest_agreement(verdicts, list(csv.DictReader(SV2023_MANIFEST.open(encoding="utf-8"), delimiter="\t")))
            print(line)
            for d in details:
                print(d, file=sys.stderr)
            return 1 if (stale or not ok) else 0
        out_tsv.write_text(new_tsv); out_md.write_text(new_md); print(f"wrote {out_tsv.relative_to(ROOT)} and {out_md.relative_to(ROOT)}")
    if "--show" in sys.argv:
        for name, v in per_name.items():
            if v == "DIFFERS":
                a, b = p17[name].split(" "), p23[name].split(" ")
                i = 0
                while i < min(len(a), len(b)) and a[i] == b[i]: i += 1
                print(f"\n== {name} (first difference at token {i} of {len(a)}/{len(b)})\n 2017: …{' '.join(a[max(0, i - 6):i + 14])}\n 2023: …{' '.join(b[max(0, i - 6):i + 14])}")
    return 0


def md_code(s: str) -> str:
    """A GFM code span that RENDERS for any content (TABLE-ARITY-RATCHET, SV-CORPUS-GRAD.3.37):
    an empty value is a dash — a bare `` `` `` is an unpaired run that eats the next cell — and a
    value that itself contains backticks (a stuck token that IS a directive tick, `; ` ID`) takes a
    delimiter run one longer than its longest internal run, padded so a leading or trailing
    backtick cannot fuse with the delimiter."""
    if not s:
        return "—"
    longest = max((len(m) for m in __import__("re").findall(r"`+", s)), default=0)
    fence = "`" * (longest + 1)
    pad = " " if (s.startswith("`") or s.endswith("`") or longest) else ""
    return f"{fence}{pad}{s}{pad}{fence}"


if __name__ == "__main__":
    sys.exit(main())
