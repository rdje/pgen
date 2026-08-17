#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.31 (b)/(c) — the label-hoist RECONSTRUCTION identity.

The hoist replaces one string literal per emitted `Logger::log_*` site with a reference to a
single module constant.  The claim that makes it a *substitution* rather than a *change* is:

    ARM 2  minus its `const PGEN_SOURCE_LABEL: &str = "<path>";` declaration,
           with every `PGEN_SOURCE_LABEL` token replaced by the literal that declaration holds,
    ==     ARM 1, byte for byte.

That is what this file computes.  It is deliberately a *textual* reconstruction rather than a
re-generation: a re-generation would prove only that two runs of one emitter agree, while this
proves that the two EMITTERS differ by exactly the substitution and by nothing else — including
that the `file` argument every diagnostic receives is the same string it received before
(acceptance (c), at the source level; the runtime leg is the before/after trace comparison).

⛔ The substitution is applied to a line only AFTER the declaration line has been consumed, so the
declaration itself can never be rewritten into its own value.  The declaration must appear exactly
once; zero or two is a refusal, not a fallback — a reconstruction that silently skipped a missing
declaration would compare ARM 2 against itself and pass.

⚠️ THIS SCRIPT MUST NOT BE USED AS A HOISTER.  It reverses the hoist for one comparison; the
shipped emission comes from `rust/src/ast_pipeline/ast_based_generator.rs`, and the only supported
way to produce ARM 1 is the tracked `unhoist.patch` beside this file.

USAGE
    python3 reconstruct.py --arm1 <unhoisted.rs> --arm2 <hoisted.rs> [--wrong-literal]
EXIT
    0 = the reconstruction is byte-identical to ARM 1
    1 = it is not (with both digests printed)
    2 = the comparison could not be made correctly (never a pass)
"""

from __future__ import annotations

import argparse
import hashlib
import re
import sys

DECL = re.compile(r'^const PGEN_SOURCE_LABEL: &str = ("(?:[^"\\]|\\.)*");$')
IDENT = "PGEN_SOURCE_LABEL"


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def reconstruct(path: str, override_literal: str | None) -> tuple[str, str, int]:
    """Return (digest, declared_literal, substituted_line_count) for the reconstruction."""
    digest = hashlib.sha256()
    literal: str | None = None
    declarations = 0
    substituted = 0
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            match = DECL.match(line.rstrip("\n"))
            if match is not None:
                declarations += 1
                if declarations == 1:
                    literal = match.group(1)
                    continue  # drop the declaration — ARM 1 has no such line
            if literal is None:
                # A `PGEN_SOURCE_LABEL` use before its declaration would be reconstructed with
                # nothing to put in its place. Refuse rather than emit a hole.
                if IDENT in line:
                    print(
                        f"reconstruct: {path} uses {IDENT} before declaring it — refusing",
                        file=sys.stderr,
                    )
                    sys.exit(2)
            else:
                replacement = override_literal if override_literal is not None else literal
                if IDENT in line:
                    substituted += line.count(IDENT)
                    line = line.replace(IDENT, replacement)
            digest.update(line.encode("utf-8"))
    if declarations != 1:
        print(
            f"reconstruct: {path} carries {declarations} `const {IDENT}` declarations, expected "
            "exactly 1 — refusing rather than comparing an artifact whose label is ambiguous",
            file=sys.stderr,
        )
        sys.exit(2)
    assert literal is not None
    return digest.hexdigest(), literal, substituted


def main() -> int:
    ap = argparse.ArgumentParser(description="ES31 label-hoist reconstruction identity")
    ap.add_argument("--arm1", required=True, help="the UN-hoisted artifact (per-site literals)")
    ap.add_argument("--arm2", required=True, help="the HOISTED artifact (one module constant)")
    ap.add_argument(
        "--wrong-literal",
        action="store_true",
        help="RED control: reconstruct with a deliberately wrong literal; the comparison MUST fail",
    )
    args = ap.parse_args()

    override = '"../generated/NOT_THE_REAL_PATH.rs"' if args.wrong_literal else None
    got, literal, substituted = reconstruct(args.arm2, override)
    want = sha256_file(args.arm1)

    if substituted == 0:
        print(
            f"reconstruct: {args.arm2} declares {literal} and USES it 0 times — a hoist that hoists "
            "nothing would reconstruct trivially, so this is a refusal",
            file=sys.stderr,
        )
        return 2

    if got == want:
        print(f"identical  literal={literal}  substituted_lines={substituted}  sha={want[:16]}…")
        return 0
    print(
        f"DIFFERS    literal={literal}  substituted_lines={substituted}\n"
        f"           arm1={want}\n           recon={got}",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
