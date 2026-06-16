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
