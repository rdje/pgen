#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13e.3 — the ivtest `-g` extension-flag census.

WHAT IT ANSWERS
  Which ivtest descriptors carry a `-g` flag that is NOT a generation flag, which of
  those flags actually decide what the LANGUAGE is, and where each of those rows is
  currently adjudicated.

WHY IT IS TRACKED
  `.13e.2` reported "two rows carry an extension flag" as a lower bound and said the
  sweep was `.13e.3`'s first job. A published population needs a producer anyone can
  re-run (`docs/CLAIM_VERIFICATION.md` leg 3), not a number in a leaf.

⛔ THE CLASSIFICATION IS NOT REPEATED HERE. It is imported from the adjudicator, which
   derives it from the vendored compiler and REFUSES when that compiler moves. A second
   copy is a second thing to drift.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/ivtest_extension_flags/census.py [--md OUT]
"""
import argparse
import collections
import csv
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(ROOT / "stimuli" / "sv"))
from adjudicate_external_corpus import IvtestIndex  # noqa: E402

IVTEST = ROOT / "stimuli/sv/subs/iverilog/ivtest"
MANIFESTS = {
    "sv": ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
    "v2005": ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
}


def load_manifest(p):
    if not p.is_file():
        return {}
    with p.open(encoding="utf-8") as fh:
        return {r["relpath"]: r for r in csv.DictReader(fh, delimiter="\t")}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--md", type=Path, default=Path(__file__).with_name("census.md"))
    args = ap.parse_args()

    if not IVTEST.is_dir():
        print(f"⛔ REFUSING: {IVTEST} is absent (vendored corpora are git submodules). "
              "This census is NOT EVALUATED — never report that as a pass.", file=sys.stderr)
        return 2

    idx = IvtestIndex(IVTEST)
    gens = set(IvtestIndex.SV_GENS) | set(IvtestIndex.V2005_GENS)
    lang = {f"-g{f}" for f in IvtestIndex.EXT_LANGUAGE_AFFECTING}

    descs, flag_counts, files = [], collections.Counter(), set()
    total_desc = 0
    for jf in sorted((IVTEST / "vvp_tests").glob("*.json")):
        try:
            d = json.loads(jf.read_text(errors="replace"))
        except ValueError:
            continue
        total_desc += 1
        other = [a for a in d.get("iverilog-args", [])
                 if a.startswith("-g") and a not in gens]
        if not other:
            continue
        for a in other:
            flag_counts[a] += 1
        src = d.get("source")
        rel = f"ivtest/ivltests/{src}" if src else None
        if rel:
            files.add(rel)
        descs.append((jf.stem, d.get("type"), d.get("iverilog-args", []), rel, other))

    man = {k: load_manifest(v) for k, v in MANIFESTS.items()}

    L = [f"# ivtest `-g` extension-flag census — SV-CORPUS-GRAD.13e.3", "",
         "> DERIVED. Re-run: `python3 "
         "docs/tasks/artifacts/sv_corpus_grad/ivtest_extension_flags/census.py`",
         "> The language-affecting classification is IMPORTED from "
         "`stimuli/sv/adjudicate_external_corpus.py`, which derives it from the vendored",
         "> compiler and REFUSES when that compiler moves — never re-stated here.", "",
         f"- vvp descriptors scanned: **{total_desc}**",
         f"- descriptors carrying a non-generation `-g` flag: **{len(descs)}**",
         f"- distinct source files behind them: **{len(files)}**",
         f"- distinct non-generation `-g` flags: **{len(flag_counts)}**", "",
         "## Flags, by descriptor count", "",
         "| flag | descriptors | decides the language? |", "|---|---:|---|"]
    for a, c in flag_counts.most_common():
        L.append(f"| `{a}` | {c} | {'**YES**' if a in lang else 'no'} |")

    L += ["", "## Every carrier, with its current adjudication", "",
          "| descriptor | type | `iverilog-args` | lane | adjudication |",
          "|---|---|---|---|---|"]
    lane_counts = collections.Counter()
    for stem, ttype, dargs, rel, _other in descs:
        r5 = man["v2005"].get(rel or "")
        rsv = man["sv"].get(rel or "")
        lane = "v2005" if r5 else ("sv" if rsv else "—")
        row = r5 or rsv
        adj = row["adjudication"] if row else "—"
        lane_counts[(lane, adj)] += 1
        L.append(f"| `{stem}` | {ttype} | `{' '.join(dargs)}` | {lane} | `{adj}` |")

    L += ["", "## Roll-up", "", "| lane | adjudication | descriptors |", "|---|---|---:|"]
    for (lane, adj), c in sorted(lane_counts.items(), key=lambda kv: -kv[1]):
        L.append(f"| {lane} | `{adj}` | {c} |")
    L.append("")

    args.md.write_text("\n".join(L) + "\n", encoding="utf-8")
    print(f"IVTEST-EXTENSION-CENSUS: descriptors={total_desc} carriers={len(descs)} "
          f"files={len(files)} flags={len(flag_counts)} "
          f"language_affecting={sum(c for a, c in flag_counts.items() if a in lang)}")
    print(f"wrote {args.md.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
