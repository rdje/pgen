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

⛔⛔ CORRECTED AGAIN BY THE FIX ITSELF (`.72`, 2026-09-29), TWICE: (1) the macro's LINE NUMBER was a
hand-carried copy (`FILES` held 532 / 90) — the macro is now located by TEXT, and a file without one
reports the shadow ABSENT, so every bare call there is stderr; (2) a bare `eprintln!` INSIDE a
`quote!` / `quote_spanned!` template is EMITTED TEXT, not a call — it becomes a token of the generated
parser, where `eprintln!` is std's — so it was never shadowed. The first census counted five such
template tokens in the generator as suppressed calls (247 read where 242 is right), and the fix's first
cut rewrote them, which changed every generated artifact by exactly 127 bytes; the reproducibility
verify caught it. Template tokens are now counted on their own line and never as shadowed.

Usage:  python3 docs/tasks/artifacts/eus72/eprintln_shadow_census.py [--list] [--rev GIT_REV]
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
    "rust/src/ast_pipeline/mod.rs",
    "rust/src/ast_pipeline/ast_based_generator.rs",
]
SHADOW_DEF = re.compile(r"^macro_rules! eprintln \{", re.M)
QUOTE_OPEN = re.compile(r"\bquote(?:_spanned)?!\s*[\{\(]")


def inside_quote_template(src: str, pos: int) -> bool:
    """Is `pos` inside an unclosed `quote!{…}` / `quote!(…)` block? Such text is EMITTED, never called."""
    for m in QUOTE_OPEN.finditer(src):
        if m.end() > pos:
            break
        opener = src[m.end() - 1]
        closer = {"{": "}", "(": ")"}[opener]
        depth, i = 1, m.end()
        while i < pos and depth:
            if src[i] == opener:
                depth += 1
            elif src[i] == closer:
                depth -= 1
            i += 1
        if depth > 0:
            return True
    return False


def read_source(path: str, rev: str | None) -> str:
    if rev is None:
        return open(path).read()
    import subprocess
    return subprocess.run(["git", "show", f"{rev}:{path}"], capture_output=True, text=True, check=True).stdout
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
    rev = sys.argv[sys.argv.index("--rev") + 1] if "--rev" in sys.argv else None
    grand_shadowed = grand_sev = 0
    for path in FILES:
        src = read_source(path, rev)
        shadow = SHADOW_DEF.search(src)
        macro_line = src.count("\n", 0, shadow.start()) + 1 if shadow else None
        shadowed = severity_hits = before_shadow = template_tokens = 0
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
            if inside_quote_template(src, start):
                template_tokens += 1      # emitted text of a generated parser, never a call here
                continue
            if macro_line is None or src.count("\n", 0, start) + 1 < macro_line:
                before_shadow += 1        # no shadow, or above the definition => real stderr
                continue
            shadowed += 1
            body = statement_at(src, m.end() - 1)
            if SEVERITY.search(body):
                severity_hits += 1
                rows.append((src.count("\n", 0, start) + 1, body.replace("\n", " ")[:96]))
        grand_shadowed += shadowed
        grand_sev += severity_hits
        print(f"{path}{'' if rev is None else f' @ {rev}'}")
        print(f"  shadow macro                                  : {'line ' + str(macro_line) if macro_line else 'ABSENT'}")
        print(f"  std::eprintln!  (reaches stderr)              : {std_qualified}")
        print(f"  bare eprintln! NOT shadowed (real stderr)     : {before_shadow}")
        print(f"  eprintln! tokens inside quote! templates      : {template_tokens}  (emitted text, never calls)")
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
