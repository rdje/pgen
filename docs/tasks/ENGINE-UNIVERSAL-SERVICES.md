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

### `.10` — the cert generator cannot witness the rules the ENGINE synthesizes (`todo`, opened 2026-08-11 by `GRAMMAR-WELLFORMED.A2.5`; ⛔ **BLOCKER for returning SV's union UNKNOWN to 0**)

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
