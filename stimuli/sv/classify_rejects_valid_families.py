#!/usr/bin/env python3
"""Classify the rejects-valid population into construct families (SV-CORPUS-GRAD.3.2).

Read-only diagnosis pass. Consumes the per-row stuck-point TSV emitted by
``cluster_rejects_valid.py`` (columns: suite, relpath, surface_pos,
furthest_pos, signature, stuck_line) and buckets every
``divergence:unexplained_rejects_valid`` row into a small, well-defined set of
SystemVerilog construct families using priority-ordered regex classifiers over
the stuck source line. The output is the *leaf-cutting map* the ``.3`` burn-down
leaves are cut from — the ADD-v1-era refresh of the ``.3.0`` hand-merged family
table (which was built over the pre-ADD-v1 315-row population).

The classifier is a coarse structural bucketer, not an adjudicator: it names the
dominant construct at the stuck point so families can be ranked by cross-suite
yield. Per-row precision belongs to the owning fix leaf's TOOLBOX diagnosis.

Deterministic and stdlib-only: same input TSV -> byte-identical output.

Usage:
  python3 stimuli/sv/classify_rejects_valid_families.py \
      [--clusters docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters_v2.tsv] \
      [--out-tsv docs/tasks/artifacts/sv_corpus_grad/rejects_valid_families_v2.tsv] \
      [--out-md  docs/tasks/artifacts/sv_corpus_grad/rejects_valid_families_v2.md]
"""

import argparse
import re
from collections import Counter, defaultdict
from pathlib import Path

# Priority-ordered (first match wins). Each entry: (family, compiled regex over
# the stuck source line). LRM chapter tags name the governing 1800-2017 clause.
_FAMILY_RULES = [
    ("SVA implication/property (ch16)",
     re.compile(r"\|->|\|=>|##|\[\*|\[->|\[=|disable\s+iff|\bproperty\b|"
                r"\bsequence\b|first_match|throughout|intersect\b|s_until|"
                r"nexttime|\baccept_on\b|\breject_on\b")),
    ("coverage bins/cross (ch19)",
     re.compile(r"\bbins\b|binsof|\bcross\b|coverpoint|covergroup")),
    ("constraint/randomize (ch18)",
     re.compile(r"\bdist\b|with\s*\{|\bconstraint\b|randomize|\bsolve\b|"
                r"\bsoft\b")),
    ("interface/modport (ch25)",
     re.compile(r"\bmodport\b|\.\w+\s*\(")),
    ("compiler directives (ch22)",
     re.compile(r"`__FILE__|`__LINE__|`begin_keywords|`pragma|`line|"
                r"`resetall|`default_nettype|`__")),
    ("number literal spaced-based (ch5)",
     re.compile(r"\d+\s*'\s*[hdbospHDBOS]\s|'\s*[hdbo]\s+[0-9a-fA-FxXzZ_]")),
    ("size/type cast N'(...) (ch6/11)",
     re.compile(r"\)\s*'\s*\(|\w'\s*\(|\d+'\s*\(")),
    ("enum base range (ch6)",
     re.compile(r"\benum\s*\[")),
    ("unique0 (ch12)",
     re.compile(r"\bunique0\b")),
    ("named block/label (ch9/27)",
     re.compile(r"\bbegin\s*:")),
    ("drive/charge strength (ch28)",
     re.compile(r"\b(weak0|weak1|strong0|strong1|pull0|pull1|supply0|"
                r"supply1|highz0|highz1)\b")),
    ("foreach/array (ch7)",
     re.compile(r"\bforeach\b")),
]
_OTHER = "OTHER (per-row triage)"


def classify(stuck_line: str) -> str:
    for family, rx in _FAMILY_RULES:
        if rx.search(stuck_line):
            return family
    return _OTHER


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    root = Path(__file__).resolve().parent.parent.parent
    base = root / "docs/tasks/artifacts/sv_corpus_grad"
    ap.add_argument("--clusters", type=Path,
                    default=base / "rejects_valid_clusters_v2.tsv")
    ap.add_argument("--out-tsv", type=Path,
                    default=base / "rejects_valid_families_v2.tsv")
    ap.add_argument("--out-md", type=Path,
                    default=base / "rejects_valid_families_v2.md")
    args = ap.parse_args()

    rows = []
    lines = args.clusters.read_text(encoding="utf-8").splitlines()
    for line in lines[1:]:  # skip header
        cols = line.split("\t", 5)  # stuck_line may contain tabs
        if len(cols) < 6:
            continue
        suite, rel, _surf, _furth, sig, stuck = cols
        rows.append((suite, rel, sig, stuck, classify(stuck)))

    fam_total = Counter()
    fam_by_suite = defaultdict(Counter)
    fam_examples = defaultdict(list)
    for suite, rel, _sig, stuck, fam in rows:
        fam_total[fam] += 1
        fam_by_suite[fam][suite] += 1
        if len(fam_examples[fam]) < 4:
            fam_examples[fam].append((suite, rel, stuck[:80]))

    # Stable ordering: count desc, then family name asc (deterministic ties).
    ordered = sorted(fam_total.items(), key=lambda kv: (-kv[1], kv[0]))

    args.out_tsv.parent.mkdir(parents=True, exist_ok=True)
    with args.out_tsv.open("w", encoding="utf-8") as fh:
        fh.write("family\tsuite\trelpath\tsignature\tstuck_line\n")
        for suite, rel, sig, stuck, fam in sorted(rows):
            fh.write(f"{fam}\t{suite}\t{rel}\t{sig}\t{stuck}\n")

    total = len(rows)
    md = [
        "# rejects-valid construct families (SV-CORPUS-GRAD.3.2)",
        "",
        f"{total} `divergence:unexplained_rejects_valid` rows classified into "
        f"{len(ordered)} construct families (priority-ordered structural "
        "bucketer over the stuck source line; the leaf-cutting map for the "
        "`.3` burn-down). Families ranked by cross-suite row count.",
        "",
        "| # | family | rows | suite split |",
        "|---|---|---|---|",
    ]
    for i, (fam, n) in enumerate(ordered, 1):
        split = ", ".join(f"{s}:{c}" for s, c in
                          sorted(fam_by_suite[fam].items(),
                                 key=lambda kv: (-kv[1], kv[0])))
        md.append(f"| {i} | {fam} | {n} | {split} |")
    md.append("")
    md.append("## Representative stuck lines per family")
    for fam, _ in ordered:
        md.append("")
        md.append(f"### {fam}")
        for suite, rel, stuck in fam_examples[fam]:
            md.append(f"- `{suite}`: `{stuck}`  ({rel})")
    md.append("")
    args.out_md.write_text("\n".join(md), encoding="utf-8")

    print(f"classified {total} rows into {len(ordered)} families")
    for fam, n in ordered:
        print(f"  {n:4d}  {fam}")


if __name__ == "__main__":
    main()
