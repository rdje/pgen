#!/usr/bin/env python3
"""Inspect every preserved-probe input-view carrier load and its consumers."""

from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
FIELD_DIR = TASK_ARTIFACTS / "parser_direct_field"
sys.path[:0] = [str(CAPTURE_DIR), str(FIELD_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_parser_fields as fields  # noqa: E402


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
            if fields.is_parser_direct(code[address]):
                counts[(target, address)] += 1
        band_counts[band] = counts

    union = set().union(*(counts for counts in band_counts.values()))
    fields.qualify_machine_roles(code, target_ranges, union)
    input_sites = sorted(
        (target, address)
        for target, address in union
        if fields.parser_offset(code[address][1]) in {0x2F0, 0x2F8}
    )
    all_input_sites = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if address in code
        and fields.is_parser_direct(code[address])
        and fields.parser_offset(code[address][1]) in {0x2F0, 0x2F8}
        and known.owner(address) is None
    }
    sampled_addresses = {address for _, address in input_sites}
    print(
        f"input_sites={len(input_sites)} all_machine_sites={len(all_input_sites)} "
        f"all_len={sum(fields.parser_offset(code[address][1]) == 0x2F8 for address in all_input_sites)} "
        f"all_data={sum(fields.parser_offset(code[address][1]) == 0x2F0 for address in all_input_sites)} "
        f"unsampled={len(all_input_sites - sampled_addresses)}"
    )
    data_failures = []
    for address in sorted(all_input_sites):
        if fields.parser_offset(code[address][1]) != 0x2F0:
            continue
        destination = code[address][1].split(",", 1)[0]
        following = [
            code.get(candidate, ("", ""))
            for candidate in range(address + 4, address + 36, 4)
        ]
        aliases = {destination}
        for child_mnemonic, child_operands in following:
            add_match = re.fullmatch(r"(x\d+), (x\d+), .+", child_operands)
            if child_mnemonic == "add" and add_match and add_match.group(2) in aliases:
                aliases.add(add_match.group(1))
        if not any(
            child_mnemonic.startswith(("ldr", "ldur"))
            and any(f"[{alias}" in child_operands for alias in aliases)
            for child_mnemonic, child_operands in following
        ):
            data_failures.append(address)
    print(
        "data_consumer_failures="
        + ",".join(f"{address:#x}" for address in data_failures)
    )
    for target, address in input_sites:
        mnemonic, operands = code[address]
        register_match = re.match(r"(x\d+),", operands)
        if register_match is None:
            raise SystemExit(f"REFUSE: load destination absent at {address:#x}")
        register = register_match.group(1)
        counts = "/".join(
            str(band_counts[band][(target, address)]) for band, *_ in capture.BANDS
        )
        print(
            f"\nsite target={target} address={address:#x} counts={counts} "
            f"offset={fields.parser_offset(operands):#x} dest={register}"
        )
        for candidate in range(address, address + 52, 4):
            child_mnemonic, child_operands = code.get(candidate, ("", ""))
            marker = ">" if candidate == address else " "
            print(f"{marker} {candidate:#x} {child_mnemonic:8} {child_operands}")


if __name__ == "__main__":
    main()
