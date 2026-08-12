"""Locate the repository root by walking up to the tracked marker.

Directive 12 (repo-root-relative paths): the repository may be moved to any
directory on any volume, so no script here may hard-code a depth. Each one
resolves the root from its own location at run time.
"""
from pathlib import Path


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "CLAUDE.md").is_file() and (parent / "grammars").is_dir():
            return parent
    raise SystemExit(f"REFUSE: no repository root above {here}")
