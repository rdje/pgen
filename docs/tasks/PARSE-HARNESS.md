# PARSE-HARNESS — general arbitrary-grammar parse capability (the grammar-AST interpreter + compile-and-run + scratch-register), each made 100% trustworthy

- Tree ID: `PARSE-HARNESS`
- Status: `complete` (re-closed 2026-07-18, session #149 — ops-hardening leaf `.10` **`done`** (`PGEN-PARSE-HARNESS-0018`): the feature-surface tripwire for the compile-and-run oracle's shelled-out `ast_pipeline`, the #140-class trap ×3, opened + landed under the director's `PGEN-RGX-0078-0125` GO, do-first before `RGX-0078.5.j.2` STEP-2 which leans on this battery; §22. Originally `complete` 2026-07-07, session #51 — `.9` capstone; `.7` fuzz parked-optional per D6.) Created 2026-07-05, session #37, `PGEN-PARSE-HARNESS-0001`. `.1` DESIGN is this file; `.2` (scratch-register) **`done`** (`PGEN-PARSE-HARNESS-0002`, session #38); `.3` (compile-and-run) **`done`** (`PGEN-PARSE-HARNESS-0003`, session #39). **Phase A complete.** `.4` (interpreter core — the director's flagship) **`done`** (`PGEN-PARSE-HARNESS-0004`, session #40): byte-identical to the generated parser on the structural + return-annotation smoke set. **Phase B open.** `.5` (the differential-equivalence GATE — `parse_harness_equivalence_gate`) **`done`** (`PGEN-PARSE-HARNESS-0005`, session #41): the deterministic interpreter-vs-generated-parser differential over a bounded stimuli corpus (seeds 0/7/42, large-stack workers), **certifying 6 grammars byte-identical** — `json`, `semantic_annotation`, `rtl_frontend`, `vhdl`, **`systemverilog` (sv_2017)**, `scratch` — with an honest DEFERRED ratchet + EXCLUDED classification (no silent caps) for the remainder. The measurement discovered the honest per-grammar split (tool-backed, §14/§15). `.5.1` (regex fidelity) **`done`** (`PGEN-PARSE-HARNESS-0007`, session #42): FOUR tool-pinpointed interpreter-fidelity fixes (whitespace-sensitive layout policy / unresolved-reference built-ins / `@transform` numeric coercion + PCRE2 post-parse contract / `@profiles` dialect gating) certified **`regex`** byte-identical (deep stress 400/400) AND incidentally closed **`.5.4`** (`systemverilog_preprocessor`, 459/459) — 8 grammars CERTIFIED. `.5.2` (ebnf fidelity) **`done`** (`PGEN-PARSE-HARNESS-0009`, session #43): the interpreter's two layout skippers unconditionally skipped all three comment introducers (`#`/`//`/`/*`), but codegen SUPPRESSES a comment arm per-grammar when the grammar claims that introducer as a real token (H.11.5 — ebnf's `block_comment := "/*" …`); the interpreter now gates each arm via codegen's OWN predicate (`comment_arm_suppression_for_grammar`), certifying **`ebnf`** byte-identical — 9 grammars CERTIFIED. `.5.3` (return_annotation fold) **`done`** (`PGEN-PARSE-HARNESS-0011`, session #44): the `_pgen_lr_chain` `wrapper_specs` blob was serialized from a non-deterministic std `HashMap` (`UnifiedReturnAST::Object`) — codegen froze one arbitrary order, the interpreter re-serialized a fresh non-deterministic order each load; a `serialize_with` SORTED serializer canonicalizes every site, certifying **`return_annotation`** byte-identical — now **10 grammars CERTIFIED**. `.5.5` (rtl_const_expr corpus) **`done`** (`PGEN-PARSE-HARNESS-0012`, session #45): NOT a fidelity fix but a CORPUS fix — the stimuli generator yields 0 usable samples for rtl_const_expr's ~16-level precedence cascade within the bounded ladder (depths ≤28 fail, ~32 pathologically huge, ≥40 hang — tool-established); a general parser-agnostic **curated-input corpus** (`CURATED_CORPUS`) gives the differential inputs and the interpreter is byte-identical over it (CLEAN 151/151), certifying **`rtl_const_expr`** — now **11 grammars CERTIFIED, DEFERRED empty**. `.6.1` (structural combinator isolating suite) **`done`** (`PGEN-PARSE-HARNESS-0014`, session #46): a systematic, gated suite of **16 isolating grammars** (module `rust/src/parse_harness_combinator_suite.rs`, gate `make -C rust parse_harness_combinator_gate`) proving the interpreter byte-identical to the `.3` compile-and-run oracle per structural combinator (choice under all 3 branch_policies, quantifiers `?`/`*`/`+` incl. zero-length guard, lookahead, sequence-backtrack, atoms/regex-token, rule-ref, LR-eliminated) — **16/16 CLEAN**, gate 2/2, incl. the folded-in A2.2/A2.3 discrimination proof; three tool-established findings surfaced (§20). `.6.2` (semantic-directive orchestration) **`done`** (`PGEN-PARSE-HARNESS-0015`, session #47): the interpreter gains the FULL store-gated orchestration mirror + split packrat memo (mirrored from the emitted codegen templates; the queries/store/transactions reused verbatim from the shared runtime), certified byte-identical to the `.3` oracle per construct by a **20-case isolating suite** (module `rust/src/parse_harness_semantic_suite.rs`, gate `make -C rust parse_harness_semantic_gate`) — **20/20 CLEAN** (pre-mirror baseline 1/18), `.5` gate 4/4 with all 11 CERTIFIED grammars byte-identical under the now-directive-aware interpreter (~21 s — the memo closed the no-memo slowness), **the `.6` per-construct coverage is COMPLETE** and the §3.4/§13.4 honest bound is CLOSED; six tool-established findings surfaced (§21). `.8` (the A2.3 proof — the capability's FIRST REAL USE) **`done`** (`PGEN-PARSE-HARNESS-0016`, session #51): live scratch-slot probe + fresh `.6.1` gate — default `longest_match` SELECTS the "shadowed" later alt on `a|ab` (`🏁 selected branch 2/2`) while `--lint-grammar` hard-fails the same grammar rc=1; verdict handed to `GRAMMAR-WELLFORMED.A2.3`, which FIXED the `FixedTerminalPrefix` verdict + certificate to be branch-policy-aware the same session (`PGEN-GRAMMAR-WELLFORMED-0151`). `.9` (lockstep capstone) **`done`** (`PGEN-PARSE-HARNESS-0017`, session #51, docs-only): D5 audit held (TOOLBOX 1.3–1.8 + book chapter per component, same-commit throughout); added TOOLBOX **Protocol D** ("which alternative WINS / is this branch live?") + quick-chooser row + the book chapter's canonical-probe section. `.7` (fuzz) **parked-optional** (D6). **TREE COMPLETE** — all 7 tree-level acceptance criteria (§5) met. See §13 (`.4` plan), §14 (`.5` plan), §15 (measurement map), §16 (`.5.1`), §17 (`.5.2`), §18 (`.5.3`), §19 (`.5.5`), §20 (`.6.1` checklist), §21 (`.6.2` plan + checklist).
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
- `.2` — **(approach 3) scratch-register path — `in-progress` (session #38, `PGEN-PARSE-HARNESS-0002`).** A
  `grammars/scratch/` slot (blessed default `scratch.ebnf`) + auto registry wiring + `make focus_scratch`
  so a probe grammar is drivable by the full `parseability_probe` toolbox. Verify: authoritative-by-construction
  (integration test) + a `scratch` grammar parses a known input to a known AST. Unblocks A2.3.
  **Implementation shape (tool-verified, §11 below):** the generated struct/method names are derived from
  the grammar-name (the `.ebnf` file stem) and the entry rule respectively (`ast_generator_direct.rs:109`
  `snake_to_pascal(grammar_name)` → `ScratchParser`; `ast_based_generator.rs:352`), and the canonical
  `parse_full()` method is **entry-rule-agnostic** (`generated/json_parser.rs:283` — `parse_full_json` just
  delegates to it), so a single blessed slot `grammars/scratch/scratch.ebnf` (stem `scratch`) yields a
  STABLE registry entry (`grammar_name: "scratch"`, `ScratchParser::parse_full()`) independent of whatever
  entry-rule name the probe grammar uses. Wiring mirrors `json` exactly: Makefile `SCRATCH_*` vars +
  `focus_scratch`, `build.rs` `has_generated_scratch_parser` cfg + `PGEN_SCRATCH_PARSER_PATH*`, `lib.rs`
  `generated_parsers::scratch`, and the `parser_registry.rs` dispatch (`parse_with_scratch` via `parse_full()`,
  `parse_and_cover_scratch`, detail with `furthest_position`, AST-JSON, entry-aware variants). `generated/` is
  git-ignored so scratch artifacts never pollute the tracked set; `grammars/scratch/scratch.ebnf` IS tracked
  (the blessed slot + the integration-test fixture).
- `.3` — **(approach 2) compile-and-run harness — `done` (session #39, `PGEN-PARSE-HARNESS-0003`).**
  `compile_and_parse(grammar_ebnf, input, opts) -> ParseOutcome` over the **real codegen** + a **throwaway
  external-crate compile**. Verified: reproduces the `json` registry verdict + **byte-identical typed AST** (integration
  test **1 passed**); becomes the CI oracle for `.5`.
  **Implementation shape (spike-verified this session — decisive tool evidence, §12 below):** the make-or-break
  question was *"can a generated parser compile in an EXTERNAL crate (not inside `pgen`)?"* — because the generated
  source hard-codes `use crate::ast_pipeline::…` and inside `pgen` that resolves `pub(crate)` items an external crate
  can't see. Tool-verified answer: **YES.** A static audit shows every generated parser reaches **only**
  `crate::ast_pipeline::*` (22 distinct symbols, ALL `pub`) plus externs `regex`/`rustc_hash`/`serde_json`; a throwaway
  crate that path-deps `pgen`, adds the single shim `use pgen::ast_pipeline;` (so `crate::ast_pipeline` resolves), and
  `include!`s the generated file **compiles and runs** — reproducing the registry verdicts byte-for-byte on TWO grammars
  with distinct machinery (`scratch`: `"hello, world!"`→ACCEPT, `"hello, mars!"`→REJECT `furthest=7`; `json`:
  `{"a":1,"b":[true,null]}`→ACCEPT, `{"a": }`→REJECT `furthest=6`). `ParseNode` derives `serde::Serialize`, so the
  throwaway emits the **byte-identical typed AST** the registry's `parse_node_to_json` (`serde_json::to_value(node)`) does.
  So the harness = pure plumbing: (1) shell the pre-built `ast_pipeline` binary exactly as `RUST_GENERATOR`
  (`--emit-raw-ast-json` then `--generate-parser --debug --trace --eliminate-left-recursion`), (2) discover the struct
  name by grepping the emitted `pub struct <Name>Parser<'input>` (exactly one per file — verified), (3) synthesize a
  throwaway cargo crate (path-dep `pgen`, isolated `CARGO_TARGET_DIR` → no lock contention with an outer `cargo test`),
  (4) `cargo run` it on the input file (+ optional `--entry-rule` via `parse_full_from`), (5) parse its sentinel-wrapped
  JSON `{accepted, furthest_position, error, ast}` back into `ParseOutcome`. Uses `parse_full()`/`parse_full_from(entry)`
  → entry-rule-agnostic like the scratch slot. Honest bound: `scratch`+`json` prove the mechanism + full return-annotation/
  regex-token/memo surface; SV's 69MB method surface is unverified here — any `pub(crate)` method it needs made `pub` is a
  bounded follow-up surfaced by `.5`, not a `.3` blocker.
- `.4` — **(approach 1) interpreter core — `done` (session #40, `PGEN-PARSE-HARNESS-0004`).** The
  shared-core dynamic dispatcher over the gen-AST (§2.1). VERIFIED byte-identical to the generated parser on
  the per-combinator smoke set: the `json` registry oracle (7 accepts + 4 rejects, byte-identical verdict + typed
  AST) + 5 synthetic grammars via the `.3` compile-and-run oracle (A2.3 fixed-prefix, `*`/`+`, optional `?`,
  `&`/`!` lookahead — byte-identical verdict + `furthest_position` + typed AST); `parse_harness_interpreter`
  **7 tests pass**. Wired for `.5` (returns the same `ParseOutcome` as `compile_and_parse`; the core entry
  `interpret_parse_gen_ast` is the gate's driver). **Tool-mapped plan, committed scope, shared-core boundary,
  and the enforced acceptance checklist are in §13.** Key decisions this leaf makes (all tool-backed, §13): the interpreter reuses the shipped semantic /
  AST / recursion / quantifier-bounds primitives **verbatim** (Section A) and re-expresses only the combinator
  dispatch + the lexical/speculation primitives + the return-fold (Sections B/C), which exist **only** as
  codegen `quote!` templates — so the divergence surface is the combinator half, exactly as D2 predicted; a
  string **interner** leaks the finite set of rule-names / quantifier-labels / annotation-literals to
  `&'static str` so the exact shipped `ParseNode` type is reused (byte-identical serde); **no memo** in `.4`
  (a transparent cache — AST-invariant — deferred to `.5` where the corpus needs it); the **semantic-directive
  orchestration** (store-gated parsing) is threaded (SemanticRuntimeState present + speculation-faithful) but
  its full orchestration is the `.6` extension (honest bound, §3.4 / §13).
- `.5` — **the differential-equivalence gate — `done` (session #41, `PGEN-PARSE-HARNESS-0005`).**
  `parse_harness_equivalence_gate` (module `rust/src/parse_harness_equivalence.rs`): the deterministic
  interpreter-vs-generated-parser differential over a bounded stimuli corpus (seeds 0/7/42, depth ladder,
  large-stack workers). Delivers the gate INFRASTRUCTURE + the CERTIFIED byte-identical baseline (6
  grammars) + an honest DEFERRED ratchet + EXCLUDED classification. The original "byte-identical over
  ALL registered grammars" target is **re-scoped** into the certified baseline (now) + `.5.1`–`.5.5`
  (the per-grammar closures) — an honest task-tree refinement (the measurement §15 discovered exactly
  which grammars need more work and why). Acceptance checklist + tool-mapped plan in §14.
- `.5.1` — **regex fidelity — `done` (session #42, `PGEN-PARSE-HARNESS-0007`).** Root-caused + fixed
  the interpreter's regex divergence (FOUR distinct interpreter-fidelity gaps, each tool-pinpointed), then
  PROMOTED `regex` DEFERRED→CERTIFIED. The general fixes ALSO closed `.5.4` (`systemverilog_preprocessor`
  — promoted the same commit; see below). All fixes are **interpreter-tooling only** — no engine / grammar
  / codegen / generated-parser change, so the certified grammars stay byte-identical by construction.
  **The four root causes (WHY + WHERE, tool-backed) + fixes:**
  1. **Whitespace-sensitivity (layout policy).** `regex` is whitespace-SENSITIVE — codegen keys this on the
     grammar NAME (`allow_layout_skip_for_terminals = normalized != "regex"` @`ast_based_generator.rs:4509`;
     `allow_layout_skip_for_regexes` @`:4510`; `allow_trailing_layout` @`:1144`), emitting `if false {
     consume_layout… }` in the generated `match_string`/`match_regex`/`parse_full`. The interpreter
     unconditionally skipped layout, so on `\Q]\E* ?` it skipped the literal space and bound the `?` as a
     lazy `quant_suffix` (`greediness:"lazy"`) where the generated parser leaves it empty (`greediness:[]`).
     (The prior "REFUTED" note missed that regex's `match_string` never *calls* `consume_layout_for_terminal`.)
     **Fix:** a per-grammar `LayoutPolicy` mirroring the three codegen decisions expression-for-expression,
     gated at the same three call sites.
  2. **Unresolved-reference built-ins.** `unicode_char = !builtin_ascii_char builtin_any_char` matches every
     non-ASCII char via codegen-native matchers (`generate_unresolved_reference_method`
     @`ast_based_generator.rs:833-946`) for rules referenced-but-undefined. The interpreter hard-errored on
     any rule not in the grammar tree → rejected every `é`-bearing input at `furthest≈3`. **Fix:**
     `parse_unresolved_reference` mirrors codegen's `true`/`false`/`semantic_annotation`/`builtin_any_char`/
     `builtin_ascii_char` matchers + the Backtrack stub, byte-identical.
  3. **`@transform` numeric span coercion + the PCRE2 post-parse contract.** (a) `digits = digit+` carries
     `@transform: str::parse::<usize>()` (`generate_post_body_span_transform` @`:4094`) → a
     `TransformedTerminal` that `to_json_value()` renders as a NUMBER (`{min:12}`); the interpreter produced
     a digit-char array (`["1","2"]`). **Fix:** `apply_post_body_span_transform` mirrors it for the canonical
     integer/float/bool target types. (b) `parse_sample` for regex applies `validate_regex_compile_contract`
     (PCRE2-fidelity — rejects e.g. `$+`, a quantifier on an anchor, which the grammar accepts but PCRE2
     rejects; `parser_registry.rs:357`) ON TOP of the grammar parse. **Fix:** a general
     `parser_registry::post_parse_semantic_contract(name, sample)` the gate applies to the interpreter's
     verdict, so both sides compare at the same "grammar-parse + registry contract" layer (the interpreter
     core stays a pure grammar-parse reproduction; parser-agnostic).
  4. **`@profiles` dialect gating.** `directive_name_relaxed = … @profiles:["relaxed"]` is excluded under the
     default strict `pcre2` profile (codegen rule-entry `profile_guard` @`:2692-2704` + `rule_profile_is_enabled`
     @`:4903`), so the generated parser rejects `(*H_2Y-:)`; the interpreter ignored `@profiles` and accepted
     it. **Fix:** thread the ALREADY-NORMALIZED active profile (`parser_registry::active_grammar_profile`,
     new pub) into the interpreter; a `@profiles` rule Backtracks at entry when the active profile is not
     allowed — mirrors `rule_profiles` (@`:7108`) + `rule_profile_is_enabled`. General (also correctly gates
     SV `sv_2017` vs `sv_2023` — SV stays byte-identical).
  **Verification (tools-first, re-runnable):** `probe_regex_divergence_minimizer` agrees on all fixed
  constructs; `probe_regex_deep_stress` (5 seeds, depths 6-30) — regex **400/400 CLEAN**, svpp **459/459
  CLEAN**; `parse_harness_equivalence_gate` **3/3** (regex+svpp CERTIFIED byte-identical; json/semantic_annotation/
  rtl_frontend/vhdl/systemverilog/scratch stay byte-identical; ratchet holds for ebnf/return_annotation/
  rtl_const_expr; completeness). Interpreter unit tests **7/7**. See §16 for the enforced acceptance checklist.
- `.5.2` — **ebnf fidelity — `done` (session #43, `PGEN-PARSE-HARNESS-0009`).** Root-caused + fixed the
  interpreter's ebnf VERDICT divergence, then PROMOTED `ebnf` DEFERRED→CERTIFIED. **Root cause (tool-backed,
  §17):** the interpreter's two layout skippers (`consume_layout_for_terminal` / `consume_layout_for_regex`)
  UNCONDITIONALLY skip all three comment introducers (`#`/`//`/`/*`), but codegen SUPPRESSES a comment arm
  per-grammar (GRAMMAR-WELLFORMED.H.11.5) when the grammar claims that introducer as a non-comment token —
  ebnf's `block_comment := "/*" …` makes `"/*"` a real token, so the generated ebnf parser has NO `/*`
  layout arm and rejects the comment-only `/**/`, while the interpreter skipped it as layout and accepted.
  **Fix (single source of truth):** the interpreter now computes the per-introducer suppression via codegen's
  OWN predicate `grammar_claims_introducer_as_non_comment` (exposed as `comment_arm_suppression_for_grammar`)
  and gates each layout arm on it — so it matches codegen for EVERY grammar (the gating can only reduce
  divergence). General ⇒ also removed the latent over-skip for regex/vhdl/sv/rtl_frontend (`#` arm) and
  semantic_annotation (`//`+`/*` arms). All interpreter-tooling; codegen emit path untouched. VERIFIED:
  `PGEN_PHEQ_ONLY=ebnf` DIVERGE 6→CLEAN 83/83; deep-stress ebnf CLEAN 242/242 (5 seeds × gate depths);
  `parse_harness_equivalence_gate` 4/4 (ebnf CERTIFIED + the new matrix-pinning test). Original scouting log
  retained below for provenance.
  **Scouting log (session #42, tool-backed — `PGEN_PHEQ_ONLY=ebnf …::measurement`; applies the `.5.1` lesson
  = enumerate the FULL divergence set BEFORE theorizing):**
  **Scouting log (session #42, tool-backed — `PGEN_PHEQ_ONLY=ebnf …::measurement`; applies the `.5.1` lesson
  = enumerate the FULL divergence set BEFORE theorizing):**
  - **The FULL divergence set (ladder [6,12,18], 83 samples / 77 agree / 6 diverge) — ALL are the SAME class:
    the interpreter ACCEPTS a COMMENT/LAYOUT-ONLY input the generated parser REJECTS.** Diverging samples:
    `"/**/"` (interp furthest=3), `"/*"` (furthest=2), `"       /**//***/"` (furthest=10) + 3 more of the same
    shape. Verdict-only (`interp.accepted=true (grammar-parse=true)` vs `oracle.accepted=false`); no AST case.
  - **WHERE (tool-established):** entry rule `grammar_file := (include_directive | semantic_annotation |
    grammar_rule | comment | whitespace)* -> …` (`grammars/ebnf.ebnf:25`) — a `*` over alternatives that
    INCLUDE `comment` + `whitespace`. The oracle is the RAW generated parser `EbnfParser::parse_full_grammar_file()`
    (`parser_registry.rs:266-268`) — **no adapter, no post-parse contract** (the only contract is regex's,
    `parser_registry.rs:1185`). So the divergence is a genuine grammar-parse-of-`grammar_file` difference, NOT
    a registry-layer artifact. Interpreter acceptance requires `position == input.len()` after the entry-rule
    parse + a trailing layout consume (`parse_harness_interpreter.rs:242-267`); `furthest_position` is bumped
    ONLY at rule-entry (`:457`), so a small furthest (2/3/10) with a full-input accept is self-consistent (the
    final comment is eaten by a layout-consume that does not bump furthest).
  - **CONCRETE located bug (tool-backed, the `/*` sub-case):** the interpreter's HAND-ROLLED block-comment
    layout skipper treats an UNTERMINATED `/*` as a COMPLETE comment. In `consume_layout_for_terminal`
    (`parse_harness_interpreter.rs:1082-1095`) — and identically in `consume_layout_for_regex` (`:1145-1161`)
    — the `/*` arm does `position += 2` then `while position+1 < len && !(bytes==*/) { position += 1 }` then
    an UNCONDITIONAL `if position+1 < len { position += 2 }`, so on `/*` (len 2) it advances to EOF and
    `continue`s → the comment is "consumed" with NO required `*/`. The generated `block_comment := "/*"
    block_comment_content "*/"` REQUIRES the close, so it rejects `/*`. This skipper is too lenient.
  - **The `/**/` / `/**//***/` sub-case (NOT yet root-caused — next session's first job, DO NOT assume):**
    `/**/` is a well-terminated empty comment, so the lenient-skipper bug above does NOT explain it. Both naive
    theories (comment-consumed-as-leading-layout-then-`*`-matches-zero; comment-matched-by-the-`comment`-ALT)
    predict ACCEPT, yet the generated parser REJECTS. The likely real mechanism (VERIFY with a tool first —
    read the EMITTED generated `parse_grammar_file` `*`-loop + its layout handling, or trace it): codegen
    treats `comment`/`whitespace` as auto-skipped LAYOUT (the layout skipper carries comment arms —
    `ast_based_generator.rs:4525/4596/9699`), so the `comment`/`whitespace` ALTERNATIVES in `grammar_file` are
    effectively DEAD, and the generated `*`-loop likely skips layout, tries an element, FAILS at EOF, and
    ROLLS BACK the layout consumption → grammar_file matches zero at position 0 → EOF check (`0 != len`) →
    REJECT. The interpreter instead keeps the layout-consume (or matches `comment` as an element) → reaches
    EOF → ACCEPT. **Next tool step:** diff the emitted generated `grammar_file` quantifier-loop-with-layout
    against the interpreter's `*`-loop + `consume_layout_for_terminal`; the fix is to make the interpreter's
    comment/layout handling mirror codegen's (rollback semantics + required-close), a GENERAL layout-fidelity
    fix (parser-agnostic), likely also tightening the two skippers above. Then re-run `PGEN_PHEQ_ONLY=ebnf`
    + a generalized deep-stress, promote `ebnf`, full no-regression + lockstep + `.5.2` acceptance checklist.
- `.5.3` — **return_annotation fidelity — `done` (session #44, `PGEN-PARSE-HARNESS-0011`).** Root-caused +
  fixed the AST divergence, then PROMOTED `return_annotation` DEFERRED→CERTIFIED (**10th** grammar). **The
  scouting log's hypothesis was REFUTED by the tools** — see §18 for the enforced acceptance checklist and the
  real root cause. In brief (tool-backed): the divergence is the `_pgen_lr_chain` `wrapper_specs` blob (the
  serialized per-alt `annotation_template`s of the LR-eliminated `property_access_expression` / array-access
  rules), which is `serde_json`-serialized from a std `HashMap` (`UnifiedReturnAST::Object.properties`, whose
  per-instance iteration order is NON-deterministic). There is no stable "codegen order" for the interpreter to
  mirror — codegen merely FROZE one arbitrary HashMap order into the generated parser, while the interpreter
  re-serialized a fresh (itself non-deterministic — proven: `$1.S15`→`{property,base,…}`, `$1[$1*]**`→`{type,…}`,
  `$1.S15K.vbn**`→`{property,type,…}` in ONE run) order each load. **Fix (single source of truth):** a
  `serialize_with` sorted serializer on `UnifiedReturnAST::Object.properties` (`rust/src/ast_pipeline/unified_return_ast.rs`)
  canonicalizes EVERY serialization site at once (gen-AST, the codegen-frozen literal, the interpreter) → sorted
  keys everywhere → byte-identical. General/parser-agnostic; it also closed a LATENT codegen non-determinism (a
  fresh `--generate-parser` could previously freeze a different — behaviorally-equivalent — `wrapper_specs` order;
  now byte-identical across regens, tool-verified). Blast radius: only `return_annotation` + `semantic_annotation`
  carry Object-template `wrapper_specs` (both regenerated; no shape-contract pins the blob). VERIFIED:
  `PGEN_PHEQ_ONLY=return_annotation` DIVERGE 8→CLEAN 68/68; `parse_harness_equivalence_gate` 4/4 (return_annotation
  CERTIFIED; semantic_annotation still byte-identical after regen; ratchet now only rtl_const_expr); interp unit
  tests 7/7; `unified_return_ast` 27/27. **CERTIFIED now 10:** json/semantic_annotation/rtl_frontend/vhdl/
  **systemverilog (sv_2017)**/scratch/regex/systemverilog_preprocessor/ebnf/**return_annotation**; DEFERRED →
  rtl_const_expr; EXCLUDED → builtin_*. Original scouting log retained below for provenance.
  **Scouting log (session #43, tool-backed — `PGEN_PHEQ_ONLY=return_annotation …::measurement`; the KEY-ORDER
  hypothesis below was the right SYMPTOM but the wrong CAUSE — the order is not a mirrorable codegen order but a
  non-deterministic HashMap artifact, per §18):**
  - **The divergence set: `DIVERGE samples=68 agree=50 diverge=8 (+10 suppressed)` — ALL are `[Ast]` (NOT
    verdict): both interp + oracle ACCEPT and produce the SAME-LENGTH AST (e.g. 828/828), but the byte content
    differs.** Diverging samples: `$1.S15`, `$1.S15K.vbn**`, `$1[$1*]**`, … — every one involves a **dotted
    property access** (`$1.S15`) or **array access** (`$1[$1*]`), i.e. a rule that was **left-recursion-eliminated**.
  - **WHERE / likely cause (tool-established, VERIFY before fixing):** the divergence is an **Object-property
    KEY-ORDER** difference in the folded return-annotation AST for LR-eliminated rules. `property_access_expression
    := accessor_base '.' identifier -> {type:"property_access", base:$1, property:$3}` (`grammars/return_annotation.ebnf:99-100`)
    is LR-eliminated into a `_pgen_lr_chain` carrying a `wrapper_specs` string. The ORACLE dump of `$1.S15`
    (`parseability_probe --parse-dump-ast-pretty return_annotation`) shows the wrapper_specs `annotation_template`
    serializes the Object properties in order **`{property, type, base}`** — NOT grammar source order `{type, base,
    property}`. The measurement diff shows the interpreter emits a DIFFERENT order at the same offset (interp
    `…PositionalRef:{index:1}},"type":{StringLiteral…` vs oracle `…PositionalRef:{index:1}},"property":{PositionalRef…`),
    i.e. interp `{base, type, …}` vs oracle `{property, type, base}`. So the interpreter's return-annotation Object
    fold (or its `_pgen_lr_chain` wrapper handling) preserves a different property ORDER than codegen froze into
    `wrapper_specs`. **Next tool step:** dump the interpreter's full AST for `$1.S15` (extend the measurement's
    `probe_regex_divergence_minimizer` pattern, or add a scouting probe), diff it against `/tmp` oracle dump; find
    where codegen fixes the wrapper_specs Object key order (likely `ast_based_generator.rs` LR-elimination /
    annotation-template serialization — an `IndexMap`/ordered map vs the interpreter's `HashMap` iteration) and
    make the interpreter's fold reproduce that EXACT order (parser-agnostic; the fix is in `parse_harness_interpreter.rs`'s
    return-fold, mirroring codegen). Then re-run `PGEN_PHEQ_ONLY=return_annotation`, promote, full no-regression +
    lockstep + a `.5.3` acceptance checklist.
  - **Note on scope:** this is the FOLD (return-annotation Object serialization for LR-eliminated rules), NOT
    layout (`.5.2`) or the semantic store (`.6`). It is likely a general ordering-fidelity fix that also hardens
    any future LR-eliminated grammar with Object return annotations.
- `.5.4` — **systemverilog_preprocessor fidelity — `done` (CLOSED by `.5.1`, session #42,
  `PGEN-PARSE-HARNESS-0007`).** The `.5.1` general fixes (the `systemverilog_preprocessor` regex-token
  whitespace-sensitivity via the shared `LayoutPolicy` — `allow_layout_skip_for_regexes` is `false` for both
  `regex` and `systemverilogpreprocessor` — plus the unresolved-reference built-ins) incidentally closed the
  `.5.4` AST span/shape divergence. The `.5` ratchet DETECTED svpp had become byte-identical and DEMANDED its
  promotion (the no-silent-progress discipline working as designed); PROMOTED to CERTIFIED after
  `probe_regex_deep_stress` confirmed **459/459 CLEAN** (5 seeds, depths 6-30).
- `.5.5` — **rtl_const_expr corpus — `done` (session #45, `PGEN-PARSE-HARNESS-0012`).** PROMOTED
  `rtl_const_expr` DEFERRED→CERTIFIED (**11th** grammar) via a general **curated-input-corpus** mechanism —
  NOT a fidelity fix but a corpus fix (the leaf's thesis, now VERIFIED: byte-identical once a corpus
  exists). **Tool-established (§19):** the CLI generation sweep confirmed the razor-thin window — depths
  6-28 FAIL (`depth exceeded max_depth=N while expanding 'multiplicative_expr'`), depth 32 generates but
  only pathologically huge (thousands-of-chars) expressions, depths ≥40 HANG — so tuned generation is not
  a robust corpus source. Fix = a parser-agnostic `CURATED_CORPUS` table + `curated_corpus_for` +
  `build_corpus` integration (deduped, with truncation reject-path probes), seeded with a
  construct-complete `rtl_const_expr` curated list. VERIFIED: `PGEN_PHEQ_ONLY=rtl_const_expr` DIVERGE
  samples=0 → **CLEAN 151/151**; `parse_harness_equivalence_gate` 4/4; no `generated/*` regenerated. See
  §19 for the enforced acceptance checklist.
- `.6` — **the per-combinator + semantic-directive differential suite — `not-started`.** Isolating
  grammars for every construct (§3.3) — the load-bearing coverage for "trust it on ANY grammar." The
  end-to-end `.5` gate only covers the constructs the *shipped* grammars use; `.6` proves equivalence
  **per combinator** on small synthetic grammars (interpreter vs the `.3` compile-and-run oracle / a
  scratch parser), so the interpreter is trusted on ANY grammar built from PGEN's constructs. Decomposed
  into two sub-leaves by build-risk (session #45 scoping):
  - `.6.1` — **structural combinator isolating suite — `done` (session #46, `PGEN-PARSE-HARNESS-0014`).**
    A systematic, gated suite (module `rust/src/parse_harness_combinator_suite.rs`, gate
    `make -C rust parse_harness_combinator_gate`) of **16 small isolating grammars**, one per structural
    combinator the `.4` interpreter dispatches, each differentially verified **byte-identical**
    (verdict + `furthest_position` + typed AST) against the `.3` compile-and-run oracle over curated
    inputs: ordered choice under EACH `branch_policy` (`longest_match` default+explicit / `ordered` /
    `priority_first`), the `a | ab` fixed-prefix + `e? | keyword` always-succeeds shapes (A2.2/A2.3
    directly — front-loads `.8`); sequence + backtrack; `?`/`*`/`+` incl. the zero-length guard; lookahead
    `&`/`!`; atom terminal + regex-token; rule-ref; left-recursion (the **LR-eliminated wrapper form**).
    Report-first (never-panic) + 2 enforcing gate tests (byte-identity incl. the folded-in A2.2/A2.3
    discrimination proof; combinator-coverage completeness) + 2 `--ignored` scouting probes, all on a
    512 MiB large-stack worker. **VERIFIED (tools-first): 16/16 CLEAN**, gate 2/2. Test-only tooling —
    NO engine/grammar/codegen/generated change (no `generated/*` regenerated). Three tool-established
    findings surfaced (see §20 + `DEVELOPMENT_NOTES.md`): (1) bounded quantifiers `{N,M}` are half-wired
    (frontend + runtime `parse_quantifier_bounds` support them; **codegen** aborts `Unknown quantifier` →
    unreachable via the oracle → `?`/`*`/`+` are the covered forms); (2) LR-elimination **prepends**
    `_lr_base`/`_lr_suffix` to `rule_order`, shifting `rule_order[0]` off the semantic entry (the case
    names the entry explicitly, applied to both sides); (3) bare **direct** LR `A := A x | y` is NOT
    structurally eliminated (only the wrapper form is) → runtime cycle-breaking, where interp & oracle
    diverge on `furthest_position` (interp 2/4, oracle 0; verdicts agree) — a KNOWN out-of-scope
    interpreter-fidelity gap kept as a durable re-runnable probe. Acceptance checklist in §20.
  - `.6.2` — **semantic-directive orchestration suite — `done` (session #47, `PGEN-PARSE-HARNESS-0015`).**
    The surface `.4`/`.5` explicitly **DEFERRED** (§13.4) — store-gated *parse outcomes* — is now BOTH
    implemented in the interpreter AND certified per-construct: the full orchestration mirror
    (`with_rule_transaction` skeleton, tournament C3-B + branch gates + branch-start actions, the
    `$reference` resolver family, library phases, and the **split packrat memo** with semantic-delta
    replay) mirrored from the EMITTED codegen templates, plus the 20-case isolating suite (module
    `rust/src/parse_harness_semantic_suite.rs`, gate `make -C rust parse_harness_semantic_gate`) proving
    the interpreter **byte-identical to the `.3` compile-and-run oracle per construct — 20/20 CLEAN**
    (pre-mirror honest baseline: 1/18). Closes the §3.4/§13.4 honest bound — **the `.6` per-construct
    coverage is COMPLETE**. Six tool-established findings surfaced (§21.2/§21.4 + CHANGES.md): the
    store-blind memo failure cache (stale-failure replay, pinned BOTH sides), quoted-arg
    String-vs-Identifier fact-name mismatch, inline branch-predicate rule-wide flattening, the `$`-strip
    positional-ref dead path, zero-length-success emission persistence, and the no-preamble
    unresolved-reference stubs (a real interpreter-fidelity fix). **Tool-mapped plan, committed scope,
    and the enforced acceptance checklist are in §21.**
- `.7` — **the fuzzing lane (optional) — `parked-optional` (D6, session #51).** Random gen-ASTs × random
  inputs, differential; pushes coverage toward exhaustive (§3.4). PARKED at tree closure: the
  differential surface is already combinator-complete (`.6.1`, 16/16), semantic-construct-complete
  (`.6.2`, 24/24), and shipped-grammar-complete (`.5`, 11 CERTIFIED / DEFERRED empty), so fuzz adds
  diminishing returns relative to its build+maintenance cost; §3.4's honest-bounds wording already
  states the no-fuzz bound. Re-open on demand (e.g. if a future divergence class escapes all three
  existing surfaces).
- `.8` — **first real use: run the A2.3 proof on the harness — `done` (session #51, `PGEN-PARSE-HARNESS-0016`).**
  The capability's first real consumer: the `GRAMMAR-WELLFORMED.A2.3` `FixedTerminalPrefix` soundness
  question, answered with live harness evidence (evidence-only slice — no code change; the disposition
  + fix belong to `GRAMMAR-WELLFORMED.A2.3`, opened this session with this hand-back).
  **Live scratch-slot probe (TOOLBOX 1.3 — its WHEN names exactly this probe).** Body
  `scratch := "a" | "a" "b"` (DEFAULT policy — no annotations), `make focus_scratch` + release-probe
  rebuild, then on input `"ab"`:
  - `--parse` → **ACCEPT** (`parse_full passed`, rc 0); input `"a"` also ACCEPTs.
  - `--parse-dump-ast-pretty` → the typed AST is the TWO-terminal sequence (`Terminal "a"` span 0-1 +
    `Terminal "b"` span 1-2) — the LATER alternative's shape, not the one-terminal earlier alt.
  - `PGEN_TRACE_VERBOSITY=debug … --trace-rules scratch` → codegen's own selection line:
    **`🏁 Rule 'scratch' selected branch 2/2 consuming 2 chars (priority=0, associativity=left,
    branch_policy=longest_match)`** — the engine SELECTS the alternative the linter brands dead.
  - `ast_pipeline <same grammar> --lint-grammar` → `ordered_choice_shadowing=1 (error)`, message
    `alternative #1 is unreachable — alternative #0 is a fixed-terminal prefix of it (PEG commits to
    the earlier alternative)`, **rc=1** — the certifying linter HARD-FAILS a live grammar on a false
    deadness verdict, in the same session the engine demonstrably selects that branch.
  **Fresh `.6.1` gate + scout (the pinned cross-implementation oracle):** `parse_harness_combinator_gate`
  2/2 in 50.91 s; scout map `choice_longest_default CLEAN 4` (accepts `"ab"`), `choice_longest_explicit
  CLEAN 3`, `choice_ordered CLEAN 3` (**REJECTS `"ab"`** — first-alt commit; the one policy where the
  PEG argument holds), `choice_priority_first CLEAN 3` (accepts `"ab"` via the higher-`@priority` later
  alt), `always_succeeds CLEAN 4` (accepts `"keyword"` via the later alt past an always-succeeding
  earlier alt) — interpreter and compile-and-run oracle byte-identical on every case.
  **The handed-back verdict:** `FixedTerminalPrefix`'s "later alternative is unreachable" claim is
  **FALSE under `longest_match` (the DEFAULT) and `priority_first`** — the two policies 100% of
  shipped grammars use — and **TRUE only under `@branch_policy: ordered`** (first-success commit),
  absent branch-phase predicates that could block the earlier alternative. Scratch fixture restored +
  `focus_scratch` + probe rebuild re-run after the probe (the #50 compile-time-embed trap).
- `.9` — **lockstep capstone — `done` (session #51, `PGEN-PARSE-HARNESS-0017`, docs-only).** `TOOLBOX.md` tool entries + probe protocol; the
  **top-level mdBook** sections for each landed component (interpreter / compile-and-run / scratch-slot),
  each documenting it as a structural AST-pipeline component to the depth of the existing linter /
  parser-generator / stimuli-generator chapters (director-directive 2026-07-05 — thorough, no drift,
  landed same-commit as the component); contract (if any); this tree. Note: per the same directive, the
  book coverage of the **existing** structural components was assessed 2026-07-05 and found present
  (linter/stimuli-gen/parser-gen have chapters) — this leaf ADDS the 3 new ones + keeps all in lockstep.
  **Capstone audit (this session):** the per-component obligations were all landed same-commit as their
  components (D5 held throughout): TOOLBOX entries 1.3–1.8 (slot / compile-and-run / interpreter /
  equivalence gate / combinator suite / semantic suite) + quick-chooser rows; the top-book *The Parse
  Harness* chapter covers all three components + both suites + honest bounds (§-audited, 28 sections);
  no contract owed (the harness is an internal diagnostic surface, not a shipped parser family). The
  genuine residue = the canonical PROBE PROTOCOL for the harness's signature question ("which
  alternative WINS / is this branch live?" — the A2.2/A2.3 class), now added as **TOOLBOX Protocol D**
  (+ a quick-chooser row) and mirrored in the book chapter's workflow section. Tree closure: `.7`
  (fuzz) PARKED-OPTIONAL (see §8 D6).

---

## 7. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `PARSE-HARNESS.2` (approach 3 — scratch-register path) | `done` (`PGEN-PARSE-HARNESS-0002`, #38) | Phase A, first authoritative-by-construction harness; richest A2.3 evidence. |
| 2 | `PARSE-HARNESS.3` (approach 2 — compile-and-run) | `done` (`PGEN-PARSE-HARNESS-0003`, #39) | Phase A complete; the self-contained CI oracle for the `.5` equivalence gate; external-crate compile proven + integration test green. |
| 3 | `PARSE-HARNESS.4` (interpreter core — Phase B) | `done` (#40, `PGEN-PARSE-HARNESS-0004`) | The shared-core dynamic dispatcher over the gen-AST — the director's flagship. Byte-identical on the smoke set (json registry + 5 synthetic combinator grammars via the `.3` oracle); 7 tests pass. Tool-mapped plan + committed scope + acceptance checklist in §13. |
| 4 | `PARSE-HARNESS.5` (differential-equivalence gate) | `done` (#41, `PGEN-PARSE-HARNESS-0005`) | `parse_harness_equivalence_gate` landed: deterministic interpreter-vs-generated differential over a bounded stimuli corpus (seeds 0/7/42, large-stack). CERTIFIED byte-identical: json, semantic_annotation, rtl_frontend, vhdl, **systemverilog (sv_2017)**, scratch. DEFERRED ratchet + EXCLUDED classification for the rest (§14/§15). 3 gate tests green; SV cert unchanged. |
| 5 | `PARSE-HARNESS.5.1` (regex fidelity) | `done` (#42, `PGEN-PARSE-HARNESS-0007`) | 4 tool-pinpointed interpreter-fidelity fixes (layout policy / unresolved-ref built-ins / `@transform`+PCRE2 contract / `@profiles` gating). regex CERTIFIED byte-identical (deep stress 400/400). Also closed `.5.4` (svpp, 459/459). Gate 3/3; SV+certified unchanged. §16 checklist. |
| 6 | `PARSE-HARNESS.5.4` (svpp fidelity) | `done` (CLOSED by `.5.1`, #42) | Incidentally closed by `.5.1`'s shared layout policy + built-ins; the `.5` ratchet detected + demanded the promotion. CERTIFIED (459/459). |
| 7 | `PARSE-HARNESS.5.2` (ebnf fidelity) | `done` (#43, `PGEN-PARSE-HARNESS-0009`) | Root cause: interpreter's layout skippers unconditionally skip all 3 comment introducers; codegen suppresses arms per-grammar (H.11.5). Fix gates the arms via codegen's shared predicate. ebnf CERTIFIED byte-identical (DIVERGE 6→CLEAN 83). §17 checklist. |
| 8 | `PARSE-HARNESS.5.3` (return_annotation fold) | `done` (#44, `PGEN-PARSE-HARNESS-0011`) | Root cause (tools REFUTED the scouting hypothesis): the `_pgen_lr_chain` `wrapper_specs` blob was serialized from a non-deterministic std `HashMap` (`UnifiedReturnAST::Object`); codegen froze one arbitrary order, the interpreter re-serialized a fresh (itself non-deterministic) order each load. Fix = a `serialize_with` SORTED serializer canonicalizing every site. return_annotation CERTIFIED (DIVERGE 8→CLEAN 68). Also closed a latent codegen non-determinism. §18 checklist. |
| 9 | `PARSE-HARNESS.5.5` (rtl_const_expr corpus) | `done` (#45, `PGEN-PARSE-HARNESS-0012`) | Curated-input corpus for the deep precedence chain (exceeds bounded gen; unbounded hangs — tool-confirmed §19). A corpus problem, not an interpreter divergence. rtl_const_expr CERTIFIED (DIVERGE 0→CLEAN 151); DEFERRED now empty (11 CERTIFIED). §19 checklist. |
| 10 | `PARSE-HARNESS.6.1` (structural combinator isolating suite) | `done` (`PGEN-PARSE-HARNESS-0014`, #46) | Phase B — 16 isolating grammars, interpreter byte-identical to the `.3` compile-and-run oracle per structural combinator (branch_policy choice incl. `a\|ab`, quantifiers `?`/`*`/`+` incl. zero-length guard, lookahead, sequence-backtrack, atoms/regex-token, rule-ref, LR-eliminated wrapper form). 16/16 CLEAN; gate 2/2 (byte-identity + A2.2/A2.3 discrimination + coverage completeness). 3 tool-findings surfaced (§20). Test-only; no `generated/*` regen. |
| 11 | `PARSE-HARNESS.6.2` (semantic-directive orchestration suite) | `done` (`PGEN-PARSE-HARNESS-0015`, #47) | Phase B — the interpreter's full store-gated orchestration mirror + split memo, certified per construct vs the compile-and-run oracle (**20/20 CLEAN**; pre-mirror baseline 1/18). The `.6` per-construct coverage is COMPLETE. Six tool-established findings surfaced (§21). |
| 12 | `PARSE-HARNESS.7` (fuzz, optional) | `parked-optional` (D6, #51) | Phase B — random gen-ASTs × random inputs, differential. Parked at tree closure: the combinator-complete + semantic-complete + shipped-grammar-complete differential surfaces already cover the trust claim (§3.4 honest bound); re-open on demand. |
| 13 | `PARSE-HARNESS.8` (the A2.3 proof — first real use) | `done` (`PGEN-PARSE-HARNESS-0016`, #51) | Live scratch-slot probe + fresh `.6.1` gate: default `longest_match` SELECTS the "shadowed" later alt on `a\|ab` (`🏁 selected branch 2/2`) while `--lint-grammar` hard-fails the same grammar rc=1 — the `FixedTerminalPrefix` verdict is FALSE under `longest_match`/`priority_first`, TRUE only under `ordered`. Verdict handed to `GRAMMAR-WELLFORMED.A2.3` (opened same session; FIXED there as `-0151`). Evidence-only; no code change. |
| 14 | `PARSE-HARNESS.9` (lockstep capstone) | `done` (`PGEN-PARSE-HARNESS-0017`, #51, docs-only) | Capstone audit: D5 held throughout (TOOLBOX 1.3–1.8 + top-book chapter landed same-commit per component; no contract owed). Residue closed: TOOLBOX **Protocol D** ("which alternative WINS / is this branch live?") + quick-chooser row + the book chapter's canonical-probe section. `.7` parked-optional (D6). **TREE COMPLETE** — all 7 tree-level acceptance criteria met (§5: three approaches ✓, by-construction proofs ✓, by-verification gate 11-CERTIFIED ✓, A2.3 first-real-use landed AND consumed ✓, no regression ✓, full lockstep ✓, honest bounds ✓). |
| 15 | `PARSE-HARNESS.10` (the `ast_pipeline` feature-surface tripwire — ops hardening) | `done` (`PGEN-PARSE-HARNESS-0018`, 2026-07-18, session #149; director-licensed do-first, `PGEN-RGX-0078-0125` GO) | The #140-class single-feature-binary vintage trap fired a THIRD time (`PGEN-RGX-0078-0123` battery): a `--features generated_parsers`-only build silently overwrites the dual-feature `target/debug/ast_pipeline` the compile-and-run oracle shells out to, and the gates fail with a generic codegen error. Fix landed: `ast_pipeline --report-feature-surface` (feature-independent pre-clap probe) + `compile_and_parse` asserts the surface BEFORE codegen → actionable `HarnessError::StaleTool` refusal carrying the exact dual-feature rebuild command (covers both the single-feature and the pre-vintage-binary case). Battery `28 passed; 0 failed` incl. the 4 compile-and-run gates. §22 plan + checklist. **Tree re-closes** (`.7` fuzz stays parked-optional per D6). |

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
- **D6 (2026-07-07, session #51, tree closure).** The optional fuzz lane `.7` is **PARKED**, not built:
  the trust claim is already carried by three complete differential surfaces (per-combinator 16/16,
  per-semantic-construct 24/24, all-shipped-grammars 11 CERTIFIED at seeds 0/7/42), §3.4 documents the
  no-fuzz honest bound, and fuzz's marginal coverage does not justify its build+maintenance cost now.
  Re-open on demand if a divergence class ever escapes all three surfaces.

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

## 11. PARSE-HARNESS.2 — Acceptance Checklist (enforced)

> Per `TOOLBOX.md` / `DOCTRINE_ENFORCEMENT.md`: a CODE change (this leaf touches `grammars/scratch/scratch.ebnf`,
> `rust/src/*`) MUST pass ROOT CAUSE + ADDRESSED + NO REGRESSION, each ticked and evidence-backed.
> This is a **capability build**, so "ROOT CAUSE" = the tool-confirmed capability gap that motivates the slot,
> and "ADDRESSED" = the new capability demonstrated by the toolbox on a synthetic grammar.

- [x] **REPRODUCE / ISSUE** — the capability gap is real: before this slice, driving the shipped parser
  toolbox on an *arbitrary* grammar was impossible. `parseability_probe --parse scratch <input>` and
  `--supports scratch` both reported the grammar as unsupported (no registry entry / no
  `has_generated_scratch_parser` cfg). Confirmed in `.1` DESIGN §1.1 (registry-keyed drive paths only).
- [x] **ROOT CAUSE (WHY + WHERE)** — the drive paths (`parser_registry::parse_sample` /
  `parseability_probe --parse`) are **registry-keyed** to compiled-in grammars; there was no `scratch`
  slot. WHERE: `rust/src/parser_registry.rs` `GENERATED_PARSER_REGISTRY` (no `scratch` entry),
  `rust/build.rs` (no `has_generated_scratch_parser` cfg), `rust/src/lib.rs` `generated_parsers` (no
  `scratch` module), `rust/Makefile` (no `focus_scratch`). Tool-verified via
  `ast_pipeline grammars/scratch/scratch.ebnf --lint-grammar` (grammar well-formed → generatable) and the
  generated-parser naming derivation read from `ast_generator_direct.rs:109` + `ast_based_generator.rs:352`.
- [x] **FIX** — fix-hierarchy tier = **plumbing/registry** (no engine or annotation change): add the blessed
  `grammars/scratch/` slot + the `json`-identical wiring (Makefile `SCRATCH_*` + `focus_scratch`, `build.rs`
  cfg, `lib.rs` module, `parser_registry.rs` dispatch keyed on `parse_full()` so the slot is entry-rule-agnostic).
- [x] **ADDRESSED (verified)** — before→after on the symptom: `parseability_probe --parse scratch "hello, world!"`
  went from **grammar-unsupported → `parse_full passed` (rc 0)** after `make -C rust focus_scratch` + rebuild;
  `--parse-dump-ast-pretty scratch` emits the known typed AST (rooted at `scratch` → `element_1` → `name` =
  text `world`); `--trace-rules scratch` shows the real `ScratchParser::parse_scratch` branch-entry
  (A2.3-style evidence); the known reject `"hello, mars!"` reports `[furthest_position=7]` (rc 1);
  `--lint-grammar` → well-formed (`non_terminating=0`, `ordered_choice_shadowing=0`, `unreachable_rules=0`,
  `profile_orphans=0`, rc 0); `--report-certificate-coverage --entry-rule scratch` →
  `total=2 proof=0 witness=2 UNKNOWN=0 fully_certified=true (sample_parse_failures=0)`. Re-runnable oracle =
  the `#[cfg(has_generated_scratch_parser)]` integration test in `parser_registry.rs`
  (`scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast`) — **1 passed**.
- [x] **NO REGRESSION** — the 6 fully-certified grammars + the SV parser stay **byte-identical** (scratch is
  additive, `#[cfg(has_generated_scratch_parser)]`-gated + keyed on `"scratch"`; only `focus_scratch` ran, all
  other `generated/*_parser.rs` mtimes unchanged; no existing grammar/parser/codegen/dispatch path changed);
  canonical SV cert `1343/10/1321/12` unchanged at seeds 0/7/42 via `sv_cert_recognized_union_gate` (the
  tracked-contract oracle; `sample_parse_failures=0`); `mdbook_docs_gate` GREEN; my hand-written code +
  the scratch generated parser are clippy-clean — the 179 `clippy::eq_op` errors under the generated stage
  are **pre-existing** (114 in shipped `systemverilog_parser.rs`, 64 in `rtl_frontend_parser.rs`; codegen'd
  `"x" == "x"` `branch_policy` constants), unchanged by this slice.
- [x] **LOCKSTEP** — top-level mdBook gains `docs/book/src/parse-harness.md` (the scratch-slot section, to
  the depth of the linter/stimuli/parser-gen chapters, per D5, SAME-COMMIT); `SUMMARY.md` updated;
  `grammars/scratch/README.md` documents the slot; `TOOLBOX.md` scratch-probe entry (brief now; the full
  tool-entry + probe protocol is the `.9` capstone). CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md updated.

## 12. PARSE-HARNESS.3 — Acceptance Checklist (enforced)

> Per `TOOLBOX.md` / `DOCTRINE_ENFORCEMENT.md`: a CODE change (this leaf adds `rust/src/parse_harness.rs`,
> wires `rust/src/lib.rs`) MUST pass ROOT CAUSE + ADDRESSED + NO REGRESSION, each ticked and evidence-backed.
> This is a **capability build**, so "ROOT CAUSE" = the tool-confirmed gap + the make-or-break feasibility
> question the spike had to settle, and "ADDRESSED" = the compile-and-run harness reproducing a *registered*
> grammar's known verdict + typed AST by construction.

- [x] **REPRODUCE / ISSUE** — the gap `.1` DESIGN §1.1 named: there is no way to run the REAL generated parser
  on an arbitrary grammar *without* the full registry+codegen+compile-in ceremony. Approach 3 (scratch) closes
  it by permanently wiring one registry slot + rebuilding `pgen`; approach 2 must do it as a **self-contained
  throwaway** (no registry edit, no `pgen` rebuild) so it can be the CI oracle for the `.5` equivalence gate.
- [x] **ROOT CAUSE (WHY + WHERE)** — the make-or-break design risk: a generated parser source hard-codes
  `use crate::ast_pipeline::{…}` (`generated/*_parser.rs` line 3). Inside `pgen` that resolves fine (same crate
  sees `pub(crate)` items); an **external** throwaway crate sees only `pub` items, so an emitted reference to any
  `pub(crate)` type/method would fail to compile. Tool-verified this session, decisively:
  - static audit — `grep -oE "crate::ast_pipeline::[A-Za-z_]+"` over **all** `generated/*_parser.rs` → **22
    distinct symbols, every one `pub`** (structs/enums/`pub fn`/`pub mod`/`pub use` in `rust/src/ast_pipeline/mod.rs`);
    the only externs are `regex`/`rustc_hash`/`serde_json`. Core parse types (`ParseNode`/`ParseError`/… ) all `pub`;
    `ParseNode` derives `serde::Serialize` (`mod.rs:754`) → the throwaway serializes the **byte-identical** AST that
    the registry's `parse_node_to_json` = `serde_json::to_value(node)` (`parser_registry.rs:1044`) does.
  - empirical spike (`scratchpad/ph_spike`, external crate, path-dep `pgen` + `use pgen::ast_pipeline;` +
    `include!` of the generated file): **compiles and runs**, reproducing registry verdicts on two grammars with
    distinct machinery — `scratch` (`"hello, world!"`→ACCEPT, `"hello, mars!"`→REJECT `furthest=7`) and `json`
    (`{"a":1,"b":[true,null]}`→ACCEPT, `{"a": }`→REJECT `furthest=6`). Exactly one `pub struct <Name>Parser<'input>`
    per generated file (verified across 7 grammars) → struct name is discoverable by grep, no name re-derivation.
- [x] **FIX** — fix-hierarchy tier = **new tooling / plumbing** (no engine, annotation, grammar, or codegen change):
  add `rust/src/parse_harness.rs` (`compile_and_parse(grammar_ebnf, input, &opts) -> ParseOutcome`), `pub mod
  parse_harness;` in `lib.rs`. It shells the pre-built `ast_pipeline` binary for codegen exactly as `RUST_GENERATOR`,
  synthesizes a throwaway external crate (isolated `CARGO_TARGET_DIR`), and marshals the result back — authoritative
  BY CONSTRUCTION (it runs the shipped codegen + runtime; the only trusted surface is this plumbing).
- [x] **ADDRESSED (verified)** — before→after: the capability now EXISTS. The compile-and-run harness
  (`pgen::parse_harness::compile_and_parse`) takes `grammars/json.ebnf` + an input and reproduces the shipped
  `json` parser's behaviour through a throwaway external compile. Re-runnable oracle = the
  `#[cfg(all(feature="generated_parsers", has_generated_json_parser))]` integration test
  `parse_harness::tests::compile_and_run_harness_reproduces_json_registry_verdict_and_ast` — **1 passed**
  (`cargo test --lib --features generated_parsers parse_harness` → `6 passed; 0 failed`). It asserts, for
  `{"a": 1, "b": [true, null, "x"]}`: `outcome.accepted==true` AND `outcome.ast_json ==
  parser_registry::parse_sample_ast_json("json", …)` (the harness's typed AST is **byte-identical** to the
  shipped registry's), and for `{"a": }`: `outcome.accepted==false` matching `parse_sample("json", …)==Some(false)`.
  Plus 5 unit tests for the plumbing (struct-name discovery, sentinel/JSON marshalling, reject shape). Manual
  spike parity earlier this session: `scratch` (`"hello, world!"`→ACCEPT, `"hello, mars!"`→REJECT `furthest=7`)
  and `json` (`furthest=6` on the reject) — both matching the scratch slot / registry.
- [x] **NO REGRESSION** — the change is **purely additive tooling**: a new module `rust/src/parse_harness.rs` +
  one `pub mod parse_harness;` line in `lib.rs`; it is **never invoked by any parse/codegen/cert path**. `git
  status` confirms only `rust/src/parse_harness.rs` (new), `rust/src/lib.rs`, and the two docs files changed; **no
  `generated/*_parser.rs` regenerated** (mtimes unchanged — SV `12:37`, json `20:30`, scratch `12:13`), so the 6
  fully-certified grammars + SV are **byte-identical by construction**. SV cert re-verified unchanged via
  `sv_cert_recognized_union_gate` (the tracked-contract oracle) at seeds 0/7/42 → canonical `1343/10/1321/12`,
  union `1343/10/1332/1`, `sample_parse_failures=0`. `parse_harness` is **clippy-clean** (`cargo clippy --lib
  --tests --features generated_parsers` — 0 findings in the module; the 179 generated-stage `eq_op` errors are
  pre-existing, unchanged). `mdbook_docs_gate` **GREEN**. Full lib builds with `--features generated_parsers`.
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` gains the **compile-and-run harness**
  section (D5, SAME-COMMIT, to the depth of the scratch-slot / linter / stimuli / parser-gen chapters: what it is,
  the API, the 5-step plumbing, the load-bearing external-compile fact, the by-construction trust argument,
  cost/reuse) + the approaches table row promoted `forthcoming → landed`; `TOOLBOX.md` gains a compile-and-run
  entry (§1.4); CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md updated.

## 13. PARSE-HARNESS.4 — implementation plan, tool-mapped design, committed scope, acceptance checklist

> Session #40. Two focused code-mapping passes over the tree (recorded here so the design is durable, not
> conversation-bound) established exactly where the interpreter's "thin dynamic-dispatch layer" boundary is
> (D2). The maps are the WHY+WHERE evidence for the build.

### 13.1 The shared-core boundary (tool-mapped — this is what D2 predicted, now precise)

A generated parser is ~9K lines per grammar, but its *combinator control-flow is fully inlined per-rule as
codegen `quote!` templates* — there is **no shared `impl`** for it. What survives as callable runtime is
almost entirely the **semantic + type layer**. So the interpreter splits cleanly into three sections:

- **Section A — reuse VERBATIM (the shared core; all `pub` in `crate::ast_pipeline`):** `SemanticRuntimeState`
  (checkpoint / rollback_to_named / extract_delta_since / apply_delta / emit_fact / open_scope / close_scope /
  apply_directive(s) / evaluate_predicate / evaluate_content_aware_predicate / push_rule_context /
  pop_rule_context), `CompiledSemanticRuntimeAnnotations` (+ `compile_semantic_runtime_annotations`),
  `ParseNode` / `ParseContent` (+ `to_json_value`), `RecursionGuard`, `parse_quantifier_bounds`,
  `UnifiedReturnAST`, `SemanticBranchPolicy`, `ParseError` / `ParseResult`, the gen-AST IR (`ASTNode` /
  `ASTValue` / `TokenValue` / `Annotations` / `BranchAnnotation`), and the in-process gen-AST loader
  (`ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope` → `RustASTPipeline::transform_from_raw_ast`).
- **Section B — re-implement (parser-AGNOSTIC, mechanical; exist ONLY as codegen templates):** `try_parse`
  (position + `semantic_runtime_state.checkpoint()` save, `rollback_to_named` on `Err`), `match_string`,
  `match_regex` (anchored `\A(?:pat)` + empty-match probe), `consume_layout_for_terminal` /
  `consume_layout_for_regex` / `consume_optional_whitespace` / `consume_horizontal_whitespace`,
  `bytes_match_at`, `regex_token_matches_at_cursor`. **Mirrored byte-for-byte from the emitted code in a
  generated parser** (the authoritative behavior), not from prose.
- **Section C — re-express the dispatch (a dynamic walk over `ASTNode`):** the ordered-choice tournament +
  `branch_policy` (longest_match / ordered / priority_first) + per-branch `@priority` + associativity;
  sequence assembly (each element wrapped `ParseNode{rule_name:"element_{i}", content, span}`; the `?`-element
  special case → inner content or empty `Sequence`); the quantifier loop (`parse_quantifier_bounds` + per-iter
  `try_parse` + zero-length guard + `SAFETY_LIMIT=10_000` + min/max); lookahead (`&`/`!`, zero-width); the
  return-annotation fold (`AstReturnTransformer::generate_transform` mirrored at runtime — see 13.3); the
  synthetic `-> $1` passthrough default for single-element bodies (`body_has_single_element`: `Sequence` len≤1
  or non-`Quantified` atom/Or/Lookahead → `true`; `Quantified` → `false`); the rule-node wrap
  (`ParseNode{rule_name:<rule>, content:<result>, span:start..end}`) + monotonic `furthest_position` at entry.

### 13.2 Atom / node shapes (tool-verified from the emitted `json` + `rtl_const_expr` parsers)

- `Atom(Token[tag,val])`: `quoted_string`/`number`/`probability`/`include_dir`/`include_file`/`rule` →
  `match_string(val)` → `ParseContent::Terminal(matched)`; `regex` → `match_regex(val, true)` →
  `Terminal(matched)`; `rule_reference` → `ParseContent::Alternative(Box::new(parse_rule(val)))`; anything
  else / `parts.len()<2` → empty `Terminal("")` (consumes 0).
- `Sequence{elements}` → `ParseContent::Sequence(vec![ParseNode{rule_name:"element_{i}", content, span}, …])`.
- `Quantified{element,quantifier}` → `ParseContent::Quantified(vec![iter_node…], intern(quantifier))`.
- `Lookahead{element,positive}` → zero-width (position restored); consumes nothing.
- `Or{alternatives}` → the winning branch's (transformed) content; single-branch and multi-branch paths differ
  (multi-branch = the try_parse tournament).

### 13.3 The return-fold model (decisive detail — nails byte-identity on `json`)

Every codegen call site passes `captured_vars = &["result"|"content"]` (len 1) — verified at
`ast_based_generator.rs:2499,3031,3169`. So the fold's `$N` always resolves against a **single base** = the
branch's raw structural `ParseContent`, with the len==1 rule (`generate_positional_ref`): `$1` →
`match &base { Sequence(elements) if !empty => elements[0].content.clone(), Alternative(node) =>
node.content.clone(), other => other.clone() }`; `$N` (N>1) → `Sequence(elements) if len>N-1 =>
elements[N-1].content.clone()` else `Terminal("<invalid_sequence_access>")`. Object → `Json(Object)` (keys
sorted); Array → `Sequence(vec)` with Spread/FlattenSpread flattening; literals → the typed `Json(...)` /
`Terminal(interned)` the codegen emits. The runtime fold is a faithful mirror of `AstReturnTransformer`.

### 13.4 Committed scope (what `.4` delivers vs what `.5`/`.6` extend — honest bounds, §3.4)

- **IN `.4`:** `pgen::parse_harness_interpreter::interpret_parse(grammar_ebnf, input, &opts) -> ParseOutcome`
  (+ an in-process gen-AST core entry), reusing `crate::parse_harness::ParseOutcome` so `.5` can diff it
  against `compile_and_parse` trivially. Full combinator dispatch (Sections B/C) + the return-fold, reusing
  Section A verbatim, with a string interner for `&'static str`. **Verified byte-identical** against the
  oracle on the smoke set: **json** (registry `parse_sample_ast_json` — byte-identical, no compile; covers
  Or / Sequence / regex-token / rule-ref / return-fold / layout) + **synthetic per-combinator grammars** via
  the `.3` `compile_and_parse` oracle (quantifier forms, lookahead `&`/`!`, ordered-vs-longest choice,
  priority). Verdict (accept/reject) + `furthest_position` are the rock-solid core (structure-only, no
  content-shape subtlety).
- **DEFERRED to `.5`/`.6` (threaded but not the `.4` smoke set — the honest boundary):** the
  **semantic-directive orchestration** (store-gated parsing — `@predicate` branch/pre/post gates changing the
  verdict, `@emit_fact`/scope effects, the `$reference`-against-content resolution, `@import`/`@export`
  library); **memoization** with semantic-delta replay (a transparent cache — AST-invariant — added in `.5`
  where the full corpus, incl. SV, needs it for performance/termination); **full-corpus** byte-identity across
  all registered grammars. `SemanticRuntimeState` is constructed + threaded through `try_parse` (speculation
  snapshots it, faithfully) so `.6` extends without restructuring; rules that carry semantic directives are
  detected (`CompiledSemanticRuntimeAnnotations::has_rule`) and the smoke set excludes grammars that *gate
  parse outcomes* on the store. No over-claim: `.4` is byte-identical on the **structural + return-annotation**
  surface, proven on the smoke set; the combinator-complete + full-corpus proof is `.5`/`.6`.

### 13.5 PARSE-HARNESS.4 — Acceptance Checklist (enforced)

> A CODE change (adds `rust/src/parse_harness_interpreter.rs`, wires `rust/src/lib.rs`). Capability build:
> "ROOT CAUSE" = the tool-mapped shared-core boundary that makes the interpreter a *thin* dispatcher (the
> make-or-break design fact); "ADDRESSED" = the interpreter reproducing the generated parser's verdict +
> byte-identical typed AST on the smoke set, differentially against the authoritative oracle.

- [x] **REPRODUCE / ISSUE** — the capability gap `.1` §1.1: no in-process, no-codegen way to parse an input
  against an arbitrary grammar. Approaches 2/3 are authoritative-by-construction but pay a per-probe compile;
  approach 1 (this leaf) is the fast in-process interpreter — its trust must be *earned* (§3), so it is
  verified against the by-construction oracle.
- [x] **ROOT CAUSE (WHY + WHERE)** — the tool-mapped shared-core boundary (13.1): the combinator half exists
  ONLY as codegen `quote!` templates (`ast_based_generator.rs` generate_or_logic @2995 / generate_sequence_element
  @3590 / generate_quantified_logic @3859 / generate_lookahead_logic @2930 / generate_atom_logic @3647 +
  `AstReturnTransformer::generate_transform` @ast_return_transform.rs:14), the semantic half is callable runtime
  (`semantic_runtime.rs` / `mod.rs`, all `pub`: `ParseNode`@755 / `ParseContent`@715 / `parse_quantifier_bounds`@916
  / `SemanticRuntimeState`@1205). So the interpreter reuses Section A verbatim and re-expresses Sections B/C — the
  divergence surface is exactly the combinator dispatch, which the differential oracle then pins. The one wrong
  guess (quantifier iteration node) was caught by the oracle on the first run and fixed to `rule_name:"quantified"`
  / `span:0..0` / zero-length-discard by reading `generate_quantified_logic` @3986-4009.
- [x] **FIX** — new tooling module `rust/src/parse_harness_interpreter.rs` (fix-hierarchy tier = new tooling /
  plumbing; no engine / grammar / codegen / existing-runtime change); `pub mod parse_harness_interpreter;` in
  `rust/src/lib.rs`. `interpret_parse` (`.ebnf`-loading, `ebnf_dual_run`-gated) + `interpret_parse_gen_ast` (core).
- [x] **ADDRESSED (verified)** — before→after: the in-process interpreter capability now EXISTS. Re-runnable
  oracles = the differential smoke tests (`cargo test --lib --features "generated_parsers ebnf_dual_run"
  parse_harness_interpreter` → **7 passed; 0 failed**): (1) `interpreter_is_byte_identical_to_the_json_registry_parser`
  — `interpret_parse` byte-identical to `parser_registry::parse_sample_ast_json("json", …)` across 7 accepts + 4
  rejects (multi-branch `Or`/longest_match, sequence, regex tokens, rule-ref recursion, object/array/spread fold,
  layout); (2) `interpreter_agrees_with_compile_and_run_on_synthetic_combinators` — byte-identical verdict +
  `furthest_position` + typed AST vs `parse_harness::compile_and_parse` on 5 synthetic grammars (A2.3 fixed-prefix,
  `*`/`+`, optional `?`, `&`/`!` lookahead); + 5 unit tests.
- [x] **NO REGRESSION** — purely additive tooling (new module + one `pub mod` line); NEVER invoked by any
  parse/codegen/cert path; no `generated/*_parser.rs` regenerated (mtimes unchanged) → the 6 fully-certified
  grammars + SV byte-identical by construction; SV cert re-verified unchanged via `sv_cert_recognized_union_gate`
  at seeds 0/7/42 → canonical **1343/10/1321/12**, union **1343/10/1332/1**, `sample_parse_failures=0`; the new
  module clippy-clean (`cargo clippy --lib --features "generated_parsers ebnf_dual_run"` — 0 findings in the
  module; the 179 generated-stage `eq_op` errors are pre-existing, unchanged); `mdbook_docs_gate` GREEN.
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` gains the **grammar-AST interpreter**
  section (D5, SAME-COMMIT, to the depth of the scratch-slot / compile-and-run / linter / stimuli / parser-gen
  chapters: what it is, the API + core entry, the by-verification trust argument, the shared-core boundary, the
  interning-to-`'static` subtlety, honest bounds) + the approaches-table row promoted `forthcoming → core landed`;
  `TOOLBOX.md` §1.5 interpreter entry + quick-chooser row; CHANGES.md / DEVELOPMENT_NOTES.md (+ the novel
  shared-primitive-factoring observation, surfaced for director feedback) / MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md
  updated.

## 14. PARSE-HARNESS.5 — the differential-equivalence gate: plan, tool-mapped design, acceptance checklist

> Session #41. A CODE change (adds `rust/src/parse_harness_equivalence.rs`, wires `rust/src/lib.rs`,
> adds the `parse_harness_equivalence_gate` Makefile target). Capability build: "ROOT CAUSE" = the
> tool-established honest per-grammar split that scopes the certified baseline vs the deferred closures;
> "ADDRESSED" = the enforcing gate certifying 6 grammars byte-identical + ratcheting the rest.

### 14.1 What `.5` delivers (honest scope)

The differential-equivalence gate that makes the interpreter (`.4`) authoritative-by-verification. It
runs, per registered grammar, the interpreter (`interpret_parse_gen_ast`) and the shipped generated
parser (`parser_registry::parse_sample*`) over ONE deterministic corpus and asserts byte-identical
verdict + typed AST. The corpus is the grammar's own **stimuli generator** at seeds 0/7/42 over a bounded
depth ladder (`generate_many_bounded` — the same deterministic step budget the cert-coverage pass uses,
so the super-linear generators cannot hang) + first-half **truncation probes** for reject-path parity.
The differential runs on a **large-stack worker** (`LARGE_STACK_BYTES` = 512 MiB) so a deeply-nested
sample cannot overflow the small default test-thread stack and abort the run
([[feedback_recursion_ceiling_must_bound_the_real_stack]]).

Rather than certify all registered grammars in one leap (the interpreter is not byte-identical on all of
them yet — §15), `.5` lands the gate INFRASTRUCTURE + the CERTIFIED byte-identical baseline and an
**honest three-way classification** (the no-silent-caps discipline, [[feedback_always_signoff_decisions]]):

- **CERTIFIED** (`json`, `semantic_annotation`, `rtl_frontend`, `vhdl`, `systemverilog` @ sv_2017,
  `scratch`) — must stay byte-identical; a regression fails the gate.
- **DEFERRED** (`regex`, `ebnf`, `return_annotation`, `systemverilog_preprocessor`, `rtl_const_expr`) —
  the gate asserts they are STILL divergent (a RATCHET: if one becomes byte-identical the gate fails,
  demanding it be promoted to CERTIFIED — progress is never lost silently). Each owns a `.5.x` leaf.
- **EXCLUDED** (`builtin_return_annotation`, `builtin_semantic_annotation`) — out of scope by
  construction: their registry oracle is NOT a codegen parser of their own `.ebnf`
  (`builtin_return_annotation` aliases the `return_annotation` parser; `builtin_semantic_annotation`
  uses the hand-rolled `UnifiedSemanticAST::parse_bootstrap`), so the differential's premise fails.

A COMPLETENESS test asserts every registered grammar is classified exactly once, so a newly-registered
grammar cannot be silently unmeasured. This is a corpus-scoped certification (byte-identical over THIS
deterministic corpus), NOT an all-inputs proof — the honest bound of §3.4; the `.6` combinator+semantic
suite and `.7` fuzz push coverage further.

### 14.2 The honest scope decision (why re-scoped into `.5` + `.5.1`–`.5.5`)

The tools-first measurement (§15) — not a guess — established that the interpreter is ALREADY
byte-identical on 6 grammars (including the store-using SV/VHDL/rtl_frontend, a better result than
`.4`'s honest bound predicted) but genuinely diverges on 4 (regex/ebnf/return_annotation/svpp) and has
no corpus for 1 (rtl_const_expr). Fixing all 4 divergences in one commit would violate one-clean-slice;
so `.5` lands the gate + certified baseline and each remaining grammar gets its own root-cause+fix leaf
(`.5.1`–`.5.5`). This is a legitimate task-tree refinement (discovering subtasks), fully honest.

### 14.3 PARSE-HARNESS.5 — Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the `.4` interpreter's trust was proven only on a smoke set (json + 5
  synthetic grammars); its fidelity across the real registered-grammar corpus was UNMEASURED. Before this
  leaf there was no re-runnable gate proving (or disproving) interpreter==generated-parser per grammar.
- [x] **ROOT CAUSE (WHY + WHERE)** — the tools-first measurement (`parse_harness_equivalence::measurement`,
  §15) established the exact per-grammar split + the WHY for each non-certified grammar: `builtin_*` are
  registry aliases/bootstrap (not own-grammar codegen — `parser_registry.rs:218,232`); `rtl_const_expr`'s
  deep precedence chain exceeds bounded generation depth (probe: "depth exceeded expanding
  `multiplicative_expr`" at depth 20) and unbounded deep generation hangs (the known super-linear
  pathology); `regex`/`ebnf`/`return_annotation`/`svpp` have concrete interpreter divergences (verdict/AST,
  first-divergence sample + byte offset recorded per grammar). Two interpreter robustness gaps also
  surfaced + were handled: a **stack overflow** on deep samples (fixed via the large-stack worker) and
  no-memo **slowness** on deep recursion/store-heavy grammars (bounded the corpus; memoization is `.6`).
- [x] **FIX** — fix-hierarchy tier = **new tooling / plumbing** (no engine / grammar / codegen / existing
  runtime change): new module `rust/src/parse_harness_equivalence.rs` (the report-first differential
  driver + the CERTIFIED/DEFERRED/EXCLUDED classification + the large-stack harness + the 3 gate tests),
  `#[cfg(all(feature="ebnf_dual_run", feature="generated_parsers"))] pub mod parse_harness_equivalence;`
  in `lib.rs` (BOTH features — the differential needs the `.ebnf` loader AND the registry oracle), and the
  `parse_harness_equivalence_gate` Makefile target.
- [x] **ADDRESSED (verified)** — the capability now EXISTS + is enforced. Re-runnable oracle = the 3 gate
  tests (`make -C rust parse_harness_equivalence_gate` / `cargo test --features "generated_parsers
  ebnf_dual_run" --lib parse_harness_equivalence::gate` → **3 passed**): (1)
  `every_registered_grammar_is_classified_exactly_once` (completeness), (2)
  `certified_grammars_are_byte_identical` (json 117 / semantic_annotation 134 / rtl_frontend 143 / vhdl 78
  / systemverilog 94 / scratch 3 samples — all byte-identical verdict + typed AST), (3)
  `deferred_grammars_are_still_divergent_or_promote_them` (the ratchet). Deterministic (seeds 0/7/42,
  bounded generation, large-stack).
- [x] **NO REGRESSION** — purely additive tooling (new module + one `pub mod` line + one Makefile target);
  NEVER invoked by any parse/codegen/cert path; no `generated/*_parser.rs` regenerated (`git status` shows
  only `rust/src/lib.rs` + the new module + docs) → the 6 fully-certified grammars + SV byte-identical by
  construction; SV cert re-verified unchanged via `sv_cert_recognized_union_gate` at seeds 0/7/42 →
  canonical **1343/10/1321/12**, union **1343/10/1332/1**, `sample_parse_failures=0`; the new module is
  clippy-clean; the module is correctly EXCLUDED under an `ebnf_dual_run`-only build (`cargo check
  --features ebnf_dual_run` clean — the `parser_registry`-needs-`generated_parsers` cfg bug the SV-cert
  build first caught, then fixed).
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` gains the **differential-equivalence
  gate** section (D5, SAME-COMMIT: what it is, the corpus, the CERTIFIED/DEFERRED/EXCLUDED classification,
  the large-stack + bounded-generation robustness, the honest corpus-scoped bound) + the approaches-table
  updated; `TOOLBOX.md` gains a gate entry; CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md /
  LIVE_ACHIEVEMENT_STATUS.md updated.

## 15. PARSE-HARNESS.5 — the differential measurement map (tool-established, session #41)

The `parse_harness_equivalence::measurement` scouting tests (re-runnable with `--ignored --nocapture`)
established the honest per-grammar state. Corpus per grammar: stimuli `generate_many_bounded` at seeds
0/7/42 over depth ladder `[6,12,18]`, count 8/rung, + first-half truncation probes; interpreter vs
`parser_registry` oracle; large-stack workers.

| Grammar | Verdict | Samples | Note (WHY) |
| --- | --- | --- | --- |
| `json` | ✅ CERTIFIED | 117 | byte-identical (also proven at 132 on a deeper ladder). |
| `semantic_annotation` | ✅ CERTIFIED | 134 | byte-identical. |
| `rtl_frontend` | ✅ CERTIFIED | 143 | byte-identical — store-using, yet no divergence on this corpus. |
| `vhdl` | ✅ CERTIFIED | 78 | byte-identical. |
| `systemverilog` (sv_2017) | ✅ CERTIFIED | 94 | byte-identical — the big store-gated/profiled grammar. ~18 s (no-memo). |
| `scratch` | ✅ CERTIFIED | 3 | byte-identical (blessed fixture). |
| `regex` | ✅ CERTIFIED (`.5.1`, #42) | 400 | byte-identical after the 4 fidelity fixes (layout policy / built-ins / `@transform`+PCRE2 contract / `@profiles`); deep stress 5 seeds × depths 6-30. |
| `ebnf` | ✅ CERTIFIED (`.5.2`, #43) | 83 | byte-identical after gating the interpreter's layout comment arms via codegen's per-introducer suppression predicate (H.11.5); ebnf claims `/*` as a non-comment token so its `/* */` block-comment layout arm is suppressed. DIVERGE 6→CLEAN. |
| `return_annotation` | ✅ CERTIFIED (`.5.3`, #44) | 68 | byte-identical after canonicalizing the `_pgen_lr_chain` `wrapper_specs` blob (a `serialize_with` SORTED serializer on `UnifiedReturnAST::Object.properties`) — it was serialized from a non-deterministic std `HashMap`; codegen froze one arbitrary order, the interpreter re-serialized a fresh non-deterministic order each load. DIVERGE 8→CLEAN. Also closed a latent codegen non-determinism. |
| `systemverilog_preprocessor` | ✅ CERTIFIED (`.5.1`, #42) | 459 | byte-identical — the `.5.1` shared layout policy (regex-token whitespace-sensitivity) + built-ins closed the `.5.4` span/shape divergence; deep stress 459/459. |
| `rtl_const_expr` | ✅ CERTIFIED (`.5.5`, #45) | 151 | byte-identical over a **curated** input corpus (the stimuli generator yields 0 usable samples: its ~16-level precedence cascade needs depth ≳30 to reach a leaf — depths ≤28 fail, ~32 emits pathologically-huge exprs, ≥40 hang). A corpus fix, not a fidelity fix. DIVERGE 0→CLEAN. |
| `builtin_return_annotation` | ⛔ EXCLUDED | — | oracle = the `return_annotation` parser (different grammar). |
| `builtin_semantic_annotation` | ⛔ EXCLUDED | — | oracle = hand-rolled `parse_bootstrap`, not codegen. |

**Notable (surfaced to the director):** the interpreter is byte-identical on the *complex* store-using
grammars (SV/VHDL/rtl_frontend) but diverges on the *simpler* meta/annotation grammars
(regex/ebnf/return_annotation/svpp) — the divergences concentrate in the **return-annotation fold** +
specific regex constructs, NOT in the semantic store as `.4`'s honest bound had anticipated. That
reshapes where the remaining `.5`/`.6` work is (fold + regex fidelity first; deep store-gated-outcome
orchestration is exercised further only in `.6`). Two interpreter robustness gaps were also found and
handled (stack overflow → large-stack worker; no-memo slowness → bounded corpus, memoization deferred).

## 16. PARSE-HARNESS.5.1 (+ .5.4 closure) — Acceptance Checklist (enforced)

> Session #42. A CODE change (interpreter-tooling only): edits `rust/src/parse_harness_interpreter.rs`,
> `rust/src/parse_harness_equivalence.rs`, `rust/src/parser_registry.rs` (two NEW pub helpers only). NO
> engine / grammar / codegen / generated-parser change. Capability build: "ROOT CAUSE" = the four
> tool-pinpointed interpreter-fidelity gaps (§`.5.1` leaf); "ADDRESSED" = regex + svpp certified
> byte-identical, differentially, over a deterministic + deep-stress corpus.

- [x] **REPRODUCE / ISSUE** — `parse_harness_equivalence::measurement PGEN_PHEQ_ONLY=regex` (ladder
  [6,12,18]) → `DIVERGE samples=45 agree=42 diverge=3`; the deep stress (ladder 6-30) surfaced the full
  divergence set (greediness fold, `$+` verdict, `é`/`\Q…\E` verdict, counted-quantifier fold, `(*verb:)`
  over-acceptance). svpp DEFERRED `.5.4` for an AST span/shape divergence.
- [x] **ROOT CAUSE (WHY + WHERE)** — FOUR tool-backed causes (full detail in the `.5.1` leaf): (1)
  whitespace-sensitivity keyed on the grammar name (`ast_based_generator.rs:4509/4510/1144` → generated
  `match_string`/`match_regex`/`parse_full` emit `if false { consume_layout… }`); the interpreter skipped
  layout unconditionally → `greediness:"lazy"` vs `[]` on `\Q]\E* ?`. (2) `generate_unresolved_reference_method`
  (`:833-946`) native built-ins `builtin_any_char`/`builtin_ascii_char` (via `unicode_char = !builtin_ascii_char
  builtin_any_char`); the interpreter hard-errored on undefined refs → rejected every non-ASCII char at
  `furthest≈3`. (3) `generate_post_body_span_transform` (`:4094`) `@transform` numeric coercion of `digits`
  (`{min:12}` vs `["1","2"]`) + the `validate_regex_compile_contract` PCRE2 post-parse layer in `parse_sample`
  (`parser_registry.rs:357` — rejects `$+`). (4) `@profiles` rule-entry gating (`:2692-2704`/`:4903`/`:7108`);
  the interpreter ignored the active profile → accepted the relaxed-only `directive_name_relaxed` under strict
  `pcre2`.
- [x] **FIX** — fix-hierarchy tier = **new interpreter tooling** (no engine/grammar/codegen touched): a
  per-grammar `LayoutPolicy` + `parse_unresolved_reference` + `apply_post_body_span_transform`/`rule_span_transform`
  + `@profiles` gating (`rule_profiles`/`profile_enabled` + threaded active profile), each mirroring the cited
  codegen verbatim; plus two NEW parser-agnostic `parser_registry` helpers (`post_parse_semantic_contract`,
  `active_grammar_profile`) the gate applies symmetrically.
- [x] **ADDRESSED (verified)** — before→after, re-runnable oracles: `probe_regex_deep_stress` (5 seeds ×
  depths 6-30) — regex `DIVERGE 3` → **`CLEAN 400/400`**, svpp **`CLEAN 459/459`**; `probe_regex_divergence_minimizer`
  agrees on every fixed construct (`\Q]\E* ?`→greedy, `$+`→reject, `(*H_2Y-:)`→reject); `make -C rust
  parse_harness_equivalence_gate` → **3 passed** (`certified_grammars_are_byte_identical` now includes regex+svpp;
  the ratchet forced the svpp promotion; completeness holds).
- [x] **NO REGRESSION** — purely additive tooling; `git status` = only the 3 source files (NO
  `generated/*_parser.rs` regenerated → the fully-certified grammars + SV byte-identical by construction);
  the equivalence gate re-proves json/semantic_annotation/rtl_frontend/vhdl/**systemverilog (sv_2017)**/scratch
  STILL byte-identical (SV under the NEW `@profiles`-aware interpreter — profile-awareness can only align
  interp→oracle, never diverge); interpreter unit tests **7/7**; SV cert re-verified unchanged via
  `sv_cert_recognized_union_gate` (canonical `1343/10/1321/12`, seeds 0/7/42); my new code clippy-clean (0
  findings after the one `collapsible_if` collapse; the generated-stage `eq_op` errors are pre-existing).

> **LIVE-SPEC note (2026-07-07, session #54).** The `.5.1` root-cause #1 grammar-NAME layout gate
> (`allow_layout_skip_for_terminals`/`_for_regexes`/`allow_trailing_layout` keyed on
> `grammar_name != "regex"` / `"systemverilogpreprocessor"`, mirrored expression-for-expression by the
> interpreter's `grammar_layout_policy(grammar_name)`) — the latent tension recorded when `.5.1` landed —
> is **RESOLVED by `WS-DIRECTIVE.2`**: the layout policy is now declared IN the grammar via the
> grammar-level `@whitespace_sensitive:` directive (regex `true`; svpp `{ regex_tokens: true }`), compiled
> once by `semantic_runtime::compile_layout_sensitivity` and consumed by BOTH codegen and the interpreter
> (the name literals are deleted; all generated parsers byte-identical; the `.6.1` suite grew 3 `layout_*`
> isolating cases, 20→23). See `docs/tasks/WS-DIRECTIVE.md`.
>
> **LIVE-SPEC note (2026-07-07, session #55).** The SIBLING name-gate class — the regex→`pcre2`
> default-profile literals (`parser_registry.rs` / `main.rs` / `embedding_api.rs`) that the `.5.1`
> interpreter had to normalize through `active_grammar_profile` — is **RESOLVED by
> `DEFAULT-PROFILE.2`**: the default is now the grammar-level `@default_profile:` directive
> (regex `pcre2`), compiled once by `semantic_runtime::compile_default_profile`, burned into the
> generated parser (constant + ctor + `set_grammar_profile(None)` restore), and resolved by the
> interpreter as requested-or-declared-default; the `.6.1` suite grew the
> `profile_unspecified_permissive`/`profile_default_gate` pair, 23→25. See
> `docs/tasks/DEFAULT-PROFILE.md`.
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` (the differential-equivalence gate
  section: CERTIFIED/DEFERRED lists + the four regex-fidelity root causes); `TOOLBOX.md` §1.6 CERTIFIED/DEFERRED
  lists; this tree (`.5.1` done + checklist, `.5.4` closed, frontier, §15 map); CHANGES.md / DEVELOPMENT_NOTES.md
  / MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md.

## 17. PARSE-HARNESS.5.2 — ebnf fidelity — Acceptance Checklist (enforced)

> Session #43. A CODE change (interpreter-tooling only): edits `rust/src/parse_harness_interpreter.rs`
> (gate the layout skippers' comment arms) + `rust/src/ast_pipeline/ast_based_generator.rs` (expose a
> SHARED read-only query, no emit-path change). NO engine / grammar / codegen-emit / generated-parser
> change. Capability closure: "ROOT CAUSE" = the interpreter's layout skippers unconditionally skip all
> three comment introducers while codegen SUPPRESSES arms per-grammar (H.11.5); "ADDRESSED" = ebnf
> certified byte-identical, differentially, over the deterministic + deep-stress corpus.

- [x] **REPRODUCE / ISSUE** — `PGEN_PHEQ_ONLY=ebnf cargo test … parse_harness_equivalence::measurement`
  (ladder [6,12,18], seeds 0/7/42), THIS session → `ebnf DIVERGE samples=83 agree=77 diverge=6`. ALL 6 one
  class — the interpreter ACCEPTS a comment/layout-only input the generated ebnf parser REJECTS:
  `"/**/"` (interp furthest=3), `"/*"` (furthest=2), `"       /**//***/"` (furthest=10) + 3 more of the
  same shape. Verdict-only (`interp.accepted=true (grammar-parse=true)` vs `oracle.accepted=false`).
- [x] **ROOT CAUSE (WHY + WHERE)** — tool-established by reading the EMITTED oracle + codegen (the scouting
  log's mandated next step). The generated ebnf parser's `consume_layout_for_terminal`
  (`generated/ebnf.rs:70486-70523`) has arms for `#` and `//` but **NO `/* */` block-comment arm**. This is
  codegen's per-introducer static suppression (GRAMMAR-WELLFORMED.H.11.5): `terminal_block_comment_arm = if
  claims_block_comment { EMPTY } else { EMIT }` (`ast_based_generator.rs:4713-4733`), where
  `claims_block_comment = grammar_claims_introducer_as_non_comment(tree, "/*")` (`:4522`) is **true** for
  ebnf because `block_comment := "/*" block_comment_content "*/"` (`grammars/ebnf.ebnf:560`) makes `"/*"` a
  real, non-comment grammar token — so codegen does NOT treat `/* */` as layout and requires structural
  matching. The interpreter's `consume_layout_for_terminal` (`parse_harness_interpreter.rs:1044-1101`) and
  `consume_layout_for_regex` (`:1103-1167`) **unconditionally** include all three comment arms
  (`#`/`//`/`/*`), so on `/**/` the interpreter skips the block comment as layout, reaches EOF, and ACCEPTs;
  the generated parser (no `/*` layout arm) requires the `comment` alternative to match structurally in the
  `grammar_file` `*`-loop and REJECTs. The per-grammar arm matrix was read directly from ALL 10 generated
  parsers (json/svpp/rtl_const_expr/return_annotation = all 3 arms; regex/vhdl/systemverilog/rtl_frontend =
  no `#` arm; semantic_annotation = `#` only; ebnf = no `/*` arm), confirming the interpreter's unconditional
  arms are a LATENT over-skip for every suppressed cell — ebnf's `/*` is simply the first the corpus
  witnessed. (The unterminated-`/*` leniency the scouting log flagged is subsumed: once the `/*` arm is
  suppressed for ebnf it no longer applies; where an arm IS kept, the generated arm has the SAME lenient
  close, so the interpreter stays faithful.)
- [x] **FIX** — fix-hierarchy tier = **new interpreter tooling + a SHARED read-only codegen query** (single
  source of truth, no drift; codegen's emit path is NOT touched). (a) `ast_based_generator.rs`: a
  `pub(crate) struct CommentArmSuppression` + a `pub(crate) fn comment_arm_suppression_for_grammar(grammar_name,
  grammar_tree, annotations)` that reconstructs codegen's minimal generator config
  (`AstBasedGenerator::new(snake_to_pascal(name))` + `.annotations`) and calls codegen's OWN predicate
  `grammar_claims_introducer_as_non_comment` for `#`/`//`/`/*` — so the interpreter computes the IDENTICAL
  booleans codegen emits with, and can never drift. (b) `parse_harness_interpreter.rs`: `Interp` gains a
  `comment_arms: CommentArmSuppression` computed via that function; each of the three arms in
  `consume_layout_for_terminal` + `consume_layout_for_regex` is gated on `!claims_*`, mirroring codegen's
  `terminal_*_arm` / `regex_*_arm` suppression.
- [x] **ADDRESSED (verified)** — before→after, re-runnable oracles. `PGEN_PHEQ_ONLY=ebnf …::measurement`
  (ladder [6,12,18], seeds 0/7/42) → `ebnf DIVERGE 6` → **`CLEAN agree=83 diverge=0`**; the deep-stress
  probe (`probe_layout_deep_stress`; ebnf widens to **5 seeds** × the gate depths [6,12,18], 16/rung — its
  no-memo deep recursion is impractical past depth ~18, the `.6` memoization is that fix) → **`CLEAN
  agree=242 diverge=0`**; `make -C rust parse_harness_equivalence_gate` → **4 passed**
  (`certified_grammars_are_byte_identical` now includes `ebnf`; the ratchet forced its promotion;
  `every_registered_grammar_is_classified_exactly_once`; the new `comment_arm_suppression_matrix_is_pinned`
  pins the per-grammar `(#, //, /*)` matrix against the shipped parsers — ebnf claims `/*`;
  regex/vhdl/sv/rtl_frontend claim `#`; semantic_annotation claims `//`+`/*`). Interpreter unit tests
  **7/7**.
- [x] **NO REGRESSION** — interpreter-tooling + an ADDITIVE `pub(crate)` query (the codegen EMIT path is not
  touched — codegen still uses its inline `grammar_claims_introducer_as_non_comment` calls; my additions are
  a new struct + free fn + a method + a `snake_to_pascal` visibility bump, none on the emit path). The
  equivalence gate re-proves json/semantic_annotation/rtl_frontend/vhdl/**systemverilog (sv_2017)**/scratch/
  regex/svpp STILL byte-identical (the shared predicate makes the interpreter match codegen for ALL grammars
  → the gating can only REDUCE divergence, never introduce it); interpreter unit tests **7/7**. **The
  strongest proof that the additive codegen change is emit-neutral:** `sv_cert_recognized_union_gate`
  REGENERATED `systemverilog_parser.rs` with the modified `ast_pipeline` and got the IDENTICAL cert —
  `recognized_basis_green: true`, canonical `UNKNOWN=12`, union `UNKNOWN=1`, residual
  `["context_member_method_call"]`, `sample_parse_failures=0`, deterministic byte-identical across seeds
  0/7/42. My commit stages no `generated/*` (git-ignored). New code clippy-clean; `mdbook_docs_gate` GREEN.
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` (CERTIFIED list gains `ebnf`; the
  per-introducer layout-arm suppression documented as the fidelity mechanism); `TOOLBOX.md` §1.6
  CERTIFIED/DEFERRED lists; this tree (`.5.2` done + this checklist, frontier, §15 map); CHANGES.md /
  DEVELOPMENT_NOTES.md / MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md.

## 18. PARSE-HARNESS.5.3 — return_annotation fidelity — Acceptance Checklist (enforced)

> Session #44. A CODE change: a SHARED codegen serialization primitive
> (`rust/src/ast_pipeline/unified_return_ast.rs` — a `serialize_with` sorted serializer on
> `UnifiedReturnAST::Object.properties`) + the interpreter/gate classification promotion
> (`rust/src/parse_harness_equivalence.rs`) + a new `.5.3` scouting probe. This is the FIRST `.5.x`
> leaf that touches codegen (all prior were interpreter-only), because the tools REFUTED the scouting
> log's "interpreter-only, mirror codegen's order" hypothesis: the divergence is a genuine shared
> serialization non-determinism, not an interpreter-side fold bug. Both affected generated parsers
> (`return_annotation`, `semantic_annotation`) were regenerated; no engine/grammar change.

- [x] **REPRODUCE / ISSUE** — `PGEN_PHEQ_ONLY=return_annotation cargo test … parse_harness_equivalence::measurement`
  (ladder [6,12,18], seeds 0/7/42) → `return_annotation DIVERGE samples=68 agree=50 diverge=8 (+10 suppressed)`.
  ALL 8 are `[Ast]` (both ACCEPT, same-length AST, different bytes); every diverging sample is a dotted
  property-access (`$1.S15`) or array-access (`$1[$1*]**`) — an LR-eliminated rule.
- [x] **ROOT CAUSE (WHY + WHERE)** — TOOLS-FIRST, and it REFUTED the scouting log's KEY-ORDER-to-mirror
  hypothesis. The new `.5.3` scouting probe (`probe_return_annotation_divergence`) dumped both exact
  `wrapper_specs` strings: the interpreter's `_pgen_lr_chain` `wrapper_specs` blob picks a DIFFERENT Object
  key order for the SAME rule across samples **in one run** (`$1.S15`→`{property,base,…}`,
  `$1[$1*]**`→`{type,…}`, `$1.S15K.vbn**`→`{property,type,…}`) — the fingerprint of std `HashMap`
  non-determinism, NOT a stable "codegen order". WHERE: the blob is `serde_json::to_string(&wrapper_specs)`
  at `rust/src/ast_pipeline/mod.rs:1943` (LR-elimination), where each `wrapper_specs[i].annotation_template`
  is a `UnifiedReturnAST::Object { properties: HashMap }` (`unified_return_ast.rs:56-58`). Codegen FROZE one
  arbitrary HashMap order into the generated parser (`generated/return_annotation_parser.rs:11473` — a
  string literal); the interpreter re-serialized a fresh (itself non-deterministic) order each `load_gen_ast`.
  So there is nothing for the interpreter to "mirror" — the two blobs are independent HashMap serializations
  of the same semantically-orderless content. Tool trail: gen-AST wrapper_specs (`--dump-gen-ast`) ≠ the
  generated literal ≠ the runtime output = three different orders; the oracle order was stable run-to-run
  (deterministic-only-because-frozen), the interpreter's was not.
- [x] **FIX** — fix-hierarchy tier = **shared codegen serialization primitive** (the lowest tier that can
  fix it — an interpreter-only fix is impossible when the oracle's own order is a non-canonical frozen
  artifact). A `#[serde(serialize_with = "serialize_properties_sorted")]` on `UnifiedReturnAST::Object.properties`
  emits keys in SORTED order at EVERY serialization site at once (the LR-elim gen-AST blob, codegen's frozen
  literal, and the interpreter's in-process re-serialization) → all sorted → byte-identical. Parser-agnostic
  (a shared type used by every grammar). Deserialization is unchanged (still into a `HashMap`; key order is
  semantically irrelevant — the blob is only ever deserialized back). Then promoted `return_annotation`
  DEFERRED→CERTIFIED in `parse_harness_equivalence.rs`.
- [x] **ADDRESSED (verified)** — before→after, re-runnable oracles: `PGEN_PHEQ_ONLY=return_annotation …::measurement`
  → `return_annotation DIVERGE 8` → **`CLEAN agree=68 diverge=0`**; `make -C rust parse_harness_equivalence_gate`
  → **4 passed** (`certified_grammars_are_byte_identical` now includes `return_annotation`; the ratchet forced
  its promotion, leaving only `rtl_const_expr` DEFERRED; `every_registered_grammar_is_classified_exactly_once`;
  `comment_arm_suppression_matrix_is_pinned` unaffected). Interpreter unit tests **7/7**; `unified_return_ast`
  serialization round-trip tests **27/27**. BONUS (latent bug closed): the generated `return_annotation_parser.rs`
  `wrapper_specs` is now **byte-identical across regens** (tool-verified: regen twice → `diff` empty), where it
  was previously a non-deterministic frozen HashMap order.
- [x] **NO REGRESSION** — the shared serialization change affects ONLY grammars whose LR-eliminated rules carry
  **Object** return-annotation templates: a repo-wide audit found exactly TWO (`return_annotation`,
  `semantic_annotation`); both were regenerated and both are byte-identical in the equivalence gate
  (`semantic_annotation` stays CERTIFIED — its sorted frozen blob matches the sorted interpreter blob). NO
  `ast_shape_contract` manifest pins a `wrapper_specs` blob (grep-verified), and the FINAL folded AST is
  unaffected (it is a `serde_json::Value` built by the fold, which already sorts). **Emit-neutrality proof for
  the shared codegen change on the big grammar:** `sv_cert_recognized_union_gate` REGENERATED
  `systemverilog_parser.rs` with the modified `ast_pipeline` → IDENTICAL cert (canonical `1343/10/1321/12`,
  union residual `["context_member_method_call"]`, `sample_parse_failures=0`, deterministic byte-identical
  seeds 0/7/42) — SV has no Object-template `wrapper_specs`, so its emit is untouched. New code clippy-clean
  (`unified_return_ast` / `parse_harness_equivalence` / `parse_harness_interpreter` — 0 findings; the
  generated-stage `eq_op` errors are pre-existing); `mdbook_docs_gate` GREEN. My commit stages no `generated/*`
  (git-ignored).
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` (CERTIFIED list gains `return_annotation`;
  the `wrapper_specs` canonicalization documented as the fidelity mechanism + the latent-non-determinism note);
  `TOOLBOX.md` §1.6 CERTIFIED/DEFERRED lists; this tree (`.5.3` done + this checklist, frontier, §15 map);
  CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md.

## 19. PARSE-HARNESS.5.5 — rtl_const_expr corpus — Acceptance Checklist (enforced)

> Session #45. A CODE change (tooling-only): edits `rust/src/parse_harness_equivalence.rs` — a general
> **curated-input-corpus** mechanism (`CURATED_CORPUS` table + `curated_corpus_for` + `build_corpus`
> integration) + a construct-complete `rtl_const_expr` curated list + the DEFERRED→CERTIFIED promotion.
> NO engine / grammar / codegen / generated-parser change (no `generated/*` regenerated). This is a
> **corpus problem, not an interpreter divergence** (the leaf's thesis) — VERIFIED by the differential
> being byte-identical on the curated corpus.

- [x] **REPRODUCE / ISSUE** — `PGEN_PHEQ_ONLY=rtl_const_expr cargo test … parse_harness_equivalence::measurement`
  (default ladder [6,12,18], seeds 0/7/42), THIS session → `rtl_const_expr DIVERGE samples=0 agree=0
  diverge=0`. The differential has ZERO corpus, so `is_clean()` is false (samples_total==0) and the grammar
  stays honestly DEFERRED — but for a NON-fidelity reason (no inputs), unlike every other `.5.x` leaf.
- [x] **ROOT CAUSE (WHY + WHERE)** — the stimuli generator cannot produce a usable corpus for
  `rtl_const_expr` within the bounded depth ladder, and the only depth window that generates at all is
  pathological + hang-adjacent. Tool-established via the CLI generation sweep (`ast_pipeline
  grammars/rtl_const_expr.ebnf --generate-stimuli --count 8 --seed 0 --max-depth D`, debug binary rebuilt
  with `ebnf_dual_run`): depths **6/12/16/18/20/24/28 → FAIL** rc=1 (`Error: Stimuli generation depth
  exceeded max_depth=N while expanding rule 'multiplicative_expr'` / `'identifier'`), depth **32 →
  succeeds but emits only pathologically huge expressions** (thousands of chars, giant binop chains —
  hostile to the no-memo interpreter), depths **40/60/80 → HANG** (25 s timeout; the known super-linear
  pathology). WHY: the grammar's precedence chain is ~16 rules deep (`rtl_const_expr → conditional_expr →
  logical_or_expr → … → multiplicative_expr → unary_expr → primary_expr → literal → decimal_integer →
  terminal`, `grammars/rtl_const_expr.ebnf:11-74`), so the generator needs `max_depth ≳ 30` just to bottom
  out to a leaf — and by then the 10 `(op X)*` quantifier levels each re-descend the whole chain,
  exploding super-linearly. WHERE: the corpus is generation-only in `build_corpus`
  (`rust/src/parse_harness_equivalence.rs`), which returns `[]` for `rtl_const_expr`. This is a
  CORPUS-CONSTRUCTION gap, not an interpreter fidelity bug (confirmed by ADDRESSED: the differential is
  byte-identical once a corpus exists).
- [x] **FIX** — fix-hierarchy tier = **new tooling / plumbing** (no engine / grammar / codegen / generated
  change): a general per-grammar **curated-input-corpus** mechanism — `CURATED_CORPUS: &[(&str, &[&str])]`
  (parser-agnostic, keyed by grammar name, next to CERTIFIED/DEFERRED as one source of truth) +
  `curated_corpus_for(name)` + `build_corpus` appending the curated inputs (deduped, with the same
  truncation-probe reject-path coverage as generated samples) — seeded with a construct-complete
  `rtl_const_expr` curated list (both literal kinds incl. underscores, plain/dotted/package-qualified
  identifiers, all four unary ops incl. nesting, every binary op at every one of the 10 precedence
  levels + multi-term chains + mixed precedence, ternary incl. nesting, parentheses, whitespace/trivia
  variety, and near-miss rejects). The differential compares the interpreter against the AUTHORITATIVE
  generated parser, so curated INPUTS carry no expected-output mirror risk (the oracle supplies the
  verdict + AST). Then promoted `rtl_const_expr` DEFERRED→CERTIFIED.
- [x] **ADDRESSED (verified)** — before→after, re-runnable oracles: `PGEN_PHEQ_ONLY=rtl_const_expr …::measurement`
  (default ladder [6,12,18], seeds 0/7/42) → `rtl_const_expr DIVERGE samples=0` → **`CLEAN samples=151
  agree=151 diverge=0`** (the interpreter is byte-identical to the generated parser over the curated
  corpus — verdict + typed AST, incl. the truncation reject-path probes); `make -C rust
  parse_harness_equivalence_gate` → **4 passed** (`certified_grammars_are_byte_identical` now includes
  `rtl_const_expr`; the ratchet forced its promotion, leaving DEFERRED empty;
  `every_registered_grammar_is_classified_exactly_once` holds — rtl_const_expr now CERTIFIED-only;
  `comment_arm_suppression_matrix_is_pinned` unaffected). Grounding cross-check (tool, not a regression of
  mine): `rtl_const_expr`'s own certificate-coverage still fully certifies at its tuned `--max-depth 32`
  window (`total=48 witness=48 UNKNOWN=0 fully_certified=true`, `sample_parse_failures=0`) — the same
  narrow depth band that motivates the curated corpus.
- [x] **NO REGRESSION** — the change is **purely additive test-only tooling** in
  `rust/src/parse_harness_equivalence.rs` (a `CURATED_CORPUS` table + `curated_corpus_for` + a
  `push_sample_with_probes` helper + the DEFERRED→CERTIFIED move); NEVER invoked by any parse/codegen/cert
  path. `git status` shows only `rust/src/parse_harness_equivalence.rs` + docs — **no `generated/*`
  regenerated** → the 10 previously-certified grammars + SV are byte-identical **by construction**, and
  the equivalence gate re-proves them so directly (`certified_grammars_are_byte_identical` **4 passed**,
  which re-runs the interpreter-vs-generated differential for `systemverilog` (sv_2017) and all other
  CERTIFIED grammars — a stronger, more targeted SV no-regression signal than the SV cert gate for a
  change that touches no SV/codegen surface). New code clippy-clean (`cargo clippy --lib --features
  "generated_parsers ebnf_dual_run"` — 0 findings in `parse_harness_equivalence`; the generated-stage
  `eq_op` errors are pre-existing, unchanged). `mdbook_docs_gate` GREEN (verified below).
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` (CERTIFIED list gains
  `rtl_const_expr`; DEFERRED now empty; a new *A curated corpus for un-generatable grammars* subsection
  documents the mechanism + the precedence-cascade root cause + the no-mirror-risk argument + the
  depth-32 cert-window note); `TOOLBOX.md` §1.6 CERTIFIED (11)/DEFERRED (empty) + the `.5.5` mechanism
  note; this tree (`.5.5` done + this checklist, frontier, §15 map); CHANGES.md / DEVELOPMENT_NOTES.md /
  MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md.

## 20. PARSE-HARNESS.6.1 — structural combinator isolating suite — Acceptance Checklist (enforced)

> ⚠️ LIVE-SPEC NOTE (2026-07-07, session #53): tool-established finding #1 below (bounded quantifiers
> `{N,M}` half-wired — codegen aborts `Unknown quantifier`) is **RESOLVED** by **`BOUNDED-QUANT.1`**
> (`docs/tasks/BOUNDED-QUANT.md`): the canonical `parse_quantifier_bounds` decoder now also accepts the
> frontend's brace-stripped raw-AST spelling, the stimuli generator's private duplicate decoder
> delegates to it, and the suite grew four `quant_bounded_*` isolating cases (16 → **20**, all CLEAN).
> The 16/16 numbers below are the honest historical record of `.6.1`'s landing.

> ⚠️ LIVE-SPEC NOTE (2026-08-08, session #215): the suite gained a 28th case,
> `recursion_guarded_memo_isolation`, from **`SV-CORPUS-GRAD.3.12`** — the packrat memo × RUNTIME
> CYCLE-BREAKING composition. ⭐ Worth recording as a coverage lesson for this tree specifically:
> that defect is a *structural* one, it lives in the engine both implementations share, and **no
> shipped-grammar corpus surfaced it** — the `.5` end-to-end gate was green on all 11 certified
> grammars throughout. It took an SV corpus burn-down leaf to find it, and it took a synthetic
> isolating grammar to pin it. The honest reading is that per-combinator coverage was incomplete in a
> dimension the enumeration did not name: the suite listed the CONSTRUCTS the interpreter dispatches,
> not the ENGINE MECHANISMS (memo, guard, speculation) those constructs run on top of, nor the
> compositions between them.

> Session #46, `PGEN-PARSE-HARNESS-0014`. A CODE change (adds `rust/src/parse_harness_combinator_suite.rs`,
> wires `rust/src/lib.rs`, adds the `parse_harness_combinator_gate` Makefile target). Capability build:
> "ROOT CAUSE" = the coverage gap that end-to-end `.5` equivalence leaves (only the shipped grammars'
> constructs), which per-combinator isolation closes; "ADDRESSED" = the interpreter proven byte-identical to
> the compile-and-run oracle on a systematic per-combinator suite. Test-only tooling — never invoked by any
> parse/codegen/cert path; NO `generated/*` regenerated (the 11 CERTIFIED grammars + SV byte-identical by
> construction).

- [x] **REPRODUCE / ISSUE** — the `.5` differential-equivalence gate certifies the interpreter byte-identical
  only over constructs the *shipped* grammars use (§3.3). To trust the interpreter on an ARBITRARY grammar
  (the harness's whole point — e.g. A2.3's `a | ab`), equivalence must be proven **per combinator** on small
  synthetic grammars. Before this leaf there was no such suite.
- [x] **ROOT CAUSE (WHY + WHERE)** — this is a capability build, so the tool-backed evidence is the empirical
  grounding of the suite (measure-then-lock, [[feedback_always_signoff_decisions]]): every isolating grammar
  was validated to compile through `ast_pipeline --generate-parser` before use, and the differential
  measurement (`measure_combinator_suite`) established the true oracle verdicts. That loop caught THREE
  tool-established facts (WHY+WHERE):
  - bounded quantifiers `{N,M}` fail codegen with `Unknown quantifier: 2` (`ast_pipeline <g> --generate-parser`;
    the frontend emits a `["quantifier","2"]` node, codegen `generate_quantified_logic` has no handler) → the
    compile-and-run oracle cannot build them → `?`/`*`/`+` are the covered forms;
  - the first LR grammar (bare direct `start := start "+" term | term`) is NOT structurally eliminated —
    `detect_left_recursive_chain_plan` / `extract_rule_reference_name` (`rust/src/ast_pipeline/mod.rs`) match
    only the **wrapper/indirect** form (a multi-element sequence alt is not a bare rule-ref), so direct LR
    falls to runtime cycle-breaking, where interp & oracle diverge on `furthest_position` (measured interp
    2/4 vs oracle 0) — a KNOWN out-of-scope gap (`DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE` + ignored probe);
  - the wrapper LR form IS eliminated (`4 _lr_base + 6 _lr_suffix` gen-AST nodes) but LR-elimination
    **prepends** the helpers to `rule_order`, so `rule_order[0]` ≠ the semantic entry `expr` → the case names
    the entry explicitly, applied identically to both sides.
- [x] **FIX** — fix-hierarchy tier = **new tooling / plumbing** (no engine / grammar / codegen / existing-runtime
  change): the new module (a `CombinatorCase` table of 16 isolating grammars + a report-first
  `run_combinator_case` / `evaluate_all_combinator_cases_on_large_stack` driver over `interpret_parse` vs
  `compile_and_parse`), `pub mod parse_harness_combinator_suite;` in `lib.rs` (`#[cfg(feature = "ebnf_dual_run")]`),
  and the `parse_harness_combinator_gate` Makefile target. Runs on a 512 MiB large-stack worker — the
  measurement first overflowed the default 2 MiB test stack on the LR case (the interpreter's logical
  recursion guard fires only past 2 MiB of real stack — [[feedback_recursion_ceiling_must_bound_the_real_stack]]),
  exactly as the `.5` gate anticipated.
- [x] **ADDRESSED (verified)** — before→after: the per-combinator capability now EXISTS. Re-runnable oracle =
  `make -C rust parse_harness_combinator_gate` → **2 gate tests pass**: `every_structural_combinator_is_byte_identical`
  (all **16/16** cases CLEAN — interp == compile-and-run oracle, byte-identical verdict + `furthest_position` +
  typed AST, every spec-reasoned anchor holding; folds in the A2.2/A2.3 discrimination proof: `longest_match`
  accepts `"ab"`, `ordered` rejects `"ab"`, both matching the real generated parser) and
  `combinator_coverage_is_complete` (every enumerated combinator has ≥1 case, names unique). Measurement map:
  `16/16 combinator cases CLEAN`.
- [x] **NO REGRESSION** — purely additive test-only tooling; NEVER invoked by any parse/codegen/cert path;
  `git status` shows only `rust/src/parse_harness_combinator_suite.rs` (new), `rust/src/lib.rs`, `rust/Makefile`
  (+ docs); **no `generated/*_parser.rs` regenerated** (mtimes unchanged) → the 11 CERTIFIED grammars + SV
  byte-identical by construction; the `.5` gate re-proves them byte-identical independently. New module
  clippy-clean (`cargo clippy --lib --tests --features "generated_parsers ebnf_dual_run"` — 0 findings in the
  module; the pre-existing generated-stage `eq_op`/naming lints are unchanged); `clippy_on_rust_change`
  strict-source GREEN; `mdbook_docs_gate` GREEN.
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` gains *The structural combinator suite*
  section (D5, SAME-COMMIT, to the depth of the sibling sections: what it is, the 16-case table, the
  completeness + A2.2/A2.3 discrimination invariants, and the two tool-established subtleties + the direct-LR
  out-of-scope note) + the interpreter honest-bound updated; `TOOLBOX.md` gains a combinator-suite gate entry;
  this tree (`.6.1` done + this checklist + frontier → `.6.2`); CHANGES.md / DEVELOPMENT_NOTES.md (+ the three
  novel findings surfaced for director feedback) / MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md updated.

## 21. PARSE-HARNESS.6.2 — semantic-directive orchestration: tool-mapped plan, committed scope, acceptance checklist

> Session #47. The plan below was established TOOLS-FIRST before any code: a minimal semantic grammar
> (`@fact_kind` + `@emit_fact` + `has_fact` post-`@predicate`) was pushed through the REAL codegen
> (`ast_pipeline … --generate-parser` — it compiles, 41 semantic-runtime call sites) and the emitted
> orchestration read line-by-line against the codegen templates. This section records the boundary and
> the design so the build is durable, not conversation-bound (same discipline as §13).
>
> ⚠️ LIVE-SPEC NOTE (2026-07-06, session #49): the suite this section landed at **20 cases** was
> extended + deliberately re-anchored by **`MEMO-STORE-SOUNDNESS.2`** — the memo × store composition
> is now TAINT-GATED (store-consulting bodies cached on neither memo side), `sem_memo_wrapper` pins
> the sound ACCEPT, and two success-side cases (`sem_memo_success_verdict`/`sem_memo_success_ast`)
> joined, making **22 cases**. The 20/20 numbers below are the honest historical record of `.6.2`'s
> landing; the live suite contract is documented in `docs/tasks/MEMO-STORE-SOUNDNESS.md` + the book's
> *The Parse Harness* chapter.

### 21.1 The shared-runtime vs generated-template boundary (tool-mapped)

The generated parser's semantic orchestration splits exactly like §13.1 predicted for the combinator half:

- **Section A — reuse VERBATIM (all `pub` in `crate::ast_pipeline::semantic_runtime`):**
  `compile_semantic_runtime_annotations(&Annotations)` (the in-process compiler — the SAME function
  codegen calls at `ast_based_generator.rs:6586` to freeze the compiled literal);
  `CompiledSemanticRuntimeAnnotations::{has_rule, is_empty, pre_predicates_for_rule,
  effect_directives_for_rule, post_predicates_for_rule, branch_predicates_for_rule,
  branch_predicates_for_rule_branch, branch_effect_directives_for_rule_branch, library_imports_for_rule,
  library_exports_for_rule, needs_raw_post_capture_for_rule, exportable_fact_kinds, clone_predicate_defs}`;
  `SemanticRuntimeState::{new, set_predicate_defs, push_rule_context, pop_rule_context, checkpoint,
  rollback_to_named, transaction_named, evaluate_directive_predicate, evaluate_content_aware_predicate,
  extract_delta_since, apply_delta}`; `SemanticRuntimeTransaction::{state, state_mut, apply_directive,
  commit}`; the spec types (`SemanticRuntimeDirective`, `SemanticPredicateSpec`, `SemanticPredicatePhase`
  (default `Pre`), `SemanticPredicateContentView` (default **`Raw`**), `SemanticFactSpec`,
  `SemanticScopeSpec`, `SemanticCloseScopeSpec`, `SemanticLibraryImportSpec/ExportSpec`); the builtin
  predicate vocabulary (`has_fact`, `lacks_fact`, `fact_attribute_equals`, `lacks_fact_attribute_equals`,
  `fact_count_at_least` — `semantic_runtime.rs:2031-2200`) + `@define_predicate` defs.
- **Section B — mirror from the EMITTED code (exist only as codegen `quote!` templates; authoritative
  reference = the emitted spike parser + `ast_based_generator.rs`):**
  - `with_semantic_runtime_rule_transaction` (`:1661`) — the rule-level orchestration skeleton: fast path
    (no annotations → push/pop rule context only); else push context → checkpoint → PRE predicates
    (`evaluate_directive_predicate`; `Some(false)` → `Backtrack{position: self.position}`) → body →
    `semantic_raw_content.unwrap_or(&node.content)` → effect directives → library imports → POST
    predicates (`resolve_semantic_predicate_spec_against_content` then `evaluate_content_aware_predicate`;
    `Some(false)` → `Backtrack{position: node.span.start}`) → library exports (vs
    `checkpoint.fact_len()`) → `transaction.commit()`; on any `Err` → `rollback_to_named(checkpoint)`;
    always `pop_rule_context` (exactly one pop per push).
  - `apply_semantic_runtime_effect_directive` (`:2010-2110`) — OpenScope/CloseScope/EmitFact with
    `$ref` resolution against `root_content = node.content`; Predicate/library/declaration → `Ok(false)`.
  - `apply_branch_start_effect_directive` (`:1225-1318`, INLINE-ACTIONS.2) — the WINNING branch's
    branch-start `@emit_fact`/`@open_scope`/`@close_scope` applied DIRECTLY to the live state (no
    transaction — the enclosing rule transaction + tournament checkpoint own rollback).
  - `apply_semantic_runtime_library_import_directive` / `_export_directive` (emitted `:415-561` in the
    spike) — filesystem-library I/O; **`library_in_dir`/`library_out_dir` are `None` unless set**
    (`parser_registry.rs:722` sets them from registry options; the compile-and-run throwaway main does
    NOT) → both sides no-op on missing dirs; resolution failures are `ContextualError`s.
  - The resolver family (`:2116-2360`, `:5534+`): `resolve_semantic_runtime_value_against_content`,
    `resolve_unified_semantic_value_against_content` (+`try_`), `resolve_unified_semantic_properties_…`,
    `resolve_semantic_predicate_spec_against_content` (+`try_` — an unresolvable `$ref` in a BRANCH
    predicate blocks that branch rather than erroring), `resolve_semantic_reference`
    (`$N` positional / named / dotted / `[N]` indexed / `.len`; SEMREF-SHAPED: a named ref against a
    `ParseContent::Json` walks the SHAPED object), `semantic_node_scalar`, `coerce_semantic_runtime_scalar`.
  - The multi-branch tournament semantic discipline (`:3230-3612`, C3-B): per-branch — branch-phase
    predicates (`branch_predicates_for_rule` ∪ `…_for_rule_branch`, evaluated against
    `(raw_content, transformed)`; blocked branch never takes) → `extract_delta_since(tournament_checkpoint)`
    → `rollback_to_named(tournament_checkpoint)` (no loser leakage); winner — `apply_delta(winner's delta)`
    → branch-start effects → `semantic_raw_content = best_raw_content` (captured only when
    `needs_raw_post_capture_for_rule`).
  - `memoized_call` (`:6440-6545`, PARSE-TERMINATION.6 split memo): the TRANSACTION WRAPS the memo —
    `with_semantic_runtime_rule_transaction(rule, |p| p.memoized_call(rule_id, body))` — so a rule's OWN
    gates/effects are NEVER cached (re-evaluated fresh on every memo hit); the memo stores the BODY's
    `(node, raw_semantic_content, end_pos, semantic_delta)` and replays the delta on hit; failures land in
    a lean `memo_fail` set keyed `(rule_id, position)` only.
- **Section C — the interpreter wiring:** compile the annotations once per parse
  (`compile_semantic_runtime_annotations`), `set_predicate_defs`, wrap `parse_rule_inner`'s body in the
  §B skeleton, thread `semantic_raw_content` through the Or tournament, mirror the C3-B discipline in
  `parse_or`, and (decision by measurement, §21.3) the split memo.

### 21.2 The isolating suite (module `rust/src/parse_harness_semantic_suite.rs`, gate `parse_harness_semantic_gate`)

Same architecture as `.6.1` (a static case table, report-first driver, curated inputs with
independently-reasoned anchors, differential vs the `.3` compile-and-run oracle, large-stack worker,
deterministic by construction). The LANDED table has **20 cases / 20 enumerated constructs** (each
verdict- or AST-changing so the differential is live); every grammar was validated through
`ast_pipeline --generate-parser` before locking:

1. `sem_post_gate` — `@emit_fact` + `has_fact` post gate (declare-then-use ACCEPT; use-undeclared REJECT).
2. `sem_pre_gate` — a `phase: pre` predicate blocking rule entry.
3. `sem_branch_gate` — the INLINE `phase: branch` predicate — pins the tool-established RULE-WIDE
   flattening (`branch_predicates_for_rule` flat-maps every branch bucket, `semantic_runtime.rs:735`).
4. `sem_branch_select` — branch-LOCAL selection via helper rules with post gates (the SV `.b.6.2.2`
   idiom): the store flips WHICH branch wins (AST-changing).
5. `sem_attr_gate` / 6. `sem_lacks_gate` / 7. `sem_count_gate` — the query vocabulary
   (`fact_attribute_equals` / `lacks_fact` / `fact_count_at_least`).
8. `sem_scope` — `@open_scope`/`@close_scope` + `has_fact_in_current_scope` (first coverage of scopes by
   any grammar; verdict-observable scope tree). 9. `sem_scope_is` — `current_scope_is` at rule entry.
10. `sem_rollback_loser` — C3-B: a SUCCESSFUL-but-losing branch's emission must not persist; the winner's
    must; a failed branch's rolls back (three-input discrimination).
11. `sem_zero_len_emit` — the zero-length-guard × store composition: a discarded zero-length iteration's
    rule-level emission PERSISTS (tool-established; pinned).
12. `sem_ref_raw_named` — named `$word` over RAW (no `->`) content — the recursive named-descendant walk.
13. `sem_ref_positional_unresolvable` — positional `$N` in a directive payload can NEVER resolve (the
    compiler strips `$`; the named lexer rejects a digit head) — hard-error parity pinned.
14. `sem_ref_shaped` — `view: shaped` dotted resolution against the `->` Json (SEMREF-SHAPED).
15. `sem_ref_len` — the `.len` suffix.
16. `sem_branch_start_emit` — INLINE-ACTIONS.2 winning-branch-only branch-start `@emit_fact`.
17. `sem_emit_attrs` — `@emit_fact` attributes resolved from `$ref`s + `fact_attribute_equals` on them.
18. `sem_library_noop` — `@export_to_library`/`@import_from_library` with no configured dirs (no-op parity).
19. `sem_memo_gate_retry` — transaction-wraps-memo: a gated rule re-tried at the same position after a
    zero-width store change re-evaluates its own gates FRESH (measured: parity, the sound side).
20. `sem_memo_wrapper` — the CONFIRMED genuine finding from the §21.1 scouting: an UNANNOTATED wrapper
    rule over a store-gated rule caches the composed failure in `memo_fail` keyed `(rule, position)` only;
    a same-position retry after a zero-width emission replays the STALE failure (oracle REJECTS `go!`
    where fresh evaluation would accept) — pinned on both sides via the interpreter's mirrored split memo.

The session-#47 measurement flow (measure-then-lock): the pre-mirror baseline was **1/18 CLEAN** (only
the library no-op — the interpreter was directive-blind, every gate case diverged interp-ACCEPT vs
oracle-REJECT); three oracle-side surprises were then root-caused with the toolbox (scratch-slot traces +
the frozen-literal reads — the quoted-String vs coerced-Identifier fact-name mismatch, the branch-gate
rule-wide flattening, the `$`-strip positional dead path); after the §21.1 orchestration mirror + split
memo landed, the suite went **20/20 CLEAN** with zero interpreter-vs-oracle divergence.

### 21.3 Committed scope + decision points (honest bounds)

- **IN `.6.2`:** the §21.1 Section B/C orchestration mirror (interpreter-tooling only — no engine /
  grammar / codegen-emit / generated-parser change); the §21.2 suite + measurement probes + gate; the
  book/TOOLBOX lockstep. Target: every suite case CLEAN (byte-identical verdict + `furthest_position` +
  typed AST) or an explicitly-classified KNOWN divergence (the direct-LR precedent).
- **Memoization (decision by measurement — RESOLVED, session #47):** `sem_memo_wrapper` CONFIRMED the
  observable memo effect (the oracle REJECTS `go!` — a stale `(rule, position)`-keyed failure replay —
  where fresh evaluation would accept), so the interpreter mirrors the split memo (memo + memo_fail +
  semantic-delta replay + raw-content carry; no coverage lane — the interpreter has no coverage) and is
  byte-identical INCLUDING the quirk; the staleness is surfaced to the director as a platform finding
  (memoization × store composition on the FAILURE side — the success side was closed by `.b.6.2.36.4`
  and is pinned sound by `sem_memo_gate_retry`). Bonus: the memo also removed the `.5`-documented
  no-memo interpreter slowness (the equivalence gate now runs in ~21 s).
- **OUT (honest bounds, stated per §3.4):** bootstrap facts (`push_fact_record` cross-file veer surface —
  the harness `ParseOutcome` API has no bootstrap-facts input); real library I/O through the throwaway
  oracle (the compile-and-run main sets no library dirs; the no-op path is pinned — real I/O stays proven
  by the registry-path SV gates); coverage-delta replay (coverage is a registry/cert surface the
  interpreter does not implement); `@define_predicate` composition beyond what the builtin vocabulary
  exercises (no shipped grammar uses it; noted for a follow-up if a suite case proves cheap).

### 21.4 PARSE-HARNESS.6.2 — Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the interpreter evaluated NO semantic directive (tool-verified: zero
  `evaluate_predicate`/`emit_fact`/`apply_directive` call sites in `parse_harness_interpreter.rs`); the
  §21.2 pre-mirror measurement enumerated the honest baseline: **1/18 CLEAN** — every verdict-changing
  gate case diverged interp-ACCEPT vs oracle-REJECT (e.g. `sem_post_gate "decl a;use b;"
  interp=true oracle=false`), only the library no-op agreed.
- [x] **ROOT CAUSE (WHY + WHERE)** — the §21.1 boundary: the orchestration exists only as codegen
  templates (`with_semantic_runtime_rule_transaction` @`ast_based_generator.rs:1661`, effects @`:2010`,
  branch-start @`:1225`, tournament C3-B @`:3230-3612`, memo @`:6440`, resolvers @`:2116`/`:5534`); the
  semantic half is callable runtime (Section A) — so the interpreter reuses A verbatim and mirrors B
  from the EMITTED code. The three oracle-side surprises were pinpointed with the toolbox BEFORE the
  mirror: the scratch-slot scoped trace (`PGEN_TRACE_VERBOSITY=debug … --trace-rules pick`) named the
  exact rejection — `🔍 has_fact(kind=mode, name=String("special")) → false` with the fact present
  (quoted-String arg vs coerced-`Identifier` fact name) and `🛡️ predicate 'has_fact' REJECTED branch
  2/2` (the rule-wide flattening, `branch_predicates_for_rule` flat-map @`semantic_runtime.rs:735-747`);
  the `sem_count_gate` full-trace showed `furthest_position=0` with `parse_word` = a bare
  `Err(Backtrack)` stub (my missing `word` rule + the no-preamble unresolved-stub fidelity fact); the
  frozen compiled literal showed `name: $2` → `RuleReference("2")` (the `$`-strip making positional
  refs unresolvable).
- [x] **FIX** — fix-hierarchy tier = **new interpreter tooling / plumbing** (no engine / grammar /
  codegen-emit / generated change): the §21.1 Section-B/C mirror in `parse_harness_interpreter.rs`
  (~+1300 lines: `with_rule_transaction`, `memoized_call` split memo, effect/branch-start appliers,
  the resolver family, the full tournament ladder with C3-B, raw-capture threading, the
  unresolved-stub dispatch reorder), the suite module `parse_harness_semantic_suite.rs` (20 cases),
  `lib.rs` wiring, two `pub(crate)` comparator helpers in the `.6.1` module, and the
  `parse_harness_semantic_gate` Makefile target.
- [x] **ADDRESSED (verified)** — before→after: **1/18 → 20/20 CLEAN**. Re-runnable oracle =
  `make -C rust parse_harness_semantic_gate` → **2 gate tests pass**
  (`every_semantic_construct_is_byte_identical` — 20/20 CLEAN, byte-identical verdict +
  `furthest_position` + typed AST vs the compile-and-run oracle; `semantic_construct_coverage_is_complete`).
  The `.4` interpreter unit tests still **7/7**.
- [x] **NO REGRESSION** — `parse_harness_equivalence_gate` → **4/4** (all **11 CERTIFIED grammars stay
  byte-identical** under the now-directive-AWARE + memoized interpreter — the sharpest signal, since
  SV's 45 `@predicate`s + 23 `@emit_fact`s are now actively evaluated where they were previously
  ignored; re-run and green again after the scratch-fixture restore; the gate also dropped to ~21 s —
  the memo closed the `.5`-documented no-memo slowness); `parse_harness_combinator_gate` → **16/16
  CLEAN, 2/2**; SV cert re-verified unchanged via `sv_cert_recognized_union_gate` at seeds **0/7/42** —
  `recognized_basis_green: true`, canonical `UNKNOWN=12`, union `UNKNOWN=1` witness `1332`, residual
  `["context_member_method_call"]`, `sample_parse_failures=0`, `unmet_criteria_count=0`, deterministic
  byte-identical; NO shipped `generated/*` regenerated (only the scratch slot was cycled for TRACING
  and restored from the blessed fixture); `clippy_on_rust_change` strict-source GREEN (0 findings in
  the three parse-harness modules; the 179 generated-stage `eq_op` errors are pre-existing, unchanged);
  `mdbook_docs_gate` GREEN.
- [x] **LOCKSTEP** — top-level mdBook `docs/book/src/parse-harness.md` gains *The semantic-directive
  orchestration suite* section (D5, SAME-COMMIT, to the depth of the sibling sections: what it is, the
  20-case table, the orchestration-mirror description, and the six pinned grammar-author facts) + the
  honest-bounds paragraph updated; `TOOLBOX.md` §1.8 entry + quick-chooser row; this tree (`.6.2` done,
  §21 plan + §21.2 landed table + §21.3 memo decision RESOLVED + this checklist, frontier);
  `docs/TASK_TREE.md` row refreshed (was stale at `.5.2`); CHANGES.md / DEVELOPMENT_NOTES.md (+ the six
  findings surfaced for director feedback) / MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md updated.

---

## 22. PARSE-HARNESS.10 — the `ast_pipeline` feature-surface tripwire (ops hardening) — plan + acceptance checklist

### 22.1 The trap (tool-recorded, three occurrences)

The compile-and-run oracle (`.3`, `compile_and_parse`) shells out to `target/debug/ast_pipeline`
for codegen, which MUST carry `ebnf_dual_run` to read a `.ebnf` directly. But several routine
builds legitimately produce a `--features generated_parsers`-only `ast_pipeline` at the SAME path
(`make focus_*` line `cargo build --features generated_parsers --bin ast_pipeline`; ad-hoc census
CLI builds), silently overwriting the dual-feature binary. The next parse-harness battery then
fails its 4 compile-and-run gates with a **generic** codegen error whose real cause (a stale
single-feature binary, not a behavior regression) must be re-diagnosed from the stderr plumbing
signature every time. Occurrences: session #140's first battery, the `-0118` battery, and the
`-0123` battery (4 gates failed; `RGX-0078.md` `.5.j.1` NO-REGRESSION log + `DEVELOPMENT_NOTES.md`
2026-07-18 entry, which queued this exact mechanization). MEMORY.md carries a standing manual
warning — "a warning that has to be remembered three times is a missing tripwire."

### 22.2 The fix (minimal, engine-tier, two seams)

1. **`ast_pipeline --report-feature-surface`** (`rust/src/main.rs`): a feature-INDEPENDENT probe
   flag handled BEFORE clap (so it needs no input file and answers identically in every feature
   configuration), printing one machine-readable marker line and exiting 0:
   `AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=<bool> generated_parsers=<bool>`.
   A binary that cannot answer the probe is, by definition, a pre-tripwire vintage — itself stale.
2. **The harness-side assertion** (`rust/src/parse_harness.rs`): `compile_and_parse` probes the
   resolved binary BEFORE any codegen/workdir work. Marker absent (probe unsupported) OR
   `ebnf_dual_run=false` ⇒ a new `HarnessError::StaleTool { path, detail }` whose Display carries
   the exact rebuild command (`cd rust && cargo build --features "generated_parsers ebnf_dual_run"
   --bin ast_pipeline`) and names the overwrite mechanism. The probe requires exactly
   `ebnf_dual_run` (what the harness's codegen step needs — the canonical `focus_*` regen path is
   itself an `ebnf_dual_run`-only build, so that surface is sufficient); the hint recommends the
   repo-standard dual-feature rebuild.

Single chokepoint: every consumer of the compile-and-run oracle (the combinator suite, the
semantic suite, the `.3` integration test, ad-hoc probes) flows through `compile_and_parse`, so
one assertion covers the whole gate battery. No caching — a mid-run binary swap (the exact trap
scenario) must be caught on the next call; the probe spawn is noise next to the per-call codegen +
rustc compile.

### 22.3 PARSE-HARNESS.10 — Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the #140-class trap, 3 recorded occurrences; latest: the `-0123`
  dual-feature battery failed 4 parse-harness gates with the generic plumbing signature
  `requires building with --features ebnf_dual_run` after a census-CLI single-feature build
  overwrote the binary (`RGX-0078.md` `.5.j.1` NO-REGRESSION log; `DEVELOPMENT_NOTES.md`
  2026-07-18 "single-feature-binary vintage trap fired a THIRD time").
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `compile_and_parse` (`rust/src/parse_harness.rs`)
  resolves `target/debug/ast_pipeline` by PATH only and discovers a feature-surface mismatch only
  deep inside codegen, as a generic `HarnessError::Codegen`; WHERE: the resolution site
  (`parse_harness.rs` `ast_pipeline_bin` resolution) has no capability check, and `ast_pipeline`
  offers no feature-surface introspection to check against (`main.rs` clap surface requires an
  input file for every invocation).
- [x] **FIX** — engine tier (a diagnostic/ops surface, no grammar involvement): the §22.2 probe
  flag + pre-codegen assertion + `StaleTool` error variant + unit tests (fake single-feature and
  pre-vintage binaries via `CompileAndParseOptions::ast_pipeline_bin`).
- [x] **ADDRESSED (verified)** — before: a single-feature binary yields the generic codegen error
  mid-gate (the `-0123` battery's 4-gate failure); after: `compile_and_parse` refuses it UP FRONT
  with the actionable message — unit tests
  `a_single_feature_binary_is_refused_with_the_actionable_message` +
  `a_pre_tripwire_vintage_binary_is_refused` GREEN (both assert `HarnessError::StaleTool` + the
  exact rebuild command in the message; `cargo test --lib parse_harness::tests` → `8 passed; 0
  failed`); live CLI demonstration on the rebuilt dual-feature binary:
  `./rust/target/debug/ast_pipeline --report-feature-surface` →
  `AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=true generated_parsers=true` (rc=0).
- [x] **NO REGRESSION** — the dual-feature parse-harness lib battery (`cargo test --features
  "generated_parsers ebnf_dual_run" --lib parse_harness`, memory-guarded `--budget-mb 16384`,
  peak 11102 MB): `28 passed; 0 failed; 8 ignored` (the 8 = the durable `--ignored` measurement
  scouts, by design) — the `-0123` baseline 25 + the 3 new tripwire tests, including all 4
  compile-and-run gates against the rebuilt probe-carrying binary; `mdbook_docs_gate` ✅; clippy
  source-strict via `clippy_on_rust_change` clean on the touched modules; no `generated/*`
  artifact change (codegen untouched — the flag is pre-clap in `main.rs` only; the probe lives
  harness-side).
- [x] **LOCKSTEP** — `TOOLBOX.md` §1.4 tripwire note; top-level book *The Parse Harness* chapter
  (compile-and-run section) tripwire paragraph (`mdbook_docs_gate` ✅); `DEVELOPMENT_NOTES.md`
  queued candidate marked DELIVERED; this tree (`.10` leaf + frontier + status); `docs/TASK_TREE.md`
  row; CHANGES.md / MEMORY.md.

---

## 23. PARSE-HARNESS.11 — the blessed scratch slot's OPERATING MANUAL is not part of the throwaway (`done`, `PGEN-PARSE-HARNESS-0004`, 2026-08-15 session #236, director-raised)

### 23.1 The incident (measured, this session, by the author of the change)

While preserving evidence for `ENGINE-UNIVERSAL-SERVICES.21` acceptance (g), the agent needed a
directly-left-recursive probe grammar in the slot. It wrote the whole of
`grammars/scratch/scratch.ebnf` — **deleting the 32-line header block** and replacing it with a
four-line probe. The director caught it immediately; nothing mechanical did, and nothing mechanical
*could*.

The header is not decoration. It is the only place that records:

| the header says | what is lost without it |
|---|---|
| `make -C rust SHELL=/bin/bash focus_scratch` | how to regenerate the slot at all |
| `parseability_probe --parse scratch <input>`, the trace/AST/lint/cert variants | how to drive the slot with the toolbox |
| ⛔ rebuild `ast_pipeline` **AFTER** `focus_scratch` | the `GENERATED-LINT-CORRECTNESS.13` ordering trap — a binary built by that run judges the PREVIOUS grammar (session #218 lost a session to it: `UNKNOWN=9`, every probe `parsed=false`, on a correct grammar) |
| `generated/scratch_parser.rs` / `scratch.json` are git-ignored; **THIS file is tracked** | that the slot is also the committed integration-test fixture |
| `git checkout grammars/scratch/scratch.ebnf` | how to restore |

### 23.2 ⛔⛔ WHY NO EXISTING GATE COULD SEE IT — the invisibility is STRUCTURAL, not an oversight

- **A commit-time gate cannot see it.** The correct probe workflow *ends* in
  `git checkout grammars/scratch/scratch.ebnf`. The slot is restored before the commit, so the
  commit shows **no diff on that path at all**. This is the same argument
  `scripts/preserve_scratch_probe.sh`'s own header makes for why *it* is a tool and not a gate.
- **The scratch integration test cannot see it.** It asserts the default fixture parses
  `"hello, world!"`. A header-less file carrying the identical body passes it.
- **`LIVE-DOC-CURRENCY` cannot see it.** That doctrine watches `.md` surfaces; this is an `.ebnf`
  fixture.

⇒ the loss window is open only while the probe is loaded, and only a check that runs **then** can
close it.

### 23.3 ⛔ THE REMEDY IS DELIBERATELY *NOT* ANOTHER SENTENCE IN THE HEADER

The director explicitly declined to add a "do not remove this header" banner and asked for the
engineer's view. Concurred, on the repository's own evidence: the header **already** said *"Edit the
grammar body below"*, inside a section literally headed `HOW TO USE`, and it was overwritten anyway.
`DOCTRINE_ENFORCEMENT.md` §1 — *a rule nothing checks is a suggestion* — makes the prediction
explicit: a second suggestion, in a file whose first suggestion had just been ignored, buys nothing.

### 23.4 The fix — one check script, three call sites, two tiers

`scripts/check_scratch_slot_header.sh` is the single source of truth (`DOCTRINE_ENFORCEMENT.md` §5).

| tier | invariant | called from | catches |
|---|---|---|---|
| **STRUCTURAL** (default) | a banner-delimited header block exists and still names all four operational **anchors** | `scripts/check_doctrines.sh` (doctrine `SCRATCH-SLOT-HEADER`, the 20th) → pre-commit + CI | a header removal that *is* committed |
| **`--probe-time`** | the above **plus** byte-identity with the committed header | the `$(SCRATCH_JSON)` Makefile rule; `preserve_scratch_probe.sh` | ⭐ **the actual incident** |

Four design points, each because the obvious version is worse:

1. **The `--probe-time` tier hangs off `$(SCRATCH_JSON)`, not off `focus_scratch`.** That rule
   depends on `$(SCRATCH_EBNF)`, so make re-runs it *exactly* when the slot has changed since the
   last generation — which is precisely the moment a destroyed header is still on disk and still
   recoverable. A phony prerequisite would either run always (noise) or never (useless).
2. **The committed header is DERIVED from `git show HEAD:`, never copied into the script.** A
   second copy of the manual is a second thing to drift, and this doctrine exists because the first
   copy was lost.
3. **The anchors are commands and paths, not prose phrases.** Enumerating wordings is what made
   `LIVE-DOC-CURRENCY`'s instrument B measure one population as 10, then 16, then 18 — every miss
   silent in the passing direction. And the structural tier deliberately does **not** diff against
   HEAD, so improving the manual stays free; only *probing* is held byte-identical.
4. ⭐ **`--restore-header` makes the refusal a one-command repair** that re-attaches the committed
   header above whatever body is loaded. A guard that only says NO invites a hand reconstruction,
   and a hand-reconstructed manual is exactly how a manual quietly loses a line.

⚠️ **Honest limit, stated rather than discovered later.** `--probe-time` is reachable only through
the two tracked entry points above. An author who invokes `ast_pipeline` on the slot directly gets
the structural tier at commit time and nothing sooner. That is a real gap and it is accepted: the
two wired paths are the ones every documented probe workflow goes through, and the header's own
`HOW TO USE` names `focus_scratch`.

### 23.5 PARSE-HARNESS.11 — Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — reproduced by command, not recalled: writing a four-line probe over
  the whole slot and running `make -C rust SHELL=/bin/bash focus_scratch` previously generated
  happily. `git diff --quiet -- grammars/scratch/scratch.ebnf` after the restore exits **0**, which
  is the proof that no commit-time gate can ever observe the loss. `bash scripts/check_doctrines.sh`
  on that tree: **ALL 19 PASS** — every doctrine green over a destroyed operating manual.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: the slot is a tracked file with two parts under **one**
  path and opposite lifecycles — a body that is *meant* to be overwritten and a header that must
  never be — and nothing distinguished them, so whole-file replacement (the natural way to load a
  probe) destroys the half that must survive. The loss is then unobservable because the workflow
  restores the path before any commit. WHERE, established by the ops/build-flow toolbox rather than
  asserted:

  ```
  $ git ls-files grammars/scratch/          # the slot IS tracked content — its header is too
  grammars/scratch/README.md
  grammars/scratch/scratch.ebnf
  $ git ls-files generated/ | wc -l         # …while everything it generates is not
  0
  $ git show HEAD:rust/Makefile | grep -A3 '^\$(SCRATCH_JSON):' | grep -c check_scratch_slot_header
  0                                          # BEFORE: the one rule that reads the slot guarded nothing
  $ make -n -C rust SHELL=/bin/bash focus_scratch     # AFTER: the guard is IN the recipe, and first
  bash ../scripts/check_scratch_slot_header.sh --probe-time
  ./target/ebnf_frontend_build/debug/ast_pipeline ../grammars/scratch/scratch.ebnf --emit-raw-ast-json ../generated/scratch.json
  ```

  ⇒ the two entry points that read the slot are `rust/Makefile`'s `$(SCRATCH_JSON)` rule and
  `scripts/preserve_scratch_probe.sh`, and neither looked at lines 1-32. Replaying the incident
  against the new guard names it exactly: `scratch-slot-header: grammars/scratch/scratch.ebnf has
  no banner-delimited header block.` → `make: *** [../generated/scratch.json] Error 1`.
  ⭐ `make -n` is also the reachability proof: the guard is not merely written, it is the FIRST
  thing the recipe runs.
- [x] **FIX** — fix-hierarchy tier = **ops/build-flow guard**; zero engine, grammar or generated
  bytes. New `scripts/check_scratch_slot_header.sh` (two tiers + `--restore-header` + `--self-test`),
  registered as the 20th doctrine `SCRATCH-SLOT-HEADER`, and wired `--probe-time` into the
  `$(SCRATCH_JSON)` rule and into `preserve_scratch_probe.sh` — the latter because that script tells
  the next reader to `cp <artifact> <slot>`, so preserving a header-less slot would bake the loss
  into tracked evidence and re-create it on every replay.
- [x] **ADDRESSED (verified)** — measured before→after, all arms fired. **The incident replayed:**
  whole-file overwrite → `make -C rust SHELL=/bin/bash focus_scratch` **exit 2**, refusing at the
  `$(SCRATCH_JSON)` rule with the repair command; `--restore-header` → the probe body is preserved
  (`grep -c 'expr := expr'` = 1) and the header is **byte-identical to HEAD**; re-run → **exit 0**
  and the parser generates (`expr_lr_base expr_lr_suffix`). **`--self-test`: 8 passed, 0 failed** —
  GREEN on the real slot, RED on a deleted header, RED on a banner with all four anchors gone, RED
  on **exactly one** anchor removed, RED on an empty slot, GREEN after repair, plus a positive check
  that the repair preserved the probe body. **`--probe-time` matrix:** body-only swap (the correct
  workflow) → 0; whole-file overwrite → 1; header reworded mid-probe → 1; same with
  `PGEN_SCRATCH_HEADER_EDIT=1` → 0. **preserve refuses a header-less slot** → rc 1 with **no
  artifact written**.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 20 enforced doctrines PASS**
  (19 + this one; the `<meta:mirror>` check holds `DOCTRINE_ENFORCEMENT.md` §10 equal to the
  registry). `bash scripts/preserve_scratch_probe.sh --self-test` → **4 passed, 0 failed**, unchanged
  by the new arm. The default fixture regenerates with 0 LR rules and the slot is byte-identical to
  HEAD (`git diff --quiet`). `make -C rust SHELL=/bin/bash mdbook_docs_gate` passes. Zero code,
  grammar and generated bytes.
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10 row (the 20th doctrine); `scripts/check_doctrines.sh`
  registry; the top-level book *The Parse Harness* chapter; this tree + `docs/TASK_TREE.md` row;
  `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md`.
- promotion: `docs/knowledge/a-two-lifecycle-file-needs-a-guard-at-the-boundary.md` **NEW** — the
  transferable shape is *one path, two lifecycles, no boundary*, and its companion insight that a
  loss which the correct workflow restores is invisible to every commit-time gate by construction.
