# GRAMMAR-WELLFORMED — the grammar linter PROVES an EBNF is well-formed; the stimuli generator constructively corroborates it

## Metadata

- Tree ID: `GRAMMAR-WELLFORMED`
- Status: `active` (director-commissioned 2026-06-05 from a design brainstorm)
- Roadmap lane: parser sign-off — grammar-correctness foundation; unblocks SV literal-0 (`SV-EXH-PROOF.7`)
- Created: `2026-06-05`
- Parser-AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]). Disciplines:
  [[feedback_no_codebase_change_without_tool_backed_facts]],
  [[feedback_corpus_expected_from_spec_not_fix]] (never game — prove, don't accept),
  [[feedback_research_grounded_sota_no_trial_and_revert]].

## The frame (the brainstorm — binding)

The grammar linter's **purpose is to PROVE an EBNF is well-formed / well-defined.** Not a bag
of warnings — a proof. **Well-formedness** ≜ the conjunction the linter must certify, per declared
profile:
1. **Terminating** — every rule derives a finite string (Ford PEG well-formedness; `non_terminating`).
2. **All-reachable** — every rule AND every branch is reachable from the entry rule.
3. **No dead branches** — no ordered-choice alternative is shadowed/subsumed by an earlier one
   (exact-duplicate AND FIRST-set domination).
4. **No dangling references / no profile orphans.**
5. **No nullable-repetition loops.**

**⚠️ LITERATURE-GROUNDED (director-required research, 2026-06-05) — the contract has TWO layers,
not one.** My first 5-condition list was unsourced + MISSED the entire SEMANTIC well-DEFINEDNESS
layer. The four foundational theories for PGEN's grammar class (a stateful/data-dependent PEG with
attribute-style annotations), each contributing one orthogonal axis:

**Well-FORMED (syntactic):**
1. **Reduced — no useless symbols** (Hopcroft–Ullman): every rule is **productive/terminating**
   AND **reachable** from the start symbol. [`detect_nonterminating_rules` ✅; reachability ⟶ A1b]
2. **PEG-complete — terminates on every input** (Ford, POPL 2004): no direct/indirect left
   recursion (PGEN eliminates ✅) + no nullable-repetition loop (`detect_nullable_repetition` ⚠️).
3. **No shadowed/dead branches** (PEG ordered-choice hygiene — branch-level useless symbols):
   exact-duplicate (`detect_ordered_choice_shadowing`, NOW a HARD gate — A1a ✅) + FIRST-domination
   (⟶ A2).
4. **No dangling refs / no profile orphans** (PGEN profile layer): `detect_profile_orphans` ✅ hard.

**Well-DEFINED (semantic) — NEW, the gap the research exposed:**
5. **Attribute non-circularity** (Knuth 1968; circularity is intrinsically exponential, Jazayeri et
   al. CACM 1975, but feasible in practice): no return-annotation / `@semantic_value` attribute may
   depend transitively on itself. ❌ PGEN does not check this — `-> {$1,$2}` / `@semantic_value`
   ARE an attribute grammar. ⟶ E1.
6. **Attribute completeness** (modular well-definedness): every referenced attribute/binding (`$N`,
   a consulted fact) has a defining source. ❌/⚠️ ⟶ E2.
7. **Data-dependent binding-before-use** (Jim, Mandelbaum, Walker, POPL 2010): every `@predicate`
   references only facts establishable EARLIER in some parse (no predicate gated on a fact nothing
   can emit before it — the static twin of the C2 semantic-prelude reach). ❌ ⟶ F1.

**Theorem (the consequence):** a well-formed AND well-defined grammar ⟹ every rule/branch has a
reaching witness. Reachability is a *definitional consequence*, not a separate thing to chase.

**Completeness claim (honest):** these four theories ARE the established foundations for exactly
PGEN's grammar class (CFG hygiene + PEG + attribute + data-dependent), each an orthogonal axis; I
know of no fifth axis for this class — so this is the complete contract *for this class*, revisable
if a new axis surfaces. Sources: Ford POPL 2004 (peg.pdf); Medeiros et al. (arXiv 1207.0443);
Hopcroft–Ullman (useless symbols / reduced grammar); Knuth 1968 + Jazayeri et al. CACM 1975
(attribute circularity); Jim/Mandelbaum/Walker POPL 2010 (data-dependent grammars).

**The DUALITY (two independent proofs of the same reachability property):**
- the **linter** proves reachability **statically** ("a witness EXISTS for every branch");
- the **stimuli generator** proves it **constructively** ("here IS the witness for every branch").
They must AGREE. Disagreement localizes the bug:
- linter-reachable but generator-can't-witness → a **constructor** bug (never an accepted dead end);
- generator-covers but linter-called-unreachable → a **linter** bug.
Both green on the same grammar ⟹ reachability proven twice (static + constructive) = signoff-grade.
**Literal-0 stimuli coverage stops being a goal in itself — it is the OBSERVABLE CONSEQUENCE of
"linter proves well-formed" + "generator constructively confirms it."**

**The ATTRIBUTION RULE (director directive, 2026-06-06 — binding; the operational form of the
duality).** When the stimuli generator FAILS TO REACH a target (branch / rule / EBNF fragment) there
are EXACTLY TWO possible causes, and it MUST be attributed to one — NEVER silently accepted as a
residual: (1) **generator deficiency** — the target IS reachable → improve the generator; or (2)
**EBNF not well-formed** — the target is genuinely unreachable (dead branch/rule) → fix the GRAMMAR.
The **linter is the ADJUDICATOR**: for the decidable cases it PROVES which; the undecidable remainder
is flagged loudly on that exact target for manual adjudication (still never silently accepted).
⚠️ ORDER OF SUSPICION = **GRAMMAR-FIRST**: an unreachable target is FIRST a well-formedness signal, NOT
evidence the generator is weak (this inverts the historical SV literal-0 instinct of chasing the
generator). Run the linter BEFORE adding generator machinery — chasing a linter-proven-dead branch in
the generator is wasted effort that can never succeed. ⇒ literal-0 is a THEOREM (both proofs agree for
every target), and every uncovered target is a TICKET (grammar or generator), not a shrug. FIRST
WORKED EXAMPLE: the boolean-abbrev `?`-per-arm bug (A2.1, below) — generator couldn't reach 7 branches
→ linter PROVED them unreachable → blame = EBNF → grammar fixed. Composes with
[[feedback_prefer_grammar_leave_engine_alone]] (grammar-first is also the cheaper, more-correct fix).

**Decidability boundary (honest):** the linter proves the DECIDABLE core; the generator's
construction extends the proof into the undecidable region (it witnesses what static analysis
can't decide) and corroborates the rest. A branch neither linter-provable-unreachable nor
generator-witnessable is a LOUD, specific flag ("this exact branch — fix the constructor or it's a
subtle dead branch"), never a silent accept.

## Capability audit (verified in code 2026-06-05)

**Q1 — well-formedness / "no unreachable":**
| Capability | Status |
|---|---|
| Terminating (`detect_nonterminating_rules`) | ✅ detector + lint (error) |
| Structural reachability (rule from entry) | ⚠️ COMPUTED in coverage (`unreachable_rules`/`unreachable_from_entry`) but NOT a `--lint-grammar` gate |
| Missing-reference | ✅ detected (`missing_rule_references`; universe excludes) |
| Exact-duplicate shadowing | ⚠️ `detect_ordered_choice_shadowing` — WARNING only (not a hard gate) |
| FIRST-set DOMINATION shadowing | ⚠️ FIRST-set machinery exists (`FirstSetSummary`/`branch_first_set`/`W_GRAM_FIRST_SET_OVERLAP`) but as an OVERLAP warning, not domination/unreachability |
| Semantic-precondition unsatisfiable | ❌ missing (no fact-flow) |
| profile_orphans | ✅ HARD gate (this session) |

**Q2 — reach / construction:**
| Capability | Status |
|---|---|
| Reach-path graph + branch-forcing | ✅ `reach_plan`/`set_reach_plan` |
| Min-terminal-length off-path filler + `construct_mode` | ✅ (`.7.4.2`, `.7.4.6.3`) |
| Defeat-earlier-branch crafting | ❌ missing (forces branch i but doesn't make earlier branches FAIL → replay may pick an earlier branch) |
| Semantic-prelude reach | ❌ missing — **the generator is SEMANTICS-BLIND** (no `@emit_fact`/`@predicate` modeling during generation); likely the dominant SV residual driver |
| Bounded-ordered backtracking | ❌ missing (dead-ends fall back to the timed search) |
| Deterministic budget | ❌ missing (`generation_deadline_exceeded` uses `Instant::now()` wall-clock = the non-determinism) |

## Ordered build list (A makes 0 possible, B makes it measurable, C makes it happen)

### Phase A — make the linter a well-formedness PROVER (the universe becomes provably clean)
- `A1a` — **DONE (PGEN-GRAMMAR-WELLFORMED-0001):** exact-duplicate shadowing is now a HARD `--lint-grammar`
  gate (was warning), alongside `profile_orphans`/`non_terminating`. SV passes (shadowing=0 after the
  de-dup); regex (1) + semantic_annotation (3) now correctly FAIL the gate (surfaced TODOs — the
  lint is opt-in, not in CI, so no build breakage). Report labels corrected (shadowing/orphans=error).
- `A1a.1` / `A1a.2` — **DONE (PGEN-GRAMMAR-WELLFORMED-0002):** cleaned regex (1: a duplicate `'^'` in
  `directive_special`) + semantic_annotation (3: duplicate annotation-name literals `interface`,
  `contract`, `feature` in `predefined_annotation`). Both now `--lint-grammar` exit 0 (shadowing=0).
  Parse-neutral (exact-dup alternatives never fired); regen + lib generated_parsers 652/0.
  **⇒ ALL authored grammars now pass the shadowing hard gate** (the A1a gate is fully green).
- `A1b` — **DONE (PGEN-GRAMMAR-WELLFORMED-0003):** structural unreachability is now a HARD
  `--lint-grammar` gate. `detect_unreachable_rules` (grammar_wellformedness.rs): roots = `rule_order[0]`
  ∪ every unreferenced rule (a secondary entry, e.g. `sv_multi_entry_root` which unions in
  `systemverilog_file`/`library_text`/`systemverilog_parseable_file`), reachability = transitive
  closure. MULTI-ENTRY-SAFE (the unreferenced `sv_multi_entry_root` is a root → no false positives;
  CONSERVATIVE: catches referenced-but-unreachable dead ISLANDS; an unreferenced dead orphan is
  treated as a root → not flagged, a safe false-negative). Unit-tested (dead-island + multi-entry
  safety). VERIFIED: SV unreachable_rules=0 (multi-entry handled correctly), ALL 10 authored grammars
  =0, SV lint exit 0; lib (no-features) grammar_wellformedness 17/0. Follow-up `A1b.1`: catch
  unreferenced dead orphans (needs an entry-declaration so an orphan ≠ a secondary entry).
- `A2` — **DONE (PGEN-GRAMMAR-WELLFORMED-0006):** the SOUND DECIDABLE SUBSET of FIRST-domination —
  **earlier-branch-ALWAYS-SUCCEEDS shadowing.** New `node_always_succeeds`/`compute_always_succeeds`
  (the dual of `compute_nullable`, differing ONLY on the lookahead arm: a predicate `&e`/`!e` is
  nullable but CAN FAIL, so it is NOT always-succeeds → no false positive). In an ordered choice, an
  earlier alternative that always succeeds (`e?`, `e*`, an all-optional sequence, or a ref to such a
  rule) makes every later alternative provably dead (PEG commits to the first success).
  `ShadowingReason::EarlierAlwaysMatches` + `is_hard_gate()`. CONSERVATIVE (only flags PROVEN
  always-success → never false-accuses a live branch). Unit-tested (positive: `e?`/`e*`/all-optional/
  nullable-ref; negative: lookahead earlier branch does NOT shadow). VERIFIED across all 17 grammars:
  ZERO false positives on the clean authored grammars; it found **52 REAL dead branches in
  `systemverilog.ebnf`** (the recurring `( X )?`-as-an-alternative anti-pattern — e.g.
  `consecutive_repetition`, `covergroup_value_range_sv_2023`, `bins_or_empty`, `boolean_abbrev_*`).
  STAGED AS A WARNING (not yet the hard gate) — mirrors how exact-dup shadowing was staged before
  A1a promoted it; SV stays `--lint-grammar` exit 0 (hard gate = exact-dup + fixed-prefix, both 0)
  while the 52 are a loud `always_matches_shadowing` backlog. The general (UNSOUND) FIRST-domination
  is deliberately NOT added: FIRST(a)⊇FIRST(b) does NOT imply a shadows b in PEG (a may match the
  first token then fail, after which b IS tried) — adding it would false-accuse live branches,
  violating "never game / never falsely reclassify". The two sound forms (fixed-terminal-prefix +
  always-succeeds) are the decidable, zero-false-positive core.
- `A2.1` — **FRONTIER: clean the 52 SV `always_matches_shadowing` defects LRM-grounded, then promote
  EarlierAlwaysMatches to the hard gate.** Each is a latent (not active) defect — the dead branch
  never fires today because alt #0 always wins, so a fix CHANGES parse behavior and MUST be verified
  parse-neutral (or parse-IMPROVING, LRM-grounded) against the SV corpus + the global stimuli metric,
  ONE change/family at a time ([[feedback_no_codebase_change_without_tool_backed_facts]], never derive
  expecteds from the fix). When warnings reach 0, flip
  `ShadowingReason::EarlierAlwaysMatches::is_hard_gate()` → true. *Effort: high (multi-slice).*

  **Investigation (2026-06-06, tools-first) — 26 rules / ~7 fix families:**
  1. **boolean-abbrev family** — **DONE (PGEN-GRAMMAR-WELLFORMED-0010, leaf A2.1.1).** Dropped the
     spurious `?` on each arm of `consecutive_repetition` (3 arms), `non_consecutive_repetition_sv_2017`,
     `nonconsecutive_repetition_sv_2023`, `goto_repetition` (kept the `( )` group so `-> {range:$1}` is
     preserved). LRM-grounded (IEEE 1800 §16.9.2 — `[*n]`/`[*]`/`[+]` are required forms; the single
     optionality lives at the caller `( boolean_abbrev )?` / `( sequence_abbrev )?`, verified across ALL
     call sites). The spurious `?` is almost certainly an LRM-PDF→.ebnf extraction artifact (the
     `( X )?`-per-arm signature is too regular to be hand-authored). VERIFIED: lint always_matches
     **52→45** (the 7 boolean-abbrev findings resolved, NO new shadowers exposed because all three
     sub-rules were fixed together); SV parser regenerated fresh (mtime confirmed) + strict annotation
     validation passed; regenerated parser COMPILES clean (release); common-case sequence (no abbrev)
     still parses; the one ad-hoc parse failure seen was an UNRELATED port-list issue (Family 4),
     pre-existing, not in the official corpus. Full official corpus + closed-loop global metric =
     deferred milestone verification (director: "we'll see later"). THE FIRST WORKED EXAMPLE OF THE
     ATTRIBUTION RULE (generator-couldn't-reach → linter-proved-unreachable → grammar fixed).
     ⚠️ SEPARATE FINDING (logged, NOT part of this fix): this grammar models SVA repetition BRACKET-LESS
     (`star const_or_range` = `*N`, not `[*N]`) — so real SV `a[*3]`/`a[*]`/`a[+]` still fail to parse
     (verified: `[*3]` rejected; no-abbrev sequence parses → no regression). Missing `[`/`]` around the
     boolean/sequence-abbrev is a distinct COMPLETENESS gap (likely the same extraction artifact dropping
     brackets) → its own ticket; orthogonal to the dead-branch `?` fix. Original:
     `consecutive_repetition := ( star const_or_range )? | ( star )? | ( plus )?` — each alt is
     `( X )?` so the rule always-succeeds (matches empty), killing `[*]`/`[+]` AND shadowing
     boolean_abbrev's later branches. ROOT = the spurious `?`; LRM-grounded fix (IEEE 1800 §16.9.2,
     `[*n]`/`[*]`/`[+]`) = drop the `?` (make each required). ⚠️ MUST fix `non_consecutive_repetition_sv_2017`,
     `nonconsecutive_repetition_sv_2023`, `goto_repetition` IN THE SAME SLICE (all are `( X )?` too —
     fixing only `consecutive_repetition` just exposes the next as the shadower). ⚠️ ANNOTATION RISK:
     the `?`-groups carry `-> {range: $1}`; removing `?` while keeping the `( )` group preserves `$1`,
     but VERIFY with `parseability_probe --parse-dump-ast-pretty` ([[feedback_ebnf_consult_annotation_docs]]).
  2. **covergroup-range family** (`covergroup_value_range` #1, `covergroup_value_range_sv_2023` #2#3#4):
     alt #1 = `( expr : expr )?` always-succeeds → dollar/tolerance variants dead. Same `?`-drop fix
     (LRM §A.9.3 value ranges).
  3. **rs-prod family** (`rs_prod_sv_2017` #2#3#4, `rs_prod_sv_2023` #2#3): alt #1 always-matches.
  4. **implicit-type/port family** (`let_formal_type`, `property_formal_type`, `net_port_type`/`_sv_2017`/
     `_sv_2023`, `port`, `ansi_port_declaration`): alt #0 = a `data_type_or_implicit`-like rule that
     always-succeeds (implicit type matches empty) → trailing keyword/variant branches dead. Needs
     care: the always-succeed may be LEGITIMATE (implicit type) → the later branch may be the genuine
     dead one to RESTRUCTURE, not a simple `?`-drop. Per-rule LRM analysis required.
  5. **list-of-arguments family** (`list_of_arguments` #1#2, `let_list_of_arguments`,
     `property_list_of_arguments`, `list_of_checker_port_connections`, `class_constructor_super_args`,
     `class_constructor_arg_sv_2023`): all-optional element list always-succeeds.
  6. **module-path family** (`module_path_primary` #3#4#5, `module_path_mintypmax_expression`).
  7. **`sv_multi_entry_root`** (#1#2) — NOT a grammar defect: it is the A1b reachability MARKER
     (an unreferenced union of `systemverilog_file`|`library_text`|`systemverilog_parseable_file` so the
     latter two count as reachable roots). `systemverilog_file` is `*`-based (always-succeeds), so the
     later alts are unreachable AS A PARSED ordered choice — but the rule is never PARSED. Resolution:
     EXEMPT marker rules (e.g. recognize the `*_multi_entry_root` role / an `@entry`-style marker), do
     NOT "fix" by grammar edit. This is the one case to handle in the LINTER, not the grammar.

### Phase B — make the constructive proof deterministic (the count becomes signal)
- `B1` — **DONE (code, PGEN-GRAMMAR-WELLFORMED-0005; gate-residual confirm in flight):** replaced the
  wall-clock generation deadline with a DETERMINISTIC step counter. `GenerationTimeoutBudget` and
  `ActiveGenerationDeadline` now carry a step budget/deadline (not `Duration`/`Instant`); a
  monotonic `generation_step_counter: Cell<u64>` bumps once per `generation_deadline_exceeded`
  check (called at every rule/node chokepoint via `enforce_generation_deadline`); the ms config maps
  to steps via `generation_steps_per_ms()` (env `PGEN_GENERATION_STEPS_PER_MS`, default 1000) so the
  gate's existing ms budgets keep working but the cutoff is machine-INDEPENDENT. `std::time::{Duration,
  Instant}` removed from the generator → the only non-seeded input is gone ⇒ a seeded run yields the
  SAME residual every time (the literal-0 measurement prerequisite). VERIFIED: lib (no-features)
  591/0; the two timeout-abort unit tests pass with `step_budget: 0`; **canonical gate run TWICE →
  closed_loop_replay_targets_total = 84 BOTH times (DETERMINISTIC — the ±25 wobble of 97/89/105/120
  is GONE)**, gate passes both, realistic corpus green. 84 < the old noisy band → the `.7.4.6.7`
  de-dup's effect is now measurable. THE LITERAL-0 METRIC IS NOW SIGNAL, NOT NOISE — the constructive
  half of the duality can now be driven + measured deterministically.
- `B2` — replace the timed search-fallback with BOUNDED-ORDERED backtracking (next-shortest sibling at
  the last choice point, depth-bounded). *Effort: medium.*

### Phase C — complete the constructor (witness every reachable branch)
- `C1` — **defeat-earlier-branch crafting**: when forcing branch i, choose content diverging from
  earlier branches' FIRST-sets so the parser SELECTS i on replay. *Effort: med-high. Reuses A2.*
- `C2` — **semantic-prelude reach** (the deepest gap): model the store during construction; for a
  `@predicate`-gated target, emit the `@emit_fact` prelude first (e.g. a `typedef` before the
  type-position use), sequencing prelude→target. *Effort: high; generator currently semantics-blind.*

### Phase D — well-DEFINEDNESS (the semantic layer the literature exposed; NEW)
- `E1` — **attribute non-circularity** (Knuth 1968) — **DONE / SATISFIED BY CONSTRUCTION
  (PGEN-GRAMMAR-WELLFORMED-0004, analysis).** Tools-first source check: PGEN's return annotations are
  PURELY SYNTHESIZED — `UnifiedReturnAST::PositionalRef { index }` (`$N`/`$0` = the rule's OWN
  children/match, bottom-up) plus literals/access/spread; there is NO inherited/parent/sibling
  attribute construct in the annotation language (grep: zero inherited refs — only test strings). A
  synthesized-only attribute grammar is **non-circular by construction** (a cycle requires inherited
  attributes feeding back up; Knuth 1968). ⇒ E1 holds STRUCTURALLY — no runtime cycle is possible,
  so no detector is needed; the proof is the annotation-language design. (The store flow
  `@predicate`/`@emit_fact` is DATA-DEPENDENT, a SEPARATE axis = `F1`, not classic attribute
  circularity.) If an inherited construct is ever added, Knuth's bounded cycle test becomes required
  (re-open E1).
- `E2` — **attribute completeness** — **DONE / SATISFIED BY EXISTING VALIDATION
  (PGEN-GRAMMAR-WELLFORMED-0007, analysis).** The synthesized-attribute half — every `$N` has a
  defining source — is ALREADY enforced: `annotation_validator.rs` emits `E_RET_POS_OUT_OF_RANGE`
  (`AnnotationSeverity::Error`) for a positional ref with no defining child capture, plus the bound
  checks `W_RET_BRANCH_INDEX_OOB` / `W_RET_POS_RULE_BOUND` / `W_RET_BRANCH_NOT_SEQUENCE`. It is HARD:
  `ast_generator_direct.rs:152` aborts parser generation when `strict_validation && has_errors()`
  (strict mode is the CI default / `PGEN_STRICT_ANNOTATION_VALIDATION=1`), and it is regression-
  locked by `return_validator_honors_capture_index_bounds` + `grammar_aware_validation_warns_when_
  positional_ref_exceeds_branch_bound`. PGEN's return AST is dynamic JSON (no static field-existence
  on `$1.field`), so the only DECIDABLE completeness condition for synthesized attributes is the
  positional-binding range — which is enforced. The CONSULTED-FACT half ("a `@predicate`'s fact kind
  has an emitting source") is the sound core of `F1` (done there) — it is a data-dependent, not a
  synthesized-attribute, condition. ⇒ E2 holds for its decidable synthesized-attribute scope with no
  new code; the fact scope is F1.
- `F1` — **data-dependent binding-before-use** (Jim et al. 2010) — **DONE (PGEN-GRAMMAR-WELLFORMED-0008),
  HARD GATE.** Sound decidable core: a `@predicate` that consults a fact-KIND no `@emit_fact` ever
  emits. `new detect_unbound_fact_kinds(annotations)`: enumerates EMITTED kinds (every `@emit_fact`
  carries a literal `kind`) across ALL annotation surfaces (rule-level + per-branch + mid-sequence —
  complete, the soundness requirement) and CONSULTED kinds from each `@predicate` (parsing the inline
  expression via `parse_predicate_expression` + the structured form, walking has_fact / lacks_fact /
  fact_attribute_equals / fact_count_at_least — the four primitives whose `args[0]` is a kind and which
  query the exact `fact_index` `@emit_fact` populates). A consulted kind ∉ emitted ⇒ the fact can never
  be established (has_fact always-false / lacks_fact always-true) = binding-before-use. CONSERVATIVE on
  the consult side (literal kinds only; dynamic/arg-ref kinds skipped → under-report, never
  false-accuse). VERIFIED: 0 across ALL 17 grammars (SV's consulted {type_name, variable_binding} ⊆
  emitted {type_name, variable_binding, checker_name, let_name, package, …}); SV `--lint-grammar` exit
  0. Unit-tested (`detects_unbound_fact_kind_but_not_bound_one` + `unbound_fact_kind_skips_dynamic_kinds_
  and_finds_branch_emitters`). The undecidable refinement — per-NAME + parse-ORDER reachability
  ("establishable EARLIER in some parse") — is deliberately NOT attempted; the kind-existence core is
  the sound decidable subset (cf. A2's exclusion of unsound general FIRST-domination).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `GRAMMAR-WELLFORMED.A1a` | `done` (`-0154`) | Shadowing now a hard gate; SV well-formed re: dead branches; embodies "a well-defined EBNF has no unreachable rules". |
| — | `GRAMMAR-WELLFORMED.A1a.1/.2` | `done` (`-0002`) | regex + semantic_annotation shadows cleaned → ALL authored grammars pass the shadowing hard gate. |
| — | `GRAMMAR-WELLFORMED.A1b` | `done` (`-0003`) | Structural unreachability now a hard, multi-entry-safe gate; all grammars =0. The headline "no unreachable rules" is enforced. |
| — | `GRAMMAR-WELLFORMED.E1` | `done` (`-0004`, satisfied by construction) | Attribute non-circularity holds structurally (synthesized-only annotation language). |
| — | `GRAMMAR-WELLFORMED.B1` | `done` (`-0005`) | Deterministic step-budget replaces the wall-clock deadline → residual = 84 IDENTICAL across two runs (the ±25 noise gone). The literal-0 metric is now signal. |
| — | `GRAMMAR-WELLFORMED.A2` | `done` (`-0006`) | Sound subset of FIRST-domination — earlier-ALWAYS-SUCCEEDS shadowing. 0 false positives; found 52 real SV dead branches (warning-staged). General unsound FIRST-domination deliberately excluded. |
| — | `GRAMMAR-WELLFORMED.E2` | `done` (`-0007`, satisfied by existing validation) | `$N` attribute completeness already enforced (`E_RET_POS_OUT_OF_RANGE`, hard under strict mode, test-locked); consulted-fact completeness → F1. |
| — | `GRAMMAR-WELLFORMED.F1` | `done` (`-0008`, HARD GATE) | Binding-before-use (Jim 2010) — consulted-but-never-emitted fact-KIND. 0 across all grammars (sound, zero FP). **⇒ the well-DEFINEDNESS layer (E1/E2/F1) is COMPLETE; the linter now proves all 7 contract axes' decidable cores.** |
| 1 | `GRAMMAR-WELLFORMED.A2.1` | `in-progress` (family 1/7 done `-0010`; always_matches **52→45**) | Clean the SV `always_matches` defects LRM-grounded, family by family → promote EarlierAlwaysMatches to the hard gate. ✓ boolean-abbrev family (first worked example of the ATTRIBUTION RULE). Remaining: covergroup-range, rs-prod, implicit-type/port, list-of-arguments, module-path, sv_multi_entry_root (linter-exempt). |
| 2 | `GRAMMAR-WELLFORMED.B2/C1/C2` | `pending` | The CONSTRUCTIVE side (stimuli generator): bounded-ordered backtracking, defeat-earlier-branch crafting, semantic-prelude reach. Riskier (touch generator runtime; measure the global metric). |

## Decisions
- `2026-06-05`: Created from the director brainstorm. The frame UNIFIES the linter (static proof) +
  the stimuli generator (constructive proof) of reachability. Cross-refs: `PARSE-SOTA` (existing
  lint checks A1/.9), `SV-EXH-PROOF.7` (the generator/literal-0 consumer), `PARSE-TERMINATION`
  (`non_terminating`). The de-dup of 25 dead branches (`SV-EXH-PROOF.7.4.6.7`) was the first
  embodiment; this tree generalizes it into a proof.
