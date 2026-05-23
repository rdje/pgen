# SV-EXH-PROOF — Systematic Defect Taxonomy

**Living tracking document.** Every defect class uncovered during the SV-EXH-PROOF campaign (and beyond) is recorded here so we can:

- analyse each class on its own terms,
- run the audit patterns to find hidden instances,
- never re-encounter a "we already knew about this" surprise,
- track open / partial / risk states without losing them between sessions.

---

## How to use + amend this document

**When a new defect class is found.** Add an entry under the right category section with the standard fields (below). If you can't fit it cleanly into A–G, add a new category and explain it.

**When a class is fixed.** Update `Status` to ✅, fill `Fix` with the commit + diff summary, and add an entry to the **Amendment log** at the bottom.

**When an audit-pattern sweep finds more instances of a known class.** Either inline them in the class entry's `Instances` field, or split into sub-classes if the pattern is non-uniform.

**Standard fields for each defect class.**
- **ID** — `A1`, `D1`, etc. Stable; never reused.
- **Name** — short, descriptive.
- **Status** — `FIXED` ✅ / `PARTIAL` 🟡 / `OPEN` 🔴 / `RISK` ⚠ / `PROCEDURAL` 🛠 / `MITIGATION ONLY` 🩹.
- **Discovered** — leaf ID + commit hash (link back to `docs/tasks/SV-EXH-PROOF.md`).
- **Pattern** — one-paragraph description of the structural shape.
- **Root cause** — *why* it manifests.
- **Instances** — count + scope ("class of one", "17 across `systemverilog.ebnf`", "uninventoried").
- **Fix** — approach + commit; empty if still OPEN.
- **Audit pattern** — mechanical command(s) to find more instances, or the oracle to compare against.
- **Related** — memory links `[[name]]`, sibling defect IDs.

**Status legend.** ✅ FIXED (closed, no follow-up). 🟡 PARTIAL (some fixed, more known). 🔴 OPEN (not started). ⚠ RISK (not yet hit but anticipated). 🛠 PROCEDURAL (no code fix; a workflow / checklist). 🩹 MITIGATION ONLY (no real fix, only workarounds — flag for engine work).

---

## A. LRM-extraction defects

The SV grammar is auto-extracted from the IEEE 1800 LRM. The extractor occasionally mis-encodes LRM productions; these defects show up as grammar that *almost* matches the spec.

### A1. `&X X` mandates an LRM-optional clause ✅
- **Discovered**: `.3.3.4.b.1` (commit `PGEN-SV-EXH-PROOF-0028`).
- **Pattern**: LRM `[ X ]` (optional) extracted as `&X X` (positive-lookahead + mandatory) — the clause appears optional but PEG semantically requires it.
- **Root cause**: extractor mis-mapped the LRM bracket notation.
- **Instances**: **1** — `conditional_statement` `[else statement_or_null]`. Empirically class-of-one across the 1448-rule grammar.
- **Fix**: `conditional_else_clause` helper rule + `( … )?` quantifier on the else branch (PGEN-SV-EXH-PROOF-0028, rel 1.0.125).
- **Audit pattern**: `grep -nE '&[a-z_][a-z_0-9]* [a-z_][a-z_0-9]*' grammars/systemverilog.ebnf` — should return ≤1 hit (the legitimate `&question question` in `conditional_expression`).
- **Related**: [[feedback_verify_rule_correctness_before_runtime_hypotheses]].

### A2. LRM `[ X ]` ambiguity — literal-brackets vs EBNF-optional ✅ (mostly-resolved)
- **Discovered**: flagged at `.b.1` / `.b.4` as a residual class.
- **Pattern (refined 2026-05-23)**: the LRM Annex A uses `[ X ]` for BOTH (a) EBNF-optional and (b) literal SystemVerilog bracket syntax (`arr[i]`, `bins b[3]`, `foreach (arr[i])`, `enum_id [ N [ : M ] ]`, …). The notation is ambiguous in the source PDF — the same square brackets serve both roles. Similarly `{ X }` serves both EBNF-repetition AND literal SV brace syntax (`struct { … }`, `'{...}`). The auto-extractor `tools/extract_systemverilog_lrm_profiles.py` treats all `[ X ]` as `( X )?` and all `{ X }` as `X*`, mis-encoding the literal-bracket/brace cases.
- **Root cause**: ambiguous LRM notation that the mechanical extractor can't disambiguate without semantic context.
- **What the active grammar does**: extensively manually corrected — literal-bracket cases rewritten to `lbrack X rbrack`, literal-brace cases to `lbrace X rbrace`, and the surviving `( X )?` / `X*` quantifiers correctly mark true optional/repetition. The active grammar has **447** `?` quantifiers vs the extractor's **292** — *more* optionality, reflecting correct additions where the extractor failed.
- **Instances**: oracle (`tools/lrm_optional_audit.py`) flags **15 candidate rules** where the LRM-extracted version has MORE `?` than the active. Manual review (2026-05-23): **all 15 are legitimate** — 6 are literal-bracket fixes, 6 are refactor / helper-rule extractions, 3 are mixed. **Zero un-handled A2 defects in the current active grammar.** Only `.3.3.4.b.1`'s `conditional_statement` was a genuine open instance; it was its own sub-class (A1) and is fixed.
- **Fix (already done)**: the manual correction work across the grammar's history; `.b.1` for the one remaining truly-dropped optional.
- **Audit pattern**: `python3 tools/lrm_optional_audit.py` — flags any new mismatch; per-candidate review against `docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt` to classify (legitimate refactor / literal-bracket / true regression).
- **Related**: A1 (sibling `&X X` form, also class-of-one); the LRM extractor `tools/extract_systemverilog_lrm_profiles.py`.
- **Closing notes (2026-05-23, deep-understanding pass)**: the original taxonomy entry predicted "many likely instances" of dropped-`?`. The systematic audit refuted that prediction — the active grammar is already LRM-faithful for all 15 oracle-candidate rules. The defect class A2 is therefore better characterised as a **historically-active class now MITIGATED**, with the oracle script as the durable regression guard.

---

## B. PEG semantics defects

The PEG model (ordered choice, possessive `*`/`+`, lookaheads) interacts with the grammar's shape in ways that bite — especially around chained constructs and method calls.

### B1. Over-greedy hierarchical_identifier `*`-loop vs trailing `.method(args)` ✅
- **Discovered**: `.3.3.4.b.2`.
- **Pattern**: `hierarchical_identifier`'s `( identifier . )*` consumes through what should be a trailing `.method(args)`, leaving no identifier for the method call.
- **Root cause**: PEG `*` is possessive; without a `!`-guard at the loop boundary, it greedily eats the final segment.
- **Instances**: **1** (`hierarchical_identifier`).
- **Fix**: surgical loop-guard fix at the rule.
- **Audit pattern**: `grep -nE '\( [a-z_]+ \. \)\*' grammars/systemverilog.ebnf` — any `( ident . )*` followed by something that *could* be a method call needs an explicit guard.

### B2. Receiver-type-blindness in method-call disambiguation ✅
- **Discovered**: `.b.4` (diagnosis); `.b.6.2` (fix), commit `3c754283`.
- **Pattern**: `callable_method_call_body` matches a bare identifier (via `array_manipulation_call`'s optional parens), so `!callable_method_call_body` in `split_hierarchical_callable_receiver`'s `*`-loop always trips → 3+level chains fail when routed through `call_primary` (no chain wrapper).
- **Root cause**: the bare-parens form of `array_manipulation_call` is LRM-correct (`arr.unique`), so disambiguation needs semantic context, not pure syntax.
- **Instances**: 1 grammar shape, but blocks an unbounded class of inputs (`a.b.c(x)`, `…(x).d`).
- **Fix**: new `context_member_method_call` priority-first branch in `call_primary`, gated by `@predicate has_fact(variable_binding, $head) phase: post` — a *regression firewall* that only fires for known-variable heads.
- **Audit pattern**: anywhere `call_primary` (or another expression-level rule) uses a path-receiver rule *without* a chain wrapper — at risk of the same 3+level cascade.

### B3. PEG quantifier non-atomicity (pre-Layer-0) ✅
- **Discovered**: `.b.3` (Layer 0), commit `PGEN-SV-EXH-PROOF-0029`.
- **Pattern**: codegen emitted `+`'s first iteration **inline** without `try_parse`; on first-iter failure the cursor stayed past partial consumption. Bounded forms `{N}/{N,M}/{N,}/{,M}` were a silent `_ => Err("Unknown quantifier")` fallthrough.
- **Root cause**: three independently-written code paths in the codegen, with the `+` case asymmetric vs `*`/`?`.
- **Instances**: every `+` and every bounded-quantifier usage grammar-wide.
- **Fix**: unified quantifier engine (`parse_quantifier_bounds` helper + one `loop` with every iteration in `try_parse` + min-failure rollback to `quantifier_start_position`).
- **Audit pattern**: every quantifier surface must route through `parse_quantifier_bounds`; the helper is the single source of truth.
- **Related**: [[feedback_layer_0_unified_quantifier]].

---

## C. Codegen defects

The generator itself emits wrong or incomplete code.

### C1. Registry never serialised into generated parsers 🟡
- **Discovered**: `.b.6.1.1` (`fact_kinds` half fixed at commit `25e5c10d`).
- **Pattern**: `generate_compiled_semantic_runtime_annotations_tokens` builds `CompiledSemanticRuntimeAnnotations` via `from_parts(directives_by_rule, branch_directives_by_rule)` — which zeroes `fact_kinds` AND `predicate_defs`. So the runtime registry is populated only on the *compile-time* path; generated parsers always see an empty registry.
- **Root cause**: invisible until a grammar actually USED `@fact_kind:` / `@predicate_def:`. `.b.5.1.2` and `.b.5.1.5` added compile-time tests but never end-to-end-tested the generated parser.
- **Instances**: **2** — `fact_kinds` (✅ fixed), `predicate_defs` (❌ open).
- **Fix (partial)**: `fact_kinds` via `generate_fact_kind_decl_tokens` + `CompiledSemanticRuntimeAnnotations::set_fact_kinds`. **`predicate_defs` still un-emitted** — needs a recursive `PredicateExpr` → TokenStream serialiser. Owned by `.b.6.2`-followups when composed `@predicate_def:` is first used in a grammar.
- **Audit pattern**: every field of `CompiledSemanticRuntimeAnnotations` must have a corresponding emit-block in `generate_compiled_semantic_runtime_annotations_tokens`. Diff the struct fields against the codegen.
- **Related**: [[project_b61_producer_pass_plan]].

### C2. `?`-bypasses-manual-cleanup transaction hazard ✅
- **Discovered**: `.3.3.3` (SEMTRACE-definitive after 3 prior hypotheses disproven), commit `PGEN-SV-EXH-PROOF-0024`.
- **Pattern**: `std::mem::take` / `std::mem::replace` followed by `?`-fallible calls and a manual `if result.is_err() { restore }` — the `?` skips the restore on every error path; the restore is dead code.
- **Root cause**: classic Rust hazard with manual transaction-style code.
- **Instances**: 1 fixed at `with_semantic_runtime_rule_transaction`; the *pattern* is an audit hazard wherever it recurs.
- **Fix**: IIFE try-block emulation — wrap the fallible body in `(|| -> Result<…> { … })()` so `?` returns into a `result` local; the restore on `result.is_err()` is then reachable. Zero `unsafe`.
- **Audit pattern**: `grep -nB2 -A8 -E 'mem::take|mem::replace' rust/src/` — for each hit, check whether a following `?` precedes a manual restore.
- **Related**: [[feedback_question_bypasses_manual_cleanup]].

---

## D. Grammar semantic-annotation defects

Wrong `$<ref>` payloads in `@predicate` / `@emit_fact` / `@open_scope` directives — the directive parses, but resolution silently fails or rejects.

### D1. `@predicate $<rulename>` ref against `->`-shaped content — THE BIG ONE ✅
- **Discovered**: `.b.6.2.2`, commit `c766bc8b`.
- **Pattern**: `@predicate … args:[…, $<rulename>, …]` where `<rulename>` is a sub-rule whose content shapes to `{body:scalar}` via the `non_keyword_identifier -> {body: $2.body}` chain. The Json-mode resolver looks up `<rulename>` as a top-level field — not present — raises `ContextualError("could not resolve attribute reference '<rulename>'")` — rule hard-rejects EVERY time.
- **Root cause**: rule author wrote `$<rulename>` thinking the resolver would do raw-tree rule-name lookup, but the rule has a `->` shape that puts the resolver in Json field-name mode. Aggravated by E2 (resolver throws on missing ref instead of returning indeterminate).
- **Instances**: **17 in `systemverilog.ebnf`** — exhaustively fixed (`non_typedef_package_scope`, `scoped_or_hierarchical_tf_identifier`-branch, `known_unscoped_class_scope_class_identifier` + interface_class + type_parameter, `known_unscoped_base_class_type_identifier`, `known_unscoped_block_class_type` ×2, `known_unscoped_covergroup_type_identifier`, `known_unscoped_interface_class_type_identifier`, `known_unscoped_let_identifier`, `known_unscoped_checker_identifier`, `known_unscoped_property_identifier`, `known_unscoped_sequence_identifier`, `known_unscoped_parameter_identifier`, `known_unscoped_class_scoped_call_class_identifier` + interface_class + type_parameter).
- **Fix**: drill through each rule's actual shape (`$body` / `$head.body` / `$body.name.body` / `$scope.name.body`).
- **Audit pattern**:
  - `grep -nE '@predicate.*args: *\[.*\$[a-z_]+_identifier' grammars/systemverilog.ebnf` — should return 0.
  - Same sweep on `grammars/vhdl.ebnf`, `grammars/rtl_*.ebnf` — D1 may exist there too (uninventoried).
- **Related**: E1, E2 (sibling resolver gaps that make D1 silent); [[feedback_semantic_annotation_no_dotted_refs]].

### D2. `@emit_fact $X.body` shape workarounds (pre-`.a.1`) ✅
- **Discovered**: `.3.3.4.a.1`, commit `PGEN-SV-EXH-PROOF-0026`.
- **Pattern**: before `.a.1` enabled dotted refs in semantic-annotation payloads, authors surfaced scalars at top level via `-> {…, body: $4.body}` so `@emit_fact name:$body` would resolve.
- **Fix**: `.a.1` extended dotted refs end-to-end; the SV grammar's workarounds were reverted in the same slice.
- **Audit pattern**: `grep -nE '\$[0-9a-z_]+\.body\b' grammars/systemverilog.ebnf` inside any `->` shape that exists *only* to surface a scalar for an `@emit_fact` — any such `body:` is a workaround and can now be removed.
- **Related**: D1 (the class D2 workarounds were trying to dodge).

---

## E. Engine / resolver gaps

Limitations of the semantic-runtime resolver and predicate machinery — not "defects" per se, but they shape what the grammar can express.

### E1. Resolver scalar-izes — `$ref` to a non-scalar value is rejected 🩹
- **Discovered**: `.b.6.2` design audit.
- **Pattern**: `resolve_unified_semantic_value_against_content`'s `RuleReference` arm coerces via `coerce_unified_semantic_scalar`. `$ref` to a structured (object/array) value returns None → ContextualError.
- **Consequence**: blocks the structured `type_ref` design for richer producer facts. Drove the `.b.6.2` decision to go minimal-name-presence-only.
- **Status**: 🩹 sidestepped in `.b.6.2`. A *conditional* engine extension for the rich producer pass.
- **Audit pattern**: any future `@emit_fact <attr>: $structured_ref` proposal needs to first verify the ref resolves to a scalar; otherwise routes back here.

### E2. Resolver throws on missing ref → `phase: post` predicate hard-rejects 🩹
- **Discovered**: `.b.6.2.2` trace.
- **Pattern**: `resolve_semantic_reference` returns `Option<String>`; caller does `.ok_or_else(|| ContextualError(...))`. So a missing `$ref` becomes an ERROR, not an indeterminate; a `phase: post` predicate whose arg ref doesn't resolve → ContextualError → rule fails.
- **Consequence**: amplifies D1-class defects into silent always-fail. (If missing → indeterminate → predicate `None` → no-block, D1 would have been visibly broken much earlier.)
- **Status**: 🩹 current behaviour is defensible but compounded the D1 cost. Consider: missing-ref → `None` (indeterminate) instead of error.
- **Audit pattern**: any rule that *silently always rejects* under load → check ref resolution end-to-end with `--trace`.

### E3. Composed-predicate dispatch only works on the unit-test direct-construction path 🟡
- **Discovered**: `.b.6.1.1` (twin gap to C1's `fact_kinds`).
- **Pattern**: a generated parser's `predicate_defs` is always empty (C1) — so `@predicate_def:`-defined composed predicates exist at compile time but never dispatch at parse time.
- **Status**: ❌ open, owned by the slice that introduces the first real `@predicate_def:` in a grammar.
- **Related**: C1 (`predicate_defs` codegen).

---

## F. Tooling / diagnostic gaps

Not grammar defects — they're things that *hide* defects or make them harder to localise.

### F1. PEG "did not consume full input at position X" masks deep failure 🩹
- **Discovered**: `.b.6.2.1`.
- **Pattern**: PEG reports the *unconsumed region's start*. For `package foo;…endpackage` failing, the error position is always the package start — hiding the actual deep failure inside.
- **Consequence**: triage gates can show identical failure positions across genuine forward progress. `.b.6.2` advanced uvm's deep position but the triage's "byte 113637" was unchanged.
- **Mitigation**: bisection harness (`/tmp/uvm_bisect.py` — shrinking-window over a list-rule item set) + `--trace` for the deep position.
- **Tooling proposal**: the parser could track a `furthest_position` (deepest position reached across all backtracks) and surface it on failure. **Not implemented today.**

### F2. Stale `parseability_probe` after a regen 🛠
- **Discovered**: `.b.6.2.3`.
- **Pattern**: `parseability_probe --features generated_parsers` *embeds* the generated parsers at build time. After `make focus_systemverilog`, the probe binary is silently stale; probe pass/fail lies.
- **Cost**: real time chased a phantom "still fails" in `.b.6.2.2`.
- **Procedure**: after every regen, `touch rust/src/lib.rs && cargo build --release --features generated_parsers --bin parseability_probe`. Or assert `target/release/parseability_probe` mtime > `generated/systemverilog_parser.rs` mtime before trusting probe results.
- **Related**: [[feedback_verify_sv_parser_regen_mtime]].

### F3. `make focus_systemverilog` can silently no-op 🛠
- **Pattern**: per [[feedback_verify_sv_parser_regen_mtime]], mtime-based make targets can leave a stale parser even after a "successful" regen.
- **Procedure**: assert parser-mtime > grammar-mtime *or* observe a real FAIL→PASS behavioural change.

---

## G. Design risks (known but not yet hit)

### G1. Declaration ordering / forward references ⚠
- **Pattern**: a class method may reference a property declared *later* in the same class. Single-pass parsing → `has_fact` gate may fail on forward refs.
- **Mitigation in real code**: UVM uses `typedef class X;` forward-declarations heavily — these emit `type_name` facts before the real class definition.
- **Status**: ⚠ not yet hit by the campaign. `.b.6.2.2`'s consumer captures `$head` *in the same branch* and doesn't depend on cross-rule ordering; later richer wirings may.
- **Audit pattern**: any consumer using `has_fact(<kind>, X)` where X could be forward-declared.

### G2. Priority-first branch reordering can change passing-construct shapes ⚠
- **Pattern**: adding a new priority-first branch (e.g. `.b.6.2`'s `context_member_method_call` in `call_primary`) is regression-safe **only if** the new branch's body requires enough syntactic structure to *only* match currently-failing inputs.
- **Guard used in `.b.6.2`**: `( dot identifier &dot )+` requires 3+ identifiers + a real `(args)` cmcb at the end; 2-level `a.b(x)` is structurally unreachable, no shape change.
- **Status**: ⚠ principle generalises — every future priority-first addition needs the same analysis.

---

## Cross-cutting observations

- **Compounding masking**: D1 + E2 + F1 + C1 compounded to make a 17-instance grammar defect entirely invisible until `.b.6.2.2`. Each layer hid the layer beneath. *Lesson:* when a class of defects is silent for long, suspect tooling/diagnostic gaps in tandem.

- **Two-frontier pattern**: as each blocker falls (A1, B1, B2, B3, D1), the parser advances and exposes the next blocker. The residual blockers are always at the *advanced* deep-parse frontier. **The campaign is genuinely progressing**; triage-position stasis is often a tooling artefact (F1), not parse stasis.

- **Audit patterns are the durable contribution.** A fixed defect that has no audit pattern is at risk of re-introduction. Every entry in this document should have one.

---

## Audit checklist — periodic mechanical sweeps

Run these periodically; failure of any check is a likely new instance of a known class.

| ID | Check (run from repo root) | Expected |
|---|---|---|
| A1 | `grep -nE '&[a-z_][a-z_0-9]* [a-z_][a-z_0-9]*' grammars/systemverilog.ebnf` | ≤1 hit (legit `&question question`) |
| A2 | `python3 tools/lrm_optional_audit.py` (from repo root) | 15 known-legitimate candidates as of 2026-05-23; any NEW candidate needs per-rule LRM review |
| B1 | `grep -nE '\( [a-z_]+ \. \)\*' grammars/systemverilog.ebnf` | each hit hand-checked for a `!`-guard at the loop boundary |
| C2 | `grep -nB2 -A8 -E 'mem::take\|mem::replace' rust/src/` | each hit: confirm no `?` before a manual restore (or IIFE wraps it) |
| D1 (SV) | `grep -nE '@predicate.*args: *\[.*\$[a-z_]+_identifier' grammars/systemverilog.ebnf` | 0 |
| D1 (other grammars) | same on `vhdl.ebnf`, `rtl_const_expr.ebnf`, `rtl_frontend.ebnf` | inventory needed |
| D2 | `grep -nE '\$[0-9a-z_]+\.body\b' grammars/systemverilog.ebnf` (inside `->` shapes) | each `body:` hit reviewed for "is this still a workaround for D1?" |
| C1 | diff `CompiledSemanticRuntimeAnnotations` struct fields against the `generate_compiled_…_tokens` emit-blocks | every field emitted |
| F2 | `stat -f %m target/release/parseability_probe` vs `stat -f %m generated/systemverilog_parser.rs` | probe > parser, or rebuild |

---

## Open items / next frontier

- **C1 (`predicate_defs` codegen)** — un-emitted; blocks composed-predicate dispatch in any generated parser. Owned by the first slice that uses `@predicate_def:` in a real grammar.
- ~~**A2** (`[ X ]` LRM-optional sweep)~~ — **CLOSED 2026-05-23**: oracle built (`tools/lrm_optional_audit.py`), all 15 candidates verified legitimate; the class is the LRM-notation ambiguity, mostly-mitigated, monitored via the oracle.
- **D1 in other grammars** — re-run the audit pattern on `vhdl.ebnf` / `rtl_*.ebnf`; the same shape of defect may exist.
- **F1 (PEG furthest-position tracker)** — engine proposal to expose the deepest-position-reached on failure, removing the need for bisection in most cases.
- **E1 / E2** — engine extensions if richer producer facts ever need structured-value refs or indeterminate predicates.

---

## Amendment log

| Date | Change | By |
|---|---|---|
| 2026-05-23 | Initial taxonomy created from SV-EXH-PROOF campaign through `.b.6.2.2` (commit `c766bc8b`). Classes A1, A2, B1, B2, B3, C1, C2, D1, D2, E1, E2, E3, F1, F2, F3, G1, G2. | `.b.6.2.X-TAXONOMY` |
| 2026-05-23 | A2 refined + downgraded to MITIGATED. Built oracle `tools/lrm_optional_audit.py`, verified all 15 candidate "drops" are legitimate (refactors / literal-bracket fixes); active grammar is LRM-faithful. Reformulated A2 as the LRM `[X]` / `{X}` ambiguity class with the oracle as the durable regression guard. | A2-CODIFY |

*Add a row per non-trivial amendment.*
