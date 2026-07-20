#!/usr/bin/env python3
"""Census exact residual stack-memory PCs from the -0182 band capture."""

from __future__ import annotations

import sys
import re
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
CAPTURE_DIR = ARTIFACT_DIR.parent / "three_band_pc_capture"
sys.path.insert(0, str(CAPTURE_DIR))

import analyze_capture as capture  # noqa: E402
import classify_mechanisms as known  # noqa: E402


STACK_OFFSET = re.compile(r"\[(?:sp|x29)(?:, #(?P<offset>-?0x[0-9a-f]+))?")


def is_stack_memory(instruction: tuple[str, str]) -> bool:
    mnemonic, operands = instruction
    return (
        capture.instruction_class(mnemonic) == "memory"
        and ("[sp" in operands or "[x29" in operands)
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
            if is_stack_memory(code[address]):
                counts[(target, address)] += 1
        band_counts[band] = counts

    union = set().union(*(counts for counts in band_counts.values()))
    print(f"probe_sha256={capture.PROBE_SHA256}")
    print("bands=" + ",".join(band_counts))
    print("rows sorted by aggregate dynamic count")
    for target, address in sorted(
        union,
        key=lambda item: (
            -sum(counts[item] for counts in band_counts.values()),
            item[0],
            item[1],
        ),
    ):
        mnemonic, operands = code[address]
        offset = address - target_ranges[target][0]
        values = " ".join(f"{band}={band_counts[band][(target, address)]}" for band in band_counts)
        print(
            f"pc target={target:12} address={address:#011x} offset={offset:5d} "
            f"total={sum(counts[(target, address)] for counts in band_counts.values()):4d} "
            f"{values} | {mnemonic} {operands}".rstrip()
        )

    print("\ncontext for PCs with aggregate count >= 10")
    for target, address in sorted(
        union,
        key=lambda item: (
            -sum(counts[item] for counts in band_counts.values()),
            item[0],
            item[1],
        ),
    ):
        total = sum(counts[(target, address)] for counts in band_counts.values())
        if total < 10:
            continue
        print(f"\ncontext target={target} address={address:#x} total={total}")
        for nearby in range(address - 24, address + 28, 4):
            if nearby in code:
                marker = ">" if nearby == address else " "
                mnemonic, operands = code[nearby]
                print(f"{marker} {nearby:#011x} {mnemonic:8} {operands}".rstrip())

    print("\nstack-slot aggregate")
    slot_counts: Counter[tuple[str, str]] = Counter()
    slot_pcs: dict[tuple[str, str], set[int]] = {}
    for target, address in union:
        match = STACK_OFFSET.search(code[address][1])
        if match is None:
            raise SystemExit(f"REFUSE: no stack offset at {address:#x}")
        offset = match.group("offset") or "0x0"
        key = (target, offset)
        slot_counts[key] += sum(counts[(target, address)] for counts in band_counts.values())
        slot_pcs.setdefault(key, set()).add(address)
    for (target, offset), total in slot_counts.most_common():
        print(
            f"slot target={target:12} offset={offset:>6} total={total:4d} "
            f"pcs={len(slot_pcs[(target, offset)])}"
        )

    print("\nre-sum")
    for band, counts in band_counts.items():
        print(f"band={band} stack_residual={sum(counts.values())}")


if __name__ == "__main__":
    main()
