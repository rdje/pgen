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

⛔ THE DEFAULTS POINT AT THE LIVE ARTIFACTS, AND THE INPUT IS RECONCILED BEFORE USE
(``SV-CORPUS-GRAD.3.21``). Until that leaf the three defaults were the RETIRED ``_v2``
artifacts, so the obvious no-argument invocation (a) classified a 2026-07-23 cluster table
and (b) wrote the ``_v2`` outputs, leaving the LIVE ``rejects_valid_families.{tsv,md}`` —
the artifact the burn-down actually reads to pick its next leaf — untouched and stale.
Its sibling ``cluster_rejects_valid.py`` already defaulted to the live paths, so the two
halves of one pipeline disagreed about which vintage was current: the worst possible
arrangement, because each is individually self-consistent. It surfaced only because a
regenerated families report said **304 rows** while clusters cut minutes earlier said
**296**, and both numbers happened to be on screen together.

So the row count is no longer something a reader has to notice. This script now REFUSES
unless the cluster table's row count equals the live adjudication manifest's current
``divergence:unexplained_rejects_valid`` count — two independently produced numbers that
must agree ([[feedback_instrument_needs_ground_truth]]) — and REFUSES on any malformed
input line instead of silently skipping it (the silent skip is what hid a TSV corruption:
see ``tsv_cell`` in the clusterer).

Usage:
  python3 stimuli/sv/classify_rejects_valid_families.py \
      [--clusters docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.tsv] \
      [--out-tsv docs/tasks/artifacts/sv_corpus_grad/rejects_valid_families.tsv] \
      [--out-md  docs/tasks/artifacts/sv_corpus_grad/rejects_valid_families.md] \
      [--manifest stimuli/sv/characterization/adjudication_manifest.tsv | --expect-rows N]
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


_REJECTS_VALID = "divergence:unexplained_rejects_valid"


def manifest_rejects_valid_count(manifest: Path) -> int:
    """The live population size, read from the adjudicator's own output."""
    n = 0
    with manifest.open(encoding="utf-8", errors="replace") as fh:
        next(fh, "")  # header
        for line in fh:
            cols = line.rstrip("\n").split("\t")
            if len(cols) >= 5 and cols[4] == _REJECTS_VALID:
                n += 1
    return n


def ground_truth_controls() -> None:
    """Prove both moving parts still work before any number is published.

    The reconciliation below is only as good as the two predicates it rests on: that
    `classify` actually discriminates, and that a malformed line is actually detected.
    A bucketer that silently returned one family for everything, or a parser that
    silently accepted a 2-column line, would both produce a confidently wrong report.
    """
    if classify("assert property (@(posedge clk) a |-> b);") == _OTHER:
        raise SystemExit("REFUSE (control): the POSITIVE control failed — a textbook SVA "
                         "implication line classified as OTHER, so the bucketer is not "
                         "discriminating and every family count would be meaningless.")
    if classify("wombat frobnicate zorp") != _OTHER:
        raise SystemExit("REFUSE (control): the NEGATIVE control failed — a line matching no "
                         "SystemVerilog construct was given a family, so the classifiers are "
                         "over-matching.")
    if len("a\tb".split("\t", 5)) >= 6:
        raise SystemExit("REFUSE (control): the malformed-line detector is not live.")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    root = Path(__file__).resolve().parent.parent.parent
    base = root / "docs/tasks/artifacts/sv_corpus_grad"
    # ⛔ LIVE artifacts, matching the sibling clusterer. See the module docstring: these
    # defaulted to the retired `_v2` vintage until SV-CORPUS-GRAD.3.21.
    ap.add_argument("--clusters", type=Path,
                    default=base / "rejects_valid_clusters.tsv")
    ap.add_argument("--out-tsv", type=Path,
                    default=base / "rejects_valid_families.tsv")
    ap.add_argument("--out-md", type=Path,
                    default=base / "rejects_valid_families.md")
    ap.add_argument("--manifest", type=Path,
                    default=root / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    help="live adjudication manifest; its unexplained_rejects_valid count "
                         "must equal the cluster table's row count")
    ap.add_argument("--expect-rows", type=int, default=None,
                    help="expected row count, for a deliberate off-manifest run (e.g. a "
                         "second lane or a historical table); overrides --manifest")
    args = ap.parse_args()

    ground_truth_controls()

    rows = []
    lines = args.clusters.read_text(encoding="utf-8").splitlines()
    for lineno, line in enumerate(lines[1:], start=2):  # skip header
        if not line.strip():
            continue
        cols = line.split("\t", 5)  # stuck_line is the last field, so a tab in it is safe
        if len(cols) < 6:
            # ⛔ REFUSE, never skip. The old `continue` here is what hid a TSV corruption:
            # the clusterer wrote a multi-line probe error into `stuck_line`, one row became
            # four physical lines, and the three orphan fragments were dropped in silence
            # while the totals still looked right (SV-CORPUS-GRAD.3.21).
            raise SystemExit(
                f"REFUSE: {args.clusters} line {lineno} has {len(cols)} column(s), expected "
                f"6 — the cluster table is malformed, not merely unfamiliar. Re-cut it with "
                f"stimuli/sv/cluster_rejects_valid.py (whose `tsv_cell` collapses embedded "
                f"newlines/tabs) rather than classifying a partial population.\n"
                f"  offending line: {line[:160]!r}")
        suite, rel, _surf, _furth, sig, stuck = cols
        rows.append((suite, rel, sig, stuck, classify(stuck)))

    # ⛔ Reconcile against a number this script did not produce.
    expected = args.expect_rows
    source = "--expect-rows"
    if expected is None:
        if not args.manifest.is_file():
            raise SystemExit(
                f"REFUSE: no live manifest at {args.manifest} and no --expect-rows given, so "
                f"the input's vintage cannot be checked. Pass one or the other; an unchecked "
                f"input is how a July cluster table got classified as current.")
        expected = manifest_rejects_valid_count(args.manifest)
        source = f"{args.manifest} ({_REJECTS_VALID})"
    if len(rows) != expected:
        raise SystemExit(
            f"REFUSE: {args.clusters} carries {len(rows)} rows but {source} says {expected}. "
            f"The cluster table is a different vintage from the manifest — re-cut it with "
            f"stimuli/sv/cluster_rejects_valid.py before classifying, or pass --expect-rows "
            f"{len(rows)} if this off-manifest run is deliberate. (Classifying a stale table "
            f"mis-aims the whole burn-down: it is the PICK step's input.)")
    print(f"reconciled: {len(rows)} rows == {source}")

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
