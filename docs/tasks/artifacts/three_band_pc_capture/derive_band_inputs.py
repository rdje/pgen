#!/usr/bin/env python3
"""Byte-derive the PGEN-RGX-0078-0182 PCRE2 timing-band inputs."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
CANONICAL = (
    REPO_ROOT
    / "regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl"
)
CANDIDATE_TIMES = (
    REPO_ROOT / "docs/tasks/artifacts/k4b_delta/c1_ab/corpus_cand.jsonl"
)
REFERENCE_DIR = REPO_ROOT / "docs/tasks/artifacts/geomean_reprofile"
EXPECTED_SHA256 = {
    CANONICAL: "4b990bf54585d0171f318c72d7fc1e322cde97be2af723c2449b11b551a79492",
    CANDIDATE_TIMES: "439eadfcccef175d2dba226218430513781de926f0f188ecc964002c9cab23d2",
}
BANDS = (
    ("sub1us", 0, 1_000, 939),
    ("1to2p5", 1_000, 2_500, 751),
    ("2p5to20", 2_500, 20_000, 488),
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def jsonl_rows(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def unique_by_id(rows: list[dict[str, object]], label: str) -> dict[str, dict[str, object]]:
    result: dict[str, dict[str, object]] = {}
    for row in rows:
        case_id = row.get("id")
        if not isinstance(case_id, str) or case_id in result:
            raise SystemExit(f"REFUSE: invalid/duplicate {label} id {case_id!r}")
        result[case_id] = row
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()

    for path, expected in EXPECTED_SHA256.items():
        actual = sha256(path)
        if actual != expected:
            raise SystemExit(f"REFUSE: {path.relative_to(REPO_ROOT)} sha256 {actual} != {expected}")

    raw_lines = [line for line in CANONICAL.read_bytes().splitlines(keepends=True) if line.strip()]
    canonical_rows = [json.loads(line) for line in raw_lines]
    canonical = unique_by_id(canonical_rows, "canonical")
    candidate = unique_by_id(jsonl_rows(CANDIDATE_TIMES), "candidate timing")
    if set(canonical) != set(candidate):
        raise SystemExit("REFUSE: canonical and candidate timing ID sets differ")

    args.output_dir.mkdir(parents=True, exist_ok=True)
    for name, lower, upper, expected_count in BANDS:
        selected_lines = [
            line
            for line, row in zip(raw_lines, canonical_rows, strict=True)
            if lower <= int(candidate[str(row["id"])]["min_ns"]) < upper
        ]
        if len(selected_lines) != expected_count:
            raise SystemExit(
                f"REFUSE: {name} count {len(selected_lines)} != {expected_count}"
            )
        output = args.output_dir / f"band_{name}.jsonl"
        output.write_bytes(b"".join(selected_lines))

        derived_ids = [str(json.loads(line)["id"]) for line in selected_lines]
        reference = REFERENCE_DIR / f"band_{name}_times.jsonl"
        reference_ids = [str(row["id"]) for row in jsonl_rows(reference)]
        if derived_ids != reference_ids:
            raise SystemExit(f"REFUSE: {name} derived order/IDs differ from banked reference")
        print(
            f"band={name} range=[{lower},{upper}) count={len(selected_lines)} "
            f"input_sha256={sha256(output)} reference_times_sha256={sha256(reference)}"
        )


if __name__ == "__main__":
    main()
