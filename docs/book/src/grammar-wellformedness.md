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
a single JSON value* (`ParseContent::Json`), which has no child nodes. So an AST walk stops at the
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
ebnf_frontend_dual_run_gate`. As of 2026-06-25 (`GRAMMAR-WELLFORMED.H.14.3`, release `1.0.147`), the
generated EBNF parser self-parses **all 12 tracked grammars — 12/12**. The final gap was a duplicate
`->` return-annotation defect in `systemverilog.ebnf` (a single rule may carry only one rule-level
return annotation), removed in `1.0.147`; that same fix restored the UDP truth-table entry AST shape
(ledger `SV-0009`). The three raw IEEE-LRM *extraction snapshots*
(`systemverilog_2017/2023_lrm_extracted`, `verilog_2005_lrm_extracted`) are traceability artifacts,
not part of the tracked self-hosting set. The live count and per-gap history are tracked in
`LIVE_ACHIEVEMENT_STATUS.md` and the `GRAMMAR-WELLFORMED` task tree (`H.13`/`H.14`).

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
