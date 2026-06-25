# GRAMMAR-WELLFORMED.H.12.5.8.3 — SVA precedence-cascade IMPLEMENT (sequence layer `.8.3.1`)

Implementation leaf for the `H.12.5.8` SVA infix property/sequence binary-operator parse bug. Root cause
(`.8.1`/`-0118`): `sequence_expr`/`property_expr` are never left-recursion-eliminated (PGEN's eliminator
handles only the indirect bare-reference wrapper-chain, not direct inline `A := A op A`), so the runtime
cycle-breaker blocks every infix branch and `a OP b` rejects at the operator. Fix direction (`.8.2`/`-0119`,
director-decided): **direction A — restructure to a §16 (IEEE 1800-2017 Table 16-3) precedence cascade.**

> Split (this leaf): `.8.3.1` SEQUENCE layer (this file) · `.8.3.2` PROPERTY layer, both profiles (next).
> Status: `.8.3.1` **IN PROGRESS** — sequence cascade implemented + **parse-verified**, but a coverage
> witness-reach regression is pending a fix (see below). Slice `PGEN-GRAMMAR-WELLFORMED-0120` is the
> PURE-DOCS checkpoint that records this progress (no committed code change; the cascade lives uncommitted
> in the working tree — see "Resume state").
> Reads with [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_always_signoff_decisions]],
> [[feedback_be_alert_root_cause_fishy_immediately]], [[project_cert_coverage_tournament_loser_leak]],
> [[project_witness_pass_per_target_depth_budget]], [[feedback_quantified_group_extraction]],
> [[project_ebnf_is_single_source_of_truth]], [[feedback_correctness_before_speed]].

## DIRECTOR DECISION (this session, in response to the surfaced fork)

When the faithful cascade implementation produced a coverage regression (below), I surfaced the fork:
**(1) flat operator-chain** (match the existing SV `constant_expression` `operand (binary_operator operand)*`
idiom — shallow, no regression, but precedence not nested in the AST) vs **(2) keep the cascade + fix the
coverage regression** (AST nests by precedence; needs extra probe/engine work). **Director chose (2): keep
the precedence cascade and fix the coverage separately.** So the cascade stays; the next step is the
coverage-reach fix.

## TOOL-BACKED BASELINE (locked this session, seed 0, count 40, `--grammar-profile sv_2017 --entry-rule systemverilog_file`)

- Pre-change SV cert: `total=1289 proof=1 witness=1232 UNKNOWN=56 spf=0` (matches MEMORY).
- The 6 UNKNOWN keyword rules attributable to the infix bug: `kw_intersect`, `kw_within`, `kw_until`,
  `kw_s_until`, `kw_until_with`, `kw_s_until_with` (the `until`-family are property-layer → `.8.3.2`).
  `kw_throughout` is ALREADY witnessed (its LHS is `expression_or_dist`, non-recursive — `x throughout a`
  parses today), confirming the bug is specifically the *left-recursive* infix branches.
- Pre-change infix matrix (`parseability_probe --parse systemverilog --profile sv_2017`):
  `a and b`/`or`/`intersect`/`within`/`a ##1 b` **reject** at the operator; bare `a`, `##1 a`,
  `x throughout a`, `first_match(a)` **parse**.
- No AST shape-contract sample lock on the sequence cascade (`systemverilog_v1.json` has 3 samples, none
  reference `sequence_expr`/`seq_*`/`delay_*`).

## THE SEQUENCE CASCADE (validated text — applied to `grammars/systemverilog.ebnf`, replacing the flat `sequence_expr`)

Replaces the previous flat, directly-left-recursive `sequence_expr` (12 branches incl. 5 LR branches:
`delay_binary`/`and`/`intersect`/`or`/`within`). Loosest→tightest per Table 16-3:
`or > and > intersect > within > throughout > ## > unary` (left-assoc except `throughout` right-assoc).
Proven idiom only: `head tail tail*` chains with named tail rules, `-> $1` passthrough for bare operands,
`[$v, $rest*]` 2-element arrays. Exact text (recovery copy — also present uncommitted in the working tree):

```
@probe_sample: "1"
sequence_expr := seq_or_expr -> $1

seq_or_tail := kw_or_1758356d seq_and_expr -> $2
seq_or_expr := seq_and_expr seq_or_tail seq_or_tail*
                       -> {kind: "or",         lhs: $1, rest: [$2, $3*]}
            | seq_and_expr -> $1

seq_and_tail := kw_and_cffa50a3 seq_intersect_expr -> $2
seq_and_expr := seq_intersect_expr seq_and_tail seq_and_tail*
                       -> {kind: "and",        lhs: $1, rest: [$2, $3*]}
             | seq_intersect_expr -> $1

seq_intersect_tail := kw_intersect_6c96caaf seq_within_expr -> $2
seq_intersect_expr := seq_within_expr seq_intersect_tail seq_intersect_tail*
                       -> {kind: "intersect",  lhs: $1, rest: [$2, $3*]}
                   | seq_within_expr -> $1

seq_within_tail := kw_within_f42ef621 seq_throughout_expr -> $2
seq_within_expr := seq_throughout_expr seq_within_tail seq_within_tail*
                       -> {kind: "within",     lhs: $1, rest: [$2, $3*]}
                | seq_throughout_expr -> $1

seq_throughout_expr := expression_or_dist kw_throughout_330dbf47 seq_throughout_expr
                       -> {kind: "throughout", condition: $1, body: $3}
                    | seq_delay_expr -> $1

seq_delay_expr := cycle_delay_range seq_unary ( cycle_delay_range seq_unary )*
                       -> {kind: "delay_head",   delay: $1, body: $2, tail: $3}
                | seq_unary cycle_delay_range seq_unary ( cycle_delay_range seq_unary )*
                       -> {kind: "delay_binary", lhs: $1, delay: $2, rhs: $3, tail: $4}
                | seq_unary -> $1

seq_unary := expression_or_dist ( boolean_abbrev )?
                       -> {kind: "expression",  body: $1, abbrev: $2}
          | sequence_instance ( sequence_abbrev )?
                       -> {kind: "instance",    body: $1, abbrev: $2}
          | lparen sequence_expr ( comma sequence_match_item )* rparen ( sequence_abbrev )?
                       -> {kind: "paren",       body: $2, match_items: $3, abbrev: $5}
          | kw_first_match_2ddb8d09 lparen sequence_expr ( comma sequence_match_item )* rparen
                       -> {kind: "first_match", body: $3, match_items: $4}
          | @probe_sample: "@clk 1" clocking_event sequence_expr
                       -> {kind: "clocking",    event: $1, body: $2}
```

## PARSE VERIFICATION — PASS (rebuilt parser via `make focus_systemverilog` + `ast_pipeline`/`parseability_probe` rebuilt)

All forms via `parseability_probe --parse systemverilog --profile sv_2017`:
- previously-working still parse: `a`, `a [*2]`, `first_match(a)`, `x throughout a`, `##1 a`, `(a)`.
- previously-broken infix NOW parse: `a or b`, `a and b`, `a intersect b`, `a within b`, `a ##1 b`,
  `a ##1 b ##2 c`.
- precedence/assoc chains parse: `a or b and c`, `a and b or c`, `a ##1 b or c`, `a intersect b within c`,
  `a or b or c`.
- Cert confirms `kw_intersect` and `kw_within` now WITNESS, and `sample_parse_failures=0`.

**⇒ The cascade is parse-correct and complete. The released SVA infix sequence parse bug is fixed at the
grammar level.**

## ⚠️ THE OPEN PROBLEM — COVERAGE WITNESS-REACH REGRESSION (`UNKNOWN 56 → 65`)

Post-cascade cert (seed 0): `total=1300 proof=1 witness=1234 UNKNOWN=65 spf=0`. `total` +11 = the 11 new
cascade rules. The cascade WITNESSES `kw_intersect`/`kw_within` (good) but **de-witnesses 9 rules that were
witnessed before**, all reachable only through `seq_unary`:
`boolean_abbrev`, `boolean_abbrev_sv_2017`, `consecutive_repetition`, `non_consecutive_repetition`,
`non_consecutive_repetition_sv_2017`, `goto_repetition`, `const_or_range_expression`, `sequence_abbrev`,
`kw_first_match_2ddb8d09`, plus `direct_method_call` / `known_unscoped_sequence_identifier` (via
`sequence_instance`). The forms still PARSE (`spf=0`) — this is a witness-REACH gap, not a parse defect.
Net regression ⇒ NOT committable per "commit only improvements".

### WHY+WHERE (tool-backed, decisive)

- **NOT depth.** Re-running the cert at `--max-depth 32` and `--max-depth 40` left `UNKNOWN=65` with all
  de-witnessed rules unchanged. The witness-pass per-target depth budget (`max_depth*2 + target_subtree`,
  tuned in `RTL-FE-CLOSURE.5.2/.5.5`; it fully certifies `rtl_const_expr`'s ~14-deep cascade) is adequate.
- **It is a forcing bug.** With `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, the plannable-witness pass generates
  MALFORMED samples for the deep operands, e.g. for `consecutive_repetition`:
  `sequence\foo ;## \foo [*] within ## \foo [*] intersect ## \foo [*] ... endsequence`. Reaching
  `seq_unary`'s operands routes through `seq_delay_expr`'s `delay_head` branch
  (`cycle_delay_range seq_unary`): `cycle_delay_range` (`## \foo`) greedily consumes the operand identifier,
  the following `seq_unary`'s `expression_or_dist` comes out EMPTY, and the forced `boolean_abbrev` lands as
  a bare `[*]` with no host → `## \foo [*]` is invalid SVA → the probe doesn't re-parse → no witness. The
  pass chains this at every cascade operand position. (`504` probe samples did not re-parse vs `456`
  baseline; `0` generation failures.)
- **WHERE:** the plannable witness pass's forced-descent / reach-path construction in
  `rust/src/ast_pipeline/stimuli_generator.rs` (around `compute_reach_path` and the per-target witness
  passes at `:3053`+ / `:3258`+), interacting with the cascade's `seq_delay_expr` `delay_head` branch
  (`cycle_delay_range seq_unary`).

### FIX DIRECTIONS FOR THE COVERAGE-REACH (next session — pick tools-first)

1. **Engine — reach-planner branch preference (most general, likely best).** Make the forced-descent
   prefer the clean passthrough path to `seq_unary` (`seq_delay_expr := … | seq_unary -> $1`) when the
   target operand needs no leading `##`, instead of routing through `delay_head`. Must be additive
   (witnesses MORE, never fewer — verify the 6 fully-certified grammars stay byte-identical, decisive A/B,
   seeds 0/7/42). Compare with `rtl_const_expr` (always-wrap, simple operands) which reaches its deep
   operands cleanly — isolate whether the trigger is the passthrough branch, the complex operands, or the
   `delay_head` operand competition. This fix would also serve the deeper `.8.3.2` property layer.
2. **Grammar — make `seq_unary`'s operands reachable without routing through the `##` delay branches**
   (e.g. separate the leading-`##` head form so the planner's shortest clean path to `seq_unary` does not
   pass through `cycle_delay_range seq_unary`). Grammar-local, but must preserve precedence + parse-acceptance.
3. **Declarative — `@probe_sample`/`@sample` seeds** on `seq_unary` / the repetition rules with complete
   valid host forms (`\foo[*2]`, `\foo[=2]`, `\foo[->2]`, `inst[*2]`, `first_match(a)`) so the cert
   witnesses them via clean seeds rather than the broken forced reach. Lowest blast radius; may need several
   seeds (boolean_abbrev has 3 kinds).

## RESUME STATE (working tree — for the next session)

- **Committed/baseline state:** flat `sequence_expr`, SV cert `UNKNOWN=56` (`systemverilog` row unchanged,
  `Mostly Done`). This PURE-DOCS checkpoint commits NO code change.
- **Working tree (UNCOMMITTED, self-consistent):** `grammars/systemverilog.ebnf` holds the cascade above;
  `generated/` holds the matching regenerated SV parser (untracked); `ast_pipeline` (debug, features
  `generated_parsers ebnf_dual_run`) and `parseability_probe` (release) are rebuilt against it. So a cert
  run NOW reports `UNKNOWN=65` (the WIP). The exact cascade is also captured above for recovery.
- **Canonical cert reproduce:** `PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0`.
  DEBUG forcing samples: add `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`.

## NEXT ACTIONS (sequence)

1. `.8.3.1` cont.: implement a coverage-reach fix (direction 1/2/3 above) so the cascade NETS an
   improvement (`UNKNOWN` drops below 56 — `kw_intersect`/`kw_within` witness AND the 9 de-witnessed rules
   re-witness); decisive A/B + global cert seeds 0/7/42 `spf=0`; 6 fully-certified grammars byte-identical;
   clippy clean. Then commit `.8.3.1` (the FIRST committable, improving slice).
2. `.8.3.2`: the property layer cascade (both `property_expr_sv_2017`/`_sv_2023` profiles) per the `.8.2`
   blueprint — DEEPER than the sequence layer, will hit the SAME coverage-reach class, so the `.8.3.1`
   coverage fix should generalize. Witnesses the `until`-family keywords. Re-confirm operand types vs
   Annex A.2.10 (`throughout` LHS = `expression_or_dist`; implication LHS = `sequence_expr`, RHS =
   `property_expr`).
3. Wave lockstep: reconcile `docs/book/src/developer-architecture.md` (overclaims direct-LR auto-elim,
   per `.8.1`); EBNF meta-grammar lockstep check (`ebnf.ebnf` — the cascade adds no new EBNF *construct*,
   only new rules using existing forms, so likely no change — verify); schema/release/ledger ceremony per
   the final AST-shape outcome (the working forms are shape-preserved via `-> $1`; the new infix shapes
   were never realized pre-fix, so a schema bump may be avoidable — confirm via the shape-contract gate).

## VERIFICATION (this slice — `PGEN-GRAMMAR-WELLFORMED-0120`)

- PURE-DOCS checkpoint: no committed code/grammar/generated/release/schema/ledger change ⇒ no clippy, no
  gate run. SV committed-baseline stays `UNKNOWN=56`; `systemverilog` LIVE row unchanged (`Mostly Done`).
- All cert/parse numbers above are tool-produced this session (seed 0; `--max-depth` 24/32/40 sweep;
  `DEBUG_PROBES` forcing dump).

---

## 2026-06-24 RE-MEASURE on TODAY's witness machinery (`PGEN-GRAMMAR-WELLFORMED-0127`, pure-docs) — the stale `56→65` is SUPERSEDED; the residual collapsed to ONE rule

**Context.** This leaf's `56→65` regression was measured `2026-06-22` (the `-0120` checkpoint), BEFORE
~10 commits of new witness-pass machinery landed (STORE-AWARE-GEN carrier-diversification `.4b.12`,
off-path-sibling `.4b.10`, target-own-structure passes, etc.). Per [[feedback_no_codebase_change_without_tool_backed_facts]]
I re-applied the EXACT validated cascade (the recovery copy above) onto the CURRENT committed baseline
(`UNKNOWN=28`, not the `-0120`-era `56`), regenerated, and re-measured. **The regression does NOT
reproduce.**

**TOOL-BACKED FINDINGS (seed 0, count 40, `--grammar-profile sv_2017 --entry-rule systemverilog_file`):**
- Committed flat baseline: `total=1288 proof=1 witness=1259 UNKNOWN=28 spf=0`.
- Cascade re-applied: `total=1299 proof=1 witness=1271 UNKNOWN=27 spf=0` — a **NET IMPROVEMENT (28→27)**,
  NOT a regression. Set diff vs baseline: **newly WITNESSED** = `kw_intersect_6c96caaf`, `kw_within_f42ef621`
  (the 2 SVA sequence-layer infix operators — the cascade's whole point ✅); **newly DE-WITNESSED** =
  `kw_first_match_2ddb8d09` ONLY (1 rule). The 9 rules the `-0120` analysis worried about
  (`boolean_abbrev`/`consecutive_repetition`/`sequence_abbrev`/… reachable via `seq_unary`) are now
  handled by the newer passes (the cert reports `carrier-diversification reach pass: … 10 witnessed`,
  `target-own-structure reach pass: … 2 witnessed`).
- Lint clean on the cascade grammar (`unreachable_rules=0 non_terminating=0 ordered_choice_shadowing=0
  profile_orphans=0 unbound_fact_kinds=0`; `always_matches_shadowing=8` unchanged A2 backlog).

**ROOT CAUSE of the sole residual `kw_first_match` (tools-first, decisive):**
- `--generate-stimuli --entry-rule seq_unary --count 60` → **200** `first_match` renders (the branch
  weighting is fine); `--generate-stimuli --entry-rule sequence_expr --count 60` → **0** `first_match`.
  So it is purely a **cascade-depth reach** gap, not branch weighting.
- `PGEN_REACH_PATH_DUMP=1`: the plan for `kw_first_match` descends the cascade via the OPERATOR branches
  (`seq_or_expr root/o0/s0` … `seq_within_expr root/o0/s0`) and reaches `seq_unary` via `seq_delay_expr`'s
  `delay_head` branch (`root/o0/s1` = the `seq_unary` AFTER `cycle_delay_range`), so the forced render is
  the `##…within##…intersect##…and##…or` operator cascade and the first_match branch (`seq_unary root/o3`)
  never renders cleanly. `DEBUG_PROBES` sample: `…;##4096_.10e394within##\foo_0 intersect##\foo_0 and##\foo_0 or##\foo_0…` (`parsed=false`).
- WHERE: `reach_hops_pass` (`rust/src/ast_pipeline/stimuli_generator.rs:6771`) is a plain BFS where the
  FIRST-enumerated reference site wins (`collect_rule_reference_sites` emits `o0` before `o1`/`o2`), so the
  operator branch (`o0`) is chosen over the sole-element passthrough (`seq_*_expr := … -> $1`, `o1`; and
  `seq_delay_expr := … | seq_unary -> $1`, `o2`).
- Direction-3 (declarative `@probe_sample` on the first_match branch) was TRIED and is **inert**: the
  branch is never selected in diverse generation (0/60), and the plannable pass FORCES the branch (so the
  branch-level hint stands down per the `H.12.3` `reach_forces_this_branch` guard at `:8677`) yet the
  forced upstream path still renders the operator cascade.

**THE FIX (pinned → new sub-leaf `H.12.5.8.3.1.1`, ENGINE, generator-only):** direction-1 — a
minimal-pollution reach-site preference in `reach_hops_pass`, mirroring `prefer_non_self_recursive_reference_sites`
(`:7165`): when several reference sites in a rule reach the same target, prefer the **sole-element
passthrough** site (the `-> $1` alternative whose containing alternative renders no mandatory siblings)
over an operator-form site (reference + mandatory tail). This routes the cascade descent through the clean
passthroughs so `seq_unary`'s primary branches (incl. first_match) render un-polluted. Expected: SV cert
`28→26` (intersect+within witnessed, kw_first_match re-witnessed, zero newly-UNKNOWN). It is generator-only
(no parser-accepted-language change) and is LIKELY INERT for the 6 fully-certified grammars (none has a
deep precedence cascade with passthrough-vs-operator site ambiguity) — verify byte-identical, seeds 0/7/42.
It **also closes the deeper `.8.3.2` property-layer** until-family (`kw_until`/`kw_s_until`/`kw_until_with`/`kw_s_until_with`),
which has the same cascade shape.

**SEQUENCING (why this is a checkpoint, not a land).** The cascade is a *consumer-visible released-SV-parser*
change (widens accepted language: `a and b`/`a or b`/`a intersect b`/`a within b` now parse), so it must
land with the full same-commit ceremony (release bump, schema decision, ledger row, contract + SV book +
top-level book, 6-grammar byte-identical, external corpus 14/14, `ast_shape_contract`, cross-family,
clippy) AND the direction-1 engine change (cross-grammar verified). That is a dedicated, fresh-budget
effort. Per [[feedback_always_signoff_decisions]] (checkpoint rather than start a released-parser commit I
might not complete cleanly), this slice REVERTS the working tree to the clean `UNKNOWN=28` baseline and
captures the now-turnkey plan. The validated cascade text is the recovery copy above; the engine fix is
designed above.

**RESUME STATE (supersedes the `-0120` "RESUME STATE" block above):** committed baseline is the FLAT
`sequence_expr`, SV cert `UNKNOWN=28`, `systemverilog` row `Mostly Done` — UNCHANGED by this slice
(pure-docs; working tree reverted + SV parser regenerated back to baseline). Next leaf `H.12.5.8.3.1.1` =
implement the direction-1 reach-site preference, re-apply the cascade, verify `28→26` + zero newly-UNKNOWN
+ 6-grammar byte-identical, then the release ceremony; then `.8.3.2` (property layer).

### Acceptance Checklist (this pure-docs checkpoint — no code lands)
- [x] **ROOT CAUSE (WHY + WHERE)** — `reach_hops_pass` BFS first-site-wins picks the operator branch over the passthrough → cascade-depth reach pollutes `kw_first_match`'s sample (`REACH_PATH_DUMP` + `DEBUG_PROBES` + the `seq_unary` 200/60 vs `sequence_expr` 0/60 generation A/B, all pasted above).
- [x] **ADDRESSED (verified)** — N/A (no code lands; the cascade was re-applied + measured `28→27` then REVERTED per "checkpoint, don't half-land a released change"). Baseline restored: working tree reverted (`git checkout grammars/systemverilog.ebnf`) + SV parser regenerated; cert re-confirms `UNKNOWN=28`.
- [x] **NO REGRESSION** — pure-docs; committed baseline byte-identical (`UNKNOWN=28`, the 6 fully-certified grammars untouched). No tracked Rust/grammar/generated change.
- [x] **LOCKSTEP** — this leaf, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`. No book/contract/ledger/release change (no user-facing behavior change this slice).

---

## `H.12.5.8.3.1.1` — IMPLEMENT: reach-site passthrough preference + LAND the sequence cascade (released-SV ceremony) — `PGEN-GRAMMAR-WELLFORMED-0132`, 2026-06-25

**Status: `done` — LANDED.** The turnkey plan from the `-0127` re-measure executed: the direction-1
engine change (reach-site passthrough preference) + the validated sequence cascade landed together as
one released-SV slice. SV cert `UNKNOWN 28 → 26` (the 2 SVA sequence-layer infix operators
`kw_intersect`/`kw_within` now witnessed), deterministic seeds 0/7/42, zero newly-UNKNOWN; the 6
fully-certified grammars byte-identical (the engine change is inert for them). Release `1.0.147 →
1.0.148`, schema STAYS `6` (strictly-more-permissive — only previously-REJECTED infix forms gain new
shapes; every previously-accepted form keeps its carrier via the cascade's `-> $1` passthroughs),
ledger `SV-0010`.

**THE ENGINE CHANGE (generator-only, parser-agnostic — `rust/src/ast_pipeline/stimuli_generator.rs`).**
New `prefer_sole_reference_passthrough_sites` + `count_mandatory_yield_atoms`, called inside
`reach_hops_pass` immediately BEFORE the existing `prefer_non_self_recursive_reference_sites`. When
several reference sites in a rule reach the same target, it stably sorts a SOLE-MANDATORY-REFERENCE
passthrough alternative (the `-> $1` form, mandatory-yield count `== 1`) ahead of an operator-form
alternative (`inner op_tail op_tail*`, count `> 1`). Applied before the self-recursive preference so —
by stable-sort composition — self-recursion stays the PRIMARY key (a self-recursive passthrough never
beats a non-self-recursive operator branch, e.g. `R := R | a b`) and passthrough is the tie-breaker.
`count_mandatory_yield_atoms` counts the minimum guaranteed terminal/reference yield (optional `?`/`*`
quantifiers and lookaheads contribute 0; a nested `Or` contributes the min across its branches).

**THE GRAMMAR CHANGE (`grammars/systemverilog.ebnf`).** Replaced the flat 12-branch directly-left-
recursive `sequence_expr` with the IEEE 1800-2017 §16 (Table 16-3) precedence cascade
(loosest→tightest `or > and > intersect > within > throughout > ## > unary`): `sequence_expr := seq_or_expr -> $1`
over 11 new rules (`seq_or_expr`/`seq_or_tail`/`seq_and_expr`/`seq_and_tail`/`seq_intersect_expr`/
`seq_intersect_tail`/`seq_within_expr`/`seq_within_tail`/`seq_throughout_expr`/`seq_delay_expr`/
`seq_unary`), each operator layer a `head tail tail*` chain with a named tail rule, `-> $1` passthrough
on bare operands. The validated recovery copy from the `-0127` checkpoint (above).

### Acceptance Checklist (enforced) — H.12.5.8.3.1.1
- [x] **REPRODUCE / ISSUE** — baseline `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0` ⇒ `CERTIFICATE-COVERAGE: … total=1288 witness=1259 UNKNOWN=28 spf=0`; the UNKNOWN list contains `kw_intersect_6c96caaf`, `kw_within_f42ef621` (the LR sequence-layer infix operators never reached). `parseability_probe --parse systemverilog --profile sv_2017`: `a and b`/`a or b`/`a intersect b`/`a within b` REJECT at the operator (pre-fix).
- [x] **ROOT CAUSE (WHY + WHERE)** — two mechanisms: (1) PARSE — `sequence_expr`/`property_expr` are directly-left-recursive (`A := A op A`), which PGEN's eliminator (indirect wrapper-chain only) does not eliminate, so the runtime cycle-breaker blocks every infix branch (`-0118`). (2) WITNESS-REACH — even with the cascade, `reach_hops_pass` (`stimuli_generator.rs:6771`) BFS first-site-wins discovers the inner cascade layer via the OPERATOR branch (`o0`), so the forced descent renders a `## … within … intersect …` operator soup that pollutes the deep operand; `PGEN_REACH_PATH_DUMP=1` + `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` (`witnessed_target=false`, `parsed=false` forced samples) pinned it (`-0127`).
- [x] **FIX** — tier: grammar (precedence cascade, direction A — director-decided `-0119`) + generator engine (reach-site passthrough preference). Both parser-agnostic; the engine half is a general reach-honesty preference mirroring `prefer_non_self_recursive_reference_sites`.
- [x] **ADDRESSED (verified)** — SV cert `UNKNOWN 28 → 26` (`CERTIFICATE-COVERAGE: … total=1299 proof=1 witness=1272 UNKNOWN=26 spf=0`), **deterministic seeds 0/7/42** (all = 26); set-diff vs baseline = newly WITNESSED `{kw_intersect_6c96caaf, kw_within_f42ef621}`, newly-UNKNOWN = ∅. `parseability_probe`: `a and b`/`a or b`/`a intersect b`/`a within b`/`a ##1 b`/`a ##1 b ##2 c`/`a or b and c`/`a intersect b within c` REJECT→PASS; previously-working `first_match(a)`/`a [*2]`/`x throughout a` still PASS.
- [x] **NO REGRESSION** — cert seeds **0/7/42** `spf=0` (`sample_parse_failures=0`); the **6 fully-certified grammars BYTE-IDENTICAL** (decisive A/B, new binary vs baseline: json `9/9`, regex `198/198`, vhdl `216/216`, svpp `74/74`, rtl_frontend `169` proof1/witness168, rtl_const_expr `48/48` — all `fully_certified=true`, the engine change inert); **SV external corpus 14/14** (`sv_external_corpus_triage_gate`, `parse_fail_total=0`); `ast_shape_contract` GREEN (18/18; no SV sample exercises `sequence_expr`); `--lint-grammar` clean (`non_terminating=0`, `unreachable_rules=0`, `ordered_choice_shadowing=0`, pre-existing `always_matches=8` A2 backlog unchanged; `1415` rules, +11 cascade rules); clippy source-clean.
- [x] **LOCKSTEP** — release `1.0.147 → 1.0.148`, schema stays `6`, ledger `SV-0010`; contract `PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`; SV book `changelog-index.md` + `json-carrier.md` (new sequence cascade shapes); `PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SV-0010`); shape-contract manifest calibration; `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md` / `LIVE_ACHIEVEMENT_STATUS.md`. The `developer-architecture.md` direct-LR claim is already correct (no edit). `[[feedback_ebnf_meta_grammar_lockstep]]`: the cascade adds no new EBNF *construct* (only new rules using existing forms), so `ebnf.ebnf` needs no change.

**NEXT (frontier):** `.8.3.2` — the PROPERTY-layer cascade (`property_expr_sv_2017`/`property_expr_sv_2023`), which has the SAME cascade shape and will reuse this engine fix to witness the `until`-family (`kw_until`/`kw_s_until`/`kw_until_with`/`kw_s_until_with`, currently 4 of the residual 26 UNKNOWN).
