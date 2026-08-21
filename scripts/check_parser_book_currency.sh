#!/usr/bin/env bash
# PARSER-BOOK-CURRENCY — a per-parser BOOK must publish the release its family actually ships.
#
# Provenance: SV-CORPUS-GRAD.13c.2z, 2026-08-21. The director's standing instruction is that the
# book is "my only window into the project — I review the book, not the code", and that it must
# reflect what the codebase does with NO drift. Measured that day, the SystemVerilog book's
# changelog index published `1.0.183` while the family shipped `1.0.192`, and its schema table
# published `20` while the contract carried `25` — nine releases and five schemas invisible on the
# one surface the director reads.
#
# ⛔⛔ WHY A CHECK AND NOT ANOTHER HAND-FIX. This exact drift had already happened once: between
# `1.0.167` and `1.0.179` the same table read "is now 16" while the contract carried `19`, skipping
# schemas 17-19. `SV-CORPUS-GRAD.3.18` reconstructed the rows by hand and wrote a warning into the
# file naming the contract as the winner. That bought exactly ONE cycle -- the gap recurred and grew
# from three releases to nine, and the warning sentence itself had drifted a third time (reading
# "is now 21" against `26`) by the time this script was written. A prose warning is not an
# instrument.
#
# ⛔ WHY NOTHING ELSE CAUGHT IT. Three currency doctrines already run and all three pass here:
# PUBLISHED-VERSION-CURRENCY holds the *regex* identity pair and the per-family *status* row equal
# to their sources; LIVE-DOC-CURRENCY classifies surfaces by date spread against a declared charter;
# SV-CONTRACT-CURRENCY holds the *contract* current with the *grammar*. The family BOOK sits in the
# seam between all three.
#
# THE CHECK, in both directions:
#   for every per-parser book, the NEWEST release published in its changelog surface must EQUAL the
#   `Parser release version` in that family's integration contract. A book BEHIND its contract is
#   the rot this exists to stop; a book AHEAD of it is a release that was documented but never
#   recorded as shipped, which is the same defect wearing the other sign.
#
# ⛔⛔ THE POPULATION IS CLOSED, AND THAT IS DELIBERATE. `.13c.2z` measured SystemVerilog only and
# its own acceptance said this must not be designed from a sample of one -- so every one of the ten
# per-parser books is CLASSIFIED here, and a book matching no class is a HARD FAILURE rather than a
# silent skip. Sizing the population first is what found the other two: `vhdl` was stale by a
# release, and `systemverilog_preprocessor` uses a different heading convention that a
# SystemVerilog-shaped regex would have reported as an empty book.
#
# Exit: 0 = every book current; 1 = drift or an unclassified book.
set -uo pipefail
cd "$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"

PYBIN="${PYTHON:-python3}"
"$PYBIN" - "$@" <<'PY'
import glob, os, re, sys

SELFTEST = "--self-test" in sys.argv

# ── the CLOSED population, measured 2026-08-21 ────────────────────────────────
# `release_sections`  — the book publishes a `### <version>` section per release; compare the max.
# `date_keyed`        — the book's shape-change log is keyed by DATE, not by release number
#                       (the annotation families never adopted per-release sections). Nothing to
#                       compare; recorded so the book is not silently unwatched.
# `no_contract`       — the family ships no integration contract, so there is no release to be
#                       current WITH.
CLASSES = {
    "systemverilog":              ("release_sections", "changelog-index.md"),
    "vhdl":                       ("release_sections", "changelog-index.md"),
    "regex":                      ("release_sections", "changelog-index.md"),
    "rtl_const_expr":             ("release_sections", "changelog-index.md"),
    "rtl_frontend":               ("release_sections", "changelog-index.md"),
    "systemverilog_preprocessor": ("release_sections", "changelog-index.md"),
    "return_annotation":          ("date_keyed",       "schema-and-versioning.md"),
    "semantic_annotation":        ("date_keyed",       "schema-and-versioning.md"),
    "ebnf":                       ("no_contract",      None),
    "json":                       ("no_contract",      None),
}

# Two heading conventions are in the tree and BOTH are legitimate; a checker that knew only the
# SystemVerilog one would read `systemverilog_preprocessor` as an empty book and pass it.
VER_RE = re.compile(r'^###\s+(?:Release\s+)?(\d+\.\d+\.\d+)\b', re.M)
CONTRACT_RE = re.compile(r'- Parser release version:\s*\n\s*- `([0-9]+\.[0-9]+\.[0-9]+)`')

def vkey(v): return tuple(int(x) for x in v.split('.'))

def book_newest(path):
    txt = open(path, encoding='utf-8').read()
    vs = VER_RE.findall(txt)
    return max(vs, key=vkey) if vs else None

def contract_release(fam):
    p = f'docs/contracts/PGEN_{fam.upper()}_PARSER_INTEGRATION_CONTRACT.md'
    if not os.path.exists(p): return None, p
    m = CONTRACT_RE.search(open(p, encoding='utf-8').read())
    return (m.group(1) if m else None), p

def run():
    fails, notes = [], []
    books = sorted(glob.glob('docs/*_parser_book'))
    fams = [os.path.basename(b)[:-len('_parser_book')] for b in books]

    # closed-population check, both directions
    for fam in fams:
        if fam not in CLASSES:
            fails.append(f"book `docs/{fam}_parser_book` is NOT CLASSIFIED in this checker's "
                         f"population. Add it to CLASSES with a measured class — an unclassified "
                         f"book is indistinguishable from an unwatched one.")
    for fam in CLASSES:
        if fam not in fams:
            fails.append(f"`{fam}` is classified here but has no `docs/{fam}_parser_book` — the "
                         f"population and the tree disagree; fix whichever is wrong.")

    for fam in fams:
        if fam not in CLASSES: continue
        kind, surface = CLASSES[fam]
        if kind == "no_contract":
            _, cp = contract_release(fam)
            if os.path.exists(cp):
                fails.append(f"`{fam}` is classified `no_contract` but `{cp}` now EXISTS — it has "
                             f"gained a contract and must be reclassified `release_sections`.")
            else:
                notes.append(f"  · {fam:28s} no contract — nothing to be current with (declared)")
            continue

        crel, cp = contract_release(fam)

        if kind == "date_keyed":
            # ⛔ This class is CHECKED, not merely declared. These families carry NO
            # `Parser release version` in their contract at all — they are versioned by grammar
            # version, and their contracts say so ("not a versioned-consumer-impacting evolution").
            # So the class asserts BOTH halves: the book must still be date-keyed, AND the contract
            # must still carry no release. If either changes, the family has moved to the
            # release-numbered model and must be reclassified so it is actually compared.
            sp = f'docs/{fam}_parser_book/src/{surface}'
            if not os.path.exists(sp):
                fails.append(f"`{fam}`: declared surface `{sp}` is missing.")
            elif VER_RE.search(open(sp, encoding='utf-8').read()):
                fails.append(f"`{fam}` is classified `date_keyed` but `{sp}` now publishes "
                             f"`### <version>` sections — reclassify it `release_sections` so it "
                             f"is actually compared against its contract.")
            elif crel is not None:
                fails.append(f"`{fam}` is classified `date_keyed` but `{cp}` now carries a "
                             f"`Parser release version` (`{crel}`) — the family has adopted the "
                             f"release-numbered model and must be reclassified `release_sections`.")
            else:
                notes.append(f"  · {fam:28s} date-keyed shape log, contract carries no release "
                             f"(both halves checked)")
            continue

        if crel is None:
            fails.append(f"`{fam}`: could not read `Parser release version` from `{cp}`.")
            continue

        sp = f'docs/{fam}_parser_book/src/{surface}'
        if not os.path.exists(sp):
            fails.append(f"`{fam}`: declared changelog surface `{sp}` is missing.")
            continue
        bnew = book_newest(sp)
        if bnew is None:
            fails.append(f"`{fam}`: `{sp}` publishes NO release section this checker can read. "
                         f"Contract says `{crel}`.")
        elif bnew != crel:
            direction = "BEHIND" if vkey(bnew) < vkey(crel) else "AHEAD OF"
            fails.append(f"`{fam}`: the book is {direction} the contract — `{sp}` newest published "
                         f"release is `{bnew}`, `{cp}` says `{crel}`. "
                         + ("Publish the missing release section(s) in the book, authored from the "
                            "contract and the bug ledger."
                            if direction == "BEHIND" else
                            "Either the release shipped and the contract was not updated, or the "
                            "book documents a release that does not exist."))
        else:
            notes.append(f"  ✓ {fam:28s} book {bnew} == contract {crel}")
    return fails, notes

if SELFTEST:
    # Four controls, each proving one refusal actually fires. They mutate in memory only.
    import copy
    ok = True
    def probe(name, mutate, expect):
        global CLASSES
        saved = copy.deepcopy(CLASSES)
        mutate()
        f, _ = run()
        hit = any(expect in x for x in f)
        CLASSES = saved
        print(f"  {'PASS' if hit else 'FAIL'}  {name}")
        return hit
    ok &= probe("an UNCLASSIFIED book is refused",
                lambda: CLASSES.pop("json"), "NOT CLASSIFIED")
    ok &= probe("a classified book that no longer exists is refused",
                lambda: CLASSES.update({"ghost_family": ("release_sections", "changelog-index.md")}),
                "has no `docs/ghost_family_parser_book`")
    ok &= probe("a `no_contract` family that GAINED a contract is refused",
                lambda: CLASSES.update({"vhdl": ("no_contract", None)}),
                "has gained a contract")
    ok &= probe("a `date_keyed` book that started publishing releases is refused",
                lambda: CLASSES.update({"regex": ("date_keyed", "changelog-index.md")}),
                "is classified `date_keyed` but")

    # ⭐⭐ The two controls that matter most: the DEFECT this doctrine was written for, in both
    # directions. The four above prove the population is closed; these prove the comparison
    # itself can go RED. A gate whose primary assertion has never been seen to fail is a gate
    # nobody has tested — and this doctrine exists precisely because a book sat nine releases
    # stale with every other check green.
    _real = contract_release
    def _shift(fam, delta):
        def patched(f):
            v, cp = _real(f)
            if f == fam and v is not None:
                parts = [int(x) for x in v.split('.')]
                parts[-1] += delta
                return '.'.join(str(x) for x in parts), cp
            return v, cp
        return patched

    def probe_fn(name, fam, delta, expect):
        global contract_release
        saved = contract_release
        contract_release = _shift(fam, delta)
        f, _ = run()
        hit = any(expect in x for x in f)
        contract_release = saved
        print(f"  {'PASS' if hit else 'FAIL'}  {name}")
        return hit

    ok &= probe_fn("a book BEHIND its contract is refused (the founding defect)",
                   "systemverilog", +1, "is BEHIND the contract")
    ok &= probe_fn("a book AHEAD OF its contract is refused (the other sign)",
                   "systemverilog", -1, "is AHEAD OF the contract")

    print("parser-book-currency: SELF-TEST " + ("6/6 OK" if ok else "FAILED"))
    sys.exit(0 if ok else 1)

fails, notes = run()
print("PARSER-BOOK-CURRENCY: %d book(s) in the closed population" % len(CLASSES))
for n in notes: print(n)
if fails:
    print("\nparser-book-currency: %d drift(s):" % len(fails))
    for f in fails: print("  ✗ " + f)
    print("\n⛔ The book is the director's review surface. A book behind its contract is the rot")
    print("   this doctrine exists to stop; a book ahead of it is a release documented but never")
    print("   recorded as shipped. Fix the SURFACE, never this checker's expectation.")
    sys.exit(1)
print("parser-book-currency: OK (every book publishes the release its family ships)")
PY
