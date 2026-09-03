#!/usr/bin/env bash
# scripts/check_ebnf_alternative_parity.sh
#
# DOCTRINE `EBNF-ALTERNATIVE-PARITY` (structural + oracle) — EBNF-FRONTEND-SILENT-TRUNCATION.1.
#
#   For every rule in every tracked grammar, the number of alternatives the SOURCE declares equals
#   the number the frontend's IR carries. Mechanism-agnostic: it does not know HOW a frontend
#   drops or invents an alternative, only that it did.
#
# ⛔ WHY THIS IS A DOCTRINE, MEASURED TWICE. (1) SV-CORPUS-GRAD.3.19 (2026-08-10) lost two
# `use_clause` alternatives because a column-0 comment terminated the rule body early — the
# frontend read FEWER alternatives than the file said. (2) EBNF-FRONTEND-SILENT-TRUNCATION.6
# (2026-09-03) found the mirror image: `parseable_source_item := semi` followed by three `//`
# lines (a line comment by grammars/ebnf.ebnf:583-586, unknown to the frontend) shipped with FOUR
# alternatives — commented-out text LIVE in the released SystemVerilog parser for four months,
# reachable through `--entry-rule`. The frontend<->meta-parser envelope differential
# (LANG-CAPABILITY-AUDIT.10.6) saw that rule diverge at its annotation token and, by its own
# design, filed every token after the first divergence as UNVERIFIED — the extra alternatives
# were in the unverified tail. A per-rule COUNT is orthogonal to token position: it reads 1 vs 4.
#
# THE TWO READERS. Arm 2 is the SOURCE: the parser GENERATED from grammars/ebnf.ebnf (the
# meta-grammar is the authority on what an author can write — EBNF-SOURCE-OF-TRUTH), run by
# `ebnf_dual_run_diff --emit-ast-json`; a rule's top expression is an `alternation` whose
# `alternatives` list IS the count. Arm 1 is the IR: `ast_pipeline --emit-raw-ast-json`, the
# hand-written frontend's `raw_ast` token stream, where alternatives are the top-level
# `["operator","|"]` tokens plus one (depth over `group_open`/`group_close`). Neither arm is
# re-implemented here: two readers that already exist, one count each, compared.
#
# WHAT IT ASSERTS, over a population DERIVED on every run (`git ls-files 'grammars/*.ebnf'`):
#   (1) PAIRING — both arms produce the same rule names in the same source order (otherwise the
#       counts would compare rules that may not correspond; reported, never guessed around).
#   (2) PARITY — per rule, arm-2 alternatives == arm-1 alternatives. Any inequality FAILS, naming
#       the grammar, the rule and both counts.
#   NOT EVALUATED, printed by name and counted in the summary — never a pass: a grammar the
#   meta-parser refuses (the three `*_lrm_extracted.ebnf` snapshots are raw LRM syntax), a grammar
#   the frontend refuses, a grammar carrying an include directive (arm 1 resolves it, arm 2 stops
#   at it — the arms describe different rule sets), or a missing binary.
#
# ⚠️ HONEST LIMIT. A count is not a shape: two readers can agree on THREE alternatives and
# disagree on what is inside them — that is the envelope differential's question, not this
# gate's. And arm 2's meta-parser is the `generated/ebnf.rs` compiled into `ebnf_dual_run_diff`;
# GENERATED-REPRODUCIBILITY holds that seed equal to HEAD's grammar, this gate does not.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr; deterministic;
# mutates nothing tracked (scratch under rust/target); ~10 s on the tracked grammars (two parses of
# the SystemVerilog grammar dominate).
# Seam: `PGEN_ALTERNATIVE_PARITY_GRAMMARS` = a newline-separated grammar list replaces the derivation.
# Usage:
#   bash scripts/check_ebnf_alternative_parity.sh             # doctrine mode
#   bash scripts/check_ebnf_alternative_parity.sh --report    # every grammar, every rule pair
#   bash scripts/check_ebnf_alternative_parity.sh --self-test # the counters and the verdict on literals
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 1
exec python3 - "$@" <<'PYEOF'
import json, os, subprocess, sys, tempfile

TAG = "ebnf-alternative-parity"
PIPELINE = "rust/target/debug/ast_pipeline"
DUAL = "rust/target/debug/ebnf_dual_run_diff"
SCRATCH = "rust/target/ebnf_alternative_parity"
BUILD_HINT = ("build them: cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline "
              "--manifest-path rust/Cargo.toml && cargo build --features ebnf_dual_run "
              "--bin ebnf_dual_run_diff --manifest-path rust/Cargo.toml")


def population():
    env = os.environ.get("PGEN_ALTERNATIVE_PARITY_GRAMMARS")
    if env:
        return [l.strip() for l in open(env, encoding="utf-8") if l.strip()]
    out = subprocess.run(["git", "ls-files", "grammars/*.ebnf"], capture_output=True, text=True).stdout
    return sorted(l for l in out.splitlines() if l.strip())


# ---- the two counters, pure functions of the arms' JSON (self-tested on literals) --------------
def arm1_counts(raw_ast_envelope):
    """[(rule, alternatives)] from the frontend IR: top-level `|` operators + 1, in source order."""
    out = []
    for rule in raw_ast_envelope.get("raw_ast", []):
        name, depth, bars = None, 0, 0
        for tok in rule:
            if not (isinstance(tok, list) and len(tok) == 2 and isinstance(tok[0], str)):
                continue
            kind, val = tok
            if kind == "rule" and name is None:
                name = val
            elif kind == "group_open":
                depth += 1
            elif kind == "group_close":
                depth = max(0, depth - 1)
            elif kind == "operator" and val == "|" and depth == 0:
                bars += 1
        if name is not None:
            out.append((name, bars + 1))
    return out


INCLUDE_PREFIXES = ("include(", "include_file(", "include_dir(", "file(")  # the frontend's own spelling


def include_directives(grammar_text):
    """How many include directives the SOURCE carries, by the frontend's own rule
    (`is_include_directive`): arm 1 resolves them into the caller's rule set, arm 2 stops at
    them, so the two arms describe different rule sets and the comparison is NOT EVALUATED."""
    return sum(1 for line in grammar_text.splitlines() if line.strip().startswith(INCLUDE_PREFIXES))


def arm2_counts(typed_ast):
    """([(rule, alternatives)], includes) from the meta-parser's typed AST, in source order."""
    out, includes = [], 0
    content = typed_ast.get("content", {})
    elements = content.get("Json", {}).get("elements", []) if isinstance(content, dict) else []
    for el in elements:
        if not isinstance(el, dict):
            continue
        if "include" in str(el.get("type", "")):
            includes += 1
        rule = el.get("rule")
        if not isinstance(rule, dict):
            continue
        name = (rule.get("name") or {}).get("name")
        expr = rule.get("expression") or {}
        n = len(expr.get("alternatives", [])) if expr.get("type") == "alternation" else 1
        if name:
            out.append((name, max(1, n)))
    return out, includes


def verdict(a1, a2, includes):
    """(status, detail): 'not_evaluated' | 'pairing' | 'disagree' | 'agree'."""
    if includes:
        return "not_evaluated", f"include directive(s) {includes}: the arms describe different rule sets"
    n1, n2 = [n for n, _ in a1], [n for n, _ in a2]
    if n1 != n2:
        i = next((k for k in range(min(len(n1), len(n2))) if n1[k] != n2[k]), min(len(n1), len(n2)))
        return "pairing", (f"rule sequences differ (IR {len(n1)} rules, source {len(n2)}): first at #{i} "
                           f"IR={n1[i] if i < len(n1) else None!r} source={n2[i] if i < len(n2) else None!r}")
    bad = [(n, s, r) for (n, r), (_, s) in zip(a1, a2) if r != s]
    if bad:
        return "disagree", "; ".join(f"{n}: source {s} vs IR {r}" for n, s, r in bad[:12]) + (f" (+{len(bad)-12} more)" if len(bad) > 12 else "")
    return "agree", f"{len(a1)} rules"


# ---- the arms, from the producers -----------------------------------------------------------------
def run_arm1(grammar, out):
    p = subprocess.run([PIPELINE, grammar, "--emit-raw-ast-json", out], capture_output=True, text=True)
    if p.returncode != 0 or not os.path.exists(out):
        return None
    return json.load(open(out, encoding="utf-8"))


def run_arm2(grammar, out):
    p = subprocess.run([DUAL, "--input", grammar, "--output", out + ".report.json", "--emit-ast-json", out],
                       capture_output=True, text=True)
    if p.returncode != 0 or not os.path.exists(out):
        return None
    return json.load(open(out, encoding="utf-8"))


# ---- ground truth on literals: both counters, every verdict cell ----------------------------------
def self_test():
    ok = True
    def arm(label, cond):
        nonlocal ok
        print(("  ✓ " if cond else "  ✗ ") + label)
        ok = ok and cond
    # counters
    ir = {"raw_ast": [[["rule", "a"], ["rule_reference", "b"], ["operator", "|"], ["group_open", "("],
                       ["rule_reference", "c"], ["operator", "|"], ["rule_reference", "d"], ["group_close", ")"],
                       ["operator", "*"], ["operator", "|"], ["rule_reference", "e"], ["return_object", "{k: 1}"]],
                      [["rule", "b"], ["quoted_string", "b"]]]}
    arm("arm1 counter: nested | inside a group is not an alternative (a=3, b=1)", arm1_counts(ir) == [("a", 3), ("b", 1)])
    src = {"content": {"Json": {"elements": [
        {"type": "rule", "rule": {"name": {"name": "a"}, "expression": {"type": "alternation", "alternatives": [1, 2, 3]}}},
        {"type": "whitespace", "content": "\n"},
        {"type": "rule", "rule": {"name": {"name": "b"}, "expression": {"type": "sequence", "elements": []}}}]}}}
    arm("arm2 counter: alternation length / sequence = 1, whitespace skipped", arm2_counts(src) == ([("a", 3), ("b", 1)], 0))
    # verdicts
    arm("agree", verdict([("a", 3), ("b", 1)], [("a", 3), ("b", 1)], 0)[0] == "agree")
    # THE FOUNDING RED: parseable_source_item as the pre-fix frontend read it (4) vs the source (1)
    prefix = {"raw_ast": [[["rule", "parseable_source_item"], ["semantic_annotation", ["branch_policy", "priority_first"]],
                           ["semantic_annotation", ["priority", "[20, 6, 4, 2]"]], ["rule_reference", "semi"],
                           ["return_scalar", "{kind: \"semi\"}\n//"], ["operator", "|"],
                           ["rule_reference", "package_import_declaration"], ["regex", ""], ["operator", "|"],
                           ["rule_reference", "timeunits_declaration"], ["regex", ""], ["operator", "|"],
                           ["rule_reference", "compiler_directive"]]]}
    v = verdict(arm1_counts(prefix), [("parseable_source_item", 1)], 0)
    arm("HISTORICAL RED: the pre-fix parseable_source_item envelope reads source 1 vs IR 4", v[0] == "disagree" and "source 1 vs IR 4" in v[1])
    # the .3.19 direction: the IR LOST alternatives
    arm("RED (fewer): IR 1 vs source 3 disagrees", verdict([("use_clause", 1)], [("use_clause", 3)], 0)[0] == "disagree")
    arm("pairing: a rule sequence mismatch is reported, never compared by index", verdict([("a", 1), ("c", 1)], [("a", 1), ("b", 1)], 0)[0] == "pairing")
    arm("not_evaluated: an include directive stops the comparison", verdict([("a", 1)], [("a", 1)], 1)[0] == "not_evaluated")
    arm("include detector: the frontend's four spellings, at any indentation, nothing else",
        include_directives('include("x")\n  include_file("y")\ninclude_dir("d")\nfile("f")\na := "include(" | b\n') == 4)
    # a REAL mutation through the real arms, when the binaries exist: drop one `|` from a fresh arm-1 envelope
    if os.path.isfile(PIPELINE) and os.path.isfile(DUAL):
        os.makedirs(SCRATCH, exist_ok=True)
        g = os.path.join(SCRATCH, "selftest.ebnf")
        open(g, "w").write('a := b | c "x"\n   | (d | e)* -> {kind: "y"}\nb := "b"\nc := "c"\nd := "d"\ne := "e"\n')
        e1 = run_arm1(g, os.path.join(SCRATCH, "st1.json")); e2 = run_arm2(g, os.path.join(SCRATCH, "st2.json"))
        if e1 is None or e2 is None:
            arm("real arms on the synthetic grammar (both binaries ran)", False)
        else:
            c2, inc = arm2_counts(e2)
            arm("real arms agree on the synthetic grammar (a=3, b..e=1)", verdict(arm1_counts(e1), c2, inc)[0] == "agree" and dict(c2)["a"] == 3)
            mutated = json.loads(json.dumps(e1))
            toks = mutated["raw_ast"][0]; k = next(i for i, t in enumerate(toks) if t == ["operator", "|"]); del toks[k]
            arm("RED through the real arms: dropping one top-level `|` from the IR is caught (source 3 vs IR 2)",
                "source 3 vs IR 2" in verdict(arm1_counts(mutated), c2, inc)[1])
    else:
        print("  NOT EVALUATED: real-arm controls need both binaries — " + BUILD_HINT)
    print(f"{TAG}: self-test {'OK' if ok else 'FAILED'}")
    sys.exit(0 if ok else 1)


args = sys.argv[1:]
if "--self-test" in args:
    self_test()
report = "--report" in args

missing = [b for b in (PIPELINE, DUAL) if not os.path.isfile(b)]
if missing:
    print(f"{TAG}: NOT EVALUATED — {', '.join(missing)} not built; nothing was compared. {BUILD_HINT}", file=sys.stderr)
    print(f"EBNF-ALTERNATIVE-PARITY: NOT EVALUATED (missing binaries: {len(missing)})")
    sys.exit(0)

os.makedirs(SCRATCH, exist_ok=True)
grammars = population()
if not grammars:
    print(f"{TAG}: REFUSING — the population is EMPTY (git ls-files 'grammars/*.ebnf' returned nothing)", file=sys.stderr)
    sys.exit(2)

fails, not_eval, agree_rules, agree_grammars = [], [], 0, 0
for g in grammars:
    stem = os.path.join(SCRATCH, os.path.basename(g))
    e1 = run_arm1(g, stem + ".arm1.json")
    e2 = run_arm2(g, stem + ".arm2.json")
    if e1 is None or e2 is None:
        who = " and ".join(w for w, e in (("frontend", e1), ("meta-parser", e2)) if e is None)
        not_eval.append((g, f"refused by the {who}"))
        if report: print(f"  not_evaluated  {g}: refused by the {who}")
        continue
    c2, inc = arm2_counts(e2)
    inc += include_directives(open(g, encoding="utf-8", errors="replace").read())
    status, detail = verdict(arm1_counts(e1), c2, inc)
    if status == "agree":
        agree_grammars += 1; agree_rules += len(c2)
        if report: print(f"  agree          {g}: {detail}")
    elif status == "not_evaluated":
        not_eval.append((g, detail))
        if report: print(f"  not_evaluated  {g}: {detail}")
    else:
        fails.append((g, status, detail))
        if report: print(f"  {status:<14} {g}: {detail}")

for g, why in not_eval:
    print(f"{TAG}: NOT EVALUATED — {g}: {why}", file=sys.stderr)
if not fails and agree_grammars == 0:
    # A check whose every input was NOT EVALUATED has not been tested: nothing was compared.
    print(f"{TAG}: REFUSING — nothing was compared: every grammar in the population was NOT EVALUATED "
          f"({len(not_eval)} of {len(grammars)})", file=sys.stderr)
    sys.exit(2)
if fails:
    print(f"{TAG}: {len(fails)} grammar(s) where the SOURCE and the IR disagree on alternatives:", file=sys.stderr)
    for g, status, detail in fails:
        print(f"  ✗ {g} [{status}]: {detail}", file=sys.stderr)
    print(f"{TAG}: a dropped alternative is a silent language loss; an invented one is text the author "
          "commented out, shipping. Read the rule in the grammar and the frontend's reading of it "
          "(ast_pipeline <g> --emit-raw-ast-json / ebnf_dual_run_diff --emit-ast-json).", file=sys.stderr)
    sys.exit(1)
print(f"EBNF-ALTERNATIVE-PARITY: OK (grammars={agree_grammars} rules={agree_rules} not_evaluated={len(not_eval)}"
      f"{' [' + ', '.join(os.path.basename(g) for g, _ in not_eval) + ']' if not_eval else ''})")
PYEOF
