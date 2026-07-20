#!/usr/bin/env python3
"""PGEN-RGX-0078-0206 — role partition + re-pricing on the LANDED thin-memo
direct-index representation (`dindex_d21fab44`).

Method (the `-0186`/`-0204` offset discipline re-derived from scratch for the
new binary, per the `-0171` standing rule):

- Tier 1 (exact): every in-target parser-direct memory sample ([SELF_REG,
  #offset]) is partitioned by a role map whose EVERY offset is qualified by a
  machine/dataflow fingerprint in THIS probe's own disassembly.  Unknown
  offsets REFUSE.
- Tier 2 (derived, window-qualified): an in-target memory sample whose base
  register was defined by a parser-direct load of a qualified offset within
  the preceding 8 instructions (no intervening redefinition) is credited to
  `derived:<role>`.
- Out-of-target in-text samples are bucketed by containing symbol into named
  populations — the `-0171` caller-attribution view of the out-of-target mass
  on the new floor.  These are POPULATION shares, not per-site lever prices.
- ns conversion: weighted share over ALL stored samples per band
  (weights = the band log-shares of the corpus geomean, band_manifest.json)
  x the floor geomean 1038.361702538721 ns — byte-for-byte the `-0186`
  convention.

The landed `-0205` program is asserted structurally: the old thin-memo
FxHashMap RawTable offsets/fingerprints are GONE (no NEON ctrl-group probe in
the thin-memo windows); the direct-index carrier appears instead as the
generation-stamped row table (slot load + `lsr #32` generation compare + the
sorted per-rule row-stride computation) and the dense 96-byte `thin_entries`
vec.

Custody: the probe SHA, band manifest SHA, and the three raw capture SHAs
(raw_custody.json, banked at capture time) are asserted before any number is
emitted.
"""

from __future__ import annotations

import json
import re
import sys
from collections import Counter
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
sys.path[:0] = [str(ARTIFACT_DIR)]

import analyze_capture as capture  # noqa: E402


FLOOR_GEOMEAN_NS = 1038.361702538721
OLD_FLOOR_NS = 1263.4
OLD_NOISE_NS = 28.8
NOISE_NS = OLD_NOISE_NS / OLD_FLOOR_NS * FLOOR_GEOMEAN_NS

SELF_REG = {"piece": "x19", "atom_closure": "x20"}

GROUP_OFFSETS = {
    # Two 16-byte-element stacks + the depth-ceiling compare (0x60); element
    # addressing is `lsl #4`, growth via mi_malloc_aligned / grow_one.
    "recursion_guard": frozenset({0x8, 0x10, 0x18, 0x20, 0x28, 0x60}),
    # Seven checkpoint sources co-loaded in one capture window and spilled as
    # the SemanticRuntimeCheckpoint (unchanged offsets from the `-0204`
    # vintage; 0x258 doubles as the unchanged-compare word and the thin-entry
    # epoch-validation source — still one checkpoint source).
    "semantic_checkpoint_state": frozenset(
        {0xF0, 0x108, 0x120, 0x150, 0x178, 0x198, 0x258}
    ),
    # Memo soundness signal: snapshot load before a rule body, compare +
    # cset after — a changed predicate-evaluation count taints the store-blind
    # thin-memo key.
    "memo_taint_signal": frozenset({0x250}),
    # THE landed -0205 direct-index probe machinery: generation-stamped row
    # table (data 0x280 / len 0x288, 8-byte rows via `lsl #3`, `lsr #32`
    # generation extract vs the current-generation word 0x290) + the
    # per-parse (len+1) row stride 0x538 multiplied by the sorted
    # compile-time THIN_ROW_* consts (madd / add-lsl immediates).
    "thin_memo_rows": frozenset({0x280, 0x288, 0x290, 0x538}),
    # The landed -0205 dense entry vec: data 0x2A0 / len 0x2A8, 96-byte
    # (#0x60) elements addressed via umaddl, epoch-validated against 0x258.
    "thin_memo_entries": frozenset({0x2A0, 0x2A8}),
    # THE landed -0203 carrier: ONE Vec<TapeWord> = cap/data/len, 8-byte
    # elements (`lsl #3` stores), RawVec::grow_one on the cap path.
    "unified_tape": frozenset({0x2B0, 0x2B8, 0x2C0}),
    # &str input view: data pointer (0x360) feeding `ldrb [data, pos]` byte
    # loads, length (0x368) feeding bounds compares.
    "input_view": frozenset({0x360, 0x368}),
    # &NodeArena pointer: typed-arena borrow flag (ldr [x0]; cbnz; str #-1)
    # + inline-capacity check ([x0,#0x18] vs [x0,#0x8]) — the G1-C
    # node/arena residue lane.
    "arena_node_g1c": frozenset({0x370}),
    # Current position (0x4E8) + the monotone furthest-position mirror
    # (0x530: load/cmp/b.ls/store).
    "position_progress": frozenset({0x4E8, 0x530}),
}

BUCKET_RULES: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("allocator", ("_mi_", "mi_malloc", "mi_free", "_malloc", "_free", "_realloc")),
    ("memops", ("_memcpy", "_memmove", "_memset", "_bzero", "_platform_mem")),
    ("teardown_drop", ("drop_in_place",)),
    ("hashbrown_map", ("hashbrown", "RawTable", "HashMap")),
    ("thin_entry_push", ("push_mut17h2d2239d8f4ba2a41",)),
    (
        "vec_growth",
        ("finish_grow", "grow_one", "append_elements", "RawVec", "reserve",
         "push_mut"),
    ),
    ("arena_alloc", ("alloc_extend", "NodeArena", "typed_arena", "alloc_slow")),
    (
        "build_value",
        ("to_shaped_value", "cascade_build", "shaped", "build_committed",
         "UnifiedAstValue", "to_serde"),
    ),
    (
        "semantic_runtime",
        ("semantic_runtime", "SemanticRuntimeState", "FactIndex",
         "apply_semantic", "rule_context_path", "with_semantic"),
    ),
    ("tape_helpers", ("TapeWord", "deriv_next", "DerivEvent")),
    (
        "parse_setup",
        ("prepare_parse_state", "new_parser", "reset_for_parse",
         "CompiledSemanticRuntimeAnnotations",),
    ),
    ("spine_other", ("RegexParser",)),
    ("harness", ("parse_once_timed", "regex_perf_probe", "4main", "Timespec")),
)


def bucket_for(symbol: str) -> str:
    for bucket, needles in BUCKET_RULES:
        if any(needle in symbol for needle in needles):
            return bucket
    return "other_text"


def load_raw_custody() -> dict[str, str]:
    path = ARTIFACT_DIR / "raw_custody.json"
    if not path.exists():
        raise SystemExit("REFUSE: raw_custody.json missing — bank the capture SHAs first")
    return json.loads(path.read_text())


BASE_REG = re.compile(r"\[(x\d+|sp|x29)(?:, #-?(?:0x[0-9a-f]+|\d+))?\]")
SELF_OFF = {
    label: re.compile(r"\[" + reg + r"(?:, #(?P<offset>0x[0-9a-f]+|\d+))?\]")
    for label, reg in SELF_REG.items()
}


def parser_offset(label: str, operands: str) -> int | None:
    matches = list(SELF_OFF[label].finditer(operands))
    if not matches:
        return None
    if len(matches) != 1:
        raise SystemExit(f"REFUSE: multiple self-reg operands: {operands}")
    raw = matches[0].group("offset")
    if raw is None:
        return 0
    return int(raw, 16) if raw.startswith("0x") else int(raw)


def is_memory(mnemonic: str) -> bool:
    return capture.instruction_class(mnemonic) == "memory"


def group_for_offset(offset: int) -> str:
    matches = [group for group, offsets in GROUP_OFFSETS.items() if offset in offsets]
    if len(matches) != 1:
        raise SystemExit(f"REFUSE: parser offset {offset:#x} has groups {matches}")
    return matches[0]


def window_text(
    code: dict[int, tuple[str, str]], address: int, before: int, after: int
) -> str:
    return "\n".join(
        f"{code[pc][0]} {code[pc][1]}"
        for pc in range(address - before * 4, address + after * 4 + 4, 4)
        if pc in code
    )


def static_offset_sites(
    code: dict[int, tuple[str, str]], ranges: dict[str, tuple[int, int]]
) -> dict[int, list[tuple[str, int]]]:
    sites: dict[int, list[tuple[str, int]]] = {}
    for label, (start, end) in ranges.items():
        for pc in range(start, end, 4):
            instruction = code.get(pc)
            if instruction is None or not is_memory(instruction[0]):
                continue
            offset = parser_offset(label, instruction[1])
            if offset is not None:
                sites.setdefault(offset, []).append((label, pc))
    return sites


def qualify_machine_roles(
    code: dict[int, tuple[str, str]], sites: dict[int, list[tuple[str, int]]]
) -> None:
    """Refuse unless each offset group carries its exclusive binary fingerprint."""

    known = set().union(*GROUP_OFFSETS.values())
    unknown = {offset for offset in sites if offset not in known}
    if unknown:
        raise SystemExit(f"REFUSE: unqualified parser offsets {sorted(map(hex, unknown))}")

    # Input view: data-pointer loads feed byte loads; length loads feed bounds
    # compares.
    input_data = "\n".join(window_text(code, pc, 0, 12) for _, pc in sites.get(0x360, []))
    if input_data.count("ldrb") < len(sites.get(0x360, ())) // 2:
        raise SystemExit("REFUSE: input data-pointer byte-load fingerprint differs")
    input_len = "\n".join(window_text(code, pc, 0, 8) for _, pc in sites.get(0x368, []))
    if input_len.count("cmp") < len(sites.get(0x368, ())) // 2:
        raise SystemExit("REFUSE: input length bounds-compare fingerprint differs")

    # Arena pointer: typed-arena borrow-flag + inline capacity check.
    arena_text = "\n".join(window_text(code, pc, 0, 14) for _, pc in sites.get(0x370, []))
    if "cbnz" not in arena_text or "#-0x1" not in arena_text or "[x0, #0x18]" not in arena_text:
        raise SystemExit("REFUSE: arena borrow/capacity fingerprint differs")

    # Unified tape: one Vec, 8-byte elements, grow_one on the cap path.
    tape_text = "\n".join(
        window_text(code, pc, 8, 12)
        for offset in GROUP_OFFSETS["unified_tape"]
        for _, pc in sites.get(offset, [])
    )
    if "grow_one" not in tape_text or "lsl #3" not in tape_text:
        raise SystemExit("REFUSE: unified-tape Vec fingerprint differs")
    for stale in ("deriv_events", "deriv_boundary"):
        if stale in tape_text:
            raise SystemExit(f"REFUSE: retired lane symbol {stale} present")

    # Thin-memo direct-index rows: 8-byte slot load + `lsr #32` generation
    # extract compared (w-register) against the current-generation word; the
    # hashbrown NEON ctrl-group probe must be GONE from these windows.
    row_text = "\n".join(
        window_text(code, pc, 4, 12)
        for offset in (0x280, 0x288, 0x290)
        for _, pc in sites.get(offset, [])
    )
    if "lsr" not in row_text or "lsl #3" not in row_text:
        raise SystemExit("REFUSE: row-table slot-load/generation fingerprint differs")
    if row_text.count("cmp w") < len(sites.get(0x290, ())) // 2:
        raise SystemExit("REFUSE: generation-compare fingerprint differs")
    if "cmeq.8b" in row_text or "madd" in row_text and "cmeq" in row_text:
        raise SystemExit("REFUSE: retired hashbrown group-probe fingerprint present")
    stride_text = "\n".join(
        window_text(code, pc, 2, 8) for _, pc in sites.get(0x538, [])
    )
    if "madd" not in stride_text and "lsl #1" not in stride_text:
        raise SystemExit("REFUSE: row-stride THIN_ROW_* multiply fingerprint differs")

    # Thin-memo dense entries: 96-byte (#0x60) element addressing.
    entry_text = "\n".join(
        window_text(code, pc, 2, 10)
        for offset in GROUP_OFFSETS["thin_memo_entries"]
        for _, pc in sites.get(offset, [])
    )
    if "#0x60" not in entry_text:
        raise SystemExit("REFUSE: thin-entry 96-byte stride fingerprint differs")

    # Position progress: monotone furthest mirror.
    furthest_text = "\n".join(window_text(code, pc, 4, 6) for _, pc in sites.get(0x530, []))
    if furthest_text.count("cmp") < len(sites.get(0x530, ())) // 2:
        raise SystemExit("REFUSE: furthest-position monotone fingerprint differs")

    # Checkpoint: the seven sources co-loaded in one window at multiple sites.
    checkpoint_sources = GROUP_OFFSETS["semantic_checkpoint_state"]
    co_captured = 0
    for label, pc in sites.get(0xF0, []):
        loaded = {
            parser_offset(label, code[address][1])
            for address in range(pc - 16, pc + 64, 4)
            if address in code and is_memory(code[address][0])
            and parser_offset(label, code[address][1]) is not None
        }
        if checkpoint_sources <= loaded:
            co_captured += 1
    if co_captured < max(1, len(sites.get(0xF0, ())) // 2):
        raise SystemExit(
            f"REFUSE: checkpoint co-capture fingerprint differs ({co_captured} sites)"
        )

    # Memo taint: snapshot load + compare/cset pair.
    taint_text = "\n".join(window_text(code, pc, 4, 10) for _, pc in sites.get(0x250, []))
    if "cset" not in taint_text and "cmp" not in taint_text:
        raise SystemExit("REFUSE: memo-taint snapshot/compare fingerprint differs")

    # Recursion guard: 16-byte element stacks + depth ceiling.
    guard_text = "\n".join(
        window_text(code, pc, 20, 24)
        for offset in GROUP_OFFSETS["recursion_guard"]
        for _, pc in sites.get(offset, [])
    )
    if "lsl #4" not in guard_text:
        raise SystemExit("REFUSE: recursion-guard element fingerprint differs")

    print(
        "machine qualification: PASS "
        "input=bounds+byte-load arena=borrow+capacity tape=one-vec-8B "
        "thin_memo_rows=slot-load+gen-compare+row-stride(no-NEON-probe) "
        "thin_memo_entries=96B-stride position=monotone "
        f"checkpoint=7-sources-co-captured({co_captured} sites) taint=snapshot/compare "
        "guard=16B-stack+ceiling; no unknown offsets"
    )


DEST_FORBIDDEN = ("str", "stp", "stur", "cmp", "cmn", "tst", "ccmp", "b", "ret")


def defines(instruction: tuple[str, str], register: str) -> bool:
    mnemonic, operands = instruction
    if mnemonic.startswith(DEST_FORBIDDEN):
        return False
    first = operands.split(",")[0].strip() if operands else ""
    if first in (register, register.replace("x", "w")):
        return True
    # ldp defines its first two operands.
    if mnemonic.startswith(("ldp",)):
        parts = [part.strip() for part in operands.split(",")[:2]]
        return register in parts or register.replace("x", "w") in parts
    return False


def derived_role(
    code: dict[int, tuple[str, str]],
    label: str,
    address: int,
) -> str | None:
    instruction = code[address]
    if not is_memory(instruction[0]):
        return None
    if parser_offset(label, instruction[1]) is not None:
        return None
    match = BASE_REG.search(instruction[1])
    if match is None:
        return None
    base = match.group(1)
    if base in ("sp", "x29", SELF_REG[label]):
        return None
    for step in range(1, 9):
        prior_address = address - step * 4
        prior = code.get(prior_address)
        if prior is None:
            return None
        if prior[0] == "ldr":
            offset = parser_offset(label, prior[1])
            destination = prior[1].split(",")[0].strip()
            if offset is not None and destination == base:
                if offset in set().union(*GROUP_OFFSETS.values()):
                    return group_for_offset(offset)
                return None
        if defines(prior, base):
            return None
    return None


def main() -> None:
    if capture.sha256(capture.PROBE) != capture.PROBE_SHA256:
        raise SystemExit("REFUSE: preserved probe SHA-256 mismatch")
    raw_custody = load_raw_custody()
    manifest = capture.load_manifest()
    symbols = capture.all_symbols()
    ranges = capture.symbol_ranges(symbols)
    code = capture.disassembly()

    sites = static_offset_sites(code, ranges)
    qualify_machine_roles(code, sites)

    weights = {
        name: manifest["bands"][name]["log_share"] for name in capture.BAND_NAMES
    }
    print(
        "weights (band log-shares of the corpus geomean): "
        + " ".join(f"{name}={weights[name]:.6f}" for name in capture.BAND_NAMES)
        + f" | floor={FLOOR_GEOMEAN_NS!r} ns | noise={NOISE_NS:.3f} ns "
        f"(the banked {OLD_NOISE_NS} ns span carried over: "
        f"{OLD_NOISE_NS}/{OLD_FLOOR_NS} x floor)"
    )

    symbol_addresses = [address for address, _ in symbols]
    symbol_names = [name for _, name in symbols]

    def symbol_at(address: int) -> str:
        import bisect

        index = bisect.bisect_right(symbol_addresses, address) - 1
        return symbol_names[index] if index >= 0 else "<pre-first-symbol>"

    role_counts: dict[str, Counter[str]] = {}
    derived_counts: dict[str, Counter[str]] = {}
    bucket_counts: dict[str, Counter[str]] = {}
    bucket_symbols: dict[str, Counter[tuple[str, str]]] = {}
    unattributed: dict[str, Counter[str]] = {}
    raw_totals: dict[str, int] = {}
    target_totals: dict[str, int] = {}

    for name in capture.BAND_NAMES:
        raw_path = ARTIFACT_DIR / f"capture_band_{name}_raw_pcs.txt"
        if capture.sha256(raw_path) != raw_custody[name]:
            raise SystemExit(f"REFUSE: {name} raw SHA-256 mismatch vs banked custody")
        header, pcs = capture.parse_raw(raw_path)
        slide = int(header["main_slide"], 16)
        roles: Counter[str] = Counter()
        derived: Counter[str] = Counter()
        buckets: Counter[str] = Counter()
        per_symbol: Counter[tuple[str, str]] = Counter()
        classes: Counter[str] = Counter()
        target_count = 0
        for sampled_pc in pcs:
            address = sampled_pc - slide
            if not (capture.TEXT_START <= address < capture.TEXT_END):
                buckets["external_or_injected"] += 1
                continue
            target = next(
                (
                    label
                    for label, (start, end) in ranges.items()
                    if start <= address < end
                ),
                None,
            )
            if target is None:
                symbol = symbol_at(address)
                bucket = bucket_for(symbol)
                buckets[bucket] += 1
                per_symbol[(bucket, symbol)] += 1
                continue
            target_count += 1
            instruction = code[address]
            offset = (
                parser_offset(target, instruction[1])
                if is_memory(instruction[0])
                else None
            )
            if offset is not None:
                roles[group_for_offset(offset)] += 1
                continue
            role = derived_role(code, target, address)
            if role is not None:
                derived[role] += 1
                continue
            classes[capture.instruction_class(instruction[0])] += 1
        role_counts[name] = roles
        derived_counts[name] = derived
        bucket_counts[name] = buckets
        bucket_symbols[name] = per_symbol
        unattributed[name] = classes
        raw_totals[name] = len(pcs)
        target_totals[name] = target_count
        in_target_sum = (
            sum(roles.values()) + sum(derived.values()) + sum(classes.values())
        )
        if in_target_sum != target_count:
            raise SystemExit(f"REFUSE: {name} in-target partition does not re-sum")
        print(
            f"\nband {name}: stored={len(pcs)} target={target_count} "
            f"tier1={sum(roles.values())} tier2={sum(derived.values())} "
            f"unattributed={sum(classes.values())} {dict(classes)}"
        )

    def weighted_ns(counts_by_band: dict[str, int]) -> tuple[float, float]:
        share = sum(
            weights[name] * counts_by_band.get(name, 0) / raw_totals[name]
            for name in capture.BAND_NAMES
        )
        return share, share * FLOOR_GEOMEAN_NS

    print("\n=== TIER-1 parser-direct role prices (exact offsets) ===")
    print(
        f"{'role':28} {'sub1us':>7} {'1to2p5':>7} {'2p5to20':>8} "
        f"{'w-share%':>9} {'ns':>10} {'30%':>8} {'50%':>8} {'70%':>8}"
    )
    tier_rows: dict[str, tuple[float, float]] = {}
    for role in GROUP_OFFSETS:
        counts = {name: role_counts[name][role] for name in capture.BAND_NAMES}
        share, nanoseconds = weighted_ns(counts)
        tier_rows[role] = (share, nanoseconds)
        print(
            f"{role:28} {counts['sub1us']:7d} {counts['1to2p5']:7d} "
            f"{counts['2p5to20']:8d} {100 * share:9.4f} {nanoseconds:10.4f} "
            f"{0.3 * nanoseconds:8.3f} {0.5 * nanoseconds:8.3f} {0.7 * nanoseconds:8.3f}"
        )

    print("\n=== TIER-1+2 role prices (direct + window-qualified derived) ===")
    combined_rows: dict[str, tuple[float, float]] = {}
    for role in GROUP_OFFSETS:
        counts = {
            name: role_counts[name][role] + derived_counts[name][role]
            for name in capture.BAND_NAMES
        }
        share, nanoseconds = weighted_ns(counts)
        combined_rows[role] = (share, nanoseconds)
        print(
            f"{role:28} {counts['sub1us']:7d} {counts['1to2p5']:7d} "
            f"{counts['2p5to20']:8d} {100 * share:9.4f} {nanoseconds:10.4f} "
            f"{0.3 * nanoseconds:8.3f} {0.5 * nanoseconds:8.3f} {0.7 * nanoseconds:8.3f}"
        )

    print("\n=== Out-of-target population buckets (per-band share of stored) ===")
    bucket_names = sorted(
        {bucket for counts in bucket_counts.values() for bucket in counts}
    )
    for bucket in bucket_names:
        counts = {name: bucket_counts[name][bucket] for name in capture.BAND_NAMES}
        share, nanoseconds = weighted_ns(counts)
        print(
            f"{bucket:22} {counts['sub1us']:7d} {counts['1to2p5']:7d} "
            f"{counts['2p5to20']:8d} {100 * share:9.4f}% {nanoseconds:10.4f} ns"
        )

    print("\n=== Out-of-target symbols with >=20 samples in any band ===")
    all_symbol_keys = sorted(
        {
            key
            for counts in bucket_symbols.values()
            for key, count in counts.items()
            if any(bucket_symbols[b][key] >= 20 for b in capture.BAND_NAMES)
        },
        key=lambda key: -sum(bucket_symbols[b][key] for b in capture.BAND_NAMES),
    )
    for bucket, symbol in all_symbol_keys:
        counts = [bucket_symbols[b][(bucket, symbol)] for b in capture.BAND_NAMES]
        share, nanoseconds = weighted_ns(
            dict(zip(capture.BAND_NAMES, counts, strict=True))
        )
        print(
            f"   {counts[0]:6d} {counts[1]:6d} {counts[2]:6d} {nanoseconds:9.4f}ns "
            f"[{bucket}] {symbol[:100]}"
        )

    print("\n=== Static caller census for hot out-of-target callees ===")
    callee_needles = (
        "6insert17h50f472a538f20890E",
        "6insert17hccf7033f85f6247aE",
        "FactIndex6insert",
        "push_mut17h2d2239d8f4ba2a41",
        "hash_one",
        "sip6Hasher",
        "alloc_extend",
        "grow_one",
        "finish_grow",
        "_mi_malloc",
        "to_shaped_value",
    )
    for needle in callee_needles:
        callers: Counter[str] = Counter()
        for address, instruction in code.items():
            if instruction[0] == "bl" and needle in instruction[1]:
                callers[symbol_at(address)] += 1
        print(f"-- bl-sites for *{needle}*:")
        for caller, count in callers.most_common(12):
            print(f"   {count:4d} {caller[:105]}")

    print(f"\nnoise floor on this floor: {NOISE_NS:.3f} ns; a solo lever needs its")
    print("honest capture fraction to clear that with margin; sub-noise levers")
    print("may only proceed BATCHED per the -0170 amended rule.")


if __name__ == "__main__":
    main()
