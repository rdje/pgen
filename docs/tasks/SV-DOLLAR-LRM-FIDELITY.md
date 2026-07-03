# SV-DOLLAR-LRM-FIDELITY: restore LRM `$`-spelling fidelity for the mangled `sv_dollar_*` keyword-token family

## Metadata

- Tree ID: `SV-DOLLAR-LRM-FIDELITY`
- Status: `complete` (2026-07-03 — all 4 leaves done; `SV-0029` + `SV-0030` `Released` at
  releases `1.0.159`–`1.0.161`)
- Roadmap lane: SystemVerilog parser-family LRM fidelity (released-parser correctness defect
  family; sibling of `SV-COVERGROUP-FIDELITY` / `SV-SVA-PROPERTY-FIDELITY` /
  `SV-AST-SHAPE-FIDELITY`)
- Created: `2026-07-02`
- Last updated: `2026-07-03`
- Owner: repo-local workflow
- Origin: discovered tools-first during `VERILOG-2005-PROFILE.6.4`
  (`PGEN-VERILOG-2005-PROFILE-0017`) — see that tree's "`.6.4` Findings" for the full
  evidence record.
- Ledger rows: `SV-0029` (the 19-token mangled-literal family), `SV-0030` (`scalar_constant`
  digit loss + `scalar_timing_check_condition` PEG shadowing).

## Goal

Make the flattened `grammars/systemverilog.ebnf` accept the IEEE LRM `$` spellings (and stop
accepting the mangled nonsense spellings) for every keyword-like `$` token, with correct
per-profile gating:

1. The **19 `kw_sv_dollar_*` tokens** (`grammars/systemverilog.ebnf:6248-6284`) currently match
   the literal TEXT `sv_dollar_*` (e.g. `kw_sv_dollar_setup_b58bdaae := trivia
   /sv_dollar_setup\b/`) instead of the LRM `$*` spellings. Proven consequences (probes recorded
   in `VERILOG-2005-PROFILE` "`.6.4` Findings", 2026-07-02, release `1.0.158`):
   - LRM `$setup(d, posedge clk, 1);` in a specify block REJECTS under `sv_2017`
     (`furthest_position=26`) AND under `verilog_2005` — although specify timing checks are
     CORE surface in BOTH IEEE 1800-2017 §31 and IEEE 1364-2005 §15.
   - LRM `$unit::y` REJECTS under `sv_2017` (IEEE 1800 §3.12.1); nonsense `sv_dollar_unit::y`
     ACCEPTS.
   - LRM module-level `$fatal;` (IEEE 1800 §20.11 elaboration severity task) REJECTS;
     nonsense `sv_dollar_fatal;` ACCEPTS.
   - `$root.top.f(1);` ACCEPTS but MIS-ROUTED: the AST carries `kind:"system_tf"` only —
     `$root` is consumed by the generic `system_tf_identifier` token (`:449`), never the
     `$root`-anchored `hierarchical_identifier` (`:2356`) / `hierarchical_tf_identifier`
     (`:2372`) forms, so downstream consumers never see the LRM `$root` anchor.
   - Nonsense `sv_dollar_setup(…)` ACCEPTS (over-acceptance of non-language text).
2. The **`scalar_constant` digit loss** (`SV-0030`): IEEE `scalar_constant ::= 1'b0 | 1'b1 |
   1'B0 | 1'B1 | 'b0 | 'b1 | 'B0 | 'B1 | 1 | 0` (faithful in
   `grammars/systemverilog_2017_lrm_extracted.ebnf:951`) was flattened into digit-less prefixes
   (`kw_n_1_tick_b_f4c81681 := trivia "1'b"`, `:6068`; rule `:4713`). Additionally
   `scalar_timing_check_condition` (`:4720`) lists bare `expression` FIRST, PEG-shadowing the
   `expression equal scalar_constant` branches (probes: `e == 1'b0` ACCEPTS via the bare
   expression branch; defective `e == 1'b` REJECTS at the outer sequence).

Provenance (WHY+WHERE of the mangling): the extracted LRM snapshots carry the true `$`
spellings (zero `sv_dollar` matches in both `systemverilog_2017_lrm_extracted.ebnf` and
`verilog_2005_lrm_extracted.ebnf`); the mangling entered via the profiled-synthesis rule-name
canonicalization (`tools/extract_systemverilog_lrm_profiles.py:315`, `$name` → `sv_dollar_name`
— correct for RULE NAMES, leaked into keyword-token LITERALS). No pre-parse rewrite exists
(`grep -rn "sv_dollar" rust/src/` → zero hits), so the defect is live in released `1.0.158`.

## Non-Goals

- No re-run of the LRM extraction/synthesis pipeline: the flattened
  `grammars/systemverilog.ebnf` is the actively-maintained artifact (census 1450 with many
  post-synthesis hand-landed fixes); the fix lands there directly. The synthesis tool's
  name-vs-literal confusion is recorded for provenance; fixing the *tool* is only worth doing
  if the profiles are ever re-synthesized (tracked as an open question, not a leaf).
- No behavioral change to the generic `system_tf_identifier` surface (`$display`, `$random`, …)
  — that token (`/\$[a-zA-Z0-9_$]+/`) is correct and is how the external corpus parses today.
- Witness/cert ratcheting beyond what the fixes naturally earn (owned by the per-profile trees).

## Profile map (from the `.6.4` adjudication — drives the gating design)

| Token group | LRM home | `verilog_2005` | `sv_2017`/`sv_2023` |
| --- | --- | --- | --- |
| 12 timing checks (`$setup`,`$hold`,`$setuphold`,`$recovery`,`$recrem`,`$removal`,`$skew`,`$timeskew`,`$fullskew`,`$period`,`$width`,`$nochange`) | 1364-2005 §15 + 1800 §31 | **in-profile** | in-profile |
| `$root` (`:2356` optional prefix, `:2372` mandatory) | 1800 only | must REJECT | in-profile |
| `$unit` (`:3762`) | 1800 only | must REJECT | in-profile |
| `$fatal`/`$error`/`$warning`/`$info` elaboration tasks (`:2019+`, `:4871+`) | 1800 only (module-level elaboration form) | must REJECT | in-profile |
| bare `$` primary (`kw_sv_dollar_04da59ec`, `:2976`/`:2993`/`:4026`/`:4081`) | 1800 (`$` as unbounded literal) | adjudicate in `.1` | in-profile |

## Acceptance Criteria

- Every affected LRM spelling ACCEPTS in its correct profiles and every mangled nonsense
  spelling REJECTS, proven by a per-token probe matrix (accept + reject, both directions).
- `scalar_constant` accepts exactly the 10 LRM alternatives, and the
  `scalar_timing_check_condition` eq/case_eq/ne/case_ne branches are reachable (shadowing
  removed) — with the AST-shape/schema impact adjudicated and, if shapes change, the release/
  schema/ledger/book lockstep performed.
- No regression: canonical cert (seeds 0/7/42, `spf=0`) with residual-set drift adjudicated
  (witness-count drift is EXPECTED — new tokens become witnessable; every delta must be
  set-diff-proven honest), union gate + `verilog_2005` conformance gate re-pinned in the same
  commit as any drift, `ast_shape_contract` green, external corpus 14/14, the 6 fully-certified
  grammars byte-identical, clippy source clean.
- Ledger `SV-0029`/`SV-0030` → `Fixed`/`Released`; SV integration contract + SV parser book +
  top-level book lockstep.
- Each leaf passes the `TOOLBOX.md` acceptance checklist and commits per `COMMIT.md`.

## Task Tree

- ID: `SV-DOLLAR-LRM-FIDELITY`
  Status: `complete`
  Goal: restore LRM `$`-spelling fidelity for the mangled token family (SV-0029) + the
  `scalar_constant` digit loss (SV-0030), with correct per-profile gating.
  Children: `.1`, `.2`, `.3`, `.4`

- ID: `SV-DOLLAR-LRM-FIDELITY.1` — Status: `done` (2026-07-02, session #21,
  `PGEN-SV-DOLLAR-LRM-FIDELITY-0001`, ZERO code — DESIGN/AUDIT leaf, tools-first): the full
  per-token × per-profile probe matrix, the collision audit, the shape/schema adjudication, the
  gating map, and the wave plan are all recorded (see "`.1` Findings"). Headlines: all 12
  timing checks are literal-only defects (LRM REJ / mangled ACC, 24/24 probes, both dialects;
  rule bodies faithful to IEEE 1800-2017 A.7.5.1 incl. `$width`'s mandatory `threshold`);
  `specify_item` has NO generic system-TF alternative → wave 1 is collision-free; `$unit` +
  the 4 severity tasks are LRM-REJ/mangled-ACC on all 3 profiles, and the mangled severity
  spellings ACCEPT under `verilog_2005` TODAY (their host rides the v2005-admitted
  `module_common_item_sv_2017`) → wave-3 gates required; `$root` accepts in BOTH spellings but
  the LRM spelling mis-routes (`kind:"system_tf"`) — wave 3 is PEG-ordered-choice-commit
  collision work; **`SV-0030` has a SECOND site**: `init_val` (`:2429`) lost the same digits —
  UDP `initial q = 1'b0;` (LRM) REJECTS while `initial q = 1'b;` ACCEPTS (x-forms `1'bx` etc.
  intact). Token-naming decision: literals change, names stay (set-comparability of pinned
  residual lists; the sha1-of-literal hash suffix mismatch is accepted cosmetic debt).

- ID: `SV-DOLLAR-LRM-FIDELITY.2` — Status: `done` (2026-07-02, session #21,
  `PGEN-SV-DOLLAR-LRM-FIDELITY-0002`, CODE — grammar-only, SV release `1.0.158`→`1.0.159`,
  schema `13` unchanged): wave 1 LANDED — the 12 specify timing-check token literals corrected
  `sv_dollar_X` → `$X` (`grammars/systemverilog.ebnf:6254-6284`, names kept per the `.1`
  decision; provenance comment added at the token block). VERIFIED (see "Acceptance Checklist
  (`.2`)"): 12/12 LRM spellings REJECT→**ACCEPT** and 12/12 mangled spellings
  ACCEPT→**REJECT** under BOTH `sv_2017` and `verilog_2005`; 3 procedural collision controls
  unchanged; all 8 wave-3 token probes byte-unchanged (wave isolation). NO-REGRESSION earned
  fresh with pins EXACT (no re-pin needed anywhere): canonical cert `1328/2/1306/UNKNOWN=20
  spf=0` seeds 0/7/42 with the 20-rule residual SET-IDENTICAL to the pre-fix union-gate log;
  union gate GREEN (canonical 20 / union 1 / residual `context_member_method_call`);
  `verilog_2005` conformance gate GREEN — 168 checks/0 mismatches (56 cases incl. the 2 new
  wave-1 locks `accept/specify_timing_checks.v` + `reject/specify_timing_check_mangled.v`),
  lint 0 orphans, cert `1138/2/809/327` deterministic (count-neutral witness re-route, as
  designed); shape 18/18; external corpus green; clippy source strict-clean; the 6
  fully-certified grammars byte-identical by construction (SV-only regen). Ledger `SV-0029` →
  `Fix In Progress` (12 of 19 tokens fixed; wave 3 = `$root`/`$unit`/severity/bare-`$`).

- ID: `SV-DOLLAR-LRM-FIDELITY.3` — Status: `done` (2026-07-02/03, session #22,
  `PGEN-SV-DOLLAR-LRM-FIDELITY-0003`, CODE — grammar + shape-test dispatch arms, SV release
  `1.0.159`→`1.0.160`, schema `13`→`14`): wave 2 LANDED, `SV-0030` CLOSED at BOTH sites.
  **Design discovery (tools-first):** the `.1`-sketched "reorder the eq branches first" is
  INSUFFICIENT alone — the compare branches' lhs is the full `expression`, whose PEG-greedy
  operand chain (`expression_operand ( binary_operator … )*`, `binary_operator` includes
  `equal`) consumes `== 1'b0` ITSELF, so the branch can never match regardless of order
  (AST-dump-proven: `e == 1'b0` parsed with ZERO `kind:"eq"` nodes). The landed design:
  (a) the ten IEEE digit alternatives restored at `scalar_constant` + `init_val` via 8 new
  `\b`-guarded sha1-named tokens (`kw_n_1_tick_b0_cc597d72` … `kw_tick_B1_230dd583`; the
  digit-less `1'b`/`1'B` tokens removed with their last sites; the `'b`/`'B` forms — formerly
  `tick kw_b` — now whole `\b`-guarded tokens; `kw_b`/`kw_B` stay for `level_symbol`);
  (b) `!tick` follow-guards on scalar_constant's bare `1`/`0` branches (else `e == 1'bx` /
  `e == 1'b01` would commit a partial rhs and flip ACCEPT→REJECT);
  (c) the compare branches ordered FIRST with a NEW precedence-restricted lhs
  (`scalar_timing_check_compare_lhs` → `scalar_timing_check_compare_chain` whose operator
  tier `binary_operator_above_equality` = only IEEE 1800-2017 Table 11-2 rows 3–7, i.e. the
  operators binding TIGHTER than equality) + a trailing `!binary_operator` follow-guard per
  branch — so `e == 1'b0` emits `{kind:"eq", lhs, rhs}` with an expression-shaped lhs, while
  `e == 1'b0 && f` (guard) and `a & b == 1'b0` (`==` binds tighter than `&`; lhs chain stops
  at `&`) keep the precedence-correct flat parse BYTE-IDENTICALLY;
  (d) the `tilde expression` branch deliberately stays AFTER `expression` (adjudicated:
  promoting it would mis-associate `~e && f` as `~(e && f)` — the flat parse is the
  precedence-correct AST; the LRM language is unaffected since `~expr ⊆ expression`).
  Verified per the enforced checklist below; ledger `SV-0030` → `Released`.

- ID: `SV-DOLLAR-LRM-FIDELITY.4` — Status: `done` (2026-07-03, session #23,
  `PGEN-SV-DOLLAR-LRM-FIDELITY-0004`, CODE — grammar + shape-test dispatch, SV release
  `1.0.160`→`1.0.161`, schema `14`→`15`): wave 3 LANDED, `SV-0029` CLOSED in full. The 7
  remaining token literals carry their IEEE spellings (`$root`/`$unit`/`$fatal`/`$error`/
  `$warning`/`$info`/bare-`$`); `system_tf_call`'s optional-parens alternative carries the
  `!( $root . ) !( $unit :: )` steal guards (annotation re-based `$3/$4` — a lookahead
  OCCUPIES a `$N` slot, codegen-proven); three profile-gated lifts
  (`hierarchical_root_prefix_sv_only` → `root: {kind:"root"}` marker,
  `package_scope_dollar_unit_sv_only` → `{kind:"dollar_unit", name:{body:"$unit"}}` with the
  load-bearing resolvable name, `primary_dollar_sv_only`) + the `!( identifier )`-guarded
  `rooted_tf_call_sv_only` carrier in `subroutine_call`/`call_primary` (routes AROUND the
  runtime-broadcast branch predicate that hard-rejects rooted results inside
  `scoped_or_hierarchical_tf_identifier` — see Open Questions). Census 1459→1463. TWO `.1`
  hypotheses corrected tools-first (severity "v2005 leak" = identifier reading, dump-proven;
  pure literal fix INERT for `$unit`/`$root`-tf without the predicate-resolvability +
  carrier fixes, trace-proven). Verified per the enforced checklist below; ledger `SV-0029`
  → `Released`.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (tree complete) | — | all 4 leaves done; `SV-0029` + `SV-0030` both `Released` (releases `1.0.159`–`1.0.161`); tree ready to close |

## Acceptance Checklist (`.4`, enforced)

- [x] **REPRODUCE / ISSUE** — fresh HEAD (release `1.0.160`) 16-probe matrix ×3 profiles:
  `$unit::y` REJ×3 / `sv_dollar_unit::y` ACC×3; module-level `$fatal;`/`$error;`/`$warning;`/
  `$info;` REJ×3 vs mangled ACC×3; `x = $;` REJ×3 vs `x = sv_dollar;` ACC×3; `$root.m.y` +
  `$root.top.f(1);` ACC×3 but MIS-ROUTED (`kind:"system_tf"` chains, dump-proven —
  `root_expr_lrm`/`root_tf_lrm` pre-dumps). Control: bare `foo;` ACC×3 (the implicit-var-decl
  surface) — the AST dump proved mangled `sv_dollar_fatal;` rides THAT route (chain
  `variable > implicit > variable_decl > data_declaration`), disproving the `.1` severity-host
  leak hypothesis. Ledger `SV-0029`.
- [x] **ROOT CAUSE (WHY + WHERE)** — four tool-named mechanisms: (1) the 7 token literals
  matched mangled rule-name text (`grammars/systemverilog.ebnf:6312-6352`, e.g.
  `kw_sv_dollar_root_f65f0e67 := trivia /sv_dollar_root\b/`); (2) the generic
  `system_tf_identifier` (`/\$[a-zA-Z0-9_$]+/`, `:471`) matches `$root`/`$unit` as whole
  tokens and `system_tf_call`'s optional-parens alternative SUCCEEDS on the bare identifier —
  a PEG commit that steals the anchor (`root_expr_lrm` pre-dump: `system_tf >
  call_postfix_chain`); (3) `non_typedef_package_scope`'s post predicate + the
  `scoped_or_hierarchical_tf_identifier` branch predicate drill `$….name.body`, and an
  UNRESOLVABLE ref is a hard reject — trace: `$unit` token matched 22→28 then the route died
  (`furthest_position=30`), entry-rule isolation pinned `non_typedef_package_scope` REJ on
  `$unit::` while `package_scope` ACC; (4) the predicate runtime BROADCASTS per-branch
  directives rule-wide (`branch_predicates_for_rule` flat-maps —
  `rust/src/ast_pipeline/semantic_runtime.rs:735`): trace `🚫 Branch 3/3 for rule
  'scoped_or_hierarchical_tf_identifier' rejected by branch predicate
  'lacks_fact_attribute_equals [… scope.name.body …]'` fired on the SUCCEEDED rooted branch
  (`hierarchical_tf_identifier` selected branch 1 consuming 12 chars) — the compiled
  registration is correctly per-branch (`[[], [pred], []]`), so the flat-map is the defect
  (ledgered as an open engine question; adjudicated OUT of this leaf).
- [x] **FIX** — GRAMMAR tier + shape-test dispatch (no engine change):
  `grammars/systemverilog.ebnf` — 7 literals fixed; steal guards on `system_tf_call` alt 1
  (annotation `$1/$2`→`$3/$4`; a lookahead occupies a slot, proven on a minimal generated
  parser); lifts `hierarchical_root_prefix_sv_only` (+ `hierarchical_identifier`/
  `hierarchical_tf_identifier` rewires, `$3/$4`→`$2/$3` re-base),
  `package_scope_dollar_unit_sv_only` (load-bearing `name: {body: "$unit"}` — nested-object
  literal codegen-verified), `primary_dollar_sv_only`, `rooted_tf_call_sv_only`
  (`!( identifier )` no-re-route firewall; wired into `subroutine_call` + `call_primary`);
  `rust/src/ast_shape_contract.rs` — 6 new dispatch arms. Census 1450→1459→1463 (net +4 this
  leaf); regen + both binaries rebuilt.
- [x] **ADDRESSED (verified)** — 28-probe AFTER matrix: every LRM spelling ACC under
  `sv_2017`+`sv_2023` and REJ under `verilog_2005` (`unit_lrm`, `unit_tf_lrm`,
  `sev_{fatal,error,warning,info}_lrm` + args forms, `root_expr_lrm`, `root_tf_lrm`,
  `root_call_expr_lrm`, `bare_lrm`); anchor markers AST-dump-proven (`dollar_unit`, `root`,
  `rooted_tf`+`rooted`, `fatal`, `system_dollar`) with ZERO `kind:"system_tf"` residue;
  24/24 mangled+control dumps byte-identical pre↔post (cmp, both dialects) incl.
  `$root(1);`/`$rootabc(1);`/`$unit`-bare/`a.b.c`/`top.f(1);`/`pkg::y`/`foo;` controls.
- [x] **NO REGRESSION** — canonical cert `CERTIFICATE-COVERAGE: … total=1341 proof=2
  witness=1319 UNKNOWN=20 (spf=0, prf=0)` seeds 0/7/42, 20-rule residual SET-IDENTICAL
  pre↔post (python set-compare per seed: `True`; +4 total = exactly the census delta, all 4
  new rules witnessed); `verilog_2005` cert `1147/4/816/327` deterministic seeds 0/7/42
  (residual set-diff: sole add `kw_sv_dollar_04da59ec` — NO-reach-by-design 294→295;
  `kw_sv_dollar_root`/`kw_sv_dollar_unit` upgrade witness→PROOF, the honest direction);
  `verilog_2005_conformance_gate` GREEN (`gate_green: true`, 192 checks/0 mismatches, 64
  cases incl. 4 new locks, lint orphans=0, cert pins earned ×3 seeds);
  `sv_cert_recognized_union_gate` GREEN re-pinned (`1341/2/1319/20`, union `1338/1`,
  residual `context_member_method_call`, `unmet_criteria_json: []`, seeds 0/7/42);
  `ast_shape_contract_gate` green (18 passed/0 failed; 31 SV samples incl. 6 new);
  `sv_external_corpus_triage_gate` rc 0 — 10/10 non-uvm rows pass, the 4 uvm rows are the
  documented 24 GB-host mem-cap posture (uvm_pkg/uvm_compat_pkg × 2 profiles — same rows as
  the pre-fix state); `--lint-grammar` rc 0 (1463 rules, all error classes 0,
  `always_matches=8` pin unchanged, `profile_orphans=0`); `clippy_on_rust_change` rc 0
  (source strict-clean; generated-stage non-strict debt = 178 eq_op const-fold sites (+1
  from the new choice, same documented class) + 1 pre-existing svpp `overly_complex_bool_expr`
  — svpp parser mtime-untouched); the other 9 generated parsers mtime-untouched ⇒ the 6
  fully-certified grammars byte-identical by construction; `systemverilog_parser_book_gate` +
  `mdbook_docs_gate` rc 0.
- [x] **LOCKSTEP** — ledger `SV-0029` → `Released` (wave-3 proof + fix/release/assessment
  columns); SV integration contract → `1.0.161`/schema `15` (identity, schema-15 paragraph
  with the 14 note retained, 1.0.161 highlights section, trust-posture + known-issues
  bullets, release-stream note, and the line-62 v2005 cert-pin that had gone stale at `.3`
  corrected); SV parser book (schema-versioning row 15 + intro chain, changelog-index
  1.0.161) + book gate; top-level book `parser-families.md` (SV-0029 → FIXED-in-full
  narrative, three-open-defects roster, v2005 cert chain incl. the witness→proof upgrades);
  conformance contract +4 case rows + cert re-pin `1147/4/816/327` + NO-reach note 295;
  union contract re-pin `1341/1319/1338`; shape manifest +6 samples (+ the dollar_unit
  name-field update); tree + `docs/TASK_TREE.md`; MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE.

## Acceptance Checklist (`.3`, enforced)

- [x] **REPRODUCE / ISSUE** — fresh HEAD (release `1.0.159`) probe matrix, both `sv_2017` AND
  `verilog_2005`: UDP `initial q = 1'b0;` REJ (`furthest_position=67`) / `1'b1` REJ /
  nonsense `1'b` ACC / `1'bx` ACC / `1` ACC (site 2); `e == 1'b0` in a specify condition ACC
  but with ZERO `kind:"eq"` nodes in the AST dump (flat `kind:"expression"`; site 1 shape
  loss); `e == 1'b` REJ at the outer sequence. Entry-rule isolation
  (`--entry-rule scalar_constant`): LRM `1'b0` REJ at position 3 (the digit-less token
  consumed `1'b` and left `0`), nonsense `1'b` ACC. Ledger `SV-0030`.
- [x] **ROOT CAUSE (WHY + WHERE)** — three tool-named mechanisms: (1) digit-less prefix
  tokens `kw_n_1_tick_b_f4c81681 := trivia "1'b"` (`grammars/systemverilog.ebnf:6068`, twin
  `:6062`) referenced by `init_val:2429` + `scalar_constant:4713` — the entry-rule probe
  above pins the 3-byte partial consume; (2) `scalar_timing_check_condition:4720` listed
  bare `expression` FIRST (PEG commit); (3) the deeper starvation: the compare branches' own
  greedy `expression` lhs consumes `== 1'b0` into its operand chain (`expression_base:315`
  chain star with `equal ∈ binary_operator:256`), proven by the ops-chain engine probe
  (`a ** b`/`a << b`/`a ==? b` all ACC ⇒ group-failure choice re-entry, so ONLY a
  lhs-tier restriction can stop the chain before `==`) — a pure reorder is INERT.
- [x] **FIX** — GRAMMAR tier + the shape-test dispatch arms (no engine change):
  `grammars/systemverilog.ebnf` — the ten IEEE alternatives at both sites (8 new `\b` tokens,
  2 removed), `!tick` guards, compare-first ordering with
  `scalar_timing_check_compare_lhs`/`_chain` + `binary_operator_above_equality` (14
  higher-than-equality operators, binary_operator's relative order) + `!binary_operator`
  follow-guards; `rust/src/ast_shape_contract.rs` — 4 new `rule_under_test` dispatch arms.
  Census 1450→1459 (net +9); regen + both binaries rebuilt.
- [x] **ADDRESSED (verified)** — UDP `1'b0`/`1'b1` REJECT→**ACCEPT** + `1'b`
  ACCEPT→**REJECT** under BOTH dialects; `e == 1'b0` now emits exactly 1
  `{kind:"eq", lhs:{kind:"base", body:{kind:"operand_chain",…}}, rhs:{kind:"1'b0"}}` node
  (AST-dump-proven; `===`→`case_eq`, `!=`→`ne`, `'b0` rhs OK); entry-rule isolation FLIPPED
  (`1'b0` ACC / `1'b` REJ, `furthest_position=1`); ALL 9 fallback-guard probes
  (`e == 1'b0 && f`, `d & e == 1'b0`, `e == 2'b01`, `e == 1'b01`, `e == 1'bx`, `~e`,
  `udp 1'bx`, `udp 1`, ops-chain) **byte-identical ASTs** pre↔post (cmp).
- [x] **NO REGRESSION** — canonical cert `CERTIFICATE-COVERAGE: … total=1337 proof=2
  witness=1315 UNKNOWN=20 (spf=0, prf=0)` seeds 0/7/42 with the 20-rule residual
  SET-IDENTICAL pre↔post (python set-compare per seed: `pre==post: True`; total +9 = exactly
  the census delta, witness +9 = every new rule/token witnessed); `verilog_2005` cert
  `1147/2/819/326` — the sole residual change is `scalar_constant` LEAVING the UNKNOWN set
  (set-diff: pre-only=`{scalar_constant}`, post-only=∅; NO-reach 294 set-identical — the
  `.6.4` class-D witness now genuinely earned); `verilog_2005_conformance_gate` GREEN
  (`gate_green: true`, 180 checks/0 mismatches — 60 cases incl. the 4 new SV-0030 locks —
  lint orphans=0, cert pins earned at seeds 0/7/42); `sv_cert_recognized_union_gate` GREEN
  re-pinned (`1337/2/1315/20`, union `1334/1`, residual `context_member_method_call`, seeds
  0/7/42); `ast_shape_contract_gate` 18/18 (25 SV samples incl. the 4 new);
  `sv_external_corpus_triage_gate` green (`primary_parse_failure_profile: <none>`);
  `--lint-grammar` rc 0 (1459 rules, all error classes 0, `always_matches=8` pin unchanged,
  `profile_orphans=0`); `clippy_on_rust_change` rc 0 (source strict-clean; the generated-stage
  177 `eq_op` errors are the documented pre-existing const-fold codegen debt —
  `"priority_first" == "priority_first"` sites — non-strict stage, zero hits in
  `ast_shape_contract.rs`); the other 9 generated parsers mtime-untouched ⇒ the 6
  fully-certified grammars byte-identical by construction.
- [x] **LOCKSTEP** — ledger `SV-0030` → `Released` (fix + verification record); SV
  integration contract → `1.0.160`/schema `14` (identity, schema-bump paragraph, highlights
  section, boundary bullets, release-stream note); SV parser book (`schema-versioning` row
  14 + intro chain, `changelog-index` 1.0.160 entry) + book gate; top-level book
  `parser-families.md` (corpus/cert numbers — incl. correcting the stale pre-`.2` "162
  checks" — SV-0030 → FIXED narrative, four-open-defects roster); conformance contract +4
  case rows + cert re-pin + NO-reach note 294-of-326; union contract re-pin; shape manifest
  +4 samples; tree + `docs/TASK_TREE.md`; MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE.

## Acceptance Checklist (`.2`, enforced)

- [x] **REPRODUCE / ISSUE** — the `.1` probe matrix on pre-fix HEAD: all 12 LRM timing-check
  spellings REJECT (e.g. `$setup(d, posedge clk, 1);` in specify →
  `Parser did not consume full input at position 0 [furthest_position=26, +26 bytes deeper]`
  under `sv_2017`, REJECT under `verilog_2005`) while all 12 mangled `sv_dollar_*` spellings
  ACCEPT (`parse_full passed`) — 26 probe files under the session scratchpad `sv0029_matrix/`,
  verdict table in "`.1` Findings" (a). Ledger `SV-0029`.
- [x] **ROOT CAUSE (WHY + WHERE)** — the 12 keyword tokens matched the mangled RULE-NAME text
  instead of the LRM `$` spelling: `kw_sv_dollar_setup_b58bdaae := trivia /sv_dollar_setup\b/`
  (`grammars/systemverilog.ebnf:6272`, likewise `:6254-6284` for the other 11). Provenance
  tool-named in `VERILOG-2005-PROFILE` "`.6.4` Findings": the profiled-synthesis rule-name
  canonicalization (`tools/extract_systemverilog_lrm_profiles.py:315`) leaked into token
  LITERALS; both extracted LRM snapshots are `$`-faithful (`grep -c sv_dollar` = 0); no
  pre-parse rewrite exists (`grep -rn "sv_dollar" rust/src/` = 0 hits).
- [x] **FIX** — GRAMMAR tier (fix-hierarchy: declarative/grammar, no engine change): the 12
  literals swapped to `/\$setup\b/`, `/\$hold\b/`, `/\$setuphold\b/`, `/\$recovery\b/`,
  `/\$recrem\b/`, `/\$removal\b/`, `/\$skew\b/`, `/\$timeskew\b/`, `/\$fullskew\b/`,
  `/\$period\b/`, `/\$width\b/`, `/\$nochange\b/`; token NAMES kept (`.1` decision); generated
  parser regenerated (`\\$setup\\b` present at `generated/systemverilog_parser.rs:924005`,
  rule names unchanged); both binaries rebuilt.
- [x] **ADDRESSED (verified)** — 12/12 LRM spellings REJECT→ACCEPT under `sv_2017` AND
  `verilog_2005`; 12/12 mangled spellings ACCEPT→REJECT under both; `$width` threshold-less
  control stays REJECT in both spellings (1800-faithful); procedural controls
  (`$display("x");`, `$setup(1);`, `$setup2(1);` in `initial`) ACC unchanged — zero collision;
  the 8 wave-3 probes (`$unit`/severity/`$root`/bare-`$` LRM+mangled) byte-unchanged — wave
  isolation proven. Full before→after table in the `.2` Verification Log entry.
- [x] **NO REGRESSION** — canonical cert
  `CERTIFICATE-COVERAGE: … total=1328 proof=2 witness=1306 UNKNOWN=20 fully_certified=false
  (sample_parse_failures=0, proof_reverify_failures=0)` at seeds 0/7/42 with the 20-rule
  residual SET-IDENTICAL to the pre-fix union-gate `cert_seed_0.log` (python set-compare:
  `pre==post: True`); `sv_cert_recognized_union_gate` GREEN (`unmet_criteria_json: []`,
  canonical `UNKNOWN=20`, union `UNKNOWN=1`, residual `context_member_method_call`, seeds
  0/7/42); `verilog_2005_conformance_gate` GREEN (`gate_green: true`, 168 checks/0 mismatches,
  lint orphans=0, cert `1138/2/809/327` deterministic — pins EXACT, count-neutral);
  `ast_shape_contract_gate` 18/18; `sv_external_corpus_triage_gate` green
  (`primary_parse_failure_profile: <none>`); `clippy_on_rust_change` rc 0 (source
  strict-clean; generated-stage debt pre-existing); the 6 fully-certified grammars
  byte-identical by construction (only `generated/systemverilog_parser.rs` regenerated —
  mtime audit).
- [x] **LOCKSTEP** — ledger `SV-0029` → `Fix In Progress` + wave-1 fix record; SV integration
  contract (release `1.0.159` highlights + honest-boundary + version stream); SV parser book
  (`changelog-index` + `schema-versioning` row `13 unchanged @ 1.0.159`) + book gate; top-level
  book `parser-families.md` SV-0029 narrative; conformance contract +2 case rows (same
  commit); tree + `docs/TASK_TREE.md`; LIVE/CHANGES/DEVELOPMENT_NOTES/MEMORY.

## `.1` Findings (tools-first, 2026-07-02 — the per-token audit)

All probes: `parseability_probe --parse systemverilog <file> --profile {sv_2017, sv_2023,
verilog_2005}` on HEAD binaries (release `1.0.158`), files preserved under the session
scratchpad (`sv0029_matrix/`). "LRM" = the IEEE `$` spelling; "MAN" = the mangled
`sv_dollar_*` spelling.

- **(a) The 12 timing checks — literal-only defect, 24/24 probes:** every LRM spelling REJECTS
  and every mangled spelling ACCEPTS, under BOTH `sv_2017` and `verilog_2005` (minimal legal
  instance per rule shape, e.g. `$setup(d, posedge clk, 1);`, `$width(posedge clk, 1, 0);`,
  `$nochange(posedge clk, d, 0, 0);`). The rule BODIES are faithful to IEEE 1800-2017 A.7.5.1
  — including `$width`'s MANDATORY `threshold` (verified against
  `docs/systemverilog/2017/txt/section-31-timing-checks.txt:71-72`:
  `$width ( controlled_reference_event , timing_check_limit , threshold [ , [ notifier ] ] )`;
  the threshold-less `$width(posedge clk, 1);` REJECTS in BOTH spellings — correct per 1800,
  a 1364-2005-vs-1800 arg-shape nuance recorded as an open question for the v2005 profile,
  NOT a defect).
- **(b) Wave-1 collision audit — CLEAN:** `specify_item` (`:4934`) = specparam | pulsestyle |
  showcancelled | path_declaration | system_timing_check — NO generic `system_tf_call`
  alternative, so inside `specify` the fixed `$setup` literal has no competing route. In
  procedural/expression contexts `$setup(…)` continues to lex via `system_tf_identifier`
  (`:449`) — the keyword tokens are referenced ONLY by the 12 timing-check rules
  (`:5074-5107`), so no procedural behavior changes.
- **(c) `$unit` / severity / bare-`$` / `$root` (3-profile matrix):** `$unit::y` REJ/REJ/REJ
  vs `sv_dollar_unit::y` ACC/ACC/ACC (host `package_scope:3760`); module-level `$fatal;` /
  `$error;` / `$warning;` / `$info;` all REJ×3 vs mangled ACC×3 — **note the mangled severity
  spellings ACCEPT under `verilog_2005` TODAY** (their hosts `elaboration_system_task_sv_2017`
  (`:2019`) / `severity_system_task_sv_2023` (`:4871`) ride `module_common_item_sv_2017`,
  which is `@profiles ["sv_2017","verilog_2005"]`-admitted — a live v2005 over-acceptance of
  nonsense text, and post-fix the real `$fatal;` would leak into v2005 unless the branch is
  gated in wave 3); bare `x = $;` REJ×3 vs `x = sv_dollar;` ACC×3 (per IEEE 1800 A.8.4
  `primary ::= … $ …`, the fixed literal would make `x = $;` grammatically parseable —
  1800-faithful; semantic restriction to queue-bound contexts is out of parser scope);
  `$root.m.y` / `$root.m.f(1);` ACCEPT in BOTH spellings on ALL 3 profiles — but the LRM
  spelling routes through the generic system-TF token (`kind:"system_tf"`, AST-proven in
  `.6.4`), NOT the `$root`-anchored forms.
- **(d) The wave-3 hazard named — PEG ordered-choice COMMIT:** `system_tf_identifier`
  (`/\$[a-zA-Z0-9_$]+/`) matches `$root`/`$unit` as prefixes; a `primary`/statement alternative
  that succeeds on the system-TF reading COMMITS, and an outer failure (`::`-tail,
  hierarchical tail) backtracks past the WHOLE choice without re-entering later alternatives.
  So merely fixing the `$root`/`$unit` literals may be INERT (the steal continues) or may flip
  accept→reject in tail contexts — every referencing context needs its order proven with
  before→after probes in wave 3, not assumed.
- **(e) `SV-0030` second site:** `init_val` (`:2429`) — UDP probes (both profiles):
  `initial q = 1'b0;` REJ, `initial q = 1'b1;` REJ (the LRM spellings!), `initial q = 1'b;`
  ACC (nonsense), `initial q = 1'bx;` ACC, `initial q = 1;` ACC. Same prefix-merge digit loss
  as `scalar_constant` — the x-forms (`1'bx`/`1'bX`/`1'Bx`/`1'BX`, `:6060-6070`) survived
  because they were not prefix-mergeable. Ledger row `SV-0030` extended with this site.
- **(f) Token naming:** the `kw_*_<hash>` suffix is `sha1(literal)[:8]` (verified:
  `c2543fff`=sha1("this"), `b58bdaae`=sha1("sv_dollar_setup"), `f4c81681`=sha1("1'b")).
  Decision: change literals, KEEP the existing token names — renames would churn the pinned
  cert residual/NO-reach name lists and break exactly the set-diff signal the no-regression
  proofs rely on; the resulting name-hash/literal mismatch is accepted, documented cosmetic
  debt (new tokens introduced by wave 2 DO follow the sha1-of-literal convention).

## Decisions

- `2026-07-02`: fix lands in the flattened `grammars/systemverilog.ebnf` (the maintained
  artifact), NOT via re-running the synthesis pipeline (see Non-Goals). The synthesis tool
  defect is provenance only.
- `2026-07-02`: tree named `SV-DOLLAR-LRM-FIDELITY` to sit beside the existing SV fidelity
  trees; the defect family is ONE mechanism (name-canonicalization leaked into literals) so it
  gets ONE ledger row (`SV-0029`), with the `scalar_constant` content loss as a distinct
  second mechanism (`SV-0030`).
- `2026-07-02` (`.1`): waves re-sliced by RISK CLASS, one concern per commit — `.2` = pure
  literal swap (12 timing checks, no gates, no shape change), `.3` = shape-affecting `SV-0030`
  (digits + reorder ⇒ schema bump), `.4` = SV-only group (PEG-order proofs + `verilog_2005`
  gates). The originally sketched ".2 = timing + SV-0030" bundle was split so every cert/shape
  re-pin has exactly one cause.
- `2026-07-02` (`.1`): token literals change, token NAMES stay — renames would churn the
  pinned residual/NO-reach name lists and destroy the set-diff no-regression signal; the
  sha1-of-literal hash-suffix mismatch is accepted, documented cosmetic debt (wave-2's NEW
  tokens do follow the convention).
- `2026-07-02/03` (`.3`): the compare-branch lhs is a NEW precedence-restricted chain, not the
  full `expression` — tool-proven necessary (the greedy expression lhs consumes `== 1'b0`
  itself; a pure reorder is inert). The operator tier (`binary_operator_above_equality`)
  admits exactly the IEEE 1800-2017 Table 11-2 operators binding TIGHTER than equality, so
  the eq shape is only emitted where it is the precedence-correct reading; a trailing
  `!binary_operator` follow-guard withdraws the compare branch whenever the flat expression
  branch would have consumed more (`e == 1'b0 && f`) — fallback proven byte-identical on 9
  guard probes.
- `2026-07-02/03` (`.3`): new digit tokens are `\b`-guarded REGEXES (not plain strings) so
  `1'b01`/`1'b0x` are not prefix-stolen; scalar_constant's bare `1`/`0` branches carry
  `!tick` follow-guards (else `e == 1'bx` would flip ACCEPT→REJECT via a committed partial
  rhs). `init_val` needs no such guards (rejection of `1'd0`-style continuations is
  LRM-correct there).
- `2026-07-02/03` (`.3`): the `tilde expression` branch stays shadowed (after `expression`)
  DELIBERATELY — promoting it would mis-associate `~e && f` as `~(e && f)`; the flat parse is
  the precedence-correct AST and `~expr ⊆ expression` keeps the language identical. Recorded
  as adjudicated, not an oversight.
- `2026-07-03` (`.4`): the `.1`(c) "v2005 severity leak / wave-3 gates required" hypothesis is
  DISPROVEN by AST dump — mangled `sv_dollar_fatal;` accepts under EVERY profile as an
  ordinary implicit-type variable declaration (identifier reading; control probe: bare `foo;`
  accepts ×3 profiles), NOT through the severity host. Consequence: NO severity profile-gate is
  needed (the hosts' existing `@profiles` tags exclude v2005 — post-fix `$fatal;` REJECTS under
  v2005 naturally), and "mangled must REJECT" is NOT an earnable criterion for the wave-3
  tokens: their mangled spellings are legal identifier text and keep their identifier readings
  (byte-identical ASTs pre↔post, cmp-proven). The criterion refines to: the special-construct
  routing is carried ONLY by the LRM `$` spellings.
- `2026-07-03` (`.4`): a negative lookahead OCCUPIES a positional `$N` slot — codegen-proven
  with a minimal generated parser (the guard is pushed as `element_0`); every annotation on a
  guard-carrying sequence must index past the guards (`system_tf_call` → `$3/$4`;
  `rooted_tf_call_sv_only` → `$2/$3/$4`).
- `2026-07-03` (`.4`): steal-guard placement — only `system_tf_call`'s FIRST (optional-parens)
  alternative needs the `!( $root . ) !( $unit :: )` guards; the parens-mandatory alternatives
  fail naturally on `.`/`::` tails without committing. `$root(…)`/`$unit` (no tail) stay on the
  generic route byte-identically.
- `2026-07-03` (`.4`): TWO trace-diagnosed blockers required follow-up fixes beyond the `.1`
  sketch: (1) `non_typedef_package_scope`'s post predicate and
  `scoped_or_hierarchical_tf_identifier`'s branch predicate drill `$….name.body`, and an
  unresolvable ref is a hard predicate REJECT — fixed DECLARATIVELY by giving the dollar_unit
  lift a load-bearing `name: {body: "$unit"}` literal (resolvable + impossible-identifier ⇒
  `lacks_fact` passes unconditionally, semantically exact). (2) the predicate runtime
  BROADCASTS per-branch directives rule-wide (`branch_predicates_for_rule` flat-maps them —
  `rust/src/ast_pipeline/semantic_runtime.rs:735`), so `scoped_or_hierarchical_tf_identifier`
  branch 3 is predicate-dead for rooted results (`🚫 Branch 3/3 … rejected by branch predicate`,
  trace-proven) — routed AROUND via the new `!( identifier )`-guarded `rooted_tf_call_sv_only`
  carrier (wired into `subroutine_call` + `call_primary`); an engine fix was adjudicated OUT of
  this leaf (un-broadcasting would re-route every identifier-headed hierarchical TF call from
  the method_call machinery to tf_call — a wide shape drift; the broadcast defect is ledgered
  as an open question for its own tree).

## Open Questions

- **Branch-predicate BROADCAST (engine, found in `.4`, tools-proven):**
  `branch_predicates_for_rule` (`rust/src/ast_pipeline/semantic_runtime.rs:735`) flat-maps
  EVERY per-branch directive into the rule-wide set, so a branch-local `@predicate` with
  `phase: branch` is evaluated against every branch of its rule — the compiled per-branch
  registration (`branch_directives_by_rule`, correctly `[[], [pred], []]`) plus the caller's
  redundant `branch_predicates_for_rule_branch` chain say branch-local was the INTENT. Visible
  consequence today: `scoped_or_hierarchical_tf_identifier` branch 3 (hierarchical) is
  predicate-dead in-context (statement-context hierarchical TF calls ride the method_call
  machinery instead). Un-broadcasting is a behavior-visible engine change (those calls would
  re-route tf_call-first ⇒ wide shape drift) — needs its own tree with a full shape-impact
  adjudication; NOT a quick fix.
- Whether `tools/extract_systemverilog_lrm_profiles.py` should be fixed too (only matters on a
  future re-synthesis; does not block the frontier).
- The bare-`$` primary sites (`kw_sv_dollar_04da59ec` at `:2976`/`:2993`/`:4026`/`:4081`) are a
  genuine IEEE 1800 A.8.4 surface (`primary ::= … $ …`), distinct from the correct
  `kw_dollar := /\$/` covergroup/queue-bound sites — the wave-3 literal fix makes `x = $;`
  grammatically parseable (1800-faithful; semantic queue-bound restriction is out of parser
  scope). Resolved in `.1` (c).
- `verilog_2005`-profile nuance: 1364-2005 §15.5.4 allows `$width(controlled_ref, limit)`
  (threshold optional) while our 1800-faithful rule mandates the threshold — a potential
  future v2005-profile arg-shape refinement, NOT a defect in the SV grammar (verified against
  the 1800-2017 section-31 BNF). Does not block any wave.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-02` | (origin) | the `VERILOG-2005-PROFILE.6.4` probe matrix (LRM-vs-mangled × profiles) | recorded in that tree's `.6.4` Findings + Verification Log |
| `2026-07-02` | `.1` | 12 timing checks × {LRM, mangled} × {sv_2017, verilog_2005} = 26 probes (incl. the two `width_min` controls); `$unit`/severity×4/bare-`$`/`$root`×2 × {LRM, mangled} × 3 profiles; UDP `init_val` × 5 digit forms × 2 profiles; `specify_item` alternatives read; severity-host profile-admission cross-checked against the `man_sev_fatal` v2005=ACC probe; `$width` BNF verified in `section-31-timing-checks.txt`; sha1 naming convention verified on 3 samples | all recorded in "`.1` Findings"; zero code |
| `2026-07-02` | `.2` | lint (1450 rules, 0 error classes, `profile_orphans=0`, `always_matches=8` unchanged, rc 0); regen + both binaries rebuilt fresh-mtime; generated-parser literal audit (`\$setup\b` present, rule names unchanged); AFTER matrix: 12/12 LRM ACC + 12/12 mangled REJ (both dialects) + width_min REJ + 3 procedural controls ACC + 8 wave-3 probes unchanged; canonical cert 3 seeds `1328/2/1306/20 spf=0` + residual set-compare vs pre-fix union log `pre==post: True`; union gate GREEN; conformance gate GREEN 168/0 + cert pins EXACT `1138/2/809/327`; shape 18/18; external corpus green; clippy rc 0; other 9 generated parsers mtime-untouched | all green — pins exact, zero re-pins |
| `2026-07-03` | `.4` | BEFORE: 16-probe matrix ×3 profiles + `foo;` control + 24 AST pre-dumps (both dialects) + cert DUMP_ALL baselines (canonical ×3 seeds + v2005 ×1) captured before any edit; lookahead-slot proof (minimal grammar → generated parser: guard pushed as `element_0`); nested-object `name:{body:"$unit"}` annotation codegen-verified; lint post-edit (1463 rules, all error classes 0, rc 0); regen + both binaries ×2 rounds (the mid-leaf trace diagnosis added 2 follow-up fixes); AFTER: 28-probe matrix (all LRM ACC/ACC/REJ, all mangled unchanged), 7 anchor-marker dumps (zero system_tf residue), 24/24 byte-identical cmp controls; entry-rule isolation chain (`package_scope` ACC / `non_typedef_package_scope` REJ pinned the predicate; `scoped_or_hierarchical_tf_identifier` trace named the broadcast); canonical cert 3 seeds `1341/2/1319/20 spf=0` residual set-identical; v2005 cert 3 seeds `1147/4/816/327` set-diff adjudicated; conformance GREEN 192/0 (64 cases); union GREEN re-pinned `1341/2/1319/20` + union `1338/1`; shape 18/18 (31 samples); external corpus 10 non-uvm pass + 4 documented uvm mem-cap rows; clippy rc 0 source-clean; both book gates rc 0; other 9 generated parsers mtime-untouched | all green — canonical residual set-identical; v2005 ratchet honest (bare-`$` → NO-reach, `$root`/`$unit` → PROOF); every contract re-pin has the one `.4` cause |
| `2026-07-02/03` | `.3` | BEFORE matrix (17 probes × 2 profiles) + pre-fix AST dumps (10) + pre-fix cert DUMP_ALL baselines (canonical seeds 0/7/42 + v2005 seed 0) captured BEFORE any edit; lint post-edit (1459 rules, all error classes 0, `always_matches=8`, `profile_orphans=0`, rc 0); regen + both binaries rebuilt; AFTER matrix (UDP digits REJ→ACC ×2 forms ×2 profiles, `1'b` ACC→REJ ×2 profiles, 9 fallback probes ACC with byte-identical ASTs, `e == 1'b` stays REJ); eq/case_eq/ne/'b0 shape dumps (1 `kind:"eq"` node, correct lhs/rhs nesting); entry-rule isolation flip; canonical cert 3 seeds `1337/2/1315/20 spf=0` residual set-identical; v2005 cert `1147/2/819/326` set-diff = `scalar_constant` ratchet only, NO-reach 294 set-identical; conformance gate GREEN 180/0 (60 cases, 4 new locks) + cert pins earned 3 seeds; union gate GREEN re-pinned `1337/2/1315/20` + union `1334/1`; shape gate 18/18 (25 samples, 4 new + 4 dispatch arms); external corpus triage green; clippy rc 0 source-clean (generated-stage debt pre-existing, 0 hits in changed source); other 9 generated parsers mtime-untouched | all green — canonical residual set-identical; v2005 witness ratchet honest (+`scalar_constant`); 3 contract re-pins each with the one `.3` cause |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| (origin) | `PGEN-VERILOG-2005-PROFILE-0017` (`VERILOG-2005-PROFILE.6.4`) | discovery + ledger rows `SV-0029`/`SV-0030`; zero code |
| `.1` | `PGEN-SV-DOLLAR-LRM-FIDELITY-0001` (`SV-DOLLAR-LRM-FIDELITY.1`) | design/audit closed; wave plan re-sliced `.2` literals / `.3` SV-0030 / `.4` SV-only; `SV-0030` extended with the `init_val` site; zero code |
| `.2` | `PGEN-SV-DOLLAR-LRM-FIDELITY-0002` (`SV-DOLLAR-LRM-FIDELITY.2`) | wave 1 LANDED — 12 timing-check literals `sv_dollar_X`→`$X`; release `1.0.159` (schema 13 unchanged); all gates green, pins exact; ledger `SV-0029` → `Fix In Progress` |
| `.3` | `PGEN-SV-DOLLAR-LRM-FIDELITY-0003` (`SV-DOLLAR-LRM-FIDELITY.3`) | wave 2 LANDED — `SV-0030` CLOSED both sites (ten IEEE digit alternatives at `scalar_constant`+`init_val`; compare branches made reachable via the precedence-restricted lhs + follow-guards); release `1.0.160`, schema `13`→`14`; canonical residual set-identical, v2005 `scalar_constant` witness ratchet earned; ledger `SV-0030` → `Released` |
| `.4` | `PGEN-SV-DOLLAR-LRM-FIDELITY-0004` (`SV-DOLLAR-LRM-FIDELITY.4`) | wave 3 LANDED — `SV-0029` CLOSED in full (7 anchor literals + steal guards + 3 profile-gated lifts + the rooted-TF carrier + the resolvable `$unit` name field); release `1.0.161`, schema `14`→`15`; canonical residual set-identical, v2005 `$root`/`$unit` witness→PROOF upgrades; ledger `SV-0029` → `Released`; tree COMPLETE |

## Changelog

- `2026-07-02`: Created task tree from the `VERILOG-2005-PROFILE.6.4` adjudication findings.
- `2026-07-02`: `.1` design/audit leaf closed (`PGEN-SV-DOLLAR-LRM-FIDELITY-0001`); frontier →
  `.2` (wave 1, the 12 timing-check literals).
- `2026-07-02`: `.2` wave 1 landed (`PGEN-SV-DOLLAR-LRM-FIDELITY-0002`, release `1.0.159`);
  frontier → `.3` (wave 2, `SV-0030` digits + reorder).
- `2026-07-03`: `.3` wave 2 landed (`PGEN-SV-DOLLAR-LRM-FIDELITY-0003`, release `1.0.160`,
  schema `14`); `SV-0030` `Released`; frontier → `.4` (wave 3, the SV-only group).
- `2026-07-03`: `.4` wave 3 landed (`PGEN-SV-DOLLAR-LRM-FIDELITY-0004`, release `1.0.161`,
  schema `15`); `SV-0029` `Released`; TREE COMPLETE — both ledger rows closed. The
  branch-predicate-broadcast engine finding stays recorded under Open Questions for a future
  tree.
