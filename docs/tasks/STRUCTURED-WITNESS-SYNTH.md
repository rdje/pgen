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

- Re-adjudicate the fix-hierarchy level first ([[feedback_no_workarounds_fix_hierarchy]]):
  (1) annotation tier (re-check the `.4b.18` code-refutation), (2) existing
  store, (3) new annotation, (4) new store capability, (5) engine — the
  expected landing level is a Level-3+ parser-agnostic generator capability,
  but the adjudication must be re-earned at the current vintage.
- Architecture: ONE coherent synthesizer pass composing the five requirements
  (decl-hosting carrier selection + same-scope sibling prelude + typed-decl
  forcing + reach-forcing into the target's parent alt + mandatory-branch
  forcing for the disambiguating `()` + head name-pin), generalizing the
  proven (a)/(b1)/(b2)/(c) parts — **parser-agnostic** (no rule-name/sigil
  hardcoding; capability-gated so every other grammar is byte-inert),
  the `.4b.19` lesson honored: parts in isolation are inert, land + measure
  as ONE capability.
- Deliverable: code-grounded design note (current sources cited), predicted
  before→after (union `1→0`, canonical `12→11`, witness `+1`), inertness plan,
  bisection order if it does not close.

### `.3` — IMPLEMENT (generator-only, measured GLOBALLY, keep only on improvement)

- Land the synthesizer per `.2`; measure the GLOBAL cert (canonical + union,
  seeds 0/7/42); keep ONLY on improvement ([[project_cert_coverage_tournament_loser_leak]]);
  ZERO newly-UNKNOWN; `spf=0`; fully-certified roster byte-identical/inert;
  `cargo test --lib` green (+ new lock tests); clippy source-clean;
  `sv_stimuli_quality_gate` + `stimuli_cross_family_platform_gate` PASS.
- Generator-only ⇒ NO release/schema/ledger bump (no parser accepted-language
  change). If the metric does not move: root-cause with the toolbox, bank the
  finding, revert, and extend the obstacle map honestly — the director-committed
  goal stands, so the next design iteration starts from the enriched map.

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
| 1 | `STRUCTURED-WITNESS-SYNTH.2` (DESIGN) | `next` | `.1` verified the witness template (cell E) and SUPERSEDED the fragile parts of the old map — the design is now a composition of proven parts + one new tail-branch forcing, on the existing carrier. |
| — | `STRUCTURED-WITNESS-SYNTH.1` (live re-baseline + obstacle re-verification) | `done` (`PGEN-STRUCTURED-WITNESS-SYNTH-0002`, read-only) | Baseline byte-exact (canonical 12 / union 1 / residual = the rule); A–F matrix superseded obstacles 4/6/8; the `.2` design target verified. |
| 3 | `STRUCTURED-WITNESS-SYNTH.3` (IMPLEMENT) | `pending` | Blocked on `.2`. |
| 4 | `STRUCTURED-WITNESS-SYNTH.4` (VERIFY + lockstep) | `pending` | Blocked on `.3`. |

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
