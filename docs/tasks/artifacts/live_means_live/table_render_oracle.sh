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
# ⛔ IT EARNED ITS KEEP IMMEDIATELY. Run against `docs/TASK_TREE.md` it settled three things the
#   census could not:
#     1. **15 of 67** rows lose their `File` link in the rendered HTML — both renderers agreeing
#        row-for-row. The census counts **14 over-wide + 3 under-wide = 17** arity-defective rows, so
#        the census is a SOUND but not EXACT predictor: **0 false negatives, 2 false positives**
#        (`RGX-0078`, `DOCTRINE-GAP-OWNERSHIP` are over-wide yet still render their link, because
#        their surplus cell is trailing). Zero false negatives is the safe direction for a gate.
#     2. `\|` inside a code span in a table cell renders as a LITERAL pipe and does NOT split the
#        row — in BOTH renderers. That is what makes the repair `|` -> `\|` and nothing else.
#     3. ⚠️ `\\|` is NOT portable. pulldown-cmark renders it as a literal `\|` and does NOT split;
#        it is the one sequence where a re-implementation and a renderer can disagree. `SV-0066` is
#        the only row carrying it, it renders at the declared 13 cells, and it is therefore NOT
#        defective — a row a naive fixer would have "repaired" into a real defect.
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
import os, re
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
        res[name]=(len(tds), tds[-1])
    print(f"  {label}: header cols={len(ths)}  body rows={len(res)}  cells/row={sorted({c for c,_ in res.values()})}")
    return res

print(f"== rendered arity oracle: {src_file} (table {idx}) ==")
a=analyse(f"{W}/book/t.html", "mdbook (pulldown-cmark)")
b=analyse(f"{W}/pandoc.html",  "pandoc (independent)   ")
if a is None:
    print("  NOT EVALUATED — no such table in the render"); raise SystemExit(2)
if b is not None:
    only=[k for k in a if (a[k][0]!=b.get(k,(None,))[0])]
    print(f"  renderers agree on cells/row for {len(a)-len(only)} of {len(a)} rows" + (f"; DISAGREE on {only}" if only else ""))
# A row whose LAST rendered cell no longer looks like the header's last column is losing content.
lost=[k for k,(c,last) in a.items() if 'tasks/' not in last and '](' not in last]
print(f"  rows whose last rendered cell is NOT a link (content lost): {len(lost)}")
for k in sorted(lost): print(f"     {k}")
PY
