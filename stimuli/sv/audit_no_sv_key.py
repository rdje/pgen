#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.13e` — does iverilog offer a SECONDARY KEY for the `no_sv_key` rows?

`.13a` stratified `deferred:no_sv_key` (743 rows) and found that only its DARK members can hide a
defect: the rest were fully consumed by the parser standing alone. This leaf owes an answer for
the DARK ones — *"whether iverilog offers any secondary key (a companion golden log, a Makefile
target) — and if not, an honest-permanent deferral naming the 78 rather than the 743."*

⭐⭐ THE ANSWER IS YES, AND THE LARGEST SUB-CLASS IS NOT A MISSING KEY AT ALL — IT IS A KEY THE
ADJUDICATOR DECLINED TO READ. Those rows carry the basis *"vvp_tests descriptor(s) without an
explicit generation flag - dialect unresolved (the upstream default generation is not encoded in
the descriptor)"*. That sentence is true about the DESCRIPTOR and false about the QUESTION: the
default generation is compiled into `iverilog` itself, and `iverilog` is vendored in this very
corpus. `compiler.h` declares `GN_DEFAULT = 4` in an enum whose `GN_VER2005 = 4`, and `main.cc`
initialises `generation_flag = GN_DEFAULT`. ⇒ a descriptor with no `-g` flag runs under plain
IEEE 1364-2005 Verilog, so the row belongs to the **verilog_2005 profile lane** (ROUTED — answered
in the v2005 manifest), never to `no_sv_key` (NO VERDICT — answered nowhere).

⛔ THIS IS NOT THE RELABELLING `.13` FORBIDS. That warning is about pinning a deferred row to an
expectation and calling it ADJUDICATED. Nothing here becomes adjudicated. A row moves from one
deferral to a *weaker and checkable* one, and it moves because the stated ground for the original
classification — "the dialect cannot be determined" — is refuted by measurement.

WHAT THIS SCRIPT DOES, and it refuses rather than guessing at every step:

  1. Re-derives `GN_DEFAULT` from the vendored compiler source, and REFUSES if the enum cannot be
     read or resolves to a generation this script does not know.
  2. Re-derives the ONE thing that could override it — `vvp_reg.py`'s `force_gen()`, which forces
     `-g2023` — and REFUSES if `--force-sv` is no longer default-off or if the vendored Makefile
     target has started passing it. Both would invert the verdict.
  3. For every `deferred:no_sv_key` row, reports which of the ivtest list files names it, so the
     52 rows whose basis is *"no regress-sv.list entry"* are answered by enumeration rather than
     by the absence of one list.
  4. Prints a per-class disposition and the row counts a follow-up slice would move.

  python3 stimuli/sv/audit_no_sv_key.py
"""
from __future__ import annotations

import re
import sys
from collections import Counter, defaultdict
from pathlib import Path


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "CLAUDE.md").is_file() and (parent / "grammars").is_dir():
            return parent
    raise SystemExit(f"REFUSE: no repository root above {here}")


ROOT = repo_root()
IVL = ROOT / "stimuli/sv/subs/iverilog"
IVTEST = IVL / "ivtest"
MANIFEST = ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv"

# The basis strings the adjudicator writes for this class. Matched by prefix so a reworded tail
# does not silently drop a row into "unclassified" — but an entirely new basis MUST refuse.
BASIS_VVP_NO_FLAG = "ivtest: vvp_tests descriptor(s) without an explicit generation flag"
BASIS_NO_LIST_ENTRY = "ivtest: no regress-sv.list entry"


def refuse(msg: str) -> None:
    raise SystemExit(f"REFUSE: {msg}")


# --------------------------------------------------------------------------------------------
# 1. The compiler's own default generation — the fact the adjudicator called unknowable
# --------------------------------------------------------------------------------------------
def resolve_default_generation() -> tuple[str, str]:
    """Return (enum_name, evidence). REFUSES rather than assuming."""
    header = IVL / "compiler.h"
    main_cc = IVL / "main.cc"
    if not header.is_file() or not main_cc.is_file():
        refuse(f"vendored iverilog source not found ({header} / {main_cc}) — the corpus moved")

    text = header.read_text(errors="replace")
    block = re.search(r"enum\s+generation_t\s*\{(.*?)\}", text, re.S)
    if not block:
        refuse(f"cannot find `enum generation_t` in {header.relative_to(ROOT)}")
    members = dict(
        (m.group(1), int(m.group(2)))
        for m in re.finditer(r"(GN_[A-Z0-9_]+)\s*=\s*(\d+)", block.group(1))
    )
    if "GN_DEFAULT" not in members:
        refuse("`GN_DEFAULT` is not a member of `enum generation_t` — upstream changed the model")
    value = members["GN_DEFAULT"]
    named = sorted(k for k, v in members.items() if v == value and k != "GN_DEFAULT")
    if len(named) != 1:
        refuse(
            f"GN_DEFAULT = {value} aliases {len(named)} generations ({named or 'none'}) — "
            "ambiguous, and this script will not pick one"
        )
    # The initialiser is the second half: a default nothing reads decides nothing.
    if not re.search(r"generation_flag\s*=\s*GN_DEFAULT\s*;", main_cc.read_text(errors="replace")):
        refuse(
            f"{main_cc.relative_to(ROOT)} no longer initialises `generation_flag = GN_DEFAULT` — "
            "the enum value may be inert"
        )
    return named[0], f"{header.relative_to(ROOT)}: GN_DEFAULT = {value} = {named[0]}"


def check_force_sv_is_off_by_default() -> str:
    """The ONE mechanism that would make an unflagged descriptor SystemVerilog."""
    runner = IVTEST / "vvp_reg.py"
    makefile = IVTEST / "Makefile.in"
    if not runner.is_file() or not makefile.is_file():
        refuse(f"vendored ivtest runner/Makefile not found ({runner} / {makefile})")

    rtext = runner.read_text(errors="replace")
    if "def force_gen(" not in rtext:
        refuse(f"{runner.relative_to(ROOT)} no longer defines force_gen() — re-read the runner")
    if not re.search(r"--force-sv['\"],\s*action=['\"]store_true", rtext):
        refuse(
            f"{runner.relative_to(ROOT)}: `--force-sv` is no longer an off-by-default store_true "
            "flag — an unflagged descriptor may now be forced to SystemVerilog, which INVERTS "
            "this audit's verdict"
        )
    if not re.search(r"if\s+cfg\[['\"]force-sv['\"]\]:\s*\n\s*force_gen\(", rtext):
        refuse(
            f"{runner.relative_to(ROOT)}: force_gen() is no longer guarded by cfg['force-sv'] — "
            "it may now run unconditionally, which INVERTS this audit's verdict"
        )

    mtext = makefile.read_text(errors="replace")
    invocations = [l for l in mtext.splitlines() if "vvp_reg.py" in l and "echo" not in l]
    if not invocations:
        refuse(f"{makefile.relative_to(ROOT)} no longer invokes vvp_reg.py — the harness moved")
    forced = [l for l in invocations if "--force-sv" in l]
    if forced:
        refuse(
            f"{makefile.relative_to(ROOT)} now passes --force-sv ({forced[0].strip()}) — the "
            "upstream default run IS SystemVerilog and this audit's verdict is INVERTED"
        )
    return (
        f"{runner.relative_to(ROOT)}: force_gen() forces -g2023 but only under cfg['force-sv']; "
        f"--force-sv is store_true (default off) and {makefile.relative_to(ROOT)}'s own target "
        "does not pass it"
    )


# --------------------------------------------------------------------------------------------
# 2. The other ivtest list files — the "secondary key" the leaf asked about
# --------------------------------------------------------------------------------------------
def read_lists() -> dict[str, dict[str, list[str]]]:
    out: dict[str, dict[str, list[str]]] = {}
    for path in sorted(IVTEST.glob("*.list")):
        rows: dict[str, list[str]] = {}
        for line in path.read_text(errors="replace").splitlines():
            s = line.split("#", 1)[0].strip()
            if not s:
                continue
            parts = s.split()
            # `vNN:name` selects a version; the bare name is the key either way.
            key = parts[0].split(":")[-1]
            rows.setdefault(key, parts)
        out[path.name] = rows
    if "regress-sv.list" not in out:
        refuse("regress-sv.list is missing from the vendored ivtest tree")
    return out


def main() -> int:
    if not MANIFEST.is_file():
        refuse(f"{MANIFEST.relative_to(ROOT)} not found — run the corpus adjudication first")

    gen_name, gen_evidence = resolve_default_generation()
    force_evidence = check_force_sv_is_off_by_default()
    lists = read_lists()

    rows = []
    with MANIFEST.open(errors="replace") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        idx = {name: i for i, name in enumerate(header)}
        for need in ("relpath", "observed", "adjudication", "basis"):
            if need not in idx:
                refuse(f"{MANIFEST.relative_to(ROOT)} has no `{need}` column — schema moved")
        for line in fh:
            f = line.rstrip("\n").split("\t")
            if len(f) <= idx["basis"]:
                continue
            if f[idx["adjudication"]] == "deferred:no_sv_key":
                rows.append(f)
    if not rows:
        refuse("no `deferred:no_sv_key` rows in the manifest — the class moved or the run is stale")

    print("SV-CORPUS-GRAD.13e — secondary keys for the `deferred:no_sv_key` population")
    print(f"manifest: {MANIFEST.relative_to(ROOT)}   rows in class: {len(rows)}")
    print()
    print("GROUND TRUTH, re-derived from the vendored tool rather than assumed:")
    print(f"  · {gen_evidence}")
    print(f"  · {force_evidence}")
    print(
        f"  ⇒ an ivtest run whose args carry no `-g` flag compiles under **{gen_name}**, "
        "not SystemVerilog."
    )
    print()

    by_basis: dict[str, list[list[str]]] = defaultdict(list)
    for f in rows:
        basis = f[idx["basis"]]
        if basis.startswith(BASIS_VVP_NO_FLAG):
            by_basis["vvp_no_flag"].append(f)
        elif basis.startswith(BASIS_NO_LIST_ENTRY):
            by_basis["no_list_entry"].append(f)
        else:
            by_basis[f"UNCLASSIFIED::{basis}"].append(f)

    unclassified = {k: v for k, v in by_basis.items() if k.startswith("UNCLASSIFIED::")}
    if unclassified:
        for k, v in unclassified.items():
            print(f"⛔ {len(v)} row(s) carry an unrecognised basis: {k[len('UNCLASSIFIED::'):]}")
        refuse(
            "the adjudicator writes a basis this audit has no bucket for — classify it rather "
            "than letting it fall into a total"
        )

    def dark(fs):
        return [f for f in fs if f[idx["observed"]] == "fail"]

    vvp, nolist = by_basis["vvp_no_flag"], by_basis["no_list_entry"]
    print("THE TWO BASES, AND THEY GET DIFFERENT ANSWERS")
    print()
    print(f"  A. `{BASIS_VVP_NO_FLAG} …`")
    print(f"     rows {len(vvp):4}   of which DARK (the parse FAILED) {len(dark(vvp)):4}")
    print(f"     ⇒ RESOLVED. The dialect is not unresolved — it is {gen_name}. These rows are")
    print("       `v2005_profile_lane` (ROUTED), not `no_sv_key` (NO VERDICT).")
    print()
    print(f"  B. `{BASIS_NO_LIST_ENTRY} …`")
    print(f"     rows {len(nolist):4}   of which DARK (the parse FAILED) {len(dark(nolist)):4}")
    print("     ⇒ enumerated below: which OTHER ivtest list names the row, if any.")
    print()

    print("B, ENUMERATED — the lists that name each DARK row (a row may appear in several)")
    print()
    combos: Counter = Counter()
    per_list: Counter = Counter()
    for f in dark(nolist):
        stem = Path(f[idx["relpath"]].replace("\\", "/")).stem
        naming = tuple(sorted(n for n, entries in lists.items() if stem in entries))
        combos[naming] += 1
        for n in naming:
            per_list[n] += 1
    print(f"  {'lists naming the row':<56} rows")
    print(f"  {'-'*56} ----")
    for naming, n in combos.most_common():
        label = ", ".join(naming) if naming else "(NONE — no ivtest list names it)"
        print(f"  {label:<56} {n:4}")
    print()
    keyed = sum(n for naming, n in combos.items() if naming)
    unkeyed = sum(n for naming, n in combos.items() if not naming)
    print(f"  ⇒ {keyed} of {keyed + unkeyed} DARK rows in basis B ARE named by another ivtest list;")
    print(f"    {unkeyed} are named by none and are the honest-permanent-deferral candidates.")
    print()

    print("⚠️ WHAT THIS AUDIT DOES NOT DO, stated rather than implied")
    print("  · It does NOT adjudicate what each secondary list IMPLIES for SV parse validity.")
    print("    That is per-list work and the lists disagree: `regress-fsv.list` is read ONLY under")
    print("    --force-sv (so it testifies about FORCED SystemVerilog), `regress-vlog95.list` is a")
    print("    Verilog-95 OUTPUT lane whose `CE` means the back end refused, not the parser, and")
    print("    `vhdl_regress.list` / `blif.list` / `vpi_regress.list` are other back ends again.")
    print("    Reading a `CE` out of one of those as `must_reject` would manufacture a defect.")
    print("  · It does NOT move any row. Reclassification touches the adjudicator, which can move")
    print("    the SV graduation bar with zero parser change (DOCTRINE-GAP-OWNERSHIP.6), so it is")
    print("    sequenced as its own slice with the corpus re-measure in the same commit.")
    print(f"  · The {gen_name} verdict is about the UPSTREAM DEFAULT INVOCATION. A user running")
    print("    `vvp_reg.py --force-sv` gets -g2023, which is why that flag is checked above and")
    print("    why this script refuses if its default ever changes.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
