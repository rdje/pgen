#!/usr/bin/env python3
"""Inspect the redundant scope_len word at all 40 checkpoint sites.

Thirty-nine sites are ordinary inlined ``try_parse`` checkpoints that the
banked stack-carrier classifier finds from their entry-trace literals.  The
40th is the trace-less outer C3-B tournament checkpoint at 0x100073f40.  It
must be added explicitly: a trace anchor is not a complete checkpoint census.
"""

from __future__ import annotations

import sys
import hashlib
import re
import subprocess
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
CAPTURE_DIR = TASK_ARTIFACTS / "three_band_pc_capture"
STACK_DIR = TASK_ARTIFACTS / "residual_stack_carrier"
FIELD_DIR = TASK_ARTIFACTS / "parser_direct_field"
INPUT_DIR = TASK_ARTIFACTS / "input_view_carrier"
sys.path[:0] = [str(CAPTURE_DIR), str(STACK_DIR), str(FIELD_DIR), str(INPUT_DIR)]

import analyze_capture as capture  # noqa: E402
import classify_mechanisms as known  # noqa: E402
import classify_parser_fields as fields  # noqa: E402
import classify_stack_carriers as stack  # noqa: E402
import classify_input_view_carrier as input_view  # noqa: E402


SEMANTIC_RUNTIME_SHA256 = "26ddf12ea89927a07f5d0f8acb79e8e5162cf0f0a32099dcb3746e35552f5701"
LAYOUT_PROBE_SHA256 = "33df32d2461a57313bb72c55bc6fb158cd4485a5c995d6a0aac8a640e19e826d"
RUSTC_VERSION = "rustc 1.95.0 (59807616e 2026-04-14)"
EXTRA_SITE = ("atom_closure", 0x100073F84, 0, "sp", 0x560)
EXTRA_SOURCE = 0x100073F40
EXTRA_STORES = set(range(0x100073F68, 0x100073F84, 4))
FLOOR_NS = 1263.4
NOISE_NS = 28.8
PRE_CHECKPOINT_STRICT_NS = 101.279665699
PROFILE_WEIGHTS = {"sub1us": 0.380, "1to2p5": 0.353, "2p5to20": 0.260}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_custody() -> None:
    source = REPO_ROOT / "rust/src/ast_pipeline/semantic_runtime.rs"
    if sha256(source) != SEMANTIC_RUNTIME_SHA256:
        raise SystemExit("REFUSE: semantic-runtime source differs")
    text = source.read_text()
    checkpoint_match = re.search(
        r"pub struct SemanticRuntimeCheckpoint \{(?P<body>.*?)\n\}", text, re.S
    )
    if checkpoint_match is None:
        raise SystemExit("REFUSE: checkpoint declaration missing")
    fields_found = re.findall(
        r"^\s*(scope_len|fact_len|deferred_len|scope_arena_len|chain_len|"
        r"chain_trail_len|write_epoch): (usize|u64),\s*$",
        checkpoint_match.group("body"),
        re.M,
    )
    expected_fields = [
        ("scope_len", "usize"),
        ("fact_len", "usize"),
        ("deferred_len", "usize"),
        ("scope_arena_len", "usize"),
        ("chain_len", "usize"),
        ("chain_trail_len", "usize"),
        ("write_epoch", "u64"),
    ]
    if fields_found != expected_fields:
        raise SystemExit(f"REFUSE: checkpoint fields differ: {fields_found}")

    # The two depth mirrors are initialized and mutated in lockstep.  The
    # whole-file source hash makes these exact snippets a complete-version
    # proof rather than a best-effort grep over an unpinned source tree.
    required = (
        "scopes: vec![SemanticScopeFrame {",
        "active_chain: smallvec![ScopeId::ROOT]",
        "self.active_chain.push(new_id);",
        "self.scopes.push(SemanticScopeFrame {",
        "let popped_id = self.active_chain.pop();",
        "let popped_frame = self.scopes.pop();",
        "self.scopes = delta.final_scopes;",
        "self.scopes = self\n                .active_chain",
        "final_active_chain: self.active_chain.clone(),",
        "final_scopes: self.scopes.clone(),",
    )
    missing = [snippet for snippet in required if snippet not in text]
    if missing:
        raise SystemExit(f"REFUSE: depth-lockstep proof differs: {missing}")
    mutation_counts = {
        "active_push": text.count("self.active_chain.push("),
        "active_pop": text.count("self.active_chain.pop()"),
        "scopes_push": text.count("self.scopes.push("),
        "scopes_pop": text.count("self.scopes.pop()"),
        "scopes_assign": text.count("self.scopes = "),
    }
    expected_counts = {
        "active_push": 2,
        "active_pop": 2,
        "scopes_push": 1,
        "scopes_pop": 1,
        "scopes_assign": 2,
    }
    if mutation_counts != expected_counts:
        raise SystemExit(
            f"REFUSE: depth-lockstep mutation census differs: {mutation_counts}"
        )


def layout_custody() -> None:
    probe = ARTIFACT_DIR / "checkpoint_layout_probe.rs"
    if sha256(probe) != LAYOUT_PROBE_SHA256:
        raise SystemExit("REFUSE: checkpoint layout probe differs")
    version = subprocess.run(
        ["rustc", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    if version != RUSTC_VERSION:
        raise SystemExit(f"REFUSE: rustc differs: {version}")
    binary = Path("/tmp/pgen_checkpoint_layout_probe")
    subprocess.run(
        ["rustc", str(probe), "-O", "-o", str(binary)], check=True
    )
    output = subprocess.run(
        [str(binary)], check=True, capture_output=True, text=True
    ).stdout
    expected = (
        "checkpoint7 size=56 align=8 drop=false\n"
        "checkpoint6 size=48 align=8 drop=false\n"
    )
    if output != expected:
        raise SystemExit(f"REFUSE: checkpoint layout differs: {output!r}")


def weighted_ns(counts: dict[str, int], raw_totals: dict[str, int]) -> float:
    return FLOOR_NS * sum(
        PROFILE_WEIGHTS[band] * counts[band] / raw_totals[band]
        for band in PROFILE_WEIGHTS
    )


def main() -> None:
    source_custody()
    layout_custody()
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: probe differs")
    code = capture.disassembly()
    target_ranges = capture.symbol_ranges()
    known.qualify_ranges(code, target_ranges)
    _, trace_sites = stack.checkpoint_store_pcs(code, target_ranges)
    if any(site[1] == EXTRA_SITE[1] for site in trace_sites):
        raise SystemExit("REFUSE: trace classifier unexpectedly owns C3-B checkpoint")
    if any(
        "Starting speculative parse" in code.get(address, ("", ""))[1]
        for address in range(EXTRA_SOURCE, EXTRA_SITE[1] + 4, 4)
    ):
        raise SystemExit("REFUSE: C3-B checkpoint unexpectedly has entry-trace anchor")
    if code.get(EXTRA_SOURCE) != ("ldr", "x8, [x20, #0xd8]"):
        raise SystemExit("REFUSE: C3-B checkpoint source fingerprint differs")
    if {
        address
        for address in EXTRA_STORES
        if stack.stack_write_words(*code.get(address, ("", ""))) is not None
    } != EXTRA_STORES:
        raise SystemExit("REFUSE: C3-B checkpoint seven-store fingerprint differs")
    if "extract_delta_since" not in code.get(0x1000751A4, ("", ""))[1]:
        raise SystemExit("REFUSE: C3-B extract consumer differs")
    if "rollback_to_labeled" not in code.get(0x10007526C, ("", ""))[1]:
        raise SystemExit("REFUSE: C3-B rollback consumer differs")
    sites = trace_sites + [EXTRA_SITE]

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

    exclusive: set[int] = set()
    paired: set[int] = set()
    all_materialization: set[int] = set()
    source_loads = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if code.get(address, ("", ""))[0] == "ldr"
        and "[x20, #0xd8]" in code[address][1]
        and known.owner(address) is None
    }
    selected_source_loads: set[int] = set()
    for target, guard, _, base, slot_start in sites:
        checkpoint_words = {slot_start + 8 * index for index in range(7)}
        word_stores: list[tuple[int, set[int]]] = []
        for address in range(guard - 64, guard, 4):
            decoded = stack.stack_write_words(*code.get(address, ("", "")))
            if decoded is None:
                continue
            store_base, words = decoded
            if store_base == base and words <= checkpoint_words:
                all_materialization.add(address)
            if store_base == base and slot_start in words:
                word_stores.append((address, words))
        if len(word_stores) != 1:
            raise SystemExit(
                f"REFUSE: scope word has {len(word_stores)} stores at {guard:#x}"
            )
        address, words = word_stores[0]
        if words == {slot_start}:
            exclusive.add(address)
            form = "exclusive"
        elif words == {slot_start, slot_start + 8}:
            paired.add(address)
            form = "paired_fact_len"
        else:
            raise SystemExit(f"REFUSE: unexpected scope-word store {words} at {address:#x}")

        loads = [
            candidate
            for candidate in range(target_ranges[target][0], guard, 4)
            if code.get(candidate, ("", ""))[0] == "ldr"
            and "[x20, #0xd8]" in code[candidate][1]
        ]
        if not loads:
            raise SystemExit(f"REFUSE: no scope_len source load before {guard:#x}")
        selected_source_loads.add(loads[-1])
        counts = "/".join(str(band_pcs[band][address]) for band, *_ in capture.BANDS)
        print(
            f"site target={target:12} guard={guard:#x} source={loads[-1]:#x} "
            f"store={address:#x} form={form:15} samples={counts}"
        )

    if len(sites) != 40 or len(selected_source_loads) != 40:
        raise SystemExit("REFUSE: complete checkpoint census is not 40 sites")
    if len(exclusive) != 28 or len(paired) != 12:
        raise SystemExit("REFUSE: checkpoint word-0 store-form census differs")
    if source_loads != selected_source_loads:
        raise SystemExit("REFUSE: not every scope_len source load maps to one checkpoint")
    if len(all_materialization) != 246:
        raise SystemExit(
            f"REFUSE: checkpoint materialization instruction census differs: "
            f"{len(all_materialization)}"
        )

    owned = source_loads | exclusive | paired
    input_view.overlap_custody(code, target_ranges, owned)
    input_sites = {
        address
        for start, end in target_ranges.values()
        for address in range(start, end, 4)
        if code.get(address, ("", ""))[0] == "ldr"
        and any(offset in code[address][1] for offset in ("#0x2f0", "#0x2f8"))
    }
    if owned & input_sites:
        raise SystemExit("REFUSE: checkpoint carrier overlaps held input-view sites")

    print(
        f"summary sites={len(sites)} source_loads={len(source_loads)} "
        f"trace_sites={len(trace_sites)} c3b_sites=1 "
        f"materialization_instructions={len(all_materialization)} "
        f"exclusive_stores={len(exclusive)} paired_stores={len(paired)}"
    )
    print(
        "unmapped_source_loads="
        + ",".join(f"{address:#x}" for address in sorted(source_loads - selected_source_loads))
    )
    for label, owned in (
        ("source", source_loads),
        ("all_materialize", all_materialization),
        ("exclusive_store", exclusive),
        ("paired_store", paired),
    ):
        counts = {
            band: sum(band_pcs[band][address] for address in owned)
            for band, *_ in capture.BANDS
        }
        print(
            f"dynamic {label:15} "
            + " ".join(f"{band}={counts[band]}" for band, *_ in capture.BANDS)
        )

    source_counts = {
        band: sum(band_pcs[band][address] for address in source_loads)
        for band, *_ in capture.BANDS
    }
    exclusive_counts = {
        band: sum(band_pcs[band][address] for address in exclusive)
        for band, *_ in capture.BANDS
    }
    paired_counts = {
        band: sum(band_pcs[band][address] for address in paired)
        for band, *_ in capture.BANDS
    }
    source_ns = weighted_ns(source_counts, raw_totals)
    exclusive_ns = weighted_ns(exclusive_counts, raw_totals)
    paired_ceiling_ns = weighted_ns(paired_counts, raw_totals)
    strict_ns = source_ns + exclusive_ns
    optimistic_ns = strict_ns + paired_ceiling_ns
    strict_composite = PRE_CHECKPOINT_STRICT_NS + strict_ns
    optimistic_composite = PRE_CHECKPOINT_STRICT_NS + optimistic_ns
    print(
        f"pricing source={source_ns:.9f} exclusive_store={exclusive_ns:.9f} "
        f"strict={strict_ns:.9f} paired_instruction_ceiling={paired_ceiling_ns:.9f} "
        f"optimistic={optimistic_ns:.9f}"
    )
    print(
        f"accumulated strict={strict_composite:.9f} "
        f"capture30={0.30 * strict_composite:.9f} "
        f"margin={0.30 * strict_composite - NOISE_NS:.9f}"
    )
    print(
        f"accumulated optimistic={optimistic_composite:.9f} "
        f"capture30={0.30 * optimistic_composite:.9f} "
        f"margin={0.30 * optimistic_composite - NOISE_NS:.9f}"
    )


if __name__ == "__main__":
    main()
