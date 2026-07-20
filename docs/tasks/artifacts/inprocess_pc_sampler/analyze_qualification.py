#!/usr/bin/env python3
"""Custody-check and analyze the PGEN-RGX-0078-0181 PC-sampler run."""

from __future__ import annotations

import argparse
import hashlib
import math
import re
import subprocess
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
PROBE = REPO_ROOT / "preserved_probes" / "regex_perf_probe_c1_1d3fa0ee"
PROBE_SHA256 = "1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8"
TEXT_START = 0x100000980
TEXT_END = 0x1007CEAD8
REQUIRED_RETENTION = 0.95


@dataclass(frozen=True)
class Target:
    label: str
    nm_key: str
    expected_start: int


TARGETS = (
    Target("piece", "19cascade_match_piece17h65565a4aa37893a3E", 0x100058D20),
    Target(
        "atom_closure",
        "18cascade_match_atom28_$u7b$$u7b$closure$u7d$$u7d$17h3f63d2eb4d8831caE",
        0x10006ACF4,
    ),
)
DISASM_ROW = re.compile(r"^([0-9a-f]+)\t\s*(\S+)(?:\s+(.*))?$")
BENCH_ROW = re.compile(
    r"^(?P<name>[a-z_]+)\s+(?P<minimum>\d+)\s+(?P<p50>\d+)\s+"
    r"(?P<mean>\d+)\s+(?P<p99>\d+)\s+(?P<maximum>\d+)\s+(?P<samples>\d+)$"
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def symbol_ranges() -> dict[str, tuple[int, int]]:
    nm_text = subprocess.run(
        ["nm", "-nm", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    symbols: list[tuple[int, str]] = []
    for line in nm_text.splitlines():
        fields = line.split()
        if len(fields) >= 4 and re.fullmatch(r"[0-9a-f]{16}", fields[0]):
            symbols.append((int(fields[0], 16), fields[-1]))
    symbols.sort()
    result: dict[str, tuple[int, int]] = {}
    for target in TARGETS:
        matches = [
            index for index, (_, symbol) in enumerate(symbols) if target.nm_key in symbol
        ]
        if len(matches) != 1:
            raise SystemExit(f"REFUSE: {target.label} symbol matches {matches}")
        index = matches[0]
        start = symbols[index][0]
        end = symbols[index + 1][0]
        if start != target.expected_start:
            raise SystemExit(
                f"REFUSE: {target.label} start {start:#x} != {target.expected_start:#x}"
            )
        result[target.label] = (start, end)
    return result


def disassembly() -> dict[int, tuple[str, str]]:
    text = subprocess.run(
        ["otool", "-tV", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    result: dict[int, tuple[str, str]] = {}
    for line in text.splitlines():
        match = DISASM_ROW.match(line)
        if match:
            result[int(match.group(1), 16)] = (
                match.group(2),
                match.group(3) or "",
            )
    return result


def instruction_class(mnemonic: str) -> str:
    if mnemonic in {"bl", "blr"}:
        return "call"
    if mnemonic.startswith(("ldr", "str", "ldp", "stp", "ldur", "stur", "ld1", "st1")):
        return "memory"
    if mnemonic.startswith(("b.", "cb", "tb")) or mnemonic in {
        "b", "br", "ret", "cmp", "cmn", "tst", "ccmp", "csel", "cset",
    }:
        return "control"
    return "other"


def parse_raw(path: Path) -> tuple[dict[str, str], list[int]]:
    header: dict[str, str] = {}
    pcs: list[int] = []
    for line in path.read_text().splitlines():
        key, separator, value = line.partition("=")
        if not separator:
            raise SystemExit(f"REFUSE: malformed raw line {line!r}")
        if key == "pc":
            pcs.append(int(value, 16))
        elif key in header:
            raise SystemExit(f"REFUSE: duplicate raw header key {key}")
        else:
            header[key] = value
    required = {
        "format", "pid", "timer", "interval_us", "capacity", "total_seen",
        "stored", "dropped", "cpu_elapsed_us", "main_slide",
    }
    if set(header) != required:
        raise SystemExit(f"REFUSE: raw header keys {set(header)} != {required}")
    if header["format"] != "pgen-pc-samples-v2" or header["timer"] != "ITIMER_PROF":
        raise SystemExit("REFUSE: raw format/timer mismatch")
    stored = int(header["stored"])
    total_seen = int(header["total_seen"])
    dropped = int(header["dropped"])
    if len(pcs) != stored or stored + dropped != total_seen:
        raise SystemExit(
            f"REFUSE: pc lines/stored/dropped do not re-sum: "
            f"{len(pcs)}/{stored}/{dropped}/{total_seen}"
        )
    return header, pcs


def parse_bench(path: Path) -> dict[str, dict[str, int]]:
    rows: dict[str, dict[str, int]] = {}
    for line in path.read_text().splitlines():
        match = BENCH_ROW.match(line.strip())
        if match:
            values = match.groupdict()
            name = values.pop("name")
            rows[name] = {key: int(value) for key, value in values.items()}
    if len(rows) != 8:
        raise SystemExit(f"REFUSE: expected 8 bench rows in {path}, found {len(rows)}")
    return rows


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--injected", type=Path, required=True)
    parser.add_argument("--sampler-sha256", required=True)
    args = parser.parse_args()

    actual_probe_sha = sha256(PROBE)
    if actual_probe_sha != PROBE_SHA256:
        raise SystemExit(
            f"REFUSE: probe sha256 {actual_probe_sha} != {PROBE_SHA256}"
        )
    if not re.fullmatch(r"[0-9a-f]{64}", args.sampler_sha256):
        raise SystemExit("REFUSE: malformed sampler dylib SHA-256")

    ranges = symbol_ranges()
    instructions = disassembly()
    header, pcs = parse_raw(args.raw)
    slide = int(header["main_slide"], 16)

    location_counts: Counter[str] = Counter()
    target_classes: Counter[str] = Counter()
    target_offsets: Counter[tuple[str, int, str, str]] = Counter()
    for sampled_pc in pcs:
        unslid = sampled_pc - slide
        label = "external_or_injected"
        if TEXT_START <= unslid < TEXT_END:
            label = "main_text_other"
            instruction = instructions.get(unslid)
            if instruction is None:
                raise SystemExit(f"REFUSE: no pinned instruction at main PC {unslid:#x}")
            for target, (start, end) in ranges.items():
                if start <= unslid < end:
                    label = target
                    mnemonic, operands = instruction
                    category = instruction_class(mnemonic)
                    target_classes[category] += 1
                    target_offsets[(target, unslid - start, mnemonic, operands)] += 1
                    break
        location_counts[label] += 1

    if sum(location_counts.values()) != len(pcs):
        raise SystemExit("REFUSE: location counts do not re-sum")
    target_count = sum(location_counts[target.label] for target in TARGETS)
    if target_count == 0:
        raise SystemExit("REFUSE: no target samples captured")
    retained_target = sum(target_classes.values())
    retention = retained_target / target_count
    if retention < REQUIRED_RETENTION:
        raise SystemExit(
            f"REFUSE: target one-PC retention {retention:.6f} < {REQUIRED_RETENTION}"
        )

    baseline = parse_bench(args.baseline)
    injected = parse_bench(args.injected)
    if set(baseline) != set(injected):
        raise SystemExit("REFUSE: baseline/injected pattern sets differ")
    p50_ratios = [injected[name]["p50"] / baseline[name]["p50"] for name in baseline]
    p50_geomean_ratio = math.exp(sum(math.log(value) for value in p50_ratios) / len(p50_ratios))

    print(f"probe custody: PASS sha256={actual_probe_sha}")
    print(f"sampler dylib sha256: {args.sampler_sha256}")
    print(
        f"raw: pid={header['pid']} interval_us={header['interval_us']} "
        f"seen={header['total_seen']} stored={header['stored']} dropped={header['dropped']} "
        f"slide={header['main_slide']}"
    )
    effective_interval_us = int(header["cpu_elapsed_us"]) / int(header["total_seen"])
    print(
        f"delivery: cpu_elapsed_us={header['cpu_elapsed_us']} "
        f"effective_interval_us={effective_interval_us:.3f}"
    )
    print("location re-sum:")
    for key in ("piece", "atom_closure", "main_text_other", "external_or_injected"):
        print(f"  {key:20} {location_counts[key]:8d}  {100.0 * location_counts[key] / len(pcs):8.4f}%")
    print(f"  {'TOTAL':20} {sum(location_counts.values()):8d}  {100.0:8.4f}%")
    print(
        f"target one-PC retention: {retained_target}/{target_count} = "
        f"{100.0 * retention:.4f}% (required >= {100.0 * REQUIRED_RETENTION:.1f}%)"
    )
    print("target instruction classes:")
    for key in ("memory", "control", "call", "other"):
        print(f"  {key:8} {target_classes[key]:8d}  {100.0 * target_classes[key] / target_count:8.4f}%")
    print("top target PCs:")
    for (target, offset, mnemonic, operands), count in target_offsets.most_common(20):
        print(f"  {count:6d}  {target:12} +{offset:5d}  {mnemonic} {operands}".rstrip())
    print(f"observer p50 geomean ratio injected/base: {p50_geomean_ratio:.6f}")
    for name in baseline:
        ratio = injected[name]["p50"] / baseline[name]["p50"]
        print(
            f"  {name:24} {baseline[name]['p50']:6d} -> "
            f"{injected[name]['p50']:6d} ns  {ratio:.6f}x"
        )
    print("qualification: PASS (raw PCs re-sum; target retention clears 95%)")
    print("scope: qualification workload only; no corpus nanoseconds priced")


if __name__ == "__main__":
    main()
