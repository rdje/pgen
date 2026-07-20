#!/usr/bin/env python3
"""Price the five preserved-probe thin-memo lookup mechanisms exactly.

The complete ranges are the inlined ``FxHashMap::get`` operations only.  Hit
validation/replay, stale-entry removal, and success segment-copy/insert work
are outside them.  The replacement-aware subset also retains a deliberately
minimal proxy for a direct position-indexed lookup: the three parser-relative
table-state loads and the initial absent/present branch at each site.  This is
an optimistic proxy because a real direct lookup additionally needs a slot
load and dense-entry address computation.
"""

from __future__ import annotations

import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
BARE_DIR = TASK_ARTIFACTS / "bare_diagnostic_expansion"
sys.path[:0] = [str(CAPTURE_DIR), str(BARE_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_bare_diagnostics as bare  # noqa: E402
import classify_mechanisms as known  # noqa: E402


PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
FLOOR_NS = 1263.4
HELD_BUNDLE_NS = 92.427632
NOISE_NS = 28.8
CAPTURE_LOW = 0.30


@dataclass(frozen=True)
class LookupSite:
    label: str
    target: str
    start: int
    end: int
    rule_id: int
    position_register: str


# Half-open inlined FxHashMap::get regions.  Each starts at the map occupancy
# load and ends immediately after the second control-byte probe loop.  The hit
# target begins stamp validation; the miss target begins the rule body.
SITES = (
    LookupSite("piece.rule_piece", "piece", 0x100058DFC, 0x100058ECC, 0x08, "x21"),
    LookupSite("piece.rule_atom", "piece", 0x10005C140, 0x10005C210, 0x1B, "x22"),
    LookupSite(
        "atom.rule_pattern", "atom_closure", 0x100070534, 0x100070604, 0xAA, "x28"
    ),
    LookupSite(
        "atom.rule_directive",
        "atom_closure",
        0x100074498,
        0x100074568,
        0x89,
        "x22",
    ),
    LookupSite(
        "atom.rule_named_group",
        "atom_closure",
        0x10007A1A0,
        0x10007A270,
        0x7C,
        "x24",
    ),
)

EXPECTED_FULL_COUNTS = {
    "sub1us": (47, 25, 0, 0, 12),
    "1to2p5": (41, 47, 4, 2, 19),
    "2p5to20": (79, 73, 12, 2, 20),
}
EXPECTED_PROXY_COUNTS = {
    "sub1us": 30,
    "1to2p5": 42,
    "2p5to20": 68,
}


def instructions(
    code: dict[int, tuple[str, str]], site: LookupSite
) -> list[tuple[int, str, str]]:
    return [
        (address, *code[address])
        for address in range(site.start, site.end, 4)
        if address in code
    ]


def direct_parser_memory(mnemonic: str, operands: str) -> bool:
    return capture.instruction_class(mnemonic) == "memory" and "[x20" in operands


def qualify_sites(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> tuple[set[int], set[int]]:
    x20_490 = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if code.get(address) == ("ldr", "x8, [x20, #0x490]")
    }
    if x20_490 != {site.start for site in SITES}:
        raise SystemExit(
            "REFUSE: complete target-symbol thin-memo entry census differs "
            f"actual={sorted(hex(address) for address in x20_490)}"
        )

    full: set[int] = set()
    replacement_proxy: set[int] = set()
    for site in SITES:
        target_start, target_end = target_ranges[site.target]
        if not (target_start <= site.start < site.end <= target_end):
            raise SystemExit(f"REFUSE: lookup site outside target: {site}")
        rows = instructions(code, site)
        if len(rows) != 52 or site.end - site.start != 0xD0:
            raise SystemExit(f"REFUSE: lookup region size differs: {site}")
        if rows[0][1:] != ("ldr", "x8, [x20, #0x490]"):
            raise SystemExit(f"REFUSE: lookup occupancy entry differs: {site}")
        if rows[1][1] != "cbz":
            raise SystemExit(f"REFUSE: lookup occupancy branch differs: {site}")

        text = "\n".join(f"{mnemonic} {operands}" for _, mnemonic, operands in rows)
        required = (
            "#0xa9c5",
            "madd",
            "ror",
            "dup.8b",
            "cmeq.8b",
            "#0x8080808080808080",
            "rbit",
            "clz",
            "#-0xa0",
            "#-0x98",
            "umaxv.8b",
        )
        # otool operands are tab-free after capture.disassembly(), so accept
        # the exact semantic fragments independent of its whitespace spelling.
        required = tuple(fragment.replace("\t", " ") for fragment in required)
        normalized = text.replace("\t", " ")
        if any(fragment not in normalized for fragment in required):
            missing = [fragment for fragment in required if fragment not in normalized]
            raise SystemExit(f"REFUSE: hash/probe fingerprint differs at {site}: {missing}")
        rule_compares = [
            operands
            for _, mnemonic, operands in rows
            if mnemonic == "cmp" and operands.endswith(f", #{site.rule_id:#x}")
        ]
        position_compares = [
            operands
            for _, mnemonic, operands in rows
            if mnemonic == "ccmp" and operands.startswith(f"{site.position_register}, ")
        ]
        if len(rule_compares) != 1 or len(position_compares) != 1:
            raise SystemExit(f"REFUSE: exact key comparison differs at {site}")

        parser_memory = {
            address
            for address, mnemonic, operands in rows
            if direct_parser_memory(mnemonic, operands)
        }
        expected_parser_memory = {site.start, site.start + 0x38, site.start + 0x3C}
        if parser_memory != expected_parser_memory:
            raise SystemExit(f"REFUSE: table-state carrier set differs at {site}")

        site_pcs = set(range(site.start, site.end, 4))
        if full & site_pcs:
            raise SystemExit(f"REFUSE: lookup region overlap at {site}")
        full.update(site_pcs)
        replacement_proxy.update(parser_memory)
        replacement_proxy.add(site.start + 4)

    if len(full) != 260 or len(replacement_proxy) != 20:
        raise SystemExit(
            f"REFUSE: static union differs full={len(full)} proxy={len(replacement_proxy)}"
        )

    prior = {address for address in full if known.owner(address) is not None}
    trace, _, _ = bare.trace_gate_pcs(code, target_ranges)
    coverage, _, _ = bare.coverage_pcs(code, target_ranges)
    rollback, _, _ = bare.rollback_diagnostic_pcs(code, target_ranges)
    diagnostics = trace | coverage | rollback
    if prior or full & diagnostics:
        raise SystemExit(
            "REFUSE: lookup overlaps accepted work "
            f"prior={len(prior)} diagnostics={len(full & diagnostics)}"
        )
    return full, replacement_proxy


def band_counts(
    pcs: list[int],
    slide: int,
    full_by_site: tuple[set[int], ...],
    replacement_proxy: set[int],
) -> tuple[tuple[int, ...], int]:
    site_counts = tuple(
        sum(sampled_pc - slide in site_pcs for sampled_pc in pcs)
        for site_pcs in full_by_site
    )
    proxy_count = sum(sampled_pc - slide in replacement_proxy for sampled_pc in pcs)
    return site_counts, proxy_count


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    full, replacement_proxy = qualify_sites(code, target_ranges)
    hash_exclusive = full - replacement_proxy
    full_by_site = tuple(set(range(site.start, site.end, 4)) for site in SITES)

    static_full = Counter(capture.instruction_class(code[address][0]) for address in full)
    static_proxy = Counter(
        capture.instruction_class(code[address][0]) for address in replacement_proxy
    )
    static_exclusive = Counter(
        capture.instruction_class(code[address][0]) for address in hash_exclusive
    )
    expected_static = (
        Counter({"other": 180, "control": 45, "memory": 35}),
        Counter({"memory": 15, "control": 5}),
        Counter({"other": 180, "control": 40, "memory": 20}),
    )
    if (static_full, static_proxy, static_exclusive) != expected_static:
        raise SystemExit("REFUSE: static instruction-class census differs")

    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print(
        "raw custody: "
        + " ".join(
            f"{band}={known.RAW_SHA256[band]}" for band in PROFILE_WEIGHTS
        )
    )
    print(
        "profile weights: sub1us=0.380 1to2p5=0.353 2p5to20=0.260 "
        "excluded_tail=0.007 priced_at_zero"
    )
    print(
        "static sites: count=5 pcs_per_site=52 full_union=260 "
        "classes=memory:35,control:45,other:180"
    )
    print(
        "static minimal_replacement_proxy: pcs=20 "
        "classes=memory:15,control:5 ownership=three_table_state_loads+occupancy_branch"
    )
    print(
        "static hash_exclusive: pcs=240 classes=memory:20,control:40,other:180 "
        "accepted_success_overlap=0 bare_diagnostic_overlap=0"
    )
    for site in SITES:
        print(
            f"site={site.label:24} target={site.target:12} "
            f"range=[{site.start:#x},{site.end:#x}) rule={site.rule_id:#04x} "
            f"position={site.position_register}"
        )

    raw_totals: dict[str, int] = {}
    full_totals: dict[str, int] = {}
    proxy_totals: dict[str, int] = {}
    exclusive_totals: dict[str, int] = {}
    print("\nDYNAMIC COUNTS")
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: raw capture SHA-256 mismatch for {band}")
        header, pcs = capture.parse_raw(raw)
        slide = int(header["main_slide"], 16)
        site_counts, proxy_count = band_counts(
            pcs, slide, full_by_site, replacement_proxy
        )
        if site_counts != EXPECTED_FULL_COUNTS[band]:
            raise SystemExit(f"REFUSE: {band} site counts differ {site_counts}")
        if proxy_count != EXPECTED_PROXY_COUNTS[band]:
            raise SystemExit(f"REFUSE: {band} proxy count differs {proxy_count}")
        raw_totals[band] = len(pcs)
        full_totals[band] = sum(site_counts)
        proxy_totals[band] = proxy_count
        exclusive_totals[band] = sum(site_counts) - proxy_count
        rendered_sites = ",".join(
            f"{site.label}:{count}" for site, count in zip(SITES, site_counts)
        )
        print(
            f"band={band:8} raw={len(pcs):5d} full={sum(site_counts):3d} "
            f"proxy={proxy_count:2d} hash_exclusive={sum(site_counts)-proxy_count:3d} "
            f"sites={rendered_sites}"
        )

    def weighted_ns(counts: dict[str, int]) -> tuple[float, float]:
        share = sum(
            PROFILE_WEIGHTS[band] * counts[band] / raw_totals[band]
            for band in PROFILE_WEIGHTS
        )
        return share, share * FLOOR_NS

    full_share, full_ns = weighted_ns(full_totals)
    proxy_share, proxy_ns = weighted_ns(proxy_totals)
    exclusive_share, exclusive_ns = weighted_ns(exclusive_totals)
    print("\nPRICING")
    print(
        f"full_lookup_ceiling_share={100.0 * full_share:.9f}% "
        f"full_lookup_ceiling_ns={full_ns:.9f}"
    )
    print(
        f"minimal_replacement_proxy_share={100.0 * proxy_share:.9f}% "
        f"minimal_replacement_proxy_ns={proxy_ns:.9f}"
    )
    print(
        f"hash_exclusive_share={100.0 * exclusive_share:.9f}% "
        f"hash_exclusive_ns={exclusive_ns:.9f}"
    )
    low_captured = CAPTURE_LOW * (HELD_BUNDLE_NS + exclusive_ns)
    margin = low_captured - NOISE_NS
    print(
        f"held_bundle_ns={HELD_BUNDLE_NS:.6f} replacement_aware_bundle_ns="
        f"{HELD_BUNDLE_NS + exclusive_ns:.9f}"
    )
    print(
        f"capture={CAPTURE_LOW:.2f} captured_ns={low_captured:.9f} "
        f"noise_margin_ns={margin:.9f}"
    )
    print(
        "replacement_proxy_optimism=slot_load+dense_entry_base/address+initialization "
        "priced_at_zero"
    )
    print(
        "verdict=HOLD (positive sliver is not honest margin; mandatory direct-index "
        "work remains unpriced)"
    )
    print("THIN-MEMO LOOKUP EXPANSION: PASS")


if __name__ == "__main__":
    main()
