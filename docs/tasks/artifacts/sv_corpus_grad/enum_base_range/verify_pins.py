#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.3.23` — the PER-FILE justification for the `enum [N:M]` pins.

⛔ THE TRAP THIS INSTRUMENT EXISTS TO CLOSE. A file can reject for more than one reason.
Re-adjudicating it `must_reject` on the base-less-`enum [` ground would then MASK a second,
real parser defect behind a correct-looking pin — the row leaves `unexplained_rejects_valid`
and never testifies again. So a pin is only honest when the file is proven stuck **AT the
base-less enum header and nowhere earlier**. `sweep.py` sizes the population; this script
earns each individual pin.

THE PROOF, per row:
  1. locate every base-less `enum` + packed-dimension header in the file, offsets in BYTES,
     with comments and string literals blanked (so a `// enum [3:0]` never anchors a pin);
  2. run the REAL release probe (`--profile sv_2017`) and take its `furthest_position=`;
  3. require verdict REJECT **and** furthest_position inside the FIRST such header's span
     (from the `e` of `enum` through the `{` that opens the member list).
Anything else — ACCEPT, stuck EARLIER, stuck LATER — REFUSES the pin for that row.

Nothing earlier can be the blocker once the parser has demonstrably reached inside that
span: `furthest_position` is the furthest byte any alternative consumed to.

⭐ GROUND TRUTH — this instrument REFUSES rather than guesses
([[feedback_instrument_needs_ground_truth]]). Three controls run before any verdict is
published, and a miss aborts with a nonzero exit:
  * POSITIVE  — the tracked reproducer `e1_dim_no_base.sv` must classify AT-ENUM.
                A miss means the SPAN/anchor logic is broken.
  * NEGATIVE-A— `e1` with a syntax error PLANTED before the enum must classify EARLIER.
                A miss means the differ is blind to exactly the masking case above, i.e.
                it would rubber-stamp every row.
  * NEGATIVE-B— `e3_vector_base_dim.sv` (`enum logic [2:0]`, a LEGAL typed base) must yield
                ZERO base-less matches. A miss means the construct detector matches typed
                enums too, so "the population" would be meaningless.

Usage (repo-root-relative, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/enum_base_range/verify_pins.py
Exit 0 = every row earns its pin. Nonzero = a control missed, or a row must NOT be pinned.
"""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "config_use_param_override"))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
HERE = Path(__file__).resolve().parent
PROBE = ROOT / "rust/target/release/parseability_probe"
SUBS = ROOT / "stimuli/sv/subs"

# The 9 `divergence:unexplained_rejects_valid` rows `sweep.py` sized (sweep.txt).
# (suite, relpath-under-the-suite-submodule)
ROWS = [
    ("sv2v", "test/core/enum_scope.sv"),
    ("verilator", "test_regress/t/t_cast.v"),
    ("verilator", "test_regress/t/t_debug_emitv.v"),
    ("verilator", "test_regress/t/t_enum.v"),
    ("verilator", "test_regress/t/t_enum_bad_value.v"),
    ("verilator", "test_regress/t/t_enum_bad_wrap.v"),
    ("verilator", "test_regress/t/t_enum_const_methods.v"),
    ("verilator", "test_regress/t/t_enum_type_methods_bad.v"),
    ("verilator", "test_regress/t/t_enum_type_nomethod_bad.v"),
]

POS_RE = re.compile(r"furthest_position=(\d+)")
# `enum`, optional layout, `[` — with NO type in between. Byte patterns throughout:
# `furthest_position` is a BYTE offset, so every index here must be one too.
BASELESS_RE = re.compile(rb"(?<![A-Za-z0-9_$])enum\s*\[")
# Comments and string literals, blanked length-preservingly before matching.
COMMENT_STRING_RE = re.compile(rb'//[^\n]*|/\*.*?\*/|"(?:[^"\\\n]|\\.)*"', re.DOTALL)


def blank_noncode(data: bytes) -> bytes:
    """Blank comments/strings, preserving every byte offset (newlines kept)."""
    def repl(m: re.Match) -> bytes:
        return bytes(b if b == 0x0A else 0x20 for b in m.group(0))
    return COMMENT_STRING_RE.sub(repl, data)


def header_spans(data: bytes):
    """Byte spans of every base-less `enum [ … ] {` header, outermost-first.

    span = (start of `enum`, index of the `{` that opens the member list). The `{` is
    included because the whole header is one indivisible LRM-underivable unit: A.2.2.1
    admits a packed_dimension only BEHIND a type, so a parser cannot legally be anywhere
    between the `enum` keyword and that brace on this text.
    """
    code = blank_noncode(data)
    spans = []
    for m in BASELESS_RE.finditer(code):
        i = m.end() - 1  # the `[`
        depth = 0
        while i < len(code):
            if code[i] == 0x5B:      # [
                depth += 1
            elif code[i] == 0x5D:    # ]
                depth -= 1
                if depth == 0:
                    break
            i += 1
        if i >= len(code):
            continue                 # unbalanced `[` — not a header we can bound
        j = i + 1
        while j < len(code) and code[j] in (0x20, 0x09, 0x0D, 0x0A):
            j += 1
        if j < len(code) and code[j] == 0x7B:   # `{` opens the member list
            spans.append((m.start(), j))
    return spans


def probe(path: Path):
    r = subprocess.run(
        [str(PROBE), "--parse", "systemverilog", str(path), "--profile", "sv_2017"],
        capture_output=True, text=True, timeout=300)
    out = r.stdout + r.stderr
    if "parse_full passed" in out:
        return "ACCEPT", None
    m = POS_RE.search(out)
    return "REJECT", int(m.group(1)) if m else None


def classify(path: Path):
    """-> (status, furthest, span, detail). status in AT-ENUM / EARLIER / LATER /
    ACCEPTS / NO-CONSTRUCT / NO-POSITION."""
    data = path.read_bytes()
    spans = header_spans(data)
    verdict, furthest = probe(path)
    if not spans:
        return "NO-CONSTRUCT", furthest, None, "no base-less `enum [` header in the file"
    span = spans[0]
    if verdict == "ACCEPT":
        return "ACCEPTS", None, span, "the parser ACCEPTS this file"
    if furthest is None:
        return "NO-POSITION", None, span, "reject carried no furthest_position="
    if furthest < span[0]:
        return "EARLIER", furthest, span, "stuck BEFORE the enum header — a different defect"
    if furthest > span[1]:
        return "LATER", furthest, span, "stuck AFTER the enum header"
    return "AT-ENUM", furthest, span, ""


def line_of(data: bytes, off: int) -> int:
    return data[:off].count(b"\n") + 1


def run_controls() -> None:
    print("## GROUND TRUTH — controls (a miss ABORTS before any row is published)")
    ok = True

    st, furthest, span, detail = classify(HERE / "e1_dim_no_base.sv")
    good = st == "AT-ENUM"
    ok &= good
    print(f"  POSITIVE   e1_dim_no_base.sv        -> {st:<13} "
          f"{'PASS' if good else 'FAIL — span/anchor logic is broken: ' + detail}")

    src = (HERE / "e1_dim_no_base.sv").read_bytes()
    # Plant a syntax error BEFORE the enum: a stray `)` on its own line after `module m;`.
    planted = src.replace(b"module m;\n", b"module m;\n  );\n", 1)
    with tempfile.NamedTemporaryFile(
            "wb", suffix=".sv", delete=False,
            dir=str(ROOT / "rust/target")) as fh:     # repo volume (directive 13)
        fh.write(planted)
        tmp = Path(fh.name)
    try:
        st_n, _f, _s, detail_n = classify(tmp)
        good = st_n == "EARLIER"
        ok &= good
        print(f"  NEGATIVE-A e1 + planted early error -> {st_n:<13} "
              f"{'PASS' if good else 'FAIL — the differ cannot see an EARLIER stuck point, '
                                     'so it would rubber-stamp every row: ' + detail_n}")
    finally:
        tmp.unlink(missing_ok=True)

    n = len(header_spans((HERE / "e3_vector_base_dim.sv").read_bytes()))
    good = n == 0
    ok &= good
    print(f"  NEGATIVE-B e3_vector_base_dim.sv    -> {n} base-less matches "
          f"{'PASS' if good else 'FAIL — a LEGAL typed base matched the construct detector'}")

    print()
    if not ok:
        raise SystemExit("REFUSE: a ground-truth control missed — no verdict is published.")


def main() -> int:
    if not PROBE.is_file():
        raise SystemExit(f"REFUSE: probe not built at {PROBE.relative_to(ROOT)}")

    run_controls()

    print("## PER-ROW JUSTIFICATION — is each row stuck AT the base-less enum, and nowhere earlier?")
    print(f"  {'suite':<11} {'file':<38} {'status':<10} {'furthest':>8}  "
          f"{'header span':<16} line  construct")
    refused = []
    for suite, rel in ROWS:
        path = SUBS / suite / rel
        if not path.is_file():
            refused.append((suite, rel, "MISSING", "file not vendored at " + str(path)))
            print(f"  {suite:<11} {rel:<38} {'MISSING':<10}")
            continue
        data = path.read_bytes()
        st, furthest, span, detail = classify(path)
        if st != "AT-ENUM":
            refused.append((suite, rel, st, detail))
        span_s = f"{span[0]}..{span[1]}" if span else "-"
        line = line_of(data, span[0]) if span else 0
        text = data[span[0]:span[1] + 1].decode("utf-8", "replace") if span else ""
        text = " ".join(text.split())
        if len(text) > 34:
            text = text[:31] + "..."
        print(f"  {suite:<11} {Path(rel).name:<38} {st:<10} "
              f"{furthest if furthest is not None else '-':>8}  {span_s:<16} {line:<5} {text}")

    print()
    if refused:
        print("## ⛔ REFUSED — these rows must NOT be pinned on the enum ground:")
        for suite, rel, st, detail in refused:
            print(f"    {suite}/{rel}: {st} — {detail}")
        print("\n  A pin here would mask a different defect. Fix the diagnosis, not the number.")
        return 1

    print(f"  ALL {len(ROWS)}/{len(ROWS)} rows are stuck INSIDE the base-less enum header and")
    print("  nowhere earlier => each pin is justified per-file, not by family membership.")
    print("  ⛔ Honest bound: a pinned row can no longer testify about anything AFTER its")
    print("     enum header. That is inherent — the text is LRM-underivable, so the file can")
    print("     never be `must_accept` — but it is a real loss of reach, not a free win.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
