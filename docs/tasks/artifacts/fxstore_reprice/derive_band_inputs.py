#!/usr/bin/env python3
"""Byte-derive the PGEN-RGX-0078-0208 timing-band inputs.

The `-0207` landed fix's own corpus timing population
(docs/tasks/artifacts/semantic_store_fxhash/corpus_candidate.jsonl, the
floor-of-record sweep of preserved probe `fxstore_a4067793`) partitions the
canonical PCRE2 corpus into the three geomean bands.  The 11 cells >= 20 us
(0.78% log-share) are excluded by design, exactly as in
`-0162`/`-0182`/`-0204`/`-0206`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
REPO_ROOT = ARTIFACT_DIR.parents[3]
CANONICAL = (
    REPO_ROOT
    / "regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl"
)
CANDIDATE_TIMES = (
    REPO_ROOT / "docs/tasks/artifacts/semantic_store_fxhash/corpus_candidate.jsonl"
)
EXPECTED_SHA256 = {
    CANONICAL: "4b990bf54585d0171f318c72d7fc1e322cde97be2af723c2449b11b551a79492",
    CANDIDATE_TIMES: "7204a541b109530c57608660e660a9d5fc72c7ce00dec51276d664f7ce4738fc",
}
FLOOR_GEOMEAN_NS = 1037.804231341058
BANDS = (
    ("sub1us", 0, 1_000, 1073),
    ("1to2p5", 1_000, 2_500, 731),
    ("2p5to20", 2_500, 20_000, 374),
)
EXCLUDED_GE_20US = 11


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
    candidate_rows = jsonl_rows(CANDIDATE_TIMES)
    candidate = unique_by_id(candidate_rows, "candidate timing")
    if set(canonical) != set(candidate):
        raise SystemExit("REFUSE: canonical and candidate timing ID sets differ")
    if len(candidate_rows) != 2189:
        raise SystemExit(f"REFUSE: candidate population {len(candidate_rows)} != 2189")

    logs = [math.log(int(row["min_ns"])) for row in candidate_rows]
    geomean = math.exp(sum(logs) / len(logs))
    if f"{geomean!r}" != f"{FLOOR_GEOMEAN_NS!r}":
        raise SystemExit(
            f"REFUSE: timing population geomean {geomean!r} != floor {FLOOR_GEOMEAN_NS!r}"
        )
    total_log = sum(logs)

    args.output_dir.mkdir(parents=True, exist_ok=True)
    manifest: dict[str, object] = {
        "floor_geomean_ns": FLOOR_GEOMEAN_NS,
        "candidate_times_sha256": EXPECTED_SHA256[CANDIDATE_TIMES],
        "canonical_sha256": EXPECTED_SHA256[CANONICAL],
        "bands": {},
    }
    selected_total = 0
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
        selected_total += len(selected_lines)
        output = args.output_dir / f"band_{name}.jsonl"
        output.write_bytes(b"".join(selected_lines))

        band_ids = [str(json.loads(line)["id"]) for line in selected_lines]
        band_log = sum(
            math.log(int(candidate[case_id]["min_ns"])) for case_id in band_ids
        )
        identity = [
            {
                key: candidate[case_id][key]
                for key in ("id", "pattern_bytes", "expected_parse", "actual_parse")
            }
            for case_id in band_ids
        ]
        manifest["bands"][name] = {  # type: ignore[index]
            "range_ns": [lower, upper],
            "count": len(selected_lines),
            "log_share": band_log / total_log,
            "input_sha256": sha256(output),
            "identity": identity,
        }
        print(
            f"band={name} range=[{lower},{upper}) count={len(selected_lines)} "
            f"log_share={band_log / total_log:.6f} input_sha256={sha256(output)}"
        )
    if selected_total + EXCLUDED_GE_20US != len(candidate_rows):
        raise SystemExit("REFUSE: band partition + >=20us exclusion does not re-sum")

    manifest_path = ARTIFACT_DIR / "band_manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=1, sort_keys=True) + "\n")
    print(
        f"manifest={manifest_path.name} sha256={sha256(manifest_path)} "
        f"excluded_ge_20us={EXCLUDED_GE_20US} "
        f"floor_geomean_ns={FLOOR_GEOMEAN_NS!r}"
    )


if __name__ == "__main__":
    main()
