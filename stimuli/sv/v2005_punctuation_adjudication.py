#!/usr/bin/env python3
"""Adjudicate the L4 punctuation candidates — `SV-CORPUS-GRAD.13e.10` (c).

`v2005_punctuation_faithfulness_census.py` derives the CANDIDATE set: reachable punctuation-only
terminals whose literal never appears in IEEE 1364-2005 Annex A's own BNF alphabet.  A candidate is
not a defect.  This instrument runs the probe legs that turn a candidate into a verdict.

THE FOUR LEGS
-------------
    leg 1  the witness ACCEPTS under `--profile verilog_2005`      (else the profile already refuses it)
    leg 2  the same witness ACCEPTS under `--profile sv_2017`      (else the witness is simply malformed)
    leg 3  the operator's OWN TERMINAL RULE **commits** under `verilog_2005`
    leg 3b ⭐ a MATCHED CONTROL — the same enclosing construct with the operator absent — in which
           that same terminal must read **committed = 0**

⛔ LEG 3 OF THE KEYWORD CENSUS DOES NOT TRANSFER.  There, substituting the keyword for a fresh
identifier and requiring REJECT proves the token was consumed AS A KEYWORD.  For punctuation that
substitution is meaningless — an operator has no identifier reading to fall back on.  The
punctuation analogue is the committed count, read by rule NAME from `rule_committed_counts`
(`--dump-rule-outcome-counts-json`, TOOLBOX 3.5) and ⛔ **never from the AST**: PGEN's AST is
annotation-shaped, so grepping it for a rule name is pinned at 0 for any rule without a `->`, which
cost `.13c.2x.9` two wrong root-cause labels.

⛔⛔ AND LEG 3b IS NOT OPTIONAL, BECAUSE `committed >= 1` ALONE IS A READING RATHER THAN A
MEASUREMENT (`.13e.12`, `PGEN-SV-CORPUS-GRAD-0300`).  Measured there: `wildcard_equal` is **ENTERED
twice in BOTH arms** — the parser speculates that branch whether or not `==?` is present — so the
ENTRY counter is provably blind to the question this instrument asks, and only the COMMITTED map
separates an accept from a failed probe.  A row whose control does not read 0 is REFUSED, not
published: an instrument that can only return one reading is not a measurement
(`feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement`).

⚠️ A STANDING CAVEAT, from the same leaf.  `try_parse` truncates the coverage stack only on its
**Err** arm, so a POSITIVE LOOKAHEAD THAT SUCCEEDS keeps its inner pushes.  The SV grammar's 7
positive-lookahead sites are all *peek-the-token-the-next-element-consumes*, so today this inflates a
multiplicity rather than manufacturing a phantom — but that is a property of THIS GRAMMAR, not of
the instrument, and leg 3b is the only thing that would catch it if that ever changed.

⛔ THE POPULATION IS DERIVED, NEVER RE-TYPED.  The candidate set comes from the census's own
`candidate_set()`; this file only names a WITNESS and a CONTROL per candidate.  If the two sets
disagree in either direction the instrument REFUSES (`rc=2`) rather than adjudicating a stale
population — a hand-listed population beside a derived one is a copy that rots.

Read-only with respect to the grammar, the parser and every generated artifact.  Writes tracked
report files only under `--write`.
"""
from __future__ import annotations

import argparse
import importlib
import pathlib
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve()
REPO_ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parent))
CENSUS = importlib.import_module("v2005_punctuation_faithfulness_census")

PROBE = REPO_ROOT / "rust/target/release/parseability_probe"
ARTIFACT = REPO_ROOT / "docs/tasks/artifacts/sv_corpus_grad/v2005_punctuation_adjudication"
STRICT, PERMISSIVE = "verilog_2005", "sv_2017"


class Refused(Exception):
    """The instrument cannot measure, so it must not report."""


def _shown(path: pathlib.Path) -> str:
    """Repo-root-relative when possible; never raises (a refusal message must not be able to fail)."""
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def _stmt(body: str) -> str:
    return f"module m; initial begin {body} end endmodule"


# The witness/control table.  KEYED BY RULE NAME, cross-checked against the census's derived set.
# A control is the SAME enclosing construct with the operator absent — never a different construct,
# because a control that changes two things at once explains nothing.
WITNESSES: dict[str, tuple[str, str]] = {
    "arithmetic_shift_left_assign":  (_stmt("a <<<= 1;"),   _stmt("a = 1;")),
    "arithmetic_shift_right_assign": (_stmt("a >>>= 1;"),   _stmt("a = 1;")),
    "shift_left_assign":             (_stmt("a <<= 1;"),    _stmt("a = 1;")),
    "shift_right_assign":            (_stmt("a >>= 1;"),    _stmt("a = 1;")),
    "percent_assign":                (_stmt("a %= 1;"),     _stmt("a = 1;")),
    "and_assign":                    (_stmt("a &= 1;"),     _stmt("a = 1;")),
    "or_assign":                     (_stmt("a |= 1;"),     _stmt("a = 1;")),
    "xor_assign":                    (_stmt("a ^= 1;"),     _stmt("a = 1;")),
    "star_assign":                   (_stmt("a *= 1;"),     _stmt("a = 1;")),
    "slash_assign":                  (_stmt("a /= 1;"),     _stmt("a = 1;")),
    "plus_assign":                   (_stmt("a += 1;"),     _stmt("a = 1;")),
    "minus_assign":                  (_stmt("a -= 1;"),     _stmt("a = 1;")),
    "plus_plus":                     (_stmt("a++;"),        _stmt("a = 1;")),
    "minus_minus":                   (_stmt("a--;"),        _stmt("a = 1;")),
    "wildcard_equal":                (_stmt("if (a ==? b) ;"),  _stmt("if (a == b) ;")),
    "wildcard_not_equal":            (_stmt("if (a !=? b) ;"),  _stmt("if (a != b) ;")),
    "iff_arrow":                     (_stmt("if (a <-> b) ;"),  _stmt("if (a && b) ;")),
    "tick":                          (_stmt("a = '{1,2};"), _stmt("a = 1;")),
    "dot_star": ("module s; endmodule\nmodule m; s u(.*); endmodule",
                 "module s; endmodule\nmodule m; s u(.a(1)); endmodule"),
}


def _run(src: str, profile: str, dump: bool, work: pathlib.Path) -> tuple[bool, dict[str, int]]:
    """Parse `src` under `profile`; return `(accepted, rule_committed_counts)`."""
    sv = work / "w.sv"
    sv.write_text(src)
    cmd = [str(PROBE), "--parse", "systemverilog", str(sv), "--profile", profile]
    out = work / "outcome.json"
    if dump:
        if out.exists():
            out.unlink()
        cmd += ["--dump-rule-outcome-counts-json", str(out)]
    rc = subprocess.run(cmd, capture_output=True, text=True).returncode
    committed: dict[str, int] = {}
    if dump and out.exists():
        import json
        committed = json.loads(out.read_text()).get("rule_committed_counts", {})
    return rc == 0, committed


def adjudicate() -> list[dict[str, object]]:
    candidates, _, _ = CENSUS.candidate_set()

    missing = sorted(set(candidates) - set(WITNESSES))
    extra = sorted(set(WITNESSES) - set(candidates))
    if missing:
        raise Refused(
            f"the census derives {len(candidates)} candidates and {len(missing)} of them have NO "
            f"witness here: {', '.join(missing)}. A candidate with no witness is an UNADJUDICATED "
            f"row, and reporting the rest as a complete verdict table fails in the flattering "
            f"direction — add a witness or say why the row is unadjudicable."
        )
    if extra:
        raise Refused(
            f"{len(extra)} witness row(s) name a rule the census no longer derives as a candidate: "
            f"{', '.join(extra)}. The grammar or the Annex A oracle moved under this table; "
            f"re-derive rather than adjudicating a population that no longer exists."
        )

    rows: list[dict[str, object]] = []
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "rust/target") as td:
        work = pathlib.Path(td)
        for name in sorted(candidates, key=lambda n: (-len(candidates[n]), candidates[n])):
            literal = candidates[name]
            witness, control = WITNESSES[name]
            strict_ok, strict_committed = _run(witness, STRICT, True, work)
            permissive_ok, _ = _run(witness, PERMISSIVE, False, work)
            committed = int(strict_committed.get(name, 0))
            ctl_ok, ctl_committed = _run(control, STRICT, True, work)
            ctl = int(ctl_committed.get(name, 0))

            if not permissive_ok:
                verdict = "REFUSED: witness rejects under sv_2017 too — it is malformed, not a probe"
            elif not strict_ok:
                verdict = "correctly excluded"
            elif not ctl_ok:
                verdict = "REFUSED: the CONTROL does not parse — it changed more than the operator"
            elif ctl != 0:
                verdict = f"REFUSED: control committed={ctl}, so this row's oracle cannot discriminate"
            elif committed >= 1:
                verdict = "CONFIRMED over-acceptance"
            else:
                verdict = "INCONCLUSIVE: accepts, but the operator's own terminal never commits"

            rows.append({
                "rule": name, "literal": literal, "leg1_v2005": "accept" if strict_ok else "reject",
                "leg2_sv2017": "accept" if permissive_ok else "reject",
                "leg3_committed": committed, "leg3b_control": ctl, "verdict": verdict,
                "witness": witness, "control": control,
            })
    return rows


def render(rows: list[dict[str, object]]) -> tuple[str, str]:
    tsv = ["\t".join(["rule", "literal", "leg1_verilog_2005", "leg2_sv_2017",
                      "leg3_committed", "leg3b_control_committed", "verdict"])]
    for r in rows:
        tsv.append("\t".join([str(r["rule"]), str(r["literal"]), str(r["leg1_v2005"]),
                              str(r["leg2_sv2017"]), str(r["leg3_committed"]),
                              str(r["leg3b_control"]), str(r["verdict"])]))
    confirmed = [r for r in rows if r["verdict"] == "CONFIRMED over-acceptance"]
    excluded = [r for r in rows if r["verdict"] == "correctly excluded"]
    other = [r for r in rows if r not in confirmed and r not in excluded]

    md = [
        "# L4 punctuation adjudication — IEEE 1364-2005 (`verilog_2005`)",
        "",
        "⛔ **DERIVED — do not edit by hand.** Regenerate with",
        "`python3 stimuli/sv/v2005_punctuation_adjudication.py --write`.",
        "",
        "Candidate population from `stimuli/sv/v2005_punctuation_faithfulness_census.py`'s own",
        "`candidate_set()` — this table names a witness and a matched control per candidate and",
        "never re-types the population. Owning leaf: `SV-CORPUS-GRAD.13e.10` (c).",
        "",
        f"**{len(rows)} candidates · {len(confirmed)} CONFIRMED over-acceptances · "
        f"{len(excluded)} correctly excluded · {len(other)} neither.**",
        "",
        "| rule | literal | leg 1 `verilog_2005` | leg 2 `sv_2017` | leg 3 `committed` | ⭐ leg 3b control | verdict |",
        "|---|---|---|---|---|---|---|",
    ]
    for r in rows:
        mark = "⛔ " if r["verdict"] == "CONFIRMED over-acceptance" else ""
        md.append(f"| `{r['rule']}` | `{r['literal']}` | {r['leg1_v2005']} | {r['leg2_sv2017']} | "
                  f"**{r['leg3_committed']}** | {r['leg3b_control']} | {mark}{r['verdict']} |")
    md += [
        "",
        "## Why leg 3b exists",
        "",
        "`committed >= 1` alone is a **reading**, not a measurement. `SV-CORPUS-GRAD.13e.12`",
        "measured that `wildcard_equal` is **ENTERED twice in BOTH arms** — the parser speculates",
        "that branch whether or not `==?` is present — so the ENTRY counter is provably blind to",
        "the question and only the COMMITTED map discriminates. Every row above therefore carries",
        "a matched control in which the same enclosing construct lacks the operator and the same",
        "terminal must read **0**. A row whose control does not read 0 is REFUSED, never published.",
        "",
        "## Witnesses",
        "",
        "| rule | witness | control |",
        "|---|---|---|",
    ]
    for r in rows:
        w = str(r["witness"]).replace("\n", " ⏎ ")
        c = str(r["control"]).replace("\n", " ⏎ ")
        md.append(f"| `{r['rule']}` | `{w}` | `{c}` |")
    return "\n".join(tsv) + "\n", "\n".join(md) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="refresh the tracked report files")
    ap.add_argument("--self-test", action="store_true", help="drive every refusal arm + a GREEN control")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    if not PROBE.exists():
        print(f"v2005-punctuation-adjudication: REFUSED — {_shown(PROBE)} is absent; build it "
              f"first. A probe that cannot measure must refuse rather than report a flattering "
              f"zero.", file=sys.stderr)
        return 2
    try:
        rows = adjudicate()
    except (CENSUS.Refused, Refused) as exc:
        print(f"v2005-punctuation-adjudication: REFUSED — {exc}", file=sys.stderr)
        return 2

    tsv, md = render(rows)
    tsv_path, md_path = ARTIFACT / "adjudication.tsv", ARTIFACT / "adjudication.md"
    if args.write:
        ARTIFACT.mkdir(parents=True, exist_ok=True)
        tsv_path.write_text(tsv)
        md_path.write_text(md)
        print(f"wrote {_shown(tsv_path)} and {_shown(md_path)}")

    confirmed = [r for r in rows if r["verdict"] == "CONFIRMED over-acceptance"]
    refused = [r for r in rows if str(r["verdict"]).startswith("REFUSED")]
    inconclusive = [r for r in rows if str(r["verdict"]).startswith("INCONCLUSIVE")]
    for r in rows:
        print(f"  {r['literal']:>5}  {r['rule']:32s} v2005={r['leg1_v2005']:6s} "
              f"sv_2017={r['leg2_sv2017']:6s} committed={r['leg3_committed']:>2} "
              f"control={r['leg3b_control']:>2}  {r['verdict']}")
    print(f"\ncandidates={len(rows)} confirmed={len(confirmed)} "
          f"excluded={len(rows) - len(confirmed) - len(refused) - len(inconclusive)} "
          f"inconclusive={len(inconclusive)} refused={len(refused)}")

    if refused:
        print("⛔ at least one row could not be adjudicated soundly — see the REFUSED verdicts "
              "above.", file=sys.stderr)
        return 2
    if not args.write:
        stale = []
        for path, want in ((tsv_path, tsv), (md_path, md)):
            if not path.exists() or path.read_text() != want:
                stale.append(_shown(path))
        if stale:
            print(f"⛔ tracked artifact(s) DIFFER from a fresh derivation: {', '.join(stale)} — "
                  f"re-run with --write.", file=sys.stderr)
            return 1
    return 0


def self_test() -> int:
    """Drive every refusal arm, plus a GREEN control on the real tree in the same run."""
    passed = failed = 0

    def check(name: str, ok: bool, detail: str = "") -> None:
        nonlocal passed, failed
        if ok:
            print(f"  ✅ {name}")
            passed += 1
        else:
            print(f"  ❌ {name}  {detail}")
            failed += 1

    saved = dict(WITNESSES)

    # ARM 1 — a candidate with no witness must REFUSE, not silently shrink the table.
    dropped = next(iter(saved))
    WITNESSES.pop(dropped)
    try:
        adjudicate()
        check("a candidate with NO witness refuses", False, "it adjudicated anyway")
    except Refused as exc:
        check("a candidate with NO witness refuses", "NO witness here" in str(exc), str(exc)[:90])
    finally:
        WITNESSES.clear(); WITNESSES.update(saved)

    # ARM 2 — a witness naming a rule the census does not derive must REFUSE.
    WITNESSES["a_rule_the_census_never_derives"] = ("module m; endmodule", "module m; endmodule")
    try:
        adjudicate()
        check("a witness for a NON-candidate refuses", False, "it adjudicated anyway")
    except Refused as exc:
        check("a witness for a NON-candidate refuses", "no longer derives" in str(exc), str(exc)[:90])
    finally:
        WITNESSES.clear(); WITNESSES.update(saved)

    # ARM 3 — ⭐ THE LOAD-BEARING ONE. Substitute a control that still CONTAINS the operator; the
    # row must be REFUSED rather than confirmed, because its oracle can no longer discriminate.
    WITNESSES["wildcard_equal"] = (_stmt("if (a ==? b) ;"), _stmt("if (a ==? b) ;"))
    try:
        rows = adjudicate()
        row = next(r for r in rows if r["rule"] == "wildcard_equal")
        check("a control that does not remove the operator refuses",
              str(row["verdict"]).startswith("REFUSED: control committed="), str(row["verdict"]))
    finally:
        WITNESSES.clear(); WITNESSES.update(saved)

    # ARM 4 — GREEN control on the real tree, in the same run, so the refusals are not vacuous.
    rows = adjudicate()
    confirmed = [r for r in rows if r["verdict"] == "CONFIRMED over-acceptance"]
    excluded = [r for r in rows if r["verdict"] == "correctly excluded"]
    check(f"the real tree adjudicates cleanly ({len(rows)} rows, {len(confirmed)} confirmed, "
          f"{len(excluded)} excluded)",
          len(rows) > 0 and not [r for r in rows if str(r["verdict"]).startswith("REFUSED")])
    # ARM 5 — the verdict column MOVES: confirmed and excluded are both non-empty. A column that
    # only ever prints one value is not a measurement.
    check("the verdict column MOVES (both confirmed and excluded are non-empty)",
          bool(confirmed) and bool(excluded))

    print(f"\nself-test: {passed} passed, {failed} failed")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
