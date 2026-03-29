#!/usr/bin/env python3
"""Normalize PCRE2 testinput/testoutput compile expectations into JSONL.

This script pairs `testdata/testinput2` with `testdata/testoutput2` and emits a
UTF-8-safe, suffix-filtered compile-oracle corpus. Each JSONL row records
whether PCRE2 accepted or rejected the pattern at compile time.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


ROOT = Path(__file__).resolve().parents[1]
LOCKFILE = ROOT / "manifests" / "upstreams.lock.json"

DEFAULT_OUTPUT_JSONL = ROOT / "corpus" / "pcre2" / "canonical" / "pcre2_compile_oracle_cases.jsonl"
DEFAULT_SUMMARY_JSON = ROOT / "corpus" / "pcre2" / "canonical" / "pcre2_compile_oracle_summary.json"
DEFAULT_SKIP_JSONL = ROOT / "corpus" / "pcre2" / "canonical" / "pcre2_compile_oracle_skips.jsonl"

UNSUPPORTED_SUFFIX_TOKENS = {"glob", "hex", "literal"}
FAILED_RE = re.compile(r"^Failed:\s+error\s+(\d+)\b(?::\s*(.*))?$")


@dataclass
class RawCase:
    relative_file: str
    line_number: int
    raw_pattern: bytes
    raw_suffix: bytes
    expected_parse: Optional[str] = None
    expected_error_code: Optional[str] = None
    expected_error_message: Optional[str] = None


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2, ensure_ascii=False)
        fh.write("\n")


def write_jsonl(path: Path, rows: Iterable[Dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, ensure_ascii=False))
            fh.write("\n")


def resolve_pcre2_upstream(lockfile: Dict[str, Any]) -> Dict[str, Any]:
    for upstream in lockfile.get("upstreams", []):
        if upstream.get("name") == "pcre2":
            return upstream
    raise SystemExit("error: pcre2 upstream entry missing from manifests/upstreams.lock.json")


def parse_spec_bytes(line: bytes) -> Optional[Tuple[bytes, bytes]]:
    if not line.startswith(b"/"):
        return None
    index = 1
    while index < len(line):
        if line[index] == 0x2F:
            backslashes = 0
            scan = index - 1
            while scan >= 0 and line[scan] == 0x5C:
                backslashes += 1
                scan -= 1
            if backslashes % 2 == 0:
                return line[1:index], line[index + 1 :]
        index += 1
    return None


def compact_line_preview(line: bytes, max_bytes: int = 96) -> str:
    preview = line[:max_bytes]
    try:
        rendered = preview.decode("utf-8")
    except UnicodeDecodeError:
        rendered = preview.decode("utf-8", errors="replace")
    return rendered.replace("\n", "\\n").replace("\r", "\\r")


def split_suffix_tokens(raw_suffix: str) -> List[str]:
    return [token.strip() for token in raw_suffix.split(",") if token.strip()]


def extract_input_specs(path: Path, upstream_root: Path) -> List[RawCase]:
    relative_file = path.relative_to(upstream_root).as_posix()
    rows: List[RawCase] = []
    for line_number, line in enumerate(path.read_bytes().splitlines(), start=1):
        parsed = parse_spec_bytes(line)
        if parsed is None:
            continue
        raw_pattern, raw_suffix = parsed
        rows.append(
            RawCase(
                relative_file=relative_file,
                line_number=line_number,
                raw_pattern=raw_pattern,
                raw_suffix=raw_suffix,
            )
        )
    return rows


def extract_output_specs(path: Path, upstream_root: Path) -> List[RawCase]:
    relative_file = path.relative_to(upstream_root).as_posix()
    lines = path.read_bytes().splitlines()
    rows: List[RawCase] = []

    for index, line in enumerate(lines):
        parsed = parse_spec_bytes(line)
        if parsed is None:
            continue
        raw_pattern, raw_suffix = parsed
        expected_parse = "ok"
        expected_error_code = None
        expected_error_message = None

        probe = index + 1
        while probe < len(lines):
            next_line = lines[probe]
            if parse_spec_bytes(next_line) is not None:
                break
            try:
                rendered = next_line.decode("utf-8")
            except UnicodeDecodeError:
                rendered = next_line.decode("utf-8", errors="replace")
            stripped = rendered.strip()
            if stripped.startswith("Failed:"):
                expected_parse = "fail"
                match = FAILED_RE.match(stripped)
                if match:
                    expected_error_code = match.group(1)
                    expected_error_message = match.group(2) or stripped
                else:
                    expected_error_message = stripped
                break
            probe += 1

        rows.append(
            RawCase(
                relative_file=relative_file,
                line_number=index + 1,
                raw_pattern=raw_pattern,
                raw_suffix=raw_suffix,
                expected_parse=expected_parse,
                expected_error_code=expected_error_code,
                expected_error_message=expected_error_message,
            )
        )
    return rows


def build_case(
    upstream: Dict[str, Any],
    relative_file: str,
    line_number: int,
    pattern: str,
    raw_suffix: str,
    suffix_tokens: List[str],
    expected_parse: str,
    expected_error_code: Optional[str],
    expected_error_message: Optional[str],
) -> Dict[str, Any]:
    normalized_tokens = [token.lower() for token in suffix_tokens]
    tags = ["upstream-pcre2test", "compile-oracle"]
    tags.extend(f"suffix:{token}" for token in normalized_tokens)

    notes = []
    if raw_suffix:
        notes.append(f"pcre2test_suffix={raw_suffix}")

    return {
        "id": f"pcre2:{relative_file}:line_{line_number}",
        "flavor": "pcre2",
        "tier": "canonical",
        "pattern": pattern,
        "source": {
            "repo": upstream["repo"],
            "ref": upstream["ref"],
            "file": relative_file,
            "case_ref": f"line_{line_number}",
            "line_start": line_number,
            "line_end": line_number,
            "parent_case_id": None,
        },
        "wrapper": {
            "kind": "pcre2test",
            "delimiter": "/",
            "modifiers": None,
            "callsite": None,
        },
        "compile": {
            "options": suffix_tokens,
            "newline": None,
            "bsr": None,
            "jit": None,
        },
        "expected": {
            "parse": expected_parse,
            "error_layer": "compile" if expected_parse == "fail" else None,
            "error_code": expected_error_code,
            "error_message": expected_error_message,
        },
        "subjects": [],
        "tags": tags,
        "notes": notes,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-jsonl", type=Path, default=DEFAULT_OUTPUT_JSONL)
    parser.add_argument("--summary-json", type=Path, default=DEFAULT_SUMMARY_JSON)
    parser.add_argument("--skip-jsonl", type=Path, default=DEFAULT_SKIP_JSONL)
    parser.add_argument("--max-cases", type=int, default=0)
    args = parser.parse_args()

    if args.max_cases < 0:
        raise SystemExit("error: --max-cases must be >= 0")

    lockfile = load_json(LOCKFILE)
    upstream = resolve_pcre2_upstream(lockfile)
    upstream_root = ROOT / upstream["destination"]
    input_path = upstream_root / "testdata" / "testinput2"
    output_path = upstream_root / "testdata" / "testoutput2"

    if not input_path.is_file():
        raise SystemExit(f"error: missing PCRE2 input file: {input_path}")
    if not output_path.is_file():
        raise SystemExit(f"error: missing PCRE2 output file: {output_path}")

    input_specs = extract_input_specs(input_path, upstream_root)
    output_specs = extract_output_specs(output_path, upstream_root)

    if len(input_specs) != len(output_specs):
        raise SystemExit(
            "error: input/output case count drift for testinput2/testoutput2 "
            f"({len(input_specs)} != {len(output_specs)})"
        )

    rows: List[Dict[str, Any]] = []
    skips: List[Dict[str, Any]] = []
    option_counts: Counter[str] = Counter()
    unsupported_token_counts: Counter[str] = Counter()
    skip_reason_counts: Counter[str] = Counter()
    expected_parse_counts: Counter[str] = Counter()

    for input_case, output_case in zip(input_specs, output_specs):
        if (
            input_case.raw_pattern != output_case.raw_pattern
            or input_case.raw_suffix != output_case.raw_suffix
        ):
            raise SystemExit(
                "error: input/output case drift while pairing compile-oracle corpus "
                f"at {input_case.relative_file}:line_{input_case.line_number}"
            )

        try:
            pattern = input_case.raw_pattern.decode("utf-8")
            raw_suffix = input_case.raw_suffix.decode("utf-8")
        except UnicodeDecodeError:
            skip_reason_counts["utf8_decode_failure"] += 1
            skips.append(
                {
                    "source": {
                        "repo": upstream["repo"],
                        "ref": upstream["ref"],
                        "file": input_case.relative_file,
                        "line_start": input_case.line_number,
                        "line_end": input_case.line_number,
                    },
                    "reason": "utf8_decode_failure",
                    "raw_line_preview": compact_line_preview(
                        b"/" + input_case.raw_pattern + b"/" + input_case.raw_suffix
                    ),
                }
            )
            continue

        suffix_tokens = split_suffix_tokens(raw_suffix)
        normalized_tokens = [token.lower() for token in suffix_tokens]
        unsupported = sorted(set(normalized_tokens) & UNSUPPORTED_SUFFIX_TOKENS)
        if unsupported:
            skip_reason_counts["unsupported_suffix_token"] += 1
            unsupported_token_counts.update(unsupported)
            skips.append(
                {
                    "source": {
                        "repo": upstream["repo"],
                        "ref": upstream["ref"],
                        "file": input_case.relative_file,
                        "line_start": input_case.line_number,
                        "line_end": input_case.line_number,
                    },
                    "reason": "unsupported_suffix_token",
                    "unsupported_suffix_tokens": unsupported,
                    "raw_suffix": raw_suffix,
                    "raw_line_preview": compact_line_preview(
                        b"/" + input_case.raw_pattern + b"/" + input_case.raw_suffix
                    ),
                }
            )
            continue

        expected_parse = output_case.expected_parse or "ok"
        expected_parse_counts[expected_parse] += 1
        option_counts.update(normalized_tokens)
        rows.append(
            build_case(
                upstream=upstream,
                relative_file=input_case.relative_file,
                line_number=input_case.line_number,
                pattern=pattern,
                raw_suffix=raw_suffix,
                suffix_tokens=suffix_tokens,
                expected_parse=expected_parse,
                expected_error_code=output_case.expected_error_code,
                expected_error_message=output_case.expected_error_message,
            )
        )

        if args.max_cases > 0 and len(rows) >= args.max_cases:
            break

    summary = {
        "source": "pcre2",
        "repo": upstream["repo"],
        "ref": upstream["ref"],
        "mode": "compile-oracle-normalization",
        "input_file": "testdata/testinput2",
        "output_file": "testdata/testoutput2",
        "pattern_specs_detected_input": len(input_specs),
        "pattern_specs_detected_output": len(output_specs),
        "cases_emitted": len(rows),
        "max_cases": args.max_cases,
        "expected_parse_counts": dict(sorted(expected_parse_counts.items())),
        "skip_reason_counts": dict(sorted(skip_reason_counts.items())),
        "unsupported_suffix_token_counts": dict(sorted(unsupported_token_counts.items())),
        "compile_option_counts": dict(sorted(option_counts.items())),
        "outputs": {
            "jsonl": str(args.output_jsonl),
            "summary_json": str(args.summary_json),
            "skip_jsonl": str(args.skip_jsonl),
        },
    }

    write_jsonl(args.output_jsonl, rows)
    write_json(args.summary_json, summary)
    write_jsonl(args.skip_jsonl, skips)

    print(
        json.dumps(
            {
                "cases_emitted": len(rows),
                "expected_parse_counts": dict(sorted(expected_parse_counts.items())),
                "skip_reason_counts": dict(sorted(skip_reason_counts.items())),
                "unsupported_suffix_token_counts": dict(sorted(unsupported_token_counts.items())),
            },
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
