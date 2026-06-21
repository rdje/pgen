# GRAMMAR-WELLFORMED.H.12.5.7.2 — M3 prefix property/sequence temporal-operator plannable-reach extension

Generator-only constructive-reach extension that lets the certificate-coverage plannable-witness
pass synthesise a minimal witness for the **prefix** property/sequence temporal operators the
`H.12.5.7.1` adjudication left as a *generator-reach gap* (cause 1), without touching the grammar
or the parser. Models on the `RTL-FE-CLOSURE.5.x` constructive-reach precedents — specifically a
generalisation of the `.5.6` self-recursion-suppression from **direct** to **indirect** recursion.

> Slice `PGEN-GRAMMAR-WELLFORMED-0117` (GENERATOR-ONLY engine change — no grammar/parser/release/
> schema/ledger change). Status: `done` (WHY+WHERE confirmed tools-first; implement landed; A/B green —
> SV `UNKNOWN 67→56`, the 6 fully-certified grammars intact, `--generate-stimuli` byte-identical).
> Reads with [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_features_parser_agnostic_enable_all_parsers]],
> [[feedback_ast_pipeline_parser_agnostic]], [[project_cert_coverage_tournament_loser_leak]],
> [[feedback_always_signoff_decisions]], [[project_witness_pass_per_target_depth_budget]].
> Parent: `GRAMMAR-WELLFORMED.H.12.5.7` (split by `H.12.5.7.1`, `-0116`). Lane 1 of the binding
> 2026-06-17 priority order (SV `UNKNOWN`→0).

## Targets (the M3 prefix cluster, from `H.12.5.7.1`)

The `property_expr_sv_2017` (`grammars/systemverilog.ebnf:4055`) **prefix** branches — each
`keyword (range)? property_expr` (or a case/instance shape) — and `property_case_item`
(`:4047`). `parseability_probe` (`.7.1`) proved each PARSES, so they are reachable ⇒ a witness
exists ⇒ generator-reach gap, not a parser/grammar defect:

`kw_accept_on` · `kw_reject_on` · `kw_sync_accept_on` · `kw_sync_reject_on` · `kw_eventually` ·
`kw_s_eventually` · `kw_nexttime` · `kw_s_nexttime` · `kw_s_always` · `kw_constant`
(inside the `nexttime` branch) · `property_case_item`.

The **infix** `until`-family (`until`/`until_with`/`s_until`/`s_until_with`) and sequence
`intersect`/`within` are NOT in scope — they are the `H.12.5.8` LR-elimination parse defect (the
parser rejects them; the generator-as-oracle adjudicates any candidate as `parsed=false`, so they
stay honestly UNKNOWN and are routed to lane 2, never false-witnessed).

## WHY+WHERE (confirmed tools-first — corrects the `H.12.5.7.1` "or" hypothesis)

Diagnostics (DEBUG `ast_pipeline`, `PGEN_REACH_PATH_DUMP=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1
PGEN_CERT_COVERAGE_DUMP_ALL=1 … --grammar-profile sv_2017 --entry-rule systemverilog_file
--count 40 --seed 0`):

1. **The reach plan IS found and correct.** Every prefix operator gets a `[reach-path]` line ending
   at the right branch, e.g. `kw_eventually_5865020f → property_expr_sv_2017 root/o21/s0`,
   `kw_nexttime → o13/s0`, `kw_accept_on → o29/s0`, `kw_reject_on → o30/s0`,
   `kw_sync_accept_on → o31/s0`, `kw_sync_reject_on → o32/s0`, `kw_s_always → o19/s0`,
   `kw_s_eventually → o20/s0`, `kw_s_nexttime → o15/s0`, `property_case_item → o10/s4`,
   `kw_constant → o14/s1/q/s0`. So `set_reach_plan_for_rule_mode` returns `true` (NOT `no_path`).
2. **`property_expr_sv_2017` is NOT LR-eliminated in the tree the reach planner walks.** The branch
   indices are the original `o0..o33` (grammar order) — there is no `_lr_base`/`_lr_suffix`. So
   `.7.1`'s "transform pipeline LR-eliminates `property_expr`" framing is wrong for the prefix
   branches. (The reach planner walks the pre-/non-eliminated rule-reference graph.)
3. **No `[plannable-probe]` line for any prefix operator** ⇒ each is a generation `Err` (the dump's
   "29 generation failures"). By contrast the non-recursive-body branches witness fine: the FIRST
   branch `o0` (`sequence_expr` → a number) gives `property_expr`/`property_expr_sv_2017` their own
   witnesses, and the `o33` `property_instance` branch (non-recursive body) witnesses too.

**Root cause (authoritative code, `stimuli_generator.rs:7104-7118`).** The `.5.6`
`suppress_recursive_forced_branch` guard scopes re-fire suppression to **direct** self-recursion:
it collects the forced branch's *direct* rule references (`collect_rule_references`, `:4947`) and
tests `refs.contains(current_rule)`. For branch `o21` = `kw_eventually (constant_range)?
property_expr`, the direct refs are `{kw_eventually, constant_range, property_expr}` — **not**
`property_expr_sv_2017`. So the guard is `false`, suppression never fires, and the directive keyed
on `(property_expr_sv_2017, "root")` re-fires on **every** re-entry of `property_expr_sv_2017`.
Re-entry happens because the forced branch's mandatory `property_expr` body re-descends the one-hop
wrapper `property_expr := property_expr_sv_2017` (`:4198`) — an **indirect** recursion. Result:
`eventually eventually eventually …` forced without bound until depth/`max_rule_visits` exhaustion
→ `generate_from_entry_with_optional_timeout` returns `Err` → the rule stays UNKNOWN. The `.5.6`
comment (lines 7090-7103) documents this exact infinite-re-fire failure for the **direct** case
(`unary_expr := bang unary_expr` → `!!!!…`); the prefix temporal operators are the structurally
identical **indirect** case that escapes the direct-only test.

## DESIGN (generalise `.5.6` direct → indirect; GENERAL/parser-agnostic)

In `suppress_recursive_forced_branch` (`:7104`), broaden the self-reference test so it also
recognises a forced branch that can recurse back into `current_rule` **indirectly**:

- Keep the cheap re-entry gate FIRST (`call_stack.count(current_rule) >= 2`) so the (more
  expensive) reachability query only runs on a genuine re-entry during the reach pass — off-reach
  (`reach_plan == None`) and off-recursion behaviour stays byte-identical at zero cost.
- Replace `refs.contains(current_rule)` with
  `refs.contains(current_rule) || refs.iter().any(|r| self.rule_can_reach(r, current_rule))`,
  where `rule_can_reach(from, to)` is a transitive reachability query over the rule-reference
  graph (new helper, memoised closure built once and cached). The direct case (`refs.contains`)
  remains exactly as before; the indirect case (`property_expr` ⤳ `property_expr_sv_2017`) is now
  also caught.

Effect on a suppressed re-entry: `reach_forced_local`/`reach_bypass_branch` become `None`, so the
re-entered `property_expr` falls through to the existing minimal-derivation ordering and takes its
shortest terminating alternative (the `o0 sequence_expr` base) — producing a minimal witness such
as `assert property (eventually <base>)`. The operator fired ONCE on the shallow entry (that is the
witness); the body terminates instead of re-forcing.

- **Keyed purely on structure + the live recursion count, never on rule names** → parser-agnostic;
  benefits ANY grammar with the same indirect-recursion-through-a-wrapper shape (consistent with
  [[feedback_features_parser_agnostic_enable_all_parsers]]).
- **Strictly additive / safe by construction**: the reach plan is only installed during the
  plannable-witness pass, so `--generate-stimuli` carries no reach plan ⇒ byte-identical output.
  The parser remains the sole witness judge (a generated-but-rejected candidate, e.g. an infix
  `until`, can only fail loudly as `parsed=false`, never false-witness).

## Non-Goals

- The infix `until`/`intersect`/`within` parse defect (→ `H.12.5.8`, lane 2).
- The bounded `eventually [r] p` / `s_always [r] p` bracket-range forms (deferred with the
  `H.12.5.8` fix per `.7.1`).
- M2b store-gated identifiers (`H.12.5.6.3`, parked → STORE-AWARE-GEN) and any grammar change.

## Acceptance Criteria

- SV cert `UNKNOWN` strictly decreases from `67` (the prefix cluster becomes `witnessed`), with
  `witness` strictly increasing by the same amount and **zero newly-UNKNOWN** rules.
- Deterministic at canonical seeds `0`/`7`/`42`; `sample_parse_failures (spf) = 0` at all three.
- The six fully-certified grammars (rtl_const_expr, json, svpp, regex, vhdl, rtl_frontend) stay
  `fully_certified=true` (cert byte-identical) — guarded TRULY INERT for grammars with no
  indirect-recursion forcing on a reach path.
- `--generate-stimuli` byte-identical for SV (no reach plan off the witness pass).
- `clippy_on_rust_change` strict source-clean.
- Commit ONLY an improvement (a regression = flawed analysis → revert), TARGETED, per
  [[project_cert_coverage_tournament_loser_leak]] discipline.

## Verification Plan

1. Decisive A/B: rebuild DEBUG `ast_pipeline`; re-run SV cert at seeds 0/7/42 (baseline captured
   pre-change = `total=1289 proof=1 witness=1221 UNKNOWN=67 spf=0`). Confirm UNKNOWN↓, witness↑,
   no newly-UNKNOWN, spf=0.
2. Confirm WHICH rules flipped via `PGEN_CERT_COVERAGE_DUMP_ALL=1` (expect the prefix cluster).
3. Re-run cert for all six fully-certified grammars (their canonical entry/depth) — must stay
   `fully_certified=true`, byte-identical witness counts.
4. `--generate-stimuli` SV byte-identical (diff against a pre-change capture).
5. `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`.

## IMPLEMENTATION (landed)

`rust/src/ast_pipeline/stimuli_generator.rs`, four edits, GENERAL/parser-agnostic:
1. New grammar-scoped thread-local `RULE_REACH_CACHE` (next to `NULLABLE_CACHE`) memoising per-`from`
   transitive reachability over the rule-reference graph.
2. Cleared alongside `NULLABLE_CACHE` on grammar change (same rule-name keying) in `new`.
3. New `rule_can_reach(from, to)` helper — BFS the rule-reference graph (reusing
   `collect_rule_references` + `self.grammar_tree`), memoised; `from == to` is true.
4. `suppress_recursive_forced_branch` (`:7104`) generalised: cheap re-entry gate FIRST
   (`call_stack.count(current_rule) >= 2`), then `refs.contains(current_rule) ||
   refs.iter().any(|r| self.rule_can_reach(r, current_rule))` — direct (`.5.6`) OR indirect recursion.

## OUTCOME — DONE (A/B green)

Decisive A/B (DEBUG `ast_pipeline`, `--manifest-path` rebuilds, git-stash of ONLY the engine change):

- **SV cert `UNKNOWN 67 → 56`** (−11), **`witness 1221 → 1232`** (+11), `total=1289` unchanged,
  **`generation_failures 29 → 0`**, `spf=0` — **deterministic at seeds 0/7/42** (all three
  `witness=1232 UNKNOWN=56`). Pre-fix re-confirmed `UNKNOWN=67` (stash validity).
- **11 resolved = the prefix temporal cluster + a `constant_cast` bonus**: `kw_accept_on`,
  `kw_eventually`, `kw_nexttime`, `kw_reject_on`, `kw_s_always`, `kw_s_eventually`, `kw_s_nexttime`,
  `kw_sync_accept_on`, `kw_sync_reject_on`, `property_case_item`, `constant_cast`. **Zero newly-UNKNOWN**
  (the 56 is a strict subset of the 67).
- The **infix** `until`/`s_until`/`until_with`/`s_until_with` + sequence `intersect`/`within` now
  *generate* (no longer `generation_failures`) but the parser rejects them ⇒ they move to "probe did
  not re-parse" and stay UNKNOWN — correctly routed to `H.12.5.8` (no false witness; the parser judges).
  `kw_constant` stays UNKNOWN (inside the optional `( kw_constant expression )?` sub-branch — minimal
  generation expands it to zero; a separate, narrower deferred question).
- **Six fully-certified grammars intact** (cert verdict matches baseline): json 9/9, regex 198/198,
  systemverilog_preprocessor 74/74, vhdl 216/216, rtl_frontend 169 (168 witness + 1 proof),
  rtl_const_expr 48/48 — all `fully_certified=true`.
- **`--generate-stimuli` byte-identical** for SV (md5 `7eb4374bf6e9897889d34029f4ff645f` pre == post) —
  the reach plan is installed only by the witness pass, so plain generation is unaffected.
- **`clippy_on_rust_change`** strict source clean (the generated-parser stage debt is the pre-existing
  tolerated non-strict 188-site naming debt, unchanged).

Commit: `PGEN-GRAMMAR-WELLFORMED-0117`. SV `UNKNOWN` arc `…→84→67→56`; SV row stays `Mostly Done`
(the only non-fully-certified shipped grammar) — closure-debt RETIREMENT, not a row flip.

## Artifacts (scratch — not tracked)

- `/tmp/h12572/cert_seed0_dumpall.txt` — pre-change baseline cert dump (seed 0).
- `/tmp/h12572/diag_seed0.txt` — reach-path + debug-probe diagnostics confirming the WHY+WHERE.
