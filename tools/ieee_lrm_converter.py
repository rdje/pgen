#!/usr/bin/env python3
"""
End-to-end IEEE LRM conversion helper.

Pipeline:
1. split_sections.py (PDF TOC -> section text files + manifest)
2. txt_to_md_converter.py (section text -> markdown)
3. optional grammar extraction (extract_grammar.py + extract_grammar_v2.py + create_clean_grammar.py)
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def run(cmd: list[str]) -> None:
    print("+ " + " ".join(cmd))
    subprocess.run(cmd, check=True)


def main() -> int:
    ap = argparse.ArgumentParser(description="Convert IEEE LRM PDF into section txt/md outputs.")
    ap.add_argument("--pdf", required=True, help="Input PDF path")
    ap.add_argument(
        "--out-root",
        required=True,
        help="Output root directory (expects/creates txt and md subdirs)",
    )
    ap.add_argument("--document", required=True, help="Document title")
    ap.add_argument("--standard", required=True, help="Standard label (for example IEEE 1800-2023)")
    ap.add_argument("--domain", default="HDL", help="Domain label")
    ap.add_argument("--clause-depth", type=int, default=1, help="Max clause depth from TOC")
    ap.add_argument("--toc-max-level", type=int, default=3, help="Max TOC nesting level inspected")
    ap.add_argument("--include-annex", action="store_true", help="Include Annex headings")
    ap.add_argument("--limit", type=int, default=None, help="Optional section count cap")
    ap.add_argument(
        "--extract-grammar",
        action="store_true",
        help="Also run grammar extraction/normalization pipeline on markdown output",
    )
    args = ap.parse_args()

    script_dir = Path(__file__).resolve().parent
    pdf = Path(args.pdf).expanduser().resolve()
    out_root = Path(args.out_root).expanduser().resolve()
    txt_dir = out_root / "txt"
    md_dir = out_root / "md"
    txt_dir.mkdir(parents=True, exist_ok=True)
    md_dir.mkdir(parents=True, exist_ok=True)

    if not pdf.is_file():
        raise SystemExit(f"error: PDF not found: {pdf}")

    split_cmd = [
        sys.executable,
        str(script_dir / "split_sections.py"),
        "--pdf",
        str(pdf),
        "--out-dir",
        str(txt_dir),
        "--clause-depth",
        str(args.clause_depth),
        "--toc-max-level",
        str(args.toc_max_level),
        "--standard",
        args.standard,
    ]
    if args.include_annex:
        split_cmd.append("--include-annex")
    if args.limit is not None:
        split_cmd.extend(["--limit", str(args.limit)])
    run(split_cmd)

    run(
        [
            sys.executable,
            str(script_dir / "txt_to_md_converter.py"),
            "--txt-dir",
            str(txt_dir),
            "--md-dir",
            str(md_dir),
            "--document",
            args.document,
            "--standard",
            args.standard,
            "--domain",
            args.domain,
            "--source-pdf",
            str(pdf),
        ]
    )

    if args.extract_grammar:
        catalog = out_root / "grammar_catalog.txt"
        normalized = out_root / "grammar_normalized.ebnf"
        report = out_root / "grammar_report.json"
        clean = out_root / "grammar_clean.ebnf"
        run(
            [
                sys.executable,
                str(script_dir / "extract_grammar.py"),
                "--md-dir",
                str(md_dir),
                "--output",
                str(catalog),
            ]
        )
        run(
            [
                sys.executable,
                str(script_dir / "extract_grammar_v2.py"),
                "--catalog",
                str(catalog),
                "--output-ebnf",
                str(normalized),
                "--output-json",
                str(report),
            ]
        )
        run(
            [
                sys.executable,
                str(script_dir / "create_clean_grammar.py"),
                "--input-ebnf",
                str(normalized),
                "--output-ebnf",
                str(clean),
            ]
        )

    print(f"pdf: {pdf}")
    print(f"out_root: {out_root}")
    print("done: conversion pipeline complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

