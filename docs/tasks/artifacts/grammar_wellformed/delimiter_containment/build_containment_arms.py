#!/usr/bin/env python3
r"""GRAMMAR-WELLFORMED.H.16.6d — build the three CANDIDATE containment formulations on SCRATCH COPIES.

`H.16.6c` proved the mechanism (four `[^\s]` path/URL terminals swallow `] } ) ,`) and priced the
first two candidates. This builds all three the leaf must choose between, so the choice is scored
rather than argued:

  arm1_close  the value language's delimiters only          exclude  , ] } )
  arm2_arrow  arm1 plus the arrow's own characters          exclude  , ] } ) = >
  arm4_escape arm3 PLUS a backslash escape, so a delimiter can still appear in a path or URL —
              `https://h/a\,b` — instead of forcing the whole value to be quoted. ⭐ This is NOT a
              new convention: `double_quoted_string := /"([^"\\]|\\.)*"/` (:158) and
              `single_quoted_string` (:165) already use exactly this shape for the quote delimiter,
              so the four path/URL terminals are being made CONSISTENT with the six string
              terminals in the same file. (Director's proposal, 2026-08-23.)

  arm3_rfc    RFC 3986 §2 excludes < > from a URI outright, and `=` stays legal INSIDE a URL
              (query strings) but may not be the LAST character — which is what stops
              `http://x=>y` from eating the arrow, without forbidding `a=1&b=2`.
              exclude  , ] } ) >   AND  no trailing `=`

⛔ A FOURTH CANDIDATE — a trailing `!(/\s*/ "=>")` guard on the terminal — is REFUTED and
   deliberately not built: a guard after a greedy regex atom cannot SHORTEN the match, only reject
   the whole alternative. Measured: it leaves `http://PYJ=>http://aFC` accepted (the regex still ate
   the arrow, so the guard sees nothing after it) while making `{ http://PYJ => http://aFC }` REJECT
   outright — a URL can no longer be a map key at all. Strictly worse than the control.

⛔ No arm touches `grammars/semantic_annotation.ebnf`. Every substitution is a VERBATIM literal
   replace that REFUSES on a missing or non-unique anchor — a silently-skipped edit would produce an
   arm identical to the control and score as a clean pass.

usage: build_containment_arms.py [outdir]   (default: rust/target/h1666d/arms)
"""
import pathlib, sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
SRC = ROOT / "grammars" / "semantic_annotation.ebnf"
OUT = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "rust/target/h1666d/arms"
OUT.mkdir(parents=True, exist_ok=True)
TEXT = SRC.read_text()

# (label, anchor, body-after-the-`/`-prefix, whether the tail may be EMPTY)
#   `absolute_path`/`home_path`/`relative_path` use `*` — a bare `/` is a legal path — so their
#   rewrite must keep the tail optional; `url_reference` uses `+` and must not.
TERMINALS = [
    ("absolute_path", r"absolute_path := /\/[^\s]*/",                    r"/\/",                    True),
    ("relative_path", r"relative_path := /\.\.?\/[^\s]*/",               r"/\.\.?\/",               True),
    ("home_path",     r"home_path := /~\/[^\s]*/",                       r"/~\/",                   True),
    ("url_reference", r"url_reference := /(https?|ftp|file):\/\/[^\s]+/", r"/(https?|ftp|file):\/\/", False),
]


def sub(text, old, new, label):
    n = text.count(old)
    if n != 1:
        sys.exit(f"build_containment_arms: REFUSED — anchor {label!r} occurs {n} times (want exactly 1)")
    return text.replace(old, new)


def arm(name, excluded, no_trailing_eq, escape=False):
    if escape:
        # the shape `double_quoted_string` already uses: "not a delimiter and not a backslash,
        # OR a backslash followed by anything".
        cls = r"([^\s" + excluded + r"\\]|\\.)"
        last = r"([^\s" + excluded + r"=\\]|\\.)"
    else:
        cls = r"[^\s" + excluded + "]"
        last = r"[^\s" + excluded + "=]"
    t = TEXT
    for label, anchor, prefix, may_be_empty in TERMINALS:
        if no_trailing_eq:
            body = f"({cls}*{last})?" if may_be_empty else f"{cls}*{last}"
        else:
            body = f"{cls}*" if may_be_empty else f"{cls}+"
        new_rule = f"{label} := {prefix}{body}/"
        t = sub(t, anchor, new_rule, label)
    (OUT / name).write_text(t)
    print(f"{name:<22} excluded={excluded!r:<16} no_trailing_eq={str(no_trailing_eq):<5} "
          f"escape={str(escape):<5} chars={len(t)}")


(OUT / "arm0_pristine.ebnf").write_text(TEXT)
print(f"{'arm0_pristine.ebnf':<22} excluded={'':<18} no_trailing_eq=n/a   chars={len(TEXT)}")
arm("arm1_close.ebnf", r",\]\}\)",   False)
arm("arm2_arrow.ebnf", r",\]\}\)=>", False)
arm("arm3_rfc.ebnf",   r",\]\}\)>",  True)
arm("arm4_escape.ebnf", r",\]\}\)>", True, escape=True)
