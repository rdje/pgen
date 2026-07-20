#!/usr/bin/env python3
"""Audit whether banked `sample` reports retain usable fused-spine self PCs.

PGEN-RGX-0078-0179 is deliberately read-only.  The banked reports were made by
macOS `sample`, whose normal call tree may collapse several instruction
addresses into one node.  This script inverts each call tree and measures how
much of the two largest fused-spine functions' flat self population still has
one unambiguous program counter.  It refuses to infer a distribution within a
multi-PC node.
"""

from __future__ import annotations

import hashlib
import re
import shutil
import subprocess
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
PROFILE_DIR = ARTIFACT_DIR.parent / "geomean_reprofile"
PROBE = REPO_ROOT / "preserved_probes" / "regex_perf_probe_c1_1d3fa0ee"
PROBE_SHA256 = "1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8"
UNSLID_TEXT_BASE = 0x100000000
FLOOR_NS = 1263.4


@dataclass(frozen=True)
class Target:
    label: str
    symbol: str
    nm_key: str
    expected_start: int


TARGETS = (
    Target(
        "piece",
        "pgen::generated_parsers::regex::RegexParser::cascade_match_piece::"
        "h65565a4aa37893a3",
        "19cascade_match_piece17h65565a4aa37893a3E",
        0x100058D20,
    ),
    Target(
        "atom_closure",
        "pgen::generated_parsers::regex::RegexParser::cascade_match_atom::"
        "_$u7b$$u7b$closure$u7d$$u7d$::h3f63d2eb4d8831ca",
        "18cascade_match_atom28_$u7b$$u7b$closure$u7d$$u7d$17h3f63d2eb4d8831caE",
        0x10006ACF4,
    ),
)
TARGET_BY_SYMBOL = {target.symbol: target for target in TARGETS}


@dataclass(frozen=True)
class Profile:
    filename: str
    sha256: str
    total_samples: int
    log_weight: float
    expected_self: tuple[int, int]


PROFILES = (
    Profile(
        "sample_band_sub1us.txt",
        "d16e7def6f05da33ac05a664b9588b8dc29648e9ec37828f3387a95f436573ed",
        11875,
        0.380,
        (947, 793),
    ),
    Profile(
        "sample_band_1to2p5.txt",
        "e94a57152fe1e85cd1a66d8ca14b5707536ae36e20583c2712bcef64932098e6",
        11805,
        0.353,
        (834, 825),
    ),
    Profile(
        "sample_band_2p5to20.txt",
        "1c9643db9c342f315179ff49232575423f949f724ed22dadc39bdc218b7ea788",
        10673,
        0.260,
        (843, 830),
    ),
)


ROOT_ROW = re.compile(
    r"^    (?P<count>\d+) (?P<symbol>.+?)  \(in regex_perf_probe\) \+ "
    r"(?P<offsets>.+?)  \[(?P<addresses>[^]]+)\]$"
)
LOAD_ADDRESS = re.compile(r"^Load Address:\s+(0x[0-9a-f]+)$", re.MULTILINE)
DISASM_ROW = re.compile(r"^([0-9a-f]+)\t\s*(\S+)(?:\s+(.*))?$")


@dataclass(frozen=True)
class ExactRow:
    target: Target
    offset: int
    count: int
    mnemonic: str
    operands: str


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require_custody() -> tuple[dict[str, int], dict[int, tuple[str, str]]]:
    actual_probe_sha = sha256(PROBE)
    if actual_probe_sha != PROBE_SHA256:
        raise SystemExit(
            f"REFUSE: probe sha256 {actual_probe_sha} != {PROBE_SHA256}"
        )
    for tool in ("filtercalltree", "nm", "otool"):
        if shutil.which(tool) is None:
            raise SystemExit(f"REFUSE: required tool not found: {tool}")

    nm_text = subprocess.run(
        ["nm", "-nm", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    starts: dict[str, int] = {}
    for target in TARGETS:
        matches = [
            int(line.split()[0], 16)
            for line in nm_text.splitlines()
            if target.nm_key in line
        ]
        if matches != [target.expected_start]:
            raise SystemExit(
                f"REFUSE: {target.label} symbol starts {matches} != "
                f"[{target.expected_start:#x}]"
            )
        starts[target.label] = matches[0]

    disasm_text = subprocess.run(
        ["otool", "-tV", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    disasm: dict[int, tuple[str, str]] = {}
    for line in disasm_text.splitlines():
        match = DISASM_ROW.match(line)
        if match:
            disasm[int(match.group(1), 16)] = (
                match.group(2),
                match.group(3) or "",
            )
    return starts, disasm


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


def parse_profile(
    profile: Profile,
    starts: dict[str, int],
    disasm: dict[int, tuple[str, str]],
) -> tuple[dict[str, int], list[ExactRow]]:
    path = PROFILE_DIR / profile.filename
    actual_sha = sha256(path)
    if actual_sha != profile.sha256:
        raise SystemExit(
            f"REFUSE: {profile.filename} sha256 {actual_sha} != {profile.sha256}"
        )
    text = path.read_text()
    totals = re.findall(
        r"^\s*(\d+) Thread_\d+\s+DispatchQueue_1:", text, re.MULTILINE
    )
    if totals != [str(profile.total_samples)]:
        raise SystemExit(
            f"REFUSE: {profile.filename} main-thread totals {totals} != "
            f"[{profile.total_samples}]"
        )
    load_match = LOAD_ADDRESS.search(text)
    if load_match is None:
        raise SystemExit(f"REFUSE: no load address in {profile.filename}")
    slide = int(load_match.group(1), 16) - UNSLID_TEXT_BASE

    inverted = subprocess.run(
        ["filtercalltree", str(path), "-invertCallTree", "-pruneCount", "1"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout

    self_counts: Counter[str] = Counter()
    exact_rows: list[ExactRow] = []
    for line in inverted.splitlines():
        match = ROOT_ROW.match(line)
        if match is None:
            continue
        target = TARGET_BY_SYMBOL.get(match.group("symbol"))
        if target is None:
            continue
        count = int(match.group("count"))
        self_counts[target.label] += count

        offsets = match.group("offsets")
        addresses = match.group("addresses")
        if "," in offsets or "," in addresses or "..." in offsets:
            continue
        offset = int(offsets)
        sampled_address = int(addresses, 16)
        expected_sampled_address = starts[target.label] + offset + slide
        if sampled_address != expected_sampled_address:
            raise SystemExit(
                f"REFUSE: {profile.filename} {target.label}+{offset} sampled "
                f"address {sampled_address:#x} != {expected_sampled_address:#x}"
            )
        binary_address = starts[target.label] + offset
        instruction = disasm.get(binary_address)
        if instruction is None:
            raise SystemExit(
                f"REFUSE: no instruction at {target.label}+{offset} "
                f"({binary_address:#x})"
            )
        exact_rows.append(ExactRow(target, offset, count, *instruction))

    actual_self = tuple(self_counts[target.label] for target in TARGETS)
    if actual_self != profile.expected_self:
        raise SystemExit(
            f"REFUSE: {profile.filename} target self {actual_self} != "
            f"{profile.expected_self}"
        )
    return dict(self_counts), exact_rows


def main() -> None:
    starts, disasm = require_custody()
    observed = [
        (profile, *parse_profile(profile, starts, disasm)) for profile in PROFILES
    ]

    print("custody: PASS (probe + 3/3 profiles + flat-self totals + ASLR addresses)")
    print("instrument: filtercalltree -invertCallTree -pruneCount 1")
    print("targets: cascade_match_piece + cascade_match_atom{closure}")
    print()
    print("band                         target_self  unique_pc  collapsed_pc  coverage")
    target_weighted_share = 0.0
    unique_weighted_share = 0.0
    aggregate_exact: dict[tuple[str, int, str, str], list[int]] = defaultdict(
        lambda: [0] * len(PROFILES)
    )
    for band_index, (profile, self_counts, exact_rows) in enumerate(observed):
        target_self = sum(self_counts.values())
        unique = sum(row.count for row in exact_rows)
        collapsed = target_self - unique
        target_weighted_share += (
            profile.log_weight * target_self / profile.total_samples
        )
        unique_weighted_share += (
            profile.log_weight * unique / profile.total_samples
        )
        print(
            f"{profile.filename:28} {target_self:11d}  {unique:9d}  "
            f"{collapsed:12d}  {100.0 * unique / target_self:7.4f}%"
        )
        for row in exact_rows:
            key = (row.target.label, row.offset, row.mnemonic, row.operands)
            aggregate_exact[key][band_index] += row.count

    print()
    print(
        f"target dynamic share, full-corpus conservative: "
        f"{100.0 * target_weighted_share:.4f}%"
    )
    print(
        f"uniquely attributable share of full corpus:      "
        f"{100.0 * unique_weighted_share:.4f}%"
    )
    print(
        f"unique coverage of target dynamic share:         "
        f"{100.0 * unique_weighted_share / target_weighted_share:.4f}%"
    )
    print(
        f"unresolved target dynamic share:                  "
        f"{100.0 * (target_weighted_share - unique_weighted_share):.4f}%"
    )
    print(
        f"invalid target if unique rows were called a lever: "
        f"{FLOOR_NS * unique_weighted_share:.3f} ns"
    )

    class_counts: Counter[str] = Counter()
    for (label, offset, mnemonic, operands), counts in sorted(aggregate_exact.items()):
        category = instruction_class(mnemonic)
        class_counts[category] += sum(counts)
        counts_text = "/".join(str(count) for count in counts)
        print(
            f"exact {label:12} +{offset:5d} {counts_text:>8}  "
            f"{category:7}  {mnemonic} {operands}".rstrip()
        )
    print()
    print(
        "exact-row raw class counts (sub/mid/high pooled; diagnostic only): "
        + " ".join(f"{key}={class_counts[key]}" for key in sorted(class_counts))
    )
    print("decision: INSTRUMENT-INCOMPLETE; no mechanism or nanoseconds admitted")
    print("next: validate a raw/timeline per-stack-PC sampler in a separate leaf")


if __name__ == "__main__":
    main()
