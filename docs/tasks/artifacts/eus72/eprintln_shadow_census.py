#!/usr/bin/env python3
"""docs/tasks/artifacts/eus72/eprintln_shadow_census.py

`ENGINE-UNIVERSAL-SERVICES.72` — which `eprintln!` calls are SILENTLY verbosity-gated?

⛔ THE MECHANISM. `rust/src/ast_pipeline/mod.rs` and `.../ast_based_generator.rs` each
declare `macro_rules! eprintln { ($($arg:tt)*) => { crate::pgen_trace_debug!($($arg)*) } }`.
Inside those files a bare `eprintln!` is therefore NOT stderr: it is a trace record that
materialises only at `PGEN_TRACE_VERBOSITY=debug` and is dropped silently otherwise. Only
`std::eprintln!` reaches stderr unconditionally.

That is deliberate and mostly correct — those files carry large verbose debug dumps. It
collides with a standing doctrine only where the message is SEVERITY-BEARING:
`docs/decisions/feedback_severity_never_gated_by_verbosity`. The file already defends one
banner with exactly that argument (`mod.rs`, NON_CANONICAL_BANNER).

⛔⛔ THE FIRST CENSUS I WROTE FOR THIS WAS WRONG AND ITS NUMBER MUST NOT BE QUOTED. It
searched a fixed 6-LINE WINDOW after each `eprintln!`, so a severity marker belonging to a
LATER statement was attributed to an earlier one — it scored `eprintln!("File: {}:{}")` as
severity-bearing because a `⚠️ WARNING` line sat two statements below. This version reads
the ONE statement, by balancing parentheses from the macro's own opening paren, and
attributes only what that statement contains.

⚠️ WHAT THIS STILL CANNOT DECIDE, stated before the number is read: a severity marker inside
a deliberately verbose debug DUMP is correctly gated. This census finds CANDIDATES; the
adjudication of intent is human and belongs in the leaf, not here.

Usage:  python3 docs/tasks/artifacts/eus72/eprintln_shadow_census.py [--list]
Exit:   0 always — this is a census, not a gate.
"""
from __future__ import annotations

import re
import sys

# (path, line of its `macro_rules! eprintln`). ⛔ THE LINE IS LOAD-BEARING: `macro_rules!`
# is TEXTUALLY SCOPED, so ONLY call sites AFTER the definition are shadowed. The first
# version of this census ignored that and scanned whole files, counting `mod.rs:424` —
# which sits ABOVE the definition and reaches stderr normally — as suppressed. It cost one
# site here because the macro sits near the top; in a file where it sat near the bottom the
# same bug would have inverted the answer.
FILES = [
    ("rust/src/ast_pipeline/mod.rs", 532),
    ("rust/src/ast_pipeline/ast_based_generator.rs", 90),
]
SEVERITY = re.compile(r"WARNING|ERROR|FAILED|FAILURE|⚠|❌|🛑|⛔")
CALL = re.compile(r"(?<![:\w])eprintln!\s*[(\[{]")
# ⛔ SEPARATE PATTERN, because the one above CANNOT match a qualified call: its
# `(?<![:\w])` lookbehind excludes the `:` of `std::`. The first version of this
# census counted `std_qualified` by testing that same match for a `std::` prefix,
# so the counter read 0 BY CONSTRUCTION — on a file that demonstrably has two.
# A counter that cannot be non-zero is not a measurement.
STD_CALL = re.compile(r"std::eprintln!\s*[(\[{]")


def statement_at(src: str, open_idx: int) -> str:
    """Return the macro's full argument text by balancing its delimiter."""
    opener = src[open_idx]
    closer = {"(": ")", "[": "]", "{": "}"}[opener]
    depth, i, in_str, esc = 0, open_idx, False, False
    while i < len(src):
        c = src[i]
        if in_str:
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == opener:
            depth += 1
        elif c == closer:
            depth -= 1
            if depth == 0:
                return src[open_idx : i + 1]
        i += 1
    return src[open_idx:]


def main() -> int:
    show = "--list" in sys.argv
    grand_shadowed = grand_sev = 0
    for path, macro_line in FILES:
        src = open(path).read()
        shadowed = severity_hits = before_shadow = 0
        std_qualified = len(
            [
                m
                for m in STD_CALL.finditer(src)
                if not src[src.rfind("\n", 0, m.start()) + 1 : m.start()].lstrip().startswith("//")
            ]
        )
        rows = []
        for m in CALL.finditer(src):
            start = m.start()
            line_start = src.rfind("\n", 0, start) + 1
            prefix = src[line_start:start]
            if prefix.lstrip().startswith("//") or prefix.lstrip().startswith("///"):
                continue
            if "macro_rules" in src[line_start : line_start + 60]:
                continue
            if src.count("\n", 0, start) + 1 < macro_line:
                before_shadow += 1        # above the definition => real stderr
                continue
            shadowed += 1
            body = statement_at(src, m.end() - 1)
            if SEVERITY.search(body):
                severity_hits += 1
                rows.append((src.count("\n", 0, start) + 1, body.replace("\n", " ")[:96]))
        grand_shadowed += shadowed
        grand_sev += severity_hits
        print(f"{path}")
        print(f"  std::eprintln!  (reaches stderr)              : {std_qualified}")
        print(f"  bare eprintln! ABOVE the macro (real stderr)  : {before_shadow}")
        print(f"  bare eprintln!  (SHADOWED -> debug-gated)     : {shadowed}")
        print(f"  ...of which the STATEMENT carries a severity  : {severity_hits}")
        if show:
            for ln, body in rows:
                print(f"      :{ln}  {body}")
        print()
    print(f"TOTAL shadowed={grand_shadowed}  severity-bearing candidates={grand_sev}")
    print("⚠️ CANDIDATES, not defects: a severity marker inside a deliberately verbose")
    print("   debug dump is correctly gated. Adjudication belongs in the leaf.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
