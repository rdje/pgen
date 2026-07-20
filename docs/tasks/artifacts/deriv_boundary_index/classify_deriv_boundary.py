#!/usr/bin/env python3
"""Audit lossless index feasibility for the derivation-boundary tape.

The strict price is replacement-aware: narrowing a pointer load/store to an
index preserves the instruction and adds reconstruction.  Tournament copy and
constructor-allocation child time are reported only as optimistic ceilings.
Already-owned G1-C push/growth and thin-memo success ranges receive no credit.
"""

from __future__ import annotations

import hashlib
import io
import re
import subprocess
import sys
from contextlib import redirect_stdout
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
CALL_TREE_DIR = TASK_ARTIFACTS / "recursion_name_growth"
INPUT_DIR = TASK_ARTIFACTS / "input_view_carrier"
EVENT_DIR = TASK_ARTIFACTS / "deriv_event_width"
POSITION_DIR = TASK_ARTIFACTS / "position_progress_carrier"
sys.path[:0] = [
    str(CAPTURE_DIR),
    str(CALL_TREE_DIR),
    str(INPUT_DIR),
    str(EVENT_DIR),
    str(POSITION_DIR),
]

import analyze_capture as capture  # noqa: E402
import classify_deriv_event_width as event_width  # noqa: E402
import classify_input_view_carrier as input_view  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_position_progress as position_progress  # noqa: E402
import classify_recursion_name_growth as call_tree  # noqa: E402


FLOOR_NS = 1263.4
NOISE_NS = 28.8
PRE_BOUNDARY_STRICT_NS = 102.138976561
PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
SOURCE_PINS = {
    "rust/src/ast_pipeline/mod.rs": "031e8f31407c171d93dd699d5907fd025ff126a4405d8e498174a87e6d6efbd8",
    "rust/src/ast_pipeline/ast_based_generator.rs": "b5ed80b80adfcf1a6fb5df38884e249e06eff2f32a5e78dfef8760d256f374df",
    "rust/src/ast_pipeline/ast_based_generator/cascade.rs": "413fefe99dbfe8d4a03886f4eb4999d078fb03696f861ee1e7fefc23db03a100",
    "rust/Cargo.lock": "a5dbc4dc0b981f756fa175df7faa132e8d1fe1fb478a3afe1c449886aa1096c6",
}
RUSTC_VERSION = "rustc 1.95.0 (59807616e 2026-04-14)"
LAYOUT_SOURCE_SHA256 = "3f4d4c05afe5b6a4d05e9942607248f27ef4b5ff6e60d7ef810bf89c3a3fc784"
EXPECTED_LAYOUT_OUTPUT = (
    "BoundaryRef size=8 align=8 needs_drop=false\n"
    "BoundaryIndex32 size=4 align=4 needs_drop=false\n"
    "BoundaryIndexUsize size=8 align=8 needs_drop=false\n"
    "usize_bits=64\n"
)

PIECE_START = 0x100058D20
ATOM_START = 0x10006ACF4
PARSE_ONCE_START = 0x10000179C
BOUNDARY_COPY_HASH = "h25a2bd99ab497b65"
BOUNDARY_GROW_HASH = "h1f5402799ab88b1e"
BOUNDARY_ALLOC_CALL = 0x100001E24

BOUNDARY_GROW_CALLS = (
    0x10005D5FC,
    0x10006B1F8,
    0x10006BE00,
    0x100071A70,
    0x100074814,
    0x100079550,
    0x10007B7A8,
)
OUTLINED_COPY_CALLS = (
    0x10005A1F4,
    0x10005B250,
    0x10005B910,
    0x10005E0B4,
    0x10006E564,
    0x1000734DC,
    0x100073B9C,
    0x10007408C,
    0x100076A1C,
    0x100076C8C,
    0x100077208,
    0x100077E88,
    0x1000781B4,
    0x1000785AC,
    0x100078724,
    0x1000788D4,
    0x100078C40,
    0x1000798E8,
    0x100079DB8,
    0x10007ADB8,
    0x10007B90C,
    0x10007B9DC,
)
DIRECT_COPY_CALLS = (
    0x10005A7F0,
    0x10005EE44,
    0x10006BC08,
    0x10006C8E8,
    0x10006CF70,
    0x10006E0D0,
    0x10006E9B4,
    0x10006FF64,
    0x1000710E0,
    0x1000712A8,
    0x100071C58,
    0x1000721B4,
    0x1000749EC,
    0x100074EBC,
    0x10007C3C0,
)
PUSH_STORES = (
    0x10005D604,
    0x10006B200,
    0x10006BE08,
    0x100071A78,
    0x10007481C,
    0x100079558,
    0x10007B7B0,
)
REPLAY_LOAD_COUNT = 138
REPLAY_LOAD_SHA256 = "1f8405c2601995e244b2589afeea648a4e63446fba1059b396b0fb5b3e3b89d8"
EXPECTED_DIRECT_COUNTS = {
    "sub1us": {"push": 0, "replay": 0},
    "1to2p5": {"push": 0, "replay": 0},
    "2p5to20": {"push": 0, "replay": 0},
}


@dataclass(frozen=True)
class ChildExpectation:
    occurrences: int
    parent: int
    child: int


EXPECTED_CHILDREN = {
    "sub1us": {
        "grow": ChildExpectation(0, 0, 0),
        "outlined_copy": ChildExpectation(2, 2, 2),
        "direct_copy": ChildExpectation(8, 62, 62),
        "boundary_alloc": ChildExpectation(1, 29, 29),
    },
    "1to2p5": {
        "grow": ChildExpectation(0, 0, 0),
        "outlined_copy": ChildExpectation(5, 6, 6),
        "direct_copy": ChildExpectation(18, 38, 38),
        "boundary_alloc": ChildExpectation(1, 12, 12),
    },
    "2p5to20": {
        "grow": ChildExpectation(0, 0, 0),
        "outlined_copy": ChildExpectation(6, 6, 6),
        "direct_copy": ChildExpectation(28, 42, 42),
        "boundary_alloc": ChildExpectation(1, 2, 2),
    },
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_and_layout_custody() -> None:
    texts: dict[str, str] = {}
    for relative, expected in SOURCE_PINS.items():
        path = REPO_ROOT / relative
        actual = sha256(path)
        if actual != expected:
            raise SystemExit(f"REFUSE: source pin differs for {relative}: {actual}")
        texts[relative] = path.read_text()

    generator = texts["rust/src/ast_pipeline/ast_based_generator.rs"]
    cascade = texts["rust/src/ast_pipeline/ast_based_generator/cascade.rs"]
    runtime = texts["rust/src/ast_pipeline/mod.rs"]
    lock = texts["rust/Cargo.lock"]
    required = (
        (runtime, "nodes: typed_arena::Arena<ParseNode<'input>>"),
        (runtime, "self.nodes.alloc(node)"),
        (generator, "deriv_boundary: Vec<&'input ParseNode<'input>>"),
        (generator, "deriv_boundary: Vec::with_capacity((input.len() + 1).clamp(16, 8192))"),
        (cascade, "fn deriv_next_boundary(&mut self) -> &'input ParseNode<'input>"),
        (cascade, "parser.deriv_boundary.push(parser.arena.alloc(__pgen_alt_child));"),
        (cascade, "parser.deriv_boundary.extend_from_slice(__pgen_thin_b_seg);"),
        (cascade, "&parser.deriv_boundary[__pgen_thin_b_mark..]"),
        (cascade, "parser.deriv_boundary.copy_within(__pgen_cand_b_start.., __pgen_or_b_mark);"),
        (lock, 'name = "typed-arena"\nversion = "2.0.2"'),
    )
    missing = [snippet for text, snippet in required if snippet not in text]
    if missing:
        raise SystemExit(f"REFUSE: boundary source contract differs: {missing}")
    if cascade.count("deriv_boundary.push") != 2:
        raise SystemExit("REFUSE: boundary push emitter-site census differs")
    if cascade.count("deriv_boundary.copy_within") != 1:
        raise SystemExit("REFUSE: boundary copy emitter-site census differs")
    if cascade.count("deriv_boundary.extend_from_slice") != 1:
        raise SystemExit("REFUSE: boundary memo-replay emitter-site census differs")

    source = ARTIFACT_DIR / "layout_probe.rs"
    if sha256(source) != LAYOUT_SOURCE_SHA256:
        raise SystemExit("REFUSE: layout probe source differs")
    version = subprocess.run(
        ["rustc", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    if version != RUSTC_VERSION:
        raise SystemExit(f"REFUSE: rustc version differs: {version}")
    binary = Path("/tmp/pgen_deriv_boundary_layout_probe")
    subprocess.run(["rustc", str(source), "-O", "-o", str(binary)], check=True)
    output = subprocess.run([str(binary)], check=True, capture_output=True, text=True).stdout
    if output != EXPECTED_LAYOUT_OUTPUT:
        raise SystemExit(f"REFUSE: layout output differs:\n{output}")


def symbol_table() -> list[tuple[int, str]]:
    output = subprocess.run(
        ["nm", "-nm", str(capture.PROBE)], check=True, capture_output=True, text=True
    ).stdout
    symbols: list[tuple[int, str]] = []
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 4 and re.fullmatch(r"[0-9a-f]{16}", fields[0]):
            symbols.append((int(fields[0], 16), fields[-1]))
    symbols.sort()
    return symbols


def discover_replay_loads(
    code: dict[int, tuple[str, str]], symbols: list[tuple[int, str]]
) -> set[int]:
    loads: set[int] = set()
    for index, (start, symbol) in enumerate(symbols[:-1]):
        if (
            "generated_parsers5regex11RegexParser" not in symbol
            or "cascade_build_" not in symbol
        ):
            continue
        end = symbols[index + 1][0]
        for address in range(start, end, 4):
            mnemonic, operands = code.get(address, ("", ""))
            preceding = [
                code.get(candidate, ("", ""))
                for candidate in range(max(start, address - 16), address, 4)
            ]
            following = [
                code.get(candidate, ("", ""))
                for candidate in range(address + 4, min(end, address + 20), 4)
            ]
            if (
                mnemonic == "ldr"
                and "lsl #3" in operands
                and any(
                    child_mnemonic == "ldr" and "[x20, #0x298]" in child_operands
                    for child_mnemonic, child_operands in preceding
                )
                and any(
                    child_mnemonic == "str" and "[x20, #0x4f0]" in child_operands
                    for child_mnemonic, child_operands in following
                )
            ):
                loads.add(address)
    encoded = ",".join(f"{address:016x}" for address in sorted(loads)).encode()
    digest = hashlib.sha256(encoded).hexdigest()
    if len(loads) != REPLAY_LOAD_COUNT or digest != REPLAY_LOAD_SHA256:
        raise SystemExit(
            f"REFUSE: replay load census differs count={len(loads)} sha256={digest}"
        )
    return loads


def machine_custody(code: dict[int, tuple[str, str]]) -> set[int]:
    target_ranges = capture.symbol_ranges()
    within = lambda address: any(  # noqa: E731
        start <= address < end for start, end in target_ranges.values()
    )

    discovered_grow = set()
    discovered_outlined = set()
    discovered_direct = set()
    for address, (mnemonic, operands) in code.items():
        if mnemonic != "bl" or not within(address):
            continue
        preceding = [
            code.get(candidate, ("", ""))
            for candidate in range(address - 36, address, 4)
        ]
        if BOUNDARY_GROW_HASH in operands and any(
            "#0x290" in child_operands
            for _, child_operands in preceding
        ):
            discovered_grow.add(address)
        if BOUNDARY_COPY_HASH in operands:
            discovered_outlined.add(address)
        if (
            "_memmove" in operands
            and any("#0x298" in child_operands for _, child_operands in preceding)
            and any(
                child_mnemonic == "lsl" and child_operands.endswith("#3")
                for child_mnemonic, child_operands in preceding
            )
        ):
            discovered_direct.add(address)
    if discovered_grow != set(BOUNDARY_GROW_CALLS):
        raise SystemExit("REFUSE: boundary grow call census differs")
    if discovered_outlined != set(OUTLINED_COPY_CALLS):
        raise SystemExit("REFUSE: outlined boundary copy census differs")
    if discovered_direct != set(DIRECT_COPY_CALLS):
        raise SystemExit("REFUSE: direct boundary memmove census differs")

    if code.get(BOUNDARY_ALLOC_CALL) != ("bl", "_mi_malloc_aligned"):
        raise SystemExit("REFUSE: boundary constructor allocation differs")
    if code.get(BOUNDARY_ALLOC_CALL - 12) != ("lsl", "x21, x23, #3"):
        raise SystemExit("REFUSE: boundary allocation is not 8-byte qualified")

    with redirect_stdout(io.StringIO()):
        known.qualify_ranges(code, target_ranges)
    for address in PUSH_STORES:
        mnemonic, operands = code.get(address, ("", ""))
        owner = known.owner(address)
        if (
            mnemonic != "str"
            or not re.fullmatch(r"x\w+, \[x\d+, x\d+, lsl #3\]", operands)
            or owner is None
            or owner.mechanism != "g1c"
            or owner.role != "boundary_push"
        ):
            raise SystemExit(f"REFUSE: owned boundary push differs at {address:#x}")
    for address in BOUNDARY_GROW_CALLS:
        owner = known.owner(address)
        if owner is None or owner.mechanism != "g1c" or owner.role != "boundary_push":
            raise SystemExit(f"REFUSE: boundary growth is not G1-C-owned at {address:#x}")

    replay = discover_replay_loads(code, symbol_table())
    new_owned = (
        replay
        | set(OUTLINED_COPY_CALLS)
        | set(DIRECT_COPY_CALLS)
        | {BOUNDARY_ALLOC_CALL}
    )
    accepted = {
        address: known.owner(address)
        for address in new_owned
        if known.owner(address) is not None
    }
    if accepted:
        raise SystemExit(f"REFUSE: new boundary ownership overlaps accepted ranges: {accepted}")
    with redirect_stdout(io.StringIO()):
        input_view.overlap_custody(code, target_ranges, new_owned)
        _, input_len, input_data, _ = input_view.input_sites(code, target_ranges)
    position_sites = position_progress.update_sites(code, target_ranges)
    event_loads = event_width.discover_build_payload_loads(
        code, event_width.symbol_table()
    )
    event_union = (
        set(event_width.PAYLOAD_STORES)
        | event_loads
        | set(event_width.EVENT_GROW_CALLS)
        | set(event_width.OUTLINED_COPY_CALLS)
        | set(event_width.DIRECT_COPY_CALLS)
        | {event_width.EVENT_ALLOC_CALL}
    )
    overlaps = {
        "input_0192": len(new_owned & (input_len | input_data)),
        "position_0194": len(
            new_owned
            & set().union(
                position_sites["all_load"],
                position_sites["store"],
                position_sites["compare"],
                position_sites["branch"],
            )
        ),
        "event_0191": len(new_owned & event_union),
        "checkpoint_0193": sum(
            code[address][0] in {"ldr", "str"}
            and "[x20, #0xd8]" in code[address][1]
            for address in new_owned
        ),
    }
    if any(overlaps.values()):
        raise SystemExit(f"REFUSE: later held overlap: {overlaps}")
    return replay


def target_context(address: int) -> tuple[int, str]:
    if address < ATOM_START:
        return PIECE_START, "cascade_match_piece::h65565a4aa37893a3"
    return ATOM_START, "h3f63d2eb4d8831ca"


def classify_exact_callers(
    nodes: list[call_tree.Node], calls: tuple[int, ...], context: str
) -> ChildExpectation:
    occurrences = parent_samples = child_samples = 0
    for address in calls:
        symbol_start, fragment = target_context(address)
        return_offset = address + 4 - symbol_start
        parents = [
            index
            for index, node in enumerate(nodes)
            if fragment in node.text
            and re.search(rf"\+ {return_offset}(?: |,)", node.text)
        ]
        occurrences += len(parents)
        for parent_index in parents:
            parent = nodes[parent_index]
            parent_samples += parent.count
            children: list[call_tree.Node] = []
            index = parent_index + 1
            while index < len(nodes) and nodes[index].depth > parent.depth:
                if nodes[index].depth == parent.depth + 2:
                    children.append(nodes[index])
                index += 1
            if context == "grow":
                allowed = (BOUNDARY_GROW_HASH,)
            elif context == "outlined_copy":
                allowed = (BOUNDARY_COPY_HASH, "memmove")
            else:
                allowed = ("memmove",)
            foreign = [
                child
                for child in children
                if not any(term in child.text for term in allowed)
            ]
            if foreign:
                raise SystemExit(
                    f"REFUSE: {context} foreign children at {address:#x}: "
                    f"{[(child.line_number, child.text) for child in foreign]}"
                )
            child_samples += sum(child.count for child in children)
    if child_samples > parent_samples:
        raise SystemExit(f"REFUSE: {context} child exceeds parent samples")
    return ChildExpectation(occurrences, parent_samples, child_samples)


def classify_boundary_alloc(nodes: list[call_tree.Node]) -> ChildExpectation:
    return_offset = BOUNDARY_ALLOC_CALL + 4 - PARSE_ONCE_START
    parents = [
        index
        for index, node in enumerate(nodes)
        if "regex_perf_probe::parse_once_timed" in node.text
        and re.search(rf"\+ {return_offset}(?: |,)", node.text)
    ]
    parent_samples = child_samples = 0
    for parent_index in parents:
        parent = nodes[parent_index]
        parent_samples += parent.count
        index = parent_index + 1
        while index < len(nodes) and nodes[index].depth > parent.depth:
            if nodes[index].depth == parent.depth + 2:
                child = nodes[index]
                if not any(term in child.text for term in ("mi_", "OUTLINED_FUNCTION_32")):
                    raise SystemExit(f"REFUSE: boundary allocation foreign child: {child.text}")
                child_samples += child.count
            index += 1
    return ChildExpectation(len(parents), parent_samples, child_samples)


def raw_counts(owned: set[int]) -> tuple[dict[str, int], dict[str, int]]:
    counts: dict[str, int] = {}
    totals: dict[str, int] = {}
    for band in PROFILE_WEIGHTS:
        path = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(path) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: raw capture SHA-256 mismatch for {band}")
        header, pcs = capture.parse_raw(path)
        slide = int(header["main_slide"], 16)
        counts[band] = sum(sampled_pc - slide in owned for sampled_pc in pcs)
        totals[band] = len(pcs)
    return counts, totals


def weighted_child_ns(
    profile_rows: dict[str, tuple[int, dict[str, ChildExpectation]]],
    label: str,
    multiplier: float,
) -> float:
    return multiplier * FLOOR_NS * sum(
        PROFILE_WEIGHTS[band]
        * profile_rows[band][1][label].child
        / profile_rows[band][0]
        for band in PROFILE_WEIGHTS
    )


def main() -> None:
    source_and_layout_custody()
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    replay_loads = machine_custody(code)

    push_counts, raw_totals = raw_counts(set(PUSH_STORES))
    replay_counts, replay_totals = raw_counts(replay_loads)
    if raw_totals != replay_totals:
        raise SystemExit("REFUSE: direct-operation raw totals differ")
    for band in PROFILE_WEIGHTS:
        actual = {"push": push_counts[band], "replay": replay_counts[band]}
        if actual != EXPECTED_DIRECT_COUNTS[band]:
            raise SystemExit(f"REFUSE: {band} direct counts differ: {actual}")

    profile_rows: dict[str, tuple[int, dict[str, ChildExpectation]]] = {}
    for profile in call_tree.PROFILES:
        total, nodes = call_tree.parse_call_graph(
            (call_tree.PROFILE_DIR / profile.filename).read_bytes(), profile
        )
        actual = {
            "grow": classify_exact_callers(nodes, BOUNDARY_GROW_CALLS, "grow"),
            "outlined_copy": classify_exact_callers(
                nodes, OUTLINED_COPY_CALLS, "outlined_copy"
            ),
            "direct_copy": classify_exact_callers(
                nodes, DIRECT_COPY_CALLS, "direct_copy"
            ),
            "boundary_alloc": classify_boundary_alloc(nodes),
        }
        if actual != EXPECTED_CHILDREN[profile.band]:
            raise SystemExit(f"REFUSE: {profile.band} child rows differ: {actual}")
        profile_rows[profile.band] = (total, actual)

    outlined_ns = weighted_child_ns(profile_rows, "outlined_copy", 0.5)
    direct_ns = weighted_child_ns(profile_rows, "direct_copy", 0.5)
    allocation_ns = weighted_child_ns(profile_rows, "boundary_alloc", 1.0)
    copy_ns = outlined_ns + direct_ns

    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print(
        "raw custody: "
        + " ".join(
            f"{band}={known.RAW_SHA256[band]}" for band in PROFILE_WEIGHTS
        )
    )
    print(
        "profile custody: "
        + " ".join(
            f"{profile.band}={profile.expected_sha256}" for profile in call_tree.PROFILES
        )
    )
    print(
        "source custody: PASS "
        + " ".join(f"{path}={digest}" for path, digest in SOURCE_PINS.items())
    )
    print(EXPECTED_LAYOUT_OUTPUT.strip())
    print(
        "semantic identity: each element is one arbitrary stable arena reference; "
        "current NodeArena/typed-arena API returns only that reference"
    )
    print(
        "lossless candidates: usize ordinal stays 8 bytes; u32 needs a <=2^32 cap; "
        "u32+pointer side table retains one 8-byte pointer per unique node; "
        "chunk ordinal needs a new indexed arena plus chunk lookup and wide escape"
    )
    print(
        "reconstruction debit: load handle + bounds/escape control + chunk/side-table "
        "base load + final pointer load; no debit is priced at zero"
    )
    print(
        f"static: push_stores={len(PUSH_STORES)} replay_loads={len(replay_loads)} "
        f"grow_calls={len(BOUNDARY_GROW_CALLS)} outlined_copies={len(OUTLINED_COPY_CALLS)} "
        f"direct_memmoves={len(DIRECT_COPY_CALLS)} thin_memo_success_sites=2"
    )
    print(
        "ownership: pushes+growth=G1-C excluded; thin-memo success copy/insert=accepted "
        "ranges excluded; tournament copies + replay + constructor accepted_held_overlap=0"
    )
    print()
    print("DYNAMIC DIRECT INSTRUCTIONS")
    for band in PROFILE_WEIGHTS:
        print(
            f"band={band:8} raw={raw_totals[band]:5d} "
            f"owned_push={push_counts[band]:2d} replay_load={replay_counts[band]:2d}"
        )
    print()
    print("CALLER-ATTRIBUTED CHILDREN")
    print("band       total  grow  outlined_copy  direct_memmove  boundary_alloc")
    for band in PROFILE_WEIGHTS:
        total, rows = profile_rows[band]
        print(
            f"{band:9} {total:5d} {rows['grow'].child:5d} "
            f"{rows['outlined_copy'].child:14d} {rows['direct_copy'].child:15d} "
            f"{rows['boundary_alloc'].child:14d}"
        )
    print()
    print("REPLACEMENT-AWARE PRICING")
    print("strict_removable_instruction_floor_ns=0.000000000")
    print(
        f"u32_fantasy_linear_byte_copy_ceiling_ns={copy_ns:.9f} "
        f"outlined={outlined_ns:.9f} direct={direct_ns:.9f}"
    )
    print(f"whole_boundary_allocation_child_absolute_ceiling_ns={allocation_ns:.9f}")
    for label, contribution in (
        ("strict", 0.0),
        ("plus_linear_copy", copy_ns),
        ("plus_absolute_allocator", copy_ns + allocation_ns),
    ):
        composite = PRE_BOUNDARY_STRICT_NS + contribution
        print(
            f"view={label:23} boundary_ns={contribution:.9f} "
            f"composite_ns={composite:.9f} capture_0.30_ns={0.30 * composite:.9f} "
            f"noise_margin_ns={0.30 * composite - NOISE_NS:.9f}"
        )
    print(
        "verdict=HOLD (strict zero; every compact lossless design adds identity "
        "storage or indexed-arena reconstruction absent from the gross ceilings)"
    )
    print("DERIVATION BOUNDARY INDEX PRICING: PASS")


if __name__ == "__main__":
    main()
