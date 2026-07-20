#!/usr/bin/env python3
"""Align preserved-probe target PCs to a DWARF twin and census source lines."""

from __future__ import annotations

import argparse
import difflib
import hashlib
import re
import subprocess
import tempfile
from collections import Counter
from pathlib import Path

import analyze_capture as capture


ARTIFACT_DIR = Path(__file__).resolve().parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
GENERATED_REGEX = REPO_ROOT / "generated/regex_parser.rs"
GENERATED_REGEX_SHA256 = "e4924024a4bf7a91a75bb8b0dfb8460c71f19ef83e08313661fbd5d4e1ad590f"
BANDS = ("sub1us", "1to2p5", "2p5to20")
ABSOLUTE_ADDRESS = re.compile(r"(?<!#)0x[0-9a-f]+")
SOURCE_SUFFIX = re.compile(r" \((?P<path>.+):(?P<line>\d+)\)$")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def symbol_range(binary: Path, target: capture.Target) -> tuple[int, int]:
    output = subprocess.run(
        ["nm", "-nm", str(binary)], check=True, capture_output=True, text=True
    ).stdout
    symbols: list[tuple[int, str]] = []
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 4 and re.fullmatch(r"[0-9a-f]{16}", fields[0]):
            symbols.append((int(fields[0], 16), fields[-1]))
    symbols.sort()
    matches = [index for index, (_, symbol) in enumerate(symbols) if target.nm_key in symbol]
    if len(matches) != 1:
        raise SystemExit(f"REFUSE: {target.label} debug symbol matches {matches}")
    index = matches[0]
    return symbols[index][0], symbols[index + 1][0]


def disassembly(binary: Path) -> dict[int, tuple[str, str]]:
    output = subprocess.run(
        ["otool", "-tV", str(binary)], check=True, capture_output=True, text=True
    ).stdout
    result: dict[int, tuple[str, str]] = {}
    for line in output.splitlines():
        match = capture.DISASM_ROW.match(line)
        if match:
            result[int(match.group(1), 16)] = (match.group(2), match.group(3) or "")
    return result


def normalized(instruction: tuple[str, str]) -> tuple[str, str]:
    mnemonic, operands = instruction
    return mnemonic, ABSOLUTE_ADDRESS.sub("<address>", operands)


def align_target(
    target: capture.Target,
    preserved_disasm: dict[int, tuple[str, str]],
    debug_disasm: dict[int, tuple[str, str]],
    debug_range: tuple[int, int],
) -> tuple[dict[int, int], int, int]:
    preserved_start = target.expected_start
    preserved_end = capture.symbol_ranges()[target.label][1]
    preserved_addresses = sorted(
        address for address in preserved_disasm if preserved_start <= address < preserved_end
    )
    debug_addresses = sorted(
        address for address in debug_disasm if debug_range[0] <= address < debug_range[1]
    )
    preserved_sequence = [normalized(preserved_disasm[address]) for address in preserved_addresses]
    debug_sequence = [normalized(debug_disasm[address]) for address in debug_addresses]
    matcher = difflib.SequenceMatcher(
        None, preserved_sequence, debug_sequence, autojunk=False
    )
    mapping: dict[int, int] = {}
    for block in matcher.get_matching_blocks():
        for delta in range(block.size):
            old_address = preserved_addresses[block.a + delta]
            new_address = debug_addresses[block.b + delta]
            if normalized(preserved_disasm[old_address]) != normalized(debug_disasm[new_address]):
                raise SystemExit("REFUSE: alignment admitted unequal instructions")
            mapping[old_address] = new_address
    return mapping, len(preserved_addresses), len(debug_addresses)


def atos_lines(debug_probe: Path, addresses: list[int]) -> dict[int, str]:
    with tempfile.NamedTemporaryFile("w", prefix="pgen-atos-0182-", delete=True) as handle:
        for address in addresses:
            handle.write(f"{address:#x}\n")
        handle.flush()
        output = subprocess.run(
            [
                "atos", "-o", str(debug_probe), "-l", "0x100000000",
                "-fullPath", "-f", handle.name,
            ],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    if len(output) != len(addresses):
        raise SystemExit(f"REFUSE: atos rows {len(output)} != addresses {len(addresses)}")
    return dict(zip(addresses, output, strict=True))


def normalize_source(symbolized: str) -> tuple[str, int, str]:
    match = SOURCE_SUFFIX.search(symbolized)
    if match is None:
        return "<no-source>", 0, symbolized
    path = Path(match.group("path"))
    try:
        relative = path.resolve().relative_to(REPO_ROOT)
        label = str(relative)
    except ValueError:
        label = str(path)
    line_number = int(match.group("line"))
    source_text = "<unavailable>"
    source_path = REPO_ROOT / label
    if source_path.is_file():
        lines = source_path.read_text().splitlines()
        if 1 <= line_number <= len(lines):
            source_text = lines[line_number - 1].strip()
    return label, line_number, source_text


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--debug-probe", type=Path, required=True)
    args = parser.parse_args()
    debug_probe = args.debug_probe.resolve()
    if not debug_probe.is_file():
        raise SystemExit(f"REFUSE: missing debug probe {debug_probe}")
    if sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    if sha256(GENERATED_REGEX) != GENERATED_REGEX_SHA256:
        raise SystemExit("REFUSE: generated regex SHA-256 mismatch")

    preserved_disasm = disassembly(capture.PROBE)
    debug_disasm = disassembly(debug_probe)
    mappings: dict[str, dict[int, int]] = {}
    ranges = capture.symbol_ranges()
    print(f"preserved probe sha256={capture.PROBE_SHA256}")
    print(f"debug probe sha256={sha256(debug_probe)}")
    print(f"generated regex sha256={GENERATED_REGEX_SHA256}")
    for target in capture.TARGETS:
        debug_range = symbol_range(debug_probe, target)
        mapping, preserved_count, debug_count = align_target(
            target, preserved_disasm, debug_disasm, debug_range
        )
        mappings[target.label] = mapping
        print(
            f"alignment {target.label}: preserved_instructions={preserved_count} "
            f"debug_instructions={debug_count} exact_aligned={len(mapping)} "
            f"static_coverage={100.0 * len(mapping) / preserved_count:.4f}%"
        )

    band_pc_counts: dict[str, Counter[tuple[str, int]]] = {}
    all_mapped_addresses: set[int] = set()
    for band in BANDS:
        header, pcs = capture.parse_raw(ARTIFACT_DIR / f"capture_band_{band}_raw_pcs.txt")
        slide = int(header["main_slide"], 16)
        counts: Counter[tuple[str, int]] = Counter()
        for sampled_pc in pcs:
            unslid = sampled_pc - slide
            for label, (start, end) in ranges.items():
                if start <= unslid < end:
                    counts[(label, unslid)] += 1
                    mapped = mappings[label].get(unslid)
                    if mapped is not None:
                        all_mapped_addresses.add(mapped)
                    break
        band_pc_counts[band] = counts

    symbols = atos_lines(debug_probe, sorted(all_mapped_addresses))
    for band in BANDS:
        source_counts: Counter[tuple[str, int, str]] = Counter()
        target_total = sum(band_pc_counts[band].values())
        mapped_total = 0
        for (label, address), count in band_pc_counts[band].items():
            mapped = mappings[label].get(address)
            if mapped is None:
                continue
            mapped_total += count
            source_counts[normalize_source(symbols[mapped])] += count
        print(f"\n=== BAND {band} SOURCE MAP ===")
        print(
            f"dynamic exact-alignment coverage={mapped_total}/{target_total} "
            f"= {100.0 * mapped_total / target_total:.4f}%"
        )
        for (path, line, source), count in source_counts.most_common():
            print(
                f"source {count:6d} {100.0 * count / target_total:9.4f}% "
                f"{path}:{line} | {source}"
            )


if __name__ == "__main__":
    main()
