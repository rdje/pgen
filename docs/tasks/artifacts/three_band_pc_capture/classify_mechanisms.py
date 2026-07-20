#!/usr/bin/env python3
"""Exact, lower-bound mechanism attribution for the three-band PC capture.

This deliberately classifies only instruction ranges whose machine structure is
exclusive to one of the two already-banked mechanisms under test:

* G1-C: construct one 72-byte ParseNode, arena-allocate it, and append its
  address to deriv_boundary;
* thin-memo success: construct the event and boundary SmallVec segments and
  insert the successful thin-memo entry.

Everything else remains residual.  In particular, the generic memory class is
never converted to a mechanism or to nanoseconds.
"""

from __future__ import annotations

import hashlib
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

import analyze_capture as capture


ARTIFACT_DIR = Path(__file__).resolve().parent
RAW_SHA256 = {
    "sub1us": "88623effc4b4897029d5fa16453f22d377bb777778f586c074040e24db2bb7d2",
    "1to2p5": "459adfbca1b05dd1e2a7b60119de4d6120f4fcb5aac70d3c4d1d90ccefed9c12",
    "2p5to20": "901b03b95caa9b47e4340876e305794c9fa219f0cc4ce5c753df54800698903c",
}


@dataclass(frozen=True)
class CodeRange:
    mechanism: str
    site: str
    target: str
    start: int
    end: int
    role: str


def code_range(
    mechanism: str,
    site: str,
    target: str,
    start: int,
    end: int,
    role: str,
) -> CodeRange:
    return CodeRange(mechanism, site, target, start, end, role)


# Half-open, instruction-aligned ranges in the custody-pinned 1d3fa0ee probe.
# Each G1-C site has an inline arena block, its cold allocation block, and the
# join that appends the resulting pointer to deriv_boundary.  Panic-only
# RefCell blocks are excluded because they are not work removable from a valid
# parse.  These seven sites are every `parser.arena` load in the two target
# symbols; qualification below proves that census mechanically.
G1C_RANGES = (
    code_range("g1c", "piece.directive_name", "piece", 0x10005BD38, 0x10005BDB4, "arena_inline"),
    code_range("g1c", "piece.directive_name", "piece", 0x10005D590, 0x10005D5E0, "arena_slow"),
    code_range("g1c", "piece.directive_name", "piece", 0x10005D5E0, 0x10005D610, "boundary_push"),
    code_range("g1c", "atom.letter", "atom_closure", 0x10006B008, 0x10006B084, "arena_inline"),
    code_range("g1c", "atom.letter", "atom_closure", 0x10006B18C, 0x10006B1D4, "arena_slow"),
    code_range("g1c", "atom.letter", "atom_closure", 0x10006B1D4, 0x10006B20C, "boundary_push"),
    code_range("g1c", "atom.digit", "atom_closure", 0x10006BCC0, 0x10006BD40, "arena_inline"),
    code_range("g1c", "atom.digit", "atom_closure", 0x10006BD90, 0x10006BDDC, "arena_slow"),
    code_range("g1c", "atom.digit", "atom_closure", 0x10006BDDC, 0x10006BE14, "boundary_push"),
    code_range("g1c", "atom.pattern", "atom_closure", 0x1000708F8, 0x100070978, "arena_inline"),
    code_range("g1c", "atom.pattern", "atom_closure", 0x1000719FC, 0x100071A4C, "arena_slow"),
    code_range("g1c", "atom.pattern", "atom_closure", 0x100071A4C, 0x100071A84, "boundary_push"),
    code_range("g1c", "atom.directive", "atom_closure", 0x1000743EC, 0x100074470, "arena_inline"),
    code_range("g1c", "atom.directive", "atom_closure", 0x1000747B0, 0x1000747FC, "arena_slow"),
    code_range("g1c", "atom.directive", "atom_closure", 0x1000747FC, 0x100074828, "boundary_push"),
    code_range("g1c", "atom.builtin_any", "atom_closure", 0x10007946C, 0x1000794EC, "arena_inline"),
    code_range("g1c", "atom.builtin_any", "atom_closure", 0x1000794EC, 0x100079534, "arena_slow"),
    code_range("g1c", "atom.builtin_any", "atom_closure", 0x100079534, 0x100079564, "boundary_push"),
    code_range("g1c", "atom.named_group", "atom_closure", 0x10007AB2C, 0x10007ABAC, "arena_inline"),
    code_range("g1c", "atom.named_group", "atom_closure", 0x10007B72C, 0x10007B784, "arena_slow"),
    code_range("g1c", "atom.named_group", "atom_closure", 0x10007B784, 0x10007B7BC, "boundary_push"),
)


# `cascade_match_atom` is inlined into piece around the separately outlined
# atom body closure.  The two logical success paths below are the exact
# SmallVec::from_slice + successful thin_memo.insert regions for RULE_ATOM (27)
# and RULE_PIECE (8).  Failure-entry inserts are deliberately excluded: the
# BATCH-1 member is the pair of segment copies on a successful store.
THIN_MEMO_RANGES = (
    code_range("thin_memo_success", "rule_atom", "piece", 0x10005C5B0, 0x10005C5F0, "event_segment_inline"),
    code_range("thin_memo_success", "rule_atom", "piece", 0x10005CDB4, 0x10005CEBC, "spill_boundary_insert"),
    code_range("thin_memo_success", "rule_piece", "piece", 0x10005CB5C, 0x10005CBA0, "event_segment_inline"),
    code_range("thin_memo_success", "rule_piece", "piece", 0x10005CC2C, 0x10005CD34, "spill_boundary_insert"),
)

ALL_RANGES = G1C_RANGES + THIN_MEMO_RANGES


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def range_instructions(
    code: dict[int, tuple[str, str]], item: CodeRange
) -> list[tuple[int, str, str]]:
    return [
        (address, *code[address])
        for address in range(item.start, item.end, 4)
        if address in code
    ]


def qualify_ranges(
    code: dict[int, tuple[str, str]],
    target_ranges: dict[str, tuple[int, int]],
) -> None:
    occupied: dict[int, CodeRange] = {}
    for item in ALL_RANGES:
        target_start, target_end = target_ranges[item.target]
        if not (target_start <= item.start < item.end <= target_end):
            raise SystemExit(f"REFUSE: range outside {item.target}: {item}")
        if item.start % 4 or item.end % 4:
            raise SystemExit(f"REFUSE: unaligned range: {item}")
        rows = range_instructions(code, item)
        if len(rows) != (item.end - item.start) // 4:
            raise SystemExit(f"REFUSE: disassembly hole in range: {item}")
        for address, _, _ in rows:
            if address in occupied:
                raise SystemExit(
                    f"REFUSE: mechanism overlap at {address:#x}: "
                    f"{occupied[address]} versus {item}"
                )
            occupied[address] = item

    # Every target-symbol arena-field load is owned by exactly one G1-C inline
    # range; no site was selected by sampling outcome.
    arena_loads = {
        address
        for target_start, target_end in target_ranges.values()
        for address in range(target_start, target_end, 4)
        if code.get(address, ("", ""))[0] == "ldr"
        and "[x20, #0x300]" in code[address][1]
    }
    inline_starts = {
        item.start for item in G1C_RANGES if item.role == "arena_inline"
    }
    if arena_loads != inline_starts or len(arena_loads) != 7:
        raise SystemExit(
            f"REFUSE: G1-C arena census differs loads={sorted(arena_loads)} "
            f"starts={sorted(inline_starts)}"
        )

    for site in sorted({item.site for item in G1C_RANGES}):
        site_ranges = [item for item in G1C_RANGES if item.site == site]
        if {item.role for item in site_ranges} != {
            "arena_inline", "arena_slow", "boundary_push"
        }:
            raise SystemExit(f"REFUSE: incomplete G1-C site {site}")
        inline = next(item for item in site_ranges if item.role == "arena_inline")
        inline_text = "\n".join(
            f"{mnemonic} {operands}"
            for _, mnemonic, operands in range_instructions(code, inline)
        )
        required = ("#0x300", "#0x18", "#0x8", "#0x10", "lsl", "str q0")
        if any(token not in inline_text for token in required):
            raise SystemExit(f"REFUSE: G1-C 72-byte fingerprint differs at {site}")
        slow = next(item for item in site_ranges if item.role == "arena_slow")
        slow_text = "\n".join(
            f"{mnemonic} {operands}"
            for _, mnemonic, operands in range_instructions(code, slow)
        )
        if "alloc_slow_path" not in slow_text:
            raise SystemExit(f"REFUSE: G1-C slow-path fingerprint differs at {site}")
        push = next(item for item in site_ranges if item.role == "boundary_push")
        push_text = "\n".join(
            f"{mnemonic} {operands}"
            for _, mnemonic, operands in range_instructions(code, push)
        )
        if not all(field in push_text for field in ("#0x290", "#0x298", "#0x2a0")):
            raise SystemExit(f"REFUSE: G1-C boundary-push fingerprint differs at {site}")

    for site, rule_id in (("rule_atom", "#0x1b"), ("rule_piece", "#0x8")):
        site_ranges = [item for item in THIN_MEMO_RANGES if item.site == site]
        text = "\n".join(
            f"{mnemonic} {operands}"
            for item in site_ranges
            for _, mnemonic, operands in range_instructions(code, item)
        )
        required = (
            "[x20, #0x288]", "[x20, #0x280]", "[x20, #0x2a0]",
            "[x20, #0x298]", "_memcpy", "#0x478", rule_id,
            "HashMap$LT$K$C$V$C$S$C$A$GT$6insert",
        )
        if any(token not in text for token in required):
            raise SystemExit(f"REFUSE: thin-memo success fingerprint differs at {site}")

    print(
        "range qualification: PASS "
        f"g1c_sites={len(arena_loads)} thin_memo_success_sites=2 "
        "instruction_overlap=0"
    )


def owner(address: int) -> CodeRange | None:
    matches = [item for item in ALL_RANGES if item.start <= address < item.end]
    if len(matches) > 1:
        raise SystemExit(f"REFUSE: runtime overlap at {address:#x}")
    return matches[0] if matches else None


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    target_ranges = capture.symbol_ranges()
    code = capture.disassembly()
    print(f"probe custody: PASS sha256={capture.PROBE_SHA256}")
    qualify_ranges(code, target_ranges)

    for name, *_ in capture.BANDS:
        raw_path = ARTIFACT_DIR / f"capture_band_{name}_raw_pcs.txt"
        actual_sha = sha256(raw_path)
        if actual_sha != RAW_SHA256[name]:
            raise SystemExit(f"REFUSE: raw capture SHA-256 mismatch for {name}")
        header, pcs = capture.parse_raw(raw_path)
        slide = int(header["main_slide"], 16)
        counts: Counter[str] = Counter()
        classes: Counter[tuple[str, str]] = Counter()
        sites: Counter[tuple[str, str]] = Counter()
        residual_memory_bases: Counter[str] = Counter()
        residual_vector_memory = 0
        target_total = 0
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
            if target is None:
                continue
            target_total += 1
            item = owner(address)
            mechanism = item.mechanism if item else "residual"
            counts[mechanism] += 1
            mnemonic, _ = code[address]
            classes[(mechanism, capture.instruction_class(mnemonic))] += 1
            if mechanism == "residual" and capture.instruction_class(mnemonic) == "memory":
                operands = code[address][1]
                if "[sp" in operands or "[x29" in operands:
                    residual_memory_bases["stack_frame"] += 1
                elif "[x20" in operands:
                    residual_memory_bases["parser_direct"] += 1
                else:
                    residual_memory_bases["indirect"] += 1
                if any(f"q{register}" in operands for register in range(8)):
                    residual_vector_memory += 1
            if item:
                sites[(item.mechanism, item.site)] += 1

        if sum(counts.values()) != target_total:
            raise SystemExit(f"REFUSE: {name} mechanism counts do not re-sum")
        g1c = counts["g1c"]
        thin = counts["thin_memo_success"]
        residual = counts["residual"]
        print(f"\n=== BAND {name} ===")
        print(
            f"raw_sha256={actual_sha} total_samples={len(pcs)} "
            f"target_samples={target_total}"
        )
        for mechanism in ("g1c", "thin_memo_success", "residual"):
            value = counts[mechanism]
            print(
                f"mechanism {mechanism:18} {value:6d} "
                f"target={100.0 * value / target_total:8.4f}% "
                f"all={100.0 * value / len(pcs):8.4f}%"
            )
            class_total = sum(
                classes[(mechanism, kind)]
                for kind in ("memory", "control", "call", "other")
            )
            if class_total != value:
                raise SystemExit(f"REFUSE: {name}/{mechanism} classes do not re-sum")
            print(
                "  classes "
                + " ".join(
                    f"{kind}={classes[(mechanism, kind)]}"
                    for kind in ("memory", "control", "call", "other")
                )
            )
        print(f"overlap g1c_and_thin_memo_success=0")
        print(
            f"resum {g1c}+{thin}+{residual}={g1c + thin + residual} "
            f"target={target_total} PASS"
        )
        residual_memory = classes[("residual", "memory")]
        if sum(residual_memory_bases.values()) != residual_memory:
            raise SystemExit(f"REFUSE: {name} residual memory bases do not re-sum")
        print(
            "residual_memory_bases "
            + " ".join(
                f"{base}={residual_memory_bases[base]}"
                for base in ("stack_frame", "parser_direct", "indirect")
            )
            + f" total={residual_memory} PASS vector_subset={residual_vector_memory}"
        )
        for (mechanism, site), value in sorted(sites.items()):
            print(f"site {mechanism:18} {site:24} {value:6d}")

    print("\nMECHANISM ATTRIBUTION: PASS (exact ranges; zero overlap; full target re-sum)")
    print(
        "SCOPE: lower-bound in-target instruction samples only; residual and "
        "out-of-target samples remain unpriced; no nanoseconds derived"
    )


if __name__ == "__main__":
    main()
