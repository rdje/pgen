# PARSE-HARNESS — general arbitrary-grammar parse capability (the grammar-AST interpreter + compile-and-run + scratch-register), each made 100% trustworthy

- Tree ID: `PARSE-HARNESS`
- Status: `active` (created 2026-07-05, session #37, `PGEN-PARSE-HARNESS-0001`) — frontier `.2` (Phase A, first authoritative-by-construction harness). `.1` DESIGN is this file.
- Roadmap lane: cross-cutting **tooling / diagnostics** — closes the "no cheap way to parse an input against an *arbitrary* grammar" capability gap surfaced by `GRAMMAR-WELLFORMED.A2.2`/`A2.3`.
- Director directive (2026-07-05): *"let's build this general grammar-AST interpreter … task-tree track all 3 ways … find a SOTA, signoff way to make (1) authoritative … we need to be able to 100% trust their outcome … their task-tree shall describe them in gory detail."*

---

## 1. Purpose & motivation

### 1.1 The capability gap (tool-confirmed, 2026-07-05)

While preparing the `GRAMMAR-WELLFORMED.A2.3` investigation (does the `FixedTerminalPrefix`
shadowing *hard gate* falsely flag `a | ab` as dead under PGEN's non-PEG-commit engine?), a
tools-first check confirmed there is **no cheap way to parse an arbitrary input string against a
throwaway ("synthetic") grammar**:

- `ast_pipeline foo.ebnf --generate-parser` emits **Rust source** — running it needs a compile.
- The only paths that drive the **real** parser are **registry-keyed** (compiled-in grammars only):
  `parser_registry::parse_and_cover` (what `--report-certificate-coverage` uses to verify witnesses,
  `rust/src/ast_pipeline/grammar_wellformedness.rs:1804`) and `embedding_api::parse_*`
  (`rust/src/embedding_api.rs:693+`). `parseability_probe --parse <grammar> <file>` is likewise
  registry-only.
- The only in-memory parsers in the tree are the **EBNF meta-grammar** parser (`EbnfParser`, which
  parses `.ebnf` *files*, not arbitrary grammars' inputs) and the registered embedding-API grammars.
- There is **no general grammar-AST interpreter** that parses input against an arbitrary grammar
  without codegen.

So a one-rule probe grammar like `r := "a" | "a" "b"` cannot be parsed on `"ab"` in one step to see
which alternative wins — the exact `--parse-dump-ast-pretty` + `--trace-rules` evidence style that
decisively settled `A2.2`. Nothing is *broken* (the platform is intentionally codegen-to-Rust); it is
a genuine, nameable **capability gap**.

### 1.2 Why it is worth building (general value, not one-off)

- **Linter-soundness proofs.** `A2.3` (and any future "is this shadowing/deadness verdict sound under
  the real engine?" question) needs to run the real parser on a *made-up* grammar. This is the
  immediate driver. Reinforces [[feedback_certifying_linter_trustworthiness]] (verify, don't trust).
- **Grammar-authoring debugging.** "Does *this* fragment parse *that* input, and with what AST?" is a
  daily question when authoring `.ebnf`. Today it requires the full register→codegen→compile ceremony.
- **Toolbox completeness.** `TOOLBOX.md`'s standing directive: *"If the existing tools cannot surface
  the WHY and WHERE, the next step is to BUILD a tool, not to speculate."* This is that tool.
- **Parser-agnostic + reusable** ([[feedback_ast_pipeline_parser_agnostic]],
  [[feedback_features_parser_agnostic_enable_all_parsers]]): keyed only on the grammar AST +
  annotations, usable for every grammar family, current and future.

### 1.3 What this tree delivers (the capability, precisely)

Given **(a)** an arbitrary grammar (a `.ebnf` file **or** its normalized generation-input AST) and
**(b)** an input string, produce the **parse verdict** (accept / reject, with `furthest_position` on
reject) and the **typed AST** — using semantics **identical to the shipped generated parsers** —
*without* the full production ceremony (registry wiring + codegen-to-Rust + compile-in). Delivered
via the **three** director-named approaches, each with an explicit trust proof.

---

## 2. The three delivery approaches (gory detail)

> **Shared IR insight (applies to all three).** The harness should operate on the **normalized
> generation-input AST** — the same IR `--dump-gen-ast` emits and that codegen consumes (post
> left-recursion-elimination, post-normalization). Parsing the *identical* representation the shipped
> parser is built from is what makes fidelity attainable, and it inherits LR-elimination for free.
> Reaching input from a raw `.ebnf` = the existing `ebnf_frontend` → normalize pipeline, then hand the
> gen-AST to the harness.

> **Shared semantic insight.** PGEN parsing is not purely structural: it runs **return-annotations**
> (AST shaping), **`@predicate`** store gates, **`@emit_fact`/the semantic store**, **`branch_policy`**
> (`longest_match` / `ordered` / `priority_first`, `semantic_directive_registry.rs:49`), and
> **lookahead** `&`/`!`. Any faithful harness must run the **same `semantic_runtime`** the generated
> parser runs. This is the crux of trust and drives the shared-core design (§3).

### 2.1 Approach (1) — the general grammar-AST INTERPRETER  *(authoritative by VERIFICATION)*

- **WHAT.** An in-process evaluator, roughly
  `interpret_parse(gen_ast, entry_rule, input, profile, seed) -> ParseOutcome`, that recursively
  evaluates gen-AST node kinds (`Or` / `Sequence` / `Quantified` / `Lookahead` / `Atom` /
  rule-reference) against the input cursor, **dynamically dispatching** each combinator instead of
  running generated match-arms.
- **HOW it must work (faithfully).** For every node kind it must reproduce the generated parser's
  behavior *bit for bit*:
  - ordered choice honoring the rule's `branch_policy` (default = the global `longest_match`; this is
    the exact knob that makes `A2.2`/`A2.3` non-PEG-commit — see
    [[project_earlier_always_matches_unsound_backtracking]]);
  - sequences with per-element `try_parse`/backtrack + cursor restore;
  - the unified quantifier engine (`?`/`*`/`+`/`{N}`/`{N,M}`/`{N,}`/`{,M}` via
    `parse_quantifier_bounds`, per-iteration atomicity, the zero-length guard, the safety limit);
  - lookahead `&e`/`!e` (consume-free, can fail);
  - atoms/terminals/regex-tokens, rule-refs (with **memoization** incl. the semantic-delta replay);
  - the **semantic layer**: `@predicate` evaluation against the store, `@emit_fact` + scope tree +
    rollback/transactions, `@export/@import` library, and **return-annotation folding** ($N / dotted /
    indexed / spread) into the typed AST;
  - the same error/`furthest_position` accounting.
- **TRUST — the hard one (this is a SECOND parser implementation).** Two legs (§3 is the full design):
  1. **Minimize the trusted surface (shared core).** The interpreter must **reuse the exact runtime
     helpers** the generated parser uses (`semantic_runtime.rs`, the memo, the quantifier engine, the
     predicate evaluator, the return-transform) — differing *only* in that it dispatches combinators by
     walking the gen-AST dynamically instead of via generated match-arms. Then equivalence is
     **near-structural**, not testing-dependent, and the residual divergence surface is just the thin
     dynamic-dispatch layer.
  2. **Differential-equivalence gate (the certifying oracle).** Verify the residual against the
     authoritative generated parser (§3.2 / leaf `.5`).
- **COST.** Highest up-front (a second dispatch path + the equivalence gate), but reusable and fast at
  use time (no `rustc`).
- **VALUE.** The in-process, fast, general capability the director named — THE tool for any
  grammar-authoring probe or linter-soundness question, once trusted.

### 2.2 Approach (2) — lightweight COMPILE-AND-RUN harness  *(authoritative by CONSTRUCTION)*

- **WHAT.** A driver `compile_and_parse(ebnf_or_gen_ast, entry, input) -> ParseOutcome` that runs the
  **real codegen**, compiles the emitted parser as a **throwaway** unit (a `cargo` fixture crate, a
  one-shot `rustc`, or a `trybuild`-style harness), and executes it on the input — no registry wiring,
  no ceremony beyond the compile.
- **TRUST.** **Authoritative by construction** — it *is* the real generated parser + the real runtime;
  the only trusted surface is the harness plumbing (codegen invocation + input/output marshalling),
  covered by a light integration test that reproduces a *registered* grammar's known verdict/AST.
- **COST.** Per-probe `rustc` compile (~seconds). Heavier per use, but authoritative and self-contained
  (no registry change) → the natural **CI oracle** for the equivalence gate.
- **VALUE.** "When in doubt, use the real thing." Also the independent oracle that validates approach
  (1) without touching the registry.

### 2.3 Approach (3) — SCRATCH-REGISTRATION path  *(authoritative by CONSTRUCTION)*

- **WHAT.** A blessed `grammars/scratch/*.ebnf` slot + **auto registry wiring** + a `make focus_scratch`
  target so a probe grammar becomes a **first-class registered parser** drivable by
  `parseability_probe --parse scratch <input>` — and therefore by the WHOLE toolbox
  (`--parse-dump-ast-pretty`, `--trace-rules`, `--report-certificate-coverage`, `--lint-grammar`) — in
  one command.
- **TRUST.** **Authoritative by construction** — identical to how shipped grammars are built and driven
  (same codegen + registry + `parseability_probe`). Trusted surface = the scratch registry wiring (a
  small, reviewable, one-time addition), covered by an integration test.
- **COST.** A registry-slot addition once; then per-probe = a `make focus_scratch` regen + build.
- **VALUE.** The most **production-faithful** path and the one that yields the **richest evidence** for
  `A2.3` (real `--trace-rules` branch-entry logs + `--parse-dump-ast-pretty` on the synthetic grammar —
  the exact A2.2 evidence style). Reuses the entire shipped pipeline + CLI toolbox.

---

## 3. Trustworthiness architecture — "100% trust their outcome" (the director's crux)

The three approaches have **different trust profiles**, and the design makes each one's trust
*earned and re-checkable* (never "trust me"), in the project's certifying-algorithm spirit
([[feedback_certifying_linter_trustworthiness]]; cf. the EBNF dual-run gate + the linter⟷generator
duality).

### 3.1 The trust hierarchy

| Approach | Trust basis | Trusted surface (what could still be wrong) | How that surface is checked |
|---|---|---|---|
| (2) compile-and-run | **BY CONSTRUCTION** (runs the shipped codegen + runtime) | the harness plumbing (codegen call, I/O marshalling, compile flags) | one integration test: reproduce a *registered* grammar's known verdict + AST through the harness |
| (3) scratch-register | **BY CONSTRUCTION** (identical to the shipped register→codegen→drive pipeline) | the scratch registry wiring | one integration test: a known scratch grammar → known verdict/AST via `parseability_probe` |
| (1) interpreter | **BY VERIFICATION** (shared core + differential-equivalence) | the thin dynamic-dispatch layer over the shared runtime | the differential-equivalence gate (§3.2) + the per-combinator suite (§3.3) |

(2) and (3) are the **cheap-trust** paths — they inherit the shipped parser's correctness and only add
plumbing. (1) is the **earn-your-trust** path, and §3.2–§3.4 is the SOTA/signoff way to make it
authoritative.

### 3.2 Making the INTERPRETER authoritative — differential equivalence (leaf `.5`)

**Thesis: don't trust the interpreter — VERIFY it, continuously, against the authoritative generated
parser, byte for byte.**

- For **every registered grammar** (systemverilog + all profiles, vhdl, regex, json, rtl_const_expr,
  rtl_frontend, systemverilog_preprocessor, the return/semantic/builtin annotation grammars, ebnf), run
  **both** the interpreter and the generated parser on the **same input corpus** and assert **identical**:
  (i) accept/reject verdict, (ii) `furthest_position` on reject, (iii) **byte-identical typed AST** on
  accept.
- **Corpus = the union of the strongest existing oracles** (all already in-repo, all already exercise
  ~every construct): the **stimuli generator's** seeded output (grammar-valid, deterministic, thousands
  of samples), the **external corpora** (`*_corpus_bundle/`, the SV/VHDL realistic corpora), the
  **AST-shape-contract** cases (`rust/test_data/ast_shape_contract/`), the **certificate-coverage
  witnesses**, and the curated **accept/reject matrices** (e.g. `verilog_2005_conformance`).
- **Deterministic** (seeds 0/7/42), **re-runnable**, **gated** (`parse_harness_equivalence_gate`) →
  it is an **ORACLE** (the un-fakeable leg per `DOCTRINE_ENFORCEMENT.md` §6.1). The interpreter is
  trusted **iff** byte-identical to the generated parser across the whole corpus. Any divergence is a
  hard failure that names the grammar + input + the diff.

### 3.3 The per-combinator differential suite (leaf `.6`) — load-bearing for "trust it on ANY grammar"

End-to-end equivalence on shipped grammars only covers the constructs **those** grammars use. To trust
the interpreter on a **synthetic** grammar (the whole point — e.g. `A2.3`'s `a | ab`), we need
equivalence **per combinator**. So: a suite of **small isolating grammars**, one per construct, each
differential-tested (interpreter vs a compiled/scratch real parser) over targeted inputs:

- ordered choice under **each** `branch_policy` (`longest_match` / `ordered` / `priority_first`),
  including the `a | ab` fixed-prefix shape and the `e? | keyword` always-succeeds shape (the A2.2/A2.3
  cases directly);
- sequence + backtrack; every quantifier form (`?`/`*`/`+`/`{N}`/`{N,M}`/`{N,}`/`{,M}`) incl. the
  zero-length guard; lookahead `&`/`!`; atom/terminal/regex-token; rule-ref; left-recursion
  (LR-eliminated); `@predicate` gate hit **and** miss; `@emit_fact` + query + scope + rollback;
  return-annotation shapes (`$N` / dotted / indexed / spread).

This suite is what upgrades the claim from "trusted on the shipped grammars" to "trusted on **any**
grammar built from PGEN's constructs."

### 3.4 Honest bounds (state them, per [[feedback_always_signoff_decisions]])

- Differential testing proves equivalence on the **tested inputs**, not a formal proof for **all**
  inputs (that would need mechanized semantics). We do **not** over-claim a formal proof.
- Mitigations that make "100% trust" a signoff-grade claim rather than a hope: (a) the **shared-core**
  design shrinks the divergence surface to the dispatch layer; (b) the corpus is **combinator-complete**
  (§3.3) + real-world + generator-diverse; (c) an **optional fuzzing lane** (leaf `.7`: random gen-ASTs
  × random inputs, differential) pushes coverage toward exhaustive. Divergence-free over a
  combinator-complete, real-world, fuzzed corpus, with a shared core, is the **same** standard the rest
  of the platform is held to.

---

## 4. Phasing & dependency order

**Build the authoritative-by-construction paths FIRST** — they are the low-trust-burden way to unblock
`A2.3` immediately **and** they become the **oracle** that validates the interpreter.

- **Phase A — the authoritative harness / oracle** (approaches (3) then (2)):
  - `.2` (approach 3) scratch-register path — richest evidence (full CLI toolbox on a synthetic
    grammar), unblocks `A2.3` empirically.
  - `.3` (approach 2) compile-and-run harness — self-contained CI oracle for the equivalence gate.
- **Phase B — the general interpreter, VERIFIED** (approach (1)):
  - `.4` interpreter core (shared-core dynamic dispatcher over the gen-AST).
  - `.5` the differential-equivalence gate (all registered grammars, seeds 0/7/42).
  - `.6` the per-combinator differential suite.
  - `.7` (optional) the fuzzing lane.
- **Phase C — first real use + lockstep:**
  - `.8` run the `A2.3` empirical proof on the harness (hands back to `GRAMMAR-WELLFORMED.A2.3`).
  - `.9` lockstep: `TOOLBOX.md` (new tool entries + a probe protocol), the book (a parse-harness
    section), any contract, and this tree.

(Order rationale: (3) gives the richest A2.3 evidence and reuses the whole toolbox but touches the
registry; (2) is self-contained and CI-friendly; either alone suffices as the interpreter's oracle, so
Phase A can stop after whichever lands first if the schedule demands — but both are in scope.)

---

## 5. Acceptance criteria (tree-level)

1. **All three approaches delivered**, each drivable in ≤1 command (modulo a per-probe compile for (2)/(3)).
2. **(2) and (3) proven authoritative-by-construction** — each with an integration test reproducing a
   *registered* grammar's known verdict + AST through the harness.
3. **(1) the interpreter proven authoritative-by-verification** — **byte-identical** to the generated
   parser across the full differential corpus for **all** registered grammars **and** the per-combinator
   suite, **deterministic at seeds 0/7/42**, as a **re-runnable gate** (`parse_harness_equivalence_gate`).
4. **The capability's first real use lands:** `A2.3` (FixedTerminalPrefix soundness) is *answerable*
   with the harness (decisive `a | ab` evidence), handed back to `GRAMMAR-WELLFORMED`.
5. **No regression** to any shipped gate (cert-coverage seeds 0/7/42, the 6 fully-certified grammars
   byte-identical, `ast_shape_contract`, external corpora, clippy source-clean).
6. **Full lockstep, no drift (director-directive 2026-07-05, non-negotiable):** each landed
   component — the **grammar-AST interpreter (1)**, the **compile-and-run harness (2)**, and the
   **scratch-registration slot (3)** — gets its **own thorough section in the top-level mdBook**
   (`docs/book/`), documented as a structural component of the AST pipeline **to the same depth** the
   existing structural components are (the **grammar linter** → `grammar-wellformedness.md`, the
   **stimuli generator** → `stimuli-and-quality.md`, the **parser generator** → `developer-architecture.md`).
   The section lands **in the same commit** as the component (codebase↔book lockstep — the book is the
   user's only window; drift is a tracked correctness defect, [[feedback_regex_book_live]],
   [[feedback_ast_pipeline_components_documented_in_top_book]]). Plus `TOOLBOX.md` tool entries + any
   contract + this tree.
7. **Honest bounds documented** (§3.4) — no over-claim of a formal all-inputs proof.

Each **code** leaf (`.2`–`.8`) additionally carries the enforced **Acceptance Checklist** (ROOT CAUSE
+ ADDRESSED + NO REGRESSION, evidence-backed) at implementation time, per `TOOLBOX.md` /
`DOCTRINE_ENFORCEMENT.md`. `.1` (this design) is a non-code planning leaf.

---

## 6. Leaves

- `.1` — **DESIGN-LOCK — `done` (this file, `PGEN-PARSE-HARNESS-0001`, 2026-07-05).** The capability, the
  three approaches (§2), the trustworthiness architecture (§3), the phasing (§4), and the acceptance
  criteria (§5) are captured in gory detail. Decides: shared IR = the normalized gen-AST; shared
  semantic core = `semantic_runtime`; build order = authoritative-by-construction first (oracle), then
  the verified interpreter. Open sub-decision for `.2`/`.3`: which of (3)/(2) lands first (lean (3) for
  A2.3 evidence richness). NO code.
- `.2` — **(approach 3) scratch-register path — `not-started`.** A `grammars/scratch/*.ebnf` slot + auto
  registry wiring + `make focus_scratch` so a probe grammar is drivable by the full `parseability_probe`
  toolbox. Verify: authoritative-by-construction (integration test) + a `scratch` grammar parses a known
  input to a known AST. Unblocks A2.3.
- `.3` — **(approach 2) compile-and-run harness — `not-started`.** `compile_and_parse(...)` over the real
  codegen + a throwaway compile. Verify: reproduces a registered grammar's known verdict/AST; becomes the
  CI oracle for `.5`.
- `.4` — **(approach 1) interpreter core — `not-started`.** The shared-core dynamic dispatcher over the
  gen-AST (§2.1). Verify: parses the per-combinator smoke set; wired for `.5`.
- `.5` — **the differential-equivalence gate — `not-started`.** `parse_harness_equivalence_gate`:
  interpreter vs generated parser, byte-identical over the full corpus (§3.2), all registered grammars,
  seeds 0/7/42. This is the leg that makes (1) authoritative.
- `.6` — **the per-combinator differential suite — `not-started`.** Isolating grammars for every
  construct (§3.3) — the load-bearing coverage for "trust it on ANY grammar."
- `.7` — **the fuzzing lane (optional) — `not-started`.** Random gen-ASTs × random inputs, differential;
  pushes coverage toward exhaustive (§3.4).
- `.8` — **first real use: run the A2.3 proof on the harness — `not-started`.** Parse `a | ab` on `"ab"`
  (+ the always-succeeds cases) and read which alternative wins; hand the verdict back to
  `GRAMMAR-WELLFORMED.A2.3`.
- `.9` — **lockstep capstone — `not-started`.** `TOOLBOX.md` tool entries + probe protocol; the
  **top-level mdBook** sections for each landed component (interpreter / compile-and-run / scratch-slot),
  each documenting it as a structural AST-pipeline component to the depth of the existing linter /
  parser-generator / stimuli-generator chapters (director-directive 2026-07-05 — thorough, no drift,
  landed same-commit as the component); contract (if any); this tree. Note: per the same directive, the
  book coverage of the **existing** structural components was assessed 2026-07-05 and found present
  (linter/stimuli-gen/parser-gen have chapters) — this leaf ADDS the 3 new ones + keeps all in lockstep.

---

## 7. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `PARSE-HARNESS.2` (approach 3 — scratch-register path) | `not-started` (frontier) | Phase A, first authoritative-by-construction harness; richest A2.3 evidence. **Fresh-session recommended** for the build (director-agreed 2026-07-05 — a substantial, careful tooling build best done sharp). |
| 2 | `PARSE-HARNESS.3` (approach 2 — compile-and-run) | `not-started` | Phase A, the CI oracle for the equivalence gate. |
| 3 | `PARSE-HARNESS.4`–`.7` (interpreter + equivalence gate + combinator suite + fuzz) | `not-started` | Phase B — the verified general interpreter (the director's flagship). |

---

## 8. Decisions

- **D1 (2026-07-05).** Operate on the **normalized generation-input AST** (`--dump-gen-ast`), not the
  raw `.ebnf` — the same IR codegen consumes → maximal fidelity + free LR-elimination.
- **D2 (2026-07-05).** The interpreter (1) **reuses the shipped `semantic_runtime` + combinator
  primitives**; it is a thin *dynamic dispatcher*, not a re-implementation → the divergence surface is
  minimized before any testing.
- **D3 (2026-07-05).** Build **authoritative-by-construction ((2)/(3)) first**; they are the **oracle**
  for the interpreter. "100% trust" for (1) = shared core + differential-equivalence gate + a
  per-combinator suite (+ optional fuzz); (2)/(3) = authoritative by construction + a plumbing test.
- **D4 (2026-07-05).** Do **not** over-claim a formal all-inputs proof; the signoff-grade claim is
  divergence-free over a combinator-complete, real-world, fuzzed corpus with a shared core (§3.4).
- **D5 (director-directive 2026-07-05, non-negotiable).** Every structural component of the AST pipeline
  is **thoroughly documented in the top-level mdBook**, and each of the three harness components lands
  its top-level-book section **in the same commit** as the component (codebase↔book lockstep, no drift,
  no exception). See [[feedback_ast_pipeline_components_documented_in_top_book]].

## 9. Blockers

None. The tree is unblocked. (Implementation of `.2`+ is deferred to a **fresh session** by
director agreement 2026-07-05 — a sharpness/quality call, not a blocker.)

## 10. Relationships

- **Unblocks `GRAMMAR-WELLFORMED.A2.3`** (FixedTerminalPrefix soundness) — the capability's first real
  use; and any future linter-soundness / grammar-authoring probe.
- **Reinforces** [[feedback_certifying_linter_trustworthiness]] (verify, don't trust — the differential
  oracle), [[feedback_ast_pipeline_parser_agnostic]] / [[feedback_features_parser_agnostic_enable_all_parsers]]
  (parser-agnostic, all grammars), and the `TOOLBOX.md` build-a-tool directive.
- **Composes with** the certificate-coverage machinery (`parse_and_cover`), the stimuli generator (the
  differential corpus source), and the EBNF dual-run gate (the precedent for a two-implementation
  equivalence gate).
