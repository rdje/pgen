---
name: project-rgx-0078-regex-slowness-followup
description: QUEUED (director 2026-06-03) — right AFTER the SV main parser convincingly goes Mostly-Done -> Done, attempt another round at RGX issue PGEN-RGX-0078 (pgen regex parser slowness). Closure = geomean PGEN-parse / PCRE2-compile < 5x. Try NEW speed techniques; use the PCRE2 test corpus + the vendorable pgen_iteration_flow harness.
metadata:
  node_type: memory
  type: project
---

**Director directive (2026-06-03):** *"Right after the SV main parser convincingly goes from
Almost Done to Done, please attempt another round at
`/Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-0078.yaml` from RGX. It is about
the slowness of the PGEN regex parser. You will need to try other techniques to speed up its
runtime. For that you can use the PCRE2 test corpus."*

**PRECONDITION (sequencing):** SV main parser reaches **Done** first. SV is currently
`Mostly Done`; the one short criterion is `focused_replay_target_debt_zero` (literal-0) — the
273-residual closed-loop work (`SV-EXH-PROOF.7.4.6` derivation-directed construction). RGX-0078
is queued AFTER that. Do NOT start RGX-0078 before SV is convincingly Done.

**The issue (PGEN-RGX-0078, status open):** pgen's `regex` parser parse-time is **~360x slower
than PCRE2's full no-JIT compile, ~85x slower than PCRE2 JIT** (geomean over the 8-pattern
bench corpus from RGX-0073). The PRIMARY absolute <50µs target was met (release 1.1.30); the
INTEGRATION closure criterion is the **PCRE2-RELATIVE ratio: geomean PGEN-parse / PCRE2-compile
< 5x**. Bug class: pathological performance. A prior round (RGX-0073) closed the absolute
target; 0078 is the relative-ratio follow-up — still open.

**Resources (provided in the issue):**
- A vendorable `pgen_iteration_flow/` directory (in the RGX issue area): standalone C PCRE2
  baselines, a self-contained Rust microbench, Cargo.toml/Makefile integration snippets, a
  one-shot driver — so pgen runs the same PCRE2-relative measurement without RGX. See
  `pgen_iteration_flow/README.md`.
- The **PCRE2 test corpus** (the director's named tool — for both correctness regression and a
  broader perf corpus than the 8-pattern bench).

**Methodology / approach (grounded by this session's TERMINATION.3 work):**
- **Profile first (WHY+WHERE before fix):** macOS `sample` on a hot regex parse (the technique
  that pinned SV's O(N²) this session) to find regex's actual bottleneck — then act on the
  profiled fact, not a guess. ([[feedback_tools_first_no_guessing]],
  [[feedback_research_grounded_sota_no_trial_and_revert]].)
- **NOTE — regex's cause is DIFFERENT from SV's:** the SV slowness was the per-rule
  `SemanticRuntimeState` clone in `with_semantic_runtime_rule_transaction`
  ([[stateful-packrat-not-linear]]); regex has **no semantic predicates**, so it takes that
  function's FAST-PATH (skips the clone). So `PARSE-TERMINATION.3.1` will NOT help regex —
  regex's bottleneck is elsewhere (likely the general parse machinery: memo allocation per
  rule, ParseNode/clone churn, the FxHashMap memo, per-char overhead vs PCRE2's hyper-tuned
  compile). Profile to pin it.
- **Try other techniques:** candidates to evaluate AFTER profiling — memo/allocation tuning,
  arena/zero-copy for ParseNode, reducing per-rule overhead, the cut operator / bounded memo
  (Mizushima 2010, PARSE-SOTA B2), avoiding redundant work. Pick based on the profile.
- **Measure the GLOBAL metric** (geomean PGEN/PCRE2 ratio) before/after each change; keep iff
  it improves; confirm no RGX conformance regression (the PCRE2 corpus is the oracle).

**Cross-repo:** the issue lives in the `rgx` repo (`rgx/pgen-issues/`); the WORK is on pgen's
regex parser (`grammars/regex.ebnf` + the engine/codegen) — both in this repo. Coordinate the
closure report back to the RGX issue. Likely owned by a new `RGX-0078` task tree when activated.

Composes with [[feedback_correctness_before_speed]] (this is the speed phase — regex is already
correct/conformant), [[feedback_no_codebase_change_without_tool_backed_facts]].

---

**⚡ REFRAMING (director 2026-07-11) — SPEED is the PUBLICATION BLOCKER + the slowness is
HYPOTHESIZED SYSTEMIC (all PGEN parsers).** Director: *"what prevents me from publishing RGX (hence
the PGEN regex parser) is the current speed — it is way too slow. And I think the slowness is
somehow INHERENT, i.e. it affects ALL PGEN-generated parsers. PGEN is really good at accuracy now;
it needs to be respected on speed too."* This ELEVATES RGX-0078 from a regex-hotspot hunt to a
**PGEN-WIDE engine-performance investigation** and makes it release-critical (RGX cannot ship until
closed). Chosen as the active frontier under the [[project_horizon_universal_parser]] north star (a
universal parser platform must be usably fast across ALL languages).

**Discipline — VERIFY the systemic claim, don't assume it** ([[feedback_no_codebase_change_without_tool_backed_facts]],
[[feedback_prove_independence_with_decisive_baseline]]): profile FIRST, and TEST "systemic" by
measuring across grammars (regex + json + vhdl + a small SV parse) and locating the cost in the
SHARED engine/codegen, not a regex-specific path. If systemic, the fix is a **parser-agnostic engine
primitive** benefiting ALL parsers ([[feedback_features_parser_agnostic_enable_all_parsers]],
[[feedback_ast_pipeline_parser_agnostic]]), measured GLOBALLY, correctness floor NEVER traded
([[feedback_correctness_before_speed]] ⛔).

**Leading hypothesis-set to profile against (ranked; my architectural read, to CONFIRM/REFUTE with
`sample`/`perf` + `MEMO_STATS`, not act on blind):**
1. **`longest_match` tournament tries EVERY branch** (not PEG first-match-commit): the default `|`
   evaluates all alternatives to compare lengths ⇒ O(branches) work per position — worst for
   wide-alternation grammars (regex `atom`), and systemic (every grammar's `|`). Candidate fix:
   FIRST-SET predictive branch dispatch (skip non-viable branches) + commit-to-first where
   `ordered`/unambiguous. Likely the single biggest systemic lever.
2. **Per-speculation state snapshot/restore** even for predicate-free grammars: every `try_parse`
   snapshots+restores position AND `semantic_runtime_state`; a semantic-INERT fast-path (static "no
   effects below" analysis → snapshot position only) would cut it for regex/json.
3. **Packrat memo hit-rate/overhead** — verify `MEMO_STATS` for regex (is it memoizing effectively,
   or is the FxHashMap overhead > benefit on small inputs?); selective/bounded memo (cut operator,
   Mizushima 2010).
4. **ParseNode allocation churn** per attempt (arena/zero-copy; [[project_uvm_memory_not_the_memo]]:
   profile allocs, don't infer).
5. **Per-char terminal matching** vs PCRE2's hyper-tuned compile.

**FIRST STEP for the fresh session:** open the `RGX-0078` task-tree leaf; build `--release`; profile
a hot regex parse (`sample`) + capture `MEMO_STATS`; profile json/vhdl too to confirm/refute
"systemic"; report WHERE the time goes BEFORE any change. EXCLUDE the pathological corpus cells
(catastrophic `\(…\)` line 878/881 + 80-deep-paren line 1340) up front
([[feedback_dont_run_jobs_that_hit_known_pathological_inputs]]).

---

**✅ PROFILE LANDED (session #90, 2026-07-11) — WHERE + WHY, tool-backed + systemic.** `sample` (release,
1 ms) on regex (`regex_perf_probe` hot loop) + json (740 KB `--parse json`) + vhdl (146 KB `--parse vhdl`).

- **Baseline:** regex 8-pattern-bench geomean ≈ **496µs/parse** (Optim #1–#16 already put it ~2–3× under
  the RGX-0073 reference). `pgen_iteration_flow` NOT vendored ⇒ PCRE2-relative closure number deferred to `.4`.
- **WHERE (regex self-time, 9630 leaf samples):** **59.4% `libsystem_malloc`** (symptom) · 6.9%
  semantic-runtime · 6.0% SipHash/hashmap · 4.4% string-formatting · 1.5% clone/drop.
- **Attribution (call graph):** **`SemanticRuntimeState::rollback_to_named` ≈ 24% of total** — the single
  biggest allocation source, on the backtrack path of every failed speculation. Its allocating children
  are the UNCONDITIONAL `self.scopes = active_chain.iter().map(..).collect()` (`semantic_runtime.rs:2804`)
  + `active_chain_snapshot.clone()` (`:2793`).
- **SYSTEMIC — CONFIRMED:** json (`create_contextual_error` 174, `RecursionGuard::check_cycle` 130) + vhdl
  (same top malloc/SipHash/clone frames) burn in the SAME shared engine code — the slowness is inherent to
  the shared parse machinery, not a regex path. Confirms the director's "systemic" hypothesis.
- **REFUTED (profile > inference — the [[project_uvm_memory_not_the_memo]] lesson, again):** ranked
  hypothesis #4-ish "per-parser construction rebuilds the annotation table ⇒ ~160µs floor" — `new()` is
  ~0.7%, `parse_full` is the 9584 samples. The parse (per-char `atom`-tournament under `longest_match`,
  constant backtracking) dominates, confirming ranked hypothesis **#1 as the DRIVER** and **#2
  (per-speculation overhead) as the COST**.
- **CORRECTED prior:** the 2026-06-03 "regex has no semantic predicates → takes the clone-skipping
  fast-path" note is too strong — regex emits `regex_capture_group` facts + opens/closes lookaround
  scopes, so `SemanticRuntimeState` IS on its hot path (that is exactly why `rollback_to_named` is 24%).
- **Ranked levers → `.3`+ (tree `docs/tasks/RGX-0078.md`):** (1) gate the rollback active-chain/scopes
  restore on scope-state-changed — no regen, correctness-neutral, sound (scopes lockstep w/ active_chain);
  (2) FxHash / codegen-elide the per-rule annotation-table SipHash lookups for annotation-free rules;
  (3) reduce per-speculation ParseContent/Vec churn; (4) first-set predictive dispatch (deeper, later);
  (5) defer the generated `try_parse` Err-arm trace strings (codegen/regen). One change at a time, measure
  the GLOBAL geomean before→after, correctness floor NEVER traded ([[feedback_correctness_before_speed]] ⛔).
