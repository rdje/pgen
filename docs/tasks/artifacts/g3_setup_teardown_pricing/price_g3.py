#!/usr/bin/env python3
"""Custody-check and price the in-metric G3 virgin-reset mechanism."""

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
FLOOR_NS = 1263.4
NOISE_NS = 28.8
G1C_NS = 26.7
MEMO_NS = 30.7
CAPTURE_FRACTIONS = (0.30, 0.50, 0.70)

# Return-address offsets inside parse_once_timed, proven against the preserved
# probe below. These four calls form the directly sampled lower bound of the
# first parse's redundant prepare_parse_state reset.
G3_OFFSETS = (3184, 3192, 3200, 3420)
# Both normal-return destruction calls occur after Instant::elapsed() reads the
# ending clock. They are profiled CPU work but cannot change the stored metric.
OUT_OF_METRIC_TEARDOWN_OFFSETS = (8764, 8792)


@dataclass(frozen=True)
class Profile:
    filename: str
    expected_sha256: str
    expected_total_samples: int
    log_weight: float
    expected_drop_samples: int
    expected_g3_by_offset: tuple[int, int, int, int]
    expected_teardown_by_offset: tuple[int, int]


PROFILES = (
    Profile(
        "sample_band_sub1us.txt",
        "d16e7def6f05da33ac05a664b9588b8dc29648e9ec37828f3387a95f436573ed",
        11875,
        0.380,
        43,
        (14, 199, 148, 17),
        (844, 217),
    ),
    Profile(
        "sample_band_1to2p5.txt",
        "e94a57152fe1e85cd1a66d8ca14b5707536ae36e20583c2712bcef64932098e6",
        11805,
        0.353,
        36,
        (3, 80, 64, 5),
        (686, 296),
    ),
    Profile(
        "sample_band_2p5to20.txt",
        "1c9643db9c342f315179ff49232575423f949f724ed22dadc39bdc218b7ea788",
        10673,
        0.260,
        43,
        (3, 28, 18, 1),
        (615, 284),
    ),
)


def require_binary_custody() -> None:
    actual_sha256 = hashlib.sha256(PROBE.read_bytes()).hexdigest()
    if actual_sha256 != PROBE_SHA256:
        raise SystemExit(
            f"REFUSE: preserved probe sha256 {actual_sha256} != {PROBE_SHA256}"
        )

    for tool in ("nm", "otool"):
        if shutil.which(tool) is None:
            raise SystemExit(f"REFUSE: required macOS binary tool not found: {tool}")
    nm_text = subprocess.run(
        ["nm", "-nm", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    if (
        "000000010000179c (__TEXT,__text) non-external "
        "__ZN16regex_perf_probe16parse_once_timed17h297949940cc4fc4dE"
        not in nm_text
    ):
        raise SystemExit("REFUSE: parse_once_timed symbol/start does not match custody pin")

    disasm = subprocess.run(
        ["otool", "-tV", str(PROBE)], check=True, capture_output=True, text=True
    ).stdout
    required = (
        ("0000000100002408", "to_vec"),
        ("0000000100002410", "SemanticRuntimeState3new"),
        ("0000000100002418", "drop_in_place", "SemanticRuntimeState"),
        ("00000001000024f4", "clone_predicate_defs"),
        ("000000010000399c", "Timespec3now"),
        ("00000001000039d4", "drop_in_place", "RegexParser"),
        ("00000001000039f0", "drop_in_place", "NodeArena"),
    )
    disasm_by_address = {line.split("\t", 1)[0]: line for line in disasm.splitlines()}
    for address_and_terms in required:
        address, *terms = address_and_terms
        line = disasm_by_address.get(address, "")
        if not line or not all(term in line for term in terms):
            raise SystemExit(
                f"REFUSE: expected disassembly terms {terms} at {address}, got {line!r}"
            )


def parse_profile(profile: Profile) -> tuple[int, int, int, int]:
    path = PROFILE_DIR / profile.filename
    raw = path.read_bytes()
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    if actual_sha256 != profile.expected_sha256:
        raise SystemExit(
            f"REFUSE: {profile.filename} sha256 {actual_sha256} != "
            f"{profile.expected_sha256}"
        )
    text = raw.decode("utf-8")

    total_matches = re.findall(
        r"^\s*(\d+) Thread_\d+\s+DispatchQueue_1:", text, flags=re.MULTILINE
    )
    if len(total_matches) != 1:
        raise SystemExit(
            f"REFUSE: expected one main-thread total in {profile.filename}, "
            f"found {len(total_matches)}"
        )
    total_samples = int(total_matches[0])
    if total_samples != profile.expected_total_samples:
        raise SystemExit(
            f"REFUSE: {profile.filename} total {total_samples} != "
            f"{profile.expected_total_samples}"
        )

    offset_counts: dict[int, int] = {}
    line_pattern = re.compile(
        r"^\s+\+.*?\s(\d+) regex_perf_probe::parse_once_timed.*?"
        r"\(in regex_perf_probe\) \+ (\d+)(?:,|\s)",
        flags=re.MULTILINE,
    )
    for count_text, offset_text in line_pattern.findall(text):
        offset = int(offset_text)
        offset_counts[offset] = offset_counts.get(offset, 0) + int(count_text)

    actual_g3 = tuple(offset_counts.get(offset, 0) for offset in G3_OFFSETS)
    if actual_g3 != profile.expected_g3_by_offset:
        raise SystemExit(
            f"REFUSE: {profile.filename} G3 offset counts {actual_g3} != "
            f"{profile.expected_g3_by_offset}"
        )
    actual_teardown = tuple(
        offset_counts.get(offset, 0) for offset in OUT_OF_METRIC_TEARDOWN_OFFSETS
    )
    if actual_teardown != profile.expected_teardown_by_offset:
        raise SystemExit(
            f"REFUSE: {profile.filename} teardown offset counts {actual_teardown} != "
            f"{profile.expected_teardown_by_offset}"
        )

    drop_rows = []
    for line in text.splitlines():
        stripped = line.lstrip()
        if not stripped.startswith("core::ptr::drop_in_place"):
            continue
        if "ParseError" not in stripped or "(in regex_perf_probe)" not in stripped:
            continue
        match = re.search(r"\s(\d+)\s*$", stripped)
        if match:
            drop_rows.append(int(match.group(1)))
    if drop_rows != [profile.expected_drop_samples]:
        raise SystemExit(
            f"REFUSE: {profile.filename} ParseError drop rows {drop_rows} != "
            f"[{profile.expected_drop_samples}]"
        )

    return (
        total_samples,
        sum(actual_g3),
        sum(actual_teardown),
        profile.expected_drop_samples,
    )


def main() -> None:
    require_binary_custody()
    observed = [(profile, *parse_profile(profile)) for profile in PROFILES]
    weight_sum = sum(profile.log_weight for profile, *_ in observed)

    g3_weighted_share = sum(
        profile.log_weight * g3_samples / total_samples
        for profile, total_samples, g3_samples, _, _ in observed
    )
    teardown_weighted_share = sum(
        profile.log_weight * teardown_samples / total_samples
        for profile, total_samples, _, teardown_samples, _ in observed
    )
    drop_weighted_share = sum(
        profile.log_weight * drop_samples / total_samples
        for profile, total_samples, _, _, drop_samples in observed
    )

    g3_ns = g3_weighted_share * FLOOR_NS
    teardown_false_ns = teardown_weighted_share * FLOOR_NS
    residual_ns = G1C_NS + MEMO_NS + drop_weighted_share * FLOOR_NS
    enlarged_ns = residual_ns + g3_ns

    print("custody: PASS (probe disassembly + 3/3 profile hashes/totals/offsets)")
    print("timer boundary: elapsed @ +8704; parser drop @ +8760; arena drop @ +8788")
    print("G3 return offsets: to_vec=3184 new=3192 old-drop=3200 clone=3420")
    print()
    print("band                         total  G3_samples  G3_share  teardown_out  teardown_share")
    for profile, total_samples, g3_samples, teardown_samples, _ in observed:
        print(
            f"{profile.filename:28} {total_samples:5d}  {g3_samples:10d}  "
            f"{100.0 * g3_samples / total_samples:8.4f}%  "
            f"{teardown_samples:12d}  "
            f"{100.0 * teardown_samples / total_samples:12.4f}%"
        )
    print()
    print(f"covered log weight:                     {100.0 * weight_sum:9.1f}%")
    print(
        "G3 share, covered-band normalized:     "
        f"{100.0 * g3_weighted_share / weight_sum:9.4f}%"
    )
    print(
        "G3 target, covered-band estimate:       "
        f"{g3_weighted_share / weight_sum * FLOOR_NS:9.3f} ns"
    )
    print(
        "G3 share, full-corpus conservative:     "
        f"{100.0 * g3_weighted_share:9.4f}%"
    )
    print(f"G3 target used in bundle:               {g3_ns:9.3f} ns")
    print(
        "teardown profile share excluded:        "
        f"{100.0 * teardown_weighted_share:9.4f}%"
    )
    print(f"false target if teardown were admitted:  {teardown_false_ns:9.3f} ns")
    print(f"prior residual BATCH-1 target:           {residual_ns:9.3f} ns")
    print(f"BATCH-1 + G3 target:                     {enlarged_ns:9.3f} ns")
    print(f"enlarged target / floor:                 {100.0 * enlarged_ns / FLOOR_NS:9.4f}%")
    print()
    print("capture  saving_ns  geomean_delta  noise_multiple")
    for capture in CAPTURE_FRACTIONS:
        saving_ns = capture * enlarged_ns
        print(
            f"{100.0 * capture:6.0f}%  {saving_ns:9.3f}  "
            f"{-100.0 * saving_ns / FLOOR_NS:12.4f}%  "
            f"{saving_ns / NOISE_NS:13.4f}x"
        )
    print()
    print(f"low-case shortfall below noise:          {NOISE_NS - 0.30 * enlarged_ns:9.3f} ns")
    print(f"midpoint margin above noise:             {0.50 * enlarged_ns - NOISE_NS:9.3f} ns")
    print("decision: HOLD (low capture still misses noise; price fused-spine memory next)")


if __name__ == "__main__":
    main()
