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
- `H.2` — **BLOCKED (2026-06-07, attempt under `PGEN-GRAMMAR-WELLFORMED-0036`): Phase H for non-focus
  grammars (vhdl/svpp/rtl_*/json) needs a canonical parser-regen recipe FIRST.** Wiring `parse_and_cover`
  for vhdl (mirror the regex H.1 pattern: `parse_and_cover_vhdl` is trivial — no profile/stdlib/dedicated
  stack) requires the vhdl parser to carry the unconditional G.4.6 coverage methods, i.e. a regen. But
  unlike regex/SV there is **no `focus_vhdl` make target**, and `generated/vhdl.json` is **STALE**
  (`vhdl.ebnf` mtime > `vhdl.json` mtime), and the committed `generated/vhdl_parser.rs` was produced from
  a fresher source than the on-disk `vhdl.json` (parser mtime > json mtime) — so there is no clean,
  canonical "regenerate vhdl" path to reproduce the current parser + add coverage. A correct regen needs
  the full chain (frontend `vhdl.ebnf → fresh vhdl.json` via the `ebnf_dual_run` binary, then generator
  `vhdl.json → vhdl_parser.rs` with the standard `--generate-parser --debug --trace
  --eliminate-left-recursion` flags), then the HEAVY vhdl conformance re-verify (vhdl corpus triage /
  stimuli gate) to confirm no regression. UNBLOCK: add a `focus_vhdl` (and `focus_<grammar>`) make target
  that runs that chain deterministically — a small infra slice — THEN the per-grammar `parse_and_cover`
  wiring (H.2..) is mechanical. NO tracked code change was made (the local `generated/vhdl_parser.rs` was
  regenerated from the stale `vhdl.json` during investigation — harmless: untracked + regenerated
  downstream by any vhdl gate/build; a correct regen needs the fresh-json chain above).
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
| 1 | `GRAMMAR-WELLFORMED.H` (Phase H per-grammar cert-coverage) | `in-progress` | Wire `parse_and_cover` for every grammar so `--report-certificate-coverage` runs per-grammar. ✓ H.1 regex (`-0034`, UNKNOWN residuals + 6→3 witness-parseability), ✗ H.2 vhdl (`-0036`, BLOCKED on the fresh-json regen chain / no `focus_vhdl`), ✓ **H.3 json (`-0037`, `fully_certified=true` — the FIRST grammar fully certified via Phase H)**, ✓ **H.4 rtl_const_expr (`-0038`, cert-coverage runs; zero-drift regen proof retires the H.2 mtime-staleness fear)**, ✓ **H.5 svpp (`-0039`, cert-coverage runs at default depth)**. NEXT: rtl_frontend (a `focus_<grammar>` target + regen + a `parse_and_cover_<grammar>` registry fn — now mechanical); then vhdl, now de-risked by H.4's checkout-illusion proof. |
| 2 | `GRAMMAR-WELLFORMED.B2/C1/C2` | `pending` | The CONSTRUCTIVE side (stimuli generator): bounded-ordered backtracking, defeat-earlier-branch crafting, semantic-prelude reach. Riskier (touch generator runtime; measure the global metric). Feeds G.3 (the witness producer). |

## Decisions
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
