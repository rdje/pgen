#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2w — the CONTAINMENT predicate, and the rule-reference graph it reads.

⛔ WHY A FOURTH INVARIANT COULD NOT BE ANOTHER IDENTITY OVER THREE TOTALS.
`PARSE-COST-RATCHET` accepts a measured RISE only under an invariant that is CODE in
`scripts/check_parse_cost_ratchet.sh` and is re-evaluated against the run's own numbers. Its three
existing invariants are exact arithmetic identities over `entries`/`memo_hits`/`committed`:

    pure_memo_lookups              Δentries ==     Δmemo  ∧  Δcommitted == 0
    unmatched_terminal_alternatives Δentries == 2 · Δmemo  ∧  Δcommitted == 0
    unmatched_lookahead_terminals   Δentries == 2 · Δmemo  ∧  Δcommitted == 0

The rise that needs accepting next fits none of them. `SV-0065` — restoring the `randomize_call`
alternative IEEE 1800 A.8.2 gives `primary`, without which `std::randomize(a,b) with { … }` is
unreachable from every expression — measures `Δentries +1,917,021`, `Δmemo +1,012,779`,
`Δcommitted 0`. `Δentries / Δmemo = 1.893`: no coded identity holds, and a fourth identity over the
same three numbers would be numerology. ⛔⛔ **Three totals cannot distinguish "the added
alternative speculates inside its own sub-graph" from "the parser now speculates everywhere", and
that distinction is the whole question.** The discriminating evidence is PER-RULE.

THE PREDICATE, stated so it can FAIL:

    containment(introduced) :=
        Δcommitted <= 0                          no accepted derivation got dearer
      ∧ no rule's entry count FELL               the change is purely additive
      ∧ every rule whose entry count ROSE is reachable, in the GRAMMAR's own reference graph,
        from one of the `introduced` rules       the rise cannot escape its subtree

⭐ IT ALREADY DISCRIMINATES, ON MEASURED ARMS — and the RED one is a real fix that a real doctrine
really refused, not a constructed control (`docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/`):

    arm0 -> t_only    introduced=data_type                     54 risers, 0 fell, 0 escaped  CONTAINED
    t_only -> designB introduced=reserved_non_keyword_identifier
                                                               82 risers, 238 fell, 82 escaped  NOT CONTAINED
                      (identifier +9,003,436 · non_keyword_identifier +8,816,751)

⚠️⚠️ THE HONEST BOUND, AND IT LIVES HERE RATHER THAN IN A COMMIT MESSAGE. Containment says the rise
is CONFINED to the construct that caused it. It does NOT say the rise was UNAVOIDABLE.
`SV-CORPUS-GRAD.13c.2k` is the standing proof that a contained-looking cost can still have a
strictly cheaper spelling — its design B and design C accept the same language at wildly different
cost — and no predicate over totals or per-rule counts can see that. Only another ARM can. So this
invariant is the right acceptance exactly when the added construct is required for the grammar to
derive its standard's language AND the cheaper spellings have been built and measured.

⛔ ONE PREDICATE, NOT TWO. `check_parse_cost_ratchet.sh` (ratchet artifacts) and
`docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/containment.py` (arm measurements) both
call `containment()` below. Coding it twice is the defect
[[one-metric-name-two-predicates-is-a-contract-defect]] names, measured in this repository three
days ago on `unreachable_rules`: one metric name, two implementations, both correct about different
populations, and a contract that could not be adjudicated.

⛔ AND THE GRAPH IS NEVER READ FROM `generated/systemverilog.json`. That is a FLOATING BUILD
ARTIFACT the arm driver rewrites for every arm; the first containment run in `.13c.2k` read the
`designB` graph while analysing the `t_only` arm — the `ENGINE-UNIVERSAL-SERVICES.20` slice-4 shape,
caught only by its own timestamp. The graph is derived from the EBNF frontend's `raw_ast` envelope
(what the code generator consumes; comments never reach it) and is STAMPED with that envelope's
digest, so a consumer can refuse a graph that describes a different grammar.

USAGE
  python3 scripts/parse_cost_containment.py --graph <grammar.ebnf> [--out <graph.json>]
  python3 scripts/parse_cost_containment.py --check <base_rule_costs.tsv> <new_rule_costs.tsv> \
          --graph-json <graph.json> --introduced RULE [RULE …] [--d-committed N] [--top N]
EXIT
  0 = contained / graph written · 1 = NOT contained · 2 = refused
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
IDENTITY = os.path.join(ROOT, "scripts", "check_baseline_identity.sh")
RULE_COSTS_HEADER = ["rule", "entries", "committed", "memo_hits"]


class Refused(Exception):
    """The predicate cannot be evaluated. ⛔ NEVER caught into a pass: an un-evaluable acceptance
    must read as a refusal, because the alternative is a rise that nobody checked reading green."""


# ── the rule-reference graph ────────────────────────────────────────────────────────────────────

def edges_from_raw_ast(raw: list) -> dict[str, list[str]]:
    """`{rule: [rules it references]}` from the EBNF frontend's `raw_ast` envelope.

    ⭐ THE POPULATION IS THE GENERATOR'S, NOT A TEXT SCAN'S. `raw_ast` is exactly what the code
    generator consumes, so a rule the frontend dropped is absent here too — which is the property
    that makes reachability in this graph mean "reachable in the parser that was measured".
    """
    edges: dict[str, list[str]] = {}
    for rule in raw:
        name = rule[0][1]
        edges[name] = sorted({t[1] for t in rule[1:] if t[0] == "rule_reference"})
    return edges


def raw_ast_of(ebnf_rel: str) -> list:
    """The frontend's `raw_ast` for one grammar, via the ONE tool that owns that envelope.

    ⛔ NOT A SECOND FRONTEND INVOCATION. `scripts/check_baseline_identity.sh` already owns the
    `ebnf_raw_ast` definition — the on-volume scratch, the `--emit-raw-ast-json` call, the canonical
    key ordering — and `BASELINE-IDENTITY`, `SV-CONTRACT-CURRENCY` and `PARSE-COST-RATCHET` all key
    on it. A local re-implementation would be a fourth copy of a definition three doctrines share,
    which is exactly how `ENGINE-UNIVERSAL-SERVICES.38` keyed one grammar by BYTES in two places.
    """
    proc = subprocess.run([IDENTITY, "--raw-ast", ebnf_rel], capture_output=True, text=True)
    if proc.returncode != 0:
        raise Refused(f"the raw_ast of `{ebnf_rel}` could not be derived "
                      f"(`check_baseline_identity.sh --raw-ast` exited {proc.returncode}): "
                      f"{(proc.stderr or '').strip()[:400]}")
    try:
        return json.loads(proc.stdout)["raw_ast"]
    except (ValueError, KeyError) as exc:
        raise Refused(f"the raw_ast envelope of `{ebnf_rel}` is not readable: {exc}")


def raw_ast_digest(ebnf_rel: str) -> str:
    """The `ebnf_raw_ast` digest of one grammar, from the same single owner."""
    proc = subprocess.run([IDENTITY, "--digest", "ebnf_raw_ast", ebnf_rel],
                          capture_output=True, text=True)
    digest = (proc.stdout or "").strip()
    if proc.returncode != 0 or len(digest) != 64:
        raise Refused(f"the ebnf_raw_ast digest of `{ebnf_rel}` could not be computed")
    return digest


def derive_graph(ebnf_rel: str) -> dict:
    """The frozen reference graph of one grammar, stamped with the envelope digest it is a
    function of."""
    edges = edges_from_raw_ast(raw_ast_of(ebnf_rel))
    return {
        "_derivation": ("the EBNF frontend's raw_ast rule-reference graph — "
                        "python3 scripts/parse_cost_containment.py --graph " + ebnf_rel),
        "grammar": ebnf_rel,
        "grammar_raw_ast_sha256": raw_ast_digest(ebnf_rel),
        "rules": len(edges),
        "edges": edges,
    }


def reachable(edges: dict[str, list[str]], starts: list[str]) -> set[str]:
    seen, stack = set(starts), list(starts)
    while stack:
        for child in edges.get(stack.pop(), ()):
            if child not in seen:
                seen.add(child)
                stack.append(child)
    return seen


def origin_rule(rule: str, known: set[str]) -> str | None:
    """The SOURCE-grammar rule a measured counter belongs to, or None when it cannot be located.

    ⛔⛔ WHY THIS EXISTS — A HOLE MEASURED IN THIS SLICE, IN THE ALREADY-PUBLISHED PREDICATE.
    The graph is the SOURCE grammar's; the counters come from the GENERATED parser, and PGEN's
    indirect-left-recursion eliminator SYNTHESISES rules that exist in no `.ebnf` file
    (`indirect_lr_elimination.rs`). Measured on the tracked `t_only` arm: **67 of the 1,077 rules
    the parser reports are absent from the 1,483-rule source graph** — every one of them an
    `_lr_base` / `_lr_seed_*` / `_lr_guard*` / `_lr_suffix*` name. A synthesised rule can never be in
    scope, so without this fold **any one of them that ROSE is escaped unconditionally**, wherever it
    actually sits (one that does not rise is not classified at all). Demonstrated, not argued: on the
    same data `casting_type_lr_base` reads `NOT CONTAINED escaped=['casting_type_lr_base']` without
    the fold and `CONTAINED escaped=0` with it, while `casting_type` provably IS inside the
    `data_type` sub-graph — so the pre-fix verdict is wrong, not merely unproven.

    ⚠️ IT NEVER FIRED, AND THAT IS WHY NOTHING CAUGHT IT. Re-measured here on the two recorded arms:
    `arm0_head -> t_only` has 54 risers and `t_only -> designB` has 82, and **zero** of either set
    is a synthesized name — so the published `0 escaped` / `82 escaped` verdicts are unaffected.
    The gap fails in the REFUSING direction, which is the safe one; a legitimate contained rise
    would simply have been rejected on the first day an LR-family rule moved.

    ⭐ THE FOLD IS GRAPH-DIRECTED, NOT NAME-SHAPE-DIRECTED. It does not re-implement the
    `_lr_(base|suffix|seed|guard|alt)` classifier the instrument ships and `--verify-families`
    gates; it asks the GRAPH — the longest `X` such that `rule` is `X_lr_…` and `X` is a rule the
    grammar has. The oracle is the population itself, so it cannot drift with the emitter's naming.
    ⛔ And it is TOTAL or it REFUSES: `containment()` below rejects the whole measurement if any
    measured rule cannot be located, rather than dropping the ones it cannot place. Verified on the
    tracked arm: all 67 fold, onto exactly `casting_type` (19) and `property_expr` (48).
    """
    if rule in known:
        return rule
    # ⛔ LONGEST PREFIX FIRST, NOT SHORTEST, and the difference is not hypothetical arithmetic: if a
    # SOURCE rule were itself named `X_lr_…`, the eliminator's children would read
    # `X_lr_…_lr_<shape>`, and a shortest-first scan would attribute them to `X` — a silent
    # mis-attribution into the WRONG sub-graph, i.e. a containment verdict about a rule that never
    # rose. ⚠️ MEASURED on this grammar: **0** source rule names contain `_lr_` at all, so the two
    # orders agree today. That is exactly why the safe order is the one written down — the day they
    # disagree, nothing would announce it.
    cuts = []
    i = rule.find("_lr_")
    while i > 0:
        cuts.append(i)
        i = rule.find("_lr_", i + 1)
    for i in reversed(cuts):
        if rule[:i] in known:
            return rule[:i]
    return None


# ── the per-rule measurement ────────────────────────────────────────────────────────────────────

def read_rule_costs(path: str) -> dict[str, dict[str, int]]:
    """`{rule: {entries, committed, memo_hits}}` from a `rule_costs.tsv`.

    ⛔ REFUSES ON AN UNEXPECTED HEADER rather than unpacking positionally. A consumer that outlives
    its producer's schema is gate-flow failure §7.8, and a positional read of a renamed column
    fails SILENTLY with a plausible number — the one failure shape this whole tree exists to remove.
    """
    if not os.path.isfile(path):
        raise Refused(f"no per-rule measurement at `{os.path.relpath(path, ROOT)}` — the "
                      f"containment predicate is per-RULE and three totals cannot stand in for it")
    with open(path, encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        if header != RULE_COSTS_HEADER:
            raise Refused(f"`{os.path.relpath(path, ROOT)}` header is {header}, expected "
                          f"{RULE_COSTS_HEADER} — fix this reader rather than trusting a "
                          f"positional unpack")
        out: dict[str, dict[str, int]] = {}
        for lineno, line in enumerate(fh, 2):
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            if len(c) != 4:
                raise Refused(f"`{os.path.relpath(path, ROOT)}`:{lineno} has {len(c)} field(s), "
                              f"expected 4")
            try:
                out[c[0]] = {"entries": int(c[1]), "committed": int(c[2]), "memo_hits": int(c[3])}
            except ValueError:
                raise Refused(f"`{os.path.relpath(path, ROOT)}`:{lineno} carries a non-integer count")
    return out


# ── the predicate ───────────────────────────────────────────────────────────────────────────────

def containment(base: dict[str, dict[str, int]], new: dict[str, dict[str, int]],
                edges: dict[str, list[str]], introduced: list[str],
                d_committed: int) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the containment predicate.

    Raises `Refused` when it cannot be evaluated — an `introduced` name the grammar does not have,
    or an empty introduced set. ⛔ Both are REFUSALS and not failures-to-hold, because a predicate
    that silently evaluates a typo'd rule name to "nothing is in scope" would refuse every rise for
    the wrong reason and teach its reader to distrust the verdict.
    """
    if not introduced:
        raise Refused("the containment predicate needs the rule set the change INTRODUCED; the "
                      "acceptance row declares none, so the scope of the rise is undeclared")
    known = set(edges) | {r for refs in edges.values() for r in refs}
    unknown = [r for r in introduced if r not in known]
    if unknown:
        raise Refused(f"{unknown} name no rule in the grammar's reference graph — a row cannot "
                      f"declare a scope the measured grammar does not have")

    scope = reachable(edges, introduced)
    names = set(base) | set(new)

    # ⛔ TOTAL OR REFUSED. Every measured rule must be locatable in the source graph, directly or
    # through the LR-eliminator fold. Dropping the ones that cannot be placed would make the
    # predicate quietly weaker exactly where the parser is least like its grammar.
    origins = {r: origin_rule(r, known) for r in names}
    unplaced = sorted(r for r, o in origins.items() if o is None)
    if unplaced:
        raise Refused(f"{len(unplaced)} measured rule(s) cannot be located in the grammar's "
                      f"reference graph, so their entries can be neither contained nor escaped: "
                      f"{unplaced[:5]}{' …' if len(unplaced) > 5 else ''}")

    deltas = {r: new.get(r, {}).get("entries", 0) - base.get(r, {}).get("entries", 0)
              for r in names}
    rose = {r: d for r, d in deltas.items() if d > 0}
    fell = {r: d for r, d in deltas.items() if d < 0}
    escaped = {r: d for r, d in rose.items() if origins[r] not in scope}

    facts = {"scope": len(scope), "rose": rose, "fell": fell, "escaped": escaped,
             "d_committed": d_committed}

    if d_committed > 0:
        return False, (f"committed moved {d_committed:+,} — an accepted derivation got dearer, "
                       f"which containment does not cover"), facts
    if fell:
        worst = sorted(fell.items(), key=lambda t: t[1])[:3]
        return False, (f"{len(fell)} rule(s) LOST entries ({sum(fell.values()):+,}) — the change "
                       f"is not purely additive, so a per-rule rise cannot be attributed to it: "
                       + ", ".join(f"{r} {d:+,}" for r, d in worst)), facts
    if escaped:
        worst = sorted(escaped.items(), key=lambda t: -t[1])[:3]
        return False, (f"{len(escaped)} rule(s) rose OUTSIDE the sub-graph reachable from "
                       f"{' '.join(introduced)} ({sum(escaped.values()):+,}) — the rise escaped "
                       f"the construct that caused it: "
                       + ", ".join(f"{r} {d:+,}" for r, d in worst)), facts
    return True, (f"{len(rose):,} rule(s) rose ({sum(rose.values()):+,}), 0 fell, 0 escaped the "
                  f"{len(scope):,}-rule sub-graph reachable from {' '.join(introduced)}, "
                  f"committed {d_committed:+,} — the rise is confined to the construct that "
                  f"caused it"), facts


def host_memo_eviction(base: dict[str, dict[str, int]], new: dict[str, dict[str, int]],
                       edges: dict[str, list[str]], introduced: list[str],
                       d_committed: int) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the MEMO-EVICTION predicate (`ENGINE-UNIVERSAL-SERVICES.88`).

    THE SHAPE. A change re-spells ONE alternative of a memoized rule H (the row's `introduced` rules
    are the new spelling). The generator then stops memoizing H: its memo hits fall to ZERO while its
    entries and commits are unchanged — the same parses arrive at H the same number of times, but the
    re-entries the memo used to answer now RE-RUN H's other alternatives. So each DIRECT alternative
    of H that already existed rises by EXACTLY the hits H lost, each rule those alternatives CONSULT
    (the layout rule a terminal reads first) rises by the lost hits times the number of re-run
    alternatives that consult it, and nothing else that already existed gets dearer. Committed does
    not move: no derivation is accepted more often, work was re-done, not re-decided.

    ⛔ WHY THE OTHER FIVE CANNOT COVER IT, measured (`SV-CORPUS-GRAD.13o`, 2026-09-01): gating the
    `edge` alternative of `edge_identifier` — as the `.13e.7` alias AND as a profile-gated terminal —
    cost `kw_posedge` +52, `kw_negedge` +52 and `trivia` +104 entries while `edge_identifier` went
    from 52 memo hits to 0. `profile_split_respelling` refuses because pre-existing rules rose;
    `contained_in_introduced_subgraph` refuses because they are SIBLINGS of the introduced rule,
    not reachable from it, and because the replaced terminal lost entries; the two arithmetic
    identities need `Δentries == Δmemo` or a terminal-alternative shape, and `+260 / +104` is
    neither. Two arms were built and both paid it: the cost is the generator's memo policy moving
    when an alternative becomes a profile-conditional reference, not the spelling.

    ⚠️⚠️ HONEST BOUND, IN THE PREDICATE'S OWN DOCSTRING: this attributes the rise to the eviction and
    proves the parser does no NEW work anywhere else. It does NOT say the eviction is unavoidable —
    that is a question about the generator's memo policy, owned by the engine leaf, and a third arm
    that keeps H memoized would be the proof. Say so in the row's `why`.

    Raises `Refused` (never a false verdict) when the scope is undeclared, names a rule the grammar
    does not have, or names a rule that already existed in the baseline.
    """
    if not introduced:
        raise Refused("the memo-eviction predicate needs the rule names the change INTRODUCED; the "
                      "acceptance row declares none, so the new spelling is unattributed")
    not_new = sorted(r for r in introduced if r in base)
    if not_new:
        raise Refused(f"{not_new} already existed in the baseline measurement — `introduced` names "
                      f"the rules the re-spelling CREATES, and listing an existing rule would turn "
                      f"this acceptance into a waiver for real work on it")
    known = set(edges) | {r for refs in edges.values() for r in refs}
    unknown = [r for r in introduced if r not in known]
    if unknown:
        raise Refused(f"{unknown} name no rule in the grammar's reference graph — a row cannot "
                      f"declare a scope the measured grammar does not have")

    facts: dict = {"d_committed": d_committed}
    if d_committed != 0:
        return False, (f"committed moved {d_committed:+,} — an eviction re-runs work, it never "
                       f"changes what is accepted, so this is not one"), facts

    both = sorted(set(base) & set(new))
    hosts = [r for r in both
             if base[r]["memo_hits"] > 0 and new[r]["memo_hits"] == 0
             and new[r]["entries"] == base[r]["entries"]
             and new[r]["committed"] == base[r]["committed"]]
    facts["hosts"] = hosts
    if len(hosts) != 1:
        if not hosts:
            return False, ("no pre-existing rule lost ALL its memo hits while keeping its entries "
                           "and commits — nothing was evicted, so the rise is not an eviction"), facts
        return False, (f"{len(hosts)} pre-existing rules lost their memo at once ({hosts[:3]}) — "
                       f"one re-spelled alternative evicts one host; this is something else"), facts
    host = hosts[0]
    lost = base[host]["memo_hits"]
    direct = set(edges.get(host, []))
    facts.update({"host": host, "lost": lost})

    dearer = {}
    for r in both:
        de = new[r]["entries"] - base[r]["entries"]
        dc = new[r]["committed"] - base[r]["committed"]
        dm = new[r]["memo_hits"] - base[r]["memo_hits"]
        if dc > 0:
            return False, (f"`{r}` committed {dc:+,} more — a pre-existing derivation got dearer, "
                           f"which an eviction cannot explain"), facts
        if de > 0 or dm > 0:
            dearer[r] = (de, dm)
    direct_dearer = sorted(r for r in dearer if r in direct)
    consulted = {}
    for r, (de, dm) in dearer.items():
        if r in direct:
            if de != lost:
                return False, (f"`{r}` is a direct alternative of `{host}` and rose {de:+,} entries, "
                               f"not the {lost:,} memo hits `{host}` lost — the re-run is not the "
                               f"whole story"), facts
            if dm != 0:
                return False, (f"`{r}` is a direct alternative of `{host}` and gained {dm:+,} memo "
                               f"hits — a re-run alternative reads its inputs again, it does not "
                               f"become memoized"), facts
            continue
        referrers = sorted(d for d in direct_dearer if r in edges.get(d, []))
        if not referrers:
            return False, (f"`{r}` rose ({de:+,} entries, {dm:+,} memo hits) and is neither an "
                           f"alternative of `{host}` nor consulted by one — the rise ESCAPED the "
                           f"evicted rule"), facts
        want = lost * len(referrers)
        if de != want or dm not in (0, want):
            return False, (f"`{r}` is consulted by {len(referrers)} re-run alternative(s) of "
                           f"`{host}` ({', '.join(referrers)}), which accounts for {want:,} entries "
                           f"and at most {want:,} memo hits, but it rose {de:+,} / {dm:+,}"), facts
        consulted[r] = referrers
    facts.update({"direct_dearer": direct_dearer, "consulted": consulted})

    undeclared = sorted(r for r in new if r not in base and r not in introduced)
    if undeclared:
        return False, (f"{len(undeclared)} rule name(s) appear in the measurement and are not "
                       f"declared as introduced, so their entries are unattributed: "
                       + " ".join(undeclared[:5])), facts
    gained = sum(new[r]["entries"] for r in new if r not in base)
    released = sum(base[r]["entries"] - new[r]["entries"] for r in both
                   if new[r]["entries"] < base[r]["entries"])
    facts.update({"gained": gained, "released": released})
    if released > gained:
        return False, (f"pre-existing rules released {released:,} entries but the introduced rule(s) "
                       f"absorbed only {gained:,} — work vanished, which an eviction cannot explain "
                       f"(and a fall is a different acceptance)"), facts

    return True, (f"memo eviction: `{host}` lost {lost:,} memo hits with entries and commits "
                  f"unchanged; {len(direct_dearer)} direct alternative(s) re-ran +{lost:,} each "
                  f"({', '.join(direct_dearer) or 'none'}); consulted rule(s) "
                  + (", ".join(f"{r} +{lost * len(v):,}" for r, v in sorted(consulted.items()))
                     or "none")
                  + f"; {len(introduced)} introduced rule(s) gained {gained:+,} entries against "
                    f"{released:,} released by the spelling they replace; committed {d_committed:+,}"), facts


def self_test_host_memo_eviction() -> int:
    """Every refusal branch of `host_memo_eviction`, fired on synthetic tables that carry the
    measured `SV-CORPUS-GRAD.13o` shape, plus the GREEN arm. Prints one line per arm."""
    def tables():
        base = {"host": {"entries": 2560, "committed": 2, "memo_hits": 52},
                "alt1": {"entries": 2508, "committed": 1, "memo_hits": 0},
                "alt2": {"entries": 2508, "committed": 1, "memo_hits": 0},
                "old_kw": {"entries": 2508, "committed": 2, "memo_hits": 0},
                "trivia": {"entries": 1000, "committed": 1000, "memo_hits": 900},
                "other": {"entries": 100, "committed": 10, "memo_hits": 5}}
        new = {k: dict(v) for k, v in base.items()}
        new["host"]["memo_hits"] = 0
        new["alt1"]["entries"] = 2560
        new["alt2"]["entries"] = 2560
        new["old_kw"] = {"entries": 0, "committed": 0, "memo_hits": 0}
        new["trivia"] = {"entries": 1104, "committed": 1000, "memo_hits": 1004}
        new["new_kw"] = {"entries": 2560, "committed": 2, "memo_hits": 52}
        edges = {"host": ["alt1", "alt2", "new_kw"], "alt1": ["trivia"], "alt2": ["trivia"],
                 "new_kw": ["trivia"], "old_kw": ["trivia"], "other": ["trivia"]}
        return base, new, edges

    arms: list[tuple[str, bool, str]] = []

    def arm(name, expect_holds, want_text, mutate=None, introduced=("new_kw",), d_committed=0,
            expect_refused=False):
        base, new, edges = tables()
        if mutate:
            mutate(base, new, edges)
        try:
            holds, detail, _ = host_memo_eviction(base, new, edges, list(introduced), d_committed)
            ok = (not expect_refused) and holds == expect_holds and want_text in detail
            got = f"holds={holds}: {detail[:110]}"
        except Refused as exc:
            ok = expect_refused and want_text in str(exc)
            got = f"REFUSED: {str(exc)[:110]}"
        arms.append((name, ok, got))
        print(f"  {'✓' if ok else '✗'} {name} — {got}")

    arm("green: the measured .13o shape HOLDS", True, "memo eviction: `host` lost 52")
    arm("red: the host's entries also rose", False, "nothing was evicted",
        mutate=lambda b, n, e: n["host"].__setitem__("entries", 2612))
    arm("red: a rise on a rule that is neither an alternative nor consulted", False, "ESCAPED",
        mutate=lambda b, n, e: n["other"].__setitem__("entries", 152))
    arm("red: two hosts lost their memo at once", False, "lost their memo at once",
        mutate=lambda b, n, e: n["other"].__setitem__("memo_hits", 0))
    arm("red: a direct alternative rose by more than the lost hits", False, "not the 52 memo hits",
        mutate=lambda b, n, e: n["alt1"].__setitem__("entries", 2570))
    arm("red: a consulted rule rose by more than lost × referrers", False, "accounts for 104",
        mutate=lambda b, n, e: n["trivia"].__setitem__("entries", 1200))
    arm("red: committed moved", False, "committed moved", d_committed=2)
    arm("red: a pre-existing rule committed more", False, "committed +1 more",
        mutate=lambda b, n, e: n["alt1"].__setitem__("committed", 2))
    arm("red: an undeclared new rule", False, "not declared as introduced",
        mutate=lambda b, n, e: n.__setitem__("stray", {"entries": 5, "committed": 0, "memo_hits": 0}))
    arm("red: more entries released than absorbed", False, "work vanished",
        mutate=lambda b, n, e: b["old_kw"].__setitem__("entries", 9000))
    arm("refused: introduced names a pre-existing rule", False, "already existed",
        introduced=("alt1",), expect_refused=True)
    arm("refused: introduced is empty", False, "declares none", introduced=(), expect_refused=True)
    arm("refused: introduced names no grammar rule", False, "name no rule",
        introduced=("no_such_rule",), expect_refused=True)
    failed = [a for a in arms if not a[1]]
    print(f"HOST-MEMO-EVICTION-SELFTEST: arms={len(arms)} failed={len(failed)}")
    return 1 if failed else 0


# ── CLI ─────────────────────────────────────────────────────────────────────────────────────────

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--graph", metavar="EBNF",
                    help="derive the reference graph of this grammar (repo-root-relative)")
    ap.add_argument("--out", help="write the derived graph here instead of stdout")
    ap.add_argument("--check", nargs=2, metavar=("BASE_TSV", "NEW_TSV"))
    ap.add_argument("--graph-json", help="the frozen graph the --check reads")
    ap.add_argument("--introduced", nargs="+", default=[])
    ap.add_argument("--d-committed", type=int, default=0)
    ap.add_argument("--top", type=int, default=15)
    ap.add_argument("--self-test", action="store_true",
                    help="fire every refusal branch of host_memo_eviction on synthetic tables")
    a = ap.parse_args()

    try:
        if a.self_test:
            return self_test_host_memo_eviction()

        if a.graph:
            g = derive_graph(a.graph)
            text = json.dumps(g, indent=1, sort_keys=True) + "\n"
            if a.out:
                with open(a.out, "w", encoding="utf-8") as fh:
                    fh.write(text)
                print(f"RULE-GRAPH: grammar={a.graph} rules={g['rules']} "
                      f"raw_ast={g['grammar_raw_ast_sha256'][:16]}… -> {a.out}")
            else:
                sys.stdout.write(text)
            return 0

        if a.check:
            if not a.graph_json:
                raise Refused("--check needs --graph-json <graph.json>")
            with open(a.graph_json, encoding="utf-8") as fh:
                edges = json.load(fh)["edges"]
            base = read_rule_costs(a.check[0])
            new = read_rule_costs(a.check[1])
            holds, detail, facts = containment(base, new, edges, a.introduced, a.d_committed)
            print(f"CONTAINMENT: introduced={' '.join(a.introduced)}")
            print(f"  sub-graph reachable from the introduced rule(s) : {facts['scope']:>6,} rules")
            print(f"  rules whose entries ROSE                        : {len(facts['rose']):>6,}  "
                  f"({sum(facts['rose'].values()):+,})")
            print(f"  rules whose entries FELL                        : {len(facts['fell']):>6,}  "
                  f"({sum(facts['fell'].values()):+,})")
            print(f"  Δcommitted                                      : {facts['d_committed']:>+7,}")
            print(f"  risers OUTSIDE that sub-graph                   : "
                  f"{len(facts['escaped']):>6,}  ({sum(facts['escaped'].values()):+,})")
            if facts["escaped"]:
                print(f"\n  the {min(a.top, len(facts['escaped']))} largest escaping rules:")
                for r, d in sorted(facts["escaped"].items(), key=lambda t: -t[1])[: a.top]:
                    print(f"    {r:<50} {base.get(r, {}).get('entries', 0):>13,} -> "
                          f"{new.get(r, {}).get('entries', 0):>13,}  {d:>+12,}")
            print(f"\n  VERDICT: {'CONTAINED' if holds else 'NOT CONTAINED'} — {detail}")
            return 0 if holds else 1
    except Refused as exc:
        print(f"parse-cost-containment: REFUSED — {exc}", file=sys.stderr)
        return 2

    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
