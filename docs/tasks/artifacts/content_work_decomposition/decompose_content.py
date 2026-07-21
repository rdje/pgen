#!/usr/bin/env python3
"""PGEN-RGX-0078-0215 — content-work decomposition (read-only research slice).

Decomposes the remaining >=noise UN-decomposed populations of the `-0213`
re-price on the 1,004.4 ns floor — the "content work of USED state" the
`-0214` design law names as the only mass that can still move the geomean
(pooling refused x2, construction-elision refused, every in-target lane
sub-noise, spine_other mechanism-less):

    build_value       72.6611 ns   (to_shaped_value + cascade_build_* + ...)
    arena_alloc       69.4154 ns   (typed_arena alloc_extend monomorphs)
    semantic_runtime  49.3605 ns   (FactIndex / transaction shells / state)
    vec_growth        28.6025 ns   (finish_grow + grow_one family)

Combined 220.04 ns ~ 9.6x noise (22.871 ns).  Inputs are the SHA-banked
`-0213` capture of the floor probe `carrier48_8d392176` — same binary, same
raw PCs, same band manifest; custody re-asserted before any number is
emitted.  This slice owns NO product change.

Views (all mechanical and re-runnable):

1. Per-bucket per-symbol weighted-ns tables over the COMPLETE population
   (each must re-sum to its `-0213` bucket price — REFUSE otherwise).
2. Within-symbol hot-cluster tables for every symbol >= CLUSTER_MIN_NS:
   contiguous sampled-PC clusters (gap <= CLUSTER_GAP bytes) with per-band
   counts, weighted ns, and the annotated disassembly window — the honest
   evidence for any mechanism naming done in the adjudication.
3. Per-bucket TRANSPORT tally (operand-shape mechanical classification:
   q payload copies / sp frame spill / scalar state / alu / control / call)
   — re-sum gated per bucket.
4. Static caller census (bl-sites) for the hot callees, so every priced
   library monomorph is tied to its calling parser surface.

Weighted-ns convention byte-for-byte the `-0186`/`-0206`/`-0213` one: band
log-shares of the corpus geomean x the floor geomean over ALL stored samples
per band.  Noise span carried: 22.871 ns.
"""

from __future__ import annotations

import bisect
import re
import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
CAPTURE_DIR = ARTIFACT_DIR.parent / "carrier48_reprice"
sys.path[:0] = [str(CAPTURE_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_and_reprice as classify  # noqa: E402


FLOOR_GEOMEAN_NS = classify.FLOOR_GEOMEAN_NS
NOISE_NS = classify.NOISE_NS

# The -0213 role_reprice.txt bucket prices these views must re-sum to.
EXPECTED_BUCKET_NS = {
    "build_value": 72.6611,
    "arena_alloc": 69.4154,
    "semantic_runtime": 49.3605,
    "vec_growth": 28.6025,
}
RESUM_TOLERANCE_NS = 0.01
CLUSTER_MIN_NS = 3.0  # symbols above this get the within-symbol cluster view
CLUSTER_GAP = 16  # bytes; sampled PCs closer than this merge into one cluster
CLUSTER_TOP = 10  # clusters printed per symbol
WINDOW_BEFORE = 2  # extra instructions annotated either side of a cluster
WINDOW_AFTER = 2


def short(symbol: str) -> str:
    """Strip the mangling shell down to the recognizable method path."""
    text = symbol
    text = re.sub(r"^__ZN4pgen17generated_parsers5regex11RegexParser\d+", "", text)
    text = re.sub(r"^__ZN4pgen12ast_pipeline\d*", "pgen::", text)
    text = re.sub(r"^__ZN11typed_arena\d*", "typed_arena::", text)
    text = re.sub(r"^__ZN5alloc\d*", "alloc::", text)
    text = re.sub(r"(17h[0-9a-f]{16})E?$", r"#\1", text)
    text = text.replace("28_$u7b$$u7b$closure$u7d$$u7d$", "::{closure}")
    return text or symbol


def load_bands() -> tuple[dict[str, object], dict[str, list[int]], dict[str, int], dict[str, int]]:
    raw_custody = classify.load_raw_custody()
    manifest = capture.load_manifest()
    pcs_by_band: dict[str, list[int]] = {}
    slides: dict[str, int] = {}
    totals: dict[str, int] = {}
    for name in capture.BAND_NAMES:
        raw_path = CAPTURE_DIR / f"capture_band_{name}_raw_pcs.txt"
        if capture.sha256(raw_path) != raw_custody[name]:
            raise SystemExit(f"REFUSE: {name} raw SHA-256 mismatch vs banked custody")
        header, pcs = capture.parse_raw(raw_path)
        pcs_by_band[name] = pcs
        slides[name] = int(header["main_slide"], 16)
        totals[name] = len(pcs)
    return manifest, pcs_by_band, slides, totals


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    manifest, pcs_by_band, slides, totals = load_bands()
    weights = {name: manifest["bands"][name]["log_share"] for name in capture.BAND_NAMES}
    print(
        "weights: "
        + " ".join(f"{name}={weights[name]:.6f}" for name in capture.BAND_NAMES)
        + f" | floor={FLOOR_GEOMEAN_NS!r} ns | noise={NOISE_NS:.3f} ns"
    )

    symbols = capture.all_symbols()
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges(symbols)  # in-target pair: priced as
    # tier-1 role lanes in -0213, so EXCLUDED from every population bucket.
    symbol_addresses = [address for address, _ in symbols]
    symbol_names = [name for _, name in symbols]

    def symbol_at(address: int) -> str:
        index = bisect.bisect_right(symbol_addresses, address) - 1
        return symbol_names[index] if index >= 0 else "<pre-first-symbol>"

    def symbol_start(address: int) -> int:
        index = bisect.bisect_right(symbol_addresses, address) - 1
        return symbol_addresses[index] if index >= 0 else 0

    def weighted_ns(counts_by_band: dict[str, int]) -> float:
        share = sum(
            weights[name] * counts_by_band.get(name, 0) / totals[name]
            for name in capture.BAND_NAMES
        )
        return share * FLOOR_GEOMEAN_NS

    # ---- collect every sample of the four buckets, per symbol / PC / band ----
    per_symbol: dict[str, dict[str, Counter[str]]] = {b: {} for b in EXPECTED_BUCKET_NS}
    per_pc: dict[str, dict[str, dict[int, Counter[str]]]] = {
        b: {} for b in EXPECTED_BUCKET_NS
    }
    for name in capture.BAND_NAMES:
        slide = slides[name]
        for sampled_pc in pcs_by_band[name]:
            address = sampled_pc - slide
            if not (capture.TEXT_START <= address < capture.TEXT_END):
                continue
            if any(start <= address < end for start, end in target_ranges.values()):
                continue
            symbol = symbol_at(address)
            bucket = classify.bucket_for(symbol)
            if bucket not in EXPECTED_BUCKET_NS:
                continue
            per_symbol[bucket].setdefault(symbol, Counter())[name] += 1
            per_pc[bucket].setdefault(symbol, {}).setdefault(address, Counter())[name] += 1

    # ---- view 1: complete per-symbol tables (each re-sum gated) ----
    rows_by_bucket: dict[str, list[tuple[float, str, Counter[str]]]] = {}
    for bucket, expected in EXPECTED_BUCKET_NS.items():
        rows = sorted(
            (
                (weighted_ns(dict(counts)), symbol, counts)
                for symbol, counts in per_symbol[bucket].items()
            ),
            reverse=True,
        )
        rows_by_bucket[bucket] = rows
        total_ns = sum(row[0] for row in rows)
        print(
            f"\n=== VIEW 1 [{bucket}]: per-symbol (complete; {len(rows)} symbols; "
            f"total {total_ns:.4f} ns vs -0213 bucket {expected} ns) ==="
        )
        if abs(total_ns - expected) > RESUM_TOLERANCE_NS:
            raise SystemExit(
                f"REFUSE: {bucket} re-sum {total_ns:.4f} ns differs from the "
                f"-0213 bucket {expected} ns by more than {RESUM_TOLERANCE_NS}"
            )
        print("re-sum: PASS")
        for nanoseconds, symbol, counts in rows:
            bands = " ".join(f"{counts.get(name, 0):6d}" for name in capture.BAND_NAMES)
            print(f"{nanoseconds:9.4f}ns {bands}  {short(symbol)[:100]}")

    # ---- view 2: within-symbol hot clusters with annotated windows ----
    print(
        f"\n=== VIEW 2: within-symbol clusters (symbols >= {CLUSTER_MIN_NS} ns; "
        f"gap <= {CLUSTER_GAP} B; top {CLUSTER_TOP} clusters each) ==="
    )
    for bucket, rows in rows_by_bucket.items():
        for nanoseconds, symbol, _ in rows:
            if nanoseconds < CLUSTER_MIN_NS:
                continue
            pc_map = per_pc[bucket][symbol]
            start = symbol_start(next(iter(pc_map)))
            print(f"\n--- [{bucket}] {short(symbol)} [{nanoseconds:.4f} ns] ---")
            addresses = sorted(pc_map)
            clusters: list[list[int]] = [[addresses[0]]]
            for address in addresses[1:]:
                if address - clusters[-1][-1] <= CLUSTER_GAP:
                    clusters[-1].append(address)
                else:
                    clusters.append([address])
            priced = sorted(
                (
                    (
                        weighted_ns(
                            {
                                name: sum(
                                    pc_map[address].get(name, 0) for address in cluster
                                )
                                for name in capture.BAND_NAMES
                            }
                        ),
                        cluster,
                    )
                    for cluster in clusters
                ),
                reverse=True,
            )
            for cluster_ns, cluster in priced[:CLUSTER_TOP]:
                band_counts = {
                    name: sum(pc_map[address].get(name, 0) for address in cluster)
                    for name in capture.BAND_NAMES
                }
                bands = " ".join(f"{band_counts[name]}" for name in capture.BAND_NAMES)
                print(
                    f"  cluster +{cluster[0] - start:#06x}..+{cluster[-1] - start:#06x} "
                    f"{cluster_ns:8.4f} ns  bands[{bands}]"
                )
                for address in range(
                    cluster[0] - WINDOW_BEFORE * 4,
                    cluster[-1] + (WINDOW_AFTER + 1) * 4,
                    4,
                ):
                    instruction = code.get(address)
                    if instruction is None:
                        continue
                    count = sum(pc_map.get(address, Counter()).values())
                    marker = f"{count:5d}*" if count else "      "
                    print(
                        f"    {marker} +{address - start:#06x} "
                        f"{instruction[0]} {instruction[1]}".rstrip()
                    )

    # ---- view 3: per-bucket TRANSPORT tally (mechanical, operand-shape) ----
    Q_REG = re.compile(r"\bq\d+\b")
    SP_REL = re.compile(r"\[(?:sp|x29)[,\]]")
    print("\n=== VIEW 3: TRANSPORT tally per bucket (mechanical; re-sum gated) ===")
    for bucket, expected in EXPECTED_BUCKET_NS.items():
        transport: dict[str, Counter[str]] = {}
        for symbol, pc_map in per_pc[bucket].items():
            for address, band_counter in pc_map.items():
                mnemonic, operands = code[address]
                klass = capture.instruction_class(mnemonic)
                if klass == "memory":
                    if Q_REG.search(operands):
                        key = (
                            "q_payload_copy (sp)"
                            if SP_REL.search(operands)
                            else "q_payload_copy (heap/arena)"
                        )
                    elif SP_REL.search(operands):
                        key = "x_frame_spill (sp/fp)"
                    else:
                        key = "x_scalar_state_or_elem"
                elif klass == "call":
                    key = "call"
                elif klass == "control":
                    key = "control"
                else:
                    key = "other_alu"
                transport.setdefault(key, Counter()).update(band_counter)
        rows = sorted(
            ((weighted_ns(dict(counts)), key) for key, counts in transport.items()),
            reverse=True,
        )
        total = 0.0
        print(f"\n-- {bucket} --")
        for nanoseconds, key in rows:
            total += nanoseconds
            print(f"{nanoseconds:9.4f}ns  {key}")
        print(f"{total:9.4f}ns  TOTAL (bucket price)")
        if abs(total - expected) > RESUM_TOLERANCE_NS:
            raise SystemExit(f"REFUSE: {bucket} transport view does not re-sum")

    # ---- view 4: static caller census for the hot priced callees ----
    print("\n=== VIEW 4: static bl-site caller census for the hot callees ===")
    callee_needles = (
        "alloc_extend17h88670e1bf10cf0ff",
        "alloc_extend17h0f20dcea3f658c24",
        "alloc_extend17h916724f9b9255e27",
        "to_shaped_value",
        "cascade_build_piece",
        "cascade_build_alternative",
        "cascade_build_quantifier",
        "FactIndex6insert",
        "SemanticRuntimeState3new",
        "with_semantic_runtime_rule_transaction",
        "extract_delta_since_slow",
        "decode_event",
        "finish_grow",
        "grow_one",
    )
    for needle in callee_needles:
        callers: Counter[str] = Counter()
        for address, instruction in code.items():
            if instruction[0] == "bl" and needle in instruction[1]:
                callers[symbol_at(address)] += 1
        print(f"-- bl-sites for *{needle}*:")
        if not callers:
            print("   (none — fully inlined or reached indirectly)")
        for caller, count in callers.most_common(12):
            print(f"   {count:4d} {short(caller)[:105]}")

    print(
        f"\nnoise floor: {NOISE_NS:.3f} ns — a solo mechanism must clear this with "
        "margin; the adjudication (step_result.md + the tree leaf) names the "
        "mechanisms from the annotated windows above."
    )


if __name__ == "__main__":
    main()
