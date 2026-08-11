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

### `.8` — ⭐⭐ THE LR-ELIMINATION FOLD LEAKS AN INTERNAL REPRESENTATION INTO THE TYPED AST (`todo`, found 2026-08-11 by `GRAMMAR-WELLFORMED.A2.5`, **BLOCKS the A2.5 engine fix from shipping**)

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

## Acceptance Criteria (tree)

- The catalogue exists in the live book, every entry evidence-backed, and its counts are derived.
- The scar-tissue census is a re-runnable instrument, and its current output is published.
- The gap list is priced well enough for the director to choose from it.
- The responsibility-boundary rule is a decision record that a reviewer can cite.
