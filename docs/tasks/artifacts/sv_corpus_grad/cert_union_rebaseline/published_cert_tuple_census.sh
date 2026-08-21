#!/usr/bin/env bash
# published_cert_tuple_census.sh — SV-CORPUS-GRAD.13c.2x(c) / .13c.2x.6
#
# SIZES THE POPULATION `.13c.2x.6` LEFT AS AN HONEST BOUND: *"this leaf measured ONE published
# tuple … parser-families.md / stimuli-and-quality.md publish other families' certificate tuples
# that were NOT checked here. Sizing that population is part of (c)."*
#
# ⛔ IT IS AN INSTRUMENT, NOT A GATE, AND THAT IS DELIBERATE. The book is a narrative surface: it
# quotes historical tuples on purpose (`total 1304→1324`, "the then-current headline was …"). A
# checker that failed on every cert tuple in prose would be `.13c.2x.4` again — a doctrine whose
# adoption cost blocks every commit. So this REPORTS the split and routes it; only tuples inside a
# watched marker block are held equal to a producer.
#
# The one classification rule: a published tuple is WATCHED iff it sits inside a `<!-- NAME:BEGIN -->`
# / `<!-- NAME:END -->` pair that `scripts/check_published_version_currency.sh` names.
#
# Usage: bash docs/tasks/artifacts/sv_corpus_grad/cert_union_rebaseline/published_cert_tuple_census.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

python3 - "$ROOT" <<'PY'
import os, re, sys

root = sys.argv[1]
book = os.path.join(root, "docs", "book", "src")
checker = open(os.path.join(root, "scripts", "check_published_version_currency.sh"),
               encoding="utf-8").read()

# A cert tuple, as the report itself prints it: a `total=` / `witness=` / `UNKNOWN=` headline, or a
# `fully_certified` verdict. Deliberately broad — over-collecting is safe for a census, the point is
# a FLOOR that cannot silently miss a surface.
TUPLE = re.compile(r"(CERTIFICATE-COVERAGE|total\s*=\s*\d+|UNKNOWN\s*=\s*\d+|fully_certified)")
MARK  = re.compile(r"<!--\s*([A-Z0-9-]+):(BEGIN|END)")

watched_names, loose, watched = set(), [], []

for name in sorted(os.listdir(book)):
    if not name.endswith(".md"):
        continue
    path = os.path.join(book, name)
    open_block = None
    for lineno, line in enumerate(open(path, encoding="utf-8"), 1):
        m = MARK.search(line)
        if m:
            open_block = m.group(1) if m.group(2) == "BEGIN" else None
            if m.group(2) == "BEGIN":
                watched_names.add(m.group(1))
        if TUPLE.search(line):
            row = (name, lineno, open_block)
            (watched if open_block else loose).append(row)

# A marker block only counts as WATCHED if the enforcer actually names it — a block nobody reads is
# the SV-CORPUS-GRAD.13i defect, and this census must not launder one into a pass.
unread = sorted(n for n in watched_names if n not in checker)
really_watched = [r for r in watched if r[2] in checker]
watched_but_unread = [r for r in watched if r[2] not in checker]

# ⚠️ THE UNIT IS A LINE, NOT A NUMBER, AND SAYING SO IS PART OF THE MEASUREMENT. This counts LINES
# carrying a certificate-tuple token, which is a FLOOR: a watched block whose rows name contract
# keys (`expected_total`) rather than report syntax (`total=1385`) contributes fewer matched lines
# than it publishes rows. The enforcer reports the row count for the block it holds; this reports
# the exposure. Conflating the two would flatter whichever number is larger.
print("PUBLISHED-CERT-TUPLE-CENSUS: tuple_lines_watched=%d tuple_lines_loose=%d in_unread_block=%d "
      "marker_blocks=%d unread_blocks=%d"
      % (len(really_watched), len(loose), len(watched_but_unread), len(watched_names), len(unread)))
print("")
print("marker blocks found : %s" % (", ".join(sorted(watched_names)) or "<none>"))
print("named by the enforcer: %s" % (", ".join(sorted(n for n in watched_names if n in checker)) or "<none>"))
print("NOT named (unread)   : %s" % (", ".join(unread) or "<none>"))
print("")
print("WATCHED tuples (inside an enforcer-named block):")
for f, l, b in really_watched:
    print("  %s:%s  [%s]" % (f, l, b))
print("")
print("LOOSE tuples (published prose, held by nothing) — by page:")
per = {}
for f, l, _ in loose:
    per.setdefault(f, []).append(l)
for f in sorted(per):
    lines = per[f]
    print("  %-36s %3d  (lines %s%s)" % (f, len(lines), ", ".join(str(x) for x in lines[:8]),
                                         ", …" if len(lines) > 8 else ""))
PY
