#!/usr/bin/env python3
"""Price the duplicate name-stack push/pop subset of the C1 recursion guard.

The preserved modern generated parser maintains both a 24-byte
``(&'static str, position)`` stack and a 16-byte ``(RuleId, position)`` stack.
The latter is the required cycle-check representation.  This classifier owns
only instructions exclusively implementing name-stack pushes and pops in the
two captured fused symbols plus the one outlined ``enter_id`` helper.  It does
not price cycle scans, depth checks, try-parse snapshots/truncation, last-rule
name reconstruction, error paths, or time inside allocator children.
"""

from __future__ import annotations

import subprocess
import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
BARE_DIR = TASK_ARTIFACTS / "bare_diagnostic_expansion"
LOOKUP_DIR = TASK_ARTIFACTS / "thin_memo_lookup_expansion"
TELEMETRY_DIR = TASK_ARTIFACTS / "semantic_telemetry_expansion"
sys.path[:0] = [
    str(CAPTURE_DIR),
    str(BARE_DIR),
    str(LOOKUP_DIR),
    str(TELEMETRY_DIR),
]

import analyze_capture as capture  # noqa: E402
import classify_bare_diagnostics as bare  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_semantic_telemetry as telemetry  # noqa: E402
import classify_thin_memo_lookup as thin_lookup  # noqa: E402


PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
FLOOR_NS = 1263.4
PRE_LOOKUP_BUNDLE_NS = 92.654979146
OPTIMISTIC_LOOKUP_BUNDLE_NS = 96.479585876
NOISE_NS = 28.8
EXPECTED_POP_SITES = (
    ("piece", 0x10005C550),
    ("piece", 0x10005CB10),
    ("atom_closure", 0x100071F0C),
    ("atom_closure", 0x100075B24),
    ("atom_closure", 0x100075E0C),
    ("atom_closure", 0x1000763D4),
    ("atom_closure", 0x1000766C4),
    ("atom_closure", 0x10007B01C),
    ("atom_closure", 0x10007C134),
)
EXPECTED_BAND_COUNTS = {
    "sub1us": Counter({"inline_push": 48, "pop": 18, "outlined_push": 5}),
    "1to2p5": Counter({"inline_push": 38, "pop": 23, "outlined_push": 13}),
    "2p5to20": Counter({"inline_push": 57, "pop": 23, "outlined_push": 12}),
}
EXPECTED_DYNAMIC_CLASSES = {
    "sub1us": Counter({"memory": 71}),
    "1to2p5": Counter({"memory": 71, "control": 2, "other": 1}),
    "2p5to20": Counter({"memory": 87, "control": 3, "other": 2}),
}

ENTER_ID_START = 0x10008DEC8
ENTER_ID_END = 0x10008DF98


def pc_range(start: int, end_inclusive: int) -> set[int]:
    return set(range(start, end_inclusive + 4, 4))


# These sets are the source/dataflow-qualified name half of four inlined paired
# pushes.  The two pre-index carrier instructions at 0x100059170/174,
# 0x100070648/64c, and 0x10007a4a8/4ac are deliberately excluded: an ID-only
# replacement still needs an ID-stack base/capacity load.  The far first-site
# slow block is included, but allocator-child PCs are not.
INLINE_NAME_PUSH = (
    pc_range(0x100059160, 0x10005916C)
    | pc_range(0x100059178, 0x10005919C)
    | pc_range(0x10005A6EC, 0x10005A6FC)
    | pc_range(0x10005C4B8, 0x10005C4FC)
    | pc_range(0x10007062C, 0x100070644)
    | pc_range(0x100070650, 0x100070674)
    | pc_range(0x10007A488, 0x10007A4A4)
    | pc_range(0x10007A4B0, 0x10007A4D4)
)

# The outlined helper's fast name push and name-only grow/rejoin arm.  ABI
# prologue/epilogue, the complete ID push, and the ID grow arm remain required
# and are excluded.
OUTLINED_NAME_PUSH = pc_range(0x10008DEDC, 0x10008DF04) | pc_range(
    0x10008DF44, 0x10008DF70
)


def symbol_identity() -> None:
    output = subprocess.run(
        ["nm", "-nm", str(capture.PROBE)], check=True, capture_output=True, text=True
    ).stdout
    rows: list[tuple[int, str]] = []
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 4 and len(fields[0]) == 16:
            try:
                rows.append((int(fields[0], 16), fields[-1]))
            except ValueError:
                pass
    rows.sort()
    matches = [
        index
        for index, (_, name) in enumerate(rows)
        if "RecursionGuard8enter_id" in name
    ]
    if len(matches) != 1:
        raise SystemExit(f"REFUSE: outlined enter_id symbol matches {matches}")
    index = matches[0]
    if rows[index][0] != ENTER_ID_START or rows[index + 1][0] != ENTER_ID_END:
        raise SystemExit(
            "REFUSE: outlined enter_id range differs "
            f"{rows[index][0]:#x}..{rows[index + 1][0]:#x}"
        )


def discover_name_pops(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> list[tuple[str, int, set[int]]]:
    """Find the name half of paired, inlined ``RecursionGuard::exit`` calls."""

    result: list[tuple[str, int, set[int]]] = []
    for target, (start, end) in target_ranges.items():
        for address in range(start, end - 12, 4):
            mnemonic, operands = code.get(address, ("", ""))
            if mnemonic != "ldr" or "[x20, #0x10]" not in operands:
                continue
            register = operands.split(",", 1)[0].strip()
            expected = (
                ("ldr", operands),
                ("cbz", f"{register}, {code[address + 4][1].split(', ', 1)[-1]}"),
                ("sub", f"{register}, {register}, #0x1"),
                ("str", f"{register}, [x20, #0x10]"),
            )
            actual = tuple(code.get(address + offset, ("", "")) for offset in range(0, 16, 4))
            if actual[1][0] != "cbz" or not actual[1][1].startswith(register + ", "):
                continue
            if actual != expected:
                continue
            # C1's lockstep exit must immediately begin the equivalent ID pop.
            id_load = code.get(address + 16, ("", ""))
            if id_load[0] != "ldr" or "[x20, #0x28]" not in id_load[1]:
                raise SystemExit(f"REFUSE: unpaired name pop at {address:#x}")
            result.append((target, address, pc_range(address, address + 12)))
    return result


def assert_push_fingerprints(code: dict[int, tuple[str, str]]) -> None:
    required = {
        0x100059160: ("ldr", "[x20, #0x10]"),
        0x100059164: ("ldr", "[x20]"),
        0x10005C4B8: ("ldr", "[x20, #0x10]"),
        0x10005C4BC: ("ldr", "[x20]"),
        0x10007062C: ("ldr", "[x20]"),
        0x10007A488: ("ldr", "[x20, #0x10]"),
        0x10007A48C: ("ldr", "[x20]"),
        0x10008DEDC: ("ldr", "[x0, #0x10]"),
        0x10008DEE0: ("ldr", "[x0]"),
    }
    for address, (mnemonic, operand_fragment) in required.items():
        actual = code.get(address, ("", ""))
        if actual[0] != mnemonic or operand_fragment not in actual[1]:
            raise SystemExit(f"REFUSE: name-push fingerprint differs at {address:#x}: {actual}")

    combined = INLINE_NAME_PUSH | OUTLINED_NAME_PUSH
    if len(INLINE_NAME_PUSH) != 72 or len(OUTLINED_NAME_PUSH) != 23 or len(combined) != 95:
        raise SystemExit(
            "REFUSE: name-push static set differs "
            f"inline={len(INLINE_NAME_PUSH)} outlined={len(OUTLINED_NAME_PUSH)}"
        )
    for address in combined:
        if address not in code:
            raise SystemExit(f"REFUSE: name-push disassembly hole at {address:#x}")


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    symbol_identity()
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    assert_push_fingerprints(code)

    pops = discover_name_pops(code, target_ranges)
    pop_pcs = set().union(*(pcs for _, _, pcs in pops), set())
    owned = INLINE_NAME_PUSH | OUTLINED_NAME_PUSH | pop_pcs
    if tuple((target, address) for target, address, _ in pops) != EXPECTED_POP_SITES:
        raise SystemExit(
            "REFUSE: name-pop site census differs "
            f"{[(target, address) for target, address, _ in pops]}"
        )

    trace, _, _ = bare.trace_gate_pcs(code, target_ranges)
    coverage, _, _ = bare.coverage_pcs(code, target_ranges)
    nonempty, _, _ = bare.rollback_diagnostic_pcs(code, target_ranges)
    bare_union = trace | coverage | nonempty
    lookup_full, _ = thin_lookup.qualify_sites(code, target_ranges)
    telemetry_sites = telemetry.discover_sites(code, target_ranges)
    telemetry_union = set().union(*(pcs for _, _, _, pcs in telemetry_sites), set())
    target_owned = INLINE_NAME_PUSH | pop_pcs
    prior_overlap = {address for address in target_owned if known.owner(address) is not None}
    overlaps = {
        "accepted": len(prior_overlap),
        "bare_0185": len(target_owned & bare_union),
        "thin_lookup": len(target_owned & lookup_full),
        "telemetry_0187": len(target_owned & telemetry_union),
    }
    if any(overlaps.values()):
        raise SystemExit(f"REFUSE: recursion subset overlap {overlaps}")

    static_classes = Counter(capture.instruction_class(code[address][0]) for address in owned)
    expected_static_classes = Counter(
        {"other": 55, "memory": 50, "control": 21, "call": 5}
    )
    if len(owned) != 131 or static_classes != expected_static_classes:
        raise SystemExit(
            f"REFUSE: static subset differs pcs={len(owned)} classes={static_classes}"
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
        "source contract: rule_id_stack=required_complete_modern_stack "
        "parse_stack=duplicate_names+legacy/error_text"
    )
    print(
        f"static sites: inline_name_push=4 outlined_name_push=1 name_pop={len(pops)}"
    )
    print(
        f"static subset: pcs={len(owned)} inline_push_pcs={len(INLINE_NAME_PUSH)} "
        f"outlined_push_pcs={len(OUTLINED_NAME_PUSH)} pop_pcs={len(pop_pcs)} "
        f"classes=memory:{static_classes['memory']},control:{static_classes['control']},"
        f"call:{static_classes['call']},other:{static_classes['other']} overlap={overlaps}"
    )
    print(
        "excluded: cycle_scan depth_check try_parse_snapshot/truncate last_rule_lookup "
        "contextual_error allocator_children abi_prologue_epilogue"
    )
    print("name-pop sites:")
    for target, address, _ in pops:
        print(f"site target={target:12} address={address:#011x}")

    raw_totals: dict[str, int] = {}
    category_totals: dict[str, Counter[str]] = {}
    print("\nDYNAMIC COUNTS")
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: raw capture SHA-256 mismatch for {band}")
        header, pcs = capture.parse_raw(raw)
        slide = int(header["main_slide"], 16)
        counts: Counter[str] = Counter()
        classes: Counter[str] = Counter()
        for sampled_pc in pcs:
            address = sampled_pc - slide
            category = None
            if address in INLINE_NAME_PUSH:
                category = "inline_push"
            elif address in OUTLINED_NAME_PUSH:
                category = "outlined_push"
            elif address in pop_pcs:
                category = "pop"
            if category is not None:
                counts[category] += 1
                classes[capture.instruction_class(code[address][0])] += 1
        raw_totals[band] = len(pcs)
        category_totals[band] = counts
        if counts != EXPECTED_BAND_COUNTS[band] or classes != EXPECTED_DYNAMIC_CLASSES[band]:
            raise SystemExit(
                f"REFUSE: {band} dynamic subset differs counts={counts} classes={classes}"
            )
        total = sum(counts.values())
        print(
            f"band={band:8} raw={len(pcs):5d} inline_push={counts['inline_push']:3d} "
            f"outlined_push={counts['outlined_push']:3d} pop={counts['pop']:3d} "
            f"total={total:3d} classes=memory:{classes['memory']},"
            f"control:{classes['control']},call:{classes['call']},other:{classes['other']}"
        )

    target_counts = {
        band: sum(category_totals[band].values()) for band in PROFILE_WEIGHTS
    }
    share = sum(
        PROFILE_WEIGHTS[band] * target_counts[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    target_ns = share * FLOOR_NS
    pre_lookup = PRE_LOOKUP_BUNDLE_NS + target_ns
    optimistic = OPTIMISTIC_LOOKUP_BUNDLE_NS + target_ns
    print("\nPRICING")
    print(f"guard_subset_share={100.0 * share:.9f}% guard_subset_target_ns={target_ns:.9f}")
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
        "scope=conservative fixed subset only; replacement name reconstruction and "
        "required ID stack are priced at zero removal"
    )
    print(
        "verdict=HOLD (pre-lookup arithmetic margin is only 0.178617 ns; "
        "ID-to-name reconstruction remains unpriced and the lookup composite is optimistic)"
    )
    print("RECURSION GUARD EXPANSION: PASS")


if __name__ == "__main__":
    main()
