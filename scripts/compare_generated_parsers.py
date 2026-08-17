#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.25 (c) — the ONE home for comparing two generated parsers.

WHY THIS FILE EXISTS — AND WHY IT IS NOW A TRIPWIRE
---------------------------------------------------
Every parser PGEN generated used to write its own `-o` destination into the emitted source, so *the
size of a generated parser was a function of its own output path* and one extra character in the
`-o` spelling added one byte per embedded site. That trap inverted **three** published readings in
this repository, one of which founded a task leaf on two hypotheses that were both wrong
(TOOLBOX 5.6, `ENGINE-UNIVERSAL-SERVICES.25`). Three independent copies of "normalise the path
first" then grew — the second written *after* the first defect was recorded, which is the evidence
that prose does not transfer. This module is that logic's single home.

⭐⭐ **THE TRAP IS NOW ELIMINATED, NOT REDUCED, AND THIS MODULE'S JOB CHANGED WITH IT.** Three slices
of `ENGINE-UNIVERSAL-SERVICES.31` took the embedded-site count **63 186 → 60 482 → 11 → 0**:

    slice 1  (a)  the dead `let filename_str` binding, read by nothing, deleted      63 186 → 60 482
    slice 2  (b)(c)(d)  the surviving literals hoisted to ONE module constant        60 482 → 11
    slice 3  (e)  that constant's VALUE corrected — the path was the WRONG LABEL,
                  naming the generated parser beside an INPUT byte offset — so it
                  now reads `"<grammar> input byte"` and the path is gone entirely        11 → 0

⇒ **no generated parser embeds its output path any more.** `--sites` reports `0` and `--spelling`
reports `(none)`; both are the CORRECT answer, not a failure. `--compare` still works and is now
*stronger*: with nothing to normalise it is a raw byte comparison.

⛔ **The module is kept, deliberately, as a RE-INTRODUCTION TRIPWIRE** — the shape
`LIVE-DOC-CURRENCY`'s dormant instrument B uses. `GENERATED-REPRODUCIBILITY` calls `--sites` on
every artifact it verifies and BREACHES on any non-zero, so an emitter that starts writing its
output path back into the artifact is caught the next time that gate runs. A deleted module could
not do that, and the property it guards is one this repository has already paid for four times.

THE DETECTION RULE IS DERIVED FROM THE PRODUCER, NOT FROM THE CALLER
--------------------------------------------------------------------
⭐ The caller never tells this module which path to normalise. It is read out of the artifact:
every generated parser contains **exactly one** distinct string literal ending in `.rs`, and its
occurrence count equals the embedded-site count exactly. That property is what makes the derivation
possible, and it is unchanged by the hoist — only the count moved. Measured, per era:

    pre-slice-1   json 217 · regex 11 647 · return_annotation 675 · rtl_const_expr 605
                  rtl_frontend 2 508 · scratch 71 · semantic_annotation 3 758
                  systemverilog 36 346 · svpp 895 · vhdl 3 075 · ebnf 3 389
                                                                  (total 63 186, 11/11 exact)
    post-slice-1  the same minus the dead class-D bindings         (total 60 482)
    post-slice-2  1 per artifact, the `const PGEN_SOURCE_LABEL` declaration
                                                                  (total 11, 11/11 exact)
    post-slice-3  NONE — the constant holds `"<grammar> input byte"`      (total 0, 11/11 exact)

⛔⛔ Deriving it matters, and the failure it prevents is measured. The `-o` spellings in play are
`../generated/systemverilog_parser.rs` (36 chars, what `rust/Makefile` passes from `rust/`) and
`generated/systemverilog_parser.rs` (33 chars, what an ad-hoc run from the repo root passes) — and
**the short spelling is a SUBSTRING of the long one**. A helper handed the short spelling and run
over a long-spelling artifact rewrites `"../generated/x.rs"` into `"../<TOKEN>"`: measured on the
then-shipped SystemVerilog parser (pre-slice-2, when the count was per site), that leaves **36 346
`"../<TOKEN>"` residues and 0 normalised sites**, i.e. a "normalisation" that normalised nothing
while reporting success. Two arms normalised that way with their OWN spellings still differ by 3
bytes per site — which reads as *"something other than the path moved"*, the exact false verdict
`.19` was founded on. ⚠️ The substring hazard is a property of the two SPELLINGS, not of the site
count, so it survives the hoist unchanged; only its blast radius shrank from 36 346 sites to 1.

HONEST BOUNDS (stated here, before the module is trusted)
---------------------------------------------------------
- Normalised bytes are a COMPARISON BASIS, not the artifact's size. Both sides are rewritten to one
  fixed token, so `--normalise` output is only ever meaningful against another `--normalise` output
  produced by this same module.
- It REFUSES (exit 2) rather than guesses whenever the artifact carries MORE THAN ONE distinct
  `.rs` literal. A guess here is exactly how a wrong reading gets published. ⚠️ ZERO used to refuse
  too and no longer does — zero is the correct state since `.31` (e), and the old refusal message
  said so in advance: *"the emitter stopped embedding its `-o` path (in which case this module is
  obsolete and should be retired deliberately, not bypassed)"*. Retired deliberately, into the
  tripwire role above.
- Equal normalised sha256 proves the `-o` path was the ONLY difference. It does NOT prove the two
  arms were generated from the same grammar — two arms with different rule counts normalise to
  different bytes, which is a real difference and is reported as one.
- Site counts are OCCURRENCES, not lines-with-a-match. `grep -c` counts lines and would undercount
  two sites on one line; the literals are one-per-line today, but that is a measurement, not a
  property to rely on.

USAGE
-----
    scripts/compare_generated_parsers.py --spelling FILE        # the derived embedded -o spelling
    scripts/compare_generated_parsers.py --sites FILE           # the derived embedded-site count
    scripts/compare_generated_parsers.py --normalise FILE [-o OUT]
    scripts/compare_generated_parsers.py --compare A B [--json]
    scripts/compare_generated_parsers.py --self-test

EXIT CODES
----------
    0  ok / the two artifacts are identical once normalised
    1  the two artifacts DIFFER by something other than the `-o` path
    2  REFUSED — the question could not be answered (ambiguous, absent or unreadable input)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

# The token both sides are rewritten to. Fixed and path-shaped, so a normalised artifact still
# reads as a path at a glance and can never collide with a real `-o` spelling.
NORMALISED_TOKEN = "<PGEN-GENERATED-OUT>"

# A generated parser embeds its `-o` destination as a plain double-quoted Rust string literal that
# ends in `.rs`. Rust string escapes cannot occur inside a path PGEN itself wrote, so a
# non-greedy no-escape scan is exact here rather than approximate.
_RS_LITERAL = re.compile(r'"([^"\n\\]*\.rs)"')


class Refusal(Exception):
    """Raised when the question cannot be answered. Always surfaces as exit 2, never as a verdict."""


def read_artifact(path: Path) -> str:
    if not path.is_file():
        raise Refusal(f"{path} is not a readable file")
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError as exc:  # a generated parser PGEN wrote is always UTF-8
        raise Refusal(f"{path} is not valid UTF-8 ({exc}) — this is not a generated parser") from exc


NO_EMBEDDING = "(none)"


def derive_spelling_or_none(src: str, where: str = "<artifact>") -> str | None:
    """The embedded `-o` spelling, read out of the artifact itself — or `None` if there is none.

    ⭐⭐ `None` IS THE CORRECT STATE SINCE `ENGINE-UNIVERSAL-SERVICES.31` (e), AND THIS FUNCTION'S
    OWN PREDECESSOR SAID SO. It used to REFUSE on zero candidates, with the message *"the emitter
    stopped embedding its `-o` path (in which case this module is obsolete and should be retired
    deliberately, not bypassed)"*. That is exactly what happened: fixing the emitted diagnostic
    label — it named the generated parser's path beside an INPUT byte offset — removed the last
    consumer of the path, and no generated parser embeds it any more. Retired deliberately, here.

    ⛔ It still REFUSES on MORE than one candidate. Ambiguity is a different question from absence:
    picking the most frequent literal would be a guess, and removing guesses is why this module
    exists.
    """
    distinct = set(_RS_LITERAL.findall(src))
    if not distinct:
        return None
    if len(distinct) > 1:
        listed = ", ".join(sorted(distinct)[:5])
        raise Refusal(
            f"{where}: {len(distinct)} distinct `\"….rs\"` literals ({listed}) — ambiguous. "
            f"Refusing rather than guessing which one is the `-o` path."
        )
    return distinct.pop()


def derive_spelling(src: str, where: str = "<artifact>") -> str:
    """[`derive_spelling_or_none`], refusing on absence.

    For the operations that are meaningless without an embedded path (`--normalise` alone). The
    comparison path deliberately does NOT use this: with no embedding, normalisation is the
    identity and a raw byte comparison is the stronger check, not a degraded one.
    """
    spelling = derive_spelling_or_none(src, where)
    if spelling is None:
        raise Refusal(
            f"{where}: no `\"….rs\"` string literal to normalise. Since "
            f"`ENGINE-UNIVERSAL-SERVICES.31` (e) that is the CORRECT state for a generated parser "
            f"— use `--compare`, which needs no normalisation when there is nothing to normalise."
        )
    return spelling


def site_count(src: str, spelling: str) -> int:
    """OCCURRENCES of the embedded spelling (not lines carrying it)."""
    return src.count(spelling)


def normalise(src: str, spelling: str, token: str = NORMALISED_TOKEN) -> str:
    return src.replace(spelling, token)


def describe(path: Path, token: str = NORMALISED_TOKEN) -> dict:
    src = read_artifact(path)
    # ⭐ Absence is the CORRECT state since `.31` (e); normalisation is then the identity and the
    # comparison below becomes a RAW byte comparison, which is strictly stronger than a normalised
    # one. `sites: 0` is the property to assert, not a failure to report.
    spelling = derive_spelling_or_none(src, str(path))
    norm = src if spelling is None else normalise(src, spelling, token)
    # ⛔ CHARACTERS AND BYTES ARE NOT THE SAME NUMBER HERE, AND BOTH ARE PUBLISHED SOMEWHERE. A
    # generated parser carries emoji in its trace strings: `generated/systemverilog_parser.rs` is
    # 143 909 594 bytes and 143 801 151 characters — a 108 443 gap, i.e. larger than the whole dead
    # `let filename_str` population. `.20` slice 3's three-arm table was published in CHARACTERS
    # (Python `len(str)`); `wc -c` and `stat` report BYTES. Reporting both, named, is the only way a
    # later reader can tell which basis a figure sits on.
    return {
        "path": str(path),
        "spelling": NO_EMBEDDING if spelling is None else spelling,
        "spelling_chars": 0 if spelling is None else len(spelling),
        "sites": 0 if spelling is None else site_count(src, spelling),
        "raw_chars": len(src),
        "raw_bytes": len(src.encode("utf-8")),
        "normalised_chars": len(norm),
        "normalised_bytes": len(norm.encode("utf-8")),
        "raw_sha256": hashlib.sha256(src.encode("utf-8")).hexdigest(),
        "normalised_sha256": hashlib.sha256(norm.encode("utf-8")).hexdigest(),
    }


def compare(a: Path, b: Path, token: str = NORMALISED_TOKEN) -> tuple[int, dict]:
    da, db = describe(a, token), describe(b, token)
    identical = da["normalised_sha256"] == db["normalised_sha256"]
    verdict = {
        "identical_once_normalised": identical,
        "a": da,
        "b": db,
        "raw_byte_delta": db["raw_bytes"] - da["raw_bytes"],
        "normalised_byte_delta": db["normalised_bytes"] - da["normalised_bytes"],
        "path_attributable_bytes": (
            db["sites"] * db["spelling_chars"] - da["sites"] * da["spelling_chars"]
        ),
    }
    return (0 if identical else 1), verdict


def render(verdict: dict) -> str:
    a, b = verdict["a"], verdict["b"]
    lines = [
        "GENERATED-PARSER-COMPARE",
        f"  A  {a['path']}",
        f"     spelling={a['spelling']!r} ({a['spelling_chars']} chars) sites={a['sites']}",
        f"     raw={a['raw_bytes']} normalised={a['normalised_bytes']} "
        f"norm_sha={a['normalised_sha256'][:16]}…",
        f"  B  {b['path']}",
        f"     spelling={b['spelling']!r} ({b['spelling_chars']} chars) sites={b['sites']}",
        f"     raw={b['raw_bytes']} normalised={b['normalised_bytes']} "
        f"norm_sha={b['normalised_sha256'][:16]}…",
        f"  raw delta        {verdict['raw_byte_delta']:+d} bytes  "
        f"(⛔ NOT a source comparison — includes the embedded `-o` path)",
        f"  path-attributable{verdict['path_attributable_bytes']:+d} bytes",
        f"  normalised delta {verdict['normalised_byte_delta']:+d} bytes  ← the real difference",
    ]
    lines.append(
        "  ✓ IDENTICAL once normalised — the `-o` path was the ONLY difference"
        if verdict["identical_once_normalised"]
        else "  ✗ DIFFER once normalised — something other than the `-o` path moved"
    )
    return "\n".join(lines)


# ──────────────────────────────────────────────────────────────────────────────────────────────────
# Self-test. Every arm is fired, including the ones that must go RED and the ones that must REFUSE:
# a control never observed failing is not known to work (`docs/CLAIM_VERIFICATION.md` §3 leg 2).
# ──────────────────────────────────────────────────────────────────────────────────────────────────

_LONG = "../generated/systemverilog_parser.rs"
_SHORT = "generated/systemverilog_parser.rs"


def _artifact(spelling: str, sites: int = 4, body: str = "") -> str:
    parts = [f'    let filename_str = "{spelling}";\n' for _ in range(sites)]
    return f"// synthetic\n{body}" + "".join(parts)


def self_test() -> int:
    arms: list[tuple[str, bool, str]] = []

    def arm(name: str, ok: bool, detail: str = "") -> None:
        arms.append((name, ok, detail))

    # 1 GREEN — the spelling is derived, and the count is exact.
    src = _artifact(_LONG, sites=7)
    arm(
        "derives the embedded spelling and counts its sites",
        derive_spelling(src) == _LONG and site_count(src, _LONG) == 7,
    )

    # 2 GREEN — two arms written through DIFFERENT spellings normalise to the same bytes.
    a, b = _artifact(_LONG, 5), _artifact(_SHORT, 5)
    arm(
        "two spellings of one artifact are identical once normalised",
        normalise(a, derive_spelling(a)) == normalise(b, derive_spelling(b)),
    )

    # 3 RED — a genuine content difference must SURVIVE normalisation.
    c = _artifact(_SHORT, 5, body="// a real difference\n")
    arm(
        "RED: a real content difference is NOT masked by normalisation",
        normalise(a, derive_spelling(a)) != normalise(c, derive_spelling(c)),
    )

    # 4 RED — THE MEASURED SUBSTRING TRAP. Normalising a long-spelling artifact with the SHORT
    #   spelling (what a caller-supplied path does) leaves `../<TOKEN>` residues and zero clean
    #   sites; deriving the spelling instead gets it right. This arm is the whole reason the
    #   caller is not allowed to supply the path.
    naive = a.replace(_SHORT, NORMALISED_TOKEN)
    derived = normalise(a, derive_spelling(a))
    arm(
        "RED: the caller-supplied SHORT spelling mis-normalises a LONG-spelling artifact",
        naive.count(f'"../{NORMALISED_TOKEN}"') == 5
        and naive.count(f'"{NORMALISED_TOKEN}"') == 0
        and derived.count(f'"{NORMALISED_TOKEN}"') == 5,
        f"naive left {naive.count(f'../{NORMALISED_TOKEN}')} residues; derived left 0",
    )

    # 5 REFUSE — no `.rs` literal at all.
    try:
        derive_spelling("fn main() {}", "<no-literal>")
        arm("REFUSE: an artifact with no `.rs` literal", False, "it returned instead of refusing")
    except Refusal:
        arm("REFUSE: an artifact with no `.rs` literal", True)

    # 6 REFUSE — ambiguous: two distinct `.rs` literals.
    try:
        derive_spelling(_artifact(_LONG, 2) + _artifact("../generated/vhdl_parser.rs", 2), "<ambig>")
        arm("REFUSE: an ambiguous artifact (2 distinct literals)", False, "it guessed")
    except Refusal:
        arm("REFUSE: an ambiguous artifact (2 distinct literals)", True)

    # 7 GREEN — path-attributable arithmetic reproduces the raw gap when content is equal.
    raw_gap = len(a.encode()) - len(b.encode())
    predicted = 5 * (len(_LONG) - len(_SHORT))
    arm(
        "the raw byte gap equals sites × Δchars when only the spelling differs",
        raw_gap == predicted,
        f"raw_gap={raw_gap} predicted={predicted}",
    )

    # 8 RED — a WRONG normalisation must not match (the es19 arm-5 control, kept in the shared home).
    wrong = a.replace(_LONG, "x" + NORMALISED_TOKEN)
    arm(
        "RED: a deliberately wrong normalisation does not match",
        wrong != normalise(b, derive_spelling(b)),
    )

    width = max(len(n) for n, _, _ in arms)
    failed = 0
    print("GENERATED-PARSER-COMPARE SELF-TEST")
    for name, ok, detail in arms:
        if not ok:
            failed += 1
        suffix = f"   [{detail}]" if detail and not ok else ""
        print(f"  {'✓' if ok else '✗'} {name:<{width}}{suffix}")
    total = len(arms)
    print(
        f"GENERATED-PARSER-COMPARE: {total - failed}/{total} arms as declared"
        + ("" if failed == 0 else f" — {failed} FAILED")
    )
    return 1 if failed else 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(
        prog="compare_generated_parsers.py",
        description="Compare two generated parsers with the embedded `-o` path normalised away.",
    )
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--spelling", metavar="FILE", help="print the derived embedded `-o` spelling")
    g.add_argument("--sites", metavar="FILE", help="print the derived embedded-site count")
    g.add_argument("--normalise", metavar="FILE", help="write the artifact with the path normalised")
    g.add_argument("--describe", metavar="FILE", help="every derived figure for one artifact, as JSON")
    g.add_argument("--compare", nargs=2, metavar=("A", "B"), help="compare two artifacts")
    g.add_argument("--self-test", action="store_true", help="fire every arm, including the RED ones")
    p.add_argument("-o", "--output", metavar="FILE", help="destination for --normalise (default stdout)")
    p.add_argument("--json", action="store_true", help="machine-readable output for --compare")
    # ⛔ The token changes every NORMALISED BYTE COUNT by `sites × Δlen`, so a caller comparing
    # against an already-published normalised figure MUST pass the token that figure was produced
    # with. `ENGINE-UNIVERSAL-SERVICES.20` slice 3 published its three-arm table on `<OUT>`; its
    # runner therefore asks for `<OUT>` rather than silently re-basing numbers already in the record.
    # It never affects a COMPARISON verdict — both sides use one token — only the byte figures.
    p.add_argument(
        "--token",
        default=NORMALISED_TOKEN,
        metavar="STR",
        help=f"replacement token (default {NORMALISED_TOKEN!r}); changes normalised BYTE COUNTS only",
    )
    args = p.parse_args(argv)

    try:
        if args.self_test:
            return self_test()

        if args.spelling:
            found = derive_spelling_or_none(read_artifact(Path(args.spelling)), args.spelling)
            print(NO_EMBEDDING if found is None else found)
            return 0

        if args.sites:
            path = Path(args.sites)
            src = read_artifact(path)
            found = derive_spelling_or_none(src, args.sites)
            print(0 if found is None else site_count(src, found))
            return 0

        if args.describe:
            print(json.dumps(describe(Path(args.describe), args.token), indent=2))
            return 0

        if args.normalise:
            path = Path(args.normalise)
            src = read_artifact(path)
            out = normalise(src, derive_spelling(src, args.normalise), args.token)
            if args.output:
                Path(args.output).write_text(out, encoding="utf-8")
            else:
                sys.stdout.write(out)
            return 0

        rc, verdict = compare(Path(args.compare[0]), Path(args.compare[1]), args.token)
        print(json.dumps(verdict, indent=2) if args.json else render(verdict))
        return rc

    except Refusal as exc:
        print(f"compare_generated_parsers: REFUSED — {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
