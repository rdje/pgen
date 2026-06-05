# Grammar Well-Formedness & Well-Definedness

> **Part II · Inside PGEN.** For contributors. This chapter explains what it means for a PGEN
> grammar to be *correct as a grammar* — and why that is the foundation under exhaustive stimuli
> coverage. It is the normative contract the grammar linter exists to prove.

## The principle

The grammar linter's purpose is **not** to emit a bag of warnings. Its purpose is to **prove that
an EBNF is well-formed and well-defined.** A grammar that passes the linter is a clean, well-defined
object; a grammar that doesn't has a *defect to fix at the source* — never a wart to tolerate.

This matters because of a consequence: **a well-formed, well-defined grammar has no unreachable
rules or branches** — every rule and every alternative has some input that exercises it. So
exhaustive stimuli coverage (every grammar branch witnessed) is not a separate mountain to climb;
it is the *observable consequence* of "the grammar is clean" plus "the generator is complete."

## The contract (what "well-formed / well-defined" means)

PGEN's grammars are a specific class — **stateful, data-dependent PEGs with attribute-style
annotations** — so the contract draws on four established theories, each contributing one
independent axis. (This is literature-grounded, not invented; sources at the end.)

### Well-formed (syntactic)

1. **Reduced — no useless symbols.** Every rule must be both *productive* (it can derive a finite
   terminal string) and *reachable* (from the start symbol). A rule that is neither is dead weight.
   *(Hopcroft–Ullman.)*
2. **Complete — it terminates on every input.** A PEG is well-formed when it has no direct or
   indirect left recursion (PGEN eliminates left recursion for you) and no repetition over an
   empty-matching body (which would loop without consuming). *(Ford, PEG, 2004.)*
3. **No dead branches.** In an ordered choice `a | b | …`, a later alternative is *shadowed* if an
   earlier one always matches first — it can never be selected, so it is unreachable. This is the
   branch-level form of "useless symbol." (Exact-duplicate shadowing is detected and gated today;
   the more general FIRST-set domination is in progress.)
4. **No dangling references, no profile orphans.** Every referenced rule is defined, and every rule
   present under a language profile (e.g. `sv_2017` vs `sv_2023`) is actually satisfiable under it.

### Well-defined (semantic)

PGEN's return annotations (`-> {a: $1, b: $2}`) and `@semantic_value` form an **attribute
grammar**, and its `@predicate`/`@emit_fact` store makes it **data-dependent** — so two more axes
apply:

5. **Attribute non-circularity.** No annotation value may depend, transitively, on itself. A
   circular attribute definition makes the grammar's *meaning* undefined. *(Knuth, 1968 — the
   circularity test is famously expensive in the worst case but cheap on real grammars.)* **PGEN
   satisfies this by construction:** its return annotations are purely *synthesized* (`$N` refers
   only to a rule's own children, bottom-up) — there is no inherited (top-down) attribute construct,
   and a synthesized-only attribute grammar cannot be circular. So this needs no runtime check; the
   proof is the annotation language's design. (The store-based `@predicate`/`@emit_fact` flow is a
   *separate*, data-dependent axis — see requirement 7.)
6. **Attribute completeness.** Every attribute or binding a rule reads (`$N`, a consulted fact) has
   a defining source — no "use of an undefined value."
7. **Binding before use.** Every `@predicate` may only consult facts that *can be established
   earlier* in some parse. A rule gated on a fact that nothing can ever emit before it is, in
   effect, dead. *(Jim, Mandelbaum, Walker — data-dependent grammars, 2010.)*

A grammar satisfying 1–4 is **well-formed**; satisfying 5–7 as well is **well-defined**.

## The duality: two proofs of the same property

Reachability is provable two independent ways, and PGEN uses **both**:

- the **linter** proves it *statically* — "a witness *exists* for every rule/branch";
- the **stimuli generator** proves it *constructively* — "here *is* the witness for every one."

They must agree, and the disagreement localizes the bug:

| Linter says | Generator says | Meaning |
|---|---|---|
| reachable | witnessed | ✅ proven twice |
| reachable | can't witness | a **generator/constructor** gap (never an accepted dead end) |
| unreachable | covers it | a **linter** bug |

Both green on the same grammar = reachability proven statically *and* constructively = signoff-grade.
**Exhaustive coverage ("literal-0" uncovered branches) is exactly the point where these two proofs
meet.**

## The decidability boundary (an honest limit)

Full reachability and language-inclusion are undecidable, so the linter only ever proves the
*decidable* forms (structural reachability, exact-duplicate and FIRST-domination shadowing,
termination, profile consistency, attribute circularity). It **never guesses** a rule is unreachable
— if it cannot prove it, the rule is presumed reachable and must be witnessed. Anything that is
neither linter-provable-unreachable nor generator-witnessable is a *loud, specific flag* on that
exact rule — "fix the constructor, or this is a subtle dead branch" — never a silent acceptance.
This is what keeps the coverage number honest: it can never be gamed by quietly reclassifying a
branch as "doesn't count."

## Where PGEN stands

The linter (`--lint-grammar`) already proves: termination, profile-orphan freedom, exact-duplicate
shadow freedom, and **structural reachability** ("no unreachable rules", multi-entry-aware) — all
hard gates — plus **attribute non-circularity**, which holds by construction (the annotation
language is synthesized-only). In progress: FIRST-domination shadowing and the rest of the
well-*defined* layer (attribute completeness, binding-before-use).

On the **constructive side**, the generator's coverage measurement is now **deterministic**: the
generation budget was changed from a wall-clock timeout to a fixed step counter, so a seeded run
produces the *same* residual every time. That matters because a *completeness* number that wobbled
run-to-run could never be driven to a hard zero — it has to be a stable signal first. With that in
place, the static proof (linter) and the constructive proof (generator) can finally be compared on
equal, reproducible footing. The design rationale and ordered build plan live in the
`GRAMMAR-WELLFORMED` task tree.

## Extending: adding a new annotation tag-kind

Profile tags (`@profiles`) are the first *tag-kind* whose values **compose across rule references**
— a rule present under a profile must be *satisfiable* under it (else it's an orphan), and the
linter both checks that and suggests the derived minimal fix. The composition machinery is written
to be **tag-agnostic**: a tag-kind is defined by four things — its value domain, its composition
algebra (how a sequence/alternation/quantifier/reference combine the tag), its consistency
invariant, and its resolution policy. A second tag-kind plugs in by following the same shape — a
*derive* function (cf. `derive_rule_profiles`) and a *check* function (cf. `detect_profile_orphans`)
over the AST, wired into the linter — with no rewrite of the existing checks. The concrete recipe
and the worked `@profiles` instance live in the composition-doctrine decision record; a generic
runtime registry is deliberately deferred until a second real tag-kind exists (so the abstraction is
extracted from two instances, not guessed from one).

## Sources

- B. Ford, *Parsing Expression Grammars: A Recognition-Based Syntactic Foundation*, POPL 2004.
- S. Medeiros et al., *Left Recursion in Parsing Expression Grammars*, 2012.
- Hopcroft & Ullman, *Introduction to Automata Theory* — reduced grammars / useless symbols.
- D. Knuth, *Semantics of Context-Free Languages*, 1968; Jazayeri, Ogden & Rounds, *The
  intrinsically exponential complexity of the circularity problem for attribute grammars*, CACM 1975.
- T. Jim, Y. Mandelbaum, D. Walker, *Semantics and Algorithms for Data-Dependent Grammars*, POPL 2010.
