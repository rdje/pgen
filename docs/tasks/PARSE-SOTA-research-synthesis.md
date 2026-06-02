# PARSE-SOTA — research synthesis + prioritized adoption backlog (.1–.6)

> Research/synthesis. **No code change.** Pure docs.
> Owner: `PARSE-SOTA` (`.1`–`.5` research + `.6` synthesis). Commit: `PGEN-PARSE-SOTA-0002`.
> Method: 5 parallel literature agents (~150 web searches/fetches), primary-source
> citations, verbatim-flagged. Full agent reports in the commit transcript.
> Director frame (2026-06-02): *"improve the EXISTING EBNF parser-generator flow — it
> is good already — but ground it with available knowledge, get as much inspiration as
> possible, don't reinvent the wheel, and still think out-of-the-box."*

---

## 0. TL;DR — the flow is well-founded; here is where to sharpen it

**The biggest finding is reassuring: PGEN's design is a recognized, published architecture,
not an ad-hoc one.** Five separate literature sweeps independently mapped our constructs
onto named, peer-reviewed formalisms — in several cases the field published *our exact
design*. So the directive "don't reinvent the wheel" cuts both ways: we already built the
right wheel; the work now is to (a) cite it, (b) add the cheap static checks the field
has that we lack, and (c) pick the few high-value upgrades.

### What the literature VALIDATES (keep — now with citations)
| Our construct | Published as | Source |
|---|---|---|
| `@emit_fact` + `@predicate`-gates-a-rule-during-parse + scope tree | **symbol-table-based context-sensitive PEG** — published *twice, essentially identically* | SPEG (Maeda & Kuramitsu, SLE 2017); Nez (Kuramitsu) |
| same, formal model | **data-dependent grammars** (binding + constraints control parsing) | Yakker (Jim–Mandelbaum–Walker, POPL 2010); Iguana (CC 2016) |
| memo entry re-fires semantic side effects via a stored "delta" | **principled stateful parsing** (snapshot/restore/**diff/merge**) — same word, same fix | Laurent & Mens, SLE 2016 |
| the bug we fixed (memo cached result, not side effects) | the exact "statelessness" limitation Ford warns about | Ford, ICFP 2002, §5.2 |
| RETURN annotations `-> {kind, field:$N}` | **synthesized attributes** | Knuth, 1968 |
| consult a symbol table during parse for type-vs-expr (SV/C) | the standard, accepted technique ("lexer hack"; Clang Sema) | Clang/C++ literature |
| codegen EBNF→IR→emit specialized Rust | **staged (partially-evaluated) combinator parsing** | Parsley (Willis–Wu–Pickering, ICFP 2020) |
| parser-agnostic "events → tree", tree-shape independent of engine | the rust-analyzer architecture | rowan / rust-analyzer |
| `furthest_position` diagnostics | Ford's **furthest-failure** heuristic (the PEG baseline) | Ford, 2002/2004 |

**Implication:** our standing disciplines are independently corroborated. "A `@predicate`
must be a pure function of facts" = the data-dependent-grammar well-formedness condition
*and* the formal justification for our `try_parse must snapshot semantic state` invariant
([[feedback_try_parse_must_snapshot_semantic_state]]). "Declarative annotations, no
arbitrary host code, parser-agnostic primitives" = Nez's exact design rationale and our
[[feedback_no_workarounds_fix_hierarchy]] level bar.

---

## 1. Prioritized adoption backlog (the `.6` deliverable)

Ranked by value ÷ blast-radius. **Tier A** = high value, low blast radius, engine
untouched (fits [[feedback_prefer_grammar_leave_engine_alone]]). Each item names the
NO-WORKAROUNDS fix-hierarchy level and a candidate owning tree. **Nothing here is
scheduled — director reviews this backlog first.**

### Tier A — high value, low risk (IR / codegen / tooling; engine untouched)

**A1. Static grammar well-formedness check (left-recursion + nullable-repetition).**
Ford's fixed-point analysis (POPL 2004 §3.6) conservatively detects left-recursive and
empty-loop rules and *rejects them at generate time with a clear error*, instead of
emitting a parser that infinite-loops / stack-overflows at runtime. Guards every grammar
PGEN compiles (SV/VHDL/regex/JSON + future). Pure IR-compiler pass; no codegen/engine
change. **Gain:** robustness/safety. **Blast radius:** IR analysis only. **Level 3**
(general parser-agnostic generator capability). *Both the parsing-theory and the
architecture sweep ranked this the #1 cheap win.*

**A2. Static ordered-choice shadowing / unreachable-alternative lint.** ⭐ *out-of-the-box*
The PEG `A := a | ab` quirk ("`ab` can never match") is **exactly the SV defect class we
keep hitting** — provisional/catch-all rules silently shadowing more specific ones (e.g.
`T::P` wrongly accepted, the `.5.2.4.1` fix; the retired Architecture-B detour). ANTLR's
ALL(*) authors call silent first-match a genuine defect. A FIRST-set + prefix-subsumption
analysis over each ordered choice can emit compile-time warnings ("alternative N is
unreachable / shadowed by alternative M"). **No mainstream PEG generator ships a general
shadowing lint — PGEN could own this.** Building blocks: FIRST-set computation
(Swierstra–Duponcheel) + Mizushima's cut-point analysis. **Gain:** turns a class of silent
mis-parses into generate-time diagnostics — directly serves WHY+WHERE-before-fix
([[feedback_why_and_where_before_solution]]). **Blast radius:** IR static analysis +
warnings. **Level 3.** Couples with our existing stimuli/replay coverage machinery
(ALL(*): high grammar coverage ↔ high code coverage) — *this is where the parser-generator
and stimuli-generator groundings connect.*

**A3. Labeled failures (`⇑l` / `[p]ˡ`) + rustc-style structured diagnostics.** A label is a
per-grammar-point annotation — native to our `@`-EBNF idiom. Codegen lowers `[p]ˡ` to "if
`p` fails after this commit point, stop backtracking and emit `expected <label>` at the
failure position." Turns our biggest diagnostic pain — deep, contextless `furthest_position`
byte offsets in uvm_pkg — into *"expected `)` to close port_list, found `;`"*. Composes
with (doesn't replace) `furthest_position`. **Gain:** signoff-grade error messages; the
foundation every recovery technique builds on. **Blast radius:** annotation-language +
codegen + `parseability_probe`; engine essentially untouched. **Level 3.** (Maidl et al.,
SCP 2016; Medeiros et al., SBLP 2013.) Phase-1 of a 3-phase recovery path (see C-tier).

**A4. Round-trip / determinism property testing + golden-file AST snapshots in the drift
gate.** Our `ast_shape_contract` is *structural* — it catches shape drift but, per the
contract-testing literature (Pactflow), cannot catch *meaning* drift (a field whose value
silently changes). Add: (i) a property that parse is deterministic + structurally
idempotent over the corpus; (ii) golden-file input→expected-JSON snapshots per grammar;
(iii) once `_meta.source_text` ships (A5), a per-node `parse(node._meta.source_text)`
re-parse oracle. **Gain:** closes the meaning-drift hole; makes `_meta` an executable
oracle. **Blast radius:** test-harness only, no codegen. **Level 3.** (Rendel–Ostermann
2010; PBT round-trip literature.)

**A5. Ship the approved `_meta` carrier (span / line_col / source_text / trivia).** The
entire serious-tooling industry (Roslyn, rust-analyzer/rowan, SwiftSyntax, tree-sitter) is
lossless/full-fidelity and converges on the same invariants. `_meta` is the *additive*
first step — gets ~80% of the fidelity benefit (round-trip via stored source_text;
IDE-grade positions) at AST-emit + codegen blast radius, and as an additive sibling key it
is **schema-compatible (no shape-contract break)** per schema-evolution rules. Honest
limit: per-node `_meta` is coarser than Roslyn's token-owned trivia (can't place a comment
*between* two abstract fields). **Gain:** fidelity, round-trip, linter/IDE readiness.
**Blast radius:** AST emit + codegen. **Level 3.** Already director-approved
([[feedback_meta_carrier_design]]). *Do NOT adopt red-green trees or incremental parsing —
those are in-memory live-edit optimizations with zero payoff for a batch JSON emitter.*

### Tier B — high value, engine-adjacent (needs care / measurement first)

**B1. Memoization-soundness audit → conditional memoization.** ⭐ *the deepest correctness
question.* Ford (ICFP 2002 §5.2) and Chida et al. (CC 2020) warn that once memoization
depends on mutable state, (a) reusing a memo entry computed under a *different* store can
be **unsound**, and (b) naive handling can go **super-linear** (recall our real uvm_pkg
furthest-position explosions). Action = a tools-first investigation (no code change first):
audit whether our memo *key* ignores the store (unsoundness risk for `@predicate`-gated
rules) or keys on the whole store (quadratic risk). The literature's fix is **conditional
memoization** — key the memo on only the store-slice a rule actually consulted (Chida et
al.). **Gain:** correctness + perf robustness. **Blast radius:** investigation = none;
fix = engine-core (Level 5, last resort, explicit auth). The audit is the highest-value
*next investigation* and is squarely tools-first per
[[feedback_no_codebase_change_without_tool_backed_facts]].

**B2. Cut operator + codegen auto-insertion.** Mizushima (PASTE 2010): after a committed
keyword in an ordered choice, sibling alternatives can't apply, so those memo entries (and
our snapshotted semantic states) are dead and can be released — "mostly constant space."
Highest perf/space lever; doubly valuable for us because every memo entry also stores a
semantic delta. **Gain:** perf + memory. **Blast radius:** codegen static analysis (the
cut points) + engine release logic. **Level 5** (engine), but codegen-driven.

**B3. Parametric / templated rules.** rust-peg has them; pest notably doesn't. Would kill
the `X (sep X)*` / `::N*` extraction-spread boilerplate that our memory already records as
a pain ([[feedback_quantified_group_extraction]]). e.g. `list<T>`, `sep_by<T,S>`.
**Gain:** grammar expressiveness/maintainability. **Blast radius:** grammar-language +
codegen; zero engine. **Level 3.**

### Tier C — larger, aspirational (own trees; for the linter/IDE future)

**C1. Scope graphs** (Néron–Tolmach–Visser–Wachsmuth, ESOP 2015) generalize our scope-tree
+ `resolve_path` + `@import_from_library` into first-class typed **edges** (lexical /
import / inheritance) resolved by one language-parametric calculus. Would make
cross-package `extends`, `Class::member`, imports, and shadowing *principled* rather than
hand-rolled — exactly the SV surfaces we keep fighting. Caveat: scope graphs are a
*post-parse* model; parse-time resolution is research-grade ("Knowing When to Ask", critical
edges, OOPSLA 2020). **Big store-engine lift; parser-agnostic (Level 5).** Smaller stepping
stone: adopt Nez's `<block>`/`<local>` scoping vocabulary into the annotation language.

**C2. Error recovery → multi-error + partial AST.** After A3 (labels): recovery expressions
`R(l)` emitting typed ERROR/MISSING nodes + FOLLOW-resync (Medeiros & Mascarenhas, SAC
2018; tree-sitter's node contract), then continue-after-error for many errors per file
(ANTLR cascade suppression; CPCT+ rank-by-cost *idea*), then automatic FIRST/FOLLOW
labeling at scale (2025) — accepting documented degradation on SV's overlapping
alternatives. The LINTER lane needs this. **Phased; annotation-language + codegen.**

**C3. Grammar module system / composition** (Rats!, PLDI 2006). PEG is composition-closed;
a module layer (import/modify/compose) would help the *huge* SV grammar. **Grammar-language
+ IR; larger lift.**

### Explicitly DO NOT adopt (don't reinvent / wrong fit — the "don't reinvent the wheel" list)
- Switching the engine to GLL / GLR / Earley / Marpa — ~135× slower on real corpora (ALL(*)
  measurement), and we *want* a single deterministic AST, not a parse forest.
- Dynamic (parse-time) ALL(*) ambiguity detection — conflicts with PEG determinism; we get
  the same safety statically via A2.
- Red-green / persistent trees + incremental parsing — in-memory live-edit optimizations;
  zero payoff for a batch JSON emitter (revisit only if PGEN becomes editor-resident).
- Runtime left-recursion (Warth seed/grow; Medeiros bounded) — our grammar-authoring
  elimination idiom works and avoids a partial-semantic-side-effect hazard in our memo.

---

## 2. Per-branch detail (condensed; full reports in transcript)

### .1 PEG/packrat theory & runtime model
- We are a faithful PEG (ordered choice = first-match commit; `&`/`!` restore position;
  greedy possessive quantifiers) — Ford POPL 2004. Our packrat memo = Ford ICFP 2002; its
  known cost is space ∝ input size (matters at uvm scale).
- **Crux:** packrat assumes a *pure* parse function; we read+write a store, violating it.
  Our delta-replay is the published-correct fix (Laurent & Mens, SLE 2016). Open half:
  memo-reuse soundness + linear-time under state (Chida CC 2020 → conditional memoization).
  → **B1, A1.**
- Left recursion: we eliminate at authoring time (correct choice); runtime support (Warth
  2008 / Medeiros 2014) not worth the side-effect hazard. → do-not-adopt.
- Cut (Mizushima 2010) + bounded memo (Redziejowski) for space. → **B2.**

### .2 Parser-generator architecture & codegen
- We belong to the PEG-RD-codegen family (pest/rust-peg/LPeg/Rats!); staged combinator
  parsing (Parsley) is the academic name for our codegen.
- **Top safety gap:** ordered-choice shadowing (ALL(*) critique) → **A2**; well-formedness
  → **A1**. **Top expressiveness wins:** parametric rules (rust-peg) → **B3**; module system
  (Rats!) → **C3**.
- Our `@predicate` store IS data-dependent grammar (Yakker/Iguana) — adopt the vocabulary +
  the "predicate = pure function of bound facts" well-formedness rule.
- Don't switch to general parsing (GLL/GLR/Earley/Marpa).

### .3 Attribute grammars, scope graphs, the annotation/store layer
- RETURN = synthesized attributes (Knuth). Store-gates-rules = SPEG/Nez/data-dependent —
  *our design is published, sound, packrat-safe.*
- scope-tree + resolve_path + import = an ad-hoc partial **scope graph** (ESOP 2015). →
  **C1** highest-value principled upgrade (typed edges, language-parametric resolution,
  robust imports/shadowing; parse-time integration is research-grade).
- Reference attribute grammars / JastAdd: first-class use→decl binding edges + demand-driven
  declarative scheduling (vs our manual fact ordering — relevant to `.7.x` ordering pain).

### .4 AST/IR design, lossless trees, schema/contract robustness
- Industry tooling is uniformly lossless/full-fidelity (Roslyn/rowan/SwiftSyntax/tree-sitter)
  with shared invariants. `_meta` is the right additive first step → **A5**; not a red-green
  rewrite.
- Schema: name-keyed JSON has no rename protection (unlike Protobuf field numbers);
  classify additive vs breaking; "never rename, add+deprecate". Structural contracts miss
  meaning drift → golden files + round-trip → **A4**.

### .5 Error recovery & diagnostics
- `furthest_position` = the PEG baseline; its weakness (no expected-context, error known
  only after unwind) is exactly ours.
- **Labeled failures** map cleanly onto our `@`-EBNF → **A3** (Phase 1: messages). Then
  recovery expressions (Phase 2) and automatic FIRST/FOLLOW labeling (Phase 3) → **C2**.
- Adopt tree-sitter's ERROR/MISSING *output contract* and rustc's diagnostic *structure*;
  don't adopt their engines (GLR / LR-table).

---

## 3. Out-of-the-box opportunities (director frame: "still think out-of-the-box")
1. **A general PEG ordered-choice shadowing lint (A2)** — no mainstream generator ships
   one; PGEN could be first, and it directly attacks our recurring SV defect class.
2. **Couple the shadowing lint with our stimuli/replay coverage (SV-EXH-PROOF.7)** —
   ALL(*) ties high grammar coverage to high code coverage; the parser-generator and
   stimuli-generator groundings reinforce each other (coverage *is* ambiguity testing).
3. **`_meta.source_text` as an executable round-trip oracle (A4+A5)** — turn a passive
   fidelity payload into a correctness test.
4. **Formalize + document PGEN as a "data-dependent PEG with a typed fact store"** — gives
   a checkable well-formedness rule and a citable identity; positions PGEN against the
   literature rather than as a one-off.

---

## 4. Bibliography (primary)
PEG/packrat: Ford POPL 2004 https://bford.info/pub/lang/peg.pdf · Ford ICFP 2002
https://bford.info/pub/lang/packrat-icfp02.pdf · Warth PEPM 2008
https://web.cs.ucla.edu/~todd/research/pepm08.pdf · Medeiros SCP 2014
https://www.inf.puc-rio.br/~roberto/docs/sblp2012.pdf · Mizushima PASTE 2010
https://kmizu.github.io/papers/paste513-mizushima.pdf · Redziejowski FI 2008
https://www.romanredz.se/papers/FI2008.pdf · Laurent & Mens SLE 2016
https://arxiv.org/abs/1609.05365 · Chida et al. CC 2020
https://dl.acm.org/doi/10.1145/3377555.3377898 · Nez https://arxiv.org/abs/1511.08414
Generators: ALL(*) OOPSLA 2014 https://www.antlr.org/papers/allstar-techreport.pdf · GLL
https://dotat.at/tmp/gll.pdf · Earley CACM 1970
https://dl.acm.org/doi/10.1145/362007.362035 · Marpa https://arxiv.org/abs/1910.08129 ·
Yakker POPL 2010 https://www.cs.princeton.edu/~dpw/papers/ddgrammars-0709.pdf · Iguana CC
2016 https://ir.cwi.nl/pub/25126/25126.pdf · LPeg
https://www.inf.puc-rio.br/~roberto/docs/lpeg-primer.pdf · Rats! PLDI 2006
https://dl.acm.org/doi/10.1145/1133255.1133987 · Hutton & Meijer
https://people.cs.nott.ac.uk/pszgmh/monparsing.pdf · Parsley ICFP 2020
https://mpickering.github.io/papers/parsley-icfp.pdf · rust-peg
https://github.com/kevinmehall/rust-peg
Attribute grammars / scope graphs: Knuth 1968
https://link.springer.com/article/10.1007/BF01692511 · Hedin RAG 2000
https://portal.research.lu.se/en/publications/reference-attributed-grammars · JastAdd
OOPSLA 2007 https://dl.acm.org/doi/10.1145/1297027.1297029 · Silver
https://melt.cs.umn.edu/silver/ · A Theory of Name Resolution ESOP 2015
https://web.cecs.pdx.edu/~apt/esop15.pdf · Scopes as Types OOPSLA 2018 (DOI
10.1145/3276484) · Knowing When to Ask OOPSLA 2020
https://research.tudelft.nl/en/publications/knowing-when-to-ask · SPEG SLE 2017
https://dl.acm.org/doi/10.1145/3136014.3136025
AST/IR: Roslyn red-green
https://github.com/dotnet/roslyn/blob/main/docs/compilers/Design/Red-Green%20Trees.md ·
rowan https://github.com/rust-analyzer/rowan · SwiftSyntax
https://github.com/swiftlang/swift-syntax · tree-sitter
https://tree-sitter.github.io/tree-sitter/ · Wagner & Graham TOPLAS 1998
https://harmonia.cs.berkeley.edu/papers/twagner-parsing.pdf · Rendel & Ostermann 2010
https://ps.informatik.uni-tuebingen.de/publications/rendel10invertible/
Error recovery: Maidl et al. SCP 2016 https://arxiv.org/abs/1405.6646 · Medeiros &
Mascarenhas SAC 2018 https://arxiv.org/abs/1806.11150 · auto-recovery 2025
https://arxiv.org/abs/2507.03629 · Burke & Fisher TOPLAS 1987
https://dl.acm.org/doi/10.1145/22719.22720 · CPCT+ ECOOP 2020
https://arxiv.org/abs/1804.07133 · de Jonge et al. TOPLAS 2012
https://dl.acm.org/doi/10.1145/2400676.2400678 · rustc diagnostics
https://rustc-dev-guide.rust-lang.org/diagnostics.html

**Verification flags (carried from agents):** Chida CC 2020 read at abstract level (ACM
full text blocked). Purdom-class secondary-source descriptions where PDFs were binary
scans. Wagner & Graham abstract reconstructed from corroborating snippets. Roslyn reuse %
is vendor-reported. All load-bearing mappings above were read from primary sources.
