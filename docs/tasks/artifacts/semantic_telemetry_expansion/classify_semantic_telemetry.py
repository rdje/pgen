#!/usr/bin/env python3
"""Expand rollback telemetry updates in the preserved bare fused targets.

The arm64 optimizer combines cumulative rollback counters into two vectorized
forms: a two-counter increment for ordinary rollback and a four-counter
increment for tournament cleanup.  This classifier discovers every form from
its parser-relative counter-base materialization and owns only the complete
counter-update sequence.  Required epoch/deferred checks and rollback work are
outside the owned set; the nonempty-chain counter was already owned by -0185.
"""

from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
BARE_DIR = TASK_ARTIFACTS / "bare_diagnostic_expansion"
LOOKUP_DIR = TASK_ARTIFACTS / "thin_memo_lookup_expansion"
sys.path[:0] = [str(CAPTURE_DIR), str(BARE_DIR), str(LOOKUP_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_bare_diagnostics as bare  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_thin_memo_lookup as thin_lookup  # noqa: E402


PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
FLOOR_NS = 1263.4
PRE_LOOKUP_HELD_BUNDLE_NS = 92.427632
OPTIMISTIC_LOOKUP_BUNDLE_NS = 96.252238730
NOISE_NS = 28.8
EXPECTED_BAND_COUNTS = {
    "sub1us": Counter({"plain2": 3}),
    "1to2p5": Counter({"plain2": 7}),
    "2p5to20": Counter({"plain2": 3}),
}


def register(operands: str) -> str:
    return operands.split(",", 1)[0].strip()


def discover_sites(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> list[tuple[str, int, str, set[int]]]:
    sites: list[tuple[str, int, str, set[int]]] = []
    for target, (start, end) in target_ranges.items():
        for address in range(start, end, 4):
            mnemonic, operands = code.get(address, ("", ""))
            if mnemonic != "add" or "x20, #0x228" not in operands:
                continue
            base = register(operands)
            next_mnemonic, next_operands = code[address + 4]
            if next_mnemonic == "ldr" and next_operands == f"q0, [{base}]":
                form = "plain2"
                pcs = set(range(address, address + 24, 4))
                expected = (
                    ("add", operands),
                    ("ldr", f"q0, [{base}]"),
                    ("mov", "w8, #0x1"),
                    ("dup.2d", "v1, x8"),
                    ("add.2d", "v0, v0, v1"),
                    ("str", f"q0, [{base}]"),
                )
                actual = tuple(code[pc] for pc in range(address, address + 24, 4))
            elif next_mnemonic == "ldp" and next_operands == f"q1, q2, [{base}]":
                form = "tournament4"
                pcs = set(range(address - 8, address + 20, 4))
                expected = (
                    ("mov", "w8, #0x1"),
                    ("dup.2d", "v0, x8"),
                    ("add", operands),
                    ("ldp", f"q1, q2, [{base}]"),
                    ("add.2d", "v1, v1, v0"),
                    ("add.2d", "v0, v2, v0"),
                    ("stp", f"q1, q0, [{base}]"),
                )
                actual = tuple(code[pc] for pc in range(address - 8, address + 20, 4))
            else:
                raise SystemExit(f"REFUSE: unknown rollback counter form at {address:#x}")
            if actual != expected:
                raise SystemExit(
                    f"REFUSE: {form} rollback-counter fingerprint differs at {address:#x}"
                )
            sites.append((target, address, form, pcs))
    return sites


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)

    sites = discover_sites(code, target_ranges)
    form_census = Counter((target, form) for target, _, form, _ in sites)
    expected_forms = Counter(
        {
            ("piece", "plain2"): 12,
            ("piece", "tournament4"): 4,
            ("atom_closure", "plain2"): 27,
            ("atom_closure", "tournament4"): 20,
        }
    )
    if form_census != expected_forms or len(sites) != 63:
        raise SystemExit(f"REFUSE: rollback-counter site census differs {form_census}")
    telemetry = set().union(*(pcs for _, _, _, pcs in sites))
    if len(telemetry) != 402:
        raise SystemExit(f"REFUSE: telemetry union differs ({len(telemetry)})")

    trace, _, _ = bare.trace_gate_pcs(code, target_ranges)
    coverage, _, _ = bare.coverage_pcs(code, target_ranges)
    nonempty, _, _ = bare.rollback_diagnostic_pcs(code, target_ranges)
    bare_union = trace | coverage | nonempty
    lookup_full, _ = thin_lookup.qualify_sites(code, target_ranges)
    prior_overlap = {address for address in telemetry if known.owner(address) is not None}
    if prior_overlap or telemetry & bare_union or telemetry & lookup_full:
        raise SystemExit(
            "REFUSE: telemetry overlap "
            f"accepted={len(prior_overlap)} bare={len(telemetry & bare_union)} "
            f"lookup={len(telemetry & lookup_full)}"
        )

    static_classes = Counter(
        capture.instruction_class(code[address][0]) for address in telemetry
    )
    expected_classes = Counter({"other": 276, "memory": 126})
    if static_classes != expected_classes:
        raise SystemExit(f"REFUSE: static class census differs {static_classes}")

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
        "source contract: rollback quartet=diagnostic predicate_evaluations=required_memo_taint"
    )
    print(
        "static sites: total=63 piece_plain2=12 piece_tournament4=4 "
        "atom_plain2=27 atom_tournament4=20"
    )
    print(
        "static telemetry: pcs=402 classes=memory:126,control:0,other:276 "
        "accepted_overlap=0 bare_0185_overlap=0 thin_lookup_overlap=0"
    )
    print(
        "forms: plain2=39x6pcs(rollbacks+unchanged) "
        "tournament4=24x7pcs(rollbacks+unchanged+tournament+tournament_unchanged)"
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
        counts: Counter[str] = Counter()
        classes: Counter[str] = Counter()
        for sampled_pc in pcs:
            address = sampled_pc - slide
            matches = [form for _, _, form, owned in sites if address in owned]
            if len(matches) > 1:
                raise SystemExit(f"REFUSE: dynamic site overlap at {address:#x}")
            if matches:
                counts[matches[0]] += 1
                classes[capture.instruction_class(code[address][0])] += 1
        if counts != EXPECTED_BAND_COUNTS[band]:
            raise SystemExit(f"REFUSE: {band} telemetry counts differ {counts}")
        raw_totals[band] = len(pcs)
        totals[band] = sum(counts.values())
        print(
            f"band={band:8} raw={len(pcs):5d} plain2={counts['plain2']:2d} "
            f"tournament4={counts['tournament4']:2d} total={totals[band]:2d} "
            f"classes=memory:{classes['memory']},control:{classes['control']},other:{classes['other']}"
        )

    share = sum(
        PROFILE_WEIGHTS[band] * totals[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    target_ns = share * FLOOR_NS
    print("\nPRICING")
    print(f"telemetry_share={100.0 * share:.9f}% telemetry_target_ns={target_ns:.9f}")
    pre_lookup_bundle = PRE_LOOKUP_HELD_BUNDLE_NS + target_ns
    optimistic_bundle = OPTIMISTIC_LOOKUP_BUNDLE_NS + target_ns
    print(
        f"pre_lookup_held_bundle_ns={pre_lookup_bundle:.9f} "
        f"capture_0.30_ns={0.30 * pre_lookup_bundle:.9f} "
        f"noise_margin_ns={0.30 * pre_lookup_bundle - NOISE_NS:.9f}"
    )
    print(
        f"optimistic_lookup_composite_ns={optimistic_bundle:.9f} "
        f"capture_0.30_ns={0.30 * optimistic_bundle:.9f} "
        f"noise_margin_ns={0.30 * optimistic_bundle - NOISE_NS:.9f}"
    )
    print(
        "observability_boundary=public SemanticRuntimeState::counters() has no "
        "pre-parse observer latch; elision requires an explicit contract"
    )
    print(
        "verdict=HOLD (telemetry alone remains 1.003506 ns short; optimistic "
        "lookup composite has only 0.143876 ns margin before omitted replacement work)"
    )
    print("SEMANTIC TELEMETRY EXPANSION: PASS")


if __name__ == "__main__":
    main()
