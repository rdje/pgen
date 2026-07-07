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

## Promotion Gates: Ratcheting a No-Regression Floor

Once a parser family is closed, PGEN protects its demonstrated quality with a
*promotion gate* — a deterministic, multi-trial acceptance checkpoint that
re-earns a "this family is ready to run in a stricter mode" recommendation on
every run. The reference instance is the VHDL strict-promotion gate:

```bash
make -C rust SHELL=/bin/bash vhdl_strict_promotion_gate
```

It runs a fixed number of deterministic trials (default `3`) at fixed seeds.
Each trial generates a small closed-loop stimuli batch and measures two things:

- **parse-full ratio** — of the `N` generated samples, how many parse *fully*
  with the real generated parser (not just the bootstrap path),
- **realistic-corpus parity** — the curated realistic corpus produces exactly
  the expected pass/fail split.

When every trial passes, the gate emits
`recommendation: enable_required_strict_mode` with `primary_blocker: none`. The
report and `summary.txt` record the per-trial telemetry (`observed_ratio_min` /
`_max` / `_avg`) so a later session can see the sustained quality, not just a
single lucky run.

The parse-full ratio is checked against a floor, `TARGET_MIN_RATIO`. A trial
fails if its ratio drops below the floor. This is where the *ratchet* lives:

- a floor of `0` is a **no-op** — any ratio is accepted, so the gate proves the
  trials complete but never guards the ratio itself;
- raising the floor to a value the family *sustains* turns the gate into a real
  no-regression net — a future change that degrades closed-loop parse-full
  quality below the demonstrated level now trips the gate instead of passing
  silently.

VHDL's floor is ratcheted to `75`. The evidence: an eight-seed sweep plus the
canonical three-seed run — **eleven distinct seeds, all at `100%` parse-full
(8/8 samples each)**. `75` sits two sample-steps (`25%`) below that demonstrated
minimum: a strong floor (far above the no-op `0`, and far above the family's
own `2026-03-17` historical low of `12%` when the VHDL generator was weaker),
while still tolerating a benign RNG-stream perturbation of one or two samples on
the fixed gate seeds without a false failure. The floor stays overridable for
deliberate experiments:

```bash
# restore the legacy no-op posture for a one-off investigation
PGEN_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO=0 \
  make -C rust SHELL=/bin/bash vhdl_strict_promotion_gate
```

The same floor default is mirrored in the aggregate `sota_exit_gate` policy, so
every path that runs the promotion gate enforces the identical ratchet. The
doctrine generalizes: when a family is a closed no-regression baseline, prefer
ratcheting its promotion floor up to the *measured, sustained* minimum (with a
small margin) over leaving a permissive `0` that proves nothing about the ratio.

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

The first repair attempt used standalone rule-level `@sample` on the declaration rules themselves. Direct probes showed those hints were not taking effect. The kept fix in [systemverilog.ebnf](grammars/systemverilog.ebnf) therefore moved to supported inline branch-local `@sample` placement on the declaration alternatives, plus branch-local canonical samples on the `module_declaration` / `program_declaration` profile wrapper branches.

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

The kept repair in [systemverilog.ebnf](grammars/systemverilog.ebnf) is intentionally small:

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

The kept fix lives in [stimuli_generator.rs](rust/src/ast_pipeline/stimuli_generator.rs):

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

## Per-Target Reach Classification

Each residual target in the coverage gap report carries a **read-only** `reach_classification` field, computed by `generate_gap_report()` (in `../../rust/src/ast_pipeline/stimuli_generator.rs`) purely as analysis over the already-computed targets — it never changes generation, so it cannot affect coverage counts.

It turns the bare residual count into an evidence-backed partition: for each target you can see *why* it is still open, not just *that* it is open. The values are:

- `reachable_by_plan` — a deterministic reach plan exists to steer generation to this branch (it is in principle coverable; remaining miss is a steering gap, not a structural impossibility).
- `no_reach_path` — no reach plan exists to this specific branch from the entry rule as encoded (the reach-path search returned nothing).
- `reachable_rule_not_generated` — a rule-level target that is graph-reachable from the entry but was never successfully generated.

The classification is additive (older report JSON without the field still parses) and is intended as signoff evidence: it answers, per target, whether closing the residual to zero is attainable by steering or whether some targets are structurally out of reach.

Another useful boundary in the same main-SystemVerilog lane was the old runtime restriction on rule-level steering.

Until this slice, rule-level `@sample` and active-entry `@probe_sample` could not fire when the target rule root was an `Or`. That meant top-level wrapper rules with plain alternation could only be steered with branch-local annotations, even when the cleaner design would have been a rule-level helper foothold.

That runtime restriction is now gone in [stimuli_generator.rs](rust/src/ast_pipeline/stimuli_generator.rs). Rule-level literal and probe overrides now work on `Or` roots too, with focused regression tests covering:

- rule-level `@sample` on an `Or`-root entry rule
- rule-level `@probe_sample` on an `Or`-root helper rule that must stay inactive during non-entry expansion

The important practical lesson is that a newly-valid capability is not automatically a good idea everywhere.

The first broad use, a standalone `@probe_sample: ";"` on `statement_or_null`, was intentionally rejected because the bounded maintained gate regressed to:

- `closed_loop_replay_targets_total=4070`

The kept use was narrower:

- [systemverilog.ebnf](grammars/systemverilog.ebnf)
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

## Round-Trip Stability: The Closed Loop Must Not Out-Generate Its Own Parser

The closed-loop generator's most important contract is the one that is
easiest to violate silently: **every sample it emits must parse back.**
A grammar-driven generator can produce a token sequence that is locally
valid at every production step yet, when the parser re-lexes the whole
string left-to-right, re-associates tokens differently than the
generator intended — yielding a structurally-invalid file the parser
*correctly* rejects. That is **generator over-generation**, not a
parser or grammar bug. The fix is always to constrain the generator (we
never loosen the `parser_rejections == 0` precondition, and we never
file a parser bug for output the parser is right to reject).

This section documents the parser/EBNF-agnostic round-trip-stability
hardening landed for the SystemVerilog preprocessor closed loop
(`SV-EXH-PROOF.2.3.2`). Every mechanism below is a **general property
of grammar structure or regex shape** — there is not one grammar rule
name, sigil, or parser identifier in the production logic. They benefit
*any* grammar with the same shape.

### The failure shape, by example

The preprocessor closed loop was emitting files like (minimised):

```text
`ifdef X
`define Y `endif
```

Each line is locally valid: `` `ifdef X `` opens a conditional,
`` `define Y … `` defines a macro whose body is the rest of the line.
But a macro body is line-greedy text, so on re-parse the only
`` `endif `` is **swallowed into `Y`'s macro body** — the conditional
is never closed, the file is invalid SV, the parser rejects it. A
seven-sample matrix pinned the mechanism precisely:

| Sample | Parses? | Why |
| --- | --- | --- |
| `` `ifdef X⏎`define Y `endif `` | **reject** | only `` `endif `` absorbed by the macro body ⇒ unclosed |
| `` `ifdef X⏎`define Y z⏎`endif `` | pass | a real newline ends the macro body; `` `endif `` is its own line |
| `` `define Y `endif `` (no open conditional) | pass | the lexeme is valid *in isolation* — the defect is **contextual** |
| `` `ifdef X⏎`endif `` | pass | plain balanced |
| `` `ifdef X⏎`define Y `endif⏎`endif `` | pass | the *second* `` `endif `` closes it — proving first-match closer-stealing |
| `` `ifdef X⏎`define Y `notakw⏎`endif `` | pass | a non-closer macro-ref is harmless |
| `` `ifdef X⏎`define Y `notakw `` | reject | genuinely unclosed (control) |

The decisive lesson from the matrix: the same lexeme is fine standalone
and fatal inside an open construct, so the constraint must be
**contextual**, never a blanket ban (a blanket ban would also delete
legitimate language coverage).

### Mechanism 1 — scoped structural-closer guard

When a rule has the shape `R := … item* CLOSE` (a quantified/optional
body followed by a *required fixed-literal closer*), the generator now
pushes `CLOSE`'s literal onto a dynamically-scoped forbidden set while
it generates the body, and pops it before generating `CLOSE` itself.
Any **free** terminal (a regex whose HIR has a class / repetition /
alternation — e.g. an identifier-like macro reference) materialised
anywhere in that open body is re-rolled, then cleanly discarded, if its
text contains an active closer lexeme.

- Fixed-literal terminals are *exempt* — a legitimately-nested
  same-construct's own closer still generates ⇒ **nesting-safe**.
- The set is empty when no closer-bearing construct is open ⇒ the
  lexeme is still freely generatable standalone ⇒
  **coverage-preserving** (this is what the `` `define Y `endif ``
  standalone case above requires).

The closer literal is resolved purely structurally (rule reference →
sequence → regex HIR), so a rule like
`pp_endif := kw_endif directive_tail? newline?` correctly resolves to
`` `endif `` (the optional trailing parts are nullable-skipped).

### Mechanism 2 — the structural-sigil hazard gate (the subtle one)

The first cut of Mechanism 1 over-fired. Consider a macro-parameter
list `macro_formals := lparen … rparen`: its closer lexeme is `)`. A
substring guard would then forbid *any* free terminal containing `)` —
including a block comment `/*N)*/` — even though a `)` inside a
`/*…*/` comment is **never re-lexed as the closer** (the comment is one
token). Forbidding it is pure, harmful over-restriction.

The agnostic fix: a closer is a genuine round-trip hazard for free
content **only when its lexeme begins with a grammar-declared
structural sigil** — a character some content rule's author
*leading-negated* (the same `grammar_content_sigils` set used by the
permissive-content rule). `` `endif `` begins with `` ` ``, and
`non_directive_text := /[^`\r\n]…/` leading-negates `` ` ``, so it *is*
a sigil; `)` is ordinary punctuation no content rule negates, so it is
*not*. The guard engages for `` `endif ``, stays inert for `)`. This is
derived entirely from the grammar the author wrote.

> Discipline note: this bug was found by **tracing the actual
> generator decisions on a faithful reproduction**, not by reasoning.
> The first four root-cause theories were each falsified by the gate or
> by a decisive experiment. The rule we keep relearning: *verify the
> mechanism on the failing artifact; the gate is the arbiter, not the
> argument.*

### Mechanism 3 — literal/probe-hint route also guarded

`@sample` / `@probe_sample` literal hints return their string directly
from `generate_rule` / `generate_or`, bypassing the terminal
materialisation path. Under target-drive those routes are active, so
the same closer-collision check is applied there too (skip the hint,
fall through to normal guarded generation). Same principle, every
generation route.

### Mechanism 4 — line-terminator completeness

The deepest case: `pp_define := … macro_body? newline?`. The trailing
newline is *optional* in the grammar, but `macro_body` is line-greedy,
so when the generator skips the newline and another directive follows,
the macro body absorbs it (the failure shape above). The agnostic rule:
a sequence of the form `… <line-greedy content terminal> … <optional
newline terminator>` must **force-emit** that trailing newline
(generate the quantified inner exactly once). "Line-greedy" and
"newline terminator" are both decided from regex HIR
(`regex_is_line_greedy` = an unbounded repetition over a class that
excludes `\n`; `regex_is_newline_only` = language ⊆ `{\r,\n}`,
non-nullable). A trailing newline is universally benign, and the
detector fires *only* on that shape, so grammars without it are
untouched.

This one has a real trade-off, documented here because the trade-off
is the interesting part: forcing the newline removes the
`newline?` = 0 (no-trailing-newline, EOF) variant from the *single
deterministic syntax-probe sample*. That is **not** new dead grammar —
every branch stays statically reachable from entry; the genuine
unreachable surface (gap-report `reason=unreachable_from_entry`) stayed
exactly the benign `trivia` pocket, *and shrank* (`line_comment` became
reachable). The count=1 burn-down arithmetic shifted, so the two
downstream proof contracts were re-baselined **in the same slice**
(syntax-closure `max_unreachable_branches` 13→24 with the arithmetic
spelled out; zero-plausible-gap `allowed_unreachable_rules`
`[line_comment,trivia]→[trivia]`, a *stricter* invariant). A generator
change owns all its downstream proof contracts in-slice — re-baselining
honestly with the genuine surface verified is the doctrine, not
masking.

### Results

- Closed-loop `parser_rejections` for the SV preprocessor:
  `5 / 3 → 0` on both the aggregate and reachability sidecars
  (gate-verified, fresh build).
- `zero_plausible_grammar_level_gap_proof_surface: true`,
  `unmet_proof_criteria: []` — the zero-plausible-gap proof surface is
  established.
- Cross-parser closed loop: full engine test suite green
  (`448` + `468` lib, integration suite green) — the shared
  `generate_sequence` / regex-materialisation path is all-lanes-safe.

### The portable rule

> A closed-loop generator must be **round-trip self-consistent**: a
> generated artifact has to re-parse to the structure the generator
> intended. The hazards are general and recur across grammars — a free
> terminal spelling a structural closer; a line-greedy content terminal
> not newline-terminated before following structure; a free *identifier*
> terminal spelling a reserved keyword that a sibling negative lookahead
> excludes. Detect them from grammar/HIR shape, constrain generation
> contextually (never a blanket ban, never loosen `== 0`), and when a
> correctness fix legitimately shifts a downstream burn-down metric,
> re-baseline that contract honestly in the same slice after proving the
> genuine reachable surface is intact.

### Mechanism 5 — keyword-as-identifier (negative-lookahead) round-trip guard

A fifth instance landed for `rtl_frontend` (`RTL-FE-CLOSURE.6`). Its
`non_keyword_identifier := !kw_always … !kw_wire simple_identifier` rule
excludes every reserved keyword from an identifier position, but the closed
loop could still synthesise an identifier terminal spelling one — e.g. an
enum item `if` — which the parser's `!kw_if` guard then rejects (a
`sample_parse_failure`), or, subtler, re-parses as the *keyword* inside a
different construct (it happens to parse, but to a structure the generator
did not intend). The generator now detects the `!kw … TERM` shape from
grammar/HIR alone — a leading run of negative lookaheads over fixed
keyword-literal rules followed by a terminal — and, on a collision, repairs
the identifier with a **deterministic, RNG-neutral `_` prefix**. `_` is a
valid identifier start and no keyword begins with `_`, so `_`+keyword can
neither *equal* nor `\b`-prefix-match any excluded keyword — covering both
the exact spelling and the `keyword$…` edge that a `\b` word boundary leaves
open. The `_` prefix is chosen over *re-rolling* the terminal precisely
because it consumes no RNG: every other sample in the seed's stream stays
byte-identical, so the certificate-coverage landscape is not perturbed
(re-rolling advances the RNG and silently shifts unrelated samples'
witnesses).

`SV-KEYWORD-PRIMARY-FIDELITY.3.2` (2026-07-04) extended the guard from the
`!kw_A … !kw_Z` **fixed-literal** shape to the `!reserved-regex` shape that
SystemVerilog actually uses — `non_keyword_identifier :=
!reserved_non_keyword_identifier identifier`, whose lookahead target is a
regex **whole-word list** `trivia? /(?:parameter|localparam|…|wire|xor)\b/`
(reached via an `Or` of the profile-gated `reserved_non_keyword_identifier_sv
| reserved_non_keyword_identifier_v2005` sub-rules). Before the extension the
detector only recognised fixed-literal keyword rules, so SystemVerilog's
identifier exclusion was **not** actually enforced on the generation side — a
latent gap: the closed loop could emit a reserved word (e.g. `import or::…`,
where `or` is a reserved net/gate keyword) that the parser then rejects. The
detector now resolves the reserved word list from the regex (all branches must
be identifier-shaped whole words, so no other rule shape is ever misread; the
words are precomputed once per generator from the already-profile-filtered
tree), and the same RNG-neutral `_` repair applies (`or → _or`). The
**safety invariant** is unchanged: the generator excludes a word in an
identifier position only where a `!<rule-matching-word> identifier` sequence
exists — exactly where the parser rejects it — so generator and parser agree
by construction. The change is provably inert for every grammar without such a
shape: the six fully-certified grammars (`json`, `regex`, `rtl_const_expr`,
`systemverilog_preprocessor`, `vhdl`, `rtl_frontend`) are measured
byte-identical (they carry no `!reserved-regex` identifier rule, and
`rtl_frontend`'s `!kw` shape is handled by the original fixed-literal path).
Result: `rtl_frontend` certificate-coverage `sample_parse_failures` `1 → 0`
(the original `RTL-FE-CLOSURE.6` case) and SystemVerilog `sample_parse_failures`
`1 → 0` at the affected seed after the `.3.2` reserved-list extension, with the
witness landscape byte-identical at every seed — generator-only, no parser/grammar
change to those grammars, no schema bump.

## Semantic Round-Trip: Context-Valid Generation

Round-trip self-consistency has a **semantic** dimension as well as a structural one. A grammar can
gate a rule with a **semantic predicate** — a `@predicate` annotation the parser evaluates against its
semantic store — and the generator must honour that same predicate, or it emits a structurally-valid
string the parser *semantically* rejects.

The motivating case is a numeric backreference. The regex grammar gates it with

```ebnf
@emit_fact: { kind: regex_capture_group, name: capture }
capture_open = "("                       # each capture group emits a fact at group-open

@predicate: { name: fact_count_at_least, args: [regex_capture_group, $index], phase: post }
numeric_backreference = "\" backreference_digits   # \NN is valid only if ≥ NN capture groups exist
```

The parser accepts `\3` only when group 3 exists. A predicate-**blind** generator would happily emit
`\98495` in a pattern with no groups — which the parser correctly rejects (PCRE2: error 115, reference
to a non-existent subpattern).

PGEN's generator is **semantic-store-aware**: it maintains a generation-time semantic store, **emits the
same facts the parser would** as it generates (a generated capture group emits `regex_capture_group`),
and **consults the same predicate** before committing a gated rule. A `fact_count_at_least(K, …)` rule is
unsatisfiable when zero `K` facts exist (no positive reference can match an empty set), so the generator
*backtracks* to a satisfiable alternative (a single-digit `\1`, or it generates a capture group first)
instead of emitting an invalid backreference. The store snapshots/rolls back in lockstep with the
generator's own backtracking, exactly like the parser's speculation, so a discarded alternative leaves no
phantom facts behind.

This needs **no new annotation** — the generator becomes a second consumer of the existing
`@emit_fact` / `@predicate` vocabulary, so the *same* grammar steers both parsing and generation. It is
**on only for grammars that declare such a predicate**, so every grammar without one generates
byte-identically (the capability is gated on the grammar's own facts/predicates, never on a grammar
name). The capability is tracked by the `STORE-AWARE-GEN` task tree, which has grown the first
`fact_count_at_least` cut (regex) into the full composable set exercised by **SystemVerilog** — the
name-coordinated *declare-then-use prelude* family (`has_fact` / `fact_attribute_equals` over
`type_name` / `variable_binding` / class-scope facts: the generator emits the prior declaration a gated
use needs, diversifies a free name that would otherwise collide with a consumed fact, and arms preludes
even for gates carried by a mandatory *off-path sibling* along the reach path) plus a literal-threshold
*count-prelude* (`fact_count_at_least` with a constant bound — e.g. a required wildcard import).

The result is verified per grammar with certificate-coverage, where `sample_parse_failures` is the
**semantic round-trip metric**: zero means every generated sample re-parses through the real parser's
predicates. At the current baseline (deterministic at seeds 0/7/42) the six fully-certified grammars
report `UNKNOWN=0`, `fully_certified=true`, `sample_parse_failures=0` (json, regex, vhdl,
systemverilog_preprocessor, rtl_frontend, rtl_const_expr), and SystemVerilog — the predicate-heaviest
grammar — reports `sample_parse_failures=0` with a canonical (`sv_2017`) residual of `UNKNOWN=12` (down
from 20 since `VERILOG-2005-PROFILE.6.7` promoted the 8 `sv_2017`-profile-unreachable SystemVerilog-only
rules to per-profile `proof`) that the sound multi-config recognized union collapses to `UNKNOWN=1` (the
single `context_member_method_call` reach-gap, deferred to a future structured-witness synthesizer). No grammar emits a sample its own
parser semantically rejects.

## Directed (Learned) Generation — the FdLoop Loop

Beyond the diverse pass (coverage-guided random sampling) and the witness pass (deterministic
single-target construction), the generator has a third, opt-in mode: **directed generation with a
learned distribution**, adopted from *FdLoop* (Kirschner & Soremekun, "Directed Grammar-Based Test
Generation", arXiv 2508.01472). Where the witness pass forces one named target, the directed loop
steers the **whole distribution** of generated samples toward a measurable goal.

Each round it:

1. **generates** a batch of samples and **scores** each one against the goal (the *fitness*);
2. **selects** the round's best sample and keeps its *derivation log* — the exact ordered-choice
   branches its generation resolved (recorded per sample, at zero cost when the mode is off);
3. **re-learns** a per-choice-point branch distribution from all selected logs so far (counting
   which alternative won at each `rule::path` choice point — a *probabilistic grammar* learned
   from the best inputs), and **resets one randomly-chosen learned group back to uniform** so the
   loop keeps exploring (FdLoop's exploration mutation, drawn from the run's seeded RNG);
4. **installs** the learned distribution: the next round's weighted branch tournament multiplies
   each alternative's weight by its learned count (composed with — never replacing — the
   grammar's declared probabilities and the coverage-deficit guidance).

The first supported goal is **`k_path`**: fitness = how many *new* k-paths (a rule in the context
of its depth-k ancestor chain — the coverage-universe metric) the sample's generation covered.
Run it with:

```bash
./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --grammar-profile sv_2017 --entry-rule systemverilog_file \
  --directed-generation-goal k_path \
  --directed-rounds 10 --directed-samples-per-round 5 --directed-k 2 \
  --seed 0 [--directed-report-json report.json]
```

The headline always includes the honest comparator — a **same-seed, same-budget diverse baseline**
(a fresh generator generating `rounds × samples_per_round` samples with no learning):

```text
DIRECTED-GENERATION: goal=k_path grammar='systemverilog' entry='systemverilog_file' k=2
  rounds=10 samples_per_round=5 seed=0
  -> directed covered 723/4111 (17.6%) vs diverse baseline 680/4111 (16.5%) [delta +43]
  learned_groups=56
```

Measured on SystemVerilog (`sv_2017`, k=2, 50-sample budget) the directed loop beats the diverse
baseline at every canonical seed — `+43` k-paths at seed 0, `+104` at seed 7, `+13` at seed 42 —
and the run is **deterministic**: the same seed reproduces the identical report byte-for-byte. On
a small saturating grammar (json: 13/13 k-paths for both) directed and diverse honestly tie —
there is nothing left to steer toward.

The related read-only report `--report-k-path-coverage K` prints the plain diverse-pass coverage
(`covered/universe` at depth K) without the loop, using `--count` samples.

Honest bounds: `k_path` is the only goal wired today (the wider goal vocabulary — parser
code-coverage feedback, parse-failure revelation, corpus-mimicry learning from real-world files —
is designed and tracked in the `STIMULI-SIGNOFF` tree, leaf `.4`); learning covers ordered-choice
branch selection, not repetition counts; and the loop is generation-side and opt-in only — the
diverse pass, the witness pass, and certificate coverage stay byte-identical with the mode off
(proven by pre/post byte-compares at seeds 0/7/42).
