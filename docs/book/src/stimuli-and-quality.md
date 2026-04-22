# Stimuli and Quality

The stimuli system is one of PGEN's defining features.

## Why It Matters

PGEN is not satisfied with "the parser compiles." It aims for:

- grammar-aware stimuli generation,
- parseability-aware generation,
- target-driven replay,
- coverage and gap analysis,
- promotion and counterexample retention,
- cross-family quality proof.

## Current Stimuli Doctrine

The live direction for stimuli work now includes the first five planned upgrades in bounded initial form:

1. grammar-aware mutation
2. constrained-random steering
3. stronger near-valid negative generation
4. corpus export / promotion groundwork
5. smarter shrinkers, starting with delimiter-aware structural minimization

The shrinker work is deliberately not complete yet. The first landed slice teaches the existing counterexample minimizer to try balanced `()`, `[]`, and `{}` reductions before and after generic chunk minimization. Future work should push deeper into grammar-tree-aware shrinkers that can drop optional nodes, collapse alternations, reduce repetitions, and prune subtrees while preserving the failing property.

## Cross-Family Rule

Major stimuli-generator upgrades should prove themselves on at least:

- `regex`
- `vhdl`
- `systemverilog`

That rule keeps stimuli work platform-grade instead of grammar-specific.

## Key Quality Lanes

- `stimuli_cross_family_platform_gate`
- family-specific quality gates
- parseability reports and target-driven replay
- bounded contract files and summary artifacts

## Probe-Only Steering

When a family is down to a stubborn replay frontier, PGEN now distinguishes between two kinds of literal steering:

- `@sample` for ordinary always-on literalish steering
- `@probe_sample` for active-entry-only target-drive replay

That split matters because a hint that is useful when probing a single dependency rule can be harmful if it fires everywhere during normal top-level generation. The current maintained rule is:

- use `@sample` when the grammar really should always short-circuit to that literal shape
- use `@probe_sample` when the literal is meant to accelerate targeted replay of a specific rule without flattening ordinary coverage

The next SystemVerilog lesson made that rule more precise. If the retained replay debt is actually carried by a recursive parent branch, probing the child rule can be the wrong seam. A kept example is the 2023 assertion/sequence lane:

- probing `clocking_event_sv_2023` directly made the local surface look simpler but regressed the bounded replay frontier
- probing the recursive parent branch on `sequence_expr` (`clocking_event sequence_expr`) with `@probe_sample: "@clk 1"` improved the retained replay frontier and retired the separate `clocking_event_sv_2023` debt

So the maintained doctrine is:

- use replay-gap evidence to locate the rule or branch that truly owns the debt
- prefer branch-local `@probe_sample` on that owning seam over broad child-rule canonicalization
- keep the recursive branch visible if it still remains open after the local debt around it has been reduced

## Replay Progress Tracing

The heavy replay gates are intentionally quiet by default, but the retained SystemVerilog replay lane now has an opt-in progress surface when a closed-loop stage is CPU-hot and needs inspection.

- `rust/scripts/sv_stimuli_quality_gate.sh` accepts `PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY`
- the maintained shell-gate default is now `low`
- that does not make the terminal noisy because `run_logged` still captures replay-stage output into the stage log files
- direct `ast_pipeline` invocations still keep their own ordinary default trace posture
- use the environment variable when you want something other than the gate default
- low trace now also surfaces:
  - immediate helper-probe activation lines when replay switches away from the primary entry rule
  - helper-probe result lines showing pending-target payoff after each helper attempt

That trace is meant for honest progress visibility during stubborn replay work, not as a replacement for the gate's final summary artifacts. The practical payoff is simple: if a long `profile_2017_closed_loop_replay` run is still active, the stage log is now tail-able by default instead of staying empty unless someone remembered an extra env override first.

## Tracing Doctrine

PGEN now treats tracing as a first-class tool-design requirement, not as optional afterthought logging.

- every new tool or operational binary should implement the same trace levels:
  - `none`
  - `low`
  - `medium`
  - `high`
  - `debug`
- use one shared tracing model instead of bespoke per-tool debug switches
- prefer shared trace helpers/macros over scattered ad hoc prints
- instrument real execution seams:
  - function entry and exit
  - meaningful branch choices
  - fallbacks and retries
  - timeout paths
  - error boundaries

The current Rust reference surface already follows this shape through `TraceVerbosity`, `PGEN_TRACE_VERBOSITY`, and the `pgen_trace*` macros. The maintained direction for future tools is to align with that contract rather than inventing incompatible local tracing schemes.

PGEN now also lets the replay selector learn a little within a single target-drive run. If a helper rule has already retired meaningful target debt earlier in that same run, later probe selection can treat that observed payoff as part of the ranking signal instead of relying only on static dependency heuristics. This is intentionally bounded to the current replay session; it is replay-local guidance, not a persisted cross-run learning system.

Low replay trace now also exposes the helper competition directly. At each helper activation, PGEN can show the selected helper pool plus the top dependency and pending candidates. That makes replay tuning less mystical: you can see whether a stubborn lane is dominated by one stable pending frontier or by rapidly changing dependency probes.

That same comparison now influences selection in one bounded way. PGEN no longer treats the mere existence of a dependency candidate as an absolute trump card. If the top dependency is only a fresh marginal probe while the top pending rule still carries a much broader untouched frontier, the pending helper can be selected instead. This is deliberate replay steering, not a claim that pending rules are always better; the tradeoff is that these broader pending probes can be much slower once they begin running.

PGEN now stages that broader pending-frontier escape hatch too. In the maintained cheap replay lane, the pending frontier is only allowed to outrank dependency churn after replay has already stayed stagnant beyond the ordinary helper threshold for a little longer. Low trace exposes that state explicitly as `pending_frontier_unlocked=true|false`, and now also reports the effective unlock threshold plus the configured extra stagnation budget, so users can tell whether a replay stayed in its cheap dependency-first budget or crossed into a heavier pending-frontier regime.

That heavier regime is now a deliberate control surface instead of a hidden constant. `ast_pipeline` exposes `--target-pending-frontier-extra-stagnation`, the maintained default stays at `8`, and the SystemVerilog quality gate can override the same behavior with `PGEN_SV_STIMULI_QUALITY_PENDING_FRONTIER_EXTRA_STAGNATION` for focused proof runs. Stimuli corpus bundle metadata records the configured value too, so replay posture stays auditable after the fact.

The first focused main-SystemVerilog measurement also clarified how that control should be used. Setting the extra stagnation budget to `0` does unlock the heavy lane immediately and can flip the second helper from dependency churn to the broad pending frontier `property_expr_sv_2017`, but the same run became dramatically slower and was still active after more than three minutes of wall clock without finishing the retained 128-attempt probe. So the maintained documentation stance is simple: keep `8` as the default proof posture and treat `0` as an experiment knob, not the ordinary lane.

PGEN now pairs that heavier knob with an explicit safety rail too. `ast_pipeline` exposes `--target-helper-generation-timeout-ms`, the maintained default is `1000`, `0` disables the helper timeout, and the SystemVerilog quality gate can override the same behavior with `PGEN_SV_STIMULI_QUALITY_TARGET_HELPER_TIMEOUT_MS`. That timeout applies only to alternate helper-entry probes, not to ordinary primary-entry generation.

That follow-up changed the heavy-lane story in an important way. Reusing the same focused `sv_2017` immediate-unlock repro, the run that previously stalled now completes the full retained 128-attempt probe at `970/2593` resolved with `7` bounded helper timeouts on `property_expr_sv_2017`. So the doctrine remains: `8` is still the maintained default posture, but immediate unlock is no longer operationally hostile in the same way because broad helper probes are now effort-bounded.

PGEN now also surfaces those bounded failures directly in the replay-facing artifacts. Target-driven summaries report `helper_timeout_errors`, validator-backed parseability reports preserve the same counter inside `target_drive_validation`, and stimuli corpus bundles retain it too. That means a future replay investigation can distinguish "generic generation error" from "helper budget fired" without scraping low trace by hand.

That distinction now survives the shell gate layer as well. The maintained annotation, SystemVerilog preprocessor, SystemVerilog replay-shadow, and VHDL replay-shadow quality surfaces now preserve `helper_timeout_errors_total` anywhere they already republish target-drive validation, so the operator-facing summaries do not collapse helper-budget expirations back into anonymous generation churn.

PGEN now makes the same kind of containment available for primary target-drive attempts too, but in a deliberately stricter form. The new `--target-generation-timeout-ms` budget applies to canonical-entry target-drive attempts, defaults to `0`, and is therefore opt-in rather than silently changing the maintained proof posture. The design intent is not "make replay faster at all costs." It is "give investigators a clean way to bound pathological canonical attempts when a local proof run is clearly spending minutes inside one attempt."

The maintained main-SystemVerilog shell workflow now adds one narrow practical layer on top of that runtime rule. `sv_stimuli_quality_gate.sh` defaults its gate-local primary budget to `5ms`, while the underlying CLI/runtime default still remains `0`. That change was made after a real contract-default rerun got past the old `epsilon` and `simple_identifier_no_scope` seams but could still sit indefinitely inside one canonical replay attempt. So the doctrine is:

- runtime/API default: `0`
- main-SV shell-gate default: `5`
- explicit shell override back to legacy unbounded mode: `PGEN_SV_STIMULI_QUALITY_TARGET_GENERATION_TIMEOUT_MS=0`

That primary budget is also surfaced honestly in the replay-facing artifacts. Target-driven summaries now report `target_timeout_errors` separately from `helper_timeout_errors`, validator-backed target-drive telemetry preserves the same counter, stimuli corpus bundle metadata records the configured `target_generation_timeout_ms`, and the main SystemVerilog shell gate now accepts `PGEN_SV_STIMULI_QUALITY_TARGET_GENERATION_TIMEOUT_MS` while republishing `target_timeout_errors_total` in replay-shadow aggregate output. So a future session can distinguish:

- generic generation churn
- helper-probe budget expiry
- primary target-drive budget expiry

without reconstructing that story from low trace alone.

That distinction now survives much more of the shell stack too. The maintained annotation and SystemVerilog preprocessor direct gates, the VHDL replay-shadow summary surface, the SystemVerilog/VHDL promotion reports, and the aggregate `sota_exit_gate` summary layer now all preserve `target_timeout_errors_total` wherever they already repackage target-drive validation. That matters for continuity: once the runtime tells the truth about primary timeout pressure, the higher-level reports should not erase that distinction on the way up to the user-facing summary.

PGEN now uses the same "smallest honest fix" doctrine for stubborn helper-entry grammar seams too. The kept main-SystemVerilog example is `property_case_item`: once primary attempts were bounded, the next retained 2017 replay log showed the helper repeatedly timing out while trying to rediscover the simplest legal property-case forms. The fix was not a broad generation rewrite. It was two helper-only branch seeds in the grammar:

- `@probe_sample: "1: 1;"`
- `@probe_sample: "default: 1;"`

That distinction matters. `@probe_sample` gives the alternate-entry helper a deterministic foothold without flattening ordinary generation the way a blanket `@sample` would. After that repair, the retained bounded `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128` main-SV gate now completes both profiles, and the old `property_case_item` wedge disappears from `profile_2017_closed_loop_replay.log`; the first visible helper pivot moves on to `expression` and retires `91` targets in one probe. The doctrinal lesson is simple: if a helper-entry rule has an obvious canonical fragment shape, prefer a probe-only seed before reaching for heavier runtime changes.

## Rule-Level Vs Branch-Level Annotation Placement

One practical lesson from the same SystemVerilog closure lane is that annotation placement is not cosmetic.

- a standalone annotation line above a rule definition is rule-level
- a same-line inline annotation inside the rule body is branch-level

That distinction is easy to miss on single-alternative rules, because there is only one branch. But the semantics still differ. In the retained main-SV header slice, the first attempt used inline same-line `@sample` on:

- `module_ansi_header`
- `module_nonansi_header`
- `program_ansi_header`
- `program_nonansi_header`

The generation dump showed those hints landing in `branch_semantic_annotations`, not rule-level `semantic_annotations`, so ordinary direct generation still emitted noisy organic headers. After moving those same samples into standalone annotation lines, direct entry probes returned the intended canonical headers immediately:

- `module m(input logic a);`
- `module m(a,b);`
- `program p(input logic a);`
- `program p(a,b);`

The bounded retained `128`-attempt main-SV gate then improved from:

- parseability-shadow acceptance `68/73`
- replay targets `4608`
- helper timeout totals `31`

to:

- parseability-shadow acceptance `73/73`
- replay targets `4217`
- helper timeout totals `24`

The maintained rule is therefore simple:

- use standalone annotations when the steering is meant to apply to the rule as a whole
- use inline annotations when the steering is deliberately branch-local
- do not assume a single-alternative rule makes inline placement “close enough”

## Profile Symmetry Matters

Another practical lesson from the active main-SystemVerilog replay lane is that stubborn debt is sometimes just profile asymmetry in disguise.

The retained `net_declaration` seam looked at first like a runtime bug: a `debug` direct-entry probe on `net_declaration_sv_2023` generated a huge organic sample instead of short-circuiting. Max-trace inspection showed the real story. The runtime was not ignoring a branch literal hint; `net_declaration_sv_2023` simply had no helper-only branch probes, while the sibling `net_declaration_sv_2017` rule already had:

- `@probe_sample: "wire a;"`
- `@probe_sample: "nt a;"`
- `@probe_sample: "interconnect logic a;"`

Mirroring those same three branch-local `@probe_sample` footholds onto `net_declaration_sv_2023` restored the intended direct replay behavior. The direct `debug` probe now reports `Selected OR branch literal hint` and emits `interconnect logic a;`, and the retained bounded main-SV shell proof improved its replay frontier from `3870` targets to `3860`.

The honest lesson is simple:

- use max-trace evidence when a seam feels mysterious
- compare sibling profiles before assuming the runtime is wrong
- keep the real metric in view after the fix

That last point matters. This slice was worth keeping because replay debt improved, but some secondary telemetry moved sideways, so the right takeaway is “real frontier reduction,” not “everything got uniformly better.”

## Main-SV Runtime Reuse

One practical lesson from the active main-SystemVerilog closure lane is that "slow proof" is not always "hard grammar." Sometimes it is just repeated front-end work.

PGEN now keeps a normalized generation-input bundle around for the active main-SV quality gate:

- `ast_pipeline --dump-gen-ast` writes a directly reloadable transformed-style bundle
- older metadata-free generation-AST dumps still load for continuity
- `rust/scripts/sv_stimuli_quality_gate.sh` now emits that bundle once during parser generation and reuses it for the later closed-loop and per-sample `ast_pipeline` invocations

That change matters because a bounded retained rerun had been failing its performance budget on a tiny already-accepted sample at about `17061ms`. After switching the gate to reuse the normalized bundle, the same bounded proof slice (`PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128`, `PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0`) now passes with:

- `closed_loop_profiles_passed=2/2`
- `parseability_generation_parser_rejections_total=0`
- `parse_full_passes=16/16`
- `perf_observed_generate_avg_ms=173`
- `perf_observed_generate_max_ms=624`

The doctrinal point is simple: keep the cheaper runtime path, but do not overclaim from it. This is a real proof-lane improvement for main SystemVerilog, not a declaration that the full family is closed.

## Primary Source Docs

- `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`
- `docs/reference/STRESS_TEST_STANDARDIZATION.md`
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `regex_corpus_bundle/README.md`

## Alternation Steering Boundary

One practical lesson from the active main-SystemVerilog closure lane is that not every annotation placement can steer every runtime node shape.

- ordinary rule-level literal overrides still do not fire on `ASTNode::Or`
- so a standalone rule-level `@sample` above an alternation-heavy declaration rule can look correct in the grammar and still be operationally dead

That mattered directly on the retained declaration-family replay frontier. After the header-seeding slice, the next bounded main-SV proof debt clustered around declaration forms such as:

- `module_declaration*`
- `program_declaration*`
- `udp_declaration_port_list`

The first repair attempt used standalone rule-level `@sample` on the declaration rules themselves. Direct probes showed those hints were not taking effect. The kept fix in [systemverilog.ebnf](/Users/richarddje/Documents/github/pgen/grammars/systemverilog.ebnf) therefore moved to supported inline branch-local `@sample` placement on the declaration alternatives, plus branch-local canonical samples on the `module_declaration` / `program_declaration` profile wrapper branches.

The direct entry probes now return the intended canonical declaration footholds:

- `module_declaration` -> `module m; endmodule`
- `program_declaration` -> `program p; endprogram`
- `module_declaration_sv_2017` -> `module m(a); endmodule`
- `program_declaration_sv_2017` -> `program p(a); endprogram`

Matching `sv_2023` probes return the same canonical declaration shapes.

The retained bounded proof:

- `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-decl-samples-r1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`

records:

- `closed_loop_profiles_passed=2/2`
- `closed_loop_replay_targets_total=4423`
- `closed_loop_parseability_shadow_accepted_total=90`
- `closed_loop_parseability_shadow_parser_rejections_total=0`
- `closed_loop_parseability_shadow_target_timeout_errors_total=145`
- `closed_loop_parseability_shadow_helper_timeout_errors_total=7`
- `parse_full_passes=16/16`
- `perf_observed_generate_avg_ms=154`
- `perf_observed_generate_max_ms=243`

The right interpretation is deliberately narrow.

- wrapper-level replay debt for `module_declaration` and `program_declaration` disappeared in the bounded replay-gap sidecars
- the declaration-family frontier did not fully close
- the remaining declaration-adjacent bounded replay debt is still:
  - `module_declaration_sv_2017`
  - `module_declaration_sv_2023`
  - `program_declaration_sv_2017`
  - `program_declaration_sv_2023`
  - `udp_declaration_port_list`

So the maintained rule is simple:

- use standalone annotations when the steering is meant for a non-`Or` rule as a whole
- use inline branch-local annotations when the rule is an alternation-heavy `Or` surface and the runtime needs deterministic footholds today
- do not overclaim a frontier move as closure just because wrapper-level replay debt disappeared

## Child Footholds

The next retained main-SystemVerilog slice showed the complementary rule.

If a family is already being entered, prefer a child-rule foothold that preserves real descent instead of adding another parent or wrapper short-circuit.

That was the right move for the ANSI UDP seam. After the declaration-wrapper slice, the bounded replay-gap sidecars still carried:

- `udp_ansi_declaration`
- `udp_declaration_port_list`

This was different from the wrapper-level declaration seam:

- the UDP family was already being entered
- the missing debt was inside the ANSI child path
- so another wrapper literal override would have risked retiring only parent debt again

The kept repair in [systemverilog.ebnf](/Users/richarddje/Documents/github/pgen/grammars/systemverilog.ebnf) is intentionally small:

- `udp_declaration_port_list` now carries `@sample: "output o, input i"`

That placement matters. It makes the ANSI UDP path cheap through real descent:

- `udp_declaration_sv_*`
- `udp_ansi_declaration`
- `udp_declaration_port_list`

The retained bounded proof:

- `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-udp-ansi-r1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`

records:

- `closed_loop_profiles_passed=2/2`
- `closed_loop_replay_targets_total=4158`
- `closed_loop_parseability_shadow_accepted_total=98`
- `closed_loop_parseability_shadow_parser_rejections_total=0`
- `closed_loop_parseability_shadow_target_timeout_errors_total=136`
- `closed_loop_parseability_shadow_helper_timeout_errors_total=7`
- `parse_full_passes=16/16`
- `perf_observed_generate_avg_ms=145`
- `perf_observed_generate_max_ms=233`

The replay-gap sidecars now no longer carry:

- `udp_ansi_declaration`
- `udp_declaration_port_list`

and the remaining declaration-adjacent bounded frontier is down to:

- `module_declaration_sv_2017`
- `module_declaration_sv_2023`
- `program_declaration_sv_2017`
- `program_declaration_sv_2023`

So the practical rule is:

- use wrapper or branch short-circuits when the real problem is selecting a family at all
- use child-rule footholds when the family is already being entered and you want honest descent through the missing inner path

## Active-Profile Truth

One more practical lesson from the same main-SystemVerilog lane is that the runtime and the proof surface have to agree about what the active grammar actually is.

That mattered after the declaration and UDP foothold slices. A focused wrapper-descent experiment showed something subtle:

- active-profile generation was no longer selecting opposite-profile wrapper branches such as `module_declaration -> module_declaration_sv_2023` while running under `sv_2017`
- but the replay-gap sidecars were still reporting those branches as reachable `never_selected` debt

That is not just noisy reporting. It misstates the real closure frontier. Once `@profiles` removes a referenced rule from the active grammar tree, that branch is not an actionable replay target anymore.

The kept fix lives in [stimuli_generator.rs](/Users/richarddje/Documents/github/pgen/rust/src/ast_pipeline/stimuli_generator.rs):

- `ASTNode::Or` generation now prunes alternatives whose referenced rules are missing from the active grammar tree
- `generate_gap_report()` now mirrors that same rule and classifies those branches as `unreachable_branch_debt`
- the explicit recorded reason is `references_rule_missing_from_active_grammar`

The retained bounded proof:

- `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-profile-prune-r2 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`

records:

- `closed_loop_profiles_passed=2/2`
- `closed_loop_replay_targets_total=3878`
- `closed_loop_parseability_shadow_accepted_total=118`
- `closed_loop_parseability_shadow_parser_rejections_total=0`
- `closed_loop_parseability_shadow_target_timeout_errors_total=119`
- `closed_loop_parseability_shadow_helper_timeout_errors_total=0`
- `parse_full_passes=16/16`

The important interpretation is again narrow and honest:

- this does not close the remaining declaration or UDP frontier
- it does remove bogus opposite-profile wrapper debt from the reachable/actionable lane
- the remaining reachable debt is now the real active-profile frontier, not proof-surface churn caused by branches that cannot exist in the active grammar tree

So the maintained rule is:

- if a rule reference is pruned out by profile selection, generation and replay-gap accounting must both treat that branch as outside the active grammar
- proof surfaces should report impossible branches as unreachable, not as reachable work we merely failed to hit

## Or-Root Rule-Level Probes

Another useful boundary in the same main-SystemVerilog lane was the old runtime restriction on rule-level steering.

Until this slice, rule-level `@sample` and active-entry `@probe_sample` could not fire when the target rule root was an `Or`. That meant top-level wrapper rules with plain alternation could only be steered with branch-local annotations, even when the cleaner design would have been a rule-level helper foothold.

That runtime restriction is now gone in [stimuli_generator.rs](/Users/richarddje/Documents/github/pgen/rust/src/ast_pipeline/stimuli_generator.rs). Rule-level literal and probe overrides now work on `Or` roots too, with focused regression tests covering:

- rule-level `@sample` on an `Or`-root entry rule
- rule-level `@probe_sample` on an `Or`-root helper rule that must stay inactive during non-entry expansion

The important practical lesson is that a newly-valid capability is not automatically a good idea everywhere.

The first broad use, a standalone `@probe_sample: ";"` on `statement_or_null`, was intentionally rejected because the bounded maintained gate regressed to:

- `closed_loop_replay_targets_total=4070`

The kept use was narrower:

- [systemverilog.ebnf](/Users/richarddje/Documents/github/pgen/grammars/systemverilog.ebnf)
- standalone `@probe_sample: "1"` on `sequence_expr`

Direct probes now emit only `1` for both profiles:

- `sv_2017`
- `sv_2023`

The retained bounded proof:

- `PGEN_SV_STIMULI_QUALITY_STATE_DIR=/tmp/pgen-sv-or-probe-sequence-r1 PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=128 PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`

records:

- `closed_loop_profiles_passed=2/2`
- `closed_loop_replay_targets_total=3870`
- `closed_loop_parseability_shadow_accepted_total=102`
- `closed_loop_parseability_shadow_parser_rejections_total=0`
- `closed_loop_parseability_shadow_target_timeout_errors_total=124`
- `closed_loop_parseability_shadow_helper_timeout_errors_total=27`
- `parse_full_passes=16/16`
- `perf_observed_generate_avg_ms=151`
- `perf_observed_generate_max_ms=230`

That is a small but real replay improvement over the retained `3878` baseline, and it gives us a sharper rule for future work:

- `Or`-root rule-level probes are now a valid runtime tool
- still spend them on the narrowest helper seam that actually lowers replay debt
- do not keep a broader probe just because the runtime now supports it
