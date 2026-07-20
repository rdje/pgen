#!/usr/bin/env python3
"""Exact lower-bound attribution of -0182 residual stack-memory samples.

The classifier deliberately recognizes only machine structures whose carrier
semantics can be proved from the custody-pinned probe.  Every other residual
stack-memory sample remains explicit ``unmatched``; an address base or a nearby
source line is never promoted to a mechanism by itself.
"""

from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
CAPTURE_DIR = ARTIFACT_DIR.parent / "three_band_pc_capture"
sys.path.insert(0, str(CAPTURE_DIR))

import analyze_capture as capture  # noqa: E402
import classify_mechanisms as known  # noqa: E402


EXPECTED_LAYOUT = """\
ParseError                       size= 80 align= 8 needs_drop=true
ParseResult<()> (Result ABI)     size= 80 align= 8 needs_drop=true
SemanticRuntimeCheckpoint        size= 56 align= 8 needs_drop=false
SemanticRuntimeDelta             size=168 align= 8 needs_drop=true
Option<SemanticRuntimeDelta>     size=168 align= 8 needs_drop=true
"""

PROFILE_WEIGHTS = {
    "sub1us": 0.380,
    "1to2p5": 0.353,
    "2p5to20": 0.260,
}


@dataclass(frozen=True)
class Region:
    category: str
    target: str
    start: int
    end: int
    role: str


# Stack-memory instructions in the ABI save/restore sequences.  The stack
# pointer adjustment is outside these half-open regions and is not part of the
# stack-memory population classified here.
ABI_REGIONS = (
    Region("abi_frame", "piece", 0x100058D20, 0x100058D38, "prologue"),
    Region("abi_frame", "piece", 0x10005CD98, 0x10005CDB0, "epilogue"),
    Region("abi_frame", "atom_closure", 0x10006ACF4, 0x10006AD0C, "prologue"),
    Region("abi_frame", "atom_closure", 0x10007C824, 0x10007C83C, "epilogue"),
)


# Each range starts at Option<SemanticRuntimeDelta>'s discriminant load and
# ends immediately after the apply_delta call.  The first four are direct
# in-place gates.  The last includes the exact 168-byte winner-delta movement
# into an apply buffer before the same call.
DELTA_REGIONS = (
    Region("semantic_delta", "piece", 0x10005CAC0, 0x10005CAFC, "apply_gate"),
    Region("semantic_delta", "atom_closure", 0x100070F48, 0x100070F8C, "apply_gate"),
    Region("semantic_delta", "atom_closure", 0x100074C2C, 0x100074C70, "apply_gate"),
    Region("semantic_delta", "atom_closure", 0x10007C088, 0x10007C0C4, "apply_gate"),
    Region("semantic_delta", "atom_closure", 0x10007C72C, 0x10007C7D4, "winner_move_apply"),
)


# The final 80-byte ParseResult<()> sret copy in cascade_match_piece.  A 32-byte
# indexed error carrier still needs the two prefix loads; only the three tail
# loads are an exact width-removal subset of the already held G1-B mechanism.
RESULT_PREFIX_PCS = frozenset((0x10005CD88, 0x10005CD8C))
RESULT_WIDTH_TAIL_PCS = frozenset((0x10005CD74, 0x10005CD78, 0x10005CD80))


STACK_ADDRESS = re.compile(r"\[(?P<base>sp|x29)(?:, #(?P<offset>-?0x[0-9a-f]+))?\]")


def is_stack_memory(instruction: tuple[str, str]) -> bool:
    mnemonic, operands = instruction
    return (
        capture.instruction_class(mnemonic) == "memory"
        and ("[sp" in operands or "[x29" in operands)
    )


def stack_write_words(mnemonic: str, operands: str) -> tuple[str, set[int]] | None:
    if mnemonic not in {"str", "stur", "stp"}:
        return None
    match = STACK_ADDRESS.search(operands)
    if match is None:
        return None
    registers = operands[: match.start()].rstrip(", ").split(",")
    if not registers or any(not register.strip().startswith("x") for register in registers):
        return None
    offset = int(match.group("offset") or "0", 16)
    width_words = 2 if mnemonic == "stp" else 1
    return match.group("base"), {offset + 8 * index for index in range(width_words)}


def checkpoint_store_pcs(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> tuple[set[int], list[tuple[str, int, int, str, int]]]:
    """Find the exact seven-word checkpoint materialization before each entry trace.

    Source order is ``checkpoint(); if trace_enabled()``.  In the probe, every
    inlined entry trace owns one ``Starting speculative parse`` literal and one
    immediately preceding logger gate.  Inside the 16-instruction pre-gate
    window there must be exactly one base-register-local contiguous 56-byte
    stack-write span.  That span is the Copy checkpoint, independently fixed
    at seven words by the banked type-layout probe.
    """

    all_pcs: set[int] = set()
    sites: list[tuple[str, int, int, str, int]] = []
    for target, (target_start, target_end) in target_ranges.items():
        literals = [
            address
            for address in range(target_start, target_end, 4)
            if "Starting speculative parse" in code.get(address, ("", ""))[1]
        ]
        for literal in literals:
            guards = [
                address
                for address in range(max(target_start, literal - 128), literal, 4)
                if code.get(address, ("", ""))[0] == "ldrb"
                and "[x20, #0x504]" in code[address][1]
            ]
            if len(guards) != 1:
                raise SystemExit(
                    f"REFUSE: entry-trace literal {literal:#x} has {len(guards)} nearby guards"
                )
            guard = guards[0]
            writes: list[tuple[int, str, set[int]]] = []
            for address in range(guard - 64, guard, 4):
                decoded = stack_write_words(*code.get(address, ("", "")))
                if decoded is not None:
                    base, words = decoded
                    writes.append((address, base, words))

            candidates: list[tuple[str, int, set[int]]] = []
            for base in ("sp", "x29"):
                covered = set().union(
                    *(words for _, write_base, words in writes if write_base == base),
                    set(),
                )
                for start in covered:
                    expected = {start + 8 * index for index in range(7)}
                    if expected <= covered:
                        candidates.append((base, start, expected))
            # A saved position/length word can be adjacent immediately below
            # the checkpoint (one site produces an eight-word covered run).
            # Source order fixes the checkpoint as the trailing seven words,
            # whose final field is the last pre-gate write.
            if candidates:
                latest_start = max(start for _, start, _ in candidates)
                candidates = [item for item in candidates if item[1] == latest_start]
            if len(candidates) != 1:
                raise SystemExit(
                    f"REFUSE: guard {guard:#x} has {len(candidates)} checkpoint spans "
                    f"{[(base, start) for base, start, _ in candidates]}"
                )
            base, slot_start, words = candidates[0]
            site_pcs = {
                address
                for address, write_base, written_words in writes
                if write_base == base and written_words <= words
            }
            written = set().union(
                *(written_words for address, write_base, written_words in writes
                  if address in site_pcs and write_base == base),
                set(),
            )
            if written != words:
                raise SystemExit(f"REFUSE: checkpoint words do not re-sum at {guard:#x}")
            if all_pcs & site_pcs:
                raise SystemExit(f"REFUSE: checkpoint instruction overlap at {guard:#x}")
            all_pcs.update(site_pcs)
            sites.append((target, guard, literal, base, slot_start))

    if len(sites) != 39:
        raise SystemExit(f"REFUSE: expected 39 inlined checkpoints, found {len(sites)}")
    return all_pcs, sites


def qualify_regions(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> None:
    occupied: dict[int, Region] = {}
    for region in ABI_REGIONS + DELTA_REGIONS:
        target_start, target_end = target_ranges[region.target]
        if not (target_start <= region.start < region.end <= target_end):
            raise SystemExit(f"REFUSE: region outside target: {region}")
        if region.start % 4 or region.end % 4:
            raise SystemExit(f"REFUSE: unaligned region: {region}")
        for address in range(region.start, region.end, 4):
            if address not in code:
                raise SystemExit(f"REFUSE: disassembly hole at {address:#x}")
            if address in occupied:
                raise SystemExit(f"REFUSE: region overlap at {address:#x}")
            occupied[address] = region

    for region in ABI_REGIONS:
        mnemonics = [code[address][0] for address in range(region.start, region.end, 4)]
        if mnemonics != ["stp"] * 6 and mnemonics != ["ldp"] * 6:
            raise SystemExit(f"REFUSE: ABI fingerprint differs: {region}")

    for region in DELTA_REGIONS:
        text = "\n".join(
            f"{code[address][0]} {code[address][1]}"
            for address in range(region.start, region.end, 4)
        )
        if "#-0x8000000000000000" not in text:
            raise SystemExit(f"REFUSE: delta discriminant fingerprint differs: {region}")
        if "SemanticRuntimeState11apply_delta" not in code[region.end - 4][1]:
            raise SystemExit(f"REFUSE: delta apply terminus differs: {region}")

    result_text = "\n".join(
        f"{code[address][0]} {code[address][1]}"
        for address in range(0x10005CD74, 0x10005CD94, 4)
    )
    required = (
        "[sp, #0x410]",
        "[sp, #0x420]",
        "[sp, #0x430]",
        "[sp, #0x440]",
        "[sp, #0x450]",
    )
    if any(token not in result_text for token in required):
        raise SystemExit("REFUSE: final 80-byte ParseResult copy fingerprint differs")


def region_owner(target: str, address: int) -> Region | None:
    matches = [
        region
        for region in ABI_REGIONS + DELTA_REGIONS
        if region.target == target and region.start <= address < region.end
    ]
    if len(matches) > 1:
        raise SystemExit(f"REFUSE: runtime region overlap at {address:#x}")
    return matches[0] if matches else None


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    layout = (ARTIFACT_DIR / "carrier_layout_probe.txt").read_text()
    if layout != EXPECTED_LAYOUT:
        raise SystemExit("REFUSE: carrier layout evidence differs")

    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    qualify_regions(code, target_ranges)
    checkpoint_pcs, checkpoint_sites = checkpoint_store_pcs(code, target_ranges)

    static_categories: dict[int, str] = {}
    for address in checkpoint_pcs:
        static_categories[address] = "semantic_checkpoint"
    for address in RESULT_PREFIX_PCS:
        static_categories[address] = "parse_result_prefix"
    for address in RESULT_WIDTH_TAIL_PCS:
        static_categories[address] = "parse_result_width_tail"
    for address, category in static_categories.items():
        target = next(
            (label for label, (start, end) in target_ranges.items() if start <= address < end),
            None,
        )
        if target is None or known.owner(address) is not None or region_owner(target, address):
            raise SystemExit(f"REFUSE: static carrier overlap/outside target at {address:#x}")

    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print("raw custody: " + " ".join(f"{band}={known.RAW_SHA256[band]}" for band in PROFILE_WEIGHTS))
    print("layout custody: PASS ParseResult=80B checkpoint=56B/Copy delta=168B")
    print(
        "machine qualification: PASS "
        f"checkpoint_sites={len(checkpoint_sites)} abi_regions={len(ABI_REGIONS)} "
        f"delta_regions={len(DELTA_REGIONS)} result_stack_loads=5 overlap=0"
    )
    print(
        "trace-spill hypothesis: REFUTED for the observed seven-word blocks — "
        "every Starting-speculation site is preceded by exactly one contiguous "
        "7-word checkpoint materialization"
    )
    print("checkpoint sites (target guard literal base slot_start):")
    for target, guard, literal, base, slot_start in checkpoint_sites:
        print(f"site {target:12} {guard:#011x} {literal:#011x} {base:3} {slot_start:#x}")

    categories = (
        "abi_frame",
        "semantic_checkpoint",
        "semantic_delta",
        "parse_result_prefix",
        "parse_result_width_tail",
        "unmatched",
    )
    all_counts: dict[str, Counter[str]] = {}
    raw_totals: dict[str, int] = {}
    for band, *_ in capture.BANDS:
        raw_path = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw_path) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: {band} raw SHA-256 mismatch")
        header, pcs = capture.parse_raw(raw_path)
        raw_totals[band] = len(pcs)
        slide = int(header["main_slide"], 16)
        counts: Counter[str] = Counter()
        for sampled_pc in pcs:
            address = sampled_pc - slide
            target = next(
                (
                    label
                    for label, (start, end) in target_ranges.items()
                    if start <= address < end
                ),
                None,
            )
            if target is None or known.owner(address) is not None:
                continue
            if not is_stack_memory(code[address]):
                continue
            category = static_categories.get(address)
            if category is None:
                region = region_owner(target, address)
                category = region.category if region else "unmatched"
            counts[category] += 1

        total = sum(counts.values())
        expected = {"sub1us": 902, "1to2p5": 848, "2p5to20": 987}[band]
        if total != expected or sum(counts[category] for category in categories) != expected:
            raise SystemExit(f"REFUSE: {band} stack re-sum {total} != {expected}")
        all_counts[band] = counts
        print(f"\n=== BAND {band} ===")
        for category in categories:
            value = counts[category]
            print(f"carrier {category:25} {value:5d} stack={100.0 * value / total:8.4f}%")
        print("resum " + "+".join(str(counts[category]) for category in categories) + f"={total} PASS")

    tail_share = sum(
        PROFILE_WEIGHTS[band]
        * all_counts[band]["parse_result_width_tail"]
        / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    print("\nPROFILE CROSS-CHECK")
    print(
        f"parse_result_width_tail weighted_all_sample_share={100.0 * tail_share:.6f}% "
        "(exact G1-B overlap; not additive)"
    )
    print("NEW NANOSECONDS: 0.000 ns — no new non-overlapping removable population")
    print("STACK CARRIER ATTRIBUTION: PASS (exact lower bounds; zero overlap; full re-sum)")


if __name__ == "__main__":
    main()
