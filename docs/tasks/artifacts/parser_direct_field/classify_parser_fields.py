#!/usr/bin/env python3
"""Classify every residual x20-relative parser-memory sample from -0182.

The preserved C1 probe has no DWARF, and the current generated parser is a
different artifact vintage.  This classifier therefore treats an offset as a
candidate only: each group is admitted after a machine/dataflow fingerprint
has qualified the corresponding instructions in the custody-pinned binary.
No current-vintage address is transferred into the preserved probe.
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


PARSER_OFFSET = re.compile(r"\[x20(?:, #(?P<offset>0x[0-9a-f]+))?\]")

GROUP_OFFSETS = {
    # Bounds checks followed by byte loads through the paired data pointer.
    "input_view": frozenset({0x2F0, 0x2F8}),
    # Two Vec-like stacks plus the maximum-depth comparison in the outlined
    # recursion guard.  The machine code pushes/pops 16-byte entries in both.
    "recursion_guard": frozenset({0x0, 0x8, 0x10, 0x18, 0x20, 0x28, 0x60}),
    # Seven-word SemanticRuntimeCheckpoint source.  SmallVec's inline/spilled
    # length needs the two 0x178/0x198 loads, so eight physical words feed the
    # seven logical values.
    "semantic_checkpoint_state": frozenset(
        {0xD8, 0xF0, 0x108, 0x120, 0x150, 0x178, 0x198, 0x258}
    ),
    # Vec length/capacity/data accesses for the event and boundary tapes.
    "derivation_tapes": frozenset({0x278, 0x280, 0x288, 0x298, 0x2A0}),
    # Current parse position and its monotone furthest-position mirror.
    "position_progress": frozenset({0x498, 0x4E0}),
    # The first field is the cached logger-enabled byte.  The other two fields
    # are read immediately after it only on the enabled arm.
    "trace_gate": frozenset({0x504}),
    # Snapshot/conditional-truncate length of the otherwise-empty transactional
    # coverage vector at speculation boundaries.
    "coverage_rollback": frozenset({0x270}),
    # Hashbrown table state used by the thin-memo lookup paths.
    "thin_memo_lookup": frozenset({0x478, 0x480, 0x490}),
    # Semantic observability counters: rollback-nonempty and predicate-eval.
    "semantic_observer_counters": frozenset({0x248, 0x250}),
}

EXPECTED_PC_COUNTS = {
    0x0: 4,
    0x8: 1,
    0x10: 40,
    0x18: 1,
    0x20: 3,
    0x28: 26,
    0x60: 5,
    0xD8: 9,
    0xF0: 17,
    0x108: 6,
    0x120: 5,
    0x150: 20,
    0x178: 8,
    0x198: 4,
    0x248: 1,
    0x250: 5,
    0x258: 32,
    0x270: 23,
    0x278: 3,
    0x280: 3,
    0x288: 37,
    0x298: 1,
    0x2A0: 23,
    0x2F0: 14,
    0x2F8: 50,
    0x478: 1,
    0x480: 2,
    0x490: 5,
    0x498: 22,
    0x4E0: 16,
    0x504: 51,
}

EXPECTED_BAND_TOTALS = {"sub1us": 894, "1to2p5": 859, "2p5to20": 1081}
PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}


def parser_offset(operands: str) -> int:
    matches = list(PARSER_OFFSET.finditer(operands))
    if len(matches) != 1:
        raise SystemExit(f"REFUSE: expected one direct x20 operand: {operands}")
    return int(matches[0].group("offset") or "0", 16)


def is_parser_direct(instruction: tuple[str, str]) -> bool:
    mnemonic, operands = instruction
    return capture.instruction_class(mnemonic) == "memory" and "[x20" in operands


def group_for_offset(offset: int) -> str:
    matches = [group for group, offsets in GROUP_OFFSETS.items() if offset in offsets]
    if len(matches) != 1:
        raise SystemExit(f"REFUSE: parser offset {offset:#x} has groups {matches}")
    return matches[0]


def text_window(
    code: dict[int, tuple[str, str]], address: int, before: int, after: int
) -> str:
    return "\n".join(
        f"{code[pc][0]} {code[pc][1]}"
        for pc in range(address - before, address + after + 4, 4)
        if pc in code
    )


def qualify_machine_roles(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
    union: set[tuple[str, int]],
) -> None:
    """Refuse unless each offset group has its exclusive binary fingerprint."""

    by_offset: dict[int, set[int]] = {}
    for _, address in union:
        by_offset.setdefault(parser_offset(code[address][1]), set()).add(address)
    actual_pc_counts = {offset: len(addresses) for offset, addresses in by_offset.items()}
    if actual_pc_counts != EXPECTED_PC_COUNTS:
        raise SystemExit(
            "REFUSE: parser-direct PC census differs "
            f"actual={actual_pc_counts} expected={EXPECTED_PC_COUNTS}"
        )

    covered_offsets = set().union(*GROUP_OFFSETS.values())
    if covered_offsets != set(EXPECTED_PC_COUNTS):
        raise SystemExit("REFUSE: group offset partition differs from the exact census")

    # The cached logger-enabled byte gates both full trace_enabled() sites
    # (which then read trace_rules/active_depth) and direct logging helpers
    # (which bit-test the byte).  Every sampled access must immediately feed
    # one of those control forms.
    trace_forms: Counter[str] = Counter()
    for address in by_offset[0x504]:
        rows = [code.get(address + step, ("", "")) for step in range(0, 40, 4)]
        rendered = [f"{mnemonic} {operands}" for mnemonic, operands in rows]
        if rows[0][0] != "ldrb":
            raise SystemExit(f"REFUSE: trace-gate fingerprint differs at {address:#x}")
        near = rendered[1:8]
        if any(row.startswith(("tbz ", "tbnz ")) for row in near):
            trace_forms["direct_bit_gate"] += 1
        elif any(row.startswith("cmp ") for row in near) and any(
            row.startswith(("b.ne ", "b.eq ")) for row in near
        ):
            trace_forms[
                "full_trace_gate" if "[x20, #0x4b0]" in "\n".join(rendered) else "direct_cmp_gate"
            ] += 1
        else:
            raise SystemExit(f"REFUSE: trace-gate control use differs at {address:#x}")
    if sum(trace_forms.values()) != 51:
        raise SystemExit(f"REFUSE: trace-gate forms re-sum to {sum(trace_forms.values())}")

    # The input length is consumed by one or more bounds comparisons; the
    # paired data pointer feeds byte loads.  This proves a string/input view
    # without importing a source-vintage field name.
    if not all(code[address][0] == "ldr" for address in by_offset[0x2F8]):
        raise SystemExit("REFUSE: input-length load form differs")
    input_windows = "\n".join(
        text_window(code, address, 0, 48) for address in by_offset[0x2F8]
    )
    if input_windows.count("[x20, #0x2f0]") < 40 or input_windows.count("ldrb") < 40:
        raise SystemExit("REFUSE: input length/data/byte-load fingerprint differs")
    if not all(code[address][0] == "ldr" for address in by_offset[0x2F0]):
        raise SystemExit("REFUSE: input-data load form differs")

    # The monotone position pair appears as load/load/compare/(conditional)
    # store at every sampled furthest-position site.
    for address in by_offset[0x4E0]:
        window = text_window(code, address, 8, 16)
        if (
            "cmp" not in window
            or "#0x4e0" not in window
            or ("#0x498" not in window and window.count("#0x4e0") < 2)
        ):
            raise SystemExit(f"REFUSE: position-progress fingerprint differs at {address:#x}")

    # Reuse the already-qualified 39-site checkpoint detector.  Thirty-five
    # sites reload every physical checkpoint source in the immediate pre-gate
    # window; four keep some values live in registers from earlier code.
    _, checkpoint_sites = stack_carriers.checkpoint_store_pcs(code, target_ranges)
    checkpoint_sources = GROUP_OFFSETS["semantic_checkpoint_state"]
    fully_reloaded = 0
    for _, guard, *_ in checkpoint_sites:
        loaded = {
            parser_offset(code[address][1])
            for address in range(guard - 96, guard, 4)
            if address in code and is_parser_direct(code[address])
        }
        if checkpoint_sources <= loaded:
            fully_reloaded += 1
    if fully_reloaded != 35:
        raise SystemExit(
            f"REFUSE: checkpoint source fingerprint differs ({fully_reloaded}/39 full)"
        )

    # The two guard vectors are manipulated as 16-byte element stacks, and
    # the third field is compared as the recursion-depth ceiling.
    guard_text = "\n".join(
        text_window(code, address, 20, 24)
        for offset in GROUP_OFFSETS["recursion_guard"]
        for address in by_offset[offset]
    )
    for token in ("#0x10", "#0x18", "#0x20", "#0x28", "#0x60", "lsl #4"):
        if token not in guard_text:
            raise SystemExit(f"REFUSE: recursion-guard fingerprint lacks {token}")

    # Both derivation tapes have the Vec cap/data/len grow/push/copy shape.
    tape_text = "\n".join(
        text_window(code, address, 20, 28)
        for offset in GROUP_OFFSETS["derivation_tapes"]
        for address in by_offset[offset]
    )
    for token in ("#0x278", "#0x280", "#0x288", "#0x298", "#0x2a0", "grow_one"):
        if token not in tape_text:
            raise SystemExit(f"REFUSE: derivation-tape fingerprint lacks {token}")

    # Coverage is a length-only speculation snapshot/truncate: every sampled
    # access is a load and rollback windows compare then conditionally store.
    if not all(code[address][0] == "ldr" for address in by_offset[0x270]):
        raise SystemExit("REFUSE: coverage-length load form differs")
    coverage_text = "\n".join(
        text_window(code, address, 0, 16) for address in by_offset[0x270]
    )
    if coverage_text.count("str") < 15 or coverage_text.count("cmp") < 15:
        raise SystemExit("REFUSE: coverage snapshot/truncate fingerprint differs")

    # Five table-occupancy loads enter the same hash mixing sequence; the
    # nearby base-field accesses call hashbrown get.  This is the residual
    # thin-memo lookup, not the already-owned success-segment insertion range.
    for address in by_offset[0x490]:
        window = text_window(code, address, 0, 40)
        if code[address][0] != "ldr" or "cbz" not in window or "#0xa9c5" not in window:
            raise SystemExit(f"REFUSE: thin-memo lookup fingerprint differs at {address:#x}")
    for offset in (0x478, 0x480):
        for address in by_offset[offset]:
            starts = [
                start for start in by_offset[0x490] if 0 < address - start <= 0x80
            ]
            if len(starts) != 1 or "#0xa9c5" not in text_window(
                code, starts[0], 0, 40
            ):
                raise SystemExit(
                    f"REFUSE: thin-memo table-field fingerprint differs at {address:#x}"
                )

    # 0x248 is incremented in place; 0x250 is snapshotted and compared across
    # speculative execution.  They are observability counters, not state that
    # affects the parse result.
    counter_text = "\n".join(
        text_window(code, address, 8, 16)
        for offset in (0x248, 0x250)
        for address in by_offset[offset]
    )
    if "add" not in counter_text or "cmp" not in counter_text:
        raise SystemExit("REFUSE: semantic observer-counter fingerprint differs")

    print(
        "machine qualification: PASS "
        "input=bounds+byte-load recursion=two-stacks+depth "
        "checkpoint=35-full/39-sites tapes=two-vecs "
        f"position=monotone trace=51-gates/{dict(trace_forms)} coverage=snapshot/truncate "
        "thin_memo=5-hash-lookups counters=increment/snapshot"
    )


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)

    band_counts: dict[str, Counter[tuple[str, int]]] = {}
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: {band} raw SHA-256 mismatch")
        header, pcs = capture.parse_raw(raw)
        slide = int(header["main_slide"], 16)
        counts: Counter[tuple[str, int]] = Counter()
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
            if is_parser_direct(code[address]):
                counts[(target, address)] += 1
        band_counts[band] = counts

    union = set().union(*(counts for counts in band_counts.values()))
    qualify_machine_roles(code, target_ranges, union)

    category_counts: dict[str, Counter[str]] = {}
    raw_totals: dict[str, int] = {}
    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print(
        "vintage boundary: preserved=C1/cb322b75 current=G1-A/e4924024 "
        "address_transfer=REFUSED"
    )
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        _, raw_pcs = capture.parse_raw(raw)
        raw_totals[band] = len(raw_pcs)
        counts: Counter[str] = Counter()
        for (_, address), samples in band_counts[band].items():
            offset = parser_offset(code[address][1])
            counts[group_for_offset(offset)] += samples
        total = sum(counts.values())
        if total != EXPECTED_BAND_TOTALS[band]:
            raise SystemExit(
                f"REFUSE: {band} parser-direct re-sum {total} "
                f"!= {EXPECTED_BAND_TOTALS[band]}"
            )
        category_counts[band] = counts
        print(f"\n=== BAND {band} ===")
        for category in GROUP_OFFSETS:
            value = counts[category]
            print(
                f"field_role {category:28} {value:4d} "
                f"parser_direct={100.0 * value / total:8.4f}%"
            )
        expression = "+".join(str(counts[category]) for category in GROUP_OFFSETS)
        print(f"resum {expression}={total} PASS")

    print("\nOFFSET MAP (qualified preserved-probe offsets; no DWARF transfer)")
    for category, offsets in GROUP_OFFSETS.items():
        rendered = ",".join(f"{offset:#x}" for offset in sorted(offsets))
        print(f"offsets {category:28} {rendered}")

    diagnostic_groups = (
        "trace_gate",
        "coverage_rollback",
        "semantic_observer_counters",
    )
    diagnostic_counts = {
        band: sum(category_counts[band][group] for group in diagnostic_groups)
        for band in category_counts
    }
    weighted_share = sum(
        PROFILE_WEIGHTS[band] * diagnostic_counts[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    print("\nADDRESSABLE DIAGNOSTIC LOWER BOUND")
    print("groups=" + "+".join(diagnostic_groups))
    print(
        "counts="
        + " ".join(f"{band}:{diagnostic_counts[band]}" for band in PROFILE_WEIGHTS)
    )
    print(f"weighted_all_sample_share={100.0 * weighted_share:.6f}%")
    print(
        "scope=direct-load/store samples only; companion control/formatting/hash "
        "instructions remain outside this leaf"
    )
    print("new_nanoseconds=0.000 (mechanism expansion required before pricing)")
    print("PARSER-DIRECT FIELD ATTRIBUTION: PASS (complete re-sum; zero overlap)")


if __name__ == "__main__":
    main()
