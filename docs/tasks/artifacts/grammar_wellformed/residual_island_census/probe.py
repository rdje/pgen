#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.1 — partition a grammar's entry-unreachable residual into ISLANDS,
and split PGEN's own LR-elimination residue from rules that are orphaned in the SOURCE grammar.

`--report-certificate-coverage` names the residual (`N UNKNOWN rules have NO reach path from the
entry (dead-rule candidates — adjudicate via the linter)`), and `PGEN_CERT_RESIDUAL_CLASSIFICATION`
sorts it into `profile_entry_unreachable` / `store_unproducible_under_profile` / `genuine`. Neither
answers the question an adjudicator actually has to answer:

    is this rule orphaned because the GRAMMAR never wires it, or because PGEN'S OWN
    left-recursion elimination rewrote its referrer and left the original behind?

⛔ `--lint-grammar` cannot answer it either, and reads a reassuring `unreachable_rules=0` over this
whole population BY CONSTRUCTION: `detect_unreachable_rules`
(`rust/src/ast_pipeline/grammar_wellformedness.rs`) roots reachability at the canonical entry PLUS
every rule NOTHING references, so an unreferenced orphan is a ROOT, never an "unreachable" — and a
dead island hanging off one inherits that pass. The lint is *correct on its own terms* (it is
deliberately multi-entry-safe) and *structurally blind* to exactly the rules this census is for.

The two arms, both authoritative dumps of the same grammar, both produced by `ast_pipeline`:

  PRE  `--emit-raw-ast-json`  the frontend's raw token-list AST — the grammar AS WRITTEN.
  POST `--dump-gen-ast`       the normalized generation-input AST — what codegen and the
                              certificate-coverage pass actually consume, i.e. AFTER the
                              LR-elimination pass has rewritten referrers.

A rule outside the entry closure in POST but inside it in PRE is **LR residue**: the source wires
it, the engine's own rewrite orphaned it. A rule outside in BOTH is a **source orphan**. The delta
is the whole point — reading POST alone records an engine artifact as a grammar fact.

Usage:
    python3 docs/tasks/artifacts/grammar_wellformed/residual_island_census/probe.py \
        --grammar ebnf --entry grammar_file [--raw PATH --gen PATH]

With no `--raw`/`--gen` the dumps are produced into a scratch dir under `rust/target/` by invoking
`./rust/target/debug/ast_pipeline` (build it first: `cargo build --features "generated_parsers
ebnf_dual_run"` from `rust/`).

⛔ NOT registered in `DIAGNOSIS_SIG` (`scripts/check_diagnosis_evidence.sh`): this instrument sizes
and partitions a population, it does not root-cause a defect. The diagnosing tools for `H.16` are
`--report-certificate-coverage` (+ `PGEN_CERT_RESIDUAL_CLASSIFICATION`, `PGEN_CERT_COVERAGE_DUMP_ALL`,
`PGEN_CERT_COVERAGE_DEBUG_PROBES`) and `--interpret-parse`.

Exit: 0 = census produced · 2 = harness refused (an input could not be read or a dump could not be
      produced, so NOTHING was scored — never conflate a refusal with a clean census).
"""

import argparse
import collections
import json
import os
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", ".."))
PIPELINE = os.path.join(ROOT, "rust/target/debug/ast_pipeline")


def refuse(msg):
    print("RESIDUAL-ISLAND-CENSUS: REFUSED — %s (nothing was scored)" % msg, file=sys.stderr)
    sys.exit(2)


# ---------------------------------------------------------------- PRE arm (raw frontend AST)

def load_pre(path):
    """Referrer graph of the grammar AS WRITTEN. `raw_ast` is a list of rules, each a flat token
    list whose first `["rule", NAME]` names it and whose `["rule_reference", X]` tokens are its
    edges. References to rules this grammar does not define (includes / codegen builtins) are
    dropped — they are not nodes of this graph."""
    try:
        raw = json.load(open(path))["raw_ast"]
    except Exception as exc:  # noqa: BLE001 — any unreadable input is a refusal, not a verdict
        refuse("cannot read raw-AST dump %s (%s)" % (path, exc))
    edges, order = {}, []
    for entry in raw:
        name = None
        refs = []
        for tok in entry:
            if not (isinstance(tok, list) and len(tok) >= 2):
                continue
            if tok[0] == "rule" and name is None:
                name = tok[1]
            elif tok[0] == "rule_reference":
                refs.append(tok[1])
        if name is None:
            continue
        if name not in edges:
            edges[name] = set()
            order.append(name)
        edges[name] |= set(refs)
    return {r: {t for t in s if t in edges} for r, s in edges.items()}, order


# --------------------------------------------------------------- POST arm (generation-input AST)

def _gen_refs(node, out):
    if isinstance(node, dict):
        for key, val in node.items():
            if key == "Atom":
                inner = val.get("value", {})
                tok = inner.get("Token")
                if isinstance(tok, list) and len(tok) == 2:
                    kind = tok[0].get("String") if isinstance(tok[0], dict) else None
                    name = tok[1].get("String") if isinstance(tok[1], dict) else None
                    if kind == "rule_reference" and name:
                        out.add(name)
                _gen_refs(inner, out)
            else:
                _gen_refs(val, out)
    elif isinstance(node, list):
        for element in node:
            _gen_refs(element, out)


def load_post(path):
    try:
        dump = json.load(open(path))
        tree, order = dump["grammar_tree"], dump["rule_order"]
    except Exception as exc:  # noqa: BLE001
        refuse("cannot read gen-AST dump %s (%s)" % (path, exc))
    edges = {}
    for rule, body in tree.items():
        seen = set()
        _gen_refs(body, seen)
        edges[rule] = {t for t in seen if t in tree}
    return edges, [r for r in order if r in tree]


# ---------------------------------------------------------------------------------- analysis

def closure(edges, roots):
    seen, stack = set(), list(roots)
    while stack:
        rule = stack.pop()
        if rule in seen or rule not in edges:
            continue
        seen.add(rule)
        stack.extend(sorted(edges[rule]))
    return seen


def islands(edges, order, entry):
    """Every rule outside the entry closure, grouped under the unreferenced ROOT that reaches it.
    Returns (entry_closure, [(root, [members…])…], uncovered)."""
    referrers = collections.defaultdict(set)
    for rule, targets in edges.items():
        for target in targets:
            referrers[target].add(rule)
    reached = closure(edges, [entry])
    rank = {r: i for i, r in enumerate(order)}
    groups, covered = [], set()
    for root in order:
        if referrers[root] or root in reached:
            continue
        members = closure(edges, [root]) - reached
        if members:
            groups.append((root, sorted(members, key=lambda r: rank.get(r, 1 << 30))))
            covered |= members
    uncovered = [r for r in order if r not in reached and r not in covered]
    return reached, groups, uncovered


def dump(grammar, entry, path, flag, out_dir):
    target = os.path.join(out_dir, "%s_%s.json" % (grammar, "raw" if flag == "raw" else "gen"))
    if not os.path.exists(PIPELINE):
        refuse("%s is missing — build it with `cargo build --features \"generated_parsers "
               "ebnf_dual_run\"` from rust/" % PIPELINE)
    if flag == "raw":
        cmd = [PIPELINE, path, "--emit-raw-ast-json", target]
    else:
        cmd = [PIPELINE, path, "--generate-stimuli", "--count", "1", "--seed", "0",
               "--dump-gen-ast", target, "-o", os.devnull]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if not os.path.exists(target):
        refuse("`%s` produced no dump (rc=%d): %s" % (" ".join(cmd), proc.returncode,
                                                      proc.stderr.strip()[:400]))
    return target


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--grammar", required=True, help="grammar name, e.g. `ebnf`")
    ap.add_argument("--entry", required=True, help="declared entry rule, e.g. `grammar_file`")
    ap.add_argument("--raw", help="pre-elimination raw-AST dump (default: produce one)")
    ap.add_argument("--gen", help="post-elimination gen-AST dump (default: produce one)")
    args = ap.parse_args()

    out_dir = os.path.join(ROOT, "rust/target/residual_island_census")
    os.makedirs(out_dir, exist_ok=True)
    source = os.path.join(ROOT, "grammars", "%s.ebnf" % args.grammar)
    if not (args.raw and args.gen) and not os.path.exists(source):
        refuse("no grammar at %s and no explicit --raw/--gen given" % source)

    pre_edges, pre_order = load_pre(args.raw or dump(args.grammar, args.entry, source, "raw", out_dir))
    post_edges, post_order = load_post(args.gen or dump(args.grammar, args.entry, source, "gen", out_dir))

    # ⛔ A MISNAMED ENTRY MUST REFUSE, NOT REPORT. Without this the census answers a typo with
    # `post_outside == post_rules` — "every rule is orphaned" — which is a plausible-looking
    # reading that is pure harness error, and is exactly the shape of result this repo refuses to
    # let an instrument return silently.
    for arm, order in (("raw", pre_order), ("gen", post_order)):
        if args.entry not in order:
            refuse("entry rule '%s' is not defined in the %s arm of grammar '%s'"
                   % (args.entry, arm, args.grammar))

    pre_reached, pre_groups, pre_left = islands(pre_edges, pre_order, args.entry)
    post_reached, post_groups, post_left = islands(post_edges, post_order, args.entry)

    pre_outside = {r for r in pre_order if r not in pre_reached}
    post_outside = {r for r in post_order if r not in post_reached}
    lr_residue = sorted(post_outside - pre_outside, key=post_order.index)
    source_orphans = sorted(post_outside & pre_outside, key=post_order.index)
    re_wired = sorted(pre_outside - post_outside, key=pre_order.index)

    print("RESIDUAL-ISLAND-CENSUS: grammar='%s' entry='%s' pre_rules=%d post_rules=%d "
          "pre_outside=%d post_outside=%d lr_residue=%d source_orphans=%d re_wired=%d islands=%d"
          % (args.grammar, args.entry, len(pre_order), len(post_order), len(pre_outside),
             len(post_outside), len(lr_residue), len(source_orphans), len(re_wired),
             len(post_groups)))
    print("  lr_residue (reachable in the SOURCE; orphaned by PGEN's own LR elimination): %s"
          % lr_residue)
    print("  source_orphans (outside the entry closure in BOTH arms): %s" % source_orphans)
    if re_wired:
        print("  re_wired (outside PRE, inside POST — normalization ADDED a referrer): %s" % re_wired)
    for root, members in post_groups:
        kind = "LR-RESIDUE" if root in set(lr_residue) else "SOURCE-ORPHAN"
        print("  island root='%s' [%s] size=%d: %s" % (root, kind, len(members), members))
    if post_left:
        print("  ⛔ outside the entry closure and reached by NO orphan root — the partition is NOT "
              "closed, adjudicate these by hand: %s" % post_left)
    return 0


if __name__ == "__main__":
    sys.exit(main())
