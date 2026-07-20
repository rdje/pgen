#!/usr/bin/env python3
"""Custody-check and classify the PGEN-RGX-0078-0182 three-band PC capture."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
PROBE = REPO_ROOT / "preserved_probes/regex_perf_probe_c1_1d3fa0ee"
PROBE_SHA256 = "1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8"
SAMPLER_SOURCE = ARTIFACT_DIR / "pc_sampler.c"
SAMPLER_SOURCE_SHA256 = "9bb51a53c4ecd5e4708925ae78f90c503f8e6511c8b46a51612c373db3c3bf85"
REFERENCE_DIR = REPO_ROOT / "docs/tasks/artifacts/geomean_reprofile"
TEXT_START = 0x100000980
TEXT_END = 0x1007CEAD8
MIN_TOTAL = 10_000
MIN_TARGET = 1_000
BANDS = (
    ("sub1us", 939, 665, 274, 165),
    ("1to2p5", 751, 678, 73, 106),
    ("2p5to20", 488, 480, 8, 50),
)


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


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def jsonl_rows(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def symbol_ranges() -> dict[str, tuple[int, int]]:
    output = subprocess.run(
        ["nm", "-nm", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    symbols: list[tuple[int, str]] = []
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 4 and re.fullmatch(r"[0-9a-f]{16}", fields[0]):
            symbols.append((int(fields[0], 16), fields[-1]))
    symbols.sort()
    ranges: dict[str, tuple[int, int]] = {}
    for target in TARGETS:
        matches = [i for i, (_, symbol) in enumerate(symbols) if target.nm_key in symbol]
        if len(matches) != 1:
            raise SystemExit(f"REFUSE: {target.label} symbol matches {matches}")
        index = matches[0]
        start = symbols[index][0]
        if start != target.expected_start:
            raise SystemExit(
                f"REFUSE: {target.label} start {start:#x} != {target.expected_start:#x}"
            )
        ranges[target.label] = (start, symbols[index + 1][0])
    return ranges


def disassembly() -> dict[int, tuple[str, str]]:
    output = subprocess.run(
        ["otool", "-tV", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    result: dict[int, tuple[str, str]] = {}
    for line in output.splitlines():
        match = DISASM_ROW.match(line)
        if match:
            result[int(match.group(1), 16)] = (match.group(2), match.group(3) or "")
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
            raise SystemExit(f"REFUSE: malformed raw line in {path.name}: {line!r}")
        if key == "pc":
            pcs.append(int(value, 16))
        elif key in header:
            raise SystemExit(f"REFUSE: duplicate raw key {key} in {path.name}")
        else:
            header[key] = value
    required = {
        "format", "pid", "timer", "interval_us", "capacity", "total_seen",
        "stored", "dropped", "forced_pending_sigprof", "pending_sigprof_drained",
        "cpu_elapsed_us", "main_slide",
    }
    if set(header) != required:
        raise SystemExit(f"REFUSE: raw header keys differ in {path.name}")
    if header["format"] != "pgen-pc-samples-v3" or header["timer"] != "ITIMER_PROF":
        raise SystemExit(f"REFUSE: raw format/timer mismatch in {path.name}")
    seen = int(header["total_seen"])
    stored = int(header["stored"])
    dropped = int(header["dropped"])
    if len(pcs) != stored or stored + dropped != seen or dropped != 0:
        raise SystemExit(f"REFUSE: raw count/drop mismatch in {path.name}")
    if header["forced_pending_sigprof"] != "0":
        raise SystemExit(f"REFUSE: teardown test hook active in {path.name}")
    if int(header["pending_sigprof_drained"]) not in (0, 1):
        raise SystemExit(f"REFUSE: impossible pending SIGPROF count in {path.name}")
    return header, pcs


def compare_verdicts(name: str, expected_count: int, expected_ok: int,
                     expected_fail: int, expected_mismatch: int) -> None:
    current = jsonl_rows(ARTIFACT_DIR / f"capture_band_{name}_times.jsonl")
    reference = jsonl_rows(REFERENCE_DIR / f"band_{name}_times.jsonl")
    if len(current) != expected_count or len(reference) != expected_count:
        raise SystemExit(f"REFUSE: {name} row count differs")
    identity_fields = ("id", "pattern_bytes", "expected_parse", "actual_parse")
    for index, (actual, banked) in enumerate(zip(current, reference, strict=True)):
        if any(actual.get(field) != banked.get(field) for field in identity_fields):
            raise SystemExit(f"REFUSE: {name} verdict identity differs at row {index + 1}")
    accepts = sum(row["actual_parse"] == "ok" for row in current)
    rejects = sum(row["actual_parse"] == "fail" for row in current)
    mismatches = sum(
        row["expected_parse"] != "unknown"
        and row["expected_parse"] != row["actual_parse"]
        for row in current
    )
    if (accepts, rejects, mismatches) != (expected_ok, expected_fail, expected_mismatch):
        raise SystemExit(f"REFUSE: {name} verdict tuple differs")
    print(
        f"verdicts: rows={len(current)} accepts={accepts} rejects={rejects} "
        f"expectation_mismatches={mismatches} identity=PASS"
    )


def main() -> None:
    if sha256(PROBE) != PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    if sha256(SAMPLER_SOURCE) != SAMPLER_SOURCE_SHA256:
        raise SystemExit("REFUSE: sampler source SHA-256 mismatch")
    ranges = symbol_ranges()
    instructions = disassembly()
    print(f"probe custody: PASS sha256={PROBE_SHA256}")
    print(f"sampler source custody: PASS sha256={SAMPLER_SOURCE_SHA256}")

    for name, count, accepts, rejects, mismatches in BANDS:
        print(f"\n=== BAND {name} ===")
        compare_verdicts(name, count, accepts, rejects, mismatches)
        header, pcs = parse_raw(ARTIFACT_DIR / f"capture_band_{name}_raw_pcs.txt")
        slide = int(header["main_slide"], 16)
        locations: Counter[str] = Counter()
        classes: Counter[str] = Counter()
        target_pcs: Counter[tuple[str, int, str, str]] = Counter()
        for sampled_pc in pcs:
            unslid = sampled_pc - slide
            label = "external_or_injected"
            if TEXT_START <= unslid < TEXT_END:
                label = "main_text_other"
                instruction = instructions.get(unslid)
                if instruction is None:
                    raise SystemExit(f"REFUSE: no pinned instruction at {unslid:#x}")
                for target, (start, end) in ranges.items():
                    if start <= unslid < end:
                        label = target
                        mnemonic, operands = instruction
                        classes[instruction_class(mnemonic)] += 1
                        target_pcs[(target, unslid - start, mnemonic, operands)] += 1
                        break
            locations[label] += 1
        target_count = sum(locations[target.label] for target in TARGETS)
        if len(pcs) < MIN_TOTAL or target_count < MIN_TARGET:
            raise SystemExit(
                f"REFUSE: {name} underfilled total={len(pcs)} target={target_count}"
            )
        if sum(locations.values()) != len(pcs) or sum(classes.values()) != target_count:
            raise SystemExit(f"REFUSE: {name} classification does not re-sum")
        effective = int(header["cpu_elapsed_us"]) / int(header["total_seen"])
        print(
            f"raw: pid={header['pid']} stored={len(pcs)} dropped=0 "
            f"pending_sigprof_drained={header['pending_sigprof_drained']} "
            f"cpu_elapsed_us={header['cpu_elapsed_us']} effective_interval_us={effective:.3f}"
        )
        for key in ("piece", "atom_closure", "main_text_other", "external_or_injected"):
            print(f"location {key:20} {locations[key]:8d} {100.0 * locations[key] / len(pcs):9.4f}%")
        print(f"target: {target_count}/{len(pcs)} = {100.0 * target_count / len(pcs):.4f}%")
        for key in ("memory", "control", "call", "other"):
            print(f"class {key:8} {classes[key]:8d} {100.0 * classes[key] / target_count:9.4f}%")
        print("target PCs (all nonzero, descending count):")
        for (target, offset, mnemonic, operands), pc_count in target_pcs.most_common():
            print(
                f"pc {pc_count:6d} {target:12} +{offset:5d} "
                f"{ranges[target][0] + offset:#011x} {mnemonic} {operands}".rstrip()
            )
    print("\nCAPTURE QUALIFICATION: PASS (counts, zero-drop, disassembly, verdict identity)")
    print("SCOPE: dynamic PC evidence only; mechanism/overlap mapping still required before ns")


if __name__ == "__main__":
    main()
