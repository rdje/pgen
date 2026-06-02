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
  Status: `active` (research `.1`–`.7` DONE; director review DONE — Tier A greenlit → owned leaves `.8`–`.11`; FRONTIER = `.8` A1 well-formedness check, begins once `SV-EXH-PROOF.7.4.3` commits)
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
  Status: `done` (`-0002`; research — PEG/packrat parsing theory & the runtime parser model)
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
  Status: `done` (`-0002`; research — parser-generator architecture, EBNF→IR→codegen)
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
  Status: `done` (`-0002`; research — semantic actions, attribute grammars & the annotation/store layer)
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
  Status: `done` (`-0002`; research — AST/IR design, lossless syntax trees, the JSON shape-contract)
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
  Status: `done` (`-0002`; research — error recovery & diagnostics for signoff quality)
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
  Status: `done` (`-0002`; synthesis — capability-gap audit + prioritized adoption backlog in docs/tasks/PARSE-SOTA-research-synthesis.md §1)
  Goal: `Synthesize .1–.5 into ONE prioritized, risk-assessed adoption backlog: for
  each candidate technique — what it is (citation), the gain (more signoff/robust/
  accurate/precise), the blast radius, the NO-WORKAROUNDS fix-hierarchy level, and
  which task tree/leaf would own the adoption. Rank by value/risk. Explicitly mark
  what we already do well (don't reinvent) vs the real gaps. Director-facing.`
  Acceptance: `a single ranked backlog doc; every item cited + mapped to an owning
  tree/leaf; clear "already-strong, leave alone" list; director review gate before
  any adoption is scheduled.`

- ID: `PARSE-SOTA.7`
  Status: `done` (`-0003`, 2026-06-02, pure-docs / live-book)
  Goal: `Per director directive 2026-06-02 ("all literature references — books/articles/papers + authors — shall be thoroughly documented in the top-level mdBook") and [[feedback_regex_book_live]]: add a thorough, citable Academic Foundations chapter to the top-level mdBook consolidating EVERY reference from BOTH groundings (this tree + SV-EXH-PROOF.7.3), each with authors/title/venue/year/URL + how it grounds PGEN.`
  Acceptance: `new chapter docs/book/src/academic-foundations.md + SUMMARY.md entry; HTML rebuilt in lockstep (docs/book-html); mdbook_docs_gate green; organized by area (grammar-based test generation, PEG/packrat theory, parser generators, attribute grammars/scope graphs, AST/IR & lossless trees, error recovery); verification-flag note carried.`
  Verification: `done — docs/book/src/academic-foundations.md (6 sections A–F, ~60 primary sources, each authors+title+venue+year+URL + PGEN-relevance), added to SUMMARY.md after Developer Architecture; mdbook build OK (academic-foundations.html 58 KB rendered); make -C rust mdbook_docs_gate = "✅ mdBook docs gate passed". Cross-references both in-repo syntheses (PARSE-SOTA-research-synthesis.md + SV-EXH-PROOF-7.3-...). Pure docs; no code; no release bump.`
  Commit: `PGEN-PARSE-SOTA-0003`

- ID: `PARSE-SOTA.8` (adoption A1 — static grammar well-formedness check)
  Status: `analysis DONE` (`-0005`, 2026-06-02; left-recursion detection landed; wiring = `.8.1`)
  Verification: `done (analysis sub-step) — new self-contained module rust/src/ast_pipeline/grammar_wellformedness.rs: detect_left_recursion(grammar, rule_order) -> Vec<WellformednessIssue> via Ford's well-formedness (POPL 2004 §3.6) — a nullable fixpoint (conservative: terminals never-nullable, so NO false positives) + a left-edge call graph (Sequence extends past a nullable leading element; Quantified/Lookahead expose their element at the left edge) + cycle detection (records the recursion path for the diagnostic). PARSER-AGNOSTIC, pure analysis (no I/O, no generation), in a NEW file (no conflict with the in-flight .7.4.3). 5 unit tests: direct (expr:=expr...), indirect (a->b->a), nullable-prefix left recursion DETECTED; the iterative `next (OP next)*` idiom + a consuming-prefix chain NOT flagged (no false positives). lib (no-features) 577/577 (+5); clippy 0 errors. WIRING (reject at generate time via pgen_error! + real-grammar no-false-positive verification on SV/VHDL/regex/JSON/EBNF) is sub-step .8.1, after .7.4.3 commits (frees the generate path). NON-TERMINATING-rule detection (reuses .7.4.2 min-length) also deferred to .8.1.`
  Commit: `PGEN-PARSE-SOTA-0005`
  Goal: `Static well-formedness analysis over the compiled grammar IR (Ford, PEG, POPL
  2004 §3.6): detect (a) LEFT-RECURSIVE rules (a rule reachable from itself through only
  nullable left-edge positions → infinite loop / stack overflow at runtime) and (b)
  NON-TERMINATING rules (no finite terminal derivation — reuses the .7.4.2 min-length
  fixpoint: a rule absent from the min-length table is non-terminating). Surface findings
  as an always-on diagnostic via the DIAG-SEVERITY pgen_error! channel at grammar-compile/
  generate time — so a loop-prone grammar is REJECTED with a clear message instead of
  hanging at runtime. PARSER-AGNOSTIC; self-contained analysis module (no engine/runtime
  change). Start analysis-only + unit-tested (mirror .7.4.2), wire the reject as a 2nd
  sub-step.`
  Acceptance: `a grammar_wellformedness analysis fn + unit tests (left-recursive grammar
  detected; nullable-loop detected; well-formed grammar passes); reject path emits via
  pgen_error!; lib+clippy green; no false positives on the shipped grammars (SV/VHDL/regex/
  JSON/EBNF). NO grammar/generated change for the analysis; wiring is a measured sub-step.`

- ID: `PARSE-SOTA.9` (adoption A2 ⭐ — static ordered-choice shadowing lint)
  Status: `pending` (code; director-greenlit 2026-06-02)
  Goal: `The out-of-the-box win: a static lint over each Or node that flags an
  alternative UNREACHABLE because an earlier alternative subsumes its prefix (the PEG
  "A := a | ab" quirk; ALL(*) critique) — the exact catch-all-shadows-specific defect
  class the SV work keeps hitting (T::P, the Architecture-B detour). FIRST-set / prefix-
  subsumption analysis; emit compile-time WARNINGS via pgen_warn!. No mainstream PEG
  generator ships a general shadowing lint — PGEN can own it. Couples with the stimuli/
  replay coverage (high grammar coverage ↔ high code coverage).`
  Acceptance: `a shadowing/unreachable-alternative analysis + unit tests (a shadowed alt
  flagged; a genuinely-reachable set passes); pgen_warn! emission; tuned to NO false
  positives on the shipped grammars (else it is noise); lib+clippy green.`

- ID: `PARSE-SOTA.10` (adoption A4 — round-trip / determinism + golden-file tests)
  Status: `pending` (code/tests; director-greenlit 2026-06-02)
  Goal: `Close the meaning-drift gap the structural ast_shape_contract misses (Pactflow;
  Rendel-Ostermann): add (i) a parser-determinism + structural-idempotence property over
  the corpus, (ii) golden-file input→expected-JSON AST snapshots per grammar, and (once
  A5/_meta lands) a per-node parse(node._meta.source_text) re-parse oracle. Test-harness
  only; no codegen change.`
  Acceptance: `determinism/idempotence property test; golden-file AST snapshots wired into
  the drift gate; lib green; documented in the book.`

- ID: `PARSE-SOTA.11` (adoption A5 — ship the approved `_meta` carrier)
  Status: `pending` (code; director-greenlit 2026-06-02; design approved [[feedback_meta_carrier_design]])
  Goal: `Emit the approved additive `_meta` sibling key (span / line_col / source_text /
  trivia) on typed AST nodes. Additive → schema-compatible (no shape-contract break) per
  schema-evolution rules (Roslyn/rowan fidelity model — but NOT a red-green/incremental
  rewrite). Unlocks A4's round-trip oracle + the linter lane.`
  Acceptance: `_meta emitted per node; additive (existing shapes + schema unchanged);
  ast_shape_contract green; lib green; book updated.`

---

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `PARSE-SOTA.1`–`.6` | `done` (`-0002`, 2026-06-02) | 5 parallel literature sweeps + synthesis landed in docs/tasks/PARSE-SOTA-research-synthesis.md. KEY: the existing flow is a recognized published architecture (store-gates-rules = SPEG/Nez/data-dependent grammars; memo delta-replay = Laurent & Mens SLE 2016; RETURN = synthesized attributes; codegen = staged combinators) — VALIDATED, not idiosyncratic. Prioritized adoption backlog §1: Tier A (A1 well-formedness check, A2 ordered-choice shadowing lint ⭐, A3 labeled failures, A4 round-trip/golden-file testing, A5 ship `_meta`) all engine-untouched; Tier B (B1 memo-soundness audit ⭐, B2 cut operator, B3 parametric rules); Tier C (C1 scope graphs, C2 error recovery, C3 grammar modules); + an explicit do-NOT-adopt list (GLL/GLR/Earley engine swap, red-green trees, incremental parsing, runtime left-recursion). |
| — | director review | `done` (2026-06-02) | Director greenlit Tier A (A1/A2/A4/A5) → now owned leaves `.8`–`.11`. |
| 1 | `PARSE-SOTA.8` (A1 well-formedness) | `pending` (frontier) | Static left-recursion + non-terminating-rule detection, rejected via pgen_error!; reuses .7.4.2 min-length + DIAG-SEVERITY channel. Lowest-risk Tier-A win. Starts once SV-EXH-PROOF.7.4.3 commits (frees stimuli_generator.rs). |
| 2 | `PARSE-SOTA.9` (A2 ⭐ shadowing lint) | `pending` | The out-of-the-box ordered-choice shadowing lint (attacks the recurring SV catch-all defect class). |
| 3 | `PARSE-SOTA.10` / `.11` (A4 round-trip / A5 `_meta`) | `pending` | Robustness + fidelity. |

## Decisions

- `2026-06-02` (creation): tree opened by director directive to back the
  parser-generator path with literature, in parallel with the stimuli-path grounding
  (`SV-EXH-PROOF.7.3`). Research-only tree; adoptions spin out as owned leaves and are
  measured. Parser-agnostic + no-workarounds fix-hierarchy govern any adoption.
