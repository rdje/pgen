# STIMULI-SIGNOFF: Signoff-Grade EBNF Stimuli Generator (capability-gap closure)

## Metadata

- Tree ID: `STIMULI-SIGNOFF`
- Status: `proposed`
- Roadmap lane: `Stimuli generator → best-in-class / signoff-grade (user vision 2026-05-31)`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
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
  Status: `active` (`.1` audit DONE 2026-06-03; 6 gaps → leaves `.2`–`.7`)
  Goal: `Signoff-grade, parser-agnostic EBNF stimuli generator via capability-gap closure.`
  Children: `.1` audit → `.2` k-path coverage · `.3` code-coverage feedback · `.4` directed/learned generation · `.5` uniform/Boltzmann · `.6` grammar-tree-aware shrinking · `.7` mutation-maturity metric

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
  Status: `pending` (gap #2 — CO-OWNED with SV-EXH-PROOF.7.4.6)
  Goal: `DIRECTED / LEARNED GENERATION (FDLOOP, Kirschner & Soremekun, arXiv 2508.01472, 2025) — goal-specific inputs via probabilistic-grammar learning + feedback, to reach deep targets our deterministic reach-plan SEARCH times out on. CO-OWNED: SV-EXH-PROOF.7.4.6 is the SV application (derivation-directed construction → literal-0); this leaf is the GENERAL parser-agnostic capability. Read FDLOOP before building.`
  Acceptance: `directed generation reaches targets the search misses; SV residual 273 → ~0; parser-agnostic.`
  Verification: `pending`
  Commit: `pending`

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

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `STIMULI-SIGNOFF.1` | `done` (`-0001`) | Audit landed; 6 gaps → leaves `.2`–`.7` (KM [[stimuli-generator-capability-gaps]]). |
| 2 | `STIMULI-SIGNOFF.2` | `pending` | k-path metric DEFINES the signoff bar (rule+branch under-defines "done"). |
| 3 | `STIMULI-SIGNOFF.4` | `pending` | directed/FDLOOP = the literal-0 reach (co-owned by `SV-EXH-PROOF.7.4.6`). |
| 4 | `STIMULI-SIGNOFF.3` | `pending` | code-coverage feedback pairs with `.2` (input→code coverage). |
| — | `.5` / `.6` / `.7` | `pending` | secondary (uniform/Boltzmann · grammar-tree shrinking · mutation-maturity metric). |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` to own the user's stimuli-generator-signoff vision. `SV-EXH-PROOF.7` (close `focused_replay_target_debt_zero`) feeds this tree: its generator limitations are audit data points, but `.7` itself stays owned by SV-EXH-PROOF (SV closure) — STIMULI-SIGNOFF owns the GENERAL capability-gap closure.
- `2026-06-10` (audit data point from `GRAMMAR-WELLFORMED.H.10.2.3`, tool-proven): **the generator cannot materialize parser-side BUILTIN primitives, and negative-lookahead guards are generation-blind.** `builtin_any_char`/`builtin_ascii_char` are codegen-native matchers with no grammar definition and no generator special-case — `generate_rule("builtin_any_char")` errors `Missing rule`, so every rule body referencing them is ungeneratable (regex `unicode_char`; also why `comment_text`, the callout/directive payloads only ever generate EMPTY via their `*`-quantifiers). Additionally `ASTNode::Lookahead → Ok("")` (`stimuli_generator.rs`) means a `!X Y` idiom's guard is never honoured when materializing `Y`. The signoff-grade capability would be: (a) generator-side materializers for the builtin primitives, (b) lookahead-guard-aware terminal choice (materialize `Y` such that `X` does not match). Worked around declaratively in `H.10.2.3` via a rule-level `@sample` witnessing literal — adequate for cert-coverage, but the general capability belongs here when prioritized.

## Open Questions

- Does the first concrete capability (deterministic target-reach/path-forcing, from `.7`) land under SV-EXH-PROOF.7 or graduate here? (resolve when `.7` resumes)

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
