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
  ⛔ **THIRD ABANDONMENT, 2026-08-13 (`.17` slice 2):** a confirmatory full sweep was abandoned at
  **7 h 21 m**, its log untouched for the last 7 h 13 m, with every other test — including both
  byte-identity gates — already reported `ok`. The three measurements now read **68 min → 2 h 27 m →
  7 h 21 m**, all on the same two tests, which is the cost of an unbounded ceiling stated plainly.
  ⭐ Two process lessons, recorded because they are cheaper than a fourth data point: the repository's
  own precedent for a confirmatory sweep is `--lib -- --skip deep_nesting` (used by `.13` slice 4b and
  cited three times in this file), and `scripts/run_with_memory_guard.sh` has carried `--timeout-s`
  all along. The 7 h run used neither. A bounded timeout is not optional on a background job — it is
  the difference between "still running" and "hung", and without it those two states are
  indistinguishable from the outside.
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

### `.13` — INDIRECT left recursion is not eliminated at all, and the runtime guard REJECTS the derivation: an LRM-legal SystemVerilog cast is unparseable (`in progress` — ⭐ **acceptance (a) CLOSED by slice 1** `PGEN-ENGINE-UNIVERSAL-SERVICES-0011`, **(b) CLOSED by slice 2** `PGEN-ENGINE-UNIVERSAL-SERVICES-0012` (both 2026-08-12 session #221), **the isolating synthetic by slice 3** `-0013`, and **the DERIVED plan + base-rule criterion by slice 4** `PGEN-ENGINE-UNIVERSAL-SERVICES-0015` (session #223, which moved the target rule to `constant_primary_sv_2017`/`_sv_2023` and routed the property knot out to `.15`); opened 2026-08-12 session #220 by `GRAMMAR-WELLFORMED.A2.6`, with a minimal repro)

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

**Prior art to consult before designing** ([[feedback_read_prior_art_before_designing]]): Warth/Douglass/Millstein
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
   clean-looking result. ⭐ **ROUTED OUT as a tool defect in its own right →
   `LANG-CAPABILITY-AUDIT.10.6a`**, with the measurement that BOUNDS it: the gate and `TOOLBOX.md`
   §1.9's recipe both pass `--envelope-differential`, which DOES run arm 2 and fails correctly, so
   the exposure is ad-hoc use only — not a gate defect. That bound is recorded HERE, at the routing
   point, rather than left for the receiving leaf to rediscover (the `ROUTING-EVIDENCE` doctrine).
   The documentation half is fixed in the same commit; the tool half is parked with a named trigger.

⭐ **EBNF-1/EBNF-2 ARE ALSO A FRONTEND-REPLACEMENT BLOCKER.** Both are accepted by the hand-written
`ebnf_frontend` and rejected by the parser generated from `grammars/ebnf.ebnf`, so `ebnf.ebnf` cannot
replace the frontend while these cycles survive — a concrete, named reason for
`LANG-CAPABILITY-AUDIT.10.6`'s question, measured rather than estimated.

**REMAINING ON `.13` after slice 1:** acceptance (b) the prior-art-grounded design decision, (c) the
isolating synthetic in the combinator suite, (d) the fix + the flips (now: `int'(2)'(3)`, the two
`defect_constant_size_cast*` reproducers, the two OpenTitan rows, the six REJECT rows and two arm-2
`rc=1` rows of `adjudicate.py`).

#### ✅ SLICE 2 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0012`, 2026-08-12 session #221) — acceptance (b) is CLOSED: eliminate at GENERATION, do not grow the seed at runtime

Full record + the prior-art survey:
[[project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime]]. ZERO grammar
bytes, ZERO Rust bytes. The short form:

- ⛔ **The literature does NOT settle it, and both answers are in production.** CPython's PEG parser
  (PEP 617 / `pegen`) grows seeds in the memo cache and supports indirect + mutual left recursion;
  **ANTLR4 rewrites DIRECT left recursion and REFUSES indirect left recursion**, by a stated
  engineering decision, calling the classical elimination algorithm's exponential blow-up *"wholly
  unworkable in practice"*. Academic anchors: Warth et al. (PEPM 2008), Medeiros et al.
  (arXiv 1207.0443 / SCP 2014), Tratt (2010), *Eliminating Left Recursion without the Epsilon*
  (arXiv 1908.10888).
- **PGEN's second non-negotiable decides it.** Seed growing is a per-parse protocol paid on every
  input forever; elimination is a build-time rewrite, free at parse time by construction — the exact
  axis `.7`'s taxonomy prices an engine feature on.
- ⭐ **And "those rules are cold anyway" was checked, not assumed** (`--dump-rule-entry-counts-json`,
  three diverse corpus files): knot A is **15 of SV's 1 481 rules (1.0 %)** and carries
  **2.85 % / 3.24 % / 3.29 %** of all rule entries — about **3× its fair share**. A runtime protocol
  there taxes the hot path.
- ⭐ **ANTLR4's blow-up objection is priced against the wrong denominator here.** It is a property of
  eliminating an *arbitrary* grammar; slice 1 measured PGEN's real surface as **3 knots**. If a
  future grammar ever presents a knot that genuinely explodes, the honest answer is ANTLR4's —
  refuse it with a named diagnostic — not to buy a runtime protocol for every grammar that does not
  need one. ⛔ This decision explicitly does NOT pre-authorise that rescue.
- ⭐ **`.8`'s `lr_chain_fold` already paid the objection the literature raises against elimination**
  (*"it changes the resulting trees"*). Generalising that fold from one rule to a mutually-recursive
  **set** is the real work of acceptance (c)/(d), and is where the next slice starts.

#### ✅ SLICE 3 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0013`, 2026-08-12 session #222) — the ISOLATING SYNTHETIC for acceptance (c), and the design it REFUTES

> **The synthetic that acceptance (c) asks for now exists, and measuring it moved the fix's target
> rule.** Artifacts + the re-runnable dual-oracle driver:
> `docs/tasks/artifacts/engine_universal_services/indirect_lr/` (`README.md` carries the full
> matrix). ZERO grammar bytes; the only Rust is a new INSTRUMENT (`--interpret-parse`), no engine
> or codegen change.

**THE SYNTHETIC.** Six rules carrying knot A edge for edge — `prim := lit | cast_expr`,
`cast_expr := ct "'" "(" lit ")"`, `ct := kw | prim`, entry OUTSIDE the cycle as `source_text` is in
SystemVerilog. `--lint-grammar` reports the same single surviving cycle; the generated parser
reproduces the SystemVerilog signature exactly (`t'(n)` — one cast level, seeded by `ct`'s own
alternative — ACCEPTS, and everything that needs the cycle REJECTS). The trace names the mechanism:

```text
💥 Infinite recursion detected in rule 'prim' at position 0
🔙 Speculative parse failed … backtracked to position 0 (rule=ct)
🏁 Rule 'ct' selected branch 1/2 consuming 1 chars     ← kw = "t", the ONLY branch left
🏁 Rule 'prim' selected branch 2/2 consuming 5 chars   ← "t'(n)", and there it stops
```

⭐ Both of this leaf's real victims are one table now: `t'(n)'(n)` is `int'(2)'(3)`, and `n'(n)` —
seeded by `prim`'s own alternative instead of `ct`'s — is `SV-CORPUS-GRAD.13c.2b`'s `8'(1)`.

⛔⛔ **(1) ELIMINATING AT THE RULE THE LINT NAMES IS A REGRESSION, NOT A FIX.** P2 rewrites `ct`
(= `casting_type`, the rule every lint row and this leaf's own opening paragraph names first) into
`ct_base ( ct_suffix )*` — what a generalized `detect_left_recursive_chain_plan` produces if it
simply follows the bare-reference chain. Measured: `t'(n)` goes **accept → reject** and nothing
recovers. PGEN's `*` is greedy and does **not** backtrack its iteration count
(`generated/systemverilog_parser.rs:7463` — `loop { if let Some(node) = try_parse(…) { … } else { break } }`),
so the eliminated `ct` swallows the whole chain and `cast_expr`'s own trailing `"'" "(" lit ")"` can
never match. The verdict proves itself: `reject@5` on a five-byte input = the parser reached the end
and still needed four more bytes.

⭐ **(2) THE TARGET IS THE CONSUMER RULE, AND P3 IS ITS SPECIFICATION.** Rewriting `prim`
(= `constant_primary` — what a SystemVerilog expression actually asks for; nothing outside the cycle
ever asks for a `casting_type`) accepts all five inputs on **both** oracles, including the
two-traversal `t'(n)'(n)'(n)`. The transformation acceptance (d) must synthesize:

- the **consumer** rule becomes `base ( suffix )*`;
- `suffix` = the cycle's residual — everything after the leading back-reference in the rule that
  closes the cycle (`constant_cast`'s `tick lparen constant_expression rparen`);
- `base` = every non-cyclic left corner, **including a CLONE of each intermediate rule with the
  cycle edge removed** (`cast_expr_seed`; in SV, `constant_cast` re-emitted over a `casting_type`
  shorn of its `constant_primary` arm).

⇒ ANTLR4's blow-up objection lands on **one clone per intermediate on the cycle path**, not an
exponential closure — but each clone is a NEW RULE NAME IN THE TYPED AST, so acceptance (d) grows an
`ast_shape_contract` obligation it did not have. That is a cost slice 2's decision record did not
price, and it is recorded there now.

⛔ **(3) THE CHEAP ORACLE IS WRONG ON THIS SHAPE — SPLIT OUT AS `.14`.** The interpreter disagrees
with the generated parser on two of P1's five rows, **in both directions** (`t'(n)` gen=accept /
interp=reject; `n'(n)` gen=reject / interp=accept), and agrees on every P2/P3 row — so the
divergence is specific to a SURVIVING cycle. `PARSE-HARNESS.6.1`'s one indirect-LR case
(`recursion_guarded_memo_isolation`) is green because its cycle is escapable one hop in.

⛔ **CORRECTION, same session, before the next slice started — `.14` does NOT block acceptance (c),
and the first draft of this section said it did.** The claim was reasoned, not checked, and two
in-tree facts refute it:

1. **Closing `.14` would not make the case green either.** A combinator case is clean only when
   `agreed && anchor_ok` (`parse_harness_combinator_suite.rs`, `CombinatorCaseReport::is_clean`).
   Making the interpreter mirror `check_cycle_id` would move the two oracles to AGREEING on REJECT —
   `agreed` yes, `anchor_ok` still no, because the anchor is the spec-reasoned ACCEPT. So the case is
   RED until the ENGINE parses the text, i.e. until acceptance (d).
2. **And (d) makes it green without `.14`.** Both sides run the same `transform_from_raw_ast`
   LR-elimination (`parse_harness_combinator_suite.rs:514`), so once the eliminator handles the
   shape they consume the identical eliminated tree — which is why P2/P3, the hand-eliminated
   probes, agree on every row. The precedent is exact: bare DIRECT LR was
   `DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE` while its cycle survived and became a green first-class
   case, on the same source grammar, the moment `GRAMMAR-WELLFORMED.A2.5` eliminated it
   (`docs/tasks/PARSE-HARNESS.md:1311`).

⇒ **(c) and (d) land TOGETHER, and `.14` is an independent oracle defect running in parallel** — it
does not gate this leaf, and this leaf's fix shrinks its exposure without closing it. The frontier
after this slice is therefore **`.13` (d), the fix itself**.

⭐ **THE INSTRUMENT THIS SLICE HAD TO BUILD** (`--interpret-parse`, TOOLBOX §1.5b). PGEN had three
ways to parse an arbitrary grammar and none answered the question from a shell: §1.3 costs a
`focus_scratch` + relink per edit, §1.4/§1.5 are Rust APIs. So the unit in which every left-recursion
defect on this tree has actually been diagnosed — a five-rule synthetic — could not be measured
without a multi-minute rebuild or a source edit. It is now one command, and the per-rule entry probe
(`--interpret-entry-rule`) is what localized this slice's rejection to `cast_expr` in six seconds.

⚠️ **AND THE FIRST DRAFT OF THIS MEASUREMENT WAS WRONG BECAUSE IT TRUSTED THAT INSTRUMENT ALONE.**
The interpreter-only table had P1's `t'(n)` and `n'(n)` rows inverted — not under-reported,
MISREPORTED — and it read as "even one cast level fails", which would have sent the fix after a
mechanism that does not exist. `probe.sh` therefore runs BOTH oracles by construction and prints an
`AGREE` column; `--interp-only` exists for a fast re-read and can never back a claim on its own.
⇒ ⭐ **the lesson is the dual of `SV-CORPUS-GRAD.13c.2b`'s**: an ACCEPT is not evidence until you
name the winning branch, and a REJECT is not evidence until you name the rejecting mechanism — a
verdict from an oracle that is authoritative *by verification* is not evidence at all on a shape
outside what verified it.

**REMAINING ON `.13` after slice 3:** acceptance (c) the combinator case (⛔ BLOCKED on `.14`), and
(d) the fix + the flips. Acceptance (a)+(b) are closed.
⛔ **Both halves of that sentence were corrected the same session** — (c) is NOT blocked on `.14`
(see the correction block above), and slice 4 below re-scopes (d).

##### Acceptance Checklist (enforced) — slice 3, the instrument + the synthetic

- [x] **REPRODUCE / ISSUE** — knot A reproduced on a six-rule synthetic through the real generated
  parser (scratch slot): `parseability_probe --parse scratch` on `t'(n)'(n)` →
  `Parser did not consume full input at position 5 [furthest_position=3]`, while `t'(n)` passes —
  the SystemVerilog `int'(3)` / `int'(2)'(3)` signature exactly. Full matrix (3 probes × 5 inputs ×
  2 oracles): `docs/tasks/artifacts/engine_universal_services/indirect_lr/README.md`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_TRACE_VERBOSITY=debug parseability_probe --parse scratch
  … --trace-rules ct,prim,cast_expr` names both mechanism and site: `💥 Infinite recursion detected
  in rule 'prim' at position 0` → `🔙 Speculative parse failed … (rule=ct)` → `🏁 Rule 'ct' selected
  branch 1/2 consuming 1 chars`. The guard kills `ct`'s branch 2/2 at the seed position, so `ct` is
  pinned to its non-recursive alternative and `prim` tops out at ONE cast level. The P2 regression's
  own root cause is the non-backtracking `*` at `generated/systemverilog_parser.rs:7463`, evidenced
  by `reject@5` on a 5-byte input under a rule needing 4 more bytes.
- [x] **FIX** — fix-hierarchy tier = **new tooling / instrument** (no engine, grammar, codegen or
  runtime change): `ast_pipeline --interpret-parse <FILE>` + `--interpret-entry-rule` +
  `--interpret-parse-ast-json` (`rust/src/main.rs`, `run_interpret_parse`), the three tracked
  synthetics and the dual-oracle `probe.sh`. Registered in the same commit, per the
  instrument-registration rule: `INTERPRET-PARSE:`/`--interpret-parse` added to `DIAGNOSIS_SIG`
  (`scripts/check_diagnosis_evidence.sh`) and to `TOOLBOX.md`'s group-1 table + new §1.5b + the
  quick chooser + the book mirror. Why no lower tier: the engine fix is acceptance (d) and this
  slice deliberately does not attempt it — measuring first is what moved its target rule.
- [x] **ADDRESSED (verified)** — the acceptance-(c) question is answered with a measurement rather
  than an argument: `probe.sh` REJECT→PASS on the target shape (P3 accepts all 5 inputs on both
  oracles, including the two-traversal `t'(n)'(n)'(n)`), and accept→REJECT on the naive design
  (P2 `t'(n)`), which is what refutes it. Deterministic by construction — fixed grammars × curated
  inputs, no seed. The instrument itself is verified by agreement: 13 of 15 rows match the
  authoritative-by-construction oracle, and the 2 that do not are `.14`.
- [x] **NO REGRESSION** — `make -C rust SHELL=/bin/bash parse_harness_combinator_gate` re-run on the
  landed tree: **35/35 CLEAN, 2 gate tests pass** (`every_structural_combinator_is_byte_identical`,
  `combinator_coverage_is_complete`), byte-identical to the pre-change baseline taken at the top of
  the session. `bash scripts/check_doctrines.sh` green. The scratch slot is restored to its
  committed fixture (`probe.sh` traps EXIT), so no generated artifact or fixture drift survives.
  ⛔ **THAT LAST CLAUSE IS FALSE, and slice 4b MEASURED it**: the trap restored the TRACKED half and
  left `generated/scratch_parser.rs` — git-ignored, hence invisible to `git status` — holding the P3
  synthetic's grammar, which left `certified_grammars_are_byte_identical` and
  `scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast` RED for a day. Fixed in 4b.
  No engine, codegen or grammar byte changed, so the 6 fully-certified grammars are untouched by
  construction — `git diff --stat` names no file under `grammars/` or `generated/`.
- [x] **LOCKSTEP** — `TOOLBOX.md` (§1.5b + quick chooser + group-1 signature table),
  `docs/book/src/diagnosing-unknowns.md` (the book mirror), the slice-2 decision record (the
  consumer-rule refinement + the AST-shape cost), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`.

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

#### ✅ SLICE 4 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0015`, 2026-08-12 session #223) — the PLAN is now DERIVED from the shipped grammars, and it moves the fix's target rule a SECOND time

> **Acceptance (d)'s one free variable — which rule absorbs the chain — is no longer a description
> of a hand-written synthetic. It is a mechanical criterion, unit-tested against slice 3's measured
> P2/P3 outcomes, and run over every grammar in `grammars/`.** Instrument:
> `ast_pipeline <g> --report-indirect-lr-plan` (TOOLBOX §5.5, `rust/src/ast_pipeline/indirect_lr_plan.rs`).
> Artifacts + the re-runnable census:
> `docs/tasks/artifacts/engine_universal_services/indirect_lr/survey/`. ZERO grammar bytes, ZERO
> codegen bytes, ZERO engine-behaviour bytes — the module is pure analysis behind a new read-only flag.

**THE CRITERION.** A candidate base rule `X` is **STARVED** when some rule that OUTLIVES the rewrite
holds `X` at its left corner with a **non-empty residual** — `cast_expr := ct "'" "(" lit ")"` is
exactly that shape — because PGEN's `*` is greedy and never retries at a lower iteration count. Two
refinements, each forced by a measurement rather than by argument, both now unit-tested:

1. ⛔ **An on-route holder is NOT exempt, and the first draft made it one.** The reasoning that
   failed: "the rewrite shears the cycle edge out of the clone it makes of this rule". It shears the
   CLONE and leaves the ORIGINAL standing. With that exemption the survey called `ct` `MAY-ABSORB` —
   and `ct` is precisely the rule slice 3 measured turning `t'(n)` from accept into reject.
2. ⭐ **A holder the rewrite makes UNREACHABLE cannot starve anything**, and `grammars/ebnf.ebnf`
   forced it: `arithmetic_return := return_expression arithmetic_operator return_expression` reads as
   a hazard, but it is named only from `expression_return`, named only from the very alternative the
   rewrite replaces with a clone. Without this the survey rejected `return_expression` — the knot
   slice 1 named for all three `ebnf` cycles.

**THE CENSUS** (`survey/census.txt`; reproduces slice 1's population row for row — 30 / 23 / 5 / 0):

| grammar | cycle rows | covered by a route | candidates | starvation-safe |
|---|---|---|---|---|
| `systemverilog` | 30 | **30** | 12 | **5** |
| `systemverilog_lrm_profiled_wrapper` | 23 | 18 | 11 | 7 |
| `ebnf` | 5 | **5** | 2 | 2 |

⭐⭐ **(1) THE TARGET RULE MOVES AGAIN — `constant_primary` CANNOT BE THE BASE.** Slice 3 concluded
the fix targets `constant_primary`. The shipped grammar declines it:

```text
constant_primary [no_acyclic_seed]: constant_primary -> constant_primary_sv_2017
                                    -> constant_cast -> casting_type -> constant_primary
```

`constant_primary := constant_primary_sv_2017 | constant_primary_sv_2023` — **both** alternatives
reach the cycle, so there is no seed and `X := X_base ( suffix )*` has no `X_base`. The synthetic
could not show this: its one compression — collapsing SystemVerilog's two-hop bare-reference chain
to one hop — is exactly the hop the dialect split lives on. P1's README called that compression
harmless (*"Nothing on the cycle's shape changes"*); for reproducing the DEFECT it was, for choosing
the BASE RULE it was not. ⇒ **the target is `constant_primary_sv_2017` (and `_sv_2023`)**, both
`MAY-ABSORB`, and the survey derives their suffix from `grammars/systemverilog.ebnf` itself:

```text
route alt#11: constant_primary_sv_2017 -> constant_cast -> casting_type -> constant_primary
              -> constant_primary_sv_2017      suffix: tick lparen constant_expression rparen
```

That suffix is the `'(3)` of `int'(2)'(3)` and the `'(1)` of `8'(1)` — this leaf's two victims —
recovered with no synthetic in the loop.

⭐ **(2) ANTLR4's BLOW-UP OBJECTION, PRICED AGAINST THIS REPOSITORY AT LAST.** Slice 3 answered it
with "one clone per intermediate, not an exponential closure", reasoning from a 6-rule synthetic.
The real figure for SV's biggest knot is **13 clones**, because the real cycle is 13 rules long
(`constant_function_call -> call_primary -> call_with_postfix_chain -> chainable_call_initial ->
direct_callable_method_call -> method_call_root -> method_call_receiver ->
method_call_receiver_sv_2017 -> cast -> casting_type -> …`), not the 4-rule cycle the lint prints
first. Still LINEAR in the cycle — the objection does not land — but 13 new rule names in the typed
AST is a materially bigger `ast_shape_contract` obligation than slice 3 recorded. Per knot:
cast/call **13**, method-call receiver **12**, class scope **1**, property **6**, `ebnf` **4**.

⛔⛔ **(3) THE PROPERTY KNOT (SV-6/SV-7) HAS NO STARVATION-SAFE BASE RULE — THIS FIX CANNOT CLOSE
IT.** `prop_primary_sv_2017` / `_sv_2023` are its only candidates and both are STARVED, by
`prop_and_sv_2017 := prop_primary_sv_2017 kw_and prop_and_sv_2017`, a holder that stays reachable
after any rewrite. That is a measured limit on acceptance (d), not a guess, and it re-prices the
leaf: slice 1's "3 knots" closes to **2 knots by this transformation**. ⭐ It is knot-specific, not
shape-specific — the same construct in the RAW Annex A transcription has `property_expr_sv_2017` as
a `MAY-ABSORB` candidate at `clone_cost=1`; what disqualifies the shipped grammar is its
hand-written `prop_and`/`prop_or`/`prop_iff`/`prop_until` precedence cascade, i.e. exactly the
scar tissue `.2`'s census exists to find. ⇒ **ROUTED to a new leaf `.15`** rather than worked here.

**(4) THE SURVEY'S OWN COVERAGE, MEASURED NOT ASSUMED.** The walk follows only a bare leading rule
reference and declines anything else by name. `systemverilog` 30/30 and `ebnf` 5/5 have ZERO such
declines; `systemverilog_lrm_profiled_wrapper` has 5, all one `module_path_*` knot — which notably
closes through `module_path_expression_lr_base`, a rule the EXISTING elimination pass created.

**REMAINING ON `.13` after slice 4:** (c)+(d) together — the transformation itself, now specified
against `constant_primary_sv_2017`/`_sv_2023` (knot A), `method_call_receiver_sv_2017`/`_sv_2023`
and `incomplete_class_scoped_type_sv_2023`, with the property knot routed out to `.15`.

##### Acceptance Checklist (enforced) — slice 4, the survey instrument

- [x] **REPRODUCE / ISSUE** — the gap is reproduced as a MEASUREMENT, not an anecdote:
  `ast_pipeline grammars/systemverilog.ebnf --report-indirect-lr-plan` reports
  `surviving_cycle_rules=30 candidates=12 declined=18`, of which only **5** are starvation-safe,
  and `--lint-grammar`'s own headline (`left_recursion_unhandled=30`) says nothing about any of it.
  Full transcripts: `docs/tasks/artifacts/engine_universal_services/indirect_lr/survey/`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY acceptance (d) could not be written from slice 3 alone, and
  WHERE: `PGEN_INDIRECT_LR_DUMP_ALL=1 ast_pipeline grammars/systemverilog.ebnf
  --report-indirect-lr-plan` prints `constant_primary [no_acyclic_seed]` — the rule slice 3 named as
  the fix's target has NO non-cyclic alternative in the shipped grammar
  (`grammars/systemverilog.ebnf:1641`, `constant_primary := constant_primary_sv_2017 |
  constant_primary_sv_2023`, both on the cycle), so `X := X_base ( suffix )*` has no `X_base` there.
  The synthetic hid it by collapsing exactly the hop the dialect split lives on
  (`p1_knot_a_defect.ebnf`'s `prim := lit | cast_expr` merges `constant_primary` with
  `constant_primary_sv_2017`). The same dump names the replacement and its suffix
  (`route alt#11 … suffix: tick lparen constant_expression rparen`).
- [x] **FIX** — fix-hierarchy tier = **new tooling / instrument** (no engine, grammar, codegen or
  runtime behaviour change): `rust/src/ast_pipeline/indirect_lr_plan.rs` (pure analysis, 5 unit
  tests) + `ast_pipeline --report-indirect-lr-plan` / `--indirect-lr-plan-json` /
  `PGEN_INDIRECT_LR_DUMP_ALL` (`rust/src/main.rs`, `run_indirect_lr_plan_report`). Registered in the
  same commit per the instrument-registration rule: `INDIRECT-LR-SURVEY:`/`--report-indirect-lr-plan`
  added to `DIAGNOSIS_SIG` (`scripts/check_diagnosis_evidence.sh`), to `TOOLBOX.md`'s group-1
  signature table + quick chooser + new §5.5, and to the book mirror
  (`docs/book/src/diagnosing-unknowns.md`). Why no lower tier: the engine rewrite IS acceptance (d),
  and this slice deliberately does not attempt it — measuring first is what moved its target rule,
  for the second time on this leaf.
- [x] **ADDRESSED (verified)** — the criterion is verified against slice 3's independently measured
  ground truth, in BOTH directions and without being told which is which: on `p1_knot_a_defect.ebnf`
  it calls `prim` `MAY-ABSORB` (probe P3 accepts all 5 inputs on both oracles) and `ct` `STARVED`
  (probe P2 turns `t'(n)` accept→reject). Both are unit tests
  (`the_consumer_rule_is_starvation_safe_and_the_lint_named_rule_is_not`), and the derived suffix is
  asserted byte-equal to P3's hand-written `prim_suffix` (`knot_a_route_reproduces_the_p3_suffix`).
  `cargo test --lib indirect_lr_plan` → **5 passed, 0 failed**. The census independently reproduces
  slice 1's grammar population (30 / 23 / 5 / 0), which is the cross-check that the two instruments
  count the same thing. Deterministic by construction — fixed grammars, `rule_order` iteration, no
  seed.
- [x] **NO REGRESSION** — nothing on the generation path is reachable from the new code: the module
  is called only from the new CLI branch, which returns before any generator runs. `git diff --stat`
  names no file under `grammars/` or `generated/`, so the 6 fully-certified grammars are
  byte-identical by construction. `make -C rust SHELL=/bin/bash parse_harness_combinator_gate` →
  **all 35 cases CLEAN (`diverge=0 anchor_miss=0`), 2 gate tests pass**
  (`every_structural_combinator_is_byte_identical`, `combinator_coverage_is_complete`).
  `cargo test --lib indirect_lr_plan` → **5 passed, 0 failed**. `bash scripts/check_doctrines.sh`
  → **18/18**. ⛔ **AMENDED BY SLICE 4b** — the confirmatory
  `cargo test --features "generated_parsers ebnf_dual_run" --lib` sweep was still running when this
  box was first written, and it landed **RED on three tests**: two were a real repository defect
  THIS leaf shipped in slice 3 (fixed in 4b) and one is the pre-existing
  `LANG-CAPABILITY-AUDIT.10.15`. Final, after 4b:
  `cargo test --features "generated_parsers ebnf_dual_run" --lib -- --skip deep_nesting` →
  **1 085 passed / 1 failed / 28 ignored**, the single failure being `.10.15`.
- [x] **LOCKSTEP** — `TOOLBOX.md` (§5.5 + quick chooser + group-1 signature table),
  `scripts/check_diagnosis_evidence.sh`, `docs/book/src/diagnosing-unknowns.md`,
  `docs/tasks/artifacts/engine_universal_services/indirect_lr/survey/README.md`,
  `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.


#### ⛔⛔ SLICE 4b (`PGEN-ENGINE-UNIVERSAL-SERVICES-0016`, 2026-08-13 session #223) — slice 3's `probe.sh` left TWO GATES RED for a day, and slice 3's own no-regression box says it did not

> **The confirmatory sweep slice 4 started at commit time came back RED on three tests. Two are a
> repository defect THIS leaf shipped.** Instrument fixed:
> `docs/tasks/artifacts/engine_universal_services/indirect_lr/probe.sh`. ZERO grammar bytes, ZERO
> Rust bytes.

**SYMPTOM**, on a tree with no uncommitted changes:

```text
test parse_harness_equivalence::gate::certified_grammars_are_byte_identical ... FAILED
  PARSE-HARNESS.5: CERTIFIED grammar(s) regressed — interpreter no longer byte-identical:
    scratch   DIVERGE samples=3 agree=1 diverge=2
      · [Verdict] hello, world! :: interp.accepted=true oracle.accepted=false (interp furthest=7)
test parser_registry::tests::scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast ... FAILED
```

**ROOT CAUSE (WHY + WHERE).** `probe.sh`'s EXIT trap ran `git checkout -- grammars/scratch/scratch.ebnf`
and stopped there. `generated/scratch_parser.rs` is **git-ignored**, so `git checkout` cannot restore
it and `git status` cannot report it: the tracked fixture came back to the greeting grammar while the
generated parser kept the LAST probe's. Measured on the tree slice 3 left behind —
`grep -c "cast_expr\|prim" generated/scratch_parser.rs` = **128** against a fixture containing
neither, with the artifact's mtime (`Aug 12 21:45`) inside slice 3's probe window. The two tests read
the slot as a MATCHED PAIR (interpreter over the `.ebnf`, oracle over the generated parser), so a
half-restore breaks both, and breaks them where no tracked-state check can see it.

⭐ **THE GENERALISABLE FINDING.** "Restore the fixture" is only sound when the fixture IS the whole
state. This slot is a **PAIR** — one tracked file plus one git-ignored artifact derived from it — and
a clean `git status` is not evidence about the second. Promoted to the retrievable layer rather than
left here.

##### Acceptance Checklist (enforced) — slice 4b

- [x] **REPRODUCE / ISSUE** — `cargo test --features "generated_parsers ebnf_dual_run" --lib
  parse_harness_equivalence::gate::certified_grammars_are_byte_identical` → `FAILED`,
  `scratch DIVERGE samples=3 agree=1 diverge=2`, on a clean tree at `09fbec24`.
- [x] **ROOT CAUSE (WHY + WHERE)** — only the tracked half was restored:
  `grep -c "cast_expr\|prim" generated/scratch_parser.rs` = **128** (the P3 synthetic's rules) while
  `grammars/scratch/scratch.ebnf` is the greeting fixture, and the gate's own message names the
  mechanism — `interp.accepted=true … oracle.accepted=false (interp furthest=7)`: the interpreter
  reading the `.ebnf` accepts `hello, world!` and the generated parser, built from a different
  grammar, rejects it at the comma. WHERE: `probe.sh`'s `restore_scratch`, calling `git checkout` on
  a path whose derived artifact is `.gitignore`d.
- [x] **FIX** — fix-hierarchy tier = **instrument repair** (no engine, grammar, codegen or runtime
  change): `restore_scratch` now REGENERATES after checking out (`make -C rust SHELL=/bin/bash
  focus_scratch`, ~80 s on a run that already pays three of them) and prints a named manual recovery
  if that regeneration fails. `bash -n probe.sh` clean. Why no lower tier: nothing else can restore a
  git-ignored artifact — not `git`, not the doctrine enforcer, and not `git status`, which is exactly
  why this survived a full commit workflow.
- [x] **ADDRESSED (verified)** — measured before→after on the same two commands:
  `certified_grammars_are_byte_identical` **FAILED → ok** and
  `scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast` **FAILED → ok**, once
  `make -C rust SHELL=/bin/bash focus_scratch` restored the artifact
  (`grep -c "cast_expr\|prim" generated/scratch_parser.rs` **128 → 0**).
- [x] **NO REGRESSION** — `cargo test --features "generated_parsers ebnf_dual_run" --lib --
  --skip deep_nesting` → **1 085 passed / 1 failed / 28 ignored**; the single failure is
  `unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`, **pre-existing and
  already owned by `LANG-CAPABILITY-AUDIT.10.15`** (`.10.3` deleted the emission and left `.10.4`'s
  assertion behind, 146 commits before this session) — re-observed with a current count, not routed
  anew. Both heavy differential gates (combinator, semantic) are inside that sweep and pass.
  `bash scripts/check_doctrines.sh` → **18/18**. No grammar, codegen or `generated/` source byte
  changed — only the git-ignored `generated/scratch_parser.rs` was regenerated back to its own
  fixture.
- [x] **LOCKSTEP** — the promoted knowledge record, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, slice 3's
  and slice 4's boxes amended above, `docs/TASK_TREE.md`, `MEMORY.md`.

#### ⭐⭐ SLICE 5 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0017`, 2026-08-13 session #224) — THE TRANSFORMATION lands, and a CONTROL row stops the version that would have regressed SystemVerilog

> **PGEN eliminates indirect left recursion now** — and the cast/call knot is NOT what it closes.
> `ebnf` **5 → 0**, SystemVerilog **30 → 28**, every other shipped grammar untouched. Engine module:
> `rust/src/ast_pipeline/indirect_lr_elimination.rs`. ZERO grammar bytes.
>
> ⛔⛔ **THE HEADLINE THIS SLICE NEARLY SHIPPED WAS `30 → 13`, AND IT WAS A REGRESSION.** The
> transformation ran, the lint agreed, every unit test passed, `int'(2)'(3)` and `8'(1)` both flipped
> REJECT → ACCEPT on the real generated parser — and `stimuli/sv/run_adjudication_repros.py` failed
> on a **CONTROL**: `initial k = 8'(1);` — the same cast in a NON-constant expression — went
> **ACCEPT → REJECT**. The two-sided ratchet is the only thing in the chain that could have caught
> it, and the row it caught it on exists precisely because `SV-CORPUS-GRAD.13c.2` insisted a
> reproducer be pinned to its accepting neighbour. See *THE STARVATION CRITERION WAS NOT TRANSITIVE*
> below; the corrected criterion refuses `constant_primary`, and SystemVerilog's cast/call knot is
> re-priced as blocked by the greedy non-backtracking `*`, not by the route walk.

**WHAT LANDED.** The pass runs inside `eliminate_left_recursive_patterns`, after the direct
normalization and the one-hop wrapper planner, on what those left behind. Per starvation-safe
survey candidate it builds

```text
X            := X_lr_base ( X_lr_suffix )*
X_lr_base    := <the author's alternatives, in the author's order, each CYCLIC one replaced
                 IN PLACE by a reference to a sheared CLONE>
X_lr_suffix  := <one branch per route: the route's residuals, innermost first>
<base>_lr_seed_<rule> := <the intermediate, with the cycle-closing edge sheared off>
```

and iterates to a fixpoint, re-surveying after each rewrite.

⭐ **THE FOLD NEEDED ZERO BYTES.** The leaf's own frontier line called the work *"generalizing
`.8`'s `lr_chain_fold` from ONE rule to a mutually-recursive SET"*. It is not the fold that
generalizes — it is the **spec**. A route's per-hop declared annotations **compose** into one
template: each hop's `$1` is filled by the hop below it, and every other `$N` is remapped into the
flattened suffix's capture space by `offset + N`, where `offset` is the total residual length of the
hops *deeper* than this one (the suffix concatenates innermost-first, so the order is the reverse of
the route). Knot A's four hops compose to

```text
{kind: "cast", body: {type: {kind: "constant_primary",
                             body: {kind: "sv_2017", body: $1}},
                      body: $4}}
```

and `lr_chain_fold::fold_lr_chain` consumes it unchanged. Asserted hop for hop in
`the_composed_template_is_the_hand_derived_nesting`; the nesting it produces for `t'(n)'(n)` is the
AST the UN-eliminated grammar declares for the same input, which is the property the whole
transformation exists to preserve.

⛔⛔ **THE STARVATION CRITERION WAS NOT TRANSITIVE, AND THAT IS THE REAL FINDING OF THIS SLICE.**
The survey asked *"does any rule hold the BASE RULE at a left corner with a residual?"*. It must ask
*"does any rule hold the base **or anything TRANSPARENT to it**?"* — because a rule whose alternative
is a **bare reference** (empty residual) adds nothing between its caller and the base, so once the
base becomes `X_base ( X_suffix )*` the transparent rule is **just as greedy**. SystemVerilog, two
lines apart:

```text
casting_type := … | constant_primary          ← bare reference ⇒ TRANSPARENT to constant_primary
cast         := casting_type tick lparen expression rparen   ← holds it WITH a residual
```

Measured: `initial k = 8'(1);` ACCEPT → REJECT (`furthest_position=42` on the minimal form), while
`initial k = 8;` and `parameter logic [7:0] K = 8'(1);` both still pass. The greedy
`constant_primary` swallows `8'(1)` as a cast of its own and `cast` can never match its trailing
`tick lparen expression rparen` — **slice 3's P2 starvation exactly, one hop further out**, where a
direct-holder scan is blind to it.

⭐ **The synthetic could not have found it, and its own README said so.** P3 deleted `ct` and
`cast_expr` because nothing else reached them; the file's header reads *"SystemVerilog reaches
`casting_type` from `cast` too, so a real transformation must ADD the clone and KEEP the
originals"*. Keeping the original is exactly what leaves the starved holder standing. The sentence
was right and nobody had turned it into a criterion.

⇒ `rules_transparent_to` (a least fixed point over empty-residual left-corner edges) now feeds
`collect_starvation_sites`. With it, **`constant_primary` is STARVED** and the pass never plans it:
`starvation-safe candidates: 0/28` on SystemVerilog.

⛔⛔ **SO THE CAST/CALL KNOT IS RE-PRICED: IT IS BLOCKED BY THE GREEDY NON-BACKTRACKING `*`, NOT BY
THE ROUTE WALK.** Chain absorption cannot close it at any base rule, because every candidate is
transparently held by `cast` or `constant_cast` with a residual. That is a different problem from
the one `.13` was opened for, it is the same mechanism `.15` names for the property knot, and it
needs its own leaf → **`.17` NEW** (below). ⇒ acceptance (d) closes **1 of 3 knots** in
SystemVerilog (class-scope, `SV-4`, which slice 1 adjudicated `DEAD-BUT-COVERED`) plus **all 3** in
`ebnf`; the two knots that cost real LRM text are owned by `.15` and `.17`.

⭐ **THE DOMINATOR FINDING STANDS, and it is what the trial guard was for.** Before the transitive
criterion was written, the pass planned `constant_primary_sv_2017` and its own postcondition check
refused it, printing the path:

```text
constant_primary_sv_2017 -> constant_primary_sv_2017_lr_base
  -> …_lr_seed_constant_cast -> …_lr_seed_casting_type -> …_lr_seed_constant_primary
  -> constant_primary_sv_2023      ← the SIBLING dialect arm, not sheared by any route
  -> constant_cast -> casting_type -> constant_primary -> constant_primary_sv_2017
```

The twins are a genuine **mutually-recursive SET** reaching each other through the shared
`constant_primary` spine, and that path re-enters three rules already on the route — so it is not a
*simple* route, and `indirect_lr_plan::collect_routes` refuses non-simple routes by design. Only the
**dominator** puts both arms inside one plan.

⭐ **AND THE CRITERION THAT HID IT WAS INHERITED FROM THE WRONG TRANSFORM.** `no_acyclic_seed` is
sound for the DIRECT/wrapper elimination, which *drops* a left-recursive alternative — a rule with
none left really has nothing to seed from. The indirect transform **clones** it with the cycle edge
sheared, so the seeds come from *under* the cyclic alternative: `constant_primary` has 0 acyclic
alternatives and its two clones carry **13 and 14**. The decline is deleted from the survey; `seeds=`
stays as data.

⭐⭐ **THE GUARD THAT FOUND ALL OF THIS IS PART OF THE PASS, AND IT EARNED ITS KEEP ON ITS FIRST
RUN.** A route set only covers cycles whose every hop exposes a bare leading rule reference, so a
cycle closing through a nullable prefix, a quantifier, a group — or through a rule already on the
route — is invisible to the plan, and a clone would leave it live. Rather than argue the coverage,
the pass **applies each plan to a copy, re-runs `detect_left_recursion`, and commits only if** the
base rule's cycle is gone *and* the total row count strictly fell. Cost: one grammar clone per
rewrite, on grammars whose rewrite count is single-digit. Without it, SystemVerilog would have been
rewritten into a grammar that is still left-recursive — a change that looks like a fix and is not.

⛔⛔ **AND THE FIRST WORKING VERSION SHIPPED AN OVER-ACCEPTANCE — CAUGHT BY READING THE EMITTED
PARSER, NOT BY REASONING ABOUT THE PASS.** `@profiles:` is a **rule-level** annotation, and the
clone builder copied only the per-BRANCH ones. Measured in `generated/systemverilog_parser.rs`:

```text
fn parse_constant_primary_sv_2017 …
    if !self.rule_profile_is_enabled(&["sv_2017", "verilog_2005"]) { …
fn parse_constant_primary_lr_seed_constant_primary_sv_2017 …      ← no gate at all
```

⇒ the sv_2017 primary became reachable under an `sv_2023` parse — the exact failure mode the
strict-LRM default exists to prevent ([[feedback_sv_strict_lrm_compliance_default]]), introduced by
a rewrite that *looks* purely structural. Three separate profile defects came out of pulling that
thread, and all three are fixed and pinned by
`a_profile_gate_survives_on_both_the_clone_and_the_suffix_route`:

1. **The clone lost the rule's gate** (above). Clones now copy the source rule's rule-level
   `semantic_annotations` and its `lexical_follow_restriction` verbatim.
2. **The HELPERS were profile orphans.** `X_lr_base` / `X_lr_suffix` for a gated base rule were
   universal, so they are "present" under a profile in which nothing they reference is satisfiable.
   Measured: `profile_orphans` **0 → 4** (an ERROR-class lint counter) on
   `incomplete_class_scoped_type_sv_2023`. The helpers now inherit the base rule's `@profiles:` —
   ⭐ and **only** `@profiles:`, because it is the one rule-level directive that says *whether the
   rule exists*; `@predicate:`/`@emit_fact:` say what happens when it RUNS, and the base rule still
   runs, so copying those would fire them twice per parse.
3. ⭐⭐ **A SUFFIX BRANCH IS A RULE, NOT AN ALTERNATIVE — because an alternative cannot carry
   `@profiles:`.** The two dialect routes through the shared spine iterate **byte-identical**
   suffixes (`tick lparen constant_expression rparen`) and differ *only* in the AST they declare
   (`{kind: "sv_2017", …}` vs `{kind: "sv_2023", …}`). Inline, the ordered choice would hand the
   sv_2017 template to an sv_2023 parse. Each route therefore gets its own
   `X_lr_suffix_r<N>` rule carrying the route's gate — copied verbatim from the rule on the route
   whose declared list IS the intersection, never synthesized — and `X_lr_suffix` is the ordered
   choice of bare references over them. Where no gate can separate two indistinguishable routes
   that declare different ASTs, the plan is **refused** rather than ordered.

⭐ **A fourth finding fell out of the same work: the route walk is PROFILE-BLIND, and some of the
routes it finds are derivable under no profile at all.** SystemVerilog has **4** — each splicing an
`sv_2017` constant primary onto an `sv_2023` method-call receiver, or the mirror. Their `@profiles`
lists intersect to ∅. They are still **sheared** (the structural cycle is real, and the lint that
verifies this pass is profile-blind too) but they contribute **no suffix branch**, because a branch
for a derivation no profile admits is an over-acceptance wearing a chain's clothes.

⚠️ **ALL FOUR WERE OBSERVED ON THE `constant_primary` PLAN, WHICH THE TRANSITIVE STARVATION
CRITERION NOW DECLINES.** They are fixed and unit-pinned, but **only finding 2 (helper inheritance)
is still exercised by a shipped grammar** — via `incomplete_class_scoped_type_sv_2023`, the one knot
SystemVerilog does absorb. Findings 1, 3 and 4 are held by
`a_profile_gate_survives_on_both_the_clone_and_the_suffix_route` alone until `.17` reopens the
cast/call knot. Recorded rather than left implicit: a fix whose only witness is a unit test is a fix
whose exposure is a unit test.

**THE MEASURED BLAST RADIUS** (`--report-indirect-lr-plan`, whose header now reports what the pass
DID: `indirect_eliminated_base_rules=/indirect_clone_rules=/indirect_refusals=`):

| grammar | cycle rows | absorbed at | clones | starvation-safe candidates |
|---|---|---|---|---|
| `ebnf` | **5 → 0** | `return_expression` | 1 | 0/0 after |
| `systemverilog` | **30 → 28** | `incomplete_class_scoped_type_sv_2023` | 0 | **0/28** — every candidate transitively starved |
| `systemverilog_lrm_profiled_wrapper` | 23 → 23 | — | 0 | 0 |
| every other shipped grammar | 0 → 0 | — | 0 | 0 |

⛔ **`starvation-safe candidates: 0/28` IS THE HEADLINE FOR SYSTEMVERILOG, not a footnote.** It says
the cast/call and property knots are not merely unplanned — they are **unplannable by chain
absorption at any base rule**, because every candidate is transitively held with a residual. Chain
absorption is the wrong instrument for them; that is `.15`'s and `.17`'s finding, now measured
rather than suspected.

⭐ **EVERY OTHER LINT COUNTER IS UNCHANGED — measured on ONE binary via the new
`--no-eliminate-indirect-left-recursion` A/B switch**, so the before and after are the same code
answering the same question.

⛔ **THE SVA PROPERTY KNOT IS `.15`'s, WITH A SHARPER MECHANISM THAN SLICE 4 HAD.** Its dominator
IS a candidate (`property_expr`, 80 routes, 12 clones) and it is **STARVED**, by
`prop_primary_sv_2017`/`_sv_2023` alt#13 `… implies property_expr`. So the knot is not merely "no
starvation-safe member" (slice 4's finding) — its *dominator* is starved too, by the same
hand-written cascade. That is a grammar-tier cause, exactly `.15`'s charter.

#### ⛔⛔ SLICE 5b (`PGEN-ENGINE-UNIVERSAL-SERVICES-0018`, 2026-08-13 session #224) — the criterion slice 5 landed to STOP a regression had no test, so a revert would have been silent

> **Slice 5's own fix was unprotected.** `rules_transparent_to` is the whole reason
> `initial k = 8'(1);` still parses, and reverting it to the pre-5b direct-holder scan left **every
> test green**. Measured, not assumed: with the one-line revert applied,
> `cargo test --lib indirect_lr` → **11 passed, 0 failed**. ZERO engine bytes; one test + one
> tracked synthetic.

⛔ **WHY THE EXISTING TESTS COULD NOT SEE IT — and it is the same blind spot as the regression.**
`knot_a()` (P1/P4) is the fixture behind every starvation test, and in it the only holder of the
transparent rule is `cast_expr`, which **dies with the rewrite** (`survives_rewrite=false`), so the
site is filtered before the verdict. Direct-scan and transitive-scan return the *same* answer on
that shape. SystemVerilog reaches `casting_type` from `cast` as well, so its holder outlives the
plan — the exact ingredient the synthetic lacked. ⇒ a criterion whose fixture cannot express the
distinction it encodes is untested by construction
([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

**THE FIX** — `p5_transparent_holder.ebnf` (the fifth synthetic, tracked) and
`a_transparent_holder_that_outlives_the_rewrite_starves_the_base_rule`. P5 is P4 plus one rule:

```text
scratch    := outer_cast | prim         ← outer_cast is reached from OUTSIDE the plan …
outer_cast := ct "'" "(" lit ")"        ← … and holds ct WITH a residual        ~ cast
ct         := kw | prim                 ← bare reference ⇒ TRANSPARENT to prim  ~ casting_type
```

⭐ **The test carries its own one-difference control**: the same assertion re-run on `knot_a()`
must report `prim` **safe**, because there the holder dies with the rewrite. If both halves agreed,
the criterion would be measuring nothing.

##### Acceptance Checklist (enforced) — slice 5b

- [x] **REPRODUCE / ISSUE** — the guard is missing, and that is measured rather than inferred:
  reverting `collect_starvation_sites` to `step.next_rule != base_rule` (the pre-5b scan) and
  running `cargo test --lib indirect_lr` gives **11 passed, 0 failed** — the criterion that stops
  SystemVerilog's `initial k = 8'(1);` regression can be deleted without a single red test.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: every starvation test builds on `knot_a()`
  (`indirect_lr_plan.rs`), whose only transparent-rule holder is `cast_expr`, an on-route rule the
  `rules_surviving_rewrite` fixpoint marks dead — so `collect_starvation_sites` filters the site out
  before either criterion can disagree. WHY: the fixture omits the one property that makes
  transitivity load-bearing, a holder that OUTLIVES the rewrite. P3's header names that property in
  prose (*"SystemVerilog reaches `casting_type` from `cast` too"*) and no fixture encoded it.
- [x] **FIX** — fix-hierarchy tier = **test + tracked fixture** (no engine, grammar, codegen or
  runtime change): `docs/tasks/artifacts/engine_universal_services/indirect_lr/p5_transparent_holder.ebnf`
  and `a_transparent_holder_that_outlives_the_rewrite_starves_the_base_rule`. Why no lower tier:
  the engine is already correct as of slice 5 — what was missing is the evidence that it stays so.
- [x] **ADDRESSED (verified)** — RED-proven in both directions on the same command:
  with the pre-5b scan restored, `cargo test --lib indirect_lr` → **FAILED, 11 passed / 1 failed**,
  the single failure being the new test (`prim is starved THROUGH ct, which is transparent to it`);
  with the criterion restored → **12 passed / 0 failed**. The control half asserts the opposite
  verdict on `knot_a()` in the same test, so the two shapes are separated, not merely both accepted.
- [x] **NO REGRESSION** — no engine, grammar or codegen byte changes, so `generated/` is untouched
  by construction (`git diff --stat` names nothing under `grammars/` or `generated/`); the other 11
  `indirect_lr` tests are unchanged and green; `bash scripts/check_doctrines.sh` → 18/18.
- [x] **LOCKSTEP** — this leaf, the `indirect_lr` artifact README (P5 added to the file table and to
  the blind-spot note), the promoted knowledge record
  `revert-your-fix-and-re-run-a-fix-no-test-defends-is-not-finished` (Knowledge Map 105 → **106**
  facts), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.

#### ⛔⛔ `.17` `in progress` — chain absorption cannot close SystemVerilog's cast/call knot, because PGEN's combinators COMMIT ONCE and never retry (opened 2026-08-13 session #224 by `.13` slice 5; slices 1-4 CLOSED `PGEN-ENGINE-UNIVERSAL-SERVICES-0020`…`-0023`, **slice 5 = implement**)

> ⛔⛔ **THE TITLE OF THIS LEAF WAS HALF WRONG UNTIL SLICE 4, AND THE MISSING HALF IS A SECOND
> DEFECT.** It read *"because PGEN's `*` is greedy and never retries at a lower iteration count"*,
> which is true and incomplete: slice 4 measured the CHOICE committing too (one winner kept, losers
> discarded — `ast_based_generator.rs:5037`), so an over-long SEED starves its holder exactly as an
> over-long LOOP does. A design that guards only the `*` closes half the knot and **regresses
> `initial k = int'(1);`**. Read slice 4's RESULT 2 before slice 1's FINDING 1, which it refutes.

**ROUTING EVIDENCE** — measured here, at the point it was found, rather than left for the receiving
leaf to rediscover:

- **The mechanism, isolated to three inputs on the shipped grammar.** `initial k = 8;` ACCEPT ·
  `parameter logic [7:0] K = 8'(1);` ACCEPT · `initial k = 8'(1);` **REJECT** (`furthest_position=42`)
  with `constant_primary` absorbed. The greedy `constant_primary` takes `8'(1)` entire — a legal
  `constant_cast` in its own right — and `cast := casting_type tick lparen expression rparen` then
  has no `tick` left. Correct-by-PEG, wrong-by-LRM.
- **It is not a route-walk gap and not fixable by choosing a different base.** With the transitive
  criterion, `starvation-safe candidates: 0/28` on SystemVerilog: EVERY candidate on the knot is
  transitively held with a residual by `cast` or `constant_cast`. There is no base rule to move to.
- **It reproduces outside SystemVerilog by construction**, because the cause is the engine's `*`
  (`generated/systemverilog_parser.rs`, `loop { if let Some(node) = try_parse(…) { … } else { break } }`
  — no re-entry at a lower count), not a SystemVerilog shape. `ebnf`'s knot is unaffected only
  because nothing holds `return_expression` transitively with a residual once the rewrite runs.
- **It is the same mechanism `.15` names** for the SVA property knot (a starved dominator), which is
  why the two should be designed together even though their proximate causes differ (`.15`'s is a
  hand-written precedence cascade; this one is the LRM's own `casting_type` factoring, byte-identical
  to Annex A per `constant_primary_lrm_alternative_audit.py` — so a grammar-tier repair is refused
  by [[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]]).
- ⛔ **Acceptance (b)'s decision record does NOT pre-authorise the obvious rescue.** Making the
  eliminated `*` backtrack is a per-parse cost on a knot measured at ~3× its fair share of rule
  entries, and *"costs are REJECTED, not traded"* is the second non-negotiable. Any design here has
  to price that explicitly.

**Acceptance:** (a) a design that closes `int'(2)'(3)`, `8'(1)` and `w()'(1)` **without** regressing
`initial k = 8'(1);`, priced against parse-time cost; ✅ **(b) the isolating synthetic — DONE by
slice 5b**: `p5_transparent_holder.ebnf` is P4 plus an outside holder of the transparent rule, the
one shape P1–P4 do not have; (c) the flips, including the two OpenTitan corpus rows.

⛔⛔ **(d) NEW (slice 7, 2026-08-14) — NO SLICE MAY SHIP A GUARD WHOSE HOP-CLONE HALF IS UNEXERCISED,
and today it is.** Every guard chain the SystemVerilog dry run synthesizes has `chain:` of length 1
(`max_hops=0` at both `casting_type` and `property_expr`), so `X_lr_guard{v}_<hop>` — the rule that
carries the entire call-site-scoping argument, and the one whose absence turns the repair into
slice 4's `g4`, which REJECTS `k = n;` — is reached by **no corpus run at all**. Its only coverage is
two unit tests on synthetics.

⛔ **This is a latent risk and NOT a hypothetical one**: the branching, multi-hop chains DO exist in
the shipped grammar (`cast` and `constant_cast`, `hops=3` over five cloned rules — slice 7 RESULT 5).
The driver simply never picks those candidates today. **Any change to the admission or to the
candidate ORDERING can make them the pick**, at which point an untested emitter path ships. ⇒ the
slice that flips either one must either exercise a multi-hop chain end-to-end, or measure and state
that its own picks are still all `max_hops=0` — silence is not an option, because the report already
tells you (`chain=` per site) and nothing else will.

⭐⭐ **DISCHARGED FOR SLICE 8, STILL OWED BY SLICE 9 (2026-08-14).** Slice 8's source grammar names
the holder at `ct` rather than at `prim`, so its chain is `ct > prim` (`max_hops=1`) and PGEN must
emit the hop clone. `guard_parses/probe.sh` row `B hop_clone` asserts `prim_lr_guard0_ct` is a
function **in the compiled parser**, and row `B e7` (`k = n;` ACCEPT) is the input that proves it is
isolating the two call sites rather than merely existing. ⇒ the rule is no longer covered by unit
tests on synthetics ALONE — it is exercised end to end. ⛔ What that does NOT do is exercise it on
SystemVerilog, which is why this obligation stays live for the admission/ordering flip.

⭐ **A DESIGN NOTE THIS LEAF INHERITS, so `.17` does not restart from zero.** The construct is
genuinely ambiguous under a greedy non-backtracking `*`, and that is provable from the LRM rather
than from the engine: `casting_type ::= … | constant_primary` means `int'(2)` is a legal *type* for
another cast (`int'(2)'(3)`), while in `8'(1)` the same rule must stop at `8` and leave `'(1)` to
the enclosing `cast`. **The same rule needs the chain in one context and the seed in the other**, so
no choice of base rule and no static shear can separate them — only re-entering the `*` at a lower
iteration count can. ⇒ the design space is (i) a re-enterable `*` for LR-eliminated rules only, paid
on the failure path rather than on every parse, or (ii) ANTLR4's answer — refuse the knot with a
named diagnostic, which is what the pass does today. ⛔ Prior art must be read before (i) is
attempted ([[feedback_read_prior_art_before_designing]]); "make it backtrack" is a decision about
PGEN's second non-negotiable, not a local edit.

⭐ **NAME THE FLAVOUR PRECISELY — greediness and re-enterability are INDEPENDENT axes, and confusing
them sends (i) after the wrong semantics.** In Perl 5's vocabulary:

| Perl | preference order | gives back on failure? |
|---|---|---|
| `a*` `a+` `a?` `a{n,m}` | **greedy** — longest first | ✅ yes |
| `a*?` `a+?` `a??` `a{n,m}?` | **lazy** — shortest first | ✅ yes |
| `a*+` `a++` `a?+` `a{n,m}+` | greedy | ❌ never (possessive) |

**PGEN emits `a*+` today** — verified in `generated/systemverilog_parser.rs`:
`loop { … if let Some(()) = attempt { … } else { break } }` takes the maximum count and never
revisits it. ⇒ **(i) is asking for `a*`, NOT for `a*?`.** Lazy is also re-enterable and is the wrong
default: on `int'(2)'(3)` the chain must take the FULL length — that is the LRM's reading and the
left-nested AST `lr_chain_fold` rebuilds — while `8'(1)` inside a `cast` must give back to zero.
Greedy-first-with-give-back gets both; lazy-first flips which parse wins whenever both counts lead
to an overall success, which is a semantic change, not a performance one.

⛔ **And possessive is not a codegen shortcut — it is the FORMALISM.** PEG defines `e*` as
`A ← e A / ε`, and PEG's ordered choice commits once an alternative succeeds. So (i) does not
"fix a loop"; it reintroduces exactly the backtracking PEG removed to buy its memoization
guarantee — which is why the memo half (a rule with several valid results at one position) is the
load-bearing risk, not the loop.

⛔ **DO NOT re-run `.13`'s route-walk work here.** Slice 5 measured the route machinery as sound and
the starvation as the blocker; the open question is the `*`, not the plan.

⛔⛔ **`generated/ebnf.rs` IS SEED-ONLY, SO THE `ebnf` RESULT IS A CLAIM ABOUT A FRESH CLONE, NOT
ABOUT THIS WORKING COPY — and that is a defect class in its own right (→ `.16` NEW).**
`regenerate_generated_parsers` rebuilds the annotation pair plus the **7** `GENERATED_PARSER_FAMILIES`
(`rust/Makefile:932`); `generated/ebnf.rs` is in none of them. It is seeded once
(`rust/Makefile:850`, `if [ ! -f … ]`) and then never regenerated. Measured here: the local artifact
is dated **2026-07-30** while the grammar and codegen have moved since, and generating it fresh from
the tracked `generated/ebnf.json` with today's binary produces the 6 new LR rules
(`return_expression_lr_base`, `_lr_seed_expression_return`, `_lr_suffix`, `_lr_suffix_r0..r2`) and
otherwise the identical rule-function set. ⇒ **the `ebnf` 5 → 0 is REAL in the grammar and only
reaches an artifact on a clone that has to seed one.**

⛔ **What is measured about the ebnf victims, and what is not.** EBNF-1/EBNF-2 (`-> $1 + $2`,
`-> $1 ? $2 : $3`) accept on the INTERPRETER both before and after
(`--interpret-parse` × `--no-eliminate-indirect-left-recursion`), which is not a flip and not
evidence: slice 1's own table says these fail on the **meta-parser** (`generated/ebnf.rs`, arm 2),
and the interpreter has no cycle guard at all — the `.14` divergence, exactly. So the ebnf flip is
claimed **only** once the re-seeded artifact is built and `ebnf_dual_run` re-run; until then this
leaf claims the grammar-level 5 → 0 and nothing about arm 2.

##### ✅ `.17` SLICE 1 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0020`, 2026-08-13 session #225) — the PRIOR ART this leaf's design owes, and the three measurements that shrink it

> **DOCS + a tracked probe bank only — ZERO grammar bytes, ZERO Rust bytes, ZERO codegen bytes.**
> The leaf's own gate says *"⛔ Prior art must be read before (i) is attempted"*. This slice reads
> it, **re-measures every engine claim the design note makes** rather than quoting the note, and
> hands slice 2 a smaller, better-posed decision. ⛔ The design decision is deliberately NOT taken
> here — what changed is which option is the expensive one.
>
> Probe bank (re-runnable, self-checking, 7 cases):
> `docs/tasks/artifacts/engine_universal_services/quantifier_policy/probe.sh` →
> `QUANTIFIER-POLICY-CONTROLS: 7/7 as declared`.

###### PRIOR ART

Searched in the four sources [[feedback_read_prior_art_before_designing]] ranks by authority, plus
the external literature the second non-negotiable demands before any parse-time cost is bought.

**1. `grammars/ebnf.ebnf` — is it already expressible?** No. The meta-grammar declares
`simple_quantifier` (`?`/`*`/`+`, `:184`), `bounded_quantifier` (`{N}`/`{N,M}`/`{N,}`/`{,M}`,
`:191`) and `probability_quantifier` (`:214`) — **and no flavour axis at all**: no `*?`, no `*+`, no
per-site policy. The contrast is the finding: the *choice* combinator **does** have a declared
policy surface (`@branch_policy: longest_match | ordered`,
`rust/src/ast_pipeline/semantic_directive_registry.rs:59-71`), and the 46-name registry has no
quantifier sibling. ⇒ the quantifier is the one combinator PGEN never gave a policy, in the
meta-grammar as well as in the engine.

**2. `docs/decisions/` — already decided or designed?** Three hits, one of them decisive.

- ⭐⭐ [[feedback_layer_0_unified_quantifier]] (2026-05-21) — Layer 0 folded `?`/`*`/`+`/bounded into
  ONE parameterised codegen loop with a two-layer atomicity model. Its closing section is a direct
  hit on `.17`: *"**What Layer 0 does NOT fix**: cross-rule backtracking (e.g. `hierarchical_identifier`'s
  first-set-overlap case `(identifier constant_bit_select dot)* identifier` — PEG can't try the
  trailing `identifier` at each iter before committing the prefix). That's a separate engine class"*
  — and it records `.b.2`'s surgical patch as deliberately re-appliable. ⇒ **`.17`'s class was
  identified, named and consciously deferred in May; it is not a new discovery, and the design must
  not re-derive it.**
- [[project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime]] — this leaf's
  parent record. Its 2026-08-13 amendment already routes the blocker here and states plainly that it
  does **not** reopen seed growing, *"the blocker is a quantifier's backtracking policy, not the
  elimination mechanism"*. Binding on slice 2.
- [[project_earlier_always_matches_unsound_backtracking]] (2026-07-05) — retired a linter check whose
  premise was *"PEG commits to the earlier alternative"*, because **PGEN's engine backtracks**. That
  is the same law Q2/Q3 below re-measure.

**3. `docs/tasks/` — does a tree already own it?** Four hits; the first is prior art for the *fix
shape* and for the *ruling on who may own it*.

- ⭐⭐ `SV-EXH-PROOF.3.3.4.b.2` (`completed-as-diagnosis-only`) — a surgical
  `!callable_method_call_body` **negative-lookahead stop-guard** on `hierarchical_identifier`
  (`grammars/systemverilog.ebnf:2158`) was diagnosed AND applied, and *"would have worked for the
  first-set-overlap case at uvm_pkg byte 159297"*. It was **reverted on a director decision**:
  *"revert and pivot to general Layer-0 engine fix rather than patch case-by-case"*, clean state
  tagged `checkpoint/post-3-3-4-b-1-clean-pre-layer-0` @ `f758b878`. ⇒ two things are prior art
  here: the guard shape, and the ruling that it must be an **engine service**, not a hand-written
  per-rule patch — the identical ruling
  [[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]] makes.
- `LANG-CAPABILITY-AUDIT.3b` FINDING 3 — measured the possessive law head-on (`r10-bt-ab`,
  `r10-bt-aaab` both REJECT, trace-confirmed, located at `ast_based_generator.rs:5692`) and recorded
  the author-facing consequence — *"write `(!close body)* close`"* — with a working measurement
  (`r10_static := "q" "/" ( !"/" char )* "/"` ACCEPTs `q/abc/` and `q//`). ⇒ the stop-guard is
  already known to work in PGEN; Q4 below reproduces it on `.17`'s own shape.
- `GRAMMAR-WELLFORMED` (`:1036`, `:1275`) — `macro_default_value := macro_default_atom+` is a
  **third** victim of the same law, in a third family. The class is recurrent, not SV-specific.
- `PARSE-SOTA-research-synthesis` §.1/§.2 — the repo's own literature survey. It already names the
  possessive law as Ford POPL 2004, refuses GLL/GLR/Earley/Marpa outright, refuses runtime LR, and
  lists **parametric rules (rust-peg)** as `B3` and **cut (Mizushima 2010)** as `B2`.
- `.15` (this tree) — the SVA property knot: same starvation mechanism, different proximate cause.

**4. `docs/book/` — a designed-but-unbuilt form?** One near-miss, and one claim that is NOT stale.

- `docs/book/src/lexical-annotations.md` documents `[> LIST ]` / `[>! LIST ]` explicitly as **SDF2
  *follow restrictions***. That is exactly the right formalism for "stop a maximal match short" —
  but at the **lexical** tier (character classes after a token), and its own text says the `require`
  arm is *"carried and available, but a documented generation no-op today"*, consumed by the stimuli
  generator rather than by the parser. ⇒ right idea, wrong tier, not parser-consumed. A *structural*
  follow restriction on a quantifier is not expressible today.
- ⚠️ `docs/book/src/developer-architecture.md:17-25` is **current and correct** — re-read this
  session, not quoted from `LANG-CAPABILITY-AUDIT.3b`'s finding, which is superseded. It already
  warns *"Do not read `|` as 'first alternative wins'"* and states the `longest_match` default.
  `LANG-CAPABILITY-AUDIT.5` repaired it and is `done`. No book defect to route.

**External literature.**

- **Ford, POPL 2004** — PEG defines `e*` as `A ← e A / ε`, and ordered choice commits once an
  alternative succeeds ⇒ possessive **by construction**. The design note's formalism claim is
  correct and stands unchanged.
- **Perl 5 / PCRE** — the three flavours (greedy `a*`, lazy `a*?`, possessive `a*+`). PGEN emits
  `a*+`; the note's `.19` correction that (i) wants `a*` and not `a*?` is confirmed and stands.
- **ANTLR4 ALL(\*)** (Parr, Harwell & Fisher, OOPSLA 2014) — the mainstream alternative to
  backtracking a loop is **prediction**: simulate the ATN in full context with a DFA cache and
  decide loop exit before entering it. ⛔ `PARSE-SOTA` already refuses *dynamic ALL(\*) ambiguity
  detection*; the **loop-exit** use is a distinct question and slice 2 should price it as such
  rather than inherit that refusal.
- **SDF2 / Rascal follow restrictions (`-/-`)** — the declarative, generation-time formalism for
  the same problem. PGEN already borrows it lexically (above); the structural form is the gap.
- **Mizushima, PASTE 2010 (cut)** — commits *harder*; the wrong direction here, and already tracked
  as `PARSE-SOTA` `B2`.
- **Warth PEPM 2008 / Medeiros SCP 2014** — runtime LR. Explicitly **not** reopened, per the parent
  decision record.

**What the search did NOT find.** No surface — meta-grammar, directive registry, book or tree —
expresses a per-site quantifier give-back policy, and no tree owns one. So slice 2's proposal is
genuinely new. It is also far smaller than the note assumed, because both mechanisms it would need
already ship: a codegen-computed **per-iteration guard slot** in the emitted loop, and **sheared
clones** from the `.13` eliminator.

###### ⛔⛔ FINDING 1 — ~~the same engine gives back at a CHOICE and refuses to at a QUANTIFIER~~ **REFUTED by slice 4 — DO NOT CITE**

> ⛔⛔ **REFUTED 2026-08-13 by slice 4 (below). The choice does not give back either.** Q2's ACCEPT
> is fully explained by FINDING 2's own `longest_match` default — `"ab"` wins outright and `"c"`
> matches the one remaining byte — so the case is predicted identically with and without a
> give-back and **discriminates nothing**. The shape that separates them (`ch := "a" | "ab"` with
> `scratch := ch "bc"`) REJECTS on both oracles; the tournament keeps a single `best_content` slot
> (`ast_based_generator.rs:5037`) and discards every loser.
> ⇒ there is **no asymmetry between the combinators** — both commit — and the starvation has TWO
> independent sources, not one. See [[project_pgen_gives_back_at_neither_combinator]]. The text
> below is retained verbatim for the audit trail.

The one-difference pair, measured (`probe.sh` Q1/Q2):

| | grammar | input | verdict |
|---|---|---|---|
| **Q1** | `( "a" )* "a"` | `aaa` | **REJECT** — the star takes all three, the trailing `"a"` starves |
| **Q2** | `( "a" \| "ab" ) "c"` | `abc` | **ACCEPT** — `"a"` wins, `"c"` fails, the choice gives back and retries `"ab"` |

Both are *"a sub-match succeeds, then the element after it starves"*. The only variable is which
combinator produced the sub-match, and the engine answers differently. ⇒ **`.17`'s blocker is an
asymmetry between two of PGEN's own combinators — not a property of PEG.**

###### ⭐⭐ FINDING 2 — a multi-attempt protocol is already the DEFAULT here, confined by STATIC ELISION

Q3: `( "a" | "ab" )` on `ab` **ACCEPTs**, i.e. the longest alternative wins, not the first. Located:
`ast_based_generator.rs:4273` — *"Multi-branch - evaluate all branches and keep the longest
successful match"* — with selection ordered priority → longest → associativity (`:4387`), and the
tournament **elided** where codegen can prove it unnecessary (`degenerate_dispatch_byte_sets`,
`:4556`).

⇒ the second non-negotiable has never been read here as *"never attempt twice"*. It has been
satisfied by **proving the extra attempts away at codegen time**. That reframes `.17`'s pricing
question from *"may the engine attempt more than once?"* (already answered: yes, everywhere) to
*"can the extra attempts be confined, statically, to the sites that provably need them?"* — a
question this codebase has answered once already, for the combinator next door.

###### ⭐ FINDING 3 — a PEG-native stop-guard closes the starvation, and its price is a CLONE, not backtracking

| | grammar | input | verdict |
|---|---|---|---|
| **Q4a** | `( "a" &"a" )* "a"` | `aaa` | **ACCEPT** — the guard refuses the fatal 3rd iteration |
| **Q4b** | same | `a` | **ACCEPT** — and does not break the zero-iteration case |
| **Q5** | `star_rule := ( "a" &"a" )*`, holder with **no** residual | `aaa` | **REJECT** |
| **Q5b** | Q5 with the guard removed (one-difference control) | `aaa` | **ACCEPT** |

Q4 closes Q1's starvation with **zero engine change, zero re-entry and zero memo exposure** — the
loop still never revisits its count; it simply never takes the iteration that starves the holder.
Q5/Q5b price it exactly: the guard is **context-dependent**, so it cannot be written onto the rule
(`constant_primary` must reserve the trailing `' ( … )` when reached from `cast` and must **not**
when reached from an ordinary expression). It has to be written onto the **call site** — which for
this pass means the sheared clone `.13` already emits.

⭐ **The emitted loop already has the slot.** `#quant_guard_tokens`
(`ast_based_generator.rs:6006`, the `RGX-0078.5.i.7` Q-GUARD) breaks the loop on a static byte-set
test at every iteration boundary, with furthest-position parity proven; `@stop_at_rule_boundary`
(`:5915`) is a second, directive-driven break in the same position. ⛔ **That precedent is
architectural, not a licence**: the Q-GUARD elides an attempt that would have **failed anyway**
(sound by construction), while a follow-restriction guard refuses an attempt that would have
**succeeded** — it changes the accepted language and carries a soundness burden the Q-GUARD never
had.

###### ⛔ WHAT THIS LEAF'S OWN DESIGN NOTE GOT WRONG, precisely

1. *"it reintroduces exactly the backtracking PEG removed to buy its memoization guarantee"* —
   **half wrong, and the wrong half is load-bearing.** PGEN never removed it: Q2/Q3 show a
   give-back longest-match tournament running at every non-degenerate choice. What is missing is
   give-back at **one** combinator. The formalism half (PEG's `e*` is possessive) is correct.
2. *"no choice of base rule and no static shear can separate them — only re-entering the `*` at a
   lower iteration count can"* — **refuted by Q4/Q5.** A per-iteration lookahead separates the two
   contexts with no re-entry at all. What Q5 supplies is the constraint the note was reaching for:
   a shear of the **rule** cannot separate them; a shear of the **call site** can.
3. *"the memo half … is the load-bearing risk, not the loop"* — **correct, and it is now what makes
   (iii) the cheap option**: a stop-guard leaves every rule with exactly one result per position, so
   the memo is untouched. A re-enterable `*` does not.

###### THE DESIGN SPACE AS IT NOW STANDS (slice 2 decides; this slice does not) — ⭐ **SETTLED BY SLICE 4: (iii), structural, TWO positions**

> ⭐ **Outcome, recorded here so this list is not read as still-open:** slice 4 ADOPTS **(iii)** in
> its structural form with a guard in **two** positions on one sheared clone, REFUSES **(i)** (now
> the first give-back in the engine, not parity), leaves **(ii)** as the fallback it always was, and
> does not reach **(iv)**. The byte-set form named below as *"the candidate to price first"* is
> **dead** — slice 2 measured exactness at 0/157 and slice 4 measured it failing on the first
> comment.

- **(iii) call-site-scoped follow-restriction guard**, synthesized by the eliminator onto the sheared
  clone — **the candidate to price first.** Zero parse-time cost beyond one guard test per iteration,
  zero memo exposure, generation-time by construction, and it reuses two shipped mechanisms.
  ⛔ Its open question must be answered with `--report-indirect-lr-plan` on the **shipped** grammar,
  not on a synthetic: is the holder's residual FIRST set statically computable at each of the 28
  rows, and does the guard stay sound when that residual is nullable or when two holders of the same
  clone disagree?
- **(i) re-enterable `*`, per-site and declared** — not refuted, but now the **expensive** option:
  it buys memo soundness work that (iii) does not need. Kept alive because (iii) may not generalise.
- **(ii) refuse the knot with a named diagnostic** — ANTLR4's answer and the status quo; stays the
  honest fallback.
- **(iv) predict the loop exit (ALL(\*)-style)** — newly listed. Distinct from `PARSE-SOTA`'s refusal
  of dynamic *ambiguity detection*; priced separately or explicitly declined.

⭐ **And one framing the note could not have had:** `@branch_policy` is a live **per-rule policy
surface for a combinator**, so a quantifier-policy directive would be the *second* of its kind, not
the first. Whether the policy should be author-declared at all — versus derived by the eliminator and
never surfaced — is itself a slice-2 decision, and
[[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]] argues for derived.

###### Acceptance Checklist (enforced) — `.17` slice 1

- [x] **REPRODUCE / ISSUE** — this leaf's design note asserted two engine properties as premises for
  choosing between (i) and (ii), neither re-measured:
  `INTERPRET-PARSE: grammar='q1_possessive_star' … accepted=false … error="Backtrack { position: 3 }"`
  is the starvation; `INTERPRET-PARSE: grammar='q2_choice_gives_back' … accepted=true` is the same
  shape surviving at a choice.
- [x] **ROOT CAUSE (WHY + WHERE)** — the give-back asymmetry is emitted, and both sides are located:
  the possessive loop at `rust/src/ast_pipeline/ast_based_generator.rs:6041-6091`
  (`if let Some(node) = parser.try_parse(…) { … } else { break }`, min-count enforced afterwards, no
  emission path re-tries the loop shorter) versus the tournament at `:4273` (*"evaluate all branches
  and keep the longest successful match"*) with static elision at `:4556`. The *why* the note gave —
  *"PEG removed backtracking"* — is contradicted by `:4273` in the same file.
- [x] **FIX** — none applied; this is the prior-art/design slice the leaf's own gate requires before
  (i) may be attempted. Fix-hierarchy note for slice 2: the surviving front-runner (iii) is
  **declarative + generation-time**, i.e. the *highest* tier, above the engine change (i) the note
  assumed was mandatory.
- [x] **ADDRESSED (verified)** — `bash docs/tasks/artifacts/engine_universal_services/quantifier_policy/probe.sh`
  → `QUANTIFIER-POLICY-CONTROLS: 7/7 as declared`, rc 0. Ground truth in **both** directions: the
  bank carries must-accept and must-reject cases, and a deliberately flipped expectation makes it
  exit rc 1 naming the case (`Q5 … => REJECT (want ACCEPT) ⛔`), so it can notice its own breakage.
  ⭐ The three load-bearing cases were re-run on the **real generated parser** (scratch slot,
  authoritative BY CONSTRUCTION) and agree with the interpreter arm exactly:

  ```text
  # Q1  ( "a" )* "a"        on "aaa"
  Error: parse_full rejected sample for grammar 'scratch' on 'rust/target/es17/aaa.txt':
    Backtrack at position 3 [furthest_position=0, +0 bytes deeper than surface position]   (rc 1)
  # Q4a ( "a" &"a" )* "a"   on "aaa"
  parse_full passed for grammar 'scratch' on 'rust/target/es17/aaa.txt'                    (rc 0)
  # Q4b ( "a" &"a" )* "a"   on "a"
  parse_full passed for grammar 'scratch' on 'rust/target/es17/a.txt'                      (rc 0)
  ```

  ⇒ the give-back asymmetry and its stop-guard repair are properties of the **shipped codegen +
  runtime**, not of the interpreter. (`Backtrack at position 3` is the same failure the interpreter
  reports as `Backtrack { position: 3 }`.) Both probe rounds ran under the memory guard
  (`peak_tree_rss=10650MB elapsed=1208s`, exit 0) — an unguarded release build of this crate dies at
  ~10 min with `signal: 15` and no rustc error.
- [x] **NO REGRESSION** — nothing shipped was touched: ZERO bytes under `grammars/`, ZERO under
  `rust/src/`, ZERO codegen, ZERO generated artifacts staged. The probe bank drives standalone
  `.ebnf` files through `--interpret-parse` and acquires no restore obligation of its own.
  ⭐ **The scratch-slot restore is VERIFIED, not asserted** — `.13` slice 4b's whole finding is that
  a clean `git status` is not evidence here, because `generated/scratch_parser.rs` is git-ignored and
  a half-restore is invisible. So the restore ran in both halves (`git checkout` **then**
  `make focus_scratch`, regenerating the artifact FROM the restored fixture) and the two gates that
  read the slot as a matched pair were re-run:
  `parse_harness_equivalence::gate::certified_grammars_are_byte_identical` → `test result: ok.
  1 passed; 0 failed` (23.16s) and `parser_registry::tests::scratch_slot_…` → `1 passed; 0 failed`.
  ⛔ Trap worth carrying: the first attempt at this measurement piped `cargo test` into `tail`, and
  the memory guard duly reported `exit=0` for a run whose cargo invocation had **failed with
  `unexpected argument`** — the pipe swallowed the status. A green number from a masked exit code is
  the same failure shape as `CI-PARITY-GATE-ROT.24`'s two empty result sets diffing clean.
- [x] **LOCKSTEP** — no user-visible behaviour changed, so no book/contract/schema edit is owed. The
  one book surface this slice checked (`developer-architecture.md:17-25`) was re-read and is already
  correct; `MEMORY.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `docs/TASK_TREE.md` updated.

##### ✅ `.17` SLICE 2 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0021`, 2026-08-13 session #226) — option (iii) PRICED on the shipped grammar: 16/28, and the cheapest row in the census belongs to `.15`

> **One analysis module + its report columns + a tracked probe bank. ZERO grammar bytes, ZERO
> codegen bytes, ZERO generated-parser bytes, ZERO change to any shipped verdict.**
> The leaf's own gate for this slice: *"Its open question must be answered with
> `--report-indirect-lr-plan` on the **shipped** grammar, not on a synthetic."* This slice answers
> all three halves of that question with measured numbers and hands slice 3 a decision with a price
> on it. ⛔ The design decision is still deliberately NOT taken — what changed is that (iii) now has
> a cost, a coverage figure and two named limits instead of a plausibility argument.
>
> Census bank (re-runnable, self-checking, 8 cases):
> `docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh` →
> `GUARD-FEASIBILITY-CENSUS: 8/8 as declared`.

###### THE CRITERION, DERIVED — and why the verdict needs FIVE outcomes

After the `.13` rewrite the base reads `X := X_lr_base ( X_lr_suffix )*` with a **possessive** `*`.
A holder `H := X residual` starves when the loop eats text `residual` needed. Option (iii) is slice
1's measured Q4 repair, scoped to the call site:

```text
X_guarded := X_lr_base ( X_lr_suffix &FIRST(residual) )*
```

⭐ **The completeness condition is `FIRST(suffix) ⊆ FIRST(residual)`, and it is decidable.** At any
position where the loop continues, the suffix matched there, so that position starts with
`FIRST(suffix)`; containment then makes the guard PASS there, so the only iteration the guard can
refuse is the last one — the fatal one. Without containment the guard cuts the loop short at an
intermediate iteration and trades one under-acceptance for another (`( "a" &"b" )* "b"` on `aab`).
And it can never over-accept in either case: `L(X_guarded) ⊆ L(X)`, so the holder still accepts only
strings in `L(X)·L(residual)` — the guard **recovers derivations the declarative grammar already
licensed**, it never invents one. Both tests run on the shared FIRST over-approximation
(`ast_pipeline/first_set.rs`) in the sound direction: disjointness of over-approximated sets implies
disjointness of the true sets, and the containment test compares against exactly the byte set the
emitted guard would evaluate.

| verdict | condition | consequence for (iii) |
|---|---|---|
| `guardable` | competing, `FIRST(suffix) ⊆ FIRST(residual)` | the guard closes the site |
| `no_competition` | `FIRST(suffix) ∩ FIRST(residual) = ∅` | **no guard owed** — the loop can never take this holder's text |
| `residual_nullable` | `residual` can match empty | **no guard owed** — the holder cannot starve |
| `guard_incomplete` | competing, `FIRST(suffix) ⊄ FIRST(residual)` | BLOCKS the candidate |
| `undecidable` | a FIRST set is `unresolved`, or the suffix is nullable | BLOCKS the candidate |

###### THE THREE ANSWERS THE LEAF ASKED FOR

**(1) "is the holder's residual FIRST set statically computable at each of the 28 rows?" — YES,
`undecidable=0`** at all **185** surviving starvation sites (126 SystemVerilog + 59 wrapper). Not
one site needs a conservative decline.

**(2) "does the guard stay sound when that residual is nullable?" — the question does not arise,
and the reason is a defect in the SHIPPED criterion.** A nullable residual means the holder succeeds
on the empty match and **cannot starve**, so no guard is owed: **29 of 126** SV sites and 19 of 59
wrapper sites are structural false alarms. ⚠️ It is also a hole in the transitive walk, recorded
rather than closed here: `rules_transparent_to` follows only **syntactically** empty residuals, so a
nullable-but-non-empty holder stops the chain and a hazard one hop further out is never searched
from there.

**(3) "or when two holders of the same clone disagree?" — at the rules that matter, they agree.**
**17 of 28** SV candidates need exactly ONE guard variant. Every multi-variant row (2–6) is a
non-dominator member of one of the two knots and is `BLOCKED` for an independent reason, so the
multiplier is never paid.

###### ⭐⭐ THE HEADLINE, AND THE ROW THAT CHANGES `.15`'s PREMISE

| grammar | starvation-safe (shipped criterion) | **guard-feasible (option (iii))** |
|---|---|---|
| `systemverilog` | **0 / 28** | **16 / 28** |
| `systemverilog_lrm_profiled_wrapper` | 3 / 18 | **13 / 18** |

Both remaining SystemVerilog knots are guard-feasible **at their dominators**, each with one
variant:

```text
[candidate] constant_primary   guard: FEASIBLE  suffix_first={'/}  variants=1 {'/}  max_hops=1
[candidate] property_expr      guard: FEASIBLE  suffix_first={-/}  variants=1 {-/}  max_hops=0
```

⇒ the cast/call knot costs **one** guarded clone chain of depth 1 (through `casting_type`) on top of
the 14 clones the plan already emits, and the SVA property knot costs **zero extra clones**.

⛔⛔ **ROUTED OUT — the SVA row contradicts `.15`'s founding premise.** `.15` was opened on slice 4's
finding that the property knot has NO starvation-safe base rule and concluded it needs `.3`'s
precedence-declaration service. The census says the *starvation* blocker is closable at the
dominator with one variant and zero hops — the cheapest row in the whole census. ⛔ That is a claim
about the starvation blocker ONLY: whether the annotation-composability check also passes at
`property_expr` is **not** measured here (the wrapper refuses it for a missing return annotation on
`property_expr_sv_2017` alternative 5) and is slice 3's first check. Recorded in `.15` below.

###### ⛔⛔ THE QUALIFIER ON EVERY NUMBER ABOVE — FEASIBLE IS NOT CLOSED, AND IT IS MEASURED AT 157/157 ⛔ (`157` → **129**: the denominator was 129 sites + 28 candidate rows — `.17` slice 5 RESULT 3; the *claim* is unchanged.)

`guard-feasible 16/28` means *a guard is expressible and provably **sound** at 16 candidates*. It
does **not** mean 16 knots close, and the report now says so in the output rather than in prose: `~`
marks an **over-approximated** byte set, i.e. one where the guard passes at positions the residual
cannot actually start from and therefore silently declines to refuse the fatal iteration.

```text
guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=1
```

**Measured: 157 of 157 sites are over-approximated — not one exact guard exists on either grammar** ⛔ (`157` → **129**: the denominator was 129 sites + 28 candidate rows — `.17` slice 5 RESULT 3; the *claim* is unchanged.)
(probe case C7, pinned absolutely against the UNCAPPED report). Two compounding causes:

1. **Structural** — `FirstSetSummary::byte_decided` holds only for single-byte-decided shapes, so any
   multi-element residual (`tick lparen expression rparen`) is approximate by construction.
2. **Layout** — `trivia := (line_comment | block_comment)*` (`grammars/systemverilog.ebnf:619`) is
   nullable and leads every token, so **`/` is in the FIRST set of every token**; concretely,
   `int'(2)/*c*/'(3)` slips a byte-test guard.

⇒ ⛔ **This FLIPS slice 3's starting point, and the first draft of this leaf had it wrong.** Slice 1
called the byte-test guard *"the candidate to price first"* and this slice's own first draft filed
the comment case as an edge-case caveat. With exactness measured at **0 of 157**, the cheap form is
not a proof of closure anywhere on SystemVerilog, and a **trivia-aware structural lookahead**
(`&( residual )` — exact, at the price of a per-iteration sub-parse instead of one byte compare) is
effectively mandatory rather than a refinement. Slice 3 still prices the trade; it may no longer
assume the cheap form suffices.

⭐ The same layout fact is why **`no_competition=0`**: with `/` in every set, no two token-led FIRST
sets are ever disjoint, so the disjointness refinement is sound, implemented, and structurally unable
to fire here. Pinned at 0 in both directions (probe case C5) so a future layout-model change is
noticed rather than assumed. ⭐ It is also why the first reading of the census looked contaminated —
every FIRST set containing `/` is the grammar's own layout model, and it was root-caused rather than
classified.

###### ⛔⛔ TWO THINGS THIS SLICE DELIBERATELY DID NOT DO — both were defects in its own first draft

**A. It does not let a report edit reach the parser, and that is now true BY CONSTRUCTION.**
`render_elements` carried the doc comment *"Report-only — nothing parses this back"* and that comment
is **false**: `indirect_lr_elimination.rs:483` decides the ambiguity refusal *"two routes iterate the
identical suffix under the identical profile gate but declare different ASTs"* by comparing two of
its strings, and `indirect_lr_elimination.rs:209` calls the survey itself. The first draft of this
slice added the group parentheses to that shared renderer — which makes the comparison strictly
finer, i.e. **refuse less, absorb more**, i.e. a shipped-parser change riding on a cosmetic fix,
justified only by a before/after measurement on three grammars. ⛔ Corrected: `render_elements` is
**frozen byte-for-byte** and the human-facing `render_elements_display` is a separate entry point on
a shared, flag-parameterised walker (zero duplication). A unit test pins the collision in BOTH
directions — `( "a" "b" )?` and `"a" "b"?` must still render IDENTICALLY under the frozen renderer
and DIFFERENTLY under the display one. The coupling itself is routed as `.18`.

**B. It does not judge a candidate on partial evidence.** Every test here reads `suffix_first` as an
OVER-approximation. A union over a **truncated** route set is an UNDER-approximation, so
`FIRST(suffix) ⊆ FIRST(residual)` would pass too easily and report `guardable` — and a `FEASIBLE`
candidate — on evidence never gathered, with nothing downstream able to notice. ⛔ The first draft
had exactly that hole. Corrected: `routes_truncated` poisons `suffix_first` to `unresolved`, forcing
every site to `undecidable` and the candidate to BLOCKED. It costs nothing today (the widest knot
enumerates **80** routes against a budget of **128**) and is exercised through a new
budget-parameterised entry point `survey_indirect_left_recursion_with_route_budget`, because an
unreachable branch that must fail SAFE is precisely the branch a release trusts and never runs
([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

###### ⛔⛔ `.18` NEW `todo` — a DISPLAY function decides a semantic refusal in the eliminator, so making the report more legible changes what the pass absorbs (opened 2026-08-13 session #226 by `.17` slice 2)

**ROUTING EVIDENCE** — measured here, at the point it was found. ⭐ It stays in THIS tree because it
is an engine/analysis boundary defect, not a language one; nothing is sent to another family.

- **The coupling, located.** `indirect_lr_elimination.rs:483-486` decides the ambiguity refusal
  *"two routes iterate the identical suffix under the identical profile gate but declare different
  ASTs"* by comparing **rendered strings**:
  `render_elements(&left.elements) == render_elements(&right.elements)`. `render_elements`
  (`indirect_lr_plan.rs`) is documented *"Render nodes as compact EBNF-ish text. Report-only —
  nothing parses this back"*, and that comment is now measurably false.
- **It reproduces as a defect in BOTH directions, independent of this slice.** The pre-slice-2
  renderer dropped a quantified group's parentheses, so `( a b )?` and `a b?` — structurally
  different suffixes — rendered IDENTICALLY and compared EQUAL. A plan with those two routes would
  have been **refused for an ambiguity it does not have**. The renderer is also lossy in the other
  direction for any shape it abbreviates, so a real collision could in principle be missed.
- **How it was found: slice 2's own first draft walked into it.** That draft fixed the missing
  parentheses in the SHARED renderer, which makes the ambiguity comparison strictly more faithful —
  **refuse less**, i.e. potentially absorb MORE. A parser-behaviour change riding on a report edit,
  and exactly the shape a "docs-only" framing hides. It was caught by asking what else calls the
  function, not by a gate.
- **The slice was then RESTRUCTURED so the hazard cannot be reached from it:** `render_elements` is
  frozen byte-for-byte and `render_elements_display` carries the parentheses, with a unit test
  pinning that `( "a" "b" )?` and `"a" "b"?` still render IDENTICALLY under the frozen renderer. So
  this leaf owns a LATENT coupling, not a live regression, and slice 2's no-regression claim rests on
  construction rather than on a three-grammar sample.
- ⚠️ **The collision is real and unfixed today.** Under the frozen renderer those two structurally
  different suffixes compare EQUAL, so a plan carrying both would be refused for an ambiguity it does
  not have. No shipped grammar exhibits the pair (the before/after capture shows every
  `✅ absorbed at` and `⛔ REFUSED` line identical on `systemverilog`,
  `systemverilog_lrm_profiled_wrapper` and `ebnf`), which is why this is PARKED rather than urgent.

**Acceptance:** (a) the ambiguity check compares STRUCTURE, not a rendering — a dedicated key over
the gen-AST (or a derived canonical form) with `render_elements` reserved for humans; (b) a test
that is RED against the string comparison, i.e. two routes whose rendered forms collide but whose
structures differ; (c) `render_elements`' doc comment corrected, or the function split, so the next
reader cannot re-acquire the same assumption. ⛔ PARKED behind the SV lane lock — it blocks no SV
release work, and slice 2 measured its live effect at zero.

###### Acceptance Checklist (enforced) — `.17` slice 2

- [x] **REPRODUCE / ISSUE** — the leaf's own open question was unanswerable from the shipped report:
  `--report-indirect-lr-plan` printed each starvation site's residual as TEXT and nothing about
  whether a guard on it is expressible, complete, or shareable. Reproduced as the absence it is —
  HEAD's report for `constant_primary` carries the site and no guard column:
  `⛔ starved by cast alt#0 (on-route, still reachable after the rewrite) — residual 'tick lparen expression rparen' a greedy suffix could steal`
- [x] **ROOT CAUSE (WHY + WHERE)** — the survey's starvation criterion
  (`indirect_lr_plan.rs::collect_starvation_sites`) is purely **structural**: it fires on any
  non-empty residual and consults no FIRST set, so it cannot separate a site that would really
  starve from one that merely looks like it, and it says nothing about a guard. Measured
  consequence, both directions: **29 of 126** SV sites are `residual_nullable` — holders that
  cannot starve at all — while `starvation-safe candidates: 0/28` reports the population as
  uniformly hopeless. The FIRST machinery that decides all of it already ships
  (`ast_pipeline/first_set.rs::branch_first_set`) and this pass simply never called it.
- [x] **FIX** — fix-hierarchy tier = **new tooling / instrument** (no engine, grammar, codegen or
  generated-artifact byte moves; the shipped `is_starvation_safe` verdict is deliberately
  UNCHANGED). `GuardVerdict` / `GuardAssessment` / `GuardFirstBytes` in `indirect_lr_plan.rs`, the
  new report columns and JSON fields, and `rules_transparent_to` refactored from a set to a DEPTH map
  so the guard clone-chain length is a measured number instead of a guess.
  ⛔ The refinement is REPORTED, not APPLIED: making `is_starvation_safe` consult it would change
  which knots the eliminator absorbs — a real parser change, owed a two-sided repro ratchet, and
  that is slice 3's decision, not this slice's.
  ⛔ **Three soundness properties are held BY CONSTRUCTION rather than by measurement**, each fixing
  a hole in this slice's own first draft (detailed above): `render_elements` frozen with
  `render_elements_display` split off, so nothing here can reach the eliminator's ambiguity refusal;
  `routes_truncated` poisoning `suffix_first` to `unresolved`, so a partial route union can never
  produce a `FEASIBLE`; and `exact` (from `FirstSetSummary::byte_decided`) surfaced as `~` on every
  rendered byte set, so "sound" is never printed where "closed" would be read.
- [x] **ADDRESSED (verified)** — the three answers above, plus the bank:
  `bash docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh` →
  `GUARD-FEASIBILITY-CENSUS: 9/9 as declared`, rc 0. Ground truth in BOTH directions: two cases
  (C4 `undecidable=0`, C5 `no_competition=0`) assert a bucket must stay EMPTY, C7 pins the exactness
  ratio at **157/157 over-approximated**, and a deliberately flipped expectation exits rc 1 naming ⛔ (`157` → **129**: the denominator was 129 sites + 28 candidate rows — `.17` slice 5 RESULT 3; the *claim* is unchanged.)
  the case (`C3 … => FEASIBLE variants=1 max_hops=0 ⛔ want BLOCKED variants=9 max_hops=9` →
  `GUARD-FEASIBILITY-CENSUS: MISMATCH`), so the bank can notice its own breakage.
  ⛔ **ROUTED OUT — none of this slice's four self-found defects was catchable by any of the 18
  doctrines, and the reason is structural: `TASK-ACCEPTANCE` audits the PRESENCE of proof, never its
  FALSIFIABILITY.** The roster's three archetypes (structural / oracle / evidence) all ask *does the
  invariant hold?*; none asks *would this check notice if it did not?* ⇒ **`CI-PARITY-GATE-ROT.27`
  NEW** (parked behind the lane lock), proposing the missing MUTATION archetype and an 8-tier ladder
  — with the measured routing evidence that `--self-test` already exists in 2 `scripts/*.sh` and in
  **0 of 3** tracked probe banks, all three of which proved their falsifiability by hand and recorded
  it as prose.
  ⛔ **C7's own first draft was TAUTOLOGICAL** — it compared `$approx/$sites` against
  `$sites/$sites`, an expectation derived from the number under test, so it would have passed on 0/0
  and on any future count alike; and it read the DEFAULT report, whose per-candidate site list is
  capped at 5, so it measured 123 of 157 sites and called that "every". Now pinned ABSOLUTELY
  against the uncapped report. A tautological assertion inside the gate meant to prevent exactly
  that shape is worth recording, not quietly fixing.
- [x] **NO REGRESSION** — ⭐ **primary argument is BY CONSTRUCTION, with the measurement as
  confirmation.** `indirect_lr_elimination.rs` consumes exactly two things from this module — the
  survey's decision fields and `render_elements` — and neither moves: `is_starvation_safe` and every
  field feeding it are untouched, and `render_elements` is frozen byte-for-byte with a unit test
  pinning its collision behaviour. So no input to codegen can differ, whatever the report prints.
  The measurement agrees, in two independent cuts. BEFORE captured by restoring both files to HEAD,
  rebuilding the SAME binary path and re-running (`PGEN_INDIRECT_LR_DUMP_ALL=1`, three grammars);
  FINAL captured the same way.
  **(i) The frozen-renderer path, checked on its own** — `elimination_ran`, `eliminated_base_rules`,
  `indirect_eliminated_base_rules`, `indirect_clone_rules`, `indirect_refusals`, every
  `✅ absorbed at`, every `⛔ REFUSED …` (whose message text is BUILT from `render_elements`), the
  cycle-rule coverage line and `starvation-safe candidates:` are **BYTE-IDENTICAL** on all three
  grammars. That is the direct evidence that nothing reaching codegen moved.
  **(ii) The whole report** — with the new guard columns removed and both sides normalised for the
  group-parenthesisation fix: `systemverilog` **0** differing lines,
  `systemverilog_lrm_profiled_wrapper` **0**, `ebnf` **0**. Every pre-existing number —
  `candidates`, `declined`, every route, every site — is unmoved.
  `cargo test --lib indirect_lr` **19 passed / 0 failed** (was 12; seven new).
  ⭐ **RED-proven TWICE, not merely green.** (a) Collapsing the verdict chain to an unconditional
  `Guardable` → **15 passed / 3 failed**, naming exactly the cases that carry the criterion
  (`a_residual_the_suffix_can_never_consume_needs_no_guard`,
  `a_suffix_that_can_start_outside_the_residual_blocks_the_guard`,
  `a_nullable_residual_is_bucketed_apart_from_a_disjoint_one`). (b) Removing the `routes_truncated`
  poison **and** un-freezing `render_elements` in one probe → **17 passed / 2 failed**, naming
  `a_truncated_route_set_refuses_a_guard_verdict_instead_of_guessing_one` and
  `a_quantified_group_renders_with_its_parentheses`. Restored to 19/0 after each.
  ⭐ **The exactness marker DISCRIMINATES rather than being always-on** — on the synthetic a
  single-byte residual returns `{!}` with `exact=true` while the multi-element `"'" "(" lit ")"`
  returns `{'}~`. ⛔ That synthetic carries NO layout rule at all, which proves the two causes are
  INDEPENDENT: the structural one alone suffices, and SystemVerilog's nullable `trivia` compounds it
  rather than creating it.
  **Confirmatory sweep** (the repository's established scope — `.13` slice 4b's, for the reason
  recorded under `.3`): `cargo test --features "generated_parsers ebnf_dual_run" --lib --
  --skip deep_nesting` → **1 099 passed / 1 failed / 28 ignored** in 372.63s. The single failure is
  `unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`, **pre-existing and
  owned by `LANG-CAPABILITY-AUDIT.10.15`** — confirmed by NAME and by its verbatim assertion string
  (`expected semantic_annotation fallback to detect '@' directives`) against that leaf's recorded
  symptom, not by assuming a familiar-looking red. ⭐ Both byte-identity gates
  (`every_structural_combinator_is_byte_identical`, `every_semantic_construct_is_byte_identical`),
  `comment_arm_suppression_matrix_is_pinned` and
  `interpreter_agrees_with_compile_and_run_on_synthetic_combinators` are GREEN — those are the ones
  that fire if anything on the parser/codegen path moved.
  `make -C rust clippy_on_rust_change` → **pass** (strict source lint + generated-parser correctness
  stage, roster integrity 68/68).
  ⛔ Every build and test run went through `scripts/run_with_memory_guard.sh --budget-mb 16384`
  WITHOUT a pipe, so the exit code is the job's own — the trap slice 1 recorded and
  `CI-PARITY-GATE-ROT.25` owns.
  ⛔ No scratch-slot obligation is acquired: the probe drives `--report-indirect-lr-plan`, which is
  pure analysis over the post-elimination gen-AST and writes nothing.
- [x] **LOCKSTEP** — `TOOLBOX.md` §5.5 (the two new headline lines, the per-candidate `guard:` line,
  the per-site `[guard=… first=… hops=…]` column, the `no_competition=0` layout finding, and the
  parenthesisation fix); the book's grammar-wellformedness chapter; `.15` below (its founding
  premise, re-adjudicated); `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `docs/TASK_TREE.md`,
  `docs/reference/RUST_CODEBASE_ANALYSIS.md`. No user-visible parser behaviour changed, so no
  contract or schema edit is owed.
  ⭐ **One stale live number found and dated on the way**: `docs/TASK_TREE.md`'s slice-4 census
  (*"wrapper 23/18/11/7"*) was measured BEFORE slice 5's elimination pass and its transitive
  starvation criterion landed; a HEAD binary this session measures the wrapper at **18 candidates /
  3 safe**. It is now marked as the slice-4-era figure it is rather than read as current. ⛔ It was
  NOT moved by this slice — the BEFORE/AFTER capture below proves every structural number unchanged.

##### ✅ `.17` SLICE 3 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0022`, 2026-08-13 session #227) — the census run through the REAL PLANNER: both SV knots reach a plan, `28 → 0`, and the wrapper's refusal was never evidence about the shipped grammar

> **One admission mode + one opt-in report section + a tracked probe bank. ZERO grammar bytes, ZERO
> codegen bytes, ZERO generated-parser bytes, ZERO change to any shipped verdict.**
> The leaf's gate for this slice, in slice 2's own words: *"whether the annotation-composability
> check also passes at `property_expr` is **not** measured here … and is slice 3's first check."*
> ⛔ The design decision is STILL not taken. What changed is that the first check is answered, and
> one of the two facts slice 2 rested it on turns out not to be a fact about this grammar at all.
>
> Dry-run bank (re-runnable, self-checking, 9 cases):
> `docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh` →
> `GUARD-DRY-RUN: 9/9 as declared`.
> ⛔ **`9` is what THIS SLICE measured, not the bank's current size** — slice 7 added D9-D13b and made
> the headline DERIVE from the cases that ran. Narrating record, deliberately not rewritten; read the
> bank's own output for the total.

###### THE INSTRUMENT, AND WHY IT IS THE REAL PLANNER RATHER THAN A MODEL OF IT

`--report-indirect-lr-plan --indirect-lr-plan-guard-dry-run` admits the guard-feasible candidates
into the **shipped** elimination driver, on a CLONE of the grammar, and reports what it did:

```text
--- GUARD DRY-RUN: which guard-feasible candidates actually reach a PLAN ---
    inputs: annotations=present rules_with_branch_return_annotations=1069
    would_absorb=2 would_refuse=0 clone_rules=24 left_recursive_rule_rows 28 -> 0
    ✅ would absorb 'casting_type'
    ✅ would absorb 'property_expr'
```

⭐ **A second implementation of the plan stage would have been the wrong instrument** — it would
drift from the driver it predicts, and the question is precisely *"what does the driver do?"*. So
the admission policy became a parameter (`CandidateAdmission`) and nothing else moved:
`eliminate_indirect_left_recursion` delegates with `StarvationSafe` and is otherwise the same
function. The three refusal sources downstream of the starvation check — a hop with no declared
return annotation, the trial re-lint, the ambiguity comparison — are all **independent of whether a
guard is emitted**, which is what makes a guardless dry run a sound predictor for them.

###### ⭐⭐ THE ANSWER TO THE LEAF'S FIRST CHECK — AND IT IS YES

**`property_expr` composes.** `would_refuse=0` on the shipped grammar: the SVA property knot reaches
a plan, so `.15`'s re-adjudication (its acceptance (a) gains a third and cheapest design) **stands**,
and the doubt slice 2 recorded against it is discharged.

⭐⭐ **And the payoff figure is larger than the leaf assumed.** `left_recursive_rule_rows 28 -> 0`,
re-derived from the rewritten clone by the same `detect_left_recursion` the lint runs — never
inferred as *"before minus absorbed"*. **Two** rewrites clear all **28** surviving rows, because both
are at DOMINATORS: the 16 guard-feasible candidates are not 16 rewrites, they are 16 rules on two
knots. The price is **24 clone rules**.

###### ⛔⛔ THE FACT SLICE 2 REASONED FROM WAS A FACT ABOUT A DIFFERENT GRAMMAR

Slice 2 left `property_expr`'s composability open *because the wrapper refuses it* — *"hop
`property_expr_sv_2017` alternative 5 declares no return annotation"*. The dry run reproduces that
refusal exactly, and then measures why:

| grammar | would_absorb | would_refuse | rules with branch return annotations |
|---|---|---|---|
| `systemverilog` | **2** | **0** | **1069** |
| `systemverilog_lrm_profiled_wrapper` | 0 | **13** | **21** |
| `ebnf` | 0 | 0 | 143 |

**All 13 wrapper refusals are `declares no return annotation`.** The wrapper is a 33-line file over
`systemverilog_lrm_profiled_generated.ebnf` — 148 KB **generated from the LRM Annex A markdown**,
carrying one `->` line in total; the 21 annotated rules are the wrapper's own entrypoints. ⇒ **the
two grammars are not annotation-comparable, and no composability conclusion may be transferred
between them in either direction.** The wrapper remains a good oracle for *structure* (it is the raw
transcription `.15` uses to prove the shipped cascade is scar tissue) and is worthless for
*annotation* questions. ⛔ That distinction did not exist in the leaf before this slice, and slice 2
crossed it without noticing.

⭐ **This is also why the instrument prints its own INPUT.** `compose_route_template` returns
*"nothing to compose"* when a grammar declares no annotations at all, so `would_refuse=0` has two
readings — the chain composes, or there was nothing to compose. The `inputs:` line separates them,
and probe cases D4/D6 pin both sides. Without it this slice's headline would have been unfalsifiable
by construction.

###### ⛔⛔ WHAT `28 → 0` DOES NOT MEAN — the qualifier is concrete, not ceremonial

The dry run **emits no guard**. The grammar it builds is therefore the one `.13` slice 5 measured as
a REGRESSION, and the clearest possible demonstration is that the driver's first pick is
`casting_type` — the exact rule `TOOLBOX.md` §5.5 warns about: *"the rule the lint names FIRST is a
measured regression to eliminate at: rewriting SV's `casting_type` turns the accepted `int'(3)` into
a rejection"* (probe P2). ⇒ `28 → 0` is **plan-stage reachability**, not closure. Stacked with slice
2's exactness result (0 of 157 sites exact), what is now established is:

1. a guard is **expressible and sound** at both knots (slice 2), and
2. the plan the guard would unlock is **buildable and AST-composable** at both knots (this slice),
3. while the guard's *effectiveness* — that a trivia-aware structural lookahead refuses exactly the
   fatal iteration on real SystemVerilog text — is **still unmeasured**, and is slice 4's burden.

⭐ **One routing observation for slice 4, measured rather than assumed.** The driver picked
`casting_type` (guard `max_hops=0`) over `constant_primary` (`max_hops=1`), i.e. the cheaper guard
site — but it did so on the existing `acyclic_alternative_indices` tiebreak (`casting_type` seeds=4,
`cast` seeds=0), **not** because it consulted the census: the ordering does not read the guard
verdict at all. Slice 4 must decide whether to make it guard-aware; on this grammar the coincidence
is favourable, which is exactly the condition under which such a coupling ships unnoticed.

###### Acceptance Checklist (enforced) — `.17` slice 3

- [x] **REPRODUCE / ISSUE** — the leaf's first check was unanswerable from any shipped tool, and the
  reason is structural: `plan_elimination` is only ever reached from `survey.safe_candidates()`, so
  every refusal downstream of the starvation check is unobservable for a STARVED candidate.
  Reproduced as the absence it is — HEAD's report on `systemverilog.ebnf` prints
  `indirect_refusals=0` next to `starvation-safe candidates: 0/28`, i.e. **no refusal data exists
  for any of the 28 rows**, and `property_expr` is one of them.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `indirect_lr_elimination.rs::eliminate_indirect_left_recursion`,
  the loop's admission filter. WHY the leaf could not answer its own question: the starvation check
  is a **gate**, not a score, so a candidate it refuses never reaches `compose_route_template`
  (`:645`), the trial re-lint (`:243`) or the ambiguity comparison (`:483`). ⭐ And the WHY behind
  the wrapper/shipped split is measured, not inferred: `branch_return_annotations` is **1069** on
  `systemverilog.ebnf` against **21** on the wrapper, whose body is LRM-generated and carries a
  single `->` line in 148 KB.
- [x] **FIX** — fix-hierarchy tier = **new tooling / instrument** (no grammar, codegen or
  generated-artifact byte moves; no shipped verdict changes). `CandidateAdmission` +
  `dry_run_guard_feasible_elimination` + `GuardDryRun` in `indirect_lr_elimination.rs`,
  `IndirectChainSurvey::guard_admissible_candidates` in `indirect_lr_plan.rs`, and an **opt-in**
  `--indirect-lr-plan-guard-dry-run` section on the report.
  ⛔ Deliberately NOT done: `is_starvation_safe` is unchanged, the admission widening is reachable
  only from the report flag, and it works on a clone. The shipped pass is byte-for-byte the same
  function it was, with its admission set passed in rather than hard-coded.
- [x] **ADDRESSED (verified)** — the three answers above, plus the bank:
  `bash docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh` →
  `GUARD-DRY-RUN: 9/9 as declared`, rc 0. Ground truth in BOTH directions: D4/D6 pin the
  instrument's own INPUT (so `would_refuse=0` can never be read out of "no annotations"), D8 pins
  that the default report is unchanged, and a deliberately flipped expectation exits rc 1 naming the
  case (`D1 … => 2/0 ⛔ want 0/16` → `GUARD-DRY-RUN: MISMATCH`), with the script restored to a
  byte-identical hash afterwards.
  ⛔ **The bank caught a defect in its own first draft, which is the point of declaring the
  expectation first.** D5b grepped the WHOLE wrapper report for `declares no return annotation` and
  counted **16** — the dry run's 13 plus the 3 `⛔ REFUSED` lines the real pass already prints for
  the candidates it admits today. Two different populations summed into one plausible-looking
  number; it is now scoped to `would still REFUSE`.
- [x] **NO REGRESSION** — ⭐ **primary argument is BY CONSTRUCTION, with the measurement as
  confirmation.** The shipped entry point delegates with `CandidateAdmission::StarvationSafe`, whose
  admission set is the same `survey.safe_candidates()` call the loop made before; the only other
  behavioural branch is `narrates()`, which is `true` on that path. Nothing else in the driver, the
  planner or codegen can see the change.
  The measurement agrees, in two independent cuts, captured by running HEAD's binary against the
  rebuilt one over the same three grammars with `PGEN_INDIRECT_LR_DUMP_ALL=1`. ⭐ The BEFORE binary
  was copied aside *before* the first rebuild and then **verified to be HEAD** rather than assumed —
  it rejects the new flag (`error: unexpected argument '--indirect-lr-plan-guard-dry-run' found`),
  which no post-slice binary does. (It lived under `rust/target/`, i.e. untracked and reproducible by
  rebuilding at HEAD; it is not carried as an artifact.)
  **(i) stdout — the whole default report:** `systemverilog` **0** differing lines,
  `systemverilog_lrm_profiled_wrapper` **0**, `ebnf` **0**.
  **(ii) stderr — what the pass NARRATED:** **0**, **0**, **0**. That is the direct evidence that
  gating the two `eprintln!`s did not move the shipped path, and it is a separate stream from the
  report precisely so it cannot be confused with it.
  `cargo test --features "generated_parsers ebnf_dual_run" --lib indirect_lr` → **21 passed / 0
  failed** (was 19; two new).
  ⭐ **RED-proven TWICE, each probe isolating one property.** (a) Pointing the dry run at
  `CandidateAdmission::StarvationSafe` → **20 passed / 1 failed**, naming
  `the_guard_dry_run_reaches_what_the_shipped_pass_refuses_without_touching_the_grammar` with its
  stated reason (*"an empty outcome here means it saw the same empty admission set the shipped pass
  does"*). (b) Collapsing `guard_admissible_candidates` to `is_starvation_safe` alone → **19 passed /
  2 failed**, adding `the_guard_admission_set_is_the_safe_one_plus_the_guard_feasible_one`. Restored
  to 21/0 after each.
  ⛔ **One assertion in the new driver test is deliberately NOT counted as evidence, and says so in
  its own doc comment.** `dry_run_guard_feasible_elimination` takes `&HashMap` / `&[String]` /
  `&Annotations`, so the three "did not mutate the caller's grammar" checks are already guaranteed by
  the type system and cannot fail while that signature holds. They are a tripwire against a future
  refactor that widens those borrows — at which point the safety argument moves out of the compiler
  and into those three lines. A test that can only pass is documentation
  ([[a-check-whose-inputs-all-pass-has-not-been-tested]]); labelling it beats counting it.
  **Confirmatory sweep** (the repository's established scope — `.13` slice 4b's): `cargo test
  --features "generated_parsers ebnf_dual_run" --lib -- --skip deep_nesting` → **1 101 passed / 1
  failed / 28 ignored** in 623.56s. The count is slice 2's **1 099** plus exactly this slice's two
  new tests. The single failure is
  `unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`, **pre-existing and
  owned by `LANG-CAPABILITY-AUDIT.10.15`** — confirmed by NAME *and* by its verbatim assertion
  string (`expected semantic_annotation fallback to detect '@' directives`, panicking at
  `ast_based_generator.rs:14841`) against the symptom that leaf records at
  `docs/tasks/LANG-CAPABILITY-AUDIT.md:2917`, not by assuming a familiar-looking red.
  `make -C rust clippy_on_rust_change` → **pass** (`clippy_source_all_targets` ok,
  `clippy_generated_all_targets` ok, `Generated-parser clippy stage: pass`, correctness
  roster-integrity 68/68). ⛔ That run was piped through `tail`, i.e. the exact exit-code-masking
  trap `CI-PARITY-GATE-ROT.25` owns, so the verdict is read from the target's OWN terminal lines
  (`✅ clippy_on_rust_change completed.`) rather than from the guard's reported exit.
  ⛔ Every build and test run went through `scripts/run_with_memory_guard.sh --budget-mb 16384`.
  ⛔ No scratch-slot obligation is acquired: the dry run works on a clone and writes nothing.
- [x] **LOCKSTEP** — `TOOLBOX.md` §5.5 (the new flag, its output line and its two qualifiers); the
  book's grammar-wellformedness chapter; `.15` below (its open doubt, discharged); the new probe
  bank's README; `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `docs/TASK_TREE.md`,
  `docs/reference/RUST_CODEBASE_ANALYSIS.md`. No user-visible parser behaviour changed, so no
  contract or schema edit is owed.

##### ✅ `.17` SLICE 4 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0023`, 2026-08-13 session #228) — (iii) is DECIDED, its shape is CORRECTED to TWO guard positions, and the engine law slices 1-3 reasoned from is REFUTED

> **DOCS + a tracked probe bank only — ZERO grammar bytes, ZERO Rust bytes, ZERO codegen bytes,
> ZERO generated-parser bytes.**
> The leaf's gate for this slice, in slice 3's own words: *"the guard's effectiveness — that a
> trivia-aware structural lookahead refuses exactly the fatal iteration on real SystemVerilog text —
> is **still unmeasured**, and is slice 4's burden."* It is now measured, on both oracles. ⛔ Two of
> the three results were not on the leaf's map: option (iii) **as specified through slice 3 does not
> close SystemVerilog's cast knot**, and the engine law that framed the whole design is **false**.
>
> Effectiveness bank (re-runnable, self-checking, 37 rows × 2 oracles):
> `docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh` →
> `GUARD-EFFECTIVENESS: 37/37 as declared`.

###### THE INSTRUMENT — P5's shape, PLUS the one ingredient no probe in this family had

The bank is `.13` slice 5b's `p5_transparent_holder.ebnf` hand-eliminated at `prim`, wrapped in an
enclosing statement so the holder is MANDATORY (`k = … ;` — without the terminator a shorter parse
hides the starvation), and carrying **SystemVerilog's own layout model**: a nullable
`trivia := (line_comment | block_comment)*` leading every token, copied in shape from
`grammars/systemverilog.ebnf:618-624`. Slice 2 measured that layout fact as the second cause of
`exact = 0 of 157`; no synthetic in this tree had ever carried it, which is why no earlier probe
could have found E3.

The mapping is rule-for-rule (full table in `g0_unguarded.ebnf`'s header):
`outer_cast` ~ `cast := casting_type tick lparen expression rparen` (the holder, residual
`tick lparen lit rparen`) · `ct := kw | prim` ~ `casting_type := simple_type | constant_primary | …`
(`:1032`, transparent on its second arm) · `prim := prim_base ( prim_suffix )*` ~
`constant_primary := …_lr_base ( …_lr_suffix )*` · `cast_seed` ~ the sheared `constant_cast` clone
`.13` slice 5 emits.

###### ⭐⭐ RESULT 1 — the structural guard is effective; the byte-set form is DEAD

| | `prim` | e1 `k = n'(n);` | e2 `…'(n)'(n);` | **e3 `…'(n)/*c*/;`** | e4 `…/*c*/'(n);` | e6 `k = t'(n)'(n);` |
|---|---|---|---|---|---|---|
| **G0** | `prim_base ( prim_suffix )*` | REJECT | REJECT | REJECT | REJECT | REJECT |
| **G1** | `… ( prim_suffix &residual_first_byte )*` | ACCEPT | ACCEPT | **REJECT** ⛔ | ACCEPT | ACCEPT |
| **G2** | `… ( prim_suffix &( tick lparen lit rparen ) )*` | ACCEPT | ACCEPT | **ACCEPT** ⭐ | ACCEPT | ACCEPT |

E3 is E1 with a comment at the iteration boundary, and it is the whole discriminator. `trivia` is
nullable and leads `tick`, so `/` ∈ `FIRST(residual)`; the byte test passes exactly where it had to
refuse, the loop commits the fatal iteration, and the holder starves. ⇒ slice 2's *"a trivia-aware
structural lookahead is effectively mandatory rather than a refinement"* is **measured**, not argued.

⭐ **G1 is written as the CHARITABLE form on purpose**, so the result is a lower bound. It is a
lookahead over a regex, so the engine's layout skipper runs first; the emitted Q-GUARD reads
`parser.input.as_bytes()[parser.position]` raw (`ast_based_generator.rs:6014-6020`) and would also
mis-fire on plain whitespace. Even given free whitespace handling the cheap form still slips.

###### ⛔⛔ RESULT 2 — (iii) AS SPECIFIED DOES NOT CLOSE THE KNOT: there is a SECOND starvation, and it is at the CHOICE

`e5` (`k = t'(n);`) **REJECTS under G0, G1 and G2 alike.** Bisected rather than reasoned about: it
still rejects with the quantifier deleted outright (`prim := prim_base`), so no guard on the `*` can
reach it. The over-long match comes from `prim_base`'s own sheared clone `cast_seed`, which matches
the whole cast and wins the `ct` tournament; the holder then has no residual left, and the choice
never gives back (RESULT 4).

⛔ **The shipped analogue is exact, not analogous — and it is READ OFF THE SHIPPED REPORT, not
inferred.** The driver's first pick is `casting_type` (slice 3), whose own clone set contains
`constant_cast`, and the survey already names the site:

```text
[candidate] casting_type  routes=10  seeds=4  clone_cost=14  verdict=STARVED
    clones: … cast, … constant_cast, constant_function_call, constant_primary, …
    ⛔ starved by constant_cast alt#0 (on-route, still reachable after the rewrite)
       — residual 'tick lparen constant_expression rparen' a greedy suffix could steal
```

⛔⛔ **CORRECTED BY SLICE 5 — THIS PARAGRAPH NAMES THE WRONG BASE RULE, AND IT SPECIFIES.** The
shape below is real and is exactly what the `guard_effectiveness` bank models, but it belongs to
**`constant_primary`**, not to `casting_type`. `casting_type`'s ten routes CLOSE at `cast` /
`constant_cast`, each of which has exactly ONE alternative — the cycle-closing one — so the shear
leaves no clone and `casting_type_lr_base` carries **no seed at all** (`seed_routes=0/10`, and the
real eliminator emits no `casting_type_lr_seed_cast`). Read `constant_primary` (`10/10`) wherever
this paragraph says `casting_type`; `casting_type_acyclic` below is `constant_primary`'s closing
clone of `casting_type`, which is why the name looked right. ⛔ Marked rather than rewritten because
the surrounding RESULT-2 argument — that a SECOND starvation exists at the CHOICE and no guard on the
`*` can reach it — is CORRECT and is what decision (c) rests on. Slice 5's RESULT 1 is authoritative
for which rule carries it.

After the rewrite `constant_primary := constant_primary_lr_base ( constant_primary_lr_suffix )*`, and
`constant_primary_lr_base` carries the sheared `constant_cast` clone —
`casting_type_acyclic tick lparen constant_expression rparen`, which is `cast_seed` in this bank,
byte for byte in shape. On `int'(1)` it matches the whole cast, `constant_primary` keeps it as the
longest alternative, and the holder `cast := casting_type tick lparen expression rparen` has no
`tick` left (reached through `casting_type`, which is transparent to `constant_primary` — hence
`max_hops=1` there and `0` at `casting_type`).

⛔ **"It parses today" is MEASURED, not assumed** — this leaf's own slice-4 lesson applied to its own
headline. On the shipped parser at HEAD:

```text
parse_full passed for grammar 'systemverilog' on '…/sv_int_cast.sv'    # module m; initial k = int'(1); endmodule
parse_full passed for grammar 'systemverilog' on '…/sv_size_cast.sv'   # module m; initial k = 8'(1); endmodule
```

⇒ **shipping (iii) with the loop guard alone would trade `.13`'s defect for a new one**, on the same
knot, in the same session that measured `28 → 0`. That is the single most valuable thing this slice
found, and it was found *before* any engine byte moved.

###### ⭐⭐ RESULT 3 — and it is closable, by a SECOND guard POSITION on the SAME clone

| | `prim` | e1 | e2 | e3 | e4 | **e5** | e6 |
|---|---|---|---|---|---|---|---|
| **G2** per-iteration only | `( suffix &R )*` | ✅ | ✅ | ✅ | ✅ | **REJECT** | ✅ |
| **G6** trailing only | `( suffix )* &R` | REJECT | REJECT | REJECT | REJECT | **ACCEPT** | REJECT |
| **G3** both | `( suffix &R )* &R` | ✅ | ✅ | ✅ | ✅ | **ACCEPT** | ✅ |

⭐⭐ **The two positions close DISJOINT starvations and neither is redundant** — G6 is the control
that proves it, and it is the exact complement of G2:

- the **per-iteration** guard stops the LOOP at the right count, and cannot touch an over-long SEED
  (on `e5` the loop runs zero times);
- the **trailing** guard refuses an over-long SEED, and cannot touch the LOOP (by the time it runs
  the possessive `*` has committed to the maximum count, and there is no give-back to a shorter one
  — so it can only turn a starved parse into a failed one).

⭐ **Why the trailing guard costs no give-back**, which is what keeps it inside the second
non-negotiable: refusing an over-long seed makes that branch **fail** rather than win, and a failed
branch is not a losing branch — the tournament simply has one fewer candidate and picks the next.
The repair converts a would-be give-back into a branch failure the shipped selection already handles.

⛔ **It is still CALL-SITE scoped, and that is measured, not inherited from slice 1's Q5.** G4 is G3
plus a second, residual-free holder of `prim`; on `e7` (`k = n;`) it **REJECTS**, because the
trailing guard demands a residual that holder never wanted. G5 — G4 with the trailing guard removed —
**ACCEPTS** the same input, so the REJECT is the guard and not the shape. ⇒ both guards hang on the
same sheared clone the plan already emits; the transformation buys **one** clone chain carrying two
lookaheads, not two chains.

###### ⛔⛔ RESULT 4 — SLICE 1's FINDING 1 IS FALSE, AND SO IS THE DECISION RECORD IT PRODUCED

Slice 1's FINDING 1 — *"the same engine gives back at a CHOICE and refuses to at a QUANTIFIER"* — is
the premise for this leaf's central framing (*"`.17`'s blocker is an asymmetry between two of PGEN's
own combinators, not a property of PEG"*) and was promoted to a decision record,
[[project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier]].

**It rests on Q2** — `( "a" | "ab" ) "c"` on `abc` ⇒ ACCEPT, read as *"`"a"` wins, `"c"` fails, the
choice gives back and retries `"ab"`."* ⛔ That does not follow, and slice 1's own Q3 is why: the
default policy is `longest_match`, so `"ab"` wins OUTRIGHT and `"c"` matches the single remaining
byte. The parse completes on the first and only alternative the choice ever kept. **Q2's verdict is
predicted identically with and without a give-back.**

The discriminating shape is one where both alternatives match and only the SHORTER lets the caller
finish. Measured on both oracles:

| | grammar | input | GEN | INTERP | reading |
|---|---|---|---|---|---|
| **C1** | `ch := "a" \| "ab"` · `scratch := ch "bc"` | `abc` | **REJECT** | REJECT | the choice does **not** give back |
| **C2** | the same under `@branch_policy: ordered` | `abc` | ACCEPT | ACCEPT | the parse EXISTS ⇒ C1 is the commit, not the grammar |
| **C3** | slice 1's Q2, as a rule | `abc` | ACCEPT | ACCEPT | Q2's verdict reproduced — and non-discriminating |

**Located.** The tournament declares ONE winner slot —
`let mut best_content: Option<ParseContent<'input>> = None;` (`ast_based_generator.rs:5037`) — the
policy cascade at `:4380-4440` only decides whether a candidate DETHRONES the incumbent, and the
winner is consumed once at `:5102`. No losing alternative is retained anywhere, so there is nothing
for a caller-failure to retry against. The *"evaluate all branches"* comment at `:4273` that slice 1
cited is about **selection**, not retention.

⇒ [[project_pgen_gives_back_at_neither_combinator]] supersedes the earlier record, which is retained
and marked (per `MEMORY_ARCHITECTURE.md` *"supersede, don't mutate"*). What survives: the possessive
quantifier; *costs are REJECTED, not traded* implemented as **prove the cost away at generation
time** rather than *never attempt twice*; and the call-site scoping of the repair. What does not: the
combinator asymmetry, and **any argument for a re-enterable `*` that leans on parity with the
choice** — option (i) would be the FIRST give-back in the engine and must be priced as new behaviour.

⭐ **The transferable lesson, and it is bigger than `.17`.** Slice 1's bank is self-checking in both
directions, carries must-accept and must-reject cases, and exits non-zero on a flipped expectation —
and it still could not notice, because the expectation it checked (`ACCEPT`) was **the right answer
for the wrong reason**. A control that passes under both hypotheses discriminates nothing, and a
*mechanism* may never be read out of one. Sibling of
[[a-check-whose-inputs-all-pass-has-not-been-tested]]: that one is about inputs that never exercise a
branch, this one about an output that never separates two explanations.

###### ⭐⭐ THE DECISION THIS SLICE WAS ASKED TO TAKE

**(a) ADOPT option (iii), in its STRUCTURAL form, with TWO guard positions on one sheared clone.**

```text
X_guarded := X_lr_base ( X_lr_suffix &( residual ) )* &( residual )
```

The byte-set form is refused outright — not "preferred against" — because slice 2 measured exactness
at **0 of 157** sites and E3 shows what that costs on the first comment. The structural form's price ⛔ (`157` → **129**: the denominator was 129 sites + 28 candidate rows — `.17` slice 5 RESULT 3; the *claim* is unchanged.)
is one residual sub-parse per committed iteration plus one at rule exit, and it is confined by
construction: the guard lives on a clone reached only from the holder, so every other caller of the
base rule pays nothing, and `no_competition` / `residual_nullable` sites are owed no guard at all
(slice 2: 29 of 126 SV sites).

**(b) REFUSE option (i) (a re-enterable `*`).** Unchanged in outcome, strengthened in reason: with
FINDING 1 refuted it can no longer be argued as restoring parity, and its memo burden — a rule with
several valid results at one position — is untouched by anything measured here. (iii) needs no
give-back anywhere, which is precisely why it survived the refutation intact.

**(c) (iii) MUST NOT SHIP WITHOUT THE TRAILING GUARD.** RESULT 2 is not a caveat; it is a predicted
regression on `initial k = int'(1);`. Slice 5 implements both positions or neither.

**(d) NO NEW LEAF IS OPENED FOR THE SEED STARVATION.** It is not a separate defect — it is the same
follow restriction at the same call site in a second position, and it is fixed by the same clone.
Routing it out would split one design across two trees.

###### THE ORDERING QUESTION — deferred to slice 5, and the reason is now stronger than slice 3's

Slice 3 asked whether the driver's candidate ordering should become guard-aware, having measured
that it picked `casting_type` (`max_hops=0`) over `constant_primary` (`max_hops=1`) on the existing
`acyclic_alternative_indices` tiebreak — *"the coincidence is favourable, which is exactly the
condition under which such a coupling ships unnoticed."*

**The ordering, located and read** (`indirect_lr_elimination.rs:330-343`) — four keys, none of them
about guards:

```text
covered_rules().len()               DESC   prefer the DOMINATOR of a knot
acyclic_alternative_indices.len()   DESC   prefer the rule with more acyclic seeds
clone_cost().len()                  ASC    prefer the cheaper clone set
position(base_rule)                 ASC    deterministic tiebreak
```

Re-measured this session on the shipped grammar: `casting_type` `routes=10 seeds=4 clone_cost=14`
beats `constant_primary` `routes=10 seeds=0 clone_cost=14` on key 2, and the dry run picks
`casting_type` + `property_expr` for `would_absorb=2 … 28 -> 0`.

⛔ **This slice does not answer it, and deliberately so: the question changed shape under RESULT 2.**
A candidate's guard cost is no longer `guard_hops` alone — it is *(hops for the loop guard)* **plus**
*(whether the base's own seed set can swallow the holder's residual)*, and the second term does not
exist in `GuardAssessment` at all (`verdict` / `residual_first` / `guard_hops`, `indirect_lr_plan.rs`).
Wiring the ordering to the census as it stands would hard-code a preference derived from a model
known to be incomplete — the same shape of error as slice 2 reasoning from the wrapper's annotation
count. ⇒ **slice 5 extends the census with the seed term first, then decides the ordering against a
complete model.** Recorded here so the deferral is a decision with a reason, not an omission.

###### Acceptance Checklist (enforced) — `.17` slice 4

- [x] **REPRODUCE / ISSUE** — the leaf's own gate was unanswerable from any shipped surface, and in
  two independent ways. (1) The guard's effect had never been executed: slice 2 proved a guard
  EXPRESSIBLE and slice 3 proved the plan BUILDABLE, but no artifact in this tree had ever run a
  guarded parse, so *"the guard refuses exactly the fatal iteration"* was an unmeasured premise.
  Reproduced as the starvation it repairs — `g0_unguarded` REJECTs all six inputs on BOTH oracles,
  `Backtrack at position 9` on `k = n'(n);`. (2) The engine law the design rests on had never been
  run on a discriminating case (see ROOT CAUSE).
- [x] **ROOT CAUSE (WHY + WHERE)** — two, both located, both re-measured rather than quoted.
  - **The byte guard's failure.** WHY: `trivia := (line_comment | block_comment)*`
    (`grammars/systemverilog.ebnf:619`) is nullable and leads every token, so `/` ∈ `FIRST` of every
    token and a byte test over `FIRST(residual)` passes at a position the residual cannot start
    from. WHERE the shipped form would live: the Q-GUARD's byte-set test,
    `ast_based_generator.rs:6006-6020` (`!matches!(parser.input.as_bytes()[parser.position], …)`).
    Measured consequence: `g1_byte_guard` REJECTs `k = n'(n)/*c*/;` where `g2_structural_guard`
    accepts it — one input, one difference.
  - **The second starvation, and the false law behind it.** WHY: the multi-branch tournament keeps
    exactly ONE winner — `let mut best_content: Option<ParseContent<'input>> = None;`
    (`ast_based_generator.rs:5037`), dethrone-only cascade at `:4380-4440`, consumed once at
    `:5102` — so a successful-but-losing alternative is DISCARDED and there is nothing for a
    caller-failure to retry against. The *"evaluate all branches"* comment at `:4273` that slice 1
    cited describes SELECTION, not retention. ⇒ an over-long seed starves its holder exactly as an
    over-long loop does, and `.17`'s framing of the blocker as a quantifier-only asymmetry was
    wrong at the root.
- [x] **FIX** — fix-hierarchy tier = **design decision + tracked probe bank** (no engine, grammar,
  codegen or generated-artifact byte moves; no shipped verdict changes). The leaf's design gate is
  discharged and the shape of (iii) is CORRECTED before any of it is built: two guard positions on
  one sheared clone, structural rather than byte-set, with the ordering deliberately deferred and
  the reason recorded. ⛔ Deliberately NOT done: `is_starvation_safe` is untouched, no guard is
  emitted, and `CandidateAdmission` is unchanged — the implementation is slice 5, and it now has a
  specification that cannot ship the regression this slice found.
  ⛔ The durable layer is corrected too, not just the leaf:
  [[project_pgen_gives_back_at_neither_combinator]] supersedes
  [[project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier]], which is RETAINED and marked
  (`MEMORY_ARCHITECTURE.md` *"supersede, don't mutate"*), and slice 1's bank carries the correction
  to Q2's reading **with its verdict unchanged** — the expectation was right, only the mechanism read
  out of it was wrong.
- [x] **ADDRESSED (verified)** — `bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh`
  → `GUARD-EFFECTIVENESS: 37/37 as declared`, rc 0, **every row on BOTH oracles with zero DIVERGE**.
  Ground truth in both directions: 19 rows must ACCEPT and 18 must REJECT, and the ladder is built
  so that each rung differs from its neighbour by exactly one construct (G0→G1→G2 the guard's form,
  G2↔G6 its position, G3 both, G4↔G5 its scoping, C1↔C2 the branch policy).
  ⭐ **Falsifiability proven, not asserted** — flipping `g2_structural_guard e3` from `ACCEPT` to
  `REJECT` exits **rc 1** naming that row (`g2_structural_guard e3 … ACCEPT REJECT ⛔` →
  `GUARD-EFFECTIVENESS: MISMATCH`), and the script was restored to a byte-identical hash afterwards.
  ⛔ That rc was read from an UNPIPED run: the first attempt read it through `| grep` and got the
  grep's `0`, which is the exact exit-code-masking trap `CI-PARITY-GATE-ROT.25` owns.
  ⛔ **The bank refuses a case that cannot discriminate rather than running it.** G4/G5 are measured
  on `e1` and `e7` only — their second alternative absorbs `e2`–`e6` on its own, so those rows would
  pass under both hypotheses. That is this slice's own finding applied to its own instrument.
  ⭐ **The GEN arm is what makes RESULT 4 a claim about the engine.** `c1_choice_never_gives_back`
  REJECTs and `c2_ordered_control` ACCEPTs on the REAL generated parser, not only in the
  interpreter — so the superseding decision record rests on shipped codegen.
- [x] **NO REGRESSION** — ⭐ **BY CONSTRUCTION, and the one obligation this slice DOES acquire is
  verified rather than assumed.** ZERO bytes under `grammars/`, ZERO under `rust/src/`, ZERO codegen,
  ZERO generated artifacts staged; nothing shipped was touched, so no parser input can differ.
  ⛔ The bank drives the scratch slot, which is precisely the obligation `.13` slice 4b root-caused:
  `generated/scratch_parser.rs` is git-ignored, so a fixture-only restore leaves a clean-looking tree
  and two RED gates. `probe.sh` restores in BOTH halves on every exit path (`git checkout` **then**
  `make focus_scratch`, regenerating the artifact FROM the restored fixture), and the result is
  checked rather than trusted: `grammars/scratch/scratch.ebnf` hashes to
  `a8caa53d5a0d038dd8670d9d5383f457e8bcafc724bf3a22494127afd94d9273`, its pre-run value, and
  `git status --porcelain grammars/scratch/` is empty.
  The two gates that read the slot as a matched pair were re-run after the bank, each by EXACT name:
  `parse_harness_equivalence::gate::certified_grammars_are_byte_identical` → `test result: ok.
  1 passed; 0 failed` (23.39s) and
  `parser_registry::tests::scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast` →
  `test result: ok. 1 passed; 0 failed`, alongside `parser_registry::tests` **42 passed / 0 failed**.
  ⛔ **The second gate had to be re-run to be claimed at all**, and the reason is worth carrying: the
  first attempt piped `cargo test` into `tail -12`, so the log kept the summary line and dropped the
  per-test lines — the run was green, but the specific test could not be shown to have executed.
  That is `CI-PARITY-GATE-ROT.25`'s trap in its *second* form: the pipe did not mask an exit code
  here, it masked the EVIDENCE. A summary that says `42 passed` is not a statement about which 42.
  ⛔ Every build and probe run went through `scripts/run_with_memory_guard.sh --budget-mb 16384`
  WITHOUT a pipe, so the reported exit is the job's own (bank: `exit=0 peak_tree_rss=10660MB
  elapsed=645s`).
- [x] **LOCKSTEP** — `TOOLBOX.md` §5.5 (the three slice-4 results, the corrected reading of
  `starvation-safe 0/28` as *commit-once* rather than *the greedy star*, and the ordering keys);
  the book's grammar-wellformedness chapter; `docs/decisions/` (the superseding record + INDEX);
  slice 1's `quantifier_policy/probe.sh` header (Q2's reading corrected, verdicts unchanged); the new
  bank's README; `.15` below (the seed-starvation question, unmeasured there); `KNOWLEDGE_MAP.md`
  (regenerated — the new decision record is a fact source); `MEMORY.md`, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `docs/TASK_TREE.md`. No user-visible parser behaviour changed, so no
  contract or schema edit is owed.
  ⛔ **`MEMORY.md` was TRIMMED, not grown, to land this.** The rewritten `next_action` first came out
  at 7 346 bytes against the 7 168-byte cap; the overflow was demoted to this leaf rather than the
  cap raised (`MEMORY-ARCH`, *"a cap is never raised to land content"*). Final: 30 lines / 7 107
  bytes, `check_memory_architecture.sh` rc 0.

##### ⛔ `.17` SLICE 4b (`PGEN-ENGINE-UNIVERSAL-SERVICES-0024`, 2026-08-14 session #228) — slice 4's own LOCKSTEP missed two citations of the record it refuted, and one of them was SOURCE CODE

> **A doc comment + a reference line + one promoted discipline record. ZERO grammar bytes, ZERO
> executable Rust bytes** (the only `rust/src/` change is a `///` block), ZERO codegen, ZERO
> generated artifacts.

Slice 4 superseded [[project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier]] and updated
the surfaces it *remembered* citing it. Asked afterwards whether the decision was signoff-grade, the
first check run was the one slice 4 never ran: `grep -rl` for the refuted record's name. **Twelve
files cite it; two were unswept.**

- ⛔⛔ **`rust/src/ast_pipeline/indirect_lr_plan.rs:323`** — and it is worse than a stale link. That
  doc comment is the census module's *"## What the guard IS"* section, and it still SPECIFIED the
  superseded design: `X_guarded := X_lr_base ( X_lr_suffix &FIRST(residual) )*` — the byte-set form
  slice 4 measured dead, in the single position slice 4 measured insufficient. The next reader of
  the module — most plausibly slice 5, in a fresh session, reading the code rather than the leaf —
  would have implemented exactly the shape that regresses `initial k = int'(1);`. Now carries both
  corrections inline, with the pointer to `GuardAssessment`'s missing seed term and the explicit
  order not to make the ordering guard-aware before it exists.
- **`docs/reference/RUST_CODEBASE_ANALYSIS.md:76`** — *"Durable form: [[…]]"* re-pointed. Its
  fix-tier conclusion (a gen-AST → gen-AST rewrite) is unaffected and says so.

⭐ **The other ten citations are CORRECT and must stay**, which is why the sweep is a judgement and
not a `sed`: `KNOWLEDGE_MAP.md` is derived, `CHANGES.md` and `docs/TASK_TREE.md` are append-only
history that must quote the name they were written with, the two `quantifier_policy` files and this
leaf's FINDING 1 carry the correction *beside* the original, and both decision records must name
each other. ⇒ the rule is **re-point what SPECIFIES, mark what NARRATES, never touch history**.

⛔ **THE PROCESS DEFECT, named.** Slice 4's LOCKSTEP box enumerated the surfaces it edited. That is a
list of what the author remembered, and nothing measured it against the surfaces that actually
reference the changed fact. A supersede is a rename with a blast radius, and the blast radius is
`grep -rl <old_id>`, not recall. ⛔ It is also invisible to the doctrine roster: `KNOWLEDGE-MAP`
checks the derived map is in sync with its *fact sources*, never that a superseded id stopped being
cited as live — the same shape as `LESSON-PROMOTION`'s own founding gap.
⇒ **routed to `CI-PARITY-GATE-ROT.28` NEW** (parked behind the lane lock, with the measured routing
evidence: 12 citations, 2 stale, 1 in `rust/src/`, caught by hand after the gate said 18/18).

⭐ **AND THE TRANSFERABLE HALF OF SLICE 4 IS NOW FIRST-CLASS, not buried.** Slice 4 wrote its lesson
into `DEVELOPMENT_NOTES.md` and into the *project* record's closing section, where someone asking
*"is my probe bank enough to claim a mechanism?"* would never find it. Promoted to
[[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] — a `feedback` discipline
record with its own `answers:` keys, sibling to
[[a-check-whose-inputs-all-pass-has-not-been-tested]]. A discipline filed under the defect that
produced it is retrievable only by people who already know the defect.

###### Acceptance Checklist (enforced) — `.17` slice 4b

- [x] **REPRODUCE / ISSUE** — `grep -rl project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier`
  over the tracked tree returns **12** files after slice 4 landed; two of them present the refuted
  law as live, and one is a `rust/src/` doc comment specifying the superseded design.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `indirect_lr_plan.rs:323` (the `GuardVerdict` module
  doc) and `docs/reference/RUST_CODEBASE_ANALYSIS.md:76`. WHY: slice 4's LOCKSTEP was an
  author-recalled enumeration, and no doctrine measures a superseded record's remaining citations —
  `KNOWLEDGE-MAP` verifies the derived map against its fact sources, which a stale *citation* does
  not touch. The gate reported 18/18 with both defects present.
- [x] **FIX** — fix-hierarchy tier = **documentation correctness** (no executable byte moves). Both
  citations re-pointed, the source-code one additionally corrected on substance (byte-set →
  structural, one position → two, plus the missing seed term and the ordering embargo). The
  ten legitimate citations were classified and deliberately left.
- [x] **ADDRESSED (verified)** — re-swept: `grep -rl` now shows the refuted id only where it is
  narrated or explicitly marked superseded, and `grep -rl project_pgen_gives_back_at_neither_combinator`
  covers both corrected surfaces. `KNOWLEDGE_MAP.md` regenerated (`knowledge-map: OK — facts valid,
  ids unique, map in sync`). `bash scripts/check_doctrines.sh` → **ALL 18 PASS** on the staged diff.
- [x] **NO REGRESSION** — the only `rust/src/` change is a `///` doc comment; no item, signature,
  expression or attribute moves, so codegen output cannot differ by construction. Confirmed by
  compiling: `cargo test --features "generated_parsers ebnf_dual_run" --lib indirect_lr` →
  **21 passed / 0 failed**, unchanged from slice 3's count. No scratch-slot obligation is acquired.
- [x] **LOCKSTEP** — `docs/decisions/` (the new `feedback` record + INDEX), `KNOWLEDGE_MAP.md`,
  `docs/reference/RUST_CODEBASE_ANALYSIS.md`, `rust/src/ast_pipeline/indirect_lr_plan.rs`,
  `CHANGES.md`, `docs/TASK_TREE.md`. `MEMORY.md` unchanged — the frontier did not move.

##### ⭐⭐ `.17` SLICE 5 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0025`, 2026-08-14 session #229) — the SEED term is in the census, it SPLITS the two candidates on SystemVerilog's cast knot, and slice 4's shipped analogue named the wrong base rule

> **ANALYSIS + report columns + a pinned bank. ZERO grammar bytes, ZERO codegen bytes, and the
> generated tree re-derived byte-for-byte identical** (all 11 parsers, hashes below). `is_starvation_safe`
> — the verdict the shipped eliminator acts on — is untouched.
>
> Slice 4's deferral, verbatim: *"a candidate's guard cost is no longer `guard_hops` alone — it is
> (hops for the loop guard) **plus** (whether the base's own seed set can swallow the holder's
> residual), and the second term does not exist in `GuardAssessment` at all … slice 5 extends the
> census with the seed term first, then decides the ordering against a complete model."* This slice
> is that first job. ⛔ The ordering is still NOT decided — see the closing section for why the
> answer changed shape again.

###### THE TERM, DERIVED FROM THE ELIMINATOR RATHER THAN FROM THE DESIGN NOTE

`X_lr_base`'s cyclic alternative is `clone(steps[1].rule) ++ residual(steps[0])`, each clone in turn
`clone(steps[i+1].rule) ++ residual(steps[i])` — until the **cycle-closing** step, whose alternative
`clone_rule` deletes outright (`Shear::Drop`, `indirect_lr_elimination.rs:681`) rather than
redirecting. Two consequences, and neither is visible from `suffix_first`:

1. **The seed tail is the route suffix MINUS the closing step's residual.** That residual travels
   with the dropped alternative and reaches the `*` alone. `ChainRoute::seed_tail_elements`.
2. **A route whose clone chain does not survive contributes no seed at all.** `clone_rule` returns
   `None` when every alternative of a rule was sheared away, and the referring alternative then
   vanishes from its parent in turn. `clone_chain_survives` mirrors exactly that rule.

⇒ `GuardAssessment::seed_verdict` (`SeedVerdict`: `no_seed_tail` · `seed_no_competition` ·
`seed_residual_nullable` · `trailing_guard_required` · `seed_undecidable`), plus candidate-level
`seed_first` / `seed_tail_routes` / `requires_trailing_guard()`.

⛔ **No `seed_incomplete` tier, deliberately, and the asymmetry with `GuardVerdict` is the point.**
The loop guard can cut a chain short at an intermediate iteration — that is what `guard_incomplete`
names. The trailing guard runs ONCE, at rule exit, on a clone reached only from a holder that wants
the residual next, so refusing an over-long seed there cannot lose a derivation that holder had.

###### ⭐⭐ RESULT 1 — the term SPLITS one knot's two candidates, and slice 4's shipped analogue named the wrong one

```text
[candidate] casting_type      seed: no trailing guard owed      seed_first={}       seed_routes=0/10
[candidate] constant_primary  seed: TRAILING GUARD REQUIRED     seed_first={'/}~    seed_routes=10/10
[candidate] property_expr     seed: TRAILING GUARD REQUIRED     seed_first={/aiosu}~ seed_routes=78/80
[candidate] cast              seed: TRAILING GUARD REQUIRED     seed_first={./}~    seed_routes=8/8
```

`casting_type`'s ten routes all close at `cast` or `constant_cast`, and **each of those has exactly
one alternative** (`grammars/systemverilog.ebnf:1029`, `:1539`) — the cycle-closing one. The shear
deletes it, the clone is `None`, and no seed chain reaches `casting_type_lr_base`.

⛔⛔ **GROUND TRUTH, from the REAL eliminator and not from this model.** The guard dry run's own
clone set for `casting_type` is 12 rules — `casting_type_lr_seed_constant_primary`,
`…_constant_primary_sv_2017/_sv_2023`, `…_constant_function_call`, `…_call_primary`,
`…_call_with_postfix_chain`, `…_chainable_call_initial`, `…_direct_callable_method_call`,
`…_method_call_root`, `…_method_call_receiver`, `…_method_call_receiver_sv_2017/_sv_2023` — and
contains **no `casting_type_lr_seed_cast` and no `casting_type_lr_seed_constant_cast`**. Pinned as
bank case C9c.

⇒ slice 4's RESULT 2 wrote *"After the rewrite `casting_type := casting_type_lr_base (…)*`, and
`casting_type_lr_base` carries the sheared `constant_cast` clone — `casting_type_acyclic tick lparen
constant_expression rparen`"*. That shape is real and it is the one the `guard_effectiveness` bank
models, but it belongs to **`constant_primary`** (whose routes close at `casting_type`, a
many-alternative rule whose clone keeps `simple_type` and its siblings), not to `casting_type`. The
bank's own mapping table said so all along — `prim ~ constant_primary` — and slice 4 transposed it
onto the rule the driver happens to pick.

⭐ **Slice 4's DECISION (c) is UNAFFECTED and is now per-candidate measured rather than universal.**
"(iii) must not ship without the trailing guard" stands: SystemVerilog **15/28** candidates need it,
the LRM wrapper **13/18**. What changes is the regression prediction attached to it — at
`casting_type` specifically the loop guard alone would NOT regress `initial k = int'(1);`, because
there is no seed there to over-consume it. At `property_expr`, the driver's *other* pick, it would.

###### ⭐ RESULT 2 — `ebnf`, the one knot the eliminator absorbs TODAY, owes no trailing guard

`0/5` candidates, census `seed_no_competition=6`. ⇒ the rewrite PGEN already ships carries no seed
starvation, so this slice's finding is not a latent defect in a shipped parser. Pinned as C10.

###### ⛔⛔ RESULT 3 — THE PINNED `157/157` WAS NEVER THE SITE COUNT, AND THE BANK FOUND IT ONLY BECAUSE A NEW LINE MADE THE AMBIGUITY VISIBLE

`guard_feasibility/probe.sh` C7 extracted with `grep -c 'first='`, which counts **lines** containing
that substring — and the per-candidate summary line `guard: … suffix_first=…` carries it too. So the
pinned `157` was **129 starvation sites + 28 candidate rows**, one row per candidate, and the count
had been quoted as a site count in TOOLBOX, the book, two decision records, the knowledge base and
two bank READMEs. Adding slice 5's `seed_first=` line took it to 185 and the bank went red.

Re-adjudicated rather than re-fitted, with two independent extractions agreeing:

| | count |
|---|---|
| lines containing `first=` (the old extractor) | 185 |
| … of which per-candidate `guard:` lines | 28 |
| … of which per-candidate `seed:` lines (new) | 28 |
| site lines, by `[guard=` | **129** |
| site lines, by `— residual '` (independent) | **129** |
| `⛔ starved by` (surviving) + `·  benign site` | 126 + 3 = **129** |
| surviving-site census `68 + 29 + 29` | **126** ✅ |

⭐ **The CLAIM was never at risk and that is worth stating precisely**: every candidate summary is
`~` as well, so *"not one exact guard exists"* held under either count — only the denominator was
inflated. C7 is now anchored on `[guard=` and `~ hops=`, each occurring exactly once per site line
and nowhere else, and **C7b pins the wrapper at 77/77**, which the `157`-era bank never counted at
all.

⛔ **The transferable half.** A bank that reads a report by substring is coupled to every line the
report will ever grow. The failure mode is silent in the direction that matters — a *larger*
denominator makes an "N of N" ratio look like broader coverage — and it survived a slice that
explicitly re-derived the case for being tautological and for reading a capped report. ⇒ an
extractor must anchor on something STRUCTURAL in the line it means (a bracket group, a marker in its
only legal position), never on a substring that a sibling line may also contain. Recorded as
[[a-report-scraper-must-anchor-on-structure-not-on-a-substring]].

###### THE ORDERING QUESTION — still open, and the reason changed AGAIN

Slice 4 deferred it because `GuardAssessment` had no seed term. It has one now, and the answer moved
anyway: the driver's current pick `casting_type` is **cheaper** on the seed axis (`0/10`, no trailing
guard) while `constant_primary` is cheaper on nothing and costs `max_hops=1` besides. So the existing
four-key sort already lands on the seed-cheap candidate here — for reasons that have nothing to do
with guards, which is exactly the coincidence slice 3 flagged.

⛔ **Not decided here, and the missing input is now a MEASUREMENT, not a model.** Making the ordering
guard-aware changes which knots get absorbed ⇒ a shipped parser change ⇒ it needs the two-sided
repro ratchet and a corpus re-measure, which is a slice of its own. What this slice removes is the
excuse: the model is complete, so the next slice can decide against it.

###### Acceptance Checklist (enforced) — `.17` slice 5

- [x] **REPRODUCE / ISSUE** — the census could not answer the question slice 4 handed it. Before this
  slice, `--report-indirect-lr-plan` printed the same `guard: FEASIBLE … max_hops=` line for
  `casting_type` and `constant_primary`, with no column anywhere in the report or the JSON that
  distinguishes a candidate whose sheared clone can starve its holder from one whose cannot — and
  slice 4 had already measured that difference as the gap between a working repair and a REGRESSION
  (`guard_effectiveness` rows G2/G6/G3 on `e5`).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `GuardAssessment` carried `verdict` / `residual_first` /
  `guard_hops`, all three derived from `suffix_first`, which is the FIRST set of what the **loop**
  iterates. The seed's tail is a different string — the suffix minus the cycle-closing step's
  residual — and whether it exists at all depends on clone survival, neither of which the struct
  modelled. WHERE: `rust/src/ast_pipeline/indirect_lr_plan.rs` (`GuardAssessment`, `assess_guard`,
  and the candidate build in `survey_indirect_left_recursion_with_route_budget`); the shear/drop
  semantics it has to mirror are `indirect_lr_elimination.rs:465-476` (the shear map) and `:675-706`
  (`clone_rule`, including the `kept.is_empty()` → `None` path).
- [x] **FIX** — fix-hierarchy tier = **analysis module + report/JSON columns + bank** (no engine
  behaviour, no grammar byte, no codegen byte). `SeedVerdict` + `GuardAssessment::seed_verdict`;
  `ChainRoute::seed_tail_elements` / `closing_step`; `IndirectChainCandidate::seed_first` /
  `seed_tail_routes` / `seed_blocking_sites` / `trailing_guard_sites` / `requires_trailing_guard`;
  `IndirectChainSurvey::seed_verdict_census` / `trailing_guard_candidates`; report lines and JSON
  fields. ⛔ `is_guard_feasible` now reads BOTH positions — a feasibility claim silent about a
  starvation slice 4 measured is the defect, not the caution — and the cost of that was **measured
  as zero**: `seed_undecidable=0` on every shipped grammar, so 16/28, 13/18 and 5/5 are unmoved and
  the dry run still reports `would_absorb=2 … 28 -> 0`.
  ⛔ Deliberately NOT done: no guard is emitted, `is_starvation_safe` is untouched, and the candidate
  ordering is unchanged.
- [x] **ADDRESSED (verified)** — `bash docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh`
  → `GUARD-FEASIBILITY-CENSUS: 18/18 as declared`, rc 0 read from an UNPIPED run
  (`CI-PARITY-GATE-ROT.25`), with eight NEW cases: C7b (the wrapper's exactness, never counted
  before), C8a-d (the seed census and the 15/28 headline), C9a-c (the `casting_type` ↔
  `constant_primary` split **plus the eliminator's own clone set as ground truth**) and C10 (`ebnf`
  at 0/5). `cargo test --lib indirect_lr` → **24 passed / 0 failed** (21 before).
  ⭐ **Falsifiability proven, not asserted, on BOTH axes of the term** — and the two probes are
  deliberately different, because a single break could not have exercised both:
  - the seed-tail arithmetic: `.rev().skip(1)` → `.skip(0)` ⇒ `the_seed_tail_is_the_route_suffix_minus_the_closing_step_residual`
    and `a_seed_can_starve_a_holder_the_loop_provably_cannot` FAIL by name (`left: "{'}~" right: "{!}"`);
  - clone survival: the shear map forced empty ⇒ `a_seed_can_starve_a_holder_the_loop_provably_cannot`
    FAILS on *"no clone, no seed"*.
  The source was restored to a byte-identical hash afterwards
  (`f177552474996dffb420f9b172b3f69d7aa2619839e6ca5fdbeb1ff49bc60de5`) and the suite re-run green.
  ⛔ **And the honesty note this slice owes its own test**: `two_base_rules_on_one_knot_disagree_about_the_seed`
  survived BOTH breaks, because on the `ct` side the empty tail alone forces the verdict — its
  docstring now says so instead of implying it measures survival
  ([[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]]).
- [x] **NO REGRESSION** — ⭐ **measured at the strongest available tier, not argued from the diff.**
  `make -C rust regenerate_generated_parsers` under the memory guard (`exit=0 peak_tree_rss=6444MB
  elapsed=262s`) re-derived all **11** generated parsers with the new binary, and
  `shasum -a 256 generated/*.rs` is byte-identical to the pre-change snapshot — `ebnf.rs`,
  `systemverilog_parser.rs`, `vhdl_parser.rs` and the eight others, every hash unchanged. ⇒ no parser
  input can behave differently, because no parser byte moved.
  The two report-reading banks both pass (`GUARD-DRY-RUN: 9/9`, `GUARD-FEASIBILITY-CENSUS: 18/18`).
  ⛔ The three scratch-slot banks (`indirect_lr`, `quantifier_policy`, `guard_effectiveness`) were
  deliberately NOT run: none of them reads this report, they exercise PARSE behaviour, and parse
  behaviour is proven unchanged by the byte-identical generated tree — running them would acquire
  `.13` slice 4b's scratch-slot restore obligation for no measurement. Stated rather than silent.
- [x] **LOCKSTEP** — `TOOLBOX.md` §5.5 (the `seed:` line, the per-site `seed=` column, the verdict
  table, the `casting_type`/`constant_primary` split, and the ordering bullet, which claimed the seed
  term "is not in `GuardAssessment` at all" and is now false); the book's grammar-wellformedness
  chapter (a new *"Which candidates actually need the trailing guard"* section, the corrected sample
  report, and a marker on the superseded `&FIRST(residual)` snippet); `docs/decisions/` (the new
  scraper-anchoring record + INDEX); `docs/knowledge/` + `KNOWLEDGE_MAP.md` (regenerated); the two
  bank READMEs; `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`.
  ⛔ **The `157 → 129` correction was swept by `grep -rn`, not by recall** — slice 4b's lesson
  applied on the slice that inherited it. Twelve citations: six SPECIFY and were corrected
  (`indirect_lr_plan.rs`, `TOOLBOX.md` ×3 loci, the knowledge record, the `neither_combinator`
  decision record, both bank READMEs, the bank itself), and the rest NARRATE — `CHANGES.md` is
  append-only history and keeps the number it was written with, and this leaf's slice-2/3/4 boxes
  are marked in place rather than rewritten.

##### ⭐⭐ `.17` SLICE 6 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0026`, 2026-08-14 session #229) — the CALL-SITE-SCOPED shape is BUILT and MEASURED, and it was the one rung the design had never expressed

> **A tracked probe grammar + bank rows + docs. ZERO grammar bytes under `grammars/` (the scratch
> slot is driven and restored), ZERO Rust bytes, ZERO codegen bytes, ZERO generated artifacts staged.**
>
> Slice 4 decided the guard is call-site scoped and slice 5 completed the model. Neither BUILT the
> shape. This slice does, so that slice 7's planner has a measured rule-for-rule target instead of a
> design note — and so the decision itself is falsifiable before any engine byte moves.

###### ⛔⛔ THE HOLE, NAMED — the bank proved the two halves and never the combination

Slice 4's ladder ends in three rungs that do not meet:

| | shape | measured |
|---|---|---|
| `g3_trailing_guard` | both lookaheads on the SHARED rule `prim` | closes all six starvation inputs |
| `g4_trailing_guard_needs_a_clone` | `g3` + a second, residual-FREE holder of `prim` | ⛔ `e7` (`k = n;`) **REJECTS** |
| `g5_trailing_guard_clone_control` | `g4` minus the trailing guard | `e7` accepts ⇒ `g4`'s REJECT is the guard, not the shape |

⇒ the decision that followed — *"both belong on a clone reached only from the holder"* — was the
**only** shape with `g3`'s power and without `g4`'s damage, and **nothing in this tree had ever run
it.** A design whose deciding artifact does not exist is a design believed, not measured; that is
this leaf's own recurring failure mode ([[a-check-whose-inputs-all-pass-has-not-been-tested]],
[[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]]) applied one level up, to
the *set* of rungs rather than to one rung's inputs.

###### THE SHAPE — `g7_guarded_clone_chain.ebnf`, and it is `g4` with ONE difference

```text
                     g4 (guard on the shared rule)          g7 (guard on a clone chain)
holder               outer_cast := ct …                     outer_cast := ct_guard …
transparent hop      ct   := kw | prim                      ct   := kw | prim          ← untouched
                                                            ct_guard := kw | prim_guard
the loop             prim := … &( … ) … &( … )              prim := prim_base ( prim_suffix )*   ← untouched
                                                            prim_guard := prim_base ( prim_suffix &( R ) )* &( R )
```

Same two holders, same seven inputs; the only change is WHERE the two lookaheads live. The chain's
length is exactly the census's `guard_hops` (here `1`) — one guarded clone per transparent hop plus
the guarded base — and only the holder's LEFT CORNER is repointed, so every other element and
therefore every `$N` position in its annotation is preserved.

⭐ **The fallback is the mechanism, and it is why the guard must sit INSIDE a choice rather than at
the holder's own call.** On `e5` (`k = t'(n);`) the seed `cast_seed` matches `t'(n)` entire, the
trailing guard refuses it, and `prim_guard` FAILS — at which point `ct_guard`'s other alternative
`kw` wins with the short match `t` and `outer_cast` gets its `'(n)` back. A guard that made
`outer_cast` itself fail would have had nothing to fall back to.

###### ⭐⭐ RESULT — it does both, on both oracles

`GUARD-EFFECTIVENESS: 44/44 as declared`, rc 0, **zero `DIVERGE`**:

| input | `g4` | `g7` |
|---|---|---|
| `e1`–`e6` (the six starvation shapes) | `e1` ✅ (others not run — see below) | **all six ACCEPT** |
| `e7` `k = n;` (the residual-FREE holder) | ⛔ **REJECT** | ⭐⭐ **ACCEPT** |

⇒ one grammar with `g3`'s power and without `g4`'s damage. The decision slice 4 recorded is now a
measured shape rather than an inference, and slice 7's planner has a rule-for-rule target.

⛔ **`g7` runs all seven inputs where `g4`/`g5` run two, and the asymmetry is deliberate.** `g4`/`g5`
are restricted to `e1`/`e7` because their second alternative absorbs `e2`–`e6` on its own, so those
rows would pass under both hypotheses. `g7` has no such shortcut on `e2`–`e6`: those inputs contain
the cast, which only reaches a parse through `outer_cast` — the guarded chain — so every row
discriminates.

###### ⛔ TWO STALE CLAIMS IN THE BANK'S OWN DOCUMENTATION, found while extending it

Neither changes a verdict; both are the class `.17` slice 4b and slice 5 keep finding — a prose
number or pointer that nothing derives, left behind when the artifact around it grew.

- **`probe.sh`'s header said *"19 cases must ACCEPT and 18 must REJECT"*.** The 37-row bank it
  described was **21/16**; both numbers were wrong, and the script never reads them (it compares row
  by row). Now stated as what a count of the `CASES` table returns: **28/16** over 44 rows.
- **`README.md` said *"Routed to leaf `.19`"*, and no such leaf exists.** Slice 4's decision (d)
  explicitly REFUSED to route the seed starvation out. The line predates that decision and was never
  swept. ⭐ The same README names **`constant_primary`** as the base carrying the `constant_cast`
  clone — which slice 5 measured as correct, and which the leaf's slice-4 prose had transposed onto
  `casting_type`. The bank's README was right where the leaf was wrong.
- `README.md`'s ladder table also documented only `g0`/`g1`/`g2`, four rungs behind the bank slice 4
  itself shipped. Now complete.

###### WHAT SLICE 7 INHERITS — the planner's obligations, read off `g7`

1. **Reconstruct the transparent CHAIN, not just its depth.** `rules_transparent_to` returns a depth
   map (`indirect_lr_plan.rs:1292`); the planner needs the actual rule path from the holder's
   referenced rule down to the base, because each hop becomes one guarded clone.
2. **Copy the non-chain alternatives verbatim** — `ct_guard := kw | prim_guard` keeps `kw`, and that
   copy is what the tournament falls back to. A clone that dropped it would close nothing.
3. **One chain per distinct residual**, ~~which is exactly what `guard_variants` already counts~~.
   ⛔⛔ **CORRECTED BY SLICE 7, AND IT SPECIFIES.** `guard_variants` counts distinct residual **byte
   sets** — the key the DEAD byte-test form would have compared. The shipped guard is a structural
   sub-parse, so the chain key is the residual's STRUCTURE (plus its position set and its chain), and
   the census figure is only a LOWER BOUND. Measured on the driver's own first pick: `casting_type`
   reports `variants=1` and the planner emits **two** chains, because `tick lparen
   constant_expression rparen` and `tick lparen expression rparen` share a FIRST set and are
   different sub-parses.
4. **Emit which positions each site needs**, from slice 5's two verdicts: the loop guard where
   `guard=guardable`, the trailing guard where `seed=trailing_guard_required`. `casting_type` needs
   only the first; `constant_primary` and `property_expr` need both.
5. **The trailing lookahead appends**, so `$N` positions are unaffected: a lookahead emits
   `ParseContent::Sequence(Vec::new())` (`ast_based_generator.rs:4155`) and `original_body_length`
   only BOUNDS the template's valid `$N` range (`lr_chain_fold.rs:317-321`) — it is not an index.
   Verified by reading both, not assumed.

###### Acceptance Checklist (enforced) — `.17` slice 6

- [x] **REPRODUCE / ISSUE** — the decision's own shape had never been executed. Reproduced as the
  gap between two rungs that both exist: `g3_trailing_guard e5` ACCEPTs (the guard works) while
  `g4_trailing_guard_needs_a_clone e7` REJECTs (the same guard breaks a residual-free holder), and
  no grammar in the bank did both. Every claim that the clone chain resolves that tension was, until
  this slice, an argument.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the shared-rule form breaks: the trailing `&( residual )`
  is a property of the CALL SITE, not of the rule, so a holder that wants the whole run of `prim`
  with no residual after it is refused by a guard it never asked for (`g4` vs `g5`, one difference).
  WHY the clone chain fixes it: the guard is reachable only through `outer_cast → ct_guard →
  prim_guard`, and the second holder still reaches the untouched `prim`. WHERE the fallback comes
  from: the guarded clone of the transparent hop keeps its OTHER alternative, so when
  `prim_guard` fails the `ct_guard` tournament still has `kw` — the same single-winner tournament
  (`ast_based_generator.rs:5037`) that [[project_pgen_gives_back_at_neither_combinator]] describes,
  used here as the recovery path rather than fought.
- [x] **FIX** — fix-hierarchy tier = **a tracked probe grammar + bank rows + documentation**. No
  engine, grammar, codegen or generated-artifact byte moves; no shipped verdict changes.
  `g7_guarded_clone_chain.ebnf` + 7 rows, plus the two stale-claim corrections above.
  ⛔ Deliberately NOT done: the planner is not written, `is_starvation_safe` is untouched, and no
  guard is emitted by anything. That is slice 7, and it now has a target.
- [x] **ADDRESSED (verified)** — `bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh`
  → **`GUARD-EFFECTIVENESS: 44/44 as declared`**, rc 0 read from an UNPIPED run under the memory
  guard (`exit=0 peak_tree_rss=12687MB elapsed=866s`), **every row on BOTH oracles with zero
  DIVERGE**. The seven new rows are `g7 e1`–`e7`, all ACCEPT on GEN and INTERP alike — including
  `e7`, the row `g4` REJECTs three lines above it in the same run, which is what makes the pair a
  measurement rather than two separate observations.
  ⭐ The bank was ALSO run `--interp-only` first as a cheap pre-check; that arm can never carry the
  claim on its own (`.13` slice 3's recorded lesson) and is recorded here only as sequence, not as
  evidence.
- [x] **NO REGRESSION** — the 37 pre-existing rows are unchanged and all still pass, so nothing this
  slice added moved an earlier verdict.
  ⛔ **The scratch-slot obligation was ACQUIRED and is verified discharged, not trusted** (`.13`
  slice 4b's class): `grammars/scratch/scratch.ebnf` hashes to
  `a8caa53d5a0d038dd8670d9d5383f457e8bcafc724bf3a22494127afd94d9273`, its pre-run value, and
  `git status --porcelain grammars/scratch/` is empty. The two gates that read the slot as a matched
  pair were re-run by EXACT name with per-test lines shown, not a summary:
  `parse_harness_equivalence::gate::certified_grammars_are_byte_identical` → `test result: ok.
  1 passed; 0 failed` (22.93s), and
  `parser_registry::tests::scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast` →
  `ok`, inside `parser_registry::tests` **42 passed / 0 failed**.
  No `rust/src/` or `grammars/` byte moved, so no generated parser can differ.
- [x] **LOCKSTEP** — the bank's `probe.sh` header + `README.md` (ladder table completed, counts
  re-derived, the `.19` pointer corrected); `TOOLBOX.md` §5.5; the book's grammar-wellformedness
  chapter; `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`. No user-visible
  parser behaviour changed, so no contract or schema edit is owed.
  ⛔ `promotion: declined (a third card would COLLIDE with two already in the retrievable layer, making the map worse at the one job it has)`
  — the slice-6 lesson is the LADDER-LEVEL restatement of
  [[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] (a rung whose output
  cannot separate two explanations) and of
  [[a-report-scraper-must-anchor-on-structure-not-on-a-substring]] (the prose-count half). Its
  `answers:` keys would overlap both, and retrieval degrades when two cards answer one question.
  Recorded in full in the slice-6 record above and in `DEVELOPMENT_NOTES.md` instead.

##### ⛔ `.17` SLICE 6b (`PGEN-ENGINE-UNIVERSAL-SERVICES-0027`, 2026-08-14 session #229) — asked whether slices 5/6 were signoff-grade, the audit found ONE of the three corrections had not been swept, and it was the one that SPECIFIES

> **DOCS + two probe-bank shell edits. ZERO grammar bytes, ZERO Rust bytes, ZERO codegen bytes,
> ZERO generated artifacts.** Prompted by a director check — *"hope you took signoff-grade decisions
> on those three findings"* — answered by re-auditing rather than by asserting.

**GAP A (the dangerous one) — slice 4's RESULT 2 still named the wrong base rule, in a SPECIFYING
paragraph.** Slice 5 measured that `casting_type_lr_base` carries no seed (`seed_routes=0/10`;
`cast`/`constant_cast` each have one alternative, so the shear leaves no clone) and recorded the
correction in its OWN section — but slice 4's paragraph still read *"After the rewrite
`casting_type := casting_type_lr_base (…)*`, and `casting_type_lr_base` carries the sheared
`constant_cast` clone"*. ⛔⛔ **That is `.17` slice 4b's defect reproduced by the slice that inherited
its lesson**: the next reader is slice 7, building the guard chain, and the most likely thing they
read is the design paragraph — not the census section three screens later. Now corrected in place
(`constant_primary`), with the RESULT-2 argument itself deliberately RETAINED because it is correct
and decision (c) rests on it.

⭐ **Why slice 5's own sweep missed it.** The sweep was `grep -rn "157"` — the number it had just
disproved. The wrong RULE NAME contains no number, so a numeric sweep could not see it. ⇒ **a
correction has TWO blast radii — the value you changed and the CLAIM it was part of** — and only the
first is greppable. The second needs re-reading the paragraph the value lived in.

**GAP B — the banks' headline totals were STORED, and one of them rotted three times in one
session.** `guard_feasibility`'s summary line carried a hand-typed `9/9`, which I edited to `10/10`
and then `18/18` while adding cases; `guard_effectiveness`'s header prose carried a split that was
wrong for its own bank twice over. Both are `docs/DERIVED_STATE_CONTAINMENT.md` R1/R3 violations — a
number a command answers exactly, written where it can drift. Fixed by DERIVING:
`guard_feasibility` counts cases inside `check()` itself (`$checks/$checks`), and
`guard_effectiveness` computes and PRINTS the split from the `CASES` table
(`ground truth, both directions: 44 rows — 28 must ACCEPT, 16 must REJECT`) plus two new guards — a
row declaring neither verdict, and the loss of either direction, both fail the bank.

**FINDING 3 was already signoff-grade** (the `.19` pointer and the stale ladder table were corrected
in place in slice 6), and **FINDING 2's own fix was too** (six specifying surfaces corrected, the
narrating ones marked, history untouched) — but its *promotion* was a knowledge card and nothing
more. A card is retrievable; it is not unavoidable. ⇒ **routed to `CI-PARITY-GATE-ROT.29` NEW** with
measured routing evidence: two instances one week apart in the same bank family, both found by
accident, both failing in the flattering direction, and the existing 18-doctrine roster blind to
each (`GATE-REACHABILITY` asks if a check is invoked, `.27` if it is falsifiable, `KNOWLEDGE-MAP`
re-derives from record files — none asks whether a bank's arithmetic describes the bank).

###### Acceptance Checklist (enforced) — `.17` slice 6b

- [x] **REPRODUCE / ISSUE** — `sed -n '3121,3140p' docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` at
  `d4e6cca9` returns the uncorrected slice-4 paragraph naming `casting_type_lr_base` as the carrier
  of the `constant_cast` clone, three screens above slice 5's measurement refuting it. And
  `grep -n "as declared" docs/tasks/artifacts/engine_universal_services/*/probe.sh` returns one
  hand-typed total (`18/18`) beside one derived one (`$total/$total`) — the asymmetry is the defect.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` slice 4
  RESULT 2 (two paragraphs), `guard_feasibility/probe.sh` summary line,
  `guard_effectiveness/probe.sh` header + summary. WHY (A): slice 5's citation sweep was keyed on the
  numeric value it corrected, and the co-located claim it belonged to carries no number — a numeric
  grep cannot find a wrong identifier. WHY (B): a total stated in prose has no deriving command
  behind it, so nothing fails when the table it describes grows; measured three drifts of the same
  number inside one session.
- [x] **FIX** — fix-hierarchy tier = **documentation correctness + instrument hardening**. (A) the
  specifying paragraphs corrected in place, the correct argument retained and marked. (B) both totals
  derived at run time, plus two NEW refusals in `guard_effectiveness` (malformed row, lost
  direction). (C) the mechanizable class ROUTED to `CI-PARITY-GATE-ROT.29` rather than left as a
  knowledge card.
- [x] **ADDRESSED (verified)** — `guard_feasibility/probe.sh` → `GUARD-FEASIBILITY-CENSUS: 18/18 as
  declared`, rc 0, the headline now DERIVED and agreeing with the value it replaced (which is what
  makes the derivation trustworthy rather than merely new). `guard_effectiveness/probe.sh
  --interp-only` → `ground truth, both directions: 44 rows — 28 must ACCEPT, 16 must REJECT`,
  `44/44`, rc 0.
  ⭐ **The new refusal is PLANT-PROVEN, not asserted** ([[a-check-whose-inputs-all-pass-has-not-been-tested]]):
  a planted `zz_planted_malformed abc MAYBE` row exits **rc 1** printing `45 rows — 28 must ACCEPT,
  16 must REJECT` and `⛔ a CASES row declares neither ACCEPT nor REJECT`. ⛔ The restore of that
  plant is itself recorded as a MISTAKE worth carrying: `git checkout --` was used on a file that had
  UNCOMMITTED edits, so it reverted to `d4e6cca9` and silently discarded them. Caught by comparing
  the post-restore hash against the pre-plant hash — which is exactly why that comparison is in the
  procedure — and the edits were re-applied and re-verified. **Snapshot the CONTENT, not the commit,
  before planting into a dirty file.**
- [x] **NO REGRESSION** — no row, grammar, oracle or expectation changed; `guard_effectiveness`'s
  edits are a header comment and post-loop arithmetic, so the 44/44 GEN result measured at
  `d4e6cca9` still stands for every row and is not re-claimed here. ⛔ The GEN arm was deliberately
  NOT re-run (~14 min + a scratch-slot obligation) because nothing this slice touches can reach a
  parse; stated rather than silent. No `rust/src/` or `grammars/` byte moved.
- [x] **LOCKSTEP** — this leaf (slice 4 RESULT 2 corrected in place), both probe banks,
  `docs/tasks/CI-PARITY-GATE-ROT.md` (`.29` NEW), `CHANGES.md`, `docs/TASK_TREE.md`.
  `MEMORY.md` unchanged — the frontier did not move. `DEVELOPMENT_NOTES.md` unchanged: the durable
  lesson here is *"a correction has two blast radii"*, recorded in this box, and
  `promotion: declined (it is the same discipline as CI-PARITY-GATE-ROT.29's acceptance (c), which now owns turning it into a gate)`.

##### ⭐⭐ `.17` SLICE 7 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0028`, 2026-08-14 session #230) — the PLANNER emits the `g7` shape, and BOTH of the census's prices for it were measured too low

> **ENGINE code + report/JSON columns + bank rows. ZERO grammar bytes, and the generated tree
> re-derived BYTE-IDENTICAL — all 11 parsers, hashes below.** The guard planner runs on every plan;
> what keeps it silent on the shipped path is the admission criterion, not a flag.
>
> Slice 6's closing section, verbatim: *"⇒ this grammar is the rule-for-rule TARGET a guard planner
> must synthesize."* This slice is that planner. ⛔ It emits the shape and nothing PARSES it — see
> the honest bound at the end, which is narrower than slice 6's and still not closure.

###### WHAT WAS BUILT, against slice 6's five obligations

| # | obligation | where it landed |
|---|---|---|
| 1 | reconstruct the transparent CHAIN, not its depth | `guard_chain_from` → `StarvationSite::guard_chain` (`indirect_lr_plan.rs`) |
| 2 | copy the non-chain alternatives verbatim | `plan_guard_chains`, the `None` arm of the left-corner match |
| 3 | one chain per distinct residual | ⛔ **refuted as stated — see RESULT 2**; the key is (positions, chain, residual STRUCTURE) |
| 4 | emit which positions each site needs | `GuardVerdict::Guardable` → loop, `SeedVerdict::Required` → trailing |
| 5 | the trailing lookahead appends | held — and it covers only that position; the loop one is designed AROUND, see RESULT 3 |

The emitted shape, from the SystemVerilog dry run's own output:

```text
🛡  guard 'property_expr_lr_guard0' [loop+trailing]  chain: property_expr  residual 'implies property_expr'
    sites: prop_primary_sv_2017 alt#13, prop_primary_sv_2023 alt#13
    rules: property_expr_lr_guard0_suffix, property_expr_lr_guard0
```

###### ⭐⭐ RESULT 1 — the positions the EMITTER writes agree with slice 5's census, and the two are different code paths

`casting_type` gets `[loop]` on both of its chains; `property_expr` gets `[loop+trailing]`. That is
slice 5's `seed:` line — `casting_type seed_routes=0/10` *"no trailing guard owed"*, `property_expr
78/80` *"TRAILING GUARD REQUIRED"* — reproduced by a component that never reads it as a headline: the
emitter reads `SeedVerdict` per SITE, the report prints a per-CANDIDATE aggregate. ⭐ Two paths, one
answer, and the bank pins the pair (D10) so a future disagreement names the candidate rather than
showing up as a parse failure three slices later. ⛔ The `[positions]` string D10 compares is read
back off the EMITTED grammar, not off the plan — see RESULT 4, where the first version of it was not.

###### ⛔⛔ RESULT 2 — `guard_variants` UNDER-COUNTS THE CHAINS, and it does so on the driver's own first pick

Slice 6's obligation 3 said *"one chain per distinct residual, which is exactly what `guard_variants`
already counts."* Measured: **it is not**.

```text
[candidate] casting_type  …  variants=1 {'/}~          ← the census
🛡  guard 'casting_type_lr_guard0' … residual 'tick lparen constant_expression rparen'
🛡  guard 'casting_type_lr_guard1' … residual 'tick lparen expression rparen'
```

`guard_variants` keys on `residual_first.render()` — a FIRST **byte set**. That was the right key for
the byte-test form, and slice 4 measured that form **dead**. The shipped guard is a structural
sub-parse, so two residuals with one FIRST set are two different rules; `constant_cast` wants
`constant_expression` inside its parentheses and `cast` wants `expression`. ⇒ **the census figure is
a LOWER BOUND on option (iii)'s rule-name price**, off by one at `casting_type` and pinned as the
PAIR `1,2` (bank D12) so the gap itself is the measurement rather than a number that later gets
"corrected" to agree.

⭐ **Why this is the same defect class the leaf keeps finding, one level up.** Slice 2 priced the
design with the byte set because the byte set was the design; slice 4 killed the byte form and
nobody re-priced. A number that outlives the decision it was derived under is
[[a-report-scraper-must-anchor-on-structure-not-on-a-substring]] wearing a different hat — here the
anchor is a *model*, not a substring.

###### ⛔ RESULT 3 — the LOOP guard cannot be written inline, and the reason is the AST fold

Slice 6's obligation 5 verified that a TRAILING lookahead is safe because it appends and
`original_body_length` only bounds the template's `$N` range. That argument does **not** extend to
the loop position. Writing `X_lr_guard{v} := X_lr_base ( X_lr_suffix &( R ) )* &( R )` literally
makes the quantifier iterate a **Sequence** rather than the suffix rule, and `$2` — the list
`fold_lr_chain` consumes (`apply_plan`'s `PositionalRef { index: 2 }`) — is read as a list of
`{alt_index, captures}` records.

⛔ **What that inline group's `$2` actually becomes is NOT MEASURED here, and the design is what
makes the measurement unnecessary rather than what excuses it.** Slice 6 could answer its obligation
by reading two files because a trailing lookahead only APPENDS; the loop position changes the
quantified element's own type, which is a question about `ParseContent` shapes that would need a
generated parser and an input to settle. Rather than answer it, this slice removes it: hoisting the
group into a rule makes the quantified element a bare rule reference — exactly what `X` itself
carries — so there is no group-content question left to have. Stated because "I did not need to
measure it" and "I measured it" are different claims and only one of them is true.

⇒ the emitter hoists it into a rule instead:

```text
X_lr_guard{v}        := X_lr_base ( X_lr_guard{v}_suffix )* &( R )
X_lr_guard{v}_suffix := X_lr_suffix &( R )          -> $1
```

which is `g7` with the inline group named — the identical language, and the SAME transformation
`apply_plan` already applies to the suffix branches for the same kind of reason (*"a branch is a RULE
and not an inline alternative … a rule can carry `@profiles:` and an alternative cannot"*). ⭐ The
principle is stronger than the workaround: **the guarded base rule is `X`'s body with one element
appended and one rule name substituted**, so the fold's `$N` are unchanged BY CONSTRUCTION rather
than by an argument about group content. The test asserts the two folds are byte-equal serialized.

###### ⛔⛔ RESULT 4 — THIS SLICE'S OWN INSTRUMENT NARRATED THE PLAN, AND THE FALSIFIABILITY PLANT CAUGHT IT

The first version of `GuardChain::summary` built `positions` from `self.loop_guard` /
`self.trailing_guard` — the PLAN's two booleans. The bank rows D9/D10 read that report. So when the
plant *"delete the trailing-lookahead emission in `apply_plan`"* was applied and the binary rebuilt:

```text
D10  systemverilog: the chains, with their guard positions
     => casting_type_lr_guard0[loop],…,property_expr_lr_guard0[loop+trailing]   ✅   ⛔ WRONG
```

**14/14, green, and describing a lookahead the emitted grammar no longer carried.** The unit test
caught the same break (it asserts the rule body), so the defect was in the REPORT and its bank rows,
not in the emitter — but a bank whose rows cannot see the emission is a bank that would have signed
off on a broken one.

⛔ **`synthesized_clone_rules` was audited for the same defect and deliberately LEFT ALONE**, and the
distinction is the useful part: `apply_plan` inserts every clone unconditionally, so its plan and its
artifact cannot diverge. The guard positions are the field with a CONDITIONAL emission behind them,
which is precisely why they are the field that could lie. ⇒ read back what is conditionally emitted;
a value with no branch between plan and artifact is not the same risk.

⛔ **Fixed by DERIVING, not by adding a row.** `summary` now reads the tree `apply_plan` just wrote:
a position is present iff the rule that should carry it ENDS in a positive `Lookahead`, and a rule
name the plan allocated but nothing wrote is filtered out of `rules`. Re-planted afterwards, and it
now flips by name — `property_expr_lr_guard0[loop]`, `GUARD-DRY-RUN: MISMATCH (14 case(s) checked)`.
The summary is therefore built AFTER `apply_plan` rather than before, which is why the call moved.

⭐ **The transferable half, and it is the third instance of one shape in three slices.** Slice 5
pinned a count a sibling line could inflate; slice 6b found two bank totals nobody re-derived; this
one wrote a report from the DECISION rather than from the ARTIFACT. All three fail silently and in
the flattering direction, and all three are the same rule: **a report about a thing must be computed
from that thing**. `IndirectEliminationOutcome`'s own docstring had already said so — *"what the pass
DID, as opposed to a belief about it"* — and the new field was the one that did not obey it. Routed
to `CI-PARITY-GATE-ROT.29`, which slice 6b opened for exactly this class and which now has a third
measured instance in its evidence.

###### ⛔⛔ RESULT 5 — `max_hops` UNDER-PRICES THE CLONE COUNT TOO, and the claim that it does not was written BY THIS SLICE

Having found RESULT 2 by building the thing the census priced, the same question was asked of the
other price. This slice's own JSON comment had just asserted *"`guard_hops` is `guard_chain.len() - 1`
by construction"*. Swept across the shipped grammars rather than believed:

```text
systemverilog                      sites=129  guard_hops == len(chain)-1 violations=6
systemverilog_lrm_profiled_wrapper sites=77   guard_hops == len(chain)-1 violations=5
ebnf                               sites=0    violations=0
```

Every violation is a **dialect twin**, and the shape is identical in all eleven:

```text
base=cast   holder=expression_base alt#1   hops=3
            chain=[expression_operand, primary, primary_sv_2017, primary_sv_2023, cast]
```

`primary` reaches `cast` through `primary_sv_2017` **AND** `primary_sv_2023`. `guard_hops` is the
SHORTEST transparency distance — 3 — while the guard has to clone **five** rules, because leaving
either twin unguarded leaves a live unguarded route to the same starvation. ⇒ **`max_hops` is a
lower bound on the clone count exactly as `guard_variants` is a lower bound on the chain count**: two
independent under-prices in one census, both invisible until something was built against them.

⛔ **What was wrong here was MY OWN sentence, one hour old.** The code already said the right thing
(`guard_chain`'s docstring: *"a SET in DAG order, not a single path, because transparency can
branch"*) — and the JSON comment beside it asserted the equality that contradicts it. Corrected in
both places, `chain=` added to the per-site report line so the disagreement is READABLE rather than
derivable, and pinned as bank rows **D13 (6) / D13b (5)** with a third data point (`ebnf`/`vhdl` → 0)
proving the extractor is reading its input rather than returning a constant.

⭐ **And it exposed an untested code path in this slice's own emitter.** Neither the SystemVerilog dry
run (all chains length 1) nor the P5 fixture has a branching chain, so the loop that repoints BOTH
arms had never executed. `knot_a_annotated_with_branching_transparency` + the new test close that,
RED-proved by a plant that follows only the first transparent arm
(`left: ["ct","pa","prim"]` vs `right: ["ct","pa","pb","prim"]`).

⛔ **A third thing the probe found, and only because it was run rather than reasoned about.** The
first fixture gave the two transparent arms DIFFERENT return annotations, and `plan_elimination`
refused the plan outright — *"two routes iterate the identical suffix … but declare different
ASTs"*. That refusal is correct and pre-existing, and it is worth recording as a property of
branching transparency: **two transparent arms that are syntactically indistinguishable must agree on
their AST or be separated by a `@profiles:` gate**, which is exactly what SystemVerilog's twins do
and what `SuffixBranch::profile_key` exists to carry.

###### ⛔⛔ THE HOP-CLONE HALF IS NOT EXERCISED BY SYSTEMVERILOG, AND SAYING SO IS THE POINT

All three chains the SV dry run synthesizes have `chain:` of length 1 — `max_hops=0` at both
`casting_type` and `property_expr`, i.e. every holder names its base rule DIRECTLY. So the
corpus-scale run walks straight past `X_lr_guard{v}_<hop>`, the rule that carries the whole
call-site-scoping argument. The only coverage of it is the two unit tests —
`the_guarded_clone_chain_reproduces_the_hand_written_g7_shape` (one hop, asserted rule for rule
against the bank grammar) and
`a_branching_transparent_chain_clones_every_arm_and_guard_hops_undercounts_them` (a branching hop) —
and they are therefore load-bearing rather than illustrative. ⛔ Stated rather than left implicit: a
reader who sees `guard_chains=3` on SystemVerilog and concludes the hop machinery is corpus-proven
would be wrong. ⭐ The `cast` / `constant_cast` candidates DO have branching chains
(RESULT 5) — the driver simply never picks them, so the shipped grammar contains the shape and the
dry run never emits for it.

###### Acceptance Checklist (enforced) — `.17` slice 7

- [x] **REPRODUCE / ISSUE** — nothing in PGEN could emit the shape slice 6 measured. Reproduced from
  the shipped surface at `671d8f3e`: `--report-indirect-lr-plan --indirect-lr-plan-guard-dry-run` on
  `grammars/systemverilog.ebnf` printed `would_absorb=2 would_refuse=0 clone_rules=24
  left_recursive_rule_rows 28 -> 0` with **no guard rule of any kind**, under a banner that said so
  — *"it emits no guard and is NOT a claim that the rewrite parses"*. The design was decided
  (slice 4), modelled (slice 5) and BUILT as a grammar (slice 6); the engine could not produce it.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the emission was missing rather than broken: the survey
  carried the guard's *verdicts* and its *depth* but neither of the two things an emitter needs — the
  residual as ELEMENTS (`StarvationSite::residual` is a rendered `String`) and the transparent chain
  as RULES (`rules_transparent_to` returns `BTreeMap<String, usize>`, a depth map,
  `indirect_lr_plan.rs:1292`). WHERE the emission had to attach: `plan_elimination` step 6 and
  `apply_plan`, because the guarded base rule must reference `X_lr_base` / `X_lr_suffix`, which do not
  exist until the plan is built. ⛔ And WHERE it must NOT attach: the admission filter
  (`indirect_lr_elimination.rs`, `CandidateAdmission::StarvationSafe => survey.safe_candidates()`) is
  untouched, so the population that reaches a guard is unchanged.
- [x] **FIX** — fix-hierarchy tier = **engine capability**, the first in this leaf.
  `indirect_lr_plan`: `guard_chain_from` + `StarvationSite::guard_chain`, `left_corner_step` made
  `pub` (so the emitter reads a residual through the survey's own narrowness instead of re-deriving
  it), and `render_node_with` parenthesises a multi-element lookahead body — ⛔ **gated on
  `parenthesize_groups`, so `render_elements` stays byte-frozen for the ambiguity refusal that
  compares it** (`.17` slice 2's standing constraint).
  `indirect_lr_elimination`: `GuardChain` / `GuardRedirect` / `SynthesizedGuard`, `plan_guard_chains`,
  `site_residual`, the emission and holder-repointing passes in `apply_plan`, and
  `IndirectEliminationOutcome::synthesized_guards`. Report: `guard_chains=` / `guard_rules=` / the
  `🛡` lines / `indirect_guard_chains=` on the shipped header / `guard_chain` and `guard_chains` in
  the JSON.
  ⛔ Deliberately NOT done: the admission is unchanged, the candidate ORDERING is unchanged, and no
  generated parser byte moves. Those are the shipped-parser change and they are owed the two-sided
  repro ratchet plus a corpus re-measure.
  ⛔⛔ **AND THE NEXT SLICE IS NOT THAT FLIP — this box's own first draft said it was, and that was
  wrong.** What this slice emits has **never been executed**: `guard_effectiveness` measured a
  HAND-WRITTEN `g7`, and slice 7 asserts PGEN's emission matches it in the gen-AST. Nothing has
  generated a parser from PGEN's own guarded output and run an input through it. Flipping the
  admission first would ship a shape whose deciding artifact does not exist — which is verbatim the
  failure slice 6 opened with (*"a design whose deciding artifact does not exist is a design
  believed, not measured"*), one level further along. ⇒ **slice 8 = the emitted guard PARSES** (an
  opt-in admission widener that reaches the generation path, then the `g7` inputs run against a
  parser generated from PGEN's own emission — the ratchet's first leg), **slice 9 = the flip.**
  ⛔ **DECLINES LOUDLY, three ways**, rather than emitting a partial guard: a site whose loop or seed
  verdict BLOCKS refuses the whole plan; a chain member with no bare arm into the chain refuses; and
  a bare arm into the chain whose target the construction order has not reached — a cyclic
  transparency relation — refuses by name instead of leaving an unguarded path to the same
  starvation. The second pass over `guarded_by_source` closes the related hole where a chain member
  is ALSO a starvation holder.
  ⛔ **And the chain key ends in a SERIALIZATION of the residual, not a rendering.**
  `render_elements` is paren-free by contract (it is frozen for the ambiguity refusal), so `a b*`
  renders both `Sequence[a, Quantified{b}]` and `Quantified{Sequence[a, b]}`. Two sites colliding
  there would share one chain — and a chain carries ONE lookahead, so the second site would be
  guarded against a follow restriction that is not its own, silently. Not observed on any shipped
  grammar; refused by construction rather than by absence. The readable fields still lead the key,
  so variant numbering sorts by something a report reader can follow.
- [x] **ADDRESSED (verified)** — `cargo test --lib indirect_lr` → **28 passed / 0 failed** (24 before).
  ⭐ **Falsifiability PROVEN on FIVE independent breaks, each failing a DIFFERENT assertion by
  name** ([[a-check-whose-inputs-all-pass-has-not-been-tested]]) — and the source was restored from a
  CONTENT snapshot and re-hashed after each, which is `.17` slice 6b's own recorded mistake applied:
  1. **guard on the shared rule** (skip the hop clones) ⇒
     `the_guarded_clone_chain_reproduces_the_hand_written_g7_shape` FAILS at the chain-top assertion,
     `left: "ct" right: "prim"` — the plan for `prim` is refused outright and the driver falls back;
  2. **drop the trailing guard** (ship the loop guard alone — slice 4's decision (c) verbatim) ⇒ the
     same test FAILS at a different line, `left: ["prim_lr_base prim_lr_guard0_suffix*"]` vs
     `right: [… &"'" "(" lit ")"]`;
  3. **widen the shipped admission** to `guard_admissible_candidates()` — the exact edit slice 9 will
     make deliberately ⇒ `the_shipped_admission_synthesizes_no_guard_on_the_same_starved_knot` FAILS
     printing the whole `SynthesizedGuard` it emitted;
  4. **follow only the FIRST transparent arm** in `guard_chain_from` ⇒
     `a_branching_transparent_chain_clones_every_arm_and_guard_hops_undercounts_them` FAILS with
     `left: ["ct","pa","prim"]` vs `right: ["ct","pa","pb","prim"]` — the single-arm chain that
     would leave an unguarded route to the same starvation.
  Restored hashes: `indirect_lr_elimination.rs`
  `a98f4aa8c836500f5a0e790570649f8be553b5b67d67f903d859ecc452e13c0a`, `indirect_lr_plan.rs`
  `3f9eaa28fd31f6fb24357c6f144ff60340030b7a91f3a519a8bb2deb2b7bb02f`, suite re-run green at both.
  ⭐⭐ **A FOURTH plant was run against the BANK rather than the unit tests, and it FOUND A DEFECT
  (RESULT 4)** — break 2 rebuilt through the CLI left `GUARD-DRY-RUN: 14/14` green while the emitted
  grammar had lost a lookahead. After the read-back fix the same plant flips D10 by name
  (`property_expr_lr_guard0[loop]`, `MISMATCH (14 case(s) checked)`); source restored to
  `1db24f0b52045290b10008670f974c032a4ceda610417f84577a083c40008a92`, suite and bank green.
  ⛔ That plant is why the byte-identity measurement below was RE-RUN at the final source state
  rather than inherited from the earlier one — the fix landed after the first regeneration, and a
  no-regression claim taken at a different source state is not a measurement of what ships.
  `guard_dry_run/probe.sh` → **`GUARD-DRY-RUN: 16/16 as declared`** (9 before; the headline is now
  DERIVED from the cases that ran, `.17` slice 6b's class applied to the one bank it had not
  reached). `guard_feasibility/probe.sh` → `18/18`, unmoved — its `[guard=` / `~ hops=` anchors
  survive the new `chain=` field in the same bracket group, which is what re-running it checks.
  ⛔ **The full `cargo test --lib --features "generated_parsers ebnf_dual_run"` suite was STARTED and
  DELIBERATELY ABANDONED after ~50 minutes**, and saying so is the point. What stands in its place is
  stronger for this change: the generated tree is byte-identical, so no parser input can behave
  differently, and the bare `cargo test --lib` run is **955 passed / 9 failed** where all nine
  failures are one build-configuration refusal (*"needs the generated annotation backend, but this
  binary was built WITHOUT `--features generated_parsers`"*) raised by annotation-transform tests this
  diff does not touch. ⇒ the featured suite would have re-measured what the byte-identity already
  proves, at a cost that had already exceeded the whole rest of the slice.
- [x] **NO REGRESSION** — ⭐ **measured at the strongest available tier, not argued from the diff.**
  `make -C rust regenerate_generated_parsers` under the memory guard (`exit=0 peak_tree_rss=7474MB
  elapsed=337s`) re-derived all **11** generated parsers with the new binary and
  `shasum -a 256 generated/*.rs` is byte-identical to the pre-change snapshot:

  ```text
  7ce6578f799c36c79343f98c1e441feb70b453eee25be860ae64824f751938e5  generated/ebnf.rs
  829056dfabc5346cce2c0306d436f58818df57fb39c07d5e7b4adcb500ac43e9  generated/json_parser.rs
  eefd327d8db0d8c0ea3491d28715c65456e8467cd1103fb2a196c5aa4d5f38c5  generated/regex_parser.rs
  c1e48f5e1ab5457b9154dbaea3ff4292c05c86390add192a472505ff942bee52  generated/return_annotation_parser.rs
  b59ef442d69f70bbfe18f7a796779fada281eae29e513503d7339a43679ade63  generated/rtl_const_expr_parser.rs
  67bc01cd8d6466aaf40e025d85be6333f8a788177ce99a0e803051e1d95719cd  generated/rtl_frontend_parser.rs
  c339a24075ffa1f02e35ea6d6407b22ce497643a0eb3e10593cba0e558b4e49a  generated/scratch_parser.rs
  e9c132b709a2e72d7d19312752e63857cc2259464a2c6fe403584794c4934a3e  generated/semantic_annotation_parser.rs
  4330ff8e14c8511865cfd5eeb0ab3eabe323cba127c6713e86d7654a7ca970bd  generated/systemverilog_parser.rs
  f46b0d29c328e0ebdbf07e30af6cf2b2a518cfaf72e29be2877fc84aa10229c3  generated/systemverilog_preprocessor_parser.rs
  a90ae37b74131c4dd73ab7b663e6c01f92479aaa1342613c12c307716686b3e3  generated/vhdl_parser.rs
  ```

  ⇒ no parser input can behave differently, because no parser byte moved. The shipped counter agrees
  independently: `indirect_guard_chains=0` on `ebnf`, `systemverilog`, `vhdl` and `regex`.
  ⛔ **Run THREE times, and only the last one counts** — the first preceded the RESULT-4 read-back
  fix and the second preceded RESULT 5's `chain=` field, so each measured a source state that is not
  the one being committed. Verified rather than assumed: every `rust/src` file in the diff has an
  mtime STRICTLY BEFORE the final run's start (`09:56:32`/`09:57:36`/`09:58:13`/`09:03:55` against a
  `10:02:04` start). A no-regression claim inherited across an edit is not a measurement.
  `guard_dry_run/probe.sh` (extended, D9-D13b) and `guard_feasibility/probe.sh` both green.
  ⛔ The scratch-slot banks (`indirect_lr`, `quantifier_policy`, `guard_effectiveness`) were
  deliberately NOT run: they exercise PARSE behaviour, which the byte-identical generated tree
  proves unchanged, and running them would acquire `.13` slice 4b's scratch-slot restore obligation
  for no measurement. Stated rather than silent.
- [x] **LOCKSTEP** — this leaf (slice 6's obligation 3 corrected in place, since it SPECIFIES);
  `TOOLBOX.md` §5.5 (the `variants` bullet, the dry-run bullet, the `indirect_guard_chains=` counter,
  the bank's now-derived headline); the book's grammar-wellformedness chapter (the `🛡` output, the
  hoisted-suffix explanation, the `variants` lower bound, and the *"emits no guard"* paragraph, which
  is now false); `guard_dry_run/probe.sh` + its README; `docs/tasks/CI-PARITY-GATE-ROT.md` (`.29`
  gains a third measured instance and a new acceptance (d)); `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  `MEMORY.md`, `docs/TASK_TREE.md`. No user-visible parser behaviour changed, so no contract or
  schema edit is owed.
  ⭐ `promotion: PROMOTED` — [[a-report-must-be-computed-from-the-artifact-it-describes]], plus a
  `KNOWLEDGE_MAP.md` regeneration. ⛔ Deliberately a NEW card rather than an extension of
  [[a-report-scraper-must-anchor-on-structure-not-on-a-substring]], and slice 7 is itself the
  evidence for the split: that card exists, was written one day earlier by this same leaf, and did
  not prevent this — because the extraction here was CORRECT and the defect was upstream of it. Two
  different questions ("did my grep match the right rows?" vs "was the value ever about the
  artifact?") retrieve on different keys.

##### ⛔ `.17` SLICE 7b (`PGEN-ENGINE-UNIVERSAL-SERVICES-0029`, 2026-08-14 session #230) — asked whether slice 7's three findings were signoff-grade, the audit found TWO were not, and one of them was a claim about a commit that does not exist

> **DOCS + one tracked patch artifact. ZERO grammar bytes, ZERO Rust bytes, ZERO codegen bytes, ZERO
> generated artifacts.** Prompted by a director check on the three findings slice 7 surfaced, answered
> by re-auditing rather than by asserting — the same prompt and the same posture as slice 6b, which is
> the precedent for doing this at all.

**GAP A (the false one) — `CI-PARITY-GATE-ROT.29` acceptance (d) cited a RED/GREEN pair that is NOT
IN GIT.** Slice 7 wrote: *"the calibration pair is `GuardChain::summary` before and after slice 7 …
which is a real RED/GREEN pair in git rather than a constructed one."* Measured:
`git log -S "self.loop_guard, self.trailing_guard" -- rust/src/ast_pipeline/indirect_lr_elimination.rs`
returns **nothing**. The plan-derived version was found and fixed INSIDE slice 7, so it never reached
a commit — an implementer following that criterion would have hunted for a diff that does not exist.

⛔ **Fixed by MAKING IT TRUE, not by softening the sentence.** The pre-fix state is reconstructed and
shipped as a tracked artifact —
`docs/tasks/artifacts/engine_universal_services/guard_dry_run/plan_derived_summary.patch`, verified
with `git apply --check` — and the 2×2 is re-measured from it:

| summary source | + emission plant | `guard_dry_run/probe.sh` |
|---|---|---|
| plan-derived (the patch) | yes | **`16/16` GREEN** ⛔ reports `[loop+trailing]` for a rule with no trailing lookahead |
| tree-derived (HEAD) | yes | **`MISMATCH`** ✅ D10 flips by name |

⭐⭐ **And the gap generalizes, which is the more valuable half.** A defect found and fixed within one
slice leaves **no reproducer behind**: the acceptance box records that it happened, and the artifact a
future gate would be calibrated against is gone by commit time. Every instance in this family has the
same hole — `.17` slice 5's `157`, slice 6b's two hand-typed totals, slice 7's plan-derived summary.
⇒ **`CI-PARITY-GATE-ROT.29` acceptance (e) NEW**: a slice that finds and fixes an instrument defect
before committing preserves the pre-fix state as a tracked patch, or states why it is not worth
preserving. One `git diff` before the fix.

**GAP B — the FRONTIER named the wrong next slice, and it SPECIFIES.** Slice 7's own FIX box and the
tree index both said *"slice 8 = FLIP the admission"*. But what slice 7 emits has **never been
executed**: `guard_effectiveness` measured a HAND-WRITTEN `g7`, and slice 7 asserts PGEN's emission
matches it **in the gen-AST**. No parser has ever been generated from PGEN's own guarded output.
Flipping first would ship a shape whose deciding artifact does not exist — verbatim the failure
slice 6 opened with, one step further along. ⇒ corrected in the leaf, `docs/TASK_TREE.md` and
`MEMORY.md`: **slice 8 = the emitted guard PARSES** (the ratchet's first leg), **slice 9 = the flip**.

**GAP C — finding 2 was STATED and OWNED BY NOBODY.** *"The hop-clone half is not corpus-exercised"*
appeared in the leaf, the bank README, `TOOLBOX.md` and the tree index — as prose, in four places,
with no obligation anywhere. This repository's rule is that every finding is FIXED and routing decides
WHEN; a risk with no owner is a note. ⇒ promoted to **`.17` acceptance (d)**: no slice may ship a
guard whose hop-clone half is unexercised, and the slice that changes the admission or the candidate
ordering must either exercise a multi-hop chain end-to-end or MEASURE and state that its own picks are
all still `max_hops=0`.

**FINDING 3's first half was already signoff-grade** — the knowledge card, the map regeneration and
`.29`'s routing evidence all landed with slice 7 — and finding 1's *decision* was too; what was wrong
in each case was a downstream pointer, not the judgement. ⇒ **two of three needed work, and neither
was visible from the decision itself.**

###### Acceptance Checklist (enforced) — `.17` slice 7b

- [x] **REPRODUCE / ISSUE** — `git log -S "self.loop_guard, self.trailing_guard" -- rust/src/ast_pipeline/indirect_lr_elimination.rs`
  at `eb75df0b` returns EMPTY, against a criterion committed one commit earlier asserting the pair is
  *"a real RED/GREEN pair in git"*. And `grep -n "slice 8" docs/tasks/ENGINE-UNIVERSAL-SERVICES.md`
  returned the flip as the next slice, three screens from this leaf's own statement that nothing has
  parsed the emitted shape.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `docs/tasks/CI-PARITY-GATE-ROT.md` acceptance (d), this
  leaf's slice-7 FIX box, `docs/TASK_TREE.md` line 99, `MEMORY.md` `next_action`. WHY (A): a defect
  fixed inside the slice that found it never becomes a commit, so *"before and after"* has only one
  side in history — and the sentence was written from the author's memory of the session rather than
  from `git log`. WHY (B): the frontier was written when *"emit"* and *"flip"* looked like the whole
  remaining ladder; the missing rung — *execute what you emitted* — was named in the same slice's
  honest-bound paragraph and never propagated to the pointer. WHY (C): a finding stated in four
  narrating surfaces reads as tracked, and none of the four is an obligation.
- [x] **FIX** — fix-hierarchy tier = **documentation correctness + a tracked reproducer artifact**.
  (A) the pre-fix state reconstructed, captured, headed with its measured 2×2 and the exact commands,
  and `git apply --check`-verified; the criterion rewritten to cite it and to say plainly that the
  version was never committed. (B) the frontier corrected in all four surfaces, with the reason.
  (C) `.17` acceptance (d) NEW. (D) the general class routed as `.29` acceptance (e).
- [x] **ADDRESSED (verified)** — the calibration is re-measured from the TRACKED artifact rather than
  from memory: `git apply` the patch + the emission plant → `GUARD-DRY-RUN: 16/16 as declared` while
  `property_expr_lr_guard0` carries no trailing lookahead; restore to `eb75df0b`'s content
  (`65aa20756662c07e5aa9f6061cce8f904ed68df6a0933490fef810cc66367e6c`) + the SAME plant →
  `GUARD-DRY-RUN: MISMATCH (16 case(s) checked)`, D10 naming `property_expr_lr_guard0[loop]`. Source
  restored to that hash, `cargo test --lib indirect_lr` **28 passed / 0 failed**, bank `16/16`, rc 0.
- [x] **NO REGRESSION** — no `rust/src`, `grammars/` or `generated/` byte differs from `eb75df0b`
  (hash above, `git status` clean before staging). The only added file is a `.patch` under
  `docs/tasks/artifacts/`, which nothing executes. ⛔ No regeneration was run and none is owed —
  stated rather than silent.
- [x] **LOCKSTEP** — this leaf (acceptance (d) NEW, slice 7's FIX box corrected in place since it
  SPECIFIES), `docs/tasks/CI-PARITY-GATE-ROT.md` (`.29` (d) corrected + (e) NEW), `docs/TASK_TREE.md`,
  `MEMORY.md`, `CHANGES.md`. `DEVELOPMENT_NOTES.md` gains the durable lesson;
  `promotion: declined (it is `.29` acceptance (e)'s own subject, and a card that duplicates an open gate criterion degrades retrieval — `.17` slice 6b's precedent)`.

##### ⭐⭐ `.17` SLICE 8 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0030`, 2026-08-14 session #231) — the emitted guard PARSES, and the warning saying it is not the shipped policy was itself invisible

> **ENGINE code + a new probe bank. ZERO grammar bytes, and the generated tree re-derived
> BYTE-IDENTICAL — all 11 parsers, hashes below.**
>
> Slice 7b's correction, verbatim: *"slice 8 = the emitted guard PARSES (the ratchet's first leg),
> slice 9 = the flip."* This slice is that leg. ⛔ It is NOT the flip: the shipped admission is
> untouched, `indirect_guard_chains=0` on every family, and no deliverable parser byte moves.

###### THE GAP, NAMED PRECISELY — two artifacts that agree in a tree and meet nowhere

`guard_effectiveness/g7_guarded_clone_chain.ebnf` is a grammar a **person** wrote, measured on a real
parser (slice 6). `plan_guard_chains` is an **emitter**, checked against that grammar in the
generated AST (slice 7). Both are real; the claim *"the guard works"* rests on the join between them,
and nothing had ever generated a **parser** from PGEN's own guarded output and run bytes through it.
⇒ this slice builds the join: grammars that are still LEFT RECURSIVE, eliminated and guarded by the
real pass, compiled, and fed the same `e1`–`e7` the hand-written `g7` was fed.

###### WHAT WAS BUILT — one opt-in door, and it is deliberately a SECOND door

| # | piece | where |
|---|---|---|
| 1 | the admission reaches CODEGEN | `eliminate_indirect_left_recursion_admitting_guard_feasible` (`indirect_lr_elimination.rs`) |
| 2 | opt-in, per run | `PipelineConfig::indirect_lr_admit_guard_feasible` (default `false`) → `--indirect-lr-admit-guard-feasible` |
| 3 | the source grammars PGEN eliminates ITSELF | `guard_parses/s1_guard_source.ebnf` (two holders) + `s2_holder_only.ebnf` (one) — PRE-rewrite |
| 4 | the four-arm bank | `guard_parses/probe.sh` → **`GUARD-PARSES: 64/64 as declared`** |

⛔ **A second entry point, not a widened first one.** `eliminate_indirect_left_recursion` still names
the shipped policy at its own call site, so widening what ships is an edit to *that function's*
criterion (slice 9) and cannot happen by a caller passing a different argument.
⛔ **And no `make` target reaches it** — a property of the call graph, checkable by grep rather than
by reading a default: `grep -rn "indirect.lr.admit.guard.feasible" rust/Makefile scripts/ .github/`
returns **0**, and `git grep -l` names only the three source files that define it plus the bank that
is its only caller.

###### ⭐⭐ RESULT 1 — IT PARSES, and six rows FLIP

```text
  ARM  IN   GEN      WANT     INTERP   WANT
  1A   e2   REJECT   REJECT   ACCEPT   ACCEPT   ✅ DIVERGE(.14)  the runtime cycle guard refuses the chained cast
  1B   e7   ACCEPT   ACCEPT   ACCEPT   ACCEPT   ✅  ⭐⭐ THE CALL-SITE-SCOPING ROW: the residual-free holder is UNHARMED
  2B   e1   ACCEPT   ACCEPT   ACCEPT   ACCEPT   ✅  ⭐⭐ THE LOOP-GUARD ROW
  2B   e5   ACCEPT   ACCEPT   ACCEPT   ACCEPT   ✅  ⭐⭐ THE TRAILING-GUARD ROW
GEN flips A REJECT -> B ACCEPT: 6   ·   declared oracle divergences (.14): 6
```

Every input ACCEPTs on arm B of both grammars, on real generated parsers built from PGEN's own
guarded emission — including the over-long SEED (which the loop guard provably cannot reach) and
`k = n;` (the residual-free holder, which the shared-rule shape `g4` REJECTS). ⭐ **One design doing
both is its whole claim**, and until this bank it had only ever been shown on grammars a person wrote.
The six `e2`/`e4`/`e6` flips are `.13`'s founding defect closed end to end, and `probe.sh` FAILS if
the flip count reaches zero — a bank where both arms accept everything is describing a grammar rather
than measuring an admission.

###### ⛔⛔ RESULT 2 — THE FIRST PLANT DID NOT FALSIFY, AND THAT IS THIS SLICE'S MOST VALUABLE FINDING

The bank's first shape was ONE grammar — `s1`, modelled on `g7` — and it went green on the first
honest run. Then the falsifiability plant: **delete the trailing-lookahead emission from
`apply_plan`, rebuild, re-run the whole bank.** Result:

```text
==================== PLANT 1 ====================
bank rc=0
GUARD-PARSES: 35/35 as declared
```

**Green. A plant that cannot fail is a row that cannot check.** Root cause, and it is in the GRAMMAR,
not the engine: `s1`'s entry rule has two alternatives, and the second one —
`scratch := … | kw_k eq prim semi`, the residual-FREE holder — parses `e1`–`e6` **on its own**
through the eliminated-but-unguarded `prim`. So on `s1` those six rows pass under both hypotheses.

⭐⭐ **`.17` slice 4 had already recorded exactly this, one ladder down**, and neither `g7` nor `s1`
inherited it. `guard_effectiveness/probe.sh` runs `g4` and `g5` on `e1` and `e7` ONLY, with the reason
written beside them: *"their second alternative (`kw_k eq prim semi`) absorbs `e2`–`e6` on its own, so
those rows would pass under BOTH hypotheses and prove nothing about scoping. A case that does not
discriminate is left out rather than run and over-read."*

⛔⛔ **⇒ A CORRECTION TO SLICE 6, ROUTED AND FIXED IN PLACE.** `g7` has `g4`'s two holders and slice 6
ran all seven of its rows, so *"it accepts all six starvation inputs AND `e7`"* — the sentence in the
leaf, the bank README and `TOOLBOX.md` — is **six rows too strong**. On `g7` those six prove the guard
does NO DAMAGE; the POWER claim rests on the single-holder rungs `g0`/`g2`/`g6`/`g3`, which slice 4
built correctly. ⛔ **No expectation anywhere was changed and none should be** — every verdict in that
bank is still right. What was wrong was the strength claimed for six of them, and it is corrected in
all three surfaces.

⇒ **the fix is the discriminating control the ladder never had**: `s2_holder_only.ebnf`, `s1` minus
the rescuing alternative, nothing else changed. Its `e1`–`e6` have exactly one route to a parse, so
each tests a guard POSITION — and the bank now refuses to pass if the `s2` arm did not run.

###### ⭐⭐ RESULT 3 — with `s2` in place, all three guard properties are PLANT-PROVEN on a real parser

| plant | one-line edit | rows that FAIL, and ONLY these |
|---|---|---|
| drop the TRAILING guard | delete `guarded_elements.push(lookahead())` | `2B e5` |
| drop the LOOP guard | `guarded_suffix_rule = None` | `2B e1 e2 e3 e4 e6` + both `guard_rule_count` |
| guard the SHARED rule (`g4`) | `guarded_base_rule = base_rule` | `1B e7` on BOTH oracles + both `guard_rule_count` |

⭐ The third is the strongest single row in this slice: **the call-site-scoping decision, which `.17`
has carried since slice 4 as an argument from two hand-written grammars, now fails a real generated
parser by name when it is violated.**

⭐⭐ **AND THE FIRST TWO PLANTS PARTITION THE ROWS — which is slice 4's disjointness result reproduced
END TO END, through PGEN's own emitter.** Dropping the trailing guard breaks `2B e5` and *nothing
else*; dropping the loop guard breaks *everything except* `2B e5`. Slice 4 measured that
complementarity by hand-writing `g2` (loop only) and `g6` (trailing only) and reading their rows; here
it falls out of two one-line edits to the EMITTER, with no hand-written grammar in the loop. ⇒ *"the
two positions close DISJOINT starvations"* is no longer a property of two synthetics — it is a
property of what PGEN generates.

⛔ **The structural rows discriminate too, and the reason is worth reading**: the `g4` plant and the
loop plant both take `guard_rule_count` 3 → 2, for different reasons (the guarded base stops being a
separate rule; the guarded suffix is never allocated). A row that only ever reads `0`/non-`0` would
have missed both.

###### ⭐⭐ RESULT 4 — the HOP CLONE is exercised END TO END, which is acceptance (d)'s standing obligation

Both source grammars' holders name `ct`, **not** `prim`, so the transparent chain is two rules long
(`chain: ct > prim`, `max_hops=1`) and PGEN must emit `prim_lr_guard0_ct`. Measured in the compiled
artifact, not in the plan:

```text
  2B   guard_rule_count   3      3      ✅  prim_lr_guard0 + its suffix + the HOP CLONE
  2B   hop_clone          1      1      ✅  prim_lr_guard0_ct EXISTS in the emitted parser
```

⭐ And `1B e7` is the input that proves the clone is doing its job rather than merely existing:
`k = n;` reaches `prim` through the **unguarded original**, which is the entire call-site-scoping
argument. ⛔ Acceptance (d) is therefore discharged **for this slice**; it stays OPEN as an obligation
on slice 9, which must exercise a multi-hop chain on the real grammar or measure and state that its
own picks are still all `max_hops=0`.

###### ⛔⛔ RESULT 5 — THE OPT-IN WARNING WAS INVISIBLE, AND ONLY RUNNING IT SHOWED THAT

The first draft of the banner used `eprintln!`. A default-verbosity run of
`--indirect-lr-admit-guard-feasible` was then **completely silent** while absorbing candidates the
shipped criterion refuses as STARVED. Root cause, located rather than guessed: `ast_pipeline/mod.rs`
declares

```rust
macro_rules! eprintln { ($($arg:tt)*) => { crate::pgen_trace_debug!($($arg)*) }; }   // :513
```

and `pub mod indirect_lr_elimination;` (`:6277`) is declared **after** it, so every bare `eprintln!`
in that file — including the pre-existing `✅ Absorbing` / `⏭️ Declining` lines — is a trace call
gated on `PGEN_TRACE_VERBOSITY=debug`. ⇒ fixed with `std::eprintln!`, and the fully-qualified path is
LOAD-BEARING; the bank pins the fix in **both directions** (`A banner 0`, `B banner 1`).

⭐ **The distinction the fix draws, because "make it all loud" would be wrong.** Per-rewrite
narration is legitimately trace-gated — it is one line per absorbed rule on grammars with thousands.
A **policy warning** is not: it says *this run is not the shipped behaviour*, and a warning that only
fires when you already asked for debug output is not a warning. The shadow is left in place for the
former and bypassed for the latter.

###### ⛔⛔ RESULT 6 — THE ADMISSION ENUM'S OWN DOC HAD BEEN FALSE SINCE SLICE 7

`CandidateAdmission::GuardFeasibleDryRun`'s doc read: *"⛔ **This admits them WITHOUT emitting any
guard**, so the grammar it produces is the measured-regressing one."* True when slice 3 wrote it.
**Falsified by slice 7's own emitter one slice later** — `plan_elimination` step 6 runs
`plan_guard_chains` unconditionally, and a guard exists exactly for a surviving starvation site,
which only a guard-feasible candidate has. Never swept.

⛔ The reader it would have misled is precisely this slice's: someone deciding whether that admission
is safe to generate a parser from. ⇒ corrected, and the variant **renamed** `GuardFeasible` — the
`DryRun` suffix was a third false claim, since slice 8 takes the same admission to codegen.

⭐ **The name was fusing three independent questions** — *which candidates are admitted*, *does the
result reach a parser*, *does the pass narrate* — and slice 8 needs the combination the fused name
declares impossible (guard-feasible **and** reaching codegen **and** loud). Narration is now the
driver's own `Narration` parameter, decided per entry point on the honest criterion: **a pass whose
output someone keeps says what it did; a pass whose output is thrown away stays quiet.** The dry run
keeps its silence, for its own reason rather than by sharing a name.

###### ⛔ RESULT 7 — THIS SLICE'S OWN INSTRUMENT BROKE TWICE, IN TWO DIFFERENT LAYERS

`guard_rule_count` / `eliminated` / `hop_clone` first read `^\s*fn parse_…`. Every rule function the
codegen emits is `pub fn parse_<rule>(`, so the anchors matched **nothing** and the first bank run
reported `guard_rule_count=0` on the arm that has three of them.

⭐ **Caught in one run, and by design rather than by luck**: the row DECLARES `3`. That is the whole
reason the bank carries structural expectations instead of printing what it found — a scraper that
prints its own finding is unfalsifiable, and this one would have printed a confident `0`.
⇒ **the FOURTH measured instance of [[a-report-scraper-must-anchor-on-structure-not-on-a-substring]]
in this leaf** (after `.17` slice 5's `157`, slice 6b's two hand-typed totals, slice 7's plan-derived
summary). Routed to `CI-PARITY-GATE-ROT.29` as a fifth data point. ⛔ The same run also carried a
`grep -c … || echo 0` fallback that printed a SECOND zero line — `grep -c` prints its own zero AND
exits 1 — so every comparison ran against a two-line value. Both fixed, both explained at the anchor.

⛔⛔ **AND A SECOND, IN A LAYER NOBODY AUDITS: A BACKTICK INSIDE A ROW'S NOTE IS A COMMAND.** The
tables are double-quoted shell strings, so the note *"the `` `*` `` must stop at zero iterations"*
made bash run the command `*`, which globbed to the repository root's first file:

```text
probe.sh: line 143: AGENTS.md: command not found
```

⭐ **It failed loudly only because the substitution happened to be nonsense.** `$HOME` or `$(date)`
would have substituted silently and left a table whose printed notes are not the notes anyone wrote —
a bank narrating something no one authored, which is this leaf's recurring defect in yet another
layer. ⇒ fixed, and **made unable to recur**: `probe.sh` now reads its OWN source, greps the
`CASES=(`/`STRUCT=(` block for a backtick or a `$` expansion, and exits 2 naming the row — before any
of the fifteen minutes of builds. ⛔ Deliberately reading the FILE and not the arrays: by the time an
array element exists the substitution has already happened and the evidence is gone. RED-proven by
planting one backtick (`rc=2`, the row named) and restoring the script to a byte-identical hash.

⭐ **And the check's own first version was too broad, which is worth one sentence because the failure
mode is generic**: it refused to start over a backtick in its OWN explanatory comment inside the array
block. Comment lines there are real shell comments and are never substituted, so they are exempt —
measured, not assumed. **A self-check that cannot describe itself is one people delete.** It cost 50
seconds to find, because the check runs before the fifteen minutes of builds rather than after.

###### ⛔⛔ ROUTED OUT — `.14` DIVERGES IN BOTH DIRECTIONS, AND WHICH ONE IS A PROPERTY OF THE SURROUNDING GRAMMAR

`.14`'s title says *"in BOTH directions"*. This bank reproduces both **in one run, on two grammars
that differ by one line** — which nothing in the repository had:

| | `1A` (two holders) | `2A` (one holder) |
|---|---|---|
| `e2` `k = n'(n)'(n);` | GEN **REJECT** · INTERP ACCEPT | both REJECT |
| `e1` `k = n'(n);` | both ACCEPT | GEN **ACCEPT** · INTERP **REJECT** |

⭐ **One mechanism, and it explains the flip.** The interpreter has no cycle guard, only a whole-stack
depth ceiling (`parse_harness_interpreter.rs:746-749`), and under the `longest_match` default it must
EVALUATE the cyclic alternative in order to compare it. On S1 the surviving second entry alternative
rescues the parse after that evaluation blows the ceiling; S2 has nothing to rescue it. ⇒ **the
direction `.14` bites in is a property of the surrounding grammar, not of the cycle** — a fact that
leaf records in its title and has never had a fixture for.

⛔ **Not fixed here** (`.14` is a parked leaf and the lane lock binds work, not conversation); routed
INTO `.14` with this bank named as its repro. ⭐ Better than the existing `p1_knot_a_defect` fixture
in two specific ways: it carries BOTH directions as a one-difference pair, and the same file's B arm
shows the divergence VANISH once the cycle is eliminated — the control `.14` acceptance (a) needs to
prove its case is about a *surviving* cycle rather than about the grammar.

###### ⛔ AND THE ONE PLACE THIS BANK'S EXPECTATIONS WERE CORRECTED — recorded, not quietly edited

Arm `2A`'s INTERP column was first written `ACCEPT`×6 **by extrapolation from `1A`**, where the
interpreter does accept. Measured: `REJECT`×6. The extrapolation was the error — S2 drops exactly the
alternative that rescues S1's interpreter parse — and the corrected cells are the divergence pin
above rather than a design claim.

⛔ **Stated because "I adjusted an expectation" is the sentence this repository refuses to let pass
silently.** Two things make it legitimate here and both are checkable: (1) nothing in the design says
what a cycle-guardless interpreter does at its depth ceiling, so those six cells were never a design
claim to begin with — every design claim in the bank lives in the **GEN** column; and (2) **no GEN
cell was ever wrong** — all 26 were written before the first run and measured correct. ⇒ the rule
that was broken is not *"don't adjust expectations"* but *"derive an expectation from ground truth,
never from a neighbouring measurement"*, and the bank's header now says so at the rows themselves.

###### Acceptance Checklist (enforced) — `.17` slice 8

- [x] **REPRODUCE / ISSUE** — at `c66c7a32`, nothing in PGEN could execute what it emits. Reproduced
  from the shipped surface: `--report-indirect-lr-plan --indirect-lr-plan-guard-dry-run` on the new
  source grammar printed `guard_chains=1 guard_rules=3` under a banner reading *"it parses nothing,
  so it is NOT a claim that the rewritten grammar accepts or rejects any input"*, and
  `git grep -l "indirect_lr_admit"` returned **nothing** — there was no path from that plan to a
  parser.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the emission could not be executed: the guard-feasible
  admission existed only inside `dry_run_guard_feasible_elimination`, which applies the plan to a
  CLONE it discards (`indirect_lr_elimination.rs`), so no `PipelineConfig` and therefore no codegen
  run could ever see a guarded tree. WHERE it had to attach: `PipelineConfig` →
  `eliminate_left_recursive_patterns`'s indirect branch (`ast_pipeline/mod.rs:3046`), the one place
  the generation path enters the pass. ⛔ Tool-located, not read off the source: the first
  default-verbosity run of the new flag emitted **no banner and no `✅ Absorbing` line**, and
  `PGEN_TRACE_VERBOSITY=debug` on the same command printed both — naming the trace shadow at
  `ast_pipeline/mod.rs:513` as the reason (RESULT 5). ⛔ And a `parseability_probe --parse scratch`
  run against an ARM-A parser reproduces the symptom the flip closes: `e2` REJECTs while `e1`
  accepts, i.e. the runtime cycle guard, not a grammar error.
- [x] **FIX** — fix-hierarchy tier = **engine capability**, plus two instrument-correctness fixes and
  two documentation-correctness fixes found while building it.
  `indirect_lr_elimination`: `eliminate_indirect_left_recursion_admitting_guard_feasible` (the opt-in
  generation entry point, with a `std::eprintln!` banner), `CandidateAdmission::GuardFeasibleDryRun`
  → `GuardFeasible` with its false doc corrected, and the narration axis split out of the enum into a
  `Narration` parameter.
  `ast_pipeline/mod.rs`: `PipelineConfig::indirect_lr_admit_guard_feasible` (default `false`) and the
  two-door branch. `main.rs`: `--indirect-lr-admit-guard-feasible`. `bin/pgen_ast.rs`: the field in
  its exhaustive literal — ⛔ **a compile break this slice shipped and the STRICT SOURCE CLIPPY stage
  caught**, in a binary `cargo build --bin ast_pipeline` never compiles
  (`error[E0063]: missing field indirect_lr_admit_guard_feasible`). ⭐ The lesson is small and
  transferable: **adding a `pub` struct field breaks every exhaustive literal in the workspace, and
  the one you are not building is the one that breaks** — `grep -rn "PipelineConfig {"` names all
  four sites in a second, and `--all-targets` is what makes forgetting non-optional.
  Bank: `guard_parses/{s1_guard_source.ebnf,s2_holder_only.ebnf,probe.sh,README.md}` — `s2` exists
  because a plant proved `s1`'s rows could not check what they appeared to check (RESULT 2), and
  `probe.sh` refuses to pass without it.
  Corrections to slice 6's claim about `g7`'s six accepts, in the leaf, the `guard_effectiveness`
  README and `TOOLBOX.md` — ⛔ **claim strength only; not one expectation in that bank changed.**
  ⛔ Deliberately NOT done: the shipped admission, the candidate ORDERING and every generated parser
  byte are unchanged. Those are slice 9.
  ⛔ The two guard-emission unit tests were moved onto the NEW entry point rather than the private
  driver — the shape they pin is now reachable by codegen, so the test must enter by the door codegen
  enters by.
- [x] **ADDRESSED (verified)** — `cargo test --lib indirect_lr` → **28 passed / 0 failed**, and the
  bank `guard_parses/probe.sh` → **`GUARD-PARSES: 64/64 as declared`**, rc 0, with the before→after on
  the symptom carried by the bank's own arms (**6 GEN rows REJECT → ACCEPT**, three per grammar).
  ⭐⭐ **Falsifiability PROVEN by THREE plants run against the BANK, one per guard property** — not
  against the unit tests, because `.17` slice 7's lesson is that a plant the unit tests catch can
  still leave a bank green. Each fails a DIFFERENT declared row by name; the source was restored from
  a CONTENT snapshot and re-hashed after each (`.17` slice 6b's recorded mistake applied):
  1. **drop the trailing-guard emission** ⇒ `2B e5` REJECT — the over-long SEED is no longer refused;
  2. **guard the SHARED rule instead of a clone** (`guarded_base_rule = base_rule`, the `g4` shape)
     ⇒ `1B e7` REJECT **on both oracles** — `k = n;` breaks, which is the CALL-SITE-SCOPING claim
     planted end to end and the strongest single row in this slice;
  3. **drop the loop-guard emission** (`guarded_suffix_rule = None`) ⇒ `2B e1` / `2B e2` REJECT.
  ⛔⛔ **AND THE FIRST ATTEMPT AT PLANT 1 LEFT THE BANK GREEN**, which is RESULT 2 and is recorded as a
  result rather than as a false start: the plant is what proved the row set could not check the guard,
  and `s2` is what it bought. A fourth plant — one backtick in a row's note — RED-proves the new
  table self-check (`rc=2`, the row named), script restored byte-identically.
  Source restored to `a49e0b5e5a0038b625eefcfa662abf6f7dd49eae8cdc0184893bfea36a27d13c` and the bank
  re-run GREEN at that state; logs in `rust/target/es17_guard_parses/`.
- [x] **NO REGRESSION** — ⭐ measured at the strongest available tier: the generated tree re-derived
  with the new binary and `shasum -a 256 generated/*.rs` **byte-identical** to the pre-change
  snapshot (hashes below) ⇒ no parser input can behave differently, because no parser byte moved.
  The shipped counter agrees independently: `indirect_guard_chains=0`. `git grep` proves the flag is
  unreachable from `rust/Makefile`, `scripts/` and `.github/` (0 hits).
  ⛔⛔ **AND THE DOCUMENTED RECIPE COULD NOT BE USED, WHICH IS A DEFECT THIS SLICE FOUND AND ROUTED —
  not a shortcut taken.** `make -C rust regenerate_generated_parsers` FAILS in 15 s on a warm tree:
  `regex_parser_bootstrap` unconditionally runs `cargo build --features ebnf_dual_run --bin
  ast_pipeline`, which REPLACES `rust/target/debug/ast_pipeline` with a binary missing
  `generated_parsers`, and the `$(RUST_AST_PIPELINE)` rule that declares the full feature set is then
  skipped because make sees the downgraded binary as the newest file. Measured
  (`AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=true generated_parsers=false`, prerequisites at 17:44
  against a binary at 17:52), and it is invisible in CI **by construction**: on a cold clone those
  prerequisites do not exist, so make rebuilds them and the featured rule fires. ⇒ routed as
  **`CI-PARITY-GATE-ROT.30`** with its calibration case (a warm tree — no fixture needed).
  ⇒ this slice ran the SAME sequence minus the downgrading bootstrap step: build the featured binary
  explicitly, verify it with `scripts/require_ast_pipeline_features.sh`, then `annotation_parsers`
  plus every `focus_<family>`. ⛔ Stated rather than silent, because a no-regression box that quietly
  substitutes its own command is exactly the shape this leaf keeps finding.
  The scratch slot is restored AND regenerated by the bank's own trap, which additionally rebuilds
  `parseability_probe` — that binary compiles the scratch parser IN, so a restored artifact beside a
  stale binary is a third inconsistent state no `git status` shows.

  ```text
  7ce6578f799c36c79343f98c1e441feb70b453eee25be860ae64824f751938e5  generated/ebnf.rs
  829056dfabc5346cce2c0306d436f58818df57fb39c07d5e7b4adcb500ac43e9  generated/json_parser.rs
  eefd327d8db0d8c0ea3491d28715c65456e8467cd1103fb2a196c5aa4d5f38c5  generated/regex_parser.rs
  c1e48f5e1ab5457b9154dbaea3ff4292c05c86390add192a472505ff942bee52  generated/return_annotation_parser.rs
  b59ef442d69f70bbfe18f7a796779fada281eae29e513503d7339a43679ade63  generated/rtl_const_expr_parser.rs
  67bc01cd8d6466aaf40e025d85be6333f8a788177ce99a0e803051e1d95719cd  generated/rtl_frontend_parser.rs
  c339a24075ffa1f02e35ea6d6407b22ce497643a0eb3e10593cba0e558b4e49a  generated/scratch_parser.rs
  e9c132b709a2e72d7d19312752e63857cc2259464a2c6fe403584794c4934a3e  generated/semantic_annotation_parser.rs
  4330ff8e14c8511865cfd5eeb0ab3eabe323cba127c6713e86d7654a7ca970bd  generated/systemverilog_parser.rs
  f46b0d29c328e0ebdbf07e30af6cf2b2a518cfaf72e29be2877fc84aa10229c3  generated/systemverilog_preprocessor_parser.rs
  a90ae37b74131c4dd73ab7b663e6c01f92479aaa1342613c12c307716686b3e3  generated/vhdl_parser.rs
  ```

  ⭐ **`generated/scratch_parser.rs` is in that list and matters most here**, because it is the one
  artifact this slice's bank rewrites eight times: it comes back byte-identical to the pre-change
  value, so the restore-and-regenerate trap is verified rather than trusted.
  ⛔ The shipped counter agrees on four families independently:
  `indirect_guard_chains=0` on `ebnf`, `systemverilog`, `vhdl` and `regex`.
- [x] **LOCKSTEP** — this leaf (acceptance (d) discharged for this slice and restated as slice 9's
  obligation; slice 6's `g7` claim corrected in place, since it SPECIFIES what the ladder proves);
  `TOOLBOX.md` §5.5 (the new flag, the bank, the trace-shadow trap, the `g7` correction, and a STALE
  `37/37` for a 44-row bank that this slice replaced with the derived form); the
  `guard_effectiveness` README; the book's grammar-wellformedness chapter (a new
  *"Executing what the planner emits"* section); `docs/tasks/CI-PARITY-GATE-ROT.md` (`.29` gains a
  fourth instance and a cheap partial answer to its acceptance (b)); this file's `.14` leaf (the
  routed reproducer); `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`. No
  user-visible parser behaviour changed, so no contract or schema edit is owed.
  ⭐ `promotion: PROMOTED` — [[a-warning-that-is-trace-gated-is-not-a-warning]], plus a
  `KNOWLEDGE_MAP.md` regeneration. ⛔ Deliberately a new card rather than an extension of
  [[a-report-must-be-computed-from-the-artifact-it-describes]]: that card is about a value being
  *wrong*, this one about a correct message never being *delivered*, and the retrieval key is
  different ("why did nothing print?" vs "is this number about the artifact?").

##### ⭐⭐⭐ `.17` SLICE 9 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0032`, 2026-08-14 session #232) — THE FLIP: the guarded admission is what PGEN ships, and `generated/systemverilog_parser.rs` is the one artifact that moved

> ⛔⛔ **THE SHIPPED-PARSER CHANGE THIS LEAF HAS BEEN BUILDING TOWARD SINCE SLICE 4.** `.13`'s
> founding defect — an LRM-legal SystemVerilog cast that PGEN could not parse — closes on the
> deliverable path, not on a synthetic and not behind a flag. Slice 7b's correction, verbatim:
> *"slice 8 = the emitted guard PARSES (the ratchet's first leg), **slice 9 = the flip**."*
>
> ⛔ Scope, stated as a boundary rather than as a promise: **one criterion changed**, in
> `eliminate_indirect_left_recursion`. No grammar byte, no codegen change, no new capability. Every
> mechanism this slice ships was built by slices 4–8 and measured then; slice 9 is the decision to
> use it.

###### THE ONE-LINE CHANGE, AND WHY IT IS NOT A ONE-LINE SLICE

```rust
-        CandidateAdmission::StarvationSafe,     // 0 of 28 SV candidates admitted
+        CandidateAdmission::GuardFeasible,      // 16 of 28, each surviving site guarded
```

Everything else in the diff exists because that line moved: the opt-in door inverts into an A/B
lever, four banks whose subject is *"the knots the eliminator has NOT absorbed"* have to name which
admission they mean, and the report that describes what a parser contains has to start describing
guards it never used to contain.

###### ⭐⭐ RESULT 1 — THE FLIP'S OWN RED/GREEN PAIR WAS WRITTEN ONE SLICE AHEAD, AND IT FIRED VERBATIM

`.17` slice 7's falsifiability list recorded plant 3 as: *"widen the shipped admission to
`guard_admissible_candidates()` — **the exact edit slice 9 will make deliberately** ⇒
`the_shipped_admission_synthesizes_no_guard_on_the_same_starved_knot` FAILS printing the whole
`SynthesizedGuard` it emitted."* Slice 9 made that edit. Measured, before touching the test:

```text
test …::the_shipped_admission_synthesizes_no_guard_on_the_same_starved_knot ... FAILED
  the shipped admission admits only starvation-SAFE candidates … — it emitted
  [SynthesizedGuard { base_rule: "prim", guarded_base_rule: "prim_lr_guard0",
    chain: ["ct", "prim"], positions: "loop+trailing",
    residual: "\"'\" \"(\" lit \")\"", call_sites: ["outer_cast alt#0"], … }]
```

⭐⭐ **That discharges `CI-PARITY-GATE-ROT.29` acceptance (e) with no reconstructed artifact**, and it
is the first instance in this family where that was true: the calibration pair for the flip is
`git show 88b06424` — the pre-fix state IS the parent commit, because the defect this slice closes was
not found *and fixed* inside one slice. ⇒ the acceptance (e) obligation is *"preserve the pre-fix
state, or state why it needs no preserving"*, and this is the second answer.

⛔ The test was **renamed and made two-sided**, not deleted: `the_narrowed_admission_synthesizes_no_guard_on_the_knot_the_shipped_one_guards`
asserts both halves on one fixture. A control that only said *"the narrow door emits nothing"* would
pass just as well on a repository where the flip was reverted and both doors were narrow — which is
exactly the failure a rename invites.

###### ⭐⭐ RESULT 2 — MEASURED ON ONE BINARY, VIA A LEVER THAT NOW NARROWS

The opt-in widener `--indirect-lr-admit-guard-feasible` becomes the opt-in narrower
`--indirect-lr-admit-starvation-safe-only`. ⛔ **The structural invariant slice 8 established is
preserved in both directions**: the shipped call site names the shipped policy, so changing what
ships is an edit to the function's own criterion and can never happen by a caller passing an
argument. What inverted is which branch carries a flag.

```text
grammar                        NARROW (pre-flip)               SHIPPED (flipped)
systemverilog                  left_recursion_unhandled=28     left_recursion_unhandled=0
vhdl / ebnf / regex / json      …=0                            …=0
rtl_frontend / rtl_const_expr   …=0                            …=0
systemverilog_preprocessor      …=0                            …=0
semantic_annotation / return_…  …=0                            …=0
```

⛔ **Every other SystemVerilog lint counter is IDENTICAL across the two arms** — `non_terminating=0`,
`ordered_choice_shadowing=0`, `always_succeeds_alternatives=6`, `unreachable_rules=0`,
`undefined_references=0`, `unbound_fact_kinds=0`, `nullable_repetition=0`, `profile_orphans=0`. Rule
count 1488 → 1608. The shipped report absorbs at three rules (`casting_type`, `property_expr`,
`incomplete_class_scoped_type_sv_2023` — the last was already absorbed pre-flip) with
`indirect_clone_rules=24 indirect_guard_chains=3`.

###### ⭐⭐ RESULT 3 — WHAT THE DRY RUN PREDICTED IS WHAT THE PASS DID, RULE FOR RULE

`guard_dry_run/probe.sh` has predicted `would_absorb=2` at `casting_type` and `property_expr`,
`clone_rules=24`, `guard_chains=3 guard_rules=6`, `left_recursive_rule_rows 28 -> 0` since slice 3.
The shipped pass produced exactly that. ⭐ **Every declared value in that bank and in
`guard_feasibility/probe.sh` is byte-identical after the flip** — 18/18 and 18/18 — once each is told
which admission its census is about. That invariance is the finding: the dry run models the driver
rather than approximating it, and this is the first time the two could be compared on the real
grammar.

###### ⛔⛔ RESULT 4 — FOUR BANKS MEASURE "THE SHIPPED GRAMMAR", AND THE FLIP MOVED WHAT THAT MEANS

A census **of the knots the eliminator has not absorbed** is empty once the knots are absorbed. Both
`guard_feasibility` (C1 `0/0 / 0/0`, C7 `0/0`, C8a `=0` …) and `guard_dry_run` (D1 `0/0`, D13 `0`)
went RED on the first post-flip run, and **every one of those reds would have read as good news**:
`0/28 → 0/0` looks like "no starved candidates", `129/129 → 0/0` looks like "no over-approximated
guards", `would_absorb=2 → 0` looks like "nothing left to do". All three are the population having
moved, not shrunk.

⇒ each bank now names the admission its subject belongs to (`--indirect-lr-admit-starvation-safe-only`
for the census rows, the bare path for the shipped rows), with the reason written at the variable.
⭐ **The precedent was already inside one of them**: `guard_feasibility` C10 has always passed
`--no-eliminate-indirect-left-recursion` to see `ebnf`'s knot before the pass eats it, with that
reason beside it. The generalization is the finding —

> **a bank pinned to "the shipped X" is pinned to a moving target, and it fails in the direction that
> reads as success.**

⇒ routed to `CI-PARITY-GATE-ROT.29` as a sixth instance and its first *non-scraper* one: nothing here
mis-extracted a value; the extraction was correct and the SUBJECT changed underneath it.

###### ⛔⛔ RESULT 5 — THE FLIP CREATED A REPORTING GAP AND THIS SLICE CLOSED IT

Post-flip the ordinary report printed `indirect_guard_chains=3` and named **none of them**. The `🛡`
detail renderer lived only in the opt-in dry-run block, because until this slice the shipped count was
`0` by construction and there was nothing to render. ⇒ an operator asking *"what did the engine put in
my parser?"* would have received a count.

Fixed with ONE renderer shared by both blocks — two spellings of one block is two things that have to
agree — distinguished by a **verb** that is load-bearing rather than cosmetic: the shipped block writes
`🛡 guard '…'`, the dry run writes `🛡 would guard '…'`. Both print in one report, so a scraper can
tell a fact from a prediction without tracking which section it is inside. ⛔ `guard_dry_run`'s
`chains_of` and D12 were re-anchored onto the dry-run verb in the same edit; leaving them on the bare
marker would have summed two populations, which is verbatim what D5b's first draft did.

###### ⭐⭐ RESULT 6 — `.17` ACCEPTANCE (d), DISCHARGED BY MEASUREMENT AND MECHANISED

The obligation on the flipping slice: *"either exercise a multi-hop chain end to end, or MEASURE and
state that its own picks are all `max_hops=0` — silence is not an option."* Measured, and the
instrument now computes it rather than leaving it to be counted off a joined chain by eye:

```text
systemverilog                  indirect_guard_chains=3   max_hops set: max_hops=0
vhdl / ebnf / regex / json / rtl pair / sv preprocessor   indirect_guard_chains=0   (no chains)
```

⇒ **all three shipped picks are zero-hop**, so `X_lr_guard{v}_<hop>` is on no shipped path and its
end-to-end coverage remains slice 8's `guard_parses` bank plus the unit tests. Stated, not silent.
⛔ Pinned as bank `guard_dry_run` **D11c**, so the next slice that moves the admission or the
candidate ordering is told by a failing row rather than by remembering to look.

###### ⭐ RESULT 7 — A WIDER ADMISSION IS NOT A WEAKER ONE, and the control is a synthetic that absorbs NOTHING

`p1_knot_a_defect.ebnf` is unannotated by design. Under the flip it gains a **third** refusal — `ct`,
with the same named reason as the other two (*"hop 'cast_expr' alternative 0 declares no return
annotation and its residual is … so the chain's AST cannot be composed faithfully"*) — and still
absorbs nothing. ⇒ the guard admits a candidate to **planning**, not to absorption; every refusal
downstream of the starvation gate still fires.

⭐ That also protects a load-bearing dependency this leaf recorded as honest bound 2:
`parse_harness_combinator_suite`'s `recursion_guarded_memo_isolation` needs P1's cycle to SURVIVE, and
it does.

⭐⭐ **And `p5_transparent_holder.ebnf` — `.17` acceptance (b)'s isolating synthetic — flips from
`eliminated=0` to `eliminated=1 clone_rules=2 guard_chains=1`.** The one shape P1–P4 do not have is
now fixed by the shipped pass.

###### ⭐⭐⭐ RESULT 8 — THE TWO-SIDED RATCHET: TWO DEFECT ROWS FLIP, ZERO CONTROLS BREAK

`stimuli/sv/run_adjudication_repros.py` is the instrument that caught the regression `.13` slice 5
nearly shipped — *"neither the lint, nor 11 unit tests, nor the byte-identity of 10 generated
parsers, nor 18/18 doctrines could see it"*. On the regenerated parser, before any manifest edit:

```text
ADJUDICATION-REPROS: checked=29 armed=7 listed=29 failures=2
  ⛔ defect_constant_size_cast.sv:              expected REJECT, got ACCEPT
  ⛔ defect_constant_size_cast_corpus_shape.sv: expected REJECT, got ACCEPT
```

⭐⭐ **Both failures are DEFECT REPRODUCERS THAT NOW PARSE** — the two rows whose manifest note has
read *"FLIPS TO ACCEPT WHEN FIXED"* since `.13c.2`. ⛔ And the half that matters more: **not one
control moved.** All 12 `control_*` rows, all 5 `fixed` rows and all 9 `invalid_*` over-acceptance
guards hold — including `control_size_cast_in_statement.sv`, the exact row that read
`expect=ACCEPT got=REJECT` on `.13` slice 5's over-eager version and stopped it shipping.

⇒ re-baselined as the runner itself instructs — both rows to `expect=ACCEPT class=fixed`, citing this
work unit — and re-run: **`checked=29 armed=7 listed=29 failures=0`**. ⭐ The ratchet is still
two-sided after the edit, in the other direction: a revert now makes those rows REJECT and FAIL.

###### ⭐⭐ RESULT 9 — AND IT REACHES REAL CORPUS TEXT, NOT ONLY THE REDUCED REPRO

`.17` acceptance (c) names *"the flips, including the two OpenTitan corpus rows"*. Measured on the
files themselves, `parseability_probe --parse systemverilog … --profile sv_2017`:

| corpus file | tracked `results.tsv` (pre-flip) | measured now |
|---|---|---|
| `opentitan/hw/top_darjeeling/rtl/autogen/testing/top_darjeeling_rnd_cnst_pkg.sv` | `fail` | **`parse_full passed`** |
| `opentitan/hw/top_earlgrey/rtl/autogen/testing/top_earlgrey_rnd_cnst_pkg.sv` | `fail` | **`parse_full passed`** |

⭐ The reduced repro and the corpus shape flipped **together**, which is what separates *"the fix
works on my synthetic"* from *"the fix reaches the text that motivated it"*.

###### ⭐⭐⭐ RESULT 10 — THE CORPUS RE-MEASURE: 12 FILES GAINED, ZERO LOST, ACROSS FOUR INDEPENDENT UPSTREAMS

Full re-run of `stimuli/run_external_corpus.sh sv` at the tracked parameters (60 s, 8 jobs, no cap)
against the regenerated parser, written to a sandbox via `PGEN_CORPUS_OUT_DIR` so nothing tracked is
overwritten before it is diffed:

```text
16336 parsed — pass=9774 fail=6562 timeout=0 crash=0   (baseline: pass=9762 fail=6574)
```

⛔ **A net figure can hide offsetting moves, so the load-bearing check is PER FILE.** Joining the two
`results.tsv` on path: **12 transitions, every one `fail → pass`, and ZERO `pass → fail`.**

| upstream | files gained |
|---|---|
| Surelog | 5 — `BlackBePipeInt`, `CastShift`, `ClogCast`, `ParamArraySelect`, `ParamTypespec` |
| black-parrot | 4 — `bp_common_{cache,cfg_bus,clint,host}_pkgdef.svh` |
| opentitan | 2 — `top_{darjeeling,earlgrey}_rnd_cnst_pkg.sv` |
| sv2v | 1 — `test/core/nest_order.sv` |

⭐ Four independent codebases, and the Surelog names (`CastShift`, `ClogCast`, `ParamTypespec`) are
the construct family by their own authors' naming.

⭐⭐ **AND THE DECISIVE NEGATIVE CHECK ON A WIDENING CHANGE: `accepts-invalid` is UNCHANGED at 21.**
A criterion that admits more could have bought its 12 by accepting text the LRM forbids; re-adjudicated,
it did not accept one new invalid file. Together with the 9 `invalid_*` repro rows still REJECTing,
that is over-acceptance measured in two independent places rather than argued from the diff.

###### ⛔⛔ RESULT 11 — SIX OF THE TWELVE DO NOT MOVE THE BAR, AND WHICH SIX IS THE FINDING

Re-adjudicating the new results (sandboxed) splits the 12 exactly:

| was | becomes | files | effect on the bar |
|---|---|---|---|
| `divergence:unexplained_rejects_valid` | **`match`** | 5 Surelog + 1 sv2v | ⭐ **axis-2 bar 309 → 303**, `match` 5814 → 5820 |
| `deferred:chained_only` | `deferred:chained_only` | 4 black-parrot + **2 opentitan** | **none** — the row's verdict improved and it still contributes nothing |

⛔⛔ **The two OpenTitan rows are in the second group.** They are the rows `.13c.2b` priced, the rows
`defect_constant_size_cast.sv`'s manifest note names, and the rows `.17` acceptance (c) calls out —
and because a design file parsed in isolation is `deferred:chained_only` by policy, **their flip is
invisible to the graduation bar.** The fix is real, measured twice, and the published number does not
see it.

⇒ that is `.13`'s standing *"only 46.3 % of the corpus is adjudicated"* caveat with a price attached
for the first time: **the deferred half absorbed half of this slice's corpus gain.** Routed as
evidence into `SV-CORPUS-GRAD.13`, and it strengthens rather than weakens the case that the
DENOMINATOR is the real bar.

###### ⛔ THE ORACLE PROMOTION IS OWED, AND IT IS A SEPARATE UNIT — `SV-CORPUS-GRAD.13h` NEW

The tracked characterization declares its own staleness rule: *"re-hash these three inputs; if any
hash differs, this report no longer describes your tree and the honest act is to re-measure, not to
quote."* `generated/systemverilog_parser.rs` changed, so it is stale by that rule.

⛔ It is deliberately NOT promoted inside this slice, and the reason is mechanical rather than
preferential: promoting `results.tsv` alone makes the census instrument **REFUSE** —

```text
⛔ REFUSING: Surelog/tests/BlackBePipeInt/dut.sv is `fail` in the manifest and `pass`
   in the results file. One of the two is stale; a stratification built on the stale one would be wrong.
```

— so the promotion is an atomic cascade (characterization → adjudication → census → every live
surface carrying the tuple → `MEMORY.md`'s bar) that **changes a published status number** and is
owned by the corpus tree, not by the engine tree.

⭐ **And the measurement it starts from is TRACKED, not left in a build directory.**
`docs/tasks/artifacts/sv_corpus_grad/es17_slice9_flip/corpus_transitions.tsv` carries all 12 rows
with their before/after observed verdict, their before/after adjudication class and their bar effect.
⛔ Deliberately the DELTA and not the 16 336-row result set: the full run lives under
`rust/target/`, which is **git-ignored**, and a hand-off that points at a git-ignored path is a
hand-off to nothing (`.13` slice 4b's class). The full re-run costs ~4 minutes of parse, so it is
re-derivable rather than irreplaceable — which is the stated reason for tracking one and not the
other (`CI-PARITY-GATE-ROT.29` acceptance (e)).

###### ⛔⛔⛔ RESULT 12 — THE FLIP COSTS **+24.3 %** PARSE TIME, MEASURED CLEANLY — AND MY FIRST NUMBER (`~11 %`) WAS WRONG BECAUSE IT TRUSTED A STALE BASELINE

⛔ **Measured and surfaced rather than classified away**, because *"peak speed — costs are REJECTED,
not traded"* is one of the project's two non-negotiables and this slice changes the flagship parser's
hot path.

⛔⛔ **AND THE FIRST NUMBER THIS BOX CARRIED WAS WRONG — RECORDED, NOT QUIETLY REPLACED.** It read
*"parse time rose ~11 %"*, computed against the tracked 2026-08-12 baseline, with the attribution
left open. The deciding arm has since been built and it says **+24.3 %**: the stale baseline had been
measured under materially faster conditions, so comparing to it **halved the apparent cost**. ⇒ the
lesson is this leaf's own recurring one applied to a performance number —
[[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] — and the corollary is
sharper: **a stale baseline does not merely add noise, it can bias in a specific direction**, and the
direction here was flattering.

**THE CLEAN A/B — one session, one binary build, one `generated/systemverilog.json`, and the
admission is the only difference:**

| arm | total | mean/file | corpus verdicts |
|---|---:|---:|---|
| NARROW (`--indirect-lr-admit-starvation-safe-only`) | **303.0 s** | 0.0185 s | pass 9 762 / fail 6 574 |
| SHIPPED, run 1 | 376.7 s | 0.0231 s | pass 9 774 / fail 6 562 |
| SHIPPED, run 2 | 373.6 s | 0.0229 s | — |
| *(stale tracked baseline, 2026-08-12, different machine state)* | *339.7 s* | *0.0208 s* | *pass 9 762 / fail 6 574* |

⇒ **shipped / narrow = 1.243 and 1.233 ⇒ +23 % to +24 %.** Per file: **5 796 slower, 155 faster**,
74 moving by more than 50 ms — an across-the-board cost, not the twelve newly-passing files doing
more work.

⭐⭐ **AND THE NARROW ARM VALIDATES ITSELF, which is why this A/B can be trusted where the baseline
one could not**: it reproduces the tracked baseline's corpus verdicts **exactly** — `pass=9762
fail=6574` on all 16 336 files. So the lever is a proven behavioural inverse of the pre-flip parser,
and the +24 % is attributable to the admission and to nothing else in the build.

⛔⛔ **THIS IS A NON-NEGOTIABLE IN TENSION AND IT IS A DIRECTOR CALL, NOT AN IMPLEMENTATION
DETAIL.** *"Costs are REJECTED, not traded"* and *"accuracy before speed"* + *"SV is 100 % LRM-compliant
by default"* both bind here: the flip closes a real under-acceptance defect (12 corpus files, two
OpenTitan rows, an LRM A.8.4 construct) and charges ~24 % for it. ⇒ **surfaced to the director with
the number, and owned by `.20` NEW** (below) rather than shipped under a *"correctness beats speed"*
rationalisation. ⛔ What is NOT claimed: that 24 % is intrinsic. Nothing has profiled where it goes,
and `.20`'s first acceptance item is exactly that.

###### ⛔⛔ RESULT 13 — AN OPEN QUESTION THIS SLICE FOUND AND IS NOT CLOSING: THE NARROW ARM DOES NOT REPRODUCE THE PRE-FLIP PARSER BYTE-FOR-BYTE

Building the speed A/B's control arm surfaced something worth stating rather than absorbing.
Re-deriving SystemVerilog with `--indirect-lr-admit-starvation-safe-only` — the pre-slice-9 policy —
on the same binary and the same `generated/systemverilog.json` gives a parser that is
**behaviourally the pre-flip one** and **not byte-identical to it**:

| artifact | rule fns | guard rules | bytes | sha256 |
|---|---:|---:|---:|---|
| tracked pre-flip (`88b06424`, old binary) | — | — | 131 642 655 | `4330ff8e…` |
| narrow arm (this session, this binary) | 1 492 (= 1 488 grammar rules + 4 helpers) | **0** | 131 542 908 | `dae09343…` |
| shipped (this session) | 1 612 (= 1 608 + 4) | 6 | 143 907 016 | `463c6476…` |

⭐ The rule COUNT is exactly the pre-flip grammar's (`--lint-grammar` under the lever reports 1 488)
and the guard count is zero, so the lever is a faithful **policy** inverse — which is all the lint
A/B (`28 → 0`) and the speed A/B require, both being same-binary comparisons. ⛔ But **99 747 bytes
separate two artifacts that ought to be identical**, and *"they behave the same"* is not *"nothing
else moved"*.

⛔ **NOT root-caused here, and the reason is the anti-spin rule rather than disinterest.** Two
hypotheses survive without new tool output — (a) something in this slice's diff perturbs SV codegen
even under the narrow admission, or (b) an INPUT moved: `generated/systemverilog.json` is a
git-ignored derived artifact regenerated whenever `make` sees the frontend binary as newer, so every
session that rebuilds `ast_pipeline` silently re-derives it. ⭐ (b) is `.16`'s finding class one level
over, and the pre-flip file was produced from a JSON this working copy no longer has. Distinguishing
them needs a measurement (regenerate twice for determinism; re-derive at the parent commit's source
with today's JSON), not more reading.

⛔ **What it does NOT put in doubt, stated so the boundary is checkable**: the 10 other parsers were
measured byte-identical against the session-start snapshot with this same binary, so the diff is
codegen-neutral for every family that does not enter this pass; and both speed arms are built from
ONE JSON by ONE binary differing only in the admission, which makes that comparison *cleaner* than
one against a two-day-old artifact, not weaker.

⇒ owned as **`ENGINE-UNIVERSAL-SERVICES.19` NEW** (below) rather than left as a paragraph — this
repository's rule is that every finding is FIXED and routing decides WHEN.

⭐⭐ **AND THE SPEED ARM ADDED THE ONE FACT THAT MOST CONSTRAINS `.19`'s HYPOTHESES**, measured
after this box was first written: the narrow-arm parser reproduces the pre-flip corpus verdicts
**EXACTLY** — `pass=9762 fail=6574` across all 16 336 files, identical to the tracked baseline. So the
99 747 bytes are **provably not behavioural on the corpus**, which rules out the alarming reading
(that the diff changed what SV accepts under the old policy) and leaves the two recorded hypotheses
intact. ⛔ It does not CLOSE `.19`: *"no behavioural difference on 16 336 files"* is a strong bound,
not a proof of byte-equivalence, and this leaf declines to treat the two as one claim — which is
precisely `.19`'s own acceptance (d).

###### ⛔⛔ SELF-AUDIT (director check, 2026-08-14) — OF THE THREE FINDINGS THIS SLICE SURFACED, **ONE** WAS SIGNOFF-GRADE, AND THE WORST ONE WAS THE ONE TOUCHING A NON-NEGOTIABLE

Asked whether the three surfaced findings were signoff-grade, the audit says **no** — and the pattern
is worth more than the verdicts: **the finding with the highest stakes had the weakest method.**

**FINDING 1 (the bar cannot see half its own fix) — substance SOUND, two gaps, both fixed.**
- ⛔ **GAP A — an unverifiable superlative.** It claimed *"the FIRST priced evidence for `.13`'s
  thesis"*. Nothing was checked. Measured instead: the bucket accounting only landed at `.13b` on
  **2026-08-11**, three days earlier, so any prior instance was **unmeasurable by construction** —
  the superlative is not merely unproven, its evidence could not exist. ⇒ retracted to *"measured
  evidence"* in both surfaces, and the retraction states WHY rather than just dropping the word.
- ⛔ **GAP B — the structural finding was OWNED BY NOBODY.** *"A `deferred:chained_only` row can
  improve from `fail` to `pass` and the bar cannot see it"* appeared as prose in four places with
  **no acceptance criterion anywhere**. That is `.17` slice 7b's GAP C **verbatim, reproduced by the
  slice that cites it** — reading a card does not apply it, again. ⇒ promoted to
  `SV-CORPUS-GRAD.13h` acceptance **(f)**, which forces a ruling between exactly two dispositions
  (by-design ⇒ say so where the bar is published; defect ⇒ split the bucket) and refuses
  *"it is complicated"* as an answer.

**FINDING 2 (parse time) — NOT signoff-grade, and the audit is what caught that the NUMBER was wrong.**
- ⛔ **GAP C — n=2 reported as a noise floor.** *"The same-binary noise floor is bounded at −0.8 %"*
  came from **two** runs. Two samples cannot estimate variance; that is one paired observation
  wearing a statistic's clothes.
- ⛔ **GAP D — a possible NON-NEGOTIABLE violation with no owner.** It was surfaced with an
  in-flight measurement and no leaf. GAP C's class again: a risk with no obligation.
- ⛔⛔ **AND THE HEADLINE FIGURE WAS WRONG BY ROUGHLY 2×** — `~11 %` against a stale baseline vs
  **+24.3 %** measured cleanly. ⭐ The mechanism is the transferable part: the 2026-08-12 baseline had
  been taken under materially faster machine conditions, so **a stale baseline does not merely add
  noise — it biased the estimate in the FLATTERING direction.** ⇒ `.20` NEW owns the cost, profile-first,
  plus a standing corpus-timing ratchet, and the wrong number is corrected IN PLACE rather than replaced.

**FINDING 3 (`MEMORY.md` had 2 bytes of headroom) — SHOULD NOT HAVE BEEN SURFACED.**
It is measured and true, and it is a **status observation, not a finding**: it names no defect, and
the remedy it implies — demote content, never raise the cap — is exactly what the `MEMORY-ARCH`
doctrine already **forces** on the next session that tries. ⇒ it belonged in the routine-decision
lane the surfacing directive explicitly bounds, and putting it beside a non-negotiable **diluted the
two that mattered**. ⛔ Recorded rather than silently dropped, because "I surfaced too much" is the
failure mode that trains a director to skim.

⇒ **1 of 3 sound-but-under-owned, 1 of 3 wrong AND unowned, 1 of 3 not a finding.** The durable
lesson is the correlation: **the finding that touched a non-negotiable was the one measured worst**,
because a number that confirms an expectation gets less scrutiny than one that surprises — and `~11 %`
was comfortable in a way `+24.3 %` is not.

###### Acceptance Checklist (enforced) — `.17` slice 9

- [x] **REPRODUCE / ISSUE** — at `88b06424`, PGEN could build and execute the guarded rewrite and did
  not use it. Reproduced from the shipped surface on ONE binary, three ways:
  `ast_pipeline grammars/systemverilog.ebnf --lint-grammar` → `left_recursion_unhandled=28` with
  `starvation-safe candidates: 0/28`; `--report-indirect-lr-plan` → `indirect_guard_chains=0`; and the
  consequence in real text — `python3 stimuli/sv/run_adjudication_repros.py` listing
  `defect_constant_size_cast.sv` / `…_corpus_shape.sv` as `expect=REJECT`, both derived from
  `top_{darjeeling,earlgrey}_rnd_cnst_pkg.sv` which the tracked `results.tsv` records as `fail`.
  ⇒ two OpenTitan files and an LRM A.8.4 construct that PGEN had a working fix for and declined to
  apply.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the fix was unreachable: it was never a defect in the guard,
  the planner or the emitter — all three were built and measured by slices 4–8. It was **the
  admission criterion**, and nothing else. WHERE, to one expression:
  `indirect_lr_elimination::eliminate_indirect_left_recursion` passed
  `CandidateAdmission::StarvationSafe`, whose definition is `surviving_starvation_sites().is_empty()`,
  so a candidate that a guard would make safe was refused before `plan_elimination` ever saw it — and
  `plan_guard_chains` (step 6) therefore had nothing to emit **by construction**, which is exactly what
  `indirect_guard_chains=0` measured for two slices.
  ⛔ Tool-located rather than read off the source, and the locating tool was a test written one slice
  earlier: `.17` slice 7 recorded plant 3 as *"the exact edit slice 9 will make"* and predicted which
  assertion would fail. Making the edit produced that failure verbatim, printing the whole
  `SynthesizedGuard` (RESULT 1) — so the criterion is provably the only thing standing between the
  emitter and the parser.
- [x] **FIX** — fix-hierarchy tier = **engine policy**, and deliberately the smallest possible one:
  `CandidateAdmission::StarvationSafe` → `GuardFeasible` at that single call site. Everything else in
  the diff is a consequence.
  `indirect_lr_elimination`: the opt-in door inverts —
  `…_admitting_guard_feasible` → `…_admitting_starvation_safe_only`, with the banner rewritten to
  warn in the new direction (*"a parser built this way REJECTS LRM-legal SystemVerilog"*), still
  `std::eprintln!` because the trace shadow at `ast_pipeline/mod.rs:513` is unchanged.
  `ast_pipeline/mod.rs`: `PipelineConfig::indirect_lr_admit_guard_feasible` →
  `indirect_lr_admit_starvation_safe_only` (default `false`), the two-door branch inverted.
  `main.rs`: `--indirect-lr-admit-starvation-safe-only`, plus `print_synthesized_guards` — ONE renderer
  now shared by the shipped and dry-run blocks, with a load-bearing verb (`guard` / `would guard`) and
  a computed `max_hops=` (RESULT 5). `bin/pgen_ast.rs`: the field in its exhaustive literal.
  ⭐ **The two-door SHAPE is preserved and only its direction inverted**, which is the structural
  invariant slice 8 established: the shipped call site names the shipped policy, so changing what
  ships is an edit to that function's criterion and can never happen by a caller passing an argument.
  ⛔ Deliberately NOT done: the candidate ORDERING (still carries no guard term — `.17`'s standing
  note), the corpus oracle promotion (`SV-CORPUS-GRAD.13h`, and the census instrument REFUSES a
  partial one), and any grammar byte.
- [x] **ADDRESSED (verified)** — measured before→after on **one binary** via the new A/B lever, at
  four independent tiers, each of which could have refuted the others:
  1. **lint** — `systemverilog` `left_recursion_unhandled` **28 → 0**; every other family 0 under both
     arms; every other SV lint counter identical (`non_terminating=0`,
     `ordered_choice_shadowing=0`, `always_succeeds_alternatives=6`, `unreachable_rules=0`,
     `undefined_references=0`, `unbound_fact_kinds=0`, `nullable_repetition=0`, `profile_orphans=0`).
  2. **unit** — `cargo test --lib indirect_lr` → **28 passed / 0 failed**, including the inverted
     two-sided control `the_narrowed_admission_synthesizes_no_guard_on_the_knot_the_shipped_one_guards`.
  3. **the two-sided ratchet** — `run_adjudication_repros.py`: `failures=2`, **both of them defect
     reproducers that now PARSE**, zero controls moved (RESULT 8). Re-baselined as the runner
     instructs, re-run **`checked=29 armed=7 listed=29 failures=0`**.
  4. **the corpus** — 16 336 files re-parsed: **12 transitions, ALL `fail → pass`, ZERO `pass → fail`**,
     across four independent upstreams, with `accepts-invalid` unchanged at **21** (RESULT 10).
  5. **the narrow control arm** — a parser built from the same JSON by the same binary under
     `--indirect-lr-admit-starvation-safe-only` reproduces the pre-flip corpus verdicts **EXACTLY**
     (`pass=9762 fail=6574`). ⭐ That is the strongest single statement available about the A/B lever:
     it is a proven behavioural inverse over 16 336 real files, which is what licenses reading both
     the `28 → 0` lint delta and the **+24.3 %** speed delta as properties of the admission alone.
  ⭐⭐ **Falsifiability is not a plant here — it is the parent commit.** The flip's RED/GREEN pair was
  written by slice 7 one slice ahead of the edit and fired verbatim (RESULT 1), so the calibration
  artifact is `git show 88b06424` rather than a reconstructed patch. ⇒ `CI-PARITY-GATE-ROT.29`
  acceptance (e) discharged by the second of its two answers.
- [x] **NO REGRESSION** — ⭐ measured at the strongest available tier, and the tier is *broader* than
  byte-identity because byte-identity is exactly what this slice cannot claim.
  `shasum -a 256 generated/*.rs` after regenerating all 11 parsers with the new binary:
  **`generated/systemverilog_parser.rs` is the ONLY artifact that changed**; the other **10** are
  byte-identical to the pre-change snapshot. That matches the lint A/B prediction exactly — SV was the
  only family whose counter moved — so the blast radius is measured twice by independent means.
  ⛔ **Regression evidence for the one parser that DID change is the two-sided ratchet plus the
  corpus**, both above: 12 corpus files gained and none lost, 12 controls and 9 over-acceptance guards
  unmoved, `accepts-invalid` flat at 21. ⛔ A widening change's characteristic failure is
  over-acceptance, and it is checked in two independent places rather than argued from the diff.
  `bash scripts/check_doctrines.sh` → **ALL 18 PASS**.
  ⭐⭐ **CONFIRMATORY SWEEP, CONSUMED — and unlike slices 7 and 8 it was NOT skippable**, because their
  justification for skipping it was a byte-identical generated tree and this slice does not have one.
  `cargo test --features "generated_parsers ebnf_dual_run" --lib -- --skip deep_nesting` →
  **1 108 passed / 1 failed / 28 ignored** in 429 s (peak 11 474 MB under the guard). The single
  failure is `unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`,
  **pre-existing and owned by `LANG-CAPABILITY-AUDIT.10.15`** — the identical row slice 4b recorded at
  1 085/1/28 and slice 5 at 1 091/1/28. ⇒ the pass count rose with the tests added since; **the
  FAILURE SET is unchanged**, which is the claim that matters.
  ⭐⭐ **The load-bearing rows inside that sweep are the CROSS-ORACLE gates, and they are why a changed
  parser can be trusted here**: `parse_harness_equivalence::gate::certified_grammars_are_byte_identical`
  … ok, `parse_harness_combinator_suite::gate::every_structural_combinator_is_byte_identical` … ok,
  `combinator_coverage_is_complete` … ok. ⛔ Those assert the INTERPRETER and the GENERATED PARSER
  produce byte-identical ASTs — so the guarded rewrite did not merely keep accepting the same inputs,
  it kept building the same tree. That is the property `lr_chain_fold` owes and the one a positional
  change to a base rule could silently break; measured, not argued.
  ⛔ The two banks whose SUBJECT the flip moved were re-anchored and re-run rather than re-baselined:
  `guard_feasibility/probe.sh` **18/18** and `guard_dry_run/probe.sh` **18/18**, with **every declared
  value byte-identical to the pre-flip run** (RESULT 3/4) — the dry run had predicted this exact
  rewrite since slice 3, and that prediction is now checkable against the thing itself.
- [x] **LOCKSTEP** — this leaf (RESULT 1-13 + acceptance (d) discharged by measurement, and its own
  `~11 %` speed figure CORRECTED to `+24.3 %` in place rather than replaced); **`.19` NEW** (the
  unexplained 99 747 bytes) and **`.20` NEW** (the parse-time cost, with a DIRECTOR CALL open);
  `docs/tasks/CI-PARITY-GATE-ROT.md` (**`.31` NEW**, director-ruled: `--debug --trace` off the
  shipping/CI generation path);
  `docs/tasks/SV-CORPUS-GRAD.md` (**`.13h` NEW**, with the corpus delta as routing evidence);
  `docs/tasks/artifacts/sv_corpus_grad/es17_slice9_flip/corpus_transitions.tsv` (**new tracked
  artifact** — the delta made durable, because the run is under git-ignored `rust/target/`);
  `stimuli/sv/adjudication_repros/MANIFEST.tsv` (two rows re-baselined `defect` → `fixed`, citing this
  work unit); the three affected banks (`guard_parses` arms inverted + its README, `guard_dry_run`
  D11 rewritten as an exact-set check plus D11a/D11c NEW, `guard_feasibility` reports levered);
  `TOOLBOX.md` §5.5 (the new lever, the census bullet that said *"REPORTED, NOT APPLIED"* and is now
  false, the residual table, the `🛡` shipped block); `docs/book/src/grammar-wellformedness.md` (a new
  *"The guarded admission is what PGEN ships"* section, and the STARVED sections re-framed as the view
  of the problem); `docs/reference/RUST_CODEBASE_ANALYSIS.md` (a slice-9 steering note; the slice-2
  note's *"deliberately UNCHANGED"* marked superseded); `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  `MEMORY.md`, `docs/TASK_TREE.md`.
  ⛔ **LIVE STATUS REVIEWED AND UNCHANGED, which `COMMIT.md` requires stating rather than assuming.**
  `done_bar_family_register_v0.json` keeps `systemverilog: "Mostly Done"` and
  `docs/book/src/roadmap-and-live-status.md` is untouched: the SV `Done` bar's axis 2 is external-corpus
  graduation, which this slice improves (12 files, 0 lost) without achieving — the corpus is still
  46.3 % adjudicated and the axis-2 bar is still the PUBLISHED 309 until `SV-CORPUS-GRAD.13h` promotes
  the 303. ⇒ editing either surface here would publish a status the proof surfaces do not yet support,
  and the two `*_parser_family_status_gate.sh` arms exist to catch exactly that.
  ⛔ **A user-visible parser behaviour DID change**, so unlike slices 7 and 8 the contract surface is
  in scope and was checked: the change is purely additive acceptance (an LRM-legal construct that was
  rejected now parses) with no AST-shape change — `lr_chain_fold` rebuilds the declared left-nested
  tree and the guarded base rule is positionally identical to the rule it stands in for — so no
  contract or schema edit is owed, and that is a measured statement rather than an omission.
  ⭐ `promotion: PROMOTED` — [[a-bank-pinned-to-the-shipped-behaviour-is-pinned-to-a-moving-target]],
  plus an update to [[a-conservative-criterion-and-a-measurement-are-different-objects]] recording that
  its promotion finally happened, seven slices later, with the ratchet paid rather than waived, and a
  `KNOWLEDGE_MAP.md` regeneration.

#### ⛔⛔⛔ `.20` NEW `todo` — the guarded admission costs **+24.3 %** parse time on the SV corpus, and *"costs are REJECTED, not traded"* is a NON-NEGOTIABLE (opened 2026-08-14 session #232 by `.17` slice 9)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **Measured on the cleanest available A/B**: one session, one binary build, one
  `generated/systemverilog.json`, the admission the only difference.
  **NARROW 303.0 s / 0.0185 s per file → SHIPPED 376.7 s and 373.6 s / 0.0231 s and 0.0229 s.**
  Ratio **1.243** and **1.233**. Per file: **5 796 slower, 155 faster**, 74 moving by >50 ms.
- **The control validates itself**, which is what makes the number trustworthy: the narrow arm
  reproduces the pre-flip corpus verdicts EXACTLY (`pass=9762 fail=6574` over 16 336 files), so it is
  a proven behavioural inverse and the delta is attributable to the admission alone.
- ⛔ **The first number reported for this was `~11 %` and it was WRONG** — computed against the
  2026-08-12 tracked baseline (339.7 s), which had been measured under materially faster machine
  conditions. A stale baseline biased the estimate in the FLATTERING direction, which is worth more
  than the number itself: [[a-bank-pinned-to-the-shipped-behaviour-is-pinned-to-a-moving-target]] has
  a performance-measurement sibling, and this is it.
- **It reproduces by construction outside SystemVerilog**, and that is the reason this is an ENGINE
  leaf rather than an SV one: the cost is paid by the shape `X := X_lr_base ( X_lr_suffix )*` plus a
  per-iteration and a trailing structural lookahead, which the pass will emit for ANY grammar whose
  knot is guard-feasible. SV is simply the first family to have one absorbed.
- ⛔ **Nothing has profiled it.** *"~24 % is intrinsic to the design"* is NOT measured and must not be
  assumed — the plausible sources are at least four and they have different prices: (i) the
  eliminated `casting_type`/`property_expr` sit in the expression hot path and every expression parse
  now traverses base+suffix instead of one rule; (ii) two structural lookaheads per guarded
  iteration; (iii) 120 extra rules perturbing i-cache/branch prediction on a 143 MB parser; (iv) memo
  behaviour on the new rule set. ⭐ (i) and (iii) would be paid even with the guards deleted, which
  makes them separable by measurement.

**Acceptance:** (a) ⛔ FIRST, and before any optimisation is designed — PROFILE it and attribute the
+24 % across (i)-(iv) above, so the fix targets the cost rather than the suspicion
([[feedback_why_and_where_before_solution]], and this leaf's own `.17` history of designs built on
unmeasured premises); (b) a third A/B arm — guard-feasible admission with the guard EMISSION
suppressed — separates "absorbing the knot" from "guarding it", which is the single most
discriminating measurement available and costs one generation plus one corpus run; (c) a priced
option list with a measured target, and an explicit statement of what is IRREDUCIBLE if some of it
is; (d) ⛔ a standing corpus-timing ratchet, because this regression was invisible to every gate the
repository has — the lint, the two-sided repro ratchet, the corpus pass/fail count and all 18
doctrines were GREEN across a 24 % slowdown, and a cost nothing measures is a cost that grows;
(e) ⛔ do NOT close this by citing *"accuracy before speed"* — that ordering licenses SHIPPING the
correctness fix, not KEEPING its price.

⛔ **DIRECTOR CALL OPEN** (surfaced 2026-08-14): whether the flip ships now with `.20` owning the
burn-down, or is held until the cost is reduced. The engineering recommendation on record is to
**ship and burn down** — the defect it closes is under-acceptance of LRM-legal text, the corpus
gained 12 files and lost none, and holding a correctness fix behind an unprofiled performance number
trades a measured defect for an unmeasured one. ⛔ That is a recommendation, not the decision.

##### ✅ RULED 2026-08-14 — **DELEGATED TO THE ENGINEER BY THE DIRECTOR** (*"This call is yours to make. The decision shall be sota, signoff!"*)

⛔ **FIRST, THE QUESTION AS POSED CONFLATES THREE SEPARABLE DECISIONS**, and a signoff ruling has to
say which one it answers. Answering the fused question is how *"ship and burn down"* becomes a
euphemism for *"accept the cost"*:

| | question | ruling |
|---|---|---|
| **A** | does the guarded admission STAY IN THE TREE, or is it reverted? | **STAYS** |
| **B** | may SystemVerilog reach `Done` / be delivered to Nexsim carrying an unexplained +24.3 %? | **NO — no waiver** |
| **C** | what is the work order inside `.20`? | **the GUARD before the PROFILE** |

**A — the admission stays.** Reverting trades a **measured correctness defect** for an
**unprofiled performance number**, which is the exact inversion of the reason to be cautious here.
The flip took `left_recursion_unhandled` **28 → 0**, made LRM-legal `8'(1)` parse, and gained 12
corpus files while losing none. ⛔ *"Costs are REJECTED, not traded"* is a demand that the cost be
**eliminated**, not a licence to restore a defect — and **a revert eliminates nothing**: it
re-opens 28 unhandled cycles and re-rejects LRM-legal text, violating the sibling non-negotiable
(*SV is 100 % LRM-compliant by default; over-acceptance is a defect — and so is under-acceptance*)
to buy back a number nobody has attributed. Both arms are defects; exactly one of them is currently
**fixed**, and the ruling does not un-fix it.

**B — and this is what makes A honest rather than a euphemism.** The admission holds a
**CONDITIONAL TENANCY**, not a pass. The SV `Done` bar gains a clause: *the parse-time cost of the
guarded admission must be attributed and then either eliminated or declared IRREDUCIBLE with the
measurement that proves it.* ⛔ No waiver, and acceptance (e) already forbids the obvious escape —
*"accuracy before speed"* licenses SHIPPING the correctness fix, never KEEPING its price. SV is
`Mostly Done` and corpus-gated at 46.3 % regardless, so this clause costs the delivery **nothing
today** and binds precisely when it would otherwise be tempting to skip.

**C — reorder: acceptance (d) BEFORE (a).** ⭐ This does **not** violate (a)'s `⛔ FIRST`, which
governs *"before any optimisation is designed"* — a guard is instrumentation, not an optimisation.
Three reasons, in order of force:
1. ⛔ **The leaf's own words indict the current state**: *"this regression was invisible to every
   gate the repository has — the lint, the two-sided repro ratchet, the corpus pass/fail count and
   all 18 doctrines were GREEN across a 24 % slowdown, and a cost nothing measures is a cost that
   grows."* Profiling is a multi-session campaign. Doing it with the tree unguarded is
   `GATE-REACHABILITY`'s founding failure — *a check that nothing invokes is indistinguishable from
   a check that does not exist* — applied to time instead of to targets.
2. **The ratchet IS the profile's baseline harness.** Built first, (a) inherits a deterministic,
   re-runnable instrument instead of an ad-hoc timing script — and an ad-hoc timing script against
   a stale baseline is *literally what produced the wrong `~11 %`* recorded above.
3. ⭐⭐ **The ratchet must NOT be wall-clock-primary, and this is the substantive engineering call.**
   The `~11 %` error came from comparing across *"materially faster machine conditions"*, so a
   wall-clock ratchet inherits the very defect that misled this leaf once already. PGEN already
   ships a machine-independent substrate: **`--dump-rule-entry-counts-json`** (TOOLBOX 3.4) — exact
   per-rule entry counts, *"deterministic for a deterministic parser ⇒ a re-runnable oracle"*, and
   build-mode-independent. ⇒ **ratchet on TOTAL RULE ENTRIES over a fixed corpus sample; keep
   wall-clock as a coarse advisory with a wide band.** Entries are the *mechanism* causes (i) and
   (ii) move through; wall-clock is only the *symptom*, and it is the half that cannot survive a
   machine change or a hosted runner.
   ⚠️ **Honest bound, stated before the instrument is built, not after**: the entry counters route
   the parse to the PROTOCOL graph (the observability twin, TOOLBOX 3.4 ROUTING), so they guard
   *structural* work and **cannot** describe the fused `cascade_*` graph on which the +24.3 % was
   measured. Neither metric alone is sufficient. The ratchet therefore declares **both** and states
   which one binds — a two-metric guard that says what it cannot see, rather than one number that
   quietly means less than it appears to.

⛔ **What this ruling deliberately does NOT decide**: the attribution across (i)-(iv), the priced
option list, and whether any part of the cost is irreducible. Those are (a)/(b)/(c) and they are
**measurements, not judgements** — pre-deciding them here is the *"designs built on unmeasured
premises"* failure this leaf's own `.17` history is a record of.

##### ✅ `.20` SLICE 1 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0033`, 2026-08-15 session #234) — ACCEPTANCE (d) DISCHARGED: the cost is measured, and the ratchet refuses a rise

⭐ Ruling C's work order, executed: **the guard BEFORE the profile.** Nothing in this slice
optimises anything — it builds the instrument (a) will inherit as its baseline harness, because an
ad-hoc timing script against a stale baseline *is literally what produced the wrong `~11 %`*.

**WHAT LANDED** — three tracked surfaces plus the wiring:

| surface | what it is |
|---|---|
| `stimuli/sv/corpus_parse_cost.py` | the instrument: census, sample derivation, measurement |
| `stimuli/sv/parse_cost_sample.tsv` | the PINNED 192-file sample (40 hot / 40 lr / 112 breadth) |
| `docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/` | the baseline: `cost.md`, `entries.tsv`, `advisory.json` |
| `scripts/check_parse_cost_ratchet.sh` | the gate, registered as doctrine `PARSE-COST-RATCHET` |
| `rust/Makefile` | `sv_parse_cost_ratchet`, `sv_parse_cost_rebaseline` |

###### ⭐⭐ RESULT 1 — THE INSTRUMENT HAS GROUND TRUTH: IT REPRODUCES THE GRADUATION ORACLE ON EVERY ONE OF 16 336 FILES

A full-corpus entry census (16 336 files, `-j8`, **zero** no-dump rows) was run before any sample was
chosen. Its `accepted` verdicts were joined against the tracked `stimuli/sv/characterization/
results.tsv`:

```
tracked results.tsv pass: 9774
census accepted=True    : 9774
per-file agreement      : 16336/16336 agree, 0 disagree
```

⇒ the instrument observes the SAME parser the graduation oracle does, on every file, and
`pass 9 774 / fail 6 562` reproduces `.17` slice 9's shipped-arm A/B row exactly. An instrument with
no ground truth is a confident guess; this one was made to reproduce a number the project had
already measured before it was allowed to publish a new one.

###### ⭐⭐ RESULT 2 — THE BINDING METRIC'S THREE PROPERTIES WERE VERIFIED, NOT ASSUMED

Ruling C requires a machine-independent substrate. Each property was measured before anything was
built on it:

| property | measurement |
|---|---|
| deterministic | 3 release runs of the same file → **1 unique sha256** over the dumps |
| build-mode-independent | debug vs release → `entries`/`committed`/`memo_hits` **all MATCH** |
| consistent across dumps | entry dump vs outcome dump `total_entries` → **306 491 = 306 491** |

The third mattered concretely: the census that selected the sample used the entry dump and the
baseline uses the outcome dump, so a divergence there would have made the two incomparable.

###### ⭐⭐⭐ RESULT 3 — THE RULING NAMED 3.4; MEASUREMENT PROMOTED IT TO 3.5, FOR FREE

Ruling C specified `--dump-rule-entry-counts-json` (TOOLBOX 3.4). Measured, the **outcome** dump
(3.5) costs the SAME `0.04 s` on the same file and is a strict superset — it adds COMMITTED entries
and memo hits, and therefore `raw − committed`, the parse's **failed-speculation** work. That is not
a cosmetic upgrade: it is the mechanism a structural GUARD spends its cost through.

| measured on the pinned 192-file sample | value |
|---|---:|
| rule entries | 416 841 264 |
| committed | 7 124 616 |
| **failed speculation** | **409 716 648 = 98.3 % of entries** |
| guarded-admission family entries | 3 092 966 |
| …of which COMMITTED | **127** |

⇒ the guarded admission is, to three significant figures, **pure speculation**: entered, probed,
rolled back. A ratchet on raw entries alone would have been blind to a guard that doubles its
probing and commits the same amount. ⭐ The instrument therefore binds on **three** counters, not
one, and the extra two cost nothing.

###### ⛔⛔⛔ RESULT 4 — THE FINDING THAT BOUNDS THE INSTRUMENT, AND IT PARTLY REFUTES THE RULING'S OWN PREMISE

> ⛔⛔ **SUPERSEDED IN ITS NUMBERS BY `.21` (2026-08-15) — the finding STANDS, the magnitude does
> not.** Every `0.681 %` / `~35×` below was computed with a classifier that matched only
> `_lr_base`/`_lr_suffix` — **97 of the 128** LR rule names the parser declares, no `_lr_seed` and
> not one `_lr_guard` rule, leaving **75.1 %** of the family uncounted. Corrected: the family is
> **2.741 %** of corpus entries and the bound is **~8.9×**, i.e. the gate was UNDER-claiming its own
> sensitivity by ~4×. ⭐ The BINDING counters are byte-identical across the correction, so nothing
> this slice concluded about parse COST moves. Retained unedited below as the audit trail.

Ruling C reasoned that *"entries are the mechanism causes (i) and (ii) move through"*. **Measured,
that is true only STRUCTURALLY, and the quantitative gap is large enough to change how the number
must be read.**

Across the full 16 336-file corpus the guarded-admission family takes **6 126 595 of 899 264 997
entries = 0.681 %**. The flip's entry DELTA is strictly smaller still — the rules it replaced
(`casting_type`, `property_expr`, …) were themselves entered before the flip. So:

> the entry count moved **well under 1 %** while wall clock moved **+24.3 %**
> ⇒ the binding metric is **at least ~35× less sensitive** to *this* regression than the advisory one.

⛔ **Stated as a finding rather than absorbed, because it changes what the number PROVES.** The
+24.3 % is a rise in cost **per entry**, not in the **number** of entries — and no counter can see
that. This is the performance-measurement sibling of the lesson already recorded in RESULT 12 of
`.17` slice 9, one level deeper: a metric can be exact, deterministic, machine-independent **and
still nearly blind to the thing it was chosen to watch**.

⭐ It is NOT a reason to discard the metric, and the ruling's reasoning for choosing it survives
intact: it catches structural growth EXACTLY (RED-6 below fires on **+0.00 %**), it cannot be fooled
by a busy machine, and it is the only leg that survives a hosted runner. It IS a reason that the
baseline, the gate header, `TOOLBOX.md` 3.7 and the doctrine mirror all state the bound in the same
words on every run, rather than publishing one number that quietly means less than it appears to.
⇒ **acceptance (a)'s profile is what attributes the +24.3 %; this ratchet stops it growing further
unwatched in the meantime.** Those are different jobs and (d) never claimed the first one.

###### ⚠️ RESULT 5 — THE ADVISORY WOULD HAVE MEASURED `fork`, NOT PARSING, AND THE FLOOR IS NOW SUBTRACTED

A `parseability_probe` invocation on a one-line module takes a measured **9.8 ms** (median of 10)
before it parses anything — fork + exec + the SV stdlib preload. The median corpus file's whole wall
time is ~18 ms. ⇒ an unadjusted per-file wall-clock advisory is **majority process startup**, and
would drift with the loader rather than with the parser.

The floor is therefore **re-measured inside every run and subtracted**, never baked in as a
constant, and the advisory is restricted to the `hot` tier where the remaining signal dominates
(measured 82–90 % signal on the two heaviest files). Two further design consequences, both measured
rather than stylistic:
- the ENTRIES pass runs **parallel** (an exact integer cannot be moved by contention) while the
  wall-clock pass runs **serial** — contention is precisely the confound that produced the `~11 %`;
- the advisory lives in `advisory.json`, **outside** the byte-compared `cost.md`, because a
  machine-dependent number inside a byte-compared artifact makes the artifact undiffable — which is
  the staleness defect `SV-CORPUS-GRAD.13i` is a record of.

###### ⭐⭐ RESULT 6 — THE CHEAP TIER IS A PROOF, NOT A SAMPLING SHORTCUT

The gate runs on **every commit** via `.githooks/pre-commit`, where a 2.5-minute re-measure is not
viable. Rather than sampling less, tier 1 re-hashes the three inputs the baseline names — grammar,
generated parser, and a digest over the sampled corpus files' bytes. The binding metric is an exact
function of exactly those (RESULT 2), so:

> if none of the three moved, the measurement **cannot** have moved — and if one did, the gate
> refuses and demands a re-measure instead of guessing.

⛔ The sample-input digest is not redundant with the parser hash and its absence was a real hole:
the corpora are git **submodules**, so a bump changes what is measured **without touching one byte of
PGEN**. RED-2 fires on exactly that. ⭐ And making the baseline stale on a parser hash change is the
direct fix for `SV-CORPUS-GRAD.13i` — six tracked oracles carried an identity block, only ONE was
gate-checked, four were measurably stale. This one is checked on every commit.

###### ⭐⭐ RESULT 7 — ELEVEN ADVERSARIAL ARMS, INCLUDING TWO THAT MUST REFUSE RATHER THAN PASS

`gate-flow.md` §9: *"A gate without adversarial arms is a claim, not a proof."* Every arm was fired
and every perturbation byte-restored (verified by `cmp` and a clean `git status`):

| arm | perturbation | wanted | got |
|---|---|---|---|
| GREEN | clean tree | 0 | ✓ 0 |
| RED-1 | grammar edited | 1 | ✓ 1 |
| RED-2 | a **sampled corpus file** edited (the submodule hole) | 1 | ✓ 1 |
| RED-3 | baseline records a WRONG grammar hash | 1 | ✓ 1 |
| RED-4 | identity table **deleted** | **2 (refuse)** | ✓ 2 |
| RED-5 | `entries.tsv` column renamed | **2 (refuse)** | ✓ 2 |
| RED-6 | a binding counter **ROSE** | 1 | ✓ 1 |
| CTRL-1 | an unrelated tracked file edited | 0 | ✓ 0 |
| CTRL-2 | a binding counter **FELL** | 0 + improvement note | ✓ 0 |
| + 2 restore-verification arms | tree returns to green | 0 | ✓ 0 |

⭐⭐ **RED-6 is the sharpest result here.** It fired on a rise of **349 entries out of 416 841 264 —
`+0.00 %`**. The ratchet is EXACT, not threshold-based: there is no band inside which a structural
regression can hide, which is the property a percentage threshold would have destroyed.
⭐ **RED-4 and RED-5 matter as much**, and are the arms most gates omit: a deleted identity table and
a renamed column both make the gate **REFUSE (exit 2)** rather than pass. A check that cannot see its
subject must say so — `gate-flow.md` §7.1/§7.8, and §7.8 is exactly the *consumer that outlived its
producer's schema* failure that a positional unpack would have reproduced here.

###### ⛔ WHAT THIS SLICE DELIBERATELY DOES NOT DO

- **(a) the profile** — not started. RESULT 4 sharpens it: the attribution must explain a *per-entry*
  cost rise, so causes (i) and (iii) are now the leading candidates and (ii) is measurable via the
  failed-speculation counter this slice added.
- **(b) the third A/B arm** — not built. It is now materially cheaper: the baseline publishes the
  guarded-admission family's entries and committed counts, so the narrow arm's delta is a
  subtraction rather than a re-derivation.
- **(c) the priced option list** — not started; it depends on (a).
- ⛔ **The `.20` leaf stays `todo`.** Acceptance (d) is discharged; (a), (b), (c), (e) are not, and
  clause **B** of the ruling — no waiver, conditional tenancy — is unchanged and still binds.

###### Acceptance Checklist (enforced) — `.20` slice 1

- [x] **REPRODUCE / ISSUE** — the regression this gate exists for is already reproduced and recorded
  in `.17` slice 9 RESULT 12 on a clean one-binary A/B (narrow `303.0 s` vs shipped `376.7 s` /
  `373.6 s`, ratio **1.243** / **1.233**). The issue THIS slice reproduces is the *absence of any
  instrument*: measured at `4d1f995c`, `git ls-files 'scripts/*.sh' 'rust/scripts/*.sh'
  'rust/Makefile' '.githooks/*' '.github/workflows/*.yml' | xargs grep -l` for any parse-timing or
  entry-count ratchet returns **nothing**, and `scripts/check_doctrines.sh` registered **18**
  doctrines, none of which measures parse cost. ⇒ the +24.3 % was invisible by construction, not by
  bad luck.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY nothing caught it: every existing proof surface asks a
  **verdict** question (does it parse? does the lint fire? does the corpus pass count hold?) and the
  regression moved **no verdict at all** — `.17` slice 9 measured `pass 9 774 / fail 6 562` GREEN
  across the slowdown, and this slice's own census reproduces those verdicts on 16 336/16 336 files
  (RESULT 1). WHERE, precisely: the gap is between `gate-flow.md`'s Layer-1 leaf gates and the
  doctrine registry — `performance_gate` exists but keys on the **regex** benchmark thresholds, so
  the SV parser had **no cost surface of any kind**. Tool-located, not inferred: the full-corpus
  entry census (RESULT 1) is the first measurement of SV parse cost this repository has ever
  produced, and it had to be built to answer the question at all.
- [x] **FIX** — fix-hierarchy tier = **new proof surface** (no engine, grammar or generated byte
  changes; `generated/` is untouched and its sha256 `463c6476…` is recorded in the baseline). The
  instrument, the pinned sample, the tracked baseline, the two-tier gate, the doctrine registration
  + its `DOCTRINE_ENFORCEMENT.md` §10 mirror row, and two `make` targets. Two decisions depart from
  ruling C's letter and both are measured, not preferred: **3.5 over 3.4** (RESULT 3 — free, strictly
  more sensitive) and **three binding counters instead of one**. The ruling's substance — entries
  bind, wall clock advises with a wide band, both declared, the blind spot stated — is implemented
  exactly.
- [x] **ADDRESSED (verified)** — measured before→after on the real gate, not asserted. BEFORE: no
  instrument exists; a `+24.3 %` regression passes every gate in the repository (`.17` slice 9,
  reproduced above). AFTER: `bash scripts/check_parse_cost_ratchet.sh` → `parse-cost-ratchet: OK
  (identity fresh for: generated parser, grammar, sample inputs; 192 pinned sample files)` in
  **0.4 s**, and `PGEN_PARSE_COST_REMEASURE=1` re-measures and compares in ~2.5 min. The gate's
  discriminating power is measured by **11 adversarial arms, 11/11 as wanted** (RESULT 7),
  including RED-6 firing on a **+0.00 %** rise (349 of 416 841 264 entries) and two arms that must
  **refuse with exit 2** rather than pass. ⚠️ The bound on what this proves is measured and published
  rather than left implicit: **at least ~35× less sensitive than wall clock to the specific +24.3 %**
  (RESULT 4), which is why acceptance (a) remains open and (d) does not close the leaf.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 19 enforced doctrines PASS**
  (18 pre-existing + `PARSE-COST-RATCHET`), including the `<meta:mirror>` check proving the registry
  and `DOCTRINE_ENFORCEMENT.md` §10 list exactly the same 19 ids. ⭐ Zero code, grammar or generated
  bytes changed, so no parser behaviour can have moved — and that claim is itself gate-held: the
  baseline records `generated/systemverilog_parser.rs` = `463c6476…`, the same hash the tree carried
  before this slice. Every probe-arm perturbation was byte-restored and confirmed by `cmp` plus a
  clean `git status` (RESULT 7). `make -C rust SHELL=/bin/bash mdbook_docs_gate` passes with the
  book's doctrine count corrected `18 → 19` in both places it appears.
- promotion: `docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md` (RESULT 4).

##### ⭐⭐⭐ `.20` SLICE 2 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0034`, 2026-08-15 session #234) — ACCEPTANCE (a) FIRST PASS: cause (i) is CONFIRMED, and the counter's blindness is now EXPLAINED rather than merely measured

⛔ Tools-first, and the SPEED vocabulary the evidence gate already registers: `/usr/bin/sample` on a
**BARE** parse — deliberately, because a bare parse runs the FUSED `cascade_*` graph, which is the
graph the +24.3 % was measured on **and** the one slice 1's counters provably cannot see. A sampling
profiler is the only instrument in the toolbox that observes it.

###### ⭐⭐⭐ RESULT 1 — SELF-TIME **2 %**, INCLUSIVE **27 %**: THE LR MACHINERY DOES ALMOST NOTHING AND ROUTES ALMOST EVERYTHING

Measured on the heaviest corpus file (`pinmux_reg_top.sv`, 4.5 s bare parse, in the pinned sample),
sampled at 1 ms. ⛔ The denominator is the **parsing worker thread**, not the process: `sample`
reports two threads of 2 774 samples each and the main thread's are **all `__ulock_wait`** — idle,
waiting on the worker. Using the process total would have halved every percentage.

| | samples | % of worker thread |
|---|---:|---:|
| LR machinery **self**-time | 52 | **1.9 %** |
| LR machinery **inclusive** (subtree) | 757 | **27.3 %** |

⭐⭐ **The 14× gap between self and inclusive is the whole finding, and it RESOLVES slice 1's
tension.** Slice 1 measured the guarded-admission family at 0.681 % of rule ENTRIES and could not
explain how that squares with +24.3 % of TIME. It squares like this: the LR rules are **entered
rarely and cost almost nothing themselves**, but a quarter of the entire parse now runs *underneath*
them. A counter counts entries into cheap functions; the expense is the subtree. ⇒ the binding metric
is not merely "less sensitive" — it is measuring the one quantity this change deliberately did not
move.

###### ⭐⭐ RESULT 2 — CAUSE (i) IS CONFIRMED BY NAME, NOT BY PLAUSIBILITY

The outermost LR subtrees are dominated, by a wide margin, by a single function:

```
159, 57, 49, 30, 25, 21, 19, 18, 16, 16, 14, 12 …  samples
  SystemverilogParser::cascade_match_casting_type_lr_base
```

That is `.20`'s routing-evidence cause **(i)** verbatim — *"the eliminated `casting_type` … sits in
the expression hot path and every expression parse now traverses base+suffix instead of one rule"* —
now measured rather than hypothesised, and measured on the FUSED graph (`cascade_match_*`), i.e. on
the code the shipped parser actually executes.

###### ⭐⭐ RESULT 3 — THE CONTROL: A **PASSING** PARSE REPRODUCES IT, SO IT IS NOT A BACKTRACKING ARTIFACT

The profiled file is a corpus `fail` row, and a rejecting parse backtracks unusually — so the result
was re-run against the slowest **accepting** file (`t_math_synmul_mul.v`, 3.7 s, 1 832 worker
samples) before being believed:

| file | verdict | LR self | LR inclusive | ratio |
|---|---|---:|---:|---:|
| `pinmux_reg_top.sv` | fail | 1.9 % | **27.3 %** | 14.0× |
| `t_math_synmul_mul.v` | **pass** | 2.4 % | **27.9 %** | 11.6× |

Two independent files, opposite verdicts, different sub-corpora — and the inclusive share agrees to
within 0.6 points.

###### ⛔⛔ RESULT 4 — WHAT THIS DOES **NOT** PROVE, STATED BEFORE ANYONE READS 27 % AS 24 %

⛔ **`27.3 %` is NOT the +24.3 % delta, and the numerical proximity is a coincidence until an A/B
says otherwise.** Inclusive time beneath `casting_type_lr_base` includes all the work the *pre-flip*
parser also did — parsing the contents of a casting type is not new. What the flip changed is the
**path**, not the existence of the work. So this measurement locates the cost and identifies the
mechanism; it does not attribute the delta.

⇒ **This re-prices acceptance (b) as the next unit rather than more of (a).** A third arm with the
guard emission suppressed, profiled the same way, turns `27.3 %` into a *difference* — and the
sampling harness, the worker-thread denominator, the outermost-subtree attribution and the
accept/reject control are all now built and reusable, which is most of that slice's cost already
paid. Causes (iii) i-cache and (iv) memo remain unmeasured and are NOT claimed either way.

###### ⛔⛔⛔ RESULT 5 — A FOUNDATIONALLY SURPRISING SYMBOL, ROOT-CAUSED RATHER THAN CLASSIFIED AWAY: THE LINKER FOLDS GENERATED PARSERS TOGETHER

The profile of a **SystemVerilog** parse attributed samples to
`generated_parsers::rtl_frontend::RtlFrontendParser::cascade_error_from_parse` and
`…::byte_window_lossy`. A different family's parser cannot be running here, so this was chased
rather than absorbed.

`nm -C` on the binary answers it in one command:

| helper | symbols in binary | families present |
|---|---:|---|
| `cascade_error_from_parse` | **1** | `rtl_frontend` only |
| `byte_window_lossy` | 2 | `rtl_frontend`, `scratch` |
| `create_contextual_error` | 9 | 9 families incl. `systemverilog` |
| `memoized_call` | 516 | all |

There are **10 generated families in the binary** and exactly **one** `cascade_error_from_parse`. ⇒
**identical code folding**: the generated helpers are byte-identical across families because they
come from one codegen template, the linker merges them, and the surviving symbol name is arbitrary.
The samples are SystemVerilog's own work wearing another family's name.

⛔ **Not a defect — but a standing PROFILING-INTEGRITY hazard, and it belongs in the toolbox rather
than in this leaf's memory**: in this binary, *per-family attribution read off a generated symbol
name can be wrong*, and it fails silently and plausibly. Recorded in `TOOLBOX.md` 3.8.

✅ **And it was checked against THIS slice's own numbers rather than waved off**: the LR machinery is
**165 of 174** LR symbols and every `_lr_seed`/`_lr_guard` symbol is SystemVerilog's own, with only
9 LR symbols across two annotation families that do not run in an SV corpus parse. ⇒ folding does
**not** contaminate RESULT 1–3. Had that check gone the other way, the 27.3 % would have been
unusable.

###### Acceptance Checklist (enforced) — `.20` slice 2

- [x] **REPRODUCE / ISSUE** — the subject is `.20` acceptance (a): attribute the +24.3 % across
  causes (i)–(iv). Reproduced as an open question by slice 1's own measurement, which sharpened it
  rather than answering it: the guarded-admission family is **0.681 %** of corpus rule entries
  against a **+24.3 %** wall-clock cost, so the cost is per-entry and no counter can locate it.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the counter is blind and WHERE the time actually goes, one
  answer: `/usr/bin/sample` on a bare parse gives LR **self-time 1.9 %** against LR **inclusive
  27.3 %** (worker-thread denominator 2 774; the main thread is entirely `__ulock_wait`). The LR
  rules are cheap and are entered rarely — a quarter of the parse runs in their **subtree**. WHERE,
  to one function: **`cascade_match_casting_type_lr_base`** dominates every outermost LR subtree
  (159/57/49/30/25/… samples), which is routing-evidence cause **(i)** named exactly, and measured on
  the FUSED `cascade_*` graph the shipped parser executes. Call-graph attribution over 13 753 parsed
  nodes, counting only OUTERMOST LR nodes so nested frames cannot double-count.
- [x] **FIX** — ⛔ none, deliberately, and that is the slice's correctness: ruling C's *"before any
  optimisation is designed"* still binds and nothing here is an optimisation. Zero code, grammar,
  generated, script and gate bytes changed; this slice lands MEASUREMENT plus the `TOOLBOX.md` 3.8
  hazard the measurement uncovered. `.20` stays `todo`.
- [x] **ADDRESSED (verified)** — verified by an independent CONTROL rather than by repetition: the
  profiled file is a corpus `fail` row, so the whole result was re-run on the slowest **accepting**
  file, a different sub-corpus (`t_math_synmul_mul.v`, 1 832 worker samples) → LR self **2.4 %**,
  inclusive **27.9 %**, ratio 11.6×. Two files, opposite verdicts, inclusive share agreeing within
  **0.6 points**. ⛔ And the instrument itself was falsified before its numbers were used: `nm -C`
  proved the binary folds identical generated helpers across all 10 families (`cascade_error_from_
  parse`: **1 symbol, 10 families**), then proved that folding does NOT reach the LR symbols
  (**165 of 174** are SystemVerilog's own, every `_lr_seed`/`_lr_guard` among them) — so RESULT 1–3
  survive a hazard that would otherwise have invalidated them silently.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 19 enforced doctrines PASS**,
  including `PARSE-COST-RATCHET` whose identity tier re-hashes the grammar, the generated parser and
  the sample-input digest and finds all three unchanged — i.e. the parser this slice profiled is
  byte-identically the parser slice 1 baselined, gate-held rather than asserted. No executable byte
  of the product changed, so no behaviour can have moved.
- promotion: declined (RESULT 5's transferable half is registered as a toolbox hazard in
  `TOOLBOX.md` 3.8, which is where a profiling caveat is actually retrieved from; RESULT 1's lesson
  is already carried by `a-deterministic-counter-cannot-see-a-per-entry-cost-rise`, which this slice
  confirms rather than extends).


##### ⛔⛔ `.20` SLICE 3 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0035`, 2026-08-15 session #234) — SELF-AUDIT of slices 1-2, requested by the director; **STARTED, NOT FINISHED** — one defect found and fixed, and the audit is OPEN

⛔ Director directive: *"You should always double-check all your claims and make sota, signoff and
production-grade decisions."* Following the `README-POLICY.10` (`-0009`) precedent, in which
re-deriving `-0008`'s claims instead of restating them found **four** defects, **every one flattering**.
⚠️ **This slice was interrupted by an `/exit` and is INCOMPLETE.** It is recorded in this state
deliberately — an audit that stops silently is worse than one never started, because the unaudited
claims keep their apparent endorsement.

###### ✅ AUDITED AND HELD

| claim | re-derivation | verdict |
|---|---|---|
| LR family = **0.681 %** of corpus rule entries | re-summed from the 16 336-row census: 6 126 595 / 899 264 997 = **0.6813 %** | ✅ exact |
| `24.3 / 0.681` ⇒ **~35×** | **35.7×** | ✅ |
| the bound is published in 4 places | `grep '0.681'` → `cost.md` 1, `check_parse_cost_ratchet.sh` 1, `TOOLBOX.md` 1, `DOCTRINE_ENFORCEMENT.md` 1 | ✅ all four |

###### ⛔⛔⛔ DEFECT 1 — FOUND AND FIXED: I COMMITTED, IN THE DURABLE RECORD, THE EXACT ERROR SLICE 2 RESULT 4 WARNS AGAINST

`CHANGES.md` carried *"a quarter of the parse **now** runs underneath them"* and *"it measures the
one quantity **this change deliberately did not move**"*; `MEMORY.md` carried the same second clause.
Both assert a **pre/post change**. ⛔ **Every number in slices 1-2 was measured on the SHIPPED parser
alone.** There is no pre-flip entry count and no pre-flip profile arm — that is precisely what
acceptance (b) is for, and RESULT 4 of slice 2 says so two paragraphs later in the same file.

⇒ the record was **internally inconsistent**, and the inconsistency ran in the **flattering
direction**: it makes the explanation read as *established* when it is a hypothesis consistent with
the data. ✅ Corrected in place (not deleted — supersede-don't-mutate applies to the claim, and the
correction names what would prove it). The surviving statement is: in the shipped parser LR is ~2 %
self / ~27 % inclusive, and *whether the flip moved the subtree rather than the entry count is
consistent with both measurements but unmeasured*.

⭐ The transferable half: **two individually correct measurements can compose into an unproven
third**. Neither the 0.681 % nor the 27.3 % is wrong; the *causal join between them* was never
measured, and joining them read as a conclusion because both inputs were solid.

###### ⛔ OPEN — CLAIMS NOT YET RE-DERIVED (the next session starts here)

1. ⛔⛔ **The code-folding claim is INFERRED, not verified.** Slice 2 RESULT 5 concludes *identical
   code folding* from symbol COUNTS (10 families, 1 `cascade_error_from_parse`). That is consistent
   with ICF but was **not** tested against the live alternative — that SystemVerilog's copy was
   INLINED away, leaving `sample` to attribute to the nearest preceding symbol. Both produce the
   same symbol census. ⇒ **decide it with `otool` annotated disassembly** (a registered SPEED token):
   disassemble an SV caller and check whether the call target address IS the `rtl_frontend` symbol's
   address. Until then `TOOLBOX.md` 3.8's *mechanism* is provisional — its *operational advice*
   (do not trust per-family symbol attribution; check the symbols your conclusion rests on) holds
   under either mechanism.
2. **`165 of 174` LR symbols** — not re-derived since first measurement.
3. **`1.9 % / 27.3 %` and the `2.4 % / 27.9 %` control** — single runs each; sampling is stochastic
   and neither was repeated. ⚠️ A profile is not a deterministic oracle, and slices 1-2 nowhere state
   a confidence interval for them.
4. **"the LR rules are cheap and rare"** — *"rare"* is imprecise: 0.681 % of ENTRIES, but the census
   shows **73.5 % of corpus FILES** enter them. Rare per entry, ubiquitous per file.
5. **"no such flag exists today"** (the guard-suppression lever, sizing acceptance (b)) — checked by
   one `grep` over `main.rs` + `mod.rs`, not over `indirect_lr_elimination.rs` itself.
6. **"~22-minute release rebuilds"** — quoted from `MEMORY.md`, never measured in this session.

###### Acceptance Checklist (enforced) — `.20` slice 3

- [x] **REPRODUCE / ISSUE** — the director asked for every claim to be double-checked. Re-deriving
  rather than restating immediately reproduced an inconsistency inside one file: `CHANGES.md` asserts
  a pre/post change at line 11-13 and denies having measured one at line 25.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: slices 1 and 2 each measured the SHIPPED parser only, and
  the *join* of their two correct numbers into a causal story about the FLIP was never measured.
  WHERE: `CHANGES.md` (*"now runs underneath"*, *"the one quantity this change deliberately did not
  move"*) and `MEMORY.md` (the second clause). Located by grepping the durable record for the
  change-asserting phrasing, which is how it was established that the defect was NOT confined to a
  chat message.
- [x] **FIX** — fix-hierarchy tier = **claim correction in the durable record**, zero code/gate bytes.
  Both surfaces now state the shipped-parser fact and mark the causal reading as unmeasured, naming
  (b) as what would prove it. The remaining unaudited claims are enumerated above rather than left
  implicitly endorsed.
- [x] **ADDRESSED (verified)** — `grep -n "now runs underneath\|quantity this change deliberately did
  not move\|the one quantity the change did not move"` over `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md
  CHANGES.md DEVELOPMENT_NOTES.md MEMORY.md TOOLBOX.md` returns **no hits** after the fix (it
  returned 2 in `CHANGES.md` and 1 in `MEMORY.md` before). The three audited numeric claims re-derive
  exactly (table above).
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 19 enforced doctrines PASS**,
  `PARSE-COST-RATCHET` included, its identity tier re-hashing grammar + generated parser + sample
  digest unchanged. Documentation-only; no executable byte moved.
- promotion: declined (the durable lesson — *two individually correct measurements can compose into
  an unproven third* — is a sharper restatement of the failure already carried by
  `docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md`; promoting a second
  card before the audit that produced it is FINISHED would bank a lesson from an unfinished audit).


##### ✅ `.20` SLICE 4 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0036`, 2026-08-15 session #235) — THE SELF-AUDIT IS **FINISHED**: 6 of 6 open claims re-derived, slices 1-2 SURVIVE, and the one defect found was **in the auditor's own instrument**

⛔ Slice 3 stopped mid-audit and said so, which is the only reason this could be resumed honestly.
Every one of its six enumerated open claims is now re-derived from tool output rather than restated.

⭐⭐ **The headline is not what the audit expected.** Slices 1-2's published numbers hold. The single
material defect uncovered belongs to **this slice's own re-derivation instrument**, which produced an
**8× wrong** headline while its ground-truth control stayed green — and a draft finding accusing
slice 2 of a defect was written, and is retracted below rather than deleted.

###### SCOREBOARD — 6 of 6 CLOSED

| # | slice 3's open claim | verdict |
|---|---|---|
| 1 | code folding is ICF, not inlining — *"INFERRED, not verified"* | ✅ **VERIFIED, and the mechanism is now pinned to the LINKER** |
| 2 | `165 of 174` LR symbols | ✅ exact |
| 3 | `1.9 % / 27.3 %` + the `2.4 % / 27.9 %` control — single runs, no interval | ✅ **self REPRODUCES** over 8 runs · ⚠️ **inclusive is 0.5-3.0 points HIGH** and the *"0.6 points"* agreement was LUCK |
| 4 | *"cheap and rare"* — 0.681 % of entries, 73.5 % of files | ✅ **0.6813 %** and **73.50 %**, both exact |
| 5 | *"no such flag exists today"* — checked by one grep | ✅ **VERIFIED and widened — the source ARGUES AGAINST adding one** |
| 6 | *"~22-minute release rebuilds"* — quoted, never measured | ✅ **exact on both axes: 1 324 s = 22.07 min, peak 12 281 MB = 12.0 GB** |

⭐ Plus **three findings the audit list did not contain**, all from re-deriving rather than
restating. N1 re-prices a bound this repository publishes in **four** places.

###### ⭐⭐⭐ RESULT 1 — ITEM 1 CLOSED: IT IS THE **LINKER**, PROVEN THREE WAYS, AND THE INLINING ALTERNATIVE IS REFUTED

Slice 3 was right to refuse slice 2's symbol-count argument: *10 families, 1 symbol* is equally
consistent with **ICF** and with **SystemVerilog's copy having been INLINED away**, leaving `sample`
to attribute to the nearest preceding symbol. The two hypotheses make **opposite** predictions about
the call graph, so the decisive evidence is the call graph — `otool` annotated disassembly, exactly
as slice 3 specified:

```
$ otool -tV -p '__ZN4pgen17generated_parsers13systemverilog19SystemverilogParser33cascade_kw_timeprecision_628f1cfc17heff5c4972c1a3912E' \
      rust/target/release/parseability_probe
0000000100c9b3f4  mov  x2, x22
0000000100c9b3f8  bl   __ZN4pgen17generated_parsers12rtl_frontend17RtlFrontendParser24cascade_error_from_parse17h401e57134db20f5aE
0000000100c9b3fc  ldur x8, [x29, #-0x98]
```

A **SystemVerilog** rule method makes a real `bl` into the address that carries the **`rtl_frontend`**
symbol. Inlining-away predicts no such call exists. Three independent legs:

| leg | measurement | what it settles |
|---|---|---|
| **call site** | `otool -tV -p` on the SV caller ⇒ `bl` → `rtl_frontend::…::cascade_error_from_parse` | SV *calls* the shared body; it was not inlined |
| **call-site census** | a `BL`-opcode scan of all **65 692 604 B** of `__text` finds **2 505** call sites targeting `0x103bdf850`, of which **1 964 are inside `generated_parsers::systemverilog::` methods** (regex 201, vhdl 106, semantic_annotation 91, rtl_frontend 47, sv_preprocessor 41, rtl_const_expr 30, json 15, return_annotation 10) | nine families share ONE body — not an artifact of one call |
| ⭐⭐ **pre-link** | the un-linked `libpgen-636e6a229e44ae4e.rlib` carries **9 distinct per-family symbols**, SV's own among them: `…13systemverilog19SystemverilogParser24cascade_error_from_parse17hbd2a47f7dfbb49a9E` | the **compiler emitted 9 copies and the binary has 1** ⇒ the fold happened at LINK time |

⇒ `TOOLBOX.md` 3.8's mechanism sentence (*"the linker merges them"*) is **earned, not inferred**, and
its operational advice was already safe under either mechanism. RESULT 5's table re-derives exactly —
`cascade_error_from_parse` **1**, `byte_window_lossy` **2**, `create_contextual_error` **9**,
`memoized_call` **516**. ⚠️ One refinement: the binary's ten generated families are json, regex,
return_annotation, rtl_const_expr, rtl_frontend, **scratch**, semantic_annotation, systemverilog,
systemverilog_preprocessor, vhdl — i.e. **`ebnf` is NOT among them and the toolbox `scratch` slot
IS**. Slice 2's *"10 generated families"* is right as a count and its composition is now recorded.

###### ⛔⛔⛔ RESULT 2 — ITEM 3, AND THE DEFECT IS **MINE**: A GREEN GROUND-TRUTH CONTROL OVER AN 8×-WRONG NUMBER

**8 fresh profiles** (5 on the `fail` file, 3 on the `pass` control), 1 ms sampling, TOOLBOX 3.8's
three rules applied — worker-thread denominator, OUTERMOST-only inclusive, trap-3 checked (all
**6 801** LR frames across all 8 reports are `generated_parsers::systemverilog::`, zero folding
contamination).

⛔ **The first version of the re-derivation reported LR self-time at 18.12 % against slice 2's
1.9 %, and I had drafted that as a DEFECT IN SLICE 2. It was a defect in my parser.** A `sample`
report has **four** sections and only the first is a call graph; the parser stopped at
`Sort by top of stack` and `Binary Images` but **not** at
`Total number in stack (recursive counted multiple, when >=5)`, so 694 lines of a different table
were ingested as call-graph rows, re-parenting **14 022** samples.

⭐⭐ **And the ground-truth control passed the whole time, necessarily.** The control was
`sum(self) == worker root count` — a **conservation** identity. The bug was a **misassignment**:
samples wrongly subtracted from one node's self-time reappear as the mis-parented rows' own
self-time, so the total is identical either way. What caught it was an **external oracle** —
`sample`'s own `Sort by top of stack` per-symbol table — where **2 of 195** symbols disagreed, one
with a self-time of **−14 015**. Promoted: [[a-conservation-control-cannot-catch-a-misassignment]].

⇒ the instrument now REFUSES (exit 2) on five controls, and the binding one is external: **C4 —
every symbol `sample` itself lists must match this script's independently-derived self-time**
(139-209 symbols per report, all matching). C3 (`self < 0` refuses) is free and would alone have
caught it.

**Corrected measurement, n = 8:**

| arm | n | LR self % | LR inclusive % | ratio | slice 2 published |
|---|---:|---|---|---|---|
| `pinmux_reg_top.sv` (fail) | 5 | **1.83-2.35** (mean 2.16) | **23.67-26.75** (mean 25.27) | 10.4-13.6× | self 1.9 % · incl 27.3 % · 14.0× |
| `t_math_synmul_mul.v` (pass) | 3 | **2.07-3.60** (mean 2.60) | **22.38-24.86** (mean 23.49) | 6.9-11.0× | self 2.4 % · incl 27.9 % · 11.6× |

- ✅ **The SELF half reproduces.** Both published values sit inside the measured band.
- ⚠️ **The INCLUSIVE half is optimistic by 0.5-3.0 points.** `27.3 %` is above the top of a 5-run
  band whose maximum is `26.75 %`; `27.9 %` is 3.0 points above a 3-run maximum of `24.86 %`.
- ⛔ **The *"agreeing within 0.6 points"* control claim was LUCK, and this is the sharper correction.**
  The within-file spread alone is **3.1 points** (fail, inclusive) — wider than the between-file gap
  the claim was celebrating. A single run per arm cannot support a 0.6-point agreement claim, and
  slices 1-2 nowhere state that sampling is stochastic.
- ⇒ **the durable form is `LR self ≈ 2 % (1.8-3.6), inclusive ≈ 24 % (22-27), ratio ≈ 11× (7-14),
  n=8`** — and slice 2's CONCLUSION (cheap themselves, expensive subtree) **survives intact**.
- ✅ Cause (i) survives too: `cascade_match_casting_type_lr_base` remains the dominant outermost LR
  subtree in every one of the 8 reports.

###### ⛔⛔⛔ RESULT 3 — **NEW (N1)**: THE TRACKED INSTRUMENT'S "GUARDED-ADMISSION FAMILY" OMITS **75.1 %** OF THE PASS'S OWN ENTRIES, AND THE `~35×` BOUND IS REALLY `~8.9×`

`stimuli/sv/corpus_parse_cost.py:100` defines the family as
`LR_FAMILY_RE = _lr_base$|_lr_suffix(_r\d+)?$`. The same elimination pass **also** emits
`*_lr_seed_*` and `*_lr_guard*` rules, and none of them matches. Declared in
`generated/systemverilog_parser.rs`: **128** LR rule names, of which the predicate matches **97**,
leaving **24 seed + 6 guard = 30 uncounted** (the 128th, `_pgen_lr_chain_alt`, is the internal
discriminator `.8` removed from the typed AST, not a family member; the three classes are provably
disjoint — 0 overlap). A full-corpus per-rule census (16 335 files) prices what that costs, over the
**73** of them a real corpus actually enters:

| family member | rules | entries | share of corpus entries | committed |
|---|---:|---:|---:|---:|
| `_lr_base` / `_lr_suffix` — **what the instrument counts** | 51 | 6 125 716 | **0.6813 %** | 4 240 |
| `_lr_seed` — **uncounted** | 16 | 15 140 142 | **1.6840 %** | 12 755 |
| `_lr_guard` — **uncounted** | 6 | 3 378 577 | **0.3758 %** | 2 874 |
| **the complete family** | **73** | **24 644 435** | **2.7411 %** | 19 869 |

⇒ the predicate misses **18 518 719 of 24 644 435 entries = 75.1 %** of the machinery it names.
⛔ **A predicate that claims to measure the GUARDED admission does not count a single guard rule** —
`casting_type_lr_guard1` alone is **1 374 769 entries / 2 214 committed**.

⭐⭐ **Two independent instruments agree on the size of the miss.** The profile (RESULT 2) says the
narrow predicate sees **0.54 %** of a **2.27 %** LR self-time, i.e. it misses **76 %** of the
machinery's own CPU; the corpus census says it misses **75.1 %** of its entries. Different tools,
different quantities, same answer.

⛔ **The consequence is a published number, in four places.** The blind-spot bound
`24.3 / 0.681 = ~35×` becomes `24.3 / 2.741 = **~8.9×**` — the ratchet is **four times less blind**
than `cost.md`, `scripts/check_parse_cost_ratchet.sh`, `TOOLBOX.md` 3.7 and
`DOCTRINE_ENFORCEMENT.md` §10 all currently state. ⚠️ And the direction is the one this leaf keeps
finding: under-counting the family **under-states the guarded admission's own footprint**, which is
flattering to the change under audit. That is the third flattering error in three audits.
⇒ **ROUTED to `.21`** (below) — a fix touches a tracked instrument, the pinned sample's `lr` tier
selection, the baseline and four published surfaces, so it is a slice of its own, not a footnote.

###### ⛔⛔⛔ RESULT 4 — **NEW (N2)**: THE OUTCOME DUMP HANGS ON A CORPUS FILE A BARE PARSE HANDLES IN **0.077 s**, AND SLICE 1'S *"ZERO NO-DUMP ROWS"* DOES NOT REPRODUCE

The re-run census returned **16 335 rows and 1 no-dump**, against slice 1's *"16 336 files … **zero**
no-dump rows"* and *"16 336/16 336 agree"*; its accepted count is **9 773**, not the published
**9 774**. The single missing file was chased rather than absorbed:

`stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv` — **2 787 bytes**, recorded `pass` at
**0.07 s** in `stimuli/sv/characterization/durations.tsv`.

| how it is parsed | graph | result |
|---|---|---|
| BARE `--parse` | FUSED `cascade_*` | ✅ **0.077 s**, `parse_full passed` |
| `--dump-rule-entry-counts-json` (TOOLBOX **3.4**) | PROTOCOL, **no** coverage stack | ✅ **0.062 s**, dump written, `accepted: True`, **200 975 entries** over 711 rules |
| `--dump-rule-outcome-counts-json` (TOOLBOX **3.5**) | PROTOCOL **+ transactional coverage stack** | ⛔ ~150 MB/s allocation, **peak 4 682 MB in 30 s**, terminated in **5 of 5 attempts** (23 s, 27 s, 30 s, 59 s, plus the census's own 120 s timeout) — **never produced a dump** |

⭐⭐ **The isolation is exact: 3.4 and 3.5 take the SAME graph, and only 3.5 diverges** ⇒ the
pathology is in the **transactional coverage stack**, not in protocol-graph routing. And the file is
not big: **200 975 entries** is 19× the corpus median (10 542) and **175× below** the corpus maximum
(35 107 691) — a file with 175× more work is measured successfully.

⛔ **This falsifies the basis of slice 1 RESULT 3's promotion of 3.4 → 3.5**, which read *"measured,
the outcome dump costs the SAME 0.04 s on the same file"* — **one file**. There is at least one file
on which 3.5 costs **≥ 480×** more and never terminates.
⚠️ Honest bound: whether that parse would eventually terminate given unbounded RAM is **NOT
measured** — every attempt was killed by the environment (memory-guard marker `reason=none`,
`peak_rss_mb=4682`, so the guard itself did not kill it).
⇒ **ROUTED to `.22`** (below).

###### RESULT 5 — ITEMS 2, 4, 5, 6, EACH RE-DERIVED

- **Item 2 — `165 of 174`** ✅ exact:
  `nm -C … | grep -E '_lr_(base|suffix|seed|guard)'` ⇒ **165** `systemverilog`, **6**
  `return_annotation`, **3** `semantic_annotation`. The 9 non-SV symbols cannot run in an SV corpus
  parse, and RESULT 2 confirms none appeared in 8 profiles.
- **Item 4 — *"cheap and rare"*: both numbers exact, the ADJECTIVES now split.** Re-run census:
  **0.6813 %** of entries (published `0.681 %`) and **12 007 / 16 335 = 73.50 %** of files (slice 3
  claimed `73.5 %`); `24.3 / 0.6813 = 35.7×`. ⇒ *rare per entry, ubiquitous per file* holds exactly.
  ⛔ *"cheap"* is the word that does not: RESULT 3 shows the measured family is **4×** larger than
  the instrument's, so what was priced as cheap was priced on a quarter of itself.
- **Item 5 — *"no such flag exists today"*** ✅ **verified, and the widened grep matters for (b)**.
  Zero `env::var` / `std::env` / `"PGEN_` in **any** of `indirect_lr_elimination.rs`,
  `indirect_lr_plan.rs`, `lr_chain_fold.rs`. `plan_guard_chains` has exactly one non-test call site
  — `indirect_lr_elimination.rs:926` — and it is unconditional. The only LR-related CLI flag
  repo-wide is `--eliminate-left-recursion` (all-or-nothing); the only LR env var is
  `PGEN_INDIRECT_LR_DUMP_ALL`, a reporting knob. ⛔⛔ **And the source ARGUES AGAINST the switch (b)
  needs**, in a comment at that call site: *"Unconditional, and NOT behind an admission check,
  deliberately … a second switch here would be a second thing that has to agree with the criterion,
  and the two could drift."* ⇒ **(b) cannot be a permanent admission flag.** It must be a
  measurement-only lever — a temporary local patch, or a generation-time dry-run path — and slice 3's
  one-grep claim under-stated the constraint rather than the cost.
- ✅ **Item 6 — *"~22-minute release rebuilds"*: measured, and the quoted figure is EXACT on both
  axes.** The rebuild that matters is the one acceptance (b) will pay — touch
  `generated/systemverilog_parser.rs`, then
  `cargo build --release --features generated_parsers --bin parseability_probe` under the memory
  guard. Marker: **`exit=0 peak_tree_rss=12281MB elapsed=1324s`** ⇒ **22.07 min / 12.0 GB**, against
  `MEMORY.md`'s *"≈22 min / peak 12.0 GB"*. 0 rustc errors. ⭐ Verified to be a pure recompile rather
  than a regeneration: `generated/systemverilog_parser.rs` md5 is `09b8cc21…` before **and** after,
  so only the mtime moved. ⇒ **(b) costs ≈22 min of rebuild per arm on this machine, and it needs at
  least two** (guard-suppressed generation, then the restore) — plus a corpus run each. That is the
  number to plan (b) against, and it is now measured rather than remembered.

###### ⭐⭐ RESULT 6 — **NEW (N3)**, DIRECTOR-ASKED MID-SLICE (*"LR elimination support is still an issue?"*): NO — AND `TOOLBOX.md` 5.1 STILL SAID YES

Answered with the tool rather than from the record. `--lint-grammar` over **all 17 tracked grammars**:

| population | `left_recursion_unhandled` |
|---|---|
| **all 10 shipped parser families** | **0** — SV `unhandled=0 eliminated=2`; `return_annotation` / `semantic_annotation` `eliminated=1`; the rest `0/0` |
| `systemverilog_lrm_profiled_wrapper` (⛔ **not a family** — `lrm_extraction_harness`) | **23** (also 42 profile-orphans, 23 shadowed branches, 5 undefined refs; never generated a shipped parser) |
| the 4 other non-family grammars (LRM extractions + the derived artifact) | lint exits on prior error classes before the LR headline |

⇒ **LR elimination is CORRECT and CLOSED for everything PGEN ships.** `.17` slice 9 took SV
`28 → 0`. What remains open is its **cost**, which is this leaf — feature done, invoice outstanding.

⛔ **The live doc disagreed.** `TOOLBOX.md` 5.1 published *"Current: SV **30**,
`systemverilog_lrm_profiled_wrapper` **23**, `ebnf` **5**, every other grammar **0**"* — the PRE-flip
state, stale in **two of its three numbers** (SV is 0, `ebnf` is 0) since `.17` slice 9 landed on
2026-08-14. ⚠️ Nothing detected it: `LIVE-DOC-CURRENCY` leg A counts DISTINCT DATES in a surface and
leg B is dormant, so neither can see a *number* going stale inside a sentence that was never dated.
Corrected in place in this commit, and the corrected line now names its derivation and its date so
the next reader can re-run it. ⇒ the generalisable gap — *a live doc can publish a stale MEASUREMENT
without publishing a stale DATE* — is routed to `LIVE-MEANS-LIVE` rather than fixed here.

###### ⚠️ WHAT THIS SLICE DELIBERATELY DOES NOT DO

- It does not fix N1 or N2 — both are routed to their own leaves with their own acceptance.
- It does not touch acceptance **(b)**, **(c)** or **(e)**. ⛔ **`.20` stays `todo`**, and ruling
  clause **B** — no waiver, conditional tenancy — is unchanged and still binds.
- ⭐ It DOES change (b)'s shape: item 5 proves the lever must be measurement-only, and RESULT 3 means
  (b)'s "subtraction rather than re-derivation" shortcut (slice 1) subtracts the **wrong family**
  until `.21` lands. ⇒ **`.21` before (b)**.

###### Acceptance Checklist (enforced) — `.20` slice 4

- [x] **REPRODUCE / ISSUE** — slice 3 recorded itself as *"STARTED, NOT FINISHED"* with six named
  claims *"NOT YET RE-DERIVED"*, and warned that *"an audit that stops silently is worse than one
  never started, because the unaudited claims keep their apparent endorsement."* Reproduced as an
  open state, not a symptom: those six claims carried slices 1-2's endorsement without re-derivation.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY each open claim was open and WHERE the answer is, one leg
  per claim. Item 1 — `otool` annotated disassembly of the SV caller shows
  `bl __ZN4pgen…rtl_frontend…cascade_error_from_parse…`, plus a `BL`-opcode scan of `__text`
  attributing **1 964 of 2 505** call sites to `generated_parsers::systemverilog::`, plus **9**
  pre-link per-family symbols in the rlib ⇒ the LINKER folds, inlining refuted. Item 3 — `/usr/bin/sample`
  ×8 with **call-graph attribution** on the worker thread gives **self-time 1.83-3.60 %** against
  inclusive 22.38-26.75 %; ⛔ and the WHY of this slice's own 8× error is located exactly: `sample`
  emits four sections and the parser did not stop at `Total number in stack …`, re-parenting 14 022
  samples, which a conservation control cannot see and an external per-symbol oracle caught at
  **2 of 195** symbols. N1 — `stimuli/sv/corpus_parse_cost.py:100`. N2 — isolated to the
  transactional coverage stack, because TOOLBOX 3.4 and 3.5 take the same graph and only 3.5 diverges.
- [x] **FIX** — fix-hierarchy tier = **claim correction + measurement in the durable record**; ZERO
  code, grammar, generated, script and gate bytes. Slices 1-2's surviving claims are restated with
  their measured intervals; the two that do not survive (inclusive optimistic by 0.5-3.0 points; the
  *"0.6 points"* control agreement) are corrected in place; the drafted-and-false *"slice 2 is
  defective"* finding is recorded as retracted rather than deleted. N1 and N2 are given owning leaves
  rather than being noted.
- [x] **ADDRESSED (verified)** — verified by INDEPENDENT re-execution, not by restatement.
  Deterministic legs re-derive exactly: `0.6813 %` vs published `0.681 %`, `73.50 %` vs `73.5 %`,
  `35.7×`, `165/6/3`, and the folding table `1 / 2 / 9 / 516`. Stochastic legs are re-run **8 times**
  and reported as intervals, which is what slices 1-2 lacked. ⛔ The re-derivation instrument was
  itself falsified before its numbers were used: its first version was **8× wrong** (18.12 % vs
  2.27 %) with a GREEN control, and it now refuses on five controls whose binding leg is external
  agreement with `sample`'s own table on **139-209 symbols per report**, all matching.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 19 enforced doctrines PASS**,
  `PARSE-COST-RATCHET` included: its identity tier re-hashes the grammar, the generated parser and
  the sample-input digest and finds all three unchanged, so the parser audited here is byte-identically
  the parser slices 1-2 measured — gate-held rather than asserted. Documentation-only; no executable
  byte of the product moved. `make -C rust SHELL=/bin/bash mdbook_docs_gate` passes.
- promotion: `docs/knowledge/a-conservation-control-cannot-catch-a-misassignment.md` (RESULT 2).


#### ⚠️ `.21` `in progress` — the parse-cost instrument's "guarded-admission family" counted 97 of the **127** LR rules the parser declares — no `_lr_seed`, and no `_lr_guard` at all — and so missed **75.1 %** of their corpus entries (opened 2026-08-15 session #235 by `.20` slice 4; ✅ **(a)/(c)/(d) DISCHARGED by slice 1** `PGEN-ENGINE-UNIVERSAL-SERVICES-0037`; ✅ **(g) DISCHARGED by slice 2** `PGEN-ENGINE-UNIVERSAL-SERVICES-0039`; ⛔ **(b) MEASURED and BLOCKED on `.22`**; ⏳ (e)/(f) open)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **Measured, full-corpus, per rule** (16 335 files): the complete elimination family is **73 rules /
  24 644 435 entries / 2.7411 %** of corpus entries; `LR_FAMILY_RE` at
  `stimuli/sv/corpus_parse_cost.py:100` matches **51 rules / 6 125 716 entries / 0.6813 %**. Missed:
  **18 518 719 entries = 75.1 %**, in **16** `_lr_seed` rules (1.6840 %) and **6** `_lr_guard` rules
  (0.3758 %). ⭐ Two populations, both stated so neither is over-read: **declared** in
  `generated/systemverilog_parser.rs` the split is **97 matched / 24 seed + 6 guard uncounted** of
  128 LR rule names (0 overlap between the three classes); **entered** by the real corpus it is
  **51 / 16 + 6** of 73.
- **A second, independent instrument agrees on the size of the miss**: on 8 `/usr/bin/sample`
  profiles the narrow predicate accounts for **0.54 %** of a **2.27 %** LR self-time — a **76 %**
  miss against the census's **75.1 %**. Different tool, different quantity, same answer.
- ⛔ **It reproduces outside SystemVerilog by construction**: the predicate is a property of the
  codegen's rule-naming scheme, not of SV. Any family whose knot is guard-feasible emits the same
  `_lr_seed_*` / `_lr_guard*` names and would be mis-priced identically. SV is simply the only
  family with an absorbed knot today.
- ⚠️ **The instrument's own controls do not cover it, and they look like they do.** `_self_check()`
  pins ten cases including the near-miss `something_lr_baseline` and the replaced rule
  `casting_type`; **not one of the ten is a seed or a guard rule**, so a control suite that refuses
  on an unclassified name passes with three quarters of the family unclassified.

**Acceptance:** (a) extend the predicate to the rules the pass actually emits, with the ten existing
controls kept and seed/guard positives added — ⛔ derived from `indirect_lr_elimination.rs`'s emission
sites, not from grepping today's generated parser, or the next emitted shape is missed the same way;
(b) re-derive the pinned sample, whose `lr` tier is *"the 40 heaviest by guarded-admission entries"*
and was therefore ranked on a quarter of the family — and state how many of the 40 change;
(c) re-publish the blind-spot bound **`~35×` → `~8.9×`** in all four surfaces that carry it
(`cost.md`, `scripts/check_parse_cost_ratchet.sh`, `TOOLBOX.md` 3.7, `DOCTRINE_ENFORCEMENT.md` §10)
plus the `a-deterministic-counter-cannot-see-a-per-entry-cost-rise` card, whose whole argument is
built on the `0.681 %` figure; (d) re-baseline, since the binding counters do not move but the
reported family does — and the identity tier must FAIL first, proving it noticed.

⛔ **Do NOT read this as "the ratchet was wrong".** The BINDING metric is total rule entries,
committed entries and memo hits — none of which is affected. What is wrong is the **reported family
share**, and therefore the **published statement of how blind the gate is**. The gate under-claimed
its own sensitivity by 4×.

##### ⛔⛔⛔ `.21` SELF-ASSESSMENT (director-asked 2026-08-15, *"were there sota, signoff …"*) — SLICE 1's DIAGNOSIS IS SIGNOFF-GRADE; ITS **DURABILITY IS NOT**, AND BOTH GAPS ARE CONFIRMED BY COMMAND

⛔ Answered with the tool rather than by re-reading my own prose, because *"a classifier written from
the design's prose can only confirm the design's prose"* is this very leaf's root cause and the same
trap applies to a self-assessment. **Both gaps reproduce:**

```
$ git check-ignore -q rust/target/audit_scratch/lr_attribute.py && echo IGNORED
IGNORED                       # …and family_census.py, bl_callers.py, sample_impact.py
$ ls rust/target/audit_scratch/*.txt | wc -l ; git ls-files rust/target/audit_scratch/ | wc -l
9                             # the 8 profile reports + 1
0                             # tracked: NONE
$ grep -rln "CORPUS_FAMILY_SHARE_PCT" --include=*.sh --include=*.py --include=*.yml .
stimuli/sv/corpus_parse_cost.py                    # only the file that DEFINES it
$ git ls-files 'scripts/*.sh' 'rust/scripts/*.sh' '.github/workflows/*.yml' 'rust/Makefile' \
    '.githooks/*' | xargs grep -ln -- '--census'
                              # nothing. no tracked runner invokes the census at all
```

**GAP 1 — ⛔⛔ THE INSTRUMENTS THAT PRODUCED THESE NUMBERS ARE UNTRACKED, SO THE NUMBERS ARE
UNREPRODUCIBLE.** `lr_attribute.py` (five ground-truth controls, including the external per-symbol
oracle that caught an 8× error), `family_census.py`, `bl_callers.py` and `sample_impact.py` all live
in `rust/target/audit_scratch/` — gitignored. So do the 8 profile reports. ⇒ `.20` slice 4's
`1.83-3.60 % / 22.38-26.75 %` intervals and `.21` slice 1's `24 of 192` sample-impact figure cannot
be re-derived by anyone, including me after a `cargo clean`. ⛔ **This is the same class of defect as
`.22`** — a measurement whose producer is not durable — committed one turn after writing a knowledge
card about validating instruments. `GATE-REACHABILITY`'s founding sentence applies verbatim: *a check
nothing invokes is indistinguishable from a check that does not exist.*

**GAP 2 — ⛔⛔ THE CORRECTED CONSTANT HAS EXACTLY THE PROPERTY THAT MADE THE OLD ONE ROT.**
`CORPUS_FAMILY_SHARE_PCT = "2.741"` is hand-carried, referenced only by the file that defines it, and
guarded by a **comment** (*"⛔ Re-derive it with a full-corpus census … NOT by editing this line"*).
A comment is prose, and `DOCTRINE_ENFORCEMENT.md` §1 is that a rule nothing checks is a suggestion.
The census costs **71 s** measured, so cheapness is not the excuse. ⇒ I replaced a wrong unwatched
number with a right unwatched number, and the corrected `~8.9×` will go stale the next time the
grammar moves, silently, exactly as `0.681 %` did.

**GAP 3 — ⚠️ `_lr_alt` IS COVERED ON THE EMITTER'S AUTHORITY BUT NEVER OBSERVED.** Its positive
control `expression_lr_alt1` is a string I typed from reading `mod.rs:3244`, not a name any parser
has emitted — SV has 0. A mis-read of that `format!` would reproduce as a passing control.

**GAP 4 — ⚠️ I OVERSTATED INDEPENDENCE.** *"Two independent instruments agree, 75.1 % vs 76 %"* —
the QUANTITIES are independent (corpus entries vs CPU samples); the **classifier is shared**, and it
is the classifier that was defective. The agreement therefore bounds far less than it reads.

**GAP 5 — ⚠️ THE OTHER NINE FAMILIES WERE NEVER CHECKED.** The emission shapes are engine-universal
and `return_annotation` / `semantic_annotation` each carry `left_recursion_eliminated=1`; whether
their emitted names match the new predicate is unmeasured.

⇒ **ADDED TO `.21` ACCEPTANCE, and the leaf STAYS `in progress`:** (e) promote the audit instruments
+ their profile artifacts into tracked paths, or delete the claims that depend on them — a measured
number whose producer is untracked is a *"trust me"*; (f) make the corpus family share **DERIVED or
GATED**, not carried — the census is 71 s, so a `--verify-family-share` mode re-run on demand (and a
tracked artifact the way `SV-CORPUS-DENOMINATOR` does it) is affordable; (g) observe `_lr_alt` for
real by generating a parser from a directly-left-recursive grammar, and run the predicate over all
ten families' declared rule names.

⭐ **What DOES hold**, so the assessment is not uniformly negative: the root cause was derived from
the emission sites rather than the artifact; the RED probe (8/21) proves the new controls are
non-vacuous; the reporting-only claim is gate-held by byte-identical BINDING counters rather than
asserted; and the `instrument` identity input was proven by firing (exit 1 → exit 0). The
**diagnosis** met the bar. The **durability** did not.

##### ✅ `.21` SLICE 1 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0037`, 2026-08-15 session #235) — (a), (c), (d) DISCHARGED; (b) MEASURED and then DELIBERATELY NOT ADOPTED, because it is blocked on `.22`

###### ⭐⭐⭐ RESULT 1 — (a): THE PREDICATE IS NOW DERIVED FROM THE EMISSION SITES, AND THERE ARE **EIGHT** SHAPES ACROSS **TWO** ELIMINATORS

The old predicate was written from the leaf's own prose — `X := X_lr_base ( X_lr_suffix )*` — and
that is exactly the error: it described the *shape the author had in mind*, not the shape the code
emits. Read off the emitters instead:

| emitter | line | shape | old predicate |
|---|---|---|---|
| `indirect_lr_elimination.rs` | 915 | `{base}_lr_base` | ✅ |
| | 916 | `{base}_lr_suffix` | ✅ |
| | 862 | `{base}_lr_suffix_r{index}` | ✅ |
| | 1018 | `{base}_lr_seed_{rule}` | ⛔ **missed** |
| | 1188 | `{base}_lr_guard{variant}` | ⛔ **missed** |
| | 1190 | `{base}_lr_guard{variant}_suffix` | ⛔ **missed** |
| | 1241 | `{base}_lr_guard{variant}_{hop}` | ⛔ **missed** |
| `ast_pipeline/mod.rs` (the **DIRECT** pass) | 3244 | `{rule}_lr_alt{n}` | ⛔ **missed** |
| | 3347 / 3349 | `{rule}_lr_base` / `{rule}_lr_suffix` | ✅ |

⛔⛔ **And a NINTH hole nobody had named: both allocators append `_{index}` on a name COLLISION**
(`indirect_lr_elimination.rs::allocate`, `mod.rs::allocate_synthetic_rule_name`), so the
END-anchored `_lr_base$` drops `X_lr_base_1` silently. The new predicate therefore anchors on a
**segment boundary**, not on end-of-string: `_lr_(base|suffix|seed|guard|alt)(?![a-z])`. The
negative lookahead is what still refuses the pinned near-miss `something_lr_baseline`, which was the
one thing the original reasoning got right and is kept.

⭐ `_lr_alt` is **0 rules / 0 entries in SV today** — included anyway, because the predicate must
describe the emitter, not this month's grammar. That is the whole lesson of the defect.

###### ⭐⭐ RESULT 2 — THE CONTROLS WERE THE REAL FAILURE, AND THE RED PROBE PROVES THE NEW ONES ARE NOT VACUOUS

The instrument already had a ten-case control suite that **refuses rather than publishing** on any
miss — and it passed throughout, because **all ten cases were drawn from the `_lr_base`/`_lr_suffix`
half**. A suite that refuses on an unclassified name cannot help when the missing names were never
imagined. ⇒ the controls now enumerate the **producer's** shapes: one positive per emission site,
two collision-suffix positives, and four near-miss negatives.

⭐ **RED PROBE — run the NEW 21-case suite against the OLD predicate: `8 of 21` MISS**, every one in
the should-match-but-does-not direction (`…_lr_seed_constant_primary_sv_2017`, `…_lr_guard1`,
`…_lr_guard0_suffix`, `…_lr_guard0_constant_primary`, `expression_lr_alt1`, `…_lr_base_1`, …). The
new controls would have caught the defect on day one; the old ones could not have.

###### ⭐⭐⭐ RESULT 3 — (d): THE BINDING COUNTERS DO NOT MOVE, AND THAT IS THE POINT

Re-baselined on the **same pinned 192-file sample**:

| metric | before | after | verdict |
|---|---:|---:|---|
| **rule entries** | 416,841,264 | 416,841,264 | ✅ **byte-identical (BINDING)** |
| **committed** | 7,124,616 | 7,124,616 | ✅ **byte-identical (BINDING)** |
| **memo hits** | 186,981,263 | 186,981,263 | ✅ **byte-identical (BINDING)** |
| `lr_entries` | 3,092,966 | **12,440,690** | ×4.02 |
| `lr_committed` | 127 | **514** | ×4.05 |
| family share of the sample | 0.742 % | **2.985 %** | — |

⇒ **no cost claim in this tree moves.** The ratchet's history is intact, the sample is untouched,
and what changed is only the number describing *how much of the machinery the report was counting*.
Slice 1's *"pure speculation"* conclusion survives: **514 of 12 440 690 = 0.004 %** committed.

###### ⭐⭐ RESULT 4 — THE INSTRUMENT IS NOW PART OF ITS OWN BASELINE'S IDENTITY, AND THE GATE PROVED IT NOTICED

⛔ The identity block named three inputs — grammar, generated parser, sample bytes — on the sound
argument that the BINDING counters are an exact function of exactly those. Sound, and **insufficient**:
`entries.tsv` also publishes `lr_entries`/`lr_committed` and `cost.md` a family share, and those are
functions of the **instrument's own classifier**. Correcting it staled every published family number
while the identity tier printed `fresh`. That is `SV-CORPUS-GRAD.13i`'s defect one input short
instead of one gate short. ⇒ `instrument` is the **fourth** identity input, on both sides.

⭐ **Proven by firing it, not by reading it.** With the row added to the gate and absent from the
baseline, `bash scripts/check_parse_cost_ratchet.sh` → **exit 1**,
*"the baseline's identity table has no `instrument` row, so that input is unguarded"*. After the
rebaseline → **exit 0**, *"identity fresh for: generated parser, grammar, instrument, sample inputs"*.

⛔⛔ **And adding it exposed a BOOTSTRAP DEADLOCK in the gate that had never been reachable before:**
the `instrument` row cannot exist until a rebaseline writes it, and the rebaseline refused to write
while the row was missing (`if REBASELINE and not failures`). Identity divergence was a hard failure
on *every* path, including the one whose entire purpose is to resolve it. Fixed narrowly: under
`PGEN_PARSE_COST_REBASELINE=1` a missing or stale identity row is a **NOTE**; on every other path it
stays a hard failure, and the ratchet's own breach check is untouched — a rebaseline still refuses if
a BINDING counter rose.

###### ⛔⛔ RESULT 5 — (b): THE SAMPLE **WOULD** CHANGE, AND ADOPTING IT NOW WOULD BE WRONG

Measured with the instrument's **own** `select_sample`, run over two censuses that differ only in the
predicate:

| tier | rows | changed by the fix |
|---|---:|---|
| `hot` (heaviest by total entries) | 40 | **0** |
| `lr` (heaviest by family entries) | 40 | **5** |
| `breadth` (deterministic stride over the rest) | 112 | **19** — a *consequence*: breadth fills from what `hot`+`lr` leave |

⇒ 24 of 192 rows would move. **Not adopted, for three measured reasons in increasing force:**
1. The `hot` tier — which carries the bulk of the entries — is **completely unaffected**, so the
   sample's coverage of the cost it exists to watch does not improve.
2. Re-deriving resets the ratchet's binding baseline. Trading the comparability the ratchet exists
   for, in exchange for reshuffling 12.5 % of one selection tier, is a bad trade.
3. ⛔⛔ **Decisive: the census is not currently REPRODUCIBLE, so a sample derived from it today
   cannot be pinned honestly.** Same measurement, same instrument: re-deriving the *existing*
   predicate's sample from a fresh census reproduces `hot` **40/40** and `lr` **40/40** exactly — and
   `breadth` differs by **5 of 112**, entirely because the census now yields 16 335 rows instead of
   16 336. `.22`'s single silently-dropped file perturbs the per-suite pools and moves the stride.
   ⇒ **acceptance (b) is BLOCKED on `.22`**, and that is a measured dependency, not a preference.
   ⭐ It is also the sharpest available evidence that `.22`'s silent drop is not cosmetic: **one
   unreported file moves 5 pinned sample rows.**

###### ⚠️ WHAT THIS SLICE DELIBERATELY DOES NOT DO

- ⛔ It does not re-derive the pinned sample — (b) stays open and is **blocked on `.22`**, above.
- It does not touch `.20` (a)/(b)/(c)/(e). `.20` stays `todo` and ruling clause **B** still binds.
- It does not rewrite the historical record: slices 1-3's `0.681 %` / `~35×` stand as written, with
  a SUPERSEDED banner at slice 1 RESULT 4 naming the corrected magnitude. Supersede, don't mutate.

###### Acceptance Checklist (enforced) — `.21` slice 1

- [x] **REPRODUCE / ISSUE** — the defect is measured, not suspected: a full-corpus per-rule census
  (16 335 files) sums the complete LR family at **24 644 435 of 899 064 022 entries = 2.7411 %**
  while `LR_FAMILY_RE` matched **6 125 716 = 0.6813 %**, i.e. **18 518 719 entries = 75.1 %**
  uncounted, spread over **16** `_lr_seed` and **6** `_lr_guard` rules. Independently corroborated by
  a second instrument on a different quantity: across 8 `/usr/bin/sample` profiles the old predicate
  accounts for **0.54 %** of a **2.27 %** LR self-time — a **76 %** miss against the census's 75.1 %.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: the classifier was derived from the leaf's PROSE
  (`X := X_lr_base ( X_lr_suffix )*`) rather than from the code that emits the names, and its ten
  controls were drawn from the same prose, so they could only ever confirm it. WHERE, to the line:
  `stimuli/sv/corpus_parse_cost.py:100`, against **eight** emission shapes located by
  `grep -n 'format!("{[a-z_]*}_lr'` over the two eliminators —
  `indirect_lr_elimination.rs:862/915/916/1018/1188/1190/1241` and `ast_pipeline/mod.rs:3244/3347/3349`
  — plus a ninth hole, `allocate()`'s `_{index}` collision suffix, which defeats an END-anchored
  pattern by construction. `git ls-files` + `grep -c '"[a-z_0-9]+_lr_alt[0-9]*"'` over the generated
  parser confirms `_lr_alt` is 0 in SV today, so it was covered on the emitter's authority, not the
  artifact's.
- [x] **FIX** — fix-hierarchy tier = **instrument correctness + its guard**, no engine, grammar or
  generated bytes. (1) `LR_FAMILY_RE` → `_lr_(base|suffix|seed|guard|alt)(?![a-z])`, segment-anchored
  so `allocate()`'s collision suffix cannot drop a rule and the pinned near-miss `_lr_baseline` is
  still refused; (2) the control suite rebuilt from the **producer's** shapes — 21 cases, one per
  emission site plus two collision positives and four near-miss negatives; (3) the corpus-wide bound
  moved out of report prose into named constants with a provenance string, because a number
  hand-copied into four surfaces goes stale in all four at once; (4) ⭐ `instrument` added as the
  **fourth identity input** on both the instrument and the gate, so this class of staleness cannot
  recur silently; (5) the rebaseline path un-deadlocked so an identity input can ever be adopted.
- [x] **ADDRESSED (verified)** — measured before→after on the real gate. **RED first:**
  `bash scripts/check_parse_cost_ratchet.sh` → **exit 1**, *"the baseline's identity table has no
  `instrument` row, so that input is unguarded"* — the new guard fired before it was satisfied.
  **GREEN after** `PGEN_PARSE_COST_REBASELINE=1` → **exit 0**, *"identity fresh for: generated
  parser, grammar, instrument, sample inputs; 192 pinned sample files"*. **Control-suite RED probe:**
  the new 21-case suite run against the OLD predicate **MISSES 8 of 21**, all in the
  should-match direction — the controls are demonstrably non-vacuous. **The correction is
  reporting-only, and that is gate-held rather than asserted:** across the rebaseline all three
  BINDING counters are byte-identical (`entries` 416,841,264, `committed` 7,124,616, `memo_hits`
  186,981,263) while `lr_entries` moved ×4.02 and `lr_committed` ×4.05.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 19 enforced doctrines PASS**,
  `PARSE-COST-RATCHET` included and now guarding a fourth input. ⭐ Zero code, grammar and generated
  bytes: `generated/systemverilog_parser.rs` hashes identically before and after (it is one of the
  identity rows the gate re-checks), so no parser behaviour can have moved, and the sample manifest
  is untouched. `make -C rust SHELL=/bin/bash mdbook_docs_gate` passes.
- promotion: `docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md` **updated
  rather than duplicated** — the card's entire argument rested on the `0.681 %` this slice refutes,
  and it now carries the corrected figures plus the meta-lesson its own correction demonstrates
  (*measure the coupling — with a classifier derived from the producer, not from your own prose*).


##### ✅ `.21` SLICE 2 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0039`, 2026-08-15 session #236) — (g) DISCHARGED: `_lr_alt` is OBSERVED, the predicate is verified across all ten families, and the published declared-name count was wrong by one

###### ⭐⭐⭐ RESULT 1 — (g) part 1: `_lr_alt` IS OBSERVED, BY AN ORACLE THIS INSTRUMENT DID NOT BUILD

Slice 1's self-assessment GAP 3 was exact: `expression_lr_alt1` was *"a string I typed from reading
`mod.rs:3244`, not a name any parser has emitted"*, so a mis-read of that `format!` would have
reproduced as a **passing** control. Three scratch-slot probes settle it, all preserved as tracked
artifacts (`scripts/preserve_scratch_probe.sh`, so the synthetics outlive the slot restore):

| probe | artifact | grammar shape | result |
|---|---|---|---|
| **A** | `lr_alt_consumed_and_retracted.ebnf` | `expr := expr "+" term \| expr "-" term \| term` | two hoists, both consumed; `retract_consumed_normalization_rules` deletes them ⇒ generated parser declares `expr_lr_base`/`expr_lr_suffix` and **no `_lr_alt`** |
| **B** | `lr_alt_survives_unconsumed.ebnf` | one inline direct alt + one bare ref to an indirect wrapper | the planner leaves the hoist unconsumed, and well-formedness **REFUSES** — naming it: `rule 'expr_lr_alt1' has no finite terminal derivation` |
| **C** | `lr_alt_index_discriminator.ebnf` | the only direct alt at **position 1** | engine emits **`expr_lr_alt2`** |

⭐ **B is the observation.** The name comes out of the grammar well-formedness checker — an oracle
written for another purpose entirely — not out of this instrument or its author's reading.

⭐ **C is a FALSIFICATION, not a repetition.** B alone is equally consistent with two hypotheses:
H1 the `{n}` is the ALTERNATIVE's `index + 1` (mod.rs:3244), H2 it is a running count of hoists. B
hoists at position 0, where both predict `_lr_alt1`. C moves the only direct alternative to
position 1: H1 predicts `_lr_alt2`, H2 predicts `_lr_alt1`. **Measured `expr_lr_alt2` ⇒ H1
confirmed, H2 refuted.** The two controls in `_self_check` are now these observed names.

⛔⛔ **AND THE HONEST FINDING: `_lr_alt` IS UNREACHABLE IN ANY SHIPPED PARSER, BY TWO INDEPENDENT
MECHANISMS.** (1) when the planner consumes the hoist, the retraction deletes it (probe A); (2) the
only shape that leaves it referenced — `base_alternatives` empty at `mod.rs:3335`, because
normalization's own `seed_alternatives` counter (mod.rs:3231) counts a bare wrapper reference as a
seed and so its *"all alternatives are left-recursive → leave it alone"* guard does not fire — is
refused by well-formedness before codegen (probe B). ⇒ keeping `_lr_alt` in the predicate is
**defensive coverage of the emitter**, which is correct and is the whole lesson of the defect, but
the instrument now SAYS that rather than implying it is a live dump shape.

###### ⭐⭐ RESULT 2 — (g) part 2: THE PREDICATE OVER ALL TEN FAMILIES, AS A TRACKED RE-RUNNABLE MODE

`python3 stimuli/sv/corpus_parse_cost.py --verify-families` (~1 s, reads the artifacts, no probe):

```
generated/ebnf.rs                                  138     0          0  —
generated/json_parser.rs                             9     0          0  —
generated/regex_parser.rs                          276     0          0  —
generated/return_annotation_parser.rs               35     2          2  base=1 suffix=1
generated/rtl_const_expr_parser.rs                  48     0          0  —
generated/rtl_frontend_parser.rs                   169     0          0  —
generated/semantic_annotation_parser.rs            114     2          2  base=1 suffix=1
generated/systemverilog_parser.rs                 1608   127        127  base=5 guard=6 seed=24 suffix=92
generated/systemverilog_preprocessor_parser.rs      74     0          0  —
generated/vhdl_parser.rs                           225     0          0  —

10 generated parsers, 131 declared LR rule names, 131 classified.
```

⭐ Three design points, each because the obvious version would pass by construction:
- **the roster is DERIVED from the artifacts** (every `generated/*.rs` declaring a `RULE_NAMES`
  registry), never hand-listed — a hand list is one more copy to drift, and a new family would
  escape the check the day it lands. It independently reproduces the ten families the live-status
  register names. `scratch_parser.rs` is excluded BY NAME with the reason stated: it is the blessed
  throwaway slot, so its rule set is whatever probe was last loaded.
- **the population is every declared name CONTAINING `_lr`**, not every name the predicate matches.
  Scoping it to what the predicate already accepts is exactly the ten-controls-from-the-same-prose
  shape that caused this leaf.
- **the reader checks itself**: `RULE_NAMES` is read together with the parser's own `RULE_COUNT`
  and a length mismatch REFUSES, so a truncated scan cannot make an unclassified name look absent.

###### ⛔⛔ RESULT 3 — THE PUBLISHED DECLARED-NAME COUNT WAS **128** AND IS **127**

Verified three DIFFERENT ways (`docs/CLAIM_VERIFICATION.md` §3), not three repetitions:

1. **RE-DERIVE** — the parser's own `RULE_NAMES` registry holds 1 608 entries, matching its
   self-declared `RULE_COUNT = 1608` (so the extraction is provably not truncated), of which
   **127** contain `_lr_`.
2. **FALSIFY** — the competing hypothesis *"128 is right and my reader misses one"* dies three
   ways: two other syntactic surfaces of the same file (`fn parse_*` names; bare string literals)
   yield the **set-identical** 127; the registry's own length confirms completeness; and
   ⭐ **slice 1's own decomposition already summed to 127** (97 matched + 24 `_lr_seed` +
   6 `_lr_guard`), so the published total contradicted its own parts. That last one is an oracle I
   did not build — the previous measurement's own sub-counts.
3. **DURABILITY** — `SV_DECLARED_LR_RULES = 127` is **gated, not carried**: `--verify-families`
   re-derives it and refuses on drift (RED arm 2 below).

⛔ Nothing this changes moves a cost claim: the count is the **declared** population, while the
`2.741 %` bound is measured over corpus **entries**. It is a defect in a published number, not in
the ratchet.

###### ⭐ RESULT 4 — A RED PROBE FOUND A REAL FRAGILITY IN THE NEW CODE, BEFORE IT SHIPPED

Running the new checker against slice 1's OLD predicate **crashed** instead of reporting: the
per-shape breakdown read `m.group(1)`, and the old pattern's group 1 is the optional `(_r\d+)?`, so
it was `None`. The reporting was coupled to the predicate's internal group layout — a coupling to
the one thing this leaf is a record of somebody editing. Fixed by naming the group
(`(?P<shape>…)`) and degrading to `?` when it is absent, which RED arm 4 pins.

**Five arms, GREEN control first, all fired:**

| arm | expectation | result |
|---|---|---|
| GREEN | the shipped predicate passes | ✅ rc 0 |
| RED 1 | slice 1's OLD predicate | ✅ rc 1, **30 unclassified** (24 `_lr_seed` + 6 `_lr_guard`) — this check would have caught the original defect on day one |
| RED 2 | the pinned count set to 128 | ✅ rc 1, *"declares 127 … but SV_DECLARED_LR_RULES pins 128"* |
| RED 3 | a reader blind to `RULE_COUNT` | ✅ rc **2** (REFUSES; never an empty roster) |
| RED 4 | a predicate with no named `shape` group | ✅ rc 0, reports rather than crashing |

⭐ RED 3's two negatives are drawn from the **real** corpus, not imagined: `const
RULE_COUNTED_QUANTIFIER` (a prefix collision, ×3 in the regex parser) and `const THIN_RULE_COUNT`
(a suffix collision) both defeat a loose `RULE_COUNT` match.

###### ⚠️ WHAT THIS SLICE DELIBERATELY DOES NOT DO

- (e) and (f) stay **open**. `--verify-families` is tracked and one command re-runs it, but it is
  invoked from the ratchet's **tier 2**, which is on-demand — so the other nine families are
  re-checked when an operator re-measures, **not on every commit**. Naming the gap rather than
  hiding it: SV's own names are covered every run because tier 1 re-hashes the SV parser. Wiring an
  every-run tier is (f)'s call.
- The audit instruments in `rust/target/audit_scratch/` are still untracked — that is (e).
- Slice 1's historical prose keeps its `128`; the correction is recorded here, per supersede-don't-mutate.

###### Acceptance Checklist (enforced) — `.21` slice 2

- [x] **REPRODUCE / ISSUE** — measured, not suspected. `_lr_alt` had **0** occurrences in all ten
  generated parsers (`--verify-families`: `alt` absent from every shapes column), so its control was
  a typed string no artifact backed — slice 1's own GAP 3. And the declared-name denominator
  published in four surfaces read **128** while the parser's `RULE_NAMES` registry holds **127**
  (1 608 entries total, matching its self-declared `RULE_COUNT = 1608`).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY, for `_lr_alt`: `retract_consumed_normalization_rules`
  (`rust/src/ast_pipeline/mod.rs:3139`) deletes every hoist the planner consumes, and the only
  shape that leaves one referenced makes `base_alternatives` empty at `mod.rs:3335`, which grammar
  well-formedness then refuses — so the shape is unobservable in a shipped parser by two
  independent mechanisms, and the emitter at `mod.rs:3244` is the only place it can be read. Tool
  output that located it, from the engine itself on probe B:
  `grammar well-formedness ERROR: rule 'expr_lr_alt1' has no finite terminal derivation (it can
  never produce a complete string) — it is ill-formed; add a terminating alternative`.
  WHY, for the count: it was a **carried** constant, copied into four surfaces, so it could be
  wrong in all four at once — the same defect `.21` slice 1 fixed for the family share and left in
  place for this one.
- [x] **FIX** — fix-hierarchy tier = **instrument correctness + its guard**; zero engine, grammar or
  generated bytes. (1) new `--verify-families` mode: derives the parser roster from the artifacts,
  reads each `RULE_NAMES` registry against its own `RULE_COUNT`, and refuses on any declared name
  containing `_lr` that no emission shape claims; (2) `SV_DECLARED_LR_RULES = 127` gated by that
  mode instead of carried; (3) the two `_lr_alt` controls replaced with the OBSERVED `expr_lr_alt1`
  / `expr_lr_alt2`; (4) the shape token made a NAMED group so the reporter is not coupled to the
  predicate's layout; (5) the mode invoked from the ratchet gate's tier 2; (6) the three probe
  grammars preserved as tracked artifacts.
- [x] **ADDRESSED (verified)** — measured before→after on the real gate. **RED first:**
  `bash scripts/check_parse_cost_ratchet.sh` → **exit 1**, *"the parse-cost BASELINE IS STALE …
  instrument: baseline `56df92badc9c7194…` vs live `bc5015a74d6a151f…`"* — slice 1's fourth identity
  input caught its own successor's edit, unprompted. **GREEN after**
  `PGEN_PARSE_COST_REBASELINE=1` → **exit 0**, *"identity fresh for: generated parser, grammar,
  instrument, sample inputs; 192 pinned sample files"*. **Five-arm probe on the new checker** (table
  above): GREEN + 4 RED, every one fired, including the OLD predicate leaving exactly **30** names
  unclassified.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 19 enforced doctrines PASS**.
  ⭐ Reporting-only, and gate-held rather than asserted: across the rebaseline `entries.tsv` is
  **untouched** — all five columns byte-identical (`entries` 416,841,264, `committed` 7,124,616,
  `memo_hits` 186,981,263, `lr_entries` 12,440,690, `lr_committed` 514) — and only `cost.md`
  (4 lines: the instrument hash and 128→127) and the machine-dependent `advisory.json` moved.
  `generated/systemverilog_parser.rs` hashes identically before and after, so no parser behaviour
  can have moved. The scratch slot is restored **byte-identical to HEAD**
  (`git diff --quiet -- grammars/scratch/scratch.ebnf`) and regenerates the default fixture with 0
  LR rules. `make -C rust SHELL=/bin/bash mdbook_docs_gate` passes.
- promotion: `docs/knowledge/deriving-a-control-from-the-producer-is-not-the-same-as-observing-it.md`
  **NEW**. The lesson is a genuine refinement of `docs/CLAIM_VERIFICATION.md` §3 leg 2 rather than
  an instance of it: *"derive from the producer, not from a description of the producer"* kills a
  test and an implementation descended from the same PROSE, and leaves alive a test and an
  implementation descended from the same **reading of the same line** — which is exactly what
  slice 1's `_lr_alt` arm was. Plus the corollary this slice paid for: a probe that confirms at the
  DEFAULT position has not separated the hypothesis from its rival (probe B) — move the thing
  (probe C). ⭐ The existing card `a-deterministic-counter-cannot-see-a-per-entry-cost-rise` is
  **updated in place** (128 → 127 in both its `evidence:` header and its body), not duplicated.

#### ⛔⛔⛔ `.22` NEW `todo` — the transactional coverage stack (TOOLBOX 3.5) never terminates on a corpus file that a bare parse accepts in 0.077 s, and the census silently drops it (opened 2026-08-15 session #235 by `.20` slice 4)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine):

- **The file**: `stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv`, **2 787 bytes**, recorded
  `pass` at **0.07 s** in `stimuli/sv/characterization/durations.tsv`.
- **Three parses, and the isolation is exact**: BARE (FUSED graph) **0.077 s** accepted; TOOLBOX
  **3.4** entry dump (PROTOCOL graph, no coverage stack) **0.062 s**, `accepted: True`, **200 975
  entries** over 711 rules; TOOLBOX **3.5** outcome dump (PROTOCOL graph **+ transactional coverage
  stack**) allocates ~150 MB/s to **4 682 MB peak in 30 s** and was terminated in **5 of 5** attempts
  (23 s / 27 s / 30 s / 59 s / the census's 120 s timeout) without ever writing a dump.
  ⇒ 3.4 and 3.5 take the **same** graph, so the defect is in the **coverage stack**, not the routing.
- **It is not a big-input effect**: 200 975 entries is 19× the corpus median (10 542) and **175×
  below** the corpus maximum (35 107 691), and the maximum file measures fine.
- ⛔ **It reproduces outside SystemVerilog by construction, though only SV is measured**: the
  transactional coverage stack is engine-universal (`memoized_call` + the rule transaction), not
  SV-specific. Whether another family has a triggering input is **unmeasured** and must not be
  assumed either way.
- ⛔ **It is silent by construction today.** `measure_one_entries` returns `None` on
  `subprocess.TimeoutExpired` and the row is counted into a `nodump` list; the census prints the
  count and continues. That is how slice 1's *"zero no-dump rows"* and *"16 336/16 336 agree"*
  became unreproducible without anything failing.

**Acceptance:** (a) root-cause the blow-up with the toolbox — ⛔ 3.6's memo insert/evict/replay census
is the named instrument for *"memo-served but still super-linear"*, and it needs a trace this parse
may not survive, so sizing that is part of (a); (b) decide whether the file's `accepted: True` under
3.4 and its `pass` under a bare parse are the same derivation (a verdict agreement is not a
derivation agreement); (c) ⛔ make the drop LOUD — a no-dump row is currently indistinguishable from a
measured one in every downstream number, and the fix is a REFUSAL or a published `nodump` roster, not
a bigger timeout; (d) restate slice 1's ground-truth claim with the reproducible number
(**16 335/16 335**, 1 no-dump) rather than the published `16 336/16 336`.


#### ⛔⛔ `.19` NEW `todo` — the pre-slice-9 ADMISSION reproduces the pre-slice-9 BEHAVIOUR but not the pre-slice-9 BYTES: 99 747 bytes of SystemVerilog codegen are unaccounted for (opened 2026-08-14 session #232 by `.17` slice 9)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **Measured, not inferred.** `rust/target/debug/ast_pipeline generated/systemverilog.json
  --generate-parser --debug --trace --eliminate-left-recursion
  --indirect-lr-admit-starvation-safe-only` produces `dae09343…`, **131 542 908 bytes**, with
  **1 488 grammar rules and 0 guard rules** — the pre-flip grammar exactly. The tracked pre-flip
  artifact was `4330ff8e…`, **131 642 655 bytes**. Same policy, same rule set, **99 747 bytes apart**.
- **It is not the admission.** The rule-function count, the guard count and `--lint-grammar` under the
  lever (`left_recursion_unhandled=28`, `starvation-safe candidates: 0/28`) all reproduce the pre-flip
  numbers, so the lever is a faithful POLICY inverse. The difference is in emitted code, not in what
  was absorbed.
- **It does not reproduce in the other families**, which is what bounds it: the other 10 generated
  parsers were re-derived byte-identical with this same binary against the session-start snapshot.
  ⇒ whatever moved is reachable only through the indirect-LR path, or through SystemVerilog's own
  frontend output.
- **Two hypotheses, neither eliminated**, and saying so beats picking one:
  **(a)** something in `.17` slice 9's diff perturbs SV codegen even under the narrow admission;
  **(b)** an INPUT moved — `generated/systemverilog.json` is a **git-ignored derived artifact** that
  `make` regenerates whenever the frontend binary is newer, so any session that rebuilds
  `ast_pipeline` silently re-derives it, and the pre-flip parser came from a JSON this working copy no
  longer holds. ⭐ (b) is **`.16`'s class one level over** — the same "derived artifact no `git status`
  can report on" shape, this time with a regeneration rule that fires too eagerly rather than never.
- **Why it was not chased inside slice 9**: the anti-spin tripwire. Distinguishing (a) from (b) needs
  NEW tool output, not more reading, and the two measurements that would do it are cheap but not free
  (regenerate twice for determinism; re-derive at the parent commit's source with today's JSON).
- ⛔ **The blast radius is bounded and named.** Slice 9's published claims do not rest on this: the
  lint A/B, the speed A/B and the byte-identity of the other 10 parsers are all SAME-BINARY
  measurements. What is in doubt is the narrower statement *"a narrow-arm rebuild reproduces the
  pre-flip artifact"*, which nothing in slice 9 asserts.

**Acceptance:** (a) regenerate SV twice under the narrow lever and compare — a mismatch proves codegen
is non-deterministic and ends the investigation there; (b) if deterministic, re-derive at
`88b06424`'s source with TODAY's `generated/systemverilog.json` — matching `dae09343…` proves
hypothesis (b) (the input moved) and matching `4330ff8e…` proves (a) (the diff moved codegen);
(c) whichever it is, add the gate that would have caught it — for (b) that is a freshness check on
`generated/systemverilog.json`, which is the same instrument `.16` owes `generated/ebnf.rs`, so the
two should be designed together; (d) ⛔ do NOT close this by observing that the behaviour matches —
*"it behaves the same"* and *"nothing else moved"* are different claims, and this leaf exists because
the second one is unproven.


#### ⛔ `.16` NEW `todo` — `generated/ebnf.rs` is a SEED-ONLY artifact, so local and fresh-clone builds can diverge indefinitely (opened 2026-08-13 session #224 by `.13` slice 5)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **It reproduces independent of this slice.** The local `generated/ebnf.rs` is dated 2026-07-30;
  `grammars/ebnf.ebnf` and the codegen have both moved since, and nothing in `make` regenerates it.
  So the divergence predates the indirect-LR pass — this slice only made it *visible* by being the
  first change in a while that alters what a fresh seed would produce.
- **The blast radius is bounded and named.** `generated/ebnf.rs` is compiled in only under
  `ebnf_dual_run`, where it is arm 2 of the frontend differential. A stale arm 2 makes
  `ebnf_dual_run_diff` compare today's hand-written frontend against a fortnight-old meta-parser —
  green locally, and a different comparison on a cold CI runner.
- **It is the same CLASS as slice 4b's trap**, one level up: a git-ignored derived artifact that no
  `git status` can report on, and this time with no regeneration target at all rather than an
  incomplete restore. ⛔ Not the same *instance* — 4b's was a probe script's half-restore.
- **Not worked here** (SV lane lock; this is `ebnf`-family flow machinery). Trigger to start:
  the next `ebnf_dual_run` gate investigation, or any change to `grammars/ebnf.ebnf`.

**Acceptance:** (a) decide whether `ebnf` joins `GENERATED_PARSER_FAMILIES` or gets an explicit
freshness check; (b) whichever is chosen, a gate that FAILS when the artifact is older than its
inputs — the current state is undetectable by construction.

**HONEST BOUNDS (each measured, none argued):**

1. **Simple routes only.** Enforced by the trial guard above rather than assumed, and every refusal
   prints the surviving cycle path.
2. **A hop with a residual and no return annotation is REFUSED.** Its undeclared value is the
   engine's default shaping of the whole alternative; `$1` would silently drop the residual. A hop
   that is a BARE reference needs no annotation — such an alternative adds no wrapper node, so its
   value *is* the referenced rule's value, and `$1` reproduces it exactly.
   ⇒ `p1_knot_a_defect.ebnf` (no annotations at all) **cannot** prove this fix, which is why
   `p4_knot_a_annotated.ebnf` was added — P1 plus exactly the annotations SystemVerilog declares on
   the same four hops.
   ⇒ ⭐ This is also why `parse_harness_combinator_suite`'s `recursion_guarded_memo_isolation` case
   still tests what it always tested: its grammar is unannotated with a residual on the cycle hop,
   so the pass refuses it and the surviving cycle the case depends on survives. That is correct
   behaviour, not luck — but it *is* load-bearing, so annotating that case would silently change
   what it measures.
3. **Suffix-branch starvation is not modelled.** The base-rule starvation criterion is now
   transitive, but nothing checks whether one `X_lr_suffix` route rule can starve another. Nothing
   has been observed, and nothing proves it cannot be. ⚠️ Given that the *base* form of exactly this
   omission shipped a regression in this slice, treat the absence of observations as weak evidence.
4. **The route walk is profile-blind** (finding 4 above). It is *sound* — a route no profile can
   derive is sheared and contributes nothing — but the route COUNT the survey prints includes
   routes that derive nothing, so `routes=` is an upper bound on derivable routes, not a count of
   them.
5. **The `@profiles:` gate is COPIED, never synthesized.** When a route's intersection is not equal
   to any single rule's declared list, the plan is refused rather than constructing a second
   spelling of a directive whose only authoritative spelling is the grammar's. No shipped grammar
   hits this today (dialect gates nest).

##### Acceptance Checklist (enforced) — slice 5, the transformation

- [x] **REPRODUCE / ISSUE** — ⛔ **be precise about WHICH defect this slice closes.** The leaf's
  opening defect is *"INDIRECT left recursion is eliminated by nothing"* — an absent ENGINE
  CAPABILITY — and that is what closes. It is **not** the same statement as *"SystemVerilog's
  `int'(2)'(3)` parses"*, and this slice delivers the first without the second. Measured on ONE
  binary, before and after, via the new `--no-eliminate-indirect-left-recursion` A/B switch:
  `ast_pipeline grammars/<g>.ebnf --lint-grammar --no-eliminate-indirect-left-recursion` →
  `'systemverilog' (1485 rules) left_recursion_unhandled=30` and `'ebnf' …=5`, with **no pass in
  the engine able to rewrite either shape** — every cycle left to a runtime guard that REJECTS
  re-entry rather than handling it (`GRAMMAR-WELLFORMED.A2.6`). The two LRM-legal SystemVerilog
  victims (`int'(2)'(3)`, `8'(1)`; `adjudicate.py` rows SV-1/2/3/5, plus 2 OpenTitan corpus rows
  priced by `SV-CORPUS-GRAD.13c.2b`) stay REJECT after this slice **by an explicit, measured
  refusal** rather than by absence — `starvation-safe candidates: 0/28` — and are routed to `.17`.
- [x] **ROOT CAUSE (WHY + WHERE)** — **two** root causes, both measured, neither argued.

  **(i) Why chain absorption cannot close SystemVerilog's cast/call knot** — the criterion that said
  it could was not transitive. WHERE: `indirect_lr_plan::collect_starvation_sites` scanned holders of
  the BASE rule only, so `casting_type := … | constant_primary` (`grammars/systemverilog.ebnf:1033`,
  a bare reference, empty residual) hid `cast := casting_type tick lparen expression rparen`
  (`:1029`). WHY it matters: a bare-reference alternative is AST- *and* consumption-transparent, so
  the transparent rule inherits the base's greed. Measured on the shipped grammar, three inputs one
  difference apart: `initial k = 8;` ACCEPT · `parameter logic [7:0] K = 8'(1);` ACCEPT ·
  `initial k = 8'(1);` **REJECT** `furthest_position=42`. Fixed by `rules_transparent_to`, a least
  fixed point over empty-residual left-corner edges; SystemVerilog then reports
  `starvation-safe candidates: 0/28` and the pass declines the knot instead of breaking it.

  **(ii) Why the previous two slices' target rule could not work**, printed by the pass itself:
  `ast_pipeline grammars/systemverilog.ebnf --report-indirect-lr-plan` →
  `⛔ REFUSED 'constant_primary_sv_2017': the rewrite left 'constant_primary_sv_2017' left-recursive
  — the plan sheared only part of the cycle, which still closes through constant_primary_sv_2017 ->
  constant_primary_sv_2017_lr_base -> …_lr_seed_constant_primary -> constant_primary_sv_2023 ->
  constant_cast -> casting_type -> constant_primary -> constant_primary_sv_2017`. The sibling
  dialect arm is reached through a path that re-enters three rules already on the route, so it is
  not a *simple* route and `indirect_lr_plan::collect_routes` (`indirect_lr_plan.rs`, the `visited`
  guard) refuses it by design. WHERE the wrong criterion came from: `DeclineReason::NoAcyclicSeed`,
  inherited from the DIRECT transform, which DROPS a cyclic alternative where this one CLONES it.
  The profile defect's WHY+WHERE is the emitted parser itself:
  `parse_constant_primary_sv_2017` carries `if !self.rule_profile_is_enabled(&["sv_2017",
  "verilog_2005"])` and the first draft's clone carried nothing.
- [x] **FIX** — fix-hierarchy tier = **ENGINE** (a new pass; ZERO grammar bytes, ZERO changes to
  `lr_chain_fold`): `rust/src/ast_pipeline/indirect_lr_elimination.rs` (plan → trial → commit,
  clones, composed templates, per-route profile-gated suffix rules, 6 refusal classes), wired into
  `eliminate_left_recursive_patterns` (`ast_pipeline/mod.rs`) behind
  `PipelineConfig::eliminate_indirect_left_recursion` (default on) with
  `--no-eliminate-indirect-left-recursion` as the A/B switch, and
  `LeftRecursionEliminationOutcome` extended with `indirect_eliminated_base_rules` /
  `indirect_clone_rules` / `indirect_refusals` so the report derives from the pass's outcome rather
  than asserting one (the `A2.6` posture). `indirect_lr_plan.rs` loses the `NoAcyclicSeed` decline
  and gains `covered_rules()`. Why no lower tier: a grammar-tier repair is refused by
  [[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]], and the two victims'
  rules are byte-identical LRM transcriptions (`constant_primary_lrm_alternative_audit.py`).
  Registered: the report's new header lines and the A/B switch are in `TOOLBOX.md` §5.5 and the
  book mirror; no new `DIAGNOSIS_SIG` token is needed because both ride the already-registered
  `INDIRECT-LR-SURVEY:`/`--report-indirect-lr-plan` instrument.
- [x] **ADDRESSED (verified)** — measured before→after on one binary via the A/B switch, plus the
  isolating synthetic and 11 unit tests. ⛔ **Read the SystemVerilog row as the REFUSAL it is:**
  `'systemverilog' (1485 rules) left_recursion_unhandled=30` → `(1488 rules) …=28`, and
  `starvation-safe candidates: 0/28` — the cast/call and property knots are *declined*, not fixed,
  and `.17`/`.15` own them. `'ebnf' …=5` → `…=0` (`return_expression`, 1 clone).
  `cargo test --lib indirect_lr` → **11 passed, 0 failed**, including
  `the_composed_template_is_the_hand_derived_nesting` (the composed AST asserted hop for hop),
  `a_mutually_recursive_set_is_eliminated_at_its_dominator`, and
  `a_profile_gate_survives_on_both_the_clone_and_the_suffix_route`. The isolating synthetic
  `p4_knot_a_annotated.ebnf` accepts all five probe inputs (`n`, `t'(n)`, `n'(n)`, `t'(n)'(n)`,
  `t'(n)'(n)'(n)`) — P3's hand-eliminated result, reached with **no hand-written rewrite in the
  loop** — and the same shape is now a first-class combinator case
  (`indirect_left_recursion_folded_ast`, `Combinator::IndirectLeftRecursionFoldedAst`), which is
  acceptance (c).
  ⚠️ **Honest exercise bound:** with `constant_primary` correctly declined, SystemVerilog no longer
  exercises the CLONE or SUFFIX-ROUTE profile gating — only the HELPER inheritance (via
  `incomplete_class_scoped_type_sv_2023`, `@profiles: ["sv_2023"]`, which is what took
  `profile_orphans` 0 → 4 before the fix). The other two legs are held by the unit test alone.
- [x] **NO REGRESSION** — `generated/systemverilog_parser.rs` is the ONLY artifact that changes;
  the other **10** generated parsers are byte-identical by md5 across a full
  `regenerate_generated_parsers` (`ebnf.rs`, `json_parser.rs`, `regex_parser.rs`,
  `return_annotation_parser.rs`, `rtl_const_expr_parser.rs`, `rtl_frontend_parser.rs`,
  `scratch_parser.rs`, `semantic_annotation_parser.rs`,
  `systemverilog_preprocessor_parser.rs`, `vhdl_parser.rs`). Every other SystemVerilog lint counter
  is unchanged (`non_terminating` 0, `ordered_choice_shadowing` 0,
  `always_succeeds_alternatives` 6, `unreachable_rules` 0, `undefined_references` 0,
  `nullable_repetition` 0, `profile_orphans` 0) — the last of those only after the profile fixes;
  the first working version took it to 4 and that is recorded above rather than quietly repaired.
  `bash scripts/check_doctrines.sh` → **18/18**.
  ⛔ **AND THE CLIPPY FLOW CAUGHT A THIRD DEFECT THIS SLICE SHIPPED, WHICH NO TEST COULD.**
  `PipelineConfig` gained a field, and `src/bin/pgen_ast.rs:108` constructs it **exhaustively** —
  so that binary stopped compiling (`error[E0063]: missing field
  `eliminate_indirect_left_recursion``) while the lib, every test target and every gate stayed
  green, because they all go through `PipelineConfig::default()`. ⇒ `clippy_source_all_targets` is
  the only stage that builds **all targets**, and a struct-field addition is exactly the change
  class whose blast radius lands outside the tested set. Fixed (the binary opts in explicitly, with
  the reason), and `grep -rn "PipelineConfig {" --include=*.rs rust/src/` confirms it is the only
  exhaustive construction outside the `Default` impl and one test.
  **CONFIRMATORY SWEEP, CONSUMED** (not merely started — slice 4b's lesson):
  `cargo test --features "generated_parsers ebnf_dual_run" --lib -- --skip deep_nesting` →
  **1 091 passed / 1 failed / 28 ignored** in 1 025.9 s. The single failure is
  `unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`, **pre-existing and
  owned by `LANG-CAPABILITY-AUDIT.10.15`** — the identical row slice 4b re-observed at
  1 085/1/28; the count rose to 1 091 because this slice adds 11 unit tests and a combinator case,
  and the failure set is unchanged. ⭐ **Acceptance (c) is inside that sweep and green:**
  `parse_harness_combinator_suite::gate::every_structural_combinator_is_byte_identical ... ok` and
  `combinator_coverage_is_complete ... ok`, so the new `indirect_left_recursion_folded_ast` case —
  six inputs including the two-traversal `t'(n)'(n)'(n)` — agrees between the interpreter and the
  generated parser and matches its independently reasoned anchors.
  ⭐⭐ **The load-bearing no-regression evidence is `stimuli/sv/run_adjudication_repros.py`, and it
  is the ONLY thing that caught the regression this slice nearly shipped.** On the over-eager
  version it reported `checked=29 armed=7 failures=3`, of which the decisive row was
  `FAIL control_size_cast_in_statement.sv control expect=ACCEPT got=REJECT` — *"A CONTROL STOPPED
  PARSING. Its reproducer no longer isolates one difference"*. Neither the lint, nor 11 unit tests,
  nor the byte-identity of 10 generated parsers, nor 18/18 doctrines could see it. ⇒ the two-sided
  ratchet's CONTROL arm (`SV-CORPUS-GRAD.13c.2`) is the reason this slice is correct.
  **AFTER the transitive fix, on the regenerated parser: `ADJUDICATION-REPROS: checked=29 armed=7
  listed=29 failures=0`** — measured before→after `3 → 0`. The three rows individually:
  `control_size_cast_in_statement.sv` REJECT → **parse_full passed**, and both
  `defect_constant_size_cast*.sv` back to REJECT (`furthest_position=38` / `36`), i.e. still
  `expect=REJECT` as their manifest rows say — **no `MANIFEST.tsv` re-baseline is owed, because
  nothing flipped.**
- [x] **LOCKSTEP** — `TOOLBOX.md` §5.5, `docs/book/src/grammar-wellformedness.md`,
  `docs/book/src/developer-architecture.md` (whose left-recursion paragraph was ALSO stale on
  `A2.5`), `docs/book/src/diagnosing-unknowns.md`, the slice-2 decision record + `docs/decisions/INDEX.md`,
  `docs/reference/RUST_CODEBASE_ANALYSIS.md`, two promoted knowledge records
  (`a-grammar-rewrite-must-verify-its-own-postcondition-…`,
  `copying-a-grammar-rule-means-copying-its-rule-level-directives-…`,
  `a-transparent-rule-inherits-the-greed-of-the-rule-it-forwards-to`), the `indirect_lr` artifact
  README + `p4_knot_a_annotated.ebnf` + `probe.sh`, the new combinator case
  (`parse_harness_combinator_suite.rs`), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`.

### `.15` — the SVA property knot is unfixable by chain-absorption, because the grammar HAND-FACTORED its precedence cascade (`todo`, opened 2026-08-12 session #223 by `.13` slice 4)

⛔ **A measured LIMIT on `.13` (d), routed out at the point it was found rather than left for the
receiving leaf to rediscover** (the `ROUTING-EVIDENCE` doctrine). Slice 1 adjudicated SV-6/SV-7 as
REJECTing LRM-licensed text (`property p; (a |=> b) -> c; endproperty`, `property_expr implies
property_expr`, IEEE 1800-2017 A.2.10). Slice 4 measures that `.13`'s transformation cannot reach it:

```text
[candidate] prop_primary_sv_2017  routes=40  seeds=29  clone_cost=6  verdict=STARVED
    ⛔ starved by prop_and_sv_2017 alt#0 (on-route, still reachable after the rewrite)
       — residual 'kw_and_cffa50a3 prop_and_sv_2017' a greedy suffix could steal
[candidate] prop_primary_sv_2023  …                                   verdict=STARVED
```

Those two are the knot's ONLY candidates; every other rule on it is declined `no_acyclic_seed`. So
there is no rule the chain may be absorbed at without a greedy `*` starving a live holder.

⭐ **The cause is grammar-tier scar tissue, and the RAW transcription proves it.** In
`systemverilog_lrm_profiled_wrapper` (raw Annex A) the same construct yields `property_expr_sv_2017`
/ `property_expr_sv_2023` as **`MAY-ABSORB` candidates at `clone_cost=1`**. What disqualifies the
shipped grammar is its hand-written `prop_and` / `prop_or` / `prop_iff` / `prop_until` precedence
cascade — a manual left-factoring of a construct the engine is supposed to own, which is exactly the
class `.2`'s scar-tissue census exists to find and
[[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]] argues against.

⛔ **Not scoped as "delete the cascade".** The cascade encodes SVA operator PRECEDENCE, which the
engine has no service for at all — that is `.3`'s named gap (*precedence declaration*). Removing it
without one would trade an over-rejection for a wrong parse tree. **Acceptance:** (a) the two
candidate designs priced — teach `.13`'s transformation to absorb a hand-factored cascade, versus
give the engine a precedence-declaration service and let the cascade be generated from it (`.3`);
(b) the choice recorded in `docs/decisions/`; (c) the fix, with SV-6/SV-7's probes flipping to
ACCEPT. ⛔ Sequencing: this leaf is BEHIND `.13` (d) — the transformation must exist before it can
be extended, and `.13` (d) closes the two knots that do not need this.

#### ⚠️ ROUTED IN — THE THIRD DESIGN GAINED A SECOND HALF, AND WHETHER THIS KNOT NEEDS IT IS UNMEASURED (`.17` slice 4, 2026-08-13 session #228)

`.17` slice 4 measured a **second** starvation the census below cannot see: an over-long **SEED** —
the rewritten base's own sheared clone — winning the holder's alternation, which no guard on the `*`
can reach and which needs a TRAILING guard on the same clone. On the cast/call knot it is a
predicted regression (`initial k = int'(1);`).

⛔ **Whether `property_expr`'s knot has the same exposure is NOT measured**, in either direction.
`GuardAssessment` carries `verdict` / `residual_first` / `guard_hops` and no seed term at all, so the
`FEASIBLE  variants=1  max_hops=0` line below is silent on it — exactly the way it was silent about
the wrapper's annotation count before slice 3. ⇒ do not read *"the cheapest row in the census"* as
*"one guard closes it"*; `.17` slice 5 adds the seed term to the census, and this leaf's price is
re-read after that, not before. The re-adjudication below **stands** — the starvation blocker is
still closable at the dominator — but its cost is now a lower bound.

#### ⛔⛔ ROUTED IN — THIS LEAF'S FOUNDING PREMISE IS RE-ADJUDICATED, AND A THIRD DESIGN IS NOW THE CHEAPEST (`.17` slice 2, 2026-08-13 session #226)

The heading above — *"unfixable by chain-absorption"* — rests on slice 4's measurement that the
knot's only candidates are `prop_primary_sv_2017` / `_sv_2023`, both STARVED. `.17` slice 2's
guard-feasibility census measures the same knot at its **DOMINATOR** and gets a different answer:

```text
[candidate] property_expr   routes=80  seeds=0  clone_cost=12  verdict=STARVED
    guard: FEASIBLE  suffix_first={-/}  variants=1 {-/}  max_hops=0
    ⛔ starved by prop_primary_sv_2017 alt#13 — residual 'implies property_expr' [guard=guardable first={-/} hops=0]
    ⛔ starved by prop_primary_sv_2023 alt#13 — residual 'implies property_expr' [guard=guardable first={-/} hops=0]
```

⇒ **the starvation blocker is closable at `property_expr` with ONE guard variant and ZERO clone
hops — the cheapest row in the entire census** (`.17`'s option (iii), a call-site follow-restriction
guard on the sheared clone). `property_expr_sv_2017` / `_sv_2023` are guard-feasible too, at
`variants=1 max_hops=1`. Two consequences for this leaf:

1. **Acceptance (a) gains a THIRD option to price** — *neither* extend chain absorption to a
   hand-factored cascade *nor* build `.3`'s precedence-declaration service, but let `.17`'s guard
   close the starvation and absorb at the dominator, leaving the cascade in place and untouched.
   It is the only one of the three that costs no new engine service.
2. ⛔ **Two things this does NOT establish, and neither may be assumed.** (i) The census answers the
   **starvation** question only — the wrapper refuses `property_expr` for a different reason
   (*"hop 'property_expr_sv_2017' alternative 5 declares no return annotation and its residual is
   'kw_or_1758356d property_expr', so the chain's AST cannot be composed faithfully"*), and whether
   that check passes on the SHIPPED grammar at this base rule is UNMEASURED. (ii) The cascade
   encodes SVA operator PRECEDENCE; absorbing at the dominator must be shown to preserve it, and
   `.13` slice 5's own history is the warning — the version that improved every counter was a
   REGRESSION only a CONTROL row caught.

⭐⭐ **(i) IS NOW MEASURED AND DISCHARGED (`.17` slice 3, 2026-08-13 session #227).** The guard
census, run through the REAL planner on the shipped grammar
(`--report-indirect-lr-plan --indirect-lr-plan-guard-dry-run`), absorbs `property_expr` with
`would_refuse=0`: **the chain's AST composes at this base rule.** ⛔ And the wrapper refusal quoted
above was never evidence about it — every one of that grammar's 13 dry-run refusals is *"declares no
return annotation"*, because its body is LRM-generated and carries **21** annotated rules against
`systemverilog.ebnf`'s **1069**. The two views are structurally comparable and NOT
annotation-comparable. ⇒ option 3 is not merely the cheapest on paper; it is the one whose plan the
engine can actually build today. **(ii) stands, unmeasured** — precedence preservation is still owed,
and remains the reason this leaf is not closable from a census alone.

⇒ this leaf stays `todo` and stays BEHIND `.17`, but its scope is now *"price three designs"*, not
*"build a precedence service"*. Sequencing unchanged.

### `.14` — the INTERPRETER and the generated parser disagree on an un-eliminated left-recursive cycle, in BOTH directions (`todo`, opened 2026-08-12 session #222 by `.13` slice 3)

⛔ **This is a defect in an ORACLE, which is worse than a defect in a parser.** The
`PARSE-HARNESS.5`/`.6.1` gates certify the grammar-AST interpreter byte-identical to the real
generated parser, and `.6.1`'s stated reach is *"trusted on ANY grammar built from PGEN's structural
constructs"*. Indirect left recursion is such a construct — the suite even carries a case for it
(`recursion_guarded_memo_isolation`). On a six-rule synthetic the two implementations return
**opposite verdicts on two of five inputs**:

| input | GEN (generated parser, authoritative by construction) | INTERP (`--interpret-parse`) |
|---|---|---|
| `t'(n)` | accept | **reject@3** |
| `n'(n)` | **reject@0** | accept |

Grammar + driver + full matrix:
`docs/tasks/artifacts/engine_universal_services/indirect_lr/` (`p1_knot_a_defect.ebnf`, `probe.sh`).
Both P2 and P3 — the same cycle, eliminated — agree on **every** row, so the divergence is specific
to a **surviving** cycle, i.e. exactly where `--lint-grammar` reports
`left_recursion_unhandled > 0`.

**WHY (mechanism, already documented — its CONSEQUENCE was not).** The generated parsers block on
`check_cycle_id`, whose verdict names one blocking frame; the interpreter *has no cycle guard at
all* — a whole-stack depth ceiling is its entire runtime-cycle-breaking path
(`rust/src/parse_harness_interpreter.rs:746-749`, where the difference is written down as a
deliberate design note). The note explains why the two memo-taint mechanisms differ; nothing said
the two could return different VERDICTS, and nothing measured whether they do.

**WHY THE SUITE IS GREEN ANYWAY.** `recursion_guarded_memo_isolation`'s cycle
(`cast → call → recv → cast`) is escapable one hop in: its seed `fn` sits inside the first rule the
cycle enters, so no probe input ever needs the recursion to be *entered twice*. The
`p1_knot_a_defect` shape does — and that is the shape every real victim on `.13` has.

⭐⭐ **ROUTED IN — BOTH DIRECTIONS AS A ONE-DIFFERENCE PAIR, which this leaf's title asserts and has
never had a fixture for** (`.17` slice 8, 2026-08-14 session #231). `guard_parses/probe.sh` carries
**six** declared divergence rows across two ~20-line grammars that differ by one line, produced as a
by-product of measuring something else:

| | `1A` — entry has a second alternative | `2A` — it does not |
|---|---|---|
| `e2` `k = n'(n)'(n);` | GEN **REJECT** · INTERP ACCEPT | both REJECT |
| `e1` `k = n'(n);` | both ACCEPT | GEN **ACCEPT** · INTERP **REJECT** |

⭐ **One mechanism explains the flip**, and it sharpens this leaf's WHY rather than merely adding a
case: the interpreter's whole-stack depth ceiling is reached while *evaluating* the cyclic
alternative — which `longest_match` requires in order to compare it — so whether the parse survives
depends on whether an alternative OUTSIDE the cycle is still standing to win. ⇒ **the direction the
divergence bites in is a property of the surrounding grammar, not of the cycle.**

Grammars: `guard_parses/{s1_guard_source,s2_holder_only}.ebnf`. ⛔ **This does not change this leaf's
verdict, scope or acceptance** — it is a better fixture for (a) than `p1_knot_a_defect` in two ways:
both directions in one file pair, and the same files' B arms show the divergence VANISHING once the
cycle is eliminated, which is the control (a) needs to prove the case is about a *surviving* cycle
rather than about the grammar. ⭐ And the divergences are DECLARED per oracle rather than treated as a
bank failure — the pattern any `.14` fixture needs, since a bank that fails on disagreement cannot
measure the arm where they disagree.

⛔ **THIS DOES NOT BLOCK `.13`, AND THIS LEAF'S FIRST DRAFT SAID IT DID** (corrected the same
session, before the next slice started — see `.13` slice 3's own correction block). A combinator
case is clean only when `agreed && anchor_ok`: mirroring `check_cycle_id` here would move the two
oracles to AGREEING on REJECT, leaving the spec-reasoned ACCEPT anchor unmet, so it cannot green the
case. Conversely `.13` (d) greens it WITHOUT this leaf, because both sides run the same
`transform_from_raw_ast` elimination and would consume the identical eliminated tree — the exact
path bare DIRECT LR took from `DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE` to a green first-class case
when `GRAMMAR-WELLFORMED.A2.5` landed. ⇒ **`.14` runs in PARALLEL with `.13`**, and `.13` (d)
shrinks its exposure (fewer surviving cycles in the shipped grammars) without closing it — a
synthetic or a future grammar can always present one.

⚠️ **AND IT ALREADY COST A WRONG TABLE.** `.13` slice 3's first draft ran the interpreter alone and
published P1's two divergent rows inverted — a table that read "even one cast level fails", which
would have sent the fix after a mechanism that does not exist. Caught only because the scratch slot
was run for the trace. There is a precedent for the honest interim posture: bare DIRECT left
recursion was an explicitly-classified `DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE` until
`GRAMMAR-WELLFORMED.A2.5` removed its cause (`docs/tasks/PARSE-HARNESS.md:1311`). This is the
INDIRECT twin, and it is currently unclassified, unmeasured and inside the certified claim.

**Acceptance:** (a) the divergence reproduced as a first-class case in the `.6.1` suite (RED at
first, which is the point); (b) the honest scope of `PARSE-HARNESS.5`/`.6.1` corrected in the suite
docstring, `TOOLBOX.md` §1.5/§1.5b/§1.6/§1.7 and the book — a certification claim that is false on a
named shape is repaired by narrowing the claim OR by fixing the interpreter, never by leaving the
sentence standing; (c) the decision between mirroring `check_cycle_id` in the interpreter and
classifying the shape as a KNOWN divergence, priced — ⭐ noting that `.13` (d) will REMOVE the
surviving cycles from the shipped grammars, which shrinks the exposure but does not close it (a
synthetic or a future grammar can always present one); (d) a sweep for other surviving cycles whose
verdicts differ — the 7 SV + 3 `ebnf` distinct cycles of `.13` slice 1 are the obvious first
denominator.
