#!/usr/bin/env python3
"""Expand the name-stack truncate half of generated ``try_parse`` rollback."""

from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
GUARD_DIR = TASK_ARTIFACTS / "recursion_guard_expansion"
BARE_DIR = TASK_ARTIFACTS / "bare_diagnostic_expansion"
LOOKUP_DIR = TASK_ARTIFACTS / "thin_memo_lookup_expansion"
TELEMETRY_DIR = TASK_ARTIFACTS / "semantic_telemetry_expansion"
sys.path[:0] = [
    str(CAPTURE_DIR),
    str(GUARD_DIR),
    str(BARE_DIR),
    str(LOOKUP_DIR),
    str(TELEMETRY_DIR),
]

import analyze_capture as capture  # noqa: E402
import classify_bare_diagnostics as bare  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_recursion_guard as guard  # noqa: E402
import classify_semantic_telemetry as telemetry  # noqa: E402
import classify_thin_memo_lookup as thin_lookup  # noqa: E402


PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
FLOOR_NS = 1263.4
PRE_LOOKUP_BUNDLE_NS = 96.595388589
OPTIMISTIC_LOOKUP_BUNDLE_NS = 100.419995319
NOISE_NS = 28.8
EXPECTED_SITES = (
    ("piece", 0x10005A8FC),
    ("piece", 0x10005AAC0),
    ("piece", 0x10005ACD4),
    ("piece", 0x10005B348),
    ("piece", 0x10005B584),
    ("piece", 0x10005C8F4),
    ("piece", 0x10005D2AC),
    ("piece", 0x10005D8FC),
    ("piece", 0x10005E22C),
    ("piece", 0x10005E480),
    ("piece", 0x10005E944),
    ("piece", 0x10005EB5C),
    ("atom_closure", 0x10006B324),
    ("atom_closure", 0x10006B998),
    ("atom_closure", 0x10006C9B0),
    ("atom_closure", 0x10006D014),
    ("atom_closure", 0x10006DB20),
    ("atom_closure", 0x10006E190),
    ("atom_closure", 0x10006EA70),
    ("atom_closure", 0x10007001C),
    ("atom_closure", 0x100070D84),
    ("atom_closure", 0x1000715D8),
    ("atom_closure", 0x100071820),
    ("atom_closure", 0x100071CC8),
    ("atom_closure", 0x100072340),
    ("atom_closure", 0x100074A6C),
    ("atom_closure", 0x100074F94),
    ("atom_closure", 0x100076094),
    ("atom_closure", 0x100076EC4),
    ("atom_closure", 0x1000773F8),
    ("atom_closure", 0x100077684),
    ("atom_closure", 0x100077930),
    ("atom_closure", 0x100077ACC),
    ("atom_closure", 0x10007829C),
    ("atom_closure", 0x100078D14),
    ("atom_closure", 0x1000799EC),
    ("atom_closure", 0x10007B4C8),
    ("atom_closure", 0x10007BA5C),
    ("atom_closure", 0x10007C54C),
)
EXPECTED_BAND_COUNTS = {
    "sub1us": Counter({"cmp": 3}),
    "1to2p5": Counter({"cmp": 4}),
    "2p5to20": Counter({"cmp": 1}),
}


def destination_register(operands: str) -> str:
    return operands.split(",", 1)[0].strip()


def exact_store(operands: str, register: str, offset: str) -> bool:
    return operands == f"{register}, [x20, #{offset}]"


def discover_truncate_sites(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
    prior_owned: set[int],
) -> list[tuple[str, int, int, int, int, set[int]]]:
    """Return target/load/cmp/branch/store and its exclusive three-PC subset."""

    sites: list[tuple[str, int, int, int, int, set[int]]] = []
    for target, (start, end) in target_ranges.items():
        for store in range(start, end, 4):
            mnemonic, operands = code.get(store, ("", ""))
            if mnemonic != "str" or "[x20, #0x10]" not in operands or store in prior_owned:
                continue
            saved = destination_register(operands)
            id_stores = [
                address
                for address in range(store + 4, min(end, store + 132), 4)
                if code.get(address, ("", ""))
                == ("str", f"{saved}, [x20, #0x28]")
            ]
            if len(id_stores) != 1:
                continue

            name_loads = [
                address
                # Optimized error arms can format/drop a complete ParseError
                # between loading the current name depth and reaching one of
                # several shared truncate tails.  Search within the enclosing
                # inlined try_parse region, not only the final basic block.
                for address in range(max(start, store - 2048), store, 4)
                if code.get(address, ("", ""))[0] == "ldr"
                and "[x20, #0x10]" in code[address][1]
            ]
            if not name_loads:
                raise SystemExit(f"REFUSE: paired truncate lacks name-length load at {store:#x}")
            load = name_loads[-1]
            # The name-depth branch either lands on the name store (saved depth
            # is admissible) or one instruction after it (saved depth exceeds
            # the current depth).  Register allocation can reuse the same
            # names through a long error/drop arm, so target ownership is more
            # precise than choosing the first textual cmp.
            branches = [
                address
                for address in range(max(load + 4, store - 128), store, 4)
                if code.get(address, ("", ""))[0].startswith("b.")
                and int(code[address][1], 16) in {store, store + 4}
            ]
            if len(branches) != 1:
                raise SystemExit(
                    f"REFUSE: paired truncate branch differs at {store:#x}: {branches}"
                )
            branch = branches[0]
            compare = branch - 4
            compare_instruction = code.get(compare, ("", ""))
            if (
                compare_instruction[0] != "cmp"
                or saved not in compare_instruction[1].split(", ")
            ):
                raise SystemExit(
                    f"REFUSE: paired truncate compare differs at {store:#x}: "
                    f"{compare_instruction}"
                )
            sites.append(
                (target, load, compare, branch, store, {compare, branch, store})
            )
    return sites


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    pops = guard.discover_name_pops(code, target_ranges)
    prior_owned = guard.INLINE_NAME_PUSH | set().union(
        *(pcs for _, _, pcs in pops), set()
    )
    sites = discover_truncate_sites(code, target_ranges, prior_owned)
    owned = set().union(*(pcs for *_, pcs in sites), set())
    if tuple((target, store) for target, *_, store, _ in sites) != EXPECTED_SITES:
        raise SystemExit(
            "REFUSE: try_parse name-truncate site census differs "
            f"{[(target, store) for target, *_, store, _ in sites]}"
        )
    static_classes = Counter(capture.instruction_class(code[address][0]) for address in owned)
    if len(owned) != 117 or static_classes != Counter({"control": 78, "memory": 39}):
        raise SystemExit(
            f"REFUSE: static rollback subset differs pcs={len(owned)} classes={static_classes}"
        )

    trace, _, _ = bare.trace_gate_pcs(code, target_ranges)
    coverage, _, _ = bare.coverage_pcs(code, target_ranges)
    nonempty, _, _ = bare.rollback_diagnostic_pcs(code, target_ranges)
    bare_union = trace | coverage | nonempty
    lookup_full, _ = thin_lookup.qualify_sites(code, target_ranges)
    telemetry_sites = telemetry.discover_sites(code, target_ranges)
    telemetry_union = set().union(*(pcs for _, _, _, pcs in telemetry_sites), set())
    accepted_overlap = {
        address: known.owner(address)
        for address in owned
        if known.owner(address) is not None
    }
    overlaps = {
        "accepted": len(accepted_overlap),
        "bare_0185": len(owned & bare_union),
        "thin_lookup": len(owned & lookup_full),
        "telemetry_0187": len(owned & telemetry_union),
        "guard_0188": len(owned & prior_owned),
    }
    if any(overlaps.values()):
        raise SystemExit(
            f"REFUSE: try_parse rollback overlap {overlaps} accepted={accepted_overlap}"
        )

    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print(
        "raw custody: "
        + " ".join(f"{band}={known.RAW_SHA256[band]}" for band in PROFILE_WEIGHTS)
    )
    print(
        "profile weights: sub1us=0.380 1to2p5=0.353 2p5to20=0.260 "
        "excluded_tail=0.007 priced_at_zero"
    )
    print(
        f"static subset: sites={len(sites)} pcs={len(owned)} "
        f"classes=memory:{static_classes['memory']},control:{static_classes['control']} "
        f"overlap={overlaps}"
    )
    print(
        "replacement contract: snapshot name_len->id_len is one-for-one; "
        "last RuleId->RULE_NAMES cost excluded; retained ID truncate follows every site"
    )
    print("truncate sites:")
    for target, load, compare, branch, store, _ in sites:
        print(
            f"site target={target:12} load={load:#011x} compare={compare:#011x} "
            f"branch={branch:#011x} store={store:#011x}"
        )

    raw_totals: dict[str, int] = {}
    totals: dict[str, int] = {}
    print("\nDYNAMIC COUNTS")
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: raw capture SHA-256 mismatch for {band}")
        header, pcs = capture.parse_raw(raw)
        slide = int(header["main_slide"], 16)
        counts = Counter(
            code[sampled_pc - slide][0]
            for sampled_pc in pcs
            if sampled_pc - slide in owned
        )
        if counts != EXPECTED_BAND_COUNTS[band]:
            raise SystemExit(f"REFUSE: {band} rollback counts differ {counts}")
        raw_totals[band] = len(pcs)
        totals[band] = sum(counts.values())
        print(
            f"band={band:8} raw={len(pcs):5d} compare={counts['cmp']:2d} "
            f"branch={counts['b.ls'] + counts['b.hi']:2d} store={counts['str']:2d} "
            f"total={totals[band]:2d}"
        )

    share = sum(
        PROFILE_WEIGHTS[band] * totals[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    target_ns = share * FLOOR_NS
    pre_lookup = PRE_LOOKUP_BUNDLE_NS + target_ns
    optimistic = OPTIMISTIC_LOOKUP_BUNDLE_NS + target_ns
    print("\nPRICING")
    print(f"rollback_share={100.0 * share:.9f}% rollback_target_ns={target_ns:.9f}")
    print(
        f"pre_lookup_composite_ns={pre_lookup:.9f} "
        f"capture_0.30_ns={0.30 * pre_lookup:.9f} "
        f"noise_margin_ns={0.30 * pre_lookup - NOISE_NS:.9f}"
    )
    print(
        f"optimistic_lookup_composite_ns={optimistic:.9f} "
        f"capture_0.30_ns={0.30 * optimistic:.9f} "
        f"noise_margin_ns={0.30 * optimistic - NOISE_NS:.9f}"
    )
    print(
        "verdict=HOLD (net removable rollback subset is tiny; honest margin remains "
        "below replacement/model uncertainty)"
    )
    print("TRY_PARSE NAME ROLLBACK EXPANSION: PASS")


if __name__ == "__main__":
    main()
