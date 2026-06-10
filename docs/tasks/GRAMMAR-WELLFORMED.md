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
| 1 | `GRAMMAR-WELLFORMED.A2.1` | `in-progress` (always_matches **52→8**; clean families done) | Clean the SV `always_matches` defects LRM-grounded → promote EarlierAlwaysMatches to the hard gate when 0. ✓ boolean-abbrev (`-0010`), ✓ covergroup-range + rs-prod (`-0013`), ✓ formal-type/port-reorder + list-of-arguments + module-path + bins_or_empty + class_declaration (`-0014`). **SYSTEMATIC ROOT CAUSE: dropped-delimiter + lost-ordering extraction artifacts** (`[ ]`/`{ }` lost → nullable wrappers; LRM CFG order needs PEG specific-before-general reorder). **RESIDUAL 8 (deep, DEFERRED):** (4b) port-header/net-type family (6) needs nettype/interface STORE-GATING (identifier ambiguity, [[feedback_grammar_rules_must_consult_store]]); `sv_multi_entry_root` (2) needs the entry-declaration (A1b.1) / linter-exempt. A2 stays warning-staged until these 8 resolve. |
| 1 | `GRAMMAR-WELLFORMED.G` | `in-progress` (G.1 done `-0012`) | **The CERTIFYING LINTER** — make every verdict carry a checkable certificate (witness/proof), build the independent checker, drive `UNKNOWN`→0 on SV. "Verified, not trusted." ✓ G.1 certificate model + independent re-checker for unreachability proofs (round-trip + tamper-rejection tested). NEXT: G.2 standalone checker + extend certs to all `dead` checks; G.3 generator witnesses; G.4 coverage gate. |
| 1 | `GRAMMAR-WELLFORMED.H` (Phase H per-grammar cert-coverage) | `in-progress` (all SHIPPED grammars wired) | Wire `parse_and_cover` for every grammar so `--report-certificate-coverage` runs per-grammar. ✓ H.1 regex (`-0034`, UNKNOWN residuals + 6→3 witness-parseability), ✓ **H.2 vhdl (`-0041`, cert-coverage runs at default depth; zero-drift checkout-illusion proof discharged the staleness fear — `total=217 witness=132 UNKNOWN=85 sample_parse_failures=0` @ seed 0)**, ✓ **H.3 json (`-0037`, `fully_certified=true` — the FIRST grammar fully certified via Phase H)**, ✓ **H.4 rtl_const_expr (`-0038`, cert-coverage runs; zero-drift regen proof retires the H.2 mtime-staleness fear)**, ✓ **H.5 svpp (`-0039`, cert-coverage runs at default depth)**, ✓ **H.6 rtl_frontend (`-0040`, cert-coverage runs at default depth)**. **MILESTONE: every SHIPPED parser grammar now runs under cert-coverage** (json/regex/rtl_const_expr/svpp/rtl_frontend/systemverilog/vhdl); only meta/annotation grammars (`ebnf`/`return_annotation`/`semantic_annotation`) remain unwired. ✓ **H.5.1 (`-0042`) LABELED the svpp residual + H.5.1.1 (`-0044` investigation / `-0045` fix) ROOT-CAUSED + FIXED it: surgical whitespace-only greedy-tail guard in `regex_tail_greedy_blocker` → svpp `sample_parse_failures` 24→8, `UNKNOWN` 54→7, `witness` 19→66; zero cross-grammar regression (cross-family gate PASS).** ✓ **H.5.1.2 (`-0046`) drove svpp residual-8 CLASS (a) — the `\b`-keyword↔word-char directive-keyword fusion — to 0 via the declarative `[>! /\w/]` lexical-annotation (the construct built for the generator) + a general `collect_rule_body` frontend fix it surfaced (consecutive `[>` directives now each bind; only the first bound before); svpp `sample_parse_failures` 8→1, `UNKNOWN` 7→4, `witness` 66→69 seed 0; json/regex cert-coverage unchanged; lib 621/621; cross-family gate PASS.** ✓ **`H.5.1.3.2` (`-0049`) CLOSED the LAST svpp residual** — `condition_text -> $text` (declarative atomicity, LEXICAL-ANNOTATIONS.6) suppresses the stray trailing `\n` that stranded a `` `" `` stringize; svpp cert-coverage `sample_parse_failures` **1→0** (both seeds, deterministic) ⇒ **svpp is now cert-coverage CLEAN**. Consumer-visible: svpp schema **3→4**, release **1.0.4→1.0.5** (condition_atom "text" body raw-envelope→`$text` string; annot 66→67; director-approved). ✓ **`H.4.1` (`-0050`) ROOT-CAUSED rtl_const_expr's `UNKNOWN` residual** (the FIRST per-grammar `UNKNOWN`→0 drive, tools-first, pure-docs): the residual reduces to the stubborn pair `lparen`/`rparen` = the `primary_expr := lparen conditional_expr rparen` parenthesised-primary branch, which clean diverse generation essentially NEVER selects (`0/40` samples contain `(` @ depth 32; the branch re-enters the ~15-deep precedence chain → depth-floor pruning + recursion-pressure penalty avoid it; fatal-aborts at the default depth 24). ADJUDICATED a **generator-reach deficiency** (statically reachable; `(1)` is valid) — fix belongs in the generator. The witness-pass shortcut is off the table per the explicit `main.rs:1572` design decision. ✓ **`H.4.2` (`-0051`) DONE — CONSTRUCTIVE-REACH: rtl_const_expr is now `fully_certified=true` (UNKNOWN 3→0, deterministic across seeds), with ZERO certification regression on any grammar** (decisive git-stash baseline: `sample_parse_failures` byte-identical pre/post for json/regex/vhdl/SV; UNKNOWN only decreases — regex 101→98, vhdl 85→69, SV 1160→1126). Opt-in `StimuliConfig.reach_uncovered_recursive_branches` (default OFF → all non-cert-coverage surfaces byte-identical) drives three gated `generate_or` behaviours (floor-retain + try-recursive-first + minimal-`construct_mode` depth-retry); `run_certificate_coverage_report` is two-pass (diverse certification pass byte-identical + auxiliary reach pass that only UNIONS re-parsing witnesses). lib 686/0; new test PASS; self-host + cross-family + oracle green. ✓ **`H.5.2` (`-0052`) drove svpp `UNKNOWN 3→2`** — removed the OBJECTIVELY-PROVEN-DEAD `trivia` rule (referenced by nothing; gap-report oracle `reachable:false unreachable_from_entry`; the only statically-unreachable rule) at source per the literal-0 doctrine, and tightened the `sv_preprocessor_zero_plausible_gap_proof_gate` from a `[trivia]` helper-pocket to a **literal-ZERO unreachable surface** (contract v2→3, observed==allowed==[]; gate GREEN). cert-coverage `total 72 witness 70 UNKNOWN 2 sample_parse_failures 0` (seeds 0/7); shape-contract GREEN (no AST/schema/release change); lib 716/0. svpp's remaining 2 `UNKNOWN` (`directive_tail`/`line_comment`) are reachable optionals = generator-reach → **`H.5.3`** (svpp fully_certified after it). ✓ **`H.5.3` (`-0053`) DONE — svpp `fully_certified=true` at seeds 0/7/42** via DECLARATIVE witnessing-sample steering (evidence-driven re-scope from the assumed constructive-reach engine pass): the 2 residuals were 100% generator-side, caused by stale `@sample: " "` hints (un-witnessable bare space / `line_comment?`-short-circuit), replaced with witnessing-and-faithful `@sample: " x"` / `@sample: " //"`; `sample_parse_failures=0`, deterministic, zero cross-grammar regression (grammar-only). Multi-seed measurement surfaced TWO pre-existing svpp residuals (the stimuli generator as bug-finding oracle): (1) a seed-1/12-only macro-default nested-optional UNKNOWN — but a SAMPLE-BUDGET artifact (count 100/200 → `UNKNOWN=0`) → ticketed **`H.5.4`** (low priority); (2) a genuine OVER-GENERATION (`sample_parse_failures=1` at seeds 3/6/10/12/14/15 of 0–15, IDENTICAL on the pre-H.5.3 grammar → pre-existing, an unclosed/closer-stolen `pp_conditional` round-trip hazard) → ticketed **`H.5.5`** (the real round-trip defect). ✓ `H.5.5` + `H.5.4` closed (svpp seed-robustly clean via `H.5.5`/`H.7.2`/`H.9`). ✓ **`H.10.1` (`-0060`) — the regex `UNKNOWN`→0 drive's dead-rule removal: the 12 no-path rules (both oracles: `unreachable_from_entry`) removed at source; regex `UNKNOWN 19→7`, `total 210→198`, spf=0, deterministic seeds 0/7/42; gap-report unreachable 12→0; oracle gate + lib 721/0 green; no release/schema bump.** ✓ **`H.10.2.1` (closed via `BRANCH-BROADCAST-FIX.5`, regex `UNKNOWN 7→5`).** ✓ **`H.10.2.2` (`-0061`) ENGINE FIX — memo-hit coverage-delta replay (the memoization × coverage-record composition gap, the `.36.4` class on the witness record): regex `UNKNOWN 5→3` + CROSS-GRAMMAR vhdl `31→30`, rtl_frontend `75→73`, SV `738→647` (−91), spf byte-identical 0 everywhere, oracle gate PASS, NO bump.** ✓ **`H.10.2.3` (`-0062`) unicode_char witnessed via the declarative `@sample: "é"` (the rule was ungeneratable — `builtin_any_char` has no grammar def + no generator special-case; the engine capability ticketed in STIMULI-SIGNOFF): regex `UNKNOWN 3→2`, spf=0, seeds 0/7/42, oracle PASS, NO bump.** NEXT = the regex store-gated pair `numeric_backreference`/`backreference_digits` (B2/C1/C2 fact-emitting-prelude lane) + the vhdl (30) / rtl_frontend (73) / SV (647) drives. |
| — | `GRAMMAR-WELLFORMED.C2` | `done` (`C2.1` design `-0063` + `C2.2` implement `-0064`) | **Semantic-prelude reach (count-gated MVP) LANDED — regex is the 4th `fully_certified` grammar (`UNKNOWN 2→0`, witness 198, spf=0, seeds 0/7/42 × counts 1/40/200, deterministic).** Cross-grammar byte-identical; oracle + cross-family + mdbook gates PASS; no regen/bump. |
| 1 | `GRAMMAR-WELLFORMED.H.11` (the vhdl `UNKNOWN`→0 drive) | `in-progress` (`H.11.1` ✅ `-0065`; `H.11.3` ✅ done `-0067` — ENGINE FIX, vhdl release `1.0.4`, ledger `VHDL-0002`; next = `H.11.4` white_space (the LAST vhdl UNKNOWN) + `H.11.2` seed-7 spf (conjecture REFUTED — see row) + NEW `H.11.5` SV `/`-fusion over-generation) | **vhdl-30 ROOT-CAUSED (tools-first, 2026-06-10):** probe capture (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, count 40 seed 0) showed the dominant family is a LEXICAL-FAITHFULNESS engine defect — reach-pass probes fuse a multi-token regex-terminal render's tail with the following keyword (`8 mintO`, `msThEn`, `nsaNd`), so the probes never re-parse. WHY+WHERE pinned: `physical_literal := trivia /[0-9]…[ \t\r\n]+(?:fs|ps|…)\b/` (`grammars/vhdl.ebnf:422`) renders MULTI-TOKEN text (`8 min`); `apply_word_boundary_spacing` correctly defers its trailing-`\b` separator to the join rule but recorded the tail state with the WHOLE-STRING `is_word_shaped_literal` (`"8 min"` has a space ⇒ `false`) ⇒ `append_generated_segment` never separated ⇒ fusion. The minimal `trivia := (…)*` ⇒ empty in construct mode explains why ONLY reach probes hit it (diverse spf=0 at seed 0). THIRD instance of the `H.8` Defect-A class — hints were fixed with the tail-aware helper; regex-terminal renders were not. **`H.11.1` — `done` (`PGEN-GRAMMAR-WELLFORMED-0065`, ENGINE FIX): the 3 `apply_word_boundary_spacing` recording sites switched to the tail-aware helper (renamed `literal_hint_tail_word_shaped` → `tail_word_shaped` — it now serves hints AND multi-token terminal renders); token-literal site untouched (no failing evidence — targeted fix). VERIFIED: vhdl cert-coverage `UNKNOWN 30→4` (witness 187→213), the ENTIRE fusion family closed, residual EXACTLY the predicted [based_literal, based_value, hash, white_space] — IDENTICAL at seeds 0/7/42 (pre-fix the residual seed-wobbled 30/31); regex STAYS `fully_certified` (198/198, spf=0, seeds 0/7/42); json/rtl_const_expr/svpp stay `fully_certified`; rtl_frontend 73 + SV 647+68-no-path byte-identical, spf=0; new lock `multi_token_regex_terminal_tail_separates_from_following_keyword`; default lib 639/0, dual-feature 732/0; oracle + cross-family gates PASS; mdbook gate PASS (lexical-annotations chapter: new "Multi-token regex terminals participate the same way" section); clippy strict-source clean; NO regen/bump (generator-only).** Ticketed sub-leaves: `H.11.2` = the vhdl seed-7 diverse-pass `spf=2` — **PROVEN PRE-EXISTING via decisive git-stash baseline (pre-fix binary: seed-7 spf=2 IDENTICAL)**, an over-generation residual to root-cause (the H.5.5 pattern; plausibly the same `H.11.3` parser bug seen from the diverse side — check the failing samples for based literals first); `H.11.3` = **RE-ADJUDICATED 2026-06-10 (`PGEN-GRAMMAR-WELLFORMED-0066` investigation, tools-first): NOT generator over-generation — a vhdl GENERATED-PARSER BUG (🚨 released-family, highest priority per the director's fix-parser-bugs-ASAP principle). The parser REJECTS VALID VHDL based literals.** Decisive bisect on the exact probe shell (`…SignAL I:y:=<X>;…`): `X=3374` PASSES, `X=2#1010#` REJECTS (valid VHDL!), `X=3374#BFCd#` REJECTS, while `X=16x"AB"` PASSES (so the literal tournament is NOT naive first-match shadowing — bit_string wins over the decimal prefix, refuting the initial lost-ordering hypothesis too). WHERE (rule-scoped trace `--trace-rules based_literal,hash,based_value`): inside `based_literal@40`, `unsigned_number` exits `40→41` ✓, then `hash`'s `trivia` ENTERS branch 1/2 "at position 41" but the `white_space` regex inside it EXECUTES at position **58 = EOF** ("no match … next: '<EOF>'") — the parser position JUMPS `41→58` between the branch entry and the rule body with no trace in between, so the open `#` is sought at EOF and `based_literal` fails on every input. Suspect classes (NOT yet pinned — next step is a deeper diagnostic per [[feedback_why_and_where_before_solution]], NOT a fix): (a) a multi-branch tournament arm evaluating with a stale/unrestored `parser.position` from a previous candidate (the branch-entry message prints the saved start while the live position differs — instrument or read the emitted tournament arm prologue); (b) a memoized_call interplay restoring a wrong cached end. Next steps: minimal repro (`/tmp/vhdl_based_probe2.vhd` retained: lowercase variant + parenthesized variant), pin the emitted code path (read the generated `parse_trivia`/`parse_hash`/tournament arm in `generated/vhdl_parser.rs`), and an old-binary A/B (regenerate the vhdl parser from a pre-`BRANCH-BROADCAST-FIX-0003` checkout in a worktree) to bound the regression window — based_literal was already UNKNOWN at `H.2` wiring, but the PARSE REJECTION itself has not been historically bounded. Consumer-visible fix will need: vhdl release bump + ledger row + contract/book lockstep (accept-set widening).** **`H.11.3` — `done` (`PGEN-GRAMMAR-WELLFORMED-0067`, ENGINE FIX, consumer-visible — vhdl release `1.0.3`→`1.0.4`, schema STAYS 3, ledger row `VHDL-0002`, contract+both-books lockstep): WHY pinned by reading the emitted code — the generated layout skipper `consume_layout_for_regex` HARD-CODES `#`-to-EOL comment skipping (plus `//`, `/* */`) into every non-blocklisted generated parser (an EBNF meta-grammar convention; in VHDL `#` is the based-literal delimiter), and unlike its string-terminal sibling `consume_layout_for_terminal` (which has guarded `expected != "#"`… since inception) the regex side had NO introducer guard — so `hash := trivia /#/` had its own `#` eaten as a "comment" (decisive trace: the `/#/` regex executed at position 58=EOF after the skipper consumed `#1010#;…` to end-of-line; the -0066 41→58 'jump' was the skipper inside `parse_white_space`/`parse_hash`, NOT a tournament/memo defect — both -0066 suspects refuted). Regression window bounded via git -S: the `#` arm dates to `2b953f3b` 2026-02-20 — present in EVERY shipped vhdl release. FIX (parser-agnostic, `ast_based_generator.rs`): the three comment arms now stand down when the active token's own anchored pattern matches at the introducer (`regex_token_matches_at_cursor`, cold-path sibling cache); `consume_layout_for_regex` also moved INTO the `uses_match_regex` conditional (fully-literal parsers — regex — no longer carry it as dead code); all parsers regenerated. VERIFIED: probes `2#1010#`/`3374#BFCd#` REJECT→PASS, decimal control unchanged; vhdl cert-coverage `UNKNOWN 4→1` (witness 213→216; residual EXACTLY [white_space]=`H.11.4`), IDENTICAL seeds 0/7/42, seed-0/42 spf=0; cross-family: json 9/9, rtl_const_expr 48/48, svpp 74/74, regex 198/198 all stay fully_certified; rtl_frontend `73→71`, SV (sv_2017 canonical) `647→645` + same 68 no-path; oracle gate PASS; cross-family platform gate PASS; default lib 639/0, dual-feature 733/0 (+1 lock `vhdl_based_literal_hash_token_is_not_swallowed_as_comment`); clippy strict-source clean (generated stage = the pre-existing tolerated 191-site debt, unchanged).** ADJUDICATIONS + NEW TICKETS from this slice's sweep: (1) **`H.11.2` same-bug conjecture REFUTED** — the vhdl seed-7 diverse spf=2 persists post-fix and the two failing samples (822/867 bytes, captured) contain NO based literals (view/context/configuration-heavy soup) — it is a genuine separate over-generation residual (H.5.5 pattern), still open; (2) **NEW `H.11.5`**: SV canonical-seed-0 cert-coverage spf `0→1` — the failing sample (timeunits comment-soup, captured) previously re-parsed only because the old skipper blinded SV's explicit `trivia := (ws|line_comment|block_comment)*` (comments were eaten inside following tokens' implicit skips, letting `/`-tokens steal comment-opening slashes); with grammar-faithful comment ownership the parser CORRECTLY rejects the bytes (PEG-greedy trivia is the spec) ⇒ a pre-existing GENERATOR `/`↔comment-fusion lexical-faithfulness gap EXPOSED, not a parser bug — fix generator-side (the `[>!`-style follow-restriction / fusion-guard direction); (3) **PRE-EXISTING red gate (decisive stash baseline)**: `ebnf_frontend_dual_run_gate` fails on the regex flow because the GENERATED ebnf parser cannot parse the `**` flatten-spread (`grammars/ebnf.ebnf` `quantified_marker := ("*"|"+"|"?")` models single markers; `regex.ebnf` gained `[$1**]` with RGX-0074) — comment-free minimal repro fails IDENTICALLY on the pre-fix binary; meta-grammar lags the annotation language — needs its own leaf (ebnf.ebnf grammar fix); (4) **PRE-EXISTING red gate (decisive stash baseline)**: `rtl_frontend_generated_contract_gate` fails (`always_ff_well_formed` missing required rule `module_declaration`) IDENTICALLY on the pre-fix binary with a comment-free sample — broken by some earlier engine wave, needs its own leaf (RTL-FE lane). `H.11.4` = `white_space` parses-but-routes-elsewhere (probe ` ;` — likely consumed via the trivia regex without entering the rule; adjudicate reachable-vs-dead). Then rtl_frontend (71) / SV (645 + 68 no-path, A2.1). |
| 2 | `GRAMMAR-WELLFORMED.B2/C1` | `pending` | The remaining CONSTRUCTIVE-side lanes (stimuli generator): bounded-ordered backtracking, defeat-earlier-branch crafting. Riskier (touch generator runtime; measure the global metric). Feeds G.3 (the witness producer). |

## Decisions
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
- `2026-06-05`: Created from the director brainstorm. The frame UNIFIES the linter (static proof) +
  the stimuli generator (constructive proof) of reachability. Cross-refs: `PARSE-SOTA` (existing
  lint checks A1/.9), `SV-EXH-PROOF.7` (the generator/literal-0 consumer), `PARSE-TERMINATION`
  (`non_terminating`). The de-dup of 25 dead branches (`SV-EXH-PROOF.7.4.6.7`) was the first
  embodiment; this tree generalizes it into a proof.
