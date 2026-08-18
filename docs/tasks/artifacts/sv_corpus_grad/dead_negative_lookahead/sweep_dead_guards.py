#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.13c.2c` — the DEAD-NEGATIVE-LOOKAHEAD sweep.

WHAT IT ASKS
------------
`.13c.2c` was one grammar token: `split_hierarchical_callable_receiver`'s loop guard
`!callable_method_call_body`, which could **never pass on an identifier** because IEEE 1800 A.8.2
makes `array_manipulation_call`'s parens OPTIONAL — so a bare member name already IS a
`callable_method_call_body`. The loop it guarded ran zero iterations for months and nothing could
see it, because *a guard that never passes* and *a loop that is never needed* are indistinguishable
from outside.

⛔ That is a CLASS, not an instance (the `.13c.2e` lesson: `SV-0049` fixed one brace
mis-transcription and did not sweep, which is the only reason `.13c.2e` existed three days later).
This instrument sweeps it: for every negative lookahead `!R` in the SV grammar, does `R` accept a
BARE SIMPLE IDENTIFIER? If it does, the guard is vacuously false everywhere an identifier is legal
at that position — the exact shape that killed `.13c.2c`'s loop.

HOW IT DECIDES
--------------
Not by reading the grammar — by PARSING. Each guard subject is run as a start symbol through the
grammar interpreter (TOOLBOX 1.5b, `--interpret-entry-rule`) against the one-character input `x`.
The interpreter is authoritative here by verification, not construction: `parse_harness_combinator_gate`
(TOOLBOX 1.7) pins it byte-identical to the compile-and-run oracle per structural combinator.

⛔ FOUR CONTROLS RUN BEFORE ANY VERDICT PRINTS, so a green sweep cannot be a broken probe:
  * `identifier`  MUST accept `x`  — the probe can report DEAD-RISK at all.
  * `lparen`      MUST reject `x`  — the probe can report OK at all.
  * every guard subject named in the grammar MUST resolve to a rule — a typo would otherwise be
    silently classified as "not identifier-shaped".
  * the grammar MUST yield at least one `!` site — an extractor that finds nothing reads as a clean
    sweep.

HONEST BOUND, stated rather than hidden
---------------------------------------
This decides ONE shape: *the subject accepts a bare identifier*. A guard can also be dead because
its subject accepts some OTHER token that always appears in the guarded position; this sweep does
not see that, and the inline `!( ... )` group forms are reported with their text rather than
probed, because they have no rule name to enter. Both are printed in full rather than summarised.

    python3 docs/tasks/artifacts/sv_corpus_grad/dead_negative_lookahead/sweep_dead_guards.py

⭐ `--grammar <file>` points the sweep at any grammar, which is how it was PROVEN ABLE TO FAIL
([[a-check-whose-inputs-all-pass-has-not-been-tested]]): run it against the pre-fix grammar
(`git show <fixing-commit>~1:grammars/systemverilog.ebnf`) and it reports
`dead_risk=1 -> callable_method_call_body` at exit 1.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
PIPELINE = ROOT / "rust/target/debug/ast_pipeline"
PROBE_INPUT = "x"
TIMEOUT_S = 300

NAMED_GUARD_RE = re.compile(r"!([a-zA-Z_][a-zA-Z0-9_]*)")
GROUP_GUARD_RE = re.compile(r"!\(\s*([^)]*?)\s*\)")
RULE_HEAD_RE = re.compile(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*:=", re.MULTILINE)


def grammar_lines() -> list[tuple[int, str]]:
    """Grammar lines with comment lines dropped — a `!R` inside prose is not a guard."""
    out = []
    for n, line in enumerate(GRAMMAR.read_text(encoding="utf-8").splitlines(), 1):
        stripped = line.lstrip()
        if stripped.startswith("#") or stripped.startswith("//"):
            continue
        out.append((n, line))
    return out


def accepts_bare_identifier(rule: str) -> bool:
    with tempfile.TemporaryDirectory(dir=ROOT / "rust/target") as td:
        f = Path(td) / "probe.txt"
        f.write_text(PROBE_INPUT, encoding="utf-8")
        proc = subprocess.run(
            [str(PIPELINE), str(GRAMMAR), "--interpret-parse", str(f),
             "--interpret-entry-rule", rule],
            capture_output=True, text=True, timeout=TIMEOUT_S)
        return proc.returncode == 0


def main() -> int:
    global GRAMMAR
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--grammar", type=Path, default=GRAMMAR,
                    help="grammar to sweep (default: grammars/systemverilog.ebnf); point it at a "
                         "pre-fix revision to prove this check can go RED")
    args = ap.parse_args()
    GRAMMAR = args.grammar
    if not GRAMMAR.exists():
        print(f"⛔ REFUSING: grammar {GRAMMAR} does not exist")
        return 2
    if not PIPELINE.exists():
        print(f"⛔ REFUSING: {PIPELINE} is not built. "
              'Build it: cd rust && cargo build --features "generated_parsers ebnf_dual_run" '
              "--bin ast_pipeline")
        return 2

    lines = grammar_lines()
    declared = set(RULE_HEAD_RE.findall(GRAMMAR.read_text(encoding="utf-8")))

    named: dict[str, list[int]] = {}
    groups: list[tuple[int, str]] = []
    for n, line in lines:
        for m in GROUP_GUARD_RE.finditer(line):
            groups.append((n, m.group(1)))
        # strip the group forms so `!( identifier )` is not also read as `!identifier`
        for m in NAMED_GUARD_RE.finditer(GROUP_GUARD_RE.sub("!<group>", line)):
            named.setdefault(m.group(1), []).append(n)

    # ---- controls, before any verdict -------------------------------------------------
    failures: list[str] = []
    if not named and not groups:
        failures.append("no `!` lookahead found in the grammar — the extractor is broken, "
                        "and an empty sweep reads as a clean one")
    if not accepts_bare_identifier("identifier"):
        failures.append("control: rule `identifier` does NOT accept a bare identifier — the probe "
                        "cannot report DEAD-RISK, so a green sweep would be meaningless")
    if accepts_bare_identifier("lparen"):
        failures.append("control: rule `lparen` ACCEPTS a bare identifier — the probe cannot "
                        "report OK, so every row would read DEAD-RISK")
    for rule in sorted(named):
        if rule not in declared:
            failures.append(f"guard subject `{rule}` (lines {named[rule]}) resolves to no rule "
                            "head — a typo would be silently classified as not-identifier-shaped")
    if failures:
        print("⛔ CONTROLS FAILED — no verdict printed:")
        for f in failures:
            print(f"  ⛔ {f}")
        return 2
    print(f"controls: identifier accepts `{PROBE_INPUT}` ✓ · lparen rejects it ✓ · "
          f"{len(named)} guard subjects all resolve ✓ · {len(named)+len(groups)} `!` sites found ✓")
    print()

    dead_risk = []
    print(f"{'guard subject':<38} {'sites':<26} verdict")
    print("-" * 92)
    for rule in sorted(named):
        risky = accepts_bare_identifier(rule)
        verdict = "⛔ DEAD-RISK (accepts a bare identifier)" if risky else "OK (cannot match an identifier)"
        if risky:
            dead_risk.append(rule)
        sites = ",".join(str(n) for n in named[rule])
        print(f"!{rule:<37} {sites:<26} {verdict}")

    print()
    print("inline `!( … )` group guards — reported, not probed (no rule name to enter):")
    for n, text in groups:
        print(f"  line {n:<6} !( {text} )")

    print()
    print(f"DEAD-GUARD-SWEEP: subjects={len(named)} groups={len(groups)} dead_risk={len(dead_risk)}"
          + (f" -> {', '.join(dead_risk)}" if dead_risk else ""))
    return 1 if dead_risk else 0


if __name__ == "__main__":
    sys.exit(main())
