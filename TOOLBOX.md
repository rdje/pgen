# TOOLBOX.md — PGEN's diagnostic & debug toolbox (USE THIS FIRST)

> **STANDING DIRECTIVE (director, 2026-06-22, emphatic, non-negotiable).** PGEN ships a deep,
> self-explaining debug surface. For **any** `UNKNOWN`, rejected parse, hang, reach gap, or "why
> isn't this witnessed / why was this rejected" question, you **reach for these tools FIRST and
> systematically** — never eyeball a grammar, never guess a root cause, never offer a strategy menu
> before the toolbox has shown you the exact mechanism and source location. *"We are not progressing
> because [the toolbox is ignored]. This needs to stop for good — we have the toolbox, so use it."*
> If the existing tools cannot surface the WHY and WHERE, the next step is to **build** a tool, not
> to speculate.

This file is the **single, authoritative, meticulous catalog**. Each tool entry says **WHAT** it is,
**WHEN** to reach for it, **HOW** to run it (exact command), and **WHAT** the output looks like.

- Mirrored, navigable surfaces (kept in lockstep): the book chapters
  [`docs/book/src/diagnosing-unknowns.md`](docs/book/src/diagnosing-unknowns.md) (master index + the
  `UNKNOWN` protocol) and [`docs/book/src/parseability-probe-debug.md`](docs/book/src/parseability-probe-debug.md)
  (parser-level deep dive); the Knowledge-Map cards under `docs/knowledge/tool-*.md` (one per tool,
  retrieval-indexed by "when/how" questions); the standing decision
  [`docs/decisions/feedback_systematically_use_debug_toolbox.md`](docs/decisions/feedback_systematically_use_debug_toolbox.md).

## Enforcement — this is not optional, and not "trust me"

Using the toolbox is **mechanically enforced**, so it cannot be silently skipped:

- A **code change cannot commit** unless its owning task leaf (`docs/tasks/<TREE>.md`) carries BOTH
  (1) a tool-backed **DIAGNOSIS** (WHY+WHERE — a pasted `CERTIFICATE-COVERAGE:` line, a
  `[plannable-probe]` verdict, a `rejected by post predicate` trace, a `furthest_position=` error,
  or the exact toolbox command) **and** (2) a measured **VERIFICATION** (before→after metric,
  `REJECT→PASS`, determinism at seeds 0/7/42). Enforced by `scripts/check_diagnosis_evidence.sh`,
  run by the general doctrine enforcer `scripts/check_doctrines.sh` via `.githooks/pre-commit` (E3)
  and CI (E4). Demonstrated: a staged code change without that evidence is **blocked**.
- The project's **deterministic gates** (certificate-coverage at seeds 0/7/42, `ast_shape_contract`,
  syntax-closure) **re-run the real tools**, so any number you cite is independently re-verified —
  a fabricated final state does not reproduce and fails. That is the "not trust-me-bro" leg.
- The portable model for how every mechanizable doctrine is enforced is `DOCTRINE_ENFORCEMENT.md`
  (the "rule enforcer" framework: a check-script contract + a registry/driver + the E1→E4 wiring,
  adoptable by any project).
- Reminders fire at **session start** and **before every code edit** (`.claude/settings.json`
  `SessionStart` + `PreToolUse` hooks), so the toolbox-first rule is in context the moment it matters.

### The task-acceptance checklist (required for any code-change commit)

Every code-landing task leaf (`docs/tasks/<TREE>.md`) MUST carry this checklist, each box **ticked
`[x]`** and backed by the cited tool output. An **unticked or missing required box BLOCKS the
commit** (`scripts/check_diagnosis_evidence.sh`, run by the doctrine enforcer). Copy this into the leaf:

```markdown
## Acceptance Checklist (enforced)
- [ ] **REPRODUCE / ISSUE** — <tool command + symptom output proving the issue (parse REJECT, UNKNOWN=N, …)>
- [ ] **ROOT CAUSE (WHY + WHERE)** — <debug-tool output naming the mechanism + location: predicate rejection / [plannable-probe] verdict / furthest_position= + file:line or rule>
- [ ] **FIX** — <the minimal change + fix-hierarchy tier: declarative > grammar > engine>
- [ ] **ADDRESSED (verified)** — <before→after on the symptom: REJECT→PASS, UNKNOWN N→M>
- [ ] **NO REGRESSION** — <cert seeds 0/7/42 spf=0; the 6 fully-certified grammars byte-identical; external corpus 14/14; ast_shape_contract GREEN; clippy clean>
- [ ] **LOCKSTEP** — <book / contract / ledger / schema updated, or N/A + reason>
```

**Hard-gated (required, must be ticked + evidence-backed): ROOT CAUSE, ADDRESSED, NO REGRESSION** —
these are the director's named steps (analyse → root cause → addressed → no regression). REPRODUCE /
FIX / LOCKSTEP are part of the template and good practice, but not hard-blocked, to avoid
false-positives. The whole task-tree's "start→finish" is then the sequence of its leaves, each
passing this checklist, plus the tree's own Acceptance Criteria.

**A box is EARNED, not ticked.** A `[x]` you write is a *claim*; the proof is the **oracle re-run**.
The ADDRESSED and NO-REGRESSION boxes must cite a **named, re-runnable oracle** (the exact gate /
command + its deterministic result — e.g. cert-coverage at seeds 0/7/42, `ast_shape_contract_gate`,
the byte-identical check across the fully-certified grammars, the external corpus 14/14), so CI
re-executes exactly that and earns the box independently of your tick. A self-ticked-but-false box
passes the local presence check and **fails when the oracle is re-run** (locally via the `make`
gates, un-bypassably in CI). The local pre-commit hook is leg 1 (presence) — necessary, not
sufficient; the oracle re-run (`DOCTRINE_ENFORCEMENT.md` §6.1, leg 3) is what makes the box
un-self-tickable. Honest gap: that re-run is at CI, currently manual-only.

## The two binaries

| Binary | Build | Path | Used for |
|---|---|---|---|
| `parseability_probe` | `cargo build --release --features generated_parsers` (from `rust/`) | `./rust/target/release/parseability_probe` | parse a file, dump AST, trace, dashboards |
| `ast_pipeline` | `cargo build --features "generated_parsers ebnf_dual_run"` (from `rust/`, debug) | `./rust/target/debug/ast_pipeline` | certificate-coverage, lint, gen-AST, stimuli, `.ebnf`-direct |

`ast_pipeline` needs `--features ebnf_dual_run` to read a `.ebnf` directly, and `--features
generated_parsers` for certificate-coverage (it verifies witnesses through the real generated parser).

---

## Quick chooser — symptom → tool

| Symptom / question | Go to |
|---|---|
| "Does this file parse? Where does it fail?" | [1.1 `--parse`](#11---parse--supports) + [3.2 furthest-position](#32-furthest-position-error-diagnostic) |
| "What AST did it produce? Is the shape right?" | [1.2 `--parse-dump-ast-pretty`](#12---parse-dump-ast-pretty) |
| **"Parse an input against an ARBITRARY / synthetic grammar (not registered)?"** | [1.3 the `scratch` slot](#13-the-scratch-slot--drive-the-toolbox-on-an-arbitrary-grammar) (full CLI toolbox) · [1.4 compile-and-run](#14-the-compile-and-run-harness--parse-an-arbitrary-grammar-with-no-registry-edit--no-pgen-rebuild) (in-process, authoritative by construction) · [1.5 the interpreter](#15-the-grammar-ast-interpreter--parse-an-arbitrary-grammar-in-process-with-no-codegen--no-compile) (in-process, NO compile) |
| **"Which alternative WINS this choice on this input? Is this branch LIVE or dead?"** | [Protocol D](#protocol-d--which-alternative-wins-this-choice--is-this-branch-live-the-a22a23-class-probe) — scratch slot + the `🏁 selected branch N/M` trace line + the 1.7 policy matrix |
| **"Is the interpreter byte-identical to the generated parser? which input diverges?"** | [1.6 the differential-equivalence gate](#16-the-differential-equivalence-gate--is-the-interpreter-byte-identical-to-the-generated-parser) |
| **"Is the interpreter byte-identical PER COMBINATOR (on a synthetic grammar, in isolation)?"** | [1.7 the structural combinator suite](#17-the-structural-combinator-suite--is-the-interpreter-byte-identical-per-combinator) |
| **"Is the interpreter byte-identical on the SEMANTIC-DIRECTIVE surface (`@predicate`/`@emit_fact`/scope/rollback/memo×store)?"** | [1.8 the semantic-directive orchestration suite](#18-the-semantic-directive-orchestration-suite--is-the-interpreter-byte-identical-on-the-store-gated-surface) |
| "A `@predicate` rejected valid input — which one, why?" | [2.4 predicate self-explaining trace](#24-predicate-self-explaining-trace) |
| "The parse is slow / stuck — which rules dominate?" | [3.1 `--dump-rule-call-counts`](#31---dump-rule-call-counts) |
| "I need to watch the parser step by step" | [2.1 trace verbosity](#21-trace-verbosity) + [2.2 `--trace-rules`](#22---trace-rules) |
| "How trustworthy is this grammar? proof/witness/UNKNOWN?" | [4.1 `--report-certificate-coverage`](#41---report-certificate-coverage) |
| "Give me ALL the UNKNOWN rules" | [4.2 `PGEN_CERT_COVERAGE_DUMP_ALL`](#42-pgen_cert_coverage_dump_all) |
| **"WHY is this rule UNKNOWN / not witnessed?"** | [4.3 `PGEN_CERT_COVERAGE_DEBUG_PROBES`](#43-pgen_cert_coverage_debug_probes) → [the 3-step protocol](#protocol-a-diagnose-an-unknown-3-steps) |
| "Which path did the witness planner take?" | [4.4 `PGEN_REACH_PATH_DUMP`](#44-pgen_reach_path_dump) |
| "Which residual `UNKNOWN`s are profile-excluded by construction?" | [4.6 `PGEN_CERT_RESIDUAL_CLASSIFICATION`](#46-pgen_cert_residual_classification) |
| "Is my grammar well-formed (LR / shadowing / non-terminating)?" | [5.1 `--lint-grammar`](#51---lint-grammar) |
| "What IR do the generators actually consume?" | [5.2 `--dump-gen-ast`](#52---dump-gen-ast) |
| "Packrat memo hit/miss perf?" | [3.3 `PGEN_REPORT_MEMO_STATS`](#33-pgen_report_memo_stats) |
| "EXACT per-rule entry counts for a parse (machine-readable)?" | [3.4 `--dump-rule-entry-counts-json`](#34---dump-rule-entry-counts-json) |
| "How much parse work is DISCARDED (failed speculation)? committed vs wasted per rule?" | [3.5 `--dump-rule-outcome-counts-json`](#35---dump-rule-outcome-counts-json) |
| "Which rules could a derived DFA scanner fuse? the measured ceiling? the choice-site / merged-choice surface?" | [5.3 `--report-fusibility-census`](#53---report-fusibility-census) |

---

## 1. Parse & inspect (`parseability_probe`)

### 1.1 `--parse` / `--supports`
- **WHAT:** parse a file with a registered grammar; exit `0` on a fully-consuming parse, non-zero on rejection. `--supports` just checks a grammar is registered.
- **WHEN:** the first thing to run on any "does X parse / where does it fail" question; the minimal reproducer check after you isolate a construct.
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --supports systemverilog
  ./rust/target/release/parseability_probe --parse systemverilog file.sv --profile sv_2017
  ```
- **OUTPUT:** `parse_full passed for grammar '…' on '…'` (rc 0), or an error line carrying both the surface position and the `furthest_position` (see 3.2).
- **Profiles:** SV accepts `2017`/`sv_2017`/`ieee1800-2017` and the `2023`/`1364-2005` equivalents — the request spellings are GRAMMAR-DECLARED via `@profile_alias` in `systemverilog.ebnf` (PROFILE-ALIAS.2; the generated parser carries them in `GRAMMAR_PROFILE_ALIASES` and resolves them case-insensitively in `set_grammar_profile`).

### 1.2 `--parse-dump-ast-pretty`
- **WHAT:** parse + write the typed AST as pretty JSON (`--parse-dump-ast` for compact).
- **WHEN:** "did it parse the *right* shape?", verifying a return-annotation `{kind,…}`, or confirming a fact-gated construct realized (e.g. `grep -c context_member_method out.json`).
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog file.sv out.json --profile sv_2017
  ```
- **OUTPUT:** AST JSON at `out.json` (default `<grammar>_ast.json`). Bound size with `--max-bytes N` / `PGEN_PARSE_DUMP_AST_MAX_BYTES`.
- **ENTRY-RELATIVE (`--entry-rule RULE`, SV-AST-SHAPE-FIDELITY.2.4):** dump the AST parsed from an ALTERNATE start symbol via `parse_full_from`, so a rule that is **PEG-shadowed** or entry-relative (unreachable from the canonical entry) can be inspected in isolation. Essential for the `<invalid_sequence_access>` return-annotation corruption class: `parseability_probe --parse-dump-ast-pretty systemverilog scoped.sv out.json --profile sv_2017 --entry-rule interface_class_type` reveals the sentinel a canonical-entry parse would never reach. Wired for `systemverilog` and `scratch`; omit for the byte-identical canonical dump. (Empty store ⇒ only non-store-gated scoped branches — `this.` / `pkg::` — fire in isolation.)

### 1.3 The `scratch` slot — drive the toolbox on an ARBITRARY grammar
- **WHAT:** a blessed, throwaway registered grammar (`grammar_name` = `scratch`) whose body you overwrite freely, so an **arbitrary / synthetic grammar** becomes a first-class registered parser drivable by the WHOLE toolbox above (`--parse`, `--parse-dump-ast-pretty`, `--trace-rules`, `--report-certificate-coverage`, `--lint-grammar`) — with semantics identical to the shipped generated parsers (authoritative **by construction**). This is the tool for the "which alternative wins on this input?" / linter-soundness / grammar-authoring probe (the `--build-a-tool` answer to the arbitrary-grammar capability gap). PARSE-HARNESS approach 3.
- **WHEN:** you need to parse a one-off / made-up grammar that is NOT registered — e.g. "does `r := \"a\" | \"a\" \"b\"` on `\"ab\"` pick the first or the longest alt?" (the exact `A2.3`-style evidence).
- **HOW:**
  ```bash
  # 1. Edit grammars/scratch/scratch.ebnf (keep the entry rule named `scratch`), e.g.:
  #        scratch := "a" | "a" "b"
  make -C rust SHELL=/bin/bash focus_scratch                       # regen the scratch parser artifact
  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)
  printf 'ab' > /tmp/in.txt
  ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt
  ./rust/target/release/parseability_probe --parse-dump-ast-pretty scratch /tmp/in.txt /tmp/out.json
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt --trace-rules scratch
  ./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf --lint-grammar
  ```
- **OUTPUT:** exactly as for any registered grammar (rc 0 + `parse_full passed`, the typed AST, branch-entry trace, `furthest_position` on reject). Restore the default fixture (`git checkout grammars/scratch/scratch.ebnf`) before committing. Full design + trust architecture: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md`.

### 1.4 The compile-and-run harness — parse an ARBITRARY grammar with NO registry edit / NO `pgen` rebuild
- **WHAT:** `pgen::parse_harness::compile_and_parse(grammar_ebnf, input, &opts) -> ParseOutcome` — runs the REAL codegen on an arbitrary `.ebnf`, compiles the emitted parser as a **throwaway external crate**, runs it, and returns `{accepted, furthest_position, error, ast_json}`. Authoritative **BY CONSTRUCTION** (the shipped codegen + runtime); self-contained (touches neither the registry nor `pgen`). PARSE-HARNESS approach 2 — the in-process API a gate/oracle can call (vs the scratch slot 1.3, which is the CLI-toolbox path). It IS the CI oracle for the `.5` differential-equivalence gate.
- **WHEN:** you need the verdict + typed AST for a synthetic grammar **programmatically** (a Rust test/gate), without the scratch slot's `focus_scratch` + probe-rebuild ceremony; or to cross-check the scratch slot / interpreter.
- **HOW (Rust):**
  ```rust
  use pgen::parse_harness::{compile_and_parse, CompileAndParseOptions};
  let out = compile_and_parse(
      std::path::Path::new("grammars/scratch/scratch.ebnf"),
      "hello, world!",
      &CompileAndParseOptions::default(), // .entry_rule = alternate start; .workdir = reuse to keep pgen cached
  )?;
  assert!(out.accepted);              // out.furthest_position / out.error on reject; out.ast_json (typed AST) on accept
  ```
- **OUTPUT:** a `ParseOutcome` whose `ast_json` is **byte-identical** to `parser_registry::parse_sample_ast_json` for a registered grammar (pinned by the integration test `parse_harness::tests::compile_and_run_harness_reproduces_json_registry_verdict_and_ast`). Needs an `ast_pipeline` binary built with `--features ebnf_dual_run` (the standard `target/debug/ast_pipeline`). First probe compiles `pgen` as a dep (~seconds→minutes cold); reuse `opts.workdir` to keep it warm. Full design: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md`.

### 1.5 The grammar-AST interpreter — parse an ARBITRARY grammar IN-PROCESS with NO codegen / NO compile
- **WHAT:** `pgen::parse_harness_interpreter::interpret_parse(grammar_ebnf, input, &opts) -> ParseOutcome` — an in-process **interpreter** that dynamically dispatches over the normalized gen-AST (the `--dump-gen-ast` IR) reusing the shipped `ParseNode`/`ParseContent`/semantic runtime, so it needs **neither codegen nor a compile**. Same `ParseOutcome{accepted, furthest_position, error, ast_json}` as `compile_and_parse` (1.4), so the two are directly diffable. Authoritative **BY VERIFICATION** — a second implementation whose thin dispatch layer is proven byte-identical to the generated parser by a differential oracle. PARSE-HARNESS approach 1 (`.4` core). The fast path for "which alternative wins on this input?" / grammar-authoring / linter-soundness probes when you do NOT want the per-probe `rustc` compile of 1.4.
- **WHEN:** you need the verdict + typed AST for a synthetic grammar **fast and in-process** (a Rust test/gate/REPL-style probe), and the grammar is on the `.4` structural + return-annotation surface (no store-gated `@predicate`/`@emit_fact` parse outcomes — those are the `.5`/`.6` extension). Cross-checks the scratch slot / compile-and-run harness with no compile.
- **HOW (Rust):**
  ```rust
  use pgen::parse_harness_interpreter::{interpret_parse, InterpretOptions};
  let out = interpret_parse(
      std::path::Path::new("grammars/json.ebnf"),
      r#"{"a": [1, true, null]}"#,
      &InterpretOptions::default(), // .entry_rule = Some("rule") for an alternate start symbol
  )?;
  assert!(out.accepted);            // out.furthest_position / out.error on reject; out.ast_json on accept
  // feature-independent core (already-normalized gen-AST): interpret_parse_gen_ast(tree, order, anns, entry, input)
  ```
- **OUTPUT:** a `ParseOutcome` whose `ast_json` is **byte-identical** to `parser_registry::parse_sample_ast_json` for a registered grammar (pinned by `parse_harness_interpreter::tests::interpreter_is_byte_identical_to_the_json_registry_parser`) and to `compile_and_parse` (1.4) on synthetic grammars. Needs `--features ebnf_dual_run` (the `.ebnf` frontend). No `rustc` per probe → the fast oracle-side of the `.5` differential-equivalence gate. Full design: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md` §13.

### 1.6 The differential-equivalence gate — is the interpreter byte-identical to the generated parser?
- **WHAT:** `make -C rust SHELL=/bin/bash parse_harness_equivalence_gate` (module `rust/src/parse_harness_equivalence.rs`) — the certifying oracle for the interpreter (1.5). For each registered grammar it runs the interpreter AND the shipped generated parser over ONE deterministic stimuli corpus (seeds 0/7/42, bounded generation, large-stack workers) and asserts **byte-identical** verdict + typed AST. Report-first API (`evaluate_grammar_equivalence` collects divergences, never panics) + 3 enforcing gate tests. PARSE-HARNESS.5.
- **WHEN:** after ANY interpreter change (regression guard); to MEASURE whether the interpreter matches the generated parser on a grammar; to see WHICH input first diverges and WHERE the two ASTs differ (the divergence names the grammar + sample + byte offset). The scouting probes (`cargo test … parse_harness_equivalence::measurement -- --ignored --nocapture`) print the full per-grammar map + a regex-divergence minimizer.
- **HOW:**
  ```bash
  make -C rust SHELL=/bin/bash parse_harness_equivalence_gate
  # measurement / scouting (per-grammar CLEAN/DIVERGE + first divergence):
  cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_equivalence::measurement::measure_equivalence_across_registered_grammars -- --ignored --nocapture
  # isolate one grammar under a shell timeout (no-memo interpreter can be slow on deep samples):
  PGEN_PHEQ_ONLY=regex cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_equivalence::measurement::measure_equivalence_across_registered_grammars -- --ignored --nocapture
  ```
- **OUTPUT:** the gate tests pass/fail (`certified_grammars_are_byte_identical`, `deferred_grammars_are_still_divergent_or_promote_them`, `every_registered_grammar_is_classified_exactly_once`, `comment_arm_suppression_matrix_is_pinned`). CERTIFIED (byte-identical, **11**): json, semantic_annotation, rtl_frontend, vhdl, systemverilog (sv_2017), scratch, **regex** + **systemverilog_preprocessor** (promoted by `.5.1`, session #42), **ebnf** (promoted by `.5.2`, session #43), **return_annotation** (promoted by `.5.3`, session #44), **rtl_const_expr** (promoted by `.5.5`, session #45 — a CORPUS fix, not a fidelity fix: a curated-input corpus, since its ~16-level precedence cascade is un-generatable within the bounded ladder). DEFERRED: *(empty — all promoted; the ratchet stays wired for a future divergent grammar)*. EXCLUDED (oracle is not own-grammar codegen): builtin_return_annotation, builtin_semantic_annotation. The `.5.1` regex fidelity closure mirrored four codegen decisions in the interpreter — whitespace-sensitive layout policy / unresolved-reference built-ins (`builtin_any_char`/`builtin_ascii_char`) / `@transform` numeric coercion + the PCRE2 post-parse contract / `@profiles` dialect gating — general primitives that also closed svpp. The `.5.2` ebnf closure added a fifth: per-introducer **comment-arm suppression** — the interpreter's layout skippers gate the `#`/`//`/`/*` comment arms via codegen's OWN predicate (`comment_arm_suppression_for_grammar`), so a grammar that claims an introducer as a real token (ebnf's `block_comment := "/*" …`) is matched structurally, not skipped as trivia. The `.5.3` return_annotation closure added a sixth (a SHARED codegen fix, not interpreter-only): the `_pgen_lr_chain` `wrapper_specs` blob (the serialized per-alt `annotation_template`s of an LR-eliminated rule) was serialized from a non-deterministic std `HashMap` (`UnifiedReturnAST::Object`) — codegen froze one arbitrary order, the interpreter re-serialized a fresh non-deterministic order each load; a `serialize_with` SORTED serializer on `UnifiedReturnAST::Object.properties` canonicalizes every site (also closing a latent codegen non-determinism). The `.5.5` rtl_const_expr closure is NOT a fidelity fix but a CORPUS fix: the stimuli generator yields zero usable samples for its ~16-level precedence cascade within the bounded ladder (depths ≤28 fail `depth exceeded`, ~32 emits pathologically-huge expressions, ≥40 hang), so a general parser-agnostic **curated-input corpus** (`CURATED_CORPUS`, keyed by grammar name) gives the differential inputs — the generated parser still supplies the authoritative verdict+AST, so curated INPUTS carry no mirror risk; the interpreter is byte-identical over it (151 samples), confirming a corpus gap, not a divergence. Deep-stress scouting: `PGEN_PHEQ_ONLY=<g> … parse_harness_equivalence::measurement::probe_layout_deep_stress -- --ignored --nocapture` (ladder 6-30, 5 seeds; covers regex/svpp/ebnf); the `.5.3` byte-level scout is `probe_return_annotation_divergence`. Full map: book chapter *The Parse Harness* → *The differential-equivalence gate* + `docs/tasks/PARSE-HARNESS.md` §14/§15/§16/§17/§18.

### 1.7 The structural combinator suite — is the interpreter byte-identical PER COMBINATOR?
- **WHAT:** `make -C rust SHELL=/bin/bash parse_harness_combinator_gate` (module `rust/src/parse_harness_combinator_suite.rs`) — the per-combinator differential (PARSE-HARNESS.6.1). The `.5` gate (1.6) proves byte-identity only over constructs the SHIPPED grammars use; this suite proves it **per structural combinator** on **27 small synthetic isolating grammars** (one per construct the `.4` interpreter dispatches), each run through BOTH the interpreter (1.5) and the compile-and-run oracle (1.4) and asserted byte-identical (verdict + `furthest_position` + typed AST). This is the coverage that upgrades the claim from "trusted on the shipped grammars" to "trusted on ANY grammar built from PGEN's structural constructs" — the answer to the "which alternative wins on this synthetic input?" / A2.2/A2.3 linter-soundness question.
- **WHEN:** after ANY interpreter change (regression guard for the combinator surface); to prove/measure the interpreter on a *specific* structural combinator (choice under each `branch_policy`, `?`/`*`/`+` incl. zero-length guard, the four bounded forms `{N}`/`{N,M}`/`{N,}`/`{,M}`, lookahead `&`/`!`, sequence-backtrack, atom/regex-token, rule-ref, LR-eliminated, the three `@whitespace_sensitive` layout-policy shapes — insensitive default / `true` / `{ regex_tokens: true }`, WS-DIRECTIVE.2 — the two `@default_profile` shapes — permissive unspecified default / declared-default gate, DEFAULT-PROFILE.2 — and the two `@profile_alias` shapes — declared-spelling resolution / unknown-spelling pass-through, PROFILE-ALIAS.2, driven end-to-end via each case's `requested_profile`); to see the A2.2/A2.3 discrimination directly (longest_match accepts `a|ab` on `"ab"`, ordered rejects it — on both implementations, in agreement).
- **HOW:**
  ```bash
  make -C rust SHELL=/bin/bash parse_harness_combinator_gate
  # scouting: the full per-case, per-input interp-vs-oracle map
  cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_combinator_suite::measurement::measure_combinator_suite -- --ignored --nocapture
  # the KNOWN direct-LR runtime-cycle-breaking furthest_position divergence (out of .6.1 scope):
  cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_combinator_suite::measurement::measure_direct_left_recursion_known_divergence -- --ignored --nocapture
  ```
- **OUTPUT:** 2 gate tests pass (`every_structural_combinator_is_byte_identical` — 27/27 CLEAN + the folded-in A2.2/A2.3 discrimination proof; `combinator_coverage_is_complete`). Deterministic by construction (fixed grammars × curated inputs, no seeds); runs on a 512 MiB large-stack worker (the interpreter's logical recursion guard fires only past the 2 MiB default stack on the LR case). Bounded quantifiers `{N}`/`{N,M}`/`{N,}`/`{,M}` are covered first-class by the four `quant_bounded_*` cases since **BOUNDED-QUANT.1** closed the codegen half-wire (the canonical `parse_quantifier_bounds` decoder now also accepts the frontend's brace-stripped raw-AST spelling `"2,3"`; previously codegen aborted `Unknown quantifier` while stimuli generation accepted the same grammar — a generator⟷parser duality break). The grammar-level `@whitespace_sensitive` LAYOUT policy is covered first-class by the three `layout_*` cases since **WS-DIRECTIVE.2** replaced the grammar-NAME layout gate with the declarative directive (insensitive default / full `true` / granular `{ regex_tokens: true }` — so a whitespace-sensitive SYNTHETIC/scratch grammar is now expressible and probe-able). The grammar-level `@default_profile` DEFAULT-DIALECT policy is covered first-class by the `profile_unspecified_permissive`/`profile_default_gate` pair since **DEFAULT-PROFILE.2** replaced the regex→`pcre2` name literals with the declarative directive (no directive = permissive unspecified profile / directive = the declared default gates `@profiles` rules — so a profiled SYNTHETIC/scratch grammar can now declare its default and be probe-able). The grammar-level `@profile_alias` REQUEST-SPELLING policy is covered first-class by the `profile_alias_resolves`/`profile_alias_unknown_passthrough` pair since **PROFILE-ALIAS.2** replaced the engine alias tables (`parser_registry.rs` `"systemverilog"` arm + the global `main.rs` spelling table) with the declarative directive (a declared spelling resolves to its canonical profile / an undeclared spelling passes through un-coerced — driven end-to-end through each case's `requested_profile`, which both the interpreter and the compile-and-run oracle apply identically). Honest bound (no silent caps): bare DIRECT left-recursion is a KNOWN `furthest_position` divergence on the runtime-cycle-breaking path (a durable `--ignored` probe). Full map: book chapter *The Parse Harness* → *The structural combinator suite* + `docs/tasks/PARSE-HARNESS.md` §20 + `docs/tasks/BOUNDED-QUANT.md` + `docs/tasks/WS-DIRECTIVE.md` + `docs/tasks/DEFAULT-PROFILE.md` + `docs/tasks/PROFILE-ALIAS.md`.

### 1.8 The semantic-directive orchestration suite — is the interpreter byte-identical on the STORE-GATED surface?
- **WHAT:** `make -C rust SHELL=/bin/bash parse_harness_semantic_gate` (module `rust/src/parse_harness_semantic_suite.rs`) — the `.6.1` sibling for the **semantic-directive orchestration** surface (PARSE-HARNESS.6.2): **36 isolating grammars**, one or more per store-gated construct — `@predicate` gates in every phase (`pre`/`branch`/`post`, hit AND miss, plus **`final`** — the FINAL-PHASE-PREDICATE.2 whole-input deferred-obligation gate: a legal forward reference ACCEPTs, an undefined reference REJECTs at parse completion, and a losing tournament branch's obligation is rolled back; plus **the FINAL-PHASE-PREDICATE.3 multi-branch shaped-key case** — a `phase: final` shaped-key `$ref` on an `Or` rule under the DEFAULT `view: raw` resolves via the other-view fallback), `@emit_fact` + the full builtin query vocabulary (`has_fact`/`lacks_fact`/`fact_attribute_equals`/`fact_count_at_least`/`has_fact_in_current_scope`/`current_scope_is`), the scope tree (`@open_scope`/`@close_scope`), C3-B tournament rollback (a successful-but-losing branch's emissions must not persist), zero-length-success emission, `$reference` resolution (named raw-tree walk / shaped view / `.len` / the positional-`$N` hard-error parity / **positional `$N` under a `->` transform** — RAWCAP-TRANSFORM-PATH.2, raw captured before the transform for a positional raw-view predicate), the `value_compare` rule-span constraint (six ops + backtrack + under-transform), branch-start inline actions (INLINE-ACTIONS.2), library no-op parity, and **memoization × store** composition (gates re-evaluate fresh on memo hits; the memo is TAINT-GATED with write-epoch VALIDATION since MEMO-STORE-SOUNDNESS.2 — a store-consulting body's outcome is epoch-stamped, replayable only while the store is unchanged, evicted after any store write; all pinned) — each run through BOTH the interpreter (1.5) and the compile-and-run oracle (1.4), asserted **byte-identical** (verdict + `furthest_position` + typed AST). Landing the suite also landed the interpreter's semantic-orchestration mirror + split memo, closing the §13.4/§3.4 honest bound.
- **WHEN:** after ANY interpreter or semantic-runtime orchestration change (regression guard); to prove/measure a *specific* store-gated construct in isolation; to consult the pinned grammar-author facts (unquoted predicate-arg identifiers; inline branch predicates flatten rule-wide; positional `$N` never resolves in directive payloads; the memo is taint-gated with write-epoch validation (stale store-dependent entries never replay)).
- **HOW:**
  ```bash
  make -C rust SHELL=/bin/bash parse_harness_semantic_gate
  # scouting: the full per-construct, per-input interp-vs-oracle map
  cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_semantic_suite::measurement::measure_semantic_suite -- --ignored --nocapture
  ```
- **OUTPUT:** 2 gate tests pass (`every_semantic_construct_is_byte_identical` — 36/36 CLEAN; `semantic_construct_coverage_is_complete`). Deterministic by construction (fixed grammars × curated inputs). Honest bounds (§21.3): bootstrap facts + real library I/O are registry-owned (out of harness scope); no coverage lane. Full map: book chapter *The Parse Harness* → *The semantic-directive orchestration suite* + `docs/tasks/PARSE-HARNESS.md` §21.

---

## 2. Trace — watch the parser explain itself (`parseability_probe`)

### 2.1 Trace verbosity
- **WHAT:** five additive levels — `none` < `low` (🧭 errors/backtracks) < `medium` (🧩 success exits) < `high` (🔎 branch dispatch + **predicate verdicts**) < `debug` (🧠 + predicate mechanism / resolved args).
- **WHEN:** you need to see what the parser *did*. Start at `high` for predicate/branch questions; `debug` when `high` isn't enough.
- **HOW:**
  ```bash
  PGEN_TRACE_VERBOSITY=high ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 --trace
  ```
  `--trace` alone implies `debug`. The `[file:position]` in each line is the **input byte position**, not a source line.
- **⚠️ VOLUME:** full trace on a real input is hundreds of MB — almost always scope it with `--trace-rules` (2.2).

### 2.2 `--trace-rules R1,R2,…`
- **WHAT:** activate trace ONLY inside the call-tree of the listed rules (100–1000× volume reduction). Implies `--trace`.
- **WHEN:** you already suspect a rule (e.g. from the dashboard, 3.1) and want just its story.
- **HOW:**
  ```bash
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
    --parse systemverilog f.sv --profile sv_2017 --trace-rules known_unscoped_covergroup_type_identifier
  ```
- **OUTPUT:** trace lines only within those rules' subtrees. Add `--trace-log-file dbg.log` to capture; strip ANSI with `sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g'`.

### 2.3 `--trace-log-file [FILE]`
- **WHAT:** route trace to a file (default `trace.log`). **WHEN:** large traces you want to grep. **HOW:** append `--trace-log-file dbg.log` to any `--trace`/`--trace-rules` run.

### 2.4 Predicate self-explaining trace
- **WHAT:** at `high`/`debug`, every `@predicate` evaluation emits a verdict (`🛡️ PASSED/REJECTED/INAPPLICABLE`) and, on rejection, the resolved args and the `🚫 Rule '…' rejected by post predicate '…'` line.
- **WHEN:** a parser rejects valid input and you suspect a semantic gate (`has_fact`/`fact_attribute_equals`/`lacks_fact`/…). **This is Step 2 of the `UNKNOWN` protocol.**
- **HOW:** run 2.2 scoped to the suspect rule and grep:
  ```bash
  ... --trace-rules <rule> 2>&1 | grep -iE "🚫|🛡️|has_fact|fact_attribute|NEGATIVE"
  ```
- **OUTPUT (example):**
  ```
  🚫 Rule 'known_unscoped_covergroup_type_identifier' rejected by post predicate
     'fact_attribute_equals [type_name, "\foo", declaration_family, covergroup]'
     ↪ NEGATIVE: 3 facts of kind 'type_name' exist (none matched name "\foo")
  ```
  → the use-site identifier was never declared with the required fact.

---

## 3. Performance, stuck parses, error locus

### 3.1 `--dump-rule-call-counts [N]` (+ `--dump-rule-call-counts-exclude`)
- **WHAT:** live top-N per-rule call-count dashboard (refreshes 250 ms in place); keeps refreshing until a timeout kills the process, so you see the last snapshot.
- **WHEN:** a parse hangs / is slow and you don't yet know which rules dominate; to confirm "is the recursion in `expression`?".
- **HOW:**
  ```bash
  timeout 30 ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 \
    --dump-rule-call-counts 20 --dump-rule-call-counts-exclude "trivia,identifier,lparen,rparen,comma,dot"
  ```
- **OUTPUT:** a live table of rule → call count, exclusion applied before the top-N so noise (`trivia`…) doesn't eat slots.

### 3.2 Furthest-position error diagnostic (always on)
- **WHAT:** every parse-failure error is augmented with `furthest_position` — the deepest byte any branch reached (even backtracked), which is where the real defect lives (the surface position is often megabytes shallower).
- **WHEN:** any "did not consume full input at position N" — map `furthest_position` to a line instead of bisecting.
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 2>&1 | tail -1
  # → Parser did not consume full input at position 113637 [furthest_position=643297, +529660 bytes deeper]
  F=643297; L=$(head -c "$F" f.sv | wc -l); sed -n "$((L-3)),$((L+3))p" f.sv
  ```

### 3.3 `PGEN_REPORT_MEMO_STATS`
- **WHAT:** print packrat memo hit/miss statistics. **WHEN:** perf triage of a slow parse. **HOW:** `PGEN_REPORT_MEMO_STATS=1 ./rust/target/release/parseability_probe --parse <g> f.sv`.
- **ROUTING (RGX-0078.5.i.7 D2-A):** setting the env routes the parse to the PROTOCOL graph (fused `cascade_*` fns carry no memo lane, so memo stats are truthful only there) — automatic, like every diagnostic consumer under the observability twin.

### 3.4 `--dump-rule-entry-counts-json`
- **WHAT:** after a `--parse`, write the parser's monotone per-rule ENTRY counters (every rule-method entry, successful AND backtracked — the always-on `fetch_add` on rule entry) as JSON `{grammar, accepted, total_entries, rule_entry_counts}`. The machine-readable dual of the live dashboard (3.1), which is stderr-only/refresh-based and useless for a sub-millisecond parse. Counts are a DELTA past a pre-parse baseline (SV's stdlib preload never pollutes them) and deterministic for a deterministic parser ⇒ a re-runnable oracle. Wired for every registered grammar's canonical-entry `--parse` path (not `--entry-rule`). RGX-0078.5.h.1.
- **WHEN:** quantifying rule-entry cost models (the ~262ns/entry rule-cascade analysis); feeding the fusibility census's measured entry share (5.3); any "how many times was rule R actually entered" question needing exact numbers, not a dashboard.
- **HOW:**
  ```bash
  ./rust/target/debug/parseability_probe --parse regex /tmp/pattern.txt --dump-rule-entry-counts-json /tmp/counts.json
  ```
- **OUTPUT:** the JSON file (rules sorted, zero-count rules omitted). Counts are build-mode-independent (a debug probe gives the same numbers as release).
- **ROUTING (RGX-0078.5.i.7 D2-A):** requesting this dump routes the parse to the PROTOCOL graph automatically — the `rule_call_counts()` accessor (grabbed pre-parse for the baseline) marks the parser's counter-consumer flag, so the fused cascade graph (which ticks no per-rule counters) never serves a counted parse.

### 3.5 `--dump-rule-outcome-counts-json`
- **WHAT:** after a `--parse` run with the transactional coverage stack enabled, write the raw monotone per-rule entry counters, the COMMITTED (surviving) per-rule counts, AND the per-rule memo-HIT counts as JSON `{grammar, accepted, total_entries, total_committed, total_memo_hits, rule_entry_counts, rule_committed_counts, rule_memo_hit_counts}` — plus the parse's semantic-store counter DELTAS (`store_counters`: rollbacks / facts emitted / rolled back / scopes± / predicate evaluations — RGX-0078.5.i.1 — and the checkpoint/delta-protocol exposure classification `rollbacks_unchanged` / `rollbacks_tournament` (=`extract_delta_since` calls) / `rollbacks_tournament_unchanged` (=empty-delta extractions) / `rollbacks_nonempty_chain` — RGX-0078.5.i.5, the P3c scout). `raw − committed` = the rule's FAILED-speculation entries (every entry `try_parse` rolled back — the parse's probing waste); `raw − memo_hits` = the rule's BODY executions (a memo hit answers from cache without running the body — RGX-0078.5.i.4, the P1 inline census's lost-hit input; hits are recorded by `memoized_call` under the same coverage opt-in, so ordinary parsing pays nothing). Committed semantics are C3-B: tournament winners AND successful-but-losing branches both survive. Deterministic ⇒ a re-runnable oracle; committed counts are meaningful only for an ACCEPTED parse. RGX-0078.5.h.1b + .5.i.4.
- **WHEN:** quantifying wasted vs productive parse work per rule; feeding the choice-site + inline census's dynamic input (5.3 `--fusibility-outcome-counts`); any "how much of this parse was failing probes" or "how memo-hot is this rule" question.
- **HOW:**
  ```bash
  ./rust/target/debug/parseability_probe --parse regex /tmp/pattern.txt --dump-rule-outcome-counts-json /tmp/outcome.json
  ```
- **OUTPUT:** the JSON file (rules sorted; zero-count rules omitted per map). Opt-in: unset ⇒ the coverage stack stays disabled and behavior is byte-identical. Build-mode-independent like 3.4.
- **ROUTING (RGX-0078.5.i.7 D2-A):** the outcome dump enables coverage ⇒ the parse runs the PROTOCOL graph, so the raw/committed/memo-hit pins stay byte-exact forever under the observability twin (a BARE parse — no coverage/trace/counters/memo-stats consumer — runs the fused `cascade_*` graph instead; its byte-identity is enforced by the equivalence/AST oracles, not by counters).

---

## 4. Certificate-coverage — the `UNKNOWN` / trustworthiness toolbox (`ast_pipeline`)

### 4.1 `--report-certificate-coverage`
- **WHAT:** for every rule, is it covered by a verified unreachability PROOF or a verified reachability WITNESS? Prints `proof/witness/UNKNOWN`. `UNKNOWN=0` with no failures = the objective "trustworthy on this grammar" number.
- **WHEN:** measuring a grammar's trustworthiness; the headline number before/after any grammar or generator change. **Always confirm determinism at seeds 0/7/42.**
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage \
    --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0
  ```
- **OUTPUT:** `CERTIFICATE-COVERAGE: … total=1300 witness=1244 UNKNOWN=55 (sample_parse_failures=0, proof_reverify_failures=0)`.

### 4.2 `PGEN_CERT_COVERAGE_DUMP_ALL`
- **WHAT:** prints the **full** `UNKNOWN` rule list (the default truncates to "25 of N") plus the `WARNING … NO reach path from the entry` dead-rule-candidate list.
- **WHEN:** Step 0 of the protocol — you need every residual rule by name to categorize them.
- **HOW:** prefix 4.1: `PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/ast_pipeline … --report-certificate-coverage …`.

### 4.3 `PGEN_CERT_COVERAGE_DEBUG_PROBES`
- **WHAT:** for each targeted `UNKNOWN`, prints a `[plannable-probe] rule='X' parsed=<b> witnessed_target=<b> sample="…"` line — the **forced witness sample** the pass generated and whether it parsed / hit the target.
- **WHEN:** **the WHY tool.** Step 1 of the protocol — tells you *how* each `UNKNOWN` failed.
- **HOW:**
  ```bash
  PGEN_CERT_COVERAGE_DEBUG_PROBES=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
    --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0 \
    > /tmp/probe.txt 2>&1
  grep "rule='<target>'" /tmp/probe.txt
  echo "parsed=false (malformed/store-gate): $(grep -c 'parsed=false' /tmp/probe.txt)"
  echo "parsed=true witnessed=false (reach gap): $(grep -c 'parsed=true witnessed_target=false' /tmp/probe.txt)"
  ```
- **READING:** `parsed=true witnessed_target=true` = witnessed; `parsed=true witnessed_target=false` = **reach/routing gap**; `parsed=false` = **malformed forced sample** (usually a store-gate rejection — confirm with 2.4).

### 4.4 `PGEN_REACH_PATH_DUMP`
- **WHAT:** prints the BFS hop chain (`reach_hops`) the planner installs to steer generation toward each target — the entry→target rule path + chosen branch per hop.
- **WHEN:** a `parsed=true witnessed_target=false` result — to see *which* path stole the bytes.
- **HOW:** prefix any generation/cert command: `PGEN_REACH_PATH_DUMP=1 ./rust/target/debug/ast_pipeline … 2>&1 | grep -i reach`.

### 4.5 Witness-pass knobs (A/B isolation; default-off / default-floor)
- `PGEN_WITNESS_NO_PURDOM=1` — disable Purdom shortest-derivation ordering (A/B: is the ordering the cause?).
- `PGEN_WITNESS_TIMEOUT_FLOOR_MS` — per-target witness budget floor (default 200 ms); raise to give a deep target more budget.
- `PGEN_GENERATION_STEPS_PER_MS` — steps→time calibration for the step budget.
- `PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS` — per-sample deterministic step-budget (B1) for the cert-coverage **PASS-1 diverse pass** (default `4000` ms = 4 000 000 steps). Bounds the diverse generation so a deeply-recursive grammar cannot hang the cert (e.g. `rtl_const_expr` / `conditional_expr` at `--max-depth` 40/48 spin unboundedly without it; the budget cuts them deterministically in ~15 s). The default is far above any well-behaved grammar's per-sample cost (byte-identical cert for the 6 fully-certified grammars + SV at their canonical depths). Set `0` for the legacy unbounded pass deliberately. (CERT-GEN-BUDGET.2.)
- **NOT-depth check:** re-run 4.1 at `--max-depth 24`, `32`, `40`; an unchanged `UNKNOWN` set proves the per-target budget is adequate and the cause is a forcing/store-gate bug, not depth.

### 4.6 `PGEN_CERT_RESIDUAL_CLASSIFICATION`
- **WHAT:** read-only machine classification of the residual `UNKNOWN` set under a dialect profile into `profile_entry_unreachable` (P1: not positively reachable from any DECLARED entry over the active profile-filtered tree, satisfiability-honest edges) / `store_unproducible_under_profile` (P2: a mandatory positive store-gate — `has_fact`/`fact_attribute_equals`/`fact_count_at_least`≥1, never `lacks_fact` — whose kind no live rule can emit, cascaded to fixpoint) / `genuine` (the honest remainder). Reports `DEGRADED-INERT` when a live rule carries `@import_from_library`.
- **WHEN:** adjudicating a profiled residual (e.g. `verilog_2005`'s 327) — which entries are profile-excluded BY CONSTRUCTION vs genuinely actionable. Diagnostic only; headlines/generation untouched (byte-identical with the var unset).
- **HOW:** prefix 4.1; the entry universe = the cert entry + every `--cert-union-config` entry present in the active tree, so pass the alternate entries or an entry-relative cohort (`library_text`, …) will honestly read "unreachable from the single entry":
  ```bash
  PGEN_CERT_RESIDUAL_CLASSIFICATION=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
    --report-certificate-coverage --grammar-profile verilog_2005 --entry-rule systemverilog_file \
    --count 40 --seed 0 --cert-union-config sv_multi_entry_root:verilog_2005 --cert-union-config library_text:verilog_2005
  ```
- **OUTPUT:** a `RESIDUAL-CLASSIFICATION` block after the `UNKNOWN` list — the three named lists, with per-rule reasons on the store class (`gate kind 'type_name' unproducible` / `mandatory descent forces '<rule>'` / `stranded by the store fixpoint`). Promotion to per-profile `proof` certificates = `VERILOG-2005-PROFILE.6.7`.

---

## 5. Static analysis & generation IR (`ast_pipeline`)

### 5.1 `--lint-grammar`
- **WHAT:** static well-formedness report — left-recursion info; HARD-gated errors for non-terminating rules, ordered-choice shadowing (both verdicts are now SELECTION-SEMANTICS-CONDITIONAL: exact-duplicate fires only where the tie-break provably keeps the EARLIER twin — under `@associativity: right`, a later-higher `@priority`, or `@deterministic_group` evaluation-order rotation the engine SELECTS the later twin, so no verdict; under `@associativity: nonassoc` an equal-priority tie fails the whole choice — a distinct "restructure deliberately" verdict since merge/remove would change acceptance (A2.4); fixed-terminal-prefix ONLY on an `@branch_policy: ordered` rule with no branch-phase predicate and no partition rotation — under the default `longest_match`/`priority_first` the later alternative is LIVE (A2.3)), unreachable rules, **undefined references** (a rule referencing a rule never defined — codegen would emit a never-matching stub; the check runs on the UNFILTERED grammar and allowlists codegen's native builtins), unbound fact-kinds, and profile orphans; nullable-repetition warnings; always-succeeds notes — then exits (nonzero on any error-class finding).
- **WHEN:** after any grammar edit; to adjudicate a `no_path`/dead-rule candidate; **"every parse rejects at `furthest_position=0` and nothing points at the cause"** (the undefined-ref signature); "is my grammar well-formed?".
- **HOW:** `./rust/target/debug/ast_pipeline grammars/<g>.ebnf --lint-grammar`.

### 5.2 `--dump-gen-ast` (+ `--dump-gen-ast-pretty`)
- **WHAT:** dump the **normalized generation-input AST** — the grammar IR the parser/stimuli generators actually consume (post LR-elimination etc.).
- **WHEN:** "what does the generator actually see?", debugging a codegen/stimuli surprise, diffing IR across a grammar edit.
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/<g>.ebnf --generate-parser \
    --dump-gen-ast gen_ast.json --dump-gen-ast-pretty --eliminate-left-recursion --output /tmp/p.rs
  ```

### 5.3 `--report-fusibility-census`
- **WHAT:** the DERIVED-SCANNER capability-gate classifier (RGX-0078.5.h STEP-0, `rust/src/ast_pipeline/fusibility_census.rs`) — for every rule of the UNFILTERED grammar, is its subtree compilable to a direct-coded DFA `scan_R(pos)` under the increment-1 strict gate (regular + effect-free + text-folding + policy-encodable + layout-contiguous, each criterion mirroring the engine's own resolution)? Prints per-tier counts (`fusible_token`/`fusible_lookahead`/`not_fusible`, plus `shape_only` = language-encodable but value-blocked), the MAXIMAL fusible roots (future `scan_*` sites), a disqualification histogram, and the grammar's regex-literal ATOM-site count (the `match_regex`/self-hosting surface). With `--fusibility-entry-counts F1,F2,…` (files from 3.4) it prints the measured `FUSIBILITY-ENTRY-SHARE` — the share of real parse entries fusion would eliminate and the implied ceiling (a conservative lower bound). Read-only; verdicts are sound under-approximations.
- **WHEN:** gating/sizing any scanner-fusion or token-DFA work (which rules, how many real entries, what ceiling); asking "why is rule R not token-shaped?" (`PGEN_FUSIBILITY_DUMP_ALL=1` per-rule reasons + per-site verdicts); measuring a grammar's "token-shapedness" as a standing metric; measuring the increment-(ii) MERGED-CHOICE surface (every Or site's token-shaped branch subset + the measured discarded-work kill surface — RGX-0078.5.h.1b); sizing the P2 DEGENERATE-DISPATCH surface (RGX-0078.5.i.3 — which rule-top-level choice sites can byte-switch dispatch with the tournament protocol elided, what blocks the rest, and the measured Or-body-execution exposure); sizing the P1 CASCADE/WRAPPER-INLINING surface (RGX-0078.5.i.4 — which rules are inline-eligible under gates a–d: acyclic + directive-free frame + non-entry + no `@profiles`/`@transform`, with the failing gate NAMED per rule, the wrapper-class split, and the measured frame exposure incl. the memo-hit share P1b would re-execute).
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/regex.ebnf --report-fusibility-census \
    --fusibility-entry-counts /tmp/c1.json,/tmp/c2.json --fusibility-census-json /tmp/census.json
  # the .5.h.1b merged-choice join (files from 3.5):
  ./rust/target/debug/ast_pipeline grammars/regex.ebnf --report-fusibility-census \
    --fusibility-outcome-counts /tmp/o1.json,/tmp/o2.json
  PGEN_FUSIBILITY_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/regex.ebnf --report-fusibility-census
  ```
- **OUTPUT:** `FUSIBILITY-CENSUS: grammar=regex rules=274 fusible=68 (token=40 lookahead=28) not_fusible=206 (shape_only=75) maximal_roots=57 static_share=24.8%` + the roots list + histogram; with entry counts: `FUSIBILITY-ENTRY-SHARE: … eliminated_below_roots=83 at_roots=495 … ceiling≈1.04x`. Always also prints `CHOICE-SITE-CENSUS: … sites=N with_encodable_subset=M all_encodable=A` (every Or site classified for merged-choice); with outcome counts (3.5): `OUTCOME-SHARE: … total_entries=… committed=… discarded=… (on_encodable=… on_structural=…) ceiling≈…x` — discarded = raw − committed (failed-speculation work), and the kill surface = discarded entries on shape-encodable rules (choice/optional/iteration attempts alike), plus the top choice sites by sole-attributable discarded entries. Always also prints `DEGENERACY-CENSUS: … top_level_sites=N degenerate_dispatch=K` (RGX-0078.5.i.3 — the P2 byte-switch gate per top-level site: all branches first-byte-decided + pairwise-disjoint + terminal-ws-sensitive + no branch predicates/effects; blockers NAMED per site under `PGEN_FUSIBILITY_DUMP_ALL=1`, histogram always) and, with outcome counts, `DEGENERACY-EXPOSURE: … degenerate_site_entries=… committed=… discarded=…` — the Or-body executions the degenerate emission strips of tournament protocol. Always also prints `PREFIX2-CENSUS: … blocked_top_level_sites=N prefix2_dispatchable=K (wildcard_limited=W)` (RGX-0078.5.i.7 D1 — the FIRST₂ two-level-dispatch gate per blocked top-level site: the P2 gates minus first-byte disjointness, plus per shared first byte the admitting subset's second-byte facts RESOLVED and its non-WILDCARD members pairwise-disjoint on byte 2 — a byte-2 WILDCARD member is `len1_possible` (a 1-byte match leaves byte 2 unconstrained), nullable, OR `offset1_rule_entry` (the D1 emission's furthest-position-parity license: a byte-2-refuted attempt could still enter a rule at offset ≥1, so the emitted guard never byte-2-prunes it) — wildcards are legal but kill-capping; blockers + `[PREFIX2]` per site under `PGEN_FUSIBILITY_DUMP_ALL=1`, histogram always) and, with outcome counts, `PREFIX2-EXPOSURE: … prefix2_site_entries=… committed=… discarded=…` — the Or-body executions a two-level byte dispatch strips (byte-1-admitted/byte-2-refuted attempts never entered). Note the LANDED D1 emission is per-BRANCH FIRST₂ prune guards (needing NO byte-2 disjointness), so its kill surface EXCEEDS the census's dispatchable-site model; the census remains the site-level dispatchability instrument. Always also prints `INLINE-CENSUS: … rules=N inline_eligible=K (pass_through=… alternation_leaf=… shaped=…)` (RGX-0078.5.i.4 — the P1 rule-inlining gate: acyclic + directive-free frame + non-entry + no `@profiles`/`@transform`; blocker histogram always, per-rule class/refs/body-size + emission decision under `PGEN_FUSIBILITY_DUMP_ALL=1`) followed by `INLINE-DECISIONS: … decided_under_budget=K over_budget=M (expansion_cap=12 duplication_cap=192)` — the P1a EMISSION decision under the shared code-size budget (`fusibility_census::compute_inline_decisions`, the SAME function codegen consumes, so this report and the emitted parser cannot drift; `decided` rules' call sites receive the rule body inline under `inlined_frame_call`, memo preserved) — and, with outcome counts, `INLINE-EXPOSURE: … eligible_entries=… committed=… discarded=… memo_hits_on_eligible=…` — the collapsible wrapper frames plus the memo-hit share a memo-eliding inline (P1b) would re-execute (hits require dumps from a `.5.i.4`+ parser generation) — plus `INLINE-EXPOSURE-DECIDED: … decided_entries=… committed=… discarded=… memo_hits_on_decided=…` — the same sums restricted to the budget-DECIDED emission plan (`compute_inline_decisions`): the frames the landed P1a emission actually collapses, and the lost-hit surface P1b's memo elision re-executes at the budget. Always also prints `QUANT-SITE-CENSUS: … quantified_sites=N min_zero=M guardable=K (bare_ref=… no_refs=… mixed=…)` (RGX-0078.5.i.7 Q-GUARD — every `Quantified` site classified for FIRST-guarded ATTEMPT ELISION at min-0 sites; the frontier split is the `first_set::quantified_element_frontier` furthest-emulation exactness class the LANDED `-0077` emission consumes — `bare_ref` sites get the guard + the exact `furthest` emulation, `no_refs` sites the guard alone, `mixed` sites stay unguarded: gates = min-0 quantifier + terminal whitespace-sensitivity (the raw-byte peek) + element first-byte-decided via the SHARED `first_set::branch_dispatch_first_bytes` predicate + a Branch-predicate/branch-start-effect-free reachable closure; blockers named per site, histogram always; sites censused at ANY nesting — the elision is local to the quantifier loop, and a min-0 quantifier ALWAYS attempts its element exactly once at the current position, so the furthest-position diagnostic is EXACTLY emulable) and, with outcome counts, `QUANT-EXPOSURE: … attributable_rules=… entries=… committed=… discarded=… | shared_rules=…` — POPULATION-attributed: a rule counts only when EVERY grammar-wide reference occurrence sits under a guardable min-0 site's element subtree (occurrences counted at outermost guardable sites, so nesting never double-counts); honest bounds printed alongside (attributable discards OVER-approximate the kill — byte-1-admitted refutations survive; terminal-only sites are invisible to per-rule counters). Always also prints `CASCADE-CENSUS: … cascade_eligible=N (roots=R internal=I internal_cyclic=C) residual=X boundary_edges=B` (RGX-0078.5.i.7 D2 — the FULL-CASCADE-FOLD gate per rule: can it live INSIDE a fused direct-coded region, i.e. is it EFFECT-FREE (no runtime directive any phase, no mid-sequence inline, no follow restriction) + policy-encodable + free of `@profiles`/`@transform`/value-constraints? Deliberately NOT the scanner tier gate: cycles, layout skips, lookahead, and every `UnifiedReturnAST` value shape are all emittable by a fused *matcher* and are named facts, not blockers; regions root at protocol boundaries, references eligible→ineligible are per-rule `boundary_refs` call-outs; blocker histogram always, per-rule ROOT/internal/BLOCKED verdicts under `PGEN_FUSIBILITY_DUMP_ALL=1`) and, with outcome counts, `CASCADE-EXPOSURE: … internal=… (…%) committed=… discarded=… memo_hits=… | roots=… committed=… | residual=… committed=… | committed_floor=…` — the first-order fold model: internal entries' per-entry protocol is eliminated, root entries become one specialized-function call each, `committed_floor` = the post-fold protocol-paying committed entries (the honest pricing denominator); internal memo hits re-execute (the emission design owns the re-probe boundedness proof). Always also prints `CASCADE-PLAN: … increment=A(acyclic-subregions) sub_roots=S internal=I effect_reaching_fused=E` (RGX-0078.5.i.7 D2-A — the SHARED acyclic-sub-region emission plan `fusibility_census::compute_cascade_emission_plan`, the SAME map codegen consumes so report and emission cannot drift: CYCLIC eligible rules stay protocol boundaries in this increment, sub-roots keep the full protocol frame + the observability-twin dispatch, internal rules run as fused `cascade_*` functions on the bare-parse path, and `effect_reaching` marks rules from whose body an ineligible rule is reachable — the conservative C3-B snapshot/island obligation set; the plan additionally carries `effect_targets` (= ineligible rules ∪ effect_reaching — the per-SITE membership test the LANDED `-0087` emitter uses to pick plain-restore vs `try_parse` speculation and plain-vs-island Or sites); per-rule `[cascade-plan]` verdicts under `PGEN_FUSIBILITY_DUMP_ALL=1`). Always also prints `CASCADE-PLAN-B: … increment=B(cyclic-spine) sub_roots=S internal=I thin_memo=T effect_reaching_fused=E` (RGX-0078.5.i.7 D2-B plan seam — the SAME plan function at the `CyclicSpine` increment, the map the D2-B emitter will consume: EVERY cascade-eligible rule fused, sub-roots = the census's own full-fold roots, `thin_memo` = the cycle-participating fused rules, which carry the epoch-stamped thin memo — ⛔ the session-#49 bound, they never lose memo protection; the effect fixpoint is increment-independent; per-rule `[cascade-plan-b]` verdicts under `PGEN_FUSIBILITY_DUMP_ALL=1`). The D2-A EMITTER IS LANDED: every generated parser carries its fused graph, entered only on bare parses (see the routing notes at 3.3/3.4/3.5 — any diagnostic consumer routes to the protocol graph automatically). The full 11-grammar census + method notes live in the `.5.h.1`/`.5.h.1b`/`.5.i.3`/`.5.i.4`/`.5.i.7` sections of `docs/tasks/RGX-0078.md`.

---

## 6. Coverage / gap reports (`ast_pipeline`)

- `--report-k-path-coverage K` — k-path (Havrikov-Zeller) coverage report at depth K (use 2–3); read-only.
- `--gap-report-json FILE` / `--gap-report-text FILE` (+ `--gap-report-threshold`) — emit the coverage-gap / unreachable-rule-debt report consumed downstream.
- `--target-report-input FILE` — load a prior gap report and apply its targets as generation priorities (the witness/closed-loop replay surface).

---

## Protocol A — diagnose an `UNKNOWN` (3 steps)

1. **Full list + honest number** — `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --report-certificate-coverage … --seed 0` (re-run seeds 7, 42 for determinism).
2. **WHY each one failed** — `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 …` → read `parsed` / `witnessed_target` per `[plannable-probe]` line.
3. **Exact rejection** — for a `parsed=false` rule, feed its forced sample back through `--trace-rules <rule>` at `PGEN_TRACE_VERBOSITY=debug` → the `🚫 rejected by post predicate …` line.

Cause map: `NO reach path` = dead-rule candidate (adjudicate via 5.1) · `parsed=true witnessed=false` = reach/routing gap · `parsed=false` + predicate-reject = store-gate (the precondition fact was never generated → store-aware generation).

## Protocol B — a parser rejects valid input
1. `--parse` and read `furthest_position` (3.2) → map to a line.
2. Minimal-reproduce that construct; `--parse` it. Fails on the minimal = localized; passes = contextual (bisect what precedes).
3. If a `@predicate` is suspected, `--trace-rules <suspect>` at `high`/`debug` (2.4) → the verdict names the rejecting predicate and resolved args.

## Protocol C — a parse is slow / hangs
1. `--dump-rule-call-counts 30 --dump-rule-call-counts-exclude "trivia,…"` (3.1) → the dominating rules.
2. `--trace-rules <dominator>` (2.2) → the actual call pattern; `PGEN_REPORT_MEMO_STATS=1` (3.3) for memo behavior.

## Protocol D — which alternative WINS this choice? / is this branch LIVE? (the A2.2/A2.3-class probe)

For any "is this ordered-choice branch dead or live / which alt does the engine actually select?"
question — NEVER answer it from the grammar text (PGEN's `|` is a branch TOURNAMENT, default
`longest_match`, NOT PEG first-match commit; two linter verdicts were unsound for exactly this reason).

1. **Isolate the choice in the scratch slot (1.3)** — `scratch := <alt0> | <alt1> | …`, carrying the
   real rule's `@branch_policy` / `@priority` / `@associativity` annotations if any; then
   `make -C rust SHELL=/bin/bash focus_scratch` + rebuild the release probe (it embeds scratch at
   COMPILE time).
2. **Parse the discriminating input and READ THE WINNER** — codegen's own selection line:
   ```bash
   PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
     --parse scratch /tmp/in.txt --trace-rules scratch 2>&1 | grep 🏁
   # → 🏁 Rule 'scratch' selected branch N/M consuming K chars (priority=…, associativity=…, branch_policy=…)
   ```
   `N` names the winning alternative; `--parse-dump-ast-pretty` (1.2) confirms the winner's shape.
   (For a rule inside a REAL registered grammar, skip the slot and run step 2 directly with
   `--trace-rules <rule>` on the real parser.)
3. **Cross-check against the pinned policy matrix** (`parse_harness_combinator_gate`, 1.7): only
   `ordered` commits to the first success; `longest_match` (the default) tries every alternative and
   keeps the longest (ties → earlier, or later under `@associativity: right`); `priority_first` ranks
   by `@priority` with longest-match tie-break.

Cause map: a branch the engine SELECTS is **LIVE** — any deadness verdict must be conditioned on the
rule's actual selection semantics (precedents: A2.2 `EarlierAlwaysMatches` retired; A2.3
`FixedTerminalPrefix` policy-conditioned — `docs/decisions/project_fixed_terminal_prefix_policy_conditional.md`;
A2.4 `DuplicateAlternative` tie-break-conditioned on `@associativity`/`@priority`/`@deterministic_group` —
`docs/decisions/project_duplicate_alternative_selection_semantics_conditional.md`, the R/P/D/N probes
above are its worked example). With A2.2/A2.3/A2.4 every ordered-choice deadness verdict now names AND
checks its selection semantics.

---

*This catalog is part of the four-surface "impossible to ignore" debug-toolbox capture (book + KM +
decisions + project memory). Keep it in lockstep when a debug surface is added or changed.*
