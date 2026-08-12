---
name: project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime
description: "DESIGN DECISION (2026-08-12, ENGINE-UNIVERSAL-SERVICES.13 acceptance (b), prior-art-grounded): PGEN closes INDIRECT left recursion by a gen-AST -> gen-AST ELIMINATION pass at generation time, NOT by Warth/Medeiros-style seed growing at runtime. The literature offers both and both are in production (CPython's PEG parser grows seeds; ANTLR4 rewrites and REFUSES indirect LR). PGEN's second non-negotiable decides it: a runtime protocol taxes every parse forever, and the affected rules are measurably hot — the cast/call knot is 15 of 1481 SV rules but 2.85-3.29% of all rule entries across three diverse corpus files, ~3x its fair share. Elimination is free at parse time by construction. ANTLR4's stated objection (exponential blow-up) is priced here against a MEASURED surface of 3 knots, not an arbitrary grammar."
id: project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime
title: "Indirect left recursion is ELIMINATED at generation, never grown at runtime — the prior art ships both and disagrees, so PGEN's peak-speed non-negotiable is the tie-breaker"
date: 2026-08-12
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .13 slices 1-2; docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/{canonicalize_lint_cycles.py,adjudicate.py,README.md}; docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/; Warth et al. PEPM 2008; Medeiros et al. arXiv 1207.0443 / SCP 2014; Tratt 2010; PEP 617; antlr/antlr4#522
reverify: "python3 docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/canonicalize_lint_cycles.py | grep -q '30 reported rule rows -> 7 DISTINCT cycles' && python3 docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/adjudicate.py >/dev/null && echo ELIMINATE-AT-GENERATION"
answers:
  - "does PGEN eliminate indirect left recursion at generation time or grow the seed at runtime"
  - "why not use Warth's packrat left-recursion algorithm in PGEN"
  - "what do CPython and ANTLR4 do about indirect left recursion"
  - "the prior art is split between two mechanisms — how do I choose"
  - "how hot are the rules in SystemVerilog's left-recursive cast knot"
  - "is ANTLR4's exponential-blow-up objection to left-recursion elimination decisive here"
  - "what is NOT authorised by the indirect-left-recursion design decision"
metadata:
  node_type: memory
  type: project
  created: 2026-08-12
---

**The question `ENGINE-UNIVERSAL-SERVICES.13` acceptance (b) had to settle:** when PGEN gains
indirect left-recursion support, does it **eliminate the cycle at generation time** (a gen-AST →
gen-AST rewrite, the way the existing direct/wrapper pass already works) or **grow the seed at
runtime** (the packrat technique, where the memo table iterates a left-recursive rule to a fixpoint)?

## PRIOR ART (searched 2026-08-12; [[feedback_read_prior_art_before_designing]])

**Runtime seed growing.**
[Warth, Douglass & Millstein, *Packrat Parsers Can Support Left Recursion*, PEPM 2008](https://web.cs.ucla.edu/~todd/research/pepm08.pdf)
is the origin: the memo entry for a left-recursive rule is seeded with a failure and re-evaluated
until the match stops growing, extended to indirect/mutual cycles by tracking a *head* rule and an
*involved set*. [Medeiros, Mascarenhas & Ierusalimschy](https://arxiv.org/abs/1207.0443)
([SCP 2014](https://www.sciencedirect.com/science/article/pii/S0167642314000288)) give the cleaner
semantics — left recursion as *bounded* recursion — with the same runtime shape.
[Tratt (2010)](https://tratt.net/laurie/research/pubs/papers/tratt__direct_left_recursive_parsing_expression_grammars.pdf)
is the cautionary half: Warth's algorithm yields *surprising* parses on indirect cycles, and Tratt
deliberately restricts his own treatment to the **direct** case.

**Generation-time elimination.** Paull's classical transform, and the modern refinement
[*Eliminating Left Recursion without the Epsilon*](https://arxiv.org/html/1908.10888v6). The known
cost is grammar blow-up plus a changed parse-tree shape — *"indirect left recursion is harder to
eliminate, and its elimination changes the structure of the grammar and the resulting trees even
more compared to direct left recursion."*

**Both are shipping, and they disagree.**

- **CPython** ([PEP 617](https://peps.python.org/pep-0617/), `pegen`) grows seeds in the memo cache
  — *"closer to the approach described in Warth et al."* — and explicitly supports **indirect and
  mutual** left recursion. It has parsed Python since 3.9.
- **ANTLR4** rewrites **direct** left recursion before generating the parser and **refuses**
  indirect left recursion with an error. Per [antlr/antlr4#522](https://github.com/antlr/antlr4/issues/522)
  this is a deliberate engineering decision: direct LR covers the common cases (arithmetic
  expressions, C declarators) and the standard elimination algorithm's exponential blow-up was
  judged *"wholly unworkable in practice"*.

⇒ the literature does **not** settle it. A project's own constraints have to.

## THE DECISION — eliminate at generation

PGEN's second non-negotiable is decisive: **peak speed, where costs are REJECTED rather than
traded** ([[project_north_star]], [[project_capability_growth_is_zero_cost_and_neutral]]). Seed
growing is a per-parse protocol on the memo path of every rule in a cycle, paid on every input for
the life of the parser. Elimination is a build-time rewrite: **zero parse-time cost by
construction**, which is exactly the axis `ENGINE-UNIVERSAL-SERVICES.7`'s feature taxonomy uses to
price an engine feature.

⭐ **And "the affected rules are cold, so who cares" does not survive measurement.**
`--dump-rule-entry-counts-json` over three diverse corpus files:

| file | knot-A entries | total rule entries | share |
|---|---:|---:|---:|
| `top_earlgrey_rnd_cnst_pkg.sv` | 7 076 | 215 332 | **3.29 %** |
| `friscv_rv32i_platform.sv` | 16 596 | 582 383 | **2.85 %** |
| `prim_sha2_pkg.sv` | 11 661 | 360 453 | **3.24 %** |

Knot A (the `casting_type ↔ constant_primary` cast/call cluster) is **15 of SystemVerilog's 1 481
rules — 1.0 %** — and carries ~3 % of all rule entries, about **three times its fair share**. A
runtime protocol on that set is a tax on the hot path, not on a corner.

⭐ **ANTLR4's objection is real but is priced against the wrong denominator here.** Exponential
blow-up is a property of eliminating an *arbitrary* grammar. `ENGINE-UNIVERSAL-SERVICES.13` slice 1
measured PGEN's actual surface: SystemVerilog's 30 reported rule rows are **7 distinct cycles**,
`ebnf`'s 5 are **3**, and those 10 collapse to **3 knots** — `casting_type → constant_primary`,
`property_expr implies property_expr`, and `return_expression`. The blow-up must be priced against
**3 rules**, and if a future grammar presents a knot where it genuinely explodes, the honest answer
is ANTLR4's — refuse it with a named diagnostic — not to buy a runtime protocol for every grammar
that does not need one.

## THREE PGEN-SPECIFIC REASONS THE SAME WAY

1. **The fold already exists and is the hard part.** `ENGINE-UNIVERSAL-SERVICES.8` built
   `lr_chain_fold`, which rebuilds the author's **declared** left-nested AST from the eliminator's
   internal chain. Elimination therefore inherits a solved AST-shape problem; the *"elimination
   changes the resulting trees"* objection is the one PGEN has already paid for once. Generalising
   that fold from one rule to a mutually-recursive **set** is the real work of the fix.
2. **EBNF-as-sole-source-of-truth is preserved either way, and only by a gen-AST rewrite.** The
   grammar keeps transcribing Annex A verbatim; the rewrite is invisible to the author. This is the
   same ruling that already refused a hand-written grammar-tier repair
   ([[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]]), and it applies with
   equal force to *"just restructure `casting_type`"*.
3. **Tratt's caution lands on the option PGEN would otherwise pick.** Seed growing is the technique
   whose *indirect* case is documented as producing surprising parses. PGEN's SV grammar is held to
   100 % LRM compliance by default, where an unexpected-but-accepting parse is a defect, not a
   convenience.

## AMENDMENT (2026-08-12, session #222 — `.13` slice 3, `PGEN-ENGINE-UNIVERSAL-SERVICES-0013`)

The decision **stands** — generation-time elimination was measured to work on the isolating
synthetic — but two of its details were wrong by omission, and both were found by measurement rather
than argument (`docs/tasks/artifacts/engine_universal_services/indirect_lr/README.md`):

1. ⛔ **The rule to eliminate at is the CONSUMER rule, not the rule the lint names.** Eliminating at
   `casting_type` — the rule every lint row prints first, and the one this record's own prose keeps
   naming — is a **regression**: measured on the synthetic, one-level `t'(n)` goes accept → reject
   and nothing recovers. PGEN's `*` is greedy and does not backtrack its iteration count
   (`generated/systemverilog_parser.rs:7463`), so an eliminated `casting_type` swallows the whole
   cast chain and `constant_cast`'s own trailing `' ( … )` can never match. Nothing outside the
   cycle ever asks for a `casting_type`; a SystemVerilog expression asks for a `constant_primary`,
   and that is the rule that must absorb the chain.
2. ⭐ **The blow-up is one CLONE per intermediate rule on the cycle path — and each clone is a new
   rule name in the typed AST.** The working transform needs `base` to include a copy of each
   intermediate with the cycle edge removed (`constant_cast` re-emitted over a `casting_type` shorn
   of its `constant_primary` arm). That is far short of the exponential closure ANTLR4 objects to,
   so the "3 knots, not 30 cycles" pricing above survives — but the AST-shape cost is real and was
   not priced here. `.13` (d) therefore carries an `ast_shape_contract` obligation, not only parse
   verdicts. ⛔ `lr_chain_fold` (point 1 below) rebuilds the DECLARED shape for the *eliminated*
   rule; it says nothing about a clone appearing under a new name, which is a separate question.

## HONEST BOUNDS

- This decides the **mechanism**, not the algorithm's details: which elimination transform, how the
  fold generalises to a set of rules, and what the linter says when a knot is refused are still
  open and belong to `.13`'s design/implementation slices. ⭐ The amendment above closes one of
  those details (WHICH rule absorbs the chain) and opens another (what the clone does to the AST).
- The entry-count table is a **share of rule entries**, not a share of wall time; it establishes the
  knot is hot, not the exact price of a runtime protocol. Nobody has built the runtime arm to A/B it
  — and under a *"costs are rejected, not traded"* rule, the burden sits on the arm that adds
  parse-time work.
- If a real grammar ever presents a knot whose elimination genuinely explodes, this decision does
  **not** pre-authorise buying seed growing to rescue it. Reopen the question with that grammar's
  measurement, and consider ANTLR4's answer first.

Related: [[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]],
[[project_north_star]], [[feedback_read_prior_art_before_designing]],
[[project_capability_growth_is_zero_cost_and_neutral]].
