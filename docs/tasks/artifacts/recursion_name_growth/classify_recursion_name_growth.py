#!/usr/bin/env python3
"""Price caller-owned name-stack RawVec growth from banked call trees.

The raw-PC recursion-guard classifier deliberately excluded samples delivered
inside ``RawVec::grow_one`` and allocator descendants.  This classifier uses
the banked macOS ``sample`` call trees to recover only child time whose
immediate parent is one of four qualified inlined name-stack growth calls or
the outlined ``RecursionGuard::enter_id`` name-growth arm.
"""

from __future__ import annotations

import hashlib
import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
PROFILE_DIR = ARTIFACT_DIR.parent / "geomean_reprofile"
PROBE = REPO_ROOT / "preserved_probes" / "regex_perf_probe_c1_1d3fa0ee"
PROBE_SHA256 = "1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8"
NAME_GROW_HASH = "h1ea4d81f4bfacb92"
ID_GROW_HASH = "h84ff3e56869fffcc"
FINISH_GROW_HASH = "h0ad14a629d25f3f4"
FLOOR_NS = 1263.4
NOISE_NS = 28.8
PRE_LOOKUP_COMPOSITE_NS = 96.744420971
OPTIMISTIC_LOOKUP_COMPOSITE_NS = 100.569027701


@dataclass(frozen=True)
class Callsite:
    key: str
    nm_fragment: str
    sample_fragment: str
    symbol_start: int
    call_pc: int

    @property
    def return_offset(self) -> int:
        return self.call_pc + 4 - self.symbol_start


CALLSITES = (
    Callsite("piece_1", "cascade_match_piece17h65565a4aa37893a3", "cascade_match_piece::h65565a4aa37893a3", 0x100058D20, 0x10005A6F4),
    Callsite("piece_2", "cascade_match_piece17h65565a4aa37893a3", "cascade_match_piece::h65565a4aa37893a3", 0x100058D20, 0x10005C4D4),
    Callsite("atom_1", "h3f63d2eb4d8831ca", "h3f63d2eb4d8831ca", 0x10006ACF4, 0x100070644),
    Callsite("atom_2", "h3f63d2eb4d8831ca", "h3f63d2eb4d8831ca", 0x10006ACF4, 0x10007A4A4),
    Callsite("outlined", "RecursionGuard8enter_id17hd654801ef0b2124b", "RecursionGuard::enter_id::hd654801ef0b2124b", 0x10008DEC8, 0x10008DF58),
)
ID_CONTROL_CALL_PC = 0x10008DF84


@dataclass(frozen=True)
class Profile:
    band: str
    filename: str
    expected_sha256: str
    expected_total: int
    log_weight: float
    # key -> (occurrence count, cumulative parent samples, name-grow child samples)
    expected: dict[str, tuple[int, int, int]]


PROFILES = (
    Profile(
        "sub1us",
        "sample_band_sub1us.txt",
        "d16e7def6f05da33ac05a664b9588b8dc29648e9ec37828f3387a95f436573ed",
        11875,
        0.380,
        {
            "piece_1": (0, 0, 0),
            "piece_2": (1, 3, 3),
            "atom_1": (0, 0, 0),
            "atom_2": (0, 0, 0),
            "outlined": (1, 30, 30),
        },
    ),
    Profile(
        "1to2p5",
        "sample_band_1to2p5.txt",
        "e94a57152fe1e85cd1a66d8ca14b5707536ae36e20583c2712bcef64932098e6",
        11805,
        0.353,
        {
            "piece_1": (1, 2, 2),
            "piece_2": (1, 2, 2),
            "atom_1": (0, 0, 0),
            "atom_2": (0, 0, 0),
            "outlined": (1, 39, 39),
        },
    ),
    Profile(
        "2p5to20",
        "sample_band_2p5to20.txt",
        "1c9643db9c342f315179ff49232575423f949f724ed22dadc39bdc218b7ea788",
        10673,
        0.260,
        {
            "piece_1": (7, 11, 11),
            "piece_2": (1, 1, 1),
            "atom_1": (1, 1, 1),
            "atom_2": (0, 0, 0),
            "outlined": (1, 21, 21),
        },
    ),
)


@dataclass(frozen=True)
class Node:
    line_number: int
    depth: int
    count: int
    text: str


def require_binary_custody() -> None:
    actual_sha = hashlib.sha256(PROBE.read_bytes()).hexdigest()
    if actual_sha != PROBE_SHA256:
        raise SystemExit(f"REFUSE: probe sha256 {actual_sha} != {PROBE_SHA256}")
    for tool in ("nm", "otool"):
        if shutil.which(tool) is None:
            raise SystemExit(f"REFUSE: required tool absent: {tool}")

    nm_text = subprocess.run(
        ["nm", "-nm", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    for callsite in CALLSITES:
        start = f"{callsite.symbol_start:016x}"
        matches = [
            line
            for line in nm_text.splitlines()
            if line.startswith(start + " ") and callsite.nm_fragment in line
        ]
        if len(matches) != 1:
            raise SystemExit(
                f"REFUSE: {callsite.key} symbol/start matches {len(matches)}"
            )

    disassembly = subprocess.run(
        ["otool", "-tV", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    by_address = {line.split("\t", 1)[0]: line for line in disassembly.splitlines()}
    for callsite in CALLSITES:
        line = by_address.get(f"{callsite.call_pc:016x}", "")
        if "\tbl\t" not in line or NAME_GROW_HASH not in line:
            raise SystemExit(
                f"REFUSE: {callsite.key} is not a name grow call: {line!r}"
            )
    id_line = by_address.get(f"{ID_CONTROL_CALL_PC:016x}", "")
    if "\tbl\t" not in id_line or ID_GROW_HASH not in id_line:
        raise SystemExit(f"REFUSE: required ID grow control call differs: {id_line!r}")


def parse_call_graph(raw: bytes, profile: Profile) -> tuple[int, list[Node]]:
    actual_sha = hashlib.sha256(raw).hexdigest()
    if actual_sha != profile.expected_sha256:
        raise SystemExit(
            f"REFUSE: {profile.filename} sha256 {actual_sha} != "
            f"{profile.expected_sha256}"
        )
    lines = raw.decode("utf-8").splitlines()
    totals = [
        int(match.group(1))
        for line in lines
        if (match := re.match(r"^\s*(\d+) Thread_\d+\s+DispatchQueue_1:", line))
    ]
    if totals != [profile.expected_total]:
        raise SystemExit(
            f"REFUSE: {profile.filename} main-thread totals {totals} != "
            f"{[profile.expected_total]}"
        )
    try:
        end = next(
            index
            for index, line in enumerate(lines)
            if line.startswith("Total number in stack")
        )
    except StopIteration as error:
        raise SystemExit(f"REFUSE: no call-graph terminator in {profile.filename}") from error

    nodes: list[Node] = []
    for index, line in enumerate(lines[:end]):
        match = re.match(r"^(\D*?)(\d+) (.*)$", line)
        if match:
            nodes.append(
                Node(index + 1, len(match.group(1)), int(match.group(2)), match.group(3))
            )
    return totals[0], nodes


def classify_callsite(nodes: list[Node], callsite: Callsite) -> tuple[int, int, int]:
    parent_indexes = [
        index
        for index, node in enumerate(nodes)
        if callsite.sample_fragment in node.text
        and re.search(rf"\+ {callsite.return_offset}(?: |,)", node.text)
    ]
    parent_samples = 0
    child_samples = 0
    for parent_index in parent_indexes:
        parent = nodes[parent_index]
        parent_samples += parent.count
        direct_children: list[Node] = []
        index = parent_index + 1
        while index < len(nodes) and nodes[index].depth > parent.depth:
            if nodes[index].depth == parent.depth + 2:
                direct_children.append(nodes[index])
            index += 1
        # LLVM may tail-elide the outer RawVec frame, leaving finish_grow as an
        # immediate child of the exact call return address.  Both forms are
        # inside the uniquely qualified name-growth call; no generic allocator
        # row is admitted without that parent.
        foreign = [
            child
            for child in direct_children
            if NAME_GROW_HASH not in child.text and FINISH_GROW_HASH not in child.text
        ]
        if foreign:
            raise SystemExit(
                f"REFUSE: {callsite.key} has foreign immediate children "
                f"{[(child.line_number, child.text) for child in foreign]}"
            )
        child_samples += sum(child.count for child in direct_children)
        if child_samples > parent_samples:
            raise SystemExit(f"REFUSE: {callsite.key} child count exceeds parent count")
    return len(parent_indexes), parent_samples, child_samples


def main() -> None:
    require_binary_custody()
    observations: list[tuple[Profile, int, dict[str, tuple[int, int, int]]]] = []
    for profile in PROFILES:
        total, nodes = parse_call_graph((PROFILE_DIR / profile.filename).read_bytes(), profile)
        result = {callsite.key: classify_callsite(nodes, callsite) for callsite in CALLSITES}
        if result != profile.expected:
            raise SystemExit(
                f"REFUSE: {profile.band} callsite census {result} != {profile.expected}"
            )
        observations.append((profile, total, result))

    print(f"probe custody: PASS sha256={PROBE_SHA256}")
    print(
        "profile custody: PASS "
        + " ".join(f"{profile.band}={profile.expected_sha256}" for profile in PROFILES)
    )
    print(
        "static callsites: name_inline=4 name_outlined=1 required_id_control=1 "
        "all_exact_bl_targets=PASS"
    )
    print(
        "scope: exact name-call child subtrees (including tail-elided finish_grow); "
        "aggregate h1ea rows, required h84ff ID growth, and other Vec<24> "
        "consumers excluded"
    )
    print()
    print("CALLER-ATTRIBUTED COUNTS")
    print("band       total  inline_child  outlined_child  name_child  parent_residual")
    priced: list[tuple[Profile, int, int, int]] = []
    for profile, total, result in observations:
        inline = sum(result[key][2] for key in ("piece_1", "piece_2", "atom_1", "atom_2"))
        outlined = result["outlined"][2]
        child = inline + outlined
        parent = sum(value[1] for value in result.values())
        residual = parent - child
        priced.append((profile, total, inline, outlined))
        print(
            f"{profile.band:9} {total:5d} {inline:13d} {outlined:15d} "
            f"{child:11d} {residual:16d}"
        )
    print()
    inline_share = sum(
        profile.log_weight * inline / total
        for profile, total, inline, _ in priced
    )
    outlined_share = sum(
        profile.log_weight * outlined / total
        for profile, total, _, outlined in priced
    )
    conservative_share = inline_share + outlined_share
    weight_sum = sum(profile.log_weight for profile, _, _, _ in priced)
    normalized_share = conservative_share / weight_sum
    target_ns = conservative_share * FLOOR_NS
    honest = PRE_LOOKUP_COMPOSITE_NS + target_ns
    optimistic = OPTIMISTIC_LOOKUP_COMPOSITE_NS + target_ns
    print("PRICING")
    print(
        f"inline_share={100.0 * inline_share:.9f}% "
        f"inline_target_ns={inline_share * FLOOR_NS:.9f}"
    )
    print(
        f"outlined_share={100.0 * outlined_share:.9f}% "
        f"outlined_target_ns={outlined_share * FLOOR_NS:.9f}"
    )
    print(
        f"full_corpus_conservative_share={100.0 * conservative_share:.9f}% "
        f"target_ns={target_ns:.9f} excluded_tail=priced_at_zero"
    )
    print(
        f"covered_band_normalized_share={100.0 * normalized_share:.9f}% "
        f"target_ns={normalized_share * FLOOR_NS:.9f}"
    )
    print(
        f"pre_lookup_composite_ns={honest:.9f} capture_0.30_ns={honest * 0.30:.9f} "
        f"noise_margin_ns={honest * 0.30 - NOISE_NS:.9f}"
    )
    print(
        f"optimistic_lookup_composite_ns={optimistic:.9f} "
        f"capture_0.30_ns={optimistic * 0.30:.9f} "
        f"noise_margin_ns={optimistic * 0.30 - NOISE_NS:.9f}"
    )
    print(
        "verdict=HOLD (honest 1.425 ns margin remains too small for mandatory "
        "name reconstruction and model uncertainty)"
    )
    print("RECURSION NAME GROWTH EXPANSION: PASS")


if __name__ == "__main__":
    main()
