#!/usr/bin/env python3
"""Price lossless one-word common-case derivation events on banked evidence.

The strict price admits only current second tag/payload memory operations that
a packed word can replace one-for-one.  Tournament-copy time is reported only
as a 50%-of-child linear-byte ceiling, and constructor allocation time only as
an absolute whole-child ceiling; neither model-dependent ceiling is promoted
to measured savings.
"""

from __future__ import annotations

import hashlib
import re
import subprocess
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
CALL_TREE_DIR = TASK_ARTIFACTS / "recursion_name_growth"
GUARD_DIR = TASK_ARTIFACTS / "recursion_guard_expansion"
BARE_DIR = TASK_ARTIFACTS / "bare_diagnostic_expansion"
LOOKUP_DIR = TASK_ARTIFACTS / "thin_memo_lookup_expansion"
TELEMETRY_DIR = TASK_ARTIFACTS / "semantic_telemetry_expansion"
ROLLBACK_DIR = TASK_ARTIFACTS / "try_parse_name_rollback"
sys.path[:0] = [
    str(CAPTURE_DIR),
    str(CALL_TREE_DIR),
    str(GUARD_DIR),
    str(BARE_DIR),
    str(LOOKUP_DIR),
    str(TELEMETRY_DIR),
    str(ROLLBACK_DIR),
]

import analyze_capture as capture  # noqa: E402
import classify_bare_diagnostics as bare  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_recursion_name_growth as call_tree  # noqa: E402
import classify_recursion_guard as guard  # noqa: E402
import classify_semantic_telemetry as telemetry  # noqa: E402
import classify_thin_memo_lookup as thin_lookup  # noqa: E402
import classify_try_parse_name_rollback as rollback  # noqa: E402


FLOOR_NS = 1263.4
NOISE_NS = 28.8
PRE_EVENT_COMPOSITE_NS = 100.749486548
PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
SOURCE_PINS = {
    "rust/src/ast_pipeline/mod.rs": "031e8f31407c171d93dd699d5907fd025ff126a4405d8e498174a87e6d6efbd8",
    "rust/src/ast_pipeline/ast_based_generator.rs": "b5ed80b80adfcf1a6fb5df38884e249e06eff2f32a5e78dfef8760d256f374df",
    "rust/src/ast_pipeline/ast_based_generator/cascade.rs": "413fefe99dbfe8d4a03886f4eb4999d078fb03696f861ee1e7fefc23db03a100",
    "rust/src/ast_pipeline/ast_based_generator/cascade/value.rs": "cd409c2068ed5e95f40ae3a19fa0da865571c35b91cefdbb4e9790a02d461f0c",
}
EXPECTED_LAYOUT_OUTPUT = (
    "DerivEvent size=16 align=8 needs_drop=false\n"
    "PackedDerivWord size=8 align=8 needs_drop=false\n"
    "usize_bits=64\n"
    "ordinary_payload_max=2305843009213693951\n"
)

PIECE_START = 0x100058D20
ATOM_START = 0x10006ACF4
PARSE_ONCE_START = 0x10000179C
COPY_WITHIN_HASH = "h682994d9b3d86c5b"
EVENT_GROW_HASH = "h84ff3e56869fffcc"

PAYLOAD_STORES = (
    0x1000591F8, 0x100059400, 0x10005A514, 0x10005B0A4,
    0x10005B7B4, 0x10005CABC, 0x10005D818, 0x10005E740,
    0x10006AD44, 0x10006D660, 0x10006D8F0, 0x10006EF04,
    0x100070F44, 0x100073F30, 0x100074C28, 0x10007572C,
    0x100078AE8, 0x10007924C, 0x1000796DC, 0x10007A53C,
    0x10007C084, 0x10007C728,
)
EXPECTED_STORE_COUNTS = {"sub1us": 7, "1to2p5": 7, "2p5to20": 17}

# The build-side census is mechanically rediscovered below.  Pinning the
# address-list digest keeps the evidence compact without weakening custody.
BUILD_LOAD_COUNT = 134
BUILD_LOAD_SHA256 = "9bd9edee388eb70bf91eb36cf24d1e374fb3b24301add446f7e723cc8e805f31"
EXPECTED_BUILD_LOAD_COUNTS = {"sub1us": 0, "1to2p5": 3, "2p5to20": 0}

EVENT_GROW_CALLS = (
    0x1000593E4, 0x10005A4F8, 0x10005A71C, 0x10005C640,
    0x10005D630, 0x10005D7F4, 0x10006B0FC, 0x10006D644,
    0x10006EEF4, 0x10006F51C, 0x100070778, 0x100072A18,
    0x100073F20, 0x100075718, 0x100079224, 0x10007A52C,
)

OUTLINED_COPY_CALLS = (
    0x10005A1AC, 0x10005B218, 0x10005B8C8, 0x10005E06C,
    0x100068BD8, 0x10006E51C, 0x100073494, 0x100073B54,
    0x10007405C, 0x1000769D4, 0x100076C48, 0x1000771C0,
    0x100077E40, 0x100078168, 0x100078574, 0x1000786DC,
    0x10007888C, 0x100078BF8, 0x1000798A0, 0x100079D70,
    0x10007AD6C, 0x10007B8BC, 0x10007B990,
)
DIRECT_COPY_CALLS = (
    0x10005A7B4, 0x10005EE08, 0x10006BBC4, 0x10006C8A8,
    0x10006CF30, 0x10006E090, 0x10006E974, 0x10006FF24,
    0x1000710A0, 0x100071268, 0x100071C14, 0x100072174,
    0x1000749AC, 0x100074E7C, 0x10007C380,
)
EVENT_ALLOC_CALL = 0x100001DF4
BOUNDARY_ALLOC_CONTROL = 0x100001E24


@dataclass(frozen=True)
class ChildExpectation:
    occurrences: int
    parent: int
    child: int


EXPECTED_CHILDREN = {
    "sub1us": {
        "grow": ChildExpectation(0, 0, 0),
        "outlined_copy": ChildExpectation(1, 1, 1),
        "direct_copy": ChildExpectation(11, 46, 46),
        "event_alloc": ChildExpectation(2, 22, 22),
    },
    "1to2p5": {
        "grow": ChildExpectation(0, 0, 0),
        "outlined_copy": ChildExpectation(4, 5, 5),
        "direct_copy": ChildExpectation(17, 39, 39),
        "event_alloc": ChildExpectation(2, 23, 23),
    },
    "2p5to20": {
        "grow": ChildExpectation(0, 0, 0),
        "outlined_copy": ChildExpectation(8, 9, 9),
        "direct_copy": ChildExpectation(26, 32, 32),
        "event_alloc": ChildExpectation(1, 24, 24),
    },
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_and_layout_custody() -> None:
    for relative, expected in SOURCE_PINS.items():
        actual = sha256(REPO_ROOT / relative)
        if actual != expected:
            raise SystemExit(f"REFUSE: source pin differs for {relative}: {actual}")

    mod_source = (REPO_ROOT / "rust/src/ast_pipeline/mod.rs").read_text()
    match = re.search(r"pub enum DerivEvent \{(?P<body>.*?)\n\}", mod_source, re.S)
    if not match:
        raise SystemExit("REFUSE: DerivEvent declaration absent")
    variants = re.findall(
        r"^\s*(OrWinner|QuantCount|OptPresent|TokStart|TokEnd)\((usize|bool)\),",
        match.group("body"),
        re.M,
    )
    expected_variants = [
        ("OrWinner", "usize"),
        ("QuantCount", "usize"),
        ("OptPresent", "bool"),
        ("TokStart", "usize"),
        ("TokEnd", "usize"),
    ]
    if variants != expected_variants:
        raise SystemExit(f"REFUSE: DerivEvent variants differ: {variants}")

    binary = Path("/tmp/pgen_deriv_event_width_layout_probe")
    subprocess.run(
        ["rustc", str(ARTIFACT_DIR / "layout_probe.rs"), "-O", "-o", str(binary)],
        check=True,
    )
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


def discover_build_payload_loads(
    code: dict[int, tuple[str, str]], symbols: list[tuple[int, str]]
) -> set[int]:
    loads: set[int] = set()
    for index, (start, symbol) in enumerate(symbols[:-1]):
        if (
            "generated_parsers5regex11RegexParser" not in symbol
            or "cascade_build_" not in symbol
            or not (capture.TEXT_START <= start < capture.TEXT_END)
        ):
            continue
        end = symbols[index + 1][0]
        for address in range(start, end, 4):
            mnemonic, operands = code.get(address, ("", ""))
            pointer_match = re.match(r"(x\d+), (x\d+), .*lsl #4$", operands)
            if mnemonic != "add" or pointer_match is None:
                continue
            pointer = pointer_match.group(1)
            following = [
                (candidate, *code.get(candidate, ("", "")))
                for candidate in range(address + 4, min(end, address + 20), 4)
            ]
            has_tag_load = any(
                child_mnemonic in {"ldrb", "ldur", "ldr"}
                and child_operands.endswith(f"[{pointer}]")
                for _, child_mnemonic, child_operands in following
            )
            if not has_tag_load:
                continue
            loads.update(
                candidate
                for candidate, child_mnemonic, child_operands in following
                if child_mnemonic.startswith(("ldr", "ldur"))
                and (
                    f"[{pointer}, #0x8]" in child_operands
                    or f"[{pointer}, #0x1]" in child_operands
                )
            )
    encoded = ",".join(f"{address:016x}" for address in sorted(loads)).encode()
    digest = hashlib.sha256(encoded).hexdigest()
    if len(loads) != BUILD_LOAD_COUNT or digest != BUILD_LOAD_SHA256:
        raise SystemExit(
            f"REFUSE: build payload loads differ count={len(loads)} sha256={digest}"
        )
    return loads


def machine_custody(code: dict[int, tuple[str, str]]) -> set[int]:
    for address in PAYLOAD_STORES:
        mnemonic, operands = code.get(address, ("", ""))
        if mnemonic != "str" or not re.fullmatch(r"x\w+, \[x\d+, #0x8\]", operands):
            raise SystemExit(f"REFUSE: payload store differs at {address:#x}: {(mnemonic, operands)}")

    for address in EVENT_GROW_CALLS:
        mnemonic, operands = code.get(address, ("", ""))
        if mnemonic != "bl" or EVENT_GROW_HASH not in operands:
            raise SystemExit(f"REFUSE: event grow call differs at {address:#x}")
        prior = [code.get(candidate, ("", ""))[1] for candidate in range(address - 32, address, 4)]
        if not any("x20, #0x278" in operands for operands in prior):
            raise SystemExit(f"REFUSE: event grow base not qualified at {address:#x}")

    for address in OUTLINED_COPY_CALLS:
        mnemonic, operands = code.get(address, ("", ""))
        if mnemonic != "bl" or COPY_WITHIN_HASH not in operands:
            raise SystemExit(f"REFUSE: outlined event copy differs at {address:#x}")
    for address in DIRECT_COPY_CALLS:
        mnemonic, operands = code.get(address, ("", ""))
        prior = [code.get(candidate, ("", "")) for candidate in range(address - 36, address, 4)]
        if (
            mnemonic != "bl"
            or "_memmove" not in operands
            or not any("[x20, #0x280]" in child_operands for _, child_operands in prior)
            or not any(child_mnemonic == "lsl" and child_operands.endswith("#4") for child_mnemonic, child_operands in prior)
        ):
            raise SystemExit(f"REFUSE: direct event memmove differs at {address:#x}")

    event_alloc = code.get(EVENT_ALLOC_CALL, ("", ""))
    boundary_alloc = code.get(BOUNDARY_ALLOC_CONTROL, ("", ""))
    if event_alloc != ("bl", "_mi_malloc_aligned") or boundary_alloc != ("bl", "_mi_malloc_aligned"):
        raise SystemExit("REFUSE: constructor allocation calls differ")
    if code.get(EVENT_ALLOC_CALL - 12) != ("lsl", "x21, x28, #4"):
        raise SystemExit("REFUSE: event allocation is not 16-byte width qualified")
    if code.get(BOUNDARY_ALLOC_CONTROL - 12) != ("lsl", "x21, x23, #3"):
        raise SystemExit("REFUSE: boundary allocation control is not 8-byte qualified")

    build_loads = discover_build_payload_loads(code, symbol_table())
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    trace, _, _ = bare.trace_gate_pcs(code, target_ranges)
    coverage, _, _ = bare.coverage_pcs(code, target_ranges)
    nonempty, _, _ = bare.rollback_diagnostic_pcs(code, target_ranges)
    lookup_full, _ = thin_lookup.qualify_sites(code, target_ranges)
    telemetry_sites = telemetry.discover_sites(code, target_ranges)
    telemetry_union = set().union(*(pcs for _, _, _, pcs in telemetry_sites), set())
    pops = guard.discover_name_pops(code, target_ranges)
    guard_union = guard.INLINE_NAME_PUSH | guard.OUTLINED_NAME_PUSH | set().union(
        *(pcs for _, _, pcs in pops), set()
    )
    rollback_sites = rollback.discover_truncate_sites(
        code, target_ranges, guard.INLINE_NAME_PUSH | set().union(
            *(pcs for _, _, pcs in pops), set()
        )
    )
    rollback_union = set().union(*(pcs for *_, pcs in rollback_sites), set())
    direct_owned = set(PAYLOAD_STORES) | build_loads
    accepted_overlap = {
        address: known.owner(address)
        for address in direct_owned
        if known.owner(address) is not None
    }
    if accepted_overlap:
        raise SystemExit(f"REFUSE: direct-instruction accepted overlap {accepted_overlap}")
    held_overlap = {
        "bare_0185": len(direct_owned & (trace | coverage | nonempty)),
        "thin_lookup_0186": len(direct_owned & lookup_full),
        "telemetry_0187": len(direct_owned & telemetry_union),
        "recursion_guard_0188": len(direct_owned & guard_union),
        "try_parse_rollback_0189": len(direct_owned & rollback_union),
    }
    if any(held_overlap.values()):
        raise SystemExit(f"REFUSE: direct-instruction held overlap {held_overlap}")
    prior_call_pcs = set(call_tree.CALLSITES[index].call_pc for index in range(len(call_tree.CALLSITES)))
    if (set(EVENT_GROW_CALLS) | set(OUTLINED_COPY_CALLS) | set(DIRECT_COPY_CALLS)) & prior_call_pcs:
        raise SystemExit("REFUSE: caller-attributed unit overlaps -0190 callsites")
    return build_loads


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
                foreign = [child for child in children if EVENT_GROW_HASH not in child.text]
            else:
                foreign = [
                    child
                    for child in children
                    if "memmove" not in child.text and COPY_WITHIN_HASH not in child.text
                ]
            if foreign:
                raise SystemExit(
                    f"REFUSE: {context} foreign children at {address:#x}: "
                    f"{[(child.line_number, child.text) for child in foreign]}"
                )
            child_samples += sum(child.count for child in children)
    if child_samples > parent_samples:
        raise SystemExit(f"REFUSE: {context} child count exceeds parent count")
    return ChildExpectation(occurrences, parent_samples, child_samples)


def classify_event_alloc(nodes: list[call_tree.Node]) -> ChildExpectation:
    return_offset = EVENT_ALLOC_CALL + 4 - PARSE_ONCE_START
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
                if not any(
                    term in nodes[index].text
                    for term in ("mi_", "OUTLINED_FUNCTION_32")
                ):
                    raise SystemExit(
                        f"REFUSE: event allocation foreign child: {nodes[index].text}"
                    )
                child_samples += nodes[index].count
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


def weighted_ns(counts: dict[str, int], totals: dict[str, int]) -> float:
    return FLOOR_NS * sum(
        PROFILE_WEIGHTS[band] * counts[band] / totals[band]
        for band in PROFILE_WEIGHTS
    )


def main() -> None:
    source_and_layout_custody()
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    build_loads = machine_custody(code)

    store_counts, raw_totals = raw_counts(set(PAYLOAD_STORES))
    load_counts, load_raw_totals = raw_counts(build_loads)
    if raw_totals != load_raw_totals:
        raise SystemExit("REFUSE: raw totals differ between censuses")
    if store_counts != EXPECTED_STORE_COUNTS:
        raise SystemExit(f"REFUSE: payload store dynamic counts differ: {store_counts}")
    if load_counts != EXPECTED_BUILD_LOAD_COUNTS:
        raise SystemExit(f"REFUSE: build payload load dynamic counts differ: {load_counts}")

    profile_rows: dict[str, tuple[int, dict[str, ChildExpectation]]] = {}
    for profile in call_tree.PROFILES:
        total, nodes = call_tree.parse_call_graph(
            (call_tree.PROFILE_DIR / profile.filename).read_bytes(), profile
        )
        actual = {
            "grow": classify_exact_callers(nodes, EVENT_GROW_CALLS, "grow"),
            "outlined_copy": classify_exact_callers(
                nodes, OUTLINED_COPY_CALLS, "outlined_copy"
            ),
            "direct_copy": classify_exact_callers(
                nodes, DIRECT_COPY_CALLS, "direct_copy"
            ),
            "event_alloc": classify_event_alloc(nodes),
        }
        if actual != EXPECTED_CHILDREN[profile.band]:
            raise SystemExit(
                f"REFUSE: {profile.band} caller counts differ: {actual}"
            )
        profile_rows[profile.band] = (total, actual)

    store_ns = weighted_ns(store_counts, raw_totals)
    load_ns = weighted_ns(load_counts, raw_totals)
    outlined_copy_ns = 0.5 * FLOOR_NS * sum(
        PROFILE_WEIGHTS[band]
        * profile_rows[band][1]["outlined_copy"].child
        / profile_rows[band][0]
        for band in PROFILE_WEIGHTS
    )
    direct_copy_ns = 0.5 * FLOOR_NS * sum(
        PROFILE_WEIGHTS[band]
        * profile_rows[band][1]["direct_copy"].child
        / profile_rows[band][0]
        for band in PROFILE_WEIGHTS
    )
    alloc_ceiling_ns = FLOOR_NS * sum(
        PROFILE_WEIGHTS[band]
        * profile_rows[band][1]["event_alloc"].child
        / profile_rows[band][0]
        for band in PROFILE_WEIGHTS
    )
    strict_ns = store_ns + load_ns
    linear_copy_ns = strict_ns + outlined_copy_ns + direct_copy_ns
    absolute_ns = linear_copy_ns + alloc_ceiling_ns

    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print("source custody: PASS " + " ".join(f"{path}={digest}" for path, digest in SOURCE_PINS.items()))
    print(EXPECTED_LAYOUT_OUTPUT.strip())
    print(
        "lossless carrier: low3 tags=5 ordinary variants; tag6=in-band escape; "
        "escape header carries original tag and following word carries full usize"
    )
    print(
        "correctness: no input/count/branch cap; marks/cursors/segments become word "
        "indices; wide QuantCount patch may insert one word; copying remains word-safe"
    )
    print(
        f"static: payload_stores={len(PAYLOAD_STORES)} build_payload_loads={len(build_loads)} "
        f"event_grow_calls={len(EVENT_GROW_CALLS)} outlined_copies={len(OUTLINED_COPY_CALLS)} "
        f"direct_memmoves={len(DIRECT_COPY_CALLS)} accepted_overlap=0"
    )
    print("excluded: deriv_boundary G1-C thin-memo segment copies teardown and all non-event fields")
    print()
    print("DYNAMIC DIRECT INSTRUCTIONS")
    for band in PROFILE_WEIGHTS:
        print(
            f"band={band:8} raw={raw_totals[band]:5d} "
            f"payload_store={store_counts[band]:2d} build_payload_load={load_counts[band]:2d}"
        )
    print()
    print("CALLER-ATTRIBUTED CHILDREN")
    print("band       total  grow  outlined_copy  direct_memmove  event_alloc")
    for band in PROFILE_WEIGHTS:
        total, rows = profile_rows[band]
        print(
            f"{band:9} {total:5d} {rows['grow'].child:5d} "
            f"{rows['outlined_copy'].child:14d} {rows['direct_copy'].child:15d} "
            f"{rows['event_alloc'].child:12d}"
        )
    print()
    print("REPLACEMENT-AWARE PRICING")
    print(f"strict_payload_store_ns={store_ns:.9f}")
    print(f"strict_build_payload_load_ns={load_ns:.9f}")
    print(f"strict_removable_instruction_floor_ns={strict_ns:.9f}")
    print(
        f"linear_byte_copy_ceiling_ns={outlined_copy_ns + direct_copy_ns:.9f} "
        f"outlined={outlined_copy_ns:.9f} direct={direct_copy_ns:.9f}"
    )
    print(f"whole_event_allocation_child_absolute_ceiling_ns={alloc_ceiling_ns:.9f}")
    for label, contribution in (
        ("strict", strict_ns),
        ("plus_linear_copy", linear_copy_ns),
        ("plus_absolute_allocator", absolute_ns),
    ):
        composite = PRE_EVENT_COMPOSITE_NS + contribution
        print(
            f"view={label:23} event_ns={contribution:.9f} composite_ns={composite:.9f} "
            f"capture_0.30_ns={0.30 * composite:.9f} "
            f"noise_margin_ns={0.30 * composite - NOISE_NS:.9f}"
        )
    print(
        "replacement_debits=tag_mask+payload_shift+escape_branch+wide_patch; "
        "unmeasured and therefore not subtracted from ceilings"
    )
    print(
        "verdict=HOLD (strict contribution is 0.530 ns; even the deliberately "
        "impossible whole-allocation ceiling leaves only 3.057 ns practical margin)"
    )
    print("DERIVATION EVENT WIDTH PRICING: PASS")


if __name__ == "__main__":
    main()
