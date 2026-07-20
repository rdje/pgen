#!/usr/bin/env python3
"""Audit and price an immutable input view for the fused match graph.

Only parser-field loads of the immutable `&str` data/length words are priced.
Bounds comparisons, conditional branches, byte loads, and the future view's
entry load/ABI/register-pressure cost remain required and receive zero credit.
"""

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
BARE_DIR = TASK_ARTIFACTS / "bare_diagnostic_expansion"
LOOKUP_DIR = TASK_ARTIFACTS / "thin_memo_lookup_expansion"
TELEMETRY_DIR = TASK_ARTIFACTS / "semantic_telemetry_expansion"
GUARD_DIR = TASK_ARTIFACTS / "recursion_guard_expansion"
ROLLBACK_DIR = TASK_ARTIFACTS / "try_parse_name_rollback"
EVENT_DIR = TASK_ARTIFACTS / "deriv_event_width"
NAME_DIR = TASK_ARTIFACTS / "recursion_name_growth"
sys.path[:0] = [
    str(CAPTURE_DIR),
    str(FIELD_DIR),
    str(BARE_DIR),
    str(LOOKUP_DIR),
    str(TELEMETRY_DIR),
    str(GUARD_DIR),
    str(ROLLBACK_DIR),
    str(EVENT_DIR),
    str(NAME_DIR),
]

import analyze_capture as capture  # noqa: E402
import classify_bare_diagnostics as bare  # noqa: E402
import classify_deriv_event_width as event_width  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_parser_fields as fields  # noqa: E402
import classify_recursion_guard as guard  # noqa: E402
import classify_recursion_name_growth as name_growth  # noqa: E402
import classify_semantic_telemetry as telemetry  # noqa: E402
import classify_thin_memo_lookup as thin_lookup  # noqa: E402
import classify_try_parse_name_rollback as rollback  # noqa: E402


FLOOR_NS = 1263.4
NOISE_NS = 28.8
PRE_INPUT_STRICT_NS = 101.279665699
PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}
SOURCE_PINS = {
    "rust/src/ast_pipeline/ast_based_generator.rs": "b5ed80b80adfcf1a6fb5df38884e249e06eff2f32a5e78dfef8760d256f374df",
    "rust/src/ast_pipeline/ast_based_generator/cascade.rs": "413fefe99dbfe8d4a03886f4eb4999d078fb03696f861ee1e7fefc23db03a100",
}
PROBE_SOURCE_SHA256 = "bf899af04d0a4775b2c5ff52d9049fcc2af4ae67375cae94028f66c82dc3b778"
RUSTC_VERSION = "rustc 1.95.0 (59807616e 2026-04-14)"
EXPECTED_LEN_COUNTS = {"sub1us": 226, "1to2p5": 205, "2p5to20": 297}
EXPECTED_DATA_COUNTS = {"sub1us": 4, "1to2p5": 4, "2p5to20": 11}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_custody() -> None:
    for relative, expected in SOURCE_PINS.items():
        actual = sha256(REPO_ROOT / relative)
        if actual != expected:
            raise SystemExit(f"REFUSE: source pin differs for {relative}: {actual}")
    if sha256(ARTIFACT_DIR / "input_view_probe.rs") != PROBE_SOURCE_SHA256:
        raise SystemExit("REFUSE: input-view probe source differs")

    generator = (REPO_ROOT / "rust/src/ast_pipeline/ast_based_generator.rs").read_text()
    cascade = (
        REPO_ROOT / "rust/src/ast_pipeline/ast_based_generator/cascade.rs"
    ).read_text()
    if generator.count("input: &'input str,") != 2:
        raise SystemExit("REFUSE: parser input declaration/constructor shape differs")
    assignments = re.findall(r"(?:self|parser)\.input\s*=", generator + cascade)
    if assignments:
        raise SystemExit(f"REFUSE: parser input is mutable: {assignments}")
    if cascade.count("parser.input.len()") != 3:
        raise SystemExit("REFUSE: fused-match input-length template count differs")
    if cascade.count("parser.input.as_bytes()") != 4:
        raise SystemExit("REFUSE: fused-match input-byte template count differs")
    if cascade.count("fn #match_fn(&mut self) -> ParseResult<()>") != 2:
        raise SystemExit("REFUSE: fused match signature templates differ")


def probe_disassembly() -> dict[str, list[tuple[str, str]]]:
    actual_version = subprocess.run(
        ["rustc", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    if actual_version != RUSTC_VERSION:
        raise SystemExit(f"REFUSE: rustc differs: {actual_version}")
    binary = Path("/tmp/pgen_input_view_probe")
    subprocess.run(
        [
            "rustc",
            str(ARTIFACT_DIR / "input_view_probe.rs"),
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
    expected = "baseline=98 forwarded_len=98 forwarded_view=98 equal=true\n"
    if output != expected:
        raise SystemExit(f"REFUSE: input-view probe result differs: {output!r}")

    disassembly = subprocess.run(
        ["otool", "-tV", str(binary)], check=True, capture_output=True, text=True
    ).stdout
    functions: dict[str, list[tuple[str, str]]] = {}
    current: str | None = None
    wanted = {
        "baseline_input_read",
        "forwarded_len_read",
        "forwarded_view_read",
        "safe_len_orchestrator",
        "safe_view_orchestrator",
    }
    for line in disassembly.splitlines():
        symbol_match = re.fullmatch(r"_([A-Za-z0-9_]+):", line)
        if symbol_match:
            name = symbol_match.group(1)
            current = name if name in wanted else None
            if current is not None:
                functions[current] = []
            continue
        if current is None:
            continue
        instruction_match = re.fullmatch(
            r"[0-9a-f]+\s+([a-z0-9.]+)\s*(.*)", line
        )
        if instruction_match:
            functions[current].append(
                (instruction_match.group(1), instruction_match.group(2))
            )
    if set(functions) != wanted:
        raise SystemExit(f"REFUSE: probe symbols differ: {set(functions)}")

    rendered = {
        name: "\n".join(f"{mnemonic} {operands}" for mnemonic, operands in rows)
        for name, rows in functions.items()
    }
    if "[x0, #0x8]" not in rendered["baseline_input_read"]:
        raise SystemExit("REFUSE: baseline lacks parser input-length load")
    if "[x0]" not in rendered["baseline_input_read"]:
        raise SystemExit("REFUSE: baseline lacks parser input-data load")
    # Passing only a scalar length is insufficient: safe slice indexing still
    # checks against the actual parser.input length and reloads that field.
    if "[x8, #0x8]" not in rendered["forwarded_len_read"]:
        raise SystemExit("REFUSE: scalar-forwarding bounds reload disappeared")
    if "[x0, #0x8]" not in rendered["safe_len_orchestrator"]:
        raise SystemExit("REFUSE: scalar orchestrator entry load differs")
    # Passing the complete shared byte slice carries both actual bounds and
    # data. The match callee reads only parser.position; input comes from x1/x2.
    forwarded = rendered["forwarded_view_read"]
    if "[x0, #0x8]" in forwarded or re.search(r"\[x0\](?:\n|$)", forwarded):
        raise SystemExit("REFUSE: full-view callee still reloads parser input")
    if "ldrb w0, [x1, x8]" not in forwarded:
        raise SystemExit("REFUSE: full-view callee does not index forwarded slice")
    if "ldp x1, x2, [x0]" not in rendered["safe_view_orchestrator"]:
        raise SystemExit("REFUSE: full-view orchestrator entry load differs")
    return functions


def input_sites(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> tuple[dict[str, Counter[tuple[str, int]]], set[int], set[int], dict[str, int]]:
    band_counts: dict[str, Counter[tuple[str, int]]] = {}
    raw_totals: dict[str, int] = {}
    for band, *_ in capture.BANDS:
        raw = CAPTURE_DIR / f"capture_band_{band}_raw_pcs.txt"
        if known.sha256(raw) != known.RAW_SHA256[band]:
            raise SystemExit(f"REFUSE: {band} raw SHA-256 mismatch")
        header, pcs = capture.parse_raw(raw)
        raw_totals[band] = len(pcs)
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
    len_sites = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if address in code
        and fields.is_parser_direct(code[address])
        and fields.parser_offset(code[address][1]) == 0x2F8
        and known.owner(address) is None
    }
    data_sites = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if address in code
        and fields.is_parser_direct(code[address])
        and fields.parser_offset(code[address][1]) == 0x2F0
        and known.owner(address) is None
    }
    if len(len_sites) != 93 or len(data_sites) != 77:
        raise SystemExit(
            f"REFUSE: input site counts differ len={len(len_sites)} data={len(data_sites)}"
        )

    for address in len_sites:
        mnemonic, operands = code[address]
        destination = operands.split(",", 1)[0]
        following = [code.get(candidate, ("", "")) for candidate in range(address + 4, address + 24, 4)]
        if mnemonic != "ldr" or not any(
            child_mnemonic == "cmp"
            and destination in re.findall(r"x\d+", child_operands)
            for child_mnemonic, child_operands in following
        ):
            raise SystemExit(f"REFUSE: input-length bounds consumer differs at {address:#x}")
    for address in data_sites:
        mnemonic, operands = code[address]
        destination = operands.split(",", 1)[0]
        following = [code.get(candidate, ("", "")) for candidate in range(address + 4, address + 36, 4)]
        aliases = {destination}
        for child_mnemonic, child_operands in following:
            add_match = re.fullmatch(r"(x\d+), (x\d+), .+", child_operands)
            if (
                child_mnemonic == "add"
                and add_match is not None
                and add_match.group(2) in aliases
            ):
                aliases.add(add_match.group(1))
        if mnemonic != "ldr" or not any(
            child_mnemonic.startswith(("ldr", "ldur"))
            and any(f"[{alias}" in child_operands for alias in aliases)
            for child_mnemonic, child_operands in following
        ):
            raise SystemExit(f"REFUSE: input-data byte consumer differs at {address:#x}")

    len_counts = {
        band: sum(samples for (_, address), samples in counts.items() if address in len_sites)
        for band, counts in band_counts.items()
    }
    data_counts = {
        band: sum(samples for (_, address), samples in counts.items() if address in data_sites)
        for band, counts in band_counts.items()
    }
    if len_counts != EXPECTED_LEN_COUNTS or data_counts != EXPECTED_DATA_COUNTS:
        raise SystemExit(
            f"REFUSE: dynamic input counts differ len={len_counts} data={data_counts}"
        )
    return band_counts, len_sites, data_sites, raw_totals


def overlap_custody(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
    owned: set[int],
) -> None:
    accepted = {address: known.owner(address) for address in owned if known.owner(address)}
    if accepted:
        raise SystemExit(f"REFUSE: accepted overlap: {accepted}")
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
        code,
        target_ranges,
        guard.INLINE_NAME_PUSH | set().union(*(pcs for _, _, pcs in pops), set()),
    )
    rollback_union = set().union(*(pcs for *_, pcs in rollback_sites), set())
    build_loads = event_width.discover_build_payload_loads(
        code, event_width.symbol_table()
    )
    event_union = (
        set(event_width.PAYLOAD_STORES)
        | build_loads
        | set(event_width.EVENT_GROW_CALLS)
        | set(event_width.OUTLINED_COPY_CALLS)
        | set(event_width.DIRECT_COPY_CALLS)
        | {event_width.EVENT_ALLOC_CALL}
    )
    prior_name_calls = {
        site.call_pc for site in name_growth.CALLSITES
    }
    overlaps = {
        "bare_0185": len(owned & (trace | coverage | nonempty)),
        "thin_lookup_0186": len(owned & lookup_full),
        "telemetry_0187": len(owned & telemetry_union),
        "recursion_guard_0188": len(owned & guard_union),
        "try_parse_rollback_0189": len(owned & rollback_union),
        "name_growth_0190": len(owned & prior_name_calls),
        "event_width_0191": len(owned & event_union),
    }
    if any(overlaps.values()):
        raise SystemExit(f"REFUSE: accepted/held overlap: {overlaps}")


def weighted_ns(counts: dict[str, int], raw_totals: dict[str, int]) -> float:
    return FLOOR_NS * sum(
        PROFILE_WEIGHTS[band] * counts[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )


def main() -> None:
    source_custody()
    probe_disassembly()
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    _, len_sites, data_sites, raw_totals = input_sites(code, target_ranges)
    overlap_custody(code, target_ranges, len_sites | data_sites)

    len_ns = weighted_ns(EXPECTED_LEN_COUNTS, raw_totals)
    data_ns = weighted_ns(EXPECTED_DATA_COUNTS, raw_totals)
    gross_ns = len_ns + data_ns
    gross_composite = PRE_INPUT_STRICT_NS + gross_ns

    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    print(
        "source custody: PASS "
        + " ".join(f"{path}={digest}" for path, digest in SOURCE_PINS.items())
    )
    print(
        f"feasibility probe: PASS source={PROBE_SOURCE_SHA256} rustc='{RUSTC_VERSION}' "
        "baseline=field-data+field-len scalar=REFUTED(full safe bounds reload remains) "
        "full_byte_slice=FEASIBLE(entry ldp; callee has no parser-input reload)"
    )
    print(
        "source semantics: input=&'input str initialized once assignments=0; "
        "fused_match_templates len=3 bytes=4; build string slices excluded"
    )
    print(
        f"static: len_field_loads={len(len_sites)} data_field_loads={len(data_sites)} "
        "sampled_pcs=50+14 len_consumers=bounds_cmp data_consumers=byte_load "
        "accepted_held_overlap=0"
    )
    print("required: bounds compares + conditional branches + input byte loads + safe indexing")
    print(
        "replacement: orchestrator copies parser.input.as_bytes() once and forwards &'input [u8] "
        "through match-only calls; public API/build pass unchanged; no unsafe reads"
    )
    print()
    print("DYNAMIC PARSER-FIELD LOADS")
    for band in PROFILE_WEIGHTS:
        print(
            f"band={band:8} raw={raw_totals[band]:5d} "
            f"input_len={EXPECTED_LEN_COUNTS[band]:3d} "
            f"input_data={EXPECTED_DATA_COUNTS[band]:2d} "
            f"total={EXPECTED_LEN_COUNTS[band] + EXPECTED_DATA_COUNTS[band]:3d}"
        )
    print()
    print("REPLACEMENT-AWARE PRICING")
    print(f"gross_current_input_len_load_ceiling_ns={len_ns:.9f}")
    print(f"gross_current_input_data_load_ceiling_ns={data_ns:.9f}")
    print(f"gross_current_input_carrier_ceiling_ns={gross_ns:.9f}")
    print(
        "replacement_debits=entry_ldp+fat_slice_forwarding+register_pressure+possible_spills; "
        "unmeasured and therefore strict_net_ns=0.000000000"
    )
    print(
        f"view=strict_net              input_ns=0.000000000 composite_ns={PRE_INPUT_STRICT_NS:.9f} "
        f"capture_0.30_ns={0.30 * PRE_INPUT_STRICT_NS:.9f} "
        f"noise_margin_ns={0.30 * PRE_INPUT_STRICT_NS - NOISE_NS:.9f}"
    )
    print(
        f"view=gross_current_ceiling   input_ns={gross_ns:.9f} composite_ns={gross_composite:.9f} "
        f"capture_0.30_ns={0.30 * gross_composite:.9f} "
        f"noise_margin_ns={0.30 * gross_composite - NOISE_NS:.9f}"
    )
    print(
        "verdict=HOLD (the full-view carrier is safe, but 12.316 ns is a gross current-load "
        "ceiling and its 5.279 ns accumulated margin has no measured replacement debit)"
    )
    print("IMMUTABLE INPUT-VIEW CARRIER PRICING: PASS")


if __name__ == "__main__":
    main()
