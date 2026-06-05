# PARSE-TERMINATION — the parser never hangs; bounded, near-linear time (parser-agnostic)

> Task tree. **Metadata** — Status: `active` (`.1` measurement DONE 2026-06-03 — CC 2020
> risk CONFIRMED empirically); Created: 2026-06-03; Roadmap lane: parser sign-off pillar
> **B** (of 4). Owns failure mode
> **(1.b) hang** — non-termination or super-linear blow-up (catastrophic backtracking) on
> valid input.
>
> Director-commissioned 2026-06-03 (one tree per failure mode; research-first). Parser-
> AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]). Disciplines:
> [[feedback_research_grounded_sota_no_trial_and_revert]], [[feedback_tools_first_no_guessing]],
> [[feedback_prefer_grammar_leave_engine_alone]] (engine changes are parser-agnostic
> last-resort), [[feedback_severity_never_gated_by_verbosity]] (a watchdog error is a
> severity, always emitted).

## The principle (binding)
A PGEN parser shall **terminate in bounded (near-linear) time on every input**. A hang is a
correctness defect, not "slow." "Solve for good" = make non-termination **impossible by
construction** (well-formedness + a sound complexity guarantee) **plus** a runtime watchdog
that converts any residual pathological case into a *classified severity error*, never a
silent hang.

## ⚠️ The sweep's key warning (this tree's reason to exist)
**Packrat ≠ linear for PGEN.** Ford's linear-time guarantee (*Packrat Parsing*, ICFP 2002)
**assumes a stateless PEG**. PGEN's **semantic store makes it a *stateful* PEG**, and
**Chida & Kawakoya, *Is Stateful Packrat Parsing Really Linear in Practice?* (CC 2020)**
proves real-world stateful grammars can go **exponential**. So we may **NOT assume** PGEN is
hang-free — this is a live, previously-unexamined risk. See [[stateful-packrat-not-linear]].
**That same paper hands us the fix** (adopt, don't invent): a *PEG with variable bindings* +
**stateful packrat with conditional memoization** (memoize only the state relevant to global
-state use) — **260×/217×** time/space win on pathological inputs.

## Literature grounding (citations + worked mapping)
- ✅ **Ford, PEG well-formedness (POPL 2004 §3.6)** — a well-formed PEG cannot loop forever;
  static detection of non-terminating rules. = PGEN's existing **PARSE-SOTA A1**.
- ✅ **Warth, Douglass, Millstein (PEPM 2008)** — packrat + left recursion. = PGEN's
  `pre_lr_elim` + `mutual_recursion_handler`.
- ⚠️♻️ **Chida & Kawakoya (CC 2020)** — stateful non-linearity + conditional-memoization fix
  (above). Sharpens PGEN's already-backlogged **PARSE-SOTA "memo-soundness audit"** (Tier B).
- ✅ **Mizushima et al. (2010)** — cut operators bound backtracking.
- **Worked mapping:** PGEN already has A1 (static non-termination) + memoization + LR
  handling — two of three legs. The gap is the **complexity *guarantee*** under PGEN's
  statefulness, which CC 2020 says we cannot assume.

## Leaves
### `.1` — MEASURE whether PGEN's stateful packrat is actually linear (tools-first) — DONE (2026-06-03)
Built a complexity-scaling probe: parsed SV inputs of size N ∈ {100..3200} via
`parseability_probe --parse systemverilog`, two families — **store** (N `typedef t; t v;`
pairs, exercising the store-gated type-identifier rule = the stateful path) and **width**
(N `wire w;`, stateless baseline). **RESULT (decisive, CC 2020 risk CONFIRMED):**
- **store (stateful):** 100→3200 = 32× input, 0.11 s → **34.8 s** = 316× → **~N^1.66**, and
  the per-doubling ratio *grows* monotonically (2.45 → 2.89 → 3.24 → 3.58 → 3.84 →≈4) —
  **trending QUADRATIC**.
- **width (stateless):** 0.04 s → 1.70 s = 42× → **~N^1.08 = linear** (packrat works as
  advertised when there's no state).
**So PGEN's stateful (semantic-store) packrat is NOT linear** — exactly Chida & Kawakoya
CC 2020. Likely explains historical uvm_pkg slowness (thousands of type decls/uses → the
super-linear store path). Not a hang (all completed), but a real super-linear risk at scale.
No code change (measurement only). Inputs/timings under `rust/target/w74/lin/` (gitignored).
KM card [[stateful-packrat-not-linear]] updated with the empirical confirmation. → justifies
`.3` (conditional memoization) as the fix.

### `.2` — static no-hang surface: nullable-repetition detector — DONE (PGEN-PARSE-TERMINATION-0002, 2026-06-03)
Extended `grammar_wellformedness.rs` with `detect_nullable_repetition` — flags an UNBOUNDED
quantifier (`*`/`+`/`{N,}`, i.e. `max == None`) over a NULLABLE body (the "loop without
consuming" hazard, Ford PEG well-formedness POPL 2004 §3.6). Reuses the existing
`compute_nullable`/`node_nullable` fixpoint; deterministic, parser-agnostic. New
`WellformednessIssue::NullableRepetition { rule, node_path }` + wired into `--lint-grammar`
as a WARNING (runtime is zero-length-guarded, so it's not an actual hang — but the grammar is
ill-formed). Unit-tested (positive + no-false-positive on non-nullable/bounded); lib 588/588;
source-strict clippy 0.
**RAN on shipped grammars: regex/ebnf/vhdl/rtl_frontend = 0; SystemVerilog = 7 real findings:**
`bins_or_empty` (root/o0), `bins_or_options` (root/o1/s5 — note: also a deeply-factored
slow-witness rule from `.7.4`), `module_path_concatenation` (root), `rs_code_block` (root),
`rs_production_list_sv_2017`/`_2023` (root/o0/s1), `select_condition` (root/s4/q/s1). These
are genuine ill-formed sites (runtime-guarded, low-urgency) → the 7 grammar FIXES are
follow-up targeted leaves (`.2.1`-`.2.7`, each: analyze intended quantifier/body, fix, regen
+ verify corpus/shape-contracts — NOT done here; the detector is the `.2` deliverable).

### `.3` — root-cause + fix DESIGN (DONE, PGEN-PARSE-TERMINATION-0003); implementation = `.3.1`
Director signed off the engine change. WHY+WHERE-first investigation (tools: code read +
macOS `sample` profile of the store_3200 parse) **PINNED the root cause and CORRECTED the
leaf's premise** — it is **NOT** CC 2020 conditional memoization:
- **ROOT CAUSE (profiled, decisive):** `with_semantic_runtime_rule_transaction` (codegen
  `ast_based_generator.rs:1281-1283`) does `std::mem::take(&mut self.semantic_runtime_state)`
  then `= original.clone()` — a **full clone of the entire `SemanticRuntimeState`** (facts
  BTreeMap/HashMap, scopes, strings, vecs) on EVERY rule transaction, restored by `= original`
  on failure. O(state) per call × O(rule-calls) = **O(N²)**. The profile's hot frames are
  `SemanticRuntimeState::clone` (clone_subtree / String::clone / HashMap::clone / to_vec) +
  `drop_in_place<SemanticRuntimeState>`, all under `with_semantic_runtime_rule_transaction`.
  (NOT the memo — keyed (rule,pos), never cleared; NOT has_fact — indexed `by_kind`, O(1);
  the width baseline is linear because it runs no transactions over a growing store.)
- **FIX (efficient snapshot/restore — Laurent & Mens SLE 2016):** replace the take+clone
  snapshot with `self.semantic_runtime_state.checkpoint()` (O(1)) and the err-restore
  `= original` with `self.semantic_runtime_state.rollback_to_named(cp, Some(rule_name))`
  (O(changes)). `rollback_to_named` is VERIFIED COMPLETE (truncates facts + the by_kind
  fact_index, truncates scope_arena, restores active_chain, **un-closes** scopes, rebuilds
  `scopes`) and is ALREADY used + proven in the try_parse branch path (SV corpus 14/14). Also
  translate the in-IIFE `original_semantic_runtime_state.facts().len()` (`:1385`) →
  `cp.fact_len`; rule_context push/pop is unaffected (checkpoint/rollback don't touch it).
- **EXPECTED IMPACT:** per-transaction O(state) → O(changes) ⇒ store-gated parsing
  N^1.66 → ~N^1.0 (linear, matching the width baseline); fixes the super-linearity + likely
  the historical uvm_pkg slowness.

### `.3.1` — implement the checkpoint/rollback swap — DONE (PGEN-PARSE-TERMINATION-0007, 2026-06-05)
LANDED. Codegen (`ast_based_generator.rs`, the `with_semantic_runtime_rule_transaction` quote!):
the O(N) full-state `take`+`.clone()` is replaced by an O(1) `checkpoint()`; the err-restore
`self.state = original` is replaced by `rollback_to_named(checkpoint, Some(rule_name))`
(O(changes)). The lost-facts subtlety (the mid-fn `mem::take` for the txn borrow-checker) is
handled by an INNER closure returning `ParseResult<()>` so any `?` returns LOCALLY; the taken
state is ALWAYS moved back into `self.semantic_runtime_state` BEFORE propagating, so the outer
`rollback_to_named` operates on a POPULATED state. `node` stays owned by the outer scope (not
moved into the closure) so `semantic_raw_content`'s borrow stays valid. Codegen self-test updated
to assert the checkpoint/rollback shape + the ABSENCE of the per-rule clone.
VERIFIED: regen compiles; lib (no-features) 590/0 + (features) 652/0 (shape contract incl.);
**release uvm parse PASSES** (`parse_full passed` — NO lost facts; the gate's debug failures were
1800 s TIMEOUTS, proven by the exact ~1800 s parse-log mtime gaps, NOT rejections); **memory
26 GB → ~11.6 GB peak, fluctuating DOWN (11.9→6.2 GB = rollback FREEING)**. CORRECTNESS + the
clone-memory cure confirmed.
⚠️ HONEST — `.3.1` is the *clone* cure, NOT the full uvm cure. TWO roots remain (the user's "address
the root"): (1) the remaining ~11.6 GB = the unbounded packrat MEMO (keyed (rule_id,position),
never cleared) → new leaf `.6`; (2) parse TIME: uvm ~181 s release vs >1800 s debug (the gate uses
debug → times out → corpus 10/14). Time sub-causes: the gate's debug build + `match_regex` runs
`re.find("")` EVERY call to recompute the static `can_match_empty` → leaf `.7`. The gate's 14/14 is
NOT restored until the time roots land (the `.4.0` timeout now exposes the debug slowness).

### `.3.1-historical` — original design + subtlety (kept for provenance)
Apply the fix in `with_semantic_runtime_rule_transaction` (codegen `ast_based_generator.rs`).
`generated/` is GITIGNORED (not committed) → the commit is the codegen `.rs` only; regen is
local verification. The 4 sites: snapshot (`:1281-1283`), entry-fact-len read (`:1385`),
failure-restore (`:1552`), and the codegen self-test assertion (`:7622`,
"let original_semantic_runtime_state" → update to assert checkpoint/rollback).
⚠️ **DISCOVERED SUBTLETY (2026-06-03, found while scoping the edit — NOT a 3-line swap):** the
success path does `std::mem::take(&mut self.semantic_runtime_state)` mid-function (`:1389`)
for the commit/delta machinery, then restores it (`:1546`). The original full-clone restore
(`self.state = original`) is robust to a failure BETWEEN the take and the put-back. A naive
`rollback_to(checkpoint)` would run on an EMPTIED `self.state` on that path → cannot restore
pre-checkpoint facts → **lost-facts correctness hole**. So `.3.1` must EITHER restructure the
transaction/commit machinery to operate on `self.state` in place (no mid-`take`) OR
restore-the-taken-state-before-rollback on the failure path. This is a careful refactor of the
transaction machinery, not a swap → a dedicated, fully-tested focused slice. VERIFY: `.1`
linearity probe (store → ~N^1.0), SV external corpus 14/14, lib (no-features +
generated_parsers), shape-contracts, determinism, AND a targeted test for the
failure-between-take-and-putback path. KM [[stateful-packrat-not-linear]].

### `.3.2` — EMPIRICAL real-world instance: uvm_pkg parse → ~26 GB RAM + apparent hang (PGEN-PARSE-TERMINATION-0005, 2026-06-05, OBSERVATION)
**Director surfaced (2026-06-05):** during `sv_external_corpus_triage_gate`, the
`case_uvm_compat_pkg_2017_bootstrap_1` parse — `target/debug/parseability_probe --parse
systemverilog <uvm_pkg.sv, 2.89 MB preprocessed> --profile 2017 --lib-out …` (PID 3519) —
consumed **~26 GB RAM and appeared stuck**. Killed to free the host (the gate is reproducible;
no data lost). This is the FIRST real-world hit of the failure mode this tree exists for.
**Tool evidence (controlled reproduction, debug build, `ulimit -v 12 GB` cap, this session):**
- Isolated `--parse` AND `--parse --lib-out` of uvm_pkg: RSS **60 → 104 MB over 36 s** but did
  NOT finish — so the **early/shallow parse is modest memory but SLOW**; the **26 GB blows up
  DEEPER** in the parse (super-linear, past the first ~100 MB phase), on uvm's deeply-nested
  store-gated constructs. (`--lib-out` did NOT itself balloon early → not the fact-export.)
- macOS `sample` of the early phase: hot self-time is **REGEX MATCHING** — `memchr` /
  `aho-corasick` teddy / `regex_automata` DFA (`next_state`, `ByteClasses::get`,
  `is_aligned_to`) — i.e. terminal/token matching over the 2.89 MB text dominates the bulk phase.
**Interpretation (two cost components):**
1. **MEMORY/super-linear (the 26 GB):** consistent with `.1`'s ~N^1.66 + `.3`'s pinned O(N²)
   `with_semantic_runtime_rule_transaction` **full `SemanticRuntimeState` clone** — on uvm's
   thousands of type decls the cloned-per-rule state grows, and the per-rule clone on deep
   store-gated rules balloons to GB. The 26 GB is the **real-world MEMORY face** of the known
   TIME super-linearity. → `.3.1` (checkpoint/rollback swap) is the fix; **urgency raised.**
2. **TIME/regex (the early bulk phase):** a LIKELY SECOND, possibly-separate cost — the PARSER
   appears to spend heavily in regex terminal matching (cf. the GENERATOR's per-call regex
   recompile fixed in SV-EXH-PROOF.7.4.6.1; check whether the generated parser recompiles
   terminal regexes per call / per position rather than once). Own follow-up probe.
**NOT caused by** `ANNOTATION-COMPOSITION.6.4` (the just-committed `non_zero_decimal_digit`
fix): it is a NO-OP under profile 2017 (the rule was present under 2017 before AND after; it was
only a *sv_2023* orphan). Verified via `git show HEAD~1`.
**ACTIONS:** (a) raises `.3.1` urgency (kills component 1); (b) raises `.4` urgency — a
memory/step **watchdog would have converted the 26 GB into a classified severity error instead
of exhausting the host** (this is the watchdog's reason to exist, now PROVEN by a real event);
(c) INTERIM SAFETY — the SV corpus gate should run uvm cases under `ulimit -v` + a timeout so an
OOM is a *classified failure*, not silent host-RAM exhaustion; (d) a future controlled
long-capped run to pinpoint the exact rule/depth where memory explodes + confirm the
clone-vs-regex split + check parser regex recompilation. See [[stateful-packrat-not-linear]],
[[sv-corpus-gate-uvm-memory]].

### `.4` — watchdog / resource guard (hang|blowup → classified failure, never host exhaustion) — PARTIAL
#### `.4.0` — GATE-LEVEL resource guard (memory cap + timeout) — DONE (PGEN-PARSE-TERMINATION-0006, 2026-06-05)
Immediate safety net for the `.3.2` event (uvm ~26 GB > host 24 GB → swap-thrash). The SV
external-corpus triage gate now runs every PARSE under a subshell `ulimit -v` (auto-sized to
~70% of host RAM; env `PGEN_SV_TRIAGE_MEM_CAP_KB`, 0=off) **+** `timeout` (env
`PGEN_SV_TRIAGE_PARSE_TIMEOUT_S`, default 1800 s), scoped to parses only (NOT the cargo build).
A runaway parse is now a **classified `parse_fail`/timeout**, never silent host exhaustion.
VERIFIED: scr1 (healthy) passes under the 16 GB cap; uvm under a 2 GB cap is killed at the
timeout having NEVER exceeded the cap (host-safe). **⚠️ INTERIM CONSEQUENCE:** on this 24 GB
host uvm needs ~26 GB (the O(N²) clone), so uvm cases will be classified `parse_fail` under the
auto cap UNTIL `.3.1` lands — this is the HONEST state (uvm doesn't fit host RAM yet), and is
restored to PASS by `.3.1` (which drops uvm to MB). Reversible: `PGEN_SV_TRIAGE_MEM_CAP_KB=0`
disables the cap. KM [[sv-corpus-gate-uvm-memory]].
#### `.4.1` — ENGINE step/time watchdog — PENDING (lower priority after `.3.1`)
A hard per-parse step/time budget IN the parser that converts a would-be hang into a
`pgen_error!`-class **severity** diagnostic (always emitted, never verbosity-gated —
DIAG-SEVERITY), classified by reason. Bounded by construction; additive. DEFERRED rationale:
`.3.1` (the O(N²)→linear cure) removes the actual blowup, and the `.4.0` gate guard covers the
gate; an always-on hot-path step counter is a perf cost ([[feedback_correctness_before_speed]],
[[feedback_prefer_grammar_leave_engine_alone]]) better justified only if a residual pathology
survives `.3.1`. Revisit after `.3.1`.

### `.5` — complexity-regression gate (CI) — PENDING
Assert sub-quadratic parse-time slope on the scaling corpus; catches a hang's *precursor*
(super-linear) before it becomes a hang.

### `.6` — unbounded packrat MEMO = the remaining uvm MEMORY root (~11.6 GB) — PENDING (NEW, from `.3.1` verification)
After `.3.1` removed the per-rule clone, a RELEASE uvm parse still peaks ~11.6 GB (fluctuating
down as rules rollback). The remaining bulk is the **packrat memo** — keyed `(rule_id, position)`,
**never cleared** (`.3`/SV-EXH-PROOF.3.3.4.b.6.2.36.4: `MemoEntry` stores the AST result + a
`semantic_delta`), so over uvm's 2.89 MB × thousands of rules it accumulates millions of entries =
GB. WHY+WHERE first (heap profile / memo-size instrumentation: count entries + bytes), then a
parser-agnostic bound: e.g. position-windowed eviction, or drop memo entries whose left edge is
behind the committed frontier (packrat normally keeps all; a streaming/forward-only parser can
evict). Must stay sound (memo correctness) + not regress corpus 14/14. Likely the bigger of the
two remaining roots for a small host.

### `.7` — parser regex-match per-call redundancy (TIME) — DONE (PGEN-PARSE-TERMINATION-0008, 2026-06-05)
LANDED: `match_regex` codegen now caches `(Regex, can_match_empty)` together — `can_match_empty`
is computed ONCE at compile/insert (the `re.find("")` is now once-per-pattern, not per call) and
read as a cached bool. Confirmed in the generated parser (`find("")` count 1 = compile-time only).
VERIFIED: regen compiles; lib (features) 652/0; release uvm parse PASSES. MEASURED: release uvm
parse 181 s → **163 s (~10%)**, peak RSS ~12.7 GB (unchanged — memory is the memo, not this).
HONEST: a real but MODEST win — the `re.find("")` was a contributor, NOT the dominant cost. The
DOMINANT remaining TIME root is the match VOLUME / parse complexity (needs a fresh profile) PLUS
the gate's debug build; the MEMORY root is the memo (`.6`). Monotone (identical output).

### `.7-historical` — original finding (kept for provenance)
`match_regex` in EVERY generated parser recomputes the STATIC `can_match_empty` per call by
running `re.find("")` (executing the compiled regex against the empty string) — millions of
redundant regex executions on uvm. Also two HashMap-by-pattern-string lookups per call
(`contains_key` + `get`). FIX (codegen, parser-agnostic, monotone): cache `(Regex, can_match_empty)`
together — compute `can_match_empty` ONCE at compile/insert, read the bool per call; single
`entry()` lookup. The compiled-regex cache already exists (the `.7.4.6.1`-class pattern, here in
the parser). Measure the uvm parse-time delta. NOTE: the gate's DEBUG build is the larger time
factor (uvm ~181 s release vs >1800 s debug) → also consider a release probe for the gate's parse
step to restore corpus 14/14 (gate change, separate).

## Frontier
`.1` DONE — measurement CONFIRMED super-linear (≈quadratic-trending) stateful parsing.
`.3` root-cause DONE (O(N²) `SemanticRuntimeState` clone). **`.3.2` (2026-06-05) = a REAL-WORLD
HIT: uvm_pkg parse → ~26 GB RAM + apparent hang during the corpus gate — the failure mode this
tree exists for, observed in the wild.** This raises the priority of **`.3.1`** (the
checkpoint/rollback fix that kills the O(N²) clone) and **`.4`** (the watchdog that converts an
OOM/hang into a classified severity error — proven necessary by the 26 GB event). NEXT (director
to sequence): `.3.1` fix (careful refactor, lost-facts subtlety) and/or `.4` watchdog +
interim `ulimit`/timeout on the gate's uvm cases; plus a `.3.2` follow-up probe (pinpoint the
exact rule/depth of the memory explosion + check parser regex recompilation). Grounded in facts.
