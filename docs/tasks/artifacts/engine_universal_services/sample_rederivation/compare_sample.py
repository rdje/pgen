#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.21 acceptance (b) — IS THE PINNED SAMPLE STILL THE RIGHT SAMPLE?

`.21` slice 1 corrected the LR-family classifier and measured that re-deriving the pinned
192-file sample would move **24 rows** (`hot` 0/40, `lr` 5/40, `breadth` 19/112). It DECLINED to
adopt that, and the decisive reason was not a preference:

  ⛔ the census was NOT REPRODUCIBLE. Re-deriving the EXISTING predicate's sample from a fresh
     census reproduced `hot` 40/40 and `lr` 40/40 exactly, and `breadth` differed by 5 of 112 —
     entirely because the census yielded 16 335 rows instead of 16 336. `.22`'s single silently
     dropped file perturbed the per-suite pools and moved the stride.

`.22`(e) fixed that file (it now dumps in 0.04 s; the census is 16 336/16 336, 0 no-dump) and
`.22`(c) made an undeclared drop REFUSE. So the blocker is gone and (b) can be adjudicated on a
reproducible basis — which is all (b) ever asked for.

⭐⭐ THIS SCRIPT IMPORTS `select_sample` FROM THE INSTRUMENT. It does not re-implement it.
`.21` acceptance (e) is a record of what happens otherwise: two files had RE-TYPED the family
predicate and had already drifted, and a third re-implemented the `lr` tier WITHOUT the
hot-exclusion, so its headline `0/40` was structurally meaningless while its docstring claimed to
answer this very acceptance item. One derivation, one home.

WHAT IT PROVES, in order — each leg is refused rather than assumed:

  1. REPRODUCIBILITY. Two independent full-corpus censuses must be byte-identical on the columns
     the selection reads (path, entries, lr_entries). This is the leg `.21` slice 1 could not pass.
  2. SELECTION STABILITY. The sample derived from census A must equal the sample derived from
     census B. A deterministic derivation over identical inputs that disagreed would mean the
     derivation itself is not deterministic — a different defect, and worth separating.
  3. THE DELTA. The freshly derived sample versus the TRACKED manifest, per tier, with the rows
     named. This is the number acceptance (b) asks for.

Usage:
  python3 docs/tasks/artifacts/engine_universal_services/sample_rederivation/compare_sample.py \
      --census-a <dirA>/census.tsv --census-b <dirB>/census.tsv

Output: `result.txt` beside this script (tracked), plus a nonzero exit on any refusal.
CONTRACT: reads only; writes exactly one artifact; never edits the tracked manifest.
"""

from __future__ import annotations

import argparse
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
# sample_rederivation -> engine_universal_services -> artifacts -> tasks -> docs -> ROOT
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", "..", "..", ".."))
INSTRUMENT = os.path.join(ROOT, "stimuli", "sv", "corpus_parse_cost.py")
if not os.path.isfile(INSTRUMENT):
    # ⛔ REFUSE rather than proceed with a half-resolved root. A wrong `..` count is the most
    # likely edit-time defect in a script that lives five levels down, and it must not surface as
    # a confusing ImportError from deep inside importlib.
    sys.exit(f"cannot resolve the repo root from {HERE}: {INSTRUMENT} does not exist")
TRACKED_MANIFEST = os.path.join(ROOT, "stimuli", "sv", "parse_cost_sample.tsv")
RESULT = os.path.join(HERE, "result.txt")

# Populated from `corpus_parse_cost.SAMPLE_TIERS` at run time; never literals.
HOT = LR = BREADTH = 0

# ⛔ THE TIER SIZES ARE IMPORTED, AND THIS COMMENT USED TO CLAIM THAT WHILE THE LINE BELOW IT
# RE-TYPED THEM. Measured during this slice: it read `40, 40, 112` — 112 being the number of rows
# in the manifest — against the derivation's real `breadth` default of **120**. The two happen to
# realize the identical sample (both floor to a per-suite quota of 8 across 14 sub-corpora), so
# the defect was silent and produced correct output. That is the founding defect of this entire
# leaf, reproduced by its own audit script, and caught only by reading the producer. ⇒ the sizes
# now have ONE home, `corpus_parse_cost.SAMPLE_TIERS`, and this file reads it.


def load_instrument():
    """⛔ IMPORT the shipped derivation. A re-typed copy is the defect `.21`(e) closed."""
    spec = importlib.util.spec_from_file_location("corpus_parse_cost", INSTRUMENT)
    if spec is None or spec.loader is None:
        sys.exit(f"cannot load {INSTRUMENT}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def read_census(path: str) -> dict[str, tuple[int, int]]:
    """The three columns `select_sample` actually reads, keyed by path.

    ⛔ Parsed with the SAME row filter the instrument uses (5 fields, col 2 in True/False), so a
    row this reader accepts and the selection rejects cannot exist. A comparison whose parser is
    laxer than the thing it audits reports differences that are its own.
    """
    out: dict[str, tuple[int, int]] = {}
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 5 or parts[2] not in ("True", "False"):
                continue
            out[parts[1]] = (int(parts[3]), int(parts[4]))
    return out


def read_manifest(path: str) -> dict[str, str]:
    out: dict[str, str] = {}
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) == 2:
                out[parts[1]] = parts[0]
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--census-a", required=True)
    ap.add_argument("--census-b", required=True)
    args = ap.parse_args()

    mod = load_instrument()
    # ⛔ IMPORTED, never re-typed — see the SAMPLE_TIERS note above.
    global HOT, LR, BREADTH
    HOT, LR, BREADTH = (mod.SAMPLE_TIERS["hot"], mod.SAMPLE_TIERS["lr"],
                        mod.SAMPLE_TIERS["breadth"])
    lines: list[str] = []

    def A(s: str = "") -> None:
        lines.append(s)
        print(s)

    A("=" * 78)
    A("ENGINE-UNIVERSAL-SERVICES.21 acceptance (b) — pinned-sample re-derivation")
    A("=" * 78)
    A(f"instrument: stimuli/sv/corpus_parse_cost.py (select_sample IMPORTED, not re-typed)")
    A(f"classifier: {mod.LR_FAMILY_RE.pattern}")
    A(f"tiers:      hot={HOT} lr={LR} breadth={BREADTH} (request; realized breadth is "
      f"len(sub_corpora) * (breadth // len(sub_corpora)))")
    A()

    refusals = 0

    # ── LEG 1: is the census reproducible at all? ────────────────────────────────────────────
    ca, cb = read_census(args.census_a), read_census(args.census_b)
    A("LEG 1 — CENSUS REPRODUCIBILITY (the leg `.21` slice 1 could not pass)")
    A(f"  census A rows: {len(ca):,}")
    A(f"  census B rows: {len(cb):,}")
    only_a = sorted(set(ca) - set(cb))
    only_b = sorted(set(cb) - set(ca))
    differing = sorted(p for p in set(ca) & set(cb) if ca[p] != cb[p])
    if only_a or only_b:
        refusals += 1
        A(f"  ⛔ ROW SETS DIFFER — {len(only_a)} only in A, {len(only_b)} only in B")
        for p in (only_a + only_b)[:10]:
            A(f"       {p}")
    if differing:
        refusals += 1
        A(f"  ⛔ {len(differing)} file(s) report DIFFERENT counters across the two censuses")
        for p in differing[:10]:
            A(f"       {p}: A={ca[p]} B={cb[p]}")
    if not only_a and not only_b and not differing:
        A(f"  ✅ IDENTICAL on every column the selection reads (path, entries, lr_entries).")
        A(f"     ⇒ `.21` slice 1's decisive blocker is CLEARED: a sample derived today can be")
        A(f"       pinned honestly. (It differed by 5 breadth rows then, on 16 335 vs 16 336.)")
    A()

    # ── LEG 2: is the SELECTION deterministic over identical inputs? ─────────────────────────
    sa = mod.select_sample(args.census_a, HOT, LR, BREADTH)
    sb = mod.select_sample(args.census_b, HOT, LR, BREADTH)
    A("LEG 2 — SELECTION STABILITY")
    if sa != sb:
        refusals += 1
        A(f"  ⛔ the two derivations DISAGREE ({len(set(sa) ^ set(sb))} differing rows) even though")
        A(f"     leg 1 found their inputs identical — the derivation is not deterministic.")
    else:
        A(f"  ✅ both censuses derive the SAME {len(sa)} rows.")
    A()

    # ── LEG 3: the delta acceptance (b) asks for ─────────────────────────────────────────────
    tracked = read_manifest(TRACKED_MANIFEST)
    fresh = {rel: tier for tier, rel in sa}
    A("LEG 3 — FRESH DERIVATION vs THE TRACKED PINNED MANIFEST")
    A(f"  tracked rows: {len(tracked):,}   fresh rows: {len(fresh):,}")
    A()
    A("  | tier | tracked | fresh | rows that LEAVE | rows that ENTER |")
    A("  |---|---:|---:|---:|---:|")
    for tier in ("hot", "lr", "breadth"):
        t = {p for p, v in tracked.items() if v == tier}
        f = {p for p, v in fresh.items() if v == tier}
        A(f"  | `{tier}` | {len(t)} | {len(f)} | {len(t - f)} | {len(f - t)} |")
    A()
    moved = sorted(set(tracked) ^ set(fresh))
    # ⛔ BOTH COUNTS, LABELLED. `.21` slice 1 published "24 of 192 rows would move", counting
    # SLOTS. The symmetric difference is twice that, because every slot vacated is also a slot
    # filled. Printing only the larger number here would read as "the delta grew since slice 1"
    # when nothing has changed — a false regression manufactured by a counting convention.
    A(f"  SLOTS whose occupant changes: **{len(moved) // 2}** of {len(tracked)}"
      f"   (= `.21` slice 1's counting convention)")
    A(f"  symmetric difference (rows out + rows in): **{len(moved)}**")
    retiered = sorted(p for p in set(tracked) & set(fresh) if tracked[p] != fresh[p])
    A(f"  rows present in both but in a DIFFERENT tier: **{len(retiered)}**")
    A()
    if moved:
        A("  rows that LEAVE the sample:")
        for p in sorted(set(tracked) - set(fresh)):
            A(f"    - [{tracked[p]:7s}] {p}")
        A("  rows that ENTER the sample:")
        for p in sorted(set(fresh) - set(tracked)):
            A(f"    + [{fresh[p]:7s}] {p}")
    if retiered:
        A("  rows that change TIER:")
        for p in retiered:
            A(f"    ~ {tracked[p]} -> {fresh[p]}  {p}")
    A()

    # ── LEG 4: an EXTERNAL oracle this script did not build ──────────────────────────────────
    #
    # ⭐ The tracked `entries.tsv` was produced by a DIFFERENT code path — a 192-file sample run,
    # in a different session, on the pinned manifest. Summing the full-corpus census over exactly
    # those 192 paths must reproduce its totals to the digit. If it does not, this comparison is
    # misreading the census and every number below it is void — so the check runs BEFORE they are
    # published, not after (`docs/CLAIM_VERIFICATION.md`: falsify against an oracle you did not
    # build).
    def entries_of(paths) -> int:
        return sum(ca[p][0] for p in paths if p in ca)

    t_all, f_all = set(tracked), set(fresh)
    A("LEG 4 — EXTERNAL ORACLE: does the census reproduce the tracked baseline?")
    baseline = os.path.join(
        ROOT, "docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/entries.tsv")
    if os.path.isfile(baseline):
        b_entries = b_lr = b_rows = 0
        with open(baseline, encoding="utf-8") as fh:
            header = fh.readline().rstrip("\n").split("\t")
            try:
                ie, il = header.index("entries"), header.index("lr_entries")
            except ValueError:
                ie = il = -1
            if ie < 0:
                refusals += 1
                A("  ⛔ entries.tsv header has no `entries`/`lr_entries` column — REFUSING rather")
                A("     than unpacking positionally (gate-flow §7.8).")
            else:
                for line in fh:
                    if not line.strip():
                        continue
                    c = line.rstrip("\n").split("\t")
                    b_entries += int(c[ie]); b_lr += int(c[il]); b_rows += 1
                c_entries = entries_of(t_all)
                c_lr = sum(ca[p][1] for p in t_all if p in ca)
                A(f"  baseline entries.tsv ({b_rows} rows): entries={b_entries:,} "
                  f"lr_entries={b_lr:,}")
                A(f"  fresh census over those paths:        entries={c_entries:,} "
                  f"lr_entries={c_lr:,}")
                if (b_entries, b_lr) == (c_entries, c_lr):
                    A("  ✅ EXACT on both columns, across two independent code paths and sessions.")
                else:
                    refusals += 1
                    A("  ⛔ THEY DISAGREE — this comparison is misreading the census; nothing")
                    A("     below is adoptable.")
    else:
        A(f"  ⚠️ NOT EVALUATED — {baseline} is absent.")
    A()

    A("WHAT ADOPTING WOULD COST AND BUY (measured, not argued)")
    A(f"  binding-baseline entries, tracked sample: {entries_of(t_all):,}")
    A(f"  binding-baseline entries, fresh sample:   {entries_of(f_all):,}")
    if entries_of(t_all):
        d = 100.0 * (entries_of(f_all) - entries_of(t_all)) / entries_of(t_all)
        A(f"  ⇒ the ratchet's binding total would move {d:+.2f} % — a ONE-TIME baseline reset,")
        A(f"    which is the cost. The buy is that the `lr` tier finally means what it says.")
    lr_fam_tracked = sum(ca[p][1] for p in t_all if p in ca)
    lr_fam_fresh = sum(ca[p][1] for p in f_all if p in ca)
    A(f"  LR-family entries covered, tracked sample: {lr_fam_tracked:,}")
    A(f"  LR-family entries covered, fresh sample:   {lr_fam_fresh:,}")
    if lr_fam_tracked:
        d = 100.0 * (lr_fam_fresh - lr_fam_tracked) / lr_fam_tracked
        A(f"  ⇒ coverage of the mechanism `.20` exists to watch moves {d:+.2f} %.")
    A()

    A("=" * 78)
    if refusals:
        A(f"⛔ {refusals} REFUSAL(S) — the numbers above are NOT adoptable.")
    else:
        A("✅ all legs pass — the delta above is a sound basis for the (b) adjudication.")
    A("=" * 78)

    with open(RESULT, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    return 1 if refusals else 0


if __name__ == "__main__":
    sys.exit(main())
