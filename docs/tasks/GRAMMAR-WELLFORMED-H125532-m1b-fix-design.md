# GRAMMAR-WELLFORMED.H.12.5.5.3.2 — M1b fix: edit-surface map + implementation design

Design note for the **M1b fix** leaf of the SystemVerilog `UNKNOWN`→0 drive
(`GRAMMAR-WELLFORMED.H.12`). Opened by `H.12.5.5.3.1` (`PGEN-GRAMMAR-WELLFORMED-0088`,
the WHY+WHERE investigation). Produced by `PGEN-GRAMMAR-WELLFORMED-0089` (PURE-DOCS;
baseline re-reproduced + the exact generator edit surface mapped tools-first, so the
implementation lands surgically without re-deriving the surface).

> Scope: this is layer-B detail behind the `H.12.5.5.3.2` frontier row. The mechanism
> (B-i / B-ii) + WHY+WHERE live in the `-0088` `CHANGES.md` / `DEVELOPMENT_NOTES.md`
> entries; this note adds the **code edit surface** + the **implementation plan** +
> the **verification protocol**, and is NOT a substitute for changing one thing and
> measuring the GLOBAL metric per [[feedback_no_codebase_change_without_tool_backed_facts]].

## Baseline (reproduced this session — deterministic ⇒ signal)

```
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./target/debug/ast_pipeline ../grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0
# CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40
#   total=1292 proof=1 witness=1198 UNKNOWN=93 fully_certified=false
#   (sample_parse_failures=0, proof_reverify_failures=0)
```

Byte-identical to the post-`-0087` baseline ⇒ the metric is signal. The prebuilt
`rust/target/debug/ast_pipeline` carries the right features (`generated_parsers,ebnf_dual_run`):
witnesses verify and `spf=0`. UNKNOWN=93 decomposes (per `-0083`/`-0084`/`-0088`):
20 `no_path` non-defects + 73 reachable-unwitnessed = **M1b (~8)** + M2 (~40) + M3 (~14).

M1b carriers (all `parsed=true witnessed_target=false`, organic samples, NOT `@sample` shells):
`array_range_expression`, `bit_select_expression`, `direct_index_method_call`,
`goto_repetition`, `context_member_method_call`, `constant_let_expression`,
`class_scoped_tf_call`, `sequence_method_call`.

## Mechanism (from `-0088`, restated briefly)

The reach plan forces the path to the target's **parent** and selects the parent branch
that references the target, but does **not** steer the **target rule's OWN internal
structure**. The target generates minimally (`construct_mode`: root `Or`→`o0`, `?`/`*`→min),
and for M1b the minimal form is sibling-ambiguous, so the PEG ordered choice attributes the
bytes to an earlier sibling. Two sub-classes:
- **B-i (degenerate first alternative):** target is an `Or` whose `o0` is a bare pass-through
  a sibling also accepts (`array_range_expression` `o0=expression`;
  `callable_method_call_body` `o0=built_in_method_call`); the distinguishing forms are `o1+`.
- **B-ii (distinguishing token behind a minimal optional):** target is a `Seq` whose
  distinguishing tokens sit behind a `?`/`*` minimal-expanded to absent
  (`context_member_method_call`'s `.method(args)` body; `goto_repetition`).

## Edit surface (file: `rust/src/ast_pipeline/stimuli_generator.rs`)

Line numbers are as-of `-0089`; confirm at implementation (the file moves). Cite function
names as primary.

1. **`ActiveReachPlan` struct** (`~:1068`). Fields used here:
   - `directives: HashMap<(String, String), usize>` — `(rule_name, node_path) → forced
     branch index`. Read by `forced_branch_for` (`~:1232`).
   - `forced_quantifier_min: HashMap<(String, String), usize>` — `(rule_name,
     quantified-node-path) → min repeats`.
   - `bypass_fuel`, `target_group_key`/`target_branch_index` (outcome identity),
     `baseline_*_hits`, `prelude` (C2.2 — leave untouched; `None` here).
   - `needs_rule_body_descent(rule)` (`~:1254`) — true when the plan forces any directive
     or quantifier keyed on `rule`. **Interaction:** forcing `(R,"root")` makes
     `needs_rule_body_descent(R)` true ⇒ R's own rule-level `@sample`/`@probe_sample`
     stands down (the `-0087` M1a behavior). For an M1b target we WANT R's body generated
     (not its canonical literal), so this stand-down is **desired** — confirm, don't fight it.

2. **`set_reach_plan_for_rule`** (`~:2710`). Builds the plan from `reach_hops(entry, R)`:
   for each `(hop_rule, hop_site_path)` it extends `chain` via `directives_along_path` and
   `quantifier_sites` via `quantifier_sites_along_path`, then forces each on-path quantifier
   site to min 1 (`~:2744`). **Gap:** the hops run entry → R's *reference site*; nothing
   forces sites **inside R's own body**. This is where (or adjacent to where) the target's-own
   forcing is injected, OR a new variant `set_reach_plan_for_rule_with_own(R, root_branch_j,
   force_inner_quantifiers)` is added that the retry loop calls per attempt.

3. **`reach_hops`** (`~:5224`) → `Vec<(rule, hop_site_path)>` BFS shortest-hop path, target
   NOT included. `directives_along_path` (`~:5535`) parses `oN` segments of a node-path into
   `ReachDirective{rule,node_path,branch_index}`; `quantifier_sites_along_path` (`~:5563`)
   parses `q` segments into `(rule, prefix-before-q)`. These are **string-path** parsers; they
   do NOT walk the tree — so enumerating R's OWN branches/quantifiers needs a new tree walk
   (item below).

4. **Read-sides already honor `(rule, node_path)` keying:**
   - `generate_or` (`~:6441`) reads `forced_branch_for(current_rule, node_path)` and orders the
     forced branch first (`reach_forced_local`, `~:6513`); applies a bypass if the forced branch
     was depth-pruned (`reach_bypass_branch`, `~:6452`).
   - `generate_quantified` (`~:7753`) reads `forced_quantifier_min.get((current_rule,
     node_path))` and emits exactly `forced_min.max(min_repeat).min(bounded_max…)`.
   - ⇒ Populating the plan with `(R,"root")→j` and `(R, <inner q path>)→1` is sufficient to
     steer R's own body; **no read-side change needed.** R's body root `Or` is at node_path
     `"root"`; nested sites are `"root/…"`.

5. **`generate_plannable_rule_witnesses`** (`~:2947`; per-rule loop `~:3015`). The retry loop:
   installs the plan, runs `≤ max_attempts_per_rule` attempts in `construct_mode`, judges each
   via the `witness_check(rule, &sample)` callback returning `PlannableProbeVerdict`
   (`Witnessed` / `ParsedNotWitnessed` / `NotParsed`), `clear_reach_plan()`, then a two-tier
   budget escalation. **This is where the M1b escalation belongs:** when the base plan yields
   `ParsedNotWitnessed`, escalate by re-installing the plan with the target's own root-`Or`
   forced to a non-degenerate branch + inner quantifiers ≥1, iterating across R's alternatives,
   parser-judged, bounded.

## Implementation plan (one change, then measure GLOBAL)

1. **New tree walker** (first impl step — requires reading the transformed-tree node model +
   `generate_node` dispatch, NOT yet read): for rule `R`, return
   - `root_or_branch_count: Option<usize>` (Some(n) iff R's transformed body root is an `Or[n]`), and
   - `inner_quantifier_paths: Vec<String>` (node-paths of `?`/`*` sites within R's body).
   Use the existing transformed `grammar_tree` (same structure `--dump-gen-ast` prints; the
   degenerate-vs-distinguishing analysis in `-0088` read it).
2. **Escalation in the retry loop:** on `ParsedNotWitnessed` for `R`, for `j` in the
   non-degenerate root-`Or` branches (try `o1..` first; degenerate `o0` is the pass-through that
   already failed), install `set_reach_plan_for_rule(entry, R, fuel)` PLUS
   `directives[(R,"root")] = j` and `forced_quantifier_min[(R, p)] = 1` for each
   `p ∈ inner_quantifier_paths`; generate; `witness_check`; stop on `Witnessed`. Bounded by
   `root_or_branch_count` (cap, e.g. ≤ the existing per-rule attempt budget).
3. **Off-reach byte-identical / inert:** the extra directives populate ONLY inside the
   plannable-witness pass for UNKNOWN targets; off-reach has no plan; the fully-certified roster
   never runs the reach pass; `@sample`-free grammars are inert. Keyed purely on `(R, path)`
   (parser-agnostic, never rule semantics) per [[feedback_ast_pipeline_parser_agnostic]].
4. Lineage: `RTL-FE-CLOSURE.5.3` (non-self-recursive-branch preference) extended to the
   target's OWN body.

## Verification protocol (signoff)

- Rebuild DEBUG `ast_pipeline` (`cargo build --features generated_parsers,ebnf_dual_run --bin
  ast_pipeline`) — generator-only, **NO parser regen**.
- cert-coverage **seed 0** first: expect the ~8 M1b carriers to leave UNKNOWN (witness up),
  `spf=0`, and — critically — the existing **1198 witnesses must not regress** (watch the
  `888→1717` cautionary class). Then **seeds 7 and 42** for determinism (canonical sweep).
- `stimuli_cross_family_platform_gate` (the reach pass touches the closed-loop driver ⇒ the gate
  RUNS — regex + vhdl + SV bounded closed-loop replay must stay green).
- `clippy_on_rust_change` strict-source clean.
- GENERATOR-ONLY ⇒ no parser-regen/grammar/release/schema/inventory/ledger change; off-reach
  output byte-identical by construction. Update the `H.12.5.5.3.2` row → done, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, and the LIVE row only if SV reaches `fully_certified`.

## Open implementation questions

- The transformed-tree node enum + `generate_node` dispatch + how a rule's body node is fetched
  (the walker's foundation) — **read first** before writing the walker.
- "Non-degenerate" branch selection: try all non-`o0` first, or all branches incl. `o0` with
  inner-quantifier forcing? Parser is the judge; bounded search settles it empirically.
- Whether forcing ALL inner quantifiers ≥1 over-constrains (parse failures); may need
  parser-judged subsets. Start minimal (root-`Or` only), add inner-quantifier forcing only if
  B-ii carriers (`context_member_method_call`, `goto_repetition`) stay unwitnessed.
- Per-target attempt budget sizing vs the existing `max_attempts_per_rule` / two-tier escalation.
