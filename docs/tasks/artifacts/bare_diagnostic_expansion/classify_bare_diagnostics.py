#!/usr/bin/env python3
"""Expand the preserved-probe bare diagnostic mechanisms across all classes.

This is deliberately an instruction-ownership classifier, not a source-line
estimate.  It admits only hot-path instructions that a compile-time bare parse
specialization can remove completely.  Cold trace formatting is excluded;
shared coverage ``stp`` carriers are reported but not priced because the other
word still needs a store.
"""

from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
CAPTURE_DIR = ARTIFACT_DIR.parent / "three_band_pc_capture"
STACK_DIR = ARTIFACT_DIR.parent / "residual_stack_carrier"
sys.path[:0] = [str(CAPTURE_DIR), str(STACK_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_stack_carriers as stack_carriers  # noqa: E402


PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
FLOOR_NS = 1263.4
HELD_BUNDLE_NS = 84.385
NOISE_NS = 28.8

EXPECTED_DIRECT_COUNTS = {
    "sub1us": (104, 30, 0, 9),
    "1to2p5": (95, 29, 0, 13),
    "2p5to20": (120, 38, 1, 16),
}
EXPECTED_EXPANDED_COUNTS = {
    "sub1us": (107, 41, 1),
    "1to2p5": (100, 43, 0),
    "2p5to20": (121, 71, 1),
}

# Five coverage checkpoint reloads are separated from their rollback compare
# by required ParseResult/position movement.  The other eleven reloads sit
# directly between the coverage-length load and compare and are discovered
# structurally below.  Each tuple pins the load to the rollback that consumes
# its destination register only as the saved coverage length.
DEFERRED_COVERAGE_CARRIER_LOADS = {
    0x10005B504: 0x10005B530,
    0x10005D21C: 0x10005D250,
    0x100077374: 0x1000773A4,
    0x100077600: 0x100077630,
    0x1000778AC: 0x1000778DC,
}


def register(operands: str) -> str:
    return operands.split(",", 1)[0].strip()


def target_for(
    address: int, target_ranges: dict[str, tuple[int, int]]
) -> str | None:
    return next(
        (
            target
            for target, (start, end) in target_ranges.items()
            if start <= address < end
        ),
        None,
    )


def direct_field_pcs(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
    offset: int,
) -> set[int]:
    token = f"[x20, #{offset:#x}]"
    return {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if token in code.get(address, ("", ""))[1]
        and capture.instruction_class(code[address][0]) == "memory"
    }


def trace_gate_pcs(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> tuple[set[int], set[int], Counter[str]]:
    """Own each cached logger load and only its exact deciding control use."""

    loads = direct_field_pcs(code, target_ranges, 0x504)
    if len(loads) != 170 or any(code[address][0] != "ldrb" for address in loads):
        raise SystemExit(f"REFUSE: cached-logger load census differs ({len(loads)})")

    all_pcs: set[int] = set()
    forms: Counter[str] = Counter()
    for address in sorted(loads):
        loaded = register(code[address][1])
        candidates: list[tuple[str, set[int]]] = []
        for nearby in range(address + 4, address + 28, 4):
            mnemonic, operands = code.get(nearby, ("", ""))
            if mnemonic in {"tbz", "tbnz"} and register(operands) == loaded:
                candidates.append((mnemonic, {address, nearby}))
            if mnemonic == "cmp" and operands == f"{loaded}, #0x1":
                branches = [
                    branch
                    for branch in range(nearby + 4, nearby + 28, 4)
                    if code.get(branch, ("", ""))[0] in {"b.eq", "b.ne"}
                ]
                if branches:
                    candidates.append(
                        (
                            f"cmp+{code[branches[0]][0]}",
                            {address, nearby, branches[0]},
                        )
                    )
        # Later enabled-arm tests must not become second candidates.  The first
        # candidate is the logger-byte decision and must be unique at its
        # earliest deciding address.
        if not candidates:
            raise SystemExit(f"REFUSE: no trace decision for load {address:#x}")
        earliest = min(max(pcs) for _, pcs in candidates)
        candidates = [item for item in candidates if max(item[1]) == earliest]
        if len(candidates) != 1:
            raise SystemExit(f"REFUSE: ambiguous trace decision at {address:#x}")
        form, site_pcs = candidates[0]
        if all_pcs & site_pcs:
            raise SystemExit(f"REFUSE: trace-gate overlap at {address:#x}")
        all_pcs.update(site_pcs)
        forms[form] += 1

    expected_forms = Counter(
        {"cmp+b.ne": 78, "cmp+b.eq": 1, "tbz": 89, "tbnz": 2}
    )
    if forms != expected_forms or len(all_pcs) != 419:
        raise SystemExit(
            f"REFUSE: trace expansion differs forms={forms} pcs={len(all_pcs)}"
        )
    return all_pcs, loads, forms


def coverage_pcs(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> tuple[set[int], set[int], dict[str, set[int]]]:
    """Own coverage snapshots, rollback cores, and exclusive spill carriers."""

    _, checkpoint_sites = stack_carriers.checkpoint_store_pcs(code, target_ranges)
    snapshot_loads: set[int] = set()
    single_stores: set[int] = set()
    shared_stores: set[int] = set()
    register_carriers: set[int] = set()
    for target, guard, *_ in checkpoint_sites:
        target_start = target_ranges[target][0]
        candidates = [
            address
            for address in range(max(target_start, guard - 160), guard, 4)
            if code.get(address, ("", ""))[0] == "ldr"
            and "[x20, #0x270]" in code[address][1]
        ]
        if len(candidates) != 1:
            raise SystemExit(
                f"REFUSE: checkpoint gate {guard:#x} has coverage snapshots {candidates}"
            )
        address = candidates[0]
        snapshot_loads.add(address)
        loaded = register(code[address][1])
        mnemonic, operands = code[address + 4]
        if (
            mnemonic == "str"
            and register(operands) == loaded
            and ("[sp" in operands or "[x29" in operands)
        ):
            single_stores.add(address + 4)
        elif (
            mnemonic == "stp"
            and re.search(rf"(^|, ){re.escape(loaded)}(,| )", operands)
            and ("[sp" in operands or "[x29" in operands)
        ):
            shared_stores.add(address + 4)
        else:
            register_carriers.add(address)

    if (
        len(snapshot_loads) != 39
        or len(single_stores) != 8
        or len(shared_stores) != 8
        or len(register_carriers) != 23
    ):
        raise SystemExit(
            "REFUSE: coverage snapshot/carrier census differs "
            f"snapshots={len(snapshot_loads)} single={len(single_stores)} "
            f"shared={len(shared_stores)} register={len(register_carriers)}"
        )

    all_direct = direct_field_pcs(code, target_ranges, 0x270)
    direct_loads = {address for address in all_direct if code[address][0] == "ldr"}
    direct_stores = {address for address in all_direct if code[address][0] == "str"}
    rollback_starts = direct_loads - snapshot_loads
    if (
        len(all_direct) != 117
        or len(direct_loads) != 78
        or len(direct_stores) != 39
        or len(rollback_starts) != 39
    ):
        raise SystemExit("REFUSE: coverage direct-field census differs")

    rollback_core: set[int] = set()
    immediate_carrier_loads: set[int] = set()
    rollback_checkpoints: dict[int, str] = {}
    for address in sorted(rollback_starts):
        stores = [
            nearby
            for nearby in range(address + 4, address + 24, 4)
            if code.get(nearby, ("", ""))[0] == "str"
            and "[x20, #0x270]" in code[nearby][1]
        ]
        if len(stores) != 1:
            raise SystemExit(f"REFUSE: rollback store differs at {address:#x}")
        store = stores[0]
        compares = [
            nearby
            for nearby in range(address + 4, store, 4)
            if code.get(nearby, ("", ""))[0] == "cmp"
        ]
        branches = [
            nearby
            for nearby in range(address + 4, store, 4)
            if code.get(nearby, ("", ""))[0] == "b.hi"
        ]
        if len(compares) != 1 or len(branches) != 1:
            raise SystemExit(f"REFUSE: rollback control differs at {address:#x}")
        compare = compares[0]
        current = register(code[address][1])
        compare_operands = [part.strip() for part in code[compare][1].split(",")]
        if len(compare_operands) != 2 or current not in compare_operands:
            raise SystemExit(f"REFUSE: rollback compare dataflow differs at {address:#x}")
        checkpoint = next(reg for reg in compare_operands if reg != current)
        if register(code[store][1]) != checkpoint:
            raise SystemExit(f"REFUSE: rollback store dataflow differs at {address:#x}")
        rollback_checkpoints[address] = checkpoint
        rollback_core.update((address, compare, branches[0], store))

        carrier_candidates = [
            nearby
            for nearby in range(address + 4, compare, 4)
            if code.get(nearby, ("", ""))[0] == "ldr"
            and register(code[nearby][1]) == checkpoint
            and ("[sp" in code[nearby][1] or "[x29" in code[nearby][1])
        ]
        if len(carrier_candidates) > 1:
            raise SystemExit(f"REFUSE: ambiguous rollback carrier at {address:#x}")
        immediate_carrier_loads.update(carrier_candidates)

    if len(rollback_core) != 156 or direct_stores - rollback_core:
        raise SystemExit("REFUSE: coverage rollback core does not own every direct store")
    if len(immediate_carrier_loads) != 11:
        raise SystemExit(
            f"REFUSE: immediate coverage carrier census {len(immediate_carrier_loads)} != 11"
        )

    deferred_carrier_loads = set(DEFERRED_COVERAGE_CARRIER_LOADS)
    for load, rollback in DEFERRED_COVERAGE_CARRIER_LOADS.items():
        mnemonic, operands = code.get(load, ("", ""))
        if (
            mnemonic != "ldr"
            or ("[sp" not in operands and "[x29" not in operands)
            or register(operands) != rollback_checkpoints.get(rollback)
            or not (load < rollback)
        ):
            raise SystemExit(
                f"REFUSE: deferred coverage carrier dataflow differs at {load:#x}"
            )
        for nearby in range(load + 4, rollback, 4):
            mnemonic, operands = code.get(nearby, ("", ""))
            loaded = register(code[load][1])
            if re.search(rf"\b{re.escape(loaded)}\b", operands):
                raise SystemExit(
                    f"REFUSE: deferred carrier register reused at {nearby:#x}"
                )

    carrier_loads = immediate_carrier_loads | deferred_carrier_loads
    if len(carrier_loads) != 16:
        raise SystemExit("REFUSE: coverage carrier reload census differs")

    # A one-word store can disappear.  A shared stp remains one instruction
    # after specialization (as a store of the required neighbor), so it is an
    # explicit unpriced width effect and is not admitted to the mechanism.
    removable = snapshot_loads | single_stores | rollback_core | carrier_loads
    if len(removable) != 219:
        raise SystemExit(f"REFUSE: coverage removable PC count {len(removable)} != 219")
    details = {
        "snapshot_load": snapshot_loads,
        "snapshot_single_store": single_stores,
        "snapshot_shared_store_unpriced": shared_stores,
        "snapshot_register_carrier": register_carriers,
        "rollback_core": rollback_core,
        "rollback_carrier_load": carrier_loads,
    }
    return removable, all_direct, details


def rollback_diagnostic_pcs(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> tuple[set[int], set[int], Counter[int]]:
    """Own the chain-depth test and increment of rollbacks_nonempty_chain."""

    direct = direct_field_pcs(code, target_ranges, 0x248)
    loads = {address for address in direct if code[address][0] == "ldr"}
    stores = {address for address in direct if code[address][0] == "str"}
    if len(loads) != 63 or len(stores) != 63:
        raise SystemExit("REFUSE: rollback-diagnostic direct census differs")

    all_pcs: set[int] = set()
    compare_distances: Counter[int] = Counter()
    for address in sorted(loads):
        if (
            code.get(address + 4, ("", ""))[0] != "add"
            or code.get(address + 8, ("", ""))[0] != "str"
            or "[x20, #0x248]" not in code[address + 8][1]
            or code.get(address - 4, ("", ""))[0] not in {"b.lo", "b.ls"}
        ):
            raise SystemExit(f"REFUSE: rollback counter body differs at {address:#x}")
        compares = [
            nearby
            for nearby in range(address - 20, address - 4, 4)
            if code.get(nearby, ("", ""))[0] == "cmp"
        ]
        if not compares:
            raise SystemExit(f"REFUSE: rollback counter compare absent at {address:#x}")
        compare = max(compares)
        intervening = {
            code[nearby][0] for nearby in range(compare + 4, address - 4, 4)
        }
        if not intervening <= {"ldr", "add"}:
            raise SystemExit(
                f"REFUSE: flag-setting ambiguity before counter at {address:#x}: "
                f"{intervening}"
            )
        all_pcs.update((compare, address - 4, address, address + 4, address + 8))
        compare_distances[address - compare] += 1

    if len(all_pcs) != 315 or compare_distances != Counter({8: 45, 12: 15, 16: 3}):
        raise SystemExit(
            "REFUSE: rollback-diagnostic expansion differs "
            f"pcs={len(all_pcs)} distances={compare_distances}"
        )
    return all_pcs, direct, compare_distances


def band_hits(
    code: dict[int, tuple[str, str]],
    pcs: list[int],
    slide: int,
    owned: dict[str, set[int]],
) -> tuple[Counter[str], Counter[str]]:
    mechanism_counts: Counter[str] = Counter()
    class_counts: Counter[str] = Counter()
    union = set().union(*owned.values())
    for sampled_pc in pcs:
        address = sampled_pc - slide
        for mechanism, addresses in owned.items():
            if address in addresses:
                mechanism_counts[mechanism] += 1
        if address in union:
            class_counts[capture.instruction_class(code[address][0])] += 1
    return mechanism_counts, class_counts


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)

    trace_all, trace_direct, trace_forms = trace_gate_pcs(code, target_ranges)
    coverage_all, coverage_direct, coverage_details = coverage_pcs(
        code, target_ranges
    )
    counter_all, counter_direct, counter_distances = rollback_diagnostic_pcs(
        code, target_ranges
    )
    memo_taint_direct = direct_field_pcs(code, target_ranges, 0x250)
    if len(memo_taint_direct) != 18:
        raise SystemExit("REFUSE: memo-taint direct census differs")

    expanded = {
        "trace_gate": trace_all,
        "coverage": coverage_all,
        "rollback_diagnostic": counter_all,
    }
    direct = {
        "trace_gate": trace_direct,
        "coverage": coverage_direct,
        "rollback_diagnostic": counter_direct,
        "memo_taint_required": memo_taint_direct,
    }
    occupied: dict[int, str] = {}
    for mechanism, addresses in expanded.items():
        for address in addresses:
            if address in occupied:
                raise SystemExit(
                    f"REFUSE: expanded overlap at {address:#x}: "
                    f"{occupied[address]} vs {mechanism}"
                )
            if target_for(address, target_ranges) is None or known.owner(address) is not None:
                raise SystemExit(f"REFUSE: prior-range/outside overlap at {address:#x}")
            occupied[address] = mechanism
    if len(occupied) != 953:
        raise SystemExit(f"REFUSE: expanded union has {len(occupied)} PCs, expected 953")
    if any(not addresses <= expanded[name] for name, addresses in direct.items() if name != "memo_taint_required"):
        raise SystemExit("REFUSE: direct diagnostic set is not a subset of expansion")
    if memo_taint_direct & set(occupied):
        raise SystemExit("REFUSE: required memo-taint signal overlaps removable work")

    static_classes = Counter(capture.instruction_class(code[address][0]) for address in occupied)
    expected_static_classes = Counter({"memory": 437, "control": 453, "other": 63})
    if static_classes != expected_static_classes:
        raise SystemExit(f"REFUSE: static instruction classes differ {static_classes}")

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
        "semantic correction: PASS offset_0x250=required_memo_taint_signal "
        "offset_0x248=diagnostic_rollbacks_nonempty_chain"
    )
    print(
        "static trace: sites=170 pcs=419 forms="
        + ",".join(f"{name}:{trace_forms[name]}" for name in sorted(trace_forms))
    )
    print(
        "static coverage: checkpoints=39 snapshot_loads=39 "
        "single_store_carriers=8 shared_stp_carriers_unpriced=8 "
        "register_carriers=23 rollback_sites=39 rollback_core_pcs=156 "
        "carrier_reloads=16 removable_pcs=219"
    )
    print(
        "static rollback_diagnostic: sites=63 pcs=315 compare_distance_bytes="
        + ",".join(
            f"{distance}:{counter_distances[distance]}"
            for distance in sorted(counter_distances)
        )
    )
    print(
        "static union: pcs=953 classes="
        + ",".join(
            f"{name}:{static_classes[name]}" for name in ("memory", "control", "other")
        )
        + " prior_range_overlap=0 intermechanism_overlap=0"
    )

    raw_totals: dict[str, int] = {}
    expanded_totals: dict[str, int] = {}
    print("\nDIRECT-FIELD CORRECTION")
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: {band} raw SHA-256 mismatch")
        header, pcs = capture.parse_raw(raw)
        slide = int(header["main_slide"], 16)
        raw_totals[band] = len(pcs)
        counts, _ = band_hits(code, pcs, slide, direct)
        actual = (
            counts["trace_gate"],
            counts["coverage"],
            counts["rollback_diagnostic"],
            counts["memo_taint_required"],
        )
        if actual != EXPECTED_DIRECT_COUNTS[band]:
            raise SystemExit(f"REFUSE: {band} direct correction differs {actual}")
        removable = sum(actual[:3])
        print(
            f"band={band:8} raw={len(pcs):5d} trace={actual[0]:3d} "
            f"coverage={actual[1]:2d} rollback_diagnostic={actual[2]:1d} "
            f"removable={removable:3d} memo_taint_required={actual[3]:2d}"
        )
    direct_share = sum(
        PROFILE_WEIGHTS[band]
        * sum(EXPECTED_DIRECT_COUNTS[band][:3])
        / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    print(f"corrected_direct_removable_share={100.0 * direct_share:.6f}%")
    print("supersedes_provisional_0.601206_percent=YES")

    print("\nEXPANDED REMOVABLE MECHANISM")
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        header, pcs = capture.parse_raw(raw)
        slide = int(header["main_slide"], 16)
        counts, classes = band_hits(code, pcs, slide, expanded)
        actual = (
            counts["trace_gate"],
            counts["coverage"],
            counts["rollback_diagnostic"],
        )
        if actual != EXPECTED_EXPANDED_COUNTS[band]:
            raise SystemExit(f"REFUSE: {band} expanded counts differ {actual}")
        total = sum(actual)
        expanded_totals[band] = total
        print(
            f"band={band:8} raw={len(pcs):5d} trace={actual[0]:3d} "
            f"coverage={actual[1]:2d} rollback_diagnostic={actual[2]:1d} "
            f"total={total:3d} classes=memory:{classes['memory']},"
            f"control:{classes['control']},other:{classes['other']}"
        )
    expanded_share = sum(
        PROFILE_WEIGHTS[band] * expanded_totals[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    target_ns = expanded_share * FLOOR_NS
    repriced_bundle = HELD_BUNDLE_NS + target_ns
    print(f"weighted_all_sample_share={100.0 * expanded_share:.6f}%")
    print(f"mechanism_target_ns={target_ns:.6f}")
    print(
        "excluded=trace_enabled_cold_arms,8_shared_stp_whole_instructions,"
        "register_pressure_and_width_upside"
    )
    shared_hits = {}
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        header, pcs = capture.parse_raw(raw)
        slide = int(header["main_slide"], 16)
        shared_hits[band] = sum(
            sampled_pc - slide
            in coverage_details["snapshot_shared_store_unpriced"]
            for sampled_pc in pcs
        )
    print(
        "shared_stp_samples_unpriced="
        + " ".join(f"{band}:{shared_hits[band]}" for band in PROFILE_WEIGHTS)
    )

    print("\nMARGIN ADJUDICATION")
    print(f"held_bundle_ns={HELD_BUNDLE_NS:.3f}")
    print(f"repriced_bundle_ns={repriced_bundle:.6f}")
    for fraction in (0.30, 0.50, 0.70):
        captured = repriced_bundle * fraction
        print(
            f"capture={fraction:.2f} captured_ns={captured:.6f} "
            f"noise_margin_ns={captured - NOISE_NS:.6f}"
        )
    print("verdict=HOLD (30_percent_case remains below 28.8_ns noise)")
    print("BARE DIAGNOSTIC EXPANSION: PASS")


if __name__ == "__main__":
    main()
