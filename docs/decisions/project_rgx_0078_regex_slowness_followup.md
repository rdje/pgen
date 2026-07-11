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

**🎯 GOAL FRAMING (director, 2026-07-11 — CORRECTED thesis: the gap is generator MATURITY, not
"generated vs hand-tuned").** The director's decisive point: hand-tuned code is *just code written with
certain techniques*; a code generator can emit those SAME techniques — and more (per-rule specialization no
human would maintain) — so generated code CAN match hand-tuned code. There is no expressiveness barrier;
codegen is a SUPERSET of hand-writing. Empirically true (Ragel/re2c/flex/protobuf codegen match-or-beat
hand-written). So the gap is **"the generator currently emits a NAIVE pattern (per-backtrack alloc,
try-every-branch tournament) vs the FAST pattern it could emit (arena, first-set dispatch, specialized
per-rule code)"** — a maturity gap that CLOSES, not an intrinsic one. **Correction to the earlier caution:
we are NOT racing a JIT.** PCRE2's JIT is its *matching* engine; RGX-0078 races PCRE2's *compile* step (=
our parse time), and PCRE2's compiler is **just hand-written C** — no JIT on the parse-time side. The only
thing reserved for hand-code+JIT is runtime data-dependent specialization to a specific pattern (a
*matching* concern, irrelevant to a one-shot parse). **⇒ For the parse-time goal there is NO intrinsic
barrier; the ceiling is hand-tuned-C parse speed, not "5× is the best a generator can do."** STRATEGY
CONSEQUENCE (aligns with PGEN's founding doctrine): **every speed lever lands as a codegen / shared-engine
primitive** (teach the generator to emit the fast pattern) ⇒ every win is parser-AGNOSTIC (SV/VHDL/JSON/RTL
all inherit it) — strictly better than hand-tuning one parser. Honest about EFFORT (arena/dispatch/
specialization are real codegen work, one measured step at a time) but not about CEILINGS. Captured in the
book chapter *The gap is generator maturity, not "generated vs hand-tuned"*
(`docs/book/src/inside-parser-performance.md`).

---

**✅ FIRST LEVER LANDED (RGX-0078·1, session #90/#91, 2026-07-11).** The rollback scope-restoration
guard is committed. Root cause (profiled, `.2`): `SemanticRuntimeState::rollback_to_named` was ~24% of
regex parse self-time — an UNCONDITIONAL `active_chain.clone()` + `scopes = …collect()` rebuild on every
failed speculation, even when the speculation changed no scope state (the common case under the
`longest_match` atom-tournament). Fix (`semantic_runtime.rs:2800`, shared engine, parser-agnostic, no
codegen/regen): gate the restoration on `scope_arena.len() > scope_arena_len || active_chain !=
snapshot` — sound because `scopes` is kept lockstep with `active_chain` by `open_scope`/`close_scope`,
so a no-change backtrack leaves `scopes` already correct and the skip is a no-op.

- **Speed:** decisive stash baseline (stash ONLY the code file, rebuild, re-measure) → **422µs → 344µs
  = −18.6%** geomean of noise-floor mins on the 8-pattern bench.
- **Correctness floor DECISIVELY intact** (the ⛔ HARD constraint, every oracle green at its
  pre-optimization value):
  - PCRE2 compile-oracle **BYTE-IDENTICAL** — fix-vs-no-fix debug-probe diff on the normalized corpus
    (the tracked 80-deep-paren catastrophic cell, line 1340, excluded up front; it has no named
    scopes/lookaround so the change is a provable no-op on it) returns `1878/310/262/48` either way,
    canonical-JSON diff EMPTY → not one new false-accept/false-reject.
  - cert `fully_certified`; spf spf-NEUTRAL (with-fix == no-fix `0/1/1` at seeds 0/7/42, `--count 40`);
    equivalence byte-identical; semantic 36/36; duality-hunt no new signature; ast-shape contract 1 passed;
    clippy exit 0.
- **Method note (kept for the next lever):** the full oracle gate uses a DEBUG probe over the FULL corpus
  and grinds for hours on line 1340 — do NOT run it whole. Exclude the tracked cell up front, run the fast
  remainder (release or debug), and diff fix-vs-no-fix. See [[feedback_dont_run_jobs_that_hit_known_pathological_inputs]].

**Next levers (this decision's queue, each profile-driven + re-run the full gate battery before landing):**
free AOT build flags first (LTO / `codegen-units=1` / `target-cpu=native` / PGO — the release build currently
uses cargo defaults, so these are unclaimed wins), then FxHash annotation-table lookups, first-set predictive
dispatch, and a parse cache. Every lever lands as a shared-engine / codegen primitive so all parsers inherit it.

---

**✅ SECOND LEVER LANDED (RGX-0078·4.a, session #92, 2026-07-11).** The first free AOT build-flag win is
committed: `rust/Cargo.toml` gained a `[profile.release]` section with `lto = "fat"` + `codegen-units = 1`
(the release build previously used cargo defaults — `lto=false`, `codegen-units=16`, measured: no
`[profile.release]`, no `.cargo/config`). Build-configuration change; parser-AGNOSTIC (every release binary/
grammar inherits it); NO source / grammar / codegen / regeneration. Machine-INDEPENDENT and reproducible
(unlike the `.4.b` `target-cpu` / `.4.c` PGO levers still queued).

- **Speed:** DECISIVE drift-controlled back-to-back — saved the fat-LTO binaries, stashed ONLY `Cargo.toml`,
  rebuilt the default-profile binary (distinct sha256), measured BOTH ALTERNATELY (so background CPU load
  cancels): DEFAULT `331,856 / 332,946 ns` vs FAT-LTO+cu=1 `310,919 / 309,290 ns` = **−6.7%** (≈332→310µs),
  all 8 patterns improved monotonically, zero overlap between the two profiles' floors.
- **Correctness floor byte-identical** (the ⛔ HARD constraint): a `[profile.release]`-only change touches
  ONLY release binaries — the debug profile is untouched, so the debug-built oracles (cert, equivalence,
  semantic, duality-hunt, ast-shape) are byte-identical BY CONSTRUCTION. Directly re-verified the two that
  matter: the fat-LTO RELEASE `regex_corpus_probe` observations are `diff`-EMPTY vs the UNCHANGED debug
  reference across 2188 corpus cells (`1878/310/262/48`, not one new false-accept/false-reject; 80-deep-paren
  cell `line_4674` excluded up front), and regex cert stays `fully_certified` UNKNOWN=0 / spf `0/1/1` at
  seeds 0/7/42.
- **Measurement-integrity lesson (kept):** for a modest (~6%) build-flag delta the noise-floor min alone is
  insufficient — a before and an after measured minutes apart can drift with background load. Build BOTH
  binaries to disk, then measure ALTERNATELY in one tight loop; the clean zero-overlap separation is what
  makes the delta attributable. The [[feedback_prove_independence_with_decisive_baseline]] discipline for a
  build-config change = stash only the one file, rebuild, re-measure back-to-back.

**Still queued:** `.4.c` PGO (the AOT analog of PCRE2's JIT benefit — needs a repeatable training run over
the bench corpus), then the source levers (`.5`: FxHash annotation-table lookups, per-speculation arena,
first-set predictive dispatch, parse cache).

---

**✗ LEVER REJECTED (RGX-0078·4.b, session #92, 2026-07-11) — `target-cpu=native` is a MEASURED REGRESSION.**
The commonly-assumed "free" win was TESTED, not assumed (the profile>inference discipline). Built
`regex_perf_probe` with `RUSTFLAGS="-C target-cpu=native"` on top of the `.4.a` fat-LTO profile (distinct
sha256), measured ALTERNATELY back-to-back against the saved fat-LTO-only binary: fat-LTO-only `313µs` vs
+native `324–327µs` = **+3.5…+4.6% SLOWER**, uniformly across 7 of 8 patterns (only `literal_simple` flat),
three alternated readings with zero overlap. Cause: PGEN's parser is branchy, control-flow-bound, small-input
(the `.2` profile: 59% malloc + per-char `longest_match` tournament — nothing data-parallel), so
native-codegen autovectorization/apple-m4 instruction selection bloats the code with SIMD setup that never
pays off and hurts I-cache locality. **DECISION (autonomous — two independent reasons):** do NOT bake it —
(1) it is a measured regression here; (2) it is machine-specific (non-portable), so it could not ship in a
tracked config anyway. `RUSTFLAGS` was never written to a tracked file, so the committed config is unchanged
(fat-LTO only). NO code change; NO bake-vs-consumer decision to surface (nothing worth shipping).
**Consumer-guidance takeaway:** downstream (RGX) should NOT assume `target-cpu=<host>` helps — measure the
target workload first. **General lesson (durable):** a build flag's reputation is no substitute for a
measurement — `target-cpu=native` is NOT a universal win and regresses branch-heavy, small-input parsers.

---

**⏸ STRATEGIC STATE / RECOMMENDATION at the build-flag phase boundary (session #92, 2026-07-11) — surfaced for the director.**
The cheap, safe, correctness-neutral build-flag phase of `.4` is essentially explored: **`.4.a` LTO+cu=1
LANDED −6.7%** (a real parser-agnostic win, shipped), **`.4.b` target-cpu=native REJECTED** (measured +4%
regression). One build-flag lever remains — **`.4.c` PGO** — and it is a DIFFERENT character worth an explicit
call:
- **PGO is a build-PROCESS change, not a drop-in flag.** It needs instrument-build → train-on-a-corpus →
  `llvm-profdata` merge → optimized rebuild (feasible: llvm-tools present). A *win* would raise a genuine
  SHIPPING/REPRODUCIBILITY question — how does RGX reproduce the PGO build, and train on what corpus? — which
  is a real-world side effect (director-owned per [[feedback_user_is_director_not_engineer]]).
- **Its value here is uncertain and likely modest.** The `.2` profile pins the real cost at **59% malloc +
  the per-char `longest_match` backtracking tournament**. PGO improves branch layout / inlining from a real
  profile, but does NOT reduce the malloc volume — so its ceiling on this workload is bounded (control-flow
  overhead only, not the dominant allocation). And an honest measurement must train on a DIVERSE corpus and
  measure on a held-out set (training+measuring on the 8-pattern bench overfits).
- **The profile-indicated REAL remaining win is the `.5` SOURCE/ENGINE levers**, which attack the measured
  bottleneck directly: per-speculation arena/reuse (the 59% malloc), FxHash/codegen-elided per-rule
  annotation-table lookups (~6% SipHash), first-set predictive dispatch (the backtracking multiplier). These
  are parser-agnostic and higher-EV — but they are **correctness-RISKY engine changes** (each must pass the
  full `.3`/`.4.a` oracle battery under the ⛔ HARD constraint) and deserve careful, dedicated slices.

**RECOMMENDATION:** the safe build-flag wins are banked; the next real speed requires either the PGO
build-process investment (modest, uncertain, with a shipping decision) or the correctness-risky `.5` engine
work (higher-EV, attacks the profiled malloc). Both are substantial. My lean: do the PGO measurement next
(it completes `.4` and is low-correctness-risk since a build can't change semantics), deferring the PGO
*shipping* decision until the number is in hand; then take the `.5` engine levers as fresh, careful slices.
