# Academic Foundations

PGEN is not an ad-hoc design. Every stage of its pipeline — the EBNF + annotation
front-end, the JSON IR, the parser-agnostic AST pipeline, the generated PEG parsers,
the semantic store, and the stimuli generator — corresponds to a published,
peer-reviewed body of work. In mid-2026 the project ran two deliberate
literature-grounding efforts that mapped each construct onto its academic source, so
that future work draws on decades of existing knowledge rather than re-deriving it.

This chapter is the consolidated, citable bibliography for those efforts. It is the
user-facing companion to two in-repo research syntheses:

- **Stimuli generator → literal-0 coverage:**
  `docs/tasks/SV-EXH-PROOF-7.3-literature-grounded-literal-zero-design.md`
- **Parser-generator path:** `docs/tasks/PARSE-SOTA-research-synthesis.md`

Each entry below gives **authors, title, venue, year, and a stable URL**, followed by
*how it grounds PGEN*. A short verification note closes the chapter: where a source
was read only at abstract level, the synthesis docs flag it explicitly.

> Why this matters. The headline finding of both efforts is reassuring: PGEN's design
> is a *recognized* architecture, not idiosyncratic. The semantic store that gates
> rules during parsing is the published **data-dependent grammar** / symbol-table-PEG
> model; the memoization delta-replay is published **principled stateful parsing**;
> the return annotations are **synthesized attributes**; the codegen is **staged
> combinator parsing**. Knowing the names lets us adopt the field's static checks and
> upgrades — and avoid reinventing what already exists.

---

## A. Grammar-based test generation & grammar coverage (the stimuli generator)

The stimuli generator's goal — generate inputs that exercise every reachable
production/branch of a grammar, driving the closed-loop replay residual toward
**literal zero** — is the academic problem of *grammar-based test generation* under a
*grammar coverage* criterion.

- **Paul Purdom.** "A sentence generator for testing parsers." *BIT Numerical
  Mathematics* 12(3):366–375, **1972**. <https://link.springer.com/article/10.1007/BF01932308>
  — *The foundational result: a constructive, deterministic algorithm that emits a
  small set of short sentences exercising every production at least once. Basis of
  PGEN's `.7.4` shortest-derivation (min-terminal-length) table and the
  one-minimal-witness-per-obligation literal-0 plan.* Canonical reconstruction: Malloy
  & Power, "An Interpretation of Purdom's Algorithm…," ICIS 2001,
  <https://mural.maynoothuniversity.ie/id/eprint/6434/1/JP-Purdoms-algorithm.pdf>.
- **Matthew Hennessy & James F. Power.** "An analysis of rule coverage as a criterion
  in generating minimal test suites for grammar-based software." *ASE* **2005**.
  <https://dl.acm.org/doi/10.1145/1101908.1101926> (extended: *Empirical Software
  Engineering* 13(4), 2008, <https://link.springer.com/article/10.1007/s10664-008-9067-7>)
  — *Empirical proof that 100% rule coverage on real grammars (ISO C++) is reached only
  by constructively adding the last witnesses (set-cover completion), not by sampling
  more — the direct justification for PGEN's decoupled witness approach over steering.*
- **Ralf Lämmel.** "Grammar Testing." *FASE* **2001**, LNCS 2029:201–216.
  <https://link.springer.com/chapter/10.1007/3-540-45314-8_15>
  — *Introduces context-dependent branch coverage (CDBC), the criterion that PGEN's
  per-(rule, node_path, branch_index) Branch targets implement.*
- **Ralf Lämmel & Wolfram Schulte.** "Controllable Combinatorial Coverage in
  Grammar-Based Testing." *TestCom* **2006**, LNCS 3964:19–38.
  <https://link.springer.com/chapter/10.1007/11754008_2>
  — *Depth/recursion-bounded, controllable combinatorial coverage — the framework for
  "literal-0 up to depth d" if PGEN strengthens beyond rule coverage.*
- **Nikolas Havrikov & Andreas Zeller.** "Systematically Covering Input Structure."
  *ASE* **2019**. <https://havrikov.github.io/publications/ase19-preprint.pdf>
  (journal extension, CISPA: <https://publications.cispa.saarland/3572/>)
  — *k-path grammar coverage; the "cover-on-the-way" greedy set-cover that yields a
  small, diverse witness set — the modern shape of PGEN's witness emission.*
- **Andreas Zeller, Rahul Gopinath, Marcel Böhme, Gordon Fraser, Christian Holler.**
  "Grammar Coverage," chapter in *The Fuzzing Book*.
  <https://www.fuzzingbook.org/html/GrammarCoverageFuzzer.html>
  — *The directly-implementable two-tier "uncovered-first, then uniform-random"
  expansion rule — the antidote to PGEN's diversity-collapse failure.*
- **Peter M. Maurer.** "Generating Test Data with Enhanced Context-Free Grammars"
  (DGL). *IEEE Software* 7(4):50–55, **1990**.
  <https://dl.acm.org/doi/10.1109/52.56422>
  — *The weighted-grammar ancestor; cautionary — weights bias the distribution but give
  no coverage guarantee (the lever that exhausted at residual 888).*
- **William M. McKeeman.** "Differential Testing for Software." *Digital Technical
  Journal* 10(1):100–107, **1998**.
  — *Canonical grammar-driven mass generation feeding a closed loop.*
- **Alain Denise, Marie-Claude Gaudel, Sandrine-Dominique Gouraud, Richard Lassaigne,
  Johan Oudinet, Sylvain Peyronnet.** "Coverage-biased random exploration of large
  models and application to testing." *STTT* 14(1):73–93, **2012**.
  <https://webusers.imj-prg.fr/~richard.lassaigne/articles/sttt-2012.pdf>
  — *The mathematically correct version of "steer toward uncovered targets": count-based
  uniform sampling + an LP maximizing the minimum per-element coverage probability,
  diversity-preserving.* Grammar specialization: **Dreyfus, Héam, Kouchnarenko**,
  "Random Grammar-based Testing for Covering All Non-Terminals," arXiv:1311.6606, 2013,
  <https://arxiv.org/pdf/1311.6606>.
- **Philippe Flajolet, Paul Zimmermann, Bernard Van Cutsem.** "A calculus for the
  random generation of labelled combinatorial structures." *Theoretical Computer
  Science* 132:1–35, **1994**.
  <https://www.sciencedirect.com/science/article/pii/0304397594902267>
  — *The recursive (counting-based) method for uniform sampling of grammar sentences —
  a diversity-preserving background generator that cures shallow-derivation skew.*
- **Philippe Duchon, Philippe Flajolet, Guy Louchard, Gilles Schaeffer.** "Boltzmann
  samplers for the random generation of combinatorial structures." *Combinatorics,
  Probability and Computing* 13(4–5):577–625, **2004**.
  <https://dl.acm.org/doi/10.1017/S0963548304006315>
  — *Fast near-uniform size-targeted sampling; scales the diverse-background idea to
  SystemVerilog-sized inputs.*
- **Olivier Bodini & Yann Ponty.** "Multi-dimensional Boltzmann Sampling of
  Context-Free Languages." *AofA* **2010**. <https://arxiv.org/abs/1002.0046>
  — *Uniform sampling while constraining each production to a target frequency — the
  bridge between diversity and forcing the stragglers.*
- **Cornelius Aschermann et al.** "NAUTILUS: Fishing for Deep Bugs with Grammars."
  *NDSS* **2019**.
  <https://www.ndss-symposium.org/ndss-paper/nautilus-fishing-for-deep-bugs-with-grammars/>
  — *AST-level (not byte-level) mutation so coverage feedback is never wasted on
  parse-rejected inputs.*
- **Junjie Wang, Bihuan Chen, Lei Wei, Yang Liu.** "Superion: Grammar-Aware Greybox
  Fuzzing." *ICSE* **2019**. <https://arxiv.org/abs/1812.01197>
  — *Tree-level trimming — minimizing a witness before adding it to the corpus.*
- **Rohan Padhye, Caroline Lemieux, Koushik Sen, Mike Papadakis, Yves Le Traon.**
  "Semantic Fuzzing with Zest." *ISSTA* **2019**.
  <https://rohan.padhye.org/files/zest-issta19.pdf>
  — *Two-tier coverage bookkeeping (total vs valid) — analogue of PGEN's separate
  parsed-and-covered metric.*
- **Sameer Reddy, Caroline Lemieux, Rohan Padhye, Koushik Sen.** "Quickly Generating
  Diverse Valid Test Inputs with Reinforcement Learning" (RLCheck). *ICSE* **2020**.
  <https://rohan.padhye.org/files/rlcheck-icse20.pdf>
  — *Documents PGEN's exact diversity-collapse failure mode: a target-hit reward without
  a novelty term collapses to one template; the fix is novelty-relative reward.*
- **Junjie Wang, Bihuan Chen, Lei Wei, Yang Liu.** "Skyfire: Data-Driven Seed
  Generation for Fuzzing." *IEEE S&P* **2017**.
  <https://www.ieee-security.org/TC/SP2017/papers/42.pdf>
  — *Diversity via deliberately up-weighting rare productions + capping per-rule reuse.*
- **Prashast Srivastava & Mathias Payer.** "Gramatron: Effective Grammar-Aware
  Fuzzing." *ISSTA* **2021**. <https://hexhive.epfl.ch/publications/files/21ISSTA.pdf>
  — *Grammar-automaton normalization to eliminate structural sampling bias (the
  shallow-derivation skew behind hard-to-reach branches).*
- **Renáta Hodován, Ákos Kiss, Tibor Gyimóthy.** "Grammarinator: A Grammar-Based Open
  Source Fuzzer." *A-TEST* **2018**.
  <https://www.inf.u-szeged.hu/~akiss/pub/fulltext/hodovan2018grammarinator.pdf>
  — *The weight-cooldown baseline; cautionary — no coverage guarantee, the plateau
  behaviour PGEN observed.*
- **Marcel Böhme, Van-Thuan Pham, Manh-Dung Nguyen, Abhik Roychoudhury.** "Directed
  Greybox Fuzzing" (AFLGo). *CCS* **2017**. <https://mboehme.github.io/paper/CCS17.pdf>
  — *Reach a target by adding energy on an annealing schedule — never by reshaping the
  population; the phase-separation discipline PGEN's witness pass follows.*

---

## B. PEG & packrat parsing theory (the runtime parser model)

- **Bryan Ford.** "Parsing Expression Grammars: A Recognition-Based Syntactic
  Foundation." *POPL* **2004**. <https://bford.info/pub/lang/peg.pdf>
  — *The formal foundation of PGEN's ordered choice (`|`), `&`/`!` predicates,
  quantifiers, and the well-formedness condition (PARSE-SOTA A1 static check).*
- **Bryan Ford.** "Packrat Parsing: Simple, Powerful, Lazy, Linear Time." *ICFP*
  **2002**. <https://bford.info/pub/lang/packrat-icfp02.pdf>
  — *PGEN's memoization model; §5.2 "statelessness" names exactly the bug class PGEN
  fixed with semantic-delta replay.*
- **Alessandro Warth, James R. Douglass, Todd Millstein.** "Packrat Parsers Can
  Support Left Recursion." *PEPM* **2008**.
  <https://web.cs.ucla.edu/~todd/research/pepm08.pdf>
  — *Runtime left-recursion (seed/grow); PGEN deliberately uses grammar-authoring
  elimination instead, to avoid partial-side-effect hazards in its memo.*
- **Sérgio Medeiros, Fabio Mascarenhas, Roberto Ierusalimschy.** "Left recursion in
  Parsing Expression Grammars." *Science of Computer Programming* 96:177–190, **2014**
  (SBLP 2012). <https://www.inf.puc-rio.br/~roberto/docs/sblp2012.pdf>
  — *Bounded left recursion with a conservativeness proof; the principled reference if
  PGEN ever adopts native left recursion.*
- **Kota Mizushima, Atusi Maeda, Yoshinori Yamaguchi.** "Packrat parsers can handle
  practical grammars in mostly constant space" (the cut operator). *PASTE* **2010**.
  <https://kmizu.github.io/papers/paste513-mizushima.pdf>
  — *Cut-point analysis to release dead memo entries — PGEN's highest space/perf lever
  (PARSE-SOTA B2).*
- **Roman R. Redziejowski.** "Some Aspects of Parsing Expression Grammar."
  *Fundamenta Informaticae* 85(1-4):441–451, **2008**.
  <https://www.romanredz.se/papers/FI2008.pdf>
  — *Empirical case that full memoization is often unnecessary; a tiny bounded cache
  captures most of the benefit — relevant since each PGEN memo entry also stores a delta.*
- **Nicolas Laurent & Kim Mens.** "Taming Context-Sensitive Languages with Principled
  Stateful Parsing." *SLE* **2016**. <https://arxiv.org/abs/1609.05365>
  — *Independently describes PGEN's exact memo bug AND fix (snapshot / restore / diff /
  merge, "delta") — the formal backing for PGEN's semantic-delta replay.*
- **Nariyoshi Chida et al.** "Is Stateful Packrat Parsing Really Linear in Practice? …
  conditional memoization." *CC* **2020**.
  <https://dl.acm.org/doi/10.1145/3377555.3377898>
  — *Warns that stateful memoization can go super-linear; "conditional memoization"
  (key on only the consulted store-slice) is the fix PGEN should study (PARSE-SOTA B1).*
- **Tetsuro Matsumura & Kimio Kuramitsu.** "A Declarative Extension of Parsing
  Expression Grammars for Recognizing Most Programming Languages" (Nez).
  arXiv:1511.08414, **2015**. <https://arxiv.org/abs/1511.08414>
  — *Symbol-table operators (`<def>`/`<is>`/`<block>`) = PGEN's `@emit_fact`/
  `@predicate`/scope; validates the design as sound and packrat-safe.*

---

## C. Parser-generator architecture, codegen & general parsing

- **Terence Parr, Sam Harwell, Kathleen Fisher.** "Adaptive LL(\*) Parsing: The Power
  of Dynamic Analysis" (ANTLR4). *OOPSLA* **2014**.
  <https://www.antlr.org/papers/allstar-techreport.pdf>
  — *Names PEG's silent first-match (`a | ab`) as a real defect — the basis of PGEN's
  proposed ordered-choice shadowing lint (PARSE-SOTA A2); its semantic-predicate example
  is exactly the SystemVerilog type-vs-identifier case PGEN's store solves.*
- **Trevor Jim, Yitzhak Mandelbaum, David Walker.** "Semantics and Algorithms for
  Data-dependent Grammars" (Yakker). *POPL* **2010**.
  <https://www.cs.princeton.edu/~dpw/papers/ddgrammars-0709.pdf>
  — *The formal model of "computation during parse": variable binding + constraints +
  parameterized nonterminals = PGEN's `@emit_fact`/`@predicate`/store, with a
  proven-correct semantics.*
- **Ali Afroozeh & Anastasia Izmaylova.** "Iguana: a practical data-dependent parsing
  framework." *CC* **2016**. <https://ir.cwi.nl/pub/25126/25126.pdf>
  — *Confirms a global parse-time store is the right tool for C/SV type tables, and that
  precedence/layout/keyword-gating desugar to data-dependent primitives.*
- **Elizabeth Scott & Adrian Johnstone.** "GLL Parsing." *ENTCS* 253(7), **2010**.
  <https://dotat.at/tmp/gll.pdf> — *Recursive-descent-shaped general parsing; reference
  point for "what PGEN would be without determinism" (deliberately not adopted).*
- **Masaru Tomita.** *Efficient Parsing for Natural Language* (GLR). Kluwer, **1985**.
  <https://link.springer.com/book/10.1007/978-1-4757-1885-0> — *Generalized LR / parse
  forests; ambiguity-as-output (not adopted; PGEN wants one deterministic AST).*
- **Jay Earley.** "An Efficient Context-Free Parsing Algorithm." *CACM* 13(2), **1970**.
  <https://dl.acm.org/doi/10.1145/362007.362035> — *Foundational general CFG parsing.*
- **Jeffrey Kegler.** "Marpa, A practical general parser: the recognizer."
  arXiv:1910.08129, **2019**. <https://arxiv.org/abs/1910.08129> — *Modern Earley with
  strong error/event reporting; studied for diagnostics ideas, not as an engine swap.*
- **Roberto Ierusalimschy.** "A text pattern-matching tool based on Parsing Expression
  Grammars" (LPeg). *Software: Practice & Experience* 39(3), **2009**.
  <https://www.inf.puc-rio.br/~roberto/docs/lpeg-primer.pdf> — *First-class captures =
  conceptual parent of PGEN's return annotations.*
- **Robert Grimm.** "Better Extensibility through Modular Syntax" (Rats!). *PLDI*
  **2006**. <https://dl.acm.org/doi/10.1145/1133255.1133987> — *PEG composition / a
  grammar module system — the model for managing the large SystemVerilog grammar
  (PARSE-SOTA C3).*
- **pest** (Rust PEG generator) <https://pest.rs/> and **rust-peg** (`peg` crate)
  <https://github.com/kevinmehall/rust-peg> — *Mature PEG generators; rust-peg's
  parametric/templated rules motivate PARSE-SOTA B3 (kill `X (sep X)*` boilerplate).*
- **Graham Hutton & Erik Meijer.** "Monadic Parser Combinators." Nottingham
  NOTTCS-TR-96-4, **1996** (JFP 8(4), 1998).
  <https://people.cs.nott.ac.uk/pszgmh/monparsing.pdf> — *The theoretical shape of
  PGEN's generated per-rule functions returning typed values.*
- **S. Doaitse Swierstra & Luc Duponcheel.** "Deterministic, Error-Correcting
  Combinator Parsers." *AFP* **1996**, LNCS 1129.
  <https://link.springer.com/chapter/10.1007/3-540-61628-4_7> — *Static FIRST-set
  lookahead over a combinator structure — machinery a shadowing lint reuses.*
- **Jamie Willis, Nicolas Wu, Matthew Pickering.** "Staged Selective Parser
  Combinators" (Parsley). *Proc. ACM Program. Lang.* 4(ICFP):120, **2020**.
  <https://mpickering.github.io/papers/parsley-icfp.pdf> — *The academic name for
  PGEN's codegen: stage (partially evaluate) the grammar into hand-written-quality
  specialized parser code.*

---

## D. Attribute grammars, name resolution & the annotation/store layer

- **Donald E. Knuth.** "Semantics of Context-Free Languages." *Mathematical Systems
  Theory* 2(2):127–145, **1968** (correction: 5(1):95–96, 1971).
  <https://link.springer.com/article/10.1007/BF01692511> — *Synthesized vs inherited
  attributes; PGEN's return annotations are synthesized attributes.*
- **Görel Hedin.** "Reference Attributed Grammars." *Informatica* 24(3), **2000**.
  <https://portal.research.lu.se/en/publications/reference-attributed-grammars> —
  *First-class use→declaration binding edges — a more principled form of PGEN's
  string-keyed facts.*
- **Torbjörn Ekman & Görel Hedin.** "The JastAdd Extensible Java Compiler." *OOPSLA*
  **2007**. <https://dl.acm.org/doi/10.1145/1297027.1297029> — *Modular, declarative,
  demand-driven name/type analysis — the mature version of "compute facts over a tree."*
- **Eric Van Wyk et al.** "Silver: An Extensible Attribute Grammar System." *Science of
  Computer Programming* (LDTA 2008). <https://melt.cs.umn.edu/silver/> — *Forwarding +
  modular well-definedness analysis.*
- **Harald Vogt, S. Doaitse Swierstra, Matthijs Kuiper.** "Higher-Order Attribute
  Grammars." *PLDI* **1989**. <https://dl.acm.org/doi/10.1145/73141.74830> —
  *Attributes that are themselves trees.*
- **Eva Magnusson, Torbjörn Ekman, Görel Hedin.** "Demand-driven evaluation of
  collection attributes." *Automated Software Engineering* 16, **2009**.
  <https://link.springer.com/article/10.1007/s10515-009-0046-z> — *Lazy, declarative
  attribute scheduling — relevant to PGEN's manual fact-ordering pain.*
- **Pierre Néron, Andrew Tolmach, Eelco Visser, Guido Wachsmuth.** "A Theory of Name
  Resolution." *ESOP* **2015**, LNCS 9032:205–231.
  <https://web.cecs.pdx.edu/~apt/esop15.pdf> — *Scope graphs: a language-parametric
  resolution calculus over scopes + edges (lexical/import/inheritance). PGEN's scope
  tree + `resolve_path` + `@import_from_library` are an ad-hoc partial scope graph; the
  highest-value principled upgrade (PARSE-SOTA C1).*
- **Hendrik van Antwerpen, Casper Bach Poulsen, Arjen Rouvoet, Eelco Visser.** "Scopes
  as Types." *Proc. ACM Program. Lang.* 2(OOPSLA):114, **2018**. (DOI 10.1145/3276484)
  — *Unifies name resolution and type checking over one scope graph (Statix).*
- **Arjen Rouvoet, Hendrik van Antwerpen, Casper Bach Poulsen, Robbert Krebbers, Eelco
  Visser.** "Knowing When to Ask: Sound Scheduling of Name Resolution…" *Proc. ACM
  Program. Lang.* 4(OOPSLA), **2020**.
  <https://research.tudelft.nl/en/publications/knowing-when-to-ask> — *"Critical edges"
  / query stability: when a resolution query is safe to answer mid-parse — directly
  relevant to PGEN's parse-time fact-ordering.*
- **Kimio Maeda & Kimio Kuramitsu.** "A Symbol-Based Extension of Parsing Expression
  Grammars and Context-Sensitive Packrat Parsing" (SPEG). *SLE* **2017**.
  <https://dl.acm.org/doi/10.1145/3136014.3136025> — *Publishes essentially PGEN's
  store-gates-rules design, with a unified symbol-table state, restricted predicates,
  and packrat-safety.*

---

## E. AST / IR design, lossless syntax trees & contract robustness

- **Roslyn (Microsoft).** Red-green trees & full-fidelity syntax model. Design docs:
  <https://github.com/dotnet/roslyn/blob/main/docs/compilers/Design/Red-Green%20Trees.md>;
  Eric Lippert, "Persistence, façades and Roslyn's red-green trees" (2012),
  <https://ericlippert.com/2012/06/08/red-green-trees/> — *The reference full-fidelity
  model (every byte round-trips; trivia on tokens; dual span). Informs PGEN's `_meta`
  carrier (PARSE-SOTA A5). Red-green itself is NOT adopted — an in-memory edit
  optimization with no payoff for a batch JSON emitter.*
- **rust-analyzer / rowan.** Lossless syntax trees. Architecture:
  <https://rust-analyzer.github.io/book/contributing/architecture.html>; rowan:
  <https://github.com/rust-analyzer/rowan> — *"Parser independent of tree
  representation; events → tree" mirrors PGEN's parser-agnostic design; untyped lossless
  core + typed layer on top is the model `_meta` approximates.*
- **SwiftSyntax (Apple).** Full-fidelity syntax trees.
  <https://github.com/swiftlang/swift-syntax> — *Confirms the lossless invariants;
  shows why an abstract tree caps format-preserving codegen/fixup (the linter lane).*
- **tree-sitter.** Incremental, error-tolerant concrete syntax trees.
  <https://tree-sitter.github.io/tree-sitter/> — *The IDE-grade CST + ERROR/MISSING
  node output contract PGEN's recovery should emit.*
- **Tim A. Wagner & Susan L. Graham.** "Efficient and Flexible Incremental Parsing."
  *ACM TOPLAS* 20(5):980–1013, **1998**.
  <https://harmonia.cs.berkeley.edu/papers/twagner-parsing.pdf> — *Foundational
  incremental parsing (LR-based; not portable to PEG — deferred unless PGEN becomes
  editor-resident).*
- **Tillmann Rendel & Klaus Ostermann.** "Invertible Syntax Descriptions: Unifying
  Parsing and Pretty Printing." *Haskell Symposium* **2010**.
  <https://ps.informatik.uni-tuebingen.de/publications/rendel10invertible/> — *One
  description generates both parser and printer → a free round-trip guarantee; the
  principled long-term target behind PGEN's round-trip property testing (PARSE-SOTA A4).*
- **Schema/contract practice (industry):** Confluent Schema Registry compatibility
  modes
  <https://docs.confluent.io/platform/current/schema-registry/fundamentals/schema-evolution.html>;
  Pactflow on contract testing
  <https://pactflow.io/blog/the-case-for-contract-testing-protobufs-grpc-avro/>; Adrian
  Sampson, "Try Snapshot Testing for Compilers"
  <https://www.cs.cornell.edu/~asampson/blog/turnt.html> — *Classify additive vs
  breaking changes; structural contracts miss meaning drift → golden-file + round-trip
  tests close it (PARSE-SOTA A4). PGEN's `_meta` carrier is an additive, schema-compatible
  change.*

---

## F. Error recovery & diagnostics (signoff-grade messages)

- **André Murbach Maidl, Fabio Mascarenhas, Sérgio Medeiros, Roberto Ierusalimschy.**
  "Error reporting in Parsing Expression Grammars" (labeled failures). *Science of
  Computer Programming* 132(1), **2016**. <https://arxiv.org/abs/1405.6646>
  (predecessor: SBLP 2013,
  <https://link.springer.com/chapter/10.1007/978-3-642-40922-6_1>) — *Labeled failures
  (`⇑l` / `[p]ˡ`) map cleanly onto PGEN's `@`-annotation EBNF; turn contextless
  `furthest_position` offsets into "expected X" (PARSE-SOTA A3).*
- **Sérgio Medeiros & Fabio Mascarenhas.** "Syntax Error Recovery in Parsing Expression
  Grammars." *SAC* **2018**. <https://arxiv.org/abs/1806.11150> — *Recovery expressions:
  skip to a sync point + emit a typed error node → a partial AST.* Automatic
  FIRST/FOLLOW labeling: *"Towards Automatic Error Recovery in PEGs"*, SBLP 2018 / arXiv
  2025, <https://arxiv.org/abs/2507.03629>.
- **Michael G. Burke & Gerald A. Fisher.** "A practical method for LR and LL syntactic
  error diagnosis and recovery." *ACM TOPLAS* 9(2), **1987**.
  <https://dl.acm.org/doi/10.1145/22719.22720> — *The insert/delete/substitute repair
  family; conceptual grounding.*
- **Lukas Diekmann & Laurence Tratt.** "Don't Panic! Better, Fewer, Syntax Errors for
  LR Parsers" (CPCT+). *ECOOP* **2020**. <https://arxiv.org/abs/1804.07133> — *Min-cost
  repair + cascade suppression; PGEN adopts the rank-by-cost idea, not the LR algorithm.*
- **Maartje de Jonge, Lennart Kats, Eelco Visser, Emma Söderberg.** "Natural and
  Flexible Error Recovery for Generated Modular Language Environments." *ACM TOPLAS*
  34(4):15, **2012**. <https://dl.acm.org/doi/10.1145/2400676.2400678> — *Auto-derive
  recovery rules from the grammar; layout as a recovery signal.*
- **ANTLR4 `DefaultErrorStrategy`**
  <https://www.antlr.org/api/Java/org/antlr/v4/runtime/DefaultErrorStrategy.html>;
  **rustc diagnostics** <https://rustc-dev-guide.rust-lang.org/diagnostics.html>; **Elm
  "compiler errors for humans"** <https://elm-lang.org/news/compiler-errors-for-humans>
  — *The target message structure (span + message + suggestion + applicability) and the
  human-centered philosophy for PGEN's diagnostics and the linter lane.*

---

## How to read these references

The two in-repo synthesis documents carry **verification flags**: where a source could
only be read at abstract level (publisher paywall or binary-scanned PDF), it is marked
there explicitly, and any load-bearing claim was cross-checked against a primary source.
A few specifics worth restating:

- Purdom's 1972 algorithm structure is verified across multiple secondary sources (the
  original is a binary scan); the reconstruction by Malloy & Power is the readable form.
- Chida et al. (CC 2020) was read at abstract level (ACM full text blocked).
- The Roslyn "memory reuse %" figures are vendor-reported, not independently benchmarked.

When adopting any technique from this chapter, follow the project's standing discipline:
gather the proving fact tools-first, change one thing at a time, and measure the global
metric before the next step. The literature tells us *what* is sound and *where* the
gaps are; the task trees and gates prove that *our* implementation is correct.
