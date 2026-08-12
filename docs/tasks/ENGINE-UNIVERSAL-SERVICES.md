# ENGINE-UNIVERSAL-SERVICES — what PGEN's engine does FOR every EBNF, catalogued, so we can judge whether it is enough

## Metadata

- Tree ID: `ENGINE-UNIVERSAL-SERVICES`
- Status: `active` (director-commissioned 2026-08-11, session #215)
- Roadmap lane: platform — EBNF-authoring ergonomics and the engine/grammar responsibility boundary
- Created: `2026-08-11`
- Opened by: `GRAMMAR-WELLFORMED.A2.5` / `SV-CORPUS-GRAD.13c.2a.2`, which found the boundary being
  violated in the worst way — silently.

## The frame (director, 2026-08-11 — binding)

> *"PGEN engine shall handle all things that are objectively shared, common to all EBNFs. The EBNF
> shall carry only things that are really specific to each language they describe, that's the idea.
> Hardcoding LR elimination in EBNF is as a consequence a bad, non-sota, non-signoff [decision]."*
>
> *"EBNF authors should NOT have to worry about LR elimination, they shall simply not care. This
> sort of thing shall be handled efficiently in the background by PGEN engine."*
>
> *"At some point, we need to thoroughly document what are the things the PGEN engine does that
> benefit all EBNFs, we shall describe each of them in details so that we can reflect, and decide if
> that's enough or we should add more features that will make EBNF authoring a lot easier, more
> flexible."*

This gives PGEN a **responsibility boundary** with a testable definition:

> A concern belongs to the **ENGINE** iff it is objectively shared by all EBNFs — i.e. it is a
> property of *parsing*, not of the *language being parsed*. A concern belongs to the **GRAMMAR**
> iff it is a fact about that specific language.

⛔ **The violation to hunt is a grammar that hand-compiles around an engine gap**, because it is
invisible: the grammar still parses, the gates still pass, and the only symptom is that the grammar
no longer transcribes its standard. `A2.5` is the worked example — three separate SystemVerilog
rules were hand-flattened over months, each discovered by accident, while `--lint-grammar` reported
the underlying gap as *"handled by PGEN"*.

## Goal (the tree's single deliverable)

A **complete, evidence-backed catalogue** of the engine's universal services — one entry per
service, each stating WHAT it does for the author, WHAT the author would otherwise have to write by
hand, and HOW it is proven to work — published in the live book, plus an explicit **gap list** of
services PGEN does *not* yet provide, priced, so the director can decide what to add.

⛔ **A catalogue that only lists what exists is half the deliverable and the less useful half.** The
question is *"is that enough?"*, and that cannot be answered without naming the absences.

## Ground rules

1. **Every entry cites a proof.** A service is listed only with the gate, suite case or measurement
   that demonstrates it — the same bar as any other claim here. A service nobody can demonstrate is
   a gap entry, not a catalogue entry.
2. **Every entry names its escape hatch, if any.** If an author can still be forced to hand-write
   around the service (a `@sample` hint, a manual flattening), that is part of the entry — it is
   where the abstraction leaks.
3. **The census is DERIVED, not asserted.** The count of universal services and of grammars relying
   on each is computed from the code and grammars, never stored as prose that can rot
   (`docs/DERIVED_STATE_CONTAINMENT.md`).
4. ⛔ **Scar-tissue is evidence.** Any hand-written workaround still present in a shipped grammar is
   a catalogue finding, not an implementation detail. It is the measurable form of "the engine did
   not do enough".

## Leaves

### `.1` — the INVENTORY: enumerate the engine's universal services from the code (`todo`)

Read the pipeline end to end and enumerate what the engine does that no grammar declares. Known
starting set, to be confirmed and completed rather than trusted:

| candidate service | where | what the author would otherwise hand-write |
|---|---|---|
| **left-recursion elimination**, direct AND indirect | `ast_pipeline/mod.rs` `normalize_direct_left_recursive_alternatives` + `detect_left_recursive_chain_plan` | the `seed ( continuation )*` rewrite, per rule, losing the standard's binary AST |
| runtime cycle-breaking / recursion guard | generated parser prologue | nothing — but it must not *substitute* for elimination (`A2.5`) |
| packrat memoization, taint-gated on store writes | shared runtime | manual result caching; correctness under semantic facts |
| branch tournament + `@branch_policy` (longest-match default) | `generate_or_logic` | ordering alternatives by hand and hoping |
| backtracking + `furthest_position` error attribution | shared runtime | error reporting |
| layout/trivia skipping + per-introducer comment-arm suppression | codegen layout policy | threading `trivia` through every rule |
| profile gating (`@profiles`, `@default_profile`, `@profile_alias`) | codegen | duplicating the whole grammar per dialect |
| return-annotation AST shaping | codegen | writing an AST builder |
| stimuli generation + certificate coverage | generator | writing a test corpus by hand |
| bounded quantifiers `{N}`/`{N,M}`/`{N,}`/`{,M}` | codegen | manual repetition unrolling |
| lexical follow-restrictions `[> … ]` | codegen | hand-written negative lookahead |

⚠️ The table above is a STARTING POINT written from what this session touched. `.1` is not done
until it is derived from the code rather than from memory.

### `.2` — the SCAR-TISSUE census: find every hand-compiled workaround in the shipped grammars (`todo`)

Measure, per grammar, the constructs that exist only because the engine did not do something. The
first two are already known and are the seed of the method:

- **hand-flattened left recursion** — `systemverilog.ebnf`'s `seq_and_expr` / `seq_and_tail` pair
  (and any sibling `*_tail` / `*_expr` cascades) are a manual `seed ( continuation )*`; with `A2.5`
  landed they can go back to transcribing IEEE 1800-2017 A.2.10 directly. ⛔ Sized, not assumed: the
  gen-AST sweep in [[a-directly-left-recursive-alternative-inside-a-choice-is-dead-code]] found
  **9 dead alternatives across 3 rules** in the raw Annex A transcription
  (`systemverilog_lrm_profiled_wrapper`) versus **1** in the hand-massaged shipped grammar — the
  difference IS the scar tissue.
- **`@sample` / `@probe_sample` hints that exist to rescue a witness** the generator cannot plan
  (`SV-CORPUS-GRAD.13c.2a.2` needed one). Each is a place the generator is not strong enough.

### `.3` — the GAP list: what the engine does NOT do, priced (`todo`)

Candidate gaps already visible; to be completed by `.1`/`.2`:

- **operator precedence / associativity declaration.** Standards write precedence as a cascade of a
  dozen rules (`rtl_const_expr` has ~16 levels, un-generatable within the bounded stimuli ladder —
  `PARSE-HARNESS.5.5`). An engine-level precedence-climbing directive would delete that cascade from
  every expression grammar PGEN will ever host.
- ⭐ **a BOUNDED-REFUSAL ceiling on nesting depth — and the evidence that it is an ENGINE gap, not a
  regex one, was measured by accident during `.8` (2026-08-11).** `regex` has one
  (`REGEX_MAX_NESTING_DEPTH = 250`, `RGX-0085.1`) — but it was hand-added to the *regex-specific*
  embedding path, so no other family inherited it. The cost of not having it is measurable:
  `cargo test --lib` was left running for **68 minutes**, then on a second, independent run for
  **2 h 27 m** before being abandoned, and `/usr/bin/sample` on the test binary — taken twice, hours
  apart, with identical stacks — showed the only two busy threads were
  `parser_embedding_systemverilog_deep_nesting_…` and
  `parser_embedding_vhdl_deep_nesting_…` (every other test had finished), both grinding through
  `cascade_match_expression → relation → simple_expression → term → factor → primary` on a
  2000-deep paren nest. Those tests assert *"a clean diagnostic, not a process abort"* — which they
  do eventually satisfy — but they bound the OUTCOME and not the TIME, so the parser refuses by
  exhaustion rather than by a fast located refusal.
  ⛔ **Owed before this is priced, not assumed:** (1) re-measure in RELEASE, since the observation is
  from a debug build and debug is 10–50× slower here — a release parser may be entirely fine;
  (2) if it reproduces, decide whether the ceiling belongs to the ENGINE — a bound every family
  inherits — rather than to one family's embedding wrapper. ⛔ The SPELLING is deliberately left
  un-named until that pricing; see the PRIOR ART block below for what already exists and what
  constrains the design. ⚠️⚠️ **The developer-flow cost is no longer hypothetical and now has a dated
  casualty**: because these two tests set the suite's wall time, the lib suite is effectively never
  completed — and `LANG-CAPABILITY-AUDIT.10.15` is a test that sat **RED for 146 commits** behind
  them, found only when a run was finally driven to a verdict with them skipped
  (`1074 passed / 1 failed`). ⇒ read the two findings together: the missing ceiling is not only a
  robustness question, it is why a red suite stopped being a signal.
  #### PRIOR ART — the nesting-depth bound (searched 2026-08-11, before naming any surface)

  ⭐ **The search found real prior art, which is the success case**: this is not "invent a primitive",
  it is "one family already has it, hand-wired, and the constraint on how it must be built is already
  decided". Recorded per [[feedback_read_prior_art_before_designing]].

  1. **`grammars/ebnf.ebnf` — is it already EXPRESSIBLE?** **No.** The meta-grammar has no
     depth/nesting/recursion vocabulary at all; `grep -niE "depth|nesting|recursion|max_"` returns only
     `max_count` (the `{,10}` quantifier bound) and one comment about brace-depth-aware lexing. So an
     author cannot declare a nesting bound today, by any spelling.
  2. **`docs/decisions/` — already decided or designed?** **The CONSTRAINT is decided, the SURFACE is
     not.** [[feedback_recursion_ceiling_must_bound_real_stack]] (from `PGEN-RGX-0085`) rules that a
     depth ceiling which returns a clean error is useless if its bound exceeds the stack the recursion
     actually runs on — the OS guard page faults first — so it must be bounded to the real stack or
     pre-checked at the integration boundary *before* recursion starts. ⇒ any engine-level form
     inherits that ruling; it is not a free design.
  3. **`docs/tasks/` — does a tree already own it?** `RGX-0085` owns and CLOSED the regex-family
     implementation (`REGEX_MAX_NESTING_DEPTH = 250` + `GENERATED_REGEX_INLINE_DEPTH_THRESHOLD` 16→4,
     pre-parse, in `run_generated_regex_on_dedicated_stack`). It is deliberately scoped to the regex
     embedding path; no tree owns a general form.
  4. **`docs/book/` — a designed-but-unbuilt form documented?** Only the SHIPPED regex one
     (`docs/regex_parser_book/src/rules-groups.md` *"Parenthesis nesting limit"*, released `1.1.77`,
     ledger `REGEX-0084`). Nothing describes a general or declarable form.

  ⇒ **the gap is real and its shape is now constrained rather than invented**: what is missing is a
  bound every family inherits, satisfying the `RGX-0085` ruling. Whether that is grammar-declared, an
  engine default, or an integration-boundary pre-check is **the open design question `.3` must price —
  deliberately NOT named here**, because naming a spelling before the pricing is exactly the failure
  the prior-art doctrine exists to stop.

- **error recovery / resync**, so a parser can report more than the first failure.
- **incremental / re-parse**, for editor-facing consumers.
- **left-factoring** of common prefixes across alternatives — the sibling transform to LR
  elimination, and the same "author should not care" argument applies.

### `.4` — the published catalogue + the responsibility-boundary rule (`todo`)

A live-book chapter carrying `.1`/`.2`/`.3`, plus the boundary rule stated above promoted to a
decision record, so a future grammar edit that hand-compiles around the engine is a **reviewable
defect** rather than an invisible one.

⭐ Wire it to something mechanical if possible: `.2`'s scar-tissue census is a script, and a script
that can be re-run is the difference between a catalogue and a claim.

### `.5` — the ENGINE ANATOMY: EBNF → Rust, stage by stage, with a CONTRACT per stage (`todo`, director-commissioned 2026-08-11)

> *"We need to be able to clearly understand the structure of the PGEN engine (parser generator +
> stimuli generator) from EBNF to Rust code … see where that structure is strong, where it might be
> a bit weak, which features are missing if any … anyone shall be able to reflect on the internal
> structure of the PGEN engine."*

⛔ **This is NOT a code tour, and a code tour is the failure mode to avoid.** The deliverable is the
pipeline as a sequence of **named stages, each publishing a CONTRACT**: what it guarantees for
*every* grammar, what it explicitly does not, and what it hands the next stage. Every universal
service from `.1` is then **attributed to a stage**.

⭐ **Why that shape, and it is not an aesthetic preference — it is this tree's own root cause.**
`A2.5`'s defect survived for months because the claim *"left recursion is handled by PGEN"* was
written **by hand, in the linter**, while the truth lived in `detect_left_recursive_chain_plan` two
files away. A capability claim that sits next to a *consumer* instead of next to the *stage that
implements it* is unfalsifiable, and unfalsifiable claims rot in the passing direction. Measured
cost of that specific rot: an agent with the full toolbox needed four probes plus
`grep -c _pgen_lr_chain` to discover that "handles left recursion" meant one of its two shapes.

Stages to characterize (starting list, to be confirmed against the code — the front half is the
parser generator, the back half the stimuli generator, and they share the middle):

1. **frontend** — `.ebnf` text → `raw_ast` token envelope (`ebnf_frontend.rs`; and its generated
   twin from `grammars/ebnf.ebnf`, held equivalent by the envelope differential, TOOLBOX 1.9)
2. **normalization / transform** — `raw_ast` → gen-AST (`ast_pipeline/mod.rs
   transform_from_raw_ast`): rule extraction, annotation binding, **left-recursion normalization +
   elimination**. ⭐ The stage `A2.5` lived in.
3. **analysis** — well-formedness, profiles, fusibility, reach (`grammar_wellformedness.rs`)
4. **parser codegen** — gen-AST → Rust (`ast_based_generator.rs`): branch tournament, layout policy,
   memo, recursion guard, AST shaping from return annotations
5. **stimuli generation** — gen-AST → samples (`stimuli_generator.rs`): witness planning, directed
   generation, certificate coverage
6. **runtime** — the shared engine the generated code calls (memo, store, scopes, trace)

Per stage, the entry must answer: **what does an author get for free here? what can still force
them to hand-write around it? how is the guarantee proven?**

### `.6` — TECHNIQUES: how a new universal feature gets added without breaking the others (`todo`)

The director asked not only *which* features are missing but *how* to add them. Write the method,
grounded in the ones this repo has actually done:

- **normalize into an existing path rather than adding a second one** — `A2.5`'s pre-pass is the
  worked example: the direct LR shape is rewritten into the indirect shape, so all the tested
  planner + `_pgen_lr_chain` machinery is reused and there is no second implementation to keep in
  sync. ⭐ This is the cheapest and safest shape for a new engine feature and should be the default
  technique the chapter recommends.
- **make one function the single source of truth for a decision shared by two consumers** — `A2.3`'s
  `effective_rule_branch_policy`, shared by codegen's tournament and the linter's verdict.
- **prove it on synthetic isolating grammars, not on a shipped one** — the combinator (TOOLBOX 1.7)
  and semantic (1.8) suites; `A2.5` added two cases and the second one exists purely so a
  half-correct implementation cannot pass.
- **declare it, do not hardcode it** — the `@whitespace_sensitive` / `@default_profile` /
  `@profile_alias` precedent, where engine-side name literals became grammar-declared directives.
- ⛔ **and the anti-technique:** hand-compiling the transform into a grammar. That is what this tree
  exists to make reviewable.

### `.7` — the FEATURE TAXONOMY: enumerate the *types* of engine feature ahead of demand, priced by when they cost (`todo`, director-commissioned 2026-08-11)

> *"We need to be able to list the TYPES of features that can be added to the PGEN engine before
> they are needed by any specific language. I am not sure that is even possible … PGEN [should] be
> extensible (feature-wise) but still be able to run lightning, extremely fast."*

⭐ **It is possible, and the reason is structural: a feature can only act at one of the pipeline's
insertion points, and `.5` enumerates those.** So the design space is the cross product of *where*
a feature acts with *what it changes* — both finite. That is what makes the list derivable ahead of
any language asking for it.

**Axis A — where it acts** (the `.5` stages): frontend syntax · gen-AST rewrite · analysis/verdict ·
parser codegen · stimuli generation · runtime.

**Axis B — what it changes**: the LANGUAGE accepted · the AST produced · the WORK done at parse time
· only what the AUTHOR has to write.

⭐⭐ **Axis C is the one that answers the speed question, and it is the load-bearing idea of this
leaf: WHEN does the feature cost?**

| class | cost at parse time | why | examples |
|---|---|---|---|
| **gen-AST → gen-AST rewrite** | **ZERO, by construction** | the stage runs at generation time and its output is still plain generated Rust; the runtime cannot tell the difference | left-recursion normalization (`A2.5`, landed — added a compile-time pass and *not one runtime instruction*), left-factoring, precedence climbing, **parametric-rule expansion** (`LANG-CAPABILITY-AUDIT` P2-5), quantifier unrolling |
| **codegen strategy** | zero-to-negative | changes the shape of the emitted code, not the work it does; can be a speed *win* | derived DFA scanner / rule fusion (`--report-fusibility-census`), merged choice sites, cascade fusion |
| **analysis-only** | zero | produces a verdict or an artifact, emits no parser code | linter contract items, certificate coverage, profile gating |
| **runtime service** | **REAL, and must be measured** | the generated parser calls into it on every relevant input | memoization, semantic store + predicates, error recovery, incremental reparse, tracing |

⇒ **The design rule this yields, and the answer to "extensible but lightning fast":** prefer the
rewrite class — it is free by construction — and make anything in the runtime class **opt-in and
priced**. PGEN can grow indefinitely along axis-C rows 1–3 without touching peak speed, which is a
[[project_north_star]] non-negotiable (costs are REJECTED, not traded).

⛔ **The honest caveat, so this is not read as a licence:** "free at parse time" is not "free". A
rewrite still costs generation time, generated-code size, and — the one that bit `A2.5` — it can
move the **AST shape**, which is a downstream contract. `.7` must price those three for each
candidate, not just the runtime column.

**Owed by this leaf:** the populated cross product, each cell either naming a shipped feature, a
priced candidate, or an argued-empty; plus the recovered backlog of ideas already discussed in
earlier sessions (parametric rules is the one the director recalls by name — others are to be
recovered from `LANG-CAPABILITY-AUDIT.11`'s horizon register and `docs/decisions/`, not from memory).

### `.8` — ⭐⭐ THE LR-ELIMINATION FOLD LEAKED AN INTERNAL REPRESENTATION INTO THE TYPED AST (`done` — `PGEN-ENGINE-UNIVERSAL-SERVICES-0002`, 2026-08-11 session #216; found by `GRAMMAR-WELLFORMED.A2.5`, which it **UNBLOCKS**)

⛔ **This is a PRE-EXISTING engine gap, not a regression, and it was found by accident — again.**
After `A2.5` made SystemVerilog's `select_expression` actually parse its continuations, the emitted
AST for a `bins_selection.select` is:

```jsonc
{"initial": …, "suffixes": …, "type": "_pgen_lr_chain", "wrapper_specs": …}
```

— the eliminator's own internal chain representation — instead of the author's
`{kind: "and", lhs: $1, rhs: $3}`. The data is all present (`initial` carries the seed's annotation,
`suffixes` the continuations, `wrapper_specs` the per-alternative templates); it is simply **not
folded back into the declared shape**.

**Measured as pre-existing, not introduced:** `grep -c _pgen_lr_chain generated/return_annotation_parser.rs`
→ **10** on a shipped, fully-certified grammar. Every LR-eliminated rule in every grammar has been
emitting this. It contradicts PGEN's own second doctrine — *"every generated parser returns an AST;
return annotations shape that AST"* (`README.md`) — for exactly the rules the engine transforms.

⚠️ **Why the existing gates did not catch it, which is the more valuable half of this finding:**
the differential suites assert the interpreter and the generated parser are **byte-identical to each
other**. Both fold identically — i.e. both leak identically — so agreement is total and the suites
are green. ⭐ *Two implementations agreeing is not evidence either one is right.* Nothing in the gate
set compares the emitted AST against the **declared return annotation**, which is the missing
oracle and is worth more than this single fix
([[a-negative-control-can-disable-the-assertion-it-is-testing]] is the sibling lesson).

**Owed:** fold `_pgen_lr_chain` into the declared per-alternative shape at AST construction, so an
LR-eliminated rule is indistinguishable from a hand-written one — the left-nested `lhs`/`rhs` the
standard's binary production describes. Then add the missing oracle: an
annotation-vs-emitted-AST conformance check, so "the AST matches what the grammar declared" is a
gate rather than an assumption.

⛔ **Until this lands, `A2.5`'s engine fix must not ship**: it would change SystemVerilog's
`bins_selection.select` from one of eight declared kinds to a chain blob, which is a contract
regression even though it is also a parse-correctness win.

#### DIAGNOSIS — the two mechanisms, both tool-located (session #216)

**1. WHY the AST is wrong — WHERE the fold is missing.**
`rewrite_lr_chain_annotations` (`rust/src/ast_pipeline/mod.rs`, steps 4–5) replaces an LR-eliminated
rule's declared per-branch annotations with a synthetic record and hoists the author's templates into
its `wrapper_specs` field. **No stage ever folded that record back.** The only fold in the tree was
`unified_return_ast.rs::parse_typed_lr_chain` — a *reader-side* path the pipeline uses when it
re-reads the `return_annotation` parser's own output, in the `UnifiedReturnAST` vocabulary, not in the
emitted-AST vocabulary and not reachable from any generated parser.

Reproduced end-to-end on a shipped, `fully_certified` grammar
(`./rust/target/debug/parseability_probe --parse-dump-ast-pretty return_annotation ann.txt out.json`,
input `$1.a.b`), `wrapper_specs` elided:

```jsonc
{ "initial": { "base": {"index": 1, "type": "positional"}, "property": "a", "type": "property_access" },
  "suffixes": [ {"alt_index": 0, "captures": [".", "b"], "type": "_pgen_lr_chain_alt"} ],
  "type": "_pgen_lr_chain",
  "wrapper_specs": "<439 bytes elided>" }
```

The declared shape — `property_access_expression := accessor_base '.' identifier ->
{type: "property_access", base: $1, property: $3}` — is present *as a field of an engine record*
instead of *being* the value, and the second `.b` was never applied at all.

**Blast radius, measured rather than assumed** (`grep -c '"_pgen_lr_chain"' generated/*_parser.rs`,
all 11 generated parsers): `return_annotation` 6 sites, `semantic_annotation` 10 sites, **every other
grammar 0** — one LR-eliminated rule family each (`accessor_base`, `type_reference`), each counted
once per emitter (protocol + cascade) per base/wrapper rule. ⇒ the schema move is confined to PGEN's
two *internal* annotation grammars; no downstream consumer family (SV, VHDL, regex, PNR) emits a
chain today. SystemVerilog joins them only when `A2.5` lands, and additively.

**2. WHY no gate caught it — and this half generalizes.**
The annotation-vs-emitted-AST oracle the leaf asks for **already existed**:
`run_inventory_wide_auto_gate` (`rust/src/auto_return_annotation_shape_gate.rs`), driven per grammar
by `rust/tests/auto_return_annotation_shape_gate_integration.rs`, including
`auto_gate_return_annotation_inventory_wide_shape` whose sample list literally contains `"$1.field"`
and `"$1[0]"` — the two inputs that reach the defect. It ran, it saw the value, and it passed
(`test result: ok. 9 passed; 0 failed`). It was blind in **two** independent ways:

- **Position-blind verification.** The walker verifies any object whose `type:` literal is a
  *declared* discriminator, wherever it sits in the tree, and **skips** any object whose `type:` is
  not declared. `_pgen_lr_chain` is not in the inventory — synthetic annotations are excluded from it
  by design (`python3 -c "…"` over `generated/return_annotation_return_annotations.json`:
  `has _pgen_lr_chain entry: False`) — so the record was skipped without a verdict.
- **Coverage-blind coverage.** The declared `property_access` object *nested inside* the record's
  `initial` field was found and verified, so `property_access`/`array_access` never appeared in
  `discriminators_not_covered()` either: the run reports only `{"flat_spread", "matched_text",
  "null"}` uncovered. Even the coverage leg — which is `eprintln!`'d, not asserted — read green.

⭐ Stated generally: **an oracle that only checks values it recognizes cannot see an engine that
wraps them.** The missing leg is negative space — *nothing in a published AST may carry a
representation the engine invented* — and that is what `.8` adds.

#### THE FIX — one fold, three callers, and a variant the compiler cannot let you miss

**`UnifiedReturnAST::LrChainFold { initial, suffixes, specs }`** replaces the synthetic
`Object{type: "_pgen_lr_chain", …}`. A first-class variant rather than a recognized object shape *is*
the fix's spine: the chain value is built by three independent emitters and read by the validator,
the fusibility census, the shape gate, two unparse surfaces, the pretty-printer and the legacy
string generator — and a variant makes `rustc` enumerate all of them. `cargo check --lib --tests`
listed exactly **14 exhaustive-match sites** (`error[E0004]`), every one of which had silently
ignored the synthetic object for the life of the feature; each is now an explicit, commented arm.
⚠️ Honest bound: three further sites carry a `_ =>` wildcard and so did NOT fail to compile
(`dv_ast_contains_matched_text`, `dv_ast_contains_passthrough`, and the boundary-scanner plan's
`shaped_object_class_verdict`). The first two were given real arms by hand; the third correctly
DROPS the rule with a named reason, because it requires `Some(UnifiedReturnAST::Object { .. })` —
which is why a wildcard is not a substitute for the variant, and why they had to be found by reading
rather than by the compiler.

**`rust/src/ast_pipeline/lr_chain_fold.rs`** holds the fold — ONE implementation, called by
`AstReturnTransformer::generate_lr_chain_fold` (protocol graph), the cascade emitter's
`dv_value_transform_expr` (fused bare-parse graph) and the interpreter's `fold_return`. ⭐ The
alternative — a fold per emitter — was refused precisely because this leaf's own root cause is that
mutual agreement between implementations proved nothing; with one function the three cannot diverge
*by construction*, and the gates are freed to check the property that matters (AST ≡ declaration).

Cost, by construction rather than by hope: the `wrapper_specs` JSON is parsed **once per process**
behind an emitted `OnceLock` (the interpreter interns its table to `'static` the same way it already
interns rule names), the fold borrows template strings straight into the produced `PgenValue`s, and
the giant `wrapper_specs` string literal **stops being a field of every emitted chain value**. The
per-iteration work is one small template walk — irreducible, since a left-nested binary AST is O(n)
nodes.

**Honest bounds, enforced at GENERATION time.** `validate_chain_templates` refuses `$text`/`$0`, a
quantified extraction, a nested chain, and an out-of-range positional inside a chain template, naming
the rule and the construct. A template the engine cannot fold is a missing engine capability and now
stops the build; before `.8` it would have produced a silently wrong AST.

**The oracle.** `run_inventory_wide_auto_gate` gains the negative-space leg: an emitted value whose
`type:` carries the reserved `_pgen_` prefix is a **failure**, not a skip — no grammar author can
declare that prefix, so its presence in a published AST is unambiguously the engine leaking through
the annotation-shaped contract. It is proven to FIRE by
`inventory_wide_gate_fails_on_an_engine_internal_type_discriminator`, which feeds the exact pre-`.8`
value and additionally pins the *coverage blindness* (`discriminators_not_covered()` is still empty
on the leaked shape) — so the record says why the coverage leg alone could never have caught this.
A GREEN control in the same test asserts the folded shape passes.

**The discriminating suite case.** `left_recursion_folded_ast` (combinator
`LeftRecursionFoldedAst`) is an LR grammar with **two** operators and declared object annotations;
the pre-existing `left_recursion` case is annotation-free, which is why the suite was green
throughout. Byte-identity alone still cannot judge it — both sides folded identically before, both
leaked identically — so the gate test asserts the **exact** left-nested value for `n+n-n` against the
declaration, plus the absence of any `_pgen_` marker.

#### ROUTED OUT — a derived-count drift found while doing this leaf's lockstep

Updating the combinator-suite count for the new case required re-deriving it, and the published
counts were **already wrong** — three of them, none caught by any gate:

| surface | published | re-derived from source | stale by |
|---|---|---|---|
| `TOOLBOX.md` 1.7 (twice: prose + `N/N CLEAN`) | 28 | `grep -c "^    CombinatorCase {" …combinator_suite.rs` = **31** (32 after this leaf) | 3 |
| `docs/book/src/parse-harness.md` (prose + an itemised `16+4+3+2+2+1` derivation) | 28 | same | 3 |
| `docs/book/src/parse-harness.md` (semantic half) | 29 | `grep -c "^    SemanticCase {" …semantic_suite.rs` = **36** | 7 |

⛔ **Does it reproduce outside this tree?** Yes, and that is why it is routed rather than absorbed:
two independent surfaces, two independent suites, and the drift predates this leaf (the missing 3
are `GENERATED-LINT-CORRECTNESS.2`'s associativity cases). The failure direction is the quiet one — a
count that UNDERSTATES coverage reads as a smaller suite, so nothing prompts a re-check.
`docs/DERIVED_STATE_CONTAINMENT.md` governs exactly this class, and its `E2.6` enforcer covers
layer-A `MEMORY.md` **only**; outside layer A nothing checks.

⇒ instances CORRECTED here as ordinary lockstep (TOOLBOX 32/32, book 32 structural / 36 semantic —
each re-derived, not re-typed); the **class** is routed to `LIVE-DOC-CONTAINMENT.6`, which is parked
behind the SV lane lock with a named re-open trigger. Not worked here: it is a live-doc enforcement
gap, not an engine service and not an SV-release blocker.

##### ROUTING EVIDENCE (`LIVE-DOC-CONTAINMENT.6`)

1. **Does the finding reproduce OUTSIDE the family it is being sent to?** It reproduces outside *this*
   tree, which is why it is being sent away — and it is **not** specific to the destination family
   either, which is the honest statement. Measured on **two independent documents** (`TOOLBOX.md`,
   `docs/book/src/parse-harness.md`) about **two independent suites** (structural combinator, semantic
   orchestration), i.e. 3 drifted counts, none sharing a source or a maintainer path. `LIVE-DOC-CONTAINMENT`
   is the right destination not because the defect belongs to it by locality, but because that tree
   already owns the governing standard (`docs/DERIVED_STATE_CONTAINMENT.md`, authored by its `.1`) and
   its existing enforcer (`.3`'s `E2.6`) is precisely the thing whose scope is too narrow here.
2. **What was MEASURED, not what makes it plausible?** The counts were re-derived from the source
   arrays: `grep -c "^    CombinatorCase {" rust/src/parse_harness_combinator_suite.rs` → **31** before
   this leaf's addition (32 after) against a published **28**; `grep -c "^    SemanticCase {"
   rust/src/parse_harness_semantic_suite.rs` → **36** against a published **29**. Both suites'
   `*_coverage_is_complete` gate tests pass, so the ARRAYS were always right and only the PROSE was
   wrong — which is the derived-state class exactly, not a coverage gap.
3. **What would make the routing WRONG, and was it checked?** It would be wrong if these counts were
   already enforced somewhere, making this a broken-check story owned by whoever wrote the check rather
   than an unenforced-class story. Checked: `E2.6` in `scripts/check_memory_architecture.sh` is scoped
   to layer-A `MEMORY.md`, and no doctrine in `scripts/check_doctrines.sh` re-derives a published suite
   count — the 18-doctrine report was read for this. It would also be wrong if the drift were confined
   to a surface this tree owns; it is not — `TOOLBOX.md` and the live book are both outside it.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `./rust/target/debug/parseability_probe --parse-dump-ast-pretty
  return_annotation ann2.txt out.json` on input `$1.a.b` (a shipped, `fully_certified` grammar)
  returned the eliminator's record instead of the declared shape — and dropped `.b` entirely:
  `{"initial": {"property": "a", "type": "property_access", …}, "suffixes": [{"alt_index": 0,
  "captures": [".", "b"], "type": "_pgen_lr_chain_alt"}], "type": "_pgen_lr_chain", "wrapper_specs":
  "<439 bytes>"}`. Static census `grep -c '"_pgen_lr_chain"' generated/*_parser.rs` → 6
  (`return_annotation`) + 10 (`semantic_annotation`), 0 elsewhere.
- [x] **ROOT CAUSE (WHY + WHERE)** — TWO mechanisms, each located.
  **(a)** `rewrite_lr_chain_annotations` (`rust/src/ast_pipeline/mod.rs`, steps 4–5) installs the
  synthetic chain record and **no stage folds it back**; the only fold,
  `unified_return_ast.rs::parse_typed_lr_chain`, is a reader-side `UnifiedReturnAST` path no
  generated parser reaches. The emitted value at that exact mechanism, dumped with
  `./rust/target/debug/parseability_probe --parse-dump-ast-pretty return_annotation ann2.txt out.json`
  (input `$1.a.b`), is `{"initial": {…"property": "a"…}, "suffixes": [{"alt_index": 0, "captures":
  [".", "b"], "type": "_pgen_lr_chain_alt"}], "type": "_pgen_lr_chain", "wrapper_specs": "<439 B>"}`
  — the record, not the declaration, and the second continuation unapplied. **(b)** The oracle that
  should have caught it —
  `run_inventory_wide_auto_gate`, driven by
  `rust/tests/auto_return_annotation_shape_gate_integration.rs`, whose curated samples literally
  include `"$1.field"` and `"$1[0]"` — verifies only objects whose `type:` matches a DECLARED
  discriminator and **skips** the rest, and the inventory excludes synthetics by construction
  (`python3 -c` over `generated/return_annotation_return_annotations.json` →
  `has _pgen_lr_chain entry: False`). Its coverage leg then found the declared `property_access`
  object nested inside the record's own `initial` field and marked it covered — the run reports only
  `{"flat_spread", "matched_text", "null"}` uncovered. `cargo test --features generated_parsers
  --test auto_return_annotation_shape_gate_integration` → `test result: ok. 9 passed; 0 failed` on
  the defective AST.
- [x] **FIX** — ENGINE tier (the only tier that satisfies the tree's boundary rule: left recursion is
  a property of *parsing*, not of any language). New `UnifiedReturnAST::LrChainFold` variant +
  `rust/src/ast_pipeline/lr_chain_fold.rs`, ONE fold called by all three emitters. ZERO grammar
  bytes; nothing for an author to learn or opt into.
- [x] **ADDRESSED (verified)** — before→after on the reproducer, same command, same input:
  `{initial, suffixes, type: "_pgen_lr_chain", wrapper_specs}` → `{"base": {"base": {"index": 1,
  "type": "positional"}, "property": "a", "type": "property_access"}, "property": "b", "type":
  "property_access"}` — the declared left-nested shape, with `.b` applied. `$1.a[2].b` folds across
  BOTH wrapper alternatives (`{…"index": 2, "type": "array_access"…}` nested under
  `property_access`), proving `alt_index` dispatch. Static census after regeneration:
  `grep -l '"_pgen_lr_chain"' generated/*_parser.rs` → **0 files** (was 2), with 6 + 10
  `lr_chain_fold::fold_lr_chain` call sites in their place. The oracle that was blind now FAILS the
  pre-fix value: `cargo test --lib
  auto_return_annotation_shape_gate::tests::inventory_wide_gate_fails_on_an_engine_internal_type_discriminator`
  → ok (RED probe + GREEN control in one test).
- [x] **NO REGRESSION** — every oracle that can SEE this change, re-run, plus a two-sided control on
  what it can reach.
  * `parse_harness_equivalence_gate` — interpreter vs generated parser over every registered grammar:
    **4 passed / 0 failed**, `certified_grammars_are_byte_identical` green, so the two grammars whose
    AST moved are still **byte-identical** between the two implementations.
  * `parse_harness_combinator_gate` — **32/32 CLEAN, 0 divergences** (2 passed / 0 failed, 153.92 s),
    including the new `left_recursion_folded_ast CLEAN samples=6 diverge=0 anchor_miss=0` and its
    exact-declared-shape assertion.
  * `parse_harness_semantic_gate` — **36/36 CLEAN, 0 divergences** (2 passed / 0 failed, 165.31 s).
  * `ast_shape_contract_gate` — **18 passed / 0 failed**, ✅ *AST-shape contract gate passed*.
  * `auto_return_annotation_shape_gate_integration` — **9 passed / 0 failed** WITH the new
    negative-space leg live ⇒ no emitted AST in any of those 9 grammars carries a `_pgen_`
    discriminator.
  * touched-module unit tests (`lr_chain_fold`, the shape gate, `unified_return_ast`,
    `annotation_validator`, `ast_shape_contract`, `fusibility_census`) — **160 passed / 0 failed**.
  * `make clippy_on_rust_change` — ✅ completed, **0 `^error` lines**; source strict lint clean and
    `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS` (68 pinned lints all still in
    `clippy::correctness`), with the generated-parser stage STRICT by default.
  * `mdbook_docs_gate` — ✅ passed, plus all **10** per-parser book gates.
  * CODEGEN DETERMINISM re-proven: three consecutive `--generate-parser` runs to the SAME output path
    are **byte-identical** (`60553fa7…`), and the shipped artifact equals a fresh run modulo the
    output path the generator embeds in its own source.
  * ⭐ TWO-SIDED REACH CONTROL: `_lr_base` helper-rule census over all 11 generated parsers —
    `return_annotation` **44**, `semantic_annotation` **48**, every other grammar **0**. So
    `rewrite_lr_chain_annotations` never runs for the other nine, no `LrChainFold` node is ever
    constructed there, and every arm this change added is unreachable for them. That is a proof of
    scope, not an assumption — and the control shows a non-empty population, so it is not a
    silently-broken grep.
  * ⛔ HONEST BOUND, recorded rather than waived: the FULL `cargo test --lib` sweep is **not** part of
    this evidence. It was started and abandoned at 68 minutes; `/usr/bin/sample` on the test binary
    showed the only two busy threads were `parser_embedding_{systemverilog,vhdl}_deep_nesting_…` on a
    2000-deep paren nest — pre-existing stress tests in grammars whose `_lr_base` count is **0**, i.e.
    provably beyond this change's reach. A confirmatory full sweep runs AFTER this commit rather than
    holding verified work hostage to it; the slow-test observation itself is recorded in `.3`'s gap
    list as a candidate ENGINE service (a bounded-refusal nesting ceiling), not as a conclusion.
- [x] **LOCKSTEP** — book: `annotation-system.md` (new *Left recursion is folded back into your shape*
  section, and the Proof-Expectations list now demands an annotation-vs-emitted-AST check by name),
  `parse-harness.md` (a superseded-note on the `.5.3` blob section, the new combinator row, and the
  corrected suite counts), `developer-architecture.md` (the LR paragraph now names the fold).
  `TOOLBOX.md` 1.7 (counts + what the new case proves that byte-identity cannot). Contracts: both
  annotation families gained a `Notable Recent Shape Changes` entry — measured as **not** a downstream
  break, so **no release/schema bump and no ledger row** (their published surface returns verdict +
  diagnostics and carries no AST — argued from `rust/src/embedding_api.rs`, not assumed).
  `docs/reference/RUST_CODEBASE_ANALYSIS.md` architecture note for the new shared-runtime seam. The
  authorizing decision record gained a dated addendum closing the schema question it had explicitly
  reserved. `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md` frontier row.
  Lesson PROMOTED to `docs/knowledge/an-oracle-that-only-verifies-what-it-recognizes-cannot-see-a-wrapper.md`
  (+ the derived `KNOWLEDGE_MAP.md`). Routed out: `LIVE-DOC-CONTAINMENT.6`. DONE-BAR register: **N/A**
  — no family's status changes.

### `.9` — the annotation-conformance oracle is CURATED-SAMPLE-BOUND, not corpus-bound (`todo`, opened by `.8` 2026-08-11)

`.8` gave `run_inventory_wide_auto_gate` the leg it was missing (negative space), but left two
measured weaknesses untouched, both stated by the module's own header (*"Per-rule sample mapping … is
a future extension that needs either an auto-generated samples corpus or a curated
`<grammar>_auto_gate_samples.json`"*):

1. **Its inputs are hand-written.** Each grammar gets a curated `samples` list in
   `rust/tests/auto_return_annotation_shape_gate_integration.rs`. Coverage is therefore bounded by
   what someone remembered to type. `parse_harness_equivalence::build_corpus` already generates a
   deterministic stimuli corpus (seeds 0/7/42 × a depth ladder) for every registered grammar and is
   directly reusable here.
2. **`discriminators_not_covered()` is `eprintln!`'d, never asserted.** Measured on the current
   curated samples: `semantic_annotation` **61** uncovered, `regex` **7**, `rtl_const_expr` **2**,
   `return_annotation` **3**. A declared shape the parser never produces is a real signal — it is
   how a dead alternative or a mis-wired annotation would show — and today it is a print. It cannot
   simply be flipped to a hard failure at those counts; it needs the corpus from (1) first, then a
   **two-sided ratchet** over a tracked register (a new uncovered discriminator fails; a
   now-covered one still listed also fails), per the `GATE-REACHABILITY` / `LIVE-DOC-CURRENCY`
   precedent.

⛔ Do not read `.8` as closing the conformance question. `.8` closes *"the engine must not leak its
own representation"*, which is one shape of one defect class. `.9` closes *"every declared
annotation is exercised and verified over a corpus nobody hand-picked"*.

⭐ Also owed here: the gate has no `make` target of its own — it runs only as part of the whole
`cargo test` sweep, so `GATE-REACHABILITY`'s question (*what INVOKES it?*) has a weak answer.

## Acceptance Criteria (tree)

- The catalogue exists in the live book, every entry evidence-backed, and its counts are derived.
- The scar-tissue census is a re-runnable instrument, and its current output is published.
- The gap list is priced well enough for the director to choose from it.
- The responsibility-boundary rule is a decision record that a reviewer can cite.

### `.10` — the cert generator cannot witness the rules the ENGINE synthesizes (`in progress` — mechanism 1 CLOSED `PGEN-ENGINE-UNIVERSAL-SERVICES-0005` 2026-08-11 session #217; mechanism 2 CLOSED `PGEN-ENGINE-UNIVERSAL-SERVICES-0006` 2026-08-12 session #218; **mechanism 3 split out to `.11`**; opened 2026-08-11 by `GRAMMAR-WELLFORMED.A2.5`; ⛔ the BLOCKER for returning SV's union UNKNOWN to 0 is now `.11`)

#### ⛔ STATUS AND THE CORRECTION THIS LEAF OWES ITS OWN FIRST DRAFT

`union UNKNOWN` **2 → 1**, canonical **13 → 12**, residual now `["select_expression_lr_suffix"]`
(`block_event_expression_lr_suffix` WITNESSES). ZERO grammar bytes; generated parsers **byte-identical**.

⛔⛔ **The root cause recorded below in the original draft — and copied into the union contract's
`rebaseline_note` — was WRONG, and it is corrected here rather than quietly overwritten.** The draft
said *"the reach planner cannot route a probe to a rule that did not exist when it built its graph"*.
**Measured false.** `reach_hops_pass` BFSes `self.grammar_tree` **after** elimination, so the
synthetic rules are in its graph, and `PGEN_REACH_PATH_DUMP=1` prints a **complete and correct**
21-hop chain ending
`("bins_selection_or_option","root/o1/s1"), ("bins_selection","root/s3"), ("select_expression","root/s1/q")`
— every OR already steered, the LR quantifier already forced. **The plan was never the defect;
rendering it was.** ⭐ The draft was written from the probe SAMPLES alone (which do look like a
routing failure) without dumping the plan; the lesson is that a `parsed=true witnessed_target=false`
sample tells you where generation *ended up*, never what it was *asked* to do — those need two
different instruments, and `PGEN_REACH_PATH_DUMP` is the second one.

#### DIAGNOSIS — mechanism 1 (CLOSED): the forced quantifier had no re-entry guard

⭐ **Reproduced on an 11-rule synthetic BEFORE SystemVerilog was touched**, as the leaf itself
demanded. Probe grammar in the `scratch` slot (`decl → gitem → citem → expr`, `expr` directly
left-recursive behind a choice — the `cross_body_item → bins_selection → select_expression` shape):

```
--count 1 --seed 0  →  UNKNOWN=1 ["expr_lr_suffix"], 1 parsed-but-routed-elsewhere
  [plannable-probe] rule='expr_lr_suffix' parsed=true witnessed_target=false
                    sample="unit group x{cross x{option.x=x;}}"
  [reach-path] target='expr_lr_suffix' hops=[("scratch","root/s1/q"), ("decl","root/o3/s3/q"),
               ("gitem","root/o2/s3/q"), ("citem","root/o1/s3"), ("expr","root/s1/q")]
```

The plan steers `citem` into its `bins` arm (`root/o1`) and forces `expr`'s LR quantifier — and the
render still emits `citem`'s `option` sibling. **WHERE:** `generate_quantified`
(`rust/src/ast_pipeline/stimuli_generator.rs`) looked up `forced_quantifier_min` by
`(current_rule, node_path)` with **no call-stack re-entry guard** — unlike `generate_or`, which has
carried exactly that guard since `RTL-FE-CLOSURE.5.6` (`suppress_recursive_forced_branch`, generalised
by `H.12.5.7.2`). **WHY:** LR elimination emits `X := X_lr_base ( X_lr_suffix )*` with
`X_lr_suffix := op X`, so the forced site's own body **re-enters the rule that owns the site**. Every
re-entry re-forced the `*`. `PGEN_TRACE_VERBOSITY=high` counted **119** forced decisions at site
`expr::root/s1` in a **single** probe (`candidates=[1]` each time — and when forced, the candidate
list has exactly one entry, so there is no smaller repeat count to fall back to). The derivation never
terminates, the forced branch dies on depth, and `generate_or`'s documented
forced-first-**with-fallback** then silently renders a sibling.

⇒ **left recursion was unwitnessable BY CONSTRUCTION, for every grammar** — SV was simply the first
family where a cert census and an LR plan coexist. That is what makes this an engine fix rather than
an SV one, exactly as the leaf predicted.

#### THE FIX — one guard, mirrored from the one `generate_or` already had

`generate_quantified` stands the forced minimum down on a **genuine re-entry** (≥2 live occurrences of
`current_rule` on the call stack) **and only when** the quantified element can reach back into
`current_rule` (direct or transitive). Scoped exactly like the OR guard, so every non-recursive forced
quantifier is byte-identical and the cheap call-stack test short-circuits before the memoised
reachability query ever runs. Keyed on the structural self-reference plus the live recursion count —
never on rule names, never on the eliminator's `_lr_` spelling.

⭐ This is `.6`'s *"normalize into an existing path rather than adding a second one"* technique applied
to a **guard**: the OR path and the quantifier path now enforce the same invariant — *a reach
directive is a directive for ONE firing, the shallowest entry of its owning rule* — instead of one
path enforcing it and the other silently not.

#### DIAGNOSIS — mechanism 2 (CLOSED `PGEN-ENGINE-UNIVERSAL-SERVICES-0006`, session #218): the SEED is longest-match-shadowed, so the suffix can never commit

⛔ **Do not read the remaining residual as the same defect.** Post-fix the generator emits the
**structurally correct** probe — `cross f, f { bins f = f && f; }` — i.e. it now reaches and renders
the `&&` continuation. It still reports `witnessed_target=false` because the **parser** does not
commit to it: `select_expression`'s catch-all arm `cross_set_expression → covergroup_expression →
expression` **is the full SV expression hierarchy, which parses `&&`/`||` itself**, so under
longest-match the seed swallows the whole operand and the `( _lr_suffix )*` never commits.

Measured, not argued — `--parse-dump-ast-pretty` on that exact shape:

| input shape | AST kind at the select | `select_expression_lr_suffix` entries |
|---|---|---|
| `ignore_bins ib = ca && cb;` (what the generator emits) | **`cross_set`** | 1 (entered, never committed) |
| `ignore_bins ib = ( binsof(ca) intersect { 1 } ) && binsof(cb);` (the pinned repro) | `and` | 4 |

⭐ This is the same fact the repo already documents in prose:
`stimuli/sv/adjudication_repros/fixed_select_expression_paren.sv` warns *"Parentheses around an
operand carrying no `intersect` prove nothing — that is an ordinary SV expression reaching the
catch-all `cross_set_expression` arm"*, which is exactly why all four pinned reproducers use a
`binsof(...) intersect {...}` operand. The generator did not know it.

**OWED (mechanism 2) — SEED diversification.** When the target is `X_lr_suffix`, the witness planner
must also steer `X_lr_base`'s **own** alternatives, because a seed the longest-match catch-all can
absorb makes the suffix unwitnessable no matter how well the path is forced. The parts likely exist:
`target_own_reach_sites` already forces a target rule's own root-`Or` branch, and the
carrier-diversification pass already re-routes through alternative parents — neither currently touches
the *sibling seed rule* the eliminator created. ⛔ Prove it on the synthetic first, again: the same
11-rule scratch grammar reproduces the shape once its `expr` gains a catch-all alternative that spans
the operator.

⛔ **The two repairs that remain unavailable** (unchanged from the original draft, restated so nobody
re-proposes them): deleting the LR plan, and witness-rescue `@sample`s on the affected SV rules.

#### THE FIX — mechanism 2: a THIRD tier on the target-own-structure pass, forcing the target's SIBLING

The witness planner had exactly two levers, and neither can reach this class:
`target_own_reach_sites` forces the **target's own** body; `mandatory_child_rules` forces the
**target's children**. The shadowing rule is the target's **SIBLING** — the mandatory rule the reach
path's final hop renders *before* the min-0 quantified reference to the target.

`generate_target_own_structure_witnesses` now carries a third strictly-additive tier
(`reach_seed_rules` + `collect_preceding_seed_rules` + `mandatory_rule_reference_of` +
`subtree_references_rule`, `rust/src/ast_pipeline/stimuli_generator.rs`): for a residual target `T`
it takes the FINAL hop of the chain the plan actually installed, walks that hop rule's body for a
**min-0** `Quantified` whose element subtree references `T`, and enumerates the alternatives of every
mandatory `rule_reference` sibling preceding it — nearest first.

Design points, each deliberate:
* **min-0 only.** A min-1 continuation must be committed or the parse rejects — there is no
  zero-iteration escape, so no shadowing hazard exists. min-0 is exactly the class.
* **Plain index order** over the seed's alternatives, unlike the two older tiers' *"non-degenerate
  `o1..` first, degenerate `o0` last"*. Those tiers know `o0` is the pass-through that already
  failed; here the alternative that already failed is whichever the UNFORCED generator picked
  (min-derivation, not `o0`), so no index is privileged and skipping one would be guessing.
* **One probe per branch, not two.** The older tiers re-roll twice because their skeleton is fixed
  and only terminals vary; on this axis the skeleton is exactly what each candidate changes, so the
  same per-rule ceiling (16 probes) buys twice the seed-choice coverage.
* **Only the seed is forced.** `T`'s own body renders as the base reach plan already steers it, so a
  witness here isolates the seed as the cause.
* ⛔ Keyed on `ASTNode` structure + the installed hop chain — **never** on a rule name and never on
  the eliminator's `_lr_` spelling. `X := X_lr_base ( X_lr_suffix )*` is the shape this was found
  on, not the definition: every `P := seed ( continuation )*` carries it.
* ⛔ **HONEST BOUND (no silent caps):** only the FINAL hop is scanned. A seed shadowing an
  INTERMEDIATE hop is outside this axis.

#### ⛔ SYSTEMVERILOG IS NOT CLOSED BY THIS, AND THE REASON IS A *THIRD* MECHANISM — measured, not assumed

The seed tier **fires correctly on SystemVerilog**: the run's probes for
`select_expression_lr_suffix` now visibly diversify the seed across all five
`select_expression_lr_base` alternatives (`binsof(…)&&binsof(…)`, `!binsof(…)&&!binsof(…)`,
`(…)&&(…)`, `…&&…`, `…matches …&&binsof(…)`). SV's residual stays at `UNKNOWN=1` because **no seed
spelling can witness it** — the witnessing operand is on the *suffix* side.

Measured with `--dump-rule-outcome-counts-json` (committed counts, C3-B), one covergroup cross per row:

| `ignore_bins ib = …` | `select_expression_lr_suffix` entries | **committed** |
|---|---|---|
| `binsof(ca) && binsof(cb)` | 1 | **0** |
| `!binsof(ca) && !binsof(cb)` | 1 | **0** |
| `( binsof(ca) ) && binsof(cb)` | 2 | **0** |
| `( ca ) && ( cb )` | 2 | **0** |
| `binsof(ca) intersect ca && binsof(cb)` | 1 | **0** |
| `binsof(ca) intersect { 1 } && binsof(cb)` | 1 | **0** ⛔ |
| `( binsof(ca) intersect { 1 } ) && binsof(cb)` | 4 | **1** ✅ |
| `binsof(ca) with ( ca ) && binsof(cb)` | 4 | **2** ✅ |

Two findings fall out, and **both are routed rather than absorbed here** (see `.11` and
`SV-CORPUS-GRAD.13c.2e`):

1. **The witnessing arm is `with (…)`, and the pass never renders it.**
   `select_expression_lr_suffix := Or[ && select_expression | || select_expression | with ( … ) ( matches … )? ]`
   (`--dump-gen-ast`). Tier 1's candidate order for 3 alternatives is `[1, 2, 0]` with a 4-probe
   budget at 2 probes each, so branch **2 — the `with` arm — is allotted probes 3 and 4**; both
   rendered `&&` instead. Across the **whole** SV cert run, **0 of 27** `select_expression_lr_suffix`
   probe samples carry `with`. A forced branch that dies and silently renders a sibling is the
   mechanism-1 signature exactly, one rule further out. ⇒ `.11`.
2. ⛔⛔ **`intersect { 1 }` does not force the real alternative — and the repository says it does.**
   `stimuli/sv/adjudication_repros/fixed_select_expression_paren.sv` states *"`intersect` is a
   keyword, so it forces the real alternative"*. Measured **false** for the unparenthesized form:
   `--parse-dump-ast-pretty` on `binsof(ca) intersect { 1 } && binsof(cb)` yields kind `condition`
   with **no `and` node**, and the `intersect` payload contains a `concat` of `1` whose
   `operand_chain.rest` holds `logical_and` + `binsof(cb)` — i.e. `select_condition`'s
   `covergroup_range_list*` **swallowed `{ 1 } && binsof(cb)` as one expression**. IEEE 1800-2017
   A.2.11 spells the braces as LITERAL syntax — `intersect { covergroup_range_list }` — and
   `systemverilog.ebnf:5259` models them as neither, so `{ 1 }` is parsed as a concatenation
   *expression* and nothing terminates the list. That is an LRM-fidelity defect in the grammar, in
   the locked SV lane. ⇒ routed to `SV-CORPUS-GRAD.13c.2e`.

#### Acceptance Checklist (enforced) — mechanism 2 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0006`)

- [x] **REPRODUCE / ISSUE** — SV canonical run reproduced the documented residual exactly:
  `PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1 PGEN_REACH_PATH_DUMP=1
  ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017
  --entry-rule systemverilog_file --count 40 --seed 0` → `UNKNOWN=1
  ["select_expression_lr_suffix"]`, `[plannable-probe] … parsed=true witnessed_target=false
  sample="…{bins\foo_0 =\foo_0 &&\foo_0 ;}…"`. **Isolated to a 7-rule synthetic** in the `scratch`
  slot (preserved at `docs/tasks/artifacts/engine_universal_services/mechanism2_seed_shadowing_probe.ebnf`):
  `--count 1 --seed 0` → `UNKNOWN=1 ["expr_lr_suffix"]`, `1 parsed-but-routed-elsewhere`,
  `sample_parse_failures=0`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `generate_target_own_structure_witnesses`
  (`rust/src/ast_pipeline/stimuli_generator.rs`) had two forcing levers — `target_own_reach_sites`
  (the target's own body) and `mandatory_child_rules` (its children) — and the shadowing rule is
  neither: it is the mandatory SIBLING rendered before the min-0 quantified reference to the target.
  WHY: `--dump-gen-ast` shows the eliminated shape
  `expr := Sequence[ ref expr_lr_base @s0, Quantified*{ ref expr_lr_suffix } @s1 ]` with
  `expr_lr_base := Or[ "binsof" "(" ident ")" | fullexpr ]`; `PGEN_REACH_PATH_DUMP=1` shows the hop
  chain ending `("expr","root/s1/q")` — the quantifier is forced and **nothing decides the base**.
  The base's catch-all `fullexpr` parses `&&` itself, so under longest_match the seed absorbs the
  whole operand and the `( … )*` matches zero times. Measured on the real generated parser with
  `--dump-rule-outcome-counts-json`: `bins b=a&&x;` → `expr_lr_suffix` entries=1 **committed=0**;
  `bins b=binsof(a)&&x;` → entries=3 **committed=1**. Entered, never committed — which is precisely
  what `parsed=true witnessed_target=false` on a structurally correct sample means.
- [x] **FIX** — ENGINE tier (a property of witnessing under longest-match, not of any language); no
  lower tier exists — a declarative/grammar fix would be the scar tissue `.2` exists to remove. A
  third strictly-additive tier on the existing pass (`reach_seed_rules` + 3 structural helpers),
  running only when the two older tiers failed. ZERO grammar bytes.
- [x] **ADDRESSED (verified)** — before→after on the same commands.
  * Synthetic, **all 9 combinations** of `--count 1/2/3` × `--seed 0/7/42`:
    `UNKNOWN=1 ["expr_lr_suffix"]` → **`UNKNOWN=0 fully_certified=true`**, `sample_parse_failures=0`.
    The witnessing probe is the discriminating seed:
    `[target-own-probe] rule='expr_lr_suffix' parsed=true witnessed_target=true
    sample="unit group b0{cross fss{bins ufuj3=binsof(fg)&&binsof(exy8);}}"`, after four
    catch-all-seeded probes (`…=k9l&&rj7;…`) that did not.
  * SystemVerilog canonical: `UNKNOWN=1` → **`UNKNOWN=1`** — unchanged, and the reason is measured,
    not assumed: the tier fires and diversifies all five base alternatives, but the witnessing
    operand is on the *suffix* side (the table above). Routed to `.11`, not waived.
  * The tier is proven **DISCRIMINATING, not merely present**: with it disabled (`if false && …`)
    the new lib test fails with the exact production symptom — `witnessed 0`, and **zero**
    discriminating samples ever generated (`samples seen: []`). Restored, both tests pass.
- [x] **NO REGRESSION** — every oracle that can see a stimuli-generation change.
  * **Cross-grammar cert sweep, measured before→after by stashing the change and rebuilding** —
    `json`, `regex`, `vhdl`, `rtl_frontend`, `systemverilog_preprocessor`, `scratch` × seeds
    **0/7/42** (18 populated rows each side): `diff` of the two `CERTIFICATE-COVERAGE:` line sets is
    **IDENTICAL** — zero drift, all still `fully_certified=true`, `sample_parse_failures=0`. ⚠️ Row
    counts checked non-empty on both sides first (the `CI-PARITY-GATE-ROT.24` trap: two empty result
    sets diff clean), and `scripts/require_ast_pipeline_features.sh` confirmed
    `ebnf_dual_run=true generated_parsers=true` before each sweep.
  * ⭐ **CODEGEN BYTE-IDENTITY** — `make focus_systemverilog` run in the stashed (pre-fix) and
    restored (post-fix) trees produces a **byte-identical** parser
    (`ff9a79a0fa695cbe0167364bb558f636` both sides), proving this is a stimuli-generation change that
    cannot reach any shipped parser.
  * `sv_cert_recognized_union_gate` — ✅ **PASSED** (see the run log line quoted in `CHANGES.md`):
    `union_unknown: 1`, `canonical_unknown: 12`, `union_residual_rules:
    ["select_expression_lr_suffix"]`, `sample_parse_failures=0`, **deterministic across seeds
    [0,7,42]** (the gate asserts seed agreement itself). ⭐ Contract **not** re-baselined — the census
    is unchanged, so a rebaseline would be noise.
  * Touched module: `cargo test --lib -- ast_pipeline::stimuli_generator::` — **218 → 220 passed /
    0 failed**, the delta being exactly the two tests added here; no existing test moved.
  * `make clippy_on_rust_change` — source strict lint clean, `GENERATED-CLIPPY-CORRECTNESS` pass.
  * `parse_harness_equivalence_gate` — the one differential gate whose corpus is **built by the
    stimuli generator**, so it is the suite that can actually see this change.
  * ⛔ **Deliberately NOT claimed:** `parse_harness_combinator_gate`, `parse_harness_semantic_gate`
    and `ast_shape_contract_gate` are parser-side oracles over curated inputs; the codegen
    byte-identity proof above makes them unreachable by this change, and naming them would be
    padding the checklist with runs that cannot fail for this reason.
- [x] **LOCKSTEP** — this leaf; new leaf `.11` (mechanism 3) opened with its measured evidence; the
  `intersect { … }` LRM-fidelity defect routed to `SV-CORPUS-GRAD.13c.2e`; the probe grammar
  preserved as a tracked artifact; the cert-driver transparency line now names all three tiers;
  book `grammar-wellformedness.md`; `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`,
  `docs/TASK_TREE.md`. DONE-BAR register: **N/A** — SV's claimed status is unchanged (union UNKNOWN
  is 1, not 0). No release/schema/ledger move: no shipped parser byte changes.

#### Acceptance Checklist (enforced) — mechanism 1 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0005`)

- [x] **REPRODUCE / ISSUE** — SV canonical run, `PGEN_CERT_COVERAGE_DUMP_ALL=1
  PGEN_CERT_COVERAGE_DEBUG_PROBES=1 ast_pipeline grammars/systemverilog.ebnf
  --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40
  --seed 0` → `UNKNOWN=2`, `UNKNOWN rules (2 of 2 shown): ["block_event_expression_lr_suffix",
  "select_expression_lr_suffix"]`, both `[plannable-probe] parsed=true witnessed_target=false`.
  Isolated to an **11-rule synthetic** in the `scratch` slot at `--count 1 --seed 0`:
  `UNKNOWN=1 ["expr_lr_suffix"]`, `1 parsed-but-routed-elsewhere`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_REACH_PATH_DUMP=1` first **falsified** the recorded cause:
  the hop chain is complete and correct on both grammars (SV's ends
  `("bins_selection","root/s3"), ("select_expression","root/s1/q")`), so the planner is not blind.
  WHERE: `generate_quantified` (`rust/src/ast_pipeline/stimuli_generator.rs`) reads
  `forced_quantifier_min` by `(current_rule, node_path)` with no call-stack re-entry guard, while
  `generate_or` has had one since `RTL-FE-CLOSURE.5.6`. WHY: the eliminator's own output
  (`X := X_lr_base ( X_lr_suffix )*`, `X_lr_suffix := op X`) makes the forced site's body re-enter the
  site's owner, so the forcing re-fires unboundedly — `PGEN_TRACE_VERBOSITY=high` counts **119**
  `Quantifier decision: rule='expr' path='root/s1' … candidates=[1]` lines in one probe, and a forced
  candidate list holds exactly one repeat count, so the failure has no fallback and propagates to the
  nearest OR.
- [x] **FIX** — ENGINE tier (a property of parsing, not of any language); no lower tier exists — a
  declarative or grammar-level fix would be scar tissue for an engine gap, which is what `.2` exists
  to remove. One re-entry guard in `generate_quantified`, mirroring `generate_or`'s. ZERO grammar
  bytes; nothing for an author to learn or opt into.
- [x] **ADDRESSED (verified)** — before→after on the same commands.
  * Synthetic (`--count 1/2/3 --seed 0`): `UNKNOWN=1 ["expr_lr_suffix"]` → **`UNKNOWN=0
    fully_certified=true`**; `1 parsed-but-routed-elsewhere` → **0**; the probe now renders the real
    continuation, `[plannable-probe] rule='expr_lr_suffix' parsed=true witnessed_target=true
    sample="unit group x{cross x{bins x=x&&x;}}"`. Forced decisions at `expr::root/s1`: **119 → 5**.
  * SystemVerilog canonical: `UNKNOWN=2` → **`UNKNOWN=1`**;
    `block_event_expression_lr_suffix` now `witnessed_target=true` with
    `sample="package\foo ;covergroup\foo @@(begin\foo_0 or begin\foo_0 );endgroup endpackage"` — the
    real `or` continuation of the block-event form.
  * SystemVerilog multi-config UNION (the contract's own basis): `UNKNOWN 2 → 1`,
    `witness 1354 → 1355`, canonical `13 → 12`, residual `["select_expression_lr_suffix"]`,
    `sample_parse_failures=0`.
  * The guard is proven **DISCRIMINATING**, not merely present: with it disabled the new lib test
    fails with the exact production symptom (`left: "uopt"` — the sibling fallback), and the
    monotonicity control passes in both states.
- [x] **NO REGRESSION** — every oracle that can see a stimuli-generation change.
  * `sv_cert_recognized_union_gate` — ✅ **PASSED**, `unmet_criteria_count: 0`,
    `recognized_basis_green: true`, `canonical_unknown: 12`, `union_unknown: 1`,
    `union_witness: 1355`, `union_residual_rules: ["select_expression_lr_suffix"]`, **deterministic
    across seeds [0,7,42]** (the gate asserts seed agreement itself), `sample_parse_failures=0`.
    Contract re-baselined **in this same commit** — the `CI-PARITY-GATE-ROT.22` tripwire, which
    exists because a census-moving change once shipped both cert gates RED for a whole release.
  * **Cross-grammar cert sweep, measured before→after by stashing the change and rebuilding** —
    `json`, `regex`, `vhdl`, `rtl_frontend`, `systemverilog_preprocessor`, `scratch` × seeds
    **0/7/42** (18 runs each side): `diff` of the two `CERTIFICATE-COVERAGE:` line sets is
    **IDENTICAL** — zero drift, all still `fully_certified=true`. ⛔ Honest bound: `ebnf`,
    `return_annotation`, `semantic_annotation` and `rtl_const_expr` are **not measurable on this
    path** (`certificate-coverage: no generated parser is registered for grammar '…'`), before or
    after — recorded rather than silently counted as green.
  * ⭐ **CODEGEN BYTE-IDENTITY** — the generated parser regenerated in the stashed (pre-fix) and
    restored (post-fix) trees is **byte-identical** (`fc5a61e8ae1d5f88aff0c4a4d7f847f9`), proving
    this is a stimuli-generation change that cannot reach any shipped parser.
  * Touched module: `cargo test --lib ast_pipeline::stimuli_generator::` — **228 passed / 0 failed**,
    including the real-SystemVerilog and real-`rtl_frontend` witness/reach tests.
  * `make clippy_on_rust_change` — source strict lint clean, `GENERATED-CLIPPY-CORRECTNESS: ✅
    POLICY-ONLY PASS` (pinned correctness roster intact).
  * `parse_harness_equivalence_gate` — **4 passed / 0 failed**. The one differential gate whose
    corpus is **built by the stimuli generator** (`build_corpus` constructs a `StimuliGenerator` per
    seed × depth), so it is the suite that can actually see this change. ⭐ Scope argued rather than
    assumed: it installs no reach plan, so `reach_plan` is `None` and the new guard short-circuits
    at its first condition — the gate is the *evidence* for that, not a substitute for it.
  * `mdbook_docs_gate` — ✅ passed (the two book chapters edited here).
  * ⛔ **Deliberately NOT claimed:** `parse_harness_combinator_gate`, `parse_harness_semantic_gate`
    and `ast_shape_contract_gate` are parser-side oracles over curated inputs, and the codegen
    byte-identity proof above makes them **unreachable** by this change. Naming them as evidence
    would be padding a checklist with runs that cannot fail for this reason.
  * ⚠️ A trap hit and worth recording: `make focus_scratch` **overwrote the dual-feature
    `ast_pipeline`**, so the first baseline sweep returned an empty `CERTIFICATE-COVERAGE:` line for
    every grammar. An empty result read as "no drift" would have been a false green; the sweep was
    re-run after `--report-feature-surface` confirmed `ebnf_dual_run=true`. This is the #140-class
    trap `TOOLBOX.md` 1.4 documents, and it fires against ad-hoc sweeps too, not only the harness.
- [x] **LOCKSTEP** — union contract re-baselined **in this commit** with the corrected root cause;
  `TOOLBOX.md` (`PGEN_REACH_PATH_DUMP` promoted to a first-class step of the UNKNOWN protocol);
  book `gate-flow.md` + `developer-architecture.md`; `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  `MEMORY.md`, `docs/TASK_TREE.md` frontier. DONE-BAR register: **N/A** — SV's claimed status is
  unchanged (union UNKNOWN is 1, not 0). No release/schema/ledger move: no shipped parser byte
  changes.

#### ORIGINAL DRAFT (retained — its root cause is superseded above, deliberately not deleted)

⛔⛔ **THIS IS THE PRICE A2.5 PAID, AND IT IS RECORDED AS A DEBT, NOT AS A FOOTNOTE.**
`sv_cert_recognized_union_gate` went from `union UNKNOWN=0` to `union UNKNOWN=2`, residual
`["block_event_expression_lr_suffix", "select_expression_lr_suffix"]`. Nothing about SystemVerilog got
worse — the parser accepts strictly more, correctly. What happened is that A2.5 gave SV its first LR
plans, every LR plan synthesizes a `<rule>_lr_base` / `<rule>_lr_suffix` pair, and **the reach planner
cannot route a probe to a rule that did not exist when it built its graph.**

**ROOT CAUSE (WHY + WHERE, tools-first).** `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` on both rules:

```
[plannable-probe] rule='select_expression_lr_suffix' parsed=true witnessed_target=false
  sample="package\foo ;covergroup\foo ;cross\foo_0 ,\foo_0 {option.\foo_0 =3.5;}endgroup endpackage"
[plannable-probe] rule='block_event_expression_lr_suffix' parsed=true witnessed_target=false
  sample="package\foo ;covergroup\foo @\foo_0 ;endgroup endpackage"
```

`parsed=true witnessed_target=false` with a sample that misses the target *by taking a different
alternative of an ancestor*: the `select_expression` probe emits `cross f, f { option.f = 3.5; }` — the
`option` arm of `cross_body_item`, which never enters `bins_selection`, so `select_expression` is never
reached at all; the `block_event_expression` probe emits `covergroup f @f_0 ;` — `coverage_event`'s
simple `@` form rather than the `@@( … )` block-event form. WHERE: the rule-target installer
(`set_reach_plan_for_rule` → `install_reach_plan_from_hops`, `stimuli_generator.rs`) plans over the
rule-reference graph; `_lr_suffix` is created by `apply_left_recursive_chain_plan` *after* that graph is
built, and it sits inside the `( … )*` of the rewritten base rule.

⭐ **IT IS AN INSTRUMENT GAP, AND THE DISTINCTION IS LOAD-BEARING.** The same
`parsed=true witnessed_target=false` verdict is what contract v5 used to prove `white_space` and
`comment_only_source_region` were *engine-shadowed dead* and delete them. Reading it that way here would
be a serious error, and ground truth is what separates the two cases: four adjudication reproducers drive
these exact continuations through the REAL generated parser, pass, and pin the arm that produced them —
`fixed_select_expression_{with,paren,or}.sv` (`with_matches`, `and>paren`, `or>paren`) and
`fixed_block_event_or.sv` (`or`). ⇒ the rules are LIVE and EXERCISED; the generator cannot see them.
[[instruments-need-ground-truth]] is the standing north-star entry this instance belongs to.

⛔ **The two repairs that are NOT available, so nobody re-proposes them:**
1. **Delete the LR plan** (contract v6's fix for `module_path_expression_lr_suffix`: make the producer
   natively non-left-recursive). Closed by the director's 2026-08-11 ruling — avoiding LR plans to keep a
   census clean re-introduces exactly the hand-compilation A2.5 exists to remove.
2. **Witness-rescue `@sample`s** on the affected SV rules. Closed twice over: it is grammar bytes for an
   engine-wide problem (the same category error as hand-flattening), and it is the scar tissue `.2` exists
   to *remove*. It is also probably not even expressible — `_lr_suffix` has no source-EBNF rule to annotate.

**OWED — the repair is in the generator, and it likely already has the parts.**
`set_reach_plan_forcing_quantifiers` (`SV-EXH-PROOF.7.4.6.11`) already exists to force every `?`/`*` a
reach path crosses to expand at least once, which is one of the two things a `_lr_suffix` probe needs; the
other is a reach graph that knows about post-elimination synthetic rules. Owed:
1. Make the rule-target reach planner aware of rules synthesized by LR elimination — either by rebuilding
   the reach graph post-elimination, or by teaching the planner the `base → _lr_base ( _lr_suffix )*`
   shape directly so it can route through the quantifier to the suffix.
2. ⭐ **Prove it on a synthetic grammar first, not on SystemVerilog.** The combinator suite already ships
   the isolating grammars (`direct_left_recursion*`, `left_recursion*`); a cert-coverage run over one of
   them reproduces this with four rules instead of 1 362, which is where this should be debugged.
3. Restore `expected_union_unknown` to **0** and empty `expected_union_residual_rules` in
   `systemverilog_recognized_cert_union_contract.json`, deleting this leaf's entry from its
   `rebaseline_note`.

⚠️ **Generalization, and the reason this is a `.10` rather than an SV leaf:** *any* grammar that gains an
LR plan gains two rules this census cannot witness. SV is simply the first family where a cert-coverage
census and an LR plan coexist. Fixing it in the engine fixes it for VHDL, PNR and everything after.

**Acceptance:** a cert-coverage run over an isolating left-recursive grammar witnesses its `_lr_suffix`
rule; SV's `sv_cert_recognized_union_gate` returns to `union UNKNOWN=0` with residual `[]`, deterministic
across seeds 0/7/42; and the contract's re-baseline note records the restoration.

### `.11` — a FORCED branch that dies still renders a sibling SILENTLY, so the one witnessing arm is never probed (✅ **`done`** — slice 1 INSTRUMENT `PGEN-ENGINE-UNIVERSAL-SERVICES-0008` + slice 2 FIX `PGEN-ENGINE-UNIVERSAL-SERVICES-0009`, both 2026-08-12 session #219; opened 2026-08-12 session #218 by `.10` mechanism 2. ⭐⭐ **SV's recognized union basis is now `UNKNOWN=0`, residual `[]`, `fully_certified_via_union: true` at seeds 0/7/42 — the `GRAMMAR-WELLFORMED.A2.5` debt is fully repaid**)

`.10` mechanism 1 fixed a forced **quantifier** that re-fired unboundedly and fell back to a sibling.
`.10` mechanism 2 fixed the **seed** the plan never decided. What is left is the same *fallback*
failure mode one rule further out, and on the **suffix** side.

**WHAT IS MEASURED (not assumed).**

* SV's residual is `select_expression_lr_suffix`, whose eliminated body is (`--dump-gen-ast`)
  `Or[ logical_and select_expression | logical_or select_expression | kw_with lparen with_covergroup_expression rparen ( kw_matches integer_covergroup_expression )? ]`.
* Only the **third** arm witnesses. Measured on the real generated parser with
  `--dump-rule-outcome-counts-json` (committed counts, C3-B):
  `binsof(ca) with ( ca ) && binsof(cb)` → `select_expression_lr_suffix` entries=4 **committed=2**,
  while every `&&`/`||`-only spelling the generator emits is entries=1..2 **committed=0**. (The full
  eight-row table is in `.10`.)
* The target-own tier **is allotted that arm**: for 3 alternatives its candidate order is `[1, 2, 0]`
  with a 4-probe budget at 2 probes each, so branch **2 — the `with` arm — receives probes 3 and 4**.
  Both rendered `&&` instead.
* Across the **whole** canonical SV cert run, **0 of 27** `select_expression_lr_suffix` probe samples
  carry `with` (`grep "rule='select_expression_lr_suffix'" | grep -c with` → `0`).

⇒ the forced branch was selected and did **not** render, and `generate_or`'s documented
forced-first-**with-fallback** silently emitted a sibling — the mechanism-1 signature exactly.

**THE TWO CANDIDATE MECHANISMS — one is already REFUTED (2026-08-12, `PGEN-ENGINE-UNIVERSAL-SERVICES-0006`).**

1. **Budget / downstream failure — the SURVIVING candidate.**
   `with_covergroup_expression → covergroup_expression → expression` is the full SV expression
   hierarchy; the pass's `max_depth = reach_prefix_budget + target_subtree_depth` may not admit it,
   so the forced branch aborts (depth-exceeded, visit-limit or a prune) and `generate_or`'s
   forced-first-**with-fallback** renders a sibling. The `failure_reasons` record
   (`TOOLBOX.md` 6.1 — already written by every run that emits reports, no re-run needed) names the
   per-branch error; a `--max-depth` ladder (24/32/40) is the A/B that separates depth from prune.
2. ⛔ **Suppression — REFUTED, measured.** `suppress_recursive_forced_branch` only stands a forced
   branch down when the forced alternative references `current_rule` **or** reaches it transitively.
   Computed over the post-elimination gen-AST (BFS on the rule-reference graph from each
   alternative's own references, target `select_expression_lr_suffix`):
   | alt | own references | transitive reachers of the target | predicate |
   |---|---|---|---|
   | 0 `&& select_expression` | `logical_and`, `select_expression` | `select_expression` | **true** |
   | 1 `\|\| select_expression` | `logical_or`, `select_expression` | `select_expression` | **true** |
   | 2 `with ( … ) ( matches … )?` | `kw_with_…`, `lparen`, `with_covergroup_expression`, `rparen`, `kw_matches_…`, `integer_covergroup_expression` | **none** | **false** |
   ⇒ the guard **cannot** fire for the `with` arm, whatever the call-stack depth — the second
   conjunct is false. This candidate is closed; do not re-propose it.

⭐ **A silent fallback is the real finding here, wider than SV.** `generate_or` falling back is
correct for termination and wrong for *observability*: a reach directive that was overridden leaves
no trace at default verbosity, so a probe that "did not witness" is indistinguishable from a probe
that was never actually driven. Whatever the SV cause turns out to be, this leaf should also make an
overridden forced branch **visible** — the cheapest honest form being a counter or a dump line in the
same family as `PGEN_REACH_PATH_DUMP`, since the pairing lesson `.10` already paid for is that the
plan and the render need two different instruments.

**Acceptance:** the WHY named with tool output (which of the two mechanisms, at which site); a probe
that renders `select_expression_lr_suffix`'s `with` arm; `sv_cert_recognized_union_gate` at
`union UNKNOWN=0` with residual `[]`, deterministic across seeds 0/7/42; an overridden forced branch
observable without source edits; and the isolating synthetic first, as `.10` demanded twice.

#### ⛔ CORRECTION TO THIS LEAF'S OWN PLAN — `failure_reasons` is NOT reachable from a cert-coverage run

The plan above said to read the `failure_reasons` record because it is *"already written by every run
that emits reports, no re-run needed"* (`TOOLBOX.md` 6.1). That is TRUE of the closed-loop replay gap
report and **FALSE of the path this residual lives on**, which is why session #219 started by proving
it rather than by trusting it:

* `--report-certificate-coverage` **returns** at `rust/src/main.rs:1149`
  (`return run_certificate_coverage_report(…)`), long before the `--coverage-output` write at
  `main.rs:1373`.
* `--coverage-output` cannot even be *requested* alongside it — clap rejects the pair up front:
  ```
  $ ast_pipeline grammars/json.ebnf --report-certificate-coverage --count 3 --seed 0 \
        --coverage-output rust/target/eus_11/probe_cov.json
  Error: …/--coverage-*/… require --generate-stimuli or --generate-stimuli-module
  $ ls rust/target/eus_11/probe_cov.json → No such file or directory
  ```

⇒ every `record_branch_failure` the witness passes file is discarded when the process exits. The WHY
was **structurally unobservable**, not merely unlogged — so per the TOOLBOX-FIRST standing directive
(*"if the existing tools cannot surface the WHY and WHERE, the next step is to BUILD a tool, not to
speculate"*) slice 1 built the missing leg, which the leaf's own acceptance already required.

#### SLICE 1 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0008`) — the instrument: `PGEN_REACH_FORCED_OVERRIDE_DUMP`

`rust/src/ast_pipeline/stimuli_generator.rs` — a presence-gated env flag in the
`PGEN_REACH_PATH_DUMP` family that makes `generate_or`'s forced-first-**with-fallback** visible, in
two paired lines:

| line | emitted when | answers |
|---|---|---|
| `outcome=failed reason=…` | the reach-FORCED branch is the one that just failed | **WHY** the directive was lost — the generator's own error string, the same text `record_branch_failure` files under `failure_reasons` |
| `outcome=overridden rendered_branch=M` | the OR returns `Ok` on a branch that is **not** the forced one | **WHAT** was silently substituted |

Design points, each deliberate:
* **Wired at all seven exits of the attempt loop**, not the obvious two — the four `return Ok`
  (literal-hint short-circuit, plain success, depth-slack-retry success, constructive-reach-retry
  success) and the three `record_branch_failure` sites. A partially-wired instrument that misses a
  retry path would report *"never overridden"* on exactly the runs where a retry rescued it, and that
  silent-blind-spot class is what this leaf exists to close.
* **Only the forced branch's own failure is reported.** A sibling failing afterwards is ordinary
  search; the forced one failing is the directive being lost.
* **Read ONCE per process** (`OnceLock`, the `report_memo_stats_enabled` discipline). `generate_or`
  is the generator's hottest chokepoint, so a per-node `getenv` would be a locked linear `environ`
  scan at every OR site.
* **Presence-gated PRINT only** — it never changes a generation decision. Proven, not asserted:
  the full 1 612-line probe stream of the canonical SV run is byte-identical with the flag on
  (`diff -q`), and the `CERTIFICATE-COVERAGE:` headline is unchanged.
* ⛔ Keyed on rule name + node path + branch index — **no grammar-specific spelling**, so it works on
  any EBNF (engine tier, per the 2026-08-11 director ruling).

#### DIAGNOSIS — candidate mechanism 1 CONFIRMED, and it is DEPTH (not prune, not visit-limit, not timeout)

The instrument named it on its **first** run — the canonical SV cert-coverage command with
`PGEN_REACH_FORCED_OVERRIDE_DUMP=1` added:

```
  [forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3 outcome=failed
      reason="Stimuli generation depth exceeded max_depth=67 while expanding rule 'real_number'"   ×2
  [forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3 outcome=overridden
      rendered_branch=0                                                                            ×2
```

Exactly the two probes the leaf predicted (tier 1's candidate order `[1, 2, 0]` allots branch 2 the
3rd and 4th probes), failing and being substituted by branch 0 (`&&`) — 1 784 override events across
the whole run, of which these 4 are the residual's.

**WHY:** the forced `with ( with_covergroup_expression )` arm descends the full SV expression
hierarchy and runs out of DEPTH — it died expanding `real_number`, a leaf of that cascade.

**WHERE:** `generate_target_own_structure_witnesses`, `rust/src/ast_pipeline/stimuli_generator.rs`:

```rust
let target_subtree_depth = min_derivation_depths.get(rule).copied().unwrap_or(0);
let budget = reach_prefix_budget.saturating_add(target_subtree_depth);   // 2*24 + 19 = 67
self.config.max_depth = budget;
```

The budget is computed **once per rule, outside** the `'branches:` loop that then FORCES a specific
alternative — so it is scoped to the rule's *shallowest* alternative, never to the one being forced.
⭐ The engine already knows this is wrong and says so, one function away:
`witness_target_depth_budget` (`SV-EXH-PROOF.7.4.6.9`) budgets a BRANCH target by
`min_full_derivation_depth_of_node(alternative) + 1`, and its docstring states the exact failure —
*"a rule-scoped depth is the depth of that rule's SHALLOWEST alternative — which is precisely the
alternative a residual branch target is NOT"*. The target-own pass never adopted it. ⇒ slice 2.

#### THE `--max-depth` LADDER — depth confirmed, and the global knob REFUSED as the fix

The A/B the leaf asked for, run end-to-end on the canonical SV config (`--count 40 --seed 0`):

| `--max-depth` | target-own budget | forced branch 2 | `with`-carrying probes | union-basis `UNKNOWN` | `sample_parse_failures` |
|---|---|---|---|---|---|
| **24** (default) | `2×24 + 19 = 67` | **failed** → overridden to 0 | 0 of 27 | **1** | **0** |
| 32 | `2×32 + 19 = 83` | rendered, witnessed | 1 | **0** | **8** ⛔ |
| 40 | `2×40 + 19 = 99` | rendered, witnessed | 1 | **0** | **17** ⛔ |

Two conclusions, both measured:
1. **Depth is the binding constraint** — more budget alone flips the residual. The prune/visit-limit
   and timeout candidates are excluded: the reason string names `depth exceeded`, and no
   `target_timeout`/`helper_timeout` reason appears for this rule at any rung.
2. ⛔ **Raising `--max-depth` is NOT the fix, and this is why it must not be proposed again.** It is a
   GLOBAL knob: it buys `UNKNOWN 1→0` by paying `sample_parse_failures 0→8→17`, monotonically worse
   with depth. Those are witness samples the real parser then REJECTS — trading a known residual for
   unproven witnesses on the *same* gate. The fix has to be LOCAL to the forced branch.

#### Acceptance Checklist (enforced) — slice 1, the instrument (`PGEN-ENGINE-UNIVERSAL-SERVICES-0008`)

- [x] **REPRODUCE / ISSUE** — canonical run reproduces the documented residual exactly:
  `PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1 ast_pipeline
  grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017 --entry-rule
  systemverilog_file --count 40 --seed 0` → `CERTIFICATE-COVERAGE: … UNKNOWN=1 fully_certified=false
  (sample_parse_failures=0…)`, `UNKNOWN rules (1 of 1 shown): ["select_expression_lr_suffix"]`.
  `grep -c "rule='select_expression_lr_suffix'"` → **27** probe samples,
  of which `grep -c with` → **0**. The WHY behind that 0 was unobservable: `--coverage-output` is
  refused alongside `--report-certificate-coverage` (`Error: …/--coverage-*/… require
  --generate-stimuli…`, artifact never created), so `failure_reasons` never reaches disk.
- [x] **ROOT CAUSE (WHY + WHERE)** — the new instrument names both on its first run.
  Exact command: `PGEN_REACH_FORCED_OVERRIDE_DUMP=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1 ast_pipeline
  grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017 --entry-rule
  systemverilog_file --count 40 --seed 0 2>&1 | grep forced-override` →
  `[forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3 outcome=failed
  reason="Stimuli generation depth exceeded max_depth=67 while expanding rule 'real_number'"`,
  paired with `outcome=overridden rendered_branch=0`. **WHY** = the forced `with (…)` arm exhausts
  the DEPTH budget inside the SV expression cascade; **WHERE** = the per-rule budget
  `reach_prefix_budget + min_derivation_depths[rule]` computed outside the `'branches:` loop in
  `generate_target_own_structure_witnesses` (`rust/src/ast_pipeline/stimuli_generator.rs`), which is
  scoped to the rule's shallowest alternative rather than the forced one. Independently confirmed by
  the `--max-depth` 24/32/40 ladder above (`UNKNOWN` 1→0→0).
- [x] **FIX** — engine tier, and deliberately the SMALLEST that closes the observability gap: one
  `OnceLock` env flag + two print-only helpers + one `Option<usize>` binding, wired at all seven
  exits of `generate_or`'s attempt loop. No declarative or grammar tier exists for this defect — the
  missing thing is an ENGINE diagnostic, and the leaf's own acceptance already required it.
- [x] **ADDRESSED (verified)** — before: the forced-branch override left **no trace at any
  verbosity**, and `.11` carried two candidate mechanisms with no way to choose between them. After:
  the run emits **1 784** located override records, **4** of them for the residual rule, naming the
  mechanism (`depth exceeded`), the site (`path='root' forced_branch=2/3`) and the substitution
  (`rendered_branch=0`). Candidate 1 (budget) is CONFIRMED; candidate 2 (suppression) stays REFUTED.
  Re-runnable oracle: the canonical command above with `PGEN_REACH_FORCED_OVERRIDE_DUMP=1`.
- [x] **NO REGRESSION** — the flag is print-only and measured to be so: the canonical run's full
  probe stream (`[plannable-probe]`/`[target-own-probe]`/`[store-free-probe]`/`[carrier-div-probe]`,
  **1 612** lines) is **byte-identical** with the flag ON vs the pre-change baseline (`diff -q`
  clean), and the `CERTIFICATE-COVERAGE:` headline is unchanged
  (`total=1362 proof=17 witness=1344 UNKNOWN=1 sample_parse_failures=0`). Named oracle re-run:
  `make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate` GREEN — canonical `UNKNOWN=12`,
  union `UNKNOWN=1`, union residual `["select_expression_lr_suffix"]`, `sample_parse_failures=0`,
  identical across seeds 0/7/42, all equal to the tracked contract. `clippy_on_rust_change` clean
  (strict source + strict generated). ZERO codegen bytes: the change is in the stimuli generator, so
  the generated parsers are untouched.
- [x] **LOCKSTEP** — `TOOLBOX.md` §6.4 (new tool entry) + its quick-chooser row + its five-family
  signature table; `scripts/check_diagnosis_evidence.sh` `DIAGNOSIS_SIG` (see below);
  `docs/book/src/diagnosing-unknowns.md` (mirrored surface); a new Knowledge-Map card
  `docs/knowledge/a-recorded-failure-reason-is-not-a-readable-one.md` with
  `a-probe-sample-says-where-generation-ended-up-never-what-the-plan-asked-for.md` updated to point
  at the instrument it asked for, plus the regenerated `KNOWLEDGE_MAP.md`; `CHANGES.md`;
  `DEVELOPMENT_NOTES.md`; `MEMORY.md`. The DONE-BAR register and its book view are UNCHANGED — no
  family's claimed status moves on a diagnostic-only slice.

#### ⭐ THE ACCEPTANCE GATE REFUSED THIS SLICE FIRST, AND IT WAS RIGHT — a new instrument must be REGISTERED

Worth recording because the failure mode is general and the temptation is to route around it.
`scripts/check_diagnosis_evidence.sh` blocked the first commit attempt with
*"ROOT CAUSE box is ticked and backed, but it belongs to a LEAF THIS CHANGE DID NOT TOUCH"* — an
accurate report of a real state. The box in **this** leaf carried **0** tokens from `DIAGNOSIS_SIG`
(measured, not guessed: `awk` the box body, `grep -cE` the signature alternation → `0`), because the
only tool that could produce the diagnosis was the one this very slice was creating. With no
signature match here, the checker fell through to a *different* ticked ROOT CAUSE box in a finished
leaf and correctly refused to let it be borrowed.

⇒ **group 1 of the five-family table is a VOCABULARY OF TOOLS, and an unregistered tool is an
invisible one.** `PGEN_REACH_PATH_DUMP` is already listed on identical footing. Landing a family-1
instrument without adding its token leaves an author exactly two exits — cite a tool that did not
produce the diagnosis, or waive — and the second is the failure mode `GENERATED-LINT-CORRECTNESS.4`
named when it refused a gate that *"teaches waivers"*. So `PGEN_REACH_FORCED_OVERRIDE_DUMP` and
`[forced-override]` join `DIAGNOSIS_SIG`, and `TOOLBOX.md`'s family table records the obligation in
both directions: register a token **only** for a real runnable instrument, and in the same commit.

⛔ **This is an addition to the recognized-tool list, NOT a relaxation of the bar** — the same
correction group 2 made for this repo's real profilers (`SPEED` was being forced to waive an
otherwise-correct gate because the list named `cargo flamegraph` and not `/usr/bin/sample`). Nothing
about what counts as evidence changed: the box still has to quote the output of a tool that was run,
and this slice's box quotes the exact command plus its verbatim output.

#### SLICE 2 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0009`) — the fix: budget the ALTERNATIVE you FORCE

⭐⭐ **`select_expression_lr_suffix` WITNESSES, and SystemVerilog's recognized union basis reaches
`UNKNOWN=0` with residual `[]` — the `GRAMMAR-WELLFORMED.A2.5` debt is fully repaid.**

`generate_target_own_structure_witnesses` set its depth budget **once per rule, outside** the
`'branches:` loop that then forces a specific alternative:

```rust
let target_subtree_depth = min_derivation_depths.get(rule).copied().unwrap_or(0);
let budget = reach_prefix_budget.saturating_add(target_subtree_depth);   // 2*24 + 19 = 67
self.config.max_depth = budget;
```

A rule-scoped depth is the depth of that rule's **shallowest** alternative — precisely the one a
forced branch is *not*. The fix computes, per iteration, that branch's own
`min_full_derivation_depth_of_node(alternative) + 1`, reusing `witness_target_depth_budget`'s
established per-BRANCH formula (`SV-EXH-PROOF.7.4.6.9`) instead of inventing a second one, so the two
passes cannot drift apart.

Design points, each deliberate:
* **Strictly additive and monotone** — `.max(budget)` only ever RAISES the flat budget, so no target
  that witnesses today can stop witnessing. An alternative whose depth the fixpoint never resolved
  (a non-terminating one) yields `None` and keeps the old budget exactly.
* **`bypass_fuel` moves with it** (`branch_budget + 1`), keeping the invariant the flat version had.
* **The B-ii pass is byte-identical** — `branch == None` (a rule with no top-level choice) forces no
  alternative, so it keeps the rule-scoped budget.
* **The rule-scoped budget is RESTORED after the loop.** Without that, the mandatory-child and
  seed-sibling tiers below would inherit whichever branch happened to run last — a silent,
  order-dependent budget change to passes this leaf did not touch.
* ⛔ **HONEST BOUND (no silent caps):** scoped to the target's OWN root-`Or` tier, where the defect
  was measured. The child and seed tiers force a branch of a *different* rule and still use the flat
  budget; the same under-funding is possible there in principle, but no case has been observed and
  widening on a hunch would ship untested budget arithmetic. The identical shape also exists in
  `generate_structured_witnesses`' own `'branches:` loop — same reasoning, same bound.
* ⛔ Keyed on derivation structure only — no rule names, no `_lr_` spelling. Engine tier.

**THE ISOLATING SYNTHETIC (tracked: `docs/tasks/artifacts/engine_universal_services/forced_branch_depth_budget.ebnf`).**
Built first, as `.10` demanded twice. `expr` is directly left-recursive behind a choice, so the
eliminator emits the same `expr_lr_base`/`expr_lr_suffix` shape SV's residual lives on; a catch-all
`fullexpr` parses `&&`/`||` itself so those two suffix arms are entered but never commit *for every
seed*; and the `with ( deep )` arm descends a 70-level mandatory chain.

⭐ **One design point in it is load-bearing and was found by measurement, not by reasoning.** The
first draft had no cheap route to the chain rules, so the *plannable* pass targeted each `w01..w50`
on its own behalf, forced branch 2 for each, and the grammar reported `UNKNOWN=0` — i.e. it produced
1 784-style override traffic while failing to reproduce the residual at all. Adding
`decl := "chain" deep ";"` gives the chain rules a cheap route from the entry — exactly as SV's
expression hierarchy is reachable from everywhere else — and the synthetic then reproduces the SV
residual exactly: `UNKNOWN=1 ["expr_lr_suffix"]`, `sample_parse_failures=0`, all four reach passes
failing, and
`[forced-override] rule='expr_lr_suffix' path='root' forced_branch=2/3 outcome=failed
reason="Stimuli generation depth exceeded max_depth=63 while expanding rule 'w24'"` ×2 followed by
`outcome=overridden rendered_branch=0` ×2.

#### Acceptance Checklist (enforced) — slice 2, the fix (`PGEN-ENGINE-UNIVERSAL-SERVICES-0009`)

- [x] **REPRODUCE / ISSUE** — SV: `PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1
  ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017
  --entry-rule systemverilog_file --count 40 --seed 0` → `UNKNOWN=1
  ["select_expression_lr_suffix"]`, 27 probe samples of which **0** carry `with`. **Isolated to a
  synthetic first** (`docs/tasks/artifacts/engine_universal_services/forced_branch_depth_budget.ebnf`,
  driven through the `scratch` slot): `--count 1 --seed 0` → `UNKNOWN=1 ["expr_lr_suffix"]`,
  `sample_parse_failures=0`, all four reach passes reporting 0 witnessed.
- [x] **ROOT CAUSE (WHY + WHERE)** — `[forced-override] rule='select_expression_lr_suffix'
  path='root' forced_branch=2/3 outcome=failed reason="Stimuli generation depth exceeded
  max_depth=67 while expanding rule 'real_number'"` + `outcome=overridden rendered_branch=0`
  (slice 1's `PGEN_REACH_FORCED_OVERRIDE_DUMP=1`, reproduced verbatim on the synthetic as
  `max_depth=63 … 'w24'`). **WHY** = the forced `with (…)` arm's own minimal derivation is deeper
  than the budget the pass granted it; **WHERE** = `generate_target_own_structure_witnesses`
  (`rust/src/ast_pipeline/stimuli_generator.rs`) computing
  `reach_prefix_budget + min_derivation_depths[rule]` once per rule, outside the `'branches:` loop —
  a RULE-scoped depth (`2*24 + 19 = 67`) for a per-BRANCH decision. Confirmed independently by the
  `--max-depth` 24/32/40 ladder (`UNKNOWN` 1→0→0).
- [x] **FIX** — ENGINE tier, and no lower tier exists: the defect is in the witness planner's budget
  arithmetic, not in any grammar (per the 2026-08-11 director ruling, an EBNF must not carry it).
  Per-branch budget from `min_full_derivation_depth_of_node(alternative) + 1`, reusing
  `witness_target_depth_budget`'s formula; `.max(budget)` keeps it monotone; the rule-scoped budget
  is restored before the child/seed tiers. ZERO grammar bytes, ZERO codegen bytes.
- [x] **ADDRESSED (verified)** — **synthetic:** `UNKNOWN 1 → 0 fully_certified=true`,
  `sample_parse_failures=0`, at **all 9** of `--count 1/2/3` × `--seed 0/7/42`, with **0** remaining
  `forced-override` events for the target (the forced branch now renders instead of being
  substituted). **SystemVerilog:** canonical `UNKNOWN 1 → 0 fully_certified=true
  (sample_parse_failures=0)` and probe samples carrying `with` **0 → 1**. Named re-runnable oracle:
  `make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate` → `recognized_basis_green: true`,
  `fully_certified_via_union: true`, `union_unknown: 0`, `union_residual_rules: []`, identical at
  seeds 0/7/42. ⛔ And it does NOT buy this with parse failures, which the rejected global-knob fix
  did: `sample_parse_failures=0` at every seed, against `0→8→17` on the `--max-depth` ladder.
- [x] **NO REGRESSION** — `sv_cert_recognized_union_gate` GREEN against the rebaselined contract,
  deterministic across seeds 0/7/42, `sample_parse_failures=0`. **Cross-grammar cert-coverage sweep
  BEFORE vs AFTER is byte-identical** (`diff` clean, 19 rows × the real before-binary rebuilt from
  HEAD — not a re-run of one arm): json / regex / rtl_frontend / systemverilog_preprocessor / vhdl /
  scratch each `UNKNOWN=0 fully_certified=true spf=0` at seeds 0/7, unchanged. ⛔ The 4 rows for
  `ebnf` / `return_annotation` / `rtl_const_expr` / `semantic_annotation` are recorded as
  `OUT-OF-SCOPE: no generated parser registered` with the tool's own reason, never as blank rows —
  two empty result sets diff clean (`CI-PARITY-GATE-ROT.24`). `ast_shape_contract_gate` GREEN;
  `clippy_on_rust_change` clean (strict source + strict generated). Generated parsers untouched: the
  change is in the stimuli generator, and codegen has no path through it.
- [x] **LOCKSTEP** — the union contract
  (`rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`) rebaselined
  with a full `rebaseline_note` carrying the WHY+WHERE and the ⛔ do-not-raise-`--max-depth` finding;
  the tracked probe artifact; `TOOLBOX.md` §6.4's budget warning; the book
  *Diagnosing UNKNOWNs*; `docs/TASK_TREE.md`; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `MEMORY.md`.
  ⛔ The DONE-BAR register is deliberately **UNCHANGED**: `systemverilog` stays `Mostly Done`.
  Certificate-coverage is one proof surface, and SV's release bar is gated on the corpus axis
  (`SV-CORPUS-GRAD.13` — only 46.3 % of the corpus adjudicated), so promoting the row on this result
  would be a claim the other axes do not support.

### `.12` — the SAME mis-scoped budget survives in three more forcing tiers, unmeasured (`todo`, opened 2026-08-12 session #219 by `.11` slice 2's honest bound)

`.11` slice 2 fixed a budget that was scoped to a rule's **shallowest** alternative on a pass that
forces a specific, possibly much deeper one. It fixed **one** of four places that do this. This leaf
exists so the other three are OWNED rather than living as a sentence in a closed leaf — the
every-finding-is-FIXED rule, where routing decides *when*, not *whether*.

**THE THREE SITES** (`rust/src/ast_pipeline/stimuli_generator.rs`), each forcing a root-`Or` branch
while `self.config.max_depth` holds the flat `reach_prefix_budget + min_derivation_depths[rule]`:

1. `generate_target_own_structure_witnesses` — the **mandatory-CHILD** tier. Forces a branch of a
   child rule; the child's forced alternative can be deeper than the child's shallowest one.
2. `generate_target_own_structure_witnesses` — the **seed-SIBLING** tier (`.10` mechanism 2). Same
   shape, on the seed rule.
3. `generate_structured_witnesses` — its own `'branches:` loop, which forces R's root-`Or` branches
   under a composed prelude + head-leaf pin. This is the closest twin of the site `.11` fixed.

⛔ **NOT a code change yet, and deliberately so.** No case has been observed at any of the three, and
`.11` slice 2's whole argument was that budget arithmetic must be scoped by measurement rather than
by analogy — shipping the same edit three more times on a hunch would be exactly the reasoning that
leaf refused. The cost of being wrong is not symmetric: each site raises a budget, and a raised
budget is what buys `UNKNOWN` at the price of `sample_parse_failures` when it is not the real cause.

**HOW TO MEASURE IT (no new instrument needed — `.11` slice 1 already built it).** The signature is
already observable: run any grammar's cert coverage under `PGEN_REACH_FORCED_OVERRIDE_DUMP=1` and
look for `outcome=failed reason="… depth exceeded …"` on a rule whose forced site belongs to one of
the three tiers. A cheap sweep is the natural first step — the same nine-grammar sweep `.11` slice 2
used for its regression arm, re-read for `forced-override` records instead of headline equality.

**Acceptance:** either (a) a measured case at one or more of the three sites, fixed by the same
per-branch formula and proven on an isolating synthetic first, with the cross-grammar sweep
byte-identical outside the target; or (b) a priced REFUSAL — the sweep run, the census of
`depth exceeded` forced-override records published, and the conclusion recorded that no site
reproduces, so the flat budget stays. ⛔ (b) is a first-class outcome, not a failure to fix; what is
not acceptable is leaving the question unasked.

### `.13` — INDIRECT left recursion is not eliminated at all, and the runtime guard REJECTS the derivation: an LRM-legal SystemVerilog cast is unparseable (`in progress` — ⭐ **acceptance (a) CLOSED by slice 1**, `PGEN-ENGINE-UNIVERSAL-SERVICES-0011`, 2026-08-12 session #221; opened 2026-08-12 session #220 by `GRAMMAR-WELLFORMED.A2.6`, with a minimal repro)

⭐⭐ **THIS IS THE ENGINE HALF OF THE SAME BOUNDARY `A2.5` DREW, one shape further out.** `A2.5`
taught the eliminator the **inline direct** shape (`X := X op Y | seed`) by normalizing it into the
**wrapper** shape the planner already matched. Neither covers an **indirect** cycle — `X := … A …`,
`A := … B …`, `B := … X …` — and nothing else does either. The runtime cycle guard is not a fallback
for it: it rejects re-entry at the same input position, so every derivation that needs the recursion
at the seed position is simply **unreachable**.

**MEASURED (the repro is three commands, `GRAMMAR-WELLFORMED.A2.6`).** The first cycle SystemVerilog's
lint prints is `casting_type -> constant_primary -> constant_primary_sv_2017 -> constant_cast ->
casting_type`. IEEE 1800-2017 A.8.4 makes that cycle *derivable* text — `casting_type ::= … |
constant_primary`, `constant_primary ::= … | constant_cast`, `constant_cast ::= casting_type ' (
constant_expression )` — so `int'(2)'(3)` is a legal cast chain:

```text
int'(3)      → parse_full passed
int'(2)'(3)  → REJECT, furthest_position=40
             💥 Infinite recursion detected in rule 'casting_type' at position 32
             ❌ Exiting rule 'constant_cast' with error: InvalidSyntax { message: "Infinite recursion detected" }
```

**SIZE (census, `--lint-grammar` headline, every buildable grammar):** `systemverilog` **30** surviving
cycles, `systemverilog_lrm_profiled_wrapper` (raw Annex A) **23**, `ebnf` **5**, every other grammar
**0**. So this is not an SV-only shape — it is an ENGINE gap whose only *current* victims are the two
biggest grammars, which is exactly the pattern `.2`'s scar-tissue census predicts.

⛔ **Not scoped as "add an algorithm" until it is measured per cycle.** The 30 are not 30 defects:
a surviving cycle only costs *reachability of the left-recursive derivations*, and for many of the 30
the LRM may never put a legal string on that path. The honest first slice is a **per-cycle
adjudication** — for each surviving cycle, an LRM-grounded input that needs the recursion, and a probe
verdict — so the fix is priced against real lost text rather than against a count. `casting_type` is
the first row and it is already REJECT.

**Prior art to consult before designing** ([[feedback_prior_art_before_design]]): Warth/Douglass/Millstein
(*Packrat parsers can support left recursion*, PEPM 2008) — the seed-growing runtime technique, which
is what a *guard* could become instead of a rejection; Medeiros et al. (arXiv 1207.0443) on
left-recursion semantics for PEGs; and the classic Paull indirect-LR elimination, whose cost is the
grammar blow-up `.7`'s taxonomy would price as a gen-AST → gen-AST rewrite (free at parse time by
construction). ⭐ The choice between *eliminate at generation* and *grow the seed at runtime* is the
real design call, and `.7`'s taxonomy says why it matters: the first is free at parse time, the second
is not.

**Acceptance:** (a) the per-cycle adjudication table (all 30 SV + 5 ebnf, each with an LRM/spec-grounded
input and a probe verdict); (b) a prior-art-grounded design decision recorded in `docs/decisions/`;
(c) the fix, proven first on an isolating synthetic in the combinator suite (the indirect shape has no
case there today — `recursion_guarded_memo_isolation` covers the guard's *memo* interaction, not the
lost derivation); (d) `int'(2)'(3)` parses, with the SV corpus and the 6 fully-certified grammars
byte-identical outside the target.

#### ⭐⭐ ROUTED IN — a SECOND victim of the SAME cycle, and the first one with CORPUS pricing (`SV-CORPUS-GRAD.13c.2b`, 2026-08-12 session #221)

This leaf opened with `int'(2)'(3)` — a *constructed* LRM-legal chain. It now also owns a defect that
arrived from the opposite direction: a **vendored corpus row** that `SV-CORPUS-GRAD.13c.2` had
adjudicated `DEFECT` months before this cycle was named, and whose own leaf's hypothesis (a missing
`constant_primary` alternative) was refuted by the diagnosis.

```systemverilog
package p; parameter logic [7:0] K = 8'(1); endpackage   // REJECT
module  m; logic [7:0] k; initial k = 8'(1);  endmodule  // ACCEPT
```

Same cycle, entered one rule earlier:

```text
constant_expression → constant_expression_operand → constant_primary → constant_primary_sv_2017
  → constant_cast → casting_type → constant_primary   💥 Infinite recursion detected, position 383
```

⭐ **What it adds that `int'(2)'(3)` could not:**

1. **A price.** `docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/corpus_row_cast_bisect.py`
   removes only the `N'` size-cast prefix from OpenTitan's `top_darjeeling_rnd_cnst_pkg.sv` (22
   occurrences) and `top_earlgrey_rnd_cnst_pkg.sv` (11) and both flip **REJECT → parse_full passed**.
   So this cycle is the *sole* blocker of 2 real corpus rows — the fix has a measured corpus delta,
   not just a constructed one.
2. **A sharper statement of what a surviving cycle costs.** The guard fires in the PASSING arms too:
   on `parameter logic [7:0] K = W'(1);` the trace shows `💥 Infinite recursion detected in rule
   'constant_primary'` and then `🏁 Rule 'casting_type' selected branch 1/5`. ⇒ a surviving cycle
   costs **nothing until it is the only road**. The cost is not "the cycle exists"; it is "no other
   alternative of the re-entered rule can match this text". That is the shape the per-cycle
   adjudication (acceptance (a)) must actually test for, and it is why a count of cycles is not a
   count of defects — the leaf already said so, and this measures the mechanism behind it.
3. **A fidelity proof that closes the grammar-tier escape.**
   `constant_primary_lrm_alternative_audit.py` shows `constant_primary_sv_2017` (15), `_sv_2023` (16)
   and `casting_type` (5) are **order-identical** to the LRM extraction. Hand-splitting the cycle in
   `systemverilog.ebnf` would trade a proven byte-for-byte Annex A transcription for a workaround —
   the third reason [[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]] gives for
   refusing a grammar-tier repair, now instantiated with a measurement.

⛔ **Acceptance (d) therefore grows by two rows**: when the fix lands, `defect_constant_size_cast.sv`
and `defect_constant_size_cast_corpus_shape.sv` must flip to ACCEPT (their `MANIFEST.tsv` `expect`
column re-baselined in the same commit), and the two OpenTitan rows must parse unmodified.

#### ✅ SLICE 1 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0011`, 2026-08-12 session #221) — acceptance (a) is CLOSED: the per-cycle adjudication, and the leaf's own caution is refuted

> **Two numbers changed, in opposite directions. The worklist is 3.5× SMALLER than the headline
> said, and the defect rate is far HIGHER than this leaf guessed.** Artifacts + re-runnable
> instruments: `docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/`. ZERO grammar
> bytes, ZERO Rust bytes.

**(1) `left_recursion_unhandled=30` COUNTS RULE ROWS, NOT CYCLES.** `detect_left_recursion`
(`grammar_wellformedness.rs:1181`) starts a DFS from **every** rule and reports any that closes back
on itself, so one 12-rule cycle is printed 12 times. Canonicalising each reported path by rotation
(`canonicalize_lint_cycles.py`, which REFUSES a row that is not the `rule -> … -> rule` shape the
lint guarantees, and refuses a headline that disagrees with the rows it parsed):

```text
grammars/systemverilog.ebnf: 30 reported rule rows -> 7 DISTINCT cycles
grammars/ebnf.ebnf:           5 reported rule rows -> 3 DISTINCT cycles
```

⇒ acceptance (a)'s stated worklist — *"all 30 SV + 5 ebnf"* — is **10 rows**, not 35. ⛔ The leaf
inherited "30" from the lint headline and priced the work off it; the headline was never wrong, it
was answering a different question.

**(2) THE ADJUDICATION — 8 OF THE 10 COST REAL TEXT** (`adjudicate.py`, one spec-grounded input per
cycle, both dialect profiles where the cycle has a twin):

| cycle | knot | input that NEEDS the recursion | verdict |
|---|---|---|---|
| SV-1 / SV-5 | cast/call | `initial k = w()'(1);` — casting_type is a `constant_function_call` (A.8.4 + §6.24.1) | **REJECT** both profiles, `furthest_position` at the `'` |
| SV-2 / SV-3 | cast/const | `parameter logic [7:0] K = 8'(1);` | **REJECT** both profiles (`SV-CORPUS-GRAD.13c.2b`, 2 corpus rows) |
| SV-6 / SV-7 | property | `property p; (a \|=> b) -> c; endproperty` — `property_expr implies property_expr` | **REJECT** both profiles |
| SV-4 | class-scope | `typedef A::B::C::D t4;` (sv_2023) | ACCEPT — **DEAD-BUT-COVERED** |
| EBNF-1 | return-expr | `-> $1 + $2` | frontend rc=0, **meta-parser rc=1** |
| EBNF-2 | return-expr | `-> $1 ? $2 : $3` | frontend rc=0, **meta-parser rc=1** |
| EBNF-3 | return-expr | `-> $1.name` | both rc=0 — **DEAD-BUT-COVERED** |

⭐⭐ **THAT REFUTES THIS LEAF'S OWN CAUTION.** `.13` opened with *"the 30 are not 30 defects: a
surviving cycle only costs reachability of the left-recursive derivations, and for many of the 30 the
LRM may never put a legal string on that path."* The caution was right to demand measurement and
wrong about the outcome — **8 of 10 reject text the standard licenses**, including two constructs
nobody had named before this slice (a function-call cast size, and property-level implication).

⭐ **AND THE FIX SURFACE IS 3 KNOTS, NOT 30 CYCLES.** SV-1/2/3/5 all pass through the single edge
`casting_type → constant_primary`; SV-6/7 through the single alternative `property_expr implies
property_expr` (`systemverilog.ebnf:4726`); EBNF-1/2/3 through `return_expression`. A design that
breaks three knots closes every measured loss. ⇒ the "grammar blow-up" objection the leaf raises
against Paull elimination should be priced against **3 rules**, not 30 cycles, before it is accepted
as an argument for the runtime (seed-growing) alternative.

⛔ **TWO ROWS ARE `DEAD-BUT-COVERED`, AND THE VERDICT IS EARNED, NOT ASSERTED.** An accepting probe
proves nothing about a cycle — the `SV-CORPUS-GRAD.13c.2b` trap, in the other direction. Both rows
carry mechanical evidence that `adjudicate.py` re-checks every run and fails on:

- **SV-4** — the traced run shows BOTH `💥 Infinite recursion detected in rule
  'incomplete_class_scoped_type'` AND `🏁 Rule 'data_type_or_incomplete_class_scoped_type_sv_2023'
  selected branch 1/2`. The guard killed the recursive alternative; `data_type`'s own scoped-type
  path served the text.
- **EBNF-3** — the arm-2 AST holds `positional_reference` + `property_access_suffix` and **no**
  `member_access` node, so `member_access_return` never fired.

⚠️ **HONEST BOUND:** `DEAD-BUT-COVERED` is *"no LRM-grounded input has been found that only this
alternative can derive"*, not a proof that none exists. It is the weaker claim on purpose.

⛔ **TWO TRAPS THIS MEASUREMENT WALKED INTO** — recorded because both nearly published a wrong table:

1. **`property p; a -> b; endproperty` is a passing probe that exercises NOTHING.** `->` is also an
   ordinary binary expression operator (A.8.6), so `sequence_expr` swallows it and
   `prop_primary_sv_2017 selected branch 1/30` — the left-recursive alternative is branch 26/30 and
   never fires. Forcing it needs a non-expression left operand: `(a |=> b) -> c`.
2. **`ebnf_dual_run_diff` exits 0 on an arm-2 rejection unless `--emit-ast-json` is passed** — without
   it only arm 1 (the hand-written frontend) is reported. The first draft of `adjudicate.py` omitted
   the flag and printed three green ebnf rows where two are rejections. Same shape as
   `CI-PARITY-GATE-ROT.24`'s under-featured-binary trap: an incomplete invocation yields a
   clean-looking result.

⭐ **EBNF-1/EBNF-2 ARE ALSO A FRONTEND-REPLACEMENT BLOCKER.** Both are accepted by the hand-written
`ebnf_frontend` and rejected by the parser generated from `grammars/ebnf.ebnf`, so `ebnf.ebnf` cannot
replace the frontend while these cycles survive — a concrete, named reason for
`LANG-CAPABILITY-AUDIT.10.6`'s question, measured rather than estimated.

**REMAINING ON `.13`:** acceptance (b) the prior-art-grounded design decision, (c) the isolating
synthetic in the combinator suite, (d) the fix + the flips (now: `int'(2)'(3)`, the two
`defect_constant_size_cast*` reproducers, the two OpenTitan rows, the six REJECT rows and two arm-2
`rc=1` rows of `adjudicate.py`).

#### ROUTED OUT of slice 1 — the SVA `implies` KEYWORD does not exist in the grammar (→ `LRM-GRAMMAR-FIDELITY.1b`)

Found while building the SV-6 probe, and **not** part of this leaf: `implies` in
`grammars/systemverilog.ebnf` is `implies := trivia "->"` (`:6388`) — the name the LRM extractor's
`PUNCTUATION_TOKEN_NAMES` table gives the `->` operator glyph
(`tools/extract_systemverilog_lrm_profiles.py:133`). But IEEE 1800-2017 A.2.10 writes
`property_expr ::= … | property_expr implies property_expr` where `implies` is a **reserved keyword**
(§16.12.7). Both spellings collapse onto the same rule name. Measured: `grep -c kw_implies
grammars/systemverilog.ebnf` = **0** while `kw_iff_ee1c009e` has **18** uses — the sibling operator
in the very same production was extracted as a keyword — and
`property p; a implies b; endproperty` REJECTs at `furthest_position=39`. ⇒ an over-REJECTION of
LRM-legal SystemVerilog, of exactly the metachar-collision silent-drop class `LRM-GRAMMAR-FIDELITY`
was created to own. Routed there rather than worked here.
