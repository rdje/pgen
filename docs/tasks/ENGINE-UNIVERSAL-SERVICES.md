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

#### ⛔⛔ `.17` NEW `todo` — chain absorption cannot close SystemVerilog's cast/call knot, because PGEN's `*` is greedy and never retries at a lower iteration count (opened 2026-08-13 session #224 by `.13` slice 5)

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

###### ⭐⭐ FINDING 1 — the same engine gives back at a CHOICE and refuses to at a QUANTIFIER

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

###### THE DESIGN SPACE AS IT NOW STANDS (slice 2 decides; this slice does not)

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

###### ⛔⛔ THE QUALIFIER ON EVERY NUMBER ABOVE — FEASIBLE IS NOT CLOSED, AND IT IS MEASURED AT 157/157

`guard-feasible 16/28` means *a guard is expressible and provably **sound** at 16 candidates*. It
does **not** mean 16 knots close, and the report now says so in the output rather than in prose: `~`
marks an **over-approximated** byte set, i.e. one where the guard passes at positions the residual
cannot actually start from and therefore silently declines to refuse the fatal iteration.

```text
guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=1
```

**Measured: 157 of 157 sites are over-approximated — not one exact guard exists on either grammar**
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
  ratio at **157/157 over-approximated**, and a deliberately flipped expectation exits rc 1 naming
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
