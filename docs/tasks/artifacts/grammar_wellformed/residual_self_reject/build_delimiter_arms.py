#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6c — build the DELIMITER-CONTAINMENT arms on SCRATCH COPIES.

The four `path_reference` / `url_reference` terminals are spelled with `[^\\s]`, i.e.
"any non-whitespace", so a path or URL that is immediately followed by a structural
delimiter SWALLOWS that delimiter and the enclosing collection can never close. These
arms are the CONTROLLED EXPERIMENT that attributes each residual self-rejected stimulus
to that mechanism or clears it — a scratch arm re-scored over the corpus, never a reading
of the input text.

  arm1_close  — exclude the closing/separating delimiters only:  , ] } )
  arm2_arrow  — arm1 plus the arrow's own characters:            = >

⛔ No arm touches `grammars/semantic_annotation.ebnf`. Every substitution is a VERBATIM
   literal replace that REFUSES if its anchor is absent or non-unique — a silently-skipped
   edit yields an arm identical to the control and would score as a clean pass.

usage: build_delimiter_arms.py [outdir]   (default: rust/target/h1666c/arms)
"""
import pathlib, sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
SRC = ROOT / "grammars" / "semantic_annotation.ebnf"
OUT = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "rust/target/h1666c/arms"
OUT.mkdir(parents=True, exist_ok=True)
TEXT = SRC.read_text()

ANCHORS = {
    "absolute_path": r"absolute_path := /\/[^\s]*/",
    "relative_path": r"relative_path := /\.\.?\/[^\s]*/",
    "home_path":     r"home_path := /~\/[^\s]*/",
    "url_reference": r"url_reference := /(https?|ftp|file):\/\/[^\s]+/",
}


def sub(text, old, new, label):
    n = text.count(old)
    if n != 1:
        sys.exit(f"build_delimiter_arms: REFUSED — anchor {label!r} occurs {n} times (want exactly 1)")
    return text.replace(old, new)


def arm(name, excluded):
    """Rewrite each terminal's `[^\\s]` class to also exclude `excluded`."""
    t = TEXT
    cls = r"[^\s" + excluded + "]"
    for label, anchor in ANCHORS.items():
        t = sub(t, anchor, anchor.replace(r"[^\s]", cls), label)
    (OUT / name).write_text(t)
    print(f"{name:<24} excluded={excluded!r} anchors_edited={len(ANCHORS)} chars={len(t)}")


(OUT / "arm0_pristine.ebnf").write_text(TEXT)
print(f"{'arm0_pristine.ebnf':<24} excluded=''          anchors_edited=0 chars={len(TEXT)}")
arm("arm1_close.ebnf", r",\]\}\)")
arm("arm2_arrow.ebnf", r",\]\}\)=>")
