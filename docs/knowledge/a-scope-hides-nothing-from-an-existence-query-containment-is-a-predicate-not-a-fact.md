---
id: a-scope-hides-nothing-from-an-existence-query-containment-is-a-predicate-not-a-fact
title: A scope hides nothing from an existence query — to make a name legal only inside a region, gate the region with in_scope_kind; do not emit a fact and expect the scope to hide it
answers:
  - "I emitted a fact inside a scope and closed the scope — why does has_fact still find it"
  - "is has_fact scope-aware; which semantic-store predicates walk the active scope chain"
  - "how do I make a name legal only inside a region (a cross body, a block) in a PGEN grammar"
  - "why did SV-CORPUS-GRAD.3.29 use in_scope_kind instead of @emit_fact for CrossValType and CrossQueueType"
  - "why can't has_fact simply be made chain-visible (the regex grammar depends on the store-wide reading)"
  - "where do @open_scope and @close_scope bind so the scope covers the region (open-marker rule idiom)"
  - "the certificate says my new rule is UNKNOWN because the planner renders a synthetic identifier — how do I make a recognizer witnessable"
  - "the parse-cost ratchet refused my grammar fix — dispatcher first vs ordered second branch vs pre vs post predicate, which spelling is cheapest and why"
  - "why does a pre predicate on a refusal path make the packrat memo re-run everything (epoch-tainted cached failures)"
  - "how do I A/B two grammar spellings for parse cost without a release build (debug probe, protocol-graph counters, per-file entries.tsv baseline)"
tags: [semantic-store, scopes, predicates, engine, grammar-authoring, sv-corpus, over-acceptance, claim-verification]
date: 2026-09-03
status: current
evidence: "SV-CORPUS-GRAD.3.29 (PGEN-SV-CORPUS-GRAD-0347): the pre-registered fix (@open_scope + two @emit_fact on the cross open-marker, @close_scope on the close-marker) flipped the two fixed pins AND the invalid pin `CrossQueueType q;` after the endgroup — trace: 🔓 open ScopeId(1), 📥 both facts scope_id=ScopeId(1), 🔒 close OK, then has_fact(type_name, CrossQueueType) → true from ScopeId(0). Source: has_fact = fact_index.any_with_name, 'across the whole store … in any scope_depth'. Reproduced on a six-rule synthetic with no SystemVerilog in it (`{x}T` accepts, control `T` rejects). Routed: ENGINE-UNIVERSAL-SERVICES.89. Shipped: in_scope_kind(cross) as a post predicate on a terminal recognizer (a terminal, not a lookahead: the certificate planner renders a terminal from its regex and witnesses it from a cross_body union entry; post, not pre: a pre gate that reads the store on the refusal path epoch-taints the callers' cached failures — measured by the parse-cost ratchet) — the invalid pin stays REJECT, the fixed pins ACCEPT, the AST is byte-identical."
reverify: "bash docs/tasks/artifacts/engine_universal_services/eus89_store_wide_has_fact/probe.sh   # 2/2: the closed-scope fact is found (A1) and the gate can refuse (A2 RED control)"
---

**The scope machinery does everything the book says — open, emit-into, close, roll back — and the
existence query ignores all of it.**

## The rule

`has_fact`, `lacks_fact`, `has_fact_attribute`, `fact_attribute_equals`, `lacks_fact_attribute_equals`
and `fact_count_at_least` answer over the **whole store**: any scope, open or closed
(`fact_index.any_with_name`, keyed by `(scope_depth, name)`, never consulting `active_chain`). The
scope-aware primitives are `in_scope_kind` / `not_in_scope_kind` (walk the active chain),
`has_fact_in_current_scope` (the innermost open scope's DEPTH only) and `resolve_path`. So:

- A fact emitted inside a scope that later closes is **still found** from the enclosing scope. A
  grammar cannot use `@emit_fact` + `@open_scope`/`@close_scope` to declare a name *only inside* a
  region; the name leaks into the rest of the file — over-acceptance.
- To make a name legal only inside a region, **gate the region, not the name**: open a scope at the
  region's opener token (a small open-marker rule, because `@open_scope` fires as an effect *after*
  its rule's body), close it at the closer, and put `@predicate: { name: in_scope_kind, args: [<kind>],
  phase: post }` on the rule that recognises the name — `post`, so the store is read only once the
  text has matched: a `pre` gate on a hot refusal path is a store read that epoch-taints every
  caller's cached failure (measured by the parse-cost ratchet). That is a predicate over the chain, so it is
  right by construction and needs no engine change. It is how the regex grammar rejects `\K` inside
  a lookaround and how SystemVerilog admits `CrossValType` / `CrossQueueType` inside a `cross` body
  (IEEE 1800-2017 §19.6.1.3) and nowhere else.
- **Do not "fix" `has_fact` to walk the chain.** `grammars/regex.ebnf` checks at `phase: final` that a
  named back-reference names a *defined* group, and a group defined inside a lookaround lives in a
  scope that closed long before parse completion. The store-wide reading is load-bearing; the missing
  thing is an *opt-in* chain-visible query or a per-`@fact_kind` `visibility:` — owned by
  `ENGINE-UNIVERSAL-SERVICES.89`.

## How it was caught, and why that matters more than the rule

The fix was pre-registered in the leaf as "the typedef idiom, scoped". Built, it flipped both fixed
pins to ACCEPT — a green that would have shipped — and the one pin written to fail *if the scope did
not contain the names* went green too. Pair every accept-set widening with an **invalid pin that the
fix must keep rejecting**; the pin is what turned a plausible design into a measured leak, and it now
guards the shipped one forever (`stimuli/sv/adjudication_repros/invalid_cross_predefined_type_after_cross.sv`,
`class=invalid`).

## Three refusals, three measurements — how the shipped spelling was found

The same slice had three instruments refuse three spellings of one two-name recognizer, and every
refusal named its mechanism:

1. **Certificate.** A *lookahead* over `type_identifier` (`&cross_predefined_type_name type_identifier`)
   was UNKNOWN in both bases and cost the union its `fully_certified`: the witness planner renders the
   identifier synthetically (`sample="localparam\foo \foo ;"`), so the lookahead refuses before any
   gate. A **terminal** spelling the names is renderable from its own regex — and it is witnessed from
   the union entry that *opens the scope* (`cross_body:sv_2017`), never from the canonical entry.
2. **Ratchet, totals.** A dispatcher rule tried FIRST at every type use-site cost `entries` +2 419 875:
   two entries per attempt, and on `pinmux_reg_top.sv` the store gate accepts none of its 22 889. The
   check belongs BEHIND the store gate, as an ordered second branch at the callers.
3. **Ratchet, per-rule table.** The ordered branch with a `pre` scope gate still cost +2 154 348, and
   `rule_costs.tsv` said why: every `data_type` alternative re-entered ~44 900 times, deep parents
   losing memo replays. A `pre` predicate that reads the store on the REFUSAL path is a store read,
   and a failure that read the store is cached epoch-stamped (`MEMO-STORE-SOUNDNESS.2`) — every later
   fact emission re-ran it. As a `post` predicate the store is read only once the text has matched;
   the residual is the refused-attempt path itself (+634 818 entries in exactly three rules,
   `committed` 0, nothing fell), accepted through the coded `contained_in_introduced_subgraph` invariant.

⭐ A debug `parseability_probe` gives the SAME protocol-graph counters as the release one, and the
tracked per-file `entries.tsv` is a HEAD baseline you already have — an A/B of two spellings costs a
4-minute debug build, not a 20-minute release link (`ENGINE-UNIVERSAL-SERVICES.90` owns the
engine-tier elimination of the residual).

Canonical homes: `docs/book/src/semantic-store.md` §2, §6, §7 (corrected the same day: the chapter
had promised chain-walking for every query); `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` `.89`;
`docs/tasks/SV-CORPUS-GRAD.md` `.3.29`.
