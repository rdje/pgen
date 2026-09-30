#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.117 — the ORPHANED-ATTRIBUTE control.

A Rust attribute binds to the NEXT item, across any number of blank lines. When a refactor
deletes the item under a `#[cfg(...)]` and leaves the attribute, the attribute silently gates
whatever item follows it — `.112` slice 2 left one above `generated_backend_enabled` in
`rust/src/embedding_api.rs` (both arms of the function vanished under the full feature set with
no family artifact present, the state every cold clone passes through) and `.113` left one above
`usage()` in `rust/src/bin/parseability_probe.rs`. This scan names every `#[cfg(...)]` that is
separated from the item it now governs by a blank line, under `rust/src`. It prints one line per
site and exits 1 when any exists; `.117` reads 0 on HEAD and both sites on `.117`'s parent.

    python3 docs/tasks/artifacts/engine_universal_services/eus117_bare_clone/orphan_cfg_scan.py
    git stash -q; ...; git stash pop -q            # or: run it inside `git archive <rev> | tar -x`
"""
from __future__ import annotations

import glob
import re
import sys

ATTRIBUTE_THEN_BLANK_THEN_ITEM = re.compile(
    r"(#\[cfg\((?:[^\[\]]|\[[^\]]*\])*\)\])\n[ \t]*\n[ \t]*(#\[|fn |pub |use |impl |mod |struct |enum |const |static |type |trait |macro_rules!)"
)


def main() -> int:
    sites = 0
    for path in sorted(glob.glob("rust/src/**/*.rs", recursive=True)):
        text = open(path, encoding="utf-8").read()
        for match in ATTRIBUTE_THEN_BLANK_THEN_ITEM.finditer(text):
            line = text[: match.start()].count("\n") + 1
            print(f"ORPHANED-CFG: {path}:{line}: {match.group(1)}")
            sites += 1
    print(f"ORPHANED-CFG: sites={sites}")
    return 1 if sites else 0


if __name__ == "__main__":
    sys.exit(main())
