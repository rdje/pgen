#!/usr/bin/env bash
# table_render_oracle.sh — answer the arity question with the RENDERER, not with a re-implementation.
# Owning leaf: `LIVE-DOC-CONTAINMENT.8`.
#
# ⭐⭐ WHY AN ORACLE AND NOT ANOTHER COUNTER. `table_arity_census.sh` re-implements GFM's cell
#   splitting, so it can only ever prove that MY MODEL of the rule is violated. The question that
#   matters is what a READER SEES, and only a renderer answers that. This drives BOTH renderers
#   available here — `mdbook` (pulldown-cmark, the one this project actually publishes through) and
#   `pandoc` (an independent implementation) — and reports the cells they really emit.
#
# ⛔ IT EARNED ITS KEEP IMMEDIATELY, AND THEN IT CAUGHT THE CENSUS. What it has settled:
#     1. `\|` inside a code span in a table cell renders as a LITERAL pipe and does NOT split the
#        row — in BOTH renderers. That is what makes the repair `|` -> `\|` and nothing else.
#     2. ⚠️ `\\|` is NOT portable. pulldown-cmark renders it as a literal `\|` and does NOT split;
#        it is the one sequence where a re-implementation and a renderer can disagree. `SV-0066` is
#        the only row carrying it, it renders at the declared 13 cells, and it is therefore NOT
#        defective — a row a naive fixer would have "repaired" into a real defect.
#     3. ⛔⛔ **THE CENSUS HAD A FALSE NEGATIVE, AND ONLY THE RENDERER COULD SHOW IT.** The tail
#        probe below reads the bug ledger at **26** rows losing their final authored cell; the
#        `NF-2` census read **25**, because `REGEX-0098` omits its trailing pipe and GFM makes that
#        pipe OPTIONAL. An earlier revision of this header claimed the census had "0 false
#        negatives" — that was measured on `docs/TASK_TREE.md` alone and did not generalise. It is
#        corrected here rather than left standing: a claim about an instrument is a claim.
#     4. Cell COUNT in the render is useless as a defect signal — GFM truncates every over-wide row
#        to the header width, so healthy and defective rows both report the declared arity. The
#        question a reader cares about is whether the AUTHORED TAIL is still on the page, which is
#        what this tool now asks.
#
# ⚠️ HONEST LIMIT: it reports what THESE two renderers do. GitHub.com runs cmark-gfm, which is not
#   installed here; where the two available implementations agree, a third disagreeing is unlikely
#   but not disproven. Stated rather than implied.
#
# USAGE: docs/tasks/artifacts/live_means_live/table_render_oracle.sh <file.md> [table-index]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT" || exit 1

FILE="${1:-docs/TASK_TREE.md}"
IDX="${2:-0}"
W="rust/target/table_render_oracle"

[ -r "$FILE" ] || { echo "render-oracle: cannot read $FILE" >&2; exit 2; }
command -v mdbook >/dev/null 2>&1 || { echo "render-oracle: NOT EVALUATED — mdbook is absent, so no renderer can be driven" >&2; exit 2; }

rm -rf "$W"; mkdir -p "$W/src"
printf '[book]\ntitle = "render oracle"\nsrc = "src"\n' > "$W/book.toml"
printf '# S\n\n- [t](./t.md)\n' > "$W/src/SUMMARY.md"
cp "$FILE" "$W/src/t.md"
mdbook build "$W" >"$W/build.log" 2>&1 || { echo "render-oracle: mdbook failed, see $W/build.log" >&2; exit 2; }
if command -v pandoc >/dev/null 2>&1; then
  pandoc -f gfm -t html "$FILE" -o "$W/pandoc.html" 2>"$W/pandoc.log" || true
fi

FILE="$FILE" IDX="$IDX" W="$W" python3 - <<'PY'
import os, re, html
W=os.environ['W']; idx=int(os.environ['IDX']); src_file=os.environ['FILE']

def analyse(path,label):
    if not os.path.exists(path): return None
    s=open(path,encoding='utf-8').read()
    tables=re.findall(r'<table>.*?</table>', s, re.S)
    if idx >= len(tables): return None
    t=tables[idx]
    head=re.search(r'<thead>.*?</thead>', t, re.S).group(0)
    # ⛔ `<th[^>]*>` also matches `<thead>`; require whitespace or an immediate close.
    ths=re.findall(r'<th(?:\s[^>]*)?>(.*?)</th>', head, re.S)
    rows=re.findall(r'<tr[^>]*>(.*?)</tr>', t[t.index('</thead>'):], re.S)
    res={}
    for r in rows:
        tds=re.findall(r'<td(?:\s[^>]*)?>(.*?)</td>', r, re.S)
        if not tds: continue
        name=re.sub(r'<[^>]+>','',tds[0]).strip().strip('` ')
        res[name]=(len(tds), tds[-1], html.unescape(re.sub(r'<[^>]+>', ' ', ' '.join(tds))))
    print(f"  {label}: header cols={len(ths)}  body rows={len(res)}  cells/row={sorted({c for c,_,_ in res.values()})}")
    return res


def source_rows(path, want):
    """The table's rows AS WRITTEN, split the way GFM splits them (escaped `\\|` is not a break)."""
    sep = re.compile(r'^\| *:?-+:? *\|')
    out, n, inside, prev = {}, 0, False, False
    for line in open(path, encoding='utf-8').read().split('\n'):
        isp = line.startswith('|')
        if isp and sep.match(line):
            prev = True
            continue
        if isp and not prev:
            n += 1
            inside = (n == want + 1)
        elif isp and inside:
            cells = [c.replace('\x01', '\\|') for c in line.replace('\\|', '\x01').split('|')]
            if cells and cells[0].strip() == '': cells = cells[1:]
            if cells and cells[-1].strip() == '': cells = cells[:-1]
            if cells: out[cells[0].strip().strip('` ')] = cells
        prev = isp
    return out


def key(x):
    """Alphanumeric-only comparison key.

    ⛔ mdbook applies SMART PUNCTUATION (`'`->U+2019, `"`->U+201C/D), so any probe that compares
    punctuation reports the RENDERER'S TYPOGRAPHY as content loss. Measured: comparing raw text
    false-positived 11 healthy ledger rows (`SV-0015`..`SV-0020`, `REGEX-0078`, `RTL-FE-0002`,
    `SV-0053`, `SV-0064`) before this normalisation, at 36 reported against a true 26.
    """
    return re.sub(r'[^0-9A-Za-z]', '', re.sub(r'\[([^\]]*)\]\([^)]*\)', r'\1', x))

print(f"== rendered arity oracle: {src_file} (table {idx}) ==")
a=analyse(f"{W}/book/t.html", "mdbook (pulldown-cmark)")
b=analyse(f"{W}/pandoc.html",  "pandoc (independent)   ")
if a is None:
    print("  NOT EVALUATED — no such table in the render"); raise SystemExit(2)
if b is not None:
    only=[k for k in a if (a[k][0]!=b.get(k,(None,))[0])]
    print(f"  renderers agree on cells/row for {len(a)-len(only)} of {len(a)} rows" + (f"; DISAGREE on {only}" if only else ""))

# ⭐ THE ACCEPTANCE TEST ITSELF, ASKED OF THE RENDERER: does the row's FINAL AUTHORED CELL survive?
#   Cell COUNT cannot answer it — GFM truncates every over-wide row to the header width, so the
#   render reports the declared arity for healthy and defective rows alike. What separates them is
#   whether the tail the author wrote is still on the page. This is table-shape agnostic and
#   replaces the earlier `docs/TASK_TREE.md`-specific "last cell is a link" heuristic, which could
#   not be pointed at the bug ledger (whose last column is free prose) and which also flagged the
#   three SHORT rows, a different defect.
src = source_rows(src_file, idx)
lost = []
for name, cells in src.items():
    tail = key(cells[-1])[-40:]
    if tail and tail not in key(a.get(name, (0, '', ''))[2]):
        lost.append((name, len(cells), tail))
print(f"  rows whose FINAL AUTHORED CELL is ABSENT from the render (content LOST): {len(lost)} of {len(src)}")
for name, c, tail in sorted(lost):
    print(f"     {name:34s} {c:3d} source cells  missing tail: …{tail}")
if b is not None:
    lb = {n for n, cells in src.items()
          if key(cells[-1])[-40:] and key(cells[-1])[-40:] not in key(b.get(n, (0, '', ''))[2])}
    la = {n for n, _, _ in lost}
    print(f"  the two renderers agree on the lost set: {la == lb}" + ("" if la == lb else f"  mdbook-only={sorted(la-lb)} pandoc-only={sorted(lb-la)}"))


# ⭐ SECOND DEFECT CLASS, SAME ORACLE (`LIVE-DOC-CONTAINMENT.9`): a code span that cannot hold its
#   own content. A single-backtick span CANNOT contain a backtick — the first inner one closes it —
#   and SystemVerilog's compiler directives ARE backtick-initial tokens (`` `timescale ``,
#   `` `include ``), so quoting one inside a single-backtick span breaks and the break CASCADES.
#
# ⛔ THE DETECTOR MUST EXCLUDE <code> CONTENT, AND GETTING THAT WRONG OVERCOUNTS BY 31 %. A backtick
#   rendered INSIDE a <code> element is the author's correctly-escaped content, rendered exactly as
#   intended. The loose test — "any backtick in the rendered text" — reads the bug ledger at 21 cells
#   in 12 rows against a true 16 in 11: `SV-0046` was entirely healthy and `SV-0048` scored 5 cells
#   against a real 1. Measured, and it is why this arm strips <code> before looking.
def backtick_census(path, label):
    if not os.path.exists(path):
        return None
    s = open(path, encoding='utf-8').read()
    tables = re.findall(r'<table>.*?</table>', s, re.S)
    if idx >= len(tables):
        return None
    rows = re.findall(r'<tr[^>]*>(.*?)</tr>', tables[idx], re.S)
    hits, cells = {}, 0
    for r in rows:
        tds = re.findall(r'<td(?:\s[^>]*)?>(.*?)</td>', r, re.S)
        if not tds:
            continue
        nm = re.sub(r'<[^>]+>', '', tds[0]).strip().strip('` ')
        for i, td in enumerate(tds):
            cells += 1
            prose = re.sub(r'<code[^>]*>.*?</code>', ' ', td, flags=re.S)
            if '`' in html.unescape(re.sub(r'<[^>]+>', ' ', prose)):
                hits.setdefault(nm, []).append(i)
    print(f"  {label}: cells whose PROSE shows a backtick (a code span that could not hold it): "
          f"{sum(len(v) for v in hits.values())} of {cells}"
          + (f" in {len(hits)} rows" if hits else ""))
    for k, v in sorted(hits.items()):
        print(f"     {k:14s} cells {v}")
    return hits

ha = backtick_census(f"{W}/book/t.html", "mdbook ")
hb = backtick_census(f"{W}/pandoc.html", "pandoc ")
if ha is not None and hb is not None:
    print(f"  the two renderers agree on the backtick set: {ha == hb}")
PY
