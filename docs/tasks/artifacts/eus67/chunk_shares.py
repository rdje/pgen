#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) leg 3 — turn the chunk-capacity TSV into the shares.

`-0121` could only bound the arena's share of the parse footprint by
`[items, ~2 x items]`, because `typed_arena` exposes no chunk-capacity accessor; that
single exclusion is what left the arena anywhere in 28-71 %. The instrument now MEASURES
chunk capacity, so this reports the band that replaces it.

⛔⛔ IT IS STILL A BAND, AND THE SECOND END IS NOT WHAT THE FIRST DRAFT OF THIS SCRIPT
PUBLISHED. Chunk capacity is what the arena took from the ALLOCATOR, and this leaf is
attributing PEAK RSS: a fresh chunk is untouched anonymous memory whose pages enter RSS
only when written. MEASURED by `measure_chunk_residency.sh` — one extra item doubles the
allocated bytes (1 008 -> 2 016 MB) and moves peak RSS by 32 768 BYTES. So the honest
attribution is `touched .. allocated`, TOUCHED is the RSS-relevant end, and quoting
`allocated` against an RSS denominator attributes the arena's unused slack to the arena.

Two rules from earlier slices are carried forward deliberately, and the SECOND one is
carried forward CORRECTED:

1. The process floor is a BAND (14.72-18.86 MB, `.67`(a)), so every floor-subtracted
   share is printed at both ends and a conclusion that survives only one end is not a
   conclusion.
2. ⛔ The 2x-floor admissibility rule is scoped to the quantity it was derived for. It
   governs floor-SUBTRACTED shares, whose denominator is unreliable near the floor. It
   does NOT govern the chunk-overhead ratio (`chunk bytes / item bytes`), which subtracts
   no floor at all — so that column is reported over EVERY row, `floor.sv` included.
   `-0125`(3) found the opposite mistake in `-0124`: a range trimmed by a rule borrowed
   from a different column, and trimmed in the flattering direction.
"""
import sys

FLOOR_LO = 14.72 * 1024 * 1024
FLOOR_HI = 18.86 * 1024 * 1024
MB = 1024.0 * 1024.0


def num(cell):
    return None if cell in ("NA", "") else float(cell)


def main(path):
    rows = [l.rstrip("\n").split("\t") for l in open(path)]
    head, body = rows[0], rows[1:]
    idx = {name: i for i, name in enumerate(head)}
    missing = [c for c in ("allocated_total_MB", "touched_total_MB", "arena_ast_touched_MB",
                           "arena_ast_allocated_MB", "items_agree", "extend_regrow")
               if c not in idx]
    if missing:
        print(f"REFUSE: {path} has no leg-3 columns ({', '.join(missing)}) — "
              f"it was written by a pre-leg-3 producer")
        return 2

    # ------------------------------------------------------------------ integrity
    # A byte figure from a model whose item count disagrees with the arena's own
    # `len()` is not a measurement, so this is a REFUSAL and not a warning.
    bad_agree = [r[0] for r in body if r[idx["items_agree"]] != "true"]
    bad_regrow = [r[0] for r in body if r[idx["extend_regrow"]] not in ("0",)]
    if bad_agree:
        print(f"REFUSE: model/arena item counts disagree on {len(bad_agree)} row(s): "
              f"{', '.join(bad_agree)}")
        return 2
    print(f"integrity: items_agree=true on {len(body)}/{len(body)} rows; "
          f"extend_regrow non-zero on {len(bad_regrow)} row(s)"
          + (f" ({', '.join(bad_regrow)})" if bad_regrow else ""))

    # ⭐ THE STRUCTURAL CLAIM, CHECKED PER ARENA RATHER THAN ARGUED. A chunk is retired
    # either FULL (the `alloc` path) or at exactly the item count it held (the
    # `alloc_extend` reserve path), and the only path that can leave written-but-dead
    # slots behind needs an iterator that understates its own `size_hint` — which is
    # what `extend_regrow` counts. So for PGEN's call pattern WRITTEN slots should equal
    # LIVE items, arena for arena. If that holds, the pre-leg-3 item-bytes figure was
    # already the RSS-relevant one and the 28-71 % band was never an RSS band.
    pairs = [("nodes", "arena_nodes", "chunk_nodes_touched"),
             ("shaped_values", "arena_shaped_values", "chunk_sv_touched"),
             ("shaped_pairs", "arena_shaped_pairs", "chunk_sp_touched"),
             ("rendered_strings", "arena_rendered_strings", "chunk_rs_touched")]
    eq = neq = 0
    for _, items_col, touched_col in pairs:
        if items_col not in idx or touched_col not in idx:
            continue
        for r in body:
            i, t = num(r[idx[items_col]]), num(r[idx[touched_col]])
            if i is None or t is None:
                continue
            if i == t:
                eq += 1
            else:
                neq += 1
                print(f"  WRITTEN != ITEMS  {r[0].split('/')[-1]} {touched_col}: {t} vs {i}")
    print(f"written slots == live items on {eq}/{eq + neq} measurable arena rows")

    # ------------------------------------------- chunk overhead (no floor involved)
    print(f"\n{'input':<34}{'KB':>8}{'items MB':>10}{'touched MB':>12}{'alloc MB':>10}"
          f"{'writ/item':>11}{'slack %':>9}{'spine B':>9}")
    overheads, slacks = [], []
    for r in body:
        items = num(r[idx["arena_total_MB"]])
        touched = num(r[idx["touched_total_MB"]])
        alloc = num(r[idx["allocated_total_MB"]])
        name = r[0].split("/")[-1]
        if items is None or touched is None or alloc is None:
            print(f"{name:<34}  NOT MEASURED")
            continue
        over = touched / items if items else float("nan")
        slack = 100.0 * (alloc - touched) / alloc if alloc else float("nan")
        overheads.append((over, name))
        slacks.append((slack, name))
        print(f"{name:<34}{float(r[idx['bytes']])/1024:>8.0f}"
              f"{items:>10.2f}{touched:>12.2f}{alloc:>10.2f}"
              f"{over:>11.3f}{slack:>9.1f}{r[idx['spine_B']]:>9}")
    if overheads:
        overheads.sort(); slacks.sort()
        mid = len(overheads) // 2
        print(f"\nWRITTEN chunk bytes / arena ITEM bytes, over ALL {len(overheads)} rows "
              f"(no admissibility rule governs this column): min {overheads[0][0]:.3f} "
              f"({overheads[0][1]}), median {overheads[mid][0]:.3f}, "
              f"max {overheads[-1][0]:.3f} ({overheads[-1][1]})")
        print(f"ALLOCATED-BUT-UNWRITTEN slack, share of allocated arena bytes: "
              f"min {slacks[0][0]:.1f} % ({slacks[0][1]}), median {slacks[len(slacks)//2][0]:.1f} %, "
              f"max {slacks[-1][0]:.1f} % ({slacks[-1][1]}) — NOT resident, measured")

    # ------------------------------------------------- floor-subtracted AST shares
    print(f"\n{'input':<34}{'peak MB':>10}{'items%lo':>10}{'items%hi':>10}"
          f"{'TOUCHED%lo':>12}{'TOUCHED%hi':>12}{'alloc%lo':>10}")
    lo_shares, hi_shares, excluded = [], [], []
    for r in body:
        rss = num(r[idx["peak_rss_bytes"]])
        lower = num(r[idx["arena_owned_MB"]])
        resident = num(r[idx["arena_ast_touched_MB"]])
        allocated = num(r[idx["arena_ast_allocated_MB"]])
        name = r[0].split("/")[-1]
        if rss is None or lower is None or resident is None or allocated is None:
            print(f"{name:<34}  NOT MEASURED (a missing field is not a zero)")
            continue
        if rss < 2 * FLOOR_HI:
            excluded.append(name)
            print(f"{name:<34}{rss/MB:>10.0f}   EXCLUDED — peak < 2x the floor band")
            continue
        marg_lo = (rss - FLOOR_HI) / MB   # smallest marginal => LARGEST share
        marg_hi = (rss - FLOOR_LO) / MB
        print(f"{name:<34}{rss/MB:>10.0f}"
              f"{100*lower/marg_lo:>10.1f}{100*lower/marg_hi:>10.1f}"
              f"{100*resident/marg_lo:>12.1f}{100*resident/marg_hi:>12.1f}"
              f"{100*allocated/marg_lo:>10.1f}")
        lo_shares.append(100 * resident / marg_lo)
        hi_shares.append(100 * resident / marg_hi)
    if lo_shares:
        lo_shares.sort(); hi_shares.sort()
        mid = len(lo_shares) // 2
        print(f"\nadmitted {len(lo_shares)} row(s); excluded {len(excluded)}: "
              f"{', '.join(excluded) if excluded else 'none'}")
        print(f"TOUCHED arena+AST share of the floor-subtracted peak "
              f"(the RSS-relevant end of the band): "
              f"{min(lo_shares):.1f}%-{max(lo_shares):.1f}% at the 14.72 MB floor, "
              f"{min(hi_shares):.1f}%-{max(hi_shares):.1f}% at 18.86 MB; "
              f"median {lo_shares[mid]:.1f}% / {hi_shares[mid]:.1f}%")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1
                  else "docs/tasks/artifacts/eus67/attribution_chunks.tsv"))
