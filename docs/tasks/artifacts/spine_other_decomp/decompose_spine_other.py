#!/usr/bin/env python3
"""PGEN-RGX-0078-0211 — spine_other decomposition (read-only research slice).

Decomposes the `-0208` spine_other population (203.75 ns ≈ 8.6x noise on the
1037.804231341058 ns floor) WITHOUT a new capture: the `-0208` three-band raw
PC captures are reused under full custody (probe SHA + the three banked raw
SHAs + band manifest asserted before any number is emitted).  Legitimate
because the floor, the probe, and all 11 generated artifacts are UNCHANGED
since that capture (`-0209`/`-0210` were SHA-verified byte-restores).

Three views, all mechanical and re-runnable:

1. Per-symbol weighted-ns table over the COMPLETE spine_other population
   (must re-sum to the `-0208` bucket price — REFUSE otherwise).
2. Within-symbol hot-cluster tables for every symbol >= CLUSTER_MIN_NS:
   contiguous sampled-PC clusters (gap <= CLUSTER_GAP bytes) with per-band
   counts, weighted ns, and the annotated disassembly window — the honest
   evidence for any mechanism naming done in the adjudication.
3. Cross-symbol mechanism tallies:
   - `call:<callee>` — samples ON a bl/blr instruction (call overhead
     neighborhoods, attributed by callee symbol);
   - `role:<group>` / `off:<hex>` — memory samples whose operand is a
     [self_reg, #offset] access, where self_reg is the prologue-provable
     callee-saved copy of x0 (labeled HEURISTIC: the prologue proof is
     `mov xN, x0` within the first PROLOGUE_WINDOW instructions; closures
     and re-homed registers stay unattributed rather than guessed);
   - instruction-class residue (memory/control/other) per symbol family
     (parse_* / cascade_match_* / scan_* / resolve_* / memoized_call /
     other) — the coarse "what kind of work" split.

The weighted-ns convention is byte-for-byte the `-0186`/`-0208` one: band
log-shares of the corpus geomean x the floor geomean, over ALL stored samples
per band.  Noise span carried: 23.657 ns.  This slice owns NO product change.
"""

from __future__ import annotations

import bisect
import json
import re
import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
FXSTORE_DIR = ARTIFACT_DIR.parent / "fxstore_reprice"
sys.path[:0] = [str(FXSTORE_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_and_reprice as classify  # noqa: E402


FLOOR_GEOMEAN_NS = classify.FLOOR_GEOMEAN_NS
NOISE_NS = classify.NOISE_NS
EXPECTED_BUCKET_NS = 203.7475  # the -0208 spine_other price this must re-sum to
RESUM_TOLERANCE_NS = 0.01
CLUSTER_MIN_NS = 3.0  # symbols above this get the within-symbol cluster view
CLUSTER_GAP = 16  # bytes; sampled PCs closer than this merge into one cluster
CLUSTER_TOP = 10  # clusters printed per symbol
WINDOW_BEFORE = 2  # extra instructions annotated either side of a cluster
WINDOW_AFTER = 2
PROLOGUE_WINDOW = 12  # instructions scanned for the `mov xN, x0` self-reg proof


def short(symbol: str) -> str:
    """Strip the mangling shell down to the recognizable method path."""
    text = symbol
    text = re.sub(r"^__ZN4pgen17generated_parsers5regex11RegexParser\d+", "", text)
    text = re.sub(r"17h[0-9a-f]{16}E?$", "", text)
    text = text.replace("28_$u7b$$u7b$closure$u7d$$u7d$", "::{closure}")
    return text or symbol


def family(symbol: str) -> str:
    name = short(symbol)
    for prefix in (
        "parse_",
        "cascade_match_",
        "cascade_build",
        "cascade_",
        "scan_",
        "resolve_",
        "memoized_call",
        "try_parse_bare",
        "lex_",
    ):
        if name.startswith(prefix):
            return prefix.rstrip("_") if prefix.endswith("_") else prefix
    return "other"


def load_bands() -> tuple[dict[str, object], dict[str, list[int]], dict[str, int], dict[str, int]]:
    raw_custody = classify.load_raw_custody()
    manifest = capture.load_manifest()
    pcs_by_band: dict[str, list[int]] = {}
    slides: dict[str, int] = {}
    totals: dict[str, int] = {}
    for name in capture.BAND_NAMES:
        raw_path = FXSTORE_DIR / f"capture_band_{name}_raw_pcs.txt"
        if capture.sha256(raw_path) != raw_custody[name]:
            raise SystemExit(f"REFUSE: {name} raw SHA-256 mismatch vs banked custody")
        header, pcs = capture.parse_raw(raw_path)
        pcs_by_band[name] = pcs
        slides[name] = int(header["main_slide"], 16)
        totals[name] = len(pcs)
    return manifest, pcs_by_band, slides, totals


def find_self_reg(code: dict[int, tuple[str, str]], start: int) -> str | None:
    """The prologue-provable callee-saved copy of x0, or None (honest refusal)."""
    for step in range(PROLOGUE_WINDOW):
        instruction = code.get(start + step * 4)
        if instruction is None:
            return None
        mnemonic, operands = instruction
        if mnemonic == "mov":
            parts = [part.strip() for part in operands.split(",")]
            if len(parts) == 2 and parts[1] == "x0" and re.fullmatch(r"x1\d|x2[0-8]", parts[0]):
                return parts[0]
        if mnemonic in {"bl", "blr", "b", "ret"}:
            return None
    return None


def callee_of(operands: str, symbol_at) -> str:
    match = re.search(r"0x[0-9a-f]+", operands)
    if match is None:
        # otool usually resolves bl targets to symbol names directly.
        name = operands.strip().split()[-1] if operands.strip() else "<indirect>"
        return short(name) if "RegexParser" in name else name[:80]
    return short(symbol_at(int(match.group(0), 16)))


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
    target_ranges = capture.symbol_ranges(symbols)  # the -0208 in-target pair:
    # priced separately as tier-1 lanes there, so EXCLUDED from spine_other.
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

    # ---- collect every spine_other sample, per symbol / per PC / per band ----
    per_symbol: dict[str, Counter[str]] = {}
    per_pc: dict[str, dict[int, Counter[str]]] = {}
    for name in capture.BAND_NAMES:
        slide = slides[name]
        for sampled_pc in pcs_by_band[name]:
            address = sampled_pc - slide
            if not (capture.TEXT_START <= address < capture.TEXT_END):
                continue
            if any(start <= address < end for start, end in target_ranges.values()):
                continue
            symbol = symbol_at(address)
            if classify.bucket_for(symbol) != "spine_other":
                continue
            per_symbol.setdefault(symbol, Counter())[name] += 1
            per_pc.setdefault(symbol, {}).setdefault(address, Counter())[name] += 1

    # ---- view 1: the complete per-symbol table (must re-sum) ----
    rows = sorted(
        (
            (weighted_ns(dict(counts)), symbol, counts)
            for symbol, counts in per_symbol.items()
        ),
        reverse=True,
    )
    total_ns = sum(row[0] for row in rows)
    print(
        f"\n=== VIEW 1: spine_other per-symbol (complete; {len(rows)} symbols; "
        f"total {total_ns:.4f} ns vs -0208 bucket {EXPECTED_BUCKET_NS} ns) ==="
    )
    if abs(total_ns - EXPECTED_BUCKET_NS) > RESUM_TOLERANCE_NS:
        raise SystemExit(
            f"REFUSE: spine_other re-sum {total_ns:.4f} ns differs from the "
            f"-0208 bucket {EXPECTED_BUCKET_NS} ns by more than {RESUM_TOLERANCE_NS}"
        )
    print("re-sum: PASS")
    for nanoseconds, symbol, counts in rows:
        bands = " ".join(f"{counts.get(name, 0):6d}" for name in capture.BAND_NAMES)
        print(f"{nanoseconds:9.4f}ns {bands}  {short(symbol)[:90]}")

    # ---- view 2: within-symbol hot clusters with annotated windows ----
    print(
        f"\n=== VIEW 2: within-symbol clusters (symbols >= {CLUSTER_MIN_NS} ns; "
        f"gap <= {CLUSTER_GAP} B; top {CLUSTER_TOP} clusters each) ==="
    )
    for nanoseconds, symbol, _ in rows:
        if nanoseconds < CLUSTER_MIN_NS:
            continue
        start = symbol_start(next(iter(per_pc[symbol])))
        self_reg = find_self_reg(code, start)
        print(
            f"\n--- {short(symbol)} [{nanoseconds:.4f} ns] "
            f"self_reg={'UNPROVEN' if self_reg is None else self_reg} ---"
        )
        addresses = sorted(per_pc[symbol])
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
                                per_pc[symbol][address].get(name, 0)
                                for address in cluster
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
                name: sum(per_pc[symbol][address].get(name, 0) for address in cluster)
                for name in capture.BAND_NAMES
            }
            bands = " ".join(f"{band_counts[name]}" for name in capture.BAND_NAMES)
            print(
                f"  cluster +{cluster[0] - start:#06x}..+{cluster[-1] - start:#06x} "
                f"{cluster_ns:8.4f} ns  bands[{bands}]"
            )
            for address in range(
                cluster[0] - WINDOW_BEFORE * 4, cluster[-1] + (WINDOW_AFTER + 1) * 4, 4
            ):
                instruction = code.get(address)
                if instruction is None:
                    continue
                count = sum(per_pc[symbol].get(address, Counter()).values())
                marker = f"{count:5d}*" if count else "      "
                print(
                    f"    {marker} +{address - start:#06x} "
                    f"{instruction[0]} {instruction[1]}".rstrip()
                )

    # ---- view 3: cross-symbol mechanism tallies ----
    call_ns: Counter[str] = Counter()
    role_counts: dict[str, Counter[str]] = {}
    class_by_family: dict[tuple[str, str], Counter[str]] = {}
    role_qualified_symbols = 0
    role_unproven_symbols = 0
    for symbol, pc_map in per_pc.items():
        start = symbol_start(next(iter(pc_map)))
        self_reg = find_self_reg(code, start)
        if self_reg is None:
            role_unproven_symbols += 1
        else:
            role_qualified_symbols += 1
        fam = family(symbol)
        for address, band_counter in pc_map.items():
            mnemonic, operands = code[address]
            klass = capture.instruction_class(mnemonic)
            if klass == "call":
                call_ns[callee_of(operands, symbol_at)] += 0  # placeholder key
                call_key = callee_of(operands, symbol_at)
                role_counts.setdefault(f"call:{call_key}", Counter()).update(band_counter)
                continue
            if self_reg is not None and klass == "memory":
                pattern = re.compile(
                    r"\[" + self_reg + r"(?:, #(?P<offset>0x[0-9a-f]+|\d+))?\]"
                )
                matches = list(pattern.finditer(operands))
                if len(matches) == 1:
                    raw = matches[0].group("offset")
                    offset = (
                        0
                        if raw is None
                        else int(raw, 16) if raw.startswith("0x") else int(raw)
                    )
                    known = {
                        group
                        for group, offsets in classify.GROUP_OFFSETS.items()
                        if offset in offsets
                    }
                    key = (
                        f"role:{next(iter(known))}"
                        if len(known) == 1
                        else f"off:{offset:#x}"
                    )
                    role_counts.setdefault(key, Counter()).update(band_counter)
                    continue
            class_by_family.setdefault((fam, klass), Counter()).update(band_counter)

    print(
        f"\n=== VIEW 3a: HEURISTIC self-reg role/offset + call tallies "
        f"(prologue-proven symbols: {role_qualified_symbols}; "
        f"UNPROVEN (unattributed, honest): {role_unproven_symbols}) ==="
    )
    tallies = sorted(
        ((weighted_ns(dict(counts)), key) for key, counts in role_counts.items()),
        reverse=True,
    )
    for nanoseconds, key in tallies:
        if nanoseconds < 0.25:
            continue
        print(f"{nanoseconds:9.4f}ns  {key[:100]}")

    print("\n=== VIEW 3b: instruction-class residue by symbol family ===")
    fam_rows = sorted(
        (
            (weighted_ns(dict(counts)), fam, klass)
            for (fam, klass), counts in class_by_family.items()
        ),
        reverse=True,
    )
    for nanoseconds, fam, klass in fam_rows:
        if nanoseconds < 0.25:
            continue
        print(f"{nanoseconds:9.4f}ns  {fam:16} {klass}")

    # ---- view 3c: the TRANSPORT tally (mechanical, operand-shape based) ----
    # q-register memory ops = 16-byte SIMD payload copies — on this parser these
    # are the ParseNode/ParseResult value-transport chains (checkpoint spills
    # use x-pair stp; state loads use [reg,#off] scalars), so the q tally is a
    # mechanical LOWER BOUND on result-payload transport.  sp-relative x-ops =
    # frame spill/reload ceremony (incl. prologue/epilogue pair saves).  The
    # remainder splits into non-sp scalar memory (state/element access) and
    # non-memory classes.
    Q_REG = re.compile(r"\bq\d+\b")
    SP_REL = re.compile(r"\[(?:sp|x29)[,\]]")
    transport: dict[str, Counter[str]] = {}
    for symbol, pc_map in per_pc.items():
        for address, band_counter in pc_map.items():
            mnemonic, operands = code[address]
            klass = capture.instruction_class(mnemonic)
            if klass == "memory":
                if Q_REG.search(operands):
                    key = "q_payload_copy (sp)" if SP_REL.search(operands) else "q_payload_copy (heap/arena)"
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

    print("\n=== VIEW 3c: TRANSPORT tally over ALL spine_other samples (mechanical) ===")
    transport_rows = sorted(
        ((weighted_ns(dict(counts)), key) for key, counts in transport.items()),
        reverse=True,
    )
    view3c_total = 0.0
    for nanoseconds, key in transport_rows:
        view3c_total += nanoseconds
        print(f"{nanoseconds:9.4f}ns  {key}")
    print(f"{view3c_total:9.4f}ns  TOTAL (must equal the bucket price)")
    if abs(view3c_total - EXPECTED_BUCKET_NS) > RESUM_TOLERANCE_NS:
        raise SystemExit("REFUSE: view 3c does not re-sum to the bucket price")

    print(
        f"\nnoise floor: {NOISE_NS:.3f} ns — a solo mechanism must clear this with "
        "margin; the adjudication (step_result.md + the tree leaf) names the "
        "mechanisms from the annotated windows above."
    )


if __name__ == "__main__":
    main()
