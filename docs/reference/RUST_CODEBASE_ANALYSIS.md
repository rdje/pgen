# docs/reference/RUST_CODEBASE_ANALYSIS.md

## Recent Architecture Change Note (2026-08-22) — the reach planner steers a PATH, and until now nobody steered that path's mandatory SIBLINGS

**What moved.** `rust/src/ast_pipeline/stimuli_generator.rs` gains a third store-gate scope and a
sibling-escape analysis, both parser-agnostic. A reach plan is a list of `(rule, node_path)` hops
turned into `ReachDirective`s; it constrains only the rules ON the path from the entry to the target.
A rule that is a MANDATORY CHILD of one of those hops but not itself on the path — the classic shape
being the head identifier of `R := head_id name lparen <the plan descends here> rparen semi` — was
left entirely to the ordinary generator.

**Why that is a correctness problem and not a quality one.** Those siblings can carry parse-time
store gates. When the generator draws a gated alternative the PARSER rejects it, the enclosing choice
commits a different alternative, and the reach plan is silently lost — while the sample still parses,
because some sibling alternative accepted the same bytes. Measured on the shipped SystemVerilog
grammar: `checker_instantiation`'s `ps_checker_identifier` rendered
`known_unscoped_checker_identifier`, whose `has_fact(checker_name, …)` rejected (the declare-then-use
prelude plants `package_name` and `property_name`, never `checker_name`), so `bind_instantiation`
committed the textually identical `program_instantiation` and the target's committed count was `0`.

**The new shape.**

| piece | what it does |
|---|---|
| `StoreGateScope::ParseRejects` | positive gates ONLY — `gen_name_gate` ∪ `gen_count_kinds`. The scope a *parse* prediction needs. |
| `hop_escape_sites` / `escape_walk` | per hop, walk the hop rule's mandatory subtree; FOLLOW the on-path descent, and for every off-path `Or` with a mixed gated/ungated split record the first ungated alternative's path. |
| `install_reach_plan_from_hops` | prepends those sites' directives, so an on-path directive always wins the map and `chain.last()` still defines the plan's target. |

⛔ **The scope choice is the whole design, and the first cut got it wrong in a way that produced
zero output rather than wrong output.** `StoreGateScope::AnyQuery` — correct for the reach-BFS edge
*deprioritization*, where over-counting only costs a preference — counts `lacks_fact*` as
unsatisfiable. An empty store SATISFIES a `lacks_fact*`. So `scoped_checker_identifier`, whose only
predicate is a `lacks_fact_attribute_equals`, read as gated, the two-alternative `Or` read as "every
alternative gated", and the analysis found no escape for the very target it was built for.
`GenerationPruned` fails the other way: it deliberately excludes name gates because GENERATION never
prunes on them — but the PARSER does reject on them, and the parse is what has to be predicted.

**Blast radius, measured rather than argued.** This is the stimuli side: no grammar byte, no codegen
byte, no `generated/*.rs` byte moves. Certificate tuples before → after, one binary each:
`json 9/0/9/0`, `regex 269/9/260/0`, `vhdl 225/0/225/0`, `systemverilog_preprocessor 74/0/74/0`,
`rtl_frontend 169/1/168/0`, `rtl_const_expr 48/0/48/0` — **all identical**; SystemVerilog canonical
`1385/18/1366/1 fully_certified=false` → **`1385/18/1367/0 fully_certified=true`**, deterministic at
seeds 0/7/42. Exactly one number moved, in one family, in the intended direction.

**Observability.** `PGEN_REACH_ESCAPE_DUMP=1` prints the installed escape sites (TOOLBOX 4.4a), the
sibling of `PGEN_REACH_PATH_DUMP`'s hop chain. Presence-gated print only.

## Recent Architecture Change Note (2026-08-21) — the recursion taint SPLITS INTO TWO CHANNELS, because one field was carrying a frame index and a whole-stack fact (`ENGINE-UNIVERSAL-SERVICES.43`)

**What moved.** Every generated parser (both execution graphs) and the parse-harness interpreter
gain a second recursion-taint channel. Before, all three guard verdicts — `Infinite`,
`LeftRecursive`, and the whole-stack `MutualRecursive` depth ceiling — wrote the same field,
`recursion_block_floor`, a **parse-stack frame index**, and `SV-CORPUS-GRAD.3.12`'s memo gate
refused to cache any failure whose floor came from outside the memoized rule. Two of those
verdicts genuinely name a frame. The ceiling does not, so it passed `0` — the index outside every
rule — and the gate became true for every rule at `entry_depth >= 2`. **One ceiling trip therefore
disabled FAILURE memoisation for the whole remaining parse**, and the packrat parse degenerated
into exponential backtracking. Measured on the shipped SystemVerilog parser: 315 nested parens
0.15 s accepted, 316 0.23 s, 317 **4.90 s**, 318 no result in 30 s.

**The new shape.**

| verdict | depends on | channel | failure cached? |
|---|---|---|---|
| `Infinite` / `LeftRecursive` | which rules are on the stack (**content**) | `recursion_block_floor` (unchanged) | no — `.3.12`'s rule, untouched |
| depth ceiling | how deep the stack is (**depth**) | `recursion_depth_block_events` (new, monotone counter) | **yes**, under a depth STAMP |

Storage is deliberately per-graph and additive, and **no shared runtime type changed**: the
protocol memo gains `memo_fail_depth_gated: FxHashMap<(RuleId, usize), u32>`, the fused thin memo
gains `thin_fail_depth_gated` with the same shape, and the interpreter gains its own. ⛔ The first
cut put the stamp on `ThinTapeMemoEntry` — a type in `ast_pipeline/mod.rs` — and that is the
**STALE-ARTIFACT trap the cold-clone bootstrap documents** (`PGEN-RGX-0090`): widening a shared
runtime type makes every already-emitted artifact stop compiling, and since `regenerate_generated_
parsers` must build `ast_pipeline` *with* `generated_parsers` before it can regenerate the
families, the tree cannot bootstrap out of it. A parser-local side map has none of that coupling.

**Why the stamp is sound where the frame index was not.** The depth ceiling is **monotone in
depth**: re-entering the same rule at the same position from a deeper stack explores the same tree
with the ceiling pruning at least as much, and extra pruning cannot turn a failure into a success,
so a failure at entry depth `D` holds at every depth `>= D`. The `.3.12` gate still runs first and
returns early on any content-scoped block, so an entry only reaches the depth channel once
content-dependence is excluded — the gates compose. Both maps are probed only when
`recursion_depth_block_events != 0`, so a parse that never reaches the ceiling — every parse of
every shipped corpus file — pays one predictable integer compare and never touches them.

**Assessment.** The engine's resource bound is now a bound on TIME as well as on stack depth,
which is what `SV-CORPUS-GRAD.8c.3`'s embedding contract had claimed since `1.3.1` and could not
deliver. The generalisable risk this exposes is not about recursion: **one field was carrying two
facts of different kinds, and the fact that did not fit was encoded with the most conservative
available value.** `0` was perfectly sound and cost the entire cache. When a value is widened to
fit a channel, the cost of the widening is part of the design — and here it went unpriced for the
seven weeks between the two leaves.

## Recent Architecture Change Note (2026-08-17) — `build.rs` gains a second job, the crate gains its FIRST build-dependency, and both profiles gain a `build-override` (`ENGINE-UNIVERSAL-SERVICES.24` slice 2)

**What moved.** `rust/build.rs` had one job: resolve each generated parser's include path and emit
the `has_generated_*` cfg flags. It now has a second — **sha256 every parser it resolves** and
publish the digest as `PGEN_<FAMILY>_PARSER_SHA256` — so a compiled binary can answer *"which
generated parser am I?"*. `rust/src/bin/parseability_probe.rs` reads them back under
`--parser-fingerprint` (JSON on stdout, `option_env!` per family so an absent parser reports
ABSENT rather than a placeholder digest).

**Why it lives at build time and not at emit time.** `build.rs` already declared
`cargo:rerun-if-changed` for every parser it resolves, so it re-runs exactly when one moves — the
dependency edge the feature needs was already present and unused. The alternative (emit the
fingerprint into the generated parser) would move **every** generated artifact and re-baseline
everything keyed on them: six byte-identity controls, the parse-cost ratchet, and
`GENERATED-REPRODUCIBILITY`. This costs **zero generated bytes**.

**Two Cargo-surface changes worth knowing about, because they are firsts:**

1. `rust/Cargo.toml` gains a `[build-dependencies]` section — the crate had none — with a single
   entry, `sha2`. sha256 specifically, because the consumer re-hashes the same file with Python's
   `hashlib` and the two must agree byte for byte; `std`'s `DefaultHasher` is ruled out at the
   source (its algorithm is explicitly unstable across releases).
2. `[profile.dev.build-override]` and `[profile.release.build-override]` set `opt-level = 2`.
   ⛔ **Cargo builds build scripts at `opt-level = 0` in every profile**, and unoptimized `sha2`
   hashes the 236 MB of generated parsers in **5.5 s** per build-script run; optimized it is
   **0.39 s** (**14.1×**). This is the first thing in the crate for which build-script *speed* is
   load-bearing.

**Assessment.** The build layer is now part of the measurement-provenance story, not only of the
compile story. Anything that measures with a compiled PGEN binary can — and for the parse-cost
ratchet now must — state which parser that binary carries. The facility is engine-universal (all
nine resolved parsers are fingerprinted); only the SystemVerilog digest currently has a consumer.

## Steering Note (2026-08-15) — the OBSERVABILITY TWIN has an unbounded-cost input, and the instrument that meets it drops the row silently (`ENGINE-UNIVERSAL-SERVICES.20` slice 4 → new leaf `.22`)

**Risk picture, not an architecture change — no Rust byte moved in this slice.** It is recorded here
because it concerns an engine-universal subsystem and it was found by accident.

PGEN runs two execution graphs: the fused `cascade_*` graph a production (BARE) parse takes, and the
PROTOCOL graph any diagnostic consumer routes to. Measured on
`stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv` — **2 787 bytes**, recorded `pass` at
0.07 s:

| how it is parsed | graph | outcome |
|---|---|---|
| BARE `--parse` | FUSED | **0.077 s**, accepted |
| `--dump-rule-entry-counts-json` (TOOLBOX 3.4) | PROTOCOL, **no** coverage stack | **0.062 s**, `accepted: True`, **200 975 entries** / 711 rules |
| `--dump-rule-outcome-counts-json` (TOOLBOX 3.5) | PROTOCOL **+ transactional coverage stack** | ⛔ ~150 MB/s allocation, **4 682 MB peak in 30 s**, terminated 5 of 5 attempts, **no dump ever written** |

⇒ **The divergence is the transactional coverage stack, not the protocol routing** — 3.4 and 3.5
take the same graph and only 3.5 diverges. And it is not an input-size effect: 200 975 entries is 19×
the corpus median and **175× below** the corpus maximum, which measures fine.

Two consequences for anyone reasoning about this subsystem:

1. **The observability twin is not cost-equivalent to the shipped graph on all inputs**, and nothing
   currently bounds the gap. Every instrument built on 3.5 — the parse-cost baseline, the fusibility
   census's `--fusibility-outcome-counts` join, the memo census's `--verify` leg — inherits it.
2. ⛔ **The failure is silent by construction.** `measure_one_entries` returns `None` on
   `subprocess.TimeoutExpired`, the row lands in a `nodump` list, the count is printed and the run
   continues. That is how `.20` slice 1's *"zero no-dump rows / 16 336/16 336 agree"* became
   unreproducible (**16 335 rows, 1 no-dump**) without any gate going red.

Owned by `ENGINE-UNIVERSAL-SERVICES.22`. ✅ **ROOT-CAUSED AND HALF-CLOSED 2026-08-16
(`PGEN-ENGINE-UNIVERSAL-SERVICES-0046`).** The mechanism is one line of codegen:
`ast_based_generator.rs:9166` stores `coverage_stack[memo_coverage_checkpoint..].to_vec()` into every
`MemoEntry` and `:9049` replays it with `extend_from_slice` on every hit, so the coverage recorder
**materialises the shared parse DAG as a TREE** — the memo makes the parse linear by SHARING and the
recorder undoes exactly that. Measured growth law (`…/coverage_stack_blowup/probe.sh`): rule entries
LINEAR (+19 295 per `else if` arm, constant) while the coverage stack is EXPONENTIAL (**×4.00 per
arm**, 353 005 042 slots at 7 arms; the corpus file has 10 ⇒ ~93 GB). `/usr/bin/sample`:
`_platform_memmove` is **4302 of 4314** worker samples = **99.7 %** of CPU. Peak RSS is **13.7 GB
within one second** on a 24 GB machine, spiking then falling (`Vec` reallocation), which corrects the
gentler *"150 MB/s to 4 682 MB in 30 s"* first reading.

⇒ **(c) is DISCHARGED**: the drop is now classified by reason, DECLARED in a tracked roster with its
owning leaf (`…/parse_cost_ratchet/corpus_nodump.tsv`), and any undeclared drop **REFUSES** (exit 2)
in all three corpus-touching modes. ⛔ Not by raising the timeout — at ×4.00 per arm no timeout is
large enough.

✅ **AND (e) IS SHIPPED (`PGEN-ENGINE-UNIVERSAL-SERVICES-0047`) — THE ENGINE NO LONGER BUILDS THE
TREE, IT COUNTS IT.** `MemoEntry.coverage_delta` is now an `Option<u32>` index into an append-only
`coverage_deltas` side table; the memo INSERT moves its range out (`split_off`) and leaves one tagged
`COVERAGE_REPLAY_TAG` marker, a memo HIT pushes one marker, and `exercised_rule_entry_counts` folds
the DAG once in descending delta order carrying a `u64` multiplier — linear in recorded slots, with
the descending order discharged by construction (a delta id is allocated when a body COMPLETES, so
every marker inside it references a strictly smaller id). ⛔ The replay is KEPT: `GRAMMAR-WELLFORMED.
H.10.2.2` added it to close a real completeness hole. Results: the pathological file dumps in
**0.04 s**; the full-corpus census is **16 336/16 336, 0 no-dump** and its denominator recovered
**exactly 200 975** entries; the pinned 192-file `entries.tsv` is **BYTE-IDENTICAL**; certificate
coverage at seed 0 is **character-identical** (`witness=1496 UNKNOWN=109`); the eight pinned ladder
rungs reproduce exactly through `353 005 042`. The roster's stale-row direction was added the same
day, and the roster is now empty by design.

⛔ **A SECOND, WIDER CONSEQUENCE FOR ANY CONSUMER OF 3.5:** `raw − committed` is published as *failed
speculation* and is **not an invariant**. `committed` is the coverage stack folded per rule, so a
memo hit adds a subtree the parser never re-entered; measured `committed/entries` runs **0.62× →
2 173.88×**, i.e. the difference goes negative. It is positive on all 192 pinned-sample rows, but
that sample was selected from a census that DROPS the files where it fails. Consumers must check the
sign. ⛔ That framing — *"the fix is an engine-SEMANTICS decision"* — was **wrong and is corrected**:
the chosen design changes how the number is computed, not what it is, which the byte-identical
`entries.tsv` proves on 192 real inputs. What remains open is `.22`(b) (is a verdict agreement a
DERIVATION agreement?) and `.22`(f), opened by the fix itself: every FULL-CORPUS census taken with
the 3.5 dump while the drop existed measured 16 335 files and published a 16 336-file denominator,
so each must be re-derived or explicitly exonerated.

## Steering Note (2026-08-14) — the eliminator's ADMISSION CRITERION flipped, and one generated parser moved with it (`ENGINE-UNIVERSAL-SERVICES.17` slice 9)

**One criterion changed; `generated/systemverilog_parser.rs` is the only artifact that is not
byte-identical.** `eliminate_indirect_left_recursion` now admits
`survey.guard_admissible_candidates()` instead of `survey.safe_candidates()`, so a surviving
starvation site is closed by the guarded clone chain slice 7 emits rather than by declining the whole
knot. Measured on one binary via the new A/B lever: `systemverilog` `left_recursion_unhandled`
**28 → 0**, every other family 0 under both arms, every other SV lint counter identical.

⭐ **The architectural decision worth carrying is that the two-door SHAPE survived the flip
unchanged, and only its direction inverted.** Slice 8 made the widened admission an opt-in second
entry point so that *"the shipped call site names the shipped policy"*; slice 9 kept exactly that,
with `eliminate_indirect_left_recursion_admitting_starvation_safe_only` now holding the non-shipped
one. The invariant is direction-free: **changing what ships is an edit to
`eliminate_indirect_left_recursion`'s own criterion and can never happen by a caller passing a
different argument.** `PipelineConfig::indirect_lr_admit_starvation_safe_only` (default `false`) and
`--indirect-lr-admit-starvation-safe-only` are the lever, and no `make` target reaches them.

⛔ **Three consequences a future change in this area must respect.**
1. **`indirect_guard_chains=` is no longer a zero-check.** The old protection — *"0 on every shipped
   grammar, non-zero means the admission changed"* — cannot distinguish "the criterion holds" from
   "the pass stopped running". It is replaced by an EXACT-SET check on the shipped chains
   (`guard_dry_run` D11/D11a/D11b), which fails on a silent revert, an accidental widening, and a
   moved guard position alike.
2. **A census bank is pinned to a moving target unless it names its admission.** Four banks measure
   *"the knots the eliminator has NOT absorbed"*; when the knots were absorbed, three of them went
   RED **in the direction that reads as success** (`0/28 → 0/0` looks like "no starved candidates").
   `guard_feasibility` and `guard_dry_run` now pass the lever explicitly, with the reason at the
   variable. This is `CI-PARITY-GATE-ROT.29`'s class with the extraction CORRECT and the subject
   moved.
3. **A report about the shipped path must describe the shipped path.** The `🛡` renderer lived only
   in the opt-in dry-run block, because the shipped count was 0 by construction. It is now one shared
   renderer with a distinguishing VERB (`guard` vs `would guard`), because both blocks print in one
   report and a scraper must be able to tell a fact from a prediction.

⇒ **Assessment change for the note below:** its closing sentence — *"Flipping the admission is the
shipped-parser change and is owed the two-sided repro ratchet plus a corpus re-measure"* — is
discharged, not pending. Both were run and are recorded in the `.17` slice 9 acceptance box.

## Steering Note (2026-08-14) — the eliminator gained a SECOND synthesis stage, and it is silent on the shipped path by construction rather than by a flag (`ENGINE-UNIVERSAL-SERVICES.17` slice 7)

**A new engine capability landed; nothing a generated parser executes changed.** `plan_elimination`
now has a step 6 — `plan_guard_chains` — that turns a candidate's SURVIVING starvation sites into
call-site-scoped guarded clone chains, and `apply_plan` installs them and repoints the holders' left
corners. `IndirectEliminationOutcome` gained `synthesized_guards`, surfaced as `guard_chains=` /
`guard_rules=` / the `🛡` lines in the dry-run report and as `indirect_guard_chains=` on the ordinary
one.

⭐ **The architectural decision worth carrying is the ABSENCE of a switch.** The stage runs on every
plan. What keeps it from touching a shipped parser is that a guard chain exists only for a surviving
starvation site, and the shipped admission (`CandidateAdmission::StarvationSafe`) is *defined* as
having none — so a second gate here would be a second thing that has to agree with the criterion, and
two things that must agree can disagree. The consequence is measurable rather than argued: all 11
generated parsers re-derive byte-identical, and `indirect_guard_chains=0` on every shipped grammar.
⛔ **Both of those numbers moved at slice 9** — see the note above. The ABSENCE-of-a-switch decision
did not: the guard stage still runs on every plan, and what it emits is still decided by the
candidate's own surviving starvation sites rather than by a second gate.

⛔ **Two constraints a future change in this area must respect.**
1. `render_elements` stays byte-frozen — `indirect_lr_elimination`'s ambiguity refusal compares its
   output, so the new lookahead parenthesisation lives in `render_elements_display` only.
2. The guarded base rule must stay POSITIONALLY IDENTICAL to the base rule it stands in for (same
   first element, same quantified rule reference, one appended lookahead). That is what keeps
   `lr_chain_fold`'s `$1`/`$2` meaning what they meant, and it is why the loop guard is hoisted into
   `X_lr_guard{v}_suffix` instead of being written as an inline group.

⇒ **Assessment change for the note below:** the dry run's grammar is no longer the unguarded one. The
bound moves inward, not away — nothing here generates a parser or runs an input, so `28 -> 0` is
still plan-stage reachability. Flipping the admission is the shipped-parser change and is owed the
two-sided repro ratchet plus a corpus re-measure.

## Steering Note (2026-08-13) — the elimination driver's ADMISSION POLICY is now a parameter, and that is what let the census be measured through the real planner (`ENGINE-UNIVERSAL-SERVICES.17` slice 3)

**One driver gained a parameter; nothing the parser executes changed.**
`indirect_lr_elimination.rs` gained `CandidateAdmission` (`StarvationSafe` | `GuardFeasibleDryRun`),
and `eliminate_indirect_left_recursion` now delegates to
`eliminate_indirect_left_recursion_with_admission` with `StarvationSafe`. The widened mode is
reachable only from `dry_run_guard_feasible_elimination`, which works on a clone and is called only
by the opt-in `--indirect-lr-plan-guard-dry-run` report section.

⭐ **The architectural point is that the alternative was worse.** The question — *"of the
guard-feasible candidates, how many actually reach a plan?"* — is a question about what the DRIVER
does, and the three refusals that answer it (annotation composability at `compose_route_template`,
the trial re-lint, the ambiguity comparison) are all unobservable for a candidate the starvation gate
refuses. A second implementation of the plan stage would have drifted from the one it predicts;
parameterising the admission set keeps exactly one planner in the tree.

⇒ **Assessment change for the note below:** the guard is no longer only *expressible*; the plan it
unlocks is *buildable*. `would_absorb=2 would_refuse=0 clone_rules=24 left_recursive_rule_rows 28 ->
0` on `systemverilog` — the 16 guard-feasible candidates are 16 rules on TWO knots, absorbed at
`casting_type` and `property_expr`. ⛔ Bound, and it is the same one: the dry run emits no guard, so
that is plan-stage reachability, not closure. ⭐ One new architectural fact worth carrying:
`systemverilog.ebnf` declares return annotations on **1069** rules while the LRM-generated
`systemverilog_lrm_profiled_wrapper` declares **21** — the two views are comparable in structure and
NOT in annotation, and any oracle that crosses that line is measuring the wrong grammar.

## Steering Note (2026-08-13) — the survey now carries a MEASUREMENT beside its conservative verdict, and it moves the fix's size (`ENGINE-UNIVERSAL-SERVICES.17` slice 2)

**One analysis module grew; nothing the parser executes changed.**
`rust/src/ast_pipeline/indirect_lr_plan.rs` gained a guard-feasibility layer
(`GuardVerdict` / `GuardAssessment` / `GuardFirstBytes`, `assess_guard`) that calls the shared FIRST
machinery (`ast_pipeline/first_set.rs::branch_first_set`) the pass had never used, and
`rules_transparent_to` became a DEPTH map so the guard's clone-chain length is measured rather than
guessed. `rust/src/main.rs` prints two new headline lines, one per-candidate `guard:` line and one
per-site column, plus six JSON fields.

⛔ **The architectural point is the restraint, not the addition.** `is_starvation_safe` — the verdict
`indirect_lr_elimination.rs` acts on — is deliberately UNCHANGED. Promoting the refinement into it
would change which knots the eliminator absorbs, i.e. a real parser change owed a two-sided repro
ratchet ([[a-conservative-criterion-and-a-measurement-are-different-objects]]).
⛔⛔ **SUPERSEDED AT SLICE 9 — the promotion happened, and the ratchet was PAID rather than waived.**
Six slices separate this restraint from that change, and each one was a step the next could have
refuted (decide → model → emit → execute → flip). The card still holds: a conservative criterion and
a measurement stayed different objects for exactly as long as the measurement was unproven.

⇒ **Assessment change for the note below:** the greedy `*` is still the blocker, but it is now priced.
`guard-feasible 16/28` against `starvation-safe 0/28`, and both remaining SystemVerilog knots are
feasible at their dominators with ONE guard variant each — `constant_primary` at one clone hop,
`property_expr` at zero. The fix `.17` has to build is a **generation-time synthesis onto the clone
the eliminator already emits**, not an engine-core change to the quantifier. ⛔ Bound: a byte-test
guard is defeated by an intervening comment, because `trivia` is nullable and leads every token —
sound (it can only fail to fire), not exhaustive; slice 3 prices a structural lookahead against it.

## Steering Note (2026-08-13) — the give-back asymmetry is the engine's own, so `.17`'s fix drops a tier (`ENGINE-UNIVERSAL-SERVICES.17` slice 1)

**No Rust changed. What changed is the assessment of what `.17` has to build**, and the note below
(*"blocked by the greedy non-backtracking `*`"*) should be read with this correction.

The blocker was framed as PEG's possessive repetition, implying an **engine-core** change — make the
LR-eliminated `*` re-enterable — with the packrat memo as the exposed surface. Measured
(`docs/tasks/artifacts/engine_universal_services/quantifier_policy/probe.sh`, 7/7 self-checking):

- **PGEN already gives back at a CHOICE and only refuses to at a QUANTIFIER.** The default
  `@branch_policy` is `longest_match` — `generate_or` evaluates every alternative and keeps the
  longest (`ast_based_generator.rs:4273`, selection priority → longest → associativity at `:4387`),
  and a successful-but-losing alternative is retried when a later element fails. The quantifier loop
  (`:6041-6091`) takes the maximum count and never revisits it. ⇒ the asymmetry is **between two of
  our own combinators**, not between PGEN and PEG.
- **The multi-attempt cost is already paid everywhere and is controlled by STATIC ELISION**
  (`degenerate_dispatch_byte_sets`, `:4556`; and at the quantifier, the `RGX-0078.5.i.7` Q-GUARD at
  `:6006`). ⇒ the peak-speed non-negotiable is implemented here as *"prove the cost away at
  generation time"*, not as *"never attempt twice"* — which is the bar a `.17` design must clear.
- **A per-iteration lookahead stop-guard closes the starvation with no re-entry and no memo
  exposure**, and must be **call-site scoped** (a rule-global guard breaks the caller that has no
  residual). The `.13` eliminator already emits sheared clones, so the injection point exists.

⇒ the front-runner moves from an engine-core change to a **gen-AST → gen-AST rewrite** — the highest
fix tier, and the one `ENGINE-UNIVERSAL-SERVICES.7`'s taxonomy prices as free at parse time by
construction. Durable form: [[project_pgen_gives_back_at_neither_combinator]] — ⛔ which SUPERSEDES
`project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier` (`.17` slice 4): the choice does
not give back either, so the starvation has TWO sources and the repair needs a guard in TWO
positions. The fix-tier conclusion above is unaffected — it is still a gen-AST → gen-AST rewrite.

## Recent Architecture Change Note (2026-08-13) — the indirect-LR rewrite lands, and it verifies its own postcondition (`ENGINE-UNIVERSAL-SERVICES.13` slice 5)

**New module `rust/src/ast_pipeline/indirect_lr_elimination.rs` — the first engine pass that
changes the grammar based on the slice-4 survey.** It runs inside
`eliminate_left_recursive_patterns`, after the direct normalization and the one-hop wrapper planner,
and rewrites a whole multi-hop ROUTE: `X := X_lr_base ( X_lr_suffix )*`, one sheared CLONE per
intermediate, one per-route `X_lr_suffix_r<N>` rule. Iterates to a fixpoint, re-surveying after each
rewrite. Measured: `ebnf` 5 → 0 surviving cycle rows, SystemVerilog 30 → **28**, every other
shipped grammar untouched; `generated/systemverilog_parser.rs` is the only artifact that moves.

⛔⛔ **The version that reported 30 → 13 was a REGRESSION, and only the two-sided reproducer ratchet
saw it.** The starvation criterion scanned holders of the BASE rule; it must also scan holders of
anything TRANSPARENT to it, because a bare-reference alternative is *consumption*-transparent and
inherits the rewritten rule's greed. `casting_type := … | constant_primary` therefore hid
`cast := casting_type tick lparen expression rparen`, and `initial k = 8'(1);` went ACCEPT → REJECT
while every other signal stayed green. `rules_transparent_to` (a least fixed point over
empty-residual left-corner edges) closes it, and SystemVerilog then reports
`starvation-safe candidates: 0/28` — its cast/call knot is **unplannable by chain absorption at any
base rule**, blocked by the greedy non-backtracking `*` (`ENGINE-UNIVERSAL-SERVICES.17`).

**The seam with `lr_chain_fold` did NOT need widening, which was the surprise.** The design note
above expected the fold to generalize "from one rule to a mutually-recursive set". It does not: a
route's per-hop declared annotations **compose** into a single `LrChainWrapperSpec` template (each
hop's `$1` filled by the hop below, every other `$N` remapped by `offset + N` into the flattened
suffix), and `fold_lr_chain` consumes it unchanged. The generalization is entirely in the SPEC
builder, so the one shared runtime service stays one function with one contract.

**A new architectural pattern worth naming: TRIAL-THEN-COMMIT inside the pass.** The route walk is
narrower than the lint (bare left corners, simple paths only), so a plan can silently half-shear a
cycle. Rather than argue coverage, the pass applies each plan to a *copy*, re-runs
`detect_left_recursion`, and commits only if the base rule's cycle is gone and the row count
strictly fell — one grammar clone per rewrite, single-digit rewrites per grammar. It caught a real
defect on its first run and is what moved the fix's target rule for the third time. ⇒ where a pass's
precondition is cheaper to *check after* than to *prove before*, put the check in the pass.

**⛔ A COPY-A-GRAMMAR-OBJECT HAZARD this module had to close, and every future one will too.** The
clone builder first copied only per-BRANCH annotations, so `@profiles:` — a RULE-level annotation —
was dropped and the sv_2017 constant primary became reachable under an `sv_2023` parse. Three
defects sat behind that: the clone's lost gate, the `_lr_base`/`_lr_suffix` helpers as profile
ORPHANS, and the need for a suffix branch to be a **rule** rather than an alternative (an
alternative cannot carry `@profiles:`, and two dialect routes iterate byte-identical suffixes with
different declared ASTs). ⇒ when a transform copies a grammar object, enumerate what is attached to
the original **by kind** — branch return / branch semantic / mid-sequence / rule-level semantic /
lexical follow-restriction — not by the fields that came to mind. Only `@profiles:` is copied onto
the helpers, because it is the one rule-level directive that says whether the rule *exists*; the
rest say what happens when it *runs*, and the base rule still runs.

**Exposure:** on by default; `--no-eliminate-indirect-left-recursion` (and
`PipelineConfig::eliminate_indirect_left_recursion`) holds the grammar at the direct/wrapper-only
shape so a before→after is measurable on one binary.

## Recent Architecture Change Note (2026-08-12) — indirect left recursion gets a PLANNER before it gets a rewrite (`ENGINE-UNIVERSAL-SERVICES.13` slice 4)

**New module `rust/src/ast_pipeline/indirect_lr_plan.rs` — pure analysis, no engine behaviour.** It
answers, for every cycle `--lint-grammar` reports as `left_recursion_unhandled`, what an elimination
would have to DO: the left-corner ROUTES from a candidate base rule back to itself, the SUFFIX the
rewrite would iterate (`X := X_lr_base ( X_lr_suffix )*`), the CLONE cost, and every **starvation
site** — a rule holding the candidate at its left corner with a non-empty residual.

**Why a separate module rather than an extension of `detect_left_recursive_chain_plan`.** The
existing planner (`ast_pipeline/mod.rs`) matches ONE shape and mutates the grammar as it goes; this
one enumerates every candidate across the whole grammar and mutates nothing, which is what makes it
safe to run on all 17 grammars and to unit-test against a synthetic. Keeping it out of `mod.rs` also
keeps the analysis reviewable — the survey's criterion was refuted twice by measurement before it
was right, and both refutations were found by running it, not by reading it.

**The seam it defines for the eventual rewrite.** `ChainRoute::suffix_elements()` /
`intermediate_rules()` are the two functions a generalized `apply_left_recursive_chain_plan` needs:
the suffix is the route's residuals concatenated INNERMOST FIRST (a derivation descends to the
cycle-closing reference and consumes each residual on the way back out), and each intermediate is
one clone rule. Measured on `grammars/systemverilog.ebnf`: 13 clones for the cast/call knot, 12 for
the method-call receiver, 1 for class scope — linear in the cycle length, and each clone is a new
name in the typed AST (`ast_shape_contract`).

**Exposure:** `ast_pipeline --report-indirect-lr-plan` (+ `--indirect-lr-plan-json`,
`PGEN_INDIRECT_LR_DUMP_ALL`), a read-only CLI branch that returns before any generator runs, so no
generation path can reach the new code.

## Recent Architecture Change Note (2026-08-11) — the left-recursion chain FOLD is one shared runtime service (`ENGINE-UNIVERSAL-SERVICES.8`)

**New module `rust/src/ast_pipeline/lr_chain_fold.rs` — the single implementation of "an
LR-eliminated rule returns the AST its grammar declared."** Left-recursion elimination
(`rewrite_lr_chain_annotations` in `ast_pipeline/mod.rs`) hoists an author's per-alternative return
annotations into a spec table and rewrites the rule into `base (suffix)*`. Folding that back into the
declared left-nested shape was, until now, **not done anywhere** on the emitted-AST path, so every
LR-eliminated rule of every grammar published the eliminator's internal record.

**The seam this creates, and why it is shaped this way.** The chain value is produced by three
independent emitters — `AstReturnTransformer` (protocol graph, `ParseContent`),
`ast_based_generator::cascade::value::dv_value_transform_expr` (fused bare-parse graph, `PgenValue`)
and `parse_harness_interpreter::fold_return` — and each of them now emits or performs a **call into
the same function**. That is a deliberate departure from the usual pattern here (each emitter
implements a construct in its own vocabulary, with the differential gates proving they agree): those
gates prove the three agree *with each other*, and all three leaked identically for the life of the
feature. Making agreement structural frees the gates to assert the property that actually matters —
the AST against the **declaration**.

- Carrier: a new `UnifiedReturnAST::LrChainFold { initial, suffixes, specs }` variant replaces the
  synthetic `Object{type: "_pgen_lr_chain", …}`. The variant, not a recognized object shape, is the
  point: `rustc` enumerated the **14** consumers (validator, fusibility census, both emitters, the
  interpreter, the shape gate, two unparse surfaces, the legacy string generator, the pretty-printer)
  that had been silently ignoring the synthetic object.
- Cost posture: the codegen-embedded spec JSON is parsed **once per process** behind an emitted
  `OnceLock`; the interpreter interns its table to `'static` via the module's `intern_specs`, so the
  fold borrows template strings straight into the produced `PgenValue`s and allocates nothing per
  iteration. The multi-hundred-byte `wrapper_specs` literal is no longer a field of every emitted
  chain value.
- Failure direction: `validate_chain_templates` runs at GENERATION time and refuses a template the
  fold cannot express (`$text`/`$0`, quantified extraction, nested chain, out-of-range positional),
  naming the rule and the construct — a build failure instead of a silently wrong AST.
- Reserved namespace: `ENGINE_INTERNAL_TYPE_PREFIX` (`_pgen_`) is now a contract. No grammar author
  may declare it, and `auto_return_annotation_shape_gate` FAILS any emitted value carrying it — the
  standing regression guard for this whole defect class, not just for left recursion.

## Recent Architecture Change Note (2026-07-22) — the dedicated 256 MiB parse stack (`SV-CORPUS-GRAD.8c.3`)

**New module `rust/src/dedicated_parse_stack.rs` — big-stack execution for generated-parser
entry boundaries.** The generated parsers' 4096-frame recursion ceiling only protects the
process when the calling thread has real stack for 4096 frames (measured for SV: ≈2 KB/frame
release / ≈17 KB/frame debug ⇒ ≈8 MB / ≈70 MB needed, vs the 8 MB default main stack — a
~400-deep parenthesized SV expression SIGABRTed a release process pre-fix). The module offers
`run_on_dedicated_parse_stack` (spawn-per-call, panic captured, re-entrant calls run inline
via a thread-local flag) and the CLI wrapper `run_cli_main_on_dedicated_parse_stack`
(panic `resume_unwind`). Three boundary loci route through it: the embedding-API SV/VHDL
family entries in `embedding_api.rs` (worker panic → `E_PARSE_FAILURE`; embedding API
`1.3.0` → `1.3.1`), and the whole `main` bodies of `parseability_probe` and `ast_pipeline`
(thread-local trace/dump config stays coherent because setup + parses share the one thread).
Integration seams to know: the REGEX embedding path is deliberately untouched (it keeps its
own RGX-0085 64 MiB worker + nesting pre-check — the perf-floor law); nothing inside any
parse loop changed; the shared ceiling constant is untouched; new engine-level code is
thread-plumbing only.

## Recent Architecture Change Note (2026-07-15, second)

**The cyclic-spine fold + the taint-classed thin memo (`RGX-0078.5.i.7` D2-B,
`PGEN-RGX-0078-0090`, session #126).** The fused cascade graph (see the D2-A note below)
now extends through each grammar's RECURSIVE core: the emitter consumes the shared plan
at `CascadeIncrement::CyclicSpine`, so every cascade-eligible rule fuses (regex 234 fns;
VHDL folds into ONE region rooted at its entry rule) and fused functions may recurse.
Two lean protocol parts are retained exactly where a cycle makes them load-bearing, on
cyclic INTERNAL fused rules only: (1) the protocol-mirror recursion-guard frame
(`check_cycle` + `enter`/`exit` — infinite/left-recursion detection scans for same-rule
in-flight frames, exact iff every cyclic rule pushes in both graphs); (2) a THIN memo —
the new engine type `ThinMemoEntry` in `ast_pipeline/mod.rs`: `position → (end, value)`
with NO stored deltas, so entries carry the protocol memo's own taint classes measured
across the body by three engine counters (write epoch / deferred-obligation count /
predicate evaluations): PURE replays at any store state, STORE-READ replays under the
unchanged-epoch license, STORE-MUTATING is never cached (honest re-execution — a
value-only replay would skip effects). Cyclic SUB-ROOTS get neither lane: fused bodies
reference sub-roots through their protocol METHODS (real memo + guard). Integration
seams to know: the emitter hard-bails if the census's `rule_reaches_itself` and the
generator's `compute_recursive_rules` ever disagree on a fused rule (one cyclicity
resolution), and CHANGING AN EMITTED TYPE's fields (the `MemoEntry`/`ThinMemoEntry`
class) breaks every generator build against the on-disk artifacts — the recorded unbreak
is an exact-semantics transient migration of `generated/*.rs`, never a bootstrap-binary
regen (annotation-payload degradation).

## Recent Architecture Change Note (2026-07-27)

**Grammar configuration is now resolved at codegen, not re-asked in the emitted parser
(`GENERATED-LINT-CORRECTNESS.1`, `PGEN-GENERATED-LINT-CORRECTNESS-0002`, session #214).**
Several per-grammar configuration constants used to be interpolated into the emitted
parser as string/bool **literals** and then compared there — e.g. `#branch_policy_mode ==
"ordered"`, `skip_leading_whitespace && #allow_layout_skip_for_regexes`. rustc folded
them, but the generated artifacts still carried every non-selected branch, and `clippy`
reported the identical-operand cases as `deny`-by-default **correctness** errors
(`eq_op`, `overly_complex_bool_expr`), which made the generated-parser lint stage
impossible to run strictly. The generator now resolves `SemanticBranchPolicy` and the
`@whitespace_sensitive` layout policy itself and emits **only the decided form**.

⚠️ **The seam an editor must know: this shape lives in THREE emitters, one per parse
graph** — `ast_based_generator.rs` (protocol / memoized),
[rust/src/ast_pipeline/ast_based_generator/cascade.rs](../../rust/src/ast_pipeline/ast_based_generator/cascade.rs)
(fused cascade / bare) and
[rust/src/ast_pipeline/ast_based_generator/scan.rs](../../rust/src/ast_pipeline/ast_based_generator/scan.rs)
(scan / match-only). Each carries its own copy of the branch tournament, so **a codegen
change to one is a change to all three**, and a `rust/src/ast_pipeline/*.rs` glob does not
reach the latter two. Measured effect across the two leaves that closed the class
(`.1` branch policy + regex layout; `.2` associativity, per-rule `@negative_case`, and the
terminal/trailing layout guards): **30,880** statically-decided expressions removed, and
the eleven generated artifacts fell from **241.1 MB to 210.8 MB**
(`systemverilog_parser.rs` 149.8 → 130.6 MB) — which more than gives back the fused
graphs' growth recorded in the 2026-07-15 note below. Behaviour is unchanged and
differentially proven: the interpreter still evaluates these directives at runtime from
their enums, and the parse-harness combinator suite requires it to agree byte-for-byte
with the folded generated parser.

⚠️ **Three emitted bindings are now conditional**, because folding a guard away can orphan
what the guard reached: `consume_layout_for_terminal` is emitted only when a grammar is
layout-insensitive on terminals or on trailing layout; `nonassoc_tie` and its failure arm
only under `@associativity: nonassoc`; and `best_branch_index` only where something reads
it — which in `scan.rs` also gates `current_branch_index`, but in `cascade.rs` gates
nothing, because its tape writes `DerivEvent::OrWinner(best_branch_index)`
unconditionally. An editor changing one emitter must check the same seam in the other
two rather than assume symmetry.

## Recent Architecture Change Note (2026-07-15)

**The observability twin — dual-graph parser emission (`RGX-0078.5.i.7` D2-A,
`PGEN-RGX-0078-0087`, session #124).** Every generated parser is now emitted with TWO parse
graphs. The protocol methods (`parse_<rule>`) are unchanged — memoization, counters,
transactional coverage, trace, semantic transactions. Alongside them, the census-planned
acyclic effect-free regions gain one compact fused function per rule
(`cascade_<rule>`), emitted by the new child module
[rust/src/ast_pipeline/ast_based_generator/cascade.rs](../../rust/src/ast_pipeline/ast_based_generator/cascade.rs)
from the SHARED plan `fusibility_census::compute_cascade_emission_plan` (the
`compute_inline_decisions` no-drift pattern). Routing is a per-parse boolean
(`bare_parse`): a parse with no diagnostic consumer (no coverage, no trace, no
counter-handle taker, no memo-stats env) enters the fused graph through a dispatch at each
plan sub-root's memoized body; ANY diagnostic consumer keeps the full protocol graph, so
every counter/witness/trace surface remains the exact machinery rather than an emulation.
Store soundness inside fused code is per-site: speculation scopes whose subtree can reach
a semantic effect (the plan's `effect_targets`) run under `try_parse`, and alternations
with such branches keep the protocol tournament as an island. Integration seams to know:
the `rule_call_counts()` accessor now MARKS the parser as counter-observed
(interior-mutable `Cell` — taking the handle is the routing request), and
`parse_from` (entry-relative) always runs the protocol graph. Byte-identity is enforced
by the interpreter-equivalence gate (all registered grammars run their fused graphs on
bare parses), the AST-dump oracles, and the PCRE2 conformance corpus. Cost note: the
fused graphs grew every artifact (regex ×1.11; SV +5.5%) and pushed the heavy compile
classes to ≈12.4–12.5 GB peaks — the memory-guard budget for those jobs is now an
explicit 16384 MB (see the host-RAM decision record).

## Recent Architecture Change Note (2026-07-08)

**Generation-side store gates (`STIMULI-SIGNOFF.13.4` + `REGEX-PCRE2-FIDELITY.3.17`, session
#65).** The stimuli generator gains a declarative surface for CROSS-REFERENTIAL constraints whose
parse-time evaluation is unsound (forward references legal — the regex scan-substring capture
list): two new StimuliSteering directives, `@gen_emit_fact` / `@gen_predicate`, with EXACTLY the
`@emit_fact`/`@predicate` payload schemas (shared parsers in `semantic_runtime.rs`; validator
lints; never serialized into parser artifacts — codegen fast-path pinned). Architecturally they
fold into the EXISTING store-aware generation maps (`compute_store_aware_gen_directives` →
`gen_emit_facts`/`gen_count_kinds`), so the `.3` count-prune and the C2.2/`.4b.17` count-prelude
witness arming apply unchanged; the new pieces are (a) per-rule render-time VALUE DRAWS
(`GenValueDraw::NameOf` — the rule's whole render drawn seeded-deterministically from live fact
names; `::IndexUpTo` — drawn from `1..=count(K)`), hooked in `generate_rule` beside the
literal-hint override, and (b) `count_gate_via_mandatory_descent` — the count-gate analogue of the
`.4b.4`/`.4b.7` name-gate mandatory descent, so a NON-gated carrier whose mandatory render forces
an inner count-gated value rule still arms a producer prelude when targeted. Draws key ONLY off
`@gen_predicate` (parse-time `@predicate` generation behavior byte-identical — svpp certs/corpora
stash-A/B proven). First consumer: regex's scs capture list (the last hunter-visible duality
class, closed; regex cert 224/224 with the new rules witnessed).

## Recent Architecture Change Note (2026-07-06)

**The parse-harness subsystem is complete as a verified second parse implementation
(`PARSE-HARNESS.2`–`.6.2`, sessions #37–#47).** Five sibling modules now form a coherent
arbitrary-grammar parse capability alongside the registry path: `parse_harness.rs` (the
compile-and-run oracle — real codegen compiled as a throwaway external crate, authoritative by
construction), `parse_harness_interpreter.rs` (the in-process grammar-AST interpreter — a dynamic
dispatcher over the normalized gen-AST that reuses the shipped `ParseNode`/`ParseContent`/
`semantic_runtime` types verbatim and, since `.6.2`, mirrors the FULL semantic-directive
orchestration — the rule-transaction skeleton, tournament C3-B delta discipline, branch-start inline
actions, the `$reference` resolver family, and the split packrat memo with semantic-delta replay —
from the emitted codegen templates, and, since `STIMULI-SIGNOFF.13.2` (2026-07-08), the SC-08
value-constraint atom guards `@enum`/`@regex`/`@range`/`@len` through the shared
`effective_rule_value_constraints` registry resolution codegen compiles the emitted guards from),
and the three certifying oracles: `parse_harness_equivalence.rs`
(byte-identity vs the shipped generated parsers over a deterministic corpus — 11 grammars CERTIFIED),
`parse_harness_combinator_suite.rs` (per-structural-combinator isolation, 27 cases as of
PROFILE-ALIAS), and `parse_harness_semantic_suite.rs` (per-semantic-construct isolation, 29 cases
as of STIMULI-SIGNOFF.13.2 — the counts at first landing were 16/20). Architectural
significance: the interpreter is a genuine second implementation of PGEN's parse semantics whose
divergence surface is pinned mechanically per construct, which (a) gives every linter/authoring
question an in-process arbitrary-grammar probe with no codegen, and (b) doubles as an executable
specification of the generated parsers' orchestration (the emitted `quote!` templates now have a
readable, tested mirror). The `.6.2` differential work also pinned several engine behaviors as
grammar-author facts (store-blind memo failure cache; variant-sensitive fact-name matching;
rule-wide inline branch-predicate flattening; the `$`-stripped positional-reference dead path) —
see the parse-harness book chapter and `docs/tasks/PARSE-HARNESS.md` §21.

## Recent Architecture Change Note (2026-06-10, later)

**Layout-skipper comment-introducer guard for regex terminals (`GRAMMAR-WELLFORMED.H.11.3`,
vhdl release `1.0.4`, ledger `VHDL-0002`).** The emitted layout skipper
`consume_layout_for_regex` ([rust/src/ast_pipeline/ast_based_generator.rs](../../rust/src/ast_pipeline/ast_based_generator.rs))
hard-codes `#`-to-end-of-line, `//`, and `/* */` comment skipping into every
non-blocklisted generated parser. Its string-terminal sibling
`consume_layout_for_terminal` has always guarded against swallowing
comment-introducer tokens (`expected != "#"` …); the regex side had no such
guard, so a regex terminal that IS a comment introducer — VHDL's
`hash := trivia /#/` — had its own token consumed as a "comment" and
`based_literal` failed on every input since the family's first release. The
fix adds the symmetric guard: each comment arm now calls the new emitted
helper `regex_token_matches_at_cursor(pattern)` (anchored match at the cursor,
cold-path sibling regex cache) and stands down when the active token's own
pattern matches at the introducer. `consume_layout_for_regex` also moved
inside the `uses_match_regex` conditional (it is only called from
`match_regex`), so fully-literal parsers (the regex grammar) no longer carry
it as dead code. Knock-on behavior worth knowing: explicit comment-token
rules (`line_comment`, `block_comment`) in grammars that model trivia
explicitly (SV, rtl_frontend) can now match directly at a comment instead of
being pre-empted by the skipper — comment ownership is grammar-faithful. That
surfaced one pre-existing generator-side `/`↔comment fusion hazard in
generated SV samples (ticketed `GRAMMAR-WELLFORMED.H.11.5`); acceptance for
real-token inputs is unchanged (the guard only fires where the old behavior
self-defeated the active token).

## Recent Architecture Change Note (2026-06-10)

**Memo-hit coverage-delta replay (`GRAMMAR-WELLFORMED.H.10.2.2`).** The shared packrat
`MemoEntry` ([rust/src/ast_pipeline/mod.rs](../../rust/src/ast_pipeline/mod.rs)) gains a
`coverage_delta: Option<Vec<u32>>` field, and the emitted `memoized_call`
([rust/src/ast_pipeline/ast_based_generator.rs](../../rust/src/ast_pipeline/ast_based_generator.rs))
now (a) snapshots the transactional coverage stack's length before executing a rule body,
(b) stores the body's pushed coverage entries in the `MemoEntry` on success (`None` — no
allocation — when coverage recording is disabled, the ordinary-parsing default), and (c)
replays that delta onto the live coverage stack on every memo hit. This closes the
memoization × transactional-record composition gap for the certifying-linter witness record —
the same defect class `.b.6.2.36.4` closed for the semantic store (`semantic_delta` replay):
a memo hit bypasses the rule body, so without the replay a subtree first parsed inside a
rolled-back speculation and then memo-hit on the committed path was silently absent from
`exercised_rule_names()`. Engine-wide rule for future work: **any new transactional per-rule
record added to generated parsers must also be delta-captured in `MemoEntry` and replayed on
hits** (see the `memo-hit-transactional-replay` knowledge card). All generated parsers were
regenerated with the new shape; parse outcomes are byte-identical (the change affects only
the opt-in diagnostic coverage record).

## Recent Architecture Change Note (2026-07-16) — the committed-value representation: `ParseContent::Shaped(PgenValue)`

**Arena value carrier (`RGX-0078.5.i.7` REPRESENTATION, `PGEN-RGX-0078-0104`/`-0105`).** [rust/src/ast_pipeline/pgen_value.rs](../../rust/src/ast_pipeline/pgen_value.rs) introduces `PgenValue<'input>` — a `Copy` enum (`Null/Bool/Int(i64)/UInt(u64)/Float(f64)/Str(&'input str)/Array(&'input [PgenValue])/Object(&'input [(&'input str, PgenValue)])`) whose composite payloads are slices owned by `NodeArena` (now a struct owning the node arena plus shaped-value/pair/rendered-string arenas; `new()`/`alloc()` signatures preserved, so every artifact call site compiles unchanged). `ParseContent` gains `Shaped(PgenValue<'input>)` carrying `#[serde(rename = "Json")]`, and `PgenValue`'s manual `Serialize` mirrors `serde_json::Value`'s formatter variant-for-variant — so the released typed-AST JSON is **byte-identical** (no schema bump) while the in-parse representation drops `BTreeMap` nodes, template-key `String`s, and the double deep-clone per nested reference. The return-annotation emitter builds objects as codegen-time-sorted static-key stack arrays bumped once into the arena; property access is a sorted-slice binary search (byte-exact `Map::get`); `ParseContent::to_shaped_value(arena)` / `PgenValue::from_serde(value, arena)` / `to_serde_value()` convert between the carriers at boundaries. All 11 generated parsers construct `Shaped` and reference the transitional `Json` variant **nowhere**, so its retirement is a lib-only change. Constraint worth knowing: `typed_arena::alloc_extend` holds the arena's internal borrow while draining its iterator — composite conversions materialize into a `Vec`/stack array first (documented at the helpers). Measured: regex cold-parse geomean **−32.0%** (≈8.90µs → ≈6.11µs), proven byte-identical by the byte-identity oracle suite, the all-grammar differential-equivalence gate, a 16/16 dump tripwire, and the regex typed-hook differential gate.

## Recent Architecture Change Note (2026-07-12) — `ParseNode` moved onto a bump arena

**Node arena (`RGX-0078.5.d.4.i`, `PGEN-RGX-0078-0031`).** [rust/src/ast_pipeline/mod.rs](../../rust/src/ast_pipeline/mod.rs) gains `pub type NodeArena<'input> = typed_arena::Arena<ParseNode<'input>>`, and the child-bearing variants of `ParseContent<'input>` now hold **borrowed references** into that arena rather than owned allocations: `Sequence(Vec<&'input ParseNode<'input>>)`, `Alternative(&'input ParseNode<'input>)`, `Quantified(Vec<&'input ParseNode<'input>>, …)` (previously `Box<ParseNode>` / `Vec<ParseNode>`). Every generated parser struct gains an `arena: &'input NodeArena<'input>` field; the constructor signature is `new(input, &arena, logger)` and child construction sites call `arena.alloc(child)`. The arena is created per parse at the call boundary and freed en masse (running each leaf's `Drop`) when the parse result is consumed. Motivation: the profiled ~55% construction `malloc` collapses to a handful of arena growths — measured **−21.9%** regex geomean, byte-identical.

Implications for the Rust codebase shape:

- **Single `'input` lifetime, not two.** Reborrowing collapses the arena's lifetime onto the input's, so `ParseNode`/`ParseContent` keep the one `'input` parameter they already had — no viral `<'input, 'arena>` split. The change is mechanical, compiler-checked ref-threading across ~23 tracked source files (`parser_registry`, `embedding_api`, `ebnf_frontend`, `ast_shape_contract`'s `run_manifest` 3rd arg, the interpreter, fixtures, and the perf/differential bins), plus the codegen quote! templates in `ast_based_generator.rs` / `ast_return_transform.rs` (including the emitted `#[cfg(test)]` harness).
- **Public API unchanged; wire format unchanged.** Every public entry point still returns *owned* output (a `serde_json::Value` or its string), and — the decisive design property — a borrowed child serialises **identically** to an owned one under `#[derive(Serialize)]`, so the authoritative typed-AST bytes the oracles compare are byte-identical for free. This is why the *reference*-arena shape (candidate B, `typed-arena`) was chosen over an *index*-arena shape (candidate C, `NodeId(u32)`), which would have forced a hand-rewritten serialiser — the very oracle. See [[project_rgx_0078_regex_slowness_followup]].
- **New dependency:** `typed-arena = "2.0.2"` (a Drop-running arena; a naïve non-Drop bump allocator would leak the leaves' heap-owned strings/JSON every parse).

## Purpose
Live architecture and state assessment for the Rust codebase.

This document exists to preserve the high-level understanding needed to steer implementation, review future changes, and ramp up a new session without having to rediscover the whole Rust stack from scratch.

This is a live document, not an archival write-up. It should be amended whenever the Rust architecture, major risks, public integration surfaces, or codebase shape materially change.

## Session-Start Maintenance Rule
- Review this file at the start of any new session that materially touches the Rust codebase.
- Refresh it when the current codebase no longer matches this snapshot in a meaningful way.
- Prefer amending this file over scattering duplicate architectural assessments into ad hoc chat history.
- Keep historical detail in `CHANGES.md` / `DEVELOPMENT_NOTES.md`; keep this file focused on current structure, current risks, and the current steering picture.

## Scope Of This Assessment
- This is a source-structure and architecture assessment of the maintained Rust-first platform.
- It covers the main Rust crate, the generated-parser integration layer, the major Rust-owned binaries, and the Rust-owned gate/build ecosystem around them.
- It is not a claim that every parser family is closed.
- It is not a replacement for the live closure status in `docs/book/src/roadmap-and-live-status.md`.
- It should be read alongside the roadmap priority rule:
  - active parser-family closure work is now centered on the remaining main-`systemverilog` debt and still outranks deferred maintainability refactors.
  - tracing is now an explicit repo doctrine for future tools and operational binaries:
    - new Rust-owned tools should expose the shared five-level trace contract:
      - `none`
      - `low`
      - `medium`
      - `high`
      - `debug`
    - prefer `TraceVerbosity`, `PGEN_TRACE_VERBOSITY`, and the existing `pgen_trace*` macro family over bespoke debug flags or scattered prints
    - instrument real control-flow seams, not just final errors:
      - function entry/exit
      - meaningful branch decisions
      - fallbacks
      - retries
      - timeout paths
      - failure boundaries
  - annotation placement is now an explicit frontend-semantics rule for the active main-SV proof lane:
    - standalone semantic annotations above a rule definition are the maintained rule-level form
    - same-line inline semantic annotations inside the rule body are branch-local, even when the rule has only one alternative
    - the retained `module_ansi_header` / `module_nonansi_header` / `program_ansi_header` / `program_nonansi_header` repair depended on this distinction: the first inline attempt landed in `branch_semantic_annotations` and therefore did not behave like ordinary rule-level `@sample` steering
  - profile symmetry is now an explicit root-cause check for stubborn replay debt:
    - before assuming the runtime ignored a branch literal hint, compare the active profile’s rule body against the sibling profile’s steering surface
    - the retained `net_declaration_sv_2023` repair came from exactly that check:
      - `net_declaration_sv_2017` already carried helper-only probes for `wire a;`, `nt a;`, and `interconnect logic a;`
      - `net_declaration_sv_2023` had the same three structural branches but none of those `@probe_sample` footholds
      - a direct `debug` trace confirmed the runtime was descending through a full branch because the hint surface was missing, not because branch-level hint consumption was broken
  - recursive replay debt now has one sharper steering rule too:
    - when the replay-gap artifacts show that the open debt is owned by a recursive parent branch, prefer steering that parent branch directly over probing the child rule named in the first visible failure reason
    - the retained `sequence_expr::root#11` SystemVerilog repair is the current reference example:
      - child-rule probes on `clocking_event_sv_2023` regressed the bounded replay frontier
      - a branch-local helper probe on the parent alternative `clocking_event sequence_expr` improved the frontier and retired the separate `clocking_event_sv_2023::root#2` debt
    - treat that as a concrete replay-triage heuristic, not as a one-off anecdote
  - same-line inline semantic-annotation payload parsing in `rust/src/ebnf_frontend.rs` is now an explicit correctness seam rather than scanner trivia:
    - the maintained rule is that same-line inline annotations consume only their own payload (quoted, balanced, or scalar) and must leave following rule-body syntax intact
    - this recently removed a false-epsilon branch leak from the focused main-`systemverilog` replay lane
    - the retained adapter-backed branch-sample probes now both land at `180/180` accepted with `0` parser rejections, so the remaining main-SV direct-lane debt is back to target-resolution/coverage closure rather than a known inline-annotation parsing seam
  - `vhdl`, `systemverilog_preprocessor`, and `regex` have reached their current published closure bars and should be treated as no-regression proof baselines unless their contracts are intentionally widened.
  - downstream regex hardening should now be treated as maintained PCRE2-compatibility contract precision work, not as a reason to reopen the `regex` family row by default.
  - RGX's downstream review now treats regex handoff `1.1.29` / integration contract `1.1.31` as the current public handoff. That line carries the accepted-tree transport fixes, named recursion-condition support, returned-capture subroutine forms, strict PCRE2 VERSION conditionals, quoted literal handling both outside and inside character classes, quoted class range endpoints, short Unicode property escapes, bounded variable-length lookbehind, Unicode capture names, orphan class `\E` as a zero-width scanner marker, dedicated `\C` single-code-unit escape transport, callout-prefixed conditional assertions, PCRE2 POSIX word-boundary aliases, DEFINE-in-lookbehind zero-width handling, UTF width start-option aliases, scan-substring forward-reference validation, plain class `\N` rejection, nonliteral class range endpoint rejection, decoded escaped class-range endpoint ordering including bare octal `\NNN`, POSIX-class fallback/adapter clarifications, MARK/PRUNE/SKIP/THEN payload generalization, braced padded `\g{...}` / `\k{...}` references, generated-host depth resilience for legal deep PCRE2 inputs, and the 2026-04-14 through 2026-04-18 source-derived PCRE2 audit slices.
  - `ci_workflow_local_gate` now also audits against that current regex handoff (`1.1.29` / `1.1.31`) and the current user-guide regex parseability total (`5911`), so filtered local workflow replays no longer fail on stale regex-public-surface literals.
  - `ci_workflow_local_gate` is now also self-pruning on success: it still exports a tracked-only `run.*` tree under `rust/target/ci_workflow_local_gate`, but successful runs delete that scratch directory by default while failed runs retain it for triage; `PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS=1` preserves successful runs when a bundle needs to be inspected or archived.
  - PCRE2 compatibility work has a clear source-of-truth workflow:
    - read `pcre2syntax(3)` and `pcre2pattern(3)` for documented intent
    - cross-reference `src/pcre2_compile.c` for exact edge cases
    - validate against upstream `testdata/testinput*` plus expected outputs as the executable oracle
  - remaining regex caveats are now deliberate scope-widening questions around full PCRE2 parity, stronger JS/Lua shielding, host-language wrapper parsing, and any future AST-semantic stability promises beyond the current JSON schema contract.
  - current Phase S `rtl_frontend` generated-contract work is proof tightening rather than closure promotion:
    - generate-local `typedef` and named `import` declarations are now part of that retained proof surface:
      - handwritten `rtl_frontend` parsing now allows those declarations inside `generate` bodies
      - the implementation keeps generate-body alias scope local by snapshotting/restoring `type_aliases` around `parse_generate_body(...)`
      - the generated contract now carries dedicated positive samples for generate-local typedef-backed and generate-local named-import typedef-backed struct-member actuals
      - selector-only syntax parameter overrides now also have a bounded semantic lane:
        - ordered constant part-select overrides such as `MASK_BITS[HI:LO]`
        - named constant selector overrides such as `MASK_BITS[7:4]` and `MASK_BITS[IDX]`
        - package-qualified selector actuals and selector-only overrides such as `cfg_pkg::MASK_BITS[7:4]`
      - handwritten assignment-target validation now keeps that widening on the value side only:
        - package/global constant roots are valid RHS actuals and bounded selector-only overrides
        - those same roots still reject as assignment targets
      - elaboration replay minimums now ratchet to `59` total samples, `46` accepts, `13` rejects, `20` child-path samples, `37` top-parameter checks, `21` child-parameter checks, and `85` child-port-binding checks
    - `expected_rule_texts` in `rtl_frontend_generated_contract_probe` remains the exact full-vector retained-text assertion
    - `required_rule_texts` remains for subset retained-text assertions over recursive rules such as `conditional_expr`, `additive_expr`, `shift_expr`, and `signal_reference`, where the salient span should be proven without freezing every incidental scalar expression subtree
    - the generated contract now tightens `always_ff_well_formed`, `always_star_scalar_if_else_block`, `always_latch_scalar_nonblocking_block`, and `always_latch_unknown_body_identifier_parse_surface` so scalar procedural lanes prove retained port-shell context and selected `signal_reference` evidence around already locked event-control, event-edge, procedural-block, assignment-operator, assignment-target, and procedural keyword spans
    - the generated contract now tightens `procedural_and_dataflow_concat_member_paths`, `procedural_and_dataflow_ternary_binary_exprs`, `rich_assignment_targets_ternary_exprs`, `procedural_concatenated_assignment_target_ternary_exprs`, and `continuous_ranged_member_assignment_target_ternary_exprs` so mixed procedural/dataflow lanes prove retained parameter, port, inline struct, struct-field, net-declaration, packed-range, and unpacked-dimension context where applicable around already locked `always_comb`, continuous-assignment, assignment-target, concatenation, ranged-reference, and ternary/binary expression spans
    - the generated contract now tightens `always_star_rich_assignment_targets` and `always_latch_rich_assignment_targets` so rich plain `always @(*)` / `always_latch` lanes prove retained parameter, port, inline struct, struct-field, net-declaration, packed-range, and unpacked-dimension context around already locked procedural-block, assignment-operator, assignment-target, continuous-assign, and concatenation-expression spans
    - the generated contract now tightens `always_comb_struct_member_concatenation_target` so the isolated `always_comb` struct-member concatenated-target lane proves retained parameter, port, inline struct, struct-field, net-declaration, packed-range, and unpacked-dimension context around already locked `always_comb`, procedural-block, assignment-operator, and assignment-target spans
    - the generated contract now tightens `always_ff_rich_nonblocking_assignment_targets`, `always_ff_struct_member_bitselect_nonblocking_target`, `always_ff_struct_member_concatenation_value`, and `always_ff_unknown_event_identifier_parse_surface` so `always_ff` lanes prove retained parameter, port, inline struct, struct-field, net-declaration, packed-range, unpacked-dimension, and syntax-only unknown event identifier context around already locked event-control, procedural-block, nonblocking assignment, assignment-target, concatenation, ranged-reference, and expression spans
    - the generated contract now tightens `scalar_named_parameter_override_and_named_ports` and `parameterized_instance_array_with_named_ports` so hierarchy lanes prove retained child/top parameter declaration context, parameter group context, and scalar packed ranges around already locked named overrides, named ports, symbolic instance-array ranges, expression evidence, and signal references
    - the generated contract now tightens `unpacked_array_struct_member_actual` and `unpacked_array_element_actual` so unpacked-array actual lanes prove retained parameter context around already locked array/member actual text
    - the generated contract now tightens `module_local_parameter_and_localparam_items` so module-local parameter/localparam syntax proves retained parameter-sequence vectors, `parameter` / `localparam` keyword text, port-list context, and selected recursive expression evidence, while the shared elaboration replay now also locks the smaller `arithmetic` top-parameter environment (`WIDTH = 8`, `DEPTH = 12`, `EXTRA = 13`, `TOTAL = 16`) instead of only implying it through the larger integrated arithmetic sample
    - the generated contract now tightens `labeled_always_comb_block` and `labeled_always_comb_parameter_exprs_and_packed_multi_nets` so labeled procedural samples prove retained `begin` / `if` / `else` keyword text, declaration/range/port context, and selected recursive expression evidence around the existing procedural-block locks
    - the generated contract now tightens `generate_for_symbolic_limit_nonunit_stride` so symbolic non-unit generate-for syntax proves retained `generate` / `for` / `genvar` keyword text plus the `parameter LIMIT = 5` parameter sequence/group around the existing loop-expression evidence
    - the generated contract now tightens `generate_if_with_dataflow_and_named_instantiation`, `generate_if_else_with_dataflow`, and `generate_if_else_with_local_net_declarations` so generate/dataflow lanes prove local net declarations, local packed range, parameter-sequence, port-list, and ternary-expression retained context around already locked generate structures, while the shared elaboration replay now also locks the parameterized local-net branch lane directly through `WIDTH = 8`, `TOTAL = 16`, and zero child instances instead of only implying that shape through neighboring generate samples
    - the generated contract now tightens continuous struct-member assignment lanes again so their `struct_union_field` vectors exact-lock `logic [7:0] data;` and `logic valid;` across bit-select, unknown-member, concatenated-target, and concatenation-value samples
    - the generated contract now tightens `continuous_struct_member_bitselect_assignment_target`, `continuous_unknown_struct_member_target_parse_surface`, `continuous_unknown_struct_member_value_parse_surface`, `continuous_unknown_struct_member_concatenated_target_parse_surface`, `continuous_struct_member_concatenation_assignment_target`, and `continuous_struct_member_concatenation_value` so continuous struct-member assignment lanes prove inline struct declaration, input-port context, and `BIT` parameter context around assignment targets and values
    - the generated contract now tightens `named_port_bitselect_and_concat_actuals`, `named_port_member_bitselect_and_repeat_actuals`, and `named_port_actual_ternary_member_paths` so named-port actual expression lanes prove declaration and parameter context around bit-select, concatenation, repetition, and ternary member-path actuals
    - the generated contract now tightens `ordered_parameter_and_port_actual_repetition`, `ordered_actuals_repeat_concat_member_ranges`, and `named_port_actuals_repeat_member_ranges` so repeat-concat actual lanes prove retained parameter, port, net, packed-range, unpacked-dimension, and struct-field context around already locked ordered/named actual expressions
    - the generated contract now tightens `ordered_parameter_override_ternary_binary_expr`, `named_parameter_override_repeat_expr`, and `named_parameter_override_ternary_binary_expr` so parameter-override expression lanes prove retained child/top parameter declarations, port shells, and packed ranges around already locked override expressions
    - the generated contract now tightens `unpacked_array_struct_member_actual`, `unpacked_array_element_actual`, and `unpacked_array_struct_member_bitselect_actual` so indexed unpacked-array actual lanes prove richer member-path, inline struct body, array declaration, packed/unpacked dimension, and module-instantiation retained text while leaving array-indexing and parameter semantics in elaboration
    - the generated contract now tightens `unindexed_unpacked_array_struct_member_actual_parse_surface` and `unknown_inline_struct_member_actual_parse_surface` so inline struct-member actual lanes subset-lock the inline struct body, with `cfgs.data` plus `[0:1]` proven on the unindexed unpacked-array lane and `struct packed { ... } cfg;` proven on the unknown inline-member lane
    - the generated contract now tightens `named_port_union_member_actual` and `named_port_unknown_union_member_actual_parse_surface` so inline union-member actual lanes exact-lock the union body and `payload` net declaration, with the known-member lane also exact-locking the successful `payload.data` signal-reference path
    - the generated contract now tightens `packed_union_width_mismatch_parse_surface` and `builtin_integral_packed_union_width_mismatch_parse_surface` so inline and builtin-integral packed-union mismatch lanes exact-lock full module declarations, simple output-port shells, union bodies, datatype/range or builtin keyword spans, and final net declarations while leaving semantic width evaluation in elaboration
    - the generated contract now tightens `typedef_backed_struct_member_actual`, `file_scope_typedef_backed_struct_member_actual`, `package_wildcard_import_typedef_backed_struct_member_actual`, `package_named_import_typedef_backed_struct_member_actual`, `header_named_import_typedef_backed_struct_member_actual`, `unknown_typedef_backed_struct_member_actual_parse_surface`, and `typedef_backed_packed_union_width_mismatch_parse_surface` so typedef-backed struct-member actual lanes exact-lock typedef declarations and struct bodies, while the typedef-backed packed-union mismatch lane exact-locks typedef declaration, union body, and packed ranges
    - the generated contract now tightens `inline_struct_typed_net_declaration`, `inline_union_typed_net_declaration`, `typedef_union_named_net_declaration`, and `typedef_enum_named_net_declaration` so inline aggregate typed-net and typedef-backed aggregate named-net lanes exact-lock full module declarations, port shells, builtin datatype vectors, packed ranges, aggregate type bodies, typedef declarations, named data-type uses, and final net declarations where applicable
    - the generated contract now tightens `typedef_struct_named_net_declaration`, `file_scope_typedef_struct_named_net`, `file_scope_typedef_struct_port_and_net_multimodule`, and `package_typedef_struct_port_and_wildcard_net_multimodule` so local/file-scope/multi-module/package struct typedef lanes exact-lock typedef declarations, struct bodies, builtin datatype vectors, packed ranges, ANSI port shells, full module declarations, and `cfg_t cfg;` net declarations where applicable
    - the generated contract now tightens `package_qualified_typedef_struct_port`, `package_wildcard_import_typedef_struct_named_net`, and `package_named_import_typedef_struct_named_net` so package/import struct typedef lanes exact-lock compact typedef declarations, struct bodies, builtin datatype vectors, packed ranges, package-qualified/named data-type uses, port shells, and imported `cfg_t cfg;` net declarations where applicable
    - the generated contract now tightens the header-imported typedef port family so `header_imported_enum_typedef_port`, `header_imported_union_typedef_port`, and `header_imported_struct_typedef_port` exact-lock imported named-type uses, ANSI port-list/group shells, compact typedef/type bodies, relevant builtin datatype vectors, and packed ranges
    - the generated contract now tightens `multiple_empty_modules_without_port_lists` so the no-port multi-module lane exact-locks both `module` keyword spans, module identifiers `first` / `second`, and both `endmodule` keyword spans alongside the existing full declaration locks and forbidden-rule checks
    - the generated contract now exact-locks statement-level `parameter_declaration_statement` retained text for module-local parameter/localparam items and package-backed constant-flow samples, while header parameter-port declarations remain covered by `parameter_declaration_head` / `parameter_declaration_tail`
    - the generated contract now exact-locks aggregate typed `net_declaration` retained text for inline enum/union declarations and typedef-backed struct/enum/union named net uses
    - the generated contract now retains the integrated handwritten-baseline arithmetic/procedural/generate sample `arithmetic_integrated_generate_and_procedural_flow`, combining dependent parameters, ANSI port ranges, module-body parameter/localparam statements, packed nets, continuous ternary dataflow, labeled `always_comb`, generate `if/else`, and generate `for` with exact retained text for compact structural rules and subset retained text for recursive expression spans
    - the generated contract now retains the integrated child/generate hierarchy sample `integrated_child_parameter_generate_instances`, combining direct, generate-if, and generate-for child instantiations with named parameter overrides and named port connections while explicitly keeping elaboration semantics out of the generated-parser proof claim
    - the generated contract now retains `generate_for_symbolic_limit_nonunit_stride`, proving symbolic generate-for bounds plus non-unit step syntax (`i < LIMIT`, `i + 2`) without claiming generated semantic unrolling
    - the generated contract now tightens `parameterized_instance_array_with_named_ports` so symbolic instance-array range text (`LANES-1`) and both `LANES` signal-reference uses are proven alongside parameter/port declaration context and exact hierarchy spans
    - the generated contract now tightens `unpacked_array_ports_and_nets` so packed/unpacked dimensional declaration text, the full port-list/port-group surface, and repeated `DEPTH` references are proven for the foundational unpacked-array declaration lane
    - the generated contract now tightens `builtin_integral_atom_typed_net_declarations` so builtin datatype text, builtin keyword text, and the simple output-port shell are proven for the `byte` / `shortint` / `longint` declaration lane
    - the generated contract now tightens `inline_enum_logic_typed_net_declaration` so duplicate `logic` datatype spans, enum base/range text, full enum body text, and the output-port shell are proven for the inline enum declaration lane
    - the generated contract now tightens `inline_enum_byte_base_typed_net_declaration` so `logic` / `byte` datatype spans, `byte` keyword text, full enum body text, and the output-port shell are proven for the byte-base inline enum declaration lane
    - the generated contract now exact-locks generate `if/else` structural retained text for the existing dataflow and local-net branch samples by requiring `generate_region`, `generate_if`, and branch-level `generate_body` spans
    - the generated contract now exact-locks compact hierarchy retained text for package-backed constant-flow `module_instantiation`, unpacked-array struct-member `instance_item`, and generate-contained `module_instantiation` / `instance_item` spans
    - the generated contract now exact-locks the single-branch generate-if named-instantiation sample by requiring `module_instantiation` and exact `generate_region`, `generate_if`, `generate_body`, full `module_instantiation`, and `instance_item` retained text
    - the generated contract now exact-locks generate `for` structural retained text for the existing local-net and named-instantiation/dataflow loop samples by requiring `generate_region` and branch-level `generate_body` spans, with exact hierarchy locks for the looped instantiation sample
    - the generated contract now also locks the plain unpacked-array element named-port actual `child u_child (.a(banks[IDX]), .y(y));` plus a nearby malformed `banks[]` rejection, matching the handwritten baseline's simpler unpacked-array element actual lane without changing the live status label
    - the generated-contract gate now also runs handwritten `rtl_frontend::parse_design` and ratcheted optional `expected_elaboration` replays over the same `125`-sample manifest; the manifest still has zero active `expected_handwritten_parse_ok` annotations after the handwritten parser was aligned to reject the two mixed named/ordered instance-list cases, reject scalar `always_ff` blocking assignments at parse time, accept selector/concat-rich runtime expression text at the parse boundary, and carry package-qualified selector roots through the handwritten semantic lane, and it now retains at least `59` semantic replay samples (`46` accepts / `13` rejects), `20` child-path samples, `37` top-parameter checks, `21` child-parameter checks, and `85` child-port-binding checks for procedural blocks, isolated module-local parameter/localparam replay, parameterized generate-if/else local-net replay, hierarchy/instance arrays, package constants, aggregate member actuals, union-width checks, unknown event/member diagnostics, unindexed unpacked-array members, unknown parent actual identifiers, scalar named-override/named-port replay, scalar wildcard expansion, ordered positional parameter-and-port actual repetition, ordered actual member-range repetition replay, selector-only ordered/named parameter-override replay, package-qualified selector actual plus parameter-override replay, syntax-only ordered/named parameter-override rejection replay including the named ternary/binary override lane, package-constant assignment-target rejection, single-branch generate-if named-instantiation elaboration replay, generate-if/else dataflow elaboration replay, generate-for named-instantiation/dataflow elaboration replay, named-port member-range repetition replay, descending-range instance-array wildcard/named-port expansion, and richer signal/member, bit/part-select, concat, repeat, expression, and expression-text actual shapes
    - the live `rtl_frontend` row remains `In Progress` until broader generated grammar exhaustiveness, semantic elaboration parity, and parser-stack closure are achieved
  - downstream regex hardening under `regex_corpus_bundle/` now also has two distinct external-corpus roles:
    - `regex_pcre2_textsafe_corpus_gate` for accepted-syntax widening
    - `regex_pcre2_compile_oracle_gate` for compile-truth comparison against pinned PCRE2 source truth
  - the 2026-04-14/2026-04-15 regex audits deliberately split grammar syntax from compile-contract enforcement:
    - `grammars/regex.ebnf` now models more PCRE2 source-derived syntax such as short Unicode property escapes, quoted class literals, quoted class range endpoints, escaped quoted-literal body characters, bounded variable-length lookbehind surface, Unicode capture-name characters, orphan class `\E` as `stray_class_end_quote`, `\C` as `single_byte_escape`, callout-prefixed conditional assertions, `\K`, one-digit and whitespace-braced hex/octal escapes, string callouts, `(*atomic:...)`, non-atomic symbolic and alpha lookarounds, scan-substring groups, script-run groups, strict inline modifier letters, strict no-whitespace `VERSION>=...` / `VERSION=...` conditionals, and comma-only `{,}` as literal text rather than a counted quantifier
    - `rust/src/regex_compile_validation.rs` now mirrors PCRE2 compile-time checks that are not clean grammar productions: unbounded lookbehind rejection, malformed named references, PCRE2 capture-name byte/first-character limits, malformed short Unicode properties, empty quoted or orphan-quote class regions, plain class `\N` rejection, shorthand/property class range endpoint rejection, numeric callout range, start-option and verb-name tables, non-ACCEPT verb quantification, `\K` in lookaround, forbidden character-class escapes, POSIX class-name validation, scan-substring capture-list existence, unsupported default escapes, and `(?R1)` rejection
    - the maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` result of `1843` matches, `352` mismatches, `307` false accepts, and `45` false rejects over `2195` normalized compile-oracle cases
  - the current HDL closure tactic has now shifted away from broad hint sweeps:
    - when a family is down to a small stubborn replay/rejection set, prefer the new branch-level triage tooling over more blanket grammar/sample nudges
    - the stimuli runtime now also has a narrower replay-only steering seam:
      - ordinary literalish directives (`@sample`, `@literal`, `@example`, legacy `@stimulus`, and bare string payloads) remain active in their existing regex / non-regex / branch-local use sites
      - `@probe_sample` is now the maintained active-entry-only replay accelerator for broad dependency rules
      - current focused `systemverilog` evidence is mixed but useful:
        - 2017 direct replay improved from `333/2619` to `365/2619` resolved targets in 5 attempts
        - the matching 25-attempt replay stayed flat at `831/2619`
      - treat this as a retained stimuli-platform seam that still needs another measured follow-up before claiming broader family-level payoff
    - the retained `systemverilog_preprocessor` orphan-closer seam is now solved on the focused and aggregate lanes by a stimuli-only generator fix: repeated `pp_item` expansions are forced onto separate lines when the previous generated item did not already end with newline
    - current retained preprocessor truth is now:
      - focused quality `33/33/0/0` with `final_targets=0`
      - aggregate contract `parseability_rejected_total=0`, `parseability_parser_rejections_total=0`, `counterexamples_captured_total=0`
      - zero-plausible-gap proof is now explicit and green through `sv_preprocessor_zero_plausible_gap_proof_gate`
      - formal exhaustive closure is now green through `sv_preprocessor_formal_exhaustive_closure_gate`
    - main-SV replay-shadow triage now also has one important reporting guard:
      - primary-entry parser-rejection counts remain the tracked debt surface
      - alternate-entry probe failures are still useful telemetry, but they should not be mixed into the main parser-debt counterexample set
      - the retained Rust path now carries entry-context metadata through target-drive validation so the replay-shadow counterexamples can stay primary-entry-only
    - treat `systemverilog_preprocessor` as a closed no-regression baseline unless its published contract is intentionally widened; active HDL closure work should now bias toward the remaining `systemverilog` main-parser debt and the broader Phase `S` build-out

## Rust-Adjacent Cargo Surface
- Main product crate
  - `rust/Cargo.toml`
  - This is the primary crate for parser generation, runtime parser exposure, embeddings, operational binaries, and the main proof-facing Rust surfaces.
- Active companion crates
  - `rtl_const_expr/Cargo.toml`
    - standalone constant-expression crate used by the RTL/frontend track
  - `rtl_frontend/Cargo.toml`
    - standalone frontend crate layered on top of `rtl_const_expr`
  - These are not part of the main `rust/` crate, but they are still part of the project’s live Rust implementation story.
- Auxiliary/peripheral Rust crates
  - `tools/generators/Cargo.toml`
    - auxiliary generator-tool surface, not the main runtime/proof spine
  - `test_parsers/json_test/Cargo.toml`
    - narrow test/example parser surface, not a central product architecture owner
  - root `Cargo.toml`
    - repository-local Rust surface, but not the canonical source for the main parser-generation architecture described here

Current toolchain floor:
- the maintained Cargo packages in this repository now declare:
  - `rust-version = "1.95"`
- treat Rust `1.95` as the explicit MSRV for the repo's maintained Rust surfaces unless a future task deliberately raises it again

Operational rule:
- When a task says “the Rust codebase,” default to `rust/` first, then pull in `rtl_const_expr/` and `rtl_frontend/` when the task touches Phase S or frontend/constant-expression ownership.
- Do not let peripheral Cargo manifests distract from the main architecture unless the task is explicitly about those support crates.

## Rust-Facing Repo Doc Crosswalk
- `README.md`
  - Use for:
    - repo-level orientation
    - current doc map
    - first-hop navigation into the maintained project surfaces
    - high-level entrypoints into aggregate annotation proof surfaces
- `QUICKSTART_AI_ONBOARDING.md`
  - Use for:
    - session-start ramp-up expectations
    - which current docs a new session should read first
- `PGEN_USER_GUIDE.md`
  - Use for:
    - user-facing workflow framing
    - understanding which operational surfaces are meant to be consumed externally
    - operator-facing map of aggregate annotation / semantic / return local gates
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md` and `docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md`
  - Use for:
    - downstream parser handoff
    - family-specific integration promises and caveats
    - build/availability checks a host project should perform before relying on a parser family
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
  - Use for:
    - downstream parser bug-report bundles
    - exact repro artifacts PGEN expects back from host projects
    - trace / AST-dump / structured-outcome capture procedure for integration bugs
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  - Use for:
    - release-support continuity
    - tracking accepted downstream parser bugs through fix and release
    - linking real integration bugs back to proof and regression artifacts
- `docs/book/src/roadmap-and-live-status.md`
  - Use for:
    - current closure/status truth
    - distinguishing architecture work from “family actually closed” claims
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
  - Use for:
    - current project doctrine
    - phase/closure expectations
    - deferred or still-open engineering directions
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - Use for:
    - return/semantic annotation meaning
    - typed-annotation intent that should constrain Rust-side parser/generator/runtime changes
    - annotation proof obligations and gate targets behind aggregate annotation claims
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
  - Use for:
    - semantic-steering behavior expectations
    - understanding whether a semantic-runtime or stimuli/generator change still matches repo policy
- `COMMIT.md`
  - Use for:
    - workflow and continuity expectations
    - knowing when `docs/reference/RUST_CODEBASE_ANALYSIS.md` itself should be refreshed as part of a task
- `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`
  - Use for:
    - short-term continuity
    - recent implementation history
    - crash/handoff recovery
- `regex_corpus_bundle/README.md` and `regex_corpus_bundle/docs/regex_corpus_plan.md`
  - Use for:
    - PCRE2-first external regex hardening doctrine
    - corpus acquisition / normalization layout
    - the maintained difference between text-safe widening and compile-oracle measurement lanes
- `rust/docs/COVERAGE_GAP_TRIAGE.md`
  - Use for:
    - branch-level closure debugging on the remaining HDL seams
    - joining gap-report, coverage, and grammar-AST artifacts into one readable triage surface
    - avoiding more blind `@sample` / lexical hint sweeps when a family is down to a small stubborn target set

Operational rule:
- If a Rust task raises a question about doctrine, status, semantics, or workflow, reach for the matching repo doc instead of trying to infer everything from code alone.
- The code explains implementation; these docs explain whether the current implementation is aligned with project intent.

## Executive Summary
- PGEN's Rust codebase is not just a parser implementation. It is a parser-generation and parser-proof platform.
- The center of gravity is the AST pipeline in `rust/src/ast_pipeline/`, especially:
  - `mod.rs`
  - `ast_based_generator.rs`
  - `stimuli_generator.rs`
  - `annotation_validator.rs`
  - `semantic_runtime.rs`
- The generated parser path, stimuli/coverage closure path, semantic-steering path, and proof/gate path are deeply integrated rather than loosely bolted together.
- The strongest quality of the Rust codebase is coherence around determinism, observability, and machine-checkable proof.
- The maintained tracing reference surface is already centralized rather than fragmented:
  - `rust/src/ast_pipeline/mod.rs` owns `TraceVerbosity`, `PGEN_TRACE_VERBOSITY` resolution, trace-log routing, and the `pgen_trace*` macro family
  - `rust/src/main.rs`, `rust/src/bin/pgen_ast.rs`, `rust/src/bin/ebnf_dual_run_diff.rs`, and `rust/src/bin/parseability_probe.rs` already consume that common surface instead of inventing separate verbosity models
- The main architectural risk is concentration of complexity in a few very large modules and a few repeated adapter seams.
- The newest downstream-trust expansion for `regex` is no longer just synthetic or narrative: `regex_corpus_bundle/` now feeds a maintained compile-oracle lane through `normalize_pcre2_compile_oracle.py`, `regex_corpus_probe`, `regex_pcre2_compile_oracle_gate.sh`, and a dedicated post-parse compile-contract layer in `rust/src/regex_compile_validation.rs`; the generated regex host path also retains a larger bounded worker stack plus widened generated recursion guard for legal deep PCRE2 conformance inputs.

## Snapshot Metrics
- Rust maintained source surface inspected in this pass: about `44k` lines.
- Biggest source hotspots:
  - `rust/src/ast_pipeline/stimuli_generator.rs`: `7907` lines
  - `rust/src/ast_pipeline/ast_based_generator.rs`: `7046` lines
  - `rust/src/ast_pipeline/annotation_validator.rs`: `4014` lines
  - `rust/src/main.rs`: `3183` lines
  - `rust/src/ast_pipeline/mod.rs`: `2920` lines
  - `rust/src/ast_pipeline/semantic_runtime.rs`: `2522` lines
  - `rust/src/ast_pipeline/unified_return_ast.rs`: `2625` lines
- Rust-owned shell gate scripts under `rust/scripts/`: `58`

## What The Rust Codebase Actually Is
- A grammar-to-parser pipeline:
  - `grammars/*.ebnf -> raw AST / JSON -> generated/*.rs`
- A parser generator:
  - AST-based Rust parser emission via `syn`, `quote`, and `prettyplease`
- A typed annotation platform:
  - return annotations
  - semantic annotations
  - validation and runtime-steering layers
- A stimuli-generation and coverage-closure platform:
  - in-memory stimuli generation
  - gap reporting
  - target planning
  - replay/closure-oriented telemetry
- A public integration surface:
  - parser registry
  - embedding API
  - grammar-profile-aware parse entrypoints
- A proof/gate system:
  - build orchestration
  - closure/status/contract sidecars
  - release/SOTA aggregate gates

## Major Architectural Layers

### 1. Core AST Pipeline And Grammar Normalization
Primary files:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/return_annotation_handler.rs`
- `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
- `rust/src/ast_pipeline/mutual_recursion_handler.rs`
- `rust/src/ast_pipeline/ast_return_transform.rs`

Role:
- Defines the central IR for grammar transformation and parse-tree handling.
- Normalizes raw AST into the grammar tree used downstream.
- Handles branch/rule annotations.
- Performs left-recursion elimination and related normalization work.
- Provides shared runtime types:
  - parse node/content types
  - recursion/memoization machinery
  - trace/logging support

Assessment:
- This is the real heart of the crate.
- A lot of project doctrine is encoded here, not just in docs.
- It is powerful, but `mod.rs` itself is large enough that understanding the full transform pipeline now requires careful re-reading.

### 2. Parser Code Generation
Primary files:
- `rust/src/ast_pipeline/ast_based_generator.rs`
- `rust/src/ast_pipeline/ast_code_generator.rs`
- `rust/src/ast_pipeline/ast_generator_direct.rs`

Role:
- Turns normalized grammar AST into generated Rust parser source.
- Uses AST/token generation instead of raw string concatenation.
- Emits parser implementations with:
  - memoization and recursion guards
  - recovery telemetry
  - coverage-target telemetry
  - negative-case telemetry
  - deterministic-partition telemetry
  - semantic-runtime hooks

Assessment:
- The project is not generating “simple recognizers”; it is generating instrumented parsing systems.
- `ast_based_generator.rs` is one of the most important files in the repo.
- The AST-based codegen approach is a real strength because it reduces syntax-generation fragility.
- The downside is that too much emitted-parser policy is encoded in one giant generator module.

⛔ **There are FOUR emission surfaces, not one, and a per-atom decision has to reach all of them.**
`ast_based_generator.rs` emits the PROTOCOL graph; `ast_based_generator/cascade.rs` emits the fused
`cascade_match_*` / `cascade_build_*` pair; `ast_based_generator/cascade/value.rs` emits the
direct-value build and discard forms; `ast_based_generator/scan.rs` emits the derived `scan_*`
graph. A parse chooses between them at runtime — `self.bare_parse = !self.coverage_enabled &&
!self.logger_enabled && !self.counters_observed.get() && !report_memo_stats_enabled()` — so an
ordinary consumer parse runs the FUSED graph while **every diagnostic consumer, including
`certificate_coverage`'s witness hook, forces the PROTOCOL one**.

The practical consequence is a measurement hazard, and it is the single most important thing to know
before trusting a before→after on an emission change: *a certificate delta does not prove a codegen
change reached the parse a consumer runs* (`GRAMMAR-WELLFORMED.H.16.4a` — a layout fix
applied to one of six sites moved the certificate to its correct post-fix value at all three seeds
while the shipped parse was untouched; every counter-based instrument routes the same way, so only
`parse_harness_equivalence_gate` could see it). The `cascade_match_*` / `cascade_build_*` pair is
additionally coupled — they share the derivation tape — so a decision they disagree on panics at
parse time rather than mis-parsing quietly.

⇒ **the structural discipline is one predicate with N callers plus a test pinning N**, not a
remembered list of sites. `AstBasedGenerator::regex_atom_skips_leading_layout` is the worked example:
it is the sole definition of whether an emitted `match_regex` skips leading layout, folding both the
`string_content_*` allowlist and the per-terminal layout-ownership property
(`regex_pattern_owns_its_layout` — every match is whitespace and the pattern cannot match empty),
and `layout_owning_terminal_tests::the_layout_skip_decision_has_exactly_one_definition` fails if any
emission module re-spells it. `parse_harness_interpreter.rs` mirrors that same predicate, and codegen
REFUSES to emit a parser whose raw and `effective_regex_pattern` verdicts differ — the one way
steering could make the mirror unsound.

### 3. Stimuli, Coverage, Debt, And Closure Planning
Primary file:
- `rust/src/ast_pipeline/stimuli_generator.rs`

Role:
- Generates stimuli from grammar AST.
- Tracks coverage metrics across rules and branches.
- Computes reachable vs unreachable debt.
- Builds target plans for closure work.
- Supports recovery-biased and negative-ish generation modes.
- Integrates semantic steering into generation decisions.

Assessment:
- This is not a side tool; it is a second core engine beside the parser generator.
- It explains why PGEN should be thought of as a parser-proof platform, not only a parser generator.
- The module is very capable, but at nearly eight thousand lines it is a major maintainability hotspot.

### 4. Typed Annotation Model, Validation, And Runtime Semantics
Primary files:
- `rust/src/ast_pipeline/unified_return_ast.rs`
- `rust/src/ast_pipeline/unified_semantic_ast.rs`
- `rust/src/ast_pipeline/annotation_validator.rs`
- `rust/src/ast_pipeline/semantic_directive_registry.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`
- `rust/src/ast_pipeline/semantic_transform.rs`

Role:
- Parses and normalizes typed return/semantic annotation payloads.
- Validates annotation contracts against grammar structure.
- Defines semantic directive parsing and capability rules.
- Compiles semantic runtime annotations and executes them transactionally during parse.

Assessment:
- Return-annotation support appears deeper and more mature than the typed semantic AST layer.
- Semantic support is still substantial, but more directive-oriented and more spread across registry/runtime/validator seams.
- The validator is far beyond a “lint” layer; it is grammar-aware and contract-bearing.
- The semantic runtime is a meaningful subsystem in its own right.

### 5. Grammar-Specific Subsystems
Primary files:
- `rust/src/ebnf_frontend.rs`
- `rust/src/ebnf_envelope_differential.rs`
- `rust/src/sv_preprocessor.rs`

Role:
- `ebnf_frontend.rs` provides a Rust-native `.ebnf -> raw_ast` frontend path.
- `ebnf_envelope_differential.rs` (new, `LANG-CAPABILITY-AUDIT.10.6` part 2) measures how far
  that hand-written frontend is from being REPLACEABLE by the parser generated from
  `grammars/ebnf.ebnf` — PGEN's stated endgame. It projects the generated meta-parser's typed
  AST into the frontend's `raw_ast` token vocabulary and diffs them token by token, which is a
  strictly stronger question than the parse verdict every prior instrument compared. Feature-gated
  on `ebnf_dual_run`, driven by `ebnf_dual_run_diff --envelope-differential`, consumed by
  `ebnf_frontend_dual_run_gate` under a two-sided per-grammar ratchet.
  - **Architectural significance**: this is the first OUTPUT-level comparison between PGEN's two
    EBNF frontends, and it made the self-hosting claim concrete — 6 of 14 grammars are
    envelope-equivalent, `grammars/ebnf.ebnf` itself at 913/913 tokens. It also found two live
    meta-grammar defects that every existing instrument passed, because both parse `Ok`
    (`.10.13`, `.10.14`).
  - **Trust model**: the module carries its own positive and negative ground-truth controls and
    refuses to publish a report if either misses — the projection is a new trusted surface, and
    an unvalidated projection would report its own bugs as findings (two did, during
    construction). Its declared normalizations reuse `ebnf_frontend`'s own
    `decode_quoted_literal_body` rather than re-implementing the escape rules.
- `sv_preprocessor.rs` implements a policyful SystemVerilog preprocessing stage with:
  - macro handling
  - include resolution
  - conditional compilation
  - diagnostics
  - source maps
  - event logging

Assessment:
- The Rust EBNF frontend is a real parser/tokenizer subsystem, not a small adapter.
- The SV preprocessor is substantial enough to deserve treatment as its own engine.
- The SV preprocessor’s explicit policies and observability surfaces are a strength.

### 6. Public Consumer Surfaces
Primary files:
- `rust/src/parser_registry.rs`
- `rust/src/embedding_api.rs`
- `rust/src/lib.rs`

Role:
- `parser_registry.rs` centralizes grammar-name dispatch across generated/bootstrap/profile-aware parsers. As of `PARSE-HARNESS.2` it also carries the additive, `#[cfg(has_generated_scratch_parser)]`-gated **`scratch`** slot — the parse-harness approach-3 entry that makes an arbitrary probe grammar (`grammars/scratch/scratch.ebnf`, built by `make focus_scratch`) drivable by the whole `parseability_probe` toolbox; its dispatch uses the generated parser's entry-rule-agnostic `parse_full()` so the registration is stable across arbitrary probe grammars.
- `embedding_api.rs` exposes a stable, versioned consumer contract with limits, result shapes, and AST-dump modes.
- `lib.rs` controls feature-gated exposure of the major subsystems.

Assessment:
- The embedding API is one of the cleaner and more disciplined pieces of the codebase.
- The registry layer is intentionally small and useful.
- There is still some repeated grammar/backend/profile branching across registry/API/binaries that could be unified further.

### 7. CLI, Build, And Operational Proof Layer
Primary files:
- `rust/src/main.rs`
- `rust/build.rs`
- `rust/Makefile`
- `rust/src/bin/*.rs`
- `rust/scripts/*.sh`

Role:
- `main.rs` is the large orchestration CLI for the core pipeline modes.
- `build.rs` resolves generated parser include paths at build time and emits `cfg` flags for available grammars. ⭐ Since `ENGINE-UNIVERSAL-SERVICES.24` it also **fingerprints** each resolved parser (sha256 → `PGEN_<FAMILY>_PARSER_SHA256`), so a compiled binary can state which parser it embeds — see the steering note at the top of this file.
- `rust/Makefile` coordinates bootstrap vs normal parser-generation flows.
- `rust/scripts/*.sh` provide the proof/gate ecosystem used for closure and release-grade validation.

Assessment:
- The build/gate layer is a major part of the product, not an afterthought.
- `build.rs` is strategically important because it lets the crate tolerate optional/generated parser availability.
- `main.rs` is functionally rich but overly large.
- The shell-gate surface is now big enough that architecture comprehension requires understanding both Rust and shell proof plumbing together.

## End-To-End Artifact Spine
1. Grammar/source input
   - Typical starting artifacts:
     - `grammars/*.ebnf`
     - generated grammar JSON inputs
     - real parser input samples
     - raw SystemVerilog source for preprocessing mode
2. Frontend / ingestion layer
   - `rust/src/ebnf_frontend.rs` can produce raw-AST envelopes directly from `.ebnf`
   - older or compatibility flows may still enter from precomputed JSON instead of live `.ebnf`
   - SystemVerilog preprocessing can branch here and emit expanded source plus source-map/diagnostic metadata before parsing
3. Normalization / transformation layer
   - `RustASTPipeline` in `rust/src/ast_pipeline/mod.rs` turns raw AST into the normalized grammar tree used for downstream generation
   - Important intermediate artifacts:
     - transformed / generation-input AST JSON
     - annotation metadata
     - normalization statistics
4. Generation layer
   - `ast_based_generator.rs` turns normalized grammar AST into generated Rust parser source
   - `stimuli_generator.rs` turns normalized grammar AST into:
     - in-memory stimuli
     - stimuli modules
     - coverage JSON
     - parseability reports
     - gap reports
     - target-drive telemetry
5. Runtime / consumer layer
   - Generated parser source becomes runtime parser modules through `build.rs` + `lib.rs`
   - Those runtime surfaces are then consumed by:
     - `parser_registry.rs`
     - `embedding_api.rs`
     - `parseability_probe`
     - `test_runner`
     - `perf_bench`
     - grammar-specific operational binaries
   - Parser-backed AST dumps can reappear here as a second artifact family, distinct from generation-input AST dumps
6. Proof / release layer
   - `rust/scripts/*.sh` collect upstream artifacts and emit machine-readable sidecars such as:
     - `summary.txt`
     - `summary.json`
     - `summary.csv`
   - Higher-level status / contract / combined-telemetry / SOTA gates then aggregate those sidecars into the project’s executable proof surface

Operational reading rule:
- Many bugs show up one stage later than where they originate.
- If a proof gate or parser runtime looks wrong, first identify which artifact family is wrong:
  - raw/frontend AST
  - normalized generation-input AST
  - generated parser source
  - runtime parser output
  - stimuli/coverage telemetry
  - proof sidecar summaries

## Artifact Persistence Classes
- Hand-authored source-of-truth artifacts
  - Examples:
    - `rust/src/**/*.rs`
    - `grammars/*.ebnf`
    - repo docs such as `README.md`, `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`, and this analysis doc
  - How to treat them:
    - edit directly when changing architecture, grammar intent, or authored implementation
- Tracked generated artifacts
  - Examples:
    - checked-in generated parser sources under `generated/`
    - especially tracked annotation/runtime-support artifacts like `generated/return_annotation_parser.rs` and `generated/semantic_annotation_parser.rs`
  - How to treat them:
    - do not hand-edit unless the project explicitly treats that file as maintained source
    - prefer regenerating from the real upstream grammar/pipeline and then committing the regenerated result when that tracked file is part of the intended contract
- Build-discovered parser artifacts
  - Examples:
    - parser files reached through `PGEN_*_PARSER_PATH`
    - files whose availability is surfaced via `build.rs` as `has_generated_*` cfgs
  - How to treat them:
    - confirm whether the current task expects a checked-in parser source, a locally generated file, or a gate/workdir-produced file
    - these are often runtime-available without being repository source-of-truth
- Ephemeral operational artifacts
  - Examples:
    - `rust/target/**`
    - temporary AST dumps
    - parseability reports
    - stimuli outputs
    - gate `summary.txt` / `summary.json` sidecars
  - How to treat them:
    - use them for proof, debugging, and validation
    - do not treat them as authoring surfaces unless a task is explicitly about emitted proof artifacts
- Consumer-visible but derived contract artifacts
  - Examples:
    - generated parser-backed AST JSON output
    - proof summaries consumed by higher gates
    - machine-readable reports consumed by status/aggregate layers
  - How to treat them:
    - these may be operationally important even when they are not checked in
    - fix the producer or the upstream source-of-truth first; do not “repair” the artifact by hand unless the task is specifically about the artifact schema/text itself

Operational rule:
- Before editing an artifact, classify it first:
  - authored source,
  - tracked generated file,
  - build-discovered runtime file,
  - or ephemeral proof/debug output
- A lot of accidental drift in this repo comes from patching a derived artifact when the real intended change belonged one layer upstream.

## Operational Vocabulary
- Raw AST
  - The frontend-oriented grammar structure coming directly from `.ebnf` ingestion or equivalent JSON import.
  - This is earlier and looser than the normalized generation-input AST.
- Generation-input AST
  - The normalized grammar tree actually consumed by parser generation and stimuli generation.
  - When this shape changes, parser output and stimuli output often both move.
- Generated parser source
  - The emitted Rust parser code produced from the generation-input AST before it becomes a compiled runtime module.
- Parser-backed AST dump
  - An AST or parse-tree-shaped output produced by a runtime parser surface such as `parseability_probe` or embedding APIs.
  - This is a runtime artifact, not the same thing as the generation-input AST.
- Parseability report
  - A machine-readable report about whether generated samples or corpus inputs are accepted by the relevant parser surface.
  - In this repo, parseability is often the bridge between generation logic and proof gates.
- Coverage / gap report
  - The stimuli-side telemetry that says what rules/branches were covered, which reachable targets remain, and what target-drive work is still open.
- `summary.txt`
  - The human-readable proof sidecar for a gate or contract layer.
  - Usually a flat `key: value` compatibility surface for quick inspection and shell-level checks.
- `summary.json`
  - The machine-readable proof sidecar for the same gate or contract layer.
  - This is the preferred surface for higher-level gates when the proof chain needs structured consumption.
- Family status / family status contract
  - Mid-layer proof artifacts that summarize a parser family’s current closure state and then freeze that state into a parity-checked contract surface.
- Combined telemetry
  - A higher aggregate layer that collects family-contract, family-status, and contract-sidecar evidence into one family-level proof view.
- SOTA / `sota_exit_gate`
  - The top aggregate proof layer that rolls family-level evidence into the project’s highest-level executable status surface.

## Canonical Source-Of-Truth Map
- Cargo binary and feature surface
  - Canonical source: `rust/Cargo.toml`
  - Use this first when the question is “does this binary/feature even exist?”
- Generated parser path resolution and `has_generated_*` cfg emission
  - Canonical source: `rust/build.rs`
  - Use this first when the question is “why is this generated parser path available or unavailable?”
- Feature-gated parser module exposure inside the crate
  - Canonical source: `rust/src/lib.rs`
  - Use this first when the question is “which generated modules are actually compiled into the library?”
- Grammar/profile parse dispatch and parser availability
  - Canonical source: `rust/src/parser_registry.rs`
  - Use this first when the question is “which parser surface does this grammar/profile name resolve to?”
- Embedder-facing parse, dump, and result contract
  - Canonical source: `rust/src/embedding_api.rs`
  - Use this first when the question is “what is the supported host-facing Rust/API behavior?”
- Main orchestration modes and generation/stimuli/preprocess CLI wiring
  - Canonical source: `rust/src/main.rs`
  - Use this first when the question is “what does the main Rust pipeline CLI do in this mode?”
- Stimuli, coverage, gap, and target-drive behavior
  - Canonical source: `rust/src/ast_pipeline/stimuli_generator.rs`
  - Use this first when the question is “why did generation/coverage/targeting behave this way?”
- EBNF frontend behavior
  - Canonical source: `rust/src/ebnf_frontend.rs`
  - Use this first when the question is “what raw AST did the Rust frontend mean to produce?”
- Proof-sidecar schema and aggregate proof flow
  - Canonical source: the emitting gate in `rust/scripts/*.sh`, with `summary.json` as the preferred structured contract surface
  - Use this first when the question is “which proof fields are actually promised here?”
- Aggregate annotation proof composition
  - Canonical source: `rust/Makefile`
  - Use this first when the question is “which top-level annotation/semantic/return proof targets are supposed to run together?”
- Annotation proof obligations and semantic intent
  - Canonical source: `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - Use this first when the question is “what annotation behavior and proof obligations are we actually claiming?”
- Operator-facing annotation gate map
  - Canonical source: `PGEN_USER_GUIDE.md`
  - Use this first when the question is “which aggregate annotation gates are meant to be run or read by humans?”
- Current architecture/risk/steering snapshot
  - Canonical source: `docs/reference/RUST_CODEBASE_ANALYSIS.md`
  - Use this first when the question is “what is the current repo-level understanding of the Rust codebase?”

Operational rule:
- When two layers disagree, fix the upstream source of truth first, then bring downstream consumers back into parity.
- In this repo, a lot of wasted time comes from patching adapters, sidecars, or aggregate readers before confirming which layer is actually authoritative.

## Public Contract Surface Map
- Embedder-facing Rust API contract
  - Main surface:
    - `rust/src/embedding_api.rs`
  - Compatibility promise:
    - host-visible parse, AST-dump, input-limit, and error/result behavior for Rust callers and bindings
- Grammar/profile dispatch contract
  - Main surface:
    - `rust/src/parser_registry.rs`
  - Compatibility promise:
    - which grammar/profile names resolve to which parser-backed behaviors and whether those paths are available
- Generated-parser availability contract
  - Main surface:
    - `rust/build.rs`
    - `rust/src/lib.rs`
  - Compatibility promise:
    - whether generated parser modules are actually compiled/exposed for a given feature/path configuration
- Machine-readable proof contract
  - Main surface:
    - emitting gates in `rust/scripts/*.sh`
    - especially `summary.json`
  - Compatibility promise:
    - structured proof fields consumed by family-status, combined-telemetry, SOTA, and local-CI parity readers
- Aggregate annotation proof contract
  - Main surface:
    - `rust/Makefile`
    - `annotation_contract_gate`
    - `semantic_full_contract_gate`
    - `return_annotation_support_gate`
    - `annotation_stimuli_quality_gate`
  - Compatibility promise:
    - which aggregate annotation proof targets compose the repo’s practical closure claims for validator coverage, runtime/round-trip evidence, return support, and annotation stimuli closure
- Human-readable proof compatibility surface
  - Main surface:
    - gate `summary.txt`
  - Compatibility promise:
    - stable shell-readable `key: value` summaries that remain in parity with the JSON sidecar where the shipped proof spine requires it
- Parseability/AST probe contract
  - Main surface:
    - `rust/src/bin/parseability_probe.rs`
  - Compatibility promise:
    - the narrow machine-facing parseability and AST-dump behavior used by external checks and higher proof layers

Operational rule:
- If a task changes one of these surfaces, treat it as a compatibility change even when the code edit feels local.
- Validate not only the producer, but also the nearest real consumer that relies on that contract.
- Do not assume internal parser-registry or probe availability automatically means the same family already has a public embedding contract.
  - In this repo, internal capability can be ahead of the stable embedder-facing surface.

## Symptom-To-Layer Triage Shortcuts
- Symptom: Cargo can build or list a binary, but the expected parser/runtime path is still missing
  - First likely seam:
    - `rust/build.rs`
    - `rust/src/lib.rs`
  - Usually verify:
    - feature enablement
    - `PGEN_*_PARSER_PATH` resolution
    - matching `has_generated_*` cfg emission
- Symptom: A grammar looks fine at frontend/raw-AST level, but generated parser behavior is still wrong
  - First likely seam:
    - normalized generation-input AST in `rust/src/ast_pipeline/mod.rs`
    - then `rust/src/ast_pipeline/ast_based_generator.rs`
  - Common mistake:
    - assuming the raw AST already reflects the shape the generator actually consumes
- Symptom: `parseability_probe` / embedding behavior disagrees with expectations from generation or registry changes
  - First likely seam:
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - then the generated parser availability layer
  - Common mistake:
    - debugging only the CLI surface without checking the dispatch/availability contract under it
- Symptom: Generated samples, coverage, or target-drive behavior looks wrong while parser acceptance seems fine
  - First likely seam:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - then the parseability-report and gap-report consumers in `rust/src/main.rs` and `rust/scripts/*.sh`
  - Common mistake:
    - treating stimuli/coverage problems as pure parser-codegen problems
- Symptom: Annotation-focused unit tests or leaf suites pass, but the repo-level annotation proof still feels wrong or incomplete
  - First likely seam:
    - the nearest aggregate annotation proof surface:
      - `annotation_contract_gate`
      - `semantic_full_contract_gate`
      - `return_annotation_support_gate`
      - `annotation_stimuli_quality_gate`
    - then the leaf validator/runtime/stimuli seam that feeds it
  - Common mistake:
    - stopping at the leaf suite that passed instead of checking which aggregate proof claim the repo is actually making
- Symptom: EBNF frontend output and generated-parser behavior drift apart
  - First likely seam:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
  - Common mistake:
    - debugging only downstream parser behavior without checking `parse` vs `parse_full` or frontend drift first
- Symptom: Gate summaries, family contracts, or aggregate proof outputs disagree even though lower layers “passed”
  - First likely seam:
    - the emitting gate in `rust/scripts/*.sh`
    - then the next aggregate reader above it
  - Common mistake:
    - patching only the aggregate reader when the emitting sidecar schema or TXT/JSON parity is actually wrong
- Symptom: A change compiles and unit tests pass, but the repo still feels inconsistent
  - First likely seam:
    - the next real artifact consumer, not the producer you already changed
  - Common mistake:
    - stopping validation at compile/test success instead of crossing the seam that the changed artifact feeds

## Safe Intervention Order
When a bug spans multiple layers, prefer these patch orders over ad hoc consumer-first edits.

- Build / availability problem
  - Preferred order:
    - `rust/Cargo.toml`
    - `rust/build.rs`
    - `rust/src/lib.rs`
    - `rust/src/parser_registry.rs` / `rust/src/embedding_api.rs`
    - CLI binaries / gates
  - Reason:
    - if feature wiring or cfg emission is wrong, every downstream layer will only produce misleading symptoms
- Parser-shape or acceptance problem
  - Preferred order:
    - frontend/raw-AST layer when applicable
    - normalized generation-input AST
    - parser generator
    - runtime parser consumers
    - proof/gate readers
  - Reason:
    - generator/runtime fixes applied before the normalized grammar contract is correct usually create brittle compensations
- Stimuli / coverage / target-drive problem
  - Preferred order:
    - normalized grammar contract
    - `stimuli_generator.rs`
    - CLI/report emission in `rust/src/main.rs`
    - nearest consuming gate or contract layer
  - Reason:
    - the proof artifacts are downstream of generation policy, not the source of it
- Annotation proof / closure problem
  - Preferred order:
    - typed annotation parse/validation/runtime layer
    - nearest leaf annotation contract or typed-AST suite
    - nearest aggregate annotation proof surface
      - `semantic_full_contract_gate`
      - `return_annotation_support_gate`
      - `annotation_stimuli_quality_gate`
      - `annotation_contract_gate`
    - doc / local-CI routing if the operator-facing proof map changed
  - Reason:
    - patching the top-level proof claim before the nearest annotation seam is correct tends to hide whether the real drift is semantic behavior, closure evidence, or just the proof map
- Registry / embedding disagreement
  - Preferred order:
    - generated-parser availability layer
    - `parser_registry.rs`
    - `embedding_api.rs`
    - CLI probes / external consumers
  - Reason:
    - embedder behavior should not be patched around registry/build drift
- Proof-sidecar disagreement
  - Preferred order:
    - emitting gate
    - direct reader one layer above
    - higher aggregates
    - local CI regression guard
  - Reason:
    - if the emitted proof surface is wrong, fixing only the aggregate consumer tends to multiply schema drift

Operational rule:
- Patch the earliest layer that can truthfully explain the symptom.
- Only patch a downstream layer first when the upstream contract is already verified correct and the bug is strictly in the consumer.

## Main Rust Executables And Roles
- `ast_pipeline` / `ast_pipeline_bootstrap`
  - Both are wired to `rust/src/main.rs` via Cargo features.
  - This is the main orchestration CLI for:
    - AST transformation
    - parser generation
    - stimuli generation
    - stimuli-module generation
    - generation-input AST dumps
    - SystemVerilog preprocessing
  - If a task sounds like “run the Rust pipeline on a grammar or source file,” this is usually the first executable to inspect.
- `test_runner`
  - The main round-trip and suite-running harness for bootstrap/generated parser validation.
  - Important when the task is test-suite behavior, normalization in tests, or parser-family regression coverage.
- `parseability_probe`
  - The compact machine-facing probe for “does this grammar/profile parse this input?” and “dump the AST for this parse.”
  - This is one of the cleanest executable surfaces for external parseability contracts and AST-dump behavior.
- `ebnf_dual_run_diff`
  - A specialist diagnostic tool for the generated EBNF parser path.
  - It compares `parse` vs `parse_full` behavior and emits structured diagnostics for unconsumed tails and frontend drift.
- `perf_bench`
  - Benchmark/threshold executable for bootstrap-vs-generated parser throughput and latency.
  - Relevant when performance changes need proof, not just anecdotal timing.
- `pgen_ast`
  - A focused AST-based codegen CLI that reads transformed AST JSON and emits parser source.
  - It is narrower than `ast_pipeline`, but still useful for direct generator work or compatibility testing around AST-based emission.
- `return_annotation_generated_audit`
  - A small audit executable for generated return-annotation typed-AST serialization over sample lists.
  - Useful as a niche contract checker, not as a primary day-to-day workflow surface.
- `pgen`
  - An older parser smoke-test CLI for semantic/return/regex parser inputs with log-file output.
  - It is not the main modern operational surface, but it still exists and should be treated as a legacy-adjacent utility rather than deleted-by-assumption.

Assessment:
- Not every Rust executable here is equally strategic.
- The practical “primary” binaries are:
  - `ast_pipeline` / `ast_pipeline_bootstrap`
  - `test_runner`
  - `parseability_probe`
  - `ebnf_dual_run_diff`
  - `perf_bench`
- The smaller `pgen_ast`, `return_annotation_generated_audit`, and `pgen` executables are better thought of as specialist or legacy-support utilities.

## Canonical-Vs-Legacy Surface Map
- Canonical day-to-day Rust operational surfaces
  - `ast_pipeline` / `ast_pipeline_bootstrap`
  - `test_runner`
  - `parseability_probe`
  - `ebnf_dual_run_diff`
  - `perf_bench`
  - Why:
    - these are the surfaces most likely to reflect the current intended build/runtime/proof contract
- Specialist but current surfaces
  - `pgen_ast`
  - `return_annotation_generated_audit`
  - selected grammar-specific operational flows that are narrow but still intentionally maintained
  - Why:
    - these are not the first place to look for most tasks, but they still represent real maintained seams
- Legacy-adjacent or carryover surfaces
  - `pgen`
  - older test-layer pieces like `rust/src/test_registry.rs` and `rust/src/test_discovery.rs`
  - Why:
    - they still exist and can matter for compatibility or historical behavior, but they should not be assumed to define the repo’s main modern workflow
- Canonical proof/verification surfaces
  - `rust/scripts/*.sh` gates on the shipped proof spine
  - family status / family status contract / combined telemetry / `sota_exit_gate`
  - `rust/scripts/ci_workflow_local_gate.sh`
  - Why:
    - these are the main executable proof contracts that preserve and validate the Rust-produced artifacts

Operational rule:
- If a newer canonical surface and an older carryover surface disagree, debug the canonical one first unless the user explicitly asks about compatibility or historical behavior.
- Use legacy-adjacent surfaces as corroborating evidence, not as the primary definition of current repo truth.

## Rust-To-Shell Contract Seams
- Parser availability / registry seam
  - Main Rust producers:
    - `rust/build.rs`
    - `rust/src/lib.rs`
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
  - Main shell-side consumers:
    - grammar-quality and family-contract gates under `rust/scripts/*.sh`
  - Typical failure mode:
    - a Rust cfg/registry change looks like a gate/proof failure because the shell layer only sees “adapter unavailable”, missing parseability support, or missing downstream report fields
- Parseability / stimuli / gap-report seam
  - Main Rust producers:
    - `rust/src/main.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - `rust/src/bin/parseability_probe.rs`
  - Main shell-side consumers:
    - family-contract, family-status, and aggregate proof gates that ingest parseability reports, coverage reports, gap reports, and target-drive summaries
  - Typical failure mode:
    - a Rust-side artifact schema or counting rule changes and the nearest shell consumer starts failing parity checks or status derivation
  - Regex-specific note:
    - the regex family now also depends on deterministic parseability-counterexample triage artifacts derived from the parser-backed stimuli report, so report-shape/counting changes can now break family-contract, SOTA, and combined-telemetry parity even when headline totals still look stable
    - it now also depends on an explicit regex formal-exhaustive-closure sidecar layered on top of those family artifacts, so broader-corpus proof promotion work now has a named gate/schema contract rather than a placeholder blocker string
- Summary sidecar seam
  - Main producers:
    - emitting gates under `rust/scripts/*.sh`
    - Rust tools whose outputs are re-packaged into `summary.txt` / `summary.json`
  - Main consumers:
    - family-status
    - family-status-contract
    - combined telemetry
    - `sota_exit_gate`
    - `ci_workflow_local_gate`
  - Typical failure mode:
    - the artifact still exists, but the structured contract changed and higher readers disagree on path, key, or parity expectations
- Frontend / dual-run seam
  - Main Rust producers:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/main.rs`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
  - Main shell-side consumers:
    - `rust/scripts/ebnf_*`
    - grammar-quality gates that depend on frontend parity or generated-parser freshness
  - Typical failure mode:
    - an ingestion/frontend change first appears as an `ebnf` gate drift or a generated-parser proof mismatch rather than a direct Rust test failure
- Aggregate annotation proof seam
  - Main Rust / Makefile producers:
    - `rust/Makefile`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - `rust/src/ast_pipeline/unified_return_ast.rs`
    - `rust/src/ast_pipeline/unified_semantic_ast.rs`
  - Main shell-side / doc-side consumers:
    - `rust/scripts/ci_workflow_local_gate.sh`
    - `README.md`
    - `PGEN_USER_GUIDE.md`
    - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - Aggregate proof spine:
    - `annotation_contract_gate`
      - validator + built-in bootstrap suites
      - shared bootstrap/generated annotation suites
      - SC leaf contract gates
      - `semantic_full_contract_gate`
      - `return_full_contract_gate`
      - `annotation_robustness_gate`
      - `annotation_stimuli_quality_gate`
    - `semantic_full_contract_gate`
      - `semantic_runtime_contract_gate`
      - `semantic_ast_roundtrip_gate`
      - `semantic_differential_regression_gate`
    - `return_annotation_support_gate`
      - sits above `return_full_contract_gate`
      - also requires the dedicated exhaustiveness proof
  - Typical failure mode:
    - the leaf tests still pass, but the operator-facing proof map or aggregate gate composition drifts and CI/humans stop running the intended top-level evidence path

Operational rule:
- When a Rust change affects artifact names, paths, schemas, or count semantics, inspect the nearest shell consumer above that artifact before assuming the bug is isolated to Rust.
- The meaningful validation seam is usually:
  - the emitted Rust artifact,
  - plus the next gate that consumes it.

## Where To Start By Task Type

### If the task is figuring out which Rust executable owns a workflow
Start here:
- `rust/Cargo.toml`
- `rust/src/main.rs`
- `rust/src/bin/test_runner.rs`
- `rust/src/bin/parseability_probe.rs`
- `rust/src/bin/ebnf_dual_run_diff.rs`
- `rust/src/bin/perf_bench.rs`
- `docs/reference/RUST_CODEBASE_ANALYSIS.md` section `Main Rust Executables And Roles`

Reason:
- Cargo wiring matters in this repo because feature-gated binaries share entrypoints.
- The fastest way to stop wandering is to identify whether a task belongs to the main pipeline CLI, a validation harness, a parseability contract tool, a frontend diagnostic, or a specialist audit utility.

### If the task is grammar normalization or parser-shape behavior
Start here:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
- `rust/src/ast_pipeline/mutual_recursion_handler.rs`
- `rust/src/ast_pipeline/return_annotation_handler.rs`

Reason:
- This is where raw grammar structure becomes the normalized grammar tree that the rest of the system depends on.
- Changes here can affect parser generation, stimuli generation, annotation validation, and closure metrics all at once.

### If the task is generated parser behavior or parser code shape
Start here:
- `rust/src/ast_pipeline/ast_based_generator.rs`
- `rust/src/ast_pipeline/ast_code_generator.rs`
- `rust/src/ast_pipeline/ast_generator_direct.rs`

Reason:
- This is the emitted-parser contract layer.
- Parser runtime telemetry, semantic-runtime ownership, recovery behavior, and branch ordering all converge here.

### If the task is stimuli, gap reports, or coverage closure
Start here:
- `rust/src/ast_pipeline/stimuli_generator.rs`

Then usually inspect:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`

Reason:
- Stimuli generation is highly coupled to normalized grammar shape and semantic steering.
- Coverage/debt behavior is not a thin report layer; it is part of how closure work is directed.

### If the task is return/semantic annotation parsing or validation
Start here:
- `rust/src/ast_pipeline/unified_return_ast.rs`
- `rust/src/ast_pipeline/unified_semantic_ast.rs`
- `rust/src/ast_pipeline/annotation_validator.rs`
- `rust/src/ast_pipeline/semantic_directive_registry.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`

Then usually inspect:
- `rust/Makefile`
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `PGEN_USER_GUIDE.md`

And pick the nearest aggregate proof surface:
- `annotation_contract_gate`
- `semantic_full_contract_gate`
- `return_annotation_support_gate`
- `annotation_stimuli_quality_gate`

Reason:
- The typed annotation model, validator rules, directive registry, and runtime behavior are split across these files.
- It is easy to fix one layer and accidentally leave the others inconsistent.
- In this repo, annotation work is not fully real until it still fits the aggregate proof spine that operators and CI actually run.

### If the task is external integration or embedder-facing API behavior
Start here:
- `rust/src/embedding_api.rs`
- `rust/src/parser_registry.rs`
- `rust/src/lib.rs`
- `rust/build.rs`

Reason:
- Consumer-visible behavior depends on both runtime dispatch and build-time generated-parser availability.
- Feature/cfg/build-path interactions matter here as much as function signatures do.

### If the task is SystemVerilog preprocessing
Start here:
- `rust/src/sv_preprocessor.rs`

Then usually inspect:
- `rust/src/main.rs`
- relevant `rust/scripts/sv_*` gates

Reason:
- The SV preprocessor is its own subsystem with policies, diagnostics, event logs, and source maps.
- Its behavior is not just a helper phase before parsing.

### If the task is EBNF frontend conversion
Start here:
- `rust/src/ebnf_frontend.rs`
- `rust/src/main.rs`
- `rust/Makefile`
- relevant `rust/scripts/ebnf_*` gates

Reason:
- The Rust EBNF frontend sits at the start of the build/proof spine, not only as a parsing helper.
- Changes here can affect raw-AST conversion, dual-run differentials, and the generated-parser pipeline.

### If the task is CLI mode behavior or top-level orchestration
Start here:
- `rust/src/main.rs`
- `rust/src/bin/*.rs`
- `rust/Makefile`

Reason:
- The codebase has one large orchestration entrypoint plus several smaller utility binaries.
- The main risk is changing mode behavior without aligning the supporting build/gate surface.

### If the task is proof plumbing, contract sidecars, or release-gate behavior
Start here:
- `rust/scripts/*.sh`
- `rust/Makefile`
- `rust/src/bin/parseability_probe.rs`
- `rust/src/parser_registry.rs`
- `rust/src/embedding_api.rs`

For SystemVerilog external-corpus proof normalization, narrow quickly to:
- `rust/scripts/sv_external_corpus_triage_gate.sh`
- `sv_formal_exhaustive_closure_gate` when the task is SystemVerilog external-corpus proof normalization
- `rust/scripts/sv_formal_exhaustive_closure_gate.sh` when the task is SystemVerilog external-corpus proof normalization
- `rust/scripts/sv_parser_family_status_gate.sh`
- `rust/scripts/sv_parser_family_status_contract_gate.sh`
- `rust/scripts/sota_exit_gate.sh` when the task is aggregate proof-surface propagation
- `rust/scripts/sv_combined_telemetry_contract_gate.sh` when the task is aggregate parity over retained SystemVerilog sidecars

For SystemVerilog-preprocessor formal-closure proof normalization, narrow quickly to:
- `sv_preprocessor_formal_exhaustive_closure_gate` when the task is SystemVerilog-preprocessor formal-closure proof normalization
- `rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh` when the task is SystemVerilog-preprocessor formal-closure proof normalization
- `rust/scripts/sv_preprocessor_syntax_closure_gate.sh`
- `rust/scripts/sv_preprocessor_aggregate_contract_gate.sh`
- `rust/scripts/sv_preprocessor_reachability_closure_gate.sh`
- `rust/scripts/sv_parser_family_status_gate.sh` when the task is retained family-status propagation for the preprocessor formal-closure seam
- `rust/scripts/sv_parser_family_status_contract_gate.sh` when the task is source-side contract validation for that propagated seam
- `rust/scripts/sota_exit_gate.sh` when the task is aggregate proof-surface propagation for the preprocessor formal-closure seam
- `rust/scripts/sv_combined_telemetry_contract_gate.sh` when the task is aggregate parity over the retained preprocessor formal-closure sidecar paths

For annotation-specific proof plumbing, narrow quickly to:
- `annotation_contract_gate`
- `semantic_full_contract_gate`
- `return_annotation_support_gate`
- `annotation_stimuli_quality_gate`

Reason:
- A large amount of project truth now lives in the shell-gate layer and the artifacts it consumes/emits.
- These tasks often require understanding both machine-readable sidecars and the Rust producer/consumer seams behind them.

## High-Risk Change Zones
- `rust/src/ast_pipeline/mod.rs`
  - high blast radius because it changes the normalized grammar contract used by both parser and stimuli generation.
- `rust/src/ast_pipeline/ast_based_generator.rs`
  - high blast radius because emitted parser behavior, runtime telemetry, and semantic hooks all converge here.
- `rust/src/ast_pipeline/stimuli_generator.rs`
  - high blast radius because closure metrics, target planning, and semantic steering are co-located here.
- `rust/src/main.rs`
  - high coordination cost because many modes share one orchestration entrypoint.
- `rust/build.rs`
  - easy to underestimate; build-time parser-availability bugs can look like runtime/parser bugs elsewhere.
- `rust/src/embedding_api.rs` and `rust/src/parser_registry.rs`
  - small files relative to the engines, but they sit on public integration seams where drift is expensive.
- `rust/scripts/sota_exit_gate.sh` and sibling family aggregate/status gates
  - not Rust code, but they are part of the effective Rust-owned product contract.

## Review Hotspots And Common Regression Types
- `rust/src/ast_pipeline/mod.rs`
  - First review for:
    - normalized rule-shape drift
    - left-recursion or grouping rewrites that silently change generator/stimuli inputs
    - raw-AST vs generation-input-AST confusion
- `rust/src/ast_pipeline/ast_based_generator.rs`
  - First review for:
    - emitted parser behavior drift
    - semantic-runtime hook drift
    - telemetry/counterexample/recovery output regressions
- `rust/src/ast_pipeline/stimuli_generator.rs`
  - First review for:
    - reachable/unreachable accounting drift
    - target-drive or closure-priority drift
    - parseability/coverage report shape changes
- `rust/src/main.rs`
  - First review for:
    - mode-selection drift
    - output-path / artifact-emission drift
    - wiring mismatches between CLI behavior and downstream consumer expectations
- `rust/build.rs`
  - First review for:
    - generated-parser path discovery regressions
    - missing or incorrect `has_generated_*` cfg emission
    - “binary exists, runtime path unavailable” failure shapes
- `rust/src/parser_registry.rs` and `rust/src/embedding_api.rs`
  - First review for:
    - parser availability/dispatch disagreements
    - embedder-facing compatibility drift
    - fixes that patch consumers instead of the upstream availability/source-of-truth layer
- `rust/scripts/*.sh` on the shipped proof spine
  - First review for:
    - `summary.txt` / `summary.json` parity drift
    - field/path/schema renames that higher readers were not updated for
    - aggregate layers preserving consumed provenance instead of dropping it

Operational rule:
- In this repo, “high risk” often means “easy to preserve compile success while breaking a contract.”
- Review the contract outputs and next consumers, not only the local code diff.

## Modules That Tend To Change Together
- Grammar normalization cluster
  - Typical files:
    - `rust/src/ast_pipeline/mod.rs`
    - `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
    - `rust/src/ast_pipeline/mutual_recursion_handler.rs`
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
  - Why they move together:
    - normalized rule shape leaks into parser generation, typed-annotation handling, and closure/stimuli behavior faster than the file boundaries suggest
- Generated-parser availability cluster
  - Typical files:
    - `rust/Cargo.toml`
    - `rust/build.rs`
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
  - Why they move together:
    - build-time parser discovery and runtime parser exposure are separate layers, so “parser unavailable” bugs often span both
- Semantic annotation cluster
  - Typical files:
    - `rust/src/ast_pipeline/unified_return_ast.rs`
    - `rust/src/ast_pipeline/unified_semantic_ast.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - `rust/src/ast_pipeline/semantic_directive_registry.rs`
    - `rust/src/ast_pipeline/semantic_runtime.rs`
    - `rust/src/ast_pipeline/ast_based_generator.rs`
  - Why they move together:
    - annotation parse shape, validation policy, emitted parser hooks, and runtime execution semantics are distributed rather than owned by one module
- EBNF/bootstrap ingestion cluster
  - Typical files:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/main.rs`
    - `rust/Makefile`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
    - relevant `rust/scripts/ebnf_*` gates
  - Why they move together:
    - changes at the EBNF/raw-ingestion edge often propagate into bootstrap generation, dual-run proofs, and build orchestration
- Proof/consumer cluster
  - Typical files:
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
    - relevant `rust/scripts/*.sh` gates
  - Why they move together:
    - externally visible truth in this repo is shared between Rust artifact producers and shell-side proof consumers

Operational rule:
- If one file in a cluster changes, scan the rest of that cluster before deciding your validation scope.
- If a task touches more than one cluster, validate at the first downstream artifact where those clusters converge rather than validating each layer in isolation.

## Change-Impact Checklist
Use this as a first-pass companion-check map, not as a complete proof checklist.

- If you change grammar normalization or core AST pipeline shape
  - Typical primary files:
    - `rust/src/ast_pipeline/mod.rs`
    - `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
    - `rust/src/ast_pipeline/mutual_recursion_handler.rs`
  - Usually re-check:
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - generation-input AST dump behavior in `rust/src/main.rs`
    - round-trip / parseability surfaces that implicitly depend on normalized rule shape
- If you change parser code generation
  - Typical primary files:
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/ast_code_generator.rs`
    - `rust/src/ast_pipeline/ast_generator_direct.rs`
  - Usually re-check:
    - generated parser compileability and include-path assumptions
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
    - `rust/src/bin/test_runner.rs`
    - `rust/src/bin/perf_bench.rs`
- If you change stimuli, coverage, or gap logic
  - Typical primary file:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
  - Usually re-check:
    - `rust/src/main.rs` stimuli CLI/report wiring
    - parseability validation report behavior
    - any derived counterexample-triage artifacts that shell gates summarize from parseability reports
    - coverage / gap / target-drive JSON artifacts
    - grammar-quality and family-contract gate expectations in `rust/scripts/*.sh`
- If you change annotation parsing, validation, or semantic runtime behavior
  - Typical primary files:
    - `rust/src/ast_pipeline/unified_return_ast.rs`
    - `rust/src/ast_pipeline/unified_semantic_ast.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - `rust/src/ast_pipeline/semantic_runtime.rs`
    - `rust/src/ast_pipeline/semantic_directive_registry.rs`
  - Usually re-check:
    - generated parser conversion paths
    - `test_runner` bootstrap vs generated parity
    - annotation-focused suites and typed-AST consumers
    - any docs or gates that currently treat return-annotation support as closed and semantic support as still more fluid
    - the nearest aggregate annotation proof surface:
      - `annotation_contract_gate` for shared annotation contract drift
      - `semantic_full_contract_gate` for semantic runtime/round-trip/regression drift
      - `return_annotation_support_gate` for return-annotation closure drift
      - `annotation_stimuli_quality_gate` when stimuli/coverage closure is part of the change
- If you change build-script or generated-parser availability behavior
  - Typical primary files:
    - `rust/build.rs`
    - `rust/src/lib.rs`
    - `rust/src/parser_registry.rs`
  - Usually re-check:
    - Cargo feature combinations
    - `PGEN_*_PARSER_PATH` resolution behavior
    - `has_generated_*` cfg guards
    - binaries gated by `generated_parsers` or `ebnf_dual_run`
    - embedder-facing availability behavior in `embedding_api.rs`
- If you change embedder-facing or registry-facing parse surfaces
  - Typical primary files:
    - `rust/src/embedding_api.rs`
    - `rust/src/parser_registry.rs`
  - Usually re-check:
    - `rust/src/bin/parseability_probe.rs`
    - AST dump contract behavior
    - feature/cfg fallback behavior
    - any gates or tests that rely on registry exposure or parser support checks
- If you change EBNF frontend behavior
  - Typical primary files:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/main.rs`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
  - Usually re-check:
    - raw-AST export behavior
    - dual-run drift reports
    - `ebnf_dual_run` build assumptions
    - readiness/quality gates that now rely on the Rust frontend path
- If you change SystemVerilog preprocessing behavior
  - Typical primary files:
    - `rust/src/sv_preprocessor.rs`
    - SV preprocess wiring in `rust/src/main.rs`
  - Usually re-check:
    - source-map and diagnostics behavior
    - strict-warning policy handling
    - downstream parseability expectations on preprocessed output
    - SV quality/aggregate proof gates in `rust/scripts/`
- If you change proof-sidecar shape or release-gate aggregation
  - Typical primary files:
    - `rust/scripts/*.sh`
    - sometimes `rust/src/bin/parseability_probe.rs` or `rust/src/embedding_api.rs`
  - Usually re-check:
    - `summary.txt` / `summary.json` parity
    - `ci_workflow_local_gate.sh`
    - higher aggregate readers like family-status, combined telemetry, and SOTA exit
    - `docs/reference/RUST_CODEBASE_ANALYSIS.md` if the effective operational contract changed

## Build And Feature Model
- The crate is feature-gated around bootstrap, normal, generated-parser, and EBNF-dual-run modes.
- Generated parser modules are not hardwired; `rust/build.rs` resolves them from environment-configured paths and only enables grammar-specific `cfg`s when files actually exist.
- This is a strength because it supports:
  - bootstrap cycles
  - clean checkout behavior
  - relocatable worktrees
  - partial grammar availability
- It also means more conditional complexity and more chances for path/feature divergence.

Feature/build matrix:
- `normal`
  - unlocks the `ast_pipeline` binary from `rust/src/main.rs`
  - represents the standard non-bootstrap orchestration path
- `bootstrap`
  - unlocks the `ast_pipeline_bootstrap` binary from the same `rust/src/main.rs` entrypoint
  - exists so the pipeline can keep functioning when generated-parser availability is intentionally reduced or absent
- `generated_parsers`
  - unlocks binaries and code paths that depend on the generated parser registry and generated parser includes
  - directly gates:
    - `parseability_probe`
    - `perf_bench`
    - generated-parser branches in the embedding/test surfaces
- `ebnf_dual_run`
  - unlocks the generated-EBNF differential tooling
  - directly gates:
    - `ebnf_dual_run_diff`
    - Rust-frontend/generated-frontend comparison flows in the CLI/build ecosystem

Build-time availability model:
- `rust/build.rs` does two distinct jobs:
  - resolves include paths from environment variables like `PGEN_SYSTEMVERILOG_PARSER_PATH` and `PGEN_VHDL_PARSER_PATH`
  - emits grammar-specific `cfg`s like `has_generated_systemverilog_parser` only when the resolved file actually exists
- That means feature enablement alone is not enough for every generated-parser behavior.
- In practice there are two layers of availability:
  - Cargo feature enabled
  - matching generated parser file found by `build.rs`

Generated parser env/cfg map:
- `PGEN_EBNF_PARSER_PATH`
  - resolved by `build.rs` into:
    - `PGEN_EBNF_PARSER_PATH_RESOLVED`
    - `PGEN_EBNF_PARSER_PATH_RESOLVED_BIN`
  - used by the `ebnf_dual_run` surface
  - important nuance: there is no `has_generated_ebnf_parser` cfg; EBNF availability is handled differently from the other grammar families
- `PGEN_JSON_PARSER_PATH`
  - drives `has_generated_json_parser`
  - controls `generated_parsers::json` and related parser-registry exposure
- `PGEN_REGEX_PARSER_PATH`
  - drives `has_generated_regex_parser`
  - controls `generated_parsers::regex`, related parser-registry exposure, and regex generated-backend availability inside `embedding_api.rs`
  - the repo now also carries the default tracked artifact at [generated/regex_parser.rs](generated/regex_parser.rs), so a normal checkout no longer needs an ad hoc env override just to make regex’s generated backend exist
- `PGEN_SYSTEMVERILOG_PARSER_PATH`
  - drives `has_generated_systemverilog_parser`
  - controls generated SystemVerilog registry, embedding, and parseability paths
- `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`
  - drives `has_generated_systemverilog_preprocessor_parser`
  - controls generated SV-preprocessor parser availability
- `PGEN_VHDL_PARSER_PATH`
  - drives `has_generated_vhdl_parser`
  - controls generated VHDL registry, embedding, and parseability paths
- `PGEN_RTL_CONST_EXPR_PARSER_PATH`
  - drives `has_generated_rtl_const_expr_parser`
  - controls generated RTL-const-expr parser availability

Important asymmetries:
- `return_annotation` and `semantic_annotation`
  - live under `generated_parsers`, but are included from tracked generated sources rather than `build.rs` env-driven grammar-path discovery
- `ebnf`
  - uses `build.rs`-resolved include paths, but not the same `has_generated_*` cfg pattern used by the other generated grammar families
- `systemverilog`, `vhdl`, and the other env-driven grammar families
  - usually require both:
    - `feature = "generated_parsers"`
    - matching `has_generated_*` cfg emitted by `build.rs`

Operational takeaway:
- If a binary or API path appears to “exist but not really work,” check both:
  - the Cargo feature set
  - the relevant `PGEN_*_PARSER_PATH` resolution and resulting `has_generated_*` cfg
- A surprising amount of apparent parser/runtime breakage in this repo can actually be feature/build-shape drift.

## Bootstrap-Vs-Generated Boundary Map
- `ast_pipeline`
  - The normal orchestration binary from `rust/src/main.rs`
  - Usually the right entrypoint when generated-parser-backed behavior is intended to be available
- `ast_pipeline_bootstrap`
  - The bootstrap-mode sibling from the same `rust/src/main.rs`
  - Important when the pipeline must remain usable even while generated-parser availability is intentionally constrained
- `test_runner`
  - A hybrid validation surface
  - It can exercise bootstrap behavior, generated-parser behavior, and parity between them depending on feature set and suite path
- `parseability_probe`
  - A generated-parser-oriented runtime probe
  - If this surface is unavailable or behaving oddly, suspect generated-parser availability and registry exposure before assuming generic parser bugs
- `return_annotation` / `semantic_annotation`
  - Important boundary exception
  - They sit under `generated_parsers`, but come from tracked generated sources rather than the same env-resolved grammar-family path model used by the larger HDL/regex families
- `ebnf`
  - Another important boundary exception
  - Dual-run EBNF uses build-script-resolved include paths, but not the same `has_generated_*` cfg pattern as the other env-driven grammar families
- `parser_registry.rs` / `embedding_api.rs`
  - These are the main mixed-boundary consumers
  - They are where bootstrap-vs-generated availability differences become host-visible behavior

Operational rule:
- When debugging a bootstrap-vs-generated mismatch, do not start from the consumer alone.
- First confirm:
  - which side of the boundary you are actually on
  - whether the relevant generated parser is truly available
  - whether the mismatch is in parser behavior itself or only in boundary wiring/exposure

## Grammar-Family Asymmetry Map
- `systemverilog`
  - A major env-driven generated-parser family
  - It also carries unusually heavy surrounding proof surface through aggregate, status, semantic-scope, roundtrip, and failure-context consumers
  - It now also has a dedicated formal-exhaustive-closure proof sidecar whose provenance is preserved end to end through family-status, family-status-contract, aggregate SOTA telemetry, and combined telemetry
  - The current machine-checked family-status row remains `Mostly Done` with `7` total closure criteria, `4` satisfied, `3` unsatisfied
  - The current retained blockers are now explicit and should drive the next SystemVerilog slice directly:
    - `syntax_closure_gate_status=fail failure_count=2`
    - `shadow_parser_rejections_total=3 > 0`
    - `focused_replay_target_count=2550 > 0`
- `systemverilog_preprocessor`
  - Also env-driven, but not just a smaller copy of parser-only families
  - Its runtime semantics include macro expansion, include handling, conditional policy, source mapping, diagnostics, and strict-promotion-adjacent behavior
  - The latest retained closure reduction came from one parser-side fix plus two shared parser+stimuli-safe grammar tightenings:
    - [ast_based_generator.rs](rust/src/ast_pipeline/ast_based_generator.rs) now disables auto layout skipping for regex tokens in the `systemverilog_preprocessor` generated parser family, using the generator's normalized PascalCase grammar identifier instead of only the raw underscore file stem
    - [systemverilog_preprocessor.ebnf](grammars/systemverilog_preprocessor.ebnf) now requires `condition_expr` in `pp_elsif_branch`, which closes illegal bare `` `elsif`` generation without narrowing valid preprocessor syntax
    - that same grammar now routes the line-oriented non-conditional directives `undef`, `include`, `timescale`, `default_nettype`, `celldefine`, and `endcelldefine` through `directive_comment_tail := inline_trivia line_comment?` instead of the broader `directive_tail`
  - Root causes:
    - preprocessor directive rules already encode same-line trivia explicitly via `inline_trivia`
    - the generated parser was still auto-skipping layout before regex tokens, so line-oriented regex rules like `directive_tail` could hop across a newline and swallow the following directive line
    - the grammar also still allowed `pp_elsif_branch` with no condition expression, which let the stimuli lane emit syntactically invalid bare `` `elsif`` lines
    - even after those two fixes, the broader `directive_tail` allowance on line-oriented non-conditional directives still left a narrower same-line directive-chaining seam that needed a more surgical shared-tail contract
  - Fresh focused proof on the retained shape now records `parseability_attempts_total=37`, `parseability_accepted_total=36`, `parseability_rejected_total=1`, `parseability_parser_rejections_total=1`, `parseability_counterexamples_captured_total=1`, `stage0_target_count=3`, `stage1_target_count=2`, and `final_targets=0` in the preprocessor aggregate contract lane
  - The higher-level proof readers are now aligned on that same retained aggregate: formal closure, family status, lightweight SOTA telemetry, and combined telemetry all now mirror the `1`-reject / `final_targets=0` baseline instead of the older stale `2`-reject snapshot
  - The direct minimal parser repros now pass on the rebuilt generated adapter too:
    - `/*a*/\`ifdef A`
    - `/*a*//*b*/\`ifdef A`
    - `/*a*/ /*b*/\`ifdef A`
  - The focused sample corpus no longer emits bare `` `elsif`` lines
  - Important reuse nuance: the stable main-SV aggregate reuse surface is `rust/target/sv_parser_aggregate_contract_gate_json_proof`, and the lightweight SOTA reuse surfaces for failure-context and roundtrip are the `*_json_proof` directories, not the plain gate directories
  - Two tempting shared parser+stimuli changes were explicitly rejected during this slice:
    - excluding backticks from `directive_tail` made the measured rejection surface worse, so future work should not retry that narrowing blindly
    - the broader all-directives `directive_line_tail := inline_trivia line_comment?` refactor also made the measured rejection surface worse, so future work should keep the narrower `directive_comment_tail` scope instead of broadening it again by default
- `vhdl`
  - An env-driven generated-parser family with a comparatively cleaner parser-family seam than SV
  - In practice it is strongly coupled to quality/parseability, strict-promotion, and now a dedicated formal-exhaustive-closure proof surface
  - The current machine-checked family-status row is now `Done`, with `10/10` closure criteria satisfied and no remaining tracked blocker debt. The refreshed direct quality, family-contract, formal-closure, family-status, family-status-contract, and combined-telemetry sidecars now all agree on the retained closed baseline.
  - Fresh direct VHDL quality proof on the retained Rust-side slice now records:
    - `closed_loop_initial_targets=247`
    - `closed_loop_replay_targets=0`
    - `closed_loop_parseability_shadow_parser_rejections_total=0`
    - `quality_parseability_generation_parser_rejections_total=0`
  - The refreshed machine-checked family blocker surface is now empty:
    - `quality_closed_loop_replay_targets=0`
    - `strict_promotion_primary_blocker=none`
    - `strict_promotion_trial_passed=3`
  - The parser-backed generation side of the current canonical family-quality surface is now green (`attempts=8`, `accepted=8`, `rejected=0`), realistic-corpus parity is now `13` expected pass / `1` expected fail with matching observed totals, and strict promotion is still green (`trial_passed=3`, `primary_blocker=none`)
  - The new VHDL formal-closure gate is now green off the checked-in external-corpus-backed proof surface, and the family-status / family-status-contract / SOTA / combined-telemetry stack preserves that provenance end to end
  - Recent VHDL work also reinforced the general EBNF steering rule: grammar changes must be classified as parser-only, stimuli-only, or shared parser+stimuli changes before they are kept. The retained VHDL slice (`--enforce-word-boundary-spacing`, `trivia` priority bias, newline-terminated line comments, and explicit `end process` / `end procedure` / `end function` body endings) improved the shared family-quality surface; a temporary `wait until` grammar broadening was intentionally reverted because it improved one parser-facing case while worsening replay-shadow parser debt
  - Two additional directions are now explicitly rejected rather than merely “not yet landed”:
    - a shared `stimuli_generator.rs` direct-probe rebias worsened replay debt from `11` to `30`
    - broader VHDL branch-steering experiments either worsened replay debt (`11 -> 17`) or made replay materially more expensive without yielding a keepable result
  - The current preferred tactic for the remaining VHDL replay targets is now the new branch-level triage tool plus targeted generator-side interventions instead of more blanket sample-hint sweeps:
    - [coverage_gap_triage.rs](rust/src/bin/coverage_gap_triage.rs) joins the gap report, coverage report, and grammar AST into one readable triage surface
    - the verified current VHDL run shows:
      - `trivia#line_comment` is a real selection-bias seam
      - `actual_parameter_element#range_expression` is part of a shared `range_expression` dependency failure
      - `actual_part#expression` is part of a shared `expression` dependency failure
    - the latest retained generator-side win now reflects that diagnosis directly:
      - dependency-blocked target branches are no longer failure-throttled before their still-targeted referenced rules record any success history
      - still-targeted OR branches that fail only on local depth exhaustion now get one temporary depth-slack retry during plain target driving
      - that retry is explicitly disabled during validation-aware target driving so replay-shadow parseability stays on the stricter canonical surface
      - `priority_first` branch selection now gives unresolved unseen target branches a one-shot probe bias without altering ordinary priority ordering
    - operational consequence:
      - the tracked VHDL contract is now closed
      - future VHDL work should preserve this closed family/status/aggregate baseline unless the contract is intentionally widened
  - The proof-plumbing caveat from the first `9`-target refresh attempt is now resolved in the retained gate path:
    - [vhdl_stimuli_quality_gate.sh](rust/scripts/vhdl_stimuli_quality_gate.sh) now isolates a state-local `CARGO_TARGET_DIR`
    - nested strict-promotion refreshes no longer clobber the adapter-backed generated `ast_pipeline` / `parseability_probe` binaries
    - that isolated build cache is now treated as disposable by default: the gate retains `work/` and `logs/`, but prunes the default state-local `cargo_target` on exit unless `PGEN_VHDL_STIMULI_KEEP_CARGO_TARGET=1` is set
    - future VHDL work should spend its effort on reducing the remaining replay targets, not on this normalized refresh seam
- `regex`
  - An env-driven generated-parser family, but operationally closer to the EBNF frontend world than the HDL families
  - Dual-run/frontend/stimuli closure surfaces matter a lot here, so parser-family work often crosses into ingestion and diagnostic tooling
  - It now also has a dedicated regex-only family stimuli contract (`rust/test_data/grammar_quality/regex_family_stimuli_contract.json`), so the canonical regex family/status/aggregate sidecars refresh from a family-local parser-backed baseline instead of piggybacking on the broader shared non-annotation stimuli contract
  - The current published regex family proof stack is now on the refreshed parser-family contract baseline (`5911` attempts, `5197` accepted, `714` diagnostic target-drive parser rejections, `804 -> 0` targets after `6526` target-drive attempts), and the machine-checked regex family status computes `Done` with `8/8` closure criteria satisfied
  - It now also has deterministic parser-rejection triage sidecars in its family-contract and aggregate proof stack, so regex closure work can talk about dominant failing sample/error/location buckets instead of only total rejection counts
  - It now also has a checked-in broader-corpus proof gate over the regex stress corpus (`44` executed, `44` pass, `0` fail in the current measured slice), and the formal-exhaustive-closure gate is now green because that broader-corpus proof surface exists
  - It now also has a maintained external-corpus acquisition starter under `regex_corpus_bundle/`; that bundle is deliberately separate from the closed regex family-status math and should be treated as the canonical future widening lane for PCRE2-first external hardening rather than as a replacement for the current checked-in `stress_tests.json` proof slice
  - The downstream regex host contract is now materially stronger than the first release slice: `embedding_api.rs` now publishes a regex parser release version, a regex integration-contract version, explicit generated-backend requirement metadata, a regex AST-dump schema version, and machine-localizable parse-failure locations through `ParseDiagnostic.location`
  - The current RGX handoff is parser release `1.1.29` / integration contract `1.1.31`; legal deep PCRE2 repros such as `80` nested captures plus `\80` and recursive named-group interpolation patterns still parse through the generated host path without stack abort, and the latest source-derived audit adds grammar coverage for PCRE2 short Unicode property escapes, quoted class literals, quoted class range endpoints, escaped quoted-literal body characters, bounded variable-length lookbehind, Unicode capture names, orphan class `\E`, dedicated `\C` single-code-unit escape transport, callout-prefixed conditional assertions, PCRE2 POSIX word-boundary aliases, UTF width start-option aliases, and callout/lookaround/script-run/scan-substring/modifier/escape forms while tightening compile-contract false accepts around unbounded lookbehind, malformed names, empty substantive class bodies, plain class `\N`, nonliteral class range endpoints, decoded escaped class-range endpoint ordering including bare octal `\NNN`, false unboundedness from declarative `DEFINE` conditionals inside lookbehind, and scan-substring references that truly do not exist in the whole pattern. The AST schema remains `1`
  - PCRE2-conformance fixes should follow the source-of-truth workflow captured in `docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md`: prose docs first, `pcre2_compile.c` for edge cases, and PCRE2 `testdata/testinput*` for executable regression truth
  - Recent real-world regex follow-ups showed why this family is so frontend-coupled: fixing quoted-terminal escape decoding in `ebnf_frontend.rs`, widening `literal_char` just enough for `:` and `/`, deliberately allowing an empty top-level regex, and then disabling implicit layout skipping in generated regex parsers were enough to turn the checked-in `url_pattern`, `empty_regex`, and leading-whitespace quantifier false negatives green without changing the higher-level proof architecture
  - The final regex blocker turned out to be in the stimuli engine rather than the parser itself: alternate-entry helper probes inside `generate_until_targets_with_filter` now retain helper-rule coverage progress even when those helper outputs fail the primary-entry parseability filter, so regex target driving no longer spins by rolling back legitimate helper coverage
  - It now also has a public embedding seam in `embedding_api.rs`, but that public surface should not be mistaken for complete parser-family closure by itself
  - It now also has a dedicated downstream integration contract doc plus a regex-specific host contract gate layered on top of the generic embedding API gate, so downstream consumers no longer have to reconstruct the regex host promise from the generic embedding contract alone; RGX-style consumers can now pin both parser release version and contract version explicitly when integrating or reporting bugs
- `ebnf`
  - Not just another generated runtime parser family
  - It sits at the ingestion/frontend edge, uses build-script-resolved include paths, and has its own dual-run diagnostic shape rather than the standard `has_generated_*` family contract
- `return_annotation` / `semantic_annotation`
  - Important tracked-generated exceptions
  - They live under `generated_parsers`, but come from checked-in generated sources and typed-annotation workflows rather than the env-driven grammar-family path model
- `json` / `rtl_const_expr`
  - Supporting generated-parser families
  - They matter for build/registry completeness, but they are not the main day-to-day closure-driving families in the same way as SV, VHDL, and regex

Operational rule:
- Do not assume one family’s build, validation, or proof-plumbing shape generalizes cleanly to another.
- Before copying a fix pattern across families, check whether the source family is:
  - env-driven parser runtime
  - frontend/ingestion
  - tracked generated annotation support
  - or a supporting parser family with lighter operational ownership

## Known Traps And False Assumptions
- “If Cargo lists the binary, the runtime path must be available.”
  - False here.
  - A binary can exist in Cargo while the relevant generated parser path is still unavailable because the matching `has_generated_*` cfg was never emitted.
- “Raw AST and generation-input AST are basically the same artifact.”
  - False here.
  - The normalization layer is substantial; many downstream changes only make sense once you know which side of that boundary is wrong.
- “Parser-backed AST dumps and generation-input AST dumps are the same debugging surface.”
  - False here.
  - One is a runtime parser output, the other is a pre-generation pipeline artifact.
- “A successful compile is enough validation for a Rust change.”
  - Usually false here.
  - In this repo, the next real consumer of the artifact is often the meaningful validation seam.
- “If `parseability_rejected_total=0`, the parseability counterexample sidecar must be empty.”
  - False here.
  - Counterexample triage can still retain target-drive filter evidence from rejected alternate-entry helper probes even when the canonical primary-entry parseability totals are fully green.
- “If an annotation leaf suite or one SC gate passes, the repo-level annotation proof claim is done.”
  - False here.
  - Aggregate annotation claims usually live one layer higher in `annotation_contract_gate`, `semantic_full_contract_gate`, `return_annotation_support_gate`, or `annotation_stimuli_quality_gate`.
- “Shell gates are just wrappers around the real Rust product.”
  - False here.
  - The shell proof layer is part of the effective product contract because it defines and preserves the executable proof surfaces.
- “`summary.txt` is just for humans and `summary.json` is optional.”
  - False for the shipped proof spine.
  - Higher layers increasingly depend on the JSON sidecar, while TXT/JSON parity remains part of the contract.
- “Every generated grammar family follows the same include-path and cfg rules.”
  - False here.
  - `ebnf` and the tracked annotation parsers are important exceptions.
- “The modern surface is always in `rust/src/main.rs`.”
  - Not always.
  - A lot of live operational behavior sits in `rust/src/bin/*.rs`, `rust/scripts/*.sh`, and feature/cfg wiring rather than the main orchestration CLI alone.

## Testing And Verification Shape
- The test surface is not only `cargo test`.
- The codebase relies on:
  - unit/integration tests in Rust modules
  - JSON-driven round-trip suites
  - parser-family quality and contract gates
  - SOTA/aggregate proof surfaces
- `rust/src/test_runner/round_trip_tests.rs` is the more modern JSON-driven testing spine.
- `rust/src/test_registry.rs` and `rust/src/test_discovery.rs` look older and more limited by comparison.

Assessment:
- The repo is very strong on proof surfaces.
- The downside is that the test ecosystem is mixed-generation and not fully consolidated behind one obvious canonical layer.

## Validation Ladder By Change Type
Treat this as a representative ladder, not a claim that every task needs every step.

- Docs-only or continuity-only change
  - usually enough:
    - `git diff --check`
- Build-shape or feature/cfg change
  - start with:
    - `cargo check --manifest-path rust/Cargo.toml --bins`
  - then usually add the relevant feature shape, for example:
    - `cargo check --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline`
    - `cargo check --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe`
    - `cargo check --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bins`
- Parser-registry or embedder-facing change
  - usually re-check:
    - `cargo test --manifest-path rust/Cargo.toml --features generated_parsers --lib parser_registry`
    - `cargo test --manifest-path rust/Cargo.toml --features generated_parsers --lib embedding_api`
- Parser behavior or typed-AST conversion change
  - usually re-check:
    - focused `test_runner` suites
    - focused `parseability_probe --parse ...`
    - any generated-vs-bootstrap parity path that the touched parser family relies on
  - if the parser is `return_annotation` or `semantic_annotation`, usually also add:
    - `make -C rust annotation_contract_gate`
    - and the nearest annotation aggregate proof surface above the touched seam
- EBNF frontend or dual-run change
  - usually re-check:
    - `cargo test --manifest-path rust/Cargo.toml --features ebnf_dual_run --lib ebnf_frontend::tests`
    - `cargo check --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bin ebnf_dual_run_diff`
    - focused `ebnf_dual_run_diff` execution if the issue is parse/full-parse drift
- Stimuli, coverage, or gap-report change
  - usually re-check:
    - focused `ast_pipeline --generate-stimuli ...`
    - parseability report generation when the change affects parser-backed validation
    - the nearest grammar-quality or family-contract gate that consumes those artifacts
  - for annotation-focused stimuli work, usually also add:
    - `make -C rust annotation_stimuli_quality_gate`
    - and, if annotation semantics changed too, `make -C rust annotation_contract_gate`
- Proof-sidecar or gate change
  - usually re-check:
    - the touched gate directly
    - `bash rust/scripts/ci_workflow_local_gate.sh`
  - and when the change affects aggregate proof flow:
    - the nearest family-status / family-status-contract / combined-telemetry / SOTA reader above it
  - for annotation-proof changes, the practical aggregate readers are usually:
    - `annotation_contract_gate`
    - `semantic_full_contract_gate`
    - `return_annotation_support_gate`
    - `annotation_stimuli_quality_gate`

Operational rule:
- Prefer the smallest validation slice that still crosses the seam you changed.
- In this repo, “build passes” is often weaker than “the next consumer of the artifact still agrees.”

## Strengths
- Strong architecture around determinism, observability, and machine-checkable proof.
- Clear Rust-first integration posture with explicit bootstrap escape hatches rather than hidden hand-written exceptions.
- Stable consumer-facing API design in `embedding_api.rs`.
- Sophisticated stimuli/coverage/gap machinery that matches the project’s closure doctrine.
- Good generated-parser integration model in `build.rs`.
- Real policyfulness in the SV preprocessor instead of a shallow text-prepass design.

## Main Risks And Technical Debt
- Complexity concentration in:
  - `stimuli_generator.rs`
  - `ast_based_generator.rs`
  - `annotation_validator.rs`
  - `mod.rs`
  - `main.rs`
- Repeated grammar/backend/profile adapter logic across:
  - `parser_registry.rs`
  - `embedding_api.rs`
  - selected binaries / CLI surfaces
- Bootstrap/generated duality remains necessary but expensive to reason about.
- Semantic support is powerful but distributed across several coupled files, which raises the cost of safe changes.
- The shell-gate layer is large enough that “the Rust codebase” now effectively includes a substantial Bash proof system.

## Steering Implications
- Future implementation should keep treating parser generation, stimuli closure, and proof/gate output as one system.
- Effort spent only on parser acceptance without preserving observability and proof surfaces will fight the project’s actual architecture.
- Refactors should aim to reduce concentration without weakening the current proof doctrine.

## Architectural Invariants Worth Preserving
- Explicit bootstrap-vs-generated boundaries
  - Keep parser availability visible through features, env-driven paths, and `build.rs`/cfg wiring rather than hidden fallback behavior.
- Stage-distinct artifacts
  - Preserve the distinction between:
    - raw/frontend AST
    - normalized generation-input AST
    - generated parser source
    - runtime parser output
    - proof sidecars
  - A lot of bugs come from collapsing these into one mental bucket.
- Machine-readable proof contracts
  - Preserve `summary.json` as a first-class contract surface alongside `summary.txt`, especially on the shipped proof spine.
- Upstream source-of-truth repair
  - Prefer fixing the emitting layer or canonical producer over patching an aggregate consumer to “make the drift disappear.”
- Seam-crossing validation
  - Keep validating at the next real consumer of the changed artifact; compile success alone is often not enough in this repo.
- Parser behavior plus observability
  - Avoid changes that improve acceptance or throughput by quietly weakening telemetry, counterexample capture, gap reporting, or proof surfaces.
- Shell proof layer as product surface
  - Treat `rust/scripts/*.sh` and their emitted sidecars as part of the effective Rust-owned contract, not as optional wrappers around “the real code.”

## Refactor Patterns That Fit This Codebase
- Split by artifact boundary, not by raw line count
  - Good split examples:
    - parser emission vs semantic-runtime emission vs telemetry/report emission
    - raw/frontend AST handling vs normalized-AST handling
    - report production vs report aggregation
  - Why:
    - this repo’s real seams are artifact and contract boundaries, not just file size
- Stabilize outputs before moving orchestration
  - Prefer extracting helpers around stable outputs first:
    - generated parser source
    - parseability/coverage reports
    - `summary.txt` / `summary.json`
  - Why:
    - if outputs stay stable, you can move internals without multiplying downstream drift
- Refactor one contract seam at a time
  - Prefer:
    - `build.rs` + one direct consumer
    - one report producer + one immediate gate consumer
    - one dispatch layer + one public API layer
  - Avoid broad simultaneous rewrites across:
    - feature/cfg shape
    - registry exposure
    - proof-gate consumers
  - Why:
    - this repo has enough coupled seams that multi-seam rewrites get hard to validate honestly
- Replace repeated branching with narrow shared adapters
  - Best targets:
    - repeated grammar/backend/profile dispatch
    - repeated report-path / proof-surface extraction
    - repeated parser-availability checks
  - Why:
    - duplicated branch logic is a bigger long-term risk here than small helper indirection
- Add proof before deleting carryover paths
  - If a refactor replaces a consumer or proof seam, lock the new behavior first with the nearest meaningful validation or local-CI assertion before removing the old path
  - Why:
    - subtractive cleanup is safer once the replacement seam is already machine-checked
- Keep the next consumer in the loop while refactoring
  - When extracting or moving code, validate at the first downstream consumer that depends on the moved artifact rather than stopping at compile success
  - Why:
    - many regressions here are contract regressions, not syntax or type-check failures

## Recommended Refactor Priorities
- Split `rust/src/main.rs` into subcommand or mode-focused modules.
- Break `rust/src/ast_pipeline/stimuli_generator.rs` into smaller policy/reporting/runtime units.
- Break `rust/src/ast_pipeline/ast_based_generator.rs` into emitter-focused submodules:
  - parser struct/runtime emission
  - semantic runtime emission
  - recovery/coverage telemetry emission
  - per-rule method emission
- Reduce repeated dispatch logic by introducing a more unified grammar/backend adapter layer shared by:
  - `parser_registry.rs`
  - `embedding_api.rs`
  - CLI/binary consumers
- Clarify which test layers are canonical and which are legacy carryovers.

## Open Architecture Questions
- How far can the giant engine files be split without weakening artifact/proof seams?
  - Main hotspots:
    - `rust/src/main.rs`
    - `rust/src/ast_pipeline/mod.rs`
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
- Can parser dispatch be unified further without hiding important feature/cfg/build-path realities?
  - Main seam:
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - CLI consumers
- Which older/carryover test layers still provide unique value, and which are just maintenance drag?
  - Main seam:
    - `test_runner` versus `rust/src/test_registry.rs` / `rust/src/test_discovery.rs`
- How much of the proof spine should remain in shell versus moving into Rust?
  - Main seam:
    - `rust/scripts/*.sh`
    - Rust-produced artifacts and machine-readable reports
- Which family asymmetries are permanent design facts versus candidates for normalization?
  - Main seam:
    - `ebnf`
    - tracked annotation parsers
    - env-driven generated families
    - companion crates like `rtl_const_expr` / `rtl_frontend`
- What is the long-term relationship between the main `rust/` crate and the companion RTL crates?
  - Main seam:
    - `rust/`
    - `rtl_const_expr/`
    - `rtl_frontend/`

Operational rule:
- Treat these as active design questions, not settled doctrine.
- If a future task materially resolves one of them, refresh this section instead of letting the answer stay implicit in scattered commits.

## What To Re-Check At The Start Of A New Session
- Whether the hotspot files and their responsibilities have materially shifted.
- Whether new grammar families or generated parser integrations changed the build/registry shape.
- Whether bootstrap vs generated boundaries moved.
- Whether the public consumer seam changed:
  - embedding API
  - parser registry
  - grammar-profile coverage
- Whether the proof/gate layer changed enough that this document’s description of the operational surface is stale.
- Whether the main current risks are still:
  - concentrated module size
  - repeated adapter seams
  - bootstrap/generated maintenance cost
  - mixed-generation testing layers

## Session-Start Sanity Probes
Use these as cheap orientation probes before deeper Rust work, not as a replacement for task-specific validation.

- `git status --short`
  - Confirms whether unrelated dirt, generated artifacts, or untracked directories are already present before you start attributing odd behavior to the code.
- `rg -n "^\\[\\[bin\\]\\]|^\\[features\\]" rust/Cargo.toml`
  - Re-checks whether the binary and feature surface still matches this document’s assumptions.
- `rg -n "PGEN_[A-Z_]+_PARSER_PATH|has_generated_" rust/build.rs rust/src/lib.rs`
  - Re-checks the generated-parser availability contract quickly without re-reading the full files.
- `rg --files rust/src/bin`
  - Re-confirms the active Rust utility-binary surface.
- `sed -n '1,120p' docs/reference/RUST_CODEBASE_ANALYSIS.md`
  - Fast check that the live analysis doc still presents the same top-level structure and hasn’t fallen behind a major architectural shift.
- If the task is proof/gate-heavy:
  - `rg -n "summary\\.json|summary\\.txt|sota_exit_gate|combined_telemetry|family_status" rust/scripts`
  - Quick way to confirm whether the proof-sidecar vocabulary or aggregate-gate surface has drifted materially since the last session.

## Limits Of This Snapshot
- This assessment came from a deep source read and structural review.
- It is not a benchmark report.
- It is not a full dynamic validation run of all Rust binaries and all gates.
- It should therefore be refreshed when runtime evidence materially changes the picture.

## Current Closure Steering Notes
- The active HDL closure lane is now main `systemverilog`, not `vhdl` or `systemverilog_preprocessor`.
  - `vhdl` is on a clean retained baseline.
  - `systemverilog_preprocessor` is on a clean retained baseline.
  - `regex` is closed for the currently published contract.
- Main SystemVerilog aggregate proof now has a better retained counterexample surface.
  - `rust/scripts/sv_parser_aggregate_contract_gate.sh` now summarizes:
    - unique/dominant `primary_entry_rule`
    - unique/dominant `generation_entry_rule`
    - unique/dominant `entry_mode`
    - for both generation and replay-shadow counterexample triage
  - important reuse rule:
    - prefer the top-level `rust/target/sv_stimuli_quality_gate` state as the canonical cheap reuse surface
    - older nested aggregate artifacts are not the default reuse source anymore
  - important compatibility rule:
    - current reusable `sv_stimuli_quality_gate` reports may be lean and omit embedded `counterexamples`
    - aggregate proof now normalizes those omissions to zero/empty, so `<none>` / `0` on the new entry-context fields is acceptable on lean artifacts
- Downstream SOTA/combined-telemetry proof refreshes are now aligned with that incremental aggregate surface.
  - `rust/scripts/sota_exit_gate.sh` and `rust/scripts/sv_combined_telemetry_contract_gate.sh` now default missing main-SV entry-context fields from older aggregate summaries to:
    - `0` for counts
    - `<none>` for dominant-value fields
  - practical effect:
    - cheap reuse-backed proof refreshes stay viable even when the aggregate summary predates the new entry-context keys
    - do not waste a session rebuilding the full main-SV aggregate proof solely to make downstream summaries stop showing `<none>` / `0`
- A fresh main-SV generator shortcut was tried and rejected.
  - rejected idea:
    - force a newline before any non-line-start `//...` segment in `StimuliGenerator::append_generated_segment(...)`
  - retained evidence:
    - adapter-backed direct probe regressed from `112/180 accepted, 68 parser rejects`
    - to `111/180 accepted, 69 parser rejects`
  - steering consequence:
    - do not reopen the broad line-comment normalization path
    - the remaining main-SV parser debt still needs a narrower seam than “newline all same-line comments”
- The current best retained main-SV stimuli seam is still narrow, but it is now `escaped_identifier` plus safe `line_comment` sampling plus a dedicated `timeunits_declaration` separator repair rather than any broad comment-normalization rewrite.
  - retained changes:
    - `grammars/systemverilog.ebnf`
    - `@sample: "\\foo "` above `escaped_identifier := trivia /\\[!-~]+/`
    - `@sample: "//x\n"` above `line_comment := /\/\/[^\n]*(\n|$)/`
    - `timeunit_separator_slash := timeunit_separator_trivia "/"`
    - the first `timeunits_declaration` form now uses `( timeunit_separator_slash time_literal )?`
  - important probe-path correction:
    - the older focused direct-probe recipe drifted because `PGEN_SYSTEMVERILOG_PARSER_PATH=rust/target/...` is wrong when `cargo build --manifest-path rust/Cargo.toml` is launched from repo root
    - `build.rs` resolves that env path relative to the Rust manifest directory, so the corrected local probe must use an absolute parser path or `target/...` relative to `rust/`
  - retained root cause:
    - after the safe `line_comment` sample landed, the remaining direct-probe rejects all clustered around `timeunits_declaration`
    - `slash := trivia "/"` could consume the opening slash of a same-line `//...` comment before the real precision separator slash on the next line
  - rejected follow-up:
    - a global `slash := trivia /\/(?![\/\*])/` experiment was intentionally not kept
    - the generated parser runtime compiles regex terminals through Rust `regex::Regex` at parse time, so that broader lookahead-based change regressed even plain `timeunit 336 us / 4 ms;`
  - retained evidence:
    - with the corrected adapter-backed build, the pre-fix focused `sv_2023` direct probe measured `110/181 accepted, 71 parser rejects`
    - after the safe `line_comment` sample landed, that same focused `sv_2023` probe measured `176/179 accepted, 3 parser rejects`
    - the matching `sv_2017` cross-check measured `177/179 accepted, 2 parser rejects`
    - after the narrow `timeunits_declaration` separator repair, the focused adapter-backed probes now measure:
      - `sv_2017`: `179/179 accepted, 0 parser rejects`, `304/2597` replay targets resolved in the retained 200-attempt loop
      - `sv_2023`: `179/179 accepted, 0 parser rejects`, `287/2393` replay targets resolved in the retained 200-attempt loop
  - practical steering consequence:
    - the contract-default and bounded (`PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=100`) `sv_stimuli_quality_gate` reruns are still too heavy for first local iteration on this seam because `profile_2017_closed_loop_replay` chases a retained `2597`-target initial gap and stays CPU-hot for minutes without becoming a good quick loop
    - that replay seam is no longer completely opaque though:
      - `rust/src/ast_pipeline/stimuli_generator.rs` now emits low-verbosity start/progress/completion telemetry for target-driven generation
      - `rust/scripts/sv_stimuli_quality_gate.sh` now forwards replay-stage tracing through `PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY`
      - the maintained shell-gate default is now `low`, which keeps the terminal quiet because stage output is still captured by `run_logged` into per-stage log files
      - direct `ast_pipeline` invocations still keep their own ordinary default trace posture
      - a one-attempt direct probe against the retained `profile_2017_initial_gap.json` now logs `341/2593` resolved targets on the first attempt, which is enough to distinguish “making progress slowly” from “not entering target-drive at all”
      - helper-probe activation and payoff are now visible immediately too, not only through coarse periodic checkpoints:
        - the same bounded replay lane now logs `Target-drive helper probe` whenever generation switches away from `systemverilog_file`
        - it also logs `Target-drive helper result` after each helper attempt with `pending_before`, `pending_after`, and `resolved_delta`
        - the corrected bounded 128-attempt read showed helper probing was already active before the final checkpoint, led by `property_expr`, then `expression_or_dist`, `kw_iff_ee1c009e`, `covergroup_expression`, `bin_identifier`, `kw_else_ae050f5b`, and finally `bins_keyword`
        - the measured payoff in that same run was sharply uneven:
          - `property_expr`: `resolved_delta=23`
          - `expression_or_dist`: `resolved_delta=31`
          - `covergroup_expression`: `resolved_delta=6`
          - `kw_iff_ee1c009e`, `bin_identifier`, `kw_else_ae050f5b`: `resolved_delta=1`
          - `bins_keyword`: `resolved_delta=3`
        - target-drive now also keeps same-run helper-payoff history and feeds it back into helper ranking:
          - per-helper attempts
          - total resolved delta
          - best single resolved delta
          - validation-aware worthiness now also preserves previously high-yield helpers under alternate-entry churn
        - low replay trace now also exposes the competing helper pools at each helper activation:
          - the selected pool (`dependency` vs `pending`)
          - the top dependency candidate
          - the top pending candidate
        - the retained bounded 128-attempt main-SV replay did not yet change its visible helper sequence or `917/2593` completion on top of that selector change, so this is currently a replay-selection infrastructure improvement with focused tests rather than a claimed frontier jump in the cheap probe lane
        - the new ranking trace explains that flat result:
          - selected dependency helpers churn across `property_expr`, `expression_or_dist`, `kw_iff_ee1c009e`, `covergroup_expression`, `bin_identifier`, `kw_else_ae050f5b`, and `bins_keyword`
          - the same pending-frontier leader, `property_expr_sv_2017`, stays visible throughout with `branch_targets=33..36`
          - so the next tuning question is whether a sufficiently large persistent pending frontier should sometimes outrank a marginal fresh dependency probe
        - that cross-pool tuning is now landed in a bounded form:
          - the top pending candidate can outrank the top dependency candidate when the dependency is only a fresh marginal probe and the pending frontier is at least twice as wide in both branch count and retained remaining-success volume
          - base and validation-aware selector paths now share that comparison rule
        - live replay evidence now shows the selector flipping exactly at the expected seam:
          - first helper remains `property_expr`
          - second helper flips from dependency churn to pending `property_expr_sv_2017`
        - that improvement introduced a new practical constraint:
          - once replay starts honoring the broad pending frontier, the cheap probe lane becomes materially slower
          - the next tuning question is therefore not just selection correctness but also whether broad pending probes need an explicit effort budget
        - that effort budget is now staged directly into the runtime selector:
          - broad pending-frontier selection only unlocks after `probe_threshold + 8`
          - helper-ranking trace now reports `pending_frontier_unlocked=true|false`
        - that staging is now also an explicit maintained control surface:
          - `StimuliConfig.target_pending_frontier_extra_stagnation` carries the extra unlock budget
          - default remains `8` for the maintained cheap replay lane
          - `ast_pipeline` exposes `--target-pending-frontier-extra-stagnation`
          - `sv_stimuli_quality_gate` accepts `PGEN_SV_STIMULI_QUALITY_PENDING_FRONTIER_EXTRA_STAGNATION`
          - helper-ranking trace now also reports:
            - `pending_frontier_unlock_threshold`
            - `pending_frontier_extra_stagnation`
          - stimuli corpus bundle generation metadata now preserves the configured replay budget
        - the first focused main-SV experiment now answers whether immediate unlock should be promoted:
          - default `8` still completes the retained 128-attempt focused `sv_2017` replay at `917/2593`
          - immediate unlock `0` does flip the second helper from dependency `expression_or_dist` to pending `property_expr_sv_2017`
          - but that same heavy run was still active after `03:22` elapsed and had only reached the 64-attempt checkpoint (`864/2593`) before manual stop
          - so the explicit heavy control is worth keeping for experiments, but not for the maintained default proof posture
        - the missing containment is now landed directly in the runtime:
          - `StimuliConfig.target_helper_generation_timeout_ms` defaults to `1000ms`
          - `ast_pipeline` exposes `--target-helper-generation-timeout-ms`
          - `sv_stimuli_quality_gate` accepts `PGEN_SV_STIMULI_QUALITY_TARGET_HELPER_TIMEOUT_MS`
          - helper-ranking trace now reports `helper_timeout_ms`
          - stimuli corpus bundle generation metadata now preserves the configured helper budget
        - that helper budget is now also surfaced as explicit replay telemetry instead of only low-trace evidence:
          - `TargetDriveSummary` carries `helper_timeout_errors`
          - `TargetDriveValidationSummary` carries `helper_timeout_errors`
          - target-drive progress/completion lines now print `helper_timeout_errors`
          - `TargetDriveParseabilityTelemetry` preserves the same counter for parseability report JSON and stimuli corpus bundles
        - the nearest shell/report consumers now preserve that counter too instead of dropping it during aggregation:
          - `annotation_stimuli_quality_gate.sh` emits `target_drive_helper_timeout_errors_total` in `summary.csv` and aggregate parseability report JSON
          - `sv_preprocessor_quality_gate.sh` emits the same `target_drive_helper_timeout_errors_total` field in `summary.csv` and aggregate parseability report JSON
          - `sv_stimuli_quality_gate.sh` preserves helper-timeout totals in closed-loop parseability-shadow aggregate JSON, per-profile JSONL, and `summary.txt`
          - `vhdl_stimuli_quality_gate.sh` surfaces helper-timeout totals in closed-loop parseability-shadow `summary.txt`
          - `sv_preprocessor_aggregate_contract_gate.sh` and `sv_parser_aggregate_contract_gate.sh` now sanity-check those copied totals
        - that same immediate-unlock focused repro now completes instead of wedging:
          - retained 128-attempt heavy replay finished at `970/2593`
          - the `7` generation errors are bounded `property_expr_sv_2017` helper timeouts
          - helper probes can still retire meaningful debt before timing out, so the heavy lane is now bounded rather than all-or-nothing
        - retained cheap replay evidence is back to the earlier bounded shape:
          - 96-attempt direct replay completed at `904/2593`
          - 128-attempt direct replay completed at `917/2593`
          - both traces stayed on dependency helpers with `pending_frontier_unlocked=false`
        - that means the current main-SV question is no longer “is helper probing happening at all?” but rather “does the heavier replay lane revisit enough helper competition for payoff-aware ranking to show up in aggregate proof runs?”
    - use the corrected adapter-backed direct probe for cheap local shaping, then reserve the full gate for proof refresh
  - proof consequence:
    - the next honest main-SV step is still a full `sv_stimuli_quality_gate` rerun to see whether these focused wins survive the retained proof lane
    - until that rerun lands, treat this as a focused direct-lane improvement, not a refreshed main-SV status row
- The bounded main-SV proof lane now has a keepable runtime shortcut too.
  - retained failure shape before the fix:
    - a bounded `sv_stimuli_quality_gate` rerun with `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128` failed on:
      - `stimuli_generate_ms_per_sample budget exceeded for profile=2017,sample=0 (17061 > 10000)`
    - the failing sample was tiny and already parseability-accepted, which made it a bad fit for "the grammar is still too complex" as an explanation
  - retained runtime root cause:
    - CPU sampling of the hot `ast_pipeline` process showed the time concentrated in:
      - `load_grammar_bundle(...)`
      - `RustASTPipeline::transform_from_raw_ast(...)`
      - `parse_rule_content(...)`
      - `extract_rule_annotations(...)`
      - `parse_semantic_annotation_with_generated_parser(...)`
    - the maintained gate shape had been reparsing and retransformation-heavy because it still fed `grammars/systemverilog.ebnf` into every later `ast_pipeline` invocation after parser generation
  - retained implementation:
    - `rust/src/main.rs` now writes `--dump-gen-ast` as a directly reloadable transformed-style generation bundle:
      - top-level `grammar_name`, `rule_order`, `grammar_tree`, `annotations`
      - plus `metadata.format=transformed_ast`
      - plus `metadata.pipeline_stage=generation_input_ast`
    - the same JSON loader now normalizes older generation-AST dumps that had `grammar_tree` / `rule_order` but no `metadata`
    - `rust/scripts/sv_stimuli_quality_gate.sh` now emits `${grammar_name}_gen_ast.json` during parser generation and then reuses that bundle for:
      - closed-loop initial generation
      - initial replay determinism
      - replay-shadow
      - per-sample generation
  - retained bounded evidence after the fix:
    - `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-gate-bounded-20260421-r3 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - outcome:
      - `closed_loop_profiles_passed=2/2`
      - `closed_loop_initial_replay_determinism_passes=2/2`
      - `parseability_generation_parser_rejections_total=0`
      - `parse_full_passes=16/16`
      - `perf_observed_generate_avg_ms=173`
      - `perf_observed_generate_max_ms=624`
  - steering consequence:
    - keep reusing the normalized generation-AST bundle for main-SV gate/runtime work; it removes a real source of misleading proof-lane cost
    - still do not promote the main-SV live status row from this alone; this is bounded proof-lane hardening, not a full contract-default refresh
    - paired with the gate-local default `low` replay logs, the maintained main-SV shell lane is now both cheaper and more inspectable than it was before the last two slices
- The next bounded main-SV containment seam is now explicit too.
  - retained problem shape after the last two slices:
    - the active hotspot was no longer grammar loading
    - a real contract-default rerun could still spend minutes inside a single canonical target-drive attempt
  - retained implementation:
    - `StimuliConfig` now carries:
      - `target_generation_timeout_ms`
      - `target_helper_generation_timeout_ms`
    - the primary budget still defaults to `0` in the runtime/CLI surface, so maintained API behavior stays unchanged unless explicitly requested
    - helper probes still use the existing separate helper-only budget
    - target-drive summaries/traces now report:
      - `target_timeout_errors`
      - `helper_timeout_errors`
    - `rust/src/main.rs` now exposes:
      - `--target-generation-timeout-ms`
    - `rust/scripts/sv_stimuli_quality_gate.sh` now accepts:
      - `PGEN_SV_STIMULI_QUALITY_TARGET_GENERATION_TIMEOUT_MS`
      - and records both primary/helper budgets in the gate header plus `summary.txt`
    - replay-shadow aggregate output now preserves:
      - `target_timeout_errors_total`
    - the higher shell/report stack now also preserves that same signal instead of dropping it:
      - `annotation_stimuli_quality_gate.sh`
      - `sv_preprocessor_quality_gate.sh`
      - `vhdl_stimuli_quality_gate.sh`
      - `sv_parse_full_ratio_promotion_gate.sh`
      - `sv_declared_shadow_promotion_gate.sh`
      - `vhdl_strict_promotion_gate.sh`
      - `sota_exit_gate.sh`
    - aggregate sanity checks now also validate copied target-timeout totals in:
      - `sv_preprocessor_aggregate_contract_gate.sh`
      - `sv_parser_aggregate_contract_gate.sh`
  - retained follow-on fixes and measured consequence:
    - `rust/src/ast_pipeline/stimuli_generator.rs` now treats built-in `epsilon` as an empty expansion instead of erroring on a missing user-defined rule
    - `grammars/systemverilog.ebnf` now gives `simple_identifier_no_scope` a safe ordinary hint:
      - `@sample: "foo"`
    - after those two seams were cleared, the next contract-default shell rerun still showed the practical workflow problem clearly:
      - the maintained main-SV shell lane could sit indefinitely inside one primary attempt
    - the keepable adjustment is deliberately shell-local:
      - `rust/scripts/sv_stimuli_quality_gate.sh` now defaults `closed_loop_target_generation_timeout_ms` to `5`
      - `PGEN_SV_STIMULI_QUALITY_TARGET_GENERATION_TIMEOUT_MS=0` restores legacy unbounded shell behavior explicitly
      - the runtime/CLI default remains `0`
  - retained direct SystemVerilog smoke:
    - dumped generation bundle created from `grammars/systemverilog.ebnf`
    - replay run:
      - `rust/target/debug/ast_pipeline /tmp/pgen-sv-target-timeout-smoke/systemverilog_gen_ast.json --generate-stimuli --grammar-profile 2017 --enforce-word-boundary-spacing --count 1 --seed 202 --output /tmp/pgen-sv-target-timeout-smoke/replay_no_parseability.sv --coverage-output /tmp/pgen-sv-target-timeout-smoke/replay_no_parseability_coverage.json --gap-report-json /tmp/pgen-sv-target-timeout-smoke/replay_no_parseability_gap.json --gap-report-text /tmp/pgen-sv-target-timeout-smoke/replay_no_parseability_gap.txt --target-max-attempts 2 --target-generation-timeout-ms 1 --target-report-input /tmp/pgen-sv-target-timeout-smoke/initial_gap.json`
  - observed summary:
      - `Target-driven generation: resolved 0/2560 targets in 2 attempts (generation_successes=0, generation_errors=2, target_timeout_errors=2, helper_timeout_errors=0)`
  - retained shell-layer propagation proof:
    - bounded VHDL strict promotion now produces:
      - `closed_loop_parseability_shadow.target_drive_validation.target_timeout_errors_total`
    - aggregate report consumers can therefore keep the primary-timeout distinction intact beyond the direct quality-gate layer
  - retained bounded maintained-shell proof:
    - `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-target-timeout-default PGEN_SV_STIMULI_QUALITY_COUNT=1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=32 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - outcome:
      - `closed_loop_profiles_passed=2/2`
      - `closed_loop_parseability_shadow_accepted_total=21`
      - `closed_loop_parseability_shadow_parser_rejections_total=0`
      - `closed_loop_parseability_shadow_target_timeout_errors_total=40`
      - `closed_loop_parseability_shadow_helper_timeout_errors_total=1`
      - `parse_full_passes=2/2`
  - important boundary:
    - current generated-parser parseability validation still does not support `systemverilog` on the `ast_pipeline` binary path
    - so this slice's retained end-to-end proof is target-drive summary evidence plus focused unit coverage, not a validator-backed parseability report refresh
  - steering consequence:
    - this remains proof-lane containment, not a closure claim
    - keep the runtime/API default conservative at `0`
    - keep the main-SV shell workflow bounded by default unless a session explicitly wants the old unbounded behavior
  - follow-up seam and repair:
    - after that shell containment landed, the next bounded `128`-attempt rerun finally exposed a narrower grammar seam instead of just "slow proof"
    - retained failure shape before the fix:
      - `profile_2017_closed_loop_replay.log` churned on repeated helper-entry timeouts for `property_case_item`
      - helper probes retired no target debt there (`resolved_delta=0`)
    - keepable grammar-side repair:
      - [grammars/systemverilog.ebnf](grammars/systemverilog.ebnf) now gives the two `property_case_item` alternatives helper-only footholds:
        - `@probe_sample: "1: 1;"`
        - `@probe_sample: "default: 1;"`
      - this is intentionally `@probe_sample`, not `@sample`, so the alternate-entry lane gets deterministic shapes without flattening ordinary generation
    - direct proof:
      - `cargo run --manifest-path rust/Cargo.toml --features ebnf_dual_run --bin ast_pipeline -- grammars/systemverilog.ebnf --generate-stimuli --grammar-profile 2017 --entry-rule property_case_item --count 1 --seed 712001`
      - observed output:
        - `1: 1;`
    - bounded maintained-shell refresh:
      - `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-property-case-item-r1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
      - outcome:
        - `closed_loop_profiles_passed=2/2`
        - `closed_loop_initial_targets_total=5273`
        - `closed_loop_replay_targets_total=4608`
        - `closed_loop_parseability_shadow_accepted_total=68`
        - `closed_loop_parseability_shadow_rejected_total=0`
        - `closed_loop_parseability_shadow_parser_rejections_total=0`
        - `closed_loop_parseability_shadow_target_timeout_errors_total=151`
        - `closed_loop_parseability_shadow_helper_timeout_errors_total=31`
        - `parseability_generation_parser_rejections_total=0`
        - `parse_full_passes=16/16`
        - `perf_observed_generate_avg_ms=213`
        - `perf_observed_generate_max_ms=646`
    - retained replay movement:
      - the old `property_case_item` helper wedge disappears from the bounded `profile_2017_closed_loop_replay.log`
      - the first visible helper activation now pivots to `expression`
      - that first helper retires `resolved_delta=91`
    - interpretation:
      - the meaningful result is not "timeouts vanished"
      - it is that the previous replay blocker is gone and the bounded proof lane now gets further into the frontier with a different next dependency
  - follow-up annotation-placement repair:
    - the next retained replay experiment tried to seed:
      - `module_ansi_header`
      - `module_nonansi_header`
      - `program_ansi_header`
      - `program_nonansi_header`
    - the first attempt used same-line inline `@sample`
    - the dumped generation-input bundle showed those hints landing in `branch_semantic_annotations`, not rule-level `semantic_annotations`
    - for these single-alternative rules, that meant the canonical samples were structurally branch-local and did not behave like ordinary rule-level steering
  - keepable correction:
    - [grammars/systemverilog.ebnf](grammars/systemverilog.ebnf) now spells those four seeds as standalone annotation lines above the rules:
      - `@sample: "module m(input logic a);"`
      - `@sample: "module m(a,b);"`
      - `@sample: "program p(input logic a);"`
      - `@sample: "program p(a,b);"`
  - direct proof:
    - `rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --generate-stimuli --grammar-profile 2017 --entry-rule module_ansi_header --count 1`
      - `module m(input logic a);`
    - `rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --generate-stimuli --grammar-profile 2017 --entry-rule module_nonansi_header --count 1`
      - `module m(a,b);`
    - `rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --generate-stimuli --grammar-profile 2017 --entry-rule program_ansi_header --count 1`
      - `program p(input logic a);`
    - `rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --generate-stimuli --grammar-profile 2017 --entry-rule program_nonansi_header --count 1`
      - `program p(a,b);`
  - bounded maintained-shell refresh:
    - `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-header-seed-r1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - outcome:
      - `closed_loop_profiles_passed=2/2`
      - `closed_loop_replay_targets_total=4217`
      - `closed_loop_parseability_shadow_accepted_total=73`
      - `closed_loop_parseability_shadow_rejected_total=0`
      - `closed_loop_parseability_shadow_parser_rejections_total=0`
      - `closed_loop_parseability_shadow_target_timeout_errors_total=157`
      - `closed_loop_parseability_shadow_helper_timeout_errors_total=24`
      - `parseability_generation_parser_rejections_total=0`
      - `parse_full_passes=16/16`
      - `perf_observed_generate_avg_ms=214`
      - `perf_observed_generate_max_ms=648`
  - measured movement vs the previous retained bounded run:
    - shadow acceptance improved `68/73 -> 73/73`
    - replay targets improved `4608 -> 4217`
    - helper timeout totals improved `31 -> 24`
    - the 2023 replay debt no longer carries `module_declaration` / `module_declaration_sv_2023` as unresolved rule debt
  - interpretation:
    - this is still proof-lane hardening, not a closure claim
    - the durable lesson is frontend-semantic, not merely local:
      - standalone annotations are the maintained rule-level form
      - inline same-line annotations inside a rule body are branch-local, even when the rule has only one alternative
- Literalish sample steering is now a broader stimuli-runtime tool too.
  - retained runtime widening:
    - `rust/src/ast_pipeline/stimuli_generator.rs` now honors literalish semantic hints for:
      - regex atoms
      - non-regex non-OR rules
      - branch-local OR alternatives
    - branch-local inline `@sample` can now short-circuit a targeted OR branch while still recording branch success
  - retained main-SV use:
    - `grammars/systemverilog.ebnf` now uses parser-proven inline branch samples on selected `assignment_pattern`, `case_statement`, `clocking_declaration`, `conditional_statement`, struct/enum `block_data_type` / `data_type`, simple `function_body_declaration` / `task_body_declaration`, and `net_type_declaration_sv_2017` branches
  - retained focused evidence:
    - `sv_2017`: `180/181 accepted, 1 parser reject`, `319/2613` replay targets resolved in the retained 200-attempt loop
    - `sv_2023`: `179/180 accepted, 1 parser reject`, `387/2393` replay targets resolved in the retained 200-attempt loop
    - the surviving rejects reduced to tiny comment/attribute/preprocessor-like fragments rather than any of the newly seeded branches
  - steering consequence:
    - branch-local `@sample` is now a legitimate narrow closure lever when backed by parser-proven witnesses
    - the older warning still stands: do not resume blind blanket `@sample` sweeps just because this runtime path now exists
- VHDL still has useful local triage tooling if it ever needs to be revisited.
  - non-default `--entry-rule` plus `--validate-parseability` stays intentionally rejected in `ast_pipeline`; full-entry validation only
  - `coverage_gap_triage` plus `top_failure_reasons` remains the right resume surface if VHDL closure is reopened later
- The next retained main-SV slice after that is declaration-family OR steering, and the keepable lesson is runtime/frontend boundary behavior rather than a new parser capability.
  - retained root cause:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
      - `node_supports_rule_literal_override(...)` still returns `false` for `ASTNode::Or`
    - so ordinary rule-level literal overrides on declaration-family `Or` rules were structurally dead
  - kept grammar-side repair:
    - `grammars/systemverilog.ebnf`
      - declaration-family steering now uses inline branch-local `@sample` on the declaration alternatives
      - `module_declaration` / `program_declaration` wrapper branches now also carry branch-local canonical samples
  - direct proof:
    - `module_declaration -> module m; endmodule`
    - `program_declaration -> program p; endprogram`
    - `module_declaration_sv_2017 -> module m(a); endmodule`
    - `program_declaration_sv_2017 -> program p(a); endprogram`
    - matching `sv_2023` probes return the same canonical forms
  - retained bounded proof:
    - `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-decl-samples-r1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - outcome:
      - `closed_loop_profiles_passed=2/2`
      - `closed_loop_replay_targets_total=4423`
      - `closed_loop_parseability_shadow_accepted_total=90`
      - `closed_loop_parseability_shadow_parser_rejections_total=0`
      - `closed_loop_parseability_shadow_target_timeout_errors_total=145`
      - `closed_loop_parseability_shadow_helper_timeout_errors_total=7`
      - `parse_full_passes=16/16`
      - `perf_observed_generate_avg_ms=154`
      - `perf_observed_generate_max_ms=243`
  - retained replay-gap truth:
    - wrapper-level replay debt for `module_declaration` / `program_declaration` disappeared in the bounded sidecars
    - remaining declaration-adjacent bounded replay debt:
      - `module_declaration_sv_2017`
      - `module_declaration_sv_2023`
      - `program_declaration_sv_2017`
      - `program_declaration_sv_2023`
      - `udp_declaration_port_list`
  - architectural rule:
    - for alternation-heavy rules, keep using inline branch-local annotations when deterministic steering is needed today
    - do not assume the rule-level literal-override path will help until the runtime is widened beyond the current `ASTNode::Or` restriction
- The next retained main-SV slice after that is a child-rule UDP foothold, and the keepable lesson is when a seed should live below the family-selection layer.
  - retained trigger:
    - after the declaration-wrapper slice, the bounded replay-gap sidecars still carried:
      - `udp_ansi_declaration`
      - `udp_declaration_port_list`
    - unlike the wrapper-level declaration debt, the UDP family was already being entered
  - kept grammar-side repair:
    - `grammars/systemverilog.ebnf`
      - `udp_declaration_port_list` now has `@sample: "output o, input i"`
    - the sample was intentionally placed on the child list rule rather than on `udp_ansi_declaration` or the surrounding `udp_declaration_sv_*` branch
  - why that placement matters:
    - a new wrapper or branch literal override would likely have repeated the earlier parent-debt short-circuit
    - the child-rule sample makes `udp_ansi_declaration` cheap through real descent
  - retained bounded proof:
    - `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-udp-ansi-r1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - outcome:
      - `closed_loop_profiles_passed=2/2`
      - `closed_loop_replay_targets_total=4158`
      - `closed_loop_parseability_shadow_accepted_total=98`
      - `closed_loop_parseability_shadow_parser_rejections_total=0`
      - `closed_loop_parseability_shadow_target_timeout_errors_total=136`
      - `closed_loop_parseability_shadow_helper_timeout_errors_total=7`
      - `parse_full_passes=16/16`
      - `perf_observed_generate_avg_ms=145`
      - `perf_observed_generate_max_ms=233`
  - retained replay-gap truth:
    - `udp_ansi_declaration` disappeared from both bounded replay-gap sidecars
    - `udp_declaration_port_list` disappeared from both bounded replay-gap sidecars
    - the remaining declaration-adjacent bounded replay debt is now:
      - `module_declaration_sv_2017`
      - `module_declaration_sv_2023`
      - `program_declaration_sv_2017`
      - `program_declaration_sv_2023`
  - architectural rule:
    - when a family is already being entered, prefer a child-rule foothold that preserves real descent
    - keep wrapper/branch short-circuits for genuine family-selection seams only
- The next retained main-SV slice after that is a runtime/report alignment fix for profile-pruned wrappers rather than another grammar annotation move.
  - retained trigger:
    - a focused wrapper-descent experiment showed that active-profile generation had stopped selecting opposite-profile wrapper branches
    - but `generate_gap_report()` was still classifying those branches as reachable `never_selected` debt because it reasoned over the raw source-grammar `Or` rather than the active profile-pruned grammar tree
  - kept runtime/report repair:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
      - `missing_rule_references()` now feeds both `ASTNode::Or` generation-time pruning and branch-debt classification
      - branch debt with missing active-grammar rule references is now marked unreachable with reason `references_rule_missing_from_active_grammar`
      - missing-rule references are excluded from actionable `uncovered_rule_references`, so `depends_on` only carries rules that actually exist in the active grammar tree
      - focused regression tests now pin both halves of the behavior:
        - `stimuli_generation_prunes_or_branches_that_reference_missing_rules`
        - `gap_report_marks_branches_with_missing_rule_references_unreachable`
  - retained bounded proof:
    - `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-profile-prune-r2 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
    - outcome:
      - `closed_loop_profiles_passed=2/2`
      - `closed_loop_replay_targets_total=3878`
      - `closed_loop_parseability_shadow_accepted_total=118`
      - `closed_loop_parseability_shadow_parser_rejections_total=0`
      - `closed_loop_parseability_shadow_target_timeout_errors_total=119`
      - `closed_loop_parseability_shadow_helper_timeout_errors_total=0`
      - `parse_full_passes=16/16`
  - architectural rule:
    - once `@profiles` removes a rule from the active grammar tree, every target-drive and replay-gap surface must respect that effective grammar
    - do not leave impossible cross-profile wrapper branches in actionable debt just because the source grammar still contains them textually
- The next retained main-SV slice after that widened one specific runtime boundary: rule-level literal/probe overrides are no longer blocked on `ASTNode::Or`.
  - retained trigger:
    - a standalone `@probe_sample` experiment on `statement_or_null` exposed that `generate_rule()` still gated rule-level literal/probe overrides through `node_supports_rule_literal_override()`
    - that helper explicitly rejected `ASTNode::Or`, so top-level alternation wrappers could not use the rule-level footholds available to non-`Or` roots
  - kept runtime repair:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
      - `node_supports_rule_literal_override()` now only rejects regex atoms, not `Or` roots
      - this reactivates rule-level `@sample` and active-entry `@probe_sample` for alternation-root rules
      - focused regression tests now cover:
        - rule-level `@sample` on an `Or`-root entry rule
        - rule-level `@probe_sample` on an `Or`-root helper rule that must remain inactive during non-entry expansion
  - experiment truth:
    - the first post-fix grammar use on `statement_or_null` was intentionally reverted after bounded proof regressed to `closed_loop_replay_targets_total=4070`
    - the kept use is narrower:
      - `grammars/systemverilog.ebnf`
      - standalone `@probe_sample: "1"` on `sequence_expr`
  - retained bounded proof:
    - `/tmp/pgen-sv-or-probe-sequence-r1`
    - `closed_loop_profiles_passed=2/2`
    - `closed_loop_replay_targets_total=3870`
    - `closed_loop_parseability_shadow_accepted_total=102`
    - `closed_loop_parseability_shadow_parser_rejections_total=0`
    - `closed_loop_parseability_shadow_target_timeout_errors_total=124`
    - `closed_loop_parseability_shadow_helper_timeout_errors_total=27`
    - `parse_full_passes=16/16`
    - `perf_observed_generate_avg_ms=151`
    - `perf_observed_generate_max_ms=230`
  - architectural rule:
    - the old “rule-level literal/probe overrides are dead on `ASTNode::Or`” statement is no longer true
    - but the broader lesson survived: enabling a capability is not the same as finding the right seam to spend it on
    - `sequence_expr` was a keeper; `statement_or_null` was too broad
