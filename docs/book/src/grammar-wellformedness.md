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
   indirect left recursion and no repetition over an empty-matching body (which would loop without
   consuming). *(Ford, PEG, 2004.)* ⭐ PGEN **eliminates** the wrapper shape and — since `A2.5` — the
   inline direct shape, so those you never think about. An **indirect** cycle is a different matter:
   nothing eliminates it, the runtime guard *rejects* rather than handles it, and the lint reports it
   as `left_recursion_unhandled` (see *the left-recursion verdict is derived, not asserted*, below).
3. **No dead branches.** In an ordered choice `a | b | …`, a later alternative is *shadowed* if an
   earlier one always matches first — it can never be selected, so it is unreachable. This is the
   branch-level form of "useless symbol." Exact-duplicate is detected as a hard gate.
   Fixed-terminal-prefix is a hard gate **only where its premise holds** — a rule whose effective
   `@branch_policy` is `ordered` (first-success commit) with no branch-phase predicates; under the
   default `longest_match` (and `priority_first`) the engine runs the full tournament and the later
   alternative is *live*, so no finding is emitted (**A2.3**, live-proven on the parse harness). A
   third form, *earlier-always-succeeds*, was once staged as a warning for promotion —
   but a 2026-07-05 tools-first audit found it **unsound for PGEN's backtracking engine** (it flagged
   a *live* branch as dead; see the correction under the worked example), so as of **A2.2** its
   shadowing *verdict* was removed: it survives only as a non-gating, non-verdict `[note]`
   (`always_succeeds_alternatives`), and its planned promotion to a hard gate is **retired**. The
   general FIRST-set-domination heuristic is *deliberately omitted* as unsound for PEG (see "Where
   PGEN stands").
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

### `no_path` is a producer-flaw signal — deletion is the last resort (LRM-proven-absent only)

There is one more inversion of instinct, and it is a **standing rule**: when the linter reports a
rule as `no_path` (no reach path from the chosen entry), the *first* reading is **not** "this rule is
dead, delete it" — it is **"the rule that is supposed to *lead* to it is flawed."** A genuine grammar
rule is stranded far more often by a mis-wired *producer* (a missing reference, an un-eliminated PEG
left-recursion, a mis-encoded delimiter, a wrongly-gated branch) than by being truly useless. So:

> **No rule — `no_path` or otherwise — is ever deleted unless the language LRM *objectively proves*
> it has no business in the grammar. A `no_path` verdict is first read as a producer-wiring flaw to
> FIX (make the rule reachable), never as a removal license. Deletion is the last-last-last resort,
> LRM-proven-absent only.**

Concretely, a `no_path` rule resolves to one of: **(1)** rooted under a *different LRM start symbol*
(e.g. SystemVerilog's `library_text` is a separate start symbol per IEEE 1800-2017 Annex A.1.1 — the
library/config/include subtree is reachable from it, not from `source_text`); **(2)** *profile-relative*
(a later-edition feature, e.g. SV-2023 interface-classes, correctly inert under an earlier profile);
**(3)** an LRM-extraction *decomposition artifact* (a synthetic helper or keyword leaf, not a real
production); or **(4)** a *producer-wiring flaw* — **fix the producer.** Only when none of these hold
*and* the LRM has no production for the rule does the "delete the orphan" path below apply.

A worked re-audit makes this concrete. The SystemVerilog `UNKNOWN` set carried **20 `no_path` rules**.
Cross-checked against the IEEE 1800 LRM, **19 of 20 were legitimate and stay untouched**: 10 are rooted
under the `library_text` start symbol (proven by re-running cert-coverage from the `sv_multi_entry_root`
umbrella entry, which collapses `no_path` from 20 to 9), 6 are genuine 1800-2023 features (proven by
re-running under the `sv_2023` profile, where they witness), and 3 are LRM-decomposition artifacts.
**Exactly one** — `module_path_conditional_expression` (an LRM rule, Annex A.8.3) — was a genuine
producer-wiring flaw: it was stranded by an un-eliminated *conditional* left-recursion in its producer
`module_path_expression` (mpce's condition referenced `module_path_expression`, whose first branch was
mpce). The left-recursion eliminator rewrote the producer into `…_lr_base`/`…_lr_suffix` and left
`module_path_conditional_expression` as an *unreferenced* rewritten seed — its generated parse function
*defined but never called* — so the named rule could never positively witness, and the suffix
reconstruction even leaked raw internal `wrapper_specs` metadata into the ternary module-path AST. The
resolution was to **fix the producer** (the standard non-left-recursive conditional-suffix
transformation — mpce's condition is now the operand chain, same accepted language per A.8.3), never to
delete the rule. That fix has since **landed** (`PGEN-GRAMMAR-WELLFORMED-0111`, release `1.0.143`, ledger
`SV-0005`): `module_path_expression` is now natively non-left-recursive (the LR-elimination synthetic
rules vanish), `module_path_conditional_expression` is positively **witnessed**, the `wrapper_specs` leak
is gone, and the syntax-closure contract's `max_unreachable_rules` budget drops to **0** — so the
linter's headline *"no unreachable rules"* is now literally true for SystemVerilog. SystemVerilog
`UNKNOWN` `86 → 84` (mpce plus the now-eliminated `…_lr_suffix` both leave; witness `1204`, deterministic
at seeds 0/7/42, `spf=0`), `no_path` `20 → 19`, external corpus still `14/14`. **Zero rules were deletion
candidates.**

So when the sections below describe "delete the orphan" as a resolution, read it strictly: it applies
*only* to rules the LRM proves absent — number-infrastructure decomposition artifacts and
engine-shadowed-dead lexical rules with no LRM production and no functional value — never to a genuine
LRM rule that a producer fix can make reachable.

## Worked example: a real bug the linter caught

This is not hypothetical. The always-succeeds shadowing check found a genuine defect in the
SystemVerilog grammar — a cluster of sequence "boolean abbreviation" rules written like this:

```
consecutive_repetition := ( star const_or_range_expression )?   -> {kind: "star_range", range: $1}
                        | ( star )?                              -> {kind: "star"}
                        | ( plus )?                              -> {kind: "plus"}
```

Every alternative is individually wrapped `( … )?`. Because an optional can never fail, the **first**
alternative always succeeds. (The linter originally called the `[*]`/`[+]` forms *unreachable* on the
"PEG commits to the first success" reasoning — a claim the 2026-07-05 Correction below **retracts** for
PGEN's backtracking engine; read this paragraph as the historical framing.) The real, engine-independent
defect stands regardless: the rule silently emits an empty `star_range` node on every sequence expression
even when there is no repetition at all. The signature is too regular to be
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

### ⚠️ Correction (2026-07-05): the always-succeeds check is unsound for a backtracking engine

The fixes above **stand** — they are LRM-grounded (restored delimiters, removed spurious optionals) and
they eliminate real junk-node emission and parse gaps, independent of any reachability argument. But a
later tools-first audit found the *always-succeeds shadowing check itself* to be **unsound** for PGEN's
engine, which **backtracks** rather than committing to the first matching alternative the way a pure PEG
would.

Counter-example, on the shipped parser: for `module m(interconnect p);` the check reports
`net_port_type_sv_2017` alternative #2 (the `interconnect` form) *unreachable* because alternative #0
always succeeds. Yet an AST dump shows the `interconnect` node in the **final** parse tree, and a rule
trace shows the parser **enters all three alternatives** and selects #2 — alternative #0 "always
succeeds" only by matching **empty**, then fails downstream, and the engine **backtracks** to #2. The
flagged branch is **live**.

So "an earlier alternative always succeeds" does *not* imply "later alternatives are unreachable" under
backtracking. The sound question — *can this alternative ever win?* — is a language-difference question,
undecidable in general (exactly the class this chapter excludes). This is the chapter's own principle
turned on itself — a complete-but-unsound check "would 'fix' branches that were never broken", so PGEN
keeps it honest by demoting it.

**As implemented (A2.2, 2026-07-05).** The shadowing *verdict* was removed outright: the linter no
longer produces an "earlier-always-succeeds ⇒ later-alternative-unreachable" shadowing finding, and the
matching unreachability *certificate* variant is gone (a certificate is a *proof* of deadness, which
always-succeeds cannot honestly supply). There was no sound sub-case worth keeping — every
always-succeeds form, empty-match or not, is defeated by the same backtracking argument, and the only
truly-redundant case (an exact *duplicate* alternative) is already its own sound reason. The underlying
observation survives as a **non-verdict note**: `--lint-grammar` now reports
`always_succeeds_alternatives=N (note)` and prints each as a `[note]` worded to make *no* reachability
claim — kept because a nullable earlier alternative is still a useful smell (it was the tool that found
the dropped-delimiter bugs above). Concretely, SystemVerilog's former **8** always-matches *warnings*
became **6** informational *notes* (the count reframes from "one per shadowed victim" to "one per
always-succeeding source"), `ordered_choice_shadowing` stays a hard gate at `0`, and the proven-live
`interconnect` port branch is no longer flagged at all.

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

This is why every *gating* check PGEN ships is a *sound decidable subset* and why the unsound heuristics
(general FIRST-set domination, parse-order predicate reachability, and — as of the 2026-07-05 audit —
earlier-always-succeeds shadowing under backtracking) are excluded from the gates: they would buy
completeness at the cost of soundness — the wrong direction. (Earlier-always-succeeds remains available
as a non-gating anti-pattern *hint*, not a verdict.)

### The mechanism: a certifying algorithm

Soundness on its own is a promise. To make it something you *verify* rather than *trust*, the linter
is built as a **certifying algorithm** (Mehlhorn, McConnell et al.): every verdict ships with a
checkable **certificate**, and a second, deliberately tiny program validates the certificate. You
then trust the small checker — which you can read in an afternoon — instead of the linter's complex
internals.

| Verdict | Certificate | How you check it |
|---|---|---|
| **reachable** | a **witness** — a concrete derivation *and* an input string that exercises the fragment | replay the input through the real parser; watch it hit the fragment |
| **unreachable** *(or positively-unreachable)* | a **proof** — the exact decidable argument (which sound rule fired, and the chain) | a small checker re-validates the argument |
| **`UNKNOWN`** | *(none — by design)* | it is an honest "I cannot prove this either way," never a guess |

The witness producer for the "reachable" case is the **stimuli generator** — this is the duality made
operational: the linter claims reachability, the generator *demonstrates* it. And the binding
discipline is: **no definite verdict without a certificate.** It is precisely because the linter
refuses to speak without a proof that you never have to doubt it when it does.

The **proof** verdict covers two *sound, decidable* whole-rule forms, both re-checked by a tiny
independent validator. The first is **structural unreachability** — a rule not reachable from any
root at all (a dead "useless symbol"). The second is **positive-unreachability** — a rule that *is*
structurally reachable but only ever through a `!`/`&` **lookahead** edge, never positively. This
second form matters because the witness side records *positive rule entry* (see the next section), and
a lookahead is a parser assertion that consumes no input and emits nothing — so a rule reached only
inside a lookahead can *never* be positively entered, and *can never be witnessed by construction*.
That non-witnessing is therefore **correct, not a coverage gap**, and it earns a `proof` (re-derived as
"the rule is reachable but not *positively* reachable") rather than languishing in `UNKNOWN`. The
canonical case is a guard token used only in a negative lookahead — the synthesizable-RTL grammar's
`port_direction_token`, referenced solely inside `( comma !port_direction_token port_item )*` to stop
the inner port-iteration at a direction keyword. It does real parse work, so it is *not* dead (removing
it would be wrong); it is simply un-witnessable, and the proof says so soundly.

### "Watch it hit the fragment" — how a witness is checked, rigorously

The witness check reads "replay the input through the real parser; watch it hit the fragment." That
last clause hides a subtlety worth being precise about, because the obvious way to do it is **wrong**.

The naive approach is to parse the input and then *walk the output AST*, collecting the rule name of
every node. On a grammar with **return annotations** this silently fails. A rule written
`r := … -> { … }` does not return a structural subtree — its annotation *folds the whole subtree into
a single typed value* (`ParseContent::Shaped`), which has no child nodes. So an AST walk stops at the
first annotated rule and never sees anything beneath it. On a heavily-annotated grammar like
SystemVerilog the walk collapses to **one** rule for an entire file — it would report almost
everything as un-witnessed, even though the parse genuinely exercised hundreds of rules. (The other
naive option — per-rule *call counters* — fails the opposite way: a PEG tries and backtracks
alternatives, so a counter **over-counts**, crediting rules whose match was thrown away.)

The correct source is to make the **parser itself testify** to what it parsed. PGEN's generated
parsers carry an opt-in, **transactional coverage record**: each rule, on entry, pushes its id onto a
coverage stack; the universal speculation wrapper (`try_parse`, which backs every `|`, `?`, `*`, `+`,
and lookahead) snapshots the stack's length and, on backtrack, truncates back to it — exactly as it
already rolls back the input position and the semantic state. One more piece is required for that
guarantee to actually hold: **packrat memoization composes with this record exactly the way it
composes with the semantic store.** A memo *hit* reuses a cached parse result without re-entering the
rule body — so the entry-push never fires for the cached subtree. A subtree first parsed inside a
speculation that later fails (its coverage entries truncated by the rollback) and then memo-hit on the
committed path would silently vanish from the record, even though the accepted parse genuinely
exercised it. The memo entry therefore stores the **coverage delta** its body pushed, and every cache
hit replays that delta onto the live stack — mirroring how the cached *semantic* delta is replayed on
hits. (This was found live, not hypothetically: on the regex grammar, `\Q\A\E*` parses its `\A`
through an inner speculation that fails a lookahead and is then memo-hit by the committed alternative
— pre-fix, the witness record permanently lacked the escape-tail rules, which sat in the
cert-coverage `UNKNOWN` bucket for exactly that reason.) The replay happens *inside* the current
speculation, so a later rollback still truncates it — transactionality is preserved. After a
*successful* parse, every failed attempt necessarily occurred inside some rolled-back speculation, so
the entries that survive are **exactly the rules of the accepted parse**:

- **sound** — no backtracked, thrown-away attempts (the call-counter failure mode), and
- **complete** — annotation folding cannot hide a rule, because we record the *entry*, not the
  output node (the AST-walk failure mode).

This is "verified, not trusted" applied to the witness side: the certificate is checked by the real
parser reporting its own behaviour, independent of whatever the generator *believed* it had covered.
The record is off by default (ordinary parsing pays nothing) and the mechanism is parser-agnostic —
every grammar PGEN compiles gets the same instrumentation for free.

One corollary: a witness only counts if it actually *parses*. A sample the generator believes covers a
rule, but which the real parser rejects, contributes nothing — and is reported as a labelled
sample-parse failure rather than silently dropped. This is also how generator defects surface: the
witness side of the gate doubles as a parseability check on the generator's own output (for example, it
caught the generator fusing adjacent keywords — `endprogram`+`module` → `endprogrammodule` — when
mandatory word-boundary spacing was off).

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

**First worked example — `json` is fully certified.** The `json` grammar is the first PGEN grammar to
reach `UNKNOWN = 0` end-to-end through the gate. Running
`ast_pipeline grammars/json.ebnf --report-certificate-coverage --entry-rule json` reports
`total=9 proof=0 witness=9 UNKNOWN=0 fully_certified=true (sample_parse_failures=0)`, identically across
seeds and sample counts — every one of its nine rules carries a checked reachability *witness*, none is
left unknown, and every witness re-parses. This is the internal **well-formedness / coverage** property
(the linter's claims about *this* grammar are all verified), not a statement about external conformance:
`grammars/json.ebnf` is a deliberately *simplified* JSON grammar, so "fully certified" here means the
grammar is provably well-formed and every fragment is constructively reachable — separate from how
faithfully it matches the full JSON standard, which is measured separately against external corpora.

**Rolling the gate out per grammar (Phase H).** The certificate-coverage gate is parser-agnostic, so it is
being wired to run for every PGEN grammar in turn (each grammar needs a small `parse_and_cover_<grammar>`
adapter so the witness side can replay samples through that grammar's real parser). It now runs for
`json` (fully certified), `regex` (now fully certified — see the drive arc below),
`rtl_const_expr` (fully certified), `systemverilog_preprocessor` (fully certified), `vhdl` (now fully
certified — see the drive arc below), `rtl_frontend`
(the ~5.5 MB synthesizable-RTL frontend parser, **now fully certified** — its initial wiring reported
`total=170 witness=37 UNKNOWN=133`, driven across the `.5.*` arc to `total=169 proof=1 witness=168
UNKNOWN=0` (deterministic at seeds 0/7/42, zero sample-parse failures): the last two fragments closed by
*adjudication, not reclassification* — `port_direction_token` (lookahead-only) earned a `proof` and the
engine-shadowed-dead `white_space` was removed at the source, exactly as `vhdl`'s was), `systemverilog`, and
`vhdl` (the last shipped grammar to be wired — it runs at the default depth on its flat `design_unit*`
entry and reports `total=217 witness=148 UNKNOWN=69 (sample_parse_failures=0)` at seed 0, every witness
re-parsing cleanly). With `vhdl` wired, **every shipped parser grammar now runs under the
certificate-coverage gate**; only the internal meta/annotation grammars (`ebnf`,
`return_annotation`, `semantic_annotation`) remain unwired. The other grammars carry an *honest, openly
reported* `UNKNOWN` backlog still being driven toward zero — a *loud, specific* list of fragments still
awaiting a witness, never hidden, because the gate never pretends a grammar is fully certified until every
fragment carries a checked certificate. The per-grammar drives keep shrinking it: `regex` has now been
driven all the way — `98 → 19 → 7 → 5 → 3 → 2 → 0`, making **`regex` the fourth fully-certified grammar**
(after `json`, `rtl_const_expr`, and `systemverilog_preprocessor`), deterministic across seeds with zero
sample-parse failures. The arc's slices, in order: the `19→7` slice removed twelve rules proven dead by two
independent oracles; the `7→5` slice was a declarative grammar fix — all-branch `-> $text` on two
single-char alternation rules so the generator renders them fused to their prefix, `\pC`/`aD`, exactly the
lexical-annotations atomic-token rule; the `5→3` slice was the memo-hit coverage-delta replay described
above — the two rules were *already witnessed by the accepted parses*, the record just failed to say so on
memo hits; the `3→2` slice was a declarative `@sample` witnessing literal on a rule the generator
structurally could not materialize — its body references a parser-side builtin primitive with no grammar
definition; and the final `2→0` slice was the **semantic-prelude reach** described in the next section —
the last two rules were gated by a semantic-store predicate no minimal derivation could satisfy. The
memo-replay engine fix also moved every other backlog in one step with zero generation change: vhdl
`31→30`, rtl_frontend `75→73`, SystemVerilog `738→647`. For exact current per-grammar numbers, the gate's
own report is the authority.

The `vhdl` drive (`69 → 31 → 30 → 4 → 1`) then delivered the doctrine's sharpest payoff so far. After a
generator-side lexical-faithfulness fix closed the keyword-fusion family (`30 → 4`), the residual
(`based_literal`/`based_value`/`hash`/`white_space`) refused to witness for a reason no generator
improvement could cure: **the released vhdl parser was rejecting every valid based literal** (`2#1010#`).
The generated parsers' internal layout skipper hard-coded `#`-to-end-of-line comment skipping — an EBNF
meta-grammar convention, while in VHDL `#` is the based-literal delimiter — so it ate the very `#` token
the `hash := trivia /#/` rule was about to match, and `based_literal` failed on every input since the
family's first release. The engine fix gives the skipper's comment arms the same "don't swallow the token
you are matching" guard its string-terminal side always had; the three literal rules witness (`4 → 1`,
identical at seeds 0/7/42), and the same step moved `rtl_frontend` `73 → 71` and SystemVerilog
`647 → 645` (explicit comment-token rules becoming directly witnessable). This is the attribution rule
and the bug-finding-oracle role working exactly as designed: an unwitnessable fragment was *neither*
accepted as a residual *nor* chased with generator machinery — it was adjudicated, and the blame landed
on a real, shipped parser defect (ledger row `VHDL-0002`, vhdl release `1.0.4`).

The drive's final two steps (`1 → 0`) closed the same way — by adjudication, never by reclassification.
The last `UNKNOWN`, `white_space`, was proven **engine-shadowed-dead**: the generated parsers' layout
skipper consumes whitespace before any non-empty-matchable regex executes, so a pure-whitespace-class
rule can never match its own bytes on any input — reach probes parse but never witness it. The
accept-identical removal of that dead branch was deliberately **parked** when a 16-seed sweep showed it
would double the exposure of a then-open over-generation class: organic samples were failing to re-parse
at 14 of 16 seeds. Per-sample bisection proved all fourteen failures were ONE generator engine bug — the
boundary tracker's word-shape state was not transactional across discarded render attempts (see the
lexical-annotations chapter), fusing `<identifier>`+`is` thirteen times across the sweep. With that
fixed at the root (sweep `14 → 0`), the parked removal landed: **`vhdl` is the fifth fully-certified
grammar** (`total=216 witness=216 UNKNOWN=0 fully_certified=true`, zero sample-parse failures,
deterministic at seeds 0/7/42, with the 16-seed sweep staying all-zero and the external corpus still
parsing 8/8).

The skipper-comment-arm defect had a second instance — and a sharper ending. The lone SystemVerilog
cert-coverage sample-parse failure left behind by the `VHDL-0002` fix had been adjudicated as generator
over-generation ("the parser is right to reject those bytes"). A tools-first re-investigation refuted
that: bisecting the failing sample down to
`interface i #  (  ) ;timeunit 09 ns//>Mg` ⏎ `//QF.` ⏎ `/633 s;endinterface` and reading the full trace
showed the parser rejecting **grammar-valid SV** — the *dual case* the `VHDL-0002` guard cannot cover.
That guard stands a comment arm down when the *active* token's own pattern matches at the introducer;
here the parser was speculatively attempting `line_comment` (`//…`) when the skipper's `#` arm fired at
the `#` of a real ANSI `#( )` header, swallowed the rest of the line as a "comment", matched the second
comment, and **memoized** the bogus trivia span — so the true header path later died on the poisoned
memo. The fix closes the class at the root: comment arms are now suppressed **statically at parser-emit
time**, per introducer, whenever the grammar assigns that introducer a *non-comment meaning* (a terminal
whose every match mandatorily starts with it and that is not itself comment-defining — a regex-HIR
analysis over the grammar's token inventory). SV defines `#` as a real token, so its `#` arm is simply
never emitted; SV's `line_comment`/`block_comment` tokens *agree* with the `//`/`/*` arms, so those stay.
The criterion itself was sharpened by a decisive stash A/B: a first cut that suppressed arms on mere
prefix *overlap* silently regressed the generated **ebnf** parser (whose `("#" | "//") comment_content`
tokens are comment-*defining*, and whose real grammar files carry mid-rule comments the meta-grammar does
not yet structurally own) — the refined non-comment-meaning criterion leaves every such surface
byte-identical. The released SV parser bug is ledger row `SV-0001` (SV release `1.0.139`); SV
cert-coverage now reports `sample_parse_failures=0` at the canonical seeds with the witness landscape
unchanged, and vhdl/rtl_frontend lose the same theft window with every measured surface identical. This
is the attribution rule compounding: the same oracle that exposed `VHDL-0002` exposed its dual, and the
"never silently accept a residual" discipline turned a shrugged-off generator ticket into a real,
shipped-parser fix.

SystemVerilog's own `UNKNOWN`→0 drive then opened with the largest *structural* reduction available — a
textbook application of the attribution rule's second branch (the grammar is at fault, so fix the
grammar). The certificate-coverage report flagged a whole family of number-literal fragments —
`binary_number`, `octal_number`, `decimal_number`, `hex_number`, `size`, `fixed_point_number`, and their
decomposed `*_base`/`*_digit`/`*_value` helpers plus the private single-character `kw_*` tokens those
helpers consumed — as having *no reach path at all* from the real entry. Reading the grammar explained
why: an earlier cleanup (`PGEN-SV-EXH-PROOF-0021`) had consolidated `integral_number` / `real_number` /
`unsigned_number` into single clean regexes, which **severed the reference chain** that used to reach the
decomposed sub-tree, leaving the whole subgraph as orphans that did no parse work. An independent
reference-graph dead-closure from the three real entries reproduced *exactly* the 48 rules the
syntax-closure contract had already *blessed* as intentional orphans — and that contract's own drift
policy spells out the right resolution: **delete the orphan**. Removing all 48 at the source dropped
SystemVerilog's `UNKNOWN` backlog by 48 with the witness count **byte-identical** (the rules were never
reached, so no accepted parse changed), the external corpus still parsing 14/14, and no release or schema
bump — the Hopcroft–Ullman *reduced-grammar* requirement turning into an honest block of dead code
removed, exactly as `regex` (`19→7`), `systemverilog_preprocessor` (`trivia`), and `vhdl` (`white_space`)
had each done.

The next SystemVerilog step was a **reach-honesty** fix that paid off out of all proportion to its
size — the largest single `UNKNOWN` drop the gate has seen. The certificate-coverage report's reach
pass kept reporting a huge `parsed-but-routed-elsewhere` residual: hundreds of reachable rules whose
reach probe *parsed* but routed through other rules. Capturing the probes showed the dominant shape
was a fallback to a **canonical empty shell** — `module m; endmodule` or `program p; endprogram` —
for 112 of the 130 unwitnessed keyword-token rules. The cause was a collision between two mechanisms
that had never been made to compose: a rule's `@sample` *literal-override* hint (which lets the
generator emit a rule as one canonical literal instead of expanding its body) and the reach plan's
*forced descent* (which steers generation through a rule's body to witness a deeper target). When the
reach plan forced `module_declaration`'s branch — whose `@sample` is `"module m; endmodule"` — the
literal override fired *first* and emitted the shell, so the body where every gate-instantiation,
specify-block, statement and assertion keyword lives was never generated. The fix makes the two
compose: a branch-level `@sample` override stands down for exactly the branch the active reach plan is
forcing (and only then — off-reach generation is byte-identical, and a grammar with no `@sample` on a
forced branch is unaffected). That one change descended into the module/program bodies and witnessed
not just the 130 keywords but every construct and intermediate rule along those paths —
SystemVerilog `UNKNOWN` `567 → 123` in a single step (witness `726 → 1170`), deterministic with the
diverse certification pass and every other grammar byte-identical (the fully-certified roster never
runs the reach pass, and `rtl_const_expr`/`rtl_frontend`/`vhdl`/`json` carry no `@sample` at all).
SystemVerilog remains the one shipped grammar not yet fully certified, its remaining backlog (the
profile/alternate-entry `no_path` rules and a smaller constraint/sequence keyword cluster) openly
reported and still being driven toward zero.

The next SystemVerilog step drained two more with the **engine-shadowed-dead** removal first proven on
`vhdl`. `white_space` (`/[ \t\r\n]+/`) and `comment_only_source_region` were both *reachable* — the
former via `trivia`, the latter via a `source_text_item` branch — yet could never witness, because the
generated layout skipper consumes whitespace *and* comments as leading trivia before any non-empty
regex or `source_text_item` branch is tried. The reach probe confirmed it directly (both `parsed=true` /
`witnessed=false`; a comment-only file parses to an empty `source_text` with no
`comment_only_source_region` node in *any* AST), so by the attribution rule they are genuinely dead and
the resolution is the literal-0 **delete the orphan** path — exactly `vhdl`'s `white_space` again, one
level up for `comment_only_source_region`. Removing both, with their dead references (the `trivia`
whitespace arm and the `source_text_item` branch), dropped SystemVerilog `UNKNOWN` `123 → 121` with the
witness count **byte-identical** (`1170`, no collateral), `spf=0` at seeds 0/7/42, the external corpus
still 14/14, and no release or schema bump — neither rule ever appeared in an accepted parse, so the
wire shape is unchanged and the declared `source_text_item` union simply narrows to the seven kinds that
can actually occur.

The next SystemVerilog step was the **rule-level twin** of the reach-honesty `@sample` stand-down that
delivered `567 → 123`. After that branch-level fix descended into the module/program *bodies*, a cluster
of expression-context rules — `cast`, `concatenation`, `conditional_expression`, `cond_pattern`,
`assignment_pattern_entry`, the `inside_expression` family, and the intermediate rules beneath them —
*still* would not witness. The reach pass routes a deep expression target through the module's **ANSI
port list** (`module_ansi_header → list_of_port_declarations → ansi_port_declaration`, whose named-port
form `.p(expression)` is the shallowest place an `expression` is reachable), and the generated witness was
always the fixed shell `module m(input logic a);endmodule`. That string is the tell: it is exactly the
*rule-level* `@sample` literal on `module_ansi_header`. The earlier fix taught a **branch-level** `@sample`
override (the one attached to a specific ordered-choice arm) to stand down for the arm a reach plan is
forcing — but `module_ansi_header`'s hint is attached to the **whole rule**, and a rule-level override
fires at rule *entry*, before any of the body (and its forced descent into the port list) is generated. So
the plan steered correctly all the way to `module_ansi_header` and was then short-circuited one level
above the port. The fix is the rule-level analogue: a rule-level `@sample`/`@probe_sample` override now
stands down whenever the active reach plan forces *any* decision **inside that rule's body** (an
ordered-choice branch or a quantifier the path crosses) — i.e. when the plan must descend *through* the
rule to reach its target. Off-reach generation, and any rule the plan does not steer into, keep the
literal exactly as before (byte-identical), and grammars with no such hint — the entire fully-certified
roster — are untouched. The criterion is keyed purely on the plan's own forced-decision set, never a rule
name, so it is parser-agnostic. With it, the whole M1a expression cluster descends through a real port and
witnesses: SystemVerilog `UNKNOWN` `121 → 93` in one step (witness `1170 → 1198`), deterministic across
seeds 0/7/42 with zero sample-parse failures, every other grammar byte-identical. The residual is the
honest remainder — the adjudicated alternate-entry/profile `no_path` rules, a second expression cluster
that reaches its context but routes through a *sibling* rule, the constraint/sequence over-generation and
store-gated identifier classes, and the property/sequence temporal operators — each openly reported and
still being driven toward zero.

The next SystemVerilog step closed part of that *"reaches its context but routes through a sibling"*
cluster. Those rules (`bit_select_expression`, `array_range_expression`, `context_member_method_call`,
…) reach the right parse context, but the witness pass steered the reach path only to the target rule
`R`'s **reference site** — never `R`'s *own* internal structure — so under minimal generation `R`
rendered its shallowest form (root `Or`→first alternative, `?`/`*`→zero), which is *sibling-ambiguous*:
a degenerate first alternative that is a bare pass-through an earlier sibling also accepts, or a
distinguishing token sitting behind a not-taken optional. The PEG then re-attributes `R`'s bytes to that
earlier sibling, and `R` never witnesses. The fix is a **target-own-structure reach pass**: for each
still-`UNKNOWN` rule it *also* forces `R`'s own top-level ordered choice to a non-degenerate alternative
(trying the distinguishing arms before the degenerate one) and every optional inside `R`'s body to expand
once — on top of the base reach plan — so `R` renders a *distinguishing* form the parser attributes to
`R` itself. Two properties keep it honest and grammar-neutral. It is keyed purely on the rule's `ASTNode`
structure (`(rule, node-path)`), never a rule or grammar name. And it runs as a final pass over **only**
the rules still `UNKNOWN` after the diverse / recursive-reach / plannable passes — so a grammar those
passes already fully certify has an empty residual and this pass never generates a probe at all (it is
*truly inert* for the certified roster, not merely cheap; this matters because the pass is parser-agnostic
and would otherwise add cost to a deep-recursive grammar like `rtl_const_expr` that gains nothing from
it). It moved SystemVerilog `UNKNOWN` `93 → 90` (witness `1198 → 1201`), deterministic across seeds 0/7/42
with every other grammar byte-identical. The residual cluster it could *not* close is the honest next
ticket: a handful of carriers whose distinguishing form is still absorbed by an earlier sibling at the
**parent** ordered choice, regardless of `R`'s own shape — a *parent-commit* problem the reach plan must
solve one level up.

The next SystemVerilog step extended the target-own-structure pass *one level deeper* — forcing not just a
rule `R`'s own ordered choice and optionals, but the **distinguishing structure of a mandatory child
rule** `R` references. Some carriers render a non-degenerate top-level form yet still fail to witness
because the bytes that distinguish them live in a *separate* mandatory sub-rule the original walker never
descended into (the generation-input AST stores a rule reference as a leaf token, so the child's body is
not inlined). A bounded mandatory-reference walker (sequence elements, min-1 `+`-group elements, `Atom::Node`
groups, and `rule_reference` tokens — skipping min-0 quantifiers, un-forced `Or` branches, and lookaheads)
now installs the child's own forcing on top of `R`'s plan, keyed purely on `(child, node-path)`, and runs
only over the rules still `UNKNOWN` after the earlier passes (so it is inert for the certified roster). It
moved SystemVerilog `UNKNOWN` `90 → 89` (witness `1201 → 1202`, deterministic at seeds 0/7/42) — closing
`sequence_method_call`, whose distinguishing call form needed its mandatory child forced. The residual it
could *not* close sharpened the next ticket: a few carriers whose forced distinguishing form is *still*
re-attributed to a sibling at the parent ordered choice — the parent-commit-hard class named above.

A later step added the pass's **third** tier, and it is worth stating what makes it a *different* lever
rather than more of the same. The first tier forces the target rule's own body; the second forces the
target's mandatory children. Both look *downward* from the target. But a target reached through a
**min-0 quantified** reference — the shape left-recursion elimination synthesizes for every recursive
rule, `X := X_lr_base ( X_lr_suffix )*` — has a third dependency the plan never decides: the mandatory
**sibling** rendered immediately before it, its *seed*. A min-0 continuation commits only if the seed's
derivation stops short of the continuation's own leading token; when the seed's choice carries a
catch-all alternative that spans that token, longest-match hands the seed the whole operand, the
quantifier matches zero times, and the target is **entered but never committed**. The probe is
structurally correct and still reports `parsed=true witnessed_target=false`, which is exactly the
reading that makes this class so easy to misdiagnose as a routing failure. It is measurable in one line
with `--dump-rule-outcome-counts-json`: on an isolating grammar whose recursive rule has both a
discriminating arm and a catch-all, the suffix rule is `entries=1 committed=0` on the catch-all seed and
`entries=3 committed=1` on the discriminating one. The **seed-diversification** tier enumerates the
alternatives of every mandatory rule-reference sibling preceding the quantified site at the reach path's
final hop, keyed — like the other two tiers — purely on `ASTNode` structure and the hop chain the plan
actually installed, never on a rule name and never on the eliminator's `_lr_` spelling. It gates on
min-0 deliberately: a min-1 continuation must be committed or the parse rejects, so it has no
zero-iteration escape and therefore no shadowing hazard. On the isolating grammar it moves
`UNKNOWN 1 → 0` (`fully_certified=true`) at every combination of `--count 1/2/3` × `--seed 0/7/42`, with
every other grammar's certificate line byte-identical and the generated parsers byte-identical — this is
a stimuli-generation change and cannot reach a shipped parser. Its honest bound is stated rather than
implied: only the final hop is scanned, so a seed shadowing an *intermediate* hop is outside the axis.

The next SystemVerilog step was the second branch of the attribution rule again — **the grammar is at
fault, so fix the grammar** — and it is a textbook instance of the *delimiter-drop* class the
SystemVerilog grammar's LRM extraction keeps producing. The streaming-concatenation family
(`{ >> {a, b} }`, `{ << 4 {a with [3:0]} }`) had two delimiters dropped: `stream_concatenation` lost the
literal `{ }` that wrap its stream-expression list, and `stream_expression` lost the literal `[ ]` around
its `with`-clause range — so `array_range_expression` (which lives only inside that bracketed range) was
**unreachable**, and the cert-coverage gate could never witness it (a reach probe entered it `0` times
versus `110` for the witnessing two-expression form). Restoring the delimiters per IEEE 1800 §A.8.1
(`stream_concatenation := lbrace stream_expression ( comma stream_expression )* rbrace`,
`stream_expression := expression ( kw_with lbrack array_range_expression rbrack )?`) makes the fragment
reachable — and, exactly as the delimiter-drop class so often does, *also closes a real parse gap*: the
LRM with-bracket form `{<< 4 {a with [3:0]}}` was rejected on every prior release (released-parser bug
`SV-0002`, SV release `1.0.140`). Restoring the mandatory inner brace then surfaced a second, subtler
defect — a PEG greediness in `streaming_concatenation`'s optional `( slice_size )?`, whose
`constant_expression` branch would consume the very brace-concatenation that *is* the mandatory
stream-concatenation and then fail without backtracking (which would have regressed the canonical
`{>> {data}}` forms real UVM uses). The fix is the idiomatic PEG guard: `( slice_size &lbrace )?`, encoding
the LRM structural fact that a slice-size is always followed by the stream-concatenation's `{`. This is the
duality and the bug-finding-oracle role compounding once more: a single unwitnessable fragment was neither
accepted as a residual nor chased in the generator — it was adjudicated to the grammar, the fix restored
LRM fidelity, and the restoration exposed (and the tools then caught, before it shipped) a downstream
greediness regression. It moved SystemVerilog `UNKNOWN` `89 → 88` (witness `1202 → 1203`, deterministic at
seeds 0/7/42, `spf=0`, every other grammar byte-identical), with the SV external corpus still parsing
14/14. Because the restructure also changed `stream_concatenation`'s `body` from a raw quantified node to a
clean array, it is the family's first shape-changing release in a while (AST-dump schema `3 → 4`).

The SystemVerilog drive's next payoff was a *cert-neutral* one — and it shows the bug-finding-oracle
role does not depend on the `UNKNOWN` number moving at all. Two adjudication slices on the residual
carrier `context_member_method_call` (the store-gated `head.member[idx].method()` rule) first **refuted a
mis-diagnosis**: an earlier note had called the rule a "real parser bug — `a.b[0].c()` is rejected", but a
tools-first re-check showed the rule's `@predicate has_fact(variable_binding, $head)` is live and the form
*parses and witnesses* with a declared head (`int a; … a.b[0].c()`) — the rule's cert `UNKNOWN` is a
store-gated **witness-reach gap** (the generator can't yet synthesise the name-coupled binding prelude the
gate needs), not a parser defect. But *chasing why a class-handle head would not witness* then surfaced a
genuine, separate released-parser bug: a module-scope class-handle declaration `C a;` (with `C` a declared
class) was mis-parsed as a **`net_declaration`**, because the `net_declaration` user-nettype branch's
`checked_nettype_identifier` was gated by the under-specified `has_fact(type_name, $body)` — and a class is
*also* a `type_name`, so a class name satisfied the nettype gate. The mis-route emitted no `variable_binding`
fact, so class-handle member-method chains were rejected. This is the store-consultation discipline applied
to a gate that *was* consulting the store but with too weak a predicate: the fix tightens it to
`fact_attribute_equals(type_name, $body, declaration_family, nettype)` (the proven
`known_unscoped_block_class_type` pattern), so a class no longer passes for a nettype and `C a;` routes to
`data_declaration` (the LRM-correct categorisation — a class is not a net type). It moved SystemVerilog's
cert `UNKNOWN` not at all (`88`, byte-identical at seeds 0/7/42, witness set unchanged) — the fix is a
correctness re-route, and closing `context_member_method_call`'s witness is the separate generator pass — but
it fixed a real bug shipped in every prior release (ledger `SV-0003`, SV release `1.0.141`), with the SV
external corpus still `14/14` and a new shape-contract lock pinning that `C a;` binds as a variable. The
attribution rule compounding once more: the same drive that drained the structural residual also turns a
*non-witness* into a real, shipped-parser fix — even when the coverage number itself does not move.

The next SystemVerilog step is the attribution rule's *third* outcome — neither a generator-reach fix nor
a grammar parse-gap repair, but the clean removal of a genuinely **dead branch at its source**.
`bit_select_expression` (the `[ … ]` select content) carried four alternatives, the first being
`direct_index_method_call` — a `head.method()` form inside a bit-select. But that branch never wins: its
sibling `method_call` (whose `( dot method_call_body )*` chains at least as far) subsumes every input it
could match, and a tools-first sweep confirmed it — the rule sat in the never-witnessed `UNKNOWN` set,
produced **zero** committed AST nodes on six inputs shaped exactly for it (`b[c.d()]`, `b[this.d()]`,
`b[pkg::c.d()]`, …, all routing to `method`), and no test fixture or corpus parse ever committed it. By the
attribution rule, a rule that is neither linter-provably-unreachable nor generator-witnessable *and* is
structurally subsumed is a **dead branch the grammar should not carry**, so the fix is to delete it at the
source (along with its now-orphan rule). The decisive proof is the A/B: regenerating the parser without it
left the witness set **byte-identical** (`witness 1203` unchanged, `spf=0`, the SV external corpus still
parsing `14/14`, deterministic at seeds 0/7/42) while the rule itself left the set — `UNKNOWN 88 → 87`.
Because no accepted input's parse or AST changes, the wire behaviour is identical and the release/schema do
not move: a pure, verified subtraction of dead weight.

The next SystemVerilog step was the attribution rule's *second* branch once more — **the grammar is at fault,
fix the grammar** — and a textbook instance of the *delimiter-drop* class the SV LRM extraction keeps
producing (the same family as the `SV-0002` streaming fix). The certificate-coverage report flagged
`goto_repetition` as a reachable-but-unwitnessed `UNKNOWN`. Its rule was `goto_repetition := ( implies
const_or_range_expression )` — a bare `-> const_or_range_expression`; but `->` is *also* the implication
operator, so the generator's bare `-> N` was always re-attributed to an expression and `goto_repetition`
never positively witnessed. Reading the grammar showed the defect was *family-wide*: the entire
`boolean_abbrev` sequence-repetition family had dropped the literal `[ ]` brackets the LRM mandates —
`consecutive_repetition` (`[* N]` / `[*]` / `[+]`), `goto_repetition` (`[-> N]`), and
`non_consecutive_repetition` (`[= N]`, both profiles) — and the call site `expression_or_dist ( boolean_abbrev
)?` added none. Tools-first confirmed the dual symptom this delimiter-drop always carries: every LRM-valid
bracketed form (`a[*3]`, `a[*]`, `a[+]`, `a[->2]`, `a[=2]`) was **rejected** — the grammar refused valid
IEEE 1800 §A.8.1 SystemVerilog — while the bracket-less spellings `a*3` / `a=2` were wrongly *accepted*. The
LRM ground truth (the extracted `goto_repetition ::= [-> const_or_range_expression ]`,
`non_consecutive_repetition ::= [= const_or_range_expression ]`, and §A.8.1's bracketed
`consecutive_repetition`) is unambiguous, so the fix restores the `[ ]` across the family using the proven
in-grammar bracket idiom (`lbrack op const_or_range_expression rbrack`, the `range` captured at `$3`). The
emitted `{kind, range}` / `{range}` shape is byte-identical — the brackets are parsed then folded away by the
return annotation — so the release does **not** change the AST-dump schema. Post-fix the bracketed forms parse,
the bare spellings are correctly rejected (a bare `a*3` still parses as a *multiplication expression*, so
nothing valid regresses), and `goto_repetition` becomes distinguishable and witnesses: SystemVerilog `UNKNOWN
88 → 87 → 86` (witness `1203 → 1204`, deterministic at seeds 0/7/42, `spf=0`, zero newly-unknown), with the SV
external corpus still parsing `14/14` and the shape-contract green. This is the delimiter-drop class and the
bug-finding-oracle role compounding yet again: an unwitnessable fragment was neither accepted as a residual
nor chased in the generator — it was adjudicated grammar-first, the fix restored LRM fidelity, and the
restoration closed a real, shipped parse bug (ledger row `SV-0004`, SV release `1.0.142`).

### Reaching deep recursive branches: the constructive-reach witness pass

`rtl_const_expr` was the first grammar to expose a structural gap in the witness side, and the way it was
closed is worth stating because it is a *general* capability now, not a one-off. Its operator-precedence
chain is ~14 rules deep (`conditional_expr → … → primary_expr`), and one of `primary_expr`'s alternatives is
the *parenthesised primary* `( conditional_expr )`, which **re-enters the whole chain**. So a single `( … )`
needs roughly twice the chain's depth. Ordinary clean generation spends its entire depth budget just
*reaching* `primary_expr`, leaving none for the nested expression — so the parenthesised branch is reached
only on the deepest descents, where it cannot complete, and `lparen`/`rparen` are never witnessed
(`total=48 witness=45 UNKNOWN=3`). The linter confirms the branch is genuinely reachable (`( 1 )` is valid),
so by the attribution rule this is a *generator-reach deficiency*, not a dead branch — the fix belongs in the
generator. Simply raising `--max-depth` is **not** a fix: with more depth the repetition (`*`) and ternary
(`?:`) fan-out explodes super-linearly (eight samples time out), so a bigger budget makes generation
intractable long before it makes the branch reachable.

The certificate-coverage report instead runs a **second, auxiliary witness pass** when (and only when) the
ordinary diverse pass leaves `UNKNOWN` rules. This *constructive-reach* pass tries each still-unwitnessed
branch that re-enters an active construct at its **shallowest** reach and builds its **minimal** derivation
with a fresh budget (minimum repetitions, shortest alternatives) — yielding a small, valid `( 1 )` that
re-parses and witnesses `lparen`/`rparen`. With it, `rtl_const_expr` reaches
`total=48 witness=48 UNKNOWN=0 fully_certified=true`, deterministically across seeds, with zero
sample-parse failures.

That canonical baseline — the **default** entry rule, `--max-depth 32`, `--count 40`, the default
diverse generation step-budget, seeds 0/7/42 — is now regression-locked by a standing oracle:
`make -C rust SHELL=/bin/bash rtl_const_expr_cert_gate` re-runs the exact configuration for every
declared seed and asserts each headline field (total/proof/witness/`UNKNOWN`/`fully_certified`,
`sample_parse_failures=0`, `proof_reverify_failures=0`, plus cross-seed determinism) against the
tracked contract `rust/test_data/grammar_quality/rtl_const_expr_cert_contract.json`. So
`rtl_const_expr`'s membership in the fully-certified roster is machine-checked rather than
doc-asserted — the gate exists precisely because a diagnostic sub-entry probe configuration
(`--entry-rule conditional_expr`) was once mislabeled as the canonical lane, and only a pinned
oracle makes that class of wording confusion fail mechanically.

Two properties keep this honest and safe:

- **The diverse certification pass is untouched.** The reach pass is *separate* and *additive*: it only ever
  *unions* witnesses from samples that re-parse, so the reported `sample_parse_failures` (the certification
  number) comes entirely from the diverse pass and is byte-identical for every grammar — wiring the reach
  pass cannot make any grammar's certification *worse*, only its coverage better. The reach pass probes the
  hardest-to-reach branches, so some of its own samples are *expected* not to re-parse; those are reported
  separately as an auxiliary count, never folded into the certification failures.
- **It is opt-in to certificate-coverage.** Ordinary stimuli generation (`--generate-stimuli`, the stimuli
  modules, the cross-family and oracle gates) is unaffected and byte-identical — the reach behaviour is a
  property of the witness pass alone.

### Reaching optional-gated rules: the plannable-rule reach pass

The recursive-depth case above is not the common shape of the residual `UNKNOWN`. Most never-witnessed
rules are *non*-recursive and perfectly shallow — they are simply gated by **un-taken optionals and
un-selected alternation branches** (the SystemVerilog-preprocessor `macro_default_text` is the canonical
example: two gating `?` optionals plus an 8-way choice). A bounded diverse pass can miss such a rule at
some seeds, and the recursive-reach pass above is structurally inapplicable (nothing depth-exhausts,
nothing recurses).

The certificate-coverage report therefore runs a **third pass** when `UNKNOWN` rules remain: for each
still-unwitnessed rule it computes the rule-reference path from the entry, installs a reach plan that
forces every ordered-choice decision **and every quantifier that path crosses to expand at least once**
(minimal generation alone would expand `?`/`*` to zero — the exact opposite of what an optional-gated
target needs), and generates a minimal candidate witness. The parser remains the judge: a probe only
counts when the sample **re-parses and the accepted parse actually entered the target rule**. A probe can
parse yet have its bytes routed through *other* rules — the report calls this `parsed-but-routed-elsewhere`
and retries a bounded number of times (terminal expansions vary per attempt while the structural skeleton
stays deterministic). Rules with **no path** in the rule-reference graph are flagged loudly as dead-rule
candidates for linter adjudication, and any rules beyond the pass's global attempt cap are reported as
left unattempted — never silently dropped.

#### The same capability, for *branch* targets — and the asymmetry that hid for a year

A reach plan can target two different things: a **rule** (witness this `UNKNOWN` rule) or a **branch**
(cover this ordered-choice alternative). Quantifier forcing was built for the rule-target installer and
given only to it. The branch-target installer built its plan from the ordered-choice directives alone,
so its `forced_quantifier_min` map was always empty — and the field's own doc-comment said so, in the
source, the whole time: *"EMPTY for every pre-H.7.2 caller."*

That asymmetry is invisible until a target sits **inside** an optional group:

```ebnf
nonrange_variable_lvalue :=
    ( implicit_class_handle dot | package_scope | class_scope )?   ← the target is one of these three
    hierarchical_variable_identifier nonrange_select
```

Minimal (`construct_mode`) generation renders the `?` **zero** times, so the inner choice is never
entered and every one of its alternatives reports `never_selected` — a witness *is* produced, it just
cannot possibly credit the target. In the SystemVerilog closed loop this was **43 of a 127-target
residual**, across two LRM profiles, and it looked like a generation-difficulty problem for several
slices because the count alone does not say *which way* a target failed.

The fix restores the symmetry rather than adding a mechanism: both installers now share one body, and
the crossed quantifier sites are derived from the **same walk** that builds the ordered-choice
directives, so the two can never describe different paths. The forcing is **opt-in per caller** — the
witness pass takes it; the primary/diverse target-drive pass does not, because its output must stay
byte-identical for the campaign's monotonicity guarantee. Measured effect: the residual fell
**127 → 83** with the target *universe* unchanged at 5,461 (coverage won, not denominator shrunk), and
the only non-timing line that moved in the entire gate summary was the residual itself.

**The transferable lesson:** when one capability is added to one of two sibling code paths, the gap is
not a bug you can see — it is an *absence*, and absences do not appear in traces. What surfaced this one
was classifying a residual by **failure shape** (`never_selected` vs `selected_but_failed` vs timeout)
instead of tracking its total. The shape named the mechanism; the mechanism named the missing line.

The first run of this pass moved the per-grammar `UNKNOWN` backlog substantially in one step — regex
98→19, VHDL 69→31, rtl_frontend 133→75, SystemVerilog 1134→738 — with every grammar's
`sample_parse_failures` untouched. It also demonstrated the duality working as designed in a second way:
on a handful of preprocessor seeds the probe *kept* parsing-but-routing-elsewhere, and chasing that
verdict exposed a genuine **parser bug** (a macro default argument on the *last* formal mis-parses into
the macro body — the grammar's default-text atoms can swallow the formals' closing parenthesis, where the
LRM requires a *balanced-parentheses* rule). The pass could not witness a shape the parser mis-parses —
exactly the attribution rule doing its job, with the defect routed to the grammar, not papered over in
the generator.

A later refinement made this pass **budget its generation depth per target** rather than by a fixed
multiple of the diverse default. Forcing a path to a correctly-reached target is only half the job: the
target's *own* minimal sub-derivation still has to complete, and for a grammar that nests a deep construct
inside a deep one — e.g. a synthesizable-RTL `generate if (…)` whose condition must descend a ~15-level
expression-precedence chain to a terminal — the combined depth (the path down to the nested target *plus*
its mandatory subtree) can exceed a one-size budget. When it does, the forced branch aborts with
"depth exceeded" and generation falls back to a shallow sibling, so the target re-parses-but-routes
elsewhere and never witnesses. The pass therefore sizes each target's budget as a reach-prefix allowance
**plus that target's own minimal-derivation depth**, computed by a small structural fixpoint (the depth
analogue of the shortest-terminal-length table — `Or` takes its shallowest alternative, `Sequence` its
deepest element, a `?`/`*` quantifier contributes nothing at its minimum). Deep targets get exactly the
room their minimal witness needs; shallow or genuinely-unwitnessable targets keep a tight budget and fail
*fast* (a construct that exhausts the per-attempt timeout is not retried — the same budget only reproduces
the timeout — and the redundant search fallback after a timeout is skipped). The effect is additive: a
deeper budget only ever witnesses *more*, never fewer, and grammars already certified clean never run this
pass at all. On the `rtl_frontend` subset this closed the dominant residual cluster (certificate-coverage
`UNKNOWN` 66 → 16, deterministic across seeds), and on SystemVerilog it both witnessed more rules and made
the pass markedly faster (the timeout guards turned slow run-to-timeout failures into fast ones).

A later, sharper refinement made the reach-path search itself **honest about lookaheads**. The path is
found by a breadth-first search over the grammar's rule-reference graph, and the original search descended
into *lookahead* sub-expressions (`&X` / `!X`) when collecting reference sites. But a lookahead is a parser
assertion that consumes no input — the generator materialises **nothing** for it (generation of a lookahead
node yields the empty string) — so a rule referenced *only* inside a lookahead is never positively emitted on
a path that crosses it, and the parser can never *enter* it there (and the witness primitive records rule
*entry*, not assertion). Treating such a reference as a reach edge **poisons** the search. On the
synthesizable-RTL subset every reserved keyword also appears in the identifier rule's negative-lookahead
exclusion list (`non_keyword_identifier := !kw_always … !kw_wire simple_identifier`), so a module-body
keyword like `always_latch` or `localparam` looked "reachable" through a *typedef name's* identifier — a
shorter, bogus path than its real one. The reach plan then forced the file's first choice to *typedef*, and
the construct could never witness the keyword: it fell back to the shortest declaration and routed elsewhere.
(A keyword such as `module`, whose *positive* reference sits on a shorter path than its lookahead reference,
escaped the trap — which is exactly why the symptom hit only the deeper-nested keywords.) The fix is to
follow only **positively-emitted** rule references — the reach search no longer descends into lookaheads — so
each keyword's real positive path (e.g. via the procedural block) is the one found. A rule referenced *only*
inside a lookahead then correctly has **no** reach path at all and is flagged as a dead-rule candidate for
linter adjudication — the same `!`/`&`-only class the attribution rule already names, now surfaced honestly
by the search instead of chased through a path that could never witness it. The change is additive and
parser-agnostic (keyed purely on the lookahead node shape): it only ever sharpens *which* path is taken,
never loosens the witness check. On `rtl_frontend` it moved the residual `UNKNOWN` 16 → 9 (deterministic
across seeds), with SystemVerilog byte-identical (the same rules, now correctly reported as no-reach-path).

That "dead-rule candidate — adjudicate via the linter" flag is no longer a *manual* TODO: the linter now
**adjudicates it automatically** at the proof layer. A rule that is structurally reachable but **not
positively reachable** — reachable only through `!`/`&` lookahead edges, the exact `!`/`&`-only class the
reach search now surfaces — is classified `covered_by_proof` via a sound `LookaheadOnlyRule` certificate
(re-derived independently as `reachable ∖ positively_reachable`, where `positively_reachable` repeats the
structural reachability walk but does *not* descend into lookahead nodes). So such a rule leaves the
`UNKNOWN` bucket with a *checkable proof that it is correctly un-witnessable*, exactly as the certifying
model intends — never silently, and never by removing a rule that does real parse work. On `rtl_frontend`
this closed `port_direction_token` (the sole lookahead-only rule, `UNKNOWN` → `proof`); on SystemVerilog it
was *additive* (one genuinely lookahead-only rule moved `UNKNOWN` → `proof`, every witness preserved).

A further refinement budgets for a target's **mandatory off-path siblings**, not just its own subtree. The
per-target budget above sizes depth as the reach-prefix allowance *plus the target's own minimal subtree* —
which under-budgets a **shallow** target whose forced construct must *also* complete a **deep mandatory
sibling that is not in the target's own subtree**. The synthesizable-RTL edge-control keyword is the
canonical case: to witness `negedge` the construct must build `always_ff @(negedge <expr>)`, and the
grammar's `event_control_item := event_edge? rtl_expr` makes the `rtl_expr` (a full ~15-level
expression) a *mandatory sibling* of the shallow `event_edge` that carries the keyword. The keyword's own
subtree is tiny, so the prior budget aborted the construct on depth and fell back to a shallower procedural
block. The fix is a **two-tier per-target budget**: tier 1 is the existing baseline (so every target that
already witnessed is **byte-identical**); tier 2 — a deeper budget that also covers the *deepest mandatory
off-path sibling along the reach path* — is tried **only when tier 1 did not witness**. This is the
careful part, because an unconditional deeper budget *regresses* coverage: ubiquitous rules like a numeric
literal or a binary operator are reach targets that get witnessed by a *shallow fallback sample* (any
expression contains one), and a deeper budget makes the forced construct attempt the full deep expression
(through, e.g., the conditional-expression ternary on its path) and time out instead of falling back. The
two-tier gating keeps those fallback-witnessed rules on tier 1 and pays the deep budget only for a genuinely
**rare** construct the fallback can never reach — so the change is *strictly additive* (no already-witnessed
rule can regress) and parser-agnostic. On `rtl_frontend` it moved the residual `UNKNOWN` 9 → 7 (the
edge-control keywords, deterministic across seeds), with SystemVerilog additive (it witnessed eight more
rules, none lost).

A last shape in this family is a forced branch that is itself **self-recursive**. The synthesizable-RTL
unary-operator rule is the canonical case —
`unary_expr := … | bang unary_expr | tilde unary_expr | primary_expr` — where the `bang`/`tilde`
alternatives carry the operator *and* a recursive operand. To witness `bang` the reach plan forces
`unary_expr`'s ordered choice to the `bang unary_expr` alternative; but the directive is keyed on the rule
and its OR-node position (`(unary_expr, root)`), and the operand re-enters `unary_expr` at exactly that
site, so the directive **re-fires** — it forces `bang` again, and again, building `!!!!…` until the depth/
visit budget exhausts and the construct fails. The deeper budget above cannot help here: it only buys more
recursion before the same failure, so `bang`/`tilde` produce no witness at all. The fix is to make a
self-recursive forced branch fire **once**. A reach path is a simple walk (each rule appears on it once), so
any *re-entry* of a rule already live on the generation stack is recursion *below* the directive's single
intended firing — the operator was already selected on the shallow entry, and the operand now only needs to
*terminate*. So when the forced branch references the very rule its directive is keyed on (a structural
self-reference) **and** that rule is already on the call stack, the directive is suppressed for the
re-entry: the operand falls through to the minimal-derivation ordering and takes its shortest terminating
alternative (`primary_expr`), yielding `!a`. The directive still fires on the shallow entry, so the operator
is selected and witnessed. The check is purely structural — the forced branch's self-reference plus the live
recursion count, never a rule name — so every non-self-recursive directive is byte-identical, and grammars
already certified clean never run the pass. On `rtl_frontend` it moved the residual `UNKNOWN` 7 → 3: it
witnessed `bang`/`tilde`, and — because the operator's reach path renders a full `bit[ … ]` range expression
— also `repetition_expr` and `concatenation_expr`, with SystemVerilog byte-identical.

A last refinement in this family is about *which branch a reach path travels through* when a rule offers
several routes to the same successor. The reach path is a breadth-first search over the rule-reference
graph, and when more than one reference site in a rule reaches the same next rule, the search originally
took whichever the structural walk enumerated first. For a self-recursive rule whose *earlier* alternative
is the recursive one, that first site lives inside the recursive branch. The synthesizable-RTL conditional
expression is the canonical case — `conditional_expr := logical_or_expr ? conditional_expr : conditional_expr
| logical_or_expr` — where the ternary (branch 0) precedes the plain pass-through (branch 1), so
`logical_or_expr` was discovered through the ternary and the installed directive forced `conditional_expr`
to its ternary branch. Because that directive is keyed on the rule and its ordered-choice position, it then
fired on *every* entry to `conditional_expr`, including the low and high bounds of an unrelated
`bit[ … : … ]` packed range. So a reach probe that only needed to witness a numeric literal deep under
`logical_or_expr` instead rendered a packed range full of nested `?:` ternaries — which the parser rejects,
compounded by the based-integer token class itself including `?`, so maximal munch fused the ternary marker
into the literal. The fix is reach-path *honesty*: when several sites reach the same target, prefer the one
whose enclosing top-level alternative does **not** reference the rule itself — the non-self-recursive branch
— so the directive forces the *plain* branch (`conditional_expr → logical_or_expr`) and every entry derives
a minimal expression (`bit[ 8'b1 : 8'b1 ]` witnesses the literal). The preference is a stable reordering
keyed purely on the rule's own top-level ordered-choice structure and a structural self-reference test, so a
rule with no top-level choice mixing self-recursive and non-self-recursive alternatives is byte-identical,
and grammars already certified clean never run the pass. On `rtl_frontend` it moved the residual `UNKNOWN`
3 → 2 (it witnessed `based_integer`, the last expression over-generation target), and on SystemVerilog it
was *additive* — its own self-recursive expression rules now reach plainer branches, witnessing three more
rules (`UNKNOWN` 619 → 616) with none lost.

#### The closed-loop witness pass's per-target depth budget

The two witness passes described so far are *different code*: the certificate-coverage pass witnesses
**rules**, the closed-loop stimuli pass witnesses the **coverage targets** a replay run still owes. The
per-target depth budget above was built for the first and given only to it. The second kept multiplying
the configured `--max-depth` by a flat two — and that single missing capability was, measurably, the
entire surviving SystemVerilog class-A residual.

The failure is not "the target is too deep". Three numbers, measured over every residual branch in both
LRM profiles, say where the budget actually sits:

| number | what it is | on the residual |
|---|---|---:|
| the witness budget | the gate's `--max-depth 20`, flatly doubled | **40** |
| the committed derivation | what minimal (`construct_mode`) generation actually builds | **42 – 54** |
| the shallowest derivation | the shortest one that exists at all | **18 – 36** |

**The budget sits between them.** Minimal generation orders each ordered choice by minimum *terminal
length* (Purdom SHORT) and then commits to that one alternative with no fallback inside the committed
path — and minimum terminal *length* is not minimum *depth*. The shortest-in-tokens derivation of a
SystemVerilog `property_expr` descends the property-operator ladder and then the expression precedence
cascade, 8–14 levels past a budget a shallower derivation would have fitted inside. The forced branch
then dies on depth, a shallow sibling rescues the enclosing rule, the rule returns success, and the
target is silently left uncredited — which is why every pass-level counter reported zero failures while
the per-branch record (see [Reading the residual](stimuli-and-quality.md)) carried one `depth exceeded`
entry per residual branch.

So the closed-loop pass now sizes each target the same way — with one correction that matters. The
certificate-coverage formula is `reach prefix + the minimal derivation depth of the target RULE`, and a
rule's minimal depth is the depth of its *shallowest alternative* — precisely the alternative a residual
**branch** target is not. Applied verbatim it under-budgets a container rule with a shallow minimum and a
deep residual branch. The closed-loop pass therefore scopes the addend to the **targeted alternative**:
`2 × --max-depth` (the reach-prefix allowance) plus the minimal derivation depth of that branch's own
subtree, falling back to the rule-scoped form for a rule target or an unresolvable branch. Measured over
the SystemVerilog class-A targets, the rule-scoped formula clears 37 of 40 and the branch-scoped one
clears 40 of 40.

Both addends are non-negative, so every budget only ever **grows** relative to the flat one: a target that
witnessed before witnesses identically now, which makes the change additive by construction rather than by
argument. Nothing outside this pass reads the per-target budget, so the diverse pass stays byte-identical
and the run stays deterministic. And a global `--max-depth` raise remains the *wrong* lever, now with a
number attached: raising it from 20 to 30 made the run at least 3.9× slower and it did not finish its
first phase in 40 minutes, because the raise is global — it reshapes the diverse and target-drive passes
too.

#### The closed-loop witness pass's raised entry for store-gated targets

Rooting each witness at the target's **own rule** is what makes the closed-loop pass work: the whole
depth budget goes to the target's subtree instead of being spent descending to it. For every target
whose acceptance depends only on *structure*, that policy is strictly better than starting from the
grammar entry. For one shape it is structurally fatal.

A rule can be gated by a **positive semantic-store predicate** — it is parser-accepted only if some
fact was emitted *earlier in the same sample*. If the only rule that emits that fact lives **above**
the target, then a witness rooted at the target starts from an **empty store**, the generation-side
store-aware prune refuses the rule, and the target reports `never_hit`. This is not a budget failure
and no budget can fix it: at every depth and every seed, the sample the policy can build is a sample
the predicate must reject. Nor can the semantic prelude help, because a prelude hosts its extra
producer iterations at an on-path quantifier site *earlier in the sample* — and when the entry **is**
the gated rule, there is no earlier.

The blocker is therefore the **entry policy**, and the sound policy already exists one pass over: the
certificate-coverage plannable pass generates from the grammar entry and steers *down* via reach hops,
quantifier forcing and exactly that producer prelude. So the closed-loop pass now uses it too — for
these targets only. A target is **store-entry-blocked** when its mandatory rendering forces a positive
store gate whose fact-kind no rule in the target rule's own reachable closure can emit; such a target
is generated from the run's entry rule with a rule-reach plan installed, and every other target keeps
the own-rule entry byte-for-byte.

Three scoping decisions carry the weight, and each has a failure it prevents:

- **Gates the generator actually prunes on.** A `lacks_fact` gate is *satisfied* by the empty store a
  standalone witness starts with. Counting it — as the reach-BFS edge-deprioritization deliberately
  does, because there over-counting only re-ranks two ways of generating the same thing — would raise
  the entry for targets that witness perfectly well today. ⛔ This scoping was first written as
  *"positive gates only"*, and that was one criterion short; see [What the verdict must actually ask
  about](#what-the-verdict-must-actually-ask-about-pruning-not-polarity).
- **Mandatory descent, not "a gate exists below here".** An ordered choice offering an ungated
  alternative is an escape the generator simply takes. A whole-closure scan calls the SystemVerilog
  net-declaration rule blocked; the mandatory walk correctly does not, because its first alternative
  is a plain `wire a;`.
- **Closure-relative, not absolute.** The same gate on the same rule is *not* blocking when a producer
  for its fact-kind lives inside the target's own subtree — the witness can then emit the fact itself.

The verdict is an over-approximation of what the subtree can emit (it ignores render order), which
biases it toward *not* raising: a missed raise leaves prior behaviour untouched, a spurious one would
replace a working generation. Measured on SystemVerilog, exactly **one** target in the whole run takes
the raised entry, and the witness pass reports it on its own summary line as `store_entry_raises=`.

One witness can settle more than one target, and here it does. The gated rule is referenced from
exactly one place — the net-declaration alternative that is *also* a residual coverage target — so the
raised-entry witness necessarily selects and succeeds that branch on its way to the rule. The gap
report had already recorded the dependency (`depends_on`), and closing the cause closed both effects:
per LRM profile, `rule::wildcard_escape_nettype_identifier` (`never_hit`) and
`branch::net_declaration_sv_…::root#2` (`selected_but_failed`) resolved together.

Measured on the gate's own closed-loop replay stage, single-variable (same depth, same seed, one code
change): residual **2 → 0** on both LRM profiles, each profile's two arms agreeing exactly.
List-diffed on `sv_2017`: **2 resolved, 0 new**, with `covered_rules` 1336 → 1337 and
`covered_branches` 1450 → 1451 and the unreachable debt unchanged — exactly the pair, nothing else
moved. The run also got marginally *faster* (447 s vs 455 s), because a target that can never succeed
had been paying a full construct attempt and then a full search fallback before giving up.

#### Raising the entry for a store-gated BRANCH target

The paragraph above closes a **branch** target as a side effect: the raise fired for the gated
*rule*, and because that rule is referenced from exactly one place, the witness selected the branch
on its way past. Both of those are coincidences of today's grammar, and the second one is not even
required — remove either and the branch is uncovered again with the rule still closed.

That is not hypothetical. Bounding the generator's depth-slack retry (see
[Stimuli and Quality](stimuli-and-quality.md)) makes the *target-drive* pass strictly better, so it
resolves the gated rule early; the rule is then no longer a residual target when the witness pass
runs, the raise never fires, and the branch reopens as `selected_but_failed`. A pure improvement
elsewhere in the generator took a covered target away.

So the raise applies to **branch** targets on the same verdict, with one addition that is the real
work. A branch plan already accepted an entry rule distinct from its target rule, and the blocked
verdict was already branch-aware — it judges the *targeted alternative* first. What was missing is
the **semantic prelude**: the rule-target installer attaches one, the branch-target installer never
did, so a raised branch witness would steer down correctly and still render the gated rule against
an empty store. Attaching it needs the prelude's gate discovery to be branch-aware too, for a
structural reason worth stating:

> A mandatory descent refuses an ordered choice that has an ungated escape — correctly, because the
> generator would simply take the escape and need no prelude. But a plan that **forces one
> alternative** has removed the escape. For that plan, the targeted alternative's mandatory render
> *is* the whole render, so discovery starts at the alternative node rather than at the rule.

Both the count-gate and the name-gate prelude builders gained that start, tried **last** so every
gate the existing legs already find is found identically, and the name-gate one keeps the same
"only when the render is unavoidably store-gated" guard that stops a prelude being armed on an
alternative which parses perfectly well from an empty store. The prelude-bearing branch installer is
a separate entry point that only the raised-entry arm calls — the plain branch installer, used by
every other target, is untouched.

Measured on the same replay stage. In the bounded-retry configuration — the only one where this is
observable, because at the default the branch is still closed transitively — `sv_2017` residual
**1 → 0** and the re-opened `branch::net_declaration_sv_2017::root#2` closes. The witness the fix
produces is `import\foo ::*;package\foo ;\foo \foo ;endpackage`: the real parser accepts it
(`parse_full passed`) and rejects the byte-identical sample with only the import removed
(`furthest_position=17`, the escaped net-type identifier) — so the prelude is doing the work the
gate needs, not merely appearing. At the default configuration the change is coverage-neutral and
slightly cheaper: identical `covered_rules`/`covered_branches` and identical debt lists on both
profiles, with **14 fewer witnesses** needed on `sv_2017` and 13 fewer on `sv_2023`, because a
raised-entry witness is a whole-file sample that settles more targets at once.

#### The attempt order is the guarantee

Both paragraphs above decide the raise from the **verdict alone**. That verdict is a deliberate
over-approximation — its own design note says the bias exists so that a *missed* raise leaves
behaviour untouched, while a *spurious* one would replace a working generation. Deciding from it and
then generating **only** from the raised entry puts the whole guarantee on the bias being right.

So the pass no longer does that. Every target — blocked or not — is attempted from its **own rule
first**, exactly as an unblocked target always was; the raised entry is installed only when that
attempt leaves the target **uncovered**. Three things follow, and the third is why this is worth a
section:

- a target that witnesses from its own rule keeps witnessing from its own rule, on **any** grammar,
  whatever the verdict says — the property is structural rather than measured;
- the only behaviour that can change is for targets whose own-rule attempt did not cover them, which
  is strictly more coverage, never less;
- the verdict's over-approximation becomes a **cost** knob instead of a correctness assumption.

The gate is *"is the target covered now"*, **not** *"did generation return an error"*. A forced
branch whose gated content prunes lets a sibling rescue the rule, so the attempt returns `Ok` with
the branch still uncredited — keying on the error would strand exactly the branch class the previous
section exists to close.

⭐ **And the hypothetical turned out to be the live case.** On the same replay stage, ordering the
attempts drops `store_entry_raises` from **16 to 1** on `sv_2017` and from **11 to 1** on `sv_2023`,
with residual `0`/`0`, every coverage figure identical and all four debt lists unchanged. That is
**15 of 16** and **10 of 11** raises that had been rerouting a target which witnesses perfectly well
from its own rule; the single survivor per profile is the genuine store-entry-blocked target the
raise was built for. The earlier measurement — *"15 and 14 targets take the raise and not one target
is lost"* — was true and could not see this, because replacing a working witness with another
working witness moves no coverage number. **A mechanism whose failure mode is invisible to the
metric you gate on is not validated by that metric being green.** Tightening the verdict itself is
tracked separately; it is now a cost question, not a correctness one.

The cost was measured rather than assumed, and it is not what the design predicted. The extra
own-rule attempts were expected to be provably wasted work for a correct verdict — instead
`sample_errors` is **unchanged** (50 and 97), so every one of them succeeds. What the run pays is
`+14` and `+6` witnesses, because the raised whole-file samples had been settling neighbouring
targets for free; stage elapsed moves `256 → 258 s` and `444 → 436 s`, inside run-to-run noise in
both directions.

#### What the verdict must actually ask about: pruning, not polarity

The section above leaves the verdict wrong 15 times in 16 and merely harmless. Fixing it turned out
not to need a cleverer walk — it needed a different **question**.

Naming the 25 spurious targets (the census, [Stimuli and Quality](stimuli-and-quality.md)) showed
they are separated perfectly by the **class of gate** the targeted alternative mandatorily reaches:

| class | predicate | census verdict |
|---|---|---|
| **count** | `fact_count_at_least` | 1 of 1 **genuine** |
| **name** | `has_fact`, `fact_attribute_equals` | 15 of 15 (`sv_2017`) and 10 of 10 (`sv_2023`) **spurious** |

One rule carries both, adjacent, with identical bodies: `net_declaration_sv_2017` branch `#2` leads
with a count-gated identifier and is genuine; branch `#1` leads with a name-gated one and is not.

The mechanism is a property of the generator that can be checked rather than argued: **there is
exactly one generation-side store prune, and it is count-only.** A rule whose `fact_count_at_least`
gate cannot be satisfied is refused before it renders (`STORE-AWARE-GEN: … predicate unsatisfiable
(zero source facts)`). A **name** gate has no analogue anywhere — it is consulted only when planning
a prelude, when replaying a declared name into a gated consumer, and when repairing a colliding free
name; never to refuse a render. So a name-gated rule renders a fresh identifier against an empty
store and its target is credited.

⇒ The verdict claims *"this target cannot be generated from its own rule."* Only the prune can make
that true, so the walk's scope is now exactly the map the prune reads — the count gates. Polarity was
the right *first* criterion (a negative gate is satisfied by an empty store) and one criterion short:
a positive gate that never prunes cannot block generation either.

Measured single-variable on the replay stage, census enabled on both arms: the blocked population
falls **16 → 1** on `sv_2017` and **11 → 1** on `sv_2023`, spurious **15 → 0** and **10 → 0**, the
genuine target unchanged on each — and `store_entry_raises` stays `1`/`1`, residual stays `0`/`0`,
and all **eight** stage artifacts are **byte-identical**.

⭐ That byte-neutrality was *predicted before it was measured*, which is what makes it evidence. The
raised arm is guarded `!covered && blocked`, and `&&` short-circuits — so for a spurious target,
covered by its own rule by definition, the verdict was never consulted at run time even before the
change. Ordering the attempts had already turned a correctness assumption into a cost knob; this
turns the knob to zero.

⚠️ What the narrowing gives up, stated rather than implied: the incidental benefit a raise might have
brought a name-gated target that the own-rule attempt left uncovered for some *other* reason. No such
target exists on either SystemVerilog profile, and a missed raise now costs coverage only where the
own-rule attempt also failed — but on another grammar one could exist.

### Reaching store-gated rules: the semantic-prelude reach

One last shape of unwitnessable rule remains after the recursive-depth and optional-gating passes: a rule
gated by a **semantic-store predicate** that no minimal derivation can satisfy. The regex grammar's
multi-digit numeric backreference is the canonical case: `\NN` (two or more digits, so `NN ≥ 10`) is
parser-accepted only when **at least `NN` capture groups appear earlier in the same pattern**
(`@predicate fact_count_at_least(regex_capture_group, $index)`). The generator's store-aware mode
correctly refuses to emit the backreference when zero groups exist — that is the round-trip-validity
guarantee — but it means the plannable-rule pass alone can never witness the rule: the witness needs a
**fact-emitting prelude** the minimal path does not contain.

The plannable-rule pass therefore carries a plan-scoped, two-phase extension for count-gated targets:

1. **Capture (phase 1).** With the count-prune bypassed *for the planned target only*, one probe
   generates the gated rule and records its rendered text plus the numeric reference value `v` it
   happened to produce (say `\37`). That probe is expected not to witness — it lacks the source facts —
   and lands in the pass's auxiliary probe-failure count, never in the certification number.
2. **Prelude + replay (phase 2).** The next probe injects exactly `v` extra iterations at the on-path
   quantifier site, each steered (by a nested reach plan) to a rule that **emits** the consulted fact —
   for regex, `v` capture groups, each rendering `()` — and then replays the captured text for the gated
   rule. The result, `()()…()\37`, is a grammar-valid sample whose post-predicate holds, so the real
   parser accepts it and the accepted parse enters the gated rules: witnessed.

Fitting the count to the captured value — rather than constraining the generated value to a pre-chosen
count — is what makes this work without any value-selection machinery: the prelude size is *derived from*
whatever the grammar rendered. The mechanism is keyed entirely on grammar structure (the predicate's
fact-kind, the `@emit_fact` producers, the rule-reference graph, the on-path quantifier sites), never on
grammar names; grammars with no count-gated rules attach no prelude and behave byte-identically. And as
everywhere in this gate, the parser stays the judge: a mis-fitted prelude can only fail loudly, never
manufacture a false witness.

### Reaching constraint bodies: store-free reach-honesty

A last shape of unwitnessable rule is a target reachable through **two carriers** — one store-gated,
one not — where the reach search picks the gated one. The canonical case is the SystemVerilog
constraint body. The `constraint_block` subtree (`constraint_block_item`, `constraint_expression`,
`constraint_primary`, `solve_before_list`, `uniqueness_constraint`, `loop_variables`, the
`solve`/`before`/`soft`/`foreach` keywords, …) is reachable BOTH through the in-class
`constraint_declaration` (`constraint c { … }`, which parses with no semantic state) AND through the
out-of-class `extern_constraint_declaration` (`constraint C::c { … }`, whose mandatory `class_scope`
is store-gated — it must name a *declared class*, which a minimal witness has not declared). The reach
search is a fewest-hops breadth-first walk with first-discovery-wins, and the out-of-class carrier is
the **shorter** path (a top-level item, versus descending `class_declaration → class_item` to reach
the in-class one). So the whole subtree was discovered through the gated carrier, the reach plan forced
that branch, the parser correctly rejected the undeclared-class form, and none of the body rules ever
witnessed — even though a perfectly good store-free witness exists through the other carrier.

The fix is a **store-free reach pass**. The general principle is the same attribution rule applied to
*path selection*: when a target is reachable without crossing a store-gate the path cannot satisfy,
prefer that route. The pass detects a "store-gated edge" structurally — descending into the edge's
target rule from a mandatory position is forced through a rule carrying a fact-query `@predicate`
(`has_fact` / `lacks_fact` / `fact_attribute_equals` / its dual / `fact_count_at_least`) whose
consulted fact-kind is not emitted anywhere earlier on the path (an ordered choice counts as gated only
when *every* alternative is gated; an optional/`*` quantifier never forces its body; a lookahead
renders nothing; the walk is cycle-guarded, transitive, and profile-aware). It then routes residual
targets through the store-free carrier when one exists.

Two properties keep it honest, and the second is the interesting one. First, the parser stays the
judge — a re-routed probe can only fail to witness, never manufacture a false witness. Second, the
pass is **strictly additive by construction**, and getting there took a discarded first design worth
recording. The natural implementation — make the reach search itself prefer store-free edges for
*every* target — did drop the SystemVerilog backlog (`UNKNOWN 84 → 67`) but also **de-witnessed one
unrelated rule**: a store-gated identifier that had only ever been witnessed *incidentally*, as a
bystander of some other target's probe whose path the re-routing changed. A coverage gate must never
trade one rule's witness for another's, so that design was rejected. The landed design instead leaves
the original reach search completely untouched (so every existing probe — and all its incidental
bystander coverage — is byte-identical) and runs the store-free routing as a **separate, final pass
over only the rules still unknown**, which can therefore only *union* new witnesses. The result is the
same headline number with no regression: SystemVerilog `UNKNOWN 84 → 67` (witness `1204 → 1221`),
deterministic across seeds, with zero newly-unknown rules and every other grammar — including the
fully-certified roster — byte-identical (the pass is structurally inert for any grammar with no
fact-query predicate). The constraint-body cluster now witnesses through the in-class carrier; the
genuinely store-gated remainder (`extern_constraint_declaration` itself, the declared-class-name
identifiers) correctly stays unknown, awaiting the semantic-store-aware generation that can synthesise
the declarations they require.

That synthesis is the next slice: **declare-then-use name coordination**. Many SystemVerilog rules are
*use-sites of a declared name* — `checked_type_identifier` (a type reference) is accepted only when a
`has_fact(type_name, …)` post-predicate holds, `known_unscoped_covergroup_type_identifier` only when a
fact of kind `type_name` with `declaration_family = covergroup` was emitted earlier. A minimal witness
generated in isolation writes a *fresh* identifier — `\foo` used as a type with no `typedef`/`class`/
`covergroup` declaring it — so the predicate is false and the sample does not re-parse. The earlier
"semantic prelude" already solved the analogous *counting* problem for regex backreferences (it
synthesises N capture groups upstream so `\N` is in range); this slice generalises it from counting to
*naming*. When the reach path crosses a name-gated rule, the witness pass now (1) hosts one extra
iteration of an upstream declaration whose `@emit_fact` registers a name — e.g. a real
`covergroup g; … endgroup` — choosing a producer whose `declaration_family` matches what the use-site
demands; (2) records the *actual rendered identifier* that declaration emitted into the generation-time
store; and (3) forces the use-site to render **that same name**, so the generated sample reads
declare-then-use (`covergroup g; … endgroup … g …`) and re-parses cleanly. The parser re-check stays the
only judge — a mis-coordinated name simply fails to witness, it can never manufacture a false one. A rule
that is *itself* a declaration (it emits the very fact it checks — the forward-declaration idiom) is left
out of this cohort, since forcing it to re-render an already-declared name would make it redeclare and
reject. The effect: the directly-reachable store-gated cohort — the `checked_*` and `known_unscoped_*`
type/covergroup/nettype/let/parameter identifier family — now witnesses, dropping SystemVerilog
`UNKNOWN 56 → 46` (witness `1232 → 1242`), deterministic across seeds, with zero newly-unknown rules and
every fully-certified grammar byte-identical (the path is capability-gated on the predicates' presence, so
it is structurally inert for any grammar without name-matching store gates).

A follow-up increment closed the **block-scoped carriers**. The gate that matters often lives one level
*inside* a use-site rule: `known_unscoped_block_type_identifier := checked_type_identifier packed_dimension*`
is not itself name-gated — its inner `checked_type_identifier` carries the `has_fact(type_name, …)` gate.
The first cut only armed the declare-then-use prelude when the witness *target* (or a rule on its reach
path) was *directly* gated, so these carriers got no prelude and their minimal witnesses still used an
undeclared `\foo`. The fix lets the prelude follow a rule's **mandatory-first prefix**: when the target is
not itself gated, it descends the leading mandatory rule reference (e.g. `→ checked_type_identifier`) and
arms on that inner gated rule — the one that actually renders — while a directly-gated rule still
short-circuits exactly as before (purely additive). The flat fact store means the existing top-level
declaration hosting satisfies the inner `has_fact` even when the use-site sits deep inside a procedural
block. This witnessed the block-scoped carriers (`known_unscoped_block_type_identifier`,
`known_unscoped_block_covergroup_identifier`, and a block-context class-type provisional), dropping
SystemVerilog `UNKNOWN 46 → 43` (witness `1242 → 1245`), again deterministic with zero newly-unknown rules
and every fully-certified grammar byte-identical. The deeper remainder that needs a *class scope* or
*class-member* context around the use-site (`extern_constraint_declaration`, the class-scoped call family)
is the next increment.

The next increment closed the **class-scope** carriers, and the way it was found is a small lesson in
*where* a constraint actually lives. A use-site like `known_unscoped_class_scope_class_identifier` needs
its name declared as a *class* specifically (`fact_attribute_equals(type_name, …, declaration_family,
class)`), so the prelude must host a producer whose declaration emits a `class`-family fact. Two such
producers exist — a *forward* declaration (`typedef class foo;`) and a *typedef alias*
(`typedef <existing-type> foo;`) — and the prelude was committing the alias, whose declaration **cannot
parse in isolation**: its source type is itself a `has_fact(type_name, …)` gate, so on an empty store
`typedef \foo \foo ;` is rejected at the source-type position and no class fact is ever recorded. The
instinct was that the *producer rule* was wrong, but the two producers share a byte-identical body — the
difference is one level up, in the **host branch** each is reached through: the alias is wrapped in
`kw_typedef class_type … ` (a gated mandatory *sibling*), the forward in `kw_typedef kw_class … ` (a clean
keyword). So the constraint is a property of the *path that renders the declaration*, not of the producer
rule. The fix makes producer selection **reach-path-aware**: a first pass walks each candidate producer's
forced reach path and skips any whose mandatory off-path siblings force a store-gate the empty-store
prelude cannot satisfy (reusing the same mandatory-descent store-gate analysis the store-free reach pass
uses), preferring a producer reached through a *self-contained* declaration; a second pass restores the
prior "first reachable producer" behaviour, so a candidate is never dropped and an already-witnessed
target — whose path is necessarily clean — keeps its exact prelude. With it, the forward declaration is
chosen, the class fact is recorded, and the class-scope use-sites witness:
SystemVerilog `UNKNOWN 43 → 41` (witness `1245 → 1247`), deterministic at seeds 0/7/42 with zero
newly-unknown rules and every fully-certified grammar byte-identical (the path is capability-gated on the
name-matching predicates, so it is structurally inert for any grammar without them).

A further increment widened *where* the declare-then-use prelude looks for its gate. The first version
only found a gate on a rule's mandatory-**first** element — but the out-of-class
`extern_constraint_declaration` family hides its gate behind a leading `constraint` keyword and *inside an
ordered choice* (`class_scope → class_scope_type := ( … | known_unscoped_class_scope_class_identifier |
… )`), so no prelude armed and the forced witness `constraint \foo :: \foo {}` was rejected (the class
`\foo` was never declared). The discovery now also scans past a leading run of optional/keyword elements
and descends an ordered choice — but only as far as the **first rendered position that is unavoidably
store-gated**, and only into a choice that has *no ungated escape*. That boundary matters: a first attempt
that scanned every element and every choice regressed six rules, because it reached a deep, dodgeable
`type_name` gate sitting *behind* a self-satisfying producer (a parameter declaration) and armed a
structurally-invalid prelude — a regression the deterministic strict-subset gate caught immediately. The
refined discovery, reusing the same mandatory-descent store-gate analysis, stops at the first genuinely
unavoidable gate and so arms a prelude only where one is both needed and safe:
SystemVerilog `UNKNOWN 41 → 38` (witness `1247 → 1250`), again deterministic at seeds 0/7/42 with zero
newly-unknown rules and every fully-certified grammar byte-identical.

The next increment was a subtler kind of generator⟷parser disagreement — not a missing declaration but a
*name collision*. A witness for `property_qualifier` (`rand`/`static`/… on a class property) wraps the
target in a class, and the witness generator names both the class and the property's variable with the same
canonical identifier `\foo`. Naming the class emits a `type_name` fact, so the parser then sees the
variable's `\foo` as a *known type* and — because a declaration's type slot
(`data_type_or_implicit := data_type | implicit_data_type`) tries the type branch first — greedily consumes
it as the type, leaving no variable name and rejecting the sample. The fix is the generation-side dual of
the parser's own type-vs-identifier disambiguation: **collide-aware free-name diversity** — when a free
declaring identifier would re-render a name already in the store under a fact kind some name-gate consumes,
the generator renders a distinct name instead (`\foo` → `\foo_0`). Keyed on the grammar's own name-gate
kinds (never a rule name) and active only inside the witness pass, it is byte-identical everywhere else:
SystemVerilog `UNKNOWN 38 → 37` (witness `1250 → 1251`), deterministic at seeds 0/7/42 with zero
newly-unknown rules and every fully-certified grammar still green.

The next increment reached the declare-then-use prelude to a gate carried by a mandatory **off-path
sibling**. The first two prelude-discovery legs look only at the directly-gated reach *hops* and at the
*target's own* mandatory prefix. But some targets are reached *through* a host whose store-gate is a
mandatory sibling of the on-path element — rendered alongside the path, yet on neither place the legs
inspect. The canonical case is the SystemVerilog constraint body `constraint_set`, reached via the
out-of-class `extern_constraint_declaration_sv_2017 := ( kw_static )? kw_constraint class_scope
constraint_identifier constraint_block`: the reach path forces the last element (`constraint_block`, to
descend into the body), but the `class_scope` two elements earlier is a *mandatory* sibling the forced
derivation must also render — and it is store-gated on naming a *declared class*. So the forced witness
`constraint\foo ::\foo {…}` rejected on the undeclared class `\foo`, and no prelude armed because neither
the hops nor `constraint_set`'s own prefix carries the gate. The fix adds a third discovery leg that walks
each reach hop's reference site and, at every mandatory off-path **sequence** sibling of the on-path
element (the only positions that render alongside the path), finds the positive name gate that sibling's
render routes through — reusing the *same* "stop at the first unavoidably store-gated position, refuse any
ordered choice with an ungated escape" guard that bounds the prefix-scan leg, so a prelude arms only where
one is both needed and safe. The existing machinery then does the rest unchanged: it hosts a
self-bootstrapping forward-class declaration (`class\foo ;endclass`) upstream and forces the off-path
`class_scope` identifier to echo `\foo`, so the constraint parses with a declared class. The leg is tried
only when the first two find nothing (every already-armed target is byte-identical) and runs only for
still-unknown targets (it can only *add* witnesses), with the parser the sole judge. It closed not just
`constraint_set` but three more targets of the same shape — `declared_class_alias_identifier` (whose
`typedef <source-type> <alias>` host renders the gated source type as a sibling of the alias) and the
checker-bind pair `named_checker_port_connection` / `…_sv_2017` — their witnesses showing the prelude and
the earlier collide-aware free-name diversity composing cleanly (`class\foo ;endclass typedef\foo \foo_0 ;`,
`checker\foo ;endchecker bind\foo_0 \foo \foo_0 (.*);;`): SystemVerilog `UNKNOWN 37 → 33`
(witness `1251 → 1255`), deterministic at seeds 0/7/42 with zero newly-unknown rules and every
fully-certified grammar still green (the leg is structurally inert for any grammar without name-matching
store gates). The residual is now the reach-*routing* cohort — rules that reach their context but route
through a sibling once there — plus a small declaration remainder.

The next increment attacked that reach-*routing* cohort with a **carrier-diversification reach pass**.
The reach passes so far steer to a target through the *shortest* path, which the breadth-first reach
walk discovers by reaching every rule through a single (shortest) parent. But sometimes whether a
target witnesses depends not on the path *to* it but on the **trailing context the chosen parent
supplies** *after* it. The class-scope `type_parameter`/`interface_class` family is the worked example:
the rule `class_scope := class_scope_type scope_resolution` is reached, the prelude declares `\foo` a
type parameter, and the sample parses — but reaching `class_scope` through a short data-declaration
carrier renders a `:: <id>` suffix, and on re-parse the *generic* `scoped_class_scope_identifier`
alternative (gated only to exclude a *class* head, not a type-parameter one) consumes that `<id>` and
shadows the per-family alternative. Reaching the very same `class_scope` through `class_new` instead
(`( class_scope )? new`) renders a `:: new` suffix — the generic alternative cannot consume the `new`
keyword, so the per-family alternative finally witnesses. The pass formalizes exactly this move: for a
residual target that parses-but-routes-elsewhere, it walks the default reach chain and, for each rule on
it, re-routes the plan to reach that rule through an **alternative parent** (from a reverse
rule-reference index), keeping the tail to the target intact, then re-checks the witness through the
real parser. Like every reach pass it runs last over only the still-unknown rules and unions only
re-parsing witnesses, so it is strictly additive and structurally inert for a fully-certified grammar
(empty residual ⇒ it never runs). It witnessed `known_unscoped_class_scope_type_parameter_identifier`
(via a carrier where `class_scope`'s trailing `::` has no class-identifier suffix), taking
SystemVerilog `UNKNOWN 33 → 32` (witness `1255 → 1256`), deterministic at seeds 0/7/42 with zero
newly-unknown rules. The two **call-form** cousins of that family (`…scoped_call…`) are an *honest
documented limit* of this generative approach: a `T::method()` call is ambiguous with a package-scoped
call at the expression level, *above* the rule whose alternative we want, so no carrier disambiguates
it — closing them would require tightening the grammar's generic alternative to also exclude
type-parameter and interface-class heads, a change deliberately deferred until a *real* parse failure
(not merely an unwitnessed fragment) justifies it.

A later hardening increment closed a **silent-degradation** hole in the armed prelude itself:
**fact-kind integrity with a depth-fresh retry**. The prelude's injected iteration must derive the
producer's *host* declaration — including the host's deep **mandatory siblings** (a
`property_declaration` must also complete its `property_spec`, a ~25-level expression chain) — but
the per-target witness depth budget only ever measured the *main* reach chain (the target's own
subtree at tier 1, the main chain's deepest mandatory off-path sibling at tier 2); the prelude's
own sub-path was invisible to both tiers. When the injected forced branch died on that hidden
depth (`… depth exceeded … while expanding rule 'number'`), the ordered-choice fallback quietly
rendered a *shallower sibling of the wrong declaration family* — a `sequence \foo ; … endsequence`
prelude feeding a `has_fact(property_name, …)` gate — and nothing checked that the iteration had
emitted the armed fact kind at all. A witness could then only succeed *accidentally*, when the
main path happened to render a self-emitting host; any reach re-routing away from such a host
(exactly what an unrelated grammar reshape did) turned the broken prelude into a lost witness.
The fix encodes the prelude's contract directly: after each injected iteration the generator
verifies a fact of the armed `(kind, family)` now exists in the generation-time store, and when it
does not (or the render failed outright) it rolls the discarded attempt back and retries **once**
under a depth-fresh budget measured from the injection depth *plus the sub-path's own deepest
mandatory off-path sibling* — the same tier-2 measure, applied to the sub-path the tiers cannot
see. A retry that still cannot produce the armed fact fails the attempt *loudly* instead of
handing the gate an unusable store. Count preludes and every injection that already emitted the
right fact are byte-identical by construction; on the canonical SystemVerilog run the headline
accounting is unchanged (`UNKNOWN=20`, union `UNKNOWN=1`, deterministic at seeds 0/7/42) while the
plannable pass witnesses one more target directly and eight fewer probe samples fail to re-parse —
and the property-gate witness now carries a *property* prelude, making it robust to reach
re-routing instead of dependent on an accidental self-emitting host.

### Closing the SVA operator layer, and what the residual is now

Past the reach passes, the last stretch of the SystemVerilog drive (`UNKNOWN 32 → 22`) was not
reach engineering at all — it was **grammar fidelity**: restoring IEEE-1800 syntax the extracted
grammar had dropped, and eliminating left-recursion the runtime cycle-breaker could not handle. Six
released slices closed it. Three restored missing brackets/braces the LRM mandates — the covergroup
trans-repeat forms `(1[*2])` / `(1[->2])` / `(1[=2])` (`UNKNOWN 32 → 31`, witnessing `repeat_range`),
the bins-set form `bins b = { … }` (`31 → 30`, witnessing `with_covergroup_expression`), and the
bounded-property operators `nexttime [3] a` / `s_always [1:2] a` (`30 → 29`); each was a *real* parse
defect (the bracketed forms had been rejected) as well as a cert witness. One restored a store-gated
identifier via a literal-count declaration prelude (`29 → 28`). The last two were the **SVA infix
left-recursion cascade**: the assertion `sequence_expr` and `property_expr` rules were flat,
directly-left-recursive ordered choices (`A := A op A`), which PGEN's then indirect-only LR eliminator
could not rewrite, so the runtime cycle-breaker blocked every infix operator and they *rejected* at the
operator. (⭐ Since `A2.5` the eliminator handles that shape natively — but this cascade stays, and
would still be the right answer: it encodes IEEE 1800 §16's **precedence**, which is language-specific
and no engine can infer. LR elimination fixes the *recursion*, not the *precedence*.) Restructured into an IEEE-1800 §16 Table 16-3 precedence cascade — sequence operators
`a intersect b` / `a within b` / `a ##1 b` (`28 → 26`) then property operators `a until b` /
`a s_until b` / `a iff b` (`26 → 22`) — the operators now parse with correct precedence, every
previously-parsing form keeps its exact AST carrier (the cascade is schema-preserving), and the four
property-only `until`-family keywords witness. (One honest limit is documented in place: the tight
prefix operators `not` / `nexttime` keep pre-fix dispatch precedence, so `not a until b` parses as
`not (a until b)` — unchanged behaviour, deferred until a consumer needs strict prefix precedence.)

At `UNKNOWN=22` the residual is now its irreducible core, and it is adjudicated in full. **Nineteen
are `no_path` non-defects** under the canonical `(systemverilog_file, sv_2017)` entry, re-derived by
re-running cert-coverage from other roots/profiles. **Six are genuine 1800-2023 features**
(`class_constructor_super_args`, the interface-class family, `union_modifier`): re-running under the
`sv_2023` profile drops all six from the residual — they genuinely *witness* there, so a
multi-profile union certifies them. **Eleven are rooted under the `library_text` / parseable-fragment
start symbols**: re-running from the `sv_multi_entry_root` umbrella collapses `no_path` from 19 to 8,
so the library/include/parseable-fragment subtree *gains reach* — but a tools-first re-derivation
(2026-06-29) shows it does **not** yet *witness* there (`UNKNOWN` stays 22 under that entry at seeds
0/7/42): `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` shows the witness pass produces trivial
`""`/`";"` samples that route through a sibling alternative, or malformed forced samples where the
`file_path_spec` rule emits its own literal name instead of a path token (an LRM-extraction artifact
since corrected to an LRM-faithful path lexeme — see *Library-cohort LRM fidelity* below). So these
eleven are genuine **reach/generation gaps**, not free accounting. Two more are LRM clause-number decomposition leaves
(`kw_n_29`/`kw_n_48`, synthetic, referenced, blessed — never deleted without LRM-proven absence); a
tools-first re-derivation (2026-06-29) found these two are **present in the `sv_2023` profile's rule
set and genuinely *witness* there** — the generation-IR dump (`--dump-gen-ast --grammar-profile
sv_2023`) shows both, and `sv_2023` cert-coverage leaves neither in its `UNKNOWN` set — correcting an
earlier note that they were profile-filtered out of `sv_2023`. So the multi-profile union certifies
these two as well. **Three are the genuine canonical-entry reach-gaps**: `context_member_method_call`
is a store-gated witness-reach gap (the `head.member[idx].method()` form *parses* with a declared
head — the generator just cannot yet synthesise the name-coupled declaration prelude its gate needs),
and the two `…scoped_call…` cousins are the `T::method()` expression-level ambiguity described just
above. So the headline number is honest in both directions: every one of the 22 is named, and none is
silently reclassified as "doesn't count".

That *endgame accounting* step is now a shipped, opt-in tool: `ast_pipeline
--report-certificate-coverage` accepts a repeatable `--cert-union-config <entry>[:<profile>]` flag
that unions the verified covered (`proof ∪ witness`) rule sets across the supported configs and prints
an extra `CERTIFICATE-COVERAGE-UNION:` line. The union is **sound by construction** — it credits a
rule only when some config *positively* covers it (a proof or a witness), never merely because a
profile leaves the rule out of its universe — so a rule that no config covers stays `UNKNOWN`.

A witness is only sound if the sample is **verified from the same start symbol it was generated
under**. Certificate witness verification is therefore **entry-aware**: every generated parser exposes
`parse_full_from(entry)` (and `parse_from(entry)`) — a full-input parse that begins at any rule, not
only the canonical entry — and each `--cert-union-config <entry>[:<profile>]` config verifies its
witnesses from *that* entry. This is what lets an *entry-relative* rule — one rooted under an alternate
LRM start symbol such as `library_text`, unreachable from `systemverilog_file` by design — be confirmed
at all: the same `library_text`-rooted sample the generator targets it with is now parsed back from
`library_text`, so the real parser can testify that the rule was exercised. (`parse_full_from`'s default
arm is the canonical entry, so a single-entry grammar is byte-identical to before this capability
existed.) For SystemVerilog,

```bash
ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0 \
  --cert-union-config systemverilog_file:sv_2023 \
  --cert-union-config sv_multi_entry_root:sv_2017 \
  --cert-union-config library_text:sv_2017 \
  --cert-union-config systemverilog_parseable_file:sv_2017
```

prints the byte-identical canonical line (`UNKNOWN=22`) plus `CERTIFICATE-COVERAGE-UNION: … UNKNOWN=3`
(deterministic at seeds 0/7/42). The `sv_2023` config certifies the 6 profile-relative rules **and**
the 2 extraction leaves (all eight witness there); the `library_text` / `systemverilog_parseable_file`
configs, now that verification honors the entry, certify all **11 entry-relative** library/include/
parseable-fragment rules. The union residual is therefore exactly the **3 genuine canonical-entry
reach-gaps** (`context_member_method_call` and the two `…scoped_call…` cousins). Reaching a literal
`UNKNOWN=0` for SystemVerilog then needs only that last reach-gap work.

### Library-cohort LRM fidelity

Once entry-aware verification made the library cohort *witnessable*, one of those witnesses was still
honest only by accident: the `file_path_spec` lexeme was a **literal-keyword extraction artifact**
(`kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/`) that matched only the literal word
`file_path_spec`, so the `include`/`library` productions accepted that one token and **rejected every
real file path**. IEEE 1800-2017 §33.3.1 / Annex A.1.1 define `file_path_spec` as a file-system *path*
token (absolute or relative, wildcards `?`/`*`/`...`, `/` separators) — not a keyword. The rule is
therefore corrected (release `1.0.150`, ledger `SV-0012`) to an LRM-faithful path lexeme
`/[A-Za-z0-9_.\/?*~$+]+/`, keeping the rule's name and arity (so the typed carrier and the AST-dump
schema are byte-identical). The `include`/`library` cohort now accepts real §33 paths — `include
../rtl/cpu.v;`, `library mylib /path/to/*.sv;`, `library rtl ./src/*.sv, ./pkg/*.sv -incdir ./inc;` —
and the cohort's cert witnesses now exercise a genuine path token rather than the literal name. (`-` is
deliberately excluded from the class so the `-incdir` flag stays separable; a hyphenated filename is the
one documented limitation.) The change is confined to the `library_text` analysis entry — the
`systemverilog_file` embedding entry never reaches the library cohort — so the canonical cert
(`UNKNOWN=22`) and the 4-config union (`UNKNOWN=3`) are byte-identical across seeds 0/7/42.

### Closing two of the three reach-gaps: the class-scoped-call cousins

The first two of those three canonical reach-gaps then closed with a single grammar-gate. Both
`known_unscoped_class_scoped_call_interface_class_identifier` and
`known_unscoped_class_scoped_call_type_parameter_identifier` are head branches of
`class_scoped_call_prefix` — the prefix of a class-scoped subroutine call like `IF::method()`. The
certificate probe reported them `parsed=true witnessed_target=false` (the sample parses, but the cousin
branch is never the one that fires), and a scoped `--trace-rules class_scoped_call_prefix` pinned why:
the *first* head alternative, `scoped_class_scoped_call_prefix_identifier` (the `<pkg>::<class>` form),
was gated only by `lacks_class`. A head declared `interface_class` or `type_parameter` is not a class,
so that gate passed — and because `package_identifier` accepts any identifier, the branch
**longest-matched** `IF::method` as `<pkg>::<class>` (14 bytes) over the correct cousin's `IF` (5
bytes). It then failed the mandatory trailing `::`, and the call fell through to the ungated
`package_scope` call route, emitting a `package_scope`/`tf` shape for what is semantically a
class-scoped call. The `class` head never had this problem, because `lacks_class` correctly rejects the
`<pkg>::<class>` branch for it — which is exactly why only the *other two* families were `UNKNOWN`.

The fix is the same store-consultation tightening used elsewhere: two AND-stacked
`lacks_fact_attribute_equals` predicates (`interface_class`, `type_parameter`) now join `lacks_class` on
that branch, so the `<pkg>::<class>` form fires only for a genuine package scope and an
interface-class/type-parameter head falls through to its dedicated cousin — routing the call through
`class_scoped_tf_call` (the LRM-correct shape). Both cousins witness, the canonical residual drops
`UNKNOWN 22 → 20` and the 4-config union `3 → 1` (deterministic at seeds 0/7/42, `spf=0`, every other
grammar byte-identical), and the only remaining reach-gap is `context_member_method_call` (a store-gated
declaration-hosting carrier). Because the affected calls now emit `class_scoped_tf` instead of
`package_scope`/`tf`, this is an AST-shape *correction* (the strings still parse; no new node kinds;
schema unchanged) — released as `1.0.151`, ledger `SV-0013`. SystemVerilog is now a single rule from a
fully-certified multi-config union.

### Closing the last reach-gap: the structured-witness composition pass

The final rule, `context_member_method_call`, had resisted two implement-and-revert attempts and the
entire five-pass witness apparatus — and the reason turned out to be *structural*, not a missing
mechanism. The rule is store-gated (`has_fact(variable_binding, <head>)`): a witness must
simultaneously (1) contain a *prior typed declaration* that routes through the grammar's one
`variable_binding`-emitting producer, (2) render the chain *head* as exactly the declared name, and
(3) render the target's own distinguishing chain structure. The generator already owned machinery
for **each** condition — the declare-then-use name prelude, the gated-consumer name replay, and
target-own structure forcing — but each lived in a *different* witness pass, so no single generated
sample ever satisfied all three at once. A fresh parse matrix (hand-fed variants through the real
parser) confirmed the coupling empirically: the natural attribute carrier witnesses the rule the
moment a typed declaration precedes it and the head matches; an untyped or wrong-producer
(`localparam`) prelude parses but never witnesses.

The close is therefore a **composition pass**, `generate_structured_witnesses` — a final
residual-only pass that assembles the proven pieces into ONE reach plan: the name prelude is armed
with a pass-scoped *producer admission* extension (the producer's `@emit_fact` names a dotted
sub-payload rather than its whole render, so the emitted name is resolved to the producer's leading
rendered token — kept **with its lexical terminator**, because replaying a trimmed escaped
identifier immediately before `.` would fuse the two into one token, which was precisely the one
failure the first probe run exhibited); the prelude's sub-derivation *forces the typed branch* of
any ordered choice that has a render-empty escape (`data_type_or_implicit`), so the declaration is
fact-emitting (`int foo;`-typed, never `foo;`-untyped); the gated consumer, being multi-token, pins
only its **head leaf** to the declared name (single-token consumers keep the existing whole-render
replay unchanged); and the target-own structure directives ride in the same plan. The real parser
remains the sole witness judge. Everything new is scoped to the pass itself, which runs only over
rules still unwitnessed after every earlier pass — so a fully-certified grammar (empty residual)
never executes it, and every earlier pass keeps a byte-identical stream; the six certified
cert-lane grammars were additionally proven byte-inert by a baseline↔candidate byte-compare of
their full cert reports. The witness landed on the first composed probe: canonical
`witness 1321 → 1322`, canonical `UNKNOWN 12 → 11`, union `UNKNOWN 1 → 0`, deterministic at seeds
0/7/42 with `spf=0` — **the SystemVerilog multi-config union has no `UNKNOWN` left**.

### The recognized SV certificate-coverage accounting basis

Because that union is sound and deterministic, it is SystemVerilog's **recognized
`fully_certified`-accounting basis** — the honest internal trust figure for "how close is SV to fully
certified." A number is only as trustworthy as the oracle that re-derives it, so the recognized basis
is not left as prose: it is locked by a re-runnable, deterministic gate,
`make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate`. The gate runs the
`--report-certificate-coverage` + 4-config `--cert-union-config` invocation *for each* of seeds
0/7/42 and asserts, against a tracked contract
(`rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`), the canonical
accounting (`total=1343 proof=10 witness=1322 UNKNOWN=11`), the union accounting
(`witness=1333 UNKNOWN=0`), the exact union residual rule set (`[]` — empty,
compared order-insensitively), `sample_parse_failures=0`, and that all three seeds agree
byte-for-byte. (The count pins have been re-baselined as accounted rules were added, each time
preserving the load-bearing UNION invariant — union `UNKNOWN=1`, the same single residual rule:
2026-07-02, `VERILOG-2005-PROFILE.5` — the `SV-AST-SHAPE-FIDELITY` named-lift
campaign plus the `verilog_2005`-profile named-lifts, `total 1304→1324`; then through the
`verilog_2005` boundary-leak lifts and the `SV-DOLLAR-LRM-FIDELITY` LRM-fidelity campaign
(2026-07-02→03) — 17 more accounted rules, `total 1324→1341`, canonical witness `1302→1319`, union
witness `1321→1338`; then the `SV-0032`/`SV-0034` void/dimension gates re-pinned `total 1341→1343`;
then `VERILOG-2005-PROFILE.6.7` (2026-07-05, the per-profile `proof` promotion) moved canonical
`proof 2→10` and canonical `UNKNOWN 20→12` — the 8 `sv_2017`-profile-entry-unreachable
SystemVerilog-only rules are now PROVED, not UNKNOWN — while the union stays invariant (proof gathering
is canonical-only, so `union_proof == canonical_proof == 10`; the 8 rules are proof-under-`sv_2017` +
witness-under-`sv_2023`, so union witness `1338→1332` and union `UNKNOWN` stays `1`); and finally
`STRUCTURED-WITNESS-SYNTH.3/.4` (2026-07-22, the structured-witness composition pass above) closed
the last reach-gap — canonical witness `1321→1322` / `UNKNOWN 12→11`, union witness `1332→1333` /
`UNKNOWN 1→0`, residual `[]`.) So the recognized figure cannot silently drift, and the final union
`1 → 0` flip was itself gated exactly as promised: the contract is re-baselined to
`expected_union_unknown=0` in the same wave as the capability landing, and the gate re-derives it
across seeds 0/7/42. **The `done_rule` has fired: SystemVerilog is recognized `fully_certified` on
the sound multi-config union basis** — every one of the 1,343 accounted rules is either PROVED
(profile-entry-unreachable under the canonical profile, independently re-derived) or WITNESSED
through the real parser in at least one declared configuration. The canonical single-config
accounting (`UNKNOWN=11`: the 11 entry-relative library/include/parseable-fragment rules, each
union-covered under its own entry) is recorded alongside, as always. (The gate adds a proof
surface only — it changes no grammar, parser, generator, or generated artifact; the cert numbers
are read-only measurements.)

### The corpus rule-coverage instrument (the external mirror of the certificate axis)

The certificate answers "can the *generator* reach and witness every rule?". The corpus mandate
(`SV-CORPUS-GRAD.7`) asks the mirror question from the outside: **does the vendored external
corpus actually *exercise* every rule the profile can reach?** Two parser-agnostic surfaces answer
it, both deterministic and diffable:

- `ast_pipeline <grammar>.ebnf --dump-rule-profiles out.json` — the **denominator**: the full rule
  inventory with, per rule, its declared `@profiles` set and its *derived* per-profile
  satisfiability (the same transitive computation the profile-orphan lint gates on). For
  systemverilog: 1,466 rules — satisfiable under `sv_2017` = 1,343, `sv_2023` = 1,362,
  `verilog_2005` = 1,115. The `sv_2017`/`verilog_2005` figures equal the certificate-coverage
  canonical totals — the two instruments cross-confirm each other's universe.
- `stimuli/sv/corpus_rule_coverage.py` — the **numerator**: for every external-corpus file the
  parser *accepts*, the generated parser's transactional coverage testimony (committed rules only —
  sound under PEG backtracking; failed parses contribute nothing) is unioned per profile and
  diffed against the inventory. Every rule lands in exactly one class: `covered`, **`GAP`**
  (satisfiable under the profile, fired by zero corpus files — the acquisition/crafting worklist),
  or `na_profile` (measured under its own profile's run instead). The tracked report
  (`stimuli/sv/characterization/rule_coverage_<profile>.md` + per-rule TSV) also names
  thin-coverage rules (≤3 files) and any file excluded with cause (the transactional coverage
  stack is >100× slower than a plain parse on pathological-backtracking inputs, so such files are
  timeout-excluded *by name*, never silently).

A **structural companion** lens (`SV-CORPUS-GRAD.7b`, `stimuli/sv/corpus_clause_coverage.py`) maps
the *keyed* corpus onto the LRM's own clause/chapter structure instead of onto grammar rules. Only
three inputs carry clause metadata — sv-tests `:tags:` (edition 1800-2017), ispras `ieee-1800-2012/`
dotted filenames, and ispras `ieee-1364-2005/` `test_*` filenames (Verilog) — so it speaks for a
~14% keyed slice of the vendored universe and is deliberately **not** a competing coverage
percentage. It reads only the committed adjudication manifests (no parser run) and answers two
questions the rule lens cannot: *which LRM chapters does the keyed corpus deliberately target* (every
parse-bearing chapter 5–35 currently has ≥1 keyed case — no chapter-level gap), and *how dense is the
negative axis per chapter* (the thinnest surface in the whole corpus — only 5 of 31 parse-bearing
chapters carry any clause-keyed `must_reject`; the bulk of the corpus's negatives are unkeyed ivtest
`CE` rows and sv2v bad-goldens that cannot be attributed to a clause without per-file adjudication).
Editions are kept distinct (clause *numbers* are not comparable across the 1364/1800 boundary); the
1364-2005 lane's verdicts are taken from the `verilog_2005` manifest where those files are
adjudicated. The two lenses are complementary — a rule can be covered while its chapter has no keyed
negative, and vice versa — so `.9` closes gaps surfaced by both.

Graduation (`SV-CORPUS-GRAD.5`) requires **both** zero unexplained divergences *and* 100% measured
coverage of the parseable surface (uncovered rules driven to a corpus case or an N/A-with-cause).

### The adjudication manifest — what is allowed to count as a defect signal

A raw corpus failure is not a parser bug. Roughly half of a vendored suite is *supposed* to fail:
intentionally-invalid negative tests, fragments that only compile as part of a multi-file unit, and
files whose text is not parser input at all until a preprocessing stage has run. The
**adjudication manifest** (`stimuli/sv/adjudicate_external_corpus.py` → the tracked
`stimuli/sv/characterization/adjudication_manifest.tsv`) is what separates those from the defect
signal, one row per file, deterministic and diffable.

Each row carries an **expected verdict** derived *only* from suite metadata, upstream driver
conventions, or an LRM-grounded pinned ruling — **never from what the parser currently does**. That
direction is the whole point: an expectation read off the parser can only ever confirm it.

| expected verdict | meaning |
|---|---|
| `must_accept` | valid at parse level (a file that should fail *later* — elaboration, lint — still parses) |
| `must_reject` | the text has **no derivation** in the LRM's grammar (Annex A, or the clause-5 lexical rules) |
| `chained_only` | adjudicable only with include/library chaining |
| `out_of_scope_with_cause` | owned by another lane, which the row names |

⭐ **`must_reject` is decided by the missing derivation, not by the upstream author's intent.** The
common case is a suite's own negative test, but the majority of the pinned rulings are the other
shape: text a *vendor tolerates* that the standard cannot derive. The taxonomy said "intentional
invalidity" for a long while, and by the time `SV-CORPUS-GRAD.3.23` looked at it, roughly thirty
pins contradicted their own definition — a hole waiting to be argued from, since a class defined by
intent has no answer for a file whose author intended it to be valid. It is now defined by the
grammar, which is the test that was always actually being applied.

Comparing expectation against the observed verdict yields the **adjudication class**. Only two of
them are defect signal — `divergence:unexplained_rejects_valid` and
`divergence:unexplained_accepts_invalid` — and their sum is the graduation bar. Everything else is
either a match, a lane deferral, or a **named** explanation: `explained_svpp_include`,
`explained_svpp_macro_use`, `explained_svpp_conditional`, `explained_svpp_protected_envelope`,
`explained_timeout`.

⭐ **An allowlist that conflates two spec categories converts a preprocessing dependency into a
parser defect — silently, and in the direction that makes the parser look worse.** The dependency
detector decides "does this file need the preprocessor?" by walking its `` ` ``-prefixed names and
reporting macro expansion for any name that is *not* a known compiler directive. Its allowlist held
`` `__FILE__ `` and `` `__LINE__ ``. But IEEE 1800-2017 separates exactly what the allowlist merged:
a **compiler directive** steers the compilation and leaves the surrounding text parseable as
written, whereas §22.13 defines those two names as predefined text **macros** that *expand* —
"`__FILE__ expands to the name of the current input file, in the form of a string literal" — so
`$display(`__FILE__);` is not parseable text until expansion happens. 33 files whose only
preprocessing dependency was one of them were therefore counted as parser defects
(`SV-CORPUS-GRAD.3.13`; a second, rarer case joined them — IEEE 1800-2017 §34's protected
envelopes, whose `key_block`/`data_block` payload is encoded bytes the decrypting tool replaces
with source text before compilation, now the `explained_svpp_protected_envelope` class).

Two rules keep that correction from becoming the thing it looks like — *lowering the bar by
relabelling*:

- **The criterion must be spec-derived and mechanical**, never per-row judgement. Here it is one
  clause cite and one constant. A reclassification argued case-by-case would corrupt the bar it is
  measured against, and the same leaf's four-verdict split shows why the discipline pays: of the 37
  rows in that ch22 family, 27 were the macro correction, **8 were a genuine grammar gap left in the
  bar** (in-scope compiler directives, which the SV grammar accepts only at file top level), and 2
  were the §34 case. A wholesale "it's all directives" relabel would have hidden a real defect.
- **It is reported as an adjudication correction, never as burn-down yield.** The tree states the
  before→after in both directions and separately from any grammar work.

⚠️ And an explained label is a statement about the *input*, not a clean bill of health for the
parser: it says this file is not honest parser input. Of the 33 rows above, 29 were stuck exactly at
the macro; the other 4 used one elsewhere in the file and had stopped at an unrelated construct. A
correct relabel still buries those stuck points, so they are re-routed as crafted minimal cases
where the construct — not the vendored file — is the unit.

#### Auditing the explanation itself — the asymmetry that makes it worth doing

That 29-versus-4 split is the whole problem in miniature, and `SV-CORPUS-GRAD.12` asked it of the
entire population: **1 459 rows carried an `explained_svpp_*` label; how many had actually been
checked?** The answer was one class of four rows, ever — and when a previous leaf opened the
neighbouring `explained_timeout` class, all four of *those* turned out to be a parser defect
allocating 12 GB on valid RTL, wearing a resource-limit mask.

The two possible errors are not priced the same. **A wrong `unexplained` verdict costs a wasted
investigation; a wrong `explained` verdict ships a defect**, because the row leaves the burn-down by
construction and nothing looks at it again. Only one of them was being checked.

The label is decided by a whole-file existence test — *does this file contain a `` `include ``, a
macro use, an `` `ifdef ``, or a protected envelope anywhere?* — and is then read as the far
stronger causal claim *this file fails because it needs the preprocessor*. Those are different
statements, and the gap between them is measurable: a genuinely preprocessor-blocked parse must die
**at or after** the first byte the preprocessor can alter, because a preprocessor's output is
byte-identical to its input up to that point.

Over a full census of all 1 459 rows, the two statements **agree 80.2 % of the time, disagree
provably for 26 rows, and are unresolvable by position for the remaining 18 %**. The 26 are parser
findings that had been filed as preprocessor dependencies — among them a file that stops on
`` `default_nettype `` inside a module body, which the preprocessor hands through untouched, and one
that stops on a line-continued string literal 8 000 bytes before the file's first directive.

Two details of that audit are worth carrying to any similar instrument:

- **Not every directive moves a byte.** Expansion substitutes macros, resolves conditionals and
  inlines `` `include ``; it passes `` `timescale ``, `` `default_nettype `` and their neighbours
  straight through. A test that treats all `` ` ``-tokens as the start of the preprocessor's reach
  both misses defects and manufactures them.
- **A failure position names a region, not a token.** The reported furthest position is the deepest
  byte any branch *consumed*, so a parse stopped *on* a directive reports the byte just before the
  whitespace in front of it. The first cut of this audit compared the two directly and reported 504
  misclassified rows — a 34.5 % rate that was entirely a six-byte layout gap. Every row spot-checked
  in it was healthy. Resolving the gap first took the finding from 504 to 26.

The honest form of the resulting number is a floor and a ceiling, not a point: the defect population
is **at least** 26 rows larger than the burn-down claimed, and at most 289 larger. Quoting the
ceiling as a defect count would be the same error as quoting the original figure as settled.

**The label is now decided positionally.** The corpus runner captures the probe's failure position
into a sidecar — it had been discarding the parser's stderr, which is the only reason the question
could not be asked before — and the adjudicator awards an `explained_svpp_*` label only when the
parse stops somewhere expansion can actually reach. The 26 rows returned to the defect burn-down,
which rose from 293 to 319. **A burn-down going *up* is the correct outcome here**: those rows were
always defects and were merely wearing the wrong label, so the number did not get worse — it got
true. The `verilog_2005` lane was measured the same way and came back clean, with zero rows moved.

It has since come down by one, to **318**, and the reason is worth reading beside the paragraph
above because it is the same lesson from the other end. One of those rows —
`sv2v/test/lex/latin1.sv` — was never a parser defect at all: the file is ISO-8859-1, and PGEN's
readers called `std::fs::read_to_string`, which refuses a non-UTF-8 byte stream outright. **The
file was refused; no construct was ever rejected**, and the burn-down counted the refusal as a
grammar gap. Twelve more corpus files (each carrying a single `0xA9` — the `©` in a copyright
comment) were failing the same way without being counted, because their expected verdict was
`chained_only`. `SV-CORPUS-GRAD.12c.1` gave PGEN one shared source-text decoder, and those twelve
now bank a real parse position instead of nothing at all. See
[Embedding and Downstream Integration § Reading source files](embedding-and-downstream-integration.md#reading-source-files--pgensource_text)
for what a host embedding the parser has to do about this.

One design detail is worth stating because getting it wrong would have been invisible. The
whole-file test feeds *two* decisions, not one. On the negative-test path it answers "could this
file's intended syntax error be hidden until after preprocessing?" — a question that really is about
the whole file, and whose row may have no failure position at all, because it can be the row that
wrongly *accepts*. Only the positive-test path became positional; the existence test stayed exactly
as it was for the other caller. A predicate serving two questions must be changed for one of them at
a time.

#### The denominator — what fraction of the corpus was asked a question it could answer?

Everything above counts **divergences**. That answers *how many defects do we know about* and is
silent on the question a signoff claim actually rests on: *how much of the corpus was asked a question
it could answer at all?* A 302-row defect bar over a 46 %-adjudicated corpus is not the same claim as
the same bar over a fully adjudicated one, and only the second supports shipping. (302 is the live
figure — the tuple below is the anchor a doctrine re-derives; the 318 in the paragraph above is that
era's number, kept because the story is what makes the point.)

Measured (`stimuli/sv/corpus_verdict_coverage.py`, `SV-CORPUS-GRAD.13`): of 16 336 rows, **46.3 % are
adjudicated**, 15.1 % are **routed** to the `verilog_2005` lane's own manifest — and **38.7 %
(6 321 rows) carry no verdict at all**, overwhelmingly the `deferred:chained_only` class, which is the
most realistic industry RTL in the corpus. ⛔ **Unknown and clean are different words.** A row with no
verdict is not evidence of correctness; it is not evidence of anything.

But 38.7 % is a headline, not a plan, because it fuses three strata of opposite worth (`.13a`):

| stratum | rows | % | what it is worth |
|---|---:|---:|---|
| **one-sided positive**, unit-shaped | 1 826 | 11.2 % | the parse consumed the **whole file** standalone ⇒ no *rejects-valid* defect hides here. Silent on accepts-invalid, and still not a verdict — there is no expectation to compare against |
| ⚠️ **one-sided, fragment-shaped** | 103 | 0.6 % | an `.svh` include payload or a `// verilog_syntax:` excerpt — **not** a legal compilation unit, so accepting it is not testimony *for* the parser; the accept may itself be the over-acceptance |
| ⛔ **dark** | **4 392** | **26.9 %** | the parse failed and the deferral is why nobody looked |

The number to plan against is therefore **4 392**, and reaching it moved **no row** into the
adjudicated bucket — the manifest already recorded what the parser did on every file, and that
observation was simply unused.

> ⭐ **This stratum is where a deferred row's progress shows up, and that is the design, not a
> gap** (`SV-CORPUS-GRAD.13h`, 2026-08-14). When the indirect-LR admission flip landed, twelve
> corpus files went `fail → pass`; six of them are `deferred:chained_only`, so the **defect bar
> cannot see them** — a bar counts *divergences*, a divergence needs an *expectation*, and a
> deferred row has none. The information is not lost: those six left **dark** (4 398 → 4 392),
> which is the fourth element of the live tuple below. ⛔ And two-thirds of that "invisible
> progress" is not progress on the project's own terms — four of the six are `.svh` include
> payloads, i.e. **fragment-shaped**, where an accept *may itself be the over-acceptance*. Of the
> remaining two, the dark-worklist instrument returns the verdict **`PARSES-BARE`** — *"parses with
> NO transformation at all — contradicts the corpus row"* — so what those rows expose is a
> **mislabelled deferral**, owned by `.13c`/`.13d`, not an under-reporting bar.

> ⭐ **Live verdict-coverage tuple — `adjudicated/routed/no-verdict/dark/axis-2-bar` =
> `7556/2459/6321/4393/282`.** (`SV-CORPUS-GRAD.13c.2s` moved the bar **288 → 282** — the largest
> single-slice drop in this campaign — by repairing a **greedy `( X )*` standing in front of an
> OPTIONAL `X`-shaped tail** at four sites. The star's body is nullable after its separator, so it
> ate the separator the tail needed: a system task/function call could never reach the clocking-event
> argument of IEEE 1800-2017 A.8.2, and the `let`/`property`/`sequence` argument lists of A.2.10 could
> never reach a NAMED argument after a positional one. **Six third-party corpus files flip
> fail → pass and zero move the other way**, every one attributable by an isolating arm to the
> system-task site alone; `unexplained_rejects_valid` 267 → 261; the ADJUDICATED/ROUTED/NO-VERDICT/DARK
> split entirely unmoved; and the `verilog_2005` lane **byte-identical end to end**, 0 of 2 459 rows
> moved. ⭐ Its sibling `SV-0060` fixed the same mechanism where the starved element was MANDATORY —
> there the rule matched nothing and **no verdict moved at all**, so only an AST-arm check could see
> it. Same defect, opposite observability. The previous entry:
> `SV-CORPUS-GRAD.13c.2j` moved the bar **289 → 288** by restoring
> IEEE 1800 A.8.4's `class_scope` branch to the method-call receiver, so a CLASS-SCOPED name is a
> legal receiver: `y = p::base::m.g()` had no derivation while `y = p::base::m;` — the same name as a
> value — parsed. `match` 5 831 → 5 832, `unexplained_rejects_valid` 268 → 267, `accepts-invalid`
> byte-identical at 21 (the same 21 files, set-compared), the ADJUDICATED/ROUTED/NO-VERDICT/DARK
> split **entirely unmoved**, and the `verilog_2005` lane **byte-identical end to end** because
> `class_scope` is profile-gated to the SV dialects. ⭐ Annex A writes that prefix ONCE and PGEN
> renders it FOUR times — one named rule plus three hand-spelled inline copies, which exist because
> the receiver rules are what cut the cycle `primary → call_primary → method_call → primary`. The
> named rendering gained `class_scope` two months ago for a different measured defect; none of the
> three copies did, and nothing in the repository compared them. ⛔⛔ **The fix is ONE of those three
> sites, and the reason is a cost measurement.** Restoring the branch at all three was refused by
> `PARSE-COST-RATCHET` — `entries` +0.47 %, `memo_hits` +1.05 % — and a per-site attribution showed
> `split_hierarchical_callable_receiver` alone carries the whole accept-set gain, with the corpus
> **byte-identical across 16 336 files** between the one-site and three-site variants. Costs are
> rejected here, not traded, so the two inert copies stay shorter as *pinned* divergences that a
> new instrument re-derives on every run. ⭐ The residual price of the one site that matters is
> published rather than absorbed: `entries` +0.14 % and `memo_hits` +0.31 %, **identical deltas of
> 585 252 with `committed` flat** — every added rule entry was served from the memo table, so the
> change asked 585 252 more cached questions and did zero new parsing work. That is the converse of
> this doctrine's founding lesson: a deterministic counter cannot see a per-entry cost rise, and it
> cannot see that a rise is pure cache traffic either. The one corpus row that moved carries the
> construct verbatim — verible's `nested_member_access.sv`, whose last line is
> `nested_class0::handle1::handle2.nested_function()`. The step before it,
> `SV-CORPUS-GRAD.13c.2c`, moved the bar **300 → 289** — a larger move
> than any other recorded on this line, and it was **one grammar token**. The member loop of
> `split_hierarchical_callable_receiver` was guarded by `!callable_method_call_body`, a negative
> lookahead that **can never pass on an identifier**: IEEE 1800 A.8.2 makes
> `array_manipulation_call`'s parens OPTIONAL, so a bare member name already satisfies
> `callable_method_call_body`. The loop therefore ran ZERO iterations for its whole life and the
> receiver collapsed to its first component, which is why `a.b.g()` parsed and `a.b[0].g()` did
> not. `match` 5 821 → 5 831, `unexplained_rejects_valid` 279 → 268, `accepts-invalid` byte-identical
> at 21 — the same 21 files, set-compared — and the ADJUDICATED/ROUTED/NO-VERDICT/DARK split
> **entirely unmoved**: eleven fewer known defects over the same 46.3 % of the corpus. ⛔ Eleven rows
> moved and only TEN crossed `fail → pass`; `verilator/test_regress/t/t_func_dotted.v` still FAILS
> and reclassified `unexplained_rejects_valid → explained_svpp_macro_use` because its parse now runs
> past the dotted call and dies in a macro window — the third consecutive slice in which a pass/fail
> join under-counts its own result, which is why the join is over the MANIFEST. ⭐⭐ It is also a
> **THREE-profile** fix: nothing on the path carries an `@profiles` gate, so the same defect had been
> rejecting IEEE 1364-2005 §12.4 hierarchical names in plain Verilog — `top.u1[0].t;` and
> `y = top.u1[0].f(1);` are now pinned as reproducers on all three profiles. The step before it,
> `SV-CORPUS-GRAD.13c.2f` slice 4, moved the bar **302 → 300** by
> making IEEE 1800 A.7.5's two `PATHPULSE$` tokens matchable: `match` 5 820 → 5 821,
> `unexplained_rejects_valid` 281 → 279, `accepts-invalid` unchanged at 21, and the
> ADJUDICATED/ROUTED/NO-VERDICT/DARK split **entirely unmoved** — two fewer known defects over the
> same 46.3 % of the corpus. ⛔ Two rows left `unexplained_rejects_valid`, and only ONE of them
> crossed `fail → pass`: `ispras-sv-tests/ieee-1800-2012/30/30.07.01_01.sv` (clause 30.7.1's own
> example) became `match`, while `verilator/test_regress/t/t_specparam.v` still FAILS and
> reclassified to `explained_svpp_conditional` because its parse now runs past every `PATHPULSE$`
> specparam and dies on a `` `ifdef `` at byte 1170. That second row is invisible to a delta joined
> on pass/fail — the same blindness `.13h` documented one slice earlier, reproduced, which is why
> the join is over the MANIFEST. The `verilog_2005` lane moved with it: `match` 2 186 → 2 187,
> `unexplained_rejects_valid` 54 → 53, `accepts-invalid` unchanged at 14. The step before it,
> `SV-CORPUS-GRAD.13h`, moved the bar **309 → 302** and `dark`
> **4 398 → 4 392** by promoting the corpus oracle after the `ENGINE-UNIVERSAL-SERVICES.17`
> slice-9 admission flip: `match` 5 814 → 5 820, `unexplained_rejects_valid` 288 → 281,
> `accepts-invalid` byte-identical at 21, and the ADJUDICATED/ROUTED/NO-VERDICT split unmoved —
> **fewer known defects over the same 46.3 % of the corpus, which is a smaller claim than "the bar
> fell"**. ⛔ Seven rows left `unexplained_rejects_valid`, not six: six became `match`, and one —
> `sv2v/test/core/string_byte_order.sv` — still FAILS but now fails on its `` `include `` instead of
> on the size cast ahead of it, so it reclassified to `explained_svpp_include`. A delta computed by
> joining on pass/fail cannot see that row, which is why the routed estimate said 303. The step
> before it, `SV-CORPUS-GRAD.13c.2e`, moved the bar **313 → 309**: restoring
> `select_condition`'s literal `intersect { … }` braces turned four more clause-19 covergroup files
> from `unexplained_rejects_valid` into `match`, again with the accepts-invalid set byte-identical.
> The step before it, `GRAMMAR-WELLFORMED.A2.5`, moved the bar **318 → 313** by reviving
> `select_expression`'s dead `&&` / `||` / `with ( … )` alternatives. ⚠️ Read the two together: the
> A2.5 repro needed PARENTHESES to prove anything, because the un-braced `intersect` list was still
> swallowing whatever followed it — the same clause, defective twice, and the second defect was
> masking the evidence for the first. **The denominator did not move in either step**, which is the
> point of publishing them together: four fewer known defects over the same 46.3 % of the corpus is
> a smaller claim than "the bar fell".) The
> `SV-CORPUS-DENOMINATOR` doctrine
> (`scripts/check_sv_corpus_denominator.sh`) re-derives all five numbers from the tracked
> adjudication manifest on every commit and fails if this line disagrees or if the census artifact
> has gone stale — so **the bar can never again be published without its denominator beside it**.
> The instrument's own output lives at
> `docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/coverage.md`.

Inside the dark half, a deferral can sometimes be refuted by the input's own bytes. A file containing
no `` ` `` byte anywhere cannot be altered by macro expansion, conditional resolution or
`` `include `` inlining, so its parse fails identically after chaining: **530 `chained_only` rows are
in that state.** What that refutes is the *textual* reading of the deferral — and refuting a cause is
not naming one, so the next step is the toolbox. All three rows probed die on a **user-defined type
name declared in a sibling file** (`wire csrng_req_t cmd_req;`, with the trace reporting
`🚫 checked_type_identifier rejected by post predicate 'has_fact [type_name, …]'`). In a store-gated
parser a `typedef` elsewhere supplies the fact a predicate requires, so the dependency is real — but
it is a **cross-file fact** dependency, not a text one, and the two are satisfied by different
capabilities: parsing a file list into one fact store, versus running a preprocessor. Those 530 rows
need the first and not the second; the 3 628 dark rows that *do* carry directives need the second.

Applying that falsification to the whole dark `chained_only` population — not just its
directive-free corner — sizes it properly (`SV-CORPUS-GRAD.13c`,
`stimuli/sv/audit_dark_chained_only.py`, 46 s for 4 158 rows including 2 285 traced parses):
**2 285 of 4 158 rows (55 %) have their textual deferral refuted**, and the parser's own trace then
sorts them:

| bucket | rows | what it is |
|---|---:|---|
| ⛔ not SystemVerilog source | 346 | `$readmemh` memory images carrying a `.v` extension — they can never parse and testify to nothing |
| fact-gated, cross-file | **2 057** | the parser demanded a `type_name` fact for a name **a named sibling file declares** |
| cross-library | 171 | the name lives in another vendored suite (a test using UVM) — the unit is incomplete as the corpus holds it |
| ⭐ candidate defects | **57** | a fact demanded that nothing anywhere declares (25), no fact demanded at the failure at all (24), or the name declared **in that very file** (8) |
| undecidable from text | 1 527 | the failure sits where an earlier expansion could shift the stream |

⇒ **half the dark population is blocked by cross-file *facts*, not by text** — and that decides
sequencing: those 2 057 rows unblock on parsing a file list into one fact store, which needs no
preprocessor at all, while 1 527 genuinely need expansion. Naming the two capabilities separately
was only possible once the population was measured rather than argued about.

⚠️ One methodological note is worth more than the numbers. That worklist read **1 902 → 1 933 → 53 →
1 885 → 335 → 228 → 57** across successive corrections, every intermediate answer defensible and
wrong: a text classifier that missed packed dimensions; a trace pattern that matched `has_fact` but
not the `fact_attribute_equals` spelling gating `class X extends BASE` (1 203 rows, while the trace
named the base class one line above); a declaration test that mistook a type *used inside* a
`typedef struct` body for the typedef's own name. Each was caught the same way — by probing one row
of a bucket before believing the bucket. **A number that moves by 1 900 on a one-line edit is a
heuristic, not a census.**

#### What a 57-row candidate list is actually worth

A candidate list is a hypothesis, and the only way to price one is to adjudicate every row.
`SV-CORPUS-GRAD.13c.2` did that (`stimuli/sv/adjudicate_dark_worklist.py`, 61 s for 57 rows, and
byte-identical at two different parallelism settings), by the same rule the census used: **a row
earns its verdict by being parsed**, never by being relabelled. Each verdict below is a real parse of
a *named, published* transformation of the row's own bytes — a declaration prelude, a
compilation-unit wrapper, or neither — and the prelude is minimized afterwards, so what gets
published is the smallest set that explains the row.

| verdict | rows | reading |
|---|---:|---|
| macro-blocked | 26 | once the missing declarations are supplied, the parse dies **on** a named undefined macro (`` `uvm_component_param_utils ``, `` `DRIVE_CLK ``, `` `ifdef ``) — expansion owns the remainder |
| cross-file facts | 9 | declaring the demanded names makes the file parse; `uvm_policies.svh` needs exactly one, `class uvm_object` |
| ⭐ invalid SystemVerilog | 8 | the text is not legal SV and rejecting it is **correct** |
| ⭐⭐ **parser defects** | **7** | valid SV that is rejected — **three distinct constructs** |
| not source text | 4 | a tool command line and three plain-text fixtures wearing `.sv`/`.svh` |
| fragments | 3 | include payloads that parse only inside a `module`, or an enum-member list |

Two results matter more than the split. First, **every candidate family the census had named was
wrong**: explicit-lifetime `static function`, the implicit parameter-port shorthand, class type
parameters and forward `typedef class` all parse today when isolated as minimal reproducers, and the
three real defects were named by none of them — a snippet at `furthest_position` suggests a family,
only a parse settles one. Second, the **invalid** half is a finding rather than a leftover: a trailing
comma in a named parameter list, `#(.Name)` used as a *parameter* shorthand where the LRM makes the
parentheses mandatory, `$fatal("…")` with no `finish_number`, a port called `do`, `~&` as a binary
operator, and `module $_DLATCH_P_`. Accepting any of them would be an **over-acceptance** defect,
which a pass-rate metric is blind to by construction.

So the adjudication is kept as an oracle, not a paragraph.
`stimuli/sv/run_adjudication_repros.py` re-runs a growing set of minimal reproducers — each defect
paired with an *accepting* control that isolates it to exactly one difference — as a two-sided
ratchet: a defect that starts parsing fails with "flip it, the fix landed"; an invalid case that
starts parsing fails as an over-acceptance regression; a control that stops parsing fails because its
reproducer no longer isolates anything. It was proven able to fail before it was trusted.

⭐⭐ **It has five classes, and the fifth exists because the first four could hold only one direction
of a claim.** `defect` says *valid SV that PGEN rejects today* and goes red the day the fix lands.
There was no mirror for *illegal SV that PGEN accepts today*: filing such a row as `invalid` expects
a REJECT and so fails on the very commit that files it, and filing it as anything else lies. ⛔ A
known over-acceptance could therefore be written down in prose and **nowhere the runner could see
it** — which is exactly the asymmetry that lets an over-acceptance age quietly while every
rejects-valid defect is ratcheted. `accepts_invalid` closes it: the row expects ACCEPT because that
is what the parser does, and the day the owning fix lands the runner fails with *"flip it to
`invalid`"*, after which the row guards the fix against regression forever. Two coherence guards ride
with it — an unknown class is refused rather than skipped, and a class that contradicts its own
`expect` is refused — because a manifest is an oracle and an unreadable row in it must never read as
a green one. Four adversarial arms, control included, are recorded under
`docs/tasks/artifacts/sv_corpus_grad/accepts_invalid_class/`.

⭐⭐ **A verdict is not an arm, and the difference is where a whole construct went missing.** A row
may also pin *which alternative* parsed it — a nested chain of AST `kind` values, optionally negated
with `!`. That column earned its keep twice. The first time, a control claimed two operands carried
the keyword `intersect` and therefore could not be one plain expression; it was parsing as one plain
expression, and a verdict-only oracle called it green. The second time was worse. Closing the
reserved-word hole above made two long-passing corpus files start failing, and the reason was not the
fix: IEEE 1364-2005's `enable_gatetype ::= bufif0 | bufif1 | notif0 | notif1` and
`pass_en_switchtype ::= tranif0 | tranif1 | rtranif1 | rtranif0` had been extracted with their
**trailing digits dropped**, leaving `/bufif\b/` and `/tranif\b/` — tokens that can never match,
because `\b` demands a word boundary and `0` is a word character. Eight gate keywords were
unparseable. A ninth and tenth, `buf` and `not`, were unreachable for a different reason: their
instance rule spelled `( comma output_terminal )* comma input_terminal`, and a greedy repetition the
engine will not backtrack into ate the mandatory trailing input terminal — for every ordinary gate
instantiation, though **not** literally for every input, which is a distinction the first write-up of
this got wrong. The star can only eat a terminal it can parse, so a final terminal that cannot start
a net lvalue (`buf g(o, 1'b0)`, `buf g(o, (a))`, `buf g(o, $signed(a))`) left the star nothing to
take and the rule did match. The corpus rule-coverage artifact had been recording exactly that —
four files — for as long as it has existed.

⛔ **None of that was visible, because all ten forms still parsed** — as *UDP instantiations whose
type name is a reserved keyword*, which is precisely what the unguarded `identifier` rule permitted.
The over-acceptance was **load-bearing for constructs the parser could not otherwise reach**, and
every verdict-based instrument in the repository — the corpus pass count, the two-sided repro
ratchet, the syntax-closure gates — read green across all of it. Only the arm moved, from
`udp_instantiation` to `gate_instantiation>enable>bufif0`, and only a check that looks at the arm can
see that. The general lesson is worth more than the fix: **a defect can hide behind a different
defect of the opposite sign**, and a suite that only asks *did it parse* will confirm both.

⭐⭐ **And the sharper half of that lesson is not "we had no instrument" — it is "we had one and
nothing read it."** `stimuli/sv/characterization/rule_coverage_sv_2017.tsv` is a tracked artifact
listing, per grammar rule, how many of 16 336 real files ever caused it to fire. `enable_gatetype`
and `pass_en_switchtype` sit in it as `GAP 0`, and `n_output_gate_instance` as `covered 4` — the
first two saying *this production has never once fired on real SystemVerilog*, the third quietly
bounding how far the third defect reached. Both facts were published, in the repository, before
anyone went looking. **A zero in a coverage table is a claim that a construct is unsupported, and it
is worth exactly as much as whatever reads it.** There are 104 such rows, and nothing consumes them,
which is now tracked work rather than an observation.

The first seven rows in the new class are two measured over-acceptances that had nowhere to live. One
is positional: every **non-final** component of a hierarchical path accepts a reserved keyword —
`ral.module[0].g()`, `ral.module.g()` and `module.g()` all parse — because the component loops spell
the raw `identifier` rule while only the final component is `non_keyword_identifier`. The same
keyword in any *guarded* position (as the method name, as the final component, as a declared name)
is correctly refused, which is what makes this a claim about position rather than about the keyword.
The other is an extraction defect: `class_qualifier` is an Annex A **nonterminal** that PGEN carries
as a literal keyword, and the LRM's whole `class_qualifier ::=` definition line was welded onto
`primary`'s `| null` alternative with the footnote superscript kept as a token — so the standard's own
text for a production parses as a SystemVerilog expression, under `sv_2017` with footnote `43` and
under `sv_2023` with `48`, because the 2023 edition renumbers it.

The instrument itself needed three corrections, and the third is the general lesson. Its residue
bucket — the one that means *candidate parser defect* — read **47 → 22 → 20**: a class-only
declaration prelude could never satisfy a cross-file *module*; a `type_name`-only harvest could never
see a missing *package* behind `import uvm_pkg::*;`; and worst, the harvest returned an interface's
own **port** as a demanded-and-missing name, so declaring it shadowed the port and the parse went
**backwards**. A search that can move away from an answer will report "unexplained" for files that
are fully explained. Every candidate is now admitted by measurement: a name joins the prelude only if
adding it does not reduce how far the parse reaches. The same shape defeated the keyword filter —
the grammar's own reserved-word list holds 173 spellings and omits `covergroup`, `virtual` and `bind`,
each of which a speculating PEG parser really does report as missing, and `class covergroup; endclass`
is a syntax error that poisons every later probe. The fix was not a longer hand-typed list but asking
the parser: a candidate's declaration block is parsed before it may enter a prelude.

One refinement of the census falls out of this, and it does not weaken it. The census proved 2,285
rows carry a deferral their bytes refute **at the first failure**; that stands. What the adjudication
adds is that for 26 of the 57, once the fact and unit-shape blockers are removed, the *residual*
failure lands back inside the macro window. **Text-refuted at the first failure** and **expansion is
irrelevant to this row** are different claims, and they were being read as one.

#### When an ACCEPT proves nothing

The first of those defects took one token to fix, and taught more than that. Inside a `cross`, the
semicolon was spelled twice — once in `cross_body`, once in `cross_body_item` — so the grammar
demanded `option.weight = 2;;`, and *that* spelling, the one no simulator accepts, was the only one
that parsed. The standard is genuinely self-contradictory here: A.2.11 writes the `;` in both
places, while §19.6.2 and §19.6.3 of the same clause write
`cross a, b { ignore_bins ignore = binsof(a) intersect { 5, [1:3] }; }` with one. The clause
examples win, as in [the queue-slice repair](#when-the-pin-table-already-contradicts-itself)'s
precedent, and removing the duplicate closed a rejects-valid *and* an accepts-invalid defect at once.

Then the fix disproved its own leaf. The reproducer paired with it used `ignore_bins ib = ca with
(…)` — and after the fix it *still* rejected, for an unrelated reason it had been quietly crediting
to the first. A reproducer that fails for two reasons isolates neither.

Probing the rule around it turned out to need a specific discipline, because `select_expression`
ends in a **catch-all** alternative reaching the general expression hierarchy. That makes a great
many inputs parse without the intended alternative ever firing:

| probe | verdict | what it proves |
|---|---|---|
| `binsof(ca) && binsof(cb)` | accepts | **nothing** — the whole thing is a legal expression |
| `binsof(ca) intersect {1} && binsof(cb) intersect {2}` | accepts | ⛔ **nothing either** — see below |
| `( binsof(ca) )` | accepts | **nothing** — a parenthesized expression |
| `( binsof(ca) intersect {1} ) && binsof(cb)` | rejects | the `&&` continuation is inert |
| `x with (a == 1)` | rejects | the `with` continuation is inert |

The trick is to force the alternative with a **keyword the catch-all cannot swallow** — `intersect`
here. The same trap sits one rule lower: `intersect { … }` has *literal* braces in the standard and
EBNF repetition in the grammar, yet `intersect { 5, 6 }` parses, because `{5, 6}` is read as a
concatenation. Only `intersect { 5, [1:3] }` — a range is not an expression — reveals that the
braces were lost, and that the rule is simultaneously under- and over-accepting.

Row 2 of that table is the one worth dwelling on, because it shipped in this book claiming the
opposite. The reasoning behind it was that both operands carry the keyword `intersect`, so the input
could not be passing as one ordinary expression — and that was true, and it was not the accidental
route that was actually taken. **Dumping the AST settles what an exit code cannot:** one `condition`
node, *zero* `and` nodes, with `binsof(cb)` parsed as a plain subroutine call and `intersect` as a
hierarchical identifier, all of it swallowed by the seed's own `covergroup_range_list*` — the
repetition that should have been literal braces. The `&&` continuation was dead the whole time.

**Ruling out one accidental route is not ruling out the accidental route.** There is no general
argument that finishes this job; the only reliable move is to read the arm that fired out of the
AST. That is now mechanical: rows in the reproducer manifest may pin an `arm` — a `>`-separated chain of
AST `kind` values, optionally negated — and an accept down any other route fails the oracle. The
masking pin above now carries `condition,!and`: it must keep parsing, and it must keep producing no
`and` node, which is the falsified claim written down as a check. The kinds an `arm` names have to be
*unique to the alternative under test*, which is itself a grammar-authoring constraint worth knowing.

#### Left recursion that the linter says is handled, and is not

All three of those inert alternatives turned out to be one defect. IEEE 1800-2017 A.2.11 writes
`select_expression` with three **directly left-recursive** alternatives — `&&`, `||`, and
`with ( … )` — and transcribed literally, all three were dead code. PGEN's LR elimination rewrites
only the **indirect wrapper** shape: an alternative that is a bare reference to a rule which itself
begins with the base rule. A self-reference in *first position inside a choice* matches nothing the
planner looks for, so the alternative reaches codegen intact and the runtime cycle guard meets it at
the seed:

```
🚪 Entering branch 3/8 for rule 'select_expression' at position 127
💥 Infinite recursion detected in rule 'select_expression' at position 127
…
🏁 Rule 'select_expression' selected branch 7/8 consuming 2 chars (branch_policy=longest_match)
✅ Rule 'select_expression' successfully parsed from 127 to 129 (consumed 2 bytes: ' x')
```

The seed won and nothing extended it — which is exactly what a missing continuation looks like.

⭐⭐ **The obvious repair is the wrong one, and rejecting it is the most transferable thing here.**
Flattening the recursion by hand in the grammar — `seed ( continuation )*`, the shape this grammar
already uses for `expression` — works, and it was implemented and verified before being thrown away.
It is wrong because it hand-compiles into an EBNF a transform that is *objectively common to every
EBNF*: it stops the grammar transcribing Annex A, it yields a flatter AST than the eliminator's own
left-nested fold, and it costs a schema break that the real fix would have to break again. A grammar
should carry only what is specific to its language. The repair belongs to the engine, where it fixes
every grammar at once and costs this one **zero bytes** — tracked as `A2.5`.

✅ **`A2.5` landed (2026-08-11), and the engine half is done.** A pre-pass
(`normalize_direct_left_recursive_alternatives`) hoists each directly left-recursive alternative into
a synthetic rule and leaves a bare reference behind, which is precisely the shape the existing planner
already eliminates — so one tested transform now covers both spellings, and the author's `$N` indices
ride along untouched. Measured on SystemVerilog: **4 dead alternatives → 0**, `&&` / `||` /
`with ( … )` and `block_event_expression`'s `or` all parse, each returning the AST its annotation
declares, and the grammar still transcribes Annex A verbatim.

⭐ **The engine had to clean up after itself, and a gate is what said so.** Elimination *rewrites* the
wrapper rules it consumes rather than deleting them, so the four synthetic rules survived as
**defined-but-referenced-by-nothing** — useless symbols emitted as dead parser code.
`sv_syntax_closure_gate` failed with `unreachable_rules=4 > max_unreachable_rules=0` and named all
four. The fix is a retraction pass, **not** a raised cap: this contract's own history drove that cap
from 1 to 0, and engine litter does not get a waiver a grammar would not get. The gate now passes with
the contract byte-unchanged.

✅ **`A2.6` then landed the linter half (2026-08-12) — and the "unearned" message turned out to be
false about thirty more cycles.** The verdict is now DERIVED from the elimination pass's own outcome
rather than asserted, and the derivation is stronger than the one `A2.5` sketched. Asking
`detect_left_recursive_chain_plan` "would you eliminate this rule?" cannot work here at all: the lint
sees the grammar **after** the pass, where a rewritten base rule no longer holds the wrapper
alternatives the planner matches, so that call returns `None` for every rule — including the ones it
had just eliminated. The pass has already run; its **result** is the ground truth. So the pass now
reports what it did, the loaded grammar carries that record, and the lint headline reads:

⛔ **The block below is the state that motivated this section, not today's output.** It is quoted at
the point the defect was found, when the indirect pass declined SystemVerilog's two remaining knots;
since the admission flip the same command reads `(1608 rules) — left_recursion_unhandled=0`. The
argument the block illustrates is unchanged and is why the counter can be trusted now.

```text
grammar lint: 'systemverilog' (1485 rules) —   ← historical: today this reads 1608 rules / …=0
  left_recursion_unhandled=30 (warning — the LR-elimination pass ran and these cycles survived it;
                               only the runtime guard is left, and it REJECTS same-position re-entry),
  left_recursion_eliminated=2  (info — derived from the pass's own outcome), …
  [info]  … ELIMINATED 2 left-recursive rule(s) on this grammar: block_event_expression, select_expression
```

⛔ **Thirty of thirty.** The pass rewrote 2 rules; the linter told all 30 survivors they were
*"handled by PGEN's LR elimination + runtime cycle-breaking"*. It is worth being precise about why
that was so wrong: the guard does not handle a surviving cycle, it **rejects** re-entry at the same
input position, so the derivations that need it are unreachable. The demonstrated case is SV's first
printed cycle, `casting_type -> constant_primary -> constant_cast -> casting_type` — IEEE 1800-2017
A.8.4 makes `int'(2)'(3)` a legal cast chain, and PGEN rejected it (`furthest_position=40`,
`💥 Infinite recursion detected in rule 'casting_type'`). Class size when `A2.6` measured it: SV
**30**, the raw Annex A transcription **23**, `ebnf` **5**, every other grammar **0**.

✅ **PGEN now eliminates indirect left recursion too** (`ENGINE-UNIVERSAL-SERVICES.13` slice 5,
`ast_pipeline::indirect_lr_elimination`). The pass runs after the direct/wrapper planner, on what
that planner left behind, and rewrites a whole **route** rather than a single hop:

```text
X            := X_lr_base ( X_lr_suffix )*
X_lr_base    := <the author's alternatives, each CYCLIC one replaced by a sheared CLONE>
X_lr_suffix  := <one branch per route: the route's residuals, innermost first>
```

Post-slice-5 class size: `ebnf` **5 → 0**, SystemVerilog **30 → 28**, every other shipped grammar
still **0**. ⛔ SystemVerilog's cast/call and SVA property knots are **declined**, and the reason is
measured rather than unfinished — see *what the pass refuses, and why it says so* below.

⭐ **Three properties are worth knowing as a grammar author.**

- **Your declared AST survives.** The route's per-hop return annotations are **composed** into one
  template — each hop's `$1` filled by the hop below it, every other `$N` remapped into the
  flattened suffix — and the existing chain fold (`ENGINE-UNIVERSAL-SERVICES.8`) applies it per
  iteration. The tree you get is byte-identical to the left-nested one your un-eliminated grammar
  declared.
- **The pass verifies itself before it commits.** Each plan is applied to a *copy*, the left-recursion
  lint is re-run, and the rewrite is kept only if the base rule's cycle is actually gone and the
  total row count strictly fell. A plan that would half-shear a cycle is refused with the surviving
  path printed.
- **A hop with a residual and no return annotation is refused**, because its undeclared value is the
  engine's default shaping of the whole alternative and no template can reproduce that. Annotate the
  hop and the pass will take it.

⛔ **What the pass refuses, and why it says so.** `--report-indirect-lr-plan` prints
`indirect_eliminated_base_rules=/indirect_clone_rules=/indirect_refusals=` followed by one line per
rewrite and per refusal, and — the line to read first on SystemVerilog —
`starvation-safe candidates: 0/28`.

⛔⛔ **A rule can only absorb a chain if nothing else can be STARVED by its new greed, and that
check is TRANSITIVE.** After `X := X_base ( X_suffix )*`, PGEN's `*` is greedy and never retries at a
lower iteration count. So any rule holding `X` at its left corner *with more elements after it* can
lose — and so can any rule holding something **transparent** to `X`, because an alternative that is
a bare reference consumes exactly what it forwards to:

```text
casting_type := … | constant_primary                          ← bare reference: transparent
cast         := casting_type tick lparen expression rparen     ← holds it WITH a residual
```

Absorbing SystemVerilog's cast chain at `constant_primary` makes `casting_type` greedy too, so
`8'(1)` is swallowed whole as a cast of its own and `cast` can never match its trailing
`tick lparen expression rparen`: `initial k = 8'(1);` stops parsing while
`parameter logic [7:0] K = 8'(1);` still works. Absorbing it **unguarded** therefore trades one
defect for another, and for several releases the engine simply declined the knot.

⭐⭐ **It no longer declines it — it guards it.** Everything from here to
[The guarded admission is what PGEN ships](#the-guarded-admission-is-what-pgen-ships) describes the
census the engine reports when a knot is *unabsorbed*, and on SystemVerilog that census is now empty
because both knots were absorbed with call-site guards. **Read this section with
`--indirect-lr-admit-starvation-safe-only` in mind**: it is the view of the problem, and the last
section is the view of the fix. Every figure quoted below — `0/28`, `16/28`, the verdict censuses —
is what that lever prints today, unchanged from when it was the shipped policy.

⇒ **If your grammar reports `verdict=STARVED`, read the `guard:` line next** (below): a starved
candidate that is guard-feasible is one the engine will absorb *with* a guard. If it is starved and
not guard-feasible, the fix is usually to give the holder a form that does not need the residual —
or to accept that this construct is not chain-absorbable. The report names the holder, its
alternative and the exact residual at risk.

### `verdict=STARVED` is not the last word — read the `guard:` line next

`STARVED` says *this base rule cannot be absorbed unguarded*. It does not say the construct is
hopeless, and the report prints a second, independent verdict beside it — the one the shipped
criterion reads:

⛔ The block below is the SystemVerilog census under `--indirect-lr-admit-starvation-safe-only`. On
the shipped path both of these knots are absorbed, so the same report prints `candidates=0` and the
guards themselves instead.

```text
starvation-safe candidates: 0/28
guard-feasible candidates: 16/28 (option (iii): a call-site follow-restriction guard on the sheared clone)
guard-verdict census over surviving starvation sites: guard_incomplete=68 guardable=29 residual_nullable=29
seed-verdict census over the same sites (the TRAILING guard position): no_seed_tail=65 seed_residual_nullable=29 trailing_guard_required=32
candidates needing the TRAILING guard emitted: 15/28

[candidate] constant_primary  routes=10  seeds=0  clone_cost=14  verdict=STARVED
    guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=1
    seed: TRAILING GUARD REQUIRED  seed_first={'/}~  seed_routes=10/10
    ⛔ starved by cast alt#0 — residual 'tick lparen expression rparen'
       [guard=guardable seed=trailing_guard_required first={'/}~ hops=1]
```

The idea is the PEG-native repair: commit a chain iteration only when the position after it can
still start the holder's residual.

```text
X_guarded := X_lr_base ( X_lr_suffix &FIRST(residual) )*      # the first cut — see below
```

⛔ That first cut is **not** the shape PGEN would emit, and both halves of it were corrected by
measurement: the byte test `&FIRST(residual)` is defeated by a comment at the iteration boundary,
and one guard position leaves a second starvation open. The measured form is two structural
lookaheads — *The guard, measured* below. The byte sets stay, as the inputs that DECIDE whether a
guard is owed; they are not what gets emitted.

It has to be written on the **call site**, not on the rule — `constant_primary` must reserve the
trailing `'( … )` when it is reached from `cast` and must *not* when it is reached from an ordinary
expression — which is why the guard lands on the sheared clone the eliminator already emits, and why
the report prices the clone chain (`max_hops`) and how many distinct byte tests it needs
(`variants`).

⛔ **Both of those numbers are LOWER BOUNDS, and the report prints what they under-count beside
them.**

- **`variants` under-counts the CHAINS.** It counts distinct residual *byte sets*, which is what the
  byte-test form above would have compared — and that form is measured dead. The emitted guard is a
  structural sub-parse, so two residuals sharing a FIRST set are two different rules: `casting_type`
  reports `variants=1` and the planner emits **two** chains, one for `cast`'s `'( expression )` and
  one for `constant_cast`'s `'( constant_expression )`. Read the `🛡` lines for the real count.
- **`max_hops` under-counts the CLONES.** It is the *shortest* transparency distance, while each
  site's `chain=` lists every rule on any transparent path — and they differ wherever transparency
  branches. SystemVerilog's dialect twins branch: `primary` reaches `cast` through both
  `primary_sv_2017` and `primary_sv_2023`, so the guard clones five rules where `hops=3` reads as
  four, and leaving either twin unguarded would leave a live unguarded route to the same starvation.
  Measured: 6 of 129 sites here, 5 of 77 in the LRM wrapper.

**How to read each site's verdict:**

| verdict | meaning |
|---|---|
| `guardable` | `FIRST(suffix) ⊆ FIRST(residual)` — the guard refuses the fatal iteration and provably no earlier one |
| `no_competition` | the suffix and the residual can never start on the same byte ⇒ this holder cannot be starved; **no guard owed** |
| `residual_nullable` | the residual can match empty ⇒ the holder succeeds anyway; **no guard owed** |
| `guard_incomplete` | they compete but containment fails ⇒ the guard would cut the loop short at an intermediate iteration |
| `undecidable` | a FIRST set could not be resolved, or the suffix is nullable ⇒ nothing may be concluded |

⭐ **Why `guardable` is safe in both directions.** Wherever the loop continues, the suffix matched
there, so that position begins with `FIRST(suffix)` — and containment makes the guard pass there.
The only iteration it can refuse is the last one. And because the guarded rule accepts a *subset* of
what the unguarded one does, the holder still only accepts strings the declarative grammar already
licensed: the guard **recovers** derivations, it never invents them.

⛔ **`FEASIBLE` means sound, not closed — and the report tells you which.** A trailing `~` on a byte
set (`suffix_first={'/}~`) marks it as an **over-approximation**: the guard passes at positions the
residual cannot actually start from, so it silently declines to refuse the iteration you wanted it
to refuse. It never becomes unsound — an over-permissive guard can only fail to fire, never
over-accept — but it is not a proof.

On SystemVerilog **every** site is `~`, for two compounding reasons:

1. a FIRST set is exact only for single-byte-decided shapes, so any multi-token residual
   (`tick lparen expression rparen`) is approximate by construction; and
2. `trivia := (line_comment | block_comment)*` is nullable and leads every token, so `/` belongs to
   every FIRST set — concretely, `int'(2)/*c*/'(3)` slips a byte-test guard.

⇒ if you are relying on this, the exact form is a **structural** lookahead over the residual rather
than a byte test: precise, at the cost of a per-iteration sub-parse instead of one byte compare.

⛔ **This census is reported, not applied.** The verdict the eliminator acts on is still the
structural one, so nothing about which knots PGEN absorbs has changed.

### The guard, measured — and why one guard is not enough

The census above says a guard is *expressible*. Running one says whether it *works*, and the answer
has three parts. All of them are measured on a synthetic that carries a real layout model — a
nullable `trivia` rule leading every token, the way SystemVerilog writes it — on both the generated
parser and the interpreter
(`docs/tasks/artifacts/engine_universal_services/guard_effectiveness/`).

**1. The structural form works; the byte form does not.** Take the starvation case and put a comment
at the iteration boundary:

```systemverilog
k = n'(n);          // byte guard: ACCEPT     structural guard: ACCEPT
k = n'(n)/*c*/;     // byte guard: REJECT ⛔   structural guard: ACCEPT
```

`/` is in `FIRST(residual)` because `trivia` is nullable and leads the tick, so the byte test passes
at exactly the position it had to refuse. The loop commits the fatal iteration and the holder
starves. The structural lookahead re-parses the residual there, comment and all, and refuses
correctly.

**2. A guard on the loop closes only half the problem.** The rewritten rule is
`X := X_lr_base ( X_lr_suffix )*`, and `X_lr_base` carries the *sheared clones* — copies of the
intermediates with the cycle edge removed. One of those clones can match the holder's whole text on
its own, without the loop running at all:

```systemverilog
k = t'(n);          // the loop takes ZERO iterations; the clone in X_lr_base ate the cast
```

That parse starves the holder just as badly, and no guard on the `*` can see it. The over-long match
came from an *alternation*, and — this is the part that surprises people — **PGEN's choice does not
give back either**:

```text
ch := "a" | "ab"
scratch := ch "bc"          on "abc"  ⇒  REJECT
```

The default `longest_match` policy evaluates every alternative and keeps the longest in a single
winner slot; the losers are discarded, not remembered. When `"bc"` then fails there is nothing to
retry, even though `"a"` would have worked. (Under `@branch_policy: ordered` the same grammar
accepts, because it commits to a *different* single alternative — not because it backtracks.)

⛔ **Do not read `|` as "try each until one works".** It is *"evaluate all, keep one, commit"*.

**3. So the repair has two positions, on the same clone.**

```text
X_guarded := X_lr_base ( X_lr_suffix &( residual ) )* &( residual )
                        └── stops the LOOP ──┘        └── refuses an over-long SEED ──┘
```

They close **disjoint** starvations, and dropping either one loses those cases outright:

| | `k = n'(n);` (loop) | `k = t'(n);` (seed) |
|---|---|---|
| per-iteration guard only | ACCEPT | REJECT |
| trailing guard only | REJECT | ACCEPT |
| **both** | **ACCEPT** | **ACCEPT** |

The trailing guard cannot rescue the loop case because by the time it runs the possessive `*` has
already committed to its maximum count, and there is no give-back to a shorter one. The
per-iteration guard cannot rescue the seed case because the loop never ran.

⭐ **Neither guard needs backtracking**, which is what keeps the repair affordable: refusing an
over-long seed makes that alternative *fail* rather than win, and a failed alternative is simply one
fewer candidate for the tournament that was going to run anyway.

⛔ **Both belong on a clone, never on the shared rule.** `constant_primary` must reserve a trailing
`'( … )` when it is reached from `cast`, and must **not** when it is reached from an ordinary
expression. Writing the guard onto the rule breaks every caller that wants the whole run — measured,
not assumed: a residual-free holder of a rule-guarded base rejects `k = n;`.

**4. So what does "on a clone" actually look like?** One guarded clone per *transparent hop* between
the holder and the base, plus the guarded base itself — the census reports that depth as
`max_hops` — and only the holder's **left corner** is repointed, so every other element of its body,
and therefore every `$N` in its annotation, is preserved:

```text
outer_cast := ct_guard tick lparen lit rparen     # the holder: left corner repointed, nothing else
ct         := kw | prim                           # the originals stay, untouched…
prim       := prim_base ( prim_suffix )*          # …so a residual-free caller still gets the whole run
ct_guard   := kw | prim_guard                     # the guarded chain, reachable only from outer_cast
prim_guard := prim_base ( prim_suffix &( … ) )* &( … )
```

⭐ **The fallback is the mechanism, and it is why the guard sits inside a choice rather than at the
holder's own call.** On `k = t'(n);` the seed matches the whole cast, the trailing guard refuses it,
and `prim_guard` *fails* — at which point `ct_guard`'s other alternative wins with the short match
and the holder gets its `'(n)` back. A guard that made `outer_cast` itself fail would have had
nothing to fall back to. That is the single-winner tournament from result 2 used as the recovery
path rather than fought.

This shape accepts all six starvation inputs **and** `k = n;` — the input the rule-guarded version
rejects. One grammar, both properties.

⭐ **PGEN synthesizes this shape.** The planner reconstructs the transparent chain, clones each hop
with its non-chain alternatives copied verbatim, and repoints the holder's left corner — you can see
exactly what it would emit in the dry run below. One detail differs from the hand-written form above,
and it is not cosmetic: the loop guard is hoisted into a named rule rather than written inline.

```text
X_lr_guard0        := X_lr_base ( X_lr_guard0_suffix )* &( residual )
X_lr_guard0_suffix := X_lr_suffix &( residual )
```

Written inline as `( X_lr_suffix &( residual ) )*`, the quantifier would iterate a *sequence* rather
than the suffix rule, and the left-recursion chain fold reads that quantifier's result as its list of
suffix records. Naming the group keeps the guarded rule's body **positionally identical** to the
unguarded one — same first element, same quantified rule reference, one appended lookahead — so the
AST it returns is the same by construction rather than by an argument about what a group's content
becomes. That is what lets a holder reach either rule and get the same value.

### Which candidates actually need the trailing guard — the `seed:` line

The two positions cost differently, so the report prices them separately. Beside every candidate's
`guard:` line there is now a `seed:` line, and beside every site's `[guard=…]` a `seed=…`:

```text
[candidate] casting_type      routes=10  seeds=4  clone_cost=14  verdict=STARVED
    guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=0
    seed: no trailing guard owed  seed_first={}  seed_routes=0/10

[candidate] constant_primary  routes=10  seeds=0  clone_cost=14  verdict=STARVED
    guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=1
    seed: TRAILING GUARD REQUIRED  seed_first={'/}~  seed_routes=10/10
```

Two rules on **one knot**, opposite answers — which is the whole reason the term exists.

**What `seed_first` is FIRST of**, and it is not the suffix. The sheared clone chain derives *the
closing rule's other alternatives*, then every step's residual on the way back out — every step
**except the cycle-closing one**, whose alternative the shear deletes outright rather than
redirecting. So the seed's tail is the route suffix minus its innermost residual, and a route whose
whole suffix comes from the closing step contributes no seed at all.

**`seed_routes=N/M`** is how many of the candidate's `M` routes contribute such a tail. A route
contributes only when both hold:

1. that tail is non-empty; and
2. its clone chain **survives** the shear — a rule whose alternatives were all sheared away yields no
   clone, and the alternative that referred to it vanishes from its parent in turn.

Condition 2 is what separates the two rules above. `casting_type`'s routes close at `cast` and
`constant_cast`, and each of those has exactly **one** alternative — the cycle-closing one — so the
shear leaves nothing to clone. Read it off the dry run's own clone set: it contains
`casting_type_lr_seed_constant_primary` and eleven siblings, and no `casting_type_lr_seed_cast`.

| `seed=` verdict | meaning |
|---|---|
| `trailing_guard_required` | the sheared clone can eat this holder's residual with the loop at zero iterations ⇒ **the trailing guard is mandatory here** |
| `no_seed_tail` | the rewrite adds no over-long seed at this base rule; **no trailing guard owed** |
| `seed_no_competition` | the seed tail and the residual can never start on the same byte; **none owed** |
| `seed_residual_nullable` | the residual can match empty ⇒ the holder cannot starve; **none owed** |
| `seed_undecidable` | a FIRST set could not be resolved ⇒ nothing may be concluded, and feasibility is blocked |

⭐ **There is deliberately no `seed_incomplete`.** The loop guard can cut a chain short at an
intermediate iteration, which is what `guard_incomplete` names. The trailing guard runs once, at rule
exit, on a clone reached only from a holder that wants the residual next — so refusing an over-long
seed there cannot lose a derivation that holder had.

Measured on the shipped grammars, under the pre-flip admission: SystemVerilog **15 of 28**
candidates need the trailing guard, the LRM wrapper **13 of 18**, and `ebnf` **0 of 5**. ⭐ Of the
two SystemVerilog candidates the engine now absorbs, `property_expr` is one that needed it and gets
`[loop+trailing]`; `casting_type` does not and gets `[loop]` alone. The engine emits the positions
each site actually owes rather than both by default.

### And "a guard is expressible here" is still not "this knot would close"

`guard-feasible` is a verdict about the **starvation gate**. Three more refusals live behind that
gate — a hop that declares no return annotation (the chain's AST could not be composed faithfully),
the trial re-lint, and the ambiguity comparison — and none of them is observable for a starved
candidate, because the planner is only ever reached for candidates the gate already admitted.

`--indirect-lr-plan-guard-dry-run` closes that blind spot by admitting the guard-feasible candidates
into the real elimination driver **on a clone of your grammar**, and reporting what it did:

```text
--- GUARD DRY-RUN: which guard-feasible candidates actually reach a PLAN ---
    inputs: annotations=present rules_with_branch_return_annotations=1069
    would_absorb=2 would_refuse=0 clone_rules=24 guard_chains=3 guard_rules=6
        left_recursive_rule_rows 28 -> 0
    ✅ would absorb 'casting_type'
    ✅ would absorb 'property_expr'
    🛡  guard 'casting_type_lr_guard0' [loop]  chain: casting_type  residual 'tick lparen constant_expression rparen'
        sites: constant_cast alt#0   rules: casting_type_lr_guard0_suffix, casting_type_lr_guard0
    🛡  guard 'casting_type_lr_guard1' [loop]  chain: casting_type  residual 'tick lparen expression rparen'
        sites: cast alt#0   rules: casting_type_lr_guard1_suffix, casting_type_lr_guard1
    🛡  guard 'property_expr_lr_guard0' [loop+trailing]  chain: property_expr  residual 'implies property_expr'
        sites: prop_primary_sv_2017 alt#13, prop_primary_sv_2023 alt#13   rules: property_expr_lr_guard0_suffix, property_expr_lr_guard0
```

Three things are worth reading carefully here.

**The row count is measured on the rewritten clone**, by the same detector the lint runs — not
inferred as *before minus absorbed*. That is why two rewrites can clear twenty-eight rows: both land
on a **dominator**, and a dominator's rewrite clears every rule on its knot. A large
`guard-feasible` count is a count of affected *rules*, and the number of *rewrites* is usually far
smaller.

**The `inputs:` line is not decoration.** A chain's AST only needs composing if your grammar declares
return annotations at all, so on an unannotated grammar every candidate passes that check
vacuously — and `would_refuse=0` would mean "nothing to compose" rather than "it composes". The
annotation census is printed next to the verdict so the two readings can never be confused. It is
also what explains a result that otherwise looks like a contradiction: SystemVerilog's hand-written
grammar (1069 annotated rules) absorbs both knots, while its LRM-generated wrapper view (21) refuses
thirteen candidates, every one of them for a missing return annotation. Those two grammars are
comparable in structure and **not** comparable in annotation.

**The `🛡` lines are the guard the planner synthesized**, and they are read back off the grammar it
wrote rather than off the plan that intended it. Each names the guarded stand-in rule, which of the
two positions it actually carries, the transparent `chain:` of rules cloned, the residual both
lookaheads test, and the holder call sites whose left corner was repointed. On SystemVerilog that is
three chains and six rules — and `property_expr` gets both positions while `casting_type` gets only
the loop, which is precisely what their `seed:` lines say.

⛔ **The dry run still parses nothing.** It answers *"would a plan be buildable here, and what shape
would it emit?"*. No parser is generated and no input is run, so nothing in its output is a claim
about what the rewritten grammar accepts or rejects.

⛔ **Since PGEN 0.1.0-`0032` this dry run reports what is LEFT to do, and on every shipped grammar
that is nothing.** The guard-feasible admission is now the shipped criterion (next section), so a
bare report arrives with both knots already absorbed and the dry run correctly answers
`would_absorb=0`. To see the census above — the population the flip absorbed — ask for the
pre-flip admission explicitly:

```bash
rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
    --report-indirect-lr-plan --indirect-lr-plan-guard-dry-run \
    --indirect-lr-admit-starvation-safe-only
```

⭐ The two blocks use different verbs on purpose. The dry run writes `🛡  would guard '…'`; the
shipped pass writes `🛡  guard '…'`. Both appear in one report, so a tool reading it can tell a
prediction from a fact without tracking which section it is inside.

### The guarded admission is what PGEN ships

Everything above was, for several releases, a description of a capability PGEN had and did not use.
The eliminator's criterion was *"absorb a knot only if no rule outliving the rewrite can starve
it"* — safe, and on SystemVerilog it admitted **0 of 28** candidates. The two knots it declined are
LRM-legal constructs, so declining them meant rejecting valid input:

```systemverilog
parameter logic [7:0] K = 8'(1);   // A.8.4 constant_cast — rejected, before
```

The criterion is now *"absorb a knot if it is safe, **or** if a call-site follow-restriction guard
makes it safe"*, and the guard is emitted as part of the rewrite. The path from decision to shipping
was deliberately four steps, because each one could have refuted the last: the guard shape was
**decided** against measurements, **modelled** as a hand-written grammar, **emitted** by the planner,
**executed** as a compiled parser, and only then made the default.

What that changed, measured on one binary with the A/B lever below:

| grammar | `left_recursion_unhandled`, before | after | generated parser |
|---|---|---|---|
| `systemverilog` | 28 | **0** | the one artifact that changed |
| `vhdl`, `regex`, `json`, `ebnf`, the RTL pair, the SV preprocessor, both annotation grammars | 0 | 0 | byte-identical |

Every other SystemVerilog lint counter is unchanged — `non_terminating=0`,
`ordered_choice_shadowing=0`, `always_succeeds_alternatives=6`, `unreachable_rules=0`,
`undefined_references=0`, `nullable_repetition=0`, `profile_orphans=0` — and the grammar grew from
1488 to 1608 rules, all of them synthesized clones, helpers and the six guard rules.

The three guards SystemVerilog ships are printed by the ordinary report:

```text
indirect_eliminated_base_rules=3 indirect_clone_rules=24 indirect_refusals=0 indirect_guard_chains=3
    ✅ absorbed at 'casting_type'
    ✅ absorbed at 'property_expr'
    ✅ absorbed at 'incomplete_class_scoped_type_sv_2023'
    🛡  guard 'casting_type_lr_guard0' [loop]  chain: casting_type (max_hops=0)  residual 'tick lparen constant_expression rparen'
        sites: constant_cast alt#0   rules: casting_type_lr_guard0_suffix, casting_type_lr_guard0
    🛡  guard 'casting_type_lr_guard1' [loop]  chain: casting_type (max_hops=0)  residual 'tick lparen expression rparen'
        sites: cast alt#0   rules: casting_type_lr_guard1_suffix, casting_type_lr_guard1
    🛡  guard 'property_expr_lr_guard0' [loop+trailing]  chain: property_expr (max_hops=0)  residual 'implies property_expr'
        sites: prop_primary_sv_2017 alt#13, prop_primary_sv_2023 alt#13   rules: property_expr_lr_guard0_suffix, property_expr_lr_guard0
```

⭐ **A wider admission is not a weaker one.** The guard admits a candidate to *planning*, not to
absorption: every refusal downstream of the starvation gate still fires. A hop that declares no
return annotation is still refused by name, so a chain whose AST could not be composed faithfully is
still declined rather than silently reshaped — measurable on the unannotated synthetic
`p1_knot_a_defect.ebnf`, where the flip adds a *third* named refusal and absorbs nothing.

#### The A/B lever — `--indirect-lr-admit-starvation-safe-only`

To measure what the guarded admission buys, ask for the older one. It runs the same pass with the
pre-flip criterion, so a before→after comes from **one binary** rather than from two builds that
differ in more than the question:

```bash
# the shipped policy
rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar
#   → left_recursion_unhandled=0

# the pre-flip policy, same binary, same grammar
rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar \
    --indirect-lr-admit-starvation-safe-only
#   → left_recursion_unhandled=28
```

On the worked example in
`docs/tasks/artifacts/engine_universal_services/guard_parses/` — a twenty-line grammar carrying the
same cast knot SystemVerilog has — the two arms differ on exactly the inputs the knot owns:

| input | `--indirect-lr-admit-starvation-safe-only` | shipped |
|---|---|---|
| `k = n'(n);` | accept | accept |
| `k = n'(n)'(n);` | **reject** | **accept** |
| `k = t'(n);` | accept | accept |
| `k = n;` | accept | accept |

The rejections on the left are the runtime cycle guard: the cycle was never eliminated, so the parser
refuses the derivation that would re-enter it. On the right the chain is absorbed *and* its surviving
starvation site is guarded, so all four parse — including `k = n;`, the holder that wants **no**
residual after the rule and which a guard placed on the shared rule would break.

⛔ **The lever is not a way to build a deliverable.** Nothing in `rust/Makefile`, `scripts/` or the CI
workflows passes it, and the pass prints a warning banner at default verbosity whenever it is on. A
parser built through it rejects LRM-legal SystemVerilog, on purpose — that is the point of a
measurement arm, and it is the arm that reproduces the defect the flip closed.

⭐ **One criterion had to be deleted along the way: `seeds=0` is not a disqualification.** The
direct/wrapper elimination *drops* a left-recursive alternative, so a rule whose every alternative is
on the cycle has nothing left to seed from. The indirect transform *clones* it with the cycle edge
sheared, so the seeds come from **under** the cyclic alternative — which is what makes a *dominator*
of a mutually-recursive set eligible at all.

⛔ **READ THAT COUNT CORRECTLY: it counts RULE ROWS, not cycles.** The detector starts a DFS from
every rule and reports any that closes back on itself, so one 12-rule cycle is printed 12 times —
once per rule that can start it. Canonicalising by rotation
(`docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/canonicalize_lint_cycles.py`)
gives the real worklist: SystemVerilog's **30 rows are 7 distinct cycles** and `ebnf`'s **5 are 3**.
The headline is not wrong — it is answering *"how many rules are affected"*, which is the right
question for an author looking for a rule name and the wrong one for anybody sizing a fix.
⭐ `.13`'s per-cycle adjudication then measured what those 10 cost: **8 of the 10 reject text the
standard licenses** (a numeric size cast in a constant expression, a function-call cast size,
property-level implication, and two PGEN return-annotation forms the hand-written EBNF frontend
accepts), and 2 are *dead-but-covered* — the alternative is unreachable but a sibling rule derives
the same text. All four SV cast rows and both SV property rows reduce to **one edge each**, so the
fix surface is 3 knots rather than 30 cycles.

⛔ **The warning is deliberately NOT a `dead_branch` error.** A surviving *indirect* cycle does not
prove any one alternative is dead — the intermediate rules may still have non-recursive paths, so the
alternative can still parse something. Claiming deadness there would repeat, in the failing direction,
exactly the unsound verdict the always-succeeds correction above retired. What is sound, and what the
message states, is that the cycle's *left-recursive derivations* are unreachable.

⭐ **How much a surviving cycle costs you, measured — it is nothing until the cycle is the only
road.** `SV-CORPUS-GRAD.13c.2b` pinned this on the same `casting_type` cycle, all three inputs in the
same constant-expression position:

| input | what `casting_type` can match at the seed | verdict |
|---|---|---|
| `parameter int K = int'(1);` | `simple_type` — branch 1/5 | ACCEPT |
| `parameter logic [7:0] K = W'(1);` | `simple_type → ps_type_identifier` — branch 1/5 | ACCEPT |
| `parameter logic [7:0] K = 8'(1);` | only `constant_primary` — branch 2/5 | **REJECT** |

The guard fires in **all three**: the `W'(1)` trace prints `💥 Infinite recursion detected in rule
'constant_primary'` and then `🏁 Rule 'casting_type' selected branch 1/5` and parses. ⇒ the cost of a
surviving cycle is not that it exists, it is that *no other alternative of the re-entered rule can
match this text* — which is why the class size (30 on SV) is a count of cycles and never a count of
defects, and why sizing one means hunting the inputs where every sibling alternative is dead. That
one cycle blocks 2 vendored OpenTitan corpus files whose only unparseable construct is a numeric size
cast in a package parameter (proven by removing just the `N'` prefix: both flip to `parse_full
passed`). ⛔ And the grammar-tier escape is closed by measurement, not by taste — that leaf's audit
shows `constant_primary_sv_2017`/`_sv_2023`/`casting_type` are **order-identical** to the Annex A
extraction, so hand-splitting the cycle would trade a byte-for-byte standard transcription for a
workaround.

⭐ **And the diagnostic no longer hides its own findings.** This class printed `take(10)` with no
override, so 20 of SV's 30 were unreachable from the CLI at any verbosity — the sweep that found all
this had to go around the instrument. Every class now shares one print helper with a cap of 40 and a
`PGEN_LINT_DUMP_ALL=1` escape, and the truncation line names it (*"... and 12 more … (set
PGEN_LINT_DUMP_ALL=1 to print all 52)"*). A capped diagnostic with no "show all" is how a finding
hides.

⛔ **The linter reported all of this as clean**, in a message that tells a grammar author to look
elsewhere: *"is left-recursive (cycle: `select_expression -> select_expression`) — handled by PGEN's
LR elimination + runtime cycle-breaking (informational, not an error)"*. Runtime cycle-breaking does
not *handle* a directly left-recursive alternative; it **rejects** it. This is contract item 3 — *no
dead branches* — failing in the passing direction, and it is the mirror image of the `A2.3` finding
above, where the same check hard-failed a branch that was alive. A sweep of the post-elimination
gen-AST sizes the SV surface at exactly **two rules, four dead alternatives** out of 1 481
(`select_expression`, and `block_event_expression`'s `or` arm). Both are owned; the linter fix is
`GRAMMAR-WELLFORMED.A2.5`.

Two honesty rules fell out of that census and are worth carrying to any similar instrument:

- **The same falsification means different things per bucket.** The zero-directive test also flags 8
  rows deferred to the preprocessor lane, and reading all 8 refutes the exciting interpretation: they
  are excerpt-mode fixtures and unterminated-string tests whose preprocessor relevance is the *test's
  purpose*, not a directive in the text. The instrument now pairs every class with an explicit reading
  and refuses to run if a class lacks one.
- **A number in a tracked file with no gate behind it will be wrong when it is next quoted.** This
  artifact went stale within a day of being written, publishing the superseded 319-row bar after the
  Latin-1 fix had taken it to 318. The standing denominator gate that stops that recurring is
  `SV-CORPUS-GRAD.13b`.

#### When the pin table already contradicts itself

The pinned rulings are not just an override list — they are the project's accumulated reading of the
standard, and **an inconsistency among them is itself a defect**. `SV-CORPUS-GRAD.3.15` found one.

Two fixtures from the same suite use the same construct, a `begin … end` block placed where IEEE
1800-2017 A.4.2 has no production for one:

```ebnf
generate_region ::= generate { generate_item } endgenerate
generate_item   ::= module_or_generate_item | interface_or_generate_item
                  | checker_or_generate_item
generate_block  ::= generate_item
                  | [ id : ] begin [ : id ] { generate_item } end [ : id ]
```

`generate_block` is reachable only from `loop_generate_construct`, `if_generate_construct` and
`case_generate_item` — so `for (…) begin : A … end` and `if (P) begin : A … end` are legal and do
parse, while the same block sitting *directly* in a generate region or a module body has no
production at all. It is the pre-2005 "legacy generate region".

One of those fixtures was pinned `must_reject` years earlier, because its author had written
`// LRM-invalid syntax` in the file. The other was `must_accept`, counting against the graduation
bar as a parser defect — **for no reason except that nobody had written the comment.** The verdicts
differed by an upstream annotation, not by anything in the standard.

The lesson generalizes past this construct: **before adjudicating a family, search the pin tables
for the construct itself.** A contradiction already sitting in the table is stronger evidence than
a fresh reading of the clause, because it means the question was answered once and then re-asked
under a different name. Twenty-five rows across four suites and both LRM editions moved to
`must_reject` on that basis, and no parser byte changed.

⛔ **And the lane a task names is not the lane a construct lives in.** That sweep was written
parameterized by lane rather than hard-coded to the one the task was cut from. Run against the
second profile lane it surfaced six further rows, none of them among the first nineteen. A
diagnostic copied per lane only ever covers the lane it was pasted into; a parameterized one asks
the whole question.

#### When the verification cannot fail, it is not a verification

Reclassifying a family carries one specific risk, and it runs the opposite way from the "lowering
the bar" worry: **a file can reject for more than one reason.** Pin it on construct X and it leaves
the defect population for good — including the unrelated, real defect it *also* contains. So every
pin needs a per-file proof that the parser is stuck at X **and nowhere earlier**.

The trap is that this proof is trivially satisfiable. You already know every row rejects — that is
why they were in the population — so a check written as "confirm each row rejects near X" returns
N/N for any predicate loose enough to find X somewhere in the file, and reports nothing. **A checker
whose only observed outcome is PASS has not been tested; it has been run.**

`SV-CORPUS-GRAD.3.23` (nine rows of `enum [N:M] { … }`, a packed dimension with no base type, which
IEEE 1800-2017 A.2.2.1 cannot derive) is the worked example. Its instrument locates the construct's
header in **bytes** with comments and string literals blanked, runs the real probe, and requires
`furthest_position` to land *inside that header* — refusing the pin on ACCEPT, on stuck-earlier and
on stuck-later alike. Three controls run before any row verdict is printed:

| control | what it proves | what its absence would allow |
|---|---|---|
| a known-good reproducer classifies AT-CONSTRUCT | the anchor logic works at all | a detector matching nothing, reporting 0/N as N/N |
| the same reproducer with a defect **planted before** the construct classifies EARLIER | the checker can still **refuse** | rubber-stamping every row, masking second defects |
| a legal near-miss (`enum logic [2:0]`) yields zero matches | the detector discriminates | "the population" meaning every file containing the keyword |

Only the planted one exercises the refusing branch — the branch the whole argument rests on. It is
the same control the [frontend⟷meta-parser envelope
differential](gate-flow.md) uses, where a planted mutation must be caught exactly once at exactly
its index or the run aborts before publishing a number. **When every input you have yields the same
verdict, the control cannot be found; it has to be manufactured.**

⭐ **And the same question, asked of the whole population, has a decision-changing answer.** Before
cutting more construct leaves, `SV-CORPUS-GRAD.3.24` sized how much of the remaining defect signal
is *adjudicator* work rather than *parser* work — because getting that wrong in either direction is
expensive. Classifying every remaining row by the stage claim its own suite metadata makes:

| upstream stage claim | share | what it means for the burn-down |
|---|---|---|
| upstream asserts the unit **compiles** | 70 % | parser work (or a real upstream-vs-LRM disagreement) |
| upstream names a **post-parse** failure (elab / lint / type / name) | 25 % | parser work — the text parses by upstream's own testimony |
| upstream says only **"Unsupported"** | 5 % | testimony about the *tool*, not the language — supports neither verdict |
| upstream names a **lexical/syntactic** failure | 0 % | adjudicator work — and there is none left the recorded reasoning can surface |

**95 % is parser work.** The tempting shortcut — "these files are named `_bad`, they must be
negatives" — is refuted outright: not one of that population is reclassifiable from its name, and
trusting it would have injected over-acceptance across dozens of files.

⛔ **Two bounds on that number, both load-bearing.** A census of recorded reasoning inherits
whatever the adjudicator was blind to: it classifies the *derivation*, so a row derived wrongly is
classified confidently as what the adjudicator believed. And no metadata census can ever see the
case above, where upstream is a *tool* and the oracle is the *standard* — a row can carry
impeccable "this compiles" testimony and still be LRM-underivable. So the honest reading is
"nothing left that the recorded reasoning can surface", never "nothing left".

That bound is not theoretical: the three rows that *did* move in that leaf were invisible to the
census and were found by enumerating the upstream tool's own error vocabulary instead. Verilator's
stage test was a grep for the literal string `syntax error` — which its **lexer never emits** — so
one construct family carried two different verdicts depending on whether the message happened to
contain those two words. A prefix rule over the obvious `EOF in …` family would have been wrong too:
four of the ten spellings belong to the preprocessor or to a `-f` command file, not to the source
text. The rule that landed is an enumerated allowlist, and the guard now *refuses* when upstream
grows a spelling nobody has ruled on, rather than silently under-reporting.

⚠️ Two smaller habits fall out of the same leaf. A diagnosis instrument needs a *resolved* state —
one here ended with an unconditional "this is the mis-adjudication", so re-running it after the fix
produced a tracked artifact asserting a finding that no longer existed. And a tracked number whose
predicate lives only in prose is a claim, not a measurement: "71 rows with a `_bad` basename" was
reproducible two ways that disagree (64 vs 71), so the predicate now lives in code behind a control
that refuses unless it still reproduces the recorded figure.

#### When the standard's own productions contradict its own footnote

The `must_reject` test above — *"the text has no derivation in Annex A"* — assumes Annex A is
self-consistent. It is not always, and the difference between "the LRM does not license this" and
"the LRM licenses it in prose but forgot to write the production" decides whether a rejection is
correct behaviour or a defect. Getting that backwards is expensive in both directions: fix a
correct rejection and you inject over-acceptance; pin a real gap as `must_reject` and you make a
defect permanent.

`SV-CORPUS-GRAD.3.25` is the worked example. The queue slice `q = q[1:$]` is written five times in
IEEE 1800-2017 §7.10.4's own normative example block, and §7.10.1 says outright that the bounds of
`Q[a:b]` *"may be arbitrary integral expressions and, in particular, are not required to be
constant expressions."* Yet the production tree cannot derive it:

```ebnf
select              ::= … bit_select [ [ part_select_range ] ]
part_select_range   ::= constant_range | indexed_range
constant_range      ::= constant_expression : constant_expression
constant_primary    ::= …                        -- no `$` alternative
```

The deciding evidence is **A.8.4 footnote 42**: *"The `$` primary shall be legal only in a select
for a queue variable, in an open_value_range, …"* — a rule that only makes sense if a select can
contain `$`, which the productions above cannot produce. ⇒ the contradiction is *inside the
standard*, and the parser's rejection is a defect rather than fidelity.

Three habits generalise from it:

- **Look for a footnote before concluding "no derivation".** Annex A's constraints on where a
  construct is legal live in its numbered footnotes, not in the productions, and a footnote that
  presupposes a derivation is evidence the production is the transcription error. This is the
  opposite verdict from `SV-CORPUS-GRAD.3.23`, where the same question was asked about
  `enum [N:M]` and *nothing* in the LRM licensed it — so that one was pinned `must_reject`.
- **Let the annex's own asymmetries testify.** One line below `constant_range`, Annex A writes
  `indexed_range ::= expression +: constant_expression` against
  `constant_indexed_range ::= constant_expression +: constant_expression` — i.e. it *does*
  distinguish the select form from the constant-select form on exactly the `expression` vs
  `constant_expression` axis, and then has `part_select_range` share `constant_range` anyway.
  A local inconsistency with a neighbouring production is a much stronger signal than a reading of
  the prose.
- **Repair the minimum the contradiction licenses, and price the wider fix.** The tempting
  correction is to follow the evident intent and make the bounds full `expression`s. Measured, that
  also takes `v[a++ : b]` and `v[a inside {1,2} : 0]` from REJECT to PASS — over-acceptance in a
  parser whose default posture is strict-LRM. Adding `$` and nothing else keeps every non-`$` bound
  on the existing `constant_range` path, so no previously-accepted or previously-rejected input
  changes verdict. Profile gating matters here too: IEEE 1364-2005 has neither queues nor a `$`
  primary, so the new alternative is declared `@profiles: ["sv_2017", "sv_2023"]`, and the
  `verilog_2005` manifest staying byte-identical is what proves the gate held.

⭐ **And a partial fix is reported partial.** Of the seven corpus rows the tell collected, five
flipped to PASS; the other two kept rejecting with their `furthest_position` moved *deeper*, having
cleared the `$` bound and stopped at the next construct. That is progress and a new finding, not
a seven-row win — the leaf claims five and routes the rest.

#### When Annex A is not the whole standard — and the standard says so, in its own captions

The section above is about Annex A contradicting itself. There is a second, larger case: Annex A
being **silent** where the clause bodies are not. IEEE 1800's normative text is Annex A *plus* the
clauses, and PGEN's SystemVerilog grammar is extracted from **Annex A alone**
(`tools/extract_systemverilog_lrm_profiles.py`). Everything the clauses define and Annex A omits is
therefore invisible to the extraction pipeline by construction — not by oversight, but as a property
of where the input comes from.

⭐ **This is measurable without reading a word of prose, because IEEE labels it.** Each clause
reprints the syntax it discusses in a numbered box, and each box is captioned with its provenance:

```text
Syntax 10-5—Assignment patterns syntax (excerpt from Annex A)
Syntax 18-11—Scope randomize function syntax (not in Annex A)
```

In the 2017 edition, **304** boxes say *excerpt from Annex A* and **60** say *not in Annex A*.
`stimuli/sv/lrm_annex_a_gap_census.py` turns the second population into a worklist
(`docs/tasks/artifacts/sv_corpus_grad/lrm_annex_a_gap/`), and its shape is the useful part:

| bucket | 2017 | 2023 | what adjudicating it means |
|---|---:|---:|---|
| `system_task_function` | 110 | 109 | clause 20/21 — Annex A models these generically through `system_tf_call`, so the clause box is documentation rather than a gap |
| `compiler_directive` | 23 | 24 | clause 22 — the **preprocessor's** language, owned by the `systemverilog_preprocessor` family |
| `formal_semantics_metavariable` | 7 | 7 | Annex F rewriting rules (`P ::= strong ( R )`), not syntax at all |
| **`sv_source_syntax`** | **7** | **7** | ⭐ SystemVerilog *source* constructs — the only bucket a source parser owes anything to |

⇒ **294 clause-only productions collapse to a 7-row-per-edition question.** Six of those seven were
already reachable through a general Annex A rule. The seventh, `scope_randomize` (Syntax 18-11), was
not: `std::randomize(a, b) with { a < b; }` was rejected in **every expression**, because PGEN
renders `primary`'s call alternative as the postfix-chain rule its left-recursion lift authored, and
that rule's alternatives do not include `randomize_call`. The statement path and the constant path
both had it; only expressions did not.

⛔ **It is DIAGNOSED, not fixed, and the reason is worth reading.** The repair is written, measured
and *held* — `widen=6 narrow=0 shape=0` over 180 pinned checks, six third-party corpus files
`fail → pass`, zero the other way — because it costs **+1 917 021 rule entries** and
**+1 012 779 memo hits** with `committed` unchanged, and PGEN's parse-cost ratchet accepts a rise
only under a **coded invariant that arithmetically explains it**. Its three existing invariants are
identities over three totals, and none of them fits this shape. ⇒ the defect is pinned as a
`class=defect` reproducer (so the fix cannot land silently), the patch is tracked beside its leaf,
and the blocker is the named leaf that owns teaching the ratchet to express *"the rise is confined
to the sub-graph the change introduced"*. **A correctness fix does not get to trade away a
non-negotiable; it waits for the instrument that can price it honestly.**

⛔⛔ **And the first spelling of that fix broke something else silently, which is the more useful
half of the story.** Every part of `randomize_call` is optional, so the rule matches a *bare*
`randomize` — a legal user identifier, because Annex B reserves `rand`, `randc`, `randcase` and
`randsequence` and not `randomize`. Placed in the middle of `primary`'s alternatives it captured
exactly that. **No verdict moved.** The control file parsed before and after, the linter was
identical on all nine counters, and both fixed reproducers were green. The only instrument that saw
it was the **AST axis**: `shape=2`, with the typed-AST diff naming the node (`hierarchical` 4 → 3,
`scope_randomize` 0 → 1). Making it `primary`'s **last** alternative returns the control's tree to
byte-identical and is cheaper besides. ⇒ *the guard you write for the direction you do not expect to
break is the one that earns its keep*, and it is why the reproducer manifest pins **arms**, not just
verdicts.

Three habits generalise from this one too:

- **Check whether the artifact already states the property before building a detector for it.** The
  first two attempts here diffed Annex A against the clause boxes textually and produced 172
  "divergences" that were overwhelmingly PDF-conversion noise. The caption was one `grep` away.
- **Never key a census on a converted document's file names.** The PDF→markdown splitter titles
  sections from nearby lines: the 2017 Annex A holds **zero** productions in the file named for it
  and **736** in `section-41-data-read-api.md`. The clause number IEEE prints inside the caption is
  the only stable key.
- **A worklist generator must refuse rather than guess.** The census exits non-zero on any
  clause-only production it cannot attach a caption to. That is what surfaced the Annex F
  metavariables on its first run instead of shipping them as seven mystery productions.

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
shadow freedom plus fixed-terminal-prefix shadow freedom in its sound (`ordered`-policy) sub-case,
**structural reachability** ("no unreachable rules", multi-entry-aware), and **no undefined
references** — all hard gates — plus **attribute non-circularity**, which holds by construction
(the annotation language is synthesized-only).

The **undefined-reference** gate (`undefined_references=N (error)`, since
`UNDEFINED-REF-DIAGNOSTICS.2`, 2026-07-06) is the *other half* of "no useless symbols": a rule that
references a rule that is never **defined**. Before it, codegen silently synthesized a
never-matching stub for such a reference — the grammar compiled clean and every parse rejected at
`furthest_position=0` with nothing pointing at the cause (a real session-#47 incident: a suite
grammar accidentally omitted its `word` rule). Now `--lint-grammar` names the exact referencing
rule and missing name, and `--generate-parser` additionally prints an unconditional warning at the
moment it emits a stub. Two deliberate design points: the check runs against the **unfiltered**
grammar (codegen always compiles the *full* grammar — profile selection is a runtime guard — so
`@profiles`-gated definitions deliberately stripped from a filtered lint view must not read as
dangling), and the allowlist of intentionally-undefined names is **codegen's own native-builtin
set** (`builtin_any_char`, `builtin_ascii_char`), consumed
from the single `NATIVE_UNRESOLVED_REFERENCE_BUILTINS` constant and locked to the dispatch by an
oracle test — the linter can never drift from what codegen actually synthesizes.

That coupling has a sharp edge worth stating plainly, because it cost a whole leaf's evidence
(`LANG-CAPABILITY-AUDIT.10.1`): **every name on that list is also removed from this check's sight.**
Sharing the const is what stops linter and codegen drifting apart, but it means allowlisting a name
*for codegen* necessarily blinds the diagnostic *to that name* — so `undefined_references=0` says
"no dangling reference **outside the allowlist**", not "this grammar is sound". At the time of that
audit `grammars/ebnf.ebnf` reported 0 while carrying three live references to a rule nothing
defines.

**All three non-primitives are now gone, and the two statements have converged.** `true` and
`false` compiled to matchers that always succeeded and consumed nothing while being invisible
here; both were removed in `LANG-CAPABILITY-AUDIT.10.4` (measured on one probe grammar:
`undefined_references` 0 → 2, with the linter naming both). `semantic_annotation` was the third:
it compiled to an `@`-to-end-of-line slurp that existed only to stand in for the meta-grammar's
broken `include`, and it was a *correctness ceiling* as well as a blind spot — a line-bounded
matcher cannot express the multi-line `@dispatch: { … }` payload, which is what held self-hosting
at 11/12 (see the self-hosting section below). `LANG-CAPABILITY-AUDIT.10.2` gave `ebnf.ebnf` its
own `semantic_annotation` rule and `.10.3` removed the allowlist entry and the synthesized
matcher, measured on the same probe grammar (`undefined_references` 2 → 3, the linter naming it)
and — the sharper oracle — leaving `generated/ebnf.rs` **byte-identical**, which is what proves
the fallback was genuinely unreached rather than merely believed to be.

The list is now **two names**, both `builtin_`-prefixed, and the standing rule is that a name goes
on it only when it genuinely is a codegen primitive — never to quiet a diagnostic. The payoff is
that `grammars/ebnf.ebnf` still reports `undefined_references=0`, but now *because the grammar is
sound* rather than because a name was allowlisted — a distinction the check could not previously
make about itself.

It *also* observes — but no longer as a *verdict* — an **earlier alternative that always succeeds**.
In an ordered choice `a | b`, if `a` can never fail (it is `e?`, `e*`, an all-optional sequence, or a
reference to such a rule), a *pure* PEG would commit to `a` and leave `b` unreachable. PGEN once
gated on exactly that reasoning; the 2026-07-05 audit (see the Correction under the worked example)
found it **unsound for PGEN's backtracking / longest-match engine** — after `a` succeeds by matching
*empty* and then fails downstream, the engine re-enters `b`, so `b` is *live*. It flagged
proven-live branches (the `interconnect` port form) as dead. So as of **A2.2** the always-succeeds
*shadowing verdict was removed*: it no longer emits a "shadowed/unreachable" finding and there is no
gate-promotion path. What survives is a **non-verdict note** — `--lint-grammar` reports
`always_succeeds_alternatives=N (note)` and prints each as a `[note]` that says explicitly *"not a
deadness verdict"* — because a nullable earlier alternative is still a genuine grammar smell (it was
the tool that surfaced the dropped-delimiter extraction bugs above). On SystemVerilog the former 8
always-matches *warnings* are now 6 informational *notes*; every fully-certified grammar is at 0.
(This is distinct from nullability — a lookahead `&e`/`!e` consumes nothing but *can* fail, so it
never triggers the note; no false positives on that axis.)

A note on what is **left out on purpose:** the *general* "FIRST-set domination" heuristic — "if
everything `b` could start with, `a` could also start with, then `b` is dead" — is **unsound for
PEG** and is intentionally not implemented. `a` might match the first token and then fail later, in
which case PGEN *does* backtrack and try `b`, so `b` is live. That is the *same* argument that
retired the always-succeeds verdict. Implementing either as a verdict would falsely accuse live
branches of being dead — the opposite of an honest linter.

The same argument caught one more verdict — **fixed-terminal-prefix itself** (**A2.3**,
2026-07-07). The classical claim "`a | a b` — the earlier alternative is a fixed-terminal prefix of
the later, PEG commits, so `a b` is dead" implicitly assumes a *first-success-commit* selection
semantics. PGEN's tournament has three (`@branch_policy`): the **default `longest_match`** tries
*every* alternative and keeps the longest match — the parse harness proved the engine **selects**
the supposedly-dead later alternative on `a | a b` over input `ab` (its own trace:
`🏁 selected branch 2/2 … branch_policy=longest_match`) while the old unconditional verdict
hard-failed the same live grammar. `priority_first` likewise lets a later alternative win. Only
`ordered` genuinely commits to the first success. So the fixed-prefix verdict is now
**policy-conditional**: it fires (and hard-gates, with its proof certificate) only for a rule whose
effective `@branch_policy` is `ordered` and which carries no branch-phase `@predicate` (a branch
predicate can block the earlier alternative after it matches, reviving the later one). Under
`longest_match`/`priority_first` the pattern is the normal longest-match idiom — live, correct, and
deliberately not even a note. The certificate checker re-derives the policy condition too: a
fixed-prefix certificate presented for a `longest_match` rule is *rejected as false*, not
re-verified on structure alone.

The **exact-duplicate** verdict turned out to hide the *same* assumption (**A2.4**, 2026-07-07). "The
later of two identical alternatives is unreachable" is true only if the earlier twin always beats it —
and in PGEN's tournament that depends on the tie-break. The parse harness proved three ways the engine
**selects the later twin** on `r := "a" | "a"` over input `a` (each its own `🏁 selected branch 2/2`
trace): under **`@associativity: right`** an equal-length/equal-priority tie picks the *later* branch;
under a **later-higher `@priority`** the later twin wins outright (priority is compared before
associativity); and under **`@deterministic_group`** the tournament's evaluation order is *rotated* by a
group-keyed offset, so which twin is the incumbent flips. A fourth case, **`@associativity: nonassoc`**,
is subtler: equal-priority twins *always* tie, and a nonassoc tie **fails the whole choice** — so neither
twin is ever selected (the twins *reject* the very input their body matches, while the deduplicated
control accepts it). The later twin is still unreachable there, but *removing one duplicate would change
acceptance* (the survivor then wins), so that verdict carries its own message — "restructure
deliberately," not "merge or remove." The duplicate verdict (and its proof certificate) now reads the
rule's effective `@associativity` / `@priority` / `@deterministic_group` — through the **same** shared
resolver codegen's tournament uses, so the two can never drift — and fires only where the earlier twin
provably wins (or the nonassoc tie provably fails the choice). A2.4-D also tightened the A2.3
fixed-prefix condition: partition rotation reorders `ordered` first-success too, so that verdict now
additionally requires no `@deterministic_group`.

So the *gating* shadow forms are: **exact-duplicate** (where the earlier twin provably wins the
tie, or a `nonassoc` tie provably fails the choice) and **fixed-terminal-prefix under `ordered`**
(no branch-phase predicate, no partition rotation).

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

### What the deterministic number then revealed

Making the measurement stable was worth doing for its own sake, but the first thing a trustworthy
number bought was an honest diagnosis. Applying the attribution rule above to SystemVerilog's
closed-loop residual (`SV-EXH-PROOF.7.4.6.8`) produced three results worth recording, because each
is the *opposite* of what the campaign had been assuming.

**The failure mode that dominated for months is gone.** Every earlier attempt to drive the residual
down was fighting generation *timeouts* — 91–99 % of uncovered targets. Under the deterministic
step budget, together with a backtrack-free witness constructor and an allocation fix, that class
measures **zero**. What remains is a different thing entirely: a witness *was* generated, without
error, and the target still was not credited. The work changed shape, so the tools aimed at the old
shape stopped being the right ones.

**The residual is not a fog — it is three named defects.** The remaining targets partition exactly
and totally, and near-identically across both LRM profiles, into: the transitive cone of one rule
(`property_expr`, the SVA property cascade); inner ordered choices sitting under an optional or
repeat group that never expands; and one store-gated rule whose predicate cannot be satisfied from
an empty semantic store. That is the attribution rule doing its job — a bounded ticket list, not a
shrug. Notably the largest class is *one* defect wearing forty faces: in the rule at its centre,
every alternative that re-enters the recursion cycle is uncovered and every alternative that does
not is covered, with no exceptions in either direction.

**And a completeness number nobody is allowed to accept can still drift.** The residual rose while
the lane was parked, because the gate *reports* it and never *compares* it: the count is echoed into
the summary and comparison stops there, and an uncovered target makes a family "Mostly Done" rather
than making the gate fail. So a grammar change can add coverage obligations faster than they are
discharged and every gate still passes. This is the same lesson the project keeps relearning about
instruments — **an instrument that reports without refusing will eventually report something false**
— and the fix is the two-sided ratchet PGEN already uses elsewhere: fail above the pinned ceiling
*and* fail below it, so wins are banked instead of quietly lost.

### EBNF self-hosting: the meta-grammar models itself

The EBNF *meta-grammar* (`grammars/ebnf.ebnf`) is held to the same gen↔parse duality as every
parser family: the parser generated *from* it (`generated/ebnf.rs`) must be able to parse the very
grammar files PGEN ships. Every EBNF-format construct a shipped grammar uses — per-branch return
annotations, the `**` flatten-spread, dotted `$ref` property access, the `::N*` extraction-spread,
`null` literals, and so on — must therefore be modelled *in* `ebnf.ebnf`, or the generated EBNF
parser cannot self-parse that grammar. This is the EBNF meta-grammar lockstep rule: a new EBNF
feature is not "done" until it is also expressed in the meta-grammar.

Self-hosting is measured by the `ebnf_dual_run_diff` tool (the generated EBNF parser run over each
grammar file) and gated, for the three tracked grammars `ebnf`/`json`/`regex`, by `make -C rust
ebnf_frontend_dual_run_gate`. The live count is **12 of 12 tracked grammars — 12/12**, and
this time for the right reason. The three raw IEEE-LRM *extraction snapshots*
(`systemverilog_2017/2023_lrm_extracted`, `verilog_2005_lrm_extracted`) are traceability artifacts,
not part of the tracked self-hosting set. The live count and per-gap history are tracked in
`docs/book/src/roadmap-and-live-status.md` and the `GRAMMAR-WELLFORMED` task tree (`H.13`/`H.14`).

> ⛔ **This number went DOWN before it came back, and both moves were the measurement getting
> honest.** From 2026-06-25 (`GRAMMAR-WELLFORMED.H.14.3`, release `1.0.147`) this chapter
> published **12/12**. `LANG-CAPABILITY-AUDIT.10.5` measured that claim and found it false:
> `grammars/ebnf.ebnf`'s `single_quoted_string` regex opened a `'` and **never closed it**, so a
> lone quote swallowed input to the next `'`/`\` — across newlines. `regex.ebnf:1048`'s
> `class_safe_special` is written in **29 single-quoted terminals**, not one of which the broken
> rule could close, so the parse desynchronized there and ate the two multi-line `@dispatch`/
> `@dispatch_table` payload blocks 200 lines further down — and the parser reported a clean parse
> over all 157,319 bytes. The twin rule one line above (`double_quoted_string`) always had its
> closing quote and always rejected the same shapes — which is how the typo was pinned. Closing
> the quote dropped the honest count to **11/12** and named the real gap: **the meta-grammar
> could not express a multi-line annotation payload**, because `ebnf.ebnf` defined no
> `semantic_annotation` rule at all and codegen substituted an `@`-to-end-of-line slurp.
>
> ✅ **`LANG-CAPABILITY-AUDIT.10.2` closed that gap and the count is 12/12 again.** `ebnf.ebnf`
> now defines `semantic_annotation` itself, as a **delimiter with an opaque payload** — `@` +
> name + `:` + either a brace-balanced body (which may span lines and nest) or the rest of the
> line. That mirrors what the authoritative hand-written frontend already does
> (`ebnf_frontend.rs:1410` finds the top-level colon and carries the payload as an opaque
> `String`), and it leaves the payload's real specification where it belongs — with the
> annotation parsers. ⭐ The general lesson, and the reason it is written here rather than buried
> in a task tree: **a proof surface that cannot fail is not reporting, it is agreeing** — the
> `regex` row of `ebnf_frontend_dual_run_gate` was green *because of* a defect, and only removing
> the defect could say so.

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
