# PARSE-HARNESS — general arbitrary-grammar parse capability (the grammar-AST interpreter + compile-and-run + scratch-register), each made 100% trustworthy

- Tree ID: `PARSE-HARNESS`
- Status: `active` (created 2026-07-05, session #37, `PGEN-PARSE-HARNESS-0001`). `.1` DESIGN is this file; `.2` (scratch-register) **`done`** (`PGEN-PARSE-HARNESS-0002`, session #38); `.3` (compile-and-run) **`done`** (`PGEN-PARSE-HARNESS-0003`, session #39). **Phase A complete.** `.4` (interpreter core — the director's flagship) **`done`** (`PGEN-PARSE-HARNESS-0004`, session #40): byte-identical to the generated parser on the structural + return-annotation smoke set (json registry + 5 synthetic per-combinator grammars via the `.3` oracle); **Phase B is now OPEN.** Frontier → **`.5` (the differential-equivalence gate over the full corpus, all registered grammars, seeds 0/7/42 — the `.3` compile-and-run harness is its oracle) `not-started`**. See §13 for the `.4` tool-mapped plan, committed scope, and acceptance checklist.
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
| 1 | `PARSE-HARNESS.2` (approach 3 — scratch-register path) | `done` (`PGEN-PARSE-HARNESS-0002`, #38) | Phase A, first authoritative-by-construction harness; richest A2.3 evidence. |
| 2 | `PARSE-HARNESS.3` (approach 2 — compile-and-run) | `done` (`PGEN-PARSE-HARNESS-0003`, #39) | Phase A complete; the self-contained CI oracle for the `.5` equivalence gate; external-crate compile proven + integration test green. |
| 3 | `PARSE-HARNESS.4` (interpreter core — Phase B) | `done` (#40, `PGEN-PARSE-HARNESS-0004`) | The shared-core dynamic dispatcher over the gen-AST — the director's flagship. Byte-identical on the smoke set (json registry + 5 synthetic combinator grammars via the `.3` oracle); 7 tests pass. Tool-mapped plan + committed scope + acceptance checklist in §13. |
| 4 | `PARSE-HARNESS.5` (differential-equivalence gate) | `not-started` (**frontier**) | `parse_harness_equivalence_gate`: interpreter (`.4`) vs the generated parser byte-identical over the FULL corpus, all registered grammars, seeds 0/7/42 — the leg that makes (1) authoritative. The `.3` compile-and-run harness + the registry are its oracle; `interpret_parse_gen_ast` is its driver. |
| 5 | `PARSE-HARNESS.6`–`.7` (combinator suite + fuzz) | `not-started` | Phase B — combinator-complete coverage (incl. the deferred semantic-directive orchestration) + optional fuzz. |

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
