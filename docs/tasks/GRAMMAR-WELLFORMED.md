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
   recursion (PGEN eliminates the WRAPPER shape and, since `A2.5`, the inline DIRECT shape; an
   **INDIRECT** cycle is eliminated by nothing and only the runtime guard — which *rejects* rather
   than handles — stands, so the lint reports it as `left_recursion_unhandled` ⚠️ `A2.6`; engine gap
   owned by `ENGINE-UNIVERSAL-SERVICES.13`) + no nullable-repetition loop
   (`detect_nullable_repetition` ⚠️).
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
- `A1b.2` — **DONE (PGEN-GRAMMAR-WELLFORMED-0032, 2026-06-06):** SV closure-contract no-regression FLOOR
  re-baseline (v2→v3) after two LEGITIMATE profile-variant collapses that landed since the v2 baseline.
  `sv_syntax_closure_gate` was failing 2 violations — `defined_rule_count 1453 < min_total_rules 1455`
  and `reachable_rules 1406 < min_reachable_rules 1407`. ROOT CAUSE (tools-first, decisive): `git diff`
  of the defined-rule-name set between the v2 baseline commit `f5b25b3d` (1455 rules) and HEAD (1453)
  showed EXACTLY two names disappeared — `binary_module_path_operator_sv_2023` (collapsed into its
  canonical form by `PGEN-ANNOTATION-COMPOSITION-0002`; it was REACHABLE → −1 reachable) and
  `non_zero_decimal_digit_sv_2017` (collapsed into the base by `PGEN-ANNOTATION-COMPOSITION-0005`; it was
  a BLESSED unreachable number orphan → −1 total, −1 unreachable). Both are documented, leaf-owned,
  corpus-14/14-verified grammar cleanups — NOT regressions; the stale no-regression floors simply weren't
  lowered in lockstep (the contract even still listed the deleted `non_zero_decimal_digit_sv_2017` in its
  blessed list). FIX (honest closure-debt management per the contract's own drift policy): contract v3 —
  `min_total_rules` 1455→1453, `min_reachable_rules` 1407→1406, `max_unreachable_rules` 50→49 (matches the
  now-49 blessed-orphan surface); dropped `non_zero_decimal_digit_sv_2017` from the blessed list; v3 prose
  records the two collapses + responsible commits. Contract-only change (no grammar/Rust edit). VERIFIED:
  `sv_syntax_closure_gate` ✅ (defined 1453, reachable 1406, unreachable 49 — all within v3); reachability
  is otherwise unchanged (no real loss masked). Surfaced during `LEXICAL-ANNOTATIONS.3d (ii-CLI)`
  verification as a pre-existing failure; fixed here on its own leaf per the doctrine.
- `G.4.8` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0033`, 2026-06-07): SV `use_clause` PEG ordered-choice
  ambiguity FIXED (the witness-parseability residual routed from `LEXICAL-ANNOTATIONS.4.1`).** FIX LANDED:
  reordered `use_clause` so the named_parameter-bearing alternatives precede the simple form + added an
  explicit no-lib `use cell named_parameter_assignment+ (: config)?` alternative first (the
  `-> {library,name,config}` annotation stays on the now-last simple alt). SV parser regenerated
  (mtime-verified). VERIFIED: the previously-failing `cell j use \foo.\bar () : config` now parses + all
  valid forms still parse (`parseability_probe`); SV cert-coverage `sample_parse_failures` **1 → 0**; SV
  external corpus **14/14**; SV shape-contract GREEN; lib 613/613; release 1.0.136 → 1.0.137 (schema
  stays 3, strictly-more-permissive). Contract + shape-contract manifest updated in lockstep. ORIGINAL
  ROOT CAUSE (tools-first, `parseability_probe`): the cert-coverage
  witness pass generated a config sample whose `use \foo .\foo ()` is a valid `use_clause` alt-3
  derivation (cell=`\foo`, named_param=`.\foo()`, no lib), but the parser's PEG ordered choice committed
  to the SIMPLE alt (`kw_use (library_identifier dot)? cell_identifier (colon kw_config)?`), greedily
  matching `use \foo.\foo` as `lib.cell` and stranding the `()` → `did not consume full input`. Confirmed:
  `cell x use y : config` ✓, `cell j use \foo.\bar .p() : config` ✓, but bare `cell j use \foo.\bar () :
  config` ✗. FIX: reorder `use_clause` so the named_parameter-bearing alternatives PRECEDE the simple
  alternative, and add an explicit no-lib `kw_use cell_identifier named_parameter_assignment+ (: config)?`
  alternative first (so `use cell .param()` is matched before the simple form can grab `lib.cell`); the
  `-> {library,name,config}` return annotation stays on the (now-last) simple alt. LRM-grounded (the
  `[lib.]` in IEEE 1800 §A.1.5 use_clause is optional; this is a PEG specific-before-general reorder, the
  A2.1 idiom). Requires SV regen + verification (corpus 14/14, shape-contract, cert-coverage re-run, lib).
- `H.1` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0034`, 2026-06-07): Phase H STARTED — `parse_and_cover`
  wired for `regex` (first non-SV grammar); `--report-certificate-coverage` now runs for regex.** Added
  `parse_and_cover_regex` to `parser_registry.rs` (mirrors `parse_and_cover_systemverilog`, on the
  dedicated regex worker stack, no profile/stdlib, PCRE2-compile validation intentionally omitted) + set
  `parse_and_cover: Some(parse_and_cover_regex)` on the regex entry. The regex parser was regenerated
  (mtime-verified) to pick up the unconditional G.4.6 coverage codegen (`enable_coverage` /
  `exercised_rule_names`) — no codegen change. VERIFIED: `ast_pipeline regex.ebnf
  --report-certificate-coverage --entry-rule regex` now RUNS (no bail) → `total=206 proof=0 witness=73
  UNKNOWN=133 (sample_parse_failures=6)`; the regex regen is conformant (RGX broader-corpus gate ✅ — the
  critical downstream is unaffected); `parser_registry` tests 18/0; lib `--features generated_parsers`
  compiles. NEW FINDING (honest, Phase-H-surfaced): regex has **6** witness-parseability
  `sample_parse_failures` (a regex generator↔grammar round-trip residual now MEASURABLE for the first
  time — a follow-up G.4/Phase-H investigation, NOT a regression). **ROOT-CAUSED 2026-06-07
  (`PGEN-EBNF-SOT-0003`, EBNF-SOURCE-OF-TRUTH.2.1):** these witness-parseability `sample_parse_failures`
  are STRUCTURAL — the stimuli generator's `apply_word_boundary_spacing` over-inserts a trailing `" "`
  separator after an identifier even when the next char is `)`, breaking `(*VERB )` / `(?P>NAME )` /
  `(?(COND ))` on re-parse (NOT the out-of-band validator — `parse_and_cover_regex` skips it). At count 200
  the count is 39; `--no-word-boundary-spacing` collapses it 39→2. Fix routed to a re-opened
  **`LEXICAL-ANNOTATIONS.5`** (its `apply_word_boundary_spacing` owns it). **UPDATE 2026-06-07 — DONE in
  `LEXICAL-ANNOTATIONS.5` (`-0022`/`-0023`):** successor-aware word-boundary spacing drove the count-200
  residual **39 → 3** (deterministic; cross-family gate green). The remaining **3 are NOT word-boundary
  spacing** and stay owned HERE / by a regex.ebnf grammar tweak: (a) 2× empty character class `[]` — the
  generator emits `[]` (`class_body = class_item*` allows zero items) but the regex parser rejects it
  STRUCTURALLY (PCRE2: the first `]` after `[` is a literal member) → a regex char-class grammar-modeling
  gap; (b) 1× `(?(R 1))` — the conditional-recursion `R` (a 1-char all-word literal) → a regex.ebnf tweak
  making `(?(R` a single literal would resolve it. These are the genuine regex witness-parseability
  residuals now that the spacing noise is gone. Phase H continues for vhdl / svpp /
  rtl_* / json (each: regen + a `parse_and_cover_<grammar>` registry fn). ORIGINAL scope:
  Today only `systemverilog` sets `parse_and_cover`
  in `parser_registry.rs`; the cert-coverage gate hard-bails for every other grammar (`supports_parse_and_cover`
  false → "Phase H wires more grammars"). The G.4.6 coverage instrumentation (`enable_coverage` /
  `exercised_rule_names`, via the transactional `coverage_stack`) is emitted UNCONDITIONALLY by the codegen
  (`ast_based_generator.rs` ~:577-938) — so the only reason non-SV parsers lack it is staleness (generated
  pre-G.4.6). FIX (no codegen change): regenerate the regex parser (gets the coverage methods) + add a
  `parse_and_cover_regex` registry function (mirrors `parse_and_cover_systemverilog`, on the dedicated
  regex worker stack, no profile/stdlib) + set `parse_and_cover: Some(parse_and_cover_regex)` on the regex
  entry. Phase H continues for vhdl / svpp / rtl_* / json (each: regen + a registry fn). Acceptance:
  `--report-certificate-coverage` runs for regex (no bail) + reports its sample_parse_failures.
- `H.2` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0041`, 2026-06-08): Phase H continues — `parse_and_cover`
  wired for `vhdl` (the LAST unwired SHIPPED grammar); `--report-certificate-coverage` now runs for vhdl.**
  Same mechanical recipe as H.6/H.5/H.4, with the H.2-specific staleness fear FULLY DISCHARGED tools-first.
  The original block (2026-06-07, `-0036`) was: no `focus_vhdl` target, `generated/vhdl.json` apparently
  STALE (`vhdl.ebnf` mtime > json mtime) and a parser built from a fresher source — which threatened a
  HEAVY vhdl conformance re-verify. **ZERO-DRIFT PROOF (the checkout illusion, exactly as H.4/H.6
  predicted):** regenerating `vhdl.json` from the current `vhdl.ebnf` via the `ebnf_dual_run` frontend
  yields a file BYTE-IDENTICAL to the on-disk json **except `generated_at`/embedded path metadata** (fresh
  110508 B vs on-disk 110511 B; `diff` excluding `generated_at`/`source_file`/`source_path`/`output_path`
  = EMPTY) ⇒ **ZERO grammar drift** ⇒ the parser regen is provably behaviour-preserving, so NO heavy
  conformance corpus run is needed (the zero-drift proof IS the no-regression proof, the same de-risk H.4
  established). The parser regen vs the on-disk committed parser differs ONLY by (a) the unconditional
  G.4.6 coverage instrumentation (`enable_coverage`/`exercised_rule_names`, grep=3) and (b) a benign
  REGEX-SELF-HOSTING.6a evolution — the current generator (post-`.6a`) drops the now-dead
  `use regex::Regex;` import while keeping the 104 `match_regex` calls + fully-qualified `regex::Regex::new`
  in the cached-compile helper (the committed parser predates `.6a`); the lib compiles clean, confirming
  the drop is dead-import-only. Landed (mirror H.6): (1) `VHDL_EBNF/JSON/PARSER` vars + build rules +
  `vhdl_parser` + `focus_vhdl` Makefile targets; (2) `parse_and_cover_vhdl` in `parser_registry.rs` (cfg
  `has_generated_vhdl_parser`; mirrors `parse_and_cover_systemverilog_preprocessor` — vhdl entry
  `vhdl_file := design_unit*` is a FLAT list, no profile, no worker stack) + the registry entry set to
  `Some(parse_and_cover_vhdl)`. Same one-time bootstrap-ordering step as H.4/H.5/H.6 (stale pre-coverage
  parser present → direct-regen the parser with the already-built generator binary first, else E0599). Like
  rtl_frontend, `vhdl.ebnf` parses only under `--features ebnf_dual_run` (106 `/.../` literals), so the
  cert-coverage binary is built `--features "ebnf_dual_run generated_parsers"`. **MEASURED (count 40,
  DEFAULT depth 24 — no `--max-depth`, like svpp/rtl_frontend — DETERMINISTIC seed 0 run#1==run#2):** seed 0
  → `total=217 proof=0 witness=132 UNKNOWN=85 fully_certified=false (sample_parse_failures=0,
  proof_reverify_failures=0)`; seed 7 → `witness=129 UNKNOWN=88 (sample_parse_failures=2)`.
  **`sample_parse_failures=0` @ seed 0** ⇒ NO witness-parseability round-trip residual at the canonical seed
  (CLEAN, like json/rtl_const_expr/rtl_frontend; far cleaner than svpp's 24). UNKNOWN=85 is an honest
  residual (drive-to-0 via more/targeted witnesses is a follow-up). VERIFIED: direct regen produces the
  parser carrying the coverage methods; lib `--features generated_parsers` builds clean; `parser_registry`
  lib tests 20/0 (incl. `registry_exposes_vhdl_when_generated_parser_present` + the vhdl adapters); strict
  SOURCE clippy ok (`parse_and_cover_vhdl` clean; generated stage non-strict, pre-existing codegen debt
  only); json (`fully_certified=true`) / regex / rtl_const_expr (`48/41/7` @ `--max-depth 32`) cert-coverage
  UNAFFECTED. NO tracked-artifact change (`generated/` is untracked/gitignored; the regen is local). Default
  build (vhdl cfg off in a fresh clone) unaffected. **MILESTONE: with vhdl wired, EVERY SHIPPED parser
  grammar now runs under cert-coverage** (json, regex, rtl_const_expr, svpp, rtl_frontend, systemverilog,
  vhdl); only the internal meta/annotation grammars (`ebnf`, `return_annotation`, `semantic_annotation`)
  remain unwired. Frontier: drive each wired grammar's `UNKNOWN`→0 + the `H.5.1` svpp witness-parseability
  residual (toward the locked program: all existing parsers `Done`).
- `H.3` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0037`, 2026-06-08): Phase H continues — `parse_and_cover`
  wired for `json` (the simplest unwired grammar); `--report-certificate-coverage` now runs for json and
  reports `fully_certified=true`.** RESULT (deterministic across seed 0/1/7 + count 200/500):
  `CERTIFICATE-COVERAGE: grammar='json' entry='json' total=9 proof=0 witness=9 UNKNOWN=0
  fully_certified=true (sample_parse_failures=0, proof_reverify_failures=0)` — **json is the FIRST
  grammar to report `fully_certified=true` via Phase H cert-coverage** (regex/H.1 still has UNKNOWN
  residuals + 6→3 witness-parseability failures; SV has a large UNKNOWN). All 9 json rules are
  witnessed-reachable, 0 UNKNOWN, 0 sample-parse failures. Landed: (1) canonical regen targets
  `$(JSON_JSON)`/`$(JSON_PARSER)`/`json_parser`/`focus_json` in `rust/Makefile` (mirror the regex targets;
  establishes the `focus_<grammar>` pattern the H.2 note asked for); (2) `parse_and_cover_json` in
  `parser_registry.rs` (cfg `has_generated_json_parser`; mirrors `parse_and_cover_systemverilog` minus
  profile/stdlib/worker-stack — json has no grammar profile and no deep recursion); (3) the cfg-gated json
  registry entry set to `parse_and_cover: Some(parse_and_cover_json)`. VERIFIED: `make focus_json`
  regenerates `generated/json_parser.rs` (9 rules; carries the unconditional G.4.6 `enable_coverage` /
  `exercised_rule_names` — no codegen change); `parser_registry` lib tests 7/0 (incl. the json adapters);
  lib `--features generated_parsers` builds clean; strict source clippy 0 errors (generated stage
  non-strict, pre-existing debt only — `parse_and_cover_json` not flagged). NO tracked-artifact change
  (`generated/` is gitignored; the regen is local). Default build (json cfg off) unaffected. Frontier:
  vhdl (H.2, still blocked on the fresh-json regen chain) + svpp / rtl_* (each: a `focus_<grammar>` target
  + regen + a `parse_and_cover_<grammar>` registry fn, now mechanical given the json/regex pattern).
  *(superseded plan text below kept for provenance)* **(original IN-PROGRESS plan)** Phase H continues — wire
  `parse_and_cover` for `json` (the simplest unwired grammar; resume-pointer-directed next step).
  WHY json (not vhdl/H.2): the H.2 blocker was vhdl-specific — a STALE `generated/vhdl.json`, a parser
  built from a fresher source, no `focus_vhdl` target, and a HEAVY vhdl conformance re-verify. json has
  NONE of those: `grammars/json.ebnf` is a 1 KB grammar, neither `generated/json.json` nor
  `generated/json_parser.rs` exists yet (so there is no staleness — the regen is fresh from source), and
  there is no heavy json conformance corpus. So json is genuinely unblocked while vhdl stays blocked. The
  G.4.6 transactional coverage instrumentation (`enable_coverage` / `exercised_rule_names`) is emitted
  UNCONDITIONALLY by the codegen, so a fresh json parser carries it with no codegen change (same as the
  regex H.1 pattern). PLAN (mirror H.1): (1) add canonical regen targets `$(JSON_JSON)` / `$(JSON_PARSER)`
  / `json_parser` / `focus_json` to `rust/Makefile` (mirrors the regex targets — also establishes the
  `focus_<grammar>` pattern the H.2 note asked for); (2) regen `generated/json.json` (frontend
  `--emit-raw-ast-json`) + `generated/json_parser.rs` (generator); (3) add `parse_and_cover_json` to
  `parser_registry.rs` (cfg `has_generated_json_parser`; mirrors `parse_and_cover_systemverilog` minus
  profile/stdlib/worker-stack — json has no grammar profile and no deep recursion); (4) set
  `parse_and_cover: Some(parse_and_cover_json)` on the cfg-gated json registry entry. ACCEPTANCE:
  `ast_pipeline grammars/json.ebnf --report-certificate-coverage --entry-rule json` RUNS (no bail) +
  reports its `sample_parse_failures`; `parser_registry` tests + lib `--features generated_parsers` build
  clean; no regression (json regen is local/gitignored; default build with json cfg off is unaffected).
- `H.4` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0038`, 2026-06-08): Phase H continues — `parse_and_cover`
  wired for `rtl_const_expr` (the next-simplest unwired grammar; the first of the svpp/rtl_* batch);
  `--report-certificate-coverage` now runs for rtl_const_expr.** Mirrors the json H.3 / regex H.1 pattern.
  **DE-RISK FIRST (tools-first, the H.2 staleness trap addressed head-on):** the on-disk
  `generated/rtl_const_expr.json` mtime is OLDER than `grammars/rtl_const_expr.ebnf` (and the committed
  parser is newer still) — the same surface signature that BLOCKED vhdl H.2. But the mtime skew is a
  GIT-CHECKOUT ILLUSION, not content drift: regenerating the json from the current ebnf via the
  `ebnf_dual_run` frontend yields a file BYTE-IDENTICAL to the on-disk json **except the `generated_at`
  timestamp** (both 15900 bytes; `diff` excluding `generated_at` = empty) ⇒ **ZERO grammar drift**, so the
  parser regen is provably behaviour-preserving (it only ADDS the unconditional G.4.6 coverage
  instrumentation: committed 19646 → regen 19819 lines = +173 lines of `enable_coverage` /
  `exercised_rule_names` / `coverage_stack`). Two independent regens agree (only the embedded output-path
  string differs). Landed: (1) canonical regen targets `$(RTL_CONST_EXPR_EBNF/JSON/PARSER)` +
  `rtl_const_expr_parser` + `focus_rtl_const_expr` in `rust/Makefile` (mirror the json targets); (2)
  `parse_and_cover_rtl_const_expr` in `parser_registry.rs` (cfg `has_generated_rtl_const_expr_parser`;
  mirrors `parse_and_cover_json` — rtl_const_expr has no grammar profile and no deep PARSE-stack recursion,
  so no profile arg and no dedicated worker stack); (3) the cfg-gated rtl_const_expr registry entry set to
  `parse_and_cover: Some(parse_and_cover_rtl_const_expr)`; (4) **`main.rs` `run_certificate_coverage_report`
  now threads `--max-depth`** (it had hardcoded `StimuliConfig { ..Default::default() }` = 24, the lone
  report path NOT honoring `args.max_depth`, unlike the k-path report). NEW FINDING (the attribution rule
  in action — the witness pass surfaced it): at the default depth 24 the run does NOT bail, it FATAL-ERRORS
  `Stimuli generation depth exceeded max_depth=24 while expanding rule 'primary_expr'` — rtl_const_expr's
  GENERATION-side precedence chain (`conditional_expr → logical_or → … → unary_expr → primary_expr`, ~15
  rules deep) already needs ~15, and one `( conditional_expr )` nesting doubles it past 24. This is a
  GENERATION-depth budget, distinct from the PARSE-stack worker-stack concern. Threading `--max-depth`
  keeps the default 24 (so json/regex/SV cert-coverage is byte-identical) while letting rtl_const_expr run
  at `--max-depth 32`+. VERIFIED: `make focus_rtl_const_expr` regenerates `generated/rtl_const_expr_parser.rs`
  carrying the coverage methods (grep confirmed 2); `ast_pipeline grammars/rtl_const_expr.ebnf
  --report-certificate-coverage --entry-rule rtl_const_expr --max-depth 32` RUNS (no bail, no fatal):
  count 8 seed 0 → `total=48 proof=0 witness=41 UNKNOWN=7 fully_certified=false (sample_parse_failures=0,
  proof_reverify_failures=0)`, **same-seed DETERMINISTIC** (identical across two runs); seed 7 →
  `witness=45 UNKNOWN=3` (its own deterministic result). **`sample_parse_failures=0`** ⇒ rtl_const_expr has
  NO witness-parseability round-trip residual (cleaner than regex H.1's 3). UNKNOWN>0 is an honest residual
  (driving it to 0 via more/targeted witnesses is a follow-up, exactly as regex H.1 landed with residuals);
  `parser_registry` lib tests 20/0 (`--features generated_parsers`, incl. the rtl_const_expr adapter); lib
  `--features generated_parsers` builds clean; json (`fully_certified=true`) + regex cert-coverage
  UNAFFECTED (default depth 24 unchanged); behaviour-preserving (zero-drift proof above). NO tracked-artifact
  change (`generated/` is gitignored; the regen is local). Default build (rtl_const_expr cfg off)
  unaffected. **ONE-TIME BOOTSTRAP-ORDERING NOTE (for whoever wires svpp/rtl_frontend/vhdl next):** unlike
  json/regex (which had NO pre-existing parser), these grammars already have a STALE pre-coverage parser on
  disk, so `has_generated_<grammar>_parser` is ALREADY on — meaning the `$(RUST_AST_PIPELINE)` rebuild that
  `make focus_<grammar>` triggers (it builds `--features generated_parsers`, repo convention) compiles the
  NEW `parse_and_cover_<grammar>` against the still-stale parser's missing `enable_coverage` /
  `exercised_rule_names` → E0599. RESOLUTION (one-time): regenerate the parser DIRECTLY first with the
  already-built generator binary (`target/debug/ast_pipeline --generate-parser … <grammar>.json -o
  generated/<grammar>_parser.rs`) so the methods exist BEFORE any `generated_parsers` lib build; thereafter
  the focus target + lib build are clean. This is NOT a defect in the focus target (fresh clones start with
  the parser ABSENT → cfg off → identical to the json flow); it is only the in-place transition snag. Frontier: svpp + rtl_frontend (each: a `focus_<grammar>` target + regen + a
  `parse_and_cover_<grammar>` registry fn — same mechanical pattern, each its own leaf); then vhdl, whose
  fresh-json regen chain is now de-risked by this slice's proof that the mtime skew is a checkout illusion.
- `H.4.1` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0050`, 2026-06-09): ROOT-CAUSE rtl_const_expr's cert-coverage
  `UNKNOWN` residual (tools-first; pure-docs scoping of the FIRST per-grammar `UNKNOWN`→0 drive — NO code
  change).** GOAL: own the first per-grammar `UNKNOWN`→0 drive (smallest wired grammar; H.4 left
  `total=48 witness=41 UNKNOWN=7` @ `--count 8 --max-depth 32 --seed 0`) and ADJUDICATE the residual per
  the attribution rule BEFORE touching the generator. TOOL-BACKED FINDINGS (`--report-certificate-coverage`
  + `--generate-stimuli`, debug binary):
  - **The residual is monotone in sample count and reduces to ONE structural pair.** cert-coverage seed 0:
    `--count 8`→UNKNOWN 7 `[literal, based_integer, decimal_integer, bang, tilde, lparen, rparen]`;
    `--count 40`→3 `[based_integer, lparen, rparen]`; `--count 100`→**2 `[lparen, rparen]`** (and
    `--count 40 --seed 7`→2 `[lparen, rparen]`). So `bang`/`tilde`/`literal`/`decimal_integer` (and
    `based_integer` by ~count 100) ARE eventually witnessed by the coverage-guided ×24 uncovered-branch
    weighting — a sample-budget artifact, not a reach wall. The STUBBORN pair is `lparen`/`rparen`.
  - **`lparen`/`rparen` = the `primary_expr := lparen conditional_expr rparen` (parenthesised-primary)
    branch, and clean diverse generation essentially NEVER selects it.** `0/40` diverse samples @
    `--max-depth 32 --seed 0` contain a `(` (`grep -c '(' = 0`), even though the generator's OWN
    branch-coverage self-report says `branches 24/25` (it KNOWS the alt is uncovered). The
    parser-testified certificate is the trustworthy metric (sound: backtracked attempts truncated;
    complete: annotation-fold-immune) and it confirms `lparen`/`rparen` stay UNKNOWN — a clean illustration
    of why the certificate, not the generator's self-report, is the certification number.
  - **WHY (structural):** the parenthesised branch RE-ENTERS the full ~15-deep precedence chain
    (`conditional_expr → logical_or → … → unary_expr → primary_expr`), so one `( … )` nesting needs ~30
    derivation depth. At the DEFAULT `--max-depth 24` the diverse generator FATAL-errors `Stimuli
    generation depth exceeded max_depth=24 while expanding rule 'primary_expr'` (the chain + one paren does
    not fit). At `--max-depth 32` it fits, but the diverse pass's depth-floor pruning (prune the
    highest-rule-ref branch near the depth floor) + recursion-pressure penalty systematically deprioritise
    this deeply-recursive branch, so the ×24 coverage boost cannot overcome the structural avoidance within
    100 samples.
  - **ADJUDICATION (attribution rule):** the branch is statically REACHABLE (`(1)` is a valid
    `rtl_const_expr`; linter structural reachability = 0 unreachable for this grammar) ⇒ this is a
    **generator-reach DEFICIENCY (case 1)**, NOT a dead branch — the fix belongs in the generator, not the
    grammar.
  - **The witness/target-drive pass is NOT a valid shortcut for the gate.** `rust/src/main.rs:1572-1576`
    records an EXPLICIT design decision: the witness pass produces reach-plan-FORCED samples that often
    don't re-parse, so it is the WRONG witness source for certificate-coverage — the gate deliberately uses
    CLEAN diverse samples and keeps generation/certification separate. So `UNKNOWN`→0 must come from
    improving the CLEAN diverse generator's REACH, not from bolting forced witnesses onto the gate.
  - **NEXT (`H.4.2`, code — the CONSTRUCTIVE-reach lane, composes with `B2`/`C1`/`C2`):** improve the
    diverse generator so a deeply-recursive, never-covered branch is reliably reached within budget
    (candidate levers, to be tool-confirmed ONE-AT-A-TIME: do not depth-floor-prune a never-covered branch;
    bounded-ordered backtracking into a recursive branch with a min-derivation inner; recursion-pressure
    penalty exempt for uncovered branches). ⚠️ RISK: this is the GENERATOR HOT PATH shared by ALL grammars —
    the change MUST be measured against the GLOBAL metric (json stays `fully_certified` byte-identical;
    regex/vhdl/SV/svpp/rtl_frontend cert-coverage no-regression; cross-family + self-host + oracle gates
    green; lib tests). rtl_const_expr is the first proving ground per the locked program ("smallest first").
- `H.4.2` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0051`, 2026-06-09): CONSTRUCTIVE-REACH for deeply-recursive
  never-covered branches — rtl_const_expr cert-coverage is now `fully_certified=true` (UNKNOWN 7/3→0),
  with ZERO certification regression on any grammar.** Tools-first root cause (trace, `--generate-stimuli`
  A/B): `primary_expr := lparen conditional_expr rparen` re-enters the ~14-level precedence chain, so the
  clean diverse budget is spent reaching `primary_expr` (depth 27–32) and the parenthesised branch is only
  ever tried on the DEEPEST descents, where (a) it cannot complete and (b) even a constructed witness is
  discarded because the enclosing derivation is already doomed. Decisive A/B: raising `--max-depth` is NOT a
  fix (count 8 TIMES OUT at depth 48/64/96 — the `*`/`?:` fan-out explodes); `--max-repeat 0 --max-depth 64`
  reaches `(` but regresses operator-rule coverage (27/48) and emits monstrous ternary samples. FIX
  (generator-side, parser-AGNOSTIC, OPT-IN): new `StimuliConfig.reach_uncovered_recursive_branches` (default
  OFF → every non-cert-coverage surface byte-identical). When ON, `generate_or` (1) at the depth floor
  RETAINS a still-never-covered branch (so it survives to be tried), (2) tries a never-covered branch that
  re-enters an active-stack rule FIRST (so it is reached at the SHALLOWEST point, where the parent survives),
  and (3) on its depth-exhaustion retries it once with a fresh budget + `construct_mode` (min-repeat
  quantifiers + shortest-derivation OR via the Purdom min-terminal table) → one MINIMAL `( 1 )` witness that
  re-parses. Runaway backstop `MAX_UNCOVERED_REACH_RETRIES=4096` (self-limiting — stops at the first
  success). `run_certificate_coverage_report` made TWO-PASS: pass 1 = the DIVERSE certification sample set
  (reach OFF → byte-identical, so its `sample_parse_failures` never regresses), pass 2 = the auxiliary
  constructive-reach witness pass (reach ON), run only if pass 1 leaves UNKNOWN, UNIONING witnesses from
  re-parsing samples only; the reach pass's own unparseable probes are reported SEPARATELY, never folded into
  the certification number. **VERIFIED — DECISIVE git-stash baseline (pre vs post, identical commands, count
  40 seed 0):** rtl_const_expr UNKNOWN 3→**0** `fully_certified` (fail 0=0, deterministic across seeds
  0/1/7/42); regex UNKNOWN 101→98 (fail 0=0); vhdl UNKNOWN 85→69 (fail **0=0** — the +1 the single-flag
  prototype caused is now an auxiliary reach-pass probe, NOT a certification failure); SV (default entry,
  sv_2017) UNKNOWN 1160→1126 (fail **3=3** — the 3 is PRE-EXISTING, identical pre/post); json identical
  (`fully_certified`, fail 0); svpp UNKNOWN 4→3 (fail 0). `sample_parse_failures` byte-identical pre/post for
  EVERY grammar ⇒ no certification regression; UNKNOWN only ever DECREASES. lib (`--features
  generated_parsers`) **686/0**; new `ebnf_dual_run` test `reach_uncovered_recursive_branch_witnesses_
  parenthesised_primary` PASS; regex self-hosting guard OK; cross-family + oracle gates (default generation,
  proven byte-identical by the baseline) green. NOTE (alert, separate ticket): the default-entry sv_2017
  cert-coverage carries a PRE-EXISTING `sample_parse_failures=3` that contradicts the resume pointer's "SV
  cert-coverage CLEAN (0)" — a config/entry discrepancy to investigate, NOT caused by this slice. NEXT =
  drive each remaining wired grammar's `UNKNOWN`→0 (the reach pass already helps regex/vhdl) + the per-grammar
  reach drives.
- `H.5` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0039`, 2026-06-08): Phase H continues — `parse_and_cover`
  wired for `systemverilog_preprocessor` (svpp); `--report-certificate-coverage` now runs for svpp.** Same
  mechanical pattern as H.4 (the bootstrap recipe the H.4 leaf documented made this turnkey). svpp entry
  `systemverilog_preprocessor_file := pp_item*` is a FLAT list (NOT a deep precedence chain like
  rtl_const_expr), has NO `@profiles`, and its regen is ZERO-drift (fresh json from ebnf byte-identical to
  the on-disk json except `generated_at`), so it runs at the DEFAULT depth 24 (no `--max-depth` needed).
  Landed: (1) `$(SVPP_EBNF/JSON/PARSER)` + `systemverilog_preprocessor_parser` + `focus_systemverilog_preprocessor`
  Makefile targets (mirror json/rtl_const_expr); (2) `parse_and_cover_systemverilog_preprocessor` in
  `parser_registry.rs` (cfg `has_generated_systemverilog_preprocessor_parser`; mirrors `parse_and_cover_json`
  — no profile, no worker stack) + the registry entry set to `Some(...)`. Same one-time bootstrap-ordering
  step as H.4 (stale parser present → direct regen first). VERIFIED: `make focus_systemverilog_preprocessor`
  regenerates the parser with the coverage methods; `ast_pipeline grammars/systemverilog_preprocessor.ebnf
  --report-certificate-coverage --entry-rule systemverilog_preprocessor_file --count 40` RUNS at the DEFAULT
  depth 24 (no `--max-depth` needed), DETERMINISTIC same-seed: seed 0 → `total=73 proof=0 witness=19
  UNKNOWN=54 fully_certified=false (sample_parse_failures=24)`; seed 7 → `witness=7 UNKNOWN=66
  (sample_parse_failures=32)`. `parser_registry` lib tests pass; lib `--features generated_parsers` builds
  clean; json (`fully_certified`) / regex / rtl_const_expr cert-coverage UNAFFECTED. NO tracked-artifact
  change (`generated/` is gitignored). **NEW FINDING (attribution rule — routed, NOT accepted): svpp has a
  HIGH witness-parseability residual** (24/40 `sample_parse_failures` @ seed 0; regex H.1 had 6/200, json 0).
  Tools-first peek (`--generate-stimuli`): the svpp stimuli generator over-produces STRUCTURALLY-INVALID
  preprocessor inputs — comment-dominated samples, `` `define `` with no valid macro name/body, stray
  punctuation (`,`/`=`/`)` as bare `pp_item`s) — so they don't re-parse. This is a GENERATOR↔svpp-grammar
  round-trip gap (same class as regex's witness-parseability residuals + SV G.4.7), now MEASURABLE for the
  first time because svpp is wired. Routed to a follow-up **`H.5.1`** (svpp witness-parseability root-cause:
  generator deficiency vs loose `pp_item` grammar productions — adjudicate per the attribution rule; likely
  needs a `parse_detail` adapter for svpp to label the exact errors, like SV's). The H.5 DELIVERABLE
  (cert-coverage WIRED + RUNS deterministically for svpp) is complete; driving its residual + `UNKNOWN`→0
  is the follow-up (mirrors regex H.1 landing wired-with-residuals). Frontier: `H.5.1` (svpp residual) +
  rtl_frontend (H.6) next, then vhdl.
- `H.6` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0040`, 2026-06-08): Phase H continues — `parse_and_cover`
  wired for `rtl_frontend` (the ~5.5 MB synthesizable-RTL frontend parser); `--report-certificate-coverage`
  now runs for rtl_frontend.** Same mechanical recipe as H.5/H.4. `rtl_frontend` has no `@profiles`, its
  entry `rtl_frontend_file := trivia design_item* trivia` is a FLAT `design_item*` list (like svpp's
  `pp_item*`, NOT a deep precedence chain like rtl_const_expr), and its regen is ZERO-drift (fresh json from
  ebnf byte-identical to the on-disk json except `generated_at` + the embedded `source_file` invocation-path
  string — `diff` excluding both = empty), so it runs at the DEFAULT depth 24 (NO `--max-depth` needed —
  confirmed empirically: the run does NOT bail or fatal-error at 24, unlike rtl_const_expr). Landed: (1)
  `RTL_FRONTEND_EBNF/JSON/PARSER` vars + build rules + `rtl_frontend_parser` + `focus_rtl_frontend` Makefile
  targets (mirror svpp/rtl_const_expr); (2) `parse_and_cover_rtl_frontend` in `parser_registry.rs` (cfg
  `has_generated_rtl_frontend_parser`; mirrors `parse_and_cover_systemverilog_preprocessor` — no profile, no
  worker stack) + the registry entry set to `Some(parse_and_cover_rtl_frontend)`. Same one-time
  bootstrap-ordering step as H.4/H.5 (stale pre-coverage parser present → direct-regen the parser with the
  already-built generator binary first, else E0599). NOTE (new vs H.4/H.5): `rtl_frontend.ebnf` parses only
  under `--features ebnf_dual_run`, so the cert-coverage binary must be built `--features "ebnf_dual_run
  generated_parsers"` (json/svpp parsed without ebnf_dual_run). VERIFIED: the direct regen produces the
  parser carrying the coverage methods (grep `enable_coverage`/`exercised_rule_names` = 2); `ast_pipeline
  grammars/rtl_frontend.ebnf --report-certificate-coverage --entry-rule rtl_frontend_file --count 40` RUNS
  at the DEFAULT depth 24, DETERMINISTIC (seed 0 run#1 == run#2, and seed 0 == seed 7): `total=170 proof=0
  witness=37 UNKNOWN=133 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`.
  **`sample_parse_failures=0`** ⇒ rtl_frontend has NO witness-parseability round-trip residual (clean, like
  json/rtl_const_expr; cleaner than regex H.1's 3 and svpp's 24). UNKNOWN=133 is an honest residual (driving
  it to 0 via more/targeted witnesses is a follow-up). `parser_registry` lib tests 20/0 (`--features
  generated_parsers`, incl. the rtl_frontend adapter); strict SOURCE clippy ok (`clippy_source_all_targets`
  passes — `parse_and_cover_rtl_frontend` is clean; generated stage non-strict, pre-existing codegen debt
  only); json (`fully_certified=true`) / regex / rtl_const_expr (`48/41/7` @ `--max-depth 32`) / svpp
  (`73/19/54`, 24 fails) cert-coverage UNAFFECTED (default depth unchanged; byte-identical to their leaves).
  NO tracked-artifact change (`generated/` is gitignored; the regen is local). Default build (rtl_frontend
  cfg off in a fresh clone) unaffected. Frontier: **vhdl (H.2)** is the last unwired shipped grammar — its
  fresh-json regen chain is de-risked by H.4 + this slice's checkout-illusion zero-drift proof — then the
  residual drives (svpp `H.5.1`, each grammar's `UNKNOWN`→0). NOT wired: ebnf, return/semantic_annotation.
- `H.5.1` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0042`, 2026-06-08): LABEL the svpp witness-parseability
  residual — wire svpp `parse_detail` so the cert-coverage report shows the EXACT per-sample parse error
  (the tools-first enabler for the root-cause), then READ the labels + adjudicate (generator over-production
  vs loose grammar).** H.5 surfaced svpp's HIGH residual (24/40 `sample_parse_failures` @ seed 0) but the
  cert-coverage report could only print `"(no detail-capable parser registered)"` for each failure because
  svpp's registry entry had `parse_detail: None` (unlike SV, whose `parse_detail` labels its failures since
  G.4.7). This slice WIRES it: a thin `parse_with_systemverilog_preprocessor_detail_profile(sample, _profile)`
  adapter (matching `ParseDetailFn = fn(&str, Option<&str>) -> Result<(), String>`; svpp has no profile so the
  arg is ignored) delegates to the already-existing `parse_with_systemverilog_preprocessor_detail`, and the
  svpp registry entry's `parse_detail` is set `None → Some(...)`. `parse_error` (`parser_registry.rs:535`)
  reads the registry field directly, so the cert-coverage report's `SAMPLE-PARSE FAILURES` block now shows
  the real svpp parse error + sample for each failing witness. TOOLS-BACKED EVIDENCE (`--generate-stimuli
  --count 40 --seed 0` + the now-labeled cert-coverage run): the labeled error is **`Parser did not consume
  full input at position 0`** on a sample that LEADS with `` `define/***/ `` — a `` `define `` whose only
  following content is a comment, i.e. NO macro name — so the parser rejects on the very first `pp_item`.
  The generator over-produces STRUCTURALLY-INVALID `pp_item`s — bare identifiers (`_mO`, `qNR`), stray
  punctuation as a bare item (`,`, `=`, `)`, `s=`), comment-only lines, and directives with no valid payload
  (`` `define `` / `` `include `` / `` `celldefine `` followed only by a comment). Corroborating: `pp_define`,
  `macro_formals`, `macro_body`, `macro_reference`, … are ALL in the `UNKNOWN` set (never witnessed valid),
  so the generator NEVER emits a well-formed `` `define NAME body `` — it always degrades to the
  name-less/comment-only form. ADJUDICATION (per the attribution rule + EBNF-single-source-of-truth): this is
  a GENERATOR-side deficiency (it under-fills the required macro name/payload of the directive productions),
  not a parser bug — the fix belongs in the generator (or, if `pp_define`'s name is grammar-optional, in the
  grammar). The actual fix is the follow-up `H.5.1.1`. NO grammar/
  **⚠️ ADJUDICATION CORRECTED by `H.5.1.1` (`-0044`):** the "under-fills the required macro name/payload"
  framing was an INFERENCE and is DISPROVEN. The byte-exact isolation + parser trace show the generator DOES
  emit a valid macro name; the real mechanism is the faithful-spacing trailing guard injecting a bare `\n`
  (`regex_tail_greedy_blocker` returns `"\n"` for the whitespace-only class `[ \t]+`), which pushes the macro
  name onto the next line where the line-oriented parser's `inline_trivia` cannot consume the bare newline.
  See the `H.5.1.1` entry for the proven WHERE+WHY + closed-loop A/B (24→8). This is a textbook case of
  [[feedback_be_alert_root_cause_fishy_immediately]] (a label/inference ≠ the proven mechanism).
  generator behaviour change in this slice (labeling only; the residual count is unchanged). VERIFIED: lib
  `--features generated_parsers` builds; `parser_registry` lib tests pass; svpp cert-coverage now prints the
  labeled errors; strict SOURCE clippy ok. NO tracked-artifact change (`generated/` untracked). Frontier:
  `H.5.1.1` (adjudicate + fix the svpp residual from the now-visible labels) + drive each wired grammar's
  `UNKNOWN`→0.
- `H.5.1.1` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0045`, 2026-06-08): root cause PROVEN tools-first + SURGICAL
  generator fix landed (whitespace-only greedy-tail guard), GLOBAL-measured, zero cross-grammar regression.**
  Goal: the svpp stimuli generator emits well-formed preprocessor inputs so svpp cert-coverage
  `sample_parse_failures` drops and `pp_define`/`macro_*` move out of `UNKNOWN`.
  - **FIX (`-0045`):** in `StimuliGenerator::regex_tail_greedy_blocker`
    (`rust/src/ast_pipeline/stimuli_generator.rs`) return `None` when the greedy unbounded tail class is
    WHITESPACE-ONLY (new helper `regex_class_is_whitespace_only`, checking range bounds against the ASCII ws
    set 0x09–0x0d + 0x20 without per-codepoint iteration). A trailing whitespace run needs NO anti-fusion guard
    (whitespace self-separates), so the old `Some("\n")` escalation for `[ \t]+` was both pointless and harmful
    in line-oriented grammars. Parser-agnostic + general (derived from the regex class, benefits any grammar);
    makes the generator MORE faithful to the EBNF (the single source of truth) — strict fix hierarchy: no
    grammar/annotation/store level applies because the grammar is already correct (`macro_name` is required,
    `inline_trivia` already excludes `\n`); the defect was purely the generator's grammar-derived spacing.
  - **VERIFIED (decisive A/B + global metric):** svpp cert-cov (count 40 seed 0) `sample_parse_failures`
    **24→8**, `witness` **19→66**, `UNKNOWN` **54→7** (`pp_define`/`macro_formals`/`macro_body`/… now witnessed).
    DECISIVE stash-baseline A/B (pre-fix vs post-fix, identical invocation) over json/regex/rtl_const_expr/vhdl/
    rtl_frontend: **byte-identical metrics — ZERO regression** (only svpp changed; static proof confirmed: svpp's
    `[ \t]+` is the ONLY `\n`-excluding greedy whitespace class across ALL grammars — every other uses
    `[ \t\r\n]+`/`\s+` which already returned `None`). New unit test `whitespace_only_greedy_tail_gets_no_separator`
    locks the behavior + its boundary (mixed class `[ \ta-z]+` still escalates; `[^\n]*` line comments untouched).
    no-features lib **621/621**; clippy source-strict clean; **cross-family stimuli platform gate PASS**
    (regex/vhdl/SV). No regen / no `generated/` change (the fix is in the runtime generator, not codegen).
  - **RESIDUAL 8 (follow-up, NOT this fix's class):** the target class (`` `define ``+bare-newline-before-name) is
    FULLY eliminated. The 8 remaining svpp failures are different, pre-existing mechanisms: (a) `\b`-keyword↔
    word-char macro-name spacing gap (`` `ifndefR7Sh ``, `` `timescale63_ `` — the deferred space from the join
    rule not landing for a `\b`-terminated keyword token); (b) deeper structural (`` `define NAME(formals)``
    macro_formals, `` `timescale … ms`` time_literal). Route under the `UNKNOWN`→0 drive (own leaves).
  - **INVESTIGATION DONE (`PGEN-GRAMMAR-WELLFORMED-0044`, this slice) — PROVEN ROOT CAUSE (byte-exact
    isolation + parser trace + closed-loop A/B; corrects the H.5.1 adjudication).** The FIRST STEP (a focused
    parser trace of the labeled sample) is complete and decisive:
    - **WHERE (parser side, PROVEN by `parseability_probe --trace-rules`):** the exact rejecting production
      is `identifier` (`macro_name := identifier := inline_trivia /[a-zA-Z_][a-zA-Z0-9_$]*/`). On the labeled
      sample `[0]` (`` `define/***/  \n… ``) the trace shows: `pp_define→kw_define` matches `` `define `` at 0;
      `macro_name→identifier→inline_trivia` consumes `/***/` (block comment, 7→12) + `  ` (`space_or_tab`, 12→14);
      then the identifier regex `[a-zA-Z_][a-zA-Z0-9_$]*` is attempted at position **14 — a bare `\n`** — and
      fails (`rule_stack=[…, pp_define, macro_name, identifier]`). `macro_name`→`pp_define` fail. Because
      `pp_define` is the FIRST `pp_item` and every other `pp_item` alt also fails at 0 (none start
      `` `define ``; `non_directive_text`/`newline` cannot consume the leading backtick), `pp_item*` matches
      **zero** items and `systemverilog_preprocessor_file` trivially succeeds consuming nothing → "Parser did
      not consume full input at **position 0**".
    - **WHERE+WHY (generator side, PROVEN by byte-exact `--stimuli-corpus-json` isolation):** the bare `\n` is
      **injected by the faithful-spacing (lexical-cohesion) trailing guard**, NOT by the regex sampler and NOT
      by an under-filled macro name. `space_or_tab := /[ \t]+/` generates `'    \n'`, `' \n'`, `'   \n'` (every
      sample ends in a bare `\n`); with `--no-word-boundary-spacing` it correctly generates `'    '`, `' '`
      (no `\n`). `inline_trivia := (space_or_tab|block_comment)*` accumulates these (`' \n \n \n'`). EXACT CODE
      SITE: `StimuliGenerator::regex_tail_greedy_blocker` (`rust/src/ast_pipeline/stimuli_generator.rs:7314-7329`),
      reached via `apply_word_boundary_spacing`→`regex_terminal_trailing_separator`. For an open-ended greedy
      class-repetition tail it appends "the minimal separator the class cannot absorb": for `[ \t]+` the class
      contains `' '` (so `" "` is skipped) but NOT `'\n'`, so it returns `Some("\n")`. In a LINE-ORIENTED
      grammar where `\n` terminates directives and is excluded from `inline_trivia`, that injected `\n` pushes
      the macro name onto the next line where the parser's `inline_trivia` (correctly) cannot consume the bare
      `\n` → the identifier fails. The macro name **is** generated — it just lands off-line. (This DISPROVES
      H.5.1's "the generator under-fills the required macro name/payload" inference.)
    - **CLOSED-LOOP A/B (PROVEN dominant cause):** full svpp file, 40 samples, seed 0, re-parsed: **default
      24/40 fail** (== cert-cov `sample_parse_failures=24`); **`--no-word-boundary-spacing` 8/40 fail**. So the
      `\n`-injection causes **16 of 24** failures. The residual 8 (spacing-off) are a MIX dominated by
      keyword-fusion failures NEWLY introduced BY disabling spacing (e.g. `` `ifndefR7Sh ``, `` `timescale63_ ``
      — a `\b` keyword immediately followed by a word char with no separator), which is the very thing faithful
      spacing exists to prevent. ⇒ disabling spacing is NOT the fix; the fix must be SURGICAL.
  - (The fix that this investigation pointed to landed in `-0045` — see the `H.5.1.1` DONE record above.)
- `H.5.1.2` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0046`, 2026-06-08): drove the svpp residual-8 CLASS (a) — the `\b`-keyword↔word-char directive-keyword
  fusion (`` `ifndefR7Sh ``, `` `define_CV ``, `` `ifdefyDY_1 ``, `` `elsifLgz ``, `` `default_nettyped ``,
  `` `timescale6 ``) — using the DECLARATIVE `[>! /\w/]` lexical-annotation (LEXICAL-ANNOTATIONS Obligation C),
  the construct introduced SPECIFICALLY for the stimuli generator (director directive 2026-06-08).** Fix-hierarchy
  LEVEL 1 (existing semantic annotation) — preferred over the level-5 generator-engine patch the investigation
  below also identified; leaves the generator/engine untouched per [[feedback_prefer_grammar_leave_engine_alone]],
  and aligns with the book's stated direction (the derived `enforce_word_boundary_spacing` heuristic "will be
  subsumed by" the declarative annotation, `docs/book/src/lexical-annotations.md`).
  - **ROOT CAUSE (PROVEN tools-first, source-read + empirical):** every directive keyword is
    `kw_X := inline_trivia /` + "`" + `X\b/` (`grammars/systemverilog_preprocessor.ebnf:210-221`); the trailing
    `\b` asserts a word boundary must follow. The generator's DERIVED guard `apply_word_boundary_spacing`
    (`stimuli_generator.rs:7232`) correctly computes `regex_terminal_trailing_separator = Some(" ")` for the
    trailing `\b`, then takes the LEXICAL-ANNOTATIONS.5 successor-aware DEFERRAL arm (`:7266`, returns the
    candidate so the join rule inserts the space iff the next token starts with a word char) — BUT then sets
    `last_terminal_word_shaped = is_word_shaped_literal("` + "`" + `define")` = **false** (`:7421` requires the WHOLE
    token be word-chars; the leading backtick fails it). The join rule `append_generated_segment` (`:7575`) gates
    the deferred space on `prev_tail_word_shaped == true`, so it is silently DROPPED → `` `define ``+`_CV` fuses →
    the parser's `/` + "`" + `define\b/` rejects (`\b` fails between `e` and `_`). Empirically confirmed:
    `--generate-stimuli --count 40 --seed 0` emits `` `define_ ``, `` `ifndefR ``, `` `ifdefy ``, `` `elsifL ``,
    `` `default_nettyped ``, `` `timescale6/7/1 `` (the exact cert-coverage `sample_parse_failures` samples).
  - **FIX:** add the column-0 before-rule directive `[>! /\w/]` above each directive keyword rule in
    `grammars/systemverilog_preprocessor.ebnf` (the exact canonical book example). FORBID self-terminates the
    keyword's rendered output via `apply_lexical_follow_restriction` (`:7184`, EAGER bake) — bypassing the buggy
    derived deferral entirely — so the keyword can never fuse with a following word char. Parser-agnostic, and
    `[>! …]` is a documented parse-direction no-op so the generated svpp PARSER is unchanged (no regen/behaviour
    change on the parse side).
  - **FRONTEND ENABLER (a SECOND, GENERAL bug the grammar fix surfaced — PROVEN tools-first):** with the 12
    `[>! /\w/]` directives added, ONLY `kw_define` (the FIRST) bound — `--generate-stimuli` still fused
    `` `undef ``/`` `ifndef ``/`` `timescale ``/`` `default_nettype `` with the next word char, and the raw-AST
    envelope carried just **1** `forbid` token (should be 12). ROOT CAUSE: `collect_rule_body`
    (`ebnf_frontend.rs:313`) terminated a rule body on a top-level `#`/`@`/include/rule-header line but NOT on a
    `[>` directive line, so a SINGLE-LINE rule greedily ABSORBED the *next* rule's `[>! …]` directive into its own
    body, stealing it from the rule it was meant to bind — only the first directive in a stacked run survived.
    Latent because no grammar had ever placed consecutive `[>` directives. FIX (one general line): add
    `|| trimmed.starts_with("[>")` to `collect_rule_body`'s break set, making `[>` a body terminator exactly like
    `@` (mirrors the `scan_top_level_rules` `[>` handler). Parser-agnostic, behaviour-inert for every grammar
    without `[>` lines (all of them today except svpp — verified byte-identical). Regression-locked by new test
    `consecutive_before_rule_lexical_annotations_each_bind_their_own_rule` (`ebnf_frontend.rs`).
  - **WHY NOT the generator-code fix:** the alternative (set `last_terminal_word_shaped = true` in the deferral
    arm) is a correct, more-general level-5 ENGINE change, but the strict fix hierarchy
    ([[feedback_no_workarounds_fix_hierarchy]]) mandates using the existing level-1 annotation first when it
    cleanly solves the grammar's problem — which it does. The underlying derived-guard quirk is recorded here as a
    known generator finding (moot for svpp once declared); revisit only with a concrete case the annotation can't
    cover.
  - **DIRECTOR-FEEDBACK PROVENANCE:** this slice surfaced [[feedback_full_startup_read_includes_mdbook]] — I had
    skipped `docs/book/` at startup (ramp-up item #2) and missed this construct; the director pointed it out. New
    KM card `docs/knowledge/lexical-follow-restrictions.md`.
  - **VERIFIED (decisive, tools-backed):** svpp cert-coverage (count 40 seed 0) `sample_parse_failures`
    **8→1**, `witness` **66→69**, `UNKNOWN` **7→4**, DETERMINISTIC (re-run identical); seed 7 `sample_parse_failures=1`
    (witness 70, UNKNOWN 3). Class (a) keyword-fusion is ELIMINATED (no `` `define ``/`undef`/`ifndef`/`timescale`/
    `default_nettype`-fusion samples remain; `--generate-stimuli` keyword-fusion grep EMPTY; every `kw_*` keyword
    self-terminates with a space in isolation). The remaining **1** failure is a DIFFERENT, pre-existing class
    (a `` `elsif ?…?) , `` stray-punctuation `condition_expr`/`directive_tail` over-production — NOT class (a),
    NOT introduced here), and the UNKNOWN set is now `macro_default_text`/`directive_tail`/`trivia`/`line_comment`
    (none are keyword/macro_name rules — those are now witnessed). NO cross-grammar regression: json cert-coverage
    `fully_certified=true` (total 9/witness 9/UNKNOWN 0) + regex cert-coverage `sample_parse_failures=0` UNCHANGED
    (the frontend fix is inert for grammars without `[>` lines). no-features lib **621/621**; new + existing
    binding tests pass; source-strict clippy `clippy_source_all_targets` ok (generated stage non-strict,
    pre-existing codegen debt only — no parser regen); **cross-family stimuli platform gate PASS** (regex/vhdl/SV).
    Tracked changes: `grammars/systemverilog_preprocessor.ebnf` (12 `[>! /\w/]` directives) + `rust/src/ebnf_frontend.rs`
    (`collect_rule_body` break-on-`[>` + regression test) + docs/book lexical-annotations chapter. `generated/` is
    untracked (regen is local; FORBID is parse-noop so the svpp parser is unchanged either way). RESIDUAL after this
    slice = class (b)/(c): `condition_expr`/`directive_tail`/`macro_formals` (`` `define NAME(formals)``) +
    `time_literal` (`` `timescale N unit ``) over-production → its own follow-up leaf (`H.5.1.3`), folded into the
    `UNKNOWN`→0 drive.
- `H.5.1.3` — **`in_progress`: drive the svpp residual-1 — a structural-content-class `\n`-injection
  (the SAME mechanism class as H.5.1.1, for a content class instead of a whitespace one).**
  - **ROOT CAUSE (PROVEN tools-first — byte-exact `--stimuli-corpus-json` isolation + minimal repro):** the
    failing witness is a nested `pp_conditional` whose second `` `ifndef `` never reaches its `pp_endif` because a
    stray `macro_stringize` (`` `" ``) lands at a `pp_item` boundary (and `` `" `` is not a valid standalone
    `pp_item`). The stray token is created by the generator INJECTING a bare `\n` inside a `condition_expr`:
    `condition_text := inline_trivia /[^` + "`" + `(),?:!|&\r\n]+/` is an open greedy class that excludes `\n`, so
    `StimuliGenerator::regex_tail_greedy_blocker` returns `Some("\n")` (space IS in the class → not the `" "`
    branch; `\n` is NOT → the `"\n"` branch). When a `condition_text` atom is followed by another `condition_atom`
    (e.g. a `` `" `` stringize), that injected `\n` terminates the `condition_expr` line, stranding the stringize.
    MINIMAL REPRO: `` `ifndef X⏎`elsif a⏎`"⏎`endif⏎ `` FAILS (stray `` `" `` after the injected/real `\n`), but
    `` `ifndef X⏎`elsif `"⏎`endif⏎ `` (stringize INSIDE the condition, same line) PARSES. The `\n` guard is
    UNNECESSARY (the next `condition_atom` always starts with a char EXCLUDED from `condition_text`'s class —
    backtick/paren/punct — so it self-delimits, no fusion) AND HARMFUL (`\n` is structurally significant: it
    terminates a directive/condition line).
  - **REJECTED FIX (class-only heuristic — UNSOUND, empirically disproven, reverted):** I first tried refining
    `regex_tail_greedy_blocker` to inject the `\n` only for a "line-rest" class (one containing all printable ASCII
    `0x20..=0x7e`) and return `None` for a structural content class like `condition_text`. This is UNSOUND: a class
    such as `[ \ta-z]+` (contains space + word chars, excludes `\n`) is NOT line-rest yet genuinely DOES need the
    `\n` guard (a following lowercase token fuses, and a space cannot separate it — space is in the class). The
    existing regression test `whitespace_only_greedy_tail_gets_no_separator` (`stimuli_generator.rs:10704`) asserts
    exactly `[ \ta-z]+ -> Some("\n")` — a CORRECT invariant — and the heuristic broke it (`left: None, right:
    Some("\n")`). Per [[feedback_corpus_expected_from_spec_not_fix]] I will NOT change a correct test to match a
    wrong fix. ROOT REASON the heuristic can't work: `condition_text` *includes* word chars (so by its class alone
    it COULD fuse with a word-char token) but its GRAMMATICAL successors are all excluded delimiters — a property
    only successor-awareness can see, not class analysis. Reverted; `stimuli_generator.rs` is byte-identical to
    HEAD again; the invariant test passes.
  - **SOUND FIX (identified, scoped — a generator-core change):** the correct fix is the SUCCESSOR-AWARE `\n`
    deferral — the exact analogue of the LEXICAL-ANNOTATIONS.5 space deferral, extended to the `\n`/class case:
    do not eager-bake `Some("\n")`; instead record the open tail CLASS, and at the concat join insert `\n` only
    when the NEXT emitted token's first char is ABSORBED by that class (would actually fuse). For `condition_text`
    the next `condition_atom` starts with an excluded delimiter → no `\n` (FIX); for `[^\n]` line bodies a
    following char fuses → `\n` (preserved); for `[ \ta-z]+` a following lowercase fuses → `\n` (preserved). This
    is sound and parser-agnostic, but it touches the concat/join path (`append_generated_segment` /
    `append_segment_tracked` + a new generator field holding the deferred `Class`) that affects EVERY grammar, so
    its blast radius is wider than a surgical leaf and warrants the full decisive cross-grammar A/B.
  - **CHECKPOINT (signoff decision — STOP, do not land):** root cause PROVEN tools-first; the cheap fix is unsound;
    the sound fix is a generator-core enhancement. Per the standing "leave the engine alone unless a parser-agnostic
    enhancement is explicitly justified" preference ([[feedback_prefer_grammar_leave_engine_alone]]) and
    [[feedback_always_signoff_decisions]] (a result that can't be soundly verified → STOP + checkpoint), this leaf
    stays `in_progress` pending the director's approach call: (a) implement the successor-aware `\n` deferral as its
    own careful slice (`H.5.1.3.1`, full cross-grammar A/B), or (b) route it to Phase C (the constructive-generator
    home — `B2`/`C` bounded-ordered backtracking / successor-aware construction). This is a RARE residual (1/40 ≈
    2.5%, the last svpp `sample_parse_failures` at seed 0); the wired-cert-coverage program is unaffected.
- `H.5.1.3.1` — **`done` (2026-06-08; deferral IMPLEMENTED → tools-measured INSUFFICIENT → REVERTED; NO code
  landed; residual routed to Phase C). The director chose "implement now"; the successor-aware `\n`
  deferral — the `\n` analogue of the LEXICAL-ANNOTATIONS.5 space deferral —** `regex_terminal_trailing_separator`
  is UNCHANGED (it keeps returning `Some("\n")`, so the `[ \ta-z]+ → Some("\n")` invariant test still passes — the
  deferral is a HIGHER-level decision, not a trailing-separator change). In `apply_word_boundary_spacing` the
  `Some("\n")` case DEFERS instead of eager-baking: it records the open tail CLASS in a new generator field
  `last_terminal_newline_guard_class: Option<Class>` (mirroring `last_terminal_word_shaped`). The concat join
  (`append_generated_segment`, the single choke point) inserts `\n` ONLY when the next segment's first char is
  ABSORBED by that class (would actually fuse). Threaded through `append_segment_tracked` via a parallel per-scope
  local `prev_tail_guard: Option<Class>` (mirroring `prev_tail_ws`). CORRECTNESS: for `condition_text` followed by
  a `` `" `` stringize → next char `` ` `` is EXCLUDED → no `\n` (FIX); for `[^\n]`/`[^\r\n]` line bodies followed
  by any non-newline char → ABSORBED → `\n` (preserved); for `[ \ta-z]+` followed by a lowercase → ABSORBED → `\n`
  (preserved), followed by an uppercase → EXCLUDED → no `\n` (strictly MORE faithful than the old eager guard).
  Parser-agnostic; completes the successor-awareness the space guard already has.
  - **OUTCOME (tools-measured, IMPLEMENTED THEN REVERTED):** the deferral was fully implemented (new field
    `last_terminal_newline_guard_class`, `regex_terminal_newline_guard_class`, deferral arm in
    `apply_word_boundary_spacing`, class-aware insertion in `append_generated_segment`, threaded via
    `prev_tail_guard` through all 4 concat scopes; lib compiled, invariant tests green). **It did NOT fix the
    target residual** (svpp `sample_parse_failures` stayed **1** at seed 0). TOOLS-PROVEN WHY: the failing
    `` `elsif ( %4⏎/*comment*/ `" `` case has its successor segment START WITH A COMMENT (`/`), and `/` IS
    absorbed by `condition_text`'s class → the membership check fires the `\n` ANYWAY. But absorbing that
    comment is HARMLESS (it just extends the flexible `condition_text`); the `\n` is still wrong. ROOT TRUTH:
    `condition_text` needs NO `\n` guard EVER (it is a flexible atom in a `condition_atom+` list — any successor
    either is an excluded delimiter that self-separates, or is absorbed harmlessly), whereas `[^\n]` line
    comments DO need it — and that distinction is GRAMMATICAL CONTEXT, which NO context-free class check (eager
    OR successor-aware) can capture soundly. Incidental measured effect before revert: rtl_const_expr cert-cov
    witness 41→45 / UNKNOWN 7→3 (fails 0) — an RNG-path side effect, not the goal. REVERTED (`stimuli_generator.rs`
    byte-identical to HEAD; `[ \ta-z]+` invariant + lib green). **ROUTED:** the svpp `condition_text` residual is a
    LEXICAL-context asymmetry → **Phase C** (constructive generation that models the parse). The director's
    2026-06-08 reframe — the generator outputs garbage because it is not context-steered; "context == semantic
    fact store" — elevates the broader fix to **STORE-AWARE-GEN** (full semantic-store-aware, context-aware
    generation); see [[project_store_aware_generation]] AMENDMENT. (This specific residual is lexical-context, not
    a store fact, so it stays a Phase C item; the store work is the larger context-awareness program.)
- `H.5.1.3.2` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0049`, 2026-06-08): declarative `$text`-atomicity fix for the
  svpp `condition_text` `\n`-strand residual (a NEW approach the prior heuristic attempts did not try) — svpp
  cert-coverage now CLEAN.** ROOT CAUSE re-confirmed tools-first
  (full-sample trace, `-0048` baseline reproduced): svpp `sample_parse_failures=1` (seed 0), furthest_position
  718 = a `` `" `` macro_stringize stranded at a `pp_item` boundary (rule_stack `…pp_conditional →
  pp_elsif_branch → pp_item`); byte-exact context `` `elsif (/*…*/ %4⏎/*…*/ `" `` shows a bare `\n` injected
  after `condition_text "%4"` ends the `` `elsif `` line, orphaning the following `` `" `` condition_atom. The
  prior `H.5.1.3.1` PROVED "`condition_text` needs NO `\n` guard EVER" but could only try CLASS/SUCCESSOR
  heuristics (which cannot express "this rule needs no guard"). **NEW APPROACH:** the DECLARATIVE
  LEXICAL-ANNOTATIONS.6 atomicity signal — `condition_text … -> $text` — marks the rule one flat lexical
  span, and `apply_word_boundary_spacing` (`stimuli_generator.rs:7242`) suppresses the trailing guard
  UNCONDITIONALLY when `atomic_token_depth > 0`. This is the proven truth ("no `\n` guard ever for
  condition_text") expressed declaratively (fix-hierarchy LEVEL 1, no engine change), per
  [[feedback_full_startup_read_includes_mdbook]] ("is there already a declarative construct?"). TRADE-OFF
  being measured: `.6` cross-rule cohesion also glues an atomic rule to a preceding word char, which would
  MERGE a `macro_reference`+`condition_text` into one macro_reference — harmless (a valid merge, never a parse
  failure) but a generation-shape shift; and `condition_atom`'s `body:$1` changes from a raw Sequence to a
  flat string (a consumer-visible svpp AST-dump schema bump).
  - **OUTCOME (`PGEN-GRAMMAR-WELLFORMED-0049`, LANDED — director-approved the schema bump 2026-06-08).** The
    `$text` fix is DECISIVE + ROBUST: svpp `--report-certificate-coverage` `sample_parse_failures` **1 → 0** at
    seed 0 (witness 69, UNKNOWN 4) AND seed 7 (witness 70, UNKNOWN 3), DETERMINISTIC (seed-0 re-run identical).
    The proven-correct mechanism: `condition_text -> $text` marks the rule one atomic lexical span
    (LEXICAL-ANNOTATIONS.6 `rule_is_lexically_atomic`), so `apply_word_boundary_spacing`
    (`stimuli_generator.rs:7242`) returns the candidate UNCONDITIONALLY when `atomic_token_depth > 0` —
    suppressing the spurious trailing `\n` the `\n`-excluding content class otherwise drove. This is the prior
    `H.5.1.3.1` conclusion ("`condition_text` needs NO `\n` guard EVER") expressed DECLARATIVELY, which the
    class/successor heuristics structurally could not. Grammar-first (Level 1, no engine change), per
    [[feedback_prefer_grammar_leave_engine_alone]] + [[feedback_full_startup_read_includes_mdbook]].
  - **CONSEQUENCE — svpp AST-dump schema bump `3 → 4`, release/contract `1.0.4 → 1.0.5`** (annotating a
    deliberately-raw-envelope literal-text run is a consumer-visible change: a `condition_atom` "text" atom's
    `body` becomes a flat `$text` string `" abc"`, was the raw envelope `[[" "], "abc"]`). Realizes the svpp
    book's long-anticipated "annotate the literal-text runs" milestone for `condition_text` SPECIFICALLY —
    it is a flexible same-line `condition_atom+` element (successors self-delimit or merge harmlessly), so it
    needs no line terminator, whereas the line-oriented `directive_tail`/`non_directive_text` deliberately stay
    raw. Low real-world impact (svpp has no active named downstream consumer; uses the generic profile API).
    Director-approved the schema bump before the lockstep.
  - **VERIFIED:** svpp cert-coverage clean (both seeds, deterministic); svpp shape-contract test GREEN (manifest
    `condition_text` `return_scalar` entry added, inventory 66→67); `cargo test --features generated_parsers
    --lib` **686/0**; svpp book gate GREEN (19 HTML pages regenerated); cross-family stimuli gate (generator
    code byte-unchanged — svpp grammar-only change, svpp not in the cross-family set). LOCKSTEP: grammar
    (`condition_text -> $text`); manifest (`systemverilog_preprocessor_v1.json`); svpp contract (schema-4 row +
    identity 1.0.5/schema 4/67-annot); svpp book (schema-versioning / json-carrier / changelog-index /
    rules-top-level / walking-the-ast + regenerated HTML); continuity docs. `generated/` regen is local
    (untracked). Annotation count 66→67 (first `return_scalar`), distinct annotated rules 28→29.
- `H.5.2` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0052`, 2026-06-09): drove svpp `UNKNOWN 3→2` — STEP 1, removed
  the dead `trivia` rule (+ tightened the zero-plausible-gap proof gate to a literal-ZERO unreachable
  surface).** TOOL-BACKED ROOT
  CAUSE (cert-coverage `--count 40` AND `--count 200`, seeds 0/7 — SYSTEMATIC, not sampling): svpp's 3
  `UNKNOWN` rules are `directive_tail`, `trivia`, `line_comment`, and they split by the **attribution rule**
  (`--lint-grammar` is the adjudicator → svpp `unreachable_rules=0`, no shadowing/orphans):
  - **`trivia := (space_or_tab | line_comment | block_comment)*`** (this rule in `systemverilog_preprocessor.ebnf`
    ONLY — a same-named rule is ALIVE in `rtl_frontend.ebnf`, whose entry is `rtl_frontend_file := trivia
    design_item* trivia`; the two are independently-defined rules sharing a name). **OBJECTIVELY PROVEN DEAD
    (unreachable from the svpp entry), two independent lines:** (A) static reference-graph (decidable
    Hopcroft–Ullman reduced-grammar reachability; PEG references are static literal names → reachability =
    transitive closure of the reference graph): the only standalone `trivia` token in the file is its own
    definition (line 188) → NO rule body references it; no `@include` → no external referencer; svpp entry
    is `systemverilog_preprocessor_file` ≠ `trivia` → `trivia` is in no closure from the entry. (B) independent
    computational oracle `--gap-report-json` (`generate_gap_report`, a separate code path that does NOT read
    the grep or the contract): `trivia → reachable:false, reason:"unreachable_from_entry"`, and the COMPLETE
    statically-unreachable-from-entry set is exactly `["trivia"]` — while the SAME oracle classifies
    `directive_tail`/`line_comment` as `reachable:true, reason:"never_hit"` (proving it distinguishes dead
    from merely-unwitnessed). The linter's MULTI-ENTRY leniency (it treats an unreferenced rule as a candidate
    entry) is why `--lint-grammar` reports `unreachable_rules=0` and does NOT flag `trivia`; cert-coverage's
    single-entry witness is what surfaces it as `UNKNOWN`. (The zero-plausible-gap contract documents it as
    the sole `allowed_unreachable_rules: [trivia]` "helper pocket"; that is SAME-LINEAGE corroboration of the
    gap-report classification, NOT an independent proof.) Per the GRAMMAR-WELLFORMED literal-0 doctrine
    (dead rule = EBNF defect → **remove at source**, the book's worked-example pattern), this leaf removes it
    and tightens the gate's tolerated surface from `[trivia]` to `[]`.
  - **`directive_tail`** (reachable optional `directive_tail?` in `pp_if_branch`/`pp_else_branch`/`pp_endif`)
    and **`line_comment`** (reachable via `directive_comment_tail`'s `line_comment?`) — reachable optionals
    the diverse pass SYSTEMATICALLY never selects (UNKNOWN at count 40 AND 200, both seeds) and the H.4.2
    reach pass does not target (it fires only for RECURSIVE branches that fail with DEPTH-EXHAUSTION; these
    are shallow non-recursive optionals). Attribution = **generator-reach deficiency** → deferred to **`H.5.3`**
    (constructive-reach for never-selected reachable optionals; shared mechanism → re-measure every grammar).
  - **PLAN (this leaf):** (1) remove `trivia` from `grammars/systemverilog_preprocessor.ebnf`; cascade-safe —
    `line_comment`/`block_comment` keep other referencers; (2) relax the gate's two `allowed_unreachable_*
    | length > 0` JSON-shape conjuncts to allow an empty surface (its assertion logic already handles `[]`
    vacuously); (3) re-baseline the contract to `allowed_unreachable_*: []` (version 2→3), keeping the
    `helper_only_whitelist_detail` prefix the ci gate asserts. Schema/release UNCHANGED (dead rule never
    invoked → AST/behaviour identical). Expected: cert-coverage `UNKNOWN 3→2`; zero-plausible-gap gate GREEN
    with `observed unreachable surface == []`. [[feedback_grammar_edit_proof_gate_lockstep]] — owns the
    downstream proof gate in-slice.
  - **VERIFIED (LANDED):** svpp grammar edit (1 line removed) → regenerated parser drops `parse_trivia`
    (annotation inventory unchanged 67; trivia had no `->`). Independent gap-report oracle AFTER removal:
    complete statically-unreachable-from-entry set = `[]` (was `[trivia]`), `directive_tail`/`line_comment`
    still `reachable:true never_hit` (no cascade). cert-coverage (count 40): `total 73→72`, `witness 70`,
    **`UNKNOWN 3→2`** (`["directive_tail","line_comment"]`), `sample_parse_failures=0` — DETERMINISTIC seeds
    0/7. Downstream proof gate **`sv_preprocessor_zero_plausible_gap_proof_gate` GREEN** (exit 0; all 3
    sub-gates pass; `zero_plausible_grammar_level_gap_proof_surface=true`, `helper_only_unreachable_surface_green=true`,
    `observed_unreachable_rules_json=[]==allowed`, `unmet_proof_criteria_count=0`, `contract_version=3`).
    AST shape-contract test GREEN (AST unchanged → no manifest/schema/release change). Full lib
    `--features "generated_parsers ebnf_dual_run"` **716/0** (21 ignored); svpp lib tests 28/0 incl.
    `test_basic_parsing` + shape-contract. `clippy_on_rust_change` strict-SOURCE clean (no hand-written Rust
    touched; generated-stage lint debt pre-existing/non-strict, none svpp-related). LOCKSTEP: gate script
    (2 `length>0` shape conjuncts relaxed to allow `[]`), contract (v2→3, whitelists `[trivia]`→`[]`, kept
    the `helper_only_whitelist_detail` prefix the ci gate asserts), task/frontier/continuity docs, KM card
    `prove-rule-dead-or-reachable`. NO book/contract/schema change (dead internal rule; no user-facing impact).
    `generated/` regen local (untracked). svpp NOT yet fully_certified — the 2 reachable optionals are **`H.5.3`**.
- `H.5.3` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0053`, 2026-06-09): svpp `directive_tail` + `line_comment`
  now WITNESSED at every seed → svpp `fully_certified=true` at seeds 0/7/42. EVIDENCE-DRIVEN RE-SCOPE
  of the fix mechanism (declarative WITNESSING-sample steering, NOT the originally-assumed
  constructive-reach engine pass).** TOOLS-FIRST ROOT CAUSE (not the H.5.2 hypothesis): the 2 residual
  rules are statically reachable AND fully parser-supported (hand-written `` `ifdef A some tail\n`endif`` and
  `` `undef FOO // c`` parse, AST shows `directive_tail="some tail text"` / `line_comment="// a comment"`), so
  the gap is **100% generator-side** — and NOT a reach-budget gap. The cause is the two `@sample: " "` hints
  (history: `f3ce8a64`/`b333c6b5` directive_tail, `783c8c5c` directive_comment_tail — added to *reduce
  parser-rejection debt* BEFORE the lexical-faithfulness machinery matured, 2026-06-07/08): (a)
  `directive_tail`'s `@sample: " "` emits a bare space that the leading `inline_trivia` consumes, so the
  mandatory `/[^\r\n]+/` body matches nothing on re-parse → `directive_tail?` takes the EMPTY arm → never
  witnessed; (b) `directive_comment_tail`'s `@sample: " "` short-circuits the whole rule, so the inner
  `line_comment?` is NEVER expanded → `line_comment` never constructed. So the fix is **grammar-level
  declarative** (fix-hierarchy LEVEL 1), NOT the constructive-reach engine route the leaf was scoped around —
  per [[feedback_full_startup_read_includes_mdbook]] (ask whether a declarative construct already fits before
  reaching for a level-5 engine patch). **FIX:** replace the un-witnessable `@sample: " "` with WITNESSING-and-
  faithful samples — `directive_tail` → `@sample: " x"` (survives `inline_trivia`, re-parses as
  `directive_tail`=`x`), `directive_comment_tail` → `@sample: " //"` (`line_comment := /\/\/[^\r\n]*/` accepts
  `//` with empty body, so `inline_trivia line_comment?` re-parses `//` → `line_comment` exercised). Both stay
  deterministic + benign (no free-content over-generation). **DECISIVE A/B (tools-first):** a pure REMOVAL of
  both hints also reaches `fully_certified` but RE-EXPOSED a latent over-generation (seed-7
  `sample_parse_failures` 0→1) — proving the hints were steering away a real round-trip hazard; the witnessing
  replacements keep `sample_parse_failures=0`. **MULTI-SEED BASELINE vs FIX** (count 40): original
  `seed0/7/42 UNKNOWN=2`, `seed1 UNKNOWN=6`; FIXED `seed0/7/42 UNKNOWN=0 fully_certified=true`, `seed1 UNKNOWN=4`.
  Seed 1's residual = `macro_default_value`/`macro_default_atom`/`macro_default_text`/`assign` — a PRE-EXISTING,
  seed-sensitive nested-optional reach gap (`macro_formal := macro_name (assign macro_default_value)?`), the
  SAME class but a DIFFERENT rule set H.5.2 never saw (it only measured seeds 0/7). Attributed + ticketed to
  **`H.5.4`** (NOT folded in — one slice, one concern; forcing a fixed macro-formal default would kill
  generation diversity). **VERIFIED:** svpp cert-coverage `fully_certified=true (UNKNOWN=0, sample_parse_failures=0)`
  deterministic seeds 0/7/42 (seed-0 re-run identical); directive_tail/line_comment gone from EVERY seed's
  UNKNOWN list; cross-grammar cert-coverage byte-identical (json `fully_certified`, regex `UNKNOWN=98`,
  rtl_const_expr `fully_certified`) — grammar-only change, generator code untouched; svpp parse smoke PASS;
  full lib `--features "generated_parsers ebnf_dual_run"` **716/0** (incl. svpp shape-contract — AST unchanged);
  `sv_preprocessor_zero_plausible_gap_proof_gate` GREEN; strict-SOURCE clippy clean. **NO AST / manifest /
  schema / release / contract / book change** (the `@sample` is generation-only steering; the svpp PARSER is
  byte-identical, AST shape unchanged → no user-facing impact). `generated/` regen local (untracked). svpp is
  fully_certified at the canonical seeds 0/7; full seed-robust `UNKNOWN=0` needs **`H.5.4`** (the macro-default
  nested-optional residual).
- `H.5.4` — **`pending`: witness the macro-formal-default nested-optional rules at the canonical count-40
  seed sweep.** Surfaced by H.5.3's multi-seed measurement: at seed 1 (count 40) the diverse pass does not
  witness `macro_default_value`, `macro_default_atom`, `macro_default_text`, `assign`, all reached only
  through the nested optional `macro_formal := macro_name (assign macro_default_value)?` (itself inside
  `macro_formals?` in `pp_define`). **TOOLS-FIRST FOLLOW-UP (H.5.3's sweep): this is a SAMPLE-BUDGET artifact,
  NOT a hard reach gap** — at seed 1 `--count 100` AND `--count 200` it is `UNKNOWN=0 fully_certified=true`
  (the rules ARE witnessable; the count-40 budget at seed 1/12 just doesn't happen to reach the nested
  optional). So svpp is genuinely certifiable at every measured seed; the residual is "reach the nested
  macro-default optional within the canonical count-40 budget at every seed". Seed scan (count 40): UNKNOWN>0
  only at seeds 1 (4) and 12 (4) of 0–15; all others 0. PRE-EXISTING + seed-sensitive. Approach options
  (decide tools-first): (i) constructive-reach for never-selected reachable OPTIONALS (general — also helps
  vhdl/regex/SV/rtl_frontend `UNKNOWN`), or (ii) a targeted declarative witnessing fix if level-1 suffices
  without killing macro-formal generation diversity. LOWER PRIORITY than `H.5.5` (this is a witness-budget
  micro-gap; H.5.5 is a genuine over-generation defect). Acceptance: svpp `fully_certified=true` deterministic
  across seeds 0/1/7/42 at count 40, zero cross-grammar regression. Verification: pending. Commit: pending.
- `H.5.5` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0055`, 2026-06-09, svpp release 1.0.5→1.0.6, schema stays 4):
  the svpp diverse-generation `sample_parse_failures` are now 0 across seeds 0–31 (acceptance was 0–15;
  verified to 31 for margin). EVIDENCE-DRIVEN RE-SCOPE: the defect was NOT a generator over-generation /
  `pp_conditional` closer-steal (the leaf's initial hypothesis) — it was a `condition_text` GRAMMAR GAP, fixed
  declaratively (fix-hierarchy LEVEL 1), exactly as `H.5.3` re-scoped its own mechanism.** TOOLS-FIRST ROOT
  CAUSE (`parseability_probe --trace-rules pp_conditional` on the captured seed-3 artifact, then per-seed for
  3/6/10/12/14/15): every outer `pp_conditional` failed to close because **an `` `elsif`` (or `` `ifdef``)
  condition contained a block comment with an operator char inside it** (seed 6 `|`, seed 10 backtick+`|`,
  seed 12 `&`, seed 15 `&`, seeds 3/14 same class). `condition_text := inline_trivia /[^`(),?:!|&\r\n]+/` is
  **not comment-aware**: its content regex ate the comment's `/*` (both `/` and `*` are in its class) then
  halted at the excluded char *inside* the comment → `condition_expr` (`condition_atom+`) couldn't span the
  comment → the elsif failed → the conditional never closed → file-level `pp_item*` stopped at the opening
  directive. Minimal repro: `` `ifdef A⏎`elsif x /*|*/⏎`endif`` REJECTED, `` …/*z*/… `` ACCEPTED. **A block
  comment is valid SV lexical trivia anywhere (incl. a condition), and `condition_text`'s SIBLINGS
  `macro_body_text`/`macro_default_text` were ALREADY made comment-aware for this exact reason
  (SV-EXH-PROOF.2.3.1 / SVPP-0002) — `condition_text` was simply missed by that slice.** So the parser was
  NOT "right to reject" (contra the leaf's initial framing); this is a grammar gap (EBNF-source-of-truth +
  the well-formedness attribution rule → fix the grammar, LEVEL 1), NOT a generator constraint. **FIX
  (grammar-only, `grammars/systemverilog_preprocessor.ebnf`):** `condition_text` made comment-aware with the
  proven sibling idiom `/(?:\/\*([^*]|\*+[^*\/])*\*+\/|[^`(),?:!|&\r\n])+/` (the `/*…*/` alt tried FIRST so a
  `/*` opener triggers atomic whole-comment consumption) AND its now-redundant leading `inline_trivia`
  DROPPED. **The inline_trivia drop is the decisive second half (DECISIVE A/B):** the comment-aware regex
  alone (keeping `inline_trivia`) flipped the 6 condition-comment PARSE failures into a NEW
  comment-only-condition GENERATION hazard — the generator could now emit `condition_text` = a comment, but
  the rule's own leading `inline_trivia` re-consumed it as trivia on reparse, stranding `` `elsif /* c */``
  (condition_expr needs ≥1 atom) → spf shifted to seeds 3/9/10 (traced: all furthest-backtrack at
  `condition_atom`, comment-only conditions). Dropping the now-redundant `inline_trivia` (the comment-alt +
  char-class already admit comments/whitespace) makes generation self-consistent: a comment-only
  `condition_text` round-trips (the punctuation atoms' `inline_trivia` backtracks cleanly, then `condition_text`
  matches the comment). **`$text` matched-SPAN is byte-identical** for every previously-parseable input —
  empirically: `` `elsif abc`` → `{kind:"text", body:" abc"}` (leading space preserved, exactly the contract's
  schema-4 value); `` `elsif a /* p||q */`` → `{kind:"text", body:" a /* p||q */"}` (comment captured in-span,
  parses). **VERIFIED:** svpp cert-coverage `sample_parse_failures=0` across seeds **0–31** (was 0 only at the
  canonical seeds before); `fully_certified=true UNKNOWN=0 spf=0` at canonical seeds 0 AND 7 (`total=72
  witness=72`); minimal repros (`/*|*/`, comment-only, original seed-3 1215-byte sample) all PARSE;
  cross-grammar cert-coverage BYTE-IDENTICAL (json `fully_certified`, regex `UNKNOWN=98`, rtl_const_expr
  `fully_certified` — svpp-only grammar edit); annotation inventory unchanged 67/29 (`condition_text` still
  `-> $text`); new `condition_comment_special_char` shape-contract sample locks the fix (mirrors
  `macro_body_comment_backtick`). Scattered `UNKNOWN=1` at a few non-canonical seeds (4/6/12/13/16/17/27) is
  the SEPARATE `H.5.4` macro-default witness-budget micro-gap (NOT spf, NOT introduced here — count 100/200
  closes it). NO engine/generator/codegen change (pure grammar + manifest sample). Release 1.0.5→1.0.6 (accept
  set widened: comment-bearing conditions now parse), schema STAYS 4 (no output-shape change — SVPP-0002
  pattern); ledger `SVPP-0003`. Verification: cert-coverage sweep 0–31, `sv_preprocessor_zero_plausible_gap_proof_gate`,
  shape-contract, full lib `--features "generated_parsers ebnf_dual_run"`, clippy. Commit: `PGEN-GRAMMAR-WELLFORMED-0055`.
- `H.7` — **CONSTRUCTIVE-REACH GENERALIZATION (the common enabler for the per-grammar `UNKNOWN`→0 drive).**
  Director-selected next direction (2026-06-09, after H.5.5). General, parser-agnostic stimuli-generator
  enhancement → per the standing director directive it is **discuss → own → design carefully → implement**;
  `.7.1` is the DESIGN (pure docs, no code), to be PRESENTED before any implementation.
  - `H.7.1` — **DESIGN `done` (pure docs, `PGEN-GRAMMAR-WELLFORMED-0056`): the targeted reach plan for
    never-witnessed NON-recursive reachable rules.**
    **PROBLEM (tools-first, the H.5.4 exemplar generalized):** after the diverse certification pass, several
    grammars retain `UNKNOWN` rules that are genuinely reachable + parseable but are simply never SELECTED
    within the bounded `--count` budget because the path to them runs through **un-taken optionals and/or an
    un-selected alternation branch** — NOT depth exhaustion, NOT recursion. Canonical exemplar: svpp
    `macro_default_text`, reached only via `pp_define → macro_formals? → macro_formal →
    (assign macro_default_value)? → macro_default_value → macro_default_atom`'s ONE text branch (two gating
    `?` optionals + an 8-way alternation). At count 40 it is witnessed on most seeds but missed on ~7/32
    (`H.5.4`). The same shape is the bulk of the residual `UNKNOWN` on vhdl (69), regex (98), rtl_frontend
    (133), SV (1126) — most are reachable rules behind optional/alternation gates, not the deep-recursion
    case `H.4.2` already solved.
    **WHY THE EXISTING REACH PASS DOESN'T COVER IT (root cause in code):** `H.4.2`'s
    `should_reach_retry_uncovered_recursive` (`stimuli_generator.rs:6891`) fires ONLY when ALL hold: the
    failure is **depth-exhaustion** (`is_depth_exhaustion_error`) AND the branch is **RECURSIVE** (references
    a rule already on the call stack). `macro_default_text` is neither — it never depth-exhausts (the path is
    shallow) and never recurses; the diverse pass just doesn't happen to take the two optionals AND pick the
    text branch within budget. So the existing retry is structurally inapplicable.
    **REUSABLE MACHINERY (use what we have — `feedback_prefer_grammar_leave_engine_alone` /
    `project_vision_and_discipline`):** the SV-EXH-PROOF.7.2 **reach-plan** system already does most of this:
    `compute_reach_path(entry, target_rule, target_node_path, target_branch_index)` computes the chain of
    OR-branch decisions to a target OR-branch; `set_reach_plan` / `ActiveReachPlan::from_directives` installs
    forced OR-branches; `forced_branch_for(rule, path)` returns the forced branch during generation;
    `reach_target_outcome` → `Reached` / `SelectedButFailed` / `NotReached`. The gap-report already classifies
    each residual `reachable_by_plan` vs `no_reach_path` (`reach_classification`, `generate_gap_report`). And
    `GrammarMutationSelection::Quantifier { forced_repeats }` + `StimuliDecisionTrace.quantifier_repeats`
    already model **forcing a quantifier's repeat count** (built for grammar-mutation replay). Purdom
    `witness_min_terminal_lengths` (`compute_min_terminal_lengths`) + `construct_mode`/`witness_mode` give the
    minimal off-path derivation. The two-pass cert-coverage report (`main.rs:run_certificate_coverage_report`)
    is the safe host: pass-1 diverse certification (byte-identical), pass-2 auxiliary reach that ONLY UNIONs
    re-parsing witnesses.
    **PROPOSED MECHANISM (MVP):** in the cert-coverage auxiliary reach pass, for each still-`UNKNOWN` rule R
    the gap-report classifies `reachable_by_plan`, (a) compute the reach path to R's defining site (R is
    itself an OR-branch, e.g. `macro_default_atom`'s text branch — or the rule that uniquely references R),
    (b) install a reach plan that forces BOTH the OR-branches AND **the optionals/quantifiers on that path to
    expand at least once** (the ONE genuinely-new capability — compose the existing OR-forcing reach plan with
    the existing `Quantifier{forced_repeats}` forcing, applied to the `?`/`*` nodes the reach path traverses),
    (c) generate in `construct_mode` so every OFF-path choice stays minimal, (d) union R's witness iff the
    sample re-parses. Bounded by the existing `MAX_UNCOVERED_REACH_RETRIES` backstop.
    **THE ONE NEW CAPABILITY** is "force optional expansion on the reach path": today the reach plan forces OR
    branches only, and `construct_mode` MINIMIZES optionals to zero (skip) — the opposite of what an
    optional-gated target needs. The design reuses `Quantifier{forced_repeats}` rather than inventing a new
    primitive.
    **SAFETY / NO-OP INVARIANTS (non-negotiable, the H.4.2 contract):** (1) opt-in to cert-coverage ONLY
    (gated by `reach_uncovered_recursive_branches`, or a sibling flag — open question Q2) → every other
    surface (`--generate-stimuli`, stimuli modules, cross-family + oracle gates) byte-identical; (2) the
    diverse certification pass is UNTOUCHED → `sample_parse_failures` (the certification number) byte-identical
    for every grammar — the reach pass can only make coverage BETTER, never certification WORSE; (3) the reach
    pass only UNIONs witnesses from samples that RE-PARSE (its own probe non-parses are reported separately,
    never folded into `sample_parse_failures`); (4) `UNKNOWN` can only DECREASE; (5) deterministic (seeded,
    step-budgeted).
    **VERIFICATION MATRIX (for `.7.2` implement):** svpp `UNKNOWN=0 fully_certified` across seeds 0–31 at
    count 40 (closes `H.5.4`); DECISIVE git-stash A/B — `sample_parse_failures` byte-identical pre/post for
    EVERY grammar (json/regex/vhdl/SV/rtl_const_expr/rtl_frontend), `UNKNOWN` only decreases (measure the
    vhdl/regex/SV/rtl_frontend deltas); non-cert-coverage surfaces byte-identical (cross-family + oracle +
    self-host gates green); determinism (seed re-run identical); lib `--lib` green; clippy strict-source clean.
    **OPEN QUESTIONS FOR THE DIRECTOR (before `.7.2` implement):** Q1 MVP scope — start with the
    optional+alternation reach class (the `macro_default_text` shape) only, or also the
    `reachable_rule_not_generated` rule-level class? Q2 reuse the `reach_uncovered_recursive_branches` flag or
    add a sibling `reach_uncovered_plannable_rules` (cleaner separation, two independent opt-ins)? Q3 the
    per-rule reach budget / backstop sizing for the big-`UNKNOWN` grammars (1126 SV rules × a reach attempt
    each is non-trivial — likely cap + report what was left, never silently truncate, per
    `feedback_severity_never_gated_by_verbosity`). **Verification: N/A (pure-docs design).** Commit:
    `PGEN-GRAMMAR-WELLFORMED-0056`.
  - `H.7.2` — **`done` (IMPLEMENT, engine, `PGEN-GRAMMAR-WELLFORMED-0058`): the targeted reach plan per `.7.1`.** The Q1–Q3
    design questions are RESOLVED BY THE AGENT per the director's standing "PNT yourself — do not
    involve me unless you can't decide" instruction + [[feedback_user_is_director_not_engineer]]
    (technical decisions with no real-world side effects are the agent's to make): **Q1** MVP scope =
    the optional+alternation reach class ONLY (the `macro_default_text` shape; one change at a time —
    the rule-level `reachable_rule_not_generated` class stays a follow-up leaf). **Q2** RESOLVED
    STRONGER THAN EITHER OPTION: no new config flag at all — the forced-quantifier state is CARRIED BY
    THE PLAN (`ActiveReachPlan.forced_quantifier_min`, empty for every existing caller), and the new
    pass is invoked explicitly by the cert-coverage report; the no-op invariant holds STRUCTURALLY
    (no plan installed → zero behavior change; existing plans carry empty maps → byte-identical
    replay), which is cleaner than flag-gating and trivially A/B-isolatable. **Q3** per-rule budget =
    ONE construct-mode attempt + ONE plan-forced search fallback (the proven witness-pass shape),
    bounded by the witness timeout; a deterministic global attempt cap with a LOUD
    "left unattempted" report (never silent truncation). Acceptance = the `.7.1` verification
    matrix.
    Verification: `done — IMPLEMENTATION (engine, parser-agnostic, all per the .7.1 design + reuse
    story): (1) reach_hops — the compute_reach_path BFS extracted UNCHANGED so both consumers share
    it (compute_reach_path re-verified byte-equivalent: directives derived per-hop at assembly
    instead of discovery, identical sequence); (2) quantifier_sites_along_path — one site per "q"
    segment, at the Quantified node's own path (the path generate_quantified receives); (3)
    ActiveReachPlan.forced_quantifier_min — plan-carried state, EMPTY for every pre-H.7.2 caller
    (from_directives never fills it) → existing replay byte-identical BY CONSTRUCTION; (4)
    set_reach_plan_for_rule — rule-target plan = hop OR-directives + hop quantifier sites forced to
    ≥1; (5) generate_quantified consults the plan FIRST (before construct-mode minimization),
    allocation-free when no plan/empty map; (6) generate_plannable_rule_witnesses — the driver:
    witness-mode env (Purdom ordering + depth/visit slack, all restored), per rule ≤4
    witness-checked attempts (construct first, plan-forced search fallback on generation failure;
    bounded 250ms each), verdicts via a CALLER callback (Witnessed/ParsedNotWitnessed/NotParsed) so
    the generator stays parser-agnostic; (7) main.rs PASS 3 in run_certificate_coverage_report —
    only-if-UNKNOWN-remain, unions covered rules from EVERY parsing probe, loud report line
    (targeted/witnessed/routed-elsewhere/probe-failures/generation-failures) + loud no-path
    dead-rule-candidate WARNING + loud global-cap leftover WARNING + opt-in
    PGEN_CERT_COVERAGE_DEBUG_PROBES per-probe print. RESULTS (count 40, canonical seeds):
    **regex UNKNOWN 98→19** (79 witnessed; 12 no-path = regex-literal-internal helper rules flagged
    for linter adjudication), **vhdl 69→31**, **rtl_frontend 133→75**, **SV 1134→738** (24s
    wall; 68 no-path = the multi-entry/number-literal family, A2.1 territory), **svpp
    fully_certified at 25/32 seeds** (closes the H.5.4 seed-1 gap; the 7 residual seeds are ALL
    UNKNOWN=1 macro_default_text). INVARIANTS: sample_parse_failures=0 preserved on EVERY grammar
    (the diverse pass untouched — pass-3 probe failures reported separately); json+rtl_const_expr
    stay fully_certified; determinism (seed re-run byte-identical); lib 629/0 no-features + 721/0
    dual-feature (+3 new tests: quantifier_sites_along_path extraction, the optional-gated
    end-to-end miniature incl. no_path reporting, retry-loop verdicts); full workspace 761/0;
    stimuli_cross_family_platform_gate PASS; clippy strict-source clean.
    **🐛 DISCOVERED PARSER BUG (the pass acting as the bug-finding oracle — the seed-12-class
    probes' parsed-but-routed-elsewhere verdict pointed straight at it):** legal IEEE 1800
    §22.5.1 `` `define M(a=x) y`` (default argument on the LAST formal) MIS-PARSES — decisive AST
    repro shows formals=[] and "(a=x) y" routed into the macro BODY, while `` `define M(a=x,b) y``
    parses correctly. Mechanism: macro_default_atom's bare rparen alternative + possessive PEG `+`
    lets macro_default_atom+ swallow the formals' closing ")" → macro_formals? backtracks to empty
    → body fallback. LRM 22.5.1: default text excludes a right parenthesis "not inside a balanced
    pair" — the grammar needs a BALANCED paren group atom, not bare lparen/rparen atoms. Owned by
    NEW leaf `H.9` (svpp grammar fix + release/ledger/contract/book lockstep; fixing it should take
    the svpp sweep to 32/32 fully_certified).`
    Commit: `PGEN-GRAMMAR-WELLFORMED-0058`.
- `H.10` — **the regex per-grammar `UNKNOWN`→0 drive** (the locked-program PNT continuation after
  `H.9`; regex = the smallest per-grammar residual). BASELINE (2026-06-10, release binary @ `-0059`,
  canonical `ast_pipeline grammars/regex.ebnf --report-certificate-coverage --entry-rule regex
  --count 40 --seed 0`): `total=210 proof=0 witness=191 UNKNOWN=19 sample_parse_failures=0`; reach
  pass: 98 targeted, 79 witnessed, 6 parsed-but-routed-elsewhere, 4 probe-failures, **12 no-path
  (dead-rule candidates)**. The 19 partition into exactly three classes (per-probe debug capture):
  (a) the 12 no-path rules; (b) probe-cohesion/terminal-selection residuals
  (`short_prop_letter` probe `\p C` + `ascii_restrict_modifier` probe `(?^a D-a D)` — bogus
  separator INSIDE the token; `letter_no_upper_e`/`unicode_char` probes `\Q\A\E*` parse but route
  elsewhere; `quoted_class_literal_escaped_char`); (c) the store-gated pair
  `numeric_backreference`/`backreference_digits` (`fact_count_at_least` semantic-prelude reach —
  B2/C1/C2 territory).
  - `H.10.1` — **`done` (dead-rule removal at source).** ADJUDICATION (tools-first, two
    INDEPENDENT oracles agree per [[prove-rule-dead-or-reachable]]): all 12 no-path rules are
    `reachable:false / reason:unreachable_from_entry` in the `--gap-report-json` oracle (the
    per-entry truth; `--lint-grammar unreachable_rules=0` is the known multi-entry-LENIENT view)
    AND the reach pass's rule-reference-graph search. All 12 are PROVEN leftover helpers orphaned
    by earlier documented rewrites: (a) `subroutine_ref`/`braced_subroutine_ref`/
    `signed_digits_or_name` — superseded by the INLINE `\g…` branches of `backreference`
    (`grammars/regex.ebnf:201-207`; the `(?&…)`/`(?P>…)` forms live in `subroutine_call`); the live
    comment above `backreference` still claims `ref` "carries the raw subroutine_ref shape" —
    STALE, fixed in-slice; (b) `name_start`/`name_continue` — `name` inlined them
    (REGEX-SELF-HOSTING.5c, the equivalence is already documented in the adjacent comment);
    (c) `prop_name_chars` — `prop_name` inlined; (d) `comment_char`/`comment_special` —
    `comment_text` rewritten (slice 20); (e) `directive_payload_char` — `directive_payload_simple`
    rewritten; (f) `directive_name_start`/`directive_name_continue`/`directive_special` —
    `directive_name` rewritten + the `-0008` strict/relaxed split. Per the literal-0 doctrine + the
    svpp `trivia` precedent (`H.5.2`): REMOVE at source. Removal is unreachable-only ⇒ the accepted
    language and every reachable AST shape stay byte-identical ⇒ NO release/schema bump (H.5.2
    precedent). LOCKSTEP: `rust/test_data/ast_shape_contract/regex_v1.json` drops the 6 dead-rule
    declared-annotation entries (`subroutine_ref` ×4 branches, `braced_subroutine_ref`,
    `directive_payload_char`) + calibration_history note; the `#[ignore]`d trace-of-parse-path test
    `regex_parser_integration_contract_classifies_numeric_angle_subroutine_ref`
    (`rust/src/embedding_api.rs:3936`) references the dead rule name but is quarantined AND its
    expectation already pre-dates the inline rewrite — left untouched (tracked with the ignored
    family); contract-doc mentions are ALL in append-only per-release history sections (provenance
    — untouched by the DOCPATH history convention). Acceptance: regex cert-coverage `UNKNOWN`
    19→7 with `sample_parse_failures=0` preserved (seeds 0/7/42, deterministic); accepted-language
    no-regression via `regex_pcre2_compile_oracle_gate` + the regex lib suites; gap-report
    `unreachable_rules` 12→0; lint hard gates stay green; cross-grammar untouched.
    Verification: `done — exactly as designed: 12 dead rules removed from `grammars/regex.ebnf`
    (each site keeps a one-line H.10.1 removal note; the stale `subroutine_ref` claim in the live
    `backreference` comment corrected to the inline-branch reality); manifest `regex_v1.json`
    annotations 173→167 (the 6 dead-rule entries dropped, `extracted_at` 2026-06-10); regenerated
    inventory exactly 167 — grammar↔manifest lockstep holds. VERIFIED: (1) regex cert-coverage
    `total 210→198, UNKNOWN 19→7, witness=191, sample_parse_failures=0`, IDENTICAL at seeds
    0/7/42 (the no-path warning is GONE; the 7 survivors are exactly the predicted H.10.2 pool);
    (2) gap-report oracle `reachable 198/198, unreachable_rule_debt 0` (was 12); (3) lint hard
    gates all green on the 198-rule grammar (the position-1307 dual-run warning is pre-existing,
    byte-identical pre/post); (4) `regex_pcre2_compile_oracle_gate` PASS (accepted language
    unchanged); (5) dual-feature lib 721/0 (same count as the `-0059` baseline — the auto-gate
    grammar↔manifest walkers agree); (6) cross-grammar untouched: json `fully_certified=true`,
    rtl_const_expr `fully_certified=true` at canonical count 40; (7) clippy strict-source clean
    (generated stage = pre-existing tolerated non-strict debt). NO release/schema bump (H.5.2
    precedent: unreachable-only removal; accepted language + every reachable shape byte-identical).
    Contract-doc `subroutine_ref` mentions verified to live ONLY in append-only per-release history
    sections; the quarantined `#[ignore]` trace-of-parse-path test left untouched as recorded.`
    Commit: `PGEN-GRAMMAR-WELLFORMED-0060`.
  - `H.10.2` — **`active` (pool, now 3):** originally the remaining 7 — (i) the probe-cohesion
    pair (→ `H.10.2.1`, done); (ii) what was labeled the "terminal-selection trio"
    `letter_no_upper_e`/`unicode_char`/`quoted_class_literal_escaped_char` — RE-ADJUDICATED
    tools-first 2026-06-10: TWO of the three (`letter_no_upper_e`,
    `quoted_class_literal_escaped_char`) were a witness-instrumentation ENGINE defect (memo-hit
    coverage loss → `H.10.2.2`, done), and `unicode_char` is a generation-side
    builtin-primitive gap (→ `H.10.2.3`); (iii) the semantic-prelude pair
    `numeric_backreference`/`backreference_digits` (witnessing needs ≥N capture groups BEFORE
    the backref — the generation-side store honours the predicate (STORE-AWARE-GEN), so the
    reach pass needs a fact-emitting PRELUDE; owned by the B2/C1/C2 constructive lane when
    activated). Pool after `H.10.2.2`: `unicode_char` + the store-gated pair. Pool after
    `H.10.2.3`: EXACTLY the store-gated pair (regex UNKNOWN=2) — **the B2/C1/C2 lane is now
    ACTIVATED as `C2` (design `C2.1` done 2026-06-10, `-0063`; implementation = `C2.2`).**
    - `H.10.2.1` — **`done` (re-applied + closed by `BRANCH-BROADCAST-FIX.5`,
      `PGEN-BRANCH-BROADCAST-FIX-0005`, 2026-06-10): the documented declarative fix landed
      exactly as designed once the engine defect pair was fixed (`.2` broadcast remap + `.3`
      $text span). VERIFIED: regex cert-coverage `UNKNOWN 7→5` (`witness 191→193`, `spf=0`),
      IDENTICAL at seeds 0/7/42; AST-dump A/B byte-identical on `\pL` + `(?^aD-aD)`;
      `restrict:"D"` (the `.3` live-fire proof); fused `aS`/`aW` renders; manifest synced
      167→186; `regex_pcre2_compile_oracle_gate` PASS ⇒ NO release/schema bump. The H.10.2 pool
      is now the terminal-selection trio + the store-gated pair (5).** Original record (the
      discovery): the fix attempt DISCOVERED a live engine defect pair and was cleanly REVERTED
      (baseline re-verified byte-identical: AST dumps identical, cert-coverage `UNKNOWN=7 spf=0`
      at `-0060`). ATTEMPT RECORD: applied the documented parens-group broadcast `-> $text` to
      both rules; the A/B verification caught (a) the broadcast binding branch 0 ONLY (the
      2026-05-14 inner→outer remap collapses whole-body-group inner branches — re-breaking task
      #38, incl. the SHIPPED `return_annotation` `string_literal` single-quoted shape), and (b)
      branch-level `$text` slicing an EMPTY span inside the tournament (`(?^aD-aD)` →
      `restrict:""`, `\pC` → `name:""` — the rollback `parser.position = parse_start` precedes the
      transform). Both owned by the new **`BRANCH-BROADCAST-FIX`** tree (full evidence + design
      there). The fix design itself stands (atomicity via all-branch MatchedText) and is
      re-applied as `BRANCH-BROADCAST-FIX.5` once `.2`+`.3` land. The pre-attempt root-cause
      below remains valid. ROOT CAUSE (tools-first, WHY+WHERE pinned): NOT
      reach-probe-specific — direct ordinary-generation probes emit the same broken renders
      (`--entry-rule modifier_item` → `a S`/`a W`; `--entry-rule escape_unit` → `p c`/`P M`/`p L`,
      while braced `p{…}` renders fine). WHY: `short_prop_letter` (`regex.ebnf:731`) and
      `ascii_restrict_modifier` (`:996`) are non-atomic Or-of-single-word-char rules immediately
      following a word-shaped literal in their parents (`escape_unit:726-727` `"p"/"P"
      short_prop_letter`; `modifier_item:991` `"a" ascii_restrict_modifier`); the
      LEXICAL-ANNOTATIONS.5.2 join rule correctly separates adjacent FREE word tokens, and nothing
      declares these rules as token-continuations. WHERE: `append_generated_segment` separates
      because `rule_is_lexically_atomic` (`stimuli_generator.rs:7978`) returns false — it requires
      `$text` on EVERY branch or a `@transform`. The grammar's stale `:728` comment even claims the
      rule "emits a clean string Terminal" (true pre-self-hosting, lost in the `/…/`-free rewrite).
      FIX (grammar-only, the DOCUMENTED 4th-pillar mechanism — the book's
      `recursion_condition = "R" digits?` atomic-fusion pattern): parens-group broadcast `-> $text`
      on both rules ⇒ every branch MatchedText ⇒ `rule_is_lexically_atomic` true ⇒ the token fuses
      with the preceding `p`/`P`/`a` (`\pC`, `aD`), and internal joins are suppressed. Expected
      consumer shape: byte-identical (each branch is a single char; `$text` of one char == the
      passthrough Terminal) — verified by AST-dump A/B on `\pL` + `(?^aD-aD)`. Acceptance: regex
      cert-coverage `UNKNOWN 7→5` + `spf=0` (seeds 0/7/42, deterministic); AST-dump A/B
      byte-identical; manifest synced to the regenerated inventory; oracle gate + dual-feature lib
      green; cross-grammar untouched; NO release/schema bump expected (shape-preserving).
    - `H.10.2.2` — **`done` (`PGEN-GRAMMAR-WELLFORMED-0061`, ENGINE FIX: memoization ×
      coverage-record composition gap — memo-hit coverage-delta replay).**
      Verification: `done — exactly as designed. ENGINE: MemoEntry gains
      coverage_delta: Option<Vec<u32>> (mod.rs); memoized_call snapshots coverage_stack.len()
      before f(self), stores the body's pushed slice on success (None/no-alloc when coverage
      disabled), and replays it on every hit inside the current speculation (try_parse still
      truncates → transactional soundness preserved). All 10 active generated parsers
      regenerated (ebnf.rs via the documented seed flow — Step A builds without the stale
      artifact — then the 7 grammar parsers + the 2 bootstrap annotation parsers; each carries
      capture+store+replay). VERIFIED: (1) codegen lock-test extended
      (transactional_parse_coverage_wiring_is_emitted_at_codegen asserts snapshot + store +
      replay tokens) — green; (2) NEW live-fire regression test
      parser_registry::tests::regex_parse_and_cover_replays_coverage_on_memo_hits — green
      (\Q\A\E* witnesses quoted_literal_char/quoted_literal_escaped_char/
      quoted_literal_escape_tail/letter_no_upper_e; []\Q\A\E] witnesses
      quoted_class_literal_escaped_char); (3) regex cert-coverage **UNKNOWN 5→3**
      (witness 193→195, spf=0), IDENTICAL seeds 0/7/42 — the pool is now unicode_char +
      the store-gated pair; (4) CROSS-GRAMMAR (UNKNOWN only DECREASES, spf byte-identical 0
      everywhere): **vhdl 31→30, rtl_frontend 75→73, SV 738→647 (−91)** — 91 SV rules sat in
      UNKNOWN purely from this witness-record gap; SV's 68 no-path/multi-entry family
      unchanged; json + rtl_const_expr + svpp (seeds 0/7/42) stay fully_certified; (5)
      regex_pcre2_compile_oracle_gate PASS (parse outcomes byte-identical — the fix touches
      ONLY the opt-in diagnostic coverage record); (6) suites: default 683/0,
      generated_parsers 757/0, generated_parsers+ebnf_dual_run 798/0 (count deltas vs older
      records = the freshly regenerated ebnf.rs's embedded tests, 0 failures everywhere); (7)
      clippy strict-source clean (generated stage = the pre-existing tolerated 191-site
      non-strict debt class). NO release/schema bump (memo-internal + opt-in diagnostic
      surface; no consumer-visible AST/accept-set change). Lockstep: book
      grammar-wellformedness chapter (memo-hit replay added to the witness-record mechanism +
      the 98→19→7→5→3 arc + cross-grammar numbers), KM card memo-hit-transactional-replay
      (the engine-wide rule: EVERY transactional per-rule record must be delta-captured in
      MemoEntry and replayed on hits), RUST_CODEBASE_ANALYSIS architecture note. OPERATIONAL
      NOTE (session): the pre-fix baseline initially mis-read as UNKNOWN=7 because the release
      binary predated the -0002/-0003 engine fixes — cert-coverage numbers depend on the
      BINARY embedding the engine state (the grammar is runtime-loaded but the
      atomicity/broadcast machinery is compiled); rebuilt and re-verified 5/5/5 before any
      work.` Original design record: ROOT CAUSE (tools-first, WHY+WHERE pinned, 2026-06-10):
      the "terminal-selection trio" label was WRONG for 2 of the 3 — `letter_no_upper_e` and
      `quoted_class_literal_escaped_char` are NOT a generation steering miss; they are a
      **witness-instrumentation engine defect**. Live probe evidence
      (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, seed 0): probe `\Q\A\E*` for `letter_no_upper_e` is
      `parsed=true witnessed_target=false`, yet its AST-dump shows atom `["\\","A"]` — i.e. the
      ACCEPTED parse structurally DOES route through `quoted_literal_escaped_char →
      quoted_literal_escape_tail → letter_no_upper_e`. WHY the record misses it:
      `piece_quoted_run_quantified`'s `quoted_run_inner_piece*` speculation enters
      `quoted_literal_char`(pos 2)→…→`letter_no_upper_e` (coverage pushed AND memoized as
      successes), the `!"\E"` lookahead fails the inner piece → `try_parse` truncates the
      coverage stack — then the trailing `quoted_literal_char` slot re-calls at the SAME
      `(rule,position)` key → **memo HIT returns the cached node WITHOUT re-entering the body →
      the coverage push (`ast_based_generator.rs:2701`) never fires** → the accepted parse's
      record permanently lacks the subtree. `memoized_call` (`ast_based_generator.rs:5856`)
      replays the cached **semantic delta** on hit (the `.b.6.2.36.4` fix) but NOT the coverage
      entries — the SAME memoization × transactional-record composition-gap class as
      `.36.3/.36.4`, now on the coverage stack. The class probe `[]\Q\A\E]` is the same
      mechanism via `class_item`'s ordered choice: `class_range` (tried first) →
      `quoted_class_range_atom` → `quoted_class_literal_char`(pos of `\A`) succeeds+memoizes,
      range fails on the missing `-`, rolls back → the `quoted_class_literal` alternative
      memo-hits → `quoted_class_literal_escaped_char` never recorded. Also explains the
      4-identical-retry waste (the retry premise "terminal expansions vary" is moot when the
      record, not the sample, is wrong). NOTE: this violates the book's
      grammar-wellformedness claim that the record is "sound AND complete" — complete fails on
      memo hits; book lockstep required. FIX (parser-agnostic, engine codegen — the `.36.4`
      pattern exactly): extend `MemoEntry` (`ast_pipeline/mod.rs:774`) with
      `coverage_delta: Option<Vec<u32>>`; in `memoized_call` capture
      `coverage_stack.len()` before `f(self)` and store the pushed slice on success (gated
      `coverage_enabled` → `None`/no allocation in ordinary parsing); on memo hit, when
      `coverage_enabled`, `extend_from_slice` the delta onto the live stack (inside the current
      speculation → a later rollback still truncates it → transactional soundness preserved).
      Regenerate ALL generated parsers (each embeds `memoized_call` + the `MemoEntry` shape).
      Acceptance: focused regression test `parse_and_cover_regex("\Q\A\E*")` contains
      `letter_no_upper_e` (+ the class-probe analogue); regex cert-coverage `UNKNOWN 5→3`
      (`witness 193→195`, `spf=0`, seeds 0/7/42 deterministic); cross-grammar `UNKNOWN` may only
      DECREASE vs the locked-program baseline (vhdl 31 / rtl_frontend 75 / SV 738 — measure and
      record the post-fix numbers) with `sample_parse_failures` byte-identical everywhere;
      json + rtl_const_expr + svpp stay fully_certified; `regex_pcre2_compile_oracle_gate` PASS
      (parse outcomes untouched — the fix changes ONLY the opt-in diagnostic coverage record);
      dual-feature lib green; clippy strict-source clean; NO release/schema bump (memo-internal +
      opt-in diagnostic surface only; no consumer-visible AST/accept-set change).
    - `H.10.2.3` — **`done` (`PGEN-GRAMMAR-WELLFORMED-0062`, unicode_char: declarative
      witnessing sample).**
      Verification: `done — exactly as designed (grammar-only, level-1 declarative, the H.5.3
      witnessing-sample pattern): rule-level `@sample: "é"` (U+00E9) above unicode_char in
      grammars/regex.ebnf + the explanatory comment. VERIFIED: (1) direct probe
      `--generate-stimuli --entry-rule unicode_char` now emits `é` (was the hard `Missing rule
      'builtin_any_char'` error — zero generation diversity lost, there was none); (2) regex
      cert-coverage **UNKNOWN 3→2** (witness 195→196, spf=0), IDENTICAL seeds 0/7/42 — the
      H.10.2 pool is now EXACTLY the store-gated pair `numeric_backreference`/
      `backreference_digits` (B2/C1/C2 lane); spf=0 across the seeds IS the parser-side proof
      that every é-bearing witness sample re-parses (the hint-emitted char parses through
      unicode_char by construction — the accept-set is untouched); (3)
      regex_pcre2_compile_oracle_gate PASS (accepted language unchanged); (4) focused regex
      dual-feature tests 104/0; shape-contract 14/14 (manifest↔grammar lockstep holds — @sample
      is a semantic directive, the return-annotation inventory stays 186); (5) full dual-feature
      lib 729/0. NO release/schema bump (generation-only steering; parser artifact regenerated
      for lockstep via focus_regex). The deeper generation capability (builtin-primitive
      materializers + lookahead-guard-aware terminal choice) is TICKETED as a STIMULI-SIGNOFF
      audit data point (Decisions, 2026-06-10) — also explains why comment_text/callout payloads
      only generate empty.` Original design record: ROOT CAUSE
      (tools-first, verified live 2026-06-10): `unicode_char = !builtin_ascii_char
      builtin_any_char -> $2` is **structurally unmaterializable by the generator** —
      `builtin_any_char` is a parser-side codegen-native primitive (`ast_based_generator.rs:894`)
      with NO grammar definition and NO generator special-case, so
      `generate_rule("builtin_any_char")` errors `Missing rule` (decisive direct probe:
      `--generate-stimuli --entry-rule unicode_char` → `Error: Missing rule 'builtin_any_char'
      in grammar 'regex'`); additionally `ASTNode::Lookahead → Ok("")`
      (`stimuli_generator.rs:5529`) means the `!builtin_ascii_char` guard is generation-blind.
      Every alternative chain referencing `unicode_char` backtracks today (zero diversity lost by
      a hint — there is none, only errors). FIX (grammar-only, level-1 declarative, the proven
      `H.5.3` witnessing-sample pattern): rule-level `@sample` with a single non-ASCII literal on
      `unicode_char` so generation can emit it and the reach pass witnesses it. The deeper
      parser-agnostic engine capability (generator-side materialization of `builtin_*` primitives
      + negative-lookahead-guard-aware terminal choice — would also un-empty `comment_text`,
      callout/directive payloads) is TICKETED as a STIMULI-SIGNOFF capability-gap audit data
      point, not blocking UNKNOWN→0. Acceptance: regex cert-coverage `UNKNOWN −1` with `spf=0`
      preserved (seeds 0/7/42); AST shape for the hint-emitted char identical to a parsed
      non-ASCII char (A/B); oracle gate PASS; no release/schema bump expected.
- `H.9` — **`done` (`PGEN-GRAMMAR-WELLFORMED-0059`, svpp PARSER BUG, found by H.7.2's plannable-rule reach pass): a default
  argument on the LAST macro formal mis-parses — `` `define M(a=x) y`` yields formals=[] with
  "(a=x) y" as macro BODY text (legal per IEEE 1800 §22.5.1; the LRM's own examples put defaults on
  the last formal).** Decisive minimal repro pair: `` `define M(a=x) y`` WRONG (formals=[]) vs
  `` `define M(a=x,b) y`` correct (formals=[{a,default x},{b}]). ROOT CAUSE (pinned):
  `macro_default_value := macro_default_atom+` is possessive (PEG), and `macro_default_atom`
  includes bare `lparen`/`rparen` alternatives (meant for nested parens in default text) — the atom
  run swallows the formals' CLOSING ")" so `macro_formals := lparen … rparen` can't close and the
  whole `macro_formals?` optional backtracks to empty. FIX DIRECTION (grammar-only, LRM-faithful):
  replace the bare `lparen`/`rparen` atoms with a BALANCED paren group
  (`macro_default_paren_group := lparen macro_default_atom* rparen`) per LRM 22.5.1's "balanced
  pair" rule — an unbalanced ")" then correctly terminates the default. Consumer-visible accept-set
  correction → svpp release bump + ledger SVPP-0004 + contract/book lockstep; AST shape impact to
  be assessed (the atoms' kinds change for paren-bearing defaults). Acceptance: repro pair correct;
  svpp cert-coverage sweep 0–31 fully_certified 32/32 (closes the H.7.2 residual + H.5.4
  completely); svpp zero-plausible-gap + shape-contract + cross-grammar byte-identical; lib green;
  full lockstep.
  Verification: `done — GRAMMAR-ONLY FIX exactly as designed: macro_default_paren_group :=
  lparen macro_default_group_atom* rparen -> {kind:"paren_group", atoms:$2} with
  macro_default_group_atom := macro_default_atom -> $1 | comma -> {kind:"comma"} replacing the bare
  lparen/rparen alternatives (commas legal inside the group AND ONLY there — both halves of the LRM
  22.5.1 "balanced pair" sentence modeled exactly). VERIFIED: repro pair — the last-formal default
  now parses formals=[{a, default text "x"}] body "y" (was formals=[] + body-text); the non-last
  default unchanged-correct; a paren/comma default yields paren_group{text "x", comma, text "y"}.
  **svpp cert-coverage sweep: fully_certified=true + sample_parse_failures=0 at ALL 32 seeds 0–31**
  (was 25/32 — closes the H.7.2 residual AND H.5.4 COMPLETELY; svpp is now seed-robustly
  UNKNOWN=0). sv_preprocessor_zero_plausible_gap_proof_gate GREEN; svpp AST shape-contract GREEN
  (manifest inventory synced 67→68 + new macro_default_on_last_formal sample); dual-feature lib
  721/0; full workspace 761/0; svpp book gate GREEN (HTML regenerated); clippy strict-source clean.
  LOCKSTEP: svpp release 1.0.6→1.0.7, AST-dump schema 4→5 (breaking for the two buggy shapes only:
  default-on-last-formal empty-formals/body-text → structured formals; paren-bearing defaults flat
  lparen/rparen atoms → one paren_group atom), annotation inventory 67→68 (29→31 rules); contract
  identity + schema-5 row + "Resolved Defects — SVPP-0004" section; ledger SVPP-0004 row; svpp book
  changelog-index (Release 1.0.7 entry) + schema-versioning identity + tracked HTML.`
  Commit: `PGEN-GRAMMAR-WELLFORMED-0059`.
- `H.8` — **`done` (generator-faithfulness FIX): Defect A — literal-hint renders bypass the
  lexical tail-state update → adjacent-item keyword fusion → SV cert-coverage `sample_parse_failures`.**
  The deferred ticket from `SV-PARSE-STRICT` ("The two distinct defects", Defect A; deferred behind the
  `.2` parser fix per the director's "fix parser bugs ASAP"). TOOLS-FIRST RE-MEASURE (2026-06-10, post
  SV-PARSE-STRICT.2, release `ast_pipeline --features "generated_parsers ebnf_dual_run"`, canonical
  `--entry-rule systemverilog_file --grammar-profile sv_2017 --count 40 --seed 0`): SV
  `total=1342 witness=208 UNKNOWN=1134 sample_parse_failures=6` — spf rose 3→6 exactly as anticipated
  (SV-PARSE-STRICT.2's sound store-gating now correctly REJECTS the fused `endmodulemodule` forms the
  old over-permissive parser accepted). ALL 6 failing samples are the SAME fusion class at
  `source_text_item*` boundaries: `endprogram`+`module`, `endmodule`+`module`, `endmodule`+`config`,
  `endmodule`+`extern`, `endprogram`+`program`×2, `endprogram`+`config`. ROOT CAUSE (code-pinned): BOTH
  literal-hint return paths bypass the terminal-render paths that maintain
  `last_terminal_word_shaped` (the LEXICAL-ANNOTATIONS.5.2 tail-shape state consumed by
  `append_generated_segment`'s join rule): (a) `generate_rule`'s whole-rule override (~:5196–5214) sets
  `last_terminal_from_atomic_rule` but NOT the tail shape; (b) `generate_or`'s branch-hint return
  (~:5742–5770, the LIVE path for the `module_declaration`/`program_declaration` branch-local
  `@sample`s) updates NEITHER. So after a hint render the tail state is STALE (usually false) → the
  join rule sees a non-fusable tail → no separator → `endprogram`+`module` fuse. FIX (engine,
  parser-agnostic, generator-only): new helper `literal_hint_tail_word_shaped(text)` — tail terminal
  approximation for an unstructured literal hint: trailing maximal word-char run; false when empty
  (ends non-word); true when the run is the whole hint (one free word token) or is preceded by
  whitespace (a free tail token, the `"… endprogram"` case); false when glued to a structural char
  (the `(?(R` convention of `is_word_shaped_literal` — never split a structural literal from its
  arg). Apply at both hint returns (rule-level: computed on the post-`apply_lexical_follow_restriction`
  rendered text; branch-level: on the hint). `last_terminal_from_atomic_rule` needs NO branch-path
  change (`generate_rule` already sets it on every success exit at ~:5262, after the OR returns).
  Acceptance: SV cert-coverage seed-0 `sample_parse_failures` 6→0 (or every residual honestly
  attributed to a non-fusion class); multi-seed SV sweep improves/never worsens; DECISIVE git-stash
  A/B — NO grammar's `sample_parse_failures` worsens (json/regex/vhdl/svpp/rtl_const_expr/
  rtl_frontend; diverse-pass byte-identity is NOT promised — this is a faithfulness FIX that
  legitimately changes hint-bearing grammars' samples); focused unit test (two hint-bearing rules
  concatenated → separator inserted); lib green; clippy strict-source clean; cross-family + oracle
  gates green.
  Verification: `done — FIX as designed (3 edits in stimuli_generator.rs: literal_hint_tail_word_shaped
  helper + the two hint-return updates; the rule-level one computed on the post-follow-restriction
  rendered text; last_terminal_from_atomic_rule needed no branch-path change since generate_rule sets
  it on every success exit). VERIFIED: (1) SV cert-coverage seed 0: sample_parse_failures 6→0, witness
  208 + UNKNOWN 1134 IDENTICAL pre/post (the fix changes only boundary rendering); seeds 1–7 ALL spf=0.
  (2) DECISIVE same-session git-stash A/B (release binary rebuilt at each state, identical commands):
  pre-fix SV seed-0 spf=6 / post-fix spf=0; the other 6 grammars (json/regex/rtl_const_expr/svpp/
  rtl_frontend/vhdl) headline lines BYTE-IDENTICAL pre/post (diff = empty). (3) svpp seed sweep 0–31:
  spf=0 at every seed (H.5.5's invariant preserved). (4) NEW focused tests:
  literal_hint_renders_keep_word_boundary_separation_between_items (reproduced the fusion DECISIVELY —
  fused without the tail-state update, separated with it; built with faithfulness ON since the
  annotated_generator test helper opts out of spacing) + literal_hint_tail_word_shape_classification
  (free-tail/multi-token/non-word/structural-fragment cases incl. the (?(R convention). (5) lib
  --features "generated_parsers ebnf_dual_run" 718/0 (716 + the 2 new tests). (6)
  stimuli_cross_family_platform_gate PASS (regex + vhdl + SV bounded replay). (7) clippy strict-source
  clean (generated stage = pre-existing tolerated non-strict debt). ALSO RESOLVES the H.4.2
  tracker-note ALERT (the "SV spf=3 vs resume pointer" discrepancy — Defect A was the cause; spf rose
  3→6 when SV-PARSE-STRICT.2's sound gating correctly began rejecting the fused endmodule forms, and is
  now 0). Book lexical-annotations chapter updated ("Literal steering hints participate in boundary
  tracking"). NO grammar/schema/release/contract change (generator-only; parsers untouched).`
  Commit: `PGEN-GRAMMAR-WELLFORMED-0057`.
- `G.4.9` — **DONE (`PGEN-GRAMMAR-WELLFORMED-0035`, 2026-06-07): classified the regex witness-parseability
  residuals (the 6 `sample_parse_failures` surfaced by `H.1`'s regex cert-coverage).** Tools-first
  (`parseability_probe`): the 6 failing witness samples cluster on rare regex constructs — `\u{…}` unicode
  escape, backtracking-control verbs `(*name)` / `(*pla:)` / `(*scs:…)`, branch-reset `(?|…)`, subroutine
  call `(?P>…)`, and conditional `(?(…))`. CLASSIFICATION: **STRUCTURAL generator↔grammar round-trip
  mismatches, NOT lexical.** The over-separation hypothesis (faithful-by-default inserting spaces inside
  the constructs) was DISPROVEN: `a{1,1}` and `a{1 ,1 }` BOTH parse (quantifier spaces harmless), while
  `\u{b7a2}` and `(*xjDD)` are rejected WITH OR WITHOUT the space — the constructs themselves are
  rejected, the spaces are red herrings. So the generator produces these rare forms but the regex parse
  side does not accept them (same class as the SV `use_clause` G.4.8 residual). CONFIRMS the
  LEXICAL-ANNOTATIONS close was correct (structural → owned here, not lexical). ROUTED: each construct is
  a per-construct generator↔grammar reconciliation follow-up (own leaf, like G.4.8 was for use_clause) —
  e.g. `G.4.9.{1..}` (or fold into the `UNKNOWN`→0 / witness-parseability drive). No code change this
  slice (classification). Investigation-only.
  **CORRECTION + ROOT-CAUSE (2026-06-07, `PGEN-EBNF-SOT-0001`):** the construct list above OVER-CLAIMED,
  and the "STRUCTURAL, same class as use_clause" classification was WRONG. Tools-first re-check
  (`parseability_probe --parse regex`) shows `(?|a)` / `(?P>n)` / `(?(1)a)` PARSE fine — only `\u` and
  unrecognized `(*verb)` actually fail (I pattern-guessed from the complex samples instead of pinning each
  failure — see [[feedback_be_alert_root_cause_fishy_immediately]]). The mechanism is NOT PEG-structural:
  `regex.ebnf` STRUCTURALLY accepts `\u{…}` / `(*name)`, but `rust/src/regex_compile_validation.rs` (an
  out-of-band post-parse validator, invisible to the generator + not encoded in the EBNF) rejects them.
  This is a distinct, foundational DEFECT CLASS now OWNED by the new **`EBNF-SOURCE-OF-TRUTH`** tree (the
  EBNF must be the single source of truth for the accepted language; see
  [[project_ebnf_is_single_source_of_truth]]). The actual regex fix lives there (`.3`), not as a generic
  G.4 structural reconciliation.
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
- `A2.1` — **⚠️ RE-ADJUDICATED 2026-07-05 (see `A2.1-SOUNDNESS` below): the "promote EarlierAlwaysMatches
  to the hard gate" goal is RETIRED — the check is UNSOUND for PGEN's backtracking engine (it declares a
  proven-LIVE fragment dead). The 8 residual warnings include FALSE POSITIVES, not dead branches to clean.
  FRONTIER → `A2.2` (decide the check's disposition + implement).** Historical framing (now superseded):
  "clean the 52 SV `always_matches_shadowing` defects LRM-grounded, then promote
  EarlierAlwaysMatches to the hard gate." Each was believed a latent (not active) defect — the dead branch
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
  2. **covergroup-range family — DONE (`-0013`).** The LRM range forms are `[ … ]`-BRACKETED
     (`bins b = {[0:10]}`); extraction dropped the brackets → parens + spurious `?` → always-succeeds.
     FIX: restored `lbrack ( … ) rbrack` + dropped `?` on `covergroup_value_range_sv_2017` (range) +
     `covergroup_value_range_sv_2023` (range/dollar_lo/dollar_hi/tolerance); `body: $1`→`$2` (group is
     now the 2nd element after `lbrack`). Restoring the brackets fixes BOTH the always-succeeds AND the
     would-be prefix-overlap (bracketed forms start with `[`, so the bare-expr arm no longer shadows
     them — no reorder needed). always_matches 34→27. rc=0, regen+compile clean.
  3. **rs-prod family — DONE (`-0013`).** Root cause = a NULLABLE sub-rule: `rs_code_block :=
     ( data_declaration* statement_or_null* )*` (star-over-star) always-succeeds → shadowed the later
     `rs_prod_*` arms (and 11 always-matches sites total — it's a widely-used sub-rule). The LRM form is
     `{ data_declaration* statement_or_null* }` (brace-delimited); extraction dropped the braces + added
     a spurious outer `*`. FIX: `rs_code_block := lbrace ( … ) rbrace` (`body: $1`→`$2`). always_matches
     45→34. rc=0, regen+compile clean.
     **⇒ SYSTEMATIC FINDING: the SV always-matches defects are dominantly DROPPED-DELIMITER extraction
     artifacts** — `[ ]` (consecutive_repetition, covergroup ranges), `{ }` (rs_code_block) — where the
     lost delimiter made a wrapper rule nullable/always-succeeds. The linter (A2) surfaced a whole CLASS
     of LRM-PDF→.ebnf extraction bugs. Restoring the delimiter is the LRM-grounded fix (and often also
     closes a real parse gap — the bracket-less forms couldn't parse real SV).
  4. **formal-type / keyword-after-nullable family — DONE (`-0014`, REORDER).** `let_formal_type`,
     `sequence_formal_type`, `property_formal_type`, `port`, `class_constructor_super_args`,
     `class_constructor_arg_sv_2023`, and the inline `class_declaration_sv_2023` extends-clause: a
     nullable general form (`data_type_or_implicit` / `list_of_arguments` / `( port_expression )?`)
     was FIRST, shadowing a specific keyword/`.`-form (`untyped`/`sequence`/`property`/`default`/named).
     FIX = REORDER specific-before-nullable-general (the PEG-correct realization of the order-independent
     LRM CFG). Clean; annotations travel with each arm.
  4b. **port-header / net-type family — DEFERRED (needs nettype STORE-GATING).** `net_port_type`,
     `net_port_type_sv_2017`/`_sv_2023`, `ansi_port_declaration` (inner `net_port_header |
     interface_port_header`): alt #0 (`( net_type )? data_type_or_implicit`) is LEGITIMATELY nullable
     (implicit type), so it always-succeeds and shadows `net_type_identifier` / `interconnect` /
     `interface_port_header`. `interconnect` could be reordered first (keyword), but `net_type_identifier`
     and `interface_port_header` are BARE IDENTIFIERS that overlap a data-type identifier — reordering
     them first would mis-parse real data types. The SOUND fix is SEMANTIC STORE-GATING (gate
     `net_type_identifier` on `has_fact(nettype, …)`, `interface_port_header` on `has_fact(interface, …)`)
     per [[feedback_grammar_rules_must_consult_store]] — which also needs the grammar to EMIT those fact
     kinds. A careful semantic sub-campaign; 6 of the residual 8 always-matches. Warning-staged meanwhile.
  5. **list-of-arguments family — DONE (`-0014`, REORDER).** `list_of_arguments` (mixed|named|ordered,
     ordered catch-all LAST), `let_/property_/sequence_list_of_arguments` (named_only first), and
     `list_of_checker_port_connections` (named first): the all-optional ORDERED form always-succeeds and
     must be the LAST (catch-all) arm; the non-nullable named/mixed forms go first.
  6. **module-path family — DONE (`-0014`, DELIMITER `{ }`).** `module_path_concatenation` +
     `module_path_multiple_concatenation` were `( … )*` (LRM: `{ … }`) → nullable, cascading up
     concat→primary→operand→expression→mintypmax. Restoring the braces (both rules) resolved
     `module_path_primary` #3#4#5 + `module_path_mintypmax_expression` #1 (4 findings via the cascade).
  7. **`sv_multi_entry_root`** (#1#2) — DEFERRED, NOT a grammar defect: it is a DELIBERATE non-parsed
     reachability MARKER (`systemverilog_file`|`library_text`|`systemverilog_parseable_file`) consumed by
     `sv_formal_exhaustive_closure_gate`'s contract as its `entry_rule` for multi-entry reachability
     analysis (see DEVELOPMENT_NOTES ~8085/8105: a deliberate construct; parser-gen still defaults to
     `systemverilog_file`). ⚠️ It CANNOT simply be deleted (that breaks the closure-gate contract) and it
     must NOT be reordered/edited (it's never parsed). The clean resolution is the documented long-term
     toolchain fix: give the closure-gate contract a `reachability_entry_rules: [...]` ARRAY distinct
     from the parser-gen entry → then `sv_multi_entry_root` is removed and `library_text`/
     `systemverilog_parseable_file` are declared entries (this is `A1b.1`, the entry-declaration leaf).
     Until then the 2 findings stay a benign WARNING (always-matches is warning-staged). A2 promotion to
     a hard gate is blocked on this + the 4b store-gating family.

- `A2.1-SOUNDNESS` — **INVESTIGATION `done` (2026-07-05, session #37, tools-first, ZERO code) — 🚨 THE
  A2 `EarlierAlwaysMatches` CHECK IS UNSOUND FOR PGEN's BACKTRACKING ENGINE: it declares a LIVE fragment
  DEAD, violating the [[feedback_certifying_linter_trustworthiness]] contract ("the linter NEVER declares
  a live fragment dead"). ⇒ the A2.1 goal "clean the residual then PROMOTE `EarlierAlwaysMatches` to the
  hard gate" is INVALID as framed; item 4b's "shadowed/dead → store-gate to reach" premise is REFUTED
  for the branches empirically checked.** Surfaced while verifying the residual 8 tools-first (the FIRST
  time the deferred residual was checked against the real parser rather than reasoned from grammar shape).
  - **DECISIVE EVIDENCE (three independent tool angles, existing shipped parser — NO regen needed).**
    Probe `module m(interconnect p); endmodule` (`--profile sv_2017`, ACCEPTS): (1) `--parse-dump-ast-pretty`
    → the final AST contains `{kind:"interconnect"}` on the route `… ansi > net_or_interface > sv_2017 >
    interconnect`, i.e. through `net_port_type` (the `sv_2017` wrapper is `net_port_type := net_port_type_sv_2017
    -> {kind:"sv_2017"}`); (2) producer-grep — the ONLY `{kind:"interconnect"}` producer in a PORT context is
    `net_port_type_interconnect_sv_only` (`:3490`, = `net_port_type_sv_2017` ALT #2); the other producer
    (`:3444` `interconnect_net_declaration_sv_only`) is a `;`-terminated net *declaration*, impossible in a
    port list; (3) `--trace-rules net_port_type_sv_2017` → the parser ENTERS "branch 1/3", "branch 2/3", AND
    "branch 3/3" (alt #0 logs `matched with zero length` 45×; branch 3/3 = interconnect is entered and its
    output is the winning AST node). A branch that is entered AND whose output appears in the final parse tree
    is, by definition, REACHED — so the linter's "alternative #2 is unreachable" is a FALSE POSITIVE.
  - **ROOT CAUSE.** The A2 verdict message states its own premise verbatim: "alternative #0 always matches
    (never fails) earlier than it (**PEG commits to the earlier alternative**)". PGEN does NOT commit-to-first
    — it BACKTRACKS (enters alternatives in order and keeps the one that leads to overall success; the trace
    proves all 3 entered). Alt #0 `( net_type )? data_type_or_implicit` always-*succeeds* only by matching
    EMPTY (`implicit_data_type := ( signing )? packed_dimension*` is nullable); that empty match then fails
    downstream (`port_identifier` can't be the keyword `interconnect`), the engine backtracks, and alt #2 wins.
    `compute_always_succeeds` is CORRECT (alt #0 does always-succeed); the unsound step is the CONCLUSION
    "⇒ later alternatives unreachable", which holds only under PEG-commit, a model PGEN violates. Corroborated
    by the project's own Phase `C1` ("defeat-earlier-branch crafting … so the parser SELECTS i on replay") —
    witness-crafting only makes sense because later branches ARE selectable.
  - **SCOPE / precision.** "always-succeeds shadowing" is the same family the certifying-linter decision
    EXCLUDES alongside "general FIRST-domination" and "parse-order predicate reachability" — it was mistakenly
    admitted to the sound subset. The sound property under backtracking is "can this alt ever WIN" (a
    language-difference/domination question — UNDECIDABLE in general per [[feedback_certifying_linter_trustworthiness]]),
    NOT the locally-decidable "is an earlier alt always-succeeds". Empirically MIXED within a rule: alt #2
    (interconnect) WINS (live); alt #1 (`checked_nettype_identifier`) is entered-but-never-wins for bare ids
    (alt #0 consumes any identifier as a `data_type` first) — "dominated", but still not "unreachable". The
    check cannot distinguish these soundly.
  - **RE-ADJUDICATION.** (i) A2 `EarlierAlwaysMatches` is an UNSOUND heuristic → it must NOT gate, and per the
    "no verdict without a checkable proof" discipline it should not emit a definite "unreachable" verdict at all
    (even as a warning it makes an unprovable deadness claim). (ii) Item 4b store-gating is NOT needed to "make
    dead branches reachable" (they are reachable) — any store-gating there is a separate AST-shape/fidelity
    question, not an A2 deadness fix. (iii) A2.1's "promote to hard gate" goal is RETIRED. FRONTIER → a DESIGN
    leaf `A2.2` to decide the disposition (retire the check / demote to a non-verdict informational
    anti-pattern hint / restrict to a provably-sound sub-case if one exists — the EMPTY-match earlier-alt form
    is provably unsound and must at minimum be excluded) + implement in `grammar_wellformedness.rs` (drop the
    `EarlierAlwaysMatches::is_hard_gate()`→true path; reframe the emitted text away from "unreachable") + book
    + decision-record lockstep. The confirmatory removal experiment (delete alt #2, regen, show `interconnect
    p` REJECTS) is AVAILABLE but not required — the AST already proves the branch wins. NO grammar/parser
    change and NO regen in THIS investigation (analysis-only). [[project_earlier_always_matches_unsound_backtracking]]

- `A2.2` — **DONE (code, PGEN-GRAMMAR-WELLFORMED-0150) — RETIRE the unsound `EarlierAlwaysMatches`
  deadness VERDICT + its bogus certificate; DEMOTE the always-succeeds observation to a NON-VERDICT
  informational note.** Implements the `A2.1-SOUNDNESS` disposition. DISPOSITION CHOSEN (of the three
  sanctioned options — retire / demote-to-hint / restrict-to-sound-subcase): **demote** — there is NO
  sound sub-case (every "earlier alt always-succeeds" form, empty-match or not, is unsound under a
  non-PEG-commit engine: after the earlier alt succeeds, a downstream failure makes the engine backtrack /
  longest-match into a later alt, so no later alt is provably dead — only the exact-DUPLICATE case is
  truly redundant, and it is already its own sound reason), so the always-succeeds check cannot issue any
  deadness verdict; but the underlying observation ("an earlier alternative always-succeeds, so later
  alternatives are reachable only via backtracking") is a SOUND, decidable, genuinely-useful grammar SMELL
  (it is the tool that surfaced the real dropped-delimiter extraction bugs in A2.1.1–A2.1.6), so it is
  KEPT as an explicit non-verdict `[note]`, worded to make NO unreachability claim.
  - **CHANGE (2 files, parser-agnostic, ZERO grammar/parser regen — a linter-soundness engine fix):**
    (1) `rust/src/ast_pipeline/grammar_wellformedness.rs` — removed `ShadowingReason::EarlierAlwaysMatches`
    (enum variant + its `is_hard_gate`/`message`/`certificate` arms + the detection branch in
    `collect_shadowing`) so `detect_ordered_choice_shadowing` now yields ONLY the two genuinely-sound
    unreachability reasons (exact-duplicate + fixed-terminal-prefix, both already hard-gated at 0);
    removed the unsound `UnreachabilityReason::EarlierArmAlwaysSucceeds` certificate variant + its
    `verify_unreachability_certificate` arms (a certificate is a PROOF of deadness — which always-succeeds
    cannot honestly supply); added a NEW non-verdict `WellformednessIssue::AlwaysSucceedsAlternative`
    finding + detector `detect_always_succeeds_alternatives` (mirrors `detect_nullable_repetition`), whose
    `message()` states only the decidable fact + the backtracking caveat + the extraction-artifact hint,
    with NO "unreachable"/"dead"/"PEG commits" wording. (2) `rust/src/main.rs` — `--lint-grammar` now
    reports `always_matches_shadowing` GONE from the shadowing channel (the header field is replaced by
    `always_succeeds_alternatives={N} (note)`), the always-succeeds notes print as `[note]` (never gate),
    and the shadowing warning-split (`shadow_hard`/`shadow_warn` partition) is removed since every
    surviving `ShadowingReason` is hard-gated.
  - **WHY NOT touch `FixedTerminalPrefix`?** Out of scope + not tool-proven false here: it is a DISTINCT
    reason, flags 0 branches on every shipped grammar (inert — emits no false positive today), and this
    finding's decisive evidence is specific to `EarlierAlwaysMatches`. (Theoretical note for a future
    `A2.3`: `a | ab` fixed-prefix is likewise reachable under backtracking, so `FixedTerminalPrefix`
    soundness deserves its own tool-proved re-audit — logged, NOT acted on here, to avoid unproven scope
    creep per [[feedback_no_codebase_change_without_tool_backed_facts]] + [[feedback_pinpoint_real_blocker_not_menu]].)

## Acceptance Checklist (enforced) — `A2.2`
- [x] **REPRODUCE / ISSUE** — `ast_pipeline grammars/systemverilog.ebnf --lint-grammar` →
  `grammar lint: 'systemverilog' … always_matches_shadowing=8 (warning, A2 backlog) …` with 8 `[warn]`
  lines each asserting `alternative #N is unreachable — alternative #0 always matches … (PEG commits to
  the earlier alternative)`, INCLUDING `net_port_type_sv_2017 alternative #2` — the `interconnect` branch
  A2.1-SOUNDNESS PROVED live (final-AST `{kind:"interconnect"}` + `--trace-rules` branch-3/3-wins). The
  certifying linter is emitting an unsound deadness VERDICT on a proven-LIVE branch.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammar_wellformedness.rs:1267-1270` (`collect_shadowing`) concludes
  "earlier alt always-succeeds ⇒ later alt `ShadowingReason::EarlierAlwaysMatches` (unreachable)"; the
  message (`:1176`) states the false premise "PEG commits to the earlier alternative". PGEN does NOT
  commit-to-first (backtracking / `longest_match` branch policy) — `compute_always_succeeds` is correct,
  the "⇒ unreachable" CONCLUSION is unsound. Tool-proven in `A2.1-SOUNDNESS`
  ([[project_earlier_always_matches_unsound_backtracking]]); violates
  [[feedback_certifying_linter_trustworthiness]] ("the linter NEVER declares a live fragment dead").
- [x] **FIX** — fix-hierarchy tier = ENGINE (a linter/analysis soundness fix; not expressible in the
  grammar or an annotation). Minimal: drop the unsound verdict + its certificate; demote to a non-verdict
  note. Two files, no new grammar constructs, no parser regen.
- [x] **ADDRESSED (verified)** — after: `ast_pipeline grammars/systemverilog.ebnf --lint-grammar` →
  header field `always_matches_shadowing=8` REPLACED by `always_succeeds_alternatives=8 (note)`;
  `ordered_choice_shadowing=0` unchanged; rc=0 (hard gate intact); the 8 findings now print as `[note]`
  with wording that makes NO unreachability claim (grep confirms ZERO `is unreachable`/`PEG commits`
  strings in the note channel). The proven-live `net_port_type_sv_2017 #2` is no longer branded dead.
- [x] **NO REGRESSION** — `cargo test -p pgen grammar_wellformedness` GREEN (shadowing/certificate/note
  suites updated + passing); canonical certificate-coverage byte-identical `total=1343 proof=10
  witness=1321 UNKNOWN=12` deterministic at seeds 0/7/42 (shadowing certs are branch-level, NOT in the
  rule-level `proof` pool — `grammar_wellformedness.rs:1815` — so the headline is provably unmoved and
  empirically confirmed); `--lint-grammar` rc=0 on all shipped grammars (hard gate = exact-dup +
  fixed-prefix, both still 0); the 6 fully-certified grammars' generated parsers UNCHANGED (no grammar
  edit, no regen); clippy strict clean.
- [x] **LOCKSTEP** — book `docs/book/src/grammar-wellformedness.md` reconciled (the stale
  "staged for promotion / sound decidable form" passages updated to the retire+demote disposition, joining
  the existing 2026-07-05 Correction note); decision `project_earlier_always_matches_unsound_backtracking`
  updated (disposition IMPLEMENTED); CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS +
  this tree frontier updated.

- `A2.3` — **DONE (code, `PGEN-GRAMMAR-WELLFORMED-0151`, session #51) — `FixedTerminalPrefix` soundness re-audit (the A2.2 theoretical
  note, tool-proven via PARSE-HARNESS.8) → the verdict + its certificate are now BRANCH-POLICY-AWARE.** The `A2.2` leaf logged (but did not act on) the theoretical
  concern that `ShadowingReason::FixedTerminalPrefix` — the OTHER hard-gated shadowing verdict, whose
  message also asserts "PEG commits to the earlier alternative" — is likewise unsound under PGEN's
  backtracking/tournament engine. The PARSE-HARNESS `.6.1` combinator suite + the `.8` live probes now
  supply the decisive evidence: on `start := "a" | "a" "b"` with input `"ab"`, the DEFAULT
  `longest_match` policy and `priority_first` both SELECT THE LATER alternative (accept), while only
  `@branch_policy: ordered` commits to the earlier one (reject) — so "alternative #j is unreachable"
  is FALSE for the two policies every shipped grammar actually uses (28+16+… `priority_first` uses,
  zero `ordered` uses, everything else default `longest_match`). The check is inert today (flags 0
  branches on every shipped grammar) but it is a hard GATE: the moment an author writes the legitimate
  longest-match idiom `a | ab`, `--lint-grammar` would hard-fail with a false deadness claim — a direct
  [[feedback_certifying_linter_trustworthiness]] violation ("the linter NEVER declares a live fragment
  dead"). DISPOSITION (to be confirmed by the probes + implemented here): make the verdict
  **branch-policy-aware** — `FixedTerminalPrefix` remains a hard-gated unreachability verdict ONLY for
  rules whose effective `@branch_policy` is `ordered` (first-success commit, where the classical PEG
  argument holds — and only when no branch-phase `@predicate` can block the earlier alternative);
  under `longest_match`/`priority_first` the structure is a live, legitimate idiom → NO finding (not
  even a note — unlike always-succeeds it is not a smell, it is the normal longest-match pattern).
  The `FixedTerminalPrefixBy` unreachability CERTIFICATE must carry the same policy condition (a
  certificate is a PROOF; an unconditional one would be re-verifiable yet false). Acceptance checklist
  below (enforced).
  **EVIDENCE RECEIVED (PARSE-HARNESS.8, session #51 — live-CONFIRMED, upgrading this from
  defect-in-waiting to live false-verdict):** on the scratch slot with `scratch := "a" | "a" "b"`
  (default policy) and input `"ab"`, the engine ACCEPTS with the LATER alternative's typed AST and
  its own trace says `🏁 Rule 'scratch' selected branch 2/2 … branch_policy=longest_match`, while
  `--lint-grammar` on the SAME grammar reports `alternative #1 is unreachable … (PEG commits to the
  earlier alternative)` and exits **rc=1** — the certifying linter hard-fails a live grammar on a
  false deadness verdict. Fresh `parse_harness_combinator_gate` (2/2, 50.91 s): `choice_ordered`
  REJECTS `"ab"` (the sole sound sub-case), `choice_longest_default`/`choice_longest_explicit`/
  `choice_priority_first` ACCEPT it — interpreter + compile-and-run oracle byte-identical throughout.
  **IMPLEMENTED (4 files, linter tier — ZERO parse-behavior change, ZERO grammar/parser regen):**
  (1) `semantic_directive_registry.rs` — NEW shared `semantic_directive_name_payload` (the exact
  directive-resolution codegen used privately) + `effective_rule_branch_policy` (last
  `@branch_policy` wins; default `LongestMatch`) — the SINGLE SOURCE OF TRUTH both codegen and the
  linter now read, so the two can never drift. (2) `ast_based_generator.rs` —
  `semantic_directive_parts` / `rule_branch_policy` DELEGATE to the shared functions
  (emit-identical: `focus_json` + `focus_rtl_frontend` regen byte-identical, `cmp` clean).
  (3) `grammar_wellformedness.rs` — `detect_ordered_choice_shadowing` gains the `annotations`
  param; the `FixedTerminalPrefix` arm fires ONLY when the owning rule's effective policy is
  `Ordered` AND `rule_has_branch_phase_predicates` is false (a branch-phase predicate can block the
  earlier alternative after it matches — `should_take=false` — reviving the later one; conservative
  rule-wide suppression across all 3 annotation surfaces); per-reason messages (the duplicate
  message no longer asserts "PEG commits"; the fixed-prefix message states the TRUE ordered-policy
  premise); `verify_unreachability_certificate` gains `annotations` and RE-DERIVES the policy
  condition for `FixedTerminalPrefixBy` (a certificate is a PROOF — a policy-false certificate is
  rejected, not re-verified on structure alone); `verify_wellformedness_certificate` threads its
  existing `annotations` down. (4) `main.rs` — the lint caller passes `grammar.annotations`.
  Tests: the old unconditional `detects_fixed_terminal_prefix_shadowing` REPLACED by the policy
  matrix (`fixed_terminal_prefix_is_not_a_verdict_under_default_longest_match` — default + explicit
  `longest_match` + `priority_first` → NO finding; `detects_fixed_terminal_prefix_shadowing_under_ordered_policy`
  → finding + ordered-premise message; `fixed_terminal_prefix_is_suppressed_by_branch_phase_predicates`
  → rule-level + per-branch suppression, pre-phase predicate does NOT suppress); the certificate
  test extended with the stale/policy-false + branch-predicate rejection cases; ordered-policy
  false-positive guards added to `no_false_positive_distinct_or_longer_first`. Acceptance checklist
  below (enforced).
- `A2.4` — **`done` (`PGEN-GRAMMAR-WELLFORMED-0152`, session #52, CODE / linter-soundness engine
  fix; ZERO grammar/parser regen — regen byte-identity proven on all 4 policy-carrying grammars):**
  the LAST unconditioned ordered-choice deadness verdict (`DuplicateAlternative`) is now
  SELECTION-SEMANTICS-CONDITIONAL, completing the A2.2/A2.3/A2.4 arc. FIX: new
  `RuleSelectionSemantics` resolver (`grammar_wellformedness.rs`) reads the rule's effective
  `@associativity`/`@priority`/`@deterministic_group`/branch-predicate surface through NEW shared
  registry helpers (`effective_rule_associativity` / `effective_rule_branch_priorities` /
  `effective_rule_deterministic_partition_policy` + moved-shared `SemanticDeterminismPartitionPolicy`)
  that codegen's `rule_associativity`/`rule_branch_priorities`/`rule_deterministic_partition_policy`
  now DELEGATE to (A2.3 single-source-of-truth pattern — the two can never drift). The duplicate
  verdict fires only where the earlier twin provably wins (`ordered`; or tournament with earlier
  priority ≥ later under `left` / strictly-greater under `right`/`nonassoc`, no branch-phase
  predicate, no partition rotation); the `nonassoc` equal-priority tie gets its OWN reason
  `DuplicateAlternativeNonassocTie` (unreachable, but "restructure deliberately" not "merge/remove",
  since removal changes acceptance). `DuplicateOf` certificate re-check + A2.3's `FixedTerminalPrefixBy`
  now both route through `RuleSelectionSemantics` (A2.4-D also requires no partition rotation for the
  fixed-prefix verdict — rotation reorders `ordered` first-success). ADDRESSED (before→after,
  scratch slot): probe R `@associativity: right` rc=1→**rc=0**; probe P `@priority: [0,5]`
  rc=1→**rc=0**; probe D `@deterministic_group` rc=1→**rc=0**; probe N `@associativity: nonassoc`
  keeps rc=1 with the NEW truthful tie message; defaults + `ordered` keep rc=1 (sound sub-cases).
  NO-REGRESSION: lib 695/0 (+3 A2.4 tests, 48/48 wellformedness); shipped-grammar shadowing sweep
  unchanged (12 shipped at 0, `profiled_generated` at 23 — grep-proven ZERO conditioning
  annotations, so identical default path); byte-identical regen of ALL 4 policy carriers (json
  defaults / rtl_frontend priority_first×16 / systemverilog @priority×4 / regex @precedence×4 —
  `cmp` clean, proving codegen delegation emit-identical); `parse_harness_combinator_gate` 2/2;
  `sv_cert_recognized_union_gate` GREEN (canonical UNKNOWN=12, union 1, seeds 0/7/42); clippy
  source-strict; mdbook gate. LOCKSTEP: top book grammar-wellformedness chapter + TOOLBOX §5.1 &
  Protocol D + decision record `project_duplicate_alternative_selection_semantics_conditional`. See
  the acceptance checklist below (enforced). PROBE EVIDENCE (Protocol D, scratch slot, input `a`,
  exact twins `"a" | "a"`): **(R)**
  `@associativity: right` → engine `🏁 Rule 'scratch' selected branch 2/2 consuming 1 chars
  (priority=0, associativity=right, branch_policy=longest_match)` while lint rc=1 "alternative #1
  is unreachable" — FALSE VERDICT; **(N)** `@associativity: nonassoc` → twins REJECT `a`
  (`Backtrack at position 0`) while dedup control `probe_n_single := "a"` ACCEPTS — the verdict's
  "merge or remove the duplicate" advice CHANGES ACCEPTANCE (nonassoc tie fails both twins; the
  pair is load-bearing); **(P)** `@priority: [0, 5]` default assoc/policy → `🏁 selected branch
  2/2 (priority=5, associativity=left)` — FALSE VERDICT; **(D)** `@deterministic_group: "spin"`
  (FNV offset 1) all-default assoc/policy → `🏁 selected branch 2/2 (priority=0,
  associativity=left, branch_policy=longest_match)` — evaluation-order ROTATION makes ties keep
  the rotated-first incumbent under left assoc → FALSE VERDICT; rotation ALSO reorders `ordered`
  first-success (codegen: shared `for branch_index in evaluation_order` + ordered arm
  `best_content.is_none()`), so A2.3's FixedTerminalPrefix condition must additionally require
  partition-disabled. Lint halves: all four probe grammars hard-fail rc=1 with the unconditional
  duplicate verdict today. NOTE original audit plan below (kept for the record):
  `DuplicateAlternative` tie-break soundness under non-default `@associativity` AND per-branch
  `@priority`. The engine breaks equal-length ties by priority THEN associativity
  (`ast_based_generator.rs:3440-3484`): `right` makes the LATER branch win ties — so for an
  exact-duplicate pair under `@associativity: right` the engine SELECTS the later duplicate
  (branch-index-keyed effects: per-branch return annotations, branch-start inline actions,
  `semantic_selected_branch_index`), inverting the "later duplicate is unreachable" claim; under
  `nonassoc` a duplicate tie sets `nonassoc_tie` (NEITHER taken — the choice FAILS), so removing
  the "dead" duplicate would CHANGE behavior (un-fail the tie — the `DuplicateOf` certificate's
  removal-preserves-language claim is FALSE); a LATER-higher `@priority: [lo, hi]` on the twins
  makes the later duplicate win under BOTH `longest_match` (priority is the pre-associativity
  tie-break) and `priority_first` (priority is primary). The linter arm
  (`grammar_wellformedness.rs:1409`) and the `DuplicateOf` certificate re-check (`:1680`, bare
  `ast_eq`) are both selection-semantics-BLIND — the exact A2.3 unsoundness class. Exposure:
  `@associativity` has ZERO shipped-grammar uses; `@priority` has 7 (SV family) but the 13-grammar
  lint sweep is at 0 shadowing findings → defect-in-waiting. AUDIT PLAN (evidence FIRST per
  [[feedback_no_codebase_change_without_tool_backed_facts]], Protocol D is the tool): scratch-slot
  probes (R) `@associativity: right` + `scratch := "a" | "a"` → expect engine `🏁 selected branch
  2/2` while lint hard-fails "alternative #1 unreachable"; (N) `@associativity: nonassoc` twins →
  expect engine REJECT on the tie (removal would flip to ACCEPT — the removal advice changes
  behavior); (P) `@priority: [0, 5]` twins under default policy/assoc → expect `🏁 selected branch
  2/2`. Disposition (if confirmed): condition the verdict + `DuplicateOf` certificate on the
  selection semantics via shared registry helpers (`effective_rule_associativity`,
  `effective_rule_branch_priorities` — codegen delegates, A2.3 single-source-of-truth pattern);
  sound sub-cases keep the hard gate (policy `ordered`; or tournament with `left` assoc + equal
  effective twin priorities + no branch-phase predicates — the predicate suppression mirrors
  A2.3's conservative rule-wide condition).

## Acceptance Checklist (enforced) — `A2.4`
- [x] **REPRODUCE / ISSUE** — Protocol D scratch-slot probes, exact twins `scratch := "a" | "a"`,
  input `a`: (R) `@associativity: right`, (P) `@priority: [0, 5]`, (D) `@deterministic_group:
  "spin"`, (N) `@associativity: nonassoc` — each `ast_pipeline <g>.ebnf --lint-grammar` hard-failed
  **rc=1** `ordered_choice_shadowing=1` "alternative #1 is unreachable — exact structural duplicate
  … merge or remove", while the engine on the SAME grammar (release `parseability_probe --parse
  scratch`) SELECTS the "dead" later twin (R/P/D) or REJECTS both twins (N, while the dedup control
  ACCEPTS) — the certifying linter hard-fails a live grammar / gives acceptance-changing advice.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammar_wellformedness.rs` `collect_shadowing` (pre-fix
  `:1409`) concluded "exact duplicate ⇒ later twin unreachable" SELECTION-SEMANTICS-BLIND; the
  `DuplicateOf` certificate re-check (`:1680`) was a bare `ast_eq`. PGEN's `|` is a branch
  TOURNAMENT (`generate_or_logic`, `ast_based_generator.rs:3440-3484`): equal-length twins tie, and
  the winner is `@priority` (compared first) then `@associativity` (`left`→earlier, `right`→later,
  `nonassoc`→`nonassoc_tie` fails the choice), all under a `@deterministic_group` `evaluation_order`
  ROTATION. The engine's own traces name the winner: `🏁 Rule 'scratch' selected branch 2/2` for
  R (`associativity=right`), P (`priority=5`), D; N's `Backtrack at position 0` vs the control's
  accept proves the tie fails the whole choice. Evaluation-order rotation ALSO reorders `ordered`
  first-success → tightens A2.3.
- [x] **FIX** — fix-hierarchy tier = ENGINE (linter/analysis soundness; not expressible in grammar
  or annotation). NEW `RuleSelectionSemantics::duplicate_verdict` conditions the verdict on the
  effective `@associativity`/`@priority`/`@deterministic_group`/branch-predicate surface, derived by
  NEW shared `effective_rule_associativity` / `effective_rule_branch_priorities` /
  `effective_rule_deterministic_partition_policy` (+ moved-shared `SemanticDeterminismPartitionPolicy`)
  — the same functions codegen's tournament now DELEGATES to (single source of truth). New
  `DuplicateAlternativeNonassocTie` reason for the tie sub-case. `DuplicateOf` +
  `FixedTerminalPrefixBy` certificate checks route through the shared resolver. 3 files; no new
  grammar constructs; no parser regen.
- [x] **ADDRESSED (verified)** — before→after on the probe grammars (`ast_pipeline <g> --lint-grammar`):
  R **rc=1 → rc=0** (`ordered_choice_shadowing` 1→0), P **rc=1 → rc=0**, D **rc=1 → rc=0** (all
  three false verdicts gone); N **still rc=1 at 1** with the NEW truthful tie message ("always tie …
  NEITHER can ever be selected … removing … would CHANGE acceptance … restructure deliberately");
  defaults `"a" | "a"` and `@branch_policy: ordered` variant **still rc=1 at 1** (sound sub-cases
  keep the hard gate). Duplicate certificates now REJECTED by `verify_unreachability_certificate`
  under right/priority/partition/predicate, ACCEPTED under nonassoc-tie/ordered/defaults (pinned by
  the new `duplicate_certificates_are_selection_semantics_conditional` test).
- [x] **NO REGRESSION** — lib **695/695** (+3 A2.4 tests; wellformedness module 48/48);
  shipped-grammar shadowing sweep UNCHANGED (12 shipped grammars at `ordered_choice_shadowing=0`;
  `systemverilog_lrm_profiled_generated` at 23 — grep-proven ZERO
  `@associativity`/`@priority`/`@precedence`/`@deterministic_group`/`@seed_group` annotations, so
  the identical default path). Byte-identical regen of ALL 4 policy-carrying grammars proves the
  codegen delegation is emit-identical: `json` (defaults), `rtl_frontend` (`priority_first`×16),
  `systemverilog` (`@priority`×4), `regex` (`@precedence`×4) — `cmp` clean each. `parse_harness_combinator_gate`
  **2/2** (policy/tie-break matrix); `sv_cert_recognized_union_gate` **GREEN** (canonical UNKNOWN=12,
  union UNKNOWN=1, residual `context_member_method_call`, deterministic seeds 0/7/42); clippy
  source-strict clean (generated-stage `eq_op` debt pre-existing + byte-identical); `mdbook_docs_gate`
  green.
- [x] **LOCKSTEP** — top book `grammar-wellformedness.md` (A2.4 paragraph + updated "gating shadow
  forms" conclusion), `TOOLBOX.md` §5.1 + Protocol D precedent list, decision record
  `docs/decisions/project_duplicate_alternative_selection_semantics_conditional.md`; auto-memory
  `project_earlier_always_matches_unsound_backtracking` updated (A2.4 closed).

## Acceptance Checklist (enforced) — `A2.3`
- [x] **REPRODUCE / ISSUE** — PARSE-HARNESS.8 live probe: `scratch := "a" | "a" "b"` (default
  policy), `ast_pipeline … --lint-grammar` → `ordered_choice_shadowing=1 (error)`, `[error] …
  alternative #1 is unreachable — alternative #0 is a fixed-terminal prefix of it (PEG commits to
  the earlier alternative)`, **rc=1** — while the engine on the SAME grammar ACCEPTS `"ab"` with
  the later alternative's typed AST. The certifying linter hard-fails a live grammar.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammar_wellformedness.rs` `collect_shadowing` (pre-fix
  `:1331-1342`) concluded "fixed-terminal prefix ⇒ later alt unreachable" POLICY-BLIND; the message
  (`:1244`) asserted the false premise "PEG commits to the earlier alternative". PGEN's `|` is a
  branch TOURNAMENT (`generate_or_logic`, `ast_based_generator.rs:3437-3485`): under the DEFAULT
  `longest_match` the engine tries every alternative and keeps the longest — codegen's own trace
  `🏁 Rule 'scratch' selected branch 2/2 consuming 2 chars (priority=0, associativity=left,
  branch_policy=longest_match)` names the branded-dead branch as the WINNER. The premise holds only
  for `@branch_policy: ordered` (`rule_branch_policy` @`:7178`; zero shipped uses). Fresh
  `parse_harness_combinator_gate` 2/2: `choice_ordered` REJECTS `"ab"`, `choice_longest_default`/
  `choice_longest_explicit`/`choice_priority_first` ACCEPT — both implementations byte-identical.
- [x] **FIX** — fix-hierarchy tier = ENGINE (linter/analysis soundness; not expressible in grammar
  or annotation). Verdict + certificate conditioned on the rule's effective `@branch_policy`
  (`ordered`-only, no branch-phase predicates), derived by the NEW shared
  `effective_rule_branch_policy` — the same function codegen's tournament now delegates to (single
  source of truth). 4 files; no new grammar constructs; no parser regen.
- [x] **ADDRESSED (verified)** — before→after on the probe grammars (`ast_pipeline <g> --lint-grammar`):
  default `longest_match` `a|ab` **rc=1 → rc=0** (`ordered_choice_shadowing` 1→0 — the false
  verdict is gone); explicit `priority_first` variant rc=0 at 0; `@branch_policy: ordered` variant
  **still rc=1 at 1** with the corrected ordered-premise message (the sound sub-case keeps its hard
  gate); `ordered` + branch-phase-predicate variant → `ordered_choice_shadowing=0` (suppressed;
  its rc=1 is the separate, correct `unbound_fact_kinds=1` from the probe's emitterless
  `has_fact`). Policy-false certificates now REJECTED by `verify_unreachability_certificate`
  (pinned by the extended tampering test).
- [x] **NO REGRESSION** — shipped-grammar lint sweep **13/13 rc=0** with `ordered_choice_shadowing=0`
  everywhere (unchanged — the check was inert on the shipped fleet); `make sv_cert_recognized_union_gate`
  **GREEN post-change** (canonical UNKNOWN=12, union UNKNOWN=1, residual `context_member_method_call`,
  deterministic seeds 0/7/42); decisive git-stash A/B: the bare canonical CLI run is **byte-identical
  with and without the change** (`total=1343 proof=21 witness=1321 UNKNOWN=1` both sides, 3 seeds —
  the proof=21 shape vs the tree's older proof=10 note is the VERILOG-2005-PROFILE.6.7 per-profile
  promotion, pre-dating this leaf; the gate's pinned accounting is the oracle and it is green);
  codegen delegation emit-identical (`focus_json` + `focus_rtl_frontend` regen → `cmp` byte-identical
  → all generated parsers unchanged by construction); full dual-feature lib suite **807/0**
  (`grammar_wellformedness` 45/45, `ast_based_generator` 68/68, `semantic_directive_registry` 23/23);
  `verilog_2005_conformance_gate` GREEN (the one `--lint-grammar` consumer); clippy strict-source
  clean (generated stage = pre-existing non-strict debt); `mdbook_docs_gate` + `ebnf_parser_book_gate`
  PASS.
- [x] **LOCKSTEP** — top book `grammar-wellformedness.md` (contract item 3 + *Where PGEN stands* +
  a new A2.3 passage extending the FIRST-domination argument) + `parse-harness.md` (the `.6.1`
  discrimination note now records the consumed verdict); **ebnf parser book** — fixed REAL drift:
  `rules-and-expressions.md` + `codegen-model.md` claimed `|` "commits to the first match" (the
  exact misconception `.8` disproved) → rewritten to the three-policy tournament semantics, stale
  "shadowing WARNINGS" → hard-error reality (`build-recipe.md` too), rendered HTML regenerated;
  `TOOLBOX.md` §5.1 WHAT-line policy-conditioned; decision record
  `docs/decisions/project_fixed_terminal_prefix_policy_conditional.md` + INDEX row; CHANGES /
  DEVELOPMENT_NOTES / MEMORY / LIVE tracker + this tree + `docs/TASK_TREE.md` updated.

### `A2.5` — the linter calls a DEAD branch "handled by PGEN", and it is not (✅ **BOTH HALVES `done`** — ENGINE half 2026-08-11 `-0153`; the LINTER half landed as its own leaf **`A2.6`** 2026-08-12 `-0154`)

> ⭐⭐ **LANDED.** The engine now eliminates left recursion in BOTH shapes, so the linter's message is
> no longer false. What remains open is that it is still **unearned** — it would say the same thing if
> the eliminator regressed. That half is scoped at the end of this leaf and is deliberately NOT closed
> here. Measured outcome: SystemVerilog's **4 dead alternatives → 0**, with **ZERO grammar bytes**
> changed; `select_expression`'s `&&` / `||` / `with ( … )` and `block_event_expression`'s `or` all
> parse, and all four return the AST their annotations declare.

⭐⭐ **DIRECTOR RULING, 2026-08-11 — the fix tier was ESCALATED from grammar to ENGINE, and the
first attempt was rejected on review.** This leaf originally proposed leaving the eliminator alone
and repairing the affected SystemVerilog rules in the grammar, by hand-writing the
`seed ( continuation )*` rewrite. That was implemented, verified, and then **reverted unshipped**,
because it is the wrong answer to the right question:

> *"PGEN engine shall handle all things that are objectively shared, common to all EBNFs. The EBNF
> shall carry only things that are really specific to each language they describe … hardcoding LR
> elimination in EBNF is as a consequence a bad, non-sota, non-signoff [decision]."*
> *"EBNF authors should NOT have to worry about LR elimination, they shall simply not care."*

The three reasons the hand-written rewrite was worse, recorded so the argument is not re-litigated:

1. It breaks **EBNF-as-sole-source-of-truth** — the grammar stops transcribing Annex A and starts
   transcribing a hand-compilation of it.
2. It produces a **less faithful AST**: a flat chain, where the eliminator's `_pgen_lr_chain` fold
   rebuilds the standard's **left-nested binary** `lhs`/`rhs` from the author's own annotations.
3. It costs a **schema break that the real fix would have to break again** — the one thing you do
   not do twice to a downstream consumer.

⛔ And the blast-radius argument used to justify the grammar-tier fix was **asserted, not measured**.
Measured afterwards across all 13 buildable grammars: **no other family has a single instance**
(regex, vhdl, ebnf, json, rtl_frontend, rtl_const_expr, svpp and the three annotation grammars are
all at 0), while the raw IEEE 1800-2017 Annex A transcription
(`systemverilog_lrm_profiled_wrapper`) carries **9 dead alternatives across 3 rules** —
`select_expression` 3 of 8, `sequence_expr` **5 of 12**, `block_event_expression` 1 of 3. The
shipped grammar shows only 1 because earlier sessions had already hand-flattened the others. That
difference is the scar tissue, and it is now its own tree: `ENGINE-UNIVERSAL-SERVICES`.

#### The fix, as landed

✅ **LANDED 2026-08-11.** The preserved patch at
`docs/tasks/artifacts/engine_universal_services/A2.5_direct_lr_normalization.patch` is now HISTORY,
not a resume pointer: its engine half applied clean, its test half was re-authored by hand (see
*What changed relative to the patch* below), and the landed change is strictly larger than it.

`normalize_direct_left_recursive_alternatives` in `rust/src/ast_pipeline/mod.rs`, a **pre-pass, not
a second elimination path**: a directly left-recursive alternative is mechanically the wrapper shape
with the wrapper inlined, so the pass hoists each such alternative's body verbatim into a synthetic
`<rule>_lr_altN` rule, moves that branch's annotations onto it (`hoist_branch_annotations`), and
leaves a bare rule reference behind — after which the existing, tested
`detect_left_recursive_chain_plan` / `apply_left_recursive_chain_plan` / `_pgen_lr_chain` machinery
does all the work unchanged. Hoisting the body *verbatim* is what preserves the author's `$N`
indices, the same invariant the wrapper flatten path already relies on.

⛔ Deliberately NOT normalized: a rule whose alternatives are **all** left-recursive. It derives
nothing, and hoisting would only hide the non-termination behind a helper rule; it stays visible to
the linter's `non_terminating` error.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — a 4-rule synthetic grammar
  (`expr := expr plus term | expr minus term | term`) reproduces it with no language involved:
  `--lint-grammar` says *"left-recursive … handled by PGEN's LR elimination + runtime
  cycle-breaking (informational, not an error)"* and exits 0, while the gen-AST sweep reports
  `{"rules": 4, "dead": [["expr", 1, 3], ["expr", 2, 3]]}` — both operator alternatives dead.
- [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_TRACE_VERBOSITY=debug … --trace-rules bins_selection` on
  the SV instance names the mechanism and the exact branch indices:
  `🚪 Entering branch 3/8 for rule 'select_expression' at position 127` →
  `💥 Infinite recursion detected in rule 'select_expression' at position 127`, repeated verbatim
  for branches **4** and **6**, then
  `🏁 Rule 'select_expression' selected branch 7/8 consuming 2 chars (branch_policy=longest_match)`.
  WHERE: `detect_left_recursive_chain_plan` matches an alternative only via
  `extract_rule_reference_name` → `extract_wrapper_suffix`, i.e. a **bare rule reference** to a
  wrapper rule; a multi-element `Sequence` starting with a self-reference matches neither, so the
  alternative reaches codegen intact.
- [x] **FIX** — ENGINE tier, and the tier is the point (director ruling above). Three functions in
  `rust/src/ast_pipeline/mod.rs` and one visibility change; **no grammar bytes**, no new EBNF
  construct, nothing for an author to learn or opt into:
  - `normalize_direct_left_recursive_alternatives` — the pre-pass. Hoists each directly
    left-recursive alternative's body **verbatim** into a synthetic `<rule>_lr_altN` rule and leaves
    a bare reference behind, after which the existing, tested planner does all the work unchanged.
    Hoisting *verbatim* is what preserves the author's `$N` indices.
  - `hoist_branch_annotations` — moves that branch's return / semantic / mid-sequence annotations
    onto the synthetic rule, where their `$N` still resolve.
  - `retract_consumed_normalization_rules` — ⭐ **NOT in the prototype; added because the closure
    gate caught what the prototype would have shipped.** See *What changed relative to the patch*.
  - `grammar_wellformedness::collect_node_rule_refs` → `pub(crate)`, so the retraction asks the same
    question the reachability analysis asks instead of growing a second traversal that can drift.
- [x] **ADDRESSED (measured, before → after)** — every number below is from this session's tools:

  | Measurement | Before (at `e1bff2e2`) | After |
  |---|---|---|
  | SV dead alternatives (post-elimination gen-AST sweep) | **4** (`select_expression` 3 of 8, `block_event_expression` 1 of 3) | **0** |
  | `fixed_select_expression_with.sv` (`with ( … )`) | REJECT | **ACCEPT**, arm `with_matches` |
  | `fixed_select_expression_or.sv` (`\|\|`) | REJECT | **ACCEPT**, arm `or>paren` |
  | `fixed_select_expression_paren.sv` (`&&` over a paren operand) | REJECT | **ACCEPT**, arm `and>paren` |
  | `fixed_block_event_or.sv` (`@@(begin m.t or end m.t)`) | REJECT | **ACCEPT**, arm `or` |
  | `_pgen_` marker in any published SV AST | — | **absent** (checked on all four) |
  | combinator suite cases | 32 | **35**, `2 passed / 0 failed`, 35/35 CLEAN |
  | `--lint-grammar` `left_recursive` | 32 | 30 (the two rules are now planner-eliminated) |

  ⭐ **The `&&` proof is `fixed_select_expression_paren.sv`, not the `&&` control.**
  `control_select_expression_and.sv` still reports `condition` + ZERO `and` nodes and its
  `condition,!and` masking pin still passes — correctly, because `.13c.2a.1`'s literal-brace defect
  still lets the seed swallow the remainder there. A2.5 did not fix that and does not claim to.
- [x] **NO REGRESSION** — `sv_syntax_closure_gate` **PASS with the contract UNCHANGED**
  (`unreachable_rules` 4 → **0**, `defined_rule_count=1481`, `unreachable_branches=2`; no cap moved,
  no rule blessed); `--lint-grammar` `non_terminating=0 unreachable_rules=0 undefined_references=0
  unbound_fact_kinds=0 profile_orphans=0 ordered_choice_shadowing=0`; adjudication oracle
  `checked=26 armed=6 listed=26 failures=0`, including `invalid_select_expression_double_matches.sv`
  still REJECT — a fix that revived `with` did **not** also admit the stacked `matches` tail, which
  is exactly what that row was pinned for while it was non-discriminating; the pre-existing
  `left_recursion` and `left_recursion_folded_ast` suite cases unchanged and still byte-identical.
  The pre-pass is a no-op on every grammar with no directly left-recursive alternative (**10 of the
  13 buildable grammars**). Iteration is over `rule_order`, never the `HashMap`, so codegen
  byte-determinism is preserved.
- [x] **LOCKSTEP** — see the leaf's LOCKSTEP list at the end of this section.

##### ⭐ What changed relative to the preserved patch — and why the difference matters

The landed change is **strictly larger** than the prototype, in three places. Each addition was
forced by an instrument, not by taste:

1. **`retract_consumed_normalization_rules` — the prototype leaked its own scaffolding.**
   `apply_left_recursive_chain_plan` *rewrites* the wrapper rules it consumes instead of deleting
   them, so after elimination each synthetic `_lr_altN` rule was still **defined and referenced by
   nothing**. `sv_syntax_closure_gate` caught it precisely: `unreachable_rules=4 >
   max_unreachable_rules=0`, naming all four (`reason=unreachable_from_entry`). ⛔ The tempting fix —
   raise the cap to 4 and bless them — is the move this project forbids, and the contract's own
   history says so: a prior baseline drove `max_unreachable_rules` **1 → 0** with the note *"the net
   of the LR-elimination synthetic rules no longer being created"*. Engine litter does not get a
   waiver a grammar would not get. The retraction deletes each synthetic rule once **nothing
   references it** (a fixed point, not a single pass — retracting one rule can orphan another), which
   is safe because planning has already baked the annotations into the `_pgen_lr_chain` templates.
   Result: `_lr_alt` occurrences in `generated/systemverilog_parser.rs` = **0**.
2. **A third combinator case the patch did not have — `direct_left_recursion_folded_ast`.** The
   patch's two cases are annotation-free, so they prove the direct shape *parses*; `.8`'s case proves
   annotations replay on a *wrapper* rule. Neither covers the composition SystemVerilog actually
   ships: annotations written on a **directly** left-recursive alternative, which the normalizer
   MOVES onto a synthetic rule. That is exactly the gap this leaf indicts elsewhere — a green gate
   that does not cover the changed thing — so the case was added and the gate asserts the **exact**
   left-nested value against the declaration. It is the isolating twin of `select_expression`'s
   `-> {kind: "and", lhs: $1, rhs: $3}`.
3. **Two new `Combinator` variants instead of reusing `LeftRecursion`.** The patch tagged its cases
   `LeftRecursion`; the landed version adds `DirectLeftRecursion` and `DirectLeftRecursionFoldedAst`,
   following `.8`'s precedent. The two shapes enter the engine through different doors — one is what
   the planner always matched, the other is what it never saw — so sharing a tag would let
   `combinator_coverage_is_complete` call direct LR covered on the strength of a case that never
   exercises the normalizer.

##### ⛔ A SECOND finding this leaf had to fix on the way: a durable repro that could not fail

`measure_direct_left_recursion_known_divergence` — the `--ignored` probe that documented the
direct-LR `furthest_position` divergence — is **retired**, and not only because its subject is fixed.
Running it showed all five of its oracle measurements were not measurements:

```
oracle=Err(Codegen { status: Some(1), stderr: "… no entry rule is declared. …" })
```

Its grammar constant declared no `@entry: true`, which `QUANT-PLUS-ITER.2` step C made a hard codegen
error on **2026-07-26**. From that date the probe compared one implementation against an error string,
printed the result, and exited 0 — because it `eprintln!`s and asserts nothing. Its stale numbers
(*"interpreter reaches 2/4, the generated parser stays 0"*) were still quoted as current in three
published surfaces: its own docstring, `TOOLBOX.md` §1.7, and the book's *Parse Harness* chapter. All
three are corrected here; the class (**7 print-only measurement probes**) is sized and routed to
`LANG-CAPABILITY-AUDIT.10.16`, the sibling of `.10.15`.


#### The LINTER half — ✅ **CLOSED by `A2.6`** (2026-08-12, `PGEN-GRAMMAR-WELLFORMED-0154`)

Both items below are done, and item 1's prescription was **corrected on measurement**: consulting
`detect_left_recursive_chain_plan` cannot work at the lint's position (post-pass, it answers `None`
for every rule, including the two it had just eliminated), so the verdict is derived from the pass's
own OUTCOME instead. The bigger finding is that "unearned" understated it — the message was false for
**30 of SV's 30** surviving cycles, and one of them costs LRM-legal text (`int'(2)'(3)`). See `A2.6`.
The original owed list, kept for the record:

1. Derive the `left_recursive` verdict from what the eliminator **actually accepts** — consult
   `detect_left_recursive_chain_plan` (post-normalization) rather than re-implementing its rules,
   the way `A2.3` made codegen and the linter share `effective_rule_branch_policy`. A cycle no plan
   covers must be a `dead_branch` **error**, named with the alternative index.
2. ⚠️ `main.rs:4200` prints only `lr.iter().take(10)` with no override, so 22 of SystemVerilog's 32
   left-recursive findings are invisible from the CLI; this session's sweep had to go around the
   instrument to see them. A capped diagnostic with no "show all" is how a finding hides.

#### The original finding, kept for the record

⛔ **This is contract item 3 — *no dead branches* — failing in the PASSING direction, and it is the
mirror image of `A2.3`.** `A2.3` was the linter hard-failing a live branch on a policy-blind
premise. This is the linter *blessing* a branch that cannot ever run, in a message that tells the
grammar author there is nothing to look at:

```
[info] grammar info: rule 'select_expression' is left-recursive (cycle: select_expression -> select_expression)
       — handled by PGEN's LR elimination + runtime cycle-breaking (informational, not an error)
```

**The claim is false for one shape, measured.** PGEN's LR elimination
(`ast_pipeline/mod.rs::detect_left_recursive_chain_plan`) rewrites only the **indirect wrapper**
form — an alternative that is a *bare rule reference* to a rule which itself begins with the base
rule. A **self-reference in first position inside a choice** matches nothing it looks for, so the
alternative reaches codegen intact and the runtime cycle guard meets it at the seed position. The
guard does not "handle" it; it **rejects** it:

```
🚪 Entering branch 3/8 for rule 'select_expression' at position 127
💥 Infinite recursion detected in rule 'select_expression' at position 127
```

⇒ the alternative is **dead code that the linter reports as clean**, and because the rule's later
arms still parse *something*, no corpus pass-rate and no `furthest_position` will ever point at it.
`SV-CORPUS-GRAD.13c.2a.2` found it only by reading the selected-branch trace.

**Sized, not guessed.** A sweep over the post-elimination gen-AST (`--dump-gen-ast`, alternatives
whose first element is a rule reference to the enclosing rule) puts the SV surface at exactly
**2 rules / 4 dead alternatives** out of 1 481 — `select_expression` (3 of 8) and
`block_event_expression` (1 of 3). Both are now owned: `SV-CORPUS-GRAD.13c.2a.2` and
`SV-CORPUS-GRAD.13c.2a.4`. ⛔ That the SV count is small is not evidence the *linter* defect is
small — it is a false-negative on a contract item, and the sweep that found it lives in a task leaf,
not in the instrument.

**Owed by this leaf:**
1. Split the `left_recursive` info class in two. An **eliminable** cycle (the wrapper shape the
   planner accepts) keeps today's informational message. A **direct** self-reference inside a
   multi-alternative choice is a `dead_branch` **error**, named with the alternative index, because
   it is exactly contract item 3 — and, per `A2.3`'s lesson, the verdict must be derived from what
   the eliminator *actually accepts*, not from a re-implementation of it that can drift.
2. ⭐ **Ask the eliminator, do not model it.** `detect_left_recursive_chain_plan` already returns
   `Option<LeftRecursiveChainPlan>`; the linter should consult that same function, the way `A2.3`
   made codegen and the linter share `effective_rule_branch_policy`. A second opinion about which
   shapes get rewritten is a second thing to keep in sync.
3. Decide whether the eliminator should simply **grow the direct case** — a directly left-recursive
   alternative is mechanically the wrapper shape with the wrapper inlined — which would make the
   linter's current message true instead of making it an error. ⛔ Open question, deliberately: it
   changes the AST shape of every affected rule, so it is a design call and not a bug fix.
4. ⚠️ `main.rs:4200` prints only `lr.iter().take(10)` with no override, so 22 of SV's 32
   left-recursive findings are invisible from the CLI; the sweep above had to go around the
   instrument to see them. A capped diagnostic with no "show all" is how a finding hides.

### `A2.6` — the linter's left-recursion verdict is UNEARNED, and 30 of SystemVerilog's 30 cycles are told "handled by PGEN" *after* the eliminator declined them (✅ **`done`** — `PGEN-GRAMMAR-WELLFORMED-0154`, 2026-08-12 session #220; opened + closed here as `A2.5`'s owed LINTER half)

> ⭐⭐ **This is `A2.5`'s LINTER half, promoted from a `#### Still todo` paragraph inside `A2.5` into an
> OWNED leaf.** That promotion is the `PGEN-SV-CORPUS-GRAD-0214` lesson applied on purpose: a leaf that
> owns work must be a NODE, not a sentence in its parent, or a later session re-finds the same defect
> from scratch. `A2.5` fixed the ENGINE so the message stopped being *false* for the two rules it
> repaired. This leaf is about the **other thirty**.

#### What was measured (tools first — three independent readings, one session)

**1. The linter's own output.** On the shipped SV grammar:

```text
$ ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar
grammar lint: 'systemverilog' (1485 rules) — left_recursive=30 (informational, handled by PGEN), …
  [info]  grammar info: rule 'casting_type' is left-recursive (cycle: casting_type -> constant_primary
          -> constant_primary_sv_2017 -> constant_cast -> casting_type) — handled by PGEN's LR
          elimination + runtime cycle-breaking (informational, not an error)
  …
  [info]  ... and 20 more left-recursive rules
```

**2. The elimination pass's own log, from the SAME run.** The `eprintln!` inside `ast_pipeline` is
`pgen_trace_debug!`, so the pass narrates itself at `PGEN_TRACE_VERBOSITY=debug`:

```text
🔁 Normalized 4 DIRECTLY left-recursive alternative(s) into the wrapper shape
✅ Rewriting left-recursive chain for rule 'block_event_expression' via helper 'block_event_expression_lr_base'
✅ Rewriting left-recursive chain for rule 'select_expression' via helper 'select_expression_lr_base'
🧹 Retracted 4 consumed normalization rule(s)
🏁 Completed left-recursion elimination pass (2 transformations)
```

**3. The order of the two.** `--lint-grammar` lints what `load_grammar_bundle` returns, and that path
runs `transform_from_raw_ast` → `eliminate_left_recursive_patterns` (`mod.rs:2847`, on by default)
**before** `run_grammar_lint` (`main.rs:1041`). ⇒ the 30 findings are exactly the cycles the
eliminator **already declined**: it ran, it rewrote 2 rules, and these 30 survived it. The sentence
*"handled by PGEN's LR elimination"* is therefore false for **30 of 30** printed findings — and it is
false in the PASSING direction, inside an `[info]` that tells the grammar author there is nothing to
look at. (This is also why `A2.5` moved the count 32 → 30: eliminating a rule removes it from the
cycle set entirely.)

⛔ **The `✅` in this tree's own frame is part of the defect.** *"PEG-complete … no direct/indirect
left recursion (PGEN eliminates ✅)"* is the same unearned claim one layer up, and it is corrected by
this leaf: PGEN eliminates the **wrapper** shape and — since `A2.5` — the **inline direct** shape.
An **indirect** cycle through N intermediate rules is not eliminated by anything.

#### The consequence is not cosmetic — a demonstrated SV parse gap

The runtime cycle guard does not *handle* a surviving cycle; it **cuts** it, by rejecting re-entry at
the same input position. Anything whose only derivation needs that re-entry is unparseable. Measured
on the very first cycle the lint prints (`casting_type -> constant_primary -> constant_cast ->
casting_type`), with an IEEE 1800-2017 A.8.4-legal construct (`casting_type ::= … | constant_primary`,
`constant_primary ::= … | constant_cast`, `constant_cast ::= casting_type ' ( constant_expression )`):

```text
$ parseability_probe --parse systemverilog control_cast.sv --profile sv_2017   # int'(3)
parse_full passed for grammar 'systemverilog' on '…/control_cast.sv'

$ parseability_probe --parse systemverilog nested_cast.sv  --profile sv_2017   # int'(2)'(3)
Error: parse_full rejected … Parser did not consume full input at position 0 [furthest_position=40, …]

$ PGEN_TRACE_VERBOSITY=debug … --trace-rules cast,casting_type,constant_cast,constant_primary
💥 Infinite recursion detected in rule 'casting_type' at position 32
❌ Exiting rule 'constant_cast' with error: InvalidSyntax { message: "Infinite recursion detected", position: 32 }
```

⇒ a real, LRM-legal SystemVerilog cast chain is REJECTED, by the exact cycle the linter blesses.
⭐ **That defect is NOT this leaf's to fix** — eliminating indirect left recursion is an ENGINE
capability, owned by `ENGINE-UNIVERSAL-SERVICES.13` (opened by this leaf, with this repro).
This leaf's job is that the instrument must stop calling it handled.

#### Class size (census, every buildable grammar, `--lint-grammar` headline)

| grammar | surviving left-recursive cycles |
|---|---|
| `systemverilog` | **30** |
| `systemverilog_lrm_profiled_wrapper` (raw Annex A) | **23** |
| `ebnf` | **5** |
| json, regex, vhdl, rtl_frontend, rtl_const_expr, svpp, the 3 annotation grammars | **0** each |

#### The fix (this leaf)

1. **Derive the verdict from what the eliminator ACTUALLY did, not from a belief about it.** ⭐ The
   derivation `A2.5` proposed — "consult `detect_left_recursive_chain_plan`" — is *weaker* than what
   the lint's position makes available, and would in fact return `None` for every rule (post-pass, a
   rewritten base rule no longer holds the wrapper alternatives the planner matches). The pass has
   **already run**, so its OUTCOME is the ground truth: it now reports which base rules it rewrote,
   that outcome is carried on the loaded grammar, and the linter classifies each surviving cycle
   against it. If the eliminator ever regresses, `select_expression` reappears among the survivors and
   the verdict flips by construction — which is what "earned" means.
2. **Say what is true, at a severity that is true.** A survivor becomes
   `left_recursion_unhandled` — a **warning**, naming the mechanism (the guard rejects same-position
   re-entry) and the consequence (those derivations are unreachable). ⛔ It is deliberately **not** the
   hard `dead_branch` **error** `A2.5` sketched: a surviving *indirect* cycle does not prove a specific
   alternative is dead — the intermediate rules may still have non-recursive paths, so that alternative
   can still parse something. Claiming deadness there would be the unsound-verdict mistake `A2.2`
   already retired once (`EarlierAlwaysMatches`), in the failing direction. What IS sound, and what the
   message states, is that the *left-recursive derivations* of that cycle are unreachable.
3. **Uncap the diagnostic.** `main.rs` printed `lr.iter().take(10)` with no override, hiding 20 of
   SV's 30 findings; every other class is capped at 40 with the same blind spot. `PGEN_LINT_DUMP_ALL=1`
   now prints every finding of every class. A capped diagnostic with no "show all" is how a finding
   hides — and this one hid 20.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `--lint-grammar` on `grammars/systemverilog.ebnf` prints
  `left_recursive=30 (informational, handled by PGEN)` and ten `[info] … handled by PGEN's LR
  elimination + runtime cycle-breaking` lines, then `... and 20 more left-recursive rules` — 20
  findings unreachable from the CLI at any verbosity.
- [x] **ROOT CAUSE (WHY + WHERE)** — the same `--lint-grammar` run at `PGEN_TRACE_VERBOSITY=debug`
  prints the elimination pass's own log, `🏁 Completed left-recursion elimination pass (2
  transformations)` naming only `block_event_expression` and `select_expression`, while the lint that
  runs *after* it (`main.rs:1041` `run_grammar_lint`, fed by `load_grammar_bundle` →
  `transform_from_raw_ast` → `eliminate_left_recursive_patterns`, `mod.rs:2847`) labels all 30
  survivors "handled". WHERE: `grammar_wellformedness.rs`'s `WellformednessIssue::LeftRecursive`
  message is a hard-coded claim about the engine that `detect_left_recursion` never checks — the
  detector reports a CYCLE and the message reports a HANDLING, and nothing joins them. Confirmed
  downstream by `--trace-rules cast,casting_type,constant_cast,constant_primary`:
  `💥 Infinite recursion detected in rule 'casting_type' at position 32` on an LRM-legal `int'(2)'(3)`
  that the probe rejects at `furthest_position=40`.
- [x] **FIX** — ENGINE/instrument tier, no grammar bytes. `eliminate_left_recursive_patterns` returns
  a `LeftRecursionEliminationOutcome` (did it run, which base rules it rewrote, which direct
  alternatives it normalized); `RustASTPipeline` exposes the outcome of the last transform;
  `LoadedGrammar` carries it; `classify_left_recursion` in `grammar_wellformedness.rs` splits the
  survivors (`LeftRecursionUnhandled`, warning, honest message) from the rules the pass actually
  eliminated (`left_recursion_eliminated`, info, earned); `PGEN_LINT_DUMP_ALL=1` uncaps every class's
  findings through one shared `print_lint_findings` helper (cap 40 for every class, where
  left-recursion's was 10).
- [x] **ADDRESSED (verified)** — `--lint-grammar` on SV now reports
  `left_recursion_unhandled=30 (warning …)` + `left_recursion_eliminated=2 (info — derived from the
  pass's own outcome)` and NAMES the two: *"ELIMINATED 2 left-recursive rule(s) on this grammar:
  block_event_expression, select_expression"*. Survivor lines printed **10 → 30** (the shared cap of
  40 now covers the whole class), and the false claim is gone —
  `--lint-grammar … | grep -c "handled by PGEN's LR elimination"` **10 → 0**. ⭐ The uncap is proven in
  BOTH directions on the one class that still exceeds 40, the Annex A wrapper grammar's 52
  always-succeeds notes: default prints 40 + `"... and 12 more … (set PGEN_LINT_DUMP_ALL=1 to print
  all 52)"`, `PGEN_LINT_DUMP_ALL=1` prints 52 with zero truncation lines.
- [x] **NO REGRESSION** — the eliminator's behaviour is untouched (the pass gained a return value and
  one `push`), and that is measured, not argued: `make regenerate_generated_parsers` then
  `shasum -a 256 generated/*.rs` against the pre-change snapshot — **all 11 generated parsers
  byte-identical**, so no parser, no cert run and no corpus verdict can have moved. Lint exit
  contract unchanged: all **12** shipped grammars still `rc=0` with `non_terminating=0
  ordered_choice_shadowing=0 unreachable_rules=0 undefined_references=0 unbound_fact_kinds=0
  profile_orphans=0`. `verilog_2005_conformance_gate` is the ONE gate that parses the lint headline,
  and its stage-1 lint lock is reproduced verbatim here — same binary, same contract
  (`grammars/systemverilog.ebnf`, `expected_exit_code 0`, `expected_profile_orphans 0`), same
  extraction (`grep -oE "profile_orphans=[0-9]+"`) → `rc=0 profile_orphans=0 orphan_lines=0`. ⛔ Its
  remaining stages are corpus accept/reject through the generated parsers, which are byte-identical,
  so re-running the ~20-minute release-probe rebuild could not have produced new information; that is
  a stated bound, not a skipped check. Dual-feature `--lib`: **1 110 ok, 1 failed, 3 unfinished** —
  and both non-green facts are accounted for rather than waved past. The failure is
  `ast_pipeline::ast_based_generator::semantic_usage_tests::unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`,
  proven **PRE-EXISTING by a stash A/B** (`git stash push -- rust/src` → identical panic,
  *"expected semantic_annotation fallback to detect '@' directives"* at
  `ast_based_generator.rs:14841`); it is the already-tracked `CI-PARITY-GATE-ROT` class-2 red, owned
  by `LANG-CAPABILITY-AUDIT.10.4`. The 3 unfinished are the deep-nesting stress tests
  (`parser_embedding_{systemverilog,vhdl}_deep_nesting_yields_clean_diagnostic_not_process_abort`,
  `tape_word_tests::an_event_word_can_never_be_minted_into_a_reference`), still burning 200 % CPU when
  the memory guard's 5 400 s timeout cut the run — CPU-bound, not hung, and they exercise the
  byte-identical generated parsers. ⭐ The gates that DO bind ran green inside that run:
  `parse_harness_combinator_suite::gate::every_structural_combinator_is_byte_identical` and
  `parse_harness_semantic_suite::gate::every_semantic_construct_is_byte_identical`;
  `stimuli/sv/run_adjudication_repros.py` green; `clippy_on_rust_change` rc=0 (strict source +
  generated-correctness policy, 68 pinned lints intact); `mdbook_docs_gate` green (10 per-parser books
  + the main book); `scripts/check_doctrines.sh` **18/18**. ⭐ The two new tests are RED-probed:
  deleting the outcome `push` fails
  `transform_from_raw_ast_reports_what_the_lr_pass_actually_eliminated`, and restoring it passes — the
  derivation cannot silently go empty.
- [x] **LOCKSTEP** — `TOOLBOX.md` (§5.1's derived-verdict entry + `PGEN_LINT_DUMP_ALL` registered in
  the family-1 signature table **and** in `scripts/check_diagnosis_evidence.sh`'s `DIAGNOSIS_SIG`, in
  this same commit — the instrument-registration obligation); the book
  (`grammar-wellformedness.md` incl. the stale *"PGEN eliminates left recursion for you"* contract
  line, `diagnosing-unknowns.md` ×2 index rows); the knowledge card
  `left-recursion-is-an-engine-service-not-a-grammar-authoring-burden.md` + its `KNOWLEDGE_MAP.md`
  reverify command (now also asserts the false claim is absent from the SV lint — re-run green);
  this tree's own frame (item 2's `PGEN eliminates ✅`); `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  `MEMORY.md`, `docs/TASK_TREE.md`. Routed engine defect: `ENGINE-UNIVERSAL-SERVICES.13`. No
  release/schema/ledger bump — the generated parsers are byte-identical, so the wire shape cannot
  have moved.

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
- `C2` — **semantic-prelude reach** — **`active` (activated 2026-06-10 for the regex store-gated
  pair `numeric_backreference`/`backreference_digits`, the LAST regex cert-coverage `UNKNOWN`
  residual).** Original framing: model the store during construction; for a `@predicate`-gated
  target, emit the `@emit_fact` prelude first, sequencing prelude→target. The "generator currently
  semantics-blind" clause is STALE since `STORE-AWARE-GEN.3` landed the generation-time store
  (`gen_semantic_state` + emit hook + `count(K)==0` prune); what is missing is exactly the PRELUDE
  SEQUENCING in the plannable-rule reach pass. Children: `C2.1` (design, docs) + `C2.2` (implement).
  - `C2.1` — **`done` (this slice, `PGEN-GRAMMAR-WELLFORMED-0063`, pure docs) — DESIGN: the
    fact-emitting-prelude reach capability (count-gated MVP).** Tool-backed root cause of the
    residual pair: `numeric_backreference = "\\" backreference_digits` (`grammars/regex.ebnf:253`)
    is gated `@predicate fact_count_at_least(regex_capture_group, $index)` with
    `backreference_digits = nonzero_digit digit+` (TWO+ digits ⇒ index ≥ 10), so a witness needs
    ≥ 10 capture groups BEFORE the backref. The reach pass cannot construct that today: phase-blind
    minimal construction either (a) fails fast at the STORE-AWARE-GEN.3 prune
    (`stimuli_generator.rs:5433`, `count(regex_capture_group)==0` at the target) → generation
    failure, or (b) with no prune would emit `\NN` with zero groups → the parser correctly rejects
    → never witnessed. Declarative options are structurally out: an entry-level `@sample` collapses
    ordinary generation; `@probe_sample` on the entry fires for ordinary top-level generation too;
    a target-level `@sample: "\\10"` renders a backref with no preceding groups (sample rejects).
    DESIGN (parser-agnostic, plan-scoped, two-phase; all refs verified live):
    (1) `ActiveReachPlan` gains `prelude: Option<ReachPrelude>` (default `None` in
    `from_directives` ⇒ every existing caller byte-identical by construction). `ReachPrelude` =
    { the chosen on-path quantifier `site: (rule, node_path)`, the site's quantified-body rule,
    the producer rule, the precomputed body→producer sub-plan directives + quantifier mins,
    `iterations: usize` (0 = disarmed), `captured: Option<(rule, text, value)>` }.
    (2) INSTALL (in `set_reach_plan_for_rule`, plannable mode only): find the FIRST count-gated
    rule along the hop path (rule ∈ `gen_count_kinds` — for target `backreference_digits` that is
    `numeric_backreference` ON the path; for target `numeric_backreference` it is the target
    itself); its counted kind K; producers = rules whose `gen_emit_facts` specs emit K (regex:
    `capture_open` + the 3 named-group open markers), sorted for determinism; prelude site = the
    LAST (innermost) on-path quantifier site (`quantifier_sites_along_path` order) whose quantified
    element resolves to a direct `RuleReference` body rule b with `reach_hops(b, producer)` ≠ None
    (regex: `concatenation`'s `piece+`, body `piece`, producer `capture_open`). No qualifying
    site / no count-gated rule on path ⇒ `prelude = None` (pure pre-existing behavior — vhdl/
    rtl_frontend/SV paths have no count-gated rules ⇒ no-op cross-grammar by construction).
    (3) PHASE 1 (capture): while a plannable plan with a prelude spec is installed, the
    STORE-AWARE-GEN.3 prune is BYPASSED for the count-gated rule on the plan path (scoped — other
    rules still prune), and on that rule's successful generation the plan captures (rendered text,
    numeric value v = first maximal decimal run in the render, parsed `usize`, sanity-capped).
    The phase-1 sample (e.g. `\37` with no groups) is expected NOT to re-parse — counted in the
    pass's existing auxiliary `probe_parse_failures`, never in certification spf.
    (4) ARM (driver, `generate_plannable_rule_witnesses` attempt loop): after a failed attempt,
    if the plan captured (text, v) and `iterations == 0` → arm `iterations = v` for the remaining
    attempts (fits the H.7.1 ≤ 4-attempt budget: attempt 1 captures, attempt 2 witnesses).
    (5) PHASE 2 (prelude + replay): in `generate_quantified`, when the active plan's prelude site
    matches and `iterations > 0`, generate `v` PRELUDE iterations of the quantified body FIRST
    (each under the precomputed body→producer sub-plan, main plan saved/restored around each;
    minimal expansion ⇒ regex renders `()` per iteration via `capturing_group = capture_open
    pattern? ")"`), then the normal forced on-path iteration(s); in `generate_rule`, the count-gated
    rule with a captured text short-circuits to that text (hint-route bookkeeping:
    `record_rule_success` + follow-restriction + tail word-shape + atomicity flags) so the replayed
    value EQUALS v by construction. Resulting sample `()()…()\v` (v groups then `\v`): the parser's
    post-predicate `fact_count_at_least(regex_capture_group, v)` holds at the backref ⇒ the
    accepted parse enters BOTH pool rules ⇒ witnessed (the parser stays the judge — a
    mis-extraction can only fail loudly, never false-witness).
    (6) Store interplay: prelude iterations emit K facts via the existing
    `gen_emit_facts_for_rule` success hook; the existing `generate_quantified` checkpoint/rollback
    discipline already isolates failed candidates; `generate_from_entry` already resets the store
    per attempt. Sub-plans carry no prelude ⇒ no recursion of the prelude logic.
    Acceptance for `C2.2`: regex cert-coverage `UNKNOWN 2→0` ⇒ `fully_certified=true` (the 4th
    fully-certified grammar after json/rtl_const_expr/svpp), `witness 196→198`, `spf=0`, IDENTICAL
    seeds 0/7/42; cross-grammar byte-identical certification (json/rtl_const_expr/svpp stay
    `fully_certified`; vhdl/rtl_frontend/SV spf byte-identical, `UNKNOWN` unchanged-or-better);
    `regex_pcre2_compile_oracle_gate` PASS (generator-only — NO parser/grammar change, NO regen,
    NO release/schema bump, the STORE-AWARE-GEN.3 surface-neutral precedent); unit locks (synthetic
    count-gated grammar witness + no-prelude no-op); lib suites green both feature sets; clippy
    strict-source clean; book (grammar-wellformedness chapter: the third pass's prelude extension +
    the `98→…→2→0` arc) + tracker + continuity docs in lockstep.
  - `C2.2` — **`done` (`PGEN-GRAMMAR-WELLFORMED-0064`) — IMPLEMENTED the count-gated prelude MVP
    exactly per the `C2.1` design** (engine, `stimuli_generator.rs` only: `ReachPrelude` +
    `ActiveReachPlan.prelude` (default `None`) + install-time `compute_reach_prelude` /
    `quantified_body_rule_name` + phase-1 scoped prune bypass + capture
    (`reach_prelude_capture`, first-decimal-run value, `REACH_PRELUDE_MAX_ITERATIONS=4096` cap) +
    driver arming in `generate_plannable_rule_witnesses` + phase-2 prelude injection in
    `generate_quantified` (sub-plan swap per iteration) + captured-render replay in
    `generate_rule` (hint-route bookkeeping)). One live nuance vs the design: the phase-1 probe
    PARSES (the `\NN` degrades to octal per RGX-0084) and routes elsewhere — the
    `ParsedNotWitnessed` flavor of the expected phase-1 failure; arming is outcome-independent,
    so the flow is unchanged.
    Verification: `done — **regex cert-coverage `UNKNOWN 2→0` ⇒ `fully_certified=true` — the 4th
    fully-certified grammar (after json/rtl_const_expr/svpp): `total=198 witness=198 spf=0`,
    IDENTICAL at seeds 0/7/42 × counts 1/40/200, and the same count-40/seed-0 invocation run
    TWICE is byte-identical (DETERMINISM-OK)**; reach pass reports 0 not-re-parsed probes / 0
    generation failures at every regex measurement. CROSS-GRAMMAR (canonical invocations,
    count 40 seed 0): json `9/9 fully_certified`; rtl_const_expr (`--max-depth 32`)
    `48/48 fully_certified`; svpp `74/74 fully_certified` (seeds 0 AND 7); vhdl
    `total=217 UNKNOWN=30 spf=0` (baseline 30, UNCHANGED); rtl_frontend
    `total=170 UNKNOWN=73 spf=0` (baseline 73, UNCHANGED); SV (`sv_2017`)
    `total=1342 UNKNOWN=647 spf=0` + the same 68-rule no-path warning (baseline 647+68,
    UNCHANGED). `regex_pcre2_compile_oracle_gate` PASS (zero parse-path change — generator-only,
    NO regen, NO release/schema bump); `stimuli_cross_family_platform_gate` PASS;
    `mdbook_docs_gate` PASS (chapter arc `…→2→0` + the new semantic-prelude section). Suites:
    default lib 638/0 (+2 new locks: `semantic_prelude_witnesses_count_gated_rule` end-to-end
    two-phase on a synthetic count-gated grammar incl. exact prelude-size==value assertion;
    `reach_plan_for_rule_without_count_gated_path_has_no_prelude` no-op lock); dual-feature lib
    731/0; clippy strict-source clean.` The general non-count predicate prelude (`has_fact`
    value-selection — STORE-AWARE-GEN `.4b` territory) stays OUT of scope until evidence demands
    it (no such residual exists today).

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

### Phase G — the CERTIFYING LINTER (trustworthiness — director directive 2026-06-06)

**Frame (binding, from an extended director brainstorm 2026-06-06).** The linter is the FULCRUM of
sign-off (prover + adjudicator + theorem-maker), so it must be one we NEVER have to doubt. Trust is
EARNED, not asserted: make the linter a **certifying algorithm** (Mehlhorn/McConnell et al.) — every
verdict ships a checkable CERTIFICATE that a tiny independent CHECKER validates; we trust the small
checker, not the linter's internals. Theoretical bound (honest): exact reachability is UNDECIDABLE for
this grammar class (structural reachability decidable — Hopcroft–Ullman; PEG arm-selection +
data-dependent predicate truth undecidable — Rice). So the linter is necessarily SOUND-not-COMPLETE:
it NEVER falsely convicts, and the cost is an honest `UNKNOWN` for what it can't settle (NEVER a
guess). Certificates: **reachable → a WITNESS** (derivation + input string; the stimuli GENERATOR is
the witness producer = the duality made operational; replay through the real parser); **unreachable →
a PROOF** (the decidable argument: which sound rule fired + the chain; re-validated by the checker);
`UNKNOWN` → no certificate (honest). BINDING DISCIPLINE: **no definite verdict without a certificate.**
Undecidability lives ENTIRELY in `UNKNOWN`; the theorem is about ALL grammars, NOT the one we ship —
so for the actual SV grammar we DRAIN `UNKNOWN` to ZERO (every fragment witnessed-reachable or
proven-unreachable; the residue is adjudicated once per the attribution rule → witness or proof,
never silently accepted). Goal = "100% SOUND with `UNKNOWN` driven to 0 and never hidden" → verified,
not trusted. Recorded: [[feedback_certifying_linter_trustworthiness]] + book "Trusting the linter:
certificates, not faith".

- `G.1` — **certificate MODEL + the independent CHECKER (seed) — DONE (PGEN-GRAMMAR-WELLFORMED-0012).**
  `UnreachabilityCertificate { rule, node_path, dead_index, reason }` + `UnreachabilityReason`
  {`DuplicateOf` | `FixedTerminalPrefixBy` | `EarlierArmAlwaysSucceeds`}; `ShadowingIssue::certificate()`
  emits the structured PROOF for each shadowing `dead` verdict. `verify_unreachability_certificate`
  (grammar, cert) is the INDEPENDENT CHECKER: `navigate_node_path` re-locates the cited `Or` node from
  the grammar AST and re-derives the deadness DIRECTLY (exact-dup + fixed-prefix re-checks are trivial +
  fully independent; always-succeeds re-derives via `node_always_succeeds`) — it never trusts the
  detector. Unit-tested: every real certificate re-verifies; tampered (dead_index=shadower), bogus
  (false duplicate claim), and unresolvable-path certificates are all REJECTED — proving the checker
  validates rather than rubber-stamps. Pure analysis, no regen; module suite 23/23. NEXT (`G.1.1`,
  optional refinement): structural always-succeeds witnesses (trivially checkable, no fixpoint) for
  full independence; extend certificates to A1b unreachable-rule / profile-orphan / F1 unbound-fact.*
- `G.2` — **the independent CHECKER** (small, auditable, separate from the linter): re-validates each
  unreachability PROOF independently + (later) replays each reachability WITNESS. **IN PROGRESS
  (`-0015`):** generalized `WellformednessCertificate` {`DeadAlternative` | `UnreachableRule`} +
  `verify_wellformedness_certificate` — the rule-level `UnreachableRule` checker re-derives reachability
  via a shared `reachable_rules` helper (refactored out of `detect_unreachable_rules`); unit-tested
  (valid cert verifies; a reachable rule claimed unreachable is rejected; dispatch to the shadowing
  checker works). **`G.2.1` (`-0016`):** added the `UnboundFactKind` certificate — `verify_wellformedness_certificate`
  now takes `annotations` and re-collects emitted kinds via the extracted `collect_emitted_fact_kinds`,
  confirming the consulted kind is unemitted; unit-tested (valid verifies; an emitted kind claimed
  unbound is rejected; honestly refuses without annotations). **`G.2.1b` (`-0017`):** added the
  `ProfileOrphan` certificate — `extract_profile_context` pulls `@profiles`/universe from the
  annotations, the checker re-derives per-profile satisfiability (`compute_sat_by_profile`) and confirms
  present-but-unsatisfiable-here + satisfiable-elsewhere (faithful to `detect_profile_orphans`);
  unit-tested (valid verifies; an orphan claimed under a satisfiable profile is rejected). **⇒ the
  PROOF-side certifier is COMPLETE — all 4 decidable dead-verdict types (shadowing, unreachable-rule,
  unbound-fact, profile-orphan) ship a checkable certificate, each with valid-verifies + bogus-rejected
  tests.** REMAINING: `G.2.2` standalone checker binary; witness replay (reachable side) = `G.3`.
  *Effort: medium-high.*
- `G.3` — **witness producer wiring**: the stimuli generator emits, per reachable fragment, a minimal
  witness input (the duality); fragments it can't witness become `UNKNOWN` tickets. *Effort: high
  (reuses the generator's reach/replay machinery).*
  - `G.3.1` — **DONE (`-0018`): the WITNESS certificate model + the independent checker.**
    `ReachabilityWitness { fragment, input }` (constructive dual of the unreachability PROOF) +
    `verify_reachability_witness(parse_and_cover, witness)` — parser-AGNOSTIC (the caller passes a
    closure that replays `input` through the real grammar parser and returns `(parsed, fragments_
    exercised)`); the witness holds iff the input PARSES and EXERCISES the claimed fragment. A witness
    that doesn't parse, or parses but misses its fragment, is REJECTED. Unit-tested with a mock parser
    (valid / no-parse / wrong-fragment). Mirrors the proof-side "model + checker first" pattern.
  - `G.3.2` — generator EMITS witnesses — **DONE (`-0019`).** `StimuliGenerator` now captures a
    `ReachabilityWitness { fragment: target.id, input: witness_sample }` for each resolved reachable
    target in `generate_target_witnesses` (at the per-target success arm, `stimuli_generator.rs:2680`),
    cleared per pass and exposed via `pub fn witness_certificates(&self) -> &[ReachabilityWitness]`.
    Purely ADDITIVE (a new field mirroring `generation_step_counter` + the getter; no generation-logic,
    return-type, or caller change), so it is verified by lib-build (no closed-loop needed — the captured
    witnesses are exactly what the already-validated witness pass produced). The constructive half of the
    duality now produces certificates the `verify_reachability_witness` checker (G.3.1) consumes.
  - `G.3.3` — the real `parse_and_cover` — **DONE (`-0021`).** Two pieces: (1) `parse_node_covered_rules`
    (grammar_wellformedness.rs) — the parser-AGNOSTIC SOUND coverage extractor: walks a SUCCESSFUL
    `ParseNode` tree collecting the rule names PRESENT in it (NOT the speculatively-attempted-and-failed
    rules a per-rule call counter would over-count → unsound), unit-tested + composed with the witness
    checker; (2) `parser_registry::parse_and_cover_systemverilog(sample, profile)` — the SV glue: build
    the real SV parser, `parse_full_systemverilog_file`, walk the AST → `(parsed_ok, rules_exercised)`,
    the exact closure `verify_reachability_witness` needs. Compiles under `--features generated_parsers`.
    ⇒ the WITNESS side is wired END-TO-END: generator EMITS witnesses (G.3.2) → `parse_and_cover_*`
    (G.3.3) → `verify_reachability_witness` (G.3.1). (Branch-level coverage + per-grammar `parse_and_cover_*`
    = Phase H follow-ups.) Then `G.4` ties proof+witness: every fragment a verified PROOF or WITNESS ⇒
    `UNKNOWN`=0 (the objective trust number).
- `G.4` — **certificate-COVERAGE gate**: for the SV grammar require every fragment to carry a valid
  certificate (`UNKNOWN` = 0) with all certificates checking → HARD gate. *That number, at 0, is the
  objective proof the linter is trustworthy on this grammar.* Folds in A2.1 (each grammar fix moves a
  fragment from `dead`/`UNKNOWN` to witnessed-reachable).
  - `G.4.1` — **DONE (`-0022`): the capstone coverage LOGIC.** `certificate_coverage(all_fragments,
    proof_covered, witness_covered) -> CertificateCoverageReport { total, covered_by_proof,
    covered_by_witness, unknown }` + `is_fully_certified()` (⟺ `unknown` empty). Pure + deterministic;
    the caller passes the ALREADY-VERIFIED proof/witness fragment sets (each fragment passed its
    independent checker). Unit-tested (proof/witness/UNKNOWN classification; fully-certified flips when
    the last UNKNOWN gets a witness). **⇒ the ENTIRE certifying-linter framework now exists + is
    unit-tested: PROOF side (4 verdict checkers) + WITNESS side (model/producer/checker) + the coverage
    capstone.**
  - `G.4.2` — the ORCHESTRATION — **DONE (`-0023`): the gatherers (logic complete + tested).**
    `gather_verified_proof_covered_rules(grammar, rule_order)` — runs `detect_unreachable_rules`, builds
    `UnreachableRule` certs, RE-VERIFIES each via `verify_wellformedness_certificate`, returns the
    covered set + any re-verify `failures` (a failure = a linter bug, never silently counted).
    `gather_verified_witness_covered(witnesses, parse_and_cover)` — re-verifies each witness via
    `verify_reachability_witness`, returns the covered set + `failures` (a failure = a generator/witness
    bug). Both compose into `certificate_coverage` → `is_fully_certified()`. Unit-tested end-to-end with a
    mock parser (dead island proven + reachable rules witnessed ⇒ fully certified; a non-parsing witness
    is a recorded failure, not counted). **⇒ the WHOLE framework + orchestration is in code + tested;
    the only un-exercised bit is the real-SV invocation.**
  - `G.4.3` — the real-SV GATE RUN (heavy). **DE-RISKED + wiring located (2026-06-06):** (a) the
    integration is VIABLE — `cargo build --bin ast_pipeline --features ebnf_dual_run,generated_parsers`
    COMPILES (so one binary can both generate witnesses AND call `parse_and_cover_systemverilog`); the
    `has_generated_systemverilog_parser` cfg is set when the SV parser artifact exists. (b) WIRING POINT:
    `main.rs` ~`:1474`, right after `generator.generate_target_witnesses(&target_report.targets)` — at
    that point `generator.witness_certificates()` holds the `ReachabilityWitness`es (G.3.2),
    `resolved_entry_rule` + `args.grammar_profile` are in scope. Add a `#[cfg(has_generated_systemverilog_parser)]`
    block: `gather_verified_witness_covered(generator.witness_certificates(), |s| parse_and_cover_systemverilog(s, profile))`
    + `gather_verified_proof_covered_rules(grammar, rule_order)` (needs the raw grammar/rule_order in scope
    — add a generator accessor or thread them in) → `certificate_coverage(rule_order, …)` → print
    proof/witness/UNKNOWN + hard-gate on `is_fully_certified()`. (c) RUN = the HEAVY closed-loop: it needs
    a precomputed gap report (`--gap-priority-report-input`) then the witness pass over the full SV grammar
    (minutes; mind the uvm-memory caveats) — this is the verification the director deferred "we'll see
    later". **The objective SV `UNKNOWN` number lands here** (closely tracks the existing coverage residual,
    now verified + split into proof/witness/unknown). Effort: medium wiring + a heavy run.
    **WIRING DONE (`-0025`):** `main.rs` (after the witness pass) now has a
    `#[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]` block that, for the SV
    grammar, runs `gather_verified_witness_covered(generator.witness_certificates(), |s|
    parse_and_cover_systemverilog(s, profile))` + `gather_verified_proof_covered_rules(&grammar.grammar_tree,
    &grammar.rule_order)` → `certificate_coverage` and prints `CERTIFICATE-COVERAGE (G.4): total=… proof=…
    witness=… UNKNOWN=… fully_certified=… (re-verify failures: proofs=…, witnesses=…)`. Compile-verified
    BOTH ways: dual-feature build (`--features ebnf_dual_run,generated_parsers`) active, default build
    (`ebnf_dual_run` only) cleanly compiles it out. ⇒ the binary now PRODUCES the number; only the heavy
    RUN remains. RUN command: build dual-feature, generate a gap report for SV, then run the witness path
    with `--gap-priority-report-input <report>` (the SV closed-loop, minutes; mind uvm-memory) → reads the
    `CERTIFICATE-COVERAGE` line. That run is the deferred heavy verification.
    **FIRST REAL RUN (`-0026`, bounded to 40 targets, sv_2017):** `CERTIFICATE-COVERAGE (G.4): total=1339
    proof=0 witness=0 UNKNOWN=1339 fully_certified=false (re-verify failures: proofs=0, witnesses=10)`.
    The gate RAN end-to-end on real SV — and surfaced a genuine finding (the gate WORKING, per the
    attribution rule: it refused to count witnesses that don't verify). ROOT CAUSE: the witness pass roots
    each witness at the TARGET'S OWN RULE → a sub-rule FRAGMENT (e.g. a bare `expression`), but
    `parse_and_cover_systemverilog` parses via `parse_full_systemverilog_file` (the TOP entry), so a bare
    fragment doesn't parse as a full SV file → all 10 witnesses correctly rejected. ⇒ **`G.4.4`**: the
    witness↔checker contract must agree on PARSE ENTRY — either (a) the witness pass produces FULL-FILE
    witnesses that embed the fragment (so `parse_full_systemverilog_file` covers it), or (b) verify from
    the target's rule via a per-rule parse entry (`parse_full_<rule>`, where the generated parser exposes
    one) / a coverage-instrumented sub-parse. ALSO the `fragment` identity: witness fragment = target id
    (may be `rule#branch`) vs `parse_node_covered_rules` = bare rule names → align (branch-level coverage).
  - `G.4.5` — **the CLEAN dedicated mode (`-0028`) + two DEEP findings.** Replaced the rushed
    concern-mixed witness-path block with a dedicated, parser-AGNOSTIC `--report-certificate-coverage`
    mode (`run_certificate_coverage_report`, modelled on `--report-k-path-coverage`): build generator →
    generate CLEAN diverse samples → `parser_registry::parse_and_cover(grammar.grammar_name,…)` (generic,
    no grammar names) → `gather_verified_proof_covered_rules` → `certificate_coverage` → report. Builds
    both ways; the concern-mixed `--generate-stimuli` block is removed. **RUN (50 diverse, sv_2017):**
    `total=1339 proof=0 witness=1 UNKNOWN=1338 (sample_parse_failures=25, proof_reverify_failures=0)` —
    surfaced TWO real issues the gate exists to find:
    - **(F1) AST-walk coverage is wrong for ANNOTATED grammars.** `parse_node_covered_rules` collects
      `ParseNode.rule_name`s, but a rule with a return annotation (`-> {…}`) folds its subtree into
      `ParseContent::Json` (no child `ParseNode`s) — so the walk stops at the first annotated rule and
      never sees `expression`/`identifier`/… → `witness=1`. The AST-walk massively UNDERCOUNTS. The
      witness-coverage SOURCE needs a redesign (see the design fork below).
    - **(F2) raw `generate_many` emits unparseable samples** (25/50) — the closed-loop gate
      filters/retries for parseability; raw generation does not. The witness side must use the
      parseable-stimuli path (or filter), not raw `generate_many`.
  - `G.4.6` — **RESOLVED (`-0029`): TRANSACTIONAL PARSE-COVERAGE — the parser testifies to what it
    parsed.** The fork was decided in favour of option (b) — parser-instrumented coverage — as the only
    one consistent with "verified, not trusted" (option (a) trusts the generator's coverage; (c)'s
    raw-parse mode would need every rule to emit two trees since the `-> {…}` transform builds `Json`
    directly with no retained structural tree). The design is elegant and exploits the EXISTING universal
    speculation choke point:
    - **Codegen** (`ast_based_generator.rs`, parser-AGNOSTIC — every generated parser gains it): a
      transactional `coverage_stack: Vec<u32>` + opt-in `coverage_enabled` flag on the parser struct.
      At rule entry (beside the call-counter) `if coverage_enabled { coverage_stack.push(rule_id) }`.
      In `try_parse` (the universal wrapper for `|`/`?`/`*`/`+`/`&`/`!`) snapshot `saved_coverage_len`
      next to the existing position/parse-stack/semantic-checkpoint snapshots; on `Err`, `truncate` back.
      So a rule entered inside a rolled-back speculation has its push removed too. After a SUCCESSFUL
      top-level parse every failure necessarily happened inside some rolled-back `try_parse`, so the
      surviving entries are EXACTLY the accepted-parse rules — **sound** (no backtracked attempts, which a
      call counter over-counts) and **complete** (annotation `Json`-folding can't hide an ENTRY).
      Public API: `enable_coverage()` + `exercised_rule_names() -> HashSet<String>` (via `RULE_NAMES`).
    - **Registry** (`parser_registry.rs`): `parse_and_cover_systemverilog` now `enable_coverage()` →
      parse → `exercised_rule_names()` (replacing the broken `parse_node_covered_rules` AST-walk). The
      AST-walk is retained ONLY for annotation-free unit-test trees, with a ⚠️ doc warning.
    - **Opt-in** so ordinary parsing pays nothing (empty stack ⇒ O(1) try_parse snapshot/truncate).
    - **Pinned** by a codegen render-test (`transactional_parse_coverage_wiring_is_emitted_at_codegen`).
    - **VERIFIED:** same gate (50 diverse, sv_2017, seed 1) went `witness=1 → witness=127` (deterministic
      across two runs); `total=1339 proof=0 UNKNOWN=1212 (sample_parse_failures=25)`. The F1 finding is
      FIXED. **F2 still open** (25/50 diverse samples don't parse — next leaf: feed the witness side from
      the parseable-stimuli path so the count isn't capped by unparseable samples).

  - `G.4.7` — **F2: the witness-sample parse failures (25/50).** The witness count is capped because
    half the diverse samples don't parse. Director directed a tools-first investigation BEFORE fixing
    (avoid a band-aid masking a real defect).
    - **Slice 1 (`-0030`, DONE) — investigate + build the diagnostic.** Made the gate LABEL its
      sample-parse failures instead of silently counting them ("never silently dropped"): new registry
      `parse_detail` hook + `parse_error(grammar,…)` (data-driven, parser-AGNOSTIC — the SV detail
      parser, which augments errors with `furthest_position`, lives in the registry table as data); the
      gate now prints `SAMPLE-PARSE FAILURES` with the error + sample for the first few. **ROOT CAUSE
      (verified with the tool, not inferred):** every failure is "did not consume full input" caused by
      MISSING mandatory whitespace between adjacent word-like tokens — the diverse generator emits
      `endprogram`+`module` as `endprogrammodule`, `generate`+`endgenerate` as `generateendgenerate`,
      `default`+`liblist` as `defaultliblist`, `timeunit`+`99356` fused, etc., so the lexer reads one
      wrong token and the parse stops. Confirmed at source: `append_generated_segment`
      (stimuli_generator.rs:7059) inserts the separator ONLY when `config.enforce_word_boundary_spacing`
      is true, and the gate's `StimuliConfig::default()` leaves it **false** (verified at :196). So this
      is a GENERATOR defect (invalid SV), but the fix is LEVEL-1 — enable an EXISTING feature, not new
      code. ATTRIBUTION-RULE outcome: a generator deficiency, not ill-formed EBNF.
    - **Slice 2 (`-0031`, DONE) — fix + measure.** Enabled `enforce_word_boundary_spacing: true` on the
      gate's witness `StimuliConfig` (level-1, existing feature — one line, no new code). **MEASURED
      (50 diverse, sv_2017, seed 1), deterministic:** `sample_parse_failures 25 → 1`, `witness 127 →
      199`, `UNKNOWN 1212 → 1140`. 24 of 25 failures resolved by the one change.
    - **Slice 3 (NEXT) — the residual 1/50, a SECOND distinct defect (confirmed with the tool, NOT a
      guess).** The first guess (malformed numeric/time literals `782_'daAD_`, `5907.5_80e280`) was
      DISPROVEN — they parse fine in isolation. The real cause: the generator emits a `//` LINE COMMENT
      with NO terminating newline; since the whole sample is one line, the comment swallows everything
      to EOF (incl. the `;` terminator) → "did not consume full input". Minimal repro confirmed:
      `package p; timeunit 1 ps //c` + newline + `; endpackage` PASSES, but `package p; timeunit 1 ps
      //c ; endpackage` (one line) FAILS. Fix direction: ensure generated `//` line-comment trivia is
      newline-terminated (or not emitted where a newline can't follow). Then consider whether
      `enforce_word_boundary_spacing=true` should be the GLOBAL default (broader decision — may affect
      negative-test generation that intentionally wants boundary violations).

### Phase H — ALL-GRAMMARS certification (director directive 2026-06-06)

**Scope (binding).** Full certification is NOT SV-only — EVERY PGEN grammar (SystemVerilog, VHDL,
regex, RTL const-expr/frontend, the annotation/EBNF/preprocessor grammars, and any FUTURE grammar)
must reach the same bar: static checks pass (well-formed + well-defined) AND `UNKNOWN`=0 with all
certificates checking (G.4 run per grammar). The machinery already generalizes — the linter +
Phase G are PARSER-AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]), so each grammar uses the
SAME certification unchanged. HEAD START (the F1 all-grammars sweep, `-0008`): the hand-authored
non-SV grammars are ALREADY at 0 always-matches / 0 unbound-fact / 0 unreachable / 0 orphan — they
are statically clean; SV is the outlier (its LRM-PDF extraction artifacts). So per-grammar
certification = static checks (mostly already green off-SV) + the per-grammar G.4 coverage gate.
- `H.1` — make `G.4`'s certificate-coverage gate parameterized PER GRAMMAR (not SV-hardcoded).
- `H.2` — roll each non-SV grammar to full certification (VHDL, regex, RTL, …) — most are a short hop
  given they are already statically clean; the work is mainly witness coverage (G.3 per grammar).
- `H.3` — a new grammar is "done" only when it is fully certified (add to the per-grammar gate). The
  universal closure bar already in `LIVE_ACHIEVEMENT_STATUS.md` is extended with "fully certified".

### `H.15` — **THREE GRAMMARS HAVE NO CERTIFICATE COVERAGE AT ALL, AND THEY ARE THE UNMET HALF OF A DIRECTOR ACTIVATION GATE** (**`done`**, `PGEN-GRAMMAR-WELLFORMED-0156`, CODE / registry-only — opened 2026-08-22 session #254 by `SV-CORPUS-GRAD.13c.2x.9`(c1), CLOSED 2026-08-22 session #255)

- ⛔⛔ **WHY THIS IS NOW URGENT RATHER THAN TIDY.** `SVPP-EXPANSION` is `proposed` behind an explicit,
  MEASURABLE director activation gate (2026-06-08): *"sequenced strictly AFTER the locked program
  (every existing parser cert-coverage **WIRED** + **clean** + `UNKNOWN`=0)."* On 2026-08-22
  `SV-CORPUS-GRAD.13c.2x.9`(c1) took SystemVerilog — the last family carrying `UNKNOWN > 0` — to
  `UNKNOWN=0`, so the **clean** conjunct is met across every family that can be measured. **WIRED is
  a separate conjunct and it is NOT met.** `SV-CORPUS-GRAD.13` prices the expansion at **5 539 corpus
  rows = 33.9 %** of the SV denominator, so this leaf is what stands between that measurement and the
  gate.
- ⛔ **MEASURED, by name, not inferred** — `ast_pipeline grammars/<g>.ebnf
  --report-certificate-coverage --count 40 --seed 0` refuses on each of the three:

  ```text
  Error: certificate-coverage: no generated parser is registered for grammar 'ebnf' —
    cannot verify reachability witnesses through a real parser (Phase …)
  ```

  | grammar | `parse_and_cover` registered? | cert measurable? |
  |---|---|---|
  | `ebnf` | **no** | **no** |
  | `return_annotation` | **no** | **no** |
  | `semantic_annotation` | **no** | **no** |
  | `json`, `regex`, `vhdl`, `systemverilog_preprocessor`, `rtl_frontend`, `rtl_const_expr`, `systemverilog` | yes | yes, and every one reads `UNKNOWN=0` |

- ⭐ **PHASE H'S OWN SCOPE NOTE NEVER NAMED THEM.** `H.1`/`H.2` enumerate *"vhdl / svpp / rtl_* /
  json"* and the frontier row reads *"all SHIPPED grammars wired"* — which is true, and is exactly
  how three families ended up outside the sentence. They are not shipped parser families in the
  delivery sense, but they ARE in the `done_bar_family_register` roster with live `claimed_status`
  rows, and the director's gate says *every existing parser*, not *every shipped parser*.
- **WHAT THIS LEAF OWES**: (a) establish, per grammar, whether a generated parser exists under that
  grammar's own name at all — `return_annotation`/`semantic_annotation` are generated as a PAIR and
  `ebnf` is the bootstrap seed, so the answer may be *"registered under a different key"* rather than
  *"absent"*, and those are different fixes; (b) wire `parse_and_cover_<grammar>` for each the way
  `H.1`/`H.2` did for regex and vhdl (registry fn + entry field; the G.4.6 coverage instrumentation
  is emitted UNCONDITIONALLY by codegen, so no codegen change should be needed); (c) run the coverage
  and record the tuple per grammar; (d) ⛔ **only then** may anyone state that the `SVPP-EXPANSION`
  activation gate is met — and (d) is the point of the leaf, not (b).
- ⚠️ **BOUND**: this leaf does NOT pivot to `SVPP-EXPANSION`. The gate is the director's and stays
  the director's; this closes the half of it that is engineering.

#### ✅ CLOSED — the WIRED conjunct is MET, and measuring it REFUTES the gate

- ⭐ **(a) ADJUDICATED — the parsers were never absent, and the population is CLOSED.** All three are
  registered under their **own** grammar names in `GENERATED_PARSER_REGISTRY`
  (`rust/src/parser_registry.rs`) and always had a working `parse_sample`; the single missing thing
  was the `parse_and_cover` field, which read `None`. Their generated artifacts already emit the whole
  G.4.6 coverage API — `enable_coverage` / `exercised_rule_names` / `parse_full_from` each occur
  exactly once in `generated/ebnf.rs`, `generated/return_annotation_parser.rs` and
  `generated/semantic_annotation_parser.rs` — so the leaf's *"no codegen change should be needed"*
  held exactly. ⛔ **The scope of THREE is a closed population, not a guess**:
  `done_bar_family_register_v0.json` `families` holds exactly **10** rows, seven were wired, and the
  `builtin_*` pair is `grammar_dispositions: bootstrap_contract`, **not** a family —
  `builtin_semantic_annotation` has no generated parser at all (its parse path is the hand-written
  `UnifiedSemanticAST::parse_bootstrap`), so there is nothing to verify a witness *through*.
- **(b) WIRED — registry-only.** Three `parse_and_cover_<grammar>` fns modelled verbatim on
  `parse_and_cover_json`/`_vhdl` (enable the transactional `coverage_stack`, parse from the requested
  entry or the canonical one, return the parser's OWN committed-rule record), plus three `None →
  Some(…)` table fields. `parse_and_cover_ebnf` carries the SAME
  `cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))` as the rest of the ebnf dispatch —
  which is also the build `ast_pipeline` needs to read a `.ebnf` directly, so the hook and the lane
  that consumes it are available together or not at all. **ZERO grammar bytes, ZERO codegen bytes,
  ZERO generated bytes.**
- **(c) THE TUPLES — recorded, deterministic at seeds 0/7/42** (`--count 40`, canonical entry,
  0.06–0.38 s each — these are CHEAP lanes):

  | grammar | entry | total | proof | witness | **UNKNOWN** | `fully_certified` | `spf` @ 0/7/42 |
  |---|---|---|---|---|---|---|---|
  | `ebnf` | `grammar_file` | 144 | 0 | 109 | **35** | `false` | 13 / 13 / 11 |
  | `return_annotation` | `return_annotation` | 35 | 0 | 33 | **2** | `false` | 0 / 0 / 0 |
  | `semantic_annotation` | `semantic_annotation` | 114 | 0 | 80 | **34** | `false` | 2 / 4 / 3 |

  ⭐ **The CLASSIFICATION is seed-invariant and the `spf` counter is NOT, and that distinction is
  load-bearing.** `total/proof/witness/UNKNOWN/fully_certified` are byte-identical at 0/7/42 for all
  three, and the **`UNKNOWN` SETS** are identical too (sha of the sorted set: `ebnf` `30ef303b4da80127`,
  `return_annotation` `272f9ce1e29dbfff`, `semantic_annotation` `83801f2730094f11`, each ×3 seeds).
  `sample_parse_failures` moves with the seed because it counts *generated samples the real parser
  rejects* and the sample set is seed-derived — it is a finding (below), not drift.
- ⛔⛔ **(d) THE ADJUDICATION — THE `SVPP-EXPANSION` ACTIVATION GATE IS *NOT* MET, AND IT NEVER WAS.**
  The director's gate is *every existing parser cert-coverage **WIRED** + **clean** + `UNKNOWN`=0*.
  **WIRED is now MET: 10 of 10 register families are measurable.** **`UNKNOWN`=0 is NOT met** — the
  three families that could not be measured were between them hiding **71 `UNKNOWN` rules**
  (35 + 2 + 34). SV reaching `UNKNOWN=0` on 2026-08-22 removed the last *known* blocker; it did not
  make the gate met, because three families were outside the instrument entirely. ⭐ **This is the
  whole point of the leaf**: (b) was one afternoon, (d) is the answer, and the answer is *no*.
  ⚠️ Most of the residual is *dead-rule candidates*, not reach gaps — the pass reports **NO reach
  path from the entry** for 31 of `ebnf`'s 35, 31 of `semantic_annotation`'s 34, and 2 of 2 for
  `return_annotation` (**64 of 71**). Those need linter adjudication and may be legitimately dead or
  entry-relative, but *today they are `UNKNOWN`*, so `UNKNOWN=0` is false. → routed to `H.16`.
- ⛔ **THREE FINDINGS ROUTED OUT, none worked here** (repo policy §15 — found is step 1, fixed is the
  goal): **`H.16`** (roll the three to `UNKNOWN=0`; 71 residual, 64 of them no-reach-path);
  **`H.17`** (`spf>0`: the stimuli generator emits samples the family's OWN parser rejects — 11–13 of
  40 on `ebnf`, 2–4 of 40 on `semantic_annotation`, while all seven previously-wired families read
  `spf=0`); **`H.18`** (the cert-failure LABEL is blind, and its message is FALSE — measured
  `LABEL-BLIND-CENSUS: registry_rows=13 parse_detail_none_with_working_arm=9`).
- ⚠️ **LEG 3 OF THE CLAIM-VERIFICATION BAR IS ABSENT, AND IS NAMED RATHER THAN GLOSSED**
  (`docs/CLAIM_VERIFICATION.md`; director standing directive 2026-08-15). Leg 1 (re-derive) and leg 2
  (falsify against a code-disjoint oracle, with a control proven able to go RED) are discharged below.
  **Leg 3 — DURABILITY — is not**: nothing in the repo asserts that every `done_bar_family_register`
  family is cert-coverage WIRED, so the claim *"10/10 wired"* is true today and unwatched tomorrow.
  → `H.19` owns the gate. Until it lands, this claim is published **qualified**.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `./rust/target/debug/ast_pipeline grammars/<g>.ebnf
  --report-certificate-coverage --count 40 --seed 0` refused on each of the three, by name:
  `Error: certificate-coverage: no generated parser is registered for grammar 'ebnf' — cannot verify
  reachability witnesses through a real parser (Phase H wires more grammars)` (identically for
  `return_annotation` and `semantic_annotation`) ⇒ cert-coverage was **unmeasurable**, not merely poor.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `run_certificate_coverage_report` (`rust/src/main.rs:3801`)
  bails unless `parser_registry::supports_parse_and_cover(&grammar.grammar_name)`, which reads the
  registry table's `parse_and_cover` field; WHERE: `GENERATED_PARSER_REGISTRY` in
  `rust/src/parser_registry.rs` carried `parse_and_cover: None` for exactly these three rows while
  their `parse_sample` worked. ⛔ NOT a codegen or parser gap — the generated artifacts already export
  the G.4.6 API (`enable_coverage`/`exercised_rule_names`/`parse_full_from` present in all three), so
  the defect was one unset struct field per family. Confirmed by the post-fix
  `--report-certificate-coverage` headline appearing for all three
  (`CERTIFICATE-COVERAGE: grammar='ebnf' entry='grammar_file' … UNKNOWN=35 fully_certified=false`).
- [x] **FIX** — fix-hierarchy tier **declarative** (the highest tier: a data-driven registry table
  field, no grammar edit, no engine behaviour change). Three `parse_and_cover_<grammar>` fns modelled
  verbatim on `parse_and_cover_json`/`_vhdl` + three `None → Some(…)` fields. Why no lower tier: the
  parsers, the coverage instrumentation and the entry fns all already existed; nothing below the
  registry was missing.
- [x] **ADDRESSED (verified)** — before→after on the symptom, per family: **REFUSED → MEASURED**.
  `ebnf` `total=144 proof=0 witness=109 UNKNOWN=35 fully_certified=false`; `return_annotation`
  `total=35 proof=0 witness=33 UNKNOWN=2 fully_certified=false`; `semantic_annotation`
  `total=114 proof=0 witness=80 UNKNOWN=34 fully_certified=false`. WIRED census `10/10`, re-derived
  two code-disjoint ways (running `--report-certificate-coverage` on all ten vs a static parse of the
  registry table cross-joined with the register's `families` map) — both read
  `CERT-WIRED-CENSUS: register_families=10 wired=10 unwired=0 missing=[]`. ⭐ The second arm was made
  a TRACKED, re-runnable reader by `H.18` (`-0157`):
  [`docs/tasks/artifacts/grammar_wellformed/cert_wiring_census/probe.py`](artifacts/grammar_wellformed/cert_wiring_census/probe.py),
  with both of its arms proven able to go RED.
- [x] **NO REGRESSION** — the change is additive-by-name, and that was **measured, not asserted**. All
  seven previously-wired families reproduce their pinned tuples at seed 0 with `spf=0`: `json`
  `9/0/9/0`, `regex` `269/9/260/0`, `vhdl` `225/0/225/0`, `systemverilog_preprocessor` `74/0/74/0`,
  `rtl_frontend` `169/1/168/0`, `rtl_const_expr` `48/0/48/0` (its contract's canonical lane —
  `generated/rtl_const_expr.json`, `--max-depth 32`, generation knobs stripped), `systemverilog`
  `1385/18/1367/0` — **all seven `fully_certified=true`**, matching the pinned canonical values.
  The three new lanes are byte-identical at **seeds 0/7/42** (tuple AND `UNKNOWN` set). `clippy`
  flow green (`clippy_source_all_targets` ok, `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS —
  68 pinned lints, all still in clippy::correctness`, generated stage pass). ZERO grammar / codegen /
  generated bytes ⇒ no parser artifact moved.
  `cargo test --lib --features "generated_parsers ebnf_dual_run"`: **1114 passed / 1 failed / 28
  ignored**. ⛔ **NOT written as green.** The one failure is
  `ast_pipeline::ast_based_generator::semantic_usage_tests::unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`
  (*"expected semantic_annotation fallback to detect '@' directives"*), and it is **PRE-EXISTING,
  ALREADY OWNED and ORTHOGONAL** — `CI-PARITY-GATE-ROT.21` class (2), root-caused there to
  `LANG-CAPABILITY-AUDIT.10.3` (`cc0cbffe`) deleting the `b'@'` special case while the test still
  demands it. ⭐ Re-confirmed HERE by a two-arm control rather than taken on trust: `git stash push --
  rust/src/parser_registry.rs` → the same test fails identically at HEAD (0.58 s), stash popped ⇒ this
  change is exonerated by measurement, not by argument.
  ⚠️ **CONTROL, proven able to go RED**: `builtin_return_annotation` / `builtin_semantic_annotation`
  are in the registry table with `parse_and_cover: None` and are **still refused by name** after the
  fix — so the predicate is live and this was not a blanket enable. ⛔ **My first control was
  MIS-DESIGNED and is recorded rather than quietly dropped**: `*_lrm_extracted` was expected to refuse
  and did not, because it never reaches the predicate — it dies earlier in the frontend
  (`Error: unterminated quoted literal in expression ''0 | '1 | 'z_or_x 48 ;'`). A control that fails
  for the wrong reason proves nothing.
- [x] **LOCKSTEP** — `MEMORY.md` resume pointer, `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  `docs/TASK_TREE.md` frontier, and the book's certificate-coverage surface updated in the same
  commit. `done_bar_family_register_v0.json` **deliberately unchanged**: no family's closure legs
  moved — three families became *measurable*, and measurable-but-`UNKNOWN>0` is not a status change.


## ROUTING EVIDENCE

**`H.15` → `H.16` / `H.17` / `H.18` / `H.19` (2026-08-22, `PGEN-GRAMMAR-WELLFORMED-0156`).** All four
land inside `GRAMMAR-WELLFORMED`, which is where cert-coverage wiring and its residuals already live,
so the routing question that matters here is not *which tree* but **whether each finding is really the
three-families finding it was discovered next to, or something wider that would be mis-sized by being
filed under them.** That was measured, not assumed, and for two of the four the answer changed the
routing.

1. **Does the finding reproduce OUTSIDE the family it is routed to?**
   - **`H.18` (label blindness) — YES, and this is why it is NOT a sub-item of `H.15`.** The
     symptom was discovered on `ebnf`/`semantic_annotation`, so the plausible reading is *"the three
     newly-wired families need a `parse_detail` too."* Measuring the closed registry population
     refutes that: `LABEL-BLIND-CENSUS: registry_rows=13 parse_detail_none_with_working_arm=9` — the
     nine include `json`, `vhdl`, `rtl_const_expr`, `rtl_frontend` and the `builtin_*` pair, i.e. **six
     families that have nothing to do with this leaf**, four of them fully-certified and shipped. Only
     `regex`, `systemverilog`, `systemverilog_preprocessor` and `scratch` are labelled. ⇒ it is an
     engine-universal registry defect that this leaf merely *exposed*, and filing it under the three
     grammars would have under-sized it by two thirds.
   - **`H.19` (nothing WATCHES `10/10 wired`) — YES, by construction.** It is a property of the
     register↔registry join over **all ten** families, not of any one of them. Filing it under `H.16`
     would have fused a WIRED assertion with an `UNKNOWN=0` assertion, and the fused gate would be red
     for reasons the doctrine does not own — which is the failure mode `WAIVER-ROUTING` exists to stop.
   - **`H.16` (71 `UNKNOWN`) — NO, and that was checked.** All seven previously-wired families were
     re-run at seed 0 in the same session and every one reads `UNKNOWN=0 spf=0 fully_certified=true`
     (`systemverilog` `1385/18/1367/0`, `regex` `269/9/260/0`, `vhdl` `225/0/225/0`, `svpp` `74/0/74/0`,
     `rtl_frontend` `169/1/168/0`, `rtl_const_expr` `48/0/48/0`, `json` `9/0/9/0`). The residual is
     genuinely confined to the three families this leaf made measurable.
   - **`H.17` (`spf>0`) — NO for the seven, and that is the whole signal.** `ebnf` reports 11–13 of 40
     and `semantic_annotation` 2–4 of 40, while **all seven previously-wired families report `spf=0`**.
     A defect that fires on two families and not on seven is a property of those two grammars or of how
     the generator reads them — so it stays here rather than being routed to the generator tree on the
     plausible reading that *"the stimuli generator is wrong"*. ⚠️ That plausible reading may still turn
     out to be right; what is measured today is only that it does not fire elsewhere, and `H.17` says so.

2. **What was MEASURED to place each one, as opposed to what makes it plausible?**
   The cert headline + `UNKNOWN` list for all ten families at seed 0 (and 0/7/42 for the three new
   ones); the `WARNING … NO reach path from the entry` counts that split `H.16`'s 71 into **64
   dead-rule candidates and 7 genuine reach targets**; a static parse of `GENERATED_PARSER_REGISTRY`
   cross-joined with `done_bar_family_register_v0.json`'s `families` (a second, code-disjoint
   instrument) for the WIRED and LABEL-BLIND censuses.

3. **What would have to be true for the routing to be WRONG, and was it checked?**
   - `H.18` would belong here if the nine blind rows were only the three newly-wired ones. **Checked —
     they are not; six others are affected.**
   - `H.16` would belong in a shared/engine tree if the residual also appeared in the wired families.
     **Checked — all seven read `UNKNOWN=0`.**
   - `H.17` would belong in a stimuli-generator tree if `spf>0` were general. **Checked — it is not,
     seven families read `spf=0`.** ⛔ **Honest bound**: `spf` is seed-dependent (`ebnf` 13/13/11,
     `semantic_annotation` 2/4/3), so "seven read `spf=0`" is measured at **seed 0 only** for those
     seven. A multi-seed sweep of the wired families has **not** been run, and `H.17` must run it
     before concluding the two families are special rather than merely unlucky at one seed.
   - `H.19` would be redundant if any existing doctrine already asserted the register↔registry join.
     **Checked against the 25-doctrine registry — none does**; the nearest neighbours
     (`PARSER-BOOK-CURRENCY`, `PUBLISHED-VERSION-CURRENCY`, `BASELINE-IDENTITY`) all read published
     text or tracked baselines, never the registry table's `parse_and_cover` field.

**`H.17` → `H.17.1` / `H.17.2` (2026-08-22, `-0158`).** Both stay in this tree because
certificate-coverage residuals are its charter, but the split itself was the routing decision and it
was measured. **(1) Does it reproduce outside the family it was found in? YES, and that is why the
lint half is separate.** The defect was found on `ebnf`, but the live census reads **6 uses in 3
grammars** — `ebnf.ebnf` ×3, `semantic_annotation.ebnf` ×2, `systemverilog_lrm_profiled_generated.ebnf`
×1 — and `grammars/systemverilog.ebnf:519`/`:601` record SystemVerilog having been fixed for the SAME
construct in 2026 (`SV-EXH-PROOF.3.3.4.b.6.2.15`). A defect with four families in its history and a
documented prior fix is a **class**, not an `ebnf` bug. **(2) What was measured, not what is
plausible:** a 7-arm feature bisection with 4 passing controls isolating look-around as the sole
failing construct; identical verdicts AND identical `furthest_position` values from two independent
engines; the `--lint-grammar` blindness count (**0**); the default-verbosity report count (**0**).
**(3) What would make the routing wrong, and was it checked?** ⭐ It would be wrong if `regex.ebnf`'s
25 look-around tokens were uses — that would make this a regex-family problem and a five-fold larger
population. **Checked: they are double-quoted string LITERALS** (`"(?="`, `"(?!"`, …) by which the
regex grammar *matches PCRE lookaround syntax*; the `:=`-with-`/…/` shape separates describing a
construct from using one. It would also be wrong if the interpreter were the only witness — TOOLBOX
1.5b records a measured interpreter/generated-parser divergence on un-eliminated left recursion — so
`--lint-grammar` was run first (`left_recursion_unhandled=0`, so the hole does not apply) **and** the
generated-parser arm was built and run anyway.

**`promotion: declined`** for the `-0159` `DEVELOPMENT_NOTES.md` entry's per-slice history (§§3–6 —
the free second instrument, the red-at-HEAD exposure check, the mis-measured control, the still-open
`DIAG-SEVERITY` half: all recorded in `H.17.2`). ⭐ §§1–2 are **PROMOTED** —
[`check-the-property-not-the-spelling-of-the-property.md`](../knowledge/check-the-property-not-the-spelling-of-the-property.md)
— because *"a defect found through one instance tempts you to check the instance's spelling; check the
property, and check it the way the runtime does"* is durable, general to any validation, re-verifiable
by the probe's arm 4, and it is what kept this lint from shipping both unsound and incomplete.

**`promotion: declined`** for the `-0158` `DEVELOPMENT_NOTES.md` entry's per-slice history (the census
mechanics and the probe's control paths — specific to this slice, already in the leaf). ⭐ Its core is
**PROMOTED** —
[`a-permanent-defect-reported-on-the-speculative-failure-channel-is-invisible.md`](../knowledge/a-permanent-defect-reported-on-the-speculative-failure-channel-is-invisible.md)
— because *"an unconditional authoring error routed through a channel designed to be swallowed is
indistinguishable from routine backtracking"* is durable, general, re-verifiable, and is the reason
this survived a documented prior fix in a sibling family.

**`promotion: declined`** for the `-0157` `DEVELOPMENT_NOTES.md` entry's per-slice history (§§3–5 —
the label-change no-regression arm, the orphan separation, the immediate `/**/` payoff: all true, all
specific to this slice, already in the `H.18` leaf). ⭐ §§1–2 are **PROMOTED** —
[`two-tables-answering-one-question-drift-and-the-one-that-prints-knows-less.md`](../knowledge/two-tables-answering-one-question-drift-and-the-one-that-prints-knows-less.md)
— because *"count the duplicate's readers before choosing between filling it in and deleting it"* is
durable, general, re-verifiable by the tracked census, and it is what inverted the obvious fix here.

**`promotion: declined`** for the `-0156` `DEVELOPMENT_NOTES.md` entry's per-slice history (§§3–5, 7 —
the population adjudication, the seed/`spf` split, the mis-designed control, the leg-3 note: all true
and all specific to this slice, already recorded in the leaf). ⭐ §2 is **PROMOTED** —
[`a-refusal-message-that-names-a-cause-instead-of-its-condition-oversizes-the-gap.md`](../knowledge/a-refusal-message-that-names-a-cause-instead-of-its-condition-oversizes-the-gap.md)
— because it is durable, general (any guard whose message narrates a guessed cause), re-verifiable by
one command, and it is the reason this gap read as a missing subsystem for two months.

### `H.16` — **THE THREE NEWLY-MEASURABLE FAMILIES CARRY 71 `UNKNOWN` BETWEEN THEM** (**`in_progress`** — re-priced to **68**; the whole residual is ADJUDICATED by `H.16.1` (`PGEN-GRAMMAR-WELLFORMED-0164`, 2026-08-22 session #257) and the work is routed to `H.16.2`–`H.16.5`; opened 2026-08-22 session #255 by `H.15`)

- **WHY**: `H.15` wired cert-coverage for `ebnf` / `return_annotation` / `semantic_annotation` and the
  first measurement is `UNKNOWN` **35 / 2 / 34**. The director's `SVPP-EXPANSION` activation gate
  needs `UNKNOWN=0` on *every existing parser*; these three are what now stands in the way.
- ⛔⛔ **AND `UNKNOWN=0` IS NOT A COMPLETENESS CLAIM ABOUT THE LANGUAGE — MEASURED BY `H.16.6`
  (`-0168`), and it QUALIFIES THE GATE THIS LEAF SERVES.** Certificate coverage answers *"can this
  rule fire at all?"*, not *"does the parser accept everything the grammar licenses?"*.
  `semantic_annotation`'s `map_entry` is **WITNESSED** — a string-keyed sample reaches it and commits,
  so it has never been `UNKNOWN` and never will be — and `@type: {1 => 2}` is **REJECTED**, because
  `=>` is both the map arrow and `implication_expr`'s operator. ⇒ a family can reach `UNKNOWN=0` and
  still be wrong about its own language, and **no amount of witness work moves a defect of that
  class**. ⚠️ This is a fact about the gate's METRIC, not an argument against the gate: the
  `UNKNOWN=0` conjunct stays exactly as binding as the director set it. It is recorded here, at the
  gate's home, so nobody reads a future `UNKNOWN=0` as *"this family is done"*. → `H.16.6` / `H.16.6a`.
- **THE SHAPE OF THE RESIDUAL — 64 of 71 are dead-rule candidates, not reach gaps.** The pass reports
  `WARNING plannable-rule reach pass: N UNKNOWN rules have NO reach path from the entry (dead-rule
  candidates — adjudicate via the linter)` for 31 of `ebnf`'s 35, 31 of `semantic_annotation`'s 34,
  and 2 of 2 for `return_annotation` (`["accessor_base", "parenthesized"]`). ⇒ this is mostly a
  **`--lint-grammar` adjudication** job (are they genuinely dead, entry-relative, or a real reach
  gap?), not a witness-generation job — which is the opposite of the SV lane's shape and should be
  priced that way. The remaining 7 (e.g. `ebnf`'s `epsilon`, `whitespace`, `block_comment`;
  `semantic_annotation`'s `multiline_string`, `set_value`, `set_element`) DO have a reach path and are
  the genuine witness work.
- ⛔⛔ **RE-PRICED 2026-08-22 BY `H.17`'s DIAGNOSIS — PART OF THE RESIDUAL IS NEITHER A REACH GAP NOR
  A DEAD RULE.** Regex look-around never compiles, so five live rules match nothing on any input, and
  every one of them (and their parents) is already in these `UNKNOWN` lists: `ebnf` —
  `block_comment`, `block_comment_content`, `whitespace`, `semantic_predicate`, `predicate_content`,
  `action_block`, `action_content`; `semantic_annotation` — `multiline_string`, `block_comment`.
  ⇒ **adjudicate `H.17.1` FIRST and re-measure**, because these rules will change classification
  when their terminals compile. Counting them as dead-rule candidates today would record a defect as
  a design fact.
- ✅ **`H.17.1` LANDED 2026-08-22 (`-0161`) AND THIS LEAF'S INPUT HAS MOVED — RE-PRICE BEFORE
  ADJUDICATING.** The live numbers are now `ebnf` **33** (was 35) and `semantic_annotation` **33** of
  **115** rules (was 34 of 114 — `total` rose by the new `multiline_string_content` helper);
  `return_annotation` is untouched at **2**. ⇒ the headline is **68**, not 71.
- ⭐⭐ **AND THE RE-MEASURE SETTLED THE RE-PRICING QUESTION IN BOTH DIRECTIONS, which is more useful
  than the count.** Of the nine names this leaf was warned about, the split is now MEASURED rather
  than predicted:
  - **Moved to CERTIFIED by the terminal repair (3)** — `ebnf`'s `block_comment` +
    `block_comment_content`, `semantic_annotation`'s `multiline_string`. These were the reachable
    ones; they were never dead and never reach gaps.
  - **Still `UNKNOWN`, and now provably for a DIFFERENT reason (5)** — `ebnf`'s `semantic_predicate`,
    `predicate_content`, `action_block`, `action_content` and `semantic_annotation`'s `block_comment`
    have **compiling terminals and no referrer**. ⛔ They are now honest dead-rule candidates, so this
    leaf may adjudicate them as such — which it explicitly could NOT do before `-0161`, and that is
    exactly what the re-pricing note was protecting against.
  - **Unrelated to `H.17` after all (1)** — `ebnf`'s `whitespace` (`/(\s+)/`) **always compiled**
    (it is absent from the lint's `uncompilable_regex_terminals` census, which named exactly the
    other three) and it **has a reach path** (absent from the 31-name no-reach-path set). ⇒ it is
    neither a dead rule nor a look-around casualty, and belongs to this leaf's genuine witness work.
    ⚠️ *Hypothesis, NOT measured*: it may be UNKNOWN because the layout skipper consumes whitespace
    before the rule can fire. Probe it (`--trace-rules whitespace`) before adjudicating — do not
    inherit this sentence as a finding.
- ⛔ **DO NOT DELETE A RULE TO REACH `UNKNOWN=0`.** A dead-rule candidate is a verdict to be
  adjudicated and, where the rule is legitimately entry-relative, certified by an entry-union the way
  `GRAMMAR-WELLFORMED.H.12.8.5` did for SV — not a licence to shrink the grammar until the number
  looks right.
- ⛔⛔ **AND THE ADJUDICATION TOOL THIS LEAF NAMES CANNOT DO IT — SETTLED BY `H.16.1`.**
  `--lint-grammar` reads `unreachable_rules=0` on all three families BY CONSTRUCTION
  (`detect_unreachable_rules` roots at the entry PLUS every unreferenced rule), so it is
  structurally blind to exactly the 64. The missing reading is
  `docs/tasks/artifacts/grammar_wellformed/residual_island_census/probe.py`, and it splits the 64
  into **9 of PGEN's own LR-elimination residue + 55 source orphans** across **31 islands**.
- **DETERMINISM IS ALREADY ESTABLISHED**: tuple and `UNKNOWN` set byte-identical at seeds 0/7/42
  (`H.15`), and each lane runs in 0.06–0.38 s — so this is a cheap, fast-iterating lane.

### `H.16.1` — **THE 68 ARE ADJUDICATED: 31 ISLANDS, 9 OF PGEN'S OWN LR RESIDUE, 55 SOURCE ORPHANS, AND 4 RULES THAT CAN NEVER FIRE — BY FOUR DIFFERENT MECHANISMS** (**`done`**, `PGEN-GRAMMAR-WELLFORMED-0164`, doc+artifact tier — opened AND closed 2026-08-22 session #257 by `H.16`)

- **WHY THIS LEAF EXISTS**: `H.16` is priced as *"mostly a `--lint-grammar` adjudication job"*. ⛔ **The
  linter cannot do that job, and reads a reassuring `unreachable_rules=0` over the whole population
  BY CONSTRUCTION.** `detect_unreachable_rules`
  (`rust/src/ast_pipeline/grammar_wellformedness.rs:386`) roots reachability at the canonical entry
  **PLUS every rule NOTHING references** — its own doc-comment says so (*"an unreferenced dead orphan
  is treated as a root → not flagged"*) — so every one of the 64 no-reach-path rules is a ROOT to the
  linter. Measured, all three at HEAD: `ebnf` / `return_annotation` / `semantic_annotation` each
  `unreachable_rules=0, exit 0`, while the cert pass names 31 / 2 / 31 rules with **no reach path from
  the entry**. The lint is *correct on its own terms* and *structurally blind to exactly this
  population*. ⇒ this leaf builds the missing reading before anything is adjudicated.

#### ✅ CLOSED — the residual is partitioned, closed, and every class has a named owner

- ⛔⛔ **CORRECTION (`-0170`, same session) — THE ROOT-CAUSE HALF OF THIS LEAF IS A RE-DERIVATION, NOT
  A DISCOVERY, AND THIS LEAF ORIGINALLY DID NOT SAY SO.** `LANG-CAPABILITY-AUDIT.2` is **`done`**
  (`PGEN-LANG-CAPABILITY-AUDIT-0002`, session #208) and had already resolved exactly this, quoting the
  SAME source: *"RESOLVED — the linter is NOT wrong, and neither is the closure"*, citing
  `grammar_wellformedness.rs`'s root-set doc-comment verbatim. ⇒ *"a metric's root set is part of its
  meaning"* was established here months before this session re-found it from scratch. **The leaf below
  is left standing as written, with this correction above it** (supersede, don't mutate —
  `MEMORY_ARCHITECTURE.md` §10), because the reasoning is sound; only the novelty was overstated.
- ⭐ **WHAT IS GENUINELY NEW, stated narrowly**: (a) the **LR-residue vs source-orphan split** (9 vs
  55) — no prior leaf computes it, and reading POST alone records 9 engine artifacts as grammar facts;
  (b) the **island partition** (31 islands under named orphan roots) and the re-runnable instrument;
  (c) an **exact cross-method corroboration** — `LANG-CAPABILITY-AUDIT.1` derived **27** unreachable
  `ebnf` productions by a hand closure over `--dump-gen-ast` in session #208, and this census derives
  `source_orphans=` **27** by a PRE/POST two-arm diff in session #257, from different code and a
  different dump. Two independent methods, same number.
- ⛔ **AND THE POPULATION WAS ALREADY OWNED**: the 55 source orphans belong to
  `LANG-CAPABILITY-AUDIT.1`/`.4`/`.6`, which cluster them into 7 horizon-mapped families and already
  carry per-cluster dispositions. `H.16.5` is re-scoped accordingly — see its RETRACTION.

- ⭐ **RE-DERIVED HEADLINE (leg 1), and it confirms `H.16`'s re-price: 68, not 71.**
  `PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/<g>.ebnf
  --report-certificate-coverage --entry-rule <entry> --count 40 --seed 0`:

  | grammar | entry | total | proof | witness | **UNKNOWN** | `spf` |
  |---|---|---|---|---|---|---|
  | `ebnf` | `grammar_file` | 144 | 0 | 111 | **33** | 8 |
  | `return_annotation` | `return_annotation` | 35 | 0 | 33 | **2** | 0 |
  | `semantic_annotation` | `semantic_annotation` | 115 | 0 | 82 | **33** | 2 |

- **THE MACHINE ADJUDICATION (`PGEN_CERT_RESIDUAL_CLASSIFICATION=1`, TOOLBOX 4.6) — and it CLOSES
  EXACTLY**, `profile='<none>' store_analysis=active` on all three:

  | grammar | `profile_entry_unreachable` | `store_unproducible` | `genuine` |
  |---|---|---|---|
  | `ebnf` | 31 | 0 | 2 — `epsilon`, `whitespace` |
  | `return_annotation` | 2 | 0 | 0 |
  | `semantic_annotation` | 31 | 0 | 2 — `set_value`, `set_element` |

  **64 + 4 = 68.** ⭐ `store_unproducible = 0` everywhere: none of this residual is store-gated, which
  is the opposite of the SV lane's shape and is why the SV playbook does not transfer.

- ⭐⭐ **THE READING THAT WAS MISSING, AND IT SPLITS THE 64 IN TWO — `docs/tasks/artifacts/grammar_wellformed/residual_island_census/probe.py`.**
  Two authoritative dumps of the same grammar: **PRE** = `--emit-raw-ast-json` (the grammar AS
  WRITTEN) and **POST** = `--dump-gen-ast` (what codegen and the cert pass consume, i.e. AFTER
  LR-elimination rewrites referrers). A rule outside the entry closure in POST but INSIDE it in PRE is
  **PGEN's own LR residue**; outside in BOTH is a **source orphan**.

  | grammar | pre_outside | post_outside | **lr_residue** | **source_orphans** | islands |
  |---|---|---|---|---|---|
  | `ebnf` | 27 | 31 | **4** — `expression_return`, `arithmetic_return`, `conditional_return`, `member_access_return` | 27 | 14 |
  | `return_annotation` | 1 | 2 | **1** — `accessor_base` | 1 — `parenthesized` | 2 |
  | `semantic_annotation` | 27 | 31 | **4** — `union_type`, `intersection_type`, `array_type`, `optional_type` | 27 | 15 |
  | **total** | **55** | **64** | **9** | **55** | **31** |

  ⛔ **READING POST ALONE RECORDS AN ENGINE ARTIFACT AS A GRAMMAR FACT.** All 9 ARE wired by their
  grammar; each is orphaned because the LR pass replaced its referrer with `<base>_lr_base` /
  `_lr_suffix` / `_lr_seed_*` helpers and left the original behind — the lint confirms the base rule
  by name (`left_recursion_eliminated=1`: `ebnf` `return_expression` (indirect),
  `return_annotation` `accessor_base`, `semantic_annotation` `type_reference`). This is the SAME class
  `-0271` adjudicated for `verilog_2005` (*"19 are PGEN's own LR residue"*), reproduced in three more
  grammars — ⇒ it is an ENGINE-WIDE accounting property, not an SV curiosity.
- **THE PARTITION IS CLOSED**: every one of the 64 is reached from exactly one unreferenced ROOT — the
  census's `⛔ … the partition is NOT closed` arm printed nothing on any of the three. 31 islands, and
  the largest is 9 rules (`semantic_annotation`'s `performance_value`).
- ⭐ **AND THE 55 SOURCE ORPHANS HAVE A SHAPE, NOT JUST A COUNT.** `grammars/ebnf.ebnf` labels 13 of
  its 14 island roots **`(extension)`** in its own comments — `exception_rule` (`rule except …`),
  `case_control` (`~i"…"`), `named_capture`, `rule_modifier`, `semantic_predicate` (`{? … ?}`),
  `action_block`, `parametric_rule` (`rule[p]`), `template_instantiation` (`rule<T>`), `lexer_mode`,
  `grammar_inheritance` (`grammar X extends Y`), `import_statement`, `optimization_hint`,
  `error_production` — i.e. **the meta-grammar DOCUMENTS a syntax whose own top-level `grammar_file`
  never admits it**. `semantic_annotation`'s 27 are the same shape (6 typed value-spec islands plus 4
  layout rules plus `annotation`). ⇒ this is **declared-but-unwired language surface**, and the
  question *"wire it, or record it declared-dead?"* is a scope call, not a cert-coverage chore →
  `H.16.5`.

#### ⛔⛔ THE 4 `genuine` RESIDUALS ARE FOUR DIFFERENT WAYS A RULE CAN NEVER FIRE — and three are NEW defect classes

`H.17` found one mechanism (an uncompilable regex terminal). Probing these four found **three more**,
each proven with an ACCEPTING control:

| grammar | rule | mechanism | tool evidence |
|---|---|---|---|
| `ebnf` | `epsilon` | **generator name-shadowed builtin** | `[plannable-probe] rule='epsilon' parsed=false sample="ZC:="` ×8. `rust/src/ast_pipeline/stimuli_generator.rs:11186` — `generate_rule` returns `Ok(String::new())` for **any rule literally named `epsilon`**, *before* looking it up, shadowing `grammars/ebnf.ebnf:324`'s real body `("ε"\|"epsilon"\|"empty"\|"λ")`. ⇒ every witness renders as the empty string. **The LANGUAGE is fine**: `--interpret-parse` `X := ε` → `accepted=true`; the generator's own sample `X :=` → `accepted=false`. → `H.16.3` |
| `ebnf` | `whitespace` | **layout skipper eats the bytes before the rule is offered them** | `[plannable-probe] parsed=true witnessed_target=false sample="    "`. MEASURED, not inferred: `--interpret-parse` on 4 spaces → `accepted=true furthest_position=0`, typed AST `{"elements": [], "type": "grammar_file"}` **span 0..0** — `grammar_file`'s `*` matched **ZERO** iterations. Control `"  \nX := \"a\"\n"` → `elements` holds only the `grammar_rule`, no `whitespace` node. ⇒ structurally referenced, operationally unreachable. → `H.16.4` |
| `semantic_annotation` | `set_value` | **a terminal whose PREFIX is a comment introducer, defeating BOTH guards** | see below → `H.16.2` |
| `semantic_annotation` | `set_element` | same — only reachable through `set_value` | same |

- ⭐⭐ **`set_value` IS THE SHARPEST OF THE FOUR AND IT IS ENGINE-UNIVERSAL.** `set_value := "#{" …
  "}"`. Reproducer + ACCEPTING control, on **two code-disjoint oracles**:

  ```text
  parseability_probe --parse semantic_annotation  '@type: #{"a", "b"}'  -> REJECT Backtrack at position 7
  parseability_probe --parse semantic_annotation  '@type: {"a": 1}'     -> PASS      (the object control)
  ast_pipeline --interpret-parse (reads the .ebnf) '@type: #{"a", "b"}' -> accepted=false furthest_position=7
  ast_pipeline --interpret-parse (reads the .ebnf) '@type: {"a": 1}'    -> accepted=true
  ```

  `PGEN_TRACE_VERBOSITY=debug --trace-rules` names the mechanism as a **position jump**:

  ```text
  🚪 Entering branch 4/5 for rule 'structured_value' at position 7   <- set_value, at the '#'
  💾 Memo miss for rule 39 at position 7 - computing fresh result
  🔤 Attempting to match terminal '#{' at position 18 (end: 20)      <- but its FIRST terminal at EOF
  ❌ Terminal '#{' failed at position 18 - found '<EOF>'
  ```

- **ROOT CAUSE (WHY + WHERE) — the same spelling-vs-property error, in BOTH guards, twice:**
  1. **DYNAMIC** — `rust/src/ast_pipeline/ast_based_generator.rs:7199` emits
     `let allow_comment_skip = expected != "#" && expected != "//" && …` — an **exact-equality
     allowlist**. Its own comment states the right intent (*"avoid swallowing comment-introducer
     tokens themselves"*); `"#{"` is not equal to `"#"`, so skipping stays enabled and the `#` arm
     eats `#`→EOL.
  2. **STATIC** — `grammar_claims_introducer_as_non_comment` (`:6556`) SHOULD suppress the `#` arm
     entirely for this grammar, and does not. In `node_has_non_comment_claim` (`:6586`) a literal
     claims the introducer only when no **unbounded content** follows it; `node_is_unbounded_content`
     scores a `"regex"` follower purely by `hir_has_unbounded_repetition`. `set_value`'s follower is
     the **whitespace separator `/\s*/`**, which has unbounded repetition — so `"#{" /\s*/ …` is
     misread as *"an introducer literal followed by a comment tail"* and the claim is dropped.
     ⛔ `\s*` is **layout, not content**: it cannot swallow a `}` and cannot run to end-of-line.
  ⭐ This is precisely the defect shape `H.17.2` was built to outlaw — *"checks does it COMPILE, not
  does it contain `(?` — a spelling heuristic is unsound AND incomplete"* — reappearing one layer down.
  Both copies must move together: `rust/src/parse_harness_interpreter.rs:3045` carries the same
  allowlist (it mirrors codegen byte-for-byte per `PARSE-HARNESS.5.2`), which is why both oracles agree.
- **TWO-ARM CONTROL, and it proves the designed mechanism WORKS — this grammar is the miss, not the
  design.** Emitted `#`-comment arm, per shipped parser: `systemverilog` **0** (claims `#` via `##` /
  delays ⇒ suppressed), `vhdl` **0**, `rtl_frontend` **0**, `regex` **0** — versus
  `semantic_annotation` **2**, `ebnf` **2**, `json` **2**, `return_annotation` **2**.
- **CLASS SIZE (census over every buildable grammar's raw AST — terminals that START with a comment
  introducer but are not equal to it): 11 sites in 6 grammars.**
  `ebnf` `documentation_comment` `/**`,`///` · `regex` `callout_hash_payload` `##` ·
  `semantic_annotation` `set_value` `#{` + `doc_comment` `///` · `systemverilog` `kw_token_93ac8946`
  `##` · the two `systemverilog_lrm_profiled_*` `##`,`##[*]`,`##[+]`.
  ⭐ **Exactly ONE is inert today**: `semantic_annotation`'s `#{`. `systemverilog` and `regex` emit no
  `#` arm; `ebnf`'s `documentation_comment` is **witnessed** (absent from its 33-name `UNKNOWN` set),
  so it fires. ⚠️ **BOUND, stated rather than glossed**: the other 10 were cleared by arm-suppression
  and by witness status, **not** by an individual accept/reject probe — `H.16.2` owes that.
- ⭐ **`semantic_annotation`'s `spf` IS 100 % ATTRIBUTED to this defect**: both failing samples at seed
  0 carry `#{` (`@ CBJxM :    #{        }` and `@  proved: { … #{   }}`). ⚠️ `ebnf`'s `spf=8` is **NOT
  yet attributed** and is not claimed to be — `H.16.3`/`H.16.4` own that.

#### Claim verification (`docs/CLAIM_VERIFICATION.md`) — all three legs, leg 3 NAMED

1. **RE-DERIVE** — every number above is a pasted tool headline, re-run at HEAD this session.
2. **FALSIFY against a code-disjoint oracle, with the control proven able to go RED.** The census is
   Python over two JSON dumps; the oracle is the engine's own Rust `classify_profile_residual`. They
   agree on **31 / 2 / 31 = 64** by different algorithms. The instrument is proven to give **more than
   one reading**: `json` at its real entry → `pre_outside=0 post_outside=0 lr_residue=0 islands=0`; a
   **misnamed** entry → `REFUSED … entry rule 'json_document' is not defined in the raw arm` (rc 2,
   added this slice — without it a typo answers *"every rule is orphaned"*); and a synthetic
   terminating dead CYCLE (`dead1 := ("b" dead2 | "z")`, `dead2 := "c" dead1`) fires the
   `⛔ … the partition is NOT closed` arm — on which the **linter reads `unreachable_rules=2`**.
   ⭐⭐ **THAT LAST PAIR IS THE TRANSFERABLE PART: the linter and this census are COMPLEMENTS.** The
   linter catches dead islands with **no** orphan root; the census catches everything reachable **only
   from** an orphan root. Neither is wrong; the 64 live in the seam, and that is why `H.16`'s
   *"adjudicate via the linter"* could never have worked.
3. **DURABILITY — NOT DISCHARGED, and named.** Nothing re-runs this census, so the split rots the
   moment a grammar gains a rule. It is deliberately NOT folded into `H.19` (which owns *WIRED*): →
   `H.16.5` carries the watch alongside the wiring ruling.

#### Routed out — every finding OWNED, none merely reported (repo policy §15)

- **`H.16.2`** — the `#`-introducer defect (engine-universal, codegen + interpreter). 2 rules inert.
- **`H.16.3`** — the `epsilon` name-shadowed generator builtin (engine-universal). 1 rule inert.
- **`H.16.4`** — `ebnf`'s `whitespace`: layout-skipped before `grammar_file` sees it.
- **`H.16.5`** — the 55 source orphans + the 9 LR residue: PROOF-promotion versus wiring, and the watch.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/ast_pipeline
  grammars/ebnf.ebnf --report-certificate-coverage --entry-rule grammar_file --count 40 --seed 0`
  ⇒ `CERTIFICATE-COVERAGE: … total=144 proof=0 witness=111 UNKNOWN=33 fully_certified=false
  (sample_parse_failures=8, …)`, and identically `2` on `return_annotation` and `33` on
  `semantic_annotation` — **68** unadjudicated residual rules with no owner.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the 64 were unadjudicable: `--lint-grammar` reads
  `unreachable_rules=0` on all three because `detect_unreachable_rules`
  (`grammar_wellformedness.rs:386`) treats every unreferenced rule as a ROOT. WHY the 4 `genuine` can
  never fire, each named and located: `stimuli_generator.rs:11186` (`epsilon` name-shadowed to `""`);
  the layout skipper consuming `grammar_file`'s whitespace (typed AST `elements=[] span 0..0`);
  `ast_based_generator.rs:7199` `allow_comment_skip` exact-equality allowlist **and** `:6586`
  `node_has_non_comment_claim` scoring the `/\s*/` separator as an unbounded comment tail — the
  `--trace-rules` position jump `Entering branch 4/5 … at position 7` →
  `Attempting to match terminal '#{' at position 18` → `found '<EOF>'` names both.
- [x] **ADDRESSED (verified)** — the deliverable of this leaf is the ADJUDICATION, and it is complete
  and closed: **68 = 9 LR-residue + 55 source-orphans + 4 root-caused inert rules**, 31 islands, zero
  rules left unattributed (the census's `NOT closed` arm printed nothing on any of the three), and
  each class routed to a named owning leaf (`H.16.2`–`H.16.5`). Before → after on the OWNERSHIP
  metric: **68 residual rules with no owning leaf → 0**. ⛔ No `UNKNOWN` moved and none is claimed to
  have: this slice changes ZERO grammar bytes, ZERO Rust bytes, ZERO codegen bytes, ZERO generated
  bytes.
- [x] **NO REGRESSION** — doc+artifact tier, so the parser surface is inert BY CONSTRUCTION. Verified
  anyway: `--report-certificate-coverage` re-read on all three at HEAD after the slice is
  byte-identical to the tuples above; `--lint-grammar` `exit 0` on all three with
  `uncompilable_regex_terminals=0`; `bash scripts/check_doctrines.sh` green. The new artifact is a
  read-only reader invoked by nobody — it is not in `DIAGNOSIS_SIG` and is not wired into any gate.
- [x] **LOCKSTEP** — this leaf + the `H.16` leaf's status + the Current Frontier rows in
  `docs/tasks/GRAMMAR-WELLFORMED.md`; `docs/TASK_TREE.md` frontier; `CHANGES.md`;
  `DEVELOPMENT_NOTES.md`; `MEMORY.md`. Book: N/A — no user-facing surface changed; the mechanisms
  become book material when `H.16.2`/`H.16.3` land the fixes.

### `H.16.2` — **A TERMINAL WHOSE PREFIX IS A COMMENT INTRODUCER IS EATEN AS A COMMENT: `set_value` / `set_element` ARE INERT** (**`done`**, `PGEN-GRAMMAR-WELLFORMED-0165`, CODE / engine-universal codegen — opened 2026-08-22 session #257 by `H.16.1`, CLOSED same session)

- **WHY**: fully root-caused in `H.16.1` — `semantic_annotation`'s `set_value := "#{" /\s*/ … "}"` can
  never match on any input, because the engine's `#`-comment layout arm consumes `#`→EOL before the
  rule's own first terminal is attempted. Two rules inert.

#### ✅ CLOSED — the analysis asked *is there an unbounded repetition* when the property is *can it carry text*

- ⭐⭐ **THE STATIC MECHANISM THAT SHOULD HAVE CAUGHT THIS ALREADY EXISTED AND WORKS — this grammar was
  the miss, not the design.** `H.11.5` built per-introducer arm SUPPRESSION: a grammar that assigns
  `#` / `//` / `/*` a NON-COMMENT meaning gets that arm elided entirely. Emitted `#`-arm, per shipped
  parser: `systemverilog` **0**, `vhdl` **0**, `rtl_frontend` **0**, `regex` **0** (each claims `#`)
  versus `semantic_annotation` **2**, `ebnf` **2**, `json` **2**, `return_annotation` **2**.
  `semantic_annotation` belonged in the first row: `"#{"` IS a non-comment claim on `#`, and the
  grammar defines no `#` comment rule anywhere.
- **ROOT CAUSE (WHY + WHERE)** — ONE predicate drops the claim.
  `node_has_non_comment_claim` (`rust/src/ast_pipeline/ast_based_generator.rs:6586`) treats an
  introducer-prefixed literal as *comment-defining* — i.e. not a claim — when an **unbounded content
  terminal** follows it (the shape that makes `("#" | "//") comment_content` a comment rather than two
  tokens). `node_is_unbounded_content` scored a `"regex"` follower purely by
  `hir_has_unbounded_repetition`. `set_value`'s follower is the **whitespace separator `/\s*/`**,
  which has unbounded repetition ⇒ `"#{" /\s*/ …` was read as *"an introducer followed by a comment
  tail"* ⇒ claim dropped ⇒ arm emitted ⇒ `#{` became trivia on every input.
- ⛔⛔ **IT IS THE SAME ERROR `H.17.2` OUTLAWED, ONE LAYER DOWN.** That check earned its design by
  asking *does the pattern COMPILE* rather than *does it contain `(?`* — the property, not the
  spelling. Here the analysis asked *is there an unbounded repetition* when the property that matters
  is *can it carry NON-WHITESPACE text*. ⭐ **A defect class fixed at one layer is worth grepping for
  at the others before it is assumed local.**
- **FIX (tier: engine — no lower tier exists).** The grammar is correct as written; it is the ANALYSIS
  that misreads it, so neither a declarative nor a grammar tier applies. New
  `AstBasedGenerator::hir_matches_only_whitespace` walks the HIR and answers *does every match consist
  solely of whitespace?*; `node_is_unbounded_content`'s regex arm becomes
  `hir_has_unbounded_repetition(&hir) && !hir_matches_only_whitespace(&hir)`. ⭐ Conservative in the
  safe direction BY CONSTRUCTION: anything the walk cannot PROVE whitespace-only answers `false`,
  which leaves every prior verdict exactly where it was. **ZERO grammar bytes.**
- ⭐⭐ **BLAST RADIUS IS A MEASUREMENT, NOT AN ARGUMENT — and the oracle already existed.**
  `parse_harness_equivalence::gate::comment_arm_suppression_matrix_is_pinned` pins the
  `(#, //, /*)` decision for all TEN registered grammars against ground truth read from the shipped
  `generated/*.rs`. It failed with **exactly one row moved**, which is what was predicted before it
  was run:
  ```text
  semantic_annotation: expected (#,//,/*)=(false,true,true) but got (true,true,true)
  ```
  Nine grammars unchanged. The pin is updated to the new truth in the same commit.
- ⭐ **AND THE ANNOTATION-BACKEND CONTROL IS THE ONE THAT MATTERED.** The annotation parsers are what
  codegen LINKS to generate every OTHER parser, so a change to
  `generated/semantic_annotation_parser.rs` could in principle move every artifact in the tree.
  Measured: `generated/json_parser.rs` regenerated through the changed generator AND the changed
  annotation backend is **BYTE-IDENTICAL** (`6088e53d444ed5ca…` both sides), while
  `semantic_annotation_parser.rs` moved (`4def8be380961f0d…` → `e2a3d4cdc663cf1f…`). The control can
  go RED, and did, on exactly the artifact it should. ⚠️ `generated/semantic_annotation.json` also
  re-hashed; proven to be the embedded `generated_at` alone — two consecutive dumps have
  **byte-identical `raw_ast`** and differ only in that field.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `./rust/target/release/parseability_probe --parse semantic_annotation`
  on `@type: #{"a", "b"}` ⇒ `Error: parse_full rejected … Backtrack at position 7
  [furthest_position=7]`, with the ACCEPTING object control `@type: {"a": 1}` ⇒ `parse_full passed`.
  Reproduced identically on the code-disjoint `--interpret-parse` oracle (`accepted=false
  furthest_position=7` vs `accepted=true`), so it is not a stale-binary artifact.
- [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_TRACE_VERBOSITY=debug --trace-rules` names it as a POSITION
  JUMP: `🚪 Entering branch 4/5 for rule 'structured_value' at position 7` → `💾 Memo miss for rule 39
  at position 7` → `🔤 Attempting to match terminal '#{' at position 18 (end: 20)` → `❌ Terminal '#{'
  failed at position 18 - found '<EOF>'`. The rule is ENTERED at 7 and its own first terminal is
  attempted at 18 (EOF) — the layout skipper consumed 7→18. WHERE: the emitted `#` arm in
  `consume_layout_for_terminal` (`generated/semantic_annotation_parser.rs:130824` pre-fix), emitted
  because `grammar_claims_introducer_as_non_comment(grammar_tree, "#")`
  (`rust/src/ast_pipeline/ast_based_generator.rs:6556`) returned `false`, because
  `node_is_unbounded_content` (`:6650`) scored the `/\s*/` separator as a comment content tail.
- [x] **ADDRESSED (verified)** — **REJECT → PASS on THREE oracles** (fresh release
  `parseability_probe` on the regenerated parser · `--interpret-parse` · the cert pass), for
  `@type: #{"a", "b"}` and `@type: #{}`, with the object control still PASS. Certificate coverage
  `semantic_annotation` **`115/0/82/33 spf=2` → `115/0/84/31 spf=0`** (seed 0), `UNKNOWN=31`
  byte-identical at seeds **0/7/42**. ⭐ **THE `UNKNOWN` DELTA IS ATTRIBUTED BY RULE NAME, NOT BY
  COUNT**: exactly `set_value` and `set_element` left the set, and **nothing** became newly UNKNOWN.
  The emitted `#` arm count in that parser goes **2 → 0** (and `allow_comment_skip` 2 → 0, since with
  all three introducers claimed the skipper reduces to whitespace).
- [x] **NO REGRESSION** — `comment_arm_suppression_matrix_is_pinned` PASS with exactly one row
  re-pinned, nine unchanged; `certified_grammars_are_byte_identical` (TOOLBOX 1.6, the
  interpreter↔generated-parser differential) **PASS**; `every_registered_grammar_is_classified_exactly_once`
  and `deferred_grammars_are_still_divergent_or_promote_them` PASS. `generated/json_parser.rs`
  regenerates BYTE-IDENTICAL. `ebnf` (`144/0/111/33 spf=8`) and `return_annotation` (`35/0/33/2
  spf=0`) cert byte-identical. `cargo test --lib --features "generated_parsers ebnf_dual_run"`
  **1118 passed / 1 failed** — and that one failure is proven pre-existing by a **two-arm control**,
  not by citing a record: reverting both source files to HEAD and re-running it alone reproduces the
  identical panic (`expected semantic_annotation fallback to detect '@' directives`); it is already
  tracked by `ENGINE-UNIVERSAL-SERVICES`. `make clippy_on_rust_change` PASS (source strict + generated
  strict + the correctness-roster policy, 68 pinned lints intact). `scripts/check_doctrines.sh` 25/25.
  ⭐ **`ACCEPT-SET-LEDGER:` MOVES IN BOTH DIRECTIONS AND THE REPORT SAYS SO** — WIDEN-from-∅ on
  `#{…}` (the rule matched nothing before, so nothing can regress) and **NARROW** on `#`-suffixed
  input (`@type: 1 # trailing` PASS → REJECT; `// trailing` unchanged, a different arm). The narrow is
  the correct reading of a grammar that defines `line_comment := "//"` and no `#` comment at all, and
  its live reach was MEASURED rather than assumed: across all **17** tracked grammars, **zero**
  annotation lines contain a `#`.
- [x] **LOCKSTEP** — book `docs/book/src/grammar-wellformedness.md` gains *"A second way a terminal
  never matches: the layout skipper reaches it first"* (sibling to the look-around section, with the
  suppression table, the position-jump trace and the two-direction accept-set report); the pinned
  matrix in `rust/src/parse_harness_equivalence.rs` re-pinned with the reason inline; this leaf + the
  Current Frontier + `docs/TASK_TREE.md`; `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`.

### `H.16.2b` — **THE DYNAMIC COMMENT-SKIP GUARD IS STILL AN EXACT-EQUALITY ALLOWLIST** (`todo`, opened 2026-08-22 session #257 by `H.16.2`)

- **WHY**: `H.16.2` fixed the STATIC half. The DYNAMIC backstop is still spelling-based —
  `rust/src/ast_pipeline/ast_based_generator.rs:7199` emits
  `let allow_comment_skip = expected != "#" && expected != "//" && expected != "/*" && expected != "/**"
  && expected != "///" && expected != "/";`. Its own comment states the correct intent (*"avoid
  swallowing comment-introducer tokens themselves"*), but a terminal that merely **starts with** an
  introducer — `"#{"`, a hypothetical `"//="` — is not equal to one, so skipping stays enabled.
- **SHAPE OF THE FIX**: `expected != "/" && !expected.starts_with('#') && !expected.starts_with("//")
  && !expected.starts_with("/*")`. ⭐ A strict SUPERSET of the current exemptions (`"/**"` starts with
  `/*`, `"///"` starts with `//`), so it can only ever DISABLE comment skipping in more cases, never
  enable it in fewer — the accept set can only NARROW, never silently widen.
- ⛔ **BOTH COPIES MOVE TOGETHER**: `rust/src/parse_harness_interpreter.rs:3045` carries the same
  allowlist and mirrors codegen byte-for-byte (`PARSE-HARNESS.5.2`); TOOLBOX 1.6's
  differential-equivalence gate is the check that they did.
- ⚠️ **WHY IT IS A SEPARATE SLICE, AND THE HONEST BOUND**: this edits the emitted layout skipper of
  **every** arm-emitting parser, so every such artifact changes bytes — a repo-wide regeneration and
  rebaseline, versus `H.16.2`'s measured one-artifact radius. **NO LIVE VICTIM IS KNOWN**: in the
  grammars that DO emit an arm, the introducer-prefixed terminals are `ebnf`'s `/**` and `///` (both
  already exempt by exact match, and `documentation_comment` is WITNESSED, so it fires) and
  `semantic_annotation`'s `#{` (fixed by `H.16.2`) and `///`. This is prophylactic hardening of a
  proven class, not a live defect — which is exactly why it must not ride along inside another slice's
  verification.
- **OWED BY `H.16.1`**: the other 10 sites of the 11-site class census were cleared by arm-suppression
  and witness status, **not** by an individual accept/reject probe. Probe them here.

### `H.16.6` — **`=>` IS BOTH THE MAP ARROW AND THE IMPLICATION OPERATOR, SO A MAP ENTRY PARSES *IFF* ITS `key => value` IS NOT A VALID EXPRESSION** (**`diagnosed`**, `PGEN-GRAMMAR-WELLFORMED-0168`, doc+artifact tier — opened 2026-08-22 session #257 by `H.16.2`, DIAGNOSED same session; the FIX is a language-design call routed to `H.16.6a`)

- **WHY THE LEAF EXISTS**: found by **attributing** a leftover `sample_parse_failures` rather than
  rounding it down. After `H.16.2`, `semantic_annotation` read `spf=0` at seeds 0 and 42 and **1** at
  seed 7, on `@ idempotent :  {    +3.67=> psnG,+.6    =>   0xFFa0  }`. ⛔ **Proven pre-existing, not
  introduced**: the same input rejects **byte-identically** (`Backtrack at position 16
  [furthest_position=33]`) on the pre-fix release binary.

#### ⛔⛔ DIAGNOSIS — the map KEY swallows the whole entry, arrow included

- **THE MINIMAL REPRODUCER IS SHARPER THAN THE SAMPLE**, and its control is the same input with one
  token changed (`--interpret-parse`, all with the same annotation name so the name is not a variable):

  | input | verdict |
  |---|---|
  | `@type: {"a" => "b"}` | **accepted** |
  | `@type: {1 => "b"}` | **accepted** |
  | `@type: {"a" => 1}` | **accepted** |
  | `@type: {1 => 2}` | **REJECTED** `furthest_position=14` |

  ⇒ neither a numeric KEY alone nor a numeric VALUE alone breaks it. **Only both together.**
- **THE TRACE NAMES THE MECHANISM IN FOUR LINES** (`PGEN_TRACE_VERBOSITY=debug`, the BAD arm):
  ```text
  ✅ Rule 'annotation_value' successfully parsed from 8 to 14 (consumed 6 bytes: '1 => 2')
  ✅ Regex '\s*' matched at position 14 (len 0)
  🔤 Attempting to match terminal '=>' at position 14 (end: 16)
  ❌ Terminal '=>' failed at position 14
  ❌ Exiting rule 'map_entry' with error: Backtrack { position: 14 } - backtracked to 14
  ```
  `map_entry := annotation_value /\s*/ "=>" /\s*/ annotation_value`, and its **FIRST** `annotation_value`
  — the KEY — consumed **`1 => 2`** entire. `map_entry` then looks for its own `"=>"` at 14 and finds
  `}`. The GOOD arm at the same point reads `✅ Exiting rule 'map_entry' successfully - advanced from
  8 to 16`.
- ⭐⭐ **ROOT CAUSE (WHY + WHERE): `=>` IS OVERLOADED.** `grammars/semantic_annotation.ebnf:332`
  ```ebnf
  implication_expr := logical_or_expr (/\s*/ "=>" /\s*/ logical_or_expr)?
  ```
  `annotation_value := primitive_value | structured_value | expression_value | reference_value`, and
  branch 3 `expression_value → logical_expression → implication_expr` reaches that arrow. The
  `rule_stack` the error carries is the whole route, in one string:
  `["semantic_annotation", "annotation_value", "structured_value", "map_value", "map_entry",
  "annotation_value", "expression_value", "logical_expression", "implication_expr", …]`.
- ⭐ **THE CHARACTERIZATION IS EXACT, AND THE TWO TABLES ARE COMPLEMENTS** — which is what makes this a
  diagnosis rather than a hypothesis. Tested standalone, outside any map:

  | input | verdict |
  |---|---|
  | `@type: 1 => 2` | **accepted** — a valid `implication_expr` |
  | `@type: "a" => 1` | rejected |
  | `@type: 1 => "b"` | rejected |

  Exactly the inputs that are valid implications are the ones that FAIL inside a map, and exactly the
  ones that are not are the ones that succeed. ⇒ **a map entry parses iff its `key => value` is NOT a
  valid `implication_expr`.** PEG ordered choice commits `annotation_value` to the expression reading,
  and `map_entry` has no way to ask for a shorter key.
- ⛔ **`--lint-grammar` IS BLIND TO IT**: `ordered_choice_shadowing=0`, `exit 0`. The two readings do
  not shadow each other at a single choice point — the ambiguity is over a **token shared by two rules
  at different depths**, which no current lint class describes.
- ⚠️ **AND IT IS INVISIBLE TO CERTIFICATE COVERAGE BY CONSTRUCTION.** `map_entry` is **witnessed**
  (`[plannable-probe] rule='map_entry' parsed=true witnessed_target=true sample="@   type   : {
  \"3\"=> \"\\i\"  }"` — a string-keyed entry, which is in the accepted half). ⭐⭐ **THE TRANSFERABLE
  PART: a WITNESSED rule can still reject inputs its grammar licenses.** `UNKNOWN=0` on this family
  would not have caught this, and no amount of witness work will.
- ⛔ **A THIRD "FINDING" WAS KILLED BY ITS OWN CONTROL, AND IS RECORDED SO IT IS NOT RE-FOUND.**
  `@type: 1 => 2` yields a typed AST whose `value` is the EMPTY STRING, which looks like a silent AST
  loss from `implication_expr`'s `-> $1` discarding the right operand. It is **not**: plain `@type: 1`
  and `@type: 1 + 2` yield `value: ""` too, so the empty value is this annotation shape's normal
  reporting and has nothing to do with `=>`.

#### Routed — the FIX is a language-design call, not a repair

- **`H.16.6a`** owns it. The options are not equivalent and each moves the accept set:
  (a) give the map key its own rule that routes around the implication level — a map key can then no
  longer BE an implication, which is arguably correct since `{a => b => c}` is ambiguous anyway;
  (b) guard `implication_expr`'s optional tail with a negative lookahead so it declines when the
  right operand is followed by `}` or `,` — narrower, but encodes map context into an expression rule;
  (c) change one of the two spellings of `=>` — the largest accept-set move and the only one that
  removes the ambiguity outright.
  ⛔ Each needs an `ACCEPT-SET-LEDGER:` entry; **do not pick one without measuring all three**, which is
  the discipline `H.16.4`'s closed facet matrix just demonstrated.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `./rust/target/debug/ast_pipeline grammars/semantic_annotation.ebnf
  --interpret-parse` on `@type: {1 => 2}` ⇒ `accepted=false furthest_position=14`, against three
  ACCEPTING controls differing by one token (`{1 => "b"}`, `{"a" => 1}`, `{"a" => "b"}`). Reproduced on
  the real generated parser too (`Backtrack at position 16` on the seed-7 stimuli sample) and shown
  **pre-existing** on the pre-`H.16.2` release binary, byte-identically.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `grammars/semantic_annotation.ebnf:332`
  `implication_expr := logical_or_expr (/\s*/ "=>" /\s*/ logical_or_expr)?` versus `:290`
  `map_entry := annotation_value /\s*/ "=>" /\s*/ annotation_value`. WHY: `PGEN_TRACE_VERBOSITY=debug`
  shows `Rule 'annotation_value' successfully parsed from 8 to 14 (consumed 6 bytes: '1 => 2')`
  followed by `Terminal '=>' failed at position 14` and `Exiting rule 'map_entry' with error:
  Backtrack { position: 14 }` — the key consumed the arrow. The carried `rule_stack` names the full
  route through `expression_value → logical_expression → implication_expr`.
- [x] **ADDRESSED (verified)** — this leaf's deliverable is the DIAGNOSIS and it is complete: the
  defect is characterized by an **exact biconditional** (*a map entry parses iff its `key => value` is
  not a valid `implication_expr`*), demonstrated by two complementary four-row tables, with the
  mechanism named at file and line and the fix routed to `H.16.6a` with three priced options.
  Before → after on this leaf's own metric: an unattributed seed-dependent `spf` reading → a named,
  minimally-reproduced, mechanism-level defect with an owning leaf. ⛔ **ZERO grammar bytes, ZERO Rust
  bytes, ZERO codegen bytes, ZERO generated bytes** — no verdict moved and none is claimed to have.
- [x] **NO REGRESSION** — doc+artifact tier, parser surface inert BY CONSTRUCTION. Re-verified at HEAD
  after the slice: `ebnf` `144/0/112/32 spf=4`, `return_annotation` `35/0/33/2 spf=0`,
  `semantic_annotation` `115/0/84/31 spf=0`, byte-identical to the previous slice's readings;
  `scripts/check_doctrines.sh` 25/25. No grammar file was modified (`git diff grammars/` empty).
- [x] **LOCKSTEP** — this leaf + `H.16.6a` + the Current Frontier + `docs/TASK_TREE.md`; `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`. Book: N/A while the leaf is `diagnosed` — the accepted language
  has not moved; it becomes book material when `H.16.6a` rules, since that WILL move it.

### `H.16.6a` — **PICK AND MEASURE THE DISAMBIGUATION OF `=>`** (`todo`, opened 2026-08-22 session #257 by `H.16.6`)

- **WHY**: `H.16.6` proved the defect and priced three fixes; each moves the accept set differently and
  none is obviously right. Options (a) a dedicated map-key rule routing around the implication level,
  (b) a negative-lookahead guard on `implication_expr`'s optional tail, (c) re-spell one of the two
  `=>` uses.
- ⛔ **MEASURE ALL THREE BEFORE PICKING** — the shape `H.16.4` demonstrated: build each arm on a
  SCRATCH copy of the grammar and score it with `--interpret-parse` against a two-arm control (does
  `{1 => 2}` parse / does every currently-accepted input still parse). No regeneration is needed for
  any of it.
- **AND EACH ARM OWES AN `ACCEPT-SET-LEDGER:` ENTRY** — (a) narrows what a map KEY may be, (b) narrows
  where an implication may END, (c) narrows or moves the surface outright. ⚠️ None of the three is
  WIDEN-only, so the `-0161`/`H.16.2` "widen-from-∅, nothing can regress" argument does **not** apply
  here and must not be reused.

### `H.16.3` — **THE STIMULI GENERATOR SHADOWS ANY RULE NAMED `epsilon` WITH THE EMPTY STRING** (**`done`**, `PGEN-GRAMMAR-WELLFORMED-0166`, CODE / engine-universal stimuli generator — opened 2026-08-22 session #257 by `H.16.1`, CLOSED same session)

- **WHY**: `rust/src/ast_pipeline/stimuli_generator.rs:11186` — `generate_rule` returned
  `Ok(String::new())` for `rule_name == "epsilon"` **before consulting the grammar**, so
  `grammars/ebnf.ebnf:324`'s real definition `epsilon := ("ε" | "epsilon" | "empty" | "λ")` was
  unreachable to the generator while codegen honoured it.

#### ✅ CLOSED — a DEFINED rule always wins; the builtin is the fallback it was written to be

- ⭐ **THE BUILTIN ITSELF IS LEGITIMATE AND IS KEPT.** Its contract is pinned by
  `built_in_epsilon_rule_reference_generates_empty_string` (`:19324`) and relied on by
  `SV-EXH-PROOF.7.4.6.15`'s control (`:28762`), and **both use a grammar in which `epsilon` is
  UNDEFINED** — a bare `rule_reference` with no rule behind it. Gating the builtin on
  `!self.grammar_tree.contains_key("epsilon")` therefore preserves both **exactly**, which is why the
  fix is one condition rather than a redesign. ⚠️ `:28866` had already foreseen this edit in writing
  (*"if this assert ever fails the verdict was tightened (e.g. `epsilon` taught to resolve)"*); it did
  not fire, because that control's grammar does not define the rule.
- ⛔ **CODEGEN AND THE GENERATOR ALREADY DISAGREED ABOUT WHAT THE NAME MEANS.** `epsilon` is **not**
  in `AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS`
  (`ast_based_generator.rs:1405` — only `builtin_any_char` and `builtin_ascii_char`), so codegen
  treats a defined `epsilon` as an ordinary rule. One name, two meanings, in the two halves of the
  same pipeline: **that is the defect, not the empty-string expansion.**
- ⚠️ **AND THE OTHER DIRECTION IS ADJUDICATED, NOT IGNORED**: for an UNDEFINED `epsilon` the generator
  renders `""` while codegen emits a never-matching `Err(Backtrack)` stub — the generator would emit
  samples for a production the parser can never accept. It is **not** a live defect and needs no leaf:
  `undefined_references` is a HARD `--lint-grammar` error class, so no shipped grammar can carry an
  undefined `epsilon` reference; the only inputs that do are the two synthetic test grammars above.
- **BLAST RADIUS IS ONE GRAMMAR, AND THE POPULATION IS CLOSED.** `grep -l '^epsilon' grammars/*.ebnf`
  → `grammars/ebnf.ebnf` **only**. `grammars/systemverilog.ebnf`'s remaining mentions are COMMENTS
  recording its removal (`lines 77-80`, `2400`). ⭐ The **trap** is universal: any grammar in the world
  that defines a rule named `epsilon` silently lost it.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 … --report-certificate-coverage` on
  `grammars/ebnf.ebnf` prints `[plannable-probe] rule='epsilon' parsed=false witnessed_target=false
  sample="ZC:="` ×8 (plus `[target-own-probe]` and `[carrier-div-probe]` variants) — every forced
  witness renders an EMPTY right-hand side, which the real parser REJECTS, so `epsilon` could never be
  witnessed and each probe also inflated `sample_parse_failures`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/src/ast_pipeline/stimuli_generator.rs:11186`,
  `if rule_name == "epsilon" { … return Ok(String::new()); }`, evaluated before any grammar lookup.
  WHY it is the GENERATOR and not the language: `--interpret-parse` against `grammars/ebnf.ebnf` gives
  `X := ε` → `accepted=true`, `X := epsilon` → `accepted=true`, `X := empty` → `accepted=true`,
  `X := λ` → `accepted=true`, and the generator's own sample `X :=` → **`accepted=false`
  furthest_position=4**. The parser was right about the grammar and the generator was not.
- [x] **ADDRESSED (verified)** — `grammars/ebnf.ebnf` certificate coverage, **three seeds, both arms
  measured on the same binary path** (the BEFORE arm was re-measured this session by reverting the
  file and rebuilding, not quoted from an earlier note):

  | | seed 0 | seed 7 | seed 42 |
  |---|---|---|---|
  | BEFORE | `144/0/111/33` `spf=8` | `144/0/111/33` `spf=11` | `144/0/111/33` `spf=7` |
  | AFTER | `144/0/112/32` `spf=4` | `144/0/112/32` `spf=5` | `144/0/112/32` `spf=3` |

  ⭐ **`spf` falls at EVERY seed** (8→4, 11→5, 7→3) — a monotone three-seed improvement, not a
  single-seed reading. ⭐ **THE `UNKNOWN` DELTA IS ATTRIBUTED BY RULE NAME**: exactly `epsilon` left
  the set; **nothing** became newly UNKNOWN. The `[plannable-probe] rule='epsilon'` lines are gone
  entirely — the rule is now witnessed by the ordinary diverse pass, so no forced probe is generated.
  ⚠️ **HONEST BOUND**: the remaining 4/5/3 `spf` are the `ebnf` residue `H.16.1` recorded as NOT YET
  ATTRIBUTED, and they are **not** claimed to be epsilon-related. Two of the four at seed 0 now
  contain `epsilon`/`λ` because the sample stream shifted; all four spellings were probed individually
  in `primary_element` position (`X := epsilon`, `ε`, `empty`, `λ`, `&epsilon`, `"a" | λ`) and **all
  six ACCEPT**, so the residue is not the epsilon branch failing to parse.
- [x] **NO REGRESSION** — **The stimuli generator is not parser codegen, and the tracked baseline
  PROVES it rather than assuming it**: `generated_reproducibility_rebaseline` re-derives all **11**
  artifacts byte-identically and every `parser_sha` in the rebaselined file is **unchanged**
  (`return_annotation` `0d610b7a…`, `semantic_annotation` `e2a3d4cd…`, `rtl_const_expr` `8f14a980…`);
  only `verified_at_commit`, `emission_sha` and three `.json` `input_sha` (the embedded `generated_at`)
  moved. Cert unchanged for every other wired family: `return_annotation` `35/0/33/2 spf=0`,
  `semantic_annotation` `115/0/84/31 spf=0`, and the **fully-certified** set holds — `json` `9/0/9/0`,
  `regex` `269/9/260/0`, `rtl_frontend` `169/1/168/0`, `vhdl` `225/0/225/0`,
  `systemverilog_preprocessor` `74/0/74/0`, all `fully_certified=true spf=0`; the canonical
  `rtl_const_expr_cert_gate` PASSES `48/0/48/0` deterministic at seeds 0/7/42. `cargo test --lib`
  **1118 passed / 1 failed** — the same pre-existing failure, counts identical to the previous slice.
  `clippy_on_rust_change` PASS. `scripts/check_doctrines.sh` 25/25.
  ⭐ **A SCARE THAT RESOLVED INTO A LESSON**: an ad-hoc `--report-certificate-coverage` on
  `rtl_const_expr` returned `Error: Stimuli generation depth exceeded max_depth=24`. That was **the
  wrong invocation, not a regression** — the canonical gate runs at depth 32 and passes. ⛔ *For a
  grammar with a canonical cert gate, the GATE is the oracle; an ad-hoc command with default flags is
  a different measurement wearing the same name.*
- [x] **LOCKSTEP** — this leaf + the Current Frontier + `docs/TASK_TREE.md`; `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`; the rebaselined
  `rust/test_data/grammar_quality/generated_reproducibility_v0.json`. Book: N/A — no user-facing
  surface changed (the accepted LANGUAGE is untouched; only which samples the generator emits), and
  the `ebnf` grammar itself is unmodified.

### `H.16.4` — **`ebnf`'s `whitespace` RULE IS LAYOUT-SKIPPED BEFORE `grammar_file` IS OFFERED THE BYTES** (**`done`** — ADJUDICATED, `PGEN-GRAMMAR-WELLFORMED-0167`, doc+artifact tier — opened 2026-08-22 session #257 by `H.16.1`, CLOSED same session; the CAPABILITY it needs is routed to `H.16.4a`)

- **WHY**: `grammar_file := (include_directive | semantic_annotation | grammar_rule | comment |
  whitespace)*` and `whitespace := /(\s+)/`, yet the rule can never be witnessed.

#### ✅ ADJUDICATED — the engine guards every COMMENT arm against the active token and does not guard the WHITESPACE skip

- ⭐⭐ **THE ASYMMETRY IS IN THE EMITTED SOURCE, AND IT EXPLAINS WHY `comment` IS WITNESSED AND
  `whitespace` IS NOT — the two sit in the SAME alternation.** `generated/ebnf.rs`
  `consume_layout_for_regex`:
  ```rust
  loop {
      let before = self.position;
      self.consume_optional_whitespace();                        // <- UNCONDITIONAL. No guard.
      …
      if bytes[self.position] == b'#' {
          if self.regex_token_matches_at_cursor(pattern) { break; }   // <- GUARDED (H.11.3)
          … skip to EOL …
      }
      if bytes[self.position] == b'/' && bytes[self.position+1] == b'/' {
          if self.regex_token_matches_at_cursor(pattern) { break; }   // <- GUARDED
  ```
  Every comment arm asks *"would the token I am about to match consume these bytes itself?"* before
  eating them. **The whitespace skip never asks.** ⇒ `comment`'s branch survives because its own first
  terminal is a comment introducer and the dynamic guard protects it; `whitespace`'s cannot, because
  the bytes are gone before any guard is consulted. ⭐ **Third instance of this family in one session**
  (`H.17` an uncompilable terminal, `H.16.2` an unguarded comment introducer, this) — each time the
  layout machinery protects exactly the arm someone remembered to protect.
- **MEASURED, not inferred** — `--interpret-parse` on an input of 4 spaces:
  `accepted=true furthest_position=0`, typed AST `{"elements": [], "type": "grammar_file"}` **span
  0..0**. The `*` matched **ZERO** iterations; the trailing-layout skip consumed everything.

#### ⛔⛔ THE DECLARATIVE FIX EXISTS, WAS TRIED, AND IS **REFUTED** BY A CONTROL — over a CLOSED facet matrix

`@whitespace_sensitive:` (`semantic_runtime::compile_layout_sensitivity`) is the declarative tier, and
a sibling grammar already ships the exact shape: **`grammars/systemverilog_preprocessor.ebnf:23`
declares `@whitespace_sensitive: { regex_tokens: true }`, its `space_or_tab := /[ \t]+/` IS witnessed,
and that family reads `74/0/74/0 fully_certified=true`.** So the precedent said this was a one-line
lookup, exactly like `-0161`'s.

**It is not.** Every facet was measured on `grammars/ebnf.ebnf` through `--interpret-parse` (which
mirrors codegen's layout skipping byte-for-byte, `PARSE-HARNESS.5.2`, so **no regeneration was
needed**), against two arms — does the target rule commit, and does the meta-parser still read a REAL
shipped grammar file:

| `@whitespace_sensitive:` | `whitespace` commits? | `grammars/json.ebnf` still parses? |
|---|---|---|
| *(none — the shipped baseline)* | no | **yes** |
| `{ regex_tokens: true }` | **YES** (AST `elements:[{"content":"    ","type":"whitespace"}]`, span 0..4) | **no** |
| `{ terminals: true }` | no | **no** |
| `{ trailing: true }` | no — and it makes it WORSE (`accepted=false`, the ws-only input no longer parses at all) | **no** |
| `true` (all three facets) | **YES** | **no** |

⭐ **The population is CLOSED — three boolean facets, all four meaningful settings tried — and NO
setting satisfies both arms.** Two of the four witness the rule; **all four break `json.ebnf`.**

- **WHY the precedent does not transfer, stated as a property rather than a shrug**: the directive is
  **GRAMMAR-WIDE**, and it disables layout skipping before *every* regex terminal in the grammar.
  `systemverilog_preprocessor` can afford that because a preprocessor grammar structurally owns ALL of
  its whitespace. `ebnf.ebnf` cannot: `rule_name := /([a-zA-Z_][a-zA-Z0-9_]*)/` and its other regex
  terminals RELY on preceding layout being skipped, which is why a real grammar file stops parsing.
- ⛔ **AND THE GRAMMAR TIER HAS NO MOVE EITHER.** The skip runs before any terminal match, so no
  spelling of `whitespace` avoids it; the only grammar-tier "fixes" are deleting the rule from the
  alternation (forbidden by this tree) or anchoring it on a non-whitespace byte (a different language).
  ⇒ declarative REFUTED by measurement · grammar tier empty · **the remaining tier needs a capability
  that does not exist** → `H.16.4a`.
- **DISPOSITION UNTIL THEN — a NAMED residual class, not an unexplained `UNKNOWN`.** `whitespace` is
  **not** a dead rule (it is referenced, and it has a reach path from the entry) and **not** a reach
  gap (the planner routes to it and the sample parses). It is **layout-shadowed**: provably never
  witnessable while the engine owns layout, which makes it a candidate for a `proof` certificate
  rather than a witness → the same machinery `H.16.5` owns. ⚠️ It is deliberately left `UNKNOWN` here
  rather than promoted, because promoting it before `H.16.4a` rules would record a fixable defect as a
  design fact — the exact error `H.16`'s own re-pricing note warns against.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 … --report-certificate-coverage` on
  `grammars/ebnf.ebnf` prints `[plannable-probe] rule='whitespace' parsed=true
  witnessed_target=false sample="    "` ×4. `parsed=true` with `witnessed_target=false` is the
  reach/routing signature, not a malformed sample: the probe IS four spaces and the parser accepts it.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `consume_layout_for_regex` in the emitted parser
  (`generated/ebnf.rs`), whose `self.consume_optional_whitespace()` runs UNCONDITIONALLY at the head
  of the loop while every comment arm below it is gated on `regex_token_matches_at_cursor(pattern)`.
  WHY it is not the planner (TOOLBOX 4.4's standing warning): `--interpret-parse` on 4 spaces returns
  `accepted=true furthest_position=0` with typed AST `elements: []` and span **0..0** — the
  alternation ran ZERO iterations, so nothing was routed anywhere; the bytes were gone first. Control:
  `"  \nX := \"a\"\n"` likewise yields only a `grammar_rule` element and no `whitespace` node.
- [x] **ADDRESSED (verified)** — this leaf's deliverable is the ADJUDICATION, and it is complete and
  CLOSED: the declarative tier is identified, the sibling precedent is named
  (`systemverilog_preprocessor.ebnf:23`), and **all four settings of the three-facet directive are
  measured against a two-arm control**, with the result that none satisfies both arms — `{
  regex_tokens: true }` and `true` witness the rule and break `grammars/json.ebnf`; `{ terminals: true
  }` and `{ trailing: true }` break it without even witnessing. Before → after on this leaf's own
  metric: `whitespace` goes from *an unexplained `UNKNOWN` carrying an unmeasured hypothesis* to *a
  named residual class (**layout-shadowed**) with a measured refutation of the obvious fix and an
  owning leaf for the capability it needs*. ⛔ **ZERO grammar bytes, ZERO Rust bytes, ZERO codegen
  bytes, ZERO generated bytes** — no `UNKNOWN` moved and none is claimed to have.
- [x] **NO REGRESSION** — doc+artifact tier, so the parser surface is inert BY CONSTRUCTION; the
  facet matrix was measured on a **scratch copy** (`rust/target/h16/ebnf_ws.ebnf`, untracked) and
  `grammars/ebnf.ebnf` is byte-unmodified. Re-verified at HEAD after the slice: `ebnf`
  `144/0/112/32 spf=4`, `return_annotation` `35/0/33/2`, `semantic_annotation` `115/0/84/31`, all
  byte-identical to the previous slice's readings; `scripts/check_doctrines.sh` 25/25.
- [x] **LOCKSTEP** — this leaf + `H.16.4a` + the Current Frontier + `docs/TASK_TREE.md`; `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`. Book: N/A — nothing user-facing changed and no book claim is
  falsified; the layout-guard asymmetry becomes book material when `H.16.4a` lands a fix.

### `H.16.4a` — **`@whitespace_sensitive` IS GRAMMAR-WIDE; THE PROPERTY A LAYOUT-OWNING RULE NEEDS IS PER-TERMINAL** (`todo`, opened 2026-08-22 session #257 by `H.16.4`)

- **WHY**: proven by the closed facet matrix in `H.16.4` — a grammar cannot say *"this ONE rule owns
  its whitespace"*, only *"no regex terminal in this grammar gets layout skipped"*. For `ebnf.ebnf`
  the first is exactly right and the second demonstrably breaks reading `grammars/json.ebnf`.
- **THE SHAPE IS ALREADY IN THE ENGINE, ONE ARM OVER.** `consume_layout_for_regex`'s comment arms are
  each gated on `regex_token_matches_at_cursor(pattern)` — *"would the token I am about to match
  consume these bytes itself?"*. The whitespace skip needs the same question, and the predicate for
  asking it SAFELY already exists: `AstBasedGenerator::hir_matches_only_whitespace`, landed by
  `H.16.2`. ⇒ guard `consume_optional_whitespace()` when the active token's pattern is provably
  whitespace-only, so a rule like `whitespace := /(\s+)/` keeps its bytes and every other terminal is
  untouched.
- ⚠️ **PRICE THE BLAST RADIUS BEFORE BUILDING, and the census is already taken** (`H.16.4`): **112**
  whitespace-only regex terminal sites across the grammars, but the overwhelming majority are `/\s*/`
  SEPARATORS which take `consume_layout_for_regex`'s `can_match_empty` early return and are NOT
  affected. The exposed class is the non-empty-matching ones: `ebnf` `whitespace`,
  `semantic_annotation` `whitespace` / `precedence_value` / `constraint_value`,
  `systemverilog_preprocessor` `space_or_tab` (already covered by its own declaration), and the two
  `systemverilog_lrm_profiled_*` `white_space` (not shipped families). ⛔ Re-derive this rather than
  inheriting it — the census was produced by a REGEX over grammar text, whereas the engine's own
  authority is the HIR walk.
- **AND IT DECIDES A CLASSIFICATION, NOT JUST A RULE.** If the guard lands, `whitespace` is witnessed
  and `ebnf` drops to `UNKNOWN=31`. If it is refused, `whitespace` is provably never witnessable and
  belongs in `H.16.5`'s `proof`-promotion population as a **layout-shadowed** residual. ⛔ Do not
  promote it before this leaf rules — recording a fixable defect as a design fact is precisely what
  `H.16`'s re-pricing note exists to prevent.

### `H.16.5` — **THE 55 SOURCE ORPHANS ARE ALREADY OWNED BY `LANG-CAPABILITY-AUDIT`; WHAT THIS LEAF OWES IS THE 9 LR RESIDUE AND THE PROOF-PROMOTION GATE** (`todo`, opened 2026-08-22 session #257 by `H.16.1`, **RE-SCOPED same session by `-0170` after a director correction**)

- ⛔⛔ **RETRACTION — THIS LEAF ORIGINALLY ESCALATED A QUESTION THE REPOSITORY HAD ALREADY ANSWERED.**
  It read: *"DIRECTOR-FACING: does PGEN's EBNF intend to accept `except`, `~i"…"`, `{? … ?}`,
  `rule[p]`, `rule<T>`, `mode`, `extends`, `import`, optimization hints and error productions?"*
  **That was wrong, and the director corrected it (2026-08-22):** *"there are features not yet
  supported that have been documented, not fully though, that EBNF shall support in the future…
  search the task-trees and KM cards and ADRs, you should find mentions about those things"* —
  together with the binding instruction **do not simply park findings; task-tree OWN them**.
- ⭐⭐ **AND EVERY ONE OF THE 13 ISLANDS WAS ALREADY TRACKED.** A census over `docs/tasks/`,
  `docs/decisions/` and `docs/knowledge/` finds **all 13 names present**, most in several layers.
  The authoritative homes, which this leaf must CITE rather than re-invent:
  - **`LANG-CAPABILITY-AUDIT.1`** — the same population, found first and organized better: **27
    unreachable productions** in `grammars/ebnf.ebnf`, clustered into **7 families** each mapped to a
    horizon axis (Error recovery · Parameterized productions · Lexer modes · Generics/templates ·
    Semantic predicates (inline form) · Grammar composition · Misc). Its headline is the one to quote:
    ***"PGEN's meta-grammar has been quietly documenting its own capability gaps — 20.6 % of our own
    EBNF language is decorative."***
  - **`LANG-CAPABILITY-AUDIT.4`/`.6`** — the DISPOSITIONS already exist, per-cluster: `~i"…"`
    (`case_control`), `[a-z]` (`character_class`) and `import`/`extends` are **superseded by a
    canonical existing form** (`(?i:…)`, `/.../`, `include()` per P0-1) and `.6` may retire them;
    ⛔ with one carve-out — **`parametric_rule` is a capability the roadmap WANTS (P2-5) and must be
    re-surfaced, NEVER deleted.**
  - **[[feedback_capability_work_is_greenlit_by_standing_authorization]]** — the capability is
    **already greenlit by standing authorization**. What is genuinely open is only the parametric
    **NOTATION**, and even that is bounded: the declared placeholder
    `parametric_rule := rule_name "[" parameter_list "]"` **silently miscompiles**, because `[ … ]` is
    already the optional-element form, so `expr[In, Yield]` parses as `expr ( In Yield )?` with the
    comma swallowed. That record's own *"How to apply"* says it outright: ***"Do not open a leaf asking
    'may we build capability X?' for a priced row — build it, or schedule it, and say so."***
  ⇒ ⛔ **The escalation violated a standing instruction that was already written down.** The
  transferable rule is [[feedback_answer_your_own_technical_questions]]: SEARCH the trees, ADRs and KM
  cards BEFORE declaring a finding novel or routing a question upward.
- ⭐ **THE ONE THING `H.16.1` ADDED HERE IS A CROSS-METHOD CORROBORATION, AND IT IS EXACT.**
  `LANG-CAPABILITY-AUDIT.1` derived **27** by a hand closure over `--dump-gen-ast` in session #208;
  `H.16.1`'s census derives `ebnf source_orphans=` **27** from a PRE/POST two-arm diff in session
  #257, by different code, from a different dump, with a different definition. **The two agree
  exactly**, which is worth more than either number alone.

#### What this leaf actually owes (re-scoped)

1. **THE 9 LR RESIDUE — genuinely new, owned by nobody else.** `expression_return` +3,
   `accessor_base`, `union_type`/`intersection_type`/`array_type`/`optional_type`. These are wired by
   their grammar and orphaned by PGEN's OWN elimination pass, so they are **not** capability gaps and
   must never be routed to `LANG-CAPABILITY-AUDIT` — same class `-0271` adjudicated for
   `verilog_2005`. They are `proof`-promotion candidates, not grammar debt.
2. **THE PROOF-PROMOTION GATE.** `VERILOG-2005-PROFILE.6.7` built P1/P2 promotion and `main.rs:3380`
   gates it on `profile.is_some()`; all three families are `profiles=[]`, so it never runs and they
   read `proof=0`. The gate's stated reason is about the ENTRY UNIVERSE, which `main.rs:3821` builds
   from the entry plus every `--cert-union-config` independently of any profile ⇒ decoupling them is
   the **engine-universal, agnostic** change. ⛔ Promotion must FOLLOW adjudication: a rule that
   SHOULD be wired must not be quietly proved dead.
3. **LEG 3 FOR `H.16.1`** — nothing re-runs the island census, so the 9/55 split rots the moment a
   grammar gains a rule. ⚠️ Bound it to what is genuinely unwatched: `LANG-CAPABILITY-AUDIT.1`'s 27
   are already a tracked population, so the watch this owes is over the **LR-residue delta**, which is
   the half no existing instrument computes.

## Current Frontier

> ⛔ **SEQUENCED, NOT PARKED (director, 2026-08-22: *"do not simply park them"*).** Every `todo` below
> carries an ORDER, and ordering already-approved work is execution, not a director call
> ([[feedback_answer_your_own_technical_questions]]). The order is: **1** `H.16.6a` (a live accept-set
> defect — a real input the grammar licenses is rejected) → **2** `H.16.4a` (the last unfixed arm of
> the layout-guard family, and it decides `whitespace`'s classification) → **3** `H.16.2b` (the same
> family, prophylactic, repo-wide radius so it must not ride along inside another slice) → **4**
> `H.16.5` (the 9 LR residue + the `profile.is_some()` proof-promotion gate) → **5** `H.20` (a RED
> gate at HEAD) → **6** `H.19` (leg 3 for `H.15`). ⭐ The 55 source orphans are NOT in this queue:
> they are already owned by `LANG-CAPABILITY-AUDIT.1`/`.4`/`.6` with per-cluster dispositions, and
> `H.16.5`'s retraction records why re-opening them here would be duplication.

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `GRAMMAR-WELLFORMED.H.16.2` (a terminal whose PREFIX is a comment introducer is eaten as a comment) | **`done`** (`PGEN-GRAMMAR-WELLFORMED-0165`, CODE / engine-universal codegen) | ✅ `set_value` / `set_element` REJECT→PASS on THREE oracles; cert `115/0/82/33 spf=2` → **`115/0/84/31 spf=0`**, `UNKNOWN=31` at seeds 0/7/42, delta ATTRIBUTED BY NAME (exactly those two left; nothing newly UNKNOWN). Root cause: `node_is_unbounded_content` scored the `/\s*/` SEPARATOR as a comment content TAIL, dropping the `#` claim — the same spelling-vs-property error `H.17.2` outlawed one layer up. Blast radius MEASURED by the pinned suppression matrix: **exactly one row moved**, nine unchanged; `json_parser.rs` regenerates BYTE-IDENTICAL through the changed annotation backend |
| — | `GRAMMAR-WELLFORMED.H.16.6` (`=>` is both the map arrow and the implication operator) | **`diagnosed`** (`PGEN-GRAMMAR-WELLFORMED-0168`, doc+artifact tier) | ✅ ROOT-CAUSED to an EXACT BICONDITIONAL: **a map entry parses iff its `key => value` is NOT a valid `implication_expr`** — two complementary 4-row tables prove it. `map_entry`'s KEY consumes the arrow (`annotation_value … consumed 6 bytes: '1 => 2'` → `Terminal '=>' failed at position 14`). ⛔ `--lint-grammar` `ordered_choice_shadowing=0`, and `map_entry` is WITNESSED ⇒ **a witnessed rule can still reject inputs its grammar licenses**. Fix is a language-design call → `H.16.6a` |
| 1 | `GRAMMAR-WELLFORMED.H.16.6a` (pick and measure the disambiguation of `=>`) | **`todo`** (opened 2026-08-22 by `H.16.6`) | Three priced options, none WIDEN-only, so the `-0161`/`H.16.2` "widen-from-∅" argument does NOT transfer. Measure all three on a SCRATCH grammar copy via `--interpret-parse` (no regeneration), each with an `ACCEPT-SET-LEDGER:` entry |
| 3 | `GRAMMAR-WELLFORMED.H.16.2b` (the DYNAMIC comment-skip guard is still an exact-equality allowlist) | **`todo`** (opened 2026-08-22 by `H.16.2`) | Prophylactic hardening of a proven class, deliberately NOT ridden along inside `H.16.2`: the prefix test edits the emitted layout skipper of EVERY arm-emitting parser (repo-wide rebaseline) versus `H.16.2`'s measured one-artifact radius. No live victim known. Also owes the 10 unprobed sites of the 11-site class census |
| — | `GRAMMAR-WELLFORMED.H.16.3` (the generator shadows any rule named `epsilon` with `""`) | **`done`** (`PGEN-GRAMMAR-WELLFORMED-0166`, CODE / engine-universal stimuli generator) | ✅ A DEFINED rule now wins; the builtin is gated on `!grammar_tree.contains_key("epsilon")`, preserving both existing callers exactly (their grammars leave `epsilon` UNDEFINED). `ebnf` cert `144/0/111/33` → **`144/0/112/32`** with `spf` falling at EVERY seed (**8→4 · 11→5 · 7→3**), both arms measured on the same binary path; delta ATTRIBUTED BY NAME (exactly `epsilon` left, nothing newly UNKNOWN). ⭐ ZERO generated-parser bytes move — the rebaselined reproducibility file shows every `parser_sha` unchanged across all 11 artifacts |
| — | `GRAMMAR-WELLFORMED.H.16.4` (`ebnf`'s `whitespace` is layout-skipped before `grammar_file` sees it) | **`done`** — ADJUDICATED (`PGEN-GRAMMAR-WELLFORMED-0167`, doc+artifact tier) | ✅ Root cause is an ASYMMETRY in the emitted layout skipper: every COMMENT arm is gated on `regex_token_matches_at_cursor(pattern)`, the whitespace skip is not — which is why `comment` is witnessed and `whitespace`, in the SAME alternation, is not. ⛔⛔ The declarative tier EXISTS (`@whitespace_sensitive`, and `systemverilog_preprocessor.ebnf:23` ships the exact shape) and is **REFUTED by a closed facet matrix**: all four settings break `grammars/json.ebnf`, and only two of them even witness the rule. Named a **layout-shadowed** residual; the capability is `H.16.4a` |
| 2 | `GRAMMAR-WELLFORMED.H.16.4a` (`@whitespace_sensitive` is grammar-wide; the property needed is per-terminal) | **`todo`** (opened 2026-08-22 by `H.16.4`) | The guard shape is already in the engine one arm over, and `H.16.2`'s `hir_matches_only_whitespace` is exactly the predicate that makes it safe. Decides a CLASSIFICATION, not just a rule: guard lands ⇒ `ebnf` `UNKNOWN=31`; guard refused ⇒ `whitespace` joins `H.16.5`'s `proof`-promotion population |
| 4 | `GRAMMAR-WELLFORMED.H.16.5` (the 9 LR residue + the proof-promotion gate) | **`todo`** — RE-SCOPED by `-0170` | ⛔ **RETRACTED its own escalation**: the 55 source orphans were ALREADY owned by `LANG-CAPABILITY-AUDIT.1`/`.4`/`.6` (27 productions, 7 horizon-mapped clusters, per-cluster dispositions) and the capability is ALREADY greenlit by [[feedback_capability_work_is_greenlit_by_standing_authorization]] — only the parametric NOTATION is open, and `[ … ]` is taken by the optional-element form. What remains genuinely unowned: the **9 LR residue** (PGEN's own, never a capability gap) and the `profile.is_some()` gate on proof promotion |
| — | `GRAMMAR-WELLFORMED.H.16.1` (adjudicate the 68) | **`done`** (`PGEN-GRAMMAR-WELLFORMED-0164`, doc+artifact tier) | ✅ **68 = 9 LR-residue + 55 source-orphans + 4 root-caused inert rules**, in **31 islands**, partition CLOSED (zero rules unattributed). ⛔ `--lint-grammar` could never have adjudicated this — it reads `unreachable_rules=0` on all three BY CONSTRUCTION. New reader: `docs/tasks/artifacts/grammar_wellformed/residual_island_census/probe.py`, proven to go RED three ways. Residual-rules-with-no-owning-leaf **68 → 0** |
| — | `GRAMMAR-WELLFORMED.H.16` (roll `ebnf` / `return_annotation` / `semantic_annotation` to `UNKNOWN=0`) | **`in_progress`** (adjudicated by `H.16.1`; work routed to `H.16.2`–`H.16.5`) | The **clean** conjunct, and now the only engineering half of the `SVPP-EXPANSION` gate left. `H.15` made the three measurable and they read `UNKNOWN` **35 / 2 / 34** = **71**, of which **64 are dead-rule candidates** (no reach path from the entry) ⇒ a `--lint-grammar` adjudication lane, not a witness-generation lane. Lanes cost 0.06–0.38 s and are seed-invariant. |
| — | `GRAMMAR-WELLFORMED.H.17.1` (repair the 5 live regex-look-around rules) | **`done`** (`PGEN-GRAMMAR-WELLFORMED-0161`, CODE / grammar + codegen) | ✅ All five repaired; `uncompilable_regex_terminals` `ebnf` 3→**0**, `semantic_annotation` 2→**0**, both exit 1→**0**. `/* x */` now parses. Cert: `ebnf` `144/0/109/35 → 144/0/111/33` (`spf` 13→8), `semantic_annotation` `114/0/80/34 → 115/0/82/33`. ⭐ `UNKNOWN` delta attributed **by rule name** — exactly `block_comment`+`block_comment_content` and `multiline_string`; **nothing** newly UNKNOWN. ⛔ 3 of the 5 stay UNKNOWN **correctly** — their parents are unreferenced, and a terminal repair cannot confer reachability. Generated parsers verified byte-identical to the interpreter (79/79, 134/134). Opened `H.20`. |
| 5 | `GRAMMAR-WELLFORMED.H.20` (the envelope gate is RED at HEAD on `systemverilog`, `155 > 151`) | **`todo`** (opened 2026-08-22 by `H.17.1`) | Found while measuring `H.17.1`'s blast radius; **pre-existing**, proven by a regenerate-the-parser two-arm control after the naive `git stash` control turned out to be a no-op for that instrument. Name the 4 new rows before touching the ceiling. |
| — | `GRAMMAR-WELLFORMED.H.17.2` (a `--lint-grammar` error class: every regex terminal must compile) | **`done`** (`PGEN-GRAMMAR-WELLFORMED-0159`, CODE / engine-universal) | ✅ `uncompilable_regex_terminals` is a hard error class. Checks **does it COMPILE**, not *does it contain `(?`* — a spelling heuristic is unsound (`(?i)`, `(?s:.)`) AND incomplete (backreferences). `ebnf` `0/exit 0 → 3/exit 1`. ⭐ Independently reproduced `H.17`'s grep census from a disjoint code path over all 12 grammars — leg 2, earned. |
| — | `GRAMMAR-WELLFORMED.H.17` (`spf>0` root cause) | **`diagnosed`** (`PGEN-GRAMMAR-WELLFORMED-0158`, doc+artifact tier) | ⛔ **NOT a generator defect — the leaf's own title was wrong.** Rust's `regex` crate does not support look-around, so the terminal never compiles and the rule matches nothing, ever. The `ebnf` meta-grammar **cannot parse ANY block comment**, `/* x */` included. Both engines agree to the `furthest_position`. Fix owned by `H.17.1`/`H.17.2`. |
| — | `GRAMMAR-WELLFORMED.H.18` (the cert-failure LABEL is blind for 9 of 13 registry rows) | **`done`** (`PGEN-GRAMMAR-WELLFORMED-0157`, CODE / registry-only) | ✅ The duplicate table was **DELETED, not filled in** — `parse_detail` had exactly ONE reader, so `parse_error()` now delegates to the single dispatch and the divergence cannot recur. `ebnf` labels `5 → 0` false / `0 → 5` real `furthest_position=`; all ten cert tuples byte-identical ⇒ the label moved no classification. |
| 4 | `GRAMMAR-WELLFORMED.H.19` (a doctrine that WATCHES "every register family is cert-WIRED") | **`todo`** (opened 2026-08-22 by `H.15`) | Leg 3 of the claim-verification bar for `H.15`'s `10/10 wired`, NAMED rather than skipped. Both inputs are tracked text ⇒ no cargo, no parser run, cheap always-on tier. |
| — | `GRAMMAR-WELLFORMED.H.15` (wire cert-coverage for `ebnf` / `return_annotation` / `semantic_annotation`) | **`done`** (`PGEN-GRAMMAR-WELLFORMED-0156`, CODE / registry-only) | ✅ **WIRED conjunct MET — 10/10 register families measurable**, ZERO grammar/codegen/generated bytes. ⛔ **And measuring it REFUTED the gate**: the three hid **71 `UNKNOWN`**, so `SVPP-EXPANSION`'s *WIRED + clean + `UNKNOWN`=0* is **NOT met** and never was. Seven wired families byte-identical, seeds 0/7/42 deterministic. Routed out `H.16`/`H.17`/`H.18`/`H.19`. |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.3` (close the 3 canonical reach-gaps → SV `fully_certified` via the union) | `active` (`.8.3.1` ✅ `-0146` CODE; `.8.3.2` remaining) | The **director-reaffirmed literal-`UNKNOWN=0` goal** (chosen over the parked `.8.5` accounting lane). After `.8.3.1`: canonical `UNKNOWN 22 → 20`, sound 4-config union `3 → 1`; ONE reach-gap (`context_member_method_call`) remains between SV and `fully_certified`. |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.3.1` (close the 2 `…scoped_call…` cousins — branch-1 longest-match grammar-gate) | `done` (`PGEN-GRAMMAR-WELLFORMED-0146`, CODE / released-SV; release `1.0.151`, ledger `SV-0013`, schema `6`) | Tool-proven via `--trace-rules class_scoped_call_prefix`: branch 1 `scoped_class_scoped_call_prefix_identifier` (gated only `lacks_class`) longest-matched `IF::m`/`T::m` as `<pkg>::<class>` (14 bytes) and shadowed cousins #3/#4 (5 bytes); added AND-stacked `lacks(interface_class)`+`lacks(type_parameter)` ⇒ both witness via `class_scoped_tf_call`. Canonical `22 → 20`, union `3 → 1` (residual = `context_member_method_call`), deterministic seeds 0/7/42, `spf=0`; 6 fully-certified grammars byte-identical; SV corpus 14/14; `cargo test --lib` 739/0; clippy source-clean. Detail: [GRAMMAR-WELLFORMED-H12831-scoped-call-cousins-grammar-gate.md](GRAMMAR-WELLFORMED-H12831-scoped-call-cousins-grammar-gate.md). |
| 2 | `GRAMMAR-WELLFORMED.H.12.8.3.2` (close `context_member_method_call` — store-gated declaration-hosting carrier) | **`done`** (delivered by tree `STRUCTURED-WITNESS-SYNTH` leaves `.3`/`.4`, `PGEN-STRUCTURED-WITNESS-SYNTH-0004`/`-0005`, 2026-07-22) | ⭐ CLOSED: the dedicated structured-witness COMPOSITION pass `generate_structured_witnesses` (PASS 3f — the `.4b.18`/`.4b.19` proven parts composed into ONE plan: pass-scoped dotted-emit producer admission with lexical-terminator-preserving name resolution + typed-branch forcing on the prelude sub-path + the consumer head-leaf pin + target-own directives; parser sole judge) witnessed the rule — canonical `UNKNOWN 12→11`, union `1→0`, residual `[]`, seeds 0/7/42, `spf=0` ⇒ **SV recognized `fully_certified`** (`sv_cert_recognized_union_gate` green at the re-baselined contract). |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.5` (recognize the SV multi-config cert-union as the `fully_certified`-accounting basis) | `active` (`.8.5.1` ✅ `-0144` DESIGN; `.8.5.2` ✅ `-0147` IMPLEMENT — gate SHIPPED + green; `.8.5.3` deferred; `.8.5.4` ✅ `-0148` stage-log `prune_log` port — the ~5 GB per-run `focus_systemverilog` scratch log now auto-prunes to a 2000-line tail in the union gate + the rce cert gate, both gates re-run GREEN) | The director-chosen **cert-accounting vehicle** (UNBLOCKED, picked over the `STORE-AWARE-GEN.4b`-blocked `.8.3` at the post-`-0143` fresh-session fork). The union *mechanism* is built (`-0141`); `.8.5.2` made it a **re-runnable, regression-locked recognized basis** (`sv_cert_recognized_union_gate`, canonical `UNKNOWN=20` / union `UNKNOWN=1`), not just live-tracker prose. |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.5.1` (reproduce + recognition audit + regression-lock DESIGN) | `done` (`PGEN-GRAMMAR-WELLFORMED-0144`, PURE-DOCS) | Reproduced the union oracle (canonical `UNKNOWN=22`, recognized 4-config union `witness=1300 UNKNOWN=3 spf=0`, residual = EXACTLY the 3 `.8.3` reach-gaps; seeds 0/7/42 deterministic); audited the recognition gap (no cert-coverage gate exists; SV `Done` = 7 closure criteria, NONE cert-coverage); designed `sv_cert_recognized_union_gate` (script + contract JSON + Make/CI, modeled on `sv_formal_exhaustive_closure_gate`) + recognition lockstep; chose standalone-oracle (A) over family-status-criterion (B, deferred) over docs-only (C, rejected). Owns IMPLEMENT `.8.5.2`. Detail: [GRAMMAR-WELLFORMED-H1285-recognized-cert-union-basis-design.md](GRAMMAR-WELLFORMED-H1285-recognized-cert-union-basis-design.md). |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.5.2` (IMPLEMENT `sv_cert_recognized_union_gate` + recognition lockstep) | `done` (`PGEN-GRAMMAR-WELLFORMED-0147`, CODE / proof surface only) | Turned the recognized cert-union basis from live-tracker prose into a re-runnable, deterministic, regression-locked oracle. Toolbox-first re-run pinned the **current** numbers (the `.8.5.1` design's `UNKNOWN=3` was stale post-`.8.3.1`): canonical `total=1304 proof=1 witness=1283 UNKNOWN=20`, union `witness=1302 UNKNOWN=1 spf=0`, residual `["context_member_method_call"]`, byte-identical seeds 0/7/42. Landed `rust/scripts/sv_cert_recognized_union_gate.sh` (modeled on `sv_formal_exhaustive_closure_gate.sh`) + tracked contract `systemverilog_recognized_cert_union_contract.json` + `make` target + help + CI YAML; recognition lockstep (book / SV contract / live-docs). Gate exits 0 ("✅ …passed"); NO engine/grammar/generated change ⇒ 6 fully-certified grammars byte-identical by construction. Detail: [GRAMMAR-WELLFORMED-H12852-recognized-cert-union-gate-implement.md](GRAMMAR-WELLFORMED-H12852-recognized-cert-union-gate-implement.md). |
| — | `GRAMMAR-WELLFORMED.A1a` | `done` (`-0154`) | Shadowing now a hard gate; SV well-formed re: dead branches; embodies "a well-defined EBNF has no unreachable rules". |
| — | `GRAMMAR-WELLFORMED.A1a.1/.2` | `done` (`-0002`) | regex + semantic_annotation shadows cleaned → ALL authored grammars pass the shadowing hard gate. |
| — | `GRAMMAR-WELLFORMED.A1b` | `done` (`-0003`) | Structural unreachability now a hard, multi-entry-safe gate; all grammars =0. The headline "no unreachable rules" is enforced. |
| — | `GRAMMAR-WELLFORMED.E1` | `done` (`-0004`, satisfied by construction) | Attribute non-circularity holds structurally (synthesized-only annotation language). |
| — | `GRAMMAR-WELLFORMED.B1` | `done` (`-0005`) | Deterministic step-budget replaces the wall-clock deadline → residual = 84 IDENTICAL across two runs (the ±25 noise gone). The literal-0 metric is now signal. |
| — | `GRAMMAR-WELLFORMED.A2` | `done` (`-0006`) | Sound subset of FIRST-domination — earlier-ALWAYS-SUCCEEDS shadowing. 0 false positives; found 52 real SV dead branches (warning-staged). General unsound FIRST-domination deliberately excluded. |
| — | `GRAMMAR-WELLFORMED.E2` | `done` (`-0007`, satisfied by existing validation) | `$N` attribute completeness already enforced (`E_RET_POS_OUT_OF_RANGE`, hard under strict mode, test-locked); consulted-fact completeness → F1. |
| — | `GRAMMAR-WELLFORMED.F1` | `done` (`-0008`, HARD GATE) | Binding-before-use (Jim 2010) — consulted-but-never-emitted fact-KIND. 0 across all grammars (sound, zero FP). **⇒ the well-DEFINEDNESS layer (E1/E2/F1) is COMPLETE; the linter now proves all 7 contract axes' decidable cores.** |
| 1 | `GRAMMAR-WELLFORMED.A2.1` | `closed` (superseded by `A2.1-SOUNDNESS`; hard-gate-promotion goal RETIRED) | The LRM-grounded fixes ✓ boolean-abbrev (`-0010`), ✓ covergroup-range + rs-prod (`-0013`), ✓ formal-type/port-reorder + list-of-arguments + module-path + bins_or_empty + class_declaration (`-0014`) STAND (real dropped-delimiter extraction bugs). What is retired is the "promote `EarlierAlwaysMatches` to a hard gate" goal — the check is UNSOUND for PGEN's backtracking engine (`A2.1-SOUNDNESS`). The residual 8 warnings were FALSE POSITIVES (proven-live port-header/net-type branches), not dead branches to clean. |
| 1 | `GRAMMAR-WELLFORMED.A2.2` (retire the unsound `EarlierAlwaysMatches` verdict + demote to a non-verdict note) | `done` (`PGEN-GRAMMAR-WELLFORMED-0150`, CODE / linter-soundness engine fix; ZERO grammar/parser regen) | Removed the `EarlierAlwaysMatches` shadowing verdict + its bogus `EarlierArmAlwaysSucceeds` certificate; added a non-verdict `AlwaysSucceedsAlternative` `[note]`. SV `--lint-grammar` now reports `always_succeeds_alternatives=8 (note)` (was `always_matches_shadowing=8 (warning)`), `ordered_choice_shadowing=0` unchanged, rc=0; the proven-live `net_port_type_sv_2017 #2` interconnect branch is no longer branded dead. Cert-coverage byte-identical (`1343/10/1321/12` seeds 0/7/42 — shadowing certs aren't in the rule-level proof pool); `cargo test grammar_wellformedness` GREEN; 6 fully-certified grammars byte-identical. |
| 1 | `GRAMMAR-WELLFORMED.A2.3` (`FixedTerminalPrefix` verdict + certificate made branch-policy-aware) | `done` (`PGEN-GRAMMAR-WELLFORMED-0151`, session #51, CODE / linter-soundness engine fix; ZERO grammar/parser regen — regen byte-identity proven) | The OTHER "PEG commits" verdict live-proven FALSE under the DEFAULT `longest_match` policy by PARSE-HARNESS.8 (engine `🏁 selected branch 2/2` on `a\|ab` while lint hard-failed rc=1). Now fires ONLY under `@branch_policy: ordered` with no branch-phase predicates; the `FixedTerminalPrefixBy` certificate re-derives the policy condition (a policy-false certificate is REJECTED). Policy derivation = NEW shared `effective_rule_branch_policy` (codegen delegates to it — single source of truth). ADDRESSED: default-policy probe rc=1→rc=0; ordered probe keeps rc=1 with the corrected message. NO-REGRESSION: 13/13 shipped lint sweep at 0; `sv_cert_recognized_union_gate` GREEN seeds 0/7/42; stash A/B cert byte-identical; json+rtl_frontend regen `cmp` byte-identical; lib 807/0; v2005 conformance gate GREEN; clippy source-strict; both book gates. LOCKSTEP: top book + ebnf book (fixed the real "`\|` commits to first match" drift) + TOOLBOX §5.1 + decision record `project_fixed_terminal_prefix_policy_conditional`. Sibling audit `A2.4` (duplicate tie-break under right/nonassoc) logged, not acted. |
| 1 | `GRAMMAR-WELLFORMED.A2.4` (`DuplicateAlternative` verdict + `DuplicateOf` certificate made selection-semantics-aware) | `done` (`PGEN-GRAMMAR-WELLFORMED-0152`, session #52, CODE / linter-soundness engine fix; ZERO grammar/parser regen — regen byte-identity proven) | The LAST unconditioned ordered-choice deadness verdict, closing the A2.2/A2.3/A2.4 arc. Protocol D probes on `scratch := "a" \| "a"` live-proved the engine SELECTS the "dead" later twin under `@associativity: right` (R), a later-higher `@priority` (P), and `@deterministic_group` rotation (D) — each engine `🏁 selected branch 2/2` while lint hard-failed rc=1 — and under `@associativity: nonassoc` (N) the equal-priority tie FAILS the whole choice (twins REJECT while the dedup control ACCEPTS), so "merge or remove" would change acceptance. FIX: new `RuleSelectionSemantics::duplicate_verdict` conditions the verdict on the effective `@associativity`/`@priority`/`@deterministic_group`/branch-predicate surface via NEW shared `effective_rule_associativity`/`effective_rule_branch_priorities`/`effective_rule_deterministic_partition_policy` (codegen delegates — single source of truth); new `DuplicateAlternativeNonassocTie` reason ("restructure deliberately"); `DuplicateOf` + `FixedTerminalPrefixBy` certificate checks route through the resolver (A2.4-D also requires no partition rotation for the fixed-prefix verdict). ADDRESSED: R/P/D rc=1→rc=0, N keeps rc=1 with the truthful tie message, defaults+ordered keep rc=1. NO-REGRESSION: lib 695/0 (+3 tests, 48/48 wellformedness); shipped shadowing sweep unchanged (12 at 0, `profiled_generated` 23 grep-proven no conditioning annotations); ALL 4 policy carriers byte-identical (json/rtl_frontend/systemverilog/regex `cmp` clean); `parse_harness_combinator_gate` 2/2; `sv_cert_recognized_union_gate` GREEN (UNKNOWN 12/union 1, seeds 0/7/42); clippy source-strict; mdbook gate. LOCKSTEP: top book grammar-wellformedness chapter + TOOLBOX §5.1 & Protocol D + decision record `project_duplicate_alternative_selection_semantics_conditional`. |
| 1 | `GRAMMAR-WELLFORMED.G` | `in-progress` (G.1 done `-0012`) | **The CERTIFYING LINTER** — make every verdict carry a checkable certificate (witness/proof), build the independent checker, drive `UNKNOWN`→0 on SV. "Verified, not trusted." ✓ G.1 certificate model + independent re-checker for unreachability proofs (round-trip + tamper-rejection tested). NEXT: G.2 standalone checker + extend certs to all `dead` checks; G.3 generator witnesses; G.4 coverage gate. |
| 1 | `GRAMMAR-WELLFORMED.H` (Phase H per-grammar cert-coverage) | `in-progress` (all SHIPPED grammars wired) | Wire `parse_and_cover` for every grammar so `--report-certificate-coverage` runs per-grammar. ✓ H.1 regex (`-0034`, UNKNOWN residuals + 6→3 witness-parseability), ✓ **H.2 vhdl (`-0041`, cert-coverage runs at default depth; zero-drift checkout-illusion proof discharged the staleness fear — `total=217 witness=132 UNKNOWN=85 sample_parse_failures=0` @ seed 0)**, ✓ **H.3 json (`-0037`, `fully_certified=true` — the FIRST grammar fully certified via Phase H)**, ✓ **H.4 rtl_const_expr (`-0038`, cert-coverage runs; zero-drift regen proof retires the H.2 mtime-staleness fear)**, ✓ **H.5 svpp (`-0039`, cert-coverage runs at default depth)**, ✓ **H.6 rtl_frontend (`-0040`, cert-coverage runs at default depth)**. **MILESTONE: every SHIPPED parser grammar now runs under cert-coverage** (json/regex/rtl_const_expr/svpp/rtl_frontend/systemverilog/vhdl); only meta/annotation grammars (`ebnf`/`return_annotation`/`semantic_annotation`) remain unwired. ✓ **H.5.1 (`-0042`) LABELED the svpp residual + H.5.1.1 (`-0044` investigation / `-0045` fix) ROOT-CAUSED + FIXED it: surgical whitespace-only greedy-tail guard in `regex_tail_greedy_blocker` → svpp `sample_parse_failures` 24→8, `UNKNOWN` 54→7, `witness` 19→66; zero cross-grammar regression (cross-family gate PASS).** ✓ **H.5.1.2 (`-0046`) drove svpp residual-8 CLASS (a) — the `\b`-keyword↔word-char directive-keyword fusion — to 0 via the declarative `[>! /\w/]` lexical-annotation (the construct built for the generator) + a general `collect_rule_body` frontend fix it surfaced (consecutive `[>` directives now each bind; only the first bound before); svpp `sample_parse_failures` 8→1, `UNKNOWN` 7→4, `witness` 66→69 seed 0; json/regex cert-coverage unchanged; lib 621/621; cross-family gate PASS.** ✓ **`H.5.1.3.2` (`-0049`) CLOSED the LAST svpp residual** — `condition_text -> $text` (declarative atomicity, LEXICAL-ANNOTATIONS.6) suppresses the stray trailing `\n` that stranded a `` `" `` stringize; svpp cert-coverage `sample_parse_failures` **1→0** (both seeds, deterministic) ⇒ **svpp is now cert-coverage CLEAN**. Consumer-visible: svpp schema **3→4**, release **1.0.4→1.0.5** (condition_atom "text" body raw-envelope→`$text` string; annot 66→67; director-approved). ✓ **`H.4.1` (`-0050`) ROOT-CAUSED rtl_const_expr's `UNKNOWN` residual** (the FIRST per-grammar `UNKNOWN`→0 drive, tools-first, pure-docs): the residual reduces to the stubborn pair `lparen`/`rparen` = the `primary_expr := lparen conditional_expr rparen` parenthesised-primary branch, which clean diverse generation essentially NEVER selects (`0/40` samples contain `(` @ depth 32; the branch re-enters the ~15-deep precedence chain → depth-floor pruning + recursion-pressure penalty avoid it; fatal-aborts at the default depth 24). ADJUDICATED a **generator-reach deficiency** (statically reachable; `(1)` is valid) — fix belongs in the generator. The witness-pass shortcut is off the table per the explicit `main.rs:1572` design decision. ✓ **`H.4.2` (`-0051`) DONE — CONSTRUCTIVE-REACH: rtl_const_expr is now `fully_certified=true` (UNKNOWN 3→0, deterministic across seeds), with ZERO certification regression on any grammar** (decisive git-stash baseline: `sample_parse_failures` byte-identical pre/post for json/regex/vhdl/SV; UNKNOWN only decreases — regex 101→98, vhdl 85→69, SV 1160→1126). Opt-in `StimuliConfig.reach_uncovered_recursive_branches` (default OFF → all non-cert-coverage surfaces byte-identical) drives three gated `generate_or` behaviours (floor-retain + try-recursive-first + minimal-`construct_mode` depth-retry); `run_certificate_coverage_report` is two-pass (diverse certification pass byte-identical + auxiliary reach pass that only UNIONS re-parsing witnesses). lib 686/0; new test PASS; self-host + cross-family + oracle green. ✓ **`H.5.2` (`-0052`) drove svpp `UNKNOWN 3→2`** — removed the OBJECTIVELY-PROVEN-DEAD `trivia` rule (referenced by nothing; gap-report oracle `reachable:false unreachable_from_entry`; the only statically-unreachable rule) at source per the literal-0 doctrine, and tightened the `sv_preprocessor_zero_plausible_gap_proof_gate` from a `[trivia]` helper-pocket to a **literal-ZERO unreachable surface** (contract v2→3, observed==allowed==[]; gate GREEN). cert-coverage `total 72 witness 70 UNKNOWN 2 sample_parse_failures 0` (seeds 0/7); shape-contract GREEN (no AST/schema/release change); lib 716/0. svpp's remaining 2 `UNKNOWN` (`directive_tail`/`line_comment`) are reachable optionals = generator-reach → **`H.5.3`** (svpp fully_certified after it). ✓ **`H.5.3` (`-0053`) DONE — svpp `fully_certified=true` at seeds 0/7/42** via DECLARATIVE witnessing-sample steering (evidence-driven re-scope from the assumed constructive-reach engine pass): the 2 residuals were 100% generator-side, caused by stale `@sample: " "` hints (un-witnessable bare space / `line_comment?`-short-circuit), replaced with witnessing-and-faithful `@sample: " x"` / `@sample: " //"`; `sample_parse_failures=0`, deterministic, zero cross-grammar regression (grammar-only). Multi-seed measurement surfaced TWO pre-existing svpp residuals (the stimuli generator as bug-finding oracle): (1) a seed-1/12-only macro-default nested-optional UNKNOWN — but a SAMPLE-BUDGET artifact (count 100/200 → `UNKNOWN=0`) → ticketed **`H.5.4`** (low priority); (2) a genuine OVER-GENERATION (`sample_parse_failures=1` at seeds 3/6/10/12/14/15 of 0–15, IDENTICAL on the pre-H.5.3 grammar → pre-existing, an unclosed/closer-stolen `pp_conditional` round-trip hazard) → ticketed **`H.5.5`** (the real round-trip defect). ✓ `H.5.5` + `H.5.4` closed (svpp seed-robustly clean via `H.5.5`/`H.7.2`/`H.9`). ✓ **`H.10.1` (`-0060`) — the regex `UNKNOWN`→0 drive's dead-rule removal: the 12 no-path rules (both oracles: `unreachable_from_entry`) removed at source; regex `UNKNOWN 19→7`, `total 210→198`, spf=0, deterministic seeds 0/7/42; gap-report unreachable 12→0; oracle gate + lib 721/0 green; no release/schema bump.** ✓ **`H.10.2.1` (closed via `BRANCH-BROADCAST-FIX.5`, regex `UNKNOWN 7→5`).** ✓ **`H.10.2.2` (`-0061`) ENGINE FIX — memo-hit coverage-delta replay (the memoization × coverage-record composition gap, the `.36.4` class on the witness record): regex `UNKNOWN 5→3` + CROSS-GRAMMAR vhdl `31→30`, rtl_frontend `75→73`, SV `738→647` (−91), spf byte-identical 0 everywhere, oracle gate PASS, NO bump.** ✓ **`H.10.2.3` (`-0062`) unicode_char witnessed via the declarative `@sample: "é"` (the rule was ungeneratable — `builtin_any_char` has no grammar def + no generator special-case; the engine capability ticketed in STIMULI-SIGNOFF): regex `UNKNOWN 3→2`, spf=0, seeds 0/7/42, oracle PASS, NO bump.** NEXT = the regex store-gated pair `numeric_backreference`/`backreference_digits` (B2/C1/C2 fact-emitting-prelude lane) + the vhdl (30) / rtl_frontend (73) / SV (647) drives. |
| — | `GRAMMAR-WELLFORMED.C2` | `done` (`C2.1` design `-0063` + `C2.2` implement `-0064`) | **Semantic-prelude reach (count-gated MVP) LANDED — regex is the 4th `fully_certified` grammar (`UNKNOWN 2→0`, witness 198, spf=0, seeds 0/7/42 × counts 1/40/200, deterministic).** Cross-grammar byte-identical; oracle + cross-family + mdbook gates PASS; no regen/bump. |
| 1 | `GRAMMAR-WELLFORMED.H.11` (the vhdl `UNKNOWN`→0 drive) | `done` (`H.11.1` ✅ `-0065`; `H.11.3` ✅ `-0067` — ENGINE FIX, vhdl release `1.0.4`, ledger `VHDL-0002`; `H.11.2-FIX` ✅ `-0072`; `H.11.4-UNPARK` ✅ `-0073` — 🏁 vhdl FULLY-CERTIFIED, the vhdl drive COMPLETE; **`H.11.5-FIX` ✅ done (`PGEN-GRAMMAR-WELLFORMED-0075`, 2026-06-11, ENGINE FIX, consumer-visible — SV release `1.0.138`→`1.0.139`, schema STAYS 3, ledger row `SV-0001`, contract + SV book + top-level book lockstep): the `-0074`-designed static emit-time per-introducer comment-arm suppression LANDED with one decisive-A/B-driven refinement.** IMPLEMENTATION (`ast_based_generator.rs`, parser-agnostic): `generate_helper_methods` now takes the grammar tree and computes, per introducer (`#`, `//`, `/*`), whether ANY terminal assigns it a NON-COMMENT meaning — regex terminals via a HIR **mandatory-prefix** walk (`hir_mandatory_prefix`: every match must START with the introducer; a content class like `[^\r\n]*` that merely CAN start with it is NOT a claim) minus comment-DEFINING patterns (mandatory introducer head + unbounded tail = SV `line_comment`/`block_comment`); literal terminals via starts_with minus the two-token comment shape (introducer literal immediately followed by an unbounded content terminal, one-hop rule deref — the ebnf `("#"|"//") comment_content` idiom). A claimed introducer's arm is NOT EMITTED in either skipper (`consume_layout_for_regex` AND `consume_layout_for_terminal`); when all three are claimed the skippers reduce to whitespace and the dynamic-guard helper is elided. RESULT per grammar: SV/vhdl/rtl_frontend lose the `#` arm (SV `hash := trivia "#"`, vhdl `/#/`, rtl_frontend `#` overrides); ALL other arms/grammars byte-equivalent status quo. ⚖️ REFINEMENT ADJUDICATION (tools-first, decisive stash A/B): the leaf's original "any claim suppresses" criterion REGRESSED the generated-ebnf-parser dual-run flow (pre-fix baseline 1 failing flow [regex `**`-gap, pre-existing] with ebnf consumed_pct=100; all-claims-suppress → 2 failing flows, ebnf 15.32% — the generated ebnf parser relied on the arms for mid-rule comments `ebnf.ebnf` does not structurally own, e.g. trailing comments inside `rule_operator`'s alternation; ALSO `comment_content := /([^\r\n]*)/` itself can-match-prefix-claimed every introducer) ⇒ the criterion was refined to non-comment MEANING (mandatory prefix, comment-defining exemption); post-refinement the dual-run gate is byte-identical to its pre-existing state (1 flow, regex only, ebnf 100%/pass). VERIFIED (full matrix, fresh regen of ALL 10 parsers via the canonical order): y4/y2/y5 repros REJECT→PASS + all 10 controls PASS + the 618-byte canonical sample PASSES; SV cert-coverage seed-0 spf **1→0** (seeds 0/7/42, witness/UNKNOWN byte-identical 697/645); SV external corpus triage gate clean; vhdl fully_certified seeds 0/7/42 + 16-seed sweep `[0×16]` + external corpus 8/8; svpp 32/32 seeds fully_certified spf=0; json 9/9, regex 198/198, rtl_const_expr 48/48 fully_certified; rtl_frontend 99/71 byte-identical (seed-42 spf=1 stays the pre-existing ticket); rtl_frontend contract gate red with the IDENTICAL pre-existing signature; oracle + cross-family gates PASS; default lib 646/0 (+4 analysis locks), dual-feature 741/0 (+`systemverilog_hash_token_is_not_stolen_by_comment_arms_during_speculation` in `ast_shape_contract.rs`); clippy strict-source clean (generated stage = pre-existing tolerated debt); mdbook + SV book gates PASS. Repro corpus retained `/tmp/h112_bisect/*` + `/tmp/h115_f0.sv`. The `VHDL-0002` ledger note's generator-side adjudication superseded in-row (→ `SV-0001`).** THE ENTIRE `H.11` ARC IS CLOSED.) | **vhdl-30 ROOT-CAUSED (tools-first, 2026-06-10):** probe capture (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, count 40 seed 0) showed the dominant family is a LEXICAL-FAITHFULNESS engine defect — reach-pass probes fuse a multi-token regex-terminal render's tail with the following keyword (`8 mintO`, `msThEn`, `nsaNd`), so the probes never re-parse. WHY+WHERE pinned: `physical_literal := trivia /[0-9]…[ \t\r\n]+(?:fs|ps|…)\b/` (`grammars/vhdl.ebnf:422`) renders MULTI-TOKEN text (`8 min`); `apply_word_boundary_spacing` correctly defers its trailing-`\b` separator to the join rule but recorded the tail state with the WHOLE-STRING `is_word_shaped_literal` (`"8 min"` has a space ⇒ `false`) ⇒ `append_generated_segment` never separated ⇒ fusion. The minimal `trivia := (…)*` ⇒ empty in construct mode explains why ONLY reach probes hit it (diverse spf=0 at seed 0). THIRD instance of the `H.8` Defect-A class — hints were fixed with the tail-aware helper; regex-terminal renders were not. **`H.11.1` — `done` (`PGEN-GRAMMAR-WELLFORMED-0065`, ENGINE FIX): the 3 `apply_word_boundary_spacing` recording sites switched to the tail-aware helper (renamed `literal_hint_tail_word_shaped` → `tail_word_shaped` — it now serves hints AND multi-token terminal renders); token-literal site untouched (no failing evidence — targeted fix). VERIFIED: vhdl cert-coverage `UNKNOWN 30→4` (witness 187→213), the ENTIRE fusion family closed, residual EXACTLY the predicted [based_literal, based_value, hash, white_space] — IDENTICAL at seeds 0/7/42 (pre-fix the residual seed-wobbled 30/31); regex STAYS `fully_certified` (198/198, spf=0, seeds 0/7/42); json/rtl_const_expr/svpp stay `fully_certified`; rtl_frontend 73 + SV 647+68-no-path byte-identical, spf=0; new lock `multi_token_regex_terminal_tail_separates_from_following_keyword`; default lib 639/0, dual-feature 732/0; oracle + cross-family gates PASS; mdbook gate PASS (lexical-annotations chapter: new "Multi-token regex terminals participate the same way" section); clippy strict-source clean; NO regen/bump (generator-only).** Ticketed sub-leaves: `H.11.2` = the vhdl seed-7 diverse-pass `spf=2` — **PROVEN PRE-EXISTING via decisive git-stash baseline (pre-fix binary: seed-7 spf=2 IDENTICAL)**, an over-generation residual to root-cause (the H.5.5 pattern; plausibly the same `H.11.3` parser bug seen from the diverse side — check the failing samples for based literals first); `H.11.3` = **RE-ADJUDICATED 2026-06-10 (`PGEN-GRAMMAR-WELLFORMED-0066` investigation, tools-first): NOT generator over-generation — a vhdl GENERATED-PARSER BUG (🚨 released-family, highest priority per the director's fix-parser-bugs-ASAP principle). The parser REJECTS VALID VHDL based literals.** Decisive bisect on the exact probe shell (`…SignAL I:y:=<X>;…`): `X=3374` PASSES, `X=2#1010#` REJECTS (valid VHDL!), `X=3374#BFCd#` REJECTS, while `X=16x"AB"` PASSES (so the literal tournament is NOT naive first-match shadowing — bit_string wins over the decimal prefix, refuting the initial lost-ordering hypothesis too). WHERE (rule-scoped trace `--trace-rules based_literal,hash,based_value`): inside `based_literal@40`, `unsigned_number` exits `40→41` ✓, then `hash`'s `trivia` ENTERS branch 1/2 "at position 41" but the `white_space` regex inside it EXECUTES at position **58 = EOF** ("no match … next: '<EOF>'") — the parser position JUMPS `41→58` between the branch entry and the rule body with no trace in between, so the open `#` is sought at EOF and `based_literal` fails on every input. Suspect classes (NOT yet pinned — next step is a deeper diagnostic per [[feedback_why_and_where_before_solution]], NOT a fix): (a) a multi-branch tournament arm evaluating with a stale/unrestored `parser.position` from a previous candidate (the branch-entry message prints the saved start while the live position differs — instrument or read the emitted tournament arm prologue); (b) a memoized_call interplay restoring a wrong cached end. Next steps: minimal repro (`/tmp/vhdl_based_probe2.vhd` retained: lowercase variant + parenthesized variant), pin the emitted code path (read the generated `parse_trivia`/`parse_hash`/tournament arm in `generated/vhdl_parser.rs`), and an old-binary A/B (regenerate the vhdl parser from a pre-`BRANCH-BROADCAST-FIX-0003` checkout in a worktree) to bound the regression window — based_literal was already UNKNOWN at `H.2` wiring, but the PARSE REJECTION itself has not been historically bounded. Consumer-visible fix will need: vhdl release bump + ledger row + contract/book lockstep (accept-set widening).** **`H.11.3` — `done` (`PGEN-GRAMMAR-WELLFORMED-0067`, ENGINE FIX, consumer-visible — vhdl release `1.0.3`→`1.0.4`, schema STAYS 3, ledger row `VHDL-0002`, contract+both-books lockstep): WHY pinned by reading the emitted code — the generated layout skipper `consume_layout_for_regex` HARD-CODES `#`-to-EOL comment skipping (plus `//`, `/* */`) into every non-blocklisted generated parser (an EBNF meta-grammar convention; in VHDL `#` is the based-literal delimiter), and unlike its string-terminal sibling `consume_layout_for_terminal` (which has guarded `expected != "#"`… since inception) the regex side had NO introducer guard — so `hash := trivia /#/` had its own `#` eaten as a "comment" (decisive trace: the `/#/` regex executed at position 58=EOF after the skipper consumed `#1010#;…` to end-of-line; the -0066 41→58 'jump' was the skipper inside `parse_white_space`/`parse_hash`, NOT a tournament/memo defect — both -0066 suspects refuted). Regression window bounded via git -S: the `#` arm dates to `2b953f3b` 2026-02-20 — present in EVERY shipped vhdl release. FIX (parser-agnostic, `ast_based_generator.rs`): the three comment arms now stand down when the active token's own anchored pattern matches at the introducer (`regex_token_matches_at_cursor`, cold-path sibling cache); `consume_layout_for_regex` also moved INTO the `uses_match_regex` conditional (fully-literal parsers — regex — no longer carry it as dead code); all parsers regenerated. VERIFIED: probes `2#1010#`/`3374#BFCd#` REJECT→PASS, decimal control unchanged; vhdl cert-coverage `UNKNOWN 4→1` (witness 213→216; residual EXACTLY [white_space]=`H.11.4`), IDENTICAL seeds 0/7/42, seed-0/42 spf=0; cross-family: json 9/9, rtl_const_expr 48/48, svpp 74/74, regex 198/198 all stay fully_certified; rtl_frontend `73→71`, SV (sv_2017 canonical) `647→645` + same 68 no-path; oracle gate PASS; cross-family platform gate PASS; default lib 639/0, dual-feature 733/0 (+1 lock `vhdl_based_literal_hash_token_is_not_swallowed_as_comment`); clippy strict-source clean (generated stage = the pre-existing tolerated 191-site debt, unchanged).** ADJUDICATIONS + NEW TICKETS from this slice's sweep: (1) **`H.11.2` same-bug conjecture REFUTED** — the vhdl seed-7 diverse spf=2 persists post-fix and the two failing samples (822/867 bytes, captured) contain NO based literals (view/context/configuration-heavy soup) — it is a genuine separate over-generation residual (H.5.5 pattern), still open; (2) **NEW `H.11.5`**: SV canonical-seed-0 cert-coverage spf `0→1` — the failing sample (timeunits comment-soup, captured) previously re-parsed only because the old skipper blinded SV's explicit `trivia := (ws|line_comment|block_comment)*` (comments were eaten inside following tokens' implicit skips, letting `/`-tokens steal comment-opening slashes); with grammar-faithful comment ownership the parser CORRECTLY rejects the bytes (PEG-greedy trivia is the spec) ⇒ a pre-existing GENERATOR `/`↔comment-fusion lexical-faithfulness gap EXPOSED, not a parser bug — fix generator-side (the `[>!`-style follow-restriction / fusion-guard direction); (3) **PRE-EXISTING red gate (decisive stash baseline)**: `ebnf_frontend_dual_run_gate` fails on the regex flow because the GENERATED ebnf parser cannot parse the `**` flatten-spread (`grammars/ebnf.ebnf` `quantified_marker := ("*"|"+"|"?")` models single markers; `regex.ebnf` gained `[$1**]` with RGX-0074) — comment-free minimal repro fails IDENTICALLY on the pre-fix binary; meta-grammar lags the annotation language — needs its own leaf (ebnf.ebnf grammar fix); (4) **PRE-EXISTING red gate (decisive stash baseline)**: `rtl_frontend_generated_contract_gate` fails (`always_ff_well_formed` missing required rule `module_declaration`) IDENTICALLY on the pre-fix binary with a comment-free sample — broken by some earlier engine wave, needs its own leaf (RTL-FE lane). **`H.11.4` — INVESTIGATION DONE (`PGEN-GRAMMAR-WELLFORMED-0069`, pure docs — all code reverted; tree clean): (1) ADJUDICATED — `white_space` is ENGINE-SHADOWED-DEAD: 4 reach probes (` ;` etc.) all `parsed=true witnessed_target=false`; the emitted layout skipper consumes whitespace via `consume_optional_whitespace` before ANY non-empty-matchable regex executes, so a pure-whitespace-class rule can never match its own bytes ON ANY INPUT (engine-model fact, not a generator gap; the engine-side wake-up alternative was REJECTED — it would put whitespace nodes into trivia envelopes = wire-shape churn + schema bumps across 3 released families, for ONE rule). (2) The removal EXPERIMENT (`trivia := (line_comment)*`, provably accept-identical since the branch never matches) reached **vhdl UNKNOWN 1→0 `fully_certified=true` at seeds 0/7/42** — but a DECISIVE same-engine 16-seed A/B sweep showed it AMPLIFIES the pre-existing organic over-generation class ~2× (B=15 failures/16 seeds on the old grammar — H.11.2 was never a seed-7-only quirk, it is a PERVASIVE ~1/seed class; A=29/16 post-removal, every trivia iteration now renders a comment) ⇒ REMOVAL PARKED until the class closes (the edit is one line, kept in this row). (3) 🐛 NEW ENGINE BUG root-caused tools-first (a [WORD-JOIN-SKIP] probe at the single join choke-point): `last_terminal_word_shaped` (+ sibling `last_terminal_from_atomic_rule`) are NOT TRANSACTIONAL across DISCARDED render attempts — an Or-branch retry/tournament loser/quantifier discard leaves a stale flag, the concat tracker mis-reads a free word tail (`…'--7I]\nA`) as non-fusable, the join rule never fires ⇒ `A`+`gEneRATE`→`AgEneRATE` (single-space patch flips the whole 794-byte sample to PASS — decisive). The 4th H.8 Defect-A instance and the THIRD instance of the standing transactional-record engine rule (semantic delta `.36.4`, coverage delta `H.10.2.2`, now the word-shape flags). (4) FAILED CANDIDATE recorded (tried-X-failed-because-Y): deriving the tracker tail from the SEGMENT TEXT (`tail_word_shaped(generated)`) fixes the fusion (vhdl seed-0 spf 2→0; unit lock passed) but is semantically TOO BROAD — SV `UNKNOWN 645→688` (−43 witnesses) + spf 1→4, rtl_frontend spf 0→~9/seed (s0=8 s1=10) ⇒ REVERTED; the NEXT-SLICE design = transactional flags: capture the flag pair per Or-candidate and restore the CHOSEN candidate's pair on commit (entry pair on failure); restore at quantifier-iteration discard, closer re-roll, relational retry; lock = or-grammar(branch1 = comment-tailed render then failing ref; branch2 = word terminal) ⇒ post-Or flags must equal branch2's tail, NOT branch1's. (5) `@sample: "--x\n"` comment-canonicalization tested and REJECTED (failures are placement-class, not content-class — triple merely reshuffled). Corpora retained: `/tmp/h114_fail{0,1}.vhd`, `/tmp/h114_s{7,42}_f*.vhd`, micro-grammars `/tmp/h114_micro*.ebnf`, sweeps in the leaf. ⚠️ OPERATIONAL gotcha for the record: `ast_pipeline/mod.rs:492` SHADOWS `eprintln!` → `pgen_trace_debug!` crate-wide — lib-internal eprintln debugging is verbosity-gated and SILENT at default (cost an hour of phantom 'function never called' debugging; use `::std::eprintln!` or trace levels).** FRONTIER (re-ordered): **`H.11.2-CLASS`** — root-cause the pervasive organic over-generation — NOW THE BLOCKER for vhdl fully_certified. **CHECKPOINT `-0070` (2026-06-11, director-ordered STOP mid-investigation):** the TRUE-baseline 16-seed sweep (HEAD grammar + HEAD engine) measured `spf` per seed = [0,1,1,4,0,2,0,2,0,1,0,1,0,0,1,1] (TOTAL **14/16 seeds**, seeds 3 and 5/7 heaviest); 9 failing samples extracted to `/tmp/h112_s<seed>_f{idx}.vhd` (filename literal `{idx}` — an extraction-script quoting slip, one file per seed, later failures of a seed overwrote earlier ones; re-extract when resuming) with reject positions probed: failures cluster at DESIGN-UNIT boundaries — `architecture`/`package body`/`package` units rejecting at or near their start (s2@0 `PACKAGE Body G IS FunCTioN…`, s5@0 `ARCHITECTURE a5 Of Z6Vrd IS suBtYPE…`, s7@2, s9@3, s15@1) or at a mid-file second-unit start (s1@123 `;PACKAGE C iS tYpE SB4mk iS ArRa…`, s3@192 `…END wSX; arcHitEctUrE kOfsz Of Ge40 IS SignaL…`, s11@150 `…;; pAckaGE BODy s Is ImpurE FUNCtiON Z Re…`, s14@85 `…; pACKAGe vyt iS GEnERIC(…`) — i.e. the rejected construct is INSIDE a unit body (surface position = unit start per PEG error convention; furthest-position triage pending). **`H.11.2-CLASS` — INVESTIGATION COMPLETE (`PGEN-GRAMMAR-WELLFORMED-0071`, 2026-06-11, pure docs): ALL 14 FAILURES ARE ONE MECHANISM — the `-0069`-root-caused WORD-JOIN-SKIP engine bug (non-transactional `last_terminal_word_shaped`/`last_terminal_from_atomic_rule` across discarded render attempts). NO second mechanism exists; "fix per hierarchy" collapses to the ONE designed engine slice.** METHOD: HEAD rebuild (release, `ebnf_dual_run,generated_parsers`) → the 16-seed sweep reproduced the spf vector EXACTLY (`[0,1,1,4,0,2,0,2,0,1,0,1,0,0,1,1]`, deterministic); all 14 samples re-extracted with corrected naming (`/tmp/h112_s<seed>_f<idx>.vhd`); the 3 samples >2000 B exceed the cert-coverage failure-preview cap (2000-char preview, `main.rs` SAMPLE-PARSE-FAILURES block) and were recovered BYTE-EXACTLY from plain `--generate-stimuli --count 40 --seed N --entry-rule vhdl_file` output — **plain generate is PROVEN byte-identical to the cert-coverage diverse pass** (needle-match of full failing samples; a generally useful recovery fact). CLASSIFICATION (automated single-space fusion-site finder + H.11.3 probe-shell bisection for the 4 stubborn samples): **15 fusion sites / 14 samples — 13× `<identifier>`+`is`** (`FC.o|is` s2, `Gg|is` s7_f0, `CTx|Is` s3_f1, `Aa8mT|IS` s9, `DAN|IS` s14, `Dy09B|IS` s11, `uFnlZ|Is` s15, `e|IS` s3_f2, `b|is` s3_f3, `P|iS` s1, `F|iS` s5_f1, `rbqRJ|is` s5_f0, `CC_qc|is` s3_f0) **+ 2× `<attr-ident>`+`generate`** (`j|GenErATe` s7_f1, `H|GeneRAtE` s9 — the `-0069` `A|gEneRATE` shape); s9_f0 carries two sites. **EVERY site flips its whole sample REJECT→PASS with ONE inserted space — all 14 verified end-to-end.** The `is`-dominance corroborates the mechanism: `is` follows an identifier/selected-name whose render involves discarded attempts (`(dot identifier)*` quantifier discards, `constraint?` optional discards) that leave the stale non-word-tail flag, so the join rule skips the separator. ADJUDICATIONS: (a) the class is **COMMENT-INDEPENDENT** (s2_f0 has ZERO comments) — comment soup is incidental, confirming `-0069`(5)'s placement-class verdict from the content side; (b) two PEG-triage lessons: the fusion can sit far BEFORE the surface reject (early-unit-closure cascade — `Gg|is` re-associated the nested body's begin/end upward, closed the architecture early, stranded the tail at byte 297; same shape for `CTx|Is`), so search the WHOLE failing unit, never a window anchored at the reject position; and a multi-defect unit pins the surface position at the unit start, so single-fix progress is invisible without per-construct probe-shell bisection (s9_f0). Repro corpus retained: `/tmp/h112_s*_f*.vhd` (14 true samples) + `*_patched.vhd` PASS controls + `/tmp/h112_{sweep.sh,extract.py,classify.py,triage.sh}` + `/tmp/h112_bisect/`. **`H.11.2-FIX` — `done` (`PGEN-GRAMMAR-WELLFORMED-0072`, 2026-06-11, ENGINE FIX, generator-only — NO regen/release/schema bump): the transactional word-shape-flags slice CLOSES THE ENTIRE H.11.2-CLASS — 16-seed vhdl sweep spf `[0,1,1,4,0,2,0,2,0,1,0,1,0,0,1,1]` → `[0×16]` (14→0), vhdl UNKNOWN stays exactly 1 (the parked `white_space`), witness 216.** IMPLEMENTATION (`stimuli_generator.rs` only, parser-agnostic, UNGATED — every grammar needs it): the flag pair (`last_terminal_word_shaped`, `last_terminal_from_atomic_rule`) is captured at each choice-point entry and restored at every discard boundary, mirroring the STORE-AWARE-GEN checkpoint shape at the SAME three loops — (1) `generate_or` attempt loop (restore at attempt top + at the `Err` arm start so the depth-slack/constructive-reach retries start clean + before the final-failure return; the recovery-fallback return now records its own tail via the H.8 hint idiom instead of inheriting loser garbage), (2) `generate_quantified` repeat-count candidate loop (top-of-candidate + final-failure restores — THE dominant class shape: a discarded iteration's lone `.` from `(dot identifier)*` mis-described the kept tail when the kept `repeats=0` candidate rendered empty ⇒ `<identifier>`+`is` fused 13×), (3) the relational attempt loop (top-of-attempt + budget-exhaustion restores). The closer re-roll needed NO separate fix (each re-materialization records its own tail via `apply_word_boundary_spacing`; its discard path exits through the fixed boundaries). The simplification vs the `-0069` design sketch: winners RETURN immediately in all three loops, so "restore the chosen candidate's pair on commit" is automatic — only entry-pair restores at discard boundaries were needed. +3 lib locks: `discarded_or_attempt_restores_word_shape_flags` + `discarded_quantified_candidate_restores_word_shape_flags` (deterministic entry-pair restore on total failure) + `discarded_quantifier_iteration_does_not_fuse_following_keyword` (the end-to-end `xy`+`is` no-fusion lock, seed-swept 0–15). VERIFIED: default lib 642/0; 16-seed vhdl sweep 0/16 (acceptance); cross-grammar seeds 0/7/42 — json 9/9, regex 198/198, rtl_const_expr 48/48, svpp 74/74 all stay `fully_certified` spf=0 byte-identical; SV 1342/697/645 byte-identical (seed-0 spf=1 = the pre-existing `H.11.5`, untouched); rtl_frontend 99/71 byte-identical with seed-42 spf=1 **stash-proven PRE-EXISTING** (decisive A/B: pre-fix binary emits the byte-identical line — NEW small ticket: rtl_frontend seed-42 over-generation residual, own leaf when the rtl_frontend drive resumes); `regex_pcre2_compile_oracle_gate` PASS; `stimuli_cross_family_platform_gate` PASS; clippy strict-source clean; book lexical-annotations chapter synced same-wave (new "Discarded render attempts are transactional" section). The FOURTH transactional-record instance landed (semantic delta `.36.4`, coverage delta `H.10.2.2`, word-shape flags here) — the standing engine rule held exactly.** **`H.11.4-UNPARK` — `done` (`PGEN-GRAMMAR-WELLFORMED-0073`, 2026-06-11, vhdl GRAMMAR slice — accept-identical, NO release/schema bump): 🏁 the parked `white_space` removal LANDED — vhdl is the FIFTH fully-certified grammar (`total=216 witness=216 UNKNOWN=0 fully_certified=true spf=0`, IDENTICAL seeds 0/7/42) — THE `GRAMMAR-WELLFORMED.H.11` vhdl `UNKNOWN`→0 DRIVE IS COMPLETE (85→69→31→30→4→1→0).** EDIT (`grammars/vhdl.ebnf`): `trivia := (white_space | line_comment)*` → `trivia := (line_comment)*`; the orphaned `white_space := /[ \t\r\n]+/` removed at source (literal-0 doctrine, H.10.1/H.5.2 precedent — leaving it would trip the structural-unreachability hard gate); the now-dead 2-branch `@branch_policy`/`@priority: [32,1]` annotation pair removed with it; in-grammar comment records the engine-shadowed-dead adjudication. Accept-identity holds BY THE -0069 ADJUDICATION (the branch never matched on any input — the skipper consumes whitespace first) and is measured: vhdl parser regenerated (`focus_vhdl`), **16-seed sweep STAYS `[0×16]`** (the `-0069` 2×-amplification objection definitively retired — amplified exposure of a root-fixed bug is nothing), **`vhdl_external_corpus_triage_gate` clean (`primary_parse_failure_corpus: <none>`, 8/8 real-world files)**, dual-feature lib **736/0** (= the -0067 733 + the 3 `-0072` locks; based-literal lock + vhdl shape-contract green with the regenerated parser), `stimuli_cross_family_platform_gate` PASS, mdbook gate PASS, clippy strict-source clean. Wire-shape identical (white_space nodes never appeared in any AST) ⇒ schema stays 3, release stays `1.0.4`. Book lockstep: grammar-wellformedness chapter — the vhdl arc extended `…→ 1 → 0` (the parked-removal story + the fifth-fully-certified milestone) + the rolling-gate grammar list marks vhdl fully-certified. Cross-grammar blast radius: NONE by construction (vhdl-only grammar edit; engine untouched; other parsers not regenerated). FULLY-CERTIFIED ROSTER: json, rtl_const_expr, svpp (32/32 seeds), regex, **vhdl**. Locked-program remaining `UNKNOWN`: **rtl_frontend 71, SV 645 (+68 no-path, A2.1)**.** **`H.11.5` — RE-ADJUDICATED (`PGEN-GRAMMAR-WELLFORMED-0074` investigation, 2026-06-11, tools-first, pure docs): 🚨 NOT generator over-generation — a RELEASED SV PARSER BUG (the parser rejects grammar-valid input), the SECOND instance of the `-0067` skipper-comment-arm defect family, in the DUAL case the `-0067` guard does not cover.** The `-0067` adjudication ("`/`↔comment generator fusion; parser right to reject") is REFUTED by the deeper trace. MINIMAL REPRO (bisected from the canonical seed-0 618-byte failing sample, `/tmp/h115_f0.sv`): `interface i #  (  ) ;timeunit 09 ns//>Mg\n//QF.\n/633 s;endinterface` REJECTS (`/tmp/h112_bisect/y4.sv`; also y2/y5 variants) while every single-comment variant (x1/x2), the no-`#()` variant (x4/v4), and the no-second-comment variant (y1) PASS — the failing shape needs `#( )` in the header AND a line comment followed by another comment before the timeunit-ratio `/`. WHY (full-trace decisive, `/tmp/h112_bisect/y4_full.log`): while the parser speculatively attempts `line_comment` (a regex token) at position 11 (after `interface i`), the emitted layout skipper `consume_layout_for_regex`'s hard-coded **`#`-to-EOL comment arm** fires at the `#` of `# ( )` — the `-0067` guard (`regex_token_matches_at_cursor`) only stands the arms down when the ACTIVE token's pattern matches at the introducer, and `line_comment`'s `//…` does NOT match at `#` — so the skipper swallows `#  (  ) ;timeunit 09 ns//>Mg` to end-of-line as a "comment" (in SV `#` is a REAL TOKEN — delays, parameter lists), the regex then matches the SECOND comment `//QF.` at 41 (`line_comment` "advanced from 11 to 47"), `trivia@11→47` is MEMOIZED, and the real ANSI-header path's `#` string-terminal consults that memo and dies (`Terminal '#' failed at position 47 — found '/'`, 4 memo-poison reuse sites). The two-comment requirement is now explained: the second comment gives the post-theft cursor a matchable `//`, turning the bogus skip into a "successful" memoizable parse — with one comment the bogus attempt fails harmlessly. FIX DESIGN (engine, `ast_based_generator.rs`, parser-agnostic — needs its own slice with release ceremony): the comment arms must be suppressed **statically at EMIT time per introducer** — if ANY grammar terminal's literal/anchored-pattern can START with the arm's introducer (`#`, `//`, `/*`), that arm is wrong for that grammar and must not be emitted (SV defines `#` ⇒ no `#` arm — also retro-explains `-0067`: vhdl defines `/#/` ⇒ no `#` arm would have prevented VHDL-0002; SV defines `line_comment`/`block_comment` tokens ⇒ the `//`//`/*` arms likely go too, with explicit `trivia` owning comments — verify acceptance against the full SV corpus + suites). CONSUMER-VISIBLE: SV release bump + ledger row + contract/book lockstep (accept-set widening); all-parsers regen; cross-family + 16-seed sweeps. Priority: HIGHEST (fix-parser-bugs-ASAP).** THEN: the red-gate tickets (ebnf meta-grammar 6-gap catch-up leaf; rtl_frontend contract gate) + the rtl_frontend (71, + the stash-proven seed-42 spf ticket) / SV (645 + 68 no-path, A2.1) drives. |
| 1 | `GRAMMAR-WELLFORMED.H.12` (the SV `UNKNOWN`→0 drive) | `in-progress` (`H.12.1` ✅ `-0076` — SV `UNKNOWN 615→567`; `H.12.2` ✅ `-0077` — CLASSIFICATION of the 567; `H.12.3` ✅ `-0078` — the branch-`@sample` reach-honesty ENGINE FIX, **`UNKNOWN 567→123`** witness `726→1170`; remaining `123` = 20 `no_path` A1/A2 + 18 `OTHER` constraint/sequence `kw_*` + deeper residual → next slice; `H.12.4` ✅ `-0079` — cert `no_path`/`UNKNOWN` full-dump observability (`PGEN_CERT_COVERAGE_DUMP_ALL`, the 20-no_path + 123-UNKNOWN enumeration tool, OBSERVABILITY-ONLY), UNBLOCKS the `H.12.5` enumeration; `H.12.5` ✅ SPLIT 2026-06-15 — `H.12.5.1` `-0080` CLASSIFIED+ADJUDICATED the 123 [A1 11 proven alt-entry-only via the `library_text`-entry cert run, A2 5 candidate profile-orphan (interface-class, sv_2017 §8.26), 4 blessed; 103 bucketed B1 store-gated/B2 kw_*/B3 constructs/B4 white_space]; `H.12.5.2` ✅ `-0081` ADJUDICATED A2 NOT-a-defect — LRM-faithful (1800-2017 orphans `interface_class_declaration` in the item hierarchy, 1800-2023 wires it; PGEN mirrors both; the 5 A2 rules are witnessed under `--grammar-profile sv_2023`), so the 20 no_path are FULLY adjudicated as NON-defects (A1 11 entry-relative + A2 5 profile-relative + 4 blessed); `H.12.5.3` ✅ `-0082` B4 engine-shadowed-dead removal — **SV `UNKNOWN 123→121`** (`white_space` + `comment_only_source_region` removed at source, accept-identical, witness byte-identical 1170, closure contract v4→v5); `H.12.5.4` ✅ `-0083` CLASSIFIED the residual 121 into 3 generator mechanisms (M1 reach-shell-fallback 46 / M2 over-gen+store 40 / M3 no-plannable-probe 34); `H.12.5.5.1` ✅ `-0084` split M1→M1a/M1b; `H.12.5.5.2.1` ✅ `-0085` reach-path tool; `H.12.5.5.2.2` ✅ `-0087` the **M1a FIX (rule-level `@sample` stand-down) — SV `UNKNOWN 121→93`** (witness 1170→1198, spf=0, seeds 0/7/42); frontier → `H.12.5.5.3` M1b in-expression sibling-routing) | **The SystemVerilog per-grammar `UNKNOWN`→0 drive — SV is the ONLY non-fully-certified shipped grammar.** BASELINE (deterministic, this session, DEBUG `ast_pipeline`, count 40 seed 0 `--grammar-profile sv_2017 --entry-rule systemverilog_file`): `total=1342 proof=1 witness=726 UNKNOWN=615 sample_parse_failures=0` (matches the layer-A pointer). **`H.12.1` = remove the 48 blessed LRM-decomposed number-infrastructure orphans** (the regex `H.10.1` / svpp `trivia` / vhdl `white_space` literal-0 precedent). ROOT CAUSE (tools-first, WHY+WHERE): `PGEN-SV-EXH-PROOF-0021` (.3.2 Strategy 1) consolidated `integral_number`/`real_number`/`unsigned_number` into clean single regexes, SEVERING the decomposed-number reference chain and orphaning the entire subgraph — the grammar's own header note (lines 24-33 "Numeric chain … part of the reach-set via the chain" is now STALE) + the inline note (lines 416-423 "orphaned-but-harmless") + the `systemverilog_syntax_closure_contract.json` blessed lists (`lrm_decomposed_number_orphans` 27 + `lrm_decomposed_number_kw_helpers` 21 = **48**) all document it. An INDEPENDENT reference-graph dead-closure from the 3 real entries (`systemverilog_file`/`library_text`/`systemverilog_parseable_file`, following lookahead edges too for removal safety) reproduces the SAME 48 exactly. The cert pass reports them as no-path UNKNOWN; the linter tolerates them as benign orphan-roots (`unreachable_rules=0`, RGX-0081 precedent) — but the cert-coverage `UNKNOWN`→0 doctrine + the Hopcroft-Ullman REDUCED-grammar requirement (no useless symbols) demand removal, and the contract's OWN drift policy sanctions exactly this (option **b**: "DELETING the orphan"). SAFETY: no live rule body, no Rust source/registry/hook, and no shape-sample references them (only append-only `systemverilog_v1.json` calibration_history prose + the closure contract's blessed lists). ACCEPTANCE: 48 removed at source; cert UNKNOWN drops ~48 (615→~567) with witness/proof/spf otherwise byte-identical; `sv_syntax_closure_gate` green with re-baselined `constraints` (max_unreachable_rules 49→1, both blessed number lists emptied, min_total_rules + max_unreachable_branches lowered to MEASURED values) — only `module_path_conditional_expression` remains the 1 LRM mutual-recursion unreachable; SV external corpus 14/14; shape-contract green; default+dual-feature lib green; cross-family + pcre2 oracle green; parse-neutral ⇒ NO release/schema bump (orphans never parsed ⇒ wire-shape identical). **VERIFICATION (all green, tools-first):** `--lint-grammar` clean (`unreachable_rules=0`, `non_terminating=0`, `unbound_fact_kinds=0`, `profile_orphans=0`, `unresolved_rule_reference_count=0`); cert-coverage `total=1342→1294 proof=1 witness=726 (BYTE-IDENTICAL) UNKNOWN=615→567 spf=0` deterministic at seeds 0/7/42; no-path 68→20; `sv_syntax_closure_gate` PASS (re-baselined contract v3→v4: `min_total_rules` 1453→1405, `max_unreachable_rules` 49→1, `max_unreachable_branches` 109→11, both number blessed lists `[]`; measured `defined_rule_count=1408 reachable_rules=1409 unreachable_rules=1 unreachable_branches=11`); `ast_shape_contract_gate` 16/16; SV external corpus **14/14** (`parse_fail_total=0`); dual-feature lib **752/0** (21 ignored); `stimuli_cross_family_platform_gate` PASS. NO release/schema/inventory bump, no ledger event. **Commit: `PGEN-GRAMMAR-WELLFORMED-0076`.** |
| 1 | `GRAMMAR-WELLFORMED.H.12.2` (classify the SV `UNKNOWN=567`) | `done` (`PGEN-GRAMMAR-WELLFORMED-0077`, pure-docs INVESTIGATION) | **The SV `UNKNOWN`→0 drive's classification slice — partition the 567 into actionable classes + pick the next fix target (the prerequisite to any H.12.3 generator/grammar work, per [[feedback_why_and_where_before_solution]] + [[feedback_tools_first_no_guessing]]).** METHOD (tools-first, DEBUG `ast_pipeline`, `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, `--report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0`; log `/tmp/h122_sv_probes_seed0.log`). BASELINE RECONFIRMED (matches `H.12.1`): `total=1294 proof=1 witness=726 UNKNOWN=567 spf=0`; plannable reach pass `1076 UNKNOWN targeted; 442 witnessed, 540 parsed-but-routed-elsewhere, 456 probe non-reparses (per-attempt), 29 generation failures`; `no_path=20`. **PARTITION OF THE 567:** **(A) `no_path` 20 — NOT dead** (the `H.10.1`/`H.12.1` literal-0 REMOVAL does NOT apply; this is the attribution rule's *entry/profile-relative* case, distinct from a dead orphan). TWO statically-confirmed mechanisms: **(A1) alternate-entry-only roots** — `sv_multi_entry_root`, `systemverilog_parseable_file`, `parseable_source_item`, and the `library_text` subtree (`library_text`/`library_description`/`library_declaration`/`library_identifier`/`kw_library` + `include_statement`/`kw_include`): reachable ONLY from the `library_text` / `systemverilog_parseable_file` / `sv_multi_entry_root` entries, NEVER from `systemverilog_file` by design (LRM separates library-map files + the parseable-fragment entry); **(A2) sv_2023-profile-relative** — `interface_class_declaration`/`interface_class_item`/`interface_class_method`/`declared_interface_class_identifier` (refs ONLY at `systemverilog.ebnf:490` `anonymous_program_item_sv_2023`, `:971` `class_item_sv_2023`, `:3577` `package_or_generate_item_declaration_sv_2023`) + `class_constructor_super_args` (ref ONLY at `:915` `class_constructor_declaration_sv_2023`): their only positive references are from `*_sv_2023` rules, FILTERED OUT under `--grammar-profile sv_2017` ⇒ no reach path under sv_2017 (CORRECTION of the loose family-grep: `config_declaration`/`config_*` are NOT no_path — `description:1801` references `config_declaration` directly and `description` IS reachable from `systemverilog_file`, so they are pass-1/2-witnessed). ADJUDICATION: A1/A2 are witnessed by per-entry cert runs (`--entry-rule library_text` / `systemverilog_parseable_file`), the multi-entry root (`--entry-rule sv_multi_entry_root`), or the `sv_2023` profile run — NOT by removal. ⚠️ FLAG (A2): interface classes are SV-2012 (valid in 2017) — the sv_2017 reach gap may be a PROFILE-orphan defect (missing sv_2017 reference) rather than a genuine sv_2023-only construct; needs a profile-reachability audit. HONEST LIMIT: the report caps the `no_path` print at 10 (and the `UNKNOWN` list at 25), so the EXACT full-20 enumeration + per-rule A1-vs-A2 adjudication needs the multi-entry/profile cert tooling, per-entry runs, OR a one-line observability uncap → ticketed `H.12.4` (cert no_path/UNKNOWN full-dump observability). **(B) reachable-but-unwitnessed ≈ 547 — the BULK** (final `UNKNOWN` minus the 20). VERDICT (of the 599 probed-but-unwitnessed has-path rules; ~52 leave final-UNKNOWN via incidental cross-probe witnessing): **DOMINATED by `parsed-but-routed-elsewhere` (540)** — the reach plan generates a sample that PARSES but routes through OTHER rules, not the target (canonical evidence: `inside_expression` probe → `"module m; endmodule"`, the reach plan falling back to a trivial source-text item routing elsewhere); + 59 never-reparsed + ~15 generation-failure-only. This is the GENERATOR-REACH-HONESTY class — the SV analogue of the `RTL-FE-CLOSURE.5.x` reach-path fixes (lookahead-poisoning `.5.4`, self-recursive-branch preference `.5.3`/`.5.6`, two-tier depth `.5.5`, mandatory-sibling depth) on SV's far larger/deeper grammar. FAMILY breakdown of the 599: **`kw_*` 130 (largest single coherent class)**, `*_identifier` 49, `*_expr(ession)` 37, `*_sv_2017` 39, `*_statement` 24, `*_declaration` 20, `*_item` 20, `*_type` 13, `*_lvalue` 5, `*_call` 5, `other` 256. **NEXT FIX TARGET (`H.12.3`):** the `parsed-but-routed-elsewhere` bulk, STARTING with the `kw_*` keyword-token cluster (130 — largest coherent + the proven-tractable shape: the rtl_frontend reach fixes were keyword-driven). DETERMINISM: the `no_path`/structural facts are seed-independent by construction; the verdict COUNTS are a seed-0 snapshot (the FAMILIES are the durable finding). **PURE-DOCS ⇒ NO code/grammar/generated/release/schema/inventory/ledger change; clippy not invoked. Status: parser-family rows UNCHANGED — SV stays the only non-fully-certified shipped grammar (closure-debt analysis toward its eventual `Done`).** **Commit: `PGEN-GRAMMAR-WELLFORMED-0077`.** |
| 1 | `GRAMMAR-WELLFORMED.H.12.3` (SV reach-honesty: the branch-`@sample` short-circuit) | `done` (`PGEN-GRAMMAR-WELLFORMED-0078`, ENGINE FIX — generator witness/reach-pass only; NO release/schema/wire change) | **SV cert `UNKNOWN 567 → 123` (witness `726 → 1170`, +444), `spf=0`, deterministic seeds 0/7/42 (`123/122/123`; the ±1 is the reach pass's wall-clock per-attempt timeout, not a structural wobble).** ROOT CAUSE (tools-first WHY+WHERE, env-gated `PGEN_CERT_COVERAGE_DEBUG_PROBES`/`…DEBUG_REACH` reach-path dump, since removed): the BRANCH-level `@sample`/`@probe_sample` literal override in `generate_or` (`stimuli_generator.rs:6822`) fired UNCONDITIONALLY — it never consulted the active reach plan. So when the plannable-rule reach pass steered THROUGH `module_declaration`'s branch-0 `@sample:"module m; endmodule"` (or `program_declaration`'s `@sample:"program p; endprogram"`) to witness a deep `kw_*`, the override emitted the minimal shell and never descended into the forced body where the construct + keyword live. EVIDENCE (count 40 seed 0): 130 `kw_*` unwitnessed, **112/130 fell back to a shell** (`module m; endmodule` ×57, `program p; endprogram` ×55); the reach-path dump proved **57 failing paths route through `module_declaration@`, 55 through `program_declaration`** — exactly the 57+55 buckets (canonical: `kw_cmos_aabe63a7` chain `…→description→module_declaration→module_declaration_sv_2017→non_port_module_item→…→cmos_switchtype→kw_cmos`, directives force `module_declaration@root->0` + `module_declaration_sv_2017@root->0`). FIX (parser-agnostic, additive, `stimuli_generator.rs`): in `generate_or`, suppress the branch literal override when the active reach plan FORCES this exact branch (`forced_branch_for(rule,node_path)==Some(selected_global)`) so the plan descends into the body. Off-reach (no plan / a non-forced branch) is BYTE-IDENTICAL (the gate computes `reach_forces_this_branch` only inside a reach plan). The single fix CASCADED far beyond the 130 `kw_*` (descending into module/program bodies witnessed every deep construct + all rules along those paths): plannable `witnessed 442 → 896`, `parsed-but-routed-elsewhere 540 → 83`. INERT for the 6 fully-certified grammars: rtl_const_expr/rtl_frontend/vhdl/json carry **0** `@sample`; regex(1)/svpp(6) are `fully_certified` ⇒ the plannable reach pass (PASS 3) never runs in cert. The rule-level gate + `on_path_rules` infra explored first were REVERTED once the dump proved the short-circuit is branch-level (targeted-fix discipline — no code on a hypothesis). VERIFIED (all green): SV cert seeds 0/7/42 `UNKNOWN 123/122/123 spf=0`; json `9/9`, regex `198/198`, svpp `74/74` stay `fully_certified` (rtl_const_expr/rtl_frontend/vhdl provably inert via 0 `@sample`); default lib **653/0** (+1 lock `reach_plan_forced_branch_suppresses_branch_sample_override` — control: off-reach `@sample` honored; fix: forced branch descends); clippy strict-source clean (generated stage = pre-existing 191-site tolerated debt); `regex_pcre2_compile_oracle_gate` PASS; `stimuli_cross_family_platform_gate` PASS (regex/vhdl/SV bounded replay). NO release/schema/inventory/ledger change (parser, AST shape, and the diverse certification pass are all untouched). Remaining SV `UNKNOWN 123` (the 20 `no_path` A1/A2 + the 18 `OTHER` constraint/sequence `kw_*` + deeper residual) → next H.12 slice. **Commit: `PGEN-GRAMMAR-WELLFORMED-0078`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5` (SV `UNKNOWN 123` residual drive — CONTAINER, split 2026-06-15) | `active` (`.1` ✅ classify `-0080`; `.2` ✅ A2 adjudicated NOT-a-defect `-0081`; `.3`+ per-mechanism fix children pending) | Classify + drive the post-`H.12.3` SV residual `123`: (a) the **20 `no_path`** (A1 alternate-entry-only roots / A2 sv_2023-profile-relative — needs `H.12.4` observability to enumerate + adjudicate; A1 are witnessed by per-entry cert runs not removal, A2 may be a profile-orphan defect); (b) the **18 `OTHER` `kw_*`** (constraint/sequence/class keywords — `and`/`or`/`before`/`constraint`/`foreach`/`implements`/`intersect`/`packed`/`rand`/`randc`/`soft`/`solve`/`tagged`/`token`/`typedef`/`union`/`void`/`within` — these produced non-shell `parsed-but-routed-elsewhere` samples, a DIFFERENT mechanism than the `H.12.3` `@sample` short-circuit); (c) the deeper `parsed-but-routed-elsewhere`/`generation-failure` residual. Tools-first WHY+WHERE per cluster; additive/general reach-honesty or grammar fixes; measure SV `UNKNOWN`+spf seeds 0/7/42 + cross-grammar byte-identity. |
| — | `GRAMMAR-WELLFORMED.H.12.5.1` (classify + adjudicate the enumerated 123) | `done` (`PGEN-GRAMMAR-WELLFORMED-0080`, pure-docs INVESTIGATION) | **Full tool-backed classification of the post-`H.12.3` SV `UNKNOWN=123` — the prerequisite WHY+WHERE map before any fix (per [[feedback_why_and_where_before_solution]] + [[feedback_tools_first_no_guessing]]; the `H.12.2` precedent applied to the post-`H.12.3` residual, now FULLY enumerable via the `H.12.4` tool).** METHOD (tools-first, DEBUG `ast_pipeline`, `PGEN_CERT_COVERAGE_DUMP_ALL=1`, `--report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0`; logs `/tmp/h125_sv_cert_seed0.log` + the A1 adjudication run `--entry-rule library_text` `/tmp/h125_sv_cert_library_entry.log`). BASELINE RECONFIRMED: `total=1294 proof=1 witness=1170 UNKNOWN=123 spf=0` (matches the layer-A pointer; deterministic). **THE 20 `no_path` — FULLY ADJUDICATED:** **(A1) 11 alternate-entry-only roots — PROVEN NOT DEAD** (`sv_multi_entry_root`, `systemverilog_parseable_file`, `parseable_source_item`, `library_text`, `library_description`, `library_declaration`, `include_statement`, `kw_library`, `kw_include`, `kw_incdir`, `kw_file_path_spec`): a per-entry cert run with `--entry-rule library_text` WITNESSES all 11 (none appears in that run's UNKNOWN/no_path lists) ⇒ they are `no_path` from `systemverilog_file` purely because the LRM separates library-map files + the parseable-fragment entry from the design-source entry. NOT a defect; correctly entry-relative; certifiable via per-entry/multi-entry cert runs, NEVER removal. **(A2) 5 sv_2023-only interface-class — CANDIDATE PROFILE-ORPHAN DEFECT** (`interface_class_declaration`, `interface_class_item`, `interface_class_method`, `declared_interface_class_identifier`, `class_constructor_super_args`): every positive reference to `interface_class_declaration` is inside a `*_sv_2023` rule (`systemverilog.ebnf:490` anonymous_program_item_sv_2023, `:971` class_item_sv_2023, `:3577` package_or_generate_item_declaration_sv_2023; `class_constructor_super_args` only at `:915` class_constructor_declaration_sv_2023), and the dispatchers (`class_item`/`package_or_generate_item_declaration`/`anonymous_program_item`) select the sv_2017 variant under `--grammar-profile sv_2017` ⇒ unreachable. BUT interface classes are valid SystemVerilog-2012/2017 (IEEE 1800-2017 §8.26) ⇒ likely a PROFILE-ORPHAN grammar defect (the well-formedness "no profile orphans" axis). ⚠️ HONEST LIMIT before a fix: `grammars/systemverilog_2017_lrm_extracted.ebnf` does NOT carry `interface_class_declaration` as a production (only `interface_class_type`) — so the exact sv_2017 BNF placement must be LRM-grounded before wiring ⇒ owned by `H.12.5.2` (consumer-visible accept-set widening; release ceremony). **(blessed/other) 4:** `module_path_conditional_expression` (LRM mutual-recursion, blessed in the closure contract), `union_modifier`, `kw_n_29`, `kw_n_48`. **THE 103 reachable-but-unwitnessed — BUCKETED:** **(B1) store-gated identifier-categorization cluster (~26)** — `known_unscoped_*`/`checked_*`/`provisional_*`/`declared_class_alias_identifier`/`instance_or_class_scope`/`(ps_or_)hierarchical_array_identifier`/`class_scoped_tf_call`/`context_member_method_call`/`direct_index_method_call` consult the semantic store ⇒ witnessing needs a sample where the identifier IS categorized (declared type/class/covergroup/let/param) — **STORE-AWARE-GEN territory** ([[feedback_grammar_rules_must_consult_store]]; STORE-AWARE-GEN.4a/.4b is the measured SV effort). **(B2) kw_* temporal/sequence/constraint cluster (24)** — `kw_accept_on/before/eventually/intersect/nexttime/or/reject_on/s_always/s_eventually/s_nexttime/s_until/s_until_with/soft/solve/sync_accept_on/sync_reject_on/until/until_with/within/foreach/constant/class_qualifier/sv_dollar/n_43`: property/sequence temporal operators + constraint keywords inside CLASS/CONSTRAINT/SEQUENCE/PROPERTY bodies the diverse pass rarely descends into + the reach pass routes-elsewhere — the generator-reach-honesty class (the `H.12.5` plan's group b; a DIFFERENT mechanism than the `H.12.3` `@sample` short-circuit). **(B3) constraint/sequence/property/expression construct rules (~40)** — `constraint_*`/`uniqueness_constraint*`/`extern_constraint_declaration*`/`solve_before_list`/`cond_pattern`; `property_*`/`sequence_method_call`/`cycle_delay*`/`goto_repetition`/`repeat_range`/`with_covergroup_expression`; `conditional_expression`/`inside_expression*`/`open_range_list*`/`open_value_range*`/`range_expression`/`array_range_expression`/`concatenation`/`multiple_concatenation`/`empty_unpacked_array_concatenation`/`tagged_union_expression*`/`cast`/`constant_cast`/`casting_type`/`scalar_constant`/`question`/`module_path_expression_lr_suffix`/`assignment_pattern_entry`; param/port `parameter_port_declaration*`/`mixed_*parameter_port_list`/`named_checker_port_connection*`/`loop_variables`/`associative_dimension`/`queue_dimension`/`index_variable_identifier` — B2's reach-into-deep-bodies mechanism on construct rules. **(B4) misc (2)** — `white_space` + `comment_only_source_region`: `white_space := /[ \t\r\n]+/` is the ENGINE-SHADOWED-DEAD case (the layout skipper consumes whitespace before any non-empty regex executes — the vhdl `H.11.4`/`H.11.4-UNPARK` literal-0 precedent; removal candidate, but here `white_space` is woven into BOTH `trivia` and `comment_only_source_region` so the edit is larger than vhdl's). **NEXT FIX TARGETS (each its own split child, tractability order):** `H.12.5.2` A2 interface-class profile-orphan (LRM-ground → wire into sv_2017; −5, a real defect); then `white_space`/`comment_only_source_region` literal-0 (B4); then B2/B3 reach-honesty; then B1 STORE-AWARE-GEN. **PURE-DOCS ⇒ NO code/grammar/generated/release/schema/inventory/ledger change; clippy not invoked. Parser-family rows UNCHANGED — SV stays the only non-fully-certified shipped grammar.** **Commit: `PGEN-GRAMMAR-WELLFORMED-0080`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.2` (A2 interface-class — ADJUDICATION) | `done` (`PGEN-GRAMMAR-WELLFORMED-0081`, pure-docs — the candidate defect is REFUTED) | **A2 is NOT a PGEN defect — the `H.12.5.1` "candidate profile-orphan" flag is RESOLVED as LRM-faithful.** WHY (tools-first, LRM-grounded): the IEEE **1800-2017** LRM DEFINES `interface_class_declaration` (§8.26.1 / A.1.2) but does NOT reference it as an alternative in `class_item` (`docs/systemverilog/2017/md/section-8-classes.md:95` carries `class_declaration` only), `package_or_generate_item_declaration` (`section-26-packages.md:60`), or `anonymous_program_item` — i.e. the 2017 LRM ORPHANS interface_class_declaration in the source-text item hierarchy. The **1800-2023** LRM FIXED this: `class_item` (`2023/md/section-8-classes.md:102`), `package_or_generate_item_declaration` (`section-26-packages.md:71`), and program items (`section-24-programs.md:133`) each add `\| interface_class_declaration` next to `class_declaration` (2023 Annex A confirms `:519`/`:681`/`:696`). PGEN's grammar mirrors BOTH LRMs — interface_class_declaration is referenced ONLY from the `*_sv_2023` variants (`systemverilog.ebnf:490`/`:971`/`:3577`) — so under `--grammar-profile sv_2017` it is CORRECTLY unreachable (faithful to the 2017 LRM's own omission), and WIRING it into sv_2017 would be a DEFECT (LRM-divergence; the candidate fix is REFUTED — [[project_ebnf_is_single_source_of_truth]] + [[feedback_no_codebase_change_without_tool_backed_facts]]). WITNESSING EVIDENCE (the analogue of the A1 `library_text` proof): a cert run with `--grammar-profile sv_2023` WITNESSES all 5 A2 rules (`interface_class_declaration`/`_item`/`_method`, `declared_interface_class_identifier`, `class_constructor_super_args` — none in that run's UNKNOWN/no_path lists; sv_2023 cert `total=1314 witness=1202 UNKNOWN=111 spf=0`). VERDICT: the 5 A2 rules are CORRECT profile-relative no_path under sv_2017, certifiable under the sv_2023 profile — NOT removal, NOT sv_2017-wiring. IMPLICATION for the SV `fully_certified` endgame: the 20 no_path are now FULLY adjudicated as NON-defects (A1 11 entry-relative + A2 5 profile-relative + 4 blessed) ⇒ SV "fully_certified under sv_2017" needs a multi-profile/multi-entry accounting (witness A1 via per-entry runs, A2 via the sv_2023 profile, bless the 4), recorded as an open endgame decision. **PURE-DOCS ⇒ NO code/grammar/generated/release/schema/inventory/ledger change; clippy not invoked. Parser-family rows UNCHANGED.** **Commit: `PGEN-GRAMMAR-WELLFORMED-0081`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.3` (B4 `white_space`/`comment_only_source_region` literal-0) | `done` (`PGEN-GRAMMAR-WELLFORMED-0082`, GRAMMAR — accept-identical, NO release/schema bump) | **The B4 engine-shadowed-dead removal LANDED — SV cert `UNKNOWN 123→121` (witness BYTE-IDENTICAL 1170, no collateral), the vhdl `H.11.4-UNPARK` literal-0 precedent applied one level up.** PROOF (tools-first WHY+WHERE, DEBUG `ast_pipeline --features generated_parsers,ebnf_dual_run`, count 40 seed 0): the cert reach probe (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`) shows BOTH `white_space` (`parsed=true witnessed_target=false` on whitespace samples `"     "`) AND `comment_only_source_region` (`parsed=true witnessed_target=false` on `"//x\n"`) are ENGINE-SHADOWED-DEAD — the generated layout skipper consumes whitespace AND comments as leading trivia before any non-empty-matchable regex / `source_text_item` branch executes; six comment-only/comment-containing inputs (`//x\n`, `/* */`, lead/trail comments, no-`\n` EOF, ws+comments) parse with **ZERO** `comment_only_source_region` nodes in the AST (a comment-only file → `source_text: []`). So by the attribution rule both are genuinely dead ⇒ the literal-0 **delete the orphan** path (NOT `@sample`-coverable — the probe already FALSIFIED that for `comment_only_source_region`). EDIT (`grammars/systemverilog.ebnf`): (1) `trivia := (white_space \| line_comment \| block_comment)*` → `(line_comment \| block_comment)*` (priority comment rewritten for the 2 remaining mutually-exclusive arms); (2) `white_space := /[ \t\r\n]+/` removed at source; (3) the `comment_only_source_region` rule removed; (4) its `source_text_item` branch removed (`@priority` `[24,16,16,12,10,8,6,4]` → `[24,16,16,12,10,8,4]`); in-grammar adjudication comments record the engine-shadowed-dead removal. CONTRACT re-baseline (`systemverilog_syntax_closure_contract.json` v4→v5, leaf-owned per the drift policy): `min_total_rules` 1405→1403 (`defined_rule_count` 1408→1406), `min_reachable_rules` 1406→1404 (`reachable_rules` 1409→1407), `max_unreachable_branches` 11→2 (removing `comment_only_source_region`'s quantifier/alternation sub-branch accounting); `max_unreachable_rules` stays 1 (`module_path_conditional_expression`). **VERIFIED (all green, tools-first):** cert-coverage `total=1294→1292 proof=1 witness=1170 (BYTE-IDENTICAL) UNKNOWN=123→121 spf=0` deterministic seeds 0/7/42; the UNKNOWN delta is EXACTLY `{white_space, comment_only_source_region}` with NO new collateral UNKNOWN (set-diff verified); `--lint-grammar` clean (`unreachable_rules=0`, `non_terminating=0`, `ordered_choice_shadowing=0`, `unbound_fact_kinds=0`, `profile_orphans=0`; 1292 rules); `sv_syntax_closure_gate` PASS (re-baselined; measured `defined=1406 reachable=1407 unreachable_rules=1 unreachable_branches=2`); `ast_shape_contract_gate` **16/16**; SV external corpus **14/14** (`parse_fail_total=0`, uvm/uvm_compat both `sv_2017`/`sv_2023`); `stimuli_cross_family_platform_gate` PASS (regex/vhdl/SV); dual-feature lib PASS; `clippy_on_rust_change` strict-source clean (generated stage = pre-existing tolerated debt); `mdbook_docs_gate` + `systemverilog_parser_book_gate` PASS (HTML regenerated). ACCEPT-IDENTICAL ⇒ NO release/schema/inventory/ledger bump — neither rule ever appeared in any AST (wire-shape identical); the declared `source_text_item` union narrows 8→7 kinds, a never-emitted-variant correction. LOCKSTEP: SV book `json-carrier.md` (`source_text_item` 8→7 branches + the removal note), top-level book `grammar-wellformedness.md` (the SV arc `123→121` step), SV integration contract Current-Trust-Statement accept-set note. SV stays the only non-fully-certified shipped grammar (`UNKNOWN=121`). **Commit: `PGEN-GRAMMAR-WELLFORMED-0082`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.4` (CLASSIFY the post-`H.12.5.3` residual into root-cause mechanisms) | `done` (`PGEN-GRAMMAR-WELLFORMED-0083`, PURE-DOCS INVESTIGATION) | **Tool-backed root-cause partition of the SV `UNKNOWN=121` (the prerequisite WHY+WHERE before any reach/gen fix, per [[feedback_why_and_where_before_solution]]). The `H.12.5.1` surface buckets (B1/B2/B3) resolve into THREE generator MECHANISMS.** METHOD (tools-first, DEBUG `ast_pipeline --features generated_parsers,ebnf_dual_run`, `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` + `PGEN_CERT_COVERAGE_DUMP_ALL=1`, count 40 seed 0): cross-referenced each of the 120 enumerable UNKNOWN rules against its plannable-reach probe outcome (`parsed`/`witnessed_target`). PARTITION: **(M1) reach-shell-fallback — 46 rules** (`parsed=true witnessed_target=false`): expression/select/postfix/`cast`/`concatenation`/`conditional_expression`/`inside_expression*`/`assignment_pattern_entry`/`array_range_expression`/… — the reach plan forces a path toward the deep expression target but FALLS BACK to a canonical `module m(input logic a);endmodule` shell (the probe sample is literally that shell), never descending into an expression position. The **same class as `H.12.3`'s shell short-circuit but for targets DEEPER than the forced branch** — `H.12.3`'s branch-`@sample` stand-down only covers the EXACTLY-forced branch, not an expression target many levels below `module_declaration`. **(M2) over-generation / store-unfaithful — 40 rules** (only `parsed=false`): the generator emits a structurally-INVALID body the parser REJECTS → never witnesses. Two sub-shapes: **M2a** constraint/sequence/property bodies (`constraint_expression`/`constraint_primary`/`constraint_set`/`boolean_abbrev`/`consecutive_repetition`/`cycle_delay*`/… — e.g. `constraint\foo ::\foo {4080.1;}` and `sequence\foo ;8_233.29_intersect 9.00261E0endsequence` are rejected: a bare number is not a valid `constraint_expression`/`sequence_expr`); **M2b** B1 store-gated identifier rules (`checked_*`/`known_unscoped_*`/`declared_class_alias_identifier`/… — the generated sample never establishes the required store fact ⇒ the `phase: post` predicate fails ⇒ reject — STORE-AWARE-GEN territory, [[feedback_grammar_rules_must_consult_store]]). **M2 needs a per-rule over-gen-vs-parser-bug adjudication** (a `parsed=false` witness sample could be a generator over-generation OR a parser wrongly rejecting valid SV — the director's bug-finding-oracle principle). **(M3) no-plannable-probe — 34 rules**: the plannable-witness pass never synthesizes a candidate at all = the 20 already-adjudicated `no_path`/blessed NON-defects (`H.12.5.1`/`.2`: A1 alternate-entry + A2 sv_2023 interface-class + 4 blessed) PLUS ~14 property/sequence temporal operators (`kw_accept_on`/`eventually`/`nexttime`/`reject_on`/`s_*`/`sync_*`/`until*`, `property_case_item`) the plannable pass cannot reach into the property/sequence-expression grammar to construct. NEXT (each its own split child, tractability order): **`H.12.5.5`** M1 expression-reach (largest, reuses `H.12.3` machinery); **`H.12.5.6`** M2 over-gen/store (per-rule over-gen-vs-parser-bug adjudication first); **`H.12.5.7`** M3 plannable-probe synthesis for property/sequence temporal operators. **PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked. Parser-family rows UNCHANGED — SV stays the only non-fully-certified shipped grammar (`UNKNOWN=121`).** **Commit: `PGEN-GRAMMAR-WELLFORMED-0083`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5` (M1 expression-context reach) | `active` (SPLIT 2026-06-16 by `H.12.5.5.1` — the tools-first investigation refined the `-0083` M1 bucket into TWO distinct reach mechanisms) | **The 46 M1 (`parsed=true witnessed_target=false`) rules — the reach probe parses but never positively enters the target rule.** Children: `H.12.5.5.1` (WHY+WHERE investigation, done `-0084`), `H.12.5.5.2` (M1a header-reach-honesty fix, frontier), `H.12.5.5.3` (M1b in-expression sibling-routing fix). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.1` (M1 reach WHY+WHERE investigation) | `done` (`PGEN-GRAMMAR-WELLFORMED-0084`, PURE-DOCS INVESTIGATION) | **Tools-first WHY+WHERE (DEBUG `ast_pipeline`, `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` + `PGEN_CERT_COVERAGE_DUMP_ALL=1`, count 40 seed 0): reproduced the deterministic baseline (`total=1292 proof=1 witness=1170 UNKNOWN=121 spf=0` — BYTE-matches `H.12.5.3`, so the metric is SIGNAL) and split the `-0083` M1 bucket into TWO mechanisms.** **M1a (reach DEAD-ENDS at the module/program header):** `cast`/`concatenation`/`conditional_expression`/`cond_pattern`/`cond_predicate`/`assignment_pattern_entry`/`inside_expression*`/`associative_dimension`/`empty_unpacked_array_concatenation`/`expression_or_cond_pattern`/`instance_or_class_scope` — the probe sample is a minimal `module m(input logic a);endmodule` (the body `module_item*` expands to ZERO; the expression target's OR-node is never entered). **M1b (reach DOES descend into an expression/sequence/attr/decl context but routes through a SIBLING rule):** `array_range_expression`/`bit_select_expression`/`direct_index_method_call` → `program p(...);assign \foo.\foo[\foo].\foo=<num>;endprogram`; `goto_repetition`/`class_scoped_tf_call` → `sequence\foo ;…endsequence`; `context_member_method_call`/`constant_let_expression`/`class_scoped_call_prefix` → `(*\foo =+…*)`; `inout_declaration`/`input_declaration`/`callable_identifier`/`checker_instantiation` reach a declaration but route through a sibling. **WHERE (pinned):** `reach_hops` (`stimuli_generator.rs:5186`) is a BFS **shortest-HOP** path over the rule-reference graph; `set_reach_plan_for_rule` (`:2686`) forces ONLY the OR-branches (`directives_along_path`) + quantifiers (`quantifier_sites_along_path`) that lie ON that hop path → for M1a the shortest path lands at a header position that completes minimally without entering the target. SAME reach-path-SELECTION-honesty family as `RTL-FE-CLOSURE.5.3`/`.5.4` (both refined *which* path `reach_hops` picks). NOTE: `casting_type`/`constant_cast` are `parsed=false` ⇒ M2 (`H.12.5.6`), NOT M1. PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; rows UNCHANGED. **Commit: `PGEN-GRAMMAR-WELLFORMED-0084`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.2` (M1a header-reach-honesty) | `done` (both children landed `-0085`/`-0087`) | The M1a rules (reach dead-ends at the module/program header). Children: `.2.1` (reach-path observability tool + port-reach pinpoint, done `-0085`), `.2.2` (the M1a fix — rule-level `@sample` stand-down on a reach-descended rule, done `-0087`, SV cert `UNKNOWN 121→93`). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.2.1` (reach-path observability tool + M1a port-reach pinpoint) | `done` (`PGEN-GRAMMAR-WELLFORMED-0085`, OBSERVABILITY) | **TOOL (`stimuli_generator.rs` `set_reach_plan_for_rule`, generator-only, env-gated, byte-identical when off): `PGEN_REACH_PATH_DUMP=1` prints the BFS hop chain `reach_hops` installs per plannable-witness target** (the sibling of `PGEN_CERT_COVERAGE_DEBUG_PROBES` / `PGEN_CERT_COVERAGE_DUMP_ALL`). With it, the M1a reach carrier is pinpointed: EVERY M1a expression target (`cast`/`conditional_expression`/`concatenation`/`cond_pattern`/`inside_expression`/`assignment_pattern_entry`/…) shares the BFS shortest-HOP prefix `systemverilog_file → source_text → source_text_item → description → module_declaration → module_declaration_sv_2017 → module_ansi_header (root/s6/q) → list_of_port_declarations → ansi_port_declaration (root/o2/s4/q) → expression → …` — the reach routes the deep expression target through the **ANSI PORT** (`ansi_port_declaration`'s unpacked-dimension / default), NOT the module body. The generated sample is a minimal `module m(input logic a);endmodule`: the on-path port-expression quantifier does NOT materialize, so the target is never positively entered (`parsed=true witnessed_target=false`). By contrast the M1b targets that DID witness reach via the module/program BODY (`program p(...);assign … = <expr>`). The `forced_quantifier_min` lookup key `(current_rule, node_path)` (`generate_quantified:7705`) MATCHES `quantifier_sites_along_path`'s key, so the forcing *should* fire — `.2.2` must determine why the forced port-path quantifier still renders empty (branch-not-taken vs forced-then-backtrack) BEFORE the surgical change. **OBSERVABILITY-ONLY ⇒ off-by-default byte-identical (cert `UNKNOWN=121 witness=1170` unchanged with the dump ON); generator-only ⇒ NO parser-regen/grammar/release/schema/ledger change; rows UNCHANGED.** **Commit: `PGEN-GRAMMAR-WELLFORMED-0085`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.2.2` (M1a fix) | `done` (`PGEN-GRAMMAR-WELLFORMED-0087`, GENERATOR-ONLY) | ✅ **M1a FIX LANDED (`-0087`): SV cert `UNKNOWN 121→93`** (witness `1170→1198`, `spf=0`, deterministic seeds 0/7/42). ROOT CAUSE (tools-first, transformed-tree model per `-0086`): the `--dump-gen-ast` transformed `ansi_port_declaration` is an `Or` with `expression` at `root/o2/s4` (named-port form); the `-0085` reach path routes every M1a expression target through `module_ansi_header`, whose RULE-LEVEL `@sample:"module m(input logic a);"` (confirmed in the dump's `semantic_annotations` bucket) fires at rule entry and short-circuits the body — the `-0085` witness `module m(input logic a);endmodule` is uniquely that literal + the minimal `module_declaration_sv_2017` body, and `-0086`'s `apd_keys=[]`/`o2/s4`-never-reached is the same fact (the body was never entered). WHERE: the rule-level override (`generate_rule:6115`) had NO reach-plan stand-down, unlike the `H.12.3` branch-level one (`generate_or:6768`). FIX (generator-only): `ActiveReachPlan::needs_rule_body_descent(rule)` gates the rule-level `@sample`/`@probe_sample` override to STAND DOWN when the plan forces any directive/quantifier keyed on the rule — the rule-level analogue of `H.12.3`; off-reach byte-identical, certified roster + `@sample`-free grammars inert; +1 test `reach_plan_through_rule_body_suppresses_rule_sample_override`. VERIFIED: cert seeds 0/7/42 (`DUMP_ALL` = every M1a target witnessed; residual 93 = 20 `no_path` non-defects + M1b + M2 + M3); unit tests 4/4; clippy source-strict clean; `stimuli_cross_family_platform_gate` PASS. GENERATOR-ONLY ⇒ NO parser-regen/grammar/release/schema/ledger change. **Audit trail of the prior discrimination:** **DISCRIMINATION (`-0086`, a throwaway `[q-force]` trace in `generate_quantified`, since reverted):** the forced port-path quantifier `(ansi_port_declaration, root/o2/s4)` is NEVER reached/applied during the M1a reach plans — `generate_quantified` is called for `ansi_port_declaration` only at branch-START sites (`root/o0/s0`/`root/o1/s0`/`root/o2/s0`) with `forced=None` and an EMPTY `apd_keys` (no forced `ansi_port_declaration` quantifier in the active plan when the port generates). So the expression-bearing branch `o2` falls back to a simpler port branch (`input logic a`) BEFORE its `s4` expression quantifier can fire — it is NEITHER a quantifier-forcing gap (old hypothesis a) NOR a forced-then-backtrack-at-the-quantifier (old hypothesis b). The transformed `grammar_tree` `ansi_port_declaration` (an `Or` with `expression` at `root/o2/s4`) differs heavily from the source (`( net_port_header \| interface_port_header )? port_identifier unpacked_dimension* ( assign constant_expression )?` — no top-level `Or`, no direct `expression` ref), so the model must be rebuilt on the TRANSFORMED tree. NEXT (tools-first): (1) dump the TRANSFORMED `ansi_port_declaration` (`--dump-gen-ast` or a targeted rule dump) to see branch `o2`'s real shape; (2) re-trace with the plan's `target_group_key` printed + whether `forced_branch_for(ansi_port_declaration, root)` returns `Some(2)` at generation, to learn whether branch `o2` is forced-then-fails vs the port is generated under an INLINED rule context (`current_rule ≠ ansi_port_declaration`, so the directive can't key); (3) THEN design the fix — likely reach-path SELECTION to prefer the module-BODY carrier the M1b targets witness through (`assign …=<expr>`) over the shorter-hop port carrier, the `RTL-FE-CLOSURE.5.3`/`.5.4` reach-path-honesty family. Change ONE thing; rebuild DEBUG `ast_pipeline` (generator-only, NO parser regen); measure GLOBAL cert + spf seeds 0/7/42 + cross-family (reach pass touches the closed-loop driver ⇒ gates RUN). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3` (M1b: in-expression sibling-routing) | `active` (SPLIT 2026-06-16 by `H.12.5.5.3.1` — tools-first WHY+WHERE pinned the M1b mechanism) | The M1b rules (`array_range_expression`/`bit_select_expression`/`direct_index_method_call`/`goto_repetition`/`context_member_method_call`/`constant_let_expression`/`class_scoped_tf_call`/`sequence_method_call`) whose reach descends into a construct context but routes through a SIBLING. Children: `.3.1` (WHY+WHERE investigation, done `-0088`), `.3.2` (the target-own-structure fix, done `-0090`, SV cert `93→90` — closed 2 carriers + 1 collateral), `.3.3` (M1b residual parent-commit WHY+WHERE, done `-0091` — re-parse-proved all 6 carriers ABSENT from their own AST ⇒ genuine sibling absorption; partitioned into 4 fix mechanisms C-i..C-iv, spawned fix children `.3.3.1`/`.3.3.2`/`.3.3.3` + reclassified `class_scoped_tf_call` to `H.12.5.6`). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.1` (M1b WHY+WHERE investigation) | `done` (`PGEN-GRAMMAR-WELLFORMED-0088`, PURE-DOCS INVESTIGATION) | **The M1b residual = the reach forces the path to the target but NOT the target's OWN distinguishing structure.** Tools-first (`PGEN_REACH_PATH_DUMP` + `PGEN_CERT_COVERAGE_DEBUG_PROBES`/`DUMP_ALL` + `--dump-gen-ast`; post-`-0087` baseline `witness=1198 UNKNOWN=93 spf=0` reproduced): the M1b carriers persist (the M1a fix `-0087` left them untouched — distinct mechanism), all `parsed=true witnessed_target=false` with REAL organic samples (no `@sample` shell ⇒ not M1a). MECHANISM: the reach forces the path to the target's PARENT + the parent branch referencing it (e.g. `call_primary`=`Or[10]`, `context_member_method_call` at `o0`, forced), but the TARGET rule generates MINIMALLY (root `Or`→`o0`, `?`/`*`→min) → a sibling-ambiguous form the PEG attributes to an earlier sibling. **B-i** degenerate first alternative (`array_range_expression` `o0=expression`; `callable_method_call_body` `o0=built_in_method_call`; `class_scoped_tf_call` `o0=class_scoped_tf_call_with_args`); **B-ii** distinguishing token behind a minimal optional (`context_member_method_call`'s `.method(args)`; `goto_repetition`). WHERE: `directives_along_path`(`stimuli_generator.rs:5511`)/`set_reach_plan_for_rule`(`:2686`) force decisions ALONG the path (entry → the target's reference site) but stop AT the target — the target's own root `Or`+quantifiers aren't in the plan. PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change. **Commit: `PGEN-GRAMMAR-WELLFORMED-0088`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.2` (M1b fix) | `done` (`PGEN-GRAMMAR-WELLFORMED-0090`, GENERATOR-ONLY) | **The M1b FIX — SV cert `UNKNOWN 93→90` (witness `1198→1201`, spf=0, deterministic seeds 0/7/42).** Implemented the designed mechanism: a new parser-agnostic **target-own-structure reach pass** that, for a still-`UNKNOWN` target rule `R`, forces `R`'s OWN root-`Or` branch (`o1..` first, then degenerate `o0`) + every `?`/`*` quantifier inside `R`'s body (≥1) ON TOP of the base reach plan to `R`'s reference site, so `R` renders a DISTINGUISHING form instead of the sibling-ambiguous minimal one. NEW: `target_own_reach_sites`/`collect_optional_quantifier_paths` (the structural walker, keyed purely on `(R, node_path)` ASTNode structure — NO grammar/rule name) + `generate_target_own_structure_witnesses` (`stimuli_generator.rs`); wired as cert-driver **PASS 3c** (`main.rs`) over ONLY the rules still UNKNOWN after passes 1+2+3. **Truly inert** for the fully-certified roster (empty residual ⇒ PASS 3c never generates a probe — `rtl_const_expr`/`json` confirmed byte-identical `UNKNOWN=0`, no pass line). Witnessed 2 named carriers (`bit_select_expression`, `constant_let_expression`) + 1 collateral; the other 6 carriers stay UNKNOWN (their distinguishing form is still sibling-absorbed at the PARENT even with `R`'s body forced ⇒ a parent-commit problem → `.3.3`). STRICTLY ADDITIVE (the `888→1717` guard): only ever unions witnesses ⇒ the 1198 baseline witnesses byte-safe. **VERIFIED (all green):** cert seeds 0/7/42 identical (`1201/90/spf0`); `stimuli_cross_family_platform_gate` PASS (regex+vhdl+SV closed-loop); `clippy_on_rust_change` strict-source clean; new focused unit test `target_own_reach_sites_finds_root_or_and_inner_optionals` PASS. GENERATOR-ONLY ⇒ NO grammar/EBNF/parser-regen/release/schema/inventory/ledger change. **Commit: `PGEN-GRAMMAR-WELLFORMED-0090`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3` (M1b residual: parent-commit WHY+WHERE) | `done` (`PGEN-GRAMMAR-WELLFORMED-0091`, PURE-DOCS INVESTIGATION) | **Tool-backed WHY+WHERE for the 6 M1b carriers `-0090` could not close — confirms parent-commit + partitions into 4 fix mechanisms.** DECISIVE DISCRIMINATOR (tools-first): for each carrier I re-parsed a representative generated witness with `parseability_probe --parse-dump-ast-pretty systemverilog <sample> --profile sv_2017`; **all 6 parse OK (full consume) but contain ZERO nodes of the carrier rule** ⇒ a sibling genuinely absorbs the bytes (NOT the `H.10.2.2` memo-hit coverage-record bug — there the rule WAS in the AST). The `-0088` parent-commit hypothesis is confirmed and refined. PARTITION: **C-i operator-shadow (1):** `goto_repetition` (`:= ( implies const_or_range_expression )` = bare `-> expr`; `->` is also the `operand_chain` `implies` infix operator — `017.22->4096` parses as an expression, the `( boolean_abbrev )?` host @4565 never entered; LRM has `[ ]` brackets the grammar lacks — LRM-ground first). **C-ii mandatory-inner-structure-not-forced (2):** `direct_index_method_call` (distinguishing tail `dot method_call_body` needs `(args)`; rendered bare `.\foo` → `!lparen`-guarded `select`/`bit_select` chain absorbs) + `context_member_method_call` (`( dot identifier constant_bit_select &dot )+` REQUIRES `[idx]`; rendered `\foo.\foo.\foo` has none → plain scoped name absorbs) — `-0090` forces root-`Or`+optionals but not MANDATORY sub-rule content. **C-iii store-gated (1):** `class_scoped_tf_call` (`class_scoped_call_prefix` @6202 STORE-GATED on known-class identifiers; `\foo` not a class → `package_scope tf_call` absorbs `\foo::\foo`) — RECLASSIFIED to `H.12.5.6` (M2/store). **C-iv reach-path-selection (2):** `sequence_method_call` (reach routed into a module named-port-connection host) + `array_range_expression` (`:= expression` alias; reach must force the parent `stream_expression … with (…)?` optional, not deep-force the alias body which over-constrains → `parsed=false`). Detail: [GRAMMAR-WELLFORMED-H12533-m1b-residual-parent-commit-whywhere.md](GRAMMAR-WELLFORMED-H12533-m1b-residual-parent-commit-whywhere.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked. Parser-family rows UNCHANGED — SV stays the only non-fully-certified shipped grammar (`UNKNOWN=90`). **Commit: `PGEN-GRAMMAR-WELLFORMED-0091`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.1` (C-ii: mandatory-inner-structure forcing) | `done` (DESIGN `PGEN-GRAMMAR-WELLFORMED-0092` + IMPL `PGEN-GRAMMAR-WELLFORMED-0093`, GENERATOR-ONLY) | **The mandatory-inner-structure forcing capability LANDED — SV cert `UNKNOWN 90→89` (witness `1201→1202`, spf=0, deterministic seeds 0/7/42).** New `mandatory_child_rules` walker (Sequence elements + min-1 `+`-group elements + `Atom::Node` groups + `rule_reference` tokens; skip min-0 quantifiers/un-forced `Or`/lookaheads/self) + a purely-additive child-forcing loop in `generate_target_own_structure_witnesses` (runs only when R-own forcing failed; per-child budget so a distinguishing child is reached even when an earlier child is also forceable; keyed `(child, node_path)` — the read-sides already fire on the rule being generated). **HONEST OUTCOME:** the mechanism is general + correct + strictly additive + inert for the fully-certified roster (rtl_const_expr/json `UNKNOWN=0`, no slowdown), but it closed **`sequence_method_call`** (a `-0091` C-iv carrier) — NOT the 2 hypothesized C-ii carriers. Re-parse proves `direct_index_method_call` (forced `method_call_body` call form → credited to `split_direct_callable_method`/`method_call` sibling in `bit_select_expression`) and `context_member_method_call` (forced `constant_bit_select [idx]` → still re-attributed at the `call_primary`/attribute parent) are **sibling-absorbed at the PARENT regardless of forced structure** ⇒ reclassified to `H.12.5.5.3.3.4` (parent-commit forcing / shadowing adjudication). VERIFIED: cert seeds 0/7/42 identical; `mandatory_child_rules` + `target_own_reach_sites` unit tests PASS; `stimuli_cross_family_platform_gate` PASS; clippy source-clean. GENERATOR-ONLY ⇒ no parser-regen/grammar/release/schema/ledger change. Design+outcome note: [GRAMMAR-WELLFORMED-H125331-cii-mandatory-inner-design.md](GRAMMAR-WELLFORMED-H125331-cii-mandatory-inner-design.md). **Commits: `PGEN-GRAMMAR-WELLFORMED-0092` (design), `PGEN-GRAMMAR-WELLFORMED-0093` (impl).** _Original scope text:_ extend the `-0090` target-own-structure pass to force the distinguishing content of a MANDATORY sub-rule, not just the root `Or` branch + min-0 optionals. Carriers: `direct_index_method_call` (force the `dot method_call_body` tail's `(args)` call alt) + `context_member_method_call` (force the `( dot identifier constant_bit_select &dot )+` group's `constant_bit_select` `[idx]`). **DESIGN (`-0092`, tool-backed):** the `--dump-gen-ast` transform stores a rule reference as a LEAF token `Atom(Token(["rule_reference", name]))` — NOT inlined — so both carriers' distinguishing structure lives in a SEPARATE rule (`method_call_body` Or @2793; `constant_bit_select := (lbrack constant_expression rbrack)*` @1262, a min-0 `*` rendering empty) the `-0090` walker never follows; both carriers' body root is a `Sequence` (`root_or=None`). The read-sides already force ANY rule keyed `(current_rule, node_path)` (that is how `set_reach_plan_for_rule` forces rules along the reach path) ⇒ NO read-side change. FIX = a bounded MANDATORY-reference walker (Sequence elements + min-1 `+`-group elements + `Atom::Node` groups + `rule_reference` tokens; skip min-0 quantifiers, un-forced `Or` branches, lookaheads) that, for each mandatory child `C` reachable in R's body, installs `target_own_reach_sites(C)` forcing keyed `(C, …)` on top of R's plan; parser-judged, residual-only PASS 3c, strictly additive. Design note: [GRAMMAR-WELLFORMED-H125331-cii-mandatory-inner-design.md](GRAMMAR-WELLFORMED-H125331-cii-mandatory-inner-design.md). Impl: change ONE thing; rebuild DEBUG `ast_pipeline` (generator-only); measure GLOBAL cert + spf seeds 0/7/42 + `stimuli_cross_family_platform_gate` + `rtl_const_expr`/`json` inertness. |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.2` (C-iv: reach-path selection) | `done` (`PGEN-GRAMMAR-WELLFORMED-0094`, PURE-DOCS INVESTIGATION — generator-neutral, RE-ADJUDICATED) | **Tools-first WHY+WHERE REFUTED the C-iv generator framing for `array_range_expression` — it is a C-i GRAMMAR delimiter-drop defect, NOT a generator reach gap; and the leaf's other carrier `sequence_method_call` was already closed by `-0093` (collateral) — so this C-iv leaf closes with NO generator change.** Live diagnosis on the canonical SV cert (seed 0, `total=1292 witness=1202 UNKNOWN=89 spf=0`) via DEBUG `ast_pipeline` `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` + `PGEN_REACH_PATH_DUMP=1`, then `parseability_probe --parse-dump-ast-pretty` / `--trace-rules`: the reach plan DOES force `stream_expression`'s `( kw_with ( array_range_expression )? )?` optionals and renders `<expr> with <arr>`, but on RE-PARSE the host `streaming_concatenation := lbrace stream_operator ( slice_size )? stream_concatenation rbrace`'s OFF-PATH `( slice_size )?` sibling greedily steals the stream_expression's leading `<expr>` (→ `slice_size`), so `with` lands in expression position and parses as a BARE IDENTIFIER (`kind:"hierarchical", name:{body:"with"}`) — `kw_with` never fires, `array_range_expression` never entered (trace: **0** entries vs **110** for the witnessing two-expression form `{>> e1 e2 with e3}`, whose trace confirms entry `…streaming_concatenation→stream_concatenation→stream_expression→array_range_expression`). ROOT CAUSE (LRM-confirmed, `systemverilog_2017_lrm_extracted.ebnf:1041/1044`): extraction DROPPED the literal `{ }` braces on `stream_concatenation` (`{ stream_expression { , stream_expression } }`) AND the literal `[ ]` brackets on `stream_expression`'s with-clause (`expression [ with [ array_range_expression ] ]`) — the documented dropped-delimiter class — which ALSO makes the grammar REJECT valid SV (`{>>{aa with bb}}`, `{>>4{aa with [bb]}}` both rejected). A generator force-`slice_size` hack would be a WORKAROUND against the fix hierarchy (grammar-FIRST attribution rule, [[feedback_no_workarounds_fix_hierarchy]], [[feedback_prefer_grammar_leave_engine_alone]]). ⇒ `array_range_expression` RE-ROUTED to the C-i grammar-restoration leaf `H.12.5.5.3.3.5`. |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.3` (C-i: `goto_repetition` operator-shadow → `boolean_abbrev`-family LRM `[ ]` DELIMITER-RESTORE) | `done` (`PGEN-GRAMMAR-WELLFORMED-0105`, GRAMMAR FIX, release 1.0.142, schema stays 4, ledger `SV-0004`) | **DONE — SV cert `UNKNOWN 87→86` (`goto_repetition` closed; witness `1203→1204`; spf=0; deterministic seeds 0/7/42; zero newly-unknown; 1291 rules) + a real released-parser bug fixed.** Restored the LRM `[ ]` brackets the extraction dropped on the whole `boolean_abbrev` sequence-repetition family (4 `systemverilog.ebnf` rules, the proven `lbrack…rbrack` idiom, `range:$3`): `consecutive_repetition` (`[*N]`/`[*]`/`[+]`), `goto_repetition` (`[->N]`), `non_consecutive_repetition` (`[=N]`, both profiles). Fixed the dual symptom (bug-finding-oracle hit, same delimiter-drop class as `SV-0002`): pre-fix `a[*3]`/`a[->2]`/`a[=2]`/`a[*]`/`a[+]` REJECTED (valid §A.8.1) + bare `*N`/`=N` wrongly accepted; `->` operator-shadowed `goto_repetition`. Emitted `{kind,range}`/`{range}` shape byte-identical (brackets folded) ⇒ NO schema bump. VERIFIED: post-fix bracketed forms parse + `range`→const_or_range_expression (`--parse-dump-ast-pretty`) + bare `a *3` still parses (multiply, no regress); `--lint-grammar` clean (pre-existing always_matches=6 unchanged); **SV external corpus 14/14** (parse_fail_total=0, both uvm); SV shape-contract GREEN (3/3 aligned) + new Rust lock `systemverilog_sequence_repetition_requires_lrm_brackets`; clippy source-clean. Full lockstep: ledger SV-0004, contract 1.0.142+schema-4 note, SV parser-book changelog+schema-versioning, top-book grammar-wellformedness SV-arc, manifest calibration_history, live docs. SEPARATE finding flagged (future leaf, NOT this slice): `a ##1 b` cycle-delay (`##`) is REJECTED pre-existing (same position with/without repetition; independent of this fix). **Commit: `PGEN-GRAMMAR-WELLFORMED-0105`.** _Original scope:_ `goto_repetition := ( implies const_or_range_expression )` is a bare `-> expr` shadowed by the `operand_chain` `implies` infix operator; IEEE 1800 has `goto_repetition ::= '[' '->' const_or_range_expression ']'` (brackets). **LRM-ground the bracket question against `docs/systemverilog/2017` FIRST** ([[project_ebnf_is_single_source_of_truth]]) — note sibling `consecutive_repetition := ( star const_or_range_expression )` is bare too (`*` is also an operator), so this is a `boolean_abbrev`-family question, not a one-rule typo. If a grammar edit is warranted it is consumer-visible (regen/release/lockstep ceremony); if not, the resolution may be a reach-into-host or a blessed-no-path adjudication. Tools-first; smallest blast radius last. |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4` (C-ii residual: parent-commit ADJUDICATION) | `done` (`PGEN-GRAMMAR-WELLFORMED-0097`, PURE-DOCS ADJUDICATION) | **Tools-first adjudication: BOTH carriers are GRAMMAR-class, NOT (b) parent-commit generator forcing — and they need DIFFERENT grammar fixes.** Baseline reproduced (`total=1292 proof=1 witness=1203 UNKNOWN=88 spf=0`, seed 0 — matches the layer-A pointer). Linter (the adjudicator): `ordered_choice_shadowing=0`; the 6 `always_matches` warnings are unrelated rules ⇒ no SOUND shadowing on either carrier (expected — the relationship is FIRST-set SUBSUMPTION, the unsound-for-PEG class the book deliberately omits, so manual adjudication). **Carrier 1 `direct_index_method_call` (a — effectively DEAD/subsumed):** `bit_select_expression` branch 1 (`priority_first`, tried first) NEVER wins — a routing matrix of 6 diverse `x[body]` inputs (`b.c()`/`this.foo()`/`this.foo`/`super.bar()`/`b.c`/`a.b.c()`) ALL route to `method_call` (branch 2), `direct_index_method` node count 0 every time; trace shows its `method_call_body` settles on the BARE method name leaving the `()`, while `method_call`'s language (`method_call_initial (dot method_call_body)*` incl. `direct_method_call`/`split_direct_callable`) is a SUPERSET; LRM §11.5.1 `bit_select ::= {[expression]}` ⇒ `direct_index_method_call` is a redundant synthesis artifact (branches 2/3/4 cover the LRM). Fix = REMOVE it (decisive A/B parse-neutral; closes `88→87`; referenced ONLY at `:670`) → child `.4.1`. **Carrier 2 `context_member_method_call` (a′ — real latent PARSE GAP / bug-finding-oracle hit) — ⛔ SUPERSEDED by `-0098` (REFUTED, see `.4.2`):** store-gate hypothesis REFUTED (NO `@predicate` in the generated parser — the `:2915` "gated by variable_binding" comment is stale; rejected even with a declared head); its OWN designed form `a.b[0].c()` (`head.member[idx].method()`) is REJECTED parser-wide (sharp boundary: `a[0].c()`✅ `a.c()`✅ `a[0].c`✅ `a.b[0].c`✅ but `a.b[0].c()`❌); valid SV, the exact uvm `elements[i].clone()` shape the rule was authored for; uvm-2020.3.1 has 0 instances of the shape ⇒ corpus never caught it. Fix = root-cause the rejection + REPAIR the grammar so `X.member[i].method()` parses (which witnesses the rule); real released-parser bug ⇒ HIGHEST priority + full grammar-edit lockstep → child `.4.2`. **⛔ This verdict is WRONG (`-0098`): the `@predicate` IS live (`generated:3041-3055`; the no-predicate grep hit a one-line-grep trap), and `a.b[0].c()` PARSES + witnesses the rule with a DECLARED head (`int a; …`) — `-0097`'s failing tests never bound the head. No parser bug; the cert `UNKNOWN` is a store-gated WITNESS-REACH gap (generator fix `.4.2.1`).** Detail: [GRAMMAR-WELLFORMED-H125534-cii-residual-parent-commit-adjudication.md](GRAMMAR-WELLFORMED-H125534-cii-residual-parent-commit-adjudication.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; parser-family rows UNCHANGED (SV stays the only non-fully-certified shipped grammar, `UNKNOWN=88`). **Commit: `PGEN-GRAMMAR-WELLFORMED-0097`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2` (context_member RE-ADJUDICATION) | `done` (`PGEN-GRAMMAR-WELLFORMED-0098`, PURE-DOCS RE-ADJUDICATION) | **The `-0097` "real released-parser bug → repair the grammar" premise is REFUTED, tool-backed — there is NO parser bug; the cert `UNKNOWN` is a store-gated WITNESS-REACH gap.** (1) The `@predicate has_fact(variable_binding, $head)` IS live in the generated parser (`systemverilog_parser.rs:3041-3055`); `-0097`'s "no predicate" was a one-line-grep trap (rustfmt wraps `insert(` and the rule name onto separate lines). (2) `a.b[0].c()` (`head.member[idx].method()`) **PARSES and witnesses `context_member_method_call`** with a *declared* head — `int a; … a.b[0].c()` (also `logic`/`bit`) → accepted, AST node `context_member_method`; the undeclared form correctly rejects (the `post`-predicate "regression firewall" working as designed). `-0097`'s failing tests (`class C; … a.b[0].c()`, `module m; C a; …` with `C` undeclared) never bound the head. ⇒ no grammar fix, no release, no ledger row. TRUE root cause: the witness pass can't *generate* a sample satisfying the `has_fact(variable_binding,$head)` gate — it needs a name-coupled binding-producer **prelude** the minimal reach derivation lacks (the `has_fact` analogue of the regex `\NN` semantic-prelude class; the C2.2 pass at `stimuli_generator.rs:2769` is `fact_count_at_least`-only with no name-coupling). Generator gap by the attribution rule → fix child `.4.2.1`. SECONDARY (separate ticket `.4.2.2`): a class-handle head (`C a;`, C a declared class) does NOT witness the indexed chain (`C a; … a.b[0].c()` rejects; `int a;` works) — a potential real gap for the realistic uvm `elements[i].clone()` shape. Detail: [GRAMMAR-WELLFORMED-H1255334-42-context-member-readjudication.md](GRAMMAR-WELLFORMED-H1255334-42-context-member-readjudication.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; parser-family rows UNCHANGED (SV stays the only non-fully-certified shipped grammar, `UNKNOWN=88`). **Commit: `PGEN-GRAMMAR-WELLFORMED-0098`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1` (context_member semantic-prelude witness) | `active` (CONTAINER — `.4.2.1.1` WHY+WHERE+DESIGN done `-0101`; `.4.2.1.2` FIX-ATTEMPT REFUTED `-0102`; re-scoped into `.4.2.1.2.1` soundness gap + `.4.2.1.2.2` genuine-witness composition) | Witness `context_member_method_call` in cert-coverage (closes `UNKNOWN 88→87`) by extending the semantic-prelude reach (C2.x) from `fact_count_at_least`-gated rules to **`has_fact`**-gated rules. `.4.2.1.1` locked a design; `.4.2.1.2` IMPLEMENTED it and the design's own mandated tools-first check (dump the armed sample) REFUTED it — the prelude alone false-witnesses the gated rule (the post-predicate passes on a degenerate no-`.method()` render and a memo×coverage gap credits a rule the committed parse never enters). Re-scoped: `.4.2.1.2.1` fixes the cert-coverage witness-soundness gap (foundational), then `.4.2.1.2.2` re-introduces the prelude composed with mandatory method-call-structure forcing so the witness is GENUINE (Test-c2-shaped, AST node present). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.1` (context_member prelude WHY+WHERE+DESIGN) | `done` (`PGEN-GRAMMAR-WELLFORMED-0101`, PURE-DOCS WHY+WHERE+DESIGN) | **Tools-first WHY+WHERE + an empirically-validated design.** Reach-path dump: `context_member_method_call`'s shallowest reach is inside an `attribute_instance` `(* attr = const_expr *)` (via `description → attribute_instance → attr_spec → constant_expression → … → call_primary`), and the minimal probe `(*\foo =+\foo .\foo .\foo *)` is **not even a method call** (no `()`) and has no declared head ⇒ `parsed=true witnessed_target=false`. Witness matrix (parser the judge): a top-level decl + a real indexed method call witnesses — `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*)` → `context_member_method` (**Test c2**), proving (★) a TOP-LEVEL `variable_binding` fact is visible to `has_fact` at the later attribute instance — so the existing reach path IS salvageable via a top-level binding-producer prelude (no reach-path re-selection needed) — and that identifiers render canonically `\foo` ⇒ **name-coupling is free**. DESIGN: new `gen_has_fact_gates` map (rule → kind+name-ref), a `has_fact` branch in `compute_reach_prelude` (producer `variable_decl_assignment`, site `source_text := source_text_item*`, `iterations=1`, NO numeric capture / NO count-prune bypass — the gate is a PARSE-time post-predicate so only the prelude TEXT is needed since SV is not `store_aware_gen`). Open composition question handed to the FIX: the binding prelude (plannable pass) and the mandatory `.method()` structure (target-own pass / `.3.3.1` mandatory-child lineage) must co-occur in ONE sample. KM card [sv-store-fact-scope-and-canonical-name-coupling](../knowledge/sv-store-fact-scope-and-canonical-name-coupling.md). Detail: [GRAMMAR-WELLFORMED-H125533421-context-member-prelude-whywhere-design.md](GRAMMAR-WELLFORMED-H125533421-context-member-prelude-whywhere-design.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV `UNKNOWN=88` unchanged. **Commit: `PGEN-GRAMMAR-WELLFORMED-0101`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2` (context_member `has_fact` prelude FIX ATTEMPT → REFUTED) | `done` (`PGEN-GRAMMAR-WELLFORMED-0102`, FIX-ATTEMPT REFUTED + foundational discovery, landed PURE-DOCS) | Implemented the `-0101` design faithfully (new `gen_has_fact_gates` map + `compute_gen_has_fact_gates` + a `PreludeKind` discriminator + a `has_fact` branch in `compute_reach_prelude` factored into a shared `build_semantic_prelude`, `iterations=1`). It compiled clean and the seed-0 metric moved exactly as the row predicted — `UNKNOWN 88→87`, witness `1203→1204`, spf=0. **But the design's own mandated check ("dump the armed sample; confirm decl + `.method()`") REFUTED it: the witness is FALSE.** The plannable pass (not the target-own pass) witnessed it on a sample `(*\foo =+type(struct{…bit\foo ;…})*)(*\foo =+\foo .\foo .\foo *);` whose chain is a bare hierarchical ref with **no `()`** — `parseability_probe --parse-dump-ast-pretty` shows **0** `context_member_method` AST nodes (vs **1** for the genuine Test c2 `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*)`), and `--trace-rules context_member_method_call` shows only backtracks (no positive success-exit). The prelude's binding makes the `has_fact` post-predicate PASS on the plannable pass's **degenerate (no-`.method()`) render**, and a memo × transactional-coverage composition gap (the `H.10.2.2` class, on the post-predicate-passes-then-structurally-fails path) credits the rule (id 547) as witnessed though it never enters the committed parse. **REVERTED** (cert honest at `UNKNOWN=88`, rebuild-verified). Per [[feedback_always_signoff_decisions]] a false witness must not land. KM card [sv-cert-coverage-predicate-gated-false-witness](../knowledge/sv-cert-coverage-predicate-gated-false-witness.md). Detail: [GRAMMAR-WELLFORMED-H1255334212-context-member-prelude-fix-attempt-refuted.md](GRAMMAR-WELLFORMED-H1255334212-context-member-prelude-fix-attempt-refuted.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0102`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.1` (cert-coverage witness-soundness gap — degenerate predicate-passing render must NOT be witnessed) | **WHY+WHERE `done`** (`-0103`) → **FIX ATTEMPTED + REVERTED → `deferred`** (director ruling 2026-06-17 — the engine fix regresses; documented known limitation) | **⛔ DEFERRED — DOCUMENTED KNOWN LIMITATION (not "fixed").** The engine fix (multi-branch LongestMatch tournament loser-branch coverage rollback, symmetric with the C3-B semantic handling) is tool-verified CORRECT (deterministic seeds 0/7/42; two-way regression test passes; 5/6 fully-certified grammars byte-identical) but REGRESSES the tracker — the leak was INFLATING witness counts, so the honest cert is svpp `Done`→`UNKNOWN=1` (`kw_none`) and SV `88→146` (58 false witnesses). One shared codegen change ⇒ can't fix one grammar without exposing all ⇒ ≥58 genuine witnesses required before it commits without regression = the whole SV cert endgame. A wide-blast soundness fix is the wrong, non-targeted tool for one demonstrated false witness; REVERTED, baseline intact. The honest re-baseline is a deliberate director call. See Decisions `2026-06-17` + [[project_cert_coverage_tournament_loser_leak]]. _Original WHY+WHERE:_ **Reproduced the gap on a PLAIN input (no reverted prelude): `int \foo ; (*\foo =+\foo .\foo .\foo *);` → `context_member_method_call` ∈ `exercised_rule_names` but 0 `context_member_method` AST nodes.** Pinned the leak (`--trace-rules` + engine-source read): the rule's body memoizes a structurally-degenerate SUCCESS, its `has_fact` post-predicate PASSES, the committed parse routes the bytes through a SIBLING (rule absent from AST) — and its coverage push, never truncated by `try_parse` on the discard, is frozen into the `coverage_delta` of an ANCESTOR memo entry (`attribute_instance`) that is memo-hit on the committed path and replayed → false witness. `H.10.2.2`/`.b.6.2.36.4` memo×coverage class, new trigger. Root: a coverage push is removed ONLY by `try_parse` truncate; a success-then-reject (post-predicate-reject `:1835`/`:1874`, or a committed-then-abandoned ordered-choice branch) leaves it. FIX candidates: (a) truncate `coverage_stack` on the `with_semantic_runtime_rule_transaction` reject/error path symmetric with `try_parse`; (b) at memo capture (`:6383`) exclude coverage ids not in the memo's output node. Detail: [docs/tasks/GRAMMAR-WELLFORMED-H1255334212-1-cert-coverage-witness-soundness-whywhere.md](GRAMMAR-WELLFORMED-H1255334212-1-cert-coverage-witness-soundness-whywhere.md). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.2` (context_member GENUINE-witness composition — CONTAINER) | `active` (split 2026-06-17 by `-0107`: composition RESOLVED + safe design ⇒ `.2.2.1` done, `.2.2.2` implement) | The `-0101` open composition question is RESOLVED tools-first: the minimal genuine witness is `int \foo ; (*\foo =+\foo .\foo .\foo ()*)` (decl + ≥1 member + CALL; index NOT required — proven `1` `context_member_method` AST node), and the safe arming avoids BOTH the `-0102` plannable false witness AND the `.4.2.1.2.1` deferred soundness gap. Split into `.2.2.1` (DESIGN, done) + `.2.2.2` (GENERATOR IMPLEMENT, frontier). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.2.1` (context_member GENUINE-witness composition WHY+WHERE + SAFE DESIGN) | `done` (`PGEN-GRAMMAR-WELLFORMED-0107`, PURE-DOCS WHY+WHERE+DESIGN) | **Tools-first, parser-judged.** (1) Minimal recipe pinned: top-level decl (satisfies `has_fact(variable_binding,$head)` post-gate) + `callable_method_call_body` rendered as a CALL — `[idx]` NOT required (`int \foo ; (*\foo =+\foo .\foo .\foo ()*)` → **1** `context_member_method` node; same WITHOUT decl → **0**). (2) Two gaps pinned on the shipped generator (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`): **A** SV has NO prelude (`gen_count_kinds` empty ⇒ `compute_reach_prelude`→`None`); **B** the `.3.3.1` child-forcing renders `[idx]` and `()` in SEPARATE probes, never together, never with a binding; a binding+bare-ref probe IS the `.4.2.1.2.1` FALSE-witness shape (gap-credited, 0 AST nodes). (3) SAFE composition: a `Presence` (`has_fact`) prelude (`captured=None` ⇒ the plannable driver never arms it ⇒ no `-0102` trap), armed ONLY in the target-own pass, ONLY on the all-mandatory-children (call-forced) probe ⇒ c2-shaped GENUINE witness; genuineness oracle = the AST node, never the count. Edit surface mapped (`ReachPrelude`+`PreludeKind`, `gen_has_fact_gates`, `has_fact` branch in `compute_reach_prelude`, target-own arm+compose). Detail: [GRAMMAR-WELLFORMED-H1255334212-2-context-member-genuine-witness-design.md](GRAMMAR-WELLFORMED-H1255334212-2-context-member-genuine-witness-design.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; SV stays `UNKNOWN=86`. **Commit: `PGEN-GRAMMAR-WELLFORMED-0107`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.2.2` (context_member GENUINE-witness composition — GENERATOR IMPLEMENT ATTEMPT) | `refuted` (`PGEN-GRAMMAR-WELLFORMED-0110`, PURE-DOCS — implemented faithfully, measured, two `-0107` assumptions disproven, REVERTED) | The `-0107` safe design was implemented GENERATOR-only (`PreludeKind {Count,Presence}` + `gen_has_fact_gates` + `compute_gen_has_fact_gates` + a `Presence` branch in `compute_reach_prelude` factored into a shared `build_semantic_prelude` + `kind==Count`-guarded consumers + the composed target-own probe). The detection chain worked (`PGEN_PRESENCE_DEBUG`: `gate=Some([("variable_binding","head")])`, `set_ok=true`, `armed=true`) but the genuineness oracle REFUTED it: **(A)** the innermost-first scan picked an inner `(description,root/o5/s0)` site reaching the producer via a STRUCT MEMBER (wrong scope), not the file-scope `source_text` site the design assumed; **(B)** the `has_fact(variable_binding,$head)` gate is NAME-sensitive (`int \bar ; (*\foo …)` → 0 nodes; `int \foo ; …` → 1) and the injected decl renders `\foo` while the on-path head renders `cBN` ⇒ the composed sample does not even parse. SAFE (no false witness; `UNKNOWN` stayed 86, byte-identical). BLOCKED on generation-time name/value-selection (`STORE-AWARE-GEN.4b`) → re-scoped to `.4.2.1.2.2.3`. Detail: [GRAMMAR-WELLFORMED-H1255334212-2-2-context-member-implement-refuted.md](GRAMMAR-WELLFORMED-H1255334212-2-2-context-member-implement-refuted.md). |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.2.3` (context_member GENUINE-witness via gen-time name-coupling) | `blocked` (on `STORE-AWARE-GEN.4b` value-selection) | Re-scoped from `.2.2.2`'s refutation: a genuine `context_member_method_call` witness needs the injected file-scope declaration's name to EQUAL the on-path chain head's name (the `has_fact(variable_binding,$head)` gate is name-sensitive). Requires (1) a `Presence` file-scope site selector (prefer the entry-level `source_text` quantifier over inner producer-reaching sites) AND (2) generation-time value-selection so the chain head consults the emitted `variable_binding` fact and renders the SAME name — the generation-side dual of the parser gate, owned by `STORE-AWARE-GEN.4b`. Design-first when that capability lands; do NOT chase with a fragile name-coupling hack. |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.2` (class-handle head WHY+WHERE) | `done` (`PGEN-GRAMMAR-WELLFORMED-0099`, PURE-DOCS INVESTIGATION — a real store-gating parser bug found) | **Tools-first WHY+WHERE: a real released-parser bug in the M2 store-gating family.** `C a;` (C a declared class) at MODULE scope is mis-parsed as a `net_declaration` (AST-confirmed; `--trace-rules` shows `checked_nettype_identifier` matching `C`), so `a` gets no `variable_binding` fact and `context_member_method_call`'s `has_fact(variable_binding,$head)` gate fails → the indexed chain rejects. WHERE: `checked_nettype_identifier`'s gate (`systemverilog.ebnf:3310`) is `has_fact(type_name, $body)` — UNDER-specified: a class is also a `type_name`, so a class name satisfies the nettype gate (the rule's own comment says the intent is `declaration_family: nettype`, but the predicate never checks it — a [[feedback_grammar_rules_must_consult_store]] defect). LRM: a class is not a nettype ⇒ `C a;` is a `data_declaration`, not a `net_declaration`. FIX (verified-sound, → child `.4.2.2.1`): tighten to `fact_attribute_equals(type_name, $body, declaration_family, nettype)`. Preconditions tool-verified: **P1** `nettype logic NT; … NT a;` parses (nettypes preserved — they carry `declaration_family: nettype`); **P2** the SAME class-handle indexed chain PARSES and witnesses `context_member_method` in CLASS scope (where net_declaration isn't a `class_item` so only data_declaration matches + binds `a`) — exactly the module-scope outcome the fix reproduces. Detail: [GRAMMAR-WELLFORMED-H1255334-422-classhandle-nettype-whywhere.md](GRAMMAR-WELLFORMED-H1255334-422-classhandle-nettype-whywhere.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV `UNKNOWN=88` unchanged. **Commit: `PGEN-GRAMMAR-WELLFORMED-0099`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.2.1` (checked_nettype_identifier store-gate fix) | `done` (`PGEN-GRAMMAR-WELLFORMED-0100`, GRAMMAR FIX, release 1.0.141, schema stays 4, ledger `SV-0003`) | **DONE — a real released-parser bug fixed: module-scope class-handle `C a;` was mis-parsed as a `net_declaration`; now `data_declaration`.** ONE-line grammar change: `checked_nettype_identifier`'s `@predicate` (`systemverilog.ebnf:3310`) tightened `has_fact(type_name, $body)` → `fact_attribute_equals(type_name, $body, declaration_family, nettype)` (the proven `known_unscoped_block_class_type`/`checked_type_identifier` store-gate pattern; the parser-agnostic engine UNTOUCHED). VERIFIED (all green): `C a;` → `data_declaration`/`variable_decl` (binds `a`; was `net_declaration`); module-scope `C a; … a.b[0].c()` REJECT→PASS + witnesses `context_member_method`; `wire a;` still `net_declaration`; `nettype logic NT; NT a;` preserved; undeclared-head chain still rejected. SV cert-coverage byte-identical `total=1292 witness=1203 UNKNOWN=88 spf=0` seeds 0/7/42 (UNKNOWN SET identical — correctness-only re-route; `context_member_method_call`'s cert witness is the separate `.4.2.1` generator pass); `--lint-grammar` clean; **SV external corpus 14/14**; `stimuli_cross_family_platform_gate` PASS (regex/vhdl/sv); SV shape-contract GREEN + new Rust regression lock `systemverilog_class_handle_decl_is_data_declaration_not_net`; clippy source-clean. Full lockstep: ledger `SV-0003`, contract 1.0.141 schema-note, SV parser-book changelog-index + schema-versioning, grammar-wellformedness book SV-arc narrative. Detail: [GRAMMAR-WELLFORMED-H1255334-422-classhandle-nettype-whywhere.md](GRAMMAR-WELLFORMED-H1255334-422-classhandle-nettype-whywhere.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0100`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.1` (direct_index dead-branch removal) | `done` (`PGEN-GRAMMAR-WELLFORMED-0104`, GRAMMAR FIX — parse-neutral dead-branch removal, release/schema UNCHANGED) | **DONE — SV cert `UNKNOWN 88→87` (`direct_index_method_call` leaves the 1292→1291 rule set; witness `1203` UNCHANGED; spf=0; deterministic seeds 0/7/42).** Removed the effectively-dead, subsumed `direct_index_method_call` rule + its `bit_select_expression` branch 1; `bit_select_expression` now 3 alts (method / dollar / expression). RE-VERIFIED the "dead" premise tools-first FIRST (the `-0097` adjudication was flagged unreliable): **0** committed `direct_index_method` AST nodes on 6 inputs shaped exactly for it (`b[c.d()]` / `b[this.d()]` / `b[pkg::c.d()]` / `b[super.d()]` / `b[c.d.e()]` / `b[c::d.e()]` → all route to `method`); never-witnessed cert status; structural subsumption by `method_call`'s `( dot method_call_body )*` (chains ≥ as far); no live fixture / expected-AST / corpus node anywhere; no orphan cascade (sub-rules used 10–36× elsewhere). VERIFIED (decisive A/B): regen SV → witness 1203 unchanged + `UNKNOWN 88→87` + spf=0 (seeds 0/7/42); **SV external corpus triage 14/14** (`parse_fail_total=0` — no acceptance regression); SV shape-contract GREEN against the regenerated parser; clippy source-clean (generated non-strict). Parse-neutral (orphan never selected ⇒ wire-shape identical) ⇒ NO release/schema/ledger bump (precedent: 25 shadow branches `.7.4.6.7`, 48 number orphans `H.12.1`, `white_space`). Lockstep: contract typed-AST table (`bit_select_expression` 4→3 kinds; `direct_index_method_call` row removed), manifest calibration_history, grammar-wellformedness book SV-arc (the `88→87` dead-branch-removal beat). **Commit: `PGEN-GRAMMAR-WELLFORMED-0104`.** |
| 2 | `GRAMMAR-WELLFORMED.H.12.5.5.3.3.5` (C-i: stream LRM delimiter-drop GRAMMAR fix) | `done` (`PGEN-GRAMMAR-WELLFORMED-0096`, GRAMMAR FIX, release 1.0.140, schema 3→4, ledger `SV-0002`) | **DONE — SV cert `UNKNOWN 89→88` (`array_range_expression` closed; witness `1202→1203`; spf=0; seeds 0/7/42; zero newly-unknown; 1292 rules unchanged) + a real released-parser bug fixed.** Implemented the `-0095` design (3 `systemverilog.ebnf` rules): `stream_concatenation := lbrace stream_expression ( comma stream_expression )* rbrace -> {body: [$2, $3::2*]}` (mandatory inner `{ }`; `body` raw→array), `stream_expression := expression ( kw_with lbrack array_range_expression rbrack )?` (literal `[ ]`), AND a tools-discovered refinement BEYOND the design: `streaming_concatenation`'s `( slice_size )?` → `( slice_size &lbrace )?`. WHY the refinement: restoring the mandatory inner brace exposed a latent PEG greediness (`slice_size`'s `constant_expression` ate the brace-concatenation that IS the mandatory `stream_concatenation`, then failed without backtracking — the probe matrix proved it REGRESSED the canonical `{>>{data}}`/`{<<{v}}` forms real UVM uses); the `&lbrace` guard encodes the LRM fact that a slice_size is always followed by the stream-concatenation's `{` (transparent — `{op,slice_size,body}` byte-identical). Effect: with-bracket LRM form `{<< 4 {a with [3:0]}}` parses (was REJECTED ≤1.0.139); canonical brace forms parse with the correct structure; bare `with X` without brackets correctly rejected. VERIFIED: `--lint-grammar` clean; SV external corpus 14/14; probe matrix all-accept; `stimuli_cross_family_platform_gate` PASS; AST shape via `--parse-dump-ast-pretty`; clippy source-clean (generated non-strict). Note for `.3.3.3`: the new `lbrack array_range_expression rbrack` is the with-clause bracket lane and does NOT by itself resolve `goto_repetition` (a distinct `( implies … )` operator-shadow bracket lane — still LRM-ground first). Commit: `PGEN-GRAMMAR-WELLFORMED-0096`. _Original scope text:_ **GRAMMAR FIX (code, language-changing) — restores LRM-literal delimiters dropped in extraction, closing the `array_range_expression` cert `UNKNOWN` AND a real PARSE GAP.** Edit two `systemverilog.ebnf` rules to match `systemverilog_2017_lrm_extracted.ebnf:1041/1044`: `stream_concatenation := lbrace stream_expression ( comma stream_expression )* rbrace` (was `( stream_expression ( comma stream_expression )* )*` — MISSING `{ }`, spurious outer `*`) and `stream_expression := expression ( kw_with lbrack array_range_expression rbrack )?` (was `… ( kw_with ( array_range_expression )? )?` — MISSING `[ ]`, array-range wrongly optional-inside). WHY it closes the cert: the restored `{ }` braces SEPARATE `slice_size` from the stream_expression's leading expression (killing the `( slice_size )?` theft that blocked the witness — see `-0094`), and the `[ ]` brackets make the array-range token unambiguous; valid SV `{>>4{aa with [bb]}}` then parses with `array_range_expression` entered. FULL PROOF REQUIRED (a parse-acceptance change): regen SV parser → cert seeds 0/7/42 (close `array_range_expression` + GLOBAL no-regress on the other 88) + external corpus 14/14 + `stimuli_cross_family_platform_gate` + `--lint-grammar`; then SV release bump + `PGEN_RELEASED_PARSER_BUG_LEDGER` row + SV contract/parser-book sync + AST-shape/schema check. Tools-first; change ONE thing; measure GLOBAL. Note: also confirm whether the new `lbrack array_range_expression rbrack` resolves the C-i `goto_repetition` bracket lane (`.3.3.3`) idiom. |
| — | `GRAMMAR-WELLFORMED.H.12.5.6` (M2 over-generation / store-unfaithful) | `active` (SPLIT 2026-06-17 by `.6.1` — re-enumerated at `UNKNOWN=84`; the `-0083` M2 label was made at 121) | The M2 set (`parsed=false` — the parser rejects the witness) is **36 rules in THREE mechanisms**, not one over-gen/store class: **M2a** constraint/sequence/property bodies via a store-gated out-of-class context (≈18, → `.6.2`); the **sequence-operator** trio (`kw_intersect`/`kw_or`/`kw_within`) = a REAL parser bug (whole `sequence_expr`/`property_expr` binary-operator layer rejected) → routed to `H.12.5.8` (lane 2, broadened); **M2b** store-gated identifiers whose witness never establishes the fact (≈15, → `.6.3`, parked-adjacent on `STORE-AWARE-GEN.4b`). Children: `.6.1` (re-enumeration+adjudication, done `-0112`), `.6.2` (M2a constraint reach-honesty fix, pending — LEAD), `.6.3` (M2b store-gated, pending). |
| — | `GRAMMAR-WELLFORMED.H.12.5.6.1` (M2 re-enumeration + over-gen-vs-parser-bug adjudication) | `done` (`PGEN-GRAMMAR-WELLFORMED-0112`, PURE-DOCS INVESTIGATION) | Re-enumerated the SV `UNKNOWN=84` residual tools-first (`PGEN_CERT_COVERAGE_DUMP_ALL=1`/`DEBUG_PROBES=1`, count 40 seed 0): **84 = 19 `no_path` (NON-defects, `H.12.6`) + 36 M2 + 14 M1-residual (→`H.12.5.5`) + 15 M3 (→`H.12.5.7`)**, sums exactly. Adjudicated M2's three mechanisms via `parseability_probe`: (M2a) the constraint cluster is witnessed only through the store-gated out-of-class `constraint \foo ::\foo {…}` (an undeclared class) — parser CORRECT, fix = reach-honesty preference for the proven non-gated in-class path (`.6.2`, M1-reach family, NOT the parked prelude); (M2-seq) `kw_intersect`/`kw_or`/`kw_within` exposed a REAL released-parser bug — `a and b`/`a or b`/`a intersect b`/`a within b`/`a ##1 b` ALL reject while bare `a` parses (`sequence_expr@4573` 5-branch left-recursion, LR-elim-defect signature, the `-0111` family) → `H.12.5.8` broadened; (M2b) `localparam \foo \foo ;`/`typedef \foo \foo ;` store-gated identifiers whose minimal witness never declares the type/class → `.6.3` (parked-adjacent). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; rows UNCHANGED — SV stays the only non-fully-certified shipped grammar (`UNKNOWN=84`). Detail: [GRAMMAR-WELLFORMED-H12561-m2-reenumeration-adjudication.md](GRAMMAR-WELLFORMED-H12561-m2-reenumeration-adjudication.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0112`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.6.2` (M2a constraint reach-honesty fix) | `done` (2026-06-21 via `.2.2.2` `-0115`) | The M2a constraint/sequence/property rules witnessed only through the store-gated out-of-class `extern_constraint_declaration` (`constraint <class_scope>::<name> {…}`), not the non-gated in-class `constraint_declaration`. FIXED by the strictly-additive store-free reach pass (`-0115`): SV `UNKNOWN 84 → 67`, the `constraint_block`-subtree subset (17 rules) now witnessed via the in-class route; `extern_constraint_declaration` + `known_unscoped_class_scope_*` + `constraint_set` remain UNKNOWN (extern-only / store-gated ⇒ STORE-AWARE-GEN; residual `constraint_set`). Children all done: `.2.1` (WHY+WHERE+DESIGN, `-0113`), `.2.2` → `.2.2.1` (empirical confirm + refined design, `-0114`) + `.2.2.2` (engine implement, `-0115`). |
| — | `GRAMMAR-WELLFORMED.H.12.5.6.2.1` (M2a reach-honesty WHY+WHERE + fix DESIGN) | `done` (`PGEN-GRAMMAR-WELLFORMED-0113`, PURE-DOCS DESIGN) | Read the reach path BEFORE changing it (leaf mandate). WHY (tool-confirmed): the rejected witness is `extern_constraint_declaration_sv_2017` (`systemverilog.ebnf:2055` = `constraint class_scope constraint_identifier constraint_block`), store-gated via `class_scope` (`:1068`→`known_unscoped_class_scope_class_identifier` `:1025`, a *declared class*); the minimal `\foo` witness can't declare it ⇒ gate fails ⇒ body never witnesses. The non-gated in-class `constraint_declaration_sv_2017` (`:1395`, no `class_scope`) reaches the SAME `constraint_block → constraint_block_item → constraint_expression` subtree and parses without a store fact. WHERE: `reach_hops` (`stimuli_generator.rs:5606`) is a SHORTEST-path BFS; the out-of-class rule is a top-level `package_or_generate_item_declaration` ref (`:3584`/`:3601`, FEWER hops) vs the in-class `class_constraint` ref (`:976`/`:986`, must descend `class_declaration → class_item` first) ⇒ BFS prefers the gated rule. FIX DIRECTION: deprioritize reach edges crossing a store-gated rule whose consulted fact-kind no on-path `@emit_fact` producer establishes (re-use the linter `F1` emitter/consumer machinery), via a two-pass-BFS (exclude-gated-then-fallback) or weighted BFS — same bias-hook family as `prefer_non_self_recursive_reference_sites` (`:5714`), GENERAL/parser-agnostic, inert for the 6 fully-certified grammars. PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; rows UNCHANGED (SV `UNKNOWN=84`). Detail: [GRAMMAR-WELLFORMED-H125621-m2a-constraint-reach-honesty-whywhere.md](GRAMMAR-WELLFORMED-H125621-m2a-constraint-reach-honesty-whywhere.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0113`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.6.2.2` (M2a reach-honesty engine fix) | `done` (2026-06-21 via `.2.2.2` `-0115`) | The `.2.1` design was confirmed + REFINED by `.2.2.1`, then IMPLEMENTED by `.2.2.2` (`-0115`): the strictly-additive store-free reach pass landed SV `UNKNOWN 84 → 67`, deterministic, no newly-UNKNOWN, 6 fully-certified grammars byte-identical. Children both done: `.2.2.1` (empirical confirm + refined design, `-0114`), `.2.2.2` (the engine implement, `-0115`). |
| 3 | `GRAMMAR-WELLFORMED.H.12.5.6.2.2.1` (M2a reach-honesty empirical confirm + refined design) | `done` (`PGEN-GRAMMAR-WELLFORMED-0114`, PURE-DOCS) | Rebuilt-from-source DEBUG `ast_pipeline`: reproduced the decisive baseline (`total=1289 UNKNOWN=84 spf=0`, matches layer A) + CONFIRMED the `-0113` WHY via `PGEN_REACH_PATH_DUMP=1` (resolves open-Q (a): re-routing the discovery edge suffices, but must beat a SHORTER gated route — a same-distance tie-break is insufficient). THREE refinements: (R1) the re-routable subset is the `constraint_block` subtree only — `extern_constraint_declaration` itself is extern-ONLY (needs STORE-AWARE-GEN) ⇒ the `-0113` "→~66" target is optimistic; (R2) all 4 `class_scope_type` head alternatives ARE store-gated (verified) ⇒ the conservative "Or gated iff all alts gated" detector rule fires on the edge into `extern_constraint_declaration`; (R3) NEW second mechanism — `scoped_class_scope_identifier`'s NEGATIVE `lacks_fact` predicate is satisfiable by an undeclared name ⇒ the cluster is ALSO influenced by which `class_scope_type` alternative the reach pass selects (a lever re-routing can't reach). Safety: `reach_hops` path selection can't create a false witness (parser adjudicates). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; rows UNCHANGED (SV `UNKNOWN=84`). Detail: [GRAMMAR-WELLFORMED-H1256221-m2a-reach-honesty-empirical-confirm.md](GRAMMAR-WELLFORMED-H1256221-m2a-reach-honesty-empirical-confirm.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0114`.** |
| 4 | `GRAMMAR-WELLFORMED.H.12.5.6.2.2.2` (M2a reach-honesty engine implement) | `done` (`PGEN-GRAMMAR-WELLFORMED-0115`, GENERATOR-ONLY) | **DONE (store-free reach pass): SV `UNKNOWN 84 → 67`** (witness 1204→1221, deterministic seeds 0/7/42, `spf=0`, **NO newly-UNKNOWN**; the 6 fully-certified grammars byte-identical; clippy ✅). STRICTLY-ADDITIVE design: iteration A's two-pass `reach_hops` was REJECTED — the decisive A/B showed it de-witnessed 1 store-gated bystander (`known_unscoped_checker_identifier`), unacceptable per [[project_cert_coverage_tournament_loser_leak]]; iteration B reverts `reach_hops` to the original all-edges BFS + runs a store-free pass LAST over only residual UNKNOWN ⇒ no newly-UNKNOWN by construction. The store-gate detector (`compute_reach_gate_kinds`/`edge_is_store_gated`/`mandatory_reach_gate`/`mandatory_node_gated`) is parser-agnostic + guarded truly inert for predicate-free grammars. 17 closed rules = the `constraint_block` subtree + adjacents; `extern_constraint_declaration`/`known_unscoped_class_scope_*`/`constraint_set` stay UNKNOWN (STORE-AWARE-GEN / residual). R3 (alt-selection lever) analyzed + deferred (change-one-thing; not needed for the subtree subset). Detail: [GRAMMAR-WELLFORMED-H1256222-m2a-reach-honesty-engine-implement.md](GRAMMAR-WELLFORMED-H1256222-m2a-reach-honesty-engine-implement.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0115`.** ORIGINAL PLAN: implement the `.2.2.1` refined design in `reach_hops` (`stimuli_generator.rs`): two-pass BFS deprioritizing store-gated edges (pass-1 exclude-gated, pass-2 fallback) + emitted-on-path tracking in the BFS `Discovery` struct, reusing the F1 `collect_emitted_fact_kinds`/`consulted_kinds_in_predicate` machinery; settle the R3 second mechanism (satisfiable `class_scope_type` alternative selection) tools-first. GENERATOR-only ⇒ NO release/schema bump. PROOF (corrected): decisive A/B SV cert seeds 0/7/42 — the `constraint_block`-subtree subset witnessed (NOT all ≈18; extern-only + store-gated-id rules stay `UNKNOWN` for STORE-AWARE-GEN), `spf=0`, no newly-UNKNOWN; the 6 fully-certified grammars byte-identical (stash A/B); `--generate-stimuli` + cross-family + oracle byte-identical; lockstep grammar-wellformedness book SV-arc + continuity docs. ⚠️ HEAVY slice (build + 3-seed cert + 7-grammar verify) — best done with fresh, sharp context. Open Qs in `.2.2.1`: (a confirmed) re-discovery suffices; (b) `constant_cast`/`property_qualifier` per-rule context; (c) `loop_variables`/`index_variable_identifier` ride-along; (d NEW) does steering `class_scope_type` to the satisfiable alternative witness `extern_constraint_declaration` itself. |
| — | `GRAMMAR-WELLFORMED.H.12.5.6.3` (M2b store-gated identifiers) | `pending` (parked-adjacent) | The ≈15 M2b store-gated identifier rules (`known_unscoped_{class_scope_class,class_scope_interface_class,block_type,data_type,block_class_type,covergroup_type,block_covergroup,interface_class_type}_identifier`, `provisional_unscoped_block_class_type`, `checked_type_identifier`, `checked_nettype_identifier`, `wildcard_escape_nettype_identifier`, `declared_class_alias_identifier`) whose minimal witness never establishes the consulted declared-type/class fact. Same family as the PARKED `context_member_method_call`; BLOCKED on `STORE-AWARE-GEN.4b` gen-time name/value-coupled prelude synthesis (director STANDING: do NOT chase with a fragile name-coupling hack). Per-rule re-check for a non-gated reach path (M2a-style) before parking each. |
| — | `GRAMMAR-WELLFORMED.H.12.5.7` (M3 property/sequence temporal operators) | `done` (SPLIT + both children resolved 2026-06-21: `.7.1` WHY+WHERE/adjudication `-0116`; `.7.2` prefix-operator reach extension `-0117`, SV `UNKNOWN 67→56`) | The M3 property/sequence temporal-operator cluster in the `UNKNOWN=67` residual. The **prefix** operators are now WITNESSED (`.7.2` generator-reach fix); the **infix** `until`-family + sequence `intersect`/`within` are folded into `H.12.5.8` (LR-elim parse defect, lane 2). Children below. |
| — | `GRAMMAR-WELLFORMED.H.12.5.7.1` (M3 temporal WHY+WHERE + parse/reject adjudication) | `done` (`PGEN-GRAMMAR-WELLFORMED-0116`, PURE-DOCS INVESTIGATION) | Tools-first WHY+WHERE (cert `DUMP_ALL`/`DEBUG_PROBES` + `parseability_probe`, count 40 seed 0): the **prefix** temporal operators (`accept_on`/`eventually`/`nexttime`/`reject_on`/`s_always`/`s_eventually`/`s_nexttime`/`sync_accept_on`/`sync_reject_on`/`kw_constant`/`property_case_item`) **PARSE** but the plannable pass `generation_failures` (no probe emitted, NOT in `no_path`) on the deep **indirectly-left-recursive** `property_expr` descent — a genuine generator-reach gap. The **infix** `until`-family (`until`/`until_with`/`s_until`/`s_until_with`) **REJECTS** at the operator (`furthest=47`, the SAME locus as the known `a or b`/`a and b`/`a ##1 b` defect), as do sequence `intersect`/`within` — the LR-elim parse defect, NOT a reach gap → folded into `H.12.5.8`. SV stays `UNKNOWN=67`. Detail: [GRAMMAR-WELLFORMED-H12571-m3-temporal-whywhere.md](GRAMMAR-WELLFORMED-H12571-m3-temporal-whywhere.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0116`.** |
| — | `GRAMMAR-WELLFORMED.H.12.5.7.2` (M3 prefix-operator plannable-reach extension) | `done` (`PGEN-GRAMMAR-WELLFORMED-0117`, GENERATOR-ONLY) | **DONE — SV cert `UNKNOWN 67→56` (−11), witness 1221→1232, `generation_failures 29→0`, deterministic seeds 0/7/42 `spf=0`, ZERO newly-UNKNOWN; the 6 fully-certified grammars intact (json/regex/svpp/vhdl/rtlfe/rtlce); `--generate-stimuli` byte-identical (decisive stash A/B, md5 identical, pre-fix re-confirmed 67); clippy source-clean. 11 resolved = the prefix temporal cluster (`accept_on`/`eventually`/`nexttime`/`reject_on`/`s_always`/`s_eventually`/`s_nexttime`/`sync_accept_on`/`sync_reject_on`/`property_case_item`) + a `constant_cast` bonus; the infix `until`-family/`intersect`/`within` now GENERATE but parser-reject ⇒ stay UNKNOWN → `H.12.5.8`; `kw_constant` stays UNKNOWN (optional sub-branch, deferred).** WHY+WHERE CONFIRMED tools-first (corrects `.7.1`'s "LR-eliminated" half):** the reach plan IS found + targets the right branch (e.g. `kw_eventually → property_expr_sv_2017 root/o21/s0`) and `property_expr_sv_2017` is NOT LR-eliminated (original `o0..o33` indices). ROOT CAUSE (`stimuli_generator.rs:7104-7118`): the `.5.6` `suppress_recursive_forced_branch` guard only catches **direct** self-recursion (`refs.contains(current_rule)`); the prefix branch `kw_eventually (range)? property_expr` references `property_expr` (the one-hop wrapper `property_expr := property_expr_sv_2017`), so the directive re-fires on every **indirect** re-entry → forced `eventually eventually …` → depth/`max_rule_visits` exhaustion → generation `Err` (the 29 `generation_failures`). FIX (generalise `.5.6` direct→indirect): broaden the self-reference test to transitive `rule_can_reach(branch_ref, current_rule)`, gated behind the cheap `call_stack.count(current_rule) >= 2` re-entry check. Generator-only, parser-agnostic, strictly additive (no reach plan off the witness pass ⇒ `--generate-stimuli` byte-identical); decisive A/B + global cert + `spf` seeds 0/7/42; the 6 fully-certified grammars byte-identical; commit ONLY an improvement. Detail: [GRAMMAR-WELLFORMED-H12572-prefix-temporal-reach-extension.md](GRAMMAR-WELLFORMED-H12572-prefix-temporal-reach-extension.md). |
| 2 | `GRAMMAR-WELLFORMED.H.12.4` (cert `no_path`/`UNKNOWN` full-dump observability) | `done` (`PGEN-GRAMMAR-WELLFORMED-0079`, OBSERVABILITY-ONLY) | **TOOLING (code change, `main.rs` cert-coverage report PRINT path only — no grammar/parser/generated/engine change): an env-gated full dump lifts the report's `no_path@10` / `UNKNOWN@25` print caps so the residual can be enumerated + adjudicated deterministically.** WHY (tools-first): the report capped the `no_path` warning at 10 and the `UNKNOWN` list at 25 (`UNKNOWN rules (25 of 123 shown)`), so the full 20-no_path enumeration + the full 123-UNKNOWN list — the prerequisite data for the `H.12.5` A1-vs-A2 adjudication + residual classification — could not be read from stdout. FIX (additive, `rust/src/main.rs` `run_certificate_coverage_report`): `let dump_all = std::env::var_os("PGEN_CERT_COVERAGE_DUMP_ALL").is_some();` (presence-gated, the sibling idiom of `PGEN_CERT_COVERAGE_DEBUG_PROBES`) drives `let shown = if dump_all { len } else { len.min(10\|25) };` at BOTH print sites; the lists emit in the report's existing deterministic rule-order sequence (`certificate_coverage` iterates `all_fragments` in `rule_order` ⇒ stable). **VERIFIED (rebuilt DEBUG `ast_pipeline --features "ebnf_dual_run generated_parsers"`, count 40 seed 0):** DEFAULT (unset) → `no_path` 10 shown + `UNKNOWN rules (25 of 123 shown)` — caps intact, **byte-identical** to prior; `PGEN_CERT_COVERAGE_DUMP_ALL=1` → all **20** `no_path` + `UNKNOWN rules (123 of 123 shown)`; cert summary IDENTICAL in both (`total=1294 proof=1 witness=1170 UNKNOWN=123 spf=0` — the dump changes the PRINT only, never the computation). `clippy_on_rust_change` ✅ source-clean (generated stage = pre-existing 191-site tolerated debt, unchanged). FULL 20 `no_path` now enumerable = A1 alternate-entry/library subtree (`sv_multi_entry_root`, `systemverilog_parseable_file`, `parseable_source_item`, `library_text`/`library_description`/`library_declaration`, `include_statement`, `kw_include`/`kw_incdir`/`kw_library`/`kw_file_path_spec`) + A2 sv_2023 interface-class (`interface_class_declaration`/`_item`/`_method`, `declared_interface_class_identifier`, `class_constructor_super_args`) + the blessed LRM mutual-recursion `module_path_conditional_expression` + `union_modifier` + `kw_n_29`/`kw_n_48`. **OBSERVABILITY-ONLY ⇒ NO grammar/parser/generated/release/schema/inventory/ledger change; no book/README change (internal debug env var, sibling to the undocumented `PGEN_CERT_COVERAGE_DEBUG_PROBES`). Status: all parser-family rows UNCHANGED — SV stays the only non-fully-certified shipped grammar; this UNBLOCKS the `H.12.5` enumeration.** **Commit: `PGEN-GRAMMAR-WELLFORMED-0079`.** |
| 2 | `GRAMMAR-WELLFORMED.H.12.5.8` (SV infix property/sequence binary-operator parse bug — PRIORITY #2 per the 2026-06-17 director sequencing) | `active` (SPLIT 2026-06-21 by `-0118` WHY+WHERE → children `.8.1` done / `.8.2` fix-design pending) | **Real released-parser bug found this session (bug-finding-oracle):** the cycle-delay sequence form `a ##1 b` is REJECTED (valid IEEE 1800 SV) at `furthest_position=27` — the SAME position with OR without a trailing repetition (`a ##1 b [*2]` fails identically), so the defect is in the `##` cycle-delay / `cycle_delay_range` / sequence-binary lane, NOT the boolean_abbrev `[ ]` fix (`-0105`) and NOT a regression (rejected pre-`-0105` too). Tools-first WHY+WHERE FIRST (`parseability_probe --parse systemverilog`/`--trace-rules` on `module m; sequence s; a ##1 b; endsequence endmodule`; LRM-ground the `##`/cycle_delay_range grammar against `docs/systemverilog/2017` per [[project_ebnf_is_single_source_of_truth]]) before any fix; likely another grammar defect (possibly a dropped/mis-encoded delimiter or operator-precedence issue in `cycle_delay_range`/`sequence_expr`). **BROADENED by `H.12.5.7.1` (`-0116`, 2026-06-21):** the same indirectly-left-recursive infix-branch parse defect also rejects the **property-level** infix operators — `a until b` / `a until_with b` / `a s_until b` / `a s_until_with b` all fail at `furthest=47`, the identical locus as `a or b` / `a and b` / `a ##1 b` (and sequence `a intersect b` / `a within b`). This lane therefore owns the WHOLE infix property/sequence class (`property_expr_sv_2017` / `sequence_expr` left-recursion-elimination), and its fix is what unblocks the M3 infix-operator witnesses that `H.12.5.7` cannot reach in the generator. Tools-first WHY+WHERE FIRST (`parseability_probe --parse systemverilog`/`--trace-rules` on `module m; sequence s; a ##1 b; endsequence endmodule`; LRM-ground the `##`/cycle_delay_range grammar against `docs/systemverilog/2017` per [[project_ebnf_is_single_source_of_truth]]) before any fix. If a grammar edit is warranted: consumer-visible (regen/release/lockstep ceremony, ledger row). Priority #2 — AFTER SV `UNKNOWN`→0, BEFORE PARSE-COMPLETENESS. |
| 2 | `GRAMMAR-WELLFORMED.H.12.5.8.1` (infix binop LR — WHY+WHERE) | `done` (`PGEN-GRAMMAR-WELLFORMED-0118`, PURE-DOCS INVESTIGATION) | **DONE — root cause: `sequence_expr` (direct, self-binary `A := A op A`) and `property_expr_sv_2017`/`property_expr` (indirect via the union wrapper) are NEVER left-recursion-eliminated (generated SV parser has ZERO `_lr_base`/`_lr_suffix`), so the un-eliminated infix branches are blocked by the runtime cycle-breaker (`💥 Infinite recursion detected`, `mutual_recursion_handler.rs:119`) and only the first operand parses ⇒ `a OP b` rejects at the operator** (`parseability_probe --profile 2017`: `a ##1 b`/`and`/`or`/`intersect`/`within`/`until`/`s_until` all FAIL `furthest≈40/42`; bare `a` PASS; `--trace-rules sequence_expr` confirms the cycle-break). ROOT site: `eliminate_left_recursive_patterns`→`detect_left_recursive_chain_plan` (`rust/src/ast_pipeline/mod.rs:1692`) recognizes ONLY the INDIRECT bare-reference wrapper-chain shape (`extract_rule_reference_name`@2127 needs a bare ref; `extract_wrapper_suffix`@2170 needs the wrapper entirely `base suffix`); **direct inline LR `A := Aα\|β` is unhandled by construction** — decisively proven on a minimal `expr := expr plus term \| term` (`0 transformations` at debug verbosity; generated parser rejects `1+2`). Also surfaced: `developer-architecture.md` overclaims direct-LR auto-elimination (book↔code drift, reconcile in `.8.2`). PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change; SV stays `UNKNOWN=56`. Detail: [GRAMMAR-WELLFORMED-H12581-infix-binop-lr-whywhere.md](GRAMMAR-WELLFORMED-H12581-infix-binop-lr-whywhere.md). |
| 2 | `GRAMMAR-WELLFORMED.H.12.5.8.2` (infix binop LR — FIX DESIGN/DECISION) | `done` (`PGEN-GRAMMAR-WELLFORMED-0119`, PURE-DOCS DESIGN) | **DONE — DECISION: direction A** (grammar restructure to a §16 precedence cascade). B is DOMINATED: the cascade is required EITHER way (the current grammar is a flat precedence-FREE operator list, but IEEE 1800-2017 **Table 16-3** gives a real precedence/associativity — `intersect`≻`and`≻`or`, `until`/`iff`/implication right-assoc — that a naive `β (op β)*` would flatten), so B = the same grammar work PLUS a new high-blast-radius engine feature for identical correctness. A = level-1 declarative fix, proven `next (OP next)*` idiom, smallest blast radius (one grammar file + SV regen), correctness-first. Table 16-3 extracted tools-first from the vendored LRM; cascade blueprint (sequence + property layers, both profiles) + `.8.3` implementation sub-questions (AST-shape/schema, Annex A.2.10 operand types, book reconciliation) in the detail file. Recorded as separate future general work (NOT this lane): completing the engine direct-LR eliminator (option B — would also fix the sibling un-eliminated `block_event_expression@:681`). Detail: [GRAMMAR-WELLFORMED-H12582-infix-binop-lr-fix-design.md](GRAMMAR-WELLFORMED-H12582-infix-binop-lr-fix-design.md). **Original A-vs-B framing:** Decide tools-first / research-grounded between **(A)** grammar restructure to the proven non-left-recursive `next (OP next)*` iterative idiom ([[feedback_quantified_group_extraction]] `binary_operator` idiom; grammar-local, must encode IEEE 1800 §16 operator precedence/associativity, likely AST-shape/schema change) vs **(B)** engine fix = genuine **direct** left-recursion elimination in `detect_left_recursive_chain_plan` (`A := Aα\|β ⇒ A := β α*`; general/parser-agnostic, makes the book claim true, fixes ALL grammars, but high blast-radius + must be strictly additive/byte-identical for shipped grammars + self-binary `A := A op A` + precedence care; `-0117` interaction is the cautionary precedent). Decision leaf ONLY; implement = `.8.3`. Reconcile the `developer-architecture.md` direct-LR claim in the same wave as the chosen direction. |
| 2 | `GRAMMAR-WELLFORMED.H.12.5.8.3` (infix binop LR — IMPLEMENT the precedence cascade) | `active` (SPLIT 2026-06-22 by `-0120` → `.8.3.1` SEQUENCE layer IN PROGRESS / `.8.3.2` PROPERTY layer pending) | **The actual fix, direction A.** Implement the §16 precedence-cascade restructure of `sequence_expr` + `property_expr_sv_2017`/`_sv_2023` per the `.8.2` blueprint, split into a verifiable SEQUENCE sub-leaf (`.8.3.1`) then PROPERTY sub-leaf (`.8.3.2`). Re-confirm operand types vs Annex A.2.10; mirror both profiles; keep the `-0105` `boolean_abbrev [ ]` fix; verify `ebnf.ebnf` lockstep; reconcile `developer-architecture.md` direct-LR overclaim in-wave. VERIFY each sub-leaf: `make focus_systemverilog` + rebuild; full infix matrix parses with correct precedence/assoc; SV corpus 14/14; GLOBAL cert UNKNOWN NETS a drop (commit only improvements) seeds 0/7/42 `spf=0`; 6 fully-certified grammars byte-identical; clippy clean; schema/release/ledger ceremony per the final AST-shape outcome. |
| 2 | `GRAMMAR-WELLFORMED.H.12.5.8.3.1` (SEQUENCE-layer cascade) | `parked` — superseded by `STORE-AWARE-GEN.4b.1` (`PGEN-GRAMMAR-WELLFORMED-0121` 2026-06-22: the `@sample "##1 "` band-aid was REJECTED by the director ("no lipstick on a pig") and the cascade grammar WIP was discarded from the working tree — fully recoverable from the committed `-0120` task file; the real SV-`UNKNOWN` fix is store-aware name-coordinated witness generation) | **Cascade IMPLEMENTED + PARSE-VERIFIED, coverage-regression ROOT-CAUSED (not yet committable).** `sequence_expr` rewritten into the §16 cascade (`seq_or→seq_and→seq_intersect→seq_within→seq_throughout→seq_delay(##)→seq_unary`; proven `head tail tail*` + `-> $1` passthrough idiom; exact text in the detail file). PARSE-CORRECT: all infix (`a and/or/intersect/within b`, `a ##1 b`, `a or b and c`) parse, bare/throughout/first_match/##-head preserved, `kw_intersect`/`kw_within` WITNESS, `spf=0` — the released SVA infix sequence bug is fixed at the grammar level. ⚠️ BUT cert REGRESSED `UNKNOWN 56→65`: de-witnesses 9 `seq_unary` operand sub-rules (boolean_abbrev family / sequence_abbrev / kw_first_match / sequence_instance-related). WHY (tool-backed): **NOT depth** (`--max-depth` 24/32/40 all `65`) — a FORCING bug: the plannable pass reaches `seq_unary` via `seq_delay_expr` delay_head (`cycle_delay_range seq_unary`), `cycle_delay_range` eats the operand id, the `seq_unary` expr comes out empty → invalid `## \foo [*]` → no re-parse → no witness. WHERE: forced-descent/reach in `stimuli_generator.rs` (`:3053+`/`:3258+`). Director-decided KEEP CASCADE + FIX COVERAGE. FIX DIRECTIONS (next, tools-first): (1) engine reach-planner prefer clean passthrough to `seq_unary`; (2) grammar — reach `seq_unary` off the `##` delay branches; (3) declarative `@probe_sample` seeds. OUTCOME (`-0121`, 2026-06-22): the band-aid path was REJECTED by the director; the cascade grammar WIP was DISCARDED from the working tree (recoverable from the committed `-0120` task file) and the lane is SUPERSEDED by `STORE-AWARE-GEN.4b.1` (store-aware name-coordinated witness generation, against the clean-regen-verified flat baseline `UNKNOWN=56`). The cascade *restructure* itself stays a valid future infix-SVA parse fix if relanded cleanly without the band-aid. Detail: [GRAMMAR-WELLFORMED-H12583-sva-precedence-cascade-implement.md](GRAMMAR-WELLFORMED-H12583-sva-precedence-cascade-implement.md). |
| — | `GRAMMAR-WELLFORMED.H.12.5.8.3.1.1` (reach-site passthrough preference + LAND the sequence cascade — released-SV ceremony) | `done` (`PGEN-GRAMMAR-WELLFORMED-0132`, GRAMMAR + GENERATOR — released SV `1.0.147→1.0.148`, schema stays `6`, ledger `SV-0010`) | **LANDED — the cascade relanded cleanly (no band-aid) WITH the direction-1 engine fix.** The `-0127` re-measure already proved the stale `56→65` regression does NOT reproduce on today's witness machinery (cascade-alone `28→27`, only `kw_first_match` de-witnessed). This slice adds the parser-agnostic `prefer_sole_reference_passthrough_sites` + `count_mandatory_yield_atoms` to `reach_hops_pass` (applied before `prefer_non_self_recursive_reference_sites` so self-recursion stays the PRIMARY stable-sort key), routing the cascade descent through the clean `-> $1` passthroughs so `kw_first_match` re-witnesses. SV cert **`UNKNOWN 28 → 26`** (`kw_intersect_6c96caaf`/`kw_within_f42ef621` witnessed; `total 1288→1299` +11; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown). NO REGRESSION: 6 fully-certified grammars BYTE-IDENTICAL (engine inert); SV external corpus 14/14; `ast_shape_contract` 18/18; `--lint-grammar` clean (`1415` rules, +11); clippy source-clean. Schema stays `6` (strictly-more-permissive — simple forms keep their carrier via `-> $1`, only previously-REJECTED infix forms gain new shapes; `SV-0008` category). Acceptance checklist in the detail file. **Frontier → `.8.3.2`** (PROPERTY-layer cascade — same shape, reuses this engine fix to witness the `until`-family, 4 of the residual 26 UNKNOWN). |
| — | `GRAMMAR-WELLFORMED.H.12.5.8.3.2` (PROPERTY-layer precedence cascade — released-SV ceremony) | `done` (`PGEN-GRAMMAR-WELLFORMED-0133`, GRAMMAR-ONLY — released SV `1.0.148→1.0.149`, schema stays `6`, ledger `SV-0011`) | **LANDED — the property-layer dual of `.8.3.1.1`, completing the `H.12.5.8` SVA infix left-recursion class.** The flat directly-left-recursive `property_expr_sv_2017`/`property_expr_sv_2023` are restructured into the IEEE 1800-2017 §16 (Table 16-3) binary-infix precedence cascade — `property_expr_sv_<profile> := prop_until_sv_<profile> -> $1` over 5 new `prop_*` rules/profile (`prop_until > prop_iff > prop_or > prop_and > prop_primary`, all right-assoc inline `head op self` so NO tail rules; `prop_primary` keeps every non-LR branch byte-identical, bodies still referencing the `property_expr` dispatch → schema-preserving). REUSES the `.8.3.1.1` `prefer_sole_reference_passthrough_sites` engine fix (NO engine change). SV cert **`UNKNOWN 26 → 22`** (the 4 property-only `until`-family keywords `kw_until`/`kw_s_until`/`kw_until_with`/`kw_s_until_with` witnessed; `total 1299→1304` (+5), `witness 1272→1281`; **deterministic seeds 0/7/42**; `spf=0`; zero newly-unknown — set-diff = exactly the 4 `until`-family). Parse matrix REJECT→PASS with correct precedence (`a until b iff c` ⇒ `until(a,iff(b,c))`; `a and b or c` ⇒ `or(and(a,b),c)`); previously-parsing carriers byte-identical (`not not a` unchanged). NO REGRESSION: 6 fully-certified grammars byte-identical (grammar-only, SV-isolated); SV external corpus 14/14; `ast_shape_contract` 18/18; `--lint-grammar` clean (`1425` rules, +10); clippy source-clean. Schema stays `6` (strictly-more-permissive — the newly-parsing infix forms reuse the existing `{kind, lhs, rhs}` carriers; `SV-0008`/`SV-0010` category). DOCUMENTED LIMITATION: tight prefix ops `not`/`nexttime`/`s_nexttime` keep pre-fix dispatch-body precedence (`not a until b` ⇒ `not (a until b)`), unchanged + schema-preserving. Acceptance checklist in the detail file `GRAMMAR-WELLFORMED-H12832-property-cascade-implement.md`. **Frontier → the SV `UNKNOWN`=22 residual** (`no_path`/multi-entry-root + store-gate cohorts — the SVA-infix class is now CLOSED). |
| 2 | `GRAMMAR-WELLFORMED.B2/C1` | `pending` | The remaining CONSTRUCTIVE-side lanes (stimuli generator): bounded-ordered backtracking, defeat-earlier-branch crafting. Riskier (touch generator runtime; measure the global metric). Feeds G.3 (the witness producer). |
| 1 | `GRAMMAR-WELLFORMED.H.12.6` (`no_path` LRM-grounded re-audit + no-deletion policy) | `done` (`-0108`, PURE-DOCS) | Director directive 2026-06-17: NO rule deleted unless the LRM objectively proves it absent; `no_path` ⇒ fix the producer first. LRM-grounded re-audit of all 20 SV `no_path` rules → **19/20 LRM-legitimate (STAY)** (10 rooted under the `library_text` start symbol per IEEE 1800-2017 §33/A.1.1; 6 profile-relative SV-2023; 3 decomposition artifacts) + **1 genuine producer-wiring suspect** (`module_path_conditional_expression`) → `H.12.6.1`; **0 deletions**. Policy persisted as [[feedback_no_rule_deletion_without_lrm_proof]]; tracker [GRAMMAR-WELLFORMED-H126-no-path-lrm-reaudit.md](GRAMMAR-WELLFORMED-H126-no-path-lrm-reaudit.md). |
| 1 | `GRAMMAR-WELLFORMED.H.12.6.1` (`module_path_conditional_expression` producer-wiring WHY+WHERE + fix) | `done` (`PGEN-GRAMMAR-WELLFORMED-0111`, GRAMMAR FIX, release 1.0.143, schema stays 4, ledger `SV-0005`) | **DONE — SV cert `UNKNOWN 86→84` AND the LAST `no_path` budget case RETIRED (`unreachable_rules=0`, the linter's "no unreachable rules" now LITERALLY true for SV).** The ONE genuine `no_path` defect: mpce (LRM Annex A.8.3) was stranded by an indirect LEFT-RECURSION — its condition referenced `module_path_expression`, whose first branch was mpce; the LR-eliminator (`--dump-gen-ast` + reference-graph + call-site grep) rewrote the producer into `module_path_expression_lr_base`/`_lr_suffix` and left mpce as an UNREFERENCED rewritten seed (`parse_module_path_conditional_expression` defined-but-never-called), and the `_lr_suffix` reconstruction LEAKED raw `wrapper_specs` metadata into the ternary module-path AST (`if (a?b:c)` → 0 conditional / 3 wrapper_specs). The "RecursionGuard handles it / budget case" comment was DISPROVEN. FIX (surgical, grammar-only, LRM-faithful — changed ONLY mpce): mpce's condition is now the non-left-recursive `module_path_expression_operand` chain (same A.8.3 language — right-associative ternary), so `module_path_expression` is natively non-left-recursive (eliminator no longer fires; `_lr_base`/`_lr_suffix` vanish), mpce is positively referenced + WITNESSED, the leak is gone; the "budget case" comment retired same-edit. VERIFIED (deterministic seeds 0/7/42): cert `UNKNOWN 86→84` (witness `1204`; total `1291→1289`; spf=0); `no_path 20→19` (exactly mpce; 19 LRM-legitimate stay); genuineness oracle 1 conditional / 0 wrapper_specs; SV external corpus 14/14; `stimuli_cross_family_platform_gate` PASS; syntax-closure v6 (`max_unreachable_rules 1→0`, blessed emptied); clippy source-clean. Detail: `docs/tasks/GRAMMAR-WELLFORMED-H1261-module-path-conditional-fix.md`. Part of lane (1) SV `UNKNOWN`→0. |
| 1 | `GRAMMAR-WELLFORMED.H.12.7` (post-SVA-cascade SV `UNKNOWN=22` full residual re-adjudication + top-level book lockstep) | `done` (`PGEN-GRAMMAR-WELLFORMED-0134`, PURE-DOCS) | **Tools-first re-adjudication of the SV cert residual at its post-cascade count (the `H.12.6` mechanism applied at `UNKNOWN=22`) + closing 6 slices of top-level book drive-arc drift.** The SVA-infix LR class is CLOSED (`-0132`/`-0133`); reproduced the canonical baseline `total=1304 proof=1 witness=1281 UNKNOWN=22 spf=0` (deterministic seeds 0/7/42, `PGEN_CERT_COVERAGE_DUMP_ALL=1`). TWO narrowing runs prove the categories: `--entry-rule sv_multi_entry_root` collapses `no_path` 19→8 (the 11 `library_text`/parseable-entry rules gain reach); `--grammar-profile sv_2023` drops the 6 profile-relative rules (`UNKNOWN 22→17`, set-diff = exactly those 6). **VERDICT: 22 = 11 entry-relative + 6 profile-relative + 2 blessed decomposition (`kw_n_29`/`kw_n_48`, referenced-but-unreachable) NON-defects + 3 deferred honest-limit reach-gaps** (`context_member_method_call` store-gated declaration-hosting carrier; the two `…scoped_call…` cousins = `T::method()` ambiguity above the rule). 0 deletions per [[feedback_no_rule_deletion_without_lrm_proof]]. SURFACED (not decided — director call): SV `fully_certified` needs a multi-entry/multi-profile cert accounting (the 17 entry/profile rules witness elsewhere) leaving the 3 deferred reach-gaps as the only canonical-entry residual. BOOK LOCKSTEP: `grammar-wellformedness.md` new `### Closing the SVA operator layer` subsection narrates the `32 → 22` arc (covergroup/SVA fidelity ×3 + count-prelude + SVA sequence/property cascades) + the consolidated 22-residual adjudication. PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; all deterministic gates inherit `-0133` green byte-identical; `mdbook_docs_gate` re-run GREEN. Detail: [GRAMMAR-WELLFORMED-H127-post-cascade-residual-adjudication.md](GRAMMAR-WELLFORMED-H127-post-cascade-residual-adjudication.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0134`.** |
| 1 | `GRAMMAR-WELLFORMED.H.12.8` (drive SV to `fully_certified` `UNKNOWN=0`) | `active` — `.8.0` ✅ `-0136` (TOOL-BACKED re-adjudication of the union premise, PURE-DOCS); **`.8.1` DESIGN ✅ `-0137`** (opt-in `--cert-union-config` design, PURE-DOCS); **`.8.1.1` IMPLEMENT ✅ `-0138`** (opt-in `--cert-union-config` shipped; SV union `22 → 14` deterministic seeds 0/7/42 — tool-corrected from the design's `16`: `kw_n_29`/`kw_n_48` are PRESENT+WITNESSED under `sv_2023`, so `.8.2` is SUBSUMED by the union); `.8.3` NOT STARTED; `.8.4` IN PROGRESS (`.8.4.1` ✅ `-0139` file_path_spec WHY+WHERE/design; `.8.4.2` ✅ `-0140` — that prediction REFUTED: file_path_spec is cert-NEUTRAL; TRUE blocker = hardwired cert verification entry `parse_and_cover_systemverilog`→`parse_full_systemverilog_file` makes all 11 un-witnessable; real fix `.8.4.3` = entry-aware verification infra); **DIRECTOR-COMMITTED 2026-06-25** ([[project_sv_full_certification_via_multi_entry]]) | **THE committed SV endgame: `UNKNOWN=0` IS achievable and WILL be achieved.** ⚠️ **`.8.0` TOOL-BACKED CORRECTION (`-0136`, 2026-06-29):** a tools-first re-derivation before any code (canonical + alt-config `--report-certificate-coverage` seeds 0/7/42 + `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`) overturned the `-0135` premise — the multi-entry/multi-profile union delivers SV **`22 → 16`, NOT `22 → 5`**. `sv_2023` witnesses the **6 profile-relative** rules (`UNKNOWN 22→17`, set-diff = those 6); but `sv_multi_entry_root` leaves `UNKNOWN=22` UNCHANGED (collapses `no_path` 19→8 = the 11 entry-relative rules GAIN REACH, but witness NONE — probes show trivial `""`/`";"` routing + malformed `file_path_spec` literal-name samples). So the union (`.8.1`) certifies only 6 of 22; the 11 entry-relative are genuine generation gaps (NEW `.8.4`). Each sub-leaf: tools-first WHY+WHERE before code, decisive A/B GLOBAL cert + spf seeds 0/7/42, 6 fully-certified grammars byte-identical, SV corpus 14/14, parser-agnostic + inert for the certified roster. **`.8.1`** — multi-entry/multi-profile cert accounting (certify a rule if witnessed-or-proven in ANY supported `(entry, profile)` config — at least `(systemverilog_file, sv_2017)`/`(…, sv_2023)`; UNKNOWN only if UNKNOWN in ALL) → certifies the **6 profile-relative** rules (CORRECTED from "17"); true yield **`22 → 16`**. **DESIGN ✅ `-0137` (`.8.1` DESIGN done):** decided the open Q — **explicit opt-in `--cert-union-config <entry>[:<profile>]` (repeatable), NOT auto-union** (auto ≈3× runtime + un-parser-agnostic). Refactor = extract the per-config covered-set passes (`main.rs:2385-2754`) into `gather_cert_covered_sets(... emit_diagnostics) -> CertCoveredSets`; canonical call verbose (byte-identical), union calls quiet; classify canonical `rule_order` against `⋃(proof∪witness)`; derive `Clone` on `LoadedGrammar` to re-filter per profile. **TOOL-PROVEN SOUNDNESS RULE (this session):** union over POSITIVELY-covered sets, NEVER over "not-UNKNOWN-in-some-config" — a naïve set-diff over-claims 8 because `kw_n_29`/`kw_n_48` are profile-FILTERED-OUT of `sv_2023` (absent, not witnessed); the sound covered-set union certifies only the 6 genuinely-witnessed profile rules → **`22 → 16`** confirmed. NEXT = `.8.1.1` IMPLEMENT. Full design in [project_sv_full_certification_via_multi_entry.md](../decisions/project_sv_full_certification_via_multi_entry.md) `## ✅ .8.1 IMPLEMENTATION DESIGN`. (Was: open Q explicit-mode-vs-auto; inert + byte-identical for the 6 fully-certified grammars either way.) **`.8.4` (NEW)** — the **11 entry-relative `library_text`/parseable-fragment generation gaps** (`sv_multi_entry_root`, `systemverilog_parseable_file`, `parseable_source_item`, `library_text`/`_declaration`/`_description`, `include_statement`, `kw_include`/`kw_incdir`/`kw_library`/`kw_file_path_spec`): real reach/generation work (fix trivial-alternative routing + `file_path_spec` literal-name expansion), tool-proven they do NOT witness even from the multi-entry root. **`.8.2`** — the **2 LRM-extraction artifacts** `kw_n_29`(`/29\b/`)/`kw_n_48`(`/48\b/`): spurious literal clause-numbers leaked into productions (`systemverilog.ebnf:1492` covergroup-extends; `:2869`/`:3926`/`:4328` `local::48`); LRM-ground + correct at source (no-deletion policy [[feedback_no_rule_deletion_without_lrm_proof]]) OR certify as a verified unreachability PROOF; released-SV ceremony if accept-changing. **`.8.3`** — close the **3 canonical reach-gaps**: `context_member_method_call` via the STORE-AWARE-GEN declaration-hosting carrier (`H.12.5.5.3.3.4.2.1.2.2.3` design, on `STORE-AWARE-GEN.4b` value-selection); the two `…scoped_call…` cousins via the grammar tightening excluding type-parameter/interface-class heads from the generic scoped-call alternative. Detail: [project_sv_full_certification_via_multi_entry.md](../decisions/project_sv_full_certification_via_multi_entry.md). |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.1.1` (IMPLEMENT the opt-in `--cert-union-config` multi-config cert union) | `active` (`PGEN-GRAMMAR-WELLFORMED-0138`, **CODE**) | **The IMPLEMENT slice of the `.8.1` DESIGN (`-0137`).** Adds a repeatable `--cert-union-config <entry>[:<profile>]` flag to `ast_pipeline --report-certificate-coverage`; extracts the per-config covered-set passes from `run_certificate_coverage_report` into `gather_cert_covered_sets(... emit_diagnostics) -> CertCoveredSets`; derives `Clone` on `LoadedGrammar` to re-filter the unfiltered bundle per union config; unions the POSITIVELY-covered (`proof ∪ witness`) sets (the soundness rule — NEVER "not-UNKNOWN-in-some-config"); prints the byte-identical canonical `CERTIFICATE-COVERAGE:` line plus a `CERTIFICATE-COVERAGE-UNION:` line when configs are present. SV invocation (`--cert-union-config systemverilog_file:sv_2023 --cert-union-config sv_multi_entry_root:sv_2017`) ⇒ **`UNKNOWN 22 → 14`** (deterministic seeds 0/7/42; certifies the 6 profile-relative rules **+ the 2 extraction leaves `kw_n_29`/`kw_n_48`** — all 8 witness under `sv_2023`). **TOOL-CORRECTION to the `-0137` design (`22 → 16`):** the design's premise that `kw_n_29`/`kw_n_48` are profile-filtered OUT of `sv_2023` was disproven by `--dump-gen-ast --grammar-profile sv_2023` (`kw_n_29`×3, `kw_n_48`×5 present) + `sv_2023` cert (UNKNOWN=17, neither in it) ⇒ they genuinely witness there, so the sound union legitimately certifies them. Residual 14 = 11 entry-relative (`.8.4`) + 3 reach-gaps (`.8.3`); `.8.2` (`kw_n_*`) is SUBSUMED by the union. Default-empty ⇒ byte-identical single-config behavior for EVERY grammar. Acceptance checklist in Decisions. |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.4.1` (entry-relative cohort WHY+WHERE + LRM-grounded fix design) | `done` (`PGEN-GRAMMAR-WELLFORMED-0139`, PURE-DOCS) | **DONE — tools-first root-cause map of the 11 entry-relative union-residual rules under `--entry-rule sv_multi_entry_root`.** Mechanism A (6 `parsed=false`): the generator emits the literal text `file_path_spec` because `systemverilog.ebnf:5736` `kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/` is an LRM-extraction artifact (the LRM nonterminal — a file path, IEEE 1800-2017 §33.3.1 / Annex A.1.1 — flattened to a literal keyword); transitively blocks `library_text`/`library_description` (2 more) ⇒ dominates 8 of 11. Mechanism B (3): multi-entry scaffolding routing (`sv_multi_entry_root`/`systemverilog_parseable_file`/`parseable_source_item` pick a trivial/empty alt). Fix design (`.8.4.2`): replace the `:5736` lexeme with the LRM-faithful path char-class `/[A-Za-z0-9_.\/?*~$+-]+/`, rule name/arity unchanged (AST-shape-neutral). Detail: [GRAMMAR-WELLFORMED-H1284-entry-relative-filepathspec-whywhere-design.md](GRAMMAR-WELLFORMED-H1284-entry-relative-filepathspec-whywhere-design.md). |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.4.2` (`file_path_spec` implement attempt — REFUTED + TRUE root cause) | `done` (`PGEN-GRAMMAR-WELLFORMED-0140`, PURE-DOCS — implement attempted, MEASURED cert-neutral, REVERTED) | **The `.8.4.1` prediction is REFUTED.** Applying the `file_path_spec` lexeme fix + regen + rebuild measured the union `UNKNOWN=14` **unchanged** (the edit took effect — samples became real paths — but moved nothing). TRUE root cause (tool-proven): cert witness VERIFICATION is hardwired — `parse_and_cover_systemverilog` (`parser_registry.rs:571`) always parses from `parse_full_systemverilog_file`, and the generated parser exposes no `pub fn parse_full_*` for the LRM alternate start symbols ⇒ the 11 entry-relative rules are un-witnessable regardless of `--entry-rule`/`--cert-union-config` (proven: `--entry-rule include_statement` ⇒ `sample_parse_failures=8/8`; `library_description` already complete `:2594-2597`). REAL fix (parser-agnostic infra, `.8.4.3`): codegen `pub fn parse_full_<entry>` for alternate entries + `parse_and_cover` entry dispatch. Edit REVERTED (commit only improvements). Detail: [GRAMMAR-WELLFORMED-H1284-entry-relative-cert-verification-entry-rootcause.md](GRAMMAR-WELLFORMED-H1284-entry-relative-cert-verification-entry-rootcause.md). |
| 1 | `GRAMMAR-WELLFORMED.H.12.8.4.3` (entry-aware cert witness verification — the REAL entry-relative enabler) | `done` (`PGEN-GRAMMAR-WELLFORMED-0141`, **CODE**) | **LANDED the `-0140` real fix.** Codegen (`ast_based_generator.rs`): emit `pub fn parse_from`/`parse_full_from(entry)` — a per-rule dispatch so a full parse can start from ANY rule (default arm = canonical, so single-entry grammars are byte-identical). Registry (`parser_registry.rs`): `ParseAndCoverFn` + the 7 closures + generic `parse_and_cover` take an `entry: Option<&str>` and verify via `parse_full_from(entry)`. main.rs: thread the per-config entry into the 6 cert `parse_and_cover` sites. Parser-agnostic, inert for single-entry grammars by construction. **DECISIVE A/B (seed 0): union with `library_text:sv_2017` + `systemverilog_parseable_file:sv_2017` configs ⇒ `UNKNOWN 14 → 3`** (all 11 entry-relative rules witness — better than the `≤6` prediction, because verification from the matching entry confirms the `file_path_spec`-literal samples that `systemverilog_file` could never parse). Canonical byte-identical `UNKNOWN=22 spf=0`. Residual 3 = the `.8.3` canonical reach-gaps. `parseability_probe --entry-rule` split to `.8.4.3.1`. `.8.4.4` (`file_path_spec` LRM-fidelity) is now a cleanup, not a witnessing blocker. Detail: [GRAMMAR-WELLFORMED-H1284-entry-aware-cert-witness-verification.md](GRAMMAR-WELLFORMED-H1284-entry-aware-cert-witness-verification.md). |
| 2 | `GRAMMAR-WELLFORMED.H.12.8.4.3.1` (`parseability_probe --entry-rule` tool-build) | `done` (`PGEN-GRAMMAR-WELLFORMED-0142`, **CODE**) | **LANDED.** `parseability_probe --parse … --entry-rule <RULE>` now parses from an alternate start symbol via the `.8.4.3` `parse_full_from` dispatch. Registry: extracted `parse_with_systemverilog_detail_profile_entry(…, entry)` (registered `ParseDetailFn` delegates with `None` ⇒ byte-identical, keeps `furthest_position`) + parser-agnostic `parse_sample_detail_from_entry` (each grammar via `parse_full_from`; regex on its worker stack). Probe: `--entry-rule` (`GlobalOptions` field + arg parse + `command_parse` entry path + usage). **DECISIVE A/B** (freshly-built release probe): `include file_path_spec;` rejects from the default entry (`furthest_position=7`) but `parse_full passed` with `--entry-rule library_text`; control `module m; endmodule` passes from the default entry. NO REGRESSION: `None` path unchanged; cert/codegen/grammars untouched ⇒ cert byte-identical to `-0141`; `cargo test --lib` 771/0; clippy source-clean. NO grammar/release/schema change. Detail: [GRAMMAR-WELLFORMED-H12843-1-parseability-probe-entry-rule.md](GRAMMAR-WELLFORMED-H12843-1-parseability-probe-entry-rule.md). |
| 3 | `GRAMMAR-WELLFORMED.H.12.8.4.4` (`file_path_spec` LRM-fidelity fix) | `done` (`PGEN-GRAMMAR-WELLFORMED-0143`, **CODE / released-SV**, release `1.0.150`, schema `6`, ledger `SV-0012`) | **LANDED the `.8.4` library-cohort LRM-fidelity fix `-0141` retained after `-0140` (cert-neutral).** `systemverilog.ebnf:5736` `kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/` (literal-keyword extraction artifact — `include`/`library` REJECTED real §33 paths) → `/[A-Za-z0-9_.\/?*~$+]+/` (LRM-faithful path lexeme per IEEE 1800-2017 §33.3.1/Annex A.1.1; name/arity preserved ⇒ AST-shape/schema byte-identical; `-` excluded so the glued `kw_incdir` witness boundary survives — tool-grounded refinement of the `-0139` design). **DECISIVE A/B** (release probe, `--entry-rule library_text`): `include ../rtl/cpu.v;`/`library mylib /path/to/*.sv;`/`library rtl ./src/*.sv, ./pkg/*.sv -incdir ./inc;` REJECT→PASS, literal `include file_path_spec;` still PASS. NO REGRESSION: SV canonical `UNKNOWN=22 spf=0` + complete 4-config union (incl. `sv_multi_entry_root:sv_2017`) `witness=1300 UNKNOWN=3` byte-identical seeds 0/7/42 (residual = exactly the 3 `.8.3` reach-gaps; `kw_incdir` witnesses); regex 198/198 + vhdl 216/216 `fully_certified=true`; `ast_shape_contract` 18/18; SV external corpus 14/14; `--lint-grammar` clean (`1425` rules); clippy source-clean; `cargo test --lib` 771/0/21. (3-config union omitting `sv_multi_entry_root:sv_2017` shows `UNKNOWN=4` — structural: that no_path parent only witnesses from its own config; refuted comment-collision `//`/`/*` hypothesis tools-first.) The `.8.4` sub-lane is CLOSED; frontier returns to `.8.3`. Detail: [GRAMMAR-WELLFORMED-H12844-filepathspec-lrm-fidelity.md](GRAMMAR-WELLFORMED-H12844-filepathspec-lrm-fidelity.md). |
| 1 | `GRAMMAR-WELLFORMED.H.13` (EBNF meta-grammar `ebnf.ebnf` 6-gap catch-up — the ticketed lockstep leaf from the `-0068` audit) | `done` — gate strict-GREEN (split per-construct; `.1` ✅ `-0122`; `.2` ✅ `-0123`; `.3` ✅ `-0124`; `.4` ✅ `-0125`; `.5` ✅ `-0126`) | The 6 EBNF-format constructs that live in shipped grammars but are ABSENT from `grammars/ebnf.ebnf`, so the generated ebnf parser cannot self-parse them and `ebnf_frontend_dual_run_gate` stays red ([[feedback_ebnf_meta_grammar_lockstep]]): (1) per-branch return annotations [dominant], (2) `::N*` extraction-spread, (3) **`**` flatten-spread** (`.1`), (4) `[>`/`[>!` lexical annotations, (5) dotted `$refs`, (6) indexed `$refs`. One sub-leaf per construct, each proven FAIL→PASS by a minimal `ebnf_dual_run_diff` probe + no regression on the passing dual-run grammars (`ebnf`/`json`) and the production parsers. **NOTE (post-`.2`):** the `-0068` whole-file probes were first-failure-masked, so closing earlier gaps surfaces additional masked constructs beyond the original 6-enumeration (e.g. `.3` = object-literal-as-array-element) — each is tracked as a sub-leaf as it appears; the true done-criterion is the `ebnf_frontend_dual_run_gate` going green, not a fixed count. **UPDATE (`.5`, `-0126`): the gate is now strict-GREEN — `ebnf`/`json`/`regex` all self-parse under the generated ebnf parser (`regex` the feature-richest probe, now 100% / `parse_end 78429/78429`). HONEST SCOPE (never over-claim, [[feedback_always_signoff_decisions]]): the gate tracks ONLY those 3 grammars; a fresh all-grammars `ebnf_dual_run_diff` scan shows the generated ebnf parser self-parses 8/12 grammars — `systemverilog` / `vhdl` / `systemverilog_preprocessor` / `rtl_frontend` still FAIL ⇒ FULL self-hosting ("every grammar self-parses") is NOT yet met and continues under new leaf `H.14`. So `H.13` is `done` re: its gate-defined criterion, NOT re: universal self-hosting.** |
| 1 | `GRAMMAR-WELLFORMED.H.13.1` (port the `**` flatten-spread return-marker to `ebnf.ebnf`) | `done` (`PGEN-GRAMMAR-WELLFORMED-0122`, GRAMMAR — `grammars/ebnf.ebnf` only; `generated/ebnf.rs` regenerated, untracked) | See the 2026-06-24 Decisions entry + its acceptance checklist. The regex `concatenation := piece+ -> [$1**]` (the original `-0067` / `H.11.5(3)` red-gate trigger) now parses under the generated ebnf parser; regex dual-run advances past byte 1307 to its next gap (dotted `$refs`). Meta-grammar internal: NO production-parser regen, NO release/schema/ledger change. |
| 1 | `GRAMMAR-WELLFORMED.H.13.2` (port per-branch return annotations to `ebnf.ebnf`) | `done` (`PGEN-GRAMMAR-WELLFORMED-0123`, GRAMMAR — `grammars/ebnf.ebnf` only; `generated/ebnf.rs` regenerated, untracked) | See the 2026-06-24 Decisions entry + its acceptance checklist. `alternation := sequence (return_annotation? "\|" sequence)*` — the **dominant** `-0068` gap: a `->` before a `\|` (e.g. regex's `piece = piece_quoted_run_quantified -> $1 \| atom quantifier? -> {…}`) now parses under the generated ebnf parser; regex dual-run advances from byte `1726`. Additive + provably inert for `ebnf`/`json` (neither uses per-branch annotations — they pass today while the feature is unsupported); the trailing/last-branch annotation stays with `rule_definition`. Meta-grammar internal: NO production-parser regen, NO release/schema/ledger change. |
| 1 | `GRAMMAR-WELLFORMED.H.13.3` (allow an object literal as an array element in `ebnf.ebnf`'s `array_element_return`) | `done` (`PGEN-GRAMMAR-WELLFORMED-0124`, GRAMMAR — `grammars/ebnf.ebnf` only; `generated/ebnf.rs` regenerated, untracked) | See the 2026-06-24 Decisions entry + its acceptance checklist. `array_element_return` gained `object_return` in its ordered choice, so regex's `piece_quoted_run_quantified -> [$2**, {type:"piece", atom:$3, quantifier:$5}]` (an object element after a `**` flatten) now parses; regex dual-run advances from byte `2413`. A gap BEYOND the original `-0068` 6-enumeration, surfaced by advancing past per-branch (`.2`). Additive + provably inert for `ebnf`/`json` (no array carries an object element today). Meta-grammar internal: NO production-parser regen, NO release/schema/ledger change. |
| 1 | `GRAMMAR-WELLFORMED.H.13.4` (port dotted property access on a reference — `$1.min` — to `ebnf.ebnf`) | `done` (`PGEN-GRAMMAR-WELLFORMED-0125`, GRAMMAR — `grammars/ebnf.ebnf` only; `generated/ebnf.rs` regenerated, untracked) | See the 2026-06-24 Decisions entry + its acceptance checklist. `scalar_return` gained a `reference_property_access := reference_base property_access_suffix property_access_suffix*` alternative (tried FIRST), so regex's `quantifier := quant_base quant_suffix? -> {type:"quantifier", min: $1.min, max: $1.max, greediness: $2}` (dotted `$1.min`/`$1.max`; also the function-call args `generate_range_check($1.start, $1.end)`) now parses under the generated ebnf parser; regex dual-run advances from byte `3960` → `4533`. The original `-0068` gap #5 (dotted `$refs`); the companion gap #6 (indexed `$N[i]`) stays a future leaf — no dual-run grammar uses it. Additive + provably inert for `ebnf`/`json` (a bare `$N` falls through to `positional_reference`). Meta-grammar internal: NO production-parser regen, NO release/schema/ledger change. |
| 1 | `GRAMMAR-WELLFORMED.H.13.5` (port the `null` object-value literal to `ebnf.ebnf`) | `done` (`PGEN-GRAMMAR-WELLFORMED-0126`, GRAMMAR — `grammars/ebnf.ebnf` only; `generated/ebnf.rs` regenerated, untracked) | See the 2026-06-24 Decisions entry + its acceptance checklist. `literal_return` gained `null_literal := "null" -> {type:"null"}`, so regex's `quant_base = "*" -> {min: 0, max: null}` (`regex.ebnf:84`) now parses — the LAST `rust_parse_full` gap for `regex.ebnf`, which now self-parses to 100% (`parse_end 78429/78429`). **This flips `ebnf_frontend_dual_run_gate` strict-GREEN** (ebnf 127/127, json 99.90%, regex 100% — all `overall=pass`, `make -C rust ebnf_frontend_dual_run_gate` exit 0). Original `-0068` gap (JSON's 6th value type). Additive + non-shadowing (`null` shares no prefix with the other `literal_return` alternatives). Meta-grammar internal: NO production-parser regen, NO release/schema/ledger change. |
| 2 | `GRAMMAR-WELLFORMED.H.14` (extend EBNF self-hosting beyond the dual-run gate's 3 tracked grammars — drain the remaining 4) | `active` — `.1` ✅ `-0128` (`::N*` extraction-spread; self-host **8/12 → 11/12** — `vhdl`/`rtl_frontend`/`svpp` 100%); `.2` ✅ `-0129` (root-caused the LAST gap = SV duplicate-`->` defect; deleting it yields **12/12** but is shipped-SV-parser territory ⇒ deferred); `.3` ✅ `-0130` (**12/12 ACHIEVED** — SV self-host 72476→100%; UDP-entry raw→typed RESTORED, schema 5→6 / release 1.0.147 / ledger SV-0009; cert 0/7/42 `UNKNOWN=28 spf=0`, shape 18/18, corpus 14/14); `.3.1` ✅ `-0131` = UDP shape-lock added to the SV `ast_shape_contract` manifest (non-root `rule_under_test` gate-mechanism extension + 2 UDP-entry samples; negative-control-proven the lock bites) | HONEST follow-up from `H.13.5`'s all-grammars scan: the generated ebnf parser self-parses 8/12 grammars but `systemverilog` / `vhdl` / `systemverilog_preprocessor` / `rtl_frontend` still FAIL `rust_parse_full` under `ebnf_dual_run_diff`. Per [[feedback_ebnf_meta_grammar_lockstep]] every EBNF-format construct a shipped grammar uses MUST be modelled in `ebnf.ebnf`. Drain gap-by-gap (tools-first `ebnf_dual_run_diff --input grammars/<g>.ebnf` → inspect the `parse_end` byte → port the missing construct), and consider WIDENING the dual-run gate's tracked set to all shipped grammars so the lockstep cannot silently regress. SECONDARY to the SV `UNKNOWN`→0 headline lane; director-prioritisable. |
| 2 | `GRAMMAR-WELLFORMED.H.14.1` (port the `::N*` extraction-spread construct family to `ebnf.ebnf`) | `done` (`PGEN-GRAMMAR-WELLFORMED-0128`) — self-host **8/12 → 11/12**; `vhdl`/`rtl_frontend`/`svpp` now self-parse 100%; dual-run gate strict-GREEN; SV `UNKNOWN=28` unchanged | See the 2026-06-25 Decisions entry + acceptance checklist. The `$N::target spread?` extraction-spread (`$3::2*`, `$2::first`, `$2::2`, …) — listed in the `H.13` row as original `-0068` gap #2 but NEVER closed in `H.13` (the gate's 3 tracked grammars ebnf/json/regex don't use it; only vhdl/sv/svpp/rtl_frontend do). Tool-proven gap (`ebnf_dual_run_diff` minimal probes): `[$1::2*]` FAIL@9 but `[$2**]` / `[$1, $3]` PASS ⇒ absent from `ebnf.ebnf`'s `array_element_return`/`scalar_return` (`quantified_marker` models only `* + ?`). 138 uses across the 4 self-parse-failing grammars (vhdl 17, rtl_frontend 18, svpp 2, systemverilog 119+). Authoritative model = `grammars/return_annotation.ebnf:43` `extraction_expression := positional_reference '::' extraction_target spread_suffix?`. The earliest dual-run gap in vhdl (@1666) and rtl_frontend (@5018). |
| — | `GRAMMAR-WELLFORMED.H.14.2` (root-cause the LAST `systemverilog` self-parse gap; fix DEFERRED to fresh-budget SV) | `done` (`PGEN-GRAMMAR-WELLFORMED-0129`, PURE-DOCS INVESTIGATION) — root-caused @72476 = a duplicate-return-annotation DEFECT (`grammars/systemverilog.ebnf:1168` + `:4639`); MEASURED that deleting them yields **12/12** self-host (369412/369413) but proved the fix is a shipped-SV-parser change that byte-identity CANNOT verify (SV codegen byte-non-deterministic: `cbe76f0e`/`8cf1515b`/`11ebfda1` across regens) ⇒ needs the full released-SV cert/corpus/shape ceremony ⇒ DEFERRED + REVERTED. See the 2026-06-25 Decisions entry. Recovery: delete the 2 lines, run the SV ceremony. |
| 2 | `GRAMMAR-WELLFORMED.H.14.3` (LAND the 12/12 self-host fix: delete the 2 duplicate `->` + released-SV ceremony) | `done` (`PGEN-GRAMMAR-WELLFORMED-0130`, GRAMMAR — flagship SV, released-SV ceremony) — **EBNF self-hosting 12/12**; UDP-entry AST shape RESTORED raw→typed (schema 5→6, release 1.0.147, ledger SV-0009) | DONE: deleted `grammars/systemverilog.ebnf:1168` + `:4639` (the 2nd `->` at each UDP-table site) → `systemverilog` self-parses 72476→100% ⇒ **EBNF self-hosting 12/12** (tracked set; the 3 raw LRM-extraction snapshots are out of scope). Adjudicated the UDP-entry shape change tools-first: the duplicate `->` had made the return annotation FAIL TO PARSE → raw `Sequence` since SV-Slice-66 (`b48f66de`, `1.0.66`); the fix RESTORES the typed `{inputs, output}` / `{inputs, current_state, next_state}` documented since SV-Slice-18 + in the book json-carrier. Previously-valid UDP carrier change ⇒ schema 5→6 (covergroup `SV-0006`/`SV-0007` criterion). VERIFIED: cert seeds 0/7/42 `UNKNOWN=28 spf=0`; `ast_shape_contract` 18/18; external corpus 14/14; clippy source-clean; surgical 97-line generated-parser diff confined to the 2 rules; BEFORE raw `Sequence` → AFTER `{"inputs":[…],"output":…}` empirical. See the 2026-06-25 Decisions entry + acceptance checklist. |
| — | `GRAMMAR-WELLFORMED.H.14.3.1` (lock the UDP truth-table entry shapes in the SV `ast_shape_contract` manifest so the raw↔typed regression class cannot silently recur) | `done` (`PGEN-GRAMMAR-WELLFORMED-0131`, HARDENING — test-harness + shape-contract manifest only; NO grammar/codegen/generated/release/schema/ledger change) | DONE: extended the shape-contract gate to support a **non-root `rule_under_test`** — `run_manifest`'s callback now receives the sample's rule and the SV test dispatches it to the generated parser's per-rule entry (`parse_combinational_entry` / `parse_sequential_entry`), so a sample can lock a *nested* rule's carrier (the 7 other grammars' callbacks ignore the new arg = no behavior change). Added 2 samples to `systemverilog_v1.json` — `udp_combinational_entry` (locks typed `{inputs,output}`) + `udp_sequential_entry` (locks `{inputs,current_state,next_state}`), both asserting `json_object`. VERIFIED: SV shape `samples 3→5 aligned=5 drift=0`; full `ast_shape_contract` suite 18 test-fns pass; **negative control** (transiently flip `current_content_kind→sequence`) FAILS with `observed JsonObject != manifest current_content_kind Sequence` ⇒ the lock BITES on raw↔typed drift; clippy source-clean. The default `ast_shape_contract_gate` aggregate stays 18 (SV compiled out of the default build; the 2 SV samples run under `PGEN_SYSTEMVERILOG_PARSER_PATH`/gate). See the 2026-06-25 Decisions entry + acceptance checklist. |

## Decisions
- `2026-06-29` (`H.12.8.4.3.1` `parseability_probe --entry-rule` tool-build — `-0142`, **CODE**: `rust/src/parser_registry.rs` + `rust/src/bin/parseability_probe.rs`; NO grammar/release/schema/ledger/contract/generated change): the entry-relative debug tool the `.8.4.3` work needed for Step 3 of the UNKNOWN protocol. `parseability_probe --parse <grammar> <file> [--profile P] --entry-rule <RULE>` now parses from an alternate start symbol via the `-0141` `parse_full_from(entry)` dispatch, so a rule rooted under an alternate LRM start symbol (e.g. SV `library_text`) can be reproduced/traced in isolation. Registry: extracted `parse_with_systemverilog_detail_profile_entry(sample, profile, entry)` — the registered 2-arg `ParseDetailFn` delegates with `entry=None` ⇒ byte-identical to the prior `parse_full_systemverilog_file` path, preserving the `furthest_position` augmentation; added parser-agnostic `parse_sample_detail_from_entry(grammar_name, sample, profile, entry)` (each grammar dispatches via `parse_full_from`, regex via its worker stack, meta-grammars without a generated parser return `None`). Probe: `--entry-rule RULE` added to `GlobalOptions` + arg parse (mirrors `--profile`) + `command_parse` entry path + usage. **DECISIVE A/B** (freshly-built release `parseability_probe`): file `include file_path_spec;` ⇒ REJECT from the default entry (`Parser did not consume full input at position 0 [furthest_position=7]`) but `parse_full passed` with `--entry-rule library_text`; control `module m; endmodule` ⇒ `parse_full passed` from the default entry (the `None` path unchanged). NO REGRESSION: cert witness path / codegen / grammars untouched ⇒ all cert numbers byte-identical to `-0141` by construction; `cargo test --lib` 771/0/21; clippy source-clean. Tool-build only; no grammar/release/schema change. Detail: [GRAMMAR-WELLFORMED-H12843-1-parseability-probe-entry-rule.md](GRAMMAR-WELLFORMED-H12843-1-parseability-probe-entry-rule.md). [[feedback_systematically_use_debug_toolbox]], [[feedback_tools_first_no_guessing]], [[feedback_ast_pipeline_parser_agnostic]], [[project_sv_full_certification_via_multi_entry]].
- `2026-06-29` (`H.12.8.4.3` entry-aware certificate witness verification — `-0141`, **CODE**: `rust/src/ast_pipeline/ast_based_generator.rs` + `rust/src/parser_registry.rs` + `rust/src/main.rs` + `rust/src/bin/rtl_frontend_generated_contract_probe.rs`; all 7 grammar parsers regenerated; NO grammar/release/schema/ledger/contract change): **the `-0140` REAL fix LANDED — the SV multi-config cert union drops `UNKNOWN 14 → 3`.** Cert witness VERIFICATION now honors the configured entry. Codegen `generate_parse_method` extracts the reset ceremony into `prepare_parse_state()` and emits `parse_from(entry)` (reset → `match entry { <rule> => self.parse_<rule>(), … _ => <canonical> }`) + `parse_full_from(entry)` (the trailing/full-consume wrap); the dispatch arm set is exactly the rules with a `parse_<rule>` method (present in `grammar_tree`), deduped, canonical = default arm ⇒ single-entry grammars byte-identical. `parser_registry.rs`: `ParseAndCoverFn` + the 7 `parse_and_cover_*` closures + the generic `parse_and_cover` take `entry: Option<&str>` and verify via `parse_full_from` (`None` ⇒ the existing `parse_full_<canonical>` call, unchanged; regex owns the entry into its 'static worker). `main.rs`: the 6 cert `parse_and_cover` sites (inside `gather_cert_covered_sets`) thread the per-config `entry_rule`. **DECISIVE A/B** (`PGEN_CERT_COVERAGE_DUMP_ALL=1 … --cert-union-config systemverilog_file:sv_2023 --cert-union-config sv_multi_entry_root:sv_2017 --cert-union-config library_text:sv_2017 --cert-union-config systemverilog_parseable_file:sv_2017`): canonical `UNKNOWN=22` byte-identical, union **`UNKNOWN 14 → 3`** (`witness 1281 → 1300`), deterministic seeds 0/7/42; union residual = EXACTLY the 3 `.8.3` reach-gaps (`context_member_method_call` + the two `…scoped_call…` cousins). The 11 entry-relative rules witness because, once verification runs from `library_text`, the `file_path_spec`-literal samples (`include file_path_spec;`) are self-consistent — so the `≤6` prediction was beaten and `.8.4.4` (`file_path_spec` LRM char-class) is reframed from witnessing-blocker to LRM-fidelity cleanup. NO REGRESSION: 6 fully-certified grammars byte-identical `fully_certified=true`; `cargo test --lib` 771/0; clippy source-clean; SV external corpus 14/14. `parseability_probe --entry-rule` carved out to `.8.4.3.1`. SV stays `Mostly Done`. Detail: [GRAMMAR-WELLFORMED-H1284-entry-aware-cert-witness-verification.md](GRAMMAR-WELLFORMED-H1284-entry-aware-cert-witness-verification.md). [[feedback_systematically_use_debug_toolbox]], [[feedback_pinpoint_real_blocker_not_menu]], [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_ast_pipeline_parser_agnostic]], [[project_sv_full_certification_via_multi_entry]].
- `2026-06-29` (`H.12.8.4.2` `file_path_spec` implement attempt REFUTED + TRUE root cause — `-0140`, **PURE-DOCS**; implement applied, MEASURED, REVERTED; NO net code/grammar/generated/release/schema/ledger change): the `.8.4.1` prediction (`file_path_spec` fix ⇒ union `14 → ≤ 6`) is **refuted by measurement**. The lexeme edit (`systemverilog.ebnf:5736` → `/[A-Za-z0-9_.\/?*~$+-]+/`) was applied, regenerated, rebuilt, and **verified to take effect** (probe samples became real paths: `include q;`, `library \foo G0;`, `include 3M-3;`), yet the cert union `UNKNOWN` stayed **`14`** (canonical `22`), byte-identical to baseline. **TRUE root cause (tool-proven):** cert witness VERIFICATION is hardwired to a single entry — `parse_and_cover_systemverilog` (`rust/src/parser_registry.rs:559-574`) always calls `parser.parse_full_systemverilog_file()` (`:571`), and `generated/systemverilog_parser.rs` exposes only `pub fn parse_full_systemverilog_file` (+ specify-path helpers); the library cohort are PRIVATE inner fns. So **every** witness verification parses from `systemverilog_file`, regardless of `--entry-rule`/`--cert-union-config` ⇒ the 11 entry-relative rules (LRM `library_text` start symbol, §33 / Annex A.1.1, `no_path` from `systemverilog_file` by design) are **un-witnessable by construction**. Proven: `--entry-rule include_statement` ⇒ `sample_parse_failures=8/8`; `library_description` already complete (`:2594-2597`, all 4 LRM alternatives). **REAL fix (`.8.4.3`, parser-AGNOSTIC infra, NOT a grammar edit):** codegen emits `pub fn parse_full_<entry>` for alternate start symbols + `parse_and_cover` takes an entry and dispatches to it; then the cohort witnesses via `library_text:sv_2017`/`systemverilog_parseable_file:sv_2017` union configs, at which point `file_path_spec`+`library_description` become load-bearing (`.8.4.4`). The `file_path_spec` edit was **REVERTED** (cert-neutral flagship change does not land — *commit only improvements*, [[project_cert_coverage_tournament_loser_leak]]). Also exposed a tool gap: `parseability_probe --parse` has no `--entry-rule` (Step-3 trace unavailable for entry-relative rules) — fold into `.8.4.3`. Full evidence in [GRAMMAR-WELLFORMED-H1284-entry-relative-cert-verification-entry-rootcause.md](GRAMMAR-WELLFORMED-H1284-entry-relative-cert-verification-entry-rootcause.md). [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_always_signoff_decisions]], [[feedback_pinpoint_real_blocker_not_menu]], [[project_sv_full_certification_via_multi_entry]].
- `2026-06-29` (`H.12.8.4.1` entry-relative cohort WHY+WHERE + LRM-grounded fix design — `-0139`, **PURE-DOCS**; NO code/grammar/generated/release/schema/ledger change): tools-first root-cause map of the 11 entry-relative union-residual rules (the SV union `UNKNOWN=14` cohort minus the 3 `.8.3` reach-gaps), reproduced this session via `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 … --entry-rule sv_multi_entry_root --count 40 --seed 0`. **Mechanism A (dominant, 6 `parsed=false`):** the generator emits the literal text `file_path_spec` because `grammars/systemverilog.ebnf:5736` `kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/` is an LRM-extraction artifact (the IEEE 1800-2017 §33.3.1 / Annex A.1.1 file-path nonterminal flattened into a literal-keyword terminal — same defect class as `kw_n_29`/`kw_n_48`); it transitively blocks `library_text`/`library_description` ⇒ dominates **8 of 11**. **Mechanism B (3):** multi-entry scaffolding routing (`sv_multi_entry_root`/`systemverilog_parseable_file`/`parseable_source_item` pick a trivial/empty alternative). **Fix design (`.8.4.2`):** replace the `:5736` lexeme with the LRM-faithful path char-class `/[A-Za-z0-9_.\/?*~$+-]+/`, keeping the rule name + arity (AST-shape-neutral, schema stays 6); expected union `UNKNOWN 14 → ≤ 6`, canonical `22` unchanged (the library cohort is `no_path` from `systemverilog_file` by LRM design). Full evidence table + LRM grounding + ceremony in [GRAMMAR-WELLFORMED-H1284-entry-relative-filepathspec-whywhere-design.md](GRAMMAR-WELLFORMED-H1284-entry-relative-filepathspec-whywhere-design.md). [[feedback_systematically_use_debug_toolbox]], [[feedback_why_and_where_before_solution]], [[feedback_no_rule_deletion_without_lrm_proof]], [[project_sv_full_certification_via_multi_entry]].
- `2026-06-29` (`H.12.8.1.1` multi-config cert-union **IMPLEMENT** — `-0138`, **CODE**: `rust/src/main.rs` + `rust/src/ast_pipeline/grammar_wellformedness.rs`; NO grammar/generated/release/schema/ledger change; book + live-doc lockstep): **landed the opt-in `--cert-union-config <entry>[:<profile>]` multi-config certificate-coverage union per the `-0137` design.** A repeatable CLI flag whose default-empty value reproduces today's single-config behavior byte-identically for EVERY grammar; when present, each value names an additional `(entry, profile)` config whose verified `proof ∪ witness` covered sets union into the canonical accounting, classifying the canonical `rule_order` against the bigger covered sets (the load-bearing soundness rule: union over POSITIVELY-covered sets, NEVER over "not-UNKNOWN-in-some-config"). The refactor extracts the per-config covered-set passes (diverse PASS-1 + proof gathering + constructive-reach + plannable + target-own + store-free + carrier-div) into `gather_cert_covered_sets(... emit_diagnostics) -> CertCoveredSets` (canonical call verbose/byte-identical; union calls quiet); derives `Clone` on `LoadedGrammar` to re-filter the unfiltered bundle per union profile; prints the byte-identical canonical `CERTIFICATE-COVERAGE:` line plus a new `CERTIFICATE-COVERAGE-UNION:` line when configs are present. SV ⇒ **`UNKNOWN 22 → 14`** (deterministic seeds 0/7/42; certifies the 6 profile-relative rules **+ the 2 extraction leaves `kw_n_29`/`kw_n_48`**, all witnessed under `sv_2023`; `sv_multi_entry_root` adds 0 — that is `.8.4`). **TOOL-CORRECTION to the `-0137` design's `22 → 16`** (per [[feedback_be_alert_root_cause_fishy_immediately]] + [[feedback_no_codebase_change_without_tool_backed_facts]]): the design asserted `kw_n_29`/`kw_n_48` are profile-filtered OUT of `sv_2023`; `--dump-gen-ast --grammar-profile sv_2023` (`kw_n_29`×3, `kw_n_48`×5 present) + `sv_2023` cert (UNKNOWN=17, neither in it) prove they are PRESENT and WITNESSED there, so the sound union legitimately certifies them. `.8.2` (the `kw_n_*` artifacts) is therefore SUBSUMED by the union; residual 14 = 11 entry-relative (`.8.4`) + 3 reach-gaps (`.8.3`). Parser-agnostic, inert by construction for the 6 fully-certified grammars (they never pass the flag). Tools-first per [[feedback_systematically_use_debug_toolbox]] + [[feedback_no_codebase_change_without_tool_backed_facts]]; ONE clean approach per [[feedback_pinpoint_real_blocker_not_menu]]. Detail: the `H.12.8.1.1` frontier row + [project_sv_full_certification_via_multi_entry.md](../decisions/project_sv_full_certification_via_multi_entry.md) `## ✅ .8.1 IMPLEMENTATION DESIGN`.

  ### Acceptance Checklist (enforced) — H.12.8.1.1
  - [x] **REPRODUCE / ISSUE** — canonical SV cert (this session, Jun-25 binary): `ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0` ⇒ `CERTIFICATE-COVERAGE: grammar='systemverilog' … total=1304 proof=1 witness=1281 UNKNOWN=22 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)` (exact `-0136`/`H.12.7` baseline). SV is the ONLY non-fully-certified shipped grammar; the 6 IEEE-1800-2023 rules (`class_constructor_super_args`, `declared_interface_class_identifier`, `interface_class_declaration`/`_item`/`_method`, `union_modifier`) DO witness under `--grammar-profile sv_2023` (`UNKNOWN 22→17`) but the single-config canonical accounting cannot credit them.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `run_certificate_coverage_report` (`rust/src/main.rs:2349`) classifies the canonical `rule_order` against a SINGLE-config `witness_covered`/`proof_covered`, so a rule witnessed only under a different supported profile stays UNKNOWN. The classifier `certificate_coverage(all_fragments, proof_covered, witness_covered)` (`rust/src/ast_pipeline/grammar_wellformedness.rs:1685`) already takes the covered SETS as inputs — the correct primitive — it just needs bigger (unioned) covered sets and the same canonical `all_fragments`. Soundness subtlety (root-caused, not assumed, via `PGEN_CERT_COVERAGE_DUMP_ALL=1` set-diff): naïve `canonical_UNKNOWN − sv_2023_UNKNOWN` = 8, but `kw_n_29`/`kw_n_48` are profile-FILTERED-OUT of `sv_2023`'s `rule_order` (ABSENT, not witnessed) ⇒ the union MUST be over POSITIVELY-covered (`proof ∪ witness`) sets, never over "not-UNKNOWN-in-some-config."
  - [x] **FIX** — tier = proof-tooling (parser-agnostic; NO grammar/engine/codegen change). Opt-in repeatable `--cert-union-config <entry>[:<profile>]`; extract `gather_cert_covered_sets(... emit_diagnostics) -> CertCoveredSets`; derive `Clone` on `LoadedGrammar`; classify canonical `rule_order` against `⋃(proof ∪ witness)`; print a `CERTIFICATE-COVERAGE-UNION:` line when configs are present. Default-empty ⇒ byte-identical canonical path.
  - [x] **ADDRESSED (verified)** — SV union (`--cert-union-config systemverilog_file:sv_2023 --cert-union-config sv_multi_entry_root:sv_2017`) ⇒ canonical line **byte-identical `UNKNOWN=22`** + `CERTIFICATE-COVERAGE-UNION: … witness=1289 UNKNOWN=14 fully_certified=false`, **deterministic at seeds 0/7/42**. Residual 14 = the 11 entry-relative + the 3 reach-gaps (`context_member_method_call`, the 2 `…scoped_call…`) — the 6 profile rules AND `kw_n_29`/`kw_n_48` are certified (NOT in residual). Isolation runs proved soundness + root-caused the 14-vs-16: sv_2023-only union ⇒ 14; svmer-only union ⇒ 22 (adds 0); `--dump-gen-ast --grammar-profile sv_2023` shows `kw_n_29`×3/`kw_n_48`×5 PRESENT and sv_2023 cert (UNKNOWN=17) covers them ⇒ legitimately witnessed, NOT a "not-UNKNOWN" leak. Lock test `certificate_coverage_union_is_over_positively_covered_sets` (pure, proves a rule covered by NO config stays UNKNOWN; a naïve per-config-UNKNOWN-subtraction would falsely certify it) — `cargo test --lib` ⇒ `ok`.
  - [x] **NO REGRESSION** — SV canonical (NO flag) byte-identical `total=1304 proof=1 witness=1281 UNKNOWN=22 fully_certified=false (sample_parse_failures=0, …)` at seeds 0/7/42 (= the `-0136` baseline). The **6 fully-certified grammars byte-identical `fully_certified=true UNKNOWN=0`** at their canonical entries (json 9, regex 198, vhdl 216, svpp 74, rtl_frontend 169 proof=1, rtl_const_expr 48 `--max-depth 32`). `cargo test --lib` (from `rust/`) ⇒ **669 passed; 0 failed**. `make clippy_on_rust_change` ⇒ **✅ completed** (source strict clean; generated-parser stage debt pre-existing, non-strict). SV external corpus 14/14 + `ast_shape_contract` GREEN by construction: `git diff` touches ONLY `rust/src/main.rs` (cert tooling) + `rust/src/ast_pipeline/grammar_wellformedness.rs` (one `#[cfg(test)]` fn) + docs — ZERO grammar/codegen/generated bytes; `generated/*` untouched, `parseability_probe` parser behavior unchanged, so the corpus/shape surfaces cannot move.
  - [x] **LOCKSTEP** — book `docs/book/src/grammar-wellformedness.md` (cert-union flag + `CERTIFICATE-COVERAGE-UNION:` + the corrected `22 → 14` accounting), `diagnosing-unknowns.md` (toolbox table row), `cli-and-workflows.md` (capability list); `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md` / `LIVE_ACHIEVEMENT_STATUS.md`; decision record `project_sv_full_certification_via_multi_entry.md` (`22→14` IMPLEMENT correction). NO contract/ledger/schema/release bump (unshipped proof-tooling, no shipped-parser behavior change; SV LIVE row UNCHANGED `Mostly Done`, `UNKNOWN=22` canonical / `14` via the opt-in union).

- `2026-06-29` (`H.12.8.1` multi-config cert-union **DESIGN** — `-0137`, **PURE-DOCS**; NO code/grammar/generated/release/schema/ledger change): **the concrete, tool-backed implementation design for the director-chosen `.8.1` multi-entry/multi-profile cert-coverage union, decided design-first per the project DESIGN→IMPLEMENT cadence (the `.8.1` refactor is ~400 lines + a new CLI surface + new struct, too large to land cold as the first slice of a fresh session).** Tools-first re-derivation on the Jun-25 binary (read-only) reproduced canonical `total=1304 … UNKNOWN=22 spf=0` (seed 0) and `sv_2023` `total=1324 … UNKNOWN=17`, then ROOT-CAUSED a fishy set-diff per [[feedback_be_alert_root_cause_fishy_immediately]]: the naïve `canonical_UNKNOWN − sv_2023_UNKNOWN` returns **8** (the 6 profile rules + `kw_n_29`/`kw_n_48`), but `kw_n_29`/`kw_n_48` are ABSENT from `sv_2023`'s `rule_order` (profile-filtered out — their sv_2017 host productions `systemverilog.ebnf:1492`/`:2869`/`:3926` are profile-variant), so they are NOT witnessed there. **This proves the load-bearing soundness rule:** the union must classify the canonical fragment set against `⋃(proof_covered ∪ witness_covered)` — POSITIVELY covered in some config — **never** over "not-UNKNOWN-in-some-config" (which would falsely certify profile-filtered-out rules). The sound union certifies only the 6 genuinely-witnessed profile rules ⇒ **`22 → 16` CONFIRMED** (matching `-0136`). DECIDED (director "make the technical decisions"): explicit opt-in **`--cert-union-config <entry>[:<profile>]`** (repeatable), default-empty ⇒ byte-identical single-config behavior; refactor extracts `gather_cert_covered_sets(... emit_diagnostics) -> CertCoveredSets` (canonical verbose/byte-identical, union quiet); derive `Clone` on `LoadedGrammar` to re-filter per profile; prints the canonical line byte-identical + a `CERTIFICATE-COVERAGE-UNION:` line when configs present. Done tools-first per [[feedback_systematically_use_debug_toolbox]] + [[feedback_pinpoint_real_blocker_not_menu]] (ONE chosen approach, no menu) + [[feedback_no_codebase_change_without_tool_backed_facts]] (design rests on this-session tool evidence). NO book change yet (the union CLI is unshipped — book/contract lockstep at `.8.1.1` IMPLEMENT). SV LIVE row UNCHANGED (`Mostly Done`, `UNKNOWN=22`). Detail: the `H.12.8`/`.8.1` frontier row + decision record `## ✅ .8.1 IMPLEMENTATION DESIGN`.
- `2026-06-25` (`H.14.3.1` UDP truth-table entry shape-lock — `-0131`, **HARDENING**; `rust/src/ast_shape_contract.rs` (test harness) + `rust/test_data/ast_shape_contract/systemverilog_v1.json` (test manifest) only; NO grammar / codegen / generated-parser / release / schema / ledger change): **the follow-up ticketed by `H.14.3` — the `SV-0009` raw→typed UDP regression hid ~48 slices because the SV `ast_shape_contract` gate's 3 samples were all the root `systemverilog_file`, so no sample exercised a nested UDP truth-table entry. Now locked: the gate harness gained a non-root `rule_under_test` capability and 2 UDP-entry samples assert the typed `json_object` carriers, so any future raw↔typed drift trips the hard regression-lock.** Done tools-first per [[feedback_systematically_use_debug_toolbox]] (negative-control-proven the lock bites) + [[feedback_be_alert_root_cause_fishy_immediately]] (close the silent-regression class the `H.14.3` defect exposed) + [[feedback_corpus_expected_from_spec_not_fix]] (the locked keys are derived from the grammar's own return annotations, not from observed output). Detail in the frontier `H.14`/`H.14.3.1` rows.
  - **Gate-mechanism extension (general, parser-agnostic):** `run_manifest`'s parser callback signature changed from `FnMut(&str)` to `FnMut(&str, &str)` (input + the sample's `rule_under_test`). Only the SystemVerilog test callback dispatches on the rule — to the generated parser's per-rule entry methods `parse_combinational_entry` / `parse_sequential_entry` — so a sample can lock a *nested* rule whose own return annotation is the carrier. The 7 other grammar callbacks bind the new arg as `_rule` (byte-no-op). All 8 `run_manifest` callers live inside the `ast_shape_contract.rs` test module; no shipped library/binary path consumes `run_manifest`, so nothing ships-behaviorally changes.
  - **Why option (a) not a deep-key walk:** under return annotations the root `systemverilog_file` folds its whole subtree into one `ParseContent::Json`, so a nested carrier would have to be found by JSON-path navigation (fragile, shape-dependent). Parsing the input *as* the UDP-entry rule yields the carrier at top level and reuses the existing `expected_json_object_keys_present` machinery directly; because the carrier comes from the rule's OWN annotation, the standalone shape is identical to the in-context shape — exactly the property that regressed.

  ### Acceptance Checklist (enforced) — H.14.3.1
  - [x] **REPRODUCE / ISSUE** — `grep '"rule_under_test"' rust/test_data/ast_shape_contract/systemverilog_v1.json` ⇒ all 3 samples = `systemverilog_file` (root only); `combinational_entry`/`sequential_entry` UNCOVERED. This is why the `H.14.3` raw→typed UDP regression (`SV-0009`) survived ~48 slices: the shape-contract gate could not see a nested UDP carrier.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `rust/src/ast_shape_contract.rs::run_manifest` (the gate harness) passed only the sample `input` to the parser callback (`FnMut(&str)`), and the SV callback parsed the whole input via `parse_full_systemverilog_file`, so the runner classified/asserted the **top-level** carrier only — there was no mechanism to lock a nested rule's shape. WHERE = `run_manifest` (callback signature + the per-sample call) + the SV test callback. Tool-confirmed the fix is feasible: `grep -aoE 'pub fn parse_(combinational|sequential)_entry' generated/systemverilog_parser.rs` ⇒ the generated parser exposes per-rule entry methods (same `ParseResult<ParseNode>` type as the root entry).
  - [x] **FIX** — tier = test-harness/proof-surface (no grammar/engine change). Thread `rule_under_test` to the callback; SV dispatches `combinational_entry`→`parse_combinational_entry`, `sequential_entry`→`parse_sequential_entry`, else root. Add 2 manifest samples locking the typed carriers (`{inputs,output}` and `{inputs,current_state,next_state}`) as `json_object`. Minimal inputs derived from the grammar: `0 1 : 1 ;` (combinational), `0 : 1 : 0 ;` (sequential).
  - [x] **ADDRESSED (verified)** — `PGEN_SYSTEMVERILOG_PARSER_PATH=… cargo test --features generated_parsers --lib ast_shape_contract` ⇒ `[systemverilog] samples=5 aligned=5 drift=0 regression_lock_failures=0`, both new samples `observed=JsonObject … structural_ok=true`; suite `test result: ok. 18 passed; 0 failed`. **Negative control (lock bites):** transiently set `udp_combinational_entry.current_content_kind→sequence` ⇒ `test result: FAILED. 0 passed; 1 failed` with `sample 'udp_combinational_entry': observed content_kind JsonObject != manifest current_content_kind Sequence` (the runner's hard regression-lock) — then reverted (manifest restored from `git show HEAD:…` + re-applied the 2-sample edit; `git diff` = +31/-0, pure addition).
  - [x] **NO REGRESSION** — full `ast_shape_contract` suite GREEN (18 test-fns pass; the 7 non-SV families byte-unchanged: regex 4, return_annotation 3+1-documented-drift, semantic 2, rtl_const_expr 2, rtl_frontend 3, vhdl 3, svpp 6); `clippy_on_rust_change` source stage `clippy_source_all_targets → ok` (generated debt pre-existing, non-strict). Cert seeds 0/7/42 `UNKNOWN=28` and external corpus 14/14 are unaffected BY CONSTRUCTION — `git diff --stat` = ONLY `rust/src/ast_shape_contract.rs` + `systemverilog_v1.json` (a test harness + test data); ZERO grammar/codegen bytes, `generated/systemverilog_parser.rs` untouched (not regenerated), so the cert/corpus/parser-behavior surfaces cannot move. The 6 fully-certified grammars inert.
  - [x] **LOCKSTEP** — developer-facing proof-surface capability ⇒ book `docs/book/src/quality-and-closure-model.md` note (the shape-contract harness can lock non-root / nested rule carriers; SV UDP entries are the first use). `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md` / `LIVE_ACHIEVEMENT_STATUS.md` updated. NO contract / ledger / schema / release bump (no shipped-parser behavior change). SV main-parser LIVE row UNCHANGED (`Mostly Done`, `UNKNOWN=28`).

- `2026-06-25` (`H.14.3` EBNF self-hosting — LAND the 12/12 fix + released-SV ceremony, `-0130`, **GRAMMAR FIX — flagship SystemVerilog**; `grammars/systemverilog.ebnf` 2 lines deleted; `generated/` regenerated/untracked; **schema 5→6, release 1.0.146→1.0.147, ledger SV-0009**): **the turnkey fix from `H.14.2`. Deleting the duplicate `->` on `combinational_entry` (`:1168`) + `sequential_entry` (`:4639`) makes `systemverilog.ebnf` self-parse 100% ⇒ EBNF self-hosting 12/12 — AND restores the two rules' typed AST carriers, which had silently regressed to raw `Sequence` since SV-Slice-66 (`1.0.66`).** The earlier deferral (`H.14.2`) was because SV codegen was byte-non-deterministic; `CODEGEN-DETERMINISM.1` fixed that, so the scope/shape was adjudicable this session. Flagged + adjudicated tools-first per [[feedback_be_alert_root_cause_fishy_immediately]] + [[feedback_always_signoff_decisions]]; lockstep per [[feedback_ebnf_meta_grammar_lockstep]] + [[feedback_regex_book_live]]. Detail in the frontier `H.14`/`H.14.3` rows + ledger `SV-0009` + the contract 1.0.147 schema note.
  - **Scope discovery:** the `--emit-raw-ast-json` (generation input) was NOT byte-identical before/after — the duplicate `->` was absorbed into the return-annotation TEXT (`"{inputs: $1, output: $3}\n -> {inputs: $1, output: $3}"`). The OLD generated parser shows `_pgen_unparsed_return_annotation_warning` (FAILED TO PARSE) → raw `ParseContent::Sequence`; the NEW one builds `ParseContent::Json(Object{…})`. So this is an AST-fidelity fix (raw→typed), not a no-op — hence the released-SV ceremony.
  - **Schema adjudication (5→6):** a previously-VALID UDP truth-table parse's carrier changes (raw→typed) — the covergroup `SV-0006`/`SV-0007` criterion; distinct from the SVA `SV-0008` no-bump case (previously-invalid input only). The book json-carrier already documented the typed shape (the contract was unchanged; the realized parser had regressed), but the realized carrier changes for valid input ⇒ bump. No SV schema code constant exists (doc-tracked in the contract); bump landed in contract + ledger + book.

  ### Acceptance Checklist (enforced) — H.14.3
  - [x] **REPRODUCE / ISSUE** — `rust/target/debug/ebnf_dual_run_diff --input grammars/systemverilog.ebnf` ⇒ `parse_full.ok=false error_position=72476`, `error_context="-> {inputs: $1, output: $3}\n ... -> {inputs: $1, output: $3} ..."` (the duplicate at `combinational_entry`). Generated ebnf parser self-parsed 11/12 tracked grammars (SV the last holdout). BEFORE empirical: `parseability_probe --parse-dump-ast-pretty systemverilog <udp>.sv` ⇒ `combinational_entry` is raw `Sequence` (`"kind":"colon"`/`"kind":"semi"` present, no `inputs`/`output`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `git log -L 1166,1168:grammars/systemverilog.ebnf` ⇒ SV-Slice-66 (`b48f66de`, `1.0.66`) added a SECOND identical `->`; SV-Slice-18 (`399c70c3`) had added the correct single `->`. A single EBNF rule carries ONE rule-level `return_annotation?`; the generated parser's return-transform compiler emitted `_pgen_unparsed_return_annotation_warning` for the doubled text and fell back to raw `ParseContent::Sequence`. WHERE = `grammars/systemverilog.ebnf:1168` (`combinational_entry`) + `:4639` (`sequential_entry`).
  - [x] **FIX** — deleted the redundant 2nd `->` at `:1168` and `:4639` (declarative > grammar > engine: this is the **grammar** tier — removing a defect, not adding machinery). Per [[project_grammar_wellformedness_contract]] the meta-grammar must NOT model a duplicate `->`; the duplicate is a shipped-grammar defect.
  - [x] **ADDRESSED (verified)** — SV self-host `error_position=72476 → parse_full.ok=true` (100%) ⇒ **EBNF self-hosting 12/12** (all-grammars `ebnf_dual_run_diff` scan; 3 raw LRM-extraction snapshots out of scope). AFTER empirical: `combinational_entry → {"inputs":[{"kind":"0"},{"kind":"0"}],"output":{"kind":"0"}}` (typed; colon/semi folded away), `sequential_entry → {inputs,current_state,next_state}` ×2. Regenerated-parser byte-diff = 97 lines, confined to EXACTLY `parse_combinational_entry` + `parse_sequential_entry` (`ParseContent::Sequence` → `ParseContent::Json(Object{…})`).
  - [x] **NO REGRESSION** — SV cert seeds 0/7/42 `total=1288 proof=1 witness=1259 UNKNOWN=28 sample_parse_failures=0 proof_reverify_failures=0` (= baseline, deterministic); `make ast_shape_contract_gate` ⇒ `test result: ok. 18 passed; 0 failed`; `make sv_external_corpus_triage_gate` ⇒ `cases_executed=14 parse_pass_total=14 parse_fail_total=0`; `make clippy_on_rust_change` source-clean (generated debt pre-existing, non-strict); the 6 fully-certified grammars untouched (grammar-local change).
  - [x] **LOCKSTEP** — `[[feedback_ebnf_meta_grammar_lockstep]]` + `[[feedback_regex_book_live]]`: ledger `SV-0009`; contract `PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` (contract+release 1.0.147, schema 5→6, last-updated); book `docs/systemverilog_parser_book/src/changelog-index.md` (1.0.147 entry) + `json-carrier.md` (UDP-entry realization note); top book `docs/book/src/grammar-wellformedness.md` (12/12 milestone); `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md` / `LIVE_ACHIEVEMENT_STATUS.md`. Follow-up `H.14.3.1` ticketed (UDP shape-lock). SV main-parser LIVE row UNCHANGED (`Mostly Done`, `UNKNOWN=28`).

- `2026-06-25` (`H.14.2` EBNF self-hosting — root-cause the LAST `systemverilog` self-parse gap (= a duplicate-return-annotation DEFECT) + prove the fix is flagship-SV behavior territory, `-0129`, **PURE-DOCS INVESTIGATION** — the SV grammar edit was made, MEASURED, then REVERTED; NO code/grammar/generated/release/schema/ledger change lands this slice): **after `H.14.1`, `systemverilog` self-parse stops @72476 at a genuine grammar DEFECT — a duplicate identical return annotation on two UDP-table rules (`combinational_entry` `:1167-1168`, `sequential_entry` `:4638-4639`). Deleting the redundant second `->` at each site makes `systemverilog` self-parse 100% (369412/369413) ⇒ EBNF self-hosting 11/12 → 12/12. BUT the fix is a SHIPPED-SV-PARSER change that CANNOT be cheaply verified, so it is DEFERRED to a fresh-budget SV session.** Flagged + root-caused immediately per [[feedback_be_alert_root_cause_fishy_immediately]]; deferred per [[feedback_always_signoff_decisions]] + [[feedback_correctness_before_speed]] (do not rush a flagship-parser change at the tail of a long turn). Detail in the frontier `H.14`/`H.14.2` rows.
  - **REPRODUCE (tool-backed):** `ebnf_dual_run_diff --input grammars/systemverilog.ebnf` ⇒ `parse_full.ok=false error_position=72476` → maps to `grammars/systemverilog.ebnf:1168`, the SECOND of two identical `-> {inputs: $1, output: $3}` on `combinational_entry`. Whole-file scan: exactly TWO duplicate consecutive identical `->` (`:1168` combinational_entry; `:4639` sequential_entry `-> {inputs: $1, current_state: $3, next_state: $5}`); ZERO non-identical.
  - **ROOT CAUSE (WHY+WHERE):** a single EBNF rule carries ONE rule-level `return_annotation?`; the generated ebnf parser has no production for a SECOND consecutive `->`, so it strands at the duplicate. This is a defect in the SHIPPED grammar (NOT a missing `ebnf.ebnf` construct — per [[project_grammar_wellformedness_contract]] the meta-grammar must NOT model a duplicate `->`). WHERE = `grammars/systemverilog.ebnf:1168` and `:4639`.
  - **MEASURED (then REVERTED):** removed the 2 lines → SV self-parse `72476 → 369412/369413` (100%, **12/12** full self-hosting). SV parser md5 `cbe76f0e` (no-change regen) → `8cf1515b` (after-fix). BUT a control regen of the SAME committed grammar (after `git checkout`) produced a THIRD md5 `11ebfda1` ⇒ **SV codegen is BYTE-NON-DETERMINISTIC**, so the md5 deltas are CONFOUNDED and byte-identity CANNOT decide whether the duplicate removal changes PARSE BEHAVIOR. (My earlier "determinism check" was a make no-op — make skipped the regen because nothing changed.) Working tree REVERTED; `generated/systemverilog_parser.rs` regenerated back to the committed grammar.
  - **WHY DEFERRED:** because byte-identity is unavailable, landing the fix safely needs the full released-SV ceremony: cert seeds 0/7/42 (`UNKNOWN=28`, `spf=0`), external corpus 14/14, `ast_shape_contract` 16/16, and adjudication of whether the UDP-entry AST shape changes (the double `->` may have been emitting a double transform) ⇒ possible schema/release/ledger + SV/top books. That is the "fresh-budget effort / clean handoff point" the resume pointer designates.
  - **NEW SUB-FINDING (separate, tracked):** SV codegen is byte-non-deterministic across regens of the same grammar (`cbe76f0e` vs `11ebfda1`) — likely HashMap/HashSet iteration order in codegen; cert stays seed-deterministic because that is a SEMANTIC property (seeded RNG), independent of codegen byte layout. A reproducible-build concern worth its own investigation/leaf (candidate). Does NOT block the cert proof surface.
  - **RECOVERY (turnkey for the fresh SV session):** delete `grammars/systemverilog.ebnf:1168` and `:4639` (the 2nd `->` at each site) → 12/12 self-host; then run the released-SV ceremony above. The duplicate removal is the correct fix; only its flagship verification is deferred.

- `2026-06-25` (`H.14.1` EBNF meta-grammar lockstep — port the **`::N*` extraction-spread construct family** to `ebnf.ebnf`, `-0128`, **GRAMMAR**, meta-grammar only — `generated/ebnf.rs` regenerated/untracked, NO production-parser regen, NO release/schema/ledger change): **the original `-0068` gap #2 (`::N*` extraction-spread) — listed in the `H.13` row but never closed there because the dual-run gate's 3 tracked grammars (ebnf/json/regex) don't use it — is the first `H.14` drain. The `$N::target spread?` extraction form (`$3::2*`, `$2::first`, `$2::2`) used by all 4 self-parse-failing grammars (vhdl/rtl_frontend/svpp/systemverilog) is absent from `grammars/ebnf.ebnf`, so the generated ebnf parser cannot self-parse them.** First `H.14` lane slice; advances `vhdl`/`rtl_frontend` dual-run `parse_full` past their earliest gaps (vhdl @1666, rtl_frontend @5018). SV stays `UNKNOWN=28` (meta-grammar-only). Detail in the frontier `H.14`/`H.14.1` rows.

  ### Acceptance Checklist (enforced) — H.14.1
  - [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff` minimal probes (scratchpad test grammars, the generated ebnf parser is the judge): `r := a b -> [$1::2*]` ⇒ `parse_full.ok=false error_position=9`; `r := a comma b -> [$1, $3::2*]` ⇒ `false @15`; vhdl-exact `library_clause := … -> [$2, $3::2*]` ⇒ `false @65`. Controls PASS: `-> [$1, $3]` (plain scalars) `ok=true`, `-> [$2**]` (flatten-spread, closed in `H.13.1`) `ok=true`. Whole-grammar: `ebnf_dual_run_diff --input grammars/vhdl.ebnf` ⇒ `parse_full.ok=false error_position=1666` (the `library_clause := kw_library identifier (comma identifier)* semi -> [$2, $3::2*]` at `grammars/vhdl.ebnf:31`); `rtl_frontend` @5018.
  - [x] **ROOT CAUSE (WHY + WHERE)** — grammar read of `grammars/ebnf.ebnf`: `array_element_return` (`:356-362` = `flatten_reference | quantified_reference | object_return | scalar_return | spread_reference`) and `scalar_return` (`:317-322`) have NO `$N::…` extraction form; `quantified_reference := positional_reference quantified_marker` (`:371`) where `quantified_marker := ("*" | "+" | "?")` (`:374`) models ONLY plain quantifier markers, not the `::index spread?` extraction. So for `$3::2*`, PEG tries flatten (`$3`+`**`→fail on `::`), quantified (`$3`+`*|+|?`→fail on `::`), then `scalar_return`/`positional_reference` commits to bare `$3` and strands `::2*`. WHERE = `array_element_return` / `scalar_return` ordered choice (`grammars/ebnf.ebnf:317-362`). Authoritative model the meta-grammar must mirror: `grammars/return_annotation.ebnf:43` `extraction_expression := positional_reference '::' extraction_target spread_suffix?`, `extraction_target := positive_integer | 'first' | 'last'` (`:52`), `spread_suffix := '*'` (`:91`). Footprint survey (`grep '\$N::'`): 138 uses — vhdl 17, rtl_frontend 18, svpp 2, systemverilog 119+; shapes `$N::2*` (153), `$N::3*` (8), `$N::2` (3, no-spread scalar), `$N::first`/`$N::last`/`$N::first*`/`$N::1*`.
  - [x] **FIX** — tier = grammar ([[project_ebnf_is_single_source_of_truth]], [[feedback_ebnf_meta_grammar_lockstep]]). Added `extraction_reference := positional_reference "::" extraction_target spread_marker?` (+ helpers `extraction_target := ( positive_integer_literal | "first" | "last" )`, `spread_marker := "*"`) mirroring `return_annotation.ebnf`. Placed FIRST in `scalar_return`'s ordered choice (before `reference_property_access`/`positional_reference`) so the `::` form binds before a bare `$N` is committed — a `$N` with no `::` fails the required `"::"` and falls through to the existing alternatives, leaving existing scalar/array returns byte-unchanged. Targeted to the proven shapes; no dual-run grammar uses a `$name::` (named-ref) extraction, so the base stays `positional_reference` (the `return_annotation.ebnf` model).
  - [x] **ADDRESSED (verified)** — regen (`ast_pipeline grammars/ebnf.ebnf --emit-raw-ast-json generated/ebnf.json` → `--generate-parser --debug --eliminate-left-recursion` → `generated/ebnf.rs`; 4 new `parse_extraction_*`/`parse_spread_marker` fns present) + rebuilt `ebnf_dual_run_diff`. Minimal probes FAIL→PASS: `[$1::2*]`, `[$1, $3::2*]`, `$2::first`, `$2::2`, `[$2::last]` all `parse_full.ok=true` (were `false`); controls still PASS (`[$1,$3]`, `[$2**]`, `[$1,$2*]`, `{m:$1.min}`). Whole-grammar self-parse: **`vhdl` 1666→100% (32776/32777), `rtl_frontend` 5018→100% (25501/25502), `systemverilog_preprocessor` 6900→100% (14586/14587)** — all three had `::N*` as their first gap and now fully self-parse; `systemverilog` advances `32189→72476` (19.6%, next deeper gap → future `H.14` slice). **EBNF self-hosting 8/12 → 11/12** (only `systemverilog` remains; full 12-grammar scan confirmed).
  - [x] **NO REGRESSION** — `make -C rust ebnf_frontend_dual_run_gate` STRICT **exit 0** (`ebnf` 100% rule_count 127→131 raw_ast parity, `json` 99.90% parity, `regex` 100% — all `overall=pass`); the independent Perl-frontend regen path proves the edit is Perl-parseable too. SV `UNKNOWN=28` unchanged — `generated/systemverilog_parser.rs` byte-identical (mtime 2026-06-24 23:37, untouched; only `generated/ebnf.*` regenerated) ⇒ meta-grammar-only, no production-parser regen (same precedent as `H.13.1`–`.5`). NO release/schema/ledger change. `git status`: only `grammars/ebnf.ebnf` + `docs/tasks/GRAMMAR-WELLFORMED.md` tracked-modified; `generated/` untracked (not staged).
  - [x] **LOCKSTEP** — [[feedback_ebnf_meta_grammar_lockstep]]; `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md` updated at commit; `LIVE_ACHIEVEMENT_STATUS.md` tracker note (self-host 8/12→11/12). Book: `::N*` extraction-spread is an existing documented return-annotation construct (`docs/book/src/annotation-system.md`) — this is meta-grammar-internal self-hosting catch-up; added the EBNF-self-hosting-milestone note recommended by the `H.13.5` leaf to `docs/book/src/grammar-wellformedness.md`. NO contract/ledger (meta-grammar internal).

- `2026-06-24` (`H.13.5` EBNF meta-grammar lockstep — port the **`null` object-value literal** to `ebnf.ebnf`, `-0126`, **GRAMMAR**, meta-grammar only — `generated/ebnf.rs` regenerated/untracked, NO production-parser regen, NO release/schema/ledger change): **closing the `null` gap flips the `ebnf_frontend_dual_run_gate` strict-GREEN — `regex.ebnf` now self-parses to 100% (the LAST `rust_parse_full` gap), the feature-richest tracked grammar.** `grammars/ebnf.ebnf`'s `literal_return := ( quoted_string | numeric_literal | boolean_literal )` had no `null` alternative, so regex's `quant_base = "*" -> {min: 0, max: null}` (`regex.ebnf:84`) failed at `null`. Tools-first isolation (`ebnf_dual_run_diff`): `-> {m: true}` / `-> {m: "x"}` PASS but `-> {m: null}` / `-> null` FAIL; whole `regex.ebnf` `error_position=4533` at `quant_base`. FIX (mirrors `grammars/return_annotation.ebnf`'s `null_literal := 'null'`): add `null_literal := "null" -> {type:"null"}` to `literal_return`'s ordered choice (non-shadowing — `null` shares no prefix with quote/digit/`true`|`false`). VERIFIED: `null` probes FAIL→PASS; `regex.ebnf` `parse_end 4533 → 78429/78429` (100%); **`make -C rust ebnf_frontend_dual_run_gate` STRICT exit 0** (ebnf 127/127, json 99.90%, regex 100% all `overall=pass`). **HONEST SCOPE (never over-claim, [[feedback_always_signoff_decisions]]):** the gate tracks only `ebnf`/`json`/`regex`; a fresh all-grammars `ebnf_dual_run_diff` scan = generated ebnf parser self-parses **8/12** (`systemverilog`/`vhdl`/`systemverilog_preprocessor`/`rtl_frontend` still FAIL) ⇒ FULL self-hosting continues under new leaf `H.14`; `H.13` is `done` re: its gate-criterion only. SV stays `UNKNOWN=28`. Detail in the frontier `H.13`/`H.13.5`/`H.14` rows. **Commit: `PGEN-GRAMMAR-WELLFORMED-0126`.**

  ### Acceptance Checklist (enforced) — H.13.5
  - [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff --input` minimal-probe matrix (pre-fix): `r := "a" -> {m: true}` ⇒ `parse_full.ok=true`, `r := "a" -> {m: "x"}` ⇒ `true`; but `r := "a" -> {m: null}` and `r := "a" -> null` ⇒ `parse_full.ok=false`. Whole-grammar `regex.ebnf` ⇒ `parse_full.ok=false error_position=4533` at `quant_base = "*" -> {min: 0, max: null}` (`regex.ebnf:84`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `ebnf_dual_run_diff` (`error_position=4533`; `{m:true}`/`{m:"x"}` PASS vs `{m:null}` FAIL) + grammar read + `ast_pipeline … --lint-grammar`: `grammars/ebnf.ebnf:330` `literal_return := ( quoted_string | numeric_literal | boolean_literal )` has NO `null` alternative, so the JSON-`null` value (a valid object/array value) is unparsable. WHERE = `literal_return` (`grammars/ebnf.ebnf:330`). Reference: `grammars/return_annotation.ebnf:150` `null_literal := 'null'` (JSON's sixth value type) is the authoritative model.
  - [x] **FIX** — tier = grammar ([[project_ebnf_is_single_source_of_truth]]). Added `null_literal := "null" -> {type:"null"}` and appended it to `literal_return`'s ordered choice. Additive + non-shadowing (`null` shares no prefix with `quoted_string` / `numeric_literal` / `boolean_literal`), so ordering is irrelevant to correctness. Mirrors `return_annotation.ebnf`.
  - [x] **ADDRESSED (verified)** — post regen (`make -C rust ebnf_frontend_dual_run_diff` re-derives the parser) + rebuild, the `null` probes `parse_full.ok` false→**true**: `-> {m: null}`, `-> null`; controls `-> {m: true}`, `-> {m: "x"}` still `true`. Whole-grammar `regex.ebnf` `parse_end` advances **4533 → 78429** = **100%** (`parse_end 78429/78429`), so regex.ebnf FULLY self-parses (it was the lone failing flow). **The strict gate `make -C rust ebnf_frontend_dual_run_gate` now exits 0** ("✅ EBNF dual-run differential passed for all tracked grammars").
  - [x] **NO REGRESSION** — `make -C rust ebnf_frontend_dual_run_gate` (STRICT, exit 0): `ebnf` `rust_parse_full=pass` (`22648/22649`, raw_ast **parity 127/127** = 126 prior + `null_literal`), `json` `pass` (99.90%, parity 19/19), `regex` `pass` (100%, `overall=pass`). `ast_pipeline grammars/ebnf.ebnf --lint-grammar` 127 rules: `non_terminating=0`, `ordered_choice_shadowing=0`, `unreachable_rules=0`, `unbound_fact_kinds=0`, `profile_orphans=0` (5 pre-existing return-expression `left_recursive` cycles unchanged — `null_literal` is a simple terminal, no LR). NO tracked Rust source amended → clippy strict-source unchanged. NO production parser regenerated → SV cert `UNKNOWN=28 spf=0` byte-identical seeds 0/7/42; the 6 fully-certified grammars byte-identical.
  - [x] **LOCKSTEP** — `[[feedback_ebnf_meta_grammar_lockstep]]`; CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md updated at commit; LIVE_ACHIEVEMENT_STATUS.md Tracker note added (the gate going strict-GREEN is the milestone the prior `.1`–`.4` leaves deferred "until GREEN"). NO book change (the `null` literal is an existing documented return-annotation value type — `docs/book/src/annotation-system.md`; meta-grammar-internal self-hosting catch-up). A book note documenting the EBNF self-hosting / dual-run milestone is a recommended PURE-DOCS follow-up (paired with `H.14`'s honest 8/12 status). NO contract/ledger (meta-grammar internal).

- `2026-06-24` (`H.13.4` EBNF meta-grammar lockstep — port **dotted property access on a reference** (`$1.min`) to `ebnf.ebnf`, `-0125`, **GRAMMAR**, meta-grammar only — `generated/ebnf.rs` regenerated/untracked, NO production-parser regen, NO release/schema/ledger change): **the original `-0068` gap #5 (dotted `$refs`) is closed — `grammars/ebnf.ebnf`'s `scalar_return` now admits a `$N.field` / `$name.field` postfix chain, so regex's `quantifier := … -> {type:"quantifier", min: $1.min, max: $1.max, greediness: $2}` (and the function-call args `generate_range_check($1.start, $1.end)`) now parse under the generated ebnf parser.** Tools-first isolation (`ebnf_dual_run_diff` minimal probes): `-> {m: $1}` (bare positional) PASS, but `-> {m: $1.min}` / `-> $1.min` / `-> {min: $1.min, max: $2.max}` all FAIL ⇒ the missing construct is precisely a dotted `.field` postfix on a reference (NOT the bare ref, which already parsed). WHY+WHERE: `scalar_return` (`grammars/ebnf.ebnf:317`) commits PEG-ordered to the bare `positional_reference` for `$1` and strands the dangling `.min`; the only dotted rule `member_access_return` (`:407`) is BOTH left-recursive AND ordered AFTER `scalar_return` inside `return_expression` (`:308-314`), so it is never reached for a reference base. The fix mirrors `grammars/return_annotation.ebnf`'s `property_access_expression` / `accessor_base`, re-expressed in non-left-recursive `reference_base property_access_suffix property_access_suffix*` form and placed FIRST in `scalar_return` so the `.field` chain binds tighter than a bare ref. Honest framing: gap-by-gap until the dual-run gate goes green — after this, regex.ebnf advances `3960 → 4533`, now failing at the `null` object-value literal (`quant_base`, `regex.ebnf:84` `{min: 0, max: null}`; `ebnf.ebnf`'s `literal_return` has no `null`), the next sub-leaf (`H.13.5`). SV stays `UNKNOWN=28`. Detail in the frontier `H.13`/`H.13.4` rows. **Commit: `PGEN-GRAMMAR-WELLFORMED-0125`.**

  ### Acceptance Checklist (enforced) — H.13.4
  - [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff --input` minimal-probe matrix (pre-fix): `r := "a" -> {m: $1}` ⇒ `parse_full.ok=true`; but `r := "a" -> {m: $1.min}`, `r := "a" -> $1.min`, and `r := "a" "b" -> {min: $1.min, max: $2.max}` all ⇒ `parse_full.ok=false`. Whole-grammar `regex.ebnf` ⇒ `parse_full.ok=false error_position=3960` at the `quantifier` rule's return `-> {type:"quantifier", min: $1.min, max: $1.max, greediness: $2}` (`regex.ebnf:75-76`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `ebnf_dual_run_diff` (`error_position=3960`; minimal `{m:$1}` PASS vs `{m:$1.min}` FAIL) + grammar read + `ast_pipeline … --lint-grammar`: `grammars/ebnf.ebnf:316-321` `scalar_return := ( positional_reference | named_reference | literal_return )` has NO postfix `.field` form, so PEG ordered choice commits to the bare `positional_reference` for `$1` and strands the `.min`. The only dotted rule `member_access_return := return_expression "." identifier_literal` (`:407`) is BOTH left-recursive (cycle via `return_expression`) AND ordered AFTER `scalar_return` inside `return_expression` (`:308-314`), so it is never reached for a reference base. WHERE = `scalar_return` / `return_expression` ordered choice (`grammars/ebnf.ebnf:308-321`). Reference: `grammars/return_annotation.ebnf`'s `property_access_expression := accessor_base '.' identifier` + `accessor_base := positional_reference | …` is the authoritative model the meta-grammar must mirror.
  - [x] **FIX** — tier = grammar ([[project_ebnf_is_single_source_of_truth]]). Added `reference_property_access := reference_base property_access_suffix property_access_suffix*` (+ helpers `reference_base := (positional_reference | named_reference)`, `property_access_suffix := "." identifier_literal`) and placed it FIRST in `scalar_return`'s ordered choice. Non-left-recursive base+suffix* (the proven `[$first, $rest*]` idiom) — a bare `$N` with no suffix fails the required-first suffix and falls through to `positional_reference`, so existing scalar returns are byte-unchanged. Targeted to the proven gap #5 (dotted); indexed `$N[i]` (gap #6) deferred — no dual-run grammar uses it (the `H.13.3` "targeted to the proven gap" discipline).
  - [x] **ADDRESSED (verified)** — post regen (`make -C rust ebnf_frontend_dual_run_diff` re-derives the parser) + rebuild, the dotted probes all `parse_full.ok` false→**true**: `-> {m: $1.min}`, `-> $1.min`, `-> {min: $1.min, max: $2.max}`, the regex-faithful `-> {type:"quantifier", min: $1.min, max: $1.max, g: $2}`, the function-call args `-> f($1.start, $1.end)`, named `-> $start.value`, and the arbitrary-depth chain `-> $1.a.b.c`; controls `-> {m: $1}`, `-> $1`, `-> [$1**]`, `-> [$1, $2*]` still `true`. Whole-grammar `regex.ebnf` dual-run `parse_end` advances **3960 → 4533** (consumed 573 more bytes), now failing at a DIFFERENT deeper gap (the `null` object-value literal at `quant_base`, `regex.ebnf:84`) — NOT a regression.
  - [x] **NO REGRESSION** — `make -C rust ebnf_frontend_dual_run_diff` report gate: `ebnf` `rust_parse_full=pass` (`parse_end 22261/22262`, raw_ast **parity 126/126** — the 123 prior rules + the 3 new ones), `json` `rust_parse_full=pass` (`1013/1014`=99.90%, raw_ast **parity 19/19**) — both stable; regex the lone failing flow (advanced, not new). `ast_pipeline grammars/ebnf.ebnf --lint-grammar` 126 rules: `non_terminating=0`, `ordered_choice_shadowing=0`, `unreachable_rules=0`, `unbound_fact_kinds=0`, `profile_orphans=0`, `always_matches_shadowing=0` (the 5 pre-existing return-expression `left_recursive` cycles are unchanged — the 3 new rules are non-left-recursive). NO tracked Rust source amended → clippy strict-source unchanged. NO production parser regenerated → SV cert `UNKNOWN=28 spf=0` byte-identical seeds 0/7/42; the 6 fully-certified grammars byte-identical.
  - [x] **LOCKSTEP** — `[[feedback_ebnf_meta_grammar_lockstep]]`; CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md to update at commit. NO book change (dotted property access `$a.b.c[i]` is an existing documented return-annotation feature — `docs/book/src/annotation-system.md`; this is meta-grammar-internal self-hosting catch-up). NO LIVE-status row change (the dual-run gate stays red until all gaps close). NO contract/ledger (meta-grammar internal).

- `2026-06-24` (`H.13.3` EBNF meta-grammar lockstep — allow an **object literal as an array element** in `ebnf.ebnf`'s `array_element_return`, `-0124`, **GRAMMAR**, meta-grammar only — `generated/ebnf.rs` regenerated/untracked, NO production-parser regen, NO release/schema/ledger change): **a meta-grammar gap BEYOND the original `-0068` six-enumeration, surfaced by advancing regex.ebnf past the per-branch gap (`.2`): `grammars/ebnf.ebnf`'s `array_element_return` accepted scalar/positional/quantified/flatten/spread elements but NOT an object literal, so regex's `piece_quoted_run_quantified -> [$2**, {type:"piece", atom:$3, quantifier:$5}]` failed at the `{`.** Tools-first isolation (`ebnf_dual_run_diff` minimal probes): `-> {…}` (top-level object) PASS, `-> [$2**]` PASS, but `-> [{…}]` / `-> [$2, {…}]` / `-> [$2**, {…}]` all FAIL@9 (the `{`) ⇒ the missing element type is precisely `object_return`, NOT a "mixed-spread" issue (the [[feedback_annotation_no_mixed_spread]] note is about the annotation *engine*, a different layer; here it is the meta-grammar's *parse* of the array). The fix mirrors `grammars/return_annotation.ebnf`, whose `array_literal` admits a full `expression` (incl. `object_literal`). Honest framing: the `-0068` whole-file probes were first-failure-masked, so the lockstep is gap-by-gap until the dual-run gate goes green — this is the next such gap. SV stays `UNKNOWN=28`. Detail in the frontier `H.13`/`H.13.3` rows. **Commit: `PGEN-GRAMMAR-WELLFORMED-0124`.**

  ### Acceptance Checklist (enforced) — H.13.3
  - [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff --input` minimal-probe matrix: `r := "a" -> {type:"x"}` ⇒ `parse_full.ok=true`; `r := "a" -> [$2**]` ⇒ `true`; but `r := "a" -> [{type:"x"}]`, `r := "a" -> [$2, {type:"x"}]`, and the regex form `r := "a" -> [$2**, {type:"piece", atom:$3, quantifier:$5}]` all ⇒ `parse_full.ok=false error_position=9` (the `{`). Whole-grammar `regex.ebnf` ⇒ `parse_full.ok=false error_position=2414` at `piece_quoted_run_quantified`'s return `-> [$2**, {…}]` (`regex.ebnf:47`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `ebnf_dual_run_diff` (object element FAIL@9; top-level object + `[$2**]` both PASS) localizes it precisely: `grammars/ebnf.ebnf:336` `array_element_return := ( flatten_reference | quantified_reference | scalar_return | spread_reference )` has NO `object_return` alternative, so an object literal `{…}` inside an array is unparsable even though `object_return` is a valid top-level `return_expression` (`:308-311`). WHERE = `array_element_return` (`grammars/ebnf.ebnf:336`). Reference: `grammars/return_annotation.ebnf`'s `array_literal` admits a full `expression` (incl. `object_literal`), so the meta-grammar is strictly more restrictive than the language it must model.
  - [x] **FIX** — tier = grammar ([[project_ebnf_is_single_source_of_truth]]). Add `object_return` to `array_element_return`'s ordered choice. Additive; `{`-prefixed `object_return` shares no prefix with the existing reference/scalar/spread forms ⇒ non-shadowing. Targeted to the proven gap (object element); nested `array_return` is a separate, currently-unused construct left for a future leaf only if a grammar needs it.
  - [x] **ADDRESSED (verified)** — post regen+rebuild, the object-element probes all `parse_full.ok` false→**true**: `-> [{type:"x"}]`, `-> [$2, {type:"x"}]`, and the regex form `-> [$2**, {type:"piece", atom:$3, quantifier:$5}]`; controls `-> [$2**]`, `-> {type:"x"}`, `-> [$1, $2*]` still `true`. Whole-grammar `regex.ebnf` dual-run `parse_end` advances **2413 → 3959** (consumed 1546 more bytes), now failing at a DIFFERENT deeper gap (NOT a regression).
  - [x] **NO REGRESSION** — `make -C rust ebnf_frontend_dual_run_diff` report gate: `ebnf` `rust_parse_full=pass` (`parse_end 20969/20970`, raw_ast **parity** 123/123), `json` `rust_parse_full=pass` (`1013/1014`=99.90%, raw_ast **parity** 19/19) — both stable; regex the lone failing flow (advanced). `ast_pipeline grammars/ebnf.ebnf --lint-grammar` 123 rules: `non_terminating=0`, `ordered_choice_shadowing=0`, `unreachable_rules=0`, `unbound_fact_kinds=0`, `profile_orphans=0` (5 pre-existing return-expression `left_recursive` cycles, not `array_element_return`). NO tracked Rust source amended → clippy strict-source unchanged. NO production parser regenerated → SV cert `UNKNOWN=28` unaffected; 6 fully-certified grammars byte-identical.
  - [x] **LOCKSTEP** — `[[feedback_ebnf_meta_grammar_lockstep]]`; CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md to update at commit. NO book change (object-literal array elements are an existing documented return-annotation feature; meta-grammar-internal self-hosting catch-up). NO LIVE-status row change (dual-run gate stays red until all gaps close). NO contract/ledger (meta-grammar internal).

- `2026-06-24` (`H.13.2` EBNF meta-grammar lockstep — port **per-branch return annotations** to `ebnf.ebnf`, `-0123`, **GRAMMAR**, meta-grammar only — `generated/ebnf.rs` regenerated/untracked, NO production-parser regen, NO release/schema/ledger change): **the SECOND and DOMINANT of the six `-0068` meta-grammar gaps is closed — `grammars/ebnf.ebnf` now models a return annotation on each non-last alternation branch (a `->` before a `|`), so the generated ebnf parser parses regex's `piece = piece_quoted_run_quantified -> $1 | atom quantifier? -> {type:"piece", atom:$1, quantifier:$2}`.** Owns the EBNF-meta-grammar-lockstep doctrine ([[feedback_ebnf_meta_grammar_lockstep]]). Tools-first: gap + fix diagnosed/verified entirely through `ebnf_dual_run_diff` minimal probes + the `ebnf_frontend_dual_run_diff` report gate; this is the EBNF lockstep lane, NOT the regex parser (regex.ebnf is only the test input). Decisive gate-mechanics finding: the dual-run **raw_ast parity** column compares the Perl vs Rust *frontends* (`--emit-raw-ast-json` ⇒ `ebnf_frontend.rs`), so a `grammars/ebnf.ebnf`-only edit (which regenerates `generated/ebnf.rs`) can affect ONLY the `rust_parse_full` column — and since `ebnf`/`json` contain no per-branch annotation (proven: they pass while the feature is unsupported), an additive change leaves their `parse_full` green by construction. SV stays `UNKNOWN=28` — unaffected by this meta-grammar-only change; the SV `UNKNOWN`→0 drive remains the headline lane. Detail in the frontier `H.13`/`H.13.2` rows. **Commit: `PGEN-GRAMMAR-WELLFORMED-0123`.**

  ### Acceptance Checklist (enforced) — H.13.2
  - [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff --input` on the minimal probe `r := "a" -> $1 | "b" -> $2` ⇒ `parse_full.ok=false error_position=15` (the generated ebnf parser's rule-level `return_annotation?` eats `-> $1`, stranding `| "b" -> $2` at the `|`); the single-trailing control `r := "a" -> $1` already PASSES (`parse_full.ok=true`); whole-grammar `regex.ebnf` ⇒ `parse_full.ok=false error_position=1726` (the `piece` rule's per-branch annotation, gap #1 — the next gap after `H.13.1`'s `**`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `ebnf_dual_run_diff` (`error_position=15` at the `|`) + grammar read: `grammars/ebnf.ebnf:102` `alternation := sequence ("|" sequence)*` models NO per-branch return annotation; the only `return_annotation?` is rule-level (`rule_definition`, `:78`, AFTER the whole `rule_expression`), so a `->` appearing BEFORE a `|` is consumed by the rule-level slot and the rest of the alternation is stranded. WHERE = `grammars/ebnf.ebnf` `alternation` (`:102`). The authoritative Rust frontend `rust/src/ebnf_frontend.rs:611-638` (`tokenize_rule_expression`) treats a top-level `->` as an inline per-branch annotation whose payload extends to the next top-level `|` or rule end — the behavior the meta-grammar must mirror.
  - [x] **FIX** — tier = grammar (the EBNF is the single source of truth, [[project_ebnf_is_single_source_of_truth]]). `grammars/ebnf.ebnf:102` `alternation := sequence ("|" sequence)*` → `alternation := sequence (return_annotation? "|" sequence)*`. Additive: a `return_annotation?` is now consumed between a branch's `sequence` and the following `|`; the trailing/last-branch annotation is still consumed by `rule_definition`'s rule-level `return_annotation?` (so single-branch and last-branch trailing annotations are byte-identical). Mirrors the Rust frontend's inline-at-`->` semantics.
  - [x] **ADDRESSED (verified)** — minimal `r := "a" -> $1 | "b" -> $2` probe `parse_full.ok` false→**true** (`ebnf_dual_run_diff`, post regen+rebuild against the freshly-generated parser); the 3-branch `r := "a" -> $1 | "b" -> $2 | "c"` (last branch no annotation) also `true`; the single-trailing control `r := "a" -> $1` still `true`. Whole-grammar `regex.ebnf` dual-run `parse_end` advances **1726 → 2413** (consumed 687 more bytes), now failing at a DIFFERENT deeper gap (NOT a regression — the per-branch `piece` rule is consumed).
  - [x] **NO REGRESSION** — `make -C rust ebnf_frontend_dual_run_diff` report gate: `ebnf` `rust_parse_full=pass` (`parse_end 20949/20950`, raw_ast **parity** 123/123), `json` `rust_parse_full=pass` (`1013/1014` = 99.90%, raw_ast **parity** 19/19) — both byte-stable vs `-0122`; regex remains the lone failing flow (advanced, not new). `ast_pipeline grammars/ebnf.ebnf --lint-grammar` 123 rules: `non_terminating=0`, `ordered_choice_shadowing=0`, `unreachable_rules=0`, `unbound_fact_kinds=0`, `profile_orphans=0` (the 5 `left_recursive` are pre-existing return-expression cycles, NOT `alternation`). NO tracked Rust source amended (`git diff` shows only `grammars/ebnf.ebnf` + this task doc) → clippy strict-source result unchanged from `-0122` (clean). NO production parser regenerated (`generated/systemverilog_parser.rs` etc. untouched) → SV cert `UNKNOWN=28` unaffected; the 6 fully-certified grammars' parsers byte-identical (not regenerated).
  - [x] **LOCKSTEP** — `[[feedback_ebnf_meta_grammar_lockstep]]`; CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md to update at commit. NO book change — per-branch return annotations are already documented (`docs/book/src/annotation-system.md` "Parens-grouped Or with trailing annotation" + per-branch placement); this is a meta-grammar-internal self-hosting catch-up (no production-parser behavior change). NO LIVE-status row change (the dual-run gate stays red until all 6 gaps close). NO contract/ledger (meta-grammar internal).

- `2026-06-24` (`H.13.1` EBNF meta-grammar lockstep — port the `**` flatten-spread return-marker to `ebnf.ebnf`, `-0122`, **GRAMMAR**, meta-grammar only — `generated/ebnf.rs` regenerated/untracked, NO production-parser regen, NO release/schema/ledger change): **the FIRST of the six `-0068` meta-grammar gaps is closed — `grammars/ebnf.ebnf` now models the `**` flatten-spread return-marker, so the generated ebnf parser parses regex's `concatenation := piece+ -> [$1**]`.** Owns the EBNF-meta-grammar-lockstep doctrine ([[feedback_ebnf_meta_grammar_lockstep]]); the `**` gap is the original `-0067` / `H.11.5(3)` red-gate trigger ("the generated ebnf parser cannot parse the `**` flatten-spread … needs its own leaf (ebnf.ebnf grammar fix)"). Tools-first: the gap and fix are diagnosed and verified entirely through `ebnf_dual_run_diff` minimal probes + the dual-run gate; this is the EBNF lockstep lane, NOT the regex parser (regex.ebnf is only the test input). SV stays `UNKNOWN=28` (the only non-fully-certified shipped grammar) — unaffected by this meta-grammar-only change; the SV `UNKNOWN`→0 drive (SVA cascade `H.12.5.8.3`, the deferred generator capabilities, the `no_path` multi-profile accounting) remains the headline lane. Detail in the frontier `H.13`/`H.13.1` rows. **Commit: `PGEN-GRAMMAR-WELLFORMED-0122`.**

  ### Acceptance Checklist (enforced) — H.13.1
  - [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff --input` on the minimal probe `r := "a" -> [$1**]` ⇒ `parse_full.ok=false error_position=9` (the generated ebnf parser backtracks the whole return annotation, leaving `->` unconsumed); whole-grammar `regex.ebnf` ⇒ `parse_full.ok=false error_position=1307` at the `concatenation := piece+ -> [$1**]` rule. The single-star control `[$1*]` already PASSES.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `ebnf_dual_run_diff` (`error_position=1307` at `[$1**]`) + `ast_pipeline … --lint-grammar` (well-formed, no new LR/shadowing): `grammars/ebnf.ebnf`'s `array_element_return` models `quantified_reference := positional_reference quantified_marker` with `quantified_marker := ("*"|"+"|"?")` — single markers ONLY; there is NO `**` flatten-spread alternative, so `$1**` matches `$1` + `*` and strands the second `*`, failing the array close. WHERE = `grammars/ebnf.ebnf` `array_element_return` (no `flatten_reference` rule). The authoritative annotation grammar `grammars/return_annotation.ebnf` models it as `flat_spread_expression := spreadable_expression '**'`, listed BEFORE `*` (PEG ordered choice).
  - [x] **FIX** — tier = grammar (the EBNF is the single source of truth, [[project_ebnf_is_single_source_of_truth]]). Added `flatten_reference := positional_reference "**" -> {type:"flatten_reference", reference:$1}` to `grammars/ebnf.ebnf` and placed it FIRST in `array_element_return`'s ordered choice so `**` is tried before `*`. Additive; mirrors `return_annotation.ebnf`.
  - [x] **ADDRESSED (verified)** — minimal `[$1**]` probe `parse_full.ok` false→**true** (`ebnf_dual_run_diff`, post-regen+rebuild); `regex.ebnf` dual-run `error_position` advances **1307 → 1726** (consumed 419 more bytes; IDENTICAL for the Rust-frontend `generated/ebnf.rs` AND the Perl-frontend bootstrap parser ⇒ frontend lockstep), now failing at a DIFFERENT known gap — the `piece` rule's per-branch return annotation (gap #1), NOT a regression; `[$1*]`/`[$1, $2*]` still PASS (ordered-choice safe); `ebnf`/`json` self-parse still parse_full PASS.
  - [x] **NO REGRESSION** — `ebnf_frontend_dual_run_diff` report gate: `ebnf` PASS (100%, raw_ast parity), `json` PASS (99.90%, raw_ast parity), regex remains the lone failing flow (advanced, not new); SV cert `UNKNOWN=28 spf=0 proof_reverify=0` byte-identical seeds 0/7/42 (no production parser regenerated — `generated/systemverilog_parser.rs` untouched); the 6 fully-certified grammars' parsers byte-identical (not regenerated); `--lint-grammar` ebnf.ebnf `123 rules` all-errors-0 (no new LR/shadowing); clippy strict-source clean (generated stage = pre-existing tolerated debt).
  - [x] **LOCKSTEP** — `[[feedback_ebnf_meta_grammar_lockstep]]`; CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md. NO book change — the `**` annotation feature is already documented (`docs/book/src/annotation-system.md`) and its user-facing behavior is unchanged; this is a meta-grammar-internal self-hosting catch-up (no production-parser behavior change). NO LIVE-status row change (the dual-run gate stays red until all 6 gaps close). NO contract/ledger (meta-grammar internal).

- `2026-06-21` (`H.12.5.8.2` SVA infix binary-operator parse bug FIX DESIGN/DECISION, `-0119`, **PURE-DOCS DESIGN**): **DECISION = direction A (grammar restructure to a §16 precedence cascade); option B (engine direct-LR elimination) is DOMINATED.** Deciding fact (tools-first, IEEE 1800-2017 **Table 16-3** §16.12 extracted from the vendored LRM): the current `sequence_expr`/`property_expr` is a FLAT precedence-free operator list, but SVA operators have real precedence/associativity (`[*]`≻`##`≻`throughout`≻`within`≻`intersect`≻`{not,nexttime}`≻`and`≻`or`≻`iff`≻`until`-family≻implication≻`{always,eventually}`≻guards; `until`/`iff`/implication/`throughout` are right-assoc, the rest left), so a precedence cascade is REQUIRED either way — a naive direct-LR `β (op β)*` would flatten all operators to one precedence + left-assoc (wrong). Therefore B = the same grammar restructure PLUS a new high-blast-radius engine feature, for identical correctness ⇒ A wins (level-1 declarative fix per [[feedback_no_workarounds_fix_hierarchy]], proven `next (OP next)*` idiom per [[feedback_quantified_group_extraction]], one grammar file + SV regen only, correctness-first per [[feedback_correctness_before_speed]], "use what we have" per [[project_vision_and_discipline]]). Designed the cascade blueprint (sequence layer: `seq_or → seq_and → seq_intersect → seq_within → seq_throughout → seq_delay(##) → seq_unary`; property layer: `prop_guard → prop_temporal → prop_impl → prop_until → prop_iff → prop_or → prop_and → prop_not → prop_primary`; both profiles) + `.8.3` implementation sub-questions (AST-shape preservation vs schema bump; Annex A.2.10 operand types; book reconciliation of the overclaimed direct-LR auto-elim). Recorded as SEPARATE future general work (NOT this lane): completing the engine direct-LR eliminator (would also fix the sibling un-eliminated `block_event_expression@:681`). `H.12.5.8` SPLIT continues → `.8.2` done + **`.8.3`** IMPLEMENT (new frontier, LARGE/signoff-critical, best with fresh context). PURE-DOCS DESIGN ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; no gate run; SV stays `UNKNOWN=56`; live status UNCHANGED. Detail: [GRAMMAR-WELLFORMED-H12582-infix-binop-lr-fix-design.md](GRAMMAR-WELLFORMED-H12582-infix-binop-lr-fix-design.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0119`.** [[feedback_research_grounded_sota_no_trial_and_revert]], [[feedback_why_and_where_before_solution]], [[feedback_no_workarounds_fix_hierarchy]], [[project_ebnf_is_single_source_of_truth]], [[feedback_uvm_is_valid_sv]], [[feedback_regex_book_live]].
- `2026-06-21` (`H.12.5.8.1` SVA infix property/sequence binary-operator parse bug WHY+WHERE, `-0118`, **PURE-DOCS INVESTIGATION**): **root-caused the whole infix property/sequence reject class (`a ##1 b`/`and`/`or`/`intersect`/`within`/`until`/`s_until` — all FAIL at the operator `furthest≈40/42`; bare `a` PASS) to UN-ELIMINATED left recursion.** Tools-first (`parseability_probe --profile 2017` + `--trace-rules sequence_expr`): generated `systemverilog_parser.rs` has ZERO `_lr_base`/`_lr_suffix` ⇒ `sequence_expr` (direct, self-binary `A := A op A`) and `property_expr_sv_2017`/`property_expr` (indirect via the union wrapper) are never LR-eliminated, so the runtime cycle-breaker (`mutual_recursion_handler.rs:119` `CycleType::LeftRecursive => false`) blocks the infix branches (`💥 Infinite recursion detected`, trace-confirmed at the operator position) and only the first operand parses. ROOT site: `eliminate_left_recursive_patterns`→`detect_left_recursive_chain_plan` (`rust/src/ast_pipeline/mod.rs:1692`) recognizes ONLY the indirect bare-reference wrapper-chain shape (`extract_rule_reference_name`@2127 + `extract_wrapper_suffix`@2170); **direct inline LR `A := Aα|β` is unhandled by construction** — decisively proven on minimal `expr := expr plus term | term` (`0 transformations` at `PGEN_TRACE_VERBOSITY=debug`; generated parser rejects `1+2`). BOOK finding: `developer-architecture.md` overclaims direct-LR auto-elimination (book↔code drift, reconcile in `.8.2`). `H.12.5.8` SPLIT → `.8.1` done + `.8.2` fix-design (new lane-2 frontier: (A) grammar `next (OP next)*` idiom vs (B) engine direct-LR elimination; `.8.3` implement). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; no gate run; SV stays `UNKNOWN=56`, the only non-fully-certified shipped grammar; live status UNCHANGED. Detail: [GRAMMAR-WELLFORMED-H12581-infix-binop-lr-whywhere.md](GRAMMAR-WELLFORMED-H12581-infix-binop-lr-whywhere.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0118`.** [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[project_grammar_wellformedness_contract]], [[feedback_uvm_is_valid_sv]], [[feedback_regex_book_live]].
- `2026-06-21` (`H.12.5.7.2` M3 prefix-operator plannable-reach extension, `-0117`, **GENERATOR-ONLY ENGINE FIX**): **the `.7.1`-adjudicated prefix temporal-operator generator-reach gap is FIXED — SV cert `UNKNOWN 67→56` — by generalising the RTL-FE-CLOSURE.5.6 recursion-suppression from DIRECT to INDIRECT recursion.** WHY+WHERE CONFIRMED tools-first (correcting `.7.1`'s "LR-eliminated" hypothesis): the reach plan IS found and targets the right branch (`PGEN_REACH_PATH_DUMP=1`, e.g. `kw_eventually → property_expr_sv_2017 root/o21/s0`), and `property_expr_sv_2017` is **NOT** LR-eliminated (original `o0..o33` indices). ROOT CAUSE (`stimuli_generator.rs:7104-7118`): `suppress_recursive_forced_branch` scoped re-fire suppression to **direct** self-recursion (`refs.contains(current_rule)`); the prefix branch `kw_eventually (constant_range)? property_expr` references `property_expr` (the one-hop wrapper `property_expr := property_expr_sv_2017`), so the directive re-fired on every **indirect** re-entry → forced `eventually eventually …` → depth/`max_rule_visits` exhaustion → generation `Err` (the 29 `generation_failures`). FIX (GENERAL/parser-agnostic, 4 edits): new grammar-scoped `RULE_REACH_CACHE` thread-local + cleared with `NULLABLE_CACHE` on grammar change; new memoised `rule_can_reach(from,to)` BFS over the rule-reference graph; the suppress test broadened to `refs.contains(current_rule) || refs.iter().any(|r| self.rule_can_reach(r, current_rule))`, gated behind the cheap `call_stack.count(current_rule) >= 2` re-entry check (so off-reach + off-recursion are byte-identical). VERIFIED (decisive stash A/B, `--manifest-path` rebuilds, parser is the judge): SV `UNKNOWN 67→56` / `witness 1221→1232` / `generation_failures 29→0` / `total=1289` / `spf=0` **deterministic seeds 0/7/42**, **ZERO newly-UNKNOWN** (56 ⊂ 67); pre-fix re-confirmed `67` (stash validity). 11 resolved = the prefix cluster (`accept_on`/`eventually`/`nexttime`/`reject_on`/`s_always`/`s_eventually`/`s_nexttime`/`sync_accept_on`/`sync_reject_on`/`property_case_item`) + a `constant_cast` bonus; the infix `until`-family + `intersect`/`within` now GENERATE but the parser rejects ⇒ stay UNKNOWN (no false witness; → `H.12.5.8`); `kw_constant` stays UNKNOWN (optional sub-branch, deferred). The 6 fully-certified grammars intact (json 9/9, regex 198/198, svpp 74/74, vhdl 216/216, rtlfe 169, rtlce 48/48 — verdict matches baseline); **`--generate-stimuli` byte-identical** (md5 `7eb4374b…` pre==post); `clippy_on_rust_change` source-clean. GENERATOR-ONLY ⇒ NO grammar/parser-regen/release/schema/inventory/manifest/ledger change (the `-0072`/`-0115` precedent); SV row stays `Mostly Done` (closure-debt retirement). `H.12.5.7` now `done` (both children resolved). Detail: [GRAMMAR-WELLFORMED-H12572-prefix-temporal-reach-extension.md](GRAMMAR-WELLFORMED-H12572-prefix-temporal-reach-extension.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0117`.** [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_features_parser_agnostic_enable_all_parsers]], [[feedback_prove_independence_with_decisive_baseline]], [[project_cert_coverage_tournament_loser_leak]].
- `2026-06-21` (`H.12.5.7.1` M3 temporal-operator WHY+WHERE + parse/reject adjudication, `-0116`, **PURE-DOCS INVESTIGATION**): **the M3 property/sequence temporal-operator cluster in the SV `UNKNOWN=67` residual splits into TWO attribution-rule causes — and the split re-scopes `H.12.5.7` + broadens `H.12.5.8`.** Tools-first (DEBUG `ast_pipeline`, `PGEN_CERT_COVERAGE_DUMP_ALL=1`/`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, count 40 seed 0; `parseability_probe` per-operator minimal samples): (1) the **prefix** operators (`accept_on`/`eventually`/`nexttime`/`reject_on`/`s_always`/`s_eventually`/`s_nexttime`/`sync_accept_on`/`sync_reject_on`/`kw_constant`/`property_case_item`) **PARSE** (`s_eventually a`, `nexttime a`, `accept_on (a) b`, `case (x) 1: a; endcase`, the no-range `eventually a`/`s_always a` all accepted) ⇒ genuine generator-reach gap — yet the plannable-witness pass emits **no probe** for them and they are **not** in the 19 `no_path`, so by `run_plannable_witness_pass`'s own accounting (`stimuli_generator.rs:3196-3205`: `rule_attempted=false`⇒`no_path`; reach-found-but-all-`Err`⇒`generation_failures`, no `witness_check` print) they are in the dump's **"29 generation failures"** — the construct can't be built descending the deep **indirectly-left-recursive** `property_expr→property_expr_sv_2017→property_expr` chain (LR-eliminated `_lr_base`/`_lr_suffix`). (2) the **infix** `until`-family (`until`/`until_with`/`s_until`/`s_until_with`) **REJECTS** at the operator (`furthest=47`, the SAME locus as `a or b`/`a and b`/`a ##1 b`); sequence `intersect`/`within` are probed but `parsed=false` — all the LR-elim parse defect, NOT reach gaps. RESOLUTION (attribution rule — check the parser BEFORE adding generator machinery): `H.12.5.7` SPLIT → `.7.1` (this, done) + `.7.2` (prefix-operator plannable-reach extension, the new lane-1 frontier, model on `rtl_const_expr` constructive-reach + RTL-FE-CLOSURE.5.x); the infix `until`-family folded into **`H.12.5.8`** (broadened to own the whole infix property/sequence class). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV stays `UNKNOWN=67`, the only non-fully-certified shipped grammar; live status UNCHANGED. Detail: [GRAMMAR-WELLFORMED-H12571-m3-temporal-whywhere.md](GRAMMAR-WELLFORMED-H12571-m3-temporal-whywhere.md). **Commit: `PGEN-GRAMMAR-WELLFORMED-0116`.** [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[project_grammar_wellformedness_contract]], [[feedback_uvm_is_valid_sv]].
- `2026-06-17` (`H.12.6.1` `module_path_conditional_expression` producer left-recursion fix, `-0111`, **GRAMMAR FIX**, release 1.0.143 / schema stays 4 / ledger `SV-0005`): **the ONE genuine `no_path` producer-wiring suspect from the `H.12.6` re-audit is FIXED — SV cert `UNKNOWN 86→84`, and the LAST `no_path` budget case is RETIRED (`unreachable_rules=0`, the linter's "no unreachable rules" now literally true for SV).** WHY+WHERE (tools-first, decisive): mpce (IEEE 1800 A.8.3) was stranded by an indirect LEFT-RECURSION — its condition referenced `module_path_expression`, whose first branch was mpce. The LR-eliminator (proven via `--dump-gen-ast` reference graph + the generated `parse_module_path_conditional_expression` defined-but-never-called) rewrote the producer into `module_path_expression_lr_base`/`_lr_suffix` and left mpce an UNREFERENCED rewritten seed, while the `_lr_suffix` reconstruction LEAKED raw `wrapper_specs` metadata into the ternary module-path AST (genuineness oracle on `module m; specify if (a ? b : c) (x => y) = 1; endspecify endmodule` → 0 `conditional` / 3 `wrapper_specs`). The grammar's "RecursionGuard handles it / `max_unreachable_rules<=1` budget case" comment was DISPROVEN. FIX (surgical, grammar-only, changed ONLY mpce): its condition is now the non-left-recursive `module_path_expression_operand ( binary_module_path_operator attribute_instance* module_path_expression_operand )*` chain (`-> {condition:{kind:"chain",first:$1,rest:$2}, attributes:$4, then_expr:$5, else_expr:$7}`) — same accepted language (the lower-precedence binary chain as condition; `then`/`else` stay full mpe ⇒ right-associative ternary, exactly A.8.3); `module_path_expression` is unchanged in source but now natively non-left-recursive (eliminator no longer fires; `_lr_base`/`_lr_suffix` vanish), mpce positively referenced + WITNESSED, the leak gone. The stale "budget case" comment block retired same-edit. VERIFIED (deterministic seeds 0/7/42, parser is the judge): cert `UNKNOWN 86→84` (mpce witnessed + `module_path_expression_lr_suffix` removed; witness `1204`; total `1291→1289`; spf=0, proof_reverify_failures=0); `no_path 20→19` (verbatim diff = exactly mpce leaves; the 19 LRM-legitimate rules stay); genuineness oracle 1 clean `conditional` / 0 `wrapper_specs`; targeted forms (`if (a)`/`if (a & b)`/`if (a ? b : c)`/`if (!a ? b & c : d)`/bare path) parse; **SV external corpus 14/14** (`parse_fail_total=0`); `stimuli_cross_family_platform_gate` PASS; syntax-closure gate PASS (contract **v6**: `max_unreachable_rules 1→0`, blessed `lrm_mutual_recursion_cases` emptied — `unreachable_rules=0 reachable_rules=1405`; the floors `min_total_rules 1403`/`min_reachable_rules 1404`/`max_unreachable_branches 2` unchanged); clippy source-clean (generated non-strict). Schema stays 4 (the prior `wrapper_specs`-leaking conditional was never a realized clean shape; common chain byte-identical — the `-0118`/`-0119`/`-0121` precedent). Lockstep: ledger `SV-0005`, contract 1.0.143, SV book changelog+schema pages, top-level `grammar-wellformedness.md`, syntax-closure contract v6, CHANGES/DEVELOPMENT_NOTES/MEMORY/LIVE_ACHIEVEMENT_STATUS. Detail: [GRAMMAR-WELLFORMED-H1261-module-path-conditional-fix.md](GRAMMAR-WELLFORMED-H1261-module-path-conditional-fix.md). The literal-0 doctrine: a `no_path` budget case RETIRED by fixing the producer, never by deletion ([[feedback_no_rule_deletion_without_lrm_proof]]). [[feedback_tools_first_no_guessing]], [[feedback_why_and_where_before_solution]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_grammar_edit_proof_gate_lockstep]], [[project_ebnf_is_single_source_of_truth]].
- `2026-06-17` (`H.12.5.5.3.3.4.2.1.2.2.2` context_member GENUINE-witness composition — IMPLEMENT ATTEMPT → two `-0107` assumptions REFUTED, `-0110`, PURE-DOCS): **the `-0107` safe `Presence`-prelude design was implemented faithfully, measured with the parser as the only judge, and its two load-bearing assumptions were tool-disproven — the witness did not materialize, NO false witness occurred (cert stayed `UNKNOWN=86` byte-identical), and the generator code was REVERTED.** Implemented GENERATOR-only in `stimuli_generator.rs`: a `PreludeKind {Count,Presence}` discriminator, `gen_has_fact_gates` + `compute_gen_has_fact_gates` (sibling of `compute_store_aware_gen_directives`, keyed on `has_fact(K,$ref) phase: post`), a `Presence` branch in `compute_reach_prelude` factored with the count branch into a shared `build_semantic_prelude`, all count-specific consumers (`reach_prelude_capture`/`_replay_text`/`_bypasses_count_prune` + the plannable-pass arming) guarded on `kind==Count` (so a Presence prelude is provably inert in the plannable pass), and a composed target-own probe arming the Presence prelude + forcing every mandatory child together. The detection chain worked (env-gated `PGEN_PRESENCE_DEBUG`: `gate=Some([("variable_binding","head")])`, `set_ok=true`, `armed=true`) but the genuineness oracle (`parseability_probe --parse-dump-ast-pretty`, counting `context_member_method` AST nodes) REFUTED it: **(A)** the innermost-first (`quantifier_sites.iter().rev()`) site scan resolved to an inner `(description,root/o5/s0)` quantifier reaching the producer `variable_decl_assignment` via a STRUCT MEMBER (`struct{ bit \foo ; }`) — a struct-scoped binding, NOT the file-scope `source_text := source_text_item*` site (and `int \foo ;` declaration) the design assumed; **(B)** the `has_fact(variable_binding,$head)` gate is NAME-sensitive — tool-proven: `int \bar ; (*\foo =+\foo .\foo .\foo ()*);` (decl name ≠ head) → **0** nodes vs `int \foo ; …` → **1** — and the injected declaration rendered `\foo` while the on-path chain head rendered `cBN`/`Q` (the prelude injection advances the seeded RNG past the deterministic first-identifier `\foo` the binding-less R-own probes all got), so the composed sample does not even parse. The implementation was SAFE (the `-0102` false-witness mode did NOT recur). The genuine `context_member_method_call` witness is BLOCKED on **generation-time name/value-selection** (the generation-side dual of the parser's `has_fact` gate = `STORE-AWARE-GEN.4b`) plus a `Presence` file-scope site selector → re-scoped to new leaf `.4.2.1.2.2.3`. SV `UNKNOWN`→0 continues on the independently-actionable `H.12.6.1`/M2/M3 frontier (context_member parked behind the capability dependency, not chased with a fragile name-coupling hack). Detail: [GRAMMAR-WELLFORMED-H1255334212-2-2-context-member-implement-refuted.md](GRAMMAR-WELLFORMED-H1255334212-2-2-context-member-implement-refuted.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change (the implemented code was reverted; rebuild-confirmed baseline `total=1291 witness=1204 UNKNOWN=86 spf=0`); clippy not invoked. [[feedback_always_signoff_decisions]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_tools_first_no_guessing]], [[feedback_why_and_where_before_solution]], [[feedback_corpus_expected_from_spec_not_fix]], [[project_store_aware_generation]], [[project_cert_coverage_tournament_loser_leak]].
- `2026-06-17` (`H.12.6` `no_path` LRM-grounded re-audit + STANDING no-deletion policy, `-0108`, PURE-DOCS): the director ruled (emphatic, repeated) that **every rule shall be reachable; a `no_path` rule indicates a flaw in the rules that should LEAD to it, not a removable rule; deletion is the last-last-last resort and only when the LRM objectively proves the rule absent.** Persisted as standing policy [[feedback_no_rule_deletion_without_lrm_proof]] (binds all future grammar work; raises the bar on past "dead/subsumed rule" removals). LRM-grounded re-audit of all 20 SV `no_path` rules (entry `systemverilog_file`, `sv_2017`): narrowing runs proved them entry/profile-relative — `--entry-rule sv_multi_entry_root` shrinks `no_path` 20→9 (11 reachable from the LRM's separate `library_text` start symbol, IEEE 1800-2017 §33/A.1.1); `--grammar-profile sv_2023` drops the interface-class family + `class_constructor_super_args` + `union_modifier` (genuine 1800-2023 features). Verdict: **19/20 LRM-legitimate (STAY)** = 10 `library_text`-rooted + 6 profile-relative SV-2023 + 3 decomposition artifacts (`sv_multi_entry_root`, `kw_n_29`, `kw_n_48`); **1 genuine producer-wiring suspect** (`module_path_conditional_expression`, A.8.3, stranded by `module_path_expression`'s un-eliminated conditional left-recursion @`systemverilog.ebnf:3117`) → fix leaf `H.12.6.1`; **0 deletions.** Tools-honest caveat: the `if (a?b:c)` specify probe PARSES but the AST oracle shows NO `conditional` node ⇒ the branch never matched — the grammar's "blessed mutual-recursion budget case" comment is NOT accepted without a `--trace-rules` proof (deferred to `H.12.6.1`). NOTE: no LRM **PDF** in-repo; the MD workspace (`docs/systemverilog/{2017,2023}/md/`) + extracted EBNF are the ground truth. Tracker [GRAMMAR-WELLFORMED-H126-no-path-lrm-reaudit.md](GRAMMAR-WELLFORMED-H126-no-path-lrm-reaudit.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; SV stays `UNKNOWN=86`. [[feedback_unreachable_target_attribution_rule]], [[project_ebnf_is_single_source_of_truth]], [[feedback_be_alert_root_cause_fishy_immediately]].
- `2026-06-17` (`H.12.5.5.3.3.4.2.1.2.2` context_member GENUINE-witness composition — WHY+WHERE + SAFE DESIGN, `-0107`, PURE-DOCS): **the `-0101` open composition question is RESOLVED tools-first, and the `.4.2.1.2.2` leaf is split into a done DESIGN (`.2.2.1`) + a known-safe IMPLEMENT (`.2.2.2`, new frontier).** Baseline reproduced byte-identical (seed 0 `total=1291 witness=1204 UNKNOWN=86 spf=0`). (1) MINIMAL genuine-witness recipe pinned with the parser as judge (`parseability_probe --parse-dump-ast-pretty`, counting `context_member_method` AST nodes): a top-level decl of the head (satisfies `has_fact(variable_binding,$head)`) + `callable_method_call_body` rendered as a CALL — **the `[idx]` is NOT required** (`int \foo ; (*\foo =+\foo .\foo .\foo ()*)` → **1** node; identical WITHOUT the decl → **0**; bare-ref → **0**), refining the `-0101`/`-0102` `.foo[0].foo()` framing. (2) Two gaps pinned on the SHIPPED generator (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, 16 probe lines): **A** SV emits NO prelude (empty `gen_count_kinds` ⇒ `compute_reach_prelude:2776`→`None`); **B** the `.3.3.1` mandatory-child forcing renders `[idx]` and `()` in SEPARATE probes, never together, never with a declared head. The binding+bare-ref probe is EXACTLY the `.4.2.1.2.1` FALSE-witness shape (the deferred soundness gap credits a rule with 0 AST nodes), so the `UNKNOWN` count is necessary-but-NOT-sufficient — the genuineness oracle is the AST node. (3) SAFE composition design: a `Presence` `has_fact` prelude with `captured=None` (the plannable driver arms only when `captured.is_some()` ⇒ a Presence prelude is structurally inert in the plannable pass ⇒ the `-0102` trap cannot recur with no change to that driver), armed ONLY in the target-own pass, ONLY on the all-mandatory-children (call-forced) probe ⇒ a c2-shaped GENUINE witness, never a binding+degenerate render that the gap could false-witness. Edit surface mapped (`ReachPrelude`+new `PreludeKind`, `gen_has_fact_gates` via `compute_store_aware_gen_directives:9437`, a `has_fact` branch in `compute_reach_prelude`, arm+compose in `generate_target_own_structure_witnesses`). Director rhythm honored: WHY+WHERE/design-first on tangible proof, then a TARGETED fix — not trial-and-revert. Detail: [GRAMMAR-WELLFORMED-H1255334212-2-context-member-genuine-witness-design.md](GRAMMAR-WELLFORMED-H1255334212-2-context-member-genuine-witness-design.md). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV stays the only non-fully-certified shipped grammar (`UNKNOWN=86`, unchanged). [[feedback_research_grounded_sota_no_trial_and_revert]], [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_always_signoff_decisions]], [[project_cert_coverage_tournament_loser_leak]].
- `2026-06-17` (DIRECTOR CROSS-LANE SEQUENCING — BINDING): after the SV+VHDL external-corpus acquisition (`PGEN-EXTERNAL-CORPUS-0007`), the director set the order for the three open lanes — **do them in sequence, do NOT pivot to lane 3 until 1+2 are clean:** **(1) SV `UNKNOWN`→0** (`GRAMMAR-WELLFORMED.H.12`, at 86 — `.4.2.1.2.2` → `H.12.5.6` M2 → `H.12.5.7` M3, to SV fully_certified); **(2) the `a ##1 b` cycle-delay (`##`) parse bug** (`H.12.5.8`, found during `-0105`; rejects valid SV; open the leaf, tools-first WHY+WHERE first); **(3) THEN full-time PARSE-COMPLETENESS** — the big external-corpus gap-drive (adjudicate the `-0007` fails via each corpus's own answer key — VESTS compliant[must-parse]/non_compliant[must-reject], GHDL `gna` expected-fail drivers, sv-tests `:should_fail_because:` — then fix the confirmed real gaps; VESTS alone shows ~2042 compliant-but-rejected real gaps + ~391 non_compliant-but-accepted over-accepts). The director "won't pivot right now [to] the first [lane 3]" — lane 3 waits for 1+2.
- `2026-06-17` (`H.12.5.5.3.3.3` C-i `goto_repetition` operator-shadow — WHY+WHERE + GRAMMAR-FIX design, LRM-grounded FIRST): **the whole `boolean_abbrev` repetition family dropped the LRM-mandated literal `[ ]` delimiters in extraction — one coherent delimiter-drop defect (the documented SV-extraction class) with TWO symptoms: (1) the parser REJECTS valid IEEE 1800 sequence repetitions `[*N]`/`[*]`/`[+]`/`[->N]`/`[=N]` (real shipped bug) and (2) the bare delimiter-less forms collide with the `*`/`->`/`=` operators so the rules are operator-shadowed → `goto_repetition` ∈ cert `UNKNOWN`.** LRM ground truth (tools-first, [[project_ebnf_is_single_source_of_truth]], [[feedback_verify_rule_correctness_before_runtime_hypotheses]]): `docs/systemverilog/2017/grammar_clean.ebnf:414` `goto_repetition ::= [-> const_or_range_expression ]`, `:738` `non_consecutive_repetition ::= [= const_or_range_expression ]`, and IEEE 1800 §A.8.1 `consecutive_repetition ::= [* const_or_range_expression ] | [*] | [+]` — ALL carry literal brackets the active grammar lacks (`grammars/systemverilog.ebnf:1246` `consecutive_repetition := ( star const_or_range_expression ) | ( star ) | ( plus )`, `:2238` `goto_repetition := ( implies const_or_range_expression )`, `:3355`/`:3415` `non_consecutive_repetition_{sv_2017,sv_2023} := ( assign const_or_range_expression )` — no `lbrack`/`rbrack` anywhere in the chain, and the call site `sequence_expr := … expression_or_dist ( boolean_abbrev )?` adds none). PER-SITE PARSE EVIDENCE (`parseability_probe --parse systemverilog <file> --profile sv_2017`, against the `-0104` baseline binary): `a [*3]`❌(furthest 27), `a [*]`❌, `a [+]`❌, `a [->2]`❌(28), `a [=2]`❌(27), realistic `a ##1 b [*2]`❌ — every LRM-valid bracketed form REJECTED; bare `a *3`✅ / `a =2`✅ wrongly accepted (the over-acceptance dual). CERT BASELINE reproduced (`PGEN_CERT_COVERAGE_DUMP_ALL=1 … --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0`): `total=1291 proof=1 witness=1203 UNKNOWN=87 spf=0 fully_certified=false` (matches the layer-A pointer); of the family ONLY `goto_repetition` ∈ UNKNOWN (its bare `->` is ALWAYS shadowed by the implication operator; `consecutive_repetition`/`non_consecutive_repetition` happen to be witnessed via their bare forms elsewhere) and `goto_repetition` is NOT in the 20-rule `no_path` set ⇒ reachable-but-unwitnessed, a genuine fix target (not a dead rule). ATTRIBUTION-RULE VERDICT: GRAMMAR-class (delimiter-drop), exactly like the `.3.3.5` stream fix (`SV-0002`); a generator/reach hack would be a workaround against the grammar-first fix hierarchy ([[feedback_no_workarounds_fix_hierarchy]], [[feedback_prefer_grammar_leave_engine_alone]], [[feedback_fix_parser_bugs_asap_highest_priority]]). **FIX DESIGN (restore the LRM `[ ]` across the whole family, mirroring the proven in-grammar `associative_dimension := lbrack star rbrack` / `lbrack data_type rbrack -> {body: $2}` and `bit_select := ( lbrack bit_select_expression rbrack )*` idiom — `lbrack`=$1, op=$2, range=$3, rbrack=$4):** `consecutive_repetition := lbrack star const_or_range_expression rbrack -> {kind: "star_range", range: $3} | lbrack star rbrack -> {kind: "star"} | lbrack plus rbrack -> {kind: "plus"}`; `goto_repetition := lbrack implies const_or_range_expression rbrack -> {range: $3}`; `non_consecutive_repetition_sv_2017 := lbrack assign const_or_range_expression rbrack -> {range: $3}`; `nonconsecutive_repetition_sv_2023 := lbrack assign const_or_range_expression rbrack -> {range: $3}`. The EMITTED shape stays `{kind,range}`/`{range}` (the brackets are parsed then folded away by the annotation) ⇒ NO AST-dump schema change (schema stays 4; contrast `.3.3.5` which bumped 3→4 for a raw→array body change). LOCKSTEP (consumer-visible parse-acceptance change ⇒ full ceremony per [[feedback_grammar_edit_proof_gate_lockstep]]): regen SV parser (`make focus_systemverilog`, assert mtime>grammar per [[feedback_verify_sv_parser_regen_mtime]]) → rebuild `ast_pipeline` → cert seeds 0/7/42 (expect `goto_repetition` leaves UNKNOWN; siblings stay witnessed via the now-bracketed forms; GLOBAL no-regress; spf=0) + post-fix parse re-test (`[*3]`/`[->2]`/`[=2]`/`a ##1 b [*2]` now PASS; bare `a *3` falls back to a multiply expression — confirm still PASS, no new reject of valid input) + AST-dump confirms `range`=the const_or_range_expression + SV external corpus 14/14 + `--lint-grammar` clean + `stimuli_cross_family_platform_gate` + SV shape-contract GREEN + clippy source-clean; then SV release `1.0.141 → 1.0.142`, ledger row `SV-0004`, SV contract + parser-book schema/changelog sync, grammar-wellformedness book SV-arc beat, live docs. Change ONE coherent thing (the family delimiter restore); measure GLOBAL; commit ONLY on improvement.
- `2026-06-17` (`H.12.5.5.3.3.4.1` direct_index DEAD-BRANCH REMOVAL, `-0104`, GRAMMAR FIX — parse-neutral, release/schema UNCHANGED): **SV cert `UNKNOWN 88→87` via the attribution rule's delete-the-dead-branch-at-source path.** RE-VERIFIED the "dead" premise tools-first BEFORE touching the grammar (the `.4` re-adjudication flagged the `-0097` adjudication unreliable): `direct_index_method_call` produces **0** committed `direct_index_method` AST nodes on 6 inputs shaped exactly for it (`b[c.d()]`/`b[this.d()]`/`b[pkg::c.d()]`/`b[super.d()]`/`b[c.d.e()]`/`b[c::d.e()]` all route to `method`), sits in the never-witnessed cert UNKNOWN set, is structurally subsumed by `method_call`'s `( dot method_call_body )*`, has no live fixture/corpus node, and removing it orphans nothing (sub-rules used 10–36×). Removed the rule + its `bit_select_expression` branch 1 (now 3 alts). VERIFIED (decisive A/B): witness 1203 UNCHANGED, `UNKNOWN 88→87`, spf=0, seeds 0/7/42; **SV external corpus triage 14/14** (`parse_fail_total=0`); SV shape-contract GREEN; clippy source-clean. Parse-neutral ⇒ NO release/schema/ledger bump. Lockstep: contract typed-AST table, manifest calibration_history, grammar-wellformedness book SV-arc. Commit: `PGEN-GRAMMAR-WELLFORMED-0104`.
- `2026-06-17` (`H.12.5.5.3.3.4.2.1.2.1` cert-coverage witness-SOUNDNESS gap FIX ATTEMPT → **REVERTED at director direction**; baseline restored): **the engine fix for the soundness leak is tool-verified CORRECT but its blast radius REGRESSES the tracker, so it must not land — a wide-blast "soundness fix" is the wrong, non-targeted instrument.** Root-caused the leak (NOT WHY+WHERE candidate (a)/(b)): the multi-branch **LongestMatch tournament** (`generate_or_logic`) rolls back semantic state per losing branch (the C3-B fix) but never rolled back the `coverage_stack`, so a *losing-but-successful* branch (try_parse returns `Some`) leaks its rule-entry coverage. Built the symmetric fix (snapshot len at tournament start, capture+truncate per successful branch, replay only the winner) — verified correct: deterministic seeds 0/7/42, two-way SV regression test passes (bare-ref no longer false-witnesses `context_member_method_call`; genuine method-call still does), 5/6 fully-certified grammars BYTE-IDENTICAL. **BUT it exposed that the leak was INFLATING witness counts: svpp `UNKNOWN 0→1` (`kw_none`, lost `Done`), SV `88→146` (58 false witnesses).** Because it is ONE shared codegen change, the math forces ≥58 genuine witnesses before it can commit without regression = the whole SV cert endgame. Director ruling (2026-06-17, emphatic, repeated): "we can't commit a regression / commit only improvements / the regression means the analysis leading to the code change was flawed / revert your latest code change / no point keeping it." → **REVERTED, baseline restored + verified** (SV witness=1203 UNKNOWN=88, svpp UNKNOWN=0 fully_certified, byte-identical). Leaf re-classified **`deferred` — a DOCUMENTED known soundness limitation, NOT "fixed".** The honest cert re-baseline is a deliberate director decision, not something to slip in under a leaf. Finding preserved in durable memory ([[project_cert_coverage_tournament_loser_leak]]); the positive-lookahead `&X` is a second latent instance of the same root. No code/grammar/generated/release/ledger change (baseline intact). [[feedback_tools_first_no_guessing]] (fix shall always be targeted), [[feedback_always_signoff_decisions]].
- `2026-06-16` (`H.12.5.5.3.3.4.2.1.2.1` cert-coverage witness-SOUNDNESS gap WHY+WHERE, `-0103`, landed PURE-DOCS): **the foundational leak is PINNED tools-first and reproduced on a PLAIN SV input — no reverted prelude needed.** A real declaration `int \foo ;` emits the `variable_binding` fact, so `context_member_method_call`'s `has_fact($head)` post-gate passes naturally; the bare-ref `int \foo ; (*\foo =+\foo .\foo .\foo *);` then **false-witnesses** the rule: through the witness oracle `parse_and_cover_systemverilog` (a throwaway SOURCE test, since reverted) `context_member_method_call` ∈ `exercised_rule_names`, while `parseability_probe --parse-dump-ast-pretty` shows **0** `context_member_method` AST nodes (the method-call control `… \foo .\foo [0].\foo ()` → 1). MECHANISM (`--trace-rules` + engine-source read, doctrine-clean): the body memoizes a structurally-DEGENERATE success (`💾 Memoized successful result for rule 547`), the `has_fact(variable_binding,\foo)→true` post-predicate PASSES, the committed parse routes the bytes through a SIBLING (`call_with_postfix_chain → …`, so the rule is absent from the AST) — and the rule's coverage push, **never truncated by `try_parse` on the discard**, is frozen into the `coverage_delta` of an ANCESTOR memo entry (`attribute_instance`, the innermost on-chain ancestor that memoizes a success and is memo-HIT on the committed path) whose replay re-introduces it onto the live stack → it survives into `exercised_rule_names`. This is the `H.10.2.2` / `.b.6.2.36.4` memoization × transactional-coverage composition class in a NEW trigger; the `H.10.2.2` replay-transactionality fix cannot help because the captured delta was already wrong AT CAPTURE TIME (its result node omits the rule but its `coverage_delta` includes it). WHERE in source (`rust/src/ast_pipeline/ast_based_generator.rs`, emitted verbatim): coverage push `:2701-2702`; `try_parse` truncate `:6128`/`:6209` (the ONLY removal leg); `memoized_call` capture/replay `:6374`/`:6383-6384`/`:6354-6356`; `with_semantic_runtime_rule_transaction` (`:1538-1885`) manages ONLY `semantic_runtime_state` — its post-predicate-reject (`:1835`) and error-restore (`:1874-1883`) roll back facts + rule-context but NEVER touch `coverage_stack`. ROOT: a coverage push is removed only by `try_parse`; a success-then-reject (post-predicate-reject, or a committed-then-abandoned ordered-choice branch) that does not pass a truncating `try_parse` before an enclosing rule memoizes leaks the push. **A non-committing, predicate-passing (or otherwise discarded) rule must never be witnessed — today it can be; this protects EVERY witness number, latent on shipped grammars only because none currently generate a degenerate predicate-passing render in the witness pass.** FIX handed to the next slice (`.4.2.1.2.1` FIX): (a) truncate `coverage_stack` to the rule's entry length on the `with_semantic_runtime_rule_transaction` reject/error path symmetric with `try_parse`, and/or (b) at memo capture exclude coverage ids not in the memo's output node — settled tools-first, change ONE thing, measure GLOBAL cert + spf seeds 0/7/42 for every wired grammar (fully-certified roster byte-identical), re-deriving the exact discard via CODEGEN instrumentation + `make focus_systemverilog`, never by editing `generated/*.rs` ([[feedback_never_edit_generated_artifacts]]). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV stays the only non-fully-certified shipped grammar (`UNKNOWN=88`, unchanged). Detail: [GRAMMAR-WELLFORMED-H1255334212-1-cert-coverage-witness-soundness-whywhere.md](GRAMMAR-WELLFORMED-H1255334212-1-cert-coverage-witness-soundness-whywhere.md). [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_never_edit_generated_artifacts]].
- `2026-06-16` (`H.12.5.5.3.3.4.2.1.2` context_member `has_fact` prelude FIX ATTEMPT → REFUTED, `-0102`, landed PURE-DOCS): **the `-0101` prelude-alone design produces a FALSE witness — implemented, measured, proven false, and REVERTED; the cert stays honest at `UNKNOWN=88`.** Implemented the design faithfully in `stimuli_generator.rs` (new `gen_has_fact_gates` map + `compute_gen_has_fact_gates`; a `PreludeKind` discriminator on `ReachPrelude` — `Count` two-phase capture/replay vs `Presence` armed-at-1 no-capture/no-replay, with `Count`-only guards on capture/replay/count-prune-bypass; a `has_fact` branch in `compute_reach_prelude` factored with the count branch into a shared `build_semantic_prelude`). It compiled clean and the seed-0 metric moved exactly as predicted: `UNKNOWN 88→87`, witness `1203→1204`, spf=0. **The design's OWN mandated verification ("dump the armed sample; confirm decl + `.method()`") REFUTED it.** The PLANNABLE pass (not the target-own pass) witnessed it on `(*\foo =+type(struct{…bit\foo ;…})*)(*\foo =+\foo .\foo .\foo *);` — a bare hierarchical ref with **no `()`**. Three tools agree it is NOT a genuine witness: (1) `parseability_probe --parse-dump-ast-pretty` → **0** `context_member_method` AST nodes (genuine Test c2 `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*)` → **1**); (2) `--trace-rules context_member_method_call` → only backtracks, NO positive success-exit; (3) the committed AST omits the rule (`call_primary` never yields the `context_member_method` kind). MECHANISM: the prelude's binding makes the `has_fact(variable_binding,$head)` post-predicate PASS on the plannable pass's DEGENERATE no-`.method()` render; a memo × transactional-coverage composition gap (the `H.10.2.2`/`.b.6.2.36.4` class, on the post-predicate-passes-then-structurally-fails path) then credits `context_member_method_call` (rule id 547 — `Memoized successful result at position 101` while absent from the committed AST). Latent in the shipped grammar (without the prelude the gate fails → the rule rejects early → no degenerate render is generated). Per [[feedback_always_signoff_decisions]] a false witness must not land ⇒ **REVERTED**; rebuild-verified baseline `total=1292 witness=1203 UNKNOWN=88 spf=0`. Re-scope: `.4.2.1.2.1` (ENGINE, foundational) fixes the cert-coverage witness-soundness gap (a non-committing predicate-passing rule must never be witnessed — protects EVERY witness number), then `.4.2.1.2.2` (GENERATOR) re-introduces the prelude composed with mandatory method-call-structure forcing so the witness is GENUINE (Test-c2-shaped). LESSON: the `UNKNOWN` count is necessary but NOT sufficient for a predicate-gated rule — the genuineness oracle is the construct's AST node (or a positive success-exit), not the count. New KM card [[sv-cert-coverage-predicate-gated-false-witness]]. Detail: [GRAMMAR-WELLFORMED-H1255334212-context-member-prelude-fix-attempt-refuted.md](GRAMMAR-WELLFORMED-H1255334212-context-member-prelude-fix-attempt-refuted.md). Landed PURE-DOCS (the implemented code was reverted) ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV stays the only non-fully-certified shipped grammar (`UNKNOWN=88`, unchanged). [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_always_signoff_decisions]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_why_and_where_before_solution]].
- `2026-06-16` (`H.12.5.5.3.3.4.2.1.1` context_member prelude WHY+WHERE+DESIGN, `-0101`, PURE-DOCS): **the `-0098` "store-gated witness-reach gap → generator prelude" routing is CONFIRMED and realizable on the existing reach path, with an empirically-validated design.** Tools-first reach-path dump (`PGEN_REACH_PATH_DUMP=1` + `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, seed-0 baseline `total=1292 witness=1203 UNKNOWN=88 spf=0` reproduced byte-identical): `context_member_method_call`'s shallowest reach is inside an `attribute_instance` `(* attr = const_expr *)` (`description → attribute_instance → attr_spec → constant_expression → … → constant_function_call → call_primary`), and the minimal probe `(*\foo =+\foo .\foo .\foo *)` is **not even a method call** (no `()`) and has no declared head ⇒ `parsed=true witnessed_target=false`. Witness matrix (`parseability_probe --parse-dump-ast-pretty`, `context_member_method` AST node = witness): **(a)** bare probe → no; **(b)** `module m; int \foo ; int \bar ; initial \bar = \foo .\foo [0].\foo (); endmodule` → **YES**; **(c)** `int \foo ; (*\foo =+\foo .\foo .\foo *)` (decl but bare ref) → no; **(c2)** `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*)` (decl + real method call) → **YES**; **(d)** undeclared head in a module → REJECT; **(e)** declared, no index → YES. ⇒ three operative facts: the head decl is necessary (the `has_fact(variable_binding,$head)` gate), the chain must be a real method call (`.callable_method_call_body`), and **(★) a TOP-LEVEL `variable_binding` fact is visible to `has_fact` at a LATER attribute_instance** — so the existing reach path is salvageable via a top-level binding-producer prelude (no reach-path re-selection). Identifiers render canonically `\foo` in construct mode ⇒ **name-coupling is free**. WHERE: `compute_reach_prelude` (`stimuli_generator.rs:2769`) returns `None` for SV (`gen_count_kinds.is_empty()`, `:2776`) — it is `fact_count_at_least`-only; there is no `has_fact` analogue of `gen_count_kinds`. DESIGN: new `gen_has_fact_gates` map (rule → kind + name-ref capture) + a `has_fact` branch in `compute_reach_prelude` (producer `variable_decl_assignment`, site `source_text := source_text_item*`, `iterations=1`, NO numeric capture / NO count-prune bypass — SV is not `store_aware_gen`; the gate is a PARSE-time post-predicate so only the prelude TEXT is needed). Open composition question handed to `.4.2.1.2`: the binding prelude (plannable pass) and the mandatory `.method()` structure (target-own pass / `.3.3.1` mandatory-child lineage) must co-occur in ONE sample. New KM card [[sv-store-fact-scope-and-canonical-name-coupling]]. Detail: [GRAMMAR-WELLFORMED-H125533421-context-member-prelude-whywhere-design.md](GRAMMAR-WELLFORMED-H125533421-context-member-prelude-whywhere-design.md). Split `.4.2.1` (now a CONTAINER) → `.4.2.1.1` (this, done) + `.4.2.1.2` (the FIX, frontier). PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV stays the only non-fully-certified shipped grammar (`UNKNOWN=88`, unchanged). [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[project_ebnf_is_single_source_of_truth]].
- `2026-06-16` (`H.12.5.5.3.3.4.2.2.1` checked_nettype_identifier store-gate FIX, `-0100`, GRAMMAR FIX — release 1.0.141, schema stays 4, ledger `SV-0003`): **a real released-parser bug FIXED — module-scope class-handle `C a;` was mis-parsed as a `net_declaration`; now correctly a `data_declaration`.** ONE-line grammar change implementing the `-0099` fix direction: `checked_nettype_identifier`'s `@predicate` (`grammars/systemverilog.ebnf:3310`) tightened from `has_fact(type_name, $body)` to `fact_attribute_equals(type_name, $body, declaration_family, nettype)`, so the `net_declaration` user-nettype branch resolves ONLY to a declared nettype (a class is also a `type_name`, so the bare gate over-accepted it). The parser-agnostic engine (`rust/src/ast_pipeline/`) is UNTOUCHED — SV logic stays in the SV grammar ([[feedback_prefer_grammar_leave_engine_alone]], [[feedback_grammar_rules_must_consult_store]]). VERIFIED (all green): `C a;` → `data_declaration`/`variable_decl` (binds `a`); module-scope `C a; … a.b[0].c()` REJECT→PASS + witnesses `context_member_method`; `wire a;` still net; `nettype logic NT; NT a;` preserved; undeclared-head chain still rejected. SV cert-coverage byte-identical (`total=1292 witness=1203 UNKNOWN=88 spf=0`, seeds 0/7/42, UNKNOWN set identical — correctness re-route, NOT a cert-witness closure; `context_member_method_call`'s cert witness is the separate `.4.2.1` generator pass); `--lint-grammar` clean; **SV external corpus 14/14**; `stimuli_cross_family_platform_gate` PASS; SV shape-contract GREEN + new Rust regression lock `systemverilog_class_handle_decl_is_data_declaration_not_net` (in `rust/src/ast_shape_contract.rs`, the per-parser verification runner — NOT engine code; the established SV-0001/VHDL-0002 lock location); clippy source-clean. Full lockstep: ledger `SV-0003`, contract 1.0.141 schema-note, SV parser-book changelog-index + schema-versioning, grammar-wellformedness book SV-arc narrative (cert-neutral bug-finding-oracle outcome). The bug-finding-oracle role compounding: the cert `UNKNOWN`→0 drive (chasing `context_member_method_call`'s class-handle non-witness) surfaced a real shipped-parser bug even though the coverage number did not move. [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_grammar_rules_must_consult_store]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_always_signoff_decisions]].
- `2026-06-16` (`H.12.5.5.3.3.4.2.2` class-handle head WHY+WHERE, `-0099`, PURE-DOCS): **a real released-parser bug found in the M2 store-gating family — `checked_nettype_identifier` over-accepts a declared class as a nettype, so `C a;` at module scope is mis-parsed as a `net_declaration`.** Tools-first chase of the `-0098` secondary finding: AST dump shows `C a;` (C a declared class) → `net_declaration` (vs `int a;` → `data_declaration`/`variable_decl`); `--trace-rules` shows `checked_nettype_identifier` matching `C`. A `net_declaration` does not run `variable_decl_assignment`, so `a` gets no `variable_binding` fact → `context_member_method_call`'s `has_fact(variable_binding,$head)` post-predicate fails → the indexed 3-level chain rejects (the non-indexed form survives via `ident_postfix_chain`/`split_hierarchical`, which need no binding). WHERE: `systemverilog.ebnf:3310` — `@predicate has_fact(type_name, $body)` on `checked_nettype_identifier := declaration_identifier` is UNDER-specified; a class is also a `type_name` (`class C` emits `type_name` `declaration_family: class`), so a class name satisfies the nettype gate, even though the rule's own comment states the intent is `declaration_family: nettype`. LRM: a class is not a nettype ⇒ `C a;` is a `data_declaration`. The defect is the [[feedback_grammar_rules_must_consult_store]] class (consults the store, but with too weak a predicate). FIX (verified-sound, → child `.4.2.2.1`): tighten to `fact_attribute_equals(type_name, $body, declaration_family, nettype)` (mirrors `known_unscoped_block_class_type`/`checked_type_identifier`, which the comment cites). Preconditions tool-verified: **P1** `nettype logic NT; … NT a;` parses today and the tightened gate preserves it (nettypes carry `declaration_family: nettype`); **P2** the same class-handle indexed chain `C a; … a.b[0].c()` PARSES and witnesses `context_member_method` in CLASS scope (where `net_declaration` is not a `class_item`, so `data_declaration` matches and binds `a`) — the exact module-scope outcome the fix reproduces. Language-changing (`C a;`: net→data) ⇒ full grammar-edit lockstep in `.4.2.2.1`; real parser bug ⇒ HIGH priority (fix parser bugs ASAP). Squarely the M2 store-gating class (`H.12.5.6` family); the over-gen-vs-bug adjudication is done here (it is a real bug). Does NOT block the `.4.2.1` cert witness (the integral-head witness already proves `context_member_method_call` witnessable). Detail: [GRAMMAR-WELLFORMED-H1255334-422-classhandle-nettype-whywhere.md](GRAMMAR-WELLFORMED-H1255334-422-classhandle-nettype-whywhere.md). PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change; SV stays the only non-fully-certified shipped grammar (`UNKNOWN=88`, unchanged). [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_grammar_rules_must_consult_store]], [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[project_ebnf_is_single_source_of_truth]].
- `2026-06-16` (`H.12.5.5.3.3.4.2` context_member RE-ADJUDICATION, `-0098`, PURE-DOCS): **the `-0097` Carrier-2 verdict (a′) "real latent released-parser bug → repair the grammar" is REFUTED, tool-backed — there is NO parser bug; the cert `UNKNOWN` is a store-gated WITNESS-REACH gap.** Two `-0097` claims were checkable and both false: (1) "the generated parser carries NO `@predicate`" — the `has_fact(variable_binding, $head)` predicate IS live (`generated/systemverilog_parser.rs:3041-3055`; the no-predicate grep hit a one-line-grep trap — rustfmt wraps `directives_by_rule.insert(` and the rule-name string onto separate lines); (2) "`a.b[0].c()` is rejected even with a declared head" — it PARSES and the accepted AST contains a `context_member_method` node with a *declared* head (`module m; int a; int x; initial x = a.b[0].c(); endmodule`; also `logic`/`bit`), the undeclared form correctly rejecting (the `post`-predicate "regression firewall" working as designed). `-0097`'s failing tests never established the binding (`class C; … a.b[0].c()` left `a` undeclared; `module m; C a; …` rejected at the `C a;` decl because `C` was an undeclared type). Baseline reproduced byte-identical (`total=1292 witness=1203 UNKNOWN=88 spf=0`, seed 0). TRUE root cause: the witness pass cannot *generate* a sample satisfying the `has_fact(variable_binding,$head)` gate — it needs a name-coupled binding-producer PRELUDE (a `variable_decl_assignment` declaring a variable whose name equals the rule's `$head` render) the minimal reach derivation lacks; this is the `has_fact` analogue of the regex `\NN` semantic-prelude class, and the existing C2.2 pass (`stimuli_generator.rs:2769 compute_reach_prelude`) is `fact_count_at_least`-only with no name-coupling. Generator constructor gap by the attribution rule (grammar/parser are both correct) → fix child `.4.2.1` (GENERATOR capability; NO grammar/release/ledger). SECONDARY (separate WHY+WHERE ticket `.4.2.2`): a class-handle head (`C a;`, C a declared class) does NOT witness `a.b[0].c()` (rejects at the `[`) while integral heads do — a potential real gap for the realistic uvm `urme_container.elements[i].clone()` shape, to be tools-first investigated. Carrier 1 (`direct_index_method_call`, `.4.1`) stands but its "effectively dead/subsumed" premise must be INDEPENDENTLY re-verified given this demonstrated adjudication unreliability (its decisive A/B is mandatory regardless). Detail: [GRAMMAR-WELLFORMED-H1255334-42-context-member-readjudication.md](GRAMMAR-WELLFORMED-H1255334-42-context-member-readjudication.md). PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change; SV stays the only non-fully-certified shipped grammar (`UNKNOWN=88`, unchanged). [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_tools_first_no_guessing]], [[feedback_why_and_where_before_solution]], [[feedback_no_workarounds_fix_hierarchy]], [[project_ebnf_is_single_source_of_truth]].
- `2026-06-16` (`H.12.5.5.3.3.4` C-ii residual parent-commit ADJUDICATION, `-0097`, PURE-DOCS): **both C-ii carriers are GRAMMAR-class — the leaf's hypothesized branch (b) parent-commit GENERATOR forcing is REFUTED for both — but they need DIFFERENT grammar fixes.** [⛔ NOTE: the Carrier-2 (`context_member_method_call`) portion of this entry is SUPERSEDED/REFUTED by `-0098` above — there is no parser bug; see that entry. Carrier 1 stands pending independent re-verification.] Tools-first (DEBUG `ast_pipeline` cert `DUMP_ALL`+`DEBUG_PROBES`, `--lint-grammar`, `parseability_probe --parse-dump-ast-pretty`/`--trace-rules`; baseline `total=1292 witness=1203 UNKNOWN=88 spf=0` seed 0, deterministic ⇒ signal). The linter (adjudicator) reports `ordered_choice_shadowing=0` with the 6 `always_matches` warnings all on UNRELATED rules ⇒ no SOUND shadowing on either carrier — expected, because the relationship is FIRST-set SUBSUMPTION (the unsound-for-PEG class the book deliberately omits), so manual tool-backed adjudication. **`direct_index_method_call` → (a) effectively DEAD/subsumed:** it is `bit_select_expression` branch 1 under `priority_first` (tried FIRST, so it would WIN if it ever matched), yet a routing matrix of 6 diverse `x[body]` inputs (`b.c()`/`this.foo()`/`this.foo`/`super.bar()`/`b.c`/`a.b.c()`) ALL route to `method_call` (branch 2) with `direct_index_method` AST-node count 0 every time; trace shows its `method_call_body` settles on the BARE method name (leaving the `()` args) so the `bit_select` `]` fails and PEG falls to `method_call`, whose language (`method_call_initial (dot method_call_body)*` incl. `direct_method_call`/`split_direct_callable_method_call`) is a strict SUPERSET; LRM §11.5.1 `bit_select ::= {[expression]}` ⇒ `direct_index_method_call` is a redundant synthesis artifact (branches 2/3/4 cover the LRM). Fix = REMOVE (decisive A/B parse-neutral; closes `88→87`; referenced ONLY at `:670`) → `.4.1`. **`context_member_method_call` → (a′) real latent PARSE GAP (bug-finding-oracle hit) [⛔ SUPERSEDED — REFUTED by `-0098`; see the `2026-06-16` `-0098` decision below]:** the store-gate hypothesis (C-iii) is REFUTED — the generated parser has NO `@predicate` on it (the `:2915` "gated by variable_binding" comment is stale dead text) and it's rejected even with a declared head — and its OWN designed form `a.b[0].c()` (`head.member[idx].method()`) is REJECTED parser-wide while `a[0].c()`/`a.c()`/`a[0].c`/`a.b[0].c`/`a.b.c()` all parse (sharp boundary). `a.b[0].c()` is valid SV and the exact uvm `elements[i].clone()` shape the rule was authored for; `uvm-core-2020.3.1` has 0 instances of the shape (`grep -cE '\[[^]]*\]\.[A-Za-z_][A-Za-z_0-9]*\('`→0) ⇒ the corpus never caught it (14/14 holds). The cert `UNKNOWN` is the SYMPTOM of the rule being broken (it can't match its own form). Fix = root-cause + REPAIR the grammar so `X.member[i].method()` parses (witnesses the rule); real released-parser bug ⇒ HIGHEST priority + full grammar-edit lockstep → `.4.2`. Both routed to the GRAMMAR by the attribution rule, exactly as the grammar-first suspicion order prescribes. Detail: [GRAMMAR-WELLFORMED-H125534-cii-residual-parent-commit-adjudication.md](GRAMMAR-WELLFORMED-H125534-cii-residual-parent-commit-adjudication.md). PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change; SV stays the only non-fully-certified shipped grammar (`UNKNOWN=88`). [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_fix_parser_bugs_asap_highest_priority]].
- `2026-06-16` (`H.12.5.5.3.3.5` C-i stream delimiter-drop FIX IMPL, `-0096`, GRAMMAR FIX, release 1.0.140, schema 3→4, ledger `SV-0002`): **DONE — the fix landed, and the implementation REVISED the `-0095` design with a tools-discovered third edit (the design's "streaming_concatenation unchanged" assumption was wrong).** SV cert `UNKNOWN 89→88` (`array_range_expression` closed; witness `1202→1203`; spf=0; deterministic seeds 0/7/42; zero newly-unknown; 1292 rules unchanged). **The two designed edits landed verbatim** ((1) `stream_concatenation := lbrace stream_expression ( comma stream_expression )* rbrace -> {body: [$2, $3::2*]}`; (2) `stream_expression := expression ( kw_with_8fcd25a3 lbrack array_range_expression rbrack )?`, `-> {expr:$1, with_clause:$2}` position-stable). **THIRD EDIT (the lesson):** restoring the mandatory inner brace exposed a latent PEG greediness in `streaming_concatenation`'s `( slice_size )?` — `slice_size`'s `constant_expression` branch greedily consumed the brace-concatenation that IS the now-mandatory `stream_concatenation`, then the rule failed without backtracking. The probe matrix (tools-first, per [[feedback_no_codebase_change_without_tool_backed_facts]]) caught it BEFORE commit: the unguarded edit REJECTED `{>> {data}}` / `{>> {a, b}}` / `{>> {a, b, x}}` — the canonical no-slice brace forms real UVM uses ({`{>>{i}}`, `{<<{v}}`, `{<<{stream[i]}}`} all appear in the corpus). WHY+WHERE root-caused, then resolved with `streaming_concatenation := lbrace stream_operator ( slice_size &lbrace )? stream_concatenation rbrace` — an `&lbrace` positive-lookahead guard encoding the LRM structural fact that a slice_size is ALWAYS immediately followed by the mandatory stream-concatenation's `{` (so the guard never wrongly rejects valid input; `( slice_size &lbrace )?` backtracks the optional to empty when the brace is absent — standard PEG). VERIFIED transparent: `streaming_concatenation`'s `{op, slice_size, body}` shape is byte-identical (the `&lbrace` lookahead does not pollute `$3`; `slice_size` serializes as the constant-expression / `[]`-when-absent, same as before), confirmed via `parseability_probe --parse-dump-ast-pretty`. SHAPE delta (the schema bump): `stream_concatenation.body` raw quantified-of-quantified → clean array `[stream_expression, …]`; `stream_expression.with_clause` inner shape now carries the bracket tokens. The 3 shape-contract samples (minimal_module/net_alias/structured_decls) do NOT exercise stream rules ⇒ samples 3/3 aligned drift=0 (the shape change is documented in the manifest calibration_history, not sample-locked). FULL LOCKSTEP LANDED: SV integration contract (Identity versions 1.0.140 + schema-4 note + 1.0.140 Highlights), `PGEN_RELEASED_PARSER_BUG_LEDGER` row `SV-0002`, SV parser book (changelog-index 1.0.140 + schema-versioning row schema-4 + welcome stale `1`→`4` drift fix), the top-level `docs/book/src/grammar-wellformedness.md` SV-drive narrative (brought current with BOTH the `90→89` `-0093` step — prior generator-only book-debt — and this `89→88` step), CHANGES/DEVELOPMENT_NOTES/LIVE_ACHIEVEMENT_STATUS/MEMORY. PROOF MATRIX (all green): cert seeds 0/7/42 (array_range closed, no GLOBAL regress); `--lint-grammar` clean (pre-existing `always_matches=6` A2 backlog unchanged); SV external corpus 14/14 (`parse_pass_total: 14, parse_fail_total: 0`); probe matrix all-accept incl. `{<< 4 {a with [3:0]}}`; `stimuli_cross_family_platform_gate` PASS; clippy source-clean (generated-parser clippy non-strict tolerated as established). LESSON (KM-worthy): restoring a dropped mandatory delimiter that shares a prefix (`{`) with an adjacent OPTIONAL constant_expression can silently regress a common form via PEG greedy-optional non-backtracking — always probe the canonical real-world forms after a delimiter restoration; the `&<following-mandatory-delimiter>` guard is the LRM-grounded fix. [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_why_and_where_before_solution]], [[feedback_corpus_expected_from_spec_not_fix]].
- `2026-06-16` (`H.12.5.5.3.3.5` C-i stream delimiter-drop FIX DESIGN, `-0095`, PURE-DOCS): **the grammar fix is a SHAPE-changing edit (positional-capture shift), not just a delimiter add — so its annotations must be re-expressed and the proof must carry the full grammar-edit lockstep.** Exact edits (`grammars/systemverilog.ebnf`): (1) `stream_concatenation := lbrace stream_expression ( comma stream_expression )* rbrace` — the restructure shifts captures to `$1=lbrace $2=first-stream_expression $3=( comma stream_expression )* $4=rbrace`, so the current `-> {body: $1}` (the whole repetition) must become **`-> {body: [$2, $3::2*]}`** (the proven Cat-A `X (sep X)*` extraction-spread idiom, [[feedback_quantified_group_extraction]]; `$3::2*` extracts element-2 = `stream_expression` from each `( comma stream_expression )` rep). (2) `stream_expression := expression ( kw_with_8fcd25a3 lbrack array_range_expression rbrack )?` — `$1=expression $2=( … )?` unchanged, so `-> {expr: $1, with_clause: $2}` is position-stable, but the INNER shape of `with_clause` changes (now `[kw_with, lbrack, array_range_expression, rbrack]` vs `[kw_with, (array_range_expression)?]`). `streaming_concatenation` (`:4787`, `-> {op:$2, slice_size:$3, body:$4}`) is unchanged. CONSEQUENCES (full lockstep, [[feedback_grammar_edit_proof_gate_lockstep]]): SHAPE change ⇒ schema bump + `rust/test_data/ast_shape_contract/systemverilog*.json` manifest update (alphabetical, [[feedback_manifest_alphabetical_order]]); LANGUAGE change (rejects old bare `{>> a with b}`, accepts LRM `{>>4{a with [b]}}`) ⇒ SV release bump + `PGEN_RELEASED_PARSER_BUG_LEDGER` row (it fixes a real parse bug — current grammar REJECTS valid SV) + SV integration-contract + SV parser-book changelog + the grammar-wellformedness book chapter's SV drive narrative. Pre-edit consult `grammars/return_annotation.ebnf` + the annotation refs ([[feedback_ebnf_consult_annotation_docs]]); verify with `parseability_probe --parse-dump-ast-pretty`. Proof matrix: regen SV parser (DEBUG `ast_pipeline`) → cert seeds 0/7/42 (array_range_expression closes `UNKNOWN 89→88`? + GLOBAL no-regress on the other 88) → external corpus 14/14 → `stimuli_cross_family_platform_gate` → `--lint-grammar` (no new shadow/orphan) → `clippy_on_rust_change`. Change ONE thing; measure GLOBAL.
- `2026-06-16` (`H.12.5.5.3.3.2` C-iv RE-ADJUDICATION, `-0094`, PURE-DOCS INVESTIGATION): **`array_range_expression` is NOT a C-iv generator reach-path gap — it is a C-i GRAMMAR delimiter-drop defect (LRM-confirmed); and the C-iv leaf's other carrier `sequence_method_call` was already closed by `-0093`, so the C-iv leaf `H.12.5.5.3.3.2` closes with NO generator change.** Tools-first (DEBUG `ast_pipeline` `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` + `PGEN_REACH_PATH_DUMP=1` on the canonical SV cert seed 0 `total=1292 witness=1202 UNKNOWN=89 spf=0`, then `parseability_probe --parse-dump-ast-pretty` / `--trace-rules`): the reach plan forces `stream_expression`'s `( kw_with ( array_range_expression )? )?` optionals, but on re-parse the host `streaming_concatenation`'s OFF-PATH `( slice_size )?` sibling steals the stream_expression's leading expression (-> `slice_size`), so `with` parses as a bare identifier and `kw_with`/`array_range_expression` are never entered (trace: 0 entries vs 110 for the witnessing two-expression `{>> e1 e2 with e3}` form, which enters `…streaming_concatenation->stream_concatenation->stream_expression->array_range_expression`). ROOT CAUSE (`systemverilog_2017_lrm_extracted.ebnf:1041/1044`): `stream_concatenation` dropped its literal `{ }` braces and `stream_expression` dropped its literal `[ ]` brackets in LRM extraction (the documented dropped-delimiter class), which ALSO makes the grammar REJECT valid SV (`{>>{aa with bb}}`, `{>>4{aa with [bb]}}`). A generator force-`slice_size` workaround is rejected per the fix hierarchy (grammar-FIRST attribution rule). RE-ROUTED to new C-i grammar-restoration leaf `H.12.5.5.3.3.5`. VERIFIED: baseline cert re-measured `total=1292 proof=1 witness=1202 UNKNOWN=89 spf=0` (seed 0) — unchanged (pure-docs, no code touched). [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_no_workarounds_fix_hierarchy]], [[feedback_why_and_where_before_solution]], [[feedback_prefer_grammar_leave_engine_alone]].
- `2026-06-16` (`H.12.5.5.3.3.1` C-ii IMPL, `-0093`, GENERATOR-ONLY): **the mandatory-inner-structure forcing capability landed (SV cert `UNKNOWN 90→89`), but it closed a DIFFERENT carrier than hypothesized — and proved the 2 named C-ii carriers are parent-commit-HARD.** New `mandatory_child_rules` walker + a purely-additive child-forcing loop in `generate_target_own_structure_witnesses` (runs only when R-own forcing failed; per-child budget; keyed `(child, node_path)` — confirmed the read-sides fire on the rule currently being generated, e.g. forcing `(constant_bit_select, …)` DID render `[idx]` and forcing `(method_call_body, …)` DID render a call form). The capability is general + strictly additive + inert for the fully-certified roster (rtl_const_expr/json `UNKNOWN=0`, no slowdown). **HONEST OUTCOME:** it witnessed `sequence_method_call` (a `-0091` C-iv carrier whose mandatory child needed structure forced), NOT `direct_index_method_call`/`context_member_method_call`. Re-parse (`parseability_probe --parse-dump-ast-pretty`) proves the 2 named carriers stay UNKNOWN because their forced distinguishing structure is STILL re-attributed to a sibling at the PARENT ordered choice (`direct_index` → `split_direct_callable_method`/`method_call` in `bit_select_expression`; `context_member` → `call_primary`/attribute-spec) — the parent-commit-HARD class, not closable by structure forcing alone. Reclassified to `H.12.5.5.3.3.4` (parent-commit forcing / `bit_select_expression` ordered-choice-shadowing adjudication). LESSON: the `-0091` C-ii vs C-iv boundary is fuzzy — "distinguishing structure lives in a mandatory child" is necessary but NOT sufficient when the parent ordered choice re-attributes even the distinguishing form; the discriminator is whether the forced form is still sibling-absorbed (re-parse to check). VERIFIED: cert seeds 0/7/42 identical (`1202/89/spf0`); `mandatory_child_rules` + `target_own_reach_sites` unit tests PASS; `stimuli_cross_family_platform_gate` PASS; clippy source-clean. GENERATOR-ONLY ⇒ no parser-regen/grammar/release/schema/ledger change ([[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_ast_pipeline_parser_agnostic]], [[feedback_always_signoff_decisions]] — report only what the tool showed).
- `2026-06-16` (`H.12.5.5.3.3.1` C-ii DESIGN, `-0092`): **the C-ii fix is a bounded mandatory-reference descent on top of the `-0090` pass — and the read-sides already support it, so it needs NO read-side change.** Tool-backed (the `--dump-gen-ast` transform): a rule reference is a LEAF token `Atom(Token(["rule_reference", name]))`, NOT an inlined subtree, so the `-0090` walker (Or/Sequence/Quantified/`Atom::Node` only) never reaches the distinguishing structure of `direct_index_method_call` (`method_call_body`'s call Or, `:2793`) or `context_member_method_call` (`constant_bit_select := (lbrack constant_expression rbrack)*` `:1262`, a min-0 `*` that renders empty); both carriers' body root is a `Sequence` (`root_or=None`), so `-0090` had nothing to force. KEY finding: `generate_or`/`generate_quantified` read `forced_branch_for(current_rule, node_path)` / `forced_quantifier_min[(current_rule, node_path)]` where `current_rule` tracks the rule being generated — so a directive keyed `(method_call_body, …)` / `(constant_bit_select, …)` fires when the generator descends that child (exactly how `set_reach_plan_for_rule` already forces rules along the reach path) ⇒ NO read-side change. FIX = a new bounded MANDATORY-reference walker (Sequence elements + min-1 `+`-group elements + `Atom::Node` groups + `rule_reference` tokens; SKIP min-0 quantifiers, un-forced `Or` branches, lookaheads) feeding `target_own_reach_sites(C)` forcing per mandatory child `C`, installed alongside R's directives in `generate_target_own_structure_witnesses`; parser-judged + residual-only PASS 3c + strictly additive keep it inert/safe. Open at impl: child-branch selection (try o1.. then o0, like R's root) + escalation bounding (start depth-1 direct children — sufficient for both carriers). Design note: [GRAMMAR-WELLFORMED-H125331-cii-mandatory-inner-design.md](GRAMMAR-WELLFORMED-H125331-cii-mandatory-inner-design.md). PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change ([[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_why_and_where_before_solution]], [[feedback_ast_pipeline_parser_agnostic]]).
- `2026-06-16` (`H.12.5.5.3.3` M1b RESIDUAL WHY+WHERE, `-0091`): **the 6 carriers `-0090` could not close are a CONFIRMED parent-commit problem, and they split into 4 distinct fix mechanisms — not one.** Tools-first decisive discriminator: re-parsing each carrier's own generated witness (`parseability_probe --parse-dump-ast-pretty`) shows all 6 parse OK (full consume) yet contain ZERO nodes of the carrier rule ⇒ a sibling genuinely absorbs the bytes (this is NOT the `H.10.2.2` memo-hit coverage-record bug — there the rule appeared in the AST). PARTITION with grammar-grounded WHERE: **C-i operator-shadow** `goto_repetition` (bare `-> expr` eaten by the `operand_chain` `implies` operator; grammar lacks the LRM `[ ]` brackets — LRM-ground first); **C-ii mandatory-inner-structure-not-forced** `direct_index_method_call` (needs `method_call_body` `(args)`) + `context_member_method_call` (needs the `+`-group's `constant_bit_select [idx]`) — the `-0090` walker forces root-`Or`+optionals but not mandatory sub-rule content; **C-iii store-gated** `class_scoped_tf_call` (`class_scoped_call_prefix` gated on known-class identifiers — store-faithfulness, RECLASSIFIED to `H.12.5.6`/M2, removed from M1b); **C-iv reach-path-selection** `sequence_method_call` (reach landed in a module named-port-connection host) + `array_range_expression` (alias `:= expression`; reach must force the parent `stream_expression … with (…)?` optional, not deep-force the alias body which `-0090` made `parsed=false`). Spawned fix children `.3.3.1` (C-ii, generator, reuses `-0090` machinery — first), `.3.3.2` (C-iv, reach-honesty `RTL-FE-CLOSURE.5.3` lineage), `.3.3.3` (C-i, LRM-ground the bracket question first). Detail note: [GRAMMAR-WELLFORMED-H12533-m1b-residual-parent-commit-whywhere.md](GRAMMAR-WELLFORMED-H12533-m1b-residual-parent-commit-whywhere.md). PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change ([[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]]).
- `2026-06-16` (`H.12.5.5.3.2` THE M1b FIX, `-0090`): **the M1b "reach forces the path but not the target's own distinguishing structure" residual is fixed by a new parser-agnostic target-own-structure reach pass — SV cert `UNKNOWN 93→90` (witness `1198→1201`, spf=0, deterministic seeds 0/7/42).** The pass forces a still-`UNKNOWN` target `R`'s OWN root-`Or` branch (`o1..` first, degenerate `o0` last) + every `?`/`*` inside `R`'s body (≥1), on top of the base reach plan to `R`'s reference site, so `R` renders a distinguishing (not sibling-ambiguous minimal) form. **Architecture decision (cross-grammar cost):** the escalation is wired as cert-driver **PASS 3c** over ONLY the rules still UNKNOWN after passes 1+2+3 — NOT inline in the plannable pass. The first cut WAS inline (fired for every plannable target not witnessed by its own probe) and, because it is parser-agnostic, ran for `rtl_const_expr` too — adding tens of seconds of waste to that already-deep-recursive grammar's cert (it fired for rules already globally witnessed-via-collateral, which M1b cannot help). A/B with a kill-switch isolated it. Moving M1b to a residual-only driver pass makes it **truly inert** for any grammar the existing passes already certify (empty residual ⇒ zero probes) while preserving the SV win — the grammar-neutral fix the parser-agnostic doctrine demands (no SV carve-out). NEW `target_own_reach_sites`/`collect_optional_quantifier_paths` walker (structural, `(R, node_path)`-keyed) + `generate_target_own_structure_witnesses` (`stimuli_generator.rs`) + PASS 3c (`main.rs`) + 1 unit test. Closed 2 named carriers (`bit_select_expression`, `constant_let_expression`) + 1 collateral; the residual 6 carriers stay UNKNOWN (their distinguishing form is sibling-absorbed at the PARENT regardless of `R`'s body — a parent-commit problem split out as `.3.3`). STRICTLY ADDITIVE (only unions witnesses ⇒ the `888→1717` guard holds). VERIFIED: cert seeds 0/7/42 identical, `stimuli_cross_family_platform_gate` PASS, strict clippy clean, unit test PASS, `rtl_const_expr`/`json` byte-identical `UNKNOWN=0`. GENERATOR-ONLY ⇒ no grammar/parser-regen/release/schema/ledger change ([[feedback_ast_pipeline_parser_agnostic]], [[feedback_no_codebase_change_without_tool_backed_facts]]).
- `2026-06-16` (`H.12.5.5.3.1` M1b WHY+WHERE, `-0088`): **the M1b residual is NOT a literal short-circuit (M1a) — it is that the reach forces the path to the target but NOT the target's OWN distinguishing structure, so the target generates minimally into a sibling-ambiguous form.** Tools-first (`PGEN_REACH_PATH_DUMP`/`DEBUG_PROBES`/`DUMP_ALL` + `--dump-gen-ast`, post-`-0087` baseline `witness=1198 UNKNOWN=93 spf=0`): the M1b carriers (`array_range_expression`/`bit_select_expression`/`direct_index_method_call`/`goto_repetition`/`context_member_method_call`/`constant_let_expression`/`class_scoped_tf_call`/`sequence_method_call`) all reach the right context with REAL organic samples but `witnessed_target=false`. The transformed tree shows the reach forces the target's PARENT branch (e.g. `call_primary`=`Or[10]`, target at `o0`), but the target rule itself generates minimally: **B-i** its root `Or`'s `o0` is a degenerate pass-through a sibling also accepts (`array_range_expression` `o0=expression`), or **B-ii** its distinguishing tokens are behind a minimal-expanded optional (`context_member_method_call`'s `.method(args)`). WHERE = `directives_along_path`/`set_reach_plan_for_rule` force ALONG the path but stop AT the target. FIX (`.3.2`): steer the target's OWN distinguishing structure (root `Or`→non-degenerate branch + distinguishing quantifiers ≥1, parser-judged bounded retries), the `RTL-FE-CLOSURE.5.3` lineage extended to the target's body — measured GLOBAL cert + spf + cross-family since it touches the closed-loop driver. Split `H.12.5.5.3` → `.3.1` investigation (this) + `.3.2` fix. PURE-DOCS ([[feedback_why_and_where_before_solution]], [[feedback_no_codebase_change_without_tool_backed_facts]]).
- `2026-06-16` (`H.12.5.5.2.2` THE M1a FIX, `-0087`): **the M1a header-reach gap is the RULE-LEVEL analogue of `H.12.3`'s branch-level `@sample` stand-down — rebuilt on the TRANSFORMED tree, fixed, SV cert `UNKNOWN 121→93`.** The `-0086` discrimination correctly refuted the forced-quantifier model and prescribed the transformed-tree rebuild; doing so showed: (1) the transformed `ansi_port_declaration` (`--dump-gen-ast`) is an `Or` whose branch `o2` (named-port `port_direction? . port_identifier ( expression? )`) holds `expression` at `root/o2/s4`; (2) the `-0085` reach path routes every M1a expression target through `module_ansi_header`, which carries a RULE-LEVEL `@sample:"module m(input logic a);"` (in the dump's `semantic_annotations` bucket); (3) the `-0085` witness `module m(input logic a);endmodule` is UNIQUELY that literal + `module_declaration_sv_2017`'s minimal body, proving the rule-level `@sample` fires during the M1a plan and short-circuits the body before the port descent (so `-0086`'s `apd_keys=[]`/`o2/s4`-never-reached is the same fact). WHERE: `generate_rule:6115` (rule-level override) had NO reach-plan stand-down, unlike `generate_or:6768` (`H.12.3` branch-level). FIX: `ActiveReachPlan::needs_rule_body_descent(rule)` (true when the plan forces any directive/quantifier keyed on the rule) gates the rule-level `@sample`/`@probe_sample` override to STAND DOWN when the plan must descend through the rule's body — keyed purely on the plan's forced keys (parser-agnostic), off-reach byte-identical, certified roster + `@sample`-free grammars inert. VERIFIED green: cert seeds 0/7/42 (`DUMP_ALL` = all M1a targets witnessed; residual 93 = 20 `no_path` non-defects + M1b + M2 + M3); unit tests 4/4; clippy source-strict clean; `stimuli_cross_family_platform_gate` PASS. GENERATOR-ONLY ⇒ no parser-regen/grammar/release/schema/ledger change. Lesson reinforced: model the generator on the TRANSFORMED `grammar_tree`, not the EBNF source ([[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_why_and_where_before_solution]]).
- `2026-06-16` (`H.12.5.5.2.2` discrimination, `-0086`): **the M1a port-quantifier does NOT fire — branch `o2` falls back before its `s4` expression quantifier, so the simple "force the port quantifier" fix is REFUTED.** A throwaway `[q-force]` trace in `generate_quantified` (gated on `PGEN_REACH_PATH_DUMP`, since reverted) measured: across the whole `count 40 seed 0` cert run, `generate_quantified` is entered for `current_rule==ansi_port_declaration` only 3× — at `root/o0/s0`/`root/o1/s0`/`root/o2/s0` (branch-START quantifiers), ALL `forced=None` with `apd_keys=[]` (the active plan holds NO forced `ansi_port_declaration` quantifier when the port renders). The forced port site `(ansi_port_declaration, root/o2/s4)` is never reached. ⇒ branch `o2` (expression-bearing) is abandoned for a simpler port branch BEFORE `s4`, so the target's expression OR-node is never entered — NOT hypothesis (a) forcing-gap NOR (b) backtrack-at-the-quantifier. The transformed `grammar_tree` `ansi_port_declaration` (`Or` with `expression` at `o2/s4`) is heavily restructured vs the source (no top-level `Or`, references `constant_expression`/`constant_range`/`net_port_type`, none `expression` directly), so the model must be rebuilt on the TRANSFORMED tree (dump it + re-trace with the plan target + `forced_branch_for(ansi_port_declaration, root)` to learn forced-then-fails vs inlined-context-can't-key). The likely fix is reach-path SELECTION toward the module-BODY carrier the M1b targets witness through, the `RTL-FE-CLOSURE.5.3`/`.5.4` family. PURE-DOCS (the trace was reverted) ⇒ no code/grammar/generated/release/schema/ledger change.
- `2026-06-16` (`H.12.5.5.2.1`, `-0085`): **the M1a reach carrier is the ANSI PORT, not the module body — a `PGEN_REACH_PATH_DUMP` observability tool pinpointed it.** Built an env-gated reach-path dump (`set_reach_plan_for_rule`, generator-only, byte-identical off) that prints the BFS hop chain `reach_hops` installs per target. It shows EVERY M1a expression target shares the shortest-HOP prefix `… → module_declaration_sv_2017 → module_ansi_header (root/s6/q) → list_of_port_declarations → ansi_port_declaration (root/o2/s4/q) → expression → …` — the reach steers the deep expression target through the ANSI port's unpacked-dimension/default, which renders minimally as `input logic a` (the on-path port-expression quantifier does not materialize) → `module m(input logic a);endmodule`, target never entered. The M1b targets that witnessed went through the module/program BODY (`assign …=<expr>`). KEY for `.2.2`: the `forced_quantifier_min` key `(current_rule, node_path)` (`generate_quantified:7705`) MATCHES `quantifier_sites_along_path`'s key, so the forcing should fire — the fix must first discriminate "forced quantifier not expanding" (a forcing gap) vs "expands then the deep expression backtracks" (depth/constant-context) before changing `reach_hops`/`set_reach_plan_for_rule`. OBSERVABILITY-ONLY (cert `UNKNOWN=121 witness=1170` unchanged with the dump on) ⇒ no parser-regen/grammar/release/schema/ledger change.
- `2026-06-16` (`H.12.5.5.1`, `-0084`): **the `-0083` M1 bucket (46 rules) is TWO reach mechanisms, not one — and the WHERE is `reach_hops`' BFS shortest-HOP path SELECTION, the `RTL-FE-CLOSURE.5.3`/`.5.4` family.** Tools-first (DEBUG `ast_pipeline --features ebnf_dual_run,generated_parsers`, `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` + `DUMP_ALL=1`, count 40 seed 0; deterministic baseline `total=1292 witness=1170 UNKNOWN=121 spf=0` reproduced BYTE-identical to `H.12.5.3` ⇒ the metric is SIGNAL). **M1a (reach dead-ends at the module/program header):** probe sample is a minimal `module m(input logic a);endmodule` (body `module_item*`→0, target OR-node never entered) — `cast`/`concatenation`/`conditional_expression`/`cond_pattern`/`cond_predicate`/`assignment_pattern_entry`/`inside_expression*`/`associative_dimension`/`empty_unpacked_array_concatenation`/`expression_or_cond_pattern`/`instance_or_class_scope`. **M1b (reach DESCENDS into a construct context but routes through a SIBLING):** `array_range_expression`/`bit_select_expression`/`direct_index_method_call` → `assign \foo.\foo[\foo].\foo=<num>`; `goto_repetition`/`class_scoped_tf_call` → `sequence …endsequence`; `context_member_method_call`/`constant_let_expression` → `(*\foo=+…*)`; `inout_declaration`/`callable_identifier`/`checker_instantiation` → a declaration routed elsewhere. **WHERE:** `reach_hops` (`stimuli_generator.rs:5186`) returns the BFS shortest-HOP path; `set_reach_plan_for_rule` (`:2686`) forces only the OR-branches (`directives_along_path`) + quantifiers (`quantifier_sites_along_path`) ON that path — so for M1a the shortest path can land at a header position that completes minimally without entering the target. Fix family = reach-path SELECTION honesty (prefer a path into a generatable expression position), the lineage of `.5.3` (non-self-recursive-branch preference) + `.5.4` (lookahead-skip). NOTE: `casting_type`/`constant_cast` are `parsed=false` ⇒ they belong to M2 (`H.12.5.6`), NOT M1. Split `H.12.5.5` → `.1` investigation (this, done) + `.2` M1a fix (frontier; starts with a reach-path-dump tool-build before the surgical change) + `.3` M1b fix. Next step for the M1a fix: BUILD the reach-path dump to read the exact `cast` BFS path BEFORE changing `reach_hops`, then change ONE thing and measure GLOBAL cert + spf at seeds 0/7/42 ([[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_why_and_where_before_solution]]).
- `2026-06-15` (`H.12.5.4`, `-0083`): **the SV `UNKNOWN=121` residual is THREE generator mechanisms, not one "reach-honesty" class — and M2 hides a possible parser-bug surface.** Tools-first cross-reference (reach probe outcome × the enumerated UNKNOWN list): **M1 reach-shell-fallback (46)** — expression-context rules whose reach probe `parsed=true witnessed_target=false` falling back to a `module …;endmodule` shell (the `H.12.3` short-circuit class, but the target is DEEPER than the forced branch so `H.12.3`'s exact-branch stand-down doesn't cover it); **M2 over-generation/store-unfaithful (40)** — `parsed=false` rejected witnesses, split M2a constraint/sequence/property bodies the generator emits structurally-invalid + M2b B1 store-gated identifier rules whose sample never establishes the store fact; **M3 no-plannable-probe (34)** — the 20 adjudicated `no_path`/blessed NON-defects + ~14 property/sequence temporal operators the plannable pass cannot synthesize a candidate for. KEY: M2's `parsed=false` samples REQUIRE a per-rule **over-gen-vs-parser-bug adjudication** before any generator fix — a rejected witness could be a released SV PARSER BUG (parser wrongly rejecting valid SV), which per the director's fix-parser-bugs-ASAP principle jumps priority. Split into fix children `H.12.5.5` (M1), `H.12.5.6` (M2), `H.12.5.7` (M3); none is a quick win — each is an engine/generator reach-or-faithfulness slice needing its own tools-first WHY+WHERE (the precise generator-function WHERE is owned by each fix child, NOT this classification).
- `2026-06-15` (`H.12.5.3`, `-0082`): **`comment_only_source_region` is engine-shadowed-dead too — not just `white_space` — so the B4 literal-0 removal is `−2`, not `−1`, and the `@sample`-coverable path is FALSIFIED.** The `H.12.5.1` classification flagged `white_space` as engine-shadowed-dead (whitespace skipper) and `comment_only_source_region` as a "removal candidate, edit larger." Tools-first this session: the cert reach probe returns `parsed=true witnessed_target=false` for BOTH (`white_space` on `"     "`, `comment_only_source_region` on `"//x\n"`), AND a direct parse of six comment-only/comment-containing inputs yields ZERO `comment_only_source_region` nodes (a comment-only file → `source_text: []`) — i.e. the layout skipper consumes COMMENTS as leading trivia before `source_text_item`'s branch is tried, exactly as it consumes whitespace before `white_space`. So `comment_only_source_region` can NEVER be witnessed by any `@sample` (the leaf's path-(a) is refuted), and the correct resolution is the same literal-0 **delete the orphan** path as `white_space` — removing the rule AND its dead `source_text_item` branch (`@priority` shrinks `[…,6,4]`→`[…,4]`). This is accept-identical (neither node ever appeared in any AST ⇒ wire-shape identical; the declared `source_text_item` union narrows 8→7 kinds, a never-emitted-variant correction) ⇒ NO release/schema bump, matching the vhdl `H.11.4-UNPARK` `white_space` precedent ([[project_ebnf_is_single_source_of_truth]], the Hopcroft-Ullman reduced-grammar requirement). The closure contract is re-baselined v4→v5 (floors −2, `max_unreachable_branches` 11→2) leaf-owned per its own drift policy. Measured: SV cert `UNKNOWN 123→121`, witness `1170` byte-identical, spf=0 seeds 0/7/42; external corpus 14/14.
- `2026-06-15` (`H.12.5.2`, `-0081`): **the SV `interface_class_declaration` sv_2017 `no_path` is NOT a defect — it is LRM-faithful; do NOT wire interface classes into the sv_2017 profile.** Tools-first refutation of the `H.12.5.1` "candidate profile-orphan defect" flag: IEEE **1800-2017** defines `interface_class_declaration` (§8.26.1) but its formal grammar does NOT reference it from `class_item` / `package_or_generate_item_declaration` / `anonymous_program_item` (those carry `class_declaration` only) — the 2017 LRM orphans it in the source-text item hierarchy; **1800-2023** wires it in (`class_item`/`package_or_generate_item_declaration`/program items each add `| interface_class_declaration`). PGEN faithfully mirrors both (the 5 A2 rules are referenced only from the `*_sv_2023` variants), so under `--grammar-profile sv_2017` they are correctly unreachable and are WITNESSED under `--grammar-profile sv_2023`. Wiring them into sv_2017 would DIVERGE PGEN from the 1800-2017 LRM ([[project_ebnf_is_single_source_of_truth]]). CONSEQUENCE: the 20 SV no_path are fully adjudicated NON-defects (11 A1 entry-relative + 5 A2 profile-relative + 4 blessed); the SV `fully_certified` endgame needs a multi-profile/multi-entry accounting for them, not an sv_2017 reach fix.
- `2026-06-10`: **META-GRAMMAR COMPLETENESS AUDIT (director question after the `-0067` `**`-gap
  discovery: "are there other EBNF-format enhancements not ported to `ebnf.ebnf`?") — answer: YES,
  six gaps; the audit is the scoping evidence for the ticketed ebnf.ebnf leaf.** Method (tools-first,
  executable): the GENERATED ebnf parser (`ebnf_dual_run_diff --input <g>.ebnf`, parse_full) run over
  all 12 shipped grammars + ~30 single-construct minimal probes (each construct in isolation, so the
  first-failure masking of whole-file runs cannot hide later gaps). WHOLE-FILE RESULT: 6/12 grammars
  FULL-FAIL (regex@1307, semantic_annotation@7166, rtl_frontend@1132, systemverilog@9676, svpp@717,
  vhdl@529); 6 pass (ebnf, json, return_annotation, builtin_×2, rtl_const_expr — note
  return_annotation.ebnf contains `**`/`::N*` only as QUOTED TOKENS of the language it defines, so it
  parses). CONSTRUCT VERDICTS — **GAPS (probe FAIL)**: (1) **per-branch return annotations**
  (`A -> ann | B -> ann`, same-line AND continuation-line — `rule_definition` models ONE rule-level
  `return_annotation?`; THE DOMINANT GAP, the first-failure of 5 of the 6 failing files; usage
  ubiquitous — SV/vhdl/regex/rtl_frontend/svpp/semantic_annotation); (2) **`::N*` extraction-spread**
  (152 live sites: SV 115, vhdl 17, rtl_frontend 17, return_annotation-as-tokens 5, svpp 2, builtin 1);
  (3) **`**` flatten-spread** (the -0067 discovery; live in regex `[$1**]`); (4) **lexical
  annotations `[> …]`/`[>! …]`** (FAIL@0; the 4th-pillar follow-restrictions, 12 live sites in svpp);
  (5) **dotted `$refs` in return annotations** (`-> {x: $1.body}` / `$name.body`; 25 live SV sites +
  2 regex); (6) **indexed `$refs` in return annotations** (`-> {x: $1[0]}`; zero live return-side
  usage today — semantic payloads parse as balanced text and are unaffected). **PORTED/OK (probe
  PASS)**: `$text`/`$0`, bounded quantifiers `{N}`/`{N,M}`, lookahead `&`/`!`, raw strings, regex
  flags, plain multi-line alternation, parens-group broadcast `( A | B ) -> ann`, inline branch-local
  `@sample`, spread-property `...$1`, and every semantic-directive payload form probed (`@sample`,
  `@probe_sample`, `@transform`, `@predicate` colon-less + payload forms, `@emit_fact`, `@fact_kind`,
  `@predicate_def`, `@profiles`, `@branch_policy`, `@open_scope`/trailing `@close_scope`,
  `@export_to_library`, `@semantic_value`). CAVEAT recorded honestly: probes verify parse ACCEPTANCE;
  a passing construct could still mis-shape (raw-AST parity is the dual-run gate's deeper layer) —
  shape parity is part of the future leaf's acceptance. IMPACT framing: production compilation uses
  the hand-written frontend, so nothing user-facing is broken TODAY; the gaps block the self-hosting
  doctrine (generated EBNF parser taking over) and keep `ebnf_frontend_dual_run_gate` red. The future
  leaf = port the 6 constructs into `grammars/ebnf.ebnf` (grammar-level work; per-branch annotations
  + extraction/flatten spread markers + `[>`/`[>!` + dotted/indexed return refs), then the dual-run
  gate goes strict-green and the `ebnf` family becomes Phase-H-wirable. Probe corpus retained at
  `/tmp/h113_p_*.ebnf`; recorded `PGEN-GRAMMAR-WELLFORMED-0068` (pure docs).
- `2026-06-10`: **H.7.1's Q1–Q3 resolved by the agent** under the director's standing "PNT yourself —
  do not involve me unless you can't decide" instruction + [[feedback_user_is_director_not_engineer]]
  (pure technical choices, no real-world side effects). Q1: MVP = optional+alternation class only
  (rule-level `reachable_rule_not_generated` = follow-up). Q2: superseded BOTH proposed options — no
  config flag at all; the forced-quantifier state is carried BY the plan
  (`ActiveReachPlan.forced_quantifier_min`, empty for every existing caller) and the pass is invoked
  explicitly by the report, so the no-op invariant holds structurally. Q3: per-rule ≤4 witness-checked
  attempts à ≤250ms + a 4096-rule global cap with a loud left-unattempted WARNING (never silent).
  Recorded in the `H.7.2` leaf; the implementation commit is `PGEN-GRAMMAR-WELLFORMED-0058`.
- `2026-06-06`: **Extended director brainstorm on linter TRUSTWORTHINESS** → Phase G (the certifying
  linter) + [[feedback_unreachable_target_attribution_rule]] + [[feedback_certifying_linter_trustworthiness]].
  Crystallized: the linter is the fulcrum (prover + adjudicator + theorem-maker) → must be never-doubted
  → achieved via a certifying algorithm (witness/proof certificates + independent checker), sound-not-
  complete (exact reachability undecidable for data-dependent PEG), `UNKNOWN` drained to 0 on the shipped
  grammar. Logged in the top-level book ("Trusting the linter: certificates, not faith" + "The attribution
  rule" + "Worked example"). Director: "we can't afford to doubt the grammar linter."
- `2026-07-06` (cross-link, session #50): the **undefined-reference** well-formedness gate — the
  "no dangling references" half of contract item 4 — LANDED via the sibling tree
  [`UNDEFINED-REF-DIAGNOSTICS`](UNDEFINED-REF-DIAGNOSTICS.md) (F6, `PGEN-UNDEFINED-REF-DIAGNOSTICS-0002`):
  `detect_undefined_references` in `grammar_wellformedness.rs` (the structural DUAL of
  `detect_unreachable_rules`), `[error]` hard-gated in `--lint-grammar`, allowlist = codegen's
  `NATIVE_UNRESOLVED_REFERENCE_BUILTINS` const (oracle-locked to the dispatch), run on the
  UNFILTERED bundle (codegen compiles the full grammar), plus an unconditional codegen warning at
  stub emission. This tree remains the charter owner of the wellformedness contract; that leaf's
  detector composes with A1b's reachability exactly as Hopcroft–Ullman's two "no useless symbols"
  halves.
- `2026-06-05`: Created from the director brainstorm. The frame UNIFIES the linter (static proof) +
  the stimuli generator (constructive proof) of reachability. Cross-refs: `PARSE-SOTA` (existing
  lint checks A1/.9), `SV-EXH-PROOF.7` (the generator/literal-0 consumer), `PARSE-TERMINATION`
  (`non_terminating`). The de-dup of 25 dead branches (`SV-EXH-PROOF.7.4.6.7`) was the first
  embodiment; this tree generalizes it into a proof.
