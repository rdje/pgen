#!/usr/bin/env python3
"""Audit the monotone furthest-position carrier in the preserved fused regions."""

from __future__ import annotations

import hashlib
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
FIELD_DIR = TASK_ARTIFACTS / "parser_direct_field"
INPUT_DIR = TASK_ARTIFACTS / "input_view_carrier"
sys.path[:0] = [str(CAPTURE_DIR), str(FIELD_DIR), str(INPUT_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_input_view_carrier as input_view  # noqa: E402
import classify_mechanisms as known  # noqa: E402


SOURCE_PINS = {
    "rust/src/ast_pipeline/ast_based_generator.rs": "b5ed80b80adfcf1a6fb5df38884e249e06eff2f32a5e78dfef8760d256f374df",
    "rust/src/ast_pipeline/ast_based_generator/cascade.rs": "413fefe99dbfe8d4a03886f4eb4999d078fb03696f861ee1e7fefc23db03a100",
    "rust/src/ast_pipeline/ast_based_generator/scan.rs": "4b45dfd4f8e635f461ac12bae9ed46111f0a1b5f19f823753b2cc0a100b75b1e",
}
PROBE_SOURCE_SHA256 = "b9b683133d19c20e2fa5e924e91d8f924d0b5aec138e2bf1180dd645fe8b30ee"
RUSTC_VERSION = "rustc 1.95.0 (59807616e 2026-04-14)"
ENTRY_FURTHEST_LOAD = 0x100058D4C
ENTRY_POSITION_LOAD = 0x100058D48
FLOOR_NS = 1263.4
NOISE_NS = 28.8
PRE_POSITION_STRICT_NS = 102.138976561
PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
EXPECTED_DIRECT_ROW = {"sub1us": 66, "1to2p5": 73, "2p5to20": 69}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_custody() -> None:
    texts: dict[str, str] = {}
    for relative, expected in SOURCE_PINS.items():
        path = REPO_ROOT / relative
        actual = sha256(path)
        if actual != expected:
            raise SystemExit(f"REFUSE: source pin differs for {relative}: {actual}")
        texts[relative] = path.read_text()
    generator = texts["rust/src/ast_pipeline/ast_based_generator.rs"]
    cascade = texts["rust/src/ast_pipeline/ast_based_generator/cascade.rs"]
    scan = texts["rust/src/ast_pipeline/ast_based_generator/scan.rs"]
    required = (
        (generator, "pub fn furthest_position(&self) -> usize"),
        (generator, "if self.position > self.furthest_position"),
        (generator, "if parser.position > parser.furthest_position"),
        (generator, "if parse_start + #w > parser.furthest_position"),
        (cascade, "if parser.position > parser.furthest_position"),
        (scan, "if parser.position > parser.furthest_position"),
    )
    missing = [snippet for text, snippet in required if snippet not in text]
    if missing:
        raise SystemExit(f"REFUSE: furthest source contract differs: {missing}")
    if "furthest_position: 0," not in generator:
        raise SystemExit("REFUSE: furthest initialization differs")
    if re.search(r"furthest_position\s*-\=|furthest_position\s*=\s*0", generator + cascade + scan):
        raise SystemExit("REFUSE: furthest carrier is no longer monotone")


def probe_custody() -> None:
    source = ARTIFACT_DIR / "position_carrier_probe.rs"
    if sha256(source) != PROBE_SOURCE_SHA256:
        raise SystemExit("REFUSE: position-carrier probe source differs")
    version = subprocess.run(
        ["rustc", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    if version != RUSTC_VERSION:
        raise SystemExit(f"REFUSE: rustc differs: {version}")
    binary = Path("/tmp/pgen_position_carrier_probe")
    subprocess.run(
        [
            "rustc",
            str(source),
            "-O",
            "-C",
            "lto=fat",
            "-C",
            "codegen-units=1",
            "-o",
            str(binary),
        ],
        check=True,
    )
    output = subprocess.run(
        [str(binary)], check=True, capture_output=True, text=True
    ).stdout
    expected = "baseline=7 forwarded_ref=7 value=7 equal=true\n"
    if output != expected:
        raise SystemExit(f"REFUSE: position-carrier probe result differs: {output!r}")
    disassembly = subprocess.run(
        ["otool", "-tV", str(binary)], check=True, capture_output=True, text=True
    ).stdout
    bodies: dict[str, str] = {}
    for name in ("baseline_update", "forwarded_ref_update", "value_update"):
        match = re.search(
            rf"_{name}:\n(?P<body>.*?)(?=\n_[A-Za-z0-9_]+:)",
            disassembly,
            re.S,
        )
        if match is None:
            raise SystemExit(f"REFUSE: probe symbol missing: {name}")
        bodies[name] = match.group("body")
    for name in ("baseline_update", "forwarded_ref_update"):
        body = bodies[name]
        for mnemonic in ("cmp", "b.ls", "str"):
            if mnemonic not in body:
                raise SystemExit(f"REFUSE: {name} lacks {mnemonic}")
    if "ldp" not in bodies["baseline_update"]:
        raise SystemExit("REFUSE: baseline update no longer loads the field pair")
    if "[x1]" not in bodies["forwarded_ref_update"]:
        raise SystemExit("REFUSE: forwarded reference no longer reloads the carrier")
    value = bodies["value_update"]
    if "csel" not in value or any(mnemonic in value for mnemonic in ("ldr", "str")):
        raise SystemExit("REFUSE: by-value max codegen shape differs")


def update_sites(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> dict[str, set[int]]:
    x20_loads = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if code.get(address, ("", ""))[0] == "ldr"
        and "[x20, #0x4e0]" in code[address][1]
    }
    if len(x20_loads) != 39:
        raise SystemExit(f"REFUSE: inner furthest-load census differs: {len(x20_loads)}")
    if code.get(ENTRY_FURTHEST_LOAD) != ("ldr", "x8, [x1, #0x4e0]"):
        raise SystemExit("REFUSE: piece-entry furthest load differs")
    if code.get(ENTRY_POSITION_LOAD) != ("ldr", "x21, [x1, #0x498]"):
        raise SystemExit("REFUSE: piece-entry position load differs")
    furthest_loads = x20_loads | {ENTRY_FURTHEST_LOAD}
    stores = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if code.get(address, ("", ""))[0] == "str"
        and "[x20, #0x4e0]" in code[address][1]
    }
    if len(stores) != 43:
        raise SystemExit(f"REFUSE: furthest-store census differs: {len(stores)}")

    compares: set[int] = set()
    branches: set[int] = set()
    for store in stores:
        branch_candidates = [
            address
            for address in range(store - 12, store, 4)
            if code.get(address, ("", ""))[0] == "b.ls"
        ]
        if len(branch_candidates) != 1:
            raise SystemExit(f"REFUSE: update branch differs at {store:#x}")
        branch = branch_candidates[0]
        compare_candidates = [
            address
            for address in range(branch - 12, branch, 4)
            if code.get(address, ("", ""))[0] == "cmp"
        ]
        if not compare_candidates:
            raise SystemExit(f"REFUSE: update compare absent at {store:#x}")
        compare = max(compare_candidates)
        source = code[store][1].split(",", 1)[0]
        if source not in code[compare][1]:
            raise SystemExit(f"REFUSE: stored max source not compared at {store:#x}")
        target = int(code[branch][1], 16)
        if target < store + 4:
            raise SystemExit(f"REFUSE: branch does not guard furthest store at {store:#x}")
        compares.add(compare)
        branches.add(branch)
    if len(compares) != 43 or len(branches) != 43:
        raise SystemExit("REFUSE: compare/branch update census differs")

    position_accesses = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if "[x20, #0x498]" in code.get(address, ("", ""))[1]
        and code[address][0] in {"ldr", "str"}
    }
    if len(position_accesses) != 227:
        raise SystemExit(
            f"REFUSE: required position-carrier census differs: {len(position_accesses)}"
        )
    return {
        "entry_load": {ENTRY_FURTHEST_LOAD},
        "inner_load": x20_loads,
        "all_load": furthest_loads,
        "store": stores,
        "compare": compares,
        "branch": branches,
        "position": position_accesses,
    }


def weighted_ns(counts: dict[str, int], raw_totals: dict[str, int]) -> float:
    return FLOOR_NS * sum(
        PROFILE_WEIGHTS[band] * counts[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )


def main() -> None:
    source_custody()
    probe_custody()
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe differs")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    sites = update_sites(code, target_ranges)

    all_owned = set().union(
        sites["all_load"], sites["store"], sites["compare"], sites["branch"]
    )
    accepted = {address: known.owner(address) for address in all_owned if known.owner(address)}
    if accepted:
        raise SystemExit(f"REFUSE: accepted overlap: {accepted}")
    input_view.overlap_custody(code, target_ranges, all_owned)

    band_pcs: dict[str, Counter[int]] = {}
    raw_totals: dict[str, int] = {}
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: raw custody differs for {band}")
        header, pcs = capture.parse_raw(raw)
        raw_totals[band] = len(pcs)
        slide = int(header["main_slide"], 16)
        band_pcs[band] = Counter(pc - slide for pc in pcs)

    dynamic: dict[str, dict[str, int]] = {}
    for label, owned in sites.items():
        dynamic[label] = {
            band: sum(band_pcs[band][address] for address in owned)
            for band, *_ in capture.BANDS
        }
        print(
            f"dynamic {label:10} static={len(owned):3} "
            + " ".join(
                f"{band}={dynamic[label][band]}" for band, *_ in capture.BANDS
            )
        )

    direct_row = {
        band: dynamic["inner_load"][band]
        + dynamic["store"][band]
        + sum(
            band_pcs[band][address]
            for address in sites["position"]
        )
        for band, *_ in capture.BANDS
    }
    if direct_row != EXPECTED_DIRECT_ROW:
        raise SystemExit(f"REFUSE: direct position row differs: {direct_row}")

    entry_ns = weighted_ns(dynamic["entry_load"], raw_totals)
    inner_ceiling_ns = weighted_ns(dynamic["inner_load"], raw_totals)
    all_load_ns = weighted_ns(dynamic["all_load"], raw_totals)
    store_ns = weighted_ns(dynamic["store"], raw_totals)
    control_ns = weighted_ns(
        {
            band: dynamic["compare"][band] + dynamic["branch"][band]
            for band, *_ in capture.BANDS
        },
        raw_totals,
    )
    direct_row_ns = weighted_ns(EXPECTED_DIRECT_ROW, raw_totals)
    gross_composite = PRE_POSITION_STRICT_NS + inner_ceiling_ns
    print(
        f"summary updates=43 furthest_loads=40 inner_loads=39 stores=43 "
        f"compares=43 branches=43 position_accesses=227"
    )
    print(
        f"pricing entry_required={entry_ns:.9f} inner_load_ceiling={inner_ceiling_ns:.9f} "
        f"all_load={all_load_ns:.9f} stores={store_ns:.9f} "
        f"required_control={control_ns:.9f} direct_row={direct_row_ns:.9f}"
    )
    print(
        f"accumulated strict={PRE_POSITION_STRICT_NS:.9f} "
        f"capture30={0.30 * PRE_POSITION_STRICT_NS:.9f} "
        f"margin={0.30 * PRE_POSITION_STRICT_NS - NOISE_NS:.9f}"
    )
    print(
        f"accumulated gross_inner={gross_composite:.9f} "
        f"capture30={0.30 * gross_composite:.9f} "
        f"margin={0.30 * gross_composite - NOISE_NS:.9f}"
    )


if __name__ == "__main__":
    main()
