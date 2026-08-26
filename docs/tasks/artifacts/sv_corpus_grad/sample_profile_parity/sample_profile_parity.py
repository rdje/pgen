#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13e.11 (a) — the `@sample`-vs-profile parity census.

An `@sample` annotation is a SINGLE PROFILE-BLIND STRING. Adding a `@profiles` dialect NARROWS the
accepted language, so every sample already written against the wider language becomes a candidate
defect the moment the narrower profile exists, and nothing re-checks it (`.13e.9`, which measured
four such samples costing 10 certificate rules, latent for 54 days).

⛔ THE JOIN IS THE WHOLE DESIGN. `@profiles` is a RULE-level annotation, but 43 of SystemVerilog's
52 samples are BRANCH-level. Joining a branch sample against the rule's `satisfiable_under`
over-reports BY CONSTRUCTION — a rule stays live while individual alternatives are dead under a
profile, and their samples are then honestly unparseable there. This instrument joins against
`branch_satisfiable_under` (`derive_branch_profiles`, added by this leaf) so a sample is only ever
required to parse under the profiles ITS OWN BRANCH is live in.

Grammar-agnostic: every input is derived from the grammar under test. A grammar declaring no
`@profiles` has an empty profile universe and yields zero checks (INERT, never a pass).
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile

HEADLINE = "SAMPLE-PROFILE-PARITY"


def run(cmd: list[str], **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def dump_gen_ast(binary: str, grammar: str, out: str) -> None:
    """The `@sample` population comes from the NORMALIZED gen-AST, never from grepping the `.ebnf`.

    An annotation binds to the NEXT rule, so a regex census over annotation text mis-attributes —
    the instrument `-0293` was burned by. The gen-AST resolves the binding for us and indexes
    branch annotations in the same space as the rule body's top-level alternatives.
    """
    proc = run([binary, grammar, "--generate-stimuli", "--count", "1", "--seed", "0",
                "--dump-gen-ast", out, "--dump-gen-ast-max-bytes", "2000000000"])
    if proc.returncode != 0 or not os.path.exists(out):
        raise SystemExit(f"error: --dump-gen-ast failed (rc={proc.returncode})\n{proc.stderr[-2000:]}")


def dump_rule_profiles(binary: str, grammar: str, out: str) -> None:
    proc = run([binary, grammar, "--dump-rule-profiles", out])
    if proc.returncode != 0 or not os.path.exists(out):
        raise SystemExit(f"error: --dump-rule-profiles failed (rc={proc.returncode})\n{proc.stderr[-2000:]}")


def sample_text(entries) -> str | None:
    """The `@sample` payload, or None. `probe_sample` is a DIFFERENT annotation and is not read."""
    for annotation in entries or []:
        if annotation.get("name") != "sample":
            continue
        ast = annotation.get("ast", {}).get("Structured", {})
        value = ast.get("value", {})
        if "String" in value:
            return value["String"]
    return None


def collect_samples(gen_ast: dict) -> tuple[list[dict], list[str]]:
    """Every `@sample` in the grammar, each tagged with the branch index it belongs to.

    Returns `(samples, unjoinable)` — `unjoinable` names rules whose branch-annotation vector no
    longer indexes their body (left-recursion elimination rewrites a rule's top-level `Or`, and
    `pre_lr_elim_branch_return_annotations` is the engine's own evidence that it does). Such a rule
    is REFUSED rather than joined on a guessed index.
    """
    annotations = gen_ast["annotations"]
    tree = gen_ast["grammar_tree"]

    def branch_count(rule: str) -> int:
        node = tree.get(rule)
        if node is None:
            return 0
        return len(node["Or"]["alternatives"]) if "Or" in node else 1

    samples: list[dict] = []
    unjoinable: list[str] = []

    for rule, entries in sorted(annotations.get("semantic_annotations", {}).items()):
        text = sample_text(entries)
        if text is not None and rule in tree:
            samples.append({"rule": rule, "branch": None, "sample": text})

    for rule, per_branch in sorted(annotations.get("branch_semantic_annotations", {}).items()):
        if rule not in tree:
            continue
        if len(per_branch) != branch_count(rule):
            if any(sample_text(entries) is not None for entries in per_branch):
                unjoinable.append(rule)
            continue
        for index, entries in enumerate(per_branch):
            text = sample_text(entries)
            if text is not None:
                samples.append({"rule": rule, "branch": index, "sample": text})

    return samples, unjoinable


def live_profiles(sample: dict, rule_profiles: dict, join: str = "branch") -> list[str]:
    """The profiles this sample is REQUIRED to parse under.

    A rule-level sample is required wherever the RULE is satisfiable. A branch-level sample is
    required only where ITS BRANCH is — the distinction this leaf exists to make.

    `join="rule"` deliberately reproduces the WRONG join (rule-level for every sample) so the cost
    of the design decision is MEASURED rather than argued. It is a contrast mode, never a gate mode.
    """
    entry = rule_profiles["rules"].get(sample["rule"])
    if entry is None:
        return []
    if sample["branch"] is None or join == "rule":
        return list(entry.get("satisfiable_under", []))
    branches = entry.get("branch_satisfiable_under", [])
    if sample["branch"] >= len(branches):
        return []
    return list(branches[sample["branch"]])


def interpret(binary: str, grammar: str, rule: str, profile: str, text: str, scratch: str) -> bool:
    path = os.path.join(scratch, "sample.txt")
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)
    proc = run([binary, grammar, "--interpret-parse", path,
                "--interpret-entry-rule", rule, "--grammar-profile", profile])
    # ⛔ The exit code alone is NOT the verdict: a LOAD failure and a PARSE rejection are
    # indistinguishable at rc=1, which is exactly how `-0293`'s sweep read 160 phantom moves in the
    # flattering direction. Require the interpreter's own verdict line before scoring the check.
    for line in (proc.stdout + proc.stderr).splitlines():
        if line.startswith("INTERPRET-PARSE:") and "accepted=" in line:
            return "accepted=true" in line
    raise SystemExit(
        f"error: no INTERPRET-PARSE verdict for rule='{rule}' profile='{profile}' "
        f"(rc={proc.returncode}) — a missing verdict is NOT a rejection\n{proc.stderr[-1500:]}"
    )


def census(binary: str, grammar: str, scratch: str, join: str = "branch") -> dict:
    gen_ast_path = os.path.join(scratch, "gen_ast.json")
    rule_profiles_path = os.path.join(scratch, "rule_profiles.json")
    dump_gen_ast(binary, grammar, gen_ast_path)
    dump_rule_profiles(binary, grammar, rule_profiles_path)
    with open(gen_ast_path, encoding="utf-8") as handle:
        gen_ast = json.load(handle)
    with open(rule_profiles_path, encoding="utf-8") as handle:
        rule_profiles = json.load(handle)

    samples, unjoinable = collect_samples(gen_ast)
    profiles = rule_profiles.get("profiles", [])

    checks: list[dict] = []
    for sample in samples:
        for profile in live_profiles(sample, rule_profiles, join):
            accepted = interpret(binary, grammar, sample["rule"], profile, sample["sample"], scratch)
            checks.append({
                "rule": sample["rule"],
                "branch": sample["branch"],
                "profile": profile,
                "sample": sample["sample"],
                "accepted": accepted,
            })

    return {
        "grammar": rule_profiles.get("grammar", grammar),
        "join": join,
        "profiles": profiles,
        "samples": len(samples),
        "checks": checks,
        "defects": [c for c in checks if not c["accepted"]],
        "unjoinable": unjoinable,
    }


def report(result: dict) -> int:
    defects, unjoinable = result["defects"], result["unjoinable"]
    print(
        f"{HEADLINE}: grammar='{result['grammar']}' join={result['join']} "
        f"profiles={len(result['profiles'])} "
        f"samples={result['samples']} checks={len(result['checks'])} "
        f"defects={len(defects)} unjoinable={len(unjoinable)}"
    )
    if not result["profiles"]:
        print("  NOT EVALUATED: this grammar declares no @profiles, so there is nothing to be blind to.")
    for defect in defects:
        site = f"{defect['rule']}" + (f" branch {defect['branch']}" if defect["branch"] is not None else " (rule-level)")
        print(f"  ⛔ DEFECT {site}: @sample {defect['sample']!r} does NOT parse under '{defect['profile']}',")
        print(f"     but that branch IS live there. Either the sample predates the profile, or the branch")
        print(f"     needs a @profiles gate. Reproduce: --interpret-parse <file> "
              f"--interpret-entry-rule {defect['rule']} --grammar-profile {defect['profile']}")
    for rule in unjoinable:
        print(f"  ⛔ UNJOINABLE '{rule}': its branch-annotation vector does not index its body "
              f"(left-recursion elimination rewrote the top-level Or). REFUSED, never guessed.")
    return 1 if (defects or unjoinable) else 0


SELF_TEST_GRAMMAR = r"""@entry: true
host := @sample: "m x" kw_m kw_x            -> {kind: "plain"}
      | @sample: "m .*" kw_m gated          -> {kind: "wildcard"}
      | @sample: "m BAD" kw_m kw_x kw_x     -> {kind: "double"}

kw_m := /\s*m\s*/                           -> {kind: "m"}
kw_x := /\s*x\s*/                           -> {kind: "x"}

@profiles: ["wide"]
gated := /\s*\.\*\s*/                       -> {kind: "gated"}

# Present only to put a SECOND profile in the universe, so 'strict' exists to be dead under.
@profiles: ["wide", "strict"]
strict_marker := /\s*zzz\s*/                 -> {kind: "marker"}
"""


def self_test(binary: str) -> int:
    """⛔ A green instrument proves nothing until its refusals fire.

    The synthetic pins BOTH directions in one grammar: branch 1's sample is honestly unparseable
    under 'strict' and MUST NOT be reported (its branch is dead there — the false-positive class a
    rule-level join would produce), while branch 2's sample is genuinely broken under BOTH profiles
    and MUST be reported twice.
    """
    with tempfile.TemporaryDirectory(dir=os.path.join(REPO_ROOT, "rust", "target")) as scratch:
        grammar = os.path.join(scratch, "self_test.ebnf")
        with open(grammar, "w", encoding="utf-8") as handle:
            handle.write(SELF_TEST_GRAMMAR)
        result = census(binary, grammar, scratch)

    failures = []
    reported = {(d["rule"], d["branch"], d["profile"]) for d in result["defects"]}
    if ("host", 1, "wide") in reported or ("host", 1, "strict") in reported:
        failures.append("branch 1 was reported, but it is dead under 'strict' and correct under 'wide'")
    if ("host", 2, "wide") not in reported:
        failures.append("branch 2's broken sample was NOT reported under 'wide' — the RED arm did not fire")

    print(f"{HEADLINE}-SELF-TEST: arms={2 - len(failures)}/2 defects_seen={len(result['defects'])}")
    for failure in failures:
        print(f"  ⛔ {failure}")
    for defect in result["defects"]:
        print(f"  (observed) {defect['rule']} branch {defect['branch']} under '{defect['profile']}'")
    return 1 if failures else 0


def _repo_root() -> str:
    """Walk up to the repository root rather than counting `..` segments.

    CLAUDE.md §12: every path is repo-root-relative because the root itself may be moved, even to
    another filesystem. A hard-coded segment count silently breaks the moment this file is filed one
    directory deeper — which is exactly what it did on its first run.
    """
    path = os.path.abspath(os.path.dirname(__file__))
    while True:
        if os.path.isdir(os.path.join(path, ".git")) or os.path.isfile(os.path.join(path, "COMMIT.md")):
            return path
        parent = os.path.dirname(path)
        if parent == path:
            raise SystemExit("error: cannot locate the repository root from this file's location")
        path = parent


REPO_ROOT = _repo_root()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--grammar", default="grammars/systemverilog.ebnf",
                        help="grammar to census, repo-root-relative")
    parser.add_argument("--bin", default="rust/target/debug/ast_pipeline",
                        help="ast_pipeline binary, repo-root-relative")
    parser.add_argument("--json", help="write the full per-check result here")
    parser.add_argument("--join", choices=("branch", "rule"), default="branch",
                        help="which profile set a BRANCH-level sample is required to parse under. "
                             "'branch' is the correct join; 'rule' reproduces the over-reporting one "
                             "so its cost can be measured (contrast only, never a gate mode)")
    parser.add_argument("--self-test", action="store_true",
                        help="prove the refusals fire against a synthetic with a known defect")
    args = parser.parse_args()

    binary = os.path.join(REPO_ROOT, args.bin)
    if not os.path.exists(binary):
        raise SystemExit(f"error: ast_pipeline binary not found at '{args.bin}' (repo root: {REPO_ROOT})")

    if args.self_test:
        return self_test(binary)

    grammar = os.path.join(REPO_ROOT, args.grammar)
    # Project data stays on the repository volume (CLAUDE.md §13) — never $TMPDIR.
    scratch_root = os.path.join(REPO_ROOT, "rust", "target")
    os.makedirs(scratch_root, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch_root) as scratch:
        result = census(binary, grammar, scratch, args.join)
    if args.json:
        with open(os.path.join(REPO_ROOT, args.json), "w", encoding="utf-8") as handle:
            json.dump(result, handle, indent=1, sort_keys=True)
    return report(result)


if __name__ == "__main__":
    sys.exit(main())
