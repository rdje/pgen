#!/usr/bin/env python3
"""Fetch and inventory upstream corpora for a PCRE2-flavor regex project.

This script is intentionally conservative:
- it reads manifests/upstreams.lock.json
- downloads pinned upstream archives
- extracts selected subsets into third_party/upstream/
- writes source-file inventories into corpus/pcre2/

It does not yet attempt full case-level normalization of pcre2test or PHPT files.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import sys
import tarfile
import urllib.request
from typing import Any, Dict, Iterable, List

ROOT = Path(__file__).resolve().parents[1]
LOCKFILE = ROOT / "manifests" / "upstreams.lock.json"
CACHE_DIR = ROOT / ".cache" / "downloads"


def load_lockfile(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def download(url: str, dst: Path, force: bool = False) -> None:
    ensure_parent(dst)
    if dst.exists() and not force:
        print(f"[skip] already downloaded: {dst}")
        return
    print(f"[download] {url}")
    with urllib.request.urlopen(url) as resp:
        data = resp.read()
    with dst.open("wb") as fh:
        fh.write(data)
    print(f"[saved] {dst} ({len(data)} bytes, sha256={sha256_file(dst)})")


def safe_extract_selected_tar(
    archive_path: Path,
    extract_root: str,
    subpaths: Iterable[str],
    destination: Path,
    force: bool = False,
) -> None:
    if destination.exists() and force:
        shutil.rmtree(destination)
    destination.mkdir(parents=True, exist_ok=True)

    wanted = [p.strip("/") for p in subpaths]
    prefix = extract_root.rstrip("/") + "/"

    with tarfile.open(archive_path, "r:gz") as tf:
        members = tf.getmembers()
        matched = 0
        for member in members:
            name = member.name
            if not name.startswith(prefix):
                continue
            rel = name[len(prefix):]
            if not rel:
                continue
            if not any(rel == w or rel.startswith(w + "/") for w in wanted):
                continue
            target_path = destination / rel
            if member.isdir():
                target_path.mkdir(parents=True, exist_ok=True)
                matched += 1
                continue
            if not member.isfile():
                continue
            target_path.parent.mkdir(parents=True, exist_ok=True)
            extracted = tf.extractfile(member)
            if extracted is None:
                continue
            with target_path.open("wb") as fh:
                shutil.copyfileobj(extracted, fh)
            matched += 1
    print(f"[extract] {archive_path.name} -> {destination} ({matched} selected entries)")


def write_json(path: Path, data: Any) -> None:
    ensure_parent(path)
    with path.open("w", encoding="utf-8") as fh:
        json.dump(data, fh, indent=2, ensure_ascii=False)
        fh.write("\n")


def write_jsonl(path: Path, rows: Iterable[Dict[str, Any]]) -> None:
    ensure_parent(path)
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, ensure_ascii=False))
            fh.write("\n")


def relative_to_root(path: Path) -> str:
    return path.resolve().relative_to(ROOT.resolve()).as_posix()


def inventory_pcre2(upstream: Dict[str, Any]) -> None:
    base = ROOT / upstream["destination"]
    testdata = base / "testdata"
    readme = base / "README"

    rows: List[Dict[str, Any]] = []
    if testdata.exists():
        for path in sorted(testdata.rglob("*")):
            if not path.is_file():
                continue
            rel = path.relative_to(base).as_posix()
            kind = "supporting"
            if path.name.startswith("testinput"):
                kind = "pcre2test-input"
            elif path.name.startswith("testoutput"):
                kind = "pcre2test-output"
            elif path.name.startswith("grep"):
                kind = "pcre2grep-test"
            rows.append(
                {
                    "id": f"pcre2-file:{rel}",
                    "source": "pcre2",
                    "tier": upstream["tier"],
                    "ref": upstream["ref"],
                    "repo": upstream["repo"],
                    "path": rel,
                    "kind": kind,
                    "size_bytes": path.stat().st_size,
                }
            )

    summary = {
        "source": "pcre2",
        "repo": upstream["repo"],
        "ref": upstream["ref"],
        "destination": upstream["destination"],
        "files_indexed": len(rows),
        "readme_present": readme.exists(),
        "counts_by_kind": count_by_key(rows, "kind"),
    }
    inv = upstream["inventory"]
    write_json(ROOT / inv["output_json"], summary)
    write_jsonl(ROOT / inv["output_jsonl"], rows)
    print(f"[inventory] wrote {inv['output_json']} and {inv['output_jsonl']}")


def inventory_php(upstream: Dict[str, Any]) -> None:
    base = ROOT / upstream["destination"]
    tests = base / "ext" / "pcre" / "tests"

    rows: List[Dict[str, Any]] = []
    if tests.exists():
        for path in sorted(tests.rglob("*.phpt")):
            rel = path.relative_to(base).as_posix()
            rows.append(
                {
                    "id": f"php-phpt:{rel}",
                    "source": "php-src",
                    "tier": upstream["tier"],
                    "ref": upstream["ref"],
                    "repo": upstream["repo"],
                    "path": rel,
                    "kind": "php-phpt",
                    "size_bytes": path.stat().st_size,
                }
            )

    summary = {
        "source": "php-src",
        "repo": upstream["repo"],
        "ref": upstream["ref"],
        "destination": upstream["destination"],
        "files_indexed": len(rows),
        "counts_by_kind": count_by_key(rows, "kind"),
    }
    inv = upstream["inventory"]
    write_json(ROOT / inv["output_json"], summary)
    write_jsonl(ROOT / inv["output_jsonl"], rows)
    print(f"[inventory] wrote {inv['output_json']} and {inv['output_jsonl']}")


def count_by_key(rows: Iterable[Dict[str, Any]], key: str) -> Dict[str, int]:
    out: Dict[str, int] = {}
    for row in rows:
        value = str(row.get(key, ""))
        out[value] = out.get(value, 0) + 1
    return dict(sorted(out.items()))


def fetch_one(upstream: Dict[str, Any], force: bool = False) -> None:
    archive_name = f"{upstream['name']}-{upstream['ref']}.tar.gz"
    archive_path = CACHE_DIR / archive_name
    download(upstream["archive_url"], archive_path, force=force)
    safe_extract_selected_tar(
        archive_path=archive_path,
        extract_root=upstream["extract_root"],
        subpaths=upstream["extract_subpaths"],
        destination=ROOT / upstream["destination"],
        force=force,
    )


def run_inventory(upstream: Dict[str, Any]) -> None:
    kind = upstream["inventory"]["kind"]
    if kind == "pcre2-testdata":
        inventory_pcre2(upstream)
        return
    if kind == "php-phpt":
        inventory_php(upstream)
        return
    raise ValueError(f"Unsupported inventory kind: {kind}")


def parse_args(argv: List[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--all", action="store_true", help="Fetch all configured sources and write inventories.")
    parser.add_argument("--inventory-only", action="store_true", help="Do not download; only rebuild inventories from existing extracted sources.")
    parser.add_argument("--source", action="append", default=[], help="Limit execution to one or more source names from the lockfile.")
    parser.add_argument("--force", action="store_true", help="Re-download and re-extract selected sources.")
    return parser.parse_args(argv)


def main(argv: List[str]) -> int:
    args = parse_args(argv)
    lock = load_lockfile(LOCKFILE)
    upstreams = lock.get("upstreams", [])
    if args.source:
        selected = set(args.source)
        upstreams = [u for u in upstreams if u["name"] in selected]
    if not upstreams:
        print("No upstreams selected.", file=sys.stderr)
        return 2

    do_fetch = args.all or not args.inventory_only
    do_inventory = args.all or args.inventory_only or not args.source or True

    for upstream in upstreams:
        if do_fetch and not args.inventory_only:
            fetch_one(upstream, force=args.force)
        run_inventory(upstream)

    print("[done]")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
