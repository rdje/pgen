#!/usr/bin/env python3
"""Normalize a conservative text-safe subset of PCRE2 pcre2test inputs.

This script intentionally does not try to infer full compile success/failure from
PCRE2 upstream. It extracts a deterministic, UTF-8-safe subset of raw `/.../`
pattern specs from `testdata/testinput*`, records why lines were skipped, and
emits schema-shaped JSONL that downstream probes can execute against the
generated regex parser.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


ROOT = Path(__file__).resolve().parents[1]
LOCKFILE = ROOT / "manifests" / "upstreams.lock.json"

DEFAULT_OUTPUT_JSONL = ROOT / "corpus" / "pcre2" / "canonical" / "pcre2_textsafe_cases.jsonl"
DEFAULT_SUMMARY_JSON = (
    ROOT / "corpus" / "pcre2" / "canonical" / "pcre2_textsafe_summary.json"
)
DEFAULT_SKIP_JSONL = ROOT / "corpus" / "pcre2" / "canonical" / "pcre2_textsafe_skips.jsonl"

UNSUPPORTED_SUFFIX_TOKENS = {"glob", "hex", "literal"}


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


def build_case(
    upstream: Dict[str, Any],
    relative_file: str,
    line_number: int,
    pattern: str,
    raw_suffix: str,
    suffix_tokens: List[str],
) -> Dict[str, Any]:
    normalized_tokens = [token.lower() for token in suffix_tokens]
    notes = []
    if raw_suffix:
        notes.append(f"pcre2test_suffix={raw_suffix}")

    tags = ["upstream-pcre2test", "text-safe-pattern"]
    tags.extend(f"suffix:{token}" for token in normalized_tokens)

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
            "parse": "unknown",
            "error_layer": None,
            "error_code": None,
            "error_message": None,
        },
        "subjects": [],
        "tags": tags,
        "notes": notes,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output-jsonl",
        type=Path,
        default=DEFAULT_OUTPUT_JSONL,
        help="Destination for normalized JSONL cases.",
    )
    parser.add_argument(
        "--summary-json",
        type=Path,
        default=DEFAULT_SUMMARY_JSON,
        help="Destination for normalization summary JSON.",
    )
    parser.add_argument(
        "--skip-jsonl",
        type=Path,
        default=DEFAULT_SKIP_JSONL,
        help="Destination for skipped-line JSONL.",
    )
    parser.add_argument(
        "--max-cases",
        type=int,
        default=0,
        help="Optional maximum emitted-case count (0 means no limit).",
    )
    args = parser.parse_args()

    if args.max_cases < 0:
        raise SystemExit("error: --max-cases must be >= 0")

    lockfile = load_json(LOCKFILE)
    upstream = resolve_pcre2_upstream(lockfile)
    upstream_root = ROOT / upstream["destination"]
    testdata_root = upstream_root / "testdata"

    if not testdata_root.is_dir():
        raise SystemExit(f"error: missing PCRE2 testdata directory: {testdata_root}")

    rows: List[Dict[str, Any]] = []
    skips: List[Dict[str, Any]] = []
    option_counts: Counter[str] = Counter()
    unsupported_token_counts: Counter[str] = Counter()
    skip_reason_counts: Counter[str] = Counter()

    files_considered = 0
    lines_scanned = 0
    pattern_specs_detected = 0

    for path in sorted(testdata_root.glob("testinput*")):
        if not path.is_file():
            continue
        files_considered += 1
        relative_file = path.relative_to(upstream_root).as_posix()
        for line_number, line in enumerate(path.read_bytes().splitlines(), start=1):
            lines_scanned += 1
            parsed = parse_spec_bytes(line)
            if parsed is None:
                continue
            pattern_specs_detected += 1

            pattern_bytes, suffix_bytes = parsed
            try:
                pattern = pattern_bytes.decode("utf-8")
                raw_suffix = suffix_bytes.decode("utf-8")
            except UnicodeDecodeError:
                skip_reason_counts["utf8_decode_failure"] += 1
                skips.append(
                    {
                        "source": {
                            "repo": upstream["repo"],
                            "ref": upstream["ref"],
                            "file": relative_file,
                            "line_start": line_number,
                            "line_end": line_number,
                        },
                        "reason": "utf8_decode_failure",
                        "raw_line_preview": compact_line_preview(line),
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
                            "file": relative_file,
                            "line_start": line_number,
                            "line_end": line_number,
                        },
                        "reason": "unsupported_suffix_token",
                        "unsupported_suffix_tokens": unsupported,
                        "raw_suffix": raw_suffix,
                        "raw_line_preview": compact_line_preview(line),
                    }
                )
                continue

            option_counts.update(normalized_tokens)
            rows.append(
                build_case(
                    upstream=upstream,
                    relative_file=relative_file,
                    line_number=line_number,
                    pattern=pattern,
                    raw_suffix=raw_suffix,
                    suffix_tokens=suffix_tokens,
                )
            )
            if args.max_cases > 0 and len(rows) >= args.max_cases:
                break
        if args.max_cases > 0 and len(rows) >= args.max_cases:
            break

    summary = {
        "source": "pcre2",
        "repo": upstream["repo"],
        "ref": upstream["ref"],
        "mode": "text-safe-pcre2test-pattern-normalization",
        "upstream_root": upstream["destination"],
        "files_considered": files_considered,
        "lines_scanned": lines_scanned,
        "pattern_specs_detected": pattern_specs_detected,
        "cases_emitted": len(rows),
        "max_cases": args.max_cases,
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
                "pattern_specs_detected": pattern_specs_detected,
                "skip_reason_counts": dict(sorted(skip_reason_counts.items())),
                "unsupported_suffix_token_counts": dict(sorted(unsupported_token_counts.items())),
            },
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
