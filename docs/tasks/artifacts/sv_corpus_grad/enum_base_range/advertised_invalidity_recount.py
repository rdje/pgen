#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.3.23` side-effect bookkeeping — re-size `.3.24`'s population.

`.3.24` records that **71 of the 296** `unexplained_rejects_valid` rows sit in files whose
PATH advertises intentional invalidity. `.3.23`'s 9 pins move 4 of those rows out
(`_bad` appears in the basename of `t_enum_bad_value.v`, `t_enum_bad_wrap.v`,
`t_enum_type_methods_bad.v`, `t_enum_type_nomethod_bad.v`), so that tracked number went
stale the moment the pins landed. This script is what re-derives it.

⛔ WHY IT EXISTS AT ALL: `.3.24`'s 71 was banked as a bare number with no re-runnable
instrument behind it, so "is it still 71?" could only be answered by re-deriving the
predicate and hoping it matched. It did not, on the first two guesses — `_bad` as a *stem
suffix* yields 64, not 71. The predicate that reproduces the tracked number exactly
(71 = verilator 59 + sv2v 11 + sv-tests 1) is `_bad` **anywhere in the basename**, plus a
`test/error/` directory, plus an `_ILLEGAL` marker. Pinning the predicate in code is the
difference between a number and a measurement.

⛔ AND THE NUMBER IS A SIZE, NOT A VERDICT. `.3.24` is explicit that the filename is NOT an
oracle: verilator's `_bad` overwhelmingly means "this test expects an ELABORATION error",
which is perfectly parseable, so `must_accept` stays the correct default for the class.
This counts how big the class is; it adjudicates nothing.

⭐ GROUND TRUTH ([[feedback_instrument_needs_ground_truth]]): the run REFUSES unless the
predicate still reproduces `.3.24`'s recorded baseline on the pre-pin manifest. That
baseline is read straight out of git (`--baseline-rev`, default the pin commit's parent), so
the control needs no external file and no absolute path — it is reproducible from any clone.
Without a resolvable baseline the leg is reported SKIPPED, never silently assumed.

Usage (repo-root-relative, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/enum_base_range/advertised_invalidity_recount.py
  ... --baseline-rev <rev>      # the pre-.3.23 revision to re-prove the 71 against
  ... --no-baseline             # census only
"""

import argparse
import subprocess
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "config_use_param_override"))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
MANIFEST_REL = "stimuli/sv/characterization/adjudication_manifest.tsv"
# `.3.24`'s recorded baseline, measured 2026-08-09 against the pre-.3.23 manifest.
BASELINE_TOTAL = 71
BASELINE_SPLIT = {"verilator": 59, "sv2v": 11, "sv-tests": 1}


def show(rev: str, rel: str):
    r = subprocess.run(["git", "-C", str(ROOT), "show", f"{rev}:{rel}"],
                       capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def revisions_of(rel: str, depth: int):
    r = subprocess.run(
        ["git", "-C", str(ROOT), "log", f"-{depth}", "--format=%H", "--", rel],
        capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"REFUSE: `git log` failed for {rel} — not a clone?")
    return r.stdout.split()


def advertises_invalidity(relpath: str) -> bool:
    """`.3.24`'s predicate, pinned: a `_bad` basename, a `test/error/` directory, or an
    `_ILLEGAL` marker. Reproduces the recorded 71/59/11/1 exactly."""
    p = relpath.replace("\\", "/")
    base = p.rsplit("/", 1)[-1]
    return "_bad" in base or "/test/error/" in "/" + p or "_ILLEGAL" in base


def census_text(text: str):
    rows = [l.split("\t") for l in text.splitlines()[1:]]
    sel = [r for r in rows if len(r) > 4 and r[4] == "divergence:unexplained_rejects_valid"]
    hit = [r for r in sel if advertises_invalidity(r[1])]
    return len(sel), hit


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--manifest", type=Path,
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv")
    ap.add_argument("--baseline-rev", default=None,
                    help="pin the baseline revision explicitly; by default the newest "
                         "tracked vintage reproducing the recorded number is SEARCHED for")
    ap.add_argument("--baseline-search-depth", type=int, default=40)
    ap.add_argument("--no-baseline", action="store_true",
                    help="census only; skip the ground-truth leg")
    args = ap.parse_args()

    print("## GROUND TRUTH — does the predicate still reproduce `.3.24`'s recorded baseline?")
    if args.no_baseline:
        print("  SKIPPED (--no-baseline) — the baseline leg is NOT assumed.\n")
    else:
        # ⛔ NOT a fixed default rev. `HEAD` names the pre-pin manifest only for as long as
        # the pins are unstaged; the moment they commit, a hard-coded default measures the
        # POST-pin vintage and the control silently inverts. So the baseline is SEARCHED:
        # the newest tracked vintage of the manifest that reproduces the recorded number,
        # named in the output. Same trap `.3.21` found in the burn-down PICK step (a
        # default that quietly read a stale vintage), refused here by construction.
        revs = ([args.baseline_rev] if args.baseline_rev else
                revisions_of(MANIFEST_REL, args.baseline_search_depth))
        found = None
        for rev in revs:
            text = show(rev, MANIFEST_REL)
            if text is None:
                continue
            total, hit = census_text(text)
            split = dict(Counter(x[0] for x in hit))
            if len(hit) == BASELINE_TOTAL and split == BASELINE_SPLIT:
                found = (rev, total, len(hit), split)
                break
        if found is None:
            raise SystemExit(
                f"REFUSE: no tracked vintage of {MANIFEST_REL} within the last "
                f"{args.baseline_search_depth} revisions reproduces `.3.24`'s recorded "
                f"{BASELINE_TOTAL} {BASELINE_SPLIT} — the predicate and the leaf disagree, "
                "so the census would be measuring a different class than the leaf records.")
        rev, total, n_hit, split = found
        print(f"  baseline: git show {rev[:8]}:{MANIFEST_REL}")
        print(f"    unexplained_rejects_valid={total}  advertised={n_hit} {split}")
        print(f"    expected {BASELINE_TOTAL} {BASELINE_SPLIT} -> PASS\n")

    total, hit = census_text(
        args.manifest.read_text(encoding="utf-8", errors="replace"))
    split = dict(Counter(r[0] for r in hit))
    print("## CENSUS — at HEAD")
    print(f"  manifest: {args.manifest.relative_to(ROOT)}")
    print(f"  unexplained_rejects_valid: {total}")
    print(f"  of which the PATH advertises intentional invalidity: {len(hit)}")
    for suite, n in sorted(split.items()):
        print(f"    {suite:<12} {n}")
    print()
    print("  ⛔ A SIZE, NOT A VERDICT: verilator's `_bad` usually names an ELABORATION error,")
    print("     which parses fine, so `must_accept` remains the correct default for the class.")
    print("     `.3.24` owns the audit; this only keeps its number from going stale.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
