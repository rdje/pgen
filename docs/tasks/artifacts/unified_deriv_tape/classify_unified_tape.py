#!/usr/bin/env python3
"""Prove and price a lossless unified derivation-tape carrier.

The strict result is replacement-aware.  A single tape really removes the
boundary Vec/cursor lane, but the only safe eight-byte representation adds a
tag decode on boundary records.  Because the preserved profile does not price
that debit, direct boundary-field time is reported as a gross ceiling rather
than a strict saving.  Previously accepted event-width, G1-C, and thin-memo
ranges are never credited twice.
"""

from __future__ import annotations

import hashlib
import io
import subprocess
import sys
from contextlib import redirect_stdout
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
EVENT_DIR = TASK_ARTIFACTS / "deriv_event_width"
BOUNDARY_DIR = TASK_ARTIFACTS / "deriv_boundary_index"
FIELD_DIR = TASK_ARTIFACTS / "parser_direct_field"
sys.path[:0] = [str(CAPTURE_DIR), str(EVENT_DIR), str(BOUNDARY_DIR), str(FIELD_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_deriv_boundary as boundary  # noqa: E402
import classify_deriv_event_width as event  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_parser_fields as parser_fields  # noqa: E402


FLOOR_NS = 1263.4
NOISE_NS = 28.8
PRE_UNIFIED_STRICT_NS = 102.138976561
PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
SOURCE_PINS = {
    "rust/src/ast_pipeline/mod.rs": "031e8f31407c171d93dd699d5907fd025ff126a4405d8e498174a87e6d6efbd8",
    "rust/src/ast_pipeline/ast_based_generator.rs": "b5ed80b80adfcf1a6fb5df38884e249e06eff2f32a5e78dfef8760d256f374df",
    "rust/src/ast_pipeline/ast_based_generator/cascade.rs": "413fefe99dbfe8d4a03886f4eb4999d078fb03696f861ee1e7fefc23db03a100",
}
PRIOR_PINS = {
    "deriv_event_width/classify_deriv_event_width.py": "9c5edbda41cc862095577911ad353b31cb0040f1d839e45929959accde82c213",
    "deriv_event_width/deriv_event_width.txt": "72ec056f6af02023c96c99edefc3f45aa7294d5fb2d6e732b6049afc870acb23",
    "deriv_boundary_index/classify_deriv_boundary.py": "246bf0d71c6fd5762e20b98ec6d4fea39efa64801b3311c490e8aec118b75f39",
    "deriv_boundary_index/deriv_boundary_index.txt": "9a65997d2281a95113aa715ad010079db537244f6018e312053a5a8b3f26e371",
}
RUSTC_VERSION = "rustc 1.95.0 (59807616e 2026-04-14)"
LAYOUT_SOURCE_SHA256 = "27674ce16c231a06996e68b3da58eb966db7478264d84acb4ad403b8e2326e95"
EXPECTED_LAYOUT_OUTPUT = (
    "DerivEvent size=16 align=8 needs_drop=false\n"
    "ParseNodeStandIn size=48 align=8 needs_drop=false\n"
    "BoundaryRef size=8 align=8 needs_drop=false\n"
    "TaggedCurrent size=16 align=8 needs_drop=false\n"
    "TaggedPacked size=16 align=8 needs_drop=false\n"
    "UntaggedWord size=8 align=8 needs_drop=false\n"
    "TaggedAddressWord size=8 align=8 needs_drop=false\n"
    "WideEventRecord size=16 align=8 needs_drop=false\n"
    "usize_bits=64\n"
    "narrow_payload_max=2305843009213693951\n"
    "codec_roundtrip=PASS\n"
)
CASCADE_COUNTS = {
    "deriv_events.len()": 13,
    "deriv_boundary.len()": 11,
    "deriv_events.truncate": 11,
    "deriv_boundary.truncate": 10,
    "deriv_events.copy_within": 1,
    "deriv_boundary.copy_within": 1,
    "deriv_events.push": 7,
    "deriv_boundary.push": 2,
    "deriv_events.extend_from_slice": 1,
    "deriv_boundary.extend_from_slice": 1,
    "deriv_ev_cursor": 5,
    "deriv_b_cursor": 4,
    "deriv_next_event()": 6,
    "deriv_next_boundary()": 1,
}
GENERATOR_COUNTS = {
    "deriv_events:": 2,
    "deriv_boundary:": 2,
    "deriv_ev_cursor": 2,
    "deriv_b_cursor": 2,
    "deriv_events.clear()": 1,
    "deriv_boundary.clear()": 1,
}
EVENT_OFFSETS = frozenset({0x278, 0x280, 0x288})
BOUNDARY_OFFSETS = frozenset({0x290, 0x298, 0x2A0})
EXPECTED_STATIC = {
    "event": {0x278: 9, 0x280: 78, 0x288: 204},
    "boundary": {0x290: 2, 0x298: 46, 0x2A0: 167},
}
EXPECTED_DYNAMIC = {
    "sub1us": {"event": 104, "boundary": 30},
    "1to2p5": {"event": 74, "boundary": 19},
    "2p5to20": {"event": 78, "boundary": 40},
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
    for relative, expected in PRIOR_PINS.items():
        path = TASK_ARTIFACTS / relative
        actual = sha256(path)
        if actual != expected:
            raise SystemExit(f"REFUSE: prior evidence pin differs for {relative}: {actual}")

    cascade_source = texts["rust/src/ast_pipeline/ast_based_generator/cascade.rs"]
    generator_source = texts["rust/src/ast_pipeline/ast_based_generator.rs"]
    runtime_source = texts["rust/src/ast_pipeline/mod.rs"]
    for needle, expected in CASCADE_COUNTS.items():
        actual = cascade_source.count(needle)
        if actual != expected:
            raise SystemExit(f"REFUSE: cascade count differs for {needle}: {actual}")
    for needle, expected in GENERATOR_COUNTS.items():
        actual = generator_source.count(needle)
        if actual != expected:
            raise SystemExit(f"REFUSE: generator count differs for {needle}: {actual}")

    required = (
        "pub struct ParseNode<'input>",
        "pub rule_name: &'static str",
        "pub content: ParseContent<'input>",
        "pub span: std::ops::Range<usize>",
        "boundary call-out VALUES live in the parser's `deriv_boundary` side vec",
    )
    if any(needle not in runtime_source for needle in required):
        raise SystemExit("REFUSE: alignment/side-tape source contract differs")

    ordered_motifs = (
        "let __pgen_or_ev_mark = parser.deriv_events.len();",
        "parser.deriv_events.push(crate::ast_pipeline::DerivEvent::OrWinner(0));",
        "let __pgen_or_b_mark = parser.deriv_boundary.len();",
        "parser.deriv_events.copy_within(__pgen_cand_ev_start.., __pgen_or_ev_mark + 1);",
        "parser.deriv_boundary.copy_within(__pgen_cand_b_start.., __pgen_or_b_mark);",
        "parser.deriv_events[__pgen_or_ev_mark] =",
        "let __pgen_quant_ev_mark = parser.deriv_events.len();",
        "parser.deriv_events.push(crate::ast_pipeline::DerivEvent::QuantCount(0));",
        "parser.deriv_events[__pgen_quant_ev_mark] =",
        "let __pgen_alt_child = parser.#method()?;",
        "parser.deriv_boundary.push(parser.arena.alloc(__pgen_alt_child));",
        "let __pgen_alt_node = parser.deriv_next_boundary();",
        "parser.deriv_pos = __pgen_alt_node.span.end;",
        "parser.deriv_events.extend_from_slice(__pgen_thin_ev_seg);",
        "parser.deriv_boundary.extend_from_slice(__pgen_thin_b_seg);",
    )
    if any(needle not in cascade_source for needle in ordered_motifs):
        raise SystemExit("REFUSE: match/build ordering motif differs")

    for script_name, output_name in (
        ("classify_deriv_event_width.py", "deriv_event_width.txt"),
        ("classify_deriv_boundary.py", "deriv_boundary_index.txt"),
    ):
        directory = EVENT_DIR if "event" in script_name else BOUNDARY_DIR
        result = subprocess.run(
            [sys.executable, str(directory / script_name)],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        expected = (directory / output_name).read_text()
        if result != expected:
            raise SystemExit(f"REFUSE: {script_name} no longer reproduces its output")

    layout_source = ARTIFACT_DIR / "layout_probe.rs"
    if sha256(layout_source) != LAYOUT_SOURCE_SHA256:
        raise SystemExit("REFUSE: unified layout probe source differs")
    version = subprocess.run(
        ["rustc", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    if version != RUSTC_VERSION:
        raise SystemExit(f"REFUSE: rustc version differs: {version}")
    binary = Path("/tmp/pgen_unified_deriv_tape_layout_probe")
    subprocess.run(["rustc", str(layout_source), "-O", "-o", str(binary)], check=True)
    output = subprocess.run([str(binary)], check=True, capture_output=True, text=True).stdout
    if output != EXPECTED_LAYOUT_OUTPUT:
        raise SystemExit(f"REFUSE: unified layout output differs:\n{output}")


def direct_sites(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
    offsets: frozenset[int],
) -> set[int]:
    sites: set[int] = set()
    for start, end in target_ranges.values():
        for address in range(start, end, 4):
            instruction = code.get(address)
            if (
                instruction is not None
                and known.owner(address) is None
                and parser_fields.is_parser_direct(instruction)
                and parser_fields.parser_offset(instruction[1]) in offsets
            ):
                sites.add(address)
    return sites


def by_offset(code: dict[int, tuple[str, str]], sites: set[int]) -> dict[int, int]:
    result: dict[int, int] = {}
    for address in sites:
        offset = parser_fields.parser_offset(code[address][1])
        result[offset] = result.get(offset, 0) + 1
    return result


def raw_counts(sites: set[int]) -> tuple[dict[str, int], dict[str, int]]:
    counts: dict[str, int] = {}
    totals: dict[str, int] = {}
    for band in PROFILE_WEIGHTS:
        path = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(path) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: raw capture SHA-256 mismatch for {band}")
        header, pcs = capture.parse_raw(path)
        slide = int(header["main_slide"], 16)
        counts[band] = sum(sampled_pc - slide in sites for sampled_pc in pcs)
        totals[band] = len(pcs)
    return counts, totals


def pair_copies(
    event_calls: tuple[int, ...], boundary_calls: tuple[int, ...]
) -> tuple[list[tuple[int, int, int]], set[int]]:
    pairs: list[tuple[int, int, int]] = []
    used: set[int] = set()
    for boundary_call in boundary_calls:
        candidates = [
            event_call
            for event_call in event_calls
            if 0 < boundary_call - event_call <= 0x50 and event_call not in used
        ]
        if len(candidates) != 1:
            raise SystemExit(
                f"REFUSE: boundary copy {boundary_call:#x} has event pairs {candidates}"
            )
        event_call = candidates[0]
        used.add(event_call)
        pairs.append((event_call, boundary_call, boundary_call - event_call))
    return pairs, set(event_calls) - used


def main() -> None:
    source_and_layout_custody()
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    with redirect_stdout(io.StringIO()):
        known.qualify_ranges(code, target_ranges)

    event_sites = direct_sites(code, target_ranges, EVENT_OFFSETS)
    boundary_sites = direct_sites(code, target_ranges, BOUNDARY_OFFSETS)
    actual_static = {
        "event": by_offset(code, event_sites),
        "boundary": by_offset(code, boundary_sites),
    }
    if actual_static != EXPECTED_STATIC:
        raise SystemExit(f"REFUSE: direct metadata census differs: {actual_static}")
    if event_sites & boundary_sites:
        raise SystemExit("REFUSE: event/boundary direct metadata sets overlap")

    event_counts, raw_totals = raw_counts(event_sites)
    boundary_counts, boundary_totals = raw_counts(boundary_sites)
    if raw_totals != boundary_totals:
        raise SystemExit("REFUSE: direct metadata raw totals differ")
    for band in PROFILE_WEIGHTS:
        actual = {"event": event_counts[band], "boundary": boundary_counts[band]}
        if actual != EXPECTED_DYNAMIC[band]:
            raise SystemExit(f"REFUSE: {band} direct metadata counts differ: {actual}")

    outlined_pairs, outlined_unpaired = pair_copies(
        event.OUTLINED_COPY_CALLS, boundary.OUTLINED_COPY_CALLS
    )
    direct_pairs, direct_unpaired = pair_copies(
        event.DIRECT_COPY_CALLS, boundary.DIRECT_COPY_CALLS
    )
    if outlined_unpaired != {0x100068BD8} or direct_unpaired:
        raise SystemExit(
            "REFUSE: event/boundary copy pairing differs "
            f"outlined_unpaired={outlined_unpaired} direct_unpaired={direct_unpaired}"
        )

    maximum_metadata_distance = max(
        min(abs(boundary_site - event_site) for event_site in event_sites)
        for boundary_site in boundary_sites
    )
    if maximum_metadata_distance > 84:
        raise SystemExit("REFUSE: boundary/event metadata locality differs")

    boundary_direct_ns = FLOOR_NS * sum(
        PROFILE_WEIGHTS[band] * boundary_counts[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )
    paired_boundary_copy_absolute_ns = 2.0 * (
        0.246096876 + 2.617411483
    )
    one_allocator_absolute_ns = 1.687336492

    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print(
        "raw custody: "
        + " ".join(
            f"{band}={known.RAW_SHA256[band]}" for band in PROFILE_WEIGHTS
        )
    )
    print(
        "source custody: PASS "
        + " ".join(f"{path}={digest}" for path, digest in SOURCE_PINS.items())
    )
    print("prior event/boundary classifiers: byte-exact reproduction PASS")
    print(EXPECTED_LAYOUT_OUTPUT.strip())
    print()
    print("SOURCE ORDER / LIFETIME")
    print(
        "match order: event and boundary appends follow AST execution order; "
        "Or/Quant/Opt placeholders precede their nested segments"
    )
    print(
        "patching: placeholders carry no absolute tape indices; a wide escape may "
        "insert its payload immediately after final compaction/iteration"
    )
    print(
        "build order: the same AST walk consumes event or boundary records at the "
        "corresponding site; lookahead discards all records"
    )
    print(
        "memo/orchestrator: one contiguous word slice preserves segment replay, "
        "nesting, mark/truncate stack discipline, and arena-reference lifetime"
    )
    print(
        "lossless carrier: tag0=unaltered aligned boundary pointer; tags1..6="
        "event variants; tag7=wide-event escape + raw usize payload word"
    )
    print(
        "safety: tag checked before the sole encapsulated boundary dereference; "
        "event words are provenance-free raw pointers and are never dereferenced"
    )
    print()
    print("SOURCE OPERATION CENSUS")
    print(
        "cascade "
        + " ".join(f"{needle}={count}" for needle, count in CASCADE_COUNTS.items())
    )
    print(
        "generator "
        + " ".join(f"{needle}={count}" for needle, count in GENERATOR_COUNTS.items())
    )
    print()
    print("MACHINE PAIRING")
    print(
        f"outlined boundary={len(boundary.OUTLINED_COPY_CALLS)} "
        f"paired={len(outlined_pairs)} event_unpaired={len(outlined_unpaired)} "
        f"diff_range={min(p[2] for p in outlined_pairs):#x}..{max(p[2] for p in outlined_pairs):#x}"
    )
    print(
        f"direct boundary={len(boundary.DIRECT_COPY_CALLS)} "
        f"paired={len(direct_pairs)} event_unpaired={len(direct_unpaired)} "
        f"diff_range={min(p[2] for p in direct_pairs):#x}..{max(p[2] for p in direct_pairs):#x}"
    )
    print(
        f"direct metadata static event={len(event_sites)} boundary={len(boundary_sites)} "
        f"boundary_nearest_event_max_bytes={maximum_metadata_distance} overlap=0"
    )
    print()
    print("DYNAMIC DIRECT METADATA")
    for band in PROFILE_WEIGHTS:
        print(
            f"band={band:8} raw={raw_totals[band]:5d} "
            f"event={event_counts[band]:3d} boundary={boundary_counts[band]:3d} "
            f"resum={event_counts[band] + boundary_counts[band]:3d}"
        )
    print()
    print("REPLACEMENT-AWARE PRICING")
    print("strict_new_removable_ns=0.000000000")
    print(f"boundary_direct_metadata_gross_ceiling_ns={boundary_direct_ns:.9f}")
    print(
        f"paired_boundary_copy_whole_child_impossible_ceiling_ns="
        f"{paired_boundary_copy_absolute_ns:.9f}"
    )
    print(f"one_allocator_whole_child_absolute_ceiling_ns={one_allocator_absolute_ns:.9f}")
    for label, contribution in (
        ("strict", 0.0),
        ("plus_direct_gross", boundary_direct_ns),
        ("plus_allocator_absolute", boundary_direct_ns + one_allocator_absolute_ns),
        (
            "plus_impossible_copy",
            boundary_direct_ns + one_allocator_absolute_ns + paired_boundary_copy_absolute_ns,
        ),
    ):
        composite = PRE_UNIFIED_STRICT_NS + contribution
        print(
            f"view={label:24} unified_ns={contribution:.9f} "
            f"composite_ns={composite:.9f} capture_0.30_ns={0.30 * composite:.9f} "
            f"noise_margin_ns={0.30 * composite - NOISE_NS:.9f}"
        )
    print(
        "overlap: event one-word byte saving=-0191 excluded; G1-C pushes/growth "
        "excluded; thin-memo success segments=accepted ranges excluded; boundary "
        "copy bytes and one surviving allocation remain required"
    )
    print(
        "verdict=FEASIBLE-HOLD (the safe 8-byte carrier is lossless, but its "
        "boundary tag check/mask and variable-width decode debit are unpriced; "
        "no strict floor-clearing contribution may be banked without an A/B)"
    )
    print("UNIFIED DERIVATION TAPE PRICING: PASS")


if __name__ == "__main__":
    main()
