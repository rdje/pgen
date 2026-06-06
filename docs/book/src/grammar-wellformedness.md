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
   branch-level form of "useless symbol." Three *sound* forms are detected: exact-duplicate and
   fixed-terminal-prefix (both hard gates today), and earlier-always-succeeds (a warning while the
   grammar is being cleaned). The general FIRST-set-domination heuristic is *deliberately omitted* as
   unsound for PEG (see "Where PGEN stands").
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
   a defining source — no "use of an undefined value." The synthesized-attribute half is enforced
   today: a return annotation that references a positional capture `$N` with no defining child is a
   hard validation error (`E_RET_POS_OUT_OF_RANGE`) that blocks parser generation under strict mode
   (the CI default). The consulted-*fact* half is requirement 7 below.
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

## The attribution rule (no unexplained residual)

The duality has a sharp operational consequence. When the stimuli generator **fails to reach** a
target — a branch, a rule, or any EBNF fragment — there are exactly **two** possible causes, and the
failure must be attributed to one of them. It is **never** silently accepted as a "residual":

1. **Generator deficiency** (a constructor gap) — the target genuinely *is* reachable, so the
   generator must be improved to witness it; **or**
2. **EBNF not well-formed** — the target is genuinely *unreachable* (a dead branch or rule), so the
   fix belongs in the *grammar*, not the generator.

The **linter is the adjudicator**: for the decidable cases it *proves* which of the two it is. The
undecidable remainder is flagged loudly on that exact target for manual adjudication — still never
silently accepted.

A subtlety worth stating explicitly, because it inverts the natural instinct: **an unreachable target
is first and foremost a signal that the *grammar* is ill-formed — not that the generator is weak.**
So the correct order of suspicion is *grammar-first*: when the generator can't reach something, run
the linter **before** adding generator machinery. If the linter proves the target unreachable, chasing
it in the generator would be wasted effort that could never succeed — the dead branch must be fixed at
the source. Only once the linter confirms the target really *is* reachable is it a generator gap.

This turns "literal-0 coverage" from a number to drive down into a **theorem**: coverage is complete
exactly when, for every target, the two proofs agree — every reachable target witnessed, every
unreachable target removed at the source. And it makes every uncovered target a *ticket* assigned to
either the grammar or the generator, rather than a shrug.

## Worked example: a real bug the linter caught

This is not hypothetical. The always-succeeds shadowing check found a genuine defect in the
SystemVerilog grammar — a cluster of sequence "boolean abbreviation" rules written like this:

```
consecutive_repetition := ( star const_or_range_expression )?   -> {kind: "star_range", range: $1}
                        | ( star )?                              -> {kind: "star"}
                        | ( plus )?                              -> {kind: "plus"}
```

Every alternative is individually wrapped `( … )?`. Because an optional can never fail, the **first**
alternative always succeeds, so PEG commits to it and the `[*]` and `[+]` forms (alternatives two and
three) are **unreachable** — and worse, the rule silently emits an empty `star_range` node on every
sequence expression even when there is no repetition at all. The signature is too regular to be
hand-written; it looks like an extraction artifact from translating the IEEE 1800 grammar out of the
LRM PDF, where an "optional" marker was attached to each alternative instead of to the construct as a
whole. The same shape recurs across several rule families (covergroup value-ranges, randsequence
productions, …).

The stimuli generator had been quietly failing to cover these branches — they were part of the
coverage residual we kept attributing to "the generator needs to try harder." The attribution rule
resolves it cleanly: the linter *proved* the branches unreachable, so the blame is **EBNF
well-formedness**, and the fix is a one-character-per-arm grammar correction (drop the spurious `?`;
the optionality already lives correctly at the caller, `( boolean_abbrev )?`). After the fix the
`[*]`/`[+]`/`[=n]`/`[->n]` forms parse, the junk empty nodes disappear, and the branches become
genuinely reachable — so the generator can witness them. Two proofs, made to agree.

**This was not a one-off.** Run across the whole SystemVerilog grammar, the always-succeeds check
surfaced **52** dead branches, and they cluster into a *systematic* class: the LRM-to-`.ebnf`
extraction repeatedly **dropped delimiters**, leaving wrapper rules that can match nothing and
therefore always succeed:

- `[ ]` dropped → `consecutive_repetition`, `covergroup_value_range` (the range forms `[lo:hi]`)
- `{ }` dropped → `rs_code_block` (`{ … }` randsequence block — one fix cleared **11** shadow
  sites), `module_path_concatenation` and `module_path_multiple_concatenation` (the cascade
  through them resolved the whole module-path family)

Restoring the delimiter is the LRM-grounded fix and frequently *also* closes a real parse gap (the
delimiter-less form couldn't accept real SystemVerilog). A second, smaller class was *lost ordering*:
the LRM grammar is an order-independent CFG, but a PEG must try the specific form before a nullable
general one — so a handful of rules (`let_formal_type`, `port`, the argument lists, …) just needed
their alternatives reordered specific-before-general. Together these cleared 44 of the 52.

The residual handful is the *honest* part of the picture: a few rules (`net_port_type`, the
port-headers) are genuinely ambiguous on a bare identifier — "is this name a net type, or a data
type?" — which no amount of reordering can settle. Those are flagged for **semantic store-gating**
(consult the fact store: *is this identifier a declared nettype?*), not silently dropped. That is the
attribution rule doing its job: every one of the 52 became a ticket — most "fix the grammar," a few
"needs a semantic gate" — and none a shrug.

## Trusting the linter: certificates, not faith

The linter is the **fulcrum** of the whole sign-off model — it is the *prover* of well-formedness, the
*adjudicator* of every generator reach-failure, and the thing that turns "literal-0 coverage" from a
number you chase into a property you prove. A tool with that much riding on it must be one you
**never have to doubt.** This section explains how that trust is *earned* — not asserted.

### Why "always give a definite yes/no" is impossible

For PGEN's grammar class — stateful, data-dependent PEGs with `@predicate`s — deciding whether an
arbitrary fragment is reachable is, in general, **undecidable.** This is a mathematical wall, not a
PGEN limitation:

- The purely *structural* question (which rules are reachable from the start symbol, as a graph) *is*
  decidable — classic reduced-grammar analysis (Hopcroft–Ullman), and PGEN's reachability check is
  complete there.
- But "is this *ordered-choice arm* ever selected" depends on whether some input makes the earlier
  arms fail and this one match — a PEG language-difference question, undecidable in general.
- And `@predicate`s make it strictly worse: "can this predicate ever be true" is asking whether a
  program state is reachable — Rice's-theorem territory, provably undecidable.

So no tool — ours or anyone's — can be **complete** (catch every dead fragment, always, with a
guaranteed definite answer). Any tool that *claims* to is lying.

### Sound, not complete — the right trade for a judge

The property we actually need, and *can* guarantee, is **soundness**: when the linter commits to a
verdict, that verdict is correct. It never declares a live fragment dead, and never declares a dead
fragment live. The price of soundness is an honest third answer — **`UNKNOWN`** — for the fragments
it cannot settle. That is not a wrong answer; it is the linter refusing to guess. A judge that never
convicts the innocent (even if it sometimes returns "not proven") is exactly what you want; the
opposite trade — a complete judge that sometimes convicts the innocent — would be worse than useless,
because you would "fix" branches that were never broken.

This is why every check PGEN ships is a *sound decidable subset* and why the unsound heuristics
(general FIRST-set domination, parse-order predicate reachability) are deliberately excluded: they
would buy completeness at the cost of soundness — the wrong direction.

### The mechanism: a certifying algorithm

Soundness on its own is a promise. To make it something you *verify* rather than *trust*, the linter
is built as a **certifying algorithm** (Mehlhorn, McConnell et al.): every verdict ships with a
checkable **certificate**, and a second, deliberately tiny program validates the certificate. You
then trust the small checker — which you can read in an afternoon — instead of the linter's complex
internals.

| Verdict | Certificate | How you check it |
|---|---|---|
| **reachable** | a **witness** — a concrete derivation *and* an input string that exercises the fragment | replay the input through the real parser; watch it hit the fragment |
| **unreachable** | a **proof** — the exact decidable argument (which sound rule fired, and the chain) | a small checker re-validates the argument |
| **`UNKNOWN`** | *(none — by design)* | it is an honest "I cannot prove this either way," never a guess |

The witness producer for the "reachable" case is the **stimuli generator** — this is the duality made
operational: the linter claims reachability, the generator *demonstrates* it. And the binding
discipline is: **no definite verdict without a certificate.** It is precisely because the linter
refuses to speak without a proof that you never have to doubt it when it does.

### Undecidability lives in `UNKNOWN` — and we drain it on the grammar we ship

The undecidability theorem is about *all possible grammars*. It does **not** stop us from fully
certifying the *one* grammar we sign off. The two proofs close the gap:

- For the actual grammar, we drive the `UNKNOWN` bucket to **zero** — every fragment is either
  witnessed-reachable or proven-unreachable.
- Anything stuck in `UNKNOWN` (the linter can't prove it dead *and* the generator can't witness it)
  is the flagged ticket from the attribution rule. A human adjudicates it once: the resolution either
  *produces a witness* (the grammar/generator is fixed so it becomes reachable) or *produces a proof*
  (it really is dead → removed at the source). Either way it leaves `UNKNOWN`.

When that bucket reaches zero with every certificate checking, **the shipped grammar is fully
certified**: you have not *trusted* the linter — you have *verified* every claim it made. That
certificate-coverage number, at zero `UNKNOWN`, is the objective, demonstrable statement that the
linter is trustworthy *on this grammar*.

> **The bottom line.** We do not aim for "100% complete" — that is provably impossible. We aim for
> **100% sound, with the unknown region driven to zero and never hidden.** That is achievable, it is
> provable, and it replaces *trust* with *verification* — which is stronger.

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
and fixed-terminal-prefix shadow freedom, and **structural reachability** ("no unreachable rules",
multi-entry-aware) — all hard gates — plus **attribute non-circularity**, which holds by construction
(the annotation language is synthesized-only).

It *also* now detects a second sound form of dead branch: an **earlier alternative that always
succeeds**. In an ordered choice `a | b`, if `a` can never fail (it is `e?`, `e*`, an all-optional
sequence, or a reference to such a rule), then PEG commits to `a` on every input and `b` is
unreachable. This is distinct from nullability — a lookahead `&e`/`!e` consumes nothing (nullable)
but *can* fail, so it never triggers this rule (no false positives). The analysis is deliberately
conservative: it flags a later branch only when the earlier one is *provably* always-succeeding,
never on a guess. It found 52 real dead branches in the SystemVerilog grammar (a recurring
"`( X )?` written as an alternative" mistake) — those are surfaced as warnings first and will become
a hard gate once the grammar is cleaned, exactly the way exact-duplicate shadowing was staged.

A note on what is **left out on purpose:** the *general* "FIRST-set domination" heuristic — "if
everything `b` could start with, `a` could also start with, then `b` is dead" — is **unsound for
PEG** and is intentionally not implemented. `a` might match the first token and then fail later, in
which case PEG *does* backtrack and try `b`, so `b` is live. Implementing the general heuristic would
falsely accuse live branches of being dead — the opposite of an honest linter. PGEN sticks to the
two *sound, decidable* forms (fixed-terminal-prefix and always-succeeds).

On the **well-*defined*** layer, all three axes are now enforced. Attribute non-circularity holds by
construction (synthesized-only annotations). Attribute completeness for synthesized attributes (`$N`)
is a hard validation error (above). And data-dependent **binding-before-use** is now checked: every
fact a `@predicate` consults must be establishable by some `@emit_fact`. Its sound, decidable core —
a consulted fact-*kind* that nothing ever emits — is a hard gate: `has_fact`, `lacks_fact`,
`fact_attribute_equals`, and `fact_count_at_least` all read the exact store `@emit_fact` populates, so
consulting a kind with no producer means the predicate can never be satisfied (the rule is dead, or
the kind is a typo). The check enumerates emitters across every annotation surface (so none is missed)
and only flags *literal* consulted kinds (a dynamic kind is skipped, never falsely accused). As with
the FIRST-domination exclusion, the *undecidable* refinement — proving a fact is established **earlier
in some actual parse**, not merely somewhere — is left out on purpose; the kind-existence core is the
sound part. With this, **the linter proves the decidable core of every axis in the contract above.**

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
