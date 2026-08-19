#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2u — PARTITION the rules that never fired on the corpus, BY CONSTRUCTION.

⛔⛔ THE DEFECT THIS EXISTS FOR IS NOT A WRONG NUMBER, IT IS AN UNREAD ONE THAT READS AS A DEFECT
SURFACE. `stimuli/sv/characterization/rule_coverage_sv_2017.tsv` records, per grammar rule, how many
accepted corpus files ever caused it to fire, and publishes a flat `GAP` list for the zeros. Nothing
consumed it, and `.13c.2u` opened on its headline — *"104 grammar rules have NEVER fired on 16,336
real SystemVerilog files"* — as the highest-yield rejects-valid hunt on the board.

Measured (`PGEN-SV-CORPUS-GRAD-0239`), the headline is misleading in BOTH directions:

  * the 104 was dated and is now **137** — the grammar gained 135 rules since;
  * and a large part of the list CANNOT fire for a reason that is not a defect at all. PGEN's
    left-recursion eliminator AUTHORS rules (`X_lr_base`, `X_lr_suffix`, `X_lr_seed_*`,
    `X_lr_guard*`) and REPLACES the body of the rule it eliminated. Those names live in the rule
    table, so the coverage instrument dutifully reports them as never-fired — but they are not IEEE
    1800 constructs and no corpus file can ever "exercise" them by name. ⭐ Demonstrated rather than
    argued: `casting_type` is listed GAP, yet on a file that ACCEPTS through a size cast
    (`8'(1)`) its nineteen `casting_type_lr_*` replacements fire and `casting_type` itself never
    appears in the entry counts at all.

⇒ a flat GAP list sends a reader hunting phantom defects. This instrument CLASSIFIES it instead, and
`scripts/check_sv_rule_fire_partition.sh` makes the result a gate, so the number is CONSUMED.

THE PARTITION — every class is decided by an instrument, never by reading:

  lr_synthetic            the name carries `_lr_`: the ELIMINATOR authored it. Not an LRM construct.
  lr_eliminated_original  `--lint-grammar`'s own `[info]` line NAMES the rules the pass eliminated;
                          the name survives, the body does not. ⚠️ NOT uniform — `property_expr` is
                          eliminated AND covered, so this is checked per rule, never assumed.
  unreachable_from_entry  certificate coverage carries a verified PROOF that the rule is
                          unreachable from the declared entry universe ⇒ it CANNOT fire, and that
                          is a PROOF, not an absence of evidence. ⚠️ Not automatically a defect:
                          `library_text` / `include_statement` / `kw_incdir` and friends are
                          reachable only from the ALTERNATE entry `library_text`, and this pass is
                          run with `--entry-rule systemverilog_file`. Adjudicate A1
                          (alternate-entry, expected) vs A2 (profile-orphan) per TOOLBOX 4.2.
  coverage_gap            certificate coverage WITNESSED the rule (`--report-certificate-coverage`
                          generated a sample that parses AND reaches it) ⇒ it CAN fire; the corpus
                          simply holds no instance. Honest and expected.
  unwitnessed             neither: the cert pass could not witness it. ⛔ **A CANDIDATE, NOT A
                          VERDICT** — TOOLBOX 4.4 is explicit that `witnessed_target=false` is a
                          statement about the witness GENERATOR, not about the parser, and reading
                          it as a defect once produced a confident wrong root cause that reached a
                          task leaf. Each one is adjudicated by hand with a minimal LRM construct
                          and recorded in `ADJUDICATED_UNWITNESSED` below.

⛔⛔ AND THE FIRST CUT OF THIS INSTRUMENT MISCLASSIFIED 18 RULES, because it inferred `witnessed`
from `not UNKNOWN`. The report published `proof=<n>` and never the NAMES, so the proof set was
unavailable and everything outside the UNKNOWN list was assumed witnessed — which silently promoted
18 PROVEN-UNREACHABLE rules into `coverage_gap`, the class that says "it can fire". `.13c.2u.2`
added the names to the report under the existing `PGEN_CERT_COVERAGE_DUMP_ALL` gate (read-only,
default output byte-identical, headline unchanged), and this now reads them. ⇒ **an absent list is
not an empty one, and inferring a class from the complement of the one list you have is how a
proven negative becomes a positive.**

⭐⭐ THE JOIN CARRIES A FREE CONTROL — THE DENOMINATOR. Both instruments enumerate the same rule
population, so the certificate pass's `total` MUST equal the count of coverage-artifact rules that
are satisfiable under the profile, and every rule the cert names must exist in the artifact. If they
disagree, the two sides are describing different grammars — which is exactly the staleness this leaf
was opened about — and no join over them means anything. This refuses rather than reporting.

⛔⛔ AND THE FIRST CONTROL WRITTEN HERE WAS WRONG, WHICH IS WORTH THE SPACE. It refused when a rule
the corpus had exercised was `UNKNOWN` to the cert, calling that a contradiction. It is not: `UNKNOWN`
means the witness GENERATOR could not route to the rule, not that the rule cannot fire (TOOLBOX 4.4).
The control fired on its first run against SEVEN rules — `kw_accept_on`, `kw_nexttime`,
`kw_reject_on`, `kw_s_eventually`, `kw_s_nexttime`, `kw_sync_reject_on`,
`known_unscoped_property_identifier` — all of which are simply corpus-covered and generator-hard.
⭐⭐ THE REAL CONTRADICTION IS NOW IMPLEMENTED, not documented as unavailable: corpus-`covered` ∩
certificate `proof` — a rule whose COMMITTED entries in an accepted real-world file refute a
verified proof that it is unreachable. `.13c.2u` recorded that check as impossible because the
report withheld the proof names; `.13c.2u.2` published them, so the check exists. It measures **0**
on this tree, which is the answer a correct tree should give.

USAGE   python3 stimuli/sv/rule_fire_partition.py [--cert-report FILE] [--out PREFIX]
        (with no --cert-report it RUNS the certificate pass itself, ~5 min)
EXIT    0 = partitioned · 1 = a contradiction or an unadjudicated `unwitnessed` rule · 2 = refused
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if not (ROOT / "grammars").is_dir():
    refuse(f"rule_fire_partition: not at the repo root (derived {ROOT})")

GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
COVERAGE = ROOT / "stimuli/sv/characterization/rule_coverage_sv_2017.tsv"
PIPE = ROOT / "rust/target/debug/ast_pipeline"
PROFILE = "sv_2017"
OUT_DEFAULT = ROOT / "stimuli/sv/characterization/rule_fire_partition_sv_2017"

# ── Hand-adjudicated `unwitnessed` rules ───────────────────────────────────────────────────────
# ⛔ A rule lands here ONLY with a measured construct behind it. The recipe is `.13c.2q`'s: hand the
# rule a minimal construct from the tracked LRM, parse it, and check the rule's own ARM fires —
# a VERDICT alone is not enough (`.13c.2q`: the construct parsed through the WRONG production and no
# verdict-only oracle could see it). The reproducers live in `stimuli/sv/rule_fire_probes/`.
ADJUDICATED_UNWITNESSED = {
    "kw_eventually_5865020f": (
        "coverage_gap", "IEEE 1800-2023 A.2.10 `property_expr ::= … | eventually [ constant_range ] "
        "property_expr`. MEASURED: `eventually [1:2] b` inside a property ACCEPTS and "
        "kw_eventually_5865020f fires 3x, so the rule is reachable; the corpus holds no SVA "
        "property operator in any accepted file"),
    "kw_s_always_0f9aa900": (
        "coverage_gap", "IEEE 1800-2023 A.2.10 `| s_always [ constant_range ] property_expr`. "
        "MEASURED: `s_always [1:2] b` ACCEPTS and kw_s_always_0f9aa900 fires 3x"),
    "kw_sync_accept_on_b65eda53": (
        "coverage_gap", "IEEE 1800-2023 A.2.10 `| sync_accept_on ( expression_or_dist ) "
        "property_expr`. MEASURED: `sync_accept_on (a) b` ACCEPTS and the keyword rule fires 4x "
        "(its sibling sync_reject_on measured identically)"),
}


def refuse(msg: str) -> "NoReturn":
    """Exit 2 — the documented `refused` code.

    ⛔ `refuse("text")` exits **1**, which this file's own contract reserves for "ran and
    found a problem". Its probe caught the mismatch: a missing certificate report scored rc=1 where
    the docstring promises 2, so a caller branching on the exit code could not tell "I could not
    run" from "I ran and the tree is bad".
    """
    print(msg, file=sys.stderr)
    raise SystemExit(2)


def lint_eliminated() -> set[str]:
    """The rules PGEN's LR pass eliminated — from the pass's OWN outcome line, not a guess."""
    r = subprocess.run([str(PIPE), str(GRAMMAR), "--lint-grammar"],
                       capture_output=True, text=True, timeout=900)
    blob = r.stdout + r.stderr
    m = re.search(r"ELIMINATED \d+ left-recursive rule\(s\) on this grammar: ([^\n]+?) —", blob)
    if not m:
        refuse("rule_fire_partition: --lint-grammar printed no ELIMINATED line; the "
                         "instrument cannot classify without it. Refusing.")
    return {x.replace(" (indirect)", "").strip() for x in m.group(1).split(",")}


def cert_report(path: Path | None) -> str:
    if path:
        if not path.is_file():
            refuse(f"rule_fire_partition: no certificate report at {path} — refusing "
                             f"rather than reporting a partition with one side missing")
        return path.read_text(encoding="utf-8", errors="replace")
    env = {**os.environ, "PGEN_CERT_COVERAGE_DUMP_ALL": "1"}
    r = subprocess.run([str(PIPE), str(GRAMMAR), "--report-certificate-coverage",
                        "--grammar-profile", PROFILE, "--entry-rule", "systemverilog_file",
                        "--count", "40", "--seed", "0"],
                       capture_output=True, text=True, env=env, timeout=3600)
    return r.stdout + r.stderr


def parse_cert(blob: str) -> tuple[set[str], set[str], dict[str, int]]:
    m = re.search(r"UNKNOWN rules \(\d+ of \d+ shown\): \[(.*?)\]", blob, re.S)
    if not m:
        refuse("rule_fire_partition: the certificate report carries no UNKNOWN rule "
                         "list — run it with PGEN_CERT_COVERAGE_DUMP_ALL=1. Refusing.")
    head = re.search(r"CERTIFICATE-COVERAGE: .*?total=(\d+) proof=(\d+) witness=(\d+) "
                     r"UNKNOWN=(\d+)", blob)
    if not head:
        refuse("rule_fire_partition: no CERTIFICATE-COVERAGE headline. Refusing.")
    totals = dict(zip(("total", "proof", "witness", "unknown"),
                      (int(head.group(i)) for i in (1, 2, 3, 4))))
    pm = re.search(r"PROOF-COVERED rules \(\d+ of \d+ shown\): \[(.*?)\]", blob, re.S)
    if not pm:
        refuse("rule_fire_partition: the certificate report carries no PROOF-COVERED rule list. "
               "That list is what separates `it cannot fire (proven)` from `it can fire, the "
               "corpus lacks it` — without it this instrument would infer `witnessed` from `not "
               "UNKNOWN` and silently promote proven-unreachable rules into the coverage-gap "
               "class, which is exactly the defect `.13c.2u.2` fixed. Re-run the certificate pass "
               "with PGEN_CERT_COVERAGE_DUMP_ALL=1 on an ast_pipeline built at or after `-0240`. "
               "Refusing.")
    proofs = set(re.findall(r'"([^"]+)"', pm.group(1)))
    if len(proofs) != totals["proof"]:
        refuse(f"rule_fire_partition: the report says proof={totals['proof']} but its PROOF-COVERED "
               f"list holds {len(proofs)} names — the report contradicts itself. Refusing.")
    return set(re.findall(r'"([^"]+)"', m.group(1))), proofs, totals


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cert-report", type=Path)
    ap.add_argument("--out", type=Path, default=OUT_DEFAULT)
    a = ap.parse_args()
    if not PIPE.exists():
        print(f"rule_fire_partition: no ast_pipeline at {PIPE}", file=sys.stderr)
        return 2
    if not COVERAGE.exists():
        print(f"rule_fire_partition: {COVERAGE} is absent — run corpus_rule_coverage.py first",
              file=sys.stderr)
        return 2

    status: dict[str, str] = {}
    for line in COVERAGE.read_text(encoding="utf-8").splitlines()[1:]:
        f = line.split("\t")
        if len(f) >= 2:
            status[f[0]] = f[1]
    gaps = sorted(r for r, s in status.items() if s == "GAP")
    if not gaps:
        print("rule_fire_partition: the coverage artifact reports ZERO gaps — that is either a "
              "triumph or a broken artifact, and this instrument will not guess. Refusing.",
              file=sys.stderr)
        return 2

    eliminated = lint_eliminated()
    unknown, proofs, totals = parse_cert(cert_report(a.cert_report))

    rows, buckets = [], {}
    for r in gaps:
        if "_lr_" in r:
            cls, why = "lr_synthetic", "the LR eliminator authored this name; not an LRM construct"
        elif r in eliminated:
            cls, why = ("lr_eliminated_original",
                        "named by --lint-grammar as LR-eliminated: the name survives, the body was "
                        "replaced by its _lr_* chain")
        elif r in proofs:
            cls, why = ("unreachable_from_entry",
                        "certificate coverage carries a verified PROOF that it is unreachable from "
                        "the declared entry universe (adjudicate A1 alternate-entry vs A2 "
                        "profile-orphan — TOOLBOX 4.2)")
        elif r in unknown:
            adj = ADJUDICATED_UNWITNESSED.get(r)
            cls, why = (adj if adj else
                        ("unwitnessed", "the certificate pass could not witness it — a CANDIDATE "
                                        "needing a minimal-LRM-construct adjudication"))
        else:
            cls, why = "coverage_gap", "certificate coverage WITNESSED it, so it can fire"
        rows.append((r, cls, why))
        buckets[cls] = buckets.get(cls, 0) + 1

    # ── the free control: both instruments must be describing the SAME grammar ────────────────
    satisfiable = sum(1 for v in status.values() if v != "na_profile")
    denom_mismatch = (satisfiable != totals["total"])
    unknown_not_in_artifact = sorted(r for r in unknown if r not in status)
    corpus_covered_but_generator_hard = sorted(
        r for r, v in status.items() if v == "covered" and r in unknown)
    # THE REAL CONTRADICTION, now computable: a verified PROOF of unreachability refuted by a rule
    # that COMMITS in an accepted real-world file.
    proof_refuted_by_corpus = sorted(r for r, v in status.items() if v == "covered" and r in proofs)

    a.out.parent.mkdir(parents=True, exist_ok=True)
    tsv = a.out.with_suffix(".tsv")
    with open(tsv, "w", encoding="utf-8") as fh:
        fh.write("rule\tclass\tbasis\n")
        for r, c, w in rows:
            fh.write(f"{r}\t{c}\t{w}\n")
    summary = {"profile": PROFILE, "gap_rules": len(gaps), "certificate": totals,
               "coverage_artifact_satisfiable": satisfiable,
               "classes": dict(sorted(buckets.items())),
               "corpus_covered_but_generator_hard": corpus_covered_but_generator_hard,
               "proof_refuted_by_corpus": proof_refuted_by_corpus,
               "unadjudicated_unwitnessed": sorted(r for r, c, _ in rows if c == "unwitnessed")}
    a.out.with_suffix(".json").write_text(json.dumps(summary, indent=1) + "\n", encoding="utf-8")

    print(f"RULE-FIRE-PARTITION: profile={PROFILE} gap_rules={len(gaps)} "
          f"(cert total={totals['total']} witness={totals['witness']} proof={totals['proof']} "
          f"UNKNOWN={totals['unknown']})")
    for c in sorted(buckets):
        print(f"  {c:<24} {buckets[c]:>4}")
    # `Path.relative_to` raises on a RELATIVE argument; `--out rust/target/x` is the
    # natural way to type it and crashed the CONTROL arm of this instrument's own
    # probe. Second occurrence of this bug in one session — use os.path.relpath.
    print(f"  -> {os.path.relpath(tsv, ROOT)}")

    print(f"  {'corpus-covered but generator-hard':<24} {len(corpus_covered_but_generator_hard):>4}"
          f"   (not a defect: the corpus is a BETTER witness than the cert pass for these)")

    rc = 0
    if proof_refuted_by_corpus:
        print(f"\n⛔ CONTRADICTION: {len(proof_refuted_by_corpus)} rule(s) carry a certificate PROOF "
              f"of unreachability AND commit in an accepted corpus file. A proof refuted by real "
              f"text means one of the two instruments is wrong; this will not pick. "
              f"{proof_refuted_by_corpus[:10]}", file=sys.stderr)
        rc = 1
    if denom_mismatch:
        print(f"\n⛔ DENOMINATOR MISMATCH: the coverage artifact holds {satisfiable} rules "
              f"satisfiable under {PROFILE}, the certificate pass reports total={totals['total']}. "
              f"The two sides describe DIFFERENT grammars, so the join is meaningless — one of them "
              f"is stale. Re-derive both, do not reconcile by hand.", file=sys.stderr)
        rc = 1
    if unknown_not_in_artifact:
        print(f"\n⛔ {len(unknown_not_in_artifact)} rule(s) the certificate pass names are ABSENT "
              f"from the coverage artifact — the report is stale relative to the grammar: "
              f"{unknown_not_in_artifact[:10]}", file=sys.stderr)
        rc = 1
    if summary["unadjudicated_unwitnessed"]:
        print(f"\n⛔ {len(summary['unadjudicated_unwitnessed'])} unwitnessed rule(s) carry NO "
              f"adjudication. Hand each a minimal construct from the tracked LRM, check its ARM "
              f"fires, and record it in ADJUDICATED_UNWITNESSED: "
              f"{summary['unadjudicated_unwitnessed']}", file=sys.stderr)
        rc = 1
    return rc


if __name__ == "__main__":
    sys.exit(main())
