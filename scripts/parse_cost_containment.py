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


def caller_split_containment(base: dict[str, dict[str, int]], new: dict[str, dict[str, int]],
                             edges: dict[str, list[str]], introduced: list[str],
                             d_committed: int) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the CALLER-SPLIT predicate (`SV-CORPUS-GRAD.3.31`).

    THE SHAPE. A rule R that several callers share is SPLIT BY CALLER: the row's `introduced` rules
    take over R's call sites in one family of callers (here the hierarchical-name chains take
    `hierarchical_bit_select` where they used `constant_bit_select`) and widen what those sites
    admit, while R keeps its other callers unchanged. Work MOVES: R's entries and committed fall by
    exactly what the introduced rules gain in committed, and — because the introduced rules ADD
    alternatives the old rule did not try — pre-existing rules the introduced rules consult rise
    (the operand tries, the layout rule a terminal reads first, the loop's own brackets). Measured
    on `.3.31`: `constant_bit_select` entries -881,589 / committed -38,027 against
    `hierarchical_bit_select` +881,591 / +38,027, and `lbrack` +357,103, `trivia` +120,109,
    `inc_or_dec_expression` +71,510, `unary_operator` +69,885 … all reachable from the three
    introduced rules.

    ⛔ WHY THE OTHER SIX CANNOT COVER IT. `contained_in_introduced_subgraph` refuses ANY rule losing
    entries (a split is defined by the old rule losing them); `profile_split_respelling` refuses ANY
    pre-existing rule rising (the widened alternatives make the consulted rules rise — real work,
    not bookkeeping); the three arithmetic identities need shapes this is not; `host_memo_eviction`
    needs a host whose memo hits fall to zero at unchanged entries.

    ⭐ WHAT THIS ONE ASSERTS. (1) `committed` is CONSERVED in total, no pre-existing rule's committed
    ROSE, and the committed the pre-existing rules LOST equals what the introduced rules gained —
    no derivation is accepted more often anywhere, the accepted work only moved onto the new names;
    (2) every rule whose ENTRIES rose is reachable, in the measured grammar's own reference graph,
    from an introduced rule — the extra speculation sits inside the construct that caused it, never
    "everywhere". Fallers are REPORTED and never refused: with every riser attributed one by one, a
    fall cannot hide a rise by netting, and a split without fallers is not a split.

    ⚠️⚠️ HONEST BOUND, IN THE PREDICATE'S OWN DOCSTRING: this proves the rise is CONFINED and that
    accepted work only moved. It does NOT prove the rise is unavoidable: half of it is the split
    rule no longer sharing the memo with its constant twin at the same position (a memo keyed on
    the RULE, not the body), the other half the tournament trying the new operand kinds at every
    index. Both are engine properties (`ENGINE-UNIVERSAL-SERVICES.15`/`.17`: commit-once, no
    give-back); say so in the row's `why` and route their elimination.

    Raises `Refused` (never a false verdict) when the scope is undeclared, names a rule the grammar
    does not have, names a rule that already existed, or a measured rule cannot be placed in the graph.
    """
    if not introduced:
        raise Refused("the caller-split predicate needs the rule names the change INTRODUCED; the "
                      "acceptance row declares none, so the moved work is unattributed")
    not_new = sorted(r for r in introduced if r in base)
    if not_new:
        raise Refused(f"{not_new} already existed in the baseline measurement — `introduced` names "
                      f"the rules the split CREATES, and listing an existing rule would turn this "
                      f"acceptance into a waiver for real work on it")
    known = set(edges) | {r for refs in edges.values() for r in refs}
    unknown = [r for r in introduced if r not in known]
    if unknown:
        raise Refused(f"{unknown} name no rule in the grammar's reference graph — a row cannot "
                      f"declare a scope the measured grammar does not have")
    scope = reachable(edges, introduced)
    names = set(base) | set(new)
    origins = {r: origin_rule(r, known) for r in names}
    unplaced = sorted(r for r, o in origins.items() if o is None)
    if unplaced:
        raise Refused(f"{len(unplaced)} measured rule(s) cannot be located in the grammar's "
                      f"reference graph, so their entries can be neither contained nor escaped: "
                      f"{unplaced[:5]}{' …' if len(unplaced) > 5 else ''}")
    d_entries = {r: new.get(r, {}).get("entries", 0) - base.get(r, {}).get("entries", 0) for r in names}
    d_comm = {r: new.get(r, {}).get("committed", 0) - base.get(r, {}).get("committed", 0) for r in names}
    rose = {r: d for r, d in d_entries.items() if d > 0}
    fell = {r: d for r, d in d_entries.items() if d < 0}
    escaped = {r: d for r, d in rose.items() if origins[r] not in scope}
    pre = [r for r in names if r in base]
    pre_comm_rose = {r: d_comm[r] for r in pre if d_comm[r] > 0}
    pre_comm_lost = -sum(d_comm[r] for r in pre if d_comm[r] < 0)
    new_comm_gained = sum(d_comm[r] for r in names if r not in base and d_comm[r] > 0)
    facts = {"scope": len(scope), "rose": rose, "fell": fell, "escaped": escaped,
             "d_committed": d_committed, "pre_committed_rose": pre_comm_rose,
             "committed_moved": pre_comm_lost, "committed_gained": new_comm_gained}
    if d_committed != 0:
        return False, (f"committed moved {d_committed:+,} in total — accepted work was re-decided, "
                       f"not moved between the split pair"), facts
    if pre_comm_rose:
        worst = sorted(pre_comm_rose.items(), key=lambda t: -t[1])[:3]
        return False, (f"{len(pre_comm_rose)} pre-existing rule(s) got dearer on `committed` — a "
                       f"derivation is accepted more often where it already was: "
                       + ", ".join(f"{r} {d:+,}" for r, d in worst)), facts
    if pre_comm_lost != new_comm_gained:
        return False, (f"the pre-existing rules lost {pre_comm_lost:,} committed while the "
                       f"introduced rules gained {new_comm_gained:,} — accepted work did not simply "
                       f"move onto the new names"), facts
    if escaped:
        worst = sorted(escaped.items(), key=lambda t: -t[1])[:3]
        return False, (f"{len(escaped)} rule(s) rose OUTSIDE the sub-graph reachable from "
                       f"{' '.join(introduced)} ({sum(escaped.values()):+,}) — the rise escaped "
                       f"the construct that caused it: "
                       + ", ".join(f"{r} {d:+,}" for r, d in worst)), facts
    worst_fell = sorted(fell.items(), key=lambda t: t[1])[:2]
    return True, (f"caller split: {len(rose):,} rule(s) rose ({sum(rose.values()):+,}), all inside "
                  f"the {len(scope):,}-rule sub-graph reachable from {' '.join(introduced)}; "
                  f"{len(fell):,} fell ({sum(fell.values()):+,}, largest "
                  + ", ".join(f"{r} {d:+,}" for r, d in worst_fell)
                  + f"); committed conserved — {pre_comm_lost:,} moved from the replaced rule(s) "
                  f"onto the introduced ones, 0 pre-existing rule accepted more often"), facts


ENTRIES_HEADER_V9 = ["sub_corpus", "tier", "path", "accepted", "entries", "committed", "memo_hits",
                     "lr_entries", "lr_committed"]
ENTRIES_HEADER_V10 = ENTRIES_HEADER_V9 + ["furthest"]


def read_entries(path: str) -> dict[str, dict]:
    """`{path: {accepted: "yes"|"no", entries, committed, memo_hits, furthest: int|None}}` from an
    `entries.tsv`. Accepts the 9-column table every baseline before `SV-CORPUS-GRAD.3.34` wrote
    (`furthest` is then None — unknown, never zero) and the 10-column one written since; REFUSES any
    other header rather than unpacking positionally."""
    if not os.path.isfile(path):
        raise Refused(f"no per-file measurement at `{os.path.relpath(path, ROOT)}` — the advanced-files "
                      f"predicate is per-FILE and the sample totals cannot stand in for it")
    out = {}
    with open(path, encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        if header not in (ENTRIES_HEADER_V9, ENTRIES_HEADER_V10):
            raise Refused(f"`{os.path.relpath(path, ROOT)}` header is {header}, expected "
                          f"{ENTRIES_HEADER_V10} (or the 9-column form before it) — fix this reader "
                          f"rather than trusting a positional unpack")
        idx = {n: i for i, n in enumerate(header)}
        for line in fh:
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            fur = None
            if "furthest" in idx and c[idx["furthest"]].isdigit():
                fur = int(c[idx["furthest"]])
            out[c[idx["path"]]] = {"accepted": c[idx["accepted"]], "entries": int(c[idx["entries"]]),
                                   "committed": int(c[idx["committed"]]), "memo_hits": int(c[idx["memo_hits"]]),
                                   "furthest": fur}
    return out


def advanced_files_only(base: dict[str, dict], new: dict[str, dict],
                        scope: list[str]) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the ADVANCED-FILES predicate (`SV-CORPUS-GRAD.3.34`).

    THE SHAPE. A widening lets a sampled file PARSE FURTHER — it flips to accepted, or it is still
    rejected but its furthest position moved forward — and the extra text it now parses costs
    entries and memo hits, spread over whatever rules that text reaches (815 risers on `.3.34`, none
    of them attributable to the changed rule's sub-graph: the parse simply went on). Measured:
    `slang/tests/regression/all.sv` furthest 3 478 → 5 694, entries +147,484, memo +68,591; the other
    191 sampled files byte-identical in every column.

    ⛔ WHY THE OTHER SEVEN CANNOT COVER IT. The sub-graph invariants (`contained_in_introduced_subgraph`,
    `caller_split_containment`) need INTRODUCED rules and risers confined to what they reach — a
    widening by one alternative introduces no rule and the risers are the whole grammar downstream of
    the advance; `profile_split_respelling` refuses any pre-existing riser; `host_memo_eviction` needs
    a host with unchanged entries; the three arithmetic identities need shapes this is not.

    ⭐ WHAT THIS ONE ASSERTS — a per-FILE conservation law. (1) Every file whose entries, committed or
    memo hits moved is named in the row's scope; (2) every scoped file ADVANCED: accepted `no → yes`,
    or still `no` with a furthest position strictly greater than the baseline's; (3) every file NOT in
    scope is byte-identical in every cost column. So the rise is exactly the cost of text the parser
    now reaches in the files it now reaches it in, and nothing else in the sample got dearer.

    ⚠️ HONEST BOUND. This proves the rise is CONFINED to the advanced files and equals their advance;
    it does not price the advance (that is the corpus's text). When the BASELINE table predates the
    `furthest` column (9 columns), a still-rejected scoped file's advance cannot be verified here: the
    predicate then REQUIRES the new table to carry a furthest position for it and records in `facts`
    that the baseline side is unknown — the next baseline carries the column and the check is mechanical.

    Raises `Refused` (never a false verdict) when the scope is empty or names a path the sample does
    not have.
    """
    if not scope:
        raise Refused("the advanced-files predicate needs the sample PATH(S) whose parse advanced; the "
                      "acceptance row declares none, so the rise is unattributed")
    unknown = [s for s in scope if s not in base or s not in new]
    if unknown:
        raise Refused(f"scope names path(s) the sample does not have: {unknown[:3]}")
    moved = sorted(k for k in base if k in new and any(base[k][c] != new[k][c] for c in ("entries", "committed", "memo_hits")))
    unscoped_moved = [k for k in moved if k not in scope]
    not_advanced, legacy = [], []
    for k in scope:
        b, n = base[k], new[k]
        if b["accepted"] == "no" and n["accepted"] == "yes":
            continue
        if n["accepted"] == "no" and n["furthest"] is not None:
            if b["furthest"] is None:
                legacy.append(k)
                continue
            if n["furthest"] > b["furthest"]:
                continue
        not_advanced.append(k)
    facts = {"moved_files": moved, "unscoped_moved": unscoped_moved, "not_advanced": not_advanced,
             "baseline_furthest_unknown": legacy,
             "advance": {k: (base[k]["accepted"], new[k]["accepted"], base[k]["furthest"], new[k]["furthest"],
                             new[k]["entries"] - base[k]["entries"], new[k]["memo_hits"] - base[k]["memo_hits"]) for k in scope}}
    if unscoped_moved:
        return (False, f"{len(unscoped_moved)} file(s) outside the declared scope changed cost "
                       f"(e.g. {unscoped_moved[0]}) — the rise is not confined to the advanced files", facts)
    if not_advanced:
        return (False, f"scoped file(s) did NOT advance (same verdict, furthest not forward): {not_advanced[:3]}", facts)
    if not moved:
        return (False, "no sampled file changed cost, so there is no rise for this predicate to explain", facts)
    detail = (f"the rise is confined to {len(scope)} advanced file(s): " +
              "; ".join(f"{os.path.basename(k)} accepted {v[0]}->{v[1]} furthest {v[2]}->{v[3]} entries {v[4]:+,} memo {v[5]:+,}"
                        for k, v in facts["advance"].items()) +
              (f" — baseline furthest UNKNOWN for {len(legacy)} (9-column baseline; verified from the next baseline on)" if legacy else "") +
              f"; {len(base) - len(scope)} other sampled file(s) byte-identical")
    return (True, detail, facts)


def self_test_advanced_files_only() -> int:
    """Arms for the ADVANCED-FILES predicate — every refusal and every failure shape, no parser."""
    def row(acc, e, c, m, fur):
        return {"accepted": acc, "entries": e, "committed": c, "memo_hits": m, "furthest": fur}
    base = {"a.sv": row("no", 100, 10, 40, 300), "b.sv": row("yes", 200, 20, 80, None), "c.sv": row("no", 50, 5, 20, 90)}
    arms, failed = [], []
    def arm(name, cond):
        arms.append(name); print(f"  {'✓' if cond else '✗'} {name}")
        if not cond: failed.append(name)
    new = {"a.sv": row("no", 160, 12, 60, 520), "b.sv": dict(base["b.sv"]), "c.sv": dict(base["c.sv"])}
    h, d, f = advanced_files_only(base, new, ["a.sv"]); arm("holds: one still-rejected file advanced (furthest forward), others byte-identical", h and f["moved_files"] == ["a.sv"])
    new2 = {"a.sv": row("yes", 160, 12, 60, None), "b.sv": dict(base["b.sv"]), "c.sv": dict(base["c.sv"])}
    h, d, f = advanced_files_only(base, new2, ["a.sv"]); arm("holds: a file that flipped no -> yes advanced", h)
    new3 = {"a.sv": row("no", 160, 12, 60, 520), "b.sv": row("yes", 201, 20, 80, None), "c.sv": dict(base["c.sv"])}
    h, d, f = advanced_files_only(base, new3, ["a.sv"]); arm("fails: an UNSCOPED file changed cost", not h and f["unscoped_moved"] == ["b.sv"])
    new4 = {"a.sv": row("no", 160, 12, 60, 300), "b.sv": dict(base["b.sv"]), "c.sv": dict(base["c.sv"])}
    h, d, f = advanced_files_only(base, new4, ["a.sv"]); arm("fails: a scoped file got dearer WITHOUT advancing (same furthest)", not h and f["not_advanced"] == ["a.sv"])
    new5 = {"a.sv": row("no", 160, 12, 60, 250), "b.sv": dict(base["b.sv"]), "c.sv": dict(base["c.sv"])}
    h, d, f = advanced_files_only(base, new5, ["a.sv"]); arm("fails: a scoped file whose furthest moved BACKWARD", not h)
    base9 = {k: dict(v, furthest=None) for k, v in base.items()}
    h, d, f = advanced_files_only(base9, new, ["a.sv"]); arm("holds with the baseline-unknown fact on a 9-column baseline when the new table carries the furthest", h and f["baseline_furthest_unknown"] == ["a.sv"])
    new6 = {"a.sv": row("no", 160, 12, 60, None), "b.sv": dict(base["b.sv"]), "c.sv": dict(base["c.sv"])}
    h, d, f = advanced_files_only(base9, new6, ["a.sv"]); arm("fails: still rejected and NO furthest on either side — the advance is unverifiable", not h)
    try:
        advanced_files_only(base, new, []); arm("refuse: empty scope", False)
    except Refused: arm("refuse: empty scope", True)
    try:
        advanced_files_only(base, new, ["zz.sv"]); arm("refuse: a scope path the sample lacks", False)
    except Refused: arm("refuse: a scope path the sample lacks", True)
    h, d, f = advanced_files_only(base, {k: dict(v) for k, v in base.items()}, ["a.sv"]); arm("fails: nothing moved — no rise to explain", not h)
    print(f"ADVANCED-FILES-ONLY-SELFTEST: arms={len(arms)} failed={len(failed)}")
    return 1 if failed else 0


def self_test_caller_split_containment() -> int:
    """Every refusal and failure branch of `caller_split_containment`, fired on synthetic tables
    that carry the measured `SV-CORPUS-GRAD.3.31` shape, plus the GREEN arm."""
    def tables():
        base = {"old_sel": {"entries": 1000, "committed": 40, "memo_hits": 990},
                "caller": {"entries": 500, "committed": 100, "memo_hits": 10},
                "lbrack": {"entries": 200, "committed": 50, "memo_hits": 100},
                "expr": {"entries": 300, "committed": 60, "memo_hits": 100},
                "other": {"entries": 100, "committed": 10, "memo_hits": 5}}
        new = {k: dict(v) for k, v in base.items()}
        new["old_sel"] = {"entries": 100, "committed": 4, "memo_hits": 90}   # the split: 900 entries / 36 committed move
        new["new_sel"] = {"entries": 900, "committed": 36, "memo_hits": 500}
        new["new_ix"] = {"entries": 80, "committed": 0, "memo_hits": 0}
        new["lbrack"]["entries"] = 560                                        # the loop's own bracket, tried more often
        new["expr"]["entries"] = 380                                          # the widened operand tries
        edges = {"caller": ["new_sel", "old_sel"], "old_sel": ["lbrack", "expr"],
                 "new_sel": ["lbrack", "new_ix"], "new_ix": ["expr"], "expr": ["lbrack"], "other": ["lbrack"]}
        return base, new, edges
    arms: list[tuple[str, bool, str]] = []
    def arm(name, expect_holds, want_text, mutate=None, introduced=("new_sel", "new_ix"), d_committed=0,
            expect_refused=False):
        base, new, edges = tables()
        if mutate:
            mutate(base, new, edges)
        try:
            holds, detail, _ = caller_split_containment(base, new, edges, list(introduced), d_committed)
            ok = (not expect_refused) and holds == expect_holds and want_text in detail
            got = f"holds={holds}: {detail[:110]}"
        except Refused as exc:
            ok = expect_refused and want_text in str(exc)
            got = f"REFUSED: {str(exc)[:110]}"
        arms.append((name, ok, got))
        print(f"  {'✓' if ok else '✗'} {name} — {got}")
    arm("green: the measured .3.31 shape HOLDS (work moved, risers inside the sub-graph, fallers reported)",
        True, "caller split:")
    arm("red: a rise on a rule NOT reachable from the introduced set ESCAPES", False, "OUTSIDE the sub-graph",
        mutate=lambda b, n, e: n["other"].__setitem__("entries", 150))
    arm("red: committed not conserved in total", False, "re-decided, not moved", d_committed=5)
    arm("red: a pre-existing rule accepted MORE often", False, "got dearer on `committed`",
        mutate=lambda b, n, e: n["caller"].__setitem__("committed", 101))
    arm("red: the committed the old rule lost is not what the new rules gained", False, "did not simply move",
        mutate=lambda b, n, e: n["new_sel"].__setitem__("committed", 30))
    arm("refuse: an introduced rule that already existed", False, "already existed",
        introduced=("new_sel", "old_sel"), expect_refused=True)
    arm("refuse: an introduced rule the grammar does not have", False, "name no rule",
        introduced=("new_sel", "ghost"), expect_refused=True)
    arm("refuse: no introduced set at all", False, "declares none", introduced=(), expect_refused=True)
    failed = [a for a in arms if not a[1]]
    print(f"CALLER-SPLIT-CONTAINMENT-SELFTEST: arms={len(arms)} failed={len(failed)}")
    return 1 if failed else 0


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


def introduced_wrapper_frames(base: dict[str, dict[str, int]], new: dict[str, dict[str, int]],
                              edges: dict[str, list[str]], introduced: list[str],
                              d_committed: int, base_edges: dict[str, list[str]] | None = None
                              ) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the INTRODUCED-WRAPPER-FRAMES predicate
    (`SV-CORPUS-GRAD.3.41`).

    THE SHAPE. A grammar change LIFTS an inline group into a NEW rule — a wrapper — because the
    engine types the inline group wrongly (`ENGINE-UNIVERSAL-SERVICES.96`), or because a chain
    element has to be a rule to carry its own return annotation. The wrapper commits ONE frame per
    commit of the body it wraps: `expression_chain_step` (the operand chain's step, the binary
    triple | `inside_operator`) commits once per binary operator the sampled corpus accepts, so
    `committed` ROSE +29,653 while `entries` (-326,690) and `memo_hits` (-761,739) FELL — the
    grouped `inside_expression` it retired had re-parsed its whole left operand at every
    expression. Measured on `.3.41`: risers in `committed` are EXACTLY the introduced names
    (`expression_chain_step` +29,697, `inside_operator` +4, `inside_operator_sv_2017` +4); thirteen
    pre-existing rules of the retired nested derivation fell by 4 each; no pre-existing rule
    committed more.

    ⛔ WHY THE OTHER EIGHT CANNOT COVER IT. The three arithmetic identities and `host_memo_eviction`
    need `committed` flat; `contained_in_introduced_subgraph` refuses `Δcommitted > 0` and any fall;
    `profile_split_respelling` refuses ANY pre-existing rise (`trivia` and `kw_inside` rise here —
    the set's terminal is now tried at every chain step); `caller_split_containment` needs
    `committed` CONSERVED in total; `declared_alternatives_net_of_retired_work` asserts (5)
    `committed` did not rise. Every one of them is RIGHT to refuse a `committed` rise on a
    pre-existing rule; none of them describes a rise that sits purely on a wrapper's own frames.

    ⭐ WHAT THIS ONE ASSERTS, per rule and over the run:
      (1) `introduced` is declared and non-empty, every declared name is measured or in the graph,
          and no declared name is a PRE-EXISTING NONTERMINAL — a nonterminal's rise cannot be
          waived by declaring it (raises `Refused`, never a false verdict);
      (2) every rule name NEW to the measurement (absent from the baseline table AND from the
          baseline's reference graph) is declared as introduced;
      (3) NO pre-existing rule's `committed` ROSE — accepted derivations got no dearer anywhere
          the parser already accepted them;
      (4) every introduced NONTERMINAL commits at most as often as the rules it references
          (`trivia` excluded) commit in total — a wrapper frame is never committed without a
          frame of something it wraps; an introduced rule that references nothing is a LEAF TWIN
          (`ENGINE-UNIVERSAL-SERVICES.99`, `SV-CORPUS-GRAD.3.47`: a terminal split by edition under a
          profile DISPATCHER — `string_literal := string_literal_sv_2017 | string_literal_sv_2023 |
          string_literal_v2005` — enters one frame per dispatcher entry and commits one per dispatcher
          commit), admitted when at least one rule references it and it commits at most as often as
          the rules that reference it commit in total — the same bound read from the other side, a
          twin frame is never committed without the dispatcher's; a rule that references nothing AND
          is referenced by nothing is refused, unattributable;
      (5) the run's total `Δcommitted` equals the introduced rules' commits net of every commit the
          pre-existing and retired rules lost — the per-rule table and the totals agree, so the
          row's exact from/to is the wrapper's own arithmetic and nothing else;
      (6) every pre-existing rule whose ENTRIES rose (terminals and `trivia` included) is COVERED:
          the sum of those rises is at most the sum of the entry falls on pre-existing and retired
          rules — the change retired more speculation than it added.

    ⚠️⚠️ HONEST BOUND, stated where it will be read: this proves the new frames are the wrapper's
    own and that nothing pre-existing got dearer. It does NOT prove the wrapper is unavoidable —
    it is exactly the cost of the `.96` workaround, and the inline spelling (byte-identical trees
    once `.96` types nested alternations as passthrough) would retire every one of these frames.
    Say so in the row's `why`, and name the leaf that removes it.
    """
    if not introduced:
        raise Refused("the introduced-wrapper predicate needs the rule names the change INTRODUCED; "
                      "the acceptance row declares none, so the new frames are unattributed")
    declared = list(dict.fromkeys(introduced))
    known_before = set(base) | (set(base_edges) if base_edges is not None else set())
    for r in declared:
        if r not in new and r not in edges:
            raise Refused(f"`{r}` is declared as introduced but the measured grammar has no such rule")
        if r in known_before:
            refs = [x for x in edges.get(r, []) if x != "trivia"]
            if refs:
                raise Refused(f"`{r}` pre-existed and is a NONTERMINAL — `introduced` may name new rules "
                              f"and the terminals they consult; declaring a pre-existing nonterminal "
                              f"would waive real work on it")
    facts: dict = {"introduced": declared}
    undeclared_new = sorted(r for r in new if r not in known_before and r not in declared)
    facts["undeclared_new"] = undeclared_new
    if undeclared_new:
        return False, (f"{len(undeclared_new)} rule name(s) new to the measurement are not declared as "
                       f"introduced: " + " ".join(undeclared_new[:5])), facts
    dc = {r: new.get(r, {}).get("committed", 0) - base.get(r, {}).get("committed", 0)
          for r in (set(base) | set(new))}
    pre_rose = sorted((r, v) for r, v in dc.items() if v > 0 and r not in declared)
    facts["pre_existing_committed_risers"] = pre_rose
    if pre_rose:
        return False, (f"{len(pre_rose)} pre-existing rule(s) COMMIT more than before: "
                       + " ".join(f"{r} {v:+,}" for r, v in pre_rose[:5])), facts
    over = []
    leaf_twins: list = []
    for r in declared:
        if r in known_before:
            continue
        refs = [x for x in edges.get(r, []) if x != "trivia"]
        if r not in edges:
            continue
        own = new.get(r, {}).get("committed", 0)
        if not refs:
            # a LEAF TWIN: the frame belongs to the rules that reference it (its profile dispatcher)
            dispatchers = sorted(d for d, targets in edges.items() if r in targets and d != r)
            if not dispatchers:
                raise Refused(f"introduced rule `{r}` references nothing and nothing references it — "
                              f"a wrapper wraps something, a leaf twin is dispatched by something")
            leaf_twins.append((r, dispatchers))
            wrapped = sum(new.get(x, {}).get("committed", 0) for x in dispatchers)
            if own > wrapped:
                over.append((r, own, wrapped))
            continue
        wrapped = sum(new.get(x, {}).get("committed", 0) for x in refs)
        if own > wrapped:
            over.append((r, own, wrapped))
    facts["wrapper_over_wrapped"] = over
    facts["leaf_twins"] = leaf_twins
    if over:
        r, own, wrapped = over[0]
        return False, (f"introduced `{r}` committed {own:,} frames but the rules it wraps (or, for a leaf twin, "
                       f"the rules that dispatch it) committed only {wrapped:,} in total — a frame without the one it "
                       f"belongs to"), facts
    intro_commits = sum(new.get(r, {}).get("committed", 0) for r in declared if r not in known_before)
    lost = sum(-v for r, v in dc.items() if v < 0)
    facts["introduced_commits"] = intro_commits
    facts["commits_lost_elsewhere"] = lost
    if intro_commits - lost != d_committed:
        return False, (f"the per-rule table says committed moved {intro_commits - lost:+,} (introduced "
                       f"{intro_commits:,} net of {lost:,} lost elsewhere) but the run's total moved "
                       f"{d_committed:+,} — the row's from/to is not this table's arithmetic"), facts
    de = {r: new.get(r, {}).get("entries", 0) - base.get(r, {}).get("entries", 0)
          for r in (set(base) | set(new))}
    pre_entry_rise = sum(v for r, v in de.items() if v > 0 and r not in declared)
    entry_falls = sum(-v for v in de.values() if v < 0)
    facts["pre_existing_entry_rise"] = pre_entry_rise
    facts["entry_falls"] = entry_falls
    if pre_entry_rise > entry_falls:
        return False, (f"pre-existing rules rose {pre_entry_rise:+,} entries in total while the change "
                       f"retired only {entry_falls:,} elsewhere — the rise is not covered by moved work"), facts
    return True, (f"introduced {' '.join(declared)} commit {intro_commits:,} frames, each bounded by the "
                  f"rules it wraps; {lost:,} commits retired elsewhere (Δcommitted {d_committed:+,}); no "
                  f"pre-existing rule commits more; pre-existing entry rises {pre_entry_rise:+,} covered by "
                  f"{entry_falls:,} retired entries"), facts


def self_test_introduced_wrapper_frames() -> int:
    """Arms for the INTRODUCED-WRAPPER-FRAMES predicate — every refusal and failure shape, no parser."""
    def row(e, c, m):
        return {"entries": e, "committed": c, "memo_hits": m}
    # the founding shape in miniature: `old` (grouped, re-parses `chain`) retired; `w` (wrapper) + `op` new
    edges = {"chain": ["operand", "w"], "w": ["binop", "operand", "op"], "op": ["kw", "set"],
             "binop": ["trivia"], "kw": ["trivia"], "operand": ["trivia"], "set": ["trivia"],
             "trivia": [], "other": ["trivia"]}
    base_edges = {"chain": ["operand", "binop", "old"], "old": ["chain", "kw", "set"], "binop": ["trivia"],
                  "kw": ["trivia"], "operand": ["trivia"], "set": ["trivia"], "trivia": [], "other": ["trivia"]}
    base = {"chain": row(1000, 100, 500), "old": row(800, 4, 400), "binop": row(700, 40, 250),
            "operand": row(460, 250, 70), "kw": row(345, 4, 0), "set": row(10, 4, 0),
            "trivia": row(90000, 1000, 88000), "other": row(50, 5, 10)}
    new = {"chain": row(600, 96, 100), "w": row(400, 40, 10), "op": row(390, 4, 1),
           "binop": row(690, 40, 240), "operand": row(459, 246, 69), "kw": row(396, 4, 0),
           "set": row(10, 4, 0), "trivia": row(90050, 996, 88050), "other": row(50, 5, 10)}
    def d_committed(b, n):
        return sum(v["committed"] for v in n.values()) - sum(v["committed"] for v in b.values())
    arms, failed = [], []
    def arm(name, cond):
        arms.append(name); print(f"  {'✓' if cond else '✗'} {name}")
        if not cond: failed.append(name)
    h, d, f = introduced_wrapper_frames(base, new, edges, ["w", "op"], d_committed(base, new), base_edges)
    arm("holds: the founding shape — wrapper + new op commit, the grouped rule retired, nothing pre-existing commits more", h and f["introduced_commits"] == 44)
    n2 = {k: dict(v) for k, v in new.items()}; n2["binop"]["committed"] = 41
    h, d, f = introduced_wrapper_frames(base, n2, edges, ["w", "op"], d_committed(base, n2), base_edges)
    arm("fails: a PRE-EXISTING rule commits more", not h and f["pre_existing_committed_risers"] == [("binop", 1)])
    h, d, f = introduced_wrapper_frames(base, new, edges, ["w"], d_committed(base, new), base_edges)
    arm("fails: an undeclared new rule (op)", not h and f["undeclared_new"] == ["op"])
    n3 = {k: dict(v) for k, v in new.items()}; n3["w"]["committed"] = 300
    h, d, f = introduced_wrapper_frames(base, n3, edges, ["w", "op"], d_committed(base, n3), base_edges)
    arm("fails: the wrapper commits MORE than the rules it wraps", not h and f["wrapper_over_wrapped"][0][0] == "w")
    h, d, f = introduced_wrapper_frames(base, new, edges, ["w", "op"], d_committed(base, new) + 1, base_edges)
    arm("fails: the row's Δcommitted is not the table's arithmetic", not h and "not this table's arithmetic" in d)
    n4 = {k: dict(v) for k, v in new.items()}; n4["other"]["entries"] = 5000
    h, d, f = introduced_wrapper_frames(base, n4, edges, ["w", "op"], d_committed(base, n4), base_edges)
    arm("fails: pre-existing entry rises NOT covered by the retired entries", not h and "not covered" in d)
    try:
        introduced_wrapper_frames(base, new, edges, [], d_committed(base, new), base_edges); arm("refuse: empty scope", False)
    except Refused: arm("refuse: empty scope", True)
    try:
        introduced_wrapper_frames(base, new, edges, ["w", "op", "chain"], d_committed(base, new), base_edges); arm("refuse: a declared PRE-EXISTING nonterminal", False)
    except Refused: arm("refuse: a declared PRE-EXISTING nonterminal", True)
    try:
        introduced_wrapper_frames(base, new, edges, ["w", "op", "zz"], d_committed(base, new), base_edges); arm("refuse: a declared name the grammar lacks", False)
    except Refused: arm("refuse: a declared name the grammar lacks", True)
    e2 = dict(edges); e2["w"] = []; e2["chain"] = ["operand"]
    try:
        introduced_wrapper_frames(base, new, e2, ["w", "op"], d_committed(base, new), base_edges); arm("refuse: an introduced rule that wraps nothing AND is dispatched by nothing", False)
    except Refused: arm("refuse: an introduced rule that wraps nothing AND is dispatched by nothing", True)
    # ENGINE-UNIVERSAL-SERVICES.99 (SV-CORPUS-GRAD.3.47): a LEAF TWIN under a profile dispatcher — `lit` (pre-existing,
    # was a terminal) now dispatches to the new terminal twin `lit_a`, which references only trivia; every `lit` entry /
    # commit is mirrored by one `lit_a` entry / commit and nothing else moves
    e3 = dict(edges); e3["lit"] = ["lit_a", "lit_b"]; e3["lit_a"] = ["trivia"]; e3["lit_b"] = ["trivia"]; e3["other"] = ["trivia", "lit"]
    be3 = dict(base_edges); be3["lit"] = ["trivia"]; be3["other"] = ["trivia", "lit"]
    b3 = {k: dict(v) for k, v in base.items()}; b3["lit"] = row(300, 20, 5)
    n3b = {k: dict(v) for k, v in base.items()}; n3b["lit"] = row(300, 20, 5); n3b["lit_a"] = row(300, 20, 0)
    h, d, f = introduced_wrapper_frames(b3, n3b, e3, ["lit_a", "lit_b"], d_committed(b3, n3b), be3)
    arm("holds: a leaf twin under a profile dispatcher — its frames equal the dispatcher's, nothing else moves", h and f["leaf_twins"] == [("lit_a", ["lit"]), ("lit_b", ["lit"])] and f["introduced_commits"] == 20)
    n3c = {k: dict(v) for k, v in n3b.items()}; n3c["lit_a"] = row(300, 21, 0)
    h, d, f = introduced_wrapper_frames(b3, n3c, e3, ["lit_a", "lit_b"], d_committed(b3, n3c), be3)
    arm("fails: a leaf twin commits MORE than its dispatcher", not h and f["wrapper_over_wrapped"][0][0] == "lit_a")
    h, d, f = introduced_wrapper_frames(base, new, edges, ["w", "op", "kw"], d_committed(base, new), base_edges)
    arm("holds: a pre-existing TERMINAL may be declared (it is never waived work)", h)
    print(f"INTRODUCED-WRAPPER-FRAMES-SELFTEST: arms={len(arms)} failed={len(failed)}")
    return 1 if failed else 0


def scope_gated_arm(base: dict[str, dict[str, int]], new: dict[str, dict[str, int]],
                    edges: dict[str, list[str]], introduced: list[str],
                    d_committed: int, base_edges: dict[str, list[str]] | None = None
                    ) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the SCOPE-GATED-ARM predicate (`SV-CORPUS-GRAD.3.38`).

    THE SHAPE. A grammar change puts ONE arm of a pre-existing rule behind a SCOPE. It introduces
    marker wrappers around pre-existing terminals that open and close the scope
    (`dpi_import_spec_open := dpi_spec_string`, `dpi_import_close := semi`), a gated wrapper on the
    arm (`dpi_open_packed_dimension := unsized_dimension !lbrack`, gated `in_scope_kind`), and a
    guard lookahead on the sibling arm that CONSULTS a pre-existing memoized rule
    (`!unsized_dimension`). The HOSTS — the pre-existing rules whose bodies were re-spelled to
    consult the new rules (`packed_dimension`, `dpi_import_export`) — are entered, answered and
    committed exactly as before. Measured on `.3.38` over the pinned sample:
    `dpi_open_packed_dimension` +79,340 entries (one per attempt of the arm it now fronts),
    `unsized_dimension` +17,832 entries of which +17,733 are memo hits (the guard asks a memoized
    rule at positions the star had already asked), the two markers +224 entries and +8 committed
    (one frame per DPI import the sample accepts, bounded by what they wrap), and two pre-existing
    rules FELL — `dpi_spec_string` -96 and `trivia` -93 — by exactly the memo hits the memoized
    marker now answers in their place (`dpi_import_spec_open` memo_hits +96).

    ⛔ WHY THE OTHER TEN CANNOT COVER IT. `contained_in_introduced_subgraph` refuses `Δcommitted > 0`
    and any fall; `introduced_wrapper_frames` explains the committed half exactly but requires every
    pre-existing entry rise to be covered by retired entries (17,931 against 189 here — the guard is
    new consultation, not moved work); `caller_split_containment` needs `committed` conserved in
    total; `host_memo_eviction` needs a host whose memo hits fall to zero; `profile_split_respelling`
    refuses any pre-existing rise; `declared_alternatives_net_of_retired_work` asserts `committed`
    did not rise; the three identities need shapes this is not; `advanced_files_only` is about files
    that parse further. Every one of them is RIGHT to refuse what it refuses.

    ⭐ WHAT THIS ONE ASSERTS, per rule and over the run:
      (1) `introduced` is declared and non-empty, no declared name is a pre-existing NONTERMINAL
          (raises `Refused`), and every rule name new to the measurement is declared;
      (2) the HOSTS are DERIVED — every pre-existing rule whose reference list differs between the
          baseline graph and the measured graph — non-empty, each consulting an introduced rule,
          and each host's own `entries` / `committed` / `memo_hits` are UNCHANGED: the change moved
          what the host consults, never how often the host is asked or answered;
      (3) NO pre-existing rule's `committed` moved in EITHER direction — a narrowing that changed an
          accepted derivation in the sample would show here and is a different shape — and the
          run's `Δcommitted` equals the introduced rules' own frames, each introduced nonterminal
          committing at most as often as the rules it wraps (`trivia` excluded);
      (4) every pre-existing rule whose `entries` ROSE is reachable, in the measured grammar's own
          reference graph, from an introduced rule or a host (the guard's consultations stay inside
          the construct), and each riser rose by at most the entries of the introduced rules and
          hosts that consult it — a consulter asks once more per entry, never "everywhere"; the
          misses (rise net of memo hits) are REPORTED, not thresholded;
      (5) every pre-existing rule whose `entries` FELL is reachable from an introduced rule and the
          total fall is at most the memo hits the introduced rules gained — a memoized wrapper
          answering in the wrapped rule's place, never lost work (its commits are unchanged by (3)).

    ⚠️⚠️ HONEST BOUND, stated where it will be read: this proves the rise is CONFINED to the gated
    construct, that the guard buys memo hits rather than work, and that nothing the parser already
    accepted got dearer. It does NOT prove the arm's wrapper frame is unavoidable — an in-branch
    `@predicate` placement (`ANNOTATION-PLACEMENT`) would retire the wrapper and its frame — nor
    that the guard lookahead is free (99 real misses at single-optional packed sites). Say so in
    the row's `why`, and name the leaf that removes it.

    Raises `Refused` (never a false verdict) when the scope is undeclared, names a rule the grammar
    lacks, names a pre-existing nonterminal, wraps nothing, or when the baseline graph needed to
    derive the hosts is absent.
    """
    if not introduced:
        raise Refused("the scope-gated-arm predicate needs the rule names the change INTRODUCED; "
                      "the acceptance row declares none, so the gated construct is unattributed")
    if base_edges is None:
        raise Refused("the scope-gated-arm predicate derives the re-spelled HOSTS from the baseline "
                      "reference graph, and no baseline graph was given")
    declared = list(dict.fromkeys(introduced))
    known_before = set(base) | set(base_edges)
    for r in declared:
        if r not in new and r not in edges:
            raise Refused(f"`{r}` is declared as introduced but the measured grammar has no such rule")
        if r in known_before:
            refs = [x for x in edges.get(r, []) if x != "trivia"]
            if refs:
                raise Refused(f"`{r}` pre-existed and is a NONTERMINAL — `introduced` may name new rules "
                              f"and the terminals they consult; declaring a pre-existing nonterminal "
                              f"would waive real work on it")
    facts: dict = {"introduced": declared}
    undeclared_new = sorted(r for r in new if r not in known_before and r not in declared)
    facts["undeclared_new"] = undeclared_new
    if undeclared_new:
        return False, (f"{len(undeclared_new)} rule name(s) new to the measurement are not declared as "
                       f"introduced: " + " ".join(undeclared_new[:5])), facts
    intro_new = [r for r in declared if r not in known_before]

    # (2) the hosts, DERIVED from the two graphs
    hosts = sorted(r for r in edges if r in known_before and r not in declared
                   and set(edges.get(r, [])) != set(base_edges.get(r, [])))
    facts["hosts"] = hosts
    if not hosts:
        return False, ("no pre-existing rule's reference list changed — a gated arm re-spells its "
                       "host, and no host was found"), facts
    orphan_hosts = [h for h in hosts if not any(x in declared for x in edges.get(h, []))]
    if orphan_hosts:
        return False, (f"{len(orphan_hosts)} pre-existing rule(s) were re-spelled without consulting an "
                       f"introduced rule: " + " ".join(orphan_hosts[:5]) + " — not a gated arm"), facts
    host_moved = []
    for h in hosts:
        for k in ("entries", "committed", "memo_hits"):
            if new.get(h, {}).get(k, 0) != base.get(h, {}).get(k, 0):
                host_moved.append((h, k, new.get(h, {}).get(k, 0) - base.get(h, {}).get(k, 0)))
    facts["host_moved"] = host_moved
    if host_moved:
        h, k, v = host_moved[0]
        return False, (f"host `{h}` is asked or answered differently ({k} {v:+,}) — the change moved more "
                       f"than what the host consults"), facts

    # (3) committed
    names = set(base) | set(new)
    dc = {r: new.get(r, {}).get("committed", 0) - base.get(r, {}).get("committed", 0) for r in names}
    pre_moved = sorted((r, v) for r, v in dc.items() if v != 0 and r not in declared)
    facts["pre_existing_committed_moved"] = pre_moved
    if pre_moved:
        return False, (f"{len(pre_moved)} pre-existing rule(s) COMMIT differently: "
                       + " ".join(f"{r} {v:+,}" for r, v in pre_moved[:5])
                       + " — the sample's accepted derivations moved, which a gate on one arm must not do"), facts
    over = []
    for r in intro_new:
        refs = [x for x in edges.get(r, []) if x != "trivia"]
        if r not in edges:
            continue
        if not refs:
            raise Refused(f"introduced rule `{r}` references nothing — a wrapper wraps something")
        own = new.get(r, {}).get("committed", 0)
        wrapped = sum(new.get(x, {}).get("committed", 0) for x in refs)
        if own > wrapped:
            over.append((r, own, wrapped))
    facts["wrapper_over_wrapped"] = over
    if over:
        r, own, wrapped = over[0]
        return False, (f"introduced `{r}` committed {own:,} frames but the rules it wraps committed only "
                       f"{wrapped:,} in total — a wrapper frame without a wrapped one"), facts
    intro_commits = sum(new.get(r, {}).get("committed", 0) for r in intro_new)
    facts["introduced_commits"] = intro_commits
    if intro_commits != d_committed:
        return False, (f"the per-rule table says committed moved {intro_commits:+,} (the introduced rules' "
                       f"own frames) but the run's total moved {d_committed:+,} — the row's from/to is not "
                       f"this table's arithmetic"), facts

    # (4) rises — confined, and memo-dominated
    known = set(edges) | {x for refs in edges.values() for x in refs}
    origins = {r: origin_rule(r, known) for r in names}
    unplaced = sorted(r for r, o in origins.items() if o is None)
    if unplaced:
        raise Refused(f"{len(unplaced)} measured rule(s) cannot be located in the grammar's reference "
                      f"graph: {unplaced[:5]}{' …' if len(unplaced) > 5 else ''}")
    de = {r: new.get(r, {}).get("entries", 0) - base.get(r, {}).get("entries", 0) for r in names}
    dm = {r: new.get(r, {}).get("memo_hits", 0) - base.get(r, {}).get("memo_hits", 0) for r in names}
    scope = reachable(edges, declared + hosts)
    risers = {r: v for r, v in de.items() if v > 0 and r not in declared}
    escaped = {r: v for r, v in risers.items() if origins[r] not in scope}
    facts["risers"] = risers
    facts["escaped"] = escaped
    if escaped:
        worst = sorted(escaped.items(), key=lambda t: -t[1])[:3]
        return False, (f"{len(escaped)} pre-existing rule(s) rose OUTSIDE the sub-graph reachable from the "
                       f"introduced rules and their hosts ({sum(escaped.values()):+,}): "
                       + ", ".join(f"{r} {v:+,}" for r, v in worst)), facts
    # each riser is consulted at most once more per entry of a consulter the change gave it: an
    # introduced rule that references it, or a host whose re-spelling references it. ⚠️ The graph
    # is a SET of references per rule, so a host that gained TWO sites for one rule reads as one;
    # the shapes this predicate is for (a guard, a fronting wrapper) add one site per host.
    consulters = {r: [c for c in list(intro_new) + hosts if origins[r] in edges.get(c, [])
                      or r in edges.get(c, [])] for r in risers}
    unbounded = {}
    for r, v in risers.items():
        cap = sum(new.get(c, {}).get("entries", 0) for c in consulters[r])
        if v > cap:
            unbounded[r] = (v, cap)
    facts["riser_consulters"] = consulters
    facts["unbounded_risers"] = unbounded
    if unbounded:
        r, (v, cap) = sorted(unbounded.items(), key=lambda t: -t[1][0])[0]
        return False, (f"`{r}` rose {v:+,} entries but the introduced rules and hosts that consult it are "
                       f"entered only {cap:,} times in total — more consultation than the construct can "
                       f"make"), facts
    misses = sum(v - dm.get(r, 0) for r, v in risers.items())
    facts["pre_existing_misses"] = misses

    # (5) falls — memo shadowing by an introduced memoized wrapper, never lost work
    fallers = {r: v for r, v in de.items() if v < 0 and r not in declared}
    scope_intro = reachable(edges, declared)
    stray = {r: v for r, v in fallers.items() if origins[r] not in scope_intro}
    facts["fallers"] = fallers
    facts["stray_fallers"] = stray
    if stray:
        worst = sorted(stray.items(), key=lambda t: t[1])[:3]
        return False, (f"{len(stray)} pre-existing rule(s) LOST entries outside the introduced rules' "
                       f"sub-graph ({sum(stray.values()):+,}): "
                       + ", ".join(f"{r} {v:+,}" for r, v in worst)
                       + " — a fall no introduced memo can explain"), facts
    total_fall = sum(-v for v in fallers.values())
    intro_memo = sum(max(0, dm.get(r, 0)) for r in intro_new)
    facts["total_fall"] = total_fall
    facts["introduced_memo_hits"] = intro_memo
    if total_fall > intro_memo:
        return False, (f"pre-existing rules lost {total_fall:,} entries but the introduced rules gained only "
                       f"{intro_memo:,} memo hits — the fall is not memo shadowing"), facts
    return True, (f"hosts {' '.join(hosts)} asked/answered/committed exactly as before; introduced "
                  f"{' '.join(declared)} commit {intro_commits:,} frames (= Δcommitted, each bounded by what "
                  f"it wraps), no pre-existing rule commits differently; {len(risers):,} pre-existing riser(s) "
                  f"(+{sum(risers.values()):,}) all inside the gated construct, each bounded by its consulters' entries, "
                  f"{misses:+,} of them misses (the rest memo hits); {len(fallers):,} faller(s) "
                  f"(-{total_fall:,}) inside the introduced sub-graph, covered by {intro_memo:,} introduced memo "
                  f"hits"), facts


def self_test_scope_gated_arm() -> int:
    """Arms for the SCOPE-GATED-ARM predicate — every refusal and failure shape, no parser."""
    def row(e, c, m):
        return {"entries": e, "committed": c, "memo_hits": m}
    # the founding shape in miniature: host `pd` (packed_dimension) gains a guard on its range arm and
    # fronts its unsized arm with `gated`; host `imp` (dpi_import_export) takes the two markers
    base_edges = {"pd": ["lb", "cr", "rb", "u"], "u": ["lb", "rb"], "imp": ["kw", "spec", "proto", "semi"],
                  "lb": ["trivia"], "rb": ["trivia"], "cr": ["trivia"], "kw": ["trivia"], "spec": ["trivia"],
                  "semi": ["trivia"], "proto": ["kw"], "trivia": [], "other": ["trivia"]}
    edges = {"pd": ["lb", "cr", "rb", "u", "gated"], "gated": ["u", "lb"], "u": ["lb", "rb"],
             "imp": ["kw", "open", "proto", "close"], "open": ["spec"], "close": ["semi"],
             "lb": ["trivia"], "rb": ["trivia"], "cr": ["trivia"], "kw": ["trivia"], "spec": ["trivia"],
             "semi": ["trivia"], "proto": ["kw"], "trivia": [], "other": ["trivia"]}
    base = {"pd": row(796, 88, 2), "u": row(1532, 0, 360), "imp": row(804, 0, 6), "lb": row(33751, 758, 28248),
            "rb": row(5093, 758, 3732), "cr": row(1691, 206, 753), "kw": row(500, 4, 0), "spec": row(200, 5, 0),
            "semi": row(9000, 900, 100), "proto": row(20, 4, 0), "trivia": row(905000, 11810, 882090),
            "other": row(50, 5, 10)}
    new = {k: dict(v) for k, v in base.items()}
    new["gated"] = row(793, 0, 1)                      # one entry per attempt of the arm it fronts
    new["u"] = row(1532 + 178, 0, 360 + 177)           # the guard asks a memoized rule: 178 more, 177 hits
    new["lb"] = row(33751 + 1, 758, 28248)             # the one real miss computes one bracket test
    new["open"] = row(2, 4, 1)                         # the marker: 4 frames = the 4 imports, 1 memo hit
    new["close"] = row(1, 4, 0)
    new["spec"] = row(200 - 1, 5, 0)                   # shadowed by the marker's memo hit
    new["trivia"] = row(905000 - 1, 11810, 882090 - 1)
    def d_committed(b, n):
        return sum(v["committed"] for v in n.values()) - sum(v["committed"] for v in b.values())
    arms, failed = [], []
    def arm(name, cond):
        arms.append(name); print(f"  {'✓' if cond else '✗'} {name}")
        if not cond: failed.append(name)
    intro = ["open", "close", "gated"]
    h, d, f = scope_gated_arm(base, new, edges, intro, d_committed(base, new), base_edges)
    arm("holds: the founding shape — hosts unchanged, markers commit their 8, guard memo-dominated, falls shadowed",
        h and f["hosts"] == ["imp", "pd"] and f["introduced_commits"] == 8 and f["total_fall"] == 2)
    n2 = {k: dict(v) for k, v in new.items()}; n2["cr"]["committed"] = 207
    h, d, f = scope_gated_arm(base, n2, edges, intro, d_committed(base, n2), base_edges)
    arm("fails: a PRE-EXISTING rule commits MORE", not h and f["pre_existing_committed_moved"] == [("cr", 1)])
    n3 = {k: dict(v) for k, v in new.items()}; n3["cr"]["committed"] = 205
    h, d, f = scope_gated_arm(base, n3, edges, intro, d_committed(base, n3), base_edges)
    arm("fails: a PRE-EXISTING rule commits LESS (an accepted derivation moved)", not h and f["pre_existing_committed_moved"] == [("cr", -1)])
    n4 = {k: dict(v) for k, v in new.items()}; n4["pd"]["entries"] = 797
    h, d, f = scope_gated_arm(base, n4, edges, intro, d_committed(base, n4), base_edges)
    arm("fails: a HOST is asked more often", not h and f["host_moved"] == [("pd", "entries", 1)])
    h, d, f = scope_gated_arm(base, new, edges, ["open", "close"], d_committed(base, new), base_edges)
    arm("fails: an undeclared new rule (gated)", not h and f["undeclared_new"] == ["gated"])
    n5 = {k: dict(v) for k, v in new.items()}; n5["other"]["entries"] = 60
    h, d, f = scope_gated_arm(base, n5, edges, intro, d_committed(base, n5), base_edges)
    arm("fails: a riser OUTSIDE the gated construct", not h and list(f["escaped"]) == ["other"])
    n6 = {k: dict(v) for k, v in new.items()}; n6["u"] = row(1532 + 2000, 0, 360 + 1990)
    h, d, f = scope_gated_arm(base, n6, edges, intro, d_committed(base, n6), base_edges)
    arm("fails: a riser consulted more often than its consulters are entered", not h and list(f["unbounded_risers"]) == ["u"])
    n7 = {k: dict(v) for k, v in new.items()}; n7["spec"]["entries"] = 150
    h, d, f = scope_gated_arm(base, n7, edges, intro, d_committed(base, n7), base_edges)
    arm("fails: a fall larger than the introduced memo hits", not h and f["total_fall"] == 51)
    n8 = {k: dict(v) for k, v in new.items()}; n8["other"]["entries"] = 40
    h, d, f = scope_gated_arm(base, n8, edges, intro, d_committed(base, n8), base_edges)
    arm("fails: a faller outside the introduced sub-graph", not h and list(f["stray_fallers"]) == ["other"])
    n9 = {k: dict(v) for k, v in new.items()}; n9["open"]["committed"] = 6
    h, d, f = scope_gated_arm(base, n9, edges, intro, d_committed(base, n9), base_edges)
    arm("fails: a marker commits MORE than the terminal it wraps", not h and f["wrapper_over_wrapped"][0][0] == "open")
    h, d, f = scope_gated_arm(base, new, edges, intro, d_committed(base, new) + 1, base_edges)
    arm("fails: the row's Δcommitted is not the table's arithmetic", not h and "not this table's arithmetic" in d)
    e2 = {k: list(v) for k, v in edges.items()}; e2["pd"] = base_edges["pd"]; e2["imp"] = base_edges["imp"]
    h, d, f = scope_gated_arm(base, new, e2, intro, d_committed(base, new), base_edges)
    arm("fails: no host re-spelled (nothing consults the introduced rules)", not h and f["hosts"] == [])
    e3 = {k: list(v) for k, v in edges.items()}; e3["other"] = ["trivia", "kw"]
    h, d, f = scope_gated_arm(base, new, e3, intro, d_committed(base, new), base_edges)
    arm("fails: a pre-existing rule re-spelled without consulting an introduced rule", not h and "not a gated arm" in d)
    try:
        scope_gated_arm(base, new, edges, [], d_committed(base, new), base_edges); arm("refuse: empty scope", False)
    except Refused: arm("refuse: empty scope", True)
    try:
        scope_gated_arm(base, new, edges, intro, d_committed(base, new), None); arm("refuse: no baseline graph", False)
    except Refused: arm("refuse: no baseline graph", True)
    try:
        scope_gated_arm(base, new, edges, intro + ["pd"], d_committed(base, new), base_edges); arm("refuse: a declared PRE-EXISTING nonterminal", False)
    except Refused: arm("refuse: a declared PRE-EXISTING nonterminal", True)
    try:
        scope_gated_arm(base, new, edges, intro + ["zz"], d_committed(base, new), base_edges); arm("refuse: a declared name the grammar lacks", False)
    except Refused: arm("refuse: a declared name the grammar lacks", True)
    e4 = {k: list(v) for k, v in edges.items()}; e4["gated"] = []
    try:
        scope_gated_arm(base, new, e4, intro, d_committed(base, new), base_edges); arm("refuse: an introduced rule that wraps nothing", False)
    except Refused: arm("refuse: an introduced rule that wraps nothing", True)
    print(f"SCOPE-GATED-ARM-SELFTEST: arms={len(arms)} failed={len(failed)}")
    return 1 if failed else 0


def typed_replacement_of_opaque_terminal(base: dict[str, dict[str, int]], new: dict[str, dict[str, int]],
                                         edges: dict[str, list[str]], introduced: list[str],
                                         d_committed: int, base_edges: dict[str, list[str]] | None = None
                                         ) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the TYPED-REPLACEMENT-OF-OPAQUE-TERMINAL predicate
    (`SV-CORPUS-GRAD.3.39`).

    THE SHAPE. A construct the baseline consumed with ONE opaque terminal — the compiler directive,
    `` trivia /`[^\r\n]*/ `` swallowing the line — is now DERIVED by typed productions (the thirteen
    syntax boxes of IEEE 1800-2017 clause 22 / IEEE 1364-2005 clause 19): a directive line that
    committed one frame commits one frame per production and per token it is made of, and the
    pre-existing tokens the typed derivation now consumes ONE BY ONE (`time_unit`, `kw_ns`,
    `timeunit_separator_slash`, `trivia` once per token) commit where the regex used to swallow
    them. Every position where the opaque terminal was TRIED and failed is guarded by one positive
    lookahead on the same line shape, so the typed alternation is entered only where a directive
    line starts. Measured on `.3.39` against `1.0.216`: the unguarded typed spelling read `entries`
    +1,454,802 / `memo_hits` +437,165 — seven directive frames, eight `kw_dir_*` attempts and
    eight memoized `trivia` lookups at 71,972 item positions holding no directive — and this
    predicate REFUSES it under (6): the pre-existing rises are not covered by what the change
    retired. The guarded spelling is the one it accepts.

    ⛔ WHY THE OTHER ELEVEN CANNOT COVER IT. The three arithmetic identities, `host_memo_eviction`
    and `caller_split_containment` need `committed` flat or conserved; `contained_in_introduced_subgraph`
    refuses `Δcommitted > 0` and any fall (the retired opaque wrapper falls to zero);
    `profile_split_respelling` refuses ANY pre-existing rise; `declared_alternatives_net_of_retired_work`
    asserts (5) `committed` did not rise; `introduced_wrapper_frames` and `scope_gated_arm` refuse
    every pre-existing `committed` rise — and here `time_unit`, `kw_ns`, `kw_ps` and `trivia` DO
    commit more, because the typed derivation consumes the tokens the regex swallowed;
    `advanced_files_only` is about text the parser newly reaches. Each is RIGHT to refuse; none
    describes frames that sit on the typed derivation of a line the baseline already accepted.

    ⭐ WHAT THIS ONE ASSERTS, per rule and over the run:
      (1) `introduced` is declared and non-empty, the BASELINE graph is present, every declared
          name is measured or in the graph, and no declared name is a PRE-EXISTING NONTERMINAL
          (raises `Refused`, never a false verdict) — a pre-existing TERMINAL may be declared;
      (2) every rule name NEW to the measurement (absent from the baseline table AND from the
          baseline's reference graph) is declared as introduced;
      (3) every pre-existing rule whose `committed` ROSE sits INSIDE the typed sub-graph — reachable
          from an introduced rule in the measured grammar's own reference graph — so the only
          derivations that got dearer are the typed ones;
      (4) those pre-existing commits rose by at most what the introduced rules commit in total — a
          consumed token commits at most once per typed frame;
      (5) the run's total `Δcommitted` equals the introduced rules' commits plus the in-scope
          pre-existing rises net of every commit lost elsewhere — the per-rule table and the totals
          agree, so the row's exact from/to is the typed derivation's own arithmetic;
      (6) every pre-existing rule whose ENTRIES rose sits inside the typed sub-graph, and the sum of
          those rises is at most the sum of the entry falls on pre-existing and retired rules — the
          guard retired at least the speculation the typed alternation added;
      (7) the pre-existing `memo_hits` rise, over every rule, is at most the introduced rules' own
          entries — each typed token attempt buys at most one memoized prefix lookup.
    Fallers are REPORTED in the facts, never refused.

    ⚠️⚠️ HONEST BOUND, stated where it will be read: this proves the new frames are the typed
    derivation's own, that the guard covers the speculation, and that nothing outside the typed
    sub-graph got dearer. It does NOT prove the typed frames are minimal — a dispatch on ONE
    directive-name token instead of an alternation of token-led productions would retire the
    remaining attempts at real directive lines — and a typed AST costs one frame per node by
    construction. Say so in the row's `why`.
    """
    if not introduced:
        raise Refused("the typed-replacement predicate needs the rule names the change INTRODUCED; "
                      "the acceptance row declares none, so the typed sub-graph is unattributed")
    if base_edges is None:
        raise Refused("the typed-replacement predicate needs the BASELINE grammar's reference graph "
                      "to tell a new rule from a pre-existing one that merely had zero entries")
    declared = list(dict.fromkeys(introduced))
    known_before = set(base) | set(base_edges)
    for r in declared:
        if r not in new and r not in edges:
            raise Refused(f"`{r}` is declared as introduced but the measured grammar has no such rule")
        if r in known_before:
            refs = [x for x in edges.get(r, []) if x != "trivia"]
            if refs:
                raise Refused(f"`{r}` pre-existed and is a NONTERMINAL — `introduced` may name new rules "
                              f"and the terminals they consume; declaring a pre-existing nonterminal "
                              f"would waive real work on it")
    facts: dict = {"introduced": declared}
    undeclared_new = sorted(r for r in new if r not in known_before and r not in declared)
    facts["undeclared_new"] = undeclared_new
    if undeclared_new:
        return False, (f"{len(undeclared_new)} rule name(s) new to the measurement are not declared as "
                       f"introduced: " + " ".join(undeclared_new[:5])), facts
    scope = reachable(edges, declared)
    facts["scope"] = len(scope)
    # A declared PRE-EXISTING terminal is a consumed token, not a new frame: its rises are judged
    # with the other pre-existing rules (in scope, bounded, in the arithmetic), never waived.
    fresh = {r for r in declared if r not in known_before}
    rules = set(base) | set(new)
    dc = {r: new.get(r, {}).get("committed", 0) - base.get(r, {}).get("committed", 0) for r in rules}
    de = {r: new.get(r, {}).get("entries", 0) - base.get(r, {}).get("entries", 0) for r in rules}
    dm = {r: new.get(r, {}).get("memo_hits", 0) - base.get(r, {}).get("memo_hits", 0) for r in rules}
    pre_c_rose = sorted((r, v) for r, v in dc.items() if v > 0 and r not in fresh)
    facts["pre_existing_committed_risers"] = pre_c_rose
    escaped = [(r, v) for r, v in pre_c_rose if r not in scope]
    facts["committed_risers_outside_scope"] = escaped
    if escaped:
        return False, (f"{len(escaped)} pre-existing rule(s) COMMIT more OUTSIDE the typed sub-graph: "
                       + " ".join(f"{r} {v:+,}" for r, v in escaped[:5])), facts
    intro_commits = sum(new.get(r, {}).get("committed", 0) for r in declared if r not in known_before)
    pre_c_rise = sum(v for _, v in pre_c_rose)
    facts["introduced_commits"] = intro_commits
    facts["consumed_commit_rise"] = pre_c_rise
    if pre_c_rise > intro_commits:
        return False, (f"the tokens the typed derivation consumes committed {pre_c_rise:+,} more in "
                       f"total but the introduced rules commit only {intro_commits:,} frames — more "
                       f"than one consumed commit per typed frame"), facts
    lost = sum(-v for v in dc.values() if v < 0)
    facts["commits_lost_elsewhere"] = lost
    facts["committed_fallers"] = sorted((r, v) for r, v in dc.items() if v < 0)
    if intro_commits + pre_c_rise - lost != d_committed:
        return False, (f"the per-rule table says committed moved {intro_commits + pre_c_rise - lost:+,} "
                       f"(introduced {intro_commits:,} + consumed {pre_c_rise:,} net of {lost:,} lost "
                       f"elsewhere) but the run's total moved {d_committed:+,} — the row's from/to is "
                       f"not this table's arithmetic"), facts
    pre_e_rose = sorted((r, v) for r, v in de.items() if v > 0 and r not in fresh)
    facts["pre_existing_entry_risers"] = pre_e_rose
    escaped_e = [(r, v) for r, v in pre_e_rose if r not in scope]
    facts["entry_risers_outside_scope"] = escaped_e
    if escaped_e:
        return False, (f"{len(escaped_e)} pre-existing rule(s) are ENTERED more OUTSIDE the typed "
                       f"sub-graph: " + " ".join(f"{r} {v:+,}" for r, v in escaped_e[:5])), facts
    pre_entry_rise = sum(v for _, v in pre_e_rose)
    entry_falls = sum(-v for v in de.values() if v < 0)
    facts["pre_existing_entry_rise"] = pre_entry_rise
    facts["entry_falls"] = entry_falls
    facts["entry_fallers"] = sorted((r, v) for r, v in de.items() if v < 0)
    if pre_entry_rise > entry_falls:
        return False, (f"pre-existing rules rose {pre_entry_rise:+,} entries in total while the change "
                       f"retired only {entry_falls:,} elsewhere — the guard does not cover the "
                       f"speculation the typed alternation added"), facts
    intro_entries = sum(new.get(r, {}).get("entries", 0) for r in declared if r not in known_before)
    pre_memo_rise = sum(v for r, v in dm.items() if v > 0 and r not in fresh)
    facts["introduced_entries"] = intro_entries
    facts["pre_existing_memo_rise"] = pre_memo_rise
    if pre_memo_rise > intro_entries:
        return False, (f"pre-existing rules gained {pre_memo_rise:+,} memo hits but the introduced rules "
                       f"were entered only {intro_entries:,} times — more than one prefix lookup per "
                       f"typed attempt"), facts
    return True, (f"introduced {len(declared)} rule(s) commit {intro_commits:,} frames and the tokens they "
                  f"consume {pre_c_rise:+,} (all inside the typed sub-graph of {len(scope)} rules); "
                  f"{lost:,} commits retired elsewhere (Δcommitted {d_committed:+,}); pre-existing entry "
                  f"rises {pre_entry_rise:+,} (in scope) covered by {entry_falls:,} retired entries; "
                  f"memo rise {pre_memo_rise:+,} within {intro_entries:,} typed attempts"), facts


def self_test_typed_replacement_of_opaque_terminal() -> int:
    """Arms for the TYPED-REPLACEMENT-OF-OPAQUE-TERMINAL predicate — every refusal and failure
    shape, no parser."""
    def row(e, c, m):
        return {"entries": e, "committed": c, "memo_hits": m}
    # the founding shape in miniature: `directive` used to be ONE opaque regex tried at 1000 item
    # positions; now `item` guards it with `&opaque_line` and `directive` derives `typed`, whose
    # tokens (`kw_ts`, `mag`) and the PRE-EXISTING `unit` -> `kw_ns` commit where the regex swallowed them.
    edges = {"item": ["opaque_line", "directive", "other"], "directive": ["typed", "opaque_line"],
             "typed": ["kw_ts", "mag", "unit"], "unit": ["kw_ns"], "opaque_line": ["trivia"],
             "kw_ts": ["trivia"], "mag": ["trivia"], "kw_ns": ["trivia"], "other": ["unit", "trivia"], "trivia": []}
    base_edges = {"item": ["directive", "other"], "directive": ["trivia"], "unit": ["kw_ns"],
                  "kw_ns": ["trivia"], "other": ["unit", "trivia"], "trivia": []}
    base = {"item": row(1000, 500, 0), "directive": row(1000, 20, 0), "other": row(1000, 480, 0),
            "unit": row(50, 5, 0), "kw_ns": row(50, 5, 0), "trivia": row(5000, 1000, 4000)}
    new = {"item": row(1000, 500, 0), "directive": row(20, 20, 0), "opaque_line": row(1000, 37, 0),
           "typed": row(20, 3, 0), "kw_ts": row(20, 3, 0), "mag": row(6, 6, 0),
           "unit": row(56, 11, 0), "kw_ns": row(56, 11, 0), "trivia": row(5052, 1015, 4052),
           "other": row(1000, 480, 0)}
    intro = ["opaque_line", "typed", "kw_ts", "mag"]
    def d_committed(b, n):
        return sum(v["committed"] for v in n.values()) - sum(v["committed"] for v in b.values())
    def copy(t):
        return {k: dict(v) for k, v in t.items()}
    arms, failed = [], []
    def arm(name, cond):
        arms.append(name); print(f"  {'✓' if cond else '✗'} {name}")
        if not cond: failed.append(name)
    P = typed_replacement_of_opaque_terminal
    h, d, f = P(base, new, edges, intro, d_committed(base, new), base_edges)
    arm("holds: the founding shape — typed frames + consumed tokens commit, the guard retires the speculation",
        h and f["introduced_commits"] == 49 and f["consumed_commit_rise"] == 27)
    h, d, f = P(base, new, edges, ["opaque_line", "typed", "kw_ts"], d_committed(base, new), base_edges)
    arm("fails: an undeclared new rule (mag)", not h and f["undeclared_new"] == ["mag"])
    n2 = copy(new); n2["other"]["committed"] = 481
    h, d, f = P(base, n2, edges, intro, d_committed(base, n2), base_edges)
    arm("fails: a pre-existing rule commits more OUTSIDE the typed sub-graph",
        not h and f["committed_risers_outside_scope"] == [("other", 1)])
    n3 = copy(new); n3["unit"]["committed"] = 200
    h, d, f = P(base, n3, edges, intro, d_committed(base, n3), base_edges)
    arm("fails: consumed tokens commit more than the typed frames", not h and "more than one consumed commit" in d)
    h, d, f = P(base, new, edges, intro, d_committed(base, new) + 1, base_edges)
    arm("fails: the row's Δcommitted is not the table's arithmetic", not h and "not this table's arithmetic" in d)
    n4 = copy(new); n4["other"]["entries"] = 1005
    h, d, f = P(base, n4, edges, intro, d_committed(base, n4), base_edges)
    arm("fails: a pre-existing rule is entered more OUTSIDE the typed sub-graph",
        not h and f["entry_risers_outside_scope"] == [("other", 5)])
    n5 = copy(new); n5["trivia"]["entries"] = 5000 + 5000
    h, d, f = P(base, n5, edges, intro, d_committed(base, n5), base_edges)
    arm("fails: pre-existing entry rises NOT covered by the retired entries (the unguarded spelling)",
        not h and "does not cover" in d)
    n6 = copy(new); n6["trivia"]["memo_hits"] = 4000 + 5000
    h, d, f = P(base, n6, edges, intro, d_committed(base, n6), base_edges)
    arm("fails: memo hits rise faster than the typed attempts", not h and "more than one prefix lookup" in d)
    try:
        P(base, new, edges, [], d_committed(base, new), base_edges); arm("refuse: empty scope", False)
    except Refused: arm("refuse: empty scope", True)
    try:
        P(base, new, edges, intro, d_committed(base, new), None); arm("refuse: no baseline graph", False)
    except Refused: arm("refuse: no baseline graph", True)
    try:
        P(base, new, edges, intro + ["unit"], d_committed(base, new), base_edges); arm("refuse: a declared PRE-EXISTING nonterminal", False)
    except Refused: arm("refuse: a declared PRE-EXISTING nonterminal", True)
    try:
        P(base, new, edges, intro + ["zz"], d_committed(base, new), base_edges); arm("refuse: a declared name the grammar lacks", False)
    except Refused: arm("refuse: a declared name the grammar lacks", True)
    h, d, f = P(base, new, edges, intro + ["kw_ns"], d_committed(base, new), base_edges)
    arm("holds: a pre-existing TERMINAL may be declared (it is never waived work)", h)
    n7 = copy(new); n7["other"]["entries"] = 990; n7["other"]["committed"] = 478
    h, d, f = P(base, n7, edges, intro, d_committed(base, n7), base_edges)
    arm("holds: fallers outside the sub-graph are reported, never refused",
        h and ("other", -2) in f["committed_fallers"] and ("other", -10) in f["entry_fallers"])
    print(f"TYPED-REPLACEMENT-OF-OPAQUE-TERMINAL-SELFTEST: arms={len(arms)} failed={len(failed)}")
    return 1 if failed else 0


def lookahead_replay_occurrences(base: dict[str, dict[str, int]], new: dict[str, dict[str, int]],
                                 edges: dict[str, list[str]], introduced: list[str],
                                 base_files: dict[str, dict], new_files: dict[str, dict]
                                 ) -> tuple[bool, str, dict]:
    """`(holds, one-line detail, facts)` for the LOOKAHEAD-REPLAY-OCCURRENCES predicate
    (`ENGINE-UNIVERSAL-SERVICES.100`, `SV-CORPUS-GRAD.3.49`).

    THE SHAPE. A change UN-GATES an alternative (a store predicate that rejected legal text is deleted)
    and, wherever a sibling alternative already derives the same token sequence, fronts the un-gated
    twin with a lookahead on THAT sibling (`checker_instantiation_beside_module :=
    !interface_instantiation …`) so the twin is dead exactly where the sibling matches. The engine
    memoizes the sibling at that position, so the lookahead is a LOOKUP — the guarded rule's entries
    and memo hits rise by the same count and it is never parsed afresh — and no derivation changes:
    the corpus AST identity sweep reads `ast_moved=0`, every sampled file keeps its verdict and its
    furthest position. But the instrument's `committed` is an OCCURRENCE count (C3-B, `parser_registry.rs`:
    tournament winners, successful-but-losing branches AND memo replays all contribute their subtree
    with multiplicity), so the memoized sibling's subtree is counted AGAIN each time the lookahead
    replays it, and the un-gated alternative's own successful-but-losing parses — where nothing competes
    and the old predicate used to fail it early — are counted as well. Measured on `.3.49` over the
    pinned sample: `interface_instantiation` entries +72,025 = memo_hits +72,025, committed +0; the
    whole `committed` rise (+72,330) sits on four files whose verdicts, furthest positions and typed
    ASTs are unchanged — `t_math_wallace_mul.v` alone +63,594, sixteen instantiations with the widest
    port lists in the sample — and the un-gated `checker_instantiation` commits +32 as a losing branch.

    ⛔ WHY THE OTHER TWELVE CANNOT COVER IT. `caller_split_containment`, `host_memo_eviction` and the
    three arithmetic identities need `committed` conserved or flat; `contained_in_introduced_subgraph`
    refuses `Δcommitted > 0`; `introduced_wrapper_frames` admits `committed` only on introduced frames
    and here the twins commit NOTHING; `advanced_files_only` needs every moved file to have ADVANCED and
    here no file moved its verdict or its furthest — the rise is on files whose derivation is identical;
    `scope_gated_arm` needs scope markers; `profile_split_respelling` refuses any pre-existing rise;
    `typed_replacement_of_opaque_terminal` and `declared_alternatives_net_of_retired_work` need retired
    work covering the rise.

    ⭐ WHAT THIS ONE ASSERTS, two-sided. (1) `introduced` declares the rules the change INTRODUCED
    (plain names, new to the baseline), the guarded SIBLING(S) the twin consults (`!name`, pre-existing)
    and the alternative(s) the change UN-GATED (`~name`, pre-existing) — every name in the measured
    grammar's reference graph, at least one `!name`; (2) every guarded sibling is a PURE LOOKUP:
    `Δentries == Δmemo_hits ≥ 0` and `Δcommitted == 0` on it; (3) NO sampled file changed its verdict or
    its furthest position — nothing was accepted differently and nothing parsed further, so the run's
    `Δcommitted` is occurrences, not derivations; (4) every rule whose `committed` ROSE is reachable, in the
    reference graph, from a guarded sibling, an un-gated alternative or an introduced rule — the replayed
    subtree and the losing branch — or is an ANCESTOR of an un-gated alternative (a losing success is
    counted on its callers up to the choice point where it loses: `.3.49` measured `concurrent_assertion_item`,
    `assertion_item`, `checker_or_generate_item` +32 each beside `checker_instantiation` +32) rising by at most
    the un-gated rules' own committed rise — one frame per losing success, never more; nowhere else; (5) every rule whose `entries` ROSE is reachable from an
    introduced or un-gated rule — the twins' frames, their lookups, the un-gated alternative's own tries — or
    is a LOOP-CONTINUATION try: an un-gated item that now succeeds inside a body loop lets the loop try the
    parent's other alternatives once more (`.3.49`: `final_construct`, `checker_or_generate_item_declaration`,
    `kw_initial` +4 each), reachable from an ancestor of the un-gated rule and rising by at most the un-gated
    rules' committed rise. Fallers are reported, never refused.

    ⚠️⚠️ HONEST BOUND, IN THE PREDICATE'S OWN DOCSTRING. This proves the rise is an occurrence-count
    ARTEFACT — identical derivations, no advance, risers confined to the subtree the guard replays and the
    branch the un-gating lets succeed. It does NOT price the artefact away: `committed` counts a replayed
    subtree as if it were parsed, and only an instrument that separates replayed occurrences from fresh
    work can say the parsing cost is zero. That separation is routed (`ENGINE-UNIVERSAL-SERVICES.100`'s
    owed follow-up: a work-only counter beside the occurrence count), so a row under this predicate
    carries a number the metric inflates and says so.

    Raises `Refused` (never a false verdict) when no guarded sibling is declared, a name is missing from
    the graph or the measurements, an introduced name already existed, or a `!`/`~` name is new.
    """
    guards = [n[1:] for n in introduced if n.startswith("!")]
    ungated = [n[1:] for n in introduced if n.startswith("~")]
    intro = [n for n in introduced if not n.startswith(("!", "~"))]
    if not guards:
        raise Refused("the lookahead-replay predicate needs the guarded SIBLING the twin consults, "
                      "declared as `!name` in the row's introduced set; none is declared, so the replayed "
                      "subtree is unattributed")
    known = set(edges) | {r for refs in edges.values() for r in refs}
    unknown = [n for n in guards + ungated + intro if n not in known]
    if unknown:
        raise Refused(f"{unknown} name no rule in the grammar's reference graph")
    not_new = [n for n in intro if n in base]
    if not_new:
        raise Refused(f"{not_new} already existed in the baseline measurement — a plain name in the "
                      f"introduced set is a rule the change CREATED; a pre-existing rule is declared `!name` "
                      f"(guarded sibling) or `~name` (un-gated alternative), never waived by listing")
    missing = [n for n in guards + ungated if n not in base or n not in new]
    if missing:
        raise Refused(f"{missing} are declared as pre-existing (`!`/`~`) but are not measured on both sides")
    names = set(base) | set(new)
    origins = {r: origin_rule(r, known) for r in names}
    unplaced = sorted(r for r, o in origins.items() if o is None)
    if unplaced:
        raise Refused(f"{len(unplaced)} measured rule(s) cannot be located in the grammar's reference "
                      f"graph: {unplaced[:5]}{' …' if len(unplaced) > 5 else ''}")
    d = lambda r, k: new.get(r, {}).get(k, 0) - base.get(r, {}).get(k, 0)
    facts: dict = {"guards": {}, "moved_files": [], "escaped_committed": {}, "escaped_entries": {}}
    for g in guards:
        de, dm, dc = d(g, "entries"), d(g, "memo_hits"), d(g, "committed")
        facts["guards"][g] = (de, dm, dc)
        if de < 0 or de != dm or dc != 0:
            return (False, f"guarded sibling `{g}` is not a pure lookup: entries {de:+,} memo_hits {dm:+,} "
                           f"committed {dc:+,} — the guard parsed it afresh or a derivation accepted it more", facts)
    for path, b in base_files.items():
        n = new_files.get(path)
        if n is None:
            continue
        if b["accepted"] != n["accepted"] or (b["furthest"] is not None and n["furthest"] is not None
                                              and b["furthest"] != n["furthest"]):
            facts["moved_files"].append((path, b["accepted"], n["accepted"], b["furthest"], n["furthest"]))
    if facts["moved_files"]:
        p0 = facts["moved_files"][0]
        return (False, f"{len(facts['moved_files'])} sampled file(s) changed verdict or furthest (e.g. "
                       f"{os.path.basename(p0[0])} accepted {p0[1]}->{p0[2]} furthest {p0[3]}->{p0[4]}) — "
                       f"that is an advance, not a replay; the advanced-files predicate owns it", facts)
    scope_c = reachable(edges, guards + ungated + intro)
    scope_e = reachable(edges, intro + ungated)
    # (4b) A successful-but-losing branch is counted on its CALLERS too: the un-gated rule's success
    # propagates up the wrapper chain to the choice point where it loses (`checker_instantiation` →
    # `concurrent_assertion_item` → `assertion_item` → `checker_or_generate_item`, +32 each on `.3.49`),
    # and those ancestors are reached by the un-gated rule only upward. They are admitted when each
    # rises by AT MOST the un-gated rules' own committed rise — one frame per losing success, never more.
    rev: dict[str, list[str]] = {}
    for parent, refs in edges.items():
        for child in refs:
            rev.setdefault(child, []).append(parent)
    ancestors = reachable(rev, ungated) - set(ungated) if ungated else set()
    ungated_rise = sum(max(0, d(u, "committed")) for u in ungated)
    rose_c = {r: d(r, "committed") for r in names if d(r, "committed") > 0}
    rose_e = {r: d(r, "entries") for r in names if d(r, "entries") > 0}
    facts["ancestor_frames"] = {r: v for r, v in rose_c.items()
                                if origins[r] not in scope_c and origins[r] in ancestors and v <= ungated_rise}
    facts["escaped_committed"] = {r: v for r, v in rose_c.items()
                                  if origins[r] not in scope_c and r not in facts["ancestor_frames"]}
    # (5b) An un-gated item that now SUCCEEDS inside a body loop lets the loop continue and try the
    # parent's other alternatives once more at the next position (`.3.49`: `final_construct`,
    # `checker_or_generate_item_declaration`, `kw_initial` +4 each) — tries reachable from an ANCESTOR of
    # the un-gated rule, admitted when each rises by at most the un-gated rules' committed rise: one try
    # per success, never more.
    from_ancestors = reachable(edges, sorted(ancestors)) if ancestors else set()
    facts["loop_continuation_tries"] = {r: v for r, v in rose_e.items()
                                        if origins[r] not in scope_e and origins[r] in from_ancestors and v <= ungated_rise}
    facts["escaped_entries"] = {r: v for r, v in rose_e.items()
                                if origins[r] not in scope_e and r not in facts["loop_continuation_tries"]}
    facts["rose_committed"] = rose_c
    facts["rose_entries"] = rose_e
    facts["fell"] = {r: d(r, "entries") for r in names if d(r, "entries") < 0}
    if facts["escaped_committed"]:
        worst = sorted(facts["escaped_committed"].items(), key=lambda t: -t[1])[:3]
        return (False, f"{len(facts['escaped_committed'])} rule(s) committed more OUTSIDE the sub-graph "
                       f"reachable from {' '.join(guards + ungated + intro)} and not an ancestor of an "
                       f"un-gated rule within its rise ({ungated_rise:+,}): "
                       + ", ".join(f"{r} {v:+,}" for r, v in worst), facts)
    if facts["escaped_entries"]:
        worst = sorted(facts["escaped_entries"].items(), key=lambda t: -t[1])[:3]
        return (False, f"{len(facts['escaped_entries'])} rule(s) rose in entries OUTSIDE the sub-graph "
                       f"reachable from {' '.join(intro + ungated)} and not a loop-continuation try within "
                       f"the un-gated rise ({ungated_rise:+,}): "
                       + ", ".join(f"{r} {v:+,}" for r, v in worst), facts)
    if not rose_c and not rose_e and not any(v[0] for v in facts["guards"].values()):
        return (False, "nothing rose, so there is no replay for this predicate to explain", facts)
    detail = ("lookahead replay: " + "; ".join(f"`{g}` entries {v[0]:+,} = memo_hits {v[1]:+,}, committed {v[2]:+,}"
                                              for g, v in facts["guards"].items())
              + f"; every sampled file keeps its verdict and furthest; {len(rose_c):,} rule(s) committed more "
              f"({sum(rose_c.values()):+,}), all inside the {len(scope_c):,}-rule sub-graph the guard replays "
              f"or the un-gated branch reaches"
              + (f" or one of {len(facts['ancestor_frames'])} ancestor frame(s) of the un-gated rule within its rise ({ungated_rise:+,})" if facts["ancestor_frames"] else "")
              + f"; {len(rose_e):,} rule(s) rose in entries ({sum(rose_e.values()):+,}), "
              f"all reachable from the introduced / un-gated rules"
              + (f" or {len(facts['loop_continuation_tries'])} loop-continuation tries within the un-gated rise" if facts["loop_continuation_tries"] else "")
              + f"; {len(facts['fell']):,} fell "
              f"({sum(facts['fell'].values()):+,}) — an occurrence-count artefact, not re-done work")
    return (True, detail, facts)


def self_test_lookahead_replay_occurrences() -> int:
    """Arms for the LOOKAHEAD-REPLAY-OCCURRENCES predicate — every refusal and failure shape, no parser."""
    def rc(e, c, m):
        return {"entries": e, "committed": c, "memo_hits": m}
    def fr(acc, e, c, m, fur):
        return {"accepted": acc, "entries": e, "committed": c, "memo_hits": m, "furthest": fur}
    edges = {"top": ["g", "u", "y"], "g": ["x"], "u": ["p"], "p": ["x"], "y": ["z"], "t": ["g", "u"]}
    base = {"top": rc(100, 50, 0), "g": rc(40, 20, 10), "x": rc(60, 30, 5), "u": rc(40, 4, 0), "p": rc(10, 4, 0),
            "y": rc(30, 15, 2), "z": rc(30, 15, 0)}
    bfiles = {"a.sv": fr("yes", 200, 90, 17, None), "b.sv": fr("no", 110, 48, 0, 300)}
    def fresh(**over):
        n = {k: dict(v) for k, v in base.items()}
        n["t"] = rc(40, 0, 0)                      # the introduced twin: frames, no commits
        n["g"] = rc(80, 20, 50)                    # the guard: +40 entries = +40 memo hits, committed flat
        n["x"] = rc(60, 45, 5)                     # the replayed subtree: committed +15 (reachable from g)
        n["u"] = rc(48, 6, 0)                      # the un-gated branch: tries +8, committed +2
        n["p"] = rc(14, 6, 0)                      # reachable from u
        for k, v in over.items():
            n[k] = v
        return n
    nfiles = {k: dict(v) for k, v in bfiles.items()}
    arms, failed = [], []
    def arm(name, cond):
        arms.append(name); print(f"  {'✓' if cond else '✗'} {name}")
        if not cond: failed.append(name)
    intro = ["t", "!g", "~u"]
    h, dd, f = lookahead_replay_occurrences(base, fresh(), edges, intro, bfiles, nfiles)
    arm("holds: guard is a pure lookup, no file moved, committed risers inside the replayed / un-gated sub-graph", h and f["guards"]["g"] == (40, 40, 0))
    h, dd, f = lookahead_replay_occurrences(base, fresh(g=rc(80, 20, 30)), edges, intro, bfiles, nfiles)
    arm("fails: the guard parsed its sibling afresh (entries > memo hits)", not h and "pure lookup" in dd)
    h, dd, f = lookahead_replay_occurrences(base, fresh(g=rc(80, 25, 50)), edges, intro, bfiles, nfiles)
    arm("fails: the guarded sibling committed more", not h and "pure lookup" in dd)
    h, dd, f = lookahead_replay_occurrences(base, fresh(), edges, intro, bfiles, {"a.sv": dict(bfiles["a.sv"]), "b.sv": fr("no", 110, 48, 0, 420)})
    arm("fails: a sampled file advanced (furthest forward) — the advanced-files predicate owns it", not h and f["moved_files"] and "advance" in dd)
    h, dd, f = lookahead_replay_occurrences(base, fresh(), edges, intro, bfiles, {"a.sv": fr("no", 200, 90, 17, 50), "b.sv": dict(bfiles["b.sv"])})
    arm("fails: a sampled file changed verdict", not h and f["moved_files"])
    h, dd, f = lookahead_replay_occurrences(base, fresh(y=rc(30, 20, 2)), edges, intro, bfiles, nfiles)
    arm("fails: a committed riser OUTSIDE the replayed / un-gated sub-graph", not h and f["escaped_committed"] == {"y": 5})
    h, dd, f = lookahead_replay_occurrences(base, fresh(top=rc(100, 52, 0)), edges, intro, bfiles, nfiles)
    arm("holds: an ANCESTOR of the un-gated rule committed more, within the un-gated rule's own rise", h and f["ancestor_frames"] == {"top": 2})
    h, dd, f = lookahead_replay_occurrences(base, fresh(top=rc(100, 53, 0)), edges, intro, bfiles, nfiles)
    arm("fails: an ancestor of the un-gated rule committed MORE than the un-gated rule's rise", not h and f["escaped_committed"] == {"top": 3})
    h, dd, f = lookahead_replay_occurrences(base, fresh(z=rc(40, 15, 0)), edges, intro, bfiles, nfiles)
    arm("fails: an entries riser unreachable from the introduced / un-gated rules", not h and f["escaped_entries"] == {"z": 10})
    h, dd, f = lookahead_replay_occurrences(base, fresh(y=rc(32, 15, 2)), edges, intro, bfiles, nfiles)
    arm("holds: a sibling tried once more by the continued loop, within the un-gated rule's rise", h and f["loop_continuation_tries"] == {"y": 2})
    h, dd, f = lookahead_replay_occurrences(base, fresh(y=rc(33, 15, 2)), edges, intro, bfiles, nfiles)
    arm("fails: a sibling tried MORE times than the un-gated rule succeeded", not h and f["escaped_entries"] == {"y": 3})
    h, dd, f = lookahead_replay_occurrences(base, {k: dict(v) for k, v in base.items()} | {"t": rc(0, 0, 0)}, edges, intro, bfiles, nfiles)
    arm("fails: nothing rose — no replay to explain", not h and "nothing rose" in dd)
    for name, sc in (("refuse: no guarded sibling declared", ["t", "~u"]),
                     ("refuse: a name the graph lacks", ["t", "!g", "!nope"]),
                     ("refuse: an introduced name that already existed", ["x", "!g"]),
                     ("refuse: a `!` name that is new to the measurement", ["!t"])):
        try:
            lookahead_replay_occurrences(base, fresh(), edges, sc, bfiles, nfiles); arm(name, False)
        except Refused:
            arm(name, True)
    print(f"LOOKAHEAD-REPLAY-OCCURRENCES-SELFTEST: arms={len(arms)} failed={len(failed)}")
    return 1 if failed else 0


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
            return (self_test_host_memo_eviction() | self_test_caller_split_containment()
                    | self_test_advanced_files_only() | self_test_introduced_wrapper_frames()
                    | self_test_scope_gated_arm() | self_test_typed_replacement_of_opaque_terminal()
                    | self_test_lookahead_replay_occurrences())

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
