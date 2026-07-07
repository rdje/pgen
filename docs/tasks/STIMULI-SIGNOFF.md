# STIMULI-SIGNOFF: Signoff-Grade EBNF Stimuli Generator (capability-gap closure)

## Metadata

- Tree ID: `STIMULI-SIGNOFF`
- Status: `active`
- Roadmap lane: `Stimuli generator → best-in-class / signoff-grade (user vision 2026-05-31)`
- Created: `2026-05-31`
- Last updated: `2026-07-07`
- Owner: repo-local workflow

## Goal

Make PGEN's EBNF-based stimuli generator (`rust/src/ast_pipeline/stimuli_generator.rs`)
signoff-grade / best-in-class / "breathtaking" per the user vision
([[project_stimuli_generator_signoff_vision]]): a deliberate capability-gap
audit of what a signoff EBNF stimuli generator SHOULD do that ours doesn't yet,
then close those gaps as GENERAL, parser-agnostic grammar-structure capabilities.

## Non-Goals

- No hardcoded rule-names/sigils — every capability is a GENERAL grammar-structure
  property ([[feedback_ast_pipeline_parser_agnostic]]).
- Not a rewrite for its own sake — gaps must be evidenced (e.g. by the
  `SV-EXH-PROOF.7` `focused_replay_target_debt` data).
- Correctness before speed ([[feedback_correctness_before_speed]]).

## Acceptance Criteria

- A capability-gap audit (what a signoff generator should do vs. ours) recorded.
- Each accepted gap → a leaf; each landed as a parser-agnostic, signoff-verified
  capability (corpus/gate-proven, no regression across families).
- Candidate axes: guaranteed/exhaustive coverage closure (not best-effort) with
  machine-checkable proof; depth strategy reaching deep-but-rare branches;
  deterministic target-reach/path-forcing; semantic-store-aware valid-by-
  construction generation; counterexample shrinking; negative/invalid stimuli;
  scale/perf; targeted generation; in-memory↔generated-module parity.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `STIMULI-SIGNOFF`
  Status: `active` (`.1` audit DONE 2026-06-03; 6 gaps → leaves `.2`–`.7`; the 2026-07-01 `-0005` survey's 4 gaps + 1 de-smell folded in as `.8`–`.12` on 2026-07-07)
  Goal: `Signoff-grade, parser-agnostic EBNF stimuli generator via capability-gap closure.`
  Children: `.1` audit → `.2` k-path coverage · `.3` code-coverage feedback · `.4` directed/learned generation · `.5` uniform/Boltzmann · `.6` grammar-tree-aware shrinking · `.7` mutation-maturity metric · `.8` CIT/n-wise · `.9` metamorphic+differential oracles · `.10` swarm · `.11` boundary-value-as-goal · `.12` quantified-separator de-smell

- ID: `STIMULI-SIGNOFF.1`
  Status: `done` (`PGEN-STIMULI-SIGNOFF-0001`, 2026-06-03)
  Goal: `CAPABILITY-GAP AUDIT (pure docs): enumerate, out-of-the-box, what a signoff EBNF stimuli generator SHOULD do that ours doesn't yet; seed it with the SV-EXH-PROOF.7 reachability finding ([[project_sv7_never_selected_rootcause]]). Produce a prioritized gap list → ordered leaf plan. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Prioritized capability-gap list + ordered leaf plan recorded; each gap tagged parser-agnostic-by-design.`
  Verification: `done — audit from the live literature sweep (2026-06-03) + the code-verified generator feature surface. HAS (grep-verified in stimuli_generator.rs): Purdom shortest-derivation (min_terminal), rule+branch coverage targets + gap report + reach_classification (StimuliCoverageTarget), directed reach plans (reach_plan, SEARCH-based), constraint/negative/recovery profiles, delimiter-aware shrinking, closed-loop round-trip self-consistency (parser_rejections==0), and — AHEAD of typical academic fuzzers — semantic-store-aware (data-dependent) generation = context-VALID inputs (declare-before-use). SIX GAPS vs the literature signoff bar → ordered leaves .2-.7. Full reasoning + citations in KM card [[stimuli-generator-capability-gaps]]. Pure docs.`
  Commit: `PGEN-STIMULI-SIGNOFF-0001`

- ID: `STIMULI-SIGNOFF.2`
  Status: `done` (gap #1 capability CLOSED) — `.2.1` universe + `.2.2` numerator + `.2.3` usable report; full closed-loop-gate integration = optional follow-up `.2.4`
  Goal: `k-PATH COVERAGE METRIC (Havrikov & Zeller, ASE 2019; tool Tribble). Generalize coverage from rule+branch (≈k=1/2) to a chosen-k path measure (a syntactic element in the context of its depth-k ancestors). This DEFINES "exhaustive coverage" more rigorously than rule+branch — "every depth-k context combination covered". Parser-agnostic grammar-structure property; re-express the SV residual as k-path debt.`
  Acceptance: `a k-path measure computed over any grammar; the signoff bar restated in k-path terms; SV residual re-expressed. Code leaf, tools-first.`
  Verification: `.2.1 DONE (PGEN-STIMULI-SIGNOFF-0002) — the coverage UNIVERSE (denominator). Implemented compute_k_paths(k) in stimuli_generator.rs (PURE analysis, no generation change, like .7.4.2's min-length table): enumerates length-k chains of nonterminals over the rule-reference graph (reuses collect_rule_references), deterministic (rule_order + sorted successors). k=1 = rules (≈ our rule coverage); k=2 = reference edges (≈ branch coverage); k>=3 = the deeper ancestor-context combinations our rule+branch metric does NOT measure. Unit-tested on synthetic_reach_grammar (k=1→6, k=2→5, k=3→3 incl. start->mid_b->deep, k=4→0, determinism). lib 585/585; source-strict clippy 0; #[allow(dead_code)] until the .2.2 caller. .2.2 DONE (PGEN-STIMULI-SIGNOFF-0003) — the NUMERATOR: k_path_recording field (default-OFF → byte-identical/monotone generation) records the last-k call_stack window at each generate_rule entry (read-only instrumentation, never changes a decision); covered_k_paths() + k_path_coverage() = (covered∩universe, |universe|). Unit-tested (covered ⊆ universe, a start-edge always covered, default-off). lib 586/586; clippy 0. So the k-path MEASURE is COMPLETE (universe + numerator + coverage). .2.3 DONE (PGEN-STIMULI-SIGNOFF-0004) — the USABLE surface: pub k_path_coverage_report(entry, samples, k) + a `--report-k-path-coverage K` CLI mode (mirrors --lint-grammar; read-only, generation unchanged). RAN on real SV (sv_2017, entry systemverilog_file, 30 diverse samples, k=2): **universe = 4105 k-paths; covered 679 (16.5%)** — the first k-path measurement on SV (the signoff-bar denominator is now a concrete number; the residual is re-expressible as k-path debt). lib 586/586; source-strict clippy 0; binary builds. So GAP #1 (k-path coverage) capability is CLOSED: universe + numerator + usable report. NOTE: k-path count grows combinatorially → bound k (2-3) on SV. .2.4 OPTIONAL (follow-up): wire the recorder into the full closed-loop SV gate (target-drive + witness passes, not just the diverse pass) for the definitive closed-loop k-path coverage + formally restate "literal-0" as chosen-k coverage.`
  Commit: `PGEN-STIMULI-SIGNOFF-0002 (.2.1) + -0003 (.2.2) + -0004 (.2.3)`

- ID: `STIMULI-SIGNOFF.3`
  Status: `pending` (gap #4 — pairs with .2)
  Goal: `CODE-COVERAGE FEEDBACK (coverage-guided grammar fuzzing). Close the loop on the PARSER's actual code coverage (Havrikov-Zeller thesis: input k-path coverage drives code coverage): use generated-input code coverage to steer generation. Today PGEN is grammar-coverage-driven ONLY — the deepest signal that we exercised the parser, not just enumerated the grammar, is missing. Parser-agnostic (instrument the generated parser).`
  Acceptance: `generation steered by measured parser code coverage; coverage delta reported. Code leaf.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.4`
  Status: `active` (design recorded 2026-07-07, `PGEN-STIMULI-SIGNOFF-0009`; next = `.4.1`)
  Goal: `DIRECTED / LEARNED GENERATION (FDLOOP, Kirschner & Soremekun, arXiv 2508.01472, 2025) — goal-directed generation via probabilistic-grammar LEARNING + a feedback loop. CO-OWNED: SV-EXH-PROOF.7.4.6 is the SV application; this leaf is the GENERAL parser-agnostic capability. FDLOOP read in full 2026-07-07 (research-grounded doctrine).`
  Design: `FULL research read-out + reconciliation + worked symbol-cited mapping + slice plan: docs/tasks/STIMULI-SIGNOFF-4-fdloop-directed-generation-design.md. CONDENSED: FdLoop = per-generation loop (learn per-production probabilities from derivation trees → generate n from the learned grammar + n validity-preserving mutants → execute + collect goal metrics → select the single best by weighted-sum fitness → re-learn from the selected set + reset random rules to uniform for exploration). RECONCILIATION (mandated): the leaf's original "SV residual 273 → ~0" acceptance is PARTIALLY SUPERSEDED — the .7.4.6 arc landed deterministic derivation-directed CONSTRUCTION (273→97→~84 best-known) and .7.4.6.7 proved the residual metric NOISE-DOMINATED (±~25; runs 97/89/105/120), so (a) single-target reach is already better-served deterministically, (b) the SV metric must first be made deterministic (SV-EXH-PROOF.7.4.6.8, owned there) before any FdLoop measurement against it. The surviving value = the GENERAL capability PGEN lacks: (1) a LEARNED per-choice-point distribution layer (HashMap<"{rule}::{node_path}", Vec<u64>> composed into generate_or phase-2 weighting alongside declared probabilities [strip_probability_prefix :10381, build_weights :10444] and coverage_guidance_multiplier [:10702]; absent map = byte-identical, the .12 default-OFF pattern); (2) learning front-ends: self-derivation counting (per-sample selection log; free attribution) + EXTERNAL-corpus learning via the gen-AST interpreter (parse_harness_interpreter walks the same IR and knows (rule,node_path,branch) natively — the parser-agnostic derivation counter); (3) the directed loop driver grafted onto run_coverage_guided_fuzz_loop (main.rs:4099): fitness → select-best-k → re-learn → seeded-RNG distribution mutation (StdRng :1638; deterministic per seed, 0/7/42 triplicate). GOAL VOCABULARY v1: G1 k-path coverage delta (primary; the .2 recorder; SV sv_2017 k=2 baseline 679/4105 = 16.5% is the ready-made before→after), G2 parse-failure revelation (duality-break hunter feeding the shrinker), G3 corpus-mimicry (learn from a real corpus, generate distributionally-similar inputs — NEW capability direction, surfaced to director), G4 parser code coverage (DEFERRED to .3; plug-in point designed). Parser-agnostic throughout (structural keys only, zero name literals).`
  Acceptance: `(1) learned-distribution layer default-OFF byte-identical (seeds 0/7/42 proof across shipped grammars); (2) deterministic directed loop driving goal G1 with a MEASURED k-path coverage improvement over the diverse pass on ≥2 grammars (SV from the 16.5% baseline); (3) external-corpus learning via the interpreter front-end with a measured mimicry demonstration; (4) parser-agnostic (structural keys only); (5) SV residual lane re-measured ONLY against the deterministic metric once SV-EXH-PROOF.7.4.6.8 lands (never chase the ±25-noisy number).`
  Children: `.4.1` learned-distribution layer + self-derivation learning · `.4.2` directed-loop driver (goal G1, k-path) · `.4.3` external-corpus learning + G3 mimicry · `.4.4` G2 duality-break hunter · `.4.5` SV lane measurement (post-.7.4.6.8)
  Verification: `design slice done (docs-only, 2026-07-07 session #58): FDLOOP read fully via the arXiv HTML; .7.4.6 reconciliation recorded (273→~84, noise-dominated metric, construction supersedes reach — the general probabilistic-learning capability is the remaining value); worked mapping symbol-cited against stimuli_generator.rs/main.rs at HEAD. Implementation pending (.4.1 next).`
  Commit: `PGEN-STIMULI-SIGNOFF-0009 (design; docs-only)`

- ID: `STIMULI-SIGNOFF.5`
  Status: `pending` (gap #3 — secondary)
  Goal: `UNIFORM RANDOM GENERATION (Boltzmann samplers, Duchon et al. 2004) — uniform-by-size sampling of derivations for unbiased coverage of the deep tail (a distribution guarantee our weighted diverse pass lacks). Secondary for SIGNOFF (directed + k-path matter more for "guarantee every construct"); valued for unbiased exploration.`
  Acceptance: `approximate-size-uniform generation available as a strategy; parser-agnostic.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.6`
  Status: `pending` (gap #5 — secondary)
  Goal: `FULL GRAMMAR-TREE-AWARE SHRINKING — drop-optional / collapse-alternation / prune-subtree / reduce-repetition reduction while preserving the failing property (we have delimiter-aware structural minimization only — see the book's stimuli shrinker note). Parser-agnostic.`
  Acceptance: `grammar-tree-aware minimizer reduces counterexamples beyond the delimiter-aware baseline; property-preserving.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.7`
  Status: `pending` (gap #6 — secondary)
  Goal: `GRAMMAR-MUTATION MATURITY METRIC (Grammar Mutation for Testing Input Parsers, TOSEM 2025) — use grammar mutation to QUANTIFY generator+grammar maturity (we generate mutations but don't use them as a maturity metric). Parser-agnostic.`
  Acceptance: `a mutation-based maturity score computed per grammar; reported.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.8`
  Status: `pending` (survey gap (a), folded 2026-07-07 from the `-0005` survey)
  Goal: `COMBINATORIAL / PAIRWISE / N-WISE INTERACTION COVERAGE (CIT) — cover n-wise combinations of INDEPENDENT choice points (distinct from .2's k-path, which is ancestor-chain context). No impl today (stimuli_generator.rs:6056 is only the k-path cost warning). Parser-agnostic grammar-structure property.`
  Acceptance: `an n-wise interaction coverage universe + numerator computable per grammar; reported like --report-k-path-coverage.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.9`
  Status: `pending` (survey gap (b), folded 2026-07-07)
  Goal: `INTEGRATED METAMORPHIC + DIFFERENTIAL ORACLES — semantics-preserving transforms (rename bound ids, reorder independent decls, redundant parens) that must preserve acceptance/AST-equivalence; promote the per-family EXTERNAL differential gates (pcre2, corpus triage) into a generation-integrated oracle. Parser-agnostic.`
  Acceptance: `at least one metamorphic transform lane wired into generation with an acceptance/AST-equivalence oracle; divergences reported.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.10`
  Status: `pending` (survey gap (c), folded 2026-07-07)
  Goal: `SWARM / FEATURE-DIVERSITY TESTING (Groce et al.) — randomly disable grammar-feature subsets per run so rare feature ABSENCE combinations are exercised. No swarm mechanism exists today. Parser-agnostic.`
  Acceptance: `a swarm strategy (per-run feature subset masking) available; coverage delta vs the diverse pass measured.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.11`
  Status: `pending` (survey gap (d), folded 2026-07-07)
  Goal: `BOUNDARY-VALUE COVERAGE AS A FIRST-CLASS GOAL — @range/@len bounds are HONORED (parse_semantic_numeric_bounds/len_bounds) but not systematically TARGETED (min/max/min-1/max+1 as coverage obligations). Parser-agnostic.`
  Acceptance: `boundary obligations derived from declared constraints; generation targets them; a boundary-coverage number reported.`
  Verification: `pending`
  Commit: `pending`

- ID: `STIMULI-SIGNOFF.12`
  Status: `done` (design + implementation, 2026-07-07 session #57)
  Goal: `RETIRE THE LAST GENERATOR NAME-GATE: should_insert_quantified_separator() (stimuli_generator.rs:11635-11659) hardcodes grammar_name=="systemverilog_preprocessor" + 4 container-rule names + the pp_item element check. Re-express as a general, grammar-declared separator-cohesion property — the 4th member of the name-gate retirement series (WS-DIRECTIVE @whitespace_sensitive → DEFAULT-PROFILE @default_profile → PROFILE-ALIAS @profile_alias → this).`
  Design:
    - `REPRODUCE/ISSUE (tool-backed): stimuli_generator.rs:11642 is the generator's ONLY grammar-name string-literal gate (grep-verified over the module; the other "regex" hits are TOKEN-TYPE matches). Introduced by 90e502b5 ("Eliminate SV preprocessor parseability debt"). It violates [[feedback_features_parser_agnostic_enable_all_parsers]] (capability-gated, never grammar-name-gated) and hides a generation-relevant lexical-cohesion property OUTSIDE the EBNF ([[project_ebnf_is_single_source_of_truth]]): a whitespace-sensitive line-oriented SYNTHETIC/scratch grammar cannot express "stacked items need a line break" at all.`
    - `WHY the capability exists: svpp is line-oriented; every directive's trailing newline is OPTIONAL (pp_define := ... newline?), so two stacked pp_item renderings can fuse into one line (a define body eats to EOL) → the generator inserts "\n" between quantified pp_item iterations when the junction lacks one. The four hardcoded containers (systemverilog_preprocessor_file :24, pp_if_branch :83, pp_elsif_branch :85, pp_else_branch :87) are EXACTLY the four pp_item* sites — the property truly belongs to pp_item itself.`
    - `FIX: new rule-level StimuliSteering directive @quantified_separator, bound to the QUANTIFIED rule (pp_item). Payload: "sep" shorthand (satisfied only by itself) or { insert: "sep", satisfied_by: ["sep", ...] }. General engine rule at a quantifier join: if the quantified element is a rule-reference to a rule declaring the directive, insert 'insert' between two non-empty adjacent renderings unless output ends with — or the segment starts with — ANY satisfied_by spelling. svpp declares { insert: "\n", satisfied_by: ["\n", "\r\n"] }: the CRLF knowledge moves INTO the grammar (its own newline := /\r?\n/ spelling); the engine keeps zero language knowledge. Byte-identity with today expected BY CONSTRUCTION: ends_with("\r\n") ⊆ ends_with("\n"); every reachable segment-leading "\r" is a pp_blank_line newline rendering "\r\n" (no other pp_item alternative can start with a bare CR).`
    - `MECHANICS: (1) semantic_directive_registry.rs — register quantified_separator as StimuliSteering (does NOT serialize into parser artifacts: svpp's @sample yields 0 directives_by_rule inserts, grep-verified → ALL 11 generated parsers stay byte-identical, NO release bump). (2) semantic_runtime.rs — shared compile fn (QuantifiedSeparatorPolicy { insert, satisfied_by }) so the validator lints through the SAME parser the generator compiles with (the WS-DIRECTIVE pattern); annotation payload strings support \n escapes (unified_semantic_ast.rs:516). (3) annotation_validator.rs — W_SEM_INVALID_QUANTIFIED_SEPARATOR_PAYLOAD via that fn. (4) stimuli_generator.rs — constructor-time HashMap<rule, policy> cache (the compute_name_gates pattern); should_insert_quantified_separator loses BOTH name-gates and becomes directive-driven; node_is_rule_reference generalizes to "element references a policy-bearing rule". (5) grammars/systemverilog_preprocessor.ebnf — the directive above pp_item. On-item placement is behavior-identical today AND more general (any future container quantifying pp_item inherits it).`
    - `VERIFICATION PLAN: pre/post svpp generated-corpus BYTE-COMPARE at seeds 0/7/42 (expect identical); svpp cert fully_certified spf=0 seeds 0/7/42 + the zero-plausible-gap gate; cross-grammar certs byte-identical (regex 198/198/0); all 11 regenerated parsers cmp byte-identical; dual-run gate; lib both feature sets; clippy; registry/validator/compile-fn unit tests incl. a scratch-style synthetic line-oriented grammar proving the capability is now EXPRESSIBLE for any grammar; lockstep = annotation normative spec + semantic steering control matrix + book annotation chapter + svpp parser book.`
  Acceptance: `the grammar-name + rule-name literals are DELETED from stimuli_generator.rs; svpp behavior proven preserved (byte-compare + cert + gates); the capability demonstrated on a NON-svpp grammar; full lockstep.`
  Verification: `DONE 2026-07-07 (session #57). IMPLEMENTATION FINDING vs the design: the "@sample precedent ⇒ emit-neutral automatically" inference was INCOMPLETE — regenerating svpp flipped pp_item's emission from the fast-path onto the with_semantic_runtime_rule_transaction wrapper (codegen's rule_has_no_semantic_annotations treats ANY annotation as runtime-relevant unless excluded; the WS/DEFAULT/ALIAS directives were already excluded there). Fixed by adding QUANTIFIED_SEPARATOR_DIRECTIVE_NAME to that exclusion + a codegen pin test (quantified_separator_directive_keeps_the_annotated_rule_on_the_fast_path); post-fix the svpp parser artifact is cmp BYTE-IDENTICAL. Full evidence in the Acceptance Checklist below. Adjacent observation (routine, logged not acted): rules carrying ONLY stimuli-only directives like @sample (svpp directive_tail/line_comment) still take the full wrapper — same-class dead weight, candidate future de-weight, NOT this leaf's scope.`
  Commit: `PGEN-STIMULI-SIGNOFF-0006 (design + tree restructure) + PGEN-STIMULI-SIGNOFF-0007 (implementation)`

### STIMULI-SIGNOFF.12 — Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — `stimuli_generator.rs:11642` (pre-change): `if self.grammar_name != "systemverilog_preprocessor" … matches!(current_rule, "systemverilog_preprocessor_file" | "pp_if_branch" | "pp_elsif_branch" | "pp_else_branch") && Self::node_is_rule_reference(element, "pp_item")` — grep-verified the ONLY `grammar_name` string-literal gate in the module (other `"regex"` hits are token-type matches); introduced by `90e502b5`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the capability exists: svpp directives' trailing `newline?` is optional, so stacked `pp_item` renderings fuse into one line (a define body eats to EOL); the generator must insert `"\n"` at un-separated quantifier joins. WHERE the defect lives: the policy was engine-hardcoded per grammar/rule NAME instead of grammar-declared ([[feedback_features_parser_agnostic_enable_all_parsers]] + EBNF-single-source-of-truth); the four hardcoded containers are exactly the grammar's four `pp_item*` sites (systemverilog_preprocessor.ebnf:24/83/85/87), so the property belongs to `pp_item`. Secondary root cause found DURING landing: codegen `ast_based_generator.rs:230` (`rule_has_no_semantic_annotations`) treats any annotation as runtime-relevant unless excluded → the directive flipped pp_item onto the transaction wrapper (svpp parser diff at line 1665) until the exclusion landed.
- [x] **FIX** — fix-hierarchy tier 3 (new declarative annotation; tiers 1-2 impossible: no existing annotation expresses generation-side junction cohesion, and no grammar-only rewrite can steer the GENERATOR's join logic). Rule-level StimuliSteering directive `@quantified_separator` (registry entry; `semantic_runtime::compile_quantified_separators` + payload parser shared with the validator lint `W_SEM_INVALID_QUANTIFIED_SEPARATOR_PAYLOAD`; generator precomputes a per-rule policy map, `quantified_separator_to_insert` replaces the name-gate, malformed payloads fail LOUDLY at the first join; codegen exclusion keeps the annotated rule on the fast path). svpp declares `{ insert: "\n", satisfied_by: ["\n", "\r\n"] }` on `pp_item` — the CRLF spelling is grammar knowledge (`newline := /\r?\n/`), moved INTO the grammar.
- [x] **ADDRESSED (verified)** — name-gate DELETED (grep `"systemverilog_preprocessor"` in stimuli_generator.rs → 0 code hits); capability now GENERAL: `quantified_separator_directive_separates_stacked_renderings_for_any_grammar` (a generator named "test" separates a synthetic `file := item+` with `@quantified_separator: ";"`) + the migrated `preprocessor_item_repetition_newline_separator_is_grammar_declared_not_name_gated` proves the NAME alone activates nothing while the declared directive reproduces the exact svpp scenario.
- [x] **NO REGRESSION** — svpp `CERTIFICATE-COVERAGE: … total=74 proof=0 witness=74 UNKNOWN=0 fully_certified=true (sample_parse_failures=0 …)` byte-identical pre/post at seeds 0/7/42; svpp `--generate-stimuli --count 40 --stimuli-corpus-json` corpora cmp BYTE-IDENTICAL at seeds 0/7/42; regex cert `total=198 witness=198 UNKNOWN=0 fully_certified=true` byte-identical (cross-grammar guard); regenerated `systemverilog_preprocessor_parser.rs` cmp BYTE-IDENTICAL vs pre-change snapshot (emit-neutrality; regex parser regen also byte-identical); lib suites no-features 747/0, `generated_parsers` 821/0, dual-feature 863/0 (= 848 baseline + 15 new tests); `sv_preprocessor_zero_plausible_gap_proof_gate` ✅; `ebnf_frontend_dual_run_gate` ✅; clippy source-strict ✅ (generated stage = known pre-existing 178 eq_op debt, non-strict by design); `mdbook_docs_gate` + `ebnf_parser_book_gate` ✅.
- [x] **LOCKSTEP** — `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` (StimuliSteering entry), `PGEN_ANNOTATION_NORMATIVE_SPEC.md` (normative section under Stimuli generation), top book `annotation-system.md` (new `@quantified_separator` subsection), ebnf parser book `semantic-annotations.md` (catalog row + full section, HTML regenerated); svpp parser book N/A (consumer AST surface unchanged — parser + corpora byte-identical, no release bump); contract/ledger N/A (no released-parser behavior change).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `STIMULI-SIGNOFF.12` | `done` (`-0006` design + `-0007` implementation, 2026-07-07) | the LAST generator name-gate RETIRED via `@quantified_separator` (completes the WS→DEFAULT→ALIAS→SEPARATOR retirement series); svpp corpora + parser artifact byte-identical, capability proven on a non-svpp grammar. |
| — | `STIMULI-SIGNOFF.1` | `done` (`-0001`) | Audit landed; 6 gaps → leaves `.2`–`.7` (KM [[stimuli-generator-capability-gaps]]). |
| — | `STIMULI-SIGNOFF.2` | `done` (`-0002`/`-0003`/`-0004`; gap #1 k-path coverage CLOSED — universe + numerator + `--report-k-path-coverage` report) | k-path metric DEFINES the signoff bar; SV measured (sv_2017, k=2): universe 4105, covered 679 (16.5%). Optional follow-up `.2.4` (wire the recorder into the full closed-loop gate). (Row was stale `pending`; corrected 2026-06-30.) |
| 1 | `STIMULI-SIGNOFF.4` | `active` (design recorded `-0009` 2026-07-07; **next = `.4.1`** learned-distribution layer) | directed/learned generation (FDLOOP) — design done + `.7.4.6` reconciliation recorded (construction supersedes reach; the GENERAL learned-distribution capability is the value); first code slice = `.4.1`. |
| 4 | `STIMULI-SIGNOFF.3` | `pending` | code-coverage feedback pairs with `.2` (input→code coverage). |
| — | `.5` / `.6` / `.7` | `pending` | secondary (uniform/Boltzmann · grammar-tree shrinking · mutation-maturity metric). |
| — | `.8` / `.9` / `.10` / `.11` | `pending` | the 2026-07-01 survey gaps (CIT · metamorphic/differential oracles · swarm · boundary-value-as-goal), folded in 2026-07-07. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` to own the user's stimuli-generator-signoff vision. `SV-EXH-PROOF.7` (close `focused_replay_target_debt_zero`) feeds this tree: its generator limitations are audit data points, but `.7` itself stays owned by SV-EXH-PROOF (SV closure) — STIMULI-SIGNOFF owns the GENERAL capability-gap closure.
- `2026-06-10` (audit data point from `GRAMMAR-WELLFORMED.H.10.2.3`, tool-proven): **the generator cannot materialize parser-side BUILTIN primitives, and negative-lookahead guards are generation-blind.** `builtin_any_char`/`builtin_ascii_char` are codegen-native matchers with no grammar definition and no generator special-case — `generate_rule("builtin_any_char")` errors `Missing rule`, so every rule body referencing them is ungeneratable (regex `unicode_char`; also why `comment_text`, the callout/directive payloads only ever generate EMPTY via their `*`-quantifiers). Additionally `ASTNode::Lookahead → Ok("")` (`stimuli_generator.rs`) means a `!X Y` idiom's guard is never honoured when materializing `Y`. The signoff-grade capability would be: (a) generator-side materializers for the builtin primitives, (b) lookahead-guard-aware terminal choice (materialize `Y` such that `X` does not match). Worked around declaratively in `H.10.2.3` via a rule-level `@sample` witnessing literal — adequate for cert-coverage, but the general capability belongs here when prioritized.

- `2026-07-01` (director-requested fresh capability re-survey, `PGEN-STIMULI-SIGNOFF-0005`, tools-first, symbol-cited): a fresh code-grounded HAVE/PARTIAL/LACK sweep of `stimuli_generator.rs`/`main.rs`/gate-scripts CONFIRMED the `.1` audit and added symbol evidence for the strengths — completeness proof (`--report-certificate-coverage`, `reach_hops()` `:6761`), k-path (`compute_k_paths()` `:6059`, `--report-k-path-coverage`), **negative/near-miss generation ALREADY HAVE** (`StimuliNegativeProfile::NearValidLocal` `:173`, `apply_near_valid_negative_profile()` `:12818`, `--stimuli-negative-profile near_valid_local`; recovery `NearSyncNegative` `:137`), grammar-aware mutation (`GrammarAwareLocal` `:148`, `:7776`), boundary constraints (`parse_semantic_numeric_bounds`/`len_bounds` `:12629`/`:12635`, `@range`/`@len`), coverage-guided fuzz (`--coverage-guided-fuzz-rounds` `main.rs:320`), provenance (`source_seed`/`new_rule_hits` `main.rs:524`), per-profile (`apply_grammar_profile_filter()`), determinism/budgets, parseability roundtrip. **Four candidate gaps the `.2`–`.7` set does NOT yet cover** (to fold in when prioritized): (a) **combinatorial/pairwise/n-wise interaction coverage (CIT)** — distinct from k-path (`.2`), covers n-wise combinations of independent choice points (no impl; `:6056` is the k-path cost warning only); (b) **integrated metamorphic + differential oracles** — semantics-preserving transforms (rename bound ids, reorder independent decls, redundant parens) that must preserve acceptance/AST-equivalence, and promoting the per-family EXTERNAL differential gates (pcre2, corpus triage) into a generation-integrated oracle; (c) **swarm / feature-diversity testing** (Groce et al. — randomly disable feature subsets per run; no `swarm` mechanism found); (d) **boundary-value coverage as a first-class coverage GOAL** (bounds are HONORED but not systematically TARGETED). Gap MAP onto existing leaves: my "general first-class ddmin (any-failure)" = existing `.6` (grammar-tree-aware shrinking; note `shrink_parseability_counterexample()` `main.rs:3567` exists but is parseability-specific); my "parser-coverage-feedback steering" = existing `.3` (code-coverage feedback); "richer ISLa-style semantic invariants" EXTENDS `STORE-AWARE-GEN` (the store is declarative-only, no SMT). **NEW concrete finding — a parser-agnosticism SMELL to remediate:** `should_insert_quantified_separator()` (`stimuli_generator.rs:11297-11321`) HARDCODES `systemverilog_preprocessor` rule names (`pp_if_branch`, `pp_elsif_branch`, `pp_else_branch`, `pp_item`) — the one per-language leak in an otherwise parser-agnostic generator; a bounded de-smell leaf (re-express as a general EBNF/annotation-driven separator-cohesion property). Full grounded baseline table in `CHANGES.md` under `-STIMULI-SIGNOFF-0005`.

## Open Questions

- Does the first concrete capability (deterministic target-reach/path-forcing, from `.7`) land under SV-EXH-PROOF.7 or graduate here? (resolve when `.7` resumes)
- ~~Fold the four `2026-07-01`-surveyed gaps + the parser-agnostic de-smell into numbered leaves when this tree is next actively worked.~~ **RESOLVED 2026-07-07**: folded as `.8`–`.12` (`PGEN-STIMULI-SIGNOFF-0006`); `.12` carries its full design and is the active frontier.

## Blockers

- None (vision lane; `SV-EXH-PROOF.7` is the active feeder).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `STIMULI-SIGNOFF.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `STIMULI-SIGNOFF.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
- `2026-07-01`: Director-requested fresh capability re-survey (`PGEN-STIMULI-SIGNOFF-0005`, PURE-DOCS) — a tools-first, symbol-cited HAVE/PARTIAL/LACK sweep confirmed the `.1` audit and added 4 candidate gaps (CIT/pairwise, integrated metamorphic+differential oracles, swarm, boundary-value-as-coverage-goal) + one concrete parser-agnosticism SMELL (`should_insert_quantified_separator()` hardcodes svpp rule names). Recorded as a `Decisions` entry; leaves `.8`–`.12` deferred to the next active work on this tree (see `Open Questions`). No structural change to the existing `.1`–`.7` plan.
- `2026-07-07`: (`PGEN-STIMULI-SIGNOFF-0006`, PURE-DOCS, session #57) — tree actively resumed per the MEMORY.md PNT frontier: the deferred survey items folded in as numbered leaves `.8`–`.11` (CIT · metamorphic/differential · swarm · boundary-value-as-goal, all `pending`) + `.12` (the quantified-separator de-smell) created `active` with its full tools-first DESIGN (the `@quantified_separator` rule-level StimuliSteering directive; evidence: the name-gate at `stimuli_generator.rs:11642` is the generator's only grammar-name literal, the 4 hardcoded containers are exactly the grammar's 4 `pp_item*` sites, StimuliSteering directives provably don't serialize into parser artifacts → emit-neutral by construction). `.12` implementation is the tree's frontier.
- `2026-07-07`: (`PGEN-STIMULI-SIGNOFF-0009`, PURE-DOCS, session #58) — `.4` RESEARCH + DESIGN slice per the decisive `-0008` resume pointer: FDLOOP (arXiv 2508.01472) read in full; the mandated `SV-EXH-PROOF.7.4.6` reconciliation recorded (the original "273 → ~0" acceptance PARTIALLY SUPERSEDED — deterministic construction landed 273→~84 and the residual metric is noise-dominated ±~25, so the surviving value is the GENERAL probabilistic-grammar-learning capability); full worked mapping + slice plan in `docs/tasks/STIMULI-SIGNOFF-4-fdloop-directed-generation-design.md`; leaf acceptance re-framed; sub-leaves `.4.1`–`.4.5` created; frontier → `.4.1`. NOVEL-direction finding surfaced: goal G3 corpus-mimicry (learn a distribution from a real corpus → generate realistic stimuli) is a NEW capability direction beyond the original gap list.
- `2026-07-07`: (`PGEN-STIMULI-SIGNOFF-0007`, CODE, session #57) — `.12` LANDED end-to-end: the last generator name-gate RETIRED. Registry entry (StimuliSteering) + `compile_quantified_separators`/payload parser in semantic_runtime (per-rule verdict map; branch-level attachment rejected; conflicting duplicates error) + validator lint `W_SEM_INVALID_QUANTIFIED_SEPARATOR_PAYLOAD` (same parser, no second dialect) + generator de-smell (`quantified_separator_to_insert` replaces `should_insert_quantified_separator`; malformed payload = loud error at the first join) + svpp declares `{insert: "\n", satisfied_by: ["\n", "\r\n"]}` on `pp_item`. IMPLEMENTATION FINDING: the design's emit-neutrality inference was incomplete — codegen's `rule_has_no_semantic_annotations` flipped pp_item onto the transaction wrapper; fixed by the 4th exclusion + a codegen fast-path pin test. PROVEN: svpp certs (74/74/0 fully_certified) + corpora byte-identical seeds 0/7/42; svpp+regex parser artifacts cmp byte-identical; capability works for ANY grammar (synthetic `file := item+` test, generator named "test"); the name alone activates nothing (migrated legacy test); suites 747/821/863 all green; svpp zero-plausible-gap + dual-run + both book gates ✅. Full evidence: the leaf's Acceptance Checklist. Name-gate retirement series now WS→DEFAULT→ALIAS→SEPARATOR, series complete AGAIN (no known name-gates remain in the generator).
