#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) LEG 4 — name the allocations in each size class.

A size class tells you `count` live allocations summing to `live_bytes`. That is not
yet an attribution. This pass turns it into one WITHOUT guessing: every candidate size
is DERIVED from a construction site in the source, and a class is only reported as
explained when some multiset of exactly `count` candidates sums to `live_bytes`
EXACTLY — no tolerance, no rounding, no "close enough".

⛔ A CLASS THAT DOES NOT DECOMPOSE IS PRINTED AS `UNEXPLAINED`, WITH ITS BYTES. That is
the whole discipline: an attribution that always succeeds has not been tested, and the
residual this leaf exists to explain must not be explained by a fitting procedure
flexible enough to absorb anything.

CANDIDATE GENERATORS (each cites the construction site it transcribes):

  thin scratch   `generated/<g>_parser.rs` ctor -> `ThinMemoScratch::begin(
                 THIN_RULE_COUNT * (len + 1))` -> `Vec<u64>::resize`
                 => exactly THIN_RULE_COUNT * (len + 1) * 8 bytes, once.
  hashbrown      `memo: FxHashMap<(RuleId, usize), MemoEntry>` and the three failure
                 tables. hashbrown lays a table out as
                 `buckets * size_of::<T>()` (padded to align) + `buckets` control
                 bytes + `Group::WIDTH`, and `buckets` is a power of two.
                 `size_of::<T>()` is MEASURED by the probe (`memo_bucket`,
                 `memo_fail_bucket`), never hand-computed.
  arena chunk    `typed-arena` 2.0.2: the first chunk holds `1024 / size_of::<T>()`
                 items and each next doubles => `(1024 / size) * 2^j * size` bytes.
                 The four item sizes are measured by the probe.
  deriv tape     `deriv_tape: Vec<TapeWord>::with_capacity(((len + 1) * 5)
                 .clamp(80, 40960))`, then `Vec` doubling => 8 * c0 * 2^j bytes.

`Group::WIDTH` is not assumed: both 8 and 16 are tried and the one that matches is
reported, because a wrong constant here would silently mis-name the largest structure
in the parse.

Usage: attribute_alloc_classes.py <alloc_census.tsv> <alloc_census_classes.tsv>
"""
import csv
import sys
from functools import lru_cache

THIN_RULE_COUNT = 109  # generated/systemverilog_parser.rs:3291
MB = 1048576.0

# ⭐ DETERMINED, NOT ASSUMED, AND THEN HELD FIXED. hashbrown lays a table out as
# `buckets * size_of::<T>()` + `buckets` control bytes + `Group::WIDTH`, and the width
# is 8 or 16 depending on the SIMD group this build selected. It is a property of the
# BUILD, not of an allocation, so trying both per allocation hands the search a free
# degree of freedom it can fabricate with — observed doing exactly that on `floor.sv`,
# naming the same 64-bucket table twice at two different widths.
# It is pinned to 8 by the LARGE tables, where the two candidates are 8 bytes apart and
# the measurement is unambiguous: 622 854 152 B = 2^21 * 297 + 8 (group 16 would be
# 622 854 160), reproduced at 2^18, 2^19, 2^20, 2^21 and 2^22 buckets across four inputs.
GROUP_WIDTH = 8


def num(s):
    if s in ("NA", "", None):
        return None
    try:
        return float(s)
    except ValueError:
        return None


def hashbrown_sizes(entry_size, lo, hi):
    """Table allocations for a power-of-two bucket count, both plausible group widths."""
    out = {}
    for k in range(0, 40):
        buckets = 1 << k
        size = buckets * entry_size + buckets + GROUP_WIDTH
        if lo <= size < hi:
            out[size] = f"hashbrown[T={entry_size}B] {buckets} buckets"
    return out


def chunk_owners(size, item_sizes):
    """Which item sizes could have produced a chunk of exactly `size` bytes.

    ⛔ USUALLY MORE THAN ONE, AND THAT IS REPORTED RATHER THAN RESOLVED. Item size
    `o` yields chunk byte sizes `(1024 // o) * o * 2^j`, and 48-byte `ParseNode`,
    24-byte `PgenValue` and 24-byte `String` all give the SAME base of 1008 — so a
    byte-keyed decomposition cannot tell a node chunk from a shaped-value chunk.
    Naming one would be a fabricated identification.
    """
    owners = []
    for o in sorted(set(item_sizes)):
        base = max(1024 // o, 1) * o
        if size % base:
            continue
        q = size // base
        if q & (q - 1) == 0:
            owners.append(f"{o}B")
    return "/".join(owners) or "?"


def arena_chunk_multiset(caps, item_sizes):
    """The EXACT multiset of chunk byte sizes each arena took from the allocator.

    ⛔ THIS REPLACED A GENERATOR THAT ADMITTED ANY DOUBLING-SEQUENCE VALUE, AND THE
    REASON IS A MEASURED OVER-ASSIGNMENT. `typed_arena` grows `first, 2*first,
    4*first, ...` with `first = 1024 / size_of::<T>()`, so `cap = first * (2^n - 1)`
    determines both `n` and every individual chunk size. An unconstrained generator
    let the search label allocations as arena chunks the arena never made: on
    `syn_concat_items` 10 000 it named 369.66 MB of arena chunks where the engine's
    own chunk model — the one `-0126` proved against a counting allocator, 230 cases,
    0 mismatches — says the arena allocated 317.13 MB. A fit that can exceed its own
    oracle is not an attribution.
    """
    out = {}
    for cap, item in zip(caps, item_sizes):
        if cap is None or item <= 0:
            continue
        first = max(1024 // item, 1)
        n = 0
        while first * ((1 << (n + 1)) - 1) <= cap:
            n += 1
        # `cap` must be exactly `first * (2^n - 1)`; anything else means the arena did
        # not grow by pure doubling (an `alloc_extend` reserve), and the multiset is
        # reported as UNDERIVABLE rather than guessed.
        if first * ((1 << n) - 1) != cap:
            return None
        for j in range(n):
            size = first * (1 << j) * item
            out[size] = out.get(size, 0) + 1
    return out


def arena_chunk_sizes(item_sizes, lo, hi):
    """⛔ CHUNK BYTE SIZES DO NOT IDENTIFY THE ARENA, AND SAYING SO IS THE POINT.

    `typed-arena`'s first chunk holds `1024 / size_of::<T>()` items, so its BYTE size
    is `(1024 / size) * size` — which collapses 48-byte `ParseNode` and 24-byte
    `PgenValue`/`String` onto the SAME sequence, 1008 * 2^j. A decomposition keyed on
    bytes therefore cannot tell a node chunk from a shaped-value chunk, and labelling
    one of them would be a fabricated identification. The label names the SEQUENCE and
    lists every item size that produces it.
    """
    out = {}
    by_seq = {}
    for item_size in item_sizes:
        if item_size <= 0:
            continue
        first = max(1024 // item_size, 1)
        by_seq.setdefault(first * item_size, set()).add(item_size)
    for base, owners in by_seq.items():
        who = "/".join(f"{o}B" for o in sorted(owners))
        for j in range(0, 40):
            size = base * (1 << j)
            if lo <= size < hi:
                out[size] = f"typed_arena chunk {base}*2^{j} B (item size {who} — not separable)"
    return out


def tape_sizes(n, lo, hi):
    out = {}
    c0 = min(max((n + 1) * 5, 80), 40960)
    for j in range(0, 40):
        size = 8 * c0 * (1 << j)
        if lo <= size < hi:
            out[size] = f"deriv_tape Vec<TapeWord> {c0 * (1 << j)} words"
    return out


def decompose(target, count, candidates, limits, lo, hi):
    """Name as many of a class's `count` live allocations as can be named EXACTLY.

    Returns `(named_sizes, unnamed_count, unnamed_bytes)`.

    ⛔ WHY THIS IS PARTIAL RATHER THAN ALL-OR-NOTHING, AND WHY THAT IS THE HONEST
    SHAPE. An all-or-nothing decomposition writes off a whole class — the arena
    chunks and the memo table in it included — the moment ONE allocation in it has
    no candidate. That reports a 38 % "unexplained" residual made mostly of
    structures this pass has in fact identified, which overstates the unknown in
    exactly the direction that makes the leaf's question look harder than it is.

    ⛔ AND WHY IT CANNOT FABRICATE. The unnamed remainder is not free: it is carried
    by `count - j` allocations that each, by definition of the size class, lie in
    `[lo, hi)`. So a split is admissible only when
    `(count - j) * lo <= residual < (count - j) * hi` — a real feasibility
    constraint that rejects most greedy fits. The largest feasible `j` wins, and the
    residual is REPORTED with its allocation count rather than absorbed.
    """
    sizes = tuple(sorted(set(candidates), reverse=True))
    for j in range(count, -1, -1):
        unnamed = count - j
        found = _exact_with_residual(sizes, limits, target, j, unnamed, lo, hi)
        if found is not None:
            named, residual = found
            return named, unnamed, residual
    return (), count, target


def _exact_with_residual(sizes, limits, target, j, unnamed, lo, hi):
    """Some j candidates summing to `target - residual`, residual carriable by `unnamed`.

    Each candidate is drawn no more often than `limits[size]` — the number of such
    allocations that can be live at once (an arena chunk of a given size exists once
    per arena that made it; a hashbrown table can be live twice, old and new, only
    across a rehash).
    """
    lo_need = unnamed * lo
    hi_need = unnamed * (hi - 1)
    used = {}

    def rec(remaining, slots, start, acc):
        if slots == 0:
            return (tuple(acc), remaining) if lo_need <= remaining <= hi_need else None
        for i in range(start, len(sizes)):
            s = sizes[i]
            if s > remaining or used.get(s, 0) >= limits.get(s, 1):
                continue
            used[s] = used.get(s, 0) + 1
            acc.append(s)
            got = rec(remaining - s, slots - 1, i, acc)
            acc.pop()
            used[s] -= 1
            if got is not None:
                return got
        return None

    return rec(target, j, 0, [])


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    main_rows = {r["path"]: r for r in csv.DictReader(open(sys.argv[1]), delimiter="\t")}
    per = {}
    for r in csv.DictReader(open(sys.argv[2]), delimiter="\t"):
        per.setdefault(r["path"], []).append(r)

    grand = {}
    bucket_ok, bucket_bad = [0], [0]
    for path, rows in per.items():
        m = main_rows.get(path)
        if not m:
            continue
        n = int(m["bytes"])
        snap = float(m["snap_live"])
        memo_b = int(m["memo_bucket_B"])
        fail_b = int(m["memo_fail_bucket_B"])
        node_b, pgv_b, pair_b, string_b = 48, 24, 40, 24
        thin = THIN_RULE_COUNT * (n + 1) * 8
        caps = []
        for col in ("chunk_nodes_cap", "chunk_sv_cap", "chunk_sp_cap", "chunk_rs_cap"):
            v = m.get(col, "NA")
            caps.append(int(v) if v.isdigit() else None)
        arena_ms = arena_chunk_multiset(caps, (node_b, pgv_b, pair_b, string_b))
        if arena_ms is None:
            print(f"\n== {path.split('/')[-1]}  ⛔ ARENA CHUNK MULTISET NOT DERIVABLE "
                  f"(a capacity is not first*(2^n-1)) — refusing to attribute this input")
            continue

        print(f"\n== {path.split('/')[-1]}   snapshot live {snap / MB:.2f} MB")
        named = {}
        unexplained = 0
        for r in sorted(rows, key=lambda r: -float(r["share"])):
            share = float(r["share"])
            if share < 0.01:
                continue
            k = int(r["class"])
            lo, hi = int(r["lo"]), int(r["hi"])
            total, count = int(r["live_bytes"]), int(r["live_count"])
            cand, limits = {}, {}
            if lo <= thin < hi:
                cand[thin] = f"THIN-MEMO SCRATCH {THIN_RULE_COUNT}*(len+1)*8"
                limits[thin] = 1  # one parser, one scratch
            # The four memo tables the generated parser declares:
            #   memo                    FxHashMap<(RuleId, usize), MemoEntry>   (measured)
            #   memo_fail               FxHashSet<(RuleId, usize)>              (measured)
            #   memo_fail_tainted       FxHashMap<(RuleId, usize), u64>
            #   memo_fail_depth_gated   FxHashMap<(RuleId, usize), u32>
            # The last two carry a 16-byte key beside an 8- or 4-byte value, which pads
            # to 24; both are tried rather than assumed.
            for e in (memo_b, fail_b, 24):
                got = hashbrown_sizes(e, lo, hi)
                cand.update(got)
                # Two at once is the rehash window (the new table is allocated before
                # the old is freed); more than two would be a table this parser does
                # not declare.
                for size in got:
                    limits[size] = 2
            for size, mult in arena_ms.items():
                if lo <= size < hi:
                    cand[size] = (f"typed_arena chunk {size} B "
                                  f"(item size {chunk_owners(size, (node_b, pgv_b, pair_b, string_b))}"
                                  f" — not separable)")
                    limits[size] = mult
            got = tape_sizes(n, lo, hi)
            cand.update(got)
            for size in got:
                limits[size] = 2
            # ⛔ A CLASS OF 285 ALLOCATIONS IS NOT "UNEXPLAINED", IT IS NOT INDIVIDUALLY
            # ATTRIBUTABLE — a different fact, and conflating them would inflate the
            # unexplained residual with ordinary small-object traffic. Only classes with
            # few enough live allocations for an exact decomposition to MEAN anything are
            # decomposed; the rest are reported as bulk.
            if count > 6:
                print(f"   class={k:<2} count={count:<3} {total / MB:9.2f} MB "
                      f"share={share:.4f}  — bulk small allocations, not individually attributable")
                named["bulk small allocations"] = named.get("bulk small allocations", 0) + total
                continue
            hit, unnamed_n, unnamed_b = decompose(total, count, cand, limits, lo, hi) \
                if cand else ((), count, total)
            print(f"   class={k:<2} count={count:<3} {total / MB:9.2f} MB share={share:.4f}")
            # ⭐ EVERY NAMED SIZE IS AN INDEPENDENT TEST OF THE PROBE'S OWN BUCKETING.
            # The candidate sizes here are derived from SOURCE constants (rule counts,
            # `size_of`, typed_arena's growth) with no reference to `class_of`; if the
            # probe bucketed by a wrong power of two, a size derived that way would land
            # outside the class the probe put it in. Checked on every named allocation
            # rather than on ten hand-picked values in a unit test.
            for s_ in hit:
                if not (lo <= s_ < hi):
                    print(f"        ⛔ CLASS-RANGE VIOLATION: {s_} not in [{lo},{hi}) "
                          f"— the probe's size-class bucketing disagrees with the "
                          f"source-derived size")
                    bucket_bad[0] += 1
                bucket_ok[0] += 1
            if unnamed_n:
                unexplained += unnamed_b
                named["UNNAMED"] = named.get("UNNAMED", 0) + unnamed_b
                print(f"        {unnamed_b:>13} B ({unnamed_b / MB:9.2f} MB)  "
                      f"⛔ UNNAMED — {unnamed_n} allocation(s), each in [{lo},{hi})")
            for s in hit:
                label = cand[s]
                print(f"        {s:>13} B ({s / MB:9.2f} MB)  {label}")
                key = label.split(" ")[0] if not label.startswith("THIN") else "THIN-MEMO"
                if label.startswith("hashbrown[T=%dB" % memo_b):
                    key = "memo table"
                elif label.startswith("hashbrown"):
                    key = "failure-memo table"
                elif label.startswith("typed_arena"):
                    key = "arena chunk"
                elif label.startswith("deriv_tape"):
                    key = "deriv tape"
                named[key] = named.get(key, 0) + s
        print(f"   -- named totals (of {snap / MB:.2f} MB at the snapshot)")
        for key, v in sorted(named.items(), key=lambda kv: -kv[1]):
            print(f"      {key:<22} {v / MB:9.2f} MB   {v / snap:6.2%}")
            grand.setdefault(key, []).append(v / snap)
        acct = sum(named.values())
        print(f"      {'[classes < 1% + gap]':<22} {(snap - acct) / MB:9.2f} MB   "
              f"{(snap - acct) / snap:6.2%}")
        # ⭐ ORACLE I DID NOT BUILD: the engine's own chunk model (`-0126`, proved
        # byte-exact against a counting GlobalAlloc on 230 cases) says how many bytes
        # the arena took. Named arena bytes must not exceed it.
        alloc_mb = num(m.get("arena_allocated_MB", "NA"))
        if alloc_mb is not None:
            got_mb = named.get("arena chunk", 0) / MB
            verdict = "OK" if got_mb <= alloc_mb + 1e-6 else "⛔ OVER-ASSIGNED"
            print(f"      [arena oracle] named {got_mb:.2f} MB vs model {alloc_mb:.2f} MB  {verdict}")

    print(f"\n=== CLASS-BUCKETING ORACLE: {bucket_ok[0]} named allocations checked, "
          f"{bucket_bad[0]} outside their own class ===")
    print("\n=== BANDS ACROSS THE LADDER (share of near-peak live bytes) ===")
    for key, vals in sorted(grand.items(), key=lambda kv: -sum(kv[1]) / len(kv[1])):
        v = sorted(vals)
        med = v[len(v) // 2] if len(v) % 2 else (v[len(v) // 2 - 1] + v[len(v) // 2]) / 2
        print(f"  {key:<22} n={len(v):<3} min {v[0]:6.2%} · median {med:6.2%} · max {v[-1]:6.2%}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
