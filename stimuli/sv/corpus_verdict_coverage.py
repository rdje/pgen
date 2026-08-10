#!/usr/bin/env python3
"""How much of the SV corpus actually carries a VERDICT? (SV-CORPUS-GRAD.13)

WHY THIS EXISTS
---------------
The graduation bar is a count of DIVERGENCES.  That number answers "how many defects do
we know about", and it is silent on the question a signoff claim actually rests on:
**what fraction of the corpus was asked a question it could answer at all?**

A row that never got a verdict is not evidence of correctness.  It is not evidence of
anything.  `deferred:chained_only` in particular is the honest thing to do with a
multi-file design when you can only parse one file in isolation -- and it means the most
realistic industry RTL in the corpus contributes NOTHING to the confidence claim.

`SV-CORPUS-GRAD.12` established the pattern for this whole family of question: an
`explained`/`deferred` label is a statement about the INPUT, and until someone checks it,
the burn-down's denominator is a guess.  This instrument makes the denominator explicit
and re-runnable so it cannot rot the way a number pasted into a doc does.

WHAT IT REPORTS
---------------
Every adjudication class, bucketed by whether the row carries a usable verdict:

  ADJUDICATED   the row was asked and answered - a match, or a known divergence
  ROUTED        answered in ANOTHER lane's manifest (verilog_2005), not lost
  NO VERDICT    the row contributes nothing to the confidence claim, for a named reason

⛔ NO VERDICT is not the same as "defect".  It is "unknown", and the whole point is that
unknown and clean are different words.

USAGE
    python3 stimuli/sv/corpus_verdict_coverage.py

Output: docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/coverage.md (+ .tsv)
"""

import argparse
import hashlib
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

# How each adjudication class contributes to a confidence claim, and WHY.
# ⛔ The `why` strings are the load-bearing part: a bucket assignment nobody can argue
# with is a bucket assignment nobody checked.
CLASSES = {
    "match": (
        "ADJUDICATED", "expected and observed agree - the row testifies FOR the parser"),
    "divergence:unexplained_rejects_valid": (
        "ADJUDICATED", "a known defect: valid SV the parser refuses (the axis-2 bar)"),
    "divergence:unexplained_accepts_invalid": (
        "ADJUDICATED", "a known defect: invalid SV the parser accepts (the axis-2 bar)"),
    "divergence:explained_svpp_macro_use": (
        "ADJUDICATED", "parse stops on a macro use - positionally gated since .12a"),
    "divergence:explained_svpp_conditional": (
        "ADJUDICATED", "parse stops on a conditional - positionally gated since .12a"),
    "divergence:explained_svpp_include": (
        "ADJUDICATED", "parse stops on an `include - positionally gated since .12a"),
    "divergence:explained_svpp_protected_envelope": (
        "ADJUDICATED", "IEEE 1800-2017 §34 encrypted envelope - not source text yet"),
    "deferred:v2005_profile_lane": (
        "ROUTED", "adjudicated in adjudication_manifest_v2005.tsv, not lost"),
    "deferred:chained_only": (
        "NO VERDICT",
        "multi-file design: needs `include/`define chaining to parse honestly - the "
        "MOST realistic RTL in the corpus, contributing nothing to the claim"),
    "deferred:no_sv_key": (
        "NO VERDICT", "no upstream answer key exists - expectation underivable"),
    "deferred:svpp_owned": (
        "NO VERDICT", "conformance owned by the preprocessor lane, by design"),
    "deferred:impl_varying": (
        "NO VERDICT", "LRM leaves the behaviour implementation-defined"),
    "deferred:verilog_ams_lane": (
        "NO VERDICT", "Verilog-AMS, a different language family"),
    "deferred:ni_unimplemented": (
        "NO VERDICT", "upstream marks the construct not-implemented"),
}
BUCKETS = ["ADJUDICATED", "ROUTED", "NO VERDICT"]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_manifest(path: Path):
    rows = []
    with path.open(encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        ic, isuite = header.index("adjudication"), header.index("suite")
        for line in fh:
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            rows.append((c[isuite], c[ic]))
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--manifest",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--manifest-v2005",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
                    type=Path)
    ap.add_argument("--audit",
                    default=ROOT / "docs/tasks/artifacts/sv_corpus_grad/"
                                   "explained_svpp_audit/after/audit.tsv",
                    type=Path)
    ap.add_argument("--outdir",
                    default=ROOT / "docs/tasks/artifacts/sv_corpus_grad/verdict_coverage",
                    type=Path)
    args = ap.parse_args()

    rows = read_manifest(args.manifest)
    counts = Counter(cls for _s, cls in rows)
    unknown = sorted(set(counts) - set(CLASSES))
    if unknown:
        raise SystemExit(
            "⛔ REFUSING: adjudication class(es) this census has no bucket for: "
            f"{unknown}. Classify them explicitly - an unclassified class silently "
            "lands in no bucket, which is the failure mode this instrument exists to "
            "prevent.")

    total = sum(counts.values())
    by_bucket = Counter()
    for cls, n in counts.items():
        by_bucket[CLASSES[cls][0]] += n

    # The .12 audit splits the `explained` rows further: corroborated vs undecidable.
    corroborated = undecidable = 0
    if args.audit.is_file():
        with args.audit.open(encoding="utf-8") as fh:
            h = fh.readline().rstrip("\n").split("\t")
            iv = h.index("verdict")
            for line in fh:
                if not line.strip():
                    continue
                v = line.rstrip("\n").split("\t")[iv]
                if v == "STUCK-ON-SVPP-TOKEN":
                    corroborated += 1
                else:
                    undecidable += 1

    L = []
    L.append("# SV corpus VERDICT COVERAGE — what fraction was asked a question it could "
             "answer? (SV-CORPUS-GRAD.13)\n")
    L.append("> Generated by `stimuli/sv/corpus_verdict_coverage.py`. ⛔ A row with no "
             "verdict is **unknown**, not clean — the two are different words, and only "
             "one of them supports a signoff claim.\n")
    L.append("## Instrument identity\n")
    L.append("| input | repo-root-relative path | sha256 |")
    L.append("|---|---|---|")
    for label, p in (("sv manifest", args.manifest),
                     ("v2005 manifest", args.manifest_v2005)):
        L.append(f"| {label} | `{p.relative_to(ROOT)}` | `{sha256(p)}` |")
    L.append("")
    L.append("## The headline\n")
    for b in BUCKETS:
        L.append(f"- **{b}: {by_bucket[b]:,} rows ({100*by_bucket[b]/total:.1f} %)**")
    L.append("")
    L.append("## Every class, bucketed\n")
    L.append("| bucket | adjudication class | rows | % | why |")
    L.append("|---|---|---:|---:|---|")
    for b in BUCKETS:
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            if CLASSES[cls][0] != b:
                continue
            L.append(f"| {b} | `{cls}` | {n:,} | {100*n/total:.1f} % | "
                     f"{CLASSES[cls][1]} |")
    L.append(f"| | **TOTAL** | **{total:,}** | **100.0 %** | |")
    L.append("")
    if corroborated or undecidable:
        L.append("## Inside `ADJUDICATED`, the `explained_svpp_*` rows split again "
                 "(SV-CORPUS-GRAD.12)\n")
        L.append(f"- **{corroborated:,} corroborated** — the parse stops *on* the "
                 "preprocessor construct. The label is positively verified.")
        L.append(f"- **{undecidable:,} undecidable by position** — the label is neither "
                 "corroborated nor refuted; only running an expander settles them "
                 "(`.12b`).")
        L.append("")
        L.append(f"⇒ Rows contributing nothing to the confidence claim, counting these: "
                 f"**{by_bucket['NO VERDICT'] + undecidable:,} "
                 f"({100*(by_bucket['NO VERDICT'] + undecidable)/total:.1f} %)**.")
        L.append("")
    L.append("## `NO VERDICT`, by suite — where the silence actually is\n")
    L.append("| suite | class | rows |")
    L.append("|---|---|---:|")
    per = Counter((s, c) for s, c in rows if CLASSES[c][0] == "NO VERDICT")
    for (s, c), n in per.most_common():
        L.append(f"| {s} | `{c}` | {n:,} |")
    L.append("")

    args.outdir.mkdir(parents=True, exist_ok=True)
    (args.outdir / "coverage.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    with (args.outdir / "coverage.tsv").open("w", encoding="utf-8") as fh:
        fh.write("bucket\tclass\trows\tpct\n")
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            fh.write(f"{CLASSES[cls][0]}\t{cls}\t{n}\t{100*n/total:.2f}\n")

    for b in BUCKETS:
        print(f"{b:>12}: {by_bucket[b]:6,}  ({100*by_bucket[b]/total:.1f} %)")
    if undecidable:
        print(f"{'+undecidable':>12}: {undecidable:6,}  (inside ADJUDICATED)")
    print(f"wrote {(args.outdir / 'coverage.md').relative_to(ROOT)}")


if __name__ == "__main__":
    main()
