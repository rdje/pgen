# PARSE-SOTA — Ground the EBNF→parser-generator path in academic literature

> Task tree. Owns the literature-grounding of the **parser-generator path**:
> `EBNF (return + semantic annotations) → JSON IR → AST pipeline → generated parser`.
> Sibling of the stimuli-path grounding (`SV-EXH-PROOF.7.3`/`.7.4`).
>
> **Metadata**
> - Status: `active`
> - Created: 2026-06-02 (`PGEN-PARSE-SOTA-0001`)
> - Roadmap lane: cross-cutting engine quality — make the parser-generator path more
>   signoff / robust / accurate / precise by adopting decades of published results,
>   **without reinventing the wheel**.
> - Director directive (2026-06-02): *"weight the EBNF (return annotation + semantic
>   annotation) → JSON → AST pipeline → parser generator path against the literature,
>   academia, research articles … take whatever we can use to make the parser
>   generator path even more signoff, more robust, more accurate, more precise …
>   by no means reinvent the wheel … back all the paths from EBNF to parser and
>   stimuli generators by academia."*
> - Disciplines: [[feedback_research_grounded_sota_no_trial_and_revert]],
>   [[feedback_no_codebase_change_without_tool_backed_facts]],
>   [[feedback_ast_pipeline_parser_agnostic]],
>   [[feedback_prefer_grammar_leave_engine_alone]],
>   [[feedback_no_workarounds_fix_hierarchy]].

---

## Root

- ID: `PARSE-SOTA`
  Status: `active` (frontier — research branches `.1`–`.5` then synthesis `.6`)
  Goal: `Survey the published state of the art for every stage of the PGEN
  parser-generator path and weigh OUR implementation against it; produce a
  citation-backed capability-gap audit and a prioritized adoption backlog that
  makes the path more signoff/robust/accurate/precise. Research is pure docs;
  any code adoption it recommends is spun out as its OWN task-tree leaf and
  measured per the no-regression discipline. PARSER-AGNOSTIC throughout.`
  Acceptance: `each of the 5 stage branches has a citation-backed research doc
  framing our design in the field's terms + naming concrete adoptable techniques;
  .6 synthesizes a single prioritized, risk-assessed adoption backlog (what to
  adopt, why, expected gain, blast radius, which existing tree/leaf would own it);
  NO code change under this tree (research only) — recommendations become leaves
  elsewhere. Director reviews the backlog before any adoption work is scheduled.`
  Verification: `pending`
  Commit: `PGEN-PARSE-SOTA-0001 (tree creation)`

---

## Leaves

- ID: `PARSE-SOTA.1`
  Status: `pending` (research — PEG/packrat parsing theory & the runtime parser model)
  Goal: `Survey PEG + packrat parsing theory and weigh our recursive-descent +
  memoization runtime against it: Ford (PEG, POPL 2004; packrat, ICFP 2002);
  left-recursion in packrat (Warth, Douglass, Millstein 2008; Medeiros et al.);
  cut / space-efficient packrat (Mizushima, Maeda, Yamaguchi 2010; Redziejowski);
  memoization correctness with side effects (relevant to our memoization × semantic-
  store delta-replay engine work, .b.6.2.36.4); ordered-choice semantics &
  PEG-vs-CFG expressiveness. Output: where our model matches the literature, where
  it diverges, and any correctness/robustness technique we should adopt.`
  Acceptance: `citation-backed doc; explicit mapping to our engine (memoization,
  try_parse atomicity, ordered choice, the semantic-delta replay); named adoptable
  techniques w/ blast-radius note.`

- ID: `PARSE-SOTA.2`
  Status: `pending` (research — parser-generator architecture, EBNF→IR→codegen)
  Goal: `Survey parser-generator and codegen architecture: recursive-descent codegen;
  parser combinators (Hutton & Meijer); ANTLR ALL(*) adaptive LL(*) (Parr, Harwell,
  Fisher, OOPSLA 2014); GLL (Scott & Johnstone); GLR (Tomita); Earley (1970) / Marpa
  (Kegler); data-dependent grammars & Yakker (Jim, Mandelbaum, Walker, POPL 2010);
  tree-sitter's GLR-based incremental generator. Weigh our EBNF→JSON-IR→Rust-codegen
  generator against these: what determinism/expressiveness/ambiguity-handling
  guarantees they give that we don't, and which are worth adopting given our PEG model.`
  Acceptance: `citation-backed doc; mapping to our codegen (ast_based_generator.rs);
  named techniques + whether each fits a PEG/recursive-descent generator; blast radius.`

- ID: `PARSE-SOTA.3`
  Status: `pending` (research — semantic actions, attribute grammars & the annotation/store layer)
  Goal: `Survey the theory under our return-annotation + semantic-annotation + semantic-
  store layer: attribute grammars (Knuth 1968); reference attribute grammars & JastAdd
  (Hedin; Ekman & Hedin); circular/higher-order AGs; scope graphs & name resolution
  (Néron, Tolmach, Visser, Wachsmuth, "A Theory of Name Resolution", ESOP 2015;
  Spoofax/Statix); symbol tables as a first-class artifact. Weigh our @emit_fact /
  @predicate / @predicate_def / @fact_kind / scope-tree / resolve_path store against
  the formalisms: is our store a (subset of a) scope graph? Are our predicates a
  (subset of) attribute equations? What proven primitive would make grammars express
  categorisation more cleanly (per feedback_grammar_rules_must_consult_store)?`
  Acceptance: `citation-backed doc; mapping of our store/annotation constructs to the
  formal models; named adoptable primitives; parser-agnostic + fix-hierarchy framing.`

- ID: `PARSE-SOTA.4`
  Status: `pending` (research — AST/IR design, lossless syntax trees, the JSON shape-contract)
  Goal: `Survey AST/IR & concrete-syntax design: concrete vs abstract syntax;
  lossless/full-fidelity syntax trees & red-green trees (Roslyn; Swift libSyntax;
  rust-analyzer rowan); tree-sitter CST; incremental parsing & reuse (Wagner & Graham
  1998); trivia/round-tripping; schema/versioning of an IR. Weigh our JSON IR + the
  _meta carrier design + the ast_shape_contract manifests + schema-versioning against
  these: fidelity (do we lose trivia/spans?), round-trip-ability, incrementality,
  and contract-drift robustness.`
  Acceptance: `citation-backed doc; mapping to our JSON IR / _meta / shape-contract /
  schema model; named adoptable techniques (e.g. lossless trees, incremental reuse);
  blast radius + which would be a new tree.`

- ID: `PARSE-SOTA.5`
  Status: `pending` (research — error recovery & diagnostics for signoff quality)
  Goal: `Survey syntax error recovery & diagnostics — central to "signoff/robust":
  panic-mode & error productions; Burke-Fisher; noise-skipping; PEG error reporting via
  furthest-failure & labeled failures (Ford; Maidl, Mascarenhas, Ierusalimschy,
  "Error reporting in Parsing Expression Grammars", 2016); automatic error recovery
  (de Jonge & Visser; Diekmann & Tratt "CPCT+", 2020); tree-sitter error recovery.
  Weigh our furthest_position diagnostics + parseability_probe against these: how far
  are we from production-grade error messages + recovery, and what is adoptable.`
  Acceptance: `citation-backed doc; mapping to our furthest_position/diagnostics;
  named adoptable recovery/reporting techniques; blast radius.`

- ID: `PARSE-SOTA.6`
  Status: `pending` (synthesis — capability-gap audit + prioritized adoption backlog)
  Goal: `Synthesize .1–.5 into ONE prioritized, risk-assessed adoption backlog: for
  each candidate technique — what it is (citation), the gain (more signoff/robust/
  accurate/precise), the blast radius, the NO-WORKAROUNDS fix-hierarchy level, and
  which task tree/leaf would own the adoption. Rank by value/risk. Explicitly mark
  what we already do well (don't reinvent) vs the real gaps. Director-facing.`
  Acceptance: `a single ranked backlog doc; every item cited + mapped to an owning
  tree/leaf; clear "already-strong, leave alone" list; director review gate before
  any adoption is scheduled.`

---

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PARSE-SOTA.1`–`.5` | `pending` | The 5 stage research branches — run in parallel (independent literature areas). |
| 2 | `PARSE-SOTA.6` | `pending` | Synthesis + prioritized adoption backlog once `.1`–`.5` land. |

## Decisions

- `2026-06-02` (creation): tree opened by director directive to back the
  parser-generator path with literature, in parallel with the stimuli-path grounding
  (`SV-EXH-PROOF.7.3`). Research-only tree; adoptions spin out as owned leaves and are
  measured. Parser-agnostic + no-workarounds fix-hierarchy govern any adoption.
