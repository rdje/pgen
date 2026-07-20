#!/usr/bin/env python3
"""Reprice residual BATCH-1 from custody-pinned, banked real-path profiles."""

from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
PROFILE_DIR = ARTIFACT_DIR.parent / "geomean_reprofile"
FLOOR_NS = 1263.4
NOISE_NS = 28.8
G1C_NS = 26.7
MEMO_NS = 30.7
CAPTURE_FRACTIONS = (0.30, 0.50, 0.70)
DROP_SYMBOL_PREFIX = "core::ptr::drop_in_place"


@dataclass(frozen=True)
class Profile:
    filename: str
    expected_sha256: str
    expected_total_samples: int
    expected_drop_samples: int
    log_weight: float


PROFILES = (
    Profile(
        "sample_band_sub1us.txt",
        "d16e7def6f05da33ac05a664b9588b8dc29648e9ec37828f3387a95f436573ed",
        11875,
        43,
        0.380,
    ),
    Profile(
        "sample_band_1to2p5.txt",
        "e94a57152fe1e85cd1a66d8ca14b5707536ae36e20583c2712bcef64932098e6",
        11805,
        36,
        0.353,
    ),
    Profile(
        "sample_band_2p5to20.txt",
        "1c9643db9c342f315179ff49232575423f949f724ed22dadc39bdc218b7ea788",
        10673,
        43,
        0.260,
    ),
)


def read_profile(profile: Profile) -> tuple[int, int]:
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

    drop_counts = []
    for line in text.splitlines():
        stripped = line.lstrip()
        if not stripped.startswith(DROP_SYMBOL_PREFIX):
            continue
        if "ParseError" not in stripped or "(in regex_perf_probe)" not in stripped:
            continue
        match = re.search(r"\s(\d+)\s*$", stripped)
        if match:
            drop_counts.append(int(match.group(1)))
    if len(drop_counts) != 1:
        raise SystemExit(
            f"REFUSE: expected one flat ParseError drop row in {profile.filename}, "
            f"found {len(drop_counts)}"
        )
    drop_samples = drop_counts[0]

    if total_samples != profile.expected_total_samples:
        raise SystemExit(
            f"REFUSE: {profile.filename} total {total_samples} != "
            f"{profile.expected_total_samples}"
        )
    if drop_samples != profile.expected_drop_samples:
        raise SystemExit(
            f"REFUSE: {profile.filename} drop count {drop_samples} != "
            f"{profile.expected_drop_samples}"
        )
    return total_samples, drop_samples


def main() -> None:
    observed = [(profile, *read_profile(profile)) for profile in PROFILES]
    weight_sum = sum(profile.log_weight for profile, _, _ in observed)
    weighted_share = sum(
        profile.log_weight * drop_samples / total_samples
        for profile, total_samples, drop_samples in observed
    )
    normalized_share = weighted_share / weight_sum
    conservative_drop_ns = weighted_share * FLOOR_NS
    covered_drop_ns = normalized_share * FLOOR_NS
    residual_ns = G1C_NS + MEMO_NS + conservative_drop_ns

    print("custody: PASS (3/3 full sha256, sample totals, and flat rows)")
    print("band                         total  drop  self_share  log_weight")
    for profile, total_samples, drop_samples in observed:
        print(
            f"{profile.filename:28} {total_samples:5d}  {drop_samples:4d}  "
            f"{100.0 * drop_samples / total_samples:9.4f}%  "
            f"{100.0 * profile.log_weight:8.1f}%"
        )
    print()
    print(f"covered log weight:                 {100.0 * weight_sum:.1f}%")
    print(f"drop share, covered-band normalized:{100.0 * normalized_share:9.4f}%")
    print(f"drop target, covered-band estimate: {covered_drop_ns:9.3f} ns")
    print(
        "drop share, full-corpus conservative:"
        f"{100.0 * weighted_share:7.4f}% (excluded tail priced at zero)"
    )
    print(f"drop target used in bundle:         {conservative_drop_ns:9.3f} ns")
    print(f"G1-C attributed target:             {G1C_NS:9.3f} ns")
    print(f"memo segment-copy target:           {MEMO_NS:9.3f} ns")
    print(f"residual BATCH-1 target:             {residual_ns:9.3f} ns")
    print(f"residual target / floor:            {100.0 * residual_ns / FLOOR_NS:9.4f}%")
    print()
    print("capture  saving_ns  geomean_delta  noise_multiple")
    for capture in CAPTURE_FRACTIONS:
        saving_ns = capture * residual_ns
        print(
            f"{100.0 * capture:6.0f}%  {saving_ns:9.3f}  "
            f"{-100.0 * saving_ns / FLOOR_NS:12.4f}%  "
            f"{saving_ns / NOISE_NS:13.4f}x"
        )
    print()
    midpoint_margin_ns = 0.50 * residual_ns - NOISE_NS
    print(f"midpoint margin above noise:        {midpoint_margin_ns:9.3f} ns")
    print("decision: HOLD (midpoint has no practical margin; price G3 next)")


if __name__ == "__main__":
    main()
