#!/usr/bin/env python3
"""CORPUS-KEY-AUDIT.1 — the answer-key CONTRADICTION census.

WHAT IT ANSWERS
  Do any two corpus rows, keyed from the SAME upstream evidence, carry OPPOSITE expected
  verdicts? A corpus large enough to contain a contradiction is an oracle you already own,
  and the question is mechanical: group rows by the upstream messages their key reads, and
  flag any message class whose rows disagree about `must_accept` vs `must_reject`.

WHY IT EXISTS
  `SV-CORPUS-GRAD.13e.3` found `br_gh552.v` and `real_invalid_ops.v` — the SAME operator,
  OPPOSITE expectations — BY HAND, and only because one of them happened to be adjudicated
  for another reason. The disagreement had been sitting in one tracked file for the life of
  the campaign, and nothing in the repository was asking the question.

⛔⛔ THIS CENSUS OVER-REPORTS BY CONSTRUCTION, AND SAYING SO IS HALF ITS VALUE.
  A row carries ONE expected verdict; its golden may carry SEVERAL messages. Attributing
  every message to the row's verdict therefore manufactures disagreements whenever a file
  was pinned for a reason unrelated to most of what its golden says — measured at adoption:
  both reported classes are exactly that shape (`br_gh1087b` is pinned for a NET DATA TYPE
  while its golden also reports a multiple-driver elaboration error). ⇒ the output is a
  WORKLIST OF CANDIDATES, never a defect count. `CORPUS-KEY-AUDIT.1` owns the refinement:
  attribute a row to its DECIDING message (the one the parser's own `furthest_position`
  lands on) rather than to all of them. Until then, every row here is adjudicated by hand.

Usage:
  python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py [--md OUT]
"""
import argparse
import collections
import csv
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
IVTEST = ROOT / "stimuli/sv/subs/iverilog/ivtest"
GOLD = IVTEST / "gold"
MANIFESTS = {
    "sv_2017": ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
    "verilog_2005": ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
}
MSG = re.compile(r"^[^:]*\.v(?:\.\w+)?:\d+:\s*(?:error|sorry):\s*(.+?)\s*$", re.M)
VERDICTS = ("must_accept", "must_reject")


def normalize(msg: str) -> str:
    """Collapse the parts an upstream message varies per file: quoted names and integers.

    ⛔ Deliberately NOT a prefix match. `SV-CORPUS-GRAD.3.24` measured a prefix rule sweeping
    preprocessor and command-file messages into a parse verdict; the same hazard applies here.
    """
    msg = re.sub(r"[`'\"][^`'\"]*[`'\"]", "X", msg)
    return re.sub(r"\b\d+\b", "N", msg).strip()


def gold_text(stem: str) -> str:
    text = ""
    for chan in ("iverilog-stderr", "iverilog-stdout"):
        f = GOLD / f"{stem}-{chan}.gold"
        if f.is_file():
            text += f.read_text(errors="replace")
    plain = GOLD / f"{stem}.gold"
    if plain.is_file():
        text += plain.read_text(errors="replace")
    return text


def gold_index() -> dict:
    """source stem -> golden stem, from BOTH descriptor sources the ivtest harness uses."""
    out = {}
    for jf in sorted((IVTEST / "vvp_tests").glob("*.json")):
        try:
            d = json.loads(jf.read_text(errors="replace"))
        except ValueError:
            continue
        if d.get("gold") and d.get("source"):
            out.setdefault(Path(d["source"]).stem, Path(d["gold"]).stem)
    lst = IVTEST / "regress-vlg.list"
    if lst.is_file():
        for line in lst.read_text(errors="replace").splitlines():
            parts = line.strip().split()
            if len(parts) < 2:
                continue
            g = next((p.split("=", 1)[1] for p in parts if p.startswith("gold=")), None)
            if g:
                out.setdefault(parts[0], Path(g).stem)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--md", type=Path, default=Path(__file__).with_name("census.md"))
    args = ap.parse_args()

    if not IVTEST.is_dir():
        print(f"⛔ REFUSING: {IVTEST} is absent (the vendored corpora are git submodules). "
              "This census is NOT EVALUATED — never report that as a pass.", file=sys.stderr)
        return 2

    gold_of = gold_index()
    by_msg = collections.defaultdict(lambda: collections.defaultdict(set))
    rows_seen = collections.Counter()
    for lane, path in MANIFESTS.items():
        if not path.is_file():
            continue
        with path.open(encoding="utf-8") as fh:
            for r in csv.DictReader(fh, delimiter="\t"):
                if r["suite"] != "iverilog" or r["expected"] not in VERDICTS:
                    continue
                gs = gold_of.get(Path(r["relpath"]).stem)
                if not gs:
                    continue
                rows_seen[lane] += 1
                for m in MSG.finditer(gold_text(gs)):
                    by_msg[normalize(m.group(1))][r["expected"]].add(
                        (lane, Path(r["relpath"]).stem))

    contra = {m: v for m, v in by_msg.items()
              if len([e for e in v if e in VERDICTS]) > 1}

    L = ["# Answer-key contradiction census — CORPUS-KEY-AUDIT.1", "",
         "> DERIVED. Re-run: `python3 "
         "docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`", ">",
         "> ⛔ **A CANDIDATE WORKLIST, NOT A DEFECT COUNT.** A row carries ONE verdict and its",
         "> golden may carry SEVERAL messages, so attributing every message to the row's verdict",
         "> manufactures a disagreement whenever a file was pinned for a reason unrelated to most",
         "> of what its golden says. Each row below is adjudicated BY HAND until `.1` lands the",
         "> deciding-message attribution.", "",
         f"- keyed rows scanned: **{sum(rows_seen.values())}** "
         f"({', '.join(f'{k} {v}' for k, v in sorted(rows_seen.items()))})",
         f"- distinct normalized upstream messages: **{len(by_msg)}**",
         f"- message classes carrying CONTRADICTORY expectations: **{len(contra)}**", ""]
    if contra:
        L += ["| upstream message (normalized) | `must_accept` rows | `must_reject` rows |",
              "|---|---|---|"]
        for m, v in sorted(contra.items(),
                           key=lambda kv: -sum(len(x) for x in kv[1].values())):
            cell = {}
            for e in VERDICTS:
                rows = sorted(f"{s}" for _l, s in v.get(e, ()))
                cell[e] = (f"**{len(rows)}** — " + ", ".join(f"`{r}`" for r in rows[:4])
                           + ("…" if len(rows) > 4 else "")) if rows else "—"
            L.append(f"| `{m}` | {cell['must_accept']} | {cell['must_reject']} |")
        L.append("")
    else:
        L += ["No message class carries contradictory expectations.", ""]

    args.md.write_text("\n".join(L) + "\n", encoding="utf-8")
    print(f"KEY-CONTRADICTION-CENSUS: rows={sum(rows_seen.values())} "
          f"messages={len(by_msg)} contradictory_classes={len(contra)}")
    print(f"wrote {args.md.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
