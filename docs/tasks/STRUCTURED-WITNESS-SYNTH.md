# STRUCTURED-WITNESS-SYNTH: the dedicated structured-witness synthesizer — close the LAST SV recognized-union UNKNOWN (`context_member_method_call`) ⇒ SV recognized `fully_certified`

## Metadata

- Tree ID: `STRUCTURED-WITNESS-SYNTH`
- Status: `active` (created 2026-07-21, session #189 — the post-RGX-campaign PNT
  frontier selection: the locked program's last gap, chosen over the parked
  deferred backlog and the horizon capability audit)
- Roadmap lane: the **locked program** (director 2026-06-08: all parsers → `Done`,
  `UNKNOWN=0`) + the **director-committed SV endgame** (2026-06-25,
  [[project_sv_full_certification_via_multi_entry]]: "`UNKNOWN=0` IS achievable and
  WILL be achieved"). SystemVerilog is the ONLY shipped grammar not recognized
  `fully_certified`; the recognized multi-config cert union is the accounting
  basis (`GRAMMAR-WELLFORMED.H.12.8.5`, gate `sv_cert_recognized_union_gate`).
- Created: `2026-07-21`
- Owner: repo-local workflow
- Cross-links: owner-of-record for the residual = `GRAMMAR-WELLFORMED.H.12.8.3.2`
  (`blocked-harder` on exactly this capability); lineage = `STORE-AWARE-GEN`
  `.4b.13.1` (`-0021`) / `.4b.18` (`-0027` DESIGN) / `.4b.19` (`-0028`
  ROOT-CAUSE), which twice implemented-then-reverted partial capabilities and
  tool-proved that a **dedicated structured-witness synthesizer** for the
  declare→chain→method-call shape is required.

## Goal (the tree's single deliverable)

Witness `context_member_method_call` in the SV certificate-coverage recognized
union — i.e. earn union `UNKNOWN 1 → 0` — via a **generator-only, parser-agnostic,
capability-gated structured-witness synthesizer**, deterministic across seeds
0/7/42 with `spf=0`, ZERO newly-UNKNOWN rules, and the fully-certified roster
(json / regex / vhdl / svpp / rtl_frontend / rtl_const_expr + the annotation
grammars) byte-identical/inert. That closes the recognized-basis `done_rule`
(`systemverilog_recognized_cert_union_contract.json`): **SV becomes recognized
`fully_certified`**, completing the locked program's certificate axis.

## Background — the pinned baseline and the proven obstacle map

### The regression-locked oracle (as of the last re-baseline, `VERILOG-2005-PROFILE.6.7`, 2026-07-05)

`rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`:
canonical `total=1343 proof=10 witness=1321 UNKNOWN=12`; union
`witness=1332 UNKNOWN=1`, residual exactly `["context_member_method_call"]`;
seeds 0/7/42 byte-identical; `spf=0`. The 12 canonical = the 11-rule
entry-relative library/include/parseable cohort (union-covered) +
`context_member_method_call` (the genuine generation gap).

### The target rule (current grammar, `grammars/systemverilog.ebnf:3132`)

```
context_member_method_call := identifier ( dot identifier constant_bit_select &dot )+ dot callable_method_call_body ( dot method_call_body )*
```

consumed at `call_primary := context_member_method_call -> {kind:
"context_member_method", body: $1}` (`:3185`), store-gated on a
`variable_binding` fact for the head identifier (the `.4b.11.1` finding: the
gate sits on the TARGET rule itself, and its sole producer's dotted emit was
filtered by `emit_name_is_whole_render` — fixed by the proven (b1) producer
admission below).

### What a witness must look like (the `.4b.13.1`/`.4b.19` A/B-proven shape)

`module m; int \foo ; initial … \foo .\bar [0].\baz () …; endmodule` — five
coupled requirements, ALL simultaneously:

1. a **prior TYPED declaration** (`int \foo ;` — untyped `\foo ;` emits count=0)
   registering `variable_binding` for the head name;
2. the declaration must be a **same-scope prior sibling** of the use site
   (obstacle 8 — a struct-member or localparam prelude does NOT bind the
   use-site scope);
3. the use-site expression **reach-forced into `call_primary` alt-0** reliably
   (obstacle 6 — observed `module m; endmodule` fallbacks when the deep forced
   path collapses);
4. the chain tail must take the **parenthesised `callable_method_call_body`
   branch** `… ( list_of_arguments )` (obstacle 7 — without explicit `()` the
   render re-parses as a plain hierarchical/member select and never enters the
   target);
5. the use-site **head identifier pinned to the declared name** (the (c)
   head-only pin — proven mechanism, byte-identical for every whole-render
   cohort consumer).

### What is already tool-proven (the reusable foundation, all reverted-but-recorded)

- **(b1) producer admission** (`rule_leads_with_rule_reference` +
  dotted-`$name.body` resolution) — proven WORKING in `.4b.19`: the name-prelude
  ARMS and injects a prior declaration.
- **(b2) typed-decl forcing** (`data_type_or_implicit → data_type` via the
  min-terminal-length `Some(0)` escape detection) — implemented + verified
  inert-safe in `.4b.19`.
- **(c) head-only name-pin** (`NameGateArm.whole_render` + `pending_head_pin`) —
  implemented + verified inert-safe in `.4b.19`.
- **(a) decl-hosting carrier bias** in `carrier_diversification_candidates` —
  designed in `.4b.18` (the BFS-shortest carrier is a bare top-level
  `attribute_instance` with NO declaration-hosting sibling site — obstacle 4).
- The declarative `@sample`/`@probe_sample` tier was **code-refuted** in
  `.4b.18` (3 reasons) — re-verify that refutation still holds at the current
  vintage before re-adjudicating the fix-hierarchy level.

### Why a fresh re-baseline first

Everything above was proven at the 2026-06-30 vintage (SV ~`1.0.145`,
canonical `UNKNOWN=20`). Since then: SV releases → `1.0.167` (LRM-bracket /
`$`-anchor / digit-alternative / covergroup restorations, `verilog_2005`
profile gating) + the `.6.7` per-profile proof promotion (canonical
`20 → 12`). The obstacle map, rule line numbers, carrier selection, and even
the residual's probe verdict may have moved. Per ⛔ TOOLBOX-FIRST, `.1`
re-derives the WHY+WHERE live before any design.

## Leaves

### `.1` — Live re-baseline + obstacle-map re-verification (tools-first, read-only)

- **Status: `done`** (`PGEN-STRUCTURED-WITNESS-SYNTH-0002`, session #189,
  2026-07-21; READ-ONLY — zero code/grammar/generated change).
- **Baseline re-confirmed byte-exact** (guarded `sv_cert_recognized_union_gate`
  re-run, exit 0, peak 7,695 MB, 655 s): canonical `UNKNOWN=12` / union
  `UNKNOWN=1` / residual exactly `["context_member_method_call"]`,
  deterministic seeds 0/7/42, `spf=0`, `recognized_basis_green=true`. The
  canonical SOLO run (no union configs) reads `total=1343 proof=21
  witness=1321 UNKNOWN=1` — with the single-entry universe the 11
  entry-relative rules are ProfileEntryUnreachable proofs, so the residual is
  THE single UNKNOWN even solo. Evidence: `union_gate_rerun.txt`.
- **Protocol-A probe run** (seed 0, `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` +
  `PGEN_REACH_PATH_DUMP=1`, guarded): verdict `parsed=true
  witnessed_target=false` across ALL FIVE probe families
  (plannable/store-free/target-own/carrier-div ×N); NO probe sample carries a
  prior declaration (the name-prelude never arms at baseline — (b1) is
  reverted) and none renders `()` on the chain tail; carrier-div at this
  vintage tries a RICHER carrier set than the 2026-06-30 map recorded (ANSI
  port default, program-case, package-class-const, named-port, specparam,
  specify-if, bind bit-select, cover-sequence) plus the `module m; endmodule`
  empty fallbacks. BFS reach path: `… attribute_instance → attr_spec →
  constant_expression → … → constant_function_call → call_primary(root/o0) →
  context_member_method_call`. Evidence: `probe_seed0_extract.txt`.
- ⭐ **The A/B matrix SUPERSEDES obstacles 4/6/8** (cells A–F,
  `ab_matrix_current_vintage.txt`, SV parser sha `27d7c67f`): the natural BFS
  attribute carrier WITNESSES the rule once three things are present — (C/E)
  a typed declaration in a PRECEDING top-level unit (minimal form: bare
  `int foo;` — no module wrapper, no same-scope requirement, no carrier
  re-route), the head pinned to the declared name, and the parenthesised
  `call_with_args` tail. (B) no-decl ⇒ gate rejects (the discriminator);
  (F) a localparam prelude parses but does NOT witness — the WRONG producer
  (no `variable_binding` emit): the measured explanation of the `.4b.19`
  armed-but-inert prelude.
- **The `.2` design target (verified):** compose (b1) producer admission
  (re-land) + producer HOSTING at a top-level data-declaration position
  (F-refuted: never localparam) + (b2) typed-branch forcing + (c) head-leaf
  name-pin + NEW forced `call_with_args` branch on the target's own
  `callable_method_call_body` tail — on the EXISTING carrier.
- Acceptance checklist (investigation slice):
  - [x] REPRODUCE — gate green byte-exact + solo-canonical `1343/21/1321/1`;
    probe verdict current.
  - [x] ROOT CAUSE (WHY+WHERE) — the A–F matrix + probe extract, all
    tool-backed at the current vintage (pins `:3118`/`:3132`/`:3184`/`:3026`/`:5917`).
  - [x] ADDRESSED — N/A (read-only slice; the design target above is the output).
  - [x] NO REGRESSION — read-only; no artifact/source touched (the gate's own
    `focus_systemverilog` regen is its maintained recipe; parser sha banked).
  - [x] LOCKSTEP — this leaf + frontier + `docs/TASK_TREE.md` row + MEMORY +
    CHANGES; no book/contract change (no user-facing behavior change).

### `.2` — DESIGN: the structured-witness synthesizer (PURE-DOCS)

- **Status: `done`** (`PGEN-STRUCTURED-WITNESS-SYNTH-0003`, session #189,
  2026-07-21). Design note:
  [STRUCTURED-WITNESS-SYNTH-2-design.md](STRUCTURED-WITNESS-SYNTH-2-design.md).
- **Root cause of the standing miss (code-grounded):** the generator already
  owns machinery for EACH of the three witness conditions — prelude arming
  (`compute_name_prelude` `:3286` + the `.6.3.2` integrity check `:10953`,
  NEWER than the `.4b.19` attempt), use-site name replay
  (`reach_prelude_replay_text` `:3957` / `store_name_for_gate` `:3650`), and
  target-own structure forcing (`generate_target_own_structure_witnesses`
  `:4507` + `mandatory_child_rules` `:7221`) — but in SEPARATE passes that
  never compose into one sample; and the prelude never arms for this gate
  because `emit_name_is_whole_render` (`:3405`) rejects the producer's DOTTED
  `$name.body` emit.
- **Architecture: PASS 3f `generate_structured_witnesses`** — residual-only,
  runs LAST, composes (f1) the (b1) producer-admission relaxation (scoped to
  3f) + (f2) prelude with (b2) typed-branch forcing + (f3) the (c) head-leaf
  pin for multi-token consumers + (f4) the `-0090`/`-0093` target-own
  directives merged into the SAME plan + (f5) the parser as sole witness
  judge. Parser-agnostic (gates/producers/shape all grammar-derived),
  capability-gated (structural no-op when `gen_name_gate` empty / nothing
  residual), bounded by the existing witness budgets.
- Fix-hierarchy re-adjudicated at this vintage: annotation tier still
  code-refuted (names must coordinate across two generation-time statements);
  store side complete (cells B/E/F behave exactly per the gate/producer);
  landing level = Level-3+ parser-agnostic generator capability; NO grammar
  change ⇒ no release/schema/ledger bump.
- Predicted before→after (the `.3` bar): solo `UNKNOWN 1→0`; gate accounting
  canonical `12→11` witness `1321→1322`, union `1→0` witness `1332→1333`,
  residual `[]` ⇒ SV recognized `fully_certified`. Bisection order with a
  LOUD tool signal per step recorded in the design note.

### `.3` — IMPLEMENT (generator-only, measured GLOBALLY, keep only on improvement)

- **Status: `done`** (`PGEN-STRUCTURED-WITNESS-SYNTH-0004`, session #189, 2026-07-21;
  generator-only code slice — `rust/src/ast_pipeline/stimuli_generator.rs` +
  the PASS 3f driver block in `rust/src/main.rs`; NO grammar/parser/release/
  schema/ledger change).
- **THE RULE IS WITNESSED — solo canonical `UNKNOWN 1→0`.** PASS 3f
  `generate_structured_witnesses` landed exactly per the `.2` architecture:
  (f1) pass-scoped dotted-emit producer admission (`structured_producer_admitted`
  + emit-time resolution `leading_rendered_token_with_terminator`) → (f2)
  typed-branch forcing on the prelude sub-path
  (`typed_branch_directives_along_subpath`) → (f3) head-leaf pin for MULTI-token
  gated consumers (`NameGateArm.whole_render` discriminator +
  `pending_head_pin`, consumed before the literal-hint route so collide-aware
  renaming cannot move the head off the declared name) → (f4) target-own
  root-Or/quantifier/mandatory-child directives merged into the SAME plan →
  (f5) parser = sole judge. Everything scoped behind `structured_witness_mode`
  (true only inside the pass).
- **One measured iteration on the bisection ladder (step 5, attribution):** the
  first probe run armed+injected+pinned but rendered the pinned head as `\foo.`
  — `split_whitespace()` had STRIPPED the escaped identifier's mandatory
  trailing-space terminator, fusing head+dot+member into ONE token, so the gate
  rejected. Fix: the resolved emit name keeps the producer render's ONE
  terminating whitespace char. Second run WITNESSED on the first 3f probe.
- Acceptance checklist:
  - [x] REPRODUCE — the `.1` baseline: solo `1343/21/1321/1`, residual exactly
    `context_member_method_call`, all five prior probe families
    `parsed=true witnessed_target=false`.
  - [x] ROOT CAUSE (WHY+WHERE) — the `.2` composition root cause, plus the
    measured iteration-1 fused-head token (probe log, WHY = stripped lexical
    terminator, WHERE = the (f1) emit-time name resolution).
  - [x] ADDRESSED — seeds 0/7/42 canonical solo ALL read
    `total=1343 proof=21 witness=1322 UNKNOWN=0 fully_certified=true spf=0`
    (pass line: 1 targeted / 1 witnessed, every seed). Witness sample:
    `(*\foo =type(struct{bit\foo ;})*)(*\foo_0 =\foo .\foo_0 .\foo_0 .\foo_0 *)bind\foo_0 \foo_0 \foo_0 ();`
  - [x] NO REGRESSION — ZERO newly-UNKNOWN (solo UNKNOWN=0; proof unchanged);
    fully-certified roster BYTE-COMPARED baseline↔candidate via git-stash A/B
    (json/regex/vhdl/svpp/rtl_frontend/rtl_const_expr cert reports ALL
    BYTE-IDENTICAL); full dual-feature `cargo test --lib` **1013/0/29** green
    (+5 new lock tests: producer admission scoped to the pass, terminator-keeping
    leading token, typed-branch first-typed-only forcing, single-leading-token
    classification, head-pin single-shot + plan-scoped clearing); clippy flow
    green (source-strict; generated-stage stays the tracked non-strict 290);
    `sv_stimuli_quality_gate` + `stimuli_cross_family_platform_gate` PASS.
  - [x] LOCKSTEP — this leaf + frontier + `docs/TASK_TREE.md` + MEMORY +
    CHANGES + DEVELOPMENT_NOTES; evidence
    `docs/tasks/artifacts/structured_witness_synth/pass3f_seed0_witness.log` +
    `pass3f_verification_summary.txt`. Union-gate contract re-baseline + book/
    LIVE/contract recognition = `.4` (the immediately-following commit of the
    same wave).

### `.4` — VERIFY + recognition lockstep

- Re-baseline `systemverilog_recognized_cert_union_contract.json`
  (`expected_union_unknown 1→0`, `expected_union_witness 1332→1333`,
  `expected_union_residual_rules []`, `expected_canonical_unknown 12→11`,
  `expected_canonical_witness 1321→1322`) + `sv_cert_recognized_union_gate`
  green; `docs/book/src/grammar-wellformedness.md` recognized-basis section →
  `UNKNOWN=0` / SV recognized `fully_certified`; `LIVE_ACHIEVEMENT_STATUS.md`;
  the SV integration contract honest-trust figure; MEMORY / CHANGES /
  DEVELOPMENT_NOTES; `GRAMMAR-WELLFORMED.H.12.8.3.2` row → `done`;
  `STORE-AWARE-GEN` deferral note annotated (capability delivered here).

## Acceptance Criteria (tree)

1. SV recognized union `UNKNOWN=0` (residual `[]`), deterministic seeds 0/7/42,
   `spf=0` — the contract's `done_rule` satisfied ⇒ SV recognized
   `fully_certified`.
2. ZERO newly-UNKNOWN anywhere; the fully-certified roster inert
   (byte-identical cert output).
3. The capability is parser-agnostic + capability-gated
   ([[feedback_ast_pipeline_parser_agnostic]],
   [[feedback_features_parser_agnostic_enable_all_parsers]]).
4. All gates green (union gate re-baselined, lib tests, clippy source-strict,
   cross-family stimuli gate); lockstep docs complete.
5. Every slice carries tool-backed WHY+WHERE + measured before→after in its
   leaf (the enforced acceptance checklist).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `STRUCTURED-WITNESS-SYNTH.4` (VERIFY + recognition lockstep) | `next` | `.3` witnessed the rule (solo `UNKNOWN=0` ×3 seeds); re-baseline the union contract (union `1→0`, residual `[]`), run `sv_cert_recognized_union_gate` green, and land the recognition lockstep (book / LIVE / SV contract / `H.12.8.3.2` row / STORE-AWARE-GEN annotation). |
| — | `STRUCTURED-WITNESS-SYNTH.1` (live re-baseline + obstacle re-verification) | `done` (`PGEN-STRUCTURED-WITNESS-SYNTH-0002`, read-only) | Baseline byte-exact (canonical 12 / union 1 / residual = the rule); A–F matrix superseded obstacles 4/6/8; the `.2` design target verified. |
| — | `STRUCTURED-WITNESS-SYNTH.2` (DESIGN) | `done` (`PGEN-STRUCTURED-WITNESS-SYNTH-0003`, PURE-DOCS) | PASS 3f composition architecture, code-grounded; predicted union `1→0`. |
| — | `STRUCTURED-WITNESS-SYNTH.3` (IMPLEMENT PASS 3f) | `done` (`PGEN-STRUCTURED-WITNESS-SYNTH-0004`) | THE RULE IS WITNESSED — solo canonical `1343/21/1322/0 fully_certified` seeds 0/7/42; roster byte-inert (A/B); lib 1013/0/29; clippy + stimuli gates green. |

## Log

- `2026-07-21` (session #189): Tree created as the post-RGX-campaign PNT frontier
  selection (MEMORY frontier: deferred backlog / SV cert / horizon audit —
  SV cert chosen: the locked program's last gap, director-committed, with a
  twice-tool-proven concrete blocker). Union-gate re-baseline run launched
  (guarded) as `.1`'s first evidence.
- `2026-07-21` (session #189, `.1` CLOSED `PGEN-STRUCTURED-WITNESS-SYNTH-0002`):
  baseline re-confirmed byte-exact; Protocol-A probe run banked; ⭐ the A–F
  parse matrix (cells A/B/C/D/E/F) SUPERSEDED obstacles 4/6/8 — the natural
  BFS attribute carrier witnesses with a minimal PRECEDING-UNIT typed-decl
  prelude (`int foo;`), a pinned head, and the parenthesised tail; the
  localparam prelude is F-REFUTED (wrong producer — the measured explanation
  of the `.4b.19` inertness). Evidence: `union_gate_rerun.txt` +
  `probe_seed0_extract.txt` + `ab_matrix_current_vintage.txt` under
  `docs/tasks/artifacts/structured_witness_synth/`.
- `2026-07-21` (session #189, `.2` CLOSED `PGEN-STRUCTURED-WITNESS-SYNTH-0003`,
  PURE-DOCS): the PASS 3f composition design written code-grounded
  ([STRUCTURED-WITNESS-SYNTH-2-design.md](STRUCTURED-WITNESS-SYNTH-2-design.md)) —
  root cause of the standing miss = the three witness conditions live in
  SEPARATE passes that never compose, and the prelude never arms because
  `emit_name_is_whole_render` rejects the dotted producer emit. Frontier →
  `.3` IMPLEMENT (fresh focused session recommended — code slice + heavy
  cert battery).
- `2026-07-21` (session #189 continuation, `.3` CLOSED
  `PGEN-STRUCTURED-WITNESS-SYNTH-0004`): ⭐ **`context_member_method_call` IS
  WITNESSED — the LAST SV generation gap closes.** PASS 3f landed per design;
  ONE measured bisection iteration (the fused-head token: the emit-time name
  resolution had stripped the escaped identifier's lexical terminator; fixed by
  `leading_rendered_token_with_terminator`); second run witnessed on the FIRST
  3f probe. Solo canonical seeds 0/7/42 ALL `1343/21/1322/0 fully_certified
  spf=0`; roster A/B byte-identical ×6; lib 1013/0/29; clippy + both stimuli
  gates green. Frontier → `.4` (union-contract re-baseline + recognition
  lockstep, same wave).
