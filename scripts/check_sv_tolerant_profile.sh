#!/usr/bin/env bash
# scripts/check_sv_tolerant_profile.sh
#
# DOCTRINE `SV-TOLERANT-PROFILE-SUPERSET` (structural) — SV-CORPUS-GRAD.43.
#
#   The opt-in tolerance profile `sv_2023_tolerant` is `sv_2023` PLUS A CLOSED LIST of spellings, and nothing
#   else (director decision 2026-10-02, docs/decisions/project_sv_opt_in_tolerance_profile.md).
#
# ⭐ WHY THIS IS A DOCTRINE. The profile rides the existing `@profiles` directive: every rule live under
# `sv_2023` is ALSO listed for `sv_2023_tolerant`. That is 393 lists, and the class of defect it creates is the
# one an otherwise-correct edit makes: a NEW rule gated `@profiles: ["sv_2023"]` that forgets the tolerant name
# silently removes a STANDARD construct from the tolerant profile — the profile then rejects valid 1800-2023
# text while every strict gate stays green, because no strict gate reads the tolerant profile.
#
# WHAT IT ASSERTS (static, on the tracked grammar; <1 s; no parser, no oracle):
#   (1) SUPERSET — every `@profiles` list that names `sv_2023` names `sv_2023_tolerant` too.
#   (2) CLOSED LIST BY NAME — a rule gated to the tolerant profile ALONE is named `tolerated_…`, so the closed
#       list is greppable and a tolerant-only rule cannot hide under an ordinary name.
#   (3) NO STRAY — `sv_2023_tolerant` never appears in a list WITHOUT `sv_2023` unless it is alone (a list such as
#       ["sv_2017", "sv_2023_tolerant"] would give the tolerant profile a construct its strict base lacks,
#       outside the closed list).
# The behavioural half — every `sv_2023` pin keeps its verdict under the tolerant profile, and each tolerated
# spelling has a strict twin that still rejects — is the pin runner's superset sweep
# (stimuli/sv/run_adjudication_repros.py), run at every release ceremony.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit 0 holds / 1 breach / 2 refuses to evaluate; explains on stderr;
# deterministic; read-only; path-agnostic.
#   bash scripts/check_sv_tolerant_profile.sh              # the doctrine
#   bash scripts/check_sv_tolerant_profile.sh --self-test  # prove each arm can fail
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2
GRAMMAR="${PGEN_SV_TOLERANT_GRAMMAR:-grammars/systemverilog.ebnf}"
exec python3 - "$GRAMMAR" "${1:-}" <<'PY'
import re, sys, tempfile, os, subprocess
TAG = "sv-tolerant-profile"; BASE = "sv_2023"; TOL = "sv_2023_tolerant"
grammar, mode = sys.argv[1], sys.argv[2]
DIRECTIVE = re.compile(r'^@profiles:\s*\[([^\]\n]*)\]\s*$')
RULE = re.compile(r'^([A-Za-z_][A-Za-z0-9_]*)\s*:=')

def evaluate(text):
    """(breaches, stats). A directive binds to the NEXT rule head."""
    breaches, lists, widened, alone = [], 0, 0, []
    pending = None
    for n, line in enumerate(text.splitlines(), 1):
        m = DIRECTIVE.match(line)
        if m:
            names = re.findall(r'"([^"]+)"', m.group(1))
            pending = (n, names)
            lists += 1
            if BASE in names and TOL not in names:
                breaches.append(f"(1) SUPERSET — line {n}: `@profiles: [{m.group(1)}]` names `{BASE}` without `{TOL}`: "
                                f"the rule it gates would be MISSING from the tolerant profile.")
            if TOL in names and BASE not in names and names != [TOL]:
                breaches.append(f"(3) STRAY — line {n}: `@profiles: [{m.group(1)}]` gives `{TOL}` a rule its strict base "
                                f"`{BASE}` lacks, outside the closed list (a tolerant-only rule names `{TOL}` ALONE).")
            if BASE in names and TOL in names:
                widened += 1
            continue
        r = RULE.match(line)
        if r and pending is not None:
            n0, names = pending
            pending = None
            if names == [TOL]:
                alone.append(r.group(1))
                if not r.group(1).startswith("tolerated_"):
                    breaches.append(f"(2) CLOSED LIST BY NAME — line {n}: rule `{r.group(1)}` is gated to `{TOL}` alone "
                                    f"(directive at line {n0}) and is not named `tolerated_…`.")
    return breaches, {"lists": lists, "widened": widened, "tolerant_only_rules": alone}

def run(path):
    if not os.path.isfile(path):
        print(f"{TAG}: REFUSED (2) — grammar {path} is missing", file=sys.stderr); return 2
    text = open(path, encoding="utf-8").read()
    breaches, st = evaluate(text)
    if TOL not in text:
        print(f"{TAG}: REFUSED (2) — {path} declares no `{TOL}` profile at all; the doctrine has no subject "
              "(was the profile removed? that is a director decision, not an edit).", file=sys.stderr); return 2
    if breaches:
        for b in breaches: print(f"{TAG}: ✗ {b}", file=sys.stderr)
        print(f"{TAG}: {len(breaches)} breach(es). `{TOL}` is `{BASE}` plus a closed list; add the tolerant name to "
              "the list (a standard construct), or name the rule `tolerated_…` and pin it both ways "
              "(a director-admitted spelling).", file=sys.stderr)
        return 1
    print(f"{TAG}: OK ({st['widened']} of {st['lists']} `@profiles` lists name `{BASE}` and every one names `{TOL}`; "
          f"tolerant-only rules={len(st['tolerant_only_rules'])}: {', '.join(st['tolerant_only_rules'])})")
    return 0

def self_test():
    fails = []
    def arm(name, text, want_substr, want_count=None):
        b, _ = evaluate(text)
        ok = (any(want_substr in x for x in b) if want_substr else not b)
        if want_count is not None: ok = ok and len(b) == want_count
        print(f"  {'PASS' if ok else 'FAIL'}  {name}")
        if not ok: fails.append(name)
    good = '@profiles: ["sv_2017", "sv_2023", "sv_2023_tolerant"]\na := b\n\n@profiles: ["sv_2023_tolerant"]\ntolerated_x := c\n\n@profiles: ["verilog_2005"]\nd := e\n'
    arm("a grammar that holds the invariant is green", good, None, 0)
    arm("a list naming sv_2023 without the tolerant profile is a SUPERSET breach",
        good + '@profiles: ["sv_2023"]\nnew_2023_rule := x\n', "(1) SUPERSET", 1)
    arm("a tolerant-only rule under an ordinary name is refused",
        good + '@profiles: ["sv_2023_tolerant"]\nplain_name := x\n', "(2) CLOSED LIST BY NAME", 1)
    arm("the tolerant profile beside another profile without its base is a STRAY",
        good + '@profiles: ["sv_2017", "sv_2023_tolerant"]\ny := x\n', "(3) STRAY", 1)
    arm("a directive binds to the NEXT rule across a comment line",
        '@profiles: ["sv_2023_tolerant"]\n# a comment\nplain := x\n', "(2) CLOSED LIST BY NAME", 1)
    arm("a commented-out directive is not a directive", good + '// @profiles: ["sv_2023"]\nz := x\n', None, 0)
    # the driver's own refusals, through this same file
    with tempfile.TemporaryDirectory() as t:
        p = os.path.join(t, "none.ebnf"); open(p, "w").write('@profiles: ["sv_2023"]\na := b\n')
        rc = run(p)
        ok = rc == 2; print(f"  {'PASS' if ok else 'FAIL'}  a grammar with no tolerant profile REFUSES (2), never passes")
        if not ok: fails.append("no-subject refusal")
        rc = run(os.path.join(t, "absent.ebnf"))
        ok = rc == 2; print(f"  {'PASS' if ok else 'FAIL'}  a missing grammar REFUSES (2)")
        if not ok: fails.append("missing refusal")
    print(f"{TAG}: self-test arms=8 failed={len(fails)}")
    return 1 if fails else 0

sys.exit(self_test() if mode == "--self-test" else run(grammar) if mode == "" else (print(f"{TAG}: REFUSED (2) — unknown argument '{mode}'", file=sys.stderr) or 2))
PY
